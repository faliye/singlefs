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
