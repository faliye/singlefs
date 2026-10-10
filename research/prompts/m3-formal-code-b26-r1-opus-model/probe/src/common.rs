//! 生成并录一段历史。mkfs 与起点那几步是固定前缀（「只固定前缀与故障」）：它们的写施加进基线，不枚举；
//! 起点之后到最后一个跑完的操作为止的写交给层 0 全量枚举（每一段都展开、原地覆写取三态）。
use std::collections::BTreeMap;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GeneratedHistory, GenerationWeights,
    HistoryDeviceWidth, HistoryExecution, HistoryOperationKind, HistorySeed, PerStepChecker,
    StepPosition,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::model::ModelRootKey;
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::SharedStream;

pub fn weights_named(name: &str) -> GenerationWeights {
    GenerationWeights::NAMED
        .iter()
        .find(|weights| weights.environment_value() == name)
        .copied()
        .unwrap_or_else(|| panic!("没有叫 {name} 的比重"))
}

pub fn width_named(name: &str) -> HistoryDeviceWidth {
    HistoryDeviceWidth::ALL
        .into_iter()
        .find(|width| width.environment_value() == name)
        .unwrap_or_else(|| panic!("没有叫 {name} 的盘宽"))
}

pub fn execution_on(width: HistoryDeviceWidth) -> HistoryExecution {
    HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    }
}

/// 在读故障下读盘的两类步（与 `crash_injection.rs` 的 `operation_reads_under_injected_faults` 同一张表）。
pub fn reads_under_injected_faults(kind: HistoryOperationKind) -> bool {
    matches!(
        kind,
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
            | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
    )
}

pub struct Recorded {
    pub history: GeneratedHistory,
    pub ending: String,
    pub parameters: MakeFilesystemParameters,
    pub execution: HistoryExecution,
    pub geometry: FixedGeometry,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    /// 每个写在整条录制流里的下标。
    pub stream_indexes: Vec<usize>,
    pub base: MemoryPool,
    /// 每一步跑完时整条录制流里有几步。
    pub after_each_step: Vec<(StepPosition, usize)>,
    pub versions: Vec<PublishedVersion>,
    /// 读故障下读盘、而且往流里发过东西的步：(步号, 它之前流里有几步)。
    pub faulted_steps: Vec<(usize, usize)>,
}

fn core_root(key: &ModelRootKey) -> (InstanceGeneration, CheckpointTxg) {
    (
        InstanceGeneration(u32::try_from(key.instance.0).expect("实例代号装得进 u32")),
        CheckpointTxg(key.checkpoint_txg.0),
    )
}

/// `prefix_steps` 步之前（含起点）是固定前缀：施加进基线、不枚举；0 时前缀只有 mkfs 与起点。
pub fn record(
    weights: &GenerationWeights,
    width: HistoryDeviceWidth,
    seed: u64,
    steps: usize,
    prefix_steps: usize,
) -> Recorded {
    record_history(generate_history_with_weights(HistorySeed(seed), steps, weights), width, prefix_steps)
}

/// 同 [`record`]，历史由调用方给（写死的步序，用户动作逐种扫）。
pub fn record_history(history: GeneratedHistory, width: HistoryDeviceWidth, prefix_steps: usize) -> Recorded {
    let execution = execution_on(width);
    let stream = SharedStream::retaining_contents();
    let mut committed: BTreeMap<ModelRootKey, Option<Vec<u8>>> = BTreeMap::new();
    let mut after_each_step: Vec<(StepPosition, usize)> = Vec::new();
    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed.insert(key, file.map(|content| content.to_vec()));
        }
        after_each_step.push((observation.position, stream.operation_count()));
    });
    let operations = stream.retained_operations();
    let geometry = width.fixed_geometry();
    let fixed_until = if prefix_steps == 0 || prefix_steps == usize::MAX {
        StepPosition::StartingPoint
    } else {
        StepPosition::Operation(prefix_steps - 1)
    };
    // `usize::MAX`：前缀只有 mkfs，起点那几步也进写表（与层 0 的新池新建文件那条流同一个基线）。
    let start = if prefix_steps == usize::MAX {
        run.operations_written_by_make_filesystem
    } else {
        after_each_step
            .iter()
            .find(|(position, _)| *position == fixed_until)
            .map_or(run.operations_written_by_make_filesystem, |(_, count)| *count)
    };
    let last = after_each_step.last().map_or(start, |(_, count)| *count).max(start);
    let (prefix_writes, _, _) = writes_and_segments_with_stream_indexes(&operations[..start], &geometry);
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], width.device_bytes());
    base.apply_writes(&prefix_writes);
    let (writes, segments, local_indexes) =
        writes_and_segments_with_stream_indexes(&operations[start..last], &geometry);
    let stream_indexes = local_indexes.iter().map(|index| index + start).collect();
    let versions = committed
        .iter()
        .filter_map(|(key, file)| {
            let (instance, checkpoint_txg) = core_root(key);
            file.as_ref().map(|content| PublishedVersion { instance, checkpoint_txg, content: content.clone() })
        })
        .collect();
    let mut faulted_steps = Vec::new();
    let mut before = run.operations_written_by_make_filesystem;
    for (position, after) in &after_each_step {
        if let StepPosition::Operation(step) = position {
            let kind = history.operations[*step].kind();
            if reads_under_injected_faults(kind) && before < *after {
                faulted_steps.push((*step, before));
            }
        }
        before = *after;
    }
    let ending = format!("{:?}", run.ending).chars().take(160).collect();
    Recorded {
        history,
        ending,
        parameters: width.parameters(),
        execution,
        geometry,
        writes,
        segments,
        stream_indexes,
        base,
        after_each_step,
        versions,
        faulted_steps,
    }
}
