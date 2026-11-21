//! 一次发布对快照、数据单元、别的 inode 与被换下单元的改动：算下一条根要用的中间结果。

use super::*;

/// 一次发布写出的那一版的用户可见树（inode 树、extent 树、`deleted_inodes`）从哪来。
#[derive(Clone, Copy, Debug)]
pub(super) enum UserVisibleTreesOfTheNewVersion<'target> {
    /// 这次发布按它的种类写出或照抄上一版的。
    AsThisPublishLeavesThem,
    /// 管理员回退那次：取回退目标那一版的（D23（journal 的角色与格式） 已定项 14）。
    TakenFromTheRollbackTarget(&'target ModelRoot),
}

/// 一次发布对第一个文件之外的 inode（[`ModelRoot::inodes_other_than_the_first`]）做了什么。
#[derive(Clone, Copy, Debug)]
pub(super) enum InodesOtherThanTheFirstChangeOfThePublish<'content> {
    /// 照抄上一版的（管理员回退那次另取回退目标那一版的）。
    Carried,
    /// 建 `count` 个 size 0 的 inode（建 inode 的两种发布）：号从上一版的水位起连着发，水位加 `count`。
    Created { count: u64 },
    /// 把 `inode` 那个文件整份写成 `content`（[`ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst`]）。
    WholeWrite { inode: u64, content: &'content [u8] },
    /// 按偏移写 `inode` 那个文件之后它读回 `content_after`（[`ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst`]）：截断字段照抄，
    /// 单元与下段按种类带的那一段另改。
    PartialWrite {
        inode: u64,
        content_after: &'content [u8],
    },
}

/// 一次发布之后这一版的快照那几样（[`IdealModel::snapshot_bookkeeping_of_the_publish`]）。
pub(super) struct SnapshotBookkeepingOfThePublish {
    pub(super) snapshots: Rc<Vec<ModelSnapshot>>,
    pub(super) previous_snapshot_txg_of_the_live_head: ModelCheckpointTxg,
    pub(super) deadlist: Rc<Vec<ModelDeadlistEntry>>,
    pub(super) pinned_mapping_entries: u64,
    pub(super) deadlist_tree_nodes_upper_bound: u64,
    /// 这次至多写几个 deadlist 树的节点（占槽用）。
    pub(super) deadlist_tree_nodes_written: u64,
    pub(super) extent_lower_node_written_at:
        Rc<BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>>,
}

/// 一次不是回退的发布对数据单元做了什么（[`IdealModel::data_unit_change_of_the_publish`]）：之后存在几个、这次写了哪几个。被换下的是上一版存在的单元里
/// 这次重写的与单元号不小于 `data_units_after` 的（尾部放掉的）；这次新有的单元都是这次写的。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DataUnitChangeOfThePublish {
    pub(super) data_units_before: u64,
    pub(super) data_units_after: u64,
    /// 这次写的单元号：写内容的整份 0 .. 新单元数，截断与不缩短的截断至多一个尾单元，别的发布空；都小于 `data_units_after`。
    pub(super) rewritten: std::ops::Range<u64>,
}

impl DataUnitChangeOfThePublish {
    /// 上一版存在、这次换下的单元号：重写的，与尾部放掉的。
    fn replaced(&self) -> impl Iterator<Item = u64> + '_ {
        (0..self.data_units_before)
            .filter(|index| *index >= self.data_units_after || self.rewritten.contains(index))
    }

    /// 这次之后每个存在的单元的分配代：重写的取这次的 txg，别的照抄上一版的。
    ///
    /// # Panics
    /// 这次新有的单元不是这次写的（单元只能经写出现）。
    pub(super) fn data_unit_written_at_after(
        &self,
        previous: &[ModelCheckpointTxg],
        checkpoint_txg: ModelCheckpointTxg,
    ) -> Vec<ModelCheckpointTxg> {
        (0..self.data_units_after)
            .map(|index| {
                if self.rewritten.contains(&index) {
                    checkpoint_txg
                } else {
                    *previous
                        .get(usize::try_from(index).expect("单元号装得进 usize"))
                        .expect("这次没写的单元是上一版就存在的（单元只能经写出现）")
                }
            })
            .collect()
    }
}

/// 按偏移写的是哪个文件（[`IdealModel::answer_partial_write`]）：第一个文件的内容记在 [`ModelFileVersion`]，别的文件记在
/// [`ModelInodeOtherThanTheFirst`]，两处各走各的。
#[derive(Clone, Copy, Debug)]
pub(super) enum ModelFileWrittenAtAnOffset<'version> {
    TheFirstFile(&'version ModelFileVersion),
    OtherThanTheFirst {
        inode_leaf_container: ModelUnitRole,
        file: &'version ModelInodeOtherThanTheFirst,
    },
}

/// 一次按偏移写 `[offset_in_bytes, end_exclusive_in_bytes)` 从第几个单元起重写、写之后有几个单元（判决
/// `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R4、R5；模型按条款自己算一份，不调实现的计划，D13（验证路线） 已定项 5）：
/// 写之后的单元数取写之前的与这次写到的最后一个单元号加一里大的（单元从 0 起连号，洞只在尾部）；重写从偏移所在的单元起（R4：写到 o 时把
/// o 之后的单元一起重写）——偏移落在最后一个存在的单元之后（空隙、洞里、紧接末尾）时，最后一个存在的单元没满一个净荷就从它起（它要补满），
/// 否则从写之前的单元数起——到写之后最后一个单元为止。`declared_length_of_the_last_existing_unit` 是写之前最后一个存在的单元的声明长度
/// （一个单元都没有为 `None`）。`end_exclusive_in_bytes` 大于 `offset_in_bytes`（这次至少一个字节）。
pub(super) fn model_partial_write_rewritten_data_units(
    data_units_before: u64,
    declared_length_of_the_last_existing_unit: Option<u64>,
    offset_in_bytes: u64,
    end_exclusive_in_bytes: u64,
) -> (u64, u64) {
    let payload = data_unit_payload_capacity_in_bytes();
    let first_written_data_unit = offset_in_bytes / payload;
    let last_written_data_unit = (end_exclusive_in_bytes - 1) / payload;
    let data_units_after = data_units_before.max(last_written_data_unit + 1);
    let first_rewritten_data_unit = if first_written_data_unit < data_units_before {
        first_written_data_unit
    } else {
        match declared_length_of_the_last_existing_unit {
            Some(declared_length) if declared_length < payload => data_units_before - 1,
            Some(_) | None => data_units_before,
        }
    };
    (first_rewritten_data_unit, data_units_after)
}

/// 整份重写文件内容的入口按哪个契约收内容：覆盖写一个数据单元以内（装不下拒，D4（校验和位置） 已定项 5），顺序写不按单元数拒。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WholeFileWriteContract {
    OneDataUnit,
    AnyNumberOfDataUnits,
}

/// 删除那几步每批释放几个数据单元：产品取一片下段叶罩的 144 个（D8（核心索引结构） 已定项 16 第 2 条，K 取
/// `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）。模型只罩产品批大小：随机历史的挂载不装只供测试压小批大小的开关。
pub(super) const MODEL_DELETION_BATCH_DATA_UNITS: u64 = EXTENT_TREE_LOWER_LEAF_DATA_UNITS;

/// 一步里计划的一次发布之后的上界（单元区墙区间的上端按它判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PlannedPublishUpperBounds {
    pub(super) occupied_slots_per_device: u64,
}

impl IdealModel {
    /// 这次发布之后这一版的快照那几样（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3、S4 与 S7）：
    /// - 这次换下的用户可见单元（[`ModelUnitRole::asks_the_snapshot_lifetime_judgement`] 那几类，加 extent 树下段被换下的节点）按出生 txg 与
    ///   上一版活头的 `previous_snapshot_txg`（p）判：出生 txg ≤ p 的进 deadlist（纪元 = p）、留一条映射条目，其余照旧释放（D5 已定项 19）。
    ///   换下的数据单元（[`DataUnitChangeOfThePublish::replaced`]）：覆盖写换下上一版的全部、不是最后一批的删除批与截断的批从尾部放 144 个、
    ///   最后一批放剩下的、截断意图与不缩短的截断换下重写的那一个尾单元；第一个文件版本没有上一版，什么都不换下。每个按它自己的出生 txg 判
    ///   （判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4）。
    /// - 建快照追加一个，活头前驱换成这次的 txg；删快照的意图给第 `position` 个带上「销毁中」、把它的前驱传给下一侧；每一批放掉它那一段
    ///   （纪元落在 [它的诞生 txg, 下一侧的诞生 txg 或这一批的 txg)、出生 txg 晚于它的前驱）里的条目，最后一批放掉剩下的、删掉它。
    /// - 管理员回退那次：回退在现行那一版或目标那一版带快照时拒（判决 S8），走到这里两版都没有快照、活头前驱是 0，一个单元都不钉；
    ///   extent 树下段取目标那一版的。
    ///
    /// `extent_upper_nodes_replaced`：extent 上段按位置换下的节点；`replaced_units_besides_the_first_file`：这次换下的第一个文件之外那个文件的数据单元与
    /// 下段节点（判决 m4-multi-inode-size-r1 第二节 S3、S5：快照下别的文件换下的单元同样判钉不钉）。回退那次两样都不用（两版都没有快照）。
    #[allow(
        clippy::too_many_arguments,
        reason = "一次发布换下的单元按来处各是一个参数，收成结构体只会多一层没人验的名字"
    )]
    pub(super) fn snapshot_bookkeeping_of_the_publish(
        previous: &ModelRoot,
        kind: ModelPublishKind,
        checkpoint_txg: ModelCheckpointTxg,
        user_visible_trees: UserVisibleTreesOfTheNewVersion<'_>,
        data_unit_change: &DataUnitChangeOfThePublish,
        extent_upper_nodes_replaced: Vec<ModelReplacedUnit>,
        replaced_units_besides_the_first_file: Vec<ModelReplacedUnit>,
    ) -> SnapshotBookkeepingOfThePublish {
        let mut snapshots = previous.snapshots.as_ref().clone();
        let mut previous_snapshot_txg_of_the_live_head =
            previous.previous_snapshot_txg_of_the_live_head;
        let mut deadlist = previous.deadlist.as_ref().clone();
        let mut pinned_mapping_entries = previous.pinned_mapping_entries;
        let (extent_lower_node_written_at, births_of_the_replaced_units) = match user_visible_trees
        {
            UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(target_root) => {
                assert!(
                    previous.snapshots.is_empty() && target_root.snapshots.is_empty(),
                    "回退在现行那一版或目标那一版带快照时拒（`answer_rollback_while_mounted` 判过）：现行 {:?}，目标 {:?}",
                    previous.snapshots,
                    target_root.snapshots
                );
                (
                    Rc::clone(&target_root.extent_lower_node_written_at),
                    Vec::new(),
                )
            }
            UserVisibleTreesOfTheNewVersion::AsThisPublishLeavesThem => {
                let (extent_lower_node_written_at, mut births) =
                    Self::units_replaced_by_the_publish(
                        previous,
                        kind,
                        checkpoint_txg,
                        data_unit_change,
                        extent_upper_nodes_replaced,
                    );
                births.extend(replaced_units_besides_the_first_file);
                (extent_lower_node_written_at, births)
            }
        };
        let live_head_previous_snapshot_txg = previous.previous_snapshot_txg_of_the_live_head;
        for replaced in births_of_the_replaced_units {
            if replaced.birth_txg <= live_head_previous_snapshot_txg {
                deadlist.push(ModelDeadlistEntry {
                    epoch: live_head_previous_snapshot_txg,
                    mapping_key_unit_class: replaced.mapping_key_unit_class,
                    mapping_key_birth_tree: replaced.mapping_key_birth_tree,
                    birth_txg: replaced.birth_txg,
                });
                pinned_mapping_entries += 1;
            }
        }
        match kind {
            ModelPublishKind::SnapshotCreation => {
                snapshots.push(ModelSnapshot {
                    birth_txg: checkpoint_txg,
                    previous_snapshot_txg: previous.previous_snapshot_txg_of_the_live_head,
                    is_being_destroyed: false,
                    // 码 14 抄的是活头 inode 树的根：截断的批与最后一批只写 extent 树（判决
                    // `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2），那之后它与用户可见树最近的写者不同。
                    inode_tree_written_by: previous
                        .contents
                        .inode_tree_written_by()
                        .expect("建快照只接在有树的一版后面（`answer_create_snapshot` 判过）"),
                    extent_root_written_at: *previous
                        .role_written_at
                        .get(&ModelUnitRole::ExtentRoot)
                        .expect("有树的一版恒有 extent 树的根（删完的一版是零条目根）"),
                    content_at_creation: previous
                        .contents
                        .live_file()
                        .map(|file| Rc::clone(&file.content)),
                });
                previous_snapshot_txg_of_the_live_head = checkpoint_txg;
            }
            ModelPublishKind::SnapshotDestructionIntent { position } => {
                let destroyed = snapshots
                    .get_mut(position)
                    .expect("意图点名的快照在上一版里（`answer_destroy_snapshot` 判过）");
                destroyed.is_being_destroyed = true;
                let predecessor = destroyed.previous_snapshot_txg;
                match snapshots.get_mut(position + 1) {
                    Some(next_snapshot) => next_snapshot.previous_snapshot_txg = predecessor,
                    None => previous_snapshot_txg_of_the_live_head = predecessor,
                }
            }
            ModelPublishKind::SnapshotDestructionBatch {
                position,
                entries_released,
            } => {
                let released: Vec<ModelDeadlistEntry> =
                    Self::deadlist_entries_the_destruction_releases(
                        previous,
                        position,
                        checkpoint_txg,
                    )
                    .into_iter()
                    .take(usize::try_from(entries_released).expect("一批至多 144 条"))
                    .collect();
                Self::remove_released_entries(&mut deadlist, &released);
                pinned_mapping_entries -= u64::try_from(released.len()).expect("条数");
            }
            ModelPublishKind::SnapshotDestructionFinalBatch { position } => {
                let released = Self::deadlist_entries_the_destruction_releases(
                    previous,
                    position,
                    checkpoint_txg,
                );
                Self::remove_released_entries(&mut deadlist, &released);
                pinned_mapping_entries -= u64::try_from(released.len()).expect("条数");
                snapshots.remove(position);
            }
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits { .. }
            | ModelPublishKind::FinalDeletionBatch { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => {}
        }
        let deadlist_tree_nodes_upper_bound = deadlist_tree_nodes_upper_bound(
            previous.deadlist_tree_nodes_upper_bound,
            u64::try_from(deadlist.len()).expect("条数"),
        );
        let deadlist_tree_nodes_written = if previous.deadlist.is_empty() && deadlist.is_empty() {
            0
        } else {
            deadlist_tree_nodes_upper_bound
        };
        SnapshotBookkeepingOfThePublish {
            snapshots: Rc::new(snapshots),
            previous_snapshot_txg_of_the_live_head,
            deadlist: Rc::new(deadlist),
            pinned_mapping_entries,
            deadlist_tree_nodes_upper_bound,
            deadlist_tree_nodes_written,
            extent_lower_node_written_at,
        }
    }

    /// 一次不是回退的发布对数据单元做了什么（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2、K3、K5 与 D8（核心索引结构）
    /// 已定项 16 第 2 条）：写内容的整份重写；删除的批与截断的批从尾部放 144 个，删除的最后一批放光、截断的最后一批放到只剩 size 罩的那几个；
    /// 截断意图重写尾单元时写单元 ⌈N ÷ 净荷⌉.max(1) − 1，不缩短的截断补零重写时写最后一个存在的单元；别的发布不动数据单元。
    /// 第一个文件活着时这次发布只换了 inode 树（建 inode）或 inode 树与 extent 树（按 inode 号写第一个文件之外的文件）：第一个文件的内容、单元、
    /// 上段叶条目的标签与截断字段照旧，两棵用户可见树的根指针换成这次写的（[`ModelFileVersion::written_by`]、
    /// [`ModelFileVersion::inode_record_written_by`]：「非空」与建快照抄的码 14 按它们认）。
    pub(super) fn file_version_whose_trees_this_publish_rewrote(
        previous_file: &ModelFileVersion,
        key: ModelRootKey,
    ) -> ModelFileVersion {
        ModelFileVersion {
            written_by: key,
            inode_record_written_by: key,
            content: Rc::clone(&previous_file.content),
            data_units_present: previous_file.data_units_present,
            extent_upper_leaf_entry_label: previous_file.extent_upper_leaf_entry_label,
            truncation_field: previous_file.truncation_field,
        }
    }

    /// 这次发布让 extent 上段里哪几个号的条目变了（进了有条目的号集合、出了它、或条目内容变了；规范形计划里罩着它们的节点重写，判决
    /// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）：按 inode 号写那个号；删除批删的那个号；最后一批删的那个号
    /// （它本来有条目时，`inodes_with_an_entry_before` 是上一版有条目的号）；意图不动 extent 树；只写第一个文件的那几种（写内容、截断那几样）在
    /// 重写上段根时是 inode 1；别的发布一个都没有。
    pub(super) fn inodes_whose_extent_upper_leaf_entry_the_publish_changes(
        kind: ModelPublishKind,
        inodes_other_than_the_first_change: InodesOtherThanTheFirstChangeOfThePublish<'_>,
        inodes_with_an_entry_before: &BTreeSet<u64>,
    ) -> BTreeSet<u64> {
        let first_file_when_the_kind_rewrites_the_upper_root = || {
            if kind
                .rewritten_roles_fixed_by_the_kind()
                .contains(&ModelUnitRole::ExtentRoot)
            {
                BTreeSet::from([FIRST_INODE_NUMBER])
            } else {
                BTreeSet::new()
            }
        };
        match kind {
            ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. } => {
                match inodes_other_than_the_first_change {
                    InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { inode, .. } => {
                        BTreeSet::from([inode])
                    }
                    InodesOtherThanTheFirstChangeOfThePublish::Carried
                    | InodesOtherThanTheFirstChangeOfThePublish::Created { .. }
                    | InodesOtherThanTheFirstChangeOfThePublish::PartialWrite { .. } => {
                        panic!("按 inode 号写那一种恰好带写了谁（`next_root_with_the_user_visible_trees` 开头断言过）")
                    }
                }
            }
            // 按偏移写别的文件：重写了它的单元，它那一条上段叶条目变了（判决 m4-partial-write-r1 第二节 R4、R6）。
            ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { inode, .. } => {
                BTreeSet::from([inode])
            }
            ModelPublishKind::UnlinkIntent { .. } => BTreeSet::new(),
            // 没有数据单元的 inode 第一次写上粘住的标签 0：它进了有条目的号集合；别的改 size 一条条目都不动（判决 m4-multi-inode-size-r1 第二节 S2）。
            ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                inode,
                change:
                    ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingWritingTheTagZeroEntry,
                ..
            } => BTreeSet::from([inode]),
            ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits {
                change:
                    ModelSizeChangeOfAnInodeWithoutDataUnits::WithoutShrinkingCarryingTheExtentTree
                    | ModelSizeChangeOfAnInodeWithoutDataUnits::TruncationIntent,
                ..
            }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => {
                BTreeSet::new()
            }
            // 带数据单元的那种改 size：尾单元重写或标签要换时它那一条变了（`transaction::SizeChangeOfTheFile::rewrites_the_upper_leaf_entry`
            // 那一读法：有单元重写、单元数变了、或尾部有没有洞变了）；它截断的批与放了单元的最后一批单元数变了。
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                inode,
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten
                    | ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel,
                ..
            }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { inode }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                inode,
                release: ModelFinalTruncationBatchRelease::AtLeastOneDataUnit,
                ..
            } => BTreeSet::from([inode]),
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                extent_tree: ModelExtentTreeChangeOfASizeChange::Untouched,
                ..
            }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits {
                release: ModelFinalTruncationBatchRelease::NoDataUnit,
                ..
            } => BTreeSet::new(),
            ModelPublishKind::DeletionBatchKeepingDataUnits { deleted } => {
                BTreeSet::from([deleted.inode_number()])
            }
            ModelPublishKind::FinalDeletionBatch { deleted, .. } => {
                if inodes_with_an_entry_before.contains(&deleted.inode_number()) {
                    BTreeSet::from([deleted.inode_number()])
                } else {
                    BTreeSet::new()
                }
            }
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. } => {
                first_file_when_the_kind_rewrites_the_upper_root()
            }
        }
    }

    /// 第一个文件已删完（树里没有 inode 1 的记录）的一版：树里还有 `inode_records_in_the_tree` 条别的 inode 的记录时是带文件的一版
    /// （D8（核心索引结构） 已定项 16 第 5 条，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M3：inode 树还有任一记录的一版
    /// 是带文件的一版，不管 inode 1 在不在），一条都没有才是有树而文件集合为空的一版。`key` 是写出这一版 inode 树的那次发布。
    pub(super) fn contents_without_the_first_file(
        key: ModelRootKey,
        inode_records_in_the_tree: u64,
    ) -> ModelVersionContents {
        if inode_records_in_the_tree == 0 {
            ModelVersionContents::TreesAndNoFile {
                trees_written_by: key,
            }
        } else {
            ModelVersionContents::InodesWithoutTheFirstFile {
                trees_written_by: key,
                inode_tree_written_by: key,
                inode_records_in_the_tree,
            }
        }
    }

    /// 一次只动第一个文件之外的一个 inode 的发布（按 inode 号写、删它的意图、批与最后一批）之后这一版里有什么：第一个文件活着时它的内容、
    /// 单元、标签与截断字段照旧，两棵用户可见树的根指针换成这次写的；第一个文件已删完时按这次之后树里还有几条记录
    /// （`inode_records_in_the_tree_after`，就是第一个文件之外的 inode 数）归带文件的一版或有树而文件集合为空的一版。
    ///
    /// # Panics
    /// 上一版不是这两种（树表 0 条、有树而文件集合为空、第一个文件删除中或截断途中）：答法只在这两种上发这几种发布（会话先走空）。
    /// 第一个文件之外一个 inode 截断的批与放了单元的最后一批之后这一版里有什么（K2「批不碰 inode 记录」，判决 m4-multi-inode-size-r1 第二节 S3）：
    /// 只写 extent 树，第一个文件活着时它的内容、单元、标签与截断字段照旧、只有 extent 树那条根指针换成这次写的
    /// （[`ModelFileVersion::written_by`] 换、[`ModelFileVersion::inode_record_written_by`] 照抄）；第一个文件已删完时同样只换
    /// `trees_written_by`、inode 树的写者照抄。
    ///
    /// # Panics
    /// 上一版不是这两种：截断条目点名别的 inode 只在第一个文件活着、或已删完而树里还有别的 inode 的一版上（会话先走空）。
    pub(super) fn contents_whose_extent_tree_a_truncation_batch_of_another_inode_rewrote(
        previous_contents: &ModelVersionContents,
        key: ModelRootKey,
        kind: ModelPublishKind,
    ) -> ModelVersionContents {
        match previous_contents {
            ModelVersionContents::LiveFile(previous_file) => {
                ModelVersionContents::LiveFile(ModelFileVersion {
                    written_by: key,
                    inode_record_written_by: previous_file.inode_record_written_by,
                    content: Rc::clone(&previous_file.content),
                    data_units_present: previous_file.data_units_present,
                    extent_upper_leaf_entry_label: previous_file.extent_upper_leaf_entry_label,
                    truncation_field: previous_file.truncation_field,
                })
            }
            ModelVersionContents::InodesWithoutTheFirstFile {
                inode_tree_written_by,
                inode_records_in_the_tree,
                ..
            } => ModelVersionContents::InodesWithoutTheFirstFile {
                trees_written_by: key,
                inode_tree_written_by: *inode_tree_written_by,
                inode_records_in_the_tree: *inode_records_in_the_tree,
            },
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => panic!(
                "别的 inode 截断的批只接在第一个文件活着、或已删完而树里还有别的 inode 的一版后面（`answer_truncation_step` 判过）：{kind:?} 接在 {previous_contents:?}"
            ),
        }
    }

    pub(super) fn contents_whose_trees_a_change_of_another_inode_rewrote(
        previous_contents: &ModelVersionContents,
        key: ModelRootKey,
        inode_records_in_the_tree_after: u64,
        kind: ModelPublishKind,
    ) -> ModelVersionContents {
        match previous_contents {
            ModelVersionContents::LiveFile(previous_file) => ModelVersionContents::LiveFile(
                Self::file_version_whose_trees_this_publish_rewrote(previous_file, key),
            ),
            ModelVersionContents::InodesWithoutTheFirstFile { .. } => {
                Self::contents_without_the_first_file(key, inode_records_in_the_tree_after)
            }
            ModelVersionContents::WithoutTrees
            | ModelVersionContents::TreesAndNoFile { .. }
            | ModelVersionContents::FileBeingDeleted { .. }
            | ModelVersionContents::FileBeingTruncated { .. } => panic!(
                "只动第一个文件之外一个 inode 的发布只接在第一个文件活着、或已删完而树里还有别的 inode 的一版后面（答法判过）：{kind:?} 接在 {previous_contents:?}"
            ),
        }
    }

    /// 一次不是回退的发布对第一个文件之外哪个文件的数据单元做了什么（同第一个文件那一份 [`IdealModel::data_unit_change_of_the_publish`]，判决
    /// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3）：按 inode 号整份写那个文件全部重写；带数据单元的那种改 size 截断意图重写
    /// 单元 ⌈N ÷ 净荷⌉.max(1) − 1、不缩短的补零重写最后一个存在的单元（尾单元要重写时），一个都不放；它截断的批从尾部放 144 个、最后一批放到只剩 size
    /// 罩的那几个；删它的批从尾部放 144 个、最后一批放光。别的发布不动第一个文件之外的单元，交 `None`。
    ///
    /// # Panics
    /// 点名的号不在上一版里（答法判过），或按 inode 号写那一种没带写了谁。
    pub(super) fn data_unit_change_of_an_inode_other_than_the_first(
        previous: &ModelRoot,
        kind: ModelPublishKind,
        inodes_other_than_the_first_change: InodesOtherThanTheFirstChangeOfThePublish<'_>,
    ) -> Option<(u64, DataUnitChangeOfThePublish)> {
        let inode_before = |inode: u64| {
            previous
                .inodes_other_than_the_first
                .get(&inode)
                .expect("这次动的号是上一版树里第一个文件之外的号（答法判过）")
        };
        let keeping_the_count = |inode: u64, rewritten: std::ops::Range<u64>| {
            let data_units_before = inode_before(inode).data_units_present();
            (
                inode,
                DataUnitChangeOfThePublish {
                    data_units_before,
                    data_units_after: data_units_before,
                    rewritten,
                },
            )
        };
        let releasing_down_to = |inode: u64, data_units_after: u64| {
            (
                inode,
                DataUnitChangeOfThePublish {
                    data_units_before: inode_before(inode).data_units_present(),
                    data_units_after,
                    rewritten: 0..0,
                },
            )
        };
        match kind {
            ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. } => {
                let InodesOtherThanTheFirstChangeOfThePublish::WholeWrite { inode, content } =
                    inodes_other_than_the_first_change
                else {
                    panic!("按 inode 号写那一种恰好带写了谁（`next_root_with_the_user_visible_trees` 开头断言过）");
                };
                let data_units_after =
                    data_units_of_content(u64::try_from(content.len()).expect("内容长度"));
                Some((
                    inode,
                    DataUnitChangeOfThePublish {
                        data_units_before: inode_before(inode).data_units_present(),
                        data_units_after,
                        rewritten: 0..data_units_after,
                    },
                ))
            }
            ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits {
                inode,
                new_size_in_bytes,
                direction,
                extent_tree,
                ..
            } => {
                let data_units_before = inode_before(inode).data_units_present();
                let rewritten = match (extent_tree, direction) {
                    (
                        ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
                        ModelDirectionOfASizeChange::TruncationIntent,
                    ) => {
                        let tail_data_unit = data_units_of_content(new_size_in_bytes) - 1;
                        tail_data_unit..tail_data_unit + 1
                    }
                    (
                        ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
                        ModelDirectionOfASizeChange::WithoutShrinking,
                    ) => data_units_before - 1..data_units_before,
                    (
                        ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
                        | ModelExtentTreeChangeOfASizeChange::Untouched,
                        ModelDirectionOfASizeChange::TruncationIntent
                        | ModelDirectionOfASizeChange::WithoutShrinking,
                    ) => 0..0,
                };
                Some(keeping_the_count(inode, rewritten))
            }
            // 按偏移写（判决 m4-partial-write-r1 第二节 R4、R5）：从 `first_rewritten_data_unit` 起到写之后最后一个单元一起重写，一个都不放。
            ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst {
                inode,
                first_rewritten_data_unit,
                data_units_after,
                ..
            } => Some((
                inode,
                DataUnitChangeOfThePublish {
                    data_units_before: inode_before(inode).data_units_present(),
                    data_units_after,
                    rewritten: first_rewritten_data_unit..data_units_after,
                },
            )),
            ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { inode }
            | ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::OtherThanTheFirst(inode),
            } => Some(releasing_down_to(
                inode,
                inode_before(inode).data_units_present() - MODEL_DELETION_BATCH_DATA_UNITS,
            )),
            ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { inode, .. } => {
                let truncated = inode_before(inode);
                Some(releasing_down_to(
                    inode,
                    truncated
                        .data_units_present()
                        .min(truncated.data_units_within_the_size()),
                ))
            }
            ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::OtherThanTheFirst(inode),
                ..
            } => Some(releasing_down_to(inode, 0)),
            ModelPublishKind::FirstFileVersion
            | ModelPublishKind::OverwriteFileVersion
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::FirstFile,
            }
            | ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::FirstFile,
                ..
            }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            | ModelPublishKind::TruncationIntent { .. }
            | ModelPublishKind::TruncationBatch
            | ModelPublishKind::FinalTruncationBatch { .. }
            | ModelPublishKind::SizeChangeWithoutShrinking { .. }
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::PartialWriteOfTheFirstFile { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => None,
        }
    }

    pub(super) fn data_unit_change_of_the_publish(
        previous: &ModelRoot,
        kind: ModelPublishKind,
        new_file_content: Option<&[u8]>,
    ) -> DataUnitChangeOfThePublish {
        let data_units_before = u64::try_from(previous.data_unit_written_at.len()).expect("单元数");
        let unchanged = DataUnitChangeOfThePublish {
            data_units_before,
            data_units_after: data_units_before,
            rewritten: 0..0,
        };
        let data_units_of_the_new_content = || {
            data_units_of_content(
                u64::try_from(
                    new_file_content
                        .expect("写内容的、截断意图与不缩短的截断带读回的整份")
                        .len(),
                )
                .expect("内容长度"),
            )
        };
        match kind {
            ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => {
                let data_units_after = data_units_of_the_new_content();
                DataUnitChangeOfThePublish {
                    data_units_before,
                    data_units_after,
                    rewritten: 0..data_units_after,
                }
            }
            // 按偏移写第一个文件（判决 m4-partial-write-r1 第二节 R4、R5）：从 `first_rewritten_data_unit` 起到写之后最后一个单元一起重写，一个都不放。
            ModelPublishKind::PartialWriteOfTheFirstFile {
                first_rewritten_data_unit,
                data_units_after,
            } => DataUnitChangeOfThePublish {
                data_units_before,
                data_units_after,
                rewritten: first_rewritten_data_unit..data_units_after,
            },
            ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::FirstFile,
            }
            | ModelPublishKind::TruncationBatch => DataUnitChangeOfThePublish {
                data_units_before,
                data_units_after: data_units_before - MODEL_DELETION_BATCH_DATA_UNITS,
                rewritten: 0..0,
            },
            ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::FirstFile,
                ..
            } => DataUnitChangeOfThePublish {
                data_units_before,
                data_units_after: 0,
                rewritten: 0..0,
            },
            // 删第一个文件之外的号：这里只记第一个文件的单元，它们一个不动（那个号的单元记在它自己那一项里）。
            ModelPublishKind::DeletionBatchKeepingDataUnits {
                deleted: ModelDeletedInode::OtherThanTheFirst(_),
            }
            | ModelPublishKind::FinalDeletionBatch {
                deleted: ModelDeletedInode::OtherThanTheFirst(_),
                ..
            } => unchanged,
            // 截回洞里的（存在的单元不多于 size 罩的）一个都不放（K5、K11）。
            ModelPublishKind::FinalTruncationBatch { .. } => DataUnitChangeOfThePublish {
                data_units_before,
                data_units_after: data_units_before.min(
                    previous
                        .contents
                        .live_file()
                        .expect("截断的最后一批只接在截断途中的那一版后面（`answer_truncation_step` 判过）")
                        .data_units_within_the_size(),
                ),
                rewritten: 0..0,
            },
            ModelPublishKind::TruncationIntent {
                extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
            } => {
                let tail_data_unit = data_units_of_the_new_content() - 1;
                DataUnitChangeOfThePublish {
                    data_units_before,
                    data_units_after: data_units_before,
                    rewritten: tail_data_unit..tail_data_unit + 1,
                }
            }
            ModelPublishKind::SizeChangeWithoutShrinking {
                extent_tree: ModelExtentTreeChangeOfASizeChange::TailDataUnitRewritten,
            } => DataUnitChangeOfThePublish {
                data_units_before,
                data_units_after: data_units_before,
                rewritten: data_units_before - 1..data_units_before,
            },
            ModelPublishKind::TruncationIntent {
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
                    | ModelExtentTreeChangeOfASizeChange::Untouched,
            }
            | ModelPublishKind::SizeChangeWithoutShrinking {
                extent_tree:
                    ModelExtentTreeChangeOfASizeChange::OnlyTheUpperLeafEntryLabel
                    | ModelExtentTreeChangeOfASizeChange::Untouched,
            }
            | ModelPublishKind::RowsOnFileVersion
            | ModelPublishKind::EmptyOnFileVersion
            | ModelPublishKind::RowsOnVersionWithoutFile
            | ModelPublishKind::ZeroUnit
            | ModelPublishKind::UnlinkIntent { .. }
            | ModelPublishKind::NewInode
            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_)
            | ModelPublishKind::SnapshotCreation
            | ModelPublishKind::SnapshotDestructionIntent { .. }
            | ModelPublishKind::SnapshotDestructionBatch { .. }
            | ModelPublishKind::SnapshotDestructionFinalBatch { .. }
            | ModelPublishKind::CompactionBatch { .. }
            | ModelPublishKind::CompactionIntentIssue
            | ModelPublishKind::CompactionIntentDeletion { .. }
            // 按 inode 号写第一个文件之外的文件：这里只记第一个文件的单元，它们一个不动。
            | ModelPublishKind::WholeWriteOfAnInodeOtherThanTheFirst { .. }
            // 没有数据单元的 inode 改 size：一个单元都不动（第一个文件的也不动）。
            | ModelPublishKind::SizeChangeOfAnInodeWithoutDataUnits { .. }
            | ModelPublishKind::SizeChangeOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::TruncationBatchOfAnInodeHoldingDataUnits { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeHoldingDataUnits { .. }
            // 按偏移写第一个文件之外的文件：这里只记第一个文件的单元，它们一个不动。
            | ModelPublishKind::PartialWriteOfAnInodeOtherThanTheFirst { .. }
            | ModelPublishKind::FinalTruncationBatchOfAnInodeWithoutDataUnits { .. } => unchanged,
        }
    }

    /// 一次不是回退的发布之后第一个文件 extent 树下段每个节点的分配代，与这次换下的用户可见单元各自的出生 txg（一个单元一项）：第一个文件的数据单元与
    /// 下段节点（[`IdealModel::data_units_and_lower_nodes_after_a_data_unit_change`]）、extent 上段按位置换下的节点（`extent_upper_nodes_replaced`，
    /// 调用方在 `next_root_with_the_user_visible_trees` 里认好，根与根之下的一样），与这次重写或删掉的 inode 树那几个角色；第一个文件之外的文件的
    /// 单元另算，由调用方并进来。
    fn units_replaced_by_the_publish(
        previous: &ModelRoot,
        kind: ModelPublishKind,
        checkpoint_txg: ModelCheckpointTxg,
        data_unit_change: &DataUnitChangeOfThePublish,
        extent_upper_nodes_replaced: Vec<ModelReplacedUnit>,
    ) -> (
        Rc<BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>>,
        Vec<ModelReplacedUnit>,
    ) {
        let (lower_nodes_after, mut births) =
            Self::data_units_and_lower_nodes_after_a_data_unit_change(
                &previous.data_unit_written_at,
                &previous.extent_lower_node_written_at,
                data_unit_change,
                checkpoint_txg,
            );
        // extent 上段换下的节点（按位置认，调用方算好的），排在 inode 树那几个角色之前（同按角色次序认上段根时的次序）。
        births.extend(extent_upper_nodes_replaced);
        // 别的用户可见角色：这次重写或删掉、上一版里有的（第一个文件版本那一版上没有它们）。数据单元上面按个数另数，extent 上段的节点按位置另认。
        let roles_rewritten_or_removed: BTreeSet<ModelUnitRole> = kind
            .rewritten_roles()
            .into_iter()
            .chain(kind.removed_roles().iter().copied())
            .filter(|role| {
                role.asks_the_snapshot_lifetime_judgement()
                    && *role != ModelUnitRole::Data
                    && !role.is_a_node_of_the_extent_upper_segment()
            })
            .collect();
        births.extend(roles_rewritten_or_removed.iter().filter_map(|role| {
            let birth_txg = previous.role_written_at.get(role).copied()?;
            let (mapping_key_unit_class, mapping_key_birth_tree) = role
                .mapping_key_unit_class_and_birth_tree()
                .expect("问快照生死判定的角色都有映射 key 的类标签与出生树");
            Some(ModelReplacedUnit {
                mapping_key_unit_class,
                mapping_key_birth_tree,
                birth_txg,
            })
        }));
        (Rc::new(lower_nodes_after), births)
    }

    /// 一个文件这次的数据单元变化（`data_unit_change`）之后它的 extent 树下段每个节点的分配代，与这次换下的数据单元和下段节点各自的出生 txg
    /// （一个单元一项）：第一个文件与第一个文件之外的文件各交自己上一版的单元（`previous_data_unit_written_at`）与下段（`previous_lower_nodes`）。
    ///
    /// 下段节点按「内容变了才写」（COW）：这次之后存在的单元的每个位置，上一版没有的、罩着这次写的单元的、罩着这次从尾部放掉的单元的
    /// （孩子少了），这次写；别的照抄上一版的分配代。写内容的整份重写因此整段重写；删除批只重写被截掉那一片下段叶与它的祖先（K-R3-1）；
    /// 截断意图重写罩着尾单元 k 的那一片到根（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K3「把罩着被重写单元 k 的那片
    /// 下段叶到根恒并进重写集」，留下的单元数是 144 的倍数时也一样）；只剩一个单元时不建下段（D8（核心索引结构） 已定项 14），整段换下。
    pub(super) fn data_units_and_lower_nodes_after_a_data_unit_change(
        previous_data_unit_written_at: &[ModelCheckpointTxg],
        previous_lower_nodes: &BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>,
        data_unit_change: &DataUnitChangeOfThePublish,
        checkpoint_txg: ModelCheckpointTxg,
    ) -> (
        BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg>,
        Vec<ModelReplacedUnit>,
    ) {
        let rewritten = &data_unit_change.rewritten;
        let lower_nodes_after: BTreeMap<ModelExtentLowerNodePosition, ModelCheckpointTxg> =
            extent_lower_node_positions(data_unit_change.data_units_after)
                .into_iter()
                .map(|position| {
                    let first_data_unit = first_data_unit_under(position);
                    let last_data_unit = last_data_unit_under(position);
                    let covers_a_rewritten_data_unit = !rewritten.is_empty()
                        && rewritten.start <= last_data_unit
                        && first_data_unit < rewritten.end;
                    let covers_a_released_data_unit = data_unit_change.data_units_after
                        < data_unit_change.data_units_before
                        && last_data_unit >= data_unit_change.data_units_after;
                    let written_at = match previous_lower_nodes.get(&position) {
                        Some(previous_written_at)
                            if !covers_a_rewritten_data_unit && !covers_a_released_data_unit =>
                        {
                            *previous_written_at
                        }
                        Some(_) | None => checkpoint_txg,
                    };
                    (position, written_at)
                })
                .collect();
        // 换下的数据单元各按自己的出生 txg（K4：截断只重写尾单元之后一个文件的数据单元不再同一个出生 txg）。
        let mut births: Vec<ModelReplacedUnit> = data_unit_change
            .replaced()
            .map(|index| ModelReplacedUnit {
                mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_DATA,
                mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                birth_txg: *previous_data_unit_written_at
                    .get(usize::try_from(index).expect("单元号装得进 usize"))
                    .expect("换下的单元是上一版存在的"),
            })
            .collect();
        // 下段节点（extent 树的索引节点）：上一版有、这一版同一个位置上分配代没变的照抄，别的都是这次换下的（整段重写、截掉、删光）。
        births.extend(
            previous_lower_nodes
                .iter()
                .filter(|(position, written_at)| {
                    lower_nodes_after.get(*position) != Some(*written_at)
                })
                .map(|(_, written_at)| ModelReplacedUnit {
                    mapping_key_unit_class: MODEL_MAPPING_KEY_UNIT_CLASS_INDEX_NODE,
                    mapping_key_birth_tree: TREE_IDENTIFIER_EXTENT,
                    birth_txg: *written_at,
                }),
        );
        (lower_nodes_after, births)
    }

    /// 删按建的先后第 `position` 个快照、这一批（txg `txg_of_this_batch`）能放的 deadlist 条目（判决 S4）：纪元落在 [lo, hi) 里、出生 txg 晚于
    /// 它的前驱 P——lo = 它的诞生 txg，hi = 下一侧是快照时取那一个的诞生 txg、是活头时取这一批的 txg（实现取的那一个，丙 2 主体交回的读法第 3 条）。
    /// 按盘上 key 的序排（[`ModelDeadlistEntry`] 的字段次序：纪元、映射 key 的类标签、出生树、出生 txg）：不是最后一批的一批放排在最前的 144 条，
    /// 与实现「按 key 升序的前一批」逐条对得上（映射 key 末段模型不记，前三段相同的几条互相顶替）。
    pub(super) fn deadlist_entries_the_destruction_releases(
        version: &ModelRoot,
        position: usize,
        txg_of_this_batch: ModelCheckpointTxg,
    ) -> Vec<ModelDeadlistEntry> {
        let destroyed = version
            .snapshots
            .get(position)
            .expect("删的快照在这一版里（`answer_snapshot_destruction_step` 判过）");
        let lowest_epoch = destroyed.birth_txg;
        let epoch_beyond = version
            .snapshots
            .get(position + 1)
            .map_or(txg_of_this_batch, |next_snapshot| next_snapshot.birth_txg);
        let mut releasable: Vec<ModelDeadlistEntry> = version
            .deadlist
            .iter()
            .filter(|entry| {
                (lowest_epoch..epoch_beyond).contains(&entry.epoch)
                    && entry.birth_txg > destroyed.previous_snapshot_txg
            })
            .copied()
            .collect();
        releasable.sort_unstable();
        releasable
    }

    /// 从 deadlist 里拿掉放掉的那几条（同样的条目可以有几条：同一次发布写的几个数据单元钉在同一个纪元上，各拿一条）。
    fn remove_released_entries(
        deadlist: &mut Vec<ModelDeadlistEntry>,
        released: &[ModelDeadlistEntry],
    ) {
        for entry in released {
            let index = deadlist
                .iter()
                .position(|kept| kept == entry)
                .expect("放掉的条目是从这一版的 deadlist 里挑的");
            deadlist.swap_remove(index);
        }
    }

    pub(super) fn upper_bounds_of(roots: &[ModelRoot]) -> Vec<PlannedPublishUpperBounds> {
        roots
            .iter()
            .map(|root| PlannedPublishUpperBounds {
                occupied_slots_per_device: root.occupied_slots_upper_bound_per_device,
            })
            .collect()
    }
}
