# m3-formal-code-r2 被判的改动（2026-10-02）

底：代码轮第一轮冻结副本 `/tmp/claude-1000/m3-formal-code-r1/tree/`；第 11 批五份实现员补丁原样，E165 装置取底与今天主工作区的 diff。

## 实现员补丁 m3-formal-11-3（`/tmp/claude-1000/m3-formal-11-3/patch/`）

```diff
diff --git a/Cargo.lock b/Cargo.lock
index 5621850..0f2e66c 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -830,6 +830,10 @@ dependencies = [
  "singlefs-format",
 ]
 
+[[package]]
+name = "singlefs-reference-filesystem"
+version = "0.1.0"
+
 [[package]]
 name = "singlefs-replay-prefix"
 version = "0.1.0"
diff --git a/Cargo.toml b/Cargo.toml
index 4509df3..8296e03 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -9,6 +9,7 @@ members = [
     "crates/singlefs-harness",
     "crates/singlefs-checker",
     "crates/singlefs-checker-tier",
+    "crates/singlefs-reference-filesystem",
 ]
 exclude = ["research"]
 
diff --git a/crates/singlefs-reference-filesystem/Cargo.toml b/crates/singlefs-reference-filesystem/Cargo.toml
new file mode 100644
index 0000000..65d281c
--- /dev/null
+++ b/crates/singlefs-reference-filesystem/Cargo.toml
@@ -0,0 +1,10 @@
+[package]
+name = "singlefs-reference-filesystem"
+version = "0.1.0"
+edition = "2021"
+publish = false
+description = "RefFS：语义级的参照文件系统，只在内存里按 POSIX 操作走，判一次崩溃之后恢复读出的内容落不落在允许集合里；依赖只有 std"
+
+# 依赖闭包只许 {自己, singlefs-format}，能只用 std 就只用 std（research/prompts/m3-formal-r5-main-verification.md 第二节 U4）；
+# 不许有指向实现、池级 checker、harness 档、checker 档的 dev-dependencies。
+[dependencies]
diff --git a/crates/singlefs-reference-filesystem/src/lib.rs b/crates/singlefs-reference-filesystem/src/lib.rs
new file mode 100644
index 0000000..81dcbcb
--- /dev/null
+++ b/crates/singlefs-reference-filesystem/src/lib.rs
@@ -0,0 +1,885 @@
+//! RefFS：语义级的参照文件系统。只在内存里按 POSIX 操作走，判一次崩溃之后恢复读出的内容落不落在允许集合里。
+//!
+//! 依据：`.claude/kb/decisions/13-验证路线.md@已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义`（不与实现共用崩溃点枚举的那段代码）；
+//! `.claude/kb/decisions/17-实现分层与第三方管道.md@已定项 4：入口 = RefFS 的操作面`（fsync 的持久性承诺由 RefFS 的崩溃语义定义）。
+//! 形态是用户 2026-10-02 的定案：语义级（`research/prompts/m3-formal-7-forks.md` Y2）；按发布边界的前缀 Q、另起、单独一个包
+//! （`research/prompts/m3-formal-r4-main-verification.md` 用户定案 F1、F3、F4）；回退目标作输入
+//! （`research/prompts/m3-formal-r5-main-verification.md` 用户定案 G2）。条款候选取那份判决第二节 U1、U2、U4 与第四节第 1、2、4、8 条（零轮）。
+//!
+//! 输入只有五样，都由调用方交入：每个操作的结局、崩溃落在第几个操作里（hi）、回退目标 m、每次恢复读出的内容、
+//! 显式发布点（发布调用与 fsync 这两种操作本身）。这个包不读盘、不读录制流、不枚举崩溃点，依赖只有 std。
+//! 只管串行提交；任意子集的丢法不做（r5 判决第四节第 8 条：串行提交下出局）。
+#![forbid(unsafe_code)]
+
+/// 一个文件的全部字节。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct FileContent(pub Vec<u8>);
+
+/// POSIX 面上看得见的状态。第一版的操作只碰一个文件：还没有文件，或者有这一个文件与它的全部字节。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum VisibleState {
+    NoFile,
+    OneFile(FileContent),
+}
+
+/// 第一版的操作。崩溃不在这里，它是 [`ReferenceFileSystem::crash_and_recover`]。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum Operation {
+    CreateTheFirstFile(FileContent),
+    OverwriteTheFile(FileContent),
+    /// 发布调用：一个显式发布点。
+    ExplicitPublish,
+    /// 一个显式发布点；返回成功时抬起下界 lo。
+    Fsync,
+    /// 回退到整段历史里第 m 次已确认发布之后的内容；m 由调用方从实现报的回退根翻出来，选得对不对这里不判。
+    RollBackToConfirmedPublish(ConfirmedPublishNumber),
+}
+
+/// 调用方交入的结局。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum OperationOutcome {
+    Succeeded,
+    /// 在任何写之前拒：这一步没发生，不进操作序列、不占号。
+    RefusedBeforeAnyWrite,
+    /// 写了之后报错：这一步做没做成两可。
+    FailedAfterWriting,
+}
+
+/// 上一次恢复（还没崩溃过时是起点）之后进了操作序列的第几个操作，从 1 起；在任何写之前被拒的不占号。
+/// 崩溃本身也是一个操作，号是序列长度 + 1。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct OperationNumber(pub usize);
+
+impl OperationNumber {
+    /// 第 k 个操作施加之后的状态是 S_k。
+    pub fn state_after(self) -> StateIndex {
+        StateIndex(self.0)
+    }
+}
+
+/// 状态下标 j：S_j 是上一次恢复之后前 j 个操作施加出来的状态，S_0 是上一次恢复读出的内容（还没崩溃过时是空池）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct StateIndex(pub usize);
+
+/// 整段历史里第几次已确认的发布，从 1 起。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct ConfirmedPublishNumber(pub usize);
+
+/// 交进来的操作进没进操作序列。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SubmittedOperation {
+    Entered(OperationNumber),
+    DidNotHappen,
+}
+
+/// 交进来的操作不收；返回它时模型一个字段都没动。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum OperationNotAccepted {
+    /// 已经有文件时再建第一个文件：条款没写它在 POSIX 面上是什么，第一版不支持。
+    CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion,
+    /// 还没有文件时覆盖写：条款没写它在 POSIX 面上是什么，第一版不支持。
+    OverwritingWithoutAFileIsNotInTheFirstVersion,
+    /// 回退目标 m 不是 1 到「到这一步为止已确认的发布数」之间的数。
+    RollBackTargetIsNotAConfirmedPublish {
+        requested: ConfirmedPublishNumber,
+        confirmed_publishes_so_far: usize,
+    },
+}
+
+/// 允许集合里的一个状态与它的下标。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct AllowedState {
+    pub index: StateIndex,
+    pub state: VisibleState,
+}
+
+/// 一次崩溃之后恢复允许读出的状态：{S_j : j = lo 或 j 恰是某次显式发布的末尾，lo ≤ j ≤ hi}，按下标升序。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct AllowedSet {
+    pub lower_bound: StateIndex,
+    pub upper_bound: StateIndex,
+    pub allowed: Vec<AllowedState>,
+}
+
+impl AllowedSet {
+    pub fn contains(&self, state: &VisibleState) -> bool {
+        self.allowed
+            .iter()
+            .any(|allowed_state| &allowed_state.state == state)
+    }
+}
+
+/// 这次崩溃判不了或判红；返回它时模型一个字段都没动。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum CrashNotAccepted {
+    /// 崩溃落在的操作号不在 1 到崩溃本身的号（序列长度 + 1）之间。
+    CrashOperationIsOutsideTheOperationsSinceTheLastRecovery {
+        crash_falls_in: OperationNumber,
+        crash_operation_number: OperationNumber,
+    },
+    /// 一个操作写了之后报错，到崩溃为止它后面又起了操作：条款只说「之后的操作接在下一次恢复读出的状态上判」，
+    /// 没写报错与崩溃之间起的操作算什么，第一版不支持。
+    OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion {
+        failed_after_writing: OperationNumber,
+        operation_after_it: OperationNumber,
+    },
+    /// 恢复读出的内容不在允许集合里：判红。
+    RecoveredStateOutsideTheAllowedSet {
+        allowed: AllowedSet,
+        recovered: VisibleState,
+    },
+}
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum EnteredOutcome {
+    Succeeded,
+    FailedAfterWriting,
+}
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum EnteredKind {
+    ContentChange,
+    ExplicitPublish,
+    Fsync,
+    RollBack,
+}
+
+impl EnteredKind {
+    /// 这个操作施加之后的下标是不是一次显式发布的末尾：发布带走它之前取号成功的全部操作，末尾就是它自己的下标。
+    fn ends_a_publish(self) -> bool {
+        match self {
+            // 发布边界只取历史里的显式发布点：发布调用与 fsync（r5 判决第二节 U2 ①）；fsync 触发的发布也是发布：
+            // .claude/kb/decisions/16-发布语义.md@已定项 6：发布计数器，四个等号写成一条
+            EnteredKind::ExplicitPublish | EnteredKind::Fsync => true,
+            // 回退是挂着时的一次向前发布：.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
+            EnteredKind::RollBack => true,
+            EnteredKind::ContentChange => false,
+        }
+    }
+}
+
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct EnteredOperation {
+    kind: EnteredKind,
+    outcome: EnteredOutcome,
+    state_after: VisibleState,
+}
+
+/// 序列里已确认的发布（返回成功的发布调用、fsync 与回退）各自发布之后的内容，按历史次序。
+fn confirmed_publishes_among(
+    operations: &[EnteredOperation],
+) -> impl Iterator<Item = &VisibleState> {
+    operations
+        .iter()
+        // 写了之后报错的发布两可，不算已确认：.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
+        .filter(|operation| {
+            operation.kind.ends_a_publish() && operation.outcome == EnteredOutcome::Succeeded
+        })
+        .map(|operation| &operation.state_after)
+}
+
+/// 允许集合 Q。调用方已核过 1 ≤ hi ≤ 序列长度 + 1。
+fn allowed_set_for_a_crash_in(
+    state_at_the_last_recovery: &VisibleState,
+    operations: &[EnteredOperation],
+    crash_falls_in: OperationNumber,
+) -> AllowedSet {
+    let state_at = |index: StateIndex| -> VisibleState {
+        match index.0.checked_sub(1) {
+            None => state_at_the_last_recovery.clone(),
+            Some(position) => operations
+                .get(position)
+                .expect("下标不超过 hi − 1，而调用方核过 hi ≤ 序列长度 + 1")
+                .state_after
+                .clone(),
+        }
+    };
+    // 崩溃落在第 hi 个操作里：前 hi − 1 个返回过；第 hi 个没返回，它是 fsync 也不抬下界
+    // （r5 判决第二节 U1：返回成功的 fsync 才抬）。
+    let returned_before_the_crash = &operations[..crash_falls_in.0 - 1];
+    // lo = 最后一次返回成功的 fsync 等到的那次发布的末尾；串行下就是 fsync 调用之前已取号成功的操作数：位置 p（从 0 起）上的
+    // fsync 是第 p + 1 个操作，它之前取号成功的操作有 p 个。没有返回成功的 fsync 时 lo = 0，起点是上一次恢复读出的内容。
+    // .claude/kb/decisions/16-发布语义.md@已定项 7：发布的持久顺序（fsync 等系统配置轮换持久之后才返回）；
+    // fsync 报错时下界停在上一次返回成功的 fsync：.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
+    let lower_bound = returned_before_the_crash
+        .iter()
+        .enumerate()
+        .rev()
+        .find(|(_, operation)| {
+            operation.kind == EnteredKind::Fsync && operation.outcome == EnteredOutcome::Succeeded
+        })
+        .map_or(StateIndex(0), |(position, _)| StateIndex(position));
+    let upper_bound = crash_falls_in.state_after();
+    // Q：j 只取 lo 与恰是某次显式发布末尾的下标，lo ≤ j ≤ hi（用户定案 F1）。一次发布整体施加或整体不施加，
+    // 切开一次发布的 S_j 不在集合里：.claude/kb/decisions/16-发布语义.md@已定项 4：一次发布整体施加或整体不施加，第二份 replay 停在句法层；
+    // .claude/kb/decisions/16-发布语义.md@已定项 12：中间状态的合法性语义改写成「根 + journal 任意前缀」；
+    // 单元级的切分（.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
+    // 「崩溃原子性的单位是一个单元」）落在一次发布里面，不在发布末尾，Q 里不出现。
+    // 判的是状态下标，不是「lo 之前的操作必须在」：回退会让早先的操作合法消失。
+    let mut allowed = vec![AllowedState {
+        index: lower_bound,
+        state: state_at(lower_bound),
+    }];
+    for (position, operation) in operations
+        .iter()
+        .enumerate()
+        .take(upper_bound.0)
+        .skip(lower_bound.0)
+    {
+        // 位置 position 上的是第 position + 1 个操作，施加之后是 S_{position+1}，lo < position + 1 ≤ hi。
+        // 写了之后报错的发布也算末尾：那次发布可在可不在（.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么）
+        if operation.kind.ends_a_publish() {
+            allowed.push(AllowedState {
+                index: StateIndex(position + 1),
+                state: operation.state_after.clone(),
+            });
+        }
+    }
+    AllowedSet {
+        lower_bound,
+        upper_bound,
+        allowed,
+    }
+}
+
+/// 参照文件系统：一段串行提交的历史，按上一次恢复切成一轮一轮。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct ReferenceFileSystem {
+    /// S_0：上一次恢复读出的内容；还没崩溃过时是空池。
+    state_at_the_last_recovery: VisibleState,
+    /// 上一次恢复之后进了序列的操作；第 k 个在位置 k − 1。
+    operations_since_the_last_recovery: Vec<EnteredOperation>,
+    /// 上一次恢复之前各轮里已确认的发布，各自发布之后的内容，按历史次序。
+    confirmed_publishes_before_the_last_recovery: Vec<VisibleState>,
+}
+
+impl ReferenceFileSystem {
+    pub fn on_a_freshly_formatted_pool() -> Self {
+        Self {
+            state_at_the_last_recovery: VisibleState::NoFile,
+            operations_since_the_last_recovery: Vec::new(),
+            confirmed_publishes_before_the_last_recovery: Vec::new(),
+        }
+    }
+
+    /// 序列里的操作全都做成时的状态。
+    fn state_after_every_operation_so_far(&self) -> &VisibleState {
+        self.operations_since_the_last_recovery
+            .last()
+            .map_or(&self.state_at_the_last_recovery, |operation| {
+                &operation.state_after
+            })
+    }
+
+    /// 回退的目标内容 = 整段历史里第 m 次已确认发布之后的内容（用户定案 G2）。
+    fn content_after_confirmed_publish(
+        &self,
+        number: ConfirmedPublishNumber,
+    ) -> Result<VisibleState, OperationNotAccepted> {
+        let confirmed: Vec<&VisibleState> = self
+            .confirmed_publishes_before_the_last_recovery
+            .iter()
+            .chain(confirmed_publishes_among(
+                &self.operations_since_the_last_recovery,
+            ))
+            .collect();
+        let not_confirmed = OperationNotAccepted::RollBackTargetIsNotAConfirmedPublish {
+            requested: number,
+            confirmed_publishes_so_far: confirmed.len(),
+        };
+        // 第 m 次从 1 起数，在第 m − 1 位：.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
+        // （回退的新根取 R_old 那一版的用户可见树）
+        let Some(position) = number.0.checked_sub(1) else {
+            return Err(not_confirmed);
+        };
+        match confirmed.get(position) {
+            Some(state) => Ok((*state).clone()),
+            None => Err(not_confirmed),
+        }
+    }
+
+    /// 交入一个操作与调用方看到的结局。
+    ///
+    /// # Errors
+    /// [`OperationNotAccepted`] 的三种，返回时模型不动。
+    pub fn submit(
+        &mut self,
+        operation: Operation,
+        outcome: OperationOutcome,
+    ) -> Result<SubmittedOperation, OperationNotAccepted> {
+        let entered_outcome = match outcome {
+            // 在任何写之前拒＝这一步没发生：不进序列、不占号（r5 判决第二节 U1「已接受」= 取号成功、没被拒）；
+            // .claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换（「在任何写之前拒」）
+            OperationOutcome::RefusedBeforeAnyWrite => return Ok(SubmittedOperation::DidNotHappen),
+            OperationOutcome::Succeeded => EnteredOutcome::Succeeded,
+            // 写了之后报错＝两可：.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
+            OperationOutcome::FailedAfterWriting => EnteredOutcome::FailedAfterWriting,
+        };
+        let state_so_far = self.state_after_every_operation_so_far();
+        let (kind, state_after) = match operation {
+            Operation::CreateTheFirstFile(content) => match state_so_far {
+                VisibleState::NoFile => (EnteredKind::ContentChange, VisibleState::OneFile(content)),
+                VisibleState::OneFile(_) => return Err(OperationNotAccepted::CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion),
+            },
+            Operation::OverwriteTheFile(content) => match state_so_far {
+                VisibleState::NoFile => return Err(OperationNotAccepted::OverwritingWithoutAFileIsNotInTheFirstVersion),
+                VisibleState::OneFile(_) => (EnteredKind::ContentChange, VisibleState::OneFile(content)),
+            },
+            Operation::ExplicitPublish => (EnteredKind::ExplicitPublish, state_so_far.clone()),
+            Operation::Fsync => (EnteredKind::Fsync, state_so_far.clone()),
+            Operation::RollBackToConfirmedPublish(number) => (
+                EnteredKind::RollBack,
+                self.content_after_confirmed_publish(number)?,
+            ),
+        };
+        self.operations_since_the_last_recovery
+            .push(EnteredOperation {
+                kind,
+                outcome: entered_outcome,
+                state_after,
+            });
+        Ok(SubmittedOperation::Entered(OperationNumber(
+            self.operations_since_the_last_recovery.len(),
+        )))
+    }
+
+    /// 崩溃落在第 hi 个操作里时的允许集合。只读，不动模型。
+    ///
+    /// # Errors
+    /// hi 越界、或写了之后报错的操作后面到崩溃为止又起了操作。
+    pub fn allowed_set_for_a_crash_in(
+        &self,
+        crash_falls_in: OperationNumber,
+    ) -> Result<AllowedSet, CrashNotAccepted> {
+        let operations = &self.operations_since_the_last_recovery;
+        // hi 由调用方交入（r5 判决用户定案 G2 那一句与第四节第 2 条的输入白名单）；落在崩溃本身里 = 序列里的操作都返回过之后才崩。
+        let crash_operation_number = OperationNumber(operations.len() + 1);
+        if crash_falls_in.0 == 0 || crash_falls_in > crash_operation_number {
+            return Err(
+                CrashNotAccepted::CrashOperationIsOutsideTheOperationsSinceTheLastRecovery {
+                    crash_falls_in,
+                    crash_operation_number,
+                },
+            );
+        }
+        let started_before_the_crash = &operations[..crash_falls_in.0.min(operations.len())];
+        // 报错之后的操作接在下一次恢复读出的状态上判（r5 判决第二节 U1）；报错之后不许照旧复用同一份内存态：
+        // .claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
+        if let Some(failed_position) = started_before_the_crash
+            .iter()
+            .position(|operation| operation.outcome == EnteredOutcome::FailedAfterWriting)
+        {
+            if failed_position + 1 < started_before_the_crash.len() {
+                return Err(CrashNotAccepted::OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion {
+                    failed_after_writing: OperationNumber(failed_position + 1),
+                    operation_after_it: OperationNumber(failed_position + 2),
+                });
+            }
+        }
+        Ok(allowed_set_for_a_crash_in(
+            &self.state_at_the_last_recovery,
+            operations,
+            crash_falls_in,
+        ))
+    }
+
+    /// 崩溃落在第 hi 个操作里，恢复读出 `recovered`：先判它在允许集合里，再以它为起点接着走。
+    ///
+    /// # Errors
+    /// [`CrashNotAccepted`] 的三种，返回时模型不动。
+    pub fn crash_and_recover(
+        &mut self,
+        crash_falls_in: OperationNumber,
+        recovered: VisibleState,
+    ) -> Result<AllowedSet, CrashNotAccepted> {
+        let allowed = self.allowed_set_for_a_crash_in(crash_falls_in)?;
+        if !allowed.contains(&recovered) {
+            return Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet {
+                allowed,
+                recovered,
+            });
+        }
+        self.restart_from_the_recovered_state(crash_falls_in, recovered);
+        Ok(allowed)
+    }
+
+    fn restart_from_the_recovered_state(
+        &mut self,
+        crash_falls_in: OperationNumber,
+        recovered: VisibleState,
+    ) {
+        // 前 hi − 1 个返回过；第 hi 个没返回，它是发布也不算已确认；第 hi 个之后的没发生。
+        let returned_before_the_crash =
+            &self.operations_since_the_last_recovery[..crash_falls_in.0 - 1];
+        let confirmed: Vec<VisibleState> = confirmed_publishes_among(returned_before_the_crash)
+            .cloned()
+            .collect();
+        self.confirmed_publishes_before_the_last_recovery
+            .extend(confirmed);
+        // 每次恢复之后以读出的内容为起点接着走（r5 判决第二节 U2 ②）
+        self.state_at_the_last_recovery = recovered;
+        self.operations_since_the_last_recovery.clear();
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn file_with(text: &str) -> VisibleState {
+        VisibleState::OneFile(FileContent(text.as_bytes().to_vec()))
+    }
+
+    fn content(text: &str) -> FileContent {
+        FileContent(text.as_bytes().to_vec())
+    }
+
+    /// 交一个结局是 `outcome` 的操作，它必须进序列；返回它的号。
+    fn enter(
+        model: &mut ReferenceFileSystem,
+        operation: Operation,
+        outcome: OperationOutcome,
+    ) -> OperationNumber {
+        match model.submit(operation, outcome) {
+            Ok(SubmittedOperation::Entered(number)) => number,
+            Ok(SubmittedOperation::DidNotHappen) => {
+                panic!("这一步的结局不是「写之前拒」，必须进序列")
+            }
+            Err(refusal) => panic!("这一步的输入合法，不该被拒：{refusal:?}"),
+        }
+    }
+
+    fn succeed(model: &mut ReferenceFileSystem, operation: Operation) -> OperationNumber {
+        enter(model, operation, OperationOutcome::Succeeded)
+    }
+
+    fn allowed_indices(allowed: &AllowedSet) -> Vec<usize> {
+        allowed
+            .allowed
+            .iter()
+            .map(|allowed_state| allowed_state.index.0)
+            .collect()
+    }
+
+    #[test]
+    fn an_overwrite_refused_before_any_write_did_not_happen_so_the_publish_after_it_does_not_allow_its_content(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::Fsync);
+        let refused = model.submit(
+            Operation::OverwriteTheFile(content("B")),
+            OperationOutcome::RefusedBeforeAnyWrite,
+        );
+        assert_eq!(
+            refused,
+            Ok(SubmittedOperation::DidNotHappen),
+            "写之前拒的不进序列"
+        );
+        let publish = succeed(&mut model, Operation::ExplicitPublish);
+        assert_eq!(
+            publish,
+            OperationNumber(3),
+            "写之前拒的不占号，发布调用是第 3 个"
+        );
+        let allowed = model
+            .allowed_set_for_a_crash_in(OperationNumber(4))
+            .expect("崩溃落在崩溃本身里，号 4 合法");
+        assert!(
+            allowed.contains(&file_with("A")),
+            "fsync 之后的 A 必须允许：{allowed:?}"
+        );
+        assert!(
+            !allowed.contains(&file_with("B")),
+            "被拒的覆盖写没发生，B 不许出现：{allowed:?}"
+        );
+    }
+
+    #[test]
+    fn a_crash_after_two_overwrites_and_one_publish_does_not_allow_the_state_that_cuts_the_publish()
+    {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::Fsync);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        succeed(&mut model, Operation::OverwriteTheFile(content("C")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        let allowed = model
+            .allowed_set_for_a_crash_in(OperationNumber(6))
+            .expect("号 6 是崩溃本身");
+        assert_eq!(
+            allowed_indices(&allowed),
+            vec![1, 2, 5],
+            "只取 lo 与发布末尾：{allowed:?}"
+        );
+        assert!(
+            !allowed.contains(&file_with("B")),
+            "B 切开了带走 B、C 的那次发布：{allowed:?}"
+        );
+        assert!(
+            allowed.contains(&file_with("C")),
+            "发布末尾的 C 允许：{allowed:?}"
+        );
+    }
+
+    #[test]
+    fn after_a_successful_fsync_the_state_just_before_the_fsync_call_is_the_lower_bound() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::Fsync);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        succeed(&mut model, Operation::Fsync);
+        let allowed = model
+            .allowed_set_for_a_crash_in(OperationNumber(5))
+            .expect("号 5 是崩溃本身");
+        assert_eq!(
+            allowed.lower_bound,
+            StateIndex(3),
+            "第 4 个操作是 fsync，它之前取号成功的有 3 个"
+        );
+        assert!(
+            !allowed.contains(&file_with("A")),
+            "第二次 fsync 返回之后 A 不许再出现：{allowed:?}"
+        );
+        assert!(allowed.contains(&file_with("B")), "B 必须允许：{allowed:?}");
+    }
+
+    #[test]
+    fn a_failed_fsync_keeps_the_lower_bound_at_the_previous_successful_fsync() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::Fsync);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        enter(
+            &mut model,
+            Operation::Fsync,
+            OperationOutcome::FailedAfterWriting,
+        );
+        let allowed = model
+            .allowed_set_for_a_crash_in(OperationNumber(5))
+            .expect("报错的 fsync 是崩溃前最后一个操作，判得了");
+        assert_eq!(allowed.lower_bound, StateIndex(1), "下界停在第一次 fsync");
+        assert!(
+            allowed.contains(&file_with("A")),
+            "报错那次发布两可，A 允许：{allowed:?}"
+        );
+        assert!(
+            allowed.contains(&file_with("B")),
+            "报错那次发布两可，B 允许：{allowed:?}"
+        );
+    }
+
+    #[test]
+    fn a_publish_that_failed_after_writing_may_be_seen_after_the_crash() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::Fsync);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        enter(
+            &mut model,
+            Operation::ExplicitPublish,
+            OperationOutcome::FailedAfterWriting,
+        );
+        let allowed = model
+            .allowed_set_for_a_crash_in(OperationNumber(5))
+            .expect("报错的发布是崩溃前最后一个操作，判得了");
+        assert_eq!(
+            allowed_indices(&allowed),
+            vec![1, 2, 4],
+            "报错的发布末尾也在集合里：{allowed:?}"
+        );
+        assert!(
+            allowed.contains(&file_with("B")),
+            "报错那次发布可在，B 允许：{allowed:?}"
+        );
+    }
+
+    #[test]
+    fn a_crash_inside_an_overwrite_does_not_allow_the_publish_that_comes_after_it() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::Fsync);
+        let overwrite = succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        let allowed = model
+            .allowed_set_for_a_crash_in(overwrite)
+            .expect("崩溃落在第 3 个操作里");
+        assert_eq!(allowed.upper_bound, StateIndex(3), "hi 是覆盖写那一步");
+        assert_eq!(
+            allowed_indices(&allowed),
+            vec![1, 2],
+            "崩溃之后的发布没发生：{allowed:?}"
+        );
+        assert!(
+            !allowed.contains(&file_with("B")),
+            "崩在覆盖写里，它后面那次发布带不出 B：{allowed:?}"
+        );
+    }
+
+    #[test]
+    fn a_crash_inside_an_fsync_does_not_raise_the_lower_bound() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        let fsync = succeed(&mut model, Operation::Fsync);
+        let allowed = model
+            .allowed_set_for_a_crash_in(fsync)
+            .expect("崩溃落在第 4 个操作 fsync 里");
+        assert_eq!(allowed.lower_bound, StateIndex(0), "fsync 没返回，下界不抬");
+        assert!(
+            allowed.contains(&file_with("A")),
+            "发布调用之后的 A 允许：{allowed:?}"
+        );
+        assert!(
+            allowed.contains(&file_with("B")),
+            "在途的 fsync 可能已发布，B 允许：{allowed:?}"
+        );
+    }
+
+    #[test]
+    fn after_a_recovery_the_next_crash_is_judged_from_the_recovered_content_so_the_dropped_overwrite_does_not_come_back(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::Fsync);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        model
+            .crash_and_recover(OperationNumber(4), file_with("A"))
+            .expect("没发布的 B 丢掉，读出 A 合法");
+        let fsync = succeed(&mut model, Operation::Fsync);
+        assert_eq!(fsync, OperationNumber(1), "恢复之后从 1 重新编号");
+        let crash = OperationNumber(fsync.0 + 1);
+        let refused = model.crash_and_recover(crash, file_with("B"));
+        assert!(
+            matches!(
+                refused,
+                Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { .. })
+            ),
+            "丢掉的 B 不许在下一次恢复里复活：{refused:?}"
+        );
+        model
+            .crash_and_recover(crash, file_with("A"))
+            .expect("从读出的 A 起头，fsync 之后读出 A 合法");
+    }
+
+    #[test]
+    fn a_recovered_state_outside_the_allowed_set_is_refused_and_the_model_does_not_move() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::Fsync);
+        let before = model.clone();
+        let refused = model.crash_and_recover(OperationNumber(3), VisibleState::NoFile);
+        assert!(
+            matches!(
+                &refused,
+                Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet {
+                    recovered: VisibleState::NoFile,
+                    ..
+                })
+            ),
+            "fsync 之后读回空池必须判红：{refused:?}"
+        );
+        assert_eq!(model, before, "判红时模型不动");
+    }
+
+    #[test]
+    fn rolling_back_to_the_first_confirmed_publish_gives_the_content_after_that_publish() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::OverwriteTheFile(content("C")));
+        let rollback = succeed(
+            &mut model,
+            Operation::RollBackToConfirmedPublish(ConfirmedPublishNumber(1)),
+        );
+        let allowed = model
+            .allowed_set_for_a_crash_in(OperationNumber(rollback.0 + 1))
+            .expect("号 7 是崩溃本身");
+        let after_the_rollback = allowed
+            .allowed
+            .iter()
+            .find(|allowed_state| allowed_state.index == rollback.state_after())
+            .expect("回退是一次发布，它的下标在集合里");
+        assert_eq!(
+            after_the_rollback.state,
+            file_with("A"),
+            "第 1 次已确认发布之后是 A"
+        );
+    }
+
+    #[test]
+    fn a_crash_right_after_a_rollback_allows_the_rolled_back_content_even_below_the_fsynced_state()
+    {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        succeed(&mut model, Operation::Fsync);
+        succeed(
+            &mut model,
+            Operation::RollBackToConfirmedPublish(ConfirmedPublishNumber(1)),
+        );
+        let allowed = model
+            .allowed_set_for_a_crash_in(OperationNumber(6))
+            .expect("号 6 是崩溃本身");
+        assert_eq!(allowed.lower_bound, StateIndex(3), "fsync 把下界抬到 B");
+        assert!(
+            allowed.contains(&file_with("A")),
+            "回退那次发布的末尾是 A，允许：{allowed:?}"
+        );
+    }
+
+    #[test]
+    fn a_publish_that_failed_after_writing_is_not_counted_as_a_confirmed_publish_for_rollback() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        enter(
+            &mut model,
+            Operation::ExplicitPublish,
+            OperationOutcome::FailedAfterWriting,
+        );
+        model
+            .crash_and_recover(OperationNumber(5), file_with("B"))
+            .expect("报错那次发布可在，读出 B 合法");
+        let refused = model.submit(
+            Operation::RollBackToConfirmedPublish(ConfirmedPublishNumber(2)),
+            OperationOutcome::Succeeded,
+        );
+        assert_eq!(
+            refused,
+            Err(OperationNotAccepted::RollBackTargetIsNotAConfirmedPublish {
+                requested: ConfirmedPublishNumber(2),
+                confirmed_publishes_so_far: 1,
+            }),
+            "报错的发布不算已确认，只有 1 次"
+        );
+    }
+
+    #[test]
+    fn a_publish_the_crash_fell_inside_is_not_counted_as_a_confirmed_publish_for_rollback() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        let publish = succeed(&mut model, Operation::ExplicitPublish);
+        model
+            .crash_and_recover(publish, file_with("B"))
+            .expect("崩在发布调用里，读出 B 合法");
+        let refused = model.submit(
+            Operation::RollBackToConfirmedPublish(ConfirmedPublishNumber(2)),
+            OperationOutcome::Succeeded,
+        );
+        assert_eq!(
+            refused,
+            Err(OperationNotAccepted::RollBackTargetIsNotAConfirmedPublish {
+                requested: ConfirmedPublishNumber(2),
+                confirmed_publishes_so_far: 1,
+            }),
+            "崩溃落在里面的发布没返回，不算已确认"
+        );
+    }
+
+    #[test]
+    fn an_overwrite_after_a_failed_fsync_before_the_next_recovery_is_refused_as_not_in_the_first_version_and_the_model_does_not_move(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        enter(
+            &mut model,
+            Operation::Fsync,
+            OperationOutcome::FailedAfterWriting,
+        );
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        let before = model.clone();
+        let refused = model.crash_and_recover(OperationNumber(4), file_with("A"));
+        assert_eq!(
+            refused,
+            Err(CrashNotAccepted::OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion {
+                failed_after_writing: OperationNumber(2),
+                operation_after_it: OperationNumber(3),
+            }),
+            "报错之后、恢复之前又起的操作条款没定"
+        );
+        assert_eq!(model, before, "拒时模型不动");
+    }
+
+    #[test]
+    fn a_crash_number_beyond_the_crash_itself_or_zero_is_refused_and_the_model_does_not_move() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        let before = model.clone();
+        for crash_falls_in in [OperationNumber(0), OperationNumber(3)] {
+            let refused = model.crash_and_recover(crash_falls_in, file_with("A"));
+            assert_eq!(
+                refused,
+                Err(
+                    CrashNotAccepted::CrashOperationIsOutsideTheOperationsSinceTheLastRecovery {
+                        crash_falls_in,
+                        crash_operation_number: OperationNumber(2),
+                    }
+                ),
+                "序列只有 1 个操作，崩溃本身是第 2 个"
+            );
+        }
+        assert_eq!(model, before, "拒时模型不动");
+    }
+
+    #[test]
+    fn creating_the_first_file_twice_is_refused_and_the_model_does_not_move() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        let before = model.clone();
+        let refused = model.submit(
+            Operation::CreateTheFirstFile(content("B")),
+            OperationOutcome::Succeeded,
+        );
+        assert_eq!(
+            refused,
+            Err(OperationNotAccepted::CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion),
+            "已经有文件"
+        );
+        assert_eq!(model, before, "拒时模型不动");
+    }
+
+    #[test]
+    fn overwriting_before_any_file_exists_is_refused_and_the_model_does_not_move() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        let before = model.clone();
+        let refused = model.submit(
+            Operation::OverwriteTheFile(content("B")),
+            OperationOutcome::Succeeded,
+        );
+        assert_eq!(
+            refused,
+            Err(OperationNotAccepted::OverwritingWithoutAFileIsNotInTheFirstVersion),
+            "还没有文件"
+        );
+        assert_eq!(model, before, "拒时模型不动");
+    }
+
+    #[test]
+    fn rolling_back_to_confirmed_publish_zero_is_refused_and_the_model_does_not_move() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        let before = model.clone();
+        let refused = model.submit(
+            Operation::RollBackToConfirmedPublish(ConfirmedPublishNumber(0)),
+            OperationOutcome::Succeeded,
+        );
+        assert_eq!(
+            refused,
+            Err(OperationNotAccepted::RollBackTargetIsNotAConfirmedPublish {
+                requested: ConfirmedPublishNumber(0),
+                confirmed_publishes_so_far: 1,
+            }),
+            "第几次从 1 起数"
+        );
+        assert_eq!(model, before, "拒时模型不动");
+    }
+}
```

### mutations-append.tsv

```
RefFS：写之前拒的操作也进序列（被拒的覆盖写在之后的发布里出现）	crates/singlefs-reference-filesystem/src/lib.rs	OperationOutcome::RefusedBeforeAnyWrite => return Ok(SubmittedOperation::DidNotHappen),	OperationOutcome::RefusedBeforeAnyWrite => EnteredOutcome::Succeeded,	-p singlefs-reference-filesystem --lib	an_overwrite_refused_before_any_write_did_not_happen_so_the_publish_after_it_does_not_allow_its_content
RefFS：允许集合放进切开一次发布的 S_j（不看发布末尾）	crates/singlefs-reference-filesystem/src/lib.rs	if operation.kind.ends_a_publish() {	if true {	-p singlefs-reference-filesystem --lib	a_crash_after_two_overwrites_and_one_publish_does_not_allow_the_state_that_cuts_the_publish
RefFS：lo 取 fsync 调用前一刻再往前一个操作	crates/singlefs-reference-filesystem/src/lib.rs	.map_or(StateIndex(0), |(position, _)| StateIndex(position));	.map_or(StateIndex(0), |(position, _)| StateIndex(position.saturating_sub(1)));	-p singlefs-reference-filesystem --lib	after_a_successful_fsync_the_state_just_before_the_fsync_call_is_the_lower_bound
RefFS：fsync 报错时把 lo 抬到报错那次	crates/singlefs-reference-filesystem/src/lib.rs	operation.kind == EnteredKind::Fsync && operation.outcome == EnteredOutcome::Succeeded	operation.kind == EnteredKind::Fsync	-p singlefs-reference-filesystem --lib	a_failed_fsync_keeps_the_lower_bound_at_the_previous_successful_fsync
RefFS：写了之后报错的发布不算发布末尾（两可丢了「可在」那一半）	crates/singlefs-reference-filesystem/src/lib.rs	if operation.kind.ends_a_publish() {	if operation.kind.ends_a_publish() && operation.outcome == EnteredOutcome::Succeeded {	-p singlefs-reference-filesystem --lib	a_publish_that_failed_after_writing_may_be_seen_after_the_crash
RefFS：允许集合不截在 hi（崩溃之后的发布也算）	crates/singlefs-reference-filesystem/src/lib.rs	.take(upper_bound.0)	.take(operations.len())	-p singlefs-reference-filesystem --lib	a_crash_inside_an_overwrite_does_not_allow_the_publish_that_comes_after_it
RefFS：崩溃落在里面的 fsync 也抬 lo	crates/singlefs-reference-filesystem/src/lib.rs	let returned_before_the_crash = &operations[..crash_falls_in.0 - 1];	let returned_before_the_crash = &operations[..crash_falls_in.0.min(operations.len())];	-p singlefs-reference-filesystem --lib	a_crash_inside_an_fsync_does_not_raise_the_lower_bound
RefFS：恢复之后不按读出内容重新起头（接着原来的直线编号）	crates/singlefs-reference-filesystem/src/lib.rs	self.state_at_the_last_recovery = recovered;\n        self.operations_since_the_last_recovery.clear();	let _ = recovered;	-p singlefs-reference-filesystem --lib	after_a_recovery_the_next_crash_is_judged_from_the_recovered_content_so_the_dropped_overwrite_does_not_come_back
RefFS：恢复读出的内容不判允许集合	crates/singlefs-reference-filesystem/src/lib.rs	if !allowed.contains(&recovered) {	if false {	-p singlefs-reference-filesystem --lib	a_recovered_state_outside_the_allowed_set_is_refused_and_the_model_does_not_move
RefFS：回退取第 m+1 次已确认发布	crates/singlefs-reference-filesystem/src/lib.rs	let Some(position) = number.0.checked_sub(1) else {	let Some(position) = number.0.checked_add(0) else {	-p singlefs-reference-filesystem --lib	rolling_back_to_the_first_confirmed_publish_gives_the_content_after_that_publish
RefFS：回退不算发布末尾	crates/singlefs-reference-filesystem/src/lib.rs	EnteredKind::RollBack => true,	EnteredKind::RollBack => false,	-p singlefs-reference-filesystem --lib	a_crash_right_after_a_rollback_allows_the_rolled_back_content_even_below_the_fsynced_state
RefFS：写了之后报错的发布也算已确认	crates/singlefs-reference-filesystem/src/lib.rs	operation.kind.ends_a_publish() && operation.outcome == EnteredOutcome::Succeeded	operation.kind.ends_a_publish()	-p singlefs-reference-filesystem --lib	a_publish_that_failed_after_writing_is_not_counted_as_a_confirmed_publish_for_rollback
RefFS：崩溃落在里面的发布也算已确认	crates/singlefs-reference-filesystem/src/lib.rs	&self.operations_since_the_last_recovery[..crash_falls_in.0 - 1];	&self.operations_since_the_last_recovery[..crash_falls_in.0.min(self.operations_since_the_last_recovery.len())];	-p singlefs-reference-filesystem --lib	a_publish_the_crash_fell_inside_is_not_counted_as_a_confirmed_publish_for_rollback
RefFS：写了之后报错、恢复之前又起的操作照判（不按条款没定拒）	crates/singlefs-reference-filesystem/src/lib.rs	if failed_position + 1 < started_before_the_crash.len() {	if false {	-p singlefs-reference-filesystem --lib	an_overwrite_after_a_failed_fsync_before_the_next_recovery_is_refused_as_not_in_the_first_version_and_the_model_does_not_move
RefFS：崩溃号超过崩溃本身不拒	crates/singlefs-reference-filesystem/src/lib.rs	if crash_falls_in.0 == 0 || crash_falls_in > crash_operation_number {	if crash_falls_in.0 == 0 {	-p singlefs-reference-filesystem --lib	a_crash_number_beyond_the_crash_itself_or_zero_is_refused_and_the_model_does_not_move
RefFS：已有文件时再建第一个文件照收	crates/singlefs-reference-filesystem/src/lib.rs	VisibleState::OneFile(_) => return Err(OperationNotAccepted::CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion),	VisibleState::OneFile(_) => (EnteredKind::ContentChange, VisibleState::OneFile(content)),	-p singlefs-reference-filesystem --lib	creating_the_first_file_twice_is_refused_and_the_model_does_not_move
RefFS：还没有文件时覆盖写照收	crates/singlefs-reference-filesystem/src/lib.rs	VisibleState::NoFile => return Err(OperationNotAccepted::OverwritingWithoutAFileIsNotInTheFirstVersion),	VisibleState::NoFile => (EnteredKind::ContentChange, VisibleState::OneFile(content)),	-p singlefs-reference-filesystem --lib	overwriting_before_any_file_exists_is_refused_and_the_model_does_not_move
RefFS：回退目标第 0 次当第 1 次收	crates/singlefs-reference-filesystem/src/lib.rs	let Some(position) = number.0.checked_sub(1) else {	let Some(position) = Some(number.0.saturating_sub(1)) else {	-p singlefs-reference-filesystem --lib	rolling_back_to_confirmed_publish_zero_is_refused_and_the_model_does_not_move
```

## 实现员补丁 m3-formal-11-4（`/tmp/claude-1000/m3-formal-11-4/patch/`）

```diff
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 075bbb4..4dafc93 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -19,6 +19,10 @@
 //! （与之后那次发布）的录制流（[`HistoryThenWritableMountRecords`]），历史那几次发布在挂载之后的池与二次崩溃镜像上还在不在也核得到。
 //! 每个二次崩溃镜像上再起一次可写挂载，判 I-8.4（重放幂等）的 R-换：「恢复加写行那一整段」做两次（第一次崩在半途）与做一次，
 //! 读出的内容相同（[`compare_content_read_once_and_twice`]；不比扇区、实例代号、实例表行）。
+//!
+//! 挂载透明（独立的第四条，判法在 `singlefs_harness::mount_transparency`；r5 判决第二节 U5、U6，用户 2026-10-02 定 Y7「崩溃注入里量」、F6「加二次」）：
+//! 挂载做成的每个崩溃状态上，崩溃镜像的只读恢复（R1）与只挂载之后、挂载加一次发布之后的只读恢复（R2，后者比发布之前那一版）逐字段比，
+//! 挂载途中每个二次崩溃镜像的只读恢复也与 R1 比；可写挂载被拒的，理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的判拒绝本身合不合法。
 
 use std::collections::{BTreeMap, BTreeSet};
 use std::fmt::Write as _;
@@ -36,9 +40,10 @@ use singlefs_core::address::{
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
-use singlefs_core::mount::mount_writable_with_space_admission;
+use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
 use singlefs_core::recovery::{
     choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
+    RecoveryReport,
 };
 use singlefs_core::transaction::{
     publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
@@ -67,6 +72,12 @@ use singlefs_harness::model_comparison::{
     model_root_key, observed_read_back_after_a_crash, refusal_reason_of_mount_error,
     refusal_reason_of_publish_error,
 };
+use singlefs_harness::mount_transparency::{
+    judge_mount_transparency, judge_writable_mount_refusal, read_recovery_fields, ComparedVersion,
+    DiskAfterTheRefusal, FieldOutcome, FloorRaisedByTheMount, FloorRaisesAtTheSecondCrash,
+    MountTransparency, MountTransparencyAction, ReadOnlyReadBackOfTheCrashImage,
+    RecoveryFieldsRead, WritableMountRefusalVerdict,
+};
 use singlefs_harness::numeric_environment_variable::{
     number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
 };
@@ -277,6 +288,30 @@ impl WritableMountPhase {
     }
 }
 
+/// 第一次崩溃镜像上挂载透明比的是哪个动作之后的池（`singlefs_harness::mount_transparency::MountTransparencyAction` 的前两种）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum MountTransparencyComparison {
+    /// 只挂载（取号、写行、暖机与挂载推的抬 F），不接发布：v2a。
+    TheMountAlone,
+    /// 挂载加一次发布，比发布之前那一版。
+    TheMountAndOnePublish,
+}
+
+impl MountTransparencyComparison {
+    pub const ALL: [MountTransparencyComparison; 2] = [
+        MountTransparencyComparison::TheMountAlone,
+        MountTransparencyComparison::TheMountAndOnePublish,
+    ];
+
+    #[must_use]
+    pub fn name(self) -> &'static str {
+        match self {
+            MountTransparencyComparison::TheMountAlone => "the_mount_alone",
+            MountTransparencyComparison::TheMountAndOnePublish => "the_mount_and_one_publish",
+        }
+    }
+}
+
 /// 一个崩溃状态上的判定落在哪一截（代码审阅第 2 条，用户 2026-09-27 定案「崩溃注入层补可写挂载」）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum CrashStateStage {
@@ -289,6 +324,9 @@ pub enum CrashStateStage {
     SecondCrashInsideTheWritableMount(WritableMountPhase),
     /// 二次崩溃镜像上再起一次可写挂载之后的池：读出的内容要与没崩过的那一次挂载之后读出的相同（I-8.4 的 R-换）。
     RemountAfterASecondCrashInsideTheWritableMount(WritableMountPhase),
+    /// 挂载透明：崩溃镜像上做了这个动作之后的池，只读恢复与崩溃镜像上的只读恢复逐字段比。二次崩溃镜像上的那一判记在
+    /// [`CrashStateStage::SecondCrashInsideTheWritableMount`] 那一截的违例里。
+    MountTransparencyAfterTheCrash(MountTransparencyComparison),
 }
 
 impl CrashStateStage {
@@ -306,6 +344,9 @@ impl CrashStateStage {
             CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(phase) => {
                 format!("remount_after_a_second_crash_inside_{}", phase.name())
             }
+            CrashStateStage::MountTransparencyAfterTheCrash(comparison) => {
+                format!("mount_transparency_of_{}", comparison.name())
+            }
         }
     }
 }
@@ -449,6 +490,16 @@ pub struct CrashInjectionTally {
     /// 每一截（崩溃镜像本身、挂载与一次发布之后、二次崩溃）判到已知红与新发现的状态数之和。
     pub crash_states_ending_known_red: BTreeMap<usize, u64>,
     pub crash_states_ending_new_finding: u64,
+    /// 挂载透明：第一次崩溃镜像上按动作判了几次、二次崩溃镜像上按崩在挂载的哪一段判了几次；整个动作没判的按理由分；
+    /// 判红的按字段分（名字里带「量过」或「推的」，提交时的快档一眼分得开）；某一样没判的按理由分。
+    pub mount_transparency_judged: BTreeMap<MountTransparencyComparison, u64>,
+    pub mount_transparency_judged_on_second_crashes_by_phase: BTreeMap<WritableMountPhase, u64>,
+    pub mount_transparency_not_judged: BTreeMap<&'static str, u64>,
+    pub mount_transparency_violations_by_field: BTreeMap<&'static str, u64>,
+    pub mount_transparency_fields_not_judged: BTreeMap<&'static str, u64>,
+    /// 崩溃后镜像上的可写挂载被拒、理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的：拒得合法的、不合法的（判红）。
+    pub writable_mount_refusals_on_the_d23_list_legal: u64,
+    pub writable_mount_refusals_on_the_d23_list_illegal: u64,
 }
 
 impl CrashInjectionTally {
@@ -505,6 +556,13 @@ impl CrashInjectionTally {
             remount_idempotence_not_judged,
             crash_states_ending_known_red,
             crash_states_ending_new_finding,
+            mount_transparency_judged,
+            mount_transparency_judged_on_second_crashes_by_phase,
+            mount_transparency_not_judged,
+            mount_transparency_violations_by_field,
+            mount_transparency_fields_not_judged,
+            writable_mount_refusals_on_the_d23_list_legal,
+            writable_mount_refusals_on_the_d23_list_illegal,
         } = following;
         self.histories += histories;
         self.histories_completed += histories_completed;
@@ -595,6 +653,40 @@ impl CrashInjectionTally {
             *self.crash_states_ending_known_red.entry(*form).or_insert(0) += count;
         }
         self.crash_states_ending_new_finding += crash_states_ending_new_finding;
+        for (comparison, count) in mount_transparency_judged {
+            *self
+                .mount_transparency_judged
+                .entry(*comparison)
+                .or_insert(0) += count;
+        }
+        for (phase, count) in mount_transparency_judged_on_second_crashes_by_phase {
+            *self
+                .mount_transparency_judged_on_second_crashes_by_phase
+                .entry(*phase)
+                .or_insert(0) += count;
+        }
+        for (reason, count) in mount_transparency_not_judged {
+            *self
+                .mount_transparency_not_judged
+                .entry(reason)
+                .or_insert(0) += count;
+        }
+        for (field, count) in mount_transparency_violations_by_field {
+            *self
+                .mount_transparency_violations_by_field
+                .entry(field)
+                .or_insert(0) += count;
+        }
+        for (reason, count) in mount_transparency_fields_not_judged {
+            *self
+                .mount_transparency_fields_not_judged
+                .entry(reason)
+                .or_insert(0) += count;
+        }
+        self.writable_mount_refusals_on_the_d23_list_legal +=
+            writable_mount_refusals_on_the_d23_list_legal;
+        self.writable_mount_refusals_on_the_d23_list_illegal +=
+            writable_mount_refusals_on_the_d23_list_illegal;
     }
 
     /// 给人看的一整块：绝对数——摆了几个崩溃状态、恢复成了几次、checker 与记录核对器跑了几次、模型判了几次、新发现几条。
@@ -703,6 +795,27 @@ impl CrashInjectionTally {
                 .collect::<Vec<_>>(),
             self.remount_idempotence_not_judged
         );
+        let _ = writeln!(
+            text,
+            "挂载透明：第一次崩溃镜像上判了 {:?}，二次崩溃镜像上按段判了 {:?}；没判的 {:?}；判红的字段 {:?}；某一样没判的 {:?}",
+            self.mount_transparency_judged
+                .iter()
+                .map(|(comparison, count)| (comparison.name(), *count))
+                .collect::<Vec<_>>(),
+            self.mount_transparency_judged_on_second_crashes_by_phase
+                .iter()
+                .map(|(phase, count)| (phase.name(), *count))
+                .collect::<Vec<_>>(),
+            self.mount_transparency_not_judged,
+            self.mount_transparency_violations_by_field,
+            self.mount_transparency_fields_not_judged
+        );
+        let _ = writeln!(
+            text,
+            "可写挂载被拒、理由在 D23 已定项 14 那张单子上的：拒得合法 {}、不合法 {}",
+            self.writable_mount_refusals_on_the_d23_list_legal,
+            self.writable_mount_refusals_on_the_d23_list_illegal
+        );
         let _ = writeln!(
             text,
             "崩溃状态：以已知红收尾 {:?}、新发现 {}",
@@ -887,7 +1000,10 @@ pub fn inject_crashes_into_history(
             tally.crash_points_inside_an_unfinished_publish += 1;
         }
 
-        let report = recover(&image, JournalPolicy::Consult);
+        // 只读恢复连同挂载透明要比的字段一起读（R1）：恢复报告就是这一次的。
+        let fields_read_after_the_crash =
+            read_recovery_fields(&image, ComparedVersion::TheRecoveredVersion);
+        let report = &fields_read_after_the_crash.report;
         match &report.outcome {
             RecoveryOutcome::FileRead { .. } => tally.recoveries_reading_a_file += 1,
             RecoveryOutcome::NoFile { .. } => tally.recoveries_without_a_file += 1,
@@ -896,7 +1012,7 @@ pub fn inject_crashes_into_history(
         if report.journal.prefix_applied > 0 {
             tally.recoveries_that_applied_journal_records += 1;
         }
-        let read_back = observed_read_back_after_a_crash(&report);
+        let read_back = observed_read_back_after_a_crash(report);
         tally.model_judgements += 1;
         let disagreement =
             crash_recovery_disagreement(&committed_versions, newest_persisted, &read_back);
@@ -988,14 +1104,44 @@ pub fn inject_crashes_into_history(
             &mut findings,
             &mount_run,
             record_check_after_the_mount,
+            RefusalOnTheCrashImage {
+                crash_image_materialized: &crash_image_materialized,
+                version_landed_on_after_the_crash: report.effective_root,
+                read_only_read_back: if disagreement.is_none() {
+                    ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
+                } else {
+                    ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
+                },
+            },
             history.seed,
             crash_point,
             image_directory,
         );
-        let content_after_the_uninterrupted_mount = content_read_after_the_uninterrupted_mount(
+        // 没崩过的那一次挂载之后的池（挂载那一段的写全落在第一次崩溃之后的镜像上，不接之后那一次发布）：v2a。挂载被拒时没有。
+        let fields_after_the_uninterrupted_mount = pool_after_the_uninterrupted_mount(
             &crash_image_materialized,
             &mount_run,
             records_across_the_crash.mount_writes(),
+        )
+        .map(|pool| {
+            let fields = read_recovery_fields(&pool, ComparedVersion::TheRecoveredVersion);
+            (pool, fields)
+        });
+        let content_after_the_uninterrupted_mount = match &fields_after_the_uninterrupted_mount {
+            None => Err(RemountIdempotenceNotJudged::TheUninterruptedMountWasRefused),
+            Some((_pool, fields)) => Ok(content_read_by_the_recovery_report(&fields.report)),
+        };
+        judge_mount_transparency_on_the_crash_image(
+            &mut tally,
+            &mut findings,
+            &fields_read_after_the_crash,
+            &mount_run,
+            fields_after_the_uninterrupted_mount
+                .as_ref()
+                .map(|(pool, fields)| (pool, fields)),
+            history.seed,
+            crash_point,
+            image_directory,
         );
         judge_second_crashes_inside_the_writable_mount(
             &mut tally,
@@ -1008,6 +1154,8 @@ pub fn inject_crashes_into_history(
                 content_read_back_after_the_crash,
                 newest_persisted_before_the_mount: newest_persisted,
                 content_after_the_uninterrupted_mount,
+                fields_read_after_the_first_crash: &fields_read_after_the_crash,
+                floor_raised_by_the_uninterrupted_mount: mount_run.floor_raised_by_the_mount,
                 parameters: &parameters,
                 space_admission: execution.space_admission,
             },
@@ -1134,11 +1282,18 @@ impl ContentReadByRecovery {
     }
 }
 
-/// 对 `reader` 跑一次只读恢复（看 journal），交回读出的内容。
+/// 对 `reader` 跑一次只读恢复（看 journal），交回读出的内容。要恢复报告与挂载透明的字段的，走
+/// `singlefs_harness::mount_transparency::read_recovery_fields`，再用 [`content_read_by_the_recovery_report`] 取内容。
 #[must_use]
 pub fn content_read_by_recovery(reader: &dyn PoolReader) -> ContentReadByRecovery {
-    match recover(reader, JournalPolicy::Consult).outcome {
-        RecoveryOutcome::FileRead { content, .. } => ContentReadByRecovery::File(content),
+    content_read_by_the_recovery_report(&recover(reader, JournalPolicy::Consult))
+}
+
+/// 一份只读恢复报告读出的内容。
+#[must_use]
+pub fn content_read_by_the_recovery_report(report: &RecoveryReport) -> ContentReadByRecovery {
+    match &report.outcome {
+        RecoveryOutcome::FileRead { content, .. } => ContentReadByRecovery::File(content.clone()),
         RecoveryOutcome::NoFile { .. } => ContentReadByRecovery::NoFile,
         RecoveryOutcome::Failed { .. } => ContentReadByRecovery::NothingRead,
     }
@@ -1241,21 +1396,19 @@ pub fn writable_mount_alone_on(
     }
 }
 
-/// 没崩过的那一次可写挂载之后读出的内容：挂载那一段的写全落在第一次崩溃之后的镜像上（不接之后那一次发布），再跑只读恢复。
-/// 挂载被拒时没有「做一次」的结果。
-fn content_read_after_the_uninterrupted_mount(
+/// 没崩过的那一次可写挂载之后的池：挂载那一段的写全落在第一次崩溃之后的镜像上（不接之后那一次发布）。I-8.4 的「做一次」与
+/// 挂载透明的 v2a 都在它上面只读恢复。挂载被拒时没有（None）。
+fn pool_after_the_uninterrupted_mount(
     crash_image_materialized: &MemoryPool,
     mount_run: &WritableMountOnTheCrashImage,
     mount_writes: &[RetainedWrite],
-) -> Result<ContentReadByRecovery, RemountIdempotenceNotJudged> {
+) -> Option<MemoryPool> {
     match mount_run.outcome {
-        WritableMountOutcome::MountRefused { .. } => {
-            Err(RemountIdempotenceNotJudged::TheUninterruptedMountWasRefused)
-        }
+        WritableMountOutcome::MountRefused { .. } => None,
         WritableMountOutcome::PublishRefused { .. } | WritableMountOutcome::Published => {
             let mut pool_after_the_mount = crash_image_materialized.clone();
             pool_after_the_mount.apply_writes(mount_writes);
-            Ok(content_read_by_recovery(&pool_after_the_mount))
+            Some(pool_after_the_mount)
         }
     }
 }
@@ -1263,10 +1416,11 @@ fn content_read_after_the_uninterrupted_mount(
 /// 崩溃后镜像上那次可写挂载与之后那一次发布的结局。
 #[derive(Debug)]
 pub enum WritableMountOutcome {
-    /// 挂载被拒：成员与它说的理由。
+    /// 挂载被拒：成员、它说的理由与那个错误本身（拒的理由在不在 D23 已定项 14 那张单子上按它判）。
     MountRefused {
         member: String,
         reason: ObservedRefusalReason,
+        refusal: Box<MountError>,
     },
     /// 挂载做成、之后那一次发布被拒。
     PublishRefused {
@@ -1285,6 +1439,8 @@ pub struct WritableMountOnTheCrashImage {
     /// 挂载做成之后那一次发布写出的那一段录制流（挂载被拒时是空的）：`pool_after` 上有它，第二截交给记录核对器的写表也要有它。
     pub publish_operations: Vec<RetainedOperation>,
     pub pool_after: MemoryPool,
+    /// 挂载交回值里推没推抬 F、最后抬到的 F（挂载透明要它）；挂载被拒时 None。
+    pub floor_raised_by_the_mount: Option<FloorRaisedByTheMount>,
 }
 
 /// 挂载之后那一次发布写的内容：随种子与崩溃状态变，装得进一个数据单元（模型只罩一个数据单元的文件）。
@@ -1326,10 +1482,15 @@ pub fn mount_and_publish_once_on_the_crash_image(
     let mounted =
         mount_writable_with_space_admission(parameters, &mut devices, execution.space_admission);
     let mount_operation_count = stream.operation_count();
+    let floor_raised_by_the_mount = mounted
+        .as_ref()
+        .ok()
+        .map(|mounted| FloorRaisedByTheMount::of_the_mount_output(&mounted.output));
     let outcome = match mounted {
         Err(error) => WritableMountOutcome::MountRefused {
             member: format!("{error:?}"),
             reason: refusal_reason_of_mount_error(&error),
+            refusal: Box::new(error),
         },
         Ok(mounted) => {
             let content = content_of_the_publish_after_the_crash(seed, crash_point);
@@ -1381,6 +1542,7 @@ pub fn mount_and_publish_once_on_the_crash_image(
         mount_operations,
         publish_operations,
         pool_after,
+        floor_raised_by_the_mount,
     }
 }
 
@@ -1534,28 +1696,77 @@ enum PublishAfterTheMountInTheState {
     NotWrittenYet,
 }
 
+/// 判可写挂载被拒合不合法要的：崩溃后镜像（拒的判据在它上面复算，拒之后的池要与它逐字节相同）、它上面只读恢复落到的那一版、
+/// 那一次只读读出在不在模型的允许集合里（第一截问模型的结论）。
+struct RefusalOnTheCrashImage<'crash> {
+    crash_image_materialized: &'crash MemoryPool,
+    version_landed_on_after_the_crash: Option<(InstanceGeneration, CheckpointTxg)>,
+    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
+}
+
 /// 第二截：挂载或那次发布被拒（单元区墙只计数，模型在单元区墙上只答允许拒绝的区间、而这里没有崩溃点上的模型状态可判区间），
 /// 做没做成都在之后的池上跑池级 checker；`record_check` 是同一个池上记录核对器的结论
-/// （[`HistoryThenWritableMountRecords::record_check_after_the_writable_mount`]）。别的理由被拒按「模型说该成、实现拒了」报：
+/// （[`HistoryThenWritableMountRecords::record_check_after_the_writable_mount`]）。挂载被拒而理由在 D23（journal 的角色与格式） 已定项 14
+/// 那张单子上的，判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal`：判据在崩溃镜像上复算为真、盘上逐字节不变、
+/// 只读读出在允许集合里），合法的只计数，不合法的报红。别的理由被拒按「模型说该成、实现拒了」报：
 /// 模型对可写挂载与覆盖写、第一个文件从不要求拒，容量墙之外也不许拒（`IdealModel::answer_mount_writable`）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "每个参数各是一样东西：计数与发现、挂载那一趟与它之后的记录核对、判拒绝要的崩溃镜像那一份、这一个崩溃状态落在哪"
+)]
 fn judge_the_writable_mount_on_the_crash_image(
     tally: &mut CrashInjectionTally,
     findings: &mut FindingsOfOneHistory,
     mount_run: &WritableMountOnTheCrashImage,
     record_check: RecordCheck,
+    refusal_on_the_crash_image: RefusalOnTheCrashImage<'_>,
     seed: HistorySeed,
     crash_point: &CrashPoint,
     image_directory: Option<&Path>,
 ) {
     tally.writable_mounts_after_the_crash += 1;
     let refusal = match &mount_run.outcome {
-        WritableMountOutcome::MountRefused { member, reason } => {
-            if *reason == ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall) {
-                tally.writable_mounts_after_the_crash_refused_at_the_unit_area_wall += 1;
-                None
-            } else {
-                tally.writable_mounts_after_the_crash_refused += 1;
-                Some(format!("崩溃后镜像上的可写挂载拒了：{member}"))
+        WritableMountOutcome::MountRefused {
+            member,
+            reason,
+            refusal: refusal_error,
+        } => {
+            let disk =
+                if mount_run.pool_after == *refusal_on_the_crash_image.crash_image_materialized {
+                    DiskAfterTheRefusal::ByteForByteUnchanged
+                } else {
+                    DiskAfterTheRefusal::Changed
+                };
+            match judge_writable_mount_refusal(
+                refusal_on_the_crash_image.crash_image_materialized,
+                refusal_on_the_crash_image.version_landed_on_after_the_crash,
+                refusal_error,
+                disk,
+                refusal_on_the_crash_image.read_only_read_back,
+            ) {
+                WritableMountRefusalVerdict::Legal(_) => {
+                    tally.writable_mount_refusals_on_the_d23_list_legal += 1;
+                    None
+                }
+                WritableMountRefusalVerdict::Illegal {
+                    reason: reason_on_the_list,
+                    conditions_not_met,
+                } => {
+                    tally.writable_mount_refusals_on_the_d23_list_illegal += 1;
+                    Some(format!(
+                        "崩溃后镜像上的可写挂载拒了，理由 {reason_on_the_list:?} 在 D23 已定项 14 那张单子上，拒得不合法（{conditions_not_met:?}）：{member}"
+                    ))
+                }
+                WritableMountRefusalVerdict::ReasonNotOnTheD23List => {
+                    if *reason == ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall)
+                    {
+                        tally.writable_mounts_after_the_crash_refused_at_the_unit_area_wall += 1;
+                        None
+                    } else {
+                        tally.writable_mounts_after_the_crash_refused += 1;
+                        Some(format!("崩溃后镜像上的可写挂载拒了：{member}"))
+                    }
+                }
             }
         }
         WritableMountOutcome::PublishRefused { member, reason } => {
@@ -1613,6 +1824,233 @@ fn judge_the_writable_mount_on_the_crash_image(
     );
 }
 
+/// 挂载透明整个动作没判的理由（计数名）：崩溃后镜像上的可写挂载被拒，判的是拒绝本身（[`judge_the_writable_mount_on_the_crash_image`]）。
+const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_MOUNT_WAS_REFUSED: &str = "可写挂载被拒（判拒绝本身）";
+/// 挂载做成、之后那一次发布被拒：没有「挂载加一次发布」那一版可比。
+const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_PUBLISH_AFTER_THE_MOUNT_WAS_REFUSED: &str =
+    "挂载之后那一次发布被拒";
+/// 二次崩溃镜像上：没崩过的那一次可写挂载被拒，没有挂载交回值（推没推抬 F）。
+const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_UNINTERRUPTED_MOUNT_WAS_REFUSED: &str =
+    "没崩过的那一次可写挂载被拒";
+
+/// 挂载透明的一判记进计数：整个动作没判的按理由、某一样没判的按理由、判红的按字段（名字里带量过或推的）；交回判红的那几样，
+/// 一样一条违例（名字是 [`singlefs_harness::mount_transparency::MountTransparencyField::violation_name`]）。
+fn violations_of_the_mount_transparency(
+    tally: &mut CrashInjectionTally,
+    transparency: &MountTransparency,
+) -> Vec<(&'static str, String)> {
+    let verdicts = match transparency {
+        MountTransparency::NotJudged(reason) => {
+            *tally
+                .mount_transparency_not_judged
+                .entry(reason.name())
+                .or_insert(0) += 1;
+            return Vec::new();
+        }
+        MountTransparency::Judged(verdicts) => verdicts,
+    };
+    let mut violations = Vec::new();
+    for verdict in verdicts {
+        match &verdict.outcome {
+            FieldOutcome::Holds => {}
+            FieldOutcome::NotJudged(reason) => {
+                *tally
+                    .mount_transparency_fields_not_judged
+                    .entry(reason.name())
+                    .or_insert(0) += 1;
+            }
+            FieldOutcome::Violated { expected, observed } => {
+                let name = verdict.field.violation_name(verdict.basis);
+                *tally
+                    .mount_transparency_violations_by_field
+                    .entry(name)
+                    .or_insert(0) += 1;
+                violations.push((name, format!("期望 {expected}；读出 {observed}")));
+            }
+        }
+    }
+    violations
+}
+
+/// 第一次崩溃镜像上一个动作的挂载透明：R1（`before_the_mount`）与动作之后的池（`pool_after_the_action`）上的只读恢复（`after_the_action`）逐字段比；
+/// 判红的落成这一截（[`CrashStateStage::MountTransparencyAfterTheCrash`]）的新发现，镜像留的是动作之后的池。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "每个参数各是一样东西：计数与发现、哪个动作、两次恢复与动作之后的池、动作的交回值、这一个崩溃状态落在哪"
+)]
+fn judge_mount_transparency_of_one_comparison(
+    tally: &mut CrashInjectionTally,
+    findings: &mut FindingsOfOneHistory,
+    comparison: MountTransparencyComparison,
+    before_the_mount: &RecoveryFieldsRead,
+    pool_after_the_action: &MemoryPool,
+    after_the_action: &RecoveryFieldsRead,
+    action: MountTransparencyAction,
+    seed: HistorySeed,
+    crash_point: &CrashPoint,
+    image_directory: Option<&Path>,
+) {
+    let transparency = judge_mount_transparency(before_the_mount, after_the_action, action);
+    match &transparency {
+        MountTransparency::Judged(_) => {
+            *tally
+                .mount_transparency_judged
+                .entry(comparison)
+                .or_insert(0) += 1;
+        }
+        MountTransparency::NotJudged(_) => {}
+    }
+    let violations = violations_of_the_mount_transparency(tally, &transparency);
+    if violations.is_empty() {
+        return;
+    }
+    let image = CrashImage {
+        base: pool_after_the_action,
+        writes: &[],
+        persisted: Vec::new(),
+    };
+    let detail = violations
+        .iter()
+        .map(|(name, detail)| format!("{name}：{detail}"))
+        .collect::<Vec<_>>()
+        .join("；");
+    let observation = crash_state_observation(
+        &image,
+        crash_point,
+        violations,
+        None,
+        RecordCheck::default(),
+    );
+    findings.settle(
+        tally,
+        FailureAtAStage {
+            seed,
+            crash_point,
+            stage: CrashStateStage::MountTransparencyAfterTheCrash(comparison),
+            observation,
+            read_back: detail,
+            image: &image,
+        },
+        image_directory,
+    );
+}
+
+/// 第一次崩溃镜像上的挂载透明（r5 判决第二节 U5）：只挂载（v2a，`after_the_uninterrupted_mount` 是那个池与它上面的只读恢复）、
+/// 挂载加一次发布（比发布之前那一版）各判一次。挂载被拒的不判（判的是拒绝本身），发布被拒的不判「挂载加一次发布」那一个。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "每个参数各是一样东西：计数与发现、R1、挂载那一趟与 v2a、这一个崩溃状态落在哪"
+)]
+fn judge_mount_transparency_on_the_crash_image(
+    tally: &mut CrashInjectionTally,
+    findings: &mut FindingsOfOneHistory,
+    before_the_mount: &RecoveryFieldsRead,
+    mount_run: &WritableMountOnTheCrashImage,
+    after_the_uninterrupted_mount: Option<(&MemoryPool, &RecoveryFieldsRead)>,
+    seed: HistorySeed,
+    crash_point: &CrashPoint,
+    image_directory: Option<&Path>,
+) {
+    match (
+        after_the_uninterrupted_mount,
+        mount_run.floor_raised_by_the_mount,
+    ) {
+        (Some((pool, after_the_mount)), Some(floor)) => judge_mount_transparency_of_one_comparison(
+            tally,
+            findings,
+            MountTransparencyComparison::TheMountAlone,
+            before_the_mount,
+            pool,
+            after_the_mount,
+            MountTransparencyAction::MountAlone(floor),
+            seed,
+            crash_point,
+            image_directory,
+        ),
+        (None, _) | (_, None) => {
+            *tally
+                .mount_transparency_not_judged
+                .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_MOUNT_WAS_REFUSED)
+                .or_insert(0) += 1;
+        }
+    }
+    match (&mount_run.outcome, mount_run.floor_raised_by_the_mount) {
+        (WritableMountOutcome::Published, Some(floor)) => {
+            let after_the_publish = read_recovery_fields(
+                &mount_run.pool_after,
+                ComparedVersion::TheVersionBeforeTheRecoveredOne,
+            );
+            judge_mount_transparency_of_one_comparison(
+                tally,
+                findings,
+                MountTransparencyComparison::TheMountAndOnePublish,
+                before_the_mount,
+                &mount_run.pool_after,
+                &after_the_publish,
+                MountTransparencyAction::MountAndOnePublish(floor),
+                seed,
+                crash_point,
+                image_directory,
+            );
+        }
+        (WritableMountOutcome::PublishRefused { .. }, _) => {
+            *tally
+                .mount_transparency_not_judged
+                .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_PUBLISH_AFTER_THE_MOUNT_WAS_REFUSED)
+                .or_insert(0) += 1;
+        }
+        (WritableMountOutcome::MountRefused { .. } | WritableMountOutcome::Published, _) => {
+            *tally
+                .mount_transparency_not_judged
+                .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_MOUNT_WAS_REFUSED)
+                .or_insert(0) += 1;
+        }
+    }
+}
+
+/// 二次崩溃镜像上的挂载透明（r5 判决第二节 U6，用户定 F6「加二次」）：只读恢复读出的与 R1 比。挂载推了抬 F 而二次崩溃崩在暖机那一段
+/// （抬 F 那几串接在暖机之后）时，F 那一样不判（条款没写落在哪一处）。没崩过的那一次挂载被拒的不判。
+fn mount_transparency_violations_of_a_second_crash(
+    tally: &mut CrashInjectionTally,
+    fields_read_after_the_first_crash: &RecoveryFieldsRead,
+    fields_read_after_the_second_crash: &RecoveryFieldsRead,
+    floor_raised_by_the_uninterrupted_mount: Option<FloorRaisedByTheMount>,
+    phase: WritableMountPhase,
+) -> Vec<(&'static str, String)> {
+    let Some(floor) = floor_raised_by_the_uninterrupted_mount else {
+        *tally
+            .mount_transparency_not_judged
+            .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_UNINTERRUPTED_MOUNT_WAS_REFUSED)
+            .or_insert(0) += 1;
+        return Vec::new();
+    };
+    let floor_raises = match (floor, phase) {
+        (FloorRaisedByTheMount::RaisedTo(_), WritableMountPhase::WarmUp) => {
+            FloorRaisesAtTheSecondCrash::SomeMayHaveLanded
+        }
+        (FloorRaisedByTheMount::NotRaised, _)
+        | (
+            FloorRaisedByTheMount::RaisedTo(_),
+            WritableMountPhase::Acquisition | WritableMountPhase::RowPublish,
+        ) => FloorRaisesAtTheSecondCrash::NoneLandedYet,
+    };
+    let transparency = judge_mount_transparency(
+        fields_read_after_the_first_crash,
+        fields_read_after_the_second_crash,
+        MountTransparencyAction::SecondCrashInsideTheMount(floor_raises),
+    );
+    match &transparency {
+        MountTransparency::Judged(_) => {
+            *tally
+                .mount_transparency_judged_on_second_crashes_by_phase
+                .entry(phase)
+                .or_insert(0) += 1;
+        }
+        MountTransparency::NotJudged(_) => {}
+    }
+    violations_of_the_mount_transparency(tally, &transparency)
+}
+
 /// 挂载那一段录制流里每一次写落在挂载的哪一段：先是取号的系统配置槽写，第一个别的写起是写行那次发布，
 /// 它的根槽写与紧跟着的系统配置槽轮换之后的第一个别的写起是暖机。
 fn writable_mount_phase_of_each_write(writes: &[RetainedWrite]) -> Vec<WritableMountPhase> {
@@ -1740,6 +2178,10 @@ struct SecondCrashInputs<'history> {
     /// 没崩过的那一次可写挂载之后读出的内容（I-8.4 的「做一次」）；挂载被拒时是没判的理由。
     content_after_the_uninterrupted_mount:
         Result<ContentReadByRecovery, RemountIdempotenceNotJudged>,
+    /// 第一次崩溃镜像上的只读恢复与挂载透明要比的字段（R1）。
+    fields_read_after_the_first_crash: &'history RecoveryFieldsRead,
+    /// 没崩过的那一次挂载交回的推没推抬 F；挂载被拒时 None。
+    floor_raised_by_the_uninterrupted_mount: Option<FloorRaisedByTheMount>,
     /// 二次崩溃之后再挂用的建池参数与空间准入开关（与没崩过的那一次挂载同一份）。
     parameters: &'history MakeFilesystemParameters,
     space_admission: SpaceAdmission,
@@ -1766,6 +2208,8 @@ fn judge_second_crashes_inside_the_writable_mount(
         content_read_back_after_the_crash,
         newest_persisted_before_the_mount,
         content_after_the_uninterrupted_mount,
+        fields_read_after_the_first_crash,
+        floor_raised_by_the_uninterrupted_mount,
         parameters,
         space_admission,
     } = inputs;
@@ -1813,13 +2257,15 @@ fn judge_second_crashes_inside_the_writable_mount(
             .second_crash_points_by_phase
             .entry(second.phase)
             .or_insert(0) += 1;
-        let report = recover(&image, JournalPolicy::Consult);
+        let fields_read_after_the_second_crash =
+            read_recovery_fields(&image, ComparedVersion::TheRecoveredVersion);
+        let report = &fields_read_after_the_second_crash.report;
         match &report.outcome {
             RecoveryOutcome::FileRead { .. } => tally.second_crash_recoveries_reading_a_file += 1,
             RecoveryOutcome::NoFile { .. } => tally.second_crash_recoveries_without_a_file += 1,
             RecoveryOutcome::Failed { .. } => tally.second_crash_recoveries_failed += 1,
         }
-        let read_back_after_the_second_crash = observed_read_back_after_a_crash(&report);
+        let read_back_after_the_second_crash = observed_read_back_after_a_crash(report);
         let newest_persisted_by_the_mount =
             newest_persisted_root(mount_writes, &persisted_in_the_mount)
                 .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
@@ -1849,7 +2295,14 @@ fn judge_second_crashes_inside_the_writable_mount(
             image_directory,
         );
         tally.second_crash_checker_runs += 1;
-        let violations = checker_violations_of(&image);
+        let mut violations = checker_violations_of(&image);
+        violations.extend(mount_transparency_violations_of_a_second_crash(
+            tally,
+            fields_read_after_the_first_crash,
+            &fields_read_after_the_second_crash,
+            floor_raised_by_the_uninterrupted_mount,
+            second.phase,
+        ));
         if violations.is_empty() && disagreement.is_none() && record_check == RecordCheck::default()
         {
             continue;
@@ -2892,6 +3345,79 @@ mod tests {
         );
     }
 
+    /// 挂载透明在崩溃注入里真的判了（r5 判决第二节 U5、U6）：与上一条同一段历史、同一种摆法，每个挂载做成的崩溃状态都判了只挂载那一个，
+    /// 挂载与之后那次发布都做成的都判了挂载加一次发布那一个，二次崩溃取号、写行、暖机三段各有判过的；一样都没红。
+    #[test]
+    fn mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field(
+    ) {
+        let history = GeneratedHistory {
+            seed: HistorySeed(3),
+            starting_point: HistoryStartingPoint::AfterFirstFile,
+            operations: generate_history(HistorySeed(3), 10).operations,
+        };
+        let injection = inject_crashes_into_history(
+            &history,
+            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+            CrashPointDraw::EveryProperSubsetOfShortSegments {
+                segment_writes: 2,
+                sampled_in_longer_segments: 2,
+            },
+            None,
+        );
+        let tally = &injection.tally;
+        assert_eq!(
+            tally.recoveries_failed, 0,
+            "前提：崩溃镜像上的只读恢复都落到了一版：{tally:?}"
+        );
+        assert!(
+            tally.writable_mounts_after_the_crash_succeeded >= 1,
+            "前提：崩溃后镜像上的可写挂载做成过：{tally:?}"
+        );
+        assert_eq!(
+            tally
+                .mount_transparency_judged
+                .get(&MountTransparencyComparison::TheMountAlone)
+                .copied()
+                .unwrap_or(0),
+            tally.writable_mounts_after_the_crash_succeeded,
+            "每个挂载做成的崩溃状态都判了只挂载那一个：{:?}",
+            tally.mount_transparency_not_judged
+        );
+        assert_eq!(
+            tally
+                .mount_transparency_judged
+                .get(&MountTransparencyComparison::TheMountAndOnePublish)
+                .copied()
+                .unwrap_or(0),
+            tally.publishes_after_the_writable_mount,
+            "每个挂载与发布都做成的崩溃状态都判了挂载加一次发布那一个：{:?}",
+            tally.mount_transparency_not_judged
+        );
+        for phase in WritableMountPhase::ALL {
+            assert!(
+                tally
+                    .mount_transparency_judged_on_second_crashes_by_phase
+                    .get(&phase)
+                    .copied()
+                    .unwrap_or(0)
+                    >= 1,
+                "崩在挂载的 {} 里的二次崩溃镜像一次都没判过挂载透明：{:?}",
+                phase.name(),
+                tally.mount_transparency_judged_on_second_crashes_by_phase
+            );
+        }
+        assert!(
+            tally.mount_transparency_violations_by_field.is_empty(),
+            "挂载透明判红的字段 {:?}：{}",
+            tally.mount_transparency_violations_by_field,
+            injection
+                .new_findings
+                .iter()
+                .map(CrashPointFinding::render)
+                .collect::<String>()
+        );
+    }
+
     /// 重放行写出的两种摆法（只抽样、短段整段枚举）都经重放入口的解析原样读回，连同种子区间与取样参数；
     /// 默认值取的是另一组，读回的不是写出的那一组就红。
     #[test]
diff --git a/crates/singlefs-harness/src/lib.rs b/crates/singlefs-harness/src/lib.rs
index 10a3857..1c434ce 100644
--- a/crates/singlefs-harness/src/lib.rs
+++ b/crates/singlefs-harness/src/lib.rs
@@ -20,6 +20,7 @@ pub mod history;
 pub mod memory_pool;
 pub mod model;
 pub mod model_comparison;
+pub mod mount_transparency;
 pub mod new_pool_file_creation_regions;
 pub mod numeric_environment_variable;
 pub mod read_tally;
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
new file mode 100644
index 0000000..acce6ba
--- /dev/null
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -0,0 +1,1109 @@
+//! 挂载透明（独立的第四条，不是 M3 的第三读：`research/prompts/m3-formal-r4-main-verification.md` 第四节第 3 条；
+//! 逐字段表与二次崩溃、拒绝合法照 `research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条，零轮）的判法。
+//!
+//! 射程：不带故障、可写挂载做成的崩溃状态。崩溃镜像上只读恢复一次（R1），在它上面做一个动作——只挂载（取号、写行、暖机与挂载自己推的抬 F），
+//! 或挂载加一次发布——之后再只读恢复一次（R2），逐字段比（[`judge_mount_transparency`]）；挂载途中三个相位的二次崩溃镜像上，
+//! 只读恢复读出的与 R1 比（用户 2026-10-02 定 F6「加二次」）。可写挂载被拒的状态不判透明，判拒绝本身合不合法（[`judge_writable_mount_refusal`]）。
+//!
+//! 读盘的只有 [`read_recovery_fields`] 与 [`judge_writable_mount_refusal`] 里在崩溃镜像上复算拒的判据那一截；[`judge_mount_transparency`]
+//! 只吃读好的字段与挂载交回值。checker 档的崩溃注入调它，harness 档的轻用例也调它（harness 档不依赖 checker 档）。
+//! r5 判决标「推的」的几样（树 ID 水位、两个根指针、txg、扫描报告）照写进判法，每一样红的时候在交回值里单独标出它是推的还是量过的
+//! （[`MountTransparencyField::basis`]）。
+
+use std::collections::BTreeMap;
+
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::instance_table::InstanceRow;
+use singlefs_core::journal::JournalRecordPlaceInPublish;
+use singlefs_core::mount::{MountError, MountOutput, StillUnreadableAfterOneReread};
+use singlefs_core::recovery::{
+    choose_root, choose_system_configuration, effective_rollback_floor, highest_root_instance,
+    highest_root_txg, instance_table_of_root, readable_roots, recover, replay_journal,
+    scan_journal, user_visible_tree_root_pointers, verified_system_configuration_slots,
+    walk_to_file, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
+    UserVisibleTreeRootPointers,
+};
+use singlefs_core::root_record::{RootRecord, UnmountMarker};
+use singlefs_core::system_configuration::SystemImmutableConfiguration;
+use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
+
+/// 一次只读恢复读出的 POSIX 内容与存在性。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum PosixContentRead {
+    File(Vec<u8>),
+    NoFile,
+    /// 恢复或走读失败，什么都没读出来。
+    NothingRead,
+}
+
+impl PosixContentRead {
+    /// 一份恢复报告读出的内容（`recover` 的结局）。
+    #[must_use]
+    pub fn of_the_recovery_report(report: &RecoveryReport) -> Self {
+        match &report.outcome {
+            RecoveryOutcome::FileRead { content, .. } => PosixContentRead::File(content.clone()),
+            RecoveryOutcome::NoFile { .. } => PosixContentRead::NoFile,
+            RecoveryOutcome::Failed { .. } => PosixContentRead::NothingRead,
+        }
+    }
+
+    /// 给人看：几个字节、没有文件、没读出来（不把整份内容打出来）。
+    #[must_use]
+    pub fn render(&self) -> String {
+        match self {
+            PosixContentRead::File(content) => format!("文件 {} 字节", content.len()),
+            PosixContentRead::NoFile => "没有文件".to_string(),
+            PosixContentRead::NothingRead => "没读出内容".to_string(),
+        }
+    }
+}
+
+/// R2 拿哪一版来比：只挂载与二次崩溃比恢复落到的那一版；挂载加一次发布比挂载后那次发布之前那一版（r4 判决第二节 T5 那一行）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ComparedVersion {
+    TheRecoveredVersion,
+    TheVersionBeforeTheRecoveredOne,
+}
+
+/// 拿来比的那一版自己的字段：根记录、读出的内容、它指着的实例表的行、树表里两棵用户可见树的根指针。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct VersionFieldsRead {
+    pub root: RootRecord,
+    pub content: PosixContentRead,
+    /// 实例表沿链有一片读不出、解不开时 None。
+    pub instance_table_rows: Option<Vec<InstanceRow>>,
+    /// 树表读不出、解不开时 None。
+    pub user_visible_tree_root_pointers: Option<UserVisibleTreeRootPointers>,
+}
+
+/// 一个池上的一次只读恢复（看 journal）连同挂载透明要比的字段（[`read_recovery_fields`] 读出来的）。
+/// 字段没有不变量，直接公开：harness 的轻用例改坏其中一样，证判法会红。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RecoveryFieldsRead {
+    /// 那一次只读恢复的报告（`recover(reader, JournalPolicy::Consult)`）。
+    pub report: RecoveryReport,
+    pub compared: ComparedVersion,
+    /// 拿来比的那一版；恢复没落到一条根、或要比的是发布之前那一版而根环里没有比恢复落到的那一版更旧的根时 None。
+    pub compared_version: Option<VersionFieldsRead>,
+    /// 择到的系统配置的不可变配置（择不到时 None）。
+    pub immutable_configuration: Option<SystemImmutableConfiguration>,
+    /// F 生效值（D16（发布语义） 已定项 1「生效」，`recovery::effective_rollback_floor`）。
+    pub effective_rollback_floor: Option<CheckpointTxg>,
+    /// 每块盘两槽里全部自证过（校验和过、fsid 与本池相同）的系统配置的实例代号的最大值（取号的 max 那一半，D18（块里携带什么信息） 已定项 11）。
+    pub highest_system_configuration_instance: Option<InstanceGeneration>,
+    /// 根环里全部可读根的实例代号的最大值（取号的 max 另一半）。
+    pub highest_root_instance_in_the_ring: Option<InstanceGeneration>,
+    /// 根环里全部可读根的 checkpoint_txg 的最大值（新实例第一次发布的 txg 下界那一半，D23（journal 的角色与格式） 已定项 14 注 3）。
+    pub highest_root_txg_in_the_ring: Option<CheckpointTxg>,
+    /// 环里全部自证通过的 journal 记录的 checkpoint_txg 的最大值（txg 下界另一半）。
+    pub highest_record_checkpoint_txg_in_the_ring: Option<CheckpointTxg>,
+}
+
+/// 对 `reader` 跑一次只读恢复（看 journal），再读挂载透明要比的字段。`compared` 定拿哪一版来比（[`ComparedVersion`]）。
+///
+/// # Panics
+/// 同一份盘、同一套读法再择一次根、再重放一次，落到的那一版与恢复报告里的不同：两次读同一份内存盘，读法是同一组函数，走不到。
+#[must_use]
+pub fn read_recovery_fields(
+    reader: &dyn PoolReader,
+    compared: ComparedVersion,
+) -> RecoveryFieldsRead {
+    let report = recover(reader, JournalPolicy::Consult);
+    let mut fields = RecoveryFieldsRead {
+        report,
+        compared,
+        compared_version: None,
+        immutable_configuration: None,
+        effective_rollback_floor: None,
+        highest_system_configuration_instance: None,
+        highest_root_instance_in_the_ring: None,
+        highest_root_txg_in_the_ring: None,
+        highest_record_checkpoint_txg_in_the_ring: None,
+    };
+    let Ok(system_configuration) = choose_system_configuration(reader) else {
+        return fields;
+    };
+    let immutable = system_configuration.immutable;
+    let slot_spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
+    fields.immutable_configuration = Some(immutable);
+    fields.highest_system_configuration_instance = reader
+        .device_identities()
+        .into_iter()
+        .flat_map(|device| {
+            verified_system_configuration_slots(
+                reader,
+                device,
+                slot_spacing,
+                &immutable.filesystem_identifier,
+            )
+        })
+        .map(|slot| slot.quantities.journal_instance)
+        .max();
+    fields.effective_rollback_floor = Some(effective_rollback_floor(
+        reader,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    ));
+    fields.highest_root_instance_in_the_ring = highest_root_instance(
+        reader,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    );
+    fields.highest_root_txg_in_the_ring = highest_root_txg(
+        reader,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    );
+    let records = scan_journal(reader, &system_configuration);
+    fields.highest_record_checkpoint_txg_in_the_ring =
+        records.values().map(|record| record.checkpoint_txg).max();
+    let Some(chosen) = choose_root(reader, &system_configuration) else {
+        return fields;
+    };
+    let Ok((_, recovered_root)) = replay_journal(
+        reader,
+        &chosen,
+        immutable.sizes.journal_ring_bytes,
+        &records,
+        true,
+    ) else {
+        return fields;
+    };
+    assert_eq!(
+        Some((recovered_root.instance, recovered_root.checkpoint_txg)),
+        fields.report.effective_root,
+        "同一份盘上再择一次根、再重放一次，要落到恢复报告里那一版"
+    );
+    fields.compared_version = match compared {
+        ComparedVersion::TheRecoveredVersion => Some(version_fields(
+            reader,
+            recovered_root,
+            PosixContentRead::of_the_recovery_report(&fields.report),
+        )),
+        ComparedVersion::TheVersionBeforeTheRecoveredOne => readable_roots(
+            reader,
+            &immutable.region_devices,
+            &immutable.sizes,
+            &immutable.filesystem_identifier,
+        )
+        .into_iter()
+        .filter(|root| {
+            (root.checkpoint_txg, root.instance)
+                < (recovered_root.checkpoint_txg, recovered_root.instance)
+        })
+        .max_by_key(|root| (root.checkpoint_txg, root.instance))
+        .map(|before| {
+            let mut mapping_fallbacks = 0usize;
+            let content = match walk_to_file(reader, &before, &mut mapping_fallbacks) {
+                Ok(Some(content)) => PosixContentRead::File(content),
+                Ok(None) => PosixContentRead::NoFile,
+                Err(_failure) => PosixContentRead::NothingRead,
+            };
+            version_fields(reader, before, content)
+        }),
+    };
+    fields
+}
+
+fn version_fields(
+    reader: &dyn PoolReader,
+    root: RootRecord,
+    content: PosixContentRead,
+) -> VersionFieldsRead {
+    VersionFieldsRead {
+        root,
+        content,
+        instance_table_rows: instance_table_of_root(reader, &root).map(|table| table.rows),
+        user_visible_tree_root_pointers: user_visible_tree_root_pointers(reader, &root).ok(),
+    }
+}
+
+/// 挂载交回值里挂载透明要的那一样：挂载在写行与暖机之后推没推抬 F，推了的话最后一串抬到的 F（`MountOutput::space_admission`）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum FloorRaisedByTheMount {
+    NotRaised,
+    RaisedTo(CheckpointTxg),
+}
+
+impl FloorRaisedByTheMount {
+    /// # Panics
+    /// 推过的一串里一次空发布都没有：抬 F 那一串推空发布直到每块盘上都有带新 F 的根，至少一次（`mount::RaisedFloor::publishes`）。
+    #[must_use]
+    pub fn of_the_mount_output(output: &MountOutput) -> Self {
+        match output.space_admission.floor_raises().last() {
+            None => FloorRaisedByTheMount::NotRaised,
+            Some(last_raise) => FloorRaisedByTheMount::RaisedTo(
+                last_raise
+                    .publishes
+                    .last()
+                    .expect("抬 F 那一串至少推一次带新 F 的空发布（RaisedFloor::publishes 的契约）")
+                    .root
+                    .rollback_floor,
+            ),
+        }
+    }
+}
+
+/// 二次崩溃镜像上，挂载推的抬 F 那几串可能已经落了一部分没有：挂载没推，或崩在暖机之前（取号、写行），一步都还没落。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum FloorRaisesAtTheSecondCrash {
+    NoneLandedYet,
+    /// 挂载推了抬 F，二次崩溃崩在暖机那一段（抬 F 那几串接在暖机之后）：F 生效值落在 R1 的与抬到的之间哪一处，条款没写。
+    SomeMayHaveLanded,
+}
+
+/// 做的是哪个动作。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum MountTransparencyAction {
+    /// 只挂载，不接发布：R2 比它恢复落到的那一版（v2a）。
+    MountAlone(FloorRaisedByTheMount),
+    /// 挂载加一次发布：R2 比挂载后那次发布之前那一版。
+    MountAndOnePublish(FloorRaisedByTheMount),
+    /// 挂载途中再崩一次：R2 是二次崩溃镜像上的只读恢复，比的对象是 v1（r5 判决第二节 U6）。
+    SecondCrashInsideTheMount(FloorRaisesAtTheSecondCrash),
+}
+
+/// 比的十样（r5 判决第二节 U5 那一行的逐字段表）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum MountTransparencyField {
+    /// POSIX 内容与存在性。
+    PosixContent,
+    /// 系统配置的不可变配置（本盘设备号除外）。
+    ImmutableConfigurationWithoutThisDevice,
+    UnmountMarker,
+    /// F：根里的与生效值。
+    RollbackFloor,
+    InstanceGeneration,
+    InstanceTableRows,
+    TreeIdentifierWatermark,
+    /// 树表里 inode 树、extent 树两个根指针。
+    UserVisibleTreeRootPointers,
+    CheckpointTxg,
+    /// 扫描报告（施加了几条、水位之上几条）。
+    JournalScanReport,
+}
+
+/// 一样字段的关系是攻方在副本装置上量过（主 agent 复跑坐实）的，还是 r5 判决标「推的」。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum FieldBasis {
+    MeasuredByTheSecondRoundAttack,
+    InferredNotMeasured,
+}
+
+impl MountTransparencyField {
+    pub const ALL: [MountTransparencyField; 10] = [
+        MountTransparencyField::PosixContent,
+        MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
+        MountTransparencyField::UnmountMarker,
+        MountTransparencyField::RollbackFloor,
+        MountTransparencyField::InstanceGeneration,
+        MountTransparencyField::InstanceTableRows,
+        MountTransparencyField::TreeIdentifierWatermark,
+        MountTransparencyField::UserVisibleTreeRootPointers,
+        MountTransparencyField::CheckpointTxg,
+        MountTransparencyField::JournalScanReport,
+    ];
+
+    /// 这一样在这个动作下的关系是量过的还是推的。二次崩溃那一截的字段一半整截是推的（r5 判决第二节 U6、攻方 U6-1），
+    /// F 在挂载推了抬 F 时那一半（「= 挂载交回的最后一串抬到的 F」）r5 判决写的是「后半没量」。
+    #[must_use]
+    pub fn basis(self, action: MountTransparencyAction) -> FieldBasis {
+        match (action, self) {
+            (MountTransparencyAction::SecondCrashInsideTheMount(_), _)
+            | (
+                MountTransparencyAction::MountAlone(FloorRaisedByTheMount::RaisedTo(_))
+                | MountTransparencyAction::MountAndOnePublish(FloorRaisedByTheMount::RaisedTo(_)),
+                MountTransparencyField::RollbackFloor,
+            )
+            | (
+                MountTransparencyAction::MountAlone(_)
+                | MountTransparencyAction::MountAndOnePublish(_),
+                MountTransparencyField::TreeIdentifierWatermark
+                | MountTransparencyField::UserVisibleTreeRootPointers
+                | MountTransparencyField::CheckpointTxg
+                | MountTransparencyField::JournalScanReport,
+            ) => FieldBasis::InferredNotMeasured,
+            (
+                MountTransparencyAction::MountAlone(_)
+                | MountTransparencyAction::MountAndOnePublish(_),
+                MountTransparencyField::PosixContent
+                | MountTransparencyField::ImmutableConfigurationWithoutThisDevice
+                | MountTransparencyField::UnmountMarker
+                | MountTransparencyField::RollbackFloor
+                | MountTransparencyField::InstanceGeneration
+                | MountTransparencyField::InstanceTableRows,
+            ) => FieldBasis::MeasuredByTheSecondRoundAttack,
+        }
+    }
+
+    /// 判红时记进违例的名字（崩溃注入按它分签名）：量过的与推的各一个名字，提交时的快档一眼分得开。
+    #[must_use]
+    pub fn violation_name(self, basis: FieldBasis) -> &'static str {
+        match (self, basis) {
+            (MountTransparencyField::PosixContent, FieldBasis::MeasuredByTheSecondRoundAttack) => {
+                "挂载透明：POSIX 内容（量过）"
+            }
+            (MountTransparencyField::PosixContent, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：POSIX 内容（推的）"
+            }
+            (
+                MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：不可变配置（量过）",
+            (
+                MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
+                FieldBasis::InferredNotMeasured,
+            ) => "挂载透明：不可变配置（推的）",
+            (MountTransparencyField::UnmountMarker, FieldBasis::MeasuredByTheSecondRoundAttack) => {
+                "挂载透明：卸载记号（量过）"
+            }
+            (MountTransparencyField::UnmountMarker, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：卸载记号（推的）"
+            }
+            (MountTransparencyField::RollbackFloor, FieldBasis::MeasuredByTheSecondRoundAttack) => {
+                "挂载透明：F（量过）"
+            }
+            (MountTransparencyField::RollbackFloor, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：F（推的）"
+            }
+            (
+                MountTransparencyField::InstanceGeneration,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：实例代号（量过）",
+            (MountTransparencyField::InstanceGeneration, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：实例代号（推的）"
+            }
+            (
+                MountTransparencyField::InstanceTableRows,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：实例表行（量过）",
+            (MountTransparencyField::InstanceTableRows, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：实例表行（推的）"
+            }
+            (
+                MountTransparencyField::TreeIdentifierWatermark,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：树 ID 水位（量过）",
+            (MountTransparencyField::TreeIdentifierWatermark, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：树 ID 水位（推的）"
+            }
+            (
+                MountTransparencyField::UserVisibleTreeRootPointers,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：两棵树的根指针（量过）",
+            (
+                MountTransparencyField::UserVisibleTreeRootPointers,
+                FieldBasis::InferredNotMeasured,
+            ) => "挂载透明：两棵树的根指针（推的）",
+            (MountTransparencyField::CheckpointTxg, FieldBasis::MeasuredByTheSecondRoundAttack) => {
+                "挂载透明：txg（量过）"
+            }
+            (MountTransparencyField::CheckpointTxg, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：txg（推的）"
+            }
+            (
+                MountTransparencyField::JournalScanReport,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：扫描报告（量过）",
+            (MountTransparencyField::JournalScanReport, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：扫描报告（推的）"
+            }
+        }
+    }
+}
+
+/// 某一样为什么没判：条款没写这一格该是什么，不替它定红绿，只计数（交主 agent）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum FieldNotJudged {
+    /// 二次崩溃崩在暖机那一段、挂载推了抬 F：F 生效值落在哪一处条款没写。
+    FloorOfASecondCrashAfterTheFloorRaisesBegan,
+    /// 二次崩溃镜像上的恢复可以合法地由记录重建暖机那一版（施加条数不为 0）：扫描报告的式子只写到挂载做成之后。
+    JournalScanReportOfASecondCrashImage,
+}
+
+impl FieldNotJudged {
+    #[must_use]
+    pub fn name(self) -> &'static str {
+        match self {
+            FieldNotJudged::FloorOfASecondCrashAfterTheFloorRaisesBegan => {
+                "二次崩溃崩在抬 F 开始之后的 F"
+            }
+            FieldNotJudged::JournalScanReportOfASecondCrashImage => "二次崩溃镜像上的扫描报告",
+        }
+    }
+}
+
+/// 一样字段比下来。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum FieldOutcome {
+    Holds,
+    Violated { expected: String, observed: String },
+    NotJudged(FieldNotJudged),
+}
+
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct FieldVerdict {
+    pub field: MountTransparencyField,
+    pub basis: FieldBasis,
+    pub outcome: FieldOutcome,
+}
+
+/// 整个动作没判的理由。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum MountTransparencyNotJudged {
+    /// R1 没落到一条根、或落到了却没读出内容：射程只到挂载做成的状态，崩溃镜像本身那一截已经把它判红了。
+    NothingWasRecoveredFromTheCrashImage,
+}
+
+impl MountTransparencyNotJudged {
+    #[must_use]
+    pub fn name(self) -> &'static str {
+        match self {
+            MountTransparencyNotJudged::NothingWasRecoveredFromTheCrashImage => {
+                "崩溃镜像上的只读恢复没落到一版"
+            }
+        }
+    }
+}
+
+/// 一个动作的挂载透明判下来。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum MountTransparency {
+    /// 逐样判过，按 [`MountTransparencyField::ALL`] 的次序（R2 没有可比的那一版时只有 POSIX 内容一样，判红）。
+    Judged(Vec<FieldVerdict>),
+    NotJudged(MountTransparencyNotJudged),
+}
+
+impl MountTransparency {
+    /// 判红的那几样。
+    #[must_use]
+    pub fn violated_fields(&self) -> Vec<&FieldVerdict> {
+        match self {
+            MountTransparency::Judged(verdicts) => verdicts
+                .iter()
+                .filter(|verdict| match verdict.outcome {
+                    FieldOutcome::Violated { .. } => true,
+                    FieldOutcome::Holds | FieldOutcome::NotJudged(_) => false,
+                })
+                .collect(),
+            MountTransparency::NotJudged(_) => Vec::new(),
+        }
+    }
+
+    /// 这一样判下来是什么（没判到这一样时 None）。
+    #[must_use]
+    pub fn outcome_of(&self, field: MountTransparencyField) -> Option<&FieldOutcome> {
+        match self {
+            MountTransparency::Judged(verdicts) => verdicts
+                .iter()
+                .find(|verdict| verdict.field == field)
+                .map(|verdict| &verdict.outcome),
+            MountTransparency::NotJudged(_) => None,
+        }
+    }
+}
+
+/// R2 那一版与 R1 是哪一种关系：动作做成之后的那一版（实例是新取的号），还是二次崩溃之后落回 v1 那一版自己。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum Relation {
+    WrittenByTheMount(FloorRelation),
+    TheVersionOfTheFirstRecovery(FloorRelation),
+}
+
+/// F 那一样按哪一句判。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum FloorRelation {
+    /// 挂载没推抬 F（或二次崩溃崩在抬 F 落盘之前）：生效值不变、R2 根里的 F = R1 的生效值。
+    Unchanged,
+    /// 挂载推了：R2 的 F 生效值 = 最后一串抬到的 F。
+    RaisedTo(CheckpointTxg),
+    NotJudged,
+}
+
+fn held_or_violated(holds: bool, expected: String, observed: String) -> FieldOutcome {
+    if holds {
+        FieldOutcome::Holds
+    } else {
+        FieldOutcome::Violated { expected, observed }
+    }
+}
+
+/// 不可变配置除本盘设备号之外逐项相同（本盘设备号是择到哪块盘的那一槽，不是池级字段）。按字段拆开写全：新加一个字段而这里没比，编译不过。
+fn immutable_configuration_is_the_same_except_this_device(
+    left: &SystemImmutableConfiguration,
+    right: &SystemImmutableConfiguration,
+) -> bool {
+    let SystemImmutableConfiguration {
+        filesystem_identifier: left_filesystem_identifier,
+        this_device: _,
+        device_count: left_device_count,
+        region_devices: left_region_devices,
+        sizes: left_sizes,
+        journal_ring_start_slot: left_journal_ring_start_slot,
+        root_ring_base_slot: left_root_ring_base_slot,
+    } = left;
+    let SystemImmutableConfiguration {
+        filesystem_identifier: right_filesystem_identifier,
+        this_device: _,
+        device_count: right_device_count,
+        region_devices: right_region_devices,
+        sizes: right_sizes,
+        journal_ring_start_slot: right_journal_ring_start_slot,
+        root_ring_base_slot: right_root_ring_base_slot,
+    } = right;
+    (
+        left_filesystem_identifier,
+        left_device_count,
+        left_region_devices,
+        left_sizes,
+        left_journal_ring_start_slot,
+        left_root_ring_base_slot,
+    ) == (
+        right_filesystem_identifier,
+        right_device_count,
+        right_region_devices,
+        right_sizes,
+        right_journal_ring_start_slot,
+        right_root_ring_base_slot,
+    )
+}
+
+/// D18（块里携带什么信息） 已定项 11「恢复写的行」：恢复在所选根指着的那一版表上给 [max(所选根的实例, 1), 新实例) 每个实例写一行，
+/// 所选根那个实例写 (i, 重放之后那个根的 checkpoint_txg, 这次重放施加的属于它的最大事务号)，严格介于中间的写 (i, 0, 0)；后写覆盖。
+/// 重放不跨实例边界（D23（journal 的角色与格式） 已定项 14 注 1），所以所选根的实例就是 R1 那一版的实例。交回按实例排好的行；
+/// R1 那一版的表读不出时 None。
+fn instance_table_rows_written_by_the_recovery(
+    first_recovery: &VersionFieldsRead,
+    maximum_applied_transaction: u64,
+    new_instance: InstanceGeneration,
+) -> Option<Vec<InstanceRow>> {
+    let mut rows: BTreeMap<InstanceGeneration, InstanceRow> = first_recovery
+        .instance_table_rows
+        .as_ref()?
+        .iter()
+        .map(|row| (row.instance, *row))
+        .collect();
+    let chosen_instance = first_recovery.root.instance;
+    for instance in chosen_instance.0.max(1)..new_instance.0 {
+        let instance = InstanceGeneration(instance);
+        let row = if instance == chosen_instance {
+            InstanceRow {
+                instance,
+                selected_root_txg: first_recovery.root.checkpoint_txg,
+                applied_transaction_high_water: maximum_applied_transaction,
+            }
+        } else {
+            InstanceRow {
+                instance,
+                selected_root_txg: CheckpointTxg(0),
+                applied_transaction_high_water: 0,
+            }
+        };
+        rows.insert(instance, row);
+    }
+    Some(rows.into_values().collect())
+}
+
+fn rows_sorted_by_instance(rows: &[InstanceRow]) -> Vec<InstanceRow> {
+    let mut sorted = rows.to_vec();
+    sorted.sort_by_key(|row| row.instance);
+    sorted
+}
+
+/// 挂载透明的逐字段判（r5 判决第二节 U5、U6 两行）。`before_the_mount` 是崩溃镜像上的只读恢复（R1，比恢复落到的那一版），
+/// `after_the_action` 是动作之后的只读恢复（R2：只挂载与二次崩溃比恢复落到的那一版，挂载加一次发布比发布之前那一版）。
+///
+/// 动作做成之后的那一版：POSIX 内容与存在性相同；不可变配置（除本盘设备号）相同；卸载记号恒为「不是卸载写的」、R1 的记号不进比较；
+/// 挂载没推抬 F 时 R2 的 F 生效值与 R2 根里的 F 都 = R1 的 F 生效值，推了时 R2 的 F 生效值 = 挂载交回的最后一串抬到的 F；
+/// 实例代号 = max(R1 镜像上各盘两槽自证系统配置的实例代号, 根环全部可读根的实例代号) + 1；实例表行照 D18 已定项 11「恢复写的行」；
+/// 树 ID 水位相同（挂载不发第一个文件）；两棵用户可见树的根指针相同；txg ≥ max(根环全部根的 txg, 环里全部自证记录的 checkpoint_txg) + 1
+/// （只判下界：kb 允许跳号）；扫描报告施加 0 条、水位之上 0 条。
+/// 二次崩溃之后落回 v1 那一版自己（实例与 txg 都是 R1 那一版的）：各样与 R1 那一版相同（推的）；落到这次挂载写的某一版：照上一段，
+/// 扫描报告不判，崩在抬 F 开始之后的 F 不判。
+///
+/// # Panics
+/// 调用方交错了比的版本：R1 不是比恢复落到的那一版，或 R2 拿的那一版与动作对不上。
+#[must_use]
+pub fn judge_mount_transparency(
+    before_the_mount: &RecoveryFieldsRead,
+    after_the_action: &RecoveryFieldsRead,
+    action: MountTransparencyAction,
+) -> MountTransparency {
+    assert_eq!(
+        before_the_mount.compared,
+        ComparedVersion::TheRecoveredVersion,
+        "R1 比的是崩溃镜像上恢复落到的那一版"
+    );
+    let compared_for_the_action = match action {
+        MountTransparencyAction::MountAlone(_)
+        | MountTransparencyAction::SecondCrashInsideTheMount(_) => {
+            ComparedVersion::TheRecoveredVersion
+        }
+        MountTransparencyAction::MountAndOnePublish(_) => {
+            ComparedVersion::TheVersionBeforeTheRecoveredOne
+        }
+    };
+    assert_eq!(
+        after_the_action.compared, compared_for_the_action,
+        "R2 拿来比的那一版要与动作对上：挂载加一次发布比发布之前那一版，别的比恢复落到的那一版"
+    );
+    let Some(first_recovery) = &before_the_mount.compared_version else {
+        return MountTransparency::NotJudged(
+            MountTransparencyNotJudged::NothingWasRecoveredFromTheCrashImage,
+        );
+    };
+    if first_recovery.content == PosixContentRead::NothingRead {
+        return MountTransparency::NotJudged(
+            MountTransparencyNotJudged::NothingWasRecoveredFromTheCrashImage,
+        );
+    }
+    let Some(compared) = &after_the_action.compared_version else {
+        return MountTransparency::Judged(vec![FieldVerdict {
+            field: MountTransparencyField::PosixContent,
+            basis: MountTransparencyField::PosixContent.basis(action),
+            outcome: FieldOutcome::Violated {
+                expected: first_recovery.content.render(),
+                observed: format!(
+                    "动作之后的只读恢复没有可比的那一版（{:?}）",
+                    after_the_action.report.outcome
+                ),
+            },
+        }]);
+    };
+    let relation = match action {
+        MountTransparencyAction::MountAlone(floor)
+        | MountTransparencyAction::MountAndOnePublish(floor) => {
+            Relation::WrittenByTheMount(match floor {
+                FloorRaisedByTheMount::NotRaised => FloorRelation::Unchanged,
+                FloorRaisedByTheMount::RaisedTo(raised) => FloorRelation::RaisedTo(raised),
+            })
+        }
+        MountTransparencyAction::SecondCrashInsideTheMount(floor_raises) => {
+            let floor = match floor_raises {
+                FloorRaisesAtTheSecondCrash::NoneLandedYet => FloorRelation::Unchanged,
+                FloorRaisesAtTheSecondCrash::SomeMayHaveLanded => FloorRelation::NotJudged,
+            };
+            if (compared.root.instance, compared.root.checkpoint_txg)
+                == (
+                    first_recovery.root.instance,
+                    first_recovery.root.checkpoint_txg,
+                )
+            {
+                Relation::TheVersionOfTheFirstRecovery(floor)
+            } else {
+                Relation::WrittenByTheMount(floor)
+            }
+        }
+    };
+    let new_instance = before_the_mount
+        .highest_system_configuration_instance
+        .max(before_the_mount.highest_root_instance_in_the_ring)
+        .and_then(|highest| highest.0.checked_add(1).map(InstanceGeneration));
+    let verdicts = MountTransparencyField::ALL
+        .into_iter()
+        .map(|field| FieldVerdict {
+            field,
+            basis: field.basis(action),
+            outcome: judge_one_field(
+                field,
+                before_the_mount,
+                first_recovery,
+                after_the_action,
+                compared,
+                relation,
+                new_instance,
+                action,
+            ),
+        })
+        .collect();
+    MountTransparency::Judged(verdicts)
+}
+
+#[allow(
+    clippy::too_many_arguments,
+    reason = "每个参数各是一样东西：判哪一样、两次恢复与各自比的那一版、两版的关系、R1 推出的新实例、动作"
+)]
+fn judge_one_field(
+    field: MountTransparencyField,
+    before_the_mount: &RecoveryFieldsRead,
+    first_recovery: &VersionFieldsRead,
+    after_the_action: &RecoveryFieldsRead,
+    compared: &VersionFieldsRead,
+    relation: Relation,
+    new_instance: Option<InstanceGeneration>,
+    action: MountTransparencyAction,
+) -> FieldOutcome {
+    match field {
+        MountTransparencyField::PosixContent => held_or_violated(
+            compared.content == first_recovery.content,
+            first_recovery.content.render(),
+            compared.content.render(),
+        ),
+        MountTransparencyField::ImmutableConfigurationWithoutThisDevice => {
+            match (
+                &before_the_mount.immutable_configuration,
+                &after_the_action.immutable_configuration,
+            ) {
+                (Some(before), Some(after)) => held_or_violated(
+                    immutable_configuration_is_the_same_except_this_device(before, after),
+                    format!("{before:?}"),
+                    format!("{after:?}"),
+                ),
+                (before, after) => FieldOutcome::Violated {
+                    expected: format!("{before:?}"),
+                    observed: format!("{after:?}"),
+                },
+            }
+        }
+        MountTransparencyField::UnmountMarker => match relation {
+            Relation::WrittenByTheMount(_) => held_or_violated(
+                compared.root.unmount_marker == UnmountMarker::NotWrittenByTheUnmountSequence,
+                format!("{:?}", UnmountMarker::NotWrittenByTheUnmountSequence),
+                format!("{:?}", compared.root.unmount_marker),
+            ),
+            Relation::TheVersionOfTheFirstRecovery(_) => held_or_violated(
+                compared.root.unmount_marker == first_recovery.root.unmount_marker,
+                format!("{:?}", first_recovery.root.unmount_marker),
+                format!("{:?}", compared.root.unmount_marker),
+            ),
+        },
+        MountTransparencyField::RollbackFloor => {
+            let effective_floor_before = before_the_mount.effective_rollback_floor;
+            // 挂载写的那一版根里的 F 照 R1 的生效值写；落回 v1 那一版自己时根就是 v1 的根。
+            let (floor, expected_floor_in_the_root) = match relation {
+                Relation::WrittenByTheMount(floor) => (floor, effective_floor_before),
+                Relation::TheVersionOfTheFirstRecovery(floor) => {
+                    (floor, Some(first_recovery.root.rollback_floor))
+                }
+            };
+            match floor {
+                FloorRelation::NotJudged => {
+                    FieldOutcome::NotJudged(FieldNotJudged::FloorOfASecondCrashAfterTheFloorRaisesBegan)
+                }
+                FloorRelation::Unchanged => held_or_violated(
+                    after_the_action.effective_rollback_floor == effective_floor_before
+                        && Some(compared.root.rollback_floor) == expected_floor_in_the_root,
+                    format!(
+                        "F 生效值是 R1 的 {effective_floor_before:?}，R2 根里的 F 是 {expected_floor_in_the_root:?}"
+                    ),
+                    format!(
+                        "R2 的 F 生效值 {:?}、R2 根里的 F {:?}",
+                        after_the_action.effective_rollback_floor, compared.root.rollback_floor
+                    ),
+                ),
+                FloorRelation::RaisedTo(raised) => held_or_violated(
+                    after_the_action.effective_rollback_floor == Some(raised),
+                    format!("F 生效值是挂载交回的最后一串抬到的 F {raised:?}"),
+                    format!("R2 的 F 生效值 {:?}", after_the_action.effective_rollback_floor),
+                ),
+            }
+        }
+        MountTransparencyField::InstanceGeneration => match relation {
+            Relation::WrittenByTheMount(_) => held_or_violated(
+                Some(compared.root.instance) == new_instance,
+                format!(
+                    "max(系统配置 {:?}, 根环 {:?}) + 1 = {new_instance:?}",
+                    before_the_mount.highest_system_configuration_instance,
+                    before_the_mount.highest_root_instance_in_the_ring
+                ),
+                format!("{:?}", compared.root.instance),
+            ),
+            Relation::TheVersionOfTheFirstRecovery(_) => held_or_violated(
+                compared.root.instance == first_recovery.root.instance,
+                format!("{:?}", first_recovery.root.instance),
+                format!("{:?}", compared.root.instance),
+            ),
+        },
+        MountTransparencyField::InstanceTableRows => {
+            let expected = match relation {
+                Relation::WrittenByTheMount(_) => new_instance.and_then(|new_instance| {
+                    instance_table_rows_written_by_the_recovery(
+                        first_recovery,
+                        before_the_mount.report.journal.maximum_applied_transaction,
+                        new_instance,
+                    )
+                }),
+                Relation::TheVersionOfTheFirstRecovery(_) => first_recovery
+                    .instance_table_rows
+                    .as_deref()
+                    .map(rows_sorted_by_instance),
+            };
+            let observed = compared
+                .instance_table_rows
+                .as_deref()
+                .map(rows_sorted_by_instance);
+            held_or_violated(
+                expected.is_some() && expected == observed,
+                format!("{expected:?}"),
+                format!("{observed:?}"),
+            )
+        }
+        MountTransparencyField::TreeIdentifierWatermark => held_or_violated(
+            compared.root.tree_identifier_watermark
+                == first_recovery.root.tree_identifier_watermark,
+            first_recovery.root.tree_identifier_watermark.to_string(),
+            compared.root.tree_identifier_watermark.to_string(),
+        ),
+        MountTransparencyField::UserVisibleTreeRootPointers => held_or_violated(
+            first_recovery.user_visible_tree_root_pointers.is_some()
+                && compared.user_visible_tree_root_pointers
+                    == first_recovery.user_visible_tree_root_pointers,
+            format!("{:?}", first_recovery.user_visible_tree_root_pointers),
+            format!("{:?}", compared.user_visible_tree_root_pointers),
+        ),
+        MountTransparencyField::CheckpointTxg => match relation {
+            Relation::WrittenByTheMount(_) => {
+                let lower_bound = before_the_mount
+                    .highest_root_txg_in_the_ring
+                    .max(before_the_mount.highest_record_checkpoint_txg_in_the_ring)
+                    .and_then(|highest| highest.0.checked_add(1).map(CheckpointTxg));
+                held_or_violated(
+                    lower_bound
+                        .is_some_and(|lower_bound| compared.root.checkpoint_txg >= lower_bound),
+                    format!(
+                        "≥ max(根环 {:?}, 记录 {:?}) + 1",
+                        before_the_mount.highest_root_txg_in_the_ring,
+                        before_the_mount.highest_record_checkpoint_txg_in_the_ring
+                    ),
+                    format!("{:?}", compared.root.checkpoint_txg),
+                )
+            }
+            Relation::TheVersionOfTheFirstRecovery(_) => held_or_violated(
+                compared.root.checkpoint_txg == first_recovery.root.checkpoint_txg,
+                format!("{:?}", first_recovery.root.checkpoint_txg),
+                format!("{:?}", compared.root.checkpoint_txg),
+            ),
+        },
+        MountTransparencyField::JournalScanReport => match action {
+            MountTransparencyAction::SecondCrashInsideTheMount(_) => {
+                FieldOutcome::NotJudged(FieldNotJudged::JournalScanReportOfASecondCrashImage)
+            }
+            MountTransparencyAction::MountAlone(_)
+            | MountTransparencyAction::MountAndOnePublish(_) => {
+                let journal = after_the_action.report.journal;
+                held_or_violated(
+                    journal.prefix_applied == 0 && journal.above_water == 0,
+                    "施加 0 条、水位之上 0 条".to_string(),
+                    format!(
+                        "施加 {} 条、水位之上 {} 条",
+                        journal.prefix_applied, journal.above_water
+                    ),
+                )
+            }
+        },
+    }
+}
+
+/// D23（journal 的角色与格式） 已定项 14 那张单子（C554 乙-配置续，用户 2026-09-27 定；C331 取甲，用户 2026-09-28 定）上的拒可写理由。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum RefusalReasonOnTheD23List {
+    /// 读阶段判据 N-配置 为真（有一槽读不出或本池自证不过按真；取号那一刻有一槽读不出也报成它），重读一次仍为真。
+    NewerPublishWitnessedBySystemConfiguration,
+    /// 重建分配器判抛弃用的最新那条根的实例表读不出，重读一次仍读不出。
+    InstanceTableOfTheNewestRootUnreadable,
+}
+
+/// 拒可写的理由在不在 D23 已定项 14 那张单子上。别的理由（单元区墙、坏盘上才有的那几种……）照崩溃注入第二截今天的写法判。
+#[must_use]
+pub fn refusal_reason_on_the_d23_list(error: &MountError) -> Option<RefusalReasonOnTheD23List> {
+    match error {
+        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) => {
+            match **still_unreadable {
+                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. } => {
+                    Some(RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration)
+                }
+                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. } => {
+                    Some(RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable)
+                }
+                // 挂着时抬 F、管理员回退那几处读坏，与影子账读一条被抛弃根的账读不出（C393）：不在 D23 已定项 14 那张单子上。
+                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
+                | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
+                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => None,
+            }
+        }
+        MountError::Recovery(_)
+        | MountError::FileVersionWithoutAnyJournalRecord
+        | MountError::InstanceTableMalformed
+        | MountError::Acquisition(_)
+        | MountError::Publish(_)
+        | MountError::RaiseFloorSequencePublishFailed(_)
+        | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
+        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
+        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
+        | MountError::RollbackFloorAboveCeiling { .. }
+        | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
+        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
+        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
+        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
+        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
+        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
+        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
+        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
+        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
+        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
+        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
+        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
+        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
+        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
+        | MountError::SequenceNumberPastTheTopOfItsRange(_)
+        | MountError::FloorRaiseFailedAfterTheMountsPublishes(_) => None,
+    }
+}
+
+/// 判据 N-配置（D23 已定项 14「可写挂载读到的更新状态读不出」那一段）在崩溃镜像上自己复算一遍，不经实现的 `mount::newer_publish_witness`：
+/// 每块盘两槽有一槽读不出或本池自证不过 ⇒ 判不出、按真；全部槽的 journal tail 取最大为 c_见证，0 ⇒ 假；所选那一版那次发布带末条标志的记录读得出 ⇒
+/// c_见证 > 它的计数器；读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, checkpoint_txg) > 所选那一版；两条都读不出按真。
+/// `selected_version` 是 R1 落到的那一版（与可写挂载读阶段同一份盘、同一套择根与重放）；没有时判据无从复算，交回假。
+fn newer_publish_witnessed_on_the_crash_image(
+    crash_image: &dyn PoolReader,
+    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
+) -> bool {
+    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
+        return true;
+    };
+    let immutable = system_configuration.immutable;
+    let slot_spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
+    let mut witnessed_journal_counter = 0u64;
+    for device in crash_image.device_identities() {
+        let slots = verified_system_configuration_slots(
+            crash_image,
+            device,
+            slot_spacing,
+            &immutable.filesystem_identifier,
+        );
+        if u64::try_from(slots.len()).expect("一块盘的系统配置槽数装得进 u64")
+            < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
+        {
+            return true;
+        }
+        witnessed_journal_counter = slots
+            .iter()
+            .map(|slot| slot.quantities.journal_tail)
+            .fold(witnessed_journal_counter, u64::max);
+    }
+    if witnessed_journal_counter == 0 {
+        return false;
+    }
+    let Some(selected_version) = selected_version else {
+        return false;
+    };
+    let records = scan_journal(crash_image, &system_configuration);
+    let last_record_counter_of_the_selected_version = records
+        .values()
+        .filter(|record| {
+            (record.instance, record.checkpoint_txg) == selected_version
+                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
+        })
+        .map(|record| record.counter)
+        .max();
+    if let Some(last_record_counter) = last_record_counter_of_the_selected_version {
+        return witnessed_journal_counter > last_record_counter;
+    }
+    let record_at_the_witnessed_counter = records
+        .values()
+        .filter(|record| record.counter == witnessed_journal_counter)
+        .map(|record| (record.instance, record.checkpoint_txg))
+        .max();
+    match record_at_the_witnessed_counter {
+        Some(record) => record > selected_version,
+        None => true,
+    }
+}
+
+/// 「重建分配器判抛弃用的最新那条根的实例表读不出」在崩溃镜像上复算：根环里择到的最新那条根指着的实例表沿链读不出、解不开（择不到根也算）。
+fn instance_table_of_the_newest_root_unreadable_on_the_crash_image(
+    crash_image: &dyn PoolReader,
+) -> bool {
+    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
+        return true;
+    };
+    choose_root(crash_image, &system_configuration)
+        .is_none_or(|newest| instance_table_of_root(crash_image, &newest).is_none())
+}
+
+/// 拒之后盘上逐字节同不同（调用方比拒之前与拒之后的整份镜像）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DiskAfterTheRefusal {
+    ByteForByteUnchanged,
+    Changed,
+}
+
+/// 崩溃镜像上只读恢复读出的，在不在允许集合里（调用方拿理想模型判的那一截的结论交进来）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ReadOnlyReadBackOfTheCrashImage {
+    InsideTheAllowedSet,
+    OutsideTheAllowedSet,
+}
+
+/// 单子上的理由，合法还差哪一条。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum RefusalConditionNotMet {
+    /// 那一条判据在崩溃镜像上复算为假：拒的理由是错的。
+    CriterionFalseWhenRecomputedOnTheCrashImage,
+    DiskChangedByTheRefusedMount,
+    ReadOnlyReadBackOutsideTheAllowedSet,
+}
+
+/// 可写挂载被拒，拒得合不合法（r5 判决第二节 U6：拒绝合法 ⇔ 理由在 D23 已定项 14 那张单子上、那一条判据在崩溃镜像上复算为真、
+/// 盘上逐字节不变、只读读出在允许集合里）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum WritableMountRefusalVerdict {
+    Legal(RefusalReasonOnTheD23List),
+    Illegal {
+        reason: RefusalReasonOnTheD23List,
+        conditions_not_met: Vec<RefusalConditionNotMet>,
+    },
+    /// 理由不在单子上：照崩溃注入第二截今天的写法判（单元区墙计数，别的报红）。
+    ReasonNotOnTheD23List,
+}
+
+/// 判一次拒可写合不合法。拒的判据在崩溃镜像（`crash_image`）上自己复算（不经 O1 判官，也不经实现判拒的那一段）；
+/// `selected_version` 是崩溃镜像上只读恢复落到的那一版（R1 恢复报告里的 `effective_root`）。
+#[must_use]
+pub fn judge_writable_mount_refusal(
+    crash_image: &dyn PoolReader,
+    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
+    refusal: &MountError,
+    disk: DiskAfterTheRefusal,
+    read_only_read_back: ReadOnlyReadBackOfTheCrashImage,
+) -> WritableMountRefusalVerdict {
+    let Some(reason) = refusal_reason_on_the_d23_list(refusal) else {
+        return WritableMountRefusalVerdict::ReasonNotOnTheD23List;
+    };
+    let criterion_holds_on_the_crash_image = match reason {
+        RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration => {
+            newer_publish_witnessed_on_the_crash_image(crash_image, selected_version)
+        }
+        RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable => {
+            instance_table_of_the_newest_root_unreadable_on_the_crash_image(crash_image)
+        }
+    };
+    let mut conditions_not_met = Vec::new();
+    if !criterion_holds_on_the_crash_image {
+        conditions_not_met
+            .push(RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage);
+    }
+    match disk {
+        DiskAfterTheRefusal::ByteForByteUnchanged => {}
+        DiskAfterTheRefusal::Changed => {
+            conditions_not_met.push(RefusalConditionNotMet::DiskChangedByTheRefusedMount);
+        }
+    }
+    match read_only_read_back {
+        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet => {}
+        ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet => {
+            conditions_not_met.push(RefusalConditionNotMet::ReadOnlyReadBackOutsideTheAllowedSet);
+        }
+    }
+    if conditions_not_met.is_empty() {
+        WritableMountRefusalVerdict::Legal(reason)
+    } else {
+        WritableMountRefusalVerdict::Illegal {
+            reason,
+            conditions_not_met,
+        }
+    }
+}
diff --git a/crates/singlefs-harness/tests/crash_recovery_judge_refuses_a_root_older_than_the_newest_persisted_one.rs b/crates/singlefs-harness/tests/crash_recovery_judge_refuses_a_root_older_than_the_newest_persisted_one.rs
new file mode 100644
index 0000000..c90beb6
--- /dev/null
+++ b/crates/singlefs-harness/tests/crash_recovery_judge_refuses_a_root_older_than_the_newest_persisted_one.rs
@@ -0,0 +1,63 @@
+//! O1 判官（`singlefs_harness::model::crash_recovery_disagreement`）判据 ③：盘上已持久的最新根槽是 (T, i) 时，择到的根按 (txg, 实例) 不许比它旧
+//! （D22（单元原子性怎么合成） 已定项 7 的择新序：txg 为主、平局按实例代号高者赢）。两版都是模型提交过的、内容也逐字节对得上，
+//! 判据 ②④ 都放行，只有 ③ 判得出。
+
+use std::collections::BTreeMap;
+use std::rc::Rc;
+
+use singlefs_harness::model::{
+    crash_recovery_disagreement, ModelCheckpointTxg, ModelInstanceGeneration, ModelRootKey,
+    ObservedReadBack,
+};
+
+fn root(instance: u64, checkpoint_txg: u64) -> ModelRootKey {
+    ModelRootKey {
+        checkpoint_txg: ModelCheckpointTxg(checkpoint_txg),
+        instance: ModelInstanceGeneration(instance),
+    }
+}
+
+fn committed(versions: &[(ModelRootKey, &[u8])]) -> BTreeMap<ModelRootKey, Option<Rc<[u8]>>> {
+    versions
+        .iter()
+        .map(|(key, content)| (*key, Some(Rc::from(*content))))
+        .collect()
+}
+
+#[test]
+fn landing_on_an_older_txg_than_the_newest_persisted_root_is_red() {
+    let older = root(1, 3);
+    let newest = root(1, 4);
+    let versions = committed(&[(older, b"older"), (newest, b"newest")]);
+    let read_back = ObservedReadBack::FileRead {
+        root: older,
+        content: b"older".to_vec(),
+    };
+    assert!(
+        crash_recovery_disagreement(&versions, None, &read_back).is_none(),
+        "前提：不给盘上最新根槽时，落到提交过的旧版、内容对得上，判绿"
+    );
+    assert!(
+        crash_recovery_disagreement(&versions, Some(newest), &read_back).is_some(),
+        "盘上已持久的最新根槽是实例 1 第 4 代，落到第 3 代要判红"
+    );
+}
+
+#[test]
+fn landing_on_the_lower_instance_of_the_same_txg_as_the_newest_persisted_root_is_red() {
+    let lower_instance = root(1, 5);
+    let newest = root(2, 5);
+    let versions = committed(&[(lower_instance, b"instance one"), (newest, b"instance two")]);
+    let read_back = ObservedReadBack::FileRead {
+        root: lower_instance,
+        content: b"instance one".to_vec(),
+    };
+    assert!(
+        crash_recovery_disagreement(&versions, None, &read_back).is_none(),
+        "前提：不给盘上最新根槽时，落到提交过的那一版、内容对得上，判绿"
+    );
+    assert!(
+        crash_recovery_disagreement(&versions, Some(newest), &read_back).is_some(),
+        "同一个 txg 上实例 2 的根已持久，落到实例 1 的那一条要判红（平局按实例代号高者赢）"
+    );
+}
diff --git a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
new file mode 100644
index 0000000..222997e
--- /dev/null
+++ b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
@@ -0,0 +1,664 @@
+//! 挂载透明的逐字段判（`singlefs_harness::mount_transparency`，r5 判决第二节 U5、U6 两行）在几个小池上：每一样一条阳性（合法挂载判绿）、
+//! 一条阴性（把 R2 的那一样改坏判红，别的仍绿）。小池是单元区 384 槽（抬 F 那一条取 256 槽）的两块内存盘，历史由随机历史的执行器照抄
+//! 一段写死的操作跑出来；「崩溃镜像」取那段历史停下时的整份盘面（或一次发布扣下根槽写之后的盘面），它就是一个不带故障的崩溃状态。
+
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::make_filesystem::MakeFilesystemParameters;
+use singlefs_core::mount::{mount_writable_with_space_admission, unmount, Mounted, Unmounted};
+use singlefs_core::mounted_session::MountedSession;
+use singlefs_core::root_record::UnmountMarker;
+use singlefs_core::transaction::{
+    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
+};
+use singlefs_harness::history::{
+    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
+    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
+};
+use singlefs_harness::memory_pool::{
+    writes_and_segments_with_stream_indexes, MemoryPool, SparseBlockDevice,
+};
+use singlefs_harness::mount_transparency::{
+    judge_mount_transparency, read_recovery_fields, ComparedVersion, FieldNotJudged, FieldOutcome,
+    FloorRaisedByTheMount, FloorRaisesAtTheSecondCrash, MountTransparency, MountTransparencyAction,
+    MountTransparencyField, PosixContentRead, RecoveryFieldsRead, VersionFieldsRead,
+};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
+
+/// 挂载之后那一次发布写的内容（装得进一个数据单元）。
+fn content_published_after_the_mount() -> Vec<u8> {
+    (0..321u32)
+        .map(|index| u8::try_from(index % 251).expect("小于 251") ^ 0x5a)
+        .collect()
+}
+
+/// 照抄一段写死的历史跑到底，交回停下时的整份盘面。
+fn pool_after_the_history(
+    width: HistoryDeviceWidth,
+    starting_point: HistoryStartingPoint,
+    operations: Vec<HistoryOperation>,
+    space_admission: SpaceAdmission,
+) -> MemoryPool {
+    let stream = SharedStream::retaining_contents();
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point,
+        operations,
+    };
+    let run = execute_history_with(
+        &history,
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: width,
+            space_admission,
+        },
+        &stream,
+        &mut |_observation| {},
+    );
+    assert_eq!(
+        run.outcomes.len(),
+        history.operations.len(),
+        "写死的那段历史每一步都要跑完：{:?}",
+        run.ending
+    );
+    let mut pool = MemoryPool::with_devices(&DEVICES, width.device_bytes());
+    pool.apply(&stream.retained_operations());
+    pool
+}
+
+fn devices_of(
+    pool: &MemoryPool,
+    parameters: &MakeFilesystemParameters,
+    stream: &SharedStream,
+) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
+    pool.devices
+        .iter()
+        .map(|(identity, sectors)| {
+            let mut device = SparseBlockDevice::new(
+                pool.device_size_in_bytes,
+                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
+            );
+            device.image = sectors.clone();
+            (
+                *identity,
+                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
+            )
+        })
+        .collect()
+}
+
+fn pool_of(
+    devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>,
+    device_size_in_bytes: u64,
+) -> MemoryPool {
+    MemoryPool {
+        devices: devices
+            .into_iter()
+            .map(|(identity, device)| (identity, device.into_inner_and_operations().0.image))
+            .collect(),
+        device_size_in_bytes,
+    }
+}
+
+fn mount(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>,
+) -> Mounted {
+    mount_writable_with_space_admission(parameters, devices, SpaceAdmission::JudgedByTheFormula)
+        .expect("小池上不带故障的盘面，可写挂载该成")
+}
+
+/// 挂载之后接着发一次布：现行那一版带文件时覆盖写，树表 0 条时发第一个文件（与崩溃注入同一种调法）。
+fn publish_once_after_the_mount(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)],
+    mounted: Mounted,
+) {
+    let content = content_published_after_the_mount();
+    let file = FirstFile {
+        content: &content,
+        write_time_seconds: 1_000,
+    };
+    let mut allocator = mounted.allocator;
+    let mut writer = PoolWriter::new(parameters, devices);
+    match &mounted.current {
+        PoolVersion::WithFile(current) => publish_overwrite(
+            &mut writer,
+            &mut allocator,
+            current,
+            file,
+            mounted.output.instance,
+        )
+        .map(|_| ()),
+        PoolVersion::WithoutFile(current) => publish_first_file(
+            &mut writer,
+            &mut allocator,
+            &current.root,
+            file,
+            mounted.output.instance,
+            &current.record_bytes,
+        )
+        .map(|_| ()),
+    }
+    .expect("挂载之后那一次发布该成");
+}
+
+/// 一次判下来要的三样：R1、R2 与动作。
+struct JudgedScenario {
+    before_the_mount: RecoveryFieldsRead,
+    after_the_action: RecoveryFieldsRead,
+    action: MountTransparencyAction,
+}
+
+impl JudgedScenario {
+    fn judge(&self) -> MountTransparency {
+        judge_mount_transparency(&self.before_the_mount, &self.after_the_action, self.action)
+    }
+}
+
+/// 在 `crash_image` 上只挂载（v2a）。
+fn mount_alone_on(width: HistoryDeviceWidth, crash_image: &MemoryPool) -> JudgedScenario {
+    let parameters = width.parameters();
+    let stream = SharedStream::new();
+    let mut devices = devices_of(crash_image, &parameters, &stream);
+    let mounted = mount(&parameters, &mut devices);
+    let floor = FloorRaisedByTheMount::of_the_mount_output(&mounted.output);
+    drop(mounted);
+    let after = pool_of(devices, crash_image.device_size_in_bytes);
+    JudgedScenario {
+        before_the_mount: read_recovery_fields(crash_image, ComparedVersion::TheRecoveredVersion),
+        after_the_action: read_recovery_fields(&after, ComparedVersion::TheRecoveredVersion),
+        action: MountTransparencyAction::MountAlone(floor),
+    }
+}
+
+/// 在 `crash_image` 上挂载加一次发布，R2 比发布之前那一版。
+fn mount_and_one_publish_on(width: HistoryDeviceWidth, crash_image: &MemoryPool) -> JudgedScenario {
+    let parameters = width.parameters();
+    let stream = SharedStream::new();
+    let mut devices = devices_of(crash_image, &parameters, &stream);
+    let mounted = mount(&parameters, &mut devices);
+    let floor = FloorRaisedByTheMount::of_the_mount_output(&mounted.output);
+    publish_once_after_the_mount(&parameters, &mut devices, mounted);
+    let after = pool_of(devices, crash_image.device_size_in_bytes);
+    JudgedScenario {
+        before_the_mount: read_recovery_fields(crash_image, ComparedVersion::TheRecoveredVersion),
+        after_the_action: read_recovery_fields(
+            &after,
+            ComparedVersion::TheVersionBeforeTheRecoveredOne,
+        ),
+        action: MountTransparencyAction::MountAndOnePublish(floor),
+    }
+}
+
+const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
+
+fn pool_after_make_filesystem() -> MemoryPool {
+    pool_after_the_history(
+        SMALL,
+        HistoryStartingPoint::AfterMakeFilesystem,
+        Vec::new(),
+        SpaceAdmission::JudgedByTheFormula,
+    )
+}
+
+fn pool_after_the_first_file() -> MemoryPool {
+    pool_after_the_history(
+        SMALL,
+        HistoryStartingPoint::AfterFirstFile,
+        Vec::new(),
+        SpaceAdmission::JudgedByTheFormula,
+    )
+}
+
+/// 第一个文件之后挂载、正常卸载：最新那几条根带卸载记号。
+fn pool_left_by_an_unmount() -> MemoryPool {
+    let pool = pool_after_the_first_file();
+    let parameters = SMALL.parameters();
+    let stream = SharedStream::new();
+    let mut devices = devices_of(&pool, &parameters, &stream);
+    let mounted = mount(&parameters, &mut devices);
+    let (_output, mut session) = MountedSession::of_the_mount(mounted);
+    let unmounted = unmount(
+        &parameters,
+        &mut devices,
+        &mut session.allocator,
+        &mut session.current,
+        session.shadow_ledger,
+    )
+    .expect("正常卸载");
+    assert!(
+        matches!(unmounted, Unmounted::FloorRaisedToTheCurrentVersion(_)),
+        "带文件的一版上卸载要推带卸载记号的空发布"
+    );
+    pool_of(devices, pool.device_size_in_bytes)
+}
+
+/// 第一个文件之后挂载、覆盖写一次，崩在那次覆盖写的根槽写之前：那次发布的单元与记录都落了，根没落。
+/// 只读恢复由记录重建那一版（施加 1 条以上、W 不为 0）。
+fn pool_rebuilt_from_a_record_of_an_overwrite() -> MemoryPool {
+    let pool = pool_after_the_first_file();
+    let parameters = SMALL.parameters();
+    let mount_stream = SharedStream::new();
+    let mut devices_of_the_mount = devices_of(&pool, &parameters, &mount_stream);
+    let mounted = mount(&parameters, &mut devices_of_the_mount);
+    let after_the_mount = pool_of(devices_of_the_mount, pool.device_size_in_bytes);
+    let publish_stream = SharedStream::retaining_contents();
+    let mut devices_of_the_publish = devices_of(&after_the_mount, &parameters, &publish_stream);
+    publish_once_after_the_mount(&parameters, &mut devices_of_the_publish, mounted);
+    let operations = publish_stream.retained_operations();
+    let (writes, _segments, stream_indexes) =
+        writes_and_segments_with_stream_indexes(&operations, &SMALL.fixed_geometry());
+    let root_write = writes
+        .iter()
+        .position(|write| write.kind == StepKind::RootRecordFua)
+        .expect("那次覆盖写有一次根槽写");
+    let mut crash_image = after_the_mount;
+    crash_image.apply(&operations[..stream_indexes[root_write]]);
+    crash_image
+}
+
+/// 单元区 256 槽的小盘：第一个文件之后重挂、关掉空间准入覆盖写 12 次，再判准入挂载一次（攻方 r5 的 fms256-12 那一种造法）：
+/// 挂载在写行与暖机之后推抬 F。
+fn pool_where_the_mount_raises_the_floor() -> MemoryPool {
+    let mut operations = vec![HistoryOperation::CloseAndMountWritable];
+    operations.extend(std::iter::repeat_n(
+        HistoryOperation::PublishOverwrite(ContentChoice {
+            length: ContentLength::InsideOneDataUnit { selector: 200 },
+            fill_seed: 11,
+        }),
+        12,
+    ));
+    pool_after_the_history(
+        HistoryDeviceWidth::UnitAreaOf256Slots,
+        HistoryStartingPoint::AfterFirstFile,
+        operations,
+        SpaceAdmission::SkippedByTheTestOnlySwitch,
+    )
+}
+
+/// 第一个文件之后的盘面上挂载，崩在取号那几次系统配置槽写之后、写行那次发布的第一个别的写之前（挂载途中的二次崩溃，取号那一段）。
+fn second_crash_after_the_acquisition(crash_image: &MemoryPool) -> JudgedScenario {
+    let parameters = SMALL.parameters();
+    let stream = SharedStream::retaining_contents();
+    let mut devices = devices_of(crash_image, &parameters, &stream);
+    drop(mount(&parameters, &mut devices));
+    let operations = stream.retained_operations();
+    let (writes, _segments, stream_indexes) =
+        writes_and_segments_with_stream_indexes(&operations, &SMALL.fixed_geometry());
+    let first_write_after_the_acquisition = writes
+        .iter()
+        .position(|write| write.kind != StepKind::SystemConfigurationSlot)
+        .expect("取号之后有写行那次发布的写");
+    let mut second_crash_image = crash_image.clone();
+    second_crash_image.apply(&operations[..stream_indexes[first_write_after_the_acquisition]]);
+    JudgedScenario {
+        before_the_mount: read_recovery_fields(crash_image, ComparedVersion::TheRecoveredVersion),
+        after_the_action: read_recovery_fields(
+            &second_crash_image,
+            ComparedVersion::TheRecoveredVersion,
+        ),
+        action: MountTransparencyAction::SecondCrashInsideTheMount(
+            FloorRaisesAtTheSecondCrash::NoneLandedYet,
+        ),
+    }
+}
+
+fn assert_holds(verdict: &MountTransparency, field: MountTransparencyField) {
+    assert_eq!(
+        verdict.outcome_of(field),
+        Some(&FieldOutcome::Holds),
+        "{field:?} 要判绿：{verdict:#?}"
+    );
+    assert!(
+        verdict.violated_fields().is_empty(),
+        "合法挂载上一样都不许红：{verdict:#?}"
+    );
+}
+
+fn assert_only_this_field_is_violated(verdict: &MountTransparency, field: MountTransparencyField) {
+    let violated: Vec<MountTransparencyField> = verdict
+        .violated_fields()
+        .iter()
+        .map(|field_verdict| field_verdict.field)
+        .collect();
+    assert_eq!(
+        violated,
+        vec![field],
+        "改坏的只有 {field:?}，红的也只该是它：{verdict:#?}"
+    );
+}
+
+fn compared_version_mut(fields: &mut RecoveryFieldsRead) -> &mut VersionFieldsRead {
+    fields
+        .compared_version
+        .as_mut()
+        .expect("动作之后的只读恢复有可比的那一版")
+}
+
+#[test]
+fn posix_content_of_the_version_before_the_publish_after_the_mount_is_the_content_recovered_from_the_crash_image(
+) {
+    let scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
+    assert_eq!(
+        PosixContentRead::of_the_recovery_report(&scenario.after_the_action.report),
+        PosixContentRead::File(content_published_after_the_mount()),
+        "前提：R2 恢复落到的是挂载之后那次发布的内容，比的是它之前那一版"
+    );
+    assert_holds(&scenario.judge(), MountTransparencyField::PosixContent);
+}
+
+#[test]
+fn posix_content_that_differs_from_the_crash_image_is_red() {
+    let mut scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
+    compared_version_mut(&mut scenario.after_the_action).content =
+        PosixContentRead::File(content_published_after_the_mount());
+    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::PosixContent);
+}
+
+#[test]
+fn the_immutable_configuration_except_this_device_is_the_same_after_a_mount_alone() {
+    let scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    assert_holds(
+        &scenario.judge(),
+        MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
+    );
+}
+
+#[test]
+fn an_immutable_configuration_with_another_device_count_after_the_mount_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    let immutable = scenario
+        .after_the_action
+        .immutable_configuration
+        .as_mut()
+        .expect("R2 择得到系统配置");
+    immutable.device_count += 1;
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
+    );
+}
+
+#[test]
+fn after_a_mount_on_a_pool_left_by_an_unmount_the_unmount_marker_is_not_written_by_the_unmount_sequence(
+) {
+    let scenario = mount_alone_on(SMALL, &pool_left_by_an_unmount());
+    assert_eq!(
+        scenario
+            .before_the_mount
+            .compared_version
+            .as_ref()
+            .expect("R1 落到一版")
+            .root
+            .unmount_marker,
+        UnmountMarker::WrittenByTheUnmountSequence,
+        "前提：R1 落到的是卸载那一串写的根（R1 的记号不进比较）"
+    );
+    assert_holds(&scenario.judge(), MountTransparencyField::UnmountMarker);
+}
+
+#[test]
+fn an_unmount_marker_after_the_mount_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_left_by_an_unmount());
+    compared_version_mut(&mut scenario.after_the_action)
+        .root
+        .unmount_marker = UnmountMarker::WrittenByTheUnmountSequence;
+    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::UnmountMarker);
+}
+
+#[test]
+fn a_mount_that_raises_no_floor_keeps_the_effective_floor_and_writes_it_into_its_root() {
+    let scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    assert_eq!(
+        scenario.action,
+        MountTransparencyAction::MountAlone(FloorRaisedByTheMount::NotRaised),
+        "前提：这次挂载没推抬 F"
+    );
+    assert_holds(&scenario.judge(), MountTransparencyField::RollbackFloor);
+}
+
+#[test]
+fn an_effective_floor_that_moved_although_the_mount_raised_none_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    let effective_floor = scenario
+        .after_the_action
+        .effective_rollback_floor
+        .expect("R2 算得出 F 生效值");
+    scenario.after_the_action.effective_rollback_floor = Some(CheckpointTxg(effective_floor.0 + 1));
+    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::RollbackFloor);
+}
+
+#[test]
+fn a_mount_that_raises_the_floor_leaves_the_effective_floor_at_the_last_one_it_raised_to() {
+    let width = HistoryDeviceWidth::UnitAreaOf256Slots;
+    let scenario = mount_alone_on(width, &pool_where_the_mount_raises_the_floor());
+    assert!(
+        matches!(
+            scenario.action,
+            MountTransparencyAction::MountAlone(FloorRaisedByTheMount::RaisedTo(_))
+        ),
+        "前提：这次挂载在写行与暖机之后推了抬 F：{:?}",
+        scenario.action
+    );
+    assert_holds(&scenario.judge(), MountTransparencyField::RollbackFloor);
+}
+
+#[test]
+fn an_effective_floor_other_than_the_last_one_the_mount_raised_to_is_red() {
+    let width = HistoryDeviceWidth::UnitAreaOf256Slots;
+    let mut scenario = mount_alone_on(width, &pool_where_the_mount_raises_the_floor());
+    let MountTransparencyAction::MountAlone(FloorRaisedByTheMount::RaisedTo(raised)) =
+        scenario.action
+    else {
+        panic!("前提：这次挂载推了抬 F：{:?}", scenario.action);
+    };
+    scenario.action = MountTransparencyAction::MountAlone(FloorRaisedByTheMount::RaisedTo(
+        CheckpointTxg(raised.0 + 1),
+    ));
+    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::RollbackFloor);
+}
+
+#[test]
+fn the_first_mount_of_a_new_pool_takes_one_above_every_instance_on_the_crash_image() {
+    let scenario = mount_alone_on(SMALL, &pool_after_make_filesystem());
+    assert_eq!(
+        scenario
+            .after_the_action
+            .compared_version
+            .as_ref()
+            .expect("R2 落到一版")
+            .root
+            .instance,
+        InstanceGeneration(1),
+        "前提：mkfs 之后第一次可写挂载取 1（D23 已定项 16）"
+    );
+    assert_holds(
+        &scenario.judge(),
+        MountTransparencyField::InstanceGeneration,
+    );
+}
+
+#[test]
+fn an_instance_generation_one_too_high_after_the_mount_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_after_make_filesystem());
+    let root = &mut compared_version_mut(&mut scenario.after_the_action).root;
+    root.instance = InstanceGeneration(root.instance.0 + 1);
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::InstanceGeneration,
+    );
+}
+
+#[test]
+fn instance_table_rows_follow_the_recovery_rule_when_the_crash_image_is_rebuilt_from_a_record() {
+    let scenario = mount_alone_on(SMALL, &pool_rebuilt_from_a_record_of_an_overwrite());
+    let journal = scenario.before_the_mount.report.journal;
+    assert!(
+        journal.prefix_applied > 0 && journal.maximum_applied_transaction > 0,
+        "前提：R1 由记录重建那一版，施加过的事务号不为 0：{journal:?}"
+    );
+    assert_holds(&scenario.judge(), MountTransparencyField::InstanceTableRows);
+}
+
+#[test]
+fn an_instance_table_row_whose_applied_transaction_is_dropped_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_rebuilt_from_a_record_of_an_overwrite());
+    let recovered_instance = scenario
+        .before_the_mount
+        .compared_version
+        .as_ref()
+        .expect("R1 落到一版")
+        .root
+        .instance;
+    let rows = compared_version_mut(&mut scenario.after_the_action)
+        .instance_table_rows
+        .as_mut()
+        .expect("R2 的实例表读得出");
+    let row = rows
+        .iter_mut()
+        .find(|row| row.instance == recovered_instance)
+        .expect("挂载给 R1 那个实例写了一行");
+    row.applied_transaction_high_water = 0;
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::InstanceTableRows,
+    );
+}
+
+#[test]
+fn the_tree_identifier_watermark_is_unchanged_by_a_mount_alone() {
+    let scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    assert_holds(
+        &scenario.judge(),
+        MountTransparencyField::TreeIdentifierWatermark,
+    );
+}
+
+#[test]
+fn a_tree_identifier_watermark_that_moved_with_the_mount_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    compared_version_mut(&mut scenario.after_the_action)
+        .root
+        .tree_identifier_watermark += 1;
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::TreeIdentifierWatermark,
+    );
+}
+
+#[test]
+fn the_two_user_visible_tree_root_pointers_before_the_publish_after_the_mount_are_those_of_the_crash_image(
+) {
+    let scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
+    assert_holds(
+        &scenario.judge(),
+        MountTransparencyField::UserVisibleTreeRootPointers,
+    );
+}
+
+#[test]
+fn an_inode_tree_root_pointer_that_moved_with_the_mount_is_red() {
+    let mut scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
+    let pointers = compared_version_mut(&mut scenario.after_the_action)
+        .user_visible_tree_root_pointers
+        .as_mut()
+        .expect("R2 那一版的树表读得出");
+    let inode_tree = pointers
+        .inode_tree
+        .as_mut()
+        .expect("带文件的一版树表里有 inode 树");
+    inode_tree[0] ^= 0xff;
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::UserVisibleTreeRootPointers,
+    );
+}
+
+#[test]
+fn the_checkpoint_txg_lies_above_every_root_and_record_of_a_crash_image_rebuilt_from_a_record() {
+    let scenario = mount_alone_on(SMALL, &pool_rebuilt_from_a_record_of_an_overwrite());
+    assert!(
+        scenario
+            .before_the_mount
+            .highest_record_checkpoint_txg_in_the_ring
+            > scenario.before_the_mount.highest_root_txg_in_the_ring,
+        "前提：环里有比全部根都新的记录（那次覆盖写的根没落）：{:?} / {:?}",
+        scenario
+            .before_the_mount
+            .highest_record_checkpoint_txg_in_the_ring,
+        scenario.before_the_mount.highest_root_txg_in_the_ring
+    );
+    assert_holds(&scenario.judge(), MountTransparencyField::CheckpointTxg);
+}
+
+#[test]
+fn a_checkpoint_txg_that_does_not_clear_the_newest_record_of_the_crash_image_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_rebuilt_from_a_record_of_an_overwrite());
+    let newest_record = scenario
+        .before_the_mount
+        .highest_record_checkpoint_txg_in_the_ring
+        .expect("环里有记录");
+    compared_version_mut(&mut scenario.after_the_action)
+        .root
+        .checkpoint_txg = newest_record;
+    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::CheckpointTxg);
+}
+
+#[test]
+fn the_recovery_after_the_mount_applies_no_record_even_when_the_crash_image_was_rebuilt_from_one() {
+    let scenario = mount_alone_on(SMALL, &pool_rebuilt_from_a_record_of_an_overwrite());
+    assert!(
+        scenario.before_the_mount.report.journal.prefix_applied > 0,
+        "前提：R1 施加过记录"
+    );
+    assert_holds(&scenario.judge(), MountTransparencyField::JournalScanReport);
+}
+
+#[test]
+fn a_recovery_after_the_mount_that_applies_a_record_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    scenario.after_the_action.report.journal.prefix_applied = 1;
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::JournalScanReport,
+    );
+}
+
+#[test]
+fn a_second_crash_right_after_the_acquisition_recovers_the_first_version_field_by_field() {
+    let scenario = second_crash_after_the_acquisition(&pool_after_the_first_file());
+    let verdict = scenario.judge();
+    for field in MountTransparencyField::ALL {
+        let expected = match field {
+            MountTransparencyField::JournalScanReport => {
+                FieldOutcome::NotJudged(FieldNotJudged::JournalScanReportOfASecondCrashImage)
+            }
+            MountTransparencyField::PosixContent
+            | MountTransparencyField::ImmutableConfigurationWithoutThisDevice
+            | MountTransparencyField::UnmountMarker
+            | MountTransparencyField::RollbackFloor
+            | MountTransparencyField::InstanceGeneration
+            | MountTransparencyField::InstanceTableRows
+            | MountTransparencyField::TreeIdentifierWatermark
+            | MountTransparencyField::UserVisibleTreeRootPointers
+            | MountTransparencyField::CheckpointTxg => FieldOutcome::Holds,
+        };
+        assert_eq!(
+            verdict.outcome_of(field),
+            Some(&expected),
+            "{field:?}：{verdict:#?}"
+        );
+    }
+}
+
+#[test]
+fn a_second_crash_recovering_content_other_than_the_first_version_is_red() {
+    let mut scenario = second_crash_after_the_acquisition(&pool_after_the_first_file());
+    compared_version_mut(&mut scenario.after_the_action).content = PosixContentRead::NoFile;
+    assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::PosixContent);
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_judged_against_the_d23_list.rs b/crates/singlefs-harness/tests/writable_mount_refusal_judged_against_the_d23_list.rs
new file mode 100644
index 0000000..199c330
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_judged_against_the_d23_list.rs
@@ -0,0 +1,259 @@
+//! 可写挂载被拒时判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal`，r5 判决第二节 U6）：拒绝合法 ⇔
+//! 拒的理由在 D23（journal 的角色与格式） 已定项 14 那张单子上、那一条判据在崩溃镜像上复算为真、盘上逐字节不变、只读读出在允许集合里。
+//! 判据由判法在镜像上自己复算，不经 O1 判官、不经实现判拒那一段。小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面。
+
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::mount::{
+    mount_writable_with_space_admission, MountError, NewerPublishWitness, RollbackTarget,
+    SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread, WitnessedCounterComparison,
+};
+use singlefs_harness::history::{
+    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistorySeed,
+    HistoryStartingPoint, PerStepChecker,
+};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use singlefs_harness::mount_transparency::{
+    judge_writable_mount_refusal, read_recovery_fields, ComparedVersion, DiskAfterTheRefusal,
+    ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet, RefusalReasonOnTheD23List,
+    WritableMountRefusalVerdict,
+};
+use singlefs_harness::SharedStream;
+
+const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
+
+fn pool_after_the_first_file() -> MemoryPool {
+    let stream = SharedStream::retaining_contents();
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: Vec::new(),
+    };
+    execute_history_with(
+        &history,
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: SMALL,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        },
+        &stream,
+        &mut |_observation| {},
+    );
+    let mut pool = MemoryPool::with_devices(
+        &[DeviceIdentity(0), DeviceIdentity(1)],
+        SMALL.device_bytes(),
+    );
+    pool.apply(&stream.retained_operations());
+    pool
+}
+
+/// 在 `crash_image` 的副本上起可写挂载：拒了交回错误与拒之后的盘面，做成了 panic。
+fn refused_mount_on(crash_image: &MemoryPool) -> (MountError, MemoryPool) {
+    let parameters = SMALL.parameters();
+    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = crash_image
+        .devices
+        .iter()
+        .map(|(identity, sectors)| {
+            let mut device = SparseBlockDevice::new(
+                crash_image.device_size_in_bytes,
+                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
+            );
+            device.image = sectors.clone();
+            (*identity, device)
+        })
+        .collect();
+    let error = match mount_writable_with_space_admission(
+        &parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    ) {
+        Err(error) => error,
+        Ok(mounted) => panic!(
+            "前提：这份盘面上可写挂载要拒，却挂上了实例 {:?}",
+            mounted.output.instance
+        ),
+    };
+    let after = MemoryPool {
+        devices: devices
+            .into_iter()
+            .map(|(identity, device)| (identity, device.image))
+            .collect(),
+        device_size_in_bytes: crash_image.device_size_in_bytes,
+    };
+    (error, after)
+}
+
+fn disk_after(crash_image: &MemoryPool, after: &MemoryPool) -> DiskAfterTheRefusal {
+    if crash_image == after {
+        DiskAfterTheRefusal::ByteForByteUnchanged
+    } else {
+        DiskAfterTheRefusal::Changed
+    }
+}
+
+/// 一个合法的拒（读阶段判据 N-配置 按真）在一份健康盘面上报出来：判据复算为假。
+fn witness_refusal_claimed_on(selected_version: (InstanceGeneration, CheckpointTxg)) -> MountError {
+    let reading = SelectedVersionAgainstTheWitness {
+        selected_version: RollbackTarget {
+            instance: selected_version.0,
+            checkpoint_txg: selected_version.1,
+        },
+        witness: NewerPublishWitness {
+            witnessed_journal_counter: 1,
+            comparison: WitnessedCounterComparison::Undecidable,
+        },
+    };
+    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
+        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
+            first_read: reading,
+            reread: reading,
+        },
+    ))
+}
+
+#[test]
+fn a_refusal_after_a_torn_system_configuration_slot_is_legal() {
+    let healthy = pool_after_the_first_file();
+    let content_of_the_healthy_pool =
+        read_recovery_fields(&healthy, ComparedVersion::TheRecoveredVersion);
+    let mut torn = healthy.clone();
+    let slot_spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
+    torn.devices
+        .get_mut(&DeviceIdentity(0))
+        .expect("有盘 0")
+        .write(DeviceOffsetInBytes(slot_spacing), &[0xa5; 4096]);
+    let (error, after) = refused_mount_on(&torn);
+    assert!(
+        matches!(
+            &error,
+            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
+                if matches!(
+                    **still_unreadable,
+                    StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. }
+                )
+        ),
+        "前提：撕了一个系统配置槽，读阶段判据 N-配置 按真、重读仍真，拒：{error:?}"
+    );
+    let fields_of_the_torn_pool = read_recovery_fields(&torn, ComparedVersion::TheRecoveredVersion);
+    let read_only_read_back = if fields_of_the_torn_pool
+        .compared_version
+        .as_ref()
+        .map(|version| &version.content)
+        == content_of_the_healthy_pool
+            .compared_version
+            .as_ref()
+            .map(|version| &version.content)
+    {
+        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
+    } else {
+        ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
+    };
+    assert_eq!(
+        judge_writable_mount_refusal(
+            &torn,
+            fields_of_the_torn_pool.report.effective_root,
+            &error,
+            disk_after(&torn, &after),
+            read_only_read_back,
+        ),
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
+        )
+    );
+}
+
+#[test]
+fn a_witness_refusal_on_a_healthy_pool_is_a_refusal_for_the_wrong_reason() {
+    let healthy = pool_after_the_first_file();
+    let fields = read_recovery_fields(&healthy, ComparedVersion::TheRecoveredVersion);
+    let selected_version = fields
+        .report
+        .effective_root
+        .expect("健康盘面上恢复落到一版");
+    assert_eq!(
+        judge_writable_mount_refusal(
+            &healthy,
+            fields.report.effective_root,
+            &witness_refusal_claimed_on(selected_version),
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            ],
+        }
+    );
+}
+
+#[test]
+fn an_unreadable_instance_table_refusal_on_a_healthy_pool_is_a_refusal_for_the_wrong_reason() {
+    let healthy = pool_after_the_first_file();
+    let fields = read_recovery_fields(&healthy, ComparedVersion::TheRecoveredVersion);
+    let refusal = MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
+        StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
+            newest_root_on_the_reread: None,
+        },
+    ));
+    assert_eq!(
+        judge_writable_mount_refusal(
+            &healthy,
+            fields.report.effective_root,
+            &refusal,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            ],
+        }
+    );
+}
+
+#[test]
+fn a_torn_slot_refusal_that_changed_the_disk_is_illegal() {
+    let healthy = pool_after_the_first_file();
+    let mut torn = healthy.clone();
+    let slot_spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
+    torn.devices
+        .get_mut(&DeviceIdentity(1))
+        .expect("有盘 1")
+        .write(DeviceOffsetInBytes(slot_spacing), &[0x5a; 4096]);
+    let (error, _after) = refused_mount_on(&torn);
+    let fields = read_recovery_fields(&torn, ComparedVersion::TheRecoveredVersion);
+    assert_eq!(
+        judge_writable_mount_refusal(
+            &torn,
+            fields.report.effective_root,
+            &error,
+            DiskAfterTheRefusal::Changed,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+            conditions_not_met: vec![RefusalConditionNotMet::DiskChangedByTheRefusedMount],
+        }
+    );
+}
+
+#[test]
+fn a_refusal_whose_reason_is_not_on_the_d23_list_is_left_to_the_campaign() {
+    let healthy = pool_after_the_first_file();
+    let fields = read_recovery_fields(&healthy, ComparedVersion::TheRecoveredVersion);
+    assert_eq!(
+        judge_writable_mount_refusal(
+            &healthy,
+            fields.report.effective_root,
+            &MountError::InstanceTableMalformed,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+        ),
+        WritableMountRefusalVerdict::ReasonNotOnTheD23List
+    );
+}
```

### mutations-append.tsv

```
挂载透明（11-4）：逐字段判的绿红颠倒（每一样都判反）	crates/singlefs-harness/src/mount_transparency.rs	    if holds {\n        FieldOutcome::Holds	    if !holds {\n        FieldOutcome::Holds	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	posix_content_of_the_version_before_the_publish_after_the_mount_is_the_content_recovered_from_the_crash_image
挂载透明（11-4）：POSIX 内容与存在性不比	crates/singlefs-harness/src/mount_transparency.rs	            compared.content == first_recovery.content,	            true,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	posix_content_that_differs_from_the_crash_image_is_red
挂载透明（11-4）：不可变配置不比	crates/singlefs-harness/src/mount_transparency.rs	                    immutable_configuration_is_the_same_except_this_device(before, after),	                    true,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	an_immutable_configuration_with_another_device_count_after_the_mount_is_red
挂载透明（11-4）：卸载记号照「必须与 R1 相同」判（r5 判决 U5 打中的那一读）	crates/singlefs-harness/src/mount_transparency.rs	compared.root.unmount_marker == UnmountMarker::NotWrittenByTheUnmountSequence,	compared.root.unmount_marker == first_recovery.root.unmount_marker,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	after_a_mount_on_a_pool_left_by_an_unmount_the_unmount_marker_is_not_written_by_the_unmount_sequence
挂载透明（11-4）：挂载没推抬 F 时 F 不比	crates/singlefs-harness/src/mount_transparency.rs	                    after_the_action.effective_rollback_floor == effective_floor_before\n                        && Some(compared.root.rollback_floor) == expected_floor_in_the_root,	                    true,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	an_effective_floor_that_moved_although_the_mount_raised_none_is_red
挂载透明（11-4）：挂载推了抬 F 仍照「F 生效值不变」判（r5 判决 U5 打中的那一读）	crates/singlefs-harness/src/mount_transparency.rs	after_the_action.effective_rollback_floor == Some(raised),	after_the_action.effective_rollback_floor == effective_floor_before,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_mount_that_raises_the_floor_leaves_the_effective_floor_at_the_last_one_it_raised_to
挂载透明（11-4）：新实例只要比根环里的大（漏了取号的 max 与 + 1）	crates/singlefs-harness/src/mount_transparency.rs	Some(compared.root.instance) == new_instance,	Some(compared.root.instance) > before_the_mount.highest_root_instance_in_the_ring,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	an_instance_generation_one_too_high_after_the_mount_is_red
挂载透明（11-4）：恢复写的行里所选根那个实例的 W 写 0	crates/singlefs-harness/src/mount_transparency.rs	                applied_transaction_high_water: maximum_applied_transaction,	                applied_transaction_high_water: 0,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	instance_table_rows_follow_the_recovery_rule_when_the_crash_image_is_rebuilt_from_a_record
挂载透明（11-4）：树 ID 水位不比	crates/singlefs-harness/src/mount_transparency.rs	            compared.root.tree_identifier_watermark\n                == first_recovery.root.tree_identifier_watermark,	            true,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_tree_identifier_watermark_that_moved_with_the_mount_is_red
挂载透明（11-4）：两棵树的根指针只看 R1 读不读得出、不比	crates/singlefs-harness/src/mount_transparency.rs	            first_recovery.user_visible_tree_root_pointers.is_some()\n                && compared.user_visible_tree_root_pointers\n                    == first_recovery.user_visible_tree_root_pointers,	            first_recovery.user_visible_tree_root_pointers.is_some(),	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	an_inode_tree_root_pointer_that_moved_with_the_mount_is_red
挂载透明（11-4）：txg 下界只取根环、不取环里自证记录的 checkpoint_txg	crates/singlefs-harness/src/mount_transparency.rs	                    .max(before_the_mount.highest_record_checkpoint_txg_in_the_ring)\n	                    .max(None)\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_checkpoint_txg_that_does_not_clear_the_newest_record_of_the_crash_image_is_red
挂载透明（11-4）：扫描报告不看施加条数	crates/singlefs-harness/src/mount_transparency.rs	journal.prefix_applied == 0 && journal.above_water == 0,	journal.above_water == 0,	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_recovery_after_the_mount_that_applies_a_record_is_red
挂载透明（11-4）：挂载加一次发布取「发布之前那一版」时连恢复落到的那一版也算（比成发布之后那一版）	crates/singlefs-harness/src/mount_transparency.rs	                < (recovered_root.checkpoint_txg, recovered_root.instance)	                <= (recovered_root.checkpoint_txg, recovered_root.instance)	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	posix_content_of_the_version_before_the_publish_after_the_mount_is_the_content_recovered_from_the_crash_image
挂载透明（11-4）：二次崩溃落回 v1 那一版也照挂载写的那一版判	crates/singlefs-harness/src/mount_transparency.rs	            if (compared.root.instance, compared.root.checkpoint_txg)\n                == (	            if false && (compared.root.instance, compared.root.checkpoint_txg)\n                == (	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_second_crash_right_after_the_acquisition_recovers_the_first_version_field_by_field
拒绝合法（11-4）：有一槽自证不过时 N-配置 复算成假（C331 取甲「按真」那一句丢了）	crates/singlefs-harness/src/mount_transparency.rs	            < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE\n        {\n            return true;\n        }	            < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE\n        {\n            return false;\n        }	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	a_refusal_after_a_torn_system_configuration_slot_is_legal
拒绝合法（11-4）：判据在崩溃镜像上复算为假也放行	crates/singlefs-harness/src/mount_transparency.rs	    if !criterion_holds_on_the_crash_image {	    if false && !criterion_holds_on_the_crash_image {	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	a_witness_refusal_on_a_healthy_pool_is_a_refusal_for_the_wrong_reason
拒绝合法（11-4）：最新那条根的实例表读不出在镜像上恒判真	crates/singlefs-harness/src/mount_transparency.rs	.is_none_or(|newest| instance_table_of_root(crash_image, &newest).is_none())	.is_none_or(|_newest| true)	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	an_unreadable_instance_table_refusal_on_a_healthy_pool_is_a_refusal_for_the_wrong_reason
拒绝合法（11-4）：拒之后盘变了不报	crates/singlefs-harness/src/mount_transparency.rs	            conditions_not_met.push(RefusalConditionNotMet::DiskChangedByTheRefusedMount);	            let _ = RefusalConditionNotMet::DiskChangedByTheRefusedMount;	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	a_torn_slot_refusal_that_changed_the_disk_is_illegal
拒绝合法（11-4）：单子外的理由也当单子上的 N-配置 判	crates/singlefs-harness/src/mount_transparency.rs	        | MountError::FloorRaiseFailedAfterTheMountsPublishes(_) => None,	        | MountError::FloorRaiseFailedAfterTheMountsPublishes(_) => {\n            Some(RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration)\n        }	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	a_refusal_whose_reason_is_not_on_the_d23_list_is_left_to_the_campaign
O1 判官判据 ③ 只比 txg、不比实例（11-4）	crates/singlefs-harness/src/model.rs	        if chosen < persisted {	        if chosen.checkpoint_txg < persisted.checkpoint_txg {	-p singlefs-harness --test crash_recovery_judge_refuses_a_root_older_than_the_newest_persisted_one	landing_on_the_lower_instance_of_the_same_txg_as_the_newest_persisted_root_is_red
O1 判官去掉判据 ③（11-4）	crates/singlefs-harness/src/model.rs	        if chosen < persisted {	        if false && chosen < persisted {	-p singlefs-harness --test crash_recovery_judge_refuses_a_root_older_than_the_newest_persisted_one	landing_on_an_older_txg_than_the_newest_persisted_root_is_red
崩溃注入（11-4）：挂载加一次发布那一个的挂载透明不判	crates/singlefs-checker-tier/src/crash_injection.rs	        (WritableMountOutcome::Published, Some(floor)) => {	        (WritableMountOutcome::Published, Some(floor)) if false => {	-p singlefs-checker-tier --lib	mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field
崩溃注入（11-4）：二次崩溃镜像上不判挂载透明	crates/singlefs-checker-tier/src/crash_injection.rs	    let Some(floor) = floor_raised_by_the_uninterrupted_mount else {	    let Some(floor) = floor_raised_by_the_uninterrupted_mount.filter(|_| false) else {	-p singlefs-checker-tier --lib	mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field
```

## 实现员补丁 m3-formal-11-5（`/tmp/claude-1000/m3-formal-11-5/patch/`）

```diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index c832678..eb53803 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -169,8 +169,30 @@ fn checker_region_device(g: CheckerSlot, region: u32) -> u32 {
     return g.region2;
 }
 
+// 全池没有一个可用的槽 0 时逐档读一块盘的槽 1（image::slot_one_read_at_every_slot_spacing_the_format_allows）：4096 起、每 512 一档、
+// 到根环起点（这一版读者只收 64 × 16384）减 4096 为止；只收可用、且记的槽距等于所在偏移的，取世代号最大的、同号取偏移小的；
+// 读到带不收之值的那一档按偏移升序交回第一个
+fn checker_slot_one_by_reading_every_spacing(device: u32) -> CheckerSlot {
+    var best = empty_checker_slot(2u);
+    var have_best = false;
+    let largest = ROOT_RING_BASE_BYTES - SYSTEM_CONFIGURATION_SLOT_BYTES;
+    for (var candidate = 4096u; candidate <= largest; candidate = candidate + 512u) {
+        let s = checker_read_system_configuration_slot(device, candidate);
+        if (s.reading == 1u) {
+            return s;
+        }
+        if (s.reading == 0u && s.spacing == candidate) {
+            if (!have_best || lt64(best.generation, s.generation)) {
+                best = s;
+                have_best = true;
+            }
+        }
+    }
+    return best;
+}
+
 // 每盘两槽的读法（image::system_configuration_slot_readings）：槽 1 的偏移按这块盘槽 0 记的槽距，不可用时按第一块槽 0 可用的盘的，
-// 全池都不可用时按 4096
+// 全池都不可用时逐档读（checker_slot_one_by_reading_every_spacing）
 var<private> slot_readings: array<CheckerSlot, 16>;
 
 fn read_every_system_configuration_slot() {
@@ -183,13 +205,15 @@ fn read_every_system_configuration_slot() {
         }
     }
     for (var d = 0u; d < device_count; d = d + 1u) {
-        var spacing = 4096u;
+        var spacing = first_spacing;
         if (slot_readings[d * 2u].reading == 0u) {
             spacing = slot_readings[d * 2u].spacing;
-        } else if (first_spacing != NONE) {
-            spacing = first_spacing;
         }
-        slot_readings[d * 2u + 1u] = checker_read_system_configuration_slot(hdr(12u + d), spacing);
+        if (spacing != NONE) {
+            slot_readings[d * 2u + 1u] = checker_read_system_configuration_slot(hdr(12u + d), spacing);
+        } else {
+            slot_readings[d * 2u + 1u] = checker_slot_one_by_reading_every_spacing(hdr(12u + d));
+        }
     }
 }
 
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs
new file mode 100644
index 0000000..9db0165
--- /dev/null
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs
@@ -0,0 +1,273 @@
+//! checker 档模块：crash
+//! io_min 8192（固定结构槽距 8192）的池上第一次可写挂载在层 0 的证据：mkfs(io_min 8192) → 可写挂载（取号 1、写行发布 txg 1、暖机 txg 2），
+//! 把 mkfs 之后的整条录制流按 D13（验证路线） 已定项 4 取崩溃状态，跑恢复 + 多版本 oracle、池级 checker、记录核对器。
+//!
+//! 比已有的流多罩的崩溃状态：已有的层 0 流全是槽距 4096 的池（`crates/singlefs-harness/tests/common/mod.rs` 的 `parameters()`），
+//! 这一条的基线镜像是槽距 8192 的池，系统配置槽 1 在 8192 处。取号那一段与暖机那次轮换那一段各有一个「两块盘的槽 0 同时撕裂」的状态
+//! （全池没有可用的槽 0），恢复与池级 checker 都要逐档找槽 1（C651（全池没有可用的系统配置槽 0 时 checker 与实现找槽 1 的读法不同）
+//! 第一轮判决第四节第 3 条，`research/prompts/m3-c651-r1-main-verification.md`）：checker 按最小槽距 4096 读时这两个状态判 I-8.5 红。
+//! 多跑的一步：没有；流到第一次可写挂载为止（取号、两次零单元发布），不发文件版本，每个状态只跑恢复与 checker、不重开挂载。
+//! 盘是内存稀疏盘外面包录制器（同一个进程里 mkfs 与挂载，不经文件镜像）。
+
+use singlefs_checker::image::{
+    chosen_system_configurations, system_configuration_slot_readings,
+    SystemConfigurationSlotReading,
+};
+use singlefs_checker_tier::crash::{
+    enumerate_layer0_quick_tier_versions, enumerate_layer0_selecting_versions_observing_each_state,
+    full_expansion, layer0_state_count_with_torn_in_place_overwrites, Layer0Tally,
+};
+use singlefs_core::address::DeviceIdentity;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
+use singlefs_core::mount::mount_writable;
+use singlefs_core::recovery::{choose_system_configuration, RecoveryReport};
+use singlefs_core::root_ring::RootRingSlotsPerRegion;
+use singlefs_core::system_configuration::SystemImmutableSizes;
+use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
+use singlefs_harness::memory_pool::{
+    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, RetainedWrite,
+    SparseBlockDevice,
+};
+use singlefs_harness::segments::{FixedGeometry, StepKind};
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+const DEVICE_BYTES: u64 = 4 << 30;
+const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-iomin8k";
+/// mkfs 时探测的 io_min：槽距 = 4096 向上取整到它的整数倍 = 8192（D2（RAID 条带策略） 已定项 19）。
+const MINIMUM_INPUT_OUTPUT_BYTES: u32 = 8192;
+const SLOT_SPACING_IN_BYTES: u64 = 8192;
+/// 全量的状态数：七段，取号与两次轮换三个系统配置槽 2 写段（原地覆写各取三态）各 3² − 1，两个记录 2 写段各 2² − 1，
+/// 两个根槽 1 写段各 1，再加全部持久那一个：1 + 3 × 8 + 2 × 3 + 2 × 1。
+const FULL_STATES: u64 = 33;
+
+fn parameters() -> MakeFilesystemParameters {
+    MakeFilesystemParameters {
+        filesystem_identifier: FILESYSTEM_IDENTIFIER,
+        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+        geometry: SystemImmutableSizes {
+            physical_block_size: 512,
+            minimum_input_output_bytes: MINIMUM_INPUT_OUTPUT_BYTES,
+            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(
+                MINIMUM_INPUT_OUTPUT_BYTES,
+            ),
+            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
+            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
+        },
+    }
+}
+
+fn geometry() -> FixedGeometry {
+    FixedGeometry {
+        fixed_structure_slot_spacing: parameters().geometry.fixed_structure_slot_spacing,
+        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
+        root_ring_slots_per_region: parameters().geometry.root_ring_slots_per_region,
+    }
+}
+
+struct Prepared {
+    base: MemoryPool,
+    writes: Vec<RetainedWrite>,
+    segments: Vec<Vec<usize>>,
+    /// 被判的那次根槽 FUA 写：暖机那一代（txg 2）的根。
+    judged_root_index: usize,
+}
+
+fn prepare() -> Prepared {
+    assert_eq!(
+        u64::from(parameters().geometry.fixed_structure_slot_spacing),
+        SLOT_SPACING_IN_BYTES,
+        "io_min 8192 的池槽距 8192"
+    );
+    let stream = SharedStream::retaining_contents();
+    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
+        [DeviceIdentity(0), DeviceIdentity(1)]
+            .into_iter()
+            .map(|identity| {
+                (
+                    identity,
+                    RecordingBlockDevice::with_shared_stream(
+                        identity,
+                        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
+                        stream.clone(),
+                    ),
+                )
+            })
+            .collect();
+    make_filesystem(&parameters(), &mut devices).expect("io_min 8192 的池 mkfs");
+    let mkfs_operation_count = stream.operations().len();
+    mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
+    let operations = stream.retained_operations();
+    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], DEVICE_BYTES);
+    base.apply(&operations[..mkfs_operation_count]);
+    let (writes, segments) = writes_and_segments(&operations[mkfs_operation_count..], &geometry());
+    assert_eq!(
+        segments.iter().map(Vec::len).collect::<Vec<_>>(),
+        vec![2, 2, 1, 2, 2, 1, 2],
+        "取号两写 | 写行发布的记录两写 | 根 | 系统配置两写 | 暖机的记录两写 | 根 | 系统配置两写"
+    );
+    assert_eq!(
+        segments
+            .iter()
+            .map(|segment| writes[segment[0]].kind)
+            .collect::<Vec<StepKind>>(),
+        vec![
+            StepKind::SystemConfigurationSlot,
+            StepKind::JournalRecord,
+            StepKind::RootRecordFua,
+            StepKind::SystemConfigurationSlot,
+            StepKind::JournalRecord,
+            StepKind::RootRecordFua,
+            StepKind::SystemConfigurationSlot,
+        ],
+        "七段的种类"
+    );
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("写流里有根槽写");
+    Prepared {
+        base,
+        writes,
+        segments,
+        judged_root_index,
+    }
+}
+
+fn assert_recovery_checker_and_record_checker_clean(tally: &Layer0Tally) {
+    assert_eq!(
+        tally.violations, 0,
+        "多版本 oracle 的第一处违例：{:?}",
+        tally.first_violation
+    );
+    assert_eq!(
+        tally.ignored_violations, 0,
+        "不看 journal 那一遍恢复同样过 oracle：{:?}",
+        tally.first_ignored_violation
+    );
+    assert_eq!(tally.failed_states, 0, "每个状态都恢复得了");
+    assert_eq!(
+        (
+            tally.record_root_without_record,
+            tally.record_claimed_state_missing_unit
+        ),
+        (0, 0),
+        "记录核对器两条判据恒 0"
+    );
+    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
+        assert_eq!(
+            tally
+                .checker_violated_states
+                .get(invariant)
+                .copied()
+                .unwrap_or(0),
+            0,
+            "{invariant} 判违例的状态数必须是 0：{:?}",
+            tally.checker_first_violation.get(invariant)
+        );
+    }
+}
+
+/// 快档：段序列与全量的状态数钉住（七段、33 态），每段按甲二展开跑一遍恢复与 checker。
+#[test]
+fn the_first_writable_mount_of_an_io_min_8192_pool_has_seven_segments_and_33_crash_states() {
+    let prepared = prepare();
+    assert_eq!(
+        closed_form_state_count(&prepared.segments),
+        1 + 5 * 3 + 2,
+        "每次写只取两态时：1 + 五个 2 写段各 3 + 两个 1 写段各 1"
+    );
+    assert_eq!(
+        layer0_state_count_with_torn_in_place_overwrites(
+            &prepared.base,
+            &prepared.writes,
+            &prepared.segments,
+            &full_expansion,
+        ),
+        FULL_STATES,
+        "原地覆写取三态的全量：1 + 3 × (3² − 1) + 2 × 3 + 2 × 1"
+    );
+    let tally = enumerate_layer0_quick_tier_versions(
+        &prepared.base,
+        &prepared.writes,
+        &prepared.segments,
+        prepared.judged_root_index,
+        &[],
+    );
+    assert!(tally.states > 1, "快档展开出了崩溃状态");
+    assert_recovery_checker_and_record_checker_clean(&tally);
+}
+
+/// 全量：33 个崩溃状态逐个跑恢复与 checker，零违例；两块盘的槽 0 都不可用的状态恰好两个（取号那一段、暖机那次轮换那一段各一个
+/// 两盘同撕），每一个上 checker 两块盘的槽 1 都在 8192 处读成可用，checker 与恢复择到的世代号相同。
+#[test]
+#[ignore = "checker 档全量：崩溃枚举用例，由门禁 54 号 --full 按 crash-case 登记跑"]
+fn every_crash_state_of_the_first_writable_mount_of_an_io_min_8192_pool_recovers_and_the_checker_reads_slot_one_at_8192(
+) {
+    let prepared = prepare();
+    let mut states_without_a_usable_slot_zero: u64 = 0;
+    let mut disagreements: Vec<String> = Vec::new();
+    let tally = enumerate_layer0_selecting_versions_observing_each_state(
+        &prepared.base,
+        &prepared.writes,
+        &prepared.segments,
+        prepared.judged_root_index,
+        &[],
+        &|_segment_index, _segment| true,
+        &mut |image: &CrashImage<'_>, _report: &RecoveryReport| {
+            let readings = system_configuration_slot_readings(image);
+            let no_usable_slot_zero = readings
+                .iter()
+                .all(|(_, slots)| !matches!(slots[0], SystemConfigurationSlotReading::Usable(..)));
+            if !no_usable_slot_zero {
+                return;
+            }
+            states_without_a_usable_slot_zero += 1;
+            let slot_one_spacings: Vec<Option<u64>> = readings
+                .iter()
+                .map(|(_, slots)| match &slots[1] {
+                    SystemConfigurationSlotReading::Usable(_, geometry) => {
+                        Some(geometry.slot_spacing)
+                    }
+                    SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
+                    | SystemConfigurationSlotReading::NotSelfVerified
+                    | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => None,
+                })
+                .collect();
+            let checker_generation_of_device_zero = chosen_system_configurations(image)
+                .into_iter()
+                .find(|(device, _)| *device == 0)
+                .and_then(|(_, chosen)| chosen.map(|(view, _)| view.slot_generation));
+            let recovery_generation = choose_system_configuration(image)
+                .ok()
+                .map(|chosen| chosen.quantities.slot_generation);
+            if slot_one_spacings != vec![Some(SLOT_SPACING_IN_BYTES); 2]
+                || checker_generation_of_device_zero.is_none()
+                || checker_generation_of_device_zero != recovery_generation
+            {
+                disagreements.push(format!(
+                    "槽 1 读到的槽距 {slot_one_spacings:?}、checker 盘 0 择的世代号 {checker_generation_of_device_zero:?}、恢复择的 {recovery_generation:?}"
+                ));
+            }
+        },
+    );
+    println!(
+        "LAYER0_IO_MIN_8192_MOUNT states={} closed_form={FULL_STATES} exhaustive={} states_without_a_usable_slot_zero={states_without_a_usable_slot_zero} violations={} failed={} checker_by_invariant(evaluated/violated/not_applicable) {}",
+        tally.states,
+        tally.states == FULL_STATES,
+        tally.violations,
+        tally.failed_states,
+        tally.checker_counts_by_invariant()
+    );
+    assert_eq!(tally.states, FULL_STATES, "全量 33 态");
+    assert_recovery_checker_and_record_checker_clean(&tally);
+    assert_eq!(
+        states_without_a_usable_slot_zero, 2,
+        "两块盘的槽 0 都不可用的状态：取号那一段两盘同撕、暖机那次轮换两盘同撕，各一个"
+    );
+    assert_eq!(
+        disagreements,
+        Vec::<String>::new(),
+        "全池没有可用的槽 0 时，checker 两块盘的槽 1 都在 8192 处可用、与恢复择同一个世代号"
+    );
+}
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 157eaa8..64dc83c 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -560,9 +560,73 @@ fn read_system_configuration_slot(
     }
 }
 
+/// 逐档读槽 1 时相邻两档槽距之差：槽距 = 4096 向上取整到 mkfs 时探测的 io_min 的整数倍（D2（RAID 条带策略） 已定项 19），
+/// io_min 是逻辑块宽的整数倍、逻辑块宽最小 512 字节，所以格式允许的槽距都是 512 的整数倍。
+const SLOT_SPACING_STEP_BYTES: u64 = 512;
+
+/// 逐档读槽 1 时槽距的上界：槽 1 整槽落在根环起点之前（[`fixed_structure_slot_spacing_lies_in_the_format_range`]），
+/// 根环起点取这一版读者收的那一个（「根环起点」字段只收第一版常量 [`ROOT_RING_BASE_SLOT`]，[`geometry_of`] 判），× 16384 − 4096。
+const LARGEST_SLOT_SPACING_THIS_READER_ACCEPTS: u64 =
+    ROOT_RING_BASE_SLOT * SLOT_BYTES - SYSTEM_CONFIGURATION_SLOT_BYTES;
+
+/// 全池没有一个可用的槽 0 时，一块盘的槽 1：槽距无处可读，在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、
+/// 不越过 [`LARGEST_SLOT_SPACING_THIS_READER_ACCEPTS`]），只收可用、且自述的固定结构槽距等于所在偏移的槽
+/// （槽 i 的设备内偏移 = i × 固定结构槽距，D22（单元原子性怎么合成） 已定项 16）；几档都收得到时取世代号最大的，
+/// 世代号相同取偏移小的（C651（全池没有可用的系统配置槽 0 时 checker 与实现找槽 1 的读法不同） 第一轮判决第四节第 1、2 条，
+/// `research/prompts/m3-c651-r1-main-verification.md`）。
+///
+/// 读到的某一档带这一版读者不收的值（[`SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses`]）：交回按偏移升序读到的第一个，
+/// 由 I-7.13（系统配置池级字段在读者收的范围里） 报违例、整池不作保——D22（单元原子性怎么合成） 已定项 16 第 5 句「越界即拒」，
+/// 与槽 0、槽 1 一个处置；它记的槽距可能正是越界的那一格，不核偏移。
+/// 一档可用的都没收到时，有自证过而不可择的就交回 [`SystemConfigurationSlotReading::SelfVerifiedButNotChoosable`]
+/// （I-8.5（tail 槽有效集非空） 数它；它记的槽距可能读不出来，不核偏移），再没有就是 [`SystemConfigurationSlotReading::NotSelfVerified`]。
+fn slot_one_read_at_every_slot_spacing_the_format_allows(
+    reader: &dyn ImageReader,
+    device: u32,
+) -> SystemConfigurationSlotReading {
+    let mut highest_generation_accepted: Option<(SystemConfigurationView, PoolGeometry)> = None;
+    let mut some_slot_verifies_itself_but_is_not_choosable = false;
+    // 迭代次数的上界是档数 (LARGEST − 4096) ÷ 512 + 1 = 2033；跨轮带着的是至今世代号最大的那一槽与「见过自证过而不可择的」，
+    // 前者只在世代号严格更大时换（同号留偏移小的），后者只置不清；提前出口只有读到带不收之值的那一档。
+    for candidate_spacing in (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES
+        ..=LARGEST_SLOT_SPACING_THIS_READER_ACCEPTS)
+        .step_by(usize::try_from(SLOT_SPACING_STEP_BYTES).expect("512"))
+    {
+        match read_system_configuration_slot(reader, device, candidate_spacing) {
+            SystemConfigurationSlotReading::Usable(view, geometry) => {
+                if geometry.slot_spacing != candidate_spacing {
+                    continue;
+                }
+                if highest_generation_accepted
+                    .as_ref()
+                    .is_none_or(|(accepted, _)| view.slot_generation > accepted.slot_generation)
+                {
+                    highest_generation_accepted = Some((view, geometry));
+                }
+            }
+            refused @ SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. } => {
+                return refused;
+            }
+            SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => {
+                some_slot_verifies_itself_but_is_not_choosable = true;
+            }
+            SystemConfigurationSlotReading::NotSelfVerified => {}
+        }
+    }
+    match (
+        highest_generation_accepted,
+        some_slot_verifies_itself_but_is_not_choosable,
+    ) {
+        (Some((view, geometry)), _) => SystemConfigurationSlotReading::Usable(view, geometry),
+        (None, true) => SystemConfigurationSlotReading::SelfVerifiedButNotChoosable,
+        (None, false) => SystemConfigurationSlotReading::NotSelfVerified,
+    }
+}
+
 /// 每盘两槽各读成一个 [`SystemConfigurationSlotReading`]：槽 0 在偏移 0；槽 1 的偏移按这块盘槽 0 记的槽距，这块盘的槽 0 不可用时
-/// 按池里（`reader.devices()` 的次序）第一块槽 0 可用的盘记的槽距，全池的槽 0 都不可用时才按最小槽距 4096（2026-09-14 用户收尾弹窗定甲；
-/// 借别的盘那一档与恢复 `crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration` 同一个读法，实审 A2a 第 37 条改的，这里跟着对齐）。
+/// 按池里（`reader.devices()` 的次序）第一块槽 0 可用的盘记的槽距（2026-09-14 用户收尾弹窗定甲；借别的盘那一档与恢复
+/// `crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration` 同一个读法，实审 A2a 第 37 条改的，这里跟着对齐）；
+/// 全池的槽 0 都不可用时逐档读（[`slot_one_read_at_every_slot_spacing_the_format_allows`]，C651 第一轮判决定逐档，取代此前的最小槽距 4096）。
 #[must_use]
 pub fn system_configuration_slot_readings(
     reader: &dyn ImageReader,
@@ -584,10 +648,12 @@ pub fn system_configuration_slot_readings(
     slot_zero_of_each_device
         .into_iter()
         .map(|(device, slot_zero)| {
-            let spacing = spacing_recorded_by_slot_zero(&slot_zero)
+            let slot_one = match spacing_recorded_by_slot_zero(&slot_zero)
                 .or(spacing_recorded_by_the_first_valid_slot_zero)
-                .unwrap_or(FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES);
-            let slot_one = read_system_configuration_slot(reader, device, spacing);
+            {
+                Some(spacing) => read_system_configuration_slot(reader, device, spacing),
+                None => slot_one_read_at_every_slot_spacing_the_format_allows(reader, device),
+            };
             (device, [slot_zero, slot_one])
         })
         .collect()
diff --git a/crates/singlefs-core/src/make_filesystem.rs b/crates/singlefs-core/src/make_filesystem.rs
index 6e4b15a..adbc5d9 100644
--- a/crates/singlefs-core/src/make_filesystem.rs
+++ b/crates/singlefs-core/src/make_filesystem.rs
@@ -522,7 +522,7 @@ pub fn make_filesystem<Device: BlockDevice>(
         u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"),
         &[],
     );
-    // 先把根环三个区域与 journal 环整段清零（每块盘每段一次调用、录制流各一步），再写元数据。
+    // 先把根环基址之前那一段、根环三个区域与 journal 环整段清零（每块盘每段一次调用、录制流各一步），再写元数据。
     // 为什么 mkfs 自己做：一条 journal 记录、一条根记录合不合法只看它自己的魔数与校验和，盘上的旧字节里
     // 凑出一条合法记录的概率不是 0（上一次 mkfs 留下的就是真合法的记录），而「这块盘是新建的全零镜像」
     // 是装置的假设、真设备上不成立。
@@ -535,9 +535,9 @@ pub fn make_filesystem<Device: BlockDevice>(
     // 它今天读不到全靠择根与 checker 两处各自按归属读（`recovery::visit_valid_roots`、
     // `singlefs-checker` 的 `image.rs`），任一处以后改成扫全部盘，漏就回来了。
     // 按几何清之后 mkfs 的后置条件只有一句：根环三段在每块盘上只剩这次写下的三条第 0 代根，别处全 0。
-    // 次序与屏障：清零按设备内偏移升序发（根环 1 / 4 / 7 MiB，journal 环 16 MiB），几段之间不另加屏障——
+    // 次序与屏障：清零按设备内偏移升序发（[0, 1 MiB)，根环 1 / 4 / 7 MiB，journal 环 16 MiB），几段之间不另加屏障——
     // 它们互不重叠，也不与同一段里的单元写重叠（`check_geometry` 判过固定结构两两不重叠；单元区起点是环末尾的下一个槽）；真正重叠的是清根环与根槽 FUA 写
-    //（区域 r 的槽 0），那一对由原有的那道屏障（单元写之后、根 FUA 之前）隔开，不靠发出次序。
+    //（区域 r 的槽 0）、清 [0, 1 MiB) 与系统配置两槽的写，这两对都由原有的那道屏障（单元写之后、根 FUA 之前）隔开，不靠发出次序。
     // mkfs 中途崩溃第一版没有条款（层 0 从 mkfs 之后的池起枚举，`.claude/kb/layout/01-first-txn.md` 八
     // mkfs 那一行的「层 0 枚举」格写「不在」），不为一个没有条款的语义加屏障。
     // 清的是 S 个槽那么长的一段，S 取 mkfs 参数里的那个——这一次写进系统配置的也是它，
@@ -547,7 +547,14 @@ pub fn make_filesystem<Device: BlockDevice>(
         parameters.geometry.root_ring_slots_per_region,
     );
     let journal_ring_start = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
+    // 根环基址之前那一段 [0, 根环基址) 也整段清零，每块盘一次调用、发在最前（按偏移升序）：系统配置两槽住在这一段里，
+    // 槽 1 的偏移随槽距走。在用过的盘上换一个 io_min 重做 mkfs，旧池写在别的槽距上的槽 1 自证得过、记的槽距也等于它所在的偏移，
+    // 全池没有可用的槽 0 时逐档读槽 1 的读者（恢复与池级 checker）分不出它与本池的槽 1、按世代号会择中它
+    // （C651（全池没有可用的系统配置槽 0 时 checker 与实现找槽 1 的读法不同） 第一轮判决岔路 W，用户 2026-10-02 定 W1 mkfs 清零，
+    // `research/prompts/m3-c651-r1-main-verification.md`「用户定案」）。与根环、journal 环的清零同一种动作。
+    let root_ring_base = region_start(0);
     for (_, device) in devices.iter_mut() {
+        device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;
         for region_to_clear in 0..ROOT_RING_REGIONS {
             device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;
         }
diff --git a/crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs b/crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs
index f372f08..70fd83b 100644
--- a/crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs
+++ b/crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs
@@ -273,29 +273,32 @@ fn recorded_stream_matches_the_registered_mkfs_segment_sequence() {
         root_ring_slots_per_region: parameters().geometry.root_ring_slots_per_region,
     };
     let segments = split_into_segments(&operations, &geometry);
+    // mkfs 清 [0, 根环基址)（C651 第一轮判决用户定案 W1）之后每块盘多一次清零、都在第一段：第一段 12 → 14 个写、操作 23 → 25；
+    // layout/01-first-txn.md 八 mkfs 那一行要跟着改（写回另派）。
     assert_eq!(
         segment_sizes_text(&segments),
-        "12+1+1+1+4",
+        "14+1+1+1+4",
         "layout/01-first-txn.md 八 mkfs 那一行"
     );
     assert_eq!(
         segment_kinds_text(&segments),
-        "[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]"
+        "[zero_fill×10,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]"
     );
     assert_eq!(
         operations.len(),
-        23,
-        "mkfs 23 次操作（每盘 3 段根环 + 1 段环共八次清零、四个单元写、三次根 FUA、四次系统配置槽写、两道池屏障——录制器按设备记屏障，每道两块盘各记一步）"
+        25,
+        "mkfs 25 次操作（每盘 [0, 根环基址) 一段 + 3 段根环 + 1 段环共十次清零、四个单元写、三次根 FUA、四次系统配置槽写、两道池屏障——录制器按设备记屏障，每道两块盘各记一步）"
     );
     assert_eq!(
         check_stream_integrity(&pool.stream.render()),
-        StreamIntegrity::Consistent { operations: 23 }
+        StreamIntegrity::Consistent { operations: 25 }
     );
     remove_images(&pool);
 }
 
 /// 并行线四的验收第一条：mkfs 之后抽样读 journal 环全 0，mkfs 交给设备的写字节数
-/// = 116 224 + 2 × 768 MiB + 6 × 32 KiB（末一项是 2026-09-22 连根环一起清之后加的：两盘各三个区域）。
+/// = 116 224 + 2 × 768 MiB + 6 × 32 KiB + 2 × 1 MiB（6 × 32 KiB 是 2026-09-22 连根环一起清之后加的：两盘各三个区域；
+/// 2 × 1 MiB 是 C651 第一轮判决用户定案 W1 连 [0, 根环基址) 一起清之后加的：两盘各一段）。
 ///
 /// 镜像先在环里撒上非 0（稀疏镜像本来就读回全 0，不先弄脏就分不出「清过」与「从来没写过」，
 /// `test-discipline.md`「读不到 ≠ 读到 0」），再跑 mkfs：环里抽样点全 0，说明清零真的发生过；
@@ -304,6 +307,8 @@ fn recorded_stream_matches_the_registered_mkfs_segment_sequence() {
 fn mkfs_zeroes_the_whole_journal_ring_and_writes_the_registered_number_of_bytes() {
     const RING_START: u64 = JOURNAL_RING_START_SLOT * SLOT_BYTES;
     const MKFS_METADATA_BYTES: u64 = 116_224;
+    /// [0, 根环基址)：根环基址 = 「根环起点」字段 64 × 16 KiB = 1 MiB（D22（单元原子性怎么合成） 已定项 16 第 1 句）。
+    const BELOW_THE_ROOT_RING_BASE_BYTES: u64 = 1 << 20;
 
     let stream = SharedStream::new();
     let mut paths = Vec::new();
@@ -363,9 +368,22 @@ fn mkfs_zeroes_the_whole_journal_ring_and_writes_the_registered_number_of_bytes(
         .collect();
     assert_eq!(
         zero_fills.len(),
-        8,
-        "每块盘四次清零调用：三个根环区域各一次、journal 环一次"
+        10,
+        "每块盘五次清零调用：[0, 根环基址) 一次（C651 第一轮判决用户定案 W1）、三个根环区域各一次、journal 环一次"
     );
+    let below_the_root_ring_base_zero_fills: Vec<&&singlefs_harness::RecordedOperation> =
+        zero_fills
+            .iter()
+            .filter(|zero_fill| zero_fill.offset == DeviceOffsetInBytes(0))
+            .collect();
+    assert_eq!(
+        below_the_root_ring_base_zero_fills.len(),
+        2,
+        "两块盘各清一次 [0, 根环基址)"
+    );
+    for zero_fill in &below_the_root_ring_base_zero_fills {
+        assert_eq!(zero_fill.length, BELOW_THE_ROOT_RING_BASE_BYTES);
+    }
     let journal_zero_fills: Vec<&&singlefs_harness::RecordedOperation> = zero_fills
         .iter()
         .filter(|zero_fill| zero_fill.offset == DeviceOffsetInBytes(RING_START))
@@ -399,10 +417,13 @@ fn mkfs_zeroes_the_whole_journal_ring_and_writes_the_registered_number_of_bytes(
         .sum();
     assert_eq!(
         written_bytes,
-        MKFS_METADATA_BYTES + 2 * JOURNAL_RING_DEFAULT_BYTES + root_ring_zeroed_bytes,
-        "并行线四验收（连根环一起清之后重算）：116 224 + 2 × 768 MiB + 6 × 32 KiB = 1 610 925 568"
+        MKFS_METADATA_BYTES
+            + 2 * JOURNAL_RING_DEFAULT_BYTES
+            + root_ring_zeroed_bytes
+            + 2 * BELOW_THE_ROOT_RING_BASE_BYTES,
+        "并行线四验收（连根环一起清、再连 [0, 根环基址) 一起清之后重算）：116 224 + 2 × 768 MiB + 6 × 32 KiB + 2 × 1 MiB = 1 613 022 720"
     );
-    assert_eq!(written_bytes, 1_610_925_568, "验收那个数写成一个字面量");
+    assert_eq!(written_bytes, 1_613_022_720, "验收那个数写成一个字面量");
     assert_eq!(
         JOURNAL_RING_DEFAULT_BYTES,
         768 * 1024 * 1024,
diff --git a/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs b/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
index 1dc9d01..c367441 100644
--- a/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
+++ b/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
@@ -326,17 +326,20 @@ fn recorded_paths_match_the_registered_segment_sequences() {
     // 那道没有写在前、写入口不发的屏障，只多第二次末尾 2 步），新池新建文件 33 → 35，mkfs 之后整条 2 + 20 + 35 = 57；
     // 暖机第二次的系统配置槽轮换不再与新池新建文件的 24 个单元写同段：整条流 26 写一段拆成 2 与 24，
     // 闭式 1 + (3 + 3 + 1 + 3 + 3 + 1 + 3 + 3 + 1 + 3) + (2^24 − 1) = 16777240（改之前 67108885）。
+    // mkfs 清 [0, 根环基址)（C651 第一轮判决用户定案 W1）之后每块盘多一步整段清零、两块盘多 2 步，都在第一段（第一道屏障之前）：
+    // mkfs 23 → 25 步，第一段 12 → 14 个写，闭式 1 + (2^14 − 1) + 1 + 1 + 1 + (2^4 − 1) = 16402（改之前 1 + 4095 + 3 + 15 = 4114）；
+    // mkfs 之后的各段不变。
     assert_eq!(
         (
             mkfs_operations.len(),
             sizes(mkfs_operations),
             closed_form(mkfs_operations)
         ),
-        (23, "12+1+1+1+4".to_string(), 4114)
+        (25, "14+1+1+1+4".to_string(), 16402)
     );
     assert_eq!(
         kinds(mkfs_operations),
-        "[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]"
+        "[zero_fill×10,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]"
     );
     assert_eq!(
         (
diff --git a/crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs b/crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs
new file mode 100644
index 0000000..75bc878
--- /dev/null
+++ b/crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs
@@ -0,0 +1,460 @@
+//! 全池没有可用的系统配置槽 0 时，池级 checker 与恢复找槽 1 的读法相同、结论相同（C651（全池没有可用的系统配置槽 0 时 checker
+//! 与实现找槽 1 的读法不同），第一轮判决 `research/prompts/m3-c651-r1-main-verification.md` 第四节三条与「用户定案」W1）：
+//! 在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、不越过根环起点 × 16384 − 4096），只收自证过、且自述的固定结构槽距
+//! 等于所在偏移的槽；几档都收得到时取世代号最大的，世代号相同取偏移小的；读到带读者不收之值的槽整池拒；
+//! mkfs 在写任何元数据之前把每块盘的 [0, 根环基址) 整段写 0，在用过的盘上换一个 io_min 重做 mkfs，旧池别的槽距上的槽 1 不再被择中。
+//! 盘都是内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`），两边读的是同一份 `MemoryPool`。
+
+use singlefs_checker::image::{
+    chosen_system_configurations, system_configuration_slot_readings, InvariantVerdict,
+    SystemConfigurationSlotReading, SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
+    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
+};
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
+};
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::checksum::wide_checksum_with_field_zeroed;
+use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
+use singlefs_core::mount::mount_writable;
+use singlefs_core::recovery::{
+    choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryFailure,
+    RecoveryOutcome,
+};
+use singlefs_core::root_ring::RootRingSlotsPerRegion;
+use singlefs_core::system_configuration::{
+    SystemConfiguration, SystemImmutableSizes,
+    INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT,
+    SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
+};
+use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+
+const DEVICE_BYTES: u64 = 4 << 30;
+const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-c651-r1";
+/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
+const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);
+/// io_min 512 的池：槽距 4096（D2（RAID 条带策略） 已定项 19 的式子）。
+const MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING: u32 = 512;
+/// io_min 8192 的池：槽距 8192，槽 1 在 8192 处（C651 的那一格）。
+const MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING: u32 = 8192;
+/// 系统配置槽里 incompat 位图第一个字节的偏移：magic 4 + 格式版本 2（与 `core_review_tree_table_duplicates_and_slot_one_search.rs` 同一处）。
+const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;
+/// journal 环起点字段写成第一版常量 1024 之外的一个槽号：自证得过、读者不收（D22（单元原子性怎么合成） 已定项 16 第 1 句）。
+const JOURNAL_RING_START_SLOT_THE_READER_REFUSES: u64 = 1025;
+
+fn geometry_with_minimum_input_output_bytes(
+    minimum_input_output_bytes: u32,
+) -> SystemImmutableSizes {
+    SystemImmutableSizes {
+        physical_block_size: 512,
+        minimum_input_output_bytes,
+        fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(
+            minimum_input_output_bytes,
+        ),
+        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
+        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
+    }
+}
+
+fn parameters_with_minimum_input_output_bytes(
+    minimum_input_output_bytes: u32,
+) -> MakeFilesystemParameters {
+    MakeFilesystemParameters {
+        filesystem_identifier: FILESYSTEM_IDENTIFIER,
+        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+        geometry: geometry_with_minimum_input_output_bytes(minimum_input_output_bytes),
+    }
+}
+
+fn two_sparse_devices() -> Vec<(DeviceIdentity, SparseBlockDevice)> {
+    [DeviceIdentity(0), DeviceIdentity(1)]
+        .into_iter()
+        .map(|identity| {
+            (
+                identity,
+                SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
+            )
+        })
+        .collect()
+}
+
+fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
+    MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: DEVICE_BYTES,
+    }
+}
+
+fn slot_bytes() -> usize {
+    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")
+}
+
+/// 按 `minimum_input_output_bytes` 算的槽距做一次 mkfs（没有文件），交回内存池。
+fn formatted_image_with_minimum_input_output_bytes(minimum_input_output_bytes: u32) -> MemoryPool {
+    let mut devices = two_sparse_devices();
+    make_filesystem(
+        &parameters_with_minimum_input_output_bytes(minimum_input_output_bytes),
+        &mut devices,
+    )
+    .expect("mkfs");
+    image_of(&devices)
+}
+
+/// 两块盘偏移 `offset` 那一个系统配置槽整槽抹成 0：读得出、magic 不对、自证不过。
+fn wipe_the_slot_at_on_every_device(image: &mut MemoryPool, offset: u64) {
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        image
+            .devices
+            .get_mut(&device)
+            .expect("池里有这块盘")
+            .write(DeviceOffsetInBytes(offset), &vec![0u8; slot_bytes()]);
+    }
+}
+
+fn slot_at(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> SystemConfiguration {
+    SystemConfiguration::parse_slot(
+        &PoolReader::read(image, device, DeviceOffsetInBytes(offset), slot_bytes())
+            .expect("槽读得到"),
+    )
+    .expect("这一槽自证得过")
+}
+
+fn plant_slot(
+    image: &mut MemoryPool,
+    device: DeviceIdentity,
+    offset: u64,
+    system_configuration: &SystemConfiguration,
+) {
+    image
+        .devices
+        .get_mut(&device)
+        .expect("池里有这块盘")
+        .write(DeviceOffsetInBytes(offset), &system_configuration.to_slot());
+}
+
+/// 整槽校验和重算：改过字节的槽 magic 与校验和两关照样过，拒不拒只看被改的那一格
+/// （与 `core_review_tree_table_duplicates_and_slot_one_search.rs` 同一个做法）。
+fn reseal_system_configuration_slot(slot: &mut [u8]) {
+    let digest =
+        wide_checksum_with_field_zeroed(slot, slot_bytes(), SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
+    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
+        .copy_from_slice(&digest);
+}
+
+/// 恢复择到的那一份：（盘、固定结构槽距、世代号）。
+fn chosen_by_recovery(image: &MemoryPool) -> (u32, u64, u64) {
+    let chosen = choose_system_configuration(image).expect("恢复择得到系统配置");
+    (
+        chosen.immutable.this_device.0,
+        u64::from(chosen.immutable.sizes.fixed_structure_slot_spacing),
+        chosen.quantities.slot_generation,
+    )
+}
+
+/// 池级 checker 每块盘择到的那一份：（盘、固定结构槽距、世代号），一块盘择不到就是 None。
+fn chosen_by_the_checker(image: &MemoryPool) -> Vec<(u32, Option<(u64, u64)>)> {
+    chosen_system_configurations(image)
+        .into_iter()
+        .map(|(device, chosen)| {
+            (
+                device,
+                chosen.map(|(view, geometry)| (geometry.slot_spacing, view.slot_generation)),
+            )
+        })
+        .collect()
+}
+
+fn violated_invariants(image: &MemoryPool) -> Vec<(&'static str, String)> {
+    check_pool_image(image)
+        .into_iter()
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+        })
+        .collect()
+}
+
+fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(name, _)| *name == invariant)
+        .map(|(_, verdict)| verdict)
+        .expect("池级 checker 报这条不变量")
+}
+
+/// C651 那一格：io_min 8192 的池（槽距 8192），两块盘的槽 0 都抹掉。恢复逐档择到盘 0 在 8192 处的槽 1；
+/// checker 也逐档读，两块盘都择到 8192 处 mkfs 写下的世代号 1，I-8.5（tail 槽有效集非空） 成立、一条不变量都不红。
+/// checker 按最小槽距 4096 读时 4096 处是 mkfs 清过的 0，两块盘都择不到、I-8.5 判红——与恢复结论相反。
+#[test]
+fn an_io_min_8192_pool_whose_slot_zeros_are_wiped_has_the_checker_and_recovery_both_choose_slot_one_at_8192(
+) {
+    let spacing = u64::from(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING);
+    let mut image = formatted_image_with_minimum_input_output_bytes(
+        MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING,
+    );
+    wipe_the_slot_at_on_every_device(&mut image, 0);
+    assert_eq!(
+        chosen_by_recovery(&image),
+        (0, spacing, 1),
+        "恢复择到盘 0 在 8192 处的槽 1（mkfs 写下的世代号 1）"
+    );
+    assert_eq!(
+        recover(&image, JournalPolicy::Consult).outcome,
+        RecoveryOutcome::NoFile {
+            root: (InstanceGeneration(0), CheckpointTxg(0)),
+        },
+        "恢复走到 mkfs 的第 0 代根"
+    );
+    assert_eq!(
+        chosen_by_the_checker(&image),
+        vec![(0, Some((spacing, 1))), (1, Some((spacing, 1)))],
+        "checker 两块盘都择到 8192 处世代号 1 的槽 1"
+    );
+    assert_eq!(
+        verdict_of(
+            &image,
+            SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
+        ),
+        InvariantVerdict::Holds,
+        "两块盘的槽 1 自证过：I-8.5 成立"
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        Vec::<(&str, String)>::new(),
+        "恢复挂得上的池，checker 一条不变量都不红"
+    );
+}
+
+/// 只收自述的固定结构槽距等于所在偏移的槽（第四节第 1 条）：io_min 8192 的池，4096 处放一份自证得过、世代号 9、记着槽距 8192 的槽
+/// （它不在自己记的槽距上），两块盘的槽 0 都抹掉：恢复与 checker 都不收它，都择 8192 处世代号 1 的那一份。
+#[test]
+fn a_self_verified_slot_whose_recorded_spacing_is_not_its_own_offset_is_taken_by_neither_the_checker_nor_recovery(
+) {
+    let spacing = u64::from(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING);
+    let mut image = formatted_image_with_minimum_input_output_bytes(
+        MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING,
+    );
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        let mut misplaced = slot_at(&image, device, spacing);
+        misplaced.quantities.slot_generation = 9;
+        plant_slot(&mut image, device, 4096, &misplaced);
+    }
+    wipe_the_slot_at_on_every_device(&mut image, 0);
+    assert_eq!(
+        chosen_by_recovery(&image),
+        (0, spacing, 1),
+        "4096 处那一份记的槽距是 8192，恢复不收；择到 8192 处世代号 1 的"
+    );
+    assert_eq!(
+        chosen_by_the_checker(&image),
+        vec![(0, Some((spacing, 1))), (1, Some((spacing, 1)))],
+        "checker 同样不收 4096 处那一份"
+    );
+}
+
+/// 几档都收得到时取世代号最大的（第四节第 2 条）：槽距 4096 的池，8192 处放一份同 fsid、记着槽距 8192、世代号 5 的槽，
+/// 两块盘的槽 0 都抹掉。4096 处世代号 1、8192 处世代号 5 都自证过、记的槽距都等于所在偏移：恢复与 checker 都择世代号 5 的那一份。
+#[test]
+fn slots_found_at_several_spacings_resolve_to_the_highest_generation_in_both_the_checker_and_recovery(
+) {
+    let mut image = formatted_image_with_minimum_input_output_bytes(
+        MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING,
+    );
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        let mut at_8192 = slot_at(&image, device, 4096);
+        at_8192.immutable.sizes.fixed_structure_slot_spacing = 8192;
+        at_8192.quantities.slot_generation = 5;
+        plant_slot(&mut image, device, 8192, &at_8192);
+    }
+    wipe_the_slot_at_on_every_device(&mut image, 0);
+    assert_eq!(
+        chosen_by_recovery(&image),
+        (0, 8192, 5),
+        "恢复择到 8192 处世代号 5 的那一份"
+    );
+    assert_eq!(
+        chosen_by_the_checker(&image),
+        vec![(0, Some((8192, 5))), (1, Some((8192, 5)))],
+        "checker 与恢复择同一份：8192 处世代号 5"
+    );
+}
+
+/// 世代号相同时取偏移小的（第四节第 2 条）：槽距 4096 的池，8192 处放一份同 fsid、记着槽距 8192、世代号同为 1 的槽，
+/// 两块盘的槽 0 都抹掉：恢复与 checker 都择 4096 处本池的那一份。
+#[test]
+fn slots_of_equal_generation_found_at_several_spacings_resolve_to_the_smaller_offset_in_both_the_checker_and_recovery(
+) {
+    let mut image = formatted_image_with_minimum_input_output_bytes(
+        MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING,
+    );
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        let mut at_8192 = slot_at(&image, device, 4096);
+        at_8192.immutable.sizes.fixed_structure_slot_spacing = 8192;
+        plant_slot(&mut image, device, 8192, &at_8192);
+    }
+    wipe_the_slot_at_on_every_device(&mut image, 0);
+    assert_eq!(
+        chosen_by_recovery(&image),
+        (0, 4096, 1),
+        "世代号都是 1：恢复取偏移小的 4096 那一份"
+    );
+    assert_eq!(
+        chosen_by_the_checker(&image),
+        vec![(0, Some((4096, 1))), (1, Some((4096, 1)))],
+        "checker 与恢复择同一份：4096 处世代号 1"
+    );
+}
+
+/// 逐档读到一份带读者不收之值的槽（8192 处，journal 环起点写成 1025、整槽校验和重算过）：恢复整池拒、点名盘 0；
+/// checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、整池不作保（D22（单元原子性怎么合成） 已定项 16 第 5 句「越界即拒」）。
+/// 4096 处本池的槽 1 照样自证得过、可择，两边都不因为它而跳过那一份。
+#[test]
+fn a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
+) {
+    let mut image = formatted_image_with_minimum_input_output_bytes(
+        MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING,
+    );
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        let mut refused = slot_at(&image, device, 4096);
+        refused.immutable.journal_ring_start_slot =
+            SlotNumber(JOURNAL_RING_START_SLOT_THE_READER_REFUSES);
+        plant_slot(&mut image, device, 8192, &refused);
+    }
+    wipe_the_slot_at_on_every_device(&mut image, 0);
+    assert!(
+        matches!(
+            choose_system_configuration(&image),
+            Err(RecoveryFailure::SystemConfigurationValueRefused {
+                device: DeviceIdentity(0),
+                ..
+            })
+        ),
+        "恢复整池拒、点名盘 0：{:?}",
+        choose_system_configuration(&image).err()
+    );
+    let readings = system_configuration_slot_readings(&image);
+    assert!(
+        readings.iter().all(|(_, slots)| matches!(
+            slots[1],
+            SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
+                offset_in_bytes: 8192,
+                ..
+            }
+        )),
+        "checker 两块盘的槽 1 都读成 8192 处那份带不收之值的槽：{readings:?}"
+    );
+    assert!(
+        matches!(
+            verdict_of(
+                &image,
+                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
+            ),
+            InvariantVerdict::Violated(_)
+        ),
+        "checker 报 I-7.13 违例"
+    );
+}
+
+/// 逐档读到的自证过的槽都不可择（4096 处本池的槽 1 也抹掉，8192 处放一份 incompat 位图不认识的槽）：恢复报 incompat 位图不认识、
+/// 点名盘 0；checker 两块盘都择不到，而那一槽自证过，I-8.5（tail 槽有效集非空） 照样成立。
+#[test]
+fn only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery(
+) {
+    let mut image = formatted_image_with_minimum_input_output_bytes(
+        MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING,
+    );
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        let mut not_choosable_slot = slot_at(&image, device, 4096).to_slot();
+        not_choosable_slot[FIRST_INCOMPAT_BYTE_OFFSET] =
+            INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT;
+        reseal_system_configuration_slot(&mut not_choosable_slot);
+        image
+            .devices
+            .get_mut(&device)
+            .expect("池里有这块盘")
+            .write(DeviceOffsetInBytes(8192), &not_choosable_slot);
+    }
+    wipe_the_slot_at_on_every_device(&mut image, 0);
+    wipe_the_slot_at_on_every_device(&mut image, 4096);
+    assert!(
+        matches!(
+            choose_system_configuration(&image),
+            Err(
+                RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized {
+                    first_device_carrying_them: DeviceIdentity(0),
+                    ..
+                }
+            )
+        ),
+        "恢复报 incompat 位图不认识、点名盘 0：{:?}",
+        choose_system_configuration(&image).err()
+    );
+    assert_eq!(
+        chosen_by_the_checker(&image),
+        vec![(0, None), (1, None)],
+        "checker 两块盘都择不到"
+    );
+    assert_eq!(
+        verdict_of(
+            &image,
+            SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
+        ),
+        InvariantVerdict::Holds,
+        "8192 处那一槽自证过：I-8.5 成立"
+    );
+}
+
+/// mkfs 清 [0, 根环基址)（W1）：一块盘先 mkfs 一次槽距 4096 的池、可写挂载一次（取号与两次发布各写一次系统配置，槽 1 世代号到 3），
+/// 再拿同一个 fsid mkfs 一次 io_min 8192 的池，之后两块盘的槽 0 都抹掉。旧池 4096 处的槽 1 记着槽距 4096、世代号比新池的 1 大，
+/// mkfs 不清它时逐档读的两个读者都会择中它；清了之后 4096 处是 0，恢复与 checker 都择到新池 8192 处的槽 1。
+#[test]
+fn remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader(
+) {
+    let mut devices = two_sparse_devices();
+    let earlier_parameters =
+        parameters_with_minimum_input_output_bytes(MINIMUM_INPUT_OUTPUT_BYTES_OF_A_4096_SPACING);
+    make_filesystem(&earlier_parameters, &mut devices).expect("先 mkfs 一次槽距 4096 的池");
+    mount_writable(&earlier_parameters, &mut devices).expect("旧池可写挂载一次");
+    let earlier_image = image_of(&devices);
+    let earlier_slot_one_generation = slot_at(&earlier_image, DeviceIdentity(0), 4096)
+        .quantities
+        .slot_generation;
+    assert!(
+        earlier_slot_one_generation > 1,
+        "旧池 4096 处的槽 1 写过系统配置，世代号 {earlier_slot_one_generation} 比新池 mkfs 写下的 1 大"
+    );
+
+    make_filesystem(
+        &parameters_with_minimum_input_output_bytes(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING),
+        &mut devices,
+    )
+    .expect("同一个 fsid 再 mkfs 一次 io_min 8192 的池");
+    let mut image = image_of(&devices);
+    wipe_the_slot_at_on_every_device(&mut image, 0);
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        let bytes_where_the_earlier_slot_one_was =
+            PoolReader::read(&image, device, DeviceOffsetInBytes(4096), slot_bytes())
+                .expect("4096 处读得到");
+        assert!(
+            bytes_where_the_earlier_slot_one_was
+                .iter()
+                .all(|byte| *byte == 0),
+            "盘 {device:?}：mkfs 之后旧池槽 1 所在的 4096 处整槽是 0"
+        );
+    }
+    assert_eq!(
+        chosen_by_recovery(&image),
+        (0, 8192, 1),
+        "恢复择到新池 8192 处世代号 1 的槽 1，不是旧池 4096 处世代号 {earlier_slot_one_generation} 的那一份"
+    );
+    assert_eq!(
+        chosen_by_the_checker(&image),
+        vec![(0, Some((8192, 1))), (1, Some((8192, 1)))],
+        "checker 与恢复择同一份：新池 8192 处世代号 1"
+    );
+}
```

### mutations-append.tsv

```
checker ③ 全池没有可用的系统配置槽 0 时槽 1 取常量 4096（C651 第一轮判决 V1-常量，出局的读法）	crates/singlefs-checker/src/image.rs	None => slot_one_read_at_every_slot_spacing_the_format_allows(reader, device),	None => read_system_configuration_slot(reader, device, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES),	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- an_io_min_8192_pool_whose_slot_zeros_are_wiped slots_of_equal_generation_found_at_several_spacings	an_io_min_8192_pool_whose_slot_zeros_are_wiped_has_the_checker_and_recovery_both_choose_slot_one_at_8192
checker ③ 逐档读槽 1 时世代号相同取偏移大的（C651 第一轮判决 V2 反过来）	crates/singlefs-checker/src/image.rs	view.slot_generation > accepted.slot_generation	view.slot_generation >= accepted.slot_generation	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- an_io_min_8192_pool_whose_slot_zeros_are_wiped slots_of_equal_generation_found_at_several_spacings	slots_of_equal_generation_found_at_several_spacings_resolve_to_the_smaller_offset_in_both_the_checker_and_recovery
mkfs 不清 [0, 根环基址)（C651 第一轮判决用户定案 W1 撤回）	crates/singlefs-core/src/make_filesystem.rs	        device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;	        let _not_cleared = root_ring_base;	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader	remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader
checker ③ 逐档读槽 1 时几档都收得到取偏移最小的那一份、不比世代号	crates/singlefs-checker/src/image.rs	                if highest_generation_accepted\n                    .as_ref()\n                    .is_none_or(|(accepted, _)| view.slot_generation > accepted.slot_generation)	                if highest_generation_accepted.is_none()	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- slots_found_at_several_spacings_resolve_to_the_highest_generation_in_both_the_checker_and_recovery	slots_found_at_several_spacings_resolve_to_the_highest_generation_in_both_the_checker_and_recovery
checker ③ 逐档读槽 1 时不核自述的槽距等于所在偏移	crates/singlefs-checker/src/image.rs	                if geometry.slot_spacing != candidate_spacing {\n                    continue;\n                }\n	                let _spacing_not_checked = candidate_spacing;\n	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- a_self_verified_slot_whose_recorded_spacing_is_not_its_own_offset_is_taken_by_neither_the_checker_nor_recovery	a_self_verified_slot_whose_recorded_spacing_is_not_its_own_offset_is_taken_by_neither_the_checker_nor_recovery
checker ③ 逐档读到带读者不收之值的槽时跳过它、不整池拒	crates/singlefs-checker/src/image.rs	            refused @ SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. } => {\n                return refused;\n            }	            SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. } => {}	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery	a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery
checker ③ 逐档读到的自证过的槽都不可择时报成自证不过	crates/singlefs-checker/src/image.rs	        (None, true) => SystemConfigurationSlotReading::SelfVerifiedButNotChoosable,	        (None, true) => SystemConfigurationSlotReading::NotSelfVerified,	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery	only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery
```

## 实现员补丁 m3-formal-11-10（`/tmp/claude-1000/m3-formal-11-10/patch/`）

```diff
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -1794,11 +1794,22 @@
 }
 
 /// 读出来的根里 (checkpoint_txg, 实例代号) 最大的那一条（与 [`choose_root`] 同一个择法）；一条都没有时 `None`。
+/// 择新键完全相同的不止一条时取读根环次序里最前的那条，与 `singlefs_replay_prefix::choose_root_slot` 的平局取法相同
+/// （只有后面那条严格更新才换），F 生效值按表判有效根用的那张表才是恢复择的那条根指着的。
+/// 入参须按读根环次序排（`every_root_ring_slot`：读根环的每一处都按这个次序走）。
 fn newest_root_among(roots_with_ring_slots: &[(RootRingSlot, RootRecord)]) -> Option<RootRecord> {
     roots_with_ring_slots
         .iter()
         .map(|(_, root)| *root)
-        .max_by_key(|root| (root.checkpoint_txg, root.instance))
+        .reduce(|newest_so_far, candidate| {
+            if (candidate.checkpoint_txg, candidate.instance)
+                > (newest_so_far.checkpoint_txg, newest_so_far.instance)
+            {
+                candidate
+            } else {
+                newest_so_far
+            }
+        })
 }
 
 /// 生效的回退下界 F 的算法本身（[`effective_rollback_floor`] 的文档），根环已经读过、判「有效根」的那张表已经读过（读不出时 `None`：
diff --git a/crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs b/crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs
@@ -0,0 +1,272 @@
+//! 择根胶水（`recovery::choose_root` 把根环每个槽摊成核 `singlefs_replay_prefix::choose_root_slot` 的择新键、照核交回的下标取回那条根）
+//! 与它下游两处，在真池的根环上：
+//! - 择新键两项的次序：D22（单元原子性怎么合成） 已定项 7 txg 为主、实例代号只破平局。胶水把两项摊反时，择到实例代号高、txg 小的那条
+//!   （代码轮判决 `research/prompts/m3-formal-code-r1-main-verification.md` 第二节 K1-b，攻方补的用例照本仓命名改写）。
+//! - 恢复（`recovery::recover`）拿所选槽里那条根调重放，不是根环里别的自证过的根（同一格「胶水拿所选根调重放」）。
+//! - 择新键完全相同的两条根：择根取读根环次序里前面那条（已定项 24、核的平局取法）；算 F 生效值时判有效根用的那张实例表，
+//!   也是前面那条根指着的（同一判决 K1-d：`newest_root_among` 文档说与 `choose_root` 同一个择法）。
+//! 池是只做过 mkfs 的池，根照 mkfs 那条根改几项、重新编码写进 mkfs 那条根之后空着的槽（择根只读根环，不看别的结构）。
+mod common;
+
+use common::format_pool;
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::recovery::{
+    choose_root, choose_system_configuration,
+    effective_rollback_floor_rereading_the_newest_instance_table_once, every_root_ring_slot,
+    instance_table_chain_of_root, read_root_ring_slot, recover, JournalPolicy, RecoveryOutcome,
+    RootRingSlotReading,
+};
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::root_ring::{slot_offset, RootRingSlot};
+use singlefs_core::system_configuration::SystemConfiguration;
+use singlefs_harness::memory_pool::MemoryPool;
+
+/// txg 大的那条根（排在前面的槽里）的实例代号：比另一条低。
+const INSTANCE_OF_THE_NEWER_CHECKPOINT_TXG_ROOT: InstanceGeneration = InstanceGeneration(1);
+/// 实例代号高的那条根（排在后面的槽里）的实例代号。
+const INSTANCE_OF_THE_HIGHER_INSTANCE_ROOT: InstanceGeneration = InstanceGeneration(2);
+/// txg 大的那条根比 mkfs 那条根新几个 txg。
+const CHECKPOINT_TXGS_OF_THE_NEWER_CHECKPOINT_TXG_ROOT_ABOVE_THE_MKFS_ROOT: u64 = 2;
+/// 实例代号高的那条根比 mkfs 那条根新几个 txg（比上一条少一个）。
+const CHECKPOINT_TXGS_OF_THE_HIGHER_INSTANCE_ROOT_ABOVE_THE_MKFS_ROOT: u64 = 1;
+
+fn ring_slot_holds_a_self_verified_root(
+    image: &MemoryPool,
+    system_configuration: &SystemConfiguration,
+    ring_slot: RootRingSlot,
+) -> bool {
+    match read_root_ring_slot(
+        image,
+        &system_configuration.immutable.region_devices,
+        &system_configuration.immutable.sizes,
+        &system_configuration.immutable.filesystem_identifier,
+        ring_slot,
+    ) {
+        RootRingSlotReading::SelfVerified(_) => true,
+        RootRingSlotReading::Bad(_) => false,
+    }
+}
+
+fn write_root_into_ring_slot(
+    image: &mut MemoryPool,
+    system_configuration: &SystemConfiguration,
+    ring_slot: RootRingSlot,
+    root: &RootRecord,
+) {
+    let immutable = &system_configuration.immutable;
+    let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
+    let root_slot_bytes = usize::try_from(immutable.sizes.physical_block_size).expect("根槽宽");
+    image.devices.get_mut(&device).expect("池里有这块盘").write(
+        slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing),
+        &root.to_slot(root_slot_bytes),
+    );
+}
+
+/// 只做过 mkfs 的池：mkfs 那条根、读根环次序里它之后第一个与最后一个空槽。
+struct MkfsPoolWithTwoEmptySlotsAfterItsRoot {
+    image: MemoryPool,
+    system_configuration: SystemConfiguration,
+    mkfs_root: RootRecord,
+    earlier_empty_slot: RootRingSlot,
+    later_empty_slot: RootRingSlot,
+}
+
+fn mkfs_pool_with_two_empty_slots_after_its_root(
+    tag: &str,
+) -> MkfsPoolWithTwoEmptySlotsAfterItsRoot {
+    let image = format_pool(tag).memory_pool();
+    let system_configuration =
+        choose_system_configuration(&image).expect("mkfs 之后择得出系统配置");
+    let mkfs_root = choose_root(&image, &system_configuration).expect("mkfs 写了一条根");
+    let ring_slots = every_root_ring_slot(&system_configuration.immutable.sizes);
+    let mkfs_root_slot_position = ring_slots
+        .iter()
+        .position(|ring_slot| {
+            ring_slot_holds_a_self_verified_root(&image, &system_configuration, *ring_slot)
+        })
+        .expect("根环里有 mkfs 那条根");
+    let empty_slots_after_the_mkfs_root: Vec<RootRingSlot> = ring_slots
+        [mkfs_root_slot_position + 1..]
+        .iter()
+        .copied()
+        .filter(|ring_slot| {
+            !ring_slot_holds_a_self_verified_root(&image, &system_configuration, *ring_slot)
+        })
+        .collect();
+    let earlier_empty_slot = *empty_slots_after_the_mkfs_root
+        .first()
+        .expect("mkfs 那条根之后有空槽");
+    let later_empty_slot = *empty_slots_after_the_mkfs_root
+        .last()
+        .expect("mkfs 那条根之后有空槽");
+    assert!(
+        earlier_empty_slot < later_empty_slot,
+        "要两个不同的空槽，一前一后"
+    );
+    MkfsPoolWithTwoEmptySlotsAfterItsRoot {
+        image,
+        system_configuration,
+        mkfs_root,
+        earlier_empty_slot,
+        later_empty_slot,
+    }
+}
+
+/// 前一个空槽写 txg 大、实例代号低的根，后一个空槽写实例代号高、txg 小的根；两条除这两项之外照抄 mkfs 那条根。
+struct PoolWhereTheNewerCheckpointTxgSitsInAnEarlierSlot {
+    image: MemoryPool,
+    system_configuration: SystemConfiguration,
+    mkfs_root: RootRecord,
+    newer_checkpoint_txg_root: RootRecord,
+    higher_instance_root: RootRecord,
+}
+
+fn pool_where_the_newer_checkpoint_txg_sits_in_an_earlier_slot(
+    tag: &str,
+) -> PoolWhereTheNewerCheckpointTxgSitsInAnEarlierSlot {
+    let MkfsPoolWithTwoEmptySlotsAfterItsRoot {
+        mut image,
+        system_configuration,
+        mkfs_root,
+        earlier_empty_slot,
+        later_empty_slot,
+    } = mkfs_pool_with_two_empty_slots_after_its_root(tag);
+    let mut newer_checkpoint_txg_root = mkfs_root;
+    newer_checkpoint_txg_root.instance = INSTANCE_OF_THE_NEWER_CHECKPOINT_TXG_ROOT;
+    newer_checkpoint_txg_root.checkpoint_txg = CheckpointTxg(
+        mkfs_root.checkpoint_txg.0
+            + CHECKPOINT_TXGS_OF_THE_NEWER_CHECKPOINT_TXG_ROOT_ABOVE_THE_MKFS_ROOT,
+    );
+    let mut higher_instance_root = mkfs_root;
+    higher_instance_root.instance = INSTANCE_OF_THE_HIGHER_INSTANCE_ROOT;
+    higher_instance_root.checkpoint_txg = CheckpointTxg(
+        mkfs_root.checkpoint_txg.0
+            + CHECKPOINT_TXGS_OF_THE_HIGHER_INSTANCE_ROOT_ABOVE_THE_MKFS_ROOT,
+    );
+    write_root_into_ring_slot(
+        &mut image,
+        &system_configuration,
+        earlier_empty_slot,
+        &newer_checkpoint_txg_root,
+    );
+    write_root_into_ring_slot(
+        &mut image,
+        &system_configuration,
+        later_empty_slot,
+        &higher_instance_root,
+    );
+    PoolWhereTheNewerCheckpointTxgSitsInAnEarlierSlot {
+        image,
+        system_configuration,
+        mkfs_root,
+        newer_checkpoint_txg_root,
+        higher_instance_root,
+    }
+}
+
+/// 胶水把择新键两项摊反（实例代号为主）时择到后一个槽里实例代号高、txg 小的那条；txg 为主择前一个槽里 txg 大的那条。
+#[test]
+fn a_later_checkpoint_txg_in_an_earlier_slot_beats_a_higher_instance_through_choose_root() {
+    let pool = pool_where_the_newer_checkpoint_txg_sits_in_an_earlier_slot(
+        "root-choice-glue-checkpoint-txg-before-instance",
+    );
+    assert!(
+        pool.higher_instance_root.instance > pool.newer_checkpoint_txg_root.instance
+            && pool.higher_instance_root.checkpoint_txg
+                < pool.newer_checkpoint_txg_root.checkpoint_txg,
+        "两条根的两项各大一项：只有 txg 为主才择前一个槽那条"
+    );
+
+    let chosen = choose_root(&pool.image, &pool.system_configuration).expect("根环里有自证过的根");
+
+    assert_eq!(
+        chosen, pool.newer_checkpoint_txg_root,
+        "txg 为主（D22 已定项 7）：txg 大的实例 1 那条赢过实例代号高、txg 小的实例 2 那条（胶水摊反两项时择到实例 2 那条）"
+    );
+}
+
+/// 恢复拿所选槽里那条根（txg 大的那条）调重放：施加之后实际走的根就是它（mkfs 池里没有它之上的 journal 记录）；
+/// 拿根环里别的自证过的根（例如读根环次序里第一条，mkfs 那条）去调，走的根就成了那一条。
+#[test]
+fn recover_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root() {
+    let pool = pool_where_the_newer_checkpoint_txg_sits_in_an_earlier_slot(
+        "recover-replays-from-the-chosen-slot",
+    );
+    let chosen_key = (
+        pool.newer_checkpoint_txg_root.instance,
+        pool.newer_checkpoint_txg_root.checkpoint_txg,
+    );
+    assert_ne!(
+        (pool.mkfs_root.instance, pool.mkfs_root.checkpoint_txg),
+        chosen_key,
+        "mkfs 那条根与所选根是两条不同的根"
+    );
+
+    let report = recover(&pool.image, JournalPolicy::Consult);
+
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::NoFile { root: chosen_key },
+        "择到的是前一个槽里 txg 大的那条，它下面没有文件"
+    );
+    assert_eq!(
+        report.effective_root,
+        Some(chosen_key),
+        "施加之后实际走的根是所选槽里那条（水位之上没有记录，原样走它），不是根环里别的自证过的根"
+    );
+}
+
+/// 择新键（txg, 实例代号）完全相同的两条根：前面是 mkfs 那条，后面那条照抄它、只把实例表指针的两条位置条目的校验和改掉（表读不出）。
+/// 择根取前面那条（只有严格更新才换）；F 生效值按表判有效根，读的也是前面那条指着的那张表，读得出，算出 mkfs 那一版的 F。
+/// 取了后面那条就读不出表，重读一次仍读不出，报错。
+#[test]
+fn of_two_roots_with_the_same_choice_key_the_effective_floor_reads_the_instance_table_of_the_earlier_slot_as_choose_root_does(
+) {
+    let MkfsPoolWithTwoEmptySlotsAfterItsRoot {
+        mut image,
+        system_configuration,
+        mkfs_root,
+        earlier_empty_slot: _,
+        later_empty_slot,
+    } = mkfs_pool_with_two_empty_slots_after_its_root("tied-choice-key-effective-floor");
+    let mut same_key_root_with_an_unreadable_instance_table = mkfs_root;
+    for location in &mut same_key_root_with_an_unreadable_instance_table
+        .instance_table
+        .locations
+    {
+        location.unit_checksum = !location.unit_checksum;
+    }
+    write_root_into_ring_slot(
+        &mut image,
+        &system_configuration,
+        later_empty_slot,
+        &same_key_root_with_an_unreadable_instance_table,
+    );
+    assert!(
+        instance_table_chain_of_root(&image, &mkfs_root).is_ok()
+            && instance_table_chain_of_root(
+                &image,
+                &same_key_root_with_an_unreadable_instance_table
+            )
+            .is_err(),
+        "前面那条的实例表读得出，后面那条的读不出：两条择新键相同、指着的表不同"
+    );
+
+    let chosen = choose_root(&image, &system_configuration).expect("根环里有自证过的根");
+    let effective_floor = effective_rollback_floor_rereading_the_newest_instance_table_once(
+        &image,
+        &system_configuration.immutable.region_devices,
+        &system_configuration.immutable.sizes,
+        &system_configuration.immutable.filesystem_identifier,
+    );
+
+    assert_eq!(
+        chosen, mkfs_root,
+        "择新键完全相同取读根环次序里前面那条（D22 已定项 24、核的平局取法）"
+    );
+    assert_eq!(
+        effective_floor,
+        Ok(mkfs_root.rollback_floor),
+        "F 生效值判有效根读的是择根择的那条根指着的表（前面那条，读得出），不是后面那条（读不出，重读一次仍读不出就报错）"
+    );
+}
```

### mutations-append.tsv

```
择根胶水把择新键两项对调（代码轮 m3-formal-code-r1 K1-b，攻方 G1）：核看到的 txg 是实例代号、实例代号是 txg，实例代号高、txg 小的根赢过 txg 大的	crates/singlefs-core/src/recovery.rs	                checkpoint_txg: PrefixKernelCheckpointTxg(root.checkpoint_txg.0),\n                instance: PrefixKernelInstanceGeneration(root.instance.0),	                checkpoint_txg: PrefixKernelCheckpointTxg(u64::from(root.instance.0)),\n                instance: PrefixKernelInstanceGeneration(u32::try_from(root.checkpoint_txg.0).unwrap_or(u32::MAX)),	-p singlefs-harness --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance	a_later_checkpoint_txg_in_an_earlier_slot_beats_a_higher_instance_through_choose_root
恢复胶水拿所选槽之外的根调重放（代码轮 m3-formal-code-r1 K1-b）：recover 把读根环次序里第一条自证过的根交给 replay_journal，不是 choose_root 择的那条	crates/singlefs-core/src/recovery.rs	                &root,\n                system_configuration.immutable.sizes.journal_ring_bytes,	                &readable_roots_with_ring_slots(reader, &system_configuration.immutable.region_devices, &system_configuration.immutable.sizes, &system_configuration.immutable.filesystem_identifier)[0].1,\n                system_configuration.immutable.sizes.journal_ring_bytes,	-p singlefs-harness --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance	recover_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root
F 生效值择最新根平局取后者（代码轮 m3-formal-code-r1 K1-d）：newest_root_among 择新键完全相同时换成读根环次序里后面那条，与 choose_root 取前面那条不同，判有效根读了另一张实例表	crates/singlefs-core/src/recovery.rs	            if (candidate.checkpoint_txg, candidate.instance)\n                > (newest_so_far.checkpoint_txg, newest_so_far.instance)	            if (candidate.checkpoint_txg, candidate.instance)\n                >= (newest_so_far.checkpoint_txg, newest_so_far.instance)	-p singlefs-harness --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance	of_two_roots_with_the_same_choice_key_the_effective_floor_reads_the_instance_table_of_the_earlier_slot_as_choose_root_does
由记录重建的根不照抄所选根的文件系统标识（代码轮 m3-formal-code-r1 K3，R3-1「其余字段等于所选根」）：改写全零	crates/singlefs-core/src/recovery.rs	                filesystem_identifier: root.filesystem_identifier,	                filesystem_identifier: [0_u8; 16],	-p singlefs-harness --test rebuilt_root_keeps_the_chosen_roots_fields_except_the_unmount_marker	a_root_rebuilt_from_the_second_unmount_publish_record_keeps_the_chosen_roots_other_fields_and_is_not_marked_by_the_unmount_sequence
由记录重建的根不照抄所选根的实例表指针（代码轮 m3-formal-code-r1 K3，R3-1「其余字段等于所选根」）：改写全零指针	crates/singlefs-core/src/recovery.rs	                instance_table: root.instance_table,	                instance_table: NodePointer::empty_root(),	-p singlefs-harness --test rebuilt_root_keeps_the_chosen_roots_fields_except_the_unmount_marker	a_root_rebuilt_from_the_second_unmount_publish_record_keeps_the_chosen_roots_other_fields_and_is_not_marked_by_the_unmount_sequence
由记录重建的根不照抄所选根的分配记录树根（代码轮 m3-formal-code-r1 K3，R3-1「其余字段等于所选根」）：改抄末条记录新根段里的树表指针	crates/singlefs-core/src/recovery.rs	                allocation_record_tree_root: root.allocation_record_tree_root,	                allocation_record_tree_root: record.new_tree_table,	-p singlefs-harness --test rebuilt_root_keeps_the_chosen_roots_fields_except_the_unmount_marker	a_root_rebuilt_from_the_second_unmount_publish_record_keeps_the_chosen_roots_other_fields_and_is_not_marked_by_the_unmount_sequence
```

## 实现员补丁 m3-formal-11-11（`/tmp/claude-1000/m3-formal-11-11/patch/`）

```diff
diff --git a/crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs b/crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs
--- a/crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs
+++ b/crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs
@@ -1,25 +1,31 @@
 //! 择根胶水（`recovery::choose_root` 把根环每个槽摊成核 `singlefs_replay_prefix::choose_root_slot` 的择新键、照核交回的下标取回那条根）
-//! 与它下游两处，在真池的根环上：
+//! 与它的下游，在真池的根环上：
 //! - 择新键两项的次序：D22（单元原子性怎么合成） 已定项 7 txg 为主、实例代号只破平局。胶水把两项摊反时，择到实例代号高、txg 小的那条
 //!   （代码轮判决 `research/prompts/m3-formal-code-r1-main-verification.md` 第二节 K1-b，攻方补的用例照本仓命名改写）。
-//! - 恢复（`recovery::recover`）拿所选槽里那条根调重放，不是根环里别的自证过的根（同一格「胶水拿所选根调重放」）。
+//! - 恢复（`recovery::recover`）、只读挂载（`mounted_read::mount_read_only`）与可写挂载的读阶段（`mount::mount_writable`）
+//!   拿所选槽里那条根调重放，不是根环里别的自证过的根（同一格「胶水拿所选根调重放」）。
 //! - 择新键完全相同的两条根：择根取读根环次序里前面那条（已定项 24、核的平局取法）；算 F 生效值时判有效根用的那张实例表，
 //!   也是前面那条根指着的（同一判决 K1-d：`newest_root_among` 文档说与 `choose_root` 同一个择法）。
-//! 池是只做过 mkfs 的池，根照 mkfs 那条根改几项、重新编码写进 mkfs 那条根之后空着的槽（择根只读根环，不看别的结构）。
+//!
+//! 池是只做过 mkfs 的池，根照 mkfs 那条根改几项、重新编码写进 mkfs 那条根之后空着的槽（择根只读根环，不看别的结构）；
+//! 只读挂载那一格要沿根读中央映射树与树表，用建过新池新建文件的池，根照新池新建文件那条根改。
 mod common;
 
-use common::format_pool;
+use common::{build_pool, format_pool, parameters};
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::mount::mount_writable;
+use singlefs_core::mounted_read::mount_read_only;
 use singlefs_core::recovery::{
     choose_root, choose_system_configuration,
     effective_rollback_floor_rereading_the_newest_instance_table_once, every_root_ring_slot,
-    instance_table_chain_of_root, read_root_ring_slot, recover, JournalPolicy, RecoveryOutcome,
-    RootRingSlotReading,
+    instance_table_chain_of_root, read_root_ring_slot, readable_roots_with_ring_slots, recover,
+    JournalPolicy, RecoveryOutcome, RootRingSlotReading,
 };
 use singlefs_core::root_record::RootRecord;
 use singlefs_core::root_ring::{slot_offset, RootRingSlot};
 use singlefs_core::system_configuration::SystemConfiguration;
-use singlefs_harness::memory_pool::MemoryPool;
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
 
 /// txg 大的那条根（排在前面的槽里）的实例代号：比另一条低。
 const INSTANCE_OF_THE_NEWER_CHECKPOINT_TXG_ROOT: InstanceGeneration = InstanceGeneration(1);
@@ -216,6 +222,171 @@
     );
 }
 
+/// 前一个空槽写一条只比 mkfs 那条根新一个 txg 的根，别的照抄 mkfs 那条（实例代号也照抄）。读根环次序里第一条自证过的根仍是 mkfs 那条。
+struct PoolWithARootOneCheckpointTxgNewerThanTheMkfsRoot {
+    image: MemoryPool,
+    mkfs_root: RootRecord,
+    newer_root: RootRecord,
+}
+
+fn pool_with_a_root_one_checkpoint_txg_newer_than_the_mkfs_root(
+    tag: &str,
+) -> PoolWithARootOneCheckpointTxgNewerThanTheMkfsRoot {
+    let MkfsPoolWithTwoEmptySlotsAfterItsRoot {
+        mut image,
+        system_configuration,
+        mkfs_root,
+        earlier_empty_slot,
+        later_empty_slot: _,
+    } = mkfs_pool_with_two_empty_slots_after_its_root(tag);
+    let mut newer_root = mkfs_root;
+    newer_root.checkpoint_txg = CheckpointTxg(mkfs_root.checkpoint_txg.0 + 1);
+    write_root_into_ring_slot(
+        &mut image,
+        &system_configuration,
+        earlier_empty_slot,
+        &newer_root,
+    );
+    PoolWithARootOneCheckpointTxgNewerThanTheMkfsRoot {
+        image,
+        mkfs_root,
+        newer_root,
+    }
+}
+
+/// 建过新池新建文件的池（`build_pool`）：新池新建文件那条根照抄一份、txg 加一，写进根环最后一个空槽。择根择到这一条，
+/// 环里的 journal 记录都在它的水位之下，重放原样走它；读根环次序里第一条自证过的根是另一条。
+/// 只读挂载要沿根把中央映射树与树表读出来，只做过 mkfs 的池打不开（实测报 `Open(Walk(UnitUnreadable { .. }))`），所以这一格用建过文件的池。
+struct BuiltPoolWithACopyOfTheFileRootOneCheckpointTxgNewer {
+    image: MemoryPool,
+    first_root_in_ring_order: RootRecord,
+    newer_root: RootRecord,
+}
+
+fn built_pool_with_a_copy_of_the_file_root_one_checkpoint_txg_newer(
+    tag: &str,
+) -> BuiltPoolWithACopyOfTheFileRootOneCheckpointTxgNewer {
+    let built = build_pool(tag);
+    let mut image = built.memory_pool();
+    let system_configuration =
+        choose_system_configuration(&image).expect("建过文件的池择得出系统配置");
+    let file_root = choose_root(&image, &system_configuration).expect("根环里有新池新建文件那条根");
+    let immutable = &system_configuration.immutable;
+    let (first_self_verified_slot, first_root_in_ring_order) = readable_roots_with_ring_slots(
+        &image,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    )[0];
+    let last_empty_slot = *every_root_ring_slot(&immutable.sizes)
+        .iter()
+        .rev()
+        .find(|ring_slot| {
+            !ring_slot_holds_a_self_verified_root(&image, &system_configuration, **ring_slot)
+        })
+        .expect("根环里有空槽");
+    assert!(
+        first_self_verified_slot < last_empty_slot,
+        "照抄的那条写在读根环次序里第一条自证过的根之后"
+    );
+    let mut newer_root = file_root;
+    newer_root.checkpoint_txg = CheckpointTxg(file_root.checkpoint_txg.0 + 1);
+    write_root_into_ring_slot(
+        &mut image,
+        &system_configuration,
+        last_empty_slot,
+        &newer_root,
+    );
+    BuiltPoolWithACopyOfTheFileRootOneCheckpointTxgNewer {
+        image,
+        first_root_in_ring_order,
+        newer_root,
+    }
+}
+
+/// 只读挂载拿所选槽里那条根（txg 新的那条）调重放：施加之后的根与打开挂载态的根都是它；
+/// 拿根环里别的自证过的根（例如读根环次序里第一条）去调，这两条根就不是它。
+#[test]
+fn mount_read_only_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root() {
+    let pool = built_pool_with_a_copy_of_the_file_root_one_checkpoint_txg_newer(
+        "mount-read-only-replays-from-the-chosen-slot",
+    );
+    let chosen_key = (pool.newer_root.instance, pool.newer_root.checkpoint_txg);
+    assert_ne!(
+        (
+            pool.first_root_in_ring_order.instance,
+            pool.first_root_in_ring_order.checkpoint_txg,
+        ),
+        chosen_key,
+        "读根环次序里第一条自证过的根与所选根是两条不同的根"
+    );
+
+    let mounted = mount_read_only(&pool.image)
+        .expect("建过文件的池加一条照抄新池新建文件那条、txg 新一个的根，只读挂得上");
+
+    assert_eq!(
+        mounted.chosen_root, pool.newer_root,
+        "择到的是照抄的那条（txg 最新）"
+    );
+    let effective_key = (
+        mounted.effective_root.instance,
+        mounted.effective_root.checkpoint_txg,
+    );
+    let opened_key = (
+        mounted.mounted.root().instance,
+        mounted.mounted.root().checkpoint_txg,
+    );
+    assert_eq!(
+        (effective_key, opened_key),
+        (chosen_key, chosen_key),
+        "施加之后的根与打开挂载态的根都是所选槽里那条（水位之上没有记录，原样走它），不是根环里别的自证过的根"
+    );
+}
+
+/// 可写挂载的读阶段拿所选槽里那条根（txg 新的那条）调重放：挂载交回的施加之后的根就是它；
+/// 拿根环里别的自证过的根（例如读根环次序里第一条，mkfs 那条）去调，施加之后的根就成了那一条。
+#[test]
+fn mount_writable_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root() {
+    let pool = pool_with_a_root_one_checkpoint_txg_newer_than_the_mkfs_root(
+        "mount-writable-replays-from-the-chosen-slot",
+    );
+    let chosen_key = (pool.newer_root.instance, pool.newer_root.checkpoint_txg);
+    assert_ne!(
+        (pool.mkfs_root.instance, pool.mkfs_root.checkpoint_txg),
+        chosen_key,
+        "mkfs 那条根与所选根是两条不同的根"
+    );
+    let mut devices: Vec<_> = pool
+        .image
+        .devices
+        .iter()
+        .map(|(identity, sparse_device)| {
+            let mut device = SparseBlockDevice::new(
+                pool.image.device_size_in_bytes,
+                PhysicalBlockSizeInBytes(parameters().geometry.physical_block_size),
+            );
+            device.image = sparse_device.clone();
+            (*identity, device)
+        })
+        .collect();
+
+    let mounted = mount_writable(&parameters(), &mut devices)
+        .expect("只做过 mkfs 的池加一条照抄 mkfs 那条、txg 新一个的根，可写挂得上");
+
+    assert_eq!(
+        mounted.output.chosen_root, pool.newer_root,
+        "择到的是前一个槽里 txg 新的那条"
+    );
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg,
+        ),
+        chosen_key,
+        "施加之后的根是所选槽里那条（水位之上没有记录，原样走它），不是根环里别的自证过的根"
+    );
+}
+
 /// 择新键（txg, 实例代号）完全相同的两条根：前面是 mkfs 那条，后面那条照抄它、只把实例表指针的两条位置条目的校验和改掉（表读不出）。
 /// 择根取前面那条（只有严格更新才换）；F 生效值按表判有效根，读的也是前面那条指着的那张表，读得出，算出 mkfs 那一版的 F。
 /// 取了后面那条就读不出表，重读一次仍读不出，报错。
```

### mutations-append.tsv

```
只读挂载胶水拿所选槽之外的根调重放（代码轮 m3-formal-code-r1 K1-b，改法落地一节余下的一处）：mount_read_only 把读根环次序里第一条自证过的根交给 replay_journal，不是 choose_root 择的那条	crates/singlefs-core/src/mounted_read.rs	        &chosen_root,\n        system_configuration.immutable.sizes.journal_ring_bytes,	        &crate::recovery::readable_roots_with_ring_slots(reader, &system_configuration.immutable.region_devices, &system_configuration.immutable.sizes, &system_configuration.immutable.filesystem_identifier)[0].1,\n        system_configuration.immutable.sizes.journal_ring_bytes,	-p singlefs-harness --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance	mount_read_only_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root
可写挂载读阶段胶水拿所选槽之外的根调重放（代码轮 m3-formal-code-r1 K1-b，改法落地一节余下的一处）：read_stage 把读根环次序里第一条自证过的根交给 replay_journal，不是 choose_root 择的那条	crates/singlefs-core/src/mount.rs	        &chosen_root,\n        system_configuration.immutable.sizes.journal_ring_bytes,	        &crate::recovery::readable_roots_with_ring_slots(cache, &system_configuration.immutable.region_devices, &system_configuration.immutable.sizes, &system_configuration.immutable.filesystem_identifier)[0].1,\n        system_configuration.immutable.sizes.journal_ring_bytes,	-p singlefs-harness --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance	mount_writable_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root
```

### mutations-replacements.tsv

```
增补 3 第 3 件：判崩溃点时取择根而不是施加记录前缀之后实际走的那条根（记录已写、根槽未写那一格内容与根身份配不上）	crates/singlefs-checker-tier/src/crash_injection.rs	        let read_back = observed_read_back_after_a_crash(report);	        let read_back = singlefs_harness::model_comparison::observed_read_back(&report.outcome);	-p singlefs-checker-tier --test crash_injection_campaign -- --include-ignored crash_injection_fast_tier	crash_injection_fast_tier_recovers_only_into_versions_the_model_committed
实审 B2 追加件（A2a 第 37 条对齐）：槽 0 无效的盘不借别的盘记的槽距，照 4096 找槽 1	crates/singlefs-checker/src/image.rs	                .or(spacing_recorded_by_the_first_valid_slot_zero)	                .or(spacing_recorded_by_the_first_valid_slot_zero.and(Some(FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES)))	-p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots -- a_device_whose_slot_zero_does_not_verify_has_its_slot_one_looked_for_at_the_spacing_another_device_records	a_device_whose_slot_zero_does_not_verify_has_its_slot_one_looked_for_at_the_spacing_another_device_records
```

## E165 装置（第一轮冻结副本 → 现状，r3 档第一、二段）

```diff
--- /tmp/claude-1000/m3-formal-code-r1/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	2026-10-02 11:00:57.163920725 +0000
+++ crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	2026-10-02 15:00:53.587413423 +0000
@@ -10,6 +10,7 @@
 //! `--only <域>`（可写几个，逗号分隔）、`--max-n <N>`、`--family legal|adversarial|all` 只跑一部分。
 //! 多盘域 M、M6、MC（登记 `research/prompts/e165-r2-prereg.md` 5.3，第三段）只在 `--profile r2 --family multi_disk` 时跑：每条记录在 0、1 号盘
 //! 同一落点各一份、各自落或没落，「完整在盘上」取任一份，直接建表先 0 号盘再 1 号盘；`--family all` 仍只含合法历史与对抗两族。
+//! 择根与重放合判一族 K、K9、KW、KA（登记 `research/prompts/e165-r3-prereg.md` 5.3 至 5.5）只在 `--profile r3 --family root_choice` 时跑：真 `choose_root`、`scan_journal`、`replay_journal` 三步串起来，判据 KJ0 至 KJ3 各报各的；`--family all` 不含它。
 //! 线程数取环境变量 `E165_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，都没设取 `available_parallelism`；计数与线程数无关。
 //! 产物是 `E7RESULT` 行，收尾 `name=done emitted=N`；耗时另打 `E7TIMING` 行，不进逐字比对。
 //! `light` 档的界：L 到 9、U 到 5、A 到 3，其余同第一段（2026-10-01 在 10 线程上量到每态挂钟 0.24–1.2 微秒，每个域取到 3 秒之内）。
@@ -38,7 +39,7 @@
 use std::ops::Range;
 use std::panic::{self, AssertUnwindSafe};
 use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
-use std::sync::Mutex;
+use std::sync::{Arc, Mutex};
 use std::time::Instant;
 
 use singlefs_core::address::{
@@ -52,7 +53,8 @@
 };
 use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
 use singlefs_core::recovery::{
-    replay_journal, scan_journal, PoolReader, RecoveryFailure,
+    choose_root, every_root_ring_slot, read_root_ring_slot, replay_journal, scan_journal,
+    BadRootRingSlotReading, PoolReader, RecoveryFailure, RootRingSlotReading,
     CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,
 };
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
@@ -188,11 +190,13 @@
     VaryWithTheVersion(MarkerParity),
 }
 
-/// 判哪几条判据：r1 档只判原登记与 r1 登记的；r2 档另判 J6Qn 两行与 J7 五行（登记 r2 5.4）。
+/// 判哪几条判据：r1 档只判原登记与 r1 登记的；r2 档另判 J6Qn 两行与 J7 五行（登记 r2 5.4）；r3 档在 r2 档之上，对抗一族的 J6Qn 按 I-8.12 再拆。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum CriteriaSet {
     RegisteredBeforeR2,
     WithJ6QnAndJ7,
+    /// r3 档（登记 `research/prompts/e165-r3-prereg.md` 5.1、5.6）：r2 档全部判据原样，对抗一族的 J6Qn 再按 I-8.12 拆行。
+    WithJ6QnAndJ7SplitByI812,
 }
 
 /// 判据一侧拿哪张记录表去判（登记 r2 5.4 T-表、T-行；`--table direct|scan|both`，不给取 `direct`，只在 r2 档有）。
@@ -275,6 +279,16 @@
         }
     }
 
+    /// r3 档（登记 r3 5.1）：生成器与 r2 档相同，判据是 r2 档的全部再加对抗一族的 I-8.12 拆行。
+    const fn r3(marker_parity: MarkerParity) -> Self {
+        Self {
+            generator: FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
+            criteria: CriteriaSet::WithJ6QnAndJ7SplitByI812,
+            table_source: RecordTableSource::Direct,
+            reported_disk_order: ReportedDiskOrder::FirstDeviceBeforeSecond,
+        }
+    }
+
     /// 只给 J7 的阳性对照用：生成器照 r1 档，判据照 r2 档。
     const R1_GENERATOR_WITH_R2_CRITERIA: Self = Self {
         generator: FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
@@ -304,7 +318,15 @@
     const fn judges_j6qn_and_j7(self) -> bool {
         match self.criteria {
             CriteriaSet::RegisteredBeforeR2 => false,
-            CriteriaSet::WithJ6QnAndJ7 => true,
+            CriteriaSet::WithJ6QnAndJ7 | CriteriaSet::WithJ6QnAndJ7SplitByI812 => true,
+        }
+    }
+
+    /// 对抗一族的 J6Qn 按 I-8.12 再拆（登记 r3 5.6）：只有 r3 档。
+    const fn splits_j6qn_by_i812(self) -> bool {
+        match self.criteria {
+            CriteriaSet::RegisteredBeforeR2 | CriteriaSet::WithJ6QnAndJ7 => false,
+            CriteriaSet::WithJ6QnAndJ7SplitByI812 => true,
         }
     }
 
@@ -1929,6 +1951,16 @@
         RowsDifferInReportedOrder => "rows_differ_in_reported_order",
         DirectTableHoldsARecord => "direct_table_holds_a_record",
         FullyLandedScanTableSizeChecked => "fully_landed_scan_table_size_checked",
+        TablesSatisfyI812 => "i812_states_hold",
+        TablesViolateI812 => "i812_states_fail",
+        J6QnPremiseHoldsI812HoldsJudged => "j6qn_premise_holds_i812_holds_judged",
+        J6QnPremiseHoldsI812HoldsViolations => "J6Qn_premise_holds_i812_holds",
+        J6QnPremiseHoldsI812FailsJudged => "j6qn_premise_holds_i812_fails_judged",
+        J6QnPremiseHoldsI812FailsViolations => "J6Qn_premise_holds_i812_fails",
+        J6QnPremiseFailsI812HoldsJudged => "j6qn_premise_fails_i812_holds_judged",
+        J6QnPremiseFailsI812HoldsViolations => "J6Qn_premise_fails_i812_holds",
+        J6QnPremiseFailsI812FailsJudged => "j6qn_premise_fails_i812_fails_judged",
+        J6QnPremiseFailsI812FailsViolations => "J6Qn_premise_fails_i812_fails",
     }
 }
 
@@ -2743,6 +2775,9 @@
     ScanTableLosesItsLargestKeyRecord,
     /// T-行（登记 r2 5.5）：扫盘建表那一遍的实现结局换成 `Panicked`，表不动。
     ScanPassOutcomeBecomesAPanic,
+    /// 对抗一族 J6Qn 按 I-8.12 拆开的四行（登记 r3 5.7 的 KF3）：第一次调用返回所选根，第二次调用报
+    /// `RootPublishCarriesMoreThanOneLastRecordFlag…`；第二次既不是返回根、也不是 txg 顶格，每个判到的态都是 J6Qn 违例。
+    FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag,
 }
 
 /// J7a 判的三个字段（登记 r2 第一节）。
@@ -3005,6 +3040,7 @@
         | ControlKind::ReportsOneRecordOverTheLimit
         | ControlKind::SecondCallReportsMoreThanOneLastRecordFlag
         | ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor
+        | ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag
         | ControlKind::ScanTableLosesItsLargestKeyRecord
         | ControlKind::ScanPassOutcomeBecomesAPanic => today(),
     }
@@ -3029,9 +3065,10 @@
                 verification_passed: 0,
             })
         }
-        Implementation::Control(ControlKind::SecondCallReportsMoreThanOneLastRecordFlag) => {
-            ReplayOutcome::Failed(FailureKind::RootPublishCarriesMoreThanOneLastRecordFlag)
-        }
+        Implementation::Control(
+            ControlKind::SecondCallReportsMoreThanOneLastRecordFlag
+            | ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag,
+        ) => ReplayOutcome::Failed(FailureKind::RootPublishCarriesMoreThanOneLastRecordFlag),
         Implementation::Control(ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor) => {
             ReplayOutcome::Failed(FailureKind::ChosenRootTxgHasNoSuccessor)
         }
@@ -3052,6 +3089,7 @@
     first: &ReturnedReplay,
     next_timeline_version_root: Option<RootRecord>,
     premise_holds: bool,
+    table_satisfies_i812: Option<bool>,
     records: &RecordTable,
     disk: &MemoryDisk<'_>,
     verify_named_units: bool,
@@ -3081,6 +3119,7 @@
             &second,
             fixed_point_holds,
             premise_holds,
+            table_satisfies_i812,
             describe_witness,
             tally,
         );
@@ -3132,13 +3171,57 @@
     second_call_reports_no_successor && first_root_txg_is_the_largest
 }
 
+/// I-8.12（同一实例里 journal 记录的 txg 随计数器不减；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行）
+/// 在「这一态交给 `replay_journal` 的那张记录表」上成不成立（登记 r3 第一节「Q3 的拆法」）：表按（实例，计数器）升序摊开，同一实例里
+/// 相邻两条记录后一条的 txg 小于前一条即不成立（相邻不减与任两条不减等价）。`parse` 拒收的序号 0 进不了表，判在表上就不看它。
+/// 本地写法，不调 `crates/` 里 Verus 前件 `journal_txg_does_not_decrease_with_the_counter_within_an_instance`。
+fn record_table_satisfies_i812(records: &RecordTable) -> bool {
+    let mut previous_record_of_the_same_instance: Option<&JournalRecord> = None;
+    for ((instance, _counter), record) in records {
+        if let Some(previous_record) = previous_record_of_the_same_instance {
+            if previous_record.instance == *instance
+                && record.checkpoint_txg < previous_record.checkpoint_txg
+            {
+                return false;
+            }
+        }
+        previous_record_of_the_same_instance = Some(record);
+    }
+    true
+}
+
+/// J6Qn 的前提真 / 假与 I-8.12 成不成立，四个格各自的「判了几态」与「违例几态」两个量（登记 r3 5.6）。
+const fn j6qn_cell_counters(premise_holds: bool, i812_holds: bool) -> (Counter, Counter) {
+    match (premise_holds, i812_holds) {
+        (true, true) => (
+            Counter::J6QnPremiseHoldsI812HoldsJudged,
+            Counter::J6QnPremiseHoldsI812HoldsViolations,
+        ),
+        (true, false) => (
+            Counter::J6QnPremiseHoldsI812FailsJudged,
+            Counter::J6QnPremiseHoldsI812FailsViolations,
+        ),
+        (false, true) => (
+            Counter::J6QnPremiseFailsI812HoldsJudged,
+            Counter::J6QnPremiseFailsI812HoldsViolations,
+        ),
+        (false, false) => (
+            Counter::J6QnPremiseFailsI812FailsJudged,
+            Counter::J6QnPremiseFailsI812FailsViolations,
+        ),
+    }
+}
+
 /// J6Qn（登记 r2 第一节，判决 S6 取 Qn）：第二次 (i) 返回根且与 ρ_1 全字段相等，或 (ii) 因 ρ_1 的 txg 顶格报
 /// `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`，记相等；别的报错与 panic 一律违例。前提真 / 假两行照原 J6 分。
+/// `table_satisfies_i812` 给了（r3 档的对抗一族）时，同一次判再按这张表满不满足 I-8.12 记进四格里的一格（登记 r3 5.6），
+/// 两行原有的计数一个不动。
 fn judge_the_fixed_point_under_qn(
     first_root: &RootRecord,
     second: &ReplayOutcome,
     fixed_point_holds: bool,
     premise_holds: bool,
+    table_satisfies_i812: Option<bool>,
     describe_witness: &dyn Fn() -> String,
     tally: &mut Tally,
 ) {
@@ -3147,6 +3230,11 @@
     } else {
         Counter::J6QnPremiseFailsJudged
     });
+    let split_cell =
+        table_satisfies_i812.map(|i812_holds| j6qn_cell_counters(premise_holds, i812_holds));
+    if let Some((judged_counter, _violations_counter)) = split_cell {
+        tally.add(judged_counter);
+    }
     let counts_as_equal_under_qn = fixed_point_holds
         || second_call_failed_because_the_first_root_txg_is_the_largest(second, first_root);
     if counts_as_equal_under_qn {
@@ -3168,6 +3256,9 @@
         },
         &describe_with_the_second_call,
     );
+    if let Some((_judged_counter, violations_counter)) = split_cell {
+        tally.add_violation(violations_counter, &describe_with_the_second_call);
+    }
 }
 
 /// J7（登记 r2 第一节，判决 S1 后半）：凡第一次调用返回根 ρ 的态都判。J7a：实例表指针、分配记录树根、fsid 各等于所选根的；
@@ -3550,6 +3641,7 @@
                 returned,
                 next_timeline_version_root,
                 truth.above_water_record_count <= truth.in_flight_limit,
+                None,
                 records,
                 disk,
                 state.verify_named_units,
@@ -5034,6 +5126,13 @@
                         None => today(),
                     }
                 }
+                ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag => {
+                    ReplayOutcome::Returned(ReturnedReplay {
+                        root: self.chosen_root,
+                        prefix_applied: 0,
+                        verification_passed: 0,
+                    })
+                }
                 ControlKind::ReportsOneRecordOverTheLimit => match today() {
                     ReplayOutcome::Returned(mut returned) => {
                         returned.prefix_applied = in_flight_limit + 1;
@@ -5071,6 +5170,16 @@
             || self.describe_the_state_with_its_first_outcome(&first, &expectations);
 
         tally.add(Counter::States);
+        // I-8.12 成不成立（登记 r3 5.6）：对抗一族每个状态各数一次，不看实现的结局；只有 r3 档数。
+        let table_satisfies_i812 = self
+            .run_profile
+            .splits_j6qn_by_i812()
+            .then(|| record_table_satisfies_i812(records));
+        match table_satisfies_i812 {
+            Some(true) => tally.add(Counter::TablesSatisfyI812),
+            Some(false) => tally.add(Counter::TablesViolateI812),
+            None => {}
+        }
         if holds_a_record_rejected_by_parse {
             tally.add(Counter::StatesHoldingARecordRejectedByParse);
         }
@@ -5153,6 +5262,7 @@
                     returned,
                     None,
                     above_water_record_count <= in_flight_limit,
+                    table_satisfies_i812,
                     records,
                     disk,
                     true,
@@ -5515,6 +5625,7 @@
         MultiDiskAppliedSetAndLandedVersion => "J5_multi_disk",
         TableComparisonSeesADifferentTable => "T_table",
         TableComparisonSeesADifferentRow => "T_row",
+        J6QnSplitByI812 => "J6Qn_i812_split",
     }
 }
 
@@ -5546,7 +5657,8 @@
             | Self::AsGapStops
             | Self::AMutual
             | Self::AStructure
-            | Self::ALimit => Family::Adversarial,
+            | Self::ALimit
+            | Self::J6QnSplitByI812 => Family::Adversarial,
             Self::MultiDiskAnyCopyIsOnDisk
             | Self::MultiDiskReturnedRoot
             | Self::MultiDiskFarthestVersion
@@ -5562,9 +5674,11 @@
         )
     }
 
-    /// 这一档判不判这条判据：J6Qn 与 J7 只在 r2 档判（登记 r2 5.4），T-表与 T-行只在 `--table both` 时判，别的两档都判。
+    /// 这一档判不判这条判据：J6Qn 与 J7 只在 r2、r3 档判（登记 r2 5.4），J6Qn 的 I-8.12 拆行只在 r3 档判（登记 r3 5.6），
+    /// T-表与 T-行只在 `--table both` 时判，别的两档都判。
     const fn is_judged_under(self, run_profile: RunProfile) -> bool {
         match self {
+            Self::J6QnSplitByI812 => run_profile.splits_j6qn_by_i812(),
             Self::TableComparisonSeesADifferentTable | Self::TableComparisonSeesADifferentRow => {
                 run_profile.compares_the_two_record_tables()
             }
@@ -5615,6 +5729,8 @@
 const CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN: usize = 3;
 const CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN: usize = 3;
 const CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN: usize = 2;
+/// 对抗一族的 6 槽小环在 I-8.12 拆行的对照里扫到 n = 3（登记 r3 5.7：A6 n ≤ 3，那里前提为假的态才出现）。
+const CONTROL_LARGEST_SIZE_OF_THE_I812_SMALL_RING_DOMAIN: usize = 3;
 /// 对照界里 L 域 j\* ≥ 1 的态数：登记第七节见证数 N = 1..5 的和，0 + 2 + 16 + 104 + 640。
 const FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND: u64 = 762;
 /// 对照界里 L 域的态数：登记 r2 第七节乙类「L 大环对照界 N ≤ 5：态数 3 110」（2 + 12 + 72 + 432 + 2 592）。
@@ -6143,9 +6259,257 @@
         Criterion::TableComparisonSeesADifferentRow => {
             table_comparison_control_readings_on_rows(criterion, run_profile, thread_count)
         }
+        Criterion::J6QnSplitByI812 => i812_split_control_readings(run_profile, thread_count),
     }
 }
 
+// ── J6Qn 按 I-8.12 拆行（登记 r3 5.6、5.7、第七节乙类）
+
+/// 对抗一族每个 (域, n) 上，表满足 / 不满足 I-8.12 的态数（登记 r3 第七节乙类「Q3 的分类计数」，`anchors_r3_i812.py` 动态规划与暴力枚举互核）：
+/// (域, n, 成立, 不成立)。三个域各 n 的态数之和等于原登记 5.7 的 57 585 320、788 838、6 139 590。
+const PINNED_I812_STATE_COUNTS: [(Domain, usize, u64, u64); 10] = [
+    (Domain::AdversarialBigRing, 1, 146, 0),
+    (Domain::AdversarialBigRing, 2, 9_122, 1_536),
+    (Domain::AdversarialBigRing, 3, 523_570, 254_464),
+    (Domain::AdversarialBigRing, 4, 28_459_330, 28_337_152),
+    (Domain::AdversarialSmallRingSix, 1, 146, 0),
+    (Domain::AdversarialSmallRingSix, 2, 9_122, 1_536),
+    (Domain::AdversarialSmallRingSix, 3, 523_570, 254_464),
+    (Domain::AdversarialBackChain, 1, 290, 0),
+    (Domain::AdversarialBackChain, 2, 35_906, 6_144),
+    (Domain::AdversarialBackChain, 3, 4_079_970, 2_017_280),
+];
+
+/// KF3 假结局（第一次返回所选根、第二次报「不止一条末条标志」）在对照界里必须读到的四格（登记 r3 第七节乙类，`anchors_r3_i812_premise.py`）：
+/// (域, n, 前提真·I-8.12 成立, 前提真·不成立, 前提假·成立, 前提假·不成立)。每格的违例数与 judged 都等于它（对照界内第一次调用都「返回」）。
+const PINNED_KF3_CELLS: [(Domain, usize, [u64; 4]); 7] = [
+    (Domain::AdversarialBigRing, 1, [146, 0, 0, 0]),
+    (Domain::AdversarialBigRing, 2, [9_122, 1_536, 0, 0]),
+    (Domain::AdversarialSmallRingSix, 1, [146, 0, 0, 0]),
+    (Domain::AdversarialSmallRingSix, 2, [9_122, 1_536, 0, 0]),
+    (
+        Domain::AdversarialSmallRingSix,
+        3,
+        [490_802, 221_696, 32_768, 32_768],
+    ),
+    (Domain::AdversarialBackChain, 1, [290, 0, 0, 0]),
+    (Domain::AdversarialBackChain, 2, [35_906, 6_144, 0, 0]),
+];
+
+/// 四格的次序（前提真·成立、前提真·不成立、前提假·成立、前提假·不成立），与 [`PINNED_KF3_CELLS`] 里四个数的次序一致。
+const I812_SPLIT_CELLS: [(bool, bool); 4] =
+    [(true, true), (true, false), (false, true), (false, false)];
+
+fn pinned_i812_state_counts(domain: Domain, size: usize) -> Option<(u64, u64)> {
+    PINNED_I812_STATE_COUNTS
+        .iter()
+        .find(|(pinned_domain, pinned_size, _, _)| *pinned_domain == domain && *pinned_size == size)
+        .map(|(_, _, holds, fails)| (*holds, *fails))
+}
+
+fn pinned_kf3_cells(domain: Domain, size: usize) -> Option<[u64; 4]> {
+    PINNED_KF3_CELLS
+        .iter()
+        .find(|(pinned_domain, pinned_size, _)| *pinned_domain == domain && *pinned_size == size)
+        .map(|(_, _, cells)| *cells)
+}
+
+/// 对抗一族这一个 (域, n) 上，表满足 / 不满足 I-8.12 的态数等于登记的分类计数；不符 ⇒ 作废条款 TV4（装置数错了）。
+fn assert_the_pinned_i812_state_counts(domain: Domain, size: usize, tally: &Tally) {
+    let (pinned_holding, pinned_failing) = pinned_i812_state_counts(domain, size).unwrap_or_else(|| {
+        panic!(
+            "作废条款 TV4：{} 域 n = {size} 没有登记 r3 第七节乙类的 I-8.12 分类计数，这一格没被钉住",
+            domain.label()
+        )
+    });
+    assert_eq!(
+        (
+            tally.get(Counter::TablesSatisfyI812),
+            tally.get(Counter::TablesViolateI812)
+        ),
+        (pinned_holding, pinned_failing),
+        "作废条款 TV4：{} 域 n = {size} 的 I-8.12 成立 / 不成立态数与登记 r3 第七节乙类不等",
+        domain.label()
+    );
+    assert_eq!(
+        tally.get(Counter::TablesSatisfyI812) + tally.get(Counter::TablesViolateI812),
+        tally.get(Counter::States),
+        "作废条款 TV4：{} 域 n = {size} 的 I-8.12 成立与不成立两类态数之和不等于状态数",
+        domain.label()
+    );
+}
+
+/// 拆开的四行对回未拆的两行（登记 r3 作废条款 TV6）：前提真那两格的违例数之和与 judged 之和各等于 `J6Qn_premise_holds` 一行的，前提假同理。
+/// 对得上交回 `None`，对不上交回一句说明。
+fn the_split_rows_differ_from_the_unsplit_rows(tally: &Tally) -> Option<String> {
+    let sum_of_cells = |premise_holds: bool, pick: fn((Counter, Counter)) -> Counter| {
+        I812_SPLIT_CELLS
+            .iter()
+            .filter(|(cell_premise_holds, _)| *cell_premise_holds == premise_holds)
+            .map(|(cell_premise_holds, i812_holds)| {
+                tally.get(pick(j6qn_cell_counters(*cell_premise_holds, *i812_holds)))
+            })
+            .sum::<u64>()
+    };
+    let judged_of = |cells: (Counter, Counter)| cells.0;
+    let violations_of = |cells: (Counter, Counter)| cells.1;
+    let split_and_unsplit = [
+        (
+            "前提真的违例数",
+            sum_of_cells(true, violations_of),
+            tally.get(Counter::J6QnPremiseHoldsViolations),
+        ),
+        (
+            "前提真的 judged",
+            sum_of_cells(true, judged_of),
+            tally.get(Counter::J6QnPremiseHoldsJudged),
+        ),
+        (
+            "前提假的违例数",
+            sum_of_cells(false, violations_of),
+            tally.get(Counter::J6QnPremiseFailsViolations),
+        ),
+        (
+            "前提假的 judged",
+            sum_of_cells(false, judged_of),
+            tally.get(Counter::J6QnPremiseFailsJudged),
+        ),
+    ];
+    let differing: Vec<String> = split_and_unsplit
+        .iter()
+        .filter(|(_, split_sum, unsplit)| split_sum != unsplit)
+        .map(|(what, split_sum, unsplit)| {
+            format!("{what}：拆开之后之和 {split_sum}、未拆那一行 {unsplit}")
+        })
+        .collect();
+    (!differing.is_empty()).then(|| differing.join("；"))
+}
+
+/// 对抗一族一个 (域, n) 的 I-8.12 拆行（登记 r3 5.6）：四格各一行（judged、违例数、违例里最先的见证），另一行报全部状态里表成立 / 不成立的态数。
+/// 不是 r3 档、或不是对抗一族，一行都不出。
+fn rows_of_the_i812_split(
+    run_profile: RunProfile,
+    domain: Domain,
+    place: &str,
+    tally: &Tally,
+) -> Vec<String> {
+    if !run_profile.splits_j6qn_by_i812() || domain.family() != Family::Adversarial {
+        return Vec::new();
+    }
+    let mut rows: Vec<String> = I812_SPLIT_CELLS
+        .iter()
+        .map(|(premise_holds, i812_holds)| {
+            let (judged_counter, violations_counter) =
+                j6qn_cell_counters(*premise_holds, *i812_holds);
+            format!(
+                "name=violations {place} criterion={} violations={} judged={} witness={}",
+                violations_counter.label(),
+                tally.get(violations_counter),
+                tally.get(judged_counter),
+                witness_field(tally, violations_counter)
+            )
+        })
+        .collect();
+    rows.push(format!(
+        "name=i812_states {place} i812_holds={} i812_fails={} states={}",
+        tally.get(Counter::TablesSatisfyI812),
+        tally.get(Counter::TablesViolateI812),
+        tally.get(Counter::States)
+    ));
+    rows
+}
+
+/// 对抗一族一个域在全部 n 上的 I-8.12 拆行合计（登记 r3 5.6「每个域收尾的 criteria=r2 判决行后面另起一行 criteria=r3_i812」）。
+fn verdict_row_of_the_i812_split(place: &str, total: &Tally) -> String {
+    let cell = |premise_holds: bool, i812_holds: bool| {
+        let (judged_counter, violations_counter) = j6qn_cell_counters(premise_holds, i812_holds);
+        (total.get(violations_counter), total.get(judged_counter))
+    };
+    let (holds_holds_violations, holds_holds_judged) = cell(true, true);
+    let (holds_fails_violations, holds_fails_judged) = cell(true, false);
+    let (fails_holds_violations, fails_holds_judged) = cell(false, true);
+    let (fails_fails_violations, fails_fails_judged) = cell(false, false);
+    format!(
+        "name=verdict {place} criteria=r3_i812 j6qn_premise_holds_i812_holds_violations={holds_holds_violations} j6qn_premise_holds_i812_holds_judged={holds_holds_judged} j6qn_premise_holds_i812_fails_violations={holds_fails_violations} j6qn_premise_holds_i812_fails_judged={holds_fails_judged} j6qn_premise_fails_i812_holds_violations={fails_holds_violations} j6qn_premise_fails_i812_holds_judged={fails_holds_judged} j6qn_premise_fails_i812_fails_violations={fails_fails_violations} j6qn_premise_fails_i812_fails_judged={fails_fails_judged} i812_holds={} i812_fails={} split_rows_equal_the_unsplit_rows={}",
+        total.get(Counter::TablesSatisfyI812),
+        total.get(Counter::TablesViolateI812),
+        the_split_rows_differ_from_the_unsplit_rows(total).is_none()
+    )
+}
+
+/// KF3 的对照（登记 r3 5.7、第七节乙类）：对抗一族对照界（A n ≤ 2、A6 n ≤ 3、AC n ≤ 2）每个 n 上，第一次返回所选根、第二次报「不止一条末条标志」，
+/// 四格各自的违例数与 judged 都等于钉的数；全部状态里表成立 / 不成立的态数也等于钉的分类计数。
+fn i812_split_control_readings(
+    run_profile: RunProfile,
+    thread_count: usize,
+) -> Vec<ControlReading> {
+    let criterion = Criterion::J6QnSplitByI812;
+    let control =
+        ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag;
+    let mut readings = Vec::new();
+    for (domain, largest_size) in [
+        (
+            Domain::AdversarialBigRing,
+            CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
+        ),
+        (
+            Domain::AdversarialSmallRingSix,
+            CONTROL_LARGEST_SIZE_OF_THE_I812_SMALL_RING_DOMAIN,
+        ),
+        (
+            Domain::AdversarialBackChain,
+            CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
+        ),
+    ] {
+        for size in domain.smallest_size()..=largest_size {
+            let tally = tally_of_a_domain_size(
+                domain,
+                size,
+                Implementation::Control(control),
+                run_profile_of_a_control(control, run_profile),
+                thread_count,
+            );
+            let pinned_cells = pinned_kf3_cells(domain, size)
+                .expect("对照界里每个 (域, n) 都有登记 r3 第七节乙类 KF3 的四格");
+            let (pinned_holding, pinned_failing) = pinned_i812_state_counts(domain, size)
+                .expect("对照界里每个 (域, n) 都有登记 r3 第七节乙类的分类计数");
+            let place = format!(
+                "domain={},n={size},implementation=first_call_returns_the_chosen_root_second_call_reports_more_than_one_last_record_flag",
+                domain.label()
+            );
+            for ((premise_holds, i812_holds), pinned) in I812_SPLIT_CELLS.iter().zip(pinned_cells) {
+                let (judged_counter, violations_counter) =
+                    j6qn_cell_counters(*premise_holds, *i812_holds);
+                readings.push(reading_that_must_equal(
+                    criterion,
+                    format!("{place},counter={}", violations_counter.label()),
+                    tally.get(violations_counter),
+                    pinned,
+                ));
+                readings.push(reading_that_must_equal(
+                    criterion,
+                    format!("{place},counter={}", judged_counter.label()),
+                    tally.get(judged_counter),
+                    pinned,
+                ));
+            }
+            readings.push(reading_that_must_equal(
+                criterion,
+                format!("{place},counter={}", Counter::TablesSatisfyI812.label()),
+                tally.get(Counter::TablesSatisfyI812),
+                pinned_holding,
+            ));
+            readings.push(reading_that_must_equal(
+                criterion,
+                format!("{place},counter={}", Counter::TablesViolateI812.label()),
+                tally.get(Counter::TablesViolateI812),
+                pinned_failing,
+            ));
+        }
+    }
+    readings
+}
+
 /// L 域 N ≤ 5 上直接建的表非空的态数（至少一条记录落了盘的态数）：登记 r2 第七节乙类「L 大环对照界 N ≤ 5：……至少一条记录落盘的态数 2 989」
 /// （每段历史的 2^N 个落盘子集里去掉空集：1 + 9 + 63 + 405 + 2 511）。
 const STATES_WITH_A_RECORD_ON_DISK_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN: u64 =
@@ -6916,6 +7280,9 @@
             witness_field(tally, Counter::RowsDiffer),
         ));
     }
+    for row in rows_of_the_i812_split(run_profile, domain, &place, tally) {
+        emitter.result(&row);
+    }
     if domain.family() == Family::Adversarial {
         emitter.result(&format!(
             "name=specification {place} expects_failure={}",
@@ -7084,6 +7451,9 @@
             total.get(Counter::J7cJudged),
         ));
     }
+    if run_profile.splits_j6qn_by_i812() && domain.family() == Family::Adversarial {
+        emitter.result(&verdict_row_of_the_i812_split(&place, total));
+    }
     if run_profile.compares_the_two_record_tables() {
         let reported_order_fields = match run_profile.reported_disk_order {
             ReportedDiskOrder::FirstDeviceBeforeSecond => String::new(),
@@ -7775,7 +8145,20 @@
         "逐态判定行同样认 --profile"
     );
     assert!(split(&["--marker-parity", "odd"]).is_err(), "r1 档不认奇偶");
-    assert!(split(&["--profile", "r3"]).is_err());
+    assert_eq!(
+        split(&["--profile", "r3", "--bounds", "segment1"]),
+        Ok((
+            RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+            owned(&["--bounds", "segment1"])
+        )),
+        "r3 档认得，参数原样留下"
+    );
+    assert_ne!(
+        RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+        RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+        "r3 档与 r2 档是两个档"
+    );
+    assert!(split(&["--profile", "r4"]).is_err());
     assert!(split(&["--profile"]).is_err());
     assert!(split(&["--profile", "r2", "--profile", "r1"]).is_err());
 }
@@ -7803,6 +8186,148 @@
     );
 }
 
+/// I-8.12 在手写的表上（登记 r3 第一节「Q3 的拆法」、9.2 甲）：同一实例里 txg 随计数器不减才成立；相等算成立（「严格增」那一读法判不成立）；
+/// 换实例的两条不比；计数器断号不影响。
+#[cfg(test)]
+fn check_the_i812_judgement_on_hand_made_tables() {
+    let in_another_instance = |mut record: JournalRecord| {
+        record.instance = InstanceGeneration(CHOSEN_INSTANCE + 1);
+        record
+    };
+    assert!(
+        record_table_satisfies_i812(&table_of(Vec::new())),
+        "空表：成立"
+    );
+    assert!(
+        record_table_satisfies_i812(&table_of(vec![hand_made_record(7, 100, 1, true)])),
+        "一条：成立"
+    );
+    assert!(
+        record_table_satisfies_i812(&table_of(vec![
+            hand_made_record(7, 100, 1, true),
+            hand_made_record(8, 100, 1, true)
+        ])),
+        "同一实例里相邻两条 txg 相等：不减，成立"
+    );
+    assert!(
+        !record_table_satisfies_i812(&table_of(vec![
+            hand_made_record(7, 101, 1, true),
+            hand_made_record(8, 100, 1, true)
+        ])),
+        "同一实例里后一条的 txg 比前一条小：不成立"
+    );
+    assert!(
+        record_table_satisfies_i812(&table_of(vec![
+            hand_made_record(7, 100, 1, true),
+            hand_made_record(9, 102, 1, true)
+        ])),
+        "计数器 7、9 之间断号、txg 上升：成立"
+    );
+    assert!(
+        !record_table_satisfies_i812(&table_of(vec![
+            hand_made_record(7, 102, 1, true),
+            hand_made_record(9, 100, 1, true)
+        ])),
+        "计数器 7、9 之间断号、txg 下降：不成立（相邻不减与任两条不减等价，断号照比）"
+    );
+    assert!(
+        record_table_satisfies_i812(&table_of(vec![
+            hand_made_record(7, 101, 1, true),
+            in_another_instance(hand_made_record(8, 100, 1, true))
+        ])),
+        "两条属于不同实例：I-8.12 只管同一实例里，换实例那一半不判"
+    );
+}
+
+/// 对抗一族最小几档上，表成立 / 不成立 I-8.12 的态数等于登记的分类计数（登记 r3 第七节乙类；9.2 甲前两条变异靠它红）。
+#[cfg(test)]
+fn check_the_i812_state_counts_of_the_smallest_sizes() {
+    let run_profile = RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
+    for (domain, largest_size) in [
+        (Domain::AdversarialBigRing, 3),
+        (Domain::AdversarialSmallRingSix, 3),
+        (Domain::AdversarialBackChain, 2),
+    ] {
+        for size in domain.smallest_size()..=largest_size {
+            let tally = tally_of_a_domain_size(
+                domain,
+                size,
+                Implementation::TodayReplayJournal,
+                run_profile,
+                1,
+            );
+            assert_the_pinned_i812_state_counts(domain, size, &tally);
+            assert_eq!(
+                the_split_rows_differ_from_the_unsplit_rows(&tally),
+                None,
+                "{} 域 n = {size}：拆开的四行对得回未拆的两行",
+                domain.label()
+            );
+        }
+    }
+}
+
+/// KF3 的四格读数（登记 r3 5.7、第七节乙类）：对照界里七个 (域, n)，每个 n 上四格的违例数与 judged、成立 / 不成立的态数共十个读数，都红。
+#[cfg(test)]
+fn check_the_i812_split_control_readings() {
+    let readings = control_readings_of(
+        Criterion::J6QnSplitByI812,
+        RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+        1,
+    );
+    assert_eq!(
+        readings.len(),
+        PINNED_KF3_CELLS.len() * 10,
+        "每个 (域, n) 十个读数：四格各违例数与 judged 八个、成立与不成立的态数两个：{readings:?}"
+    );
+    assert!(
+        readings.iter().all(|reading| reading.is_red_as_required),
+        "KF3 的读数都等于钉的数：{readings:?}"
+    );
+}
+
+/// 拆出来的行（登记 r3 5.6、9.2 甲）：r3 档对抗一族每个 (域, n) 五行（四格各一行、`i812_states` 一行）；r2 档与合法历史一族一行都没有。
+#[cfg(test)]
+fn check_the_rows_of_the_i812_split() {
+    let r3 = RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
+    let r2 = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence);
+    let tally = tally_of_a_domain_size(
+        Domain::AdversarialBigRing,
+        2,
+        Implementation::TodayReplayJournal,
+        r3,
+        1,
+    );
+    let place = "domain=A ring_slots=64 n=2";
+    let rows = rows_of_the_i812_split(r3, Domain::AdversarialBigRing, place, &tally);
+    assert_eq!(rows.len(), 5, "四格各一行、i812_states 一行：{rows:?}");
+    for criterion in [
+        "J6Qn_premise_holds_i812_holds",
+        "J6Qn_premise_holds_i812_fails",
+        "J6Qn_premise_fails_i812_holds",
+        "J6Qn_premise_fails_i812_fails",
+    ] {
+        assert!(
+            rows.iter()
+                .any(|row| row.contains(&format!(" criterion={criterion} violations="))),
+            "缺 {criterion} 那一行：{rows:?}"
+        );
+    }
+    assert_eq!(
+        rows[4],
+        format!("name=i812_states {place} i812_holds=9122 i812_fails=1536 states=10658"),
+        "A n = 2 的分类计数（登记 r3 第七节乙类）"
+    );
+    assert!(
+        rows_of_the_i812_split(r2, Domain::AdversarialBigRing, place, &tally).is_empty(),
+        "r2 档不出这几行"
+    );
+    assert!(
+        rows_of_the_i812_split(r3, Domain::LegalBigRing, place, &tally).is_empty(),
+        "合法历史一族不出这几行"
+    );
+}
+
 /// J7 五行对照的读数（登记 r2 第七节）：r2 档 J7a 各自那一格 762、另两格 0，J7b 762，J7c 2 348；r1 档生成器上都是 0。
 #[cfg(test)]
 fn check_the_j7_control_readings() {
@@ -8216,6 +8741,3383 @@
     ),
 ];
 
+// ── 择根与重放合判一族（登记 `research/prompts/e165-r3-prereg.md` 5.3 至 5.5：K、K9、KW、KA；只在 `--profile r3 --family root_choice` 时跑）
+//
+// 被测对象是 `recover` 的 `Consult` 支里串起来的三步：`choose_root`、`scan_journal`、`replay_journal(…, true)`。
+// 一个状态 = 内存盘上同时摆好根环（三个区域、每区 S 个槽）与 journal 环（两盘镜像）的一份盘；根与记录由合法历史生成，按持久顺序取崩溃点。
+// 判据一侧只用本地常量与生成时记下的真相（哪个根槽里是哪条根、哪条记录有哪几份在盘上、哪几次用户发布已确认），不调 core 的判定函数。
+
+/// 根环区域数、区域起点三个常量、区域归属、根槽宽、固定结构槽距、mkfs 的第 0 代根（登记第三节的回比表；
+/// [`cross_check_root_choice_local_constants_against_the_implementation`] 逐个回比）。
+const ROOT_CHOICE_REGION_COUNT: u64 = 3;
+const ROOT_CHOICE_BASE_SLOT: u64 = 64;
+const ROOT_CHOICE_PRIME_STEP: u64 = 3;
+const ROOT_CHOICE_CHUNK_BYTES: u64 = 1 << 20;
+const ROOT_CHOICE_REGION_DEVICES: [u32; 3] = [0, 1, 0];
+const ROOT_CHOICE_ROOT_SLOT_BYTES: usize = 512;
+const ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES: u32 = 4096;
+const ROOT_CHOICE_MKFS_INSTANCE: u32 = 0;
+const ROOT_CHOICE_MKFS_CHECKPOINT_TXG: u64 = 0;
+/// 暖机至多推几次空发布（D16 已定项 8「至多 R = 3 次」）；第一版实付 2 次是几何的结果，不是这里的常量。
+const ROOT_CHOICE_WARM_UP_EMPTY_PUBLISHES_AT_MOST: usize = 3;
+/// 每区槽数：K、K9、KA 取 8，KW 取 4（登记 5.3、第三节回比表）。
+const ROOT_CHOICE_SLOTS_PER_REGION_WIDE: u64 = 8;
+const ROOT_CHOICE_SLOTS_PER_REGION_NARROW: u64 = 4;
+/// 内存盘根槽数组的长度：三个区域 × 每区至多 8 个槽。
+const ROOT_CHOICE_ROOT_SLOTS_AT_MOST: usize = 24;
+/// 内存盘 journal 记录数组的长度：环至多 64 槽。
+const ROOT_CHOICE_RING_SLOTS_AT_MOST: usize = 64;
+
+labelled_enumeration! {
+    /// 判据表（登记 5.5）；阳性对照那段循环遍历它。
+    enum RootChoiceCriterion {
+        Kj0Defined => "KJ0",
+        Kj1NotEarlierThanTheLastConfirmedPublish => "KJ1",
+        Kj2NotEarlierThanAnySelfVerifiedRoot => "KJ2",
+        Kj3LandsOnTheIdealRecovery => "KJ3",
+    }
+}
+
+labelled_enumeration! {
+    /// 登记 5.4 的四个域：K、K9、KW 是合法历史，KA 是不合法的对抗根环。
+    enum RootChoiceDomain {
+        BigRing => "K",
+        SmallRingNine => "K9",
+        WrappingRootRing => "KW",
+        AdversarialRootRing => "KA",
+    }
+}
+
+impl RootChoiceDomain {
+    const fn ring(self) -> RingGeometry {
+        match self {
+            Self::BigRing | Self::WrappingRootRing | Self::AdversarialRootRing => RingGeometry::BIG,
+            Self::SmallRingNine => RingGeometry::SMALL_NINE,
+        }
+    }
+
+    const fn slots_per_region(self) -> u64 {
+        match self {
+            Self::BigRing | Self::SmallRingNine | Self::AdversarialRootRing => {
+                ROOT_CHOICE_SLOTS_PER_REGION_WIDE
+            }
+            Self::WrappingRootRing => ROOT_CHOICE_SLOTS_PER_REGION_NARROW,
+        }
+    }
+
+    /// 用户发布每次的记录条数（登记 5.3：k ∈ {1, 2}，KW 只取 1）。KA 没有历史。
+    const fn user_publish_record_counts(self) -> &'static [usize] {
+        match self {
+            Self::BigRing | Self::SmallRingNine => &[1, 2],
+            Self::WrappingRootRing => &[1],
+            Self::AdversarialRootRing => &[],
+        }
+    }
+
+    /// 合法历史一族判 KJ1、KJ3；KA 不是合法历史，只判 KJ0、KJ2（登记 5.5 最后一列、第一节读法）。
+    const fn is_a_legal_history(self) -> bool {
+        match self {
+            Self::BigRing | Self::SmallRingNine | Self::WrappingRootRing => true,
+            Self::AdversarialRootRing => false,
+        }
+    }
+
+    /// 全界（登记 5.4：K M ≤ 3、U ≤ 4；K9 M ≤ 2、U ≤ 2；KW M = 1、U ≤ 14）与轻档（调试用，不是登记的界）。
+    const fn largest_mounts(self, tier: BoundsTier) -> usize {
+        match (self, tier) {
+            (Self::BigRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 3,
+            (Self::BigRing, BoundsTier::Light) => 2,
+            (Self::SmallRingNine, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 2,
+            (Self::SmallRingNine | Self::WrappingRootRing, BoundsTier::Light) => 1,
+            (Self::WrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 1,
+            (
+                Self::AdversarialRootRing,
+                BoundsTier::SegmentOne | BoundsTier::SegmentTwo | BoundsTier::Light,
+            ) => 0,
+        }
+    }
+
+    const fn largest_user_publishes(self, tier: BoundsTier) -> usize {
+        match (self, tier) {
+            (Self::BigRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 4,
+            (Self::BigRing | Self::SmallRingNine, BoundsTier::Light) => 2,
+            (Self::SmallRingNine, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 2,
+            (Self::WrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 14,
+            (Self::WrappingRootRing, BoundsTier::Light) => 6,
+            (
+                Self::AdversarialRootRing,
+                BoundsTier::SegmentOne | BoundsTier::SegmentTwo | BoundsTier::Light,
+            ) => 0,
+        }
+    }
+}
+
+/// 一个 (M, U) 格：M 次挂载、U 次用户发布（跨挂载的总数）。KA 没有挂载与发布，取 (0, 0)。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct RootChoiceCell {
+    mounts: usize,
+    user_publishes: usize,
+}
+
+/// 一个域在这一档、这一次命令行下要跑的格，按 (M 升序, U 升序)。`largest_user_publishes_cap` 是 `--max-u`（只给调试用）。
+fn root_choice_cells_of(
+    domain: RootChoiceDomain,
+    tier: BoundsTier,
+    largest_user_publishes_cap: Option<usize>,
+) -> Vec<RootChoiceCell> {
+    if !domain.is_a_legal_history() {
+        return vec![RootChoiceCell {
+            mounts: 0,
+            user_publishes: 0,
+        }];
+    }
+    let largest_user_publishes = largest_user_publishes_cap
+        .map_or(domain.largest_user_publishes(tier), |cap| {
+            cap.min(domain.largest_user_publishes(tier))
+        });
+    let mut cells = Vec::new();
+    for mounts in 1..=domain.largest_mounts(tier) {
+        for user_publishes in 0..=largest_user_publishes {
+            cells.push(RootChoiceCell {
+                mounts,
+                user_publishes,
+            });
+        }
+    }
+    cells
+}
+
+/// 版本 = (`checkpoint_txg`, 实例代号)，txg 为主、实例代号破平局（D13 已定项 1 M3；与 D22 已定项 7 的择新序同一个序）。
+/// 字段次序就是比较次序。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+struct RootVersion {
+    checkpoint_txg: u64,
+    instance: u32,
+}
+
+impl RootVersion {
+    fn of_root(root: &RootRecord) -> Self {
+        Self {
+            checkpoint_txg: root.checkpoint_txg.0,
+            instance: root.instance.0,
+        }
+    }
+
+    fn describe(self) -> String {
+        format!("txg{}/instance{}", self.checkpoint_txg, self.instance)
+    }
+}
+
+// ── 根环落点与暖机计划（本地算，不调 core；回比在 [`cross_check_root_choice_local_constants_against_the_implementation`]）
+
+/// 本地算的区域起点：基槽 × 槽字节 + 区域号 × 步长 × chunk（D22 已定项 2；登记第三节回比表第 1 行）。
+const fn root_choice_region_start_in_bytes(region: u64) -> u64 {
+    ROOT_CHOICE_BASE_SLOT * LOCAL_SLOT_BYTES
+        + region * ROOT_CHOICE_PRIME_STEP * ROOT_CHOICE_CHUNK_BYTES
+}
+
+/// 本地算的根槽落点 (区域, 槽)：区域 = txg mod 3、槽 = (txg div 3) mod S（登记第三节回比表第 2 行）。
+const fn root_choice_target_of_publish(checkpoint_txg: u64, slots_per_region: u64) -> (u64, u64) {
+    (
+        checkpoint_txg % ROOT_CHOICE_REGION_COUNT,
+        (checkpoint_txg / ROOT_CHOICE_REGION_COUNT) % slots_per_region,
+    )
+}
+
+fn root_choice_slot_index_of(region: u64, slot: u64, slots_per_region: u64) -> usize {
+    usize::try_from(region * slots_per_region + slot).expect("根槽数不超过 24")
+}
+
+/// 这次发布的根写进根槽数组的哪一格（区域号主序）。
+fn root_choice_slot_index_of_publish(domain: RootChoiceDomain, checkpoint_txg: u64) -> usize {
+    let slots_per_region = domain.slots_per_region();
+    let (region, slot) = root_choice_target_of_publish(checkpoint_txg, slots_per_region);
+    root_choice_slot_index_of(region, slot, slots_per_region)
+}
+
+fn root_choice_device_of_publish(checkpoint_txg: u64) -> usize {
+    let region =
+        usize::try_from(checkpoint_txg % ROOT_CHOICE_REGION_COUNT).expect("区域号不超过 2");
+    usize::try_from(ROOT_CHOICE_REGION_DEVICES[region]).expect("盘号是 0 或 1")
+}
+
+/// 暖机计划（D16 已定项 8、登记 5.3 第 2 点）：写行那次发布取 `row_write_txg`，之后从它起逐个加一、看那个 txg 的根落哪块盘，
+/// 推到本实例的根覆盖两块盘为止，空发布至多 [`ROOT_CHOICE_WARM_UP_EMPTY_PUBLISHES_AT_MOST`] 次。返回的第一个是写行那次发布。
+fn root_choice_warm_up_plan(row_write_txg: u64) -> Vec<u64> {
+    let mut covered = [false; 2];
+    covered[root_choice_device_of_publish(row_write_txg)] = true;
+    let mut plan = vec![row_write_txg];
+    let mut checkpoint_txg = row_write_txg;
+    while !(covered[0] && covered[1]) && plan.len() <= ROOT_CHOICE_WARM_UP_EMPTY_PUBLISHES_AT_MOST {
+        checkpoint_txg += 1;
+        plan.push(checkpoint_txg);
+        covered[root_choice_device_of_publish(checkpoint_txg)] = true;
+    }
+    plan
+}
+
+/// 每个生成出来的根槽，本地算的 (区域, 槽)、偏移与 `root_ring::target_for_publish`、`root_ring::slot_offset` 相等（登记第三节回比表第 2 行；不符走 TH2）。
+fn assert_the_local_root_slot_arithmetic_against_the_implementation(
+    domain: RootChoiceDomain,
+    checkpoint_txg: u64,
+) {
+    let slots_per_region = domain.slots_per_region();
+    let (region, slot) = root_choice_target_of_publish(checkpoint_txg, slots_per_region);
+    let implementation_target = singlefs_core::root_ring::target_for_publish(
+        CheckpointTxg(checkpoint_txg),
+        RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
+            .expect("每区槽数 4 或 8 在格式承诺的区间里"),
+    );
+    assert_eq!(
+        (region, slot),
+        (implementation_target.region, implementation_target.slot),
+        "TH2：本地算的根槽 (区域, 槽) 与 root_ring::target_for_publish 不等（txg {checkpoint_txg}，每区 {slots_per_region} 槽）"
+    );
+    assert_eq!(
+        root_choice_region_start_in_bytes(region)
+            + slot * u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES),
+        singlefs_core::root_ring::slot_offset(
+            implementation_target,
+            ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES
+        )
+        .0,
+        "TH2：本地算的根槽偏移与 root_ring::slot_offset 不等（txg {checkpoint_txg}）"
+    );
+}
+
+// ── 一次发布的材料：它的根（字节与自证结果）与它的记录（每条一个槽）
+
+/// 投影之外的四个字段：全历史取 r1 档那一组常量（登记 5.3 第 4 点）。
+fn root_choice_fields_beyond_the_projection() -> FieldsBeyondTheProjection {
+    RunProfile::R1.fields_beyond_the_projection_of_version(0)
+}
+
+/// 一次发布（含 mkfs 的第 0 代根与 KA 的根）的材料：生成一次、之后只借用。
+#[derive(Debug)]
+struct RootChoicePublishMaterial {
+    version: RootVersion,
+    fields: VersionFields,
+    root_bytes: Vec<u8>,
+    records: Vec<GeneratedSlot>,
+    is_user_publish: bool,
+}
+
+/// 只有根、没有记录的材料：根过 `to_slot(512)` 再 `parse_slot` 必须等于原根，不等当场停（登记停机条款 TH9）。
+fn root_material_without_records(
+    instance: u32,
+    checkpoint_txg: u64,
+    segment: NewRootSegment,
+) -> RootChoicePublishMaterial {
+    let fields = VersionFields {
+        instance,
+        checkpoint_txg,
+        segment,
+    };
+    let root = fields.to_root_with(root_choice_fields_beyond_the_projection());
+    let root_bytes = root.to_slot(ROOT_CHOICE_ROOT_SLOT_BYTES);
+    assert_eq!(
+        RootRecord::parse_slot(&root_bytes, &ROOT_FILESYSTEM_IDENTIFIER),
+        Some(root),
+        "停机条款 TH9：生成的根过不了 to_slot(512) 再 parse_slot（实例 {instance}，txg {checkpoint_txg}）"
+    );
+    RootChoicePublishMaterial {
+        version: RootVersion {
+            checkpoint_txg,
+            instance,
+        },
+        fields,
+        root_bytes,
+        records: Vec::new(),
+        is_user_publish: false,
+    }
+}
+
+/// mkfs 的第 0 代根：实例代号 0、txg 0，种进三个区域各自的槽 0（D22 已定项 8 第 3 条；登记第三节回比表）。
+fn mkfs_root_material() -> RootChoicePublishMaterial {
+    root_material_without_records(
+        ROOT_CHOICE_MKFS_INSTANCE,
+        ROOT_CHOICE_MKFS_CHECKPOINT_TXG,
+        NewRootSegment::off_timeline(0, ROOT_CHOICE_MKFS_CHECKPOINT_TXG),
+    )
+}
+
+struct RootChoicePublishPlan {
+    domain: RootChoiceDomain,
+    instance: u32,
+    checkpoint_txg: u64,
+    publish_ordinal: u64,
+    first_counter: u64,
+    first_transaction: u64,
+    record_count: usize,
+    commit_marker_bits: u32,
+    /// 本实例写的上一条记录的 311 字节头算出的链值；本实例第一条记录没有，取 0（I-8.6）。
+    previous_chain_value: Option<u32>,
+    is_user_publish: bool,
+}
+
+/// 一次发布的材料：k 条记录（写行与暖机的空发布一条：事务号 0、提交标记 1、序号 1、带末条标志），新根段按全历史的发布序号取。
+fn root_choice_publish_material(plan: &RootChoicePublishPlan) -> RootChoicePublishMaterial {
+    assert_the_local_root_slot_arithmetic_against_the_implementation(
+        plan.domain,
+        plan.checkpoint_txg,
+    );
+    let segment = NewRootSegment::of_timeline_publish(plan.publish_ordinal, plan.checkpoint_txg);
+    let mut material = root_material_without_records(plan.instance, plan.checkpoint_txg, segment);
+    material.is_user_publish = plan.is_user_publish;
+    let mut transaction = plan.first_transaction;
+    let mut back_chain = plan.previous_chain_value.unwrap_or(0);
+    for record_position in 0..plan.record_count {
+        let is_last_record_of_the_publish = record_position + 1 == plan.record_count;
+        let is_commit =
+            is_last_record_of_the_publish || (plan.commit_marker_bits >> record_position) & 1 == 1;
+        let counter = plan.first_counter
+            + u64::try_from(record_position).expect("一次发布的记录条数不超过 2");
+        assert!(
+            counter <= plan.domain.ring().ring_slots,
+            "停机条款 TH9：界内最大计数器 {counter} 超过环槽数 {}，journal 环回绕了",
+            plan.domain.ring().ring_slots
+        );
+        let record = JournalRecord {
+            instance: InstanceGeneration(plan.instance),
+            counter,
+            checkpoint_txg: CheckpointTxg(plan.checkpoint_txg),
+            transaction: if plan.is_user_publish { transaction } else { 0 },
+            is_commit,
+            ordinal_within_publish: JournalRecordOrdinalWithinPublish(
+                u32::try_from(record_position + 1).expect("一次发布的记录条数不超过 2"),
+            ),
+            place_in_publish: if is_last_record_of_the_publish {
+                JournalRecordPlaceInPublish::LastRecordOfThePublish
+            } else {
+                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
+            },
+            back_chain,
+            filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES,
+            new_tree_table: segment.tree_table,
+            new_mapping_root: segment.mapping_root,
+            new_tree_identifier_watermark: segment.tree_identifier_watermark,
+            new_rollback_floor: CheckpointTxg(segment.rollback_floor),
+            named: Vec::new(),
+        };
+        let slot = generate_slot(
+            0,
+            plan.domain.ring(),
+            &record,
+            ParseExpectation::MustRoundTrip,
+        );
+        assert_eq!(
+            VersionFields::of_record(&record),
+            material.fields,
+            "停机条款 TH9：记录头里的新根段与这次发布的根不是同一组六个字段"
+        );
+        back_chain = slot.chain_value_of_header;
+        if plan.is_user_publish && is_commit {
+            transaction += 1;
+        }
+        material.records.push(slot);
+    }
+    material
+}
+
+// ── 一段历史走到某一刻的盘（世界）、崩溃点、判据看到的盘面
+
+#[derive(Clone, Debug)]
+struct RootChoiceLandedRecord {
+    material: Arc<RootChoicePublishMaterial>,
+    record_index: usize,
+    counter: u64,
+    /// 0 号盘、1 号盘上这条记录各有没有一份（任一份自证过即算在，D23 已定项 14）。
+    copies_on_devices: [bool; 2],
+}
+
+#[derive(Clone, Debug)]
+struct RootChoiceWorld {
+    domain: RootChoiceDomain,
+    /// 根环三个区域的槽，区域号主序、区域内槽号升序（与 `every_root_ring_slot` 同一个次序）；没写过的槽是 `None`（读成 512 个 0）。
+    root_slots: Vec<Option<Arc<RootChoicePublishMaterial>>>,
+    /// 盘上至少有一份的记录，按写的次序。
+    landed_records: Vec<RootChoiceLandedRecord>,
+    /// 崩溃点所在那次发布之前、fsync 已返回的用户发布，按写的次序（D16 已定项 7；写行与暖机的空发布不是用户的 fsync）。
+    confirmed_user_publishes: Vec<RootVersion>,
+    publishes_started: u64,
+    /// 前面各次挂载的写法（见证用）。
+    history: Vec<String>,
+}
+
+impl RootChoiceWorld {
+    /// mkfs 之后、一次挂载都还没有：第 0 代根在三个区域的槽 0，别处全 0，journal 环空（登记 5.3「mkfs」）。
+    fn right_after_mkfs(domain: RootChoiceDomain) -> Self {
+        let genesis_root = Arc::new(mkfs_root_material());
+        let slots_per_region = domain.slots_per_region();
+        let mut root_slots = vec![
+            None;
+            usize::try_from(ROOT_CHOICE_REGION_COUNT * slots_per_region)
+                .expect("根槽数不超过 24")
+        ];
+        for region in 0..ROOT_CHOICE_REGION_COUNT {
+            root_slots[root_choice_slot_index_of(region, 0, slots_per_region)] =
+                Some(Arc::clone(&genesis_root));
+        }
+        Self {
+            domain,
+            root_slots,
+            landed_records: Vec::new(),
+            confirmed_user_publishes: Vec::new(),
+            publishes_started: 0,
+            history: Vec::new(),
+        }
+    }
+
+    /// 根环里现存的根与盘上读得出的记录里最大的 txg（新实例写行那次发布的 txg 取它加一，D23 已定项 14 注 3）。
+    fn highest_checkpoint_txg_on_disk(&self) -> u64 {
+        let highest_root_txg = self
+            .root_slots
+            .iter()
+            .flatten()
+            .map(|material| material.version.checkpoint_txg)
+            .max()
+            .unwrap_or(0);
+        let highest_record_txg = self
+            .landed_records
+            .iter()
+            .map(|landed| landed.material.version.checkpoint_txg)
+            .max()
+            .unwrap_or(0);
+        highest_root_txg.max(highest_record_txg)
+    }
+
+    /// 盘上读得出的最大计数器（新实例第一条记录的计数器取它加一；没有记录时取 0，于是第一条是 1）。
+    fn highest_landed_counter(&self) -> u64 {
+        self.landed_records
+            .iter()
+            .map(|landed| landed.counter)
+            .max()
+            .unwrap_or(0)
+    }
+
+    fn landed_record_of(
+        material: &Arc<RootChoicePublishMaterial>,
+        record_index: usize,
+        copies_on_devices: [bool; 2],
+    ) -> RootChoiceLandedRecord {
+        RootChoiceLandedRecord {
+            material: Arc::clone(material),
+            record_index,
+            counter: material.records[record_index]
+                .parsed
+                .as_ref()
+                .expect("合法历史的每条记录都解得出（生成时断言过）")
+                .counter,
+            copies_on_devices,
+        }
+    }
+
+    /// 这次发布整次落了：每条记录两盘各一份，根写进它的落点（盖掉那个槽原来的根）。`is_confirmed` 是它之后 fsync 返回了（用户发布才会）。
+    fn with_publish_applied(
+        &self,
+        material: &Arc<RootChoicePublishMaterial>,
+        is_confirmed: bool,
+    ) -> Self {
+        let mut world = self.clone();
+        for record_index in 0..material.records.len() {
+            world
+                .landed_records
+                .push(Self::landed_record_of(material, record_index, [true, true]));
+        }
+        world.root_slots
+            [root_choice_slot_index_of_publish(self.domain, material.version.checkpoint_txg)] =
+            Some(Arc::clone(material));
+        if is_confirmed {
+            world.confirmed_user_publishes.push(material.version);
+        }
+        world.publishes_started += 1;
+        world
+    }
+
+    /// 这次发布崩在 `crashed.crash_point` 上之后的盘；`trace` 是这次挂载到这里为止的写法，连同崩溃点记进历史（见证用）。
+    fn with_crash_applied(&self, crashed: &RootChoiceCrashedPublish<'_>, trace: &str) -> Self {
+        let mut world = self.clone();
+        match crashed.crash_point {
+            RootChoiceCrashPoint::RecordCopiesLanded { copy_mask } => {
+                for record_index in 0..crashed.material.records.len() {
+                    let copies_on_devices = copies_of_a_record_in_a_mask(copy_mask, record_index);
+                    if copies_on_devices[0] || copies_on_devices[1] {
+                        world.landed_records.push(Self::landed_record_of(
+                            crashed.material,
+                            record_index,
+                            copies_on_devices,
+                        ));
+                    }
+                }
+            }
+            RootChoiceCrashPoint::RootLanded => {
+                for record_index in 0..crashed.material.records.len() {
+                    world.landed_records.push(Self::landed_record_of(
+                        crashed.material,
+                        record_index,
+                        [true, true],
+                    ));
+                }
+                world.root_slots[root_choice_slot_index_of_publish(
+                    self.domain,
+                    crashed.material.version.checkpoint_txg,
+                )] = Some(Arc::clone(crashed.material));
+            }
+        }
+        world.publishes_started += 1;
+        world
+            .history
+            .push(format!("{trace};{}", crashed.describe()));
+        world
+    }
+}
+
+/// 一次发布的崩溃点（登记 5.3 第 6 点，D13 已定项 4 按设备切段）：记录那一段 2k 份拷贝的任一子集落了、根没落（4^k 个，位 2j 是第 j 条的
+/// 0 号盘那一份、位 2j + 1 是 1 号盘那一份），或根落了（1 个：系统配置轮换那一段的各个子集在三步眼里是同一块盘，并成这一个）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum RootChoiceCrashPoint {
+    RecordCopiesLanded { copy_mask: u32 },
+    RootLanded,
+}
+
+fn copies_of_a_record_in_a_mask(copy_mask: u32, record_index: usize) -> [bool; 2] {
+    [
+        (copy_mask >> (2 * record_index)) & 1 == 1,
+        (copy_mask >> (2 * record_index + 1)) & 1 == 1,
+    ]
+}
+
+fn root_choice_crash_points_of(record_count: usize) -> Vec<RootChoiceCrashPoint> {
+    let mut crash_points: Vec<RootChoiceCrashPoint> = (0..(1u32 << (2 * record_count)))
+        .map(|copy_mask| RootChoiceCrashPoint::RecordCopiesLanded { copy_mask })
+        .collect();
+    crash_points.push(RootChoiceCrashPoint::RootLanded);
+    crash_points
+}
+
+struct RootChoiceCrashedPublish<'material> {
+    material: &'material Arc<RootChoicePublishMaterial>,
+    crash_point: RootChoiceCrashPoint,
+}
+
+impl RootChoiceCrashedPublish<'_> {
+    fn describe(&self) -> String {
+        let crash_point_text = match self.crash_point {
+            RootChoiceCrashPoint::RecordCopiesLanded { copy_mask } => {
+                format!("record_copies_mask_{copy_mask:b}_root_not_landed")
+            }
+            RootChoiceCrashPoint::RootLanded => "root_landed".to_string(),
+        };
+        format!(
+            "crash_in_publish_txg{}_instance{}_{}records_{crash_point_text}",
+            self.material.version.checkpoint_txg,
+            self.material.version.instance,
+            self.material.records.len()
+        )
+    }
+}
+
+/// 判据与三步调用看到的盘面：每个根槽里是哪条根（读根环次序），盘上至少有一份的每条记录与它在哪几块盘上。
+struct RootChoiceLandedRecordRef<'world> {
+    material: &'world RootChoicePublishMaterial,
+    record_index: usize,
+    copies_on_devices: [bool; 2],
+}
+
+struct RootChoiceLanding<'world> {
+    roots_in_ring_order: Vec<Option<&'world RootChoicePublishMaterial>>,
+    records: Vec<RootChoiceLandedRecordRef<'world>>,
+}
+
+/// 世界加上崩溃在途的那次发布（没有就是世界本身）落在盘上的样子。
+fn root_choice_landing_of<'world>(
+    world: &'world RootChoiceWorld,
+    crashed: Option<&'world RootChoiceCrashedPublish<'world>>,
+) -> RootChoiceLanding<'world> {
+    let mut roots_in_ring_order: Vec<Option<&RootChoicePublishMaterial>> = world
+        .root_slots
+        .iter()
+        .map(|slot| slot.as_deref())
+        .collect();
+    let mut records: Vec<RootChoiceLandedRecordRef<'_>> = world
+        .landed_records
+        .iter()
+        .map(|landed| RootChoiceLandedRecordRef {
+            material: &landed.material,
+            record_index: landed.record_index,
+            copies_on_devices: landed.copies_on_devices,
+        })
+        .collect();
+    if let Some(crashed) = crashed {
+        match crashed.crash_point {
+            RootChoiceCrashPoint::RecordCopiesLanded { copy_mask } => {
+                for record_index in 0..crashed.material.records.len() {
+                    let copies_on_devices = copies_of_a_record_in_a_mask(copy_mask, record_index);
+                    if copies_on_devices[0] || copies_on_devices[1] {
+                        records.push(RootChoiceLandedRecordRef {
+                            material: crashed.material,
+                            record_index,
+                            copies_on_devices,
+                        });
+                    }
+                }
+            }
+            RootChoiceCrashPoint::RootLanded => {
+                for record_index in 0..crashed.material.records.len() {
+                    records.push(RootChoiceLandedRecordRef {
+                        material: crashed.material,
+                        record_index,
+                        copies_on_devices: [true, true],
+                    });
+                }
+                roots_in_ring_order[root_choice_slot_index_of_publish(
+                    world.domain,
+                    crashed.material.version.checkpoint_txg,
+                )] = Some(crashed.material);
+            }
+        }
+    }
+    RootChoiceLanding {
+        roots_in_ring_order,
+        records,
+    }
+}
+
+// ── 内存盘：根槽（512 字节）与 journal 记录槽（4096 字节）摆在各自的落点上
+
+/// 三步调用的读者：两块盘；根环没写过的槽读成 512 个 0（mkfs 清零，登记 5.5 第一段）；journal 环没落盘的记录槽读成 4096 个 0；别处读不出。
+struct RootChoiceDisk<'landing> {
+    ring: RingGeometry,
+    slots_per_region: u64,
+    root_slot_bytes: [Option<&'landing [u8]>; ROOT_CHOICE_ROOT_SLOTS_AT_MOST],
+    record_bytes: [[Option<&'landing [u8]>; ROOT_CHOICE_RING_SLOTS_AT_MOST]; 2],
+}
+
+impl<'landing> RootChoiceDisk<'landing> {
+    fn of_the_landing(domain: RootChoiceDomain, landing: &RootChoiceLanding<'landing>) -> Self {
+        let mut root_slot_bytes: [Option<&[u8]>; ROOT_CHOICE_ROOT_SLOTS_AT_MOST] =
+            [None; ROOT_CHOICE_ROOT_SLOTS_AT_MOST];
+        for (slot_index, root) in landing.roots_in_ring_order.iter().enumerate() {
+            root_slot_bytes[slot_index] = root.map(|material| material.root_bytes.as_slice());
+        }
+        let mut record_bytes: [[Option<&[u8]>; ROOT_CHOICE_RING_SLOTS_AT_MOST]; 2] =
+            [[None; ROOT_CHOICE_RING_SLOTS_AT_MOST]; 2];
+        for record in &landing.records {
+            let slot = &record.material.records[record.record_index];
+            let ring_index = usize::try_from(
+                (slot
+                    .parsed
+                    .as_ref()
+                    .expect("合法历史的每条记录都解得出（生成时断言过）")
+                    .counter
+                    - LOCAL_FIRST_JOURNAL_COUNTER)
+                    % domain.ring().ring_slots,
+            )
+            .expect("环槽数不超过 64");
+            for (device_index, has_a_copy) in record.copies_on_devices.iter().enumerate() {
+                if *has_a_copy {
+                    assert!(
+                        record_bytes[device_index][ring_index].is_none(),
+                        "停机条款 TH9：两条记录落在同一个环槽里，journal 环回绕了"
+                    );
+                    record_bytes[device_index][ring_index] = Some(slot.bytes.as_slice());
+                }
+            }
+        }
+        Self {
+            ring: domain.ring(),
+            slots_per_region: domain.slots_per_region(),
+            root_slot_bytes,
+            record_bytes,
+        }
+    }
+}
+
+impl PoolReader for RootChoiceDisk<'_> {
+    fn device_identities(&self) -> Vec<DeviceIdentity> {
+        vec![DeviceIdentity(0), DeviceIdentity(1)]
+    }
+
+    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
+        (device.0 < 2).then_some(MEMORY_DISK_DEVICE_BYTES)
+    }
+
+    fn read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        for region in 0..ROOT_CHOICE_REGION_COUNT {
+            let region_start = root_choice_region_start_in_bytes(region);
+            let spacing = u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES);
+            if offset.0 < region_start || offset.0 >= region_start + self.slots_per_region * spacing
+            {
+                continue;
+            }
+            let offset_in_region = offset.0 - region_start;
+            let region_device =
+                ROOT_CHOICE_REGION_DEVICES[usize::try_from(region).expect("区域号不超过 2")];
+            if !offset_in_region.is_multiple_of(spacing)
+                || device.0 != region_device
+                || length != ROOT_CHOICE_ROOT_SLOT_BYTES
+            {
+                return None;
+            }
+            let slot_index = root_choice_slot_index_of(
+                region,
+                offset_in_region / spacing,
+                self.slots_per_region,
+            );
+            return Some(match self.root_slot_bytes[slot_index] {
+                Some(bytes) => bytes.get(..length)?.to_vec(),
+                None => vec![0u8; length],
+            });
+        }
+        let ring_start = RingGeometry::ring_start_offset_in_bytes();
+        let is_a_whole_record_slot_of_the_ring = device.0 < 2
+            && offset.0 >= ring_start
+            && offset.0 < ring_start + self.ring.ring_bytes()
+            && (offset.0 - ring_start).is_multiple_of(LOCAL_JOURNAL_RECORD_BYTES)
+            && u64::try_from(length).ok() == Some(LOCAL_JOURNAL_RECORD_BYTES);
+        if !is_a_whole_record_slot_of_the_ring {
+            return None;
+        }
+        let ring_index = usize::try_from((offset.0 - ring_start) / LOCAL_JOURNAL_RECORD_BYTES)
+            .expect("环槽数不超过 64");
+        let device_index = usize::try_from(device.0).expect("盘号是 0 或 1");
+        Some(match self.record_bytes[device_index][ring_index] {
+            Some(bytes) => bytes.get(..length)?.to_vec(),
+            None => vec![0u8; length],
+        })
+    }
+
+    fn journal_record_offsets_hint(
+        &self,
+        _device: DeviceIdentity,
+        _ring_start: DeviceOffsetInBytes,
+        _ring_bytes: u64,
+    ) -> Option<Vec<DeviceOffsetInBytes>> {
+        None
+    }
+}
+
+/// 喂给三步的系统配置：fsid 取时间线根的那个 16 字节常量，区域归属 0 / 1 / 0，根槽宽 512、槽距 4096、每区 S 槽、journal 环长与起点 1024
+/// （`choose_root`、`scan_journal` 不读的字段同 r2 修订 4 那一份的取法）。
+fn system_configuration_for_the_root_choice(domain: RootChoiceDomain) -> SystemConfiguration {
+    SystemConfiguration {
+        immutable: SystemImmutableConfiguration {
+            filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,
+            this_device: DeviceIdentity(0),
+            device_count: SCAN_CONFIGURATION_DEVICE_COUNT,
+            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+            sizes: SystemImmutableSizes {
+                physical_block_size: u32::try_from(ROOT_CHOICE_ROOT_SLOT_BYTES)
+                    .expect("512 装得进 32 位"),
+                minimum_input_output_bytes: u32::try_from(ROOT_CHOICE_ROOT_SLOT_BYTES)
+                    .expect("512 装得进 32 位"),
+                fixed_structure_slot_spacing: ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES,
+                journal_ring_bytes: domain.ring().ring_bytes(),
+                root_ring_slots_per_region:
+                    RootRingSlotsPerRegion::from_system_configuration_field(
+                        domain.slots_per_region(),
+                    )
+                    .expect("每区槽数 4 或 8 在格式承诺的区间里"),
+            },
+            journal_ring_start_slot: SlotNumber(LOCAL_JOURNAL_RING_START_SLOT),
+            root_ring_base_slot: SlotNumber(
+                SCAN_CONFIGURATION_ROOT_RING_BASE_SLOT_THE_SCAN_DOES_NOT_READ,
+            ),
+        },
+        mutable: SystemMutableConfiguration,
+        runtime: SystemRuntimeConfiguration,
+        quantities: SystemRuntimeQuantities {
+            slot_generation: SCAN_CONFIGURATION_SLOT_GENERATION_THE_SCAN_DOES_NOT_READ,
+            journal_tail: SCAN_CONFIGURATION_JOURNAL_TAIL_THE_SCAN_DOES_NOT_READ,
+            journal_instance: InstanceGeneration(CHOSEN_INSTANCE),
+            rollback_floor: CheckpointTxg(SCAN_CONFIGURATION_ROLLBACK_FLOOR_THE_SCAN_DOES_NOT_READ),
+        },
+    }
+}
+
+// ── 理想恢复与真相（登记 5.5 KJ3；判据一侧不调 core）
+
+/// 理想恢复（D13 已定项 1 M4「恰是 j*」，所选根换成给定的那条根）：从它起沿它的实例逐次往后，每次发布的每条记录至少一份在盘上、
+/// 条数累计不超在飞上限、第一次的 txg 是它的 txg + 1（D23 已定项 14 注 1 链首），走得最远的那一版。
+struct RootChoiceIdealRecovery {
+    landed_version: RootVersion,
+    landed_fields: VersionFields,
+    applied_publishes: usize,
+}
+
+fn root_choice_ideal_recovery_from(
+    start: &RootChoicePublishMaterial,
+    records: &[RootChoiceLandedRecordRef<'_>],
+    in_flight_limit: usize,
+) -> RootChoiceIdealRecovery {
+    let mut recovery = RootChoiceIdealRecovery {
+        landed_version: start.version,
+        landed_fields: start.fields,
+        applied_publishes: 0,
+    };
+    let mut records_applied = 0usize;
+    let mut next_checkpoint_txg = start.version.checkpoint_txg;
+    while let Some(following_txg) = next_checkpoint_txg.checked_add(1) {
+        let mut publish: Option<&RootChoicePublishMaterial> = None;
+        let mut landed_records_of_the_publish = 0usize;
+        for record in records {
+            if record.material.version.instance == start.version.instance
+                && record.material.version.checkpoint_txg == following_txg
+            {
+                publish = Some(record.material);
+                landed_records_of_the_publish += 1;
+            }
+        }
+        let Some(publish) = publish else {
+            break;
+        };
+        if landed_records_of_the_publish != publish.records.len()
+            || records_applied + landed_records_of_the_publish > in_flight_limit
+        {
+            break;
+        }
+        records_applied += landed_records_of_the_publish;
+        recovery.landed_version = publish.version;
+        recovery.landed_fields = publish.fields;
+        recovery.applied_publishes += 1;
+        next_checkpoint_txg = following_txg;
+    }
+    recovery
+}
+
+/// 真相里根环每个槽现存的根（读根环次序），自证过的才有。
+fn root_choice_versions_in_ring_order(landing: &RootChoiceLanding<'_>) -> Vec<Option<RootVersion>> {
+    landing
+        .roots_in_ring_order
+        .iter()
+        .map(|root| root.map(|material| material.version))
+        .collect()
+}
+
+/// 版本最大的那一条根（真相里；同版本的几条取读根环次序里最前的，版本相同的根字节相同、哪条都一样）。
+fn root_choice_newest_root<'world>(
+    landing: &RootChoiceLanding<'world>,
+) -> Option<&'world RootChoicePublishMaterial> {
+    let mut newest: Option<&RootChoicePublishMaterial> = None;
+    for root in landing.roots_in_ring_order.iter().copied().flatten() {
+        if newest.is_none_or(|current| root.version > current.version) {
+            newest = Some(root);
+        }
+    }
+    newest
+}
+
+fn root_choice_oldest_root<'world>(
+    landing: &RootChoiceLanding<'world>,
+) -> Option<&'world RootChoicePublishMaterial> {
+    let mut oldest: Option<&RootChoicePublishMaterial> = None;
+    for root in landing.roots_in_ring_order.iter().copied().flatten() {
+        if oldest.is_none_or(|current| root.version < current.version) {
+            oldest = Some(root);
+        }
+    }
+    oldest
+}
+
+// ── 数的量与见证
+
+labelled_enumeration! {
+    /// 每个 (域, M, U) 格上数的量（登记 5.5 末段）。违例类的量在第一次加一时记一个见证。
+    enum RootChoiceCounter {
+        States => "states",
+        ConfirmedStates => "confirmed_states",
+        FirstSelfVerifiedRootIsNotTheNewest => "first_self_verified_root_is_not_the_newest",
+        SameTxgDifferentInstanceRoots => "same_txg_different_instance_roots",
+        HigherInstanceLowerTxgRoots => "higher_instance_lower_txg_roots",
+        TwoOrMoreInstancesInTheRing => "two_or_more_instances",
+        IdealRecoveryApplied => "ideal_recovery_applied",
+        AboveNewestRootOverLimit => "above_newest_root_over_limit",
+        AboveNewestRootAtLimit => "above_newest_root_at_limit",
+        AboveChosenRootOverLimit => "above_chosen_root_over_limit",
+        AboveChosenRootAtLimit => "above_chosen_root_at_limit",
+        StatesWithoutASelfVerifiedRoot => "states_without_a_self_verified_root",
+        PanickedStates => "panicked",
+        ChoseNothingStates => "choose_root_returned_none",
+        ReplayFailedStates => "replay_failed",
+        ReplayAppliedStates => "replay_applied",
+        Kj0Violations => "KJ0",
+        Kj1Judged => "kj1_judged",
+        Kj1Violations => "KJ1",
+        Kj2Judged => "kj2_judged",
+        Kj2Violations => "KJ2",
+        Kj3Judged => "kj3_judged",
+        Kj3Violations => "KJ3",
+        SlotReadingsChecked => "slot_readings_checked",
+        SlotReadingsDifferingFromTheTruth => "slot_readings_differing_from_the_truth",
+    }
+}
+
+#[derive(Clone, Debug)]
+struct RootChoiceTally {
+    counts: Vec<u64>,
+    witnesses: Vec<Option<String>>,
+    /// 真相里版本最大的根之上、同实例的记录条数的最大值（登记第八节）。
+    above_newest_root_maximum: usize,
+    /// 实现所选的根之上、同实例的记录条数的最大值（登记第八节：变体择到更旧的根时在飞上限可能咬到）。
+    above_chosen_root_maximum: usize,
+    /// 已确认的用户发布条数的最大值（登记第八节）。
+    confirmed_publishes_maximum: usize,
+    /// 盘上读得出的最大计数器。
+    largest_counter: u64,
+}
+
+impl RootChoiceTally {
+    fn new() -> Self {
+        Self {
+            counts: vec![0; RootChoiceCounter::ALL.len()],
+            witnesses: vec![None; RootChoiceCounter::ALL.len()],
+            above_newest_root_maximum: 0,
+            above_chosen_root_maximum: 0,
+            confirmed_publishes_maximum: 0,
+            largest_counter: 0,
+        }
+    }
+
+    fn add(&mut self, counter: RootChoiceCounter) {
+        self.counts[counter.index()] += 1;
+    }
+
+    /// 违例加一；这个量的第一条违例记一个见证（枚举次序里最先的那个，合并时按工作单元的次序保留最先的）。
+    fn add_violation(&mut self, counter: RootChoiceCounter, describe_witness: &dyn Fn() -> String) {
+        let index = counter.index();
+        self.counts[index] += 1;
+        if self.witnesses[index].is_none() {
+            self.witnesses[index] = Some(describe_witness());
+        }
+    }
+
+    fn get(&self, counter: RootChoiceCounter) -> u64 {
+        self.counts[counter.index()]
+    }
+
+    fn witness(&self, counter: RootChoiceCounter) -> &str {
+        self.witnesses[counter.index()].as_deref().unwrap_or("none")
+    }
+
+    fn merge(&mut self, later: &Self) {
+        for (count, later_count) in self.counts.iter_mut().zip(&later.counts) {
+            *count += later_count;
+        }
+        for (witness, later_witness) in self.witnesses.iter_mut().zip(&later.witnesses) {
+            if witness.is_none() {
+                witness.clone_from(later_witness);
+            }
+        }
+        self.above_newest_root_maximum = self
+            .above_newest_root_maximum
+            .max(later.above_newest_root_maximum);
+        self.above_chosen_root_maximum = self
+            .above_chosen_root_maximum
+            .max(later.above_chosen_root_maximum);
+        self.confirmed_publishes_maximum = self
+            .confirmed_publishes_maximum
+            .max(later.confirmed_publishes_maximum);
+        self.largest_counter = self.largest_counter.max(later.largest_counter);
+    }
+}
+
+/// 把工作单元分给几条线程跑，再按工作单元的次序合并（同 [`tally_of_work_units`]，计数的种类不同）。
+fn root_choice_tally_of_work_units<Unit: Sync>(
+    units: &[Unit],
+    thread_count: usize,
+    work: &(dyn Fn(usize, &Unit) -> RootChoiceTally + Sync),
+) -> RootChoiceTally {
+    let next_unit_index = AtomicUsize::new(0);
+    let finished_units: Mutex<Vec<(usize, RootChoiceTally)>> = Mutex::new(Vec::new());
+    std::thread::scope(|scope| {
+        for _worker in 0..thread_count.clamp(1, units.len().max(1)) {
+            scope.spawn(|| loop {
+                let unit_index = next_unit_index.fetch_add(1, Ordering::Relaxed);
+                let Some(unit) = units.get(unit_index) else {
+                    break;
+                };
+                let tally = work(unit_index, unit);
+                finished_units
+                    .lock()
+                    .expect("别的线程没有在拿着这把锁的时候 panic")
+                    .push((unit_index, tally));
+            });
+        }
+    });
+    let mut finished_units = finished_units
+        .into_inner()
+        .expect("线程都已经收回，没有谁在拿着这把锁的时候 panic");
+    finished_units.sort_by_key(|(unit_index, _tally)| *unit_index);
+    assert_eq!(finished_units.len(), units.len(), "每个工作单元恰好跑一遍");
+    let mut total = RootChoiceTally::new();
+    for (_unit_index, tally) in &finished_units {
+        total.merge(tally);
+    }
+    total
+}
+
+// ── 被判的一方：今天的三步，或阳性对照的假结局（登记 5.7）
+
+/// KA 上本地写的几种择根序（登记 5.7 第四行、9.1 四行替换文的比较式各写一份；落点 = 择到的根，journal 环空）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum LocalRootChoiceRule {
+    /// 今天的序：txg 大者赢，txg 相等实例代号大者赢，键完全相同取读根环次序里前面的。
+    NewestWins,
+    /// 变异 1825：txg 平局时实例代号低者赢。
+    TieTakesTheLowerInstance,
+    /// 变异 1826：实例代号为主、txg 只破平局。
+    InstanceFirstThenTxg,
+    /// 变异 1823：比较不看实例代号。
+    IgnoresTheInstance,
+    /// 变异 1824：取第一个自证过的槽、不再换。
+    FirstSelfVerifiedWins,
+}
+
+impl LocalRootChoiceRule {
+    const ALL: [Self; 5] = [
+        Self::NewestWins,
+        Self::TieTakesTheLowerInstance,
+        Self::InstanceFirstThenTxg,
+        Self::IgnoresTheInstance,
+        Self::FirstSelfVerifiedWins,
+    ];
+
+    const fn label(self) -> &'static str {
+        match self {
+            Self::NewestWins => "newest_wins",
+            Self::TieTakesTheLowerInstance => "tie_takes_the_lower_instance",
+            Self::InstanceFirstThenTxg => "instance_first_then_txg",
+            Self::IgnoresTheInstance => "ignores_the_instance",
+            Self::FirstSelfVerifiedWins => "first_self_verified_wins",
+        }
+    }
+
+    /// 读根环次序里往后走，`candidate` 换掉已择的 `chosen` 吗。
+    fn replaces(self, candidate: RootVersion, chosen: RootVersion) -> bool {
+        match self {
+            Self::NewestWins => {
+                candidate.checkpoint_txg > chosen.checkpoint_txg
+                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
+                        && candidate.instance > chosen.instance)
+            }
+            Self::TieTakesTheLowerInstance => {
+                candidate.checkpoint_txg > chosen.checkpoint_txg
+                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
+                        && candidate.instance < chosen.instance)
+            }
+            Self::InstanceFirstThenTxg => {
+                candidate.instance > chosen.instance
+                    || (candidate.instance == chosen.instance
+                        && candidate.checkpoint_txg > chosen.checkpoint_txg)
+            }
+            Self::IgnoresTheInstance => candidate.checkpoint_txg > chosen.checkpoint_txg,
+            Self::FirstSelfVerifiedWins => false,
+        }
+    }
+
+    fn chooses_among(self, versions_in_ring_order: &[Option<RootVersion>]) -> Option<RootVersion> {
+        let mut chosen: Option<RootVersion> = None;
+        for candidate in versions_in_ring_order.iter().copied().flatten() {
+            if chosen.is_none_or(|already_chosen| self.replaces(candidate, already_chosen)) {
+                chosen = Some(candidate);
+            }
+        }
+        chosen
+    }
+}
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum RootChoiceImplementation {
+    /// 被测对象：今天的 `choose_root`、`scan_journal`、`replay_journal(…, true)`，不改一个字。
+    TodayThreeSteps,
+    /// KJ0 的对照：在指定的一态上 panic，别的态照真结局。
+    PanicsOnTheDesignatedState,
+    /// KF1：所选根取真相里版本最小的根，落点照理想恢复从它接（不调实现）。
+    ChoosesTheOldestRoot,
+    /// KF2：有已确认发布的态，落点取最后一次已确认那一版的前一版（同实例、txg 减一）；没有已确认的态照理想恢复。
+    LandsOneVersionBeforeTheLastConfirmed,
+    /// KA 上本地写的一种择根序，不调实现。
+    LocalChoice(LocalRootChoiceRule),
+}
+
+#[derive(Debug)]
+enum RootChoiceOutcome {
+    Panicked {
+        step: &'static str,
+    },
+    ChoseNothing,
+    Landed {
+        chosen: RootVersion,
+        landed_version: RootVersion,
+        landed_fields: VersionFields,
+        /// `replay_journal` 交回 `Err`：落到的版本取所选根的（登记第一节读法）。
+        replay_failed: bool,
+    },
+}
+
+// ── 判一个状态（登记 5.5：KJ0、KJ1、KJ2、KJ3 各报各的）
+
+/// 一个状态判之前先算好的真相：根环里每个槽的版本、版本最大的根、最后一次已确认的版本、理想恢复。
+struct RootChoiceStateView<'view, 'world> {
+    landing: &'view RootChoiceLanding<'world>,
+    confirmed_user_publishes: &'view [RootVersion],
+    versions_in_ring_order: Vec<Option<RootVersion>>,
+    newest_root: Option<&'world RootChoicePublishMaterial>,
+    last_confirmed: Option<RootVersion>,
+    ideal_recovery_from_the_newest_root: Option<RootChoiceIdealRecovery>,
+}
+
+fn counter_of_the_landed_record(record: &RootChoiceLandedRecordRef<'_>) -> u64 {
+    record.material.records[record.record_index]
+        .parsed
+        .as_ref()
+        .expect("合法历史的每条记录都解得出（生成时断言过）")
+        .counter
+}
+
+fn describe_the_roots_in_ring_order(
+    versions_in_ring_order: &[Option<RootVersion>],
+    slots_per_region: u64,
+) -> String {
+    let mut parts = Vec::new();
+    for (slot_index, version) in versions_in_ring_order.iter().enumerate() {
+        if let Some(version) = version {
+            let slot_number = u64::try_from(slot_index).expect("根槽数不超过 24");
+            parts.push(format!(
+                "region{}slot{}={}",
+                slot_number / slots_per_region,
+                slot_number % slots_per_region,
+                version.describe()
+            ));
+        }
+    }
+    parts.join("+")
+}
+
+struct RootChoiceJudgement {
+    domain: RootChoiceDomain,
+    implementation: RootChoiceImplementation,
+    system_configuration: SystemConfiguration,
+    in_flight_limit: usize,
+}
+
+impl RootChoiceJudgement {
+    fn new(domain: RootChoiceDomain, implementation: RootChoiceImplementation) -> Self {
+        Self {
+            domain,
+            implementation,
+            system_configuration: system_configuration_for_the_root_choice(domain),
+            in_flight_limit: domain.ring().local_in_flight_limit(),
+        }
+    }
+
+    fn judge(
+        &self,
+        landing: &RootChoiceLanding<'_>,
+        confirmed_user_publishes: &[RootVersion],
+        is_the_designated_state: bool,
+        tally: &mut RootChoiceTally,
+        describe_state: &dyn Fn() -> String,
+    ) {
+        tally.add(RootChoiceCounter::States);
+        let newest_root = root_choice_newest_root(landing);
+        let view = RootChoiceStateView {
+            landing,
+            confirmed_user_publishes,
+            versions_in_ring_order: root_choice_versions_in_ring_order(landing),
+            newest_root,
+            last_confirmed: confirmed_user_publishes.iter().copied().max(),
+            ideal_recovery_from_the_newest_root: newest_root.map(|newest| {
+                root_choice_ideal_recovery_from(newest, &landing.records, self.in_flight_limit)
+            }),
+        };
+        self.count_the_shape_of_the_state(&view, tally);
+        let outcome = self.outcome_of(&view, is_the_designated_state, tally, describe_state);
+        self.judge_the_outcome(&view, &outcome, tally, describe_state);
+    }
+
+    fn count_the_shape_of_the_state(
+        &self,
+        view: &RootChoiceStateView<'_, '_>,
+        tally: &mut RootChoiceTally,
+    ) {
+        let present_versions: Vec<RootVersion> = view
+            .versions_in_ring_order
+            .iter()
+            .copied()
+            .flatten()
+            .collect();
+        if let (Some(first_self_verified), Some(newest)) =
+            (present_versions.first(), view.newest_root)
+        {
+            if *first_self_verified != newest.version {
+                tally.add(RootChoiceCounter::FirstSelfVerifiedRootIsNotTheNewest);
+            }
+        }
+        let mut has_same_txg_different_instance = false;
+        let mut has_higher_instance_lower_txg = false;
+        for earlier in &present_versions {
+            for later in &present_versions {
+                has_same_txg_different_instance |= earlier.checkpoint_txg == later.checkpoint_txg
+                    && earlier.instance != later.instance;
+                has_higher_instance_lower_txg |= earlier.instance > later.instance
+                    && earlier.checkpoint_txg < later.checkpoint_txg;
+            }
+        }
+        if has_same_txg_different_instance {
+            tally.add(RootChoiceCounter::SameTxgDifferentInstanceRoots);
+        }
+        if has_higher_instance_lower_txg {
+            tally.add(RootChoiceCounter::HigherInstanceLowerTxgRoots);
+        }
+        let mut instances: Vec<u32> = present_versions
+            .iter()
+            .map(|version| version.instance)
+            .collect();
+        instances.sort_unstable();
+        instances.dedup();
+        if instances.len() >= 2 {
+            tally.add(RootChoiceCounter::TwoOrMoreInstancesInTheRing);
+        }
+        if view.newest_root.is_none() {
+            tally.add(RootChoiceCounter::StatesWithoutASelfVerifiedRoot);
+        }
+        if view.last_confirmed.is_some() {
+            tally.add(RootChoiceCounter::ConfirmedStates);
+        }
+        tally.confirmed_publishes_maximum = tally
+            .confirmed_publishes_maximum
+            .max(view.confirmed_user_publishes.len());
+        if view
+            .ideal_recovery_from_the_newest_root
+            .as_ref()
+            .is_some_and(|ideal| ideal.applied_publishes > 0)
+        {
+            tally.add(RootChoiceCounter::IdealRecoveryApplied);
+        }
+        for record in &view.landing.records {
+            tally.largest_counter = tally
+                .largest_counter
+                .max(counter_of_the_landed_record(record));
+        }
+        if let Some(newest) = view.newest_root {
+            let above_newest_root = view
+                .landing
+                .records
+                .iter()
+                .filter(|record| {
+                    record.material.version.instance == newest.version.instance
+                        && record.material.version.checkpoint_txg > newest.version.checkpoint_txg
+                })
+                .count();
+            tally.above_newest_root_maximum =
+                tally.above_newest_root_maximum.max(above_newest_root);
+            if above_newest_root > self.in_flight_limit {
+                tally.add(RootChoiceCounter::AboveNewestRootOverLimit);
+            } else if above_newest_root == self.in_flight_limit {
+                tally.add(RootChoiceCounter::AboveNewestRootAtLimit);
+            }
+        }
+    }
+
+    fn outcome_of(
+        &self,
+        view: &RootChoiceStateView<'_, '_>,
+        is_the_designated_state: bool,
+        tally: &mut RootChoiceTally,
+        describe_state: &dyn Fn() -> String,
+    ) -> RootChoiceOutcome {
+        match self.implementation {
+            RootChoiceImplementation::TodayThreeSteps => {
+                self.outcome_of_the_three_steps(view, tally, describe_state)
+            }
+            RootChoiceImplementation::PanicsOnTheDesignatedState => {
+                if is_the_designated_state {
+                    RootChoiceOutcome::Panicked {
+                        step: "designated_state",
+                    }
+                } else {
+                    self.outcome_of_the_three_steps(view, tally, describe_state)
+                }
+            }
+            RootChoiceImplementation::ChoosesTheOldestRoot => {
+                let Some(oldest) = root_choice_oldest_root(view.landing) else {
+                    return RootChoiceOutcome::ChoseNothing;
+                };
+                let ideal = root_choice_ideal_recovery_from(
+                    oldest,
+                    &view.landing.records,
+                    self.in_flight_limit,
+                );
+                RootChoiceOutcome::Landed {
+                    chosen: oldest.version,
+                    landed_version: ideal.landed_version,
+                    landed_fields: ideal.landed_fields,
+                    replay_failed: false,
+                }
+            }
+            RootChoiceImplementation::LandsOneVersionBeforeTheLastConfirmed => {
+                let Some(newest) = view.newest_root else {
+                    return RootChoiceOutcome::ChoseNothing;
+                };
+                if let Some(last_confirmed) = view.confirmed_user_publishes.iter().copied().max() {
+                    let one_version_before = RootVersion {
+                        checkpoint_txg: last_confirmed.checkpoint_txg - 1,
+                        instance: last_confirmed.instance,
+                    };
+                    let landed_fields = view
+                        .landing
+                        .records
+                        .iter()
+                        .find(|record| record.material.version == one_version_before)
+                        .map(|record| record.material.fields)
+                        .expect("最后一次已确认的用户发布前面一次发布的记录整次落过（生成时按这个次序写）");
+                    RootChoiceOutcome::Landed {
+                        chosen: newest.version,
+                        landed_version: one_version_before,
+                        landed_fields,
+                        replay_failed: false,
+                    }
+                } else {
+                    let ideal = root_choice_ideal_recovery_from(
+                        newest,
+                        &view.landing.records,
+                        self.in_flight_limit,
+                    );
+                    RootChoiceOutcome::Landed {
+                        chosen: newest.version,
+                        landed_version: ideal.landed_version,
+                        landed_fields: ideal.landed_fields,
+                        replay_failed: false,
+                    }
+                }
+            }
+            RootChoiceImplementation::LocalChoice(rule) => {
+                let Some(chosen) = rule.chooses_among(&view.versions_in_ring_order) else {
+                    return RootChoiceOutcome::ChoseNothing;
+                };
+                let landed_fields = view
+                    .landing
+                    .roots_in_ring_order
+                    .iter()
+                    .copied()
+                    .flatten()
+                    .find(|material| material.version == chosen)
+                    .map(|material| material.fields)
+                    .expect("择到的版本来自根环里的某个槽");
+                RootChoiceOutcome::Landed {
+                    chosen,
+                    landed_version: chosen,
+                    landed_fields,
+                    replay_failed: false,
+                }
+            }
+        }
+    }
+}
+
+impl RootChoiceJudgement {
+    /// 被测对象：`recover` 的 `Consult` 支里串起来的三步（`choose_root`、`scan_journal`、`replay_journal(…, true)`），各包 `catch_unwind`；
+    /// 另逐槽调 `read_root_ring_slot` 与真相对拍（登记 5.5 末段、停机条款 TH4）。
+    fn outcome_of_the_three_steps(
+        &self,
+        view: &RootChoiceStateView<'_, '_>,
+        tally: &mut RootChoiceTally,
+        describe_state: &dyn Fn() -> String,
+    ) -> RootChoiceOutcome {
+        let disk = RootChoiceDisk::of_the_landing(self.domain, view.landing);
+        self.check_the_slot_readings_against_the_truth(&disk, view, tally, describe_state);
+        IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(true));
+        let outcome = self.outcome_of_the_three_steps_with_the_panic_hook_quiet(&disk);
+        IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(false));
+        outcome
+    }
+
+    fn outcome_of_the_three_steps_with_the_panic_hook_quiet(
+        &self,
+        disk: &RootChoiceDisk<'_>,
+    ) -> RootChoiceOutcome {
+        let chosen_root = match panic::catch_unwind(AssertUnwindSafe(|| {
+            choose_root(disk, &self.system_configuration)
+        })) {
+            Err(_panic_payload) => {
+                return RootChoiceOutcome::Panicked {
+                    step: "choose_root",
+                }
+            }
+            Ok(None) => return RootChoiceOutcome::ChoseNothing,
+            Ok(Some(root)) => root,
+        };
+        let records = match panic::catch_unwind(AssertUnwindSafe(|| {
+            scan_journal(disk, &self.system_configuration)
+        })) {
+            Err(_panic_payload) => {
+                return RootChoiceOutcome::Panicked {
+                    step: "scan_journal",
+                }
+            }
+            Ok(records) => records,
+        };
+        let replay = panic::catch_unwind(AssertUnwindSafe(|| {
+            replay_journal(
+                disk,
+                &chosen_root,
+                self.system_configuration.immutable.sizes.journal_ring_bytes,
+                &records,
+                true,
+            )
+        }));
+        match replay {
+            Err(_panic_payload) => RootChoiceOutcome::Panicked {
+                step: "replay_journal",
+            },
+            Ok(Err(_failure)) => RootChoiceOutcome::Landed {
+                chosen: RootVersion::of_root(&chosen_root),
+                landed_version: RootVersion::of_root(&chosen_root),
+                landed_fields: VersionFields::of_root(&chosen_root),
+                replay_failed: true,
+            },
+            Ok(Ok((_report, returned_root))) => RootChoiceOutcome::Landed {
+                chosen: RootVersion::of_root(&chosen_root),
+                landed_version: RootVersion::of_root(&returned_root),
+                landed_fields: VersionFields::of_root(&returned_root),
+                replay_failed: false,
+            },
+        }
+    }
+
+    /// 读槽对拍：逐槽调 `read_root_ring_slot`，自证过的槽集合与每个槽交回的根必须等于生成时的真相（没写过的槽读成全 0、自证不过）。
+    /// 不一致不当 KJ 的违例记，单独数、记见证，收尾走停机条款 TH4。
+    fn check_the_slot_readings_against_the_truth(
+        &self,
+        disk: &RootChoiceDisk<'_>,
+        view: &RootChoiceStateView<'_, '_>,
+        tally: &mut RootChoiceTally,
+        describe_state: &dyn Fn() -> String,
+    ) {
+        tally.add(RootChoiceCounter::SlotReadingsChecked);
+        let immutable = &self.system_configuration.immutable;
+        for ring_slot in every_root_ring_slot(&immutable.sizes) {
+            let reading = read_root_ring_slot(
+                disk,
+                &immutable.region_devices,
+                &immutable.sizes,
+                &immutable.filesystem_identifier,
+                ring_slot,
+            );
+            let slot_index = root_choice_slot_index_of(
+                ring_slot.region,
+                ring_slot.slot,
+                self.domain.slots_per_region(),
+            );
+            let truth = view.landing.roots_in_ring_order[slot_index];
+            if !root_slot_reading_agrees_with_the_truth(reading, truth) {
+                tally.add_violation(
+                    RootChoiceCounter::SlotReadingsDifferingFromTheTruth,
+                    &|| {
+                        format!(
+                            "region{}_slot{}_read_as={reading:?}_truth={};{}",
+                            ring_slot.region,
+                            ring_slot.slot,
+                            truth.map_or_else(
+                                || "empty".to_string(),
+                                |material| material.version.describe()
+                            ),
+                            describe_state()
+                        )
+                    },
+                );
+            }
+        }
+    }
+
+    /// 一个状态的结局落进四条判据（各报各的，不合成一个真假）。
+    fn judge_the_outcome(
+        &self,
+        view: &RootChoiceStateView<'_, '_>,
+        outcome: &RootChoiceOutcome,
+        tally: &mut RootChoiceTally,
+        describe_state: &dyn Fn() -> String,
+    ) {
+        match outcome {
+            RootChoiceOutcome::Panicked { step } => {
+                tally.add(RootChoiceCounter::PanickedStates);
+                tally.add_violation(RootChoiceCounter::Kj0Violations, &|| {
+                    format!("panicked_in={step};{}", describe_state())
+                });
+            }
+            RootChoiceOutcome::ChoseNothing => {
+                tally.add(RootChoiceCounter::ChoseNothingStates);
+                tally.add_violation(RootChoiceCounter::Kj0Violations, &|| {
+                    format!("choose_root_returned_none;{}", describe_state())
+                });
+            }
+            RootChoiceOutcome::Landed {
+                chosen,
+                landed_version,
+                landed_fields,
+                replay_failed,
+            } => {
+                if *replay_failed {
+                    tally.add(RootChoiceCounter::ReplayFailedStates);
+                }
+                let above_chosen_root = view
+                    .landing
+                    .records
+                    .iter()
+                    .filter(|record| {
+                        record.material.version.instance == chosen.instance
+                            && record.material.version.checkpoint_txg > chosen.checkpoint_txg
+                    })
+                    .count();
+                tally.above_chosen_root_maximum =
+                    tally.above_chosen_root_maximum.max(above_chosen_root);
+                if above_chosen_root > self.in_flight_limit {
+                    tally.add(RootChoiceCounter::AboveChosenRootOverLimit);
+                } else if above_chosen_root == self.in_flight_limit {
+                    tally.add(RootChoiceCounter::AboveChosenRootAtLimit);
+                }
+                if landed_version != chosen {
+                    tally.add(RootChoiceCounter::ReplayAppliedStates);
+                }
+                let facts = || {
+                    format!(
+                        "chosen={};landed={};newest_root={};last_confirmed={};roots_in_ring_order={};{}",
+                        chosen.describe(),
+                        landed_version.describe(),
+                        view.newest_root
+                            .map_or_else(|| "none".to_string(), |newest| newest.version.describe()),
+                        view.last_confirmed
+                            .map_or_else(|| "none".to_string(), RootVersion::describe),
+                        describe_the_roots_in_ring_order(
+                            &view.versions_in_ring_order,
+                            self.domain.slots_per_region()
+                        ),
+                        describe_state()
+                    )
+                };
+                if self.domain.is_a_legal_history() {
+                    if let Some(last_confirmed) = view.last_confirmed {
+                        tally.add(RootChoiceCounter::Kj1Judged);
+                        if *landed_version < last_confirmed {
+                            tally.add_violation(RootChoiceCounter::Kj1Violations, &|| {
+                                format!("KJ1;{}", facts())
+                            });
+                        }
+                    }
+                }
+                if let Some(newest) = view.newest_root {
+                    tally.add(RootChoiceCounter::Kj2Judged);
+                    if *landed_version < newest.version {
+                        tally.add_violation(RootChoiceCounter::Kj2Violations, &|| {
+                            format!("KJ2;{}", facts())
+                        });
+                    }
+                }
+                if self.domain.is_a_legal_history() {
+                    if let Some(ideal) = &view.ideal_recovery_from_the_newest_root {
+                        tally.add(RootChoiceCounter::Kj3Judged);
+                        if *landed_fields != ideal.landed_fields {
+                            tally.add_violation(RootChoiceCounter::Kj3Violations, &|| {
+                                format!(
+                                    "KJ3;ideal_landed={};{}",
+                                    ideal.landed_version.describe(),
+                                    facts()
+                                )
+                            });
+                        }
+                    }
+                }
+            }
+        }
+    }
+}
+
+/// 一个根环槽的读数对真相：读成自证过的，它的 (txg, 实例代号) 必须等于真相里这个槽的根；读成自证不过的（全 0 的槽），真相里这个槽必须是空的；
+/// 读不出一律不合（内存盘对根环的槽只回 512 字节或全 0）。
+fn root_slot_reading_agrees_with_the_truth(
+    reading: RootRingSlotReading,
+    truth: Option<&RootChoicePublishMaterial>,
+) -> bool {
+    match reading {
+        RootRingSlotReading::SelfVerified(root) => truth.is_some_and(|material| {
+            root.checkpoint_txg.0 == material.version.checkpoint_txg
+                && root.instance.0 == material.version.instance
+        }),
+        RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => truth.is_none(),
+        RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => false,
+    }
+}
+
+// ── 枚举一个 (域, M, U) 格里的全部状态
+
+/// 一次挂载里写到哪了：下一条记录的计数器、下一个事务号、本实例上一条记录的链值、下一次发布的 txg。
+#[derive(Clone, Copy, Debug)]
+struct RootChoiceMountCursor {
+    instance: u32,
+    next_counter: u64,
+    next_transaction: u64,
+    previous_chain_value: Option<u32>,
+    next_checkpoint_txg: u64,
+}
+
+impl RootChoiceMountCursor {
+    fn after_the_publish_of(self, material: &RootChoicePublishMaterial) -> Self {
+        let last_slot = material.records.last().expect("每次发布至少一条记录");
+        let commit_records = material
+            .records
+            .iter()
+            .filter(|slot| {
+                slot.parsed
+                    .as_ref()
+                    .expect("合法历史的每条记录都解得出（生成时断言过）")
+                    .is_commit
+            })
+            .count();
+        Self {
+            instance: self.instance,
+            next_counter: self.next_counter
+                + u64::try_from(material.records.len()).expect("一次发布的记录条数不超过 2"),
+            next_transaction: if material.is_user_publish {
+                self.next_transaction
+                    + u64::try_from(commit_records).expect("一次发布的记录条数不超过 2")
+            } else {
+                self.next_transaction
+            },
+            previous_chain_value: Some(last_slot.chain_value_of_header),
+            next_checkpoint_txg: material.version.checkpoint_txg + 1,
+        }
+    }
+
+    fn plan_of_the_next_publish(
+        self,
+        domain: RootChoiceDomain,
+        publishes_started: u64,
+        record_count: usize,
+        commit_marker_bits: u32,
+        is_user_publish: bool,
+    ) -> RootChoicePublishPlan {
+        RootChoicePublishPlan {
+            domain,
+            instance: self.instance,
+            checkpoint_txg: self.next_checkpoint_txg,
+            publish_ordinal: publishes_started,
+            first_counter: self.next_counter,
+            first_transaction: self.next_transaction,
+            record_count,
+            commit_marker_bits,
+            previous_chain_value: self.previous_chain_value,
+            is_user_publish,
+        }
+    }
+}
+
+/// 走枚举的一方：收工作单元的起点，或判最后一次挂载崩溃点上的每个状态。
+trait RootChoiceVisitor {
+    /// 走到「第 `mount_number` 次挂载开始之前」（只在 `stop_before_mount` 点名的那一次挂载上触发，之后不再往下走）。
+    fn reach_the_start_of_a_mount(
+        &mut self,
+        world: &RootChoiceWorld,
+        mount_number: usize,
+        user_publishes_left: usize,
+    );
+
+    /// 最后一次挂载的崩溃点：`world` 是崩溃在途那次发布之前的盘，`crashed` 是它的崩溃点，`trace` 是这次挂载到这里为止的写法。
+    fn visit_a_final_state(
+        &mut self,
+        world: &RootChoiceWorld,
+        crashed: &RootChoiceCrashedPublish<'_>,
+        trace: &str,
+    );
+}
+
+struct RootChoiceEnumeration {
+    domain: RootChoiceDomain,
+    mounts: usize,
+    stop_before_mount: Option<usize>,
+}
+
+impl RootChoiceEnumeration {
+    /// 第 `mount_number` 次挂载（实例代号 = 它）：写行那次发布、暖机的空发布、用户发布 u_m 次，崩溃点在第 u_m 次用户发布里；
+    /// u_m = 0 时在写行或任一次暖机发布里。非最后一次挂载 u_m 取 0..=剩余的用户发布数，最后一次用完剩余的（登记 5.3）。
+    fn enumerate_a_mount(
+        &self,
+        world: &RootChoiceWorld,
+        mount_number: usize,
+        user_publishes_left: usize,
+        visitor: &mut dyn RootChoiceVisitor,
+    ) {
+        if self.stop_before_mount == Some(mount_number) {
+            visitor.reach_the_start_of_a_mount(world, mount_number, user_publishes_left);
+            return;
+        }
+        let is_the_last_mount = mount_number == self.mounts;
+        let overhead_txgs = root_choice_warm_up_plan(world.highest_checkpoint_txg_on_disk() + 1);
+        let first_user_txg = overhead_txgs.last().expect("写行那次发布至少有一次") + 1;
+        let user_publish_counts: Vec<usize> = if is_the_last_mount {
+            vec![user_publishes_left]
+        } else {
+            (0..=user_publishes_left).collect()
+        };
+        for user_publish_count in user_publish_counts {
+            let user_publishes_after_this_mount = user_publishes_left - user_publish_count;
+            let mut running_world = world.clone();
+            let mut cursor = RootChoiceMountCursor {
+                instance: u32::try_from(mount_number).expect("挂载次数不超过 3"),
+                next_counter: world.highest_landed_counter() + 1,
+                next_transaction: 1,
+                previous_chain_value: None,
+                next_checkpoint_txg: overhead_txgs[0],
+            };
+            let mut trace = format!(
+                "mount{mount_number}:overhead_txgs={}",
+                overhead_txgs
+                    .iter()
+                    .map(u64::to_string)
+                    .collect::<Vec<String>>()
+                    .join(",")
+            );
+            for overhead_txg in &overhead_txgs {
+                assert_eq!(
+                    cursor.next_checkpoint_txg, *overhead_txg,
+                    "开销发布的 txg 逐个加一"
+                );
+                let material = Arc::new(root_choice_publish_material(
+                    &cursor.plan_of_the_next_publish(
+                        self.domain,
+                        running_world.publishes_started,
+                        1,
+                        0,
+                        false,
+                    ),
+                ));
+                if user_publish_count == 0 {
+                    self.enumerate_the_crash_points(
+                        &running_world,
+                        &material,
+                        &trace,
+                        (mount_number, user_publishes_after_this_mount),
+                        visitor,
+                    );
+                }
+                running_world = running_world.with_publish_applied(&material, false);
+                cursor = cursor.after_the_publish_of(&material);
+            }
+            if user_publish_count > 0 {
+                assert_eq!(cursor.next_checkpoint_txg, first_user_txg);
+                trace.push_str(";user_publishes");
+                self.enumerate_user_publishes(
+                    &running_world,
+                    cursor,
+                    (
+                        mount_number,
+                        user_publish_count,
+                        user_publishes_after_this_mount,
+                    ),
+                    &trace,
+                    visitor,
+                );
+            }
+        }
+    }
+
+    /// 第 `user_publishes_in_the_mount`（≥ 1）次用户发布起，每次的 k 与提交标记摆法各一种；崩溃点在这一挂载的最后一次用户发布里。
+    fn enumerate_user_publishes(
+        &self,
+        running_world: &RootChoiceWorld,
+        cursor: RootChoiceMountCursor,
+        (mount_number, user_publishes_in_the_mount, user_publishes_after_this_mount): (
+            usize,
+            usize,
+            usize,
+        ),
+        trace: &str,
+        visitor: &mut dyn RootChoiceVisitor,
+    ) {
+        for record_count in self.domain.user_publish_record_counts() {
+            for commit_marker_bits in 0..(1u32 << (record_count - 1)) {
+                let material = Arc::new(root_choice_publish_material(
+                    &cursor.plan_of_the_next_publish(
+                        self.domain,
+                        running_world.publishes_started,
+                        *record_count,
+                        commit_marker_bits,
+                        true,
+                    ),
+                ));
+                let trace_with_this_publish = format!(
+                    "{trace}|txg{}k{record_count}b{commit_marker_bits}",
+                    material.version.checkpoint_txg
+                );
+                if user_publishes_in_the_mount == 1 {
+                    self.enumerate_the_crash_points(
+                        running_world,
+                        &material,
+                        &trace_with_this_publish,
+                        (mount_number, user_publishes_after_this_mount),
+                        visitor,
+                    );
+                } else {
+                    let applied_world = running_world.with_publish_applied(&material, true);
+                    self.enumerate_user_publishes(
+                        &applied_world,
+                        cursor.after_the_publish_of(&material),
+                        (
+                            mount_number,
+                            user_publishes_in_the_mount - 1,
+                            user_publishes_after_this_mount,
+                        ),
+                        &trace_with_this_publish,
+                        visitor,
+                    );
+                }
+            }
+        }
+    }
+
+    /// 崩在 `material` 这次发布的每个崩溃点上：最后一次挂载交给 `visitor` 判；之前的挂载接着枚举下一次挂载。
+    fn enumerate_the_crash_points(
+        &self,
+        running_world: &RootChoiceWorld,
+        material: &Arc<RootChoicePublishMaterial>,
+        trace: &str,
+        (mount_number, user_publishes_after_this_mount): (usize, usize),
+        visitor: &mut dyn RootChoiceVisitor,
+    ) {
+        for crash_point in root_choice_crash_points_of(material.records.len()) {
+            let crashed = RootChoiceCrashedPublish {
+                material,
+                crash_point,
+            };
+            if mount_number == self.mounts {
+                visitor.visit_a_final_state(running_world, &crashed, trace);
+            } else {
+                let crashed_world = running_world.with_crash_applied(&crashed, trace);
+                self.enumerate_a_mount(
+                    &crashed_world,
+                    mount_number + 1,
+                    user_publishes_after_this_mount,
+                    visitor,
+                );
+            }
+        }
+    }
+}
+
+// ── 一个格的计数：枚举、判、按工作单元分给几条线程
+
+struct RootChoiceUnit {
+    world: RootChoiceWorld,
+    mount_number: usize,
+    user_publishes_left: usize,
+}
+
+/// 工作单元：前面几次挂载（最多两次）的全部走法在崩溃之后得到的盘，每个单元接着枚举剩下的挂载。
+struct RootChoiceUnitCollector {
+    units: Vec<RootChoiceUnit>,
+}
+
+impl RootChoiceVisitor for RootChoiceUnitCollector {
+    fn reach_the_start_of_a_mount(
+        &mut self,
+        world: &RootChoiceWorld,
+        mount_number: usize,
+        user_publishes_left: usize,
+    ) {
+        self.units.push(RootChoiceUnit {
+            world: world.clone(),
+            mount_number,
+            user_publishes_left,
+        });
+    }
+
+    fn visit_a_final_state(
+        &mut self,
+        _world: &RootChoiceWorld,
+        _crashed: &RootChoiceCrashedPublish<'_>,
+        _trace: &str,
+    ) {
+        panic!("收工作单元的那一遍停在第 min(M, 3) 次挂载开始之前，走不到最后一次挂载的崩溃点");
+    }
+}
+
+struct RootChoiceStateJudge<'run> {
+    judgement: &'run RootChoiceJudgement,
+    tally: RootChoiceTally,
+    states_seen_in_this_unit: u64,
+    is_the_first_unit: bool,
+}
+
+impl RootChoiceVisitor for RootChoiceStateJudge<'_> {
+    fn reach_the_start_of_a_mount(
+        &mut self,
+        _world: &RootChoiceWorld,
+        _mount_number: usize,
+        _user_publishes_left: usize,
+    ) {
+        panic!("判状态的那一遍不设 stop_before_mount");
+    }
+
+    fn visit_a_final_state(
+        &mut self,
+        world: &RootChoiceWorld,
+        crashed: &RootChoiceCrashedPublish<'_>,
+        trace: &str,
+    ) {
+        let landing = root_choice_landing_of(world, Some(crashed));
+        let describe_state =
+            || format!("{}|{trace};{}", world.history.join("/"), crashed.describe());
+        let is_the_designated_state = self.is_the_first_unit && self.states_seen_in_this_unit == 0;
+        self.states_seen_in_this_unit += 1;
+        self.judgement.judge(
+            &landing,
+            &world.confirmed_user_publishes,
+            is_the_designated_state,
+            &mut self.tally,
+            &describe_state,
+        );
+    }
+}
+
+/// KA（登记 5.4）：根环读根环次序的前三个槽（区域 0 的槽 0、1、2）各取空或四条根之一，其余根槽全 0，journal 环全 0。
+const ROOT_CHOICE_ADVERSARIAL_ROOT_VERSIONS: [RootVersion; 4] = [
+    RootVersion {
+        checkpoint_txg: 100,
+        instance: 1,
+    },
+    RootVersion {
+        checkpoint_txg: 100,
+        instance: 2,
+    },
+    RootVersion {
+        checkpoint_txg: 101,
+        instance: 1,
+    },
+    RootVersion {
+        checkpoint_txg: 101,
+        instance: 2,
+    },
+];
+const ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS: usize = 3;
+
+fn root_choice_tally_of_the_adversarial_root_rings(
+    implementation: RootChoiceImplementation,
+) -> RootChoiceTally {
+    let domain = RootChoiceDomain::AdversarialRootRing;
+    let judgement = RootChoiceJudgement::new(domain, implementation);
+    let root_materials: Vec<Arc<RootChoicePublishMaterial>> = ROOT_CHOICE_ADVERSARIAL_ROOT_VERSIONS
+        .iter()
+        .enumerate()
+        .map(|(version_index, version)| {
+            Arc::new(root_material_without_records(
+                version.instance,
+                version.checkpoint_txg,
+                NewRootSegment::off_timeline(
+                    1 + u64::try_from(version_index).expect("四条根"),
+                    version.checkpoint_txg,
+                ),
+            ))
+        })
+        .collect();
+    let choices_per_slot = root_materials.len() + 1;
+    let mut tally = RootChoiceTally::new();
+    let state_count =
+        choices_per_slot.pow(u32::try_from(ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS).expect("三个槽"));
+    for state_number in 0..state_count {
+        let mut world = RootChoiceWorld {
+            domain,
+            root_slots: vec![
+                None;
+                usize::try_from(ROOT_CHOICE_REGION_COUNT * domain.slots_per_region())
+                    .expect("根槽数不超过 24")
+            ],
+            landed_records: Vec::new(),
+            confirmed_user_publishes: Vec::new(),
+            publishes_started: 0,
+            history: Vec::new(),
+        };
+        let mut remaining_choices = state_number;
+        let mut choice_texts = Vec::new();
+        for slot_index in (0..ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS).rev() {
+            let choice = remaining_choices % choices_per_slot;
+            remaining_choices /= choices_per_slot;
+            if choice > 0 {
+                world.root_slots[slot_index] = Some(Arc::clone(&root_materials[choice - 1]));
+                choice_texts.push(root_materials[choice - 1].version.describe());
+            } else {
+                choice_texts.push("empty".to_string());
+            }
+        }
+        choice_texts.reverse();
+        let landing = root_choice_landing_of(&world, None);
+        let describe_state = || format!("ka_region0_slots012=[{}]", choice_texts.join(","));
+        judgement.judge(&landing, &[], false, &mut tally, &describe_state);
+    }
+    tally
+}
+
+/// 一个 (域, M, U) 格上、一种实现下全部状态的计数。KA 单线程走 125 个状态；合法历史的三个域按工作单元分给几条线程。
+fn root_choice_tally_of_a_cell(
+    domain: RootChoiceDomain,
+    cell: RootChoiceCell,
+    implementation: RootChoiceImplementation,
+    thread_count: usize,
+) -> RootChoiceTally {
+    if !domain.is_a_legal_history() {
+        return root_choice_tally_of_the_adversarial_root_rings(implementation);
+    }
+    let collecting = RootChoiceEnumeration {
+        domain,
+        mounts: cell.mounts,
+        stop_before_mount: Some(cell.mounts.min(3)),
+    };
+    let mut collector = RootChoiceUnitCollector { units: Vec::new() };
+    collecting.enumerate_a_mount(
+        &RootChoiceWorld::right_after_mkfs(domain),
+        1,
+        cell.user_publishes,
+        &mut collector,
+    );
+    let judgement = RootChoiceJudgement::new(domain, implementation);
+    let continuing = RootChoiceEnumeration {
+        domain,
+        mounts: cell.mounts,
+        stop_before_mount: None,
+    };
+    root_choice_tally_of_work_units(&collector.units, thread_count, &|unit_index, unit| {
+        let mut judge = RootChoiceStateJudge {
+            judgement: &judgement,
+            tally: RootChoiceTally::new(),
+            states_seen_in_this_unit: 0,
+            is_the_first_unit: unit_index == 0,
+        };
+        continuing.enumerate_a_mount(
+            &unit.world,
+            unit.mount_number,
+            unit.user_publishes_left,
+            &mut judge,
+        );
+        judge.tally
+    })
+}
+
+// ── 钉绝对值（登记第七节乙类：出自 `anchors_r3_k.py` 独立生成器，常量写在断言右边；不符走 TV4）
+
+#[derive(Clone, Copy, Debug)]
+struct PinnedRootChoiceCell {
+    domain: RootChoiceDomain,
+    mounts: usize,
+    user_publishes: usize,
+    states: u64,
+    confirmed_states: u64,
+    first_self_verified_root_is_not_the_newest: u64,
+    same_txg_different_instance_roots: u64,
+    higher_instance_lower_txg_roots: u64,
+    two_or_more_instances: u64,
+    ideal_recovery_applied: u64,
+    above_chosen_root_maximum: usize,
+    largest_counter: u64,
+    /// KF1（所选根取版本最小的根）下 KJ1、KJ2 必须读到的数。
+    oldest_root_kj1: u64,
+    oldest_root_kj2: u64,
+}
+
+fn pinned_root_choice_cell(
+    domain: RootChoiceDomain,
+    cell: RootChoiceCell,
+) -> Option<&'static PinnedRootChoiceCell> {
+    PINNED_ROOT_CHOICE_CELLS.iter().find(|pinned| {
+        pinned.domain == domain
+            && pinned.mounts == cell.mounts
+            && pinned.user_publishes == cell.user_publishes
+    })
+}
+
+/// KA 上各种本地择根序的 KJ2 违例数（登记第七节乙类 KA：今天的序 0、平局取低 36、实例代号为主 18、比较不看实例 18、取第一个有效者 52），
+/// 出自 `anchors_r3_ka.py`。
+const fn pinned_root_choice_adversarial_kj2_violations(rule: LocalRootChoiceRule) -> u64 {
+    match rule {
+        LocalRootChoiceRule::NewestWins => 0,
+        LocalRootChoiceRule::TieTakesTheLowerInstance => 36,
+        LocalRootChoiceRule::InstanceFirstThenTxg => 18,
+        LocalRootChoiceRule::IgnoresTheInstance => 18,
+        LocalRootChoiceRule::FirstSelfVerifiedWins => 52,
+    }
+}
+
+/// KA 的状态数 125 与其中没有一条自证过的根的 1 态（登记第七节乙类 KA：`choose_root` 交回空，KJ0 在这一态记一条）。
+const PINNED_ROOT_CHOICE_ADVERSARIAL_STATES: u64 = 125;
+
+/// 每个跑到的合法历史格，装置数出的状态数与几样计数等于独立生成器的数；不符走作废条款 TV4（这些量只看生成器与真相，与被测实现无关）。
+fn assert_the_pinned_root_choice_cell(
+    domain: RootChoiceDomain,
+    cell: RootChoiceCell,
+    tally: &RootChoiceTally,
+) {
+    let pinned = pinned_root_choice_cell(domain, cell)
+        .expect("每个跑到的合法历史格都在登记第七节乙类的表里");
+    let place = format!(
+        "域 {} M = {} U = {}",
+        domain.label(),
+        cell.mounts,
+        cell.user_publishes
+    );
+    assert_eq!(
+        tally.get(RootChoiceCounter::States),
+        pinned.states,
+        "TV4：{place} 状态数"
+    );
+    assert_eq!(
+        tally.get(RootChoiceCounter::ConfirmedStates),
+        pinned.confirmed_states,
+        "TV4：{place} 有已确认发布的态数"
+    );
+    assert_eq!(
+        tally.get(RootChoiceCounter::FirstSelfVerifiedRootIsNotTheNewest),
+        pinned.first_self_verified_root_is_not_the_newest,
+        "TV4：{place} 读根环次序里第一个自证过的根不是版本最大那条的态数"
+    );
+    assert_eq!(
+        tally.get(RootChoiceCounter::SameTxgDifferentInstanceRoots),
+        pinned.same_txg_different_instance_roots,
+        "TV4：{place} 根环里同 txg 不同实例同在的态数"
+    );
+    assert_eq!(
+        tally.get(RootChoiceCounter::HigherInstanceLowerTxgRoots),
+        pinned.higher_instance_lower_txg_roots,
+        "TV4：{place} 实例代号更高而 txg 更低的一对根同在的态数"
+    );
+    assert_eq!(
+        tally.get(RootChoiceCounter::TwoOrMoreInstancesInTheRing),
+        pinned.two_or_more_instances,
+        "TV4：{place} 根环里有两个以上实例的根的态数"
+    );
+    assert_eq!(
+        tally.get(RootChoiceCounter::IdealRecoveryApplied),
+        pinned.ideal_recovery_applied,
+        "TV4：{place} 理想恢复施加过至少一次发布的态数"
+    );
+    assert_eq!(
+        tally.get(RootChoiceCounter::StatesWithoutASelfVerifiedRoot),
+        0,
+        "TV4：{place} 根环里没有一条自证过的根的态数（合法历史里第 0 代根至少还有一份）"
+    );
+    assert_eq!(
+        tally.above_newest_root_maximum, pinned.above_chosen_root_maximum,
+        "TV4：{place} 真相里版本最大的根之上同实例的记录条数的最大值"
+    );
+    assert_eq!(
+        tally.largest_counter, pinned.largest_counter,
+        "TV4：{place} 盘上读得出的最大计数器"
+    );
+}
+
+/// 本地常量逐个对着实现侧的同名常量与函数回比（登记第三节回比表；不符走停机条款 TH2）。
+fn cross_check_root_choice_local_constants_against_the_implementation() {
+    assert_eq!(
+        ROOT_CHOICE_REGION_COUNT,
+        singlefs_format::ROOT_RING_REGIONS,
+        "TH2：根环区域数"
+    );
+    assert_eq!(
+        ROOT_CHOICE_BASE_SLOT,
+        singlefs_format::ROOT_RING_BASE_SLOT,
+        "TH2：根环基槽"
+    );
+    assert_eq!(
+        ROOT_CHOICE_PRIME_STEP,
+        singlefs_format::ROOT_RING_PRIME_STEP,
+        "TH2：区域步长"
+    );
+    assert_eq!(
+        ROOT_CHOICE_CHUNK_BYTES,
+        singlefs_format::ROOT_RING_CHUNK_BYTES,
+        "TH2：区域 chunk 字节数"
+    );
+    for region in 0..ROOT_CHOICE_REGION_COUNT {
+        assert_eq!(
+            root_choice_region_start_in_bytes(region),
+            singlefs_core::root_ring::region_start(region).0,
+            "TH2：本地算的区域起点与 root_ring::region_start 不等（区域 {region}）"
+        );
+    }
+    assert_eq!(
+        ROOT_CHOICE_REGION_DEVICES,
+        singlefs_format::ROOT_RING_REGION_DEVICES,
+        "TH2：根环区域的设备归属（格式常量）"
+    );
+    assert_eq!(
+        ROOT_CHOICE_REGION_DEVICES.map(DeviceIdentity),
+        singlefs_core::make_filesystem::FIRST_VERSION_REGION_DEVICES,
+        "TH2：根环区域的设备归属（mkfs 第一版）"
+    );
+    assert_eq!(
+        ROOT_CHOICE_SLOTS_PER_REGION_WIDE,
+        RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
+        "TH2：mkfs 取的每区槽数"
+    );
+    assert_eq!(
+        ROOT_CHOICE_SLOTS_PER_REGION_NARROW,
+        singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
+        "TH2：每区槽数下限"
+    );
+    assert_eq!(
+        ROOT_CHOICE_MKFS_INSTANCE,
+        singlefs_core::make_filesystem::MKFS_INSTANCE_GENERATION.0,
+        "TH2：mkfs 第 0 代根的实例代号"
+    );
+    assert!(
+        u64::try_from(ROOT_CHOICE_ROOT_SLOT_BYTES).expect("512 装得进 u64")
+            >= singlefs_format::ROOT_RECORD_BYTES,
+        "TH2：根槽宽装得下根记录"
+    );
+    assert_eq!(
+        u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES),
+        singlefs_format::FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
+        "TH2：固定结构槽距"
+    );
+    for domain in RootChoiceDomain::ALL {
+        for checkpoint_txg in 0..=40 {
+            assert_the_local_root_slot_arithmetic_against_the_implementation(
+                *domain,
+                checkpoint_txg,
+            );
+        }
+    }
+    // 新池第一次挂载：写行 txg 1、暖机 txg 2（开销发布两次），第一个用户发布 txg 3（登记第七节甲类、第三节回比表第 5 行）。
+    let first_mount_plan = root_choice_warm_up_plan(1);
+    assert_eq!(
+        u64::try_from(first_mount_plan.len()).expect("开销发布不超过 4 次"),
+        singlefs_format::WARM_UP_EMPTY_PUBLISHES,
+        "TH2 / TF9：新池第一次挂载的开销发布次数与 WARM_UP_EMPTY_PUBLISHES 不等"
+    );
+    assert_eq!(
+        first_mount_plan,
+        [1, 2],
+        "TF9：新池第一次挂载写行 txg 1、暖机 txg 2"
+    );
+    assert_eq!(
+        first_mount_plan.last().expect("至少一次开销发布") + 1,
+        singlefs_format::NEW_POOL_FILE_CREATION_TXG,
+        "TH2 / TF9：新池第一个用户发布的 txg 与 NEW_POOL_FILE_CREATION_TXG 不等"
+    );
+}
+const PINNED_ROOT_CHOICE_CELLS: [PinnedRootChoiceCell; 36] = [
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 1,
+        user_publishes: 0,
+        states: 10,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 6,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 6,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 2,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 6,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 1,
+        user_publishes: 1,
+        states: 39,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 39,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 39,
+        ideal_recovery_applied: 21,
+        above_chosen_root_maximum: 2,
+        largest_counter: 4,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 39,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 1,
+        user_publishes: 2,
+        states: 117,
+        confirmed_states: 117,
+        first_self_verified_root_is_not_the_newest: 117,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 117,
+        ideal_recovery_applied: 63,
+        above_chosen_root_maximum: 2,
+        largest_counter: 6,
+        oldest_root_kj1: 117,
+        oldest_root_kj2: 117,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 1,
+        user_publishes: 3,
+        states: 351,
+        confirmed_states: 351,
+        first_self_verified_root_is_not_the_newest: 351,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 351,
+        ideal_recovery_applied: 189,
+        above_chosen_root_maximum: 2,
+        largest_counter: 8,
+        oldest_root_kj1: 351,
+        oldest_root_kj2: 351,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 1,
+        user_publishes: 4,
+        states: 1053,
+        confirmed_states: 1053,
+        first_self_verified_root_is_not_the_newest: 1053,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 1053,
+        ideal_recovery_applied: 567,
+        above_chosen_root_maximum: 2,
+        largest_counter: 10,
+        oldest_root_kj1: 1053,
+        oldest_root_kj2: 1053,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 2,
+        user_publishes: 0,
+        states: 125,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 109,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 109,
+        ideal_recovery_applied: 57,
+        above_chosen_root_maximum: 1,
+        largest_counter: 4,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 109,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 2,
+        user_publishes: 1,
+        states: 780,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 780,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 780,
+        ideal_recovery_applied: 411,
+        above_chosen_root_maximum: 2,
+        largest_counter: 6,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 780,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 2,
+        user_publishes: 2,
+        states: 4401,
+        confirmed_states: 2880,
+        first_self_verified_root_is_not_the_newest: 4401,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 4401,
+        ideal_recovery_applied: 2376,
+        above_chosen_root_maximum: 2,
+        largest_counter: 9,
+        oldest_root_kj1: 2880,
+        oldest_root_kj2: 4401,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 2,
+        user_publishes: 3,
+        states: 16281,
+        confirmed_states: 16281,
+        first_self_verified_root_is_not_the_newest: 16281,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 16281,
+        ideal_recovery_applied: 8694,
+        above_chosen_root_maximum: 2,
+        largest_counter: 11,
+        oldest_root_kj1: 16281,
+        oldest_root_kj2: 16281,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 2,
+        user_publishes: 4,
+        states: 62127,
+        confirmed_states: 62127,
+        first_self_verified_root_is_not_the_newest: 62127,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 62127,
+        ideal_recovery_applied: 33210,
+        above_chosen_root_maximum: 2,
+        largest_counter: 13,
+        oldest_root_kj1: 62127,
+        oldest_root_kj2: 62127,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 3,
+        user_publishes: 0,
+        states: 1480,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 1416,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 1416,
+        ideal_recovery_applied: 741,
+        above_chosen_root_maximum: 1,
+        largest_counter: 7,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 1416,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 3,
+        user_publishes: 1,
+        states: 13770,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 13770,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 13770,
+        ideal_recovery_applied: 7266,
+        above_chosen_root_maximum: 2,
+        largest_counter: 9,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 13770,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 3,
+        user_publishes: 2,
+        states: 92565,
+        confirmed_states: 46890,
+        first_self_verified_root_is_not_the_newest: 92565,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 92565,
+        ideal_recovery_applied: 49068,
+        above_chosen_root_maximum: 2,
+        largest_counter: 12,
+        oldest_root_kj1: 46890,
+        oldest_root_kj2: 92565,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 3,
+        user_publishes: 3,
+        states: 512514,
+        confirmed_states: 453195,
+        first_self_verified_root_is_not_the_newest: 512514,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 512514,
+        ideal_recovery_applied: 274563,
+        above_chosen_root_maximum: 2,
+        largest_counter: 14,
+        oldest_root_kj1: 453195,
+        oldest_root_kj2: 512514,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::BigRing,
+        mounts: 3,
+        user_publishes: 4,
+        states: 2225961,
+        confirmed_states: 2225961,
+        first_self_verified_root_is_not_the_newest: 2225961,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 2225961,
+        ideal_recovery_applied: 1186326,
+        above_chosen_root_maximum: 2,
+        largest_counter: 16,
+        oldest_root_kj1: 2225961,
+        oldest_root_kj2: 2225961,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::SmallRingNine,
+        mounts: 1,
+        user_publishes: 0,
+        states: 10,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 6,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 6,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 2,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 6,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::SmallRingNine,
+        mounts: 1,
+        user_publishes: 1,
+        states: 39,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 39,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 39,
+        ideal_recovery_applied: 21,
+        above_chosen_root_maximum: 2,
+        largest_counter: 4,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 39,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::SmallRingNine,
+        mounts: 1,
+        user_publishes: 2,
+        states: 117,
+        confirmed_states: 117,
+        first_self_verified_root_is_not_the_newest: 117,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 117,
+        ideal_recovery_applied: 63,
+        above_chosen_root_maximum: 2,
+        largest_counter: 6,
+        oldest_root_kj1: 117,
+        oldest_root_kj2: 117,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::SmallRingNine,
+        mounts: 2,
+        user_publishes: 0,
+        states: 125,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 109,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 109,
+        ideal_recovery_applied: 57,
+        above_chosen_root_maximum: 1,
+        largest_counter: 4,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 109,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::SmallRingNine,
+        mounts: 2,
+        user_publishes: 1,
+        states: 780,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 780,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 780,
+        ideal_recovery_applied: 411,
+        above_chosen_root_maximum: 2,
+        largest_counter: 6,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 780,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::SmallRingNine,
+        mounts: 2,
+        user_publishes: 2,
+        states: 4401,
+        confirmed_states: 2880,
+        first_self_verified_root_is_not_the_newest: 4401,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 4401,
+        ideal_recovery_applied: 2376,
+        above_chosen_root_maximum: 2,
+        largest_counter: 9,
+        oldest_root_kj1: 2880,
+        oldest_root_kj2: 4401,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 0,
+        states: 10,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 6,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 6,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 2,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 6,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 1,
+        states: 5,
+        confirmed_states: 0,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 5,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 3,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 5,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 2,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 5,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 4,
+        oldest_root_kj1: 5,
+        oldest_root_kj2: 5,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 3,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 5,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 5,
+        oldest_root_kj1: 5,
+        oldest_root_kj2: 5,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 4,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 5,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 6,
+        oldest_root_kj1: 5,
+        oldest_root_kj2: 5,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 5,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 5,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 7,
+        oldest_root_kj1: 5,
+        oldest_root_kj2: 5,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 6,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 5,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 8,
+        oldest_root_kj1: 5,
+        oldest_root_kj2: 5,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 7,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 5,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 9,
+        oldest_root_kj1: 5,
+        oldest_root_kj2: 5,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 8,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 5,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 10,
+        oldest_root_kj1: 5,
+        oldest_root_kj2: 5,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 9,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 5,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 11,
+        oldest_root_kj1: 5,
+        oldest_root_kj2: 5,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 10,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 4,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 4,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 12,
+        oldest_root_kj1: 4,
+        oldest_root_kj2: 4,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 11,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 1,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 0,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 13,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 0,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 12,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 0,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 14,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 0,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 13,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 0,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 15,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 0,
+    },
+    PinnedRootChoiceCell {
+        domain: RootChoiceDomain::WrappingRootRing,
+        mounts: 1,
+        user_publishes: 14,
+        states: 5,
+        confirmed_states: 5,
+        first_self_verified_root_is_not_the_newest: 5,
+        same_txg_different_instance_roots: 0,
+        higher_instance_lower_txg_roots: 0,
+        two_or_more_instances: 0,
+        ideal_recovery_applied: 3,
+        above_chosen_root_maximum: 1,
+        largest_counter: 16,
+        oldest_root_kj1: 0,
+        oldest_root_kj2: 0,
+    },
+];
+
+// ── 阳性对照（登记 5.7；新的几条读实现的结局或调三步，不进启动自检，放进单测，产物里打 `name=control` 行）
+
+#[derive(Clone, Debug)]
+struct RootChoiceControlReading {
+    criterion: RootChoiceCriterion,
+    measured_where: String,
+    value: u64,
+    must_see: String,
+    is_red_as_required: bool,
+}
+
+fn root_choice_reading_that_must_equal(
+    criterion: RootChoiceCriterion,
+    measured_where: String,
+    value: u64,
+    expected: u64,
+) -> RootChoiceControlReading {
+    RootChoiceControlReading {
+        criterion,
+        measured_where,
+        value,
+        must_see: format!("equals_{expected}"),
+        is_red_as_required: value == expected,
+    }
+}
+
+fn root_choice_place_of(domain: RootChoiceDomain, cell: RootChoiceCell) -> String {
+    format!(
+        "domain={} ring_slots={} slots_per_region={} mounts={} user_publishes={}",
+        domain.label(),
+        domain.ring().ring_slots,
+        domain.slots_per_region(),
+        cell.mounts,
+        cell.user_publishes
+    )
+}
+
+/// 每条判据在每个 (域, M, U) 格上的对照读数：KJ0 在 K 域 M = 1、U = 0 的指定一态上 panic（违例数 1）；KJ1、KJ2 在 KF1 下等于第七节乙类 KF1 那两列；
+/// KJ1、KJ2、KJ3 在 KF2 下各等于「有已确认发布的态数」；KJ2 在 KA 上本地写的几种择根序下等于 36、18、18、52（今天的序 0）。
+fn root_choice_control_readings(
+    selected_cells: &[(RootChoiceDomain, Vec<RootChoiceCell>)],
+    thread_count: usize,
+) -> Vec<RootChoiceControlReading> {
+    let mut readings = Vec::new();
+    for (domain, cells) in selected_cells {
+        for cell in cells {
+            let place = root_choice_place_of(*domain, *cell);
+            if !domain.is_a_legal_history() {
+                for rule in LocalRootChoiceRule::ALL {
+                    let tally = root_choice_tally_of_a_cell(
+                        *domain,
+                        *cell,
+                        RootChoiceImplementation::LocalChoice(rule),
+                        thread_count,
+                    );
+                    readings.push(root_choice_reading_that_must_equal(
+                        RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
+                        format!("{place} local_choice={}", rule.label()),
+                        tally.get(RootChoiceCounter::Kj2Violations),
+                        pinned_root_choice_adversarial_kj2_violations(rule),
+                    ));
+                }
+                continue;
+            }
+            let pinned = pinned_root_choice_cell(*domain, *cell)
+                .expect("每个跑到的合法历史格都在登记第七节乙类的表里");
+            if *domain == RootChoiceDomain::BigRing && cell.mounts == 1 && cell.user_publishes == 0
+            {
+                let tally = root_choice_tally_of_a_cell(
+                    *domain,
+                    *cell,
+                    RootChoiceImplementation::PanicsOnTheDesignatedState,
+                    thread_count,
+                );
+                readings.push(root_choice_reading_that_must_equal(
+                    RootChoiceCriterion::Kj0Defined,
+                    format!("{place} fake_outcome=panics_on_the_designated_state"),
+                    tally.get(RootChoiceCounter::Kj0Violations),
+                    1,
+                ));
+            }
+            let kf1 = root_choice_tally_of_a_cell(
+                *domain,
+                *cell,
+                RootChoiceImplementation::ChoosesTheOldestRoot,
+                thread_count,
+            );
+            for (criterion, counter, expected) in [
+                (
+                    RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
+                    RootChoiceCounter::Kj1Violations,
+                    pinned.oldest_root_kj1,
+                ),
+                (
+                    RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
+                    RootChoiceCounter::Kj2Violations,
+                    pinned.oldest_root_kj2,
+                ),
+            ] {
+                readings.push(root_choice_reading_that_must_equal(
+                    criterion,
+                    format!("{place} fake_outcome=KF1_chooses_the_oldest_root"),
+                    kf1.get(counter),
+                    expected,
+                ));
+            }
+            let kf2 = root_choice_tally_of_a_cell(
+                *domain,
+                *cell,
+                RootChoiceImplementation::LandsOneVersionBeforeTheLastConfirmed,
+                thread_count,
+            );
+            for (criterion, counter) in [
+                (
+                    RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
+                    RootChoiceCounter::Kj1Violations,
+                ),
+                (
+                    RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
+                    RootChoiceCounter::Kj2Violations,
+                ),
+                (
+                    RootChoiceCriterion::Kj3LandsOnTheIdealRecovery,
+                    RootChoiceCounter::Kj3Violations,
+                ),
+            ] {
+                readings.push(root_choice_reading_that_must_equal(
+                    criterion,
+                    format!("{place} fake_outcome=KF2_lands_one_version_before_the_last_confirmed"),
+                    kf2.get(counter),
+                    pinned.confirmed_states,
+                ));
+            }
+        }
+    }
+    readings
+}
+
+// ── 产物行
+
+/// `name=root_choice_states` 行里数的量，按这个次序逐个写 `名字=数`。
+const ROOT_CHOICE_COUNTERS_OF_THE_STATES_ROW: [RootChoiceCounter; 18] = [
+    RootChoiceCounter::States,
+    RootChoiceCounter::ConfirmedStates,
+    RootChoiceCounter::FirstSelfVerifiedRootIsNotTheNewest,
+    RootChoiceCounter::SameTxgDifferentInstanceRoots,
+    RootChoiceCounter::HigherInstanceLowerTxgRoots,
+    RootChoiceCounter::TwoOrMoreInstancesInTheRing,
+    RootChoiceCounter::IdealRecoveryApplied,
+    RootChoiceCounter::AboveNewestRootOverLimit,
+    RootChoiceCounter::AboveNewestRootAtLimit,
+    RootChoiceCounter::AboveChosenRootOverLimit,
+    RootChoiceCounter::AboveChosenRootAtLimit,
+    RootChoiceCounter::StatesWithoutASelfVerifiedRoot,
+    RootChoiceCounter::PanickedStates,
+    RootChoiceCounter::ChoseNothingStates,
+    RootChoiceCounter::ReplayFailedStates,
+    RootChoiceCounter::ReplayAppliedStates,
+    RootChoiceCounter::SlotReadingsChecked,
+    RootChoiceCounter::SlotReadingsDifferingFromTheTruth,
+];
+
+/// 一个 (域, M, U) 格的全部行：状态数与几样计数、每条判据一行违例数（KA 只判 KJ0、KJ2，另两条不出行）。
+fn emit_the_rows_of_a_root_choice_cell(
+    emitter: &mut Emitter,
+    domain: RootChoiceDomain,
+    cell: RootChoiceCell,
+    tally: &RootChoiceTally,
+) {
+    let place = root_choice_place_of(domain, cell);
+    let counter_fields: Vec<String> = ROOT_CHOICE_COUNTERS_OF_THE_STATES_ROW
+        .iter()
+        .map(|counter| format!("{}={}", counter.label(), tally.get(*counter)))
+        .collect();
+    emitter.result(&format!(
+        "name=root_choice_states {place} {} in_flight_limit={} above_newest_root_max={} above_chosen_root_max={} confirmed_publishes_max={} largest_counter={} slot_reading_witness={}",
+        counter_fields.join(" "),
+        domain.ring().local_in_flight_limit(),
+        tally.above_newest_root_maximum,
+        tally.above_chosen_root_maximum,
+        tally.confirmed_publishes_maximum,
+        tally.largest_counter,
+        tally.witness(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
+    ));
+    let mut violations_row =
+        |criterion: RootChoiceCriterion, violations: RootChoiceCounter, judged: u64| {
+            emitter.result(&format!(
+                "name=violations {place} criterion={} violations={} judged={judged} witness={}",
+                criterion.label(),
+                tally.get(violations),
+                tally.witness(violations)
+            ));
+        };
+    violations_row(
+        RootChoiceCriterion::Kj0Defined,
+        RootChoiceCounter::Kj0Violations,
+        tally.get(RootChoiceCounter::States),
+    );
+    if domain.is_a_legal_history() {
+        violations_row(
+            RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
+            RootChoiceCounter::Kj1Violations,
+            tally.get(RootChoiceCounter::Kj1Judged),
+        );
+    }
+    violations_row(
+        RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
+        RootChoiceCounter::Kj2Violations,
+        tally.get(RootChoiceCounter::Kj2Judged),
+    );
+    if domain.is_a_legal_history() {
+        violations_row(
+            RootChoiceCriterion::Kj3LandsOnTheIdealRecovery,
+            RootChoiceCounter::Kj3Violations,
+            tally.get(RootChoiceCounter::Kj3Judged),
+        );
+    }
+}
+
+/// 一个域扫完之后的判决行（各判据在这个域全部格上的合计），并记作废与停机。
+fn emit_the_verdict_of_a_root_choice_domain(
+    emitter: &mut Emitter,
+    findings: &mut RunFindings,
+    domain: RootChoiceDomain,
+    cells: &[RootChoiceCell],
+    total: &RootChoiceTally,
+) {
+    let largest_mounts = cells.iter().map(|cell| cell.mounts).max().unwrap_or(0);
+    let largest_user_publishes = cells
+        .iter()
+        .map(|cell| cell.user_publishes)
+        .max()
+        .unwrap_or(0);
+    // 登记作废条款 TV10：域是空的（合法历史三个域合起来没有一个已确认的态，KJ1 一态都没判；KA 上同 txg 不同实例同在为 0）。
+    let domain_is_nonempty = if domain.is_a_legal_history() {
+        total.get(RootChoiceCounter::ConfirmedStates) > 0
+    } else {
+        total.get(RootChoiceCounter::SameTxgDifferentInstanceRoots) > 0
+            && total.get(RootChoiceCounter::HigherInstanceLowerTxgRoots) > 0
+    };
+    if !domain_is_nonempty {
+        findings.void_conditions.push(format!(
+            "TV10 {} 域是空的（合法历史域没有一个已确认发布的态；KA 上同 txg 不同实例同在与实例代号更高而 txg 更低同在有一样为 0），这个域的 0 违例不算数",
+            domain.label()
+        ));
+    }
+    // 登记停机条款 TH4：读槽对拍不一致，不当 KJ 的违例记。
+    if total.get(RootChoiceCounter::SlotReadingsDifferingFromTheTruth) > 0 {
+        findings.halt_conditions.push(format!(
+            "TH4 {} 域里 read_root_ring_slot 交回的与生成时的真相不同的槽数 {}（见证 {}）",
+            domain.label(),
+            total.get(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
+            total.witness(RootChoiceCounter::SlotReadingsDifferingFromTheTruth)
+        ));
+    }
+    // 登记第七节甲类、失败条款 TF9：合法历史三个域里今天的 choose_root 交回空、replay_journal 报错的态数是 0。
+    if domain.is_a_legal_history()
+        && total.get(RootChoiceCounter::ChoseNothingStates)
+            + total.get(RootChoiceCounter::ReplayFailedStates)
+            > 0
+    {
+        findings.halt_conditions.push(format!(
+            "TF9 {} 域里 choose_root 交回空的态数 {}、replay_journal 报错的态数 {}（登记第七节甲类锚点都是 0）",
+            domain.label(),
+            total.get(RootChoiceCounter::ChoseNothingStates),
+            total.get(RootChoiceCounter::ReplayFailedStates)
+        ));
+    }
+    emitter.result(&format!(
+        "name=verdict domain={} ring_slots={} slots_per_region={} largest_mounts={largest_mounts} largest_user_publishes={largest_user_publishes} criteria=root_choice states={} confirmed_states={} kj0_violations={} panicked={} choose_root_returned_none={} replay_failed={} kj1_violations={} kj1_judged={} kj2_violations={} kj2_judged={} kj3_violations={} kj3_judged={} slot_readings_differing_from_the_truth={} domain_is_nonempty={domain_is_nonempty}",
+        domain.label(),
+        domain.ring().ring_slots,
+        domain.slots_per_region(),
+        total.get(RootChoiceCounter::States),
+        total.get(RootChoiceCounter::ConfirmedStates),
+        total.get(RootChoiceCounter::Kj0Violations),
+        total.get(RootChoiceCounter::PanickedStates),
+        total.get(RootChoiceCounter::ChoseNothingStates),
+        total.get(RootChoiceCounter::ReplayFailedStates),
+        total.get(RootChoiceCounter::Kj1Violations),
+        total.get(RootChoiceCounter::Kj1Judged),
+        total.get(RootChoiceCounter::Kj2Violations),
+        total.get(RootChoiceCounter::Kj2Judged),
+        total.get(RootChoiceCounter::Kj3Violations),
+        total.get(RootChoiceCounter::Kj3Judged),
+        total.get(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
+    ));
+}
+
+// ── 命令行与主流程（`--family root_choice`，只在 `--profile r3`）
+
+struct RootChoiceSelection {
+    bounds: BoundsTier,
+    only: Option<Vec<String>>,
+    largest_user_publishes_cap: Option<usize>,
+}
+
+const ROOT_CHOICE_FAMILY_LABEL: &str = "root_choice";
+
+fn arguments_name_the_root_choice_family(arguments: &[String]) -> bool {
+    arguments
+        .windows(2)
+        .any(|pair| pair[0] == "--family" && pair[1] == ROOT_CHOICE_FAMILY_LABEL)
+}
+
+/// `--bounds segment1|segment2|light`（这一族 segment1 与 segment2 同界、`light` 是调试用的小界）、`--family root_choice`、
+/// `--only K,K9,KW,KA`、`--max-u <U>`（每个格的 U 封顶，只给调试用）。
+fn parse_the_root_choice_command_line(arguments: &[String]) -> Result<RootChoiceSelection, String> {
+    let mut bounds: Option<BoundsTier> = None;
+    let mut only: Option<Vec<String>> = None;
+    let mut largest_user_publishes_cap: Option<usize> = None;
+    let mut position = 0;
+    while position < arguments.len() {
+        let option = arguments[position].as_str();
+        let value = arguments
+            .get(position + 1)
+            .ok_or_else(|| format!("{option} 后面缺一个值"))?
+            .as_str();
+        match option {
+            "--bounds" => {
+                bounds = Some(match value {
+                    "segment1" => BoundsTier::SegmentOne,
+                    "segment2" => BoundsTier::SegmentTwo,
+                    "light" => BoundsTier::Light,
+                    unknown_tier => return Err(format!("不认识的界：{unknown_tier}")),
+                });
+            }
+            "--family" => {
+                if value != ROOT_CHOICE_FAMILY_LABEL {
+                    return Err(format!("不认识的族：{value}"));
+                }
+            }
+            "--only" => {
+                let names: Vec<String> = value.split(',').map(str::to_string).collect();
+                for name in &names {
+                    if !RootChoiceDomain::ALL
+                        .iter()
+                        .any(|domain| domain.label() == name)
+                    {
+                        return Err(format!("不认识的域：{name}（这一族的域是 K、K9、KW、KA）"));
+                    }
+                }
+                only = Some(names);
+            }
+            "--max-u" => {
+                largest_user_publishes_cap = Some(
+                    value
+                        .parse::<usize>()
+                        .map_err(|error| format!("--max-u 要一个非负整数：{error}"))?,
+                );
+            }
+            unknown_option => return Err(format!("不认识的参数：{unknown_option}")),
+        }
+        position += 2;
+    }
+    Ok(RootChoiceSelection {
+        bounds: bounds.ok_or_else(|| "缺 --bounds".to_string())?,
+        only,
+        largest_user_publishes_cap,
+    })
+}
+
+fn run_the_root_choice_family(arguments: &[String], run_profile: RunProfile) {
+    let selection = match parse_the_root_choice_command_line(arguments) {
+        Ok(selection) => selection,
+        Err(problem) => {
+            eprintln!("  ✗ {problem}");
+            eprintln!("  → 怎么办：{USAGE}");
+            std::process::exit(2);
+        }
+    };
+    let thread_count = thread_count_from_the_environment();
+    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
+    let run_started = Instant::now();
+    let mut emitter = Emitter { emitted: 0 };
+    let mut findings = RunFindings::default();
+    let selected_cells: Vec<(RootChoiceDomain, Vec<RootChoiceCell>)> = RootChoiceDomain::ALL
+        .iter()
+        .copied()
+        .filter(|domain| {
+            selection
+                .only
+                .as_ref()
+                .is_none_or(|names| names.iter().any(|name| name == domain.label()))
+        })
+        .map(|domain| {
+            (
+                domain,
+                root_choice_cells_of(
+                    domain,
+                    selection.bounds,
+                    selection.largest_user_publishes_cap,
+                ),
+            )
+        })
+        .collect();
+    emitter.result(&format!(
+        "name=config experiment=E165 bounds={} family={ROOT_CHOICE_FAMILY_LABEL} only={} max_u={} root_slot_bytes={ROOT_CHOICE_ROOT_SLOT_BYTES} region_devices=0,1,0 slots_per_region_wide={ROOT_CHOICE_SLOTS_PER_REGION_WIDE} slots_per_region_narrow={ROOT_CHOICE_SLOTS_PER_REGION_NARROW}{}",
+        selection.bounds.label(),
+        selection
+            .only
+            .as_ref()
+            .map_or_else(|| "every_domain".to_string(), |names| names.join(",")),
+        selection
+            .largest_user_publishes_cap
+            .map_or_else(|| "none".to_string(), |cap| cap.to_string()),
+        profile_fields_of_the_run_rows(run_profile),
+    ));
+
+    // ① 常量回比断言（不过当场 panic，进程以非 0 退出，一行结果都不算）。
+    cross_check_root_choice_local_constants_against_the_implementation();
+    emitter.result(
+        "name=self_check check=root_choice_local_constants_equal_the_implementation passed=true",
+    );
+    let self_checks_passed = 1;
+    for ring in [RingGeometry::BIG, RingGeometry::SMALL_NINE] {
+        emitter.result(&format!(
+            "name=in_flight_limit ring_slots={} local={} implementation={}",
+            ring.ring_slots,
+            ring.local_in_flight_limit(),
+            journal_in_flight_record_limit(ring.ring_bytes())
+        ));
+    }
+
+    // ② 阳性对照：每条判据都跑；有一行没红，这一段作废（V2）。
+    let controls_started = Instant::now();
+    let control_readings = root_choice_control_readings(&selected_cells, thread_count);
+    for reading in &control_readings {
+        emitter.result(&format!(
+            "name=control criterion={} measured={} value={} must_see={} red_as_required={}",
+            reading.criterion.label(),
+            reading.measured_where.replace(' ', ","),
+            reading.value,
+            reading.must_see,
+            reading.is_red_as_required
+        ));
+        if !reading.is_red_as_required {
+            findings.void_conditions.push(format!(
+                "V2 判据 {} 的阳性对照没红：{} 量到 {}，必须 {}",
+                reading.criterion.label(),
+                reading.measured_where,
+                reading.value,
+                reading.must_see
+            ));
+        }
+    }
+    println!(
+        "E7TIMING name=elapsed scope=positive_controls elapsed_seconds={:.3} threads={thread_count}",
+        controls_started.elapsed().as_secs_f64()
+    );
+
+    // ③ 各域各格：今天的三步。
+    let mut total_states: u64 = 0;
+    for (domain, cells) in &selected_cells {
+        let mut domain_total = RootChoiceTally::new();
+        for cell in cells {
+            let cell_started = Instant::now();
+            let tally = root_choice_tally_of_a_cell(
+                *domain,
+                *cell,
+                RootChoiceImplementation::TodayThreeSteps,
+                thread_count,
+            );
+            if domain.is_a_legal_history() {
+                assert_the_pinned_root_choice_cell(*domain, *cell, &tally);
+            } else {
+                assert_eq!(
+                    tally.get(RootChoiceCounter::States),
+                    PINNED_ROOT_CHOICE_ADVERSARIAL_STATES,
+                    "TV4：KA 状态数"
+                );
+                assert_eq!(
+                    tally.get(RootChoiceCounter::StatesWithoutASelfVerifiedRoot),
+                    1,
+                    "TV4：KA 里根环全空的态数（登记第七节乙类 KA：没有一条自证过的根的 1 态）"
+                );
+            }
+            emit_the_rows_of_a_root_choice_cell(&mut emitter, *domain, *cell, &tally);
+            println!(
+                "E7TIMING name=elapsed scope=cell domain={} mounts={} user_publishes={} states={} elapsed_seconds={:.3} threads={thread_count}",
+                domain.label(),
+                cell.mounts,
+                cell.user_publishes,
+                tally.get(RootChoiceCounter::States),
+                cell_started.elapsed().as_secs_f64()
+            );
+            domain_total.merge(&tally);
+        }
+        total_states += domain_total.get(RootChoiceCounter::States);
+        emit_the_verdict_of_a_root_choice_domain(
+            &mut emitter,
+            &mut findings,
+            *domain,
+            cells,
+            &domain_total,
+        );
+    }
+
+    for condition in &findings.void_conditions {
+        emitter.result(&format!(
+            "name=void condition={}",
+            condition.replace(' ', "_")
+        ));
+    }
+    for condition in &findings.halt_conditions {
+        emitter.result(&format!(
+            "name=halt condition={}",
+            condition.replace(' ', "_")
+        ));
+    }
+    emitter.result(&format!(
+        "name=verdict scope=run bounds={} family={ROOT_CHOICE_FAMILY_LABEL} states={total_states} self_checks_passed={self_checks_passed} control_readings={} controls_not_red={} void_conditions={} halt_conditions={}{}",
+        selection.bounds.label(),
+        control_readings.len(),
+        control_readings
+            .iter()
+            .filter(|reading| !reading.is_red_as_required)
+            .count(),
+        findings.void_conditions.len(),
+        findings.halt_conditions.len(),
+        profile_fields_of_the_run_rows(run_profile),
+    ));
+    println!(
+        "E7TIMING name=elapsed scope=run states={total_states} elapsed_seconds={:.3} threads={thread_count}",
+        run_started.elapsed().as_secs_f64()
+    );
+    emitter.done();
+    if !findings.void_conditions.is_empty() || !findings.halt_conditions.is_empty() {
+        std::process::exit(1);
+    }
+}
+
 // ── 命令行与主流程
 
 struct RunSelection {
@@ -8226,7 +12128,7 @@
     family_label: &'static str,
 }
 
-const USAGE: &str = "用法：e165_replay_bounded_enumeration --bounds segment1|segment2|light [--family legal|adversarial|all|multi_disk] [--only <域>[,<域>…]] [--max-n <N>] [--profile r1|r2] [--marker-parity even|odd] [--table direct|scan|both] [--disk-order normal|reversed]（档不给取 r1；奇偶、建表法、盘次序只在 r2 档有，不给分别取 even、direct、normal；这几个参数写在哪都行，单状态重放、逐态判定行、钉住的红例逐行重放同样认）；域名：L L6 L9 U C C6 A A6 AC X E1 E2 R M M6 MC（多盘三个域只在 --family multi_disk 时跑、要 --profile r2；all 只含合法历史与对抗两族）。单状态重放：--replay-state <域>:<N>:<序号>；逐态判定行：--list-states <域>:<N>（这两种的域名用产物里 domain= 的写法，U 域是 U_verify_on、U_verify_off）；钉住的红例逐行重放：--pinned-cases <表>（格式见 .claude/gate.d/e165-pinned-red-examples.tsv 文件头）";
+const USAGE: &str = "用法：e165_replay_bounded_enumeration --bounds segment1|segment2|light [--family legal|adversarial|all|multi_disk|root_choice] [--only <域>[,<域>…]] [--max-n <N>] [--profile r1|r2|r3] [--marker-parity even|odd] [--table direct|scan|both] [--disk-order normal|reversed]（档不给取 r1；奇偶、建表法、盘次序只在 r2、r3 档有，不给分别取 even、direct、normal；这几个参数写在哪都行，单状态重放、逐态判定行、钉住的红例逐行重放同样认）；域名：L L6 L9 U C C6 A A6 AC X E1 E2 R M M6 MC（多盘三个域只在 --family multi_disk 时跑、要 --profile r2；all 只含合法历史与对抗两族；--family root_choice 只在 --profile r3 时跑，域 K K9 KW KA，认 --bounds、--only、--max-u）。单状态重放：--replay-state <域>:<N>:<序号>；逐态判定行：--list-states <域>:<N>（这两种的域名用产物里 domain= 的写法，U 域是 U_verify_on、U_verify_off）；钉住的红例逐行重放：--pinned-cases <表>（格式见 .claude/gate.d/e165-pinned-red-examples.tsv 文件头）";
 
 fn parse_the_command_line(arguments: &[String]) -> Result<RunSelection, String> {
     let mut bounds: Option<BoundsTier> = None;
@@ -8361,10 +12263,10 @@
     };
     match profile_label.as_deref().unwrap_or(DEFAULT_PROFILE_LABEL) {
         "r1" if marker_parity_label.is_some() => {
-            Err("--marker-parity 只在 --profile r2 时有".to_string())
+            Err("--marker-parity 只在 --profile r2 或 r3 时有".to_string())
         }
         "r1" if table_source_label.is_some() || disk_order_label.is_some() => {
-            Err("--table 与 --disk-order 只在 --profile r2 时有".to_string())
+            Err("--table 与 --disk-order 只在 --profile r2 或 r3 时有".to_string())
         }
         "r1" => Ok((RunProfile::R1, remaining_arguments)),
         "r2" => Ok((
@@ -8373,7 +12275,13 @@
                 .with_reported_disk_order(reported_disk_order),
             remaining_arguments,
         )),
-        unknown_profile => Err(format!("不认识的档：{unknown_profile}（只认 r1、r2）")),
+        "r3" => Ok((
+            RunProfile::r3(marker_parity)
+                .with_table_source(table_source)
+                .with_reported_disk_order(reported_disk_order),
+            remaining_arguments,
+        )),
+        unknown_profile => Err(format!("不认识的档：{unknown_profile}（只认 r1、r2、r3）")),
     }
 }
 
@@ -8409,10 +12317,18 @@
             FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
             CriteriaSet::WithJ6QnAndJ7,
         ) => format!(" profile=r2 marker_parity={}", marker_parity.label()),
+        (
+            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
+            CriteriaSet::WithJ6QnAndJ7SplitByI812,
+        ) => format!(" profile=r3 marker_parity={}", marker_parity.label()),
         (FieldsBeyondTheProjectionGenerator::SameForEveryVersion, CriteriaSet::WithJ6QnAndJ7) => {
             " profile=r1_generator_with_r2_criteria".to_string()
         }
         (
+            FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
+            CriteriaSet::WithJ6QnAndJ7SplitByI812,
+        ) => " profile=r1_generator_with_r3_criteria".to_string(),
+        (
             FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
             CriteriaSet::RegisteredBeforeR2,
         ) => format!(
@@ -8455,6 +12371,17 @@
             return;
         }
     }
+    if arguments_name_the_root_choice_family(&arguments) {
+        if !run_profile.splits_j6qn_by_i812() {
+            eprintln!(
+                "  ✗ --family root_choice 只在 --profile r3 时有（登记 r3 5.1：这一族只在 r3 档）"
+            );
+            eprintln!("  → 怎么办：{USAGE}");
+            std::process::exit(2);
+        }
+        run_the_root_choice_family(&arguments, run_profile);
+        return;
+    }
     match parse_the_state_probe(&arguments) {
         Ok(Some((domain, _size, _probe)))
             if domain.family() == Family::MultiDisk && !run_profile.judges_j6qn_and_j7() =>
@@ -8615,6 +12542,16 @@
                 thread_count,
             );
             assert_the_pinned_absolute_values(*domain, size, &tally);
+            if run_profile.splits_j6qn_by_i812() && domain.family() == Family::Adversarial {
+                assert_the_pinned_i812_state_counts(*domain, size, &tally);
+                // 登记 r3 作废条款 TV6：拆开的四行对不回未拆的两行。
+                if let Some(difference) = the_split_rows_differ_from_the_unsplit_rows(&tally) {
+                    findings.void_conditions.push(format!(
+                        "TV6 {} 域 n = {size} 拆开的 J6Qn 四行对不回未拆的两行：{difference}",
+                        domain.label()
+                    ));
+                }
+            }
             // 登记 r2 第七节甲类、SF9：每条记录在每块盘上都落了的那一态扫盘表的条数断言要真跑到过（一个都没跑到，这条断言是空的）。
             if run_profile.table_source != RecordTableSource::Direct
                 && domain.family() != Family::Adversarial
@@ -8788,6 +12725,27 @@
     }
 
     #[test]
+    fn i812_holds_when_the_txg_does_not_fall_with_the_counter_inside_one_instance_and_equal_txgs_count_as_holding(
+    ) {
+        check_the_i812_judgement_on_hand_made_tables();
+    }
+
+    #[test]
+    fn the_i812_state_counts_of_the_adversarial_domains_equal_the_pinned_values() {
+        check_the_i812_state_counts_of_the_smallest_sizes();
+    }
+
+    #[test]
+    fn the_i812_split_controls_read_the_pinned_cells_of_the_adversarial_domains() {
+        check_the_i812_split_control_readings();
+    }
+
+    #[test]
+    fn every_i812_cell_has_a_row_on_the_r3_profile_and_the_other_profiles_have_none() {
+        check_the_rows_of_the_i812_split();
+    }
+
+    #[test]
     fn the_odd_marker_parity_turns_the_unmount_marker_controls_to_0_on_l_and_above_0_on_c() {
         check_the_odd_marker_parity_turns_the_unmount_marker_controls();
     }
@@ -9035,11 +12993,23 @@
         for reading in &r1_readings {
             assert!(reading.is_red_as_required, "r1 档对照没红：{reading:?}");
         }
-        let readings = positive_control_readings(
+        // r3 档判全表（r2 档的每一行再加 J6Qn 的 I-8.12 拆行）；r2 档那一份另核：r2 档不判拆行那一条判据。
+        let r2_readings = positive_control_readings(
             &[Family::LegalHistory, Family::Adversarial, Family::MultiDisk],
             RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
             thread_count,
         );
+        assert!(
+            r2_readings
+                .iter()
+                .all(|reading| reading.criterion != Criterion::J6QnSplitByI812),
+            "r2 档不判 J6Qn 的 I-8.12 拆行"
+        );
+        let readings = positive_control_readings(
+            &[Family::LegalHistory, Family::Adversarial, Family::MultiDisk],
+            RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+            thread_count,
+        );
         for criterion in Criterion::ALL.iter() {
             assert!(
                 readings
@@ -9309,4 +13279,299 @@
             PinnedRedExampleReplay::OrdinalOutsideTheDomainSize { states_walked: 2 }
         );
     }
+
+    // ── 择根与重放合判一族（登记 r3 5.3 至 5.7、9.2 乙）
+
+    #[test]
+    fn root_choice_local_constants_equal_the_implementation_and_the_first_mount_of_a_new_pool_pushes_two_overhead_publishes(
+    ) {
+        cross_check_root_choice_local_constants_against_the_implementation();
+    }
+
+    #[test]
+    fn root_versions_compare_the_txg_first_and_the_instance_breaks_ties() {
+        let version = |checkpoint_txg: u64, instance: u32| RootVersion {
+            checkpoint_txg,
+            instance,
+        };
+        assert!(
+            version(100, 1) > version(99, 9),
+            "txg 为主：txg 大的版本大，不看实例代号"
+        );
+        assert!(
+            version(100, 2) > version(100, 1),
+            "txg 相等时实例代号大的版本大"
+        );
+        assert_eq!(version(7, 3), version(7, 3));
+    }
+
+    #[test]
+    fn the_warm_up_plan_covers_both_devices_with_at_most_two_empty_publishes_after_the_row_write() {
+        assert_eq!(root_choice_warm_up_plan(1), vec![1, 2]);
+        assert_eq!(root_choice_warm_up_plan(2), vec![2, 3, 4]);
+        assert_eq!(root_choice_warm_up_plan(3), vec![3, 4]);
+        assert_eq!(root_choice_warm_up_plan(4), vec![4, 5]);
+        for row_write_txg in 1..=30 {
+            let plan = root_choice_warm_up_plan(row_write_txg);
+            let mut covered = [false; 2];
+            for checkpoint_txg in &plan {
+                covered[root_choice_device_of_publish(*checkpoint_txg)] = true;
+            }
+            assert!(
+                covered[0] && covered[1],
+                "写行 txg {row_write_txg} 的计划 {plan:?} 没覆盖两块盘"
+            );
+            assert!(
+                plan.len() <= 3,
+                "写行之后至多两次空发布（几何的结果）：{plan:?}"
+            );
+        }
+    }
+
+    #[test]
+    fn a_root_slot_reading_with_the_same_txg_and_the_other_instance_disagrees_with_the_truth() {
+        let material_of = |checkpoint_txg: u64, instance: u32| {
+            root_material_without_records(
+                instance,
+                checkpoint_txg,
+                NewRootSegment::off_timeline(u64::from(instance), checkpoint_txg),
+            )
+        };
+        let root_of = |material: &RootChoicePublishMaterial| {
+            RootRecord::parse_slot(&material.root_bytes, &ROOT_FILESYSTEM_IDENTIFIER)
+                .expect("生成的根自证过")
+        };
+        let instance_one = material_of(7, 1);
+        let instance_two = material_of(7, 2);
+        assert!(root_slot_reading_agrees_with_the_truth(
+            RootRingSlotReading::SelfVerified(root_of(&instance_one)),
+            Some(&instance_one)
+        ));
+        assert!(
+            !root_slot_reading_agrees_with_the_truth(
+                RootRingSlotReading::SelfVerified(root_of(&instance_one)),
+                Some(&instance_two)
+            ),
+            "同 txg、实例代号互换：对拍必须报不一致"
+        );
+        assert!(!root_slot_reading_agrees_with_the_truth(
+            RootRingSlotReading::SelfVerified(root_of(&instance_one)),
+            None
+        ));
+        assert!(root_slot_reading_agrees_with_the_truth(
+            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
+            None
+        ));
+        assert!(!root_slot_reading_agrees_with_the_truth(
+            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
+            Some(&instance_one)
+        ));
+        assert!(!root_slot_reading_agrees_with_the_truth(
+            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable),
+            None
+        ));
+    }
+
+    #[test]
+    fn the_local_root_choice_rules_pick_as_the_registered_comparisons_on_hand_made_rings() {
+        let version = |checkpoint_txg: u64, instance: u32| {
+            Some(RootVersion {
+                checkpoint_txg,
+                instance,
+            })
+        };
+        let ring = [version(100, 2), version(100, 1), version(101, 1)];
+        let picked = |rule: LocalRootChoiceRule| rule.chooses_among(&ring).expect("环里有根");
+        assert_eq!(
+            picked(LocalRootChoiceRule::NewestWins).describe(),
+            "txg101/instance1"
+        );
+        assert_eq!(
+            picked(LocalRootChoiceRule::InstanceFirstThenTxg).describe(),
+            "txg100/instance2"
+        );
+        assert_eq!(
+            picked(LocalRootChoiceRule::FirstSelfVerifiedWins).describe(),
+            "txg100/instance2"
+        );
+        assert_eq!(
+            picked(LocalRootChoiceRule::IgnoresTheInstance).describe(),
+            "txg101/instance1"
+        );
+        let tie = [version(100, 2), version(100, 1)];
+        assert_eq!(
+            LocalRootChoiceRule::TieTakesTheLowerInstance
+                .chooses_among(&tie)
+                .expect("环里有根")
+                .describe(),
+            "txg100/instance1"
+        );
+        assert_eq!(
+            LocalRootChoiceRule::NewestWins
+                .chooses_among(&tie)
+                .expect("环里有根")
+                .describe(),
+            "txg100/instance2"
+        );
+        assert_eq!(
+            LocalRootChoiceRule::NewestWins.chooses_among(&[None, None]),
+            None
+        );
+    }
+
+    #[test]
+    fn the_root_choice_state_counts_of_the_smallest_cells_equal_the_pinned_values_and_the_kw_wrap_cells_read_four_and_one(
+    ) {
+        let thread_count = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
+        for (domain, mounts, user_publishes) in [
+            (RootChoiceDomain::BigRing, 1, 0),
+            (RootChoiceDomain::BigRing, 1, 1),
+            (RootChoiceDomain::BigRing, 1, 2),
+            (RootChoiceDomain::BigRing, 2, 0),
+            (RootChoiceDomain::BigRing, 2, 1),
+            (RootChoiceDomain::SmallRingNine, 1, 0),
+            (RootChoiceDomain::SmallRingNine, 2, 1),
+            (RootChoiceDomain::WrappingRootRing, 1, 0),
+            (RootChoiceDomain::WrappingRootRing, 1, 10),
+            (RootChoiceDomain::WrappingRootRing, 1, 11),
+        ] {
+            let cell = RootChoiceCell {
+                mounts,
+                user_publishes,
+            };
+            let tally = root_choice_tally_of_a_cell(
+                domain,
+                cell,
+                RootChoiceImplementation::TodayThreeSteps,
+                thread_count,
+            );
+            assert_the_pinned_root_choice_cell(domain, cell, &tally);
+            assert_eq!(
+                tally.get(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
+                0,
+                "读槽对拍与真相一致：{domain:?} {cell:?}"
+            );
+            assert_eq!(tally.get(RootChoiceCounter::ChoseNothingStates), 0);
+            assert_eq!(tally.get(RootChoiceCounter::PanickedStates), 0);
+        }
+    }
+
+    #[test]
+    fn the_adversarial_root_rings_have_125_states_one_of_them_without_a_self_verified_root_and_zero_kj2_violations_today(
+    ) {
+        let today = root_choice_tally_of_a_cell(
+            RootChoiceDomain::AdversarialRootRing,
+            RootChoiceCell {
+                mounts: 0,
+                user_publishes: 0,
+            },
+            RootChoiceImplementation::TodayThreeSteps,
+            1,
+        );
+        assert_eq!(
+            today.get(RootChoiceCounter::States),
+            PINNED_ROOT_CHOICE_ADVERSARIAL_STATES
+        );
+        assert_eq!(today.get(RootChoiceCounter::ChoseNothingStates), 1);
+        assert_eq!(today.get(RootChoiceCounter::Kj0Violations), 1);
+        assert_eq!(today.get(RootChoiceCounter::Kj2Judged), 124);
+        assert_eq!(today.get(RootChoiceCounter::Kj2Violations), 0);
+        assert_eq!(
+            today.get(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
+            0
+        );
+        assert!(today.get(RootChoiceCounter::SameTxgDifferentInstanceRoots) > 0);
+        assert!(today.get(RootChoiceCounter::HigherInstanceLowerTxgRoots) > 0);
+    }
+
+    /// 每条判据在每个被判的格上各有对照读数、都红；读数条数照格数逐条判据数（删掉某条判据的某一行对照，条数对不上）。
+    #[test]
+    fn every_root_choice_criterion_has_a_positive_control_and_every_control_is_red() {
+        let thread_count = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
+        let big_ring_cells: Vec<RootChoiceCell> = [(1, 0), (1, 2), (1, 3)]
+            .into_iter()
+            .map(|(mounts, user_publishes)| RootChoiceCell {
+                mounts,
+                user_publishes,
+            })
+            .collect();
+        let selected_cells = vec![
+            (RootChoiceDomain::BigRing, big_ring_cells.clone()),
+            (
+                RootChoiceDomain::AdversarialRootRing,
+                vec![RootChoiceCell {
+                    mounts: 0,
+                    user_publishes: 0,
+                }],
+            ),
+        ];
+        let readings = root_choice_control_readings(&selected_cells, thread_count);
+        for reading in &readings {
+            assert!(reading.is_red_as_required, "对照没红：{reading:?}");
+        }
+        let readings_of = |criterion: RootChoiceCriterion| {
+            readings
+                .iter()
+                .filter(|reading| reading.criterion == criterion)
+                .count()
+        };
+        let cell_count = big_ring_cells.len();
+        let local_rule_count = LocalRootChoiceRule::ALL.len();
+        assert_eq!(readings_of(RootChoiceCriterion::Kj0Defined), 1);
+        assert_eq!(
+            readings_of(RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish),
+            2 * cell_count,
+            "KJ1 在每个格上 KF1、KF2 各一条对照"
+        );
+        assert_eq!(
+            readings_of(RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot),
+            2 * cell_count + local_rule_count,
+            "KJ2 在每个格上 KF1、KF2 各一条，在 KA 上每种本地择根序一条"
+        );
+        assert_eq!(
+            readings_of(RootChoiceCriterion::Kj3LandsOnTheIdealRecovery),
+            cell_count,
+            "KJ3 在每个格上 KF2 一条"
+        );
+        for criterion in RootChoiceCriterion::ALL {
+            assert!(
+                readings
+                    .iter()
+                    .any(|reading| reading.criterion == *criterion),
+                "判据 {} 没有对照的读数",
+                criterion.label()
+            );
+        }
+    }
+
+    #[test]
+    fn the_root_choice_command_line_takes_the_family_and_rejects_unknown_domains_and_options() {
+        let arguments =
+            |text: &str| -> Vec<String> { text.split_whitespace().map(str::to_string).collect() };
+        assert!(arguments_name_the_root_choice_family(&arguments(
+            "--bounds segment1 --family root_choice"
+        )));
+        assert!(!arguments_name_the_root_choice_family(&arguments(
+            "--bounds segment1 --family legal"
+        )));
+        let selection = parse_the_root_choice_command_line(&arguments(
+            "--bounds light --family root_choice --only K9,KA --max-u 1",
+        ))
+        .expect("认得的参数");
+        assert_eq!(selection.bounds, BoundsTier::Light);
+        assert_eq!(
+            selection.only,
+            Some(vec!["K9".to_string(), "KA".to_string()])
+        );
+        assert_eq!(selection.largest_user_publishes_cap, Some(1));
+        assert!(parse_the_root_choice_command_line(&arguments(
+            "--bounds segment1 --family root_choice --only L"
+        ))
+        .is_err());
+        assert!(parse_the_root_choice_command_line(&arguments(
+            "--bounds segment1 --family root_choice --max-n 3"
+        ))
+        .is_err());
+        assert!(parse_the_root_choice_command_line(&arguments("--family root_choice")).is_err());
+    }
 }
```
