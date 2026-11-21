//! 解出来的发布 `ResolvedPublish`：容器树的角色、各树的角色次序与多层树的角色。

use super::*;

/// 这次之后 logged_ops 根从哪来（同 [`DeletedInodesRootOrigin`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggedOperationsRootOrigin {
    /// 没有条目：树表条目的根指针置零，这一版没有这个角色。
    NoRoot,
    CarriedFromThePreviousVersion,
    RewrittenThisPublish,
}

/// 这次之后 `deleted_inodes` 根从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeletedInodesRootOrigin {
    /// 队列空：树表条目的根指针置零，这一版没有这个角色。
    NoRoot,
    CarriedFromThePreviousVersion,
    RewrittenThisPublish,
}

/// 一棵容器树（墓碑树、谱系树）的三种角色：叶容器按叶序、根、根之下的码 2 节点按位置。
#[derive(Clone, Copy)]
pub(super) struct ContainerTreeRoles {
    pub(super) container: fn(usize) -> TransactionUnit,
    pub(super) root: TransactionUnit,
    node_below_the_root: fn(ContainerTreeNodePosition) -> TransactionUnit,
}

fn tombstone_container_role(position: usize) -> TransactionUnit {
    TransactionUnit::TombstoneLeafContainer(TombstoneContainerIndexInTree::of_position(position))
}

fn lineage_container_role(position: usize) -> TransactionUnit {
    TransactionUnit::LineageLeafContainer(LineageContainerIndexInTree::of_position(position))
}

fn retirement_container_role(position: usize) -> TransactionUnit {
    TransactionUnit::RetirementLeafContainer(RetirementContainerIndexInTree::of_position(position))
}

impl ContainerTreeRoles {
    pub(super) const OF_THE_TOMBSTONE_TREE: ContainerTreeRoles = ContainerTreeRoles {
        container: tombstone_container_role,
        root: TransactionUnit::TombstoneRoot,
        node_below_the_root: TransactionUnit::TombstoneNodeBelowTheRoot,
    };
    pub(super) const OF_THE_LINEAGE_TREE: ContainerTreeRoles = ContainerTreeRoles {
        container: lineage_container_role,
        root: TransactionUnit::LineageRoot,
        node_below_the_root: TransactionUnit::LineageNodeBelowTheRoot,
    };
    pub(super) const OF_THE_RETIREMENT_TREE: ContainerTreeRoles = ContainerTreeRoles {
        container: retirement_container_role,
        root: TransactionUnit::RetirementRoot,
        node_below_the_root: TransactionUnit::RetirementNodeBelowTheRoot,
    };

    /// 根在 `root_of_that_version`（零根 `None`）的那一版里，`position` 上那个码 2 节点的角色：是那一版的根就是根，别的按位置。
    /// 根的位置由调用方从那一版的形状（[`ContainerTreeShape::root_position`]）或上一版记着的每层节点数
    /// （[`crate::container_tree::ContainerTreeNodeCounts::root_position`]）取一次。
    pub(super) fn of_node(
        self,
        position: ContainerTreeNodePosition,
        root_of_that_version: Option<ContainerTreeNodePosition>,
    ) -> TransactionUnit {
        if root_of_that_version == Some(position) {
            self.root
        } else {
            (self.node_below_the_root)(position)
        }
    }
}

/// 一棵容器树这次重写的角色，按 bump 次序（树内先叶后根）：重写的容器按叶序，再是重写的码 2 节点（层级升序、根最末）。
pub(super) fn container_tree_roles_rewritten<Record>(
    tree: &ContainerTreeAfterThisPublish<Record>,
    roles: ContainerTreeRoles,
) -> Vec<TransactionUnit> {
    let root = tree.shape.root_position();
    tree.rewritten_containers
        .iter()
        .map(|position| (roles.container)(*position))
        .chain(
            tree.rewritten_nodes()
                .into_iter()
                .map(|position| roles.of_node(position, root)),
        )
        .collect()
}

/// 一棵容器树这一版每个进映射的单元与它的映射 key，按 bump 次序（容器按叶序，再是码 2 节点、根最末）：重写的按出生序号现算
/// （`key_of_a_unit_born_in_this_publish` 收单元类与这棵树里的出生序号），照抄的取上一版那个角色的 key（长高那次旧根照抄，
/// 上一版里它是根）。
pub(super) fn mapped_units_of_a_container_tree<Record>(
    tree: &ContainerTreeAfterThisPublish<Record>,
    roles: ContainerTreeRoles,
    key_of_a_unit_born_in_this_publish: &dyn Fn(u8, usize) -> Vec<u8>,
    key_carried_from_the_previous_version: &dyn Fn(TransactionUnit) -> Vec<u8>,
) -> Vec<(TransactionUnit, Vec<u8>)> {
    let mut mapped_units = Vec::new();
    let mut birth_sequence = 0;
    // 重写的是最右端连着的那几片：序号不小于头一片就重写了，不在重写清单上逐个找（运行时决策路径，`.claude/rules/fs-design.md`
    // 「记账是事务的副产品，不是事后的遍历」：映射 key 集决定中央映射树重写哪几个节点，进分配）。
    let first_rewritten_container = tree.first_rewritten_container();
    for position in 0..tree.containers.len() {
        let identity = (roles.container)(position);
        if first_rewritten_container.is_some_and(|first| position >= first) {
            mapped_units.push((
                identity,
                key_of_a_unit_born_in_this_publish(UNIT_CLASS_PACKED, birth_sequence),
            ));
            birth_sequence += 1;
        } else {
            mapped_units.push((identity, key_carried_from_the_previous_version(identity)));
        }
    }
    let root = tree.shape.root_position();
    let root_before_this_publish = tree.node_counts_before_this_publish.root_position();
    for (position, origin) in tree.node_origins() {
        let identity = roles.of_node(position, root);
        match origin {
            ContainerTreeNodeOrigin::RewrittenThisPublish => {
                mapped_units.push((
                    identity,
                    key_of_a_unit_born_in_this_publish(UNIT_CLASS_INDEX_NODE, birth_sequence),
                ));
                birth_sequence += 1;
            }
            ContainerTreeNodeOrigin::CarriedFromThePreviousVersion => {
                mapped_units.push((
                    identity,
                    key_carried_from_the_previous_version(
                        roles.of_node(position, root_before_this_publish),
                    ),
                ));
            }
        }
    }
    mapped_units
}

/// 记账标签 6（已承诺预留里墓碑与退役记录那两份，D3（空间分配） 已定项 2）在这次发布之后那一版的值：
/// 副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)：H_墓碑、H_退役是这次之后墓碑树、退役树的内部层数，
/// n_live 是这次之后 inode 树里 nlink ≥ 1 的记录数，n_记录 是这次之后 inode 树里全部记录数（含 nlink = 0 的；判决
/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）：两个数读 [`ResolvedPublish::inode_record_counts`]（随一版带着、
/// 提交时增量维护，[`InodeRecordCounts`]），不遍历 inode 树；两个 H 按装 `units` 时记下的下标取根单元（[`ContainerTreeRootUnitPositions`]）。
/// `TransactionOutput` 恒有树表条目（`deleted_inodes` 那一条在内），所以走不到「没有 `deleted_inodes` 条目取 0」那一支——那一支是树表 0 条的一版，
/// 它不走这条发布路径、没有记账树。记账行写它，发布路径判式子读它（「写入类发布按发布之后的标签判」）：一个函数算，两处读同一个数。
/// 两个 H 照条款字面从各自根节点的码 2 头现读（[`tombstone_tree_internal_levels_read_from_the_root_node_header`]、
/// [`retirement_tree_internal_levels_read_from_the_root_node_header`]，代码轮 U8）。
#[must_use]
pub fn committed_reservation_for_tombstones_and_retirement_records_after_this_publish(
    resolved: &ResolvedPublish,
    previous: Option<&TransactionOutput>,
    replicas: ReplicaCount,
) -> BytesSummedOverAllReplicas {
    committed_reservation_for_tombstones_and_retirement_records(
        TombstoneAndRetirementPrepayBasis::VersionWithADeletedInodesTreeTableEntry {
            tombstone_tree_internal_levels:
                tombstone_tree_internal_levels_read_from_the_root_node_header(resolved, previous),
            live_objects: resolved.inode_record_counts.live_objects,
            retirement_tree_internal_levels:
                retirement_tree_internal_levels_read_from_the_root_node_header(resolved, previous),
            inode_records: resolved.inode_record_counts.records,
        },
        replicas,
    )
}

/// 这次发布之后退役树的内部层数（同 [`tombstone_tree_internal_levels_read_from_the_root_node_header`] 的读法）：这次重写根（有身份消失）时
/// 新根的头由同一份形状编出，按构造就是 `resolved.retirement_tree.internal_levels()`；这次照抄退役树时读上一版那份 `RetirementRoot` 单元字节的码 2 头
/// （[`TransactionOutput::retirement_tree_internal_levels_read_from_its_root_node_header`]：根单元按装 `units` 时记下的下标取，不在 `units` 上找）。
///
/// # Panics
/// 上一版有退役容器而它的单元里没有 `RetirementRoot`，或那份字节解不开：单元是这个进程装出来的，或从盘上重建时读出、核过的。
#[must_use]
pub fn retirement_tree_internal_levels_read_from_the_root_node_header(
    resolved: &ResolvedPublish,
    previous: Option<&TransactionOutput>,
) -> u64 {
    if resolved.retirement_tree.is_the_root_rewritten() {
        return resolved.retirement_tree.internal_levels();
    }
    // 上一版退役树零根（或没有上一版），这次又没重写它：这次之后仍是零根，读出来是 0。
    previous.map_or(
        0,
        TransactionOutput::retirement_tree_internal_levels_read_from_its_root_node_header,
    )
}

/// 这次发布之后墓碑树的内部层数 H（D3（空间分配） 已定项 2「H 从墓碑树根节点码 2 头现读」，代码轮 U8，
/// `research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：根节点头的层级减一（零根时 0）。
/// 这次重写根（意图那次往开放容器追加墓碑）：新根的头由同一份形状编出（`build_index_node` 的层级取根在形状里的层级），按构造就是
/// `resolved.tombstone_tree.internal_levels()`，不另读；这次照抄墓碑树（写入类发布、各批）：读上一版那份 `TombstoneRoot` 单元字节的码 2 头
/// （[`TransactionOutput::tombstone_tree_internal_levels_read_from_its_root_node_header`]：根单元按装 `units` 时记下的下标取，不在 `units` 上找），
/// 照 [`TransactionOutput::height_read_from_the_root_node_header`] 的先例，不读内存里那份形状。
///
/// # Panics
/// 上一版有墓碑容器而它的单元里没有 `TombstoneRoot`，或那份字节解不开：单元是这个进程装出来的，或从盘上重建时读出、核过的
/// （`recovery::rebuild_version` 把删除那两棵树的每个节点都放进 `units`）。
#[must_use]
pub fn tombstone_tree_internal_levels_read_from_the_root_node_header(
    resolved: &ResolvedPublish,
    previous: Option<&TransactionOutput>,
) -> u64 {
    if resolved.tombstone_tree.is_the_root_rewritten() {
        return resolved.tombstone_tree.internal_levels();
    }
    // 上一版墓碑树零根（没有容器就没有根，或没有上一版），这次又没重写它：这次之后仍是零根，读出来是 0。
    previous.map_or(
        0,
        TransactionOutput::tombstone_tree_internal_levels_read_from_its_root_node_header,
    )
}

/// 一次发布在动分配器、动盘之前先算完的东西：这次之后 inode 树是哪几片叶容器、这次重写哪些角色（按 bump 次序）。
/// 只读——算它不发一个写、不动分配器，所以条款没写的那几格（`crate::inode_tree`）在这里交回时盘上逐字节不变。
#[derive(Clone, Debug)]
pub struct ResolvedPublish {
    pub inode_tree: InodeLeafContainersAfterThisPublish,
    /// 这次之后 inode 树里的 n_live 与 n_记录：上一版的按这次写进、删掉的那几条改出来（[`InodeRecordCounts::after_writing`]、
    /// [`InodeRecordCounts::after_removing`]），准入读它，不遍历这次之后的叶容器。
    pub inode_record_counts: InodeRecordCounts,
    /// 这次之后 extent 树长什么样（D8（核心索引结构） 已定项 14 的两段）。
    pub extent_tree: ExtentTreePlan,
    /// 这次之后第一个文件之外有数据单元的文件长什么样（数据单元与下段；它们在上段里的条目归 `extent_tree` 的上段）。
    pub files_other_than_the_first: FilesOtherThanTheFirstPlan,
    /// 这次之后分配记录树长什么样（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）。重写哪几个节点由发布路径在分配器的拷贝上
    /// 走到固定点定下来（`settle_the_allocation_record_tree`），交给 `PublishPlan::resolve` 的就是那一份。
    pub allocation_record_tree: AllocationRecordTreePlan,
    /// 这次之后记账树长什么样（D8（核心索引结构） 已定项 11 的分裂与收缩）。
    pub accounting_tree: CodeTwoTreePlan,
    /// 这次之后中央映射树长什么样，同上；按下面那一份映射 key 与留存的映射条目（`deadlist.kept_mapping_entries_after_this_publish`）算。
    pub central_mapping_tree: CodeTwoTreePlan,
    /// 这次之后 deadlist 树长什么样（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：按 `deadlist.entries_after_this_publish` 算，
    /// 一条都不剩时零根。
    pub deadlist_tree: CodeTwoTreePlan,
    /// 这次发布对 deadlist 与留存映射条目做的事。
    pub deadlist: DeadlistChangesOfThisPublish,
    /// 这一版每个进映射的单元与它的映射 key，按 bump 次序（每个数据单元、extent 根、每片 inode 叶容器、inode 根、分配记录树、
    /// 记账树的每个节点）：这次重写的按这次要发的出生身份现算（出生序号按 bump 次序发，D19（块指针的结构与宽度预算） 已定项 9），
    /// 照抄的取上一版的。装映射节点那一段（`publish_admitted`）核装出来的与它逐项相等。
    pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
    pub rewritten_roles: Vec<TransactionUnit>,
    /// 不写文件内容的发布照抄上一版第一个文件的前几个数据单元（删除的一批留下的、最后一批 0 个）。
    pub carried_data_units: usize,
    /// 这次之后 `deleted_inodes` 的号（升序）与它的根从哪来。
    pub deleted_inodes: Vec<InodeNumber>,
    pub deleted_inodes_root: DeletedInodesRootOrigin,
    /// 这次之后墓碑树的样子。
    pub tombstone_tree: TombstoneTreeAfterThisPublish,
    /// 这次之后谱系树的样子（管理员回退那次追加一条，别的照抄）。
    pub lineage_tree: LineageTreeAfterThisPublish,
    /// 这次之后退役树的样子（有 inode 叶容器身份消失的那次追加，别的照抄）。
    pub retirement_tree: RetirementTreeAfterThisPublish,
    /// 这次之后 logged_ops 的条目（按 key 升序）与它的根从哪来。
    pub logged_operations: Vec<LoggedOperationEntry>,
    pub logged_operations_root: LoggedOperationsRootOrigin,
    /// 上一版里有、这一版没有同一个单元接着的角色（删除批截掉的尾部数据单元、身份消失的 inode 叶容器——按它在上一版里的叶序、
    /// 删空的 `deleted_inodes` 根），照样被这次换下。
    pub previous_roles_without_a_successor: Vec<TransactionUnit>,
    /// 这次按读回的旧单元（或零）重写的那一段数据单元与它属于哪个 inode（截断意图、变长的那一个尾单元，判决 m4-trunc-r1 K3、K5；
    /// 任意活 inode，判决 m4-multi-inode-size-r1 第二节 S3；按偏移部分写的那一段，判决 m4-partial-write-r1 第二节 R4）；别的发布 `None`。
    pub data_units_rewritten_from_their_payloads: Option<DataUnitsRewrittenByTheStep>,
}

impl ResolvedPublish {
    /// 一棵多层码 2 树这次之后的计划。
    #[must_use]
    pub fn multi_level_tree(&self, tree: MultiLevelCodeTwoTree) -> &CodeTwoTreePlan {
        match tree {
            MultiLevelCodeTwoTree::Accounting => &self.accounting_tree,
            MultiLevelCodeTwoTree::CentralMapping => &self.central_mapping_tree,
            MultiLevelCodeTwoTree::Deadlist => &self.deadlist_tree,
        }
    }

    /// 这次发布换下的上一版角色：这次重写的角色里不属于那四棵多节点树（extent 树、分配记录树、记账树、中央映射树）的那些
    /// （同一个角色上一版那一份被换下），加 extent 树与分配记录树按计划换下的节点、两棵多层码 2 树里上一版被换下的节点
    /// （都按上一版的位置点名；这一版照抄进来的节点不被换下）。
    /// 释放判定路径与释放之前读盘核校验和读的都是这一张（`placements_to_release_via_mapping`、`copies_failing_the_release_checksum_check`）。
    #[must_use]
    pub fn previous_roles_replaced_by_this_publish(
        &self,
        previous: &TransactionOutput,
    ) -> Vec<TransactionUnit> {
        // 按 bump 次序：数据单元与实例表、extent 树换下的节点、inode 树、分配记录树换下的节点、记账树换下的节点、
        // 中央映射树换下的节点、树表最末——与这次重写的角色同一个次序，释放清单与读盘核校验和的次序都照它。
        let is_a_node_of_a_tree_with_its_own_replacement_list = |identity: TransactionUnit| {
            multi_level_tree_of_role(identity).is_some()
                || allocation_record_tree_node_of_role(identity).is_some()
                || matches!(
                    identity,
                    TransactionUnit::ExtentLowerNode(_)
                        | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
                        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
                        | TransactionUnit::ExtentRoot
                )
        };
        let rewritten_before_the_extent_tree =
            self.rewritten_roles.iter().copied().filter(|identity| {
                matches!(
                    identity,
                    TransactionUnit::Data(_)
                        | TransactionUnit::InstanceTable
                        | TransactionUnit::InstanceTablePageAfterTheFirst(_)
                )
            });
        let mut replaced: Vec<TransactionUnit> = rewritten_before_the_extent_tree.collect();
        // 没有同名后继的上一版角色（删除批截掉的尾部数据单元、删空的 inode 叶容器、删空的 `deleted_inodes` 根）按它们在 bump 次序里的位置插：
        // 数据单元排最前，叶容器跟在 inode 树那几个后面，`deleted_inodes` 根跟在记账树后面——下面逐段并进。
        replaced.extend(
            self.previous_roles_without_a_successor
                .iter()
                .copied()
                .filter(|identity| {
                    matches!(
                        identity,
                        TransactionUnit::Data(_)
                            | TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
                    )
                }),
        );
        // 写第一个文件之外的文件：它上一版的数据单元整份换下（数据单元那一段），下段节点排在第一个文件的下段之后、上段之前。
        replaced.extend(
            self.files_other_than_the_first
                .replaced_previous_data_roles
                .iter()
                .copied(),
        );
        replaced.extend(
            self.files_other_than_the_first
                .replaced_previous_lower_node_roles
                .iter()
                .copied(),
        );
        replaced.extend(
            self.previous_roles_without_a_successor
                .iter()
                .copied()
                .filter(|identity| {
                    matches!(
                        identity,
                        TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
                    )
                }),
        );
        replaced.extend(self.extent_tree.replaced_previous_roles.iter().copied());
        // 容器树长高那次：新根在上一版里没有落点，上一版的根不重写、留在原位置当根之下的节点——它不被这次换下
        // （一次追加几条、旧根下面也有容器重写时旧根跟着重写，照样换下：`previous_root_is_carried_down_unchanged` 只认前一种）。
        let is_a_new_root_over_a_root_carried_down = |identity: TransactionUnit| {
            (identity == TransactionUnit::TombstoneRoot
                && self
                    .tombstone_tree
                    .previous_root_is_carried_down_unchanged())
                || (identity == TransactionUnit::LineageRoot
                    && self.lineage_tree.previous_root_is_carried_down_unchanged())
                || (identity == TransactionUnit::RetirementRoot
                    && self
                        .retirement_tree
                        .previous_root_is_carried_down_unchanged())
        };
        replaced.extend(self.rewritten_roles.iter().copied().filter(|identity| {
            !is_a_node_of_a_tree_with_its_own_replacement_list(*identity)
                && !is_a_new_root_over_a_root_carried_down(*identity)
                && !matches!(
                    identity,
                    TransactionUnit::Data(_)
                        | TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
                        | TransactionUnit::InstanceTable
                        | TransactionUnit::InstanceTablePageAfterTheFirst(_)
                        | TransactionUnit::TreeTable
                )
        }));
        replaced.extend(
            self.previous_roles_without_a_successor
                .iter()
                .copied()
                .filter(|identity| {
                    !matches!(
                        identity,
                        TransactionUnit::Data(_)
                            | TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
                            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
                    )
                }),
        );
        replaced.extend(
            self.allocation_record_tree
                .replaced_previous_nodes
                .iter()
                .map(|node| role_of_allocation_record_tree_node(*node)),
        );
        for tree in [
            MultiLevelCodeTwoTree::Accounting,
            MultiLevelCodeTwoTree::Deadlist,
            MultiLevelCodeTwoTree::CentralMapping,
        ] {
            let previous_shape = &previous.multi_level_tree(tree).shape;
            replaced.extend(
                self.multi_level_tree(tree)
                    .replaced_previous_nodes
                    .iter()
                    .map(|position| tree.role_of_node(*position, previous_shape)),
            );
        }
        if self.rewritten_roles.contains(&TransactionUnit::TreeTable) {
            replaced.push(TransactionUnit::TreeTable);
        }
        replaced
    }
}

/// 一个角色属于哪棵多层码 2 树（记账树、中央映射树的根与根之下的节点）；别的角色交回 `None`。
#[must_use]
pub const fn multi_level_tree_of_role(identity: TransactionUnit) -> Option<MultiLevelCodeTwoTree> {
    match identity {
        TransactionUnit::AccountingTreeNodeBelowTheRoot(_) | TransactionUnit::AccountingTree => {
            Some(MultiLevelCodeTwoTree::Accounting)
        }
        TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
            Some(MultiLevelCodeTwoTree::CentralMapping)
        }
        TransactionUnit::DeadlistTreeNodeBelowTheRoot(_) | TransactionUnit::DeadlistTree => {
            Some(MultiLevelCodeTwoTree::Deadlist)
        }
        TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot
        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
        | TransactionUnit::AllocationTree
        | TransactionUnit::DeletedInodesRoot
        | TransactionUnit::LoggedOperationsRoot
        | TransactionUnit::TombstoneLeafContainer(_)
        | TransactionUnit::TombstoneRoot
        | TransactionUnit::TombstoneNodeBelowTheRoot(_)
        | TransactionUnit::LineageLeafContainer(_)
        | TransactionUnit::LineageRoot
        | TransactionUnit::LineageNodeBelowTheRoot(_)
        | TransactionUnit::RetirementLeafContainer(_)
        | TransactionUnit::RetirementRoot
        | TransactionUnit::RetirementNodeBelowTheRoot(_)
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable
        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => None,
    }
}

/// 分配记录树的一个节点在一版里的角色：根是 `AllocationTree`，根之下按位置（位置不随版本挪动）。
#[must_use]
pub const fn role_of_allocation_record_tree_node(
    node: AllocationRecordTreeNode,
) -> TransactionUnit {
    match node {
        AllocationRecordTreeNode::Root => TransactionUnit::AllocationTree,
        AllocationRecordTreeNode::BelowTheRoot(position) => {
            TransactionUnit::AllocationTreeNodeBelowTheRoot(position)
        }
    }
}

/// 一个角色是不是分配记录树的节点、是哪一个；别的角色交回 `None`。
#[must_use]
pub const fn allocation_record_tree_node_of_role(
    identity: TransactionUnit,
) -> Option<AllocationRecordTreeNode> {
    match identity {
        TransactionUnit::AllocationTree => Some(AllocationRecordTreeNode::Root),
        TransactionUnit::AllocationTreeNodeBelowTheRoot(position) => {
            Some(AllocationRecordTreeNode::BelowTheRoot(position))
        }
        TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
        | TransactionUnit::Data(_)
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::ExtentRoot
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::InodeRoot
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
        | TransactionUnit::DeadlistTree
        | TransactionUnit::DeletedInodesRoot
        | TransactionUnit::LoggedOperationsRoot
        | TransactionUnit::TombstoneLeafContainer(_)
        | TransactionUnit::TombstoneRoot
        | TransactionUnit::TombstoneNodeBelowTheRoot(_)
        | TransactionUnit::LineageLeafContainer(_)
        | TransactionUnit::LineageRoot
        | TransactionUnit::LineageNodeBelowTheRoot(_)
        | TransactionUnit::RetirementLeafContainer(_)
        | TransactionUnit::RetirementRoot
        | TransactionUnit::RetirementNodeBelowTheRoot(_)
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::TreeTable
        | TransactionUnit::InstanceTable
        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => None,
    }
}

/// extent 树上段一个位置在一版里的角色：这一版上段的根（最高那一层 `upper_root_level` 的第 0 个）是 `ExtentRoot`，别的按位置。
#[must_use]
pub fn role_of_extent_upper_node(
    position: ExtentUpperNodePosition,
    upper_root_level: u8,
) -> TransactionUnit {
    if position.level == upper_root_level {
        TransactionUnit::ExtentRoot
    } else {
        TransactionUnit::ExtentUpperNodeBelowTheRoot(position)
    }
}

/// 一版 extent 树里全部节点的角色，按 bump 次序（下段先叶后根，再上段先叶后根、上段根最末，D3（空间分配） 已定项 10 ⑤「树内先叶后根」：
/// 上段叶条目要下段根这一版的指针，下段排在前面）。
#[must_use]
pub fn extent_tree_roles_in_bump_order(version: &ExtentTreeVersion) -> Vec<TransactionUnit> {
    let lower = version
        .lower_nodes
        .iter()
        .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position));
    let upper_root_level = version.upper_root().level;
    let upper = version
        .upper_nodes
        .iter()
        .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level));
    lower.chain(upper).collect()
}
