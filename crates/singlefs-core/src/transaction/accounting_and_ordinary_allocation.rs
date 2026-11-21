//! 这次发布的记账项与普通分配那一部分（`OrdinaryAllocationPartOfThePublish`）、这次发布的几类树。

use super::*;

/// 记账树里不带设备维的三行（D5（快照 / 空间记账机制） 已定项 8）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PoolWideAccountingRow {
    InodeNumberWatermark,
    PendingDeleteBytes,
    CommittedReservationBytes,
}

impl PoolWideAccountingRow {
    const ALL: [Self; 3] = [
        PoolWideAccountingRow::InodeNumberWatermark,
        PoolWideAccountingRow::PendingDeleteBytes,
        PoolWideAccountingRow::CommittedReservationBytes,
    ];

    pub(super) const fn statistic(self) -> u16 {
        match self {
            PoolWideAccountingRow::InodeNumberWatermark => STATISTIC_INODE_WATERMARK,
            PoolWideAccountingRow::PendingDeleteBytes => STATISTIC_PENDING_DELETE_BYTES,
            PoolWideAccountingRow::CommittedReservationBytes => {
                STATISTIC_COMMITTED_RESERVATION_BYTES
            }
        }
    }

    /// 这一行记在哪棵树上：inode 号水位记在 inode 树上，另两行不属于任何一棵树。
    pub(super) const fn tree(self, inode_tree: TreeIdentifier) -> TreeIdentifier {
        match self {
            PoolWideAccountingRow::InodeNumberWatermark => inode_tree,
            PoolWideAccountingRow::PendingDeleteBytes
            | PoolWideAccountingRow::CommittedReservationBytes => {
                TreeIdentifier(TREE_IDENTIFIER_NONE)
            }
        }
    }
}

/// 记账树里每块盘各一行的六项（D5（快照 / 空间记账机制） 已定项 8）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PerDeviceAccountingRow {
    AllocatedBytes,
    FreeBytes,
    UnreclaimableBytes,
    DeferQueueBytes,
    FragmentationRuns,
    EmptyClusterSegments,
}

impl PerDeviceAccountingRow {
    const ALL: [Self; 6] = [
        PerDeviceAccountingRow::AllocatedBytes,
        PerDeviceAccountingRow::FreeBytes,
        PerDeviceAccountingRow::UnreclaimableBytes,
        PerDeviceAccountingRow::DeferQueueBytes,
        PerDeviceAccountingRow::FragmentationRuns,
        PerDeviceAccountingRow::EmptyClusterSegments,
    ];

    pub(super) const fn statistic(self) -> u16 {
        match self {
            PerDeviceAccountingRow::AllocatedBytes => STATISTIC_ALLOCATED_BYTES,
            PerDeviceAccountingRow::FreeBytes => STATISTIC_FREE_BYTES,
            PerDeviceAccountingRow::UnreclaimableBytes => STATISTIC_UNRECLAIMABLE_BYTES,
            PerDeviceAccountingRow::DeferQueueBytes => STATISTIC_DEFER_QUEUE_BYTES,
            PerDeviceAccountingRow::FragmentationRuns => STATISTIC_FRAGMENTATION_RUNS,
            PerDeviceAccountingRow::EmptyClusterSegments => STATISTIC_EMPTY_CLUSTER_SEGMENTS,
        }
    }
}

/// 一次发布写的记账行（池级三行加每块盘六行，代 = 这次的 txg、seq 一律 1，D8（核心索引结构） 已定项 10），
/// 加上从上一版照抄的 inode 号水位台阶行（[`TransactionOutput::inode_number_watermark_steps`]，代段都小于这次的 txg，原样照抄），按 key 升序。
/// 准入之前算记账树形状（只要 key，值给 0）与装记账树（值从分配器取）读的是这一份行表：台阶行只看上一版，不看这次的水位，
/// 所以两处的 key 集合相同。这次的水位与照抄来的最后一级台阶不同时，这次写的那一行就是这次的台阶行（下一次发布照抄它）。
pub(super) fn accounting_entries_of_this_publish(
    txg: CheckpointTxg,
    inode_tree: TreeIdentifier,
    device_identities: &[DeviceIdentity],
    inode_number_watermark_steps_carried_from_the_previous_version: &[AccountingEntry],
    pool_wide_value: &dyn Fn(PoolWideAccountingRow) -> u64,
    per_device_value: &dyn Fn(PerDeviceAccountingRow, DeviceIdentity) -> u64,
) -> Vec<AccountingEntry> {
    assert!(
        inode_number_watermark_steps_carried_from_the_previous_version
            .iter()
            .all(|step| step.statistic == STATISTIC_INODE_WATERMARK && step.generation < txg),
        "照抄来的台阶行是上一版 inode 号水位那一统计量的行，代段都早于这次的 txg {txg:?}（上一版的 txg 小于这次的）"
    );
    let mut entries: Vec<AccountingEntry> = PoolWideAccountingRow::ALL
        .into_iter()
        .map(|row| AccountingEntry {
            statistic: row.statistic(),
            tree: row.tree(inode_tree),
            device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),
            generation: txg,
            value: pool_wide_value(row),
            sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
        })
        .collect();
    for device in device_identities {
        for row in PerDeviceAccountingRow::ALL {
            entries.push(AccountingEntry {
                statistic: row.statistic(),
                tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                device: *device,
                generation: txg,
                value: per_device_value(row, *device),
                sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
            });
        }
    }
    entries.extend_from_slice(inode_number_watermark_steps_carried_from_the_previous_version);
    entries.sort_by_key(AccountingEntry::sort_key);
    entries
}

/// 一个这次发布里新写出的码 2 / 码 3 单元的映射 key（D19（块指针的结构与宽度预算） 已定项 6）：类标签、出生 (树, txg)、实例代号、出生序号。
/// key 不看落点：这次要写哪几个单元、各自的出生序号在取落点之前就定了（出生序号按 bump 次序发，D19（块指针的结构与宽度预算） 已定项 9）。
pub(super) fn mapping_key_of_a_node_born_in_this_publish(
    unit_class: u8,
    tree: TreeIdentifier,
    txg: CheckpointTxg,
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    mapping_key_for_node(
        unit_class,
        NodePointer {
            head: PointerHead {
                birth_tree: tree,
                birth_txg: txg,
            },
            locations: NodePointer::empty_root().locations,
            instance,
            birth_sequence,
        },
    )
}

/// 按 key 空间定形状的两棵树这次之后的样子（`resolve` 给映射排 key 要它们的节点清单）。
pub(super) struct MappedPositionAddressedTrees<'plans> {
    pub(super) extent: &'plans ExtentTreePlan,
    /// 第一个文件之外各文件的数据单元与下段（它们的映射 key 排在第一个文件那几把之后）。
    pub(super) files_other_than_the_first: &'plans FilesOtherThanTheFirstPlan,
    pub(super) allocation_records: &'plans AllocationRecordTreePlan,
}

/// 这次之后删除那两棵树、谱系树与照抄几个数据单元（`resolve` 给映射排 key 要它们）。
pub(super) struct DeletionTreesOfThisPublish<'plans> {
    pub(super) carried_data_units: usize,
    /// 这一步按读回的旧单元（或零）重写的那一段数据单元切成的一单元事务（截断意图与变长的那一个尾单元，判决 m4-trunc-r1 K3、K5；部分写的那一段，
    /// 判决 m4-partial-write-r1 第二节 R4）与它们属于哪个 inode：它们的映射 key 按这次的出生身份与各自的事务号现算，那个文件别的数据单元照抄。
    pub(super) data_units_rewritten_from_their_payloads:
        Option<(InodeNumber, Vec<OneUnitTransaction>)>,
    pub(super) deleted_inodes_root: DeletedInodesRootOrigin,
    pub(super) tombstone_tree: &'plans TombstoneTreeAfterThisPublish,
    pub(super) lineage_tree: &'plans LineageTreeAfterThisPublish,
    pub(super) logged_operations_root: LoggedOperationsRootOrigin,
    pub(super) retirement_tree: &'plans RetirementTreeAfterThisPublish,
}

/// 这张计划要写的记账树与中央映射树，在一切落盘动作之前算完：记账树按这一版的记账 key，中央映射树按这一版每个进映射单元的 key。
pub(super) struct MultiLevelTreesOfThisPublish {
    pub(super) accounting_tree: CodeTwoTreePlan,
    pub(super) deadlist_tree: CodeTwoTreePlan,
    pub(super) central_mapping_tree: CodeTwoTreePlan,
    pub(super) mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
}

/// 一次发布里普通分配那一半（`PublishPlan::ordinary_allocation_part`）：这次之后 inode 树的叶容器、extent 树的样子，
/// 与写的文件内容切成的一单元事务（没有文件版本的发布为空）。
pub(super) struct OrdinaryAllocationPartOfThePublish {
    pub(super) inode_tree: InodeLeafContainersAfterThisPublish,
    /// 这次之后 inode 树里的 n_live 与 n_记录（[`ResolvedPublish::inode_record_counts`]）。
    pub(super) inode_record_counts: InodeRecordCounts,
    pub(super) extent_tree: ExtentTreePlan,
    /// 第一个文件之外有数据单元的文件这次之后的样子（这次写的那一个整份重写，别的照抄）。
    pub(super) files_other_than_the_first: FilesOtherThanTheFirstPlan,
    /// 这次重写的数据单元的角色，与这次写的文件内容切出的一单元事务一一对应：写第一个文件（或重写它的尾单元）是
    /// [`TransactionUnit::Data`]，写第一个文件之外的文件是 [`TransactionUnit::DataOfAFileOtherThanTheFirst`]。
    pub(super) data_roles_rewritten: Vec<TransactionUnit>,
    /// 不写文件内容的发布照抄上一版第一个文件的前几个数据单元（删除的一批只留下前 `data_units_kept` 个，最后一批一个不留）；
    /// 写文件内容的发布不读它。
    pub(super) carried_data_units: usize,
    /// 这次之后 `deleted_inodes` 的号（升序）与根从哪来。
    pub(super) deleted_inodes: Vec<InodeNumber>,
    pub(super) deleted_inodes_root: DeletedInodesRootOrigin,
    pub(super) tombstone_tree: TombstoneTreeAfterThisPublish,
    /// 这次之后退役树的样子：有 inode 叶容器身份消失（删空摘掉、被左邻吸收）的那次往开放容器追加，别的照抄。
    pub(super) retirement_tree: RetirementTreeAfterThisPublish,
    /// 这次之后 logged_ops 的条目（按 key 升序）与根从哪来。
    pub(super) logged_operations: Vec<LoggedOperationEntry>,
    pub(super) logged_operations_root: LoggedOperationsRootOrigin,
}

impl OrdinaryAllocationPartOfThePublish {
    /// 这一半重写的角色（`admission::space_budget_of_role` 都是普通分配）：数据单元、extent 树的节点、inode 叶容器与 inode 根，
    /// 与 `PublishPlan::resolve` 排进 bump 次序的是同一批（实例表的几片夹在数据单元与 extent 树之间，不在这一半里）。
    pub(super) fn rewritten_roles(&self) -> Vec<TransactionUnit> {
        let mut roles: Vec<TransactionUnit> = self.data_roles_rewritten.clone();
        roles.extend(extent_tree_roles_rewritten(
            &self.extent_tree,
            &self.files_other_than_the_first,
        ));
        roles.extend(
            self.inode_tree
                .rewritten
                .iter()
                .map(|index| TransactionUnit::InodeLeafContainer(*index)),
        );
        if self.inode_tree.is_the_root_rewritten() {
            roles.push(TransactionUnit::InodeRoot);
        }
        roles.extend(self.deletion_roles_rewritten());
        roles.extend(self.logged_operations_roles_rewritten());
        roles.extend(self.retirement_roles_rewritten());
        roles
    }

    /// logged_ops 这次重写的角色：普通分配（同 `deleted_inodes` 根，只在整理签发、删意图的那次发布重写），按树 ID 排在谱系树之后、
    /// 退役树之前（D3（空间分配） 已定项 10 ⑤）。
    pub(super) fn logged_operations_roles_rewritten(&self) -> Vec<TransactionUnit> {
        match self.logged_operations_root {
            LoggedOperationsRootOrigin::RewrittenThisPublish => {
                vec![TransactionUnit::LoggedOperationsRoot]
            }
            LoggedOperationsRootOrigin::NoRoot
            | LoggedOperationsRootOrigin::CarriedFromThePreviousVersion => Vec::new(),
        }
    }

    /// 退役树这次重写的角色（树内先叶后根）：普通分配（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条，同墓碑），
    /// 按树 ID 排在谱系树之后、中央映射树之前（D3（空间分配） 已定项 10 ⑤）。
    pub(super) fn retirement_roles_rewritten(&self) -> Vec<TransactionUnit> {
        container_tree_roles_rewritten(
            &self.retirement_tree,
            ContainerTreeRoles::OF_THE_RETIREMENT_TREE,
        )
    }

    /// 删除那两棵树这次重写的角色，按 bump 次序（树 ID 升序：`deleted_inodes` 65544、墓碑树 65545；树内先叶后根）：
    /// 排在记账树之后、中央映射树之前（D3（空间分配） 已定项 10 ⑤）。
    pub(super) fn deletion_roles_rewritten(&self) -> Vec<TransactionUnit> {
        let mut roles = Vec::new();
        if self.deleted_inodes_root == DeletedInodesRootOrigin::RewrittenThisPublish {
            roles.push(TransactionUnit::DeletedInodesRoot);
        }
        roles.extend(container_tree_roles_rewritten(
            &self.tombstone_tree,
            ContainerTreeRoles::OF_THE_TOMBSTONE_TREE,
        ));
        roles
    }

    /// 上一版里有、这一版没有同一个单元接着的角色：删除批截掉的尾部数据单元、身份消失的 inode 叶容器（删空摘掉、被左邻吸收，
    /// 按它们在上一版里的叶序点名：它们右边那几片照抄、换到左边的叶序上，不被换下）、队列删空之后的 `deleted_inodes` 根。
    /// 它们照样被这次换下、经映射释放。
    pub(super) fn previous_roles_without_a_successor(
        &self,
        previous: &TransactionOutput,
        file_is_written_this_publish: bool,
    ) -> Vec<TransactionUnit> {
        let mut roles: Vec<TransactionUnit> = if file_is_written_this_publish {
            // 写文件内容的发布由 `roles_replaced_via_mapping` 按「这次重写几个单元」补文件变短那几个，这里不重复。
            Vec::new()
        } else {
            (self.carried_data_units..previous.data_pointers.len())
                .map(|position| {
                    TransactionUnit::Data(DataUnitIndexInFile(
                        u64::try_from(position).expect("单元序号"),
                    ))
                })
                .collect()
        };
        roles.extend(
            self.inode_tree
                .retired
                .iter()
                .map(|retired| TransactionUnit::InodeLeafContainer(retired.previous_index)),
        );
        debug_assert!(
            self.inode_tree
                .retired
                .iter()
                .all(|retired| retired.previous_index.position()
                    < previous.inode_leaf_containers.len()),
            "身份消失的叶容器在上一版里"
        );
        if self.deleted_inodes_root == DeletedInodesRootOrigin::NoRoot
            && !previous.deleted_inodes.is_empty()
        {
            roles.push(TransactionUnit::DeletedInodesRoot);
        }
        // 删光意图之后的 logged_ops 根（意图达成或丢掉的那一次）：这一版没有它，上一版的那一个照样换下、经映射释放。
        if self.logged_operations_root == LoggedOperationsRootOrigin::NoRoot
            && !previous.logged_operations.is_empty()
        {
            roles.push(TransactionUnit::LoggedOperationsRoot);
        }
        roles
    }
}
