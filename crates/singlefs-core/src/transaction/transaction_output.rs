//! 一次发布的产物 `TransactionOutput`：各单元的落点、容器树的几版、按位置寻址的树高与从这一版里查单元。

use super::*;

/// 新池新建文件写出的东西，留给验收与探针用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionOutput {
    pub root: RootRecord,
    /// 这次发布的末条记录：带「本次发布末条」标志的那一条（D23（journal 的角色与格式） 已定项 17），
    /// 也是所选根覆盖到的最后一条（已定项 14 注 1）——下一次发布的 jsn 与反向链接在它后面。只有一条记录的发布就是那一条。
    pub record: JournalRecord,
    pub record_bytes: Vec<u8>,
    /// 这次发布末条之前的那几条，按 jsn 升序：一次发布切成 N 个事务时前 N − 1 个事务各一条、各只点名自己那个数据单元，
    /// 最后一个事务装不下一条记录时再跨出去的那几条（末条除外）也在这里（D23（journal 的角色与格式） 已定项 17）。
    /// 只有一条记录的发布、从盘上重建出来的版本都为空。
    pub earlier_records_of_this_publish: Vec<WrittenJournalRecord>,
    /// 这一版全部角色的单元：这次重写的是新装的，没重写的从上一版照抄（文件 / 固定点角色按 bump 次序——每个数据单元、extent 根、
    /// 每片 inode 叶容器、inode 根、固定点那几族（分配记录树、记账树、中央映射树各连同根之下的节点，加树表单元）——实例表链的各片在末尾（按链上的次序，第 0 片在前）、mkfs 之后第一次重写之前不在）。
    pub units: Vec<PublishedUnit>,
    /// 这次发布真正写出的角色，按写出的次序（点名项与录制流里的单元写只有这些）。
    pub rewritten: Vec<TransactionUnit>,
    /// 这一版文件每个数据单元的指针，第 i 项是文件第 i 个单元（extent 根兼叶里的记录按同一次序排）。
    pub data_pointers: Vec<DataPointer>,
    /// 这一版里第一个文件之外有数据单元的文件，按 inode 号升序（按 inode 号写数据的入口写出来的，[`publish_sequential_write_of_an_inode`]）：
    /// 每个文件的数据指针与它那一棵下段的节点。只写过建 inode（size 0）的 inode 没有数据单元，不在这里。它们的单元在 `units` 里排在第一个文件的
    /// 数据单元之后（数据单元）、第一个文件的下段之后（下段节点），进映射的次序同。
    pub files_other_than_the_first: Vec<FileOfAnInodeOtherThanTheFirst>,
    pub mapping_keys: Vec<Vec<u8>>,
    /// 这一版的全部分配记录，按 (设备, 槽号) 升序（它们分装在分配记录树的各片叶里）。
    pub allocation_records: Vec<AllocationRecord>,
    /// 这一版分配记录树的每个节点与它的指针（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）；节点的字节住 `units` 里分配记录树那一族角色。
    pub allocation_record_tree: AllocationRecordTreeVersion,
    /// 这一版 extent 树上段与第一个文件下段的每个节点与它的指针（D8（核心索引结构） 已定项 14：两段按位置寻址）；
    /// 节点的字节住 `units` 里 extent 树那一族角色。
    pub extent_tree: ExtentTreeVersion,
    pub accounting_entries: Vec<AccountingEntry>,
    /// 这一版记账树的形状与每个节点的指针（D8（核心索引结构） 已定项 11：多层码 2 树）；节点的字节住 `units` 里记账树那一族角色。
    pub accounting_tree: CodeTwoTreeVersion,
    /// 这一版中央映射树的形状与每个节点的指针，同上；根指针同时住根记录。
    pub central_mapping_tree: CodeTwoTreeVersion,
    /// 这一版 deadlist 树的条目，按 key 升序（D5（快照 / 空间记账机制） 已定项 12；判决 `research/prompts/m4-snapshot-r1-main-verification.md`
    /// 第三节 S3）：被活快照引用的块被杀时进这里，不释放；删快照的批放掉其中纪元落在 [lo, hi) 里、birth > P 的那几条。空 ⇒ 零根。
    pub deadlist_entries: Vec<DeadlistEntry>,
    /// 这一版 deadlist 树的形状与每个节点的指针，同记账树那一份；零根时一个节点都没有。
    pub deadlist_tree: CodeTwoTreeVersion,
    /// 被活快照钉住的单元留在中央映射树里的那几条映射条目（判决 S3「被钉单元的映射条目留在中央映射树里（α）」）：每条 deadlist 条目的
    /// 映射 key 一条，带它的两条位置项（释放判定路径经映射释放，D19（块指针的结构与宽度预算） 已定项 5 第 1 条：删快照的批从这里取落点）。
    /// 按映射 key 的字段序升序。它们不在 `mapped_units` 里（这一版的树不引用它们）。
    pub mapping_entries_kept_for_the_snapshots: Vec<(Vec<u8>, [LocationEntry; 2])>,
    /// 这一版树表里带「销毁中」记号的码 14 条目的树号（判决 S4：删快照的意图写在那条码 14 自己身上，flags 位 0），按树号升序。
    /// 第一版删快照串行（第七节岔路 2），至多一个。
    pub snapshots_being_destroyed: Vec<TreeIdentifier>,
    pub tree_table_entries: Vec<TreeTableEntry>,
    /// 这一版那十二棵树各自的号：第一个文件版本那次从水位发出来，之后每一版照抄（从盘上重建的版本按树表条目的种类与根记录里
    /// 中央映射树根指针的出生树读回来）。
    pub tree_identifiers: FileVersionTreeIdentifiers,
    /// 这一版第一个文件那条 inode 记录（覆盖写要接着它的对象出生代）。unlink 最后一批删掉 inode 1 之后的每一版是 `None`
    /// （D8（核心索引结构） 已定项 16 第 5 条：删完还剩别的 inode 的那一版仍是带文件的一版，只是没有第一个文件那条记录、没有数据单元、
    /// extent 树是零条目根）。
    pub inode_record: Option<InodeRecord>,
    /// 这一版 `deleted_inodes` 的号（D8（核心索引结构） 已定项 16：队列 = inode 树里 nlink = 0 的号集合，任何一版至多一条），升序。
    /// 空 ⇒ 树表条目的根指针置零、这一版没有 [`TransactionUnit::DeletedInodesRoot`]。
    pub deleted_inodes: Vec<InodeNumber>,
    /// 这一版 logged_ops 树的条目，按 key 升序（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1：整理意图，
    /// 全池活整理意图至多一条；判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1：截断意图，全池活截断意图至多一条）。
    /// 空 ⇒ 树表条目的根指针置零、这一版没有 [`TransactionUnit::LoggedOperationsRoot`]。
    /// 管理员回退时按种类分来源（判决 m4-trunc-r1 K1）：截断条目取 R_old 那一版的（承诺类意图跟着它要截的那一版 inode 与 extent 树走），
    /// 整理条目随现行那一版（未完成的整理意图跨回退留着、接着推）；`mount::roll_back_by_a_forward_publish` 交
    /// [`LoggedOperationsStepOfThePublish::TakeTheTruncationEntryOfTheRollbackTarget`]。
    pub logged_operations: Vec<LoggedOperationEntry>,
    /// 这一版墓碑树的叶容器，按叶序（= 容器号升序）：每片的内容与这一版它的指针（D18（块里携带什么信息） 已定项 24）。
    /// 空 ⇒ 墓碑树零根。
    pub tombstone_containers: Vec<TombstoneContainerVersion>,
    /// 这一版墓碑树的形状与根之下每个码 2 节点的指针（根满长高之后才有根之下的节点；根的指针住树表条目）。
    pub tombstone_tree_index_nodes: ContainerTreeIndexNodesVersion,
    /// 这一版谱系树的叶容器，按叶序（= 容器号升序）：每片的内容与这一版它的指针（D18（块里携带什么信息） 已定项 24 第 5 条）。
    /// 空 ⇒ 谱系树零根。随现行那一版、只增不删：管理员回退那次发布追加一条，回退之后也不取 R_old 的（D23（journal 的角色与格式） 已定项 14）。
    pub lineage_containers: Vec<LineageContainerVersion>,
    /// 这一版谱系树的形状与根之下每个码 2 节点的指针，同墓碑树那一份。
    pub lineage_tree_index_nodes: ContainerTreeIndexNodesVersion,
    /// 这一版退役树的叶容器，按叶序（= 容器号升序）：每片的内容与这一版它的指针（判决 `research/prompts/m3-retire-r1-main-verification.md`
    /// 第三节第 5 条）。空 ⇒ 退役树零根。随现行那一版、只增不删：身份消失的那次发布追加，管理员回退不取 R_old 的。
    pub retirement_containers: Vec<RetirementContainerVersion>,
    /// 这一版退役树的形状与根之下每个码 2 节点的指针，同墓碑树那一份。
    pub retirement_tree_index_nodes: ContainerTreeIndexNodesVersion,
    /// 这一版 inode 树里的全部叶容器，左起按 key 序：每片的身份、它装的记录、这一版它的指针
    /// （这次重写的是新落点，没重写的照抄上一版）。下一次发布按它算记录落在哪一片、要不要分裂。
    pub inode_leaf_containers: Vec<InodeLeafContainerVersion>,
    /// 进映射的单元各自的映射 key（每个数据单元、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树；映射树与树表豁免）；
    /// 下一次覆盖写按它经映射取落点释放（D19（块指针的结构与宽度预算） 已定项 5 第 1 条）。
    pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
    /// 这次发布释放的落点（覆盖写换下的上一版十二个单元；新池新建文件为空）：逻辑上释放了的都在，
    /// 其中有一份核出对不上、那块盘上的记录留在已分配的也在（那几份另列在下一个字段）。
    pub released: Vec<Placement>,
    /// 这次发布释放的落点里，释放之前读盘核校验和核出对不上（读不出也算）、隔离了的那几份（D19（块指针的结构与宽度预算） 已定项 5
    /// 硬规则 1「记进隔离并计数报出」的这一次那一份）：隔离 = 那一份在它那块盘上的分配记录留在已分配（用户 2026-09-24 定案），
    /// 落盘即跨重挂。从盘上重建的版本不是这个进程发布的，是空的。
    pub quarantined_after_release_checksum_mismatch:
        Vec<CopyQuarantinedAfterReleaseChecksumMismatch>,
    /// 这次发布搬走的单元（整理的一批，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3，[`publish_relocation_batch`]）：
    /// 映射 key、它属于这一版的哪个角色（或是被快照钉住、留存映射条目的那一种）、源与目的地的两条位置项。别的发布与从盘上重建的版本都是空的。
    pub units_relocated_by_this_publish: Vec<UnitRelocatedByThisPublish>,
    /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数：同一请求内取号序与 key 序不一致的次数，新池新建文件恒 0。
    pub key_order_mismatches: u64,
    /// 这次发布交给设备的写，按结构种类（增补 1）；从盘上重建的版本不是这个进程写出的，是空账。
    pub writes: WritesByStructureKind,
    /// 这个实例到这次发布为止用过的最大事务号（空发布写 0、不推进它）。下一次发布取它加一，
    /// 而不是取上一条记录上的事务号加一（D23（journal 的角色与格式） 已定项 7：事务号按实例计数、从 1 起）。
    pub highest_transaction_number_in_this_instance: u64,
    /// 墓碑树、退役树根单元在 `units` 里的下标（[`ContainerTreeRootUnitPositions`]）：准入读两个 H 时按它直接取根单元，不在 `units` 上逐项找。
    pub container_tree_root_unit_positions: ContainerTreeRootUnitPositions,
    /// 这一版 inode 树里的 n_live 与 n_记录（[`InodeRecordCounts`]）：每次发布在提交时增量维护，准入读它，不遍历 inode 树。
    pub inode_record_counts: InodeRecordCounts,
}

/// 一版里墓碑树、退役树根单元在 `units` 里的下标；那棵树零根（没有根单元）时 `None`。准入读两个 H（D3（空间分配） 已定项 2「各从自己的
/// 树根节点码 2 头现读」）按它直接取根单元的字节：它在运行时决策路径上（ENOSPC 准入，`.claude/rules/fs-design.md`「记账是事务的副产品，
/// 不是事后的遍历」），在 `units` 上逐项找是这一版的单元数（每片容器、每个码 2 节点各一项），随容器总数涨。装好一版的 `units` 时找一次记下
/// （发布、从盘上重建、管理员回退各一处），之后 `units` 不再挪动。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerTreeRootUnitPositions {
    pub tombstone_root: Option<usize>,
    pub retirement_root: Option<usize>,
}

impl ContainerTreeRootUnitPositions {
    /// 在装好的 `units` 里各找一次两个根单元（取最先那一项，与 [`TransactionOutput::unit`] 同一项）。
    #[must_use]
    pub fn of_the_units(units: &[PublishedUnit]) -> Self {
        let first_position_of =
            |identity: TransactionUnit| units.iter().position(|unit| unit.identity == identity);
        Self {
            tombstone_root: first_position_of(TransactionUnit::TombstoneRoot),
            retirement_root: first_position_of(TransactionUnit::RetirementRoot),
        }
    }
}

/// 第一个文件之外一个在 extent 树上段里有条目的文件在一版里的样子（[`TransactionOutput::files_other_than_the_first`]）。
/// 数据单元从 0 起连号、中间没有洞；尾部的洞由上段叶条目的标签声明（`tail`，标签 3、4），一个单元都没有的是标签 0（整个文件是洞，
/// size 0 也写这一条：判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2「粘住的标签 0」——变长到 size > 0 而零单元时
/// 写上，之后缩短、缩到 0、再变长都留着，到 unlink 最后一批才去掉）。`publish_new_inodes` 建出来、size 0、没改过 size 的 inode 不在这里。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileOfAnInodeOtherThanTheFirst {
    pub inode: InodeNumber,
    /// 第 i 项是这个文件第 i 个数据单元的指针（下段叶记录，或内联在上段叶条目里的那一个，按同一次序）；标签 0 时为空。
    pub data_pointers: Vec<DataPointer>,
    /// 这个文件 extent 树下段每个节点与它这一版的指针，bump 次序（先叶后根、同层按位置升序）；只有一个数据单元（内联）或一个都没有时为空。
    pub extent_lower_nodes: Vec<(ExtentLowerNodePosition, NodePointer)>,
    /// 上段叶条目标签声明的尾部：标签 1、2 没有洞，标签 3、4 最后一个单元之后到 size 是洞（判决 m4-trunc-r1 K5，推到任意活 inode：
    /// 判决 m4-multi-inode-size-r1 第二节 S2）。标签 0 时不读它。
    pub tail: TailOfAFile,
}

impl FileOfAnInodeOtherThanTheFirst {
    /// 这个文件第 `position` 个数据单元的角色。
    #[must_use]
    pub fn data_role_at(&self, position: usize) -> TransactionUnit {
        TransactionUnit::DataOfAFileOtherThanTheFirst {
            inode: self.inode,
            unit: DataUnitIndexInFile(u64::try_from(position).expect("单元序号装得进 u64")),
        }
    }

    /// 这个文件全部数据单元的角色，按单元序号。
    #[must_use]
    pub fn data_roles(&self) -> Vec<TransactionUnit> {
        (0..self.data_pointers.len())
            .map(|position| self.data_role_at(position))
            .collect()
    }

    /// 这个文件下段全部节点的角色，bump 次序。
    #[must_use]
    pub fn extent_lower_node_roles(&self) -> Vec<TransactionUnit> {
        self.extent_lower_nodes
            .iter()
            .map(
                |(position, _)| TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst {
                    inode: self.inode,
                    position: *position,
                },
            )
            .collect()
    }
}

/// 一个被搬走的单元归谁（[`UnitRelocatedByThisPublish`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocatedUnitHolder {
    /// 这一版的树引用着它：它是这个角色。搬迁不改角色，只改它的落点。
    RoleOfThisVersion(TransactionUnit),
    /// 只被活快照引用、映射条目留存在 `mapping_entries_kept_for_the_snapshots` 里（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
    /// 第三节 S3 的 α；判决 G3「被快照引用的块照样能搬」）：这一版的树不引用它，它没有角色。
    KeptForTheSnapshots,
}

/// 整理的一批搬走的一个单元：K 个单元原样字节写目的地（头不盖戳、birth 不变），映射条目换到目的地，源落点进 defer
/// （判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitRelocatedByThisPublish {
    pub mapping_key: Vec<u8>,
    pub holder: RelocatedUnitHolder,
    /// 搬之前映射条目里的两条位置项（源落点，两盘同槽）。
    pub source: [LocationEntry; 2],
    /// 搬之后映射条目里的两条位置项（目的落点，由这次发布的分配器按 D3（空间分配） 已定项 8 现取）。
    pub destination: [LocationEntry; 2],
}

/// 一版里每一处指向进映射单元的指针，换成那个单元在映射里的位置（判决 G3「写路径改成以映射为准」）：`position_of_key(映射 key)` 交回
/// 那个单元此刻的映射位置，交 `None` 的不动。指针按类型找 key：数据指针取码 1 key，inode 叶容器与三棵容器树的叶容器取码 3 key，
/// 其余节点指针（extent 树、分配记录树、记账树、deadlist 树、容器树根之下的节点、树表条目里各棵树的根）取码 2 key；零根不动。
/// 映射树自己的节点、树表、实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8），不在这里。`units` 里进映射角色的落点同步换成映射位置。
///
/// 内存里的这一版因此记的是映射位置：父节点下一次重写时写进它的就是映射位置（判决 G3「父节点重写时写映射位置」，C417 第一版政策
/// 「随重写刷新」），照抄的单元装映射条目时取的就是上一版的映射位置；盘上没重写的父节点里仍是旧提示。两处调它：
/// 整理的一批在装单元之前对上一版的拷贝调（目的地交给它），从盘上重建一版之后对重建出来的那一版调（映射条目交给它，
/// 判决 G3「重建时映射条目的位置覆盖提示」）。
pub fn take_the_mapping_positions_over_the_location_hints<PositionOfKey>(
    version: &mut TransactionOutput,
    position_of_key: PositionOfKey,
) where
    PositionOfKey: Fn(&[u8]) -> Option<[LocationEntry; 2]>,
{
    let refresh_node_pointer = |pointer: &mut NodePointer, unit_class: u8| {
        if *pointer == NodePointer::empty_root() {
            return;
        }
        if let Some(locations) = position_of_key(&mapping_key_for_node(unit_class, *pointer)) {
            pointer.locations = locations;
        }
    };
    for data_pointer in &mut version.data_pointers {
        if let Some(locations) = position_of_key(&mapping_key_for_data(
            data_pointer.head,
            data_pointer.write_order,
        )) {
            data_pointer.locations = locations;
        }
    }
    for (_, pointer) in &mut version.extent_tree.upper_nodes {
        refresh_node_pointer(pointer, UNIT_CLASS_INDEX_NODE);
    }
    for (_, pointer) in &mut version.extent_tree.lower_nodes {
        refresh_node_pointer(pointer, UNIT_CLASS_INDEX_NODE);
    }
    for file in &mut version.files_other_than_the_first {
        for data_pointer in &mut file.data_pointers {
            if let Some(locations) = position_of_key(&mapping_key_for_data(
                data_pointer.head,
                data_pointer.write_order,
            )) {
                data_pointer.locations = locations;
            }
        }
        for (_, pointer) in &mut file.extent_lower_nodes {
            refresh_node_pointer(pointer, UNIT_CLASS_INDEX_NODE);
        }
    }
    for container in &mut version.inode_leaf_containers {
        refresh_node_pointer(&mut container.pointer, UNIT_CLASS_PACKED);
    }
    for (_, pointer) in &mut version.allocation_record_tree.nodes {
        refresh_node_pointer(pointer, UNIT_CLASS_INDEX_NODE);
    }
    for pointer in version
        .accounting_tree
        .pointers
        .iter_mut()
        .chain(version.deadlist_tree.pointers.iter_mut())
    {
        refresh_node_pointer(pointer, UNIT_CLASS_INDEX_NODE);
    }
    for container in &mut version.tombstone_containers {
        refresh_node_pointer(&mut container.pointer, UNIT_CLASS_PACKED);
    }
    for container in &mut version.lineage_containers {
        refresh_node_pointer(&mut container.pointer, UNIT_CLASS_PACKED);
    }
    for container in &mut version.retirement_containers {
        refresh_node_pointer(&mut container.pointer, UNIT_CLASS_PACKED);
    }
    for (_, pointer) in version
        .tombstone_tree_index_nodes
        .pointers_below_the_root
        .iter_mut()
        .chain(
            version
                .lineage_tree_index_nodes
                .pointers_below_the_root
                .iter_mut(),
        )
        .chain(
            version
                .retirement_tree_index_nodes
                .pointers_below_the_root
                .iter_mut(),
        )
    {
        refresh_node_pointer(pointer, UNIT_CLASS_INDEX_NODE);
    }
    for entry in &mut version.tree_table_entries {
        refresh_node_pointer(&mut entry.root, UNIT_CLASS_INDEX_NODE);
    }
    // 进映射的角色按 `mapped_units` 的次序各问一次映射位置；有位置的那几个再按角色找 `units` 里的那一项（同一角色取最先那一项，
    // 与在 `units` 上 `iter_mut().find` 同一项）。找走建一次的索引（七·6 后续批，用户 2026-10-22 定并进）：整理发批一批搬 K 项、
    // 恢复重建一版每把 key 都有位置，逐个在 `units` 上找是 K 或 P 乘单元数。`position_of_key` 不读 `units`，先问完再改与边问边改一样。
    let new_slot_of_each_mapped_role: Vec<(TransactionUnit, SlotNumber)> = version
        .mapped_units
        .iter()
        .filter_map(|(identity, key)| {
            position_of_key(key).map(|locations| (*identity, locations[0].slot))
        })
        .collect();
    if new_slot_of_each_mapped_role.is_empty() {
        return;
    }
    let _inside_the_units_of_the_mapped_roles =
        lookup_work::entering(LookupSite::UnitsOfTheMappedRolesTakingTheirMappingPositions);
    let position_in_the_units_of_each_role =
        SortedIdentityIndex::keeping_the_first_value_of_each_identity(
            version
                .units
                .iter()
                .enumerate()
                .map(|(position, unit)| (unit.identity, position))
                .collect(),
        );
    // 迭代上界是有位置的进映射角色数。
    for (identity, slot) in new_slot_of_each_mapped_role {
        if let Some(position) =
            position_in_the_units_of_each_role.value_stored_for_identity(&identity)
        {
            version.units[*position].slot = slot;
        }
    }
}

/// 释放之前按位置项读盘核校验和、这个单元有一份核出对不上（读不出也算）而隔离的一份：哪个角色、哪块盘、那一份的落点
/// （槽取位置项里的，跨度取角色的）、那一份自己读出来的样子。隔离就是那块盘上这个落点的分配记录留在已分配；
/// 一个单元只要有一份对不上，每块盘上那一份都隔离（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-25 定：各盘的账保持对称）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CopyQuarantinedAfterReleaseChecksumMismatch {
    pub unit: TransactionUnit,
    pub device: DeviceIdentity,
    pub placement: Placement,
    pub reading: QuarantinedCopyReading,
}

/// 隔离的那一份自己读出来是什么样子（硬规则 1 的读盘核，D19（块指针的结构与宽度预算） 已定项 5）。计数报出时分得开
/// 「真坏的份数」与「为了两块盘的账对称一起留下的好份」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuarantinedCopyReading {
    /// 第一次没对上（读不出或对不上），重读那一次读出来了、整单元 CRC-32C 与位置项里的仍对不上。
    ChecksumMismatch,
    /// 第一次没对上（读不出或对不上），重读那一次读不出（只重读一次，用户 2026-09-24 定）。
    UnreadableAfterOneReread,
    /// 这一份对得上，同一个单元别的盘上那一份对不上或读不出：两块盘一起留在已分配（用户 2026-09-25 定）。
    IntactButAnotherCopyFailed,
}

/// 按 key 空间定形状的两棵派生树这一版的高（每个都从根节点头现读，[`TransactionOutput::position_addressed_tree_heights_read_from_the_root_node_headers`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionAddressedTreeHeights {
    pub allocation_record_tree: u64,
    pub extent_tree_upper_segment: u64,
    /// 第一个文件的下段；只有一个数据单元（内联）时 0。
    pub extent_tree_lower_segment_of_the_file: u64,
}

/// 一版里的一片 inode 叶容器：装了什么、这一版它的指针在哪。字节不放这里，放 `TransactionOutput::units`
/// 里那个角色的单元（重写的是新装的，照抄的是上一版那一份）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InodeLeafContainerVersion {
    pub contents: InodeLeafContainer,
    pub pointer: NodePointer,
}

/// 一版里的一片墓碑容器：装了什么、这一版它的指针。字节放 `TransactionOutput::units` 里那个角色的单元。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TombstoneContainerVersion {
    pub contents: TombstoneContainer,
    pub pointer: NodePointer,
}

/// 一版里的一片谱系容器：装了什么、这一版它的指针。字节放 `TransactionOutput::units` 里那个角色的单元。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineageContainerVersion {
    pub contents: LineageContainer,
    pub pointer: NodePointer,
}

/// 一版里的一片退役容器：装了什么、这一版它的指针。字节放 `TransactionOutput::units` 里那个角色的单元。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetirementContainerVersion {
    pub contents: RetirementContainer,
    pub pointer: NodePointer,
}

/// 一版里一棵容器树（墓碑树、谱系树）的码 2 节点：形状（每层每个节点几个孩子，`crate::container_tree::ContainerTreeShape`）与
/// 根之下每个节点的位置与指针（按 bump 次序：层级升序、同层从左到右）。根的指针住树表条目，不在这里；零根与只有根时这里一个都没有。
/// 节点的字节放 `TransactionOutput::units` 里那个角色的单元。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerTreeIndexNodesVersion {
    pub shape: ContainerTreeShape,
    pub pointers_below_the_root: Vec<(ContainerTreeNodePosition, NodePointer)>,
}

impl ContainerTreeIndexNodesVersion {
    /// 零根：一片容器、一个节点都没有。
    #[must_use]
    pub fn zero_root() -> Self {
        Self {
            shape: ContainerTreeShape::zero_root(),
            pointers_below_the_root: Vec::new(),
        }
    }

    /// 根之下 `position` 上那个节点这一版的指针；那个位置上没有根之下的节点交回 `None`。
    /// `pointers_below_the_root` 按 bump 次序列根之下的每个节点各一项（根最末、不在其中），所以这个位置那一项就在「这一层之前各层的节点数之和
    /// + 同层序号」那一格，按下标直接取：装一版的单元时每个照抄的节点都问一次（`build_deletion_tree_units`），在整张上逐项找合起来是节点数的平方。
    ///
    /// # Panics
    /// 那一格上记的位置不是问的这一个：这张与形状出自同一次发布（`publish_admitted` 按 `node_origins` 的 bump 次序装）或同一次从盘上重建
    /// （`recovery` 按 bump 次序读），都按 bump 次序列根之下的全部节点。
    #[must_use]
    pub fn pointer_below_the_root(
        &self,
        position: ContainerTreeNodePosition,
    ) -> Option<NodePointer> {
        if !self.shape.has_a_node_at(position) || self.shape.is_the_root(position) {
            return None;
        }
        let nodes_on_the_levels_below: usize = (1..position.level)
            .map(|level| self.shape.nodes_at_level(level))
            .sum();
        let index_in_bump_order = nodes_on_the_levels_below
            + usize::try_from(position.index_in_level).expect("同层序号装得进 usize");
        let (listed_position, pointer) = *self
            .pointers_below_the_root
            .get(index_in_bump_order)
            .expect(
                "根之下的每个节点在指针表里各一项、按 bump 次序：形状里有、又不是根的位置都在表里",
            );
        assert_eq!(
            listed_position, position,
            "根之下节点的指针表按 bump 次序排：第 {index_in_bump_order} 项就是这个位置"
        );
        Some(pointer)
    }
}

impl TransactionOutput {
    /// 这一版谱系树的内部层数：根节点码 2 头里的层级减一（零根与只有根时 0）。形状与根节点头里的层级出自同一次发布或同一次从盘上重建
    /// （重建按读到的每个节点头的层级排出形状，`recovery`）。
    #[must_use]
    pub fn lineage_tree_internal_levels(&self) -> u64 {
        self.lineage_tree_index_nodes.shape.internal_levels()
    }

    /// 这一版退役树的内部层数：根节点码 2 头里的层级减一（零根与只有根时 0）。形状的来路同谱系树那一份。
    #[must_use]
    pub fn retirement_tree_internal_levels(&self) -> u64 {
        self.retirement_tree_index_nodes.shape.internal_levels()
    }

    /// 这一版是不是「有树、文件集合为空」（D8（核心索引结构） 已定项 16 第 5 条）：树表有条目（`TransactionOutput` 恒有，树表 0 条的一版
    /// 是 `VersionWithoutFilePublishOutput`）∧ inode 树零条目根（一片叶容器都没有，key 区间 `[0, 0]`）∧ extent 树零条目根
    /// （上段一条叶条目都没有：这一版一个数据单元都没有）∧ `deleted_inodes` 根指针置零。不看 logged_ops、墓碑树、谱系树。
    #[must_use]
    pub fn is_a_version_with_trees_and_no_file(&self) -> bool {
        let inode_tree_root_has_no_entry = self.inode_leaf_containers.is_empty();
        let extent_tree_root_has_no_entry =
            self.data_pointers.is_empty() && self.files_other_than_the_first.is_empty();
        let deleted_inodes_root_pointer_is_zero = self
            .tree_root_pointer(self.tree_identifiers.deleted_inodes.0)
            == NodePointer::empty_root();
        inode_tree_root_has_no_entry
            && extent_tree_root_has_no_entry
            && deleted_inodes_root_pointer_is_zero
    }

    /// 这一版的承诺类意图都走完了没有（判决 m4-trunc-r1 K1：「队列空」各处收成这一个判断）：`deleted_inodes` 空、没有销毁中的快照、
    /// logged_ops 里没有截断条目。整理意图是提示类，不算（D8（核心索引结构） 已定项 5）。共用走空的次序见
    /// [`walk_the_deletion_queue_until_it_is_empty`]。
    #[must_use]
    pub fn queue_of_committed_intents_is_empty(&self) -> bool {
        self.deleted_inodes.is_empty()
            && self.snapshots_being_destroyed.is_empty()
            && live_truncation_intent_of_the_entries(&self.logged_operations).is_none()
    }

    /// 这一版 extent 树上段里有条目的 inode 号（上段规范形的 S，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md`
    /// 第三节 M2）：第一个文件有数据单元、或一个单元都没有而记录还在（上段叶标签 0，size 0 也算：
    /// [`ExtentUpperLeafTarget::of_a_file_without_data_units`]，判决 m4-validate-code-r2 T1 用户定「甲」）时连 inode 1，
    /// 加第一个文件之外每个在上段里有条目的文件（有数据单元的，与粘住的标签 0：判决 m4-multi-inode-size-r1 第二节 S2）。
    /// 与装上段叶写不写第一个文件那一条（`build_extent_tree`）同一条判法，形状与条目才对得上。
    #[must_use]
    pub fn inodes_with_an_extent_upper_leaf_entry(&self) -> BTreeSet<u64> {
        let first_file_has_an_entry = !self.data_pointers.is_empty()
            || ExtentUpperLeafTarget::of_a_file_without_data_units(
                self.inode_record.map(|record| record.size),
            )
            .is_some();
        let first_file = first_file_has_an_entry.then_some(FIRST_INODE_NUMBER);
        first_file
            .into_iter()
            .chain(
                self.files_other_than_the_first
                    .iter()
                    .map(|file| file.inode.0),
            )
            .collect()
    }

    /// 第一个文件已经删完的一版上它那条整对象墓碑的死亡 txg（K-U5 的 β 时序级读法，用户 2026-10-19 定）：inode 树里没有 inode 1 的记录、
    /// 这一版父链上有 inode 1 的墓碑时交回它的死亡 txg；inode 1 的记录还在（活着、删除中）或父链上没有它的墓碑（没删过，或删它的那一段被
    /// 回退抛下了）交回 `None`。依赖墓碑第一版不回收（D18（块里携带什么信息） 已定项 24，C618）。
    ///
    /// 只认这一版父链上的墓碑（代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 1 条）：墓碑随现行那一版、
    /// 管理员回退不取回退目标的，回退之后又删一次、再回退时墓碑树里 inode 1 有两条，其中一条落在被回退抛下的那一段里、不管这一版。
    /// 父链上的回退见 [`TransactionOutput::rollbacks_on_the_parent_chain`]；落在其中任何一段 (T_old, T_rb) 开区间里的死亡 txg 不算，再取最大。
    #[must_use]
    pub fn death_txg_of_the_deleted_first_file(&self) -> Option<CheckpointTxg> {
        if self.inode_record.is_some() {
            return None;
        }
        let rollbacks_on_the_parent_chain = self.rollbacks_on_the_parent_chain();
        self.tombstone_containers
            .iter()
            .flat_map(|container| container.contents.records.iter())
            .filter(|tombstone| tombstone.object.0 == FIRST_INODE_NUMBER)
            .map(|tombstone| tombstone.death_txg)
            .filter(|death_txg| {
                !rollbacks_on_the_parent_chain.iter().any(|rollback| {
                    rollback.rollback_target_txg < *death_txg && *death_txg < rollback.rollback_txg
                })
            })
            .max()
    }

    /// 这一版父链上的管理员回退（「在谱系上」的父链，I-9.18（删除队列里的号有谱系上的墓碑） 的定义，判决
    /// `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T3 取 a）：从这一版自己的 txg 起往回走，有谱系记录 (T_rb, T_old)
    /// 的那个 txg 的父是 T_old，其余 txg 的父是 txg − 1（C616（谱系父链靠 txg 连号没有断言））。记录按 T_rb 从大到小过一遍：T_rb 大于此刻走到的
    /// txg 的不在这条链上（写在被后来的回退抛下的那一段里），跳过；否则它在链上、走到它的 T_old。按 T_rb 从大到小交回链上的那几条。
    ///
    /// 谱系记录只由 `mount::roll_back_by_a_forward_publish` 追加（T_old 是回退目标的 txg，小于回退之前现行那一版的 txg，T_rb 是这次发布的 txg），
    /// 这一份进程写出的谱系 T_old < T_rb、一个 T_rb 一条；盘上读回的 T_old ≥ T_rb 或同一个 T_rb 两条的谱系接不成一条父链，这里不另判
    /// （池级 checker 判那一形，`crates/singlefs-checker/src/walk.rs` 的 `lineage_chain_of_a_version`），照同一个走法走完（每条记录至多过一次）。
    #[must_use]
    pub fn rollbacks_on_the_parent_chain(&self) -> Vec<LineageRecord> {
        let mut records_with_the_latest_rollback_first: Vec<LineageRecord> = self
            .lineage_containers
            .iter()
            .flat_map(|container| container.contents.records.iter().copied())
            .collect();
        records_with_the_latest_rollback_first
            .sort_unstable_by_key(|record| std::cmp::Reverse(record.rollback_txg));
        let mut rollbacks_on_the_chain = Vec::new();
        let mut txg_walked_to = self.root.checkpoint_txg;
        // 迭代上界是谱系记录条数；跨轮携带的是此刻走到的 txg，接上一条就走到它的 T_old。
        for record in records_with_the_latest_rollback_first {
            if record.rollback_txg > txg_walked_to {
                continue;
            }
            txg_walked_to = record.rollback_target_txg;
            rollbacks_on_the_chain.push(record);
        }
        rollbacks_on_the_chain
    }

    /// 这一版里第一个文件之外 inode `inode` 那个文件（在上段里有条目时：有数据单元，或粘住的标签 0）。
    #[must_use]
    pub fn file_other_than_the_first(
        &self,
        inode: InodeNumber,
    ) -> Option<&FileOfAnInodeOtherThanTheFirst> {
        self.files_other_than_the_first
            .iter()
            .find(|file| file.inode == inode)
    }

    /// 这一版 inode 树里号是 `inode` 的那条记录（删除中的也在）。
    #[must_use]
    pub fn inode_record_of(&self, inode: InodeNumber) -> Option<InodeRecord> {
        self.inode_leaf_containers
            .iter()
            .flat_map(|container| container.contents.records.iter())
            .find(|record| record.inode == inode.0)
            .copied()
    }

    /// 这一版 inode 树里号是 `inode` 的那条记录（删除中的也在），按叶序折半找：叶容器左起按 key 序、各片的记录按号升序、两片之间不交叠，
    /// 先找最后一条号不小于它的那一片，再在那一片里折半找。与 [`TransactionOutput::inode_record_of`] 交回同一条；发布路径改 n_live 与 n_记录时
    /// 每写一条找一次（运行时决策路径，不遍历 inode 树）。叶容器都非空（记录数为 0 的容器不落盘，删空的那次摘掉），按最后一条找不会落空。
    #[must_use]
    pub fn inode_record_found_by_its_leaf(&self, inode: InodeNumber) -> Option<InodeRecord> {
        let holding = self.inode_leaf_containers.partition_point(|container| {
            container
                .contents
                .records
                .last()
                .is_some_and(|last| last.inode < inode.0)
        });
        let records = &self.inode_leaf_containers.get(holding)?.contents.records;
        records
            .binary_search_by_key(&inode.0, |record| record.inode)
            .ok()
            .map(|position| records[position])
    }

    /// 这一版 inode 树里的叶容器内容，交给 `crate::inode_tree` 算下一次发布的落法。
    #[must_use]
    pub fn inode_leaf_container_contents(&self) -> Vec<InodeLeafContainer> {
        self.inode_leaf_containers
            .iter()
            .map(|container| container.contents.clone())
            .collect()
    }

    /// 记账里这一版的 inode 号水位（下一个可用号，D5（快照 / 空间记账机制） 已定项 4 第 12 项）。
    /// 读的是这一版真写进记账树的那一行，不重算——下一次发布接着它发号，I-9.6（水位大于两处最大号） 在盘上判它与树里最大 key 的关系。
    /// 这一统计量的行里还有台阶行（[`TransactionOutput::inode_number_watermark_steps`]），现行值取代段最大的那一行
    /// （D8（核心索引结构） 已定项 6「读现行值 = 取该维度元组下代号最大的那一行」）：每次发布写的那一行代段是这次的 txg，台阶行的代段都比它小。
    ///
    /// # Panics
    /// 这一版的记账行里没有 inode 号水位那一行。两条来路各自保证它在：这个进程发布出来的版本由 `publish_version` 写它
    /// （池级三行之一，每次发布都重写）；从盘上重建出来的版本由 `recovery::rebuild_version` 在交出这一版之前判过
    /// （盘上的统计量标签可以是任何值，那一判返回 `InodeNumberWatermarkRowMissingFromTheAccountingTree`）。
    #[must_use]
    pub fn inode_number_watermark(&self) -> u64 {
        self.accounting_entries
            .iter()
            .filter(|entry| entry.statistic == STATISTIC_INODE_WATERMARK)
            .max_by_key(|entry| entry.generation)
            .expect(
                "inode 号水位那一行在：发布路径每次都写它（D5 已定项 8 的池级三行之一），\
                 从盘上重建的版本在 rebuild_version 的边界判上验过（盘上的标签可以是任何值）",
            )
            .value
    }

    /// 这一版记账里 inode 号水位那一统计量的台阶行 (g, H)（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）：
    /// 水位变了的每次发布留一行，g 是那次发布的 txg（行的代段），H 是那次发布之后的水位（行的值）；台阶行与每次发布写的那一行同统计量、
    /// 同树段、同设备段（不带设备维的保留值），只差在代段——这一版那次发布本身水位变了时，它写的那一行就是这一次的台阶行。
    /// 认法（按行算、不另存标记）：代段小于这一版 txg 的行都是台阶行；代段等于这一版 txg 的那一行，值与代段最大的台阶行不同（或没有更早的台阶行）
    /// 才是台阶行。下一次发布照抄这几行（随根 COW、一直留着），管理员回退那次照抄 cur 的（`mount::roll_back_by_a_forward_publish`）。
    /// 按代段升序。
    // TODO(D5 已定项 2 射程「克隆头的台阶行按 (统计量, 树) 分，建克隆那次给新头留一行」)：`crates/` 今天没有克隆路径
    // （`mount::roll_back_by_a_forward_publish` 遇到树 ID 集合不同就拒），这里不按树段分；做克隆时按树段分开取、建克隆那次给新头留一行。
    #[must_use]
    pub fn inode_number_watermark_steps(&self) -> Vec<AccountingEntry> {
        let version_txg = self.root.checkpoint_txg;
        let mut steps: Vec<AccountingEntry> = self
            .accounting_entries
            .iter()
            .filter(|entry| {
                entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < version_txg
            })
            .copied()
            .collect();
        steps.sort_by_key(|entry| entry.generation);
        let row_of_this_version = self.accounting_entries.iter().find(|entry| {
            entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == version_txg
        });
        if let Some(row) = row_of_this_version {
            if steps
                .last()
                .is_none_or(|latest_step| latest_step.value != row.value)
            {
                steps.push(*row);
            }
        }
        steps
    }

    /// 这一版墓碑树根单元：按装这一版时记下的下标取（[`ContainerTreeRootUnitPositions`]），不在 `units` 上找；墓碑树零根时 `None`。
    ///
    /// # Panics
    /// 记下的下标上不是墓碑树根单元：`units` 装好之后才记下标，之后不再挪动（三处装 `units` 的地方都是装好再找）。
    #[must_use]
    pub fn tombstone_root_unit(&self) -> Option<&PublishedUnit> {
        self.unit_at_the_recorded_position(
            self.container_tree_root_unit_positions.tombstone_root,
            TransactionUnit::TombstoneRoot,
        )
    }

    /// 这一版退役树根单元，同 [`TransactionOutput::tombstone_root_unit`]；退役树零根时 `None`。
    ///
    /// # Panics
    /// 同 [`TransactionOutput::tombstone_root_unit`]。
    #[must_use]
    pub fn retirement_root_unit(&self) -> Option<&PublishedUnit> {
        self.unit_at_the_recorded_position(
            self.container_tree_root_unit_positions.retirement_root,
            TransactionUnit::RetirementRoot,
        )
    }

    /// 这一版墓碑树的内部层数 H：从它的根节点码 2 头现读，层级减一（D3（空间分配） 已定项 2「各从自己的树根节点码 2 头现读」）；零根 0。
    /// 根单元按装这一版时记下的下标取（[`TransactionOutput::tombstone_root_unit`]），不在 `units` 上找：准入读它（运行时决策路径）。
    ///
    /// # Panics
    /// 这一版有墓碑容器而没有记下墓碑树根单元，或那份字节解不开：单元是这个进程装出来的，或从盘上重建时读出、核过的
    /// （`recovery::rebuild_version` 把删除那两棵树的每个节点都放进 `units`）。
    #[must_use]
    pub fn tombstone_tree_internal_levels_read_from_its_root_node_header(&self) -> u64 {
        if self.tombstone_containers.is_empty() {
            return 0;
        }
        let root_unit = self
            .tombstone_root_unit()
            .expect("有墓碑容器就有墓碑树根单元：装 units 时记下了它的下标");
        let root_header = parse_index_node(&root_unit.bytes)
            .expect("墓碑树根单元是这个进程装的，或重建时读出、核过的");
        u64::from(root_header.level).saturating_sub(1)
    }

    /// 这一版退役树的内部层数 H，读法同 [`TransactionOutput::tombstone_tree_internal_levels_read_from_its_root_node_header`]；零根 0。
    ///
    /// # Panics
    /// 这一版有退役容器而没有记下退役树根单元，或那份字节解不开（同墓碑树那一份）。
    #[must_use]
    pub fn retirement_tree_internal_levels_read_from_its_root_node_header(&self) -> u64 {
        if self.retirement_containers.is_empty() {
            return 0;
        }
        let root_unit = self
            .retirement_root_unit()
            .expect("有退役容器就有退役树根单元：装 units 时记下了它的下标");
        let root_header = parse_index_node(&root_unit.bytes)
            .expect("退役树根单元是这个进程装的，或重建时读出、核过的");
        u64::from(root_header.level).saturating_sub(1)
    }

    fn unit_at_the_recorded_position(
        &self,
        recorded_position: Option<usize>,
        identity: TransactionUnit,
    ) -> Option<&PublishedUnit> {
        let position = recorded_position?;
        let unit = self
            .units
            .get(position)
            .expect("记下的根单元下标在 units 里：装好 units 之后才记，之后不再挪动");
        assert_eq!(
            unit.identity, identity,
            "units 第 {position} 项是装这一版时记下的那个根单元：装好之后才记，之后不再挪动"
        );
        Some(unit)
    }

    /// 这一版里角色是 `identity` 的那个单元（`units` 里最先那一项）。在 `units` 上线性找：一次调用里只查几次的用它；
    /// 要对约 P 个角色各查一次的（照抄全部单元、整理读全部映射条目）先建 [`TransactionOutput::units_by_role`]（七·6）。
    #[must_use]
    pub fn unit(&self, identity: TransactionUnit) -> &PublishedUnit {
        self.units
            .iter()
            .find(|unit| unit.identity == identity)
            .expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)
    }

    /// `units` 按角色的索引：建一次走完 `units`，之后每次按角色查不再走整张表（[`TransactionOutput::unit`] 的批量形态，七·6）。
    #[must_use]
    pub fn units_by_role(&self) -> FirstEntryOfEachIdentity<'_, TransactionUnit, PublishedUnit> {
        FirstEntryOfEachIdentity::of_the_table(&self.units, |unit| unit.identity)
    }

    /// `mapped_units` 按角色的索引（照抄进下一版的角色取上一版的映射 key，七·6）。
    #[must_use]
    pub fn mapped_units_by_role(
        &self,
    ) -> FirstEntryOfEachIdentity<'_, TransactionUnit, (TransactionUnit, Vec<u8>)> {
        FirstEntryOfEachIdentity::of_the_table(&self.mapped_units, |(identity, _)| *identity)
    }

    /// `mapped_units` 按映射 key 的索引（整理发批按计划里的映射 key 找它在这一版里的角色，七·6）：同一把 key 取最先那一项，
    /// 与在 `mapped_units` 上按 key `iter().find` 同一项。
    #[must_use]
    pub fn mapped_units_by_key(
        &self,
    ) -> FirstEntryOfEachIdentity<'_, &[u8], (TransactionUnit, Vec<u8>)> {
        FirstEntryOfEachIdentity::of_the_table(&self.mapped_units, |(_, key)| key.as_slice())
    }

    /// `mapping_entries_kept_for_the_snapshots` 按映射 key 的索引（删快照的批取放掉的条目留存的映射条目、整理发批找计划里留存的那一项，
    /// 七·6 代码轮第三轮判决第三节第 1 条）：同一把 key 取最先那一项，与在表上按 key `iter().find` 同一项。
    #[must_use]
    pub fn mapping_entries_kept_for_the_snapshots_by_key(
        &self,
    ) -> FirstEntryOfEachIdentity<'_, &[u8], (Vec<u8>, [LocationEntry; 2])> {
        FirstEntryOfEachIdentity::of_the_table(
            &self.mapping_entries_kept_for_the_snapshots,
            |(key, _)| key.as_slice(),
        )
    }

    /// `files_other_than_the_first` 按 inode 号的索引（[`TransactionOutput::file_other_than_the_first`] 的批量形态，七·6）。
    #[must_use]
    pub fn files_other_than_the_first_by_inode(
        &self,
    ) -> FirstEntryOfEachIdentity<'_, InodeNumber, FileOfAnInodeOtherThanTheFirst> {
        FirstEntryOfEachIdentity::of_the_table(&self.files_other_than_the_first, |file| file.inode)
    }

    /// 一棵多层码 2 树的高：从它这一版根节点的码 2 头里现读层级 + 1（D8（核心索引结构） 已定项 11 ⑤；
    /// D28（挂载期承诺量） 已定项 4：ckpt_cost 里每棵码 2 树的高从根节点头现读，不用内存里另存一份）。
    ///
    /// # Panics
    /// 这一版里这棵树的根单元解不开：它是这个进程装出来的，或从盘上重建时校验过的（`recovery::rebuild_version`）。
    #[must_use]
    pub fn height_read_from_the_root_node_header(&self, tree: MultiLevelCodeTwoTree) -> u64 {
        let root_role = match tree {
            MultiLevelCodeTwoTree::Accounting => TransactionUnit::AccountingTree,
            MultiLevelCodeTwoTree::CentralMapping => TransactionUnit::MappingTree,
            MultiLevelCodeTwoTree::Deadlist => TransactionUnit::DeadlistTree,
        };
        let root_header = parse_index_node(&self.unit(root_role).bytes)
            .expect("根单元是这个进程装的，或重建时解过、核过的");
        u64::from(root_header.level) + 1
    }

    /// 按 key 空间定形状的两棵派生树的高，同样从根节点的码 2 头里现读层级 + 1（D8（核心索引结构） 已定项 14 末句：
    /// 树高照已定项 11「树高 = 根节点头层级 + 1」读；D28（挂载期承诺量） 已定项 4 的 ckpt_cost 读它）：
    /// 分配记录树读它的根；extent 树读上段的根，与第一个文件下段的根（内联、没有下段时是 0）。
    ///
    /// # Panics
    /// 这一版里那个根单元解不开：它是这个进程装的，或从盘上重建时校验过的（`recovery::rebuild_version`）。
    #[must_use]
    pub fn position_addressed_tree_heights_read_from_the_root_node_headers(
        &self,
    ) -> PositionAddressedTreeHeights {
        let height_of = |root_role: TransactionUnit| {
            let root_header = parse_index_node(&self.unit(root_role).bytes)
                .expect("根单元是这个进程装的，或重建时解过、核过的");
            u64::from(root_header.level) + 1
        };
        PositionAddressedTreeHeights {
            allocation_record_tree: height_of(TransactionUnit::AllocationTree),
            extent_tree_upper_segment: height_of(TransactionUnit::ExtentRoot),
            extent_tree_lower_segment_of_the_file: self
                .extent_tree
                .lower_nodes
                .last()
                .map_or(0, |(position, _)| {
                    height_of(TransactionUnit::ExtentLowerNode(*position))
                }),
        }
    }

    /// 这一版一棵多层码 2 树（形状与指针）。
    #[must_use]
    pub fn multi_level_tree(&self, tree: MultiLevelCodeTwoTree) -> &CodeTwoTreeVersion {
        match tree {
            MultiLevelCodeTwoTree::Accounting => &self.accounting_tree,
            MultiLevelCodeTwoTree::CentralMapping => &self.central_mapping_tree,
            MultiLevelCodeTwoTree::Deadlist => &self.deadlist_tree,
        }
    }

    /// 活头 inode 树那条条目里的 `previous_snapshot_txg`（D5（快照 / 空间记账机制） 已定项 9；已定项 19 判定里的 `p`）：
    /// 还没打过快照是 0。
    ///
    /// # Panics
    /// 树表里没有码 2 条目：带文件的一版恒有活头 inode 树（发布路径装十二条，重建时 `tree_of(TREE_KIND_INODE)` 判过）。
    #[must_use]
    pub fn previous_snapshot_txg_of_the_live_head(&self) -> CheckpointTxg {
        self.tree_table_entries
            .iter()
            .find(|entry| entry.kind == TREE_KIND_INODE)
            .expect("带文件的一版恒有活头 inode 树那条条目")
            .previous_snapshot_txg
    }

    /// 树表里某棵树的根指针（照抄没重写的角色时用）。
    #[must_use]
    pub fn tree_root_pointer(&self, tree: u64) -> NodePointer {
        self.tree_table_entries
            .iter()
            .find(|entry| entry.tree == TreeIdentifier(tree))
            .expect("树表里每棵登记的树一条")
            .root
    }

    /// 这一版树表里的只读快照（码 14 与配对的码 15），按码 14 的树号升序（`tree_table_entries` 里照盘上原样有它们：发布路径续上的、
    /// 重建时读回来的）。
    ///
    /// # Panics
    /// 快照条目配不成对：发布路径照上一版原样续、建快照一次追加一对；从盘上重建的版本读树表时判过一一配对（`recovery` 读树表那一步）。
    #[must_use]
    pub fn snapshot_entries(&self) -> Vec<SnapshotTreeTableEntries> {
        snapshot_tree_table_entries_paired_one_to_one(&self.tree_table_entries).expect(
            "这一版的快照条目是发布路径成对写的、或从盘上读回时判过一一配对的（`tree_table_entries_each_kind_at_most_once`）",
        )
    }

    /// 树表条目的诞生 txg（十二条同一个数：树建起来那次发布）。
    #[must_use]
    pub fn tree_birth_txg(&self) -> CheckpointTxg {
        self.tree_table_entries
            .first()
            .expect("树表第 1 版起恒有条目")
            .birth_txg
    }

    /// 这一版全部角色的落点，按写者自己记的槽号（位置提示那一路）。释放不走它、走 `placements_to_release_via_mapping`；
    /// 留着给验收拿两条路互相对。
    #[must_use]
    pub fn placements(&self) -> Vec<Placement> {
        self.units
            .iter()
            .map(|unit| Placement {
                slot: unit.slot,
                span: unit.identity.span_slots(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
