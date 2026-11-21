//! 一次发布的落盘：要写的单元与 journal 记录、写到一半冻住的发布与它的重发、根槽写之后的系统配置轮换。

use super::*;

/// 一次发布要交给设备的全部字节，按 D16（发布语义） 已定项 7 的持久顺序排好：这次重写的单元 → 屏障 → journal 记录 → 屏障 →
/// 根槽 FUA → 系统配置槽轮换 → 屏障。三条发布路径（带单元的、零单元的、树表 0 条上写行的）都先装成它、再交给
/// [`persist_publish_writes`] 落盘；落盘中途失败的那一次原样冻结着它，重发时照它逐字节再发一遍
/// （D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：checkpoint_txg、计数器、本次发布内序号、记录标志、
/// 单元的位置与字节都不变）。系统配置槽不在里面：它的世代号按写那一刻盘上自证过的槽现算（D22（单元原子性怎么合成） 已定项 16，逐盘计），
/// 内容是 tail、实例代号与写之前现算的整池 F 生效值。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishWrites {
    /// 这次重写的单元，按写的次序（bump 次序）。零单元发布是空的。
    pub units: Vec<PublishedUnit>,
    /// 这次发布的 journal 记录，按计数器升序。
    pub records: Vec<JournalRecordWrite>,
    pub checkpoint_txg: CheckpointTxg,
    /// 根槽 FUA 写的整槽字节。
    pub root_slot: Vec<u8>,
    /// 系统配置里的 tail：这次发布末条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 18）。
    pub journal_tail: u64,
    pub journal_instance: InstanceGeneration,
}

/// 一条要写进 journal 环的记录：计数器（定落在环里哪一槽）与整条 4096 字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalRecordWrite {
    pub counter: u64,
    pub bytes: Vec<u8>,
}

/// 把一次发布的字节按持久顺序交给设备（D16（发布语义） 已定项 7）。三条发布路径与重发冻结的那一次共用这一处——
/// 各抄一份会分叉，而「根槽在系统配置槽之前」正是崩溃窗口那几格的前提。
///
/// # Errors
/// 块设备报的第一个错原样交回，之后的步骤一步都不发。
pub(super) fn persist_publish_writes<Device: BlockDevice>(
    writer: &mut PoolWriter<'_, Device>,
    writes: &PublishWrites,
) -> Result<(), BlockDeviceError> {
    for unit in &writes.units {
        writer.perform_commit_step(CommitStep::WriteUnitToEveryDevice {
            slot: unit.slot,
            unit: &unit.bytes,
            identity: unit.identity,
        })?;
    }
    writer.perform_commit_step(CommitStep::Barrier)?;
    for record in &writes.records {
        writer.perform_commit_step(CommitStep::WriteJournalRecordToEveryDevice {
            counter: record.counter,
            record: &record.bytes,
        })?;
    }
    writer.perform_commit_step(CommitStep::Barrier)?;
    persist_the_root_then_rotate_the_system_configuration(
        writer,
        writes.checkpoint_txg,
        &writes.root_slot,
        writes.journal_tail,
        writes.journal_instance,
    )
}

/// 落盘阶段中途失败、冻结着等原样重发的那一次发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：发布不接受失败，
/// 失败的那次冻结，下一次发布之前逐字节原样重发它，重发成功才建下一次发布）。住在分配器上
/// （`PoolAllocator::frozen_publish`）：每一次发布都要交分配器进来，冻结着的时候发布路径在任何读写之前拒绝
/// （`PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet`），只有 [`resend_the_frozen_publish`] 清得掉它。
/// 于是同一 (实例代号, checkpoint_txg) 不会先后写出两次不同的发布、留下两条带末条标志的记录。
///
/// 零单元发布（`publish_without_units`）不经分配器、不冻结：它的调用方（暖机、可写挂载取号之后那一串）一失败就整个挂载返回错误，
/// 下一次发布在下一次挂载里、换了实例代号。
#[derive(Clone, Debug)]
pub struct FrozenPublish {
    pub(super) writes: PublishWrites,
    /// 这次发布是哪一种（[`KindOfTheFrozenPublish`]）：冻结在发布路径里定不了，由发起它的会话记上。重发成了，会话照它记账。
    pub(super) kind: KindOfTheFrozenPublish,
    /// 重发成功之后交出去的那一版；它的写账在重发那一刻现记（`writes` 字段），这里是空账。
    pub(super) version: PoolVersion,
    /// 这次发布成立之后的分配器（释放、取落点、记根都做完的那一份）。冻结期间调用方手里的分配器回到发布之前的样子，
    /// 重发成功时整个换成它——冻结期间对那个分配器做的别的改动随之丢掉（发布路径冻结期间一个都不许进来）。
    allocator_after_the_publish: PoolAllocator,
}

impl FrozenPublish {
    #[must_use]
    pub fn checkpoint_txg(&self) -> CheckpointTxg {
        self.writes.checkpoint_txg
    }

    #[must_use]
    pub fn instance(&self) -> InstanceGeneration {
        self.writes.journal_instance
    }

    /// 重发时要逐字节再发一遍的那些字节。
    #[must_use]
    pub fn writes(&self) -> &PublishWrites {
        &self.writes
    }

    #[must_use]
    pub fn kind(&self) -> KindOfTheFrozenPublish {
        self.kind
    }

    /// 同一次发布，种类换成发起它的会话说的那一种。
    #[must_use]
    pub(crate) fn of_the_kind(self, kind: KindOfTheFrozenPublish) -> Self {
        Self { kind, ..self }
    }
}

/// 冻结的那一次发布重发成了，会话要怎么记账（D26（后台整理与放置回收） 已定项 13 第 6 条）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KindOfTheFrozenPublish {
    /// 用户改动的发布，或这次用户改动之后接着走的删除步：这次改动已经算发成，重给一份预算、清上一条完成的意图。
    UserChange,
    /// 整理的一次发布：扣预算、记一个根槽。
    CompactionPublish,
    /// 两样都不记：删除续做的一步、推抬 F 的一次发布，与发布路径刚冻结、还没有会话说种类的。
    NeitherOfTheCountedKinds,
}

/// 发布路径的第一道：分配器上冻结着一次没重发的发布，就在任何读写之前拒绝（D23（journal 的角色与格式） 已定项 14
/// 「这一版的失败处置」：重发成功才建下一次发布）。
///
/// # Errors
/// `PublishFrozenAfterAWriteFailureIsNotResentYet`，带着冻结的那一次的 checkpoint_txg 与实例代号。
pub(super) fn refuse_while_a_publish_is_frozen(
    allocator: &PoolAllocator,
) -> Result<(), PublishError> {
    match allocator.frozen_publish() {
        None => Ok(()),
        Some(frozen) => Err(
            PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                checkpoint_txg: frozen.checkpoint_txg(),
                instance: frozen.instance(),
            },
        ),
    }
}

/// 把装好、还没落盘的一次发布落盘（经分配器的两条发布路径共用：带文件的一版、树表 0 条的一版上写行）：
/// 成功就把这次落盘的写账填进交回的那一版；落盘那几步里失败，这次已记的写进写入口的失败账（增补 2 第 20b 行），
/// 分配器换回发布之前的样子，这次发布冻结在它上面等原样重发（[`FrozenPublish`]）。
///
/// # Errors
/// 块设备报的错（`PublishError::BlockDevice`），这时这次发布已冻结。
pub(super) fn persist_the_publish_or_freeze_it<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    allocator_before_this_publish: PoolAllocator,
    writes: PublishWrites,
    version: PoolVersion,
) -> Result<PoolVersion, PublishError> {
    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
    let persisted = persist_publish_writes(pool, &writes);
    // 账各份分配器拷贝共用：落盘中途失败、分配器换回发布之前那一份，读到的照样在账上（读是真发到盘上的）。
    pool.count_the_reads_before_the_system_configuration_writes(allocator);
    if let Err(cause) = persisted {
        pool.count_failed_publish(&writes_before_this_publish);
        let allocator_after_the_publish =
            std::mem::replace(allocator, allocator_before_this_publish);
        allocator.freeze_publish(FrozenPublish {
            writes,
            kind: KindOfTheFrozenPublish::NeitherOfTheCountedKinds,
            version,
            allocator_after_the_publish,
        });
        return Err(PublishError::BlockDevice(cause));
    }
    Ok(version.with_writes(
        pool.writes_by_structure_kind
            .since(&writes_before_this_publish),
    ))
}

/// 把分配器上冻结着的那次发布逐字节原样重发（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）：单元、journal 记录、
/// 根槽照冻结时装好的字节再发一遍（checkpoint_txg、计数器、本次发布内序号、记录标志、单元的位置都不变），系统配置槽照常轮换。
/// 成功就把分配器换成这次发布成立之后的那一份、交回那一版（写账是重发这一遍真写出去的）；没有冻结着的发布就什么都不写、交回 `None`。
///
/// # Errors
/// 重发那几步里又有一步报错（`PublishError::BlockDevice`）：这一遍已记的写进失败账，那次发布照旧冻结着。
pub fn resend_the_frozen_publish<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
) -> Result<Option<PoolVersion>, PublishError> {
    let Some(frozen) = allocator.take_frozen_publish() else {
        return Ok(None);
    };
    let writes_before_the_resend = pool.writes_by_structure_kind.clone();
    let persisted = persist_publish_writes(pool, &frozen.writes);
    pool.count_the_reads_before_the_system_configuration_writes(allocator);
    if let Err(cause) = persisted {
        pool.count_failed_publish(&writes_before_the_resend);
        allocator.freeze_publish(frozen);
        return Err(PublishError::BlockDevice(cause));
    }
    let FrozenPublish {
        writes: _,
        kind: _,
        version,
        allocator_after_the_publish,
    } = frozen;
    *allocator = allocator_after_the_publish;
    Ok(Some(
        version.with_writes(
            pool.writes_by_structure_kind
                .since(&writes_before_the_resend),
        ),
    ))
}

/// 发布的最后三步（D16（发布语义） 已定项 7 的持久顺序）：根槽 FUA 写 → 系统配置槽轮换 → 屏障，屏障完成才返回。
/// 三条发布路径（带单元的、零单元的、树表 0 条上只写实例表的）与重发冻结的那一次共用这一处，不各抄一份——
/// 抄出来的几份会分叉，而「根槽在系统配置槽之前」「轮换持久之后才返回」正是崩溃窗口那几格的前提。
///
/// 末尾那道屏障（C577（系统配置没见证到的最新根，乙罩不到），用户 2026-09-27 定「发布返回前加屏障」）：
/// 没有它，发布返回时轮换还可能在写缓存里，崩溃之后系统配置只见证到上一次发布；这次的根槽一时读不出，可写挂载的见证判据
/// （D23（journal 的角色与格式） 已定项 14「可写挂载读到的更新状态读不出」）就看不出有更新的状态、把这次发布当被抛弃，
/// 它引用的单元再发出去。屏障报错与轮换报错同一格：这次发布失败，由调用方冻结或整个挂载返回错误
/// （D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）。
fn persist_the_root_then_rotate_the_system_configuration<Device: BlockDevice>(
    writer: &mut PoolWriter<'_, Device>,
    checkpoint_txg: CheckpointTxg,
    root_slot: &[u8],
    journal_tail: u64,
    journal_instance: InstanceGeneration,
) -> Result<(), BlockDeviceError> {
    writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {
        checkpoint_txg,
        root_slot,
    })?;
    writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {
        journal_tail,
        journal_instance,
    })?;
    writer.perform_commit_step(CommitStep::Barrier)
}
