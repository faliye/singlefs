//! m2-closeout-code-r3 云端攻方腿 X1（C579 那一支的邻格）：一条单故障历史，可写挂载被拒、每次重试都拒，只读挂载照常。
//! 历史：第一个文件、会话覆盖写一次（实例 1，txg 4），进程退出；可写挂载（实例 2）写行那次发布落盘（根槽 FUA、两盘轮换、屏障）之后、
//! 第一次暖机的根落盘之前掉电（录制流在那一道屏障处截断，是层 0 的一个段边界状态）。之后单故障：写行那条根所在的根环槽坏
//! （改一个字节 ⇒ 自证不过；或那一个扇区读报错）。再可写挂载若干次，看拒不拒、只读恢复读回什么。
//! 对照：同一截断状态不坏任何槽；截在第一次暖机落盘之后、坏最新那条根；会话里发布几次之后坏最新那条根。

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use common_admission::{plain_devices_on, start_plain, OVERWRITE_BYTES};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::slot_offset;
use singlefs_core::root_ring::target_for_publish;
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{root_identity_written_by, MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

/// 读某一块盘某一段字节就报块设备错的盘（坏扇区：一直坏，不止一次）。
struct BadSectorDevice {
    inner: SparseBlockDevice,
    bad_range: Option<std::ops::Range<u64>>,
}

impl BlockDevice for BadSectorDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if let Some(range) = &self.bad_range {
            let end = offset.0 + u64::try_from(buffer.len()).expect("长度");
            if offset.0 < range.end && range.start < end {
                return Err(injected_block_device_error("坏扇区"));
            }
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

fn short(text: String) -> String {
    let cut: String = text.chars().take(260).collect();
    cut.replace('\n', " ")
}

fn read_back(image: &MemoryPool) -> String {
    match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, content } => format!("FileRead root=({},{}) len={}", root.0 .0, root.1 .0, content.len()),
        RecoveryOutcome::NoFile { root } => format!("NoFile root=({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { root, failure } => short(format!("Failed root={root:?} {failure:?}")),
    }
}

/// 可写挂载的录制流里第 `publish_index` 次根槽 FUA（0 = 写行那次）之后的第一串屏障的末尾：截到那里（含）的下标。
fn cut_after_the_publish(operations: &[singlefs_harness::RetainedOperation], geometry: &FixedGeometry, publish_index: usize) -> Option<(usize, usize)> {
    let root_positions: Vec<usize> = operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| geometry.classify(&retained.operation) == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let root_position = *root_positions.get(publish_index)?;
    let first_barrier = (root_position..operations.len()).find(|index| operations[*index].operation.kind == RecordedOperationKind::Barrier)?;
    let mut last_barrier = first_barrier;
    while last_barrier + 1 < operations.len() && operations[last_barrier + 1].operation.kind == RecordedOperationKind::Barrier {
        last_barrier += 1;
    }
    Some((root_position, last_barrier))
}

#[test]
fn a_power_loss_after_the_row_publish_and_one_bad_root_slot_refuse_every_writable_mount() {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
    pool.session = None;
    let base = pool.image();
    let parameters = pool.parameters.clone();
    let bytes = base.device_size_in_bytes;
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&base)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect();
    let mounted = mount_writable(&parameters, &mut recorded).expect("可写挂载（实例 2）");
    println!("MOUNT instance={} chosen=({},{}) warm_ups={}", mounted.output.instance.0, mounted.output.chosen_root.instance.0, mounted.output.chosen_root.checkpoint_txg.0, mounted.output.warm_up_publishes.len());
    let operations = stream.retained_operations();
    for (label, publish_index) in [("after_the_row_publish", 0usize), ("after_the_first_warm_up", 1), ("after_the_last_warm_up", 1 + mounted.output.warm_up_publishes.len() - 1)] {
        let Some((root_position, cut)) = cut_after_the_publish(&operations, &geometry, publish_index) else {
            println!("CASE {label} no_such_publish");
            continue;
        };
        let mut crashed = base.clone();
        crashed.apply(&operations[..=cut]);
        let root_bytes = operations[root_position].contents.clone().expect("根槽写的内容");
        let (instance, checkpoint_txg) = root_identity_written_by(&root_bytes);
        let ring_slot = target_for_publish(checkpoint_txg, parameters.geometry.root_ring_slots_per_region);
        let root_device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        let root_offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing).0;
        println!("CASE {label} cut={cut} newest_root=({},{}) on_device={} offset={root_offset} read={}", instance.0, checkpoint_txg.0, root_device.0, read_back(&crashed));
        // 不坏任何槽：可写挂载应当做成。
        let mut devices = plain_devices_on(&crashed);
        let clean = mount_writable(&parameters, &mut devices).map(|mounted| mounted.output.instance.0).map_err(|error| short(format!("{error:?}")));
        println!("CASE {label} fault=none writable={clean:?}");
        // 单故障一：那条根所在的槽改一个字节（自证不过，一直这样）。
        let mut corrupted = crashed.clone();
        corrupted.flip_byte(root_device, DeviceOffsetInBytes(root_offset), 100);
        for attempt in 0..3 {
            let mut devices = plain_devices_on(&corrupted);
            let outcome = mount_writable(&parameters, &mut devices).map(|mounted| (mounted.output.instance.0, mounted.output.chosen_root.checkpoint_txg.0)).map_err(|error| short(format!("{error:?}")));
            let after = MemoryPool { devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(), device_size_in_bytes: bytes };
            let unchanged = after.devices.iter().all(|(identity, device)| device.written_sectors() == corrupted.devices[identity].written_sectors());
            println!("CASE {label} fault=root_slot_not_self_verified attempt={attempt} writable={outcome:?} disk_unchanged={unchanged} read_only={}", read_back(&corrupted));
        }
        // 单故障二：那条根所在的扇区读报错（一直这样）。
        for attempt in 0..2 {
            let mut devices: Vec<(DeviceIdentity, BadSectorDevice)> = plain_devices_on(&crashed)
                .into_iter()
                .map(|(identity, inner)| (identity, BadSectorDevice { inner, bad_range: (identity == root_device).then_some(root_offset..root_offset + 512) }))
                .collect();
            let outcome = mount_writable(&parameters, &mut devices).map(|mounted| (mounted.output.instance.0, mounted.output.chosen_root.checkpoint_txg.0)).map_err(|error| short(format!("{error:?}")));
            println!("CASE {label} fault=root_sector_unreadable attempt={attempt} writable={outcome:?}");
        }
    }
}

/// 同一形在只做过 mkfs 的池上（C577 结清 ③ 那条用例的历史：第一次可写挂载，写行那次是零单元发布）：写行那次落盘之后掉电
/// （或那次挂载因轮换之后那道屏障报错整个返回错误，盘上同样停在这里），写行那条根所在的槽再坏一个字节。
#[test]
fn the_same_shape_on_a_pool_that_only_went_through_mkfs() {
    let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    let bytes = HistoryDeviceWidth::UnitAreaOf384Slots.device_bytes();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let mut plain: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512))))
        .collect();
    singlefs_core::make_filesystem::make_filesystem(&parameters, &mut plain).expect("mkfs");
    let base = MemoryPool { devices: plain.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(), device_size_in_bytes: bytes };
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&base)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect();
    let mounted = mount_writable(&parameters, &mut recorded).expect("第一次可写挂载");
    println!("FORMATTED instance={} warm_ups={}", mounted.output.instance.0, mounted.output.warm_up_publishes.len());
    let operations = stream.retained_operations();
    let (root_position, cut) = cut_after_the_publish(&operations, &geometry, 0).expect("写行那次");
    let mut crashed = base.clone();
    crashed.apply(&operations[..=cut]);
    let (instance, checkpoint_txg) = root_identity_written_by(operations[root_position].contents.as_ref().expect("根槽写"));
    let ring_slot = target_for_publish(checkpoint_txg, parameters.geometry.root_ring_slots_per_region);
    let root_device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let root_offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing).0;
    let mut devices = plain_devices_on(&crashed);
    println!("FORMATTED newest_root=({},{}) fault=none writable={:?}", instance.0, checkpoint_txg.0, mount_writable(&parameters, &mut devices).map(|mounted| mounted.output.instance.0).map_err(|error| short(format!("{error:?}"))));
    let mut corrupted = crashed.clone();
    corrupted.flip_byte(root_device, DeviceOffsetInBytes(root_offset), 100);
    for attempt in 0..3 {
        let mut devices = plain_devices_on(&corrupted);
        let outcome = mount_writable(&parameters, &mut devices).map(|mounted| mounted.output.instance.0).map_err(|error| short(format!("{error:?}")));
        println!("FORMATTED fault=root_slot_not_self_verified attempt={attempt} writable={outcome:?} read_only={}", read_back(&corrupted));
    }
}
