use super::*;
use crate::tests::bare_node;

/// PC-Nw：手写一段假账（两次回退，两段被抛弃根都读得出），不碰 `crates/`，只证计数函数会数到 2。
#[test]
fn pc_nw_counts_two_live_rollback_events_from_a_hand_written_ledger() {
    let events = vec![
        RollbackEvent {
            abandoned: BTreeSet::from([(1u32, 5u64), (1, 6)]),
        },
        RollbackEvent {
            abandoned: BTreeSet::from([(2u32, 9u64)]),
        },
    ];
    let readable: BTreeSet<TimelineRoot> = BTreeSet::from([(1, 6), (2, 9), (3, 20)]);
    let (events_live, roots_live) = count_live_rollback_events(&events, &readable);
    assert_eq!(
        events_live, 2,
        "两次回退各有至少一个被抛弃根仍可读，应数出 2"
    );
    assert_eq!(
        roots_live, 2,
        "读得出的被抛弃根去重后是 (1,6) 与 (2,9) 两条"
    );
}

/// 对照：一次回退的被抛弃根全部不可读时不计入 events_live。
#[test]
fn pc_nw_does_not_count_a_rollback_event_whose_abandoned_roots_are_all_unreadable() {
    let events = vec![RollbackEvent {
        abandoned: BTreeSet::from([(1u32, 5u64)]),
    }];
    let readable: BTreeSet<TimelineRoot> = BTreeSet::from([(9, 99)]);
    let (events_live, roots_live) = count_live_rollback_events(&events, &readable);
    assert_eq!(events_live, 0);
    assert_eq!(roots_live, 0);
}

/// 回退之前的账：(1,1) 内容 older → (1,2) 内容 newer。
fn ledger_before_a_rollback() -> SimNode {
    bare_node(
        vec![(1, 1), (1, 2)],
        BTreeMap::from([
            ((1, 1), Some(b"older".to_vec())),
            ((1, 2), Some(b"newer".to_vec())),
        ]),
        Vec::new(),
    )
}

/// 回退到 (1,1)：可写挂载写行 (2,3)、暖机 (2,4)，回退那次发布 (2,5)。
fn ledger_after_rolling_back_the_hand_written_ledger() -> LedgerAfterRollback {
    ledger_after_a_rollback_published_forward(
        &ledger_before_a_rollback(),
        (1, 1),
        (1, 2),
        &[(2, 3), (2, 4)],
        (2, 5),
    )
    .expect("目标在时间线上、挂载所选根是时间线尾，账本该收")
}

#[test]
fn rollback_ledger_keeps_the_whole_timeline_and_appends_the_row_warm_up_and_rollback_roots() {
    assert_eq!(
        ledger_after_rolling_back_the_hand_written_ledger().timeline,
        vec![(1, 1), (1, 2), (2, 3), (2, 4), (2, 5)]
    );
}

#[test]
fn rollback_ledger_records_the_row_and_warm_up_roots_with_the_content_before_the_rollback() {
    let ledger = ledger_after_rolling_back_the_hand_written_ledger();
    let content_before_the_rollback = Some(b"newer".to_vec());
    assert_eq!(
        ledger.content_by_root.get(&(2, 3)),
        Some(&content_before_the_rollback)
    );
    assert_eq!(
        ledger.content_by_root.get(&(2, 4)),
        Some(&content_before_the_rollback)
    );
}

#[test]
fn rollback_ledger_records_the_rollback_root_with_the_content_of_the_target() {
    let ledger = ledger_after_rolling_back_the_hand_written_ledger();
    assert_eq!(
        ledger.content_by_root.get(&(2, 5)),
        Some(&Some(b"older".to_vec()))
    );
}

#[test]
fn rollback_ledger_refuses_a_target_that_is_not_on_the_timeline() {
    let refusal = ledger_after_a_rollback_published_forward(
        &ledger_before_a_rollback(),
        (9, 9),
        (1, 2),
        &[(2, 3)],
        (2, 4),
    );
    assert!(refusal.is_err(), "目标不在账上该走 S5");
}

#[test]
fn rollback_ledger_refuses_a_mount_that_did_not_choose_the_tip() {
    let refusal = ledger_after_a_rollback_published_forward(
        &ledger_before_a_rollback(),
        (1, 1),
        (1, 1),
        &[(2, 3)],
        (2, 4),
    );
    assert!(
        refusal.is_err(),
        "不注入故障时挂载所选根不是时间线尾该走 S5"
    );
}
