use super::*;
use crate::helpers_shared_by_the_tests::{key, ledger_version};

/// 0@0 → 1@4 → 1@5（被施加、根槽没落）→ 2@6 → 2@7 → 2@8（崩、没被施加）；3@9 的父版本是 2@7（取号越过 2@8）。
fn chain_ledger() -> Ledger {
    let mut ledger = Ledger::default();
    let entries = [
        (key(0, 0), None, PublishKind::MakeFilesystem, true, false),
        (
            key(1, 4),
            Some(key(0, 0)),
            PublishKind::Overwrite,
            true,
            false,
        ),
        (
            key(1, 5),
            Some(key(1, 4)),
            PublishKind::Overwrite,
            false,
            true,
        ),
        (
            key(2, 6),
            Some(key(1, 5)),
            PublishKind::RowPublish,
            true,
            false,
        ),
        (key(2, 7), Some(key(2, 6)), PublishKind::WarmUp, true, false),
        (
            key(2, 8),
            Some(key(2, 7)),
            PublishKind::Overwrite,
            false,
            false,
        ),
        (
            key(3, 9),
            Some(key(2, 7)),
            PublishKind::RowPublish,
            true,
            false,
        ),
    ];
    for (order, (version, parent, kind, landed, applied)) in entries.into_iter().enumerate() {
        ledger.versions.insert(
            version,
            ledger_version(parent, kind, 0, landed, applied, order),
        );
    }
    ledger
}

#[test]
fn truth_reads_the_parent_chain_over_versions_that_existed() {
    let ledger = chain_ledger();
    assert_eq!(
        truth_of_the_hole_pair(&ledger, &[key(1, 5)], key(2, 6)),
        Truth::DirectlyBridged,
        "父版本就在 R⁻ 那一组里"
    );
    assert_eq!(
        truth_of_the_hole_pair(&ledger, &[key(1, 4)], key(2, 6)),
        Truth::ExistedVersionsInBetween,
        "中间隔着被施加出来的 1@5"
    );
    assert_eq!(
        truth_of_the_hole_pair(&ledger, &[key(2, 8)], key(3, 9)),
        Truth::PreviousNotOnTheChain,
        "崩了的 2@8 不在 3@9 的祖先链上"
    );
    assert_eq!(
        truth_of_the_hole_pair(&ledger, &[key(2, 7)], key(3, 9)),
        Truth::DirectlyBridged,
        "取号越过之后直接接在 2@7 上"
    );
    assert_eq!(
        truth_of_the_hole_pair(&ledger, &[key(2, 7)], key(4, 10)),
        Truth::VersionMissingFromTheLedger,
        "不在账本里的版本单列"
    );
}
