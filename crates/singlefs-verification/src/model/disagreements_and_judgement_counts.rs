//! 模型与实现对不上时报的分歧，与一趟判下来的计数。

use super::*;

/// 模型与实现对不上的是哪一格。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelDisagreementAspect {
    SessionState,
    RefusedWhenModelRequiresSuccess,
    SucceededWhenModelRequiresRefusal,
    RefusalReason,
    WroteBeforeRefusing,
    PublishCount,
    RootIdentity,
    JournalCounter,
    RollbackFloor,
    FilePresence,
    FileContent,
    InstanceTableOfTheVersion,
    AllocationGeneration,
    InstanceGeneration,
    InstanceRows,
    RollbackFloorCeiling,
    ColdStartReadBack,
    /// 活头 inode 树那条条目的 `previous_snapshot_txg`（判决 S7「活头前驱」）。
    LiveHeadPreviousSnapshotTxg,
    /// 树表里的快照列表（判决 S7「快照列表」）：个数、每个的诞生 txg、前驱、销毁中记号、两条根指针抄的是谁。
    Snapshots,
    /// deadlist 条目，按盘上 key 的序逐条比（判决 S7 的条目数加删快照一批放的是哪几条，[`ModelDeadlistEntry`]）。
    DeadlistEntries,
    /// 留在中央映射树里、被活快照钉住的映射条目条数（判决 S7「被钉映射条目数」）。
    PinnedMappingEntries,
    /// 读快照读出的内容。
    SnapshotReadBack,
    /// 一次整理发布对意图做的事（签发、照抄、随最后一批删、单独删）与模型记的活意图、R 按记录全空没有对不上（判决 G1、G4；D8（核心索引结构） 已定项 5）。
    CompactionIntentOfThePublish,
    /// 这一版 logged_ops 里的整理意图（有没有、签发 txg、R、`c0`）。
    LoggedCompactionIntent,
    /// 这一版第一个文件那条 inode 记录里的截断字段（有没有那条记录、T、N_w；判决 `research/prompts/m4-trunc-r2-main-verification.md`
    /// 第三节 R1、R2）。
    FirstFileRecordTruncationField,
    /// 这一版第一个文件之外的 inode：有哪几个号、每个读回的内容（里程碑「删除与多次 COW」一·1）。
    InodesOtherThanTheFirst,
    /// 胶水交来的做成结局（[`ObservedEffect`] 的种类）不是这一种操作做成时交的那一种（冷启动只交冷启动读回、可写挂载只交挂载、
    /// 发布类只交发布）：接错了的胶水在判字段之前就拒，不拿空根列表比过去记成一次成功
    /// （`records/2026-10-24-singlefs-harness-恢复验证审计.md`「H03：模型允许冷启动答案被空发布结果替代」）。
    EffectKindOfTheOperation,
    NotModeled,
    /// 实现推了一次不抬 F 的空发布：准入抬 F 的上限卡在挂载那一刻读坏或读回全零、这个进程之后没写过的槽上（D16（发布语义） 已定项 1
    /// 「抬 F 的上限」那一行的推空）。模型不建这几类槽，答不了上限卡没卡在它们上，模型不答
    /// （判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 4 问；胶水 `history::judge_one_floor_raise_pushed_by_the_session`）。
    /// 与 [`ModelDisagreementAspect::NotModeled`] 分开报：故障注入照被吞的根槽写认这一格（`fault_injection::left_by_the_lying_device`）。
    EmptyPublishKeepingTheFloorWhileTheCeilingIsHeldIsNotModeled,
}

impl ModelDisagreementAspect {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ModelDisagreementAspect::SessionState => "会话状态",
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess => "模型说该成、实现拒了",
            ModelDisagreementAspect::SucceededWhenModelRequiresRefusal => "模型说该拒、实现做成了",
            ModelDisagreementAspect::RefusalReason => "拒绝的理由",
            ModelDisagreementAspect::WroteBeforeRefusing => "拒绝之前写了盘",
            ModelDisagreementAspect::PublishCount => "写出的根的条数",
            ModelDisagreementAspect::RootIdentity => "根的 (txg, 实例)",
            ModelDisagreementAspect::JournalCounter => "jsn",
            ModelDisagreementAspect::RollbackFloor => "根带的 F",
            ModelDisagreementAspect::FilePresence => "这一版有没有文件",
            ModelDisagreementAspect::FileContent => "这一版的文件内容",
            ModelDisagreementAspect::InstanceTableOfTheVersion => "这一版的实例表",
            ModelDisagreementAspect::AllocationGeneration => "单元的分配代",
            ModelDisagreementAspect::InstanceGeneration => "取到的实例代号",
            ModelDisagreementAspect::InstanceRows => "写的实例表行",
            ModelDisagreementAspect::RollbackFloorCeiling => "抬 F 的上限",
            ModelDisagreementAspect::ColdStartReadBack => "冷启动读回",
            ModelDisagreementAspect::LiveHeadPreviousSnapshotTxg => "活头的 previous_snapshot_txg",
            ModelDisagreementAspect::Snapshots => "这一版的快照列表",
            ModelDisagreementAspect::DeadlistEntries => "这一版的 deadlist 条目（按 key 逐条）",
            ModelDisagreementAspect::PinnedMappingEntries => "这一版被钉住的映射条目数",
            ModelDisagreementAspect::SnapshotReadBack => "读快照读出的内容",
            ModelDisagreementAspect::CompactionIntentOfThePublish => "这一次整理发布对意图做的事",
            ModelDisagreementAspect::LoggedCompactionIntent => "这一版 logged_ops 里的整理意图",
            ModelDisagreementAspect::FirstFileRecordTruncationField => {
                "这一版第一个文件那条 inode 记录里的截断字段"
            }
            ModelDisagreementAspect::InodesOtherThanTheFirst => {
                "这一版第一个文件之外的 inode 与它们的内容"
            }
            ModelDisagreementAspect::EffectKindOfTheOperation => {
                "交来的做成结局不是这一种操作交的那一种"
            }
            ModelDisagreementAspect::NotModeled => "模型答不了",
            ModelDisagreementAspect::EmptyPublishKeepingTheFloorWhileTheCeilingIsHeldIsNotModeled => {
                "上限卡在挂载时读坏或读回全零的槽上推的不抬 F 的空发布（模型答不了）"
            }
        }
    }
}

/// 实现拒的时候点名的那一处读不出（D16（发布语义） 已定项 1「根槽这一次读坏」那一行；D23（journal 的角色与格式） 已定项 14「在任何写之前拒」那一格）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelReportedUnreadablePlace {
    /// 算抬 F 的上限、或挂着时回退判候选集读根环时，这个进程知道住着根的那一槽读坏、重读仍坏。
    /// 故障注入按它比「被吞的根槽写落在哪个槽」（实八）。
    RootRingSlotStillBadAfterOneReread(ModelRingPosition),
    /// 管理员回退逐盘验时回退目标引用的一个单元（复活集里的或隔离的）在一块盘上的那一份读不出或整单元 CRC-32C 对不上
    /// （执行器在回退被拒那一处经 [`ModelDisagreement::note_the_unit_copy_a_rollback_refused_on`] 记上）。
    /// 故障注入按它比「被吞的写落在哪」（`research/prompts/code-backlog-b2-code-r1-main-verification.md`「更正」一节 A4）。
    UnitCopyUnreadableOrMismatched(ModelUnitCopyLocation),
}

/// 模型与实现对不上：哪一格、模型答了什么、实现做了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelDisagreement {
    pub aspect: ModelDisagreementAspect,
    pub model_answer: String,
    pub implementation_answer: String,
    /// 对不上的是抬 F 的上限（[`ModelDisagreementAspect::RollbackFloorCeiling`]）、或推空那一格
    /// （[`ModelDisagreementAspect::EmptyPublishKeepingTheFloorWhileTheCeilingIsHeldIsNotModeled`]，这时是并进那几槽之后的上限）时实现报的那个数；
    /// 别的格都是 None。故障注入按它比「模型去掉被吞的根、并进它们留下的全零槽之后重算的上限」
    /// （实七的按注入点认；[`IdealModel::rollback_floor_ceilings_without_the_roots`]）。
    pub implementation_reported_ceiling: Option<ModelCheckpointTxg>,
    /// 对不上的是「模型说该成、实现拒了」或「拒绝的理由」，而实现拒的是一处读不出、拒之前一个写都没发时它点名的那一处
    /// （[`ModelReportedUnreadablePlace`] 的两种）；别的格都是 None。故障注入按它比被吞的写落在哪。
    pub implementation_reported_unreadable_place: Option<ModelReportedUnreadablePlace>,
    /// 对不上的是「写的实例表行」（[`ModelDisagreementAspect::InstanceRows`]）时实现写的那几行；别的格都是 None。
    /// 故障注入按它比「被吞根槽的那几条根由记录重建之后模型重算的行」（实八，D23（journal 的角色与格式） 已定项 14 第 4 条）。
    pub implementation_rows_written: Option<Vec<ModelInstanceRow>>,
}

impl ModelDisagreement {
    /// 只带哪一格与两边的话，实现报的上限、读不出的那一处、写的行都是 None。
    #[must_use]
    pub fn new(
        aspect: ModelDisagreementAspect,
        model_answer: String,
        implementation_answer: String,
    ) -> Self {
        Self {
            aspect,
            model_answer,
            implementation_answer,
            implementation_reported_ceiling: None,
            implementation_reported_unreadable_place: None,
            implementation_rows_written: None,
        }
    }

    /// 管理员回退被拒、实现点名了「回退目标引用的一个单元在一块盘上的那一份读不出或对不上」时，把那一份记上：
    /// 只在对不上的是「模型说该成、实现拒了」或「拒绝的理由」、拒之前一个写都没发时记（mount.rs 那两个成员的契约：在任何写之前返回；
    /// 拒之前写了盘的，故障注入不许拿这一份去认）。别的格不动。
    pub fn note_the_unit_copy_a_rollback_refused_on(
        &mut self,
        unit_copy: Option<ModelUnitCopyLocation>,
        refused_before_any_write: bool,
    ) {
        let aspect_is_a_refusal_the_model_did_not_expect = match self.aspect {
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
            | ModelDisagreementAspect::RefusalReason => true,
            ModelDisagreementAspect::SessionState
            | ModelDisagreementAspect::SucceededWhenModelRequiresRefusal
            | ModelDisagreementAspect::WroteBeforeRefusing
            | ModelDisagreementAspect::PublishCount
            | ModelDisagreementAspect::RootIdentity
            | ModelDisagreementAspect::JournalCounter
            | ModelDisagreementAspect::RollbackFloor
            | ModelDisagreementAspect::FilePresence
            | ModelDisagreementAspect::FileContent
            | ModelDisagreementAspect::InstanceTableOfTheVersion
            | ModelDisagreementAspect::AllocationGeneration
            | ModelDisagreementAspect::InstanceGeneration
            | ModelDisagreementAspect::InstanceRows
            | ModelDisagreementAspect::RollbackFloorCeiling
            | ModelDisagreementAspect::ColdStartReadBack
            | ModelDisagreementAspect::Snapshots
            | ModelDisagreementAspect::LiveHeadPreviousSnapshotTxg
            | ModelDisagreementAspect::DeadlistEntries
            | ModelDisagreementAspect::PinnedMappingEntries
            | ModelDisagreementAspect::SnapshotReadBack
            | ModelDisagreementAspect::LoggedCompactionIntent
            | ModelDisagreementAspect::CompactionIntentOfThePublish
            | ModelDisagreementAspect::FirstFileRecordTruncationField
            | ModelDisagreementAspect::InodesOtherThanTheFirst
            | ModelDisagreementAspect::EffectKindOfTheOperation
            | ModelDisagreementAspect::NotModeled
            | ModelDisagreementAspect::EmptyPublishKeepingTheFloorWhileTheCeilingIsHeldIsNotModeled => {
                false
            }
        };
        if aspect_is_a_refusal_the_model_did_not_expect && refused_before_any_write {
            if let Some(unit_copy) = unit_copy {
                self.implementation_reported_unreadable_place =
                    Some(ModelReportedUnreadablePlace::UnitCopyUnreadableOrMismatched(unit_copy));
            }
        }
    }
}

/// 一步判完之后的计数：模型怎么答的、比了多少格。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelJudgementCounts {
    /// 覆盖写掉电之后，盘上系统配置里的 F 正好是模型算的上限、比掉电之前的 F_生效高的次数（抬 F 那一串写完系统配置、第一条根还没落：
    /// [`IdealModel::settle_floor_left_in_the_system_configurations_by_a_lost_power`] 的 [`ModelFloorLeftByALostPower::RaisedToTheCeiling`] 那一格跑到了）。
    pub lost_powers_leaving_the_floor_at_the_ceiling: u64,
    pub required_refusals_matched: u64,
    pub permitted_refusals_taken: u64,
    pub successes_matched: u64,
    pub roots_compared: u64,
    pub allocation_records_compared: u64,
    /// 比过「这次重写了哪几个角色」的次数（树表 0 条的一版上的零单元发布：它的输出不带分配记录）。
    pub rewritten_role_sets_compared: u64,
    /// 每一版比过文件内容的次数（带文件、胶水解得出内容的那几版）。
    pub file_contents_compared: u64,
    /// 每一版比过整张实例表的次数。
    pub instance_tables_compared: u64,
    /// 实现交回的东西里没有这一版整条实例表链的字节、比不了的次数（[`ObservedInstanceTable::NotInTheOutput`]）。
    pub instance_tables_not_in_the_output: u64,
    pub cold_start_contents_compared: u64,
    pub ceilings_compared: u64,
    /// 回退做成、目标的 txg 正好等于 F_生效且 F_生效 > 0（B2 那一格跑到了）。
    pub rollbacks_accepted_at_the_effective_floor: u64,
    /// 单元区墙拒（实现报每块盘上都没有合政策的落点）、在允许拒绝的区间里而放行的次数（小盘那一段的落点拒绝真的走到了、判过）。
    pub unit_area_wall_refusals_in_the_interval: u64,
    /// 比过、对得上的「不是最后一批的删除批」（文件多于一片下段叶的 144 个单元时才有，D8（核心索引结构） 已定项 16 第 2 条；代码轮 U13）。
    pub deletion_batches_keeping_data_units_matched: u64,
    /// 会话里一次用户改动之前先走空、走了至少两步，每一步都比过、对得上的次数（执行器数，D8（核心索引结构） 已定项 16 第 6 条；
    /// N3「走空多步」那一格）。
    pub walks_of_several_steps_before_a_user_change_matched: u64,
    /// 会话里一次用户改动之前先走空时，走空的删除类发布被空间拒、推了抬 F 再试，推的那几串与走空的每一步都比过、对得上的次数
    /// （执行器数 N3「走空被空间拒」那一格）。
    pub walks_before_a_user_change_refused_for_space_matched: u64,
    /// 比过快照那几样（活头前驱、快照列表、deadlist 条目数、被钉映射条目数）的版数。
    pub snapshot_states_compared: u64,
    /// 比过的快照条数（逐版逐个加）。
    pub snapshots_compared: u64,
    /// 比过的版里 deadlist 不空的那几版（被快照钉住的单元真的走到了）。
    pub versions_with_deadlist_entries_compared: u64,
    /// 比过、对得上的读快照（读出的内容逐字节比）。
    pub snapshot_read_backs_compared: u64,
    /// 比过、对得上的删快照的批里不是最后一批的那几批（那个快照的条目多于一批 144 条时才有，判决 S4）。
    pub snapshot_destruction_batches_keeping_the_snapshot_matched: u64,
    /// 比过、对得上的整理的批（判决 G7「只比成功操作的内容」：每一批写出的那一版逐项与搬之前那一版的内容相同）。
    pub compaction_batches_matched: u64,
    /// 比过、对得上的用户改动发成之后另发的那一批（[`ModelOperationKind::PublishRelocationBatchAfterAUserChange`]）。
    pub relocation_batches_after_a_user_change_matched: u64,
    /// 比过、对得上的签发（判决 G1）。
    pub compaction_intents_issued_matched: u64,
    /// 比过、对得上的同一次发布删意图的最后一批（D8（核心索引结构） 已定项 5）。
    pub compaction_intents_deleted_with_the_last_batch_matched: u64,
    /// 比过、对得上的单独删意图（R 已按记录全空，意图达成）。
    pub compaction_intents_deleted_without_a_batch_matched: u64,
    /// 比过、对得上的丢意图。
    pub compaction_intents_dropped_matched: u64,
    /// 比过 logged_ops 里那一条活整理意图的版数（有活意图的那几版）。
    pub logged_compaction_intents_compared: u64,
    /// 比过、对得上的搬了用户可见树根单元的整理发布（这一版算一个新状态「照字节比，搬根算非空」）。
    pub compaction_publishes_relocating_a_user_visible_root_matched: u64,
    /// 比过、对得上的截断意图（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2）。
    pub truncation_intents_matched: u64,
    /// 比过、对得上的截断的批里不是最后一批的那几批（size 之外挂着的单元多于一批 144 个时才有）。
    pub truncation_batches_before_the_final_batch_matched: u64,
    /// 比过、对得上的截断最后一批。
    pub final_truncation_batches_matched: u64,
    /// 比过、对得上的不缩短的截断（变长与 N = S，K5、K12）。
    pub size_changes_without_shrinking_matched: u64,
    /// 比过第一个文件那条 inode 记录里截断字段的版数（有那条记录的那几版，判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R2）。
    pub truncation_fields_compared: u64,
    /// 其中模型记的截断字段写过（T ≠ 0）的版数。
    pub written_truncation_fields_compared: u64,
    /// 比过的第一个文件之外的 inode（逐版逐个加，里程碑「删除与多次 COW」一·1）。
    pub inodes_other_than_the_first_compared: u64,
    /// 其中有数据单元、逐字节比过内容的（逐版逐个加）。
    pub contents_of_files_other_than_the_first_compared: u64,
    /// 比过、对得上的按 inode 号写第一个文件之外的文件。
    pub writes_of_an_inode_other_than_the_first_matched: u64,
    /// 其中这次让 extent 树上段的根升了一层的（写的号罩不进原来的根）。
    pub writes_growing_the_extent_upper_segment_matched: u64,
    /// 其中写的号落在上段第 0 片叶之外的（号不小于一片叶罩的 143 个）。
    pub writes_past_the_first_extent_upper_leaf_matched: u64,
    /// 比过、对得上的「删完第一个文件之后给建的 inode 写数据」那一拒（K-U5）。
    pub data_writes_refused_after_the_first_file_was_deleted_matched: u64,
    /// 比过、对得上的按偏移写第一个文件（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节）。
    pub partial_writes_of_the_first_file_matched: u64,
    /// 比过、对得上的按偏移写第一个文件之外的文件（第二节 R6）。
    pub partial_writes_of_an_inode_other_than_the_first_matched: u64,
    /// 比过、对得上的删第一个文件之外的号的最后一批（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M4「删别的号」）。
    pub deletions_of_an_inode_other_than_the_first_matched: u64,
    /// 其中接在第一个文件已删完的一版后面的（代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 3 条：
    /// 随机历史在第一个文件删完之后也按号删别的文件）。
    pub deletions_of_an_inode_other_than_the_first_after_the_first_file_was_deleted_matched: u64,
    /// 比过、对得上的删第一个文件的最后一批、之后树里还有别的 inode 的（第三节 M4「删第一个文件而别的还在」，这一版归带文件的一版）。
    pub deletions_of_the_first_file_beside_other_inodes_matched: u64,
    /// 比过、对得上的删到文件集合为空的最后一批（第三节 M4「删到空」：之后是有树而文件集合为空的一版）。
    pub final_deletion_batches_emptying_the_version_matched: u64,
    /// 比过、对得上的让 extent 上段根降了层的最后一批（第三节 M4「上段降层」：上一版里降后层级的第 0 个节点照抄成根）。
    pub deletions_lowering_the_extent_upper_segment_matched: u64,
    /// 比过第一个文件之外的记录里截断字段的条数（逐版逐条加，判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5）。
    pub truncation_fields_of_the_inodes_other_than_the_first_compared: u64,
    /// 其中模型记的截断字段写过（T ≠ 0）的条数。
    pub written_truncation_fields_of_the_inodes_other_than_the_first_compared: u64,
    /// 比过、对得上的带数据单元的别的 inode 改 size 那一次（截断意图与不缩短的那一次，第二节 S3）。
    pub size_changes_of_inodes_other_than_the_first_holding_data_units_matched: u64,
    /// 比过、对得上的带数据单元的别的 inode 截断的批与最后一批。
    pub truncation_steps_of_inodes_other_than_the_first_holding_data_units_matched: u64,
    /// 上面两样里接在有快照的一版后面的（快照下截断与变长）。
    pub size_changes_of_inodes_other_than_the_first_under_a_snapshot_matched: u64,
    /// 上面两样里接在 logged_ops 里有活整理意图的一版后面的（整理时截断）。
    pub size_changes_of_inodes_other_than_the_first_beside_a_live_compaction_intent_matched: u64,
    /// 比过、对得上的建快照里接在别的文件带数据单元的一版后面的。
    pub snapshots_created_beside_files_other_than_the_first_holding_data_units_matched: u64,
    /// 比过、对得上的整理的签发与批里接在别的文件带数据单元的一版后面的。
    pub compaction_publishes_beside_files_other_than_the_first_holding_data_units_matched: u64,
}

impl ModelJudgementCounts {
    /// 把另一份逐项加进来：按字段拆开写全，新加一个字段而这里没加，编译不过。
    pub fn add(&mut self, other: &ModelJudgementCounts) {
        let ModelJudgementCounts {
            lost_powers_leaving_the_floor_at_the_ceiling,
            required_refusals_matched,
            permitted_refusals_taken,
            successes_matched,
            roots_compared,
            allocation_records_compared,
            rewritten_role_sets_compared,
            file_contents_compared,
            instance_tables_compared,
            instance_tables_not_in_the_output,
            cold_start_contents_compared,
            ceilings_compared,
            rollbacks_accepted_at_the_effective_floor,
            unit_area_wall_refusals_in_the_interval,
            deletion_batches_keeping_data_units_matched,
            walks_of_several_steps_before_a_user_change_matched,
            walks_before_a_user_change_refused_for_space_matched,
            snapshot_states_compared,
            snapshots_compared,
            versions_with_deadlist_entries_compared,
            snapshot_read_backs_compared,
            snapshot_destruction_batches_keeping_the_snapshot_matched,
            compaction_batches_matched,
            relocation_batches_after_a_user_change_matched,
            compaction_intents_issued_matched,
            compaction_intents_deleted_with_the_last_batch_matched,
            compaction_intents_deleted_without_a_batch_matched,
            compaction_intents_dropped_matched,
            logged_compaction_intents_compared,
            compaction_publishes_relocating_a_user_visible_root_matched,
            truncation_intents_matched,
            truncation_batches_before_the_final_batch_matched,
            final_truncation_batches_matched,
            size_changes_without_shrinking_matched,
            truncation_fields_compared,
            written_truncation_fields_compared,
            inodes_other_than_the_first_compared,
            contents_of_files_other_than_the_first_compared,
            writes_of_an_inode_other_than_the_first_matched,
            writes_growing_the_extent_upper_segment_matched,
            writes_past_the_first_extent_upper_leaf_matched,
            data_writes_refused_after_the_first_file_was_deleted_matched,
            partial_writes_of_the_first_file_matched,
            partial_writes_of_an_inode_other_than_the_first_matched,
            deletions_of_an_inode_other_than_the_first_matched,
            deletions_of_an_inode_other_than_the_first_after_the_first_file_was_deleted_matched,
            deletions_of_the_first_file_beside_other_inodes_matched,
            final_deletion_batches_emptying_the_version_matched,
            deletions_lowering_the_extent_upper_segment_matched,
            truncation_fields_of_the_inodes_other_than_the_first_compared,
            written_truncation_fields_of_the_inodes_other_than_the_first_compared,
            size_changes_of_inodes_other_than_the_first_holding_data_units_matched,
            truncation_steps_of_inodes_other_than_the_first_holding_data_units_matched,
            size_changes_of_inodes_other_than_the_first_under_a_snapshot_matched,
            size_changes_of_inodes_other_than_the_first_beside_a_live_compaction_intent_matched,
            snapshots_created_beside_files_other_than_the_first_holding_data_units_matched,
            compaction_publishes_beside_files_other_than_the_first_holding_data_units_matched,
        } = other;
        self.required_refusals_matched += required_refusals_matched;
        self.permitted_refusals_taken += permitted_refusals_taken;
        self.successes_matched += successes_matched;
        self.roots_compared += roots_compared;
        self.allocation_records_compared += allocation_records_compared;
        self.rewritten_role_sets_compared += rewritten_role_sets_compared;
        self.file_contents_compared += file_contents_compared;
        self.instance_tables_compared += instance_tables_compared;
        self.instance_tables_not_in_the_output += instance_tables_not_in_the_output;
        self.cold_start_contents_compared += cold_start_contents_compared;
        self.ceilings_compared += ceilings_compared;
        self.rollbacks_accepted_at_the_effective_floor += rollbacks_accepted_at_the_effective_floor;
        self.lost_powers_leaving_the_floor_at_the_ceiling +=
            lost_powers_leaving_the_floor_at_the_ceiling;
        self.unit_area_wall_refusals_in_the_interval += unit_area_wall_refusals_in_the_interval;
        self.deletion_batches_keeping_data_units_matched +=
            deletion_batches_keeping_data_units_matched;
        self.walks_of_several_steps_before_a_user_change_matched +=
            walks_of_several_steps_before_a_user_change_matched;
        self.walks_before_a_user_change_refused_for_space_matched +=
            walks_before_a_user_change_refused_for_space_matched;
        self.snapshot_states_compared += snapshot_states_compared;
        self.snapshots_compared += snapshots_compared;
        self.versions_with_deadlist_entries_compared += versions_with_deadlist_entries_compared;
        self.snapshot_read_backs_compared += snapshot_read_backs_compared;
        self.snapshot_destruction_batches_keeping_the_snapshot_matched +=
            snapshot_destruction_batches_keeping_the_snapshot_matched;
        self.compaction_batches_matched += compaction_batches_matched;
        self.relocation_batches_after_a_user_change_matched +=
            relocation_batches_after_a_user_change_matched;
        self.compaction_intents_issued_matched += compaction_intents_issued_matched;
        self.compaction_intents_deleted_with_the_last_batch_matched +=
            compaction_intents_deleted_with_the_last_batch_matched;
        self.compaction_intents_deleted_without_a_batch_matched +=
            compaction_intents_deleted_without_a_batch_matched;
        self.compaction_intents_dropped_matched += compaction_intents_dropped_matched;
        self.logged_compaction_intents_compared += logged_compaction_intents_compared;
        self.compaction_publishes_relocating_a_user_visible_root_matched +=
            compaction_publishes_relocating_a_user_visible_root_matched;
        self.truncation_intents_matched += truncation_intents_matched;
        self.truncation_batches_before_the_final_batch_matched +=
            truncation_batches_before_the_final_batch_matched;
        self.final_truncation_batches_matched += final_truncation_batches_matched;
        self.size_changes_without_shrinking_matched += size_changes_without_shrinking_matched;
        self.truncation_fields_compared += truncation_fields_compared;
        self.written_truncation_fields_compared += written_truncation_fields_compared;
        self.inodes_other_than_the_first_compared += inodes_other_than_the_first_compared;
        self.contents_of_files_other_than_the_first_compared +=
            contents_of_files_other_than_the_first_compared;
        self.writes_of_an_inode_other_than_the_first_matched +=
            writes_of_an_inode_other_than_the_first_matched;
        self.writes_growing_the_extent_upper_segment_matched +=
            writes_growing_the_extent_upper_segment_matched;
        self.writes_past_the_first_extent_upper_leaf_matched +=
            writes_past_the_first_extent_upper_leaf_matched;
        self.data_writes_refused_after_the_first_file_was_deleted_matched +=
            data_writes_refused_after_the_first_file_was_deleted_matched;
        self.partial_writes_of_the_first_file_matched += partial_writes_of_the_first_file_matched;
        self.partial_writes_of_an_inode_other_than_the_first_matched +=
            partial_writes_of_an_inode_other_than_the_first_matched;
        self.deletions_of_an_inode_other_than_the_first_matched +=
            deletions_of_an_inode_other_than_the_first_matched;
        self.deletions_of_an_inode_other_than_the_first_after_the_first_file_was_deleted_matched +=
            deletions_of_an_inode_other_than_the_first_after_the_first_file_was_deleted_matched;
        self.deletions_of_the_first_file_beside_other_inodes_matched +=
            deletions_of_the_first_file_beside_other_inodes_matched;
        self.final_deletion_batches_emptying_the_version_matched +=
            final_deletion_batches_emptying_the_version_matched;
        self.deletions_lowering_the_extent_upper_segment_matched +=
            deletions_lowering_the_extent_upper_segment_matched;
        self.truncation_fields_of_the_inodes_other_than_the_first_compared +=
            truncation_fields_of_the_inodes_other_than_the_first_compared;
        self.written_truncation_fields_of_the_inodes_other_than_the_first_compared +=
            written_truncation_fields_of_the_inodes_other_than_the_first_compared;
        self.size_changes_of_inodes_other_than_the_first_holding_data_units_matched +=
            size_changes_of_inodes_other_than_the_first_holding_data_units_matched;
        self.truncation_steps_of_inodes_other_than_the_first_holding_data_units_matched +=
            truncation_steps_of_inodes_other_than_the_first_holding_data_units_matched;
        self.size_changes_of_inodes_other_than_the_first_under_a_snapshot_matched +=
            size_changes_of_inodes_other_than_the_first_under_a_snapshot_matched;
        self.size_changes_of_inodes_other_than_the_first_beside_a_live_compaction_intent_matched +=
            size_changes_of_inodes_other_than_the_first_beside_a_live_compaction_intent_matched;
        self.snapshots_created_beside_files_other_than_the_first_holding_data_units_matched +=
            snapshots_created_beside_files_other_than_the_first_holding_data_units_matched;
        self.compaction_publishes_beside_files_other_than_the_first_holding_data_units_matched +=
            compaction_publishes_beside_files_other_than_the_first_holding_data_units_matched;
    }
}
