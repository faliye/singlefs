//! 几条流的造法，照仓里用例的造流部分拷来（盘换成内存稀疏盘）：
//! - `two_flow`：crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs 的 record_flow；
//! - `rollback_flow`：…/crash_amplification_fixed_script_through_rollback.rs 到 D 的那一段；
//! - `unmount_flow`：…/crash_amplification_fixed_script_through_unmount.rs 到正常卸载的那一段。
//! 展开方式按 `expand_up_to` 取（同 `ExpandUpToWritesPerSegment::expansion_of`）。
use crate::common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use crate::common_crash_points::{
    mount_writable_step, overwrite_step, raise_rollback_floor_step, record_pool_build, roll_back_step,
    unmount_step,
};
use singlefs_checker_tier::crash::Layer0SegmentExpansion;
use singlefs_checker_tier::crash_amplification::{
    CrashFlow, CrashPointRecorder, CrashPointSpan, ExpandUpToWritesPerSegment,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::RollbackTarget;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, writes_and_segments_with_stream_indexes_and_entries,
    MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::RecordedEntrySpan;

pub struct RecordedFlow {
    pub name: String,
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub judged_root_index: usize,
    pub versions: Vec<PublishedVersion>,
    pub crash_points: Vec<CrashPointSpan>,
    pub expand_up_to: usize,
    expansion: Box<dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync>,
}

fn expansion_up_to(expand_up_to: usize) -> Box<dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync> {
    let limit = ExpandUpToWritesPerSegment(expand_up_to);
    Box::new(move |_segment_index: usize, segment: &[usize]| limit.expansion_of(segment))
}

impl Clone for RecordedFlow {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            base: self.base.clone(),
            writes: self.writes.clone(),
            segments: self.segments.clone(),
            judged_root_index: self.judged_root_index,
            versions: self.versions.clone(),
            crash_points: self.crash_points.clone(),
            expand_up_to: self.expand_up_to,
            expansion: expansion_up_to(self.expand_up_to),
        }
    }
}

impl RecordedFlow {
    pub fn flow(&self) -> CrashFlow<'_> {
        CrashFlow {
            base: &self.base,
            writes: &self.writes,
            segments: &self.segments,
            judged_root_index: self.judged_root_index,
            versions: &self.versions,
            expansion: &*self.expansion,
            crash_points: self.crash_points.clone(),
        }
    }
    pub fn expansion(&self) -> &(dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync) {
        &*self.expansion
    }
    pub fn renamed(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}

pub fn content_of(seed: usize, length: usize) -> Vec<u8> {
    (0..length).map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256")).collect()
}

fn version(instance: u32, txg: u64, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion { instance: InstanceGeneration(instance), checkpoint_txg: CheckpointTxg(txg), content }
}

fn last_root(writes: &[RetainedWrite]) -> usize {
    writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("根槽写")
}

fn overwrite_step_set_here(recorder: &mut CrashPointRecorder, pool: &mut BuiltPool, content: &[u8]) {
    recorder.step(file!(), "publish_overwrite", || {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.output,
            FirstFile { content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            InstanceGeneration(1),
        )
        .expect("覆盖写");
    });
}

/// 两流用例的 record_flow：mkfs → 取号加暖机 → 新池新建文件 → 一次覆盖写。
pub fn two_flow(tag: &str, overwrite_content: &[u8], tail_set_in_common: bool, expand_up_to: usize) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    if tail_set_in_common {
        overwrite_step(&mut recorder, &mut pool, overwrite_content, InstanceGeneration(1));
    } else {
        overwrite_step_set_here(&mut recorder, &mut pool, overwrite_content);
    }
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations[pool.mkfs_operation_count..], &geometry());
    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
    let judged_root_index = last_root(&writes);
    RecordedFlow {
        name: tag.to_string(),
        base,
        writes,
        segments,
        judged_root_index,
        versions: vec![version(1, 3, file_content()), version(1, 4, overwrite_content.to_vec())],
        crash_points,
        expand_up_to,
        expansion: expansion_up_to(expand_up_to),
    }
}

fn second_content() -> Vec<u8> {
    content_of(3, 4100)
}
fn third_content() -> Vec<u8> {
    (0..2500usize).map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256")).collect()
}
fn later_content(seed: usize) -> Vec<u8> {
    (0..3000 + seed).map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256")).collect()
}

/// 固定脚本走到 D（回退），`through_unmount` 时接着五次覆盖写、抬 F、E、正常卸载（两条固定脚本用例的造流部分）。
fn fixed_script(tag: &str, through_unmount: bool, expand_up_to: usize) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    let mut versions = vec![version(1, 3, file_content())];
    let second = overwrite_step(&mut recorder, &mut pool, &second_content(), InstanceGeneration(1));
    versions.push(version(1, 4, second_content()));
    pool.output = second;
    let mounted = mount_writable_step(&mut recorder, &mut pool);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current.into_file_version().expect("B 之后重开，现行那一版带文件");
    for txg in 5..=7 {
        versions.push(version(2, txg, second_content()));
    }
    let third = overwrite_step(&mut recorder, &mut pool, &third_content(), InstanceGeneration(2));
    versions.push(version(2, 8, third_content()));
    pool.output = third;
    roll_back_step(
        &mut recorder,
        &mut pool,
        RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) },
    );
    versions.push(version(2, 9, file_content()));
    if through_unmount {
        let mut latest = Vec::new();
        for (txg, seed) in [(10u64, 13usize), (11, 17), (12, 19), (13, 23), (14, 29)] {
            latest = later_content(seed);
            pool.output = overwrite_step(&mut recorder, &mut pool, &latest, InstanceGeneration(2));
            versions.push(version(2, txg, latest.clone()));
        }
        let raised = raise_rollback_floor_step(&mut recorder, &mut pool, CheckpointTxg(11));
        assert_eq!(raised.publishes.len(), 2);
        for txg in 15..=16 {
            versions.push(version(2, txg, latest.clone()));
        }
        let reuse = overwrite_step(&mut recorder, &mut pool, &later_content(31), InstanceGeneration(2));
        versions.push(version(2, 17, later_content(31)));
        pool.output = reuse;
        let _unmounted = unmount_step(&mut recorder, &mut pool);
        for txg in 18..=19 {
            versions.push(version(2, txg, later_content(31)));
        }
    }
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let entry_spans: Vec<RecordedEntrySpan> = pool
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - pool.mkfs_operation_count
                ..span.operations.end - pool.mkfs_operation_count,
        })
        .collect();
    // 到 D 那条用例切段不带入口段，到卸载那条带（两份用例原样）
    let (writes, segments, stream_indexes) = if through_unmount {
        writes_and_segments_with_stream_indexes_and_entries(
            &operations[pool.mkfs_operation_count..],
            &entry_spans,
            &geometry(),
        )
    } else {
        writes_and_segments_with_stream_indexes(&operations[pool.mkfs_operation_count..], &geometry())
    };
    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
    let judged_root_index = last_root(&writes);
    RecordedFlow {
        name: tag.to_string(),
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        crash_points,
        expand_up_to,
        expansion: expansion_up_to(expand_up_to),
    }
}

pub fn rollback_flow(tag: &str, expand_up_to: usize) -> RecordedFlow {
    fixed_script(tag, false, expand_up_to)
}

pub fn unmount_flow(tag: &str, expand_up_to: usize) -> RecordedFlow {
    fixed_script(tag, true, expand_up_to)
}

// ============================== test-script-align-code-r3 攻方腿加的：基镜像是一个崩溃状态的流 ==============================

/// 两流用例那条流（mkfs → 取号加暖机 → 新池新建文件 txg 3 → 覆盖写 txg 4）崩在覆盖写的 journal 记录落盘之后、根槽写落盘之前：
/// 把这个崩溃状态当基镜像（记录在盘上、它那一代根不在），重开可写挂载（崩溃之后的重开，与 `crash_state_devices` 那几条 harness 用例同一种起法），
/// 再发一次覆盖写（实例 2）。录下的是重开那一段与之后那次覆盖写。
/// `version_4_miswritten` 设了时版本表里实例 1 第 4 代那一版改坏一个字节（R1 那一对的造法），别的逐字相同。
pub struct CrashedBaseFlow {
    pub flow: RecordedFlow,
    /// 基镜像里带根 magic 的扇区读出的（实例，txg）。
    pub base_roots: Vec<(u32, u64)>,
    /// 原流里被截掉的那条根槽写（覆盖写那一次）的（实例，txg）。
    pub cut_root: (u32, u64),
}

pub fn crashed_base_then_mount(tag: &str, expand_up_to: usize, version_4_miswritten: bool) -> CrashedBaseFlow {
    use crate::common::{crash_state_devices, parameters};
    use singlefs_core::mount::mount_writable;
    use singlefs_harness::memory_pool::root_identity_written_by;
    use singlefs_harness::SharedStream;

    let content_a = content_of(11, 4100);
    let content_b = content_of(23, 3000);
    // 1. 原流：录到覆盖写为止，找覆盖写那一次的根槽写
    let mut pool = build_pool(tag);
    let mut first_recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut first_recorder, &pool);
    overwrite_step(&mut first_recorder, &mut pool, &content_a, InstanceGeneration(1));
    let after_mkfs = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes_before, _segments_before, _indexes) =
        writes_and_segments_with_stream_indexes(&operations[pool.mkfs_operation_count..], &geometry());
    let cut = last_root(&writes_before);
    let cut_identity = singlefs_harness::memory_pool::root_identity_of_write(&writes_before[cut]);
    assert!(
        writes_before[..cut].iter().rev().take_while(|write| write.kind != StepKind::RootRecordFua).any(|write| write.kind == StepKind::JournalRecord),
        "覆盖写那一次的根槽写之前有它的 journal 记录"
    );
    // 2. 基镜像：mkfs 之后施加覆盖写根槽写之前的每一次写（崩溃状态：记录落了、根没落）
    let mut base = after_mkfs.clone();
    base.apply_writes(&writes_before[..cut]);
    let base_roots: Vec<(u32, u64)> = base
        .devices
        .values()
        .flat_map(|device| device.written_sectors().values().cloned().collect::<Vec<_>>())
        .filter(|bytes| bytes.len() >= 36 && bytes[..4] == singlefs_core::root_record::ROOT_MAGIC[..4])
        .map(|bytes| {
            let (instance, txg) = root_identity_written_by(&bytes);
            (instance.0, txg.0)
        })
        .collect();
    // 3. 崩溃之后重开（录进一条新流），再发一次覆盖写
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&base, &[], &[], &stream);
    let mut recorder = CrashPointRecorder::new(stream.clone(), 0);
    let mounted = recorder.step(file!(), "mount_writable_after_the_crash", || {
        mount_writable(&parameters(), &mut devices).expect("崩溃之后可写挂载")
    });
    let mut allocator = mounted.allocator;
    let output = mounted.current.into_file_version().expect("重开之后现行那一版带文件");
    recorder.step(file!(), "publish_overwrite", || {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &output,
            FirstFile { content: &content_b, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120 },
            InstanceGeneration(2),
        )
        .expect("重开之后覆盖写")
    });
    let operations = stream.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(&operations, &geometry());
    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
    let judged_root_index = last_root(&writes);
    // 4. 版本表：实例 1 第 3 代（新池新建文件）、实例 1 第 4 代（崩掉的那次覆盖写，由记录重建）；重开写出的每一代根都是内容 A，
    //    最后那次覆盖写那一代是内容 B
    let mut versions = vec![version(1, 3, file_content()), version(1, 4, content_a.clone())];
    if version_4_miswritten {
        versions[1].content[0] ^= 0x01;
    }
    let roots: Vec<(u32, u64)> = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .map(|write| {
            let (instance, txg) = singlefs_harness::memory_pool::root_identity_of_write(write);
            (instance.0, txg.0)
        })
        .collect();
    let last = *roots.last().expect("重开之后有根槽写");
    for (instance, txg) in &roots {
        if versions.iter().any(|v| v.instance.0 == *instance && v.checkpoint_txg.0 == *txg) {
            continue;
        }
        let content = if (*instance, *txg) == last { content_b.clone() } else { content_a.clone() };
        versions.push(version(*instance, *txg, content));
    }
    CrashedBaseFlow {
        flow: RecordedFlow {
            name: tag.to_string(),
            base,
            writes,
            segments,
            judged_root_index,
            versions,
            crash_points,
            expand_up_to,
            expansion: expansion_up_to(expand_up_to),
        },
        base_roots,
        cut_root: (cut_identity.0 .0, cut_identity.1 .0),
    }
}
