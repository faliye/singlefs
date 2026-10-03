//! 草稿探针（只在我自己的副本里，harness 档）：量 C670 第三截测试那段历史的切段与「挂载写行那一段」的写布局。
//! 不判任何东西，只打印：候选段数与各段长度、挂载那一次的写按段列出种类、写行段的写数 N。
use std::collections::BTreeMap;

use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

#[test]
fn print_the_layout() {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 0 },
            fill_seed: 1,
        })],
    };
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let stream = SharedStream::retaining_contents();
    let mut after_last = 0usize;
    let run = execute_history_with(&history, execution, &stream, &mut |_observation| {
        after_last = stream.operation_count();
    });
    let parameters = execution.device_width.parameters();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&stream.retained_operations(), &geometry);
    let after_mkfs = run.operations_written_by_make_filesystem;
    let candidates: Vec<usize> = (0..segments.len())
        .filter(|s| {
            segments[*s]
                .iter()
                .all(|w| (after_mkfs..after_last).contains(&stream_indexes[*w]))
        })
        .collect();
    println!("历史写数 {}，段数 {}，候选段数 {}", writes.len(), segments.len(), candidates.len());
    let mut by_length: BTreeMap<usize, usize> = BTreeMap::new();
    for s in &candidates {
        *by_length.entry(segments[*s].len()).or_insert(0) += 1;
    }
    println!("候选段按长度的个数 {by_length:?}");
    for s in &candidates {
        let kinds: Vec<&str> = segments[*s].iter().map(|w| writes[*w].kind.name()).collect();
        println!("  候选段 {s} 长 {}：{kinds:?}", segments[*s].len());
    }
    // 全持久的池上起一次可写挂载，录制挂载那一段
    let mut pool = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    pool.apply_writes(&writes);
    let mount_stream = SharedStream::retaining_contents();
    let mut devices: Vec<_> = pool
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                pool.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, mount_stream.clone()),
            )
        })
        .collect();
    let mounted = mount_writable_with_space_admission(&parameters, &mut devices, execution.space_admission);
    println!("挂载结局 ok={}", mounted.is_ok());
    let (mwrites, msegments, _) =
        writes_and_segments_with_stream_indexes(&mount_stream.retained_operations(), &geometry);
    println!("挂载写数 {}，段数 {}", mwrites.len(), msegments.len());
    for (i, seg) in msegments.iter().enumerate() {
        let kinds: Vec<String> = seg
            .iter()
            .map(|w| format!("{}@dev{}", mwrites[*w].kind.name(), mwrites[*w].device.0))
            .collect();
        println!("  挂载段 {i} 长 {}：{kinds:?}", seg.len());
    }
    // 写行段 = 取号的系统配置写之后到第一个根槽写及其后的系统配置写为止
    let mut phase_writes = 0usize;
    let mut started = false;
    let mut roots = 0;
    for w in &mwrites {
        match w.kind {
            StepKind::SystemConfigurationSlot if !started => continue,
            _ => {}
        }
        started = true;
        if roots >= 1 && w.kind != StepKind::SystemConfigurationSlot {
            break;
        }
        phase_writes += 1;
        if w.kind == StepKind::RootRecordFua {
            roots += 1;
        }
    }
    println!("写行那一段（含它的轮换）的写数 N = {phase_writes}");
}
