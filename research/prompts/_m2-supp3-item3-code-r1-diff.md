# 这一轮被判的改动（`git diff HEAD -- crates/`，新模块整份在末尾）

```diff
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index e88b7de..3630c19 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -176,3 +176,5 @@ Z1-a：实例表准入按每次挂载只写一行算	crates/singlefs-core/src/mo
 增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1c；要带调试符号的构建，函数改名会悄悄失效）：只在抬 F 路径的准入里多算一个角色；抬 F 逼近墙的写死用例判出	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten.len() * allocator.devices.len();	        records_before_this_publish + (rewritten.len() + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("raise_rollback_floor"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- raising_the_floor_with_a_second_empty_publish	raising_the_floor_with_a_second_empty_publish_that_fills_the_allocation_node_to_exactly_812_records_succeeds
 增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1d；要带调试符号的构建，函数改名会悄悄失效）：只在回退路径的准入里多算一个角色；回退逼近墙的写死用例判出	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten.len() * allocator.devices.len();	        records_before_this_publish + (rewritten.len() + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("mount_rollback"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_a_root_whose_warm_up	rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds
 增补 3 第 2 件（代码三方第二轮判决第三节第 1 条）：候选排除「低于 F」报成「树表 0 条、第一版不支持」；回退到 F 之下的写死用例判出（门禁五段各只红 1 段）	crates/singlefs-core/src/mount.rs	        return Err(MountError::RollbackTargetNotACandidate {\n            target,\n            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,\n        });	        return Err(MountError::RollbackToVersionWithoutFileUnsupported(target));	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_below_the_effective_floor	rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor
+增补 3 第 3 件：抽 0 个崩溃点时照样抽一个（抽崩溃点会改变这段历史怎么跑，判别力那一半失效）	crates/singlefs-harness/src/crash_injection.rs	    let drawn = crash_points_per_history.min(candidates.len());	    let drawn = crash_points_per_history.max(1).min(candidates.len());	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- drawing_crash_points	drawing_crash_points_does_not_change_the_generated_history
+增补 3 第 3 件：判崩溃点时取择根而不是施加记录前缀之后实际走的那条根（记录已写、根槽未写那一格内容与根身份配不上）	crates/singlefs-harness/src/crash_injection.rs	        let read_back = observed_read_back_after_a_crash(&report);	        let read_back = crate::model_comparison::observed_read_back(\&report.outcome);	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- crash_injection_fast_tier	crash_injection_fast_tier_recovers_only_into_versions_the_model_committed
diff --git a/crates/singlefs-harness/src/crash.rs b/crates/singlefs-harness/src/crash.rs
index 7a9cb30..be60b91 100644
--- a/crates/singlefs-harness/src/crash.rs
+++ b/crates/singlefs-harness/src/crash.rs
@@ -732,14 +732,23 @@ fn root_identity_of_write(write: &RetainedWrite) -> (InstanceGeneration, Checkpo
         StepKind::RootRecordFua,
         "被判的那条写要是根槽 FUA 写"
     );
+    root_identity_written_by(&write.bytes)
+}
+
+/// 一次落在根环里的写写下的根身份：实例代号在偏移 24（4 字节）、checkpoint_txg 在偏移 28（8 字节）。
+/// 崩溃注入（增补 3 第 3 件）拿它从截断了的录制流里数「盘上已持久的最新根槽」，写表与层 0 那一份不共用
+/// （那一份要整条流的 `RetainedWrite`，这里只有录制流本身）。
+///
+/// # Panics
+/// 这次写短于 36 字节：落在根环里的写都是整条根记录，短了说明调用方分错了种类。
+#[must_use]
+pub fn root_identity_written_by(bytes: &[u8]) -> (InstanceGeneration, CheckpointTxg) {
     (
         InstanceGeneration(u32::from_le_bytes(
-            write.bytes[24..28]
-                .try_into()
-                .expect("根记录的实例代号在偏移 24"),
+            bytes[24..28].try_into().expect("根记录的实例代号在偏移 24"),
         )),
         CheckpointTxg(u64::from_le_bytes(
-            write.bytes[28..36]
+            bytes[28..36]
                 .try_into()
                 .expect("根记录的 checkpoint_txg 在偏移 28"),
         )),
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index 905dd10..165cf8b 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -1652,12 +1652,15 @@ pub struct HistoryRun {
     pub tally: HistoryTally,
 }
 
-/// 每一步之后交给观察者的东西：第几步、这一步是什么、结局、此刻的整份镜像。
+/// 每一步之后交给观察者的东西：第几步、这一步是什么、结局、此刻的整份镜像、跟到这一步的理想模型。
+/// 只在这一步没判出失败时调（判出失败的那一步直接停下，观察者看不到），所以模型与镜像都是判过的状态。
 pub struct StepObservation<'run> {
     pub position: StepPosition,
     pub operation: Option<&'run HistoryOperation>,
     pub outcome: Option<&'run StepOutcome>,
     pub image: &'run MemoryPool,
+    /// 崩溃注入（增补 3 第 3 件）每一步之后从它取一次 `committed_versions`，攒成「模型提交过的每一版」。
+    pub model: &'run IdealModel,
 }
 
 fn placement_refusal_member(refusal: &PlacementRefusal) -> &'static str {
@@ -2673,6 +2676,7 @@ pub fn execute_history_with(
             operation: None,
             outcome: None,
             image: &image,
+            model: &pool.model,
         });
         for (step_index, operation) in history.operations.iter().enumerate() {
             let step_position = StepPosition::Operation(step_index);
@@ -2754,6 +2758,7 @@ pub fn execute_history_with(
                 operation: Some(operation),
                 outcome: outcomes.last(),
                 image: &image,
+                model: &pool.model,
             });
         }
         let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(&image);
@@ -2882,7 +2887,9 @@ pub fn raised_floor_lands_only_on_abandoned_roots(
 }
 
 /// 按 checker 的读法：根环里最新那条根的 txg 与一圈的槽数 R × S。
-fn newest_ring_root_and_slot_count(image: &MemoryPool) -> (Option<u64>, Option<u64>) {
+/// 崩溃注入（增补 3 第 3 件）在崩溃镜像上也要这两个事实，才判得出「已知红」第 0 条那一形（根环转过一圈）。
+#[must_use]
+pub fn newest_ring_root_and_slot_count(image: &MemoryPool) -> (Option<u64>, Option<u64>) {
     let (roots, root_ring_slot_count) = ring_roots_newest_first(image);
     (roots.first().map(|(txg, _)| *txg), root_ring_slot_count)
 }
diff --git a/crates/singlefs-harness/src/lib.rs b/crates/singlefs-harness/src/lib.rs
index d0b9307..2f892f4 100644
--- a/crates/singlefs-harness/src/lib.rs
+++ b/crates/singlefs-harness/src/lib.rs
@@ -14,6 +14,7 @@ use singlefs_core::block_device::{
 };
 
 pub mod crash;
+pub mod crash_injection;
 pub mod device_log;
 pub mod first_transaction_regions;
 pub mod hexadecimal;
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index 2628347..8ff3ba3 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -662,6 +662,23 @@ impl IdealModel {
         self.session.is_some()
     }
 
+    /// 模型此刻根环里的每一条根：身份与它下面的文件内容（树表 0 条是 None）。
+    /// 崩溃注入（增补 3 第 3 件）每一步之后并一次，攒成「模型提交过的每一版」的目录，再拿它判崩溃状态恢复到的那一版
+    /// （[`crash_recovery_disagreement`]）。并不掉：根环有 R × S = 24 个槽，一步至多写出四条根（可写挂载的写行加暖机至多三次、
+    /// 抬 F 两次空发布），下一步之前必然还在环里。
+    #[must_use]
+    pub fn committed_versions(&self) -> Vec<(ModelRootKey, Option<Rc<[u8]>>)> {
+        self.ring
+            .values()
+            .map(|root| {
+                (
+                    root.key,
+                    root.file.as_ref().map(|file| Rc::clone(&file.content)),
+                )
+            })
+            .collect()
+    }
+
     fn newest_root(&self) -> &ModelRoot {
         self.ring
             .values()
@@ -1722,6 +1739,100 @@ impl IdealModel {
     }
 }
 
+/// 崩溃之后恢复到的这一版允不允许（增补 3 第 3 件：崩溃注入）。模型这一侧给的是「提交过哪些版、每一版的内容是什么」
+/// （[`IdealModel::committed_versions`] 攒出来的目录），崩溃点那一侧给的是「盘上已持久的最新根槽是哪条根」
+/// （从截断了的录制流里数出来，不经模型也不经实现的择根）。四条判据：
+/// ① 走读不许失败；
+/// ② 择到的根必须是模型提交过的某一版——读回一版从没提交过的，是 D13（验证路线） 已定项 7 那条「记录流里没有的东西不许出现在盘上」
+///    在恢复这一侧的形态；
+/// ③ 盘上已持久的最新根槽是 (T, i) 时，择到的根按 (txg, 实例) 不许比它旧（D22（单元原子性怎么合成） 已定项 7 的择新序：
+///    择新 txg 为主、平局按实例代号高者赢）；
+/// ④ 读回的内容要与模型记的那一版逐字节相同，那一版树表 0 条时只许报没有文件。
+///
+/// 与层 0 的 `crash::oracle_violation_for_versions` 分开写：那一份的版本表由调用方按固定脚本手写，这一份问的是模型
+/// （D13（验证路线） 已定项 5：模型不与 `singlefs-core` 共用代码）。那一份在「走到一条没发布过的更新的根上报没有文件」那一格只在
+/// 有更旧的带文件版本时才判红，这一份认得每一条根有没有文件，两个方向都判得出。
+#[must_use]
+pub fn crash_recovery_disagreement(
+    committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
+    newest_persisted_root: Option<ModelRootKey>,
+    read_back: &ObservedReadBack,
+) -> Option<ModelDisagreement> {
+    let (chosen, content_read_back) = match read_back {
+        ObservedReadBack::Failed { what } => {
+            return Some(ModelDisagreement::new(
+                ModelDisagreementAspect::ColdStartReadBack,
+                "崩溃之后的冷启动要走到一条模型提交过的根上，不许失败".to_string(),
+                format!("走读失败：{what}"),
+            ))
+        }
+        ObservedReadBack::NoFile { root } => (*root, None),
+        ObservedReadBack::FileRead { root, content } => (*root, Some(content)),
+    };
+    if let Some(persisted) = newest_persisted_root {
+        if chosen < persisted {
+            return Some(ModelDisagreement::new(
+                ModelDisagreementAspect::ColdStartReadBack,
+                format!(
+                    "盘上已持久的最新根槽是实例 {} 第 {} 代，不许恢复到比它旧的根",
+                    persisted.instance.0, persisted.checkpoint_txg.0
+                ),
+                format!(
+                    "走到实例 {} 第 {} 代根",
+                    chosen.instance.0, chosen.checkpoint_txg.0
+                ),
+            ));
+        }
+    }
+    let Some(committed) = committed_versions.get(&chosen) else {
+        return Some(ModelDisagreement::new(
+            ModelDisagreementAspect::ColdStartReadBack,
+            "择到的根要是模型提交过的某一版".to_string(),
+            format!(
+                "走到实例 {} 第 {} 代根，模型从没提交过这一版（{}）",
+                chosen.instance.0,
+                chosen.checkpoint_txg.0,
+                describe_observed_read_back(read_back)
+            ),
+        ));
+    };
+    match (content_read_back, committed) {
+        (None, None) => None,
+        (None, Some(content)) => Some(ModelDisagreement::new(
+            ModelDisagreementAspect::ColdStartReadBack,
+            format!(
+                "实例 {} 第 {} 代根下面有文件（{} 字节）",
+                chosen.instance.0,
+                chosen.checkpoint_txg.0,
+                content.len()
+            ),
+            "报了没有文件".to_string(),
+        )),
+        (Some(content), None) => Some(ModelDisagreement::new(
+            ModelDisagreementAspect::ColdStartReadBack,
+            format!(
+                "实例 {} 第 {} 代根树表 0 条，没有文件",
+                chosen.instance.0, chosen.checkpoint_txg.0
+            ),
+            format!("读回了 {} 字节", content.len()),
+        )),
+        (Some(content), Some(committed_content)) => (content.as_slice() != &committed_content[..])
+            .then(|| {
+                ModelDisagreement::new(
+                    ModelDisagreementAspect::ColdStartReadBack,
+                    format!(
+                        "实例 {} 第 {} 代根下面是 {} 字节（末字节 {:?}）",
+                        chosen.instance.0,
+                        chosen.checkpoint_txg.0,
+                        committed_content.len(),
+                        committed_content.last()
+                    ),
+                    format!("读回 {} 字节（末字节 {:?}）", content.len(), content.last()),
+                )
+            }),
+    }
+}
+
 fn describe_answer(answer: &ModelAnswer) -> String {
     if answer.required_refusals.is_empty() {
         let permitted: Vec<&'static str> = answer
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index 5efc813..cb3e643 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -10,7 +10,7 @@ use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
 use singlefs_core::allocator::PlacementRefusal;
 use singlefs_core::block_device::BlockDeviceError;
 use singlefs_core::mount::{InstanceRow, MountError, Mounted, RollbackCandidateExclusion};
-use singlefs_core::recovery::RecoveryOutcome;
+use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
 use singlefs_core::transaction::{
     PoolVersion, PublishError, TransactionOutput, TransactionUnit, ZeroUnitPublishOutput,
 };
@@ -286,6 +286,31 @@ pub fn observed_read_back(outcome: &RecoveryOutcome) -> ObservedReadBack {
     }
 }
 
+/// 崩溃之后的冷启动读回（增补 3 第 3 件）：根取施加 journal 记录前缀之后**实际走的**那条根（`RecoveryReport::effective_root`），
+/// 不取 `RecoveryOutcome` 里带的那条。两者的差别只在崩溃状态上看得见：`crates/singlefs-core/src/recovery.rs` 的 `recover` 里
+/// `root_key` 取的是 `choose_root`（施加之前所选的根），`walk_to_file` 走的却是 `effective_root`（施加之后的根）——
+/// 不建崩溃时记录都在水位之下、两者相等，崩溃状态上记录前缀一施加就不等了，读回的内容属于后者。
+/// 层 0 的 oracle（`crash::oracle_violation_for_versions`）判该读出哪一版拿的也是 `effective_root`。
+#[must_use]
+pub fn observed_read_back_after_a_crash(report: &RecoveryReport) -> ObservedReadBack {
+    let Some((instance, checkpoint_txg)) = report.effective_root else {
+        return ObservedReadBack::Failed {
+            what: format!("没择到根（{:?}）", report.outcome),
+        };
+    };
+    let root = model_root_key(instance, checkpoint_txg);
+    match &report.outcome {
+        RecoveryOutcome::NoFile { .. } => ObservedReadBack::NoFile { root },
+        RecoveryOutcome::FileRead { content, .. } => ObservedReadBack::FileRead {
+            root,
+            content: content.clone(),
+        },
+        RecoveryOutcome::Failed { failure, .. } => ObservedReadBack::Failed {
+            what: format!("{failure:?}（实际走的根 {root:?}）"),
+        },
+    }
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
```

## 新增整份：`crates/singlefs-harness/src/crash_injection.rs`

```rust
//! 崩溃注入（里程碑「第二个事务」增补 3 第 3 件）：第 1 件的每段历史跑的过程中开着内容保留的录制流，按种子抽若干次写、
//! 在那次写之后把流截断，对每个崩溃点重建镜像、跑恢复、跑池级 checker，并问理想模型「恢复到的这一版允不允许」
//! （`crate::model::crash_recovery_disagreement`）。
//!
//! 抽样，不全枚举：全量枚举仍归门禁 54 号的两条固定流（`crate::crash` 的层 0 枚举）。两者分工不同——
//! 层 0 在两条写死的流上把每一段的任意写子集都摆一遍（含乱序持久与撕裂），这里只摆「前缀持久」这一种，
//! 换来的是任意一段随机历史上的崩溃点：关闭重开、可写挂载、回退、抬 F、零单元发布、冷启动都在流里。
//!
//! 每段历史自己怎么跑由 `HistoryExecution` 定；崩溃点只抽在起点之后、最后一个跑完的操作为止的那段流上——
//! 起点（mkfs 与起点那次发布）整个持久，失败那一步的写不抽（模型还没判过它写出的根，目录里没有那一版）。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};

use crate::crash::{root_identity_written_by, MemoryPool};
use crate::history::{
    classify_failure, execute_history_with, generate_history_with_weights,
    newest_ring_root_and_slot_count, FailureObservation, FailureSignature, GeneratedHistory,
    GenerationWeights, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
    HistorySeed, SeededRandomSource, StepPosition, KNOWN_RED_FORMS,
};
use crate::model::{
    crash_recovery_disagreement, ModelDisagreement, ModelRootKey, ObservedReadBack,
};
use crate::model_comparison::{model_root_key, observed_read_back_after_a_crash};
use crate::segments::{FixedGeometry, StepKind};
use crate::{RecordedOperationKind, RetainedOperation, SharedStream};

/// 崩溃注入的工作线程数从这个环境变量取（十进制正整数）；没设就取 [`std::thread::available_parallelism`]。
pub const CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_INJECTION_THREADS";

/// 抽崩溃点的随机源与生成历史那一路岔开：同一个种子，历史怎么生成不受抽崩溃点影响，反过来也一样。
const CRASH_POINT_SEED_SALT: u64 = 0x63_72_61_73_68_70_74_00;

/// 每个工作线程摊到的片数：各段历史的长短差得远（一步就被拒的与连发四十次的），片切得比线程多，先跑完的线程接着领下一片。
const SLICES_PER_WORKER_THREAD: usize = 4;

/// 工作线程数是从哪来的：与实际起的线程数一起打进进度行，「机器多于 1 核却只用了 1 个线程」看得出来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashInjectionWorkerThreads {
    /// 环境变量 [`CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE`] 显式给的。
    FromTheEnvironmentVariable(usize),
    /// 没设环境变量，取 `available_parallelism`。
    FromAvailableParallelism(usize),
    /// 没设环境变量，`available_parallelism` 也报不出来：只用 1 个线程，照实报出来。
    AvailableParallelismUnknown,
    /// 调用方在代码里直接给的（用例拿不同线程数对拍）。
    GivenByCaller(usize),
}

impl CrashInjectionWorkerThreads {
    /// 线程数取环境变量，没设就取 `available_parallelism`。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数（含 0、空串、非 UTF-8）：配错了就停，不悄悄退回单线程。
    #[must_use]
    pub fn from_the_environment() -> Self {
        Self::from_the_environment_value(
            std::env::var(CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE),
            std::thread::available_parallelism,
        )
    }

    /// [`Self::from_the_environment`] 的判定本身：环境变量读到什么、`available_parallelism` 报什么都由调用方给
    /// （用例不改进程的环境变量）。
    ///
    /// # Panics
    /// 环境变量设了却不是正整数。
    #[must_use]
    fn from_the_environment_value(
        variable: Result<String, std::env::VarError>,
        available_parallelism: impl Fn() -> std::io::Result<std::num::NonZeroUsize>,
    ) -> Self {
        match variable {
            Ok(text) => {
                let count: usize = text.trim().parse().unwrap_or_else(|error| {
                    panic!(
                        "{CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE}={text} 不是十进制正整数：{error}"
                    )
                });
                assert!(
                    count > 0,
                    "{CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE}={text}：至少要 1 个线程"
                );
                Self::FromTheEnvironmentVariable(count)
            }
            Err(std::env::VarError::NotPresent) => match available_parallelism() {
                Ok(count) => Self::FromAvailableParallelism(count.get()),
                Err(_) => Self::AvailableParallelismUnknown,
            },
            Err(std::env::VarError::NotUnicode(raw)) => {
                panic!("{CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE} 不是 UTF-8：{raw:?}")
            }
        }
    }

    #[must_use]
    pub fn count(self) -> usize {
        match self {
            Self::FromTheEnvironmentVariable(count)
            | Self::FromAvailableParallelism(count)
            | Self::GivenByCaller(count) => count,
            Self::AvailableParallelismUnknown => 1,
        }
    }

    #[must_use]
    pub fn source_name(self) -> &'static str {
        match self {
            Self::FromTheEnvironmentVariable(_) => "environment_variable",
            Self::FromAvailableParallelism(_) => "available_parallelism",
            Self::AvailableParallelismUnknown => "available_parallelism_unknown",
            Self::GivenByCaller(_) => "given_by_caller",
        }
    }
}

/// 一个崩溃点：截在录制流的第几步（那一步是一次写）、那次写落在盘上的哪一区（种类）、它是历史哪一步里发出的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrashPoint {
    /// 录制流里的下标：`[0, stream_index]` 这一段持久，之后的都没落盘。
    pub stream_index: usize,
    pub truncated_write_kind: StepKind,
    pub step: StepPosition,
    /// 崩溃点落在哪一类操作里；落在起点里的是 None（起点整个持久，抽不到）。
    pub operation_kind: Option<HistoryOperationKind>,
    /// 这一步里、崩溃点之后还有根槽写：这次截断把一次发布截在它的根落盘之前。
    pub truncates_a_publish_before_its_root: bool,
}

/// 一段历史上一类崩溃点失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointFinding {
    pub signature: FailureSignature,
    pub seed: HistorySeed,
    pub crash_point: CrashPoint,
    pub observation: FailureObservation,
    /// 恢复读回了什么（给人看）。
    pub read_back: String,
}

impl CrashPointFinding {
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "崩溃点上的新发现 {:?}：种子 {}，截在录制流第 {} 步（{}，{:?}／{:?}）",
            self.signature,
            self.seed.0,
            self.crash_point.stream_index,
            self.crash_point.truncated_write_kind.name(),
            self.crash_point.step,
            self.crash_point.operation_kind
        );
        let _ = writeln!(text, "  恢复读回：{}", self.read_back);
        for (invariant, detail) in &self.observation.violations {
            let _ = writeln!(text, "  {invariant}：{detail}");
        }
        if let Some(panic) = &self.observation.panic {
            let _ = writeln!(text, "  panic 在 {}：{}", panic.location, panic.message);
        }
        if let Some(disagreement) = &self.observation.model_disagreement {
            let _ = writeln!(
                text,
                "  模型对不上（{}）：模型答 {}；实现 {}",
                disagreement.aspect.name(),
                disagreement.model_answer,
                disagreement.implementation_answer
            );
        }
        let _ = writeln!(
            text,
            "  复现：SINGLEFS_CRASH_INJECTION_FIRST_SEED={} SINGLEFS_CRASH_INJECTION_SEEDS=1 跑大档那条 #[ignore] 用例",
            self.seed.0
        );
        text
    }
}

/// 崩溃点上以「已知红」收尾的一次：种子、清单第几条、哪个崩溃点。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnownRedAtACrashPoint {
    pub seed: HistorySeed,
    pub form: usize,
    pub crash_point: CrashPoint,
}

/// 跑过的崩溃点的计数：绝对数，证明各条路径真的跑到了（`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CrashInjectionTally {
    pub histories: u64,
    /// 这段历史自己怎么收尾（崩溃点之外）。
    pub histories_completed: u64,
    pub histories_ended_known_red: BTreeMap<usize, u64>,
    pub histories_ended_new_finding: u64,
    /// 起点之后一次写都没有、一个崩溃点都抽不到的历史段数。
    pub histories_without_any_crash_point: u64,
    pub crash_points: u64,
    /// 按截断的那次写的种类分（四种落点都抽到过才算罩全）。
    pub crash_points_by_write_kind: BTreeMap<&'static str, u64>,
    /// 按崩溃点落在哪一类操作里分（固定脚本只有一条路，这里证明挂载、回退、抬 F、冷启动的写上都截过）。
    pub crash_points_by_operation_kind: BTreeMap<HistoryOperationKind, u64>,
    /// 截断之后盘上已持久的最新根槽，比这段历史最后提交的那一版旧：真的把状态截回了过去。
    pub crash_points_that_land_before_the_last_committed_version: u64,
    /// 截断落在一次发布的中间：这次写之后、这一步里还有根槽写没落盘。
    pub crash_points_inside_an_unfinished_publish: u64,
    pub recoveries_reading_a_file: u64,
    pub recoveries_without_a_file: u64,
    pub recoveries_failed: u64,
    /// 恢复真的施加过 journal 记录前缀的崩溃点数。
    pub recoveries_that_applied_journal_records: u64,
    pub checker_runs: u64,
    pub invariant_holds: BTreeMap<&'static str, u64>,
    pub invariant_not_applicable: BTreeMap<&'static str, u64>,
    /// 问过模型「恢复到的这一版允不允许」的次数（每个崩溃点一次）。
    pub model_judgements: u64,
    /// 模型认得择到的那条根、并真的逐字节比过内容的次数。
    pub model_contents_compared: u64,
    /// 模型认得择到的那条根、那一版树表 0 条、实现也报没有文件的次数。
    pub model_versions_without_a_file_matched: u64,
    /// 攒到的「模型提交过的每一版」目录里，最大的一段有几版。
    pub most_committed_versions_in_one_history: usize,
    pub crash_states_ending_known_red: BTreeMap<usize, u64>,
    pub crash_states_ending_new_finding: u64,
}

impl CrashInjectionTally {
    /// 把紧跟在后面的那一段的计数并进来：逐项相加，按字段拆开写全——新加一个字段而这里没并，编译不过。
    pub fn absorb(&mut self, following: &CrashInjectionTally) {
        let CrashInjectionTally {
            histories,
            histories_completed,
            histories_ended_known_red,
            histories_ended_new_finding,
            histories_without_any_crash_point,
            crash_points,
            crash_points_by_write_kind,
            crash_points_by_operation_kind,
            crash_points_that_land_before_the_last_committed_version,
            crash_points_inside_an_unfinished_publish,
            recoveries_reading_a_file,
            recoveries_without_a_file,
            recoveries_failed,
            recoveries_that_applied_journal_records,
            checker_runs,
            invariant_holds,
            invariant_not_applicable,
            model_judgements,
            model_contents_compared,
            model_versions_without_a_file_matched,
            most_committed_versions_in_one_history,
            crash_states_ending_known_red,
            crash_states_ending_new_finding,
        } = following;
        self.histories += histories;
        self.histories_completed += histories_completed;
        for (form, count) in histories_ended_known_red {
            *self.histories_ended_known_red.entry(*form).or_insert(0) += count;
        }
        self.histories_ended_new_finding += histories_ended_new_finding;
        self.histories_without_any_crash_point += histories_without_any_crash_point;
        self.crash_points += crash_points;
        for (kind, count) in crash_points_by_write_kind {
            *self.crash_points_by_write_kind.entry(kind).or_insert(0) += count;
        }
        for (kind, count) in crash_points_by_operation_kind {
            *self
                .crash_points_by_operation_kind
                .entry(*kind)
                .or_insert(0) += count;
        }
        self.crash_points_that_land_before_the_last_committed_version +=
            crash_points_that_land_before_the_last_committed_version;
        self.crash_points_inside_an_unfinished_publish += crash_points_inside_an_unfinished_publish;
        self.recoveries_reading_a_file += recoveries_reading_a_file;
        self.recoveries_without_a_file += recoveries_without_a_file;
        self.recoveries_failed += recoveries_failed;
        self.recoveries_that_applied_journal_records += recoveries_that_applied_journal_records;
        self.checker_runs += checker_runs;
        for (invariant, count) in invariant_holds {
            *self.invariant_holds.entry(invariant).or_insert(0) += count;
        }
        for (invariant, count) in invariant_not_applicable {
            *self.invariant_not_applicable.entry(invariant).or_insert(0) += count;
        }
        self.model_judgements += model_judgements;
        self.model_contents_compared += model_contents_compared;
        self.model_versions_without_a_file_matched += model_versions_without_a_file_matched;
        self.most_committed_versions_in_one_history = self
            .most_committed_versions_in_one_history
            .max(*most_committed_versions_in_one_history);
        for (form, count) in crash_states_ending_known_red {
            *self.crash_states_ending_known_red.entry(*form).or_insert(0) += count;
        }
        self.crash_states_ending_new_finding += crash_states_ending_new_finding;
    }

    /// 给人看的一整块：绝对数——抽了几个崩溃点、恢复成了几次、checker 跑了几次、模型判了几次、新发现几条。
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "历史 {} 段：跑完 {}、以已知红收尾 {:?}、新发现 {}；一个崩溃点都抽不到的 {} 段；模型目录最多 {} 版",
            self.histories,
            self.histories_completed,
            self.histories_ended_known_red,
            self.histories_ended_new_finding,
            self.histories_without_any_crash_point,
            self.most_committed_versions_in_one_history
        );
        let _ = writeln!(
            text,
            "崩溃点 {} 个：截在发布中间（根还没落盘）的 {} 个、截回到最后一版之前的 {} 个",
            self.crash_points,
            self.crash_points_inside_an_unfinished_publish,
            self.crash_points_that_land_before_the_last_committed_version
        );
        for (kind, count) in &self.crash_points_by_write_kind {
            let _ = writeln!(text, "  截断的那次写是 {kind}：{count} 个");
        }
        for kind in HistoryOperationKind::ALL {
            let _ = writeln!(
                text,
                "  落在 {kind:?} 里：{} 个",
                self.crash_points_by_operation_kind
                    .get(&kind)
                    .copied()
                    .unwrap_or(0)
            );
        }
        let _ = writeln!(
            text,
            "恢复：读回文件 {} 次、没有文件 {} 次、失败 {} 次；施加过 journal 记录前缀 {} 次",
            self.recoveries_reading_a_file,
            self.recoveries_without_a_file,
            self.recoveries_failed,
            self.recoveries_that_applied_journal_records
        );
        let _ = writeln!(
            text,
            "崩溃点上 checker 跑了 {} 次；问模型 {} 次：比过内容 {} 次、树表 0 条对上 {} 次",
            self.checker_runs,
            self.model_judgements,
            self.model_contents_compared,
            self.model_versions_without_a_file_matched
        );
        let _ = writeln!(
            text,
            "崩溃状态：以已知红收尾 {:?}、新发现 {}",
            self.crash_states_ending_known_red, self.crash_states_ending_new_finding
        );
        for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
            let _ = writeln!(
                text,
                "  {invariant}：判绿 {} 次、不适用 {} 次",
                self.invariant_holds.get(invariant).copied().unwrap_or(0),
                self.invariant_not_applicable
                    .get(invariant)
                    .copied()
                    .unwrap_or(0)
            );
        }
        text
    }
}

/// 一段历史注入崩溃之后交回的东西。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryCrashInjection {
    pub seed: HistorySeed,
    /// 抽到的崩溃点，按录制流里的次序。
    pub crash_points: Vec<CrashPoint>,
    pub tally: CrashInjectionTally,
    pub known_red_hits: Vec<KnownRedAtACrashPoint>,
    /// 按崩溃点从小到大排；同一段历史里同一个签名只留第一个。
    pub new_findings: Vec<CrashPointFinding>,
}

/// 一段历史跑到哪一步、录制流到那时有多长：崩溃点只抽在这段区间里。
struct StreamMarks {
    /// 起点跑完时流里有几步（崩溃点从它起）。
    after_the_starting_point: usize,
    /// 最后一个跑完的操作之后流里有几步（崩溃点到它为止）。
    after_the_last_finished_step: usize,
    /// 每一步跑完时流里有几步，按次序（崩溃点落在哪一步靠它定）。
    after_each_step: Vec<(StepPosition, usize)>,
}

/// 跑一段历史，按种子在它的录制流上抽 `crash_points_per_history` 个崩溃点，逐个重建镜像、跑恢复与池级 checker、问模型。
///
/// # Panics
/// 录制流没开内容保留（`MemoryPool::apply` 要字节）——这里自己建流，走不到。
#[must_use]
pub fn inject_crashes_into_history(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    crash_points_per_history: usize,
) -> HistoryCrashInjection {
    let stream = SharedStream::retaining_contents();
    let mut committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    let mut marks = StreamMarks {
        after_the_starting_point: 0,
        after_the_last_finished_step: 0,
        after_each_step: Vec::new(),
    };
    let run = execute_history_with(history, execution, &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed_versions.insert(key, file);
        }
        let operations_so_far = stream.operation_count();
        match observation.position {
            StepPosition::StartingPoint => marks.after_the_starting_point = operations_so_far,
            StepPosition::Operation(_) => {}
        }
        marks.after_the_last_finished_step = operations_so_far;
        marks
            .after_each_step
            .push((observation.position, operations_so_far));
    });

    let mut tally = CrashInjectionTally {
        histories: 1,
        most_committed_versions_in_one_history: committed_versions.len(),
        ..CrashInjectionTally::default()
    };
    match &run.ending {
        HistoryEnding::Completed => tally.histories_completed = 1,
        HistoryEnding::KnownRed { form, .. } => {
            tally.histories_ended_known_red.insert(*form, 1);
        }
        HistoryEnding::NewFinding { .. } => tally.histories_ended_new_finding = 1,
    }

    let operations = stream.retained_operations();
    let parameters = execution.device_width.parameters();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
    };
    let crash_points = draw_crash_points(
        history,
        &operations,
        &marks,
        &geometry,
        crash_points_per_history,
    );
    if crash_points.is_empty() {
        tally.histories_without_any_crash_point = 1;
    }
    let last_committed_version = committed_versions.keys().next_back().copied();

    let mut image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    let mut applied_operations = 0usize;
    let mut newest_persisted_root: Option<ModelRootKey> = None;
    let mut known_red_hits = Vec::new();
    let mut new_findings: Vec<CrashPointFinding> = Vec::new();
    for crash_point in crash_points.iter().copied() {
        // 崩溃点从小到大，镜像只往前叠：整条流重放一遍就够，不为每个崩溃点从头重建。
        let newly_persisted = &operations[applied_operations..=crash_point.stream_index];
        image.apply(newly_persisted);
        for retained in newly_persisted {
            if let Some(root) = root_written_by(retained, &geometry) {
                newest_persisted_root = newest_persisted_root.max(Some(root));
            }
        }
        applied_operations = crash_point.stream_index + 1;
        tally.crash_points += 1;
        *tally
            .crash_points_by_write_kind
            .entry(crash_point.truncated_write_kind.name())
            .or_insert(0) += 1;
        if let Some(kind) = crash_point.operation_kind {
            *tally
                .crash_points_by_operation_kind
                .entry(kind)
                .or_insert(0) += 1;
        }
        if newest_persisted_root < last_committed_version {
            tally.crash_points_that_land_before_the_last_committed_version += 1;
        }
        if crash_point.truncates_a_publish_before_its_root {
            tally.crash_points_inside_an_unfinished_publish += 1;
        }
        let report = recover(&image, JournalPolicy::Consult);
        match &report.outcome {
            RecoveryOutcome::FileRead { .. } => tally.recoveries_reading_a_file += 1,
            RecoveryOutcome::NoFile { .. } => tally.recoveries_without_a_file += 1,
            RecoveryOutcome::Failed { .. } => tally.recoveries_failed += 1,
        }
        if report.journal.prefix_applied > 0 {
            tally.recoveries_that_applied_journal_records += 1;
        }
        let read_back = observed_read_back_after_a_crash(&report);
        tally.model_judgements += 1;
        let disagreement =
            crash_recovery_disagreement(&committed_versions, newest_persisted_root, &read_back);
        if disagreement.is_none() {
            match &read_back {
                ObservedReadBack::FileRead { .. } => tally.model_contents_compared += 1,
                ObservedReadBack::NoFile { .. } => {
                    tally.model_versions_without_a_file_matched += 1;
                }
                ObservedReadBack::Failed { .. } => {}
            }
        }
        let violations = checker_violations_on(&image, &mut tally);
        if violations.is_empty() && disagreement.is_none() {
            continue;
        }
        let observation =
            crash_state_observation(&image, crash_point, violations, disagreement.clone());
        match classify_failure(observation) {
            HistoryEnding::Completed => {}
            HistoryEnding::KnownRed { form, .. } => {
                *tally.crash_states_ending_known_red.entry(form).or_insert(0) += 1;
                known_red_hits.push(KnownRedAtACrashPoint {
                    seed: history.seed,
                    form,
                    crash_point,
                });
            }
            HistoryEnding::NewFinding {
                signature,
                observation,
            } => {
                tally.crash_states_ending_new_finding += 1;
                if !new_findings
                    .iter()
                    .any(|finding| finding.signature == signature)
                {
                    new_findings.push(CrashPointFinding {
                        signature,
                        seed: history.seed,
                        crash_point,
                        observation,
                        read_back: format!("{read_back:?}"),
                    });
                }
            }
        }
    }
    HistoryCrashInjection {
        seed: history.seed,
        crash_points,
        tally,
        known_red_hits,
        new_findings,
    }
}

/// 一次录制的写写下了哪条根（落在根环里的就是根槽写）；不是根槽写的是 None。
fn root_written_by(retained: &RetainedOperation, geometry: &FixedGeometry) -> Option<ModelRootKey> {
    if geometry.classify(&retained.operation) != StepKind::RootRecordFua {
        return None;
    }
    let bytes = retained
        .contents
        .as_ref()
        .expect("崩溃注入的录制流开了内容保留");
    let (instance, checkpoint_txg) = root_identity_written_by(bytes);
    Some(model_root_key(instance, checkpoint_txg))
}

/// 录制流里第 `stream_index` 步落在哪一步操作里、那一步跑完时流里有几步：第一个「跑完时流里的步数」大于它的那一条登记。
/// 登记里找不到（崩溃点在最后一个跑完的操作之后，抽不到）时是 None。
fn step_containing(stream_index: usize, marks: &StreamMarks) -> Option<(StepPosition, usize)> {
    marks
        .after_each_step
        .iter()
        .find(|(_, operations_so_far)| *operations_so_far > stream_index)
        .copied()
}

/// 按种子抽崩溃点：候选是起点之后、最后一个跑完的操作为止那段流里的写（屏障不是写，截在屏障上与截在它前一个写之后同一个状态）。
/// 候选不够就全要。抽法是部分 Fisher–Yates 洗牌（同一个种子逐项复现），抽完按流里的次序排好——镜像只往前叠。
fn draw_crash_points(
    history: &GeneratedHistory,
    operations: &[RetainedOperation],
    marks: &StreamMarks,
    geometry: &FixedGeometry,
    crash_points_per_history: usize,
) -> Vec<CrashPoint> {
    let mut candidates: Vec<usize> = (marks.after_the_starting_point
        ..marks.after_the_last_finished_step.min(operations.len()))
        .filter(|index| operations[*index].operation.kind != RecordedOperationKind::Barrier)
        .collect();
    let drawn = crash_points_per_history.min(candidates.len());
    let mut source = SeededRandomSource::from_seed(history.seed.0 ^ CRASH_POINT_SEED_SALT);
    for position in 0..drawn {
        let remaining = u64::try_from(candidates.len() - position).expect("候选数装得进 u64");
        let picked = position + usize::try_from(source.below(remaining)).expect("下标装得进 usize");
        candidates.swap(position, picked);
    }
    let mut chosen: Vec<usize> = candidates[..drawn].to_vec();
    chosen.sort_unstable();
    chosen
        .into_iter()
        .map(|stream_index| {
            let (step, step_ends_at) = step_containing(stream_index, marks)
                .expect("崩溃点抽在最后一个跑完的操作之前，登记里一定找得到它那一步");
            let operation_kind = match step {
                StepPosition::StartingPoint => None,
                StepPosition::Operation(step_index) => history
                    .operations
                    .get(step_index)
                    .map(HistoryOperation::kind),
            };
            CrashPoint {
                stream_index,
                truncated_write_kind: geometry.classify(&operations[stream_index].operation),
                step,
                operation_kind,
                truncates_a_publish_before_its_root: operations
                    [stream_index + 1..step_ends_at.min(operations.len())]
                    .iter()
                    .any(|retained| root_written_by(retained, geometry).is_some()),
            }
        })
        .collect()
}

/// 对一份崩溃镜像跑池级 checker，记每条不变量判绿、不适用各几次，交回判红的那几条（与 `history.rs` 的 `violations_on` 同一条口径，
/// 计数进的是崩溃注入自己的那一份）。
fn checker_violations_on(
    image: &MemoryPool,
    tally: &mut CrashInjectionTally,
) -> Vec<(&'static str, String)> {
    tally.checker_runs += 1;
    let mut violations = Vec::new();
    for (invariant, verdict) in check_pool_image(image) {
        match verdict {
            InvariantVerdict::Holds => *tally.invariant_holds.entry(invariant).or_insert(0) += 1,
            InvariantVerdict::NotApplicable(_) => {
                *tally.invariant_not_applicable.entry(invariant).or_insert(0) += 1;
            }
            InvariantVerdict::Violated(detail) => violations.push((invariant, detail)),
        }
    }
    violations
}

/// 一个崩溃状态的失败观察：拿去对「已知红」清单（`KNOWN_RED_FORMS`，与历史那一路同一张清单）。
/// 抬 F 那一格（第 1 条）要的「新 F 落在回退留下的空档里」在崩溃状态上不算：那一条判的是抬 F 这一步之后的活盘面，
/// 这里留 None，崩溃状态上的 I-3.1 只由第 0 条（根环转过一圈）接得走。
fn crash_state_observation(
    image: &MemoryPool,
    crash_point: CrashPoint,
    violations: Vec<(&'static str, String)>,
    model_disagreement: Option<ModelDisagreement>,
) -> FailureObservation {
    let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(image);
    FailureObservation {
        position: crash_point.step,
        operation_kind: crash_point.operation_kind,
        violations,
        panic: None,
        newest_ring_root_txg,
        root_ring_slot_count,
        harness_judgement: None,
        model_disagreement,
        raised_floor_lands_only_on_abandoned_roots: None,
    }
}

/// 一批种子跑下来的崩溃注入报告。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashInjectionReport {
    pub first_seed: u64,
    pub seed_count: u64,
    pub operations_per_history: usize,
    pub crash_points_per_history: usize,
    pub weights: GenerationWeights,
    pub execution: HistoryExecution,
    pub tally: CrashInjectionTally,
    pub known_red_hits: Vec<KnownRedAtACrashPoint>,
    /// 按种子从小到大排；同一个签名只留第一个种子。
    pub new_findings: Vec<CrashPointFinding>,
}

impl CrashInjectionReport {
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(
            text,
            "种子 [{}, {})，每段 {} 步、每段抽 {} 个崩溃点，比重：{}；{}",
            self.first_seed,
            self.first_seed + self.seed_count,
            self.operations_per_history,
            self.crash_points_per_history,
            self.weights.name,
            self.execution.name()
        );
        text.push_str(&self.tally.render());
        for (form_index, form) in KNOWN_RED_FORMS.iter().enumerate() {
            let hits: Vec<(u64, usize)> = self
                .known_red_hits
                .iter()
                .filter(|hit| hit.form == form_index)
                .map(|hit| (hit.seed.0, hit.crash_point.stream_index))
                .collect();
            let _ = writeln!(
                text,
                "崩溃状态上的已知红第 {form_index} 条（{}）：{} 个崩溃点；前几个 (种子, 流下标) {:?}",
                form.closeout_table_row,
                hits.len(),
                hits.iter().take(8).collect::<Vec<_>>()
            );
        }
        for finding in &self.new_findings {
            text.push_str(&finding.render());
        }
        text
    }
}

/// 跑种子 [`first_seed`, `first_seed` + `seed_count`)，每段 `operations_per_history` 步、每段按种子抽 `crash_points_per_history`
/// 个崩溃点，分给 `worker_threads.count()` 个工作线程：种子区间切成首尾相接的片，线程按片号从小到大领片，
/// 调用线程收到一片就打一行 `CRASH_INJECTION_PROGRESS`（片号、种子区间、已跑完的片数与段数），再按片号从小到大并计数。
/// 计数按片的次序相加、新发现按种子从小到大留第一个，所以 [`CrashInjectionReport::render`] 与线程数、调度次序无关
/// （进度行是按到达次序打的，只报跑到哪了，不带判定）。
///
/// # Panics
/// 某个工作线程 panic（历史里的 panic 在执行器里接住，走到这里的是崩溃点上恢复或 checker 的断言）；有一片领了却没交回。
#[must_use]
pub fn run_crash_injection_campaign(
    first_seed: u64,
    seed_count: u64,
    operations_per_history: usize,
    crash_points_per_history: usize,
    weights: &GenerationWeights,
    execution: HistoryExecution,
    worker_threads: CrashInjectionWorkerThreads,
) -> CrashInjectionReport {
    let slices = seed_slices(seed_count, worker_threads.count());
    let spawned_worker_threads = worker_threads.count().min(slices.len().max(1));
    let started = Instant::now();
    println!(
        "CRASH_INJECTION_START seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} configured_worker_threads={} worker_threads_source={}",
        first_seed + seed_count,
        slices.len(),
        worker_threads.count(),
        worker_threads.source_name()
    );
    let next_slice_index = AtomicUsize::new(0);
    let (finished_slices, merged_slice_count) = std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel::<(usize, Vec<HistoryCrashInjection>)>();
        for _ in 0..spawned_worker_threads {
            let sender = sender.clone();
            let slices = &slices;
            let next_slice_index = &next_slice_index;
            scope.spawn(move || loop {
                let slice_index = next_slice_index.fetch_add(1, Ordering::Relaxed);
                let Some(slice) = slices.get(slice_index) else {
                    break;
                };
                let injections: Vec<HistoryCrashInjection> = slice
                    .clone()
                    .map(|offset| {
                        let history = generate_history_with_weights(
                            HistorySeed(first_seed + offset),
                            operations_per_history,
                            weights,
                        );
                        inject_crashes_into_history(&history, execution, crash_points_per_history)
                    })
                    .collect();
                if sender.send((slice_index, injections)).is_err() {
                    break;
                }
            });
        }
        drop(sender);
        let mut waiting_for_earlier_slices: BTreeMap<usize, Vec<HistoryCrashInjection>> =
            BTreeMap::new();
        let mut in_order: Vec<HistoryCrashInjection> = Vec::new();
        let mut next_slice_to_merge = 0usize;
        let mut finished_histories = 0u64;
        for (finished_slice_count, (slice_index, injections)) in receiver.iter().enumerate() {
            let slice = &slices[slice_index];
            finished_histories += slice.end - slice.start;
            println!(
                "CRASH_INJECTION_PROGRESS slice={}/{} seeds=[{},{}) finished_slices={}/{} finished_histories={finished_histories}/{seed_count} elapsed_seconds={:.1}",
                slice_index + 1,
                slices.len(),
                first_seed + slice.start,
                first_seed + slice.end,
                finished_slice_count + 1,
                slices.len(),
                started.elapsed().as_secs_f64()
            );
            waiting_for_earlier_slices.insert(slice_index, injections);
            while let Some(ready) = waiting_for_earlier_slices.remove(&next_slice_to_merge) {
                in_order.extend(ready);
                next_slice_to_merge += 1;
            }
        }
        (in_order, next_slice_to_merge)
    });
    assert_eq!(
        merged_slice_count,
        slices.len(),
        "每一片都按次序并进来了：少了说明有工作线程领了片却没交回"
    );
    println!(
        "CRASH_INJECTION_FINISHED seeds=[{first_seed},{}) slices={} worker_threads={spawned_worker_threads} elapsed_seconds={:.1}",
        first_seed + seed_count,
        slices.len(),
        started.elapsed().as_secs_f64()
    );
    let mut tally = CrashInjectionTally::default();
    let mut known_red_hits = Vec::new();
    let mut new_findings: Vec<CrashPointFinding> = Vec::new();
    let mut signatures_seen: BTreeSet<FailureSignature> = BTreeSet::new();
    for injection in &finished_slices {
        tally.absorb(&injection.tally);
        known_red_hits.extend(injection.known_red_hits.iter().copied());
        for finding in &injection.new_findings {
            if signatures_seen.insert(finding.signature.clone()) {
                new_findings.push(finding.clone());
            }
        }
    }
    CrashInjectionReport {
        first_seed,
        seed_count,
        operations_per_history,
        crash_points_per_history,
        weights: *weights,
        execution,
        tally,
        known_red_hits,
        new_findings,
    }
}

/// 把 [0, `seed_count`) 切成首尾相接的种子区间：片数取 min(种子数, 4 × 线程数)，各片长度相差至多 1。
fn seed_slices(seed_count: u64, worker_threads: usize) -> Vec<std::ops::Range<u64>> {
    if seed_count == 0 {
        return Vec::new();
    }
    let wanted = u64::try_from(worker_threads.max(1) * SLICES_PER_WORKER_THREAD).expect("片数");
    let slice_count = wanted.min(seed_count);
    (0..slice_count)
        .map(|slice_index| {
            let start = seed_count * slice_index / slice_count;
            let end = seed_count * (slice_index + 1) / slice_count;
            start..end
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{generate_history, HistoryStartingPoint};

    /// 线程数从环境变量取，没设取 `available_parallelism`；设了不是正整数就停，不悄悄退回单线程。
    #[test]
    fn worker_thread_count_comes_from_the_environment_variable_or_available_parallelism() {
        let four = || Ok(std::num::NonZeroUsize::new(4).expect("4"));
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(Ok("3".to_string()), four),
            CrashInjectionWorkerThreads::FromTheEnvironmentVariable(3)
        );
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                Err(std::env::VarError::NotPresent),
                four
            ),
            CrashInjectionWorkerThreads::FromAvailableParallelism(4)
        );
        assert_eq!(
            CrashInjectionWorkerThreads::from_the_environment_value(
                Err(std::env::VarError::NotPresent),
                || Err(std::io::Error::other("说不出几个核"))
            ),
            CrashInjectionWorkerThreads::AvailableParallelismUnknown
        );
        assert_eq!(
            CrashInjectionWorkerThreads::FromAvailableParallelism(4).count(),
            4
        );
        assert_eq!(
            CrashInjectionWorkerThreads::AvailableParallelismUnknown.count(),
            1
        );
    }

    /// 片是首尾相接、不重不漏的：并起来正好是 [0, 种子数)。
    #[test]
    fn seed_slices_tile_the_whole_range() {
        for (seed_count, worker_threads) in [(0u64, 4usize), (1, 4), (7, 3), (96, 16), (5, 32)] {
            let slices = seed_slices(seed_count, worker_threads);
            let covered: Vec<u64> = slices.iter().flat_map(|slice| slice.clone()).collect();
            assert_eq!(
                covered,
                (0..seed_count).collect::<Vec<u64>>(),
                "种子 {seed_count} 个、{worker_threads} 个线程"
            );
            assert!(
                slices.iter().all(|slice| slice.start < slice.end),
                "没有空片：{slices:?}"
            );
        }
    }

    /// 同一个种子抽到的崩溃点逐项相同；崩溃点都落在起点之后、都是写、按流里的次序排好。
    #[test]
    fn crash_points_are_reproducible_and_sorted_writes_after_the_starting_point() {
        let history = generate_history(HistorySeed(11), 12);
        let first = inject_crashes_into_history(
            &history,
            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            4,
        );
        let second = inject_crashes_into_history(
            &history,
            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            4,
        );
        assert_eq!(first, second, "同一个种子、同一个跑法，逐项相同");
        assert!(
            first.tally.crash_points >= 1,
            "种子 11 的 12 步里抽得到崩溃点：{:?}",
            first.tally
        );
        assert_eq!(
            first.tally.model_judgements, first.tally.crash_points,
            "每个崩溃点都问过模型"
        );
        assert_eq!(
            first.tally.checker_runs, first.tally.crash_points,
            "每个崩溃点都跑过 checker"
        );
    }

    /// 崩溃点抽在写上：种子越多，四种落点（单元、journal 记录、根槽、超级块槽）都抽得到。
    #[test]
    fn crash_points_cover_every_kind_of_write() {
        let mut tally = CrashInjectionTally::default();
        for seed in 0..8u64 {
            let history = generate_history(HistorySeed(seed), 16);
            let injection = inject_crashes_into_history(
                &history,
                HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
                6,
            );
            tally.absorb(&injection.tally);
        }
        for kind in [
            StepKind::UnitWrite,
            StepKind::JournalRecord,
            StepKind::RootRecordFua,
            StepKind::SuperblockSlot,
        ] {
            assert!(
                tally
                    .crash_points_by_write_kind
                    .get(kind.name())
                    .copied()
                    .unwrap_or(0)
                    >= 1,
                "八个种子里一次都没截在 {} 上：{:?}",
                kind.name(),
                tally.crash_points_by_write_kind
            );
        }
        assert!(
            tally.crash_points_that_land_before_the_last_committed_version >= 1,
            "一个崩溃点都没截回到最后一版之前"
        );
    }

    /// 起点那一段整个持久：从 mkfs 起的历史里，崩溃点的流下标都在起点之后。
    #[test]
    fn no_crash_point_falls_inside_the_starting_point() {
        let history = GeneratedHistory {
            seed: HistorySeed(3),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: generate_history(HistorySeed(3), 10).operations,
        };
        let stream = SharedStream::retaining_contents();
        let mut operations_after_the_starting_point = 0;
        let _ = execute_history_with(
            &history,
            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            &stream,
            &mut |observation| {
                if observation.position == StepPosition::StartingPoint {
                    operations_after_the_starting_point = stream.operation_count();
                }
            },
        );
        assert!(
            operations_after_the_starting_point > 0,
            "起点写过盘（mkfs 加第一个文件）"
        );
        let injection = inject_crashes_into_history(
            &history,
            HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            6,
        );
        assert!(injection.tally.crash_points >= 1);
        assert!(
            injection
                .crash_points
                .iter()
                .all(|crash_point| crash_point.stream_index >= operations_after_the_starting_point),
            "崩溃点都在起点之后（起点整个持久）：{:?}，起点占了 {operations_after_the_starting_point} 步",
            injection
                .crash_points
                .iter()
                .map(|crash_point| crash_point.stream_index)
                .collect::<Vec<usize>>()
        );
        assert!(
            injection
                .crash_points
                .windows(2)
                .all(|pair| pair[0].stream_index < pair[1].stream_index),
            "崩溃点按流里的次序排好、互不相同"
        );
    }
}
```
