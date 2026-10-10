# 删除批第 33 批 R 合入的改动原样（2026-10-07）

## 实现员 r：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch33/r/patch/crates.patch）

````diff
--- a/crates/singlefs-checker-tier/src/crash.rs
+++ b/crates/singlefs-checker-tier/src/crash.rs
@@ -33,8 +33,8 @@
 use singlefs_core::system_configuration::SystemConfiguration;
 use singlefs_harness::memory_pool::{
     newest_persisted_root, publishes_in, root_identity_of_write, root_identity_written_by,
-    CrashImage, MemoryPool, PublishWrites, PublishedVersion, RecordCheck, RecordStreamContinuity,
-    RetainedWrite, WrittenContents, SECTOR_BYTES,
+    CrashImage, MemoryPool, PublishWrites, PublishedVersion, PublishedVersionWithoutAFile,
+    RecordCheck, RecordStreamContinuity, RetainedWrite, WrittenContents, SECTOR_BYTES,
 };
 use singlefs_harness::numeric_environment_variable::{
     number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
@@ -967,7 +967,113 @@
     .map(|violation| violation.reason)
 }
 
+/// 层 0 一条流的版本表：带文件的那几版（[`PublishedVersion`]）与文件从那一版起没有了的那几版（[`PublishedVersionWithoutAFile`]，
+/// D8（核心索引结构） 已定项 16 第 4 条：删除意图那次发布起冷走读答没有文件）。表按根（实例，txg）认；落到的根不在表里时，
+/// 照比它旧的最新那一项判（按 (txg, 实例) 字典序，同 D22（单元原子性怎么合成） 已定项 7 的择根序）。
+#[derive(Clone, Copy, Debug)]
+pub struct Layer0VersionTable<'table> {
+    versions_with_a_file: &'table [PublishedVersion],
+    versions_without_a_file: &'table [PublishedVersionWithoutAFile],
+}
+
+/// 恢复落到的根在版本表里是哪一项。
+#[derive(Clone, Copy, Debug)]
+enum Layer0VersionAtTheRoot<'table> {
+    /// 这一版有文件，读回的要是它的内容。
+    WithAFile(&'table PublishedVersion),
+    /// 文件从这一版起没有了。
+    WithoutAFile,
+    /// 表里没有这个根。
+    NotInTheTable,
+}
+
+impl<'table> Layer0VersionTable<'table> {
+    /// 没有一版是「文件从这一版起没有了」：文件一旦写出就不消失的流（删除之前的每一条流）。
+    #[must_use]
+    pub fn with_a_file_only(versions_with_a_file: &'table [PublishedVersion]) -> Self {
+        Self {
+            versions_with_a_file,
+            versions_without_a_file: &[],
+        }
+    }
+
+    /// # Panics
+    /// 同一个根（实例，txg）既在带文件的那几版里、又在没有文件的那几版里：版本表自相矛盾，是用例写错了。
+    #[must_use]
+    pub fn new(
+        versions_with_a_file: &'table [PublishedVersion],
+        versions_without_a_file: &'table [PublishedVersionWithoutAFile],
+    ) -> Self {
+        for without_a_file in versions_without_a_file {
+            assert!(
+                !versions_with_a_file.iter().any(|with_a_file| {
+                    (with_a_file.instance, with_a_file.checkpoint_txg)
+                        == (without_a_file.instance, without_a_file.checkpoint_txg)
+                }),
+                "实例 {} 第 {} 代既写成有文件、又写成文件从这一版起没有了",
+                without_a_file.instance.0,
+                without_a_file.checkpoint_txg.0
+            );
+        }
+        Self {
+            versions_with_a_file,
+            versions_without_a_file,
+        }
+    }
+
+    fn version_at_the_root(
+        &self,
+        instance: InstanceGeneration,
+        checkpoint_txg: CheckpointTxg,
+    ) -> Layer0VersionAtTheRoot<'table> {
+        if let Some(with_a_file) = self.versions_with_a_file.iter().find(|version| {
+            version.checkpoint_txg == checkpoint_txg && version.instance == instance
+        }) {
+            return Layer0VersionAtTheRoot::WithAFile(with_a_file);
+        }
+        if self
+            .versions_without_a_file
+            .iter()
+            .any(|version| version.checkpoint_txg == checkpoint_txg && version.instance == instance)
+        {
+            return Layer0VersionAtTheRoot::WithoutAFile;
+        }
+        Layer0VersionAtTheRoot::NotInTheTable
+    }
+
+    /// 不在表里的根照比它旧的最新那一项：那一项有文件就交回它（报没有文件是违例），那一项是没有文件、或没有更旧的项就交回 `None`。
+    /// 文件从不消失的表里（[`Self::with_a_file_only`]）这与「任一更旧版有文件」相同。
+    fn newest_older_version_if_it_has_a_file(
+        &self,
+        instance: InstanceGeneration,
+        checkpoint_txg: CheckpointTxg,
+    ) -> Option<&'table PublishedVersion> {
+        let newest_older_with_a_file = self
+            .versions_with_a_file
+            .iter()
+            .filter(|version| {
+                (version.checkpoint_txg, version.instance) < (checkpoint_txg, instance)
+            })
+            .max_by_key(|version| (version.checkpoint_txg, version.instance));
+        let newest_older_without_a_file = self
+            .versions_without_a_file
+            .iter()
+            .map(|version| (version.checkpoint_txg, version.instance))
+            .filter(|root| *root < (checkpoint_txg, instance))
+            .max();
+        match (newest_older_with_a_file, newest_older_without_a_file) {
+            (Some(with_a_file), Some(without_a_file)) => {
+                ((with_a_file.checkpoint_txg, with_a_file.instance) > without_a_file)
+                    .then_some(with_a_file)
+            }
+            (Some(with_a_file), None) => Some(with_a_file),
+            (None, Some(_) | None) => None,
+        }
+    }
+}
+
 /// 同 [`oracle_violation_for_versions`]，另交回违例是哪一类（[`Layer0OracleViolationKind`]，层 0 发现表按它分签名）。
+/// 文件从不消失的流：版本表是 [`Layer0VersionTable::with_a_file_only`]。
 #[must_use]
 pub fn classified_oracle_violation_for_versions(
     outcome: &RecoveryOutcome,
@@ -975,6 +1081,24 @@
     newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
     versions: &[PublishedVersion],
 ) -> Option<Layer0OracleViolation> {
+    classified_oracle_violation_for_the_version_table(
+        outcome,
+        effective_root,
+        newest_persisted_root,
+        Layer0VersionTable::with_a_file_only(versions),
+    )
+}
+
+/// 层 0 的 oracle 对一张版本表（[`Layer0VersionTable`]）判：落到的根是带文件的一版，读回的得是它的内容；是「文件从这一版起没有了」
+/// 的一版，只许报没有文件；不在表里，报没有文件时看比它旧的最新那一项——那一项有文件就是违例（文件从不消失的流上与「任一更旧版有文件」
+/// 相同），不在表里却读出了内容一律是违例。根槽已持久而恢复到旧态、走读失败、没择到根同 [`oracle_violation_for_versions`]。
+#[must_use]
+pub fn classified_oracle_violation_for_the_version_table(
+    outcome: &RecoveryOutcome,
+    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
+    newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
+    versions: Layer0VersionTable<'_>,
+) -> Option<Layer0OracleViolation> {
     let violation =
         |kind: Layer0OracleViolationKind, reason: String| Layer0OracleViolation { kind, reason };
     let Some((effective_instance, effective_txg)) = effective_root else {
@@ -994,21 +1118,18 @@
             ));
         }
     }
-    let version = versions.iter().find(|version| {
-        version.checkpoint_txg == effective_txg && version.instance == effective_instance
-    });
-    match (outcome, version) {
+    match (
+        outcome,
+        versions.version_at_the_root(effective_instance, effective_txg),
+    ) {
         (RecoveryOutcome::Failed { failure, .. }, _) => Some(violation(
             Layer0OracleViolationKind::ReadFailed,
             format!("走读失败：{failure:?}"),
         )),
-        // 落在一个没发布过的更新的根上报「没有文件」：更旧的根下面有文件时是违例（第二轮攻方腿：此前一律放过）。
-        (RecoveryOutcome::NoFile { .. }, None) => versions
-            .iter()
-            .find(|candidate_version| {
-                (candidate_version.checkpoint_txg, candidate_version.instance)
-                    < (effective_txg, effective_instance)
-            })
+        // 落在一个表里没有的更新的根上报「没有文件」：比它旧的最新一项下面有文件时是违例（第二轮攻方腿：此前一律放过）；
+        // 那一项是「文件从这一版起没有了」（删除那几批落在意图之后）时不是。
+        (RecoveryOutcome::NoFile { .. }, Layer0VersionAtTheRoot::NotInTheTable) => versions
+            .newest_older_version_if_it_has_a_file(effective_instance, effective_txg)
             .map(|older_version| {
                 violation(
                     Layer0OracleViolationKind::NoFileWhereAnOlderRootHasOne,
@@ -1021,15 +1142,19 @@
                 ),
                 )
             }),
-        (RecoveryOutcome::NoFile { .. }, Some(_)) => Some(violation(
+        (RecoveryOutcome::NoFile { .. }, Layer0VersionAtTheRoot::WithoutAFile) => None,
+        (RecoveryOutcome::NoFile { .. }, Layer0VersionAtTheRoot::WithAFile(_)) => Some(violation(
             Layer0OracleViolationKind::NoFileWhereThisRootHasOne,
             format!("第 {} 代根下面有文件却报没有", effective_txg.0),
         )),
-        (RecoveryOutcome::FileRead { .. }, None) => Some(violation(
+        (
+            RecoveryOutcome::FileRead { .. },
+            Layer0VersionAtTheRoot::NotInTheTable | Layer0VersionAtTheRoot::WithoutAFile,
+        ) => Some(violation(
             Layer0OracleViolationKind::FileReadWhereThisRootHasNone,
             format!("第 {} 代根下面没有文件却读出了内容", effective_txg.0),
         )),
-        (RecoveryOutcome::FileRead { content, .. }, Some(version)) => {
+        (RecoveryOutcome::FileRead { content, .. }, Layer0VersionAtTheRoot::WithAFile(version)) => {
             (*content != version.content).then(|| {
                 violation(
                     Layer0OracleViolationKind::WrongContent,
@@ -1073,7 +1198,7 @@
         writes,
         persisted,
         judged_root_index,
-        versions,
+        Layer0VersionTable::with_a_file_only(versions),
         None,
         tally,
     )
@@ -1223,23 +1348,38 @@
 }
 
 /// 判一个崩溃镜像（写表是枚举用的那一张，撕裂镜像与重放接在后面）；`judged_root_index` 是被判的那次根槽 FUA 写。
+/// 文件从不消失的流：版本表是 [`Layer0VersionTable::with_a_file_only`]。
 #[must_use]
 pub fn judge_crash_image(
     image: &CrashImage<'_>,
     judged_root_index: usize,
     versions: &[PublishedVersion],
 ) -> Layer0StateJudgement {
+    judge_crash_image_against_the_version_table(
+        image,
+        judged_root_index,
+        Layer0VersionTable::with_a_file_only(versions),
+    )
+}
+
+/// 同 [`judge_crash_image`]，oracle 对一张版本表判（[`classified_oracle_violation_for_the_version_table`]）。
+#[must_use]
+fn judge_crash_image_against_the_version_table(
+    image: &CrashImage<'_>,
+    judged_root_index: usize,
+    versions: Layer0VersionTable<'_>,
+) -> Layer0StateJudgement {
     let root_persisted = image.persisted[judged_root_index];
     let newest_persisted = newest_persisted_root(image.writes, &image.persisted);
     let consulted = recover(image, JournalPolicy::Consult);
     let ignored = recover(image, JournalPolicy::Ignore);
-    let consulted_oracle = classified_oracle_violation_for_versions(
+    let consulted_oracle = classified_oracle_violation_for_the_version_table(
         &consulted.outcome,
         consulted.effective_root,
         newest_persisted,
         versions,
     );
-    let ignored_oracle = classified_oracle_violation_for_versions(
+    let ignored_oracle = classified_oracle_violation_for_the_version_table(
         &ignored.outcome,
         ignored.effective_root,
         newest_persisted,
@@ -1405,7 +1545,7 @@
     writes: &[RetainedWrite],
     persisted: Vec<bool>,
     judged_root_index: usize,
-    versions: &[PublishedVersion],
+    versions: Layer0VersionTable<'_>,
     position: Option<Layer0StatePosition>,
     tally: &mut Layer0Tally,
 ) -> RecoveryReport {
@@ -1414,7 +1554,8 @@
         writes,
         persisted,
     };
-    let judgement = judge_crash_image(&image, judged_root_index, versions);
+    let judgement =
+        judge_crash_image_against_the_version_table(&image, judged_root_index, versions);
     let Layer0StateJudgement {
         consulted,
         ignored,
@@ -2363,6 +2504,34 @@
 pub type Layer0StateObserver<'observer> =
     &'observer mut dyn FnMut(&CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts);
 
+/// 在工作线程上跑的逐状态观察者：同 [`Layer0StateObserver`] 拿到的三样，各工作线程共用一份、并发调用，所以只读、能跨线程。
+/// 每个状态评完就在评它的那个工作线程上调，计数记进那一片自己的观察者计数，并片时按片号次序相加（与在调用线程上看的计数相同）。
+/// 每个状态上要做重活（可写挂载）的观察者用它：调用线程上逐个看撑不住全量。
+pub type Layer0StateObserverOnTheWorkerThreads<'observer> =
+    &'observer (dyn Fn(&CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts) + Sync);
+
+/// 层 0 枚举逐状态看不看、在哪个线程上看。
+pub enum Layer0StateObserving<'observer> {
+    /// 不看。
+    NoObserver,
+    /// 在调用线程上、按状态序号从小到大看（[`Layer0StateObserver`]）。
+    OnTheCallingThread(Layer0StateObserver<'observer>),
+    /// 在评那个状态的工作线程上看（[`Layer0StateObserverOnTheWorkerThreads`]）。
+    OnTheWorkerThreads(Layer0StateObserverOnTheWorkerThreads<'observer>),
+}
+
+impl<'observer> Layer0StateObserving<'observer> {
+    /// 旧入口的写法：`None` 不看，`Some` 在调用线程上看。
+    fn on_the_calling_thread_if_given(
+        observe_state: Option<Layer0StateObserver<'observer>>,
+    ) -> Self {
+        match observe_state {
+            Some(observe) => Self::OnTheCallingThread(observe),
+            None => Self::NoObserver,
+        }
+    }
+}
+
 /// 工作线程评状态时 panic，就把旗子立起来：别的工作线程看到旗子不再领新片，调用线程不必等整条流跑完才知道红了。
 struct RaiseFlagWhenPanicking<'flag>(&'flag AtomicBool);
 
@@ -2375,10 +2544,12 @@
 }
 
 /// 工作线程要不要把每个状态的持久集合与看 journal 那一遍恢复的报告带回调用线程。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-enum StateReportRetention {
-    /// 有观察者：带回去，由调用线程按序号交给它。
+#[derive(Clone, Copy)]
+enum StateReportRetention<'observer> {
+    /// 在调用线程上看的观察者：带回去，由调用线程按序号交给它。
     HandEachStateToObserver,
+    /// 在工作线程上看的观察者：评完一个状态就地交给它，只带计数回去。
+    ObserveOnTheWorkerThread(Layer0StateObserverOnTheWorkerThreads<'observer>),
     /// 没有观察者：只带计数回去（全量流上两百多万个报告不必留）。
     CountOnly,
 }
@@ -2404,8 +2575,8 @@
     slice_index: usize,
     slice: Range<u64>,
     judged_root_index: usize,
-    versions: &[PublishedVersion],
-    retention: StateReportRetention,
+    versions: Layer0VersionTable<'_>,
+    retention: StateReportRetention<'_>,
 ) -> FinishedSlice {
     let mut tally = Layer0Tally::default();
     let mut observed_states = Vec::new();
@@ -2430,6 +2601,27 @@
                 );
                 observed_states.push((persisted, consulted_report));
             }
+            StateReportRetention::ObserveOnTheWorkerThread(observe) => {
+                let consulted_report = evaluate_state_recording_findings(
+                    base,
+                    writes,
+                    persisted.clone(),
+                    judged_root_index,
+                    versions,
+                    Some(position),
+                    &mut tally,
+                );
+                observe(
+                    &CrashImage {
+                        base,
+                        writes,
+                        persisted,
+                    },
+                    &consulted_report,
+                    &mut tally.observer_counts,
+                );
+                tally.observed_states += 1;
+            }
             StateReportRetention::CountOnly => {
                 evaluate_state_recording_findings(
                     base,
@@ -2573,7 +2765,7 @@
     writes: &[RetainedWrite],
     plan: &Layer0StatePlan<'_>,
     judged_root_index: usize,
-    versions: &[PublishedVersion],
+    versions: Layer0VersionTable<'_>,
     slices: &[Range<u64>],
     has_observer: bool,
     shard: Option<Layer0ShardOfShards>,
@@ -2621,11 +2813,24 @@
             .expect("下标")
             .to_le_bytes(),
     );
-    for version in versions {
+    for version in versions.versions_with_a_file {
         message.extend_from_slice(&version.instance.0.to_le_bytes());
         message.extend_from_slice(&version.checkpoint_txg.0.to_le_bytes());
         append_length_prefixed(&mut message, &version.content);
     }
+    // 没有文件的那几版只在有的时候拼：文件从不消失的流，计划哈希与加这一项之前的相同，旧进度文件照样相认。
+    if !versions.versions_without_a_file.is_empty() {
+        append_length_prefixed(&mut message, b"versions without a file");
+        message.extend_from_slice(
+            &u64::try_from(versions.versions_without_a_file.len())
+                .expect("版本数装得进 u64")
+                .to_le_bytes(),
+        );
+        for version in versions.versions_without_a_file {
+            message.extend_from_slice(&version.instance.0.to_le_bytes());
+            message.extend_from_slice(&version.checkpoint_txg.0.to_le_bytes());
+        }
+    }
     message.extend_from_slice(&plan.state_count.to_le_bytes());
     for slice in slices {
         message.extend_from_slice(&slice.start.to_le_bytes());
@@ -3039,8 +3244,7 @@
 /// 同 [`enumerate_layer0_in_state_slices_or_one_shard`]；另有发现日志建、写、落盘失败。
 #[allow(
     clippy::too_many_arguments,
-    clippy::too_many_lines,
-    reason = "前六个与单线程时的枚举相同，多出来的是切法、观察者、续跑与发现日志；续跑的读回、并片、落盘、收尾是同一件事的几步，拆开要把可变状态来回传"
+    reason = "前六个与单线程时的枚举相同，多出来的是切法、观察者、续跑与发现日志"
 )]
 #[must_use]
 pub fn enumerate_layer0_in_state_slices_or_one_shard_with_findings_log(
@@ -3051,7 +3255,77 @@
     versions: &[PublishedVersion],
     expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
     parallelism: Layer0Parallelism,
-    mut observe_state: Option<Layer0StateObserver<'_>>,
+    observe_state: Option<Layer0StateObserver<'_>>,
+    resume: &Layer0Resume,
+    findings_log: &Layer0FindingsLog,
+) -> Layer0EnumerationOutcome {
+    enumerate_layer0_against_the_version_table_with_findings_log(
+        base,
+        writes,
+        segments,
+        judged_root_index,
+        Layer0VersionTable::with_a_file_only(versions),
+        expansion,
+        parallelism,
+        Layer0StateObserving::on_the_calling_thread_if_given(observe_state),
+        resume,
+        findings_log,
+    )
+}
+
+/// 同 [`enumerate_layer0_in_state_slices_or_one_shard`]，oracle 对一张版本表判（[`Layer0VersionTable`]：带「文件从这一版起没有了」的流，
+/// 删除那几次发布），观察者可以放到工作线程上看（[`Layer0StateObserving::OnTheWorkerThreads`]：每个状态上可写挂载这种重活）。
+/// 进度文件的计划哈希另拼没有文件的那几版（没有时与 [`enumerate_layer0_in_state_slices_or_one_shard`] 的相同）；观察者在哪个线程上看不进哈希，
+/// 两种看法的观察者计数相同。
+///
+/// # Panics
+/// 同 [`enumerate_layer0_in_state_slices_or_one_shard`]。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "前六个与单线程时的枚举相同（版本表多了没有文件的那几版），多出来的是切法、观察者与续跑"
+)]
+#[must_use]
+pub fn enumerate_layer0_against_the_version_table_or_one_shard(
+    base: &MemoryPool,
+    writes: &[RetainedWrite],
+    segments: &[Vec<usize>],
+    judged_root_index: usize,
+    versions: Layer0VersionTable<'_>,
+    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
+    parallelism: Layer0Parallelism,
+    observing: Layer0StateObserving<'_>,
+    resume: &Layer0Resume,
+) -> Layer0EnumerationOutcome {
+    enumerate_layer0_against_the_version_table_with_findings_log(
+        base,
+        writes,
+        segments,
+        judged_root_index,
+        versions,
+        expansion,
+        parallelism,
+        observing,
+        resume,
+        &Layer0FindingsLog::from_environment(),
+    )
+}
+
+/// 层 0 枚举的本体（几个公开入口都落到这里），写法见 [`enumerate_layer0_in_state_slices`] 与
+/// [`enumerate_layer0_in_state_slices_or_one_shard_with_findings_log`]。
+#[allow(
+    clippy::too_many_arguments,
+    clippy::too_many_lines,
+    reason = "前六个与单线程时的枚举相同，多出来的是切法、观察者、续跑与发现日志；续跑的读回、并片、落盘、收尾是同一件事的几步，拆开要把可变状态来回传"
+)]
+fn enumerate_layer0_against_the_version_table_with_findings_log(
+    base: &MemoryPool,
+    writes: &[RetainedWrite],
+    segments: &[Vec<usize>],
+    judged_root_index: usize,
+    versions: Layer0VersionTable<'_>,
+    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
+    parallelism: Layer0Parallelism,
+    observing: Layer0StateObserving<'_>,
     resume: &Layer0Resume,
     findings_log: &Layer0FindingsLog,
 ) -> Layer0EnumerationOutcome {
@@ -3070,7 +3344,21 @@
         plan.state_count,
         &slicing_of_the_run(&parallelism, resume, plan.state_count),
     );
-    let has_observer = observe_state.is_some();
+    let (retention, mut observe_state) = match observing {
+        Layer0StateObserving::NoObserver => (StateReportRetention::CountOnly, None),
+        Layer0StateObserving::OnTheCallingThread(observe) => {
+            (StateReportRetention::HandEachStateToObserver, Some(observe))
+        }
+        Layer0StateObserving::OnTheWorkerThreads(observe) => (
+            StateReportRetention::ObserveOnTheWorkerThread(observe),
+            None,
+        ),
+    };
+    let has_observer = match retention {
+        StateReportRetention::HandEachStateToObserver
+        | StateReportRetention::ObserveOnTheWorkerThread(_) => true,
+        StateReportRetention::CountOnly => false,
+    };
     let (progress_settings, shard_run) = match resume {
         Layer0Resume::NoProgressFile => (None, None),
         Layer0Resume::KeepProgressFile(settings) => (Some(settings), None),
@@ -3165,10 +3453,6 @@
         .collect();
     let restored_slice_count = restored_slices.len();
     let spawned_worker_threads = parallelism.worker_threads.get().min(slices_to_run.len());
-    let retention = match observe_state {
-        Some(_) => StateReportRetention::HandEachStateToObserver,
-        None => StateReportRetention::CountOnly,
-    };
     // 分片跑一片时两行 LAYER0_PARALLEL_* 的 states= / slices= 是这一片的，另带整条流的数；不分片时什么都不加，与分片之前逐字相同。
     let shard_fields = shard.map_or_else(String::new, |shard| {
         format!(
@@ -4629,7 +4913,12 @@
 
 #[cfg(test)]
 mod oracle_instance_tests {
-    use super::{oracle_violation_for_versions, PublishedVersion};
+    use super::{
+        classified_oracle_violation_for_the_version_table,
+        classified_oracle_violation_for_versions, oracle_violation_for_versions,
+        Layer0OracleViolationKind, Layer0VersionTable, PublishedVersion,
+        PublishedVersionWithoutAFile,
+    };
     use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
     use singlefs_core::recovery::RecoveryOutcome;
 
@@ -4696,6 +4985,234 @@
         );
     }
 
+    /// 多记录流（`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs`）的版本表：实例 1 的
+    /// A、B、C 三次带文件的发布落在第 3、4、5 代（第 1、2 代是两次暖机，根下面没有文件）。
+    fn the_multi_record_publish_stream_versions() -> Vec<PublishedVersion> {
+        [
+            (3, b"A".as_slice()),
+            (4, b"B".as_slice()),
+            (5, b"C".as_slice()),
+        ]
+        .into_iter()
+        .map(|(checkpoint_txg, content)| PublishedVersion {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(checkpoint_txg),
+            content: content.to_vec(),
+        })
+        .collect()
+    }
+
+    /// 改「根不在表里只看比它旧的最新一版」之前的判法（逐字照那一版的分支写）：报没有文件、根不在表里，任一更旧版有文件就是违例。
+    fn kind_judged_by_any_older_version_with_a_file(
+        outcome: &RecoveryOutcome,
+        effective_root: (InstanceGeneration, CheckpointTxg),
+        newest_persisted_root: Option<(CheckpointTxg, InstanceGeneration)>,
+        versions: &[PublishedVersion],
+    ) -> Option<Layer0OracleViolationKind> {
+        let (effective_instance, effective_txg) = effective_root;
+        if newest_persisted_root.is_some_and(|newest| (effective_txg, effective_instance) < newest)
+        {
+            return Some(Layer0OracleViolationKind::RecoveredToAnOlderRoot);
+        }
+        let version = versions.iter().find(|version| {
+            version.checkpoint_txg == effective_txg && version.instance == effective_instance
+        });
+        match (outcome, version) {
+            (RecoveryOutcome::Failed { .. }, _) => Some(Layer0OracleViolationKind::ReadFailed),
+            (RecoveryOutcome::NoFile { .. }, None) => versions
+                .iter()
+                .any(|candidate_version| {
+                    (candidate_version.checkpoint_txg, candidate_version.instance)
+                        < (effective_txg, effective_instance)
+                })
+                .then_some(Layer0OracleViolationKind::NoFileWhereAnOlderRootHasOne),
+            (RecoveryOutcome::NoFile { .. }, Some(_)) => {
+                Some(Layer0OracleViolationKind::NoFileWhereThisRootHasOne)
+            }
+            (RecoveryOutcome::FileRead { .. }, None) => {
+                Some(Layer0OracleViolationKind::FileReadWhereThisRootHasNone)
+            }
+            (RecoveryOutcome::FileRead { content, .. }, Some(version)) => {
+                (*content != version.content).then_some(Layer0OracleViolationKind::WrongContent)
+            }
+        }
+    }
+
+    /// 文件从不消失的流上，「根不在表里只看比它旧的最新一版」与「任一更旧版有文件」判得逐项相同：多记录流与两实例同 txg 两张版本表，
+    /// 实例 0–2、第 0–8 代的每个根，报没有文件、读出表里每一版的内容、读出表里没有的内容，盘上最新根槽没有或就是这个根，两种判法交回的违例类相同。
+    #[test]
+    fn a_table_where_the_file_never_vanishes_judges_a_root_not_in_it_like_any_older_version_with_a_file(
+    ) {
+        for versions in [
+            the_multi_record_publish_stream_versions(),
+            two_instances_at_txg_seven(),
+        ] {
+            let mut judged_pairs = 0usize;
+            for instance in 0..=2 {
+                for checkpoint_txg in 0..=8 {
+                    let root = (InstanceGeneration(instance), CheckpointTxg(checkpoint_txg));
+                    let mut outcomes = vec![RecoveryOutcome::NoFile { root }];
+                    outcomes.extend(
+                        versions
+                            .iter()
+                            .map(|version| version.content.clone())
+                            .chain([b"not written by any version".to_vec()])
+                            .map(|content| RecoveryOutcome::FileRead { root, content }),
+                    );
+                    for outcome in &outcomes {
+                        for newest_persisted_root in [None, Some((root.1, root.0))] {
+                            assert_eq!(
+                                classified_oracle_violation_for_versions(
+                                    outcome,
+                                    Some(root),
+                                    newest_persisted_root,
+                                    &versions,
+                                )
+                                .map(|violation| violation.kind),
+                                kind_judged_by_any_older_version_with_a_file(
+                                    outcome,
+                                    root,
+                                    newest_persisted_root,
+                                    &versions,
+                                ),
+                                "根 {root:?}、{outcome:?}、盘上最新根槽 {newest_persisted_root:?}：两种判法应当相同"
+                            );
+                            judged_pairs += 1;
+                        }
+                    }
+                }
+            }
+            assert_eq!(
+                judged_pairs,
+                3 * 9 * (versions.len() + 2) * 2,
+                "每个根、每种结局、两种最新根槽都判到了"
+            );
+        }
+    }
+
+    /// 多记录流之后 unlink 的版本表（`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs`）：
+    /// A、B、C 有文件，第 6 代（删除意图那次）起没有文件，之后两批与最后一批（第 7–9 代）不进表。
+    fn versions_without_a_file_from_txg_six() -> Vec<PublishedVersionWithoutAFile> {
+        vec![PublishedVersionWithoutAFile {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(6),
+        }]
+    }
+
+    fn kind_judged_against_the_table(
+        outcome: &RecoveryOutcome,
+        effective_root: (InstanceGeneration, CheckpointTxg),
+        versions: Layer0VersionTable<'_>,
+    ) -> Option<Layer0OracleViolationKind> {
+        classified_oracle_violation_for_the_version_table(
+            outcome,
+            Some(effective_root),
+            Some((effective_root.1, effective_root.0)),
+            versions,
+        )
+        .map(|violation| violation.kind)
+    }
+
+    /// D8（核心索引结构） 已定项 16 第 4 条：意图那次起冷走读答没有文件。落在意图那一版、或落在它之后不在表里的根（各批）上报没有文件不是违例；
+    /// 在那几个根上读出内容、落在 C 上报没有文件照旧是违例。
+    #[test]
+    fn no_file_from_the_version_without_a_file_on_is_not_a_violation_and_a_file_read_there_is() {
+        let with_a_file = the_multi_record_publish_stream_versions();
+        let without_a_file = versions_without_a_file_from_txg_six();
+        let table = Layer0VersionTable::new(&with_a_file, &without_a_file);
+        for checkpoint_txg in 6..=9 {
+            let root = (InstanceGeneration(1), CheckpointTxg(checkpoint_txg));
+            assert_eq!(
+                kind_judged_against_the_table(&RecoveryOutcome::NoFile { root }, root, table),
+                None,
+                "第 {checkpoint_txg} 代（意图那次或之后的一批）报没有文件不是违例"
+            );
+            assert_eq!(
+                kind_judged_against_the_table(
+                    &RecoveryOutcome::FileRead {
+                        root,
+                        content: b"C".to_vec(),
+                    },
+                    root,
+                    table
+                ),
+                Some(Layer0OracleViolationKind::FileReadWhereThisRootHasNone),
+                "第 {checkpoint_txg} 代起没有文件，读出 C 的内容是违例"
+            );
+        }
+        let root_of_c = (InstanceGeneration(1), CheckpointTxg(5));
+        assert_eq!(
+            kind_judged_against_the_table(
+                &RecoveryOutcome::NoFile { root: root_of_c },
+                root_of_c,
+                table
+            ),
+            Some(Layer0OracleViolationKind::NoFileWhereThisRootHasOne),
+            "C 那一版有文件，报没有是违例"
+        );
+    }
+
+    /// 根不在表里只看比它旧的最新那一项：文件没了之后又有一版带文件的（第 8 代），落在它后面的第 9 代报没有文件是违例、点名第 8 代；
+    /// 落在没有文件那一版与第 8 代之间的第 7 代报没有文件不是违例。
+    #[test]
+    fn a_root_not_in_the_table_is_judged_by_the_newest_older_entry_only() {
+        let with_a_file = vec![
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(3),
+                content: b"A".to_vec(),
+            },
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(8),
+                content: b"written after the file vanished".to_vec(),
+            },
+        ];
+        let without_a_file = versions_without_a_file_from_txg_six();
+        let table = Layer0VersionTable::new(&with_a_file, &without_a_file);
+        let root_after_the_newer_file = (InstanceGeneration(1), CheckpointTxg(9));
+        let violation = classified_oracle_violation_for_the_version_table(
+            &RecoveryOutcome::NoFile {
+                root: root_after_the_newer_file,
+            },
+            Some(root_after_the_newer_file),
+            None,
+            table,
+        )
+        .expect("比第 9 代旧的最新一项是第 8 代、有文件：报没有文件是违例");
+        assert_eq!(
+            violation.kind,
+            Layer0OracleViolationKind::NoFileWhereAnOlderRootHasOne
+        );
+        assert!(
+            violation.reason.contains("实例 1 第 8 代下面有文件"),
+            "违例点名比它旧的最新那一版：{}",
+            violation.reason
+        );
+        let root_between = (InstanceGeneration(1), CheckpointTxg(7));
+        assert_eq!(
+            kind_judged_against_the_table(
+                &RecoveryOutcome::NoFile { root: root_between },
+                root_between,
+                table
+            ),
+            None,
+            "比第 7 代旧的最新一项是第 6 代、没有文件"
+        );
+    }
+
+    /// 同一个根既写成有文件、又写成没有文件：版本表自相矛盾，构造时停下。
+    #[test]
+    #[should_panic(expected = "既写成有文件、又写成文件从这一版起没有了")]
+    fn a_root_both_with_and_without_a_file_is_refused_when_the_table_is_built() {
+        let with_a_file = the_multi_record_publish_stream_versions();
+        let without_a_file = vec![PublishedVersionWithoutAFile {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(5),
+        }];
+        let _table = Layer0VersionTable::new(&with_a_file, &without_a_file);
+    }
+
     /// 同 txg 的两条版本是两条版本：落在实例 2 上读出实例 2 的内容不是违例。
     #[test]
     fn the_same_txg_from_two_instances_are_two_versions() {
--- a/crates/singlefs-harness/src/memory_pool.rs
+++ b/crates/singlefs-harness/src/memory_pool.rs
@@ -789,6 +789,15 @@
     pub content: Vec<u8>,
 }
 
+/// 文件从这一版起没有了：哪次发布（实例代号 + checkpoint_txg）之后冷走读答「没有文件」（D8（核心索引结构） 已定项 16 第 4 条：
+/// 删除意图那次发布起）。与 [`PublishedVersion`] 一起排成一条流的版本表：落到的根两边都不在时，看比它旧的最新那一项
+/// （`singlefs_checker_tier::crash::Layer0VersionTable`）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct PublishedVersionWithoutAFile {
+    pub instance: InstanceGeneration,
+    pub checkpoint_txg: CheckpointTxg,
+}
+
 /// 这个持久集合把一次发布截在了它的根落盘之前：那次发布的单元写或记录写至少有一次已持久，而它的根槽写没持久。
 #[must_use]
 pub fn some_publish_persisted_without_its_root(
--- /dev/null
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs
@@ -0,0 +1,933 @@
+//! checker 档模块：crash、layer0_progress、writable_mount_phases
+//! 删除类发布崩在中间（K-U6 F1：接在多记录流后面 unlink 那个文件）：多记录流「取号 → 暖机 × 2 → A → B → C」之后
+//! unlink 第一个文件（C 之后三个数据单元）：意图那次发布 → 按只供测试的批大小 1 删两批 → 最后一批（D8（核心索引结构） 已定项 16），
+//! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。每个状态跑恢复 + 多版本 oracle、池级 checker、记录核对器；
+//! 版本表是 A、B、C 三版有文件，意图那一版起没有文件（D8（核心索引结构） 已定项 16 第 4 条，`crash::Layer0VersionTable`），
+//! 两批与最后一批不进表、照比它旧的最新那一项（意图那一版）判。另逐状态核三样（快档与全量都核）：
+//! - 多记录发布的发布边界（照多记录流）；
+//! - 删除那几次发布崩在中间：这次发布的单元写落了至少一个、记录一份都没落、根没落 ⇒ 实际走的根是上一版
+//!   （意图那次的上一版是 C，读回 C 的内容；某一批的上一版是意图或上一批，nlink = 0，答没有文件）；
+//! - 可写挂载一次：续做把队列走到空（D8（核心索引结构） 已定项 5「续做是每次恢复都跑的常规动作」、已定项 16 第 6 条），
+//!   续做的步数等于恢复落到的那一版还差几步。挂载是每个状态上的重活，观察者放在工作线程上跑
+//!   （`crash::Layer0StateObserving::OnTheWorkerThreads`），一个工作线程、四个工作线程与观察者在调用线程上三种跑法的计数逐项相同（快档对拍）。
+//!
+//! **比已有的流多罩了什么**：已有的层 0 流上没有删除类发布；这里意图那次（13 个单元：墓碑容器、`deleted_inodes` 根、inode 叶容器与根……）、
+//! 两批（12、11 个单元）、最后一批（10 个单元）各自的单元段、记录段、根槽写与之后的系统配置槽轮换第一次被枚举到；
+//! 多跑的一步是每个状态上的可写挂载（取号、写行、暖机、续做）。
+//!
+//! 快档只展开写数小于 10 的段（七个单元段不展开，只以整段持久进入后面的状态）；全量标 ignored（层 0 的枚举域 1448083638 个状态，
+//! 七个单元段占 1448083449；每次写只取两态时的闭式 1448083588），登记成崩溃枚举用例，带断点续跑、可双机分片（与多记录流同一个入口）。
+//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。
+
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
+mod common;
+
+use std::num::NonZeroUsize;
+
+use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
+use singlefs_checker_tier::crash::{
+    enumerate_layer0_against_the_version_table_or_one_shard, full_expansion,
+    layer0_state_count_with_torn_in_place_overwrites, Layer0EnumerationOutcome,
+    Layer0ObserverCounts, Layer0Parallelism, Layer0SegmentExpansion, Layer0SliceLength,
+    Layer0StateObserving, Layer0Tally, Layer0VersionTable, Layer0WorkerThreadsSource,
+};
+use singlefs_checker_tier::layer0_progress::Layer0Resume;
+use singlefs_checker_tier::writable_mount_phases::materialized_crash_image;
+use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::journal::record_offset;
+use singlefs_core::mount::{
+    mount_writable_with_test_only_switches_and_deletion_switches, DeletionQueueContinuationOutcome,
+    ShadowLedger,
+};
+use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
+use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
+use singlefs_core::transaction::{
+    publish_sequential_write, publish_unlink_intent, publish_unlink_step, FirstFile, PoolVersion,
+    PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
+};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
+use singlefs_harness::memory_pool::{
+    closed_form_state_count, publishes_in, writes_and_segments, CrashImage, MemoryPool,
+    PublishedVersion, PublishedVersionWithoutAFile, RecordStreamContinuity, RetainedWrite,
+    SparseBlockDevice,
+};
+use singlefs_harness::scenario::{deletion_history_batch_size, deletion_history_switches};
+use singlefs_harness::segments::StepKind;
+
+/// 全量那一趟的流名：进断点续跑的进度文件名与双机分片的账本名（`singlefs_checker_tier::layer0_progress`）。
+const FULL_ENUMERATION_STREAM_NAME: &str = "unlink_after_the_multi_record_publish";
+
+/// unlink 的是第一个文件。
+const UNLINKED_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
+
+/// 意图那次的调用方时间比顺序写晚：与那条记录现有的时间不同，意图写的是不是调用方给的那一个分辨得出。
+const SECONDS_FROM_THE_SEQUENTIAL_WRITES_TO_THE_INTENT: u64 = 120;
+
+/// 删除段的批数（不含最后一批）：三个数据单元按批大小 1 走「留 2、留 1、最后一批」（K-U6 翻面观测：删除段少于两批不算数）。
+const DELETION_BATCHES_BEFORE_THE_FINAL_BATCH: usize = 2;
+
+/// 快档对拍里多线程那一趟的工作线程数。
+const WORKER_THREADS_OF_THE_MULTI_THREADED_COMPARISON: usize = 4;
+
+/// 恰好要 `data_units` 个数据单元的内容：最后一个单元装一半（与多记录流同一份字节）。
+fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
+    let payload_capacity = data_unit_payload_capacity();
+    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
+        .map(|index| u8::try_from((index * 11 + seed * 3 + 7) % 251).expect("小于 256"))
+        .collect()
+}
+
+fn sequential_write(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
+    let publish_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    let previous = pool.output.clone();
+    let output = publish_sequential_write(
+        &mut writer,
+        &mut pool.allocator,
+        &previous,
+        FirstFile {
+            content,
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
+        },
+        InstanceGeneration(1),
+    )
+    .expect("顺序写");
+    pool.output = output.clone();
+    output
+}
+
+fn publish_the_unlink_intent(pool: &mut BuiltPool) -> TransactionOutput {
+    let publish_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    let previous = pool.output.clone();
+    let output = publish_unlink_intent(
+        &mut writer,
+        &mut pool.allocator,
+        &previous,
+        UNLINKED_FILE,
+        FIXED_WRITE_TIME_SECONDS + SECONDS_FROM_THE_SEQUENTIAL_WRITES_TO_THE_INTENT,
+        InstanceGeneration(1),
+    )
+    .expect("unlink 意图");
+    pool.output = output.clone();
+    output
+}
+
+fn publish_the_next_unlink_step(pool: &mut BuiltPool) -> TransactionOutput {
+    let publish_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    let previous = pool.output.clone();
+    let output = publish_unlink_step(
+        &mut writer,
+        &mut pool.allocator,
+        &previous,
+        UNLINKED_FILE,
+        deletion_history_batch_size(),
+        InstanceGeneration(1),
+    )
+    .expect("unlink 的一批");
+    pool.output = output.clone();
+    output
+}
+
+/// 写表里写到某个 journal 记录槽（两盘同偏移）的那几格。
+fn writes_to_record(writes: &[RetainedWrite], counter: u64) -> Vec<usize> {
+    let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES);
+    writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| write.kind == StepKind::JournalRecord && write.offset == offset)
+        .map(|(index, _)| index)
+        .collect()
+}
+
+fn version_of(output: &TransactionOutput) -> (InstanceGeneration, CheckpointTxg) {
+    (output.root.instance, output.root.checkpoint_txg)
+}
+
+/// 一次多条记录的发布（照多记录流）：记录按 jsn 升序、每条两盘各一份的写表下标，它与上一版的根，上一版的内容。
+struct MultiRecordPublish {
+    record_writes_in_order: Vec<Vec<usize>>,
+    version: (InstanceGeneration, CheckpointTxg),
+    version_before: (InstanceGeneration, CheckpointTxg),
+    content_before: Vec<u8>,
+}
+
+impl MultiRecordPublish {
+    fn is_any_copy_persisted(persisted: &[bool], copies: &[usize]) -> bool {
+        copies.iter().any(|copy| persisted[*copy])
+    }
+
+    fn is_cut_before_its_last_record(&self, persisted: &[bool]) -> bool {
+        let first = self.record_writes_in_order.first().expect("至少两条记录");
+        let last = self.record_writes_in_order.last().expect("至少两条记录");
+        Self::is_any_copy_persisted(persisted, first)
+            && !Self::is_any_copy_persisted(persisted, last)
+    }
+}
+
+fn multi_record_publish(
+    writes: &[RetainedWrite],
+    output: &TransactionOutput,
+    before: &TransactionOutput,
+    content_before: Vec<u8>,
+) -> MultiRecordPublish {
+    let mut record_writes_in_order: Vec<Vec<usize>> = output
+        .earlier_records_of_this_publish
+        .iter()
+        .map(|earlier| writes_to_record(writes, earlier.record.counter))
+        .collect();
+    record_writes_in_order.push(writes_to_record(writes, output.record.counter));
+    for copies in &record_writes_in_order {
+        assert_eq!(copies.len(), 2, "每条记录两盘各一份");
+    }
+    MultiRecordPublish {
+        record_writes_in_order,
+        version: version_of(output),
+        version_before: version_of(before),
+        content_before,
+    }
+}
+
+/// 删除那几次发布是哪一种：上一版带不带文件分得开读回什么。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum DeletionPublishKind {
+    /// 意图那次：上一版是 C，第一个文件活着。
+    Intent,
+    /// 不是最后一批的一批：上一版是意图或上一批，第一个文件 nlink = 0。
+    Batch,
+    /// 最后一批：上一版是最后一个不是最后一批的批。
+    FinalBatch,
+}
+
+/// 删除的一次发布在写表里的样子：它的单元写、它那一条记录的两份写、它的根槽写，它与上一版的根。
+#[derive(Debug)]
+struct DeletionPublish {
+    kind: DeletionPublishKind,
+    unit_writes: Vec<usize>,
+    record_copies: Vec<usize>,
+    root_write: usize,
+    version: (InstanceGeneration, CheckpointTxg),
+    version_before: (InstanceGeneration, CheckpointTxg),
+    /// 恢复落到这一版之后，可写挂载的续做还要发几步才走到空。
+    continuation_steps_from_this_version: usize,
+}
+
+impl DeletionPublish {
+    /// 这一状态停在这次发布中间：单元写落了至少一个，记录一份都没落，根没落。
+    fn is_cut_after_some_unit_and_before_its_record(&self, persisted: &[bool]) -> bool {
+        self.unit_writes.iter().any(|write| persisted[*write])
+            && !self.record_copies.iter().any(|copy| persisted[*copy])
+            && !persisted[self.root_write]
+    }
+}
+
+struct Prepared {
+    base: MemoryPool,
+    writes: Vec<RetainedWrite>,
+    segments: Vec<Vec<usize>>,
+    judged_root_index: usize,
+    /// A、B、C 三版有文件。
+    versions: Vec<PublishedVersion>,
+    /// 意图那一版起没有文件（两批与最后一批不进表，照比它旧的最新那一项判）。
+    versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
+    multi_record_publishes: Vec<MultiRecordPublish>,
+    deletion_publishes: Vec<DeletionPublish>,
+    /// C 那一版：意图那次的上一版，读回它的内容。
+    content_before_the_deletion: Vec<u8>,
+}
+
+impl Prepared {
+    fn version_table(&self) -> Layer0VersionTable<'_> {
+        Layer0VersionTable::new(&self.versions, &self.versions_without_a_file)
+    }
+}
+
+/// 多记录流的三次带文件的发布（A 在 `build_pool` 里）之后 unlink 第一个文件走到最后一批：交回 B、C、意图、各批的输出。
+struct RecordedStream {
+    pool: BuiltPool,
+    first: TransactionOutput,
+    second: TransactionOutput,
+    second_content: Vec<u8>,
+    third: TransactionOutput,
+    third_content: Vec<u8>,
+    deletion_steps: Vec<TransactionOutput>,
+}
+
+fn record_the_stream(tag: &str) -> RecordedStream {
+    let mut pool = build_pool(tag);
+    let first = pool.output.clone();
+    let second_content = content_needing(2, 1);
+    let second = sequential_write(&mut pool, &second_content);
+    let third_content = content_needing(3, 2);
+    let third = sequential_write(&mut pool, &third_content);
+    assert_eq!(third.data_pointers.len(), 3, "C 之后第一个文件三个数据单元");
+    // 步数按计划定死（意图、两批、最后一批），不按「队列空了没有」停：最后一批留着队列里的号时（改坏「最后一批与删意图同事务」），
+    // 录流照样录完，红在判器上（池级 checker I-9.5、可写挂载续做），不红在搭建上。
+    let mut deletion_steps = vec![publish_the_unlink_intent(&mut pool)];
+    for _batch_index in 0..=DELETION_BATCHES_BEFORE_THE_FINAL_BATCH {
+        deletion_steps.push(publish_the_next_unlink_step(&mut pool));
+    }
+    RecordedStream {
+        pool,
+        first,
+        second,
+        second_content,
+        third,
+        third_content,
+        deletion_steps,
+    }
+}
+
+/// 删除那几步的形状：意图之后 nlink = 0、单元照旧；两批各少一个单元、nlink 仍 0、号还在队列里；最后一批删记录与最后一个单元。
+/// 最后一批之后队列空不空不在这里判：那是「最后一批与删意图同事务」，留给判器（池级 checker I-9.5、可写挂载续做）红。
+fn assert_the_shape_of_the_deletion_steps(deletion_steps: &[TransactionOutput]) {
+    assert_eq!(
+        deletion_steps.len(),
+        1 + DELETION_BATCHES_BEFORE_THE_FINAL_BATCH + 1,
+        "意图、两批、最后一批"
+    );
+    for (step_index, step) in deletion_steps
+        .iter()
+        .enumerate()
+        .take(1 + DELETION_BATCHES_BEFORE_THE_FINAL_BATCH)
+    {
+        let record = step
+            .inode_record
+            .expect("最后一批之前第一个文件那条记录还在");
+        assert_eq!(
+            record.nlink, NLINK_OF_AN_INODE_BEING_DELETED,
+            "第 {step_index} 步之后 nlink = 0"
+        );
+        assert_eq!(
+            step.deleted_inodes,
+            vec![UNLINKED_FILE],
+            "第 {step_index} 步之后号在队列里"
+        );
+        assert_eq!(
+            step.data_pointers.len(),
+            3 - step_index,
+            "意图不动单元，之后每批释放尾部一个"
+        );
+    }
+    let final_batch = deletion_steps.last().expect("至少一步");
+    assert!(
+        final_batch.inode_record.is_none() && final_batch.data_pointers.is_empty(),
+        "最后一批删记录、释放最后一个单元"
+    );
+}
+
+fn prepare(tag: &str) -> Prepared {
+    let recorded = record_the_stream(tag);
+    assert_the_shape_of_the_deletion_steps(&recorded.deletion_steps);
+    let pool = &recorded.pool;
+    let base = pool.memory_pool_after_mkfs();
+    let operations = pool.retained_operations();
+    let (writes, segments) =
+        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
+    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
+    assert_eq!(
+        sizes,
+        vec![
+            2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2,
+            22, 2, 1, 2, 20, 2, 1, 2
+        ],
+        "前十九段同多记录流；之后意图那次 26 个单元写（13 个单元两盘各一份）、记录 2、根 1、轮换 2；两批 24、22 个单元写；\
+         最后一批 20 个单元写；各自记录 2、根 1、轮换 2"
+    );
+    let every_write_persisted = vec![true; writes.len()];
+    let publishes = publishes_in(
+        &writes,
+        &every_write_persisted,
+        RecordStreamContinuity::OneRecording,
+    );
+    let mut deletion_publishes = Vec::new();
+    let mut version_before = version_of(&recorded.third);
+    for (step_index, step) in recorded.deletion_steps.iter().enumerate() {
+        let version = version_of(step);
+        let publish = publishes
+            .iter()
+            .find(|publish| {
+                publish.instance == version.0 .0 && publish.checkpoint_txg == version.1 .0
+            })
+            .expect("删除的每一步在写表里有它的根槽写");
+        let record_copies = writes_to_record(&writes, step.record.counter);
+        assert_eq!(record_copies.len(), 2, "删除的每一步一条记录、两盘各一份");
+        assert_eq!(publish.records, record_copies, "删除的一步恰一条记录");
+        let kind = match step_index {
+            0 => DeletionPublishKind::Intent,
+            index if index == recorded.deletion_steps.len() - 1 => DeletionPublishKind::FinalBatch,
+            _ => DeletionPublishKind::Batch,
+        };
+        deletion_publishes.push(DeletionPublish {
+            kind,
+            unit_writes: publish.units.clone(),
+            record_copies,
+            root_write: publish.root,
+            version,
+            version_before,
+            continuation_steps_from_this_version: recorded.deletion_steps.len() - 1 - step_index,
+        });
+        version_before = version;
+    }
+    let root_indexes: Vec<usize> = writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
+        .map(|(index, _)| index)
+        .collect();
+    assert_eq!(
+        root_indexes.len(),
+        9,
+        "暖机两次、A、B、C、意图、两批、最后一批各一条根槽写"
+    );
+    let versions = vec![
+        PublishedVersion {
+            instance: recorded.first.root.instance,
+            checkpoint_txg: recorded.first.root.checkpoint_txg,
+            content: file_content(),
+        },
+        PublishedVersion {
+            instance: recorded.second.root.instance,
+            checkpoint_txg: recorded.second.root.checkpoint_txg,
+            content: recorded.second_content.clone(),
+        },
+        PublishedVersion {
+            instance: recorded.third.root.instance,
+            checkpoint_txg: recorded.third.root.checkpoint_txg,
+            content: recorded.third_content.clone(),
+        },
+    ];
+    let intent = recorded.deletion_steps.first().expect("意图那一步");
+    let versions_without_a_file = vec![PublishedVersionWithoutAFile {
+        instance: intent.root.instance,
+        checkpoint_txg: intent.root.checkpoint_txg,
+    }];
+    let multi_record_publishes = vec![
+        multi_record_publish(&writes, &recorded.second, &recorded.first, file_content()),
+        multi_record_publish(
+            &writes,
+            &recorded.third,
+            &recorded.second,
+            recorded.second_content.clone(),
+        ),
+    ];
+    Prepared {
+        base,
+        writes,
+        segments,
+        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
+        versions,
+        versions_without_a_file,
+        multi_record_publishes,
+        deletion_publishes,
+        content_before_the_deletion: recorded.third_content,
+    }
+}
+
+/// 每个状态上核发布边界（照多记录流）：一次发布的第一条记录落了、末条没落 ⇒ 走的是上一版的根、读回上一版。
+fn observe_the_publish_boundary(
+    prepared: &Prepared,
+    image: &CrashImage<'_>,
+    report: &RecoveryReport,
+    counts: &mut Layer0ObserverCounts,
+) {
+    for (publish_index, publish) in prepared.multi_record_publishes.iter().enumerate() {
+        if !publish.is_cut_before_its_last_record(&image.persisted) {
+            continue;
+        }
+        counts.add(&format!("cut_in_multi_record_publish_{publish_index}"), 1);
+        assert_eq!(
+            report.effective_root,
+            Some(publish.version_before),
+            "第 {publish_index} 次多记录发布的末条没落：那次发布整体不施加，走的是上一版的根（{:?}）",
+            publish.version
+        );
+        let RecoveryOutcome::FileRead { content, .. } = &report.outcome else {
+            panic!("要读回上一版，实际 {:?}", report.outcome);
+        };
+        assert!(
+            *content == publish.content_before,
+            "读回上一版：文件是旧长度 {} 字节，实际 {} 字节",
+            publish.content_before.len(),
+            content.len()
+        );
+    }
+}
+
+/// 每个状态上核删除的中间态：删除的一次发布停在中间（单元写落了、记录没落、根没落）⇒ 走的是上一版的根；
+/// 意图那次的上一版是 C，读回 C 的内容；某一批的上一版第一个文件 nlink = 0，答没有文件（D8（核心索引结构） 已定项 16 第 4 条）。
+fn observe_the_deletion_cut_in_the_middle(
+    prepared: &Prepared,
+    image: &CrashImage<'_>,
+    report: &RecoveryReport,
+    counts: &mut Layer0ObserverCounts,
+) {
+    for (publish_index, publish) in prepared.deletion_publishes.iter().enumerate() {
+        if !publish.is_cut_after_some_unit_and_before_its_record(&image.persisted) {
+            continue;
+        }
+        counts.add(&format!("cut_in_deletion_publish_{publish_index}"), 1);
+        assert_eq!(
+            report.effective_root,
+            Some(publish.version_before),
+            "删除的第 {publish_index} 步（{:?}）停在中间：那次发布不施加，走的是上一版的根",
+            publish.kind
+        );
+        match (publish.kind, &report.outcome) {
+            (DeletionPublishKind::Intent, RecoveryOutcome::FileRead { content, .. }) => {
+                assert!(
+                    *content == prepared.content_before_the_deletion,
+                    "崩在意图那次中间读回删之前那一版：{} 字节，实际 {} 字节",
+                    prepared.content_before_the_deletion.len(),
+                    content.len()
+                );
+            }
+            (
+                DeletionPublishKind::Batch | DeletionPublishKind::FinalBatch,
+                RecoveryOutcome::NoFile { .. },
+            ) => {}
+            (kind, outcome) => panic!(
+                "删除的第 {publish_index} 步（{kind:?}）停在中间：意图那次要读回 C、各批要答没有文件，实际 {outcome:?}"
+            ),
+        }
+    }
+}
+
+/// 恢复落到的是哪一类版本：挂载之后该是哪一种池版本、续做还差几步按它定。
+#[derive(Clone, Copy, Debug)]
+enum LandedVersion<'prepared> {
+    /// mkfs 或两次暖机：A 之前，根下面没有树。
+    BeforeTheFirstFile,
+    /// A、B、C 之一：有文件、队列空。
+    WithAFile,
+    /// 意图、两批、最后一批之一。
+    OneOfTheDeletionPublishes(&'prepared DeletionPublish),
+}
+
+fn landed_version<'prepared>(
+    prepared: &'prepared Prepared,
+    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
+) -> LandedVersion<'prepared> {
+    let effective_root = effective_root.expect("快档与全量的每个状态都择到根（oracle 另判）");
+    if let Some(deletion_publish) = prepared
+        .deletion_publishes
+        .iter()
+        .find(|publish| publish.version == effective_root)
+    {
+        return LandedVersion::OneOfTheDeletionPublishes(deletion_publish);
+    }
+    if prepared
+        .versions
+        .iter()
+        .any(|version| (version.instance, version.checkpoint_txg) == effective_root)
+    {
+        return LandedVersion::WithAFile;
+    }
+    LandedVersion::BeforeTheFirstFile
+}
+
+/// 这一状态上可写挂载一次（批大小同录流，三个容量照产品路径）：续做把队列走到空，步数等于恢复落到的那一版还差几步；
+/// 落到删除那几版上的，挂完是「有树、文件集合为空」那一版；落到 A、B、C 上的挂完照旧有文件；落到 A 之前的挂完照旧没有树。
+fn observe_the_mount_continuation(
+    prepared: &Prepared,
+    image: &CrashImage<'_>,
+    report: &RecoveryReport,
+    counts: &mut Layer0ObserverCounts,
+) {
+    let materialized = materialized_crash_image(image);
+    let mount_parameters = parameters();
+    let mut devices: Vec<_> = materialized
+        .devices
+        .iter()
+        .map(|(identity, sectors)| {
+            let mut device = SparseBlockDevice::new(
+                materialized.device_size_in_bytes,
+                PhysicalBlockSizeInBytes(mount_parameters.geometry.physical_block_size),
+            );
+            device.image = sectors.clone();
+            (*identity, device)
+        })
+        .collect();
+    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
+        &mount_parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_history_switches(),
+    )
+    .unwrap_or_else(|error| panic!("每个崩溃状态上可写挂载都做得成：{error:?}"));
+    assert!(
+        matches!(
+            mounted.output.deletion_queue_continuation_outcome,
+            DeletionQueueContinuationOutcome::WalkedToEmpty
+        ),
+        "续做把队列走到空：{:?}",
+        mounted.output.deletion_queue_continuation_outcome
+    );
+    let landed = landed_version(prepared, report.effective_root);
+    let expected_continuation_steps = match landed {
+        LandedVersion::OneOfTheDeletionPublishes(publish) => {
+            publish.continuation_steps_from_this_version
+        }
+        LandedVersion::BeforeTheFirstFile | LandedVersion::WithAFile => 0,
+    };
+    assert_eq!(
+        mounted.output.deletion_queue_continuation.len(),
+        expected_continuation_steps,
+        "恢复落到 {:?}：续做还差几步就发几步",
+        report.effective_root
+    );
+    counts.add(
+        &format!("mount_continuation_steps_{expected_continuation_steps}"),
+        1,
+    );
+    match (&mounted.current, landed) {
+        (PoolVersion::WithTreesAndNoFile(current), LandedVersion::OneOfTheDeletionPublishes(_)) => {
+            assert!(current.deleted_inodes.is_empty(), "挂完队列是空的");
+        }
+        (PoolVersion::WithFile(current), LandedVersion::WithAFile) => {
+            assert!(
+                current.deleted_inodes.is_empty(),
+                "删除之前的版本队列本来就空"
+            );
+        }
+        (PoolVersion::WithoutFile(_), LandedVersion::BeforeTheFirstFile) => {
+            counts.add("mount_before_the_first_file", 1);
+        }
+        (current, landed) => panic!(
+            "落到删除那几版上的挂完是有树、文件集合为空那一版，落到 A、B、C 上的带文件，落到 A 之前的没有树：落到 {landed:?}，挂完 {current:?}"
+        ),
+    }
+}
+
+/// 每个状态上核的三样：发布边界、删除的中间态、可写挂载续做。只读 `prepared`，各工作线程共用一份。
+fn observe_every_state(
+    prepared: &Prepared,
+    image: &CrashImage<'_>,
+    report: &RecoveryReport,
+    counts: &mut Layer0ObserverCounts,
+) {
+    observe_the_publish_boundary(prepared, image, report, counts);
+    observe_the_deletion_cut_in_the_middle(prepared, image, report, counts);
+    observe_the_mount_continuation(prepared, image, report, counts);
+}
+
+fn assert_clean(tally: &Layer0Tally) {
+    assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);
+    assert_eq!(
+        tally.ignored_violations, 0,
+        "不看 journal 那一遍恢复同样过 oracle：{:?}",
+        tally.first_ignored_violation
+    );
+    assert_eq!(tally.failed_states, 0, "走读一次都不失败");
+    assert_eq!(
+        (
+            tally.record_root_without_record,
+            tally.record_claimed_state_missing_unit
+        ),
+        (0, 0),
+        "记录核对器两条判据"
+    );
+    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
+        assert_eq!(
+            tally
+                .checker_violated_states
+                .get(invariant)
+                .copied()
+                .unwrap_or(0),
+            0,
+            "{invariant} 判违例：{:?}",
+            tally.checker_first_violation.get(invariant)
+        );
+    }
+}
+
+fn expansion_of_the_segments_shorter_than_ten(
+    _segment_index: usize,
+    segment: &[usize],
+) -> Layer0SegmentExpansion {
+    if segment.len() < 10 {
+        Layer0SegmentExpansion::EveryProperSubset
+    } else {
+        Layer0SegmentExpansion::NotExpanded
+    }
+}
+
+/// 逐状态的观察者在哪个线程上看。
+#[derive(Clone, Copy, Debug)]
+enum ObserverThread {
+    WorkerThreads,
+    CallingThread,
+}
+
+/// 快档那一趟：七个单元段不展开，每个状态交 [`observe_every_state`]，不留进度文件。
+fn enumerate_the_quick_tier(
+    prepared: &Prepared,
+    parallelism: Layer0Parallelism,
+    observer_thread: ObserverThread,
+) -> Layer0Tally {
+    let observe_on_the_worker_threads: &(dyn Fn(&CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts)
+          + Sync) = &|image, report, counts| observe_every_state(prepared, image, report, counts);
+    let observe_on_the_calling_thread: &mut dyn FnMut(
+        &CrashImage<'_>,
+        &RecoveryReport,
+        &mut Layer0ObserverCounts,
+    ) = &mut |image, report, counts| observe_every_state(prepared, image, report, counts);
+    let observing = match observer_thread {
+        ObserverThread::WorkerThreads => {
+            Layer0StateObserving::OnTheWorkerThreads(observe_on_the_worker_threads)
+        }
+        ObserverThread::CallingThread => {
+            Layer0StateObserving::OnTheCallingThread(observe_on_the_calling_thread)
+        }
+    };
+    match enumerate_layer0_against_the_version_table_or_one_shard(
+        &prepared.base,
+        &prepared.writes,
+        &prepared.segments,
+        prepared.judged_root_index,
+        prepared.version_table(),
+        &expansion_of_the_segments_shorter_than_ten,
+        parallelism,
+        observing,
+        &Layer0Resume::NoProgressFile,
+    ) {
+        Layer0EnumerationOutcome::WholeStream(tally) => tally,
+        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(written) => panic!(
+            "不留进度文件、不分片的一趟交回整条流的计数，实际写了第 {} 片的账本",
+            written.shard.text()
+        ),
+    }
+}
+
+fn worker_threads_given_by_the_test(worker_threads: usize) -> Layer0Parallelism {
+    Layer0Parallelism {
+        worker_threads: NonZeroUsize::new(worker_threads).expect("工作线程数不是 0"),
+        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
+        slice_length: Layer0SliceLength::ScaledToWorkerThreads,
+    }
+}
+
+/// 平时跑的那一份：七个单元段不展开，其余每段任意真子集；每个状态核发布边界、删除的中间态，再在工作线程上可写挂载一次核续做走到空。
+#[test]
+fn every_crash_state_of_an_unlink_after_the_multi_record_publish_recovers_to_the_version_before_the_cut_and_the_mount_continues_to_empty(
+) {
+    let prepared = prepare("unlink-after-multi-record-layer0-fast");
+    let tally = enumerate_the_quick_tier(
+        &prepared,
+        Layer0Parallelism::from_environment(),
+        ObserverThread::WorkerThreads,
+    );
+    let expanded: Vec<Vec<usize>> = prepared
+        .segments
+        .iter()
+        .filter(|segment| segment.len() < 10)
+        .cloned()
+        .collect();
+    assert_eq!(
+        closed_form_state_count(&expanded),
+        1 + 10 * 3 + 7 * 3 + 9 + 15 + 63,
+        "每次写只取两态时展开的段按闭式数：1 + 十个系统配置槽 2 写段各 3 + 七个记录 2 写段各 3 + 九个 1 写段各 1 + 4 写段 15 + 6 写段 63"
+    );
+    assert_eq!(
+        tally.states,
+        layer0_state_count_with_torn_in_place_overwrites(
+            &prepared.base,
+            &prepared.writes,
+            &prepared.segments,
+            &expansion_of_the_segments_shorter_than_ten,
+        ),
+        "展开的段按层 0 的枚举域数（原地覆写取三态）"
+    );
+    assert_eq!(
+        tally.states,
+        1 + 10 * (3 * 3 - 1) + 7 * 3 + 9 + 15 + 63,
+        "1 + 十个系统配置槽 2 写段各 3² − 1 + 七个记录 2 写段各 3 + 九个 1 写段各 1 + 4 写段 15 + 6 写段 63"
+    );
+    assert_eq!(
+        tally.observed_states, tally.states,
+        "每个状态都在工作线程上看过"
+    );
+    let counts = &tally.observer_counts;
+    assert_eq!(
+        [
+            counts.count_named("cut_in_multi_record_publish_0"),
+            counts.count_named("cut_in_multi_record_publish_1"),
+        ],
+        [3, 12],
+        "崩在多记录发布的记录之间的状态真跑到了（同多记录流）"
+    );
+    // 单元段不展开：每次删除发布停在中间的状态只有「单元全落、记录一份没落」那一个（记录段的空子集）。
+    assert_eq!(
+        [0, 1, 2, 3]
+            .map(|publish_index| counts
+                .count_named(&format!("cut_in_deletion_publish_{publish_index}"))),
+        [1, 1, 1, 1],
+        "意图、两批、最后一批各自停在中间的状态真跑到了"
+    );
+    assert_eq!(
+        (0..=3)
+            .map(|steps| counts.count_named(&format!("mount_continuation_steps_{steps}")))
+            .sum::<u64>(),
+        tally.states,
+        "每个状态都可写挂载过一次"
+    );
+    for steps in 1..=3 {
+        assert!(
+            counts.count_named(&format!("mount_continuation_steps_{steps}")) > 0,
+            "恢复落到还差 {steps} 步的那一版、挂载续做 {steps} 步的状态真跑到了"
+        );
+    }
+    println!(
+        "LAYER0_UNLINK_AFTER_MULTI_RECORD_FAST states={} observer {:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
+        tally.states,
+        counts,
+        tally.states_by_publish_text(),
+        tally.checker_counts_by_invariant()
+    );
+    assert_clean(&tally);
+}
+
+/// 观察者挪进工作线程之后判定不变：同一趟快档，一个工作线程、四个工作线程（观察者都在工作线程上）与观察者在调用线程上，
+/// 三份计数（状态数、违例、checker 逐条、发现表、观察者计数）逐项相同。
+#[test]
+fn the_unlink_stream_quick_tier_counts_the_same_on_one_worker_thread_on_four_and_with_the_observer_on_the_calling_thread(
+) {
+    let prepared = prepare("unlink-after-multi-record-layer0-threads");
+    let on_one_worker_thread = enumerate_the_quick_tier(
+        &prepared,
+        worker_threads_given_by_the_test(1),
+        ObserverThread::WorkerThreads,
+    );
+    let on_several_worker_threads = enumerate_the_quick_tier(
+        &prepared,
+        worker_threads_given_by_the_test(WORKER_THREADS_OF_THE_MULTI_THREADED_COMPARISON),
+        ObserverThread::WorkerThreads,
+    );
+    let observed_on_the_calling_thread = enumerate_the_quick_tier(
+        &prepared,
+        worker_threads_given_by_the_test(WORKER_THREADS_OF_THE_MULTI_THREADED_COMPARISON),
+        ObserverThread::CallingThread,
+    );
+    assert_eq!(
+        on_several_worker_threads, on_one_worker_thread,
+        "观察者在工作线程上：四个工作线程与一个工作线程的计数逐项相同"
+    );
+    assert_eq!(
+        on_several_worker_threads, observed_on_the_calling_thread,
+        "观察者在工作线程上与在调用线程上的计数逐项相同"
+    );
+    assert_eq!(
+        on_several_worker_threads.observed_states, on_several_worker_threads.states,
+        "每个状态都看过"
+    );
+}
+
+/// 全量：35 段。七个单元段各 2^n − 1（n = 26、24、22、20 是删除那四次），每个状态两遍恢复 + checker，逐状态核发布边界、删除的中间态，
+/// 并在工作线程上可写挂载一次核续做走到空。登记成崩溃枚举用例，checker-tier-layer0-replay 认下面打印的
+/// `LAYER0_UNLINK_AFTER_MULTI_RECORD` 行里 `exhaustive=true`；带断点续跑、可双机分片（同多记录流）。
+#[test]
+#[ignore = "全量 1448083638 个状态（十四亿多）、每个两遍恢复 + checker + 一次可写挂载；门禁 checker-tier-layer0-replay --full 在 release 下跑，带断点续跑、可双机分片"]
+fn full_enumeration_of_the_unlink_after_the_multi_record_publish_stream_is_exhaustive_and_clean() {
+    let prepared = prepare("unlink-after-multi-record-layer0-full");
+    let closed_form_with_two_states_per_write = closed_form_state_count(&prepared.segments);
+    assert_eq!(
+        closed_form_with_two_states_per_write,
+        1 + 10 * 3
+            + 7 * 3
+            + 9
+            + ((1 << 24) - 1)
+            + ((1 << 28) - 1)
+            + 15
+            + ((1 << 30) - 1)
+            + 63
+            + ((1 << 26) - 1)
+            + ((1 << 24) - 1)
+            + ((1 << 22) - 1)
+            + ((1 << 20) - 1),
+        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)，三十五段"
+    );
+    assert_eq!(closed_form_with_two_states_per_write, 1_448_083_588);
+    let closed_form = layer0_state_count_with_torn_in_place_overwrites(
+        &prepared.base,
+        &prepared.writes,
+        &prepared.segments,
+        &full_expansion,
+    );
+    assert_eq!(
+        closed_form,
+        1 + 10 * (3 * 3 - 1)
+            + 7 * 3
+            + 9
+            + ((1 << 24) - 1)
+            + ((1 << 28) - 1)
+            + 15
+            + ((1 << 30) - 1)
+            + 63
+            + ((1 << 26) - 1)
+            + ((1 << 24) - 1)
+            + ((1 << 22) - 1)
+            + ((1 << 20) - 1),
+        "枚举域的闭式：十个系统配置槽 2 写段各 3² − 1、七个记录 2 写段各 3、九个 1 写段、七个单元段各 2^n − 1、4 写段 15、6 写段 63"
+    );
+    assert_eq!(closed_form, 1_448_083_638);
+    let observe_on_the_worker_threads: &(dyn Fn(&CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts)
+          + Sync) = &|image, report, counts| observe_every_state(&prepared, image, report, counts);
+    let tally = match enumerate_layer0_against_the_version_table_or_one_shard(
+        &prepared.base,
+        &prepared.writes,
+        &prepared.segments,
+        prepared.judged_root_index,
+        prepared.version_table(),
+        &full_expansion,
+        Layer0Parallelism::from_environment(),
+        Layer0StateObserving::OnTheWorkerThreads(observe_on_the_worker_threads),
+        &Layer0Resume::from_environment(FULL_ENUMERATION_STREAM_NAME),
+    ) {
+        Layer0EnumerationOutcome::WholeStream(tally) => tally,
+        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(_written) => return,
+    };
+    println!(
+        "LAYER0_UNLINK_AFTER_MULTI_RECORD states={} closed_form={closed_form} exhaustive={} violations={} observer {:?} states_by_publish=[{}] checker {} first_violation={}",
+        tally.states,
+        tally.states == closed_form,
+        tally.violations,
+        tally.observer_counts,
+        tally.states_by_publish_text(),
+        tally.checker_counts_by_invariant(),
+        tally.first_violation.as_deref().unwrap_or("none")
+    );
+    assert_eq!(tally.states, closed_form);
+    assert_eq!(tally.observed_states, closed_form, "每个状态都看过");
+    let counts = &tally.observer_counts;
+    assert_eq!(
+        [
+            counts.count_named("cut_in_multi_record_publish_0"),
+            counts.count_named("cut_in_multi_record_publish_1"),
+        ],
+        [3, 12]
+    );
+    // 单元段 n 个写里至少落一个的真子集 2^n − 2，加记录段的空子集（单元全落、记录一份没落）。
+    assert_eq!(
+        [0, 1, 2, 3]
+            .map(|publish_index| counts
+                .count_named(&format!("cut_in_deletion_publish_{publish_index}"))),
+        [(1 << 26) - 1, (1 << 24) - 1, (1 << 22) - 1, (1 << 20) - 1],
+        "意图、两批、最后一批各自停在中间的每个状态都核到了"
+    );
+    assert_eq!(
+        (0..=3)
+            .map(|steps| counts.count_named(&format!("mount_continuation_steps_{steps}")))
+            .sum::<u64>(),
+        closed_form,
+        "每个状态都可写挂载过一次"
+    );
+    assert_clean(&tally);
+}
````

## 实现员 r：mutations-append.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch33/r/patch/mutations-append.tsv，追加在 `crates/mutations.tsv` 末尾，合入时第 3102–3109 行）

````tsv
第33批 三·4 删除流（接在多记录流后面 unlink）：最后一批不删 deleted_inodes 那条 key（删 key 不与最后一批同一次发布），流的判器红	crates/singlefs-core/src/transaction.rs	DeletionStepOfThePublish::FinalBatch { inode } => keys.retain(|key| *key != inode),	DeletionStepOfThePublish::FinalBatch { .. } => {}	-p singlefs-checker-tier --test crash_enumeration_unlink_after_the_multi_record_publish_stream -- every_crash_state_of_an_unlink_after_the_multi_record_publish_recovers_to_the_version_before_the_cut_and_the_mount_continues_to_empty	every_crash_state_of_an_unlink_after_the_multi_record_publish_recovers_to_the_version_before_the_cut_and_the_mount_continues_to_empty
第33批 三·4 删除流（接在多记录流后面 unlink）：可写挂载不续做（挂载里续做到空那一处），每状态挂载核续做的那条快档红	crates/singlefs-core/src/mount.rs	    if current_version_of_trees.deleted_inodes.is_empty() {	    if current_version_of_trees.deleted_inodes.is_empty() || current_version_of_trees.root.checkpoint_txg.0 > 0 {	-p singlefs-checker-tier --test crash_enumeration_unlink_after_the_multi_record_publish_stream -- every_crash_state_of_an_unlink_after_the_multi_record_publish_recovers_to_the_version_before_the_cut_and_the_mount_continues_to_empty	every_crash_state_of_an_unlink_after_the_multi_record_publish_recovers_to_the_version_before_the_cut_and_the_mount_continues_to_empty
第33批 三·4 层 0 oracle：根不在版本表里时退回看任一更旧版有没有文件（不看比它旧的最新那一项是不是没有文件），删除流各批报没有文件判违例	crates/singlefs-checker-tier/src/crash.rs	                ((with_a_file.checkpoint_txg, with_a_file.instance) > without_a_file)\n                    .then_some(with_a_file)	                let _ = without_a_file;\n                Some(with_a_file)	-p singlefs-checker-tier --test crash_enumeration_unlink_after_the_multi_record_publish_stream -- every_crash_state_of_an_unlink_after_the_multi_record_publish_recovers_to_the_version_before_the_cut_and_the_mount_continues_to_empty	every_crash_state_of_an_unlink_after_the_multi_record_publish_recovers_to_the_version_before_the_cut_and_the_mount_continues_to_empty
第33批 三·4 层 0 oracle：文件从不消失的表上根不在表里、有更旧的带文件版也不判没有文件违例（与「任一更旧版有文件」不再等价）	crates/singlefs-checker-tier/src/crash.rs	            (Some(with_a_file), None) => Some(with_a_file),	            (Some(_with_a_file), None) => None,	-p singlefs-checker-tier --lib -- crash::oracle_instance_tests::a_table_where_the_file_never_vanishes_judges_a_root_not_in_it_like_any_older_version_with_a_file	a_table_where_the_file_never_vanishes_judges_a_root_not_in_it_like_any_older_version_with_a_file
第33批 三·4 层 0 oracle：落在「文件从这一版起没有了」那一版上报没有文件判成违例	crates/singlefs-checker-tier/src/crash.rs	        (RecoveryOutcome::NoFile { .. }, Layer0VersionAtTheRoot::WithoutAFile) => None,	        (RecoveryOutcome::NoFile { .. }, Layer0VersionAtTheRoot::WithoutAFile) => Some(violation(Layer0OracleViolationKind::NoFileWhereThisRootHasOne, String::new())),	-p singlefs-checker-tier --lib -- crash::oracle_instance_tests::no_file_from_the_version_without_a_file_on_is_not_a_violation_and_a_file_read_there_is	no_file_from_the_version_without_a_file_on_is_not_a_violation_and_a_file_read_there_is
第33批 三·4 层 0 oracle：根不在表里时看比它旧的最老一版带文件的、不看最新一版	crates/singlefs-checker-tier/src/crash.rs	            .max_by_key(|version| (version.checkpoint_txg, version.instance));	            .min_by_key(|version| (version.checkpoint_txg, version.instance));	-p singlefs-checker-tier --lib -- crash::oracle_instance_tests::a_root_not_in_the_table_is_judged_by_the_newest_older_entry_only	a_root_not_in_the_table_is_judged_by_the_newest_older_entry_only
第33批 三·4 层 0 版本表：同一个根既有文件又没有文件不拒	crates/singlefs-checker-tier/src/crash.rs	                !versions_with_a_file.iter().any(|with_a_file| {	                versions_with_a_file.iter().any(|with_a_file| {	-p singlefs-checker-tier --lib -- crash::oracle_instance_tests::a_root_both_with_and_without_a_file_is_refused_when_the_table_is_built	a_root_both_with_and_without_a_file_is_refused_when_the_table_is_built
第33批 三·4 层 0 观察者挪进工作线程：工作线程上看的观察者计数不记进那一片，与在调用线程上看的对不上	crates/singlefs-checker-tier/src/crash.rs	                    &mut tally.observer_counts,	                    &mut Layer0ObserverCounts::default(),	-p singlefs-checker-tier --test crash_enumeration_unlink_after_the_multi_record_publish_stream -- the_unlink_stream_quick_tier_counts_the_same_on_one_worker_thread_on_four_and_with_the_observer_on_the_calling_thread	the_unlink_stream_quick_tier_counts_the_same_on_one_worker_thread_on_four_and_with_the_observer_on_the_calling_thread
````

## 主 agent：`crates/mutations.tsv` 第 16 行锚点（R 合入弄断，主 agent 改到新判法同一处）

改前：`                (candidate_version.checkpoint_txg, candidate_version.instance)\n                    < (effective_txg, effective_instance)`

改后：`                (version.checkpoint_txg, version.instance) < (checkpoint_txg, instance)`（`crates/singlefs-checker-tier/src/crash.rs` 第 1055 行），替换文 `false` 与点名的测试不变。
