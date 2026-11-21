//! 一版树表的条目：I-9.16（树表条目按树 ID 严格升序且合发号次序） 与 I-9.27（树表条目的头 ID 取值）。

use super::*;

/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备身份 4, 16 KiB 槽号 6) + value (跨度段 2，最高位是已释放标志、
/// 次高位是隔离记号, 分配代或释放代 8)。checker 按字段表自己解一份，不与实现共用代码（D13（验证路线） 已定项 5）。
pub(super) const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
/// 跨度段次高位：隔离记号（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」，落点甲；D3（空间分配） 已定项 7 跨度段那一行）。
pub(super) const ALLOCATION_RECORD_QUARANTINE_MARK: u16 = 0x4000;
pub(super) const ALLOCATION_RECORD_SPAN_FIELD_OFFSET: usize = 10;
pub(super) const ALLOCATION_RECORD_GENERATION_OFFSET: usize = 12;
/// 树表条目 200（D8（核心索引结构） 已定项 8）：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 +
/// `previous_snapshot_txg` 8 + **诞生 txg 8** + 头 ID 8 + 预留 76。
pub(super) const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;
/// 树表条目里种类 2 字节的偏移（树 ID 8 + 条目长度 2 之后）。
pub(super) const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 8 + 2;
/// 树表条目里头 ID 8 字节的偏移（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8，
/// D8（核心索引结构） 已定项 8 的字段表），I-9.27 判它。
pub(super) const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8 + 8;
/// 一个数据单元占几个 16 KiB 槽（32768 字节，起点 32768 对齐 ⇒ 起点槽号是它的整数倍）：I-5.5 按槽对找落点。
pub(super) const DATA_UNIT_SLOTS: u64 = DATA_UNIT_BYTES / SLOT_BYTES;

/// I-9.27：树表条目的头 ID 按种类该等于什么（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.27 那一行；D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」
/// 句末的关系判据：判的是关系，不拿字面 65537 比）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeadIdentifierOfATreeTableEntry {
    /// 种类 2（inode 树）：等于这条条目自己的树 ID。
    ItsOwnTreeIdentifier,
    /// 种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）、13（退役树）：等于同一张树表里种类 2 条目的树 ID；
    /// 种类 2 条目多于一条时放宽成「等于其中某一条的」（收严留给多头那一批），一条都没有时判红。
    TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,
    /// 种类 3、4、6、7、8、11、12（无归属，D5（快照 / 空间记账机制） 已定项 9）：等于 0。
    Zero,
    /// 种类 15（快照 extent 根）：等于同一张树表里某一条种类 14（快照 inode 根）条目的树 ID（配对的那一条；一一配对由 I-9.16 ③ 判），
    /// 一条种类 14 都没有时判红（D5（快照 / 空间记账机制） 已定项 9 射程）。
    TreeIdentifierOfASnapshotInodeRootEntryOfTheSameTable,
    /// 别的种类码：那一行不判。
    NotJudged,
}

impl HeadIdentifierOfATreeTableEntry {
    const fn of_tree_kind(kind: u16) -> Self {
        match kind {
            TREE_KIND_INODE => Self::ItsOwnTreeIdentifier,
            // 码 14 的头 ID = 被捕获的活头 inode 树的树 ID（D5（快照 / 空间记账机制） 已定项 9 射程），与 extent 树那几种同一判。
            TREE_KIND_EXTENT
            | TREE_KIND_DELETED_INODES
            | TREE_KIND_TOMBSTONE
            | TREE_KIND_RETIREMENT
            | TREE_KIND_SNAPSHOT_INODE_ROOT => Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,
            TREE_KIND_SNAPSHOT_EXTENT_ROOT => {
                Self::TreeIdentifierOfASnapshotInodeRootEntryOfTheSameTable
            }
            TREE_KIND_ALLOCATION
            | TREE_KIND_ACCOUNTING
            | TREE_KIND_LIVELIST
            | TREE_KIND_SPARSE_SIDE_TABLE
            | TREE_KIND_DEADLIST
            | TREE_KIND_LINEAGE
            | TREE_KIND_LOGGED_OPERATIONS => Self::Zero,
            _ => Self::NotJudged,
        }
    }

    /// 判红时写进第一处违例的那半句：这一条的头 ID 该等于什么。
    fn expectation(
        self,
        tree: u64,
        inode_tree_identifiers: &[u64],
        snapshot_inode_root_tree_identifiers: &[u64],
    ) -> String {
        match self {
            Self::ItsOwnTreeIdentifier => format!("inode 树条目的头 ID 该等于它自己的树 ID {tree}"),
            Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable => format!(
                "该等于同一张树表里种类 2 条目的树 ID（这张表里的种类 2 条目：{inode_tree_identifiers:?}）"
            ),
            Self::Zero => "该等于 0".to_string(),
            Self::TreeIdentifierOfASnapshotInodeRootEntryOfTheSameTable => format!(
                "该等于同一张树表里一条种类 14 条目的树 ID（这张表里的种类 14 条目：{snapshot_inode_root_tree_identifiers:?}）"
            ),
            Self::NotJudged => "这一种不判".to_string(),
        }
    }
}

/// I-9.16（树表条目按树 ID 严格升序且合发号次序）：一版的树表判一次，四道任一道不成立即判红，第一处违例写明是哪一版、哪两条（下标从 0 数）
/// 或哪一种。
/// ① 盘上相邻两条的树 ID（条目打头 8 字节）严格升序；② 按发号次序（[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]），
/// 树表里有的种类相邻两棵的树 ID 严格升序；③ 同一张树表里每一种种类码（登记过的与没登记的都算）至多一条；④ 树表非空时
/// 进树表的十二种（[`TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`]）各恰好一条
/// （「种类 1 不止一条；种类 3 不是恰好一条」那一行；
/// 十一种取自「I-9.16 ④ 放过缺种类的树表（C1）」那一行）。
/// 0 条的树表四道都成立，1 条的 ① ② ③ 成立。checker 自己按字段表切一份，不与实现共用代码（D13（验证路线） 已定项 5；
/// 实现那一侧是 `crates/singlefs-core/src/recovery.rs` 的 `tree_table_entries_each_kind_at_most_once` 与重建一版时缺了有根的树报的
/// 「树表里缺一棵有根的树」）。③ ④ 先于 ② 判：同一张表两道都不成立时第一处违例写 ③ ④ 那一道。
/// ① 与 I-1.1 在树表上判的「key 严格递增」是同一个谓词（树表的 key 就是树 ID）：只违反 ① 的镜像两条一起红，只违反 ② 的只红这一条。
/// 条目宽不到种类那 2 字节（盘上字段，`index_node_view` 只保证 ≥ key 宽 8）时 ② 读不出种类：只判 ①，② 记成射程里判不了的那一部分。
pub(super) fn judge_tree_table_entries_ordering(
    judgements: &mut Judgements,
    tree_table_entries: &[Vec<u8>],
    version_name: &str,
) {
    let tree_identifier_of = |entry: &Vec<u8>| read_u64(entry, 0);
    let first_pair_out_of_the_on_disk_order = tree_table_entries
        .windows(2)
        .enumerate()
        .find(|(_, pair)| tree_identifier_of(&pair[0]) >= tree_identifier_of(&pair[1]));
    if let Some((index, pair)) = first_pair_out_of_the_on_disk_order {
        judgements.judge("I-9.16", false, || {
            format!(
                "{version_name}的树表：第 {index} 条（树 ID {}）与第 {} 条（树 ID {}）不按树 ID 严格升序（① 盘上次序）",
                tree_identifier_of(&pair[0]),
                index + 1,
                tree_identifier_of(&pair[1])
            )
        });
        return;
    }
    let kinds_are_readable = tree_table_entries
        .iter()
        .all(|entry| entry.len() >= TREE_TABLE_ENTRY_KIND_OFFSET + 2);
    if !kinds_are_readable {
        // 条目宽不到字段表：树表条目那一支走读断了，那一版的 I-7.4、I-4.8 红（`Walk::walk_tree_table_entry`），这里报部分没判（C701（别的判法静默跳过读不出的部分））。
        judgements.part_of_the_range_left_unjudged(
            "I-9.16",
            format!("{version_name}的树表单元自述的条目宽不到种类那 2 字节：② 发号次序读不出种类，只判了 ① 盘上次序"),
        );
        judgements.judge("I-9.16", true, String::new);
        return;
    }
    judge_tree_table_entry_kind_counts(judgements, tree_table_entries, version_name);
    // (种类, 下标, 树 ID)，按发号次序；树表里没有的种类跳过。同一种有两条时取先出现的那条（那一张表 ③ 已判红）。
    let entries_in_the_issuing_order: Vec<(u16, usize, u64)> =
        TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER
            .iter()
            .filter_map(|kind| {
                tree_table_entries
                    .iter()
                    .enumerate()
                    .find(|(_, entry)| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == *kind)
                    .map(|(index, entry)| (*kind, index, tree_identifier_of(entry)))
            })
            .collect();
    let first_pair_out_of_the_issuing_order = entries_in_the_issuing_order
        .windows(2)
        .find(|pair| pair[0].2 >= pair[1].2);
    judgements.judge("I-9.16", first_pair_out_of_the_issuing_order.is_none(), || {
        let pair = first_pair_out_of_the_issuing_order.expect("判红时找到过那一对");
        format!(
            "{version_name}的树表：种类 {} 那条（第 {} 条，树 ID {}）与发号次序上的下一棵、种类 {} 那条（第 {} 条，树 ID {}）不按树 ID 严格升序（② 发号次序）",
            pair[0].0, pair[0].1, pair[0].2, pair[1].0, pair[1].1, pair[1].2
        )
    });
}

/// I-9.16 ④（[`judge_tree_table_entry_kind_counts`]）：树表非空时各要恰好一条的种类，就是进树表的那十二种
/// （[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]，同一份，不另抄）。读者重建一版时十三棵树的号逐棵从树表取：
/// 十二棵按种类找条目、取条目里的树 ID，中央映射树不进树表、取根记录里它那条根指针的出生树；缺哪一种都重建不出这一版
/// （实现那一侧是 `crates/singlefs-core/src/recovery.rs` 的 `rebuild_version` 逐种取号时报的「树表里没有那棵树」）。
/// 只读前四种根指针的那几行读法前提读窄了。
const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 12] =
    TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER;

/// I-9.16 的 ③ ④（[`judge_tree_table_entries_ordering`]）：③ 每一种种类码至多一条；④ 树表非空时
/// [`TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`] 各恰好一条。条目都读得出种类那 2 字节（调用方判过）。
/// 0 条的表不判（④ 的前提不成立，③ 没有对象）。第一处违例写明是哪一版、哪一种、几条。
fn judge_tree_table_entry_kind_counts(
    judgements: &mut Judgements,
    tree_table_entries: &[Vec<u8>],
    version_name: &str,
) {
    if tree_table_entries.is_empty() {
        return;
    }
    let mut entries_of_each_kind: BTreeMap<u16, usize> = BTreeMap::new();
    for entry in tree_table_entries {
        *entries_of_each_kind
            .entry(read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET))
            .or_insert(0) += 1;
    }
    // 码 14、15 一个快照一对、可以有多条：不按「至多一条」判，改按头 ID 一一配对判（下面 `judge_snapshot_tree_table_entries_paired_one_to_one`）。
    let first_kind_carried_more_than_once = entries_of_each_kind
        .iter()
        .filter(|(kind, _)| {
            **kind != TREE_KIND_SNAPSHOT_INODE_ROOT && **kind != TREE_KIND_SNAPSHOT_EXTENT_ROOT
        })
        .find(|(_, entries)| **entries > 1)
        .map(|(kind, entries)| (*kind, *entries));
    judgements.judge(
        "I-9.16",
        first_kind_carried_more_than_once.is_none(),
        || {
            let (kind, entries) = first_kind_carried_more_than_once.expect("判红时找到过那一种");
            format!("{version_name}的树表：种类 {kind} 有 {entries} 条（③ 每种至多一条）")
        },
    );
    judge_snapshot_tree_table_entries_paired_one_to_one(
        judgements,
        tree_table_entries,
        version_name,
    );
    let first_rooted_kind_missing = TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES
        .iter()
        .copied()
        .find(|kind| !entries_of_each_kind.contains_key(kind));
    judgements.judge("I-9.16", first_rooted_kind_missing.is_none(), || {
        let kind = first_rooted_kind_missing.expect("判红时找到过那一种");
        format!(
            "{version_name}的树表有 {} 条，种类 {kind} 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
            tree_table_entries.len()
        )
    });
}

/// I-9.16 ③ 的快照那一格（一个快照两条，码 14 快照 inode 根与
/// 码 15 快照 extent 根，码 15 的头 ID = 配对的码 14 的树 ID）：码 14 与码 15 条数相等，每条码 15 的头 ID 恰是一条码 14 的树 ID、
/// 且那条码 14 只被这一条码 15 配上（两样合起来是一一配对）。树表里一条码 14、15 都没有时不判（没有对象，评估次数不变）。
/// 条目都读得出种类那 2 字节（调用方判过）；读不全头 ID 那 8 字节的表这一格记射程里判不了。第一处违例写明是哪一版、哪一条。
fn judge_snapshot_tree_table_entries_paired_one_to_one(
    judgements: &mut Judgements,
    tree_table_entries: &[Vec<u8>],
    version_name: &str,
) {
    let kind_of = |entry: &Vec<u8>| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET);
    let snapshot_inode_root_entries: Vec<&Vec<u8>> = tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .collect();
    let snapshot_extent_root_entries: Vec<&Vec<u8>> = tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_SNAPSHOT_EXTENT_ROOT)
        .collect();
    if snapshot_inode_root_entries.is_empty() && snapshot_extent_root_entries.is_empty() {
        return;
    }
    if !snapshot_extent_root_entries
        .iter()
        .all(|entry| entry.len() >= TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8)
    {
        judgements.part_of_the_range_left_unjudged(
            "I-9.16",
            format!("{version_name}的树表单元自述的条目宽不到头 ID 那 8 字节：③ 快照条目按头 ID 配对读不出"),
        );
        return;
    }
    if snapshot_inode_root_entries.len() != snapshot_extent_root_entries.len() {
        judgements.judge("I-9.16", false, || {
            format!(
                "{version_name}的树表：快照 inode 根（种类 14）{} 条、快照 extent 根（种类 15）{} 条，条数不相等（③ 快照条目一一配对）",
                snapshot_inode_root_entries.len(),
                snapshot_extent_root_entries.len()
            )
        });
        return;
    }
    let snapshot_inode_root_trees: Vec<u64> = snapshot_inode_root_entries
        .iter()
        .map(|entry| read_u64(entry, 0))
        .collect();
    let mut snapshot_inode_root_trees_paired: BTreeSet<u64> = BTreeSet::new();
    let first_extent_root_without_its_own_inode_root = snapshot_extent_root_entries
        .iter()
        .map(|entry| {
            (
                read_u64(entry, 0),
                read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET),
            )
        })
        .find(|(_, head_identifier)| {
            !(snapshot_inode_root_trees.contains(head_identifier)
                && snapshot_inode_root_trees_paired.insert(*head_identifier))
        });
    judgements.judge(
        "I-9.16",
        first_extent_root_without_its_own_inode_root.is_none(),
        || {
            let (tree, head_identifier) =
                first_extent_root_without_its_own_inode_root.expect("判红时找到过那一条");
            format!(
                "{version_name}的树表：快照 extent 根（种类 15，树 ID {tree}）的头 ID {head_identifier} 配不上一条只属于它的快照 inode 根（种类 14 的树 ID：{snapshot_inode_root_trees:?}；③ 快照条目一一配对）"
            )
        },
    );
}

/// I-9.27（树表条目的头 ID 取值）：一版的树表判一次（树表被几版共享时只在第一个走到它的那一版判，同 I-9.16），每条条目按种类判一格
/// （[`HeadIdentifierOfATreeTableEntry`]），第一处违例写明是哪一版、第几条（下标从 0 数）。树表 0 条的版本一格也不判。
/// checker 自己按字段表切一份，不与实现共用代码（D13（验证路线） 已定项 5）。
/// 条目宽不到头 ID 那 8 字节（盘上字段，`index_node_view` 只保证 ≥ key 宽 8）时读不出头 ID：这张树表不判，记成射程里判不了的那一部分
/// （同 I-9.16 读不出种类那一格的处置）。
pub(super) fn judge_tree_table_entry_head_identifiers(
    judgements: &mut Judgements,
    tree_table_entries: &[Vec<u8>],
    version_name: &str,
) {
    let head_identifiers_are_readable = tree_table_entries
        .iter()
        .all(|entry| entry.len() >= TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8);
    if !head_identifiers_are_readable {
        judgements.part_of_the_range_left_unjudged(
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            format!("{version_name}的树表单元自述的条目宽不到头 ID 那 8 字节（条目偏移 116 起）：这张树表的头 ID 读不出"),
        );
        return;
    }
    let tree_identifier_of = |entry: &Vec<u8>| read_u64(entry, 0);
    let kind_of = |entry: &Vec<u8>| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET);
    let inode_tree_identifiers: Vec<u64> = tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_INODE)
        .map(tree_identifier_of)
        .collect();
    let snapshot_inode_root_tree_identifiers: Vec<u64> = tree_table_entries
        .iter()
        .filter(|entry| kind_of(entry) == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(tree_identifier_of)
        .collect();
    for (index, entry) in tree_table_entries.iter().enumerate() {
        let tree = tree_identifier_of(entry);
        let kind = kind_of(entry);
        let head_identifier = read_u64(entry, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
        let expected = HeadIdentifierOfATreeTableEntry::of_tree_kind(kind);
        let holds = match expected {
            HeadIdentifierOfATreeTableEntry::ItsOwnTreeIdentifier => head_identifier == tree,
            HeadIdentifierOfATreeTableEntry::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable => {
                inode_tree_identifiers.contains(&head_identifier)
            }
            HeadIdentifierOfATreeTableEntry::Zero => head_identifier == 0,
            HeadIdentifierOfATreeTableEntry::TreeIdentifierOfASnapshotInodeRootEntryOfTheSameTable => {
                snapshot_inode_root_tree_identifiers.contains(&head_identifier)
            }
            HeadIdentifierOfATreeTableEntry::NotJudged => continue,
        };
        judgements.judge(HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES, holds, || {
            format!(
                "{version_name}的树表：第 {index} 条（树 ID {tree}、种类 {kind}）的头 ID 是 {head_identifier}，{}",
                expected.expectation(
                    tree,
                    &inode_tree_identifiers,
                    &snapshot_inode_root_tree_identifiers
                )
            )
        });
    }
}

#[cfg(test)]
mod tests;
