//! C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（被攻过零轮；
//! `research/prompts/unreadable-at-mount-r2-main-verification.md`「L3　C393」一节）：崩溃恢复抛弃根 C (2, 8) 之后再重开可写挂载（实例 4），
//! C 的分配记录树根节点或树表两份都读不出时，影子账不再跳过 C，从中央映射现算 C 引用的落点照样隔离（与读得出那一臂同是每盘 14 槽），
//! 计进 `abandoned_roots_unreadable` 往上报；之后覆盖写到 txg 31（txg 32 的根盖掉 C 的根槽）C 的单元一个都不复用、I-7.4（近 K 代块未被复用） 成立。
//! 改之前读不出只计数、跳过，隔离 0，重开之后第一次覆盖写（txg 14）就把 C 的数据单元（盘 0 槽 50184）改写、池级 checker 红 I-7.4
//! （调查员报告 `research/prompts/closeout-recheck-2026-09-28/c393-investigator-report.md`；这份用例从那份调查的副本用例改来）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs"]
mod scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
    publish_overwrite_in_process, with_unreadable_ranges, without_unreadable_ranges, BuiltPool,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
    FIXED_WRITE_TIME_SECONDS,
};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::{
    content_of, pool_after_a_recovery_abandoned_c, ISOLATED_14,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread,
    ShadowLedger,
};
use singlefs_core::transaction::TransactionUnit;
use singlefs_format::SLOT_BYTES;
use std::cell::Cell;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_readable_allocation_record_tree_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn c393_readable_allocation_record_tree_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(
) {
    scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_readable_allocation_record_tree_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_readable_tree_table_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn c393_readable_tree_table_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31() {
    scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_readable_tree_table_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_on_every_read`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn c393_allocation_record_tree_unreadable_on_every_read() {
    scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_on_every_read(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_until_the_reread_hook`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn c393_allocation_record_tree_unreadable_until_the_reread_hook() {
    scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_until_the_reread_hook(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy() {
    scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_on_every_read`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn c393_tree_table_unreadable_on_every_read() {
    scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_on_every_read(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_until_the_reread_hook`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn c393_tree_table_unreadable_until_the_reread_hook() {
    scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_until_the_reread_hook(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_only_on_the_first_read_of_each_copy`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn c393_tree_table_unreadable_only_on_the_first_read_of_each_copy() {
    scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_tree_table_unreadable_only_on_the_first_read_of_each_copy(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 钩子的阳性对照：同一段历史，最新那条根 (3, 10) 的实例表那一片在影子账那一读起坏、钩子里撤——产品的重读走到钩子（1 次），
/// 重读读得出，照隔离 14。证明上面各臂的「钩子 0 次」不是钩子接错了。
#[test]
fn c393_positive_control_the_hook_is_called_when_the_newest_instance_table_is_unreadable() {
    let (mut pool, _) = pool_after_a_recovery_abandoned_c("c393-hook-positive-control");
    let mut devices_for_the_page = pool.reopen_recorded();
    let newest = singlefs_core::recovery::choose_root(
        &devices_for_the_page,
        &singlefs_core::recovery::choose_system_configuration(&devices_for_the_page).expect("择"),
    )
    .expect("最新根");
    let _ = &mut devices_for_the_page;
    pool.devices = Some(devices_for_the_page);
    let page = newest.instance_table;
    let ranges: Vec<UnreadableRange> = page
        .locations
        .iter()
        .map(|location| UnreadableRange {
            device: location.device,
            offset_in_bytes: location.slot.to_device_offset().0,
            length_in_bytes: singlefs_format::DATA_UNIT_BYTES,
            failing_reads: if location.device == DeviceIdentity(0) {
                FailingReadsOfARange::FromTheNthOnward(3)
            } else {
                FailingReadsOfARange::Every
            },
        })
        .collect();
    let unreadable = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
    let hook_calls = Cell::new(0u32);
    let lifting = unreadable.clone();
    let mut hook = || {
        hook_calls.set(hook_calls.get() + 1);
        lifting.lift();
    };
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut hook),
    )
    .expect("重读读得出");
    pool.devices = Some(without_unreadable_ranges(devices));
    eprintln!(
        "name=c393-hook-positive-control hook_calls={} isolated={:?} abandoned_roots_unreadable={} rereads={:?}",
        hook_calls.get(),
        mounted.output.isolated_slots_per_device,
        mounted.output.abandoned_roots_unreadable,
        mounted.output.rereads
    );
    assert_eq!(hook_calls.get(), 1);
    assert_eq!(
        mounted.output.isolated_slots_per_device,
        ISOLATED_14.to_vec()
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_checker_on_a_persistently_damaged_allocation_record_tree_without_reuse`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn c393_checker_on_a_persistently_damaged_allocation_record_tree_without_reuse() {
    scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_checker_on_a_persistently_damaged_allocation_record_tree_without_reuse(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_checker_on_a_persistently_damaged_tree_table_without_reuse`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn c393_checker_on_a_persistently_damaged_tree_table_without_reuse() {
    scenarios_of_an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping::c393_checker_on_a_persistently_damaged_tree_table_without_reuse(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 抬 F 之前这次挂载里覆盖写几次：上限取「第 4 新的不同状态」（D16（发布语义） 已定项 1「抬 F 的上限」），覆盖写 5 次之后它是这次挂载里
/// 第 2 次覆盖写那一版，越过了被抛弃根的前一版（3 次时上限还停在第一个文件 B 那一版的 txg 4，副本上实测）。
const OVERWRITES_BEFORE_THE_RAISE: usize = 5;

/// 这一版的池里根环上按 `readable_roots` 的次序列出的、按 (实例, txg) 认的那几条根里，`wanted` 各排第几（抬 F 的影子账按这个次序逐条算被抛弃根）。
fn positions_in_the_ring_order(
    pool: &mut BuiltPool,
    wanted: &[(InstanceGeneration, CheckpointTxg)],
) -> Vec<usize> {
    let devices = pool.reopen_recorded();
    let system_configuration =
        singlefs_core::recovery::choose_system_configuration(&devices).expect("择系统配置");
    let roots = singlefs_core::recovery::readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    pool.devices = Some(devices);
    wanted
        .iter()
        .map(|key| {
            roots
                .iter()
                .position(|root| (root.instance, root.checkpoint_txg) == *key)
                .expect("那条根在根环里读得出")
        })
        .collect()
}

/// 把一条根的树表与中央映射树根节点两份都抹零（盘上真坏，重读也读不出）：它的账与从映射现算都不成。
fn zero_the_tree_table_and_the_mapping_root_of(
    pool: &mut BuiltPool,
    root: &singlefs_core::root_record::RootRecord,
) {
    use singlefs_core::block_device::{BlockDevice, WriteDurability};
    let mut devices = pool.reopen_recorded();
    for location in root
        .tree_table
        .locations
        .iter()
        .chain(root.mapping_root.locations.iter())
    {
        let index = devices
            .iter()
            .position(|(identity, _)| *identity == location.device)
            .expect("池里有这块盘");
        devices[index]
            .1
            .write_at(
                location.slot.to_device_offset(),
                &vec![0u8; usize::try_from(SLOT_BYTES).expect("槽宽")],
                WriteDurability::Plain,
            )
            .expect("抹零");
    }
    pool.devices = Some(devices);
}

/// 一条根的根槽在池里的位置与原字节：(盘, 偏移, 原来那一槽的字节)。
struct HiddenRootSlot {
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    original_bytes: Vec<u8>,
}

/// 把一条根的根槽抹成全零（之后可写挂载那一刻读回全零），交回原字节，挂载之后由 [`restore_the_root_slot`] 写回。
fn hide_the_root_slot_of(
    pool: &mut BuiltPool,
    root: &singlefs_core::root_record::RootRecord,
) -> HiddenRootSlot {
    use singlefs_core::block_device::{BlockDevice, WriteDurability};
    let pool_parameters = parameters();
    let ring_slot = singlefs_core::root_ring::target_for_publish(
        root.checkpoint_txg,
        pool_parameters.geometry.root_ring_slots_per_region,
    );
    let device = pool_parameters.region_devices
        [usize::try_from(ring_slot.region).expect("区域号装得进 usize")];
    let offset = singlefs_core::root_ring::slot_offset(
        ring_slot,
        pool_parameters.geometry.fixed_structure_slot_spacing,
    );
    let root_slot_bytes =
        usize::try_from(pool_parameters.geometry.physical_block_size).expect("物理块装得进 usize");
    let mut devices = pool.reopen_recorded();
    let index = devices
        .iter()
        .position(|(identity, _)| *identity == device)
        .expect("根槽所在的盘");
    let mut original_bytes = vec![0u8; root_slot_bytes];
    devices[index]
        .1
        .read_at(offset, &mut original_bytes)
        .expect("读根槽");
    devices[index]
        .1
        .write_at(offset, &vec![0u8; root_slot_bytes], WriteDurability::Plain)
        .expect("抹零根槽");
    pool.devices = Some(devices);
    HiddenRootSlot {
        device,
        offset,
        original_bytes,
    }
}

fn restore_the_root_slot(pool: &mut BuiltPool, hidden: &HiddenRootSlot) {
    use singlefs_core::block_device::{BlockDevice, WriteDurability};
    let mut devices = pool.reopen_recorded();
    let index = devices
        .iter()
        .position(|(identity, _)| *identity == hidden.device)
        .expect("根槽所在的盘");
    devices[index]
        .1
        .write_at(
            hidden.offset,
            &hidden.original_bytes,
            WriteDurability::Plain,
        )
        .expect("写回根槽");
    pool.devices = Some(devices);
}

/// 抬 F 那一串在影子账上被拒（代码轮 LR3，攻方 `m3-leftover-code-r1` Z2 ③：此前没有用例走到这一支）：
/// 崩溃恢复先后抛弃 C (2, 8) 与 E（实例 3 覆盖写 D 之后的那一版），再可写挂载一次、覆盖写 [`OVERWRITES_BEFORE_THE_RAISE`] 次，然后把两条被抛弃根里影子账排在后面的那一条的
/// 树表与中央映射树根两份都抹零。抬 F 到上限：新 F 越过两条被抛弃根的前一版，排在前面那条被抛弃根引用的槽（前一版那张实例表这类）
/// 此刻只被它引用，影子账先给它补上隔离；算到后面那条，账与从映射现算都不成、重读一次仍不成，`isolate_slots_referenced_only_by_abandoned_roots`
/// 交 `Err`——抬 F 拒成 `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`、点名那条根，一个写都不发，
/// 分配器整个换回抬 F 之前那一份（补上的隔离撤掉）。
/// 抬 F 那一串里根环表记着的被抛弃根照记着的落点算、读不出也不拒（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，用户 2026-10-07 定，
/// 根环线第 10z 批）：要走到影子账的拒，排在后面的那一条得是根环表里没记着的——再可写挂载那一刻它的根槽读回全零（挂载之后写回），
/// 根环表把那一槽记成「挂载时读回全零」，抬 F 那一遍读出它、照盘上读它的账（那一槽记的不是有效根，C 那一圈也不借：两样都没记着的照今天读盘，
/// D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定）。
/// 判别力：影子账交 `Err` 时不换回分配器，前一条补上的隔离留在分配器上，「与抬 F 之前逐项相同」那一项红。
#[test]
fn a_floor_raise_whose_shadow_ledger_meets_an_abandoned_root_unreadable_by_account_and_mapping_is_refused_and_restores_the_allocator(
) {
    let (mut pool, abandoned_c) =
        pool_after_a_recovery_abandoned_c("lr3-raise-refused-on-the-shadow-ledger");
    let before_d = pool.output.clone();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &before_d,
        &content_of(3000, 21),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(3),
    )
    .expect("D");
    let before_e = pool.output.clone();
    let abandoned_e = publish_overwrite_in_process(
        &mut pool,
        &before_e,
        &content_of(2200, 23),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(3),
    )
    .expect("E");
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned_e);
    let keys = [
        (abandoned_c.root.instance, abandoned_c.root.checkpoint_txg),
        (abandoned_e.root.instance, abandoned_e.root.checkpoint_txg),
    ];
    let positions = positions_in_the_ring_order(&mut pool, &keys);
    let (earlier, later) = if positions[0] > positions[1] {
        (&abandoned_e, &abandoned_c)
    } else {
        (&abandoned_c, &abandoned_e)
    };
    let hidden_root_slot = hide_the_root_slot_of(&mut pool, &later.root);
    let mut devices_of_the_mount = pool.reopen_recorded();
    let mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices_of_the_mount)
        .expect("排在后面那条被抛弃根的根槽读回全零、前面那条读得出：照常可写挂载");
    let instance = mounted.output.instance;
    pool.devices = Some(devices_of_the_mount);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current.into_file_version().expect("带文件");
    restore_the_root_slot(&mut pool, &hidden_root_slot);
    let root_ring_table = pool
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表");
    assert!(
        root_ring_table
            .placements_recorded_for_the_abandoned_root(
                earlier.root.instance,
                earlier.root.checkpoint_txg
            )
            .is_some()
            && root_ring_table
                .placements_recorded_for_the_abandoned_root(
                    later.root.instance,
                    later.root.checkpoint_txg
                )
                .is_none(),
        "根环表记着排在前面的那条被抛弃根、没记着排在后面的那一条"
    );
    for overwrite_index in 0..OVERWRITES_BEFORE_THE_RAISE {
        let previous = pool.output.clone();
        pool.output = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content_of(1800 + overwrite_index, overwrite_index + 31),
            FIXED_WRITE_TIME_SECONDS + 600,
            instance,
        )
        .expect("这次挂载里覆盖写");
    }
    zero_the_tree_table_and_the_mapping_root_of(&mut pool, &later.root);
    let allocator_before_the_raise = format!("{:?}", pool.allocator);
    let image_before_the_raise = pool.memory_pool();
    let mut devices_of_the_raise = pool.reopen_recorded();
    let refusal = singlefs_core::mount::raise_rollback_floor_to_the_admission_ceiling(
        &parameters(),
        &mut devices_of_the_raise,
        &mut pool.allocator,
        &mut pool.output,
        ShadowLedger::On,
    )
    .expect_err("影子账算到账与映射都读不出的被抛弃根：拒这一次抬");
    pool.devices = Some(devices_of_the_raise);
    assert!(
        matches!(
            &refusal,
            singlefs_core::mount::MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if matches!(
                    still_unreadable.as_ref(),
                    singlefs_core::mount::StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { abandoned_root }
                        if (abandoned_root.instance, abandoned_root.checkpoint_txg)
                            == (later.root.instance, later.root.checkpoint_txg)
                )
        ),
        "拒因点名影子账排在后面的那条被抛弃根：{refusal:?}"
    );
    assert_eq!(
        format!("{:?}", pool.allocator),
        allocator_before_the_raise,
        "分配器与抬 F 之前逐项相同：前一条被抛弃根补上的隔离撤掉了"
    );
    assert!(
        pool.memory_pool() == image_before_the_raise,
        "影子账在任何写之前拒：两块盘逐字节不变"
    );
}

/// 被抛弃根的实例表有两片时，账读不出、从中央映射现算的那一臂把两片都算进它引用的落点（代码轮 LR2，攻方 `m3-leftover-code-r1` Z4 ①
/// 「现算只认实例表第 0 片」放过：用例的池都只有一片实例表）。
/// 造法：A、B（实例 1）之后连取 370 次号再可写挂载（实例 372），写行把 1 到 371 的 371 行写进两片实例表；覆盖写 C、崩溃恢复抛弃 C
/// （实例 373 写行，换下两片）；C 的树表两份抹零（账读不出）再可写挂载（实例 374）：挂载那一遍读不出 C 的账、从映射现算出它引用的落点（两片都在内），
/// 根环表把 C 记成被抛弃根、记着这一份。覆盖写 [`OVERWRITES_BEFORE_THE_RAISE`] 次、抬 F 到上限：新 F 越过 C 的前一版，C 那两片实例表此刻只被 C 引用、
/// 在现行那一版的账里已释放；抬 F 那一串照根环表里记着的那一份算（不再从盘上读 C 的账、也不再现算，D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，
/// 根环线第 10z 批），两片都隔离，回收之后照样不空闲。
/// 判别力：现算只认实例表第 0 片时挂载那一遍就只记下第 0 片，抬 F 照它算，第 1 片不隔离、回收之后空闲，那一项红；
/// 抬 F 那一串记着的落点被削（只交一部分或交空）时同样红。
#[test]
fn an_abandoned_root_with_a_two_page_instance_table_and_an_unreadable_account_isolates_both_pages_from_the_mapping(
) {
    let mut pool = build_pool("lr2-two-page-instance-table-recomputed-from-the-mapping");
    let first = pool.output.clone();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(4100, 3),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("B");
    {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = singlefs_core::transaction::PoolWriter::new(
            &publish_parameters,
            devices.as_mut_slice(),
        );
        // 迭代上界 370：取号只写系统配置两槽，下一次可写挂载为每个烧掉的号写一行。
        for _ in 0..370 {
            singlefs_core::transaction::acquire_instance(&mut writer).expect("取号");
        }
    }
    let mut devices_after_the_acquisitions = pool.reopen_recorded();
    let mounted_after_the_acquisitions =
        singlefs_core::mount::mount_writable(&parameters(), &mut devices_after_the_acquisitions)
            .expect("连取 370 次号之后可写挂载");
    let instance_of_c = mounted_after_the_acquisitions.output.instance;
    pool.devices = Some(devices_after_the_acquisitions);
    pool.allocator = mounted_after_the_acquisitions.allocator;
    pool.output = mounted_after_the_acquisitions
        .current
        .into_file_version()
        .expect("带文件");
    let before_c = pool.output.clone();
    let abandoned_c = publish_overwrite_in_process(
        &mut pool,
        &before_c,
        &content_of(2500, 11),
        FIXED_WRITE_TIME_SECONDS + 60,
        instance_of_c,
    )
    .expect("C");
    let pages_of_c: Vec<singlefs_core::pointer::NodePointer> = {
        let devices_for_the_page_pointers = pool.reopen_recorded();
        let pages = singlefs_core::recovery::instance_table_page_pointers_as_far_as_readable(
            &devices_for_the_page_pointers,
            &abandoned_c.root,
        );
        pool.devices = Some(devices_for_the_page_pointers);
        pages
    };
    assert_eq!(
        pages_of_c.len(),
        2,
        "C 的实例表两片（371 行，一片装 369 行）"
    );
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned_c);
    {
        use singlefs_core::block_device::{BlockDevice, WriteDurability};
        let mut devices_to_zero_the_tree_table = pool.reopen_recorded();
        let tree_table_of_c = abandoned_c.unit(TransactionUnit::TreeTable);
        for (_, device) in &mut devices_to_zero_the_tree_table {
            device
                .write_at(
                    DeviceOffsetInBytes(tree_table_of_c.slot.0 * SLOT_BYTES),
                    &vec![0u8; tree_table_of_c.bytes.len()],
                    WriteDurability::Plain,
                )
                .expect("C 的树表两份抹零");
        }
        pool.devices = Some(devices_to_zero_the_tree_table);
    }
    let mut devices_of_the_remount = pool.reopen_recorded();
    let remounted =
        singlefs_core::mount::mount_writable(&parameters(), &mut devices_of_the_remount)
            .expect("C 的账读不出、从映射现算得成：照常可写挂载");
    assert_eq!(
        remounted.output.abandoned_roots_unreadable, 1,
        "C 的账读不出，计数往上报"
    );
    let instance = remounted.output.instance;
    pool.devices = Some(devices_of_the_remount);
    pool.allocator = remounted.allocator;
    pool.output = remounted.current.into_file_version().expect("带文件");
    for overwrite_index in 0..OVERWRITES_BEFORE_THE_RAISE {
        let previous = pool.output.clone();
        pool.output = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content_of(1800 + overwrite_index, overwrite_index + 31),
            FIXED_WRITE_TIME_SECONDS + 600,
            instance,
        )
        .expect("这次挂载里覆盖写");
    }
    let mut devices_of_the_raise = pool.reopen_recorded();
    let raised = singlefs_core::mount::raise_rollback_floor_to_the_admission_ceiling(
        &parameters(),
        &mut devices_of_the_raise,
        &mut pool.allocator,
        &mut pool.output,
        ShadowLedger::On,
    )
    .expect("抬 F 到上限");
    pool.devices = Some(devices_of_the_raise);
    assert!(
        matches!(
            &raised,
            singlefs_core::mount::RaiseToTheAdmissionCeiling::Raised(raised)
                if raised.publishes.first().is_some_and(|publish| publish.root.rollback_floor >= abandoned_c.root.checkpoint_txg)
        ),
        "新 F 越过 C 的前一版：{raised:?}"
    );
    let free_page_slots: Vec<(u32, u64)> = pages_of_c
        .iter()
        .flat_map(|page| page.locations)
        .filter(|location| {
            pool.allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == location.device)
                .expect("池里有这块盘")
                .is_free(location.slot)
        })
        .map(|location| (location.device.0, location.slot.0))
        .collect();
    assert_eq!(
        free_page_slots,
        Vec::<(u32, u64)>::new(),
        "C 那两片实例表的落点抬 F 回收之后一槽都不空闲（从映射现算两片都隔离）"
    );
}

/// 抬 F 那一串里根环表没记着落点的被抛弃根照今天读盘，账读不出就从中央映射现算、照样隔离、计进 `abandoned_roots_unreadable`
/// （C393 取 (b)-从映射现算；攻方 M11、正推 m19，判决 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第二节 K3：
/// 这一批之前默认档里没有一条用例走到抬 F 那一遍现算做成的这一支，只有一条慢档用例）。
/// 造法：崩溃恢复抛弃 C (2, 8) 之后再可写挂载（实例 4）那一刻 C 的根槽读回全零（挂载之后写回）：根环表把那一槽记成「挂载时读回全零」，
/// 没记着 C 的落点，C 那一圈在那一槽也不借（那一槽记的不是有效根）。挂上之后 C 的树表两份抹零（账读不出、中央映射读得出），正常卸载：
/// 抬 F 那一遍读出 C、按现行那一版的表被抛弃，读它的账读不出，从映射现算——卸载做成、计数 1，C 写出的单元一个都不空闲。
/// 判别力：抬 F 那一遍不从映射现算（直接重读、看凭据）时，C 的树表槽没被后来的写占、没有凭据，卸载拒成 `AccountOfAnAbandonedRoot`，这一条红。
#[test]
fn an_unmount_meeting_an_abandoned_root_not_in_the_root_ring_table_whose_account_is_unreadable_recomputes_it_from_the_mapping(
) {
    let (mut pool, abandoned_c) =
        pool_after_a_recovery_abandoned_c("m11-unmount-recomputes-from-the-mapping");
    let hidden_root_slot = hide_the_root_slot_of(&mut pool, &abandoned_c.root);
    let mut devices_of_the_mount = pool.reopen_recorded();
    let mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices_of_the_mount)
        .expect("C 的根槽读回全零：照常可写挂载");
    pool.devices = Some(devices_of_the_mount);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current.into_file_version().expect("带文件");
    restore_the_root_slot(&mut pool, &hidden_root_slot);
    let root_ring_table = pool
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表");
    assert_eq!(
        root_ring_table.placements_recorded_for_the_abandoned_root(
            abandoned_c.root.instance,
            abandoned_c.root.checkpoint_txg
        ),
        None,
        "挂载那一刻 C 的根槽读回全零：根环表里没记着 C 的落点"
    );
    assert_eq!(
        root_ring_table
            .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
                abandoned_c.root.instance,
                abandoned_c.root.checkpoint_txg
            ),
        None,
        "C 那一槽记的不是有效根：不借 C 那一圈的落点"
    );
    {
        use singlefs_core::block_device::{BlockDevice, WriteDurability};
        let mut devices_to_zero_the_tree_table = pool.reopen_recorded();
        let tree_table_of_c = abandoned_c.unit(TransactionUnit::TreeTable);
        for (_, device) in &mut devices_to_zero_the_tree_table {
            device
                .write_at(
                    DeviceOffsetInBytes(tree_table_of_c.slot.0 * SLOT_BYTES),
                    &vec![0u8; tree_table_of_c.bytes.len()],
                    WriteDurability::Plain,
                )
                .expect("C 的树表两份抹零");
        }
        pool.devices = Some(devices_to_zero_the_tree_table);
    }
    let mut version_to_unmount =
        singlefs_core::transaction::PoolVersion::WithFile(pool.output.clone());
    let mut devices_of_the_unmount = pool.reopen_recorded();
    let unmounted = singlefs_core::mount::unmount(
        &parameters(),
        &mut devices_of_the_unmount,
        &mut pool.allocator,
        &mut version_to_unmount,
        ShadowLedger::On,
    );
    pool.devices = Some(devices_of_the_unmount);
    match unmounted {
        Ok(singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
            assert_eq!(
                raised.abandoned_roots_unreadable, 1,
                "抬 F 那一遍读不出 C 的账：从映射现算，照样计数往上报"
            );
        }
        Ok(singlefs_core::mount::Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => {
            panic!("C 的账读不出、映射读得出：卸载从映射现算做成，而被拒：{refusal:?}")
        }
    }
    let free_units_of_c: Vec<(u32, u64)> = abandoned_c
        .units
        .iter()
        .flat_map(|unit| {
            pool.allocator
                .devices
                .iter()
                .filter(|device_map| device_map.is_free(unit.slot))
                .map(|device_map| (device_map.device.0, unit.slot.0))
                .collect::<Vec<(u32, u64)>>()
        })
        .collect();
    assert_eq!(
        free_units_of_c,
        Vec::<(u32, u64)>::new(),
        "卸载之后 C 写出的单元一个都不空闲（从映射现算的落点都隔离着）"
    );
}
