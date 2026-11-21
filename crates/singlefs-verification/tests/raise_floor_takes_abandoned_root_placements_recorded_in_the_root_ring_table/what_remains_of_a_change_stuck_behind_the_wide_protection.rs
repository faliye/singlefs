//! 被宽护卡住时报里记的这次改动还剩哪些：unlink 意图之后的走一遍、覆盖写之前的走一遍、顺序写、覆盖写、建 inode、unlink 本身各记哪几样，推完照旧被拒或放得下时怎么说，空聚簇段低于下限时在任何一串之前拒。

use super::*;

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
        SlotsBlockedByTheInjection::MarkedAllocated,
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

/// 注入把多出来的没挡的槽挡成哪一种：合取 7（`全空聚簇段数 ≥ N₀`）读的全空聚簇段数（`DeviceFreeMap::empty_segments`，按未释放分配记录数）
/// 跟不跟着掉。两种挡法下式子扣的槽数（已分配与第九项隔离各扣一次）、没挡的槽、段外没挡的槽对逐个相同。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotsBlockedByTheInjection {
    /// 标成已分配：位图上占着、算在未释放分配记录下（盘上不写记录），挡到的整段从此不算全空；单元区里每一整段都挡到时全空聚簇段掉到 0，
    /// 被拒的若是合取 7 判的前台写入，抬 F 之前先算的合取 7 那一判（D16（发布语义） 已定项 10 射程「抬 F 之前先算」）一串不推就拒。
    MarkedAllocated,
    /// 置影子账隔离位、不记进根环表的任何一槽（推环与抬 F 都放不出它们）：不算在未释放分配记录下，全空聚簇段数不动，
    /// 合取 7 那一判不拦，走得到推满仍不够、被宽护卡住那一支。
    IsolatedOutsideTheRootRingTable,
}

/// 注入把每块盘没挡的槽压到几个、多出来的挡成哪一种。
#[derive(Clone, Copy, Debug)]
struct SlotsLeftUnblockedByTheInjection {
    unblocked_slots_left: u64,
    blocked_as: SlotsBlockedByTheInjection,
}

/// 每块盘的单元区里没挡的槽照 `blocked_as` 挡掉、只留 `unblocked_slots_left` 个（注入级，盘上不写记录；照
/// `admission_raises_the_floor_before_refusing.rs` 的 `pool_ready_to_unlink_with_few_unblocked_slots`）。隔离着挡的，挡完核每块盘的全空聚簇段不少于 `N₀`。
fn leave_only_unblocked_slots_on_each_device(
    allocator: &mut PoolAllocator,
    unblocked_slots_left: u64,
    blocked_as: SlotsBlockedByTheInjection,
) {
    for device_map in &mut allocator.devices {
        let unit_area_start = device_map.unit_area_start().slot().0;
        // 迭代上界是单元区槽数；跨轮只带没挡的槽数；出口：只剩下要留的那几个。
        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
            if device_map.unblocked_slots() <= unblocked_slots_left {
                break;
            }
            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
                match blocked_as {
                    SlotsBlockedByTheInjection::MarkedAllocated => {
                        device_map.mark_allocated(SlotNumber(slot), 1);
                    }
                    SlotsBlockedByTheInjection::IsolatedOutsideTheRootRingTable => {
                        device_map.isolate(SlotNumber(slot), 1);
                    }
                }
            }
        }
    }
    match blocked_as {
        SlotsBlockedByTheInjection::MarkedAllocated => {
            // 标成已分配挡到的整段不再全空：全空聚簇段少不少于 `N₀` 由用到这一种的用例自己核。
        }
        SlotsBlockedByTheInjection::IsolatedOutsideTheRootRingTable => {
            assert_the_empty_cluster_segments_on_every_device_meet_the_floor(
                allocator,
                "隔离着挡到只剩那么几个没挡的槽之后",
            );
        }
    }
}

/// 每块盘此刻的全空聚簇段数（合取 7 读的那一个）不少于分配器整理开关里的 `N₀`（`why` 只用在断言消息里）：
/// 抬 F 之前先算的合取 7 那一判不拦（阳性对照）。
fn assert_the_empty_cluster_segments_on_every_device_meet_the_floor(
    allocator: &PoolAllocator,
    why: &str,
) {
    let floor_in_segments = allocator
        .compaction()
        .switches
        .empty_cluster_segments_floor
        .segments();
    for device_map in &allocator.devices {
        assert!(
            device_map.empty_segments() >= floor_in_segments,
            "{why}：盘 {} 的全空聚簇段 {} 个，不少于 N₀ = {floor_in_segments}（阳性对照：抬 F 之前先算的合取 7 那一判不拦）",
            device_map.device.0,
            device_map.empty_segments()
        );
    }
}

/// 正推 `research/prompts/m3-batch10z-vw-code-r1-sonnet-model/src/vw/part10_k3.rs` 的 `vw_k3_walk_after_the_push_sweep` 那一形
/// （k = 4）：C 那一圈整个换成从现行那一版起第几次发布写的那一槽。
const PUBLISHES_UNTIL_THE_RING_SLOT_HOLDING_THE_INJECTED_ISOLATION: u64 = 4;

/// 那一槽每块盘记着几个隔离着的空闲槽：推完之后走空照旧被拒那一形。
const ISOLATED_FREE_SLOTS_THE_INJECTED_RING_SLOT_HOLDS_ON_EACH_DEVICE: u64 = 4;

/// 攻方 K1 那一形挂上之后长 13 个单元被拒一次（会话推一串抬 F、把 F 抬到上限）；C 那一圈整个换成
/// [`PUBLISHES_UNTIL_THE_RING_SLOT_HOLDING_THE_INJECTED_ISOLATION`] 次发布之后写的那一槽、每块盘记着
/// `isolated_free_slots_on_each_device` 个隔离着的空闲槽（注入级；原来那一圈置的隔离位照旧留着，推环放不出它们），再把每块盘没挡的槽只留
/// [`UNBLOCKED_SLOTS_LEFT_FOR_THE_WALK_AFTER_THE_UNLINK_INTENT_TO_BE_STUCK`] 个（多出来的照 `blocked_as` 挡）。unlink 第一个文件：意图发成、之后的走空被宽护卡住。
/// 交回挂着的池、这一份报与那一个 unlink。
fn pool_whose_unlink_is_stuck_in_the_walk_after_the_intent_behind_a_short_push(
    isolated_free_slots_on_each_device: u64,
    blocked_as: SlotsBlockedByTheInjection,
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
            blocked_as,
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
        stuck.last_ring_slot_the_push_reaches, ring_slot_holding_the_injected_isolation,
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
            SlotsBlockedByTheInjection::MarkedAllocated,
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
/// 没挡的槽隔离着挡（[`SlotsBlockedByTheInjection::IsolatedOutsideTheRootRingTable`]）：每块盘的全空聚簇段不少于 `N₀`，建 inode 被拒时
/// 抬 F 之前先算的合取 7 那一判不拦，照推抬 F、推满仍不够。
#[test]
fn an_unlink_whose_walk_after_the_intent_fits_after_the_push_lists_the_push_saying_the_intent_is_published_and_not_to_send_the_unlink_again(
) {
    let why = "推完之后走空放得下那一形";
    let (pool, stuck, unlink) =
        pool_whose_unlink_is_stuck_in_the_walk_after_the_intent_behind_a_short_push(
            ISOLATED_FREE_SLOTS_THE_INJECTED_RING_SLOT_HOLDS_ON_EACH_DEVICE_FOR_THE_WALK_TO_FIT_AFTER_THE_PUSH,
            SlotsBlockedByTheInjection::IsolatedOutsideTheRootRingTable,
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
    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_the_recorded_publish_and_the_released_slots(
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
    // 注入隔离的槽推环与走空都放不开、落不进：注入之前整段空闲的那几段一直全空，建 inode 被拒那一刻全空聚簇段同样不少于 `N₀`。
    assert_the_empty_cluster_segments_on_every_device_meet_the_floor(
        &pushed_pool.session().allocator,
        &format!("{why}：推环之后建 inode 之前"),
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

/// 攻方 K1 那一形挂上之后长 13 个单元被拒一次（会话推一串抬 F、把 F 抬到上限），照 `injection` 把每块盘没挡的槽压到几个
/// （`None` 不动），再发 `change`：被宽护卡住在这次改动本身（之前队列是空的、一版都没发成），交回报里记着的这次改动还剩什么。
fn remainder_recorded_when_the_change_itself_is_stuck(
    change: UserChange<'_>,
    injection: Option<SlotsLeftUnblockedByTheInjection>,
    why: &str,
) -> String {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
    );
    if let Some(SlotsLeftUnblockedByTheInjection {
        unblocked_slots_left,
        blocked_as,
    }) = injection
    {
        leave_only_unblocked_slots_on_each_device(
            &mut pool.session.as_mut().expect("挂着的会话").allocator,
            unblocked_slots_left,
            blocked_as,
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

/// 覆盖写、建 inode 是合取 7 判的前台写入：多出来的槽隔离着挡，每块盘的全空聚簇段不少于 `N₀`，抬 F 之前先算的合取 7 那一判不拦，
/// 走得到推满仍不够、被宽护卡住那一支。
const INJECTION_FOR_A_FOREGROUND_WRITE_ITSELF_TO_BE_STUCK: SlotsLeftUnblockedByTheInjection =
    SlotsLeftUnblockedByTheInjection {
        unblocked_slots_left: UNBLOCKED_SLOTS_LEFT_FOR_THE_CHANGE_ITSELF_TO_BE_STUCK,
        blocked_as: SlotsBlockedByTheInjection::IsolatedOutsideTheRootRingTable,
    };

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
            Some(INJECTION_FOR_A_FOREGROUND_WRITE_ITSELF_TO_BE_STUCK),
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
            Some(INJECTION_FOR_A_FOREGROUND_WRITE_ITSELF_TO_BE_STUCK),
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
            Some(SlotsLeftUnblockedByTheInjection {
                unblocked_slots_left: UNBLOCKED_SLOTS_LEFT_FOR_THE_CHANGE_ITSELF_TO_BE_STUCK,
                blocked_as: SlotsBlockedByTheInjection::MarkedAllocated,
            }),
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

/// 覆盖写碰上全空聚簇段少于 `N₀`（D16（发布语义） 已定项 10 射程「抬 F 之前先算」合取 7 那一形）：与
/// [`an_overwrite_stuck_behind_the_wide_protection_keeps_its_whole_content_and_write_time`] 同一个池、同一次覆盖写，多出来的槽标成已分配
/// （[`SlotsBlockedByTheInjection::MarkedAllocated`]），每块盘的全空聚簇段掉到 `N₀` 以下。这次调用还一串没推、被拒是空间不够、
/// 被拒的是合取 7 判的前台写入：推串之前合取 7 那一判一串不推就拒成 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，
/// 不点名上界小于需求的盘、带合取 7 那一份（`N₀` 与每块盘的全空段数），一版都没发；两块盘逐字节不变，会话的现行那一版不动。
/// 判别力：推串之前不判合取 7 时照推、F 已在上限，拒成推满仍不够、报被宽护卡住。
#[test]
fn an_overwrite_while_empty_cluster_segments_are_below_the_floor_is_refused_before_any_series_with_the_devices_unchanged(
) {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted_and_stuck_once_by_a_write_grown_by(
        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
    );
    let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
    leave_only_unblocked_slots_on_each_device(
        allocator,
        UNBLOCKED_SLOTS_LEFT_FOR_THE_CHANGE_ITSELF_TO_BE_STUCK,
        SlotsBlockedByTheInjection::MarkedAllocated,
    );
    let floor_in_segments = allocator
        .compaction()
        .switches
        .empty_cluster_segments_floor
        .segments();
    let empty_cluster_segments_on_each_device: Vec<DeviceWithEmptyClusterSegmentsBelowTheFloor> =
        allocator
            .devices
            .iter()
            .map(|device_map| DeviceWithEmptyClusterSegmentsBelowTheFloor {
                device: device_map.device,
                empty_cluster_segments: device_map.empty_segments(),
            })
            .collect();
    assert!(
        empty_cluster_segments_on_each_device
            .iter()
            .all(|device| device.empty_cluster_segments < floor_in_segments),
        "阳性对照：标成已分配挡到每一整段，每块盘的全空聚簇段都少于 N₀ = {floor_in_segments}：{empty_cluster_segments_on_each_device:?}"
    );
    let image_before_the_overwrite = pool.image();
    let checkpoint_txg_before_the_overwrite = pool.session().current.root().checkpoint_txg;
    let content = content_of(
        2999,
        WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
    );
    let refusal = pool
        .publish(
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds:
                    WRITE_TIME_SECONDS_OF_THE_CHANGE_STUCK_BEHIND_THE_WIDE_PROTECTION,
            }),
            None,
        )
        .expect_err("全空聚簇段少于 N₀：覆盖写被拒");
    let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) = refusal
    else {
        panic!(
            "这次调用还一串没推、被拒的是前台写入而全空聚簇段少于 N₀：推串之前合取 7 那一判拒成推抬 F 全放出来也放不下，而交回 {refusal:?}"
        );
    };
    let NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {
        devices_short,
        empty_cluster_segments_below_the_floor,
        last_refusal,
        floor_raises,
        refusals_that_pushed_the_floor_raises,
        versions_published,
    } = *no_space;
    assert!(
        devices_short.is_empty(),
        "判出的是合取 7 那一判：没有上界小于需求的盘：{devices_short:?}"
    );
    assert_eq!(
        empty_cluster_segments_below_the_floor,
        Some(EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
            floor_in_segments,
            short_devices: empty_cluster_segments_on_each_device,
        }),
        "合取 7 那一份：N₀ 与每块盘写之前那一刻的全空段数"
    );
    assert!(
        refusal_is_short_of_space(&last_refusal),
        "判出它的那一次被拒是空间不够的那两种：{last_refusal:?}"
    );
    assert!(
        floor_raises.is_empty()
            && refusals_that_pushed_the_floor_raises.is_empty()
            && versions_published.is_empty(),
        "头一次被拒就判到这里：一串抬 F 都没推、一版都没发"
    );
    assert!(
        pool.image() == image_before_the_overwrite,
        "头一次被拒就判出：两块盘逐字节不变"
    );
    assert_eq!(
        pool.session().current.root().checkpoint_txg,
        checkpoint_txg_before_the_overwrite,
        "一版都没发：会话的现行那一版不动"
    );
}
