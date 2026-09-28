//! 公共件：内存稀疏盘上的会话、切段、手摆状态的掩码、逐状态评判与签名。
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    check_records, classified_oracle_violation_for_versions, evaluate_state_for_versions,
    Layer0Tally,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome, RecoveryReport};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    newest_persisted_root, CrashImage, MemoryPool, PublishedVersion, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

pub type Dev = RecordingBlockDevice<SparseBlockDevice>;
pub type Key = (InstanceGeneration, CheckpointTxg);

pub fn arm_name() -> &'static str {
    if std::env::var_os("K39_DROP_UNIT_RECORD_BARRIER").is_some() {
        "mutated(no-1058)"
    } else {
        "control(with-1058)"
    }
}

pub struct Session {
    pub width: HistoryDeviceWidth,
    pub parameters: MakeFilesystemParameters,
    pub devices: Vec<(DeviceIdentity, Dev)>,
    pub stream: SharedStream,
}

impl Session {
    pub fn fresh(width: HistoryDeviceWidth) -> Self {
        let stream = SharedStream::retaining_contents();
        let devices = [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(width.device_bytes(), PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
        Self { width, parameters: width.parameters(), devices, stream }
    }

    /// 从一份内存池起一个新会话（新录制流）：崩溃之后的挂载在它上面写。
    pub fn from_pool(pool: &MemoryPool, width: HistoryDeviceWidth) -> Self {
        let stream = SharedStream::retaining_contents();
        let devices = pool
            .devices
            .iter()
            .map(|(identity, sectors)| {
                let mut device =
                    SparseBlockDevice::new(pool.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
                device.image = sectors.clone();
                (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
            })
            .collect();
        Self { width, parameters: width.parameters(), devices, stream }
    }

    pub fn memory_pool(&self) -> MemoryPool {
        MemoryPool {
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                .collect(),
            device_size_in_bytes: self.width.device_bytes(),
        }
    }

    pub fn operation_count(&self) -> usize {
        self.stream.operation_count()
    }

    pub fn operations(&self) -> Vec<RetainedOperation> {
        self.stream.retained_operations()
    }
}

/// 一段录制流切成写表与段。
pub fn split(ops: &[RetainedOperation], width: HistoryDeviceWidth) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    singlefs_harness::memory_pool::writes_and_segments(ops, &width.fixed_geometry())
}

pub fn materialize(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut pool = base.clone();
    for (write, is_persisted) in writes.iter().zip(persisted) {
        if *is_persisted {
            pool.apply_writes(std::slice::from_ref(write));
        }
    }
    pool
}

/// 段形的一行说明：每段的写数与种类。
pub fn describe_segments(writes: &[RetainedWrite], segments: &[Vec<usize>]) -> String {
    segments
        .iter()
        .map(|segment| {
            let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
            for write in segment {
                *kinds.entry(writes[*write].kind.name()).or_insert(0) += 1;
            }
            format!("[{}:{}]", segment.len(), kinds.iter().map(|(k, n)| format!("{k}x{n}")).collect::<Vec<_>>().join(","))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 简单的可复现随机源（xorshift64*）。
pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
}

/// 一段里要评的掩码（段内下标）。段不超过 `full_limit` 个写就全枚举（去掉整段全持久，与层 0 同一个域）；
/// 更长的段手摆：记录写的每个持久子集 × {单元全落、每个单元两份都扣、每一份单独扣、全扣、只扣盘 0 的、只扣盘 1 的、
/// 数据以外的单元全扣、随机 `random_count` 个}。手摆的不是层 0 的全量，报告里按手摆报。
pub fn masks_for_segment(
    writes: &[RetainedWrite],
    segment: &[usize],
    full_limit: usize,
    random_count: usize,
    data_unit_offsets: &BTreeSet<u64>,
    rng: &mut Rng,
) -> (Vec<Vec<bool>>, bool) {
    let n = segment.len();
    if n <= full_limit {
        let masks = (0..(1u64 << n) - 1)
            .map(|mask| (0..n).map(|bit| mask & (1 << bit) != 0).collect())
            .collect();
        return (masks, true);
    }
    let unit_positions: Vec<usize> =
        (0..n).filter(|p| writes[segment[*p]].kind == StepKind::UnitWrite).collect();
    let record_positions: Vec<usize> =
        (0..n).filter(|p| writes[segment[*p]].kind == StepKind::JournalRecord).collect();
    let other_positions: Vec<usize> = (0..n)
        .filter(|p| !unit_positions.contains(p) && !record_positions.contains(p))
        .collect();
    let mut groups: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for p in &unit_positions {
        groups.entry(writes[segment[*p]].offset.0).or_default().push(*p);
    }
    let mut unit_patterns: Vec<Vec<usize>> = vec![vec![]];
    for copies in groups.values() {
        unit_patterns.push(copies.clone());
    }
    for p in &unit_positions {
        unit_patterns.push(vec![*p]);
    }
    unit_patterns.push(unit_positions.clone());
    for device in [0u32, 1] {
        unit_patterns.push(unit_positions.iter().copied().filter(|p| writes[segment[*p]].device.0 == device).collect());
    }
    unit_patterns.push(unit_positions.iter().copied().filter(|p| !data_unit_offsets.contains(&writes[segment[*p]].offset.0)).collect());
    unit_patterns.push(unit_positions.iter().copied().filter(|p| data_unit_offsets.contains(&writes[segment[*p]].offset.0)).collect());
    for _ in 0..random_count {
        unit_patterns.push(unit_positions.iter().copied().filter(|_| rng.next() & 1 == 1).collect());
    }
    let mut masks: BTreeSet<Vec<bool>> = BTreeSet::new();
    let r = record_positions.len();
    assert!(r <= 12, "记录写太多");
    for record_mask in 0..(1u64 << r) {
        for withheld in &unit_patterns {
            let mut mask = vec![true; n];
            for (k, p) in record_positions.iter().enumerate() {
                mask[*p] = record_mask & (1 << k) != 0;
            }
            for p in withheld {
                mask[*p] = false;
            }
            for p in &other_positions {
                mask[*p] = true;
            }
            if mask.iter().all(|b| *b) {
                continue;
            }
            masks.insert(mask);
        }
    }
    (masks.into_iter().collect(), false)
}

/// 一个状态的结局签名：落到哪一版、读回的是哪一版的内容、oracle / checker / 记录核对器怎么判。
pub fn signature(
    report: &RecoveryReport,
    versions: &[PublishedVersion],
    floor: Option<Key>,
    oracle_kind: Option<String>,
    checker_red: &[&str],
    record_red: (bool, bool),
) -> String {
    let outcome = match &report.outcome {
        RecoveryOutcome::FileRead { content, .. } => {
            let which: Vec<String> = versions
                .iter()
                .filter(|v| &v.content == content)
                .map(|v| format!("({},{})", v.instance.0, v.checkpoint_txg.0))
                .collect();
            format!("FileRead{}", if which.is_empty() { "<unknown-content>".to_string() } else { which.join("|") })
        }
        RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("Failed:{failure:?}"),
    };
    let below_floor = match (floor, report.effective_root) {
        (Some(floor), Some(effective)) => (effective.1, effective.0) < (floor.1, floor.0),
        (Some(_), None) => true,
        _ => false,
    };
    format!(
        "eff={:?} {} vfail={} applied={} below_floor={} oracle={} checker={:?} record=({},{})",
        report.effective_root.map(|(i, t)| (i.0, t.0)),
        outcome,
        report.journal.verification_failed.min(1),
        report.journal.prefix_applied.min(1),
        below_floor,
        oracle_kind.unwrap_or_else(|| "ok".into()),
        checker_red,
        record_red.0,
        record_red.1
    )
}

/// 评一个状态：层 0 那套（看 / 不看 journal 两遍恢复、oracle、池级 checker、记录核对器）记进 `tally`，另算签名。
pub fn evaluate(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: Vec<bool>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
    floor: Option<Key>,
    tally: &mut Layer0Tally,
) -> (String, RecoveryReport) {
    let report = evaluate_state_for_versions(base, writes, persisted.clone(), judged_root_index, versions, tally);
    let image = CrashImage { base, writes, persisted: persisted.clone() };
    let newest = newest_persisted_root(writes, &persisted);
    let oracle = classified_oracle_violation_for_versions(&report.outcome, report.effective_root, newest, versions)
        .map(|v| format!("{:?}", v.kind));
    let checker_red: Vec<&str> = check_pool_image(&image)
        .into_iter()
        .filter_map(|(invariant, verdict)| matches!(verdict, InvariantVerdict::Violated(_)).then_some(invariant))
        .collect();
    let records = check_records(&image, report.effective_root);
    let sig = signature(&report, versions, floor, oracle, &checker_red, (records.root_without_record, records.claimed_state_missing_unit));
    (sig, report)
}

pub fn recover_image(image: &MemoryPool) -> RecoveryReport {
    recover(image, JournalPolicy::Consult)
}

pub fn persisted_of(segments: &[Vec<usize>], write_count: usize, segment_index: usize, mask: &[bool]) -> Vec<bool> {
    let mut persisted = vec![false; write_count];
    for segment in &segments[..segment_index] {
        for w in segment {
            persisted[*w] = true;
        }
    }
    for (k, w) in segments[segment_index].iter().enumerate() {
        persisted[*w] = mask[k];
    }
    persisted
}

pub fn tally_line(name: &str, tally: &Layer0Tally) -> String {
    format!(
        "TALLY {name} states={} violations={} ignored_violations={} checker_violated={:?} record_root_without_record={} record_claimed_state_missing_unit={} verification_failed_states={} root_persisted_states={} failed_states={} first_violation={:?} checker_first_violation={:?}",
        tally.states,
        tally.violations,
        tally.ignored_violations,
        tally.checker_violated_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.verification_failed_states,
        tally.root_persisted_states,
        tally.failed_states,
        tally.first_violation,
        tally.checker_first_violation
    )
}
