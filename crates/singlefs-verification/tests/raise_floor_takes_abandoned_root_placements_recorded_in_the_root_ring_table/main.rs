//! 抬 F 那一串（准入与卸载）里的被抛弃根照挂载时记在根环表里的落点算影子账，不再从盘上重读它的账，读不出也不拒
//! （D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，D23（journal 的角色与格式） 已定项 14 回退段「抬 F 那一串（准入与卸载）不走这一道」）。
//! 「崩了再可写挂载」那一道照 C694 的 B 拒，不变。
//!
//! 造法照 `research/prompts/m3-batch10y-code-r1-opus-model/opus_10y_k4_probe.rs` 的 `abandoned_start`（384 槽小盘、之前覆盖写 1 次）：
//! 实例 1 第一个文件、覆盖写一次；崩了再挂（实例 2）覆盖写一次得 C；把 C 的根槽与数据单元抹成全零、每块盘的系统配置轮换退回 C 之前，
//! 崩了再挂（实例 3）择根落到 C 之前那一条、写行与暖机；再把抹掉的字节写回、进程退出。C 留在根环里，按实例 3 的表被抛弃。
//! 之后可写挂载一次（实例 4）：挂载那一刻 C 的账读得出，根环表里把 C 记成被抛弃根、带着它引用的落点。挂上之后 C 的账那几个单元
//! （树表、中央映射树根、分配记录树根的每条位置条目）从此读不出（这几块盘上的读报错）。
//! 抬 F 那一遍若照盘上重读 C 的账，读不出、从映射现算不成、重读一次仍不成、现行账里又没有 B 的凭据，就拒：正常卸载与会话写都被拒。
//! 抬 F 之后断言 C 记着的落点一槽都不空闲（隔离着，或现行账里还分配着）：记着的落点被削（只交一部分或交空）时，靠 C 之前那一版豁免、
//! 现行账里已释放的那几个（C 之前那一版的实例表片这类）回收出去，这一条红。
//!
//! 同一个池上另三形：
//! - H1「被抛弃根的槽不拒」：挂上之后 C 的**根槽**读报错（账读得出），抬 F 那一串（准入算上限、抬 F 重算影子账，卸载同一串）不拒，
//!   照根环表里记着的 C 的落点进影子账、不进上限；有效根的槽读坏、重读仍坏照拒。
//! - H2「用 C 那一圈记着的落点」：可写挂载自己写行之后才判成被抛弃的根 R（`tests/common_unwitnessed_publish/mod.rs` 那块盘，挂载那一刻
//!   R 的账之外写出的单元读回全零、择根退到 P，R 记成有效根），挂上之后 R 的账读报错：抬 F 照 C 那一圈在 R 那一槽记着的落点算、不读账、不拒。
//! - 挂载那一刻凭据放行的被抛弃根，根环表里记着的是空串：抬 F 照空串算、不再读它的账（攻方 M7）。
//! - H2 之上的丙：R 换成建一个 inode
//!   （与所选那一版 P 共用文件的数据单元），挂载那一刻 R 的账读回全零而记录齐，C 那一槽另并上那一刻现行账里仍分配着的落点；
//!   会话覆盖写释放了那几个共用的数据单元、推抬 F、再顺序写，池级 checker 不红 I-7.4。只有 H2、没有丙时这一形红（攻方 K2-2）。
//! - 公开的抬 F 上限（`rollback_floor_ceiling`）照抬 F 那一串的口径取拒集（正推 K1-1）：C 的根槽读坏不拒、不进上限。
//! - 丙补洞两形：
//!   丙并中间——R 与所选那一版 P 之间还有一次发布 Q（攻方一形），C 里 R 那一槽并上 Q 那一槽记着的落点，会话写到写掉 R 那一槽，
//!   每一步根落之前那一态与每一步之后池级 checker 不红 I-7.4；
//!   回退复活——R 是一次管理员回退（正推一形），C 里 R 那一槽并上回退目标账里仍分配、P 的账里已释放的落点，推过抬 F 复活的槽不空闲、不红 I-7.4。
//! - 认不出时宽护（tri）：
//!   回退复活那一形再让 R 写的谱系片、R 写的全部单元、R_old 的根槽、R_old 的账之一读回全零，C 里 R 那一槽退到现行账里全部已释放的落点，
//!   写到推过抬 F 不红 I-7.4；认得出回退时只多 R_old 账里仍分配、现行账里已释放的那一截；R 不是回退时一槽不多、挂载那一刻隔离数与之前相同。
//! - 被宽护卡住的报与推环入口：384 槽盘上
//!   那几形卡住时，`NoSpaceAfterRaisingTheFloor` 点名「被宽护卡住」、带 R 的 txg、挡着的槽数与两条出路；
//!   调推环入口之后 R 那一槽写掉、隔离回 0、之后写放得下；卸载重挂那一条出路也走得通；推到半路取不到落点时在任何写之前拒。
//! - 被抛弃根那一圈遇到 defer 槽不双扣：挂载那一刻与根槽读坏时的抬 F 都只隔离分配位没置着的槽，回收时照根环表补。
//! - 出路按预演定（甲：预演推环与准入，推不通就不列）：
//!   K1 那一形（`research/prompts/m3-batch10z-pqrs-code-r1-opus-model/atk_part.rs`）
//!   推完放得下的写列两条出路、照推环走完放得下，推完放不下的写（含没有 C 那一圈也被拒的长度、三形长 60 与 80 个单元）只列卸载再挂、
//!   照它走完放得下；出路一按 R 那一槽有没有读得出的有效根分两种说法，出路二写推几次、写掉几槽、放出来的隔离槽数（推完前后之差，两槽 C 重叠时不重数）；
//!   推环入口冻结着时第一道就拒、不核盘表时盘体对调照推（攻方 M1、M2）、两槽 C 推到较晚的那一槽（攻方 M12）；C 记着落点而一槽都没隔离、
//!   停因是预算用完或抬 F 取不到落点时不报被宽护卡住，记着空串的槽不算（正推 K4-2 活下来的几个）；根槽读坏的被抛弃根那一圈补满整段跨度。
//! - 要出路先核盘表次序：
//!   次序对调、多一块、少一块的盘表要出路，在预演之前拒成盘表那一种、
//!   一个盘都不读。报不带会话身份：别的会话实例代号与 txg 相同的报
//!   要出路时不拦（攻方「打中一」），照实写在 `WaysOutOfTheWideProtectionRefused` 的文档里，这里不钉。
//! - 判别力补的两样：推环做得成、推完之后走空照旧被拒的那一形（正推 K3-4 的 k = 4，注入级）只列卸载再挂，
//!   预演的拒绝与推完真再发同一个 unlink 的拒绝逐字相同（`walkSkipped` 红）；四种改动被宽护卡住时报里记的这次改动逐项相同
//!   （正推 K3-3 的 7 种改坏红；读回时换种类那一种与物理块 4096 的盘由 `singlefs-core` 的库单测钉）。
// 池（384 槽那一档小盘）与 R 那块盘都经 `tests/common_unwitnessed_publish/mod.rs` 起（它转出 `tests/common_admission_pool/mod.rs` 的全部）：
// 再声明 `common_admission` 就把 `common_admission_pool` 当模块装两遍（clippy `duplicate_mod`）。
#[path = "../common_images_for_the_given_states_engine/mod.rs"]
mod common_images_for_the_given_states_engine;
#[path = "../common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;
#[path = "../common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs"]
mod scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table;

use common_images_for_the_given_states_engine::PoolCheckerImages;
use common_unwitnessed_publish::{
    content_of, device_bytes_of_the_384_slot_small_devices, devices_reading_back_all_zero_on,
    image_of_devices_reading_back_all_zero, parameters_of_the_384_slot_small_devices, plain_device,
    plain_devices_on, pool_after_the_mount, record_one_publish_after_overwrites,
    root_is_in_the_ring, DeviceOverASparseImage, PoolUnderTest, RangesReadingBackAllZero,
    RecordedPublish, RecordedPublishKind, OVERWRITE_BYTES,
};
use scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::{
    account_writes_of, assert_none_of_the_placements_is_free,
    assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring,
    assert_the_root_ring_slot_is_unreadable,
    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_the_recorded_publish_and_the_released_slots,
    data_unit_placements_the_recorded_publish_shares_with_the_previous_version,
    devices_with_the_read_fault_on, free_slots_covered_by,
    image_after_raising_the_floor_to_the_txg_of_the_abandoned_root, isolated_slots_covered_by,
    isolated_slots_covered_by_the_ring_slots_holding_placements, isolated_slots_of,
    isolated_slots_on_each_device_of, isolated_slots_released_between, lineage_unit_writes_of,
    mounted_while_the_abandoned_root_reads, packed_unit_writes_of_type,
    pool_crashed_after_the_first_record_of_a_long_sequential_write,
    pool_failing_one_changing_call_copied_from,
    pool_whose_newest_root_was_abandoned_by_a_crash_recovery, push_the_root_ring_of,
    record_one_inode_creation_after_overwrites, record_one_rollback_after_overwrites,
    record_one_rollback_to_the_first_file_after_overwrites,
    record_the_first_publish_of_one_deletion_after_overwrites,
    refusal_of_the_same_change_after_pushing_the_root_ring_on_a_copy_of,
    released_above_the_floor_and_covered_by,
    released_by_the_push_if_the_change_is_admitted_after_it,
    released_in_the_account_of_the_selected_version,
    slots_counted_both_as_allocated_and_as_isolated, slots_covered_by,
    stuck_behind_the_wide_protection_reported_by, unmount_expecting_the_floor_raised, ways_out_of,
    DeviceWithReadFaultsOnChosenSlots, PoolWithAnAbandonedRoot, ReadFaultOnChosenSlots,
    RecordedRollback, WriteStuckBehindTheWideProtection,
    DATA_UNITS_A_WRITE_STILL_REFUSED_AFTER_THE_PUSH_GROWS_THE_FILE_BY,
    DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
    OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_THE_RECORDED_PUBLISH,
    OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_THE_RECORDED_PUBLISH,
    OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
    OVERWRITES_BEFORE_THE_ROLLBACK_WHOSE_PLACEMENTS_A_FLOOR_RAISE_RECLAIMS,
    OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH, VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration, SlotNumber,
};
use singlefs_core::admission::{
    DeviceWithEmptyClusterSegmentsBelowTheFloor, EmptyClusterSegmentsBelowTheFloorOnSomeDevices,
    SpaceAdmission,
};
use singlefs_core::allocator::{AllocationRecord, PlacementOnDevice, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_space_admission, mount_writable_with_test_only_switches,
    raise_rollback_floor, ring_slots_known_to_hold_a_root_by,
    ring_slots_whose_reread_still_bad_refuses_the_ceiling_by, rollback_floor_ceiling, unmount,
    FloorRaiseStop, MountError, RollbackTarget, ShadowLedger, StillUnreadableAfterOneReread,
    Unmounted,
};
use singlefs_core::mounted_session::{
    refusal_is_short_of_space, MountedSession, NoSpaceAfterRaisingTheFloor,
    NoSpaceEvenWithEverythingAFloorRaiseCouldRelease, RootRingPushRefused, RootRingPushRehearsal,
    StuckBehindTheWideProtection, UserChange, UserChangeRefused, WaysOutOfTheWideProtection,
    WhatIsLeftAfterThePush, WideProtectionAtTheCeiling,
};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, instance_table_chain_of_root,
    read_root_ring_slot, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{target_for_publish, RootRingSlot};
use singlefs_core::transaction::{FirstFile, PoolVersion, FIRST_INODE_NUMBER};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
use std::collections::BTreeMap;
use std::sync::Arc;

mod abandoned_root_whose_account_or_ring_slot_turns_unreadable;
mod defer_slots_an_abandoned_root_references;
mod long_sequential_write_stuck_behind_the_wide_protection;
mod pushing_the_root_ring_refused_or_writing_nothing;
mod revived_units_isolated_when_the_floor_raise_reclaims_them;
mod ring_slots_holding_placements;
mod root_abandoned_by_the_mounts_own_row;
mod stuck_behind_the_wide_protection_and_pushing_the_root_ring;
mod units_a_rollback_revived;
mod ways_out_refused_on_a_moved_session_or_a_changed_device_table;
mod what_remains_of_a_change_stuck_behind_the_wide_protection;
mod wide_protection_at_the_mount;

/// 每块盘上挑几个这一刻空闲的槽（单元区里槽号最小的那几个）置隔离位（注入级），交回这几个槽（一槽跨度一个）。
fn isolate_free_slots(
    allocator: &mut PoolAllocator,
    free_slots_on_each_device: u64,
) -> Vec<PlacementOnDevice> {
    let mut placements: Vec<PlacementOnDevice> = Vec::new();
    for device_map in &mut allocator.devices {
        let unit_area_start = device_map.unit_area_start().slot().0;
        let free_slots: Vec<SlotNumber> = (unit_area_start
            ..unit_area_start + device_map.unit_area_slots())
            .map(SlotNumber)
            .filter(|slot| device_map.is_free(*slot))
            .take(usize::try_from(free_slots_on_each_device).expect("几个槽"))
            .collect();
        assert_eq!(
            u64::try_from(free_slots.len()).expect("几个槽"),
            free_slots_on_each_device,
            "盘 {} 上有这么多个空闲的槽（阳性对照）",
            device_map.device.0
        );
        for slot in free_slots {
            device_map.isolate(slot, 1);
            placements.push(PlacementOnDevice {
                device: device_map.device,
                slot,
                span: 1,
            });
        }
    }
    placements
}
