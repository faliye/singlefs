//! extent 树的计划与第一个文件之外的文件的计划。

use super::*;

/// 这次发布之后 extent 树一个节点从哪来：照抄上一版同一个位置上的节点，或这次重写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtentTreeNodeOrigin {
    CarriedFromThePreviousVersion,
    RewrittenThisPublish,
}

/// 这次发布之后 extent 树长什么样（D8（核心索引结构） 已定项 14 的两段；第一个文件之外各文件的下段在 [`FilesOtherThanTheFirstPlan`]）：
/// 第一个文件的下段与上段。上段的形状一律由 `extent_tree::plan_the_extent_upper_segment` 按这次之后有条目的 inode 号定（判决
/// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）；不写文件内容、不删的发布整棵照抄。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtentTreePlan {
    /// 第一个文件下段这一版的节点，bump 次序（先叶后根）；只有一个数据单元（内联）时为空。
    pub lower_nodes: Vec<(ExtentLowerNodePosition, ExtentTreeNodeOrigin)>,
    /// 上段这一版的节点，bump 次序（先叶后根，根在最末）。
    pub upper_nodes: Vec<(ExtentUpperNodePosition, ExtentTreeNodeOrigin)>,
    /// 上一版被这次换下的 extent 树节点的角色（按上一版的位置与根层级取名），按上一版的 bump 次序。
    pub replaced_previous_roles: Vec<TransactionUnit>,
}

impl ExtentTreePlan {
    /// 这一版上段根的层级。
    ///
    /// # Panics
    /// 上段一个节点都没有：上段的规范形至少有一个节点（有条目的号集合空时是层级 0 的零条目根兼叶），不写的发布整棵照抄上一版的。
    #[must_use]
    pub fn upper_root_level(&self) -> u8 {
        self.upper_nodes
            .last()
            .expect("上段的规范形至少有一个节点：零条目根兼叶也是一个")
            .0
            .level
    }

    /// 这次重写的 extent 树角色，按 bump 次序（= extent 树里出生序号的发号次序）。
    #[must_use]
    pub fn rewritten_roles(&self) -> Vec<TransactionUnit> {
        let upper_root_level = self.upper_root_level();
        self.lower_nodes
            .iter()
            .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::RewrittenThisPublish)
            .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))
            .chain(
                self.upper_nodes
                    .iter()
                    .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::RewrittenThisPublish)
                    .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level)),
            )
            .collect()
    }
}

/// 这次发布之后的 extent 树。写第一个文件内容的发布：第一个文件有 `data_units` 个单元（从 0 起连号、没有洞）——一个就内联、不建下段，
/// 两个起建下段（整段重写）；上段按这次之后有条目的 inode 号（上一版的那几个加 inode 1）走统一计划（[`plan_the_extent_upper_segment`]），
/// 条目变了的是 inode 1。不写文件内容的发布（`data_units` 为 `None`）整棵照抄。
///
/// # Panics
/// 不写文件内容的发布没有上一版：它接在带树的一版之后。
#[must_use]
pub fn plan_the_extent_tree_after_this_publish(
    previous: Option<&TransactionOutput>,
    data_units_of_the_file_written_this_publish: Option<u64>,
) -> ExtentTreePlan {
    let Some(data_units) = data_units_of_the_file_written_this_publish else {
        let carried = &previous
            .expect("不写文件内容的发布接在带文件的一版之后：extent 树整棵照抄")
            .extent_tree;
        return ExtentTreePlan {
            lower_nodes: carried
                .lower_nodes
                .iter()
                .map(|(position, _)| {
                    (
                        *position,
                        ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
                    )
                })
                .collect(),
            upper_nodes: carried
                .upper_nodes
                .iter()
                .map(|(position, _)| {
                    (
                        *position,
                        ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
                    )
                })
                .collect(),
            replaced_previous_roles: Vec::new(),
        };
    };
    let mut inodes_with_an_entry_after_this_publish = previous
        .map(TransactionOutput::inodes_with_an_extent_upper_leaf_entry)
        .unwrap_or_default();
    inodes_with_an_entry_after_this_publish.insert(FIRST_INODE_NUMBER);
    let lower_nodes = lower_segment_nodes_of_a_file_without_holes(data_units)
        .into_iter()
        .map(|position| (position, ExtentTreeNodeOrigin::RewrittenThisPublish))
        .collect();
    // 换下的：上一版第一个文件的整个下段（这次每个数据单元都换了），与上一版上段里被统一计划换下的那几个（按上一版的根层级取名）。
    let previous_first_file_lower_nodes: Vec<ExtentLowerNodePosition> = previous
        .map(|version| {
            version
                .extent_tree
                .lower_nodes
                .iter()
                .map(|(position, _)| *position)
                .collect()
        })
        .unwrap_or_default();
    extent_tree_plan_with_the_unified_upper_segment(
        previous.map(|version| &version.extent_tree),
        lower_nodes,
        previous_first_file_lower_nodes,
        &inodes_with_an_entry_after_this_publish,
        &BTreeSet::from([FIRST_INODE_NUMBER]),
    )
}

/// 第一个文件的下段（`first_file_lower_nodes`，与它换下的上一版下段节点 `first_file_lower_nodes_replaced`）拼上上段的统一计划
/// （[`plan_the_extent_upper_segment`]，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）成一份 [`ExtentTreePlan`]：
/// 上段换下的节点按上一版的根层级取角色名（降层那次旧根是上一版的 `ExtentRoot`，长高那次照抄下来的旧根换成根之下的角色，由装树与排映射 key
/// 的两处按这一版与上一版各自的根层级取名）。写第一个文件、写别的文件、删除批、最后一批都经这里。
#[must_use]
pub fn extent_tree_plan_with_the_unified_upper_segment(
    previous: Option<&ExtentTreeVersion>,
    first_file_lower_nodes: Vec<(ExtentLowerNodePosition, ExtentTreeNodeOrigin)>,
    first_file_lower_nodes_replaced: Vec<ExtentLowerNodePosition>,
    inodes_with_an_entry_after_this_publish: &BTreeSet<u64>,
    inodes_whose_entry_changed: &BTreeSet<u64>,
) -> ExtentTreePlan {
    let previous_upper_nodes: Vec<ExtentUpperNodePosition> = previous
        .map(|version| {
            version
                .upper_nodes
                .iter()
                .map(|(position, _)| *position)
                .collect()
        })
        .unwrap_or_default();
    let upper = plan_the_extent_upper_segment(
        &previous_upper_nodes,
        inodes_with_an_entry_after_this_publish,
        inodes_whose_entry_changed,
    );
    let replaced_upper_roles: Vec<TransactionUnit> = match previous {
        Some(previous_version) => {
            let previous_root_level = previous_version.upper_root().level;
            upper
                .replaced_previous_positions
                .iter()
                .map(|position| role_of_extent_upper_node(*position, previous_root_level))
                .collect()
        }
        None => Vec::new(),
    };
    ExtentTreePlan {
        lower_nodes: first_file_lower_nodes,
        upper_nodes: upper.nodes,
        replaced_previous_roles: first_file_lower_nodes_replaced
            .into_iter()
            .map(TransactionUnit::ExtentLowerNode)
            .chain(replaced_upper_roles)
            .collect(),
    }
}

/// 第一个文件之外一个文件这次之后的单元从哪来：这次整份重写（按 inode 号写数据的入口写的就是它）、照抄上一版，被删除的一批截到前几个
/// （数据单元照抄留下的那几个，下段按节点各自的来历，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M1），
/// 或这次改了它的 size（截断三步、变长、只改记录，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S3）或按偏移写了它
/// （判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R4、R6）：数据单元照抄留下的那几个、`rewritten_data_units` 那一段
/// （截断意图与变长重写的尾单元，K3、K5；部分写从偏移所在的单元起到最后一个）按这次的事务重写，下段按节点各自的来历。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileOtherThanTheFirstOrigin {
    CarriedFromThePreviousVersion,
    RewrittenThisPublish,
    CutByADeletionBatch,
    SizeChangedThisPublish {
        rewritten_data_units: DataUnitsRewrittenInOnePublish,
    },
}

impl FileOtherThanTheFirstOrigin {
    /// 这个文件第 `unit` 个数据单元这次是不是重写的（不是就照抄上一版那一个）：整份重写的每一个都是，改 size 与部分写的是重写的那一段里的。
    #[must_use]
    pub fn rewrites_the_data_unit(self, unit: DataUnitIndexInFile) -> bool {
        match self {
            FileOtherThanTheFirstOrigin::RewrittenThisPublish => true,
            FileOtherThanTheFirstOrigin::SizeChangedThisPublish {
                rewritten_data_units,
            } => rewritten_data_units.contains(unit),
            FileOtherThanTheFirstOrigin::CarriedFromThePreviousVersion
            | FileOtherThanTheFirstOrigin::CutByADeletionBatch => false,
        }
    }
}

/// 第一个文件之外一个在上段里有条目的文件这次之后的样子（[`FilesOtherThanTheFirstPlan`] 的一项）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileOtherThanTheFirstPlan {
    pub inode: InodeNumber,
    /// 这次之后它有几个数据单元（从 0 起连号、中间没有洞；0 个是粘住的标签 0，判决 m4-multi-inode-size-r1 第二节 S2）。
    pub data_units: u64,
    pub origin: FileOtherThanTheFirstOrigin,
    /// 它下段这次之后的节点与各自的来历，bump 次序（先叶后根）；只有一个数据单元（内联）或一个都没有时为空。整份重写时下段每个节点都重写，
    /// 照抄时都照抄，被删除的一批截时只重写被截掉那一片叶与它的祖先（K-R3-1），改 size 时照
    /// [`crate::truncate::plan_the_lower_segment_after_a_size_change`]。
    pub lower_nodes: Vec<(ExtentLowerNodePosition, ExtentTreeNodeOrigin)>,
    /// 这次之后上段叶条目标签声明的尾部（有单元时标签 1、2 或 3、4；0 个单元时写标签 0、不读它）。
    pub tail: TailOfAFile,
}

impl FileOtherThanTheFirstPlan {
    /// 这个文件第 `position` 个数据单元的角色。
    #[must_use]
    pub fn data_role_at(&self, position: u64) -> TransactionUnit {
        TransactionUnit::DataOfAFileOtherThanTheFirst {
            inode: self.inode,
            unit: DataUnitIndexInFile(position),
        }
    }

    /// 这个文件下段一个节点的角色。
    #[must_use]
    pub fn lower_node_role(&self, position: ExtentLowerNodePosition) -> TransactionUnit {
        TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst {
            inode: self.inode,
            position,
        }
    }
}

/// 一次发布对第一个文件之外的一个文件做什么（[`plan_the_files_other_than_the_first_after_this_publish`] 的入参）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeOfAFileOtherThanTheFirst {
    /// 这次不碰第一个文件之外的文件：都照抄。
    NoFileChanges,
    /// 按 inode 号整份写 `inode`，这次切出 `data_units` 个单元（至少一个）。
    WholeWrite { inode: InodeNumber, data_units: u64 },
    /// 删 `inode` 的一批（不是最后一批）：截到前 `data_units_kept` 个单元。
    DeletionBatch {
        inode: InodeNumber,
        data_units_kept: u64,
    },
    /// 删 `inode` 的最后一批：它的数据单元与下段一个不剩（没有数据单元的 inode 这一步什么都不换下）。
    FinalDeletionBatch { inode: InodeNumber },
    /// 改 `inode` 的 size（截断三步、变长、只改记录，判决 m4-multi-inode-size-r1 第二节 S3）：单元数、重写的尾单元与尾部照 `change`，
    /// `size_after_in_bytes` 是这次之后那条记录的 size。上一版上段里没有它（size 0、没改过 size）而这次之后 size > 0 的，这次起写它那一条
    /// 标签 0、之后一直留着（第二节 S2「粘住的标签 0」）；上一版有的照旧有。
    SizeChange {
        inode: InodeNumber,
        change: SizeChangeOfTheFile,
        size_after_in_bytes: u64,
    },
}

impl ChangeOfAFileOtherThanTheFirst {
    /// 这次改的那个号（不改的为 `None`）。
    #[must_use]
    pub const fn inode(self) -> Option<InodeNumber> {
        match self {
            ChangeOfAFileOtherThanTheFirst::NoFileChanges => None,
            ChangeOfAFileOtherThanTheFirst::WholeWrite { inode, .. }
            | ChangeOfAFileOtherThanTheFirst::DeletionBatch { inode, .. }
            | ChangeOfAFileOtherThanTheFirst::FinalDeletionBatch { inode }
            | ChangeOfAFileOtherThanTheFirst::SizeChange { inode, .. } => Some(inode),
        }
    }
}

/// 第一个文件之外有数据单元的文件这次之后的样子，按 inode 号升序，与它们被这次换下的上一版角色：这次写的那一个文件整份重写
/// （上一版它每个数据单元与下段每个节点都被换下，有同名后继），删除的一批截的那一个只重写被截那一片下段叶与祖先（有同名后继）、截掉的数据单元
/// 与不再出现的下段节点没有同名后继，最后一批删的那一个整份没有同名后继；别的照抄、什么都不换下。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilesOtherThanTheFirstPlan {
    pub files: Vec<FileOtherThanTheFirstPlan>,
    /// 这次写的那个文件上一版的每个数据单元（按单元序号），第一次写它时为空。
    pub replaced_previous_data_roles: Vec<TransactionUnit>,
    /// 这次写的那个文件上一版下段的每个节点（按上一版的 bump 次序），与删除的一批重写的那几个下段节点（被截那一片叶与祖先）；
    /// 上一版内联或第一次写它时为空。
    pub replaced_previous_lower_node_roles: Vec<TransactionUnit>,
    /// 删除的一批截掉的尾部数据单元与不再出现的下段节点、最后一批删的那个文件的全部数据单元与下段节点（数据单元在前、按单元序号，
    /// 下段节点在后、按上一版的 bump 次序）：上一版里有、这一版没有同名后继（判决 m3-multi-file-unlink-r1 第三节 M1）。
    pub previous_roles_without_a_successor: Vec<TransactionUnit>,
}

impl FilesOtherThanTheFirstPlan {
    /// 上一版被这次换下、有同名后继的角色：数据单元在前、下段节点在后。
    #[must_use]
    pub fn replaced_previous_roles(&self) -> Vec<TransactionUnit> {
        self.replaced_previous_data_roles
            .iter()
            .chain(&self.replaced_previous_lower_node_roles)
            .copied()
            .collect()
    }
}

/// 这次发布之后第一个文件之外有数据单元的那几个文件（[`FilesOtherThanTheFirstPlan`]）：`change` 说这次改哪一个、怎么改，别的照抄上一版。
/// 没有上一版（第一个文件版本）时一个都没有——改第一个文件之外的文件的入口恒接在上一版之后。
///
/// # Panics
/// `change` 改的是 inode 1：第一个文件不进这一族（它的单元是 [`TransactionUnit::Data`] 那一族，由 [`ExtentTreePlan`] 管）；
/// 删除的一批截的文件在上一版里没有数据单元（剩下的多于一批才有这一步，`next_unlink_step_after` 按它上一版的单元数定）。
#[must_use]
pub fn plan_the_files_other_than_the_first_after_this_publish(
    previous: Option<&TransactionOutput>,
    change: ChangeOfAFileOtherThanTheFirst,
) -> FilesOtherThanTheFirstPlan {
    assert!(
        change
            .inode()
            .is_none_or(|inode| inode.0 != FIRST_INODE_NUMBER),
        "第一个文件的单元不进第一个文件之外的那一族"
    );
    let changed_inode = change.inode();
    let mut files: Vec<FileOtherThanTheFirstPlan> = previous
        .map(|previous_version| {
            previous_version
                .files_other_than_the_first
                .iter()
                .filter(|file| Some(file.inode) != changed_inode)
                .map(|file| FileOtherThanTheFirstPlan {
                    inode: file.inode,
                    data_units: u64::try_from(file.data_pointers.len()).expect("单元数"),
                    origin: FileOtherThanTheFirstOrigin::CarriedFromThePreviousVersion,
                    lower_nodes: file
                        .extent_lower_nodes
                        .iter()
                        .map(|(position, _)| {
                            (
                                *position,
                                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
                            )
                        })
                        .collect(),
                    tail: file.tail,
                })
                .collect()
        })
        .unwrap_or_default();
    let previous_file_of = |inode: InodeNumber| {
        previous.and_then(|previous_version| previous_version.file_other_than_the_first(inode))
    };
    let mut replaced_previous_data_roles = Vec::new();
    let mut replaced_previous_lower_node_roles = Vec::new();
    let mut previous_roles_without_a_successor = Vec::new();
    match change {
        ChangeOfAFileOtherThanTheFirst::NoFileChanges => {}
        ChangeOfAFileOtherThanTheFirst::WholeWrite { inode, data_units } => {
            assert!(
                data_units >= 1,
                "按 inode 号写数据至少写一个单元（长度 0 的内容也写一个声明长度 0 的单元）"
            );
            if let Some(previous_file) = previous_file_of(inode) {
                replaced_previous_data_roles = previous_file.data_roles();
                replaced_previous_lower_node_roles = previous_file.extent_lower_node_roles();
            }
            // 整份写从偏移 0 写满 size：切出的单元数就是 ⌈size ÷ 净荷⌉.max(1)，尾部没有洞（标签 1、2）。
            files.push(FileOtherThanTheFirstPlan {
                inode,
                data_units,
                origin: FileOtherThanTheFirstOrigin::RewrittenThisPublish,
                lower_nodes: lower_segment_nodes_of_a_file_without_holes(data_units)
                    .into_iter()
                    .map(|position| (position, ExtentTreeNodeOrigin::RewrittenThisPublish))
                    .collect(),
                tail: TailOfAFile::NoHoleAfterTheLastDataUnit,
            });
        }
        ChangeOfAFileOtherThanTheFirst::DeletionBatch {
            inode,
            data_units_kept,
        } => {
            let previous_file = previous_file_of(inode)
                .expect("删除的一批截的文件在上一版里有数据单元：剩下的多于一批才有这一步");
            let data_units_before =
                u64::try_from(previous_file.data_pointers.len()).expect("单元数");
            let lower = plan_the_lower_segment_after_a_deletion_batch(
                &previous_file.extent_lower_nodes,
                data_units_before,
                data_units_kept,
            );
            let lower_node_role = |position: &ExtentLowerNodePosition| {
                TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst {
                    inode,
                    position: *position,
                }
            };
            replaced_previous_lower_node_roles = lower
                .previous_nodes_rewritten
                .iter()
                .map(lower_node_role)
                .collect();
            previous_roles_without_a_successor = (data_units_kept..data_units_before)
                .map(|unit| TransactionUnit::DataOfAFileOtherThanTheFirst {
                    inode,
                    unit: DataUnitIndexInFile(unit),
                })
                .chain(lower.previous_nodes_dropped.iter().map(lower_node_role))
                .collect();
            // 删除的一批把 size 改成「剩下单元数 × 净荷」（D8（核心索引结构） 已定项 16 第 2 条）：剩下的单元恰好装满它，尾部没有洞——
            // 上一版尾部有洞（标签 3、4）的，这一批换回标签 1、2。
            files.push(FileOtherThanTheFirstPlan {
                inode,
                data_units: data_units_kept,
                origin: FileOtherThanTheFirstOrigin::CutByADeletionBatch,
                lower_nodes: lower.nodes,
                tail: TailOfAFile::NoHoleAfterTheLastDataUnit,
            });
        }
        ChangeOfAFileOtherThanTheFirst::FinalDeletionBatch { inode } => {
            if let Some(previous_file) = previous_file_of(inode) {
                previous_roles_without_a_successor = previous_file
                    .data_roles()
                    .into_iter()
                    .chain(previous_file.extent_lower_node_roles())
                    .collect();
            }
        }
        ChangeOfAFileOtherThanTheFirst::SizeChange {
            inode,
            change,
            size_after_in_bytes,
        } => {
            let previous_file = previous_file_of(inode);
            let previous_lower_positions: Vec<ExtentLowerNodePosition> = previous_file
                .map(|file| {
                    file.extent_lower_nodes
                        .iter()
                        .map(|(position, _)| *position)
                        .collect()
                })
                .unwrap_or_default();
            let lower =
                plan_the_lower_segment_after_a_size_change(&previous_lower_positions, change);
            let lower_node_role = |position: &ExtentLowerNodePosition| {
                TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst {
                    inode,
                    position: *position,
                }
            };
            let data_role = |unit: u64| TransactionUnit::DataOfAFileOtherThanTheFirst {
                inode,
                unit: DataUnitIndexInFile(unit),
            };
            // 重写的尾单元（K3、K5）与部分写重写的那一段里上一版有的那几个，上一版那一份有同名后继，进快照判定那一圈经
            // `replaced_previous_roles` 那条链（判决 m4-multi-inode-size-r1 第二节 S3「快照下被换下的上一版角色进 deadlist 那条链」；
            // 部分写只换下这次真重写的那几个，判决 m4-partial-write-r1 第二节 R7、截断判决 K4）。
            replaced_previous_data_roles = change
                .rewritten_data_units
                .units()
                .filter(|unit| *unit < change.data_units_before)
                .map(data_role)
                .collect();
            replaced_previous_lower_node_roles = lower
                .previous_nodes_rewritten
                .iter()
                .map(lower_node_role)
                .collect();
            previous_roles_without_a_successor = (change.data_units_after
                ..change.data_units_before)
                .map(data_role)
                .chain(lower.previous_nodes_dropped.iter().map(lower_node_role))
                .collect();
            let has_an_entry_after_this_publish =
                previous_file.is_some() || change.data_units_after > 0 || size_after_in_bytes > 0;
            if has_an_entry_after_this_publish {
                files.push(FileOtherThanTheFirstPlan {
                    inode,
                    data_units: change.data_units_after,
                    origin: FileOtherThanTheFirstOrigin::SizeChangedThisPublish {
                        rewritten_data_units: change.rewritten_data_units,
                    },
                    lower_nodes: lower.nodes,
                    tail: change.tail_after,
                });
            }
        }
    }
    files.sort_by_key(|file| file.inode);
    FilesOtherThanTheFirstPlan {
        files,
        replaced_previous_data_roles,
        replaced_previous_lower_node_roles,
        previous_roles_without_a_successor,
    }
}

/// 这次重写的 extent 树角色，按 bump 次序（= extent 树里出生序号的发号次序）：第一个文件的下段、第一个文件之外各文件的下段（按 inode 号）、
/// 上段（先叶后根）。
#[must_use]
pub fn extent_tree_roles_rewritten(
    extent_tree: &ExtentTreePlan,
    files_other_than_the_first: &FilesOtherThanTheFirstPlan,
) -> Vec<TransactionUnit> {
    let upper_root_level = extent_tree.upper_root_level();
    let first_file_lower = extent_tree
        .lower_nodes
        .iter()
        .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::RewrittenThisPublish)
        .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position));
    let lower_of_the_files_other_than_the_first =
        files_other_than_the_first.files.iter().flat_map(|file| {
            file.lower_nodes
                .iter()
                .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::RewrittenThisPublish)
                .map(|(position, _)| file.lower_node_role(*position))
        });
    let upper = extent_tree
        .upper_nodes
        .iter()
        .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::RewrittenThisPublish)
        .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level));
    first_file_lower
        .chain(lower_of_the_files_other_than_the_first)
        .chain(upper)
        .collect()
}

/// 写第一个文件之外 inode `inode` 那个文件的那次发布之后的 extent 树（第一个文件那一份 [`ExtentTreePlan`]）：第一个文件的下段整段照抄；
/// 上段走统一计划（[`plan_the_extent_upper_segment`]，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）——
/// 这次之后有条目的是上一版的那几个加 `inode`，条目变了的是 `inode`：它那一串与上一版没有的位置这次重写，别的照抄（根层级长高时旧根照抄、
/// 换成根之下的角色，新根在上面重写）；上一版有、新形状没有的节点这次换下。
#[must_use]
pub fn plan_the_extent_tree_after_writing_a_file_other_than_the_first(
    previous: &TransactionOutput,
    inode: InodeNumber,
) -> ExtentTreePlan {
    let mut inodes_with_an_entry_after_this_publish =
        previous.inodes_with_an_extent_upper_leaf_entry();
    inodes_with_an_entry_after_this_publish.insert(inode.0);
    extent_tree_plan_with_the_unified_upper_segment(
        Some(&previous.extent_tree),
        previous
            .extent_tree
            .lower_nodes
            .iter()
            .map(|(position, _)| {
                (
                    *position,
                    ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
                )
            })
            .collect(),
        Vec::new(),
        &inodes_with_an_entry_after_this_publish,
        &BTreeSet::from([inode.0]),
    )
}
