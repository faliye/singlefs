//! 抬 F 那一串（准入与卸载）里的被抛弃根照挂载时记在根环表里的落点算影子账，不再从盘上重读它的账，读不出也不拒
//! （D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，D23（journal 的角色与格式） 已定项 14 回退段「抬 F 那一串（准入与卸载）不走这一道」，
//! 用户 2026-10-07 定；判决 `research/prompts/m3-batch10y-code-r1-main-verification.md` 第二节 K4-2 与第三节）。「崩了再可写挂载」那一道照 C694 的 B 拒，不变。
//!
//! 造法照攻方 `research/prompts/m3-batch10y-code-r1-opus-model/opus_10y_k4_probe.rs` 的 `abandoned_start`（384 槽小盘、之前覆盖写 1 次）：
//! 实例 1 第一个文件、覆盖写一次；崩了再挂（实例 2）覆盖写一次得 C；把 C 的根槽与数据单元抹成全零、每块盘的系统配置轮换退回 C 之前，
//! 崩了再挂（实例 3）择根落到 C 之前那一条、写行与暖机；再把抹掉的字节写回、进程退出。C 留在根环里，按实例 3 的表被抛弃。
//! 之后可写挂载一次（实例 4）：挂载那一刻 C 的账读得出，根环表里把 C 记成被抛弃根、带着它引用的落点。挂上之后 C 的账那几个单元
//! （树表、中央映射树根、分配记录树根的每条位置条目）从此读不出（这几块盘上的读报错）。
//! 改之前抬 F 那一遍照盘上重读 C 的账，读不出、从映射现算不成、重读一次仍不成、现行账里又没有 B 的凭据，就拒：正常卸载与会话写都被拒。
//! 抬 F 之后断言 C 记着的落点一槽都不空闲（隔离着，或现行账里还分配着）：记着的落点被削（只交一部分或交空）时，此前靠 C 之前那一版豁免、
//! 现行账里已释放的那几个（C 之前那一版的实例表片这类）回收出去，这一条红。
//!
//! 同一个池上另三形（用户 2026-10-08 定，判决 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第三节）：
//! - H1「被抛弃根的槽不拒」：挂上之后 C 的**根槽**读报错（账读得出），抬 F 那一串（准入算上限、抬 F 重算影子账，卸载同一串）不拒，
//!   照根环表里记着的 C 的落点进影子账、不进上限；有效根的槽读坏、重读仍坏照拒。
//! - H2「用 C 那一圈记着的落点」：可写挂载自己写行之后才判成被抛弃的根 R（`tests/common_unwitnessed_publish/mod.rs` 那块盘，挂载那一刻
//!   R 的账之外写出的单元读回全零、择根退到 P，R 记成有效根），挂上之后 R 的账读报错：抬 F 照 C 那一圈在 R 那一槽记着的落点算、不读账、不拒。
//! - 挂载那一刻凭据放行的被抛弃根，根环表里记着的是空串：抬 F 照空串算、不再读它的账（攻方 M7）。
//! - H2 之上的丙（用户 2026-10-08 定「改成丙」，判决 `research/prompts/m3-batch10z-d-code-r1-main-verification.md` 第三节）：R 换成建一个 inode
//!   （与所选那一版 P 共用文件的数据单元），挂载那一刻 R 的账读回全零而记录齐，C 那一槽另并上那一刻现行账里仍分配着的落点；
//!   会话覆盖写释放了那几个共用的数据单元、推抬 F、再顺序写，池级 checker 不红 I-7.4。只有 H2、没有丙时这一形红（攻方 K2-2）。
//! - 公开的抬 F 上限（`rollback_floor_ceiling`）照抬 F 那一串的口径取拒集（正推 K1-1）：C 的根槽读坏不拒、不进上限。
//! - 丙补洞两形（用户 2026-10-08 定「丙继续补洞」，判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 两行、第四节；
//!   两处补法的形由第 10z 批第十块实现员定，被攻过零轮）：
//!   丙并中间——R 与所选那一版 P 之间还有一次发布 Q（攻方一形），C 里 R 那一槽并上 Q 那一槽记着的落点，会话写到写掉 R 那一槽，
//!   每一步根落之前那一态与每一步之后池级 checker 不红 I-7.4；
//!   回退复活——R 是一次管理员回退（正推一形），C 里 R 那一槽并上回退目标账里仍分配、P 的账里已释放的落点，推过抬 F 复活的槽不空闲、不红 I-7.4。
//! - 认不出时宽护（tri，用户 2026-10-08 弹窗定，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第五节；细节第 10z 批第十二块实现员定，
//!   被攻过零轮）：回退复活那一形再让 R 写的谱系片、R 写的全部单元、R_old 的根槽、R_old 的账之一读回全零，C 里 R 那一槽退到现行账里全部已释放的落点，
//!   写到推过抬 F 不红 I-7.4；认得出回退时只多 R_old 账里仍分配、现行账里已释放的那一截；R 不是回退时一槽不多、挂载那一刻隔离数与之前相同。
//! - 被宽护卡住的报与推环入口（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」；第 10z 批第十六块实现员定形，被攻过零轮）：384 槽盘上
//!   第 10z 批第十四块报告第六节那几形卡住时，`NoSpaceAfterRaisingTheFloor` 点名「被宽护卡住」、带 R 的 txg、挡着的槽数与两条出路；
//!   调推环入口之后 R 那一槽写掉、隔离回 0、之后写放得下；卸载重挂那一条出路也走得通；推到半路取不到落点时在任何写之前拒。
//! - 被抛弃根那一圈遇到 defer 槽不双扣（第十四块报告第七节第 2 条）：挂载那一刻与根槽读坏时的抬 F 都只隔离分配位没置着的槽，回收时照根环表补。
//! - 出路按预演定（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」，判决 `research/prompts/m3-batch10z-pqrs-code-r1-main-verification.md`
//!   第二、三节；第 10z 批第二十块实现员定形，被攻过零轮）：攻方 K1 那一形（`research/prompts/m3-batch10z-pqrs-code-r1-opus-model/atk_part.rs`）
//!   推完放得下的写列两条出路、照推环走完放得下，推完放不下的写（含没有 C 那一圈也被拒的长度、实现员三形长 60 与 80 个单元）只列卸载再挂、
//!   照它走完放得下；出路一按 R 那一槽有没有读得出的有效根分两种说法，出路二写推几次、写掉几槽、放出来的隔离槽数（推完前后之差，两槽 C 重叠时不重数）；
//!   推环入口冻结着时第一道就拒、不核盘表时盘体对调照推（攻方 M1、M2）、两槽 C 推到较晚的那一槽（攻方 M12）；C 记着落点而一槽都没隔离、
//!   停因是预算用完或抬 F 取不到落点时不报被宽护卡住，记着空串的槽不算（正推 K4-2 活下来的几个）；根槽读坏的被抛弃根那一圈补满整段跨度。
//! - 要出路先核盘表次序（用户 2026-10-09 弹窗定「现在补第二十四块再攻一轮」，判决 `research/prompts/m3-batch10z-vw-code-r1-main-verification.md`
//!   第二节 K1-3、第三节；第 10z 批第二十四块实现员定形，被攻过零轮）：次序对调、多一块、少一块的盘表要出路，在预演之前拒成盘表那一种、
//!   一个盘都不读（攻方「打中二」）。报不带会话身份（用户 2026-10-09 弹窗定「撤掉身份，文档照实写」）：别的会话实例代号与 txg 相同的报
//!   要出路时不拦（攻方「打中一」），照实写在 `WaysOutOfTheWideProtectionRefused` 的文档里，这里不钉。
//! - 判别力补的两样（判决第二节 K1-2、K3）：推环做得成、推完之后走空照旧被拒的那一形（正推 K3-4 的 k = 4，注入级）只列卸载再挂，
//!   预演的拒绝与推完真再发同一个 unlink 的拒绝逐字相同（`walkSkipped` 红）；四种改动被宽护卡住时报里记的这次改动逐项相同
//!   （正推 K3-3 的 7 种改坏红；读回时换种类那一种与物理块 4096 的盘由 `singlefs-core` 的库单测钉）。
// 池（384 槽那一档小盘）与 R 那块盘都经 `tests/common_unwitnessed_publish/mod.rs` 起（它转出 `tests/common_admission_pool/mod.rs` 的全部）：
// 再声明 `common_admission` 就把 `common_admission_pool` 当模块装两遍（clippy `duplicate_mod`）。
mod common_images_for_the_checker_tier;
mod common_unwitnessed_publish;
#[path = "common_images_for_the_checker_tier/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs"]
mod scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table;

use common_images_for_the_checker_tier::PoolCheckerImages;
use common_unwitnessed_publish::{
    content_of, device_bytes_of_the_384_slot_small_devices, devices_reading_back_all_zero_on,
    image_of_devices_reading_back_all_zero, parameters_of_the_384_slot_small_devices, plain_device,
    plain_devices_on, pool_after_the_mount, record_one_publish_after_overwrites,
    root_is_in_the_ring, DeviceOverASparseImage, PoolUnderTest, PublishRecordedAsR,
    RangesReadingBackAllZero, RecordedPublish, OVERWRITE_BYTES,
};
use scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::{
    account_writes_of, assert_none_of_the_placements_is_free,
    assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring,
    assert_the_root_ring_slot_is_unreadable,
    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots,
    data_unit_placements_r_shares_with_the_previous_version, devices_with_the_read_fault_on,
    free_slots_covered_by, image_after_raising_the_floor_to_the_txg_of_the_abandoned_root,
    isolated_slots_covered_by, isolated_slots_covered_by_the_ring_slots_holding_placements,
    isolated_slots_of, isolated_slots_on_each_device_of, isolated_slots_released_between,
    lineage_unit_writes_of, mounted_while_the_abandoned_root_reads, packed_unit_writes_of_type,
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
    OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_R,
    OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R, OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
    OVERWRITES_BEFORE_THE_ROLLBACK_WHOSE_PLACEMENTS_A_FLOOR_RAISE_RECLAIMS,
    OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH, VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration, SlotNumber,
};
use singlefs_core::admission::SpaceAdmission;
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
    MountedSession, NoSpaceAfterRaisingTheFloor, RootRingPushRefused, RootRingPushRehearsal,
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
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use std::collections::BTreeMap;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements,
    );
}

/// 「崩了再可写挂载」那一道照 C694 的 B 拒，不变：同一份盘面上 C 的账读不出、现行账里没有罩住它树表槽的凭据，可写挂载在取号之前拒、
/// 点名 C，两块盘逐字节不变。
#[test]
fn writable_mount_after_a_crash_still_refuses_while_the_account_of_an_abandoned_root_is_unreadable_without_the_credential(
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault = ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root);
    fault.is_active.set(true);
    let mut devices = devices_with_the_read_fault_on(&built.image, &fault);
    let refused = mount_writable_with_space_admission(
        &built.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    );
    let expected_abandoned_root = RollbackTarget {
        instance: built.abandoned_root.instance,
        checkpoint_txg: built.abandoned_root.checkpoint_txg,
    };
    match refused {
        Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) => {
            assert!(
                matches!(
                    *still_unreadable,
                    StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { abandoned_root }
                        if abandoned_root == expected_abandoned_root
                ),
                "拒因点名 C 的账：{still_unreadable:?}"
            );
        }
        Err(other) => panic!("拒因是 C 的账读不出，而是：{other:?}"),
        Ok(_) => panic!("C 的账读不出、没有凭据：崩了再可写挂载照拒，而挂上了"),
    }
    let image_after_the_refusal = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.sparse_image().clone()))
            .collect(),
        device_size_in_bytes: built.device_bytes,
    };
    assert!(
        image_after_the_refusal == built.image,
        "在取号与任何写之前拒：两块盘逐字节不变"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements,
    );
}

/// 公开的抬 F 上限（`rollback_floor_ceiling`），读坏、重读仍坏就拒的槽取 `ring_slots`：系统配置、现行那一版的实例表与 F 都照这一刻的盘与会话现读。
fn public_rollback_floor_ceiling_of(
    pool: &PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
    ring_slots: &std::collections::BTreeSet<RootRingSlot>,
) -> Result<singlefs_core::address::CheckpointTxg, MountError> {
    let system_configuration = choose_system_configuration(&pool.devices).expect("系统配置读得出");
    let table = instance_table_chain_of_root(&pool.devices, &pool.current_file_version().root)
        .expect("现行那一版的实例表读得出")
        .records;
    rollback_floor_ceiling(
        &pool.devices,
        &system_configuration,
        pool.rollback_floor(),
        &table,
        ring_slots,
    )
}

/// K1-1（正推 `research/prompts/m3-batch10z-d-code-r1-sonnet-output.md` K1-1 末段公开上限函数那一形）：挂上之后 C 的根槽读报错。自算上限的调用方
/// （checker 档 `raise-rollback-floor` 模式、e158 装置）照抬 F 那一串的口径取读坏重读仍坏就拒的槽
/// （`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`：除去根环表记成被抛弃根的槽），公开的 `rollback_floor_ceiling` 不拒，
/// C 那一槽当没有根、不进上限：算出的上限与故障关着时相同。阳性对照：同一刻照旧口径传「知道住着根的槽」整集就拒、点名 C 那一槽
/// （`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，改之前 checker 档那两处调用方就是这样）。
#[test]
fn the_public_rollback_floor_ceiling_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_equals_the_ceiling_while_it_reads(
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault =
        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let (pool, _recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
    let ring_slots_whose_reread_still_bad_refuses_the_ceiling =
        ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(&pool.session().allocator);
    let ceiling_while_the_slot_reads = public_rollback_floor_ceiling_of(
        &pool,
        &ring_slots_whose_reread_still_bad_refuses_the_ceiling,
    )
    .expect("故障关着：上限算得出");
    fault.is_active.set(true);
    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
    let ring_slot_of_the_abandoned_root = target_for_publish(
        built.abandoned_root.checkpoint_txg,
        built.parameters.geometry.root_ring_slots_per_region,
    );
    match public_rollback_floor_ceiling_of(
        &pool,
        &ring_slots_known_to_hold_a_root_by(&pool.session().allocator),
    ) {
        Err(MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
            ring_slot,
            ..
        }) => assert_eq!(
            ring_slot, ring_slot_of_the_abandoned_root,
            "旧口径（知道住着根的槽整集）拒、点名 C 那一槽"
        ),
        other => panic!("阳性对照：旧口径下 C 的根槽读坏重读仍坏就拒，而是：{other:?}"),
    }
    match public_rollback_floor_ceiling_of(
        &pool,
        &ring_slots_whose_reread_still_bad_refuses_the_ceiling,
    ) {
        Ok(ceiling) => assert_eq!(
            ceiling, ceiling_while_the_slot_reads,
            "被抛弃根的槽读坏当没有根、不进上限：上限与故障关着时相同"
        ),
        Err(refusal) => {
            panic!("被抛弃根的槽读坏不拒（与抬 F 那一串同一口径），而拒了：{refusal:?}")
        }
    }
}

/// H1 的另一半：有效根的槽照「这个进程知道住着根的槽」那一句，读坏重读仍坏就拒。挂上之后这次挂载择到的那条有效根（实例 3 退出时现行那一版，
/// 根环表记成有效根）的根槽读报错：正常卸载拒成 `NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot)`、点名那一槽，
/// 在任何写之前，两块盘逐字节不变。
#[test]
fn unmount_after_the_root_ring_slot_of_a_valid_root_turns_unreadable_is_still_refused_before_any_write(
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault = ReadFaultOnChosenSlots::on_the_root_ring_slot_of(
        &built.newest_valid_root,
        &built.parameters,
    );
    let (mut pool, _recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
    fault.is_active.set(true);
    assert_the_root_ring_slot_is_unreadable(&pool, &built.newest_valid_root);
    let expected_ring_slot: RootRingSlot = target_for_publish(
        built.newest_valid_root.checkpoint_txg,
        built.parameters.geometry.root_ring_slots_per_region,
    );
    let image_before_the_unmount = pool.image();
    let mut session = pool.session.take().expect("挂着的会话");
    let refused = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    match refused {
        Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) => {
            assert!(
                matches!(
                    *still_unreadable,
                    StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { ring_slot, .. }
                        if ring_slot == expected_ring_slot
                ),
                "拒因点名那条有效根的根槽 {expected_ring_slot:?}：{still_unreadable:?}"
            );
        }
        Err(other) => panic!("拒因是有效根的根槽重读仍坏，而是：{other:?}"),
        Ok(_) => panic!("有效根的根槽读坏、重读仍坏：卸载照拒，而做成了"),
    }
    assert!(
        pool.image() == image_before_the_unmount,
        "在任何写之前拒：两块盘逐字节不变"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot,
    );
}

/// 凭据放行那一形里，挂载之后至多再覆盖写几次去等 R 的树表槽与映射根槽都被重新分配（384 槽小盘一圈根环 24 次发布之内）。
const OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST: u64 = 24;

/// 现行账里有一条分配代大于 `root` 的 txg、罩住这一槽的记录（C694 的 B 那道凭据的一条位置条目）。
fn a_later_record_covers(
    records: &[AllocationRecord],
    root: &RootRecord,
    location: &LocationEntry,
) -> bool {
    records.iter().any(|record| {
        record.device == location.device
            && record.slot.0 <= location.slot.0
            && location.slot.0 < record.slot.0 + u64::from(record.span_slots)
            && record.generation > root.checkpoint_txg
    })
}

/// 攻方 M7（`research/prompts/m3-batch10z-b-code-r1-opus-output.md` 第三节）：挂载那一刻凭据放行的被抛弃根，根环表里记着的是空串，
/// 抬 F 那一串照空串算、不再读它的账。造法同 `abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs`
/// 「已被占的那一形」：M1 里 R 的根槽与单元读回全零、影子账关着，之后覆盖写到 R 的树表槽与映射根槽都被重新分配；再可写挂载一次（M2）：
/// R 的根槽读得出、按实例表被抛弃，账与映射都读不出、现行账里有凭据——放行，根环表把 R 记成被抛弃根、记着空串，挂载计数 1。
/// M2 正常卸载：照记着的空串算，不读 R 的账（计数 0），做成。照读盘时这一遍又数到 R（计数 1）。
#[test]
fn unmount_takes_the_empty_placements_recorded_for_an_abandoned_root_the_credential_passed_at_mount_without_reading_its_account(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH,
        PublishRecordedAsR::Overwrite,
    );
    let root_of_r = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes());
    let mut devices_of_the_first_mount = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let first_mount = mount_writable_with_test_only_switches(
        &recorded.pool.parameters,
        &mut devices_of_the_first_mount,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::Off,
    )
    .expect("R 的根槽与单元读回全零：可写挂载落到 P、做得成");
    let image_after_the_first_mount = image_of_devices_reading_back_all_zero(
        &devices_of_the_first_mount,
        recorded.pool.device_bytes,
    );
    let mut pool = pool_after_the_mount(&recorded, &image_after_the_first_mount, first_mount);
    let account_slots_of_r: Vec<LocationEntry> = root_of_r
        .tree_table
        .locations
        .iter()
        .chain(root_of_r.mapping_root.locations.iter())
        .copied()
        .collect();
    let mut overwrites_after_the_first_mount = 0;
    // 迭代上界 OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST；每一轮看一次现行账，不够再覆盖写一次。
    while !account_slots_of_r.iter().all(|location| {
        a_later_record_covers(
            &pool.current_file_version().allocation_records,
            &root_of_r,
            location,
        )
    }) {
        assert!(
            overwrites_after_the_first_mount < OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST,
            "{OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST} 次覆盖写之内 R 的树表槽与映射根槽都被重新分配"
        );
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上这一串覆盖写都落得下");
        overwrites_after_the_first_mount += 1;
    }
    let image = pool.image();
    let mut devices_of_the_second_mount = plain_devices_on(&image);
    let second_mount = mount_writable(&recorded.pool.parameters, &mut devices_of_the_second_mount)
        .expect("R 的账读不出、现行账里有凭据：再可写挂载做得成");
    assert_eq!(
        second_mount.output.abandoned_roots_unreadable, 1,
        "再可写挂载那一遍读不出 R 的账、凭据放行，计数往上报"
    );
    assert_eq!(
        second_mount
            .allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .placements_recorded_for_the_abandoned_root(
                root_of_r.instance,
                root_of_r.checkpoint_txg
            ),
        Some(Vec::new()),
        "凭据放行：根环表把 R 记成被抛弃根，记着的落点是空串"
    );
    let (_, mut session) = MountedSession::of_the_mount(second_mount);
    let PoolVersion::WithFile(_) = &session.current else {
        panic!("覆盖写之后现行那一版带文件");
    };
    let unmounted = unmount(
        &recorded.pool.parameters,
        &mut devices_of_the_second_mount,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    match unmounted {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => assert_eq!(
            raised.abandoned_roots_unreadable, 0,
            "R 记着的是空串：照它算，这一遍不再读 R 的账"
        ),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => panic!("R 记着空串：卸载做得成，而被拒：{refusal:?}"),
    }
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
// harness-test-granularity:one-scenario 一个池、一次挂载，循环是同一个会话里接着覆盖写，不是每轮新建一个池
#[test]
fn units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount,
    );
}

/// 认得出回退时只多那一截（正推 K5 的 E3、E1，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第二节 K5 那一行）：
/// 回退复活那一形、只有 R 的账读回全零（谱系片与 R_old 都读得出）。挂载那一刻 C 里 R 那一槽里、在挂载那一刻现行账里记成已释放的落点，
/// 逐个就是 R_old 账里仍分配着、现行账里已释放的那一截（按盘与起点槽），一个不多、一个不少。认出回退就并现行账里全部已释放的（E3，
/// 正推的对照臂）、回退目标按所有有效根找（E1）都会多出别的已释放落点。期望从录下来的会话算（R_old 那一刻的会话分配器），不经被测的方法。
#[test]
fn a_rollback_recognized_at_the_mount_takes_among_the_released_only_what_its_target_still_allocated(
) {
    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
    );
    let recorded = &recorded_rollback.recorded;
    let root_of_r = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(recorded)),
    );
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
    );
    let released_in_the_current_account = released_in_the_account_of_the_selected_version(&mounted);
    let borrowed_for_r = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
    let released_borrowed_for_r: std::collections::BTreeSet<(DeviceIdentity, SlotNumber)> =
        borrowed_for_r
            .iter()
            .map(|placement| (placement.device, placement.slot))
            .filter(|placement| released_in_the_current_account.contains(placement))
            .collect();
    let still_allocated_by_the_target_and_released: std::collections::BTreeSet<(
        DeviceIdentity,
        SlotNumber,
    )> = recorded_rollback
        .allocated_in_the_account_of_the_rollback_target
        .iter()
        .copied()
        .filter(|placement| released_in_the_current_account.contains(placement))
        .collect();
    assert!(
        recorded_rollback.revived.iter().all(|revived| {
            still_allocated_by_the_target_and_released.contains(&(revived.device, revived.slot))
        }),
        "复活的落点都在 R_old 账里仍分配、现行账里已释放的那一截里（阳性对照：期望那一截罩着复活集）"
    );
    assert_eq!(
        released_borrowed_for_r, still_allocated_by_the_target_and_released,
        "C 里 R 那一槽里现行账已释放的落点，就是 R_old 账里仍分配、现行账里已释放的那一截（窄取法：不多并现行账里别的已释放落点）"
    );
}

/// tri 不碰的那一支（正推 K2-3 第 1 条「读得出、确实没有这条根的谱系记录（肯定不是回退）」）：R 建一个 inode、挂载那一刻只有 R 的账读回全零，
/// R 的记录点名的打包单元（inode 叶容器）都读得出、里面没有谱系记录。C 里 R 那一槽里一个现行账已释放的落点都没有（丙只并仍分配着的），
/// 挂载那一刻两块盘隔离的槽数与 tri 之前逐格相同：18（R 写的单元，两块盘各 9 槽；正推对照臂这一形 116，
/// 量法副本 `SINGLEFS_PROBE_ARM=narrow`（tri 之前）与 tri 两臂挂载那一刻同为 18，第 10z 批第十二块实现员报告第三节）。
#[test]
fn a_root_taking_the_current_account_that_is_not_a_rollback_takes_nothing_released_and_isolates_as_before_at_the_mount(
) {
    let recorded = record_one_inode_creation_after_overwrites(
        OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_R,
    );
    let root_of_r = recorded.root();
    assert!(
        lineage_unit_writes_of(&recorded).is_empty(),
        "建 inode 不写谱系片（阳性对照：R 不是回退）"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded)),
    );
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
    );
    let released_in_the_current_account = released_in_the_account_of_the_selected_version(&mounted);
    assert!(
        !released_in_the_current_account.is_empty(),
        "挂载那一刻现行账里有已释放的落点（阳性对照：并多了看得出来）"
    );
    let released_borrowed_for_r: Vec<PlacementOnDevice> = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）")
        .into_iter()
        .filter(|placement| {
            released_in_the_current_account.contains(&(placement.device, placement.slot))
        })
        .collect();
    assert!(
        released_borrowed_for_r.is_empty(),
        "R 不是回退：C 里 R 那一槽一个现行账里已释放的落点都不并，多并的 {released_borrowed_for_r:?}"
    );
    let isolated_slots_at_the_mount: u64 = mounted
        .allocator
        .devices
        .iter()
        .map(singlefs_core::allocator::DeviceFreeMap::isolated_slots)
        .sum();
    assert_eq!(
        isolated_slots_at_the_mount,
        ISOLATED_SLOTS_AT_THE_MOUNT_WHEN_A_ROOT_TAKING_THE_CURRENT_ACCOUNT_IS_NOT_A_ROLLBACK,
        "挂载那一刻两块盘隔离的槽数与 tri 之前相同（D28（挂载期承诺量） 已定项 1 第九项）"
    );
}

/// [`a_root_taking_the_current_account_that_is_not_a_rollback_takes_nothing_released_and_isolates_as_before_at_the_mount`] 那一形挂载那一刻两块盘隔离的槽数（tri 之前量的）。
const ISOLATED_SLOTS_AT_THE_MOUNT_WHEN_A_ROOT_TAKING_THE_CURRENT_ACCOUNT_IS_NOT_A_ROLLBACK: u64 =
    18;

/// H2 双扣（判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第二节 K1-3、第三节第十四块第 1 条；正推第 1.3 节 384 槽盘 720 槽那一形）：
/// 覆盖写 40 次、回退到倒数第 7 版录 R，可写挂载那一刻 R 的账与谱系片读回全零：认不出是不是回退，C 里 R 那一槽宽护、并进现行账里全部已释放的落点。
/// 其中还在 defer 窗口里的（分配位置着）算在「已分配」里，D28（挂载期承诺量） 已定项 1 第九项「不在这里再算一次」：挂载那一刻一个槽都不同时
/// 置着分配位与隔离位，C 里 R 那一槽的每个槽也都不空闲（分配着或隔离着，恰好一样）。改之前两块盘 104 个槽两项各扣一次（隔离 720 槽）。
/// 这一形去掉双扣之后写照旧全被拒（隔离 616 槽，第 10z 批第十四块实现员报告「H1 重量」），不在这里钉，交主 agent。
#[test]
fn wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices(
) {
    let recorded = record_one_rollback_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
        VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
    )
    .recorded;
    let lineage_unit_writes = lineage_unit_writes_of(&recorded);
    assert!(
        !lineage_unit_writes.is_empty(),
        "回退那次发布写了谱系片（阳性对照：故障有处可打）"
    );
    let root_of_r = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default()
            .with_the_writes(&account_writes_of(&recorded))
            .with_the_writes(&lineage_unit_writes),
    );
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
    );
    let released_in_the_current_account = released_in_the_account_of_the_selected_version(&mounted);
    let borrowed_for_r = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
    let released_not_borrowed: Vec<&(DeviceIdentity, SlotNumber)> = released_in_the_current_account
        .iter()
        .filter(|(device, slot)| {
            !borrowed_for_r
                .iter()
                .any(|borrowed| borrowed.device == *device && borrowed.slot == *slot)
        })
        .collect();
    assert!(
        released_not_borrowed.is_empty(),
        "认不出是不是回退：C 里 R 那一槽并进现行账里全部已释放的落点（阳性对照：宽护走到了），缺 {released_not_borrowed:?}"
    );
    assert!(
        isolated_slots_of(&mounted.allocator) > 0,
        "挂载那一刻隔离着 C 里分配位没置着的槽（阳性对照）"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
        0,
        "挂载那一刻没有一个槽同时算进「已分配」与第九项（D28（挂载期承诺量） 已定项 1 第九项：现行账里仍分配着的、已释放而还在 defer 窗口里的不在这里再算一次）"
    );
    let free_among_the_borrowed = free_slots_covered_by(&mounted.allocator, &borrowed_for_r);
    assert!(
        free_among_the_borrowed.is_empty(),
        "C 里 R 那一槽的每个槽挂载那一刻都不空闲（分配着或隔离着）：空闲的（盘、槽）{free_among_the_borrowed:?}"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：1536 槽两块盘、40 步每步跑池级 checker，debug 下单条过 60 秒"]
fn wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices,
    );
}

/// R 每条记录（按本次发布内序号）点名的打包单元（类标签 3）各是哪一种打包记录类型，按点名的次序：从录下来的记录写与单元写的字节解，
/// 不经被测的方法。点名的单元 R 没写（照抄上一版的）记 `None`。
fn packed_record_types_named_by_each_record_of(
    recorded: &RecordedPublish,
) -> Vec<Vec<Option<u16>>> {
    let filesystem_identifier = singlefs_core::unit::unit_filesystem_identifier(
        &recorded.pool.parameters.filesystem_identifier,
    );
    (1..=recorded.record_count())
        .map(|ordinal_from_one| {
            let record_bytes = recorded.record_writes_of_the_record(ordinal_from_one)[0]
                .bytes()
                .expect("记录写带字节");
            let record =
                singlefs_core::journal::JournalRecord::parse(record_bytes, filesystem_identifier)
                    .expect("R 录下来的记录解得开");
            record
                .named
                .iter()
                .filter(|named| named.unit_class == singlefs_core::unit::UNIT_CLASS_PACKED)
                .map(|named| {
                    recorded
                        .unit_writes()
                        .into_iter()
                        .find(|write| {
                            write.device == named.locations[0].device
                                && write.offset == named.locations[0].slot.to_device_offset()
                        })
                        .and_then(|write| write.bytes())
                        .and_then(|bytes| singlefs_core::unit::parse_packed_unit(bytes).ok())
                        .map(|packed| packed.identity.record_type)
                })
                .collect()
        })
        .collect()
}

/// R 走丙、认不出是不是回退那一形的共同判法（tri 第一种）：可写挂载那一刻 `ranges` 读回全零，择根退到 P；C 里 R 那一槽里在现行账里记成已释放的落点
/// 就是现行账里全部已释放的（宽护：一个不缺，按盘与起点槽）。
fn assert_the_root_taking_the_current_account_takes_every_released_placement_at_the_mount(
    recorded: &RecordedPublish,
    ranges: &RangesReadingBackAllZero,
) {
    let root_of_r = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(&crash_image, ranges);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
    );
    let released_in_the_current_account = released_in_the_account_of_the_selected_version(&mounted);
    assert!(
        !released_in_the_current_account.is_empty(),
        "挂载那一刻现行账里有已释放的落点（阳性对照：并少了看得出来）"
    );
    let released_borrowed_for_r: std::collections::BTreeSet<(DeviceIdentity, SlotNumber)> = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）")
        .into_iter()
        .map(|placement| (placement.device, placement.slot))
        .filter(|placement| released_in_the_current_account.contains(placement))
        .collect();
    assert_eq!(
        released_borrowed_for_r, released_in_the_current_account,
        "R 的记录点名的打包单元有一片读不出、认不出是不是回退：C 里 R 那一槽并进现行账里全部已释放的落点（tri 宽护）"
    );
}

/// 删除那一形之前覆盖写几次。
const OVERWRITES_BEFORE_THE_DELETION_RECORDED_AS_R: u64 = 3;

/// tri 第一种的删除那一形（正推 `research/prompts/m3-batch10z-lm-code-r1-sonnet-output.md` 第 2 节「删除发布墓碑片读不出 → 宽护没有入库用例」，
/// 探针 `faults=racct,rtomb`）：R 是一次删除的第一次发布，记录点名它写的 inode 叶容器与墓碑片（叶容器排在前面）；挂载那一刻 R 的账与墓碑片
/// 读回全零：认不出是不是回退，C 里 R 那一槽并进现行账里全部已释放的落点。读盘一层只看第一个打包单元（正推 X5）时只看到读得出的叶容器、
/// 认成肯定不是回退，这一句红。
#[test]
fn a_deletion_whose_tombstone_unit_and_account_read_back_zero_at_the_mount_takes_every_released_placement(
) {
    let recorded = record_the_first_publish_of_one_deletion_after_overwrites(
        OVERWRITES_BEFORE_THE_DELETION_RECORDED_AS_R,
    );
    let tombstone_unit_writes =
        packed_unit_writes_of_type(&recorded, singlefs_core::unit::PACKED_TYPE_TOMBSTONE);
    assert!(
        !tombstone_unit_writes.is_empty(),
        "删除的第一次发布写了墓碑片（阳性对照：故障有处可打）"
    );
    let named_in_order: Vec<Option<u16>> = packed_record_types_named_by_each_record_of(&recorded)
        .into_iter()
        .flatten()
        .collect();
    let tombstone_position = named_in_order
        .iter()
        .position(|record_type| *record_type == Some(singlefs_core::unit::PACKED_TYPE_TOMBSTONE))
        .expect("R 的记录点名墓碑片");
    assert!(
        tombstone_position > 0,
        "R 的记录在墓碑片之前还点名了别的打包单元（这一形分得出只看第一个打包单元）：{named_in_order:?}"
    );
    assert_the_root_taking_the_current_account_takes_every_released_placement_at_the_mount(
        &recorded,
        &RangesReadingBackAllZero::default()
            .with_the_writes(&account_writes_of(&recorded))
            .with_the_writes(&tombstone_unit_writes),
    );
}

/// 顺序写那一形之前覆盖写几次。
const OVERWRITES_BEFORE_THE_SEQUENTIAL_WRITE_RECORDED_AS_R: u64 = 3;

/// tri 第一种的多条记录那一形：R 顺序写三个数据单元（三条记录，前两条只点名数据单元，inode 叶容器只在末条点名）；挂载那一刻 R 的账与它写的
/// inode 叶容器读回全零：认不出是不是回退，C 里 R 那一槽并进现行账里全部已释放的落点。读盘一层只看第一条记录点名的打包单元（正推 X4）时
/// 一个打包单元都看不到、认成肯定不是回退，这一句红。
#[test]
fn a_sequential_write_whose_inode_unit_named_by_its_last_record_and_account_read_back_zero_at_the_mount_takes_every_released_placement(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_SEQUENTIAL_WRITE_RECORDED_AS_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    let named_by_each_record = packed_record_types_named_by_each_record_of(&recorded);
    assert!(
        named_by_each_record.len() >= 2 && named_by_each_record[0].is_empty(),
        "R 有两条以上记录、第一条不点名打包单元（这一形分得出只看第一条记录）：{named_by_each_record:?}"
    );
    let inode_unit_writes =
        packed_unit_writes_of_type(&recorded, singlefs_core::unit::PACKED_TYPE_INODE);
    assert!(
        !inode_unit_writes.is_empty(),
        "R 写了 inode 叶容器（阳性对照：故障有处可打）"
    );
    assert_the_root_taking_the_current_account_takes_every_released_placement_at_the_mount(
        &recorded,
        &RangesReadingBackAllZero::default()
            .with_the_writes(&account_writes_of(&recorded))
            .with_the_writes(&inode_unit_writes),
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero,
    );
}

/// 抬 F 那一串卸载那一格当场放开那一截回收时照样补隔离（判决 `research/prompts/m3-batch10z-no-code-r1-main-verification.md`
/// 第二节 K2 ①；形照攻方 `research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-u1.txt`：384 槽、覆盖写 10 次、回退到倒数第 7 版录 R、
/// 挂载那一刻 R 的根槽与谱系片读回全零而挂上之后照旧、挂上就卸载）。与 [`revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero`]
/// 同一个池、同一次挂载，只把挂上之后那一串抬 F 换成正常卸载：R 复活的数据单元在 P 的账里已释放、还在 defer 窗口里，挂载那一刻不隔离
/// （D28（挂载期承诺量） 已定项 1 第九项不再算一次），卸载那一串回收它们；R 的根槽照旧读回全零，这一串不把 R 当被抛弃根，护着它们的只有 C 那一圈。
/// 卸载之后 R 还在根环里：C 那一圈罩着的槽一个都不空闲，复活的那几个隔离着，没有一个槽同时算进「已分配」与第九项。
/// 当场放开那一截回收之后不补隔离（改回 `PoolAllocator::reclaim_released_up_to`）时，复活的槽卸载之后空闲，这一条红。扣到生效那一截改回只回收，
/// 这一条照绿（第 10z 批第十七块实现员证红时量的），那一截由
/// [`unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held_until_the_floor_takes_effect_while_the_account_of_r_reads_back_zero`] 钉。
#[test]
fn unmount_right_after_the_mount_isolates_the_revived_units_it_reclaims_while_the_root_slot_of_the_rollback_reads_back_zero(
) {
    let RecordedRollback {
        recorded, revived, ..
    } = record_one_rollback_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_WHOSE_PLACEMENTS_A_FLOOR_RAISE_RECLAIMS,
        VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
    );
    assert!(
        !revived.is_empty(),
        "回退目标的数据单元 P 不引用：回退复活了它们（阳性对照：这一形走到了复活）"
    );
    let lineage_unit_writes = lineage_unit_writes_of(&recorded);
    assert!(
        !lineage_unit_writes.is_empty(),
        "回退那次发布写了谱系片（阳性对照：故障有处可打）"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default()
            .with_the_root_slot_of(&crash_image, recorded.key)
            .with_the_writes(&lineage_unit_writes),
    );
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: None,
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: recorded.content.clone(),
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    let mounted = mount_writable_with_space_admission(
        &pool.parameters,
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的根槽与谱系片读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的根槽读不出、系统配置没见证 R、记录不施加：择根落到 P（阳性对照：故障打到了）"
    );
    let named_by_the_unapplied_records_at_the_mount = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records();
    let revived_not_in_c: Vec<&PlacementOnDevice> = revived
        .iter()
        .filter(|revived_placement| {
            !named_by_the_unapplied_records_at_the_mount
                .iter()
                .any(|placement| {
                    placement.device == revived_placement.device
                        && placement.slot == revived_placement.slot
                })
        })
        .collect();
    assert!(
        revived_not_in_c.is_empty(),
        "C 那一圈并进了 R 的账（记录的新根段读得出），R 复活的数据单元都在里面（阳性对照），缺 {revived_not_in_c:?}"
    );
    // R 的根槽与谱系片挂上之后照旧读回全零，到这个进程写掉那几段为止：卸载那一串读不出 R，不把它当被抛弃根算影子账。
    let (_output, mut session) = MountedSession::of_the_mount(mounted);
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    let reclaimed_by_the_unmount = match unmounted {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => raised.reclaimed,
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("挂载落到的 P 带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => panic!("挂上就卸载做得成：{refusal:?}"),
    };
    assert!(
        revived.iter().any(|revived_placement| {
            reclaimed_by_the_unmount
                .iter()
                .any(|reclaimed| reclaimed.slot == revived_placement.slot)
        }),
        "卸载那一串回收了复活的数据单元（阳性对照：卸载那一格回收时补隔离那一处走到了），回收的 {reclaimed_by_the_unmount:?}"
    );
    assert!(
        root_is_in_the_ring(&pool.image(), recorded.key),
        "卸载之后 R 还在根环里（卸载那一串没写到 R 那一槽）"
    );
    let named_by_the_unapplied_records_after_the_unmount = session
        .allocator
        .root_ring_occupancy()
        .expect("卸载之后分配器照旧装着根环表")
        .placements_named_by_unapplied_journal_records();
    let free_among_the_named = free_slots_covered_by(
        &session.allocator,
        &named_by_the_unapplied_records_after_the_unmount,
    );
    assert!(
        free_among_the_named.is_empty(),
        "卸载之后 R 还在根环里：C 那一圈罩着的槽一个都不空闲（卸载那一串回收时照 C 那一圈补了隔离），空闲的（盘、槽）{free_among_the_named:?}"
    );
    let revived_slots_not_isolated: Vec<(DeviceIdentity, SlotNumber)> = revived
        .iter()
        .flat_map(|revived_placement| {
            let device_map = session
                .allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == revived_placement.device)
                .expect("池里有这块盘");
            (revived_placement.slot.0..revived_placement.slot.0 + revived_placement.span)
                .map(SlotNumber)
                .filter(move |slot| !device_map.is_isolated(*slot))
                .map(move |slot| (revived_placement.device, slot))
        })
        .collect();
    assert!(
        revived_slots_not_isolated.is_empty(),
        "卸载之后复活的数据单元（P 的账里已释放、卸载那一串回收了）覆盖的槽都隔离着，没隔离的（盘、槽）{revived_slots_not_isolated:?}"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&session.allocator),
        0,
        "卸载之后没有一个槽同时算进「已分配」与第九项"
    );
}

/// 镜像根环里每块盘上读得出、自证过的根（`except` 那一条除外）各取 txg 最大的一条，再在各盘之间取最小：卸载那一格当场放开那一截
/// 放到哪儿（`mount::newest_valid_root_txg_on_every_device`）在这一形上的值，从盘上独立读、不经被测的方法。`except` 是按现行那一版的
/// 实例表被抛弃的那一条（这一形里只有 R）；有一块盘上一条根都读不出就判红（这一形里每块盘上都有这次挂载落的根）。
fn smallest_among_the_newest_root_txg_on_each_device(
    image: &MemoryPool,
    except: (InstanceGeneration, CheckpointTxg),
) -> CheckpointTxg {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let immutable = &configuration.immutable;
    let mut newest_on_each_device: BTreeMap<DeviceIdentity, CheckpointTxg> = BTreeMap::new();
    // 迭代上界是根环槽数（3 区 × 每区槽数）；跨轮携带的是每块盘上到这一槽为止读到的最大 txg。
    for ring_slot in every_root_ring_slot(&immutable.sizes) {
        let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        match read_root_ring_slot(
            image,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(root)
                if (root.instance, root.checkpoint_txg) != except =>
            {
                let newest = newest_on_each_device
                    .entry(device)
                    .or_insert(root.checkpoint_txg);
                *newest = (*newest).max(root.checkpoint_txg);
            }
            RootRingSlotReading::SelfVerified(_) | RootRingSlotReading::Bad(_) => {}
        }
    }
    let devices_of_the_pool: std::collections::BTreeSet<DeviceIdentity> =
        immutable.region_devices.iter().copied().collect();
    assert_eq!(
        newest_on_each_device
            .keys()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        devices_of_the_pool,
        "每块盘上都读得出一条不被抛弃的根（这次挂载的暖机在每块盘上都落了根）"
    );
    newest_on_each_device
        .into_values()
        .min()
        .expect("池里至少一块盘")
}

/// 抬 F 那一串卸载那一格扣到生效那一截回收时照样补隔离（判决 `research/prompts/m3-batch10z-no-code-r1-main-verification.md` 第二节 K2 ②；
/// 形照第 10z 批第十七块实现员报告「停下交主 agent」推的方向，第 10z 批第十九块实现员定形，被攻过零轮）。池与挂载同丙那一形
/// （[`the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor`]：
/// 覆盖写 3 次得 P、录一次建 inode 得 R，崩在 R 的根槽落盘之后、轮换之前，R 的账读回全零、挂上之后照旧）：择根退到 P，C 里 R 那一槽并上
/// 挂载那一刻现行账里仍分配着的落点（R 与 P 共用的数据单元），它们分配位置着、挂载那一刻不隔离（D28（挂载期承诺量） 已定项 1 第九项不再算一次）。
/// 挂上之后覆盖写一次：会话自己释放这几个数据单元，释放代是这次覆盖写的 txg，高于各盘最新根取最小（这一 txg 的根只落在一块盘上），
/// 卸载那一串回收它们落在扣到生效那一截、不在当场放开那一截。卸载之后 R 还在根环里：它们隔离着、一槽都不空闲，C 那一圈罩着的槽一个都不空闲，
/// 没有一个槽同时算进「已分配」与第九项（第九项「这次挂载里释放、回收时分配器照补隔离」，I-7.4（近 K 代块未被复用））。
/// 扣到生效那一截回收之后不补隔离（改回 `PoolAllocator::reclaim_released_up_to`）时，卸载末尾放开扣住位之后它们空闲，这一条红。
#[test]
fn unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held_until_the_floor_takes_effect_while_the_account_of_r_reads_back_zero(
) {
    let recorded = record_one_inode_creation_after_overwrites(
        OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_R,
    );
    let root_of_r = recorded.root();
    let shared_with_the_previous_version =
        data_unit_placements_r_shares_with_the_previous_version(&recorded);
    assert!(
        !shared_with_the_previous_version.is_empty(),
        "R 建一个 inode、不写文件的数据单元：它与 P 共用文件的数据单元（阳性对照：有要护的槽）"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded)),
    );
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的账读回全零：R 走读不成、择根退到 P（阳性对照：故障打到了）"
    );
    let borrowed_at_the_mount = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点");
    let shared_not_borrowed: Vec<&PlacementOnDevice> = shared_with_the_previous_version
        .iter()
        .filter(|shared| {
            !borrowed_at_the_mount
                .iter()
                .any(|borrowed| borrowed.device == shared.device && borrowed.slot == shared.slot)
        })
        .collect();
    assert!(
        shared_not_borrowed.is_empty(),
        "挂载那一刻 C 里 R 那一槽并上了现行账里仍分配着的落点（丙）：R 与 P 共用的数据单元都在里面，缺 {shared_not_borrowed:?}"
    );
    let shared_free_at_the_mount =
        free_slots_covered_by(&mounted.allocator, &shared_with_the_previous_version);
    assert!(
        shared_free_at_the_mount.is_empty(),
        "挂载那一刻 R 与 P 共用的数据单元在现行账里仍分配着：一槽都不空闲，空闲的（盘、槽）{shared_free_at_the_mount:?}"
    );
    let shared_isolated_at_the_mount =
        isolated_slots_covered_by(&mounted.allocator, &shared_with_the_previous_version);
    assert!(
        shared_isolated_at_the_mount.is_empty(),
        "挂载那一刻 C 那一圈不隔离分配位置着的槽（阳性对照：卸载之后的隔离是回收那一刻补的），隔离着的（盘、槽）{shared_isolated_at_the_mount:?}"
    );
    let (output, session_of_the_mount) = MountedSession::of_the_mount(mounted);
    let content_of_the_selected_version = recorded
        .pool
        .content_of_each_root
        .get(&(
            output.effective_root.instance,
            output.effective_root.checkpoint_txg,
        ))
        .cloned()
        .expect("挂载落到的那一版是录的池写过的一版");
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session_of_the_mount),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_of_the_selected_version,
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    let published = pool
        .overwrite(OVERWRITE_BYTES)
        .expect("挂上之后第一次覆盖写落得下");
    assert_eq!(
        published.floor_raises.len(),
        0,
        "挂上之后第一次覆盖写准入够、不推抬 F（共用的数据单元留给卸载那一串回收）"
    );
    let txg_of_the_overwrite = pool.session().current.root().checkpoint_txg;
    let shared_not_released_by_the_overwrite: Vec<&PlacementOnDevice> =
        shared_with_the_previous_version
            .iter()
            .filter(|shared| {
                !pool.session().allocator.records().iter().any(|record| {
                    record.device == shared.device
                        && record.slot == shared.slot
                        && record.is_released
                        && record.generation == txg_of_the_overwrite
                })
            })
            .collect();
    assert!(
        shared_not_released_by_the_overwrite.is_empty(),
        "这次覆盖写释放了 R 与 P 共用的数据单元、释放代是它的 txg {txg_of_the_overwrite:?}（阳性对照：释放是这次挂载里会话自己做的），没这样记着的 {shared_not_released_by_the_overwrite:?}"
    );
    let released_immediately_up_to =
        smallest_among_the_newest_root_txg_on_each_device(&pool.image(), recorded.key);
    assert!(
        txg_of_the_overwrite > released_immediately_up_to,
        "释放代 {txg_of_the_overwrite:?} 高于各盘最新根取最小 {released_immediately_up_to:?}：卸载那一串回收它们落在扣到生效那一截（阳性对照：走到要钉的那一截，不在当场放开那一截）"
    );
    let mut session = pool.session.take().expect("挂着的会话");
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    let raised = match unmounted {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => raised,
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => panic!("挂上之后覆盖写一次再卸载做得成：{refusal:?}"),
    };
    assert_eq!(
        raised.rollback_floor, txg_of_the_overwrite,
        "卸载把 F 抬到现行那一版（这次覆盖写）的 txg：回收门槛够得着它释放的槽"
    );
    let shared_not_reclaimed: Vec<&PlacementOnDevice> = shared_with_the_previous_version
        .iter()
        .filter(|shared| {
            !raised
                .reclaimed
                .iter()
                .any(|reclaimed| reclaimed.slot == shared.slot)
        })
        .collect();
    assert!(
        shared_not_reclaimed.is_empty(),
        "卸载那一串回收了这次覆盖写释放的共用数据单元（阳性对照：回收时补隔离那一处走到了），没回收的 {shared_not_reclaimed:?}，回收的 {:?}",
        raised.reclaimed
    );
    assert!(
        root_is_in_the_ring(&pool.image(), recorded.key),
        "卸载之后 R 还在根环里（卸载那一串没写到 R 那一槽）"
    );
    let shared_free_after_the_unmount =
        free_slots_covered_by(&session.allocator, &shared_with_the_previous_version);
    assert!(
        shared_free_after_the_unmount.is_empty(),
        "卸载之后 R 还在根环里：扣到生效那一截回收的共用数据单元覆盖的槽一槽都不空闲（回收时照 C 那一圈补了隔离，卸载末尾放开扣住位之后照旧挡着），空闲的（盘、槽）{shared_free_after_the_unmount:?}"
    );
    assert_eq!(
        isolated_slots_covered_by(&session.allocator, &shared_with_the_previous_version),
        slots_covered_by(&shared_with_the_previous_version),
        "卸载之后共用数据单元覆盖的槽都隔离着"
    );
    let named_by_the_unapplied_records_after_the_unmount = session
        .allocator
        .root_ring_occupancy()
        .expect("卸载之后分配器照旧装着根环表")
        .placements_named_by_unapplied_journal_records();
    let free_among_the_named = free_slots_covered_by(
        &session.allocator,
        &named_by_the_unapplied_records_after_the_unmount,
    );
    assert!(
        free_among_the_named.is_empty(),
        "卸载之后 R 还在根环里：C 那一圈罩着的槽一个都不空闲，空闲的（盘、槽）{free_among_the_named:?}"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&session.allocator),
        0,
        "卸载之后没有一个槽同时算进「已分配」与第九项"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way,
    );
}

/// 推满仍不够、F 已在上限而没有 C 那一圈（崩在静止点之后再挂，水位之上没有没施加的记录）：照旧只报空间不够、不报「被宽护卡住」；
/// 这时调推环入口也不推（C 那一圈没有记着落点的槽），两块盘逐字节不变。池照 `admission_raises_the_floor_before_refusing.rs` 里
/// 「截断之后同样大小写回」那一条起（单元区 256 槽两块盘，崩了再挂、覆盖写 3 次、顺序写把文件长到会话报空间不够）。
fn pool_stuck_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written() -> (
    PoolUnderTest<SparseBlockDevice>,
    NoSpaceAfterRaisingTheFloor,
) {
    let device_width = singlefs_harness::history::HistoryDeviceWidth::UnitAreaOf256Slots;
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        device_width.parameters(),
        device_width.device_bytes(),
        plain_device,
    );
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂");
    for overwrite_number in 1..=3 {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {overwrite_number} 次覆盖写：{refusal:?}"));
    }
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let mut units = 0;
    // 迭代上界 100 个单元；跨轮只带长到的单元数；出口：被拒。
    let refusal = loop {
        match pool.sequential_write((units + 1) * unit_bytes) {
            Ok(_) => units += 1,
            Err(refusal) => break refusal,
        }
        assert!(units < 100, "256 槽的盘长不到 100 个单元");
    };
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("长大被拒是推满仍不够：{refusal:?}");
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
        ),
        "推抬 F 停在上限（阳性对照：与被宽护卡住同一种停法）：{:?}",
        no_space.stop
    );
    assert!(
        pool.session()
            .allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records()
            .is_empty(),
        "崩在静止点之后再挂：C 那一圈一槽都没有"
    );
    (pool, *no_space)
}

#[test]
fn no_space_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written_is_not_reported_as_the_wide_protection(
) {
    let (_pool, no_space) =
        pool_stuck_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written();
    assert!(
        matches!(
            no_space.wide_protection_at_the_ceiling,
            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission
        ),
        "没有 C 那一圈：照旧只报空间不够，不报被宽护卡住：{:?}",
        no_space.wide_protection_at_the_ceiling
    );
}

#[test]
fn pushing_the_root_ring_with_nothing_held_until_a_ring_slot_is_written_writes_nothing() {
    let (mut pool, _no_space) =
        pool_stuck_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written();
    let image_before = pool.image();
    let mut session = pool.session.take().expect("挂着的会话");
    let pushed = session
        .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(
            &mut pool.devices,
        );
    pool.session = Some(session);
    assert!(
        matches!(
            pushed,
            Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot)
        ),
        "C 那一圈没有记着落点的槽：不推，而交回 {pushed:?}"
    );
    assert!(pool.image() == image_before, "不推：两块盘逐字节不变");
}

/// 现行那一版树表 0 条（mkfs 之后可写挂载、还没写文件）：推环入口在任何读写之前拒成第一版不支持的那一种，两块盘逐字节不变。
#[test]
fn pushing_the_root_ring_on_a_version_without_file_is_refused_as_not_supported_in_the_first_version(
) {
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    singlefs_core::make_filesystem::make_filesystem(&parameters, &mut devices)
        .expect("小盘上 mkfs 做得成");
    let mounted = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("mkfs 之后可写挂载做得成");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    assert!(
        matches!(session.current, PoolVersion::WithoutFile(_)),
        "还没写文件：现行那一版树表 0 条（阳性对照）"
    );
    let image_of = |devices_of_the_pool: &[(DeviceIdentity, SparseBlockDevice)]| MemoryPool {
        devices: devices_of_the_pool
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    };
    let image_before = image_of(&devices);
    let pushed = session
        .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(
            &mut devices,
        );
    assert!(
        matches!(
            pushed,
            Err(RootRingPushRefused::VersionWithoutFileNotSupportedInTheFirstVersion)
        ),
        "树表 0 条：拒成第一版不支持，而交回 {pushed:?}"
    );
    assert!(image_of(&devices) == image_before, "两块盘逐字节不变");
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed,
    );
}

/// 根槽这一遍读坏的被抛弃根那一圈同样不双扣（派发第十六块第 2 条）：同一份盘面挂上（实例 5，F = C 的 txg）之后 C 的根槽读报错，
/// 把 F 抬到它现在的值（只供测试强制进入的那一档，F 不变）：抬 F 那一串读根环时 C 那一槽读坏、照根环表里记着的落点进影子账
/// （D16（发布语义） 已定项 1「根槽这一次读坏」，H1），那几个 defer 槽释放代高于 F、这一串不回收：抬完之后没有一个槽同时算进「已分配」与第九项，
/// C 引用的槽一个都不空闲。这一圈照旧整段隔离时抬完那一句红。
#[test]
fn defer_slots_an_abandoned_root_whose_ring_slot_reads_bad_references_are_not_counted_twice_by_a_floor_raise(
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let image = image_after_raising_the_floor_to_the_txg_of_the_abandoned_root(&built);
    let fault =
        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let (mut pool, recorded_for_c) = mounted_while_the_abandoned_root_reads(
        &PoolWithAnAbandonedRoot {
            parameters: built.parameters.clone(),
            device_bytes: built.device_bytes,
            image,
            write_time_seconds: built.write_time_seconds + 3000,
            abandoned_root: built.abandoned_root,
            newest_valid_root: built.newest_valid_root,
        },
        &fault,
    );
    let floor = pool.rollback_floor();
    assert_eq!(
        floor, built.abandoned_root.checkpoint_txg,
        "挂载那一刻 F 是 C 的 txg（阳性对照）"
    );
    let defer_slots_of_c =
        released_above_the_floor_and_covered_by(&pool.session().allocator, &recorded_for_c, floor);
    assert!(
        !defer_slots_of_c.is_empty(),
        "C 引用的槽里有现行账已释放、释放代高于 F 的（阳性对照）"
    );
    fault.is_active.set(true);
    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
    let mut session = pool.session.take().expect("挂着的会话");
    let raised = {
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("挂载落到的那一版带文件");
        };
        raise_rollback_floor(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            floor,
            ShadowLedger::On,
        )
    };
    pool.session = Some(session);
    let raised =
        raised.unwrap_or_else(|refusal| panic!("C 的根槽读坏、F 不变的抬 F 做得成：{refusal:?}"));
    assert_eq!(
        raised.abandoned_roots_unreadable, 0,
        "C 的根槽读坏：照根环表里记着的落点算，不读它的账"
    );
    let allocator = &pool.session().allocator;
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(allocator),
        0,
        "抬完之后没有一个槽同时算进「已分配」与第九项"
    );
    let still_allocated_defer_slots: Vec<&(DeviceIdentity, SlotNumber)> = defer_slots_of_c
        .iter()
        .filter(|(device, slot)| {
            allocator.devices.iter().any(|device_map| {
                device_map.device == *device
                    && !device_map.is_free(*slot)
                    && !device_map.is_isolated(*slot)
            })
        })
        .collect();
    assert!(
        !still_allocated_defer_slots.is_empty(),
        "抬完之后这几个 defer 槽还分配着、这一串没回收（阳性对照：这一圈遇到了分配位置着的槽）"
    );
    assert_none_of_the_placements_is_free(allocator, &recorded_for_c, "C 记着");
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both,
    );
}

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

/// 每块盘上挑几个这一刻空闲的槽、置隔离位，记成 `ring_slot` 那一槽 C 记着的落点（一槽跨度一个），并进 C 那一圈原来记着的那几槽。
/// 注入级：可写挂载自己造不出这一份，用例拿它摆出 C 那一圈多一槽的形。
fn hold_isolated_free_slots_on_a_further_ring_slot(
    allocator: &mut PoolAllocator,
    ring_slot: RootRingSlot,
    free_slots_on_each_device: u64,
) {
    let placements = isolate_free_slots(allocator, free_slots_on_each_device);
    let mut holding = allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .ring_slots_holding_placements_named_by_unapplied_journal_records();
    assert!(
        !holding.contains_key(&ring_slot),
        "注入的那一槽原来没记着落点（阳性对照）"
    );
    holding.insert(ring_slot, placements);
    allocator.note_placements_named_by_unapplied_journal_records(holding);
}

/// 两槽 C、落点重叠时出路二说放出来的隔离槽数（判决第二节「K1-1 三个量」正推注入级那一形：重叠时放出来的数偏大）：攻方 K1 那一形挂上之后
/// 长 13 个单元被拒一次（会话推一串抬 F、把 F 抬到上限），在 R 那一槽之前先轮到的一槽（现行那一版之后第一次发布写的那一槽）注入 C 的第二槽，记着 R 那一槽一半的落点（与 R 那一槽重叠）
/// 加每块盘几个隔离着的空闲槽；另在每块盘隔离一个 C 那一圈不罩的空闲槽。再写同一个长度报「被宽护卡住」、推完放得下：
/// 报的挡着的槽数只数 C 罩着的（攻方 M7：取全部隔离时多数那几个），出路二说放出来的槽数等于它（重叠的不重数、C 不罩的不算），
/// 照推环走完就是推完前后每块盘隔离数之差，同一个写放得下。
#[test]
fn the_slots_the_way_out_by_the_push_releases_with_two_ring_slots_holding_overlapping_placements_are_what_the_push_releases(
) {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
    );
    let slots_per_region = pool.parameters.geometry.root_ring_slots_per_region;
    let ring_slot_written_first = target_for_publish(
        CheckpointTxg(pool.session().current.root().checkpoint_txg.0 + 1),
        slots_per_region,
    );
    {
        let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
        let holding = allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records();
        let (ring_slot_of_r, placements_of_r) =
            holding.iter().next().expect("C 那一圈只有 R 那一槽");
        assert_ne!(
            *ring_slot_of_r, ring_slot_written_first,
            "R 那一槽不是推环第一次就写的那一槽（阳性对照：注入的那一槽先轮到）"
        );
        let overlapping_with_r: Vec<PlacementOnDevice> =
            placements_of_r.iter().step_by(2).copied().collect();
        hold_isolated_free_slots_on_a_further_ring_slot(allocator, ring_slot_written_first, 2);
        let mut two_ring_slots = allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records();
        two_ring_slots
            .get_mut(&ring_slot_written_first)
            .expect("刚注入的那一槽")
            .extend(overlapping_with_r);
        allocator.note_placements_named_by_unapplied_journal_records(two_ring_slots);
        isolate_free_slots(allocator, 1);
    }
    assert!(
        isolated_slots_covered_by_the_ring_slots_holding_placements(&pool.session().allocator)
            < isolated_slots_of(&pool.session().allocator),
        "每块盘另有一个隔离着、C 那一圈不罩的槽（阳性对照：推环放不出它，挡着的槽数与放出来的槽数都不算它）"
    );
    let length = crashed
        .bytes_of_the_file_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY);
    let refusal = pool.sequential_write(length).expect_err("两槽 C 挡着准入");
    let WriteStuckBehindTheWideProtection { stuck, .. } =
        stuck_behind_the_wide_protection_reported_by(refusal, "两槽 C、落点重叠");
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, "两槽 C、落点重叠");
    let released_in_the_rehearsal =
        released_by_the_push_if_the_change_is_admitted_after_it(&pushing_the_root_ring)
            .unwrap_or_else(|| {
                panic!(
                    "两槽 C：预演推完之后这次改动放得下，而预演的结局是 {pushing_the_root_ring:?}"
                )
            });
    assert_eq!(
        released_in_the_rehearsal, stuck.isolated_slots_on_each_device,
        "根环表里没有被抛弃根：推环放出来的就是 C 那一圈罩着的、隔离着的槽，两槽重叠的那几个只数一次"
    );
    assert_eq!(ways_out.len(), 2, "推完放得下：两条出路都列：{ways_out:?}");
    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
        &stuck,
        &ways_out[1],
        &released_in_the_rehearsal,
        WhatIsLeftAfterThePush::PublishTheChangeAgain,
        "两槽 C、落点重叠",
    );
    let isolated_before_the_push = isolated_slots_on_each_device_of(&pool.session().allocator);
    let pushed = push_the_root_ring_of(&mut pool)
        .unwrap_or_else(|push_refusal| panic!("两槽 C 推环做得成：{push_refusal:?}"));
    assert_eq!(
        pushed.ring_slots_written.len(),
        2,
        "推环写掉 C 那一圈的两槽：{:?}",
        pushed.ring_slots_written
    );
    assert_eq!(
        released_in_the_rehearsal,
        isolated_slots_released_between(
            &isolated_before_the_push,
            &isolated_slots_on_each_device_of(&pool.session().allocator),
        ),
        "出路二说放出来的隔离槽数就是推完前后每块盘隔离数之差"
    );
    pool.sequential_write(length)
        .unwrap_or_else(|refusal_after_the_push| {
            panic!("推完之后同一个写放得下：{refusal_after_the_push:?}")
        });
}

/// 不再推的原因是一次准入的发布数做满时不报「被宽护卡住」（正推 `budgetAsStuck`）：攻方 K1 那一形长 20 个单元被宽护挡着，推环之后再试，
/// 会话推两串抬 F 就碰到一次准入 8 次发布的预算（攻方 `k1-trace`）；在现行那一版那一槽（满一环才轮到）注入 C 一槽、每块盘隔离两个空闲槽，
/// 再试：照旧停在预算上，C 那一圈这一刻隔离着槽，报的照旧只是空间不够。
#[test]
fn no_space_when_the_publishes_of_one_admission_run_out_while_the_ring_slots_hold_isolated_placements_is_not_reported_as_the_wide_protection(
) {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted();
    let length = crashed.bytes_of_the_file_grown_by(
        DATA_UNITS_A_WRITE_STILL_REFUSED_AFTER_THE_PUSH_GROWS_THE_FILE_BY,
    );
    pool.sequential_write(length)
        .expect_err("长 20 个单元：宽护挡着准入");
    push_the_root_ring_of(&mut pool).unwrap_or_else(|refusal| panic!("推环做得成：{refusal:?}"));
    let ring_slot_of_the_current_version = target_for_publish(
        pool.session().current.root().checkpoint_txg,
        pool.parameters.geometry.root_ring_slots_per_region,
    );
    hold_isolated_free_slots_on_a_further_ring_slot(
        &mut pool.session.as_mut().expect("挂着的会话").allocator,
        ring_slot_of_the_current_version,
        2,
    );
    let refusal = pool
        .sequential_write(length)
        .expect_err("推完之后长 20 个单元照旧被拒");
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("被拒是推满仍不够：{refusal:?}");
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }
        ),
        "不再推的原因是一次准入的发布数做满（阳性对照）：{:?}",
        no_space.stop
    );
    assert!(
        isolated_slots_covered_by_the_ring_slots_holding_placements(&pool.session().allocator) > 0,
        "这一刻 C 那一圈还隔离着槽（阳性对照：只差停因）"
    );
    assert!(
        matches!(
            no_space.wide_protection_at_the_ceiling,
            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission
        ),
        "停因是预算用完：不报被宽护卡住，而报 {:?}",
        no_space.wide_protection_at_the_ceiling
    );
}

/// 挂上之后先覆盖写几次：之后有 4 个以上不同的非空状态，准入上限高过 F，被拒时会话真去推抬 F。
const OVERWRITES_BEFORE_THE_FLOOR_RAISE_FINDS_NO_PLACEMENT: usize = 5;

/// 不再推的原因是抬 F 那一串自己也取不到落点时不报「被宽护卡住」（正推 `spaceAsStuck`）：攻方 K1 那一形挂上之后覆盖写几次，
/// 再把每块盘没挡的槽全隔离掉（注入级），覆盖写一次：准入拒、推抬 F，抬 F 那一串在预演里取不到落点；C 那一圈这一刻隔离着槽，
/// 报的照旧只是空间不够。
#[test]
fn no_space_when_the_floor_raise_finds_no_placement_while_the_ring_slots_hold_isolated_placements_is_not_reported_as_the_wide_protection(
) {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted();
    for overwrite_number in 1..=OVERWRITES_BEFORE_THE_FLOOR_RAISE_FINDS_NO_PLACEMENT {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("挂上之后第 {overwrite_number} 次覆盖写落得下：{refusal:?}")
        });
    }
    for device_map in &mut pool.session.as_mut().expect("挂着的会话").allocator.devices {
        let unit_area_start = device_map.unit_area_start().slot().0;
        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
                device_map.isolate(SlotNumber(slot), 1);
            }
        }
        assert_eq!(
            device_map.unblocked_slots(),
            0,
            "盘 {} 上没挡的槽全隔离掉了（阳性对照）",
            device_map.device.0
        );
    }
    let refusal = pool
        .overwrite(OVERWRITE_BYTES)
        .expect_err("没挡的槽全隔离掉：覆盖写被拒");
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("被拒是推满仍不够：{refusal:?}");
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorRaiseRefusedForSpace { .. }
        ),
        "不再推的原因是抬 F 那一串自己也取不到落点（阳性对照）：{:?}",
        no_space.stop
    );
    assert!(
        isolated_slots_covered_by_the_ring_slots_holding_placements(&pool.session().allocator) > 0,
        "这一刻 C 那一圈还隔离着槽（阳性对照：只差停因）"
    );
    assert!(
        matches!(
            no_space.wide_protection_at_the_ceiling,
            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission
        ),
        "停因是抬 F 那一串取不到落点：不报被宽护卡住，而报 {:?}",
        no_space.wide_protection_at_the_ceiling
    );
}

/// 根槽读坏的被抛弃根那一圈照记着的整段跨度补隔离（正推 K4-1：`crates/mutations.tsv` 里「第10z批第九块 K3 loop-span1」那一行改之后不红——
/// 挂载那一刻 C 记着的落点两槽都已隔离着，抬 F 那一圈补的那一遍什么都不改）：挂上之后，C 记着的一个跨两槽、两槽都隔离着的落点，
/// 把后一槽的隔离位清掉（注入级：后一槽分配位没置着、空闲），再让 C 的根槽读报错、正常卸载：抬 F 那一串照根环表里记着的落点补隔离，
/// 后一槽也隔离着。只隔起点那一槽时后一槽照旧空闲、发得出去。
#[test]
fn unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_isolates_the_later_slot_of_its_recorded_two_slot_placement_left_unisolated(
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault =
        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let (mut pool, recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
    let (device, later_slot) = {
        let allocator = &pool.session().allocator;
        recorded
            .iter()
            .filter(|placement| placement.span >= 2)
            .find_map(|placement| {
                let device_map = allocator
                    .devices
                    .iter()
                    .find(|device_map| device_map.device == placement.device)
                    .expect("池里有这块盘");
                (placement.slot.0..placement.slot.0 + placement.span)
                    .all(|slot| device_map.is_isolated(SlotNumber(slot)))
                    .then_some((
                        placement.device,
                        SlotNumber(placement.slot.0 + placement.span - 1),
                    ))
            })
            .expect("C 记着的落点里有一个跨两槽以上、每一槽都隔离着的（阳性对照）")
    };
    {
        let device_map = pool
            .session
            .as_mut()
            .expect("挂着的会话")
            .allocator
            .devices
            .iter_mut()
            .find(|device_map| device_map.device == device)
            .expect("池里有这块盘");
        device_map.clear_isolation_of_slot(later_slot);
        assert!(
            device_map.is_free(later_slot),
            "后一槽清掉隔离位之后空闲：分配位没置着（阳性对照）"
        );
    }
    fault.is_active.set(true);
    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
    let (_, allocator_after_the_unmount) =
        unmount_expecting_the_floor_raised(&mut pool, "C 的根槽读不出");
    let device_map = allocator_after_the_unmount
        .devices
        .iter()
        .find(|device_map| device_map.device == device)
        .expect("池里有这块盘");
    assert!(
        device_map.is_isolated(later_slot),
        "抬 F 那一串照根环表里记着的落点补隔离、补满整段跨度：后一槽（盘 {}、槽 {}）隔离着",
        device.0,
        later_slot.0
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device,
    );
}

/// 卡在走空里那两形的造法：攻方 K1 那一形挂上之后长 13 个单元被拒一次（会话推一串抬 F、把 F 抬到上限），再把每块盘没挡的槽标成已分配、
/// 只留 [`UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK`] 个（注入级，位图上占着、不写记录；照
/// `admission_raises_the_floor_before_refusing.rs` 的 `pool_ready_to_unlink_with_few_unblocked_slots`）。
/// 实现员副本上的草稿探针（留 0–1000 个 × 先覆盖写 0–20 次）：只有不先覆盖写、留 29–31 个那三格 unlink 卡在意图之后的走空里；
/// 留 0–18 个卡在意图那一次，留 20–28 个推的抬 F 自己落不下、不报被宽护卡住，留 32 个起做成。
const UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK: u64 = 30;

fn pool_whose_unlink_of_the_first_file_is_stuck_in_the_walk_after_the_intent() -> (
    PoolUnderTest<SparseBlockDevice>,
    StuckBehindTheWideProtection,
) {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
    );
    leave_only_unblocked_slots_on_each_device(
        &mut pool.session.as_mut().expect("挂着的会话").allocator,
        UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK,
    );
    pool.write_time_seconds += 1;
    let refusal = pool
        .publish(
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
                write_time_seconds: pool.write_time_seconds,
            },
            None,
        )
        .expect_err("没挡的槽只剩 30 个：unlink 被拒");
    let WriteStuckBehindTheWideProtection {
        versions_published,
        stuck,
        ..
    } = stuck_behind_the_wide_protection_reported_by(refusal, "unlink 意图之后的走空");
    assert_eq!(
        versions_published, 1,
        "意图那一次发成、被拒的是它之后走空的一步（阳性对照：卡在走空里）"
    );
    let current = pool
        .session()
        .current
        .version_of_trees()
        .expect("意图那一版有树");
    assert_eq!(
        current.deleted_inodes,
        vec![InodeNumber(FIRST_INODE_NUMBER)],
        "现行那一版是意图那一版：队列里是第一个文件"
    );
    (pool, stuck)
}

/// 报里记着的「这次改动还剩什么」从 `{:?}` 读（私有字段，只有会话造得出这一份报）。
fn remainder_of_the_change_recorded_in(stuck: &StuckBehindTheWideProtection) -> String {
    let debug = format!("{stuck:?}");
    let start = debug
        .find("remainder_of_the_change: ")
        .expect("报的 `{:?}` 里有这次改动还剩什么那一项");
    debug[start..].to_string()
}

/// 要出路时预演里推环自己被拒（卡在走空里那两形：没挡的槽只剩那么几个，推环那一串第二次就取不到落点）：只列卸载再挂，
/// 预演交回的推环拒绝与真调推环入口的拒绝逐字相同。
fn assert_the_push_is_refused_in_the_rehearsal_as_it_is_for_real(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    stuck: &StuckBehindTheWideProtection,
    why: &str,
) {
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(pool, stuck, why);
    let RootRingPushRehearsal::PushRefused(rehearsed_push_refusal) = pushing_the_root_ring else {
        panic!("{why}：没挡的槽只剩那么几个，预演里推环被拒，而预演交回 {pushing_the_root_ring:?}");
    };
    assert_eq!(ways_out.len(), 1, "{why}：只列卸载再挂：{ways_out:?}");
    let real_push_refusal = push_the_root_ring_of(pool).expect_err("真调推环入口：同样被拒");
    assert_eq!(
        format!("{real_push_refusal:?}"),
        format!("{rehearsed_push_refusal:?}"),
        "{why}：预演里推环的拒绝与真调推环入口的拒绝逐字相同"
    );
}

/// unlink 意图之后的走空被宽护卡住（攻方第五节变异 H：这一处记成「走空再发这次改动」时，推完之后预演会把 unlink 再发一次）：
/// 报里记着的这次改动还剩什么只是走空。预演里推环被拒（造这一形要把没挡的槽压到推环也取不到落点：推完之后那一段这一形走不到，
/// 只钉报里记的那一项与出路）。
#[test]
fn an_unlink_whose_walk_after_the_intent_is_stuck_behind_the_wide_protection_keeps_only_the_walk_as_what_remains_of_the_change(
) {
    let (mut pool, stuck) =
        pool_whose_unlink_of_the_first_file_is_stuck_in_the_walk_after_the_intent();
    let remainder = remainder_of_the_change_recorded_in(&stuck);
    assert!(
        remainder.starts_with("remainder_of_the_change: WalkTheQueue }"),
        "意图已发成、被拒的是它之后走空的一步：报里记着的这次改动还剩什么只是走空，而记的是 {remainder}"
    );
    assert_the_push_is_refused_in_the_rehearsal_as_it_is_for_real(
        &mut pool,
        &stuck,
        "unlink 意图之后的走空",
    );
}

/// 一次改动之前的走空被宽护卡住（攻方第五节变异 F：这一处记成「只剩走空」时，推完之后预演不再发这次改动）：unlink 卡在意图之后的走空里，
/// 再覆盖写一次——先走空那一步就被拒、被宽护卡住；报里记着的这次改动还剩什么是走空再发这次覆盖写。预演里推环被拒（同上一条）。
#[test]
fn an_overwrite_whose_walk_before_it_is_stuck_behind_the_wide_protection_keeps_the_walk_then_the_overwrite_as_what_remains_of_the_change(
) {
    let (mut pool, _stuck_in_the_unlink) =
        pool_whose_unlink_of_the_first_file_is_stuck_in_the_walk_after_the_intent();
    let refusal = pool
        .overwrite(OVERWRITE_BYTES)
        .expect_err("队列里还有第一个文件：覆盖写之前先走空，走空那一步照旧被拒");
    let WriteStuckBehindTheWideProtection {
        versions_published,
        stuck,
        ..
    } = stuck_behind_the_wide_protection_reported_by(refusal, "覆盖写之前的走空");
    assert_eq!(
        versions_published, 0,
        "走空那一步就被拒：一版都没发成（阳性对照：卡在这次改动之前的走空里）"
    );
    let remainder = remainder_of_the_change_recorded_in(&stuck);
    assert!(
        remainder.starts_with(
            "remainder_of_the_change: WalkTheQueueThenPublishTheChange(Overwrite {"
        ),
        "被拒的是这次改动之前走空的一步：报里记着的这次改动还剩什么是走空再发这次覆盖写，而记的是 {}",
        remainder.chars().take(120).collect::<String>()
    );
    assert_the_push_is_refused_in_the_rehearsal_as_it_is_for_real(
        &mut pool,
        &stuck,
        "覆盖写之前的走空",
    );
}

/// 每块盘的单元区里没挡的槽标成已分配、只留 `unblocked_slots_left` 个（注入级，位图上占着、不写记录；照
/// `admission_raises_the_floor_before_refusing.rs` 的 `pool_ready_to_unlink_with_few_unblocked_slots`）。
fn leave_only_unblocked_slots_on_each_device(
    allocator: &mut PoolAllocator,
    unblocked_slots_left: u64,
) {
    for device_map in &mut allocator.devices {
        let unit_area_start = device_map.unit_area_start().slot().0;
        // 迭代上界是单元区槽数；跨轮只带没挡的槽数；出口：只剩下要留的那几个。
        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
            if device_map.unblocked_slots() <= unblocked_slots_left {
                break;
            }
            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device()
{
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_in_reversed_order_is_refused_without_reading_a_device`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pushing_the_root_ring_with_a_device_table_other_than_the_mount_in_reversed_order_is_refused_without_reading_a_device(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_in_reversed_order_is_refused_without_reading_a_device,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_more_is_refused_without_reading_a_device`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_more_is_refused_without_reading_a_device(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_more_is_refused_without_reading_a_device,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_fewer_is_refused_without_reading_a_device`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_fewer_is_refused_without_reading_a_device(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_fewer_is_refused_without_reading_a_device,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_the_session_moved_with_the_two_devices_in_reversed_order_is_refused_as_the_session_moved`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn asking_for_the_ways_out_after_the_session_moved_with_the_two_devices_in_reversed_order_is_refused_as_the_session_moved(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_the_session_moved_with_the_two_devices_in_reversed_order_is_refused_as_the_session_moved,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_unmounting_and_mounting_again_with_the_report_from_before_is_refused_as_the_session_moved_naming_the_new_instance`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn asking_for_the_ways_out_after_unmounting_and_mounting_again_with_the_report_from_before_is_refused_as_the_session_moved_naming_the_new_instance(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::asking_for_the_ways_out_after_unmounting_and_mounting_again_with_the_report_from_before_is_refused_as_the_session_moved_naming_the_new_instance,
    );
}

/// 正推 `research/prompts/m3-batch10z-vw-code-r1-sonnet-model/src/vw/part10_k3.rs` 的 `vw_k3_walk_after_the_push_sweep` 那一形
/// （k = 4）：C 那一圈整个换成从现行那一版起第几次发布写的那一槽。
const PUBLISHES_UNTIL_THE_RING_SLOT_HOLDING_THE_INJECTED_ISOLATION: u64 = 4;

/// 那一槽每块盘记着几个隔离着的空闲槽：推完之后走空照旧被拒那一形。
const ISOLATED_FREE_SLOTS_THE_INJECTED_RING_SLOT_HOLDS_ON_EACH_DEVICE: u64 = 4;

/// 攻方 K1 那一形挂上之后长 13 个单元被拒一次（会话推一串抬 F、把 F 抬到上限）；C 那一圈整个换成
/// [`PUBLISHES_UNTIL_THE_RING_SLOT_HOLDING_THE_INJECTED_ISOLATION`] 次发布之后写的那一槽、每块盘记着
/// `isolated_free_slots_on_each_device` 个隔离着的空闲槽（注入级；原来那一圈置的隔离位照旧留着，推环放不出它们），再把每块盘没挡的槽只留
/// [`UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK`] 个。unlink 第一个文件：意图发成、之后的走空被宽护卡住。
/// 交回挂着的池、这一份报与那一个 unlink。
fn pool_whose_unlink_is_stuck_in_the_walk_after_the_intent_behind_a_short_push(
    isolated_free_slots_on_each_device: u64,
) -> (
    PoolUnderTest<SparseBlockDevice>,
    StuckBehindTheWideProtection,
    UserChange<'static>,
) {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
    );
    let ring_slot_holding_the_injected_isolation = target_for_publish(
        CheckpointTxg(
            pool.session().current.root().checkpoint_txg.0
                + PUBLISHES_UNTIL_THE_RING_SLOT_HOLDING_THE_INJECTED_ISOLATION,
        ),
        pool.parameters.geometry.root_ring_slots_per_region,
    );
    {
        let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
        let placements = isolate_free_slots(allocator, isolated_free_slots_on_each_device);
        allocator.note_placements_named_by_unapplied_journal_records(BTreeMap::from([(
            ring_slot_holding_the_injected_isolation,
            placements,
        )]));
        leave_only_unblocked_slots_on_each_device(
            allocator,
            UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK,
        );
    }
    pool.write_time_seconds += 1;
    let unlink = UserChange::Unlink {
        inode: InodeNumber(FIRST_INODE_NUMBER),
        write_time_seconds: pool.write_time_seconds,
    };
    let refusal = pool
        .publish(unlink, None)
        .expect_err("没挡的槽只剩那么几个：unlink 被拒");
    let WriteStuckBehindTheWideProtection {
        versions_published,
        stuck,
        ..
    } = stuck_behind_the_wide_protection_reported_by(refusal, "短推环那一形的 unlink");
    assert_eq!(
        versions_published, 1,
        "意图那一次发成、被拒的是它之后走空的一步（阳性对照：卡在走空里）"
    );
    assert_eq!(
        stuck.ring_slot_of_r, ring_slot_holding_the_injected_isolation,
        "推环推到注入的那一槽（阳性对照：C 那一圈只剩这一槽）"
    );
    (pool, stuck, unlink)
}

/// unlink 意图之后的走空被宽护卡住、推环做得成、推完之后走空照旧被拒（攻方 `research/prompts/m3-batch10z-vw-code-r1-opus-output.md`
/// 第六节、正推 K3-4 都造出了这一形）：报里记着的这次改动还剩什么只是走空；要出路时预演推环做成、推完之后走空照旧被拒，只列卸载再挂；
/// 预演交回的拒绝与照出路二真推完、再发同一个 unlink 交回的拒绝 `Debug` 逐字相同。预演里只剩走空那一支当成恒放得下
/// （变异 `walkSkipped`）时这一形列推环、预演说推完放得下。
#[test]
fn an_unlink_whose_walk_after_the_intent_stays_refused_after_a_push_that_goes_through_lists_only_unmounting(
) {
    let (pool, stuck, unlink) =
        pool_whose_unlink_is_stuck_in_the_walk_after_the_intent_behind_a_short_push(
            ISOLATED_FREE_SLOTS_THE_INJECTED_RING_SLOT_HOLDS_ON_EACH_DEVICE,
        );
    let remainder = remainder_of_the_change_recorded_in(&stuck);
    assert!(
        remainder.starts_with("remainder_of_the_change: WalkTheQueue }"),
        "意图已发成：报里记着的这次改动还剩什么只是走空，而记的是 {remainder}"
    );
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, "短推环那一形");
    assert_eq!(
        ways_out.len(),
        1,
        "推完之后走空照旧被拒：只列卸载再挂：{ways_out:?}"
    );
    assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring(
        &pool,
        &pushing_the_root_ring,
        unlink,
        "短推环那一形",
    );
}

/// 注入的那一槽每块盘记着几个隔离着的空闲槽：推完之后走空放得下那一形（攻方 `research/prompts/m3-batch10z-xy-code-r1-opus-model/atk_xy_c1.rs`
/// 的 k = 4、iso = 16、left = 30）。
const ISOLATED_FREE_SLOTS_THE_INJECTED_RING_SLOT_HOLDS_ON_EACH_DEVICE_FOR_THE_WALK_TO_FIT_AFTER_THE_PUSH:
    u64 = 16;

/// 建 inode 的个数：推完之后在走空之后发的那一个别的改动。
const INODES_CREATED_AFTER_THE_PUSH_BEHIND_THE_WALK: u64 = 1;

/// H 形（判决 `research/prompts/m3-batch10z-xy-code-r1-main-verification.md` 第二节 K1-1「H 形推完放得下」）：unlink 意图之后的走空被宽护卡住、
/// 推完之后走空放得下。报里只剩走空；要出路时预演放得下、列推环，出路二末句说意图已发成、推完之后剩下的走空在下一次改动开头走、放得下，
/// 不再发这次改动——不说「推完之后这次改动放得下」（照它再发同一个 unlink：开头的走空走完、池里已没有文件，拒成 `NoFileVersionToChange`）。
/// 照出路二推环之后原样再发同一个 unlink 被拒成那一种；推环之后发一个别的改动（建 inode），开头的走空做成（队列走空、现行那一版往后走了）。
#[test]
fn an_unlink_whose_walk_after_the_intent_fits_after_the_push_lists_the_push_saying_the_intent_is_published_and_not_to_send_the_unlink_again(
) {
    let why = "推完之后走空放得下那一形";
    let (pool, stuck, unlink) =
        pool_whose_unlink_is_stuck_in_the_walk_after_the_intent_behind_a_short_push(
            ISOLATED_FREE_SLOTS_THE_INJECTED_RING_SLOT_HOLDS_ON_EACH_DEVICE_FOR_THE_WALK_TO_FIT_AFTER_THE_PUSH,
        );
    let remainder = remainder_of_the_change_recorded_in(&stuck);
    assert!(
        remainder.starts_with("remainder_of_the_change: WalkTheQueue }"),
        "{why}：意图已发成：报里记着的这次改动还剩什么只是走空，而记的是 {remainder}"
    );
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, why);
    let released_on_each_device =
        released_by_the_push_if_the_change_is_admitted_after_it(&pushing_the_root_ring)
            .unwrap_or_else(|| {
                panic!("{why}：预演推完之后走空放得下，而预演的结局是 {pushing_the_root_ring:?}")
            });
    assert_eq!(ways_out.len(), 2, "{why}：列卸载再挂与推环：{ways_out:?}");
    // 报里只剩走空：推环那一条说推完之后剩下的走空在下一次改动开头走、不再发这次改动（照它再发同一个 unlink 被拒）。
    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
        &stuck,
        &ways_out[1],
        &released_on_each_device,
        WhatIsLeftAfterThePush::WalkTheQueueAtTheStartOfTheNextChange,
        why,
    );
    let refusal_of_the_same_unlink =
        refusal_of_the_same_change_after_pushing_the_root_ring_on_a_copy_of(&pool, unlink, why);
    let UserChangeRefused::NoFileVersionToChange {
        versions_published, ..
    } = refusal_of_the_same_unlink
    else {
        panic!(
            "{why}：照出路二推环之后原样再发同一个 unlink：拒成 NoFileVersionToChange，而交回 {refusal_of_the_same_unlink:?}"
        );
    };
    assert!(
        !versions_published.is_empty(),
        "{why}：再发那一次开头的走空走完了（发成的那几步带在拒绝里），接着发 unlink 时池里已没有文件"
    );
    let (mut pushed_pool, _) = pool_failing_one_changing_call_copied_from(&pool, None);
    push_the_root_ring_of(&mut pushed_pool)
        .unwrap_or_else(|push_refusal| panic!("{why}：照出路二推环做得成：{push_refusal:?}"));
    let version_after_the_push = *pushed_pool.session().current.root();
    assert!(
        !pushed_pool
            .session()
            .current
            .version_of_trees()
            .expect("推完之后有树")
            .deleted_inodes
            .is_empty(),
        "{why}：推环只发空发布，队列里还是第一个文件（阳性对照）"
    );
    pushed_pool.write_time_seconds += 1;
    let outcome = pushed_pool.publish(
        UserChange::NewInodes {
            count: INODES_CREATED_AFTER_THE_PUSH_BEHIND_THE_WALK,
            write_time_seconds: pushed_pool.write_time_seconds,
        },
        None,
    );
    let walked = pushed_pool
        .session()
        .current
        .version_of_trees()
        .expect("走空之后有树");
    assert!(
        walked.deleted_inodes.is_empty(),
        "{why}：推环之后建 inode：开头的走空做成，队列走空了（建 inode 的结局 {:?}）",
        outcome
            .as_ref()
            .map(|published| published.versions_published.len())
    );
    assert!(
        walked.root.checkpoint_txg > version_after_the_push.checkpoint_txg,
        "{why}：推环之后建 inode：走空那几步发成了，现行那一版往后走了"
    );
    // 这一形每块盘没挡的槽只留 30 个：走空之后建 inode 自己准入不够、推抬 F 被拒（量过；与出路二那句话无关，它只说剩下的走空放得下），
    // 走空那几步已落盘、带在拒绝里。
    match outcome {
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => assert_eq!(
            no_space.versions_published.last().map(|version| version.root),
            Some(walked.root),
            "{why}：推环之后建 inode：走空那几步发成、带在拒绝里，最后一步是现行那一版"
        ),
        other => panic!(
            "{why}：推环之后建 inode：开头的走空做成，之后建 inode 准入不够被拒（这一形量过），而交回 {other:?}"
        ),
    }
}

/// 攻方 K1 那一形挂上之后长 13 个单元被拒一次（会话推一串抬 F、把 F 抬到上限），每块盘没挡的槽只留 `unblocked_slots_left` 个
/// （`None` 不动），再发 `change`：被宽护卡住在这次改动本身（之前队列是空的、一版都没发成），交回报里记着的这次改动还剩什么。
fn remainder_recorded_when_the_change_itself_is_stuck(
    change: UserChange<'_>,
    unblocked_slots_left: Option<u64>,
    why: &str,
) -> String {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
    );
    if let Some(left) = unblocked_slots_left {
        leave_only_unblocked_slots_on_each_device(
            &mut pool.session.as_mut().expect("挂着的会话").allocator,
            left,
        );
    }
    let refusal = pool.publish(change, None).expect_err(why);
    let WriteStuckBehindTheWideProtection {
        versions_published,
        stuck,
        ..
    } = stuck_behind_the_wide_protection_reported_by(refusal, why);
    assert_eq!(
        versions_published, 0,
        "{why}：被拒的是这次改动本身，一版都没发成（阳性对照）"
    );
    remainder_of_the_change_recorded_in(&stuck)
}

/// 写入时间：与池里别的写都不同，记成 0 的改坏看得出来。
const WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION: u64 = 9_000_017;

/// 顺序写（攻方 K1 那一形 g = 13，不注入）：报里记着走空再发这次顺序写，内容全长、写入时间逐字相同。
#[test]
fn a_sequential_write_stuck_behind_the_wide_protection_keeps_its_whole_content_and_write_time() {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let content = content_of(
        crashed
            .bytes_of_the_file_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY),
        WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
    );
    let change = UserChange::SequentialWrite(FirstFile {
        content: &content,
        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
    });
    assert_eq!(
        remainder_recorded_when_the_change_itself_is_stuck(change, None, "顺序写"),
        format!(
            "remainder_of_the_change: WalkTheQueueThenPublishTheChange(SequentialWrite {{ content: {content:?}, write_time_seconds: {} }}) }}",
            WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION
        ),
        "报里记着的是走空再发这次顺序写：种类、内容全长、写入时间逐字相同"
    );
}

/// 覆盖写、建 inode、unlink 被宽护卡住在这次改动本身：每块盘没挡的槽只留这么几个（注入级）。
const UNBLOCKED_SLOTS_LEFT_FOR_THE_CHANGE_ITSELF_TO_BE_STUCK: u64 = 10;

/// 覆盖写：报里记着走空再发这次覆盖写，内容全长、写入时间逐字相同。
#[test]
fn an_overwrite_stuck_behind_the_wide_protection_keeps_its_whole_content_and_write_time() {
    let content = content_of(
        2999,
        WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
    );
    let change = UserChange::Overwrite(FirstFile {
        content: &content,
        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
    });
    assert_eq!(
        remainder_recorded_when_the_change_itself_is_stuck(
            change,
            Some(UNBLOCKED_SLOTS_LEFT_FOR_THE_CHANGE_ITSELF_TO_BE_STUCK),
            "覆盖写"
        ),
        format!(
            "remainder_of_the_change: WalkTheQueueThenPublishTheChange(Overwrite {{ content: {content:?}, write_time_seconds: {} }}) }}",
            WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION
        ),
        "报里记着的是走空再发这次覆盖写：种类、内容全长、写入时间逐字相同"
    );
}

/// 建 inode 的个数：不是 1，记错个数看得出来。
const INODES_CREATED_BY_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION: u64 = 3;

/// 建 inode：报里记着走空再建这几个 inode，个数、写入时间逐字相同。
#[test]
fn an_inode_creation_stuck_behind_the_wide_protection_keeps_its_count_and_write_time() {
    let change = UserChange::NewInodes {
        count: INODES_CREATED_BY_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
    };
    assert_eq!(
        remainder_recorded_when_the_change_itself_is_stuck(
            change,
            Some(UNBLOCKED_SLOTS_LEFT_FOR_THE_CHANGE_ITSELF_TO_BE_STUCK),
            "建 inode"
        ),
        format!(
            "remainder_of_the_change: WalkTheQueueThenPublishTheChange(NewInodes {{ count: {}, write_time_seconds: {} }}) }}",
            INODES_CREATED_BY_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
            WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION
        ),
        "报里记着的是走空再建这几个 inode：个数、写入时间逐字相同"
    );
}

/// unlink 卡在意图那一次：报里记着走空再 unlink 这个 inode，inode、写入时间逐字相同。
#[test]
fn an_unlink_stuck_behind_the_wide_protection_at_the_intent_keeps_its_inode_and_write_time() {
    let change = UserChange::Unlink {
        inode: InodeNumber(FIRST_INODE_NUMBER),
        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
    };
    assert_eq!(
        remainder_recorded_when_the_change_itself_is_stuck(
            change,
            Some(UNBLOCKED_SLOTS_LEFT_FOR_THE_CHANGE_ITSELF_TO_BE_STUCK),
            "unlink 意图那一次"
        ),
        format!(
            "remainder_of_the_change: WalkTheQueueThenPublishTheChange(Unlink {{ inode: {:?}, write_time_seconds: {} }}) }}",
            InodeNumber(FIRST_INODE_NUMBER),
            WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION
        ),
        "报里记着的是走空再 unlink 这个 inode：inode、写入时间逐字相同"
    );
}
