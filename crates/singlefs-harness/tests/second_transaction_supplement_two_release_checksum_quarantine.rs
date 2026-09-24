//! 里程碑「第二个事务」增补 2 收口表第 13 行（C394（释放判定不核映射条目位置项里的单元校验和））：
//! D19（块指针的结构与宽度预算） 已定项 5 硬规则 1——释放一律经映射，经映射核到那条映射条目之后，释放之前还要按它位置项里带的
//! 单元校验和读盘核一次；核出对不上时隔离那个槽、发布照成：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出。
//!
//! 四条用例：
//! 1. 两盘上那一份都被改坏：释放照成、两盘都隔离、那个槽不回空闲池（复用窗口置 0，别的槽释放那一刻就回到空闲池，只有它回不去）；
//!    判别力自证：把那一核拿掉，那个槽回到空闲池、下一次用户数据落点就是它（`crates/mutations.tsv`）。
//! 2. 只有一块盘上那一份被改坏：只隔离那一块盘上的那一份，另一块盘照常释放。
//! 3. 从盘上重建上一版（进程重开、可写挂载）之后再覆盖写：C394 登记的那条路径，核照样生效。
//! 4. 读盘本身失败：条款没定归哪个错误成员（C394 前置那三件之一）⇒ 在任何落盘动作之前返回点名「条款没定」的成员，盘上逐字节不变。

mod common;

use std::cell::Cell;

use common::{
    build_pool, disk_snapshot, parameters, publish_overwrite_in_process, BuiltPool, Recorded,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
    SlotNumber,
};
use singlefs_core::allocator::{Placement, ReuseWindow};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{
    mapping_locations_for_key, publish_overwrite, CopyQuarantinedAfterReleaseChecksumMismatch,
    FirstFile, MappingLookup, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_harness::fault_injection::injected_block_device_error;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 11 + seed) % 247).expect("小于 256"))
        .collect()
}

/// 改坏一块盘上某个槽起的那一份单元：翻它第 4096 字节起那个扇区的第一个字节（数据单元与索引节点都罩得住这一处），
/// 位置项里的校验和从此对不上。写经录制器落盘，与别的写一样进录制流。
fn corrupt_the_copy_on(
    devices: &mut [(DeviceIdentity, Recorded)],
    device: DeviceIdentity,
    slot: SlotNumber,
) {
    let (_, block_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    let sector_offset = DeviceOffsetInBytes(slot.to_device_offset().0 + 4096);
    let mut sector = vec![0u8; 512];
    block_device
        .read_at(sector_offset, &mut sector)
        .expect("读那一个扇区");
    sector[0] ^= 0xff;
    block_device
        .write_at(sector_offset, &sector, WriteDurability::Plain)
        .expect("写回改坏的扇区");
}

/// 上一版数据单元那把映射 key。
fn data_mapping_key(version: &TransactionOutput) -> Vec<u8> {
    version
        .mapped_units
        .iter()
        .find(|(unit, _)| *unit == TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .expect("码 1 一把 key")
        .1
        .clone()
}

/// 两块盘各自那条分配记录（槽 `slot`）：(是不是已释放, 分配代或释放代)。
fn records_of_slot(
    pool: &BuiltPool,
    slot: SlotNumber,
) -> Vec<(DeviceIdentity, bool, CheckpointTxg)> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| {
            let record = pool
                .allocator
                .record_for(device, slot)
                .expect("释放只改写记录、不删");
            (device, record.is_released, record.generation)
        })
        .collect()
}

fn quarantined_slots_per_device(pool: &BuiltPool) -> Vec<(DeviceIdentity, u64)> {
    pool.allocator
        .devices
        .iter()
        .map(|device_map| {
            (
                device_map.device,
                device_map.quarantined_after_release_checksum_mismatch_slots(),
            )
        })
        .collect()
}

fn is_free_on_each_device(pool: &BuiltPool, slot: SlotNumber) -> Vec<bool> {
    pool.allocator
        .devices
        .iter()
        .map(|device_map| device_map.is_free(slot))
        .collect()
}

/// 用例 1：A（txg 3）的数据单元两盘那一份都被改坏，复用窗口置 0 再覆盖写（txg 4）。
/// 复用窗口置 0（只供测试的开关）让被换下的落点在 `release` 返回时就回到空闲池，「那个槽回没回空闲池」在这一次发布里就看得见：
/// 同一次释放的 A 的 extent 根那一槽当场回到空闲池，只有核出对不上的数据单元那一对槽回不去。
#[test]
fn a_released_unit_whose_copies_fail_the_checksum_in_its_mapping_entry_is_quarantined_instead_of_returning_to_the_free_pool(
) {
    let mut pool = build_pool("release-checksum-both-copies");
    let first = pool.output.clone();
    let data_slot = first.data_pointers[0].locations[0].slot;
    let extent_root_slot = first.unit(TransactionUnit::ExtentRoot).slot;
    assert_eq!(
        data_slot,
        SlotNumber(50180),
        "A 的数据单元落在 50180–50181（字节表五）"
    );
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot);
        corrupt_the_copy_on(devices, DeviceIdentity(1), data_slot);
    }
    pool.allocator.set_reuse_window(ReuseWindow::ForcedToZero);

    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(2800, 5),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("核出对不上时发布照成");
    assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));

    // 逻辑上照样释放：这次释放的落点里有它、两盘的分配记录都改写成已释放 + 释放代 4、B 的映射里已经没有 A 的数据单元那一条。
    let data_placement = Placement {
        slot: data_slot,
        span: 2,
    };
    assert!(
        second.released.contains(&data_placement),
        "A 的数据单元照样释放：{:?}",
        second.released
    );
    assert_eq!(
        records_of_slot(&pool, data_slot),
        vec![
            (DeviceIdentity(0), true, CheckpointTxg(4)),
            (DeviceIdentity(1), true, CheckpointTxg(4)),
        ],
        "两盘的分配记录改写成已释放、释放代 4"
    );
    assert_eq!(
        mapping_locations_for_key(
            &second.unit(TransactionUnit::MappingTree).bytes,
            &data_mapping_key(&first)
        ),
        MappingLookup::NodeMalformedOrNoEntryWithThisKey,
        "映射条目去掉：B 的映射里查不到 A 的数据单元"
    );

    // 记进隔离并计数报出：这次发布交回的隔离清单、分配器每块盘的计数。
    assert_eq!(
        second.quarantined_after_release_checksum_mismatch,
        vec![
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
                device: DeviceIdentity(0),
                placement: data_placement,
            },
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
                device: DeviceIdentity(1),
                placement: data_placement,
            },
        ],
        "只有被改坏的那两份进隔离"
    );
    assert_eq!(
        quarantined_slots_per_device(&pool),
        vec![(DeviceIdentity(0), 2), (DeviceIdentity(1), 2)],
        "每块盘隔离两槽（数据单元跨两槽）"
    );

    // 物理槽不还回空闲池：同一次释放、没被改坏的 extent 根那一槽当场回到空闲池（开关走到了），数据单元那一对槽回不去。
    assert_eq!(
        is_free_on_each_device(&pool, extent_root_slot),
        vec![true, true],
        "对照：没被改坏的 A 的 extent 根那一槽在复用窗口置 0 下释放那一刻就回到空闲池"
    );
    for slot in [data_slot, SlotNumber(data_slot.0 + 1)] {
        assert_eq!(
            is_free_on_each_device(&pool, slot),
            vec![false, false],
            "隔离的槽 {slot:?} 不回空闲池"
        );
    }
    assert_eq!(
        second.data_pointers[0].locations[0].slot,
        SlotNumber(50178),
        "B 的数据单元落在 mkfs 树表回收空的那一对槽上，没落回 50180"
    );
    let next_user_data = pool
        .allocator
        .allocate_user_data(CheckpointTxg(5))
        .expect("单元区里有的是空槽");
    assert_eq!(
        next_user_data.slot,
        SlotNumber(50182),
        "下一个用户数据落点跳过隔离的 50180，取 50182；那一核拿掉时这里是 50180"
    );
}

/// 用例 2：只改坏盘 0 那一份。硬规则 1 按位置项核，隔离的是核出对不上的那一份所在的槽：盘 0 隔离两槽，盘 1 那一份照常释放、
/// 复用窗口置 0 下当场回到盘 1 的空闲池。
#[test]
fn only_the_copy_that_fails_the_checksum_is_quarantined_and_the_matching_copy_is_released_as_usual()
{
    let mut pool = build_pool("release-checksum-one-copy");
    let first = pool.output.clone();
    let data_slot = first.data_pointers[0].locations[0].slot;
    corrupt_the_copy_on(
        pool.devices.as_mut().expect("镜像还开着"),
        DeviceIdentity(0),
        data_slot,
    );
    pool.allocator.set_reuse_window(ReuseWindow::ForcedToZero);

    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(2700, 9),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("核出对不上时发布照成");

    assert_eq!(
        second.quarantined_after_release_checksum_mismatch,
        vec![CopyQuarantinedAfterReleaseChecksumMismatch {
            unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
            device: DeviceIdentity(0),
            placement: Placement {
                slot: data_slot,
                span: 2,
            },
        }],
        "只有盘 0 那一份进隔离"
    );
    assert_eq!(
        quarantined_slots_per_device(&pool),
        vec![(DeviceIdentity(0), 2), (DeviceIdentity(1), 0)],
        "盘 0 隔离两槽、盘 1 一槽都不隔离"
    );
    assert_eq!(
        records_of_slot(&pool, data_slot),
        vec![
            (DeviceIdentity(0), true, CheckpointTxg(4)),
            (DeviceIdentity(1), true, CheckpointTxg(4)),
        ],
        "两盘都照样释放"
    );
    assert_eq!(
        is_free_on_each_device(&pool, data_slot),
        vec![false, true],
        "盘 0 那一份不回空闲池，盘 1 那一份当场回到空闲池"
    );
}

/// 用例 3：C394 登记的那条路径——从盘上重建上一版（进程重开、可写挂载的 `rebuild_version`）之后再覆盖写。
/// 重开之前改坏盘 0 那一份 A 的数据单元：重建按指针读，盘 0 那一份校验和不过就读盘 1 那一份，挂得上；
/// 之后那次覆盖写经映射核到 A 的数据单元，按映射条目的位置项读盘核出盘 0 那一份对不上 ⇒ 隔离盘 0 那一份，发布照成。
#[test]
fn after_rebuilding_the_previous_version_from_disk_the_release_still_reads_and_quarantines_the_mismatching_copy(
) {
    let mut pool = build_pool("release-checksum-rebuilt");
    let data_slot = pool.output.data_pointers[0].locations[0].slot;
    let mut devices = pool.reopen_recorded();
    corrupt_the_copy_on(&mut devices, DeviceIdentity(0), data_slot);
    let mounted = mount_writable(&parameters(), &mut devices)
        .expect("可写挂载：盘 1 那一份还对得上，重建读得出上一版");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    let instance = mounted.output.instance;
    let rebuilt = mounted
        .current
        .into_file_version()
        .expect("A 之后重开，现行那一版带文件");
    assert_eq!(
        rebuilt.data_pointers[0].locations[0].slot, data_slot,
        "挂载那几次发布都不碰数据单元：现行那一版的数据单元仍是 A 的"
    );
    assert_eq!(
        quarantined_slots_per_device(&pool),
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "挂载那几次发布换下的单元都没被改坏，一槽都没隔离"
    );

    let overwritten = publish_overwrite_in_process(
        &mut pool,
        &rebuilt,
        &content_of(2600, 13),
        FIXED_WRITE_TIME_SECONDS + 120,
        instance,
    )
    .expect("核出对不上时发布照成");
    assert_eq!(
        overwritten.quarantined_after_release_checksum_mismatch,
        vec![CopyQuarantinedAfterReleaseChecksumMismatch {
            unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
            device: DeviceIdentity(0),
            placement: Placement {
                slot: data_slot,
                span: 2,
            },
        }],
        "从盘上重建的上一版经映射释放时照样读盘核：盘 0 那一份进隔离"
    );
    assert_eq!(
        quarantined_slots_per_device(&pool),
        vec![(DeviceIdentity(0), 2), (DeviceIdentity(1), 0)]
    );
    assert_eq!(
        records_of_slot(&pool, data_slot),
        vec![
            (DeviceIdentity(0), true, overwritten.root.checkpoint_txg),
            (DeviceIdentity(1), true, overwritten.root.checkpoint_txg),
        ],
        "逻辑上照样释放"
    );
}

/// 让一块盘上某一个偏移的读返回块设备错，其余照转：造「释放之前读盘核那一读读不到」这一格。
struct ReadsAtOneOffsetFail<Inner: BlockDevice> {
    inner: Inner,
    failing_offset: Option<DeviceOffsetInBytes>,
    refused_reads: Cell<u64>,
}

impl<Inner: BlockDevice> BlockDevice for ReadsAtOneOffsetFail<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        if self.failing_offset == Some(offset) {
            self.refused_reads.set(self.refused_reads.get() + 1);
            return Err(injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
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

/// 用例 4：盘 0 上 A 的数据单元那一份读不到（读盘报错）。读盘本身失败归哪个错误成员、之后怎么办，条款没有写
/// （C394（释放判定不核映射条目位置项里的单元校验和） 前置那三件之一）⇒ 返回点名它的 `ReleaseChecksumReadFailedWhoseHandlingIsUndecided`，
/// 在任何落盘动作之前：盘上逐字节不变（两盘四个系统配置槽、根环里全部自证过的根、录制流步数），分配器一条记录都没改写、一槽都没隔离。
/// 只钉「返回这个成员、盘上不变」，不钉它之后的行为。
#[test]
fn a_release_checksum_read_that_fails_returns_the_undecided_clause_member_before_anything_is_written(
) {
    let mut pool = build_pool("release-checksum-read-fails");
    let first = pool.output.clone();
    let data_slot = first.data_pointers[0].locations[0].slot;
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let records_before = pool.allocator.records().to_vec();

    let mut wrapped: Vec<(DeviceIdentity, ReadsAtOneOffsetFail<Recorded>)> = pool
        .devices
        .take()
        .expect("镜像还开着")
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                ReadsAtOneOffsetFail {
                    inner: device,
                    failing_offset: (identity == DeviceIdentity(0))
                        .then_some(data_slot.to_device_offset()),
                    refused_reads: Cell::new(0),
                },
            )
        })
        .collect();
    let content = content_of(2500, 17);
    let result = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, wrapped.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &first,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
    };
    let refused_reads: Vec<u64> = wrapped
        .iter()
        .map(|(_, device)| device.refused_reads.get())
        .collect();
    pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );

    assert!(
        matches!(
            result,
            Err(PublishError::ReleaseChecksumReadFailedWhoseHandlingIsUndecided {
                unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
                device: DeviceIdentity(0),
                slot,
            }) if slot == data_slot
        ),
        "读盘核那一读读不到 ⇒ 点名「读失败怎么办条款没定」的成员：{result:?}"
    );
    assert_eq!(refused_reads, vec![1, 0], "盘 0 那一读报错一次，就停在那里");
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "在任何落盘动作之前返回：系统配置槽、根环、录制流步数逐项不变"
    );
    assert_eq!(
        pool.allocator.records(),
        records_before.as_slice(),
        "分配器一条记录都没改写"
    );
    assert_eq!(
        quarantined_slots_per_device(&pool),
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "一槽都没隔离"
    );
}
