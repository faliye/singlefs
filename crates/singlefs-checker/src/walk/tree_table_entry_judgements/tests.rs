use super::*;

/// 树 ID 依次给出的几条树表条目，每条只有 `entry_bytes` 字节（树表单元自述的条目宽窄过字段表）。
fn narrow_tree_table_entries(tree_identifiers: &[u64], entry_bytes: usize) -> Vec<Vec<u8>> {
    tree_identifiers
        .iter()
        .map(|tree| {
            let mut entry = vec![0u8; entry_bytes];
            entry[..8].copy_from_slice(&tree.to_le_bytes());
            entry
        })
        .collect()
}

/// I-9.27：条目宽不到头 ID 那 8 字节的树表，头 ID 读不出——报部分没判，不报不适用（射程里的对象在镜像上、配不上字段表；
/// 树表条目那一支走读断了，那一版的 I-7.4、I-4.8 红）。
#[test]
fn tree_table_entries_too_narrow_for_the_head_identifier_leave_the_head_identifier_invariant_partially_judged(
) {
    let mut judgements = Judgements::default();
    judge_tree_table_entry_head_identifiers(
        &mut judgements,
        &narrow_tree_table_entries(&[3, 5], TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 4),
        "根（实例 1、txg 3）",
    );
    let verdict = verdict_of(judgements, HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES);
    assert!(
        matches!(&verdict, InvariantVerdict::PartiallyJudged(detail) if detail.contains("根（实例 1、txg 3）的树表单元自述的条目宽不到头 ID")),
        "条目宽不到头 ID：I-9.27 应当报部分没判、点名那一版，实际 {verdict:?}"
    );
}

/// I-9.16：条目宽不到种类那 2 字节、树 ID 照样升序的树表——① 判过成立，② 读不出种类，整条报部分没判。
#[test]
fn tree_table_entries_too_narrow_for_the_kind_leave_the_tree_table_ordering_invariant_partially_judged(
) {
    let mut judgements = Judgements::default();
    judge_tree_table_entries_ordering(
        &mut judgements,
        &narrow_tree_table_entries(&[3, 5], TREE_TABLE_ENTRY_KIND_OFFSET + 1),
        "根（实例 1、txg 3）",
    );
    let verdict = verdict_of(judgements, "I-9.16");
    assert!(
        matches!(&verdict, InvariantVerdict::PartiallyJudged(detail) if detail.contains("条目宽不到种类那 2 字节")),
        "条目宽不到种类：I-9.16 应当报部分没判，实际 {verdict:?}"
    );
}

/// 一张树表上 I-9.27 的判定。
fn head_identifier_verdict_of(tree_table_entries: &[Vec<u8>]) -> InvariantVerdict {
    let mut judgements = Judgements::default();
    judge_tree_table_entry_head_identifiers(&mut judgements, tree_table_entries, "合成的那一版");
    judgements
        .into_report()
        .into_iter()
        .find(|(invariant, _)| *invariant == HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES)
        .expect("I-9.27 在清单里")
        .1
}

/// I-9.27 多于一条种类 2 条目时放宽成「等于其中某一条的树 ID」（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.27 那一行，收严留给多头那一批）：
/// 表里两条 inode 树（65537、65549），extent 树条目的头 ID 写第二条的树 ID 65549——成立；只认第一条的读法在这里误红。
#[test]
fn an_extent_entry_naming_the_second_of_two_inode_tree_entries_holds_the_head_identifier_relation()
{
    let tree_table_entries = [
        tree_table_entry_with_head_identifier(65536, TREE_KIND_EXTENT, 65549),
        tree_table_entry_with_head_identifier(65537, TREE_KIND_INODE, 65537),
        tree_table_entry_with_head_identifier(65549, TREE_KIND_INODE, 65549),
    ];
    assert_eq!(
        head_identifier_verdict_of(&tree_table_entries),
        InvariantVerdict::Holds,
        "extent 树条目的头 ID 等于两条 inode 树条目里第二条的树 ID：放宽之后成立"
    );
}

/// 同一张两条 inode 树的表，extent 树条目的头 ID 写成两条都不是的 65550：放宽之后照样红。
#[test]
fn an_extent_entry_naming_neither_of_two_inode_tree_entries_violates_the_head_identifier_relation()
{
    let tree_table_entries = [
        tree_table_entry_with_head_identifier(65536, TREE_KIND_EXTENT, 65550),
        tree_table_entry_with_head_identifier(65537, TREE_KIND_INODE, 65537),
        tree_table_entry_with_head_identifier(65549, TREE_KIND_INODE, 65549),
    ];
    assert!(
        matches!(
            head_identifier_verdict_of(&tree_table_entries),
            InvariantVerdict::Violated(_)
        ),
        "extent 树条目的头 ID 不等于表里任何一条 inode 树条目的树 ID：判红"
    );
}
