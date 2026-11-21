//! harness 测试文件 `crates/singlefs-verification/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_given_states_engine/mod.rs`）。那份 harness 测试文件与给定状态那一份
//! `crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "引它的测试目标各只调其中一部分")]

use crate::common_images_for_the_given_states_engine::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_unwitnessed_publish::{
    content_of, device_bytes_of_the_384_slot_small_devices, devices_reading_back_all_zero_on,
    devices_reading_back_all_zero_on_until_lifted, fixed_geometry_of_the_384_slot_small_devices,
    image_of_devices_reading_back_all_zero,
    image_where_both_devices_lost_their_newest_rotation_write,
    parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on, pool_after_the_mount,
    record_one_publish_after_overwrites, root_is_in_the_ring,
    units_of_the_recorded_publish_overwritten_on, DeviceOverASparseImage,
    DeviceReadingBackAllZeroOnRanges, PoolUnderTest, RangesReadingBackAllZero, ReadBackAllZeroLift,
    RecordedPublish, RecordedPublishKind, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{PlacementOnDevice, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor,
    raise_rollback_floor_to_the_admission_ceiling,
    ring_slots_recorded_as_holding_an_abandoned_root_by, roll_back_by_a_forward_publish, unmount,
    FloorRaiseStop, RaiseToTheAdmissionCeiling, RollbackTarget, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::{
    MountedSession, NoSpaceAfterRaisingTheFloor, RootRingPushRefused, RootRingPushRehearsal,
    RootRingPushed, StuckBehindTheWideProtection, UserChange, UserChangeRefused,
    WayOutOfTheWideProtection, WaysOutOfTheWideProtection, WaysOutOfTheWideProtectionRefused,
    WhatIsLeftAfterThePush, WideProtectionAtTheCeiling,
};
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::recovery::choose_system_configuration;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::{FirstFile, PoolVersion, PublishError};
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
    SparseDevice,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::{RecordingBlockDevice, SharedStream};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/abandoned_root_whose_account_or_ring_slot_turns_unreadable.rs"]
mod abandoned_root_whose_account_or_ring_slot_turns_unreadable;
#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/defer_slots_an_abandoned_root_references.rs"]
mod defer_slots_an_abandoned_root_references;
#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/long_sequential_write_stuck_behind_the_wide_protection.rs"]
mod long_sequential_write_stuck_behind_the_wide_protection;
#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/pushing_the_root_ring_refused_or_writing_nothing.rs"]
mod pushing_the_root_ring_refused_or_writing_nothing;
#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/revived_units_isolated_when_the_floor_raise_reclaims_them.rs"]
mod revived_units_isolated_when_the_floor_raise_reclaims_them;
#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/ring_slots_holding_placements.rs"]
mod ring_slots_holding_placements;
#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/root_abandoned_by_the_mounts_own_row.rs"]
mod root_abandoned_by_the_mounts_own_row;
#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/stuck_behind_the_wide_protection_and_pushing_the_root_ring.rs"]
mod stuck_behind_the_wide_protection_and_pushing_the_root_ring;
#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/units_a_rollback_revived.rs"]
mod units_a_rollback_revived;
#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/ways_out_of_the_wide_protection.rs"]
mod ways_out_of_the_wide_protection;
#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/ways_out_refused_on_a_moved_session_or_a_changed_device_table.rs"]
mod ways_out_refused_on_a_moved_session_or_a_changed_device_table;
#[path = "raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table/wide_protection_after_releasing_a_large_batch_on_the_1536_slot_devices.rs"]
mod wide_protection_after_releasing_a_large_batch_on_the_1536_slot_devices;

#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use abandoned_root_whose_account_or_ring_slot_turns_unreadable::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use defer_slots_an_abandoned_root_references::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use long_sequential_write_stuck_behind_the_wide_protection::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use pushing_the_root_ring_refused_or_writing_nothing::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use revived_units_isolated_when_the_floor_raise_reclaims_them::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use ring_slots_holding_placements::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use root_abandoned_by_the_mounts_own_row::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use stuck_behind_the_wide_protection_and_pushing_the_root_ring::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use units_a_rollback_revived::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use ways_out_of_the_wide_protection::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use ways_out_refused_on_a_moved_session_or_a_changed_device_table::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use wide_protection_after_releasing_a_large_batch_on_the_1536_slot_devices::*;

impl DeviceOverASparseImage for DeviceReadingBackAllZeroOnRanges<SparseBlockDevice> {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}
