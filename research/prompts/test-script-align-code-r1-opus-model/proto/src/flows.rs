//! 拷自 crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs 的造流部分（record_flow、content_of、
//! RecordedFlow、expansion、with_the_overwritten_version_miswritten、record_and_check），盘换成内存稀疏盘（crate::common）。
use std::path::PathBuf;

use crate::common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use crate::common_crash_points::{overwrite_step, record_pool_build};
use singlefs_checker_tier::crash::Layer0SegmentExpansion;
use singlefs_checker_tier::crash_amplification::{
    check_recorded_blocks, record_crash_points, CheckingTally, CrashFlow, CrashPlan,
    CrashPointRecorder, CrashPointSpan, UnitChecksumsOn, WeightedShares,
};
use singlefs_checker_tier::crash_identity::CoverageReport;
use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

pub const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;

pub fn expansion(_segment_index: usize, segment: &[usize]) -> Layer0SegmentExpansion {
    if segment.len() <= LARGEST_EXPANDED_SEGMENT_WRITES {
        Layer0SegmentExpansion::EveryProperSubset
    } else {
        Layer0SegmentExpansion::NotExpanded
    }
}

pub fn content_of(seed: usize, length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

#[derive(Clone)]
pub struct RecordedFlow {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub judged_root_index: usize,
    pub versions: Vec<PublishedVersion>,
    pub crash_points: Vec<CrashPointSpan>,
}

impl RecordedFlow {
    pub fn flow(&self) -> CrashFlow<'_> {
        CrashFlow {
            base: &self.base,
            writes: &self.writes,
            segments: &self.segments,
            judged_root_index: self.judged_root_index,
            versions: &self.versions,
            expansion: &expansion,
            crash_points: self.crash_points.clone(),
        }
    }
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

pub fn record_flow(tag: &str, overwrite_content: &[u8], tail_set_in_common: bool) -> RecordedFlow {
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
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    RecordedFlow {
        base,
        writes,
        segments,
        judged_root_index,
        versions: vec![
            PublishedVersion { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3), content: file_content() },
            PublishedVersion { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4), content: overwrite_content.to_vec() },
        ],
        crash_points,
    }
}

/// 同两流用例里的那一个：版本表里覆盖写那一版（txg 4）的内容改坏一个字节，写表一个字节都不动。
pub fn with_the_overwritten_version_miswritten(mut recorded: RecordedFlow) -> RecordedFlow {
    let overwritten = recorded
        .versions
        .iter_mut()
        .find(|version| version.checkpoint_txg == CheckpointTxg(4))
        .expect("版本表里有覆盖写那一版");
    overwritten.content[0] ^= 0x01;
    recorded
}

pub fn record_and_check(
    store: &mut VerdictStore,
    recorded: &RecordedFlow,
    plan: &CrashPlan,
    version: &JudgeVersion,
) -> (u64, CheckingTally) {
    let recording = record_crash_points(store, plan, version, &mut CoverageReport::default()).expect("录得进");
    let tally = check_recorded_blocks(
        store,
        &recorded.flow(),
        plan,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        version,
    )
    .expect("核得完");
    (recording.blocks_already_judged, tally)
}

pub fn repository_root() -> PathBuf {
    std::env::var_os("K2_REPOSITORY_ROOT").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tree").canonicalize().expect("副本仓根"),
        PathBuf::from,
    )
}

pub fn fresh_store(name: &str) -> (PathBuf, VerdictStore) {
    let directory = std::env::var_os("K2_SCRATCH")
        .map_or_else(std::env::temp_dir, PathBuf::from)
        .join(format!("k2-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let store = VerdictStore::create_empty(&directory).expect("建得了库");
    (directory, store)
}
