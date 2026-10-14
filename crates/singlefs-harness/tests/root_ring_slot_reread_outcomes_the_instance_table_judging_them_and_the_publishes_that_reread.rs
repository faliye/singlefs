//! 进程内记根时重读挂载那一刻读坏的根环槽（D16（发布语义） 已定项 1「环里最旧有效根」那一行的 E2 句与「根槽这一次读坏」那一行）：
//! 重读那一次的几种结局各记成什么、判「按实例表有效」用哪张表、产品路径上记根的几处各自走到重读。
//!
//! 读坏的槽在回收门槛里钉在它可能住过的最旧 txg，重读读出有效根或被抛弃的根之前门槛不越过它；重读时机照 E2（根环表里这一项成了最小的那个时）；
//! 读回全零的根槽在回收门槛里照「自证不过」那一类钉住，只是不并进准入抬 F 的上限（用户 2026-10-04 定全零不并进上限，2026-10-05 定乙；
//! 条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）。钉在最旧 txg 之后那一槽挂载那一刻起多半就是根环表里最小的那个，
//! 挂载之后第一次记根就重读。
//!
//! - 读得出而自证不过，与仍读不出同一处置，照旧钉着（用户 2026-10-04 定改法 A，代码轮第二轮判决
//!   `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1；攻方 `research/prompts/m3-root-ring-code-r2-opus-model/proto/src/bin/e2_reread_flip.rs`
//!   那一形：一条有效根所在的槽挂载那一刻读错、重读那一次读坏，改之前回收门槛越过它、I-3.1 记账少）。
//! - 读回全零：不算确认为空，照旧钉着（用户 2026-10-05 定乙）。同一目录 `front_hole_eio.rs` 那一形（从没写过的槽加挂载那一刻读错）上，
//!   改法 A 认下的「重读那一版按记的 txg 多记账、I-3.1 红在记账多的一边」因此回来了，到这个进程写掉那一槽为止（一版）。
//! - 读得出、自证过、按实例表有效的照读出来的 txg 算，那一槽从此算「这个进程知道住着根的槽」（用户 2026-10-04 定）；
//!   读得出、自证过而按实例表被抛弃的当没有根；判有效用的表写行之后是写行那一张（同一份判决第二节 RS2，正推腿探针 p1、p2）。
//! - 记根的产品路径：带单元的发布（`transaction::publish_admitted`，已有变异行管它）、树表 0 条那一版上要写行的写行发布
//!   （`transaction::prepare_the_row_publish_on_a_version_without_file`）走到重读。第一次可写挂载的零单元写行发布
//!   （`mount::establish_instance` 里 `rows_written` 为空那一支）走不到重读——那一刻根环里只有 mkfs 写的 txg 0，挂载时读坏的槽只可能是它，
//!   而它读不出时择不到根、挂载不成。
mod common;
mod common_images_for_the_checker_tier;
mod common_root_ring_holes;
#[path = "common_images_for_the_checker_tier/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs"]
mod scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::{FaultOfOneRootRingSlot, ReadsOfTheRootRingSlot, session_right_after_the_mount_before_the_reread, with_the_fault};
use common_root_ring_holes::{
    devices_of, fixed_geometry_of_the_ring_hole_pools,
    parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, recording_of,
    RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, MountError, ShadowLedger,
};
use singlefs_core::recovery::instance_table_chain_of_root;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::target_for_publish;
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg,
    );
}

/// 场景：同一盘面，挂载之后第一次覆盖写那一次重读读出 txg 18 的有效根；之后那一槽每次读都报设备错，接着准入抬 F 抬到上限。
/// 预期：算上限那一遍那一槽读不出、它是这个进程知道住着根的槽：重读一次仍读不出，在任何写之前拒
/// （`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`），盘上逐字节不变、录制流不多一步。
#[test]
fn a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write() {
    let mut session = session_right_after_the_mount_before_the_reread();
    session.overwrite_once();
    assert!(
        session.slot_is_known_to_hold_a_root(),
        "前提：重读读出了有效根"
    );
    session
        .fault
        .set_reads(ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead);
    let image_before = session.image_now();
    let operations_before = session.stream.operations().len();
    let refused = raise_rollback_floor_to_the_admission_ceiling(
        &parameters_of_the_ring_hole_pools(),
        &mut session.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    assert!(
        matches!(
            refused,
            Err(MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { ring_slot, .. })
                if ring_slot == session.ring_slot
        ),
        "知道住着根的槽重读仍读不出就拒：{refused:?}"
    );
    assert_eq!(
        session.stream.operations().len(),
        operations_before,
        "一个写、一道屏障都没发"
    );
    assert!(session.image_now() == image_before, "盘上逐字节不变");
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::an_abandoned_root_read_on_the_reread_is_taken_as_no_root`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn an_abandoned_root_read_on_the_reread_is_taken_as_no_root() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread::an_abandoned_root_read_on_the_reread_is_taken_as_no_root,
    );
}

/// 场景：实例 1 覆盖写到 txg 3 之后可写挂载（实例 2 写行）。
/// 预期：挂载交回的分配器上，进程内重读判「按实例表有效」用的那张表是写行那次发布写下的那一张（带实例 1 的行），
/// 不是挂载时最新那条根指着的那一张——池级 checker 按最新那条根的实例表判有效，写行之后它带的就是写行那一张。
#[test]
fn the_instance_table_judging_the_rereads_after_the_mount_is_the_one_the_row_publish_wrote() {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(&pool_overwritten_through(3));
    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
    let table_of = |root: &RootRecord| {
        instance_table_chain_of_root(&pool_of(&devices), root)
            .expect("实例表读得出")
            .records
    };
    let table_of_the_row_publish = table_of(mounted.output.row_publish.root());
    assert_ne!(
        table_of_the_row_publish,
        table_of(&mounted.output.chosen_root),
        "前提：写行那一张比挂载时最新那条根的多实例 1 那一行"
    );
    assert_eq!(
        mounted
            .allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表")
            .instance_table_judging_the_rereads(),
        &table_of_the_row_publish,
        "判有效用写行那一张"
    );
}

/// 树表 0 条的池上一次可写挂载：录下来，崩在 txg 落在 `crash_before_the_root_of` 里的第一个根槽写之前（那次发布的单元与记录已持久、根槽没持久；
/// 这次挂载没写那几个 txg 的根就整段落盘）。交回之后的盘面与这次挂载落盘的最后一个 txg（崩了的那一次是它的 txg）。
fn mount_a_pool_without_file(
    image: &MemoryPool,
    crash_before_the_root_of: &[u64],
) -> (MemoryPool, u64) {
    let parameters = parameters_of_the_ring_hole_pools();
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(image, &stream);
    mount_writable(&parameters, &mut recording).expect("树表 0 条的池可写挂载");
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let txg_of_the_root_written_by = |write: &singlefs_harness::memory_pool::RetainedWrite| {
        write
            .bytes()
            .and_then(|bytes| RootRecord::parse_slot(bytes, &parameters.filesystem_identifier))
            .map(|root| root.checkpoint_txg.0)
    };
    let cut = writes.iter().position(|write| {
        write.kind == StepKind::RootRecordFua
            && txg_of_the_root_written_by(write)
                .is_some_and(|txg| crash_before_the_root_of.contains(&txg))
    });
    let mut after = image.clone();
    match cut {
        Some(cut) => {
            let txg = txg_of_the_root_written_by(&writes[cut]).expect("根槽写带着一条根");
            after.apply_writes(&writes[..cut]);
            (after, txg)
        }
        None => {
            after.apply_writes(&writes);
            let last = writes
                .iter()
                .filter(|write| write.kind == StepKind::RootRecordFua)
                .filter_map(txg_of_the_root_written_by)
                .max()
                .expect("可写挂载至少写一条根");
            (after, last)
        }
    }
}

/// mkfs 之后一直不写文件，接连可写挂载、崩掉 `crash_before_the_root_of` 里那几个 txg 的根，到下一次挂载的写行是 `first_txg_of_the_next_mount` 为止。
fn pool_without_file_before_the_mount_writing_its_row_at(
    crash_before_the_root_of: &[u64],
    first_txg_of_the_next_mount: u64,
) -> MemoryPool {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices: RingHoleDevices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut image = pool_of(&devices);
    let mut last_txg = 0u64;
    // 迭代上界：每次挂载至少前进一个 txg，到下一次挂载的写行 txg 为止；跨轮携带的是盘面与落盘的最后一个 txg。
    while last_txg + 1 < first_txg_of_the_next_mount {
        let (after, last) = mount_a_pool_without_file(&image, crash_before_the_root_of);
        assert!(last > last_txg, "每次挂载都往前推");
        image = after;
        last_txg = last;
    }
    assert_eq!(
        last_txg + 1,
        first_txg_of_the_next_mount,
        "前提：下一次挂载的写行落在 {first_txg_of_the_next_mount}"
    );
    image
}

/// 场景：树表 0 条的池（S = 4），mkfs 之后接连可写挂载、从不写文件；txg 14 那次写行崩在根槽写之前（那一槽里留着上一圈 txg 2 的根），
/// txg 23 那次写行同样崩在根槽写之前；下一次可写挂载写行 24、暖机 25（两块盘各一条）。挂载那一刻（取号之前）txg 14 那一槽读不出，之后照读。
/// 预期：那一槽钉在它可能住过的最旧 txg 2，环里没有比它更旧的有效根（txg 23 那个洞里留着的是 11）：写行 24 那次记根之前它是根环表里最小的那个，
/// 重读读出 txg 2 的根（按写行那一张实例表有效）；挂载交回的根环表里那一槽不再按读坏记、算这个进程知道住着根的槽。
/// 这一条钉的是树表 0 条那一版上的写行发布（`transaction::prepare_the_row_publish_on_a_version_without_file`）走到 E2 重读。
#[test]
fn the_row_publish_on_a_version_without_file_rereads_the_slot_unreadable_at_mount_pinned_at_the_oldest_txg_it_can_have_held(
) {
    let image = pool_without_file_before_the_mount_writing_its_row_at(&[14, 23], 24);
    let parameters = parameters_of_the_ring_hole_pools();
    let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
        &parameters,
        CheckpointTxg(14),
        ReadsOfTheRootRingSlot::DeviceErrorUntilTheFirstWriteOfAnyDevice,
    );
    let stream = SharedStream::retaining_contents();
    let mut devices = with_the_fault(recording_of(&image, &stream), &fault);
    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.row_publish.root().checkpoint_txg,
            mounted
                .output
                .warm_up_publishes
                .iter()
                .map(|version| version.root().checkpoint_txg)
                .collect::<Vec<_>>()
        ),
        (CheckpointTxg(24), vec![CheckpointTxg(25)]),
        "前提：写行 24、暖机一次 25"
    );
    assert!(fault.failed_reads() > 0, "前提：挂载读阶段读那一槽读错");
    let ring_slot = target_for_publish(
        CheckpointTxg(14),
        parameters.geometry.root_ring_slots_per_region,
    );
    let occupancy = mounted
        .allocator
        .root_ring_occupancy()
        .expect("可写挂载装了根环表");
    assert_eq!(
        (
            occupancy.ring_slots_bad_at_mount().get(&ring_slot).copied(),
            occupancy
                .ring_slots_known_to_hold_a_root()
                .contains(&ring_slot)
        ),
        (None, true),
        "写行那次记根之前重读那一槽、读出 txg 2 的根"
    );
}
