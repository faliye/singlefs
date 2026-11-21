//! 树表条目的判定：每种树至多一条、快照条目一一配对、按树 ID 与发号次序升序，连同报这几种拒收的常量；
//! 一条根的树表里用户可见两棵树的根指针与每条条目的盘上原样。

use super::*;

/// 树表里同一种树出现两条时，读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
pub const TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE: &str = "树表里同一种树有两条";

/// 树表里码 14（快照 inode 根）与码 15（快照 extent 根）的条数不相等时，读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
pub const TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER: &str =
    "树表里快照 inode 根与快照 extent 根的条数不相等";

/// 树表里有一条码 15（快照 extent 根）的头 ID 不是任何一条码 14（快照 inode 根）的树 ID，或它指的那条码 14 前面已经有一条码 15 配上了：
/// 读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
pub const TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY: &str =
    "树表里有一条快照 extent 根配不上一条只属于它的快照 inode 根";

/// 树表里码 14（快照 inode 根）多于 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条：读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，
/// `what` 取这一句。
pub const TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS: &str =
    "树表里的只读快照多于第一版允许的个数";

/// 树表里有一条码 14（快照 inode 根）的头 ID 不等于同一张树表里活头 inode 树（码 2）条目的树 ID（表里没有码 2 条目时也报这一句）：
/// 读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
pub const TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE: &str =
    "树表里有一条快照 inode 根的头 ID 不是活头 inode 树的树 ID";

/// 树表的条目读不下去是哪一种（[`tree_table_entries_each_kind_at_most_once`] 交回）。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TreeTableEntriesRefusal {
    /// 一条条目解不开（条目长度、flags 未知位、预留 76 非零、根指针头部 extent 偏移非 0，`TreeTableEntry::parse`）。
    EntryMalformed,
    /// 同一种树有两条（码 14、15 两种快照条目除外）。
    OneKindOfTreeAppearsTwice,
    /// 码 14 与码 15 的条数不相等。
    SnapshotInodeAndExtentRootEntryCountsDiffer,
    /// 一条码 15 的头 ID 不是任何一条码 14 的树 ID，或它指的那条码 14 已经被前面一条码 15 配上了。
    SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,
    /// 码 14 多于 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条。
    MoreSnapshotsThanTheFirstVersionAllows,
    /// 一条码 14 的头 ID 不等于这张树表里码 2 条目的树 ID（或表里没有码 2 条目）。
    SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree,
}

impl From<TreeTableEntriesRefusal> for RecoveryFailure {
    fn from(refusal: TreeTableEntriesRefusal) -> Self {
        match refusal {
            TreeTableEntriesRefusal::EntryMalformed => RecoveryFailure::UnitMalformed {
                what: "树表条目",
            },
            TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice => RecoveryFailure::UnitMalformed {
                what: TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE,
            },
            TreeTableEntriesRefusal::SnapshotInodeAndExtentRootEntryCountsDiffer => {
                RecoveryFailure::UnitMalformed {
                    what: TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER,
                }
            }
            TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry => {
                RecoveryFailure::UnitMalformed {
                    what: TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY,
                }
            }
            TreeTableEntriesRefusal::MoreSnapshotsThanTheFirstVersionAllows => {
                RecoveryFailure::UnitMalformed {
                    what: TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS,
                }
            }
            TreeTableEntriesRefusal::SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree => {
                RecoveryFailure::UnitMalformed {
                    what: TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE,
                }
            }
        }
    }
}

/// 一个只读快照在树表里的那一对条目：码 14（快照 inode 根）与头 ID 指着它的那条码 15（快照 extent 根），字段取值见
/// [`TREE_KIND_SNAPSHOT_INODE_ROOT`] 与 [`TREE_KIND_SNAPSHOT_EXTENT_ROOT`]。重建只把它们读出来，不解它们指着的根。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotTreeTableEntries {
    pub inode_root: TreeTableEntry,
    pub extent_root: TreeTableEntry,
}

/// 树表里的快照条目（码 14、15）配成对，按码 14 的盘上次序（即树 ID 升序，I-9.16 ①）交回，并判四样（判决
/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1 与第七节岔路 1）：两种条数相等；每条码 15 的头 ID 恰是一条码 14 的树 ID、
/// 且那条码 14 只被这一条码 15 配上（两样合起来是一一配对）；码 14 至多 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条；每条码 14 的头 ID
/// 等于这张树表里码 2（活头 inode 树）条目的树 ID。没有快照条目的树表交空表，一样都不判。
///
/// # Errors
/// 按上面的次序第一样不成立的那一个：`SnapshotInodeAndExtentRootEntryCountsDiffer`、`SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry`、
/// `MoreSnapshotsThanTheFirstVersionAllows`、`SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree`。
pub(crate) fn snapshot_tree_table_entries_paired_one_to_one(
    tree_table_entries: &[TreeTableEntry],
) -> Result<Vec<SnapshotTreeTableEntries>, TreeTableEntriesRefusal> {
    let snapshot_inode_root_entries: Vec<&TreeTableEntry> = tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .collect();
    let snapshot_extent_root_entries: Vec<&TreeTableEntry> = tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT)
        .collect();
    if snapshot_inode_root_entries.len() != snapshot_extent_root_entries.len() {
        return Err(TreeTableEntriesRefusal::SnapshotInodeAndExtentRootEntryCountsDiffer);
    }
    // 每条码 14 配上的那条码 15，下标与 `snapshot_inode_root_entries` 一一对应。
    let mut extent_root_paired_with: Vec<Option<TreeTableEntry>> =
        vec![None; snapshot_inode_root_entries.len()];
    for extent_root_entry in &snapshot_extent_root_entries {
        // 头 ID 指着它、又还没被前面哪条码 15 配上的那条码 14。
        let paired_inode_root_position = snapshot_inode_root_entries
            .iter()
            .zip(&extent_root_paired_with)
            .position(|(inode_root_entry, extent_root_already_paired)| {
                inode_root_entry.tree.0 == extent_root_entry.head_identifier
                    && extent_root_already_paired.is_none()
            })
            .ok_or(
                TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,
            )?;
        extent_root_paired_with[paired_inode_root_position] = Some(**extent_root_entry);
    }
    if snapshot_inode_root_entries.len() > SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
        return Err(TreeTableEntriesRefusal::MoreSnapshotsThanTheFirstVersionAllows);
    }
    let live_inode_tree = tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_INODE)
        .map(|entry| entry.tree);
    if snapshot_inode_root_entries.iter().any(|inode_root_entry| {
        Some(TreeIdentifier(inode_root_entry.head_identifier)) != live_inode_tree
    }) {
        return Err(TreeTableEntriesRefusal::SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree);
    }
    Ok(snapshot_inode_root_entries
        .into_iter()
        .zip(extent_root_paired_with)
        .map(|(inode_root_entry, extent_root_entry)| SnapshotTreeTableEntries {
            inode_root: *inode_root_entry,
            extent_root: extent_root_entry.expect(
                "两种条数相等、每条码 15 配上一条还没配过的码 14（上面两判）：鸽巢下每条码 14 都配上了",
            ),
        })
        .collect())
}

/// 树表单元的条目逐条解开，并判每一种树至多一条（审阅第 27 条）。写者每种树只写一条；同一种出现两条时取哪一条条款没有写，
/// 取首条、取末条、取首条非空各有一个读者照着读过，同一张树表在几处读出不同的树 ⇒ 判损坏，一条都不取。
/// 码 14、15 两种快照条目例外：每个快照一对，可以有多条，改判 [`snapshot_tree_table_entries_paired_one_to_one`] 那四样。
/// 读树表取条目的每一处都经这一处：[`rebuild_version`]、[`allocation_records_under_root`]、[`walk_to_file`]、
/// [`user_visible_tree_root_pointers`]、`mounted_read::open_pool_for_read`。
///
/// # Errors
/// 按条目次序第一处读不下去的：解不开 ⇒ `EntryMalformed`；它的种类（码 14、15 除外）前面已经有一条 ⇒ `OneKindOfTreeAppearsTwice`；
/// 条目都过了之后快照条目配不成对 ⇒ [`snapshot_tree_table_entries_paired_one_to_one`] 的那四个成员。
pub(crate) fn tree_table_entries_each_kind_at_most_once(
    entry_bytes_of_the_tree_table: &[Vec<u8>],
) -> Result<Vec<TreeTableEntry>, TreeTableEntriesRefusal> {
    let mut kinds_seen: BTreeSet<u16> = BTreeSet::new();
    let mut entries = Vec::with_capacity(entry_bytes_of_the_tree_table.len());
    for entry_bytes in entry_bytes_of_the_tree_table {
        let entry =
            TreeTableEntry::parse(entry_bytes).ok_or(TreeTableEntriesRefusal::EntryMalformed)?;
        let is_a_snapshot_entry = entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
            || entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT;
        if is_a_snapshot_entry {
            entries.push(entry);
            continue;
        }
        if !kinds_seen.insert(entry.kind) {
            return Err(TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice);
        }
        entries.push(entry);
    }
    snapshot_tree_table_entries_paired_one_to_one(&entries)?;
    Ok(entries)
}

/// 树表条目不按排序时报的不变量名：I-9.16（树表条目按树 ID 严格升序且合发号次序）。两道合成这一条，任一道不成立即判红：
/// ① 盘上次序树 ID 严格升序（D8（核心索引结构） 已定项 8 排序契约「条目按树 ID 升序排」）；② 按发号次序相邻两棵的树 ID 严格升序
/// （D8 已定项 8 ②「十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547」）。立号：用户 2026-09-27 定
/// 「立不变量并同步」；`invariants.md` 那一行由 kb 第八批写。
pub const TREE_TABLE_ENTRIES_ORDERING_CONTRACT: &str = "I-9.16";

/// 十二棵进树表的树的种类，按发号次序（D8（核心索引结构） 已定项 8 ②：extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist，
/// 再接 D18（块里携带什么信息） 已定项 24 第 6 条的 `deleted_inodes`、墓碑树、谱系树、logged_ops，最后是退役树（D8 已定项 9 码 13）；
/// 中央映射树不进树表，这里没有它）。
const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 12] = [
    TREE_KIND_EXTENT,
    TREE_KIND_INODE,
    TREE_KIND_ALLOCATION,
    TREE_KIND_ACCOUNTING,
    TREE_KIND_LIVELIST,
    TREE_KIND_SPARSE_SIDE_TABLE,
    TREE_KIND_DEADLIST,
    TREE_KIND_DELETED_INODES,
    TREE_KIND_TOMBSTONE,
    TREE_KIND_LINEAGE,
    TREE_KIND_LOGGED_OPERATIONS,
    TREE_KIND_RETIREMENT,
];

/// I-9.16（树表条目按树 ID 严格升序且合发号次序）：先判 ①（盘上次序），再判 ②（发号次序）。读树表取条目、要往下走的两处都在
/// [`tree_table_entries_each_kind_at_most_once`] 之后、读任何一棵树之前调它：[`rebuild_version`]（可写挂载接着这一版发布，
/// 写者按种类装树表条目、断言树 ID 升序，走得到那条断言的镜像在任何写之前拒）与 [`walk_to_file`]。
///
/// # Errors
/// ① 不成立 ⇒ `InvariantViolated { invariant: "I-9.16", detail: "树表条目不按树 ID 严格升序" }`；
/// ① 成立、② 不成立 ⇒ 同一个不变量名，detail 是发号次序那一句。
pub(crate) fn tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(
    tree_table_entries: &[TreeTableEntry],
) -> Result<(), RecoveryFailure> {
    tree_table_entries_ascend_strictly_by_tree_identifier(tree_table_entries)?;
    tree_table_entries_ascend_strictly_in_the_issuing_order(tree_table_entries)
}

/// I-9.16 ①：树表条目按盘上的次序树 ID 严格升序（两条同号也拒：同一个号认成两棵树）。
///
/// # Errors
/// 相邻两条的树 ID 不是严格升序 ⇒ `InvariantViolated { invariant: "I-9.16", .. }`。
fn tree_table_entries_ascend_strictly_by_tree_identifier(
    tree_table_entries: &[TreeTableEntry],
) -> Result<(), RecoveryFailure> {
    if tree_table_entries
        .windows(2)
        .all(|pair| pair[0].tree < pair[1].tree)
    {
        return Ok(());
    }
    Err(RecoveryFailure::InvariantViolated {
        invariant: TREE_TABLE_ENTRIES_ORDERING_CONTRACT,
        detail: "树表条目不按树 ID 严格升序",
    })
}

/// I-9.16 ②：树表里的树按发号次序（[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]）相邻两棵的树 ID 严格升序。
/// 写者每次发布按这个次序装树表条目、断言它们升序（`transaction` 装树表那一步）：照抄上一版的号，这个次序就是上一版盘上的样子，
/// 那条断言靠的就是这一判（两条条目的种类互换之后盘上次序仍按树 ID 升序、写者按种类装出来却降序）。
/// 只比树表里有的种类：缺哪一棵、有没登记的种类，由读那一棵的那一步报（[`rebuild_version`] 报「树表里没有那棵树」，
/// [`walk_to_file`] 报「树表里缺一棵有根的树」或「树的种类没登记」）；同一种至多一条已由 [`tree_table_entries_each_kind_at_most_once`] 判过。
///
/// # Errors
/// 发号次序上相邻两棵的树 ID 不是严格升序 ⇒ `InvariantViolated { invariant: "I-9.16", .. }`。
fn tree_table_entries_ascend_strictly_in_the_issuing_order(
    tree_table_entries: &[TreeTableEntry],
) -> Result<(), RecoveryFailure> {
    let tree_identifiers_in_the_issuing_order: Vec<TreeIdentifier> =
        TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER
            .iter()
            .filter_map(|kind| {
                tree_table_entries
                    .iter()
                    .find(|entry| entry.kind == *kind)
                    .map(|entry| entry.tree)
            })
            .collect();
    if tree_identifiers_in_the_issuing_order
        .windows(2)
        .all(|pair| pair[0] < pair[1])
    {
        return Ok(());
    }
    Err(RecoveryFailure::InvariantViolated {
        invariant: TREE_TABLE_ENTRIES_ORDERING_CONTRACT,
        detail: "树表里各棵树的树 ID 不按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist、deleted_inodes、墓碑树、谱系树、logged_ops）严格升序",
    })
}

/// 一条根的树表里用户可见的两棵树（inode 树、extent 树）的根指针，取树表条目里那 86 字节的盘上原样；树表里没有那棵树的条目时 `None`。
/// 抬 F 的上限要数「非空」的有效根：一条有效根算非空 ⟺ 这两样与它前一条有效根的不同（D16（发布语义） 已定项 1「非空」从盘上怎么认，
/// 2026-09-17 用户定案）；树表单元自己的落点每次发布都变，不拿它比。
/// 只读快照的两种条目（码 14 快照 inode 根、码 15 快照 extent 根）也是用户可见的根（主 agent 2026-10-13 定：码 14 / 15 是用户可见角色，
/// 判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S2）：它们的根指针按盘上次序记在 `snapshot_tree_roots`；
/// 「非空」那一判连它一起比（用户 2026-10-14 定「建快照那次发布算非空」，`mount::user_visible_trees_changed`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserVisibleTreeRootPointers {
    pub inode_tree: Option<Vec<u8>>,
    pub extent_tree: Option<Vec<u8>>,
    /// 树表里每一条码 14、码 15 条目的根指针盘上原样，按条目的盘上次序（即树 ID 升序）；没有快照的树表为空。
    pub snapshot_tree_roots: Vec<Vec<u8>>,
}

impl UserVisibleTreeRootPointers {
    /// 树表里没有这两棵树的条目（mkfs 的第 0 代树表就是这样）：最旧的有效根没有前一条时拿它比。
    pub const ABSENT: Self = Self {
        inode_tree: None,
        extent_tree: None,
        snapshot_tree_roots: Vec::new(),
    };
}

/// 读一条根的树表，取 inode 树与 extent 树的根指针盘上字节。
///
/// # Errors
/// 树表单元读不到、解不开；树表条目解不开、同一种树出现两条（[`TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE`]，先于下一条判），或种类没登记。
pub fn user_visible_tree_root_pointers(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<UserVisibleTreeRootPointers, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;
    let mut pointers = UserVisibleTreeRootPointers::ABSENT;
    // 两串逐条对应：`tree_table_entries` 是 `tree_table.entries` 按次序逐条解出来的，条数相同。
    for (entry, entry_bytes) in tree_table_entries.iter().zip(&tree_table.entries) {
        let slot_for_this_tree = match entry.kind {
            TREE_KIND_INODE => &mut pointers.inode_tree,
            TREE_KIND_EXTENT => &mut pointers.extent_tree,
            TREE_KIND_ALLOCATION
            | TREE_KIND_ACCOUNTING
            | TREE_KIND_LIVELIST
            | TREE_KIND_SPARSE_SIDE_TABLE
            | TREE_KIND_DEADLIST
            | TREE_KIND_DELETED_INODES
            | TREE_KIND_TOMBSTONE
            | TREE_KIND_LINEAGE
            | TREE_KIND_LOGGED_OPERATIONS
            | TREE_KIND_RETIREMENT => continue,
            // 只读快照的两种条目：一个快照一对、可以有多条，按盘上次序记下（配对在 `tree_table_entries_each_kind_at_most_once` 判过）。
            TREE_KIND_SNAPSHOT_INODE_ROOT | TREE_KIND_SNAPSHOT_EXTENT_ROOT => {
                pointers
                    .snapshot_tree_roots
                    .push(TreeTableEntry::root_pointer_bytes(entry_bytes).to_vec());
                continue;
            }
            _ => {
                return Err(RecoveryFailure::UnitMalformed {
                    what: "树的种类没登记",
                })
            }
        };
        // 同一种树至多一条，上面判过：这一格在这一趟里只填一次。
        *slot_for_this_tree = Some(TreeTableEntry::root_pointer_bytes(entry_bytes).to_vec());
    }
    Ok(pointers)
}

/// 一条根的树表单元里每一条条目的盘上原样（每条是树表单元自述的条目宽那么多字节，D8（核心索引结构） 已定项 8 的字段表是 200），
/// 按盘上次序；树表 0 条的一版交空表。只读、不判条目内容（种类登没登记、同一种几条都不判）：挂载透明的判官
/// （`singlefs_verification::mount_transparency`）按字节区间比 R1 与 R2 的条目，它要的是原字节，不是解出来的几样。
///
/// # Errors
/// 树表单元读不到（两份位置都读不出或校验和对不上）、解不开。
pub fn tree_table_entry_bytes_of_root<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    root: &RootRecord,
) -> Result<Vec<Vec<u8>>, RecoveryFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    Ok(tree_table.entries)
}
