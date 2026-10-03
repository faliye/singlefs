# 代码轮第三轮被判的 diff（第二轮冻结副本 → 第三轮冻结副本，第 5 批 5-1 改过的 7 份文件与 `crates/mutations.tsv` 里 5-1 改写的 4 行、追加的 2 行；两份冻结副本之间别的会话改的文件不判）

```diff
diff --git a/crates/singlefs-replay-prefix/src/lib.rs b/crates/singlefs-replay-prefix/src/lib.rs
index d28a8af..a34bee7 100644
--- a/crates/singlefs-replay-prefix/src/lib.rs
+++ b/crates/singlefs-replay-prefix/src/lib.rs
@@ -6,8 +6,9 @@
 //! spec 照 D23（journal 的角色与格式）已定项 4、7、8、14、18、22 与 D13（验证路线）已定项 1 的 M2 那一行写（`replay_follows_the_prefix_rules`）：
 //! 施加哪一截由逐条判定（链首与 jsn 严格连续、反向链、读者规则、提交标记、末条之后同一次发布、点名单元）与在飞上限一起定，
 //! 报告的五个数与两种报错各有一句。
-//! 读盘才判得出的两件（反向链、点名单元）由调用方给两个回调，核走到那一步才调，读盘次数与次序同原文；
-//! spec 只说「核据以接受或断链的那一次回调交回了什么」，回调自己判得对不对不在 Verus 里。
+//! 读盘才判得出的两件（反向链、点名单元）由调用方给两个回调，核走到那一步才调。
+//! spec 只说「核据以接受或断链的那一次回调交回了什么」，不管回调调了几次、按什么次序调；回调自己判得对不对也不在 Verus 里。
+//! 挂载时每个候选读几次盘、按什么次序读，由 harness 用例 `crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs` 钉住。
 //! 平时的 stable 编译用 crates.io 的 `vstd` 与 `verus_builtin_macros` 把 `verus!` 擦成普通 Rust；
 //! 验的时候 Verus 用它自带的那一份，两边钉同一个发布（`Cargo.toml` 的 vstd 版本与 Verus 0.2026.09.20.aef82ed）。
 //! Verus 的必红世界登记在同目录的 `verus-mutations.tsv`。
@@ -18,25 +19,25 @@ use vstd::prelude::*;
 verus! {
 
 // ===== 核的数据 =====
-// 核不依赖 singlefs-core 与 singlefs-format（Verus 只编这一个文件），这几个类型与 singlefs-core 里的同名类型是同一个概念，
-// 胶水（recovery.rs 的 replay_journal）逐个换过来。
+// 核不依赖 singlefs-core 与 singlefs-format（Verus 只编这一个文件）。带 PrefixKernel 的四个类型与 singlefs-core 里不带它的那四个
+// 说的是同一个量，名字分开、全仓一搜只落到一处；胶水（recovery.rs 的 replay_journal）逐个换过来。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub struct InstanceGeneration(pub u32);
+pub struct PrefixKernelInstanceGeneration(pub u32);
 
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct JournalCounter(pub u64);
 
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub struct CheckpointTxg(pub u64);
+pub struct PrefixKernelCheckpointTxg(pub u64);
 
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct TransactionNumber(pub u64);
 
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub struct JournalRecordOrdinalWithinPublish(pub u32);
+pub struct PrefixKernelJournalRecordOrdinalWithinPublish(pub u32);
 
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub enum JournalRecordPlaceInPublish {
+pub enum PrefixKernelJournalRecordPlaceInPublish {
     LastRecordOfThePublish,
     MoreRecordsOfThePublishFollow,
 }
@@ -49,13 +50,13 @@ pub enum NamedUnitVerification {
 }
 
 pub struct CoreRecord<RootSource> {
-    pub instance: InstanceGeneration,
+    pub instance: PrefixKernelInstanceGeneration,
     pub counter: JournalCounter,
-    pub checkpoint_txg: CheckpointTxg,
+    pub checkpoint_txg: PrefixKernelCheckpointTxg,
     pub transaction: TransactionNumber,
     pub is_commit: bool,
-    pub ordinal_within_publish: JournalRecordOrdinalWithinPublish,
-    pub place_in_publish: JournalRecordPlaceInPublish,
+    pub ordinal_within_publish: PrefixKernelJournalRecordOrdinalWithinPublish,
+    pub place_in_publish: PrefixKernelJournalRecordPlaceInPublish,
     // 核不看它：施加了一次发布之后把那次发布末条的这一项原样交回，调用方照它拼新根。
     pub new_root_source: RootSource,
 }
@@ -94,7 +95,7 @@ pub open spec fn key_is_below<RootSource>(
         < upper.counter.0)
 }
 
-// 入参的次序：按 (实例代号, 计数器) 严格升序，与 BTreeMap<(InstanceGeneration, u64), _> 的键序相同。
+// 入参的次序：按 (实例代号, 计数器) 严格升序，与 BTreeMap<(PrefixKernelInstanceGeneration, u64), _> 的键序相同。
 pub open spec fn records_ascend_by_instance_and_counter<RootSource>(
     records: Seq<CoreRecord<RootSource>>,
 ) -> bool {
@@ -117,8 +118,8 @@ pub open spec fn every_counter_has_a_successor<RootSource>(
 // 已定项 14 定案句：只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录；注 1：实例代号与所选根不同即停。
 pub open spec fn above_the_chosen_root<RootSource>(
     record: CoreRecord<RootSource>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
 ) -> bool {
     record.instance == root_instance && record.checkpoint_txg.0 > root_checkpoint_txg.0
 }
@@ -126,8 +127,8 @@ pub open spec fn above_the_chosen_root<RootSource>(
 // 切片里 [0, end) 之间在所选根之上的记录（候选）的下标，按切片次序。
 pub open spec fn candidates_before<RootSource>(
     records: Seq<CoreRecord<RootSource>>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     end: int,
 ) -> Seq<int>
     decreases end,
@@ -147,8 +148,8 @@ pub open spec fn candidates_before<RootSource>(
 // 已定项 14 注 1（读法乙）：所选根那次发布（同实例、同 checkpoint_txg）读得出的记录里带末条标志的那几条的计数器。
 pub open spec fn last_record_flag_counters_of_the_chosen_roots_publish<RootSource>(
     records: Seq<CoreRecord<RootSource>>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     end: int,
 ) -> Seq<JournalCounter>
     decreases end,
@@ -175,8 +176,8 @@ pub open spec fn last_record_flag_counters_of_the_chosen_roots_publish<RootSourc
 // 链首的锚点：带末条标志的恰好一条时是它；一条都没有就是「那条读不出」。
 pub open spec fn anchor_of_the_chain<RootSource>(
     records: Seq<CoreRecord<RootSource>>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
 ) -> Option<JournalCounter> {
     let flagged = last_record_flag_counters_of_the_chosen_roots_publish(
         records,
@@ -196,8 +197,8 @@ pub open spec fn continues_the_jsn_chain<RootSource>(
     record: CoreRecord<RootSource>,
     previous: Option<CoreRecord<RootSource>>,
     anchor: Option<JournalCounter>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
 ) -> bool {
     match previous {
         Some(previous_record) => record.instance == previous_record.instance && record.counter.0
@@ -306,8 +307,8 @@ pub open spec fn passes_every_check_before_the_named_units<
     position: int,
     previous: Option<CoreRecord<RootSource>>,
     anchor: Option<JournalCounter>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     back_chain_broken: BackChain,
 ) -> bool {
     &&& continues_the_jsn_chain(
@@ -329,8 +330,8 @@ pub open spec fn fails_a_check_before_the_named_units<RootSource, BackChain: Fn(
     position: int,
     previous: Option<CoreRecord<RootSource>>,
     anchor: Option<JournalCounter>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     back_chain_broken: BackChain,
 ) -> bool {
     ||| !continues_the_jsn_chain(
@@ -368,8 +369,8 @@ pub open spec fn admitted_record_passed_every_check<
     admitted: Seq<int>,
     position_in_chain: int,
     anchor: Option<JournalCounter>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     named_unit_verification: NamedUnitVerification,
     back_chain_broken: BackChain,
     named_units_verified: NamedUnits,
@@ -401,8 +402,8 @@ pub open spec fn the_candidate_stops_the_chain<
     next: int,
     in_flight_limit: usize,
     anchor: Option<JournalCounter>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     named_unit_verification: NamedUnitVerification,
     back_chain_broken: BackChain,
     named_units_verified: NamedUnits,
@@ -485,8 +486,8 @@ pub open spec fn records_at<RootSource>(
 
 // D13 已定项 1 的 M2：施加的是若干整次发布、都在所选根之上；报告的条数与交回的根跟着施加的走。
 pub open spec fn applied_records_are_whole_publishes_above_the_chosen_root<RootSource>(
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     chosen_root_source: RootSource,
     records: Seq<CoreRecord<RootSource>>,
     outcome: CoreOutcome<RootSource>,
@@ -530,8 +531,8 @@ pub open spec fn outcome_follows_the_prefix_rules<
     BackChain: Fn(usize) -> bool,
     NamedUnits: Fn(usize) -> bool,
 >(
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     chosen_root_source: RootSource,
     records: Seq<CoreRecord<RootSource>>,
     in_flight_limit: usize,
@@ -611,8 +612,8 @@ pub open spec fn replay_follows_the_prefix_rules<
     BackChain: Fn(usize) -> bool,
     NamedUnits: Fn(usize) -> bool,
 >(
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     chosen_root_source: RootSource,
     records: Seq<CoreRecord<RootSource>>,
     in_flight_limit: usize,
@@ -676,8 +677,8 @@ pub open spec fn expected_next_after<RootSource>(
     records: Seq<CoreRecord<RootSource>>,
     admitted: Seq<int>,
     anchor: Option<JournalCounter>,
-    root_instance: InstanceGeneration,
-) -> Option<(InstanceGeneration, JournalCounter)> {
+    root_instance: PrefixKernelInstanceGeneration,
+) -> Option<(PrefixKernelInstanceGeneration, JournalCounter)> {
     if admitted.len() == 0 {
         match anchor {
             Some(anchor_counter) => Some(
@@ -693,8 +694,8 @@ pub open spec fn expected_next_after<RootSource>(
 
 proof fn lemma_candidates_lie_below_their_end_in_ascending_order<RootSource>(
     records: Seq<CoreRecord<RootSource>>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     end: int,
 )
     requires
@@ -727,8 +728,8 @@ proof fn lemma_candidates_lie_below_their_end_in_ascending_order<RootSource>(
 
 proof fn lemma_candidates_before_a_shorter_end_are_a_prefix<RootSource>(
     records: Seq<CoreRecord<RootSource>>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     shorter: int,
     longer: int,
 )
@@ -786,8 +787,8 @@ proof fn lemma_admitting_the_candidate_keeps_every_link<
     admitted_before: Seq<int>,
     next: int,
     anchor: Option<JournalCounter>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     named_unit_verification: NamedUnitVerification,
     back_chain_broken: BackChain,
     named_units_verified: NamedUnits,
@@ -1049,8 +1050,8 @@ fn record_ends_its_publish<RootSource>(record: &CoreRecord<RootSource>) -> (is_l
         is_last_record == is_last_record_of_its_publish(*record),
 {
     match record.place_in_publish {
-        JournalRecordPlaceInPublish::LastRecordOfThePublish => true,
-        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,
+        PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish => true,
+        PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,
     }
 }
 
@@ -1067,17 +1068,12 @@ fn readable_record_of_the_same_publish_follows<RootSource>(
         follows == a_later_record_of_the_same_publish_is_readable(records@, index as int),
 {
     let record = &records[index];
-    if record.counter.0 == u64::MAX {
-        return false;
-    }
-    let next_counter = record.counter.0 + 1;
     let mut later_index = index + 1;
     while later_index < records.len()
         invariant
             index < later_index <= records.len(),
             *record == records@[index as int],
             records_ascend_by_instance_and_counter(records@),
-            next_counter == records@[index as int].counter.0 + 1,
             forall|between: int|
                 index < between < later_index && #[trigger] is_a_later_record_of_the_same_instance(
                     records@,
@@ -1087,7 +1083,15 @@ fn readable_record_of_the_same_publish_follows<RootSource>(
         decreases records.len() - later_index,
     {
         let later = &records[later_index];
-        if later.instance.0 == record.instance.0 && later.counter.0 >= next_counter {
+        if later.instance.0 == record.instance.0 {
+            // 入参按 (实例代号, 计数器) 严格升序：切片里在这一条之后的同实例记录，计数器一定更大，不用再比计数器。
+            proof {
+                lemma_slice_order_within_an_instance_follows_the_counter(
+                    records@,
+                    index as int,
+                    later_index as int,
+                );
+            }
             if !(later.checkpoint_txg.0 <= record.checkpoint_txg.0) {
                 proof {
                     assert forall|witness: int|
@@ -1173,8 +1177,8 @@ fn readable_record_of_the_same_publish_follows<RootSource>(
 // 所选根覆盖的最后一条记录的 jsn 计数器（已定项 14 注 1，读法乙）：所选根那次发布读得出的几条里带末条标志的那一条；
 // 一条都不带 ⇒ Ok(None)；带标志的多于一条 ⇒ 认哪一条条款没写，报错并带出那几条的计数器。
 fn counter_of_the_last_record_the_root_covers<RootSource>(
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     records: &[CoreRecord<RootSource>],
 ) -> (anchor: Result<Option<JournalCounter>, CoreFailure>)
     requires
@@ -1245,8 +1249,8 @@ fn counter_of_the_last_record_the_root_covers<RootSource>(
 // 前缀规则不跨实例边界（已定项 14 第 1 条），只有所选根自己那个实例、txg 比根大的记录是候选。
 fn is_above_the_chosen_root<RootSource>(
     record: &CoreRecord<RootSource>,
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
 ) -> (is_above: bool)
     ensures
         is_above == above_the_chosen_root(*record, root_instance, root_checkpoint_txg),
@@ -1303,12 +1307,12 @@ fn maximum_transaction_of<RootSource>(
     maximum
 }
 
-// 一个候选过不过逐条判定。次序与读盘次序同原文：链首与 jsn 连续、反向链回调、读者规则与提交标记、末条之后同一次发布、点名单元回调；
+// 一个候选过不过逐条判定。判的次序同原文，读盘次序跟着它：链首与 jsn 连续、反向链回调、读者规则与提交标记、末条之后同一次发布、点名单元回调；
 // 前一判不过就不再往下判（也就不再调后面的回调）。
 enum CandidateVerdict {
     Admitted,
-    BrokenBeforeTheNamedUnits,
-    BrokenAtTheNamedUnits,
+    FailsACheckBeforeTheNamedUnits,
+    NamedUnitsReportedUnverified,
 }
 
 #[allow(clippy::too_many_arguments, reason = "逐条判定要的输入各自独立；幽灵量五个只给证明用，收成结构体只会多一层没人验的名字")]
@@ -1316,16 +1320,16 @@ fn judge_the_candidate<RootSource, BackChain: Fn(usize) -> bool, NamedUnits: Fn(
     records: &[CoreRecord<RootSource>],
     index: usize,
     records_of_the_open_publish: &[usize],
-    expected_next: Option<(InstanceGeneration, JournalCounter)>,
-    chain_start_txg_without_anchor: CheckpointTxg,
+    expected_next: Option<(PrefixKernelInstanceGeneration, JournalCounter)>,
+    chain_start_txg_without_anchor: PrefixKernelCheckpointTxg,
     named_unit_verification: NamedUnitVerification,
     back_chain_broken: &BackChain,
     named_units_verified: &NamedUnits,
     Ghost(admitted): Ghost<Seq<int>>,
     Ghost(applied): Ghost<Seq<int>>,
     Ghost(anchor): Ghost<Option<JournalCounter>>,
-    Ghost(root_instance): Ghost<InstanceGeneration>,
-    Ghost(root_checkpoint_txg): Ghost<CheckpointTxg>,
+    Ghost(root_instance): Ghost<PrefixKernelInstanceGeneration>,
+    Ghost(root_checkpoint_txg): Ghost<PrefixKernelCheckpointTxg>,
 ) -> (verdict: CandidateVerdict)
     requires
         index < records.len(),
@@ -1362,7 +1366,7 @@ fn judge_the_candidate<RootSource, BackChain: Fn(usize) -> bool, NamedUnits: Fn(
                     *named_units_verified,
                     index as int,
                 ),
-                CandidateVerdict::BrokenBeforeTheNamedUnits => fails_a_check_before_the_named_units(
+                CandidateVerdict::FailsACheckBeforeTheNamedUnits => fails_a_check_before_the_named_units(
                     records@,
                     index as int,
                     previous,
@@ -1371,7 +1375,7 @@ fn judge_the_candidate<RootSource, BackChain: Fn(usize) -> bool, NamedUnits: Fn(
                     root_checkpoint_txg,
                     *back_chain_broken,
                 ),
-                CandidateVerdict::BrokenAtTheNamedUnits => passes_every_check_before_the_named_units(
+                CandidateVerdict::NamedUnitsReportedUnverified => passes_every_check_before_the_named_units(
                     records@,
                     index as int,
                     previous,
@@ -1402,28 +1406,27 @@ fn judge_the_candidate<RootSource, BackChain: Fn(usize) -> bool, NamedUnits: Fn(
     match expected_next {
         Some(expected_key) => {
             if record.instance.0 != expected_key.0.0 || record.counter.0 != expected_key.1.0 {
-                return CandidateVerdict::BrokenBeforeTheNamedUnits;
+                return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
             }
         },
         // 锚点读不出：链首只能是 txg = 根的 txg + 1、本次发布内序号为 1 的那条（三方第一轮 K4-b）。
-        // 序号那一判与下面「攒着的为空且序号不是 1」那一判冗余：走到这一支时攒着的一定是空的（下面这句由 Verus 证），
-        // 两判之间只有反向链回调，它只会断、不改状态。
+        // 序号为 1 这里不判，交给下面「攒着的为空且序号不是 1」那一判：走到这一支时攒着的一定是空的（下面这句由 Verus 证），
+        // 那一判对链首照样判，两判之间只有反向链回调，它只会断、不改状态。
         None => {
             proof {
                 assert(records_of_the_open_publish@.len() == 0);
             }
-            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0
-                || record.ordinal_within_publish.0 != 1 {
-                return CandidateVerdict::BrokenBeforeTheNamedUnits;
+            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0 {
+                return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
             }
         },
     }
     // 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断；三态由回调交成布尔。
     if back_chain_broken(index) {
-        return CandidateVerdict::BrokenBeforeTheNamedUnits;
+        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
     }
     if records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 {
-        return CandidateVerdict::BrokenBeforeTheNamedUnits;
+        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
     }
     proof {
         if admitted.len() > 0 {
@@ -1447,33 +1450,33 @@ fn judge_the_candidate<RootSource, BackChain: Fn(usize) -> bool, NamedUnits: Fn(
             assert(!is_last_record_of_its_publish(records@[admitted.last()]));
         }
         if record.checkpoint_txg.0 != previous_record_of_the_open_publish.checkpoint_txg.0 {
-            return CandidateVerdict::BrokenBeforeTheNamedUnits;
+            return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
         }
         if u64::from(record.ordinal_within_publish.0) != u64::from(
             previous_record_of_the_open_publish.ordinal_within_publish.0,
         ) + 1 {
-            return CandidateVerdict::BrokenBeforeTheNamedUnits;
+            return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
         }
         if !previous_record_of_the_open_publish.is_commit
             && record.transaction.0 != previous_record_of_the_open_publish.transaction.0 {
-            return CandidateVerdict::BrokenBeforeTheNamedUnits;
+            return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
         }
     }
     if !record.is_commit && record_ends_its_publish(record) {
-        return CandidateVerdict::BrokenBeforeTheNamedUnits;
+        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
     }
     if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(
         records,
         index,
     ) {
-        return CandidateVerdict::BrokenBeforeTheNamedUnits;
+        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
     }
     let all_verified = match named_unit_verification {
         NamedUnitVerification::Verify => named_units_verified(index),
         NamedUnitVerification::Skip => true,
     };
     if !all_verified {
-        return CandidateVerdict::BrokenAtTheNamedUnits;
+        return CandidateVerdict::NamedUnitsReportedUnverified;
     }
     CandidateVerdict::Admitted
 }
@@ -1487,8 +1490,8 @@ pub fn replay_core<
     BackChain: Fn(usize) -> bool,
     NamedUnits: Fn(usize) -> bool,
 >(
-    root_instance: InstanceGeneration,
-    root_checkpoint_txg: CheckpointTxg,
+    root_instance: PrefixKernelInstanceGeneration,
+    root_checkpoint_txg: PrefixKernelCheckpointTxg,
     chosen_root_source: RootSource,
     records: &[CoreRecord<RootSource>],
     in_flight_limit: usize,
@@ -1556,7 +1559,7 @@ pub fn replay_core<
             return Err(failure);
         },
     };
-    let mut expected_next: Option<(InstanceGeneration, JournalCounter)> = None;
+    let mut expected_next: Option<(PrefixKernelInstanceGeneration, JournalCounter)> = None;
     if let Some(counter) = root_own_record_counter {
         expected_next = Some((root_instance, JournalCounter(counter.0 + 1)));
     }
@@ -1564,7 +1567,7 @@ pub fn replay_core<
     if root_checkpoint_txg.0 == u64::MAX {
         return Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor);
     }
-    let chain_start_txg_without_anchor = CheckpointTxg(root_checkpoint_txg.0 + 1);
+    let chain_start_txg_without_anchor = PrefixKernelCheckpointTxg(root_checkpoint_txg.0 + 1);
     // 这次发布已经过了逐条判定、还没走到末条的记录的下标。
     let mut records_of_the_open_publish: Vec<usize> = Vec::new();
     let ghost mut admitted: Seq<int> = Seq::empty();
@@ -1673,7 +1676,7 @@ pub fn replay_core<
                 ),
             is_open_publish(records_at(records@, as_ints(records_of_the_open_publish@))),
             applied.len() + records_of_the_open_publish@.len() <= index,
-            // 还没接上链首（锚点读不出、一条都没收）时攒着的一定是空的：「锚点读不出时链首序号为 1」与「首条序号为 1」两判冗余靠这一条
+            // 还没接上链首（锚点读不出、一条都没收）时攒着的一定是空的：「锚点读不出时链首序号为 1」交给「首条序号为 1」那一判靠这一条
             expected_next is None ==> records_of_the_open_publish@.len() == 0,
         ensures
             index <= records.len(),
@@ -1731,10 +1734,10 @@ pub fn replay_core<
             Ghost(root_checkpoint_txg),
         ) {
             CandidateVerdict::Admitted => {},
-            CandidateVerdict::BrokenBeforeTheNamedUnits => {
+            CandidateVerdict::FailsACheckBeforeTheNamedUnits => {
                 break ;
             },
-            CandidateVerdict::BrokenAtTheNamedUnits => {
+            CandidateVerdict::NamedUnitsReportedUnverified => {
                 report.verification_failed += 1;
                 break ;
             },
@@ -1903,13 +1906,13 @@ proof fn a_record_without_the_last_flag_alone_is_not_a_concatenation(only: CoreR
 #[cfg(test)]
 mod tests {
     use super::{
-        replay_core, CheckpointTxg, CoreRecord, InstanceGeneration, JournalCounter,
-        JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish, NamedUnitVerification,
-        TransactionNumber,
+        replay_core, CoreRecord, JournalCounter, NamedUnitVerification, PrefixKernelCheckpointTxg,
+        PrefixKernelInstanceGeneration, PrefixKernelJournalRecordOrdinalWithinPublish,
+        PrefixKernelJournalRecordPlaceInPublish, TransactionNumber,
     };
 
-    const ROOT_INSTANCE: InstanceGeneration = InstanceGeneration(1);
-    const ROOT_CHECKPOINT_TXG: CheckpointTxg = CheckpointTxg(1);
+    const ROOT_INSTANCE: PrefixKernelInstanceGeneration = PrefixKernelInstanceGeneration(1);
+    const ROOT_CHECKPOINT_TXG: PrefixKernelCheckpointTxg = PrefixKernelCheckpointTxg(1);
     const ROOT_SEGMENT: u64 = 0;
     const IN_FLIGHT_LIMIT: usize = 16;
     /// 每条记录的事务号取计数器加这个数：只为让事务号与计数器分得开，断言里的事务号从记录上取、不另算。
@@ -1919,16 +1922,18 @@ mod tests {
         counter: u64,
         checkpoint_txg: u64,
         ordinal_within_publish: u32,
-        place_in_publish: JournalRecordPlaceInPublish,
+        place_in_publish: PrefixKernelJournalRecordPlaceInPublish,
         new_root_segment: u64,
     ) -> CoreRecord<u64> {
         CoreRecord {
             instance: ROOT_INSTANCE,
             counter: JournalCounter(counter),
-            checkpoint_txg: CheckpointTxg(checkpoint_txg),
+            checkpoint_txg: PrefixKernelCheckpointTxg(checkpoint_txg),
             transaction: TransactionNumber(counter + TRANSACTION_NUMBER_ABOVE_THE_COUNTER),
             is_commit: true,
-            ordinal_within_publish: JournalRecordOrdinalWithinPublish(ordinal_within_publish),
+            ordinal_within_publish: PrefixKernelJournalRecordOrdinalWithinPublish(
+                ordinal_within_publish,
+            ),
             place_in_publish,
             new_root_source: new_root_segment,
         }
@@ -1943,21 +1948,21 @@ mod tests {
                 1,
                 ROOT_CHECKPOINT_TXG.0,
                 1,
-                JournalRecordPlaceInPublish::LastRecordOfThePublish,
+                PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish,
                 10,
             ),
             record(
                 2,
                 2,
                 1,
-                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
+                PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
                 20,
             ),
             record(
                 3,
                 2,
                 2,
-                JournalRecordPlaceInPublish::LastRecordOfThePublish,
+                PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish,
                 30,
             ),
         ];
@@ -1993,14 +1998,14 @@ mod tests {
                 1,
                 ROOT_CHECKPOINT_TXG.0,
                 1,
-                JournalRecordPlaceInPublish::LastRecordOfThePublish,
+                PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish,
                 10,
             ),
             record(
                 2,
                 2,
                 1,
-                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
+                PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
                 20,
             ),
         ];
diff --git a/crates/singlefs-replay-prefix/verus-mutations.tsv b/crates/singlefs-replay-prefix/verus-mutations.tsv
index 867f645..62d639d 100644
--- a/crates/singlefs-replay-prefix/verus-mutations.tsv
+++ b/crates/singlefs-replay-prefix/verus-mutations.tsv
@@ -5,14 +5,14 @@
 # 跑法：把原文换成替换文，在 crates/singlefs-replay-prefix 下跑
 #   verus --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating src/lib.rs
 # （Verus 0.2026.09.20.aef82ed），输出里有期望那一截、退出码不为 0 算红，然后还原。平时的 cargo test 不跑这张表。
-# 两类行。前 81 行「改注解」：每一条 ensures、每一条循环不变式（含主循环的出口）、每个引理的结论各一行，改坏了证明就断；
+# 两类行。前 80 行「改注解」：每一条 ensures、每一条循环不变式（含主循环的出口）、每个引理的结论各一行，改坏了证明就断；
 # 第一行是接回量测报告里的 red_r2（record_ends_its_publish 函数体与 ensures 一起改成恒真）。
 # 后 13 行「改判定」：只改 replay_core 一路上的一个可执行判定、不碰 spec 与不变式，spec 管得住那一条才红——
 # 攻方 research/prompts/m3-verus-land-code-r1-opus-model/verus-green-mutations.tsv 的 12 行加「删 I-8.3 那一判」
 # （代码轮判决 research/prompts/m3-verus-land-code-r1-main-verification.md 第二节 C2 那一行），扩证明之前这 13 行都验得过。
-ensures·record_ends_its_publish 函数体与 ensures 一起改成恒真（接回量测报告的 red_r2）	crates/singlefs-replay-prefix/src/lib.rs	        is_last_record == is_last_record_of_its_publish(*record),\n{\n    match record.place_in_publish {\n        JournalRecordPlaceInPublish::LastRecordOfThePublish => true,\n        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,\n    }\n}	        is_last_record == true,\n{\n    true\n}	verified, 3 errors
+ensures·record_ends_its_publish 函数体与 ensures 一起改成恒真（接回量测报告的 red_r2）	crates/singlefs-replay-prefix/src/lib.rs	        is_last_record == is_last_record_of_its_publish(*record),\n{\n    match record.place_in_publish {\n        PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish => true,\n        PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,\n    }\n}	        is_last_record == true,\n{\n    true\n}	verified, 3 errors
 ensures·record_ends_its_publish 的 ensures 取反	crates/singlefs-replay-prefix/src/lib.rs	        is_last_record == is_last_record_of_its_publish(*record),	        is_last_record == !is_last_record_of_its_publish(*record),	verified, 4 errors
-ensures·counter_of_the_last_record_the_root_covers 交回的锚点计数器小于 u64 最大值减一	crates/singlefs-replay-prefix/src/lib.rs	        anchor matches Ok(Some(counter)) ==> counter.0 < u64::MAX,	        anchor matches Ok(Some(counter)) ==> counter.0 < u64::MAX - 1,	verified, 2 errors
+ensures·counter_of_the_last_record_the_root_covers 交回的锚点计数器小于 u64 最大值减一	crates/singlefs-replay-prefix/src/lib.rs	        anchor matches Ok(Some(counter)) ==> counter.0 < u64::MAX,	        anchor matches Ok(Some(counter)) ==> counter.0 < u64::MAX - 1,	verified, 1 errors
 ensures·counter_of_the_last_record_the_root_covers 交回的锚点改成不等于 spec 的锚点	crates/singlefs-replay-prefix/src/lib.rs	                Ok(found) => flagged.len() < 2 && found == anchor_of_the_chain(	                Ok(found) => flagged.len() < 2 && found != anchor_of_the_chain(	verified, 2 errors
 ensures·replay_core 整个结论取反	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        replay_follows_the_prefix_rules(	    ensures\n        !replay_follows_the_prefix_rules(	verified, 1 errors
 ensures·replay_core 报「所选根 txg 没有后继」那一臂改成恒假	crates/singlefs-replay-prefix/src/lib.rs	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => flagged.len() < 2\n            && root_checkpoint_txg.0 == u64::MAX,	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => false,	verified, 1 errors
@@ -20,33 +20,32 @@ ensures·replay_core 施加了的每一条都改成要求在所选根之下	crat
 ensures·冒烟：两条记录的整次发布改成断言不是拼接	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        is_concatenation_of_whole_publishes(seq![first, second]),	    ensures\n        !is_concatenation_of_whole_publishes(seq![first, second]),	verified, 1 errors
 ensures·冒烟：一条不带末条标志的记录改成断言是拼接	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        !is_concatenation_of_whole_publishes(seq![only]),	    ensures\n        is_concatenation_of_whole_publishes(seq![only]),	verified, 1 errors
 ensures·is_above_the_chosen_root 交回的判定取反	crates/singlefs-replay-prefix/src/lib.rs	        is_above == above_the_chosen_root(*record, root_instance, root_checkpoint_txg),	        is_above == !above_the_chosen_root(*record, root_instance, root_checkpoint_txg),	verified, 3 errors
-ensures·readable_record_of_the_same_publish_follows 交回的判定取反	crates/singlefs-replay-prefix/src/lib.rs	        follows == a_later_record_of_the_same_publish_is_readable(records@, index as int),	        follows == !a_later_record_of_the_same_publish_is_readable(records@, index as int),	verified, 3 errors
+ensures·readable_record_of_the_same_publish_follows 交回的判定取反	crates/singlefs-replay-prefix/src/lib.rs	        follows == a_later_record_of_the_same_publish_is_readable(records@, index as int),	        follows == !a_later_record_of_the_same_publish_is_readable(records@, index as int),	verified, 4 errors
 ensures·maximum_transaction_of 交回的改成严格大于起点	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        maximum.0 >= start.0,	    ensures\n        maximum.0 > start.0,	verified, 2 errors
 ensures·maximum_transaction_of 攒着的每条事务号改成严格小于交回的	crates/singlefs-replay-prefix/src/lib.rs	                ==> records@[#[trigger] records_of_the_open_publish@[position] as int].transaction.0\n                <= maximum.0,	                ==> records@[#[trigger] records_of_the_open_publish@[position] as int].transaction.0\n                < maximum.0,	verified, 1 errors
 ensures·maximum_transaction_of 交回的改成「不等于起点、或是攒着的某一条的」	crates/singlefs-replay-prefix/src/lib.rs	        maximum == start || exists|position: int|	        maximum != start || exists|position: int|	verified, 2 errors
 ensures·judge_the_candidate 接受时读者规则改成不成立	crates/singlefs-replay-prefix/src/lib.rs	                ) && obeys_the_reader_rules_and_the_commit_markers(records@[index as int], previous)	                ) && !obeys_the_reader_rules_and_the_commit_markers(records@[index as int], previous)	verified, 2 errors
-ensures·judge_the_candidate 点名单元之前断时改成「没有一判不过」	crates/singlefs-replay-prefix/src/lib.rs	                CandidateVerdict::BrokenBeforeTheNamedUnits => fails_a_check_before_the_named_units(	                CandidateVerdict::BrokenBeforeTheNamedUnits => !fails_a_check_before_the_named_units(	verified, 2 errors
-ensures·judge_the_candidate 断在点名单元时改成点名单元之前就有一判不过	crates/singlefs-replay-prefix/src/lib.rs	                CandidateVerdict::BrokenAtTheNamedUnits => passes_every_check_before_the_named_units(	                CandidateVerdict::BrokenAtTheNamedUnits => !passes_every_check_before_the_named_units(	verified, 2 errors
+ensures·judge_the_candidate 点名单元之前断时改成「没有一判不过」	crates/singlefs-replay-prefix/src/lib.rs	                CandidateVerdict::FailsACheckBeforeTheNamedUnits => fails_a_check_before_the_named_units(	                CandidateVerdict::FailsACheckBeforeTheNamedUnits => !fails_a_check_before_the_named_units(	verified, 2 errors
+ensures·judge_the_candidate 断在点名单元时改成点名单元之前就有一判不过	crates/singlefs-replay-prefix/src/lib.rs	                CandidateVerdict::NamedUnitsReportedUnverified => passes_every_check_before_the_named_units(	                CandidateVerdict::NamedUnitsReportedUnverified => !passes_every_check_before_the_named_units(	verified, 2 errors
 ensures·引理：候选个数改成严格小于终点	crates/singlefs-replay-prefix/src/lib.rs	            &&& candidates.len() <= end\n	            &&& candidates.len() < end\n	verified, 1 errors
 ensures·引理：短终点的候选个数改成严格少于长终点的	crates/singlefs-replay-prefix/src/lib.rs	            &&& fewer.len() <= more.len()	            &&& fewer.len() < more.len()	verified, 1 errors
-ensures·引理：同一实例里切片次序改成与计数器次序相反	crates/singlefs-replay-prefix/src/lib.rs	        first < second <==> records[first].counter.0 < records[second].counter.0,	        first < second <==> records[first].counter.0 > records[second].counter.0,	verified, 4 errors
+ensures·引理：同一实例里切片次序改成与计数器次序相反	crates/singlefs-replay-prefix/src/lib.rs	        first < second <==> records[first].counter.0 < records[second].counter.0,	        first < second <==> records[first].counter.0 > records[second].counter.0,	verified, 3 errors
 ensures·引理：接受下一个候选之后过了逐条判定的多算一条	crates/singlefs-replay-prefix/src/lib.rs	            0 <= position_in_chain < admitted_before.len() + 1 ==> #[trigger] admitted_record_passed_every_check(	            0 <= position_in_chain < admitted_before.len() + 2 ==> #[trigger] admitted_record_passed_every_check(	verified, 1 errors
 ensures·引理：接受下一个候选之后攒着的改成不是这一串的尾巴	crates/singlefs-replay-prefix/src/lib.rs	        as_ints(open_before.push(index)) == admitted_before.push(index as int).subrange(	        as_ints(open_before.push(index)) != admitted_before.push(index as int).subrange(	verified, 2 errors
 ensures·引理：接受了带末条标志的候选之后攒着的改成不是一次整的发布	crates/singlefs-replay-prefix/src/lib.rs	            &&& is_last_record_of_its_publish(records[index as int]) ==> is_whole_publish(after)	            &&& is_last_record_of_its_publish(records[index as int]) ==> !is_whole_publish(after)	verified, 1 errors
 ensures·引理：已施加的与攒着的接起来改成不是整串	crates/singlefs-replay-prefix/src/lib.rs	        applied + open_tail == admitted,	        applied + open_tail != admitted,	verified, 2 errors
 ensures·引理：接上一次整的发布之后改成不是整次发布的拼接	crates/singlefs-replay-prefix/src/lib.rs	        is_concatenation_of_whole_publishes(records_at(records, applied_before + this_publish)),	        !is_concatenation_of_whole_publishes(records_at(records, applied_before + this_publish)),	verified, 2 errors
 ensures·引理：接上一次发布之后最大事务号改成不是施加的里最大的	crates/singlefs-replay-prefix/src/lib.rs	        maximum_transaction_is_the_largest_applied(\n            records,\n            applied_before + as_ints(records_of_the_open_publish),	        !maximum_transaction_is_the_largest_applied(\n            records,\n            applied_before + as_ints(records_of_the_open_publish),	verified, 2 errors
-循环不变式·readable_record_of_the_same_publish_follows 下标到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            index < later_index <= records.len(),	            index < later_index < records.len(),	verified, 3 errors
+循环不变式·readable_record_of_the_same_publish_follows 下标到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            index < later_index <= records.len(),	            index < later_index < records.len(),	verified, 2 errors
 循环不变式·readable_record_of_the_same_publish_follows 这一条改成往后扫到的那一条	crates/singlefs-replay-prefix/src/lib.rs	            *record == records@[index as int],	            *record == records@[later_index as int],	verified, 2 errors
-循环不变式·readable_record_of_the_same_publish_follows 入参次序改成不升序	crates/singlefs-replay-prefix/src/lib.rs	            records_ascend_by_instance_and_counter(records@),\n            next_counter == records@[index as int].counter.0 + 1,	            !records_ascend_by_instance_and_counter(records@),\n            next_counter == records@[index as int].counter.0 + 1,	verified, 2 errors
-循环不变式·readable_record_of_the_same_publish_follows 下一号改成加二	crates/singlefs-replay-prefix/src/lib.rs	            next_counter == records@[index as int].counter.0 + 1,	            next_counter == records@[index as int].counter.0 + 2,	verified, 2 errors
+循环不变式·readable_record_of_the_same_publish_follows 入参次序改成不升序	crates/singlefs-replay-prefix/src/lib.rs	            *record == records@[index as int],\n            records_ascend_by_instance_and_counter(records@),	            *record == records@[index as int],\n            !records_ascend_by_instance_and_counter(records@),	verified, 2 errors
 循环不变式·readable_record_of_the_same_publish_follows 扫过的同实例后继改成 txg 都更大	crates/singlefs-replay-prefix/src/lib.rs	                ) ==> records@[between].checkpoint_txg.0 < records@[index as int].checkpoint_txg.0,	                ) ==> records@[between].checkpoint_txg.0 > records@[index as int].checkpoint_txg.0,	verified, 1 errors
 循环不变式·counter_of_the_last_record_the_root_covers 下标到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	        invariant\n            index <= records.len(),\n            every_counter_has_a_successor(records@),	        invariant\n            index < records.len(),\n            every_counter_has_a_successor(records@),	verified, 2 errors
 循环不变式·counter_of_the_last_record_the_root_covers 计数器改成有到顶的	crates/singlefs-replay-prefix/src/lib.rs	        invariant\n            index <= records.len(),\n            every_counter_has_a_successor(records@),	        invariant\n            index <= records.len(),\n            !every_counter_has_a_successor(records@),	verified, 2 errors
-循环不变式·counter_of_the_last_record_the_root_covers 收下的计数器改成多看一条	crates/singlefs-replay-prefix/src/lib.rs	                root_checkpoint_txg,\n                index as int,\n            ),\n            forall|position: int|\n                0 <= position < counters_carrying_the_last_record_flag@.len()	                root_checkpoint_txg,\n                index as int + 1,\n            ),\n            forall|position: int|\n                0 <= position < counters_carrying_the_last_record_flag@.len()	verified, 2 errors
+循环不变式·counter_of_the_last_record_the_root_covers 收下的计数器改成多看一条	crates/singlefs-replay-prefix/src/lib.rs	                root_checkpoint_txg,\n                index as int,\n            ),\n            forall|position: int|\n                0 <= position < counters_carrying_the_last_record_flag@.len()	                root_checkpoint_txg,\n                index as int + 1,\n            ),\n            forall|position: int|\n                0 <= position < counters_carrying_the_last_record_flag@.len()	verified, 3 errors
 循环不变式·counter_of_the_last_record_the_root_covers 收下的计数器上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                    ==> (#[trigger] counters_carrying_the_last_record_flag@[position]).0 < u64::MAX,	                    ==> (#[trigger] counters_carrying_the_last_record_flag@[position]).0 < u64::MAX - 1,	verified, 1 errors
 循环不变式·counter_of_the_last_record_the_root_covers 收下的计数器改成要求小于 1	crates/singlefs-replay-prefix/src/lib.rs	                    ==> (#[trigger] counters_carrying_the_last_record_flag@[position]).0 < u64::MAX,	                    ==> (#[trigger] counters_carrying_the_last_record_flag@[position]).0 < 1,	verified, 1 errors
-循环不变式·maximum_transaction_of 位置到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            position <= records_of_the_open_publish@.len(),	            position < records_of_the_open_publish@.len(),	verified, 2 errors
+循环不变式·maximum_transaction_of 位置到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            position <= records_of_the_open_publish@.len(),	            position < records_of_the_open_publish@.len(),	verified, 3 errors
 循环不变式·maximum_transaction_of 攒着的下标上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                    ==> (#[trigger] records_of_the_open_publish@[open_position]) < records.len(),	                    ==> (#[trigger] records_of_the_open_publish@[open_position]) + 1 < records.len(),	verified, 1 errors
 循环不变式·maximum_transaction_of 当前最大改成严格大于起点	crates/singlefs-replay-prefix/src/lib.rs	            maximum.0 >= start.0,\n            forall|open_position: int|	            maximum.0 > start.0,\n            forall|open_position: int|	verified, 1 errors
 循环不变式·maximum_transaction_of 看过的每条事务号改成严格小于当前最大	crates/singlefs-replay-prefix/src/lib.rs	                    ==> records@[#[trigger] records_of_the_open_publish@[open_position] as int].transaction.0\n                    <= maximum.0,	                    ==> records@[#[trigger] records_of_the_open_publish@[open_position] as int].transaction.0\n                    < maximum.0,	verified, 1 errors
@@ -58,14 +57,14 @@ ensures·引理：接上一次发布之后最大事务号改成不是施加的
 循环不变式·主循环校验不过条数改成 1	crates/singlefs-replay-prefix/src/lib.rs	            report.verification_failed == 0,	            report.verification_failed == 1,	verified, 2 errors
 循环不变式·主循环下一个该接的 jsn 改成不是这一串定下的	crates/singlefs-replay-prefix/src/lib.rs	            expected_next == expected_next_after(records@, admitted, anchor, root_instance),	            expected_next != expected_next_after(records@, admitted, anchor, root_instance),	verified, 2 errors
 循环不变式·主循环下标到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            index <= records.len(),\n            root_checkpoint_txg.0 < u64::MAX,	            index < records.len(),\n            root_checkpoint_txg.0 < u64::MAX,	verified, 2 errors
-循环不变式·主循环所选根 txg 上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	            root_checkpoint_txg.0 < u64::MAX,	            root_checkpoint_txg.0 < u64::MAX - 1,	verified, 2 errors
+循环不变式·主循环所选根 txg 上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	            root_checkpoint_txg.0 < u64::MAX,	            root_checkpoint_txg.0 < u64::MAX - 1,	verified, 1 errors
 循环不变式·主循环无锚点链首 txg 改成根加二	crates/singlefs-replay-prefix/src/lib.rs	            chain_start_txg_without_anchor.0 == root_checkpoint_txg.0 + 1,	            chain_start_txg_without_anchor.0 == root_checkpoint_txg.0 + 2,	verified, 2 errors
 循环不变式·主循环入参次序改成不升序	crates/singlefs-replay-prefix/src/lib.rs	            records_ascend_by_instance_and_counter(records@),\n            every_counter_has_a_successor(records@),	            !records_ascend_by_instance_and_counter(records@),\n            every_counter_has_a_successor(records@),	verified, 2 errors
 循环不变式·主循环计数器改成有到顶的	crates/singlefs-replay-prefix/src/lib.rs	            records_ascend_by_instance_and_counter(records@),\n            every_counter_has_a_successor(records@),	            records_ascend_by_instance_and_counter(records@),\n            !every_counter_has_a_successor(records@),	verified, 2 errors
 循环不变式·主循环反向链回调的前置条件多要一格下标	crates/singlefs-replay-prefix/src/lib.rs	                record_index < records@.len() ==> #[trigger] back_chain_broken.requires(\n                    (record_index,),	                record_index <= records@.len() ==> #[trigger] back_chain_broken.requires(\n                    (record_index,),	verified, 1 errors
 循环不变式·主循环点名校验回调的前置条件多要一格下标	crates/singlefs-replay-prefix/src/lib.rs	                record_index < records@.len() ==> #[trigger] named_units_verified.requires(\n                    (record_index,),\n                ),	                record_index <= records@.len() ==> #[trigger] named_units_verified.requires(\n                    (record_index,),\n                ),	verified, 1 errors
 循环不变式·主循环锚点改成不是 spec 的锚点	crates/singlefs-replay-prefix/src/lib.rs	            anchor == anchor_of_the_chain(records@, root_instance, root_checkpoint_txg),	            anchor != anchor_of_the_chain(records@, root_instance, root_checkpoint_txg),	verified, 1 errors
-循环不变式·主循环锚点计数器上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	            anchor matches Some(anchor_counter) ==> anchor_counter.0 < u64::MAX,	            anchor matches Some(anchor_counter) ==> anchor_counter.0 < u64::MAX - 1,	verified, 2 errors
+循环不变式·主循环锚点计数器上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	            anchor matches Some(anchor_counter) ==> anchor_counter.0 < u64::MAX,	            anchor matches Some(anchor_counter) ==> anchor_counter.0 < u64::MAX - 1,	verified, 1 errors
 循环不变式·主循环水位之上条数改成多一	crates/singlefs-replay-prefix/src/lib.rs	                records@.len() as int,\n            ).len(),	                records@.len() as int,\n            ).len() + 1,	verified, 1 errors
 循环不变式·主循环过了逐条判定的改成多看一条的候选	crates/singlefs-replay-prefix/src/lib.rs	                index as int,\n            ),\n            admitted.len() <= in_flight_limit,	                index as int + 1,\n            ),\n            admitted.len() <= in_flight_limit,	verified, 2 errors
 循环不变式·主循环过了逐条判定的条数改成严格小于在飞上限	crates/singlefs-replay-prefix/src/lib.rs	            admitted.len() <= in_flight_limit,	            admitted.len() < in_flight_limit,	verified, 2 errors
@@ -85,7 +84,7 @@ ensures·引理：接上一次发布之后最大事务号改成不是施加的
 循环不变式·主循环已施加的改成都在攒着的之后	crates/singlefs-replay-prefix/src/lib.rs	                    < records_of_the_open_publish@.len() ==> applied[applied_position]\n                    < records_of_the_open_publish@[open_position],	                    < records_of_the_open_publish@.len() ==> applied[applied_position]\n                    > records_of_the_open_publish@[open_position],	verified, 1 errors
 循环不变式·主循环攒着的每一条改成要求在所选根之下	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < records_of_the_open_publish@.len() ==> above_the_chosen_root(	                0 <= position < records_of_the_open_publish@.len() ==> !above_the_chosen_root(	verified, 1 errors
 循环不变式·主循环攒着的改成不是未完的一次发布	crates/singlefs-replay-prefix/src/lib.rs	            is_open_publish(records_at(records@, as_ints(records_of_the_open_publish@))),	            !is_open_publish(records_at(records@, as_ints(records_of_the_open_publish@))),	verified, 2 errors
-循环不变式·主循环已施加加攒着的条数改成严格小于下标	crates/singlefs-replay-prefix/src/lib.rs	            applied.len() + records_of_the_open_publish@.len() <= index,	            applied.len() + records_of_the_open_publish@.len() < index,	verified, 1 errors
+循环不变式·主循环已施加加攒着的条数改成严格小于下标	crates/singlefs-replay-prefix/src/lib.rs	            applied.len() + records_of_the_open_publish@.len() <= index,	            applied.len() + records_of_the_open_publish@.len() < index,	verified, 2 errors
 循环不变式·主循环没接上链首时攒着为空改成接上之后攒着为空（「锚点读不出时链首序号为 1」那一判冗余的证明跟着断）	crates/singlefs-replay-prefix/src/lib.rs	            expected_next is None ==> records_of_the_open_publish@.len() == 0,	            expected_next is Some ==> records_of_the_open_publish@.len() == 0,	verified, 1 errors
 循环出口·主循环出口改成下标没到末尾	crates/singlefs-replay-prefix/src/lib.rs	            index <= records.len(),\n            index == records.len() ==> report.verification_failed == 0,	            index < records.len(),\n            index == records.len() ==> report.verification_failed == 0,	verified, 1 errors
 循环出口·主循环走完时校验不过条数改成 1	crates/singlefs-replay-prefix/src/lib.rs	            index == records.len() ==> report.verification_failed == 0,	            index == records.len() ==> report.verification_failed == 1,	verified, 2 errors
@@ -98,7 +97,7 @@ ensures·引理：接上一次发布之后最大事务号改成不是施加的
 改判定·点名单元不验（已定项 14 五条口径第四条；攻方 V05）	crates/singlefs-replay-prefix/src/lib.rs	        NamedUnitVerification::Verify => named_units_verified(index),	        NamedUnitVerification::Verify => true,	verified, 1 errors
 改判定·在飞上限不看（已定项 18；攻方 V06）	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {	        if false {	verified, 1 errors
 改判定·第一条水位之上的记录就停、一条都不施加（攻方 V07）	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {	        if true {	verified, 1 errors
-改判定·锚点读不出时链首不看 txg（已定项 14 注 1；三方第一轮 K4-a；攻方 V08）	crates/singlefs-replay-prefix/src/lib.rs	            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0\n                || record.ordinal_within_publish.0 != 1 {	            if false\n                || record.ordinal_within_publish.0 != 1 {	verified, 1 errors
+改判定·锚点读不出时链首不看 txg（已定项 14 注 1；三方第一轮 K4-a；攻方 V08）	crates/singlefs-replay-prefix/src/lib.rs	            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0 {	            if false {	verified, 1 errors
 改判定·每次都报所选根 txg 没有后继（攻方 V09）	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg.0 == u64::MAX {	    if true {	verified, 1 errors
 改判定·最大事务号取最小（已定项 14 注 4 的 W 的输入；攻方 V10）	crates/singlefs-replay-prefix/src/lib.rs	        if transaction.0 > maximum.0 {	        if transaction.0 < maximum.0 {	verified, 1 errors
 改判定·水位之上条数不数（报告字段；攻方 V11）	crates/singlefs-replay-prefix/src/lib.rs	            report.above_water += 1;	            report.above_water += 0;	verified, 1 errors
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 8ab2e97..c663c8a 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -18,7 +18,10 @@ use singlefs_format::{
     SYSTEM_CONFIGURATION_SLOT_BYTES,
 };
 use singlefs_replay_prefix::{
-    replay_core, CoreFailure, CoreRecord, JournalCounter, NamedUnitVerification, TransactionNumber,
+    replay_core, CoreFailure, CoreRecord, JournalCounter, NamedUnitVerification,
+    PrefixKernelCheckpointTxg, PrefixKernelInstanceGeneration,
+    PrefixKernelJournalRecordOrdinalWithinPublish, PrefixKernelJournalRecordPlaceInPublish,
+    TransactionNumber,
 };
 
 use crate::address::{
@@ -2980,6 +2983,14 @@ fn judge_back_chain_against_the_previous_record_on_disk(
     BackChainJudgement::PreviousRecordNotOnDisk
 }
 
+/// [`replay_journal`] 交给取前缀核的「新根从哪来」：核不看它，施加了发布就交回最后施加的那次发布末条带的那一项，
+/// 一次都没施加就交回调用方给的那一项。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum NewRootSource {
+    ChosenRootItself,
+    NewRootSegmentOfTheRecordAtKeyOrderIndex(usize),
+}
+
 /// 所选根的 checkpoint_txg 是 u64 的最大值时 [`replay_journal`] 报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
 pub const CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR: &str =
     "所选根的 checkpoint_txg 是 u64 的最大值：接在它后面的那次发布取不到 txg";
@@ -3034,28 +3045,29 @@ pub fn replay_journal(
         );
     }
     let records_in_key_order: Vec<&JournalRecord> = records.values().collect();
-    // 核的「新根从哪来」取记录在表里的下标：`None` 是所选根自己，`Some(下标)` 是最后施加的那次发布的末条。
-    let core_records: Vec<CoreRecord<Option<usize>>> = records_in_key_order
+    let core_records: Vec<CoreRecord<NewRootSource>> = records_in_key_order
         .iter()
         .enumerate()
-        .map(|(index, record)| CoreRecord {
-            instance: singlefs_replay_prefix::InstanceGeneration(record.instance.0),
+        .map(|(index_in_key_order, record)| CoreRecord {
+            instance: PrefixKernelInstanceGeneration(record.instance.0),
             counter: JournalCounter(record.counter),
-            checkpoint_txg: singlefs_replay_prefix::CheckpointTxg(record.checkpoint_txg.0),
+            checkpoint_txg: PrefixKernelCheckpointTxg(record.checkpoint_txg.0),
             transaction: TransactionNumber(record.transaction),
             is_commit: record.is_commit,
-            ordinal_within_publish: singlefs_replay_prefix::JournalRecordOrdinalWithinPublish(
+            ordinal_within_publish: PrefixKernelJournalRecordOrdinalWithinPublish(
                 record.ordinal_within_publish.0,
             ),
             place_in_publish: match record.place_in_publish {
                 JournalRecordPlaceInPublish::LastRecordOfThePublish => {
-                    singlefs_replay_prefix::JournalRecordPlaceInPublish::LastRecordOfThePublish
+                    PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish
                 }
                 JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => {
-                    singlefs_replay_prefix::JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
+                    PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
                 }
             },
-            new_root_source: Some(index),
+            new_root_source: NewRootSource::NewRootSegmentOfTheRecordAtKeyOrderIndex(
+                index_in_key_order,
+            ),
         })
         .collect();
     // 反向链（D23（journal 的角色与格式） 已定项 7「jsn 严格连续 + 校验和 + 反向链」；I-8.6（反向链算法））：
@@ -3092,9 +3104,9 @@ pub fn replay_journal(
         NamedUnitVerification::Skip
     };
     let outcome = match replay_core(
-        singlefs_replay_prefix::InstanceGeneration(root.instance.0),
-        singlefs_replay_prefix::CheckpointTxg(root.checkpoint_txg.0),
-        None,
+        PrefixKernelInstanceGeneration(root.instance.0),
+        PrefixKernelCheckpointTxg(root.checkpoint_txg.0),
+        NewRootSource::ChosenRootItself,
         &core_records,
         in_flight_limit,
         named_unit_verification,
@@ -3126,9 +3138,9 @@ pub fn replay_journal(
     report.verification_failed = outcome.report.verification_failed;
     report.maximum_applied_transaction = outcome.report.maximum_applied_transaction.0;
     let rebuilt = match outcome.root_source_after_replay {
-        None => *root,
-        Some(index) => {
-            let record = records_in_key_order[index];
+        NewRootSource::ChosenRootItself => *root,
+        NewRootSource::NewRootSegmentOfTheRecordAtKeyOrderIndex(index_in_key_order) => {
+            let record = records_in_key_order[index_in_key_order];
             // 每条记录都带整次发布的新根段（已定项 15），施加末条那一份。
             RootRecord {
                 filesystem_identifier: root.filesystem_identifier,
diff --git a/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs b/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs
index e19d1e1..c6e7d39 100644
--- a/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs
+++ b/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs
@@ -18,17 +18,19 @@ use singlefs_core::journal::{
 use singlefs_core::pointer::NodePointer;
 use singlefs_core::recovery::{replay_journal, RecoveryFailure};
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
+use singlefs_core::unit::unit_filesystem_identifier;
 use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
 use singlefs_harness::memory_pool::MemoryPool;
 
 const INSTANCE: InstanceGeneration = InstanceGeneration(1);
 const CHOSEN_ROOT_CHECKPOINT_TXG: CheckpointTxg = CheckpointTxg(2);
-/// 记录与根共用的 fsid 低 8 字节；读者是空池，不拿它比。
-const FILESYSTEM_IDENTIFIER_LOW_BYTES: u64 = 0x5349_4E47_4C45_4653;
+/// 根的 fsid 每个字节都取这个值；记录带它的低 8 字节（`unit_filesystem_identifier`）。读者是空池，反向链重读不了，取前缀不拿它比。
+const FILESYSTEM_IDENTIFIER_BYTE: u8 = 0x53;
+const FILESYSTEM_IDENTIFIER: [u8; 16] = [FILESYSTEM_IDENTIFIER_BYTE; 16];
 
 fn chosen_root() -> RootRecord {
     RootRecord {
-        filesystem_identifier: [0x53; 16],
+        filesystem_identifier: FILESYSTEM_IDENTIFIER,
         unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
         instance: INSTANCE,
         checkpoint_txg: CHOSEN_ROOT_CHECKPOINT_TXG,
@@ -52,7 +54,7 @@ fn single_record_publish(counter: u64, checkpoint_txg: CheckpointTxg) -> Journal
         ordinal_within_publish: JournalRecordOrdinalWithinPublish(1),
         place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
         back_chain: 0,
-        filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_BYTES,
+        filesystem_identifier: unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER),
         new_tree_table: NodePointer::empty_root(),
         new_mapping_root: NodePointer::empty_root(),
         new_tree_identifier_watermark: 0,
diff --git a/crates/singlefs-harness/tests/journal_replay_applies_no_more_records_than_the_in_flight_limit.rs b/crates/singlefs-harness/tests/journal_replay_applies_no_more_records_than_the_in_flight_limit.rs
index b00fbe5..0a2b88e 100644
--- a/crates/singlefs-harness/tests/journal_replay_applies_no_more_records_than_the_in_flight_limit.rs
+++ b/crates/singlefs-harness/tests/journal_replay_applies_no_more_records_than_the_in_flight_limit.rs
@@ -31,9 +31,27 @@ const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)]
 /// 3 个记录槽的环：在飞上限 = 3 ÷ F（3）= 1。
 const THREE_RECORD_RING_BYTES: u64 = 3 * JOURNAL_RECORD_BYTES;
 
+/// 文件内容第 `index` 个字节取 (`index` × 步长 + 种子) mod 模数：模数小于 256，字节装得进 u8；
+/// 步长与模数（11 × 23）互素，内容的周期是整个模数，换个种子就逐字节分得开。
+const CONTENT_BYTE_STRIDE: usize = 7;
+const CONTENT_BYTE_MODULUS_BELOW_256: usize = 253;
+/// B、C 两版文件的字节数与内容种子：两版都只切出一条记录，长度与种子各不相同，读回哪一版看得出。
+const CONTENT_BYTES_OF_PUBLISH_B: usize = 4100;
+const CONTENT_SEED_OF_PUBLISH_B: usize = 3;
+const CONTENT_BYTES_OF_PUBLISH_C: usize = 2500;
+const CONTENT_SEED_OF_PUBLISH_C: usize = 11;
+/// 覆盖写那几版的写入时间比池里第一版晚这么多秒。
+const OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE: u64 = 60;
+/// 改坏根槽时翻的那个字节在根记录里的下标：落在树表指针那一段（根记录 magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 之后），
+/// 翻了自证校验和就对不上。
+const BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER: u64 = 100;
+
 fn content_of(length: usize, seed: usize) -> Vec<u8> {
     (0..length)
-        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
+        .map(|index| {
+            u8::try_from((index * CONTENT_BYTE_STRIDE + seed) % CONTENT_BYTE_MODULUS_BELOW_256)
+                .expect("小于 256")
+        })
         .collect()
 }
 
@@ -44,7 +62,7 @@ fn overwrite_in_process(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutp
         pool,
         &previous,
         content,
-        FIXED_WRITE_TIME_SECONDS + 60,
+        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
         InstanceGeneration(1),
     )
     .expect("覆盖写");
@@ -59,7 +77,11 @@ fn tear_root_slot(image: &mut MemoryPool, txg: u64) {
         parameters().geometry.root_ring_slots_per_region,
     );
     let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
-    image.flip_byte(device, slot_offset(target, 4096), 100);
+    image.flip_byte(
+        device,
+        slot_offset(target, parameters().geometry.fixed_structure_slot_spacing),
+        BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER,
+    );
 }
 
 /// 把计数器 `counters` 那几条记录从默认环的落点搬到 3 槽环的落点（两盘各一份；先全部读出再写，搬过去的会盖掉更早的记录）。
@@ -101,8 +123,11 @@ fn with_an_in_flight_limit_of_one_only_the_first_of_two_single_record_publishes_
 ) {
     let mut pool = build_pool("in-flight-limit-of-one");
     let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
-    let second = overwrite_in_process(&mut pool, &content_of(4100, 3));
-    let third_content = content_of(2500, 11);
+    let second = overwrite_in_process(
+        &mut pool,
+        &content_of(CONTENT_BYTES_OF_PUBLISH_B, CONTENT_SEED_OF_PUBLISH_B),
+    );
+    let third_content = content_of(CONTENT_BYTES_OF_PUBLISH_C, CONTENT_SEED_OF_PUBLISH_C);
     let third = overwrite_in_process(&mut pool, &third_content);
     assert_eq!(
         (
diff --git a/crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs b/crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs
index ba9628e..dc22221 100644
--- a/crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs
+++ b/crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs
@@ -60,13 +60,27 @@ fn third_content() -> Vec<u8> {
     content_of(THIRD_FILE_BYTES, 11)
 }
 
+/// 交给写者的上一版的回退下界 F：照旧，或抬到给定值（只改上一版的 F，别的照旧）。抬过的那一版记录新根段里的 F 与所选根的不同，
+/// 施加出来的根取的是哪一份才分得出来（D23（journal 的角色与格式） 已定项 15「施加一条记录 = 把所选根的这四个字段换成记录里的」）。
+enum RollbackFloorHandedToTheWriter {
+    AsInThePreviousVersion,
+    RaisedTo(CheckpointTxg),
+}
+
 /// 同一个进程、同一个实例里再发布一版（发布 B 用；步 3 的「预置一条实例 1 的记录」也用它造）。
 fn overwrite_in_process(
     pool: &mut BuiltPool,
     content: &[u8],
     instance: InstanceGeneration,
+    rollback_floor: RollbackFloorHandedToTheWriter,
 ) -> TransactionOutput {
-    let previous = pool.output.clone();
+    let mut previous = pool.output.clone();
+    match rollback_floor {
+        RollbackFloorHandedToTheWriter::AsInThePreviousVersion => {}
+        RollbackFloorHandedToTheWriter::RaisedTo(raised_floor) => {
+            previous.root.rollback_floor = raised_floor;
+        }
+    }
     common::publish_overwrite_in_process(
         pool,
         &previous,
@@ -77,31 +91,15 @@ fn overwrite_in_process(
     .expect("覆盖写")
 }
 
-/// 同 [`overwrite_in_process`]，这一版的回退下界 F 抬到 `raised_floor`（只改交给写者的上一版的 F，别的照旧）：
-/// 记录新根段里的 F 与所选根的不同，施加出来的根取的是哪一份才分得出来
-/// （D23（journal 的角色与格式） 已定项 15「施加一条记录 = 把所选根的这四个字段换成记录里的」）。
-fn overwrite_in_process_raising_the_floor(
-    pool: &mut BuiltPool,
-    content: &[u8],
-    instance: InstanceGeneration,
-    raised_floor: CheckpointTxg,
-) -> TransactionOutput {
-    let mut previous_with_the_raised_floor = pool.output.clone();
-    previous_with_the_raised_floor.root.rollback_floor = raised_floor;
-    common::publish_overwrite_in_process(
-        pool,
-        &previous_with_the_raised_floor,
-        content,
-        FIXED_WRITE_TIME_SECONDS + 60,
-        instance,
-    )
-    .expect("覆盖写")
-}
-
 /// A → B 之后进程退出、重开并可写挂载。
 fn build_publish_second_version_and_remount(tag: &str) -> (BuiltPool, TransactionOutput, Mounted) {
     let mut pool = build_pool(tag);
-    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
+    let second = overwrite_in_process(
+        &mut pool,
+        &second_content(),
+        InstanceGeneration(1),
+        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
+    );
     pool.output = second.clone();
     let mut devices = pool.reopen_recorded();
     let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
@@ -427,13 +425,18 @@ fn damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_oth
 fn stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row(
 ) {
     let mut pool = build_pool("step-three-stray-record");
-    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
+    let second = overwrite_in_process(
+        &mut pool,
+        &second_content(),
+        InstanceGeneration(1),
+        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
+    );
     pool.output = second;
-    let stray = overwrite_in_process_raising_the_floor(
+    let stray = overwrite_in_process(
         &mut pool,
         &third_content(),
         InstanceGeneration(1),
-        CheckpointTxg(4),
+        RollbackFloorHandedToTheWriter::RaisedTo(CheckpointTxg(4)),
     );
     assert_eq!(
         (
@@ -578,11 +581,26 @@ fn checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_r
 fn one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact(
 ) {
     let mut pool = build_pool("step-three-chain-start");
-    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
+    let second = overwrite_in_process(
+        &mut pool,
+        &second_content(),
+        InstanceGeneration(1),
+        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
+    );
     pool.output = second;
-    let third = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
+    let third = overwrite_in_process(
+        &mut pool,
+        &third_content(),
+        InstanceGeneration(1),
+        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
+    );
     pool.output = third;
-    let fourth = overwrite_in_process(&mut pool, &content_of(3300, 17), InstanceGeneration(1));
+    let fourth = overwrite_in_process(
+        &mut pool,
+        &content_of(3300, 17),
+        InstanceGeneration(1),
+        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
+    );
     assert_eq!(
         (fourth.root.checkpoint_txg, fourth.record.counter),
         (CheckpointTxg(6), 6)
@@ -648,14 +666,24 @@ fn torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg(tear:
         AnchorTear::OnlyTheAnchor => (false, CheckpointTxg(6), content_of(3300, 17), 2),
     };
     let mut pool = build_pool("step-three-torn-anchor");
-    pool.output = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
-    pool.output = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
+    pool.output = overwrite_in_process(
+        &mut pool,
+        &second_content(),
+        InstanceGeneration(1),
+        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
+    );
+    pool.output = overwrite_in_process(
+        &mut pool,
+        &third_content(),
+        InstanceGeneration(1),
+        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
+    );
     // 第四版把 F 抬到 5（所选根 (1, 4) 带 F = 0）：只撕锚点那一格施加到 jsn 6，施加出来的根的 F 取 jsn 6 新根段里的 5（已定项 15）。
-    let fourth = overwrite_in_process_raising_the_floor(
+    let fourth = overwrite_in_process(
         &mut pool,
         &content_of(3300, 17),
         InstanceGeneration(1),
-        CheckpointTxg(5),
+        RollbackFloorHandedToTheWriter::RaisedTo(CheckpointTxg(5)),
     );
     assert_eq!(fourth.record.counter, 6);
     let mut image = pool.memory_pool();
diff --git a/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs b/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs
new file mode 100644
index 0000000..b151e1f
--- /dev/null
+++ b/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs
@@ -0,0 +1,309 @@
+//! 挂载时取前缀，水位之上每个候选读几次盘、按什么次序读：反向链重读上一条记录（D23（journal 的角色与格式） 已定项 7、I-8.6）
+//! 与点名单元的两份（已定项 14 五条口径第四条）。
+//!
+//! 取前缀的核（`singlefs_replay_prefix::replay_core`）走到那一判才调两个读盘回调；Verus 的 spec 只说「据以判的那一次回调交回了什么」，
+//! 不管调了几次、按什么次序调（代码轮第二轮判决 `research/prompts/m3-verus-land-code-r2-main-verification.md` 第二节 H2、H4 两行）。
+//! 这里在水位之上摆两次整的发布，挂载时在块层逐次记下读，钉住：环按提示扫完之后，每个候选先在 0 号盘上重读一次上一条记录，
+//! 再按记录里点名的次序把每个点名单元的两份各读一次；两个候选按 jsn 次序；之后环里再没有读。
+//!
+//! 造法同 `journal_replay_applies_no_more_records_than_the_in_flight_limit.rs` 的阳性对照：A（txg 3、jsn 3）的根落了，
+//! B（txg 4、jsn 4）、C（txg 5、jsn 5）两次单条发布的记录与单元都在、根槽都坏了 ⇒ 所选根是 A 的，B、C 都施加。
+
+mod common;
+
+use std::cell::RefCell;
+
+use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
+use singlefs_core::journal::{record_offset, JournalRecord};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::recovery::{choose_system_configuration, scan_journal, PoolReader};
+use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_core::transaction::TransactionOutput;
+use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
+use singlefs_format::{
+    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, NODE_BYTES, SLOT_BYTES,
+};
+use singlefs_harness::memory_pool::MemoryPool;
+
+/// B、C 两版文件的字节数与填充字节：两版都只切出一条记录，长度与内容各不相同。
+const CONTENT_BYTES_OF_PUBLISH_B: usize = 4100;
+const CONTENT_FILL_BYTE_OF_PUBLISH_B: u8 = 0xB0;
+const CONTENT_BYTES_OF_PUBLISH_C: usize = 2500;
+const CONTENT_FILL_BYTE_OF_PUBLISH_C: u8 = 0xC0;
+/// 覆盖写那几版的写入时间比池里第一版晚这么多秒。
+const OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE: u64 = 60;
+/// 改坏根槽时翻的那个字节在根记录里的下标：落在树表指针那一段，翻了自证校验和就对不上。
+const BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER: u64 = 100;
+/// 反向链重读从 `device_identities()` 的第一块盘试起，健康镜像上第一份就对得上。
+const FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD: DeviceIdentity = DeviceIdentity(0);
+
+/// 块层的一次读。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct BlockLayerRead {
+    device: DeviceIdentity,
+    offset: DeviceOffsetInBytes,
+    length_in_bytes: usize,
+}
+
+/// 挂载那一路对读者做的事，按发生的次序。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum ReaderEvent {
+    /// 扫环之前问一次「环里哪几个槽可能有字节」；交回的槽数就是这块盘上扫环要读几次。
+    JournalRecordOffsetsHint {
+        device: DeviceIdentity,
+        offsets: Vec<DeviceOffsetInBytes>,
+    },
+    Read(BlockLayerRead),
+}
+
+/// 包在 [`MemoryPool`] 外面，把每次读与每次要环提示都原样记下来（读是 `&self`，记录走 [`RefCell`]）。
+struct ReadLoggingPoolReader<'inner> {
+    inner: &'inner MemoryPool,
+    events: RefCell<Vec<ReaderEvent>>,
+}
+
+impl PoolReader for ReadLoggingPoolReader<'_> {
+    fn device_identities(&self) -> Vec<DeviceIdentity> {
+        self.inner.device_identities()
+    }
+
+    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
+        self.inner.device_size_in_bytes(device)
+    }
+
+    fn read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        self.events
+            .borrow_mut()
+            .push(ReaderEvent::Read(BlockLayerRead {
+                device,
+                offset,
+                length_in_bytes: length,
+            }));
+        self.inner.read(device, offset, length)
+    }
+
+    fn journal_record_offsets_hint(
+        &self,
+        device: DeviceIdentity,
+        ring_start: DeviceOffsetInBytes,
+        ring_bytes: u64,
+    ) -> Option<Vec<DeviceOffsetInBytes>> {
+        let offsets = self
+            .inner
+            .journal_record_offsets_hint(device, ring_start, ring_bytes);
+        self.events
+            .borrow_mut()
+            .push(ReaderEvent::JournalRecordOffsetsHint {
+                device,
+                offsets: offsets
+                    .clone()
+                    .expect("内存池交得出环提示：它知道哪些扇区写过"),
+            });
+        offsets
+    }
+}
+
+/// 同一个进程、同一个实例里再发布一版（一次发布一条记录）。
+fn overwrite_in_process(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
+    let previous = pool.output.clone();
+    let output = common::publish_overwrite_in_process(
+        pool,
+        &previous,
+        content,
+        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
+        InstanceGeneration(1),
+    )
+    .expect("覆盖写");
+    pool.output = output.clone();
+    output
+}
+
+/// 把 txg 那一版的根槽改坏：只留记录与单元，模拟「根槽没持久」。
+fn tear_root_slot(image: &mut MemoryPool, txg: u64) {
+    let target = target_for_publish(
+        CheckpointTxg(txg),
+        parameters().geometry.root_ring_slots_per_region,
+    );
+    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
+    image.flip_byte(
+        device,
+        slot_offset(target, parameters().geometry.fixed_structure_slot_spacing),
+        BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER,
+    );
+}
+
+/// 一个点名单元的整单元宽：数据单元与码 3 打包记录单元 32768，码 2 索引节点 16384（D23（journal 的角色与格式） 已定项 14 第四条按类读）。
+fn named_unit_bytes(unit_class: u8) -> usize {
+    let unit_bytes = match unit_class {
+        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => DATA_UNIT_BYTES,
+        UNIT_CLASS_INDEX_NODE => NODE_BYTES,
+        unknown_class => {
+            panic!("发布路径只点名数据单元、索引节点与打包记录单元，读到类 {unknown_class}")
+        }
+    };
+    usize::try_from(unit_bytes).expect("单元宽装得进 usize")
+}
+
+/// 候选 `record` 那一判该有的读：0 号盘上它的上一条记录一次，再是它点名的每个单元的两份，按点名次序。
+fn reads_judging_the_candidate(record: &JournalRecord, ring_bytes: u64) -> Vec<ReaderEvent> {
+    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
+    let mut reads = vec![ReaderEvent::Read(BlockLayerRead {
+        device: FIRST_DEVICE_TRIED_BY_THE_BACK_CHAIN_REREAD,
+        offset: record_offset(record.counter - 1, ring_bytes),
+        length_in_bytes: record_bytes,
+    })];
+    for named in &record.named {
+        for location in &named.locations {
+            reads.push(ReaderEvent::Read(BlockLayerRead {
+                device: location.device,
+                offset: location.slot.to_device_offset(),
+                length_in_bytes: named_unit_bytes(named.unit_class),
+            }));
+        }
+    }
+    reads
+}
+
+fn is_inside_the_journal_ring(offset: DeviceOffsetInBytes, ring_bytes: u64) -> bool {
+    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
+    offset.0 >= ring_start && offset.0 < ring_start + ring_bytes
+}
+
+/// A 的根落了、B（jsn 4）与 C（jsn 5）只剩记录与单元：挂载时扫完环，jsn 4 先重读 jsn 3 一次、再读它点名的单元，
+/// 然后 jsn 5 先重读 jsn 4 一次、再读它点名的单元；环里一共只多出这两次读。
+#[test]
+fn mounting_rereads_the_previous_record_once_then_each_named_unit_of_every_candidate_in_journal_order(
+) {
+    let mut pool = build_pool("mounted-recovery-reads-each-candidate-once");
+    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
+    let second = overwrite_in_process(
+        &mut pool,
+        &vec![CONTENT_FILL_BYTE_OF_PUBLISH_B; CONTENT_BYTES_OF_PUBLISH_B],
+    );
+    let third = overwrite_in_process(
+        &mut pool,
+        &vec![CONTENT_FILL_BYTE_OF_PUBLISH_C; CONTENT_BYTES_OF_PUBLISH_C],
+    );
+    assert_eq!(
+        (
+            first_version,
+            (second.root.checkpoint_txg, second.record.counter),
+            (third.root.checkpoint_txg, third.record.counter)
+        ),
+        (
+            (InstanceGeneration(1), CheckpointTxg(3)),
+            (CheckpointTxg(4), 4),
+            (CheckpointTxg(5), 5)
+        ),
+        "A 的根在 txg 3、jsn 3；B、C 各一条记录"
+    );
+    let mut image = pool.memory_pool();
+    for txg in [4u64, 5] {
+        tear_root_slot(&mut image, txg);
+    }
+    let system_configuration = choose_system_configuration(&image).expect("系统配置择得出");
+    let ring_bytes = system_configuration.immutable.sizes.journal_ring_bytes;
+    let records = scan_journal(&image, &system_configuration);
+    let candidate_b = records
+        .get(&(InstanceGeneration(1), 4))
+        .expect("jsn 4 在环里");
+    let candidate_c = records
+        .get(&(InstanceGeneration(1), 5))
+        .expect("jsn 5 在环里");
+
+    let logging = ReadLoggingPoolReader {
+        inner: &image,
+        events: RefCell::new(Vec::new()),
+    };
+    let mounted = mount_read_only(&logging).expect("只读挂载");
+    assert_eq!(
+        (
+            (
+                mounted.chosen_root.instance,
+                mounted.chosen_root.checkpoint_txg
+            ),
+            (
+                mounted.effective_root.instance,
+                mounted.effective_root.checkpoint_txg
+            ),
+            mounted.journal.above_water,
+            mounted.journal.prefix_applied,
+            mounted.journal.verification_passed
+        ),
+        (
+            first_version,
+            (InstanceGeneration(1), CheckpointTxg(5)),
+            2,
+            2,
+            2
+        ),
+        "所选根是 A 的；水位之上 B、C 两条，都过了点名验证、都施加，挂载到 C"
+    );
+
+    let events = logging.events.into_inner();
+    let last_hint_position = events
+        .iter()
+        .rposition(|event| matches!(event, ReaderEvent::JournalRecordOffsetsHint { .. }))
+        .expect("扫环之前要过环提示");
+    let ReaderEvent::JournalRecordOffsetsHint {
+        offsets: offsets_scanned_on_the_last_device,
+        ..
+    } = &events[last_hint_position]
+    else {
+        unreachable!("rposition 找的就是环提示");
+    };
+    let scan_end = last_hint_position + 1 + offsets_scanned_on_the_last_device.len();
+    let expected_reads_judging_the_candidates: Vec<ReaderEvent> = [candidate_b, candidate_c]
+        .into_iter()
+        .flat_map(|record| reads_judging_the_candidate(record, ring_bytes))
+        .collect();
+    assert_eq!(
+        (
+            candidate_b.named.len(),
+            candidate_c.named.len(),
+            expected_reads_judging_the_candidates.len()
+        ),
+        (12, 12, 50),
+        "B、C 各点名 12 个单元；判两个候选的读：各一次反向链重读，加 24 个点名单元两份各一次"
+    );
+    assert_eq!(
+        events.get(scan_end..scan_end + expected_reads_judging_the_candidates.len()),
+        Some(expected_reads_judging_the_candidates.as_slice()),
+        "扫完环之后：jsn 4 重读 jsn 3 一次、读它点名的单元，再 jsn 5 重读 jsn 4 一次、读它点名的单元，不多读、不换次序"
+    );
+    let scan_reads_inside_the_ring: usize = events[..scan_end]
+        .iter()
+        .map(|event| match event {
+            ReaderEvent::JournalRecordOffsetsHint { .. } => 0,
+            ReaderEvent::Read(read) => {
+                usize::from(is_inside_the_journal_ring(read.offset, ring_bytes))
+            }
+        })
+        .sum();
+    let reads_inside_the_ring_after_judging: usize = events
+        [scan_end + expected_reads_judging_the_candidates.len()..]
+        .iter()
+        .map(|event| match event {
+            ReaderEvent::JournalRecordOffsetsHint { .. } => 1,
+            ReaderEvent::Read(read) => {
+                usize::from(is_inside_the_journal_ring(read.offset, ring_bytes))
+            }
+        })
+        .sum();
+    assert_eq!(
+        (
+            scan_reads_inside_the_ring,
+            reads_inside_the_ring_after_judging
+        ),
+        (10, 0),
+        "扫环按提示读 jsn 1–5 五个槽、两块盘各一遍；判完两个候选之后环里不再读、也不再要提示"
+    );
+}
```

## `crates/mutations.tsv` 改写的 4 行（整行替换：名、新原文、新替换文…，见 5-1 交回的 `mutations-replacements.tsv`）

```text
步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条	crates/singlefs-replay-prefix/src/lib.rs	            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0 {	            if false && record.checkpoint_txg.0 != chain_start_txg_without_anchor.0 {	-p singlefs-harness --test writable_mount_takes_a_second_instance -- torn_anchor_record	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg_when_the_next_record_is_torn_too
P6 后一半：锚点读不出时不看本次发布内序号，链首接在下一次发布的第二条上（三方第一轮 K4-b）	crates/singlefs-replay-prefix/src/lib.rs	    if records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 {	    if records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 && expected_next.is_some() {	-p singlefs-harness --test multi_unit_file_write_and_read -- without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish	without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish
实二十：层 0 L8：恢复把不带末条标志的记录也当末条（半次发布施加上去）	crates/singlefs-core/src/recovery.rs	                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => {\n                    PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow\n                }	                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => {\n                    PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish\n                }	-p singlefs-checker-tier --test crash_enumeration_record_spill_over_stream -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
Verus 接回 C5-2（C540 读者规则）：带末条标志的那一条之后同一次发布还有没有记录，只从紧挨着的下一条之后找	crates/singlefs-replay-prefix/src/lib.rs	    let mut later_index = index + 1;	    let mut later_index = index + 2;	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- last_record_flag_followed_only_by_the_very_next_record	last_record_flag_followed_only_by_the_very_next_record_of_the_same_publish_breaks_the_chain_at_the_flag
```

## `crates/mutations.tsv` 追加的 2 行

```text
取前缀核读盘形状（攻方 S03）：反向链回调每个候选调两次，挂载时每个候选重读上一条记录两次	crates/singlefs-replay-prefix/src/lib.rs	    if back_chain_broken(index) {	    if back_chain_broken(index) || back_chain_broken(index) {	-p singlefs-harness --test mounted_recovery_reads_each_candidate_once_in_order -- mounting_rereads_the_previous_record_once	mounting_rereads_the_previous_record_once_then_each_named_unit_of_every_candidate_in_journal_order
取前缀核读盘形状（攻方 S05）：点名回调提前到逐条判定之前，挂载时点名单元先于反向链读、还多读一遍	crates/singlefs-replay-prefix/src/lib.rs	    let record = &records[index];\n    let ghost previous = previous_in_the_chain(records@, admitted, admitted.len() as int);	    let record = &records[index];\n    let _named_units_verified_before_any_check = named_units_verified(index);\n    let ghost previous = previous_in_the_chain(records@, admitted, admitted.len() as int);	-p singlefs-harness --test mounted_recovery_reads_each_candidate_once_in_order -- mounting_rereads_the_previous_record_once	mounting_rereads_the_previous_record_once_then_each_named_unit_of_every_candidate_in_journal_order
```
