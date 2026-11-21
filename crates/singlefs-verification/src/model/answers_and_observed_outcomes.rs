//! 模型的回答与实现交回的结局：拒绝理由、读回、操作种类与实现那一侧观测到的几样。

use super::*;

/// 模型对拒绝理由的分法：按条款说的「为什么不能做」分，不按实现的错误成员分（胶水把成员映射过来）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelRefusalReason {
    /// 内容装不进一个数据单元（D4（校验和位置） 已定项 5）。
    ContentExceedsDataUnitPayload,
    /// 第一个文件版本要建在上面的那一版的根，与交给它的上一条记录说的不是同一版：新根的 txg 从那条根接着算、jsn 从那条记录接着算，
    /// 两者对不上时接上去会盖在别的发布上。健康的历史里走不到（执行器恒拿现行那一版的根与记录）。
    FirstFileVersionDoesNotFollowTheVersionItBuildsOn,
    /// 第一个文件版本要建在上面的那一版已经有过文件版本：那条路径按「树还没建起来」写，树表条目的诞生 txg 与
    /// inode 1 的对象出生代都会取这次的 txg，与环里那些旧根记的对不上（I-9.14、I-9.10）。再写一版走覆盖写。
    FirstFileVersionOnAVersionThatAlreadyHasAFile,
    /// 单元区装不下（D28（挂载期承诺量） 已定项 1 的准入；模型答允许拒绝的区间）。
    UnitAreaWall,
    /// 回退目标不在根环里（D23（journal 的角色与格式） 已定项 14：候选集是根环里的根）。
    RollbackTargetNotInRing,
    /// 回退目标的 txg 低于 F_生效（D16（发布语义） 已定项 1「回退候选集」）。
    RollbackTargetBelowEffectiveFloor,
    /// 回退目标在被抛弃的时间线上（D23（journal 的角色与格式） 已定项 14：有行 (i, Ti, Wi) 且 T > Ti）。
    RollbackTargetOnAbandonedTimeline,
    /// 回退目标那一版树表 0 条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根）。
    RollbackTargetWithoutFile,
    /// 要抬的 F 超过上限（D16（发布语义） 已定项 1「抬 F 的上限」）。
    FloorAboveCeiling,
    /// 可写挂载读到的样子里有比所选那一版新的发布读不出、系统配置见证过它，重读一次仍读不出（C554 乙：
    /// 「乙 重读后再判」，`research/prompts/c554-fix-forks.md` #1「挂载时有一条根（或它引用的账）暂时读不出」；重读次数取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的
    /// 「重读一次」，挂进 D23（journal 的角色与格式） 已定项 14）：取号之前拒，盘上逐字节不变。
    NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
    /// 现行那一版有树而文件集合为空（D8（核心索引结构） 已定项 16 第 5 条：删完之后只许建 inode（size 0）、写行与空发布），
    /// 这次改动不是建 inode：覆盖写、unlink 先走空队列（走空把删除中的那一版走成这一形），再拒（`UserChangeRefused::NoFileVersionToChange`）。
    /// 建快照与删快照不在此列（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）。
    OnlyNewInodesOnAVersionWithTreesAndNoFile,
    /// 现行那一版带着只读快照：第一版有快照就拒回退（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
    /// 第三节 S8 与第七节岔路 3）。
    RollbackWithSnapshotsWhichTheFirstVersionRefuses,
    /// 现行那一版已经有 [`MODEL_SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 个只读快照：第 11 个拒（「10」，判决
    /// `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1），在任何写之前。
    MoreSnapshotsThanTheFirstVersionAllows,
    /// 截断到的长度 N 按整单元向上取整（⌈N ÷ 净荷⌉ × 净荷）溢出 u64：在任何写之前拒（判决
    /// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K12）。
    TruncationSizeWhoseWholeDataUnitsOverflow,
    /// 第一个文件已删完的一版上按 inode 号给删完之后才建的 inode 写数据：D8（核心索引结构） 已定项 16 射程「删完之后不再建带数据的文件
    /// （K-U5，记欠账）」「β 时序级」——目标的对象出生代晚于 inode 1 墓碑的死亡 txg 才拒，删之前就有的别的文件照常能写；
    /// 在任何写之前拒。删第一个文件时树里还有的别的 inode 照常能写（[`ModelInodeOtherThanTheFirst::object_birth`] 不晚于那个死亡 txg）。
    DataWriteToAnInodeBornAfterTheFirstFileWasDeleted,
    /// 读—改—写要读回的旧数据单元读盘读不回、或读回来核不过（D4（校验和位置） 已定项 4：「读不回来、或读回来 MAC 验不过时，那个部分写失败」；
    /// 判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R3「读盘」，截断与变长重写尾单元那条路同一读法）：在任何写之前拒，
    /// 不拿零填充凑一个合法的零。健康的内存盘上读得回每一个单元，模型的答案里一条都不列：实现在健康的历史里拒在这一格，对拍报「模型说该成、实现拒了」。
    OldDataUnitToRewriteNotReadBack,
}

impl ModelRefusalReason {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ModelRefusalReason::ContentExceedsDataUnitPayload => "内容装不进一个数据单元",
            ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn => {
                "第一个文件版本接不上它要建在上面的那一版"
            }
            ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile => {
                "要建在上面的那一版已经有过文件版本（再写一版走覆盖写）"
            }
            ModelRefusalReason::UnitAreaWall => "单元区装不下",
            ModelRefusalReason::RollbackTargetNotInRing => "回退目标不在根环里",
            ModelRefusalReason::RollbackTargetBelowEffectiveFloor => "回退目标低于 F_生效",
            ModelRefusalReason::RollbackTargetOnAbandonedTimeline => "回退目标在被抛弃的时间线上",
            ModelRefusalReason::RollbackTargetWithoutFile => "回退目标那一版没有文件",
            ModelRefusalReason::FloorAboveCeiling => "要抬的 F 超过上限",
            ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread => {
                "系统配置见证过比所选那一版新的发布、重读一次仍读不出（C554 乙）"
            }
            ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile => {
                "有树而文件集合为空的一版只许建 inode"
            }
            ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses => {
                "带着只读快照的回退（第一版有快照就拒）"
            }
            ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows => {
                "已经有 10 个只读快照（第一版至多 10 个）"
            }
            ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow => {
                "截断到的长度按整单元向上取整溢出 u64"
            }
            ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted => {
                "删完第一个文件之后才建的 inode 写数据（K-U5 的 β 时序级读法，第一版不建带数据的文件）"
            }
            ModelRefusalReason::OldDataUnitToRewriteNotReadBack => {
                "读—改—写读不回要重写的旧数据单元（D4 已定项 4：读不回就失败）"
            }
        }
    }

    /// 容量墙：模型只答允许拒绝的区间（按这一步计划的每次发布之后的上界判），不答必须拒。
    pub(super) fn is_capacity_wall_with_an_interval(self) -> bool {
        match self {
            ModelRefusalReason::UnitAreaWall => true,
            ModelRefusalReason::ContentExceedsDataUnitPayload
            | ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn
            | ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile
            | ModelRefusalReason::RollbackTargetNotInRing
            | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
            | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
            | ModelRefusalReason::RollbackTargetWithoutFile
            | ModelRefusalReason::FloorAboveCeiling
            | ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread
            | ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile
            | ModelRefusalReason::RollbackWithSnapshotsWhichTheFirstVersionRefuses
            | ModelRefusalReason::MoreSnapshotsThanTheFirstVersionAllows
            | ModelRefusalReason::TruncationSizeWhoseWholeDataUnitsOverflow
            | ModelRefusalReason::DataWriteToAnInodeBornAfterTheFirstFileWasDeleted
            | ModelRefusalReason::OldDataUnitToRewriteNotReadBack => false,
        }
    }
}

/// 冷启动该读回什么（D23（journal 的角色与格式） 已定项 14：所选根 = 环里 (txg, 实例) 最大的那条；不建崩溃，没有要施加的记录）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelReadBack {
    NoFile {
        root: ModelRootKey,
    },
    FileRead {
        root: ModelRootKey,
        content: Rc<[u8]>,
    },
}

/// 模型问的是哪一种操作（报告用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelOperationKind {
    PublishFirstFile,
    PublishOverwrite,
    PublishSequentialWrite,
    PublishWithoutUnits,
    MountWritable,
    RollbackWhileMounted,
    RaiseRollbackFloor,
    Unmount,
    ColdStartRecover,
    PublishUnlinkIntent,
    PublishDeletionStep,
    PublishNewInode,
    PublishSnapshotCreation,
    PublishSnapshotDestructionIntent,
    /// 删快照意图之后的一批（不是最后一批的与最后一批）：会话与挂载续做的走空走它们。
    PublishSnapshotDestructionStep,
    /// 整理的一批（`mounted_session::MountedSession::run_compaction`，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7、G8），
    /// 照抄意图的与同一次发布删意图的最后一批。
    PublishCompactionBatch,
    /// 用户改动发成之后另发的那一批：修只坏一份的单元（`mounted_session::RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::Relocated`，
    /// D4（校验和位置） 已定项 4「读到只坏一份时计数；可写挂载里读到的，再经已有的搬迁路径把这个单元整个搬到新槽」）。
    PublishRelocationBatchAfterAUserChange,
    /// 签发一条整理意图（判决 G1）。
    PublishCompactionIntentIssue,
    /// 不搬单元、R 已按记录全空，单独删意图（意图达成，判决 G4）。
    PublishCompactionIntentCompletion,
    /// 不搬单元、R 里还有未释放的记录，丢掉意图（攻方 K-c；判决 G2）。
    PublishCompactionIntentDrop,
    /// 截断这次用户改动（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2、K5、K11、K12）：缩短发截断意图，之后各批由
    /// [`ModelOperationKind::PublishTruncationStep`] 答；不缩短（变长与 N = S）一次发布做完。
    PublishTruncation,
    /// 截断意图之后的一批（不是最后一批的与最后一批）：会话与挂载续做的走空走它们（K1：次序在 `deleted_inodes`、销毁中的快照之后）。
    PublishTruncationStep,
    /// 按 inode 号整份写第一个文件之外的一个文件（里程碑「删除与多次 COW」一·1）；点名 inode 1 的照覆盖写与顺序写那两种。
    PublishWholeWriteOfAnInodeOtherThanTheFirst,
    /// 按偏移写一个活 inode（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节，[`IdealModel::answer_partial_write`]）：
    /// 第一个文件与别的文件同一种操作。
    PublishPartialWrite,
}

/// 执行器交给模型的、实现交回的一次整理发布看得见的几样（[`IdealModel::answer_compaction_publish`] 按它与模型自己记的活意图定是哪一种）。
/// 哪几个单元搬、搬几个、什么时候签发、什么时候丢由实现的选段与区域扫描定（判决 G7「只比成功操作的内容」），模型不答，只判这几样合不合条款。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelCompactionPublishSeen {
    /// 一批至多搬几个（执行器装的批大小）：占槽上界按它取。
    pub units_at_most: u64,
    /// 这一次搬了几个单元。
    pub units_relocated: u64,
    /// 搬的里头用户可见树的根单元（活头与快照条目引用的 inode 树根、extent 树根），胶水从实现交回的树表条目与搬迁表认。
    pub user_visible_root_units_relocated: Vec<ModelUserVisibleRootUnit>,
    /// 发布之后这一版的 logged_ops 里还有没有活整理意图。
    pub is_an_intent_logged_after_the_publish: bool,
    /// 模型那条活意图的 R 在实现交回的这一版里按记录全空没有（模型没有活意图时 false）。
    pub is_the_region_of_the_live_intent_empty_by_the_records_after_the_publish: bool,
    /// 执行器强制选的那一段（`CompactionSwitches::forced_region`）：签发的 R 就是它。
    pub region_forced_for_the_issue: ModelCompactionRegion,
}

/// 执行器交给模型的、用户改动发成之后另发的那一批看得见的几样（[`IdealModel::answer_relocation_batch_after_a_user_change`]）。
/// 搬哪几个、换不换实例表由这次挂载读到哪一份坏了定，模型不记盘上的读，答不了，只判这一版除这几样之外逐项是上一版的内容。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRelocationBatchAfterAUserChangeSeen {
    /// 这一批搬了几个单元：占槽上界按它取。
    pub units_relocated: u64,
    /// 搬的里头用户可见树的根单元（同 [`ModelCompactionPublishSeen::user_visible_root_units_relocated`]）。
    pub user_visible_root_units_relocated: Vec<ModelUserVisibleRootUnit>,
    pub instance_table: ModelInstanceTableOfTheBatch,
}

pub(super) fn describe_compaction_publish_seen(seen: &ModelCompactionPublishSeen) -> String {
    format!(
        "搬了 {} 个单元（其中用户可见树的根 {:?}），发布之后 logged_ops 里{}活意图，活意图的 R 按记录{}",
        seen.units_relocated,
        seen.user_visible_root_units_relocated,
        if seen.is_an_intent_logged_after_the_publish {
            "有"
        } else {
            "没有"
        },
        if seen.is_the_region_of_the_live_intent_empty_by_the_records_after_the_publish {
            "全空"
        } else {
            "不空（或没有活意图）"
        }
    )
}

/// 模型对一步操作的答案。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelAnswer {
    pub operation: ModelOperationKind,
    /// 条款要求拒绝的理由：非空就必须拒，理由是这里的任一条或区间允许的任一条墙。
    pub required_refusals: BTreeSet<ModelRefusalReason>,
    /// 做成时写出的根，按次序。
    pub expected_roots: Vec<ModelRoot>,
    /// 做成时这次挂载取到的实例代号与写的行。
    pub expected_mount: Option<(ModelInstanceGeneration, Vec<ModelInstanceRow>)>,
    /// 冷启动该读回什么。
    pub expected_read_back: Option<ModelReadBack>,
    /// 抬 F 时模型算的上限（D16（发布语义） 已定项 1），拒与成都要与实现报的相等。
    pub rollback_floor_ceiling: Option<ModelCheckpointTxg>,
    /// 管理员回退的目标（只有回退那一步有）：做成时数「回退到 F_生效 那一代」的次数要它。
    pub rollback_target: Option<ModelRootKey>,
    pub(super) planned_publish_upper_bounds: Vec<PlannedPublishUpperBounds>,
    /// 容量墙在第一次写之前一串判完（可写挂载、回退、抬 F——实现在任何写之前把那一串整串预演一遍）还是逐次发布判（发布）。
    pub(super) walls_are_judged_before_the_first_write: bool,
    pub(super) session_after_success: ModelSessionAfterSuccess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ModelSessionAfterSuccess {
    /// 会话照旧开着，现行版本换成最后写出的那条根。
    StaysOpen,
    /// 这次挂载开了一个新会话，实例是这个。
    OpenedAs(ModelInstanceGeneration),
    /// 会话关着（冷启动）。
    Closed,
}

/// 实现写出的一条根，胶水从实现交回的东西里取。每一版都拿它的文件、实例表与分配记录跟模型那一版比（代码审阅第 12 条：
/// 只比有没有文件、只遍历实现交回的角色，内容只在冷启动那一步比会漏判）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedRoot {
    pub key: ModelRootKey,
    pub journal_counter: ModelJournalCounter,
    pub rollback_floor: ModelCheckpointTxg,
    pub file: ObservedFile,
    pub instance_table: ObservedInstanceTable,
    pub unit_allocation_records: ObservedUnitAllocationRecords,
    /// 这一版的快照那几样（判决 S7）：活头前驱、快照列表、deadlist 条目数、被钉映射条目数。
    pub snapshot_state: ObservedSnapshotState,
    /// 这一版 logged_ops 里那一条活整理意图（胶水从实现交回的 logged_ops 条目取；树表 0 条的一版没有）。
    pub logged_compaction_intent: Option<ModelCompactionIntent>,
    /// 这一版第一个文件那条 inode 记录里截断字段那两段（胶水从记录的字节里取，[`ObservedTruncationField`]）；这一版没有那条记录（树表 0 条、
    /// 删完、删完之后建过 inode）为 None。
    pub first_file_record_truncation_field: Option<ObservedTruncationField>,
    /// 这一版 inode 树里第一个文件之外的每个活着的 inode（nlink 不是 0）：号 → 读回的内容（按 inode 号写数据写出来的，里程碑「删除与多次 COW」一·1）。
    /// 胶水从实现交回的那个文件的数据单元按单元号接起来、接到它那条记录的 size；没有数据单元、size 0 的是空内容。树表 0 条的一版是空表。
    pub inodes_other_than_the_first: BTreeMap<u64, ObservedFile>,
    /// 这一版 inode 树里第一个文件之外每条记录（活着的与删除中的）里截断字段那两段：号 → 原样值（胶水从记录的字节里取，[`ObservedTruncationField`]；
    /// 判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5「截断字段按 inode 比」）。树表 0 条的一版是空表。
    pub truncation_fields_of_the_inodes_other_than_the_first:
        BTreeMap<u64, ObservedTruncationField>,
}

/// 实现交回的这一版的文件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedFile {
    /// 树表 0 条：这一版没有文件。
    NoFile,
    /// 从这一版的数据单元按单元号接起来解出的内容（整份，不取摘要：模型只用 std 与格式常量，没有与胶水共用的哈希）。
    Decoded(Vec<u8>),
    /// 这一版带文件，而胶水从实现交回的单元里解不出它的内容（数据单元不在、解不开）。
    Undecodable { what: String },
}

impl ObservedFile {
    #[must_use]
    pub fn has_file(&self) -> bool {
        match self {
            ObservedFile::NoFile => false,
            ObservedFile::Decoded(_) | ObservedFile::Undecodable { .. } => true,
        }
    }
}

/// 实现交回的这一版的实例表。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedInstanceTable {
    /// 从这一版的实例表单元解出来的整张表，各片的行按链上的次序接起来。
    Rows(Vec<ModelInstanceRow>),
    /// 实现交回的东西里没有这一版整条实例表链的字节，比不了（`why` 写是哪一种；模型照计数，不判）。只许出现在这次没重写实例表的那几版
    /// （mkfs 之后第一次重写之前、从盘上重建且多于一片、零单元发布）：这次重写了实例表链的那一版交回它，模型判「这一版的实例表」对不上。
    NotInTheOutput { why: &'static str },
    /// 字节在，解不开。
    Undecodable { what: String },
}

/// 实现交回的这一版的分配记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedUnitAllocationRecords {
    /// 这一版每个单元在这一版分配记录里的那几条，每块盘各一条：实现交回的这一版的全部角色与全部数据单元（带文件的一版、树表 0 条的一版上写行的那一版）。
    EveryRoleOfTheVersion(ObservedAllocationRecordsOfEveryUnit),
    /// 实现的输出不带这一版的分配记录（树表 0 条的一版上的零单元发布：一个字节都不写、不经分配器），只交回这次发布重写了哪几个角色。
    /// 模型说这次重写了角色的那一版这么交回，判「单元的分配代」对不上。同一次挂载里前一版交回过分配记录的，执行器用的胶水从它往下带、
    /// 交回 [`ObservedUnitAllocationRecords::EveryRoleOfTheVersion`]（实审 B3c-4）；执行器里这一臂只剩一次挂载里第一版就是零单元发布的那几版。
    RewrittenRolesOnly(BTreeSet<ModelUnitRole>),
    /// 零单元发布交回的根换了分配记录树根指针（该照抄同一次挂载里前一版的那一条，胶水往下带时核出来的，实审 B3c-4）：
    /// 这一版的分配记录说不上是哪一份，模型判「单元的分配代」对不上，不当比不了。`what` 写两条指针。
    AllocationRecordTreeRootChangedByAZeroUnitPublish { what: String },
}

/// 实现交回的这一版每个单元的分配记录（[`ObservedUnitAllocationRecords::EveryRoleOfTheVersion`]）：数据单元按单元号各一项，别的角色各一项。
/// 数据单元按单元号交（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4：截断只重写尾单元之后一个文件的数据单元分配代不同，
/// 模型按单元号逐个比，单元号与分配代的对应判得出）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedAllocationRecordsOfEveryUnit {
    /// 数据单元之外的每个角色：这里不该有 [`ModelUnitRole::Data`]（数据单元在 `data_units` 里按单元号交），有就判对不上。
    pub roles: Vec<(ModelUnitRole, Vec<ObservedAllocationRecord>)>,
    /// 文件的每个数据单元：单元号与它的那几条分配记录。
    pub data_units: Vec<(u64, Vec<ObservedAllocationRecord>)>,
    /// 第一个文件之外的文件的每个数据单元：(inode 号, 单元号) 与它的那几条分配记录。
    pub data_units_of_the_files_other_than_the_first:
        Vec<((u64, u64), Vec<ObservedAllocationRecord>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedAllocationRecord {
    pub device: ModelDeviceIdentity,
    pub generation: ModelCheckpointTxg,
    pub is_released: bool,
}

/// 实现冷启动读回了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedReadBack {
    NoFile {
        root: ModelRootKey,
    },
    FileRead {
        root: ModelRootKey,
        content: Vec<u8>,
    },
    Failed {
        what: String,
    },
}

/// 实现做成了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedEffect {
    /// 一次或几次发布（发布、抬 F）；抬 F 连同实现报的上限。
    Publishes {
        roots: Vec<ObservedRoot>,
        reported_ceiling: Option<ModelCheckpointTxg>,
    },
    Mount {
        instance: ModelInstanceGeneration,
        rows_written: Vec<ModelInstanceRow>,
        roots: Vec<ObservedRoot>,
    },
    ColdStart {
        read_back: ObservedReadBack,
    },
}

/// 实现的错误成员映射到模型的理由：`Explained` 里是这个成员说的那一条，`Unexplained` 是模型没有对应理由的成员（I/O、盘坏、
/// 第一版不支持的池形状……）。一个成员只映射到一条：把「不在候选集里」映射成「两条之一」的话，实现报错了是哪一条模型分不出。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservedRefusalReason {
    Explained(ModelRefusalReason),
    Unexplained,
}

/// 实现这一步的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedOutcome {
    Succeeded(ObservedEffect),
    Refused {
        member: String,
        reason: ObservedRefusalReason,
        /// 拒之前已经做完的发布次数（抬 F 半路被拒时才可能非 0）。
        publishes_completed: usize,
        /// 录制流在这一步里有没有多出写或屏障。
        wrote_anything: bool,
        /// 实现报的上限（抬 F 被上限拒时）。
        reported_ceiling: Option<ModelCheckpointTxg>,
        /// 实现报「算抬 F 的上限、或挂着时回退判候选集时这个知道住着根的根环槽读坏、重读仍坏」时点名的那个槽
        /// （D16（发布语义） 已定项 1「根槽这一次读坏」那一行）；别的拒绝都是 None。
        reported_root_ring_slot_still_bad_after_one_reread: Option<ModelRingPosition>,
    },
}
