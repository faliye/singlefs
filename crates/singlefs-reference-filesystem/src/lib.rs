//! RefFS：语义级的参照文件系统。只在内存里按 POSIX 操作走，判一次崩溃之后恢复读出的内容落不落在允许集合里。
//!
//! 依据：`D13（验证路线） .claude/kb/decisions/13-验证路线.md@已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义`（不与实现共用崩溃点枚举的那段代码）；
//! `D17（实现分层与第三方管道） .claude/kb/decisions/17-实现分层与第三方管道.md@已定项 4：入口 = RefFS 的操作面`（fsync 的持久性承诺由 RefFS 的崩溃语义定义）。
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

use std::collections::BTreeMap;

/// 一个文件的全部字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileContent(pub Vec<u8>);

/// POSIX 面上看得见的状态。第一版的操作只碰一个文件：还没有文件，或者有这一个文件与它的全部字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VisibleState {
    NoFile,
    OneFile(FileContent),
}

/// 一个只读快照的名字：调用方按实现点名它的那个号交入（实现按码 14 条目的树号点名，判决
/// `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 5）。这个包不解释它，只拿它当键。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnapshotName(pub u64);

/// 活着的只读快照：每个快照名对应建它那一刻 POSIX 面上看得见的状态（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
/// 第三节 S7「RefFS 加 CreateSnapshot、DestroySnapshot、读快照，状态加快照集合」）。
pub type Snapshots = BTreeMap<SnapshotName, VisibleState>;

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
    /// 建一个只读快照，名字是 `SnapshotName`：捕获这一刻的状态。它是一次独立发布（判决 S2），返回成功时抬起下界 lo。
    CreateSnapshot(SnapshotName),
    /// 删一个只读快照（一次调用 = 意图 + 全部批，判决第七节岔路 2、4）：返回成功时它不在了，抬起下界 lo。POSIX 面上的文件不变。
    DestroySnapshot(SnapshotName),
    /// 把文件截到这么多字节（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8）：比现在短就截掉尾部，比现在长就补 0。
    /// 内容改动型（同 [`Operation::OverwriteTheFile`]），不是发布点：实现的意图与各批落在一次调用里，崩在里面读出的只有截前、截后两样（K2「先落目标」），
    /// 就是这个操作施加之前与之后的状态，不用另起一类「一个操作的中间内容」。
    Truncate(u64),
    /// 把 `bytes` 写到文件偏移 `offset_in_bytes` 起（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节）：盖住那一段，写的终点越过
    /// 现在的长度时文件长到终点，现在的末尾与偏移之间补 0（POSIX 面上洞、空隙与写过的 0 分不出来，内容就是整份字节）。内容改动型（同
    /// [`Operation::OverwriteTheFile`]、[`Operation::Truncate`]），不是发布点：实现一次发布做完，崩在里面读出的只有写前、写后两样。
    WriteAtOffset {
        offset_in_bytes: u64,
        bytes: FileContent,
    },
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
    /// 还没有文件时截断：实现在任何写之前拒（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1「对 nlink = 0 或不在树里的 inode
    /// 截断，在任何写之前拒」），调用方交的是「写之前拒」、走不到这里；走到了就是输入不合法。
    TruncatingWithoutAFileIsNotInTheFirstVersion,
    /// 还没有文件时按偏移写：同截断，实现对不在树里的 inode 在任何写之前拒，调用方交的是「写之前拒」、走不到这里；走到了就是输入不合法。
    WritingAtAnOffsetWithoutAFileIsNotInTheFirstVersion,
    /// 回退目标 m 不是 1 到「到这一步为止的版本数」之间的数。
    RollBackTargetIsNotAVersion {
        requested: VersionNumber,
        versions_so_far: usize,
    },
    /// 建快照用了一个已经活着的快照名（允许的每一个快照集合里它都活着）。
    SnapshotNameAlreadyTaken { name: SnapshotName },
    /// 删一个不活着的快照（允许的每一个快照集合里它都不在）。
    NoSnapshotWithThisName { name: SnapshotName },
    /// 有活快照时回退：第一版有快照就拒回退（用户 2026-10-13 定，判决第三节 S8 与第七节岔路 3）；允许的每一个快照集合都不空。
    RollBackWithSnapshotsIsNotInTheFirstVersion,
}

/// 读快照读出的内容不是允许的任何一个快照集合里那个快照的内容：判红；返回它时模型一个字段都没动。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotReadOutsideTheAllowedReads {
    pub name: SnapshotName,
    pub read_out: VisibleState,
    /// 允许的每一个快照集合里这个快照读出什么（`None` = 那个集合里没有它），去重。
    pub allowed_reads: Vec<Option<VisibleState>>,
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

/// 抛弃最新根的那次恢复，被抛弃的那条根与 RefFS 最新那一版（[`ReferenceFileSystem::newest_version`]）是什么关系：调用方按它攒的
/// 「根 → 版本」表交入（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1，用户 2026-10-16 定「乙：翻译交进来」）。
/// RefFS 只收操作序列与读出的内容，自己看不出被抛弃的是哪一条根：实现的一版常有几条根（可写挂载的写行与暖机、整理的各批、
/// 为空间推的抬 F、卸载都映到写它时的现行版本；删快照一次调用写意图与全部批）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbandonedRoot {
    /// 被抛弃的是写出最新那一版的那一条根（发布调用、fsync、回退、建快照的根，删快照的最后一批）：恢复落到它前一条根，
    /// 在之前那一版上。
    WroteTheNewestVersion,
    /// 被抛弃的是映到最新那一版、但不是写出它的那一条根（这一版写出之后的写行与暖机、整理的批、抬 F、卸载，恢复重新起头之后写的根）：
    /// 恢复落到的前一条根仍在这一版上。
    WrittenAfterTheNewestVersion,
}

/// 抛弃最新根的那次恢复判不了；返回它时模型一个字段都没动。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbandonedRootRestartNotAccepted {
    /// 允许集合判不了：只会是 hi 越界、或写了之后报错的操作后面到崩溃为止又起了操作两种（同
    /// [`ReferenceFileSystem::allowed_set_for_a_crash_in`]）；读出的内容这一步不判，不会是判红那一种。
    TheCrashIsNotAccepted(CrashNotAccepted),
    /// 调用方说被抛弃的根写出了最新那一版，而允许集合里有 S_0（上一次恢复之后没有已确认的发布）：S_0 是上一次恢复读出、
    /// 重新起头的那一版（还没崩溃过时是起点），它之前那一版在上一次恢复之前（还没崩溃过时没有），RefFS 不记那时的快照集合。
    /// 判决第三节 J1 没写这一支取什么，第一版不支持。
    TheRootThatWroteTheStateAtTheLastRecoveryIsNotInTheFirstVersion,
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
    CreateSnapshot,
    DestroySnapshot,
}

impl EnteredKind {
    /// 这个操作施加之后的下标是不是一次显式发布的末尾：发布带走它之前取号成功的全部操作，末尾就是它自己的下标。
    fn ends_a_publish(self) -> bool {
        match self {
            // 发布边界只取历史里的显式发布点：发布调用与 fsync（r5 判决第二节 U2 ①）；fsync 触发的发布也是发布：
            // D16（发布语义） .claude/kb/decisions/16-发布语义.md@已定项 6：发布计数器，四个等号写成一条
            EnteredKind::ExplicitPublish | EnteredKind::Fsync => true,
            // 回退是挂着时的一次向前发布：D23（journal 的角色与格式） .claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
            EnteredKind::RollBack => true,
            // 建快照是一次独立发布（判决 S2）；删快照一次调用发意图与全部批，返回成功时最后一批已落盘（判决第七节岔路 2、4）。
            EnteredKind::CreateSnapshot | EnteredKind::DestroySnapshot => true,
            EnteredKind::ContentChange => false,
        }
    }

    /// 这个操作写出一版时，最后一条根之前写的那几条根上它的快照改动是不是已经起了头：删快照一次调用写意图与全部批，
    /// 抛弃最后一批的那次恢复落到意图或不是最后一批的批上，快照带销毁中记号、读照读，下一次用户改动先走空把它删完，
    /// 前后两份快照集合都可能（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1「删快照这种一个操作写多条根的取前后两份」）。
    /// 别的发布最后一条之前的根是会话推的抬 F，快照集合还是之前那一版的。
    fn its_earlier_roots_may_carry_its_snapshot_change(self) -> bool {
        match self {
            EnteredKind::DestroySnapshot => true,
            EnteredKind::ContentChange
            | EnteredKind::ExplicitPublish
            | EnteredKind::Fsync
            | EnteredKind::RollBack
            | EnteredKind::CreateSnapshot => false,
        }
    }
}

/// 一个操作怎么改快照集合。不按「之后的快照集合」记：恢复之后允许的快照集合可以不止一个（判决
/// `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6），同一个操作在每一个上各施加一次。
#[derive(Clone, Debug, PartialEq, Eq)]
enum SnapshotChangeOfTheOperation {
    Unchanged,
    Created {
        name: SnapshotName,
        captured: VisibleState,
    },
    Destroyed {
        name: SnapshotName,
    },
}

impl SnapshotChangeOfTheOperation {
    fn apply_to(&self, snapshots: &mut Snapshots) {
        match self {
            SnapshotChangeOfTheOperation::Unchanged => {}
            SnapshotChangeOfTheOperation::Created { name, captured } => {
                snapshots.insert(*name, captured.clone());
            }
            SnapshotChangeOfTheOperation::Destroyed { name } => {
                snapshots.remove(name);
            }
        }
    }
}

/// 实现做了这个操作（进了序列）说明它那一刻的快照集合满足什么：实现在任何写之前拒掉不满足的，拒掉的不进序列。
/// 允许的快照集合里不满足的那几个由此排除（同一段历史里实现只有一个快照集合）。不看快照集合的操作没有这一样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SnapshotPreconditionOfTheOperation {
    NameFree(SnapshotName),
    SnapshotPresent(SnapshotName),
    NoSnapshotAtAll,
}

impl SnapshotPreconditionOfTheOperation {
    fn holds_in(self, snapshots: &Snapshots) -> bool {
        match self {
            SnapshotPreconditionOfTheOperation::NameFree(name) => !snapshots.contains_key(&name),
            SnapshotPreconditionOfTheOperation::SnapshotPresent(name) => {
                snapshots.contains_key(&name)
            }
            SnapshotPreconditionOfTheOperation::NoSnapshotAtAll => snapshots.is_empty(),
        }
    }

    /// 允许的快照集合一个都不满足时不收这个操作的那一条。
    fn refusal_when_no_allowed_snapshot_set_meets_it(self) -> OperationNotAccepted {
        match self {
            SnapshotPreconditionOfTheOperation::NameFree(name) => {
                OperationNotAccepted::SnapshotNameAlreadyTaken { name }
            }
            SnapshotPreconditionOfTheOperation::SnapshotPresent(name) => {
                OperationNotAccepted::NoSnapshotWithThisName { name }
            }
            SnapshotPreconditionOfTheOperation::NoSnapshotAtAll => {
                OperationNotAccepted::RollBackWithSnapshotsIsNotInTheFirstVersion
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EnteredOperation {
    kind: EnteredKind,
    outcome: EnteredOutcome,
    state_after: VisibleState,
    snapshot_change: SnapshotChangeOfTheOperation,
}

/// 往允许的快照集合里加一个，已经有相同的就不加（集合按内容去重）。
fn push_distinct_snapshot_set(snapshot_sets: &mut Vec<Snapshots>, snapshots: Snapshots) {
    if !snapshot_sets.contains(&snapshots) {
        snapshot_sets.push(snapshots);
    }
}

/// 序列里已确认的发布（返回成功的发布调用、fsync 与回退，原样重发成功的也算）各自发布之后的内容，按历史次序。
fn confirmed_publishes_among(
    operations: &[EnteredOperation],
) -> impl Iterator<Item = &VisibleState> {
    operations
        .iter()
        // 写了之后报错的发布两可，不算已确认：D16（发布语义） .claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
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
    // D16（发布语义） .claude/kb/decisions/16-发布语义.md@已定项 7：发布的持久顺序 里「fsync 等系统配置轮换持久之后才返回」与回退
    // 「它的根持久之后才向管理员确认，与 fsync 同一个返回条件」）。位置 p（从 0 起）上的是第 p + 1 个操作，施加之后是 S_{p+1}；
    // 发布调用与 fsync 不改内容，S_{p+1} 与调用之前那一刻相同。没有时 lo = 0，起点是上一次恢复读出的内容。
    // 报错时下界停在上一次返回成功的那一个：D16（发布语义） .claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
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
    // 切开一次发布的 S_j 不在集合里：D16（发布语义） .claude/kb/decisions/16-发布语义.md@已定项 4：一次发布整体施加或整体不施加，第二份 replay 停在句法层；
    // D16（发布语义） .claude/kb/decisions/16-发布语义.md@已定项 12：中间状态的合法性语义改写成「根 + journal 任意前缀」；
    // 单元级的切分（D23（journal 的角色与格式） .claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
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
        // 写了之后报错的发布也算末尾：那次发布可在可不在（D16（发布语义） .claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么）
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
    /// S_0 那一刻允许活着的快照集合，去重、至少一个：还没崩溃过时只有「没有快照」；恢复之后是允许集合里与读出的内容相同的每一个状态那一刻的
    /// 快照集合（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6），抛弃最新根的那次恢复之后按调用方交的
    /// [`AbandonedRoot`] 取（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1），再排除之后实现做了的操作说不通的那几个。
    /// 序列里每个操作在每一个上各施加一次（[`SnapshotChangeOfTheOperation`]），第 k 个下标与 S_0 那一刻的第 k 个一一对应。
    allowed_snapshot_sets_at_the_last_recovery: Vec<Snapshots>,
}

impl ReferenceFileSystem {
    pub fn on_a_freshly_formatted_pool() -> Self {
        Self {
            state_at_the_last_recovery: VisibleState::NoFile,
            operations_since_the_last_recovery: Vec::new(),
            // 起点是第 1 版：mkfs 写的根与 mkfs 之后可写挂载写的根映到它（r6 用户定案 H1-收严「m 的编号覆盖每一个写根的入口」）
            versions_up_to_the_last_recovery: vec![VisibleState::NoFile],
            allowed_snapshot_sets_at_the_last_recovery: vec![Snapshots::new()],
        }
    }

    /// S_j 那一刻的快照集合，与 S_0 那一刻允许的快照集合逐个对应（不去重）：每一个施加序列里前 j 个操作。
    fn snapshot_set_of_each_allowed_start_at(&self, index: StateIndex) -> Vec<Snapshots> {
        let operations = &self.operations_since_the_last_recovery[..index.0];
        self.allowed_snapshot_sets_at_the_last_recovery
            .iter()
            .map(|snapshots_at_the_last_recovery| {
                let mut snapshots = snapshots_at_the_last_recovery.clone();
                for operation in operations {
                    operation.snapshot_change.apply_to(&mut snapshots);
                }
                snapshots
            })
            .collect()
    }

    /// 序列里的操作全都做成时允许活着的快照集合，去重：恢复之后读出的文件定不了落在哪一个发布末尾时不止一个（判决第三节 F6）。
    pub fn allowed_snapshot_sets(&self) -> Vec<Snapshots> {
        let mut allowed = Vec::new();
        for snapshots in self.snapshot_set_of_each_allowed_start_at(StateIndex(
            self.operations_since_the_last_recovery.len(),
        )) {
            push_distinct_snapshot_set(&mut allowed, snapshots);
        }
        allowed
    }

    /// 读快照 `name` 允许读出什么（序列里的操作全都做成时）：允许的每一个快照集合里它活着时是建它那一刻的状态，不活着是 `None`；去重。
    pub fn allowed_reads_of_the_snapshot(&self, name: SnapshotName) -> Vec<Option<VisibleState>> {
        let mut allowed_reads = Vec::new();
        for snapshots in self.allowed_snapshot_sets() {
            let read = snapshots.get(&name).cloned();
            if !allowed_reads.contains(&read) {
                allowed_reads.push(read);
            }
        }
        allowed_reads
    }

    /// 实现读快照 `name` 读出 `read_out`（序列里的操作全都做成时）：它要是允许的某一个快照集合里那个快照的内容；
    /// 是的话只留下读得出它的那几个快照集合（实现只有一个快照集合，读出的内容说明是哪几个）。
    ///
    /// # Errors
    /// 允许的快照集合里没有一个读得出它：判红，模型不动。
    pub fn judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
        &mut self,
        name: SnapshotName,
        read_out: &VisibleState,
    ) -> Result<(), SnapshotReadOutsideTheAllowedReads> {
        let current = self.snapshot_set_of_each_allowed_start_at(StateIndex(
            self.operations_since_the_last_recovery.len(),
        ));
        let kept: Vec<Snapshots> = self
            .allowed_snapshot_sets_at_the_last_recovery
            .iter()
            .zip(&current)
            .filter(|(_, snapshots)| snapshots.get(&name) == Some(read_out))
            .map(|(snapshots_at_the_last_recovery, _)| snapshots_at_the_last_recovery.clone())
            .collect();
        if kept.is_empty() {
            return Err(SnapshotReadOutsideTheAllowedReads {
                name,
                read_out: read_out.clone(),
                allowed_reads: self.allowed_reads_of_the_snapshot(name),
            });
        }
        self.allowed_snapshot_sets_at_the_last_recovery = kept;
        Ok(())
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
        // 第 m 版从 1 起数，在第 m − 1 位：D23（journal 的角色与格式） .claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
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
            // D23（journal 的角色与格式） .claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换（「在任何写之前拒」）
            OperationOutcome::RefusedBeforeAnyWrite => return Ok(SubmittedOperation::DidNotHappen),
            OperationOutcome::Succeeded => EnteredOutcome::Succeeded,
            // 原样重发成功＝做成：D23（journal 的角色与格式） .claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
            // （「这一版的失败处置」：重发成功才建下一次发布）；r6 判决第二节「G2 回退被拒」一行补的结局
            OperationOutcome::SucceededOnAVerbatimResendAfterFailingWhileWriting => {
                EnteredOutcome::Succeeded
            }
            // 写了之后报错＝两可：D16（发布语义） .claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
            OperationOutcome::FailedAfterWriting => EnteredOutcome::FailedAfterWriting,
        };
        // 实现做了这个操作说明它那一刻的快照集合满足什么（[`SnapshotPreconditionOfTheOperation`]）：允许的快照集合里不满足的排除，
        // 一个都不满足就不收。回退先判快照、再判目标版本（两样都不收时报快照那一条）。
        let snapshot_precondition = match &operation {
            Operation::CreateTheFirstFile(_)
            | Operation::OverwriteTheFile(_)
            | Operation::ExplicitPublish
            | Operation::Fsync
            | Operation::WriteAtOffset { .. }
            | Operation::Truncate(_) => None,
            // 第一版有快照就拒回退（判决 S8）：实现在任何写之前拒，调用方交的是「写之前拒」，走不到这里；允许的快照集合都不空时走到了就是输入不合法。
            Operation::RollBackToVersion(_) => {
                Some(SnapshotPreconditionOfTheOperation::NoSnapshotAtAll)
            }
            Operation::CreateSnapshot(name) => {
                Some(SnapshotPreconditionOfTheOperation::NameFree(*name))
            }
            Operation::DestroySnapshot(name) => {
                Some(SnapshotPreconditionOfTheOperation::SnapshotPresent(*name))
            }
        };
        let allowed_snapshot_sets_at_the_last_recovery = match snapshot_precondition {
            None => self.allowed_snapshot_sets_at_the_last_recovery.clone(),
            Some(precondition) => {
                let current = self.snapshot_set_of_each_allowed_start_at(StateIndex(
                    self.operations_since_the_last_recovery.len(),
                ));
                let kept: Vec<Snapshots> = self
                    .allowed_snapshot_sets_at_the_last_recovery
                    .iter()
                    .zip(&current)
                    .filter(|(_, snapshots)| precondition.holds_in(snapshots))
                    .map(|(snapshots_at_the_last_recovery, _)| {
                        snapshots_at_the_last_recovery.clone()
                    })
                    .collect();
                if kept.is_empty() {
                    return Err(precondition.refusal_when_no_allowed_snapshot_set_meets_it());
                }
                kept
            }
        };
        let state_so_far = self.state_after_every_operation_so_far();
        let (kind, state_after, snapshot_change) = match operation {
            Operation::CreateTheFirstFile(content) => match state_so_far {
                VisibleState::NoFile => (
                    EnteredKind::ContentChange,
                    VisibleState::OneFile(content),
                    SnapshotChangeOfTheOperation::Unchanged,
                ),
                VisibleState::OneFile(_) => return Err(OperationNotAccepted::CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion),
            },
            Operation::OverwriteTheFile(content) => match state_so_far {
                VisibleState::NoFile => return Err(OperationNotAccepted::OverwritingWithoutAFileIsNotInTheFirstVersion),
                VisibleState::OneFile(_) => (
                    EnteredKind::ContentChange,
                    VisibleState::OneFile(content),
                    SnapshotChangeOfTheOperation::Unchanged,
                ),
            },
            // 截短或补 0（K8）：POSIX 面上洞与写过的 0 分不出来，内容就是整份字节。
            Operation::Truncate(new_size_in_bytes) => match state_so_far {
                VisibleState::NoFile => {
                    return Err(OperationNotAccepted::TruncatingWithoutAFileIsNotInTheFirstVersion)
                }
                VisibleState::OneFile(FileContent(bytes)) => {
                    let mut truncated = bytes.clone();
                    truncated.resize(
                        usize::try_from(new_size_in_bytes)
                            .expect("64 位上 usize 与 u64 同宽：截到的长度装得进"),
                        0,
                    );
                    (
                        EnteredKind::ContentChange,
                        VisibleState::OneFile(FileContent(truncated)),
                        SnapshotChangeOfTheOperation::Unchanged,
                    )
                }
            },
            // 按偏移盖上、越过末尾的补 0 到终点（判决 m4-partial-write-r1 第二节 R5）。
            Operation::WriteAtOffset {
                offset_in_bytes,
                bytes: FileContent(written),
            } => match state_so_far {
                VisibleState::NoFile => {
                    return Err(
                        OperationNotAccepted::WritingAtAnOffsetWithoutAFileIsNotInTheFirstVersion,
                    )
                }
                VisibleState::OneFile(FileContent(bytes)) => {
                    let offset = usize::try_from(offset_in_bytes)
                        .expect("64 位上 usize 与 u64 同宽：写的偏移装得进");
                    let end = offset
                        .checked_add(written.len())
                        .expect("调用方交的是实现做成的写：终点装得进 u64");
                    let mut written_over = bytes.clone();
                    if written_over.len() < end {
                        written_over.resize(end, 0);
                    }
                    written_over[offset..end].copy_from_slice(&written);
                    (
                        EnteredKind::ContentChange,
                        VisibleState::OneFile(FileContent(written_over)),
                        SnapshotChangeOfTheOperation::Unchanged,
                    )
                }
            },
            Operation::ExplicitPublish => (
                EnteredKind::ExplicitPublish,
                state_so_far.clone(),
                SnapshotChangeOfTheOperation::Unchanged,
            ),
            Operation::Fsync => (
                EnteredKind::Fsync,
                state_so_far.clone(),
                SnapshotChangeOfTheOperation::Unchanged,
            ),
            Operation::RollBackToVersion(number) => (
                EnteredKind::RollBack,
                self.content_of_version(number)?,
                SnapshotChangeOfTheOperation::Unchanged,
            ),
            // 文件集合为空时建快照照收，快照读出「没有文件」（用户 2026-10-16 定「放开」，判决
            // `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）。
            Operation::CreateSnapshot(name) => (
                EnteredKind::CreateSnapshot,
                state_so_far.clone(),
                SnapshotChangeOfTheOperation::Created {
                    name,
                    captured: state_so_far.clone(),
                },
            ),
            Operation::DestroySnapshot(name) => (
                EnteredKind::DestroySnapshot,
                state_so_far.clone(),
                SnapshotChangeOfTheOperation::Destroyed { name },
            ),
        };
        self.allowed_snapshot_sets_at_the_last_recovery =
            allowed_snapshot_sets_at_the_last_recovery;
        self.operations_since_the_last_recovery
            .push(EnteredOperation {
                kind,
                outcome: entered_outcome,
                state_after,
                snapshot_change,
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
        // D16（发布语义） .claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
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
        // 恢复之后允许活着的快照集合：允许集合里与读出的内容相同的每一个状态那一刻的快照集合（判决
        // `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6）。崩溃判定只比文件；建快照与删快照都是发布末尾，
        // 写了之后报错或崩在里面的那一次落没落盘读出的文件定不了，两种结局都收，不取下标最大的那一个。
        let indices_whose_state_is_the_recovered_one: Vec<StateIndex> = allowed
            .allowed
            .iter()
            .filter(|allowed_state| allowed_state.state == recovered)
            .map(|allowed_state| allowed_state.index)
            .collect();
        let allowed_snapshot_sets_after_the_recovery =
            self.distinct_snapshot_sets_at(&indices_whose_state_is_the_recovered_one);
        self.restart_from_the_recovered_state(
            crash_falls_in,
            recovered,
            allowed_snapshot_sets_after_the_recovery,
        );
        Ok(allowed)
    }

    /// 崩溃恢复抛弃最新根的那一次（恢复时有读故障）：不判读出的内容，以它重新起头（r6 判决第二节 U2 一行：
    /// 「崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步」）。重新起头时允许活着的快照集合按调用方交的
    /// [`AbandonedRoot`] 取（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1）：允许集合里每一个状态 S_j
    /// 都可能是最新那一版，各取一份、并起来——被抛弃的是写出这一版的那一条，取之前那一版的（写出它的是删快照这种一个操作写多条根的，
    /// 前后两份都取）；不是写出这一版的那一条，取 S_j 自己那一刻的。
    ///
    /// # Errors
    /// [`AbandonedRootRestartNotAccepted`] 的两种，返回时模型不动。
    pub fn restart_after_a_recovery_that_abandoned_the_newest_root(
        &mut self,
        crash_falls_in: OperationNumber,
        abandoned_root: AbandonedRoot,
        recovered: VisibleState,
    ) -> Result<(), AbandonedRootRestartNotAccepted> {
        // 只核 hi 判不判得了；读出的内容在不在允许集合里不判
        let allowed = self
            .allowed_set_for_a_crash_in(crash_falls_in)
            .map_err(AbandonedRootRestartNotAccepted::TheCrashIsNotAccepted)?;
        let mut indices_whose_snapshot_sets_the_restart_allows: Vec<StateIndex> = Vec::new();
        for allowed_state in &allowed.allowed {
            let indices_when_this_state_is_the_newest_version = self
                .indices_whose_snapshot_sets_a_restart_allows_when_the_newest_version_is(
                    allowed_state.index,
                    abandoned_root,
                )?;
            indices_whose_snapshot_sets_the_restart_allows
                .extend(indices_when_this_state_is_the_newest_version);
        }
        let allowed_snapshot_sets_after_the_recovery =
            self.distinct_snapshot_sets_at(&indices_whose_snapshot_sets_the_restart_allows);
        self.restart_from_the_recovered_state(
            crash_falls_in,
            recovered,
            allowed_snapshot_sets_after_the_recovery,
        );
        Ok(())
    }

    /// S_j（`newest_state`）是最新那一版、被抛弃的根与它的关系是 `abandoned_root` 时，抛弃最新根的那次恢复之后允许哪几个下标那一刻的快照集合
    /// （判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1）。
    ///
    /// # Errors
    /// 被抛弃的根写出了 S_0 那一版：[`AbandonedRootRestartNotAccepted::TheRootThatWroteTheStateAtTheLastRecoveryIsNotInTheFirstVersion`]。
    fn indices_whose_snapshot_sets_a_restart_allows_when_the_newest_version_is(
        &self,
        newest_state: StateIndex,
        abandoned_root: AbandonedRoot,
    ) -> Result<Vec<StateIndex>, AbandonedRootRestartNotAccepted> {
        match abandoned_root {
            // 恢复落到的前一条根仍在这一版上
            AbandonedRoot::WrittenAfterTheNewestVersion => Ok(vec![newest_state]),
            AbandonedRoot::WroteTheNewestVersion => {
                // S_j 由第 j 个操作写出（j 是 lo 或某次发布的末尾）；j = 0 时它是上一次恢复重新起头的那一版，之前那一版 RefFS 不记
                let Some(position_of_the_operation_that_wrote_it) = newest_state.0.checked_sub(1)
                else {
                    return Err(AbandonedRootRestartNotAccepted::TheRootThatWroteTheStateAtTheLastRecoveryIsNotInTheFirstVersion);
                };
                let index_of_the_version_before = self
                    .index_of_the_last_confirmed_publish_before_the_operation_at(
                        position_of_the_operation_that_wrote_it,
                    );
                let operation_that_wrote_it = self
                    .operations_since_the_last_recovery
                    .get(position_of_the_operation_that_wrote_it)
                    .expect("允许集合里 S_0 之后的下标都是某个操作的末尾，不超过序列长度");
                if operation_that_wrote_it
                    .kind
                    .its_earlier_roots_may_carry_its_snapshot_change()
                {
                    Ok(vec![index_of_the_version_before, newest_state])
                } else {
                    Ok(vec![index_of_the_version_before])
                }
            }
        }
    }

    /// 第 `position_of_the_operation` + 1 个操作之前那一版的下标：上一次恢复之后、它之前最后一次已确认发布（返回成功的发布调用、fsync、
    /// 回退、建快照、删快照）的末尾；没有时是 0（上一次恢复读出、重新起头的那一版，还没崩溃过时是起点）。它之前的操作都返回过，
    /// 写了之后报错的只能是 hi 之前的最后一个（[`Self::allowed_set_for_a_crash_in`] 拒掉别的），所以它之前的发布末尾都已确认。
    fn index_of_the_last_confirmed_publish_before_the_operation_at(
        &self,
        position_of_the_operation: usize,
    ) -> StateIndex {
        self.operations_since_the_last_recovery[..position_of_the_operation]
            .iter()
            .enumerate()
            .rev()
            .find(|(_, operation)| match operation.outcome {
                EnteredOutcome::Succeeded => operation.kind.ends_a_publish(),
                EnteredOutcome::FailedAfterWriting => false,
            })
            .map_or(StateIndex(0), |(earlier_position, _)| {
                StateIndex(earlier_position + 1)
            })
    }

    /// 这几个下标那一刻允许活着的快照集合，并起来、去重。
    fn distinct_snapshot_sets_at(&self, indices: &[StateIndex]) -> Vec<Snapshots> {
        let mut distinct_snapshot_sets = Vec::new();
        for index in indices {
            for snapshots in self.snapshot_set_of_each_allowed_start_at(*index) {
                push_distinct_snapshot_set(&mut distinct_snapshot_sets, snapshots);
            }
        }
        distinct_snapshot_sets
    }

    fn restart_from_the_recovered_state(
        &mut self,
        crash_falls_in: OperationNumber,
        recovered: VisibleState,
        allowed_snapshot_sets_after_the_recovery: Vec<Snapshots>,
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
        self.allowed_snapshot_sets_at_the_last_recovery = allowed_snapshot_sets_after_the_recovery;
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

    /// 建快照捕获那一刻的文件，之后覆盖写不动它；读快照读出建它那一刻的内容；删掉之后读不到，活头的文件不变
    /// （判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S7）。
    #[test]
    fn a_snapshot_reads_the_file_at_its_creation_until_it_is_destroyed() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(65_549)));
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        assert_eq!(
            model.allowed_reads_of_the_snapshot(SnapshotName(65_549)),
            vec![Some(file_with("A"))],
            "快照读出建它那一刻的 A"
        );
        succeed(&mut model, Operation::DestroySnapshot(SnapshotName(65_549)));
        assert_eq!(
            model.allowed_reads_of_the_snapshot(SnapshotName(65_549)),
            vec![None],
            "删掉之后读不到"
        );
        assert_eq!(
            model.allowed_snapshot_sets(),
            vec![Snapshots::new()],
            "快照集合空了"
        );
        assert_eq!(
            model.state_after_every_operation_so_far(),
            &file_with("B"),
            "删快照不动活头的文件"
        );
    }

    /// 建快照、删快照各是发布末尾（返回成功时抬起下界）：崩在建快照之后，允许集合的下界就是它；恢复读出那一刻的文件，快照接着活着。
    #[test]
    fn a_crash_after_a_snapshot_returned_keeps_the_snapshot_and_starts_from_it() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        let creation = succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        let crash = model.crash_after_every_operation_returned();
        let allowed = model
            .crash_and_recover(crash, file_with("A"))
            .expect("A 在允许集合里");
        assert_eq!(
            allowed.lower_bound,
            creation.state_after(),
            "建快照返回成功：下界是它的末尾"
        );
        assert_eq!(
            model.allowed_reads_of_the_snapshot(SnapshotName(7)),
            vec![Some(file_with("A"))],
            "恢复之后快照接着活着"
        );
    }

    /// 文件集合为空时建快照照收（用户 2026-10-16 定「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md`
    /// 第三节 F5）：空池上建快照，之后写第一个文件，读快照读出「没有文件」，活头读出写的那份。
    #[test]
    fn a_snapshot_created_while_there_is_no_file_reads_no_file_after_the_first_file_is_written() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(3)));
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        assert_eq!(
            model.allowed_reads_of_the_snapshot(SnapshotName(3)),
            vec![Some(VisibleState::NoFile)],
            "快照读出建它那一刻的「没有文件」"
        );
        assert_eq!(
            model.state_after_every_operation_so_far(),
            &file_with("A"),
            "活头读出写的 A"
        );
    }

    /// 建快照写了之后报错（做没做成两可），之后崩溃、恢复读出 A：建快照之前与之后两个发布末尾都是 A，读出的文件定不了那次发布落没落盘，
    /// 快照有与没有两种结局都收（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6）——之后删这个快照照收（落了盘），
    /// 回退也照收（没落盘：第一版有快照就拒回退，只有快照集合可以是空的才收）。
    #[test]
    fn a_crash_after_a_snapshot_creation_that_failed_after_writing_allows_the_snapshot_to_be_there_or_not(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::Fsync);
        enter(
            &mut model,
            Operation::CreateSnapshot(SnapshotName(7)),
            OperationOutcome::FailedAfterWriting,
        );
        let crash = model.crash_after_every_operation_returned();
        model
            .crash_and_recover(crash, file_with("A"))
            .expect("A 在允许集合里");
        assert_eq!(
            model.allowed_snapshot_sets(),
            vec![
                Snapshots::new(),
                Snapshots::from([(SnapshotName(7), file_with("A"))])
            ],
            "快照没有与有两种快照集合都允许"
        );
        let mut the_creation_landed = model.clone();
        succeed(
            &mut the_creation_landed,
            Operation::DestroySnapshot(SnapshotName(7)),
        );
        let mut the_creation_did_not_land = model.clone();
        let rollback_target = the_creation_did_not_land.newest_version();
        succeed(
            &mut the_creation_did_not_land,
            Operation::RollBackToVersion(rollback_target),
        );
        assert_eq!(
            the_creation_did_not_land.allowed_snapshot_sets(),
            vec![Snapshots::new()],
            "回退做成说明快照没落盘：只剩空的那个快照集合"
        );
    }

    /// 删快照写了之后报错（意图落了、批没落完，两可），之后崩溃、恢复读出 A：快照还在与已删两种结局都收（判决第三节 F6）——
    /// 读快照读出建它那一刻的 A 照收、读出别的内容判红且模型不动；读出 A 之后只剩快照还在的那个集合，回退照拒；
    /// 没读快照时回退照收（快照已删）。
    #[test]
    fn a_crash_after_a_snapshot_destruction_that_failed_after_writing_allows_the_snapshot_to_be_there_or_not(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
        enter(
            &mut model,
            Operation::DestroySnapshot(SnapshotName(7)),
            OperationOutcome::FailedAfterWriting,
        );
        let crash = model.crash_after_every_operation_returned();
        model
            .crash_and_recover(crash, file_with("A"))
            .expect("A 在允许集合里");
        assert_eq!(
            model.allowed_snapshot_sets(),
            vec![
                Snapshots::from([(SnapshotName(7), file_with("A"))]),
                Snapshots::new()
            ],
            "快照还在与已删两种快照集合都允许"
        );
        let mut the_destruction_did_not_land = model.clone();
        let before_the_wrong_read = the_destruction_did_not_land.clone();
        assert_eq!(
            the_destruction_did_not_land
                .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
                    SnapshotName(7),
                    &file_with("B")
                ),
            Err(SnapshotReadOutsideTheAllowedReads {
                name: SnapshotName(7),
                read_out: file_with("B"),
                allowed_reads: vec![Some(file_with("A")), None],
            }),
            "读出建它那一刻之外的内容判红"
        );
        assert_eq!(
            the_destruction_did_not_land, before_the_wrong_read,
            "判红的读不动模型"
        );
        the_destruction_did_not_land
            .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
                SnapshotName(7),
                &file_with("A"),
            )
            .expect("快照还在：读出建它那一刻的 A");
        assert_eq!(
            the_destruction_did_not_land.allowed_snapshot_sets(),
            vec![Snapshots::from([(SnapshotName(7), file_with("A"))])],
            "读出 A 之后只剩快照还在的那个集合"
        );
        let rollback_target_while_the_snapshot_is_kept =
            the_destruction_did_not_land.newest_version();
        assert_eq!(
            the_destruction_did_not_land.submit(
                Operation::RollBackToVersion(rollback_target_while_the_snapshot_is_kept),
                OperationOutcome::Succeeded
            ),
            Err(OperationNotAccepted::RollBackWithSnapshotsIsNotInTheFirstVersion),
            "快照还在：回退照拒"
        );
        let mut the_destruction_landed = model.clone();
        let rollback_target_after_the_destruction_landed = the_destruction_landed.newest_version();
        succeed(
            &mut the_destruction_landed,
            Operation::RollBackToVersion(rollback_target_after_the_destruction_landed),
        );
    }

    /// 不收的三种：快照名已经活着、删一个不活着的快照、有活快照时回退；返回时模型不动。
    #[test]
    fn snapshot_operations_the_first_version_does_not_take_are_not_accepted_and_leave_the_model_unchanged(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(1)));
        let before = model.clone();
        assert_eq!(
            model.submit(
                Operation::CreateSnapshot(SnapshotName(1)),
                OperationOutcome::Succeeded
            ),
            Err(OperationNotAccepted::SnapshotNameAlreadyTaken {
                name: SnapshotName(1)
            }),
            "快照名已经活着"
        );
        assert_eq!(
            model.submit(
                Operation::DestroySnapshot(SnapshotName(2)),
                OperationOutcome::Succeeded
            ),
            Err(OperationNotAccepted::NoSnapshotWithThisName {
                name: SnapshotName(2)
            }),
            "删一个不活着的快照"
        );
        assert_eq!(
            model.submit(
                Operation::RollBackToVersion(VersionNumber(1)),
                OperationOutcome::Succeeded
            ),
            Err(OperationNotAccepted::RollBackWithSnapshotsIsNotInTheFirstVersion),
            "有活快照时回退"
        );
        assert_eq!(model, before, "不收的操作不动模型");
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
        // 被抛弃的是 fsync 写出最新那一版的那一条根
        model
            .restart_after_a_recovery_that_abandoned_the_newest_root(
                crash,
                AbandonedRoot::WroteTheNewestVersion,
                file_with("A"),
            )
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

    /// 攻方 R6 那一形（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H5，读法照 U2）：写 A、发布、建快照 7、
    /// 覆盖写 B、发布，抛弃最新根（B 那次发布）的那次恢复读出 A。读出的 A 不在允许集合里也不判、照收；重新起头时的快照集合取被抛弃那一版
    /// 之前那一版（建快照那一版）的，快照 7 照在，读它读出 A 不判红。
    #[test]
    fn a_recovery_that_abandoned_the_newest_root_keeps_a_snapshot_created_in_the_published_state_before_it(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
        succeed(&mut model, Operation::ExplicitPublish);
        let crash = model.crash_after_every_operation_returned();
        let allowed = model
            .allowed_set_for_a_crash_in(crash)
            .expect("崩溃落在崩溃本身里");
        assert!(
            !allowed.contains(&file_with("A")),
            "读出的 A 不在允许集合里（只有 B 那次发布）：{allowed:?}"
        );
        // 被抛弃的是 B 那次发布写出最新那一版的那一条根
        model
            .restart_after_a_recovery_that_abandoned_the_newest_root(
                crash,
                AbandonedRoot::WroteTheNewestVersion,
                file_with("A"),
            )
            .expect("抛弃最新根的那一次恢复不判读出的内容");
        assert_eq!(
            model.allowed_snapshot_sets(),
            vec![Snapshots::from([(SnapshotName(7), file_with("A"))])],
            "快照集合取被抛弃那一版之前那一版（建快照那一版）的：快照 7 照在"
        );
        model
            .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
                SnapshotName(7),
                &file_with("A"),
            )
            .expect("读快照 7 读出建它那一刻的 A 不判红");
        assert_eq!(
            model.state_after_every_operation_so_far(),
            &file_with("A"),
            "以读出的 A 重新起头"
        );
    }

    /// 被抛弃的最新根就是建快照 7 的那一次发布：之前那一版（第一个文件那次发布）没有快照，重新起头之后快照 7 不在——
    /// 读它读出 A 判红，同名再建照收（判决第三节 H5；`research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1 下
    /// 调用方交「写出最新那一版的那一条」，取的是之前那一版的，不是这一版自己的）。
    #[test]
    fn a_recovery_that_abandoned_the_newest_root_written_by_a_snapshot_creation_restarts_without_that_snapshot(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
        let crash = model.crash_after_every_operation_returned();
        model
            .restart_after_a_recovery_that_abandoned_the_newest_root(
                crash,
                AbandonedRoot::WroteTheNewestVersion,
                file_with("A"),
            )
            .expect("抛弃最新根的那一次恢复不判读出的内容");
        assert_eq!(
            model.allowed_snapshot_sets(),
            vec![Snapshots::new()],
            "快照集合取建快照之前那一版的：没有快照"
        );
        let before_the_read = model.clone();
        assert_eq!(
            model.judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
                SnapshotName(7),
                &file_with("A")
            ),
            Err(SnapshotReadOutsideTheAllowedReads {
                name: SnapshotName(7),
                read_out: file_with("A"),
                allowed_reads: vec![None],
            }),
            "快照 7 随被抛弃的那一版一起没了：读出 A 判红"
        );
        assert_eq!(model, before_the_read, "判红的读不动模型");
        assert_eq!(
            model.submit(
                Operation::CreateSnapshot(SnapshotName(7)),
                OperationOutcome::Succeeded
            ),
            Ok(SubmittedOperation::Entered(OperationNumber(1))),
            "快照名 7 空着：同名再建照收"
        );
    }

    /// 攻方第三轮 Q2 H5 那一格的第一格（`research/prompts/m4-segment-code-r3-opus-output.md`，判决
    /// `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1）：写 A、发布、建快照 7，实现随后可写挂载一次，写行与暖机的根
    /// 都映到建快照那一版；抛弃最新根（暖机那一条）的那次恢复落到它前一条根，仍在建快照那一版上，快照 7 在、读出 A。
    /// 调用方交「不是写出这一版的那一条」，快照集合取这一版自己那一刻的：快照 7 照在，读它读出 A 不判红。
    #[test]
    fn a_recovery_that_abandoned_a_root_written_after_a_snapshot_creation_keeps_that_snapshot() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
        let crash = model.crash_after_every_operation_returned();
        model
            .restart_after_a_recovery_that_abandoned_the_newest_root(
                crash,
                AbandonedRoot::WrittenAfterTheNewestVersion,
                file_with("A"),
            )
            .expect("抛弃最新根的那一次恢复不判读出的内容");
        assert_eq!(
            model.allowed_snapshot_sets(),
            vec![Snapshots::from([(SnapshotName(7), file_with("A"))])],
            "被抛弃的不是写出建快照那一版的根：快照集合取这一版自己的，快照 7 照在"
        );
        model
            .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
                SnapshotName(7),
                &file_with("A"),
            )
            .expect("读快照 7 读出建它那一刻的 A 不判红");
    }

    /// 攻方第三轮 Q2 H5 那一格的第二格（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1「删快照这种一个操作写多条根的
    /// 取前后两份」）：写 A、发布、建快照 7、删快照 7（实现一次调用写意图与全部批），抛弃最后一批的那次恢复落到意图那条根，快照 7 带
    /// 销毁中记号、读照读，下一次用户改动先走空把它删完。调用方交「写出这一版的那一条」，快照集合前后两份都收：读快照 7 读出 A 照收、
    /// 只留快照在的那一份；走空之后同名再建也照收。
    #[test]
    fn a_recovery_that_abandoned_the_last_batch_of_a_snapshot_destruction_allows_the_snapshot_there_or_gone(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
        succeed(&mut model, Operation::DestroySnapshot(SnapshotName(7)));
        let crash = model.crash_after_every_operation_returned();
        model
            .restart_after_a_recovery_that_abandoned_the_newest_root(
                crash,
                AbandonedRoot::WroteTheNewestVersion,
                file_with("A"),
            )
            .expect("抛弃最新根的那一次恢复不判读出的内容");
        assert_eq!(
            model.allowed_snapshot_sets(),
            vec![
                Snapshots::from([(SnapshotName(7), file_with("A"))]),
                Snapshots::new(),
            ],
            "删快照之前那一版（快照 7 在）与删快照那一版（快照 7 不在）两份都收"
        );
        let mut read_before_the_walk = model.clone();
        read_before_the_walk
            .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
                SnapshotName(7),
                &file_with("A"),
            )
            .expect("落到意图那条根：快照 7 带销毁中记号、读出 A 照收");
        assert_eq!(
            read_before_the_walk.allowed_snapshot_sets(),
            vec![Snapshots::from([(SnapshotName(7), file_with("A"))])],
            "读出 A 之后只留快照 7 在的那一份"
        );
        assert_eq!(
            model.submit(
                Operation::CreateSnapshot(SnapshotName(7)),
                OperationOutcome::Succeeded
            ),
            Ok(SubmittedOperation::Entered(OperationNumber(1))),
            "走空把快照 7 删完之后同名再建照收"
        );
    }

    /// 调用方说被抛弃的根写出了最新那一版，而最新那一版是上一次恢复读出、重新起头的那一版（之后没有已确认的发布）：它之前那一版在
    /// 上一次恢复之前，RefFS 不记那时的快照集合，判决第三节 J1 没写取什么——返回第一版不支持，模型不动。
    #[test]
    fn a_recovery_that_abandoned_the_root_that_wrote_the_state_at_the_last_recovery_is_not_in_the_first_version_and_the_model_does_not_move(
    ) {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
        succeed(&mut model, Operation::ExplicitPublish);
        model
            .crash_and_recover(model.crash_after_every_operation_returned(), file_with("A"))
            .expect("发布之后读出 A 在允许集合里");
        let before_the_restart = model.clone();
        assert_eq!(
            model.restart_after_a_recovery_that_abandoned_the_newest_root(
                model.crash_after_every_operation_returned(),
                AbandonedRoot::WroteTheNewestVersion,
                file_with("A"),
            ),
            Err(AbandonedRootRestartNotAccepted::TheRootThatWroteTheStateAtTheLastRecoveryIsNotInTheFirstVersion),
            "允许集合只有 S_0：之前那一版在上一次恢复之前，第一版不支持"
        );
        assert_eq!(model, before_the_restart, "不收的那一次不动模型");
    }

    /// 截短（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8）：截到 3 字节留下前 3 个。
    #[test]
    fn truncating_to_a_shorter_size_keeps_the_leading_bytes() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("abcdef")));
        succeed(&mut model, Operation::Truncate(3));
        assert_eq!(
            model.state_after_every_operation_so_far(),
            &file_with("abc")
        );
    }

    /// 补 0（判决第三节 K8）：截到比现在长，多出来的读 0。
    #[test]
    fn truncating_to_a_longer_size_pads_with_zeros() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("ab")));
        succeed(&mut model, Operation::Truncate(5));
        assert_eq!(
            model.state_after_every_operation_so_far(),
            &VisibleState::OneFile(FileContent(vec![b'a', b'b', 0, 0, 0]))
        );
    }

    /// 截断之后读快照（判决第三节 K4、K8）：快照读出截之前的内容，活头是截之后的。
    #[test]
    fn a_snapshot_created_before_a_truncation_reads_the_content_before_it() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("abcdef")));
        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(65_549)));
        succeed(&mut model, Operation::Truncate(2));
        succeed(&mut model, Operation::ExplicitPublish);
        assert_eq!(
            model.allowed_reads_of_the_snapshot(SnapshotName(65_549)),
            vec![Some(file_with("abcdef"))],
            "快照读出截之前的内容"
        );
        assert_eq!(model.state_after_every_operation_so_far(), &file_with("ab"));
    }

    /// 崩在截断里（判决第三节 K2「先落目标」：意图那次 size 已是 N，崩在各批之间读出的就是截后）：截断交成内容改动加它那次发布，崩溃落在那次发布里，
    /// 允许集合恰好两样——截前与截后；截到一半（某一批的边界）的内容不在里面。
    #[test]
    fn a_crash_inside_a_truncation_allows_exactly_the_content_before_and_after_it() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("abcdef")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(&mut model, Operation::Truncate(2));
        let publish_of_the_truncation = succeed(&mut model, Operation::ExplicitPublish);
        let allowed = model
            .allowed_set_for_a_crash_in(publish_of_the_truncation)
            .expect("崩溃落在截断那次发布里");
        let allowed_states: Vec<VisibleState> = allowed
            .allowed
            .iter()
            .map(|allowed_state| allowed_state.state.clone())
            .collect();
        assert_eq!(
            allowed_states,
            vec![file_with("abcdef"), file_with("ab")],
            "截前、截后两样：{allowed:?}"
        );
        assert!(!allowed.contains(&file_with("abcd")), "截到一半的不在里面");
    }

    /// 按偏移写盖住中间一段（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节）：前后的字节不动，长度不变。
    #[test]
    fn writing_at_an_offset_inside_the_file_covers_that_range_and_keeps_the_rest() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("abcdef")));
        succeed(
            &mut model,
            Operation::WriteAtOffset {
                offset_in_bytes: 2,
                bytes: content("XY"),
            },
        );
        assert_eq!(
            model.state_after_every_operation_so_far(),
            &file_with("abXYef")
        );
    }

    /// 起点越过末尾（第二节 R5）：末尾与偏移之间读 0，文件长到写的终点。
    #[test]
    fn writing_past_the_end_fills_the_gap_with_zeros_and_extends_to_the_end_of_the_write() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("ab")));
        succeed(
            &mut model,
            Operation::WriteAtOffset {
                offset_in_bytes: 4,
                bytes: content("Z"),
            },
        );
        assert_eq!(
            model.state_after_every_operation_so_far(),
            &VisibleState::OneFile(FileContent(vec![b'a', b'b', 0, 0, b'Z']))
        );
    }

    /// 崩在按偏移写那次发布里：允许集合恰好写前、写后两样（一次发布做完，没有写到一半的内容）。
    #[test]
    fn a_crash_inside_a_write_at_an_offset_allows_exactly_the_content_before_and_after_it() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        succeed(&mut model, Operation::CreateTheFirstFile(content("abcdef")));
        succeed(&mut model, Operation::ExplicitPublish);
        succeed(
            &mut model,
            Operation::WriteAtOffset {
                offset_in_bytes: 4,
                bytes: content("XYZ"),
            },
        );
        let publish_of_the_write = succeed(&mut model, Operation::ExplicitPublish);
        let allowed = model
            .allowed_set_for_a_crash_in(publish_of_the_write)
            .expect("崩溃落在按偏移写那次发布里");
        let allowed_states: Vec<VisibleState> = allowed
            .allowed
            .iter()
            .map(|allowed_state| allowed_state.state.clone())
            .collect();
        assert_eq!(
            allowed_states,
            vec![file_with("abcdef"), file_with("abcdXYZ")],
            "写前、写后两样：{allowed:?}"
        );
    }

    /// 还没有文件时按偏移写：不收、模型不动。
    #[test]
    fn writing_at_an_offset_before_any_file_exists_is_refused_and_the_model_does_not_move() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        let before = model.clone();
        assert_eq!(
            model.submit(
                Operation::WriteAtOffset {
                    offset_in_bytes: 0,
                    bytes: content("a"),
                },
                OperationOutcome::Succeeded
            ),
            Err(OperationNotAccepted::WritingAtAnOffsetWithoutAFileIsNotInTheFirstVersion)
        );
        assert_eq!(model, before, "拒时模型不动");
    }

    /// 还没有文件时截断（判决第三节 K1：实现在任何写之前拒）：不收、模型不动。
    #[test]
    fn truncating_before_any_file_exists_is_refused_and_the_model_does_not_move() {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        let before = model.clone();
        assert_eq!(
            model.submit(Operation::Truncate(0), OperationOutcome::Succeeded),
            Err(OperationNotAccepted::TruncatingWithoutAFileIsNotInTheFirstVersion)
        );
        assert_eq!(model, before, "拒时模型不动");
    }
}
