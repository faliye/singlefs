# 代码轮第二轮被判的 diff（第一轮冻结副本 → 第二轮冻结副本，第 4 批 4-1、4-2、3-6 改过的文件；`crates/mutations.tsv` 里别的会话追加的行不判）

```diff
--- a/crates/singlefs-replay-prefix/src/lib.rs
+++ b/crates/singlefs-replay-prefix/src/lib.rs
@@ -2,10 +2,12 @@
 //! Verus 验的就是编进产品、被恢复路径调用的这一份（用户 2026-10-01 定「Verus 跑真实代码」；里程碑三七·3）。
 //! 单独成一个 crate、只依赖 `vstd`：Verus 只编这一个文件，不编整个 `singlefs-core`（用户 2026-10-01 定拆）。
 //!
-//! 来历：判定照提交 fcc4d340 的 `crates/singlefs-core/src/recovery.rs` 里 `replay_journal`（第 3057–3229 行）搬过来，
-//! 每个分支旁的 [原文 N] 是那一版 `recovery.rs` 的行号；[改写] 是写法换过、判定没换的。
-//! spec 照 D23（journal 的角色与格式）已定项 14 与 D13（验证路线）已定项 1 的 M2 那一行写。
-//! 读盘才判得出的两件（反向链、点名单元）由调用方给两个回调，核走到那一步才调，读盘次数与次序同原文。
+//! 判定照提交 fcc4d340 的 `crates/singlefs-core/src/recovery.rs` 里 `replay_journal` 搬过来。
+//! spec 照 D23（journal 的角色与格式）已定项 4、7、8、14、18、22 与 D13（验证路线）已定项 1 的 M2 那一行写（`replay_follows_the_prefix_rules`）：
+//! 施加哪一截由逐条判定（链首与 jsn 严格连续、反向链、读者规则、提交标记、末条之后同一次发布、点名单元）与在飞上限一起定，
+//! 报告的五个数与两种报错各有一句。
+//! 读盘才判得出的两件（反向链、点名单元）由调用方给两个回调，核走到那一步才调，读盘次数与次序同原文；
+//! spec 只说「核据以接受或断链的那一次回调交回了什么」，回调自己判得对不对不在 Verus 里。
 //! 平时的 stable 编译用 crates.io 的 `vstd` 与 `verus_builtin_macros` 把 `verus!` 擦成普通 Rust；
 //! 验的时候 Verus 用它自带的那一份，两边钉同一个发布（`Cargo.toml` 的 vstd 版本与 Verus 0.2026.09.20.aef82ed）。
 //! Verus 的必红世界登记在同目录的 `verus-mutations.tsv`。
@@ -16,16 +18,46 @@
 verus! {
 
 // ===== 核的数据 =====
-// JournalRecord（singlefs-core 的 journal.rs）里取前缀用得到的字段；枚举与 newtype 摊成原始类型。
-pub struct CoreRecord<Segment> {
-    pub instance: u32,  // InstanceGeneration.0
-    pub counter: u64,
-    pub checkpoint_txg: u64,  // CheckpointTxg.0
-    pub transaction: u64,
+// 核不依赖 singlefs-core 与 singlefs-format（Verus 只编这一个文件），这几个类型与 singlefs-core 里的同名类型是同一个概念，
+// 胶水（recovery.rs 的 replay_journal）逐个换过来。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct InstanceGeneration(pub u32);
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct JournalCounter(pub u64);
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CheckpointTxg(pub u64);
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct TransactionNumber(pub u64);
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct JournalRecordOrdinalWithinPublish(pub u32);
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum JournalRecordPlaceInPublish {
+    LastRecordOfThePublish,
+    MoreRecordsOfThePublishFollow,
+}
+
+// 关掉点名验证只供测试强制进入（.claude/rules/fs-design.md 五条硬要求第 2 条）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum NamedUnitVerification {
+    Verify,
+    Skip,
+}
+
+pub struct CoreRecord<RootSource> {
+    pub instance: InstanceGeneration,
+    pub counter: JournalCounter,
+    pub checkpoint_txg: CheckpointTxg,
+    pub transaction: TransactionNumber,
     pub is_commit: bool,
-    pub ordinal_within_publish: u32,  // JournalRecordOrdinalWithinPublish.0
-    pub ends_its_publish: bool,  // place_in_publish == LastRecordOfThePublish
-    pub new_root_segment: Segment,  // 施加时换进根里的那几个字段（原文 3216–3222）
+    pub ordinal_within_publish: JournalRecordOrdinalWithinPublish,
+    pub place_in_publish: JournalRecordPlaceInPublish,
+    // 核不看它：施加了一次发布之后把那次发布末条的这一项原样交回，调用方照它拼新根。
+    pub new_root_source: RootSource,
 }
 
 pub struct CoreReport {
@@ -33,47 +65,408 @@
     pub prefix_applied: usize,
     pub verification_passed: usize,
     pub verification_failed: usize,
-    pub maximum_applied_transaction: u64,
+    pub maximum_applied_transaction: TransactionNumber,
 }
 
 pub enum CoreFailure {
-    RootPublishCarriesMoreThanOneLastRecordFlag,
+    RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided { counters: Vec<JournalCounter> },
     ChosenRootCheckpointTxgHasNoSuccessor,
 }
 
-pub struct CoreOutcome<Segment> {
+pub struct CoreOutcome<RootSource> {
     pub report: CoreReport,
-    pub rebuilt: Segment,
-    pub applied: Ghost<Seq<int>>,  // 施加了的记录在入参切片里的下标，幽灵量，编译时擦掉
+    pub root_source_after_replay: RootSource,
+    // 幽灵量，编译时擦掉。admitted：逐条判定都过了的记录在入参切片里的下标；applied：其中施加了的那一截。
+    pub admitted: Ghost<Seq<int>>,
+    pub applied: Ghost<Seq<int>>,
 }
 
 // ===== spec（可信：照条款文字写，没有别的东西核它）=====
+pub open spec fn is_last_record_of_its_publish<RootSource>(record: CoreRecord<RootSource>) -> bool {
+    record.place_in_publish is LastRecordOfThePublish
+}
+
+pub open spec fn key_is_below<RootSource>(
+    lower: CoreRecord<RootSource>,
+    upper: CoreRecord<RootSource>,
+) -> bool {
+    lower.instance.0 < upper.instance.0 || (lower.instance == upper.instance && lower.counter.0
+        < upper.counter.0)
+}
+
+// 入参的次序：按 (实例代号, 计数器) 严格升序，与 BTreeMap<(InstanceGeneration, u64), _> 的键序相同。
+pub open spec fn records_ascend_by_instance_and_counter<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+) -> bool {
+    forall|earlier: int, later: int|
+        0 <= earlier < later < records.len() ==> #[trigger] key_is_below(
+            records[earlier],
+            records[later],
+        )
+}
+
+// 计数器是盘上 6 字节读来的；加一不溢出（已定项 9 的 48 位）。
+pub open spec fn every_counter_has_a_successor<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+) -> bool {
+    forall|record_index: int|
+        0 <= record_index < records.len() ==> (#[trigger] records[record_index]).counter.0
+            < u64::MAX
+}
+
 // 已定项 14 定案句：只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录；注 1：实例代号与所选根不同即停。
-pub open spec fn above_the_chosen_root<Segment>(
-    record: CoreRecord<Segment>,
-    root_instance: u32,
-    root_checkpoint_txg: u64,
+pub open spec fn above_the_chosen_root<RootSource>(
+    record: CoreRecord<RootSource>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+) -> bool {
+    record.instance == root_instance && record.checkpoint_txg.0 > root_checkpoint_txg.0
+}
+
+// 切片里 [0, end) 之间在所选根之上的记录（候选）的下标，按切片次序。
+pub open spec fn candidates_before<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    end: int,
+) -> Seq<int>
+    decreases end,
+{
+    if end <= 0 {
+        Seq::empty()
+    } else {
+        let earlier = candidates_before(records, root_instance, root_checkpoint_txg, end - 1);
+        if above_the_chosen_root(records[end - 1], root_instance, root_checkpoint_txg) {
+            earlier.push(end - 1)
+        } else {
+            earlier
+        }
+    }
+}
+
+// 已定项 14 注 1（读法乙）：所选根那次发布（同实例、同 checkpoint_txg）读得出的记录里带末条标志的那几条的计数器。
+pub open spec fn last_record_flag_counters_of_the_chosen_roots_publish<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    end: int,
+) -> Seq<JournalCounter>
+    decreases end,
+{
+    if end <= 0 {
+        Seq::empty()
+    } else {
+        let earlier = last_record_flag_counters_of_the_chosen_roots_publish(
+            records,
+            root_instance,
+            root_checkpoint_txg,
+            end - 1,
+        );
+        let record = records[end - 1];
+        if record.instance == root_instance && record.checkpoint_txg == root_checkpoint_txg
+            && is_last_record_of_its_publish(record) {
+            earlier.push(record.counter)
+        } else {
+            earlier
+        }
+    }
+}
+
+// 链首的锚点：带末条标志的恰好一条时是它；一条都没有就是「那条读不出」。
+pub open spec fn anchor_of_the_chain<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+) -> Option<JournalCounter> {
+    let flagged = last_record_flag_counters_of_the_chosen_roots_publish(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        records.len() as int,
+    );
+    if flagged.len() == 1 {
+        Some(flagged[0])
+    } else {
+        None
+    }
+}
+
+// 已定项 22 与已定项 14 注 1：jsn 严格连续（不等即止）；链首接在锚点之后，锚点读不出时链首的 txg = 所选根 + 1、本次发布内序号为 1。
+pub open spec fn continues_the_jsn_chain<RootSource>(
+    record: CoreRecord<RootSource>,
+    previous: Option<CoreRecord<RootSource>>,
+    anchor: Option<JournalCounter>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+) -> bool {
+    match previous {
+        Some(previous_record) => record.instance == previous_record.instance && record.counter.0
+            == previous_record.counter.0 + 1,
+        None => match anchor {
+            Some(anchor_counter) => record.instance == root_instance && record.counter.0
+                == anchor_counter.0 + 1,
+            None => record.checkpoint_txg.0 == root_checkpoint_txg.0 + 1
+                && record.ordinal_within_publish.0 == 1,
+        },
+    }
+}
+
+// 已定项 4 读者规则前三格（一次发布的首条序号是 1、一次发布之内不跳号、末条没到就换 txg 即缺了末条）与已定项 7
+// （提交标记没出现就换了事务号 ⇒ 那个事务没写完；末条不带提交标记 ⇒ 这次发布最后一个事务没写完）。
+pub open spec fn obeys_the_reader_rules_and_the_commit_markers<RootSource>(
+    record: CoreRecord<RootSource>,
+    previous: Option<CoreRecord<RootSource>>,
+) -> bool {
+    &&& match previous {
+        Some(previous_record) => if is_last_record_of_its_publish(previous_record) {
+            record.ordinal_within_publish.0 == 1
+        } else {
+            &&& record.checkpoint_txg == previous_record.checkpoint_txg
+            &&& record.ordinal_within_publish.0 == previous_record.ordinal_within_publish.0 + 1
+            &&& (previous_record.is_commit || record.transaction == previous_record.transaction)
+        },
+        None => record.ordinal_within_publish.0 == 1,
+    }
+    &&& (is_last_record_of_its_publish(record) ==> record.is_commit)
+}
+
+pub open spec fn is_a_later_record_of_the_same_instance<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    position: int,
+    other: int,
+) -> bool {
+    records[other].instance == records[position].instance && records[other].counter.0
+        > records[position].counter.0
+}
+
+// 已定项 4 读者规则第四格（C540）：同一 (实例代号, checkpoint_txg) 里这一条之后还有读得出的记录。
+// 「之后」按计数器、不要求紧接着；往后看到 txg 越过这一条的就不再看——同实例 txg 随计数器不降（K10）时与「之后任一条同 txg」逐态相同
+// （判决 research/prompts/m3-formal-r2-main-verification.md 第二节第 8 条）。
+pub open spec fn a_later_record_of_the_same_publish_is_readable<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    position: int,
+) -> bool {
+    exists|later: int|
+        0 <= later < records.len() && #[trigger] is_a_later_record_of_the_same_instance(
+            records,
+            position,
+            later,
+        ) && records[later].checkpoint_txg == records[position].checkpoint_txg && forall|
+            between: int,
+        |
+            0 <= between < records.len() && #[trigger] is_a_later_record_of_the_same_instance(
+                records,
+                position,
+                between,
+            ) && records[between].counter.0 < records[later].counter.0
+                ==> records[between].checkpoint_txg.0 <= records[position].checkpoint_txg.0
+}
+
+// 已定项 8 与 I-8.6：反向链对不上的那一条断。只说核据以接受或断链的那一次回调交回了什么。
+pub open spec fn back_chain_reported_holding<BackChain: Fn(usize) -> bool>(
+    back_chain_broken: BackChain,
+    position: int,
+) -> bool {
+    back_chain_broken.ensures((position as usize,), false)
+}
+
+pub open spec fn back_chain_reported_broken<BackChain: Fn(usize) -> bool>(
+    back_chain_broken: BackChain,
+    position: int,
+) -> bool {
+    back_chain_broken.ensures((position as usize,), true)
+}
+
+// 已定项 14 五条口径第四条：一个点名单元的两份都读得出、校验和都对才施加。
+pub open spec fn named_units_reported_verified<NamedUnits: Fn(usize) -> bool>(
+    named_unit_verification: NamedUnitVerification,
+    named_units_verified: NamedUnits,
+    position: int,
+) -> bool {
+    match named_unit_verification {
+        NamedUnitVerification::Verify => named_units_verified.ensures((position as usize,), true),
+        NamedUnitVerification::Skip => true,
+    }
+}
+
+pub open spec fn named_units_reported_unverified<NamedUnits: Fn(usize) -> bool>(
+    named_unit_verification: NamedUnitVerification,
+    named_units_verified: NamedUnits,
+    position: int,
+) -> bool {
+    named_unit_verification is Verify && named_units_verified.ensures((position as usize,), false)
+}
+
+#[verifier::opaque]
+pub open spec fn passes_every_check_before_the_named_units<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+>(
+    records: Seq<CoreRecord<RootSource>>,
+    position: int,
+    previous: Option<CoreRecord<RootSource>>,
+    anchor: Option<JournalCounter>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    back_chain_broken: BackChain,
+) -> bool {
+    &&& continues_the_jsn_chain(
+        records[position],
+        previous,
+        anchor,
+        root_instance,
+        root_checkpoint_txg,
+    )
+    &&& back_chain_reported_holding(back_chain_broken, position)
+    &&& obeys_the_reader_rules_and_the_commit_markers(records[position], previous)
+    &&& !(is_last_record_of_its_publish(records[position])
+        && a_later_record_of_the_same_publish_is_readable(records, position))
+}
+
+#[verifier::opaque]
+pub open spec fn fails_a_check_before_the_named_units<RootSource, BackChain: Fn(usize) -> bool>(
+    records: Seq<CoreRecord<RootSource>>,
+    position: int,
+    previous: Option<CoreRecord<RootSource>>,
+    anchor: Option<JournalCounter>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    back_chain_broken: BackChain,
+) -> bool {
+    ||| !continues_the_jsn_chain(
+        records[position],
+        previous,
+        anchor,
+        root_instance,
+        root_checkpoint_txg,
+    )
+    ||| back_chain_reported_broken(back_chain_broken, position)
+    ||| !obeys_the_reader_rules_and_the_commit_markers(records[position], previous)
+    ||| (is_last_record_of_its_publish(records[position])
+        && a_later_record_of_the_same_publish_is_readable(records, position))
+}
+
+pub open spec fn previous_in_the_chain<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    admitted: Seq<int>,
+    position_in_chain: int,
+) -> Option<CoreRecord<RootSource>> {
+    if position_in_chain == 0 {
+        None
+    } else {
+        Some(records[admitted[position_in_chain - 1]])
+    }
+}
+
+#[verifier::opaque]
+pub open spec fn admitted_record_passed_every_check<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    records: Seq<CoreRecord<RootSource>>,
+    admitted: Seq<int>,
+    position_in_chain: int,
+    anchor: Option<JournalCounter>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+) -> bool {
+    &&& passes_every_check_before_the_named_units(
+        records,
+        admitted[position_in_chain],
+        previous_in_the_chain(records, admitted, position_in_chain),
+        anchor,
+        root_instance,
+        root_checkpoint_txg,
+        back_chain_broken,
+    )
+    &&& named_units_reported_verified(
+        named_unit_verification,
+        named_units_verified,
+        admitted[position_in_chain],
+    )
+}
+
+// 下一个候选 next 让这一串停下：到了在飞上限（已定项 18），或它有一判不过——点名单元那一判不过时 verification_failed 记 1。
+pub open spec fn the_candidate_stops_the_chain<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    records: Seq<CoreRecord<RootSource>>,
+    admitted: Seq<int>,
+    next: int,
+    in_flight_limit: usize,
+    anchor: Option<JournalCounter>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    verification_failed: usize,
+) -> bool {
+    let previous = previous_in_the_chain(records, admitted, admitted.len() as int);
+    if admitted.len() >= in_flight_limit {
+        verification_failed == 0
+    } else {
+        ||| verification_failed == 0 && fails_a_check_before_the_named_units(
+            records,
+            next,
+            previous,
+            anchor,
+            root_instance,
+            root_checkpoint_txg,
+            back_chain_broken,
+        )
+        ||| verification_failed == 1 && passes_every_check_before_the_named_units(
+            records,
+            next,
+            previous,
+            anchor,
+            root_instance,
+            root_checkpoint_txg,
+            back_chain_broken,
+        ) && named_units_reported_unverified(named_unit_verification, named_units_verified, next)
+    }
+}
+
+// 已定项 14 第五条：施加的单位是一次发布——施加的是过了逐条判定的那一串里到最后一个末条为止的那一截。
+pub open spec fn applied_is_the_chain_cut_at_its_last_publish_end<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    admitted: Seq<int>,
+    applied: Seq<int>,
 ) -> bool {
-    record.instance == root_instance && record.checkpoint_txg > root_checkpoint_txg
+    &&& applied.len() <= admitted.len()
+    &&& forall|position: int|
+        0 <= position < applied.len() ==> #[trigger] applied[position] == admitted[position]
+    &&& (applied.len() == 0 || is_last_record_of_its_publish(records[applied.last()]))
+    &&& forall|position_in_chain: int|
+        applied.len() <= position_in_chain < admitted.len() ==> !is_last_record_of_its_publish(
+            records[#[trigger] admitted[position_in_chain]],
+        )
 }
 
 // 一次整的发布：checkpoint_txg 相同、序号依次 1..k、只有第 k 条带末条标志（已定项 4 / 14 第五条 / 17）。
-pub open spec fn is_whole_publish<Segment>(publish: Seq<CoreRecord<Segment>>) -> bool {
+pub open spec fn is_whole_publish<RootSource>(publish: Seq<CoreRecord<RootSource>>) -> bool {
     &&& publish.len() >= 1
     &&& forall|position: int|
         0 <= position < publish.len() ==> (#[trigger] publish[position]).checkpoint_txg
             == publish[0].checkpoint_txg
     &&& forall|position: int|
-        0 <= position < publish.len() ==> (#[trigger] publish[position]).ordinal_within_publish as int
+        0 <= position < publish.len() ==> (#[trigger] publish[position]).ordinal_within_publish.0 as int
             == position + 1
     &&& forall|position: int|
-        0 <= position < publish.len() ==> ((#[trigger] publish[position]).ends_its_publish
-            <==> position == publish.len() - 1)
+        0 <= position < publish.len() ==> (is_last_record_of_its_publish(
+            #[trigger] publish[position],
+        ) <==> position == publish.len() - 1)
 }
 
 // 若干整次发布的拼接（M2：重放结果等于整数个发布，不存在被前缀切开的发布）。
-pub open spec fn is_concatenation_of_whole_publishes<Segment>(
-    applied: Seq<CoreRecord<Segment>>,
+pub open spec fn is_concatenation_of_whole_publishes<RootSource>(
+    applied: Seq<CoreRecord<RootSource>>,
 ) -> bool
     decreases applied.len(),
 {
@@ -83,19 +476,20 @@
         ) && is_concatenation_of_whole_publishes(applied.subrange(0, cut))
 }
 
-pub open spec fn records_at<Segment>(
-    records: Seq<CoreRecord<Segment>>,
+pub open spec fn records_at<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
     indices: Seq<int>,
-) -> Seq<CoreRecord<Segment>> {
+) -> Seq<CoreRecord<RootSource>> {
     Seq::new(indices.len(), |position: int| records[indices[position]])
 }
 
-pub open spec fn m2_holds<Segment>(
-    root_instance: u32,
-    root_checkpoint_txg: u64,
-    root_segment: Segment,
-    records: Seq<CoreRecord<Segment>>,
-    outcome: CoreOutcome<Segment>,
+// D13 已定项 1 的 M2：施加的是若干整次发布、都在所选根之上；报告的条数与交回的根跟着施加的走。
+pub open spec fn applied_records_are_whole_publishes_above_the_chosen_root<RootSource>(
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    outcome: CoreOutcome<RootSource>,
 ) -> bool {
     let applied = outcome.applied@;
     &&& forall|position: int|
@@ -110,157 +504,798 @@
         )
     &&& is_concatenation_of_whole_publishes(records_at(records, applied))
     &&& outcome.report.prefix_applied == applied.len()
-    &&& outcome.rebuilt == if applied.len() == 0 {
-        root_segment
+    &&& outcome.root_source_after_replay == if applied.len() == 0 {
+        chosen_root_source
+    } else {
+        records[applied.last()].new_root_source
+    }
+}
+
+// 报告的最大事务号（可写挂载算 W 的输入，已定项 14 注 4）：施加的记录里最大的那个，一条都没施加时 0。
+pub open spec fn maximum_transaction_is_the_largest_applied<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    applied: Seq<int>,
+    maximum: TransactionNumber,
+) -> bool {
+    &&& forall|position: int|
+        0 <= position < applied.len() ==> (#[trigger] records[applied[position]]).transaction.0
+            <= maximum.0
+    &&& (maximum.0 == 0 || exists|position: int|
+        0 <= position < applied.len() && (#[trigger] records[applied[position]]).transaction
+            == maximum)
+}
+
+pub open spec fn outcome_follows_the_prefix_rules<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    outcome: CoreOutcome<RootSource>,
+) -> bool {
+    let candidates = candidates_before(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        records.len() as int,
+    );
+    let anchor = anchor_of_the_chain(records, root_instance, root_checkpoint_txg);
+    let admitted = outcome.admitted@;
+    let applied = outcome.applied@;
+    // 过了逐条判定的是候选从头起的一段（不跳过候选）、不超过在飞上限
+    &&& admitted.len() <= candidates.len()
+    &&& forall|position_in_chain: int|
+        0 <= position_in_chain < admitted.len() ==> #[trigger] admitted[position_in_chain]
+            == candidates[position_in_chain]
+    &&& admitted.len() <= in_flight_limit
+    &&& forall|position_in_chain: int|
+        0 <= position_in_chain < admitted.len() ==> #[trigger] admitted_record_passed_every_check(
+            records,
+            admitted,
+            position_in_chain,
+            anchor,
+            root_instance,
+            root_checkpoint_txg,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+        )
+    // 停在哪有理由：候选取完，或下一个候选让它停下
+    &&& if admitted.len() == candidates.len() {
+        outcome.report.verification_failed == 0
     } else {
-        records[applied.last()].new_root_segment
+        the_candidate_stops_the_chain(
+            records,
+            admitted,
+            candidates[admitted.len() as int],
+            in_flight_limit,
+            anchor,
+            root_instance,
+            root_checkpoint_txg,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            outcome.report.verification_failed,
+        )
+    }
+    &&& applied_is_the_chain_cut_at_its_last_publish_end(records, admitted, applied)
+    &&& applied_records_are_whole_publishes_above_the_chosen_root(
+        root_instance,
+        root_checkpoint_txg,
+        chosen_root_source,
+        records,
+        outcome,
+    )
+    &&& outcome.report.above_water == candidates.len()
+    &&& outcome.report.verification_passed == match named_unit_verification {
+        NamedUnitVerification::Verify => admitted.len(),
+        NamedUnitVerification::Skip => 0,
+    }
+    &&& maximum_transaction_is_the_largest_applied(
+        records,
+        applied,
+        outcome.report.maximum_applied_transaction,
+    )
+}
+
+// replay_core 的整个结论：两种报错各在什么时候报，不报错时施加的是哪一截、报告的每个数是什么。
+pub open spec fn replay_follows_the_prefix_rules<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    chosen_root_source: RootSource,
+    records: Seq<CoreRecord<RootSource>>,
+    in_flight_limit: usize,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
+) -> bool {
+    let flagged = last_record_flag_counters_of_the_chosen_roots_publish(
+        records,
+        root_instance,
+        root_checkpoint_txg,
+        records.len() as int,
+    );
+    match replayed {
+        // 带末条标志的多于一条：锚点认哪一条条款没写，在施加任何一条之前停下（已定项 14「这一版的失败处置」）。
+        Err(
+            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                counters,
+            },
+        ) => flagged.len() >= 2 && counters@ == flagged,
+        // 所选根的 checkpoint_txg 是盘上读来的 8 字节，最大值时接在它后面的发布取不到 txg（审阅第 36 条）。
+        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => flagged.len() < 2
+            && root_checkpoint_txg.0 == u64::MAX,
+        Ok(outcome) => flagged.len() < 2 && root_checkpoint_txg.0 < u64::MAX
+            && outcome_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            outcome,
+        ),
     }
 }
 
-// ===== 证明用的记法（不可信件：写错了只会证不出来）=====
+// ===== 证明用的记法与引理（不可信件：写错了只会证不出来）=====
 pub open spec fn as_ints(indices: Seq<usize>) -> Seq<int> {
     Seq::new(indices.len(), |position: int| indices[position] as int)
 }
 
 // 末条还没到的那一段：同 txg、序号 1..n、都不带末条标志。
-pub open spec fn is_open_publish<Segment>(publish: Seq<CoreRecord<Segment>>) -> bool {
+pub open spec fn is_open_publish<RootSource>(publish: Seq<CoreRecord<RootSource>>) -> bool {
     &&& forall|position: int|
         0 <= position < publish.len() ==> (#[trigger] publish[position]).checkpoint_txg
             == publish[0].checkpoint_txg
     &&& forall|position: int|
-        0 <= position < publish.len() ==> (#[trigger] publish[position]).ordinal_within_publish as int
+        0 <= position < publish.len() ==> (#[trigger] publish[position]).ordinal_within_publish.0 as int
             == position + 1
     &&& forall|position: int|
-        0 <= position < publish.len() ==> !(#[trigger] publish[position]).ends_its_publish
+        0 <= position < publish.len() ==> !is_last_record_of_its_publish(
+            #[trigger] publish[position],
+        )
+}
+
+// 「下一个该接的 jsn」：这一串还空着时由锚点给，否则是最后一条的下一号。
+pub open spec fn expected_next_after<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    admitted: Seq<int>,
+    anchor: Option<JournalCounter>,
+    root_instance: InstanceGeneration,
+) -> Option<(InstanceGeneration, JournalCounter)> {
+    if admitted.len() == 0 {
+        match anchor {
+            Some(anchor_counter) => Some(
+                (root_instance, JournalCounter((anchor_counter.0 + 1) as u64)),
+            ),
+            None => None,
+        }
+    } else {
+        let last = records[admitted.last()];
+        Some((last.instance, JournalCounter((last.counter.0 + 1) as u64)))
+    }
+}
+
+proof fn lemma_candidates_lie_below_their_end_in_ascending_order<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    end: int,
+)
+    requires
+        0 <= end <= records.len(),
+    ensures
+        ({
+            let candidates = candidates_before(records, root_instance, root_checkpoint_txg, end);
+            &&& candidates.len() <= end
+            &&& forall|position: int|
+                0 <= position < candidates.len() ==> 0 <= #[trigger] candidates[position] < end
+                    && above_the_chosen_root(
+                    records[candidates[position]],
+                    root_instance,
+                    root_checkpoint_txg,
+                )
+            &&& forall|earlier: int, later: int|
+                0 <= earlier < later < candidates.len() ==> candidates[earlier] < candidates[later]
+        }),
+    decreases end,
+{
+    if end > 0 {
+        lemma_candidates_lie_below_their_end_in_ascending_order(
+            records,
+            root_instance,
+            root_checkpoint_txg,
+            end - 1,
+        );
+    }
+}
+
+proof fn lemma_candidates_before_a_shorter_end_are_a_prefix<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    shorter: int,
+    longer: int,
+)
+    requires
+        0 <= shorter <= longer <= records.len(),
+    ensures
+        ({
+            let fewer = candidates_before(records, root_instance, root_checkpoint_txg, shorter);
+            let more = candidates_before(records, root_instance, root_checkpoint_txg, longer);
+            &&& fewer.len() <= more.len()
+            &&& forall|position: int|
+                0 <= position < fewer.len() ==> #[trigger] fewer[position] == more[position]
+        }),
+    decreases longer - shorter,
+{
+    if shorter < longer {
+        lemma_candidates_before_a_shorter_end_are_a_prefix(
+            records,
+            root_instance,
+            root_checkpoint_txg,
+            shorter,
+            longer - 1,
+        );
+    }
+}
+
+// 同一实例里切片次序就是计数器次序（入参按 (实例代号, 计数器) 严格升序）。
+proof fn lemma_slice_order_within_an_instance_follows_the_counter<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    first: int,
+    second: int,
+)
+    requires
+        records_ascend_by_instance_and_counter(records),
+        0 <= first < records.len(),
+        0 <= second < records.len(),
+        records[first].instance == records[second].instance,
+    ensures
+        first < second <==> records[first].counter.0 < records[second].counter.0,
+{
+    if first < second {
+        assert(key_is_below(records[first], records[second]));
+    } else if second < first {
+        assert(key_is_below(records[second], records[first]));
+    }
+}
+
+// 接受了下一个候选之后，已接受的每一条照旧过了逐条判定，新的一条也过了。
+proof fn lemma_admitting_the_candidate_keeps_every_link<
+    RootSource,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    records: Seq<CoreRecord<RootSource>>,
+    admitted_before: Seq<int>,
+    next: int,
+    anchor: Option<JournalCounter>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: BackChain,
+    named_units_verified: NamedUnits,
+)
+    requires
+        forall|position_in_chain: int|
+            0 <= position_in_chain < admitted_before.len() ==> #[trigger] admitted_record_passed_every_check(
+                records,
+                admitted_before,
+                position_in_chain,
+                anchor,
+                root_instance,
+                root_checkpoint_txg,
+                named_unit_verification,
+                back_chain_broken,
+                named_units_verified,
+            ),
+        passes_every_check_before_the_named_units(
+            records,
+            next,
+            previous_in_the_chain(records, admitted_before, admitted_before.len() as int),
+            anchor,
+            root_instance,
+            root_checkpoint_txg,
+            back_chain_broken,
+        ),
+        named_units_reported_verified(named_unit_verification, named_units_verified, next),
+    ensures
+        forall|position_in_chain: int|
+            0 <= position_in_chain < admitted_before.len() + 1 ==> #[trigger] admitted_record_passed_every_check(
+                records,
+                admitted_before.push(next),
+                position_in_chain,
+                anchor,
+                root_instance,
+                root_checkpoint_txg,
+                named_unit_verification,
+                back_chain_broken,
+                named_units_verified,
+            ),
+{
+    reveal(admitted_record_passed_every_check);
+    let admitted = admitted_before.push(next);
+    assert forall|position_in_chain: int|
+        0 <= position_in_chain < admitted.len() implies #[trigger] admitted_record_passed_every_check(
+        records,
+        admitted,
+        position_in_chain,
+        anchor,
+        root_instance,
+        root_checkpoint_txg,
+        named_unit_verification,
+        back_chain_broken,
+        named_units_verified,
+    ) by {
+        if position_in_chain < admitted_before.len() {
+            assert(admitted[position_in_chain] == admitted_before[position_in_chain]);
+            assert(previous_in_the_chain(records, admitted, position_in_chain)
+                == previous_in_the_chain(records, admitted_before, position_in_chain));
+            assert(admitted_record_passed_every_check(
+                records,
+                admitted_before,
+                position_in_chain,
+                anchor,
+                root_instance,
+                root_checkpoint_txg,
+                named_unit_verification,
+                back_chain_broken,
+                named_units_verified,
+            ));
+        } else {
+            assert(previous_in_the_chain(records, admitted, position_in_chain)
+                == previous_in_the_chain(records, admitted_before, admitted_before.len() as int));
+        }
+    }
+}
+
+// 接受了下一个候选之后，攒着的这次发布仍是「同 txg、序号 1..n」，除新的一条外都不带末条标志，攒着的仍是这一串在已施加之后的尾巴。
+proof fn lemma_admitting_the_candidate_extends_the_open_publish<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    open_before: Seq<usize>,
+    index: usize,
+    admitted_before: Seq<int>,
+    applied: Seq<int>,
+)
+    requires
+        0 <= index < records.len(),
+        forall|position: int|
+            0 <= position < open_before.len() ==> (#[trigger] open_before[position]) < records.len(),
+        is_open_publish(records_at(records, as_ints(open_before))),
+        as_ints(open_before) == admitted_before.subrange(
+            applied.len() as int,
+            admitted_before.len() as int,
+        ),
+        applied_is_the_chain_cut_at_its_last_publish_end(records, admitted_before, applied),
+        obeys_the_reader_rules_and_the_commit_markers(
+            records[index as int],
+            previous_in_the_chain(records, admitted_before, admitted_before.len() as int),
+        ),
+    ensures
+        as_ints(open_before.push(index)) == admitted_before.push(index as int).subrange(
+            applied.len() as int,
+            admitted_before.len() as int + 1,
+        ),
+        ({
+            let after = records_at(records, as_ints(open_before.push(index)));
+            &&& forall|position: int|
+                0 <= position < after.len() ==> (#[trigger] after[position]).checkpoint_txg
+                    == after[0].checkpoint_txg && after[position].ordinal_within_publish.0 as int
+                    == position + 1
+            &&& forall|position: int|
+                0 <= position < open_before.len() ==> !is_last_record_of_its_publish(
+                    #[trigger] after[position],
+                )
+            &&& after[after.len() - 1] == records[index as int]
+            &&& is_last_record_of_its_publish(records[index as int]) ==> is_whole_publish(after)
+            &&& !is_last_record_of_its_publish(records[index as int]) ==> is_open_publish(after)
+        }),
+{
+    let open = open_before.push(index);
+    let admitted = admitted_before.push(index as int);
+    assert forall|position: int|
+        0 <= position < as_ints(open).len() implies #[trigger] as_ints(open)[position]
+        == admitted.subrange(applied.len() as int, admitted.len() as int)[position] by {
+        if position < open_before.len() {
+            assert(as_ints(open_before)[position] == admitted_before.subrange(
+                applied.len() as int,
+                admitted_before.len() as int,
+            )[position]);
+        }
+    }
+    assert(as_ints(open) =~= admitted.subrange(applied.len() as int, admitted.len() as int));
+    if open_before.len() > 0 {
+        assert(as_ints(open_before).last() == admitted_before.last());
+    } else if admitted_before.len() > 0 {
+        assert(applied.len() == admitted_before.len());
+        assert(applied.last() == admitted_before.last());
+    }
+    let before = records_at(records, as_ints(open_before));
+    let after = records_at(records, as_ints(open));
+    assert forall|position: int| 0 <= position < after.len() implies (
+    #[trigger] after[position]).checkpoint_txg == after[0].checkpoint_txg
+        && after[position].ordinal_within_publish.0 as int == position + 1 by {
+        if position < open_before.len() {
+            assert(after[position] == before[position]);
+            assert(after[0] == before[0]);
+        } else {
+            assert(after[position] == records[index as int]);
+            if open_before.len() > 0 {
+                assert(before[open_before.len() - 1] == records[open_before[open_before.len()
+                    - 1] as int]);
+                assert(after[0] == before[0]);
+            }
+        }
+    }
+    assert forall|position: int|
+        0 <= position < open_before.len() implies !is_last_record_of_its_publish(
+        #[trigger] after[position],
+    ) by {
+        assert(after[position] == before[position]);
+    }
+    assert(after[after.len() - 1] == records[index as int]);
+}
+
+// 已施加的是这一串的开头、攒着的是其余的尾巴 ⇒ 两段接起来就是整串。
+proof fn lemma_the_applied_head_and_the_open_tail_make_up_the_chain(
+    admitted: Seq<int>,
+    applied: Seq<int>,
+    open_tail: Seq<int>,
+)
+    requires
+        applied.len() <= admitted.len(),
+        forall|position: int|
+            0 <= position < applied.len() ==> #[trigger] applied[position] == admitted[position],
+        open_tail == admitted.subrange(applied.len() as int, admitted.len() as int),
+    ensures
+        applied + open_tail == admitted,
+{
+    assert(applied + open_tail =~= admitted);
+}
+
+proof fn lemma_appending_a_whole_publish_keeps_a_concatenation<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    applied_before: Seq<int>,
+    this_publish: Seq<int>,
+)
+    requires
+        is_concatenation_of_whole_publishes(records_at(records, applied_before)),
+        is_whole_publish(records_at(records, this_publish)),
+    ensures
+        is_concatenation_of_whole_publishes(records_at(records, applied_before + this_publish)),
+{
+    let before = records_at(records, applied_before);
+    let publish = records_at(records, this_publish);
+    let after = records_at(records, applied_before + this_publish);
+    assert(after =~= before + publish);
+    assert(after.subrange(applied_before.len() as int, after.len() as int) =~= publish);
+    assert(after.subrange(0, applied_before.len() as int) =~= before);
+    assert(is_whole_publish(after.subrange(applied_before.len() as int, after.len() as int)));
+}
+
+proof fn lemma_the_maximum_over_the_appended_publish<RootSource>(
+    records: Seq<CoreRecord<RootSource>>,
+    applied_before: Seq<int>,
+    records_of_the_open_publish: Seq<usize>,
+    maximum_before: TransactionNumber,
+    maximum: TransactionNumber,
+)
+    requires
+        maximum_transaction_is_the_largest_applied(records, applied_before, maximum_before),
+        maximum.0 >= maximum_before.0,
+        forall|position: int|
+            0 <= position < records_of_the_open_publish.len()
+                ==> records[#[trigger] records_of_the_open_publish[position] as int].transaction.0
+                <= maximum.0,
+        maximum == maximum_before || exists|position: int|
+            0 <= position < records_of_the_open_publish.len()
+                && records[#[trigger] records_of_the_open_publish[position] as int].transaction
+                == maximum,
+    ensures
+        maximum_transaction_is_the_largest_applied(
+            records,
+            applied_before + as_ints(records_of_the_open_publish),
+            maximum,
+        ),
+{
+    let this_publish = as_ints(records_of_the_open_publish);
+    let applied = applied_before + this_publish;
+    assert forall|position: int|
+        0 <= position < applied.len() implies (#[trigger] records[applied[position]]).transaction.0
+        <= maximum.0 by {
+        if position >= applied_before.len() {
+            assert(applied[position] == this_publish[position - applied_before.len()]);
+            assert(records[records_of_the_open_publish[position
+                - applied_before.len()] as int].transaction.0 <= maximum.0);
+        }
+    }
+    if maximum.0 != 0 {
+        if maximum == maximum_before {
+            let witness = choose|position: int|
+                0 <= position < applied_before.len() && (#[trigger] records[applied_before[position]]).transaction
+                    == maximum_before;
+            assert(applied[witness] == applied_before[witness]);
+        } else {
+            let open_witness = choose|open_position: int|
+                0 <= open_position < records_of_the_open_publish.len()
+                    && records[#[trigger] records_of_the_open_publish[open_position] as int].transaction
+                    == maximum;
+            assert(applied[applied_before.len() + open_witness]
+                == records_of_the_open_publish[open_witness] as int);
+        }
+    }
 }
 
 // ===== 纯函数核 =====
-// [原文 2930–2937] record_ends_its_publish：记录标志位 0（已定项 14 第五条、已定项 17）。[改写] 原文 match 两臂枚举，这里读布尔字段。
-fn record_ends_its_publish<Segment>(record: &CoreRecord<Segment>) -> (is_last_record: bool)
+// 记录标志位 0（已定项 14 第五条、已定项 17）。
+fn record_ends_its_publish<RootSource>(record: &CoreRecord<RootSource>) -> (is_last_record: bool)
     ensures
-        is_last_record == record.ends_its_publish,
+        is_last_record == is_last_record_of_its_publish(*record),
 {
-    record.ends_its_publish
+    match record.place_in_publish {
+        JournalRecordPlaceInPublish::LastRecordOfThePublish => true,
+        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,
+    }
 }
 
-// [原文 2939–2953] readable_record_of_the_same_publish_follows：同一 (实例代号, checkpoint_txg) 里这一条之后（计数器更大）
-// 还有没有读得出的记录。同一实例里计数器与 checkpoint_txg 一起往上走，所以按计数器升序看到 txg 越过这一条的就停。
-// [改写] 原文在 BTreeMap 上 range((instance, counter+1)..=(instance, u64::MAX))，这里从切片里这一条的下一个位置往后扫，
-// 靠「切片按 (实例代号, 计数器) 严格升序」才与原文同义；这一条没有写成前置条件，也没有证。
-fn readable_record_of_the_same_publish_follows<Segment>(
-    records: &[CoreRecord<Segment>],
+// 原文在 BTreeMap 上 range((instance, counter+1)..=(instance, u64::MAX))、take_while(txg <= 这一条的)、any(txg == 这一条的)；
+// 这里从切片里这一条的下一个位置往后扫，靠入参按 (实例代号, 计数器) 严格升序与原文同义（requires 写着，证明用到它）。
+fn readable_record_of_the_same_publish_follows<RootSource>(
+    records: &[CoreRecord<RootSource>],
     index: usize,
-) -> bool
+) -> (follows: bool)
     requires
         index < records.len(),
+        records_ascend_by_instance_and_counter(records@),
+    ensures
+        follows == a_later_record_of_the_same_publish_is_readable(records@, index as int),
 {
     let record = &records[index];
-    // [原文 2945–2947] checked_add(1) 取不到就 false
-    if record.counter == u64::MAX {
+    if record.counter.0 == u64::MAX {
         return false;
     }
-    let next_counter = record.counter + 1;
+    let next_counter = record.counter.0 + 1;
     let mut later_index = index + 1;
     while later_index < records.len()
         invariant
             index < later_index <= records.len(),
+            *record == records@[index as int],
+            records_ascend_by_instance_and_counter(records@),
+            next_counter == records@[index as int].counter.0 + 1,
+            forall|between: int|
+                index < between < later_index && #[trigger] is_a_later_record_of_the_same_instance(
+                    records@,
+                    index as int,
+                    between,
+                ) ==> records@[between].checkpoint_txg.0 < records@[index as int].checkpoint_txg.0,
         decreases records.len() - later_index,
     {
         let later = &records[later_index];
-        // [原文 2949] range 的上下界：同实例、计数器 ≥ next_counter
-        if later.instance == record.instance && later.counter >= next_counter {
-            // [原文 2951] take_while(later.checkpoint_txg <= record.checkpoint_txg)
-            if !(later.checkpoint_txg <= record.checkpoint_txg) {
+        if later.instance.0 == record.instance.0 && later.counter.0 >= next_counter {
+            if !(later.checkpoint_txg.0 <= record.checkpoint_txg.0) {
+                proof {
+                    assert forall|witness: int|
+                        0 <= witness < records@.len() && #[trigger] is_a_later_record_of_the_same_instance(
+                            records@,
+                            index as int,
+                            witness,
+                        ) && records@[witness].checkpoint_txg == records@[index as int].checkpoint_txg
+                            implies !(forall|between: int|
+                        0 <= between < records@.len() && #[trigger] is_a_later_record_of_the_same_instance(
+                            records@,
+                            index as int,
+                            between,
+                        ) && records@[between].counter.0 < records@[witness].counter.0
+                            ==> records@[between].checkpoint_txg.0
+                            <= records@[index as int].checkpoint_txg.0) by {
+                        assert(records@[later_index as int].instance == records@[index as int].instance);
+                        lemma_slice_order_within_an_instance_follows_the_counter(
+                            records@,
+                            index as int,
+                            witness,
+                        );
+                        lemma_slice_order_within_an_instance_follows_the_counter(
+                            records@,
+                            later_index as int,
+                            witness,
+                        );
+                        assert(is_a_later_record_of_the_same_instance(
+                            records@,
+                            index as int,
+                            later_index as int,
+                        ));
+                    }
+                }
                 return false;
             }
-            // [原文 2952] any(later.checkpoint_txg == record.checkpoint_txg)
-            if later.checkpoint_txg == record.checkpoint_txg {
+            if later.checkpoint_txg.0 == record.checkpoint_txg.0 {
+                proof {
+                    assert forall|between: int|
+                        0 <= between < records@.len() && #[trigger] is_a_later_record_of_the_same_instance(
+                            records@,
+                            index as int,
+                            between,
+                        ) && records@[between].counter.0 < records@[later_index as int].counter.0
+                            implies records@[between].checkpoint_txg.0
+                        <= records@[index as int].checkpoint_txg.0 by {
+                        lemma_slice_order_within_an_instance_follows_the_counter(
+                            records@,
+                            index as int,
+                            between,
+                        );
+                        assert(records@[later_index as int].instance == records@[index as int].instance);
+                        lemma_slice_order_within_an_instance_follows_the_counter(
+                            records@,
+                            between,
+                            later_index as int,
+                        );
+                    }
+                    assert(is_a_later_record_of_the_same_instance(
+                        records@,
+                        index as int,
+                        later_index as int,
+                    ));
+                }
                 return true;
             }
         }
         later_index += 1;
     }
+    proof {
+        assert forall|witness: int|
+            0 <= witness < records@.len() && #[trigger] is_a_later_record_of_the_same_instance(
+                records@,
+                index as int,
+                witness,
+            ) implies records@[witness].checkpoint_txg != records@[index as int].checkpoint_txg by {
+            lemma_slice_order_within_an_instance_follows_the_counter(records@, index as int, witness);
+        }
+    }
     false
 }
 
-// [原文 2955–2987] counter_of_the_last_record_the_root_covers：所选根覆盖的最后一条记录的 jsn 计数器（已定项 14 注 1，读法乙）——
-// 所选根那次发布读得出的几条里带末条标志的那一条；一条都不带 ⇒ Ok(None)；带标志的多于一条 ⇒ 认哪一条条款没写，报错。
-// [改写] 原文 filter + collect 成 Vec 再看 len() > 1、取 first()；这里一遍循环，第二条带标志的出现就报错。
-// 报错里的 counters 清单没带（核不产 Vec），由调用方再调原来那个函数拿。
-fn counter_of_the_last_record_the_root_covers<Segment>(
-    root_instance: u32,
-    root_checkpoint_txg: u64,
-    records: &[CoreRecord<Segment>],
-) -> (anchor: Result<Option<u64>, CoreFailure>)
-    requires
-        forall|record_index: int|
-            0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter
-                < u64::MAX,
+// 所选根覆盖的最后一条记录的 jsn 计数器（已定项 14 注 1，读法乙）：所选根那次发布读得出的几条里带末条标志的那一条；
+// 一条都不带 ⇒ Ok(None)；带标志的多于一条 ⇒ 认哪一条条款没写，报错并带出那几条的计数器。
+fn counter_of_the_last_record_the_root_covers<RootSource>(
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    records: &[CoreRecord<RootSource>],
+) -> (anchor: Result<Option<JournalCounter>, CoreFailure>)
+    requires
+        every_counter_has_a_successor(records@),
     ensures
-        anchor matches Ok(Some(counter)) ==> counter < u64::MAX,
+        ({
+            let flagged = last_record_flag_counters_of_the_chosen_roots_publish(
+                records@,
+                root_instance,
+                root_checkpoint_txg,
+                records@.len() as int,
+            );
+            match anchor {
+                Ok(found) => flagged.len() < 2 && found == anchor_of_the_chain(
+                    records@,
+                    root_instance,
+                    root_checkpoint_txg,
+                ),
+                Err(
+                    CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                        counters,
+                    },
+                ) => flagged.len() >= 2 && counters@ == flagged,
+                Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => false,
+            }
+        }),
+        anchor matches Ok(Some(counter)) ==> counter.0 < u64::MAX,
 {
-    let mut found: Option<u64> = None;
+    let mut counters_carrying_the_last_record_flag: Vec<JournalCounter> = Vec::new();
     let mut index = 0;
     while index < records.len()
         invariant
             index <= records.len(),
-            forall|record_index: int|
-                0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter
-                    < u64::MAX,
-            found matches Some(counter) ==> counter < u64::MAX,
+            every_counter_has_a_successor(records@),
+            counters_carrying_the_last_record_flag@
+                == last_record_flag_counters_of_the_chosen_roots_publish(
+                records@,
+                root_instance,
+                root_checkpoint_txg,
+                index as int,
+            ),
+            forall|position: int|
+                0 <= position < counters_carrying_the_last_record_flag@.len()
+                    ==> (#[trigger] counters_carrying_the_last_record_flag@[position]).0 < u64::MAX,
         decreases records.len() - index,
     {
         let record = &records[index];
-        // [原文 2970–2972]
-        if record.instance == root_instance && record.checkpoint_txg == root_checkpoint_txg
+        if record.instance.0 == root_instance.0 && record.checkpoint_txg.0 == root_checkpoint_txg.0
             && record_ends_its_publish(record) {
-            // [原文 2976–2984]
-            if found.is_some() {
-                return Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlag);
-            }
-            found = Some(record.counter);
+            counters_carrying_the_last_record_flag.push(record.counter);
         }
         index += 1;
     }
-    // [原文 2985–2986]
-    Ok(found)
+    if counters_carrying_the_last_record_flag.len() > 1 {
+        return Err(
+            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                counters: counters_carrying_the_last_record_flag,
+            },
+        );
+    }
+    if counters_carrying_the_last_record_flag.is_empty() {
+        Ok(None)
+    } else {
+        Ok(Some(counters_carrying_the_last_record_flag[0]))
+    }
 }
 
-// [原文 3074–3080] above 的筛选条件：前缀规则不跨实例边界（已定项 14 第 1 条），只有所选根自己那个实例、txg 比根大的记录是候选。
-// 数条数那一遍与主循环共用这一判，同原文只筛一次（crates/mutations.tsv 第 23、355 行改的就是它，两处一起变）。
-// [改写] 原文比元组 (record.instance, record.checkpoint_txg) > water，这里把字典序摊开写。
-fn is_above_the_chosen_root<Segment>(
-    record: &CoreRecord<Segment>,
-    root_instance: u32,
-    root_checkpoint_txg: u64,
+// 前缀规则不跨实例边界（已定项 14 第 1 条），只有所选根自己那个实例、txg 比根大的记录是候选。
+fn is_above_the_chosen_root<RootSource>(
+    record: &CoreRecord<RootSource>,
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
 ) -> (is_above: bool)
     ensures
         is_above == above_the_chosen_root(*record, root_instance, root_checkpoint_txg),
 {
-    record.instance == root_instance && (record.instance > root_instance || (record.instance
-        == root_instance && record.checkpoint_txg > root_checkpoint_txg))
+    record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0
 }
 
-// [原文 3204–3207] 攒着的这次发布里最大的事务号。[改写] 原文 iter().map().fold(.., u64::max)。
-fn maximum_transaction_of<Segment>(
-    records: &[CoreRecord<Segment>],
-    open: &[usize],
-    start: u64,
-) -> u64
+// 攒着的这次发布里最大的事务号，与 start 取大。
+fn maximum_transaction_of<RootSource>(
+    records: &[CoreRecord<RootSource>],
+    records_of_the_open_publish: &[usize],
+    start: TransactionNumber,
+) -> (maximum: TransactionNumber)
     requires
-        forall|position: int| 0 <= position < open@.len() ==> (#[trigger] open@[position]) < records.len(),
+        forall|position: int|
+            0 <= position < records_of_the_open_publish@.len()
+                ==> (#[trigger] records_of_the_open_publish@[position]) < records.len(),
+    ensures
+        maximum.0 >= start.0,
+        forall|position: int|
+            0 <= position < records_of_the_open_publish@.len()
+                ==> records@[#[trigger] records_of_the_open_publish@[position] as int].transaction.0
+                <= maximum.0,
+        maximum == start || exists|position: int|
+            0 <= position < records_of_the_open_publish@.len()
+                && records@[#[trigger] records_of_the_open_publish@[position] as int].transaction
+                == maximum,
 {
     let mut maximum = start;
     let mut position = 0;
-    while position < open.len()
+    while position < records_of_the_open_publish.len()
         invariant
-            position <= open@.len(),
+            position <= records_of_the_open_publish@.len(),
             forall|open_position: int|
-                0 <= open_position < open@.len() ==> (#[trigger] open@[open_position]) < records.len(),
-        decreases open@.len() - position,
+                0 <= open_position < records_of_the_open_publish@.len()
+                    ==> (#[trigger] records_of_the_open_publish@[open_position]) < records.len(),
+            maximum.0 >= start.0,
+            forall|open_position: int|
+                0 <= open_position < position
+                    ==> records@[#[trigger] records_of_the_open_publish@[open_position] as int].transaction.0
+                    <= maximum.0,
+            maximum == start || exists|open_position: int|
+                0 <= open_position < position
+                    && records@[#[trigger] records_of_the_open_publish@[open_position] as int].transaction
+                    == maximum,
+        decreases records_of_the_open_publish@.len() - position,
     {
-        let transaction = records[open[position]].transaction;
-        if transaction > maximum {
+        let transaction = records[records_of_the_open_publish[position]].transaction;
+        if transaction.0 > maximum.0 {
             maximum = transaction;
         }
         position += 1;
@@ -268,24 +1303,202 @@
     maximum
 }
 
-// [原文 3057–3229] replay_journal 去掉读盘之后的那一截。
-// records：全部自证过的记录，次序就是 BTreeMap<(InstanceGeneration, u64), _>::values() 的次序。
-// 前置条件只有一条：计数器不是 u64 的最大值（原文 3096、3158 的 counter + 1 在那个值上溢出）。没有要求排序，条数不设界。
+// 一个候选过不过逐条判定。次序与读盘次序同原文：链首与 jsn 连续、反向链回调、读者规则与提交标记、末条之后同一次发布、点名单元回调；
+// 前一判不过就不再往下判（也就不再调后面的回调）。
+enum CandidateVerdict {
+    Admitted,
+    BrokenBeforeTheNamedUnits,
+    BrokenAtTheNamedUnits,
+}
+
+#[allow(clippy::too_many_arguments, reason = "逐条判定要的输入各自独立；幽灵量五个只给证明用，收成结构体只会多一层没人验的名字")]
+fn judge_the_candidate<RootSource, BackChain: Fn(usize) -> bool, NamedUnits: Fn(usize) -> bool>(
+    records: &[CoreRecord<RootSource>],
+    index: usize,
+    records_of_the_open_publish: &[usize],
+    expected_next: Option<(InstanceGeneration, JournalCounter)>,
+    chain_start_txg_without_anchor: CheckpointTxg,
+    named_unit_verification: NamedUnitVerification,
+    back_chain_broken: &BackChain,
+    named_units_verified: &NamedUnits,
+    Ghost(admitted): Ghost<Seq<int>>,
+    Ghost(applied): Ghost<Seq<int>>,
+    Ghost(anchor): Ghost<Option<JournalCounter>>,
+    Ghost(root_instance): Ghost<InstanceGeneration>,
+    Ghost(root_checkpoint_txg): Ghost<CheckpointTxg>,
+) -> (verdict: CandidateVerdict)
+    requires
+        index < records.len(),
+        records_ascend_by_instance_and_counter(records@),
+        every_counter_has_a_successor(records@),
+        back_chain_broken.requires((index,)),
+        named_units_verified.requires((index,)),
+        root_checkpoint_txg.0 < u64::MAX,
+        chain_start_txg_without_anchor.0 == root_checkpoint_txg.0 + 1,
+        anchor matches Some(anchor_counter) ==> anchor_counter.0 < u64::MAX,
+        expected_next == expected_next_after(records@, admitted, anchor, root_instance),
+        forall|position: int|
+            0 <= position < admitted.len() ==> 0 <= #[trigger] admitted[position] < records.len(),
+        applied_is_the_chain_cut_at_its_last_publish_end(records@, admitted, applied),
+        as_ints(records_of_the_open_publish@) == admitted.subrange(
+            applied.len() as int,
+            admitted.len() as int,
+        ),
+    ensures
+        ({
+            let previous = previous_in_the_chain(records@, admitted, admitted.len() as int);
+            match verdict {
+                CandidateVerdict::Admitted => passes_every_check_before_the_named_units(
+                    records@,
+                    index as int,
+                    previous,
+                    anchor,
+                    root_instance,
+                    root_checkpoint_txg,
+                    *back_chain_broken,
+                ) && obeys_the_reader_rules_and_the_commit_markers(records@[index as int], previous)
+                    && named_units_reported_verified(
+                    named_unit_verification,
+                    *named_units_verified,
+                    index as int,
+                ),
+                CandidateVerdict::BrokenBeforeTheNamedUnits => fails_a_check_before_the_named_units(
+                    records@,
+                    index as int,
+                    previous,
+                    anchor,
+                    root_instance,
+                    root_checkpoint_txg,
+                    *back_chain_broken,
+                ),
+                CandidateVerdict::BrokenAtTheNamedUnits => passes_every_check_before_the_named_units(
+                    records@,
+                    index as int,
+                    previous,
+                    anchor,
+                    root_instance,
+                    root_checkpoint_txg,
+                    *back_chain_broken,
+                ) && named_units_reported_unverified(
+                    named_unit_verification,
+                    *named_units_verified,
+                    index as int,
+                ),
+            }
+        }),
+{
+    let record = &records[index];
+    let ghost previous = previous_in_the_chain(records@, admitted, admitted.len() as int);
+    proof {
+        reveal(passes_every_check_before_the_named_units);
+        reveal(fails_a_check_before_the_named_units);
+        if as_ints(records_of_the_open_publish@).len() > 0 {
+            assert(as_ints(records_of_the_open_publish@).last() == admitted.last());
+        } else if admitted.len() > 0 {
+            assert(applied.len() == admitted.len());
+            assert(applied.last() == admitted.last());
+        }
+    }
+    match expected_next {
+        Some(expected_key) => {
+            if record.instance.0 != expected_key.0.0 || record.counter.0 != expected_key.1.0 {
+                return CandidateVerdict::BrokenBeforeTheNamedUnits;
+            }
+        },
+        // 锚点读不出：链首只能是 txg = 根的 txg + 1、本次发布内序号为 1 的那条（三方第一轮 K4-b）。
+        // 序号那一判与下面「攒着的为空且序号不是 1」那一判冗余：走到这一支时攒着的一定是空的（下面这句由 Verus 证），
+        // 两判之间只有反向链回调，它只会断、不改状态。
+        None => {
+            proof {
+                assert(records_of_the_open_publish@.len() == 0);
+            }
+            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0
+                || record.ordinal_within_publish.0 != 1 {
+                return CandidateVerdict::BrokenBeforeTheNamedUnits;
+            }
+        },
+    }
+    // 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断；三态由回调交成布尔。
+    if back_chain_broken(index) {
+        return CandidateVerdict::BrokenBeforeTheNamedUnits;
+    }
+    if records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 {
+        return CandidateVerdict::BrokenBeforeTheNamedUnits;
+    }
+    proof {
+        if admitted.len() > 0 {
+            assert(records@[admitted.last()].counter.0 < u64::MAX);
+            assert(records@[index as int].instance == records@[admitted.last()].instance);
+        } else if anchor is Some {
+            assert(records@[index as int].instance == root_instance);
+        }
+        assert(continues_the_jsn_chain(
+            records@[index as int],
+            previous,
+            anchor,
+            root_instance,
+            root_checkpoint_txg,
+        ));
+    }
+    if !records_of_the_open_publish.is_empty() {
+        let previous_record_of_the_open_publish = &records[records_of_the_open_publish[records_of_the_open_publish.len()
+            - 1]];
+        proof {
+            assert(!is_last_record_of_its_publish(records@[admitted.last()]));
+        }
+        if record.checkpoint_txg.0 != previous_record_of_the_open_publish.checkpoint_txg.0 {
+            return CandidateVerdict::BrokenBeforeTheNamedUnits;
+        }
+        if u64::from(record.ordinal_within_publish.0) != u64::from(
+            previous_record_of_the_open_publish.ordinal_within_publish.0,
+        ) + 1 {
+            return CandidateVerdict::BrokenBeforeTheNamedUnits;
+        }
+        if !previous_record_of_the_open_publish.is_commit
+            && record.transaction.0 != previous_record_of_the_open_publish.transaction.0 {
+            return CandidateVerdict::BrokenBeforeTheNamedUnits;
+        }
+    }
+    if !record.is_commit && record_ends_its_publish(record) {
+        return CandidateVerdict::BrokenBeforeTheNamedUnits;
+    }
+    if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(
+        records,
+        index,
+    ) {
+        return CandidateVerdict::BrokenBeforeTheNamedUnits;
+    }
+    let all_verified = match named_unit_verification {
+        NamedUnitVerification::Verify => named_units_verified(index),
+        NamedUnitVerification::Skip => true,
+    };
+    if !all_verified {
+        return CandidateVerdict::BrokenAtTheNamedUnits;
+    }
+    CandidateVerdict::Admitted
+}
+
+// replay_journal 去掉读盘之后的那一截。
+// 前置条件：入参按 (实例代号, 计数器) 严格升序（「末条之后同一次发布还有没有记录」那一判靠它与原文同义）；
+// 计数器都不是 u64 的最大值（链上下一号要加一）；两个回调对每个下标都可调。条数不设界。
 #[allow(clippy::too_many_arguments, reason = "两个回调是读盘的两处，摊成参数比包一层结构体好对照原文")]
-pub fn replay_core<Segment: Copy, BackChain: Fn(usize) -> bool, NamedUnits: Fn(usize) -> bool>(
-    root_instance: u32,
-    root_checkpoint_txg: u64,
-    root_segment: Segment,
-    records: &[CoreRecord<Segment>],
+pub fn replay_core<
+    RootSource: Copy,
+    BackChain: Fn(usize) -> bool,
+    NamedUnits: Fn(usize) -> bool,
+>(
+    root_instance: InstanceGeneration,
+    root_checkpoint_txg: CheckpointTxg,
+    chosen_root_source: RootSource,
+    records: &[CoreRecord<RootSource>],
     in_flight_limit: usize,
-    verify_named_units: bool,
+    named_unit_verification: NamedUnitVerification,
     back_chain_broken: BackChain,
     named_units_verified: NamedUnits,
-) -> (replayed: Result<CoreOutcome<Segment>, CoreFailure>)
+) -> (replayed: Result<CoreOutcome<RootSource>, CoreFailure>)
     requires
-        forall|record_index: int|
-            0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter
-                < u64::MAX,
+        records_ascend_by_instance_and_counter(records@),
+        every_counter_has_a_successor(records@),
         forall|record_index: usize|
             record_index < records@.len() ==> #[trigger] back_chain_broken.requires((record_index,)),
         forall|record_index: usize|
@@ -293,45 +1506,46 @@
                 (record_index,),
             ),
     ensures
-        match replayed {
-            Ok(outcome) => m2_holds(
-                root_instance,
-                root_checkpoint_txg,
-                root_segment,
-                records@,
-                outcome,
-            ),
-            Err(_) => true,
-        },
+        replay_follows_the_prefix_rules(
+            root_instance,
+            root_checkpoint_txg,
+            chosen_root_source,
+            records@,
+            in_flight_limit,
+            named_unit_verification,
+            back_chain_broken,
+            named_units_verified,
+            replayed,
+        ),
 {
-    // [原文 3064–3071]（valid_records = records.len() 留给调用方）
     let mut report = CoreReport {
         above_water: 0,
         prefix_applied: 0,
         verification_passed: 0,
         verification_failed: 0,
-        maximum_applied_transaction: 0,
+        maximum_applied_transaction: TransactionNumber(0),
     };
-    // [原文 3073]
-    let mut rebuilt = root_segment;
-    // [原文 3074–3083] above 的条数。前缀规则不跨实例边界（已定项 14 第 1 条）：只有所选根自己那个实例的记录是候选。
-    // [改写] 原文 filter + collect + sort_by_key 出一个 Vec，这里不建 Vec：
-    // 这一遍只数条数，主循环里逐条再判同一个条件。sort_by_key（原文 3082）没抄：入参已是那个次序。
+    let mut root_source_after_replay = chosen_root_source;
+    // 这一遍只数条数，主循环里逐条再判同一个条件；原文 sort_by_key 没抄：入参已是那个次序。
     let mut counting_index = 0;
     while counting_index < records.len()
         invariant
             counting_index <= records.len(),
             report.above_water <= counting_index,
+            report.above_water == candidates_before(
+                records@,
+                root_instance,
+                root_checkpoint_txg,
+                counting_index as int,
+            ).len(),
         decreases records.len() - counting_index,
     {
         let record = &records[counting_index];
-        // [原文 3078–3080]
         if is_above_the_chosen_root(record, root_instance, root_checkpoint_txg) {
             report.above_water += 1;
         }
         counting_index += 1;
     }
-    // [原文 3086–3094] 链首锚点 = 所选根覆盖的最后一条：那次发布里带末条标志的那一条（已定项 14 注 1，读法乙；三方第一轮 K3）。
     let root_own_record_counter = match counter_of_the_last_record_the_root_covers(
         root_instance,
         root_checkpoint_txg,
@@ -342,36 +1556,36 @@
             return Err(failure);
         },
     };
-    // [原文 3095–3096] counter + 1 不溢出由前置条件（计数器不是最大值）保。
-    let mut expected_next: Option<(u32, u64)> = root_own_record_counter.map(
-        |counter: u64| -> (u32, u64)
-            requires
-                counter < u64::MAX,
-            { (root_instance, counter + 1) },
-    );
-    // [原文 3097–3106] 所选根的 checkpoint_txg 是盘上读来的 8 字节，可以是 u64 的最大值：加一溢出报损坏、不 panic（审阅第 36 条）。
-    if root_checkpoint_txg == u64::MAX {
+    let mut expected_next: Option<(InstanceGeneration, JournalCounter)> = None;
+    if let Some(counter) = root_own_record_counter {
+        expected_next = Some((root_instance, JournalCounter(counter.0 + 1)));
+    }
+    // 所选根的 checkpoint_txg 是盘上读来的 8 字节，可以是 u64 的最大值：加一溢出报损坏、不 panic（审阅第 36 条）。
+    if root_checkpoint_txg.0 == u64::MAX {
         return Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor);
     }
-    let chain_start_txg_without_anchor = root_checkpoint_txg + 1;
-    // [原文 3107–3108] 这次发布已经过了前四条、还没走到末条的记录。原文存 &JournalRecord，这里存下标
-    let mut open: Vec<usize> = Vec::new();
+    let chain_start_txg_without_anchor = CheckpointTxg(root_checkpoint_txg.0 + 1);
+    // 这次发布已经过了逐条判定、还没走到末条的记录的下标。
+    let mut records_of_the_open_publish: Vec<usize> = Vec::new();
+    let ghost mut admitted: Seq<int> = Seq::empty();
     let ghost mut applied: Seq<int> = Seq::empty();
-    // [原文 3109] take(in_flight_limit) 的计数
-    let mut taken: usize = 0;
+    let ghost anchor = anchor_of_the_chain(records@, root_instance, root_checkpoint_txg);
+    let mut records_taken_toward_the_in_flight_limit: usize = 0;
     let mut index = 0;
+    proof {
+        assert(expected_next == expected_next_after(records@, admitted, anchor, root_instance));
+    }
     while index < records.len()
         invariant_except_break
-            taken <= index,
-            report.verification_passed <= index,
+            records_taken_toward_the_in_flight_limit == admitted.len(),
             report.verification_failed == 0,
+            expected_next == expected_next_after(records@, admitted, anchor, root_instance),
         invariant
             index <= records.len(),
-            root_checkpoint_txg < u64::MAX,
-            chain_start_txg_without_anchor == root_checkpoint_txg + 1,
-            forall|record_index: int|
-                0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter
-                    < u64::MAX,
+            root_checkpoint_txg.0 < u64::MAX,
+            chain_start_txg_without_anchor.0 == root_checkpoint_txg.0 + 1,
+            records_ascend_by_instance_and_counter(records@),
+            every_counter_has_a_successor(records@),
             forall|record_index: usize|
                 record_index < records@.len() ==> #[trigger] back_chain_broken.requires(
                     (record_index,),
@@ -380,6 +1594,48 @@
                 record_index < records@.len() ==> #[trigger] named_units_verified.requires(
                     (record_index,),
                 ),
+            anchor == anchor_of_the_chain(records@, root_instance, root_checkpoint_txg),
+            anchor matches Some(anchor_counter) ==> anchor_counter.0 < u64::MAX,
+            report.above_water == candidates_before(
+                records@,
+                root_instance,
+                root_checkpoint_txg,
+                records@.len() as int,
+            ).len(),
+            // 过了逐条判定的：到 index 为止的全部候选
+            admitted == candidates_before(
+                records@,
+                root_instance,
+                root_checkpoint_txg,
+                index as int,
+            ),
+            admitted.len() <= in_flight_limit,
+            forall|position_in_chain: int|
+                0 <= position_in_chain < admitted.len() ==> #[trigger] admitted_record_passed_every_check(
+                    records@,
+                    admitted,
+                    position_in_chain,
+                    anchor,
+                    root_instance,
+                    root_checkpoint_txg,
+                    named_unit_verification,
+                    back_chain_broken,
+                    named_units_verified,
+                ),
+            report.verification_passed == match named_unit_verification {
+                NamedUnitVerification::Verify => admitted.len(),
+                NamedUnitVerification::Skip => 0,
+            },
+            applied_is_the_chain_cut_at_its_last_publish_end(records@, admitted, applied),
+            as_ints(records_of_the_open_publish@) == admitted.subrange(
+                applied.len() as int,
+                admitted.len() as int,
+            ),
+            maximum_transaction_is_the_largest_applied(
+                records@,
+                applied,
+                report.maximum_applied_transaction,
+            ),
             // 已施加的
             forall|position: int|
                 0 <= position < applied.len() ==> 0 <= #[trigger] applied[position] < index,
@@ -393,187 +1649,228 @@
                 ),
             is_concatenation_of_whole_publishes(records_at(records@, applied)),
             report.prefix_applied == applied.len(),
-            rebuilt == if applied.len() == 0 {
-                root_segment
+            root_source_after_replay == if applied.len() == 0 {
+                chosen_root_source
             } else {
-                records@[applied.last()].new_root_segment
+                records@[applied.last()].new_root_source
             },
             // 攒着的
-            forall|position: int| 0 <= position < open@.len() ==> (#[trigger] open@[position]) < index,
+            forall|position: int|
+                0 <= position < records_of_the_open_publish@.len()
+                    ==> (#[trigger] records_of_the_open_publish@[position]) < index,
             forall|earlier: int, later: int|
-                0 <= earlier < later < open@.len() ==> open@[earlier] < open@[later],
+                0 <= earlier < later < records_of_the_open_publish@.len()
+                    ==> records_of_the_open_publish@[earlier] < records_of_the_open_publish@[later],
             forall|applied_position: int, open_position: int|
-                0 <= applied_position < applied.len() && 0 <= open_position < open@.len()
-                    ==> applied[applied_position] < open@[open_position],
+                0 <= applied_position < applied.len() && 0 <= open_position
+                    < records_of_the_open_publish@.len() ==> applied[applied_position]
+                    < records_of_the_open_publish@[open_position],
             forall|position: int|
-                0 <= position < open@.len() ==> above_the_chosen_root(
-                    records@[#[trigger] open@[position] as int],
+                0 <= position < records_of_the_open_publish@.len() ==> above_the_chosen_root(
+                    records@[#[trigger] records_of_the_open_publish@[position] as int],
                     root_instance,
                     root_checkpoint_txg,
                 ),
-            is_open_publish(records_at(records@, as_ints(open@))),
-            applied.len() + open@.len() <= index,
-            // 还没接上链首（锚点读不出、一条都没收）时攒着的一定是空的：tsv-448 那一判冗余靠这一条
-            expected_next is None ==> open@.len() == 0,
+            is_open_publish(records_at(records@, as_ints(records_of_the_open_publish@))),
+            applied.len() + records_of_the_open_publish@.len() <= index,
+            // 还没接上链首（锚点读不出、一条都没收）时攒着的一定是空的：「锚点读不出时链首序号为 1」与「首条序号为 1」两判冗余靠这一条
+            expected_next is None ==> records_of_the_open_publish@.len() == 0,
+        ensures
+            index <= records.len(),
+            index == records.len() ==> report.verification_failed == 0,
+            index < records.len() ==> above_the_chosen_root(
+                records@[index as int],
+                root_instance,
+                root_checkpoint_txg,
+            ) && the_candidate_stops_the_chain(
+                records@,
+                admitted,
+                index as int,
+                in_flight_limit,
+                anchor,
+                root_instance,
+                root_checkpoint_txg,
+                named_unit_verification,
+                back_chain_broken,
+                named_units_verified,
+                report.verification_failed,
+            ),
         decreases records.len() - index,
     {
         let record = &records[index];
-        // [原文 3078–3080] above 的筛选条件，同上面那一遍
+        proof {
+            lemma_candidates_lie_below_their_end_in_ascending_order(
+                records@,
+                root_instance,
+                root_checkpoint_txg,
+                index as int,
+            );
+        }
         if !is_above_the_chosen_root(record, root_instance, root_checkpoint_txg) {
             index += 1;
             continue ;
         }
-        // [原文 3109] take(in_flight_limit)
-        if taken >= in_flight_limit {
+        // 在飞上限（已定项 18）：一次恢复重放的前缀最多这么多条
+        if records_taken_toward_the_in_flight_limit >= in_flight_limit {
             break ;
         }
-        taken += 1;
-        match expected_next {
-            // [原文 3110–3113] [改写] 元组不等摊成两个字段
-            Some(expected_key) => {
-                if record.instance != expected_key.0 || record.counter != expected_key.1 {
-                    break ;
-                }
-            },
-            // [原文 3114–3118] 锚点读不出：链首只能是 txg = 根的 txg + 1、本次发布内序号为 1 的那条（三方第一轮 K4-b）。
-            // 序号那一判与下面「攒着的为空且序号不是 1」那一判冗余（crates/mutations.tsv 第 448 行原来那条变异改的就是它）：
-            // 走到这一支时攒着的一定是空的（下面这句由 Verus 证），两判之间只有反向链回调，它只会断、不改状态。
-            None => {
-                proof {
-                    assert(open@.len() == 0);
-                }
-                if record.checkpoint_txg != chain_start_txg_without_anchor
-                    || record.ordinal_within_publish != 1 {
-                    break ;
-                }
-            },
-        }
-        // [原文 3119–3126] 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断。[改写] 三态由回调交成布尔
-        if back_chain_broken(index) {
-            break ;
-        }
-        // [原文 3127–3134] 一次发布的第一条序号是 1（已定项 4 读者规则；C539）。
-        if open.is_empty() && record.ordinal_within_publish != 1 {
-            break ;
-        }
-        // [原文 3135]
-        if !open.is_empty() {
-            let previous_record_of_the_open_publish = &records[open[open.len() - 1]];
-            // [原文 3136–3140] 末条还没到、下一条已经换了 txg ⇒ 这次发布缺了末条（D16（发布语义）已定项 6）
-            if record.checkpoint_txg != previous_record_of_the_open_publish.checkpoint_txg {
-                break ;
-            }
-            // [原文 3141–3147] 一次发布之内跳号（已定项 4）
-            if record.ordinal_within_publish as u64
-                != previous_record_of_the_open_publish.ordinal_within_publish as u64 + 1 {
-                break ;
-            }
-            // [原文 3148–3156] 上一条没带提交标记而这一条换了事务号 ⇒ 那个事务的提交标记没出现（已定项 7）
-            if !previous_record_of_the_open_publish.is_commit && record.transaction
-                != previous_record_of_the_open_publish.transaction {
-                break ;
-            }
-        }
-        // [原文 3158]
-        expected_next = Some((record.instance, record.counter + 1));
-        // [原文 3159–3163] 末条不带提交标记 ⇒ 这次发布最后一个事务没提交，整体不施加
-        if !record.is_commit && record_ends_its_publish(record) {
-            break ;
-        }
-        // [原文 3164–3172] 带末条标志的这一条之后同一 (实例代号, checkpoint_txg) 里还有读得出的记录（已定项 4 读者规则；C540）
-        if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(
+        records_taken_toward_the_in_flight_limit += 1;
+        match judge_the_candidate(
             records,
             index,
+            records_of_the_open_publish.as_slice(),
+            expected_next,
+            chain_start_txg_without_anchor,
+            named_unit_verification,
+            &back_chain_broken,
+            &named_units_verified,
+            Ghost(admitted),
+            Ghost(applied),
+            Ghost(anchor),
+            Ghost(root_instance),
+            Ghost(root_checkpoint_txg),
         ) {
-            break ;
+            CandidateVerdict::Admitted => {},
+            CandidateVerdict::BrokenBeforeTheNamedUnits => {
+                break ;
+            },
+            CandidateVerdict::BrokenAtTheNamedUnits => {
+                report.verification_failed += 1;
+                break ;
+            },
         }
-        // [原文 3173–3187] [改写] .all(..) 那一串读盘由调用方的回调交成布尔；verify_named_units 关着时不调
-        let all_verified = !verify_named_units || named_units_verified(index);
-        // [原文 3188–3191]
-        if !all_verified {
-            report.verification_failed += 1;
-            break ;
+        let ghost previous = previous_in_the_chain(records@, admitted, admitted.len() as int);
+        expected_next = Some((record.instance, JournalCounter(record.counter.0 + 1)));
+        // 只数真验过的：关掉点名验证时这一格不加（fs-design.md 五条硬要求第 4 条：分支必须可观测）
+        match named_unit_verification {
+            NamedUnitVerification::Verify => {
+                report.verification_passed += 1;
+            },
+            NamedUnitVerification::Skip => {},
         }
-        // [原文 3192–3197] 只数真验过的：关掉点名验证时这一格不加（fs-design.md 五条硬要求第 4 条：分支必须可观测）
-        if verify_named_units {
-            report.verification_passed += 1;
-        }
-        // [原文 3198]
-        let ghost open_before = open@;
-        open.push(index);
+        let ghost open_before = records_of_the_open_publish@;
+        let ghost admitted_before = admitted;
+        records_of_the_open_publish.push(index);
         proof {
-            let before = records_at(records@, as_ints(open_before));
-            let after = records_at(records@, as_ints(open@));
-            assert(is_open_publish(before));
-            assert forall|position: int| 0 <= position < after.len() implies (
-            #[trigger] after[position]).checkpoint_txg == after[0].checkpoint_txg
-                && after[position].ordinal_within_publish as int == position + 1 by {
-                if position < open_before.len() {
-                    assert(after[position] == before[position]);
-                    assert(after[0] == before[0]);
-                } else {
-                    assert(after[position] == records@[index as int]);
-                    if open_before.len() > 0 {
-                        assert(before[open_before.len() - 1] == records@[open_before[open_before.len()
-                            - 1] as int]);
-                        assert(after[0] == before[0]);
-                    }
-                }
-            }
-            assert forall|position: int|
-                0 <= position < open_before.len() implies !(#[trigger] after[position]).ends_its_publish by {
-                assert(after[position] == before[position]);
-            }
+            admitted = admitted.push(index as int);
+            lemma_candidates_lie_below_their_end_in_ascending_order(
+                records@,
+                root_instance,
+                root_checkpoint_txg,
+                index as int + 1,
+            );
+            assert(admitted == candidates_before(
+                records@,
+                root_instance,
+                root_checkpoint_txg,
+                index as int + 1,
+            ));
+            lemma_admitting_the_candidate_keeps_every_link(
+                records@,
+                admitted_before,
+                index as int,
+                anchor,
+                root_instance,
+                root_checkpoint_txg,
+                named_unit_verification,
+                back_chain_broken,
+                named_units_verified,
+            );
+            assert(records_of_the_open_publish@ == open_before.push(index));
+            lemma_admitting_the_candidate_extends_the_open_publish(
+                records@,
+                open_before,
+                index,
+                admitted_before,
+                applied,
+            );
         }
-        // [原文 3199–3202] 末条没到就接着往下走，这次发布先不施加
+        // 末条没到就接着往下走，这次发布先不施加
         if !record_ends_its_publish(record) {
             index += 1;
             continue ;
         }
-        // [原文 3203]
-        report.prefix_applied += open.len();
-        // [原文 3204–3207]
+        report.prefix_applied += records_of_the_open_publish.len();
+        let ghost maximum_before = report.maximum_applied_transaction;
         report.maximum_applied_transaction = maximum_transaction_of(
             records,
-            open.as_slice(),
+            records_of_the_open_publish.as_slice(),
             report.maximum_applied_transaction,
         );
         proof {
             let ghost applied_before = applied;
-            let this_publish = as_ints(open@);
+            let this_publish = as_ints(records_of_the_open_publish@);
             applied = applied + this_publish;
-            let before = records_at(records@, applied_before);
-            let publish = records_at(records@, this_publish);
-            let after = records_at(records@, applied);
-            assert(after =~= before + publish);
-            assert(after.subrange(applied_before.len() as int, after.len() as int) =~= publish);
-            assert(after.subrange(0, applied_before.len() as int) =~= before);
-            assert(publish[publish.len() - 1] == records@[index as int]);
-            assert(is_whole_publish(publish));
-            assert(is_whole_publish(
-                after.subrange(applied_before.len() as int, after.len() as int),
-            ));
-            assert(is_concatenation_of_whole_publishes(after));
+            lemma_appending_a_whole_publish_keeps_a_concatenation(records@, applied_before, this_publish);
+            lemma_the_maximum_over_the_appended_publish(
+                records@,
+                applied_before,
+                records_of_the_open_publish@,
+                maximum_before,
+                report.maximum_applied_transaction,
+            );
+            lemma_the_applied_head_and_the_open_tail_make_up_the_chain(
+                admitted,
+                applied_before,
+                this_publish,
+            );
             assert(applied.last() == index as int);
         }
-        // [原文 3208]
-        open.clear();
-        // [原文 3209–3226] 换根：核里只换「新根段」这一块，RootRecord 的其余字段由调用方照原文拼
-        rebuilt = record.new_root_segment;
+        records_of_the_open_publish.clear();
+        // 换根：核里只换「新根从哪来」这一项，RootRecord 的其余字段由调用方照原文拼
+        root_source_after_replay = record.new_root_source;
         index += 1;
     }
-    // [原文 3228]
-    Ok(CoreOutcome { report, rebuilt, applied: Ghost(applied) })
+    proof {
+        lemma_candidates_before_a_shorter_end_are_a_prefix(
+            records@,
+            root_instance,
+            root_checkpoint_txg,
+            index as int,
+            records@.len() as int,
+        );
+        if index < records.len() {
+            lemma_candidates_before_a_shorter_end_are_a_prefix(
+                records@,
+                root_instance,
+                root_checkpoint_txg,
+                index as int + 1,
+                records@.len() as int,
+            );
+            assert(candidates_before(records@, root_instance, root_checkpoint_txg, index as int + 1)
+                == admitted.push(index as int));
+            assert(candidates_before(
+                records@,
+                root_instance,
+                root_checkpoint_txg,
+                index as int + 1,
+            )[admitted.len() as int] == index as int);
+        }
+        lemma_candidates_lie_below_their_end_in_ascending_order(
+            records@,
+            root_instance,
+            root_checkpoint_txg,
+            index as int,
+        );
+    }
+    Ok(
+        CoreOutcome {
+            report,
+            root_source_after_replay,
+            admitted: Ghost(admitted),
+            applied: Ghost(applied),
+        },
+    )
 }
 
 // ===== spec 自己的冒烟（防 spec 恒真或恒假）=====
 proof fn a_publish_of_two_records_is_whole(first: CoreRecord<u64>, second: CoreRecord<u64>)
     requires
         first.checkpoint_txg == second.checkpoint_txg,
-        first.ordinal_within_publish == 1,
-        second.ordinal_within_publish == 2,
-        !first.ends_its_publish,
-        second.ends_its_publish,
+        first.ordinal_within_publish.0 == 1,
+        second.ordinal_within_publish.0 == 2,
+        !is_last_record_of_its_publish(first),
+        is_last_record_of_its_publish(second),
     ensures
         is_concatenation_of_whole_publishes(seq![first, second]),
 {
@@ -586,7 +1883,7 @@
 
 proof fn a_record_without_the_last_flag_alone_is_not_a_concatenation(only: CoreRecord<u64>)
     requires
-        !only.ends_its_publish,
+        !is_last_record_of_its_publish(only),
     ensures
         !is_concatenation_of_whole_publishes(seq![only]),
 {
@@ -605,29 +1902,35 @@
 // （crates/mutations.tsv 里点名它们的变异改坏核，它们红）。
 #[cfg(test)]
 mod tests {
-    use super::{replay_core, CoreRecord};
+    use super::{
+        replay_core, CheckpointTxg, CoreRecord, InstanceGeneration, JournalCounter,
+        JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish, NamedUnitVerification,
+        TransactionNumber,
+    };
 
-    const ROOT_INSTANCE: u32 = 1;
-    const ROOT_CHECKPOINT_TXG: u64 = 1;
+    const ROOT_INSTANCE: InstanceGeneration = InstanceGeneration(1);
+    const ROOT_CHECKPOINT_TXG: CheckpointTxg = CheckpointTxg(1);
     const ROOT_SEGMENT: u64 = 0;
     const IN_FLIGHT_LIMIT: usize = 16;
+    /// 每条记录的事务号取计数器加这个数：只为让事务号与计数器分得开，断言里的事务号从记录上取、不另算。
+    const TRANSACTION_NUMBER_ABOVE_THE_COUNTER: u64 = 3;
 
     fn record(
         counter: u64,
         checkpoint_txg: u64,
         ordinal_within_publish: u32,
-        ends_its_publish: bool,
+        place_in_publish: JournalRecordPlaceInPublish,
         new_root_segment: u64,
     ) -> CoreRecord<u64> {
         CoreRecord {
             instance: ROOT_INSTANCE,
-            counter,
-            checkpoint_txg,
-            transaction: counter + 3,
+            counter: JournalCounter(counter),
+            checkpoint_txg: CheckpointTxg(checkpoint_txg),
+            transaction: TransactionNumber(counter + TRANSACTION_NUMBER_ABOVE_THE_COUNTER),
             is_commit: true,
-            ordinal_within_publish,
-            ends_its_publish,
-            new_root_segment,
+            ordinal_within_publish: JournalRecordOrdinalWithinPublish(ordinal_within_publish),
+            place_in_publish,
+            new_root_source: new_root_segment,
         }
     }
 
@@ -636,9 +1939,27 @@
     fn a_two_record_publish_after_the_anchor_is_applied_whole_and_the_new_root_segment_comes_from_its_last_record(
     ) {
         let records = [
-            record(1, ROOT_CHECKPOINT_TXG, 1, true, 10),
-            record(2, 2, 1, false, 20),
-            record(3, 2, 2, true, 30),
+            record(
+                1,
+                ROOT_CHECKPOINT_TXG.0,
+                1,
+                JournalRecordPlaceInPublish::LastRecordOfThePublish,
+                10,
+            ),
+            record(
+                2,
+                2,
+                1,
+                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
+                20,
+            ),
+            record(
+                3,
+                2,
+                2,
+                JournalRecordPlaceInPublish::LastRecordOfThePublish,
+                30,
+            ),
         ];
         let Ok(outcome) = replay_core(
             ROOT_INSTANCE,
@@ -646,7 +1967,7 @@
             ROOT_SEGMENT,
             &records,
             IN_FLIGHT_LIMIT,
-            false,
+            NamedUnitVerification::Skip,
             |_index: usize| false,
             |_index: usize| true,
         ) else {
@@ -654,10 +1975,13 @@
         };
         assert_eq!(outcome.report.above_water, 2, "水位之上是 txg 2 的两条");
         assert_eq!(outcome.report.prefix_applied, 2, "两条整次施加");
-        assert_eq!(outcome.rebuilt, 30, "新根段取那次发布末条的");
         assert_eq!(
-            outcome.report.maximum_applied_transaction, 6,
-            "事务号取施加的里最大的"
+            outcome.root_source_after_replay, 30,
+            "新根段取那次发布末条的"
+        );
+        assert_eq!(
+            outcome.report.maximum_applied_transaction, records[2].transaction,
+            "事务号取施加的里最大的：那次发布末条的"
         );
     }
 
@@ -665,8 +1989,20 @@
     #[test]
     fn a_publish_whose_last_record_is_missing_is_not_applied_and_the_root_segment_stays() {
         let records = [
-            record(1, ROOT_CHECKPOINT_TXG, 1, true, 10),
-            record(2, 2, 1, false, 20),
+            record(
+                1,
+                ROOT_CHECKPOINT_TXG.0,
+                1,
+                JournalRecordPlaceInPublish::LastRecordOfThePublish,
+                10,
+            ),
+            record(
+                2,
+                2,
+                1,
+                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
+                20,
+            ),
         ];
         let Ok(outcome) = replay_core(
             ROOT_INSTANCE,
@@ -674,7 +2010,7 @@
             ROOT_SEGMENT,
             &records,
             IN_FLIGHT_LIMIT,
-            false,
+            NamedUnitVerification::Skip,
             |_index: usize| false,
             |_index: usize| true,
         ) else {
@@ -682,6 +2018,9 @@
         };
         assert_eq!(outcome.report.above_water, 1, "水位之上是 txg 2 那一条");
         assert_eq!(outcome.report.prefix_applied, 0, "末条没到：一条都不施加");
-        assert_eq!(outcome.rebuilt, ROOT_SEGMENT, "新根段还是所选根的");
+        assert_eq!(
+            outcome.root_source_after_replay, ROOT_SEGMENT,
+            "新根段还是所选根的"
+        );
     }
 }
--- a/crates/singlefs-replay-prefix/verus-mutations.tsv
+++ b/crates/singlefs-replay-prefix/verus-mutations.tsv
@@ -5,42 +5,102 @@
 # 跑法：把原文换成替换文，在 crates/singlefs-replay-prefix 下跑
 #   verus --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating src/lib.rs
 # （Verus 0.2026.09.20.aef82ed），输出里有期望那一截、退出码不为 0 算红，然后还原。平时的 cargo test 不跑这张表。
-# 每一条 ensures、每一条循环不变式各一行；第一行是接回量测报告里的 red_r2（record_ends_its_publish 函数体与 ensures 一起改成恒真）。
-ensures·record_ends_its_publish 函数体与 ensures 一起改成恒真（接回量测报告的 red_r2）	crates/singlefs-replay-prefix/src/lib.rs	        is_last_record == record.ends_its_publish,\n{\n    record.ends_its_publish\n}	        is_last_record == true,\n{\n    true\n}	verified, 1 errors
-ensures·record_ends_its_publish 的 ensures 取反	crates/singlefs-replay-prefix/src/lib.rs	        is_last_record == record.ends_its_publish,	        is_last_record == !record.ends_its_publish,	verified, 2 errors
-ensures·counter_of_the_last_record_the_root_covers 交回的锚点计数器小于 u64 最大值减一	crates/singlefs-replay-prefix/src/lib.rs	        anchor matches Ok(Some(counter)) ==> counter < u64::MAX,	        anchor matches Ok(Some(counter)) ==> counter < u64::MAX - 1,	verified, 1 errors
-ensures·replay_core 报错那一臂改成恒假	crates/singlefs-replay-prefix/src/lib.rs	            Err(_) => true,	            Err(_) => false,	verified, 1 errors
+# 两类行。前 81 行「改注解」：每一条 ensures、每一条循环不变式（含主循环的出口）、每个引理的结论各一行，改坏了证明就断；
+# 第一行是接回量测报告里的 red_r2（record_ends_its_publish 函数体与 ensures 一起改成恒真）。
+# 后 13 行「改判定」：只改 replay_core 一路上的一个可执行判定、不碰 spec 与不变式，spec 管得住那一条才红——
+# 攻方 research/prompts/m3-verus-land-code-r1-opus-model/verus-green-mutations.tsv 的 12 行加「删 I-8.3 那一判」
+# （代码轮判决 research/prompts/m3-verus-land-code-r1-main-verification.md 第二节 C2 那一行），扩证明之前这 13 行都验得过。
+ensures·record_ends_its_publish 函数体与 ensures 一起改成恒真（接回量测报告的 red_r2）	crates/singlefs-replay-prefix/src/lib.rs	        is_last_record == is_last_record_of_its_publish(*record),\n{\n    match record.place_in_publish {\n        JournalRecordPlaceInPublish::LastRecordOfThePublish => true,\n        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,\n    }\n}	        is_last_record == true,\n{\n    true\n}	verified, 3 errors
+ensures·record_ends_its_publish 的 ensures 取反	crates/singlefs-replay-prefix/src/lib.rs	        is_last_record == is_last_record_of_its_publish(*record),	        is_last_record == !is_last_record_of_its_publish(*record),	verified, 4 errors
+ensures·counter_of_the_last_record_the_root_covers 交回的锚点计数器小于 u64 最大值减一	crates/singlefs-replay-prefix/src/lib.rs	        anchor matches Ok(Some(counter)) ==> counter.0 < u64::MAX,	        anchor matches Ok(Some(counter)) ==> counter.0 < u64::MAX - 1,	verified, 2 errors
+ensures·counter_of_the_last_record_the_root_covers 交回的锚点改成不等于 spec 的锚点	crates/singlefs-replay-prefix/src/lib.rs	                Ok(found) => flagged.len() < 2 && found == anchor_of_the_chain(	                Ok(found) => flagged.len() < 2 && found != anchor_of_the_chain(	verified, 2 errors
+ensures·replay_core 整个结论取反	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        replay_follows_the_prefix_rules(	    ensures\n        !replay_follows_the_prefix_rules(	verified, 1 errors
+ensures·replay_core 报「所选根 txg 没有后继」那一臂改成恒假	crates/singlefs-replay-prefix/src/lib.rs	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => flagged.len() < 2\n            && root_checkpoint_txg.0 == u64::MAX,	        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => false,	verified, 1 errors
 ensures·replay_core 施加了的每一条都改成要求在所选根之下	crates/singlefs-replay-prefix/src/lib.rs	    &&& forall|position: int|\n        0 <= position < applied.len() ==> above_the_chosen_root(	    &&& forall|position: int|\n        0 <= position < applied.len() ==> !above_the_chosen_root(	verified, 1 errors
 ensures·冒烟：两条记录的整次发布改成断言不是拼接	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        is_concatenation_of_whole_publishes(seq![first, second]),	    ensures\n        !is_concatenation_of_whole_publishes(seq![first, second]),	verified, 1 errors
 ensures·冒烟：一条不带末条标志的记录改成断言是拼接	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        !is_concatenation_of_whole_publishes(seq![only]),	    ensures\n        is_concatenation_of_whole_publishes(seq![only]),	verified, 1 errors
-ensures·is_above_the_chosen_root 交回的判定取反	crates/singlefs-replay-prefix/src/lib.rs	        is_above == above_the_chosen_root(*record, root_instance, root_checkpoint_txg),	        is_above == !above_the_chosen_root(*record, root_instance, root_checkpoint_txg),	verified, 2 errors
-循环不变式·readable_record_of_the_same_publish_follows 下标到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            index < later_index <= records.len(),	            index < later_index < records.len(),	verified, 2 errors
-循环不变式·counter_of_the_last_record_the_root_covers 下标到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	        invariant\n            index <= records.len(),\n            forall|record_index: int|	        invariant\n            index < records.len(),\n            forall|record_index: int|	verified, 2 errors
-循环不变式·counter_of_the_last_record_the_root_covers 计数器上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter\n                    < u64::MAX,\n            found matches Some(counter) ==> counter < u64::MAX,	                0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter\n                    < u64::MAX - 1,\n            found matches Some(counter) ==> counter < u64::MAX,	verified, 1 errors
-循环不变式·counter_of_the_last_record_the_root_covers 找到的计数器改成要求小于 1	crates/singlefs-replay-prefix/src/lib.rs	            found matches Some(counter) ==> counter < u64::MAX,	            found matches Some(counter) ==> counter < 1,	verified, 1 errors
-循环不变式·maximum_transaction_of 位置到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            position <= open@.len(),	            position < open@.len(),	verified, 2 errors
-循环不变式·maximum_transaction_of 攒着的下标上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                0 <= open_position < open@.len() ==> (#[trigger] open@[open_position]) < records.len(),	                0 <= open_position < open@.len() ==> (#[trigger] open@[open_position]) + 1 < records.len(),	verified, 1 errors
+ensures·is_above_the_chosen_root 交回的判定取反	crates/singlefs-replay-prefix/src/lib.rs	        is_above == above_the_chosen_root(*record, root_instance, root_checkpoint_txg),	        is_above == !above_the_chosen_root(*record, root_instance, root_checkpoint_txg),	verified, 3 errors
+ensures·readable_record_of_the_same_publish_follows 交回的判定取反	crates/singlefs-replay-prefix/src/lib.rs	        follows == a_later_record_of_the_same_publish_is_readable(records@, index as int),	        follows == !a_later_record_of_the_same_publish_is_readable(records@, index as int),	verified, 3 errors
+ensures·maximum_transaction_of 交回的改成严格大于起点	crates/singlefs-replay-prefix/src/lib.rs	    ensures\n        maximum.0 >= start.0,	    ensures\n        maximum.0 > start.0,	verified, 2 errors
+ensures·maximum_transaction_of 攒着的每条事务号改成严格小于交回的	crates/singlefs-replay-prefix/src/lib.rs	                ==> records@[#[trigger] records_of_the_open_publish@[position] as int].transaction.0\n                <= maximum.0,	                ==> records@[#[trigger] records_of_the_open_publish@[position] as int].transaction.0\n                < maximum.0,	verified, 1 errors
+ensures·maximum_transaction_of 交回的改成「不等于起点、或是攒着的某一条的」	crates/singlefs-replay-prefix/src/lib.rs	        maximum == start || exists|position: int|	        maximum != start || exists|position: int|	verified, 2 errors
+ensures·judge_the_candidate 接受时读者规则改成不成立	crates/singlefs-replay-prefix/src/lib.rs	                ) && obeys_the_reader_rules_and_the_commit_markers(records@[index as int], previous)	                ) && !obeys_the_reader_rules_and_the_commit_markers(records@[index as int], previous)	verified, 2 errors
+ensures·judge_the_candidate 点名单元之前断时改成「没有一判不过」	crates/singlefs-replay-prefix/src/lib.rs	                CandidateVerdict::BrokenBeforeTheNamedUnits => fails_a_check_before_the_named_units(	                CandidateVerdict::BrokenBeforeTheNamedUnits => !fails_a_check_before_the_named_units(	verified, 2 errors
+ensures·judge_the_candidate 断在点名单元时改成点名单元之前就有一判不过	crates/singlefs-replay-prefix/src/lib.rs	                CandidateVerdict::BrokenAtTheNamedUnits => passes_every_check_before_the_named_units(	                CandidateVerdict::BrokenAtTheNamedUnits => !passes_every_check_before_the_named_units(	verified, 2 errors
+ensures·引理：候选个数改成严格小于终点	crates/singlefs-replay-prefix/src/lib.rs	            &&& candidates.len() <= end\n	            &&& candidates.len() < end\n	verified, 1 errors
+ensures·引理：短终点的候选个数改成严格少于长终点的	crates/singlefs-replay-prefix/src/lib.rs	            &&& fewer.len() <= more.len()	            &&& fewer.len() < more.len()	verified, 1 errors
+ensures·引理：同一实例里切片次序改成与计数器次序相反	crates/singlefs-replay-prefix/src/lib.rs	        first < second <==> records[first].counter.0 < records[second].counter.0,	        first < second <==> records[first].counter.0 > records[second].counter.0,	verified, 4 errors
+ensures·引理：接受下一个候选之后过了逐条判定的多算一条	crates/singlefs-replay-prefix/src/lib.rs	            0 <= position_in_chain < admitted_before.len() + 1 ==> #[trigger] admitted_record_passed_every_check(	            0 <= position_in_chain < admitted_before.len() + 2 ==> #[trigger] admitted_record_passed_every_check(	verified, 1 errors
+ensures·引理：接受下一个候选之后攒着的改成不是这一串的尾巴	crates/singlefs-replay-prefix/src/lib.rs	        as_ints(open_before.push(index)) == admitted_before.push(index as int).subrange(	        as_ints(open_before.push(index)) != admitted_before.push(index as int).subrange(	verified, 2 errors
+ensures·引理：接受了带末条标志的候选之后攒着的改成不是一次整的发布	crates/singlefs-replay-prefix/src/lib.rs	            &&& is_last_record_of_its_publish(records[index as int]) ==> is_whole_publish(after)	            &&& is_last_record_of_its_publish(records[index as int]) ==> !is_whole_publish(after)	verified, 1 errors
+ensures·引理：已施加的与攒着的接起来改成不是整串	crates/singlefs-replay-prefix/src/lib.rs	        applied + open_tail == admitted,	        applied + open_tail != admitted,	verified, 2 errors
+ensures·引理：接上一次整的发布之后改成不是整次发布的拼接	crates/singlefs-replay-prefix/src/lib.rs	        is_concatenation_of_whole_publishes(records_at(records, applied_before + this_publish)),	        !is_concatenation_of_whole_publishes(records_at(records, applied_before + this_publish)),	verified, 2 errors
+ensures·引理：接上一次发布之后最大事务号改成不是施加的里最大的	crates/singlefs-replay-prefix/src/lib.rs	        maximum_transaction_is_the_largest_applied(\n            records,\n            applied_before + as_ints(records_of_the_open_publish),	        !maximum_transaction_is_the_largest_applied(\n            records,\n            applied_before + as_ints(records_of_the_open_publish),	verified, 2 errors
+循环不变式·readable_record_of_the_same_publish_follows 下标到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            index < later_index <= records.len(),	            index < later_index < records.len(),	verified, 3 errors
+循环不变式·readable_record_of_the_same_publish_follows 这一条改成往后扫到的那一条	crates/singlefs-replay-prefix/src/lib.rs	            *record == records@[index as int],	            *record == records@[later_index as int],	verified, 2 errors
+循环不变式·readable_record_of_the_same_publish_follows 入参次序改成不升序	crates/singlefs-replay-prefix/src/lib.rs	            records_ascend_by_instance_and_counter(records@),\n            next_counter == records@[index as int].counter.0 + 1,	            !records_ascend_by_instance_and_counter(records@),\n            next_counter == records@[index as int].counter.0 + 1,	verified, 2 errors
+循环不变式·readable_record_of_the_same_publish_follows 下一号改成加二	crates/singlefs-replay-prefix/src/lib.rs	            next_counter == records@[index as int].counter.0 + 1,	            next_counter == records@[index as int].counter.0 + 2,	verified, 2 errors
+循环不变式·readable_record_of_the_same_publish_follows 扫过的同实例后继改成 txg 都更大	crates/singlefs-replay-prefix/src/lib.rs	                ) ==> records@[between].checkpoint_txg.0 < records@[index as int].checkpoint_txg.0,	                ) ==> records@[between].checkpoint_txg.0 > records@[index as int].checkpoint_txg.0,	verified, 1 errors
+循环不变式·counter_of_the_last_record_the_root_covers 下标到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	        invariant\n            index <= records.len(),\n            every_counter_has_a_successor(records@),	        invariant\n            index < records.len(),\n            every_counter_has_a_successor(records@),	verified, 2 errors
+循环不变式·counter_of_the_last_record_the_root_covers 计数器改成有到顶的	crates/singlefs-replay-prefix/src/lib.rs	        invariant\n            index <= records.len(),\n            every_counter_has_a_successor(records@),	        invariant\n            index <= records.len(),\n            !every_counter_has_a_successor(records@),	verified, 2 errors
+循环不变式·counter_of_the_last_record_the_root_covers 收下的计数器改成多看一条	crates/singlefs-replay-prefix/src/lib.rs	                root_checkpoint_txg,\n                index as int,\n            ),\n            forall|position: int|\n                0 <= position < counters_carrying_the_last_record_flag@.len()	                root_checkpoint_txg,\n                index as int + 1,\n            ),\n            forall|position: int|\n                0 <= position < counters_carrying_the_last_record_flag@.len()	verified, 2 errors
+循环不变式·counter_of_the_last_record_the_root_covers 收下的计数器上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                    ==> (#[trigger] counters_carrying_the_last_record_flag@[position]).0 < u64::MAX,	                    ==> (#[trigger] counters_carrying_the_last_record_flag@[position]).0 < u64::MAX - 1,	verified, 1 errors
+循环不变式·counter_of_the_last_record_the_root_covers 收下的计数器改成要求小于 1	crates/singlefs-replay-prefix/src/lib.rs	                    ==> (#[trigger] counters_carrying_the_last_record_flag@[position]).0 < u64::MAX,	                    ==> (#[trigger] counters_carrying_the_last_record_flag@[position]).0 < 1,	verified, 1 errors
+循环不变式·maximum_transaction_of 位置到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            position <= records_of_the_open_publish@.len(),	            position < records_of_the_open_publish@.len(),	verified, 2 errors
+循环不变式·maximum_transaction_of 攒着的下标上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                    ==> (#[trigger] records_of_the_open_publish@[open_position]) < records.len(),	                    ==> (#[trigger] records_of_the_open_publish@[open_position]) + 1 < records.len(),	verified, 1 errors
+循环不变式·maximum_transaction_of 当前最大改成严格大于起点	crates/singlefs-replay-prefix/src/lib.rs	            maximum.0 >= start.0,\n            forall|open_position: int|	            maximum.0 > start.0,\n            forall|open_position: int|	verified, 1 errors
+循环不变式·maximum_transaction_of 看过的每条事务号改成严格小于当前最大	crates/singlefs-replay-prefix/src/lib.rs	                    ==> records@[#[trigger] records_of_the_open_publish@[open_position] as int].transaction.0\n                    <= maximum.0,	                    ==> records@[#[trigger] records_of_the_open_publish@[open_position] as int].transaction.0\n                    < maximum.0,	verified, 1 errors
+循环不变式·maximum_transaction_of 当前最大改成「不等于起点、或是看过的某一条的」	crates/singlefs-replay-prefix/src/lib.rs	            maximum == start || exists|open_position: int|	            maximum != start || exists|open_position: int|	verified, 1 errors
 循环不变式·数水位之上那一遍下标到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            counting_index <= records.len(),	            counting_index < records.len(),	verified, 2 errors
 循环不变式·数水位之上那一遍条数改成严格小于下标	crates/singlefs-replay-prefix/src/lib.rs	            report.above_water <= counting_index,	            report.above_water < counting_index,	verified, 1 errors
-循环不变式·主循环已取条数改成严格小于下标	crates/singlefs-replay-prefix/src/lib.rs	            taken <= index,	            taken < index,	verified, 1 errors
-循环不变式·主循环验过条数改成严格小于下标	crates/singlefs-replay-prefix/src/lib.rs	            report.verification_passed <= index,	            report.verification_passed < index,	verified, 1 errors
-循环不变式·主循环校验不过条数改成 1	crates/singlefs-replay-prefix/src/lib.rs	            report.verification_failed == 0,	            report.verification_failed == 1,	verified, 1 errors
-循环不变式·主循环下标到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            index <= records.len(),\n            root_checkpoint_txg < u64::MAX,	            index < records.len(),\n            root_checkpoint_txg < u64::MAX,	verified, 2 errors
-循环不变式·主循环所选根 txg 上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	            root_checkpoint_txg < u64::MAX,	            root_checkpoint_txg < u64::MAX - 1,	verified, 1 errors
-循环不变式·主循环无锚点链首 txg 改成根加二	crates/singlefs-replay-prefix/src/lib.rs	            chain_start_txg_without_anchor == root_checkpoint_txg + 1,	            chain_start_txg_without_anchor == root_checkpoint_txg + 2,	verified, 1 errors
-循环不变式·主循环计数器上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter\n                    < u64::MAX,\n            forall|record_index: usize|\n                record_index < records@.len() ==> #[trigger] back_chain_broken.requires(\n                    (record_index,),	                0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter\n                    < u64::MAX - 1,\n            forall|record_index: usize|\n                record_index < records@.len() ==> #[trigger] back_chain_broken.requires(\n                    (record_index,),	verified, 1 errors
+循环不变式·数水位之上那一遍条数改成多看一条的候选数	crates/singlefs-replay-prefix/src/lib.rs	                counting_index as int,\n            ).len(),	                counting_index as int + 1,\n            ).len(),	verified, 2 errors
+循环不变式·主循环已取条数改成比已接受的多一	crates/singlefs-replay-prefix/src/lib.rs	            records_taken_toward_the_in_flight_limit == admitted.len(),	            records_taken_toward_the_in_flight_limit == admitted.len() + 1,	verified, 2 errors
+循环不变式·主循环校验不过条数改成 1	crates/singlefs-replay-prefix/src/lib.rs	            report.verification_failed == 0,	            report.verification_failed == 1,	verified, 2 errors
+循环不变式·主循环下一个该接的 jsn 改成不是这一串定下的	crates/singlefs-replay-prefix/src/lib.rs	            expected_next == expected_next_after(records@, admitted, anchor, root_instance),	            expected_next != expected_next_after(records@, admitted, anchor, root_instance),	verified, 2 errors
+循环不变式·主循环下标到不了末尾	crates/singlefs-replay-prefix/src/lib.rs	            index <= records.len(),\n            root_checkpoint_txg.0 < u64::MAX,	            index < records.len(),\n            root_checkpoint_txg.0 < u64::MAX,	verified, 2 errors
+循环不变式·主循环所选根 txg 上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	            root_checkpoint_txg.0 < u64::MAX,	            root_checkpoint_txg.0 < u64::MAX - 1,	verified, 2 errors
+循环不变式·主循环无锚点链首 txg 改成根加二	crates/singlefs-replay-prefix/src/lib.rs	            chain_start_txg_without_anchor.0 == root_checkpoint_txg.0 + 1,	            chain_start_txg_without_anchor.0 == root_checkpoint_txg.0 + 2,	verified, 2 errors
+循环不变式·主循环入参次序改成不升序	crates/singlefs-replay-prefix/src/lib.rs	            records_ascend_by_instance_and_counter(records@),\n            every_counter_has_a_successor(records@),	            !records_ascend_by_instance_and_counter(records@),\n            every_counter_has_a_successor(records@),	verified, 2 errors
+循环不变式·主循环计数器改成有到顶的	crates/singlefs-replay-prefix/src/lib.rs	            records_ascend_by_instance_and_counter(records@),\n            every_counter_has_a_successor(records@),	            records_ascend_by_instance_and_counter(records@),\n            !every_counter_has_a_successor(records@),	verified, 2 errors
 循环不变式·主循环反向链回调的前置条件多要一格下标	crates/singlefs-replay-prefix/src/lib.rs	                record_index < records@.len() ==> #[trigger] back_chain_broken.requires(\n                    (record_index,),	                record_index <= records@.len() ==> #[trigger] back_chain_broken.requires(\n                    (record_index,),	verified, 1 errors
 循环不变式·主循环点名校验回调的前置条件多要一格下标	crates/singlefs-replay-prefix/src/lib.rs	                record_index < records@.len() ==> #[trigger] named_units_verified.requires(\n                    (record_index,),\n                ),	                record_index <= records@.len() ==> #[trigger] named_units_verified.requires(\n                    (record_index,),\n                ),	verified, 1 errors
-循环不变式·主循环已施加的下标上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < applied.len() ==> 0 <= #[trigger] applied[position] < index,	                0 <= position < applied.len() ==> 0 <= #[trigger] applied[position] + 1 < index,	verified, 2 errors
-循环不变式·主循环已施加的下标改成严格递减	crates/singlefs-replay-prefix/src/lib.rs	                0 <= earlier < later < applied.len() ==> applied[earlier] < applied[later],	                0 <= earlier < later < applied.len() ==> applied[earlier] > applied[later],	verified, 2 errors
-循环不变式·主循环已施加的每一条改成要求在所选根之下	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < applied.len() ==> above_the_chosen_root(\n                    records@[#[trigger] applied[position]],	                0 <= position < applied.len() ==> !above_the_chosen_root(\n                    records@[#[trigger] applied[position]],	verified, 2 errors
+循环不变式·主循环锚点改成不是 spec 的锚点	crates/singlefs-replay-prefix/src/lib.rs	            anchor == anchor_of_the_chain(records@, root_instance, root_checkpoint_txg),	            anchor != anchor_of_the_chain(records@, root_instance, root_checkpoint_txg),	verified, 1 errors
+循环不变式·主循环锚点计数器上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	            anchor matches Some(anchor_counter) ==> anchor_counter.0 < u64::MAX,	            anchor matches Some(anchor_counter) ==> anchor_counter.0 < u64::MAX - 1,	verified, 2 errors
+循环不变式·主循环水位之上条数改成多一	crates/singlefs-replay-prefix/src/lib.rs	                records@.len() as int,\n            ).len(),	                records@.len() as int,\n            ).len() + 1,	verified, 1 errors
+循环不变式·主循环过了逐条判定的改成多看一条的候选	crates/singlefs-replay-prefix/src/lib.rs	                index as int,\n            ),\n            admitted.len() <= in_flight_limit,	                index as int + 1,\n            ),\n            admitted.len() <= in_flight_limit,	verified, 2 errors
+循环不变式·主循环过了逐条判定的条数改成严格小于在飞上限	crates/singlefs-replay-prefix/src/lib.rs	            admitted.len() <= in_flight_limit,	            admitted.len() < in_flight_limit,	verified, 2 errors
+循环不变式·主循环过了逐条判定的每一条改成多要一条	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position_in_chain < admitted.len() ==> #[trigger] admitted_record_passed_every_check(\n                    records@,	                0 <= position_in_chain <= admitted.len() ==> #[trigger] admitted_record_passed_every_check(\n                    records@,	verified, 2 errors
+循环不变式·主循环验过条数改成比过了逐条判定的多一	crates/singlefs-replay-prefix/src/lib.rs	            report.verification_passed == match named_unit_verification {\n                NamedUnitVerification::Verify => admitted.len(),	            report.verification_passed == match named_unit_verification {\n                NamedUnitVerification::Verify => admitted.len() + 1,	verified, 2 errors
+循环不变式·主循环已施加的改成不是这一串到最后一个末条为止的那一截	crates/singlefs-replay-prefix/src/lib.rs	            applied_is_the_chain_cut_at_its_last_publish_end(records@, admitted, applied),	            !applied_is_the_chain_cut_at_its_last_publish_end(records@, admitted, applied),	verified, 2 errors
+循环不变式·主循环攒着的改成不是这一串在已施加之后的尾巴	crates/singlefs-replay-prefix/src/lib.rs	            as_ints(records_of_the_open_publish@) == admitted.subrange(	            as_ints(records_of_the_open_publish@) != admitted.subrange(	verified, 2 errors
+循环不变式·主循环最大事务号改成不是施加的里最大的	crates/singlefs-replay-prefix/src/lib.rs	            maximum_transaction_is_the_largest_applied(\n                records@,\n                applied,	            !maximum_transaction_is_the_largest_applied(\n                records@,\n                applied,	verified, 2 errors
+循环不变式·主循环已施加的下标上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < applied.len() ==> 0 <= #[trigger] applied[position] < index,	                0 <= position < applied.len() ==> 0 <= #[trigger] applied[position] + 1 < index,	verified, 1 errors
+循环不变式·主循环已施加的下标改成严格递减	crates/singlefs-replay-prefix/src/lib.rs	                0 <= earlier < later < applied.len() ==> applied[earlier] < applied[later],	                0 <= earlier < later < applied.len() ==> applied[earlier] > applied[later],	verified, 1 errors
+循环不变式·主循环已施加的每一条改成要求在所选根之下	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < applied.len() ==> above_the_chosen_root(\n                    records@[#[trigger] applied[position]],	                0 <= position < applied.len() ==> !above_the_chosen_root(\n                    records@[#[trigger] applied[position]],	verified, 1 errors
 循环不变式·主循环已施加的改成不是整次发布的拼接	crates/singlefs-replay-prefix/src/lib.rs	            is_concatenation_of_whole_publishes(records_at(records@, applied)),	            !is_concatenation_of_whole_publishes(records_at(records@, applied)),	verified, 2 errors
 循环不变式·主循环施加条数多算一条	crates/singlefs-replay-prefix/src/lib.rs	            report.prefix_applied == applied.len(),	            report.prefix_applied == applied.len() + 1,	verified, 2 errors
-循环不变式·主循环新根段两臂对调	crates/singlefs-replay-prefix/src/lib.rs	            rebuilt == if applied.len() == 0 {	            rebuilt == if applied.len() != 0 {	verified, 2 errors
-循环不变式·主循环攒着的下标上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	            forall|position: int| 0 <= position < open@.len() ==> (#[trigger] open@[position]) < index,	            forall|position: int| 0 <= position < open@.len() ==> (#[trigger] open@[position]) + 1 < index,	verified, 1 errors
-循环不变式·主循环攒着的下标改成严格递减	crates/singlefs-replay-prefix/src/lib.rs	                0 <= earlier < later < open@.len() ==> open@[earlier] < open@[later],	                0 <= earlier < later < open@.len() ==> open@[earlier] > open@[later],	verified, 1 errors
-循环不变式·主循环已施加的改成都在攒着的之后	crates/singlefs-replay-prefix/src/lib.rs	                    ==> applied[applied_position] < open@[open_position],	                    ==> applied[applied_position] > open@[open_position],	verified, 1 errors
-循环不变式·主循环攒着的每一条改成要求在所选根之下	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < open@.len() ==> above_the_chosen_root(	                0 <= position < open@.len() ==> !above_the_chosen_root(	verified, 1 errors
-循环不变式·主循环攒着的改成不是未完的一次发布	crates/singlefs-replay-prefix/src/lib.rs	            is_open_publish(records_at(records@, as_ints(open@))),	            !is_open_publish(records_at(records@, as_ints(open@))),	verified, 2 errors
-循环不变式·主循环已施加加攒着的条数改成严格小于下标	crates/singlefs-replay-prefix/src/lib.rs	            applied.len() + open@.len() <= index,	            applied.len() + open@.len() < index,	verified, 1 errors
-循环不变式·主循环没接上链首时攒着为空改成接上之后攒着为空（tsv-448 那一判冗余的证明跟着断）	crates/singlefs-replay-prefix/src/lib.rs	            expected_next is None ==> open@.len() == 0,	            expected_next is Some ==> open@.len() == 0,	verified, 1 errors
+循环不变式·主循环交回的根从哪来两臂对调	crates/singlefs-replay-prefix/src/lib.rs	            root_source_after_replay == if applied.len() == 0 {	            root_source_after_replay == if applied.len() != 0 {	verified, 2 errors
+循环不变式·主循环攒着的下标上界收紧一格	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < records_of_the_open_publish@.len()\n                    ==> (#[trigger] records_of_the_open_publish@[position]) < index,	                0 <= position < records_of_the_open_publish@.len()\n                    ==> (#[trigger] records_of_the_open_publish@[position]) + 1 < index,	verified, 1 errors
+循环不变式·主循环攒着的下标改成严格递减	crates/singlefs-replay-prefix/src/lib.rs	                    ==> records_of_the_open_publish@[earlier] < records_of_the_open_publish@[later],	                    ==> records_of_the_open_publish@[earlier] > records_of_the_open_publish@[later],	verified, 1 errors
+循环不变式·主循环已施加的改成都在攒着的之后	crates/singlefs-replay-prefix/src/lib.rs	                    < records_of_the_open_publish@.len() ==> applied[applied_position]\n                    < records_of_the_open_publish@[open_position],	                    < records_of_the_open_publish@.len() ==> applied[applied_position]\n                    > records_of_the_open_publish@[open_position],	verified, 1 errors
+循环不变式·主循环攒着的每一条改成要求在所选根之下	crates/singlefs-replay-prefix/src/lib.rs	                0 <= position < records_of_the_open_publish@.len() ==> above_the_chosen_root(	                0 <= position < records_of_the_open_publish@.len() ==> !above_the_chosen_root(	verified, 1 errors
+循环不变式·主循环攒着的改成不是未完的一次发布	crates/singlefs-replay-prefix/src/lib.rs	            is_open_publish(records_at(records@, as_ints(records_of_the_open_publish@))),	            !is_open_publish(records_at(records@, as_ints(records_of_the_open_publish@))),	verified, 2 errors
+循环不变式·主循环已施加加攒着的条数改成严格小于下标	crates/singlefs-replay-prefix/src/lib.rs	            applied.len() + records_of_the_open_publish@.len() <= index,	            applied.len() + records_of_the_open_publish@.len() < index,	verified, 1 errors
+循环不变式·主循环没接上链首时攒着为空改成接上之后攒着为空（「锚点读不出时链首序号为 1」那一判冗余的证明跟着断）	crates/singlefs-replay-prefix/src/lib.rs	            expected_next is None ==> records_of_the_open_publish@.len() == 0,	            expected_next is Some ==> records_of_the_open_publish@.len() == 0,	verified, 1 errors
+循环出口·主循环出口改成下标没到末尾	crates/singlefs-replay-prefix/src/lib.rs	            index <= records.len(),\n            index == records.len() ==> report.verification_failed == 0,	            index < records.len(),\n            index == records.len() ==> report.verification_failed == 0,	verified, 1 errors
+循环出口·主循环走完时校验不过条数改成 1	crates/singlefs-replay-prefix/src/lib.rs	            index == records.len() ==> report.verification_failed == 0,	            index == records.len() ==> report.verification_failed == 1,	verified, 2 errors
+循环出口·主循环停下时那一条改成不在所选根之上	crates/singlefs-replay-prefix/src/lib.rs	            index < records.len() ==> above_the_chosen_root(\n                records@[index as int],	            index < records.len() ==> !above_the_chosen_root(\n                records@[index as int],	verified, 2 errors
+循环出口·主循环停下的理由改成落在下一个下标上	crates/singlefs-replay-prefix/src/lib.rs	                admitted,\n                index as int,\n                in_flight_limit,	                admitted,\n                index as int + 1,\n                in_flight_limit,	verified, 2 errors
+改判定·反向链回调交回「断」也不断链（I-8.6、已定项 8；攻方 V01）	crates/singlefs-replay-prefix/src/lib.rs	    if back_chain_broken(index) {	    if false {	verified, 1 errors
+改判定·上一条没带提交标记而这一条换了事务号照接（已定项 7；攻方 V02）	crates/singlefs-replay-prefix/src/lib.rs	        if !previous_record_of_the_open_publish.is_commit\n            && record.transaction.0 != previous_record_of_the_open_publish.transaction.0 {	        if false {	verified, 1 errors
+改判定·末条不带提交标记照施加（已定项 14 五条口径第三条；攻方 V03）	crates/singlefs-replay-prefix/src/lib.rs	    if !record.is_commit && record_ends_its_publish(record) {	    if false {	verified, 1 errors
+改判定·末条之后同一次发布还有记录照施加（已定项 4 读者规则第四格、C540；攻方 V04）	crates/singlefs-replay-prefix/src/lib.rs	    if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(	    if false && readable_record_of_the_same_publish_follows(	verified, 1 errors
+改判定·点名单元不验（已定项 14 五条口径第四条；攻方 V05）	crates/singlefs-replay-prefix/src/lib.rs	        NamedUnitVerification::Verify => named_units_verified(index),	        NamedUnitVerification::Verify => true,	verified, 1 errors
+改判定·在飞上限不看（已定项 18；攻方 V06）	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {	        if false {	verified, 1 errors
+改判定·第一条水位之上的记录就停、一条都不施加（攻方 V07）	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {	        if true {	verified, 1 errors
+改判定·锚点读不出时链首不看 txg（已定项 14 注 1；三方第一轮 K4-a；攻方 V08）	crates/singlefs-replay-prefix/src/lib.rs	            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0\n                || record.ordinal_within_publish.0 != 1 {	            if false\n                || record.ordinal_within_publish.0 != 1 {	verified, 1 errors
+改判定·每次都报所选根 txg 没有后继（攻方 V09）	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg.0 == u64::MAX {	    if true {	verified, 1 errors
+改判定·最大事务号取最小（已定项 14 注 4 的 W 的输入；攻方 V10）	crates/singlefs-replay-prefix/src/lib.rs	        if transaction.0 > maximum.0 {	        if transaction.0 < maximum.0 {	verified, 1 errors
+改判定·水位之上条数不数（报告字段；攻方 V11）	crates/singlefs-replay-prefix/src/lib.rs	            report.above_water += 1;	            report.above_water += 0;	verified, 1 errors
+改判定·锚点筛选不看 txg（同实例任一带末条标志的都算；已定项 14 注 1；攻方 V12、K19）	crates/singlefs-replay-prefix/src/lib.rs	        if record.instance.0 == root_instance.0 && record.checkpoint_txg.0 == root_checkpoint_txg.0\n            && record_ends_its_publish(record) {	        if record.instance.0 == root_instance.0\n            && record_ends_its_publish(record) {	verified, 1 errors
+改判定·删掉 I-8.3 那一判（jsn 严格连续不判；已定项 22；第二轮判决第二节第 2 条）	crates/singlefs-replay-prefix/src/lib.rs	            if record.instance.0 != expected_key.0.0 || record.counter.0 != expected_key.1.0 {	            if false {	verified, 1 errors
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -17,7 +17,9 @@
     MAPPING_ENTRY_BYTES, NODE_BYTES, ROOT_RECORD_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
     SYSTEM_CONFIGURATION_SLOT_BYTES,
 };
-use singlefs_replay_prefix::{replay_core, CoreFailure, CoreRecord};
+use singlefs_replay_prefix::{
+    replay_core, CoreFailure, CoreRecord, JournalCounter, NamedUnitVerification, TransactionNumber,
+};
 
 use crate::address::{
     CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
@@ -2928,49 +2930,6 @@
     records
 }
 
-/// 一条记录是不是它那次发布的末条：记录标志位 0（D23（journal 的角色与格式） 已定项 14 第五条「发布边界按记录标志位 0 认」、
-/// 已定项 17：只有真正的最后一条带「本次发布末条」标志，每次只有一条记录的发布——含空发布记录——那一条也带）。
-fn record_ends_its_publish(record: &JournalRecord) -> bool {
-    match record.place_in_publish {
-        JournalRecordPlaceInPublish::LastRecordOfThePublish => true,
-        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,
-    }
-}
-
-/// 所选根覆盖的最后一条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 14 注 1，读法乙，用户 2026-09-24 定）：
-/// 「那条」按末条标志认——所选根那次发布（与所选根同实例、同 checkpoint_txg）读得出的几条里带「本次发布末条」标志的那一条。
-/// 一条都不带 ⇒ `Ok(None)`，就算「那条读不出」，链首走「序号为 1 的第一条可读记录」那一支；**不取**读得出的同 txg 记录里
-/// jsn 最大的那条（读法甲：末条读不出而前几条读得出时，它锚在前几条上，下一次发布一条都接不上）。
-///
-/// # Errors
-/// 带标志的多于一条 ⇒ `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`：认哪一条条款没有写。
-fn counter_of_the_last_record_the_root_covers(
-    root: &RootRecord,
-    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
-) -> Result<Option<u64>, RecoveryFailure> {
-    // `records` 按 (实例代号, 计数器) 排序 ⇒ 同一实例里按计数器升序。
-    let counters_carrying_the_last_record_flag: Vec<u64> = records
-        .values()
-        .filter(|record| {
-            record.instance == root.instance
-                && record.checkpoint_txg == root.checkpoint_txg
-                && record_ends_its_publish(record)
-        })
-        .map(|record| record.counter)
-        .collect();
-    if counters_carrying_the_last_record_flag.len() > 1 {
-        return Err(
-            RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
-                instance: root.instance,
-                checkpoint_txg: root.checkpoint_txg,
-                counters: counters_carrying_the_last_record_flag,
-            },
-        );
-    }
-    // 至多一条：带标志的那一条就是锚点，一条都没有就是「那条读不出」。
-    Ok(counters_carrying_the_last_record_flag.first().copied())
-}
-
 /// 一条记录的反向链对不对得上本实例内逻辑前一条（I-8.6（反向链算法））。封闭集合，`match` 不写通配臂。
 enum BackChainJudgement {
     /// 前一条在盘上，它的头算出来的链值等于这一条的反向链。
@@ -3027,9 +2986,9 @@
 
 /// 取前缀并施加（D23（journal 的角色与格式） 已定项 14 / 已定项 15）；返回扫描报告与施加之后的根。
 ///
-/// 链首锚在所选根覆盖的最后一条：所选根那次发布里带「本次发布末条」标志的那一条（已定项 14 注 1，读法乙，
-/// [`counter_of_the_last_record_the_root_covers`]）。
-/// 施加的单位是一次发布（第五条）：前四条判出来的前缀里，一次发布的记录要一直走到带末条标志的那一条（[`record_ends_its_publish`]）
+/// 链首锚在所选根覆盖的最后一条：所选根那次发布里带「本次发布末条」标志的那一条（已定项 14 注 1，读法乙；
+/// 不取读得出的同 txg 记录里 jsn 最大的那条）。
+/// 施加的单位是一次发布（第五条）：前四条判出来的前缀里，一次发布的记录要一直走到带末条标志（记录标志位 0）的那一条
 /// 才整体施加；前缀停在一次发布中间（末条没到、断号、校验不过、下一条换了 txg、一次发布之内跳号、
 /// 一个事务的提交标记没出现）⇒ 那次发布整体不施加。读者规则另外两格（D23（journal 的角色与格式） 已定项 4）同样断链、那次发布不施加：
 /// 一次发布的首条序号不是 1（断在这一条）；带末条标志的那一条之后同一 (实例代号, checkpoint_txg) 里还有读得出的记录（断在带标志的那一条）。
@@ -3039,6 +2998,9 @@
 /// `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`，一条记录都没施加；
 /// 所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布没有 txg 可取）⇒ `UnitMalformed`
 /// （[`CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`]），一条记录都没施加。
+///
+/// # Panics
+/// `records` 的键不等于记录自己的 (实例代号, 计数器)，或有记录的计数器是 u64 的最大值：这两条是核的前置条件，调用方违反就是 bug。
 pub fn replay_journal(
     reader: &dyn PoolReader,
     root: &RootRecord,
@@ -3058,21 +3020,42 @@
     // 把两处读盘（反向链重读、点名单元校验）包成回调交给核、照核交回的结果拼报告与新根。
     let in_flight_limit =
         usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
-    // `records` 按 (实例代号, 计数器) 升序，`values()` 就是那个次序；核里「末条之后同发布还有没有记录」那一判靠这个次序。
+    // 核的两条前置条件 Verus 只在核里假定、不在调用点查，在这里断言：
+    for (key, record) in records {
+        assert_eq!(
+            *key,
+            (record.instance, record.counter),
+            "记录表的键要等于记录自己的 (实例代号, 计数器)：scan_journal 与各调用方都按它建键，键序因此就是核要的「按 (实例代号, 计数器) 严格升序」；\
+             不等说明调用方建错了表，核的前置条件不成立"
+        );
+        assert!(
+            record.counter < u64::MAX,
+            "计数器是盘上 6 字节读来的（D23（journal 的角色与格式） 已定项 9 的 48 位），到不了 u64 的最大值；核要给链上下一号加一"
+        );
+    }
     let records_in_key_order: Vec<&JournalRecord> = records.values().collect();
-    // 核的「新根段」取记录在表里的下标：`None` 是一条都没施加，`Some(下标)` 是最后施加的那次发布的末条。
+    // 核的「新根从哪来」取记录在表里的下标：`None` 是所选根自己，`Some(下标)` 是最后施加的那次发布的末条。
     let core_records: Vec<CoreRecord<Option<usize>>> = records_in_key_order
         .iter()
         .enumerate()
         .map(|(index, record)| CoreRecord {
-            instance: record.instance.0,
-            counter: record.counter,
-            checkpoint_txg: record.checkpoint_txg.0,
-            transaction: record.transaction,
+            instance: singlefs_replay_prefix::InstanceGeneration(record.instance.0),
+            counter: JournalCounter(record.counter),
+            checkpoint_txg: singlefs_replay_prefix::CheckpointTxg(record.checkpoint_txg.0),
+            transaction: TransactionNumber(record.transaction),
             is_commit: record.is_commit,
-            ordinal_within_publish: record.ordinal_within_publish.0,
-            ends_its_publish: record_ends_its_publish(record),
-            new_root_segment: Some(index),
+            ordinal_within_publish: singlefs_replay_prefix::JournalRecordOrdinalWithinPublish(
+                record.ordinal_within_publish.0,
+            ),
+            place_in_publish: match record.place_in_publish {
+                JournalRecordPlaceInPublish::LastRecordOfThePublish => {
+                    singlefs_replay_prefix::JournalRecordPlaceInPublish::LastRecordOfThePublish
+                }
+                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => {
+                    singlefs_replay_prefix::JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
+                }
+            },
+            new_root_source: Some(index),
         })
         .collect();
     // 反向链（D23（journal 的角色与格式） 已定项 7「jsn 严格连续 + 校验和 + 反向链」；I-8.6（反向链算法））：
@@ -3103,23 +3086,31 @@
             })
         })
     };
+    let named_unit_verification = if verify_named_units {
+        NamedUnitVerification::Verify
+    } else {
+        NamedUnitVerification::Skip
+    };
     let outcome = match replay_core(
-        root.instance.0,
-        root.checkpoint_txg.0,
+        singlefs_replay_prefix::InstanceGeneration(root.instance.0),
+        singlefs_replay_prefix::CheckpointTxg(root.checkpoint_txg.0),
         None,
         &core_records,
         in_flight_limit,
-        verify_named_units,
+        named_unit_verification,
         back_chain_broken,
         named_units_verified,
     ) {
         Ok(outcome) => outcome,
-        // 报错里的计数器清单核不产，由原来那个函数再算一遍（它在同一张表上必然报同一个错）。
-        Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlag) => {
+        // 核带出它看到的那几条计数器（按升序），直接拼报错。
+        Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+            counters,
+        }) => {
             return Err(
-                match counter_of_the_last_record_the_root_covers(root, records) {
-                    Err(failure) => failure,
-                    Ok(_) => unreachable!("核报了所选根那次发布带末条标志的多于一条"),
+                RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                    instance: root.instance,
+                    checkpoint_txg: root.checkpoint_txg,
+                    counters: counters.iter().map(|counter| counter.0).collect(),
                 },
             );
         }
@@ -3133,8 +3124,8 @@
     report.prefix_applied = outcome.report.prefix_applied;
     report.verification_passed = outcome.report.verification_passed;
     report.verification_failed = outcome.report.verification_failed;
-    report.maximum_applied_transaction = outcome.report.maximum_applied_transaction;
-    let rebuilt = match outcome.rebuilt {
+    report.maximum_applied_transaction = outcome.report.maximum_applied_transaction.0;
+    let rebuilt = match outcome.root_source_after_replay {
         None => *root,
         Some(index) => {
             let record = records_in_key_order[index];
--- a/crates/singlefs-harness/tests/journal_last_record_flag_of_a_multi_record_publish.rs
+++ b/crates/singlefs-harness/tests/journal_last_record_flag_of_a_multi_record_publish.rs
@@ -8,8 +8,8 @@
 //!
 //! 历史都一样：A（第一个文件，jsn 3）→ 顺序写两个数据单元的 B（jsn 4、5，txg 4，B 的根落了）→ 顺序写三个数据单元的 C
 //! （jsn 6、7、8，txg 5，事务号 4、5、6），崩在 C 的根槽 FUA 之前：所选根是 B 的，C 的三条记录全落了。
-//! 什么都不改时恢复施加 C（阳性对照）；每条用例只改 B 或 C 的一条记录（两盘各一份，经 `JournalRecord::to_bytes` 重封校验和），
-//! 看恢复施加到哪一版。
+//! 什么都不改时恢复施加 C（阳性对照）；每条用例只改 B 或 C 的一条记录（两盘各一份，经 `JournalRecord::to_bytes` 重封校验和，
+//! 后面记录的反向链跟着重算），看恢复施加到哪一版。
 
 mod common;
 
@@ -21,8 +21,8 @@
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
 use singlefs_core::checksum::wide_checksum_with_field_zeroed;
 use singlefs_core::journal::{
-    record_offset, JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
-    JOURNAL_HEADER_CHECKSUM_OFFSET,
+    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
+    JournalRecordPlaceInPublish, JOURNAL_HEADER_CHECKSUM_OFFSET,
 };
 use singlefs_core::mount::{mount_writable, MountError};
 use singlefs_core::recovery::{
@@ -145,8 +145,11 @@
         .write(record_offset(counter, JOURNAL_RING_DEFAULT_BYTES), bytes);
 }
 
-/// 把 `counter` 那条记录的字段改一处，两盘各一份，`JournalRecord::to_bytes` 重封载荷与头部校验和：
-/// 这条记录照样自证得过，拦住它的只剩被改的那个字段。
+/// 把 `counter` 那条记录的字段改一处，两盘各一份，`JournalRecord::to_bytes` 重封载荷与头部校验和；
+/// 它后面一直到环里第一条解不出的记录，每一条的反向链按前一条改后的记录头重算、重封（D23（journal 的角色与格式） 已定项 8）：
+/// 这条记录照样自证得过、下一条的反向链也接得上，拦住它的只剩被改的那个字段。
+/// 不重算的话，读者删掉被测的那一判之后照样断在下一条的反向链上、结局不变，用例拦不住那一判
+/// （`research/prompts/m3-verus-land-code-r1-opus-output.md` C5-1：`crates/mutations.tsv` 里改这三判的三行套上都不红）。
 fn rewrite_record(image: &mut MemoryPool, counter: u64, change: impl Fn(&mut JournalRecord)) {
     for device in DEVICES {
         let mut record = JournalRecord::parse(
@@ -156,6 +159,16 @@
         .expect("改之前这条记录自证过");
         change(&mut record);
         write_record_bytes(image, device, counter, &record.to_bytes());
+        // 迭代次数的上界是这份历史在 `counter` 之后写过的记录条数（至多 C 的三条）：环里下一格没写过、解不出就是出口。
+        let mut later_counter = counter + 1;
+        while let Some(mut later) = JournalRecord::parse(
+            &record_bytes_on(image, device, later_counter),
+            filesystem_identifier(),
+        ) {
+            later.back_chain = back_chain_of(&record_bytes_on(image, device, later_counter - 1));
+            write_record_bytes(image, device, later_counter, &later.to_bytes());
+            later_counter += 1;
+        }
     }
 }
 
@@ -431,6 +444,36 @@
     );
     let verdict = publish_ordinal_verdict(&image);
     assert!(
+        matches!(&verdict, InvariantVerdict::Violated(detail) if detail.contains("末条之后还有记录")),
+        "checker 的 I-8.9 判「末条之后还有记录」：{verdict:?}"
+    );
+}
+
+/// 同一条读者规则（C540），换一个取样点：C 的末条标志挪到第二条（jsn 7 写上、jsn 8 抹掉，两条都重封校验和），
+/// 带标志的 jsn 7 之后同一个 txg 5 里只剩紧挨着的 jsn 8 ⇒ 断在 jsn 7，C 一条都不施加、读回 B。
+/// 只从「紧挨着的下一条之后」往后找的读者看不到 jsn 8，把 {jsn 6, jsn 7} 当一次发布施加（jsn 7 带着 C 的整个新根段），读回 C；
+/// 上一条用例的取样点（标志在 jsn 6，后面还有 jsn 7、8 两条）拦不住这样的读者
+/// （`research/prompts/m3-verus-land-code-r1-opus-output.md` C5-2）。
+#[test]
+fn last_record_flag_followed_only_by_the_very_next_record_of_the_same_publish_breaks_the_chain_at_the_flag(
+) {
+    let history = crash_before_the_third_root("last-flag-followed-only-by-the-very-next-record");
+    let mut image = history.image.clone();
+    rewrite_record(&mut image, history.third_counters[1], |record| {
+        record.place_in_publish = JournalRecordPlaceInPublish::LastRecordOfThePublish;
+    });
+    rewrite_record(&mut image, history.third_counters[2], |record| {
+        record.place_in_publish = JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow;
+    });
+    assert_recovered_version(
+        &recover(&image, JournalPolicy::Consult),
+        history.second_version,
+        &history.second_content,
+        0,
+        "C 的第二条带末条标志、紧挨着它还有同一次发布的一条：断在 C 的第二条，C 整体不施加",
+    );
+    let verdict = publish_ordinal_verdict(&image);
+    assert!(
         matches!(&verdict, InvariantVerdict::Violated(detail) if detail.contains("末条之后还有记录")),
         "checker 的 I-8.9 判「末条之后还有记录」：{verdict:?}"
     );
--- a/crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs
+++ b/crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs
@@ -77,6 +77,27 @@
     .expect("覆盖写")
 }
 
+/// 同 [`overwrite_in_process`]，这一版的回退下界 F 抬到 `raised_floor`（只改交给写者的上一版的 F，别的照旧）：
+/// 记录新根段里的 F 与所选根的不同，施加出来的根取的是哪一份才分得出来
+/// （D23（journal 的角色与格式） 已定项 15「施加一条记录 = 把所选根的这四个字段换成记录里的」）。
+fn overwrite_in_process_raising_the_floor(
+    pool: &mut BuiltPool,
+    content: &[u8],
+    instance: InstanceGeneration,
+    raised_floor: CheckpointTxg,
+) -> TransactionOutput {
+    let mut previous_with_the_raised_floor = pool.output.clone();
+    previous_with_the_raised_floor.root.rollback_floor = raised_floor;
+    common::publish_overwrite_in_process(
+        pool,
+        &previous_with_the_raised_floor,
+        content,
+        FIXED_WRITE_TIME_SECONDS + 60,
+        instance,
+    )
+    .expect("覆盖写")
+}
+
 /// A → B 之后进程退出、重开并可写挂载。
 fn build_publish_second_version_and_remount(tag: &str) -> (BuiltPool, TransactionOutput, Mounted) {
     let mut pool = build_pool(tag);
@@ -401,13 +422,19 @@
 
 /// 验收第三条：预置一条实例 1 的记录（jsn 5、txg 5、事务号 3，点名的单元也在盘上），把它的根槽改坏 ⇒ 重开那一刻所选根是 B 的根、
 /// 同一个实例 ⇒ 恢复必须施加它，行写 (1, 5, 3)，写行发布的 txg 从 6 起。
+/// 那一版把 F 抬到 4（B 的根带 F = 0）：施加出来的根的 F 取记录新根段里的 4、不是所选根的 0（已定项 15）。
 #[test]
 fn stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row(
 ) {
     let mut pool = build_pool("step-three-stray-record");
     let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
     pool.output = second;
-    let stray = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
+    let stray = overwrite_in_process_raising_the_floor(
+        &mut pool,
+        &third_content(),
+        InstanceGeneration(1),
+        CheckpointTxg(4),
+    );
     assert_eq!(
         (
             stray.root.checkpoint_txg,
@@ -449,6 +476,18 @@
         "施加之后的根是第 5 代"
     );
     assert_eq!(
+        (
+            mounted.output.chosen_root.rollback_floor,
+            stray.record.new_rollback_floor
+        ),
+        (CheckpointTxg(0), CheckpointTxg(4)),
+        "取样点：所选根（B 的根）带 F = 0，那条记录的新根段带抬过的 F = 4"
+    );
+    assert_eq!(
+        mounted.output.effective_root.rollback_floor, stray.record.new_rollback_floor,
+        "施加出来的根的 F 取记录新根段里的那一份，不是所选根的"
+    );
+    assert_eq!(
         mounted.output.rows_written,
         vec![InstanceRow {
             instance: InstanceGeneration(1),
@@ -611,7 +650,13 @@
     let mut pool = build_pool("step-three-torn-anchor");
     pool.output = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
     pool.output = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
-    let fourth = overwrite_in_process(&mut pool, &content_of(3300, 17), InstanceGeneration(1));
+    // 第四版把 F 抬到 5（所选根 (1, 4) 带 F = 0）：只撕锚点那一格施加到 jsn 6，施加出来的根的 F 取 jsn 6 新根段里的 5（已定项 15）。
+    let fourth = overwrite_in_process_raising_the_floor(
+        &mut pool,
+        &content_of(3300, 17),
+        InstanceGeneration(1),
+        CheckpointTxg(5),
+    );
     assert_eq!(fourth.record.counter, 6);
     let mut image = pool.memory_pool();
     for txg in [5u64, 6] {
@@ -730,6 +775,18 @@
                 vec![(CheckpointTxg(6), 4)],
                 "撕掉 {torn:?} 之后写的行"
             );
+            assert_eq!(
+                (
+                    mounted.output.chosen_root.rollback_floor,
+                    fourth.record.new_rollback_floor
+                ),
+                (CheckpointTxg(0), CheckpointTxg(5)),
+                "取样点：所选根 (1, 4) 带 F = 0，jsn 6 的新根段带抬过的 F = 5"
+            );
+            assert_eq!(
+                mounted.output.effective_root.rollback_floor, fourth.record.new_rollback_floor,
+                "施加到 jsn 6：施加出来的根的 F 取它新根段里的那一份，不是所选根的"
+            );
         }
     }
 }
--- a/crates/singlefs-harness/tests/journal_replay_applies_no_more_records_than_the_in_flight_limit.rs
+++ b/crates/singlefs-harness/tests/journal_replay_applies_no_more_records_than_the_in_flight_limit.rs
@@ -0,0 +1,206 @@
+//! 一次恢复重放的前缀最多施加「在飞记录数上限」条记录（D23（journal 的角色与格式） 已定项 18：上限 = 环槽数 ÷ F，
+//! 「这个数的含义是一次恢复重放的前缀最多 N 条」）。
+//!
+//! 来历：Verus 核接回 `crates/` 的代码轮第一轮，攻方 C5-3（`research/prompts/m3-verus-land-code-r1-opus-output.md`）：
+//! 取前缀核里 `taken >= in_flight_limit` 改成 `>`（差一）在全部 harness 目标上不红——默认环的上限是 65536，没有一条用例
+//! 在水位之上摆得出比上限还多的记录。这里把环取 3 槽，上限 = 3 ÷ 3 = 1，所选根之后摆两次单条发布，钉「只施加 1 条」。
+//!
+//! 3 槽的环过不了 mkfs 与读者择系统配置（单元区起点要落在聚簇段边界上，3 槽的环之后那一槽不在边界上：
+//! `recovery::system_configuration_values_this_reader_accepts` 报 `UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`），
+//! 所以走不了 `recover`：池照默认环建，崩溃镜像上把环里的记录按 3 槽环的落点（已定项 18「记录 n 落在环内偏移 (计数器 − 1) mod 槽数 × 4096」）
+//! 搬一份，再拿环长改成 3 槽的那份系统配置扫环、照 `recover` 同一串调 `choose_root`、`scan_journal`、`replay_journal`。
+//! 阳性对照：同一段历史在默认环的原镜像上走 `recover`，两条都施加。
+
+mod common;
+
+use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::journal::record_offset;
+use singlefs_core::recovery::{
+    choose_root, choose_system_configuration, recover, replay_journal, scan_journal, JournalPolicy,
+    PoolReader, RecoveryOutcome,
+};
+use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_core::system_configuration::journal_in_flight_record_limit;
+use singlefs_core::transaction::TransactionOutput;
+use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
+use singlefs_harness::memory_pool::MemoryPool;
+
+const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
+
+/// 3 个记录槽的环：在飞上限 = 3 ÷ F（3）= 1。
+const THREE_RECORD_RING_BYTES: u64 = 3 * JOURNAL_RECORD_BYTES;
+
+fn content_of(length: usize, seed: usize) -> Vec<u8> {
+    (0..length)
+        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
+        .collect()
+}
+
+/// 同一个进程、同一个实例里再发布一版（一次发布一条记录）。
+fn overwrite_in_process(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
+    let previous = pool.output.clone();
+    let output = common::publish_overwrite_in_process(
+        pool,
+        &previous,
+        content,
+        FIXED_WRITE_TIME_SECONDS + 60,
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
+    image.flip_byte(device, slot_offset(target, 4096), 100);
+}
+
+/// 把计数器 `counters` 那几条记录从默认环的落点搬到 3 槽环的落点（两盘各一份；先全部读出再写，搬过去的会盖掉更早的记录）。
+fn image_with_the_records_on_a_three_record_ring(
+    image: &MemoryPool,
+    counters: &[u64],
+) -> MemoryPool {
+    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
+    let mut relocated = image.clone();
+    for device in BOTH_DEVICES {
+        let records: Vec<(u64, Vec<u8>)> = counters
+            .iter()
+            .map(|counter| {
+                let bytes = image
+                    .read(
+                        device,
+                        record_offset(*counter, JOURNAL_RING_DEFAULT_BYTES),
+                        record_bytes,
+                    )
+                    .expect("默认环里那一格读得到");
+                (*counter, bytes)
+            })
+            .collect();
+        for (counter, bytes) in records {
+            relocated
+                .devices
+                .get_mut(&device)
+                .expect("有这块盘")
+                .write(record_offset(counter, THREE_RECORD_RING_BYTES), &bytes);
+        }
+    }
+    relocated
+}
+
+/// A（txg 3、jsn 3）的根落了；之后 B（txg 4、jsn 4）、C（txg 5、jsn 5）两次单条发布的记录与单元都在、根槽都坏了 ⇒ 所选根是 A 的，
+/// 水位之上是两次整的发布。在飞上限 1 ⇒ 只施加 jsn 4、重建到 B；上限按 `>` 判（多施加一条）的读者把 jsn 5 也施加、重建到 C。
+#[test]
+fn with_an_in_flight_limit_of_one_only_the_first_of_two_single_record_publishes_above_the_chosen_root_is_applied(
+) {
+    let mut pool = build_pool("in-flight-limit-of-one");
+    let first_version = (pool.output.root.instance, pool.output.root.checkpoint_txg);
+    let second = overwrite_in_process(&mut pool, &content_of(4100, 3));
+    let third_content = content_of(2500, 11);
+    let third = overwrite_in_process(&mut pool, &third_content);
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
+
+    // 阳性对照：默认环（上限 65536）上同一段历史，B、C 两次发布都施加、读回 C。
+    let report_on_the_default_ring = recover(&image, JournalPolicy::Consult);
+    assert_eq!(
+        (
+            report_on_the_default_ring.outcome,
+            report_on_the_default_ring.effective_root,
+            report_on_the_default_ring.journal.above_water,
+            report_on_the_default_ring.journal.prefix_applied
+        ),
+        (
+            RecoveryOutcome::FileRead {
+                root: first_version,
+                content: third_content
+            },
+            Some((InstanceGeneration(1), CheckpointTxg(5))),
+            2,
+            2
+        ),
+        "默认环：所选根是 A 的，水位之上两条都施加"
+    );
+
+    // 3 槽的环：上限 1。
+    assert_eq!(
+        journal_in_flight_record_limit(THREE_RECORD_RING_BYTES),
+        1,
+        "3 槽 ÷ F（3）= 1"
+    );
+    let relocated = image_with_the_records_on_a_three_record_ring(&image, &[3, 4, 5]);
+    let system_configuration =
+        choose_system_configuration(&relocated).expect("系统配置择得出（它记着的是默认环）");
+    let mut three_record_ring_configuration = system_configuration;
+    three_record_ring_configuration
+        .immutable
+        .sizes
+        .journal_ring_bytes = THREE_RECORD_RING_BYTES;
+    let chosen_root = choose_root(&relocated, &system_configuration).expect("择得出根");
+    assert_eq!(
+        (chosen_root.instance, chosen_root.checkpoint_txg),
+        first_version,
+        "B、C 的根槽都坏了：所选根是 A 的"
+    );
+    let records = scan_journal(&relocated, &three_record_ring_configuration);
+    assert_eq!(
+        records
+            .keys()
+            .map(|(_, counter)| *counter)
+            .collect::<Vec<_>>(),
+        [3, 4, 5],
+        "3 槽的环里扫到 jsn 3、4、5"
+    );
+    let (report, rebuilt) = replay_journal(
+        &relocated,
+        &chosen_root,
+        THREE_RECORD_RING_BYTES,
+        &records,
+        true,
+    )
+    .expect("所选根那次发布只有 jsn 3 带末条标志：锚点认得出");
+    assert_eq!(
+        (
+            report.valid_records,
+            report.above_water,
+            report.prefix_applied,
+            report.verification_passed
+        ),
+        (3, 2, 1, 1),
+        "水位之上两条，上限 1：只施加 jsn 4，它的点名单元验过"
+    );
+    assert_eq!(
+        (
+            rebuilt.instance,
+            rebuilt.checkpoint_txg,
+            rebuilt.mapping_root
+        ),
+        (
+            second.root.instance,
+            second.root.checkpoint_txg,
+            second.root.mapping_root
+        ),
+        "重建到 B，不是 C"
+    );
+}
--- a/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs
+++ b/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs
@@ -0,0 +1,180 @@
+//! 取前缀的核（`singlefs_replay_prefix::replay_core`）报「所选根那次发布带末条标志的多于一条」时，`replay_journal` 拿核带出的计数器
+//! 直接拼 `RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided` 返回，不再调一份手抄的筛选去重算、
+//! 也不在两份筛选对不上时 panic（代码轮判决 `research/prompts/m3-verus-land-code-r1-main-verification.md` 第二节 C4 那一行，G6）。
+//!
+//! 压着的条款：D23（journal 的角色与格式） 已定项 14 注 1（锚点 = 与所选根同实例、同 checkpoint_txg、带末条标志的那一条，读法乙）
+//! 与「这一版的失败处置」（同一 (实例代号, checkpoint_txg) 两条带末条标志就是盘坏了，在任何写之前停下）。
+//!
+//! 记录表是手造的：一块盘都没有的池当读者（反向链判不了、不拿它断前缀；记录不点名单元，点名验证恒过），
+//! 只看取前缀那一截怎么认锚点。同实例里所选根之前、之后两次发布各带一条末条标志：它们不属于所选根那次发布，
+//! 不进报错的计数器清单，也不让恢复停下。
+
+use std::collections::BTreeMap;
+
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::journal::{
+    JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
+};
+use singlefs_core::pointer::NodePointer;
+use singlefs_core::recovery::{replay_journal, RecoveryFailure};
+use singlefs_core::root_record::{RootRecord, UnmountMarker};
+use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
+use singlefs_harness::memory_pool::MemoryPool;
+
+const INSTANCE: InstanceGeneration = InstanceGeneration(1);
+const CHOSEN_ROOT_CHECKPOINT_TXG: CheckpointTxg = CheckpointTxg(2);
+/// 记录与根共用的 fsid 低 8 字节；读者是空池，不拿它比。
+const FILESYSTEM_IDENTIFIER_LOW_BYTES: u64 = 0x5349_4E47_4C45_4653;
+
+fn chosen_root() -> RootRecord {
+    RootRecord {
+        filesystem_identifier: [0x53; 16],
+        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+        instance: INSTANCE,
+        checkpoint_txg: CHOSEN_ROOT_CHECKPOINT_TXG,
+        tree_table: NodePointer::empty_root(),
+        tree_identifier_watermark: 0,
+        rollback_floor: CheckpointTxg(0),
+        instance_table: NodePointer::empty_root(),
+        mapping_root: NodePointer::empty_root(),
+        allocation_record_tree_root: NodePointer::empty_root(),
+    }
+}
+
+/// 一次只有一条记录的发布：序号 1、带末条标志与提交标记，事务号取计数器。
+fn single_record_publish(counter: u64, checkpoint_txg: CheckpointTxg) -> JournalRecord {
+    JournalRecord {
+        instance: INSTANCE,
+        counter,
+        checkpoint_txg,
+        transaction: counter,
+        is_commit: true,
+        ordinal_within_publish: JournalRecordOrdinalWithinPublish(1),
+        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
+        back_chain: 0,
+        filesystem_identifier: FILESYSTEM_IDENTIFIER_LOW_BYTES,
+        new_tree_table: NodePointer::empty_root(),
+        new_mapping_root: NodePointer::empty_root(),
+        new_tree_identifier_watermark: 0,
+        new_rollback_floor: CheckpointTxg(0),
+        named: Vec::new(),
+    }
+}
+
+fn table_of(records: Vec<JournalRecord>) -> BTreeMap<(InstanceGeneration, u64), JournalRecord> {
+    records
+        .into_iter()
+        .map(|record| ((record.instance, record.counter), record))
+        .collect()
+}
+
+fn reader_with_no_device() -> MemoryPool {
+    MemoryPool::with_devices(&[], 0)
+}
+
+/// jsn 1（txg 1）、jsn 2 与 jsn 3（都是所选根的 txg 2）、jsn 4（txg 3）各带末条标志：所选根那次发布带标志的是 jsn 2、3 两条，
+/// 锚点认哪一条条款没写 ⇒ 报错，计数器只列这两条（另外两条不在所选根那次发布里）；不 panic。
+#[test]
+fn two_last_record_flags_in_the_chosen_roots_publish_return_the_error_with_only_that_publishs_counters(
+) {
+    let records = table_of(vec![
+        single_record_publish(1, CheckpointTxg(1)),
+        single_record_publish(2, CHOSEN_ROOT_CHECKPOINT_TXG),
+        single_record_publish(3, CHOSEN_ROOT_CHECKPOINT_TXG),
+        single_record_publish(4, CheckpointTxg(CHOSEN_ROOT_CHECKPOINT_TXG.0 + 1)),
+    ]);
+    let replayed = replay_journal(
+        &reader_with_no_device(),
+        &chosen_root(),
+        JOURNAL_RING_DEFAULT_BYTES,
+        &records,
+        true,
+    );
+    assert_eq!(
+        replayed,
+        Err(
+            RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                instance: INSTANCE,
+                checkpoint_txg: CHOSEN_ROOT_CHECKPOINT_TXG,
+                counters: vec![2, 3],
+            }
+        ),
+        "报错带的是所选根那次发布里带末条标志的两条（jsn 2、3），txg 1 与 txg 3 那两条不算"
+    );
+}
+
+/// 同实例里所选根之前（jsn 1，txg 1）、之后（jsn 3，txg 3）的发布各带末条标志，所选根那次发布只有 jsn 2 带：
+/// 锚点是 jsn 2，链首接 jsn 3，txg 3 那次发布整次施加。别的发布的末条标志不让恢复停下。
+#[test]
+fn last_record_flags_of_other_publishes_of_the_same_instance_leave_the_anchor_alone_and_the_next_publish_applies(
+) {
+    let next_publish_checkpoint_txg = CheckpointTxg(CHOSEN_ROOT_CHECKPOINT_TXG.0 + 1);
+    let records = table_of(vec![
+        single_record_publish(1, CheckpointTxg(1)),
+        single_record_publish(2, CHOSEN_ROOT_CHECKPOINT_TXG),
+        single_record_publish(3, next_publish_checkpoint_txg),
+    ]);
+    let (report, rebuilt_root) = replay_journal(
+        &reader_with_no_device(),
+        &chosen_root(),
+        JOURNAL_RING_DEFAULT_BYTES,
+        &records,
+        true,
+    )
+    .expect("所选根那次发布只有 jsn 2 带末条标志：锚点认得出");
+    assert_eq!(
+        (report.above_water, report.prefix_applied),
+        (1, 1),
+        "水位之上只有 jsn 3，它接在锚点 jsn 2 之后、整次施加"
+    );
+    assert_eq!(
+        (rebuilt_root.instance, rebuilt_root.checkpoint_txg),
+        (INSTANCE, next_publish_checkpoint_txg),
+        "重建出来的根是 txg 3 那一版"
+    );
+}
+
+/// 核按切片次序收记录、前置条件是「按 (实例代号, 计数器) 严格升序」；胶水按记录表的键序交给它，所以键与记录自己的
+/// (实例代号, 计数器) 不等时，胶水在调核之前断言失败，不带着次序不对的切片往下取前缀（判决同上 C1 那一行，G2）。
+/// 造法：jsn 2、jsn 3 两条的键对调（今天没有调用方这样建表）。
+#[test]
+#[should_panic(expected = "记录表的键要等于记录自己的 (实例代号, 计数器)")]
+fn a_table_whose_keys_disagree_with_its_records_trips_the_glue_assertion_before_the_kernel_runs() {
+    let mut records = table_of(vec![single_record_publish(1, CHOSEN_ROOT_CHECKPOINT_TXG)]);
+    let next_publish_checkpoint_txg = CheckpointTxg(CHOSEN_ROOT_CHECKPOINT_TXG.0 + 1);
+    records.insert(
+        (INSTANCE, 3),
+        single_record_publish(2, next_publish_checkpoint_txg),
+    );
+    records.insert(
+        (INSTANCE, 2),
+        single_record_publish(3, next_publish_checkpoint_txg),
+    );
+    let replayed = replay_journal(
+        &reader_with_no_device(),
+        &chosen_root(),
+        JOURNAL_RING_DEFAULT_BYTES,
+        &records,
+        true,
+    );
+    panic!("键与记录不等的表没被胶水的断言拦下，取前缀交回了 {replayed:?}");
+}
+
+/// 核要给链上下一号加一，前置条件是计数器都不是 u64 的最大值；盘上的计数器是 6 字节，产品路径到不了，
+/// 直接造表的调用方给了这个值时，胶水在调核之前断言失败。
+#[test]
+#[should_panic(expected = "计数器是盘上 6 字节读来的")]
+fn a_record_whose_counter_is_the_largest_value_trips_the_glue_assertion_before_the_kernel_runs() {
+    let records = table_of(vec![single_record_publish(
+        u64::MAX,
+        CheckpointTxg(CHOSEN_ROOT_CHECKPOINT_TXG.0 + 1),
+    )]);
+    let replayed = replay_journal(
+        &reader_with_no_device(),
+        &chosen_root(),
+        JOURNAL_RING_DEFAULT_BYTES,
+        &records,
+        true,
+    );
+    panic!("计数器是 u64 最大值的记录没被胶水的断言拦下，取前缀交回了 {replayed:?}");
+}
--- a/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
+++ b/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
@@ -22,14 +22,20 @@
 //! 单状态重放（登记修订第 12 条）：`--replay-state <域>:<N>:<序号>` 只判枚举次序里的那一个状态，打一行 `name=state`
 //! （这个状态让哪几个量加了一、实现的结局、状态的写法）；`--list-states <域>:<N>` 把这个 (域, N) 的每个状态照枚举次序各打一行。
 //! 两种都单线程照工作单元的次序走，不跑自检与阳性对照，判据与正常跑是同一段代码。域名用产物里 `domain=` 的写法。
+//!
+//! 钉住的红例逐行重放：`--pinned-cases <表>` 读 `.claude/gate.d/e165-pinned-red-examples.tsv` 那种表（格式在它的文件头），
+//! 按数据行的次序每行判一个状态、打一行 `name=state`（与 `--replay-state` 打的那一行是同一段代码）；按 ②③④ 找到状态之后
+//! 拿它重算 ①（`sha256(<域>|<N>|<state= 的值，截到 ;first= 之前>)`），与表里不同、或有行找不到状态、或行本身不对，退 1。
+//! 逐态行与 ⑦ 比不比得上由读这张表的一方（`research/scripts/e165-pin-red-examples.py --replay`）判，装置不判。
 
 use std::cell::Cell;
-use std::collections::BTreeMap;
+use std::collections::{btree_map, BTreeMap};
 use std::env;
+use std::fs;
 use std::ops::Range;
 use std::panic::{self, AssertUnwindSafe};
-use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
-use std::sync::{Mutex, OnceLock};
+use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
+use std::sync::Mutex;
 use std::time::Instant;
 
 use singlefs_core::address::{
@@ -47,6 +53,7 @@
 };
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
 use singlefs_core::system_configuration::journal_in_flight_record_limit;
+use singlefs_harness::sha256::sha256_hexadecimal;
 
 // ── 本地常量：值抄自 kb，不从 `crates/` 引；[`cross_check_local_constants_against_the_implementation`] 逐个回比（登记第三节、停机条款 H2）。
 
@@ -185,7 +192,7 @@
         let is_a_whole_record_slot_of_the_ring = device.0 < 2
             && offset.0 >= ring_start
             && offset.0 < ring_start + self.ring.ring_bytes()
-            && (offset.0 - ring_start) % LOCAL_JOURNAL_RECORD_BYTES == 0
+            && (offset.0 - ring_start).is_multiple_of(LOCAL_JOURNAL_RECORD_BYTES)
             && u64::try_from(length).ok() == Some(LOCAL_JOURNAL_RECORD_BYTES);
         is_a_whole_record_slot_of_the_ring.then(|| vec![0u8; length])
     }
@@ -778,9 +785,10 @@
             StoredBytes::Readable(&residue.slot.bytes),
         ));
         if let Some(parsed) = residue.slot.parsed.as_ref() {
-            let key = (parsed.instance, parsed.counter);
-            if !records.contains_key(&key) {
-                records.insert(key, parsed.clone());
+            if let btree_map::Entry::Vacant(vacant_place) =
+                records.entry((parsed.instance, parsed.counter))
+            {
+                vacant_place.insert(parsed.clone());
                 truths.push(RecordTruthOnDisk {
                     instance: parsed.instance.0,
                     counter: parsed.counter,
@@ -889,6 +897,10 @@
     verification_passed: usize,
 }
 
+#[allow(
+    clippy::large_enum_variant,
+    reason = "每个状态一两个结局、按值比较拷来拷去要 Copy；装箱就丢了 Copy，每个状态还多一次分配"
+)]
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum ReplayOutcome {
     Panicked,
@@ -1424,18 +1436,43 @@
 /// 这一次跑只看哪些状态。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum StateProbe {
-    /// 只判枚举次序里序号等于它的那一个状态，别的状态跳过不判。
+    /// 只判枚举次序里序号等于它的那一个状态，别的状态跳过不判；打出那一行就收尾退出。
     ReplayOneState { ordinal: u64 },
     /// 每个状态都判，各打一行。
     ListEveryState,
+    /// 只判序号等于它的那一个状态，别的状态跳过不判；那一行记进 [`CAPTURED_STATE_ROW`]，不打、不退（钉住的红例逐行重放用）。
+    CaptureOneStateRow { ordinal: u64 },
 }
 
-/// 探针要的 (域, N) 与模式；没设就是正常跑，两个判定函数里的探针什么都不做。
-static STATE_PROBE: OnceLock<(Domain, usize, StateProbe)> = OnceLock::new();
+/// 探针开没开：正常跑时每个状态只读这一个原子量，不碰 [`STATE_PROBE`] 那把锁。
+static STATE_PROBE_IS_SET: AtomicBool = AtomicBool::new(false);
+/// 探针要的 (域, N) 与模式；没设就是正常跑，两个判定函数里的探针什么都不做。钉住的红例逐行重放时每一行换一次。
+static STATE_PROBE: Mutex<Option<(Domain, usize, StateProbe)>> = Mutex::new(None);
 /// 探针模式下已经走过的状态数：单线程照工作单元的次序走，它就是状态在枚举次序里的序号。
 static NEXT_STATE_ORDINAL: AtomicU64 = AtomicU64::new(0);
 /// 探针模式下打出去的 `name=state` 行数。
 static STATE_ROWS_EMITTED: AtomicU64 = AtomicU64::new(0);
+/// `CaptureOneStateRow` 记下的那一行（不带换行）；设探针时清空。
+static CAPTURED_STATE_ROW: Mutex<Option<String>> = Mutex::new(None);
+
+/// 设探针：(域, N) 与模式换成新的，走过的序号、打出去的行数、记下的那一行都从头算。
+fn set_the_state_probe(domain: Domain, size: usize, probe: StateProbe) {
+    *STATE_PROBE.lock().expect("拿着探针那把锁的地方都不 panic") = Some((domain, size, probe));
+    NEXT_STATE_ORDINAL.store(0, Ordering::Relaxed);
+    STATE_ROWS_EMITTED.store(0, Ordering::Relaxed);
+    *CAPTURED_STATE_ROW
+        .lock()
+        .expect("拿着记下那一行的锁的地方都不 panic") = None;
+    STATE_PROBE_IS_SET.store(true, Ordering::Release);
+}
+
+/// 此刻的探针；没设交 `None`。
+fn current_state_probe() -> Option<(Domain, usize, StateProbe)> {
+    if !STATE_PROBE_IS_SET.load(Ordering::Acquire) {
+        return None;
+    }
+    *STATE_PROBE.lock().expect("拿着探针那把锁的地方都不 panic")
+}
 
 /// 一个状态开判之前探针的决定。
 enum ProbeDecision {
@@ -1448,20 +1485,23 @@
 }
 
 fn begin_probing_a_state(tally: &Tally) -> ProbeDecision {
-    let Some((_domain, _size, probe)) = STATE_PROBE.get() else {
+    let Some((_domain, _size, probe)) = current_state_probe() else {
         return ProbeDecision::NotProbing;
     };
     let ordinal = NEXT_STATE_ORDINAL.fetch_add(1, Ordering::Relaxed);
     match probe {
-        StateProbe::ReplayOneState { ordinal: wanted } if *wanted != ordinal => {
+        StateProbe::ReplayOneState { ordinal: wanted }
+        | StateProbe::CaptureOneStateRow { ordinal: wanted }
+            if wanted != ordinal =>
+        {
             ProbeDecision::SkipThisState
         }
-        StateProbe::ReplayOneState { .. } | StateProbe::ListEveryState => {
-            ProbeDecision::ReportThisState {
-                ordinal,
-                counts_before: tally.counts.clone(),
-            }
-        }
+        StateProbe::ReplayOneState { .. }
+        | StateProbe::ListEveryState
+        | StateProbe::CaptureOneStateRow { .. } => ProbeDecision::ReportThisState {
+            ordinal,
+            counts_before: tally.counts.clone(),
+        },
     }
 }
 
@@ -1483,7 +1523,7 @@
     }
 }
 
-/// 一个状态判完之后：探针要报它的，打一行 `name=state`。重放模式下报完这一个就收尾退出。
+/// 一个状态判完之后：探针要报它的，打一行 `name=state`（记下模式只记不打）。重放模式下报完这一个就收尾退出。
 fn finish_probing_a_state(
     decision: ProbeDecision,
     tally: &Tally,
@@ -1496,8 +1536,8 @@
     else {
         return;
     };
-    let (domain, size, probe) = STATE_PROBE.get().expect("探针报一个状态时探针必然设过");
-    println!(
+    let (domain, size, probe) = current_state_probe().expect("探针报一个状态时探针必然设过");
+    let state_row = format!(
         "E7RESULT name=state domain={} ring_slots={} n={size} ordinal={ordinal} counters={} state={}",
         domain.label(),
         domain.ring().ring_slots,
@@ -1505,13 +1545,47 @@
         describe_state()
     );
     STATE_ROWS_EMITTED.fetch_add(1, Ordering::Relaxed);
-    if matches!(probe, StateProbe::ReplayOneState { .. }) {
-        // 配置行一行、状态行一行、收尾行一行。
-        println!("E7RESULT name=done emitted=3");
-        std::process::exit(0);
+    match probe {
+        StateProbe::ReplayOneState { .. } => {
+            println!("{state_row}");
+            // 配置行一行、状态行一行、收尾行一行。
+            println!("E7RESULT name=done emitted=3");
+            std::process::exit(0);
+        }
+        StateProbe::ListEveryState => println!("{state_row}"),
+        StateProbe::CaptureOneStateRow { .. } => {
+            let earlier_row = CAPTURED_STATE_ROW
+                .lock()
+                .expect("拿着记下那一行的锁的地方都不 panic")
+                .replace(state_row);
+            assert!(
+                earlier_row.is_none(),
+                "一个序号只对应枚举次序里的一个状态，记下模式一次设探针只记一行"
+            );
+        }
     }
 }
 
+/// 域名与 N 的原文 → (域, N)：域名用产物里 `domain=` 的写法，N 要在这个域最小的 N 到 12 之间。
+fn domain_and_size_of(domain_text: &str, size_text: &str) -> Result<(Domain, usize), String> {
+    let domain = Domain::ALL
+        .iter()
+        .copied()
+        .find(|domain| domain.label() == domain_text)
+        .ok_or_else(|| format!("不认识的域：{domain_text}"))?;
+    let size = size_text
+        .parse::<usize>()
+        .map_err(|error| format!("N 要一个非负整数：{error}"))?;
+    if size < domain.smallest_size() || size > 12 {
+        return Err(format!(
+            "{} 域的 N 要在 {} 到 12 之间，收到的是 {size}",
+            domain.label(),
+            domain.smallest_size()
+        ));
+    }
+    Ok((domain, size))
+}
+
 /// `<域>:<N>:<序号>` 或 `<域>:<N>`：域名用产物里 `domain=` 的写法（U 域是 `U_verify_on`、`U_verify_off`）。
 fn parse_the_state_probe(
     arguments: &[String],
@@ -1531,21 +1605,7 @@
             "{option} 要 {expected_parts} 段（冒号分隔），收到的是「{value}」"
         ));
     }
-    let domain = Domain::ALL
-        .iter()
-        .copied()
-        .find(|domain| domain.label() == parts[0])
-        .ok_or_else(|| format!("不认识的域：{}", parts[0]))?;
-    let size = parts[1]
-        .parse::<usize>()
-        .map_err(|error| format!("N 要一个非负整数：{error}"))?;
-    if size < domain.smallest_size() || size > 12 {
-        return Err(format!(
-            "{} 域的 N 要在 {} 到 12 之间，收到的是 {size}",
-            domain.label(),
-            domain.smallest_size()
-        ));
-    }
+    let (domain, size) = domain_and_size_of(parts[0], parts[1])?;
     let probe = if wants_one_state {
         StateProbe::ReplayOneState {
             ordinal: parts[2]
@@ -1566,18 +1626,29 @@
         match probe {
             StateProbe::ReplayOneState { .. } => "replay_state",
             StateProbe::ListEveryState => "list_states",
+            StateProbe::CaptureOneStateRow { .. } => "capture_state",
         },
         domain.label(),
         domain.ring().ring_slots
     );
-    STATE_PROBE
-        .set((domain, size, probe))
-        .expect("探针只在主流程开头设一次");
+    set_the_state_probe(domain, size, probe);
     let tally = tally_of_a_domain_size(domain, size, Implementation::TodayReplayJournal, 1);
     let walked = NEXT_STATE_ORDINAL.load(Ordering::Relaxed);
-    match probe {
-        StateProbe::ReplayOneState { ordinal } => {
-            // 走到这里说明那个序号一直没遇到（遇到的那一刻已经退出）。
+    let captured_state_row = CAPTURED_STATE_ROW
+        .lock()
+        .expect("拿着记下那一行的锁的地方都不 panic")
+        .take();
+    match (probe, captured_state_row) {
+        (StateProbe::CaptureOneStateRow { .. }, Some(state_row)) => {
+            // 记下模式走完整个 (域, N) 才回到这里；与重放模式打同样的三行。
+            println!("{state_row}");
+            println!("E7RESULT name=done emitted=3");
+        }
+        (
+            StateProbe::ReplayOneState { ordinal } | StateProbe::CaptureOneStateRow { ordinal },
+            None,
+        ) => {
+            // 走到这里说明那个序号一直没遇到（重放模式遇到的那一刻已经退出）。
             eprintln!(
                 "  ✗ {} 域 N = {size} 只有 {walked} 个状态，序号 {ordinal} 不在里面",
                 domain.label()
@@ -1588,7 +1659,10 @@
             );
             std::process::exit(2);
         }
-        StateProbe::ListEveryState => {
+        (StateProbe::ReplayOneState { .. } | StateProbe::ListEveryState, Some(_state_row)) => {
+            unreachable!("只有记下模式往 CAPTURED_STATE_ROW 里放行，设探针时清空过")
+        }
+        (StateProbe::ListEveryState, None) => {
             let rows = STATE_ROWS_EMITTED.load(Ordering::Relaxed);
             assert_eq!(rows, walked, "每个状态恰好一行");
             assert_eq!(
@@ -1601,6 +1675,210 @@
     }
 }
 
+// ── 钉住的红例逐行重放（`.claude/gate.d/e165-pinned-red-examples.tsv` 文件头的约定）：只按 ②③④ 找状态、重算 ①，不改任何判据
+
+/// 钉住的表一行有几段（① 输入指纹、② 域、③ N、④ 序号、⑤ 出处、⑥ 变异、⑦ 原实现的逐态行、⑧ 变体的逐态行）。
+const PINNED_TABLE_COLUMN_COUNT: usize = 8;
+
+/// 钉住的表里的一行数据：① 与 ②③ 留原文（重算 ① 用原文），②③④ 另解出值来找状态。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct PinnedRedExample {
+    line_number_in_the_table: usize,
+    input_fingerprint: String,
+    domain_text: String,
+    size_text: String,
+    domain: Domain,
+    size: usize,
+    ordinal: u64,
+}
+
+/// 表里一行数据 → [`PinnedRedExample`]：段数不是八、域不认识、N 不在这个域的范围、序号不是非负整数，都拒。
+fn parse_a_pinned_red_example(
+    line_number_in_the_table: usize,
+    line: &str,
+) -> Result<PinnedRedExample, String> {
+    let columns: Vec<&str> = line.split('\t').collect();
+    if columns.len() != PINNED_TABLE_COLUMN_COUNT {
+        return Err(format!(
+            "要 {PINNED_TABLE_COLUMN_COUNT} 段（制表符分隔），这一行是 {} 段",
+            columns.len()
+        ));
+    }
+    let (domain, size) = domain_and_size_of(columns[1], columns[2])?;
+    let ordinal = columns[3]
+        .parse::<u64>()
+        .map_err(|error| format!("④ 序号要一个非负整数：{error}"))?;
+    Ok(PinnedRedExample {
+        line_number_in_the_table,
+        input_fingerprint: columns[0].to_string(),
+        domain_text: columns[1].to_string(),
+        size_text: columns[2].to_string(),
+        domain,
+        size,
+        ordinal,
+    })
+}
+
+/// 输入指纹：`sha256(<域>|<N>|<逐态行里 state= 的值，截到 ;first= 之前>)` 的十六进制，与
+/// `research/scripts/e165-pin-red-examples.py` 的 `input_fingerprint` 同一个算法。逐态行里认不出 ` state=<不带空白的值>` 收尾交 `None`。
+fn input_fingerprint_of(domain_text: &str, size_text: &str, state_row: &str) -> Option<String> {
+    let (_row_before_the_state, state_text) = state_row.rsplit_once(" state=")?;
+    if state_text.is_empty() || state_text.chars().any(char::is_whitespace) {
+        return None;
+    }
+    let state_before_the_outcome = state_text
+        .split_once(";first=")
+        .map_or(state_text, |(before_the_outcome, _outcome)| {
+            before_the_outcome
+        });
+    Some(sha256_hexadecimal(
+        format!("{domain_text}|{size_text}|{state_before_the_outcome}").as_bytes(),
+    ))
+}
+
+/// 一行钉住的红例在今天的装置上重放出来的样子。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum PinnedRedExampleReplay {
+    /// 找到了状态，拿它的逐态行重算的指纹与 ① 相同。
+    FingerprintMatches { state_row: String },
+    /// 找到了状态，重算的指纹与 ① 不同：枚举次序或状态编码漂了。
+    FingerprintDrifted {
+        state_row: String,
+        recomputed_fingerprint: String,
+    },
+    /// 找到了状态，逐态行里认不出 `state=` 那一段，算不出指纹。
+    StateRowUnrecognized { state_row: String },
+    /// 这个 (域, N) 走完了也没遇到这个序号。
+    OrdinalOutsideTheDomainSize { states_walked: u64 },
+}
+
+/// 重放出来的逐态行（没遇到序号是 `None`）对着表里那一行判：纯函数，不碰探针。
+fn judge_a_pinned_red_example_replay(
+    example: &PinnedRedExample,
+    captured_state_row: Option<String>,
+    states_walked: u64,
+) -> PinnedRedExampleReplay {
+    let Some(state_row) = captured_state_row else {
+        return PinnedRedExampleReplay::OrdinalOutsideTheDomainSize { states_walked };
+    };
+    match input_fingerprint_of(&example.domain_text, &example.size_text, &state_row) {
+        None => PinnedRedExampleReplay::StateRowUnrecognized { state_row },
+        Some(recomputed_fingerprint) if recomputed_fingerprint == example.input_fingerprint => {
+            PinnedRedExampleReplay::FingerprintMatches { state_row }
+        }
+        Some(recomputed_fingerprint) => PinnedRedExampleReplay::FingerprintDrifted {
+            state_row,
+            recomputed_fingerprint,
+        },
+    }
+}
+
+/// 在今天的实现上重放一行：设记下模式的探针，单线程把这个 (域, N) 走一遍，交回记下的那一行与走过的状态数。
+fn replay_a_pinned_red_example(example: &PinnedRedExample) -> (Option<String>, u64) {
+    set_the_state_probe(
+        example.domain,
+        example.size,
+        StateProbe::CaptureOneStateRow {
+            ordinal: example.ordinal,
+        },
+    );
+    let _tally_of_the_one_judged_state = tally_of_a_domain_size(
+        example.domain,
+        example.size,
+        Implementation::TodayReplayJournal,
+        1,
+    );
+    let states_walked = NEXT_STATE_ORDINAL.load(Ordering::Relaxed);
+    let captured_state_row = CAPTURED_STATE_ROW
+        .lock()
+        .expect("拿着记下那一行的锁的地方都不 panic")
+        .take();
+    (captured_state_row, states_walked)
+}
+
+/// `--pinned-cases <表>` 的主流程：按数据行的次序每行打一行 `name=state`；有行不对、找不到状态或指纹漂了，全部报完退 1。
+fn replay_every_pinned_red_example(table_path: &str) {
+    let table_text = match fs::read_to_string(table_path) {
+        Ok(table_text) => table_text,
+        Err(error) => {
+            eprintln!("  ✗ 读不了钉住的表 {table_path}：{error}");
+            eprintln!("  → 怎么办：{USAGE}");
+            std::process::exit(2);
+        }
+    };
+    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
+    let data_lines: Vec<(usize, &str)> = table_text
+        .lines()
+        .enumerate()
+        .map(|(line_index, line)| (line_index + 1, line))
+        .filter(|(_line_number, line)| !line.trim().is_empty() && !line.starts_with('#'))
+        .collect();
+    println!(
+        "E7RESULT name=config experiment=E165 mode=pinned_cases rows={} self_checks=not_run positive_controls=not_run threads=1",
+        data_lines.len()
+    );
+    let mut problems: Vec<String> = Vec::new();
+    if data_lines.is_empty() {
+        problems.push(format!("{table_path} 一行数据都没有"));
+    }
+    let mut state_rows_printed: u64 = 0;
+    for (line_number, line) in data_lines {
+        let example = match parse_a_pinned_red_example(line_number, line) {
+            Ok(example) => example,
+            Err(problem) => {
+                problems.push(format!("第 {line_number} 行：{problem}"));
+                continue;
+            }
+        };
+        let (captured_state_row, states_walked) = replay_a_pinned_red_example(&example);
+        let place = format!(
+            "第 {} 行（{}:{}:{}）",
+            example.line_number_in_the_table,
+            example.domain_text,
+            example.size_text,
+            example.ordinal
+        );
+        match judge_a_pinned_red_example_replay(&example, captured_state_row, states_walked) {
+            PinnedRedExampleReplay::FingerprintMatches { state_row } => {
+                println!("{state_row}");
+                state_rows_printed += 1;
+            }
+            PinnedRedExampleReplay::FingerprintDrifted {
+                state_row,
+                recomputed_fingerprint,
+            } => {
+                println!("{state_row}");
+                state_rows_printed += 1;
+                problems.push(format!(
+                    "{place}：拿这个状态重算的指纹是 {recomputed_fingerprint}，表里 ① 是 {}",
+                    example.input_fingerprint
+                ));
+            }
+            PinnedRedExampleReplay::StateRowUnrecognized { state_row } => {
+                println!("{state_row}");
+                state_rows_printed += 1;
+                problems.push(format!("{place}：逐态行里认不出 state= 那一段"));
+            }
+            PinnedRedExampleReplay::OrdinalOutsideTheDomainSize { states_walked } => {
+                problems.push(format!(
+                    "{place}：这个 (域, N) 只有 {states_walked} 个状态，序号不在里面"
+                ));
+            }
+        }
+    }
+    // 配置行一行、逐态行若干、收尾行一行。
+    println!("E7RESULT name=done emitted={}", state_rows_printed + 2);
+    if !problems.is_empty() {
+        for problem in &problems {
+            eprintln!("  ✗ {problem}");
+        }
+        eprintln!(
+            "  → 怎么办：指纹漂了先查装置的枚举次序与状态编码变没变；行不对的照表头的八段格式找回原行（表只由 research/scripts/e165-pin-red-examples.py 追加）"
+        );
+        std::process::exit(1);
+    }
+}
+
 /// 把工作单元分给几条线程跑，再按工作单元的次序合并：计数是加法，见证取次序里最先的，结果与线程数、线程次序无关。
 fn tally_of_work_units<Unit: Sync>(
     units: &[Unit],
@@ -2525,10 +2803,12 @@
     let verify_named_units = domain != Domain::NamedUnitsNotVerified;
     let mut tally = Tally::new();
     let mut presence = vec![RecordPresence::NotOnDisk; record_count];
-    let judge = |presence: &[RecordPresence], is_the_first_state: bool, tally: &mut Tally| {
+    let judge = |presence_of_the_state: &[RecordPresence],
+                 is_the_first_state: bool,
+                 tally_of_the_unit: &mut Tally| {
         let state = LegalState {
             history: &history,
-            presence,
+            presence: presence_of_the_state,
             residue: residue.as_ref(),
             chosen_version: 0,
             verify_named_units,
@@ -2538,7 +2818,7 @@
             implementation,
             domain.specification_self_check(),
             unit_index == 0 && is_the_first_state && record_count == 1,
-            tally,
+            tally_of_the_unit,
         );
     };
     if unit.plan.names_units {
@@ -4670,7 +4950,7 @@
     family_label: &'static str,
 }
 
-const USAGE: &str = "用法：e165_replay_bounded_enumeration --bounds segment1|segment2|light [--family legal|adversarial|all] [--only <域>[,<域>…]] [--max-n <N>]；域名：L L6 L9 U C C6 A A6 AC X E1 E2 R。单状态重放：--replay-state <域>:<N>:<序号>；逐态判定行：--list-states <域>:<N>（这两种的域名用产物里 domain= 的写法，U 域是 U_verify_on、U_verify_off）";
+const USAGE: &str = "用法：e165_replay_bounded_enumeration --bounds segment1|segment2|light [--family legal|adversarial|all] [--only <域>[,<域>…]] [--max-n <N>]；域名：L L6 L9 U C C6 A A6 AC X E1 E2 R。单状态重放：--replay-state <域>:<N>:<序号>；逐态判定行：--list-states <域>:<N>（这两种的域名用产物里 domain= 的写法，U 域是 U_verify_on、U_verify_off）；钉住的红例逐行重放：--pinned-cases <表>（格式见 .claude/gate.d/e165-pinned-red-examples.tsv 文件头）";
 
 fn parse_the_command_line(arguments: &[String]) -> Result<RunSelection, String> {
     let mut bounds: Option<BoundsTier> = None;
@@ -4757,6 +5037,12 @@
 
 fn main() {
     let arguments: Vec<String> = env::args().skip(1).collect();
+    if let [option, table_path] = arguments.as_slice() {
+        if option == "--pinned-cases" {
+            replay_every_pinned_red_example(table_path);
+            return;
+        }
+    }
     match parse_the_state_probe(&arguments) {
         Ok(Some((domain, size, probe))) => {
             run_the_state_probe(domain, size, probe);
@@ -5066,4 +5352,90 @@
             assert!(reading.is_red_as_required, "对照没红：{reading:?}");
         }
     }
+
+    /// 钉住的表 `.claude/gate.d/e165-pinned-red-examples.tsv` 第 20、27 行的 ①②③⑦ 原样抄来：拿 ⑦ 重算的指纹要等于 ①。
+    /// 第 27 行的 state= 在 ;first= 之后还有 ;expected=，截掉的是 ;first= 起的整段。
+    #[test]
+    fn the_input_fingerprint_recomputed_from_a_pinned_state_row_equals_the_one_the_table_carries() {
+        let cross_instance_state_row = "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,j6_premise_holds_judged state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg100:watermark1000:applied0";
+        assert_eq!(
+            input_fingerprint_of("X", "2", cross_instance_state_row).as_deref(),
+            Some("371a71190dce62e090cbf0a361b11069fb68cc63ab2a44e88398bb2f0570f2f0"),
+            "X 域那一行：sha256(域|N|state= 截到 ;first= 之前) 要等于表里的 ①"
+        );
+        let adversarial_state_row = "E7RESULT name=state domain=A ring_slots=64 n=1 ordinal=112 counters=states,j6_premise_holds_judged,applied_ge1 state=anchor=present;slots=t+1:o1:L1:C1:x1:chain_true;first=returned:txg101:watermark9001038:applied1;expected=txg101:watermark9001038/txg101:watermark9001038/txg101:watermark9001038";
+        assert_eq!(
+            input_fingerprint_of("A", "1", adversarial_state_row).as_deref(),
+            Some("aacef16d8fa9f38237852f113de9db647ac0655bfe4192f8cdcce7c5e8a002b3"),
+            "A 域那一行：;first= 之后的 ;expected= 也不进指纹"
+        );
+        assert_eq!(
+            input_fingerprint_of("A", "1", "E7RESULT name=done emitted=3"),
+            None,
+            "没有 state= 那一段的行算不出指纹"
+        );
+    }
+
+    /// 钉住的表一行：八段、②③④ 解得出才收；段数不对、域不认识、N 越界、序号不是数都拒。
+    #[test]
+    fn a_pinned_red_example_row_needs_eight_columns_a_known_domain_a_size_in_range_and_an_ordinal()
+    {
+        let row = |domain: &str, size: &str, ordinal: &str| {
+            format!("abc\t{domain}\t{size}\t{ordinal}\tledger.tsv:2\tcrates/mutations.tsv#name\tE7RESULT original\tE7RESULT variant")
+        };
+        assert_eq!(
+            parse_a_pinned_red_example(20, &row("U_verify_off", "3", "36")),
+            Ok(PinnedRedExample {
+                line_number_in_the_table: 20,
+                input_fingerprint: "abc".to_string(),
+                domain_text: "U_verify_off".to_string(),
+                size_text: "3".to_string(),
+                domain: Domain::NamedUnitsNotVerified,
+                size: 3,
+                ordinal: 36,
+            })
+        );
+        let seven_columns = row("L", "2", "3")
+            .rsplit_once('\t')
+            .map(|(first_seven_columns, _eighth_column)| first_seven_columns.to_string())
+            .expect("造的行有八段");
+        assert!(parse_a_pinned_red_example(20, &seven_columns).is_err());
+        assert!(parse_a_pinned_red_example(20, &row("U", "2", "3")).is_err());
+        assert!(parse_a_pinned_red_example(20, &row("X", "1", "3")).is_err());
+        assert!(parse_a_pinned_red_example(20, &row("L", "2", "three")).is_err());
+    }
+
+    /// 重放出来的逐态行对着表里那一行判：指纹相同、指纹漂了、序号不在这个 (域, N) 里，三种各判各的。
+    #[test]
+    fn a_pinned_red_example_whose_recomputed_fingerprint_differs_is_reported_as_drifted() {
+        let state_row = "E7RESULT name=state domain=X ring_slots=64 n=2 ordinal=2 counters=states,j6_premise_holds_judged state=shape=1c0/next_instance:1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg100:watermark1000:applied0";
+        let example = PinnedRedExample {
+            line_number_in_the_table: 20,
+            input_fingerprint: "371a71190dce62e090cbf0a361b11069fb68cc63ab2a44e88398bb2f0570f2f0"
+                .to_string(),
+            domain_text: "X".to_string(),
+            size_text: "2".to_string(),
+            domain: Domain::CrossInstance,
+            size: 2,
+            ordinal: 2,
+        };
+        assert_eq!(
+            judge_a_pinned_red_example_replay(&example, Some(state_row.to_string()), 10),
+            PinnedRedExampleReplay::FingerprintMatches {
+                state_row: state_row.to_string()
+            }
+        );
+        let drifted_state_row = state_row.replace("on_disk=-,good", "on_disk=good,-");
+        assert!(
+            matches!(
+                judge_a_pinned_red_example_replay(&example, Some(drifted_state_row), 10),
+                PinnedRedExampleReplay::FingerprintDrifted { .. }
+            ),
+            "状态编码变了，重算的指纹与 ① 不同，要报漂了"
+        );
+        assert_eq!(
+            judge_a_pinned_red_example_replay(&example, None, 2),
+            PinnedRedExampleReplay::OrdinalOutsideTheDomainSize { states_walked: 2 }
+        );
+    }
 }
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -20,8 +20,8 @@
 步 2 变异：忘了改写分配记录	crates/singlefs-core/src/allocator.rs	            record.is_released = true;	            record.is_released = false;	-p singlefs-harness --test overwrite_in_one_instance -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 3：不写行（实例表照抄）	crates/singlefs-core/src/mount.rs	        instance_table: InstanceTablePlan::Rewrite(instance_table),	        instance_table: InstanceTablePlan::Carry(previous.root.instance_table),	-p singlefs-harness --test writable_mount_takes_a_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
 步 3：暖机只推一次	crates/singlefs-core/src/mount.rs	        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(warm_up_publishes.len()).expect("次数") < 1	-p singlefs-harness --test writable_mount_takes_a_second_instance -- damaging_every_instance_two_root	damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device
-步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-replay-prefix/src/lib.rs	    record.instance == root_instance && (record.instance > root_instance || (record.instance	    (record.instance > root_instance || (record.instance	-p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
-步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）	crates/singlefs-replay-prefix/src/lib.rs	            { (root_instance, counter + 1) },	            { (root_instance, counter + 2) },	-p singlefs-harness --test writable_mount_takes_a_second_instance -- stray_record_of_the_previous_instance	stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row
+步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-replay-prefix/src/lib.rs	    record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0	    record.instance.0 > root_instance.0 || (record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0)	-p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
+步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）	crates/singlefs-replay-prefix/src/lib.rs	        expected_next = Some((root_instance, JournalCounter(counter.0 + 1)));	        expected_next = Some((root_instance, JournalCounter(counter.0 + 2)));	-p singlefs-harness --test writable_mount_takes_a_second_instance -- stray_record_of_the_previous_instance	stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row
 步 3：I-3.8 判定恒真	crates/singlefs-checker/src/walk.rs	            unique && below_mount_root && chain_record_last,	            unique || below_mount_root || chain_record_last || true,	-p singlefs-harness --test writable_mount_takes_a_second_instance -- checker_rejects_an_instance_table_row	checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root
 步 3：重建分配器时不把已释放的放进 defer 队列	crates/singlefs-core/src/allocator.rs	            if record.is_released {\n                device_map.mark_released(record.slot, span);\n            }	            if record.is_released && false {\n                device_map.mark_released(record.slot, span);\n            }	-p singlefs-harness --test writable_mount_takes_a_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
 步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	            allocator.isolate_abandoned(device, slot, span_slots);	            let _ = (device, slot, span_slots);	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored with_the_shadow_ledger_a_publish_after_a_recovery	with_the_shadow_ledger_a_publish_after_a_recovery_keeps_off_the_abandoned_data_slot_and_the_abandoned_root_reads_the_third_content
@@ -33,14 +33,14 @@
 步 5：复用时追加记录而不改写（同盘同槽两条）	crates/singlefs-core/src/allocator.rs	            existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");\n            existing.generation = generation;\n            existing.is_released = false;	            records.push(AllocationRecord {\n                device,\n                slot: placement.slot,\n                span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),\n                generation,\n                is_released: false,\n            });	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5：checker 的候选集不按 F 收（F 之下的根照走）	crates/singlefs-checker/src/walk.rs	            let walked = *index == newest_index || (!abandoned && !below_floor);	            let walked = *index == newest_index || !abandoned;	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5（2026-09-26 取 SysPre 之后改写这一行）：F_生效 退回「各盘根上 F 最大值的最小值」、不读系统配置	crates/singlefs-core/src/recovery.rs	    highest_on_the_roots\n        .max(highest_in_the_system_configurations)\n        .unwrap_or(CheckpointTxg(0))	    {\n        let mut highest_per_device: BTreeMap<DeviceIdentity, CheckpointTxg> = BTreeMap::new();\n        for root in readable_roots(reader, region_devices, immutable_sizes, filesystem_identifier) {\n            let device = region_devices[usize::try_from(crate::root_ring::target_for_publish(root.checkpoint_txg, immutable_sizes.root_ring_slots_per_region).region).expect("区域号")];\n            let highest = highest_per_device.entry(device).or_insert(root.rollback_floor);\n            *highest = (*highest).max(root.rollback_floor);\n        }\n        highest_per_device.values().copied().min().unwrap_or(CheckpointTxg(0))\n    }	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored floor_carried_by_the_system_configuration_takes_effect	floor_carried_by_the_system_configuration_takes_effect_on_remount_even_when_one_device_lost_its_root_carrying_it
-步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条	crates/singlefs-replay-prefix/src/lib.rs	                if record.checkpoint_txg != chain_start_txg_without_anchor\n	                if false && record.checkpoint_txg != chain_start_txg_without_anchor\n	-p singlefs-harness --test writable_mount_takes_a_second_instance -- torn_anchor_record	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg_when_the_next_record_is_torn_too
+步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条	crates/singlefs-replay-prefix/src/lib.rs	            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0\n	            if false && record.checkpoint_txg.0 != chain_start_txg_without_anchor.0\n	-p singlefs-harness --test writable_mount_takes_a_second_instance -- torn_anchor_record	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg_when_the_next_record_is_torn_too
 步 4：普通重开不隔离被抛弃根引用的槽（影子账只在回退那一次算）	crates/singlefs-core/src/mount.rs	    mount_writable_with_test_only_switches(parameters, devices, space_admission, ShadowLedger::On)	    mount_writable_with_test_only_switches(parameters, devices, space_admission, ShadowLedger::Off)	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored the_plain_remount_after_a_recovery	the_plain_remount_after_a_recovery_that_abandoned_roots_keeps_the_isolation
 步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效	crates/singlefs-core/src/mount.rs	    if target.checkpoint_txg < effective_floor {	    if target.checkpoint_txg < newest_root.rollback_floor {	-p singlefs-checker-tier --test rollback_floor_written_into_the_system_configuration_first_and_normal_unmount -- rolling_back_below_a_floor	rolling_back_below_a_floor_only_the_system_configuration_carries_is_refused_before_any_write
 步 5：回收门槛不看环里最旧有效根	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |_oldest| effective_floor)	-p singlefs-core reclaim_floor_takes	reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root
 步 4：影子账把被抛弃根账里已释放的落点也隔离	crates/singlefs-core/src/mount.rs	            .filter(|record| !record.is_released)\n            .map(|record| (record.device, record.slot, u64::from(record.span_slots)))	            .map(|record| (record.device, record.slot, u64::from(record.span_slots)))	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5：重建分配器时不认第 0 版树表单元（另一个进程里的第一个文件版本漏释放它）	crates/singlefs-core/src/allocator.rs	                record.generation == CheckpointTxg(0)\n                    && record.span_slots == 1\n                    && !record.is_released	                record.generation == CheckpointTxg(0)\n                    && record.span_slots == 1\n                    && !record.is_released\n                    && false	-p singlefs-core rebuild_from_records_remembers	rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released
 步 4：影子账不豁免候选根引用的槽（保守读法）	crates/singlefs-core/src/mount.rs	            if referenced_by_candidates.contains(&key) || !isolated.insert(key) {	            if !isolated.insert(key) {	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored the_plain_remount_after_a_recovery	the_plain_remount_after_a_recovery_that_abandoned_roots_keeps_the_isolation
-步 4：被抛弃根的树表读不出时不计数	crates/singlefs-core/src/mount.rs	            None => {\n                unreadable += 1;\n                match placements_referenced_by_root_computed_from_the_mapping(	            None => {\n                match placements_referenced_by_root_computed_from_the_mapping(	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored torn_tree_table_of_an_abandoned_root	torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount
+步 4：被抛弃根的树表读不出时不计数	crates/singlefs-core/src/mount.rs	                None => {\n                    unreadable += 1;\n                    match placements_referenced_by_root_computed_from_the_mapping(	                None => {\n                    match placements_referenced_by_root_computed_from_the_mapping(	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored torn_tree_table_of_an_abandoned_root	torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount
 步 5：抬 F 之后不按新候选集重算影子账	crates/singlefs-core/src/mount.rs	    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {\n        match isolate_slots_referenced_only_by_abandoned_roots(	    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::Off {\n        match isolate_slots_referenced_only_by_abandoned_roots(	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5：影子账的豁免不看候选根的 txg 是否 ≥ F	crates/singlefs-core/src/mount.rs	        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)	        .filter(|root| !is_abandoned(root))	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5：抬 F 回收的槽不扣住、生效之前就能发出去	crates/singlefs-core/src/mount.rs	        ReclaimedReuse::HeldUntilFloorTakesEffect,	        ReclaimedReuse::Immediately,	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored slots_reclaimed_by_raising	slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect
@@ -223,7 +223,7 @@
 并行线二：读路径自报的设备级读数不加（实现说这次一个设备读都没发，装置在块层照样数到那几次，D17 已定项 5 射程 ① 的对账等于没有）	crates/singlefs-core/src/mounted_read.rs	        observation.device_reads_issued += 1;	        observation.device_reads_issued += 0;	-p singlefs-harness --test mounted_read_by_offset -- random_four_kibibyte_reads	random_four_kibibyte_reads_return_the_written_bytes_and_never_scan_the_journal_ring
 并行线二：这次读了几个单元的计数不加（验收第 4 条那个观测点恒 0，跨单元的页也不报 2）	crates/singlefs-core/src/mounted_read.rs	            observation.data_units_read += 1;	            observation.data_units_read += 0;	-p singlefs-harness --test mounted_read_by_offset -- every_aligned_page	every_aligned_page_reads_back_and_the_pages_that_cross_a_unit_boundary_read_two_units
 并行线二：一次读只解引用第一个单元（跨单元的页少拷后半页，D4 已定项 5 那一成多的页整档漏掉）	crates/singlefs-core/src/mounted_read.rs	        for unit_number in span.first.0..=span.last_inclusive.0 {	        for unit_number in span.first.0..=span.first.0 {	-p singlefs-harness --test mounted_read_by_offset -- corrupting_one_byte	corrupting_one_byte_in_a_data_unit_makes_reads_over_it_report_a_checksum_error_and_leaves_the_other_reads_alone
-只供测试的开关：关掉点名验证时 verification_passed 照加（两臂在健康镜像上报同一个数，运行时看不出走了哪一条）	crates/singlefs-replay-prefix/src/lib.rs	        if verify_named_units {\n            report.verification_passed += 1;\n        }	        report.verification_passed += 1;	-p singlefs-harness --test recovery_reads_the_created_file_after_reopen	the_named_unit_verification_switch_shows_up_in_the_runtime_counter
+只供测试的开关：关掉点名验证时 verification_passed 照加（两臂在健康镜像上报同一个数，运行时看不出走了哪一条）	crates/singlefs-replay-prefix/src/lib.rs	        match named_unit_verification {\n            NamedUnitVerification::Verify => {\n                report.verification_passed += 1;\n            },\n            NamedUnitVerification::Skip => {},\n        }	        report.verification_passed += 1;	-p singlefs-harness --test recovery_reads_the_created_file_after_reopen	the_named_unit_verification_switch_shows_up_in_the_runtime_counter
 I-1.10：条目宽与字段表宽的比对改成恒真（走读照样停下，只是不再判红）	crates/singlefs-checker/src/walk.rs	.judge("I-1.10", entry_width == entry_field_table_bytes, || {	.judge("I-1.10", true, || {	-p singlefs-harness --test checker_known_bad_images -- an_entry_width	an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width_invariant
 I-1.10：判红之后不停下，照样按字段表的固定偏移解那条窄条目（checker 自己被坏镜像打死）	crates/singlefs-checker/src/walk.rs	            if entry_width != entry_field_table_bytes {\n                return;\n            }	            if false && entry_width != entry_field_table_bytes {\n                return;\n            }	-p singlefs-harness --test checker_known_bad_images -- an_entry_width	an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width_invariant
 extent 叶记录的读者不判条目宽（按 112 的固定偏移切 24 字节的条目）	crates/singlefs-core/src/records.rs	    if bytes.len() < usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112") {	    if false && bytes.len() < usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112") {	-p singlefs-core --lib -- entry_readers_refuse	entry_readers_refuse_an_entry_narrower_than_its_field_table
@@ -352,7 +352,7 @@
 增补 2 第 21 行（C366，挂载层）：带文件的一版上暖机的 jsn 取上一版的 txg + 1	crates/singlefs-core/src/mount.rs	        counter: current_file_version.record.counter + 1,	        counter: current_file_version.root.checkpoint_txg.0 + 1,	-p singlefs-harness --test warm_up_journal_counter -- a_writable_mount_after_a_two_record_publish_numbers_its_records_and_tail_from_the_jsn_one_ahead_of_the_txg	a_writable_mount_after_a_two_record_publish_numbers_its_records_and_tail_from_the_jsn_one_ahead_of_the_txg
 增补 2 第 21 行（C366，挂载层）：发布轮换系统配置槽时 tail 写 txg、不写 jsn	crates/singlefs-core/src/transaction.rs	    writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail,\n        journal_instance,\n    })	    writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail: checkpoint_txg.0,\n        journal_instance,\n    })	-p singlefs-harness --test warm_up_journal_counter -- warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two	warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two
 增补 2 第 61 行（C519 已知丢失）：施加前的点名单元验证改成任一条位置条目验过即过（改法 Z2 被悄悄落地时已知丢失那条先红）	crates/singlefs-core/src/recovery.rs	            named.locations.iter().all(|location| {	            named.locations.iter().any(|location| {	-p singlefs-harness --test whole_device_loss_after_warm_up -- c519_known_loss	c519_known_loss_after_the_warm_up_covers_both_devices_losing_the_acknowledged_version_root_device_falls_back_to_the_row_publish_version
-增补 2 第 62 行（C533 复现）：前缀跨实例边界，写行那条孤记录进了水位之上	crates/singlefs-replay-prefix/src/lib.rs	    record.instance == root_instance && (record.instance > root_instance || (record.instance	    (record.instance > root_instance || (record.instance	-p singlefs-harness --test row_publish_record_without_its_root -- c533_row_publish_record_persisted	c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_are_free_slots_that_the_next_mount_overwrites
+增补 2 第 62 行（C533 复现）：前缀跨实例边界，写行那条孤记录进了水位之上	crates/singlefs-replay-prefix/src/lib.rs	    record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0	    record.instance.0 > root_instance.0 || (record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0)	-p singlefs-harness --test row_publish_record_without_its_root -- c533_row_publish_record_persisted	c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_are_free_slots_that_the_next_mount_overwrites
 增补 2 第 9 行 P1（C497）：树表 0 条的一版上写行时分配记录树节点先于实例表取落点	crates/singlefs-core/src/transaction.rs	        let rewritten_roles: Vec<TransactionUnit> = instance_table_roles\n            .iter()\n            .copied()\n            .chain(\n                allocation_plan\n                    .rewritten_nodes()\n                    .into_iter()\n                    .map(role_of_allocation_record_tree_node),\n            )\n            .collect();	        let rewritten_roles: Vec<TransactionUnit> = allocation_plan\n            .rewritten_nodes()\n            .into_iter()\n            .map(role_of_allocation_record_tree_node)\n            .chain(instance_table_roles.iter().copied())\n            .collect();	-p singlefs-harness --test presumed_clause_checks -- c497_every_publish	c497_every_publish_that_rewrites_the_instance_table_bumps_it_before_every_other_commit_generated_block
 增补 2 第 9 行 P3（C498）：暖机不看本实例的根覆盖没覆盖两块盘、恒推 R 次	crates/singlefs-core/src/mount.rs	    while all_devices\n        .iter()\n        .any(|identity| !covered.contains(identity))\n        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	    while u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	-p singlefs-harness --test presumed_clause_checks -- c498_warm_up_publishes	c498_warm_up_publishes_after_remounts_rollback_and_a_crash_in_the_middle_of_a_mount_are_counted_by_the_clause
 增补 2 第 9 行 P5（C499）：可写挂载认「上一次干净关闭」（tail 那条就是所选根那次、下一格没有记录）就只取 tail 那一条、不扫全环	crates/singlefs-core/src/mount.rs	    let records = scan_journal(cache, system_configuration);\n    let (journal, effective_root) = replay_journal(	    let tail_record = |counter: u64| {\n        crate::recovery::PoolReader::read(\n            cache,\n            PoolReader::device_identities(cache)[0],\n            crate::journal::record_offset(\n                counter,\n                system_configuration.immutable.sizes.journal_ring_bytes,\n            ),\n            4096,\n        )\n        .and_then(|bytes| {\n            crate::journal::JournalRecord::parse(\n                &bytes,\n                crate::unit::unit_filesystem_identifier(\n                    &system_configuration.immutable.filesystem_identifier,\n                ),\n            )\n        })\n    };\n    let tail = system_configuration.quantities.journal_tail;\n    let closed_cleanly = tail > 0\n        && tail_record(tail).is_some_and(|record| {\n            record.instance == chosen_root.instance\n                && record.checkpoint_txg == chosen_root.checkpoint_txg\n        })\n        && tail_record(tail + 1).is_none();\n    let records: std::collections::BTreeMap<_, _> = if closed_cleanly {\n        tail_record(tail)\n            .map(|record| ((record.instance, record.counter), record))\n            .into_iter()\n            .collect()\n    } else {\n        scan_journal(cache, system_configuration)\n    };\n    let (journal, effective_root) = replay_journal(	-p singlefs-harness --test presumed_clause_checks -- c499_remount_after	c499_remount_after_a_clean_close_and_after_a_crash_both_scan_the_whole_ring_and_recover_by_the_same_rules
@@ -445,7 +445,7 @@
 E158 root_choice_repair Q2-1 的 candidate_fault_sets_of_weight 权重公式：一条记录按 1 个单位算（该按 2 份自证算）	crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs	let weight_one_count = weight - record_count * 2;	let weight_one_count = weight - record_count;	-p singlefs-checker-tier --bin e158_root_choice_repair	candidate_fault_sets_of_weight_mixes_root_slots_and_journal_records_by_weight
 E158 root_choice_repair Q2-1 的 judge_recovery_outcome「倒挂」判定取反（该在根不在时间线上时判倒挂，改成在时间线上时判）	crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs	if !timeline.contains(&chosen) {	if timeline.contains(&chosen) {	-p singlefs-checker-tier --bin e158_root_choice_repair	judge_recovery_outcome_passes_when_root_on_timeline_and_content_matches
 E158 root_choice_repair Q2-1 的 candidate_fault_sets_of_weight 起点错位（记录计数从 1 起，权重 0 时枚不出空集合）	crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs	let mut record_count = 0u64;	let mut record_count = 1u64;	-p singlefs-checker-tier --bin e158_root_choice_repair	candidate_fault_sets_of_weight_at_zero_is_only_the_empty_set
-P6 后一半：锚点读不出时不看本次发布内序号，链首接在下一次发布的第二条上（三方第一轮 K4-b）	crates/singlefs-replay-prefix/src/lib.rs	                    || record.ordinal_within_publish != 1 {\n                    break ;\n                }\n            },\n        }\n        // [原文 3119–3126] 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断。[改写] 三态由回调交成布尔\n        if back_chain_broken(index) {\n            break ;\n        }\n        // [原文 3127–3134] 一次发布的第一条序号是 1（已定项 4 读者规则；C539）。\n        if open.is_empty() && record.ordinal_within_publish != 1 {	                    || false {\n                    break ;\n                }\n            },\n        }\n        // [原文 3119–3126] 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断。[改写] 三态由回调交成布尔\n        if back_chain_broken(index) {\n            break ;\n        }\n        // [原文 3127–3134] 一次发布的第一条序号是 1（已定项 4 读者规则；C539）。\n        if open.is_empty() && record.ordinal_within_publish != 1 && expected_next.is_some() {	-p singlefs-harness --test multi_unit_file_write_and_read -- without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish	without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish
+P6 后一半：锚点读不出时不看本次发布内序号，链首接在下一次发布的第二条上（三方第一轮 K4-b）	crates/singlefs-replay-prefix/src/lib.rs	                || record.ordinal_within_publish.0 != 1 {\n                return CandidateVerdict::BrokenBeforeTheNamedUnits;\n            }\n        },\n    }\n    // 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断；三态由回调交成布尔。\n    if back_chain_broken(index) {\n        return CandidateVerdict::BrokenBeforeTheNamedUnits;\n    }\n    if records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 {	                || false {\n                return CandidateVerdict::BrokenBeforeTheNamedUnits;\n            }\n        },\n    }\n    // 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断；三态由回调交成布尔。\n    if back_chain_broken(index) {\n        return CandidateVerdict::BrokenBeforeTheNamedUnits;\n    }\n    if records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 && expected_next.is_some() {	-p singlefs-harness --test multi_unit_file_write_and_read -- without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish	without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish
 P6 后一半：多条记录的发布每条都写本次发布内序号 1	crates/singlefs-core/src/transaction.rs	            // 恢复只从序号 1 那条接链首（已定项 14 注 1）。\n            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(\n                record_offset_in_this_publish,\n            ),\n	            // 恢复只从序号 1 那条接链首（已定项 14 注 1）。\n            ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,\n	-p singlefs-harness --test multi_unit_file_write_and_read -- without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish	without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish
 P6 后一半：本次发布内序号从 0 起	crates/singlefs-core/src/journal.rs	        Self(u32::try_from(record_offset_in_this_publish + 1).expect(	        Self(u32::try_from(record_offset_in_this_publish).expect(	-p singlefs-core --lib -- journal::tests::the_ordinal_of_the_record_at_each_offset_of_the_publish_is_that_offset_plus_one	journal::tests::the_ordinal_of_the_record_at_each_offset_of_the_publish_is_that_offset_plus_one
 P6 后一半：记录头写者不写本次发布内序号（恒写 1）	crates/singlefs-core/src/journal.rs	        writer.put_u32(self.ordinal_within_publish.0);	        writer.put_u32(1);	-p singlefs-core --lib -- journal::tests::the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips	journal::tests::the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips
@@ -509,13 +509,13 @@
 实二十：一个角色都不点名的发布一条记录都不切出来	crates/singlefs-core/src/transaction.rs	    if roles_named_by_the_last_transaction.is_empty() {	    if false {	-p singlefs-core --lib -- transaction::tests::the_last_transaction_spills_over_as_many_records_as_its_named_units_need	transaction::tests::the_last_transaction_spills_over_as_many_records_as_its_named_units_need_and_they_all_share_its_transaction
 实二十：空发布记录不带末条标志	crates/singlefs-core/src/transaction.rs	        // 含空发布记录，那一条记录标志位 0 也写 1）。\n        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,	        // 含空发布记录，那一条记录标志位 0 也写 1）。\n        place_in_publish: JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,	-p singlefs-harness --test new_pool_file_creation_publish -- root_slots_system_configurations_and_journal_ring_hold_the_published_state	root_slots_system_configurations_and_journal_ring_hold_the_published_state
 实二十：树表 0 条那一版上写行那次发布的记录不带末条标志	crates/singlefs-core/src/transaction.rs	            place_in_publish: if transaction_of_the_next_record.is_none() {\n                JournalRecordPlaceInPublish::LastRecordOfThePublish\n            } else {\n	            place_in_publish: if transaction_of_the_next_record.is_none() {\n                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow\n            } else {\n	-p singlefs-harness --test writable_mount_of_a_formatted_pool -- writable_mount_after_a_crash_right_after_acquiring_an_instance	writable_mount_after_a_crash_right_after_acquiring_an_instance_writes_a_row_for_the_burnt_instance
-实二十：锚点取读法甲——所选根那次发布读得出的记录里 jsn 最大那条，不看末条标志（替换第 438 行那条锚点已腐化的变异）	crates/singlefs-replay-prefix/src/lib.rs	root_checkpoint_txg\n            && record_ends_its_publish(record) {\n            // [原文 2976–2984]\n            if found.is_some() {\n                return Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlag);\n            }\n            found = Some(record.counter);	root_checkpoint_txg {\n            found = Some(record.counter);	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- when_the_last_record_of_the_chosen_roots_publish_is_unreadable	when_the_last_record_of_the_chosen_roots_publish_is_unreadable_its_readable_earlier_record_is_no_anchor_and_the_next_publish_starts_the_chain
-实二十：所选根那次发布带末条标志的多于一条时不停下（条款没写那一格照常接链）	crates/singlefs-replay-prefix/src/lib.rs	            if found.is_some() {\n                return Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlag);\n            }\n            found = Some(record.counter);	            if found.is_none() {\n                found = Some(record.counter);\n            }	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- two_last_record_flags_in_the_chosen_roots_publish	two_last_record_flags_in_the_chosen_roots_publish_stop_recovery_and_a_writable_mount_before_any_write
-实二十：一次发布之内跳号不断链	crates/singlefs-replay-prefix/src/lib.rs	            if record.ordinal_within_publish as u64\n                != previous_record_of_the_open_publish.ordinal_within_publish as u64 + 1 {	            if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- an_ordinal_that_skips_within_a_publish	an_ordinal_that_skips_within_a_publish_breaks_the_chain_and_the_publish_is_not_applied
-实二十：一个事务的提交标记没出现、下一条换了事务号也照接	crates/singlefs-replay-prefix/src/lib.rs	            if !previous_record_of_the_open_publish.is_commit && record.transaction\n                != previous_record_of_the_open_publish.transaction {	            if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- transaction_whose_commit_marker_never_appears	transaction_whose_commit_marker_never_appears_before_the_next_transaction_keeps_its_publish_unapplied
-实二十：末条不带提交标记也整次施加	crates/singlefs-replay-prefix/src/lib.rs	        if !record.is_commit && record_ends_its_publish(record) {	        if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- last_record_of_the_publish_without_the_commit_marker	last_record_of_the_publish_without_the_commit_marker_keeps_its_publish_unapplied
-实二十：层 0 L8：恢复在不带提交标记的记录上就停（一事务一条时的旧写法，跨两条记录的事务永远施加不上）	crates/singlefs-replay-prefix/src/lib.rs	        if !record.is_commit && record_ends_its_publish(record) {	        if !record.is_commit {	-p singlefs-checker-tier --test crash_enumeration_record_spill_over_stream -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
-实二十：层 0 L8：恢复把不带末条标志的记录也当末条（半次发布施加上去）	crates/singlefs-core/src/recovery.rs	        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,	        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => true,	-p singlefs-checker-tier --test crash_enumeration_record_spill_over_stream -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
+实二十：锚点取读法甲——所选根那次发布读得出的记录里 jsn 最大那条，不看末条标志（替换第 438 行那条锚点已腐化的变异）	crates/singlefs-replay-prefix/src/lib.rs	root_checkpoint_txg.0\n            && record_ends_its_publish(record) {\n            counters_carrying_the_last_record_flag.push(record.counter);	root_checkpoint_txg.0 {\n            counters_carrying_the_last_record_flag.clear();\n            counters_carrying_the_last_record_flag.push(record.counter);	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- when_the_last_record_of_the_chosen_roots_publish_is_unreadable	when_the_last_record_of_the_chosen_roots_publish_is_unreadable_its_readable_earlier_record_is_no_anchor_and_the_next_publish_starts_the_chain
+实二十：所选根那次发布带末条标志的多于一条时不停下（条款没写那一格照常接链）	crates/singlefs-replay-prefix/src/lib.rs	    if counters_carrying_the_last_record_flag.len() > 1 {\n        return Err(	    if false {\n        return Err(	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- two_last_record_flags_in_the_chosen_roots_publish	two_last_record_flags_in_the_chosen_roots_publish_stop_recovery_and_a_writable_mount_before_any_write
+实二十：一次发布之内跳号不断链	crates/singlefs-replay-prefix/src/lib.rs	        if u64::from(record.ordinal_within_publish.0) != u64::from(\n            previous_record_of_the_open_publish.ordinal_within_publish.0,\n        ) + 1 {	        if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- an_ordinal_that_skips_within_a_publish	an_ordinal_that_skips_within_a_publish_breaks_the_chain_and_the_publish_is_not_applied
+实二十：一个事务的提交标记没出现、下一条换了事务号也照接	crates/singlefs-replay-prefix/src/lib.rs	        if !previous_record_of_the_open_publish.is_commit\n            && record.transaction.0 != previous_record_of_the_open_publish.transaction.0 {	        if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- transaction_whose_commit_marker_never_appears	transaction_whose_commit_marker_never_appears_before_the_next_transaction_keeps_its_publish_unapplied
+实二十：末条不带提交标记也整次施加	crates/singlefs-replay-prefix/src/lib.rs	    if !record.is_commit && record_ends_its_publish(record) {	    if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- last_record_of_the_publish_without_the_commit_marker	last_record_of_the_publish_without_the_commit_marker_keeps_its_publish_unapplied
+实二十：层 0 L8：恢复在不带提交标记的记录上就停（一事务一条时的旧写法，跨两条记录的事务永远施加不上）	crates/singlefs-replay-prefix/src/lib.rs	    if !record.is_commit && record_ends_its_publish(record) {	    if !record.is_commit {	-p singlefs-checker-tier --test crash_enumeration_record_spill_over_stream -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
+实二十：层 0 L8：恢复把不带末条标志的记录也当末条（半次发布施加上去）	crates/singlefs-core/src/recovery.rs	                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => {\n                    singlefs_replay_prefix::JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow\n                }	                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => {\n                    singlefs_replay_prefix::JournalRecordPlaceInPublish::LastRecordOfThePublish\n                }	-p singlefs-checker-tier --test crash_enumeration_record_spill_over_stream -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
 实二十：I-8.9 不判记录标志其余位	crates/singlefs-checker/src/walk.rs	            if record.record_flags_byte & !RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0 {	            if false {	-p singlefs-harness --test checker_known_bad_images -- each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant	each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria
 实二十：I-8.9 不判序号 0	crates/singlefs-checker/src/walk.rs	            if record.ordinal_within_publish == 0 {	            if false {	-p singlefs-harness --test checker_known_bad_images -- each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant	each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria
 实二十：I-8.9 不判一次发布之内跳号	crates/singlefs-checker/src/walk.rs	                if ordinal_distance != i128::from(counter_distance) {	                if false {	-p singlefs-harness --test checker_known_bad_images -- each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant	each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria
@@ -610,8 +610,8 @@
 增补 2 收口表第 58 行（真设备抬 F 模式）：宿主检查把发布 D 那一段的终点记成发布 C（发布 D 那一段不单独比、红了也指不到它）	crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs	            window_ends.push((ProgramWindow::FourthTransaction, stream.operation_count()));	            window_ends.push((ProgramWindow::ThirdTransaction, stream.operation_count()));	-p singlefs-checker-tier --bin new_pool_file_creation_device_log_check	raise_rollback_floor_mode_compares_the_raise_window_and_is_red_there_when_one_step_changed
 实二二三 a（D23 已定项 14「这一版的失败处置」）：冻结着一次没重发的发布时发布路径不拒（另建下一次发布，同一 (实例, txg) 留下两条末条）	crates/singlefs-core/src/transaction.rs	    match allocator.frozen_publish() {\n        None => Ok(()),\n	    match None::<&FrozenPublish> {\n        None => Ok(()),\n	-p singlefs-harness --test publish_failure_resent_unchanged -- publish_that_fails_midway	publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish
 实二二三 a：原样重发成功之后分配器不换成那次发布成立之后的一份（下一次发布的落点落在重发的那次发布的单元上）	crates/singlefs-core/src/transaction.rs	    *allocator = allocator_after_the_publish;\n    Ok(Some(\n	    let _ = allocator_after_the_publish;\n    Ok(Some(\n	-p singlefs-harness --test publish_failure_resent_unchanged -- publish_that_fails_midway	publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish
-实二二三 b（C539，D23 已定项 4 读者规则）：锚点读得出时下一次发布的首条序号不是 1 照接	crates/singlefs-replay-prefix/src/lib.rs	        if open.is_empty() && record.ordinal_within_publish != 1 {	        if false && open.is_empty() && record.ordinal_within_publish != 1 {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- with_a_readable_anchor_a_next_publish	with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens
-实二二三 b（C540，D23 已定项 4 读者规则）：带末条标志的那一条之后同一 (实例, txg) 还有记录照字面按标志认边界	crates/singlefs-replay-prefix/src/lib.rs	        if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(	        if false && record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- last_record_flag_followed_by	last_record_flag_followed_by_the_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens
+实二二三 b（C539，D23 已定项 4 读者规则）：锚点读得出时下一次发布的首条序号不是 1 照接	crates/singlefs-replay-prefix/src/lib.rs	    if records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 {	    if false && records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- with_a_readable_anchor_a_next_publish	with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens
+实二二三 b（C540，D23 已定项 4 读者规则）：带末条标志的那一条之后同一 (实例, txg) 还有记录照字面按标志认边界	crates/singlefs-replay-prefix/src/lib.rs	    if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(	    if false && record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- last_record_flag_followed_by	last_record_flag_followed_by_the_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens
 实二二三 d（D19 已定项 5，用户 2026-09-25 定）：映射条目两条位置项指同一块盘时不拒（另一块盘那一份没核就释放）	crates/singlefs-core/src/transaction.rs	    if locations[0].device == locations[1].device {\n	    if false && locations[0].device == locations[1].device {\n	-p singlefs-harness --test release_checksum_quarantine -- two_mapping_locations_on_the_same_device	two_mapping_locations_on_the_same_device_are_refused_as_a_damaged_mapping_entry_before_anything_is_written
 实二二三 e（I-3.1 / I-3.11 的隔离读法，用户 2026-09-25 定）：已分配而没有根引用的记录一律不豁免	crates/singlefs-checker/src/walk.rs	        if some_copy_fails {\n            for device in devices_of_the_unit {\n	        if false && some_copy_fails {\n            for device in devices_of_the_unit {\n	-p singlefs-harness --test release_checksum_quarantine -- quarantined_record_is_exempted	quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read
 实二二三 e：已分配而没有根引用的记录不读那一份、一律豁免（读得出且对得上的也豁免）	crates/singlefs-checker/src/walk.rs	            .any(|device| !copy_matches_its_location_entry(*device));	            .any(|_device| true);	-p singlefs-harness --test release_checksum_quarantine -- quarantined_record_is_exempted	quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read
@@ -720,7 +720,7 @@
 实二（回退改形态：SysPre 与 B1，D16 已定项 1 / 7、D22 已定项 7 / 9、I-7.9 / I-7.12、C556 / C557）：checker 不读系统配置里的 F（读成 0）	crates/singlefs-checker/src/lib.rs	        rollback_floor: read_u64(slot, SYSTEM_CONFIGURATION_ROLLBACK_FLOOR_OFFSET),	        rollback_floor: 0,	-p singlefs-harness --test checker_known_bad_images -- one_device_whose_system_configuration_floor_is_below_a_root_on_it	one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant
 实二（回退改形态：SysPre 与 B1，D16 已定项 1 / 7、D22 已定项 7 / 9、I-7.9 / I-7.12、C556 / C557）：checker 把每条根都读成不带卸载记号	crates/singlefs-checker/src/lib.rs	    let unmount_marker = if flags & ROOT_FLAG_UNMOUNT_MARKER == 0 {	    let unmount_marker = if true {	-p singlefs-harness --test checker_known_bad_images -- marked_root_raising_the_floor	marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant
 实二（回退改形态：SysPre 与 B1，D16 已定项 1 / 7、D22 已定项 7 / 9、I-7.9 / I-7.12、C556 / C557）：I-7.9 带记号那一支把「不高于前一条根的 txg」判成「低于」	crates/singlefs-checker/src/walk.rs	                    raised_floor <= newest_root_txg_before_the_raise,	                    raised_floor < newest_root_txg_before_the_raise,	-p singlefs-harness --test checker_known_bad_images -- marked_root_raising_the_floor	marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant
-实二（回退改形态：SysPre 与 B1，D16 已定项 1 / 7、D22 已定项 7 / 9、I-7.9 / I-7.12、C556 / C557）：checker 回退候选集的下界只读最新根带的 F、不读系统配置里的（C556 之前的读法）	crates/singlefs-checker/src/walk.rs	    let effective_rollback_floor = verified_system_configurations_of_this_pool\n        .iter()\n        .flat_map(|(_, slots)| slots.iter().map(|view| view.rollback_floor))\n        .fold(highest_floor_on_the_newest_valid_and_the_abandoned_roots, u64::max);	    let effective_rollback_floor = highest_floor_on_the_newest_valid_and_the_abandoned_roots;	-p singlefs-harness --test checker_known_bad_images -- every_root_carrying_the_raised_floor_unreadable	every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor
+实二（回退改形态：SysPre 与 B1，D16 已定项 1 / 7、D22 已定项 7 / 9、I-7.9 / I-7.12、C556 / C557）：checker 回退候选集的下界只读最新根带的 F、不读系统配置里的（C556 之前的读法）	crates/singlefs-checker/src/walk.rs	    let effective_rollback_floor = verified_system_configurations_of_this_pool\n        .iter()\n        .flat_map(|(_, slots)| slots.iter().map(|view| view.rollback_floor))\n        .fold(\n            highest_floor_on_the_newest_valid_and_the_abandoned_roots,\n            u64::max,\n        );	    let effective_rollback_floor = highest_floor_on_the_newest_valid_and_the_abandoned_roots;	-p singlefs-harness --test checker_known_bad_images -- every_root_carrying_the_raised_floor_unreadable	every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor
 实二（回退改形态：SysPre 与 B1，D16 已定项 1 / 7、D22 已定项 7 / 9、I-7.9 / I-7.12、C556 / C557）：最新根带的 F 低于 F 生效值时 I-3.1 照判（按它自己的 F 记的账对按生效值取的候选集并集）	crates/singlefs-checker/src/walk.rs	        roots[newest_index].2.rollback_floor == effective_rollback_floor;	        roots[newest_index].2.rollback_floor <= effective_rollback_floor;	-p singlefs-harness --test checker_known_bad_images -- every_root_carrying_the_raised_floor_unreadable	every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor
 实二（回退改形态：SysPre 与 B1，D16 已定项 1 / 7、D22 已定项 7 / 9、I-7.9 / I-7.12、C556 / C557）：I-7.12 不比系统配置里的 F（每块盘恒成立）	crates/singlefs-checker/src/walk.rs	            .is_none_or(|root| root.rollback_floor <= highest_system_configuration_floor);	            .is_none_or(|root| root.rollback_floor <= u64::MAX);	-p singlefs-harness --test checker_known_bad_images -- one_device_whose_system_configuration_floor_is_below_a_root_on_it	one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant
 实二（回退改形态：SysPre 与 B1，D16 已定项 1 / 7、D22 已定项 7 / 9、I-7.9 / I-7.12、C556 / C557）：C557 那一道拿掉：带卸载记号的根槽写落在卸载入口之外也不列	crates/singlefs-harness/src/memory_pool.rs	                if !inside_the_unmount_entry(stream_index) {	                if !inside_the_unmount_entry(stream_index) && stream_index == usize::MAX {	-p singlefs-checker-tier --test rollback_floor_written_into_the_system_configuration_first_and_normal_unmount -- unmount_markers_are_judged_against_the_recorded_unmount_entry	unmount_markers_are_judged_against_the_recorded_unmount_entry
@@ -735,7 +735,7 @@
 实三（D23 已定项 14「这一版的失败处置」、C558）：回退入口不先判冻结着的发布	crates/singlefs-core/src/mount.rs	) -> Result<RolledBack, RollbackError> {\n    if let Some(frozen) = allocator.frozen_publish() {	) -> Result<RolledBack, RollbackError> {\n    if let Some(frozen) = allocator.frozen_publish().filter(|_| false) {	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored the_failed_root_slot_write_of_a_rollback	the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash
 实三（候选集带文件，D23 已定项 14）：树表 0 条的回退目标报成「不在环里」	crates/singlefs-core/src/mount.rs	        Ok(true) => Err(not_a_candidate(\n            RollbackCandidateExclusion::VersionWithoutFile,\n        )),	        Ok(true) => Err(not_a_candidate(\n            RollbackCandidateExclusion::NotInRing,\n        )),	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored rolling_back_to_a_missing_root	rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write
 实三（候选集 txg ≥ F_生效，D16 已定项 1）：低于 F_生效的回退目标报成「树表 0 条」	crates/singlefs-core/src/mount.rs	        return Err(not_a_candidate(\n            RollbackCandidateExclusion::BelowEffectiveFloor,\n        ));	        return Err(not_a_candidate(\n            RollbackCandidateExclusion::VersionWithoutFile,\n        ));	-p singlefs-checker-tier --test rollback_floor_written_into_the_system_configuration_first_and_normal_unmount -- rolling_back_below_a_floor_only_the_system_configuration	rolling_back_below_a_floor_only_the_system_configuration_carries_is_refused_before_any_write
-实三（影子账，理由改指崩溃恢复）：账读不出的被抛弃根不计数	crates/singlefs-core/src/mount.rs	                unreadable += 1;\n                match placements_referenced_by_root_computed_from_the_mapping(	                unreadable += 0;\n                match placements_referenced_by_root_computed_from_the_mapping(	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored torn_tree_table_of_an_abandoned_root	torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount
+实三（影子账，理由改指崩溃恢复）：账读不出的被抛弃根不计数	crates/singlefs-core/src/mount.rs	                    unreadable += 1;\n                    match placements_referenced_by_root_computed_from_the_mapping(	                    unreadable += 0;\n                    match placements_referenced_by_root_computed_from_the_mapping(	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored torn_tree_table_of_an_abandoned_root	torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount
 实三（D16 已定项 1 根环容量边界）：入口的「4 个不同状态」不按 (inode 树根, extent 树根) 去重	crates/singlefs-core/src/mount.rs	        if counted_states.contains(&&root.user_visible_tree_root_pointers) {	        if false && counted_states.contains(&&root.user_visible_tree_root_pointers) {	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored the_state_repeated_by_rollbacks	the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker
 实三（D16 已定项 1 根环容量边界）：checker 的「4 个不同状态」不去重（I-7.9 的上限算宽）	crates/singlefs-checker/src/walk.rs	                if !counted_states.contains(&state) {	                if true {	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored the_state_repeated_by_rollbacks	the_state_repeated_by_rollbacks_is_counted_once_for_the_floor_ceiling_by_the_entry_and_by_the_checker
 实三（D16 已定项 1 根环容量边界，理想模型）：模型的抬 F 上限不去重	crates/singlefs-harness/src/model.rs	            if !counted_states.contains(&state) {	            if true {	-p singlefs-harness --lib -- model::tests::after_a_forward_rollback_the_ceiling	after_a_forward_rollback_the_ceiling_counts_the_repeated_state_once
@@ -1000,8 +1000,8 @@
 实审 A2b 第 27 条：分配记录读者不经共用判定	crates/singlefs-core/src/recovery.rs	    let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;\n    let Some(allocation_entry) = tree_table_entries	    let tree_table_entries: Vec<TreeTableEntry> = tree_table.entries.iter().filter_map(|bytes| TreeTableEntry::parse(bytes)).collect();\n    let Some(allocation_entry) = tree_table_entries	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_tree_table_carrying_one_kind_of_tree_twice_is_refused_as_damaged_by_every_reader
 实审 A2b 第 27 条：用户可见两棵树的读者不经共用判定	crates/singlefs-core/src/recovery.rs	    let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;\n    let mut pointers = UserVisibleTreeRootPointers::ABSENT;	    let tree_table_entries: Vec<TreeTableEntry> = tree_table.entries.iter().filter_map(|bytes| TreeTableEntry::parse(bytes)).collect();\n    let mut pointers = UserVisibleTreeRootPointers::ABSENT;	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_tree_table_carrying_one_kind_of_tree_twice_is_refused_as_damaged_by_every_reader
 实审 A2b 第 27 条：挂载态把同一种树两条报成条目解不开（与别的读者报的不同）	crates/singlefs-core/src/mounted_read.rs	                OpenPoolForReadFailure::Walk(RecoveryFailure::from(refusal))	                OpenPoolForReadFailure::TreeTableEntryMalformed	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_tree_table_carrying_one_kind_of_tree_twice_is_refused_as_damaged_by_every_reader
-实审 A2b 第 36 条：所选根的 txg 改回裸加一	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg == u64::MAX {	    if false {	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking
-实审 A2b 第 36 条：所选根的 txg 加二才算溢出（边界）	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg == u64::MAX {	    if root_checkpoint_txg >= u64::MAX - 1 {	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking
+实审 A2b 第 36 条：所选根的 txg 改回裸加一	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg.0 == u64::MAX {	    if false {	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking
+实审 A2b 第 36 条：所选根的 txg 加二才算溢出（边界）	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg.0 == u64::MAX {	    if root_checkpoint_txg.0 >= u64::MAX - 1 {	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking
 实审 A2b 第 37 条：全池槽 0 都不可择时改回只试最小槽距 4096	crates/singlefs-core/src/recovery.rs	            None => slot_one_found_by_trying_every_slot_spacing_the_format_allows(reader, device)?,	            None => mountable_slot_or_refusal(\n                device,\n                reader.read(device, DeviceOffsetInBytes(FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES), slot_bytes),\n            )?,	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	slot_one_is_found_by_trying_every_spacing_when_no_slot_zero_in_the_pool_verifies
 实审 A2b 第 37 条：逐档按 1024 字节一档试	crates/singlefs-core/src/recovery.rs	const SLOT_SPACING_SEARCH_STEP_BYTES: u64 = 512;	const SLOT_SPACING_SEARCH_STEP_BYTES: u64 = 1024;	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_slot_spacing_that_is_a_multiple_of_512_bytes_only_is_found_too
 实审 A2b 第 37 条：逐档试不核记的槽距等于所在偏移	crates/singlefs-core/src/recovery.rs	                if recorded_spacing != candidate_spacing {	                if recorded_spacing == 0 {	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_self_verifying_slot_whose_recorded_spacing_is_not_its_own_offset_is_not_taken
@@ -1036,7 +1036,7 @@
 实审 C11 三方各算头宽：checker 那份 key 区间只算一个 key	crates/singlefs-checker/src/lib.rs	INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT * u64::from(key_width_byte_at_offset_51);	u64::from(key_width_byte_at_offset_51);	-p singlefs-harness --test index_node_header_width_computed_three_ways_agrees_for_every_key_width -- core_checker_and_model	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte
 实审 C11 三方各算头宽：checker 那份只在 key 宽 128 及以上歪（盘上那一字节丢了最高位，只有扫遍取值域才看得见）	crates/singlefs-checker/src/lib.rs	u64::from(key_width_byte_at_offset_51);	u64::from(key_width_byte_at_offset_51 & 0x7f);	-p singlefs-harness --test index_node_header_width_computed_three_ways_agrees_for_every_key_width -- core_checker_and_model	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte
 实审 C11 三方各算头宽：checker 那份漏了预留位 29	crates/singlefs-checker/src/lib.rs	            + key_range_bytes\n            + NONCE_MAC_ALGORITHM_RESERVED_BYTES,	            + key_range_bytes,	-p singlefs-harness --test index_node_header_width_computed_three_ways_agrees_for_every_key_width -- core_checker_and_model	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte
-实审 C11 三方各算头宽：checker 判单元头时明文头末尾没减预留位	crates/singlefs-checker/src/lib.rs	            let header_end = index_node_header_bytes(unit[51])\n                - usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");	            let header_end = index_node_header_bytes(unit[51]);	-p singlefs-harness --test new_pool_file_creation_publish -- every_index_node_self_checks	every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_hold
+实审 C11 三方各算头宽：checker 判单元头时明文头末尾没减预留位	crates/singlefs-checker/src/lib.rs	            let header_end = index_node_header_bytes(*unit.get(51)?)\n                - usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");	            let header_end = index_node_header_bytes(*unit.get(51)?);	-p singlefs-harness --test new_pool_file_creation_publish -- every_index_node_self_checks	every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_hold
 实审 C11 三方各算头宽：模型那份 key 区间只算一个 key	crates/singlefs-harness/src/model.rs	    let key_range_bytes = key_width_in_bytes * INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT;	    let key_range_bytes = key_width_in_bytes;	-p singlefs-harness --test index_node_header_width_computed_three_ways_agrees_for_every_key_width -- core_checker_and_model	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte
 实审 C11 三方各算头宽：模型那份只在 key 宽 0 上歪（按至少 1 字节算，只有扫到下界才看得见）	crates/singlefs-harness/src/model.rs	    let key_range_bytes = key_width_in_bytes * INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT;	    let key_range_bytes = key_width_in_bytes.max(1) * INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT;	-p singlefs-harness --test index_node_header_width_computed_three_ways_agrees_for_every_key_width -- core_checker_and_model	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte
 实审 C11 三方各算头宽：模型那份漏了预留位 29	crates/singlefs-harness/src/model.rs	    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE + key_range_bytes + NONCE_MAC_ALGORITHM_RESERVED_BYTES	    INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE + key_range_bytes	-p singlefs-harness --test index_node_header_width_computed_three_ways_agrees_for_every_key_width -- core_checker_and_model	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte
@@ -1293,7 +1293,6 @@
 实审 A3c Q1：取号屏障之前不核每块盘两槽读不读得出（读不出的盘拒到屏障之后，录制流多一道屏障）	crates/singlefs-core/src/transaction.rs	    self_verified_system_configurations_of_every_device(pool)?;\n    // 第一个取号写之前一道池屏障	    // 变异：屏障之前不核\n    // 第一个取号写之前一道池屏障	-p singlefs-harness --test acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics -- a_device_whose_slots_turn_unreadable_before_the_acquisition_barrier	a_device_whose_slots_turn_unreadable_before_the_acquisition_barrier_refuses_the_writable_mount_without_a_step
 实审 A3c Q1：屏障之后取见证值那一遍读不出的盘不拒、照写取号写	crates/singlefs-core/src/transaction.rs	        highest_journal_tail_of(&self_verified_system_configurations_of_every_device(pool)?);	        highest_journal_tail_of(&self_verified_system_configurations_of_every_device(pool).unwrap_or_default());	-p singlefs-harness --test acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics -- a_device_whose_slots_turn_unreadable_after_the_acquisition_barrier	a_device_whose_slots_turn_unreadable_after_the_acquisition_barrier_refuses_before_any_write
 实审 A3c Q1：取号那一刻两槽都读不出的盘不报、当作没有（acquire_instance 照写取号写）	crates/singlefs-core/src/transaction.rs	            if u64::try_from(slots_of_this_device.len()).expect("每块盘的系统配置槽数装得进 u64")	            if false && u64::try_from(slots_of_this_device.len()).expect("每块盘的系统配置槽数装得进 u64")	-p singlefs-harness --test acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics -- acquire_instance_refuses	acquire_instance_refuses_a_device_whose_slots_are_unreadable_before_any_step
-实审 A3c Q1：可写挂载把取号那一刻读不出的盘映射错（报成盘 0）	crates/singlefs-core/src/mount.rs	                devices: vec![DeviceWithoutTheSelectedVersion {\n                    device,\n	                devices: vec![DeviceWithoutTheSelectedVersion {\n                    device: DeviceIdentity(u32::from(device.0 == 0)),\n	-p singlefs-harness --test acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics -- a_device_whose_slots_turn_unreadable_before_the_acquisition_barrier	a_device_whose_slots_turn_unreadable_before_the_acquisition_barrier_refuses_the_writable_mount_without_a_step
 实审 A3c：零单元发布不判记录条数上限（在飞上限 0 的环上照写）	crates/singlefs-core/src/transaction.rs	    refuse_a_publish_of_more_journal_records_than_the_limit(\n        writes.records.len(),\n        pool.parameters.geometry.journal_ring_bytes,\n    )?;	    let _ = writes.records.len();	-p singlefs-harness --test acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics -- a_zero_unit_publish_on_a_ring_with_no_record_in_flight	a_zero_unit_publish_on_a_ring_with_no_record_in_flight_is_refused_before_any_write
 实审 A3c C476 缺口 inode_tree.rs:88：inode 记录宽改窄而不重算一容器的上限（盘上一片装得下第 234 条）	crates/singlefs-format/src/lib.rs	pub const INODE_RECORD_BYTES: u64 = 140;	pub const INODE_RECORD_BYTES: u64 = 120;	-p singlefs-core --lib -- inode_tree::tests::a_packed_unit_holds	a_packed_unit_holds_no_more_inode_records_than_the_leaf_record_limit
 实审 A3c C476 缺口 inode_tree.rs:88：码 3 解析不判记录区越过单元末尾（自述 234 条的 inode 叶容器）	crates/singlefs-core/src/unit.rs	    if records_start + declared_length > bytes.len() {	    if false && records_start + declared_length > bytes.len() {	-p singlefs-harness --test acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics -- an_inode_leaf_container_claiming_more_records	an_inode_leaf_container_claiming_more_records_than_a_unit_holds_is_refused_by_the_writable_mount
@@ -1494,4 +1493,31 @@
 崩溃放量判法闭包：取前缀核 singlefs-replay-prefix 停在判器闭包门外（判器闭包里没有它）	crates/singlefs-checker-tier/src/crash_identity.rs	加一个模块不改任何判定",\n    ),\n];	加一个模块不改任何判定",\n    ),\n    (\n        "crates/singlefs-replay-prefix/src/lib.rs",\n        "变异：取前缀核停在判器闭包门外",\n    ),\n];	-p singlefs-checker-tier --lib -- the_judging_closure_of_the_judges_holds	the_judging_closure_of_the_judges_holds_recovery_the_checker_and_the_kernels_and_stops_before_the_pipeline
 崩溃放量判法闭包：取前缀核 singlefs-replay-prefix 停在判器闭包门外（改核判法摘要不变）	crates/singlefs-checker-tier/src/crash_identity.rs	加一个模块不改任何判定",\n    ),\n];	加一个模块不改任何判定",\n    ),\n    (\n        "crates/singlefs-replay-prefix/src/lib.rs",\n        "变异：取前缀核停在判器闭包门外",\n    ),\n];	-p singlefs-checker-tier --lib -- the_judging_digest_ignores	the_judging_digest_ignores_the_pipeline_and_the_progress_module_but_follows_every_judge
 取前缀核单测：每条记录都当成它那次发布的末条（末条没到也施加）	crates/singlefs-replay-prefix/src/lib.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-replay-prefix --lib -- a_publish_whose_last_record_is_missing	a_publish_whose_last_record_is_missing_is_not_applied_and_the_root_segment_stays
-取前缀核单测：施加一次发布之后新根段不换（停在所选根的）	crates/singlefs-replay-prefix/src/lib.rs	        rebuilt = record.new_root_segment;	        rebuilt = root_segment;	-p singlefs-replay-prefix --lib -- a_two_record_publish_after_the_anchor	a_two_record_publish_after_the_anchor_is_applied_whole_and_the_new_root_segment_comes_from_its_last_record
+取前缀核单测：施加一次发布之后新根段不换（停在所选根的）	crates/singlefs-replay-prefix/src/lib.rs	        root_source_after_replay = record.new_root_source;	        root_source_after_replay = chosen_root_source;	-p singlefs-replay-prefix --lib -- a_two_record_publish_after_the_anchor	a_two_record_publish_after_the_anchor_is_applied_whole_and_the_new_root_segment_comes_from_its_last_record
+取前缀核锚点筛选不看 txg（攻方 K19）：所选根那次发布之外带末条标志的也算进锚点，报错的计数器多出别的发布的	crates/singlefs-replay-prefix/src/lib.rs	        if record.instance.0 == root_instance.0 && record.checkpoint_txg.0 == root_checkpoint_txg.0\n            && record_ends_its_publish(record) {	        if record.instance.0 == root_instance.0\n            && record_ends_its_publish(record) {	-p singlefs-harness --test recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw -- two_last_record_flags_in_the_chosen_roots_publish	two_last_record_flags_in_the_chosen_roots_publish_return_the_error_with_only_that_publishs_counters
+取前缀核锚点筛选不看 txg（攻方 K19）：别的发布的末条标志让恢复停在报错上	crates/singlefs-replay-prefix/src/lib.rs	        if record.instance.0 == root_instance.0 && record.checkpoint_txg.0 == root_checkpoint_txg.0\n            && record_ends_its_publish(record) {	        if record.instance.0 == root_instance.0\n            && record_ends_its_publish(record) {	-p singlefs-harness --test recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw -- last_record_flags_of_other_publishes	last_record_flags_of_other_publishes_of_the_same_instance_leave_the_anchor_alone_and_the_next_publish_applies
+胶水把核报的「所选根那次发布带末条标志的多于一条」当不变量破坏 panic（G6 改回去）	crates/singlefs-core/src/recovery.rs	                    counters: counters.iter().map(|counter| counter.0).collect(),	                    counters: unreachable!("核报了所选根那次发布带末条标志的多于一条：{counters:?}"),	-p singlefs-harness --test recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw -- two_last_record_flags_in_the_chosen_roots_publish	two_last_record_flags_in_the_chosen_roots_publish_return_the_error_with_only_that_publishs_counters
+胶水不断言记录表的键等于记录自己的 (实例代号, 计数器)（G2 改回去）	crates/singlefs-core/src/recovery.rs	            *key,\n            (record.instance, record.counter),	            (record.instance, record.counter),\n            (record.instance, record.counter),	-p singlefs-harness --test recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw -- a_table_whose_keys_disagree_with_its_records	a_table_whose_keys_disagree_with_its_records_trips_the_glue_assertion_before_the_kernel_runs
+胶水不断言计数器不是 u64 的最大值（核的前置条件在产品路径上没人担保）	crates/singlefs-core/src/recovery.rs	            record.counter < u64::MAX,	            record.counter <= u64::MAX,	-p singlefs-harness --test recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw -- a_record_whose_counter_is_the_largest_value	a_record_whose_counter_is_the_largest_value_trips_the_glue_assertion_before_the_kernel_runs
+Verus 接回 C5-2（C540 读者规则）：带末条标志的那一条之后同一次发布还有没有记录，只从紧挨着的下一条之后找	crates/singlefs-replay-prefix/src/lib.rs	        if later.instance.0 == record.instance.0 && later.counter.0 >= next_counter {	        if later.instance.0 == record.instance.0 && later.counter.0 > next_counter {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- last_record_flag_followed_only_by_the_very_next_record	last_record_flag_followed_only_by_the_very_next_record_of_the_same_publish_breaks_the_chain_at_the_flag
+Verus 接回 C5-3（D23 已定项 18）：在飞上限差一，一次恢复重放的前缀多施加一条	crates/singlefs-replay-prefix/src/lib.rs	        if records_taken_toward_the_in_flight_limit >= in_flight_limit {	        if records_taken_toward_the_in_flight_limit > in_flight_limit {	-p singlefs-harness --test journal_replay_applies_no_more_records_than_the_in_flight_limit	with_an_in_flight_limit_of_one_only_the_first_of_two_single_record_publishes_above_the_chosen_root_is_applied
+Verus 接回 C5-4（D23 已定项 15）：施加记录重建的根，回退下界 F 取所选根的（可写挂载施加上一个实例的一条遗留记录）	crates/singlefs-core/src/recovery.rs	                rollback_floor: record.new_rollback_floor,	                rollback_floor: root.rollback_floor,	-p singlefs-harness --test writable_mount_takes_a_second_instance -- stray_record_of_the_previous_instance	stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row
+Verus 接回 C5-4（D23 已定项 15）：施加记录重建的根，回退下界 F 取所选根的（锚点读不出、链首接下一个 txg、施加到 jsn 6）	crates/singlefs-core/src/recovery.rs	                rollback_floor: record.new_rollback_floor,	                rollback_floor: root.rollback_floor,	-p singlefs-harness --test writable_mount_takes_a_second_instance -- torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg_when_only_the_anchor_is_torn	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg_when_only_the_anchor_is_torn
+E165 钉住的红例指纹不截 ;first= 起的结局	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	format!("{domain_text}|{size_text}|{state_before_the_outcome}")	format!("{domain_text}|{size_text}|{state_text}")	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_input_fingerprint_recomputed_from_a_pinned_state_row_equals_the_one_the_table_carries
+E165 钉住的表一行不查是不是八段	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	if columns.len() != PINNED_TABLE_COLUMN_COUNT {	if columns.len() < 4 {	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	a_pinned_red_example_row_needs_eight_columns_a_known_domain_a_size_in_range_and_an_ordinal
+E165 钉住的红例重算的指纹只比长度、漂了也判相同	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	Some(recomputed_fingerprint) if recomputed_fingerprint == example.input_fingerprint => {	Some(recomputed_fingerprint) if recomputed_fingerprint.len() == example.input_fingerprint.len() => {	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	a_pinned_red_example_whose_recomputed_fingerprint_differs_is_reported_as_drifted
+E168 U4 保留量少算 ckpt 池那一份（8 + 13c 写成 8 + 12c）	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	(INSTANCE_SWITCHES_PER_MOUNT + 1) * one_switch + checkpoint_cost	(INSTANCE_SWITCHES_PER_MOUNT + 1) * one_switch	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	reserve_by_the_clauses_is_eight_plus_thirteen_checkpoint_costs
+E168 U5 本地 K1 的层级循环少一层	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	(0..root_level)	(0..root_level - 1)	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	allocation_record_tree_positions_match_the_registered_anchors
+E168 U6 根层级判 ≤ 169 写成 < 169	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	if positions_under_the_root <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {	if positions_under_the_root < ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	allocation_record_tree_root_level_steps_up_right_past_the_registered_boundaries
+E168 U7 mkfs 最小盘少算树表那一槽	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	(unit_area_start_slot(journal_ring_bytes) + 3) * SLOT_BYTES	(unit_area_start_slot(journal_ring_bytes) + 2) * SLOT_BYTES	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	make_filesystem_smallest_device_matches_the_registered_anchors
+E168 U8 压满停在可用 ≤ 16384	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	available_bytes <= 0 && available_bytes > -i128::from(SLOT_BYTES)	available_bytes <= i128::from(SLOT_BYTES) && available_bytes > -i128::from(SLOT_BYTES)	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	filling_lands_the_available_bytes_within_one_slot_at_or_below_zero
+E168 压满按下取整少标一槽	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	u64::try_from((available_bytes + slot_bytes - 1) / slot_bytes)	u64::try_from(available_bytes / slot_bytes)	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	filling_lands_the_available_bytes_within_one_slot_at_or_below_zero
+E168 U9 M1 判定 ≤ 0 写成 < 0	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	apex_slots - reserve_at_the_apex_slots <= 0	apex_slots - reserve_at_the_apex_slots < 0	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	m1_counts_an_apex_equal_to_the_reserve_as_covered
+E168 U10 M3 饱和只看最后一点	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	if *second_last <= *before_the_last_two && *last <= *before_the_last_two {	if *last <= *before_the_last_two {	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	m3_is_not_saturated_when_the_second_last_point_rises_and_the_last_falls_back
+E168 U13 环长写成 2S	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	ROOT_RING_REGIONS * slots_per_region	2 * slots_per_region	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	ring_length_is_three_regions_times_the_slots_per_region
+E168 M5 逐点严格增写成不减	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	first < second && second < third	first <= second && second <= third	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	m5_needs_three_strictly_rising_last_points
+E168 差额敏感性每个 δ 少算一圈外那一根	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	let roots_plus_one = i128::from(ring_length_in_roots(slots_per_region) + 1);	let roots_plus_one = i128::from(ring_length_in_roots(slots_per_region));	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	smallest_unbuilt_delta_counts_the_ring_window_per_slot
+E168 M7 环窗口界少乘一圈外那一份	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	(ring_length_in_roots(slots_per_region) + 1) * largest_new_slots_of_a_delete_publish	ring_length_in_roots(slots_per_region) * largest_new_slots_of_a_delete_publish	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	smallest_unbuilt_delta_counts_the_ring_window_per_slot
+E168 梯子 3S + 2 那一点写成 3S + 3	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	ring - 1, ring, ring + 1, ring + 2, ring + 5	ring - 1, ring, ring + 1, ring + 3, ring + 5	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	ladder_of_file_sizes_crosses_the_ring_for_both_batch_sizes
+E168 删除发布次数按下取整	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	1 + data_units.div_ceil(batch_data_units)	1 + data_units / batch_data_units	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	ladder_of_file_sizes_crosses_the_ring_for_both_batch_sizes
+E168 本地分配记录树叶槽数抄错	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;	const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 813;	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	allocation_record_tree_positions_match_the_registered_anchors
```
