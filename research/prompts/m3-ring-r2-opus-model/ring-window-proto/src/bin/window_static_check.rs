//! m3-ring-r2 攻方原型 G2（全域判法）：一段历史录成一条流（基镜像 + 写序列 + 段），对流里每一个单元写 w 判：
//! 有没有一个崩溃状态让 w 已落盘、而 w 盖住的槽正被那个状态的候选集（有效 ∧ txg ≥ F_生效 ∧ 一圈窗口，窗口锚在那个状态读得出的最新根）
//! 里某条根引用着。按层 0 的枚举域，w 在第 k 段时，凡是 w 落了的状态都是「前 k 段全落 + 第 k 段的某个子集」：
//! 单元写是 COW、根槽写 FUA 自成一段，所以这些状态读得出的根恒等于前 k 段全落时的根（第 k 段里没有根槽写），
//! 候选集只取决于 k。于是「前 k 段全落」那一个镜像上算一次候选集，就判完了 w 落了的全部状态（原地覆写撕裂只碰根槽、系统配置槽、journal 槽，不碰单元区）。
//! 打中的，再造「前 k 段全落 + 只有 w 落」那一个状态跑池级 checker（另一条路）。
//!
//! 用法：window_static_check <S> <历史> <洞 txg> <后缀覆盖写次数 K> <回退 0|1>
//! 历史：k1（新实例写行那次崩、下一次挂载 k）、k1x2（连崩两次写行、再挂载）、fillable（同一实例覆盖写崩、下一次挂载）
#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};

use ring_window_proto::{
    build_prefix, content, crash_a_mount_before_its_row_root, devices_of, geometry, parameters,
    pool_of, recording_of, violated, WRITE_TIME,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, effective_rollback_floor,
    instance_table_of_root, readable_roots_with_ring_slots, root_is_abandoned_by_the_instance_table,
    PoolReader,
};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const SLOT: u64 = 16384;

fn placements_of(reader: &dyn PoolReader, root: &RootRecord) -> BTreeSet<(u32, u64)> {
    let mut out = BTreeSet::new();
    if let Ok(records) = allocation_records_under_root(reader, root) {
        for r in records.iter().filter(|r| !r.is_released) {
            for s in r.slot.0..r.slot.0 + u64::from(r.span_slots) {
                out.insert((r.device.0, s));
            }
        }
    }
    for (locations, span) in [(&root.instance_table.locations, 2u64), (&root.tree_table.locations, 1u64)] {
        for l in locations.iter() {
            if l.slot.0 != 0 {
                for s in l.slot.0..l.slot.0 + span {
                    out.insert((l.device.0, s));
                }
            }
        }
    }
    out
}

fn windowed_candidates(image: &MemoryPool) -> Option<(RootRecord, Vec<RootRecord>, Vec<u64>)> {
    let configuration = choose_system_configuration(image).ok()?;
    let immutable = &configuration.immutable;
    let roots: Vec<RootRecord> = readable_roots_with_ring_slots(image, &immutable.region_devices, &immutable.sizes, &immutable.filesystem_identifier)
        .into_iter()
        .map(|(_, r)| r)
        .collect();
    let newest = *roots.iter().max_by_key(|r| (r.checkpoint_txg, r.instance))?;
    let table = instance_table_of_root(image, &newest)?;
    let floor = effective_rollback_floor(image, &immutable.region_devices, &immutable.sizes, &immutable.filesystem_identifier);
    let ring_len = 3 * immutable.sizes.root_ring_slots_per_region.count();
    let mut outside = Vec::new();
    let mut candidates = Vec::new();
    for r in roots {
        if root_is_abandoned_by_the_instance_table(&r, &table) || r.checkpoint_txg < floor {
            continue;
        }
        if r.checkpoint_txg.0 + ring_len > newest.checkpoint_txg.0 || std::env::var("STATIC_NO_WINDOW").is_ok_and(|v| v == "1") {
            candidates.push(r);
        } else {
            outside.push(r.checkpoint_txg.0);
        }
    }
    outside.sort_unstable();
    Some((newest, candidates, outside))
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let history = a[2].clone();
    let hole: u64 = a[3].parse().expect("洞 txg");
    let suffix: u64 = a[4].parse().expect("K");
    let with_rollback = a[5] == "1";
    let parameters = parameters(s);
    let anchor = format!("{:?}", singlefs_core::mount::window_anchor());
    // 起点镜像。
    let base: MemoryPool = match history.as_str() {
        "k1" => {
            std::env::set_var("CRASH_THE_ROW_PUBLISH", "1");
            let p = build_prefix(s, hole);
            std::env::remove_var("CRASH_THE_ROW_PUBLISH");
            p.image
        }
        "k1x2" => {
            std::env::set_var("CRASH_THE_ROW_PUBLISH", "1");
            let p = build_prefix(s, hole);
            std::env::remove_var("CRASH_THE_ROW_PUBLISH");
            crash_a_mount_before_its_row_root(s, &p.image).0
        }
        "fillable" => build_prefix(s, hole).image,
        other => panic!("{other}"),
    };
    // 被判的那段历史：一次可写挂载 + K 次覆盖写 + （可选）回退到窗口里最旧的那条候选根 + 2 次覆盖写，录成一条流。
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&base, &stream);
    let mounted = mount_writable(&parameters, &mut recording).expect("被判的挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("带文件");
    let mut rolled = String::from("none");
    for step in 0..suffix {
        let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
        current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(500 + step), write_time_seconds: WRITE_TIME + 500 + step }, instance).expect("后缀覆盖写");
    }
    if with_rollback {
        let now = {
            let mut image = base.clone();
            image.apply_writes(&writes_and_segments(&stream.retained_operations(), &geometry(&parameters)).0);
            image
        };
        if let Some((newest, candidates, _)) = windowed_candidates(&now) {
            if let Some(target) = candidates.iter().filter(|r| r.checkpoint_txg != newest.checkpoint_txg).min_by_key(|r| r.checkpoint_txg) {
                rolled = match roll_back_by_a_forward_publish(&parameters, &mut recording, &mut allocator, &mut current, RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg }) {
                    Ok(_) => format!("to{}", target.checkpoint_txg.0),
                    Err(e) => format!("refused{}:{}", target.checkpoint_txg.0, format!("{e:?}").chars().take(40).collect::<String>()),
                };
                for step in 0..2 {
                    let mut pool = PoolWriter::new(&parameters, recording.as_mut_slice());
                    current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(900 + step), write_time_seconds: WRITE_TIME + 900 + step }, instance).expect("回退之后覆盖写");
                }
            }
        }
    }
    let operations = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations, &geometry(&parameters));
    if std::env::var("STATIC_DEBUG").is_ok_and(|v| v == "1") {
        if let Some((newest, candidates, outside)) = windowed_candidates(&base) {
            eprintln!("DEBUG base newest={} outside={outside:?}", newest.checkpoint_txg.0);
            for r in candidates.iter().filter(|r| r.checkpoint_txg.0 + 3 * s <= newest.checkpoint_txg.0 + 3 || r.checkpoint_txg.0 <= 6) {
                let set: Vec<u64> = placements_of(&base, r).into_iter().filter(|(d, _)| *d == 0).map(|(_, x)| x).collect();
                eprintln!("DEBUG root txg={} slots_dev0={set:?}", r.checkpoint_txg.0);
            }
        }
        let first_root = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(writes.len());
        let unit_slots: Vec<u64> = writes[..first_root].iter().filter(|w| w.kind == StepKind::UnitWrite && w.device.0 == 0).map(|w| w.offset.0 / SLOT).collect();
        eprintln!("DEBUG first publish unit slots dev0={unit_slots:?}");
    }
    let mut hits: BTreeMap<String, u64> = BTreeMap::new();
    let mut first_hit: Option<(usize, usize, String)> = None;
    let mut cache: BTreeMap<(u32, u64), BTreeSet<(u32, u64)>> = BTreeMap::new();
    let mut image = base.clone();
    let mut unit_writes = 0u64;
    let mut outside_seen: BTreeSet<u64> = BTreeSet::new();
    for (k, segment) in segments.iter().enumerate() {
        let units: Vec<usize> = segment.iter().copied().filter(|i| writes[*i].kind == StepKind::UnitWrite).collect();
        if !units.is_empty() {
            let (newest, candidates, outside) = windowed_candidates(&image).expect("候选集");
            outside_seen.extend(outside);
            let mut referenced: BTreeMap<(u32, u64), u64> = BTreeMap::new();
            for r in &candidates {
                let key = (r.instance.0, r.checkpoint_txg.0);
                let set = cache.entry(key).or_insert_with(|| placements_of(&image, r)).clone();
                for p in set {
                    referenced.entry(p).or_insert(r.checkpoint_txg.0);
                }
            }
            for i in units {
                unit_writes += 1;
                let w = &writes[i];
                let first = w.offset.0 / SLOT;
                let last = (w.offset.0 + w.length_in_bytes().max(1) - 1) / SLOT;
                for slot in first..=last {
                    if let Some(txg) = referenced.get(&(w.device.0, slot)) {
                        let key = format!("candidate_txg={txg} newest={}", newest.checkpoint_txg.0);
                        *hits.entry(key.clone()).or_default() += 1;
                        if first_hit.is_none() {
                            first_hit = Some((k, i, key));
                        }
                    }
                }
            }
        }
        for i in segment {
            image.apply_writes(std::slice::from_ref(&writes[*i]));
        }
    }
    // 另一条路：打中的第一处造成单个崩溃状态跑 checker。
    let checker = first_hit.as_ref().map(|(k, i, _)| {
        let mut state = base.clone();
        for segment in &segments[..*k] {
            for j in segment {
                state.apply_writes(std::slice::from_ref(&writes[*j]));
            }
        }
        state.apply_writes(std::slice::from_ref(&writes[*i]));
        let red = violated(&state);
        let mut next = devices_of(&state);
        let next_mount = match mount_writable(&parameters, &mut next) {
            Ok(m) => format!("ok chosen={} effective={} first_txg={}", m.output.chosen_root.checkpoint_txg.0, m.output.effective_root.checkpoint_txg.0, m.output.row_publish.root().checkpoint_txg.0),
            Err(e) => format!("err:{}", format!("{e:?}").chars().take(60).collect::<String>()),
        };
        let red_after_next_mount = violated(&pool_of(&next));
        format!("checker_on_hit_state={red:?} next_mount={next_mount} checker_after_next_mount={red_after_next_mount:?}")
    });
    println!(
        "STATIC s={s} history={history} hole={hole} suffix={suffix} rollback={rolled} anchor={anchor} first_txg={} writes={} segments={} unit_writes={unit_writes} valid_roots_outside_window_seen={:?} hit_writes={} hits={hits:?} first_hit={:?} {}",
        mounted.output.row_publish.root().checkpoint_txg.0,
        writes.len(), segments.len(), outside_seen, hits.values().sum::<u64>(), first_hit.as_ref().map(|(k, i, key)| format!("segment{k}/write{i}/{key}")), checker.unwrap_or_default()
    );
}
