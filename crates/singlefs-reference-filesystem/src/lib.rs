//! RefFS：语义级的参照文件系统。只在内存里按 POSIX 操作走，判一次崩溃之后恢复读出的内容落不落在允许集合里。
//!
//! 依据：`.claude/kb/decisions/13-验证路线.md@已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义`（不与实现共用崩溃点枚举的那段代码）；
//! `.claude/kb/decisions/17-实现分层与第三方管道.md@已定项 4：入口 = RefFS 的操作面`（fsync 的持久性承诺由 RefFS 的崩溃语义定义）。
//! 形态是用户 2026-10-02 的定案：语义级（`research/prompts/m3-formal-7-forks.md` Y2）；按发布边界的前缀 Q、另起、单独一个包
//! （`research/prompts/m3-formal-r4-main-verification.md` 用户定案 F1、F3、F4）；回退目标作输入
//! （`research/prompts/m3-formal-r5-main-verification.md` 用户定案 G2）。条款候选取那份判决第二节 U1、U2、U4 与第四节第 1、2、4、8 条（零轮）。
//! 回退怎么算是 r6 的用户定案 H1-收严（`research/prompts/m3-formal-r6-main-verification.md` 用户定案一节）：返回成功的发布调用、fsync、
//! 回退都抬 lo；回退算一次已确认发布、它的下标算发布末尾；回退目标 m 是版本号，版本含每次恢复读出、重新起头的那一份；
//! 操作结局多「原样重发」一种。条款句取那份判决第二节 U2、X2 两行与第四节（零轮）。
//!
//! 输入只有五样（r6 判决第二节 X2 一行的白名单），都由调用方交入：每个操作的结局（成、在任何写之前拒、写了之后报错、原样重发）、
//! 崩溃落在第几个操作里（hi；按切开当前段的那道屏障由哪一步发出归属，归属由调用方做）、回退目标 m（版本号）、每次恢复读出的内容、
//! 显式发布点（发布调用与 fsync 这两种操作本身）。这个包不读盘、不读录制流、不枚举崩溃点，依赖只有 std。
//! 只管串行提交；任意子集的丢法不做（r5 判决第四节第 8 条：串行提交下出局）。自动触发的攒批发布落地之前只接显式发布点（r6 判决第二节 U2 一行）。
#![forbid(unsafe_code)]

/// 一个文件的全部字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileContent(pub Vec<u8>);

/// POSIX 面上看得见的状态。第一版的操作只碰一个文件：还没有文件，或者有这一个文件与它的全部字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VisibleState {
    NoFile,
    OneFile(FileContent),
}

/// 第一版的操作。崩溃不在这里，它是 [`ReferenceFileSystem::crash_and_recover`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    CreateTheFirstFile(FileContent),
    OverwriteTheFile(FileContent),
    /// 发布调用：一个显式发布点；返回成功时抬起下界 lo。
    ExplicitPublish,
    /// 一个显式发布点；返回成功时抬起下界 lo。
    Fsync,
    /// 回退到第 m 版的内容；m 由调用方按实现报的回退根在「根 → 版本」表里查出来，选得对不对这里不判。
    /// 回退是挂着时的一次向前发布：返回成功时抬起下界 lo，它的下标是发布末尾，它之后的内容是一个新版本。
    RollBackToVersion(VersionNumber),
}

/// 调用方交入的结局。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationOutcome {
    Succeeded,
    /// 在任何写之前拒：这一步没发生，不进操作序列、不占号。
    RefusedBeforeAnyWrite,
    /// 写了之后报错：这一步做没做成两可。
    FailedAfterWriting,
    /// 落盘途中失败、冻结之后原样重发成功：这一步做成了，与「成」一样抬 lo、算已确认。
    SucceededOnAVerbatimResendAfterFailingWhileWriting,
}

/// 上一次恢复（还没崩溃过时是起点）之后进了操作序列的第几个操作，从 1 起；在任何写之前被拒的不占号。
/// 崩溃本身也是一个操作，号是序列长度 + 1。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperationNumber(pub usize);

impl OperationNumber {
    /// 第 k 个操作施加之后的状态是 S_k。
    pub fn state_after(self) -> StateIndex {
        StateIndex(self.0)
    }
}

/// 状态下标 j：S_j 是上一次恢复之后前 j 个操作施加出来的状态，S_0 是上一次恢复读出的内容（还没崩溃过时是空池）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateIndex(pub usize);

/// RefFS 历史里第几版，从 1 起，按历史次序编、跨崩溃累计：第 1 版是起点（新格式化的空池）；之后每一次已确认的发布
/// （返回成功的发布调用、fsync、回退）之后的内容，与每一次恢复读出、以它重新起头的那一份，各占一个号。
/// 调用方把实现写下的每一条根（发布调用、回退、可写挂载的写行与暖机、抬 F、卸载）映到写它时的现行版本（r6 用户定案 H1-收严）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VersionNumber(pub usize);

/// 交进来的操作进没进操作序列。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmittedOperation {
    Entered(OperationNumber),
    DidNotHappen,
}

/// 交进来的操作不收；返回它时模型一个字段都没动。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationNotAccepted {
    /// 已经有文件时再建第一个文件：条款没写它在 POSIX 面上是什么，第一版不支持。
    CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion,
    /// 还没有文件时覆盖写：条款没写它在 POSIX 面上是什么，第一版不支持。
    OverwritingWithoutAFileIsNotInTheFirstVersion,
    /// 回退目标 m 不是 1 到「到这一步为止的版本数」之间的数。
    RollBackTargetIsNotAVersion {
        requested: VersionNumber,
        versions_so_far: usize,
    },
}

/// 允许集合里的一个状态与它的下标。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllowedState {
    pub index: StateIndex,
    pub state: VisibleState,
}

/// 一次崩溃之后恢复允许读出的状态：{S_j : j = lo 或 j 恰是某次显式发布或回退的末尾，lo ≤ j ≤ hi}，按下标升序。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllowedSet {
    pub lower_bound: StateIndex,
    pub upper_bound: StateIndex,
    pub allowed: Vec<AllowedState>,
}

impl AllowedSet {
    pub fn contains(&self, state: &VisibleState) -> bool {
        self.allowed
            .iter()
            .any(|allowed_state| &allowed_state.state == state)
    }
}

/// 这次崩溃判不了或判红；返回它时模型一个字段都没动。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CrashNotAccepted {
    /// 崩溃落在的操作号不在 1 到崩溃本身的号（序列长度 + 1）之间。
    CrashOperationIsOutsideTheOperationsSinceTheLastRecovery {
        crash_falls_in: OperationNumber,
        crash_operation_number: OperationNumber,
    },
    /// 一个操作写了之后报错，到崩溃为止它后面又起了操作：条款只说「之后的操作接在下一次恢复读出的状态上判」，
    /// 没写报错与崩溃之间起的操作算什么，第一版不支持。
    OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion {
        failed_after_writing: OperationNumber,
        operation_after_it: OperationNumber,
    },
    /// 恢复读出的内容不在允许集合里：判红。
    RecoveredStateOutsideTheAllowedSet {
        allowed: AllowedSet,
        recovered: VisibleState,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnteredOutcome {
    Succeeded,
    FailedAfterWriting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnteredKind {
    ContentChange,
    ExplicitPublish,
    Fsync,
    RollBack,
}

impl EnteredKind {
    /// 这个操作施加之后的下标是不是一次显式发布的末尾：发布带走它之前取号成功的全部操作，末尾就是它自己的下标。
    fn ends_a_publish(self) -> bool {
        match self {
            // 发布边界只取历史里的显式发布点：发布调用与 fsync（r5 判决第二节 U2 ①）；fsync 触发的发布也是发布：
            // .claude/kb/decisions/16-发布语义.md@已定项 6：发布计数器，四个等号写成一条
            EnteredKind::ExplicitPublish | EnteredKind::Fsync => true,
            // 回退是挂着时的一次向前发布：.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
            EnteredKind::RollBack => true,
            EnteredKind::ContentChange => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EnteredOperation {
    kind: EnteredKind,
    outcome: EnteredOutcome,
    state_after: VisibleState,
}

/// 序列里已确认的发布（返回成功的发布调用、fsync 与回退，原样重发成功的也算）各自发布之后的内容，按历史次序。
fn confirmed_publishes_among(
    operations: &[EnteredOperation],
) -> impl Iterator<Item = &VisibleState> {
    operations
        .iter()
        // 写了之后报错的发布两可，不算已确认：.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
        .filter(|operation| {
            operation.kind.ends_a_publish() && operation.outcome == EnteredOutcome::Succeeded
        })
        .map(|operation| &operation.state_after)
}

/// 允许集合 Q。调用方已核过 1 ≤ hi ≤ 序列长度 + 1。
fn allowed_set_for_a_crash_in(
    state_at_the_last_recovery: &VisibleState,
    operations: &[EnteredOperation],
    crash_falls_in: OperationNumber,
) -> AllowedSet {
    let state_at = |index: StateIndex| -> VisibleState {
        match index.0.checked_sub(1) {
            None => state_at_the_last_recovery.clone(),
            Some(position) => operations
                .get(position)
                .expect("下标不超过 hi − 1，而调用方核过 hi ≤ 序列长度 + 1")
                .state_after
                .clone(),
        }
    };
    // 崩溃落在第 hi 个操作里：前 hi − 1 个返回过；第 hi 个没返回，它是发布调用、fsync 或回退也不抬下界
    // （r5 判决第二节 U1：返回成功的才抬）。
    let returned_before_the_crash = &operations[..crash_falls_in.0 - 1];
    // lo = 崩溃之前最后一次返回成功的发布调用、fsync 或回退的末尾（r6 用户定案 H1-收严：三者同一个返回条件——
    // .claude/kb/decisions/16-发布语义.md@已定项 7：发布的持久顺序 里「fsync 等系统配置轮换持久之后才返回」与回退
    // 「它的根持久之后才向管理员确认，与 fsync 同一个返回条件」）。位置 p（从 0 起）上的是第 p + 1 个操作，施加之后是 S_{p+1}；
    // 发布调用与 fsync 不改内容，S_{p+1} 与调用之前那一刻相同。没有时 lo = 0，起点是上一次恢复读出的内容。
    // 报错时下界停在上一次返回成功的那一个：.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
    let lower_bound = returned_before_the_crash
        .iter()
        .enumerate()
        .rev()
        .find(|(_, operation)| {
            operation.outcome == EnteredOutcome::Succeeded && operation.kind.ends_a_publish()
        })
        .map_or(StateIndex(0), |(position, _)| StateIndex(position + 1));
    let upper_bound = crash_falls_in.state_after();
    // Q：j 只取 lo 与恰是某次显式发布末尾的下标，lo ≤ j ≤ hi（用户定案 F1）。一次发布整体施加或整体不施加，
    // 切开一次发布的 S_j 不在集合里：.claude/kb/decisions/16-发布语义.md@已定项 4：一次发布整体施加或整体不施加，第二份 replay 停在句法层；
    // .claude/kb/decisions/16-发布语义.md@已定项 12：中间状态的合法性语义改写成「根 + journal 任意前缀」；
    // 单元级的切分（.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
    // 「崩溃原子性的单位是一个单元」）落在一次发布里面，不在发布末尾，Q 里不出现。
    // 判的是状态下标，不是「lo 之前的操作必须在」：回退会让早先的操作合法消失。
    let mut allowed = vec![AllowedState {
        index: lower_bound,
        state: state_at(lower_bound),
    }];
    for (position, operation) in operations
        .iter()
        .enumerate()
        .take(upper_bound.0)
        .skip(lower_bound.0)
    {
        // 位置 position 上的是第 position + 1 个操作，施加之后是 S_{position+1}，lo < position + 1 ≤ hi。
        // 写了之后报错的发布也算末尾：那次发布可在可不在（.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么）
        if operation.kind.ends_a_publish() {
            allowed.push(AllowedState {
                index: StateIndex(position + 1),
                state: operation.state_after.clone(),
            });
        }
    }
    AllowedSet {
        lower_bound,
        upper_bound,
        allowed,
    }
}

/// 参照文件系统：一段串行提交的历史，按上一次恢复切成一轮一轮。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceFileSystem {
    /// S_0：上一次恢复读出的内容；还没崩溃过时是空池。
    state_at_the_last_recovery: VisibleState,
    /// 上一次恢复之后进了序列的操作；第 k 个在位置 k − 1。
    operations_since_the_last_recovery: Vec<EnteredOperation>,
    /// 上一次恢复为止的版本，按历史次序：起点、之前各轮里已确认的发布之后的内容、各次恢复读出的那一份；第 m 版在位置 m − 1。
    versions_up_to_the_last_recovery: Vec<VisibleState>,
}

impl ReferenceFileSystem {
    pub fn on_a_freshly_formatted_pool() -> Self {
        Self {
            state_at_the_last_recovery: VisibleState::NoFile,
            operations_since_the_last_recovery: Vec::new(),
            // 起点是第 1 版：mkfs 写的根与 mkfs 之后可写挂载写的根映到它（r6 用户定案 H1-收严「m 的编号覆盖每一个写根的入口」）
            versions_up_to_the_last_recovery: vec![VisibleState::NoFile],
        }
    }

    /// 到这一刻为止的版本，按历史次序（第 m 版在位置 m − 1）。
    fn versions_so_far(&self) -> Vec<&VisibleState> {
        self.versions_up_to_the_last_recovery
            .iter()
            .chain(confirmed_publishes_among(
                &self.operations_since_the_last_recovery,
            ))
            .collect()
    }

    /// 到这一刻为止最新的版本号：调用方把这一刻写下的根映到它。
    pub fn newest_version(&self) -> VersionNumber {
        VersionNumber(self.versions_so_far().len())
    }

    /// 崩溃落在崩溃本身里（序列里的操作都返回过之后才崩）时 hi 取的号：序列长度 + 1。
    pub fn crash_after_every_operation_returned(&self) -> OperationNumber {
        OperationNumber(self.operations_since_the_last_recovery.len() + 1)
    }

    /// 序列里的操作全都做成时的状态。
    fn state_after_every_operation_so_far(&self) -> &VisibleState {
        self.operations_since_the_last_recovery
            .last()
            .map_or(&self.state_at_the_last_recovery, |operation| {
                &operation.state_after
            })
    }

    /// 回退的目标内容 = 第 m 版的内容（用户定案 G2，m 的编号照 r6 用户定案 H1-收严）。
    fn content_of_version(
        &self,
        number: VersionNumber,
    ) -> Result<VisibleState, OperationNotAccepted> {
        let versions = self.versions_so_far();
        let not_a_version = OperationNotAccepted::RollBackTargetIsNotAVersion {
            requested: number,
            versions_so_far: versions.len(),
        };
        // 第 m 版从 1 起数，在第 m − 1 位：.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
        // （回退的新根取 R_old 那一版的用户可见树）
        let Some(position) = number.0.checked_sub(1) else {
            return Err(not_a_version);
        };
        match versions.get(position) {
            Some(state) => Ok((*state).clone()),
            None => Err(not_a_version),
        }
    }

    /// 交入一个操作与调用方看到的结局。
    ///
    /// # Errors
    /// [`OperationNotAccepted`] 的三种，返回时模型不动。
    pub fn submit(
        &mut self,
        operation: Operation,
        outcome: OperationOutcome,
    ) -> Result<SubmittedOperation, OperationNotAccepted> {
        let entered_outcome = match outcome {
            // 在任何写之前拒＝这一步没发生：不进序列、不占号（r5 判决第二节 U1「已接受」= 取号成功、没被拒）；
            // .claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换（「在任何写之前拒」）
            OperationOutcome::RefusedBeforeAnyWrite => return Ok(SubmittedOperation::DidNotHappen),
            OperationOutcome::Succeeded => EnteredOutcome::Succeeded,
            // 原样重发成功＝做成：.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
            // （「这一版的失败处置」：重发成功才建下一次发布）；r6 判决第二节「G2 回退被拒」一行补的结局
            OperationOutcome::SucceededOnAVerbatimResendAfterFailingWhileWriting => {
                EnteredOutcome::Succeeded
            }
            // 写了之后报错＝两可：.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
            OperationOutcome::FailedAfterWriting => EnteredOutcome::FailedAfterWriting,
        };
        let state_so_far = self.state_after_every_operation_so_far();
        let (kind, state_after) = match operation {
            Operation::CreateTheFirstFile(content) => match state_so_far {
                VisibleState::NoFile => (EnteredKind::ContentChange, VisibleState::OneFile(content)),
                VisibleState::OneFile(_) => return Err(OperationNotAccepted::CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion),
            },
            Operation::OverwriteTheFile(content) => match state_so_far {
                VisibleState::NoFile => return Err(OperationNotAccepted::OverwritingWithoutAFileIsNotInTheFirstVersion),
                VisibleState::OneFile(_) => (EnteredKind::ContentChange, VisibleState::OneFile(content)),
            },
            Operation::ExplicitPublish => (EnteredKind::ExplicitPublish, state_so_far.clone()),
            Operation::Fsync => (EnteredKind::Fsync, state_so_far.clone()),
            Operation::RollBackToVersion(number) => {
                (EnteredKind::RollBack, self.content_of_version(number)?)
            }
        };
        self.operations_since_the_last_recovery
            .push(EnteredOperation {
                kind,
                outcome: entered_outcome,
                state_after,
            });
        Ok(SubmittedOperation::Entered(OperationNumber(
            self.operations_since_the_last_recovery.len(),
        )))
    }

    /// 崩溃落在第 hi 个操作里时的允许集合。只读，不动模型。
    ///
    /// # Errors
    /// hi 越界、或写了之后报错的操作后面到崩溃为止又起了操作。
    pub fn allowed_set_for_a_crash_in(
        &self,
        crash_falls_in: OperationNumber,
    ) -> Result<AllowedSet, CrashNotAccepted> {
        let operations = &self.operations_since_the_last_recovery;
        // hi 由调用方交入（r5 判决用户定案 G2 那一句与第四节第 2 条的输入白名单）；落在崩溃本身里 = 序列里的操作都返回过之后才崩。
        let crash_operation_number = self.crash_after_every_operation_returned();
        if crash_falls_in.0 == 0 || crash_falls_in > crash_operation_number {
            return Err(
                CrashNotAccepted::CrashOperationIsOutsideTheOperationsSinceTheLastRecovery {
                    crash_falls_in,
                    crash_operation_number,
                },
            );
        }
        let started_before_the_crash = &operations[..crash_falls_in.0.min(operations.len())];
        // 报错之后的操作接在下一次恢复读出的状态上判（r5 判决第二节 U1）；报错之后不许照旧复用同一份内存态：
        // .claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
        if let Some(failed_position) = started_before_the_crash
            .iter()
            .position(|operation| operation.outcome == EnteredOutcome::FailedAfterWriting)
        {
            if failed_position + 1 < started_before_the_crash.len() {
                return Err(CrashNotAccepted::OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion {
                    failed_after_writing: OperationNumber(failed_position + 1),
                    operation_after_it: OperationNumber(failed_position + 2),
                });
            }
        }
        Ok(allowed_set_for_a_crash_in(
            &self.state_at_the_last_recovery,
            operations,
            crash_falls_in,
        ))
    }

    /// 崩溃落在第 hi 个操作里，恢复读出 `recovered`：先判它在允许集合里，再以它为起点接着走。
    ///
    /// # Errors
    /// [`CrashNotAccepted`] 的三种，返回时模型不动。
    pub fn crash_and_recover(
        &mut self,
        crash_falls_in: OperationNumber,
        recovered: VisibleState,
    ) -> Result<AllowedSet, CrashNotAccepted> {
        let allowed = self.allowed_set_for_a_crash_in(crash_falls_in)?;
        if !allowed.contains(&recovered) {
            return Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet {
                allowed,
                recovered,
            });
        }
        self.restart_from_the_recovered_state(crash_falls_in, recovered);
        Ok(allowed)
    }

    /// 崩溃恢复抛弃最新根的那一次（恢复时有读故障）：不判读出的内容，以它重新起头（r6 判决第二节 U2 一行：
    /// 「崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步」）。
    ///
    /// # Errors
    /// hi 越界、或写了之后报错的操作后面到崩溃为止又起了操作（同 [`Self::allowed_set_for_a_crash_in`]），返回时模型不动。
    pub fn restart_after_a_recovery_that_abandoned_the_newest_root(
        &mut self,
        crash_falls_in: OperationNumber,
        recovered: VisibleState,
    ) -> Result<(), CrashNotAccepted> {
        // 只核 hi 判不判得了；读出的内容在不在允许集合里不判
        self.allowed_set_for_a_crash_in(crash_falls_in)?;
        self.restart_from_the_recovered_state(crash_falls_in, recovered);
        Ok(())
    }

    fn restart_from_the_recovered_state(
        &mut self,
        crash_falls_in: OperationNumber,
        recovered: VisibleState,
    ) {
        // 前 hi − 1 个返回过；第 hi 个没返回，它是发布也不算已确认；第 hi 个之后的没发生。
        let returned_before_the_crash =
            &self.operations_since_the_last_recovery[..crash_falls_in.0 - 1];
        let confirmed: Vec<VisibleState> = confirmed_publishes_among(returned_before_the_crash)
            .cloned()
            .collect();
        self.versions_up_to_the_last_recovery.extend(confirmed);
        // 恢复读出、以它重新起头的那一份也是一个版本：之后可写挂载写的根映到它（r6 用户定案 H1-收严）
        self.versions_up_to_the_last_recovery
            .push(recovered.clone());
        // 每次恢复之后以读出的内容为起点接着走（r5 判决第二节 U2 ②）
        self.state_at_the_last_recovery = recovered;
        self.operations_since_the_last_recovery.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file_with(text: &str) -> VisibleState {
        VisibleState::OneFile(FileContent(text.as_bytes().to_vec()))
    }

    fn content(text: &str) -> FileContent {
        FileContent(text.as_bytes().to_vec())
    }

    /// 交一个结局是 `outcome` 的操作，它必须进序列；返回它的号。
    fn enter(
        model: &mut ReferenceFileSystem,
        operation: Operation,
        outcome: OperationOutcome,
    ) -> OperationNumber {
        match model.submit(operation, outcome) {
            Ok(SubmittedOperation::Entered(number)) => number,
            Ok(SubmittedOperation::DidNotHappen) => {
                panic!("这一步的结局不是「写之前拒」，必须进序列")
            }
            Err(refusal) => panic!("这一步的输入合法，不该被拒：{refusal:?}"),
        }
    }

    fn succeed(model: &mut ReferenceFileSystem, operation: Operation) -> OperationNumber {
        enter(model, operation, OperationOutcome::Succeeded)
    }

    fn allowed_indices(allowed: &AllowedSet) -> Vec<usize> {
        allowed
            .allowed
            .iter()
            .map(|allowed_state| allowed_state.index.0)
            .collect()
    }

    #[test]
    fn an_overwrite_refused_before_any_write_did_not_happen_so_the_publish_after_it_does_not_allow_its_content(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::Fsync);
        let refused = model.submit(
            Operation::OverwriteTheFile(content("B")),
            OperationOutcome::RefusedBeforeAnyWrite,
        );
        assert_eq!(
            refused,
            Ok(SubmittedOperation::DidNotHappen),
            "写之前拒的不进序列"
        );
        let publish = succeed(&mut model, Operation::ExplicitPublish);
        assert_eq!(
            publish,
            OperationNumber(3),
            "写之前拒的不占号，发布调用是第 3 个"
        );
        let allowed = model
            .allowed_set_for_a_crash_in(OperationNumber(4))
            .expect("崩溃落在崩溃本身里，号 4 合法");
        assert!(
            allowed.contains(&file_with("A")),
            "发布调用之后的 A 必须允许：{allowed:?}"
        );
        assert!(
            !allowed.contains(&file_with("B")),
            "被拒的覆盖写没发生，B 不许出现：{allowed:?}"
        );
    }

    #[test]
    fn a_crash_inside_a_publish_that_carries_two_overwrites_does_not_allow_the_state_that_cuts_the_publish(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::Fsync);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        succeed(&mut model, Operation::OverwriteTheFile(content("C")));
        let publish = succeed(&mut model, Operation::ExplicitPublish);
        let allowed = model
            .allowed_set_for_a_crash_in(publish)
            .expect("崩溃落在第 5 个操作发布调用里");
        assert_eq!(
            allowed_indices(&allowed),
            vec![2, 5],
            "只取 lo 与发布末尾：{allowed:?}"
        );
        assert!(
            !allowed.contains(&file_with("B")),
            "B 切开了带走 B、C 的那次发布：{allowed:?}"
        );
        assert!(
            allowed.contains(&file_with("C")),
            "在途的发布可能已发布，末尾的 C 允许：{allowed:?}"
        );
    }

    #[test]
    fn after_a_successful_fsync_its_own_index_is_the_lower_bound_so_the_content_before_it_is_not_allowed(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::Fsync);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        succeed(&mut model, Operation::Fsync);
        let allowed = model
            .allowed_set_for_a_crash_in(OperationNumber(5))
            .expect("号 5 是崩溃本身");
        assert_eq!(
            allowed.lower_bound,
            StateIndex(4),
            "第 4 个操作是返回成功的 fsync，下界是它的末尾"
        );
        assert!(
            !allowed.contains(&file_with("A")),
            "第二次 fsync 返回之后 A 不许再出现：{allowed:?}"
        );
        assert!(allowed.contains(&file_with("B")), "B 必须允许：{allowed:?}");
    }

    #[test]
    fn a_failed_fsync_keeps_the_lower_bound_at_the_previous_successful_fsync() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::Fsync);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        enter(
            &mut model,
            Operation::Fsync,
            OperationOutcome::FailedAfterWriting,
        );
        let allowed = model
            .allowed_set_for_a_crash_in(OperationNumber(5))
            .expect("报错的 fsync 是崩溃前最后一个操作，判得了");
        assert_eq!(allowed.lower_bound, StateIndex(2), "下界停在第一次 fsync");
        assert!(
            allowed.contains(&file_with("A")),
            "报错那次发布两可，A 允许：{allowed:?}"
        );
        assert!(
            allowed.contains(&file_with("B")),
            "报错那次发布两可，B 允许：{allowed:?}"
        );
    }

    #[test]
    fn a_publish_that_failed_after_writing_may_be_seen_after_the_crash() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::Fsync);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        enter(
            &mut model,
            Operation::ExplicitPublish,
            OperationOutcome::FailedAfterWriting,
        );
        let allowed = model
            .allowed_set_for_a_crash_in(OperationNumber(5))
            .expect("报错的发布是崩溃前最后一个操作，判得了");
        assert_eq!(
            allowed_indices(&allowed),
            vec![2, 4],
            "报错的发布末尾也在集合里：{allowed:?}"
        );
        assert!(
            allowed.contains(&file_with("B")),
            "报错那次发布可在，B 允许：{allowed:?}"
        );
    }

    #[test]
    fn a_crash_inside_an_overwrite_does_not_allow_the_publish_that_comes_after_it() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::Fsync);
        let overwrite = succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        succeed(&mut model, Operation::ExplicitPublish);
        let allowed = model
            .allowed_set_for_a_crash_in(overwrite)
            .expect("崩溃落在第 3 个操作里");
        assert_eq!(allowed.upper_bound, StateIndex(3), "hi 是覆盖写那一步");
        assert_eq!(
            allowed_indices(&allowed),
            vec![2],
            "崩溃之后的发布没发生：{allowed:?}"
        );
        assert!(
            !allowed.contains(&file_with("B")),
            "崩在覆盖写里，它后面那次发布带不出 B：{allowed:?}"
        );
    }

    #[test]
    fn a_crash_inside_an_fsync_does_not_raise_the_lower_bound() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        let fsync = succeed(&mut model, Operation::Fsync);
        let allowed = model
            .allowed_set_for_a_crash_in(fsync)
            .expect("崩溃落在第 4 个操作 fsync 里");
        assert_eq!(
            allowed.lower_bound,
            StateIndex(2),
            "下界在返回过的发布调用上；fsync 没返回，不抬"
        );
        assert!(
            allowed.contains(&file_with("A")),
            "发布调用之后的 A 允许：{allowed:?}"
        );
        assert!(
            allowed.contains(&file_with("B")),
            "在途的 fsync 可能已发布，B 允许：{allowed:?}"
        );
    }

    #[test]
    fn after_a_recovery_the_next_crash_is_judged_from_the_recovered_content_so_the_dropped_overwrite_does_not_come_back(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::Fsync);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        model
            .crash_and_recover(OperationNumber(4), file_with("A"))
            .expect("没发布的 B 丢掉，读出 A 合法");
        let fsync = succeed(&mut model, Operation::Fsync);
        assert_eq!(fsync, OperationNumber(1), "恢复之后从 1 重新编号");
        let crash = OperationNumber(fsync.0 + 1);
        let refused = model.crash_and_recover(crash, file_with("B"));
        assert!(
            matches!(
                refused,
                Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { .. })
            ),
            "丢掉的 B 不许在下一次恢复里复活：{refused:?}"
        );
        model
            .crash_and_recover(crash, file_with("A"))
            .expect("从读出的 A 起头，fsync 之后读出 A 合法");
    }

    #[test]
    fn a_recovered_state_outside_the_allowed_set_is_refused_and_the_model_does_not_move() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::Fsync);
        let before = model.clone();
        let refused = model.crash_and_recover(OperationNumber(3), VisibleState::NoFile);
        assert!(
            matches!(
                &refused,
                Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet {
                    recovered: VisibleState::NoFile,
                    ..
                })
            ),
            "fsync 之后读回空池必须判红：{refused:?}"
        );
        assert_eq!(model, before, "判红时模型不动");
    }

    #[test]
    fn rolling_back_to_the_first_confirmed_publish_gives_the_content_after_that_publish() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::OverwriteTheFile(content("C")));
        let rollback = succeed(&mut model, Operation::RollBackToVersion(VersionNumber(2)));
        let allowed = model
            .allowed_set_for_a_crash_in(OperationNumber(rollback.0 + 1))
            .expect("号 7 是崩溃本身");
        let after_the_rollback = allowed
            .allowed
            .iter()
            .find(|allowed_state| allowed_state.index == rollback.state_after())
            .expect("回退是一次发布，它的下标在集合里");
        assert_eq!(
            after_the_rollback.state,
            file_with("A"),
            "第 1 版是起点，第 2 版是第一次已确认发布之后的 A"
        );
    }

    #[test]
    fn a_crash_inside_a_rollback_allows_the_rolled_back_content_even_below_the_fsynced_state() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        succeed(&mut model, Operation::Fsync);
        let rollback = succeed(&mut model, Operation::RollBackToVersion(VersionNumber(2)));
        let allowed = model
            .allowed_set_for_a_crash_in(rollback)
            .expect("崩溃落在第 5 个操作回退里");
        assert_eq!(allowed.lower_bound, StateIndex(4), "fsync 把下界抬到 B");
        assert!(
            allowed.contains(&file_with("A")),
            "在途的回退可能已发布，它的末尾 A 允许：{allowed:?}"
        );
        assert!(
            allowed.contains(&file_with("B")),
            "回退没返回，fsync 之后的 B 允许：{allowed:?}"
        );
    }

    #[test]
    fn a_publish_that_failed_after_writing_is_not_counted_as_a_confirmed_publish_for_rollback() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        enter(
            &mut model,
            Operation::ExplicitPublish,
            OperationOutcome::FailedAfterWriting,
        );
        let refused = model.submit(
            Operation::RollBackToVersion(VersionNumber(3)),
            OperationOutcome::Succeeded,
        );
        assert_eq!(
            refused,
            Err(OperationNotAccepted::RollBackTargetIsNotAVersion {
                requested: VersionNumber(3),
                versions_so_far: 2,
            }),
            "报错的发布不算已确认：只有起点与 A 两版"
        );
    }

    #[test]
    fn a_publish_the_crash_fell_inside_is_not_counted_as_a_confirmed_publish_for_rollback() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        let publish = succeed(&mut model, Operation::ExplicitPublish);
        model
            .crash_and_recover(publish, file_with("A"))
            .expect("崩在发布调用里，读出 A 合法");
        let refused = model.submit(
            Operation::RollBackToVersion(VersionNumber(4)),
            OperationOutcome::Succeeded,
        );
        assert_eq!(
            refused,
            Err(OperationNotAccepted::RollBackTargetIsNotAVersion {
                requested: VersionNumber(4),
                versions_so_far: 3,
            }),
            "崩溃落在里面的发布没返回，不算已确认：起点、A、恢复读出的 A 三版"
        );
    }

    #[test]
    fn an_overwrite_after_a_failed_fsync_before_the_next_recovery_is_refused_as_not_in_the_first_version_and_the_model_does_not_move(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        enter(
            &mut model,
            Operation::Fsync,
            OperationOutcome::FailedAfterWriting,
        );
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        let before = model.clone();
        let refused = model.crash_and_recover(OperationNumber(4), file_with("A"));
        assert_eq!(
            refused,
            Err(CrashNotAccepted::OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion {
                failed_after_writing: OperationNumber(2),
                operation_after_it: OperationNumber(3),
            }),
            "报错之后、恢复之前又起的操作条款没定"
        );
        assert_eq!(model, before, "拒时模型不动");
    }

    #[test]
    fn a_crash_number_beyond_the_crash_itself_or_zero_is_refused_and_the_model_does_not_move() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        let before = model.clone();
        for crash_falls_in in [OperationNumber(0), OperationNumber(3)] {
            let refused = model.crash_and_recover(crash_falls_in, file_with("A"));
            assert_eq!(
                refused,
                Err(
                    CrashNotAccepted::CrashOperationIsOutsideTheOperationsSinceTheLastRecovery {
                        crash_falls_in,
                        crash_operation_number: OperationNumber(2),
                    }
                ),
                "序列只有 1 个操作，崩溃本身是第 2 个"
            );
        }
        assert_eq!(model, before, "拒时模型不动");
    }

    #[test]
    fn creating_the_first_file_twice_is_refused_and_the_model_does_not_move() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        let before = model.clone();
        let refused = model.submit(
            Operation::CreateTheFirstFile(content("B")),
            OperationOutcome::Succeeded,
        );
        assert_eq!(
            refused,
            Err(OperationNotAccepted::CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion),
            "已经有文件"
        );
        assert_eq!(model, before, "拒时模型不动");
    }

    #[test]
    fn overwriting_before_any_file_exists_is_refused_and_the_model_does_not_move() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        let before = model.clone();
        let refused = model.submit(
            Operation::OverwriteTheFile(content("B")),
            OperationOutcome::Succeeded,
        );
        assert_eq!(
            refused,
            Err(OperationNotAccepted::OverwritingWithoutAFileIsNotInTheFirstVersion),
            "还没有文件"
        );
        assert_eq!(model, before, "拒时模型不动");
    }

    #[test]
    fn rolling_back_to_version_zero_is_refused_and_the_model_does_not_move() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        let before = model.clone();
        let refused = model.submit(
            Operation::RollBackToVersion(VersionNumber(0)),
            OperationOutcome::Succeeded,
        );
        assert_eq!(
            refused,
            Err(OperationNotAccepted::RollBackTargetIsNotAVersion {
                requested: VersionNumber(0),
                versions_so_far: 2,
            }),
            "第几版从 1 起数"
        );
        assert_eq!(model, before, "拒时模型不动");
    }

    // ─── r6 用户定案 H1-收严 ───

    #[test]
    fn after_a_publish_call_returns_the_content_before_it_is_no_longer_allowed() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        let publish = succeed(&mut model, Operation::ExplicitPublish);
        let allowed = model
            .allowed_set_for_a_crash_in(OperationNumber(publish.0 + 1))
            .expect("崩溃落在崩溃本身里");
        assert_eq!(
            allowed.lower_bound,
            publish.state_after(),
            "返回成功的发布调用抬 lo 到它的末尾"
        );
        assert!(
            !allowed.contains(&file_with("A")),
            "第二次发布调用返回之后 A 不许再出现：{allowed:?}"
        );
    }

    #[test]
    fn after_a_rollback_returns_reading_the_content_before_the_rollback_is_red() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        succeed(&mut model, Operation::ExplicitPublish);
        let rollback = succeed(&mut model, Operation::RollBackToVersion(VersionNumber(2)));
        let crash = model.crash_after_every_operation_returned();
        let refused = model.crash_and_recover(crash, file_with("B"));
        assert!(
            matches!(
                refused,
                Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { .. })
            ),
            "回退返回之后读回回退之前的 B：回退在持久之前就返回的那一类坏实现：{refused:?}"
        );
        let allowed = model
            .crash_and_recover(crash, file_with("A"))
            .expect("回退之后读出回退目标 A 合法");
        assert_eq!(
            allowed.lower_bound,
            rollback.state_after(),
            "返回成功的回退抬 lo 到它的末尾"
        );
    }

    #[test]
    fn a_publish_resent_verbatim_after_failing_while_writing_raises_the_lower_bound_and_is_a_version(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        enter(
            &mut model,
            Operation::ExplicitPublish,
            OperationOutcome::SucceededOnAVerbatimResendAfterFailingWhileWriting,
        );
        assert_eq!(
            model.newest_version(),
            VersionNumber(3),
            "原样重发成功的发布是第 3 版"
        );
        let allowed = model
            .allowed_set_for_a_crash_in(model.crash_after_every_operation_returned())
            .expect("崩溃落在崩溃本身里");
        assert!(
            !allowed.contains(&file_with("A")),
            "原样重发成功＝做成，A 不许再出现：{allowed:?}"
        );
        assert!(allowed.contains(&file_with("B")), "B 必须允许：{allowed:?}");
    }

    #[test]
    fn a_freshly_formatted_pool_is_version_one_and_the_first_confirmed_publish_is_version_two() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        assert_eq!(model.newest_version(), VersionNumber(1), "起点是第 1 版");
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        assert_eq!(
            model.newest_version(),
            VersionNumber(1),
            "还没发布的覆盖不是新版本"
        );
        succeed(&mut model, Operation::ExplicitPublish);
        assert_eq!(
            model.newest_version(),
            VersionNumber(2),
            "第一次已确认的发布是第 2 版"
        );
    }

    #[test]
    fn after_a_recovery_adopts_a_publish_that_never_returned_rolling_back_to_the_version_that_recovery_restarted_from_gives_the_recovered_content(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        let publish = succeed(&mut model, Operation::ExplicitPublish);
        model
            .crash_and_recover(publish, file_with("A"))
            .expect("崩在第一个文件那次发布里，恢复采纳在飞的 A 合法");
        assert_eq!(
            model.newest_version(),
            VersionNumber(2),
            "在飞的发布不算已确认；恢复读出、重新起头的 A 是第 2 版"
        );
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        succeed(&mut model, Operation::ExplicitPublish);
        let rollback = succeed(&mut model, Operation::RollBackToVersion(VersionNumber(2)));
        let allowed = model
            .allowed_set_for_a_crash_in(model.crash_after_every_operation_returned())
            .expect("崩溃落在崩溃本身里");
        assert_eq!(
            allowed.allowed,
            vec![AllowedState {
                index: rollback.state_after(),
                state: file_with("A"),
            }],
            "回退到恢复重新起头的那一版，之后只允许它的内容"
        );
    }

    #[test]
    fn a_recovery_that_abandoned_the_newest_root_is_not_judged_and_the_next_crash_is_judged_from_what_it_read(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        succeed(&mut model, Operation::Fsync);
        let crash = model.crash_after_every_operation_returned();
        model
            .restart_after_a_recovery_that_abandoned_the_newest_root(crash, file_with("A"))
            .expect("抛弃最新根的那一次恢复不判：fsync 之后读出 A 也照收");
        let next_crash = model.crash_after_every_operation_returned();
        let refused = model.crash_and_recover(next_crash, file_with("B"));
        assert!(
            matches!(
                refused,
                Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { .. })
            ),
            "以读出的 A 重新起头，之后读回 B 判红：{refused:?}"
        );
    }
}
