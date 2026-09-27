//! 代码轮第一轮云端攻方腿（m2-closeout-code-r1）Z5 那一格：一条真写出来的多挂载流（mkfs、第一个文件、覆盖写、崩了再挂、
//! 准入抬 F、管理员回退、正常卸载）上，
//! 1. 取第三态的写（`TearableInPlaceOverwrites::of`）各是哪一类；每一次在新旧不同那一截里逐字节换撕裂点（前新后旧、前旧后新两个方向），
//!    撕出来的那一份能不能被读者当成一份自证过的系统配置 / journal 记录（与新、旧都不同的算「别的撕裂点给出别的内容」）；
//! 2. 按设备切段时 FUA 放行它那块盘：流里有没有「FUA 之前、同一块盘上最后一道屏障之后还有普通写」的形（有的话，段模型把那几次
//!    普通写算成随 FUA 一起放行，真设备上 FUA 不替它们做持久）。
//! 不跑层 0 枚举，只看写表与字节。
mod common;
mod common_admission;

use std::collections::BTreeMap;

use common_admission::{PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::journal::JournalRecord;
use singlefs_core::mount::{
    raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish, unmount,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::PoolReader;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::transaction::PoolVersion;
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_harness::crash::{
    torn_image_of_in_place_overwrite, writes_and_segments_with_stream_indexes_and_entries,
    MemoryPool, SparseBlockDevice, TearableInPlaceOverwrites, WrittenContents,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedOperationKind, RecordedPublishEntry, RecordingBlockDevice, SharedStream};

type Recorded = RecordingBlockDevice<SparseBlockDevice>;

fn recorded_history(stream: &SharedStream) -> PoolUnderTest<Recorded> {
    let wrap = |identity: DeviceIdentity, device: SparseBlockDevice| {
        RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
    };
    let mut pool = PoolUnderTest::start_after_the_first_file(HistoryDeviceWidth::FourGibibytes, wrap);
    for _ in 0..4 {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
    }
    pool.crash_and_mount_writable().expect("崩了再挂");
    for _ in 0..5 {
        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
    }
    {
        let session = pool.session.as_mut().expect("会话");
        let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
        let raised = raise_rollback_floor_to_the_admission_ceiling(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            ShadowLedger::On,
        );
        println!("Z5 raise_to_ceiling ok={}", raised.is_ok());
    }
    {
        let session = pool.session.as_mut().expect("会话");
        let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
        let target_txg = CheckpointTxg(current.root.checkpoint_txg.0 - 2);
        let target = RollbackTarget { instance: current.root.instance, checkpoint_txg: target_txg };
        let rolled = roll_back_by_a_forward_publish(&pool.parameters, &mut pool.devices, &mut session.allocator, current, target);
        println!("Z5 rollback ok={} err={:?}", rolled.is_ok(), rolled.as_ref().err());
    }
    pool.overwrite(OVERWRITE_BYTES).expect("回退之后覆盖写");
    {
        let session = pool.session.as_mut().expect("会话");
        let parameters = pool.parameters.clone();
        let devices = &mut pool.devices;
        let unmounted = stream.record_entry(RecordedPublishEntry::Unmount, || {
            unmount(&parameters, devices, &mut session.allocator, &mut session.current, ShadowLedger::On)
        });
        println!("Z5 unmount ok={}", unmounted.is_ok());
    }
    pool.session = None;
    pool.crash_and_mount_writable().expect("卸载之后再挂");
    pool.overwrite(OVERWRITE_BYTES).expect("再挂之后覆盖写");
    pool
}

#[test]
fn torn_split_points_and_fua_release_on_a_recorded_multi_mount_stream() {
    let stream = SharedStream::retaining_contents();
    let pool = recorded_history(&stream);
    let operations = stream.retained_operations();
    let geometry = HistoryDeviceWidth::FourGibibytes.fixed_geometry();
    let (writes, segments, _) =
        writes_and_segments_with_stream_indexes_and_entries(&operations, &stream.entry_spans(), &geometry);
    let base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], pool.device_bytes);
    let tearable = TearableInPlaceOverwrites::of(&base, &writes);
    let mut by_kind: BTreeMap<&'static str, usize> = BTreeMap::new();
    for index in tearable.write_indexes() {
        *by_kind.entry(writes[index].kind.name()).or_insert(0) += 1;
    }
    println!("Z5 stream operations={} writes={} segments={} tearable_by_kind={by_kind:?}", operations.len(), writes.len(), segments.len());

    // 1. 撕裂点：逐字节换，两个方向。
    let fsid = pool.parameters.filesystem_identifier;
    let unit_fsid = unit_filesystem_identifier(&fsid);
    let mut image = base.clone();
    let mut next = 0usize;
    let mut tried = 0u64;
    let mut self_verifying_other_than_old_and_new = 0u64;
    let mut midpoint_self_verifying = 0u64;
    for index in tearable.write_indexes() {
        image.apply_writes(&writes[next..index]);
        next = index;
        let write = &writes[index];
        let length = usize::try_from(write.length_in_bytes()).expect("写长");
        let old = PoolReader::read(&image, write.device, write.offset, length).expect("读旧字节");
        let WrittenContents::Bytes(new) = &write.contents else { continue };
        let differs: Vec<usize> = (0..length).filter(|at| old[*at] != new[*at]).collect();
        let (Some(first), Some(last)) = (differs.first().copied(), differs.last().copied()) else { continue };
        let verifies = |bytes: &[u8]| match write.kind {
            StepKind::SystemConfigurationSlot => SystemConfiguration::parse_slot(bytes)
                .is_ok_and(|parsed| parsed.immutable.filesystem_identifier == fsid),
            StepKind::JournalRecord => JournalRecord::parse(bytes, unit_fsid).is_some(),
            _ => false,
        };
        let midpoint = torn_image_of_in_place_overwrite(&old, &write.contents);
        if verifies(&midpoint) && midpoint != old && midpoint != *new {
            midpoint_self_verifying += 1;
        }
        for split in first + 1..=last {
            for new_first in [true, false] {
                let mut torn = old.clone();
                if new_first {
                    torn[..split].copy_from_slice(&new[..split]);
                } else {
                    torn[split..].copy_from_slice(&new[split..]);
                }
                tried += 1;
                if torn != old && torn != *new && verifies(&torn) {
                    self_verifying_other_than_old_and_new += 1;
                    println!("Z5 split_point_self_verifies write={index} kind={} split={split} new_first={new_first}", write.kind.name());
                }
            }
        }
        println!(
            "Z5 tearable write={index} kind={} device={} offset={} length={length} differing_span={}..={} differing_bytes={}",
            write.kind.name(), write.device.0, write.offset.0, first, last, differs.len()
        );
    }
    println!("Z5 split_points_tried={tried} self_verifying_other_than_old_and_new={self_verifying_other_than_old_and_new} midpoint_self_verifying={midpoint_self_verifying}");

    // 2. FUA 之前同一块盘上最后一道屏障之后的普通写。
    let mut plain_since_barrier: BTreeMap<DeviceIdentity, usize> = BTreeMap::new();
    let mut fua_with_unflushed_plain_writes_on_its_device = 0usize;
    let mut fua_total = 0usize;
    for retained in &operations {
        let operation = &retained.operation;
        match operation.kind {
            RecordedOperationKind::Barrier => {
                plain_since_barrier.insert(operation.device, 0);
            }
            RecordedOperationKind::Write | RecordedOperationKind::WriteZeroes => {
                *plain_since_barrier.entry(operation.device).or_insert(0) += 1;
            }
            RecordedOperationKind::WriteForceUnitAccess => {
                fua_total += 1;
                if plain_since_barrier.get(&operation.device).copied().unwrap_or(0) > 0 {
                    fua_with_unflushed_plain_writes_on_its_device += 1;
                }
            }
        }
    }
    println!("Z5 fua_total={fua_total} fua_with_unflushed_plain_writes_on_its_device={fua_with_unflushed_plain_writes_on_its_device}");
}
