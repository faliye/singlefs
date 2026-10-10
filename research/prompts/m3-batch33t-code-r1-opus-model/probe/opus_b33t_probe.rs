//! m3-batch33t-code-r1 攻方（Opus）探针：只在攻方的仓副本里编，不入库。每条用例打印以 `OPUS ` 开头的判定行，不断言。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports, reason = "攻方原型")]

#[path = "common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;

use std::collections::{BTreeMap, BTreeSet};

use common_unwitnessed_publish::*;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, scan_journal, JournalPolicy,
    PoolReader, RecoveryOutcome, RootRingSlotReading,
};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

type Key = (InstanceGeneration, CheckpointTxg);

fn k(key: Key) -> String {
    format!("({},{})", key.0 .0, key.1 .0)
}

/// 一条根（按 (实例, txg)）在这份镜像上的根槽区间（盘、偏移、长度）；环里没有自证过的那一条交 None。
fn root_slot_range(image: &MemoryPool, key: Key) -> Option<(DeviceIdentity, u64, u64)> {
    let configuration = choose_system_configuration(image).ok()?;
    let sizes = &configuration.immutable.sizes;
    every_root_ring_slot(sizes).into_iter().find_map(|ring_slot| {
        match read_root_ring_slot(image, &configuration.immutable.region_devices, sizes, &configuration.immutable.filesystem_identifier, ring_slot) {
            RootRingSlotReading::SelfVerified(root) if (root.instance, root.checkpoint_txg) == key => Some((
                configuration.immutable.region_devices[usize::try_from(ring_slot.region).unwrap()],
                singlefs_core::root_ring::slot_offset(ring_slot, sizes.fixed_structure_slot_spacing).0,
                u64::from(sizes.physical_block_size),
            )),
            _ => None,
        }
    })
}

/// 环里全部自证过的根的 (实例, txg)。
fn ring_keys(image: &MemoryPool) -> Vec<Key> {
    let Ok(configuration) = choose_system_configuration(image) else { return Vec::new() };
    let sizes = &configuration.immutable.sizes;
    let mut keys: Vec<Key> = every_root_ring_slot(sizes)
        .into_iter()
        .filter_map(|ring_slot| match read_root_ring_slot(image, &configuration.immutable.region_devices, sizes, &configuration.immutable.filesystem_identifier, ring_slot) {
            RootRingSlotReading::SelfVerified(root) => Some((root.instance, root.checkpoint_txg)),
            RootRingSlotReading::Bad(_) => None,
        })
        .collect();
    keys.sort_by_key(|key| (key.1, key.0));
    keys
}

/// 一次发布（(实例, txg)）的全部记录在每块盘上的区间（扫环取计数器，按计数器算偏移）。`last_only` 只取带末条标志的那一条。
fn record_ranges(image: &MemoryPool, key: Key, last_only: bool) -> Vec<(DeviceIdentity, u64, u64)> {
    let configuration = choose_system_configuration(image).expect("系统配置");
    let records = scan_journal(image, &configuration);
    let ring_bytes = configuration.immutable.sizes.journal_ring_bytes;
    let mut ranges = Vec::new();
    for record in records.values().filter(|record| (record.instance, record.checkpoint_txg) == key) {
        if last_only && record.place_in_publish != singlefs_core::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish {
            continue;
        }
        let offset = singlefs_core::journal::record_offset(record.counter, ring_bytes).0;
        for device in image.devices.keys() {
            ranges.push((*device, offset, 4096));
        }
    }
    ranges
}

/// 镜像拷贝上把几段写成全 0（持久的读回全 0；只读的恢复与只读挂载用它）。
fn zeroed(image: &MemoryPool, ranges: &[(DeviceIdentity, u64, u64)]) -> MemoryPool {
    let mut copy = image.clone();
    for (device, offset, length) in ranges {
        copy.devices.get_mut(device).expect("盘").write(DeviceOffsetInBytes(*offset), &vec![0u8; usize::try_from(*length).unwrap()]);
    }
    copy
}

fn label(contents: &BTreeMap<Key, Vec<u8>>, named: &[(&str, Key)], content: &[u8]) -> String {
    let hits: Vec<&str> = named.iter().filter(|(_, key)| contents.get(key).is_some_and(|c| c == content)).map(|(n, _)| *n).collect();
    if hits.is_empty() { "content=OTHER".to_string() } else { format!("content={}", hits.join("|")) }
}

fn kind_of(text: &str) -> String {
    text.split([' ', '(', '{']).next().unwrap_or("").to_string()
}

fn short_mount_error(error: &MountError) -> String {
    match error {
        MountError::NewerStateStillUnreadableAfterOneReread(still) => format!("Refused:{}", kind_of(&format!("{still:?}"))),
        other => format!("Err:{}", kind_of(&format!("{other:?}"))),
    }
}

/// 三处（恢复、只读挂载、可写挂载）在这份镜像上各落到哪一版；`hidden` 那几段读回全 0（只读两处用写零的拷贝，可写挂载用读回全零的设备）。
fn three_paths(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    hidden: &[(DeviceIdentity, u64, u64)],
    named: &[(&str, Key)],
) -> String {
    three_paths_with(recorded, &recorded.pool.content_of_each_root, image, hidden, named)
}

fn three_paths_with(
    recorded: &RecordedPublish,
    contents: &BTreeMap<Key, Vec<u8>>,
    image: &MemoryPool,
    hidden: &[(DeviceIdentity, u64, u64)],
    named: &[(&str, Key)],
) -> String {
    let name_of = |key: Key| named.iter().find(|(_, k2)| *k2 == key).map(|(n, _)| n.to_string()).unwrap_or_else(|| k(key));
    let zero_image = zeroed(image, hidden);
    let report = recover(&zero_image, JournalPolicy::Consult);
    let rec = match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => format!("recover=FileRead(root={} {})", name_of(*root), label(contents, named, content)),
        RecoveryOutcome::NoFile { root } => format!("recover=NoFile({})", name_of(*root)),
        RecoveryOutcome::Failed { root, failure } => format!("recover=Failed(root={} {})", root.map(name_of).unwrap_or_default(), kind_of(&format!("{failure:?}"))),
    };
    let ro = match mount_read_only(&zero_image) {
        Ok(mounted) => format!(
            "ro=Ok(chosen={} effective={})",
            name_of((mounted.chosen_root.instance, mounted.chosen_root.checkpoint_txg)),
            name_of((mounted.effective_root.instance, mounted.effective_root.checkpoint_txg))
        ),
        Err(error) => format!("ro=Err({})", kind_of(&format!("{error:?}"))),
    };
    let ranges = RangesReadingBackAllZero(hidden.to_vec());
    let mut devices = devices_reading_back_all_zero_on(image, &ranges);
    let rw = match mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(mounted) => {
            let (output, _session) = MountedSession::of_the_mount(mounted);
            format!(
                "rw=Ok(chosen={} effective={} inst={})",
                name_of((output.chosen_root.instance, output.chosen_root.checkpoint_txg)),
                name_of((output.effective_root.instance, output.effective_root.checkpoint_txg)),
                output.instance.0
            )
        }
        Err(error) => format!("rw={}", short_mount_error(&error)),
    };
    format!("{rec} {ro} {rw}")
}

/// 已确认的那一版 V 之前的那一版（环里比 V 旧的最新一条根）。
fn version_before(image: &MemoryPool, key: Key) -> Option<Key> {
    ring_keys(image).into_iter().filter(|other| (other.1, other.0) < (key.1, key.0)).max_by_key(|other| (other.1, other.0))
}

fn publish_kind(code: &str) -> PublishRecordedAsR {
    match code {
        "ow" => PublishRecordedAsR::Overwrite,
        "sw3" => PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
        other => panic!("{other}"),
    }
}

/// T5 第一形：R（系统配置没见证）崩在根槽落盘之后；读的那一刻 R 的单元读回全 0（R 走读失败），另加已确认的 V 的根槽读回全 0。
/// 对照：只藏 R 的单元（设计里退回 V 那一形）；R 的单元、R 的根槽、V 的根槽都藏（V 由记录重建得出，证明 V 在盘上读得出）。
#[test]
fn opus_t5_fallback_goes_past_the_witnessed_version() {
    for code in ["ow", "sw3"] {
        for overwrites_before in [1u64, 2, 4, 6] {
            let recorded = record_one_publish_after_overwrites(overwrites_before, publish_kind(code));
            let crash = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let v = recorded.previous_key;
            let r = recorded.key;
            let Some(v_minus) = version_before(&crash, v) else {
                println!("OPUS T5A {code} k={overwrites_before} no-version-before-V");
                continue;
            };
            let named = [("R", r), ("V", v), ("Vminus", v_minus)];
            let r_units: Vec<(DeviceIdentity, u64, u64)> =
                recorded.unit_writes().iter().map(|w| (w.device, w.offset.0, w.length_in_bytes())).collect();
            let r_root = vec![(recorded.root_slot_write().device, recorded.root_slot_write().offset.0, recorded.root_slot_write().length_in_bytes())];
            let v_root = vec![root_slot_range(&crash, v).expect("V 的根槽在环里")];
            let shapes: Vec<(&str, Vec<(DeviceIdentity, u64, u64)>)> = vec![
                ("none", vec![]),
                ("Ru", r_units.clone()),
                ("Ru+Vr", [r_units.clone(), v_root.clone()].concat()),
                ("Ru+Rr+Vr", [r_units.clone(), r_root.clone(), v_root.clone()].concat()),
                ("Vr", v_root.clone()),
            ];
            for (shape, hidden) in shapes {
                println!(
                    "OPUS T5A {code} k={overwrites_before} R={} V={} Vminus={} shape={shape} {}",
                    k(r), k(v), k(v_minus),
                    three_paths(&recorded, &crash, &hidden, &named)
                );
            }
        }
    }
}

/// 一次可写挂载（`hidden` 那几段整次挂载读回全 0）的录制流：交回挂载输出（或错误）与写表。
fn recorded_mount(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    hidden: &[(DeviceIdentity, u64, u64)],
) -> (Result<singlefs_core::mount::MountOutput, String>, Vec<RetainedWrite>) {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(image, &RangesReadingBackAllZero(hidden.to_vec()), &stream);
    let outcome = mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula)
        .map(|mounted| MountedSession::of_the_mount(mounted).0)
        .map_err(|error| short_mount_error(&error));
    let (writes, _segments, _) =
        writes_and_segments_with_stream_indexes(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
    (outcome, writes)
}

fn ranges_of(writes: &[&RetainedWrite]) -> Vec<(DeviceIdentity, u64, u64)> {
    writes.iter().map(|w| (w.device, w.offset.0, w.length_in_bytes())).collect()
}

/// R 那几样（u 单元、r 根槽、J 全部记录、L 末条、M 中间一条、F 第一条）在 R 的写表里的区间。
fn r_ranges(recorded: &RecordedPublish, hide: &str) -> Vec<(DeviceIdentity, u64, u64)> {
    let mut ranges = Vec::new();
    if hide.contains('u') {
        ranges.extend(ranges_of(&recorded.unit_writes()));
    }
    if hide.contains('r') {
        ranges.extend(ranges_of(&[recorded.root_slot_write()]));
    }
    let n = recorded.record_count();
    let mut ordinals: BTreeSet<usize> = BTreeSet::new();
    if hide.contains('J') {
        ordinals.extend(1..=n);
    }
    if hide.contains('L') {
        ordinals.insert(n);
    }
    if hide.contains('F') {
        ordinals.insert(1);
    }
    if hide.contains('M') {
        ordinals.insert(n.div_ceil(2).max(1));
    }
    for ordinal in ordinals {
        ranges.extend(ranges_of(&recorded.record_writes_of_the_record(ordinal)));
    }
    ranges
}

/// T5 第二形：取新实例那次挂载 M1 藏 R 的根槽与单元（K1-2，登记的故障模型），崩在 M1 写行那次发布的根槽落盘之后、它的轮换之前；
/// 下一次读时写行那次发布的单元读回全 0（它走读失败），见证计数器那条记录（V 的末条）读回全 0：退到的是 M1 抛弃了的 R。
#[test]
fn opus_t5_fallback_lands_on_a_root_the_instance_table_abandoned() {
    for overwrites_before in [1u64, 2, 4] {
        let recorded = record_one_publish_after_overwrites(overwrites_before, PublishRecordedAsR::Overwrite);
        let crash = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
        let v = recorded.previous_key;
        let r = recorded.key;
        let (m1, writes) = recorded_mount(&recorded, &crash, &r_ranges(&recorded, "ur"));
        let m1 = match m1 {
            Ok(output) => output,
            Err(error) => {
                println!("OPUS T5B k={overwrites_before} M1={error}");
                continue;
            }
        };
        let row = (m1.row_publish.root().instance, m1.row_publish.root().checkpoint_txg);
        let row_root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("写行根槽");
        let mut state = crash.clone();
        state.apply_writes(&writes[..=row_root_index]);
        let row_units: Vec<&RetainedWrite> = writes[..row_root_index].iter().filter(|w| w.kind == StepKind::UnitWrite).collect();
        let mut contents = recorded.pool.content_of_each_root.clone();
        contents.insert(row, recorded.previous_content.clone());
        let v_minus = version_before(&crash, v).expect("V 之前一版");
        let named = [("R", r), ("V", v), ("Vminus", v_minus), ("Row", row)];
        let overwritten = units_of_r_overwritten_on(&recorded, &state).len();
        let v_last = record_ranges(&state, v, true);
        let row_unit_ranges = ranges_of(&row_units);
        let shapes: Vec<(&str, Vec<(DeviceIdentity, u64, u64)>)> = vec![
            ("none", vec![]),
            ("RowU", row_unit_ranges.clone()),
            ("RowU+VlastRec", [row_unit_ranges.clone(), v_last.clone()].concat()),
            ("VlastRec", v_last.clone()),
        ];
        println!(
            "OPUS T5B k={overwrites_before} R={} V={} Row={} m1_effective={} r_units_overwritten_in_state={overwritten} ring={:?}",
            k(r), k(v), k(row), k((m1.effective_root.instance, m1.effective_root.checkpoint_txg)),
            ring_keys(&state).into_iter().map(k).collect::<Vec<_>>()
        );
        for (shape, hidden) in shapes {
            println!("OPUS T5B k={overwrites_before} shape={shape} {}", three_paths_with(&recorded, &contents, &state, &hidden, &named));
        }
    }
}

/// T1 盲区：R 是三条记录的顺序写；M1 藏 R 的根槽与某几条记录（单元读得出）。崩在 M1 写行那次发布的根槽之前（写行的单元与记录都落了）：
/// R 的单元被 M1 写掉几份、池级 checker、三处落到哪一版。
#[test]
fn opus_t1_blind_spot_last_record_hidden() {
    for overwrites_before in [1u64, 2, 4] {
        for hide in ["r", "rL", "rM", "rF", "rJ", "ur", "urL"] {
            let recorded = record_one_publish_after_overwrites(overwrites_before, PublishRecordedAsR::SequentialWriteOfThreeDataUnits);
            let crash = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let v = recorded.previous_key;
            let r = recorded.key;
            let v_minus = version_before(&crash, v).expect("V 之前一版");
            let named = [("R", r), ("V", v), ("Vminus", v_minus)];
            let (m1, writes) = recorded_mount(&recorded, &crash, &r_ranges(&recorded, hide));
            let summary = match &m1 {
                Ok(output) => format!(
                    "M1=Ok(effective={} isolated={:?})",
                    k((output.effective_root.instance, output.effective_root.checkpoint_txg)),
                    output.isolated_slots_per_device.iter().map(|(_, n)| *n).collect::<Vec<_>>()
                ),
                Err(error) => format!("M1={error}"),
            };
            let Some(row_root_index) = writes.iter().position(|w| w.kind == StepKind::RootRecordFua) else {
                println!("OPUS T1D k={overwrites_before} hide={hide} {summary} no-row-root");
                continue;
            };
            let mut state = crash.clone();
            state.apply_writes(&writes[..row_root_index]);
            let overwritten = units_of_r_overwritten_on(&recorded, &state);
            let checker = checker_violations_on(&state)
                .into_iter()
                .map(|line| line.split(':').next().unwrap_or("").to_string())
                .collect::<BTreeSet<_>>();
            println!(
                "OPUS T1D k={overwrites_before} hide={hide} records={} {summary} r_unit_copies_overwritten={} chk={:?} {}",
                recorded.record_count(),
                overwritten.len(),
                checker,
                three_paths(&recorded, &state, &[], &named)
            );
        }
    }
}

fn clone_pool(pool: &PoolUnderTest<SparseBlockDevice>) -> PoolUnderTest<SparseBlockDevice> {
    let session = pool.session();
    PoolUnderTest {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        devices: plain_devices_on(&pool.image()),
        session: Some(MountedSession {
            allocator: session.allocator.clone(),
            current: session.current.clone(),
            instance: session.instance,
            shadow_ledger: session.shadow_ledger,
            parameters_and_device_table: session.parameters_and_device_table.clone(),
        }),
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version.clone(),
        content_of_each_root: pool.content_of_each_root.clone(),
    }
}

/// 用户动作：o 覆盖写、s 顺序写 2 单元、n 建一个 inode、u 删第一个文件。
fn apply_op(pool: &mut PoolUnderTest<SparseBlockDevice>, op: char) -> Result<(), String> {
    let outcome = match op {
        'o' => pool.overwrite(OVERWRITE_BYTES),
        's' => pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity()),
        'n' => pool.create_one_inode(),
        'u' => {
            pool.write_time_seconds += 1;
            let write_time_seconds = pool.write_time_seconds;
            pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER), write_time_seconds }, None)
        }
        other => panic!("{other}"),
    };
    outcome.map(|_| ()).map_err(|e| kind_of(&format!("{e:?}")))
}

/// 环里实例代号高于 `instance` 的根槽全写零之后恢复：落到哪一版、读出什么（R 的可读性探针，同设计轮攻方的 probe）。
fn probe_after_erasing_newer_instances(image: &MemoryPool, instance: InstanceGeneration, contents: &BTreeMap<Key, Vec<u8>>, named: &[(&str, Key)]) -> String {
    let newer: Vec<(DeviceIdentity, u64, u64)> = ring_keys(image)
        .into_iter()
        .filter(|key| key.0 > instance)
        .filter_map(|key| root_slot_range(image, key))
        .collect();
    let copy = zeroed(image, &newer);
    let report = recover(&copy, JournalPolicy::Consult);
    match report.outcome {
        RecoveryOutcome::FileRead { root, content } => format!("probe=FileRead(root={} {})", k(root), label(contents, named, &content)),
        RecoveryOutcome::NoFile { root } => format!("probe=NoFile({})", k(root)),
        RecoveryOutcome::Failed { root, failure } => format!("probe=Failed(root={:?} {})", root.map(k), kind_of(&format!("{failure:?}"))),
    }
}

/// T1 盲区（续）：M1 藏 R 的根槽与末条记录（单元读得出，F1 不拒），M1 做成之后用户动作放开扫（字母表 o s n u，长度 0..=3），
/// 每个结点看 R 的单元被写掉几份、R 还在不在环里、池级 checker，与抹掉新实例的根之后恢复落到哪。
#[test]
fn opus_t1_blind_spot_after_the_mount_with_user_actions() {
    let depth: usize = std::env::var("OPUS_DEPTH").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    let ks: Vec<u64> = std::env::var("OPUS_KS").unwrap_or_else(|_| "1,4".to_string()).split(',').map(|v| v.parse().expect("OPUS_KS")).collect();
    for overwrites_before in ks {
        for hide in ["rL", "rJ", "rM", "rF"] {
            let recorded = record_one_publish_after_overwrites(overwrites_before, PublishRecordedAsR::SequentialWriteOfThreeDataUnits);
            let crash = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let (v, r) = (recorded.previous_key, recorded.key);
            let v_minus = version_before(&crash, v).expect("V 之前一版");
            let named = [("R", r), ("V", v), ("Vminus", v_minus)];
            let mut devices = devices_reading_back_all_zero_on(&crash, &RangesReadingBackAllZero(r_ranges(&recorded, hide)));
            let mounted = match mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
                Ok(mounted) => mounted,
                Err(error) => {
                    println!("OPUS T1G k={overwrites_before} hide={hide} M1={}", short_mount_error(&error));
                    continue;
                }
            };
            let image = image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
            let pool = pool_after_the_mount(&recorded, &image, mounted);
            let mut stack: Vec<(String, PoolUnderTest<SparseBlockDevice>)> = vec![(String::new(), pool)];
            let mut lines = 0usize;
            let mut red = 0usize;
            while let Some((sequence, node)) = stack.pop() {
                let image = node.image();
                let in_ring = ring_keys(&image).contains(&r);
                let overwritten = units_of_r_overwritten_on(&recorded, &image).len();
                let checker = checker_violations_on(&image).into_iter().map(|l| l.split(':').next().unwrap_or("").to_string()).collect::<BTreeSet<_>>();
                if !checker.is_empty() {
                    red += 1;
                }
                let probe = if in_ring { probe_after_erasing_newer_instances(&image, r.0, &recorded.pool.content_of_each_root, &named) } else { "-".to_string() };
                println!(
                    "OPUS T1G k={overwrites_before} hide={hide} seq={} r_in_ring={in_ring} r_unit_copies_overwritten={overwritten} chk={checker:?} {probe}",
                    if sequence.is_empty() { "-" } else { &sequence }
                );
                lines += 1;
                if sequence.len() < depth {
                    for op in ['o', 's', 'n', 'u'] {
                        let mut child = clone_pool(&node);
                        match apply_op(&mut child, op) {
                            Ok(()) => stack.push((format!("{sequence}{op}"), child)),
                            Err(kind) => println!("OPUS T1G k={overwrites_before} hide={hide} seq={sequence}{op} OP_REFUSED {kind}"),
                        }
                    }
                }
            }
            println!("OPUS T1GTALLY k={overwrites_before} hide={hide} depth={depth} nodes={lines} checker_red={red}");
        }
    }
}

/// T4：R 是三条记录的顺序写，崩在写记录那一段里（R 的单元与之前的屏障都落了，记录那一段的 6 次写取任意子集，根槽没落）。
/// 每个状态可写挂载一次：不藏（正常崩溃）与 R 的单元整次挂载读回全 0 两臂，数 F1 拒了几个。
#[test]
fn opus_t4_crash_inside_the_record_segment() {
    for overwrites_before in [1u64, 4] {
        let recorded = record_one_publish_after_overwrites(overwrites_before, PublishRecordedAsR::SequentialWriteOfThreeDataUnits);
        let record_indices: Vec<usize> = recorded.writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::JournalRecord).map(|(i, _)| i).collect();
        let first_record = record_indices[0];
        let before_records: Vec<RetainedWrite> = recorded.writes[..first_record].to_vec();
        for arm in ["none", "u"] {
            let mut tally: BTreeMap<String, usize> = BTreeMap::new();
            for mask in 0u32..(1 << record_indices.len()) {
                let mut image = recorded.base.clone();
                image.apply_writes(&before_records);
                let chosen: Vec<RetainedWrite> = record_indices
                    .iter()
                    .enumerate()
                    .filter(|(bit, _)| mask & (1 << bit) != 0)
                    .map(|(_, index)| recorded.writes[*index].clone())
                    .collect();
                image.apply_writes(&chosen);
                let hidden = if arm == "u" { r_ranges(&recorded, "u") } else { Vec::new() };
                let mut devices = devices_reading_back_all_zero_on(&image, &RangesReadingBackAllZero(hidden));
                let outcome = match mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
                    Ok(mounted) => {
                        let (output, _) = MountedSession::of_the_mount(mounted);
                        format!("Ok(effective=({},{}))", output.effective_root.instance.0, output.effective_root.checkpoint_txg.0)
                    }
                    Err(error) => short_mount_error(&error),
                };
                *tally.entry(outcome).or_default() += 1;
            }
            println!("OPUS T4 k={overwrites_before} R={} record_writes={} arm={arm} states={} {:?}", k(recorded.key), record_indices.len(), 1u32 << record_indices.len(), tally);
        }
    }
}

/// T4 续：写记录那一段只落了第一条记录（两份），R 的根槽从没落；R 的单元在盘上持久地读回全 0（一处介质故障，单故障）。
/// 连挂三次可写挂载，每次之后核盘面逐字节没动；再只读挂载、恢复各一次。
#[test]
fn opus_t4_permanent_unit_loss_under_a_publish_whose_root_never_landed() {
    for overwrites_before in [1u64, 4] {
        let recorded = record_one_publish_after_overwrites(overwrites_before, PublishRecordedAsR::SequentialWriteOfThreeDataUnits);
        let first_record = recorded.writes.iter().position(|w| w.kind == StepKind::JournalRecord).expect("记录");
        let mut image = recorded.base.clone();
        image.apply_writes(&recorded.writes[..first_record]);
        image.apply_writes(&recorded.record_writes_of_the_record(1).into_iter().cloned().collect::<Vec<_>>());
        let copies = std::env::var("OPUS_T4L_COPIES").unwrap_or_else(|_| "both".to_string());
        let hidden: Vec<(DeviceIdentity, u64, u64)> = r_ranges(&recorded, "u")
            .into_iter()
            .filter(|(device, _, _)| copies == "both" || (copies == "one" && *device == DeviceIdentity(0)) || (copies == "one-unit" && *device == DeviceIdentity(0)))
            .collect();
        let hidden: Vec<(DeviceIdentity, u64, u64)> = if copies == "one-unit" { hidden.into_iter().take(1).collect() } else { hidden };
        let image = zeroed(&image, &hidden);
        let mut outcomes = Vec::new();
        for _attempt in 0..3 {
            let mut devices = plain_devices_on(&image);
            let outcome = match mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
                Ok(mounted) => {
                    let (output, _) = MountedSession::of_the_mount(mounted);
                    format!("Ok(effective=({},{}))", output.effective_root.instance.0, output.effective_root.checkpoint_txg.0)
                }
                Err(error) => short_mount_error(&error),
            };
            let after = MemoryPool {
                devices: devices.iter().map(|(id, d)| (*id, d.image.clone())).collect(),
                device_size_in_bytes: image.device_size_in_bytes,
            };
            let unchanged = image.devices.keys().all(|id| {
                let total = usize::try_from(image.device_size_in_bytes).unwrap();
                (0..total).step_by(1 << 20).all(|offset| {
                    let length = (1usize << 20).min(total - offset);
                    image.devices[id].read(DeviceOffsetInBytes(offset as u64), length) == after.devices[id].read(DeviceOffsetInBytes(offset as u64), length)
                })
            });
            outcomes.push(format!("{outcome}/disk_unchanged={unchanged}"));
        }
        let named = [("R", recorded.key), ("V", recorded.previous_key)];
        let report = recover(&image, JournalPolicy::Consult);
        let rec = match &report.outcome {
            RecoveryOutcome::FileRead { root, content } => format!("recover=FileRead(root={} {})", k(*root), label(&recorded.pool.content_of_each_root, &named, content)),
            other => format!("recover={}", kind_of(&format!("{other:?}"))),
        };
        let ro = match mount_read_only(&image) {
            Ok(mounted) => format!("ro=Ok(effective=({},{}))", mounted.effective_root.instance.0, mounted.effective_root.checkpoint_txg.0),
            Err(error) => format!("ro=Err({})", kind_of(&format!("{error:?}"))),
        };
        println!("OPUS T4L copies={copies} zeroed_ranges={} k={overwrites_before} R={} rw_three_times={outcomes:?} {rec} {ro}", hidden.len(), k(recorded.key));
    }
}
