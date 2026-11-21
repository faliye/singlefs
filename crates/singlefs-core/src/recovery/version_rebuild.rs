//! 从盘上按所选根重建上一版（[`rebuild_version_in_the_unit_area_starting_at`]）：自举单元、有树那一版的树表条目与各棵树的号，
//! 经中央映射读这一版的各棵树（每个文件的数据单元、inode 树、删除 inode 根与记账操作根、容器树、分配记录树、记账树与 deadlist），
//! 映射条目与这一版的全部单元，连同重建的结果与失败。

use super::*;

/// 从盘上按所选根重建出来的上一版。
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "一次挂载只重建一个，两个成员差几百字节按值搬无所谓；装箱只多一层解引用"
)]
pub enum RebuiltVersion {
    /// 树表 0 条：所选根下面还没有发布过文件版本（mkfs 的第 0 代根、第一次可写挂载的暖机根），这一版只有根自己指着的两个单元。
    WithoutFile,
    WithFile(TransactionOutput),
    /// 有树、文件集合为空（D8（核心索引结构） 已定项 16 第 5 条）：判定与 [`crate::transaction::PoolVersion::of_a_version_of_trees`] 同一个
    /// （[`TransactionOutput::is_a_version_with_trees_and_no_file`]），不看 logged_ops、墓碑树、谱系树。
    WithTreesAndNoFile(TransactionOutput),
}

impl RebuiltVersion {
    /// 重建出来的一版有树时归哪个成员（判定同 `PoolVersion::of_a_version_of_trees`）。
    #[must_use]
    pub fn of_a_version_of_trees(version: TransactionOutput) -> RebuiltVersion {
        if version.is_a_version_with_trees_and_no_file() {
            RebuiltVersion::WithTreesAndNoFile(version)
        } else {
            RebuiltVersion::WithFile(version)
        }
    }
}

/// 重建上一版没做成。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RebuildVersionFailure {
    /// 单元读不到、解不开，或走读同款的判定判红。
    Walk(RecoveryFailure),
    /// 所选根下面有文件版本，而调用方拿不出一条记录顶着这一版（环里一条自证过的记录都没有）。
    NoRecordStandingForFileVersion,
}

impl From<RecoveryFailure> for RebuildVersionFailure {
    fn from(failure: RecoveryFailure) -> Self {
        RebuildVersionFailure::Walk(failure)
    }
}

/// 从盘上按所选根重建「上一版」：全部角色的单元字节、指针、树表、分配记录、记账行与 inode 记录，交给发布路径当上一版
/// （照抄没重写的角色、经映射释放被换下的角色都靠它；可写挂载在恢复之后调）。`record_standing_for_root` 是所选根覆盖的最后一条记录
/// （同实例、同 checkpoint_txg 里带「本次发布末条」标志的那一条，D23（journal 的角色与格式） 已定项 14 注 1 读法乙；读不出时由调用方顶一条；树表 0 条时用不到）。
/// extent 根兼叶里的每条记录指的数据单元都读回来：一个文件跨多个单元时（并行线一）每个单元一个角色。
/// 豁免三类之外的单元位置提示读不出时经这一版的中央映射回退（D19（块指针的结构与宽度预算） 已定项 8）；
/// 提示与映射都读不出的**数据单元**不算失败：照抄它的位置项、不读内容，那一项的字节是空的（D19 已定项 5，用户 2026-09-24 定 N2）。
///
/// 读到手的每个节点与单元照冷走读判全（代码审阅第 24 / 33 条，用户定案「重建时把已经读到的节点照冷走读那套判全，坏了报错拒挂载，不多读盘」）：
/// 四棵树与中央映射树的节点头按指着它的指针与父条目核（`EveryHeaderAgainstItsReference`，含 key 严格递增），inode 树根按
/// `tree_root_checked_against_its_pointer` 核、层级 1，树表 key 宽与 I-7.8，inode 叶容器的 I-9.2 / I-1.2 / I-9.4，分配记录每盘一条、代与跨度，
/// 记账条目数与代、seq，映射条目数，读到手的数据单元按查找路径与 inode 记录核——与 [`walk_to_file`] 调同一组判定函数。不多读一个单元。
///
/// # Errors
/// 树表不是 0 条而 `record_standing_for_root` 是 `None` ⇒ `NoRecordStandingForFileVersion`；树表里同一种树有两条
/// （[`TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE`]）；树表条目不合 I-9.16（[`TREE_TABLE_ENTRIES_ORDERING_CONTRACT`]）；实例表、树表、映射树根读不到，
/// 树根或 inode 叶容器提示读不出且经映射也读不回（`MappingMiss` / `MappingStillUnreadable`），单元解不开、上面那几道判定判红 ⇒ 走读同款的错；
/// 记账行里没有 inode 号水位那一行 ⇒ `InodeNumberWatermarkRowMissingFromTheAccountingTree`。
///
/// 单元区从 `unit_area_start` 起（择到的系统配置按环长现算的那个）：这一版的分配记录落不落在单元区里按它判。
/// 树表里的只读快照条目（码 14、15）照盘上原样在交回那一版的 `tree_table_entries` 里（发布路径接着这一版发时照它们续上，
/// `transaction` 装树表那一步）；要它们配成对的用 [`rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at`]。
pub fn rebuild_version_in_the_unit_area_starting_at(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
    record_standing_for_root: Option<JournalRecord>,
) -> Result<RebuiltVersion, RebuildVersionFailure> {
    rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
        reader,
        unit_area_start,
        root,
        record_standing_for_root,
    )
    .map(|rebuilt| rebuilt.version)
}

/// 重建出来的一版，连同它树表里的只读快照条目（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RebuiltVersionWithItsSnapshotEntries {
    pub version: RebuiltVersion,
    /// 码 14 与配对的码 15，按码 14 的树 ID 升序；树表里没有快照条目（含树表 0 条）时为空。只读出条目，不解它们指着的根。
    pub snapshot_entries: Vec<SnapshotTreeTableEntries>,
}

/// 同 [`rebuild_version_in_the_unit_area_starting_at`]（判全、报错都与它相同），另把树表里码 14、15 的条目配成对交出来
/// （[`snapshot_tree_table_entries_paired_one_to_one`]，配不成对的树表在 [`tree_table_entries_each_kind_at_most_once`] 那一步就拒了）。
///
/// # Errors
/// 同 [`rebuild_version_in_the_unit_area_starting_at`]。
pub fn rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
    record_standing_for_root: Option<JournalRecord>,
) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
    // 树表 0 条的一版也要判：可写挂载接着它发的零单元发布照抄根记录里的实例表与树表指针。
    let bootstrap_units = bootstrap_units_of_the_root_read_and_judged(reader, root)?;
    if bootstrap_units.tree_table.entries.is_empty() {
        return Ok(RebuiltVersionWithItsSnapshotEntries {
            version: RebuiltVersion::WithoutFile,
            snapshot_entries: Vec::new(),
        });
    }
    let record =
        record_standing_for_root.ok_or(RebuildVersionFailure::NoRecordStandingForFileVersion)?;
    location_entries_of_the_record_ascend_by_device(&record)?;
    let record_bytes = record.to_bytes();
    let (tree_table_entries, snapshot_entries) =
        tree_table_entries_of_a_version_with_trees_judged(&bootstrap_units.tree_table, root)?;
    let extent_pointer = root_pointer_in_the_tree_table(&tree_table_entries, TREE_KIND_EXTENT)?;
    let inode_root_pointer = root_pointer_in_the_tree_table(&tree_table_entries, TREE_KIND_INODE)?;
    let allocation_pointer =
        root_pointer_in_the_tree_table(&tree_table_entries, TREE_KIND_ALLOCATION)?;
    let accounting_pointer =
        root_pointer_in_the_tree_table(&tree_table_entries, TREE_KIND_ACCOUNTING)?;
    let tree_identifiers = tree_identifiers_of_the_version(&tree_table_entries, root)?;
    // 豁免三类之外的单元（数据单元、四棵树的根、inode 叶容器）位置提示读不出时经这一版的中央映射回退，与冷走读同一条
    // （D19（块指针的结构与宽度预算） 已定项 8）；映射树根、树表、实例表是自举豁免，只按根记录里的位置条目读。
    // 映射树根到第一次要回退、或走到它那一步时才读：提示都读得出的镜像上读序与报错次序照旧。
    let mut reading = VersionReadingThroughTheCentralMapping {
        reader,
        central_mapping_root: CentralMappingTreeWithBytesReadOnFirstUse::new(
            reader,
            root,
            CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
        ),
        stale_location_hint_hops_not_exposed_by_this_reader: 0,
    };
    // extent 树按 key 空间定形状（D8（核心索引结构） 已定项 14 的两段）：整棵读回来，每个节点按位置核，节点进映射、提示读不出经映射回退。
    let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
    let extent_tree_read = read_extent_tree(
        &ExtentTreeReading {
            tree: tree_identifiers.extent,
            judgement: ExtentTreeHeaderJudgement::EveryHeaderAgainstItsReference,
            root,
            expected_filesystem_identifier,
        },
        &extent_pointer,
        &mut |pointer: &NodePointer| {
            reading.read_mapped_tree_node(pointer, MappedTreeNodeClass::IndexNode)
        },
    )?;
    let first_file_extents = extents_of_the_first_file(extent_tree_read)?;
    let data_unit_contents = data_units_of_every_file_read(&mut reading, &first_file_extents)?;
    let inode_tree = inode_tree_read_and_judged(
        &mut reading,
        &inode_root_pointer,
        tree_identifiers.inode,
        root,
    )?;
    let inode_record = data_units_judged_against_the_inode_records(
        &first_file_extents,
        &data_unit_contents,
        &inode_tree.leaf_containers,
        tree_identifiers.extent,
        expected_filesystem_identifier,
    )?;
    let deleted_inodes_root = deleted_inodes_root_read(
        &mut reading,
        &tree_table_entries,
        tree_identifiers.deleted_inodes,
        root,
    )?;
    let logged_operations_root = logged_operations_root_read(
        &mut reading,
        &tree_table_entries,
        tree_identifiers.logged_operations,
        root,
    )?;
    let container_trees =
        container_trees_read(&mut reading, &tree_table_entries, &tree_identifiers, root)?;
    let allocation_tree_read = allocation_record_tree_read_and_judged(
        &mut reading,
        unit_area_start,
        &allocation_pointer,
        tree_identifiers.allocation_records,
        root,
    )?;
    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;
    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();
    let (accounting_tree, accounting_entries) = accounting_tree_read_and_judged(
        &mut reading,
        &accounting_pointer,
        tree_identifiers.accounting,
        root,
    )?;
    let (deadlist_tree, deadlist_entries) = deadlist_tree_read(
        &mut reading,
        &tree_table_entries,
        tree_identifiers.deadlist,
        root,
    )?;
    let snapshots_being_destroyed =
        snapshots_being_destroyed_in_the_tree_table(&bootstrap_units.tree_table);
    // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。
    let mapping_tree = reading.central_mapping_root.into_tree()?;
    mapping_entry_count_is_one_per_mapped_unit(
        mapping_tree.leaf_entries_in_key_order.len(),
        MappedUnitCounts {
            extent_tree_nodes: first_file_extents.nodes_in_bump_order.len(),
            allocation_record_tree_nodes: allocation_tree_read.nodes.len(),
            accounting_tree_nodes: accounting_tree.version.node_count(),
            deadlist_tree_nodes: deadlist_tree.version.node_count(),
            mapping_entries_kept_for_the_snapshots: deadlist_entries.len(),
            inode_leaf_containers: inode_tree.root_node.entries.len(),
            data_units: first_file_extents.data_units_of_every_file(),
            deletion_tree_nodes: usize::from(deleted_inodes_root.bytes.is_some())
                + usize::from(logged_operations_root.bytes.is_some())
                + container_trees.mapped_units(),
        },
    )?;
    let mapping_entries = mapping_entries_of_the_version(&mapping_tree, &deadlist_entries)?;
    let trees_read = TreesOfTheVersionRead {
        first_file_extents,
        data_unit_contents,
        inode_tree,
        inode_root_pointer,
        deleted_inodes_root,
        logged_operations_root,
        container_trees,
        allocation_tree_read,
        accounting_tree,
        deadlist_tree,
        mapping_tree,
    };
    let mapped_units = mapped_units_of_the_version(&trees_read);
    central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots(
        &mapping_entries,
        &mapped_units,
    )?;
    let allocation_record_tree = trees_read.allocation_tree_read.version();
    let files_other_than_the_first = files_other_than_the_first_of_the_version(&trees_read);
    let data_pointers = trees_read.first_file_extents.data_pointers.clone();
    let extent_tree = trees_read.first_file_extents.version.clone();
    let deleted_inodes = trees_read.deleted_inodes_root.inode_numbers.clone();
    let logged_operations = trees_read.logged_operations_root.entries.clone();
    let tombstone_containers = trees_read.container_trees.tombstone_containers();
    let tombstone_tree_index_nodes = trees_read.container_trees.tombstone_tree_index_nodes();
    let lineage_containers = trees_read.container_trees.lineage_containers();
    let lineage_tree_index_nodes = trees_read.container_trees.lineage_tree_index_nodes();
    let retirement_containers = trees_read.container_trees.retirement_containers();
    let retirement_tree_index_nodes = trees_read.container_trees.retirement_tree_index_nodes();
    let inode_leaf_containers = trees_read.inode_tree.leaf_containers.clone();
    let accounting_tree_version = trees_read.accounting_tree.version.clone();
    let central_mapping_tree = trees_read.mapping_tree.version.clone();
    let deadlist_tree_version = trees_read.deadlist_tree.version.clone();
    let units = units_of_the_version(trees_read, root, bootstrap_units);
    // 装好这一版的 units 之后记下两棵容器树根单元的下标（挂载之后头一次准入读 H 按它直接取）。
    let container_tree_root_unit_positions = ContainerTreeRootUnitPositions::of_the_units(&units);
    // n_live 与 n_记录的起始值：冷恢复读出这一版时逐片逐条数一遍（挂载一次），之后每次发布在提交时增量维护。
    let inode_record_counts = InodeRecordCounts::counted_over_the_inode_tree(
        inode_leaf_containers
            .iter()
            .map(|container| &container.contents),
    );
    let transaction_number_on_the_record_standing_for_this_root = record.transaction;
    let mut rebuilt = TransactionOutput {
        root: *root,
        record,
        record_bytes,
        earlier_records_of_this_publish: Vec::new(),
        units,
        rewritten: Vec::new(),
        data_pointers,
        files_other_than_the_first,
        mapping_keys: mapping_entries.mapping_keys,
        allocation_records,
        allocation_record_tree,
        extent_tree,
        accounting_entries,
        accounting_tree: accounting_tree_version,
        central_mapping_tree,
        deadlist_entries,
        deadlist_tree: deadlist_tree_version,
        mapping_entries_kept_for_the_snapshots: mapping_entries
            .mapping_entries_kept_for_the_snapshots,
        snapshots_being_destroyed,
        tree_table_entries,
        tree_identifiers,
        inode_record,
        deleted_inodes,
        logged_operations,
        tombstone_containers,
        tombstone_tree_index_nodes,
        lineage_containers,
        lineage_tree_index_nodes,
        retirement_containers,
        retirement_tree_index_nodes,
        inode_leaf_containers,
        mapped_units,
        released: Vec::new(),
        quarantined_after_release_checksum_mismatch: Vec::new(),
        units_relocated_by_this_publish: Vec::new(),
        key_order_mismatches: 0,
        writes: WritesByStructureKind::NOTHING_WRITTEN,
        // 重建出来的这一版属于**旧**实例。今天每条恢复路径（普通挂载、回退、切换）之后都要取新实例代号，
        // 而事务号按实例各算各的、从 1 重新起（D23（journal 的角色与格式） 已定项 7），所以这个值不会被拿去接着发布——
        // 写行那次发布显式传 0。将来真要在同一个实例上续发，得由调用方扫环算出这个实例的最大非 0 事务号传进来，
        // 这里这条记录上的事务号在它是空发布时是 0，单独拿它续号会重号。
        // 钉住这个前提的是池级 checker 的 I-8.7（实例内事务号不重号）：同一实例按计数器相邻的两条非 0 记录，
        // 后一条事务号要严格大于前一条。前提一旦失效，那条不变量会在盘上判红——2026-09-21 实测把发布路径改回
        // 「上一条记录的事务号 + 1」，33 条里只有 I-8.7 judged 出来。
        highest_transaction_number_in_this_instance:
            transaction_number_on_the_record_standing_for_this_root,
        container_tree_root_unit_positions,
        inode_record_counts,
    };
    // I-2.5 先判盘上读来的提示、再盖映射位置：盖过之后提示就没了，盘上逆序的提示被映射位置盖掉，池级 checker 照盘上字节判坏的镜像
    // 可写挂载却会收下。
    pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;
    take_the_mapping_positions_over_the_location_hints(&mut rebuilt, |key| {
        mapping_entries.mapping_position_of_key.get(key).copied()
    });
    // 盖进来的映射位置同样按 I-2.5 判（它的射程含中央映射条目的 value）：下一次发布重写父节点时把它们写进指针，写者在那里断言升序。
    pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;
    Ok(RebuiltVersionWithItsSnapshotEntries {
        version: RebuiltVersion::of_a_version_of_trees(rebuilt),
        snapshot_entries,
    })
}

/// 重建一版时按根记录里的位置条目读的两个自举单元（实例表、树表），树表已解开、key 宽判过。
struct BootstrapUnitsOfTheRoot {
    instance_table_bytes: Vec<u8>,
    tree_table_bytes: Vec<u8>,
    tree_table: IndexNodeHeader,
}

/// 根记录里的指针按 I-2.5 判，再按根记录里的位置条目读实例表与树表（自举豁免，不经映射回退）、解开树表、判它的 key 宽。
fn bootstrap_units_of_the_root_read_and_judged(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<BootstrapUnitsOfTheRoot, RebuildVersionFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    root_record_pointers_ascend_by_device(root)?;
    let instance_table_bytes =
        read_unit_via_locations(reader, &root.instance_table.locations, data_bytes)?;
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    tree_table_key_width_is_the_tree_tables(&tree_table)?;
    Ok(BootstrapUnitsOfTheRoot {
        instance_table_bytes,
        tree_table_bytes,
        tree_table,
    })
}

/// 有树的一版的树表条目：每种至多一条、快照条目一一配对（交回配好的对），按 I-9.16 判次序与种类（可写挂载接着这一版发布，
/// 写者按种类装树表条目、断言树 ID 升序：走得到那条断言的镜像在任何写之前拒），每条的树 ID 在根的水位之下。
fn tree_table_entries_of_a_version_with_trees_judged(
    tree_table: &IndexNodeHeader,
    root: &RootRecord,
) -> Result<(Vec<TreeTableEntry>, Vec<SnapshotTreeTableEntries>), RecoveryFailure> {
    let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)
        .map_err(RecoveryFailure::from)?;
    let snapshot_entries = snapshot_tree_table_entries_paired_one_to_one(&tree_table_entries)
        .expect(
            "tree_table_entries_each_kind_at_most_once 交回之前对同一串条目判过快照条目一一配对",
        );
    tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(&tree_table_entries)?;
    for entry in &tree_table_entries {
        tree_identifier_of_the_entry_is_below_the_watermark(entry, root)?;
    }
    Ok((tree_table_entries, snapshot_entries))
}

/// 树表里 `kind` 那一种树的根指针；没有那一种 ⇒ `UnitMalformed`（「树表里没有那棵树」）。
fn root_pointer_in_the_tree_table(
    tree_table_entries: &[TreeTableEntry],
    kind: u16,
) -> Result<NodePointer, RecoveryFailure> {
    tree_table_entries
        .iter()
        .find(|entry| entry.kind == kind)
        .map(|entry| entry.root)
        .ok_or(RecoveryFailure::UnitMalformed {
            what: "树表里没有那棵树",
        })
}

/// 这一版那十三棵树各自的号照盘上读回来：十二棵按树表条目的种类取条目里的树 ID，中央映射树不进树表、取根记录里
/// 它那条根指针的出生树（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11）。接着这一版的发布照抄它们，不另发。
fn tree_identifiers_of_the_version(
    tree_table_entries: &[TreeTableEntry],
    root: &RootRecord,
) -> Result<FileVersionTreeIdentifiers, RecoveryFailure> {
    let tree_of = |kind: u16| {
        tree_table_entries
            .iter()
            .find(|entry| entry.kind == kind)
            .map(|entry| entry.tree)
            .ok_or(RecoveryFailure::UnitMalformed {
                what: "树表里没有那棵树",
            })
    };
    Ok(FileVersionTreeIdentifiers {
        extent: tree_of(TREE_KIND_EXTENT)?,
        inode: tree_of(TREE_KIND_INODE)?,
        allocation_records: tree_of(TREE_KIND_ALLOCATION)?,
        accounting: tree_of(TREE_KIND_ACCOUNTING)?,
        central_mapping: root.mapping_root.head.birth_tree,
        livelist: tree_of(TREE_KIND_LIVELIST)?,
        sparse_side_table: tree_of(TREE_KIND_SPARSE_SIDE_TABLE)?,
        deadlist: tree_of(TREE_KIND_DEADLIST)?,
        deleted_inodes: tree_of(TREE_KIND_DELETED_INODES)?,
        tombstone: tree_of(TREE_KIND_TOMBSTONE)?,
        lineage: tree_of(TREE_KIND_LINEAGE)?,
        logged_operations: tree_of(TREE_KIND_LOGGED_OPERATIONS)?,
        retirement: tree_of(TREE_KIND_RETIREMENT)?,
    })
}

/// 重建一版时的读：位置提示读不出时经这一版的中央映射回退（D19（块指针的结构与宽度预算） 已定项 8），映射树根到第一次要回退时才读。
/// 回退的次数这里不交出去：`RebuiltVersion` 与可写挂载今天没有接多跳观测点的口子（挂载态的读有，`MountedPoolForRead`）。
struct VersionReadingThroughTheCentralMapping<'reading> {
    reader: &'reading dyn PoolReader,
    central_mapping_root: CentralMappingTreeWithBytesReadOnFirstUse<'reading>,
    stale_location_hint_hops_not_exposed_by_this_reader: usize,
}

impl VersionReadingThroughTheCentralMapping<'_> {
    /// 按映射 key 查这一版中央映射里的两条位置条目（映射树根到第一次查时才读）：提示读不出的回退都经这一处。
    pub(super) fn central_mapping_locations_of_key(
        central_mapping_root: &CentralMappingTreeWithBytesReadOnFirstUse<'_>,
        mapping_key: &[u8],
    ) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> {
        central_mapping_root.locations_of_key(mapping_key)
    }

    /// 一个进映射的树节点（`read_mapped_tree_node_via_hint_then_central_mapping`）。
    fn read_mapped_tree_node(
        &mut self,
        pointer: &NodePointer,
        class: MappedTreeNodeClass,
    ) -> Result<Vec<u8>, RecoveryFailure> {
        let central_mapping_root = &self.central_mapping_root;
        read_mapped_tree_node_via_hint_then_central_mapping(
            self.reader,
            pointer,
            class,
            &|mapping_key: &[u8]| {
                Self::central_mapping_locations_of_key(central_mapping_root, mapping_key)
            },
            &mut self.stale_location_hint_hops_not_exposed_by_this_reader,
        )
    }

    /// 一个数据单元的字节；提示与映射都读不出时字节是空的、不报错（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定 N2）：
    /// 挂载照常，读到那个文件时才报错。
    fn read_data_unit_or_nothing_when_unreadable(
        &mut self,
        data_pointer: &DataPointer,
    ) -> Result<Vec<u8>, RecoveryFailure> {
        let central_mapping_root = &self.central_mapping_root;
        Ok(
            match read_data_unit_via_hint_then_central_mapping(
                self.reader,
                data_pointer,
                &|mapping_key: &[u8]| {
                    Self::central_mapping_locations_of_key(central_mapping_root, mapping_key)
                },
                &mut self.stale_location_hint_hops_not_exposed_by_this_reader,
            )? {
                DataUnitReadThroughTheCentralMapping::Content(bytes) => bytes,
                DataUnitReadThroughTheCentralMapping::MissingFromTheMapping { .. }
                | DataUnitReadThroughTheCentralMapping::UnreadableAtTheMappedLocation { .. } => {
                    Vec::new()
                }
            },
        )
    }
}

/// 每个文件的数据单元的字节：第一个文件按单元号（第 i 个是文件第 i 个数据单元），第一个文件之外的按 inode 号、单元号。
/// 提示与映射都读不出的那几个字节是空的（[`VersionReadingThroughTheCentralMapping::read_data_unit_or_nothing_when_unreadable`]）。
struct DataUnitContentsOfEveryFile {
    first_file_in_file_order: Vec<Vec<u8>>,
    files_other_than_the_first_in_file_order: Vec<Vec<Vec<u8>>>,
}

/// 每个文件指着的每个数据单元都读回来，照抄进这一版的角色（覆盖写经映射释放它们要这几个 key，写行与暖机照抄它们的字节）：
/// 先第一个文件、再第一个文件之外各文件（按 inode 号）。照抄提示与映射都读不出的那一个的发布只搬它的位置项、不写它的字节，
/// 重写它的发布按新内容装；释放它的发布照映射条目读盘核（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1），不读这里的字节。
fn data_units_of_every_file_read(
    reading: &mut VersionReadingThroughTheCentralMapping<'_>,
    first_file_extents: &ExtentsOfTheFirstFile,
) -> Result<DataUnitContentsOfEveryFile, RecoveryFailure> {
    let mut first_file_in_file_order: Vec<Vec<u8>> =
        Vec::with_capacity(first_file_extents.data_pointers.len());
    for data_pointer in &first_file_extents.data_pointers {
        first_file_in_file_order
            .push(reading.read_data_unit_or_nothing_when_unreadable(data_pointer)?);
    }
    let mut files_other_than_the_first_in_file_order: Vec<Vec<Vec<u8>>> =
        Vec::with_capacity(first_file_extents.files_other_than_the_first.len());
    for file in &first_file_extents.files_other_than_the_first {
        let mut contents_in_file_order: Vec<Vec<u8>> = Vec::with_capacity(file.data_pointers.len());
        for data_pointer in &file.data_pointers {
            contents_in_file_order
                .push(reading.read_data_unit_or_nothing_when_unreadable(data_pointer)?);
        }
        files_other_than_the_first_in_file_order.push(contents_in_file_order);
    }
    Ok(DataUnitContentsOfEveryFile {
        first_file_in_file_order,
        files_other_than_the_first_in_file_order,
    })
}

/// 读回来、判过的 inode 树：根的字节与解开的根，每片叶容器（按根的条目次序）与它的字节。
struct InodeTreeRead {
    root_bytes: Vec<u8>,
    root_node: IndexNodeHeader,
    leaf_containers: Vec<InodeLeafContainerVersion>,
    leaf_container_bytes: Vec<Vec<u8>>,
}

/// inode 树根照冷走读核它的自描述（树 ID、key 宽、出生身份、fsid，与 `read_mapped_tree_root` 同一道）与层级 1，读完叶容器再核 key 区间是子树覆盖区间
/// （[`inode_tree_root_key_range_is_the_subtree_coverage`]）；
/// 零条目的 inode 树根是 unlink 删掉树里最后一条记录之后的那一版（D8（核心索引结构） 已定项 16 第 5 条），不拒。
/// 每一片叶容器：根的条目按分隔 key 升序，逐条读回容器与它装的记录（D8（核心索引结构） 已定项 6），按条目判，没有记录的拒。
fn inode_tree_read_and_judged(
    reading: &mut VersionReadingThroughTheCentralMapping<'_>,
    inode_root_pointer: &NodePointer,
    inode_tree: TreeIdentifier,
    root: &RootRecord,
) -> Result<InodeTreeRead, RecoveryFailure> {
    let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
    let root_bytes =
        reading.read_mapped_tree_node(inode_root_pointer, MappedTreeNodeClass::IndexNode)?;
    let root_node = tree_root_checked_against_its_pointer(
        &root_bytes,
        inode_tree,
        key_width_for_kind(TREE_KIND_INODE).expect("inode 树的 key 宽登记过"),
        inode_root_pointer,
        root,
        expected_filesystem_identifier,
    )?;
    inode_tree_root_level_is_one(&root_node)?;
    let mut leaf_containers: Vec<InodeLeafContainerVersion> = Vec::new();
    let mut leaf_container_bytes: Vec<Vec<u8>> = Vec::new();
    let mut largest_inode_number_of_the_leaves_read = None;
    for entry in &root_node.entries {
        let (separator_key, identity_in_the_entry, leaf_pointer) =
            inode_internal_entry_parsed_and_judged(entry)?;
        let leaf_bytes =
            reading.read_mapped_tree_node(&leaf_pointer, MappedTreeNodeClass::PackedRecordUnit)?;
        let leaf =
            parse_packed_unit(&leaf_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
                what: "inode 叶容器",
            })?;
        let records = inode_leaf_container_judged_against_its_entry(
            &leaf,
            InodeInternalEntry {
                separator_key,
                identity: identity_in_the_entry,
                child: leaf_pointer,
            },
            root,
            expected_filesystem_identifier,
        )?;
        if records.is_empty() {
            return Err(RecoveryFailure::UnitMalformed {
                what: "inode 叶容器没有记录",
            });
        }
        // 可写挂载拿重建出来的这一版接着写：写者的前提（全部叶的记录按叶序严格升序）在这里判，不收两片交叠的一版；
        // 分隔 key 大于前面那几片的最大记录号（I-9.12 第三句）同一处判。
        largest_inode_number_of_the_leaves_read =
            inode_records_of_the_leaf_ascend_after_the_leaves_before(
                separator_key,
                largest_inode_number_of_the_leaves_read,
                &records,
            )?;
        leaf_containers.push(InodeLeafContainerVersion {
            // 身份取容器头那一份：条目里的身份引用与它逐字相等是 I-9.2（条目身份与子头相符），上面刚判过。
            contents: InodeLeafContainer {
                identity: leaf.identity,
                // 分隔 key 取条目里那一份：诞生时是容器号，被左邻借过之后是被借那一版的最小 key（D8（核心索引结构） 已定项 6，N1）。
                separator_key_in_the_parent_entry: separator_key,
                records,
            },
            pointer: leaf_pointer,
        });
        leaf_container_bytes.push(leaf_bytes);
    }
    inode_tree_root_key_range_is_the_subtree_coverage(
        &root_node,
        leaf_containers
            .iter()
            .flat_map(|leaf| leaf.contents.records.iter().map(|record| record.inode)),
    )?;
    Ok(InodeTreeRead {
        root_bytes,
        root_node,
        leaf_containers,
        leaf_container_bytes,
    })
}

/// 数据单元照冷走读核，交回第一个文件那条 inode 记录（unlink 最后一批删掉它之后的版本没有它，那时 extent 树里也一个数据单元都没有）。
/// 第一个文件：个数对得上 inode size，size 之内读到手的每一个按查找路径与 inode 记录核；size 之外还挂着的单元（截断中间态，判决 m4-trunc-r1 K2）
/// 不按声明长度核。第一个文件之外各文件同一套核：那个 inode 要有记录，单元数对得上它的 size（尾部照标签声明的）。
/// inode 树里有记录、上段里没有它的条目的第一个文件之外的号，size 要是 0（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md`
/// 第二节 S2：「有记录、没有条目、size > 0」挂载态读者、冷读者、池级 checker 与 GPU 两个内核判同一个结论——读者拒）。
/// 提示与映射都读不出的那几个（字节是空的）不核（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定 N2）。
fn data_units_judged_against_the_inode_records(
    first_file_extents: &ExtentsOfTheFirstFile,
    data_unit_contents: &DataUnitContentsOfEveryFile,
    inode_leaf_containers: &[InodeLeafContainerVersion],
    extent_tree: TreeIdentifier,
    expected_filesystem_identifier: u64,
) -> Result<Option<InodeRecord>, RecoveryFailure> {
    // 按 inode 号找记录走建一次的索引（七·6 后续批代码轮第二轮判决第五节，用户 2026-10-23 定并进）：第一个文件之外每个文件各找一次，
    // 逐个在全部叶容器的记录上找是文件数的平方。索引按叶容器的次序、容器里记录的次序收，同一个号取最先那一条，
    // 与在记录上 `find` 同一条（`FirstEntryOfEachIdentity`）。这一处的建与查都在进的那一处里，函数返回时退出。
    let _inside_the_inode_records = crate::transaction::lookup_work::entering(
        crate::transaction::lookup_work::LookupSite::InodeRecordsOfTheFilesJudgedByTheRecovery,
    );
    let inode_records_in_container_order: Vec<&InodeRecord> = inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .collect();
    let inode_records_by_number = crate::transaction::FirstEntryOfEachIdentity::of_the_table(
        &inode_records_in_container_order,
        |inode_record| inode_record.inode,
    );
    let inode_record_of = |inode: u64| {
        inode_records_by_number
            .first_entry_of(&inode)
            .map(|inode_record| **inode_record)
    };
    let inode_record = inode_record_of(FIRST_INODE_NUMBER);
    match (&inode_record, first_file_extents.data_pointers.is_empty()) {
        (None, true) => {}
        (None, false) => {
            return Err(RecoveryFailure::UnitMalformed {
                what: "inode 树里没有第一个文件那条记录，extent 树里却有它的数据单元",
            });
        }
        (Some(inode_record_of_the_first_file), _) => {
            let data_units_within_the_size = data_unit_count_matches_the_inode_size(
                first_file_extents.data_pointers.len(),
                first_file_extents.tail,
                inode_record_of_the_first_file,
            )?;
            for (position, (data_pointer, content)) in first_file_extents
                .data_pointers
                .iter()
                .zip(&data_unit_contents.first_file_in_file_order)
                .take(data_units_within_the_size)
                .enumerate()
            {
                if content.is_empty() {
                    continue;
                }
                data_unit_judged_against_its_pointer_and_the_inode_record(
                    content,
                    DataUnitOfAFile {
                        position,
                        pointer: data_pointer,
                        extent_tree,
                    },
                    inode_record_of_the_first_file,
                    expected_filesystem_identifier,
                )?;
            }
        }
    }
    for (file, contents_in_file_order) in first_file_extents
        .files_other_than_the_first
        .iter()
        .zip(&data_unit_contents.files_other_than_the_first_in_file_order)
    {
        let inode_record_of_the_file =
            inode_record_of(file.inode.0).ok_or(RecoveryFailure::UnitMalformed {
                what: "inode 树里没有第一个文件之外那个文件的记录，extent 树里却有它的条目",
            })?;
        // 尾部照标签声明的判（判决 m4-multi-inode-size-r1 第二节 S2：别的文件与第一个文件同一判，标签 3、4、0 进得来）。
        let data_units_within_the_size = data_unit_count_matches_the_inode_size(
            file.data_pointers.len(),
            file.tail,
            &inode_record_of_the_file,
        )?;
        for (position, (data_pointer, content)) in file
            .data_pointers
            .iter()
            .zip(contents_in_file_order)
            .take(data_units_within_the_size)
            .enumerate()
        {
            if content.is_empty() {
                continue;
            }
            data_unit_judged_against_its_pointer_and_the_inode_record(
                content,
                DataUnitOfAFile {
                    position,
                    pointer: data_pointer,
                    extent_tree,
                },
                &inode_record_of_the_file,
                expected_filesystem_identifier,
            )?;
        }
    }
    // 有记录、没有条目的别的号：size 0 才收（`publish_new_inodes` 建出来、没改过 size 的；改到 size > 0 的写者照写粘住的标签 0）。
    let inodes_with_an_entry: std::collections::BTreeSet<u64> = first_file_extents
        .files_other_than_the_first
        .iter()
        .map(|file| file.inode.0)
        .collect();
    let some_record_without_an_entry_has_a_size =
        inode_records_in_container_order.iter().any(|record| {
            record.inode != FIRST_INODE_NUMBER
                && !inodes_with_an_entry.contains(&record.inode)
                && record.size != 0
        });
    if some_record_without_an_entry_has_a_size {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: RECORD_WITHOUT_AN_EXTENT_ENTRY_HAS_A_SIZE,
        });
    }
    Ok(inode_record)
}

/// inode 树里有记录、extent 树上段里没有它的条目的第一个文件之外的号 size 不是 0 时 [`RecoveryFailure::InvariantViolated`] 的 `detail`
/// （判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2；与挂载态读者 `DataUnitsOfTheInode::NoneWritten` 配 size > 0
/// 拒的是同一形）。
pub const RECORD_WITHOUT_AN_EXTENT_ENTRY_HAS_A_SIZE: &str =
    "inode 树里第一个文件之外那个号有记录、size 不是 0，extent 树上段里却没有它的条目";

/// `deleted_inodes` 根（D8（核心索引结构） 已定项 16）：根指针、读回来的字节（零根时 `None`）与它装的 inode 号。
struct DeletedInodesRootRead {
    pointer: NodePointer,
    bytes: Option<Vec<u8>>,
    inode_numbers: Vec<InodeNumber>,
}

/// `deleted_inodes` 根是码 2 层级 0（条目 = 号 8 字节）；树表条目的根指针全零就是零根。不是层级 0、条目宽 8 的非空节点的拒。
fn deleted_inodes_root_read(
    reading: &mut VersionReadingThroughTheCentralMapping<'_>,
    tree_table_entries: &[TreeTableEntry],
    deleted_inodes_tree: TreeIdentifier,
    root: &RootRecord,
) -> Result<DeletedInodesRootRead, RecoveryFailure> {
    let pointer = root_pointer_in_the_tree_table(tree_table_entries, TREE_KIND_DELETED_INODES)?;
    if pointer == NodePointer::empty_root() {
        return Ok(DeletedInodesRootRead {
            pointer,
            bytes: None,
            inode_numbers: Vec::new(),
        });
    }
    let bytes = reading.read_mapped_tree_node(&pointer, MappedTreeNodeClass::IndexNode)?;
    let node = tree_root_checked_against_its_pointer(
        &bytes,
        deleted_inodes_tree,
        key_width_for_kind(TREE_KIND_DELETED_INODES).expect("deleted_inodes 的 key 宽登记过"),
        &pointer,
        root,
        unit_filesystem_identifier(&root.filesystem_identifier),
    )?;
    if node.level != 0
        || node.entries.is_empty()
        || node
            .entries
            .iter()
            .any(|entry| entry.len() != DELETED_INODES_KEY_WIDTH_IN_BYTES)
    {
        return Err(RecoveryFailure::UnitMalformed {
            what: "deleted_inodes 根不是层级 0、条目宽 8 的非空节点",
        });
    }
    Ok(DeletedInodesRootRead {
        pointer,
        inode_numbers: inode_numbers_of_the_deleted_inodes_root_entries(&node.entries),
        bytes: Some(bytes),
    })
}

/// logged_ops 根（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）：根指针、读回来的字节（零根时 `None`）与解开的条目。
struct LoggedOperationsRootRead {
    pointer: NodePointer,
    bytes: Option<Vec<u8>>,
    entries: Vec<LoggedOperationEntry>,
}

/// logged_ops 根是码 2 层级 0，条目 44 字节，逐条照字段表解（R 不对齐、槽数不是 64、补零非零、种类码保留、整理意图多于一条都拒：
/// 盘上读来的坏条目当损坏处置，可写挂载拒）。
fn logged_operations_root_read(
    reading: &mut VersionReadingThroughTheCentralMapping<'_>,
    tree_table_entries: &[TreeTableEntry],
    logged_operations_tree: TreeIdentifier,
    root: &RootRecord,
) -> Result<LoggedOperationsRootRead, RecoveryFailure> {
    let pointer = root_pointer_in_the_tree_table(tree_table_entries, TREE_KIND_LOGGED_OPERATIONS)?;
    if pointer == NodePointer::empty_root() {
        return Ok(LoggedOperationsRootRead {
            pointer,
            bytes: None,
            entries: Vec::new(),
        });
    }
    let bytes = reading.read_mapped_tree_node(&pointer, MappedTreeNodeClass::IndexNode)?;
    let node = tree_root_checked_against_its_pointer(
        &bytes,
        logged_operations_tree,
        LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES,
        &pointer,
        root,
        unit_filesystem_identifier(&root.filesystem_identifier),
    )?;
    if node.level != 0 || node.entries.is_empty() {
        return Err(RecoveryFailure::UnitMalformed {
            what: LOGGED_OPERATIONS_ROOT_IS_NOT_A_NON_EMPTY_LEVEL_ZERO_NODE,
        });
    }
    let entries =
        logged_operation_entries_of_the_root_entries(&node.entries).map_err(|_refusal| {
            RecoveryFailure::UnitMalformed {
                what: LOGGED_OPERATIONS_ROOT_ENTRIES_REFUSED_BY_THE_FIELD_TABLE,
            }
        })?;
    Ok(LoggedOperationsRootRead {
        pointer,
        bytes: Some(bytes),
        entries,
    })
}

/// 墓碑树、谱系树、退役树（零根时 `None`）。
struct ContainerTreesOfTheVersionRead {
    tombstone: Option<ContainerTreeRead<TombstoneRecord>>,
    lineage: Option<ContainerTreeRead<LineageRecord>>,
    retirement: Option<ContainerTreeRead<RetirementRecord>>,
}

impl ContainerTreesOfTheVersionRead {
    /// 三棵树进映射的单元数之和。
    fn mapped_units(&self) -> usize {
        self.tombstone
            .as_ref()
            .map_or(0, ContainerTreeRead::mapped_units)
            + self
                .lineage
                .as_ref()
                .map_or(0, ContainerTreeRead::mapped_units)
            + self
                .retirement
                .as_ref()
                .map_or(0, ContainerTreeRead::mapped_units)
    }

    pub(super) fn tombstone_containers(&self) -> Vec<TombstoneContainerVersion> {
        self.tombstone.as_ref().map_or_else(Vec::new, |read| {
            read.containers
                .iter()
                .map(|(contents, pointer, _)| TombstoneContainerVersion {
                    contents: contents.clone(),
                    pointer: *pointer,
                })
                .collect()
        })
    }

    pub(super) fn tombstone_tree_index_nodes(&self) -> ContainerTreeIndexNodesVersion {
        self.tombstone.as_ref().map_or_else(
            ContainerTreeIndexNodesVersion::zero_root,
            ContainerTreeRead::index_nodes_version,
        )
    }

    pub(super) fn lineage_containers(&self) -> Vec<LineageContainerVersion> {
        self.lineage.as_ref().map_or_else(Vec::new, |read| {
            read.containers
                .iter()
                .map(|(contents, pointer, _)| LineageContainerVersion {
                    contents: contents.clone(),
                    pointer: *pointer,
                })
                .collect()
        })
    }

    pub(super) fn lineage_tree_index_nodes(&self) -> ContainerTreeIndexNodesVersion {
        self.lineage.as_ref().map_or_else(
            ContainerTreeIndexNodesVersion::zero_root,
            ContainerTreeRead::index_nodes_version,
        )
    }

    pub(super) fn retirement_containers(&self) -> Vec<RetirementContainerVersion> {
        self.retirement.as_ref().map_or_else(Vec::new, |read| {
            read.containers
                .iter()
                .map(|(contents, pointer, _)| RetirementContainerVersion {
                    contents: contents.clone(),
                    pointer: *pointer,
                })
                .collect()
        })
    }

    pub(super) fn retirement_tree_index_nodes(&self) -> ContainerTreeIndexNodesVersion {
        self.retirement.as_ref().map_or_else(
            ContainerTreeIndexNodesVersion::zero_root,
            ContainerTreeRead::index_nodes_version,
        )
    }
}

/// 墓碑树与谱系树（D18（块里携带什么信息） 已定项 24 第 1、5 条）：根恒为码 2，条目照 inode 树内部节点的 120 字节形态；根满长高之后
/// 根之下是类型段 0 的码 2 节点，层级 1 的节点的孩子是打包记录容器（墓碑类型 1、谱系类型 5）。两棵树读法相同，只差 `ContainerTreeToRead` 那几样。
/// 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条：形态照墓碑树，容器打包记录类型 3、记录宽 34）：
/// 接着这一版发布要往开放容器追加、算内部层数，整棵读回来，同墓碑树、谱系树。按墓碑、谱系、退役的次序读。
fn container_trees_read(
    reading: &mut VersionReadingThroughTheCentralMapping<'_>,
    tree_table_entries: &[TreeTableEntry],
    tree_identifiers: &FileVersionTreeIdentifiers,
    root: &RootRecord,
) -> Result<ContainerTreesOfTheVersionRead, RecoveryFailure> {
    let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
    let mut read_container_tree_node = |pointer: &NodePointer, class: MappedTreeNodeClass| {
        reading.read_mapped_tree_node(pointer, class)
    };
    let tombstone_root_pointer =
        root_pointer_in_the_tree_table(tree_table_entries, TREE_KIND_TOMBSTONE)?;
    let tombstone = read_container_tree(
        &ContainerTreeToRead {
            tree: tree_identifiers.tombstone,
            kind: TREE_KIND_TOMBSTONE,
            record_type: PACKED_TYPE_TOMBSTONE,
            record_width_in_bytes: TOMBSTONE_RECORD_BYTES,
            parse_record: TombstoneRecord::parse,
            what_index_node: "墓碑树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",
            what_entry: "墓碑树内部条目",
            what_container: "墓碑容器",
            what_container_against_its_entry: "墓碑容器与父节点里指它的条目对不上（身份、类型 1、出生树、分隔 key = 容器号、记录宽 40、非空）",
            what_record: "墓碑记录",
        },
        &tombstone_root_pointer,
        root,
        expected_filesystem_identifier,
        &mut read_container_tree_node,
    )?;
    let lineage_root_pointer =
        root_pointer_in_the_tree_table(tree_table_entries, TREE_KIND_LINEAGE)?;
    let lineage = read_container_tree(
        &ContainerTreeToRead {
            tree: tree_identifiers.lineage,
            kind: TREE_KIND_LINEAGE,
            record_type: PACKED_TYPE_LINEAGE,
            record_width_in_bytes: LINEAGE_RECORD_BYTES,
            parse_record: LineageRecord::parse,
            what_index_node: "谱系树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",
            what_entry: "谱系树内部条目",
            what_container: "谱系容器",
            what_container_against_its_entry: "谱系容器与父节点里指它的条目对不上（身份、类型 5、出生树、分隔 key = 容器号、记录宽 24、非空）",
            what_record: "谱系记录",
        },
        &lineage_root_pointer,
        root,
        expected_filesystem_identifier,
        &mut read_container_tree_node,
    )?;
    let retirement_root_pointer =
        root_pointer_in_the_tree_table(tree_table_entries, TREE_KIND_RETIREMENT)?;
    let retirement = read_container_tree(
        &ContainerTreeToRead {
            tree: tree_identifiers.retirement,
            kind: TREE_KIND_RETIREMENT,
            record_type: PACKED_TYPE_RETIREMENT,
            record_width_in_bytes: RETIREMENT_RECORD_BYTES,
            parse_record: RetirementRecord::parse,
            what_index_node: "退役树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",
            what_entry: "退役树内部条目",
            what_container: "退役容器",
            what_container_against_its_entry: "退役容器与父节点里指它的条目对不上（身份、类型 3、出生树、分隔 key = 容器号、记录宽 34、非空）",
            what_record: "退役记录",
        },
        &retirement_root_pointer,
        root,
        expected_filesystem_identifier,
        &mut read_container_tree_node,
    )?;
    Ok(ContainerTreesOfTheVersionRead {
        tombstone,
        lineage,
        retirement,
    })
}

/// 分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14）：整棵读回来，每个节点按位置核，节点进映射、提示读不出经映射回退；
/// 记录落在单元区里（`unit_area_start` 起）、每块盘一份落点。记录的代与跨度由调用方紧接着判（`allocation_record_generations_and_spans_are_judged`）。
fn allocation_record_tree_read_and_judged(
    reading: &mut VersionReadingThroughTheCentralMapping<'_>,
    unit_area_start: UnitAreaStart,
    allocation_pointer: &NodePointer,
    allocation_record_tree: TreeIdentifier,
    root: &RootRecord,
) -> Result<AllocationRecordTreeReadFromDisk, RebuildVersionFailure> {
    let reader = reading.reader;
    let allocation_tree_read = read_allocation_record_tree(
        allocation_pointer,
        &AllocationRecordTreeGeometry::of_reader(reader),
        allocation_record_tree,
        AllocationRecordTreeHeaderJudgement::EveryHeaderAgainstItsReference,
        root,
        unit_filesystem_identifier(&root.filesystem_identifier),
        &mut |pointer: &NodePointer| {
            reading.read_mapped_tree_node(pointer, MappedTreeNodeClass::IndexNode)
        },
    )?;
    allocation_records_fit_the_pool_geometry(
        reader,
        unit_area_start,
        &allocation_tree_read.records,
    )?;
    allocation_records_are_one_placement_per_device_on_every_device(
        reader,
        unit_area_start,
        &allocation_tree_read.records,
        &reader.device_identities(),
    )?;
    Ok(allocation_tree_read)
}

/// 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退，照冷走读判全
/// （`code_two_tree::read_code_two_tree` 的 `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，
/// 挂载之后第一次发布规划那棵树时在 `transaction` 的断言上 panic，代码审阅第 33 条）。
/// 再判条目：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`
/// 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。
/// 最后判这一版自己写的 inode 号水位那一行在（代段 = 根的 txg；`TransactionOutput::inode_number_watermark` 那句 expect 依赖的就是这一判，
/// panic 面普查 R11）：那一统计量另有代段更早的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程），只剩台阶行时现行值取到的是一级台阶。
fn accounting_tree_read_and_judged(
    reading: &mut VersionReadingThroughTheCentralMapping<'_>,
    accounting_pointer: &NodePointer,
    accounting_tree: TreeIdentifier,
    root: &RootRecord,
) -> Result<(CodeTwoTreeReadFromDisk, Vec<AccountingEntry>), RebuildVersionFailure> {
    let devices_in_the_pool = reading.reader.device_identities().len();
    let accounting_tree_read = read_code_two_tree(
        accounting_pointer,
        &MultiLevelCodeTwoTree::Accounting.read_expectation(accounting_tree),
        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
        root,
        unit_filesystem_identifier(&root.filesystem_identifier),
        &mut |pointer: &NodePointer| {
            reading.read_mapped_tree_node(pointer, MappedTreeNodeClass::IndexNode)
        },
    )?;
    every_accounting_entry_is_as_wide_as_its_field_table(
        &accounting_tree_read.leaf_entries_in_key_order,
    )?;
    let mut accounting_entries: Vec<AccountingEntry> =
        Vec::with_capacity(accounting_tree_read.leaf_entries_in_key_order.len());
    for bytes in &accounting_tree_read.leaf_entries_in_key_order {
        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(
            bytes, root,
        )?);
    }
    accounting_entry_count_is_three_plus_six_per_device(
        &accounting_tree_read.leaf_entries_in_key_order,
        root.checkpoint_txg,
        devices_in_the_pool,
    )?;
    if !accounting_entries.iter().any(|entry| {
        entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == root.checkpoint_txg
    }) {
        return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());
    }
    Ok((accounting_tree_read, accounting_entries))
}

/// deadlist 树（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3：第三棵多层码 2 树）：树表条目根指针非零时整棵读回来，
/// 同记账树（节点进映射、提示读不出经映射回退、照冷走读判全）；零根时一条条目都没有。接着这一版发布要按它规划、删快照的批要从它取条目。
fn deadlist_tree_read(
    reading: &mut VersionReadingThroughTheCentralMapping<'_>,
    tree_table_entries: &[TreeTableEntry],
    deadlist_tree: TreeIdentifier,
    root: &RootRecord,
) -> Result<(CodeTwoTreeReadFromDisk, Vec<DeadlistEntry>), RecoveryFailure> {
    let deadlist_pointer = root_pointer_in_the_tree_table(tree_table_entries, TREE_KIND_DEADLIST)?;
    let deadlist_tree_read = if deadlist_pointer == NodePointer::empty_root() {
        CodeTwoTreeReadFromDisk {
            version: CodeTwoTreeVersion::default(),
            node_bytes: Vec::new(),
            leaf_entries_in_key_order: Vec::new(),
        }
    } else {
        read_code_two_tree(
            &deadlist_pointer,
            &MultiLevelCodeTwoTree::Deadlist.read_expectation(deadlist_tree),
            CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
            root,
            unit_filesystem_identifier(&root.filesystem_identifier),
            &mut |pointer: &NodePointer| {
                reading.read_mapped_tree_node(pointer, MappedTreeNodeClass::IndexNode)
            },
        )?
    };
    let deadlist_entries = deadlist_entries_parsed(&deadlist_tree_read.leaf_entries_in_key_order)?;
    Ok((deadlist_tree_read, deadlist_entries))
}

/// 删快照的意图写在被删那条码 14 自己身上（判决 S4，flags 位 0）：读回来，接着这一版的发布照它续写记号、走批。
fn snapshots_being_destroyed_in_the_tree_table(
    tree_table: &IndexNodeHeader,
) -> Vec<TreeIdentifier> {
    tree_table
        .entries
        .iter()
        .filter_map(|entry_bytes| {
            TreeTableEntry::parse_with_destruction_mark(entry_bytes.as_slice())
        })
        .filter(|(_, mark)| *mark == SnapshotDestructionMark::BeingDestroyed)
        .map(|(entry, _)| entry.tree)
        .collect()
}

/// 逐条解开映射树的叶条目（宽不对的拒）。deadlist 条目按 key 在中央映射树里查不到：C300（deadlist 条目按 key 查映射查不到没有条款）没有条款，
/// 第一版拒这一版（删快照的批要从那条映射条目取落点，取不到就放不了），在交给发布路径之前。
fn mapping_entries_of_the_version(
    mapping_tree: &CodeTwoTreeReadFromDisk,
    deadlist_entries: &[DeadlistEntry],
) -> Result<MappingEntriesOfTheVersion, RecoveryFailure> {
    let mut mapping_keys: Vec<Vec<u8>> =
        Vec::with_capacity(mapping_tree.leaf_entries_in_key_order.len());
    let deadlist_mapping_keys: BTreeSet<&[u8]> = deadlist_entries
        .iter()
        .map(|entry| entry.mapping_key.as_slice())
        .collect();
    let mut mapping_entries_kept_for_the_snapshots: Vec<(Vec<u8>, [LocationEntry; 2])> =
        Vec::with_capacity(deadlist_entries.len());
    let mut mapping_position_of_key: BTreeMap<Vec<u8>, [LocationEntry; 2]> = BTreeMap::new();
    for entry in &mapping_tree.leaf_entries_in_key_order {
        let (key, locations) = parse_mapping_entry(entry).ok_or(
            RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
                what: "映射条目",
                entry_bytes: entry.len(),
                field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
            },
        )?;
        if deadlist_mapping_keys.contains(key.as_slice()) {
            mapping_entries_kept_for_the_snapshots.push((key.clone(), locations));
        }
        mapping_position_of_key.insert(key.clone(), locations);
        mapping_keys.push(key);
    }
    if mapping_entries_kept_for_the_snapshots.len() != deadlist_mapping_keys.len()
        || deadlist_mapping_keys.len() != deadlist_entries.len()
    {
        return Err(RecoveryFailure::UnitMalformed {
            what: "deadlist 条目的映射 key 在中央映射树里查不到，或两条 deadlist 条目同一把映射 key（C300 没有条款）",
        });
    }
    Ok(MappingEntriesOfTheVersion {
        mapping_keys,
        mapping_entries_kept_for_the_snapshots,
        mapping_position_of_key,
    })
}

/// 重建一版读回来、判过的全部树与数据单元（自举的实例表、树表除外），装 `mapped_units` 与 `units` 用。
struct TreesOfTheVersionRead {
    first_file_extents: ExtentsOfTheFirstFile,
    data_unit_contents: DataUnitContentsOfEveryFile,
    inode_tree: InodeTreeRead,
    inode_root_pointer: NodePointer,
    deleted_inodes_root: DeletedInodesRootRead,
    logged_operations_root: LoggedOperationsRootRead,
    container_trees: ContainerTreesOfTheVersionRead,
    allocation_tree_read: AllocationRecordTreeReadFromDisk,
    accounting_tree: CodeTwoTreeReadFromDisk,
    deadlist_tree: CodeTwoTreeReadFromDisk,
    mapping_tree: CodeTwoTreeReadFromDisk,
}

/// 第一个文件第 `position` 个数据单元的角色。
fn data_role_of_the_first_file_at(position: usize) -> TransactionUnit {
    TransactionUnit::Data(DataUnitIndexInFile(
        u64::try_from(position).expect("单元序号"),
    ))
}

/// 第一个文件之外那个文件第 `position` 个数据单元的角色。
fn data_role_of_a_file_other_than_the_first_at(
    file: &ExtentsOfAFileOtherThanTheFirst,
    position: usize,
) -> TransactionUnit {
    TransactionUnit::DataOfAFileOtherThanTheFirst {
        inode: file.inode,
        unit: DataUnitIndexInFile(u64::try_from(position).expect("单元序号")),
    }
}

/// 这一版进映射的每个单元（角色与映射 key），与发布路径的 `mapped_units` 同序：第一个文件的数据单元、第一个文件之外各文件的数据单元
/// （按 inode 号、单元序号）、extent 树每个节点（bump 次序）、inode 叶容器、inode 树根、分配记录树每个节点、记账树每个节点（bump 次序、根在最末）、
/// deadlist 树每个节点、删除那两棵树与 logged_ops 根、退役树（按树 ID；容器树每片容器、根之下的码 2 节点、根）。
fn mapped_units_of_the_version(
    trees_read: &TreesOfTheVersionRead,
) -> Vec<(TransactionUnit, Vec<u8>)> {
    let first_file_extents = &trees_read.first_file_extents;
    let mut mapped_units: Vec<(TransactionUnit, Vec<u8>)> = first_file_extents
        .data_pointers
        .iter()
        .enumerate()
        .map(|(position, data_pointer)| {
            (
                data_role_of_the_first_file_at(position),
                mapping_key_for_data(data_pointer.head, data_pointer.write_order),
            )
        })
        .collect();
    for file in &first_file_extents.files_other_than_the_first {
        for (position, data_pointer) in file.data_pointers.iter().enumerate() {
            mapped_units.push((
                data_role_of_a_file_other_than_the_first_at(file, position),
                mapping_key_for_data(data_pointer.head, data_pointer.write_order),
            ));
        }
    }
    for (role, pointer, _) in &first_file_extents.nodes_in_bump_order {
        mapped_units.push((*role, mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer)));
    }
    for (position, container) in trees_read.inode_tree.leaf_containers.iter().enumerate() {
        mapped_units.push((
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(
                position,
            )),
            mapping_key_for_node(UNIT_CLASS_PACKED, container.pointer),
        ));
    }
    mapped_units.push((
        TransactionUnit::InodeRoot,
        mapping_key_for_node(UNIT_CLASS_INDEX_NODE, trees_read.inode_root_pointer),
    ));
    for (node, pointer, _) in &trees_read.allocation_tree_read.nodes {
        mapped_units.push((
            role_of_allocation_record_tree_node(*node),
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
        ));
    }
    // 记账树的每个节点都进映射（码 2 树节点，D19（块指针的结构与宽度预算） 已定项 8），按 bump 次序、根在最末。
    let accounting_tree = &trees_read.accounting_tree;
    let accounting_shape = &accounting_tree.version.shape;
    for (node, pointer) in accounting_shape
        .nodes()
        .iter()
        .zip(&accounting_tree.version.pointers)
    {
        mapped_units.push((
            MultiLevelCodeTwoTree::Accounting.role_of_node(node.position, accounting_shape),
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
        ));
    }
    // deadlist 树的每个节点接在记账树后面（按树 ID），与发布路径的 `mapped_units` 同序。
    let deadlist_tree = &trees_read.deadlist_tree;
    let deadlist_shape = &deadlist_tree.version.shape;
    for (node, pointer) in deadlist_shape
        .nodes()
        .iter()
        .zip(&deadlist_tree.version.pointers)
    {
        mapped_units.push((
            MultiLevelCodeTwoTree::Deadlist.role_of_node(node.position, deadlist_shape),
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
        ));
    }
    if trees_read.deleted_inodes_root.bytes.is_some() {
        mapped_units.push((
            TransactionUnit::DeletedInodesRoot,
            mapping_key_for_node(
                UNIT_CLASS_INDEX_NODE,
                trees_read.deleted_inodes_root.pointer,
            ),
        ));
    }
    let mapped_units_of_a_container_tree =
        |tree_read: Option<Vec<(UnitOfAContainerTree, NodePointer, Vec<u8>)>>,
         roles: ContainerTreeRolesOfOneTree| {
            tree_read
                .unwrap_or_default()
                .into_iter()
                .map(move |(unit_of_the_tree, pointer, _)| {
                    (
                        roles.of(unit_of_the_tree),
                        mapping_key_for_node(unit_of_the_tree.unit_class(), pointer),
                    )
                })
        };
    let container_trees = &trees_read.container_trees;
    mapped_units.extend(mapped_units_of_a_container_tree(
        container_trees
            .tombstone
            .as_ref()
            .map(ContainerTreeRead::units_in_bump_order),
        CONTAINER_TREE_ROLES_OF_THE_TOMBSTONE_TREE,
    ));
    mapped_units.extend(mapped_units_of_a_container_tree(
        container_trees
            .lineage
            .as_ref()
            .map(ContainerTreeRead::units_in_bump_order),
        CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
    ));
    if trees_read.logged_operations_root.bytes.is_some() {
        mapped_units.push((
            TransactionUnit::LoggedOperationsRoot,
            mapping_key_for_node(
                UNIT_CLASS_INDEX_NODE,
                trees_read.logged_operations_root.pointer,
            ),
        ));
    }
    mapped_units.extend(mapped_units_of_a_container_tree(
        container_trees
            .retirement
            .as_ref()
            .map(ContainerTreeRead::units_in_bump_order),
        CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE,
    ));
    mapped_units
}

/// 第一个文件之外各文件在这一版里的样子：数据指针、下段节点（不带字节）与标签声明的尾部。
fn files_other_than_the_first_of_the_version(
    trees_read: &TreesOfTheVersionRead,
) -> Vec<FileOfAnInodeOtherThanTheFirst> {
    trees_read
        .first_file_extents
        .files_other_than_the_first
        .iter()
        .map(|file| FileOfAnInodeOtherThanTheFirst {
            inode: file.inode,
            data_pointers: file.data_pointers.clone(),
            extent_lower_nodes: file
                .lower_nodes
                .iter()
                .map(|(position, pointer, _)| (*position, *pointer))
                .collect(),
            tail: file.tail,
        })
        .collect()
}

/// 这一版的每个单元（角色、槽与字节），与发布路径装出来的 `units` 同序：第一个文件与第一个文件之外各文件的数据单元、extent 树每个节点、
/// inode 叶容器、inode 树根、分配记录树每个节点；再是记账树、deadlist 树，删除那两棵树与 logged_ops 根、退役树（按树 ID，接在中央映射树之前），
/// 中央映射树，每棵按 bump 次序（树内先叶后根）；最后树表与实例表。
fn units_of_the_version(
    trees_read: TreesOfTheVersionRead,
    root: &RootRecord,
    bootstrap_units: BootstrapUnitsOfTheRoot,
) -> Vec<PublishedUnit> {
    let unit =
        |identity: TransactionUnit, locations: &[LocationEntry; 2], bytes: Vec<u8>| PublishedUnit {
            slot: locations[0].slot,
            identity,
            bytes: Arc::from(bytes),
        };
    let TreesOfTheVersionRead {
        first_file_extents,
        data_unit_contents,
        inode_tree,
        inode_root_pointer,
        deleted_inodes_root,
        logged_operations_root,
        container_trees,
        allocation_tree_read,
        accounting_tree,
        deadlist_tree,
        mapping_tree,
    } = trees_read;
    let mut units: Vec<PublishedUnit> = first_file_extents
        .data_pointers
        .iter()
        .zip(data_unit_contents.first_file_in_file_order)
        .enumerate()
        .map(|(position, (data_pointer, bytes))| {
            unit(
                data_role_of_the_first_file_at(position),
                &data_pointer.locations,
                bytes,
            )
        })
        .collect();
    for (file, contents_in_file_order) in first_file_extents
        .files_other_than_the_first
        .iter()
        .zip(data_unit_contents.files_other_than_the_first_in_file_order)
    {
        for (position, (data_pointer, bytes)) in file
            .data_pointers
            .iter()
            .zip(contents_in_file_order)
            .enumerate()
        {
            units.push(unit(
                data_role_of_a_file_other_than_the_first_at(file, position),
                &data_pointer.locations,
                bytes,
            ));
        }
    }
    for (role, pointer, bytes) in first_file_extents.nodes_in_bump_order {
        units.push(unit(role, &pointer.locations, bytes));
    }
    for ((position, container), bytes) in inode_tree
        .leaf_containers
        .iter()
        .enumerate()
        .zip(inode_tree.leaf_container_bytes)
    {
        units.push(unit(
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(
                position,
            )),
            &container.pointer.locations,
            bytes,
        ));
    }
    units.push(unit(
        TransactionUnit::InodeRoot,
        &inode_root_pointer.locations,
        inode_tree.root_bytes,
    ));
    for (node, pointer, bytes) in allocation_tree_read.nodes {
        units.push(unit(
            role_of_allocation_record_tree_node(node),
            &pointer.locations,
            bytes,
        ));
    }
    let mut deletion_tree_units: Vec<PublishedUnit> = Vec::new();
    if let Some(bytes) = deleted_inodes_root.bytes {
        deletion_tree_units.push(unit(
            TransactionUnit::DeletedInodesRoot,
            &deleted_inodes_root.pointer.locations,
            bytes,
        ));
    }
    for (tree_read, roles) in [
        (
            container_trees
                .tombstone
                .as_ref()
                .map(ContainerTreeRead::units_in_bump_order),
            CONTAINER_TREE_ROLES_OF_THE_TOMBSTONE_TREE,
        ),
        (
            container_trees
                .lineage
                .as_ref()
                .map(ContainerTreeRead::units_in_bump_order),
            CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
        ),
    ] {
        for (unit_of_the_tree, pointer, bytes) in tree_read.unwrap_or_default() {
            deletion_tree_units.push(unit(roles.of(unit_of_the_tree), &pointer.locations, bytes));
        }
    }
    // logged_ops 根按树 ID 排在谱系树之后、退役树之前（与发布路径装出来的 `units` 同序）。
    if let Some(bytes) = logged_operations_root.bytes {
        deletion_tree_units.push(unit(
            TransactionUnit::LoggedOperationsRoot,
            &logged_operations_root.pointer.locations,
            bytes,
        ));
    }
    for (unit_of_the_tree, pointer, bytes) in container_trees
        .retirement
        .as_ref()
        .map(ContainerTreeRead::units_in_bump_order)
        .unwrap_or_default()
    {
        deletion_tree_units.push(unit(
            CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE.of(unit_of_the_tree),
            &pointer.locations,
            bytes,
        ));
    }
    let mut deletion_tree_units_not_yet_placed = Some(deletion_tree_units);
    for (tree, read) in [
        (MultiLevelCodeTwoTree::Accounting, &accounting_tree),
        (MultiLevelCodeTwoTree::Deadlist, &deadlist_tree),
        (MultiLevelCodeTwoTree::CentralMapping, &mapping_tree),
    ] {
        if tree == MultiLevelCodeTwoTree::CentralMapping {
            units.extend(
                deletion_tree_units_not_yet_placed
                    .take()
                    .expect("删除那两棵树的单元只在中央映射树之前放一次"),
            );
        }
        let shape = &read.version.shape;
        for ((node, pointer), bytes) in shape
            .nodes()
            .iter()
            .zip(&read.version.pointers)
            .zip(&read.node_bytes)
        {
            units.push(unit(
                tree.role_of_node(node.position, shape),
                &pointer.locations,
                bytes.clone(),
            ));
        }
    }
    units.extend([
        unit(
            TransactionUnit::TreeTable,
            &root.tree_table.locations,
            bootstrap_units.tree_table_bytes,
        ),
        unit(
            TransactionUnit::InstanceTable,
            &root.instance_table.locations,
            bootstrap_units.instance_table_bytes,
        ),
    ]);
    units
}

/// 同 [`rebuild_version_in_the_unit_area_starting_at`]，单元区起点先择一次系统配置现算（[`choose_system_configuration`]：
/// 多读每块盘的两个系统配置槽）。给手里没有择好的系统配置的调用方（用例）；可写挂载与管理员回退走带起点的那一份。
///
/// # Errors
/// 择不到系统配置 ⇒ `Walk(`[`choose_system_configuration`] 的各个成员`)`；其余同带起点的那一份。
pub fn rebuild_version(
    reader: &dyn PoolReader,
    root: &RootRecord,
    record_standing_for_root: Option<JournalRecord>,
) -> Result<RebuiltVersion, RebuildVersionFailure> {
    let system_configuration = choose_system_configuration(reader)?;
    rebuild_version_in_the_unit_area_starting_at(
        reader,
        unit_area_start_of_the_chosen_system_configuration(&system_configuration),
        root,
        record_standing_for_root,
    )
}

/// 同 [`rebuild_version`]（单元区起点先择一次系统配置现算），另交出树表里的只读快照条目
/// （[`rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at`]）。给手里没有择好的系统配置的调用方（用例）。
///
/// # Errors
/// 同 [`rebuild_version`]。
pub fn rebuild_version_with_its_snapshot_entries(
    reader: &dyn PoolReader,
    root: &RootRecord,
    record_standing_for_root: Option<JournalRecord>,
) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
    let system_configuration = choose_system_configuration(reader)?;
    rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
        reader,
        unit_area_start_of_the_chosen_system_configuration(&system_configuration),
        root,
        record_standing_for_root,
    )
}
