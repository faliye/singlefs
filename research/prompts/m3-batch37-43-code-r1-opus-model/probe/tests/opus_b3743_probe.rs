//! m3-batch37-43-code-r1 攻方（Opus）探针：只在攻方的仓副本里编，不入库。每条用例打印以 `OPUS ` 开头的判定行，不断言。
//! 装置照 `singlefs-harness/tests/common_unwitnessed_publish/mod.rs`（384 槽小盘、第一个文件之后覆盖写 k 次、录一次发布 R，
//! 崩在 R 的根槽落盘之后、轮换之前）。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports, unused_variables, reason = "攻方原型")]

#[path = "../../singlefs-harness/tests/common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;

use std::collections::{BTreeMap, BTreeSet};

use common_unwitnessed_publish::*;
use singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::bytes::ByteReader;
use singlefs_core::mount::{mount_writable, MountError, StillUnreadableAfterOneReread};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, tree_table_entry_bytes_of_root, JournalPolicy,
    PoolReader, RecoveryOutcome, RecoveryReport, RootRingSlotReading,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, PublishedVersion, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount, DiskAfterTheRefusal,
    ReadFaultsInTheCrashState, ReadFaultsUnderTheRefusedMount, ReadOnlyReadBackOfTheCrashImage, TreeTableEntryRootPointersByKind,
    WritableMountRefusalVerdict,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

type Key = (InstanceGeneration, CheckpointTxg);
type Range = (DeviceIdentity, u64, u64);

fn k(key: Key) -> String {
    format!("({},{})", key.0 .0, key.1 .0)
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

fn env_list(name: &str, default: &str) -> Vec<String> {
    std::env::var(name).unwrap_or_else(|_| default.to_string()).split(',').filter(|s| !s.is_empty()).map(str::to_string).collect()
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn ranges_of(writes: &[&RetainedWrite]) -> Vec<Range> {
    writes.iter().map(|w| (w.device, w.offset.0, w.length_in_bytes())).collect()
}

/// 环里全部自证过的根的 (实例, txg)，按 (txg, 实例) 排。
fn ring_keys(image: &dyn PoolReader) -> Vec<Key> {
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

fn publish_kind(code: &str) -> PublishRecordedAsR {
    match code {
        "ow" => PublishRecordedAsR::Overwrite,
        "sw3" => PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
        other => panic!("{other}"),
    }
}

/// R 写出的单元按单元归组之后每组的角色：T 树表、M 中央映射树根、I 实例表、K<种类> 树表条目里那棵树的根、D 数据、P 打包、N 其余节点。
fn roles_of_the_unit_groups(recorded: &RecordedPublish, image: &MemoryPool) -> Vec<String> {
    let root = recorded.root();
    let hits = |pointer: &NodePointer, group: &[&RetainedWrite]| {
        pointer.locations.iter().any(|l| group.iter().any(|w| l.device == w.device && l.slot.to_device_offset() == w.offset))
    };
    let kinds: Vec<u16> = (0u16..64).collect();
    let tree_roots: Vec<(u16, NodePointer)> = tree_table_entry_bytes_of_root(image, &root)
        .map(|entries| {
            TreeTableEntryRootPointersByKind::of_the_entries(&entries, &kinds)
                .root_pointer_of_each_kind
                .into_iter()
                .map(|(kind, bytes)| (kind, NodePointer::read_from(&mut ByteReader::at(&bytes, 0))))
                .collect()
        })
        .unwrap_or_default();
    recorded
        .unit_writes_grouped_by_unit()
        .iter()
        .map(|group| {
            let class = group[0].bytes().and_then(|b| b.get(6).copied()).unwrap_or(0);
            if hits(&root.tree_table, group) {
                "T".to_string()
            } else if hits(&root.mapping_root, group) {
                "M".to_string()
            } else if hits(&root.instance_table, group) {
                "I".to_string()
            } else if let Some((kind, _)) = tree_roots.iter().find(|(_, p)| hits(p, group)) {
                format!("K{kind}")
            } else if class == 1 {
                "D".to_string()
            } else if class == 3 {
                "P".to_string()
            } else {
                "N".to_string()
            }
        })
        .collect()
}

fn name_of(recorded: &RecordedPublish, key: Key) -> String {
    if key == recorded.key {
        "R".to_string()
    } else if key == recorded.previous_key {
        "P".to_string()
    } else {
        k(key)
    }
}

fn recovery_label(recorded: &RecordedPublish, report: &RecoveryReport) -> String {
    let contents = &recorded.pool.content_of_each_root;
    let content_name = |content: &[u8]| {
        if content == recorded.content.as_slice() {
            "R".to_string()
        } else if content == recorded.previous_content.as_slice() {
            "P".to_string()
        } else {
            contents.iter().find(|(_, c)| c.as_slice() == content).map(|(key, _)| k(*key)).unwrap_or_else(|| "OTHER".to_string())
        }
    };
    match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => format!("FileRead({} c={})", name_of(recorded, *root), content_name(content)),
        RecoveryOutcome::NoFile { root } => format!("NoFile({})", name_of(recorded, *root)),
        RecoveryOutcome::Failed { root, failure } => format!("Failed({} {})", root.map(|r| name_of(recorded, r)).unwrap_or_default(), kind_of(&format!("{failure:?}"))),
    }
}

fn checker_names(image: &MemoryPool) -> BTreeSet<String> {
    checker_violations_on(image).into_iter().map(|l| l.split(':').next().unwrap_or("").to_string()).collect()
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

/// 每条根带的文件内容（录的池那一份）加上 `extra` 那几版，排成版本表（层 0 枚举的 oracle 用；本探针主要看池级 checker 与恢复）。
fn versions_of(recorded: &RecordedPublish, extra: &[(Key, Vec<u8>)]) -> Vec<PublishedVersion> {
    let mut map: BTreeMap<(CheckpointTxg, InstanceGeneration), Vec<u8>> =
        recorded.pool.content_of_each_root.iter().map(|((i, t), c)| ((*t, *i), c.clone())).collect();
    for ((i, t), c) in extra {
        map.insert((*t, *i), c.clone());
    }
    map.into_iter().map(|((t, i), content)| PublishedVersion { instance: i, checkpoint_txg: t, content }).collect()
}

fn image_of_the_state(state: &CrashImage<'_>) -> MemoryPool {
    let mut image = state.base.clone();
    let persisted: Vec<RetainedWrite> = state.writes.iter().zip(&state.persisted).filter(|(_, p)| **p).map(|(w, _)| w.clone()).collect();
    image.apply_writes(&persisted);
    image
}

/// 状态数上限（每次跑）：超过就不跑那一段，打一行 SKIP。
fn layer0_cap() -> u64 {
    std::env::var("OPUS_L0_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(1_000_000)
}

/// M1 那条录制流的层 0 全量（每一段都展开）：每个状态上跑池级 checker、看 journal 的恢复落到哪一版、读出的是不是那一版。
/// 交回一行汇总：状态数（与闭式数比）、checker 红的状态数、恢复落到 R 而走读失败或内容不是 R 的状态数、R 的单元被写掉且 R 仍在环里的状态数。
fn layer0_over_the_stream_of_m1(recorded: &RecordedPublish, base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>], extra: &[(Key, Vec<u8>)]) -> String {
    let closed = closed_form_state_count(segments);
    if closed > layer0_cap() {
        return format!("L0 SKIP closed={closed}");
    }
    let judged_root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(writes.len() - 1);
    let versions = versions_of(recorded, extra);
    // 观察者只收持久位与恢复读数（调用线程上、按状态序号）；池级 checker 与环、单元的核对放到后面分线程做。
    let mut collected: Vec<(Vec<bool>, String, Option<Key>)> = Vec::new();
    let mut table: Option<Vec<RetainedWrite>> = None;
    let _tally = enumerate_layer0_selecting_versions_observing_each_state(base, writes, segments, judged_root_index, &versions, &|_, _| true, &mut |state, report| {
        if table.is_none() {
            table = Some(state.writes.to_vec());
        }
        collected.push((state.persisted.clone(), recovery_label(recorded, report), report.effective_root));
    });
    let table = table.unwrap_or_default();
    let view = RView::of(recorded);
    let threads = std::env::var("SINGLEFS_THREAD_CAP").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(4).max(1);
    let chunk = collected.len().div_ceil(threads).max(1);
    let parts: Vec<(u64, BTreeSet<String>, u64)> = std::thread::scope(|scope| {
        let handles: Vec<_> = collected
            .chunks(chunk)
            .map(|slice| {
                let view = &view;
                let table = &table;
                scope.spawn(move || {
                    let mut red = 0u64;
                    let mut names_all = BTreeSet::new();
                    let mut overwritten_in_ring = 0u64;
                    for (persisted, _, _) in slice {
                        let mut image = base.clone();
                        let applied: Vec<RetainedWrite> = table.iter().zip(persisted).filter(|(_, p)| **p).map(|(w, _)| w.clone()).collect();
                        image.apply_writes(&applied);
                        let names = checker_names(&image);
                        if !names.is_empty() {
                            red += 1;
                            names_all.extend(names);
                        }
                        if ring_keys(&image).contains(&view.key) && !view.overwritten_on(&image).is_empty() {
                            overwritten_in_ring += 1;
                        }
                    }
                    (red, names_all, overwritten_in_ring)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("工作线程")).collect()
    });
    let states = collected.len() as u64;
    let checker_red: u64 = parts.iter().map(|p| p.0).sum();
    let red_names: BTreeSet<String> = parts.iter().flat_map(|p| p.1.iter().cloned()).collect();
    let r_overwritten_in_ring: u64 = parts.iter().map(|p| p.2).sum();
    let mut landed: BTreeMap<String, u64> = BTreeMap::new();
    let mut recovered_r_bad = 0u64;
    for (_, label, effective_root) in &collected {
        *landed.entry(label.clone()).or_insert(0) += 1;
        if *effective_root == Some(recorded.key) && !label.contains("c=R") {
            recovered_r_bad += 1;
        }
    }
    format!(
        "L0 states={states} closed={closed} chk_red={checker_red} chk={red_names:?} rec_R_bad={recovered_r_bad} r_overwritten_in_ring={r_overwritten_in_ring} landed={landed:?}"
    )
}

/// 一格：`hidden` 那几段在这一次读回全零。先在同一组读故障下跑恢复（recF）与只读挂载（roF），再录一次可写挂载 M1：
/// 落到哪一版、C 隔离了几个槽、写行那次根落盘之前 R 的单元被写掉几份、那一刻的池级 checker 与（没有读故障的）恢复；
/// 再在 M1 之后放开用户动作（o 覆盖写、s 顺序写两个数据单元、n 建 inode，长度 ≤ depth），看 R 的根还在环里时 R 的单元被写掉几份。
/// `layer0` 为真且写行根落盘之前写掉了 R 的单元时，再对 M1 的录制流跑层 0 全量。
fn one_cell(recorded: &RecordedPublish, crash: &MemoryPool, hidden: &[Range], depth: usize, layer0: bool, tag: &str) -> String {
    let ranges = RangesReadingBackAllZero(hidden.to_vec());
    let faulty = devices_reading_back_all_zero_on(crash, &ranges);
    let rec_under_faults = recovery_label(recorded, &recover(&faulty, JournalPolicy::Consult));
    let ro_under_faults = match mount_read_only(&faulty) {
        Ok(mounted) => format!("Ok({})", name_of(recorded, (mounted.effective_root.instance, mounted.effective_root.checkpoint_txg))),
        Err(error) => format!("Err({})", kind_of(&format!("{error:?}"))),
    };
    let stream = SharedStream::retaining_contents();
    let mut recorded_devices = recorded_devices_reading_back_all_zero_on(crash, &ranges, &stream);
    let mounted = match mount_writable(&recorded.pool.parameters, &mut recorded_devices) {
        Ok(mounted) => mounted,
        Err(error) => {
            // 拒可写：把读回全零的那几段在盘上真的写成零，当成一份不带读故障的崩溃镜像交判官复算（判官的 selected_version 取这份镜像上恢复落到的那一版）。
            let zeroed = image_with_the_ranges_zeroed(crash, hidden);
            let report = recover(&zeroed, JournalPolicy::Consult);
            let judge = verdict_label(&zeroed, report.effective_root, &error, stream.operation_count() == 0);
            return format!(
                "OPUS {tag} recF={rec_under_faults} roF={ro_under_faults} M1={} writes_before_refusal={} judge_on_zeroed={judge} judge_selected={}",
                short_mount_error(&error),
                stream.operation_count(),
                report.effective_root.map(|key| name_of(recorded, key)).unwrap_or_default()
            )
        }
    };
    let effective = (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg);
    let isolated: Vec<u64> = mounted.allocator.devices.iter().map(|d| d.isolated_slots()).collect();
    let extra: Vec<(Key, Vec<u8>)> = std::iter::once(&mounted.output.row_publish)
        .chain(&mounted.output.warm_up_publishes)
        .map(|v| {
            let root = v.root();
            ((root.instance, root.checkpoint_txg), recorded.pool.content_of_each_root.get(&effective).cloned().unwrap_or_default())
        })
        .collect();
    let (mount_writes, segments, _i) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
    let row_root = mount_writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("写行那次有根槽写");
    let mut before_row_root = crash.clone();
    before_row_root.apply_writes(&mount_writes[..row_root]);
    let overwritten_before_row_root = units_of_r_overwritten_on(recorded, &before_row_root).len();
    let chk_before = checker_names(&before_row_root);
    let rec_before = recovery_label(recorded, &recover(&before_row_root, JournalPolicy::Consult));
    let l0 = if layer0 && (overwritten_before_row_root > 0 || std::env::var("OPUS_L0_ALWAYS").is_ok()) {
        layer0_over_the_stream_of_m1(recorded, crash, &mount_writes, &segments, &extra)
    } else {
        "L0 -".to_string()
    };
    let mut worst = (0usize, String::new());
    let mut nodes = 0usize;
    if depth > 0 {
        let mut devices = devices_reading_back_all_zero_on(crash, &ranges);
        let mounted_again = mount_writable(&recorded.pool.parameters, &mut devices).expect("同一份盘面第二次挂得上");
        let image = image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
        let pool = pool_after_the_mount(recorded, &image, mounted_again);
        let mut stack: Vec<(String, PoolUnderTest<SparseBlockDevice>)> = vec![(String::new(), pool)];
        while let Some((sequence, node)) = stack.pop() {
            nodes += 1;
            let image = node.image();
            let in_ring = ring_keys(&image).contains(&recorded.key);
            let overwritten = units_of_r_overwritten_on(recorded, &image).len();
            if in_ring && overwritten > worst.0 {
                worst = (overwritten, if sequence.is_empty() { "-".to_string() } else { sequence.clone() });
            }
            if sequence.len() < depth {
                for op in ['o', 's', 'n'] {
                    let mut child = clone_pool(&node);
                    let outcome = match op {
                        'o' => child.overwrite(OVERWRITE_BYTES).map(|_| ()),
                        'n' => child.create_one_inode().map(|_| ()),
                        _ => child.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity()).map(|_| ()),
                    };
                    if outcome.is_ok() {
                        stack.push((format!("{sequence}{op}"), child));
                    }
                }
            }
        }
    }
    format!(
        "OPUS {tag} recF={rec_under_faults} roF={ro_under_faults} M1=Ok effective={} isolated={isolated:?} before_row_root:r_copies_overwritten={overwritten_before_row_root} chk={chk_before:?} rec0={rec_before} sweep(depth={depth},nodes={nodes}):max_r_copies_overwritten_while_r_in_ring={} at={} {l0}",
        name_of(recorded, effective),
        worst.0,
        if worst.1.is_empty() { "-" } else { &worst.1 }
    )
}

/// `image` 上 `ranges` 那几段写成全零（盘上的字节真的变了）。
fn image_with_the_ranges_zeroed(image: &MemoryPool, ranges: &[Range]) -> MemoryPool {
    let mut zeroed = image.clone();
    for (device, offset, length) in ranges {
        let length = usize::try_from(*length).expect("段长");
        zeroed.devices.get_mut(device).expect("盘").write(DeviceOffsetInBytes(*offset), &vec![0u8; length]);
    }
    zeroed
}

/// 大小 ≤ max 的全部子集（按大小、再按字典序）。
fn subsets_up_to(n: usize, max: usize) -> Vec<Vec<usize>> {
    let mut out = vec![Vec::new()];
    let mut frontier = vec![Vec::new()];
    for _size in 1..=max {
        let mut next = Vec::new();
        for subset in &frontier {
            let start = subset.last().map_or(0, |last| last + 1);
            for item in start..n {
                let mut grown: Vec<usize> = subset.clone();
                grown.push(item);
                next.push(grown);
            }
        }
        out.extend(next.iter().cloned());
        frontier = next;
    }
    out
}

/// 扫：R 的哪几个单元读回全零（大小 ≤ OPUS_MAXSUB 的全部子集；OPUS_UNITS 给了就只取点名的组号，逗号分隔、组内用 + 连）×
/// R 的哪几条记录读回全零（OPUS_RECMASKS：all 全部子集、full 只取全藏、none 只取不藏，或逗号分隔的掩码）× R 的根槽藏不藏（OPUS_ROOT_HIDDEN）。
#[test]
fn opus_sweep() {
    let depth = env_usize("OPUS_DEPTH", 2);
    let max_subset = env_usize("OPUS_MAXSUB", 1);
    let layer0 = std::env::var("OPUS_L0").is_ok();
    for kind in env_list("OPUS_KINDS", "ow,sw3") {
        for overwrites_before in env_list("OPUS_KS", "5").iter().map(|v| v.parse::<u64>().expect("OPUS_KS")) {
            let recorded = record_one_publish_after_overwrites(overwrites_before, publish_kind(&kind));
            let crash = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let groups = recorded.unit_writes_grouped_by_unit();
            let roles = roles_of_the_unit_groups(&recorded, &crash);
            let record_count = recorded.record_count();
            println!(
                "OPUS INFO kind={kind} k={overwrites_before} R={} P={} groups={} records={record_count} roles={:?}",
                k(recorded.key),
                k(recorded.previous_key),
                groups.len(),
                roles.iter().enumerate().map(|(i, r)| format!("u{i}:{r}")).collect::<Vec<_>>()
            );
            let unit_subsets: Vec<Vec<usize>> = match std::env::var("OPUS_UNITS") {
                Ok(list) => list.split(',').map(|s| if s == "none" { Vec::new() } else { s.split('+').map(|u| u.parse().expect("组号")).collect() }).collect(),
                Err(_) => subsets_up_to(groups.len(), max_subset),
            };
            let all_masks: Vec<u32> = (0u32..(1 << record_count)).collect();
            let full = (1u32 << record_count) - 1;
            let masks: Vec<u32> = match std::env::var("OPUS_RECMASKS").as_deref() {
                Ok("full") => vec![full],
                Ok("none") => vec![0],
                Ok("all") | Err(_) => all_masks,
                Ok(list) => list.split(',').map(|m| u32::from_str_radix(m, 2).expect("掩码")).collect(),
            };
            for subset in &unit_subsets {
                let mut unit_ranges: Vec<Range> = Vec::new();
                for &g in subset {
                    unit_ranges.extend(ranges_of(&groups[g]));
                }
                let unit_name = if subset.is_empty() { "none".to_string() } else { subset.iter().map(|g| format!("u{g}{}", roles[*g])).collect::<Vec<_>>().join("+") };
                for &record_mask in &masks {
                    for root_hidden in [false, true].into_iter().filter(|hidden| std::env::var("OPUS_ROOT_HIDDEN").map_or(true, |only| only == hidden.to_string())) {
                        let mut hidden = unit_ranges.clone();
                        for ordinal in 1..=record_count {
                            if record_mask & (1 << (ordinal - 1)) != 0 {
                                hidden.extend(ranges_of(&recorded.record_writes_of_the_record(ordinal)));
                            }
                        }
                        if root_hidden {
                            hidden.extend(ranges_of(&[recorded.root_slot_write()]));
                        }
                        let tag = format!("CB kind={kind} k={overwrites_before} unit={unit_name} rec_hidden={record_mask:0width$b} root_hidden={root_hidden}", width = record_count);
                        println!("{}", one_cell(&recorded, &crash, &hidden, depth, layer0, &tag));
                    }
                }
            }
        }
    }
}

/// 拒可写的判官（崩溃注入第三截同一个入口）：崩溃镜像注入过读故障（造这份盘面的 M1 在读故障下跑），被拒的 M2 读的就是崩溃镜像；
/// 只读读出按「在允许集合里」交（这里不算 oracle）。
fn verdict_label(image: &MemoryPool, selected: Option<Key>, error: &MountError, disk_unchanged: bool) -> String {
    match judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
        image,
        selected,
        error,
        if disk_unchanged { DiskAfterTheRefusal::ByteForByteUnchanged } else { DiskAfterTheRefusal::Changed },
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        ReadFaultsInTheCrashState::Injected,
        ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults,
    ) {
        WritableMountRefusalVerdict::Legal(_) => "Legal".to_string(),
        WritableMountRefusalVerdict::Illegal { conditions_not_met, .. } => format!("Illegal{conditions_not_met:?}"),
        WritableMountRefusalVerdict::ReasonNotOnTheD23List => "NotOnList".to_string(),
    }
}

/// M1 的读故障：OPUS_M1_HIDE 里每一项是 + 连起来的几样：root（R 的根槽）、rec（R 的全部记录）、rec<n>（R 的第 n 条记录）、u<g>（R 的第 g 组单元）。
fn hidden_of_the_form(recorded: &RecordedPublish, form: &str) -> Vec<Range> {
    let groups = recorded.unit_writes_grouped_by_unit();
    let mut hidden = Vec::new();
    for part in form.split('+') {
        if part == "root" {
            hidden.extend(ranges_of(&[recorded.root_slot_write()]));
        } else if part == "rec" {
            for ordinal in 1..=recorded.record_count() {
                hidden.extend(ranges_of(&recorded.record_writes_of_the_record(ordinal)));
            }
        } else if let Some(n) = part.strip_prefix("rec") {
            hidden.extend(ranges_of(&recorded.record_writes_of_the_record(n.parse().expect("记录序号"))));
        } else if let Some(g) = part.strip_prefix('u') {
            hidden.extend(ranges_of(&groups[g.parse::<usize>().expect("组号")]));
        } else if part != "none" {
            panic!("不认识的读故障：{part}");
        }
    }
    hidden
}

/// 录的 R 里跨线程用得到的那几样（录的池带 Rc，不能跨线程）。
struct RView {
    parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
    key: Key,
    previous_key: Key,
    unit_writes: Vec<RetainedWrite>,
}

impl RView {
    fn of(recorded: &RecordedPublish) -> Self {
        RView {
            parameters: recorded.pool.parameters.clone(),
            key: recorded.key,
            previous_key: recorded.previous_key,
            unit_writes: recorded.unit_writes().into_iter().cloned().collect(),
        }
    }
    fn name(&self, key: Key) -> String {
        if key == self.key {
            "R".to_string()
        } else if key == self.previous_key {
            "P".to_string()
        } else {
            k(key)
        }
    }
    /// 同 `units_of_r_overwritten_on`。
    fn overwritten_on(&self, image: &MemoryPool) -> Vec<(DeviceIdentity, u64)> {
        self.unit_writes
            .iter()
            .filter(|write| {
                let length = usize::try_from(write.length_in_bytes()).expect("单元写长");
                let read_back = image.devices[&write.device].read(write.offset, length);
                !write.contents.still_on_disk(&read_back)
            })
            .map(|write| (write.device, write.offset.0))
            .collect()
    }
}

/// 一个二次崩溃状态（`image`，不带读故障）上可写挂 M2，交回它那一类的描述。`rec` 与 `effective_root` 是枚举器在同一份镜像上跑的恢复。
fn class_of_the_second_mount(view: &RView, image: &MemoryPool, rec: &str, effective_root: Option<Key>) -> String {
    let r_in_ring = ring_keys(image).contains(&view.key);
    let already: BTreeSet<(DeviceIdentity, u64)> = view.overwritten_on(image).into_iter().collect();
    let stream2 = SharedStream::retaining_contents();
    let mut devices2 = recorded_devices_reading_back_all_zero_on(image, &RangesReadingBackAllZero::default(), &stream2);
    match mount_writable(&view.parameters, &mut devices2) {
        Ok(m2) => {
            let eff2 = (m2.output.effective_root.instance, m2.output.effective_root.checkpoint_txg);
            let (w2, _s2, _i2) = writes_and_segments_with_stream_indexes(&stream2.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
            let row2 = w2.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(w2.len());
            let mut before = image.clone();
            before.apply_writes(&w2[..row2]);
            let newly = view.overwritten_on(&before).into_iter().filter(|u| !already.contains(u)).count();
            format!(
                "M2=Ok eff={} rec={rec} div={} r_in_ring={r_in_ring} r_overwritten_already={} r_newly_overwritten_by_m2_before_row_root={newly}",
                view.name(eff2),
                Some(eff2) != effective_root,
                already.len()
            )
        }
        Err(error) => {
            let unchanged = stream2.operation_count() == 0;
            format!(
                "M2={} rec={rec} verdict={} r_in_ring={r_in_ring} r_overwritten_already={}",
                short_mount_error(&error),
                verdict_label(image, effective_root, &error, unchanged),
                already.len()
            )
        }
    }
}

/// 二次崩溃：M1 在 OPUS_M1_HIDE 那组读故障下做可写挂载（录流）；对 M1 的录制流跑层 0 全量（每一段都展开），每个崩溃状态上不带读故障
/// 再可写挂 M2。按类数状态：M2 做成时落到哪一版、与崩溃镜像上恢复落到的那一版同不同（div）、R 的根在不在环里、M2 写行根落盘之前新写掉 R 的几份单元；
/// M2 被拒时拒因与判官的判定。每一类打一行，带第一个状态的持久位作样本。
#[test]
fn opus_second_crash() {
    for kind in env_list("OPUS_KINDS", "ow,sw3") {
        for overwrites_before in env_list("OPUS_KS", "5").iter().map(|v| v.parse::<u64>().expect("OPUS_KS")) {
            let recorded = record_one_publish_after_overwrites(overwrites_before, publish_kind(&kind));
            let crash = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let roles = roles_of_the_unit_groups(&recorded, &crash);
            println!(
                "OPUS SCINFO kind={kind} k={overwrites_before} R={} P={} records={} roles={:?}",
                k(recorded.key),
                k(recorded.previous_key),
                recorded.record_count(),
                roles.iter().enumerate().map(|(i, r)| format!("u{i}:{r}")).collect::<Vec<_>>()
            );
            for form in env_list("OPUS_M1_HIDE", "root+rec") {
                let hidden = hidden_of_the_form(&recorded, &form);
                let tag = format!("SC kind={kind} k={overwrites_before} m1_hide={form}");
                let stream = SharedStream::retaining_contents();
                let mut devices = recorded_devices_reading_back_all_zero_on(&crash, &RangesReadingBackAllZero(hidden.clone()), &stream);
                let mounted = match mount_writable(&recorded.pool.parameters, &mut devices) {
                    Ok(mounted) => mounted,
                    Err(error) => {
                        println!("OPUS {tag} M1={}", short_mount_error(&error));
                        continue;
                    }
                };
                let effective = (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg);
                let extra: Vec<(Key, Vec<u8>)> = std::iter::once(&mounted.output.row_publish)
                    .chain(&mounted.output.warm_up_publishes)
                    .map(|v| {
                        let root = v.root();
                        ((root.instance, root.checkpoint_txg), recorded.pool.content_of_each_root.get(&effective).cloned().unwrap_or_default())
                    })
                    .collect();
                let (writes, segments, _i) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
                let closed = closed_form_state_count(&segments);
                let segment_sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
                println!("OPUS {tag} M1=Ok effective={} writes={} segments={segment_sizes:?} closed={closed}", name_of(&recorded, effective), writes.len());
                if closed > layer0_cap() {
                    println!("OPUS {tag} SKIP closed={closed}");
                    continue;
                }
                let judged_root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(writes.len() - 1);
                let versions = versions_of(&recorded, &extra);
                // 观察者只收每个状态的持久位与恢复的读数（在调用线程上、按状态序号），M2 那一挂放到后面分线程做。
                let mut collected: Vec<(Vec<bool>, String, Option<Key>)> = Vec::new();
                // 枚举用的写表（录制流原样在前，原地覆写的撕裂镜像接在后面），每个状态的持久位与它逐条对应；取第一个状态交来的那一份。
                let mut table: Option<Vec<RetainedWrite>> = None;
                let _tally = enumerate_layer0_selecting_versions_observing_each_state(&crash, &writes, &segments, judged_root_index, &versions, &|_, _| true, &mut |state, report| {
                    if table.is_none() {
                        table = Some(state.writes.to_vec());
                    }
                    assert_eq!(state.persisted.len(), table.as_ref().map_or(0, Vec::len), "每个状态的写表同一份");
                    collected.push((state.persisted.clone(), recovery_label(&recorded, report), report.effective_root));
                });
                let table = table.unwrap_or_default();
                println!("OPUS {tag} TABLE recorded_writes={} enumeration_table={}", writes.len(), table.len());
                let states = collected.len() as u64;
                let view = RView::of(&recorded);
                let threads = std::env::var("SINGLEFS_THREAD_CAP").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(4).max(1);
                let chunk = collected.len().div_ceil(threads).max(1);
                let parts: Vec<BTreeMap<String, (u64, String)>> = std::thread::scope(|scope| {
                    let handles: Vec<_> = collected
                        .chunks(chunk)
                        .map(|slice| {
                            let view = &view;
                            let crash = &crash;
                            let writes = &table;
                            scope.spawn(move || {
                                let mut classes: BTreeMap<String, (u64, String)> = BTreeMap::new();
                                for (persisted, rec, effective_root) in slice {
                                    let mut image = crash.clone();
                                    let applied: Vec<RetainedWrite> = writes.iter().zip(persisted).filter(|(_, p)| **p).map(|(w, _)| w.clone()).collect();
                                    image.apply_writes(&applied);
                                    let class = class_of_the_second_mount(view, &image, rec, *effective_root);
                                    let sample: String = persisted.iter().map(|p| if *p { '1' } else { '0' }).collect();
                                    classes.entry(class).or_insert((0, sample)).0 += 1;
                                }
                                classes
                            })
                        })
                        .collect();
                    handles.into_iter().map(|h| h.join().expect("工作线程")).collect()
                });
                let mut classes: BTreeMap<String, (u64, String)> = BTreeMap::new();
                for part in parts {
                    for (class, (count, sample)) in part {
                        let entry = classes.entry(class).or_insert((0, sample.clone()));
                        entry.0 += count;
                        if sample < entry.1 {
                            entry.1 = sample;
                        }
                    }
                }
                println!("OPUS {tag} STATES enumerated={states} closed={closed}");
                for (class, (count, sample)) in &classes {
                    println!("OPUS {tag} CLASS count={count} {class} sample={sample}");
                }
            }
        }
    }
}

/// F2_NEWEST 那一退在 H4 那一形上再加一样故障：R 没见证（崩在根槽之后、轮换之前），P 已确认；读的那一刻读回全零的是
/// R 的单元（Ru）、每块盘较新的那个系统配置槽（cfg，P 那次轮换写的）、P 的根槽（Proot）、R 的根指着的实例表那一片（Itab）、P 的记录（Prec）
/// 这几样的组合。打印恢复、只读挂载、可写挂载各落到哪一版（R、P、Pm = P 之前那一版）。
#[test]
fn opus_f2_newest_with_one_more_fault() {
    for kind in env_list("OPUS_KINDS", "ow,sw3") {
        for overwrites_before in env_list("OPUS_KS", "3,5").iter().map(|v| v.parse::<u64>().expect("OPUS_KS")) {
            let recorded = record_one_publish_after_overwrites(overwrites_before, publish_kind(&kind));
            let crash = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let p = recorded.previous_key;
            let pm = ring_keys(&crash).into_iter().filter(|key| (key.1, key.0) < (p.1, p.0)).max_by_key(|key| (key.1, key.0)).expect("P 之前一版");
            let configuration = choose_system_configuration(&crash).expect("系统配置");
            let spacing = u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
            let mut newer_slots: Vec<Range> = Vec::new();
            for (device, sparse) in &crash.devices {
                let mut best: Option<(u64, u64)> = None;
                for offset in [0, spacing] {
                    let bytes = sparse.read(DeviceOffsetInBytes(offset), 4096);
                    if let Ok(parsed) = singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes) {
                        if best.map_or(true, |b| parsed.quantities.slot_generation > b.0) {
                            best = Some((parsed.quantities.slot_generation, offset));
                        }
                    }
                }
                newer_slots.push((*device, best.expect("每块盘有自证过的槽").1, 4096));
            }
            let p_root_slot = RangesReadingBackAllZero::default().with_the_root_slot_of(&crash, p).0;
            let r_root = recorded.root();
            let instance_table_of_r: Vec<Range> = r_root.instance_table.locations.iter().map(|l| (l.device, l.slot.to_device_offset().0, singlefs_format::DATA_UNIT_BYTES)).collect();
            let p_last_record = RangesReadingBackAllZero::default().with_the_last_record_of_the_publish(&crash, p).0;
            let name = |key: Key| if key == recorded.key { "R".to_string() } else if key == p { "P".to_string() } else if key == pm { "Pm".to_string() } else { k(key) };
            println!("OPUS F2INFO kind={kind} k={overwrites_before} R={} P={} Pm={}", k(recorded.key), k(p), k(pm));
            let parts: Vec<(&str, Vec<Range>)> = vec![
                ("Ru", ranges_of(&recorded.unit_writes())),
                ("cfg", newer_slots.clone()),
                ("Proot", p_root_slot.clone()),
                ("Itab", instance_table_of_r.clone()),
                ("Prec", p_last_record.clone()),
            ];
            for form in env_list("OPUS_F2_FORMS", "Ru,Ru+cfg,Ru+Proot,Ru+cfg+Proot,Ru+cfg+Itab,Ru+cfg+Prec,Ru+cfg+Proot+Itab") {
                let mut hidden: Vec<Range> = Vec::new();
                for part in form.split('+') {
                    hidden.extend(parts.iter().find(|(n, _)| *n == part).expect("认得的那几样").1.iter().copied());
                }
                let ranges = RangesReadingBackAllZero(hidden);
                let devices = devices_reading_back_all_zero_on(&crash, &ranges);
                let report = recover(&devices, JournalPolicy::Consult);
                let rec = match &report.outcome {
                    RecoveryOutcome::FileRead { root, .. } => format!("rec=FileRead({})", name(*root)),
                    RecoveryOutcome::NoFile { root } => format!("rec=NoFile({})", name(*root)),
                    RecoveryOutcome::Failed { root, failure } => format!("rec=Failed({} {})", root.map(name).unwrap_or_default(), kind_of(&format!("{failure:?}"))),
                };
                let ro = match mount_read_only(&devices) {
                    Ok(mounted) => format!("ro=Ok({})", name((mounted.effective_root.instance, mounted.effective_root.checkpoint_txg))),
                    Err(error) => format!("ro=Err({})", kind_of(&format!("{error:?}"))),
                };
                let mut devices_of_the_mount = devices_reading_back_all_zero_on(&crash, &ranges);
                let rw = match mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount) {
                    Ok(mounted) => format!("rw=Ok({})", name((mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg))),
                    Err(error) => format!("rw={}", short_mount_error(&error)),
                };
                println!("OPUS F2 kind={kind} k={overwrites_before} hidden={form} {rec} {ro} {rw}");
            }
        }
    }
}
