# 第 36 批代码轮第一轮：改动全文（2026-10-06）

两份补丁照实现员交回的原样，同一趟合入（在根环线 10o / 10p 合入之后）：36-1b 相对于删除线第 28 批合后的主工作区，36-2 相对于「主工作区 + 36-1b」；按这个次序打，36-1b 在 10o / 10p 合后只是行号偏移。

## m3-formal-36-1b

### crates.patch

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -4926,7 +4926,7 @@
         );
     }
     // 一版都没认出来时的专门理由（判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P3「四支豁免」那一行：
-    // 此前落到 `Judgements::into_report` 的缺省理由，与「这个函数一次都没跑」分不开）。认出来了而一版都没判到（认出来的那几版都没有数据单元）
+    // 此前落到 `Judgements::into_report` 的缺省理由，与「这个函数一次都没跑」分不开）。认出来了而一版都没判到（认出来的那几版在池里的盘上都没有数据单元）
     // 另写一句（用户 2026-10-06 定 Q4「甲 另写一句」，判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第五节第 4 条）。
     // `not_applicable` 不改已经评估过的：认出来的版本里有一版判到了，这一句不出现。
     if versions_built_from_the_mkfs_watermark == 0 {
@@ -4942,8 +4942,10 @@
 /// I-5.5 一次都没判、因为走读到的版本里没有一版满足认法 ①② 时的不适用理由。
 pub const NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK: &str = "回退候选集里每条根与由记录施加出来的那几版，没有一版是从 mkfs 水位第一次建树的那一版（树表非空、种类 1 条目的树 ID 是 65536、每条条目的诞生 txg 等于这一版的 txg）：一版都不判";
 
-/// I-5.5 一次都没判、因为认出来的从 mkfs 水位第一次建树的那几版都没有数据单元时的不适用理由（那一行「V 没有数据单元时不判」）。
-pub const VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT: &str = "认出了从 mkfs 水位第一次建树的那一版，但它们都没有数据单元（extent 树根全零或一条数据指针都没有）：一版都不判";
+/// I-5.5 一次都没判、因为认出来的从 mkfs 水位第一次建树的那几版在池里的盘上都没有数据单元时的不适用理由（那一行「V 没有数据单元时不判」）：
+/// extent 树根全零、一条非零的数据指针都没有，或数据指针的落点全在池里没有的盘上（判法逐盘只看池里有的盘），三样都走到这一句
+/// （判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P4、第三节第 1 条 R3：此前只说「没有数据单元」，后一样实际有）。
+pub const VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT: &str = "认出了从 mkfs 水位第一次建树的那一版，但认出来的那几版在池里的盘上都没有数据单元（extent 树根全零、一条非零的数据指针都没有，或数据指针的落点全在池里没有的盘上）：一版都不判";
 
 /// 一块盘上的开放段：起点是 64 的整数倍的 [`CLUSTER_SEGMENT_SLOTS`] 个槽（D3（空间分配） 已定项 10 ①）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -5595,28 +5597,18 @@
         .then(|| read_u64(header, birth_txg_offset))
 }
 
-/// 最新根之后、下一次挂载的恢复会先施加的那一版的树表指针（改法 E，`research/prompts/m2-wave3-code-r1-main-verification.md`
-/// 第三节 Y4 与第四节第 2 条，被攻过零轮）：环里一条记录与最新根**同实例**、checkpoint_txg = 最新根的 txg + 1、带提交标记，
-/// 根环里没有 (那个实例, 那个 txg) 的根——它那次发布的记录已持久、根槽没落盘，恢复要由记录重建那一版（D23（journal 的角色与格式）
-/// 已定项 15）。树表指针取那条记录新根段里的（D23（journal 的角色与格式） 已定项 4 的字段表）。一版切成几条记录时认的是
-/// **带末条标志的那一条**（改法 FL，判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b 与第三节第 1 条）：
-/// core 的恢复走到末条才整次施加、施加末条那一份新根段（`crates/singlefs-core/src/recovery.rs` 拼新根那一段）；末条还没落的那次发布
-/// core 不施加，这里也不认这一版。两块盘各一份时取先读到的那一份。I-3.10 与 I-5.4 都经这一个函数认这一版，同一个条件。
-///
-/// **只接这一步**：恢复会连着施加几条记录时，第二条及以后那几版的分配记录这里读不到；恢复的前缀规则只复刻反向链那一道
-/// （I-8.6（反向链算法）「不等的记录不进重放前缀」，[`record_can_enter_a_replay_prefix`]），断号即止、点名单元验证、发布边界不复刻——
-/// 读它只多判 I-3.10 一格，它的分配记录与它罩住的单元是同一次发布写出来的，不论施加与否两个数都同源。
+/// 下一次挂载的恢复会先施加的那一版的树表指针（改法 E，`research/prompts/m2-wave3-code-r1-main-verification.md`
+/// 第三节 Y4 与第四节第 2 条，被攻过零轮）：那一版由 [`version_the_next_mount_applies_first`] 认，树表指针取它新根段里的
+/// （D23（journal 的角色与格式） 已定项 4 的字段表）。I-3.10 经这里读，I-5.4 按版本那一层读同一个函数认出的同一版。
 fn tree_table_pointer_of_the_version_the_next_mount_applies_first(
-    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
-    roots: &[(u64, u64, crate::RootView)],
-    newest_index: usize,
+    version_the_next_mount_applies_first: Option<&VersionTheNextMountAppliesFirst>,
 ) -> Option<PointerView> {
-    version_the_next_mount_applies_first(records_by_device, roots, newest_index).map(|version| {
+    version_the_next_mount_applies_first.map(|version| {
         parse_node_pointer(&version.new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER])
     })
 }
 
-/// 下一次挂载的恢复会先施加的那一版：那次发布的末条记录已持久、根槽没落盘、还没有哪一次恢复施加过它（与 [`VersionAppliedOnlyByRecords`] 不是一类：
+/// 下一次挂载的恢复会先施加的那一版：重放施加的第一次发布，根槽没落盘、还没有哪一次恢复施加过它（与 [`VersionAppliedOnlyByRecords`] 不是一类：
 /// 那一类已被施加、实例表里有行）。
 struct VersionTheNextMountAppliesFirst {
     instance: u32,
@@ -5624,42 +5616,54 @@
     new_root_segment: Vec<u8>,
 }
 
-/// 下一次挂载的恢复会先施加的那一版（认法见 [`tree_table_pointer_of_the_version_the_next_mount_applies_first`]）：
-/// 它的 (实例代号, txg) 与那条记录的整段新根段。I-3.10 只取树表指针，I-5.4 按版本那一层还要「树表 0 条那一版的分配记录树根」。
+/// 下一次挂载的恢复会先施加的那一版：它的 (实例代号, txg) 与那次发布末条记录的整段新根段。I-3.10 只取树表指针
+/// （[`tree_table_pointer_of_the_version_the_next_mount_applies_first`]），I-5.4 按版本那一层还要「树表 0 条那一版的分配记录树根」；
+/// 两条都经这一个函数认这一版，同一个条件。
+///
+/// 认法（改法 R1，判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1 与第三节第 1 条）：所选根取最新根
+/// （与 I-4.1 同一个择法），照 I-4.1 那一份重放（[`replay_the_next_mount_would_do`]）施加的**第一次**发布、取它末条那一份新根段
+/// （已定项 15：core 施加末条那一份，`crates/singlefs-core/src/recovery.rs` 拼新根那一段）。core 的施加单位是一次发布：
+/// 从链首起连续走到末条才整次施加，末条没落、同一次发布前面某条没落（断号）、反向链对不上、序号或 txg 接不上、末条之后同一次发布还有记录、
+/// 点名单元验不过的那次发布都不施加；这里用同一份重放，那次发布同样不认（此前只看带末条标志的那一条，断号的发布 core 不施加、这里照认）。
+/// 重放一次发布都不施加、锚点认不定、所选根 txg 没有后继时没有这一版。前缀停在后面某次发布的点名单元上时，停之前整次收下的发布
+/// core 照施加，第一次那一版照认（I-4.1 对这种镜像报不适用，那是 I-4.1 自己的射程）。这一版的 txg 比最新根大，根环里没有它的根。
+///
+/// **只接这一步**：恢复连着施加几次发布时，第二次及以后那几版的分配记录这里读不到；读它只多判 I-3.10 一格，
+/// 它的分配记录与它罩住的单元是同一次发布写出来的，不论施加与否两个数都同源。
 fn version_the_next_mount_applies_first(
+    reader: &dyn ImageReader,
     records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
-    roots: &[(u64, u64, crate::RootView)],
-    newest_index: usize,
+    newest_root: &crate::RootView,
+    journal_ring_bytes: u64,
 ) -> Option<VersionTheNextMountAppliesFirst> {
-    let newest_root = &roots[newest_index].2;
-    let next_txg = newest_root.checkpoint_txg.checked_add(1)?;
-    let root_slot_is_readable = roots.iter().any(|(_, _, root)| {
-        root.instance == newest_root.instance && root.checkpoint_txg == next_txg
-    });
-    if root_slot_is_readable {
-        return None;
-    }
-    // 按恢复的读法（每个计数器取第一块盘上的那一份）：反向链对不上的记录不进重放前缀（I-8.6（反向链算法）），
-    // 下一次挂载不会施加它，它那一版不在这里读（实审 A2a 报告「checker 要跟着改什么」第 3 条）。
-    let records_as_the_recovery_reads_them = records_as_the_recovery_reads_them(records_by_device);
-    records_as_the_recovery_reads_them
-        .iter()
-        .find(|((_, counter), record)| {
-            record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0
-                && record.commit_marker == CommitMarker::Present
-                && record.instance == newest_root.instance
-                && record.checkpoint_txg == next_txg
-                && record_can_enter_a_replay_prefix(
-                    &records_as_the_recovery_reads_them,
-                    *counter,
-                    record,
-                )
-        })
-        .map(|(_, record)| VersionTheNextMountAppliesFirst {
-            instance: record.instance,
-            checkpoint_txg: record.checkpoint_txg,
-            new_root_segment: record.new_root_segment.clone(),
-        })
+    let records_the_replay_reads = records_the_replay_reads(records_by_device);
+    let last_record_of_the_first_publish_applied = match replay_the_next_mount_would_do(
+        reader,
+        &records_the_replay_reads,
+        newest_root,
+        in_flight_record_limit_of_the_journal_ring(journal_ring_bytes),
+    ) {
+        ReplayTheNextMountWouldDo::AnchorUndecided
+        | ReplayTheNextMountWouldDo::ChosenRootTxgHasNoSuccessor
+        | ReplayTheNextMountWouldDo::NothingApplied => None,
+        ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {
+            last_record_of_the_first_publish_applied_before_it,
+        } => last_record_of_the_first_publish_applied_before_it,
+        ReplayTheNextMountWouldDo::Applied {
+            last_record_of_the_first_publish,
+            publishes: _,
+            records: _,
+            last_record_counter: _,
+            last_record: _,
+        } => Some(last_record_of_the_first_publish),
+    }?;
+    Some(VersionTheNextMountAppliesFirst {
+        instance: last_record_of_the_first_publish_applied.instance,
+        checkpoint_txg: last_record_of_the_first_publish_applied.checkpoint_txg,
+        new_root_segment: last_record_of_the_first_publish_applied
+            .new_root_segment
+            .clone(),
+    })
 }
 
 /// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）读哪几片分配记录：候选集里每条根的分配记录树
@@ -8235,16 +8239,22 @@
     /// 所选根的 `checkpoint_txg` 已是 u64 的最大值，接在它后面的发布取不到 txg：实现报损坏、一条都不施加。
     ChosenRootTxgHasNoSuccessor,
     /// 前缀停在一条点名单元两份没都验过的记录上（第四条）：起点含介质损坏，用户 2026-10-01 定这种镜像不算 I-4.1 的合法镜像。
-    StoppedAtANamedUnitThatDoesNotVerify,
+    /// 停之前已经整次收下的发布 core 照施加（`crates/singlefs-core/src/recovery.rs` 的 `replay_journal`：前缀停在验不过的那一条之前，
+    /// 施加到它之前最后一个末条为止）；`last_record_of_the_first_publish_applied_before_it` 是其中第一次发布的末条，一次都没收下时是 `None`。
+    StoppedAtANamedUnitThatDoesNotVerify {
+        last_record_of_the_first_publish_applied_before_it: Option<&'records ScannedJournalRecord>,
+    },
     /// 一次发布都没施加：所选根那一版就是下一次挂载择到的那一版，它走不走得完归 I-7.2（最新根可遍历）。
     NothingApplied,
     /// 施加了 `publishes` 次发布、共 `records` 条记录；`last_record` 是最后一次发布的末条，施加出来的那一版照它的新根段
-    /// （已定项 15：一次发布的每条记录带同一份新根段）。
+    /// （已定项 15：一次发布的每条记录带同一份新根段）。`last_record_of_the_first_publish` 是第一次发布的末条：
+    /// 下一次挂载先施加的那一版照它的新根段（[`version_the_next_mount_applies_first`]，I-3.10 与 I-5.4 读它）。
     Applied {
         publishes: u64,
         records: u64,
         last_record_counter: u64,
         last_record: &'records ScannedJournalRecord,
+        last_record_of_the_first_publish: &'records ScannedJournalRecord,
     },
 }
 
@@ -8261,6 +8271,11 @@
         && record.ordinal_within_publish != 0
 }
 
+/// 重放的在飞上限：按环长与格式常量 F 现算（D23（journal 的角色与格式） 已定项 18），I-4.1 与下一次挂载先施加的那一版用同一个数。
+fn in_flight_record_limit_of_the_journal_ring(journal_ring_bytes: u64) -> u64 {
+    journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR
+}
+
 /// 重放前缀读的那一份环：键是 (实例代号, 计数器)，每个键取按盘的次序第一块盘上恢复读者读得出的那一份
 /// （「两份镜像任一份自证校验和过即在」，已定项 14）。与实现扫环建表同一个键（`crates/singlefs-core/src/recovery.rs` 的 `scan_journal`
 /// 按 (实例代号, 计数器) 先到先得）：两块盘同一计数器坐着不同实例的记录时两条都留下，不拿先读到的那块盘上的别的实例那一条挤掉
@@ -8386,6 +8401,7 @@
     let mut publishes_applied: u64 = 0;
     let mut records_applied: u64 = 0;
     let mut last_applied: Option<(u64, &ScannedJournalRecord)> = None;
+    let mut last_record_of_the_first_publish_applied: Option<&ScannedJournalRecord> = None;
     let above_the_chosen_root = records_the_replay_reads.iter().filter(|(_, record)| {
         record.instance == chosen_root.instance
             && record.checkpoint_txg > chosen_root.checkpoint_txg
@@ -8439,7 +8455,10 @@
             break;
         }
         if !named_units_of_the_record_verify_on_both_copies(reader, record) {
-            return ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify;
+            return ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {
+                last_record_of_the_first_publish_applied_before_it:
+                    last_record_of_the_first_publish_applied,
+            };
         }
         expected_next_counter = Some(
             counter
@@ -8450,6 +8469,8 @@
         if ends_its_publish {
             publishes_applied += 1;
             records_applied += u64::try_from(open_publish.len()).expect("一次发布的记录条数");
+            // 施加出来的那一版照末条那一份新根段（已定项 15），第一次发布的那一版同样取它的末条（改法 FL 的那一取法）。
+            last_record_of_the_first_publish_applied.get_or_insert(record);
             open_publish.clear();
             last_applied = Some((counter, record));
         }
@@ -8461,6 +8482,9 @@
             records: records_applied,
             last_record_counter,
             last_record,
+            last_record_of_the_first_publish: last_record_of_the_first_publish_applied.expect(
+                "一次发布收到末条时 last_applied 与它在同一处写进 Some、之后都不回 None：last_applied 是 Some，它就是 Some",
+            ),
         },
     }
 }
@@ -8478,13 +8502,12 @@
     filesystem_identifier_low: u64,
     judgements: &mut Judgements,
 ) {
-    let in_flight_record_limit = journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;
     let records_the_replay_reads = records_the_replay_reads(records_by_device);
     match replay_the_next_mount_would_do(
         reader,
         &records_the_replay_reads,
         chosen_root,
-        in_flight_record_limit,
+        in_flight_record_limit_of_the_journal_ring(journal_ring_bytes),
     ) {
         ReplayTheNextMountWouldDo::AnchorUndecided => judgements.not_applicable(
             TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
@@ -8494,7 +8517,9 @@
             TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
             "所选根的 checkpoint_txg 是 u64 的最大值：接在它后面的发布取不到 txg，重放不出一版",
         ),
-        ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify => {
+        ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {
+            last_record_of_the_first_publish_applied_before_it: _,
+        } => {
             judgements.not_applicable(
                 TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
                 "重放前缀停在一条点名单元两份没都验过的记录上：起点含介质损坏，不在 I-4.1 判的合法镜像里",
@@ -8509,6 +8534,7 @@
             records,
             last_record_counter,
             last_record,
+            last_record_of_the_first_publish: _,
         } => {
             let replayed_version = VersionAppliedOnlyByRecords {
                 instance: last_record.instance,
@@ -11004,8 +11030,12 @@
     );
     // I-5.4 按版本那一层：回退候选集里每条根，加由记录施加出来的那几版与下一次挂载先施加的那一版（改法 RV，见
     // [`judge_allocation_records_disjoint`]）。坏盘上同一版两类都认得出时判两遍：同一棵分配记录树按根节点只读一次，违例不重报。
-    let version_the_next_mount_applies_first =
-        version_the_next_mount_applies_first(&journal_records_by_device, &roots, newest_index);
+    let version_the_next_mount_applies_first = version_the_next_mount_applies_first(
+        reader,
+        &journal_records_by_device,
+        &roots[newest_index].2,
+        geometry.journal_ring_bytes,
+    );
     let versions_to_judge_the_allocation_records_of: Vec<VersionToJudgeTheAllocationRecordsOf<'_>> =
         candidate_indexes
             .iter()
@@ -11055,9 +11085,7 @@
         &candidate_indexes,
         &versions_applied_only_by_records,
         tree_table_pointer_of_the_version_the_next_mount_applies_first(
-            &journal_records_by_device,
-            &roots,
-            newest_index,
+            version_the_next_mount_applies_first.as_ref(),
         ),
         &mut index_node_cache,
     );
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -91,7 +91,7 @@
 };
 use singlefs_checker_tier::gpu_unit_checks::GpuCard;
 use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
-use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration};
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
 use singlefs_core::container_tree::{
     ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
 };
@@ -101,7 +101,8 @@
     TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
 };
 use singlefs_core::transaction::{
-    publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty, PoolWriter,
+    publish_overwrite, publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty,
+    FirstFile, PoolWriter,
 };
 use singlefs_core::unlink::UnlinkBatchSize;
 use singlefs_harness::memory_pool::{
@@ -112,6 +113,7 @@
     number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
 };
 use singlefs_harness::segments::StepKind;
+use singlefs_harness::SharedStream;
 
 const EXPAND_UP_TO_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO";
 const DEFAULT_EXPAND_UP_TO: usize = 8;
@@ -8006,39 +8008,112 @@
     );
 }
 
-/// 三个数据单元的第一个文件之后可写挂载（取号 2、写行、暖机两次）、实例 2 再覆盖写一次，然后从流里拿掉顺序写那次发布（txg 4）的根槽写：
-/// txg 4 那一版只在它的三条记录里，挂载之前的状态上它是下一次挂载先施加的那一版，挂载写了实例 1 的行之后它是由记录施加出来、
-/// 根槽从没落盘的那一版（攻方报告 P1-b 的 s4 形）。`rewrite_the_records` 再改那次发布的记录（重封、重链）。
-/// 挂载在录制时读到的是 txg 4 的根；拿掉根槽写之后，恢复施加那三条记录重建出的是同一版（新根段与根同七段，D23（journal 的角色与格式）
-/// 已定项 15），挂载写出的行与之后的发布不变。
+/// 三个数据单元的第一个文件那段历史（`pool_with_a_three_unit_file`：新池新建文件 txg 3、顺序写三个数据单元那次发布 txg 4、三条记录）
+/// 录到 txg 4 的根槽写之前停——崩在那次根槽 FUA 之前，三条记录两块盘都落了；`rewrite_the_records_before_the_mount` 在这一段写上改那次发布的记录。
+/// 再从这一段写全落的那张崩溃镜像起新进程（另一条录制流）录可写挂载（施加那三条记录、取号 2、写行、暖机两次）与实例 2 的一次覆盖写，
+/// 两段写接成一条流。txg 4 那一版只在它的三条记录里：挂载之前的状态上它是下一次挂载先施加的那一版，挂载写了实例 1 的行之后它是由记录施加出来、
+/// 根槽从没落盘的那一版（攻方报告 P1-b 的 s4 形）。挂载读到的就是这条流里那三条记录（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md`
+/// 第二节 P3 与第三节第 1 条 R2：此前录完整段历史再从流里拿掉 txg 4 的根槽写，录制时挂载读到的是 txg 4 的根、什么都没施加，
+/// 写出的实例表行（W）与之后的发布、根槽之后的系统配置轮换都与合法历史不同）。
 fn record_three_unit_file_applied_only_by_its_records_after_a_mount(
     tag: &str,
     name: &'static str,
-    rewrite_the_records: &dyn Fn(&mut RecordedFlow),
+    rewrite_the_records_before_the_mount: &dyn Fn(&mut RecordedFlow),
 ) -> RecordedFlow {
-    let mut pool = pool_with_a_three_unit_file(tag);
-    let mut devices_reopened_for_the_second_instance = pool.reopen_recorded();
-    let mounted = singlefs_core::mount::mount_writable(
-        &parameters(),
-        &mut devices_reopened_for_the_second_instance,
-    )
-    .expect("重开取号 2");
-    pool.devices = Some(devices_reopened_for_the_second_instance);
-    pool.allocator = mounted.allocator;
+    let pool = pool_with_a_three_unit_file(tag);
+    let operations = pool.retained_operations();
+    let classifier = geometry();
+    let operations_after_mkfs = &operations[pool.mkfs_operation_count..];
+    let root_of_the_three_unit_publish = operations_after_mkfs
+        .iter()
+        .position(|retained| {
+            classifier.classify(&retained.operation) == StepKind::RootRecordFua
+                && retained.contents.as_ref().is_some_and(|bytes| {
+                    u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
+                        && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == 4
+                })
+        })
+        .expect("流里有顺序写那次发布 (1, 4) 的根槽写");
+    let (writes, segments, _stream_indexes_before_the_crash) =
+        writes_and_segments_with_stream_indexes(
+            &operations_after_mkfs[..root_of_the_three_unit_publish],
+            &classifier,
+        );
+    let mut flow = RecordedFlow {
+        name,
+        base: pool.memory_pool_after_mkfs(),
+        judged_root_index: writes
+            .iter()
+            .rposition(|write| write.kind == StepKind::RootRecordFua)
+            .expect("顺序写之前有新建文件那次发布 (1, 3) 的根槽写"),
+        writes,
+        segments,
+        versions: Vec::new(),
+    };
+    rewrite_the_records_before_the_mount(&mut flow);
+
+    let stream_of_the_new_process = SharedStream::retaining_contents();
+    let every_write_landed = vec![true; flow.writes.len()];
+    let mut devices = common::crash_state_devices(
+        &flow.base,
+        &flow.writes,
+        &every_write_landed,
+        &stream_of_the_new_process,
+    );
+    let mut mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices)
+        .expect("崩在 txg 4 根槽之前的镜像上新进程可写挂载：施加那三条记录、取号 2");
+    assert_eq!(
+        (
+            mounted.output.chosen_root.checkpoint_txg,
+            mounted.output.effective_root.checkpoint_txg,
+            mounted.output.journal.prefix_applied
+        ),
+        (CheckpointTxg(3), CheckpointTxg(4), 3),
+        "流 {name}：择 (1, 3) 的根，施加 txg 4 的三条记录，现行版是 (1, 4)"
+    );
     let current = mounted
         .current
         .into_file_version()
-        .expect("三个数据单元那一版之后重开，现行那一版带文件");
-    let overwrite = common::publish_overwrite_in_process(
-        &mut pool,
-        &current,
-        &third_content(),
-        common::FIXED_WRITE_TIME_SECONDS + 120,
-        InstanceGeneration(2),
-    )
-    .expect("实例 2 覆盖写");
+        .expect("施加了 txg 4，现行那一版带文件");
+    let overwrite = {
+        let publish_parameters = parameters();
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        publish_overwrite(
+            &mut writer,
+            &mut mounted.allocator,
+            &current,
+            FirstFile {
+                content: &third_content(),
+                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 120,
+            },
+            InstanceGeneration(2),
+        )
+        .expect("实例 2 覆盖写")
+    };
+    let (
+        writes_of_the_new_process,
+        segments_of_the_new_process,
+        _stream_indexes_of_the_new_process,
+    ) = writes_and_segments_with_stream_indexes(
+        &stream_of_the_new_process.retained_operations(),
+        &classifier,
+    );
+    let first_write_of_the_new_process = flow.writes.len();
+    flow.writes.extend(writes_of_the_new_process);
+    flow.segments
+        .extend(segments_of_the_new_process.into_iter().map(|segment| {
+            segment
+                .into_iter()
+                .map(|index| first_write_of_the_new_process + index)
+                .collect()
+        }));
+    flow.judged_root_index = flow
+        .writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("覆盖写那次发布有根槽写");
     let overwrite_txg = overwrite.root.checkpoint_txg.0;
-    let mut versions = vec![
+    flow.versions = vec![
         PublishedVersion {
             instance: InstanceGeneration(1),
             checkpoint_txg: CheckpointTxg(3),
@@ -8051,46 +8126,56 @@
         },
     ];
     for txg in 5..overwrite_txg {
-        versions.push(PublishedVersion {
+        flow.versions.push(PublishedVersion {
             instance: InstanceGeneration(2),
             checkpoint_txg: CheckpointTxg(txg),
             content: content_of_three_data_units(),
         });
     }
-    versions.push(PublishedVersion {
+    flow.versions.push(PublishedVersion {
         instance: InstanceGeneration(2),
         checkpoint_txg: CheckpointTxg(overwrite_txg),
         content: third_content(),
     });
-    let mut flow = recorded_flow_of_the_pool(&pool, name, versions);
-    let root_of_the_three_unit_publish = flow
-        .writes
-        .iter()
-        .position(|write| {
-            write.kind == StepKind::RootRecordFua
-                && write.bytes().is_some_and(|bytes| {
-                    u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
-                        && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == 4
-                })
-        })
-        .expect("流里有顺序写那次发布 (1, 4) 的根槽写");
-    flow.writes.remove(root_of_the_three_unit_publish);
-    for segment in &mut flow.segments {
-        segment.retain(|index| *index != root_of_the_three_unit_publish);
-        for index in segment.iter_mut() {
-            if *index > root_of_the_three_unit_publish {
-                *index -= 1;
-            }
+    flow
+}
+
+/// 在流尾另起一段，把 `checkpoint_txg` 那次发布（实例 1）的每一份 journal 记录写（两块盘各一份）抄一遍、`rewrite` 改过、重封，
+/// 同盘里计数器接着的那一条的反向链按前一条改后的头重算：这一段全落的状态上盘面就是「整段历史写完之后那几条记录被改成这样」。
+/// 流里原来那几次记录写不动，挂载录制时读到的仍是它们。
+fn append_the_records_of_the_publish_rewritten(
+    flow: &mut RecordedFlow,
+    checkpoint_txg: u64,
+    rewrite: &dyn Fn(&mut [u8]),
+) {
+    let mut records = journal_record_writes_of_the_publish(flow, checkpoint_txg);
+    records.sort_by_key(|(index, counter)| (flow.writes[*index].device, *counter));
+    let mut rewritten: Vec<RetainedWrite> = Vec::new();
+    let mut previous_on_the_device: Option<(DeviceIdentity, u64, u32)> = None;
+    for (index, counter) in records {
+        let mut write = flow.writes[index].clone();
+        let device = write.device;
+        let WrittenContents::Bytes(bytes) = &mut write.contents else {
+            panic!("记录写是普通写");
+        };
+        rewrite(bytes);
+        if let Some((_, _, chain)) =
+            previous_on_the_device.filter(|(previous_device, previous_counter, _)| {
+                *previous_device == device && previous_counter + 1 == counter
+            })
+        {
+            bytes[91..95].copy_from_slice(&chain.to_le_bytes());
         }
+        reseal_journal_record(bytes);
+        previous_on_the_device = Some((device, counter, journal_chain_value(bytes)));
+        rewritten.push(write);
     }
-    flow.segments.retain(|segment| !segment.is_empty());
-    flow.judged_root_index = flow
-        .writes
-        .iter()
-        .rposition(|write| write.kind == StepKind::RootRecordFua)
-        .expect("覆盖写那次发布有根槽写");
-    rewrite_the_records(&mut flow);
-    flow
+    assert!(
+        !rewritten.is_empty(),
+        "流 {} 里有 txg {checkpoint_txg} 那次发布的 journal 记录",
+        flow.name
+    );
+    append_a_segment_of_writes(flow, rewritten);
 }
 
 /// 改法 FL 由记录施加出来的那一版那一支（攻方报告 P1-b 的 s4 形）：txg 4 只在三条记录里、第一条的新根段写成 txg 3 那条根的。
@@ -8168,26 +8253,22 @@
     );
 }
 
-/// Q1 甲由记录施加出来的那一版那一支：同上那条挂载之后的流，txg 4 的三条记录（两块盘各一份）新根段里的树表指针都写零、重封、重链。
-/// 挂载之后的状态上 txg 4 是由记录施加出来的那一版，CPU 照 core 跟过去、这一版走不完（红 I-2.1、I-4.8、I-7.4）；
-/// GPU 走读内核同判。挂载之前三条都落了的状态上它是下一次挂载施加的那一版，重放走不完（I-4.1）。
+/// Q1 甲由记录施加出来的那一版那一支：同上那条挂载之后的流（挂载读的是干净的三条记录），覆盖写之后在流尾另起一段，把 txg 4 的三条记录
+/// （两块盘各一份）抄一遍、新根段里的树表指针都写零、重封、重链（挂载之后改记录，`checker_known_bad_images.rs` 同一造法）。
+/// 那一段落了的状态上 txg 4 是由记录施加出来的那一版，CPU 照 core 跟过去、这一版走不完（红 I-2.1、I-4.8、I-7.4）；GPU 走读内核同判。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_by_them_carry_an_all_zero_tree_table_pointer(
 ) {
-    let flow = record_three_unit_file_applied_only_by_its_records_after_a_mount(
+    let mut flow = record_three_unit_file_applied_only_by_its_records_after_a_mount(
         "crash-judge-gpu-all-zero-tree-table-pointer-applied-only-by-records",
         "three-unit-file-applied-only-by-its-records-all-zero-tree-table-pointer",
-        &|flow| {
-            let records = journal_record_writes_of_the_publish(flow, 4);
-            for (index, _) in records {
-                rewrite_journal_record_and_reseal(flow, index, &|bytes| {
-                    bytes[JOURNAL_NEW_ROOT_SEGMENT_OFFSET
-                        ..JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NODE_POINTER_BYTES]
-                        .fill(0);
-                });
-            }
-        },
+        &|_records_left_as_written| {},
     );
+    append_the_records_of_the_publish_rewritten(&mut flow, 4, &|bytes| {
+        bytes
+            [JOURNAL_NEW_ROOT_SEGMENT_OFFSET..JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NODE_POINTER_BYTES]
+            .fill(0);
+    });
     let Some(card) = one_usable_card_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_by_them_carry_an_all_zero_tree_table_pointer",
     ) else {
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -9,9 +9,11 @@
 use common::{
     abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
     copy_each_root_into_the_new_root_segment_of_its_journal_records, crash_state_devices,
-    format_pool, geometry, memory_pool_of_sparse_devices, parameters, publish_overwrite_in_process,
-    rewrite_journal_record_and_rechain_the_records_after_it, BuiltPool, FIXED_WRITE_TIME_SECONDS,
-    IMAGE_BYTES,
+    format_pool, geometry, memory_pool_of_sparse_devices,
+    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
+    publish_overwrite_in_process, rewrite_journal_record_and_rechain_the_records_after_it,
+    tear_the_root_slot_written_for_txg, BuiltPool, CONTENT_BYTES_OF_PUBLISH_B,
+    CONTENT_BYTES_OF_PUBLISH_C, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
 };
 use singlefs_checker::image::{
     InvariantVerdict, ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
@@ -11636,6 +11638,288 @@
     );
 }
 
+/// 这次发布写的第一个数据单元，在这次发布重写的分配记录叶里它那两条未释放记录（两块盘各一条，分在两片叶里）的分配代从这次发布的
+/// txg 改成 txg + 1，叶重封、新的整单元校验和沿引用链补上去，再补进引用改过的单元的那几条记录；数据单元一个字节不动。
+/// 盘上留下的是 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 的违例。交回那个数据单元的起点槽。
+fn raise_the_allocation_generation_of_a_data_unit_the_publish_wrote_past_its_birth(
+    image: &mut MemoryPool,
+    version: &TransactionOutput,
+    record_copies_referencing_the_changed_units: usize,
+) -> u64 {
+    let units = units_found_on_device_zero(image);
+    let checksums_before = unit_checksums(image, &units);
+    let data_unit_slot = version
+        .units
+        .iter()
+        .find(|unit| {
+            matches!(unit.identity, TransactionUnit::Data(_))
+                && version.rewritten.contains(&unit.identity)
+        })
+        .expect("这次发布写了数据单元")
+        .slot
+        .0;
+    let birth_txg = version.root.checkpoint_txg.0;
+    let leaves: Vec<u64> = version
+        .units
+        .iter()
+        .filter(|unit| {
+            version.rewritten.contains(&unit.identity)
+                && matches!(
+                    unit.identity,
+                    TransactionUnit::AllocationTree
+                        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
+                )
+        })
+        .map(|unit| unit.slot.0)
+        .filter(|slot| {
+            let mut candidate_leaf = read(image, 0, slot * SLOT, 16384);
+            candidate_leaf[INDEX_NODE_LEVEL_OFFSET] == 0
+                && rewrite_unreleased_allocation_generation(
+                    &mut candidate_leaf,
+                    data_unit_slot,
+                    birth_txg,
+                    birth_txg + 1,
+                ) == 1
+        })
+        .collect();
+    assert_eq!(
+        leaves.len(),
+        2,
+        "数据单元槽 {data_unit_slot} 两块盘各一份落点，两条未释放记录分在两块盘各自那片叶里"
+    );
+    for leaf in leaves {
+        mutate_unit_of(
+            image,
+            &units,
+            leaf,
+            |bytes| {
+                assert_eq!(
+                    rewrite_unreleased_allocation_generation(
+                        bytes,
+                        data_unit_slot,
+                        birth_txg,
+                        birth_txg + 1
+                    ),
+                    1,
+                    "上面挑叶时这一片改得动"
+                );
+            },
+            true,
+        );
+    }
+    let changes = units_whose_checksum_changed(&checksums_before, &unit_checksums(image, &units));
+    assert_eq!(
+        propagate_into_journal_records(image, &changes),
+        record_copies_referencing_the_changed_units,
+        "只有这次发布落了的那几条记录（两块盘各一份）引用这一版改过的单元"
+    );
+    data_unit_slot
+}
+
+/// 改法 R1（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1 与第三节第 1 条，攻方报告
+/// `research/prompts/m3-formal-code-b35-r1-opus-output.md` P1 第 2 条 `cut=records-2..2-of-2` 那一形）：发布 B (1, 4) 切成三条记录，
+/// 首条（计数器 4，两块盘各一份）没落、后两条落了，崩在根槽 FUA 之前——断号。core 从 A 的锚点起要计数器 4、读到的是 5，前缀断在这里，
+/// B 整次不施加。B 重写的分配记录叶里放两处坏法：两条记录罩住同一个槽（I-5.4 的形），B 的一个数据单元那两条未释放记录分配代改成 txg + 1
+/// （I-3.10 的形）；新的整单元校验和补进 B 的三条记录、后两条重链。
+/// 阳性对照：三条都落了时 B 是下一次挂载先施加的那一版，I-3.10 与 I-5.4 都红在 B 上。
+/// 首条没落时 checker 照 I-4.1 那一份重放认这一版：B 不施加就不认，I-5.4 与 I-3.10 都不读 B 的分配记录树，这张镜像一条都不红
+/// （只看带末条标志的那一条、即计数器 6 时认 B，两条照红）；可写挂载照常收下：择 A 的根，一条记录都不施加，现行版仍是 A。
+#[test]
+fn a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first() {
+    let crash = crash_image_with_a_three_record_publish_but_not_its_root(
+        "known-good-publish-with-a-gap-before-its-last-record",
+    );
+    let mut image = crash.image.clone();
+    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(&mut image, &crash.publish, 6);
+    raise_the_allocation_generation_of_a_data_unit_the_publish_wrote_past_its_birth(
+        &mut image,
+        &crash.publish,
+        6,
+    );
+    let first_counter = crash
+        .publish
+        .earlier_records_of_this_publish
+        .first()
+        .expect("B 有三条记录")
+        .record
+        .counter;
+    rewrite_journal_record_and_rechain_the_records_after_it(
+        &mut image,
+        first_counter,
+        |_record_kept_as_is| {},
+    );
+    let verdicts_with_every_record_landed = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts_with_every_record_landed),
+        ["I-3.10", "I-5.4"],
+        "阳性对照：B 的三条记录都落了，B 是下一次挂载先施加的那一版，两处坏法各红一条：{verdicts_with_every_record_landed:?}"
+    );
+
+    // mkfs 把整个 journal 环写清零（`crates/singlefs-core/src/make_filesystem.rs` 里 `write_zeroes_at(journal_ring_start, …)`），
+    // 计数器 4 那一格在 B 之前是全零：写回全零就是首条那两次写没落。
+    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
+    for device in DEVICES {
+        write(
+            &mut image,
+            device,
+            record_offset(first_counter, JOURNAL_RING_DEFAULT_BYTES).0,
+            &vec![0u8; record_bytes],
+        );
+    }
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "B 断了号、core 不施加：B 的分配记录树不在 I-5.4、I-3.10 判的版本里，两处坏法都不红：{verdicts:?}"
+    );
+    for invariant in ["I-5.4", "I-3.10"] {
+        assert_eq!(
+            verdict(&image, invariant),
+            InvariantVerdict::Holds,
+            "{invariant} 在 A 那一版上真被评估过且成立"
+        );
+    }
+
+    let stream = SharedStream::new();
+    let mut devices = crash_state_devices(&image, &[], &[], &stream);
+    let mounted = mount_writable(&parameters(), &mut devices)
+        .expect("B 不施加：可写挂载照 A 那一版收下，B 的分配记录树它不读");
+    assert_eq!(
+        (
+            mounted.output.chosen_root.checkpoint_txg,
+            mounted.output.effective_root.checkpoint_txg,
+            mounted.output.journal.prefix_applied
+        ),
+        (CheckpointTxg(3), CheckpointTxg(3), 0),
+        "择 A 的根 (1, 3)，前缀断在计数器 5（要的是 4），一条记录都不施加"
+    );
+}
+
+/// A 的根落了之后实例 1 再覆盖写两次：B (1, 4)、C (1, 5)，各一条记录（计数器 4、5）。交回整段历史写完的镜像（根槽都在）与 B、C 两次发布的输出；
+/// 要「B、C 都只在记录里」的，改完单元之后再把两次的根槽改坏（`tear_the_root_slot_written_for_txg`：改单元时沿引用链重封根槽，先改坏会被封回去）。
+struct TwoSingleRecordPublishes {
+    image: MemoryPool,
+    first_publish: TransactionOutput,
+    second_publish: TransactionOutput,
+}
+
+fn image_with_two_single_record_publishes(tag: &str) -> TwoSingleRecordPublishes {
+    let mut pool = build_pool(tag);
+    let content_of = |length: usize, seed: usize| -> Vec<u8> {
+        (0..length)
+            .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
+            .collect()
+    };
+    let first_publish = overwrite_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &content_of(CONTENT_BYTES_OF_PUBLISH_B, 3),
+    );
+    let second_publish = overwrite_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &content_of(CONTENT_BYTES_OF_PUBLISH_C, 11),
+    );
+    assert_eq!(
+        (
+            (
+                first_publish.root.checkpoint_txg,
+                first_publish.record.counter
+            ),
+            (
+                second_publish.root.checkpoint_txg,
+                second_publish.record.counter
+            )
+        ),
+        ((CheckpointTxg(4), 4), (CheckpointTxg(5), 5)),
+        "B、C 各一条记录：txg 4 计数器 4、txg 5 计数器 5"
+    );
+    TwoSingleRecordPublishes {
+        image: pool.memory_pool(),
+        first_publish,
+        second_publish,
+    }
+}
+
+/// 改法 R1 的另一半：前缀停在后面那次发布的点名单元上时，停之前整次收下的发布 core 照施加（`crates/singlefs-core/src/recovery.rs`
+/// 的 `replay_journal`：前缀停在验不过的那一条之前），第一次那一版仍是下一次挂载先施加的那一版。B、C 都只在记录里，
+/// B 重写的一片分配记录叶里两条记录罩住同一个槽（补进 B 那条记录、C 那条按它重链），C 那条记录第一个点名单元在第二份落点那块盘上翻一个字节（不重封）：
+/// 重放施加 B、停在 C 之前。I-4.1 报不适用（起点含介质损坏，那一行自己的射程）；I-5.4 照认 B，只红 I-5.4、点名下一次挂载先施加的那一版
+/// （实例 1、txg 4）。core 的只读恢复施加 B 一条、验不过 C 一条，落在 (1, 4)。
+#[test]
+fn a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first(
+) {
+    let built = image_with_two_single_record_publishes(
+        "known-bad-overlapping-records-of-the-next-mount-version-before-a-named-unit-that-does-not-verify",
+    );
+    let mut image = built.image.clone();
+    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(
+        &mut image,
+        &built.first_publish,
+        2,
+    );
+    rewrite_journal_record_and_rechain_the_records_after_it(
+        &mut image,
+        built.first_publish.record.counter,
+        |_record_kept_as_is| {},
+    );
+    for txg in [4u64, 5] {
+        tear_the_root_slot_written_for_txg(&mut image, txg);
+    }
+    let offset_of_the_second_record = record_offset(
+        built.second_publish.record.counter,
+        JOURNAL_RING_DEFAULT_BYTES,
+    )
+    .0;
+    let second_record = read(&image, 0, offset_of_the_second_record, 4096);
+    assert!(
+        get_u32(&second_record, JOURNAL_RECORD_NAMED_COUNT_OFFSET) > 0,
+        "C 那条记录有点名项"
+    );
+    let second_location = JOURNAL_RECORD_HEADER_BYTES + 14;
+    let named_device = get_u32(&second_record, second_location);
+    let mut named_slot_bytes = [0u8; 8];
+    named_slot_bytes[..6]
+        .copy_from_slice(&second_record[second_location + 4..second_location + 10]);
+    let named_slot = u64::from_le_bytes(named_slot_bytes);
+    let mut unit = read(&image, named_device, named_slot * SLOT, 16384);
+    unit[1000] ^= 0xff;
+    write(&mut image, named_device, named_slot * SLOT, &unit);
+
+    let report = recover(&image, JournalPolicy::Consult);
+    assert_eq!(
+        (
+            report.effective_root,
+            report.journal.prefix_applied,
+            report.journal.verification_failed
+        ),
+        (Some((InstanceGeneration(1), CheckpointTxg(4))), 1, 1),
+        "core 施加 B 那一条、验不过 C 那一条，落在 (1, 4)"
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-5.4"],
+        "重放停在 C 之前，B 照认：它的两条记录罩住同一个槽只红 I-5.4：{verdicts:?}"
+    );
+    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
+        panic!("上面判过 I-5.4 红");
+    };
+    assert!(
+        detail.starts_with("下一次挂载先施加的那一版（实例 1、txg 4）那棵分配记录树"),
+        "红在下一次挂载先施加的那一版 B 的分配记录树上：{detail}"
+    );
+    match verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE) {
+        InvariantVerdict::NotApplicable(reason) => assert!(
+            reason.contains("介质损坏"),
+            "I-4.1 报不适用的理由要说是起点含介质损坏：{reason}"
+        ),
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("前缀停在 C 的点名单元上时 I-4.1 该报不适用，实际 {other:?}")
+        }
+    }
+}
+
 /// 计数器为 `counter` 的那条 journal 记录（两块盘各一份）新根段里的树表指针整条写零，经 `JournalRecord::to_bytes` 重封。
 /// 不重链后面的记录：这里改的两条记录（由记录施加出来的 (1, 5)、下一次挂载先施加的 B）在环里都是本实例最后一条，
 /// 下一条（若有）是别的实例的第一条，它的反向链不接在这一条上（`common::rewrite_journal_record_and_rechain_the_records_after_it`
````

### mutations-append.tsv

````diff
第36批 36-1 R1：下一次挂载先施加的那一版改回只看带末条标志的那一条记录（断号：B 的首条没落、后两条落了，core 不施加 B，checker 照认、I-5.4 与 I-3.10 红在 B 上）	crates/singlefs-checker/src/walk.rs	    let last_record_of_the_first_publish_applied = match replay_the_next_mount_would_do(\n        reader,\n        &records_the_replay_reads,\n        newest_root,\n        in_flight_record_limit_of_the_journal_ring(journal_ring_bytes),\n    ) {\n        ReplayTheNextMountWouldDo::AnchorUndecided\n        | ReplayTheNextMountWouldDo::ChosenRootTxgHasNoSuccessor\n        | ReplayTheNextMountWouldDo::NothingApplied => None,\n        ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {\n            last_record_of_the_first_publish_applied_before_it,\n        } => last_record_of_the_first_publish_applied_before_it,\n        ReplayTheNextMountWouldDo::Applied {\n            last_record_of_the_first_publish,\n            publishes: _,\n            records: _,\n            last_record_counter: _,\n            last_record: _,\n        } => Some(last_record_of_the_first_publish),\n    }?;	    let _ = (reader, journal_ring_bytes);\n    let last_record_of_the_first_publish_applied = records_the_replay_reads\n        .iter()\n        .find(|((_, counter), record)| {\n            record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0\n                && record.commit_marker == CommitMarker::Present\n                && record.instance == newest_root.instance\n                && newest_root.checkpoint_txg.checked_add(1) == Some(record.checkpoint_txg)\n                && record_back_chain_lets_it_into_the_replay_prefix(\n                    &records_the_replay_reads,\n                    *counter,\n                    record,\n                )\n        })\n        .map(|(_, record)| *record)?;	-p singlefs-harness --test checker_known_bad_images -- a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first	a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first
第36批 36-1 R1：重放停在后面一次发布的点名单元上时不认停之前已整次收下的第一次发布（core 施加 B、停在 C 之前，checker 不认 B、I-5.4 不红）	crates/singlefs-checker/src/walk.rs	            return ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {\n                last_record_of_the_first_publish_applied_before_it:\n                    last_record_of_the_first_publish_applied,\n            };	            return ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {\n                last_record_of_the_first_publish_applied_before_it: None,\n            };	-p singlefs-harness --test checker_known_bad_images -- a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first	a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first
````

### mutations-replacements.tsv

````diff
实审 B2（A2a 第 25 条对齐，前缀口径）：认下一次挂载先施加的那一版时反向链对不上的记录也算进了前缀	crates/singlefs-checker/src/walk.rs	            || !record_back_chain_lets_it_into_the_replay_prefix(\n                records_the_replay_reads,\n                counter,\n                record,\n            )	            || false	-p singlefs-harness --test checker_known_bad_images -- a_record_whose_back_chain_does_not_match_is_not_the_version_the_next_mount_applies_first	a_record_whose_back_chain_does_not_match_is_not_the_version_the_next_mount_applies_first
第35批 35-1 FL：下一次挂载先施加的那一版照旧取先到的那条记录（s3：三条记录的发布第一条写成上一版的新根段，两条分配记录罩住同一个槽照旧 0 违例）	crates/singlefs-checker/src/walk.rs	            last_record_of_the_first_publish_applied.get_or_insert(record);	            last_record_of_the_first_publish_applied.get_or_insert(open_publish[0]);	-p singlefs-harness --test checker_known_bad_images -- a_stale_first_record_of_the_publish_the_next_mount_applies_first_does_not_hide_two_allocation_records_covering_one_slot	a_stale_first_record_of_the_publish_the_next_mount_applies_first_does_not_hide_two_allocation_records_covering_one_slot
````

## m3-formal-36-2

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index 75b3445..c724068 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -3525,10 +3525,7 @@ fn checker_scan_journal_as_the_recovery_reads(fsid_low: vec2<u32>) {
 }
 
 // ───────────────────────── I-4.1：checker 自己复刻的只读重放（walk.rs 的 replay_the_next_mount_would_do）─────────────────────────
-
-fn replay_record_at(index: u32) -> CheckerRecord {
-    return checker_record_of_row(C_REPLAY_RECORDS + index * C_REPLAY_RECORD_WORDS);
-}
+// 扫描内核认下一次挂载先施加的那一版（I-3.10、I-5.4）读同一份重放的结局：第一次施加的发布的末条经交接段交过去，不另造一份重放。
 
 // 重放前缀读的那一份环（records_the_replay_reads）：每块盘上自证过、恢复读者读得出的记录（提交标记 0 / 1、记录标志位 0 之外全 0、
 // 本次发布内序号不是 0），按 (实例代号, 计数器) 建键、按盘的次序先到先得，升序排在 C_REPLAY_RECORDS。
@@ -3646,14 +3643,25 @@ fn named_units_of_the_record_verify_on_both_copies(r: CheckerRecord) -> bool {
     return true;
 }
 
-// 照 D23（journal 的角色与格式） 已定项 14 的全部规则复刻下一次挂载在所选根（chosen）之上施加哪一段记录，交回最后一次施加的发布的末条
-// 在重放读的那一份环里的下标；施加出来的那一版 = 所选根换上它的新根段。一次发布都不施加、锚点认不定（所选根那次发布带末条标志的多于一条）、
-// 所选根 txg 是 u64 的最大值、前缀停在点名单元两份没都验过的记录上，交回 NONE（I-4.1 报不适用）。
+// 下一次挂载在所选根之上施加的那一段记录的结局（walk.rs 的 ReplayTheNextMountWouldDo），两个都是重放读的那一份环里的下标：
+// last_record_of_the_last_publish_applied —— 最后一次施加的发布的末条，施加出来的那一版 = 所选根换上它的新根段（I-4.1 走它）；
+//   一次发布都不施加、锚点认不定（所选根那次发布带末条标志的多于一条）、所选根 txg 是 u64 的最大值、前缀停在点名单元两份没都验过的记录上，
+//   是 NONE（I-4.1 报不适用）。
+// last_record_of_the_first_publish_applied —— 第一次施加的发布的末条，下一次挂载先施加的那一版照它的新根段（walk.rs 的
+//   version_the_next_mount_applies_first，改法 R1）。前缀停在后面某次发布的点名单元上时，停之前已整次收下的发布 core 照施加
+//   （crates/singlefs-core/src/recovery.rs 的 replay_journal），这一个照交回；一次都没收下、锚点认不定、所选根 txg 没有后继时是 NONE。
+struct ReplayTheNextMountWouldDo {
+    last_record_of_the_last_publish_applied: u32,
+    last_record_of_the_first_publish_applied: u32,
+};
+
+// 照 D23（journal 的角色与格式） 已定项 14 的全部规则复刻下一次挂载在所选根（chosen）之上施加哪一段记录。
 // 在飞上限按环长与格式常量 F（3）现算：⌊环长 ÷ 4096 ÷ 3⌋。
-fn replayed_version_the_next_mount_would_apply(chosen: CheckerRoot, ring_bytes: vec2<u32>, fsid_low: vec2<u32>) -> u32 {
+fn replay_the_next_mount_would_do(chosen: CheckerRoot, ring_bytes: vec2<u32>, fsid_low: vec2<u32>) -> ReplayTheNextMountWouldDo {
+    let nothing_applied = ReplayTheNextMountWouldDo(NONE, NONE);
     let count = collect_the_records_the_replay_reads(fsid_low);
     if (undecidable != 0u) {
-        return NONE;
+        return nothing_applied;
     }
     var anchors = 0u;
     var anchor_counter = vec2<u32>(0u, 0u);
@@ -3665,12 +3673,12 @@ fn replayed_version_the_next_mount_would_apply(chosen: CheckerRoot, ring_bytes:
         }
     }
     if (anchors >= 2u) {
-        return NONE;
+        return nothing_applied;
     }
     var have_expected_next_counter = anchors == 1u;
     var expected_next_counter = add64(anchor_counter, u64_of(1u));
     if (chosen.txg.x == NONE && chosen.txg.y == NONE) {
-        return NONE;
+        return nothing_applied;
     }
     let chain_start_txg_without_anchor = add64(chosen.txg, u64_of(1u));
     let in_flight_record_limit = divmod64(shr64(ring_bytes, 12u), u64_of(3u)).quotient;
@@ -3679,6 +3687,7 @@ fn replayed_version_the_next_mount_would_apply(chosen: CheckerRoot, ring_bytes:
     // 开着的那次发布的上一条（open_publish_records 为 0 时不读它）
     var previous: CheckerRecord;
     var last_applied = NONE;
+    var last_record_of_the_first_publish_applied = NONE;
     for (var i = 0u; i < count; i = i + 1u) {
         let r = replay_record_at(i);
         if (r.instance != chosen.instance || !lt64(chosen.txg, r.txg)) {
@@ -3720,18 +3729,22 @@ fn replayed_version_the_next_mount_would_apply(chosen: CheckerRoot, ring_bytes:
             break;
         }
         if (!named_units_of_the_record_verify_on_both_copies(r)) {
-            return NONE;
+            return ReplayTheNextMountWouldDo(NONE, last_record_of_the_first_publish_applied);
         }
         expected_next_counter = add64(r.counter, u64_of(1u));
         have_expected_next_counter = true;
         open_publish_records = open_publish_records + 1u;
         previous = r;
         if (ends_its_publish) {
+            // 第一次发布的那一版同取它的末条（已定项 15：core 施加末条那一份新根段，改法 FL 的那一取法）
+            if (last_record_of_the_first_publish_applied == NONE) {
+                last_record_of_the_first_publish_applied = i;
+            }
             open_publish_records = 0u;
             last_applied = i;
         }
     }
-    return last_applied;
+    return ReplayTheNextMountWouldDo(last_applied, last_record_of_the_first_publish_applied);
 }
 
 // 走读的七个阶段，按次序；每个阶段里要走的每一处都经 walk_and_hand_off 循环里那一个调用点走
@@ -3750,7 +3763,8 @@ const WALK_REACHED_THE_HAND_OFF: u32 = 4u;
 // 走读那一批连同收尾：择最新根、候选根、由记录施加出来的版本、被抛弃根，判走读之后那几条；
 // 收尾是候选集之外的根整遍走一次收位置项校验和（第三批判隔离的记录要），F 之下、没被抛弃的根各走一遍，
 // 它们引用的、候选版本没走到的槽记进交接段（第三批算 defer 那一格要），再把计数与几何写进交接段；
-// 最后照下一次挂载的重放施加出那一版、另起一份走读走一遍（I-4.1），它只动上下文 1 的集合，不碰交接段与扫描内核读的表。
+// 最后照下一次挂载的重放施加出那一版、另起一份走读走一遍（I-4.1），它只动上下文 1 的集合；同一份重放第一次施加的发布的末条写进交接段
+// （H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED），扫描内核照它读重放读的那一份环里那一行（I-3.10、I-5.4）。
 // 写成按阶段推进的一个循环：每一圈先找这一阶段下一处要走的（找不到就做完这一阶段收尾的事、转下一阶段），走，再做走完之后的事。
 fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
     let device_count = hdr(3u);
@@ -4129,7 +4143,10 @@ fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
             }
         } else if (phase == WALK_PHASE_REPLAYED_VERSION) {
             // I-4.1：一条都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验不过，都报不适用（不走）
-            let last_applied = replayed_version_the_next_mount_would_apply(newest_root, g.ring_bytes, fsid_low);
+            let replay = replay_the_next_mount_would_do(newest_root, g.ring_bytes, fsid_low);
+            // 下一次挂载先施加的那一版（I-3.10、I-5.4）交给扫描内核：H_VALID 只在走过这一阶段之后才置 1
+            sws(H_BASE + H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED, replay.last_record_of_the_first_publish_applied);
+            let last_applied = replay.last_record_of_the_last_publish_applied;
             if (last_applied == NONE || undecidable != 0u) {
                 phase = WALK_PHASE_DONE;
             } else {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
index 49f9428..550f7e0 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -726,22 +726,30 @@ const H_SLOTS_PER_REGION: u32 = 29u;     // 根环每区槽数 S：I-7.9 的一
 // 1 = 主走读沿最新根的实例表链走到了末片（走读内核的 w_newest_instance_table_chain_reached_its_last_page）：扫描内核 I-9.29 跨版本那张表比不比读它。
 // 第 30、31 两个字原来空着（H_BELOW_FLOOR_SLOTS 从 H_BASE + 32 起），占第 30 个，交接段不变长
 const H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE: u32 = 30u;
+// 走读内核照 I-4.1 那一份重放（replay_the_next_mount_would_do）施加的第一次发布的末条，在重放读的那一份环（C_REPLAY_RECORDS）里的下标，
+// 没有是 NONE：扫描内核认下一次挂载先施加的那一版（I-3.10、I-5.4）读它。占原来空着的第 31 个字，交接段不变长
+const H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED: u32 = 31u;
 const H_MAX_BELOW_FLOOR_SLOTS: u32 = 4096u;
 const H_BELOW_FLOOR_SLOTS: u32 = 235504u;   // 盘、槽 lo/hi
 const H_END: u32 = 247792u;
 
-// ───────────────────────── 交接段之后：走读内核自己用、扫描内核不读的两张表 ─────────────────────────
+// ───────────────────────── 交接段之后：走读内核写的两张表（扫描内核只读重放那一张）─────────────────────────
 // 主走读（上下文 0）每个走过的单元一行：有没有记下诞生代号、诞生代号 lo/hi（walk.rs 的 birth_txg_of_walked_units，与 C_VISITED0 同下标）。
 // 别的版本再引用它时按这张表判 I-4.9（非最新候选根引用的单元不晚于它）。
 const C_VISITED_BIRTH_WORDS: u32 = 3u;
 
 const C_VISITED_BIRTHS0: u32 = 247792u;
 
-// I-4.1 重放读的那一份环（walk.rs 的 records_the_replay_reads）：版本、计数器 lo/hi、实例，按 (实例, 计数器) 升序
+// I-4.1 重放读的那一份环（walk.rs 的 records_the_replay_reads）：版本、计数器 lo/hi、实例，按 (实例, 计数器) 升序。
+// 走读内核在 I-4.1 那一阶段填；扫描内核照交接段的 H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED 读其中一行、不写
 const C_REPLAY_RECORD_WORDS: u32 = 4u;
 
 const C_REPLAY_RECORDS: u32 = 253936u;
 
+fn replay_record_at(index: u32) -> CheckerRecord {
+    return checker_record_of_row(C_REPLAY_RECORDS + index * C_REPLAY_RECORD_WORDS);
+}
+
 // I-3.5（引用区间的精确性）的出现规则读的四张表，只记主走读（上下文 0）走的那几版（最新根、回退候选根、由记录施加出来的版本）。
 // ① 落点表（walk.rs 的 unit_of_each_placement 与 user_visible_tree_of_each_placement）：跟着指针记下的每个落点一行——
 //    落点盘、槽 lo/hi，它是哪个单元（第一次记下它的那条指针第 0 条位置条目的盘、槽 lo/hi），记下它那一刻走读在不在一棵用户可见树里
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index ed398ae..2b51357 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -966,9 +966,9 @@ fn judge_allocation_records_disjoint() {
             JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER,
         );
     }
-    let next_version = record_index_of_the_version_the_next_mount_applies_first();
+    let next_version = record_row_of_the_version_the_next_mount_applies_first();
     if (next_version != NONE) {
-        let r = checker_record_at(next_version);
+        let r = checker_record_of_row(next_version);
         judge_allocation_records_disjoint_of_a_version(
             r.base,
             JOURNAL_NEW_ROOT_SEGMENT_OFFSET,
@@ -1074,34 +1074,24 @@ fn judge_allocation_generations_against_unit_births() {
         let r = checker_record_at(sw(R_DATA + v));
         judge_allocation_generations_under_tree_table(parse_node_pointer(r.base, 99u));
     }
-    let next_version = record_index_of_the_version_the_next_mount_applies_first();
+    let next_version = record_row_of_the_version_the_next_mount_applies_first();
     if (next_version != NONE) {
-        judge_allocation_generations_under_tree_table(parse_node_pointer(checker_record_at(next_version).base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET));
+        judge_allocation_generations_under_tree_table(parse_node_pointer(checker_record_of_row(next_version).base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET));
     }
 }
 
-// 下一次挂载会先施加的那一版（walk.rs version_the_next_mount_applies_first）：与最新根同实例、txg = 最新根 txg + 1、带末条标志（改法 FL：
-// core 走到末条才整次施加、施加末条那一份新根段）、带提交标记、进得了前缀、根槽读不出的记录里计数器最小的那一条的下标；没有交回 NONE。
-// 记录数组按计数器升序排（checker_insert_record），同一实例里数组次序就是 CPU 按 (实例, 计数器) 的键序，取第一条即取计数器最小的那一条
-fn record_index_of_the_version_the_next_mount_applies_first() -> u32 {
-    let newest_root = checker_root_of(c_newest_index);
-    if (add64_overflows(newest_root.txg, u64_of(1u))) {
+// 下一次挂载会先施加的那一版（walk.rs version_the_next_mount_applies_first，改法 R1：判决 m3-formal-code-b35-r1 第三节第 1 条）：
+// 走读内核照 I-4.1 那一份重放（replay_the_next_mount_would_do）施加的第一次发布的末条（core 施加末条那一份新根段），经交接段交来；
+// 交回它在草稿区里的那一行（重放读的那一份环里，交给 checker_record_of_row 解），没有交回 NONE。core 的施加单位是一次发布：
+// 断号、反向链对不上、序号或 txg 接不上、末条之后同一次发布还有记录、点名单元验不过的那次发布都不施加，这里同一份重放、同样不认；
+// 一次都不施加、锚点认不定、所选根 txg 没有后继时没有这一版；
+// 前缀停在后面某次发布的点名单元上时，停之前整次收下的第一次发布照认。
+fn record_row_of_the_version_the_next_mount_applies_first() -> u32 {
+    let replay_index = sw(H_BASE + H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED);
+    if (replay_index == NONE) {
         return NONE;
     }
-    let next_txg = add64(newest_root.txg, u64_of(1u));
-    for (var i = 0u; i < root_count; i = i + 1u) {
-        let root = checker_root_of(i);
-        if (root.instance == newest_root.instance && eq64(root.txg, next_txg)) {
-            return NONE;
-        }
-    }
-    for (var i = 0u; i < record_count; i = i + 1u) {
-        let r = checker_record_at(i);
-        if ((r.flags & 1u) != 0u && r.commit_byte == 1u && r.instance == newest_root.instance && eq64(r.txg, next_txg) && record_can_enter_a_replay_prefix(i)) {
-            return i;
-        }
-    }
-    return NONE;
+    return C_REPLAY_RECORDS + replay_index * C_REPLAY_RECORD_WORDS;
 }
 
 // instance_table_rows_of_root_without_judging：一条根自己的实例表沿链读出全部行，读不出交回 false
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index ff383a4..96a184b 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -36,6 +36,8 @@
 //! （O-1）一条钉 GPU 记判不了（实例表链过长）、回落 CPU 之后结论相同；走读内核与扫描内核的链片数上限相同一条（不用卡）。
 //! 第 35 批：由记录造出来的两类版本取带末条标志那条记录的新根段（三条记录的发布第一条写成上一版的根：下一次挂载那一版叠两条记录罩住同一个槽、
 //! 挂载之后由记录施加出来的那一版各一条），树表指针整条全零照 core 跟过去（根与它的记录、由记录施加出来的那一版的记录各一条），都整份比。
+//! 第 36 批：下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认（改法 R1）——三条记录的发布首条没落（断号）、
+//! 前缀停在后面那次发布的点名单元上（第一次那一版照认）各一条流，CPU 一侧先在整条落盘的镜像上钉形，再整份比。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
 //! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
@@ -8441,8 +8443,9 @@ fn write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publ
     }
 }
 
-/// 一片分配记录叶由顺序写那次发布（txg 4）写出、且里面有一对改得动的记录（[`widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts`]）。
-fn is_an_allocation_leaf_of_the_three_unit_publish_that_can_be_widened(bytes: &[u8]) -> bool {
+/// 一片分配记录叶由 txg 4 那次发布写出（三个数据单元那条流上是顺序写那次，两次单条覆盖写那条流上是第一次覆盖写）、
+/// 且里面有一对改得动的记录（[`widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts`]）。
+fn is_an_allocation_leaf_born_at_txg_four_that_can_be_widened(bytes: &[u8]) -> bool {
     if unit_class(bytes) != 2 || index_node_key_width(bytes) != 10 || bytes[50] != 0 {
         return false;
     }
@@ -8459,8 +8462,9 @@ fn is_an_allocation_leaf_of_the_three_unit_publish_that_can_be_widened(bytes: &[
 
 /// 改法 FL 下一次挂载那一支（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b，攻方报告的 s3 形）：
 /// 三个数据单元那条流上，顺序写那次发布（txg 4，三条记录）写的分配记录叶里两条记录罩住同一个槽（重封到根与三条记录），
-/// 再把第一条记录的新根段写成 txg 3 那条根的。三条记录都落了、根槽没落的状态上，CPU 认下一次挂载先施加的那一版取末条那一条，红 I-5.4；
-/// GPU 扫描内核 `record_index_of_the_version_the_next_mount_applies_first` 同取末条、红在同样的状态上（取第一条就走 txg 3 的树、不红）。
+/// 再把第一条记录的新根段写成 txg 3 那条根的。三条记录都落了、根槽没落的状态上，CPU 认下一次挂载先施加的那一版取重放第一次施加的发布的末条，
+/// 红 I-5.4；GPU 扫描内核 `record_row_of_the_version_the_next_mount_applies_first` 读走读内核同一份重放交来的那一条、红在同样的状态上
+/// （取第一条就走 txg 3 的树、不红）。
 fn record_three_unit_file_with_a_stale_first_record_over_two_allocation_records_covering_one_slot(
     tag: &str,
 ) -> RecordedFlow {
@@ -8468,7 +8472,7 @@ fn record_three_unit_file_with_a_stale_first_record_over_two_allocation_records_
     let mut flow = record_three_unit_file_with_a_resealed_field(
         tag,
         name,
-        &is_an_allocation_leaf_of_the_three_unit_publish_that_can_be_widened,
+        &is_an_allocation_leaf_born_at_txg_four_that_can_be_widened,
         &|bytes| {
             assert_eq!(
                 widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
@@ -8494,16 +8498,11 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_first_record_of_the_next_mount_ver
 }
 
 /// 三个数据单元的第一个文件那段历史（`pool_with_a_three_unit_file`：新池新建文件 txg 3、顺序写三个数据单元那次发布 txg 4、三条记录）
-/// 录到 txg 4 的根槽写之前停——崩在那次根槽 FUA 之前，三条记录两块盘都落了；`rewrite_the_records_before_the_mount` 在这一段写上改那次发布的记录。
-/// 再从这一段写全落的那张崩溃镜像起新进程（另一条录制流）录可写挂载（施加那三条记录、取号 2、写行、暖机两次）与实例 2 的一次覆盖写，
-/// 两段写接成一条流。txg 4 那一版只在它的三条记录里：挂载之前的状态上它是下一次挂载先施加的那一版，挂载写了实例 1 的行之后它是由记录施加出来、
-/// 根槽从没落盘的那一版（攻方报告 P1-b 的 s4 形）。挂载读到的就是这条流里那三条记录（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md`
-/// 第二节 P3 与第三节第 1 条 R2：此前录完整段历史再从流里拿掉 txg 4 的根槽写，录制时挂载读到的是 txg 4 的根、什么都没施加，
-/// 写出的实例表行（W）与之后的发布、根槽之后的系统配置轮换都与合法历史不同）。
-fn record_three_unit_file_applied_only_by_its_records_after_a_mount(
+/// 录到 txg 4 的根槽写之前停——崩在那次根槽 FUA 之前，三条记录两块盘都落了。被判的根是 (1, 3) 那一次根槽写；版本是 txg 3、txg 4 两版
+/// （txg 4 那一版只在三条记录里，记录全落的状态上恢复由记录施加它）。
+fn record_three_unit_file_up_to_the_root_of_its_three_record_publish(
     tag: &str,
     name: &'static str,
-    rewrite_the_records_before_the_mount: &dyn Fn(&mut RecordedFlow),
 ) -> RecordedFlow {
     let pool = pool_with_a_three_unit_file(tag);
     let operations = pool.retained_operations();
@@ -8524,7 +8523,7 @@ fn record_three_unit_file_applied_only_by_its_records_after_a_mount(
             &operations_after_mkfs[..root_of_the_three_unit_publish],
             &classifier,
         );
-    let mut flow = RecordedFlow {
+    RecordedFlow {
         name,
         base: pool.memory_pool_after_mkfs(),
         judged_root_index: writes
@@ -8533,10 +8532,36 @@ fn record_three_unit_file_applied_only_by_its_records_after_a_mount(
             .expect("顺序写之前有新建文件那次发布 (1, 3) 的根槽写"),
         writes,
         segments,
-        versions: Vec::new(),
-    };
+        versions: vec![
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(3),
+                content: file_content(),
+            },
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(4),
+                content: content_of_three_data_units(),
+            },
+        ],
+    }
+}
+
+/// [`record_three_unit_file_up_to_the_root_of_its_three_record_publish`] 那一段上，`rewrite_the_records_before_the_mount` 改那次发布的记录。
+/// 再从这一段写全落的那张崩溃镜像起新进程（另一条录制流）录可写挂载（施加那三条记录、取号 2、写行、暖机两次）与实例 2 的一次覆盖写，
+/// 两段写接成一条流。txg 4 那一版只在它的三条记录里：挂载之前的状态上它是下一次挂载先施加的那一版，挂载写了实例 1 的行之后它是由记录施加出来、
+/// 根槽从没落盘的那一版（攻方报告 P1-b 的 s4 形）。挂载读到的就是这条流里那三条记录（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md`
+/// 第二节 P3 与第三节第 1 条 R2：此前录完整段历史再从流里拿掉 txg 4 的根槽写，录制时挂载读到的是 txg 4 的根、什么都没施加，
+/// 写出的实例表行（W）与之后的发布、根槽之后的系统配置轮换都与合法历史不同）。
+fn record_three_unit_file_applied_only_by_its_records_after_a_mount(
+    tag: &str,
+    name: &'static str,
+    rewrite_the_records_before_the_mount: &dyn Fn(&mut RecordedFlow),
+) -> RecordedFlow {
+    let mut flow = record_three_unit_file_up_to_the_root_of_its_three_record_publish(tag, name);
     rewrite_the_records_before_the_mount(&mut flow);
 
+    let classifier = geometry();
     let stream_of_the_new_process = SharedStream::retaining_contents();
     let every_write_landed = vec![true; flow.writes.len()];
     let mut devices = common::crash_state_devices(
@@ -8776,3 +8801,301 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_
         mismatch_listing(&comparison)
     );
 }
+
+// ─── 第 36 批：下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认（改法 R1 的 GPU 一半） ───
+
+/// I-5.4 在下一次挂载先施加的那一版上的违例说明的开头（`walk.rs` 按版本那一层的称呼）。
+const VIOLATION_ON_THE_VERSION_THE_NEXT_MOUNT_APPLIES_FIRST_AT_TXG_FOUR: &str =
+    "下一次挂载先施加的那一版（实例 1、txg 4）";
+
+/// 改法 R1 断号那一形（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1 与第三节第 1 条，攻方
+/// `cut=records-2..2-of-2`；harness 档 `checker_known_bad_images.rs` 的
+/// `a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first` 同形）：三个数据单元那段历史录到顺序写那次发布
+/// （txg 4，计数器 4、5、6 三条记录）的根槽写之前停（崩在那次根槽 FUA 之前，R2 的合法造法），那次发布写的分配记录叶里两条记录罩住同一个槽，
+/// 重封进三条记录、后面的记录重链。三条记录两块盘六次写在同一段里、每个子集都展开：首条两份都没落而后两条落了的状态在里面。
+fn record_three_unit_file_crashed_before_its_root_over_two_allocation_records_covering_one_slot(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "three-unit-file-crashed-before-its-root-over-overlapping-allocation-records";
+    let mut flow = record_three_unit_file_up_to_the_root_of_its_three_record_publish(tag, name);
+    let targets = last_unit_copies(
+        &flow,
+        name,
+        &is_an_allocation_leaf_born_at_txg_four_that_can_be_widened,
+        false,
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
+        assert_eq!(
+            widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
+            1,
+            "挑叶时这一片改得动"
+        );
+    });
+    let records = journal_record_writes_of_the_publish(&flow, 4);
+    let (first_record_write, _) = *records.first().expect("顺序写那次发布有记录");
+    let segment_of_the_records = flow
+        .segments
+        .iter()
+        .find(|segment| segment.contains(&first_record_write))
+        .expect("记录写在某一段里");
+    assert!(
+        records.len() == 6
+            && records
+                .iter()
+                .all(|(index, _)| segment_of_the_records.contains(index))
+            && segment_of_the_records.len() <= DEFAULT_EXPAND_UP_TO,
+        "流 {name}：txg 4 的三条记录两块盘六次写在同一段里、那一段不多于 {DEFAULT_EXPAND_UP_TO} 次写（每个子集都展开）"
+    );
+    flow
+}
+
+/// 断号那一形（[`record_three_unit_file_crashed_before_its_root_over_two_allocation_records_covering_one_slot`]）。CPU 一侧先钉流造对了：
+/// 三条记录都落了的镜像上 txg 4 是下一次挂载先施加的那一版，I-5.4 红在它身上；首条（计数器 4）两份都不落、其余照落的镜像上 core 断号
+/// 不施加 txg 4，CPU 照同一份重放不认它，I-5.4 在 txg 3 那一版上判过而成立。GPU 扫描内核读走读内核那一份重放交来的第一次发布，
+/// 每个状态逐位相同（只看带末条标志的那一条就认计数器 6，断号的状态上 GPU 红 I-5.4 而 CPU 不红）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first(
+) {
+    let flow =
+        record_three_unit_file_crashed_before_its_root_over_two_allocation_records_covering_one_slot(
+            "crash-judge-gpu-r1-first-record-not-landed",
+        );
+    let mut image_with_every_record_landed = flow.base.clone();
+    image_with_every_record_landed.apply_writes(&flow.writes);
+    match pool_checker_verdict_on_the_image(&image_with_every_record_landed, "I-5.4") {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.starts_with(VIOLATION_ON_THE_VERSION_THE_NEXT_MOUNT_APPLIES_FIRST_AT_TXG_FOUR),
+            "三条记录都落了：I-5.4 红在下一次挂载先施加的那一版 txg 4 上：{detail}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => panic!(
+            "三条记录都落了：txg 4 是下一次挂载先施加的那一版，I-5.4 要红，实际 {other_verdict:?}"
+        ),
+    }
+    let records = journal_record_writes_of_the_publish(&flow, 4);
+    let first_counter = records
+        .iter()
+        .map(|(_, counter)| *counter)
+        .min()
+        .expect("顺序写那次发布有记录");
+    let copies_of_the_first_record: Vec<usize> = records
+        .iter()
+        .filter(|(_, counter)| *counter == first_counter)
+        .map(|(index, _)| *index)
+        .collect();
+    assert_eq!(copies_of_the_first_record.len(), 2, "首条两块盘各一份");
+    let writes_without_the_first_record: Vec<RetainedWrite> = flow
+        .writes
+        .iter()
+        .enumerate()
+        .filter(|(index, _)| !copies_of_the_first_record.contains(index))
+        .map(|(_, write)| write.clone())
+        .collect();
+    let mut image_with_the_first_record_not_landed = flow.base.clone();
+    image_with_the_first_record_not_landed.apply_writes(&writes_without_the_first_record);
+    assert_eq!(
+        pool_checker_verdict_on_the_image(&image_with_the_first_record_not_landed, "I-5.4"),
+        InvariantVerdict::Holds,
+        "首条两份都没落、后两条落了：core 断号不施加 txg 4，CPU 不认它，I-5.4 在 txg 3 那一版上判过而成立"
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-5.4"),
+        "流 {} 上 CPU 池级 checker 要红 I-5.4（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
+
+/// 流里 (实例 1, `checkpoint_txg`) 那条根的最后一次根槽写，抄一份、根记录树表指针那一段里翻一个字节（不重封：那个根槽读不出）。
+fn torn_copy_of_the_root_write_for_txg(flow: &RecordedFlow, checkpoint_txg: u64) -> RetainedWrite {
+    let mut torn = flow
+        .writes
+        .iter()
+        .rev()
+        .find(|write| {
+            write.kind == StepKind::RootRecordFua
+                && write.bytes().is_some_and(|bytes| {
+                    u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
+                        && u64::from_le_bytes(
+                            bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET
+                                ..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
+                                .try_into()
+                                .expect("8"),
+                        ) == checkpoint_txg
+                })
+        })
+        .unwrap_or_else(|| panic!("流 {} 里有 (1, {checkpoint_txg}) 的根槽写", flow.name))
+        .clone();
+    let WrittenContents::Bytes(bytes) = &mut torn.contents else {
+        panic!("根槽写是普通写");
+    };
+    let flipped = usize::try_from(common::BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER)
+        .expect("根记录里的下标");
+    bytes[flipped] ^= 0xff;
+    torn
+}
+
+/// 改法 R1 的另一半（harness 档
+/// `a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first` 同形）：
+/// 新池新建文件（A，txg 3）之后同一个进程、实例 1 里再覆盖写两次：B (1, 4)、C (1, 5)，各一条记录（计数器 4、5）；整段历史照录。
+/// B 写的分配记录叶里两条记录罩住同一个槽，重封到 B 的根与记录、C 的记录重链。流尾另起一段（R2 改记录那一条流的造法：之前的写不动）：
+/// B、C 两次的根槽各翻一个字节（两个根槽都读不出，与 harness 档那一形的 `tear_the_root_slot_written_for_txg` 同一处），
+/// C 那条记录第一个点名单元第二份落点上那一份翻一个字节、不重封（介质损坏）。这一段全落的状态上 core 择 A 的根、施加 B、停在 C 之前。
+fn record_two_single_record_publishes_stopped_by_a_named_unit_of_the_second(
+    tag: &str,
+) -> RecordedFlow {
+    let name =
+        "two-single-record-publishes-only-in-their-records-stopped-by-a-named-unit-of-the-second";
+    let mut pool = build_pool(tag);
+    common::overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &second_content());
+    common::overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &third_content());
+    let mut flow = recorded_flow_of_the_pool(
+        &pool,
+        name,
+        vec![
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(3),
+                content: file_content(),
+            },
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(4),
+                content: second_content(),
+            },
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(5),
+                content: third_content(),
+            },
+        ],
+    );
+    let targets = last_unit_copies(
+        &flow,
+        name,
+        &is_an_allocation_leaf_born_at_txg_four_that_can_be_widened,
+        false,
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
+        assert_eq!(
+            widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
+            1,
+            "挑叶时这一片改得动"
+        );
+    });
+    let (record_of_the_second_publish, _) = *journal_record_writes_of_the_publish(&flow, 5)
+        .first()
+        .expect("C 有记录");
+    let record_bytes = flow.writes[record_of_the_second_publish]
+        .bytes()
+        .expect("记录写带字节");
+    assert!(
+        u32::from_le_bytes(record_bytes[12..16].try_into().expect("4")) > 0,
+        "C 那条记录有点名项"
+    );
+    let second_location = JOURNAL_RECORD_HEADER_BYTES + 14;
+    let named_device = u32::from_le_bytes(
+        record_bytes[second_location..second_location + 4]
+            .try_into()
+            .expect("4"),
+    );
+    let mut named_slot = [0u8; 8];
+    named_slot[..6].copy_from_slice(&record_bytes[second_location + 4..second_location + 10]);
+    let named_offset = u64::from_le_bytes(named_slot) * SLOT_BYTES;
+    let mut flipped_named_unit = flow
+        .writes
+        .iter()
+        .rev()
+        .find(|write| {
+            write.kind == StepKind::UnitWrite
+                && write.device.0 == named_device
+                && write.offset.0 == named_offset
+        })
+        .expect("流里有 C 第一个点名单元第二份那一次单元写")
+        .clone();
+    let WrittenContents::Bytes(unit_bytes) = &mut flipped_named_unit.contents else {
+        panic!("单元写是普通写");
+    };
+    unit_bytes[1000] ^= 0xff;
+    let torn_roots =
+        [4u64, 5].map(|checkpoint_txg| torn_copy_of_the_root_write_for_txg(&flow, checkpoint_txg));
+    append_a_segment_of_writes(
+        &mut flow,
+        torn_roots.into_iter().chain([flipped_named_unit]).collect(),
+    );
+    flow
+}
+
+/// 前缀停在后面那次发布的点名单元上那一形（[`record_two_single_record_publishes_stopped_by_a_named_unit_of_the_second`]）。
+/// CPU 一侧先钉流造对了：整条落盘的镜像上 core 的只读恢复施加 B 一条、验不过 C 一条、落在 (1, 4)；I-5.4 照认 B（停之前已整次收下的第一次发布）、
+/// 红在它身上，I-4.1 报不适用（起点含介质损坏）。GPU 走读内核那一份重放停在 C 时照交回 B 的末条，扫描内核同红，每个状态逐位相同
+/// （停在点名单元上就交 NONE 的话，那个状态上 GPU 不红 I-5.4 而 CPU 红）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first(
+) {
+    let flow = record_two_single_record_publishes_stopped_by_a_named_unit_of_the_second(
+        "crash-judge-gpu-r1-stopped-by-a-named-unit",
+    );
+    let mut image = flow.base.clone();
+    image.apply_writes(&flow.writes);
+    let report =
+        singlefs_core::recovery::recover(&image, singlefs_core::recovery::JournalPolicy::Consult);
+    assert_eq!(
+        (
+            report.effective_root,
+            report.journal.prefix_applied,
+            report.journal.verification_failed
+        ),
+        (Some((InstanceGeneration(1), CheckpointTxg(4))), 1, 1),
+        "整条落盘：core 施加 B 那一条、验不过 C 那一条，落在 (1, 4)"
+    );
+    match pool_checker_verdict_on_the_image(&image, "I-5.4") {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.starts_with(VIOLATION_ON_THE_VERSION_THE_NEXT_MOUNT_APPLIES_FIRST_AT_TXG_FOUR),
+            "重放停在 C 之前、B 照认：I-5.4 红在下一次挂载先施加的那一版 B 上：{detail}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("重放停在 C 之前、B 照认：I-5.4 要红，实际 {other_verdict:?}")
+        }
+    }
+    match pool_checker_verdict_on_the_image(&image, "I-4.1") {
+        InvariantVerdict::NotApplicable(reason) => assert!(
+            reason.contains("介质损坏"),
+            "I-4.1 报不适用的理由要说是起点含介质损坏：{reason}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("前缀停在 C 的点名单元上时 I-4.1 该报不适用，实际 {other_verdict:?}")
+        }
+    }
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
````

### mutations-append.tsv

````diff
第36批 36-2 GPU 判器 R1：扫描内核认下一次挂载先施加的那一版改回只看带末条标志的那一条记录（断号：txg 4 首条两份没落、后两条落了，core 不施加，CPU 不认、不红 I-5.4，GPU 照认计数器 6、红 I-5.4）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    let replay_index = sw(H_BASE + H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED);\n    if (replay_index == NONE) {\n        return NONE;\n    }\n    return C_REPLAY_RECORDS + replay_index * C_REPLAY_RECORD_WORDS;	    let newest_root = checker_root_of(c_newest_index);\n    if (add64_overflows(newest_root.txg, u64_of(1u))) {\n        return NONE;\n    }\n    let next_txg = add64(newest_root.txg, u64_of(1u));\n    for (var i = 0u; i < root_count; i = i + 1u) {\n        let root = checker_root_of(i);\n        if (root.instance == newest_root.instance && eq64(root.txg, next_txg)) {\n            return NONE;\n        }\n    }\n    for (var i = 0u; i < record_count; i = i + 1u) {\n        let r = checker_record_at(i);\n        if ((r.flags & 1u) != 0u && r.commit_byte == 1u && r.instance == newest_root.instance && eq64(r.txg, next_txg) && record_can_enter_a_replay_prefix(i)) {\n            return R_RECORDS + i * RECORD_WORDS;\n        }\n    }\n    return NONE;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first	the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first
第36批 36-2 GPU 判器 R1：走读内核那一份重放停在后面一次发布的点名单元上时不交回停之前已整次收下的第一次发布（B 施加、停在 C 之前，CPU 认 B、红 I-5.4，GPU 不认、不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            return ReplayTheNextMountWouldDo(NONE, last_record_of_the_first_publish_applied);	            return ReplayTheNextMountWouldDo(NONE, NONE);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first	the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first
````

### mutations-replacements.tsv

````diff
第34批 34-1 GPU 判器 RV：I-5.4 不收下一次挂载先施加的那一版（新池新建文件那次发布的分配记录叶里两条记录罩住同一个槽，根槽没落的状态上 CPU 红、GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (next_version != NONE) {\n        let r = checker_record_of_row(next_version);	    if (next_version != NONE && false) {\n        let r = checker_record_of_row(next_version);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first	the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first
第35批 35-1 GPU 判器 FL：下一次挂载先施加的那一版照旧取记录数组里第一条合条件的（三条记录的发布第一条写成上一版的根，根槽没落的状态上 CPU 红 I-5.4、GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                last_record_of_the_first_publish_applied = i;	                last_record_of_the_first_publish_applied = i + 1u - open_publish_records;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_the_first_record_of_the_next_mount_version_carries_the_previous_root_over_two_allocation_records_covering_one_slot	the_gpu_judge_agrees_with_the_cpu_when_the_first_record_of_the_next_mount_version_carries_the_previous_root_over_two_allocation_records_covering_one_slot
````
