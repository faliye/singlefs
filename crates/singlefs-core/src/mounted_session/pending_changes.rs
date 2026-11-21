//! 攒批的会话（[`MountedSessionWithPendingChanges`]）收下改动与发布：待发的数据改动、收下时的判定、发布触发，
//! 单发改动与把会话交回，连同收下时交回的结果与拒绝。

use super::*;

/// 收下、还没发布的数据写类改动：折叠过的那一份整份写（同一 inode 的几次写只留最后一次）、按收下的次序的几次建 inode，
/// 与收下的这几次改动各自单独发布要写的 journal 字节之和（D16（发布语义） 已定项 5：按待发改动要写的 journal 字节算，批内覆盖写 K 次按 K 次的记录照实算、不折叠）。
#[derive(Debug)]
pub(super) struct PendingDataChanges {
    whole_write: Option<PendingWholeWrite>,
    new_inodes: Vec<NewInodesOfOneChange>,
    journal_bytes_of_the_changes_accepted: JournalBytesToWrite,
    changes_accepted: u64,
}

/// 待发里那一份整份写（内容自有）。
#[derive(Clone, Debug)]
struct PendingWholeWrite {
    inode: InodeNumber,
    content: Vec<u8>,
    write_time_seconds: u64,
}

impl PendingDataChanges {
    fn empty() -> Self {
        Self {
            whole_write: None,
            new_inodes: Vec::new(),
            journal_bytes_of_the_changes_accepted: JournalBytesToWrite::ZERO,
            changes_accepted: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.changes_accepted == 0
    }

    /// 待发并成一次发布的样子。
    fn as_published_together(&self) -> DataChangesPublishedTogether<'_> {
        DataChangesPublishedTogether {
            whole_write: self
                .whole_write
                .as_ref()
                .map(|whole_write| WholeWriteOfOneInode {
                    inode: whole_write.inode,
                    content: &whole_write.content,
                    write_time_seconds: whole_write.write_time_seconds,
                }),
            new_inodes: &self.new_inodes,
        }
    }

    /// 这一次改动并进待发之后那一批的整份写与按收下次序的建 inode（`DataChangesPublishedTogether` 借的那两样）：整份写后到的盖掉先到的。
    fn together_with_this_change<'batch>(
        &'batch self,
        data_change: DataChangeAccepted<'batch>,
    ) -> (
        Option<WholeWriteOfOneInode<'batch>>,
        Vec<NewInodesOfOneChange>,
    ) {
        let mut new_inodes_together = self.new_inodes.clone();
        let whole_write_together = match data_change {
            DataChangeAccepted::WholeWrite(whole_write) => Some(whole_write),
            DataChangeAccepted::NewInodes(created) => {
                new_inodes_together.push(created);
                self.whole_write
                    .as_ref()
                    .map(|pending_whole_write| WholeWriteOfOneInode {
                        inode: pending_whole_write.inode,
                        content: &pending_whole_write.content,
                        write_time_seconds: pending_whole_write.write_time_seconds,
                    })
            }
        };
        (whole_write_together, new_inodes_together)
    }

    /// 这一批里建的 inode 一共几个（之后建的号从记账水位加上它起发）。
    fn new_inodes_in_total(&self) -> u64 {
        self.as_published_together().new_inodes_in_total()
    }

    /// 收下一次数据写类改动：整份写折叠（后到的盖掉先到的），建 inode 追加在后面；这次改动单独发布要写的 journal 字节加进合计。
    fn take_in(
        &mut self,
        change: DataChangeAccepted<'_>,
        journal_bytes_of_this_change: JournalBytesToWrite,
    ) {
        match change {
            DataChangeAccepted::WholeWrite(whole_write) => {
                self.whole_write = Some(PendingWholeWrite {
                    inode: whole_write.inode,
                    content: whole_write.content.to_vec(),
                    write_time_seconds: whole_write.write_time_seconds,
                });
            }
            DataChangeAccepted::NewInodes(new_inodes) => self.new_inodes.push(new_inodes),
        }
        self.journal_bytes_of_the_changes_accepted = self
            .journal_bytes_of_the_changes_accepted
            .plus(journal_bytes_of_this_change);
        self.changes_accepted += 1;
    }
}

/// 收得进待发的一次数据写类改动（D16（发布语义） 已定项 10「数据写类可并」）。封闭集合。
#[derive(Clone, Copy, Debug)]
enum DataChangeAccepted<'content> {
    WholeWrite(WholeWriteOfOneInode<'content>),
    NewInodes(NewInodesOfOneChange),
}

impl<'content> DataChangeAccepted<'content> {
    /// 这一次改动单独并成一批的样子（单独发布要写多少 journal 字节按它判）。`new_inodes` 是装建 inode 那一项的一格槽（借给返回值）。
    fn alone(
        self,
        new_inodes: &'content mut [NewInodesOfOneChange; 1],
    ) -> DataChangesPublishedTogether<'content> {
        match self {
            DataChangeAccepted::WholeWrite(whole_write) => DataChangesPublishedTogether {
                whole_write: Some(whole_write),
                new_inodes: &[],
            },
            DataChangeAccepted::NewInodes(created) => {
                new_inodes[0] = created;
                DataChangesPublishedTogether {
                    whole_write: None,
                    new_inodes: &new_inodes[..],
                }
            }
        }
    }
}

/// 交给 [`MountedSessionWithPendingChanges::accept`] 的一次改动按什么收。封闭集合。
#[derive(Clone, Copy, Debug)]
enum ChangeAtTheAccept<'content> {
    /// 数据写类：收进待发。覆盖写那两种（一个数据单元的契约）带 `fits_one_data_unit_only`。
    Data {
        change: DataChangeAccepted<'content>,
        contract: WholeWriteContractAtTheAccept,
    },
    /// 屏障：先排空待发、再单发（unlink、截断、建删快照）。
    Barrier,
    /// 按偏移写：进不进待发、算哪一类条款没写（D16（发布语义） 已定项 10 只列了整份写、建 inode 与屏障），第一版不收。
    WriteAtOffset,
    /// 已经并好的一批：它是排空待发时交给会话的，不是用户的一次改动，收进待发条款没写，第一版不收。
    DataChangesPublishedTogether,
}

/// 整份写的入口契约（同会话两条入口：覆盖写一个数据单元以内，顺序写不按单元数拒）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WholeWriteContractAtTheAccept {
    OneDataUnit,
    AnyNumberOfDataUnits,
    /// 不是整份写（建 inode）。
    NotAWholeWrite,
}

impl<'content> ChangeAtTheAccept<'content> {
    fn of(change: UserChange<'content>) -> Self {
        let whole_write = |inode: InodeNumber,
                           content: &'content [u8],
                           write_time_seconds: u64,
                           contract: WholeWriteContractAtTheAccept| {
            ChangeAtTheAccept::Data {
                change: DataChangeAccepted::WholeWrite(WholeWriteOfOneInode {
                    inode,
                    content,
                    write_time_seconds,
                }),
                contract,
            }
        };
        let first_file = InodeNumber(crate::transaction::FIRST_INODE_NUMBER);
        match change {
            UserChange::Overwrite(file) => whole_write(
                first_file,
                file.content,
                file.write_time_seconds,
                WholeWriteContractAtTheAccept::OneDataUnit,
            ),
            UserChange::SequentialWrite(file) => whole_write(
                first_file,
                file.content,
                file.write_time_seconds,
                WholeWriteContractAtTheAccept::AnyNumberOfDataUnits,
            ),
            UserChange::OverwriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => whole_write(
                inode,
                content,
                write_time_seconds,
                WholeWriteContractAtTheAccept::OneDataUnit,
            ),
            UserChange::SequentialWriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => whole_write(
                inode,
                content,
                write_time_seconds,
                WholeWriteContractAtTheAccept::AnyNumberOfDataUnits,
            ),
            UserChange::NewInodes {
                count,
                write_time_seconds,
            } => ChangeAtTheAccept::Data {
                change: DataChangeAccepted::NewInodes(NewInodesOfOneChange {
                    count,
                    write_time_seconds,
                }),
                contract: WholeWriteContractAtTheAccept::NotAWholeWrite,
            },
            UserChange::Unlink { .. }
            | UserChange::Truncate { .. }
            | UserChange::CreateSnapshot
            | UserChange::DestroySnapshot { .. } => ChangeAtTheAccept::Barrier,
            UserChange::WriteAtOffset { .. } => ChangeAtTheAccept::WriteAtOffset,
            UserChange::DataChangesPublishedTogether(_) => {
                ChangeAtTheAccept::DataChangesPublishedTogether
            }
        }
    }
}

/// 一次触发是哪一支先到（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支可观测）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishTriggerCause {
    /// now − 上次发布的读数 ≥ `T_time`，脏量还没到。
    TimeThreshold,
    /// 待发改动要写的 journal 字节 ≥ 有效 `T_dirty`，时间还没到。
    DirtyThreshold,
    /// 两支同时到。
    BothThresholds,
}

/// D16（发布语义） 已定项 2 的触发式：待发非空 ∧（离上次发布过了 `T_time` ∨ 待发改动要写的 journal 字节到了有效 `T_dirty`）。待发空时两支都不看
/// （空闲时不推空发布：照不带这一合取的式子，空闲时每 `T_time` 一次空发布，把回退候选集挤掉——判决第二节「时间那一支」那一格）。
/// `time_since_the_last_publish` 是调用方按读数算好的间隔（读数倒退时是 0：当作没过时间）。纯函数。
#[must_use]
pub fn publish_trigger_cause(
    pending_is_empty: bool,
    time_since_the_last_publish: Duration,
    journal_bytes_of_the_pending_changes: JournalBytesToWrite,
    thresholds: PublishTriggerThresholds,
) -> Option<PublishTriggerCause> {
    if pending_is_empty {
        return None;
    }
    let is_time_reached = time_since_the_last_publish >= thresholds.time_threshold();
    let is_dirty_reached = journal_bytes_of_the_pending_changes >= thresholds.dirty_threshold();
    match (is_time_reached, is_dirty_reached) {
        (true, true) => Some(PublishTriggerCause::BothThresholds),
        (true, false) => Some(PublishTriggerCause::TimeThreshold),
        (false, true) => Some(PublishTriggerCause::DirtyThreshold),
        (false, false) => None,
    }
}

/// 攒批的会话这一次为什么发布（报给调用方，RefFS 翻译按它把每一次发布翻成一个显式发布点，判决第二节「RefFS」那一格）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReasonOfThePublish {
    /// [`MountedSessionWithPendingChanges::accept`] 收下这次改动之后触发，待发整个并成一次发布（这次改动在里面）。
    TriggeredAfterTheAccept(PublishTriggerCause),
    /// [`MountedSessionWithPendingChanges::publish_when_due`] 里触发。
    TriggeredWhenDue(PublishTriggerCause),
    /// 屏障进门之前排空待发（屏障自己不在里面）。
    PendingChangesPublishedBeforeABarrier,
    /// 屏障自己那一次（[`MountedSession::publish_user_change`] 单发：unlink、截断连同它们的全部批）。
    BarrierPublishedAlone,
    /// [`MountedSessionWithPendingChanges::publish_pending_now`]：立即排空。
    PendingChangesPublishedOnRequest,
    /// [`MountedSessionWithPendingChanges::publish_user_change`] 之前排空待发。
    PendingChangesPublishedBeforeTheUserChange,
    /// [`MountedSessionWithPendingChanges::publish_user_change`] 那一次改动自己。
    UserChangePublishedAlone,
    /// 后台整理判过起跑、跑之前排空待发。
    PendingChangesPublishedBeforeTheBackgroundCompaction,
    /// 交出会话（卸载、回退之前）排空待发。
    PendingChangesPublishedBeforeHandingBackTheSession,
}

/// 攒批的会话报的一次发布：为什么发、发成了什么（推的抬 F、发成的每一版、之后发的 discard，同 [`UserChangePublished`]）。
#[derive(Debug)]
pub struct PublishReportedByTheSession {
    pub reason: ReasonOfThePublish,
    pub published: UserChangePublished,
}

/// [`MountedSessionWithPendingChanges::accept`] 收下了：怎么收的、这次调用里报的每一次发布（按先后）、收下之前为空间当场推的抬 F。
#[derive(Debug)]
pub struct ChangeAccepted {
    pub taken: HowTheChangeWasTaken,
    /// 这次调用里发成的待发与屏障：改了用户可见状态的发布，不含 `floor_raises_pushed_for_space` 那几串空发布。
    pub publishes: Vec<PublishReportedByTheSession>,
    pub floor_raises_pushed_for_space: FloorRaisesPushedByTheAccept,
}

/// [`MountedSessionWithPendingChanges::accept`] 这次调用里，这一次改动被空间拒时当场推成的几串抬 F 的空发布（D16（发布语义） 已定项 1「准入」那一行），已落盘。
/// 空发布不是用户改动：不进 [`ChangeAccepted::publishes`]、不重给整理预算、不算上次发布的读数（同 [`MountedSession::publish_user_change`] 被拒时推的那几串），
/// 待发不在里面，收下的改动照旧等下一次排空。
#[derive(Debug)]
pub struct FloorRaisesPushedByTheAccept {
    pub floor_raises: Vec<RaisedFloor>,
    /// 每一串是因为哪一次被拒才推的（与 `floor_raises` 一一对应）。
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
}

/// [`MountedSessionWithPendingChanges::accept`] 进门判的是哪一批。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchJudgedAtTheAccept {
    ThisChangeAlone,
    PendingChangesTogetherWithThisChange,
}

/// 一次改动在 [`MountedSessionWithPendingChanges::accept`] 里怎么收的。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HowTheChangeWasTaken {
    /// 收进待发；`journal_bytes_of_this_change` 是这次改动单独发布要写的 journal 字节（加进待发的合计）。收下之后触发了，待发连同它已经发成
    /// （报在 [`ChangeAccepted::publishes`] 里）。
    KeptPending {
        journal_bytes_of_this_change: JournalBytesToWrite,
    },
    /// 屏障：先排空待发、再单发，都已发成。
    PublishedAloneAsABarrier,
}

/// [`MountedSessionWithPendingChanges::accept`] 没收下，或收下之后触发的那一次发布没发成。成员按调用方要做的决定分。
#[derive(Debug)]
pub enum AcceptRefused {
    /// 分配器上冻结着一次写失败、没重发的发布（D23（journal 的角色与格式） 已定项 14；判决第二节「串行下不立的」那一格「冻结时 `accept` 拒」）：
    /// 在任何读写之前拒，待发不动、不长。调用方要做的是先原样重发那一次（交出会话，`transaction::resend_the_frozen_publish`）。
    PublishFrozenAfterAWriteFailureIsNotResentYet {
        checkpoint_txg: CheckpointTxg,
        instance: InstanceGeneration,
    },
    /// 同 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`]，在任何读写之前。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 现行那一版没有可改的文件（树表 0 条；或有树而文件集合为空而这次是整份写），同 [`UserChangeRefused::NoFileVersionToChange`]，在任何读写之前。
    NoFileVersionToChange,
    /// 现行那一版的承诺类意图队列（`deleted_inodes`、销毁中的快照、截断条目）没走空：会话里每一次用户改动先走空（D8（核心索引结构） 已定项 16 第 6 条），
    /// 走空是写，而这次的预判要在任何读写之前做、要接在走空之后的那一版上判——两样合不到一起，条款没写，第一版不支持。在任何读写之前拒；
    /// 调用方要做的是先经 [`MountedSessionWithPendingChanges::publish_user_change`] 发一次（它先走空）。
    QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion,
    /// 按偏移写进不进待发、算数据写类还是屏障，条款没写（D16（发布语义） 已定项 10 只列整份写、建 inode 与屏障），第一版不支持。在任何读写之前拒；
    /// 调用方要做的是经 [`MountedSessionWithPendingChanges::publish_user_change`] 单发它。
    WriteAtOffsetIntoThePendingChangesNotSupportedInTheFirstVersion,
    /// 交进来的是已经并好的一批（[`UserChange::DataChangesPublishedTogether`]）：把一批再收进待发条款没写，第一版不支持。在任何读写之前拒。
    DataChangesPublishedTogetherIntoThePendingChangesNotSupportedInTheFirstVersion,
    /// 待发里已经有另一个文件的整份写：一次发布写一个文件以内是这一版发布计划的形状（`transaction::PublishPlan::file`），不同 inode 的写并不并进一次发布
    /// 条款没写（D16（发布语义） 已定项 10「同一 inode 的写折叠」管不到两个号），第一版不支持。在任何读写之前拒；调用方要做的是先
    /// [`MountedSessionWithPendingChanges::publish_pending_now`]、再交这次的写。
    WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion {
        inode_pending: InodeNumber,
        inode_written: InodeNumber,
    },
    /// 写的号是待发里建 inode 那几次要发的号（还不在现行那一版的树里）：同一次发布里建一个 inode 又写它，条款没写，第一版不支持。在任何读写之前拒。
    WriteToAnInodeCreatedByThePendingChangesNotSupportedInTheFirstVersion { inode: InodeNumber },
    /// 这一次改动单独发布就过不了落盘之前那几判（`transaction::journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write`：
    /// 号不在树里、删除中、内容装不进覆盖写的一个数据单元、记录条数、合取 7……），`cause` 原样，不是空间不够的那两种（[`refusal_is_short_of_space`]）。
    /// 在任何读写之前拒，待发不动。
    ChangeRefusedByThePublishJudgement(Box<PublishError>),
    /// 这一次改动单独过得了，并进待发合起来这一批过不了（判决第二节「一次发布里攒什么」那一格：`accept` 进门先做这一批合起来的条数预判，
    /// 过不了就在任何读写之前拒，不在返回成功之后拒），`cause` 原样，不是空间不够的那两种。在任何读写之前拒，待发不动；调用方要做的是先排空待发、再交这一次。
    PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(Box<PublishError>),
    /// `judgement` 那一批被空间拒，当场推抬 F 再判（D16（发布语义） 已定项 1「准入」那一行）之后还是没收下，`refusal` 原样：
    /// [`UserChangeRefused::NoSpaceAfterRaisingTheFloor`] 是推到不再推了仍放不下（一次准入的发布数做满、F 已在上限、抬 F 那一串自己放不下，一串都没推成时 `floor_raises` 是空的）；
    /// [`UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`] 是抬 F 之前先算判出这一次不推：槽对与式子那两判是推抬 F 放得出的全放出来也放不下，
    /// 合取 7 那一形是这次调用还一串没推、被拒的是合取 7 判的前台写入类、写之前那一刻全空聚簇段已少于 `N₀`（头一次被拒就判到这里时 `floor_raises` 是空的）；
    /// [`UserChangeRefused::FloorRaiseFailedWhilePushingForSpace`] 是抬 F 自己报了错（写盘报错时那一次空发布冻结）；
    /// [`UserChangeRefused::Publish`] 是推过之后再判拒成了别的原因。推成的那几串已落盘，现行那一版在它们后面（[`Self::floor_raises_pushed_by_this_accept`]）。
    /// 这一次改动没收进待发；待发里已有的原样留着，冻结的那一次空发布不含它们。
    RefusedAfterRaisingTheFloorForSpace {
        judgement: BatchJudgedAtTheAccept,
        refusal: Box<UserChangeRefused>,
    },
    /// 屏障进门之前排空待发没发成（`refusal` 原样；推的抬 F、发成的走空步在里面）：屏障没发。
    PublishOfThePendingChangesBeforeTheBarrierRefused(Box<UserChangeRefused>),
    /// 待发已排空（`publishes_before_the_refusal`），屏障自己没发成（`refusal` 原样）。
    BarrierRefused {
        publishes_before_the_refusal: Vec<PublishReportedByTheSession>,
        refusal: Box<UserChangeRefused>,
    },
    /// 这次改动已收进待发，收下之后触发的那一次发布没发成（`refusal` 原样）：冻结的是这一批自己时待发并进了冻结的那一次
    /// （D16（发布语义） 已定项 17：报错之后这次发布可能已成也可能未成），待发清空；别的情形待发原样留着（连同这一次），下一次触发再发。
    /// `floor_raises_pushed_for_space` 是收下之前为空间推成的几串。
    AcceptedButTheTriggeredPublishRefused {
        floor_raises_pushed_for_space: FloorRaisesPushedByTheAccept,
        refusal: Box<UserChangeRefused>,
    },
}

impl AcceptRefused {
    /// 这次调用没收下或没发成时，已经为空间推成、落了盘的那几串抬 F（空的是一串都没推）。
    #[must_use]
    pub fn floor_raises_pushed_by_this_accept(&self) -> &[RaisedFloor] {
        match self {
            AcceptRefused::RefusedAfterRaisingTheFloorForSpace { refusal, .. } => match &**refusal {
                UserChangeRefused::NoFileVersionToChange { floor_raises, .. }
                | UserChangeRefused::Publish { floor_raises, .. } => floor_raises,
                UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) => &no_space.floor_raises,
                UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) => {
                    &no_space.floor_raises
                }
                UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed) => {
                    &failed.floor_raises
                }
                UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
                | UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration {
                    ..
                }
                | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
                | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
                | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    ..
                } => &[],
            },
            AcceptRefused::AcceptedButTheTriggeredPublishRefused {
                floor_raises_pushed_for_space,
                ..
            } => &floor_raises_pushed_for_space.floor_raises,
            AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
            | AcceptRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
            | AcceptRefused::NoFileVersionToChange
            | AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion
            | AcceptRefused::WriteAtOffsetIntoThePendingChangesNotSupportedInTheFirstVersion
            | AcceptRefused::DataChangesPublishedTogetherIntoThePendingChangesNotSupportedInTheFirstVersion
            | AcceptRefused::WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion { .. }
            | AcceptRefused::WriteToAnInodeCreatedByThePendingChangesNotSupportedInTheFirstVersion { .. }
            | AcceptRefused::ChangeRefusedByThePublishJudgement(_)
            | AcceptRefused::PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(_)
            | AcceptRefused::PublishOfThePendingChangesBeforeTheBarrierRefused(_)
            | AcceptRefused::BarrierRefused { .. } => &[],
        }
    }
}

/// 排空待发那一次没发成（[`MountedSessionWithPendingChanges::publish_when_due`]、[`MountedSessionWithPendingChanges::publish_pending_now`]）：
/// `refusal` 原样。冻结的是这一批自己时待发并进了冻结的那一次、清空（D16（发布语义） 已定项 17）；别的情形（没冻结、冻结的是抬 F 的空发布或进门之前就冻结着的那一次）待发原样留着。
#[derive(Debug)]
pub struct PublishOfThePendingChangesRefused {
    pub refusal: Box<UserChangeRefused>,
}

/// [`MountedSessionWithPendingChanges::publish_user_change`] 没发成：之前排空的那一次（发成了就在 `publishes_before_the_refusal` 里）与这一次的拒绝。
#[derive(Debug)]
pub struct UserChangeAfterThePendingChangesRefused {
    pub publishes_before_the_refusal: Vec<PublishReportedByTheSession>,
    pub refusal: Box<UserChangeRefused>,
}

/// [`MountedSessionWithPendingChanges::into_session_after_publishing_the_pending_changes`] 交不出会话：排空待发没发成，攒批的会话原样交回。
#[derive(Debug)]
pub struct HandingBackTheSessionRefused {
    pub session_with_pending_changes: MountedSessionWithPendingChanges,
    pub refusal: Box<UserChangeRefused>,
}

impl MountedSessionWithPendingChanges {
    /// 会话接上待发（一开始空的），两个阈值取这次挂载的有效值（[`PublishTriggerThresholds::of_the_mount`]：`T_time` 5000 ms，
    /// `T_dirty` = min(2 GiB, 环长 ÷ F)）。
    #[must_use]
    pub fn of_the_session(session: MountedSession) -> Self {
        let thresholds = PublishTriggerThresholds::of_the_mount(
            session
                .parameters_and_device_table
                .parameters_on_disk()
                .geometry
                .journal_ring_bytes,
        );
        Self::of_the_session_with_thresholds(session, thresholds)
    }

    /// 同 [`Self::of_the_session`]，两个阈值由调用方给：产品路径上只拿得到 [`PublishTriggerThresholds::of_the_mount`] 那一份，只供测试的那一份
    /// 在默认关的特性后面（[`PublishTriggerThresholds`] 文档）。
    #[must_use]
    pub fn of_the_session_with_thresholds(
        session: MountedSession,
        thresholds: PublishTriggerThresholds,
    ) -> Self {
        Self {
            session,
            pending: PendingDataChanges::empty(),
            thresholds,
            reading_of_the_last_publish: None,
        }
    }

    /// 底下那个会话（只读）。
    #[must_use]
    pub fn session(&self) -> &MountedSession {
        &self.session
    }

    /// 这次挂载生效的两个阈值。
    #[must_use]
    pub fn thresholds(&self) -> PublishTriggerThresholds {
        self.thresholds
    }

    /// 待发里收下了几次改动（排空之后归零）。
    #[must_use]
    pub fn pending_changes_accepted(&self) -> u64 {
        self.pending.changes_accepted
    }

    /// 待发里那几次改动各自单独发布要写的 journal 字节之和（脏量那一支比的量）。
    #[must_use]
    pub fn journal_bytes_of_the_pending_changes(&self) -> JournalBytesToWrite {
        self.pending.journal_bytes_of_the_changes_accepted
    }

    /// 上一次发布那次调用交进来的读数（还没发布过时是第一次见到的读数）。
    #[must_use]
    pub fn reading_of_the_last_publish(&self) -> Option<CallerMonotonicClockReading> {
        self.reading_of_the_last_publish
    }

    /// 第一次见到的读数当基线（还没发布过时「上次发布的时刻」取它）。
    pub(super) fn note_the_reading_seen(&mut self, now: CallerMonotonicClockReading) {
        if self.reading_of_the_last_publish.is_none() {
            self.reading_of_the_last_publish = Some(now);
        }
    }

    /// 发成一次：上次发布的读数取这次调用交进来的那一个，只往前走（读数倒退时不回拨）。
    pub(super) fn note_a_publish(&mut self, now: CallerMonotonicClockReading) {
        self.reading_of_the_last_publish = Some(match self.reading_of_the_last_publish {
            Some(before) => before.max(now),
            None => now,
        });
    }

    /// 此刻（`now`）触不触发：读数倒退时间隔是 0（当作没过时间）。
    fn trigger_cause_at(&self, now: CallerMonotonicClockReading) -> Option<PublishTriggerCause> {
        let time_since_the_last_publish = match self.reading_of_the_last_publish {
            Some(last) => now.0.saturating_sub(last.0),
            None => Duration::ZERO,
        };
        publish_trigger_cause(
            self.pending.is_empty(),
            time_since_the_last_publish,
            self.pending.journal_bytes_of_the_changes_accepted,
            self.thresholds,
        )
    }

    /// 把待发整个并成一次发布交给会话（[`UserChange::DataChangesPublishedTogether`]，先走空队列、被空间拒照旧推抬 F 再判）。待发空就什么都不发、交回 `None`
    /// （不推空发布）。发成：待发清空、记下这次的读数。没发成：冻结的是这一批自己（种类 [`KindOfTheFrozenPublish::UserChange`]）时待发并进了冻结的那一次，
    /// 不放回（D16（发布语义） 已定项 17）；别的情形待发原样放回：没冻结、冻结的是这一次推的抬 F 空发布、进门之前就冻结着（那几次都不含这一批，丢了就是收下的改动没了）。
    pub(super) fn publish_the_pending_changes<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        now: CallerMonotonicClockReading,
        reason: ReasonOfThePublish,
    ) -> Result<Option<PublishReportedByTheSession>, Box<UserChangeRefused>> {
        if self.pending.is_empty() {
            return Ok(None);
        }
        let pending = std::mem::replace(&mut self.pending, PendingDataChanges::empty());
        match self.session.publish_user_change(
            devices,
            UserChange::DataChangesPublishedTogether(pending.as_published_together()),
        ) {
            Ok(published) => {
                self.note_a_publish(now);
                Ok(Some(PublishReportedByTheSession { reason, published }))
            }
            Err(refusal) => {
                // 进门之前就冻结着的那一次不含这一批：种类是用户改动的那一次冻结时待发是空的（排空发成之后、或待发并进了它），这里走不到；别的种类不是用户改动。
                let is_the_batch_in_the_frozen_publish = self
                    .session
                    .allocator
                    .frozen_publish()
                    .is_some_and(|frozen| frozen.kind() == KindOfTheFrozenPublish::UserChange);
                if !is_the_batch_in_the_frozen_publish {
                    self.pending = pending;
                }
                Err(Box::new(refusal))
            }
        }
    }

    /// 收下一次用户改动（D16（发布语义） 已定项 10，判决第二节「一次发布里攒什么」「串行下不立的」两格）：
    /// 1. 冻结着一次没重发的发布、交进来的盘表与挂载核过的不同：在任何读写之前拒。
    /// 2. 屏障（unlink、截断、建删快照）：先排空待发（一次发布）、再照 [`MountedSession::publish_user_change`] 单发，两次都报。
    /// 3. 数据写类（整份写、建 inode）：先判——现行那一版有可改的文件、队列走空了、并得进待发（至多一个文件、不写待发里建的号）、
    ///    这一次单独发布过得了落盘之前那几判、并进待发合起来的这一批也过得了（准入与条数预判，`transaction::journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write`）；
    ///    被空间拒（[`refusal_is_short_of_space`] 那两种）时当场推抬 F 的空发布再判（D16（发布语义） 已定项 1「准入」那一行，上限与预算同 [`MountedSession::publish_user_change`]，
    ///    这一次改动算一次准入），别的不过就拒，一个写都不发、待发不动。都过了才收进待发（同一 inode 的写折叠、建 inode 追加），脏量加上这一次单独发布要写的 journal 字节；
    ///    收下之后判一次触发（[`publish_trigger_cause`]），触发就把待发整个并成一次发布。
    /// 4. 按偏移写、已经并好的一批：条款没写，在任何读写之前拒。
    ///
    /// 推抬 F 只在进门判过前两样（没冻结、盘表对）与第 3 条前面那几样之后：冻结着、队列没走空时拒的是那几样，不推。推的空发布不含待发、不算用户改动，报在 [`ChangeAccepted::floor_raises_pushed_for_space`]，
    /// 没收下时在 [`AcceptRefused::floor_raises_pushed_by_this_accept`]。于是这一次调用可能写盘，至多推 B − 1 次空发布（每次一遍单元、记录、根与系统配置，一串多一遍系统配置）。
    ///
    /// 整理不在这里跑：发成之后起跑判据为真只记「该跑了」（[`MountedSession::publish_user_change`]），由后台入口去跑。
    ///
    /// # Errors
    /// [`AcceptRefused`] 的几种。
    pub fn accept<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        change: UserChange<'_>,
        now: CallerMonotonicClockReading,
    ) -> Result<ChangeAccepted, AcceptRefused> {
        self.note_the_reading_seen(now);
        if let Some(frozen) = self.session.allocator.frozen_publish() {
            return Err(
                AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {
                    checkpoint_txg: frozen.checkpoint_txg(),
                    instance: frozen.instance(),
                },
            );
        }
        if !self
            .session
            .parameters_and_device_table
            .is_the_device_table_of_the_mount(devices)
        {
            return Err(AcceptRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: self
                    .session
                    .parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        let (data_change, contract) = match ChangeAtTheAccept::of(change) {
            ChangeAtTheAccept::Barrier => return self.publish_a_barrier_alone(devices, change, now),
            ChangeAtTheAccept::WriteAtOffset => {
                return Err(
                    AcceptRefused::WriteAtOffsetIntoThePendingChangesNotSupportedInTheFirstVersion,
                )
            }
            ChangeAtTheAccept::DataChangesPublishedTogether => {
                return Err(AcceptRefused::DataChangesPublishedTogetherIntoThePendingChangesNotSupportedInTheFirstVersion)
            }
            ChangeAtTheAccept::Data { change, contract } => (change, contract),
        };
        let (journal_bytes_of_this_change, floor_raises_pushed_for_space) = self
            .judge_a_data_change_raising_the_floor_for_space(
                devices,
                change,
                data_change,
                contract,
            )?;
        self.pending
            .take_in(data_change, journal_bytes_of_this_change);
        let mut publishes = Vec::new();
        if let Some(cause) = self.trigger_cause_at(now) {
            match self.publish_the_pending_changes(
                devices,
                now,
                ReasonOfThePublish::TriggeredAfterTheAccept(cause),
            ) {
                Ok(Some(published)) => publishes.push(published),
                Ok(None) => {}
                Err(refusal) => {
                    return Err(AcceptRefused::AcceptedButTheTriggeredPublishRefused {
                        floor_raises_pushed_for_space,
                        refusal,
                    })
                }
            }
        }
        Ok(ChangeAccepted {
            taken: HowTheChangeWasTaken::KeptPending {
                journal_bytes_of_this_change,
            },
            publishes,
            floor_raises_pushed_for_space,
        })
    }

    /// 判 [`Self::judge_a_data_change_before_any_read_or_write`]；被空间拒就为被判的那一批推一串抬 F、再判（与 [`MountedSession::publish_user_change`] 被拒时走同一个
    /// [`push_one_floor_raise_for_a_refusal_for_space`]，上限与预算一样）。这一次改动算一次准入，推的数从 0 起。判过了交回这一次单独发布要写的 journal 字节与推成的几串。
    /// 推的那几串就地推进 `current`，待发不动。
    ///
    /// 迭代上界：每一轮要么判过交回，要么交回拒绝，要么推成一串（至少一次空发布，已推的数加上去，预算在 B − 1 次之内停），所以至多 8 轮。跨轮携带的是现行那一版与已推的几串。
    ///
    /// # Errors
    /// [`Self::judge_a_data_change_before_any_read_or_write`] 的几种（不是空间不够、又一串没推时原样）；[`AcceptRefused::RefusedAfterRaisingTheFloorForSpace`]。
    fn judge_a_data_change_raising_the_floor_for_space<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        change: UserChange<'_>,
        data_change: DataChangeAccepted<'_>,
        contract: WholeWriteContractAtTheAccept,
    ) -> Result<(JournalBytesToWrite, FloorRaisesPushedByTheAccept), AcceptRefused> {
        let mut published_in_this_change = PublishedInThisChange {
            floor_raises: Vec::new(),
            refusals_that_pushed_the_floor_raises: Vec::new(),
            versions_published: Vec::new(),
        };
        let mut publishes_pushed_in_this_admission = 0;
        loop {
            let (judgement, cause) = match self
                .judge_a_data_change_before_any_read_or_write(devices, data_change, contract)
            {
                Ok(journal_bytes_of_this_change) => {
                    return Ok((
                        journal_bytes_of_this_change,
                        FloorRaisesPushedByTheAccept {
                            floor_raises: published_in_this_change.floor_raises,
                            refusals_that_pushed_the_floor_raises: published_in_this_change
                                .refusals_that_pushed_the_floor_raises,
                        },
                    ))
                }
                Err(AcceptRefused::ChangeRefusedByThePublishJudgement(cause)) => {
                    (BatchJudgedAtTheAccept::ThisChangeAlone, *cause)
                }
                Err(AcceptRefused::PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(
                    cause,
                )) => (
                    BatchJudgedAtTheAccept::PendingChangesTogetherWithThisChange,
                    *cause,
                ),
                Err(
                    refused @ (AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
                    | AcceptRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
                    | AcceptRefused::NoFileVersionToChange
                    | AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion
                    | AcceptRefused::WriteAtOffsetIntoThePendingChangesNotSupportedInTheFirstVersion
                    | AcceptRefused::DataChangesPublishedTogetherIntoThePendingChangesNotSupportedInTheFirstVersion
                    | AcceptRefused::WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion { .. }
                    | AcceptRefused::WriteToAnInodeCreatedByThePendingChangesNotSupportedInTheFirstVersion { .. }
                    | AcceptRefused::RefusedAfterRaisingTheFloorForSpace { .. }
                    | AcceptRefused::PublishOfThePendingChangesBeforeTheBarrierRefused(_)
                    | AcceptRefused::BarrierRefused { .. }
                    | AcceptRefused::AcceptedButTheTriggeredPublishRefused { .. }),
                ) => return Err(refused),
            };
            if !refusal_is_short_of_space(&cause)
                && published_in_this_change.floor_raises.is_empty()
            {
                return Err(match judgement {
                    BatchJudgedAtTheAccept::ThisChangeAlone => {
                        AcceptRefused::ChangeRefusedByThePublishJudgement(Box::new(cause))
                    }
                    BatchJudgedAtTheAccept::PendingChangesTogetherWithThisChange => {
                        AcceptRefused::PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(
                            Box::new(cause),
                        )
                    }
                });
            }
            let (whole_write_together, new_inodes_together) =
                self.pending.together_with_this_change(data_change);
            let change_of_the_refused_batch = match judgement {
                BatchJudgedAtTheAccept::ThisChangeAlone => change,
                BatchJudgedAtTheAccept::PendingChangesTogetherWithThisChange => {
                    UserChange::DataChangesPublishedTogether(DataChangesPublishedTogether {
                        whole_write: whole_write_together,
                        new_inodes: &new_inodes_together,
                    })
                }
            };
            let remainder = RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(
                change_of_the_refused_batch,
            );
            let MountedSession {
                allocator,
                current,
                shadow_ledger,
                parameters_and_device_table,
                ..
            } = &mut self.session;
            let (PoolVersion::WithFile(version_of_trees)
            | PoolVersion::WithTreesAndNoFile(version_of_trees)) = &*current
            else {
                return Err(AcceptRefused::NoFileVersionToChange);
            };
            let mut version_pushed = Arc::clone(version_of_trees);
            let pushed = push_one_floor_raise_for_a_refusal_for_space(
                parameters_and_device_table.parameters_on_disk(),
                devices,
                allocator,
                &mut version_pushed,
                *shadow_ledger,
                remainder,
                // `accept` 不先走空（队列不空时在任何读写之前拒），被拒的就是判的那一批自己。
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::of_the_user_change(
                    change_of_the_refused_batch,
                ),
                publishes_pushed_in_this_admission,
                published_in_this_change,
                cause,
            );
            *current = PoolVersion::of_a_shared_version_of_trees(version_pushed);
            let publishes_of_this_raise;
            (published_in_this_change, publishes_of_this_raise) =
                pushed.map_err(
                    |refusal| AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
                        judgement,
                        refusal: Box::new(refusal),
                    },
                )?;
            publishes_pushed_in_this_admission += publishes_of_this_raise;
        }
    }

    /// [`Self::accept`] 收数据写类改动之前那几判，一个盘都不读、一个写都不发：交回这一次单独发布要写的 journal 字节。判的是此刻的现行那一版与分配器，被空间拒之后的处置在
    /// [`Self::judge_a_data_change_raising_the_floor_for_space`]。
    ///
    /// # Errors
    /// [`AcceptRefused::NoFileVersionToChange`]、`QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion`、
    /// `WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion`、`WriteToAnInodeCreatedByThePendingChangesNotSupportedInTheFirstVersion`、
    /// `ChangeRefusedByThePublishJudgement`、`PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement`。
    fn judge_a_data_change_before_any_read_or_write<Device: BlockDevice>(
        &self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        data_change: DataChangeAccepted<'_>,
        contract: WholeWriteContractAtTheAccept,
    ) -> Result<JournalBytesToWrite, AcceptRefused> {
        let current_version = match (&self.session.current, data_change) {
            (PoolVersion::WithFile(version), _)
            | (PoolVersion::WithTreesAndNoFile(version), DataChangeAccepted::NewInodes(_)) => {
                version
            }
            (PoolVersion::WithTreesAndNoFile(_), DataChangeAccepted::WholeWrite(_))
            | (PoolVersion::WithoutFile(_), _) => return Err(AcceptRefused::NoFileVersionToChange),
        };
        if !current_version.queue_of_committed_intents_is_empty() {
            return Err(AcceptRefused::QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion);
        }
        if let DataChangeAccepted::WholeWrite(whole_write) = data_change {
            if let Some(pending_whole_write) = &self.pending.whole_write {
                if pending_whole_write.inode != whole_write.inode {
                    return Err(
                        AcceptRefused::WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion {
                            inode_pending: pending_whole_write.inode,
                            inode_written: whole_write.inode,
                        },
                    );
                }
            }
            let first_number_created_by_the_pending_changes =
                current_version.inode_number_watermark();
            let is_created_by_the_pending_changes = whole_write.inode.0
                >= first_number_created_by_the_pending_changes
                && whole_write.inode.0 - first_number_created_by_the_pending_changes
                    < self.pending.new_inodes_in_total();
            if is_created_by_the_pending_changes {
                return Err(
                    AcceptRefused::WriteToAnInodeCreatedByThePendingChangesNotSupportedInTheFirstVersion {
                        inode: whole_write.inode,
                    },
                );
            }
            if contract == WholeWriteContractAtTheAccept::OneDataUnit
                && whole_write.content.len() > data_unit_payload_capacity()
            {
                return Err(AcceptRefused::ChangeRefusedByThePublishJudgement(Box::new(
                    PublishError::ContentExceedsDataUnit {
                        bytes: whole_write.content.len(),
                        capacity: data_unit_payload_capacity(),
                    },
                )));
            }
        }
        let parameters = self
            .session
            .parameters_and_device_table
            .parameters_on_disk();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        writer.set_container_tree_capacities(
            self.session
                .allocator
                .deletion_switches()
                .container_tree_capacities(),
        );
        let mut slot_of_the_new_inodes = [NewInodesOfOneChange {
            count: 0,
            write_time_seconds: 0,
        }];
        let journal_bytes_of_this_change =
            journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write(
                &writer,
                &self.session.allocator,
                current_version,
                data_change.alone(&mut slot_of_the_new_inodes),
                self.session.instance,
            )
            .map_err(|cause| AcceptRefused::ChangeRefusedByThePublishJudgement(Box::new(cause)))?;
        let (whole_write_together, new_inodes_together) =
            self.pending.together_with_this_change(data_change);
        journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write(
            &writer,
            &self.session.allocator,
            current_version,
            DataChangesPublishedTogether {
                whole_write: whole_write_together,
                new_inodes: &new_inodes_together,
            },
            self.session.instance,
        )
        .map_err(|cause| {
            AcceptRefused::PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(
                Box::new(cause),
            )
        })?;
        Ok(journal_bytes_of_this_change)
    }

    /// 屏障进门：先排空待发、再单发。
    fn publish_a_barrier_alone<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        change: UserChange<'_>,
        now: CallerMonotonicClockReading,
    ) -> Result<ChangeAccepted, AcceptRefused> {
        let mut publishes = Vec::new();
        if let Some(drained) = self
            .publish_the_pending_changes(
                devices,
                now,
                ReasonOfThePublish::PendingChangesPublishedBeforeABarrier,
            )
            .map_err(AcceptRefused::PublishOfThePendingChangesBeforeTheBarrierRefused)?
        {
            publishes.push(drained);
        }
        match self.session.publish_user_change(devices, change) {
            Ok(published) => {
                self.note_a_publish(now);
                publishes.push(PublishReportedByTheSession {
                    reason: ReasonOfThePublish::BarrierPublishedAlone,
                    published,
                });
                Ok(ChangeAccepted {
                    taken: HowTheChangeWasTaken::PublishedAloneAsABarrier,
                    publishes,
                    // 屏障为空间推的抬 F 报在它自己那一次发布里（`UserChangePublished::floor_raises`）。
                    floor_raises_pushed_for_space: FloorRaisesPushedByTheAccept {
                        floor_raises: Vec::new(),
                        refusals_that_pushed_the_floor_raises: Vec::new(),
                    },
                })
            }
            Err(refusal) => Err(AcceptRefused::BarrierRefused {
                publishes_before_the_refusal: publishes,
                refusal: Box::new(refusal),
            }),
        }
    }

    /// 调用方按周期调（D16（发布语义） 已定项 2、5：丢失窗口里的 P 是这个周期，归调用方）：判一次触发，触发就把待发整个并成一次发布。
    /// 待发空、或两支都没到，就什么都不发、交回 `None`，一个盘都不读。
    ///
    /// # Errors
    /// 触发的那一次没发成：[`PublishOfThePendingChangesRefused`]。
    pub fn publish_when_due<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        now: CallerMonotonicClockReading,
    ) -> Result<Option<PublishReportedByTheSession>, PublishOfThePendingChangesRefused> {
        self.note_the_reading_seen(now);
        let Some(cause) = self.trigger_cause_at(now) else {
            return Ok(None);
        };
        self.publish_the_pending_changes(devices, now, ReasonOfThePublish::TriggeredWhenDue(cause))
            .map_err(|refusal| PublishOfThePendingChangesRefused { refusal })
    }

    /// 立即排空（D16（发布语义） 已定项 11：只是提前触发一次发布，同一个 journal、同一条发布路径）：待发非空就整个并成一次发布；待发空就什么都不发、交回 `None`
    /// （不推空发布）。冻结着一次没重发的发布时在任何读写之前拒（D16（发布语义） 已定项 7「fsync 等系统配置轮换持久之后才返回」）：待发空着也不答 `None`，
    /// 收下的改动可能正并在冻结的那一次里。调用方要做的是先重发（[`Self::resend_the_frozen_publish`]）、发成了再调。
    ///
    /// # Errors
    /// 同 [`Self::publish_when_due`]；冻结着：`refusal` 是 [`UserChangeRefused::Publish`]，`cause` 是 [`PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet`]。
    pub fn publish_pending_now<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        now: CallerMonotonicClockReading,
    ) -> Result<Option<PublishReportedByTheSession>, PublishOfThePendingChangesRefused> {
        self.note_the_reading_seen(now);
        let frozen_publish = self.session.allocator.frozen_publish();
        if let Some(frozen) = frozen_publish {
            return Err(PublishOfThePendingChangesRefused {
                refusal: Box::new(UserChangeRefused::Publish {
                    cause: PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                        checkpoint_txg: frozen.checkpoint_txg(),
                        instance: frozen.instance(),
                    },
                    floor_raises: Vec::new(),
                    versions_published: Vec::new(),
                }),
            });
        }
        self.publish_the_pending_changes(
            devices,
            now,
            ReasonOfThePublish::PendingChangesPublishedOnRequest,
        )
        .map_err(|refusal| PublishOfThePendingChangesRefused { refusal })
    }

    /// 原入口（[`MountedSession::publish_user_change`]）语义不变，只多一条先排空待发：待发非空就先并成一次发布，再交这一次改动（任何一种）单发。
    ///
    /// # Errors
    /// 排空或这一次没发成：[`UserChangeAfterThePendingChangesRefused`]（排空发成了的那一次在里面）。
    pub fn publish_user_change<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        change: UserChange<'_>,
        now: CallerMonotonicClockReading,
    ) -> Result<Vec<PublishReportedByTheSession>, UserChangeAfterThePendingChangesRefused> {
        self.note_the_reading_seen(now);
        let mut publishes = Vec::new();
        match self.publish_the_pending_changes(
            devices,
            now,
            ReasonOfThePublish::PendingChangesPublishedBeforeTheUserChange,
        ) {
            Ok(Some(drained)) => publishes.push(drained),
            Ok(None) => {}
            Err(refusal) => {
                return Err(UserChangeAfterThePendingChangesRefused {
                    publishes_before_the_refusal: publishes,
                    refusal,
                })
            }
        }
        match self.session.publish_user_change(devices, change) {
            Ok(published) => {
                self.note_a_publish(now);
                publishes.push(PublishReportedByTheSession {
                    reason: ReasonOfThePublish::UserChangePublishedAlone,
                    published,
                });
                Ok(publishes)
            }
            Err(refusal) => Err(UserChangeAfterThePendingChangesRefused {
                publishes_before_the_refusal: publishes,
                refusal: Box::new(refusal),
            }),
        }
    }

    /// 挂着切换后台计算的设置（[`MountedSession::switch_background_compute_from_the_next_batch`]）：只记成待生效、不读写盘，所以不排空待发。
    ///
    /// # Errors
    /// 同 [`MountedSession::switch_background_compute_from_the_next_batch`]。
    pub fn switch_background_compute_from_the_next_batch(
        &mut self,
        background_compute_settings: BackgroundComputeSettings,
        gpu_compute: Option<BackgroundGpuComputeHandle>,
    ) -> Result<(), BackgroundComputeSettingsRefused> {
        self.session
            .switch_background_compute_from_the_next_batch(background_compute_settings, gpu_compute)
    }

    /// 同 [`Self::switch_background_compute_from_the_next_batch`]，连同 GC 停着时在 CPU 上补跑的阈值一起换
    /// （[`MountedSession::switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch`]）。
    ///
    /// # Errors
    /// 同 [`MountedSession::switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch`]。
    pub fn switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
        &mut self,
        background_compute_settings: BackgroundComputeSettings,
        gpu_compute: Option<BackgroundGpuComputeHandle>,
        gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
    ) -> Result<(), BackgroundComputeSettingsRefused> {
        self.session
            .switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
                background_compute_settings,
                gpu_compute,
                gc_cpu_compensation_threshold,
            )
    }

    /// 重发冻结着的那一次发布（[`MountedSession::resend_the_frozen_publish`]）。没有冻结着的发布交回 `false`，什么都不写。
    ///
    /// # Errors
    /// 同 [`MountedSession::resend_the_frozen_publish`]。
    pub fn resend_the_frozen_publish<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
    ) -> Result<bool, PublishError> {
        self.session.resend_the_frozen_publish(devices)
    }

    /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：底下那个会话（可改），用例摆开关与预算。产品路径一处都不调它：待发与会话私有的不变量靠调用方不改待发。
    pub fn session_for_tests_only(&mut self) -> &mut MountedSession {
        &mut self.session
    }

    /// 交出会话（卸载、回退、重发冻结的那一次要的是会话本身）：先把待发排空，交回排空那一次（待发空时 `None`）与会话。类型上卸载、回退之前待发一定空了
    /// （判决第二节「一次发布里攒什么」那一格：回退、卸载是屏障）。
    ///
    /// # Errors
    /// 排空没发成：[`HandingBackTheSessionRefused`]，攒批的会话原样交回（没冻结时待发还在里面）。
    pub fn into_session_after_publishing_the_pending_changes<Device: BlockDevice>(
        mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        now: CallerMonotonicClockReading,
    ) -> Result<
        (Option<PublishReportedByTheSession>, MountedSession),
        Box<HandingBackTheSessionRefused>,
    > {
        self.note_the_reading_seen(now);
        match self.publish_the_pending_changes(
            devices,
            now,
            ReasonOfThePublish::PendingChangesPublishedBeforeHandingBackTheSession,
        ) {
            Ok(drained) => Ok((drained, self.session)),
            Err(refusal) => Err(Box::new(HandingBackTheSessionRefused {
                session_with_pending_changes: self,
                refusal,
            })),
        }
    }
}
