//! core 与池级 checker 收同一张系统配置几何字段表（代码三方 m2-closeout-code-r2 Y4-a，用户 2026-09-27 定「读字段，不等于常量就整池拒」）：
//! 核心层读者（`singlefs-core` 的 `recovery::system_configuration_values_this_reader_accepts`，改法 B）与池级 checker 的
//! I-7.13（系统配置池级字段在读者收的范围里）（`singlefs-checker` 的 `image::geometry_of`，改法 A）各写一份，这里在同一份镜像上两边一起判。
//! 攻方 Y4 那五格（`research/prompts/m2-closeout-code-r2-opus-model/opus_r2_y4_system_configuration_fields_core_and_checker_read_differently.rs`，
//! 只取它造镜像的写法）：新池新建文件之后那份合法镜像，两块盘四个系统配置槽里改同一个几何字段、整槽校验和重封。每一格：
//! core 的只读挂载与可写挂载都整池拒、拒成同一个值、两块盘逐字节不变；池级 checker 报 I-7.13 违例、说明里点名它拒的成员，
//! 别的每一条报不适用（池挂不上）。改之前（第二轮快照）core 不读 325 与 371：B、C、E 三格 core 照挂而 checker 报 I-7.13
//! 「实现整池拒绝挂载」，A、D 两格 core 拒而 I-7.13 不红。
//! E 格两边拒的理由不同：core 先判槽距上界（按同一槽自述的根环起点 0 算，槽 1 不在它之前），checker 先判根环起点等不等第一版常量；
//! 这里照今天两边各自的判定次序钉。

mod common;

use common::{build_pool, parameters};
use singlefs_checker::image::{
    InvariantVerdict, SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::mounted_read::{mount_read_only, MountReadOnlyFailure};
use singlefs_core::recovery::{
    RecoveryFailure, SystemConfigurationValueOutsideWhatThisReaderAccepts,
};
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_format::{
    JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, ROOT_RING_BASE_SLOT, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// 系统配置槽里 journal 环起点那 8 字节（16 KiB 槽号，字段表 `layout/01-first-txn.md` 一）。
const JOURNAL_RING_START_SLOT_OFFSET: usize = 325;
/// 系统配置槽里 journal 环长那 8 字节（字节数）。
const JOURNAL_RING_BYTES_OFFSET: usize = 333;
/// 系统配置槽里根环起点那 8 字节（16 KiB 槽号，D22（单元原子性怎么合成） 已定项 16 第 1 句）。
const ROOT_RING_BASE_SLOT_OFFSET: usize = 371;
/// 系统配置槽里单元区起始槽号那 8 字节。
const UNIT_AREA_START_SLOT_OFFSET: usize = 417;
/// 默认环长下单元区起始槽号：journal 环起点 1024 + 768 MiB ÷ 16 KiB = 50176（在 64 槽段边界上）。
const UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING: u64 =
    JOURNAL_RING_START_SLOT + JOURNAL_RING_DEFAULT_BYTES / SLOT_BYTES;

/// 两块盘四个系统配置槽里，`fields_written` 的每个 (偏移, 值) 都写进去，整槽校验和重封。
fn rewrite_every_system_configuration_slot(
    image: &mut MemoryPool,
    fields_written: &[(usize, u64)],
) {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for device in image.devices.values_mut() {
        for slot_offset in [DeviceOffsetInBytes(0), DeviceOffsetInBytes(spacing)] {
            let mut slot = device.read(slot_offset, slot_bytes);
            for (offset, value) in fields_written {
                slot[*offset..*offset + 8].copy_from_slice(&value.to_le_bytes());
            }
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                slot_bytes,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            device.write(slot_offset, &slot);
        }
    }
}

/// 一格的两边判定：core 的只读、可写挂载都拒成 `core_refuses_as`、盘上逐字节不变；checker 的 I-7.13 违例、说明里点名
/// `checker_names_member`，别的每一条报不适用。
fn core_and_checker_refuse_the_cell(
    tag: &str,
    fields_written: &[(usize, u64)],
    core_refuses_as: SystemConfigurationValueOutsideWhatThisReaderAccepts,
    checker_names_member: &str,
) {
    let mut image = build_pool(tag).memory_pool();
    rewrite_every_system_configuration_slot(&mut image, fields_written);

    let read_only = mount_read_only(&image);
    assert!(
        matches!(
            &read_only,
            Err(MountReadOnlyFailure::Recovery(RecoveryFailure::SystemConfigurationValueRefused {
                value,
                ..
            })) if *value == core_refuses_as
        ),
        "{tag}：core 的只读挂载要拒成 {core_refuses_as:?}，得到 {:?}",
        read_only.as_ref().err()
    );
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect();
    let writable = mount_writable(&parameters(), &mut devices);
    assert!(
        matches!(
            &writable,
            Err(MountError::Recovery(RecoveryFailure::SystemConfigurationValueRefused {
                value,
                ..
            })) if *value == core_refuses_as
        ),
        "{tag}：core 的可写挂载要拒成 {core_refuses_as:?}，得到 {:?}",
        writable.as_ref().err()
    );
    for (identity, device) in &devices {
        assert!(
            device.image == image.devices[identity],
            "{tag}：盘 {} 逐字节不变",
            identity.0
        );
    }

    let verdicts = check_pool_image(&image);
    let mut judged_the_table = false;
    for (invariant, verdict) in &verdicts {
        if *invariant == SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS {
            judged_the_table = true;
            match verdict {
                InvariantVerdict::Violated(detail) => assert!(
                    detail.contains(checker_names_member),
                    "{tag}：I-7.13 的违例说明里要点名 {checker_names_member}：{detail}"
                ),
                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
                    panic!("{tag}：四槽都带这一版读者不收的值，I-7.13 要判违例，得到 {other:?}")
                }
            }
        } else {
            assert!(
                matches!(verdict, InvariantVerdict::NotApplicable(_)),
                "{tag}：池挂不上，{invariant} 要报不适用，得到 {verdict:?}"
            );
        }
    }
    assert!(judged_the_table, "{tag}：I-7.13 在判定清单里：{verdicts:?}");
}

/// A 格：单元区起始槽号改成现算起点 + 64（仍在段边界上、环末端不越过它）。
#[test]
fn a_unit_area_start_one_segment_past_the_ring_is_refused_by_core_and_reddens_the_checker_table() {
    core_and_checker_refuse_the_cell(
        "system-configuration-table-a-unit-area-start-plus-64",
        &[(UNIT_AREA_START_SLOT_OFFSET, UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING + 64)],
        SystemConfigurationValueOutsideWhatThisReaderAccepts::UnitAreaStartNotTheSlotAfterTheJournalRing {
            recorded_unit_area_start_slot: SlotNumber(UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING + 64),
            slot_after_the_journal_ring: SlotNumber(UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING),
        },
        "UnitAreaStartNotTheSlotAfterTheJournalRing",
    );
}

/// B 格：journal 环起点 1024 改成 1023。
#[test]
fn a_journal_ring_start_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table(
) {
    core_and_checker_refuse_the_cell(
        "system-configuration-table-b-journal-ring-start-1023",
        &[(JOURNAL_RING_START_SLOT_OFFSET, JOURNAL_RING_START_SLOT - 1)],
        SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingStartNotTheFirstVersionSlot {
            recorded_journal_ring_start_slot: SlotNumber(JOURNAL_RING_START_SLOT - 1),
            first_version_journal_ring_start_slot: SlotNumber(JOURNAL_RING_START_SLOT),
        },
        "JournalRingStartSlotNotTheFirstVersionConstant",
    );
}

/// C 格：根环起点 64 改成 65。
#[test]
fn a_root_ring_base_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table(
) {
    core_and_checker_refuse_the_cell(
        "system-configuration-table-c-root-ring-base-65",
        &[(ROOT_RING_BASE_SLOT_OFFSET, ROOT_RING_BASE_SLOT + 1)],
        SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingBaseNotTheFirstVersionSlot {
            recorded_root_ring_base_slot: SlotNumber(ROOT_RING_BASE_SLOT + 1),
            first_version_root_ring_base_slot: SlotNumber(ROOT_RING_BASE_SLOT),
        },
        "RootRingBaseSlotNotTheFirstVersionConstant",
    );
}

/// D 格：环长改成 768 MiB − 16 KiB、单元区起始槽号跟着改成它现算的 50175（两者自洽，不在 64 槽段边界上）。
#[test]
fn a_unit_area_start_off_the_cluster_segment_boundary_is_refused_by_core_and_reddens_the_checker_table(
) {
    core_and_checker_refuse_the_cell(
        "system-configuration-table-d-ring-off-the-segment-boundary",
        &[
            (JOURNAL_RING_BYTES_OFFSET, JOURNAL_RING_DEFAULT_BYTES - SLOT_BYTES),
            (UNIT_AREA_START_SLOT_OFFSET, UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING - 1),
        ],
        SystemConfigurationValueOutsideWhatThisReaderAccepts::UnitAreaStartOffTheClusterSegmentBoundaryUnsupported {
            unit_area_start_slot: SlotNumber(UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING - 1),
        },
        "UnitAreaStartOffTheClusterSegmentBoundary",
    );
}

/// E 格：根环起点 64 改成 0。core 先判槽距上界（槽 1 不再整槽落在同一槽自述的根环起点之前）、报槽距；checker 先判根环起点、报起点。
#[test]
fn a_root_ring_base_of_zero_is_refused_by_core_for_the_slot_spacing_and_reddens_the_checker_table_for_the_base(
) {
    core_and_checker_refuse_the_cell(
        "system-configuration-table-e-root-ring-base-0",
        &[(ROOT_RING_BASE_SLOT_OFFSET, 0)],
        SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange {
            fixed_structure_slot_spacing: 4096,
        },
        "RootRingBaseSlotNotTheFirstVersionConstant",
    );
}
