//! harness 测试文件 `crates/singlefs-verification/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_given_states_engine/mod.rs`）。那份 harness 测试文件与给定状态那一份
//! `crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "引它的测试目标各只调其中一部分")]

use crate::common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    start_plain, DeviceOverASparseImage, PoolUnderTest, FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
};
use crate::common_images_for_the_given_states_engine::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_shared_by_harness_scenarios::{
    deletion_switches_with_batches_of_one, plan_failing_every_write_from,
    READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, SlotNumber,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
    mount_writable_with_test_only_switches_and_deletion_switches,
    ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by, unmount,
    BeforeTheOneReread, MountError, Mounted, RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
};
use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
    verified_system_configuration_slots, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;

use singlefs_verification::fault_injection::FaultInjectingBlockDevice;
use singlefs_verification::history::HistoryDeviceWidth;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[path = "rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root/abandoned_root_self_verified_only_on_the_rebuild_pass.rs"]
mod abandoned_root_self_verified_only_on_the_rebuild_pass;
#[path = "rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root/abandoned_roots_slot_failing_on_the_rebuild_pass_or_the_reread.rs"]
mod abandoned_roots_slot_failing_on_the_rebuild_pass_or_the_reread;
#[path = "rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root/pools_before_the_remount_and_the_remount_under_the_plan.rs"]
mod pools_before_the_remount_and_the_remount_under_the_plan;
#[path = "rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root/rereads_and_mounts_on_the_crashed_and_the_live_file_pools.rs"]
mod rereads_and_mounts_on_the_crashed_and_the_live_file_pools;
#[path = "rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root/root_ring_reads_failing_in_the_rebuild_pass.rs"]
mod root_ring_reads_failing_in_the_rebuild_pass;
#[path = "rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root/root_self_verified_only_on_the_rebuild_pass_and_the_floor_raise_of_the_narrow_mount.rs"]
mod root_self_verified_only_on_the_rebuild_pass_and_the_floor_raise_of_the_narrow_mount;

#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use abandoned_root_self_verified_only_on_the_rebuild_pass::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use abandoned_roots_slot_failing_on_the_rebuild_pass_or_the_reread::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use pools_before_the_remount_and_the_remount_under_the_plan::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use rereads_and_mounts_on_the_crashed_and_the_live_file_pools::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use root_ring_reads_failing_in_the_rebuild_pass::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use root_self_verified_only_on_the_rebuild_pass_and_the_floor_raise_of_the_narrow_mount::*;
