//! 准入推空发布（D16（发布语义） 已定项 1「抬 F 的上限」「准入」两行；
//! 取 E173（上限卡在读坏槽的三种处置） 登记的「推空B」那一臂）：
//! 上限卡在挂载那一刻读坏、之后这个进程没写过的槽上（并进它之后上限不高于现行 F、不并它时高于现行 F）时，
//! `mount::push_one_floor_raise_within_the_admission_budget` 推一次不抬 F 的空发布（带现行 F、txg 加一，算进这次准入的已推次数）再判；
//! 预算 B = 8 放不下这一次（已推 + 1 + 这次准入为之判的那一次 > B）就停（`FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded`，
//! 调用方据此报 ENOSPC），不走不受 B 限的「推空W」；推到这个进程写掉那一槽，上限不再被它卡住，照抬 F 那一串抬。
//!
//! 盘面（S = 4 的 12 槽环洞池，区域归属 (0, 1, 0)）：实例 1 覆盖写到 txg 17；实例 2 可写挂载之后把 F 抬到 14（`mount::raise_rollback_floor`，
//! 14 不高于它算出的上限：17、16、15、14 是第 4 新的不同状态那条），进程退出；实例 3 可写挂载起 txg 14 那一槽（区域 2 槽 0）每次读都报设备错。
//! 此刻环里读得出、txg ≥ F = 14 的有效根是 15 起的那几条，用户可见的不同状态只有 17、16、15 三个（之后都是空发布、写行与暖机），
//! 照读得出的根上限取最旧的有效根 15；14 那一槽按它挂载时能住的最大 txg 14 并进来，上限 14 = F：卡在那一槽上。
//! 这个进程下一次写那一槽是 txg 26（区域 2 槽 0）。
mod common;
mod common_root_ring_holes;

use std::collections::BTreeMap;

use common::{
    with_unreadable_ranges, DeviceWithUnreadableRanges, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
};
use common_root_ring_holes::{
    devices_of, parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, recording_of,
    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::mount::{
    mount_writable, push_one_floor_raise_within_the_admission_budget, raise_rollback_floor,
    ring_slots_bad_or_reading_back_all_zero_at_mount_by, FloorOfThePushedPublishes,
    FloorRaisePushedWithinTheAdmissionBudget, FloorRaiseStop, RaisedFloor, ShadowLedger,
    PUBLISHES_PER_ADMISSION_AT_MOST,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::TransactionOutput;
use singlefs_verification::history::judge_one_floor_raise_pushed_by_the_session;
use singlefs_verification::memory_pool::SparseBlockDevice;
use singlefs_verification::model::{
    IdealModel, ModelCheckpointTxg, ModelDeviceIdentity, ModelDisagreementAspect, ModelPoolGeometry,
};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 实例 1 覆盖写到的 txg。
const LAST_TXG_OF_INSTANCE_ONE: u64 = 17;

/// 实例 2 抬到的 F：txg 14 那一条根，实例 3 挂载起它那一槽读不出。
const FLOOR_RAISED_BY_INSTANCE_TWO: CheckpointTxg = CheckpointTxg(14);

/// 照读得出的根算的上限：用户可见的不同状态只有 17、16、15 三个，取最旧的有效根 15。
const CEILING_FROM_THE_ROOTS_READ: CheckpointTxg = CheckpointTxg(15);

/// 这个进程下一次写 txg 14 那一槽（区域 2 槽 0）的 txg：14 + 环长 12。
const TXG_WRITING_THE_RING_SLOT_OF_THE_FLOOR_AGAIN: CheckpointTxg = CheckpointTxg(26);

type DevicesOfTheHeldMount = Vec<(
    DeviceIdentity,
    DeviceWithUnreadableRanges<RecordingBlockDevice<SparseBlockDevice>>,
)>;

/// 实例 3 可写挂载之后、第一次为空间推之前：盘（txg 14 那一槽按 `ring_slot_of_the_floor_reads` 读坏）、录下的流、分配器、现行那一版。
struct PoolHeldAtTheRingSlotOfTheFloor {
    devices: DevicesOfTheHeldMount,
    ring_slot_of_the_floor_reads: SharedUnreadableRanges,
    stream: SharedStream,
    allocator: PoolAllocator,
    current: TransactionOutput,
    ring_slot_of_the_floor: RootRingSlot,
}

impl PoolHeldAtTheRingSlotOfTheFloor {
    /// 一次准入里再推一次，这次准入已推 `publishes_pushed` 次。
    fn push_once_more_in_an_admission_that_already_pushed(
        &mut self,
        publishes_pushed: usize,
    ) -> FloorRaisePushedWithinTheAdmissionBudget {
        push_one_floor_raise_within_the_admission_budget(
            &parameters_of_the_ring_hole_pools(),
            &mut self.devices,
            &mut self.allocator,
            &mut self.current,
            ShadowLedger::On,
            publishes_pushed,
        )
        .unwrap_or_else(|error| panic!("为空间推这一次不报错：{error:?}"))
    }
}

/// 实例 1 覆盖写到 17；实例 2 挂上、把 F 抬到 14、退出；实例 3 挂上，txg 14 那一槽从这一次挂载起每次读都报设备错。
fn pool_whose_ceiling_is_held_by_the_ring_slot_of_the_floor_unreadable_since_the_mount(
) -> PoolHeldAtTheRingSlotOfTheFloor {
    let parameters = parameters_of_the_ring_hole_pools();
    let image_after_the_raise = {
        let mut devices = devices_of(&pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE));
        let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
        let mut allocator = mounted.allocator;
        let mut current = mounted
            .current
            .into_file_version()
            .expect("覆盖写过的池，这一版带文件");
        let raised: RaisedFloor = raise_rollback_floor(
            &parameters,
            &mut devices,
            &mut allocator,
            &mut current,
            FLOOR_RAISED_BY_INSTANCE_TWO,
            ShadowLedger::On,
        )
        .expect("实例 2 把 F 抬到 14：不高于它算出的上限（17、16、15、14 里第 4 新的那条）");
        assert_eq!(
            (raised.ceiling, current.root.rollback_floor),
            (FLOOR_RAISED_BY_INSTANCE_TWO, FLOOR_RAISED_BY_INSTANCE_TWO),
            "前提：实例 2 算出的上限与抬到的 F 都是 14"
        );
        pool_of(&devices)
    };
    let ring_slot_of_the_floor = target_for_publish(
        FLOOR_RAISED_BY_INSTANCE_TWO,
        parameters.geometry.root_ring_slots_per_region,
    );
    let ring_slot_of_the_floor_reads = SharedUnreadableRanges::new(
        vec![UnreadableRange {
            device: parameters.region_devices
                [usize::try_from(ring_slot_of_the_floor.region).expect("区域号")],
            offset_in_bytes: slot_offset(
                ring_slot_of_the_floor,
                parameters.geometry.fixed_structure_slot_spacing,
            )
            .0,
            length_in_bytes: u64::from(parameters.geometry.physical_block_size),
            failing_reads: FailingReadsOfARange::Every,
        }],
        UnreadableRangeReadBack::DeviceError,
    );
    let stream = SharedStream::retaining_contents();
    let mut devices = with_unreadable_ranges(
        recording_of(&image_after_the_raise, &stream),
        &ring_slot_of_the_floor_reads,
    );
    let mounted = mount_writable(&parameters, &mut devices)
        .expect("实例 3 的可写挂载：F 那条根那一槽读不出，择根与重建分配器照读得出的根做");
    let allocator = mounted.allocator;
    let current = mounted
        .current
        .into_file_version()
        .expect("覆盖写过的池，这一版带文件");
    assert_eq!(
        current.root.rollback_floor, FLOOR_RAISED_BY_INSTANCE_TWO,
        "前提：实例 3 现行那一版带的 F 是 14"
    );
    assert!(
        current.root.checkpoint_txg < TXG_WRITING_THE_RING_SLOT_OF_THE_FLOOR_AGAIN,
        "前提：挂载写行与暖机之后这个进程还没写过 txg 14 那一槽（现行那一版 txg {:?}）",
        current.root.checkpoint_txg
    );
    assert_eq!(
        ring_slots_bad_or_reading_back_all_zero_at_mount_by(&allocator),
        BTreeMap::from([(ring_slot_of_the_floor, FLOOR_RAISED_BY_INSTANCE_TWO)]),
        "前提：根环表把 txg 14 那一槽记成挂载时读坏、能住的最大 txg 14，准入抬 F 的上限并它"
    );
    PoolHeldAtTheRingSlotOfTheFloor {
        devices,
        ring_slot_of_the_floor_reads,
        stream,
        allocator,
        current,
        ring_slot_of_the_floor,
    }
}

/// 场景：上限卡在 F 那条根那一槽上（挂载起读不出），一次准入从零预算起推。
/// 预期：每一次都推一次不抬 F 的空发布（判过的上限 14，不并那一槽时是 15），F 一直是 14，推到 txg 26 写掉那一槽为止；
/// 写掉之后根环表不再记它、那一槽照读（读故障撤掉），下一次推照抬 F 那一串把 F 抬到 15。整次准入的发布数在 B = 8 之内。
/// 判别力：卡在那一槽上就停在上限、报 `FloorAlreadyAtTheCeiling` 时第一次推就停，这一条红
/// （`crates/mutations.tsv`「D16（发布语义） 已定项 1 推空：上限卡在挂载时读坏的槽上不推空发布、照旧停在上限」）。
#[test]
fn admission_pushes_empty_publishes_keeping_the_floor_until_the_slot_holding_the_ceiling_is_written_then_raises_the_floor(
) {
    let mut pool =
        pool_whose_ceiling_is_held_by_the_ring_slot_of_the_floor_unreadable_since_the_mount();
    let mut publishes_pushed = 0;
    // 迭代上界：txg 逐次加一推到 26，至多环长 12 次；跨轮携带的是已推次数与现行那一版。
    while pool.current.root.checkpoint_txg < TXG_WRITING_THE_RING_SLOT_OF_THE_FLOOR_AGAIN {
        let txg_before = pool.current.root.checkpoint_txg;
        let FloorRaisePushedWithinTheAdmissionBudget::Raised(pushed) =
            pool.push_once_more_in_an_admission_that_already_pushed(publishes_pushed)
        else {
            panic!(
                "txg {txg_before:?} 上推：上限卡在 F 那条根那一槽上，推一次不抬 F 的空发布、不停"
            );
        };
        assert_eq!(
            (
                pushed.floor_of_the_publishes,
                pushed.ceiling,
                pushed.publishes.len(),
                pool.current.root.checkpoint_txg,
                pool.current.root.rollback_floor,
            ),
            (
                FloorOfThePushedPublishes::KeptWhileTheCeilingIsHeldByRingSlotsBadSinceTheMount {
                    ceiling_from_the_roots_read: CEILING_FROM_THE_ROOTS_READ,
                },
                FLOOR_RAISED_BY_INSTANCE_TWO,
                1,
                CheckpointTxg(txg_before.0 + 1),
                FLOOR_RAISED_BY_INSTANCE_TWO,
            ),
            "txg {txg_before:?} 上推：一次带现行 F 14 的空发布，判过的上限 14、不并那一槽时 15"
        );
        publishes_pushed += 1;
    }
    assert!(
        ring_slots_bad_or_reading_back_all_zero_at_mount_by(&pool.allocator).is_empty(),
        "txg 26 写掉了那一槽：根环表不再把 {:?} 记成挂载时读坏",
        pool.ring_slot_of_the_floor
    );
    pool.ring_slot_of_the_floor_reads.lift();
    let FloorRaisePushedWithinTheAdmissionBudget::Raised(raised) =
        pool.push_once_more_in_an_admission_that_already_pushed(publishes_pushed)
    else {
        panic!("写掉那一槽之后上限不再被它卡住：照抬 F 那一串抬");
    };
    assert_eq!(
        (
            raised.floor_of_the_publishes,
            raised.ceiling,
            pool.current.root.rollback_floor,
        ),
        (
            FloorOfThePushedPublishes::RaisedToTheCeiling,
            CEILING_FROM_THE_ROOTS_READ,
            CEILING_FROM_THE_ROOTS_READ,
        ),
        "写掉那一槽之后照读得出的根算上限 15，F 抬到 15"
    );
    assert!(
        publishes_pushed + raised.publishes.len() < PUBLISHES_PER_ADMISSION_AT_MOST,
        "这次准入推了 {publishes_pushed} 次不抬 F 的空发布、{} 次抬 F 的空发布，再加为之判的那一次在 B = 8 之内（推的少于 8）",
        raised.publishes.len()
    );
}

/// 场景：上限卡在 F 那条根那一槽上，这次准入已经推了 7 次，再推一次就是 7 + 1 + 为之判的那一次 = 9 > B；
/// 同一刻已推 6 次时还放得下一次。
/// 预期：已推 7 次时停（`PublishesPerAdmissionWouldBeExceeded`，点名已推 7 次），一个写都不发、现行那一版不动；
/// 已推 6 次时推一次不抬 F 的空发布。推空受 B 限（推空B），不是推到写掉那一槽为止（推空W）。
/// 判别力：推空不判预算（推空W）时已推 7 次那一次也推了，这一条红（`crates/mutations.tsv`「D16（发布语义） 已定项 1 推空：不抬 F 的空发布不判 B」）。
#[test]
fn admission_with_no_budget_left_for_one_more_empty_publish_stops_while_the_ceiling_is_held_by_a_slot_unreadable_since_the_mount(
) {
    let mut pool =
        pool_whose_ceiling_is_held_by_the_ring_slot_of_the_floor_unreadable_since_the_mount();
    let operations_before = pool.stream.operation_count();
    let txg_before = pool.current.root.checkpoint_txg;
    let stopped = pool
        .push_once_more_in_an_admission_that_already_pushed(PUBLISHES_PER_ADMISSION_AT_MOST - 1);
    let FloorRaisePushedWithinTheAdmissionBudget::Stopped(
        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded {
            publishes_pushed, ..
        },
    ) = stopped
    else {
        panic!("已推 7 次：再推一次不抬 F 的空发布就超过 B = 8，停在预算上；而是 {stopped:?}");
    };
    assert_eq!(
        (
            publishes_pushed,
            pool.stream.operation_count(),
            pool.current.root.checkpoint_txg,
        ),
        (
            PUBLISHES_PER_ADMISSION_AT_MOST - 1,
            operations_before,
            txg_before,
        ),
        "停在预算上：点名已推 7 次，一个写都不发，现行那一版不动"
    );
    let FloorRaisePushedWithinTheAdmissionBudget::Raised(pushed) = pool
        .push_once_more_in_an_admission_that_already_pushed(PUBLISHES_PER_ADMISSION_AT_MOST - 2)
    else {
        panic!("已推 6 次：6 + 1 + 1 = 8 放得下一次不抬 F 的空发布");
    };
    assert_eq!(
        (
            pushed.floor_of_the_publishes,
            pool.current.root.checkpoint_txg,
            pool.current.root.rollback_floor,
        ),
        (
            FloorOfThePushedPublishes::KeptWhileTheCeilingIsHeldByRingSlotsBadSinceTheMount {
                ceiling_from_the_roots_read: CEILING_FROM_THE_ROOTS_READ,
            },
            CheckpointTxg(txg_before.0 + 1),
            FLOOR_RAISED_BY_INSTANCE_TWO,
        ),
        "已推 6 次：推一次带现行 F 14 的空发布"
    );
}

/// 场景：上限卡在 F 那条根那一槽上，准入推了一次不抬 F 的空发布；把这一份交给随机历史与故障注入共用的胶水
/// （`history::judge_one_floor_raise_pushed_by_the_session`，判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 4 问）。
/// 预期：胶水报「推空，模型答不了」那一格，带实现报的上限 14；模型一步不走（模型不建挂载时读坏、读回全零的槽，答不了上限卡没卡在它们上）。
/// 判别力：胶水不按 `floor_of_the_publishes` 分开、把这一次当抬 F 交模型比时，模型拿它去答抬到 14 的一串（这里模型没开会话，报会话状态对不上；
/// 开着会话时报根数或上限对不上，都是假的对不上），这一条红（`crates/mutations.tsv`「胶水把推空当抬 F 交模型比」）。
#[test]
fn the_history_glue_reports_that_the_model_does_not_answer_an_empty_publish_kept_while_the_ceiling_is_held(
) {
    let mut pool =
        pool_whose_ceiling_is_held_by_the_ring_slot_of_the_floor_unreadable_since_the_mount();
    let FloorRaisePushedWithinTheAdmissionBudget::Raised(pushed) =
        pool.push_once_more_in_an_admission_that_already_pushed(0)
    else {
        panic!("上限卡在 F 那条根那一槽上：推一次不抬 F 的空发布、不停");
    };
    assert_eq!(
        pushed.floor_of_the_publishes,
        FloorOfThePushedPublishes::KeptWhileTheCeilingIsHeldByRingSlotsBadSinceTheMount {
            ceiling_from_the_roots_read: CEILING_FROM_THE_ROOTS_READ,
        },
        "前提：这一次是上限卡在那一槽上推的不抬 F 的空发布"
    );
    let mut model = IdealModel::after_make_filesystem(ModelPoolGeometry {
        devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
        device_size_in_bytes: DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
    });
    let model_before = model.clone();
    let verdict = judge_one_floor_raise_pushed_by_the_session(&mut model, &pushed);
    let disagreement = verdict
        .disagreement
        .expect("推空那一份模型不答：交回对不上的那一格");
    assert_eq!(
        (
            disagreement.aspect,
            disagreement.implementation_reported_ceiling,
        ),
        (
            ModelDisagreementAspect::EmptyPublishKeepingTheFloorWhileTheCeilingIsHeldIsNotModeled,
            Some(ModelCheckpointTxg(FLOOR_RAISED_BY_INSTANCE_TWO.0)),
        ),
        "推空那一格、带实现报的上限 14：{disagreement:?}"
    );
    assert_eq!(model, model_before, "模型不答，一步不走");
}
