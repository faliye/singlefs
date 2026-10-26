//! G4 拒绝那一臂（崩溃注入第二、三截判「读阶段判完之后系统配置又读不出」那一拒，`singlefs_harness::mount_transparency`）判槽自证读池级 checker
//! 的逐盘数（`singlefs_checker::image::system_configuration_redundancy_report`），不用核心层的 `SystemConfiguration::parse_slot` 与
//! `choose_system_configuration`（用户 2026-10-22 定「改读池级 checker」，`research/prompts/m3-close-g4-rewrite-r3-main-verification.md`
//! 「用户定案（2026-10-22，弹窗）」；`.claude/rules/fs-design.md`「运行时与 checker 必须用不同的算法」）。
//! 两种读法的口径差在「自证过」分不分 fsid：池级 checker 只看 magic 与整槽校验和（同 I-8.5），核心层只认 fsid 与本池相同的槽。
//! 用例钉住这一臂读的是池级 checker 那一份：盘 0 两槽都换成别的池的 fsid 再重封校验和（magic 与整槽校验和都对），盘 1 不动；
//! 崩溃态没注入读故障，可写挂载以「读阶段判完之后系统配置又读不出」被拒。核心层口径下盘 0 一份本池自证过的都没有，U6 复算判据为真、拒得合法；
//! G4 那一臂照池级 checker 的逐盘数，盘 0 两槽都自证过、判据为假，不另记「没有读故障的历史留不下这份盘面」。这一臂换回核心层的解析，
//! 就会多记那一条、判成不合法。小池是两块文件镜像，第一个文件之后的盘面（`common::build_pool`）；崩溃镜像是内存盘面本身，读不报错。

mod common;

use common::build_pool;
use singlefs_checker::image::{
    system_configuration_redundancy_report, SystemConfigurationRedundancyReport,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::mount::{
    MountError, NewerPublishWitness, RollbackTarget, SelectedVersionAgainstTheWitness,
    SystemConfigurationUnreadableAfterTheReadStage, SystemConfigurationUnreadableWhen,
    WitnessedCounterComparison,
};
use singlefs_core::recovery::PoolReader;
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite, WrittenContents};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state, DiskAfterTheRefusal, ReadFaultsInTheCrashState,
    ReadOnlyReadBackOfTheCrashImage, RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
};
use singlefs_harness::segments::StepKind;

/// 系统配置里 fsid 那 16 字节的起点（字段表 `layout/01-first-txn.md` 一：magic 4、格式版本 2、feature 位 96 之后）。
const FILESYSTEM_IDENTIFIER_OFFSET_IN_THE_SLOT: usize = 4 + 2 + 96;
/// fsid 的字节数。
const FILESYSTEM_IDENTIFIER_BYTES: usize = 16;
/// 换上去的 fsid 每个字节都异或这个值：与本池的 fsid 逐字节不同。
const FOREIGN_FILESYSTEM_IDENTIFIER_BYTE_MASK: u8 = 0x5A;

/// 一块盘系统配置槽 `slot_index` 的设备内偏移：槽 i 在 i × 固定结构槽距（D22（单元原子性怎么合成） 已定项 16）。
fn system_configuration_slot_offset(slot_index: u64) -> u64 {
    slot_index * u64::from(common::geometry().fixed_structure_slot_spacing)
}

/// 盘 `device` 的槽 `slot_index` 换成别的池的 fsid、重封整槽校验和：magic 与整槽校验和都对，只是 fsid 不是本池的。
fn slot_rewritten_under_a_foreign_filesystem_identifier(
    pool: &MemoryPool,
    device: DeviceIdentity,
    slot_index: u64,
) -> RetainedWrite {
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节装得进 usize");
    let offset = DeviceOffsetInBytes(system_configuration_slot_offset(slot_index));
    let mut slot = PoolReader::read(pool, device, offset, slot_bytes)
        .expect("第一个文件之后的池里这一槽读得出");
    for byte in &mut slot[FILESYSTEM_IDENTIFIER_OFFSET_IN_THE_SLOT
        ..FILESYSTEM_IDENTIFIER_OFFSET_IN_THE_SLOT + FILESYSTEM_IDENTIFIER_BYTES]
    {
        *byte ^= FOREIGN_FILESYSTEM_IDENTIFIER_BYTE_MASK;
    }
    let digest =
        wide_checksum_with_field_zeroed(&slot, slot_bytes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + digest.len()]
        .copy_from_slice(&digest);
    RetainedWrite {
        device,
        kind: StepKind::SystemConfigurationSlot,
        is_force_unit_access: false,
        offset,
        contents: WrittenContents::Bytes(slot),
    }
}

/// 盘 0 两槽都只带别的池的系统配置（自证过、fsid 不是本池的），没注入读故障，可写挂载以「读阶段判完之后系统配置又读不出」被拒：
/// U6 那几条照核心层口径复算为真，拒得合法；G4 那一臂照池级 checker 的逐盘数，盘 0 两槽都自证过，不记「没有读故障的历史留不下这份盘面」。
#[test]
fn a_refusal_after_the_read_stage_where_device_zero_carries_only_another_pools_slots_stays_legal_under_the_pool_checker_counts(
) {
    let healthy = build_pool("g4-arm-counts-slots-by-the-pool-checker").memory_pool();
    let mut crash_image = healthy.clone();
    let foreign_slots: Vec<RetainedWrite> = [0, 1]
        .into_iter()
        .map(|slot_index| {
            slot_rewritten_under_a_foreign_filesystem_identifier(
                &healthy,
                DeviceIdentity(0),
                slot_index,
            )
        })
        .collect();
    crash_image.apply_writes(&foreign_slots);
    assert_eq!(
        system_configuration_redundancy_report(&crash_image),
        SystemConfigurationRedundancyReport {
            devices_without_a_self_verified_slot: Vec::new(),
            devices_with_fewer_than_two_self_verified_slots: Vec::new(),
        },
        "前提：池级 checker 的逐盘数里盘 0 两槽都自证过（magic 与整槽校验和对，不看 fsid）"
    );
    // 被拒的那次挂载交回的错：判官只认拒因（单子上哪一条），读阶段那一版与见证读数不进判法，取第一个文件那一版、什么都没见证
    let refusal = MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
        SystemConfigurationUnreadableAfterTheReadStage {
            read_stage: SelectedVersionAgainstTheWitness {
                selected_version: RollbackTarget {
                    instance: InstanceGeneration(1),
                    checkpoint_txg: CheckpointTxg(1),
                },
                witness: NewerPublishWitness {
                    witnessed_journal_counter: 0,
                    comparison: WitnessedCounterComparison::NothingWitnessed,
                },
            },
            unreadable: SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                device: DeviceIdentity(0),
            },
        },
    ));
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state(
            &crash_image,
            None,
            &refusal,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
        ),
        "U6 照核心层口径复算为真（盘 0 一份本池自证过的都没有）；G4 那一臂读池级 checker 的逐盘数，盘 0 两槽都自证过，不另记一条"
    );
}
