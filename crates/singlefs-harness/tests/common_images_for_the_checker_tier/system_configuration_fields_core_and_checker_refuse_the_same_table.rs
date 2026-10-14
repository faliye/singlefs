//! harness 测试文件 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{build_pool, parameters};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_checker::image::{
    IMPLEMENTED_INVARIANTS, SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
};
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
pub const JOURNAL_RING_START_SLOT_OFFSET: usize = 325;

/// 系统配置槽里 journal 环长那 8 字节（字节数）。
pub const JOURNAL_RING_BYTES_OFFSET: usize = 333;

/// 系统配置槽里根环起点那 8 字节（16 KiB 槽号，D22（单元原子性怎么合成） 已定项 16 第 1 句）。
pub const ROOT_RING_BASE_SLOT_OFFSET: usize = 371;

/// 系统配置槽里单元区起始槽号那 8 字节。
pub const UNIT_AREA_START_SLOT_OFFSET: usize = 417;

/// 默认环长下单元区起始槽号：journal 环起点 1024 + 768 MiB ÷ 16 KiB = 50176（在 64 槽段边界上）。
pub const UNIT_AREA_START_SLOT_OF_THE_DEFAULT_RING: u64 =
    JOURNAL_RING_START_SLOT + JOURNAL_RING_DEFAULT_BYTES / SLOT_BYTES;

/// 两块盘四个系统配置槽里，`fields_written` 的每个 (偏移, 值) 都写进去，整槽校验和重封。
pub fn rewrite_every_system_configuration_slot(
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
pub fn core_and_checker_refuse_the_cell(
    images: &mut PoolCheckerImages,
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

    // 池级 checker：四槽都带这一版读者不收的值，I-7.13 判违例、说明里点名它拒的成员；池挂不上，别的每一条报不适用
    let expected = IMPLEMENTED_INVARIANTS
        .iter()
        .filter(|invariant| {
            **invariant != SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
        })
        .fold(
            PoolCheckerExpectation::exactly_these_violated(&[
                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
            ])
            .and_violated_with_a_detail_containing(
                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
                checker_names_member,
            ),
            |expected, invariant| expected.and_not_judged(invariant),
        );
    images.hand_out_allowing_the_gpu_to_refuse_its_tables(
        tag,
        || image,
        expected,
        Some("四个系统配置槽都带这一版读者不收的几何字段：GPU 判器照它装判表可能装不出"),
    );
}

/// A 格：单元区起始槽号改成现算起点 + 64（仍在段边界上、环末端不越过它）。
pub fn a_unit_area_start_one_segment_past_the_ring_is_refused_by_core_and_reddens_the_checker_table(
    images: &mut PoolCheckerImages,
) {
    core_and_checker_refuse_the_cell(images,
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
pub fn a_journal_ring_start_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table(
    images: &mut PoolCheckerImages,
) {
    core_and_checker_refuse_the_cell(images,
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
pub fn a_root_ring_base_other_than_the_first_version_slot_is_refused_by_core_and_reddens_the_checker_table(
    images: &mut PoolCheckerImages,
) {
    core_and_checker_refuse_the_cell(
        images,
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
pub fn a_unit_area_start_off_the_cluster_segment_boundary_is_refused_by_core_and_reddens_the_checker_table(
    images: &mut PoolCheckerImages,
) {
    core_and_checker_refuse_the_cell(images,
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
pub fn a_root_ring_base_of_zero_is_refused_by_core_for_the_slot_spacing_and_reddens_the_checker_table_for_the_base(
    images: &mut PoolCheckerImages,
) {
    core_and_checker_refuse_the_cell(images,
        "system-configuration-table-e-root-ring-base-0",
        &[(ROOT_RING_BASE_SLOT_OFFSET, 0)],
        SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange {
            fixed_structure_slot_spacing: 4096,
        },
        "RootRingBaseSlotNotTheFirstVersionConstant",
    );
}
