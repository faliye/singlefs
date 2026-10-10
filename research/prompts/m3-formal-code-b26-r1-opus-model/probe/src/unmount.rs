//! 正常卸载的流（历史生成器没有卸载这一步）：mkfs → 可写挂载 → 第一个文件 → 覆盖写 `overwrites` 次 → `unmount`（抬 F 到现行版本、
//! 带卸载记号的空发布）。卸载之前是固定前缀；卸载那一段写全量枚举，卸载做完的那一态（「正常卸载后再挂」的盘）是最后一个状态。
use std::collections::BTreeMap;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, unmount, ShadowLedger, Unmounted};
use singlefs_core::transaction::{publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::history::{GeneratedHistory, HistoryDeviceWidth, HistorySeed, HistoryStartingPoint};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, SparseBlockDevice,
};
use singlefs_harness::scenario::FIXED_WRITE_TIME_SECONDS;
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes_and_entries;
use singlefs_harness::{RecordedEntrySpan, RecordedPublishEntry, RecordingBlockDevice, SharedStream};

use crate::common::{execution_on, Recorded};

fn content(round: usize) -> Vec<u8> {
    (0..(100 + round * 37)).map(|index| (index * 7 + round) as u8).collect()
}

pub fn record(width: HistoryDeviceWidth, overwrites: usize) -> Recorded {
    let parameters = width.parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            let device = SparseBlockDevice::new(
                width.device_bytes(),
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()))
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator;
    let mut contents: BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>> = BTreeMap::new();
    let PoolVersion::WithoutFile(without_file) = &mounted.current else {
        panic!("mkfs 之后第一次挂载落在树表 0 条的一版上");
    };
    let first = content(0);
    let mut current = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &without_file.root,
            FirstFile { content: &first, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &without_file.record_bytes,
        )
        .expect("第一个文件")
    };
    contents.insert((current.root.instance, current.root.checkpoint_txg), first);
    let mut last = content(0);
    for round in 1..=overwrites {
        last = content(round);
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile { content: &last, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
        )
        .expect("覆盖写");
        contents.insert((current.root.instance, current.root.checkpoint_txg), last.clone());
    }
    let start = stream.operation_count();
    let mut version = PoolVersion::WithFile(current);
    let unmounted = stream.record_entry(RecordedPublishEntry::Unmount, || {
        unmount(&parameters, &mut devices, &mut allocator, &mut version, ShadowLedger::On)
    });
    match unmounted.expect("卸载") {
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => {
            for publish in &raised.publishes {
                contents.insert((publish.root.instance, publish.root.checkpoint_txg), last.clone());
            }
        }
        Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => panic!("带文件的一版卸载要抬 F"),
    }
    let operations = stream.retained_operations();
    let geometry = width.fixed_geometry();
    let (prefix_writes, _, _) = writes_and_segments_with_stream_indexes(&operations[..start], &geometry);
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], width.device_bytes());
    base.apply_writes(&prefix_writes);
    // 卸载入口那一段的下标挪到从 `start` 数（切段之前核 C557：带卸载记号的根槽写只许在卸载入口里）。
    let spans: Vec<RecordedEntrySpan> = stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - start..span.operations.end - start,
        })
        .collect();
    let (writes, segments, local_indexes) =
        writes_and_segments_with_stream_indexes_and_entries(&operations[start..], &spans, &geometry);
    Recorded {
        history: GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterMakeFilesystem,
            operations: Vec::new(),
        },
        ending: format!("unmount after {overwrites} overwrites"),
        parameters,
        execution: execution_on(width),
        geometry,
        writes,
        segments,
        stream_indexes: local_indexes.iter().map(|index| index + start).collect(),
        base,
        after_each_step: Vec::new(),
        versions: contents
            .into_iter()
            .map(|((instance, checkpoint_txg), content)| PublishedVersion { instance, checkpoint_txg, content })
            .collect(),
        faulted_steps: Vec::new(),
    }
}
