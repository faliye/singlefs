use super::*;
use crate::helpers_shared_by_the_tests::{key, ring_version};

fn row(
    instance: u32,
    published_checkpoint_txg: u64,
    applied_transaction_high_water_mark: u64,
) -> RowOfTheNewestTable {
    RowOfTheNewestTable {
        instance,
        published_checkpoint_txg,
        applied_transaction_high_water_mark,
    }
}

#[test]
fn today_arm_answers_its_known_answer_table() {
    let one = BTreeSet::from([1]);
    let bridged_rows = [row(1, 6, 5), row(2, 0, 0)];
    assert_eq!(
        hole_class_today(6, &one, 8, 3, &bridged_rows),
        HoleClass::BridgedByTheRows,
        "行 (1, 6) 加中间实例 (2, 0, 0) 接得上"
    );
    assert_eq!(
        hole_class_today(6, &one, 8, 3, &[row(1, 6, 5)]),
        HoleClass::NotBridgedByTheRows,
        "中间实例 2 没有行"
    );
    assert_eq!(
        hole_class_today(6, &one, 8, 3, &[row(1, 6, 5), row(2, 7, 0)]),
        HoleClass::NotBridgedByTheRows,
        "中间实例发过东西"
    );
    assert_eq!(
        hole_class_today(6, &one, 8, 3, &[row(1, 5, 5), row(2, 0, 0)]),
        HoleClass::NotBridgedByTheRows,
        "行的 T 不等于 R⁻.txg"
    );
    assert_eq!(
        hole_class_today(6, &one, 8, 1, &[row(1, 6, 5)]),
        HoleClass::NotBridgedByTheRows,
        "R 的实例不比 R⁻ 的大"
    );
    assert_eq!(
        hole_class_today(6, &one, 7, 3, &bridged_rows),
        HoleClass::NoHole,
        "R.txg = R⁻.txg + 1 没有洞"
    );
    assert_eq!(
        hole_class_today(6, &BTreeSet::from([1, 2]), 9, 3, &[row(2, 6, 3)]),
        HoleClass::BridgedByTheRows,
        "R⁻ 那一组里任一个实例接得上就算"
    );
}

#[test]
fn with_pointer_birth_arm_answers_its_known_answer_table() {
    let previous = [ring_version(1, 6, Some(0))];
    assert!(
        bridged_with_the_pointer_birth(
            HoleClass::BridgedByTheRows,
            &previous,
            &ring_version(3, 8, Some(8))
        ),
        "出生 txg 等于 R.txg"
    );
    assert!(
        !bridged_with_the_pointer_birth(
            HoleClass::BridgedByTheRows,
            &previous,
            &ring_version(3, 8, Some(7))
        ),
        "出生 txg 小于 R.txg"
    );
    assert!(
        !bridged_with_the_pointer_birth(
            HoleClass::BridgedByTheRows,
            &previous,
            &ring_version(3, 8, None)
        ),
        "指针读不出按不认作取号跳过"
    );
    assert!(
        !bridged_with_the_pointer_birth(
            HoleClass::NotBridgedByTheRows,
            &previous,
            &ring_version(3, 8, Some(8))
        ),
        "行接不上"
    );
}

fn ring_root(region: u64, instance: u32, txg: u64, rollback_floor: u64) -> RootOnTheRing {
    RootOnTheRing {
        region,
        key: key(instance, txg),
        rollback_floor,
        pointer_birth: PointerBirth::Readable(0),
    }
}

fn record(
    instance: u32,
    txg: u64,
    is_commit: bool,
    is_the_last_record_of_the_publish: bool,
    birth: u64,
) -> RecordOnTheRing {
    RecordOnTheRing {
        key: key(instance, txg),
        is_commit,
        is_the_last_record_of_the_publish,
        pointer_birth: PointerBirth::Readable(birth),
    }
}

#[test]
fn versions_applied_only_by_records_need_all_four_conditions() {
    let rows = [row(1, 10, 9)];
    let roots = [ring_root(0, 1, 6, 0), ring_root(2, 1, 8, 0)];
    let records = [
        record(1, 5, true, true, 5),
        record(1, 7, true, true, 7),
        record(1, 8, true, true, 8),
        record(1, 9, true, false, 3),
        record(1, 9, true, true, 9),
        record(1, 10, false, true, 10),
        record(1, 11, true, true, 11),
        record(2, 9, true, true, 9),
    ];
    let applied = versions_applied_only_by_records(&records, &roots, &rows, 0);
    let expected = vec![
        VersionOnTheImage {
            key: key(1, 7),
            pointer_birth: PointerBirth::Readable(7),
        },
        VersionOnTheImage {
            key: key(1, 9),
            pointer_birth: PointerBirth::Readable(9),
        },
    ];
    assert_eq!(applied, expected, "1@5 环里最旧有效根不比它旧、1@8 根槽读得出、1@10 没提交、1@11 超过行的 T、实例 2 没有行；1@9 取带末条标志那一条");
    let applied_above_the_floor = versions_applied_only_by_records(&records, &roots, &rows, 8);
    assert_eq!(
        applied_above_the_floor
            .iter()
            .map(|version| version.key)
            .collect::<Vec<_>>(),
        vec![key(1, 9)],
        "F = 8 时 1@7 在 F 之下"
    );
}

#[test]
fn candidate_set_keeps_the_newest_and_drops_abandoned_and_below_the_floor() {
    let rows = [row(1, 6, 0), row(2, 7, 0)];
    let roots = [
        ring_root(0, 1, 3, 0),
        ring_root(1, 1, 4, 0),
        ring_root(0, 1, 6, 0),
        ring_root(2, 2, 8, 0),
        ring_root(0, 3, 9, 0),
    ];
    assert_eq!(newest_root_index(&roots), Some(4));
    assert_eq!(
        candidate_root_indexes(&roots, &rows, 4, 4),
        vec![1, 2, 4],
        "1@3 在 F 之下、2@8 被行 (2, 7) 判抛弃"
    );
    let abandoned_newest = [ring_root(0, 1, 3, 0), ring_root(1, 2, 8, 0)];
    assert_eq!(
        candidate_root_indexes(&abandoned_newest, &rows, 0, 1),
        vec![0, 1],
        "最新根恒收"
    );
}

#[test]
fn effective_floor_takes_the_highest_of_the_roots_and_the_configuration() {
    let rows = [row(2, 7, 0)];
    let roots = [
        ring_root(0, 1, 6, 2),
        ring_root(1, 1, 5, 3),
        ring_root(2, 2, 8, 5),
    ];
    assert_eq!(
        effective_rollback_floor(&roots, &rows, &[4]),
        5,
        "被抛弃的 2@8 带的 F 也算进来"
    );
    assert_eq!(
        effective_rollback_floor(&roots, &rows, &[7]),
        7,
        "系统配置带的 F 更高时取它"
    );
    assert_eq!(
        effective_rollback_floor(&roots[..2], &rows, &[]),
        3,
        "每块盘最新有效根带的 F 取最大"
    );
}

#[test]
fn hole_pairs_pair_every_version_of_a_txg_with_the_whole_group_before_it() {
    let rows = [row(1, 4, 0), row(2, 0, 0), row(3, 6, 1)];
    let versions = [
        ring_version(0, 0, Some(0)),
        ring_version(1, 4, Some(0)),
        ring_version(2, 6, Some(6)),
        ring_version(3, 6, Some(5)),
        ring_version(4, 9, Some(9)),
        ring_version(4, 10, Some(9)),
    ];
    let pairs = hole_pairs_of(&versions, &rows);
    let summary: Vec<(u64, Vec<VersionKey>, VersionKey, bool, bool)> = pairs
        .iter()
        .map(|pair| {
            (
                pair.previous_txg,
                pair.previous_group.clone(),
                pair.version.key,
                pair.bridged_today,
                pair.bridged_with_the_pointer_birth,
            )
        })
        .collect();
    assert_eq!(
        summary,
        vec![
            (0, vec![key(0, 0)], key(1, 4), false, false),
            (4, vec![key(1, 4)], key(2, 6), true, true),
            (4, vec![key(1, 4)], key(3, 6), true, false),
            (6, vec![key(2, 6), key(3, 6)], key(4, 9), true, true),
        ],
        "同 txg 两版并成一组、R 侧逐条各算一对；9 → 10 没有洞"
    );
}
