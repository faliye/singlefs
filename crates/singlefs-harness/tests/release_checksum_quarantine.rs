//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 13 行（C394（释放判定不核映射条目位置项里的单元校验和））：
//! D19（块指针的结构与宽度预算） 已定项 5 硬规则 1——释放一律经映射，经映射核到那条映射条目之后，释放之前还要按它位置项里带的
//! 单元校验和读盘核一次；核出对不上时隔离那一份、发布照成：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池。
//! 射程里没有条款的两问（判决 `research/prompts/m2-newq-r1-main-verification.md` 第三节 N1、N3）：
//! **读盘本身失败先重读一次，还读不出就按对不上处置**（读得出而核出对不上同样先重读一次）；**核出对不上的那一份，在它那块盘上的分配记录留在「已分配」、不改成已释放**——
//! 落盘即跨重挂，准入里照已分配算、不另进式子。
//!
//! 用例：
//! 1. 两盘上那一份都被改坏：映射条目去掉、两盘那条分配记录都留在已分配（分配代照旧），那个槽不回空闲池；
//! 2. 只有一块盘上那一份被改坏：两块盘那一份的分配记录一起留在已分配（账保持对称），覆盖写照成、冷启动走读通过、
//!    下一次覆盖写不被拒；
//! 3. 从盘上重建上一版（进程重开、可写挂载）之后再覆盖写：C394 登记的那条路径，核照样生效；
//! 4. 读盘失败一次、重读读得出：照常释放，一份都不隔离；
//! 5. 两盘都一直读不出：各重读恰好一次，之后按对不上处置；
//! 6. 隔离之后重挂：那一槽的记录从盘上读回来仍是已分配，能回收的都回收了它也不回空闲池、不被再发出去；
//! 7. 准入照已分配算：与不改坏的同一段历史比，两块盘的已分配各多那一份的两槽、可用各少两槽，被抛弃根独占量那一项是 0；
//! 8. 位置项指的盘不在池里、9. 两条位置项指同一块盘：当映射条目损坏，在任何落盘动作之前拒绝，盘上不变；
//! 10. 池级 checker 的 I-3.1 / I-3.11 怎么认隔离的记录：已分配而没有根引用的记录，checker 自己读它罩住的那一份
//!     也读不出或自证不过才豁免，读得出且对得上的照旧判违例。
//! 12. 读得出而核出对不上一次（瞬时坏读）、重读对得上：照常释放，一份都不隔离（对不上也先重读一次）；
//! 13. 每一读都核出对不上：重读恰好一次，之后按对不上处置。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/release_checksum_quarantine.rs"]
mod scenarios_of_release_checksum_quarantine;

use common::{
    build_pool, disk_snapshot, parameters, publish_overwrite_in_process, BuiltPool, Recorded,
    FIXED_WRITE_TIME_SECONDS,
};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_release_checksum_quarantine::{
    content_of, corrupt_the_copy_on, data_slot_of_the_first_version,
};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
    SlotNumber,
};
use singlefs_core::admission::{
    AdmissionReading, BytesOnOneDevice, BytesSummedOverAllReplicas, PoolWideCommitments,
};
use singlefs_core::allocator::{Placement, ReclaimedReuse, ReuseWindow};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::records::{build_mapping_entry, parse_mapping_entry};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration, recover,
    JournalPolicy, RecoveryOutcome,
};
use singlefs_core::transaction::{
    mapping_locations_for_key, publish_overwrite, CopyQuarantinedAfterReleaseChecksumMismatch,
    FirstFile, MappingLookup, PoolWriter, PublishError, QuarantinedCopyReading, TransactionOutput,
    TransactionUnit,
};
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::fault_injection::injected_block_device_error;
use std::cell::Cell;

const DATA: TransactionUnit = TransactionUnit::Data(DataUnitIndexInFile::FIRST);

/// 上一版数据单元那把映射 key。
fn data_mapping_key(version: &TransactionOutput) -> Vec<u8> {
    version
        .mapped_units
        .iter()
        .find(|(unit, _)| *unit == DATA)
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

fn is_free_on_each_device(pool: &BuiltPool, slot: SlotNumber) -> Vec<bool> {
    pool.allocator
        .devices
        .iter()
        .map(|device_map| device_map.is_free(slot))
        .collect()
}

/// 用例 1：A（txg 3）的数据单元两盘那一份都被改坏，复用窗口置 0 再覆盖写（txg 4）。
/// 复用窗口置 0（只供测试的开关）让被换下的落点在 `release` 返回时就回到空闲池，「那个槽回没回空闲池」在这一次发布里就看得见：
/// 同一次释放的 A 的 extent 根那一槽当场回到空闲池，只有核出对不上的数据单元那一对槽回不去——它两盘的分配记录都留在已分配。
#[test]
fn released_unit_whose_copies_fail_the_checksum_in_its_mapping_entry_is_quarantined_instead_of_returning_to_the_free_pool(
) {
    let mut pool = build_pool("release-checksum-both-copies");
    let first = pool.output.clone();
    let data_slot = data_slot_of_the_first_version(&pool);
    let extent_root_slot = first.unit(TransactionUnit::ExtentRoot).slot;
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

    // 逻辑上照样释放：这次释放的落点里有它、B 的映射里已经没有 A 的数据单元那一条。
    let data_placement = Placement {
        slot: data_slot,
        span: 2,
    };
    assert!(
        second.released.contains(&data_placement),
        "A 的数据单元逻辑上照样释放：{:?}",
        second.released
    );
    assert_eq!(
        mapping_locations_for_key(
            &second.unit(TransactionUnit::MappingTree).bytes,
            &data_mapping_key(&first)
        ),
        MappingLookup::NodeMalformedOrNoEntryWithThisKey,
        "映射条目去掉：B 的映射里查不到 A 的数据单元"
    );
    // 隔离 = 那一份在它那块盘上的分配记录留在已分配：两盘都没改写成已释放，分配代仍是 A 的 txg 3。
    assert_eq!(
        records_of_slot(&pool, data_slot),
        vec![
            (DeviceIdentity(0), false, CheckpointTxg(3)),
            (DeviceIdentity(1), false, CheckpointTxg(3)),
        ],
        "两盘的分配记录都留在已分配、分配代照旧"
    );
    assert!(
        second
            .allocation_records
            .iter()
            .any(|record| record.slot == data_slot
                && !record.is_released
                && record.generation == CheckpointTxg(3)),
        "B 写进分配记录树的那一份账里它仍是已分配：落盘即跨重挂"
    );
    assert_eq!(
        second.quarantined_after_release_checksum_mismatch,
        vec![
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: DATA,
                device: DeviceIdentity(0),
                placement: data_placement,
                reading: QuarantinedCopyReading::ChecksumMismatch,
            },
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: DATA,
                device: DeviceIdentity(1),
                placement: data_placement,
                reading: QuarantinedCopyReading::ChecksumMismatch,
            },
        ],
        "只有被改坏的那两份进隔离"
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

/// 用例 2：只改坏盘 0 那一份（用户 2026-09-25 定，D19（块指针的结构与宽度预算） 已定项 5「硬规则 1 的读盘核读不出、核出对不上时怎么办」：
/// 任一份核出对不上，这个单元在每块盘上的分配记录都留在「已分配」，另一块盘上那一份对得上也一起留，各盘的账保持对称）。
/// 覆盖写照成；两盘那条记录都留在已分配、分配代照旧；隔离交回两份——盘 0 那一份读出来对不上、盘 1 那一份对得上而一起留；
/// 冷启动走读通过（每块盘的记录集合相同，`recovery::allocation_records_are_one_per_device`）、读回 B；
/// 下一次覆盖写不被拒（用户数据落点各盘一致）。改之前在任何写之前返回「不对称的账第一版不支持」的成员，这个文件覆盖写不成；
/// 只留坏的那一份时，冷启动走读判整池失败、下一次覆盖写被「各盘落点不一致」拒掉。
#[test]
fn checksum_failure_on_only_one_copy_keeps_both_records_allocated_and_the_pool_stays_writable_and_readable(
) {
    let mut pool = build_pool("release-checksum-one-copy");
    let first = pool.output.clone();
    let data_slot = data_slot_of_the_first_version(&pool);
    corrupt_the_copy_on(
        pool.devices.as_mut().expect("镜像还开着"),
        DeviceIdentity(0),
        data_slot,
    );
    let second_content = content_of(2700, 9);
    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &second_content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("只一份对不上时覆盖写照成");
    let data_placement = Placement {
        slot: data_slot,
        span: 2,
    };
    assert_eq!(
        second.quarantined_after_release_checksum_mismatch,
        vec![
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: DATA,
                device: DeviceIdentity(0),
                placement: data_placement,
                reading: QuarantinedCopyReading::ChecksumMismatch,
            },
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: DATA,
                device: DeviceIdentity(1),
                placement: data_placement,
                reading: QuarantinedCopyReading::IntactButAnotherCopyFailed,
            },
        ],
        "盘 0 那一份对不上，盘 1 那一份对得上也一起隔离"
    );
    assert_eq!(
        records_of_slot(&pool, data_slot),
        vec![
            (DeviceIdentity(0), false, CheckpointTxg(3)),
            (DeviceIdentity(1), false, CheckpointTxg(3)),
        ],
        "两盘的分配记录都留在已分配、分配代照旧：账对称"
    );

    let after_the_quarantine = recover(&pool.memory_pool(), JournalPolicy::Consult);
    assert!(
        matches!(
            &after_the_quarantine.outcome,
            RecoveryOutcome::FileRead { root, content }
                if *root == (InstanceGeneration(1), CheckpointTxg(4)) && *content == second_content
        ),
        "冷启动走读通过、读回 B：{:?}",
        after_the_quarantine.outcome
    );

    let third_content = content_of(2600, 21);
    let third = publish_overwrite_in_process(
        &mut pool,
        &second,
        &third_content,
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("下一次覆盖写不被拒");
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(5));
    let after_the_next_overwrite = recover(&pool.memory_pool(), JournalPolicy::Consult);
    assert!(
        matches!(
            &after_the_next_overwrite.outcome,
            RecoveryOutcome::FileRead { root, content }
                if *root == (InstanceGeneration(1), CheckpointTxg(5)) && *content == third_content
        ),
        "再冷启动读回 C：{:?}",
        after_the_next_overwrite.outcome
    );
}

/// 用例 3：C394 登记的那条路径——从盘上重建上一版（进程重开、可写挂载的 `rebuild_version`）之后再覆盖写。
/// 挂载之后、覆盖写之前把 A 的数据单元两盘那一份都改坏：覆盖写经映射核到它、按映射条目的位置项读盘核出两份都对不上 ⇒
/// 两盘那条记录都留在已分配，发布照成。释放时的读盘核拿上一版映射节点里的位置项重新读盘，不靠重建留下什么。
#[test]
fn after_rebuilding_the_previous_version_from_disk_the_release_still_reads_and_quarantines_the_mismatching_copy(
) {
    let mut pool = build_pool("release-checksum-rebuilt");
    let data_slot = data_slot_of_the_first_version(&pool);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    corrupt_the_copy_on(&mut devices, DeviceIdentity(0), data_slot);
    corrupt_the_copy_on(&mut devices, DeviceIdentity(1), data_slot);
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

    let overwritten = publish_overwrite_in_process(
        &mut pool,
        &rebuilt,
        &content_of(2600, 13),
        FIXED_WRITE_TIME_SECONDS + 120,
        instance,
    )
    .expect("核出对不上时发布照成");
    let data_placement = Placement {
        slot: data_slot,
        span: 2,
    };
    assert_eq!(
        overwritten.quarantined_after_release_checksum_mismatch,
        vec![
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: DATA,
                device: DeviceIdentity(0),
                placement: data_placement,
                reading: QuarantinedCopyReading::ChecksumMismatch,
            },
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: DATA,
                device: DeviceIdentity(1),
                placement: data_placement,
                reading: QuarantinedCopyReading::ChecksumMismatch,
            },
        ],
        "从盘上重建的上一版经映射释放时照样读盘核：两份都进隔离"
    );
    assert_eq!(
        records_of_slot(&pool, data_slot),
        vec![
            (DeviceIdentity(0), false, CheckpointTxg(3)),
            (DeviceIdentity(1), false, CheckpointTxg(3)),
        ],
        "两盘那条都留在已分配"
    );
}

/// 那一个偏移上被拦下的读怎么坏：返回块设备错（读不出），或读得出、交回的字节里翻了一个（读出来核出对不上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FailedReadForm {
    BlockDeviceError,
    CorruptedBytes,
}

/// 让一块盘上某一个偏移的读坏掉（`form` 说怎么坏），其余照转：造「释放之前读盘核那一读读不到 / 读出来对不上」这一格。
/// 盘上的字节一个不动，坏的只是交回的那一读。
/// `reads_to_refuse` 是还要坏几次（`u64::MAX` 当一直坏）；`reads_seen` 数这个偏移上一共读了几次（坏的、放行的都算）。
struct ReadsAtOneOffsetFail<Inner: BlockDevice> {
    inner: Inner,
    failing_offset: Option<DeviceOffsetInBytes>,
    form: FailedReadForm,
    reads_to_refuse: Cell<u64>,
    reads_seen: Cell<u64>,
}

impl<Inner: BlockDevice> BlockDevice for ReadsAtOneOffsetFail<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        if self.failing_offset == Some(offset) {
            self.reads_seen.set(self.reads_seen.get() + 1);
            if self.reads_to_refuse.get() > 0 {
                self.reads_to_refuse.set(self.reads_to_refuse.get() - 1);
                match self.form {
                    FailedReadForm::BlockDeviceError => {
                        return Err(injected_block_device_error("读"));
                    }
                    FailedReadForm::CorruptedBytes => {
                        self.inner.read_at(offset, buffer)?;
                        // 翻第 4096 字节起那个扇区的第一个字节：与 `corrupt_the_copy_on` 改盘上字节的是同一处，只是这回只坏在这一读。
                        buffer[4096] ^= 0xff;
                        return Ok(());
                    }
                }
            }
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

/// A 的数据单元在盘 0、盘 1 上那一份的读各先拒 `reads_to_refuse_on_each_device` 次（返回块设备错），在这样的盘上覆盖写一次；
/// 交回发布的结局与盘 0、盘 1 上那个偏移各读了几次。
fn overwrite_with_the_first_reads_of_the_data_copies_refused(
    pool: &mut BuiltPool,
    reads_to_refuse_on_each_device: [u64; 2],
) -> (Result<TransactionOutput, PublishError>, Vec<u64>) {
    overwrite_with_the_first_reads_of_the_data_copies_failing(
        pool,
        FailedReadForm::BlockDeviceError,
        reads_to_refuse_on_each_device,
    )
}

/// 同 [`overwrite_with_the_first_reads_of_the_data_copies_refused`]，坏的形态由 `form` 给。
fn overwrite_with_the_first_reads_of_the_data_copies_failing(
    pool: &mut BuiltPool,
    form: FailedReadForm,
    reads_to_refuse_on_each_device: [u64; 2],
) -> (Result<TransactionOutput, PublishError>, Vec<u64>) {
    let first = pool.output.clone();
    let data_slot = data_slot_of_the_first_version(pool);
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
                    failing_offset: Some(data_slot.to_device_offset()),
                    form,
                    reads_to_refuse: Cell::new(
                        reads_to_refuse_on_each_device
                            [usize::try_from(identity.0).expect("盘号 0 或 1")],
                    ),
                    reads_seen: Cell::new(0),
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
    let reads_seen: Vec<u64> = wrapped
        .iter()
        .map(|(_, device)| device.reads_seen.get())
        .collect();
    pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );
    (result, reads_seen)
}

/// 用例 4：盘 0 上那一份的第一读报块设备错（瞬时读错），重读读得出、校验和对得上 ⇒ 不隔离，两盘都照常释放。
/// 这是用户 2026-09-24 定「先重读一次」要挡的那一格（判决 N1 的 F1）：不重读就把好槽当成对不上隔离掉。
#[test]
fn release_checksum_read_that_fails_once_is_read_again_and_the_intact_copy_is_released_as_usual() {
    let mut pool = build_pool("release-checksum-read-fails-once");
    let data_slot = data_slot_of_the_first_version(&pool);
    let (result, reads_seen) =
        overwrite_with_the_first_reads_of_the_data_copies_refused(&mut pool, [1, 0]);
    let second = result.expect("重读读得出、对得上：发布照成");
    assert_eq!(
        reads_seen,
        vec![2, 1],
        "盘 0 那一份读了两次（第一次报错、重读一次），盘 1 那一份读一次"
    );
    assert_eq!(
        second.quarantined_after_release_checksum_mismatch,
        Vec::new(),
        "重读读得出、校验和对得上的那一份不隔离"
    );
    assert_eq!(
        records_of_slot(&pool, data_slot),
        vec![
            (DeviceIdentity(0), true, CheckpointTxg(4)),
            (DeviceIdentity(1), true, CheckpointTxg(4)),
        ],
        "两盘都照常改写成已释放、释放代 4"
    );
}

/// 用例 5：两盘那一份都一直读不到：各先重读一次（恰好一次，不多次重试），还读不出就按对不上处置——
/// 两盘那条记录都留在已分配，发布照成（只一块盘读不出时两块盘的账会不对称，停在用例 2 那一格）。
#[test]
fn release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_the_mismatch() {
    let mut pool = build_pool("release-checksum-read-keeps-failing");
    let data_slot = data_slot_of_the_first_version(&pool);
    let (result, reads_seen) =
        overwrite_with_the_first_reads_of_the_data_copies_refused(&mut pool, [u64::MAX, u64::MAX]);
    let second = result.expect("读不出按对不上处置：发布照成");
    assert_eq!(
        reads_seen,
        vec![2, 2],
        "每块盘那一份读两次（第一次、重读一次），之后不再读"
    );
    let data_placement = Placement {
        slot: data_slot,
        span: 2,
    };
    assert_eq!(
        second.quarantined_after_release_checksum_mismatch,
        vec![
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: DATA,
                device: DeviceIdentity(0),
                placement: data_placement,
                reading: QuarantinedCopyReading::UnreadableAfterOneReread,
            },
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: DATA,
                device: DeviceIdentity(1),
                placement: data_placement,
                reading: QuarantinedCopyReading::UnreadableAfterOneReread,
            },
        ],
        "读不出的那两份按对不上隔离"
    );
    assert_eq!(
        records_of_slot(&pool, data_slot),
        vec![
            (DeviceIdentity(0), false, CheckpointTxg(3)),
            (DeviceIdentity(1), false, CheckpointTxg(3)),
        ],
        "两盘那条都留在已分配"
    );
}

/// 用例 12：盘 0 上那一份的第一读读得出、交回的字节坏了一个（瞬时坏读，盘上的字节是好的），重读对得上 ⇒ 不隔离，两盘都照常释放
/// （D19（块指针的结构与宽度预算） 已定项 5「硬规则 1 的读盘核读不出、核出对不上时怎么办」：读成功而核出对不上也先重读一次，
/// 主 agent 2026-09-25 定——一次瞬时坏读不许让单元永久隔离）。只在读不出时重读的实现在这里把好的那一对槽隔离掉：
/// 盘 0 那一份读一次、核出对不上，两块盘的记录一起留在已分配。
#[test]
fn release_checksum_read_that_returns_corrupted_bytes_once_is_read_again_and_the_intact_copy_is_released_as_usual(
) {
    let mut pool = build_pool("release-checksum-read-corrupted-once");
    let data_slot = data_slot_of_the_first_version(&pool);
    let (result, reads_seen) = overwrite_with_the_first_reads_of_the_data_copies_failing(
        &mut pool,
        FailedReadForm::CorruptedBytes,
        [1, 0],
    );
    let second = result.expect("重读对得上：发布照成");
    assert_eq!(
        second.quarantined_after_release_checksum_mismatch,
        Vec::new(),
        "重读对得上的那一份不隔离，另一块盘那一份也不跟着留"
    );
    assert_eq!(
        reads_seen,
        vec![2, 1],
        "盘 0 那一份读了两次（第一次坏、重读一次），盘 1 那一份读一次"
    );
    assert_eq!(
        records_of_slot(&pool, data_slot),
        vec![
            (DeviceIdentity(0), true, CheckpointTxg(4)),
            (DeviceIdentity(1), true, CheckpointTxg(4)),
        ],
        "两盘都照常改写成已释放、释放代 4"
    );
}

/// 用例 13：盘 0 上那一份每一读都交回坏了一个字节的内容（坏读一直在，盘上的字节是好的）：先重读一次（恰好一次），
/// 两次都对不上才按对不上处置——两块盘的记录一起留在已分配，盘 0 那一份报「重读之后仍对不上」、盘 1 那一份报「对得上而一起留」。
/// 这一格是故障注入里「每次读都给坏字节」的形态：隔离的是一对好槽，那是「两次都对不上才隔离」这条规则认下的代价，不是实现的错。
#[test]
fn release_checksum_read_that_keeps_returning_corrupted_bytes_is_read_once_more_and_then_quarantined(
) {
    let mut pool = build_pool("release-checksum-read-keeps-corrupting");
    let data_slot = data_slot_of_the_first_version(&pool);
    let (result, reads_seen) = overwrite_with_the_first_reads_of_the_data_copies_failing(
        &mut pool,
        FailedReadForm::CorruptedBytes,
        [u64::MAX, 0],
    );
    let second = result.expect("两次都对不上按对不上处置：发布照成");
    assert_eq!(
        reads_seen,
        vec![2, 1],
        "盘 0 那一份读两次（第一次、重读一次），之后不再读；盘 1 那一份读一次"
    );
    let data_placement = Placement {
        slot: data_slot,
        span: 2,
    };
    assert_eq!(
        second.quarantined_after_release_checksum_mismatch,
        vec![
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: DATA,
                device: DeviceIdentity(0),
                placement: data_placement,
                reading: QuarantinedCopyReading::ChecksumMismatch,
            },
            CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: DATA,
                device: DeviceIdentity(1),
                placement: data_placement,
                reading: QuarantinedCopyReading::IntactButAnotherCopyFailed,
            },
        ],
        "盘 0 那一份两次都对不上，盘 1 那一份为了两块盘的账对称一起留"
    );
    assert_eq!(
        records_of_slot(&pool, data_slot),
        vec![
            (DeviceIdentity(0), false, CheckpointTxg(3)),
            (DeviceIdentity(1), false, CheckpointTxg(3)),
        ],
        "两盘那条都留在已分配"
    );
}

/// 进程重开、可写挂载之后，把能回收的都回收掉（回收下界取到顶：相当于 F 抬过了每一个释放代），交回挂载之后的分配器。
/// 这一步只是把「释放代 ≤ 回收下界」那一半谓词一口气推到底，好让「隔离的槽回不回空闲池」在一次挂载里就看得见。
fn remount_and_reclaim_everything_reclaimable(pool: &mut BuiltPool) {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.allocator
        .reclaim_released_up_to(CheckpointTxg(u64::MAX), ReclaimedReuse::Immediately);
}

/// 用例 6：隔离跨重挂（用户 2026-09-24 定「落盘即跨重挂」）。两盘那一份都改坏、覆盖写（txg 4）隔离之后进程退出、重开可写挂载：
/// 分配器从盘上那棵分配记录树重建，那一槽的记录读回来仍是已分配（分配代 3）；把能回收的全回收掉，它也不回空闲池、
/// 接连要几个用户数据落点都不会拿到它。改之前隔离只住内存：重开之后那条记录读回来是「已释放、释放代 4」，
/// F 抬过去就回收、再发出去（判决 N3：写者错指到照抄着的活叶那一格，重挂之后活叶被覆写）。
#[test]
fn quarantined_copy_stays_allocated_across_the_remount_and_is_never_handed_out_again() {
    let mut pool = build_pool("release-checksum-across-remount");
    let first = pool.output.clone();
    let data_slot = data_slot_of_the_first_version(&pool);
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot);
        corrupt_the_copy_on(devices, DeviceIdentity(1), data_slot);
    }
    publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(2800, 5),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("核出对不上时发布照成");

    remount_and_reclaim_everything_reclaimable(&mut pool);
    let reopened = pool.memory_pool();
    let system_configuration = choose_system_configuration(&reopened).expect("系统配置");
    let newest = choose_root(&reopened, &system_configuration).expect("最新根");
    let records_on_disk: Vec<(DeviceIdentity, bool, CheckpointTxg)> =
        allocation_records_under_root(&reopened, &newest)
            .expect("最新根那棵账读得出")
            .into_iter()
            .filter(|record| record.slot == data_slot)
            .map(|record| (record.device, record.is_released, record.generation))
            .collect();
    assert_eq!(
        records_on_disk,
        vec![
            (DeviceIdentity(0), false, CheckpointTxg(3)),
            (DeviceIdentity(1), false, CheckpointTxg(3)),
        ],
        "重挂之后最新根那棵账里，那一槽两盘的记录仍是已分配、分配代 3"
    );
    for slot in [data_slot, SlotNumber(data_slot.0 + 1)] {
        assert_eq!(
            is_free_on_each_device(&pool, slot),
            vec![false, false],
            "重挂、回收到顶之后隔离的槽 {slot:?} 仍不在空闲池里"
        );
    }
    let handed_out: Vec<SlotNumber> = (0..8)
        .map(|_| {
            pool.allocator
                .allocate_user_data(CheckpointTxg(u64::MAX))
                .expect("单元区里有的是空槽")
                .slot
        })
        .collect();
    assert!(
        !handed_out.contains(&data_slot),
        "隔离的 {data_slot:?} 不被再发出去：{handed_out:?}"
    );
    assert!(
        handed_out.iter().any(|slot| *slot > data_slot),
        "发出去的落点越过了 {data_slot:?}（它要是空闲的，早轮到它了）：{handed_out:?}"
    );
}

/// 用例 7：准入照已分配算、不另进式子（用户 2026-09-24 定案）。两个池走同一段历史（新池新建文件、一次覆盖写、重开可写挂载、回收到顶），
/// 只一个池在覆盖写之前把 A 的数据单元两盘那一份都改坏：那个池的两盘那条记录留在已分配，另一个池的照常释放、回收回空闲——
/// 准入读数里两块盘的「已分配」都恰好多那一份的两槽、可用都恰好少两槽；式子里没有为它另开的一项，「被抛弃根独占量」那一项都是 0。
#[test]
fn quarantined_copy_counts_as_allocated_in_the_admission_reading_and_adds_no_term_of_its_own() {
    let reading_after = |tag: &str, corrupt_both_copies: bool| {
        let mut pool = build_pool(tag);
        let first = pool.output.clone();
        let data_slot = data_slot_of_the_first_version(&pool);
        if corrupt_both_copies {
            let devices = pool.devices.as_mut().expect("镜像还开着");
            corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot);
            corrupt_the_copy_on(devices, DeviceIdentity(1), data_slot);
        }
        publish_overwrite_in_process(
            &mut pool,
            &first,
            &content_of(2700, 9),
            FIXED_WRITE_TIME_SECONDS + 60,
            InstanceGeneration(1),
        )
        .expect("覆盖写（核出对不上时照成）");
        remount_and_reclaim_everything_reclaimable(&mut pool);
        AdmissionReading::of_allocator(
            &pool.allocator,
            BytesOnOneDevice::ZERO,
            PoolWideCommitments::of_the_first_version(
                BytesSummedOverAllReplicas::ZERO,
                BytesSummedOverAllReplicas::ZERO,
            ),
            BytesOnOneDevice::ZERO,
        )
    };
    let quarantined = reading_after("release-checksum-admission", true);
    let intact = reading_after("release-checksum-admission-control", false);
    let two_slots = 2 * SLOT_BYTES;
    for (quarantined_terms, intact_terms) in
        quarantined.per_device().iter().zip(intact.per_device())
    {
        assert_eq!(quarantined_terms.device, intact_terms.device);
        assert_eq!(
            quarantined_terms.allocated.0,
            intact_terms.allocated.0 + two_slots,
            "盘 {:?}：已分配恰好多那一份的两槽：{quarantined_terms:?} 对 {intact_terms:?}",
            quarantined_terms.device
        );
        assert_eq!(
            quarantined_terms.isolated_for_abandoned_roots_and_unapplied_journal_records,
            BytesOnOneDevice::ZERO,
            "不另进式子：被抛弃根独占量那一项是 0"
        );
    }
    for ((device, quarantined_available), (_, intact_available)) in quarantined
        .available_on_each_device()
        .into_iter()
        .zip(intact.available_on_each_device())
    {
        assert_eq!(
            quarantined_available.0 + i128::from(two_slots),
            intact_available.0,
            "盘 {device:?}：可用恰好少两槽"
        );
    }
}

/// 把上一版的映射节点重装成「码 1 那一条的第二条位置项指到盘 `device`」的形态，别的条目不动（节点照样自洽、校验和对得上）。
fn previous_with_the_data_mapping_entry_naming_device(
    output: &TransactionOutput,
    device: DeviceIdentity,
) -> TransactionOutput {
    let mut damaged = output.clone();
    let data_key = data_mapping_key(output);
    let mapping_node = parse_index_node(&damaged.unit(TransactionUnit::MappingTree).bytes)
        .expect("上一版的映射节点解得开");
    let entries: Vec<Vec<u8>> = mapping_node
        .entries
        .iter()
        .map(|entry| {
            let (key, mut locations) = parse_mapping_entry(entry).expect("上一版的映射条目宽 55");
            if key == data_key {
                locations[1].device = device;
                build_mapping_entry(&key, locations)
            } else {
                entry.clone()
            }
        })
        .collect();
    let rebuilt = build_index_node(
        mapping_node.tree,
        mapping_node.level,
        mapping_node.key_width,
        &entries[0][..mapping_node.key_width],
        &entries[entries.len() - 1][..mapping_node.key_width],
        mapping_node.birth_txg,
        &parameters().filesystem_identifier,
        mapping_node.instance,
        mapping_node.birth_sequence,
        u16::try_from(mapping_node.entry_width).expect("条目宽"),
        &entries,
    );
    let mapping_index = damaged
        .units
        .iter()
        .position(|unit| unit.identity == TransactionUnit::MappingTree)
        .expect("八个单元每种一个");
    damaged.units[mapping_index].bytes = rebuilt;
    damaged
}

/// 在上一版的映射里把 A 的数据单元那条映射条目的第二条位置项改指 `device`，覆盖写一次：交回结局，连同覆盖写之前与之后盘上可比的快照
/// 与分配器的记录（「在任何写之前拒绝、盘上不变」要比的三样）。
fn overwrite_on_a_mapping_entry_whose_second_location_names(
    pool: &mut BuiltPool,
    device: DeviceIdentity,
) -> (Result<TransactionOutput, PublishError>, bool, bool) {
    let previous = previous_with_the_data_mapping_entry_naming_device(&pool.output, device);
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let records_before = pool.allocator.records().to_vec();
    let result = publish_overwrite_in_process(
        pool,
        &previous,
        &content_of(2500, 17),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    );
    let disk_unchanged = disk_snapshot(&pool.memory_pool(), &pool.stream) == before;
    let records_unchanged = pool.allocator.records() == records_before.as_slice();
    (result, disk_unchanged, records_unchanged)
}

/// 用例 8：映射条目的一条位置项指到盘 7（池里只有 0 与 1；盘上读来的字节，坏镜像、外来镜像才有）。
/// D19（块指针的结构与宽度预算） 已定项 5（用户 2026-09-25 定）：位置项指向一块不在池里的盘，当映射条目损坏，在任何写之前拒绝、盘上不变——
/// 盘上逐字节不变（两盘四个系统配置槽、根环里全部自证过的根、录制流步数），分配器一条记录都没改写。
#[test]
fn mapping_location_on_the_device_outside_the_pool_is_refused_as_the_damaged_mapping_entry_before_anything_is_written(
) {
    let mut pool = build_pool("release-checksum-foreign-device");
    let data_slot = data_slot_of_the_first_version(&pool);
    let (result, disk_unchanged, records_unchanged) =
        overwrite_on_a_mapping_entry_whose_second_location_names(&mut pool, DeviceIdentity(7));
    assert!(
        matches!(
            result,
            Err(PublishError::MappingEntryLocationOnADeviceOutsideThePool {
                unit: DATA,
                device: DeviceIdentity(7),
                slot,
            }) if slot == data_slot
        ),
        "位置项指的盘不在池里 ⇒ 当映射条目损坏拒绝：{result:?}"
    );
    assert!(
        disk_unchanged,
        "在任何落盘动作之前返回：系统配置槽、根环、录制流步数逐项不变"
    );
    assert!(records_unchanged, "分配器一条记录都没改写");
}

/// 用例 9：映射条目的两条位置项都指盘 0（盘上读来的字节）。D19（块指针的结构与宽度预算） 已定项 5（用户 2026-09-25 定）：
/// 两条位置项指同一块盘，当映射条目损坏，在任何写之前拒绝、盘上不变。不拒的话盘 0 那一份核两遍、盘 1 那一份从来没核过就被释放。
#[test]
fn two_mapping_locations_on_the_same_device_are_refused_as_a_damaged_mapping_entry_before_anything_is_written(
) {
    let mut pool = build_pool("release-checksum-same-device-twice");
    let data_slot = data_slot_of_the_first_version(&pool);
    let (result, disk_unchanged, records_unchanged) =
        overwrite_on_a_mapping_entry_whose_second_location_names(&mut pool, DeviceIdentity(0));
    assert!(
        matches!(
            result,
            Err(PublishError::MappingEntryLocationsOnTheSameDevice {
                unit: DATA,
                device: DeviceIdentity(0),
                slot,
            }) if slot == data_slot
        ),
        "两条位置项指同一块盘 ⇒ 当映射条目损坏拒绝：{result:?}"
    );
    assert!(
        disk_unchanged,
        "在任何落盘动作之前返回：系统配置槽、根环、录制流步数逐项不变"
    );
    assert!(records_unchanged, "分配器一条记录都没改写");
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read(
) {
    scenarios_of_release_checksum_quarantine::quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not() {
    scenarios_of_release_checksum_quarantine::unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry(
) {
    scenarios_of_release_checksum_quarantine::quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry(&mut PoolCheckerImages::left_to_the_checker_tier());
}
