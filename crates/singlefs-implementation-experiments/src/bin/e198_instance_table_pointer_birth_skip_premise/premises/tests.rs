use super::*;
use crate::helpers_shared_by_the_tests::{key, ledger_version};

#[test]
fn premise_counts_each_clause_and_reports_instance_one_separately() {
    let mut ledger = Ledger::default();
    let entries = [
        (key(0, 0), PublishKind::MakeFilesystem, 0),
        (key(1, 1), PublishKind::RowPublish, 0),
        (key(1, 2), PublishKind::WarmUp, 0),
        (key(1, 3), PublishKind::FirstFile, 0),
        (key(2, 4), PublishKind::RowPublish, 4),
        (key(2, 5), PublishKind::WarmUp, 4),
        (key(2, 6), PublishKind::Sweep, 6),
        (key(2, 7), PublishKind::Overwrite, 6),
        (key(3, 8), PublishKind::RowPublish, 8),
        (key(3, 9), PublishKind::Overwrite, 9),
        (key(3, 10), PublishKind::Overwrite, 11),
    ];
    for (order, (version, kind, birth)) in entries.into_iter().enumerate() {
        ledger.versions.insert(
            version,
            ledger_version(None, kind, birth, true, false, order),
        );
    }
    let counts = premise_counts(&ledger);
    assert_eq!(
        counts.instance_one_first_version_mismatches, 1,
        "实例 1 的第一版照抄 mkfs 那一份指针（PC-P1）"
    );
    assert_eq!(
        counts.first_version_violations.len(),
        0,
        "实例 2、3 的写行那一版出生 txg 等于自己的 txg"
    );
    assert_eq!(counts.sweep_pointer_changes, 1, "2@6 是清扫换指针（PC-P2）");
    assert_eq!(
        counts.adjacent_version_violations.len(),
        2,
        "3@9 与 3@10 两次非清扫换指针"
    );
    assert_eq!(
        counts.birth_after_the_txg_violations.len(),
        1,
        "3@10 的出生 txg 11 大于自己的 txg"
    );
}
