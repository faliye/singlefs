//! 一步的结局与 harness 自己的判定：缺的前提、分配记录的复用与隔离标记、起点那几步、准入与读回、施加的效果。

use super::*;

/// 一步操作的前提此刻不成立，入口没调。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MissingPrecondition {
    /// 这个进程里没有可写会话（还没挂载、挂载被拒、冷启动之后）。
    NoWritableSession,
    /// 现行版本树表 0 条：覆盖写与抬 F 都要一个带文件的上一版（`TransactionOutput`）。
    CurrentVersionWithoutFile,
    /// 现行版本带文件：零单元发布只给树表 0 条的一版；带文件的一版上的空发布要重写固定点单元（分配记录树要改的那几个节点 + 记账 + 映射 + 树表），那是挂载内部的 `publish_version`，
    /// 不是这个入口。生成器的前提，没有条款明写调用约定，按入口的文档注释取的读法——`crates/singlefs-core/src/transaction.rs` 的 `publish_without_units`
    /// 文档注释「零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）」，已定项 9 在 `.claude/kb/decisions/16-发布语义.md`「已定项 9：一次空发布也写单元」。
    /// 以后要测：前提之外调它应当返回 `Err`，不许 panic（增补 3 记着）。
    CurrentVersionWithFile,
    /// 按 checker 的读法，根环里一条可读根都没有：回退的目标无从取。
    NoReadableRootInRing,
    /// 崩溃恢复抛弃根：按 checker 的读法，根环里最新那条根不是这个会话的现行版本（例如那一版的根槽写被吞了），
    /// 要藏起来的根槽与点名单元从会话里取不到。
    NewestRingRootIsNotTheCurrentVersionOfThisSession,
    /// 崩溃恢复抛弃根：根环里除了最新那条之外一条可读根都没有，择根落不到前一条根上。
    NoReadableRootBeforeTheNewest,
    /// 崩溃恢复抛弃根：最新那次发布的记录一个单元都没点名（树表 0 条上的零单元发布）——点名验证恒过，恢复照样施加它、
    /// 走到的就是它，抛弃不了。
    NewestPublishNamesNoUnit,
    /// 现行版本有树、第一个文件已删完（在删完的一版上建过 inode 之后，或删第一个文件时树里还有别的 inode，实现归带文件的一版）：覆盖写只写第一个文件
    /// （`transaction::live_record_of_the_first_file` 拒成 `WriteToAnInodeThatIsNotInTheTree`），执行器不调；unlink 按号选在第一个文件那条记录还在时调，
    /// 已删完时只在还有带数据单元的别的文件活着时调（别的几组比重不写别的文件的数据，在这一形上一向不调，执行照旧）。
    CurrentVersionWithoutTheFirstFile,
    /// 现行版本里第一个文件还活着：建 inode 只接在第一个文件已删完的一版上（`HistoryOperation::CreateInode` 的文档）；
    /// 删除中的那一版照调，会话先走空、把它走成删完的一版（代码轮 U11）。
    CurrentVersionStillHasTheFirstFile,
    /// 每块盘最新那一槽系统配置读回全 0 的可写挂载：有一块盘两槽里本池自证过的不到两份（mkfs 之后还没取过号的盘只写过一槽）——
    /// 再藏它自证过的那一槽，这块盘一份自证过的系统配置都不剩，那是那块盘两槽都坏，不是单故障（D22（单元原子性怎么合成） 已定项 8）。
    SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots,
    /// 读快照、删快照：现行那一版树表里没有不在销毁中的快照可点名。
    NoSnapshotOutsideTheDestructionInTheCurrentVersion,
    /// 挂着时整理：现行那一版里没有能强制选的段（[`segments_the_compaction_may_be_forced_onto`] 一段都没有）。
    NoSegmentForTheCompactionInTheCurrentVersion,
    /// unlink：现行那一版 inode 树多于一片叶容器。实现照删（删一条之后照 D8（核心索引结构） 已定项 6 吸收或借），模型的删除那一臂只答
    /// 一片叶容器的一版（`IdealModel::answer_unlink_intent_of_an_inode`：删一条不并、不借）——执行器不调。
    InodeTreeSpansMoreThanOneLeafContainer,
    /// 按 inode 号写：现行那一版 inode 树里一个活着的号都没有（有树而文件集合为空）。
    NoInodeToWriteInTheCurrentVersion,
}

impl MissingPrecondition {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            MissingPrecondition::NoWritableSession => "没有可写会话",
            MissingPrecondition::CurrentVersionWithoutFile => "现行版本树表 0 条",
            MissingPrecondition::CurrentVersionWithFile => "现行版本带文件",
            MissingPrecondition::NoReadableRootInRing => "根环里没有可读根",
            MissingPrecondition::NewestRingRootIsNotTheCurrentVersionOfThisSession => {
                "根环里最新那条根不是这个会话的现行版本"
            }
            MissingPrecondition::NoReadableRootBeforeTheNewest => "最新那条根之前没有可读根",
            MissingPrecondition::NewestPublishNamesNoUnit => "最新那次发布一个单元都没点名",
            MissingPrecondition::CurrentVersionWithoutTheFirstFile => "现行版本里第一个文件已删完",
            MissingPrecondition::CurrentVersionStillHasTheFirstFile => "现行版本里第一个文件还在",
            MissingPrecondition::SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots => {
                "有盘两槽系统配置不是都自证过"
            }
            MissingPrecondition::NoSnapshotOutsideTheDestructionInTheCurrentVersion => {
                "现行那一版没有能点名的快照"
            }
            MissingPrecondition::NoSegmentForTheCompactionInTheCurrentVersion => {
                "现行那一版没有能强制选的整理段"
            }
            MissingPrecondition::InodeTreeSpansMoreThanOneLeafContainer => {
                "现行那一版 inode 树多于一片叶容器"
            }
            MissingPrecondition::NoInodeToWriteInTheCurrentVersion => {
                "现行那一版树里没有活着的 inode"
            }
        }
    }
}

/// 一次发布前后分配记录的变化：复用已回收的记录（同盘同槽那条从已释放改回已分配）、复用时跨度变了、已释放的记录被删掉、
/// 已释放的记录被这次新分配或改写的记录罩住起点（第 121 行那一类的路径：罩住了就该删，删没删另算）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecordReuse {
    pub rewritten_from_released: u64,
    pub rewritten_with_changed_span: u64,
    pub released_records_removed: u64,
    pub released_records_covered: u64,
}

impl RecordReuse {
    /// 两份逐项相加（一次挂载写出的几次发布各比一次，加起来算这次挂载的）。
    #[must_use]
    pub fn plus(self, other: RecordReuse) -> RecordReuse {
        RecordReuse {
            rewritten_from_released: self.rewritten_from_released + other.rewritten_from_released,
            rewritten_with_changed_span: self.rewritten_with_changed_span
                + other.rewritten_with_changed_span,
            released_records_removed: self.released_records_removed
                + other.released_records_removed,
            released_records_covered: self.released_records_covered
                + other.released_records_covered,
        }
    }

    pub(super) fn between(before: &[AllocationRecord], after: &[AllocationRecord]) -> Self {
        let released_before: BTreeMap<(DeviceIdentity, u64), &AllocationRecord> = before
            .iter()
            .filter(|record| record.is_released)
            .map(|record| ((record.device, record.slot.0), record))
            .collect();
        let places_after: BTreeSet<(DeviceIdentity, u64)> = after
            .iter()
            .map(|record| (record.device, record.slot.0))
            .collect();
        let mut reuse = RecordReuse::default();
        for record_after in after.iter().filter(|record| !record.is_released) {
            if let Some(record_before) =
                released_before.get(&(record_after.device, record_after.slot.0))
            {
                reuse.rewritten_from_released += 1;
                if record_before.span_slots != record_after.span_slots {
                    reuse.rewritten_with_changed_span += 1;
                }
            }
        }
        reuse.released_records_removed = u64::try_from(
            released_before
                .keys()
                .filter(|place| !places_after.contains(place))
                .count(),
        )
        .expect("条数");
        let records_before: BTreeSet<&AllocationRecord> = before.iter().collect();
        let allocated_by_this_publish: Vec<&AllocationRecord> = after
            .iter()
            .filter(|record| !record.is_released && !records_before.contains(record))
            .collect();
        reuse.released_records_covered = u64::try_from(
            released_before
                .values()
                .filter(|released| {
                    allocated_by_this_publish.iter().any(|allocated| {
                        allocated.device == released.device
                            && allocated.slot.0 < released.slot.0
                            && released.slot.0 < allocated.slot.0 + u64::from(allocated.span_slots)
                    })
                })
                .count(),
        )
        .expect("条数");
        reuse
    }
}

/// 一次（几次）发布里，执行器认下的「只改了隔离记号」的记录改法（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」：
/// 隔离那次发布置记号、分配代不变；回退复活那一次，同一次发布清掉记号）。认下的记录与发布之前同一盘同一槽上那一条跨度相同、分配代相同、
/// 前后都未释放，只差记号。
#[derive(Clone, Copy, Debug)]
pub struct QuarantineMarkChangesVouchedFor<'copies> {
    /// 这次（几次）发布里实现交回的隔离：没有记号改成带记号的记录，要在这里有同一块盘、同一个起点槽、同一个跨度的那一份。
    pub copies_quarantined_by_the_publishes:
        &'copies [CopyQuarantinedAfterReleaseChecksumMismatch],
    /// 带记号的记录改回没有记号，是不是这次发布该做的。
    pub clearing: QuarantineMarkClearing,
}

/// 带隔离记号的记录改回没有记号（分配代不变、仍未释放），是不是这次发布该做的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuarantineMarkClearing {
    /// 不是回退那次发布：清了记号的记录照挑。
    NotDoneByThisPublish,
    /// 回退那次发布：回退目标引用的单元，同一次发布清掉两块盘上的记号。
    DoneByTheRollbackPublish,
}

/// `record`（发布之后）与发布之前同一盘同一槽上的一条只差隔离记号（跨度、分配代相同，前后都未释放），而且这一改是 `vouched_for` 认下的。
fn is_a_quarantine_mark_change_vouched_for(
    before: &[AllocationRecord],
    record: &AllocationRecord,
    vouched_for: QuarantineMarkChangesVouchedFor<'_>,
) -> bool {
    before.iter().any(|record_before| {
        let differs_only_in_the_quarantine_mark = record_before.device == record.device
            && record_before.slot == record.slot
            && record_before.span_slots == record.span_slots
            && record_before.generation == record.generation
            && !record_before.is_released
            && !record.is_released;
        if !differs_only_in_the_quarantine_mark {
            return false;
        }
        match (record_before.quarantine_mark, record.quarantine_mark) {
            (
                AllocationRecordQuarantineMark::NotQuarantined,
                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
            ) => vouched_for
                .copies_quarantined_by_the_publishes
                .iter()
                .any(|copy| {
                    copy.device == record.device
                        && copy.placement.slot == record.slot
                        && copy.placement.span == u64::from(record.span_slots)
                }),
            (
                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
                AllocationRecordQuarantineMark::NotQuarantined,
            ) => match vouched_for.clearing {
                QuarantineMarkClearing::NotDoneByThisPublish => false,
                QuarantineMarkClearing::DoneByTheRollbackPublish => true,
            },
            // 记号也相同就是一模一样的一条，不是改写。
            (
                AllocationRecordQuarantineMark::NotQuarantined,
                AllocationRecordQuarantineMark::NotQuarantined,
            )
            | (
                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
            ) => false,
        }
    })
}

/// 一次发布改写或新增的分配记录（发布之后有、发布之前没有一模一样的一条）里，代不在 [`first_publish_txg`, `last_publish_txg`] 里的那些，
/// 去掉 `quarantine_mark_changes` 认下的只改隔离记号的那几条。
/// 一次发布改写的记录有五种。三种改代，代都是这次发布的 txg：新分配的（分配代）、复用改写已回收的（分配代改成这次的，D3（空间分配） 的
/// 「value = 分配代」）、这次释放的（释放代，`crates/singlefs-core/src/transaction.rs` 的 `allocator.release(*placement, txg);`；
/// 释放隔离记录时记号随之清掉，也是这一种）。两种只改隔离记号、分配代不变（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」）：
/// 隔离那次发布置记号，回退复活那一次清记号。这两种不在通用的判法里一并放过：置记号的只放过这次实现交回了隔离的那几份，清记号的只在
/// 回退那一处放过（[`QuarantineMarkChangesVouchedFor`]），别的只差记号的记录代不在这次的 txg 里照挑。
/// 盘上 checker 判不出分配代没改（33 条不变量里只有 I-3.9 看代，只看已释放的；增补 2 收口表第 44 行），这里在执行器里判。
#[must_use]
pub fn records_changed_with_a_generation_outside_the_publish_txgs(
    before: &[AllocationRecord],
    after: &[AllocationRecord],
    first_publish_txg: CheckpointTxg,
    last_publish_txg: CheckpointTxg,
    quarantine_mark_changes: QuarantineMarkChangesVouchedFor<'_>,
) -> Vec<AllocationRecord> {
    let records_before: BTreeSet<&AllocationRecord> = before.iter().collect();
    after
        .iter()
        .filter(|record| !records_before.contains(record))
        .filter(|record| {
            record.generation < first_publish_txg || record.generation > last_publish_txg
        })
        .filter(|record| {
            !is_a_quarantine_mark_change_vouched_for(before, record, quarantine_mark_changes)
        })
        .copied()
        .collect()
}

/// 执行器自己判的失败：不是 checker 的不变量、不是 panic，是入口交回的东西或结局自己就说明错了。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HarnessJudgement {
    /// 这一步发布改写或新增的分配记录，代不是这次发布的 txg（`records_changed_with_a_generation_outside_the_publish_txgs`）。
    AllocationGenerationIsNotThePublishTxg {
        records: Vec<AllocationRecord>,
        first_publish_txg: CheckpointTxg,
        last_publish_txg: CheckpointTxg,
    },
    /// 冷启动恢复报错，而这一步之前的镜像 checker 判过绿（判红的话历史已经停了；冷启动不写盘，镜像没变）。
    ColdStartRecoveryFailedOnCheckerGreenImage { outcome: String },
    /// 起点段（mkfs、取号、暖机、第一个文件）里有一个入口返回了错误。不注入故障时走不到：两块内存盘的读写不报错。
    /// 故障注入（增补 3 第 4 件）把注入点摆进起点段之后走得到，判定归那一路（`fault_injection::the_fault_surfaced_as_an_error`）。
    TheStartingPointReturnedAnError {
        step: StartingPointStep,
        error: String,
    },
}

impl HarnessJudgement {
    /// 签名与报告里的名字。
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            HarnessJudgement::AllocationGenerationIsNotThePublishTxg { .. } => {
                "改写或新增的分配记录的代不是这次发布的 txg"
            }
            HarnessJudgement::ColdStartRecoveryFailedOnCheckerGreenImage { .. } => {
                "checker 判绿的镜像上冷启动恢复报错"
            }
            HarnessJudgement::TheStartingPointReturnedAnError { .. } => "起点段的入口返回了错误",
        }
    }
}

/// 起点段是哪一步：mkfs、取号、暖机、第一个文件。故障注入把注入点摆进起点段之后这四步都可能返回块设备错，
/// 执行器把错交回判定而不是 panic（增补 3 第 4 件代码三方改法二）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StartingPointStep {
    MakeFilesystem,
    /// mkfs 之后、取号之前，给这个进程的会话读盘上那份参数与盘表（`mount::ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`）。
    ReadThePoolForTheSession,
    AcquireInstance,
    WarmUp,
    PublishFirstFile,
}

impl StartingPointStep {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            StartingPointStep::MakeFilesystem => "mkfs",
            StartingPointStep::ReadThePoolForTheSession => "给会话读盘上的参数与盘表",
            StartingPointStep::AcquireInstance => "取号",
            StartingPointStep::WarmUp => "暖机",
            StartingPointStep::PublishFirstFile => "第一个文件",
        }
    }

    /// 这一步报错时，盘上有没有一个 mkfs 做完了的池。mkfs 自己就报错的那一格没有：那份盘面上没有文件系统，
    /// 模型一版都没提交过，池级 checker 与「重开要恢复到模型允许的版本」都没有可判的对象。
    #[must_use]
    pub const fn finished_filesystem_is_on_the_devices(self) -> bool {
        match self {
            StartingPointStep::MakeFilesystem => false,
            StartingPointStep::ReadThePoolForTheSession
            | StartingPointStep::AcquireInstance
            | StartingPointStep::WarmUp
            | StartingPointStep::PublishFirstFile => true,
        }
    }
}

/// 起点段停在哪一步、报的什么错，加上那一刻两块盘的整份镜像（起点段失败时池还没建起来，调用方拿不到 `HistoryPool`）。
pub struct StartingPointFailure {
    pub step: StartingPointStep,
    /// 那一步交回的错误成员，`{:?}` 出来的一行。
    pub error: String,
    pub image: MemoryPool,
    /// 录制流头上 mkfs 写的步数（mkfs 自己报错时就是这一刻录制流里的全部步数）。
    pub operations_written_by_make_filesystem: usize,
}

/// 一步操作的结局本身判不判失败：冷启动读回报错就算（不看读回的内容对不对，那是第 2 件模型的事）。
#[must_use]
pub fn harness_judgement_of_outcome(outcome: &StepOutcome) -> Option<HarnessJudgement> {
    match outcome {
        StepOutcome::Applied(AppliedEffect::Recovered { outcome, read_back }) => match read_back {
            ColdStartReadBack::Failed => Some(
                HarnessJudgement::ColdStartRecoveryFailedOnCheckerGreenImage {
                    outcome: outcome.clone(),
                },
            ),
            ColdStartReadBack::NoFile | ColdStartReadBack::FileRead => None,
        },
        StepOutcome::Applied(
            AppliedEffect::Published { .. }
            | AppliedEffect::Mounted { .. }
            | AppliedEffect::RaisedFloor { .. }
            | AppliedEffect::RolledBack { .. }
            | AppliedEffect::SnapshotPublished { .. }
            | AppliedEffect::SnapshotRead { .. }
            | AppliedEffect::Compacted { .. },
        )
        | StepOutcome::Refused { .. }
        | StepOutcome::NotApplicable(_) => None,
    }
}

/// 一次可写挂载的空间准入怎么判的（`singlefs_core::mount::MountSpaceAdmission` 的成员，只带写行之后推了几串抬 F）：
/// 挂载那一处的推（D16（发布语义） 已定项 1「准入」那一行）真的跑到了没有，计数里看这个。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountSpaceAdmissionOutcome {
    AdmittedBeforeAcquisition,
    AdmittedAfterTheFloorRaises { floor_raises: usize },
    StillShortAfterTheFloorRaises { floor_raises: usize },
    NotJudgedByTheTestOnlySwitch,
}

impl MountSpaceAdmissionOutcome {
    #[must_use]
    pub fn of(space_admission: &singlefs_core::mount::MountSpaceAdmission) -> Self {
        use singlefs_core::mount::MountSpaceAdmission;
        match space_admission {
            MountSpaceAdmission::AdmittedBeforeAcquisition => {
                MountSpaceAdmissionOutcome::AdmittedBeforeAcquisition
            }
            MountSpaceAdmission::AdmittedAfterTheFloorRaises { floor_raises, .. } => {
                MountSpaceAdmissionOutcome::AdmittedAfterTheFloorRaises {
                    floor_raises: floor_raises.len(),
                }
            }
            MountSpaceAdmission::StillShortAfterTheFloorRaises { floor_raises, .. } => {
                MountSpaceAdmissionOutcome::StillShortAfterTheFloorRaises {
                    floor_raises: floor_raises.len(),
                }
            }
            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => {
                MountSpaceAdmissionOutcome::NotJudgedByTheTestOnlySwitch
            }
        }
    }

    /// 计数里的名字（不带推了几串）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            MountSpaceAdmissionOutcome::AdmittedBeforeAcquisition => "取号之前就够",
            MountSpaceAdmissionOutcome::AdmittedAfterTheFloorRaises { .. } => {
                "写行之后推抬 F 再判够了"
            }
            MountSpaceAdmissionOutcome::StillShortAfterTheFloorRaises { .. } => {
                "写行之后推抬 F 推满仍不够"
            }
            MountSpaceAdmissionOutcome::NotJudgedByTheTestOnlySwitch => "只供测试的开关关掉准入",
        }
    }
}

/// 冷启动读回的三种结局（`recovery::RecoveryOutcome` 的成员，不带内容）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColdStartReadBack {
    NoFile,
    FileRead,
    Failed,
}

/// 入口返回 Ok 之后这一步做成了什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppliedEffect {
    Published {
        reuse: RecordReuse,
        /// 这次发布之前挂着的会话推了几串抬 F（只有经会话的覆盖写与 unlink 会推，D16（发布语义） 已定项 1「准入」那一行、C283；
        /// unlink 连同走空里删除类发布被空间拒时推的 Q-V；第一个文件与零单元发布不经会话，恒为 0）。
        floor_raises_pushed_by_the_session: usize,
        /// 这一步写的根里，从这次改动自己那一次发布起（连同之后走空的各步与其间推的抬 F）有几条，按写的先后是最后这么多条：
        /// 它们读出的都是改动之后的内容，之前的（会话先推的抬 F、先走空的各步）读出改动之前的。一次发布做完的改动恒 1；截断是意图、
        /// 各批、最后一批与其间推的抬 F（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2「先落目标」：意图那一版起读出的
        /// 就是截后的）。RefFS 翻译按它给这一步的根映版本。
        roots_carrying_the_change: usize,
    },
    Mounted {
        instance: InstanceGeneration,
        publishes: usize,
        allocation_records_compared: MountAllocationComparison,
        /// 写行与暖机那几次发布里的复用（逐次相加）。
        reuse: RecordReuse,
        /// 这次挂载的空间准入怎么判的（`singlefs_core::mount::MountSpaceAdmission` 去掉细节）。
        space_admission: MountSpaceAdmissionOutcome,
    },
    RaisedFloor {
        new_floor: CheckpointTxg,
        publishes: usize,
        reclaimed_placements: usize,
        reuse: RecordReuse,
    },
    /// 管理员回退做成：一次向前发布，释放与复活的用户可见单元各几个（复活按盘逐份数）。
    RolledBack {
        target: RollbackTarget,
        released_user_visible_units: usize,
        resurrected_user_visible_copies: usize,
    },
    Recovered {
        outcome: String,
        read_back: ColdStartReadBack,
    },
    /// 建快照或删快照经会话做成（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2、S4）：`snapshot` 是那个快照码 14 条目的
    /// 树号（快照按它点名，第七节岔路 5；RefFS 拿它当快照名）。
    SnapshotPublished {
        change: SnapshotChange,
        snapshot: u64,
        reuse: RecordReuse,
        floor_raises_pushed_by_the_session: usize,
    },
    /// 读了一个快照：它码 14 条目的树号与读出了什么（模型比过）。
    SnapshotRead {
        snapshot: u64,
        read_back: ObservedSnapshotReadBack,
    },
    /// 挂着时整理了一趟（[`HistoryOperation::CompactWhileMounted`]，每一批模型比过）：发了几批、搬了几个单元、停在哪
    /// （`compaction_plan::CompactionRunStop::name`）、停下时意图还活不活着；批间崩溃那一种之后那次可写挂载也做成了
    /// （挂载被拒时这一步的结局是那个拒绝）。
    Compacted {
        batches_published: u64,
        units_relocated: u64,
        stop: &'static str,
        is_the_intent_live_when_the_run_stopped: bool,
        crashed_then_mounted: bool,
    },
}

/// 一次快照改动是建还是删。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotChange {
    Created,
    Destroyed,
}

/// 一步操作的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepOutcome {
    /// 入口返回 Ok。
    Applied(AppliedEffect),
    /// 入口返回 Err：合法结局，带成员名（嵌套的连里层一起写）。
    Refused { member: String },
    /// 前提此刻不成立，入口没调。
    NotApplicable(MissingPrecondition),
}

/// 走到哪一步了：起点（mkfs，或 mkfs 加第一个文件）还是第几步操作（从 0 数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StepPosition {
    StartingPoint,
    Operation(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一次发布（txg 12）改写或新增的记录，代都要是 12：新分配的、复用改写已回收的、这次释放的都对就一条都不报；复用改写时代没改
    /// （还写着释放代 5，代码三方攻方的 N2）就报那一条。没动过的记录（代 3、代 0）不看。
    #[test]
    fn record_rewritten_by_a_publish_that_keeps_its_old_generation_is_reported() {
        let before = vec![
            record(50176, 2, 0, false),
            record(50178, 1, 5, true),
            record(50180, 2, 3, false),
            record(50182, 1, 11, false),
        ];
        let rewritten_and_new_and_released = vec![
            record(50176, 2, 0, false),
            record(50178, 1, 12, false),
            record(50180, 2, 3, false),
            record(50182, 1, 12, true),
            record(50184, 2, 12, false),
        ];
        assert_eq!(
            records_changed_with_a_generation_outside_the_publish_txgs(
                &before,
                &rewritten_and_new_and_released,
                CheckpointTxg(12),
                CheckpointTxg(12),
                no_quarantine_mark_change_vouched_for()
            ),
            Vec::new(),
            "新分配、复用改写、释放的代都是这次的 txg"
        );
        let rewritten_without_the_generation = vec![
            record(50176, 2, 0, false),
            record(50178, 1, 5, false),
            record(50180, 2, 3, false),
            record(50182, 1, 12, true),
            record(50184, 2, 12, false),
        ];
        assert_eq!(
            records_changed_with_a_generation_outside_the_publish_txgs(
                &before,
                &rewritten_without_the_generation,
                CheckpointTxg(12),
                CheckpointTxg(12),
                no_quarantine_mark_change_vouched_for()
            ),
            vec![record(50178, 1, 5, false)],
            "复用改写的那一条还写着释放代 5"
        );
        assert!(
            allocation_generation_judgement(
                &before,
                &rewritten_without_the_generation,
                CheckpointTxg(12),
                CheckpointTxg(12),
                &[]
            )
            .is_some(),
            "报出来就是执行器判的失败"
        );
    }

    /// 不是回退那次发布、也没有交回隔离：只改隔离记号的记录一条都不放过。
    fn no_quarantine_mark_change_vouched_for() -> QuarantineMarkChangesVouchedFor<'static> {
        QuarantineMarkChangesVouchedFor {
            copies_quarantined_by_the_publishes: &[],
            clearing: QuarantineMarkClearing::NotDoneByThisPublish,
        }
    }

    fn with_the_quarantine_mark(record: AllocationRecord) -> AllocationRecord {
        AllocationRecord {
            device: record.device,
            slot: record.slot,
            span_slots: record.span_slots,
            generation: record.generation,
            is_released: record.is_released,
            quarantine_mark:
                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
        }
    }

    /// 实现交回的一份隔离：盘、起点槽与跨度如给。
    fn copy_quarantined_on(
        device: DeviceIdentity,
        slot: u64,
        span_slots: u64,
    ) -> CopyQuarantinedAfterReleaseChecksumMismatch {
        CopyQuarantinedAfterReleaseChecksumMismatch {
            unit: singlefs_core::transaction::TransactionUnit::AllocationTree,
            device,
            placement: singlefs_core::allocator::Placement {
                slot: SlotNumber(slot),
                span: span_slots,
            },
            reading: singlefs_core::transaction::QuarantinedCopyReading::ChecksumMismatch,
        }
    }

    /// 隔离那次发布（txg 12）把分配代 7 的那条记录置上记号、别的不动，这次发布交回了这一份隔离：不报（D19 已定项 5「隔离的盘上记号」）。
    #[test]
    fn quarantine_mark_set_on_a_copy_the_publish_reported_is_not_reported() {
        let before = vec![record(50186, 2, 7, false)];
        let after = vec![with_the_quarantine_mark(record(50186, 2, 7, false))];
        let copies = [copy_quarantined_on(DeviceIdentity(0), 50186, 2)];
        assert_eq!(
            records_changed_with_a_generation_outside_the_publish_txgs(
                &before,
                &after,
                CheckpointTxg(12),
                CheckpointTxg(12),
                QuarantineMarkChangesVouchedFor {
                    copies_quarantined_by_the_publishes: &copies,
                    clearing: QuarantineMarkClearing::NotDoneByThisPublish,
                }
            ),
            Vec::new(),
            "只差记号、这次交回了这一份隔离的记录不算改了代"
        );
        assert_eq!(
            allocation_generation_judgement(
                &before,
                &after,
                CheckpointTxg(12),
                CheckpointTxg(12),
                &copies
            ),
            None,
            "通用的判法认这次发布交回的隔离"
        );
    }

    /// 记号变了而这次发布没交回那一份隔离（一份都没交、交的是别的槽、交的是同槽而跨度不同、交的是别的盘）：照报。
    #[test]
    fn quarantine_mark_set_on_a_copy_the_publish_did_not_report_is_reported() {
        let before = vec![record(50186, 2, 7, false)];
        let after = vec![with_the_quarantine_mark(record(50186, 2, 7, false))];
        for (what, copies) in [
            ("一份都没交", Vec::new()),
            (
                "交的是别的槽",
                vec![copy_quarantined_on(DeviceIdentity(0), 50188, 2)],
            ),
            (
                "交的是同槽而跨度不同",
                vec![copy_quarantined_on(DeviceIdentity(0), 50186, 1)],
            ),
            (
                "交的是别的盘",
                vec![copy_quarantined_on(DeviceIdentity(1), 50186, 2)],
            ),
        ] {
            assert_eq!(
                records_changed_with_a_generation_outside_the_publish_txgs(
                    &before,
                    &after,
                    CheckpointTxg(12),
                    CheckpointTxg(12),
                    QuarantineMarkChangesVouchedFor {
                        copies_quarantined_by_the_publishes: &copies,
                        clearing: QuarantineMarkClearing::DoneByTheRollbackPublish,
                    }
                ),
                after,
                "{what}：置记号的记录代 7 不是这次的 txg 12，照报"
            );
        }
    }

    /// 记号连同分配代一起变（代 7 改成 5）：照报。
    #[test]
    fn quarantine_mark_set_together_with_a_generation_change_is_reported() {
        let before = vec![record(50186, 2, 7, false)];
        let after = vec![with_the_quarantine_mark(record(50186, 2, 5, false))];
        let copies = [copy_quarantined_on(DeviceIdentity(0), 50186, 2)];
        assert_eq!(
            records_changed_with_a_generation_outside_the_publish_txgs(
                &before,
                &after,
                CheckpointTxg(12),
                CheckpointTxg(12),
                QuarantineMarkChangesVouchedFor {
                    copies_quarantined_by_the_publishes: &copies,
                    clearing: QuarantineMarkClearing::DoneByTheRollbackPublish,
                }
            ),
            after,
            "记号之外分配代也变了，不是只改记号"
        );
    }

    /// 置记号又改了跨度（2 改成 1；交回的隔离跨度也是 1，只有前后两条比得出跨度变了）：照报。
    #[test]
    fn quarantine_mark_set_together_with_a_span_change_is_reported() {
        let before = vec![record(50186, 2, 7, false)];
        let after = vec![with_the_quarantine_mark(record(50186, 1, 7, false))];
        let copies = [copy_quarantined_on(DeviceIdentity(0), 50186, 1)];
        assert_eq!(
            records_changed_with_a_generation_outside_the_publish_txgs(
                &before,
                &after,
                CheckpointTxg(12),
                CheckpointTxg(12),
                QuarantineMarkChangesVouchedFor {
                    copies_quarantined_by_the_publishes: &copies,
                    clearing: QuarantineMarkClearing::DoneByTheRollbackPublish,
                }
            ),
            after,
            "记号之外跨度也变了，不是只改记号"
        );
    }

    /// 前后有一条是已释放的不放过：已释放（代 7）改成未释放带记号；带记号未释放改成已释放而代还是 7（回退那一处也不放过）。
    #[test]
    fn released_records_are_not_passed_as_quarantine_mark_changes() {
        let copies = [copy_quarantined_on(DeviceIdentity(0), 50186, 2)];
        let vouched_for = QuarantineMarkChangesVouchedFor {
            copies_quarantined_by_the_publishes: &copies,
            clearing: QuarantineMarkClearing::DoneByTheRollbackPublish,
        };
        let released_before = vec![record(50186, 2, 7, true)];
        let marked_after = vec![with_the_quarantine_mark(record(50186, 2, 7, false))];
        assert_eq!(
            records_changed_with_a_generation_outside_the_publish_txgs(
                &released_before,
                &marked_after,
                CheckpointTxg(12),
                CheckpointTxg(12),
                vouched_for
            ),
            marked_after,
            "发布之前那一条已释放：不是只改记号"
        );
        let marked_before = vec![with_the_quarantine_mark(record(50186, 2, 7, false))];
        let released_after = vec![record(50186, 2, 7, true)];
        assert_eq!(
            records_changed_with_a_generation_outside_the_publish_txgs(
                &marked_before,
                &released_after,
                CheckpointTxg(12),
                CheckpointTxg(12),
                vouched_for
            ),
            released_after,
            "发布之后那一条已释放：不是只清记号"
        );
    }

    /// 带记号的记录改回没有记号、别的不动：回退那次发布放过；不是回退的发布（连同通用的判法）照报。
    #[test]
    fn quarantine_mark_cleared_is_passed_only_by_the_rollback_publish() {
        let before = vec![with_the_quarantine_mark(record(50186, 2, 7, false))];
        let after = vec![record(50186, 2, 7, false)];
        let vouched_for = |clearing| QuarantineMarkChangesVouchedFor {
            copies_quarantined_by_the_publishes: &[],
            clearing,
        };
        assert_eq!(
            records_changed_with_a_generation_outside_the_publish_txgs(
                &before,
                &after,
                CheckpointTxg(12),
                CheckpointTxg(12),
                vouched_for(QuarantineMarkClearing::DoneByTheRollbackPublish)
            ),
            Vec::new(),
            "回退那次发布清掉回退目标引用的单元的记号"
        );
        assert_eq!(
            records_changed_with_a_generation_outside_the_publish_txgs(
                &before,
                &after,
                CheckpointTxg(12),
                CheckpointTxg(12),
                vouched_for(QuarantineMarkClearing::NotDoneByThisPublish)
            ),
            after,
            "不是回退的发布清了记号，照报"
        );
        assert!(
            allocation_generation_judgement(
                &before,
                &after,
                CheckpointTxg(12),
                CheckpointTxg(12),
                &[]
            )
            .is_some(),
            "通用的判法不放过清记号"
        );
    }

    /// 回退那次发布（txg 12）清掉回退目标引用的单元那条记录的记号（分配代 7 不变）：回退那一处放过。
    #[test]
    fn the_rollback_publish_clearing_the_quarantine_mark_is_not_reported() {
        let before = vec![with_the_quarantine_mark(record(50186, 2, 7, false))];
        let after = vec![record(50186, 2, 7, false)];
        assert_eq!(
            records_the_rollback_publish_changed_with_a_generation_outside_its_txg(
                &before,
                &after,
                CheckpointTxg(12),
                &[],
                &[]
            ),
            Vec::new(),
            "回退复活那一次同一次发布清掉记号"
        );
    }

    /// 回退那次发布释放的一份核出对不上、置上记号（分配代 7 不变）：这一步交回了这一份才放过，没交回照报。
    #[test]
    fn the_rollback_publish_setting_the_quarantine_mark_is_not_reported_only_on_a_copy_the_step_reported(
    ) {
        let before = vec![record(50186, 2, 7, false)];
        let after = vec![with_the_quarantine_mark(record(50186, 2, 7, false))];
        assert_eq!(
            records_the_rollback_publish_changed_with_a_generation_outside_its_txg(
                &before,
                &after,
                CheckpointTxg(12),
                &[],
                &[copy_quarantined_on(DeviceIdentity(0), 50186, 2)]
            ),
            Vec::new(),
            "这一步交回了这一份隔离"
        );
        assert_eq!(
            records_the_rollback_publish_changed_with_a_generation_outside_its_txg(
                &before,
                &after,
                CheckpointTxg(12),
                &[],
                &[]
            ),
            after,
            "这一步没交回这一份隔离，照报"
        );
    }

    /// 整理的一批（txg 12）释放的一份核出对不上、置上记号（分配代 7 不变）：这一批交回了这一份才放过，没交回照报；清记号照报。
    #[test]
    fn relocation_batch_setting_the_quarantine_mark_is_not_reported_only_on_a_copy_it_reported() {
        let unmarked = vec![record(50186, 2, 7, false)];
        let marked = vec![with_the_quarantine_mark(record(50186, 2, 7, false))];
        let copies = [copy_quarantined_on(DeviceIdentity(0), 50186, 2)];
        assert_eq!(
            records_a_relocation_batch_changed_with_a_generation_outside_its_txg(
                &unmarked,
                &marked,
                CheckpointTxg(12),
                &[],
                &copies
            ),
            Vec::new(),
            "这一批交回了这一份隔离"
        );
        assert_eq!(
            records_a_relocation_batch_changed_with_a_generation_outside_its_txg(
                &unmarked,
                &marked,
                CheckpointTxg(12),
                &[],
                &[]
            ),
            marked,
            "这一批没交回这一份隔离，照报"
        );
        assert_eq!(
            records_a_relocation_batch_changed_with_a_generation_outside_its_txg(
                &marked,
                &unmarked,
                CheckpointTxg(12),
                &[],
                &copies
            ),
            unmarked,
            "整理的一批不清记号"
        );
    }

    /// 冷启动读回报错是执行器判的失败；读回文件、没有文件、别的结局都不是。
    #[test]
    fn failed_cold_start_read_back_is_a_harness_judgement_and_the_other_read_backs_are_not() {
        let recovered = |read_back: ColdStartReadBack| {
            StepOutcome::Applied(AppliedEffect::Recovered {
                outcome: format!("{read_back:?}"),
                read_back,
            })
        };
        assert_eq!(
            harness_judgement_of_outcome(&recovered(ColdStartReadBack::Failed)),
            Some(
                HarnessJudgement::ColdStartRecoveryFailedOnCheckerGreenImage {
                    outcome: "Failed".to_string()
                }
            )
        );
        assert_eq!(
            harness_judgement_of_outcome(&recovered(ColdStartReadBack::FileRead)),
            None
        );
        assert_eq!(
            harness_judgement_of_outcome(&recovered(ColdStartReadBack::NoFile)),
            None
        );
        assert_eq!(
            harness_judgement_of_outcome(&StepOutcome::Refused {
                member: "RollbackError::TargetNotACandidate(NotInRing)".to_string()
            }),
            None
        );
    }
}
