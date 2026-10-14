//! B1 / B4（C554 ① 与机理那一形）：P（被抛弃那一版）根槽落了、系统配置没轮换就崩；下一次可写挂载（实例 2）时 P 的一组落点一时读回全零
//! （`HIDE`），恢复落到 S、实例 2 抛弃 P；枚举实例 2 那次挂载写流的每个崩溃状态，每个状态撤故障，两块盘都在 / 丢盘 0 / 丢盘 1 各恢复一次，
//! 看落到哪条根、读回什么。外层先把 P 那次发布写流的每个崩溃状态都过一遍（可写挂载带同一组故障，看它抛弃不抛弃 P）。
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::Instant;

use singlefs_checker_tier::crash::{
    full_expansion, layer0_state_count_with_torn_in_place_overwrites, Layer0ObserverCounts,
    Layer0VersionTable,
};
use singlefs_checker_tier::layer0_enumeration::{enumerate_layer0_with_expansion, Layer0Item, Layer0Observer};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_chain_of_root, root_is_abandoned_by_the_instance_table,
    scan_journal, RecoveryReport,
};
use singlefs_core::mounted_session::UserChange;
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{FirstFile, TransactionOutput};
use singlefs_harness::memory_pool::{writes_and_segments, CrashImage, MemoryPool, PublishedVersion};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

use crate::pool::{content_of, fixed_geometry_of_the_384_slot_small_devices, OVERWRITE_BYTES};
use crate::util::*;

/// 设点文件只用来取结论版本与节点身份：库只认 checker 档包 tests/ 下的候选文件，这里借一份现成的（判定库在草稿目录里、每趟清空，不与别处共用）。
const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs";

pub type Key = (InstanceGeneration, CheckpointTxg);

/// 一段历史的起点：S 之前 k 次覆盖写，P 那次发布的写流，P 之前的镜像。
pub struct Start {
    pub base: MemoryPool,
    pub p_ops: Vec<singlefs_harness::RetainedOperation>,
    pub s: TransactionOutput,
    pub p: TransactionOutput,
    pub contents: BTreeMap<Key, Vec<u8>>,
}

pub fn start(k: u64, p_bytes: usize) -> Start {
    let stream = SharedStream::retaining_contents();
    let mut pool = start_pool(&stream);
    for _ in 0..k {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
    }
    let s = pool.current_file_version().clone();
    let before = stream.retained_operations().len();
    pool.overwrite(p_bytes).expect("P");
    let p = pool.current_file_version().clone();
    let ops = stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], pool.device_bytes);
    base.apply(&ops[..before]);
    Start { base, p_ops: ops[before..].to_vec(), s, p, contents: pool.content_of_each_root.clone() }
}

pub fn key(root: &RootRecord) -> Key {
    (root.instance, root.checkpoint_txg)
}

/// 这一格读回全零的那组落点。
pub fn hide_ranges(hide: &str, p: &TransactionOutput) -> Vec<Range> {
    if hide == "nofault" {
        return Vec::new();
    }
    let mut ranges = vec![root_slot_range(p.root.checkpoint_txg)];
    for part in hide.split('+') {
        match part {
            "root" => {}
            "unit" => ranges.extend(data_unit_ranges(p)),
            "rec" => ranges.extend(record_ranges(p)),
            "acct" => ranges.extend(account_ranges(p)),
            "new" => ranges.extend(new_unit_ranges(p)),
            "scnew" => {}
            other => panic!("HIDE 部件 {other:?} 认不得：root unit rec acct new，用 + 连"),
        }
    }
    ranges
}

pub fn versions_of(contents: &BTreeMap<Key, Vec<u8>>) -> Vec<PublishedVersion> {
    contents
        .iter()
        .map(|((instance, checkpoint_txg), content)| PublishedVersion {
            instance: *instance,
            checkpoint_txg: *checkpoint_txg,
            content: content.clone(),
        })
        .collect()
}

pub fn last_root_write(writes: &[singlefs_harness::memory_pool::RetainedWrite]) -> usize {
    writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("流里有根槽写")
}

/// 实例 2 那次可写挂载（带读回全零的故障）做完之后的样子。
pub struct Mount2 {
    pub ops: Vec<singlefs_harness::RetainedOperation>,
    pub contents: BTreeMap<Key, Vec<u8>>,
    pub effective: Key,
    pub isolated: Vec<(DeviceIdentity, u64)>,
}

/// 在 `crashed` 上可写挂载一次，挂载期间 `hide` 那几段读回全零；做成之后会话里再覆盖写 `overwrites` 次（故障照旧在）。
pub fn mount2(
    crashed: &MemoryPool,
    hide: &[Range],
    contents: &BTreeMap<Key, Vec<u8>>,
    overwrites: u64,
) -> Result<Mount2, String> {
    let stream = SharedStream::retaining_contents();
    let zeros = Rc::new(RefCell::new((hide.to_vec(), false)));
    let mut devices = faulty_recorded_devices(crashed, &stream, &zeros);
    let mounted = mount_writable(&params(), &mut devices).map_err(|e| format!("{e:?}"))?;
    let (output, mut session) = MountedSession::of_the_mount(mounted);
    let mut contents = contents.clone();
    let effective = key(&output.effective_root);
    let mut current = contents.get(&effective).cloned().unwrap_or_default();
    for root in std::iter::once(&output.row_publish)
        .chain(&output.warm_up_publishes)
        .map(|v| *v.root())
        .chain(output.space_admission.floor_raises().iter().flat_map(|r| r.publishes.iter().map(|p| p.root)))
    {
        contents.insert(key(&root), current.clone());
    }
    for i in 0..overwrites {
        let c = content_of(OVERWRITE_BYTES, 9000 + i);
        let published = session
            .publish_user_change(
                &mut devices,
                UserChange::Overwrite(FirstFile { content: &c, write_time_seconds: 1_789_000_000 + i }),
            )
            .map_err(|e| format!("session_overwrite_{i}: {e:?}"))?;
        for r in published.floor_raises.iter().flat_map(|r| r.publishes.iter()) {
            contents.insert(key(&r.root), current.clone());
        }
        current = c;
        contents.insert(key(session.current.root()), current.clone());
    }
    Ok(Mount2 { ops: stream.retained_operations(), contents, effective, isolated: output.isolated_slots_per_device })
}

/// 计数名只许小写字母、数字、_。
pub fn counter_name(text: &str) -> String {
    let head: String = text.chars().take(160).collect();
    let mut out = String::new();
    for c in head.chars() {
        if c.is_ascii_uppercase() {
            if !out.is_empty() {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else if c.is_ascii_alphanumeric() {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    let mut collapsed = String::new();
    for c in out.chars() {
        if c == '_' && collapsed.ends_with('_') {
            continue;
        }
        collapsed.push(c);
    }
    collapsed.trim_matches('_').chars().take(90).collect()
}

/// 丢一块盘之后剩下那块盘上，看不看得到判别「P 被抛弃」的东西：系统配置里取过号的实例代号比 P 的新；
/// 环里有一条比 P 新的实例写的记录，按它的新根段读得出实例表、那张表判 P 被抛弃。
pub fn discriminator_on(survivor: &MemoryPool, p: &RootRecord) -> (bool, bool) {
    let Ok(sc) = choose_system_configuration(survivor) else { return (false, false) };
    let newer_instance = sc.quantities.journal_instance > p.instance;
    let records = scan_journal(survivor, &sc);
    let table_abandons = records.values().filter(|r| r.instance > p.instance).any(|r| {
        let assembled = RootRecord {
            filesystem_identifier: p.filesystem_identifier,
            unmount_marker: p.unmount_marker,
            instance: r.instance,
            checkpoint_txg: r.checkpoint_txg,
            tree_table: r.new_tree_table,
            tree_identifier_watermark: r.new_tree_identifier_watermark,
            rollback_floor: r.new_rollback_floor,
            instance_table: r.new_instance_table,
            mapping_root: r.new_mapping_root,
            allocation_record_tree_root: r.new_allocation_record_tree_root,
        };
        instance_table_chain_of_root(survivor, &assembled)
            .map(|chain| root_is_abandoned_by_the_instance_table(p, &chain.records))
            .unwrap_or(false)
    });
    (newer_instance, table_abandons)
}

/// 照崩溃点重放的枚举域枚举一条流（每一段都展开、原地覆写三态），挂一个观察者；先按层 0 口径算状态数，超过 10⁶ 不跑。
pub fn enumerate(
    name: &str,
    base: &MemoryPool,
    writes: &[singlefs_harness::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    versions: &[PublishedVersion],
    observe: &(dyn Fn(u64, &CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts) + Sync),
) -> Option<(u64, singlefs_checker_tier::crash::Layer0Tally)> {
    let states = layer0_state_count_with_torn_in_place_overwrites(base, writes, segments, &full_expansion);
    let closed = singlefs_harness::memory_pool::closed_form_state_count(segments);
    if states > 1_000_000 {
        println!("OPUS-SKIP name={name} states={states} closed_form={closed} reason=over_1e6");
        return None;
    }
    let started = Instant::now();
    let root_index = last_root_write(writes);
    let tally = enumerate_layer0_with_expansion(
        Layer0Item { name, setting_file: THIS_FILE },
        base,
        writes,
        segments,
        root_index,
        Layer0VersionTable::with_a_file_only(versions),
        &full_expansion,
        &[Layer0Observer { name: "opus_observer", extra_inputs_digest: [0; 32], observe }],
    )
    .judged()
    .expect("单机：整份判完");
    let counts: Vec<String> = tally.observer_counts.iter().map(|(n, c)| format!("{n}={c}")).collect();
    println!(
        "OPUS-TALLY name={name} states={states} closed_form={closed} enumerated={} observed={} oracle_violations={} checker_violated={:?} secs={:.1} counts=[{}]",
        tally.states,
        tally.observed_states,
        tally.violations,
        tally.checker_violated_states,
        started.elapsed().as_secs_f64(),
        counts.join(" ")
    );
    Some((states, tally))
}

pub fn main_b1() {
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    let ks: Vec<u64> = env_list("K_LIST", "2,5").iter().map(|s| s.parse().expect("K")).collect();
    let hides = env_list("HIDE_LIST", "unit,rec,rec+unit,acct");
    let p_bytes = env_u64("P_BYTES", OVERWRITE_BYTES as u64) as usize;
    let m2_overwrites = env_u64("M2_OVERWRITES", 0);
    let outer = env_u64("OUTER", 1) == 1;
    let mut total_states = 0u64;
    for k in &ks {
        for hide in &hides {
            let st = start(*k, p_bytes);
            let p_key = key(&st.p.root);
            let hide_r = hide_ranges(hide, &st.p);
            let (w, seg) = writes_and_segments(&st.p_ops, &geometry);
            let tag = format!("b1_k{k}_{}_pb{p_bytes}_m{m2_overwrites}", hide.replace('+', "_"));
            println!("OPUS-B1-START tag={tag} s={:?} p={:?} hide_ranges={}", key(&st.s.root), p_key, hide_r.len());
            if outer {
                let observe = |_o: u64, image: &CrashImage<'_>, _r: &RecoveryReport, counts: &mut Layer0ObserverCounts| {
                    let img = materialize(image.base, image.writes, &image.persisted);
                    let name = match mount2(&img, &hide_r, &st.contents, 0) {
                        Ok(m) if m.effective == p_key => "m2_eff_p".to_string(),
                        Ok(m) if m.effective == key(&st.s.root) => "m2_eff_s".to_string(),
                        Ok(_) => "m2_eff_other".to_string(),
                        Err(e) => format!("m2_refused_{}", counter_name(&e)),
                    };
                    counts.add(&name, 1);
                };
                if let Some((n, _)) = enumerate(&format!("{tag}_outer"), &st.base, &w, &seg, &versions_of(&st.contents), &observe) {
                    total_states += n;
                }
            }
            let root_w = last_root_write(&w);
            let persisted: Vec<bool> = (0..w.len()).map(|i| i <= root_w).collect();
            let crashed = materialize(&st.base, &w, &persisted);
            let m2 = match mount2(&crashed, &hide_r, &st.contents, m2_overwrites) {
                Ok(m) => m,
                Err(e) => {
                    println!("OPUS-B1-M2 tag={tag} refused={e}");
                    continue;
                }
            };
            println!("OPUS-B1-M2 tag={tag} effective={:?} isolated={:?} ops={}", m2.effective, m2.isolated, m2.ops.len());
            let (w2, seg2) = writes_and_segments(&m2.ops, &geometry);
            let contents2 = m2.contents.clone();
            let observe = |_o: u64, image: &CrashImage<'_>, _r: &RecoveryReport, counts: &mut Layer0ObserverCounts| {
                let img = materialize(image.base, image.writes, &image.persisted);
                // 机理那一形（射程「挂着时回退的已知边角」②）：撤故障之后新实例（实例代号 > P 的）写的根全改坏（根槽清零），两块盘都在，只读挂载。
                {
                    let mut newbad = img.clone();
                    if let Ok(sc) = choose_system_configuration(&img) {
                        for root in singlefs_core::recovery::readable_roots(&img, &sc.immutable.region_devices, &sc.immutable.sizes, &sc.immutable.filesystem_identifier) {
                            if root.instance > st.p.root.instance {
                                let (d, off, len) = root_slot_range(root.checkpoint_txg);
                                newbad.devices.get_mut(&d).expect("盘").write(singlefs_core::address::DeviceOffsetInBytes(off), &vec![0u8; len as usize]);
                            }
                        }
                    }
                    let (root, result) = read_only(&newbad);
                    let cls = classify(root, &result, p_key, &contents2);
                    counts.add(&format!("newbad_ro_{cls}"), 1);
                }
                for (tag, lost) in [("both", None), ("lost0", Some(0)), ("lost1", Some(1))] {
                    let survivor = image_without(&img, lost);
                    for (path, (root, result)) in [("rec", recovered(&survivor)), ("ro", read_only(&survivor))] {
                        let cls = classify(root, &result, p_key, &contents2);
                        counts.add(&format!("{tag}_{path}_{cls}"), 1);
                        if let Err(e) = &result {
                            if cls.ends_with("failed") {
                                counts.add(&format!("{tag}_{path}_{cls}_{}", counter_name(e)), 1);
                            }
                        }
                        if path == "ro" && (cls.ends_with("failed") || cls.ends_with("read_wrong") || cls == "abandoned_read_ok") {
                            let (sc_newer, table) = discriminator_on(&survivor, &st.p.root);
                            counts.add(&format!("{tag}_ro_{cls}_sc_newer_{sc_newer}_table_{table}"), 1);
                        }
                    }
                }
            };
            if let Some((n, _)) = enumerate(&format!("{tag}_inner"), &crashed, &w2, &seg2, &versions_of(&contents2), &observe) {
                total_states += n;
            }
            println!("OPUS-B1-CUMULATIVE states={total_states}");
        }
    }
}

/// 不枚举的快扫：每个 (k, HIDE) 只做到实例 2 那次挂载交回，数它的写盖到了 P 新分配的几个槽，报 P 与 S 的根各在哪块盘。
pub fn main_b1scan() {
    let ks: Vec<u64> = env_list("K_LIST", "2,3,4,5,6,7,8,9,10,11,12,13,14").iter().map(|s| s.parse().expect("K")).collect();
    let hides = env_list("HIDE_LIST", "rec");
    let p_bytes = env_u64("P_BYTES", OVERWRITE_BYTES as u64) as usize;
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    for k in &ks {
        for hide in &hides {
            let st = start(*k, p_bytes);
            let hide_r = hide_ranges(hide, &st.p);
            let (w, _seg) = writes_and_segments(&st.p_ops, &geometry);
            let root_w = last_root_write(&w);
            let persisted: Vec<bool> = (0..w.len()).map(|i| i <= root_w).collect();
            let crashed = materialize(&st.base, &w, &persisted);
            let p_new = new_unit_ranges(&st.p);
            let outcome = mount2(&crashed, &hide_r, &st.contents, env_u64("M2_OVERWRITES", 0));
            let (overlap, eff) = match &outcome {
                Ok(m) => {
                    let (w2, _) = writes_and_segments(&m.ops, &geometry);
                    let hit = p_new
                        .iter()
                        .filter(|(d, start, len)| {
                            w2.iter().any(|x| {
                                x.kind == StepKind::UnitWrite
                                    && x.device == *d
                                    && x.offset.0 < start + len
                                    && *start < x.offset.0 + x.length_in_bytes()
                            })
                        })
                        .count();
                    (hit, format!("{:?}", m.effective))
                }
                Err(e) => (0, format!("refused:{}", counter_name(e))),
            };
            println!(
                "OPUS-B1SCAN k={k} hide={hide} s_txg={} s_dev={:?} p_txg={} p_dev={:?} p_new_placements={} overlapped_by_mount2_unit_writes={overlap} m2={eff}",
                st.s.root.checkpoint_txg.0,
                root_slot_range(st.s.root.checkpoint_txg).0,
                st.p.root.checkpoint_txg.0,
                root_slot_range(st.p.root.checkpoint_txg).0,
                p_new.len()
            );
        }
    }
}

/// 每块盘世代号最大的那个自证过的系统配置槽（整槽）。
pub fn newest_system_configuration_ranges(image: &MemoryPool) -> Vec<Range> {
    let spacing = u64::from(params().geometry.fixed_structure_slot_spacing);
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .filter_map(|device| {
            let slots = singlefs_core::recovery::verified_system_configuration_slots(image, device, spacing, &params().filesystem_identifier);
            slots
                .iter()
                .map(|s| s.quantities.slot_generation)
                .max()
                .map(|g| (device, (g % 2) * spacing, singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES))
        })
        .collect()
}

/// B4：P 整次发布做完、系统配置轮换也落了（fsync 返回过，见证过）。进程退出之后下一次可写挂载时 P 的根槽、P 的记录与每块盘较新的那个系统配置槽
/// 一时读回全零（`HIDE` 另可加 unit / new）：c_见证 落回 P 之前，P 被当没见证过、被抛弃。看实例 2 写行取的 txg、P 的根槽被不被盖掉；
/// 撤故障之后两块盘都在只读挂载读回哪一版；再把实例 2 的根全改坏读回哪一版。`OPUS_B4_ARM=skip_once` 是「避开读不出的槽对应的 txg」那一臂。
pub fn main_b4() {
    let ks: Vec<u64> = env_list("K_LIST", "2,3,4,5,10").iter().map(|s| s.parse().expect("K")).collect();
    let extra = env_list("HIDE_LIST", "rec");
    for k in &ks {
        for hide in &extra {
            let st = start(*k, OVERWRITE_BYTES);
            let mut full = st.base.clone();
            full.apply(&st.p_ops);
            let mut hide_r = hide_ranges(hide, &st.p);
            hide_r.extend(newest_system_configuration_ranges(&full));
            let p_key = key(&st.p.root);
            let p_slot = root_slot_range(st.p.root.checkpoint_txg);
            let p_slot_bytes = full.devices[&p_slot.0].read(singlefs_core::address::DeviceOffsetInBytes(p_slot.1), p_slot.2 as usize);
            match mount2(&full, &hide_r, &st.contents, env_u64("M2_OVERWRITES", 0)) {
                Err(e) => println!("OPUS-B4 k={k} hide={hide} arm={:?} mount2_refused={}", std::env::var("OPUS_B4_ARM").unwrap_or_default(), counter_name(&e)),
                Ok(m) => {
                    let mut after = full.clone();
                    after.apply(&m.ops);
                    let p_slot_kept = after.devices[&p_slot.0].read(singlefs_core::address::DeviceOffsetInBytes(p_slot.1), p_slot.2 as usize) == p_slot_bytes;
                    let new_txgs: Vec<u64> = m.contents.keys().filter(|(i, _)| i.0 >= 2).map(|(_, t)| t.0).collect();
                    let (root, result) = read_only(&after);
                    let both = classify(root, &result, p_key, &m.contents);
                    let mut newbad = after.clone();
                    let sc = choose_system_configuration(&after).expect("系统配置");
                    for r in singlefs_core::recovery::readable_roots(&after, &sc.immutable.region_devices, &sc.immutable.sizes, &sc.immutable.filesystem_identifier) {
                        if r.instance > st.p.root.instance {
                            let (d, off, len) = root_slot_range(r.checkpoint_txg);
                            newbad.devices.get_mut(&d).expect("盘").write(singlefs_core::address::DeviceOffsetInBytes(off), &vec![0u8; len as usize]);
                        }
                    }
                    let (root2, result2) = read_only(&newbad);
                    let newbad_cls = classify(root2, &result2, p_key, &m.contents);
                    let p_new = new_unit_ranges(&st.p);
                    let p_units_changed = p_new
                        .iter()
                        .filter(|(d, off, len)| {
                            let a = after.devices[d].read(singlefs_core::address::DeviceOffsetInBytes(*off), *len as usize);
                            let b = full.devices[d].read(singlefs_core::address::DeviceOffsetInBytes(*off), *len as usize);
                            a != b
                        })
                        .count();
                    println!(
                        "OPUS-B4 k={k} hide={hide}+scnew arm={:?} p={:?} effective={:?} instance2_txgs={new_txgs:?} p_root_slot_kept={p_slot_kept} p_new_units_overwritten={p_units_changed}/{} ro_both={:?}:{both} ro_after_new_instance_roots_zeroed={:?}:{newbad_cls}",
                        std::env::var("OPUS_B4_ARM").unwrap_or_default(),
                        p_key,
                        m.effective,
                        p_new.len(),
                        root.map(|r| r.1 .0),
                        root2.map(|r| r.1 .0)
                    );
                }
            }
        }
    }
}
