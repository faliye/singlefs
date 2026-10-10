# 第 30 批代码轮第一轮：改动全文（2026-10-05）

三份补丁照实现员交回的原样，按合入次序：30-1、30-2（交回前已对删除线 C712 三方重定基）、30-3。合入在删除线第 20 批之后。主 agent 合入后另改两处：`crates/mutations.tsv` 第 28 批 MUT-A 那一行改名（「5 改 4」→「5000 毫秒改 4000」，锚点与替换文不动），`.claude/kb/layout/01-first-txn.md` 第 149 行 T_time 写成「5000（单位毫秒，即 5 秒）」。

## m3-formal-30-1

### crates.patch

````diff
diff --git a/crates/singlefs-harness/tests/model_compares_the_third_and_fourth_inode_leaf_containers.rs b/crates/singlefs-harness/tests/model_compares_the_third_and_fourth_inode_leaf_containers.rs
new file mode 100644
index 0000000..a53f44e
--- /dev/null
+++ b/crates/singlefs-harness/tests/model_compares_the_third_and_fourth_inode_leaf_containers.rs
@@ -0,0 +1,267 @@
+//! O1（理想模型对拍，D13（验证路线） 已定项 6）罩 inode 树第三片、第四片叶容器：删完第一个文件之后一次建一个 inode，
+//! 建到第四片；回退到第三片在之后被重写过的那一版；回退到还没有第四片的那一版。模型逐片记各片叶容器的分配代、
+//! 逐次与实现交回的分配记录比，每一步都对得上。
+//! 只建到第三片的两条（`model_compares_inode_creations_into_the_third_inode_leaf_container`、
+//! `model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers`）分不开「第 k 片」与「第三片及以后都当第三片」，
+//! 也分不开「回退取回、丢掉右侧每一片」与「只取回第二片」「只丢第三片」（第 29 批代码轮判决
+//! `research/prompts/m3-formal-code-b29-r1-main-verification.md` L1 第三行）。
+//! 每一步的池级 checker 不跑（`PerStepChecker::Skipped`）：这几条判的是 O1 逐片记的账。
+
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_format::INODE_LEAF_RECORDS;
+use singlefs_harness::history::{
+    execute_history_with, AppliedEffect, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
+    HistoryExecution, HistoryOperation, HistoryOperationKind, HistoryRun, HistorySeed,
+    HistoryStartingPoint, PerStepChecker, RollbackTargetChoice, StepOutcome,
+};
+use singlefs_harness::SharedStream;
+
+/// 建到第四片：建满三片（3 × 233 条），再往第四片里建 4 条——第 700 条分裂出第四片，之后三条只重写第四片。
+const INODE_CREATIONS_INTO_THE_FOURTH_LEAF_CONTAINER: u64 = 3 * INODE_LEAF_RECORDS + 4;
+
+/// 第三片在目标那一版之后被重写过的那一次回退之前建几个 inode：建满两片，再往第三片里建 5 条——
+/// 根环里最新那条根的第三片 5 条、第 3 新的那条 3 条。
+const INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_REWRITTEN_THIRD_LEAF_CONTAINER: u64 =
+    2 * INODE_LEAF_RECORDS + 5;
+
+/// 回退到没有第四片的那一版之前建几个 inode：建满三片，再往第四片里建 2 条——根环里最新那条根的第四片 2 条、
+/// 第 2 新的 1 条、第 3 新的那一版三片、第三片满、还没有第四片。
+const INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_VERSION_WITHOUT_THE_FOURTH_LEAF_CONTAINER: u64 =
+    3 * INODE_LEAF_RECORDS + 2;
+
+/// 两次回退都退到根环里按 (txg, 实例) 从新到旧第几条（从 0 数）：第 3 新那条，也就是最新那一版往前数两次建 inode 的那一版。
+const ROLLBACK_TARGET_INDEX_FROM_NEWEST: u64 = 2;
+
+/// 回退之后再建几个 inode：第一个重写目标那一版的最右一片（或在它的末尾分出下一片），后两个接着重写。
+const INODE_CREATIONS_AFTER_THE_ROLLBACK: u64 = 3;
+
+fn inode_creations(count: u64) -> impl Iterator<Item = HistoryOperation> {
+    std::iter::repeat_n(
+        HistoryOperation::CreateInode,
+        usize::try_from(count).expect("几百次装得进 usize"),
+    )
+}
+
+/// 起点 `AfterFirstFile`（mkfs、取号、暖机、第一个文件），每一步不跑池级 checker、4 GiB 设备、空间按公式准入。
+fn execute_from_after_the_first_file_without_the_per_step_checker(
+    operations: Vec<HistoryOperation>,
+) -> (GeneratedHistory, HistoryRun) {
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations,
+    };
+    let run = execute_history_with(
+        &history,
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: HistoryDeviceWidth::FourGibibytes,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        },
+        &SharedStream::new(),
+        &mut |_| {},
+    );
+    (history, run)
+}
+
+/// unlink 第一个文件，一次建一个 inode 建 703 个。模型与实现每一步都对得上：历史跑完、没有新发现，
+/// unlink 与每一次建 inode 都做成。
+/// 判别力：胶水把第四片及以后都当第三片（实现交回的叶容器号封顶在 2），或模型把落点封顶在第三片，
+/// 建第 700 个 inode（分出第四片）那一步判对不上、历史停在那里。
+#[test]
+fn creating_inodes_one_at_a_time_into_the_fourth_inode_leaf_container_is_judged_by_the_model_container_by_container(
+) {
+    let mut operations = vec![HistoryOperation::Unlink];
+    operations.extend(inode_creations(
+        INODE_CREATIONS_INTO_THE_FOURTH_LEAF_CONTAINER,
+    ));
+    let (history, run) = execute_from_after_the_first_file_without_the_per_step_checker(operations);
+    assert_eq!(
+        run.ending,
+        HistoryEnding::Completed,
+        "跑到第 {} 步：{:?}",
+        run.outcomes.len(),
+        run.outcomes.last()
+    );
+    assert_eq!(
+        run.outcomes.len(),
+        history.operations.len(),
+        "每一步操作各有一个结局"
+    );
+    let not_published: Vec<(usize, &StepOutcome)> = run
+        .outcomes
+        .iter()
+        .enumerate()
+        .filter(|(_, outcome)| {
+            !matches!(
+                outcome,
+                StepOutcome::Applied(AppliedEffect::Published { .. })
+            )
+        })
+        .collect();
+    assert!(
+        not_published.is_empty(),
+        "unlink 与每一次建 inode 都做成：{not_published:?}"
+    );
+    let created = run
+        .tally
+        .operations_by_kind
+        .get(&HistoryOperationKind::CreateInode)
+        .copied()
+        .expect("建过 inode：这一类在计数里");
+    assert_eq!(
+        created.applied, INODE_CREATIONS_INTO_THE_FOURTH_LEAF_CONTAINER,
+        "建过的 inode 多于三片叶容器装的 3 × {INODE_LEAF_RECORDS} 条：{created:?}"
+    );
+    let counts = &run.tally.model_counts;
+    assert!(
+        counts.successes_matched > INODE_CREATIONS_INTO_THE_FOURTH_LEAF_CONTAINER,
+        "模型判过 unlink 与每一次建 inode：{counts:?}"
+    );
+    assert!(
+        counts.allocation_records_compared > INODE_CREATIONS_INTO_THE_FOURTH_LEAF_CONTAINER,
+        "每一次建 inode 都逐个角色比过分配记录：{counts:?}"
+    );
+}
+
+/// unlink 第一个文件，建 471 个 inode（第三片 5 条），回退到根环里第 3 新那条根（第三片 3 条），再建 3 个。
+/// 回退那次发布的新根取目标那一版的叶容器（D23（journal 的角色与格式） 已定项 14）：第三片两版都有、现行那一版之后重写过两次，
+/// 模型要把第三片的分配代整组取回目标那一版的。模型与实现每一步都对得上：历史跑完、没有新发现，回退做成，
+/// 回退前后每一次建 inode 都做成。
+/// 判别力：模型回退时只取回目标那一版的第二片（`role_written_at.extend` 那一句的过滤只留第二片），
+/// 第三片的分配代留在现行那一版的，回退那一步判对不上、历史停在那里。
+#[test]
+fn a_rollback_to_a_version_whose_third_inode_leaf_container_was_rewritten_since_then_creating_inodes_is_judged_by_the_model_container_by_container(
+) {
+    let mut operations = vec![HistoryOperation::Unlink];
+    operations.extend(inode_creations(
+        INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_REWRITTEN_THIRD_LEAF_CONTAINER,
+    ));
+    let rollback_step = operations.len();
+    operations.push(HistoryOperation::RollBackWhileMounted(
+        RollbackTargetChoice::RingRoot {
+            index_from_newest: ROLLBACK_TARGET_INDEX_FROM_NEWEST,
+        },
+    ));
+    operations.extend(inode_creations(INODE_CREATIONS_AFTER_THE_ROLLBACK));
+    let (history, run) = execute_from_after_the_first_file_without_the_per_step_checker(operations);
+    assert_eq!(
+        run.ending,
+        HistoryEnding::Completed,
+        "跑到第 {} 步：{:?}",
+        run.outcomes.len(),
+        run.outcomes.last()
+    );
+    assert_eq!(
+        run.outcomes.len(),
+        history.operations.len(),
+        "每一步操作各有一个结局"
+    );
+    assert!(
+        matches!(
+            run.outcomes[rollback_step],
+            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
+        ),
+        "回退到根环里第 3 新那条根做成：{:?}",
+        run.outcomes[rollback_step]
+    );
+    let not_published: Vec<(usize, &StepOutcome)> = run
+        .outcomes
+        .iter()
+        .enumerate()
+        .filter(|(step, _)| *step != rollback_step)
+        .filter(|(_, outcome)| {
+            !matches!(
+                outcome,
+                StepOutcome::Applied(AppliedEffect::Published { .. })
+            )
+        })
+        .collect();
+    assert!(
+        not_published.is_empty(),
+        "unlink 与回退前后每一次建 inode 都做成：{not_published:?}"
+    );
+    let created = run
+        .tally
+        .operations_by_kind
+        .get(&HistoryOperationKind::CreateInode)
+        .copied()
+        .expect("建过 inode：这一类在计数里");
+    assert_eq!(
+        created.applied,
+        INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_REWRITTEN_THIRD_LEAF_CONTAINER
+            + INODE_CREATIONS_AFTER_THE_ROLLBACK,
+        "回退前后的建 inode 都做成：{created:?}"
+    );
+}
+
+/// unlink 第一个文件，建 701 个 inode（第四片 2 条），回退到根环里第 3 新那条根（三片、第三片满、没有第四片），再建 3 个
+/// （第一个在目标那一版的末尾重新分出第四片）。模型要把现行那一版有、目标那一版没有的第四片丢掉。
+/// 模型与实现每一步都对得上：历史跑完、没有新发现，回退做成，回退前后每一次建 inode 都做成。
+/// 判别力：模型回退时只丢第三片、更右的留着（`role_written_at.retain` 那一句的过滤只丢第三片），
+/// 现行那一版的第四片的分配代留下，回退那一步判对不上、历史停在那里；
+/// 胶水把第四片及以后都当第三片、模型把落点封顶在第三片、回退只取回第二片，在这条上同样判对不上。
+#[test]
+fn a_rollback_to_a_version_without_the_fourth_inode_leaf_container_then_creating_inodes_is_judged_by_the_model_container_by_container(
+) {
+    let mut operations = vec![HistoryOperation::Unlink];
+    operations.extend(inode_creations(
+        INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_VERSION_WITHOUT_THE_FOURTH_LEAF_CONTAINER,
+    ));
+    let rollback_step = operations.len();
+    operations.push(HistoryOperation::RollBackWhileMounted(
+        RollbackTargetChoice::RingRoot {
+            index_from_newest: ROLLBACK_TARGET_INDEX_FROM_NEWEST,
+        },
+    ));
+    operations.extend(inode_creations(INODE_CREATIONS_AFTER_THE_ROLLBACK));
+    let (history, run) = execute_from_after_the_first_file_without_the_per_step_checker(operations);
+    assert_eq!(
+        run.ending,
+        HistoryEnding::Completed,
+        "跑到第 {} 步：{:?}",
+        run.outcomes.len(),
+        run.outcomes.last()
+    );
+    assert_eq!(
+        run.outcomes.len(),
+        history.operations.len(),
+        "每一步操作各有一个结局"
+    );
+    assert!(
+        matches!(
+            run.outcomes[rollback_step],
+            StepOutcome::Applied(AppliedEffect::RolledBack { .. })
+        ),
+        "回退到根环里第 3 新那条根做成：{:?}",
+        run.outcomes[rollback_step]
+    );
+    let not_published: Vec<(usize, &StepOutcome)> = run
+        .outcomes
+        .iter()
+        .enumerate()
+        .filter(|(step, _)| *step != rollback_step)
+        .filter(|(_, outcome)| {
+            !matches!(
+                outcome,
+                StepOutcome::Applied(AppliedEffect::Published { .. })
+            )
+        })
+        .collect();
+    assert!(
+        not_published.is_empty(),
+        "unlink 与回退前后每一次建 inode 都做成：{not_published:?}"
+    );
+    let created = run
+        .tally
+        .operations_by_kind
+        .get(&HistoryOperationKind::CreateInode)
+        .copied()
+        .expect("建过 inode：这一类在计数里");
+    assert_eq!(
+        created.applied,
+        INODE_CREATIONS_BEFORE_THE_ROLLBACK_TO_A_VERSION_WITHOUT_THE_FOURTH_LEAF_CONTAINER
+            + INODE_CREATIONS_AFTER_THE_ROLLBACK,
+        "回退前后的建 inode 都做成：{created:?}"
+    );
+}
````

### mutations-append.tsv

````text
O1 跨叶容器（胶水）：第 4 片及以后都当第 3 片（第 29 批攻方，判决 L1 第三行）	crates/singlefs-harness/src/model_comparison.rs	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0))	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0).min(2))	-p singlefs-harness --test model_compares_the_third_and_fourth_inode_leaf_containers	creating_inodes_one_at_a_time_into_the_fourth_inode_leaf_container_is_judged_by_the_model_container_by_container
O1 跨叶容器（理想模型自身）：落点封顶在第 3 片（第 29 批攻方，判决 L1 第三行）	crates/singlefs-harness/src/model.rs	    let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;	    let position_from_the_leftmost = (records_before / INODE_LEAF_RECORDS).min(2);	-p singlefs-harness --test model_compares_the_third_and_fourth_inode_leaf_containers	creating_inodes_one_at_a_time_into_the_fourth_inode_leaf_container_is_judged_by_the_model_container_by_container
O1 跨叶容器回退（理想模型自身）：回退只取回目标那一版的第 2 片（第 29 批攻方，判决 L1 第三行）	crates/singlefs-harness/src/model.rs	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf) if leaf.position_from_the_leftmost.get() == 1)\n                        })	-p singlefs-harness --test model_compares_the_third_and_fourth_inode_leaf_containers	a_rollback_to_a_version_whose_third_inode_leaf_container_was_rewritten_since_then_creating_inodes_is_judged_by_the_model_container_by_container
O1 跨叶容器回退（理想模型自身）：回退只丢现行那一版的第 3 片、更右的留着（第 29 批攻方，判决 L1 第三行）	crates/singlefs-harness/src/model.rs	                role_written_at.retain(|role, _| {\n                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                });	                role_written_at.retain(|role, _| {\n                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf) if leaf.position_from_the_leftmost.get() == 2)\n                });	-p singlefs-harness --test model_compares_the_third_and_fourth_inode_leaf_containers	a_rollback_to_a_version_without_the_fourth_inode_leaf_container_then_creating_inodes_is_judged_by_the_model_container_by_container
````

## m3-formal-30-2

### crates.patch

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 797ddc1..d2af769 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -2062,6 +2062,7 @@ impl<'reader> Walk<'reader> {
             let what = format!("{name}内部条目的子指针");
             judge_location_order(&mut self.judgements, &child, &what);
             judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &child, &what);
+            judge_pointer_extent_offset_is_zero(&mut self.judgements, &child, &what);
             if entry_type == 0 {
                 if node.level <= 1 {
                     self.walk_failures.push(format!(
@@ -4882,9 +4883,11 @@ fn judge_release_generation_and_tree_table_birth(
 /// （D22（单元原子性怎么合成） 已定项 7 字段表「分配记录树根指针」那一行；判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`
 /// 第二节 Y1 `allocation_record_tree_root` 那一行）。非零时这一版有两棵分配记录树，「这条根的分配记录树」不止一棵。
 /// 树表读不出的根这一格不判（认不出带不带树）。不变量清单里没有一条专管根记录这一项，暂挂在 I-5.4 下（归属交主 agent 定）。
-/// 这一格成立的那一次评估只在候选集里至少一棵分配记录树走到叶时计进 I-5.4（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md`
-/// 第二节 J1「I-5.4 挂法（C2）」那一行）：一棵都没走到时不计，I-5.4 报部分没判、逐版列出没计的那几版——否则带树一版上这一格成立一次就压过
-/// 不适用，分配记录树两份清零的镜像上 I-5.4 报成立。不成立照报违例（[`Judgements::partially_judged`] 的契约：判出违例照报违例）。
+/// 这一格成立的那一次评估只在**这一版自己的**分配记录树（树表种类 3 那一条指着的那一棵）走到叶时计进 I-5.4（判决
+/// `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法（C2）」那一行，按版本那一层是
+/// `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2 的零轮收严）：没走到的那几版不计，I-5.4 报部分没判、逐版列出没计的那几版
+/// ——否则带树一版上这一格成立一次就压过不适用或别的版本没判到的那一半，这一版分配记录树两份清零的镜像上 I-5.4 报成立。
+/// 不成立照报违例（[`Judgements::partially_judged`] 的契约：判出违例照报违例）。
 fn judge_allocation_records_disjoint(
     reader: &dyn ImageReader,
     roots: &[(u64, u64, crate::RootView)],
@@ -4893,12 +4896,18 @@ fn judge_allocation_records_disjoint(
     judgements: &mut Judgements,
 ) {
     let mut seen_allocation_nodes: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
-    let mut judged_allocation_nodes = 0u64;
-    // 带树一版「根记录里树表 0 条那一版的分配记录树根全零」那一格：不成立当场照报违例；成立的那几版按走到的次序记下哪一版，
-    // 走完全部候选根之后才定计不计进评估（至少一棵分配记录树走到叶才计）。
-    let mut versions_where_the_root_record_allocation_tree_root_is_zero: Vec<String> = Vec::new();
+    // 走到了叶的分配记录树根（按第一条位置条目认，与 `seen_allocation_nodes` 同一个键）：别的版本照抄同一个根时，
+    // 它那一版的树也算走到了叶（那棵树只判一次）；先走的那一版读不出的，照抄它的那一版同样算没走到。
+    let mut allocation_roots_walked_to_their_leaves: BTreeSet<(u32, u64, u32)> = BTreeSet::new();
+    // 带树一版「根记录里树表 0 条那一版的分配记录树根全零」那一格：不成立当场照报违例；成立的那几版按走到的次序记下哪一版、
+    // 这一版自己的分配记录树走没走到叶，走完全部候选根之后才逐版定计不计进评估（判决
+    // `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2「I-5.4『成立要至少一棵树走到叶』」那一行的零轮收严：
+    // 按版本记，别的版本的树走到了叶不替这一版作保）。
+    let mut versions_where_the_root_record_allocation_tree_root_is_zero: Vec<(String, bool)> =
+        Vec::new();
     for index in candidate_indexes.iter().copied() {
         let (_, _, root) = &roots[index];
+        let mut this_version_carries_trees_and_a_zero_root_record_allocation_tree_root = false;
         // 这条根的分配记录树：树表里种类 3 那一条指着的根，以及树表 0 条那一版由根记录直接持有的那一棵
         // （C512（树表 0 条的一版上被换下的单元记在哪），代码审阅 6c 第 9 条：原来这一棵不读）。带文件的一版后者恒全零。
         let mut allocation_roots = vec![parse_node_pointer(
@@ -4911,10 +4920,8 @@ fn judge_allocation_records_disjoint(
             {
                 if !tree_table.entries.is_empty() {
                     if allocation_roots[0].all_zero {
-                        versions_where_the_root_record_allocation_tree_root_is_zero.push(format!(
-                            "根（实例 {}、txg {}）",
-                            root.instance, root.checkpoint_txg
-                        ));
+                        this_version_carries_trees_and_a_zero_root_record_allocation_tree_root =
+                            true;
                     } else {
                         judgements.judge("I-5.4", false, || {
                             format!(
@@ -4938,16 +4945,21 @@ fn judge_allocation_records_disjoint(
                 );
             }
         }
+        let mut this_version_allocation_record_tree_walked_to_its_leaves = false;
         for allocation_root in allocation_roots {
             if allocation_root.all_zero {
                 continue;
             }
             let first_location = &allocation_root.locations[0];
-            if !seen_allocation_nodes.insert((
+            let allocation_root_key = (
                 first_location.device,
                 first_location.slot,
                 first_location.checksum,
-            )) {
+            );
+            if !seen_allocation_nodes.insert(allocation_root_key) {
+                if allocation_roots_walked_to_their_leaves.contains(&allocation_root_key) {
+                    this_version_allocation_record_tree_walked_to_its_leaves = true;
+                }
                 continue;
             }
             let Some((_, records)) =
@@ -4956,28 +4968,47 @@ fn judge_allocation_records_disjoint(
                 continue;
             };
             judge_allocation_record_ranges(&records, root.checkpoint_txg, judgements);
-            judged_allocation_nodes += 1;
+            allocation_roots_walked_to_their_leaves.insert(allocation_root_key);
+            this_version_allocation_record_tree_walked_to_its_leaves = true;
+        }
+        if this_version_carries_trees_and_a_zero_root_record_allocation_tree_root {
+            versions_where_the_root_record_allocation_tree_root_is_zero.push((
+                format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
+                this_version_allocation_record_tree_walked_to_its_leaves,
+            ));
         }
     }
-    let some_allocation_record_tree_walked_to_its_leaves = judged_allocation_nodes > 0;
-    if some_allocation_record_tree_walked_to_its_leaves {
-        for _version in &versions_where_the_root_record_allocation_tree_root_is_zero {
+    let some_allocation_record_tree_walked_to_its_leaves =
+        !allocation_roots_walked_to_their_leaves.is_empty();
+    let mut versions_not_counted: Vec<&str> = Vec::new();
+    for (version, this_version_allocation_record_tree_walked_to_its_leaves) in
+        &versions_where_the_root_record_allocation_tree_root_is_zero
+    {
+        if *this_version_allocation_record_tree_walked_to_its_leaves {
             judgements.judge("I-5.4", true, String::new);
+        } else {
+            versions_not_counted.push(version);
         }
-    } else {
+    }
+    if !some_allocation_record_tree_walked_to_its_leaves {
         judgements.not_applicable(
             "I-5.4",
             "候选集里没有一条根的分配记录树走得到叶：第 0 代树表是空的，或树表 / 分配记录树读不出、位置对不上",
         );
-        if !versions_where_the_root_record_allocation_tree_root_is_zero.is_empty() {
-            judgements.partially_judged(
-                "I-5.4",
-                format!(
-                    "候选集里没有一条根的分配记录树走得到叶，记录之间互不相交一格都没判；带树一版根记录里分配记录树根全零那一格成立而没计：{}",
-                    versions_where_the_root_record_allocation_tree_root_is_zero.join("、")
-                ),
-            );
-        }
+    }
+    if !versions_not_counted.is_empty() {
+        let unjudged = if some_allocation_record_tree_walked_to_its_leaves {
+            format!(
+                "带树一版根记录里分配记录树根全零那一格成立而没计（那一版自己的分配记录树读不出、走不到叶，别的版本的树走到了叶不替它作保）：{}",
+                versions_not_counted.join("、")
+            )
+        } else {
+            format!(
+                "候选集里没有一条根的分配记录树走得到叶，记录之间互不相交一格都没判；带树一版根记录里分配记录树根全零那一格成立而没计：{}",
+                versions_not_counted.join("、")
+            )
+        };
+        judgements.partially_judged("I-5.4", unjudged);
     }
 }
 
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index 6330e9c..e3f8dab 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -2926,6 +2926,7 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
         let child = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
         w_judge_location_order(child);
         w_judge_pointer_head(child);
+        w_judge_pointer_extent_offset(child);
         if (entry_type == 0u) {
             if (n.level <= 1u) {
                 w_failures = w_failures + 1u;
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index 7044e51..306dd0f 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -504,6 +504,13 @@ const POINTER_EXTENT_OFFSET_FIELD_OFFSET: u32 = 32u;
 fn pointer_extent_offset_is_zero(base: u32, o: u32) -> bool {
     return rd16(base, o + POINTER_EXTENT_OFFSET_FIELD_OFFSET) == 0u;
 }
+// 偏移 o 起那条指针头部 extent 偏移之前的几段是第一版的取值（pointer.rs 的 PointerHeadAsRead::judged 在 extent 偏移之前判的那几段）：
+// MAC 16 与 nonce 12 全 0、算法类型 0（不加密）、压缩算法码 0、压后长度 0。core 读走读跟随的指针（树表条目树根指针、inode 内部条目子指针、
+// extent 上段叶两种指针、下段叶数据指针、extent 内部条目子指针、分配记录树内部条目子指针、实例表链指针）都经
+// read_judging_the_encryption_compression_and_extent_offset_fields_from 一并判这几段与 extent 偏移，有一段不是第一版的取值就当那个结构损坏
+fn pointer_encryption_and_compression_fields_are_the_first_version(base: u32, o: u32) -> bool {
+    return all_zero(base, o, 28u) && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;
+}
 
 struct Ptr {
     all_zero: bool,
@@ -525,7 +532,7 @@ fn parse_pointer_common(base: u32, o: u32) -> Ptr {
     var p: Ptr;
     p.all_zero = false;
     p.mac_nonce_zero = all_zero(base, o, 28u);
-    p.head_ok = p.mac_nonce_zero && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;
+    p.head_ok = pointer_encryption_and_compression_fields_are_the_first_version(base, o);
     p.extent_offset_zero = pointer_extent_offset_is_zero(base, o);
     p.birth_tree = rd64(base, o + 34u);
     p.birth_txg = rd64(base, o + 42u);
@@ -2221,6 +2228,10 @@ fn read_allocation_tree(root_pointer: Ptr, root_pointer_base: u32, root_pointer_
         allocation_frames[top].previous_child_device = device;
         allocation_frames[top].previous_child_index = child.index;
         pending_pointer = parse_node_pointer(n.base, o + 10u);
+        // allocation_record_tree 读内部条目的子指针：头部加密、压缩那几段不是第一版的取值，这个节点损坏
+        if (!pending_pointer.head_ok) {
+            return out;
+        }
         // allocation_record_tree 读内部条目的子指针：头部 extent 偏移非 0，这个节点损坏
         if (!pending_pointer.extent_offset_zero) {
             return out;
@@ -2267,10 +2278,14 @@ fn extent_key_offset(base: u32, o: u32) -> vec2<u32> { return rd64(base, o + 16u
 fn extent_key_equals(base: u32, o: u32, inode: vec2<u32>, offset: vec2<u32>) -> bool {
     return is_zero64(rd64(base, o)) && eq64(rd64(base, o + 8u), inode) && eq64(rd64(base, o + 16u), offset);
 }
-// extent_tree 的 internal_entries：一个内部节点的每条子指针（条目 key 24 之后）头部 extent 偏移都为 0；
-// core 在往下读第一个孩子之前就把整个节点的条目解完，有一条非 0 这个节点就损坏
-fn extent_internal_child_pointers_carry_no_extent_offset(n: Node) -> bool {
+// extent_tree 的 internal_entries：一个内部节点的每条子指针（条目 key 24 之后）头部加密、压缩那几段是第一版的取值、extent 偏移为 0
+// （NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from）；
+// core 在往下读第一个孩子之前就把整个节点的条目解完，有一条不是这个节点就损坏
+fn extent_internal_child_pointer_heads_are_the_first_version(n: Node) -> bool {
     for (var e = 0u; e < n.entry_count; e = e + 1u) {
+        if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, entry_offset(n, e) + 24u)) {
+            return false;
+        }
         if (!pointer_extent_offset_is_zero(n.base, entry_offset(n, e) + 24u)) {
             return false;
         }
@@ -2370,11 +2385,19 @@ fn read_upper_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
                         if (!all_zero(n.base, o + 25u + 86u, 2u)) {
                             return out;
                         }
+                        // ExtentUpperLeafEntry::parse：下段根指针头部加密、压缩那几段不是第一版的取值，这条条目损坏
+                        if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 25u)) {
+                            return out;
+                        }
                         // ExtentUpperLeafEntry::parse：下段根指针头部 extent 偏移非 0，这条条目损坏
                         if (!pointer_extent_offset_is_zero(n.base, o + 25u)) {
                             return out;
                         }
                     } else if (tag == 2u) {
+                        // 同上：内联的数据指针（加密、压缩那几段与 extent 偏移各判一次）
+                        if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 25u)) {
+                            return out;
+                        }
                         // 同上：内联的数据指针
                         if (!pointer_extent_offset_is_zero(n.base, o + 25u)) {
                             return out;
@@ -2408,7 +2431,7 @@ fn read_upper_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
             if (n.entry_width < 110u) {
                 return out;
             }
-            if (!extent_internal_child_pointers_carry_no_extent_offset(n)) {
+            if (!extent_internal_child_pointer_heads_are_the_first_version(n)) {
                 return out;
             }
             if (depth >= MAX_FRAMES) {
@@ -2520,6 +2543,10 @@ fn read_lower_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
                     if (!is_zero64(extent_key_locality(n.base, o)) || !eq64(extent_key_inode(n.base, o), inode) || !is_zero64(qr.remainder)) {
                         return out;
                     }
+                    // records::parse_extent_record：数据指针头部加密、压缩那几段不是第一版的取值，这条叶记录损坏
+                    if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 24u)) {
+                        return out;
+                    }
                     // records::parse_extent_record：数据指针头部 extent 偏移非 0，这条叶记录损坏
                     if (!pointer_extent_offset_is_zero(n.base, o + 24u)) {
                         return out;
@@ -2556,7 +2583,7 @@ fn read_lower_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
             if (n.entry_width < 110u) {
                 return out;
             }
-            if (!extent_internal_child_pointers_carry_no_extent_offset(n)) {
+            if (!extent_internal_child_pointer_heads_are_the_first_version(n)) {
                 return out;
             }
             if (depth >= MAX_FRAMES) {
@@ -2705,8 +2732,11 @@ fn parse_instance_table_page(p: Packed, page_index: vec2<u32>) -> PageParse {
         }
         out.verdict = 0u;
     } else if (flag == 1u) {
-        // InstanceTableChainRecord::parse（走读那一种判）：链指针头部 extent 偏移非 0，这一片损坏
-        if (pointer_zero || !pointer_extent_offset_is_zero(p.base, last + 2u)) {
+        // InstanceTableChainRecord::parse（走读那一种判）：链指针头部加密、压缩那几段不是第一版的取值，或 extent 偏移非 0，这一片损坏
+        if (pointer_zero || !pointer_encryption_and_compression_fields_are_the_first_version(p.base, last + 2u)) {
+            return out;
+        }
+        if (!pointer_extent_offset_is_zero(p.base, last + 2u)) {
             return out;
         }
         out.verdict = 1u;
@@ -2942,18 +2972,30 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
         if (rd16(tree_table.base, o + 8u) != 200u || rd16(tree_table.base, o + 12u) != 0u || !all_zero(tree_table.base, o + 124u, 76u)) {
             return walk_failed(23u);
         }
+        // TreeTableEntry::parse：树根指针头部加密、压缩那几段不是第一版的取值，这条条目损坏
+        if (!pointer_encryption_and_compression_fields_are_the_first_version(tree_table.base, o + 14u)) {
+            return walk_failed(65u);
+        }
         // TreeTableEntry::parse：树根指针头部 extent 偏移非 0，这条条目损坏（与条目长度、flags、预留 76 同在读任何一棵树之前拒）
         if (!pointer_extent_offset_is_zero(tree_table.base, o + 14u)) {
             return walk_failed(63u);
         }
         let kind = rd16(tree_table.base, o + 10u);
+        // tree_table_entries_each_kind_at_most_once 的 kinds_seen 是整个 u16 的集合：种类 < 32 的记进位图，
+        // 种类 ≥ 32 的（第一版写者写不出、坏盘上才有）逐条与前面各条比（树表条目数不超过一个码 2 节点装得下的，与走读内核 ③ 的两两比同一量级）。
+        // 走读内核的 ③（crash_judge_checker.wgsl 判 I-9.16 那一段）同样比整个 u16，两个内核在种类 ≥ 32 的重复上都拦得到
         if (kind < 32u) {
             if ((kinds_seen & (1u << kind)) != 0u) {
                 return walk_failed(24u);
             }
             kinds_seen = kinds_seen | (1u << kind);
+        } else {
+            for (var earlier = 0u; earlier < e; earlier = earlier + 1u) {
+                if (rd16(tree_table.base, entry_offset(tree_table, earlier) + 10u) == kind) {
+                    return walk_failed(24u);
+                }
+            }
         }
-        // 与前一条比种类：种类 ≥ 32 的两条同种拦不到，走读同样拦不到（kinds 是 u16，但一条流写不出这种树）
     }
     // I-9.16 ①：树 ID 严格升序
     for (var e = 1u; e < tree_table.entry_count; e = e + 1u) {
@@ -3184,6 +3226,10 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
             return walk_failed(46u);
         }
         let child = parse_node_pointer(inode_root.base, o + 34u);
+        // records::parse_inode_internal_entry_or_refusal：子指针头部加密、压缩那几段不是第一版的取值，这条内部条目损坏
+        if (!child.head_ok) {
+            return walk_failed(66u);
+        }
         // records::parse_inode_internal_entry_or_refusal：子指针头部 extent 偏移非 0，这条内部条目损坏
         if (!child.extent_offset_zero) {
             return walk_failed(64u);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index d00933a..685ccdd 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -3,7 +3,8 @@
 //! 七类 oracle、记录核对器两条判据，判法与 CPU 判器（`crate::crash::judge_crash_image`）逐条相同；GPU 与 CPU 逐状态比是它的判别力自证。
 //! 内核分五份源码：`crash_judge_common.wgsl`（各内核共用的解析与查表）、`crash_judge_recovery.wgsl`（恢复、oracle、记录核对器）、
 //! `crash_judge_checker_common.wgsl`（池级 checker 两个内核共用的读法与交接段）、`crash_judge_checker.wgsl`（走读内核：第一、二批不变量，
-//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12、I-5.5、I-9.28，与里程碑三·3 删除线的删除态扫描、I-9.6 墓碑那一半）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。
+//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12、I-5.5、I-9.28，与里程碑三·3 删除线的删除态扫描、I-9.6 墓碑那一半）；判不了的状态（表越过内核容量）置最高位，
+//! 放量路径（`crate::crash_amplification` 收卡上结论那一段）遇到它就让整趟停下、报哪张表满了，配置开了 GPU 不退回 CPU 判器。
 //!
 //! 一个状态的结论 [`VERDICT_WORDS`] 个 u32：第 0 字是红位（位 0..6 看 journal 那一遍 oracle 的七类、位 8..14 不看那一遍、
 //! 位 16 根在而记录都不在、位 17 恢复自称的状态少单元、位 31 判不了）；第 1 字判不了的原因（判得了时低 16 位是看 journal 那一遍
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 7ebad14..050b98b 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -14,7 +14,9 @@
 //! 不展开的段（新建文件、顺序写、意图、第一批那几次发布的单元写）另把「之前的段全落、这一段只差一个写」的每个状态各比一次。
 //! I-9.16 ③ ④（种类重复、缺 deadlist）、根槽 428..457（I-2.4）、带树一版根记录里的分配记录树根（I-5.4）各一条坏流整份比；指针头部的
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
-//! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）。
+//! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
+//! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）。墓碑树与谱系树里指向容器的子指针 extent 偏移各一条
+//! （墓碑树那条在删除中间态的流上），树表里两条种类 40（≥ 32）一条，都整份比。
 //! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
 //! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性；
 //! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（没设不定额度）、
@@ -4193,27 +4195,29 @@ fn record_new_pool_with_the_allocation_record_tree_root_of_the_root_record_not_z
     flow
 }
 
-/// I-2.4 指针那一半的 extent 偏移那一格：发布那张树表里 inode 树那条（种类 2）的根指针头部 extent 偏移写 1、重封到根。
-/// 池级 checker 读码 2 节点（`read_index_node`）时判它，CPU 与 GPU 走读内核都红 I-2.4。core 读树表条目时判这 2 字节、把那一条当损坏拒
-/// （`TreeTableEntry::parse`），GPU 恢复内核在 `walk_to_file` 解树表条目那一处同拒：这一版两遍恢复都是读失败。
-fn record_new_pool_with_an_extent_offset_on_the_inode_tree_root_pointer(tag: &str) -> RecordedFlow {
-    record_new_pool_with_a_resealed_field(
-        tag,
-        "new-pool-inode-tree-root-pointer-extent-offset-one",
-        &is_a_tree_table_unit,
-        false,
-        &|bytes| {
-            let entry = only_tree_table_entry_of_kind(bytes, 2);
-            let extent_offset =
-                entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET + POINTER_EXTENT_OFFSET_OFFSET;
-            assert_eq!(
-                bytes[extent_offset..extent_offset + 2],
-                [0u8; 2],
-                "第一版指针的 extent 偏移恒 0"
-            );
-            bytes[extent_offset..extent_offset + 2].copy_from_slice(&1u16.to_le_bytes());
-        },
-    )
+/// I-2.4 指针那一半：发布那张树表里 inode 树那条（种类 2）的根指针头部 `field` 那一段写成非第一版的取值、重封到根。
+/// 池级 checker 读码 2 节点（`read_index_node`）时判 MAC / nonce 与 extent 偏移，CPU 与 GPU 走读内核都红 I-2.4。core 读树表条目时判加密、压缩
+/// 那几段与 extent 偏移、把那一条当损坏拒（`TreeTableEntry::parse`），GPU 恢复内核在 `walk_to_file` 解树表条目那一处同拒：这一版两遍恢复都是读失败。
+fn record_new_pool_with_a_non_first_version_head_on_the_inode_tree_root_pointer(
+    tag: &str,
+    field: PointerHeadFieldWrittenNonZero,
+) -> RecordedFlow {
+    let name = match field {
+        PointerHeadFieldWrittenNonZero::ExtentOffset => {
+            "new-pool-inode-tree-root-pointer-extent-offset-one"
+        }
+        PointerHeadFieldWrittenNonZero::FirstMacByte => {
+            "new-pool-inode-tree-root-pointer-first-mac-byte-one"
+        }
+    };
+    record_new_pool_with_a_resealed_field(tag, name, &is_a_tree_table_unit, false, &|bytes| {
+        let entry = only_tree_table_entry_of_kind(bytes, 2);
+        write_the_pointer_head_field_non_zero_at(
+            bytes,
+            entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET,
+            field,
+        );
+    })
 }
 
 // ───────────── 第 29 批 29-2：指针头部 extent 偏移，恢复那一半照 core 判、池级 checker 那一半补三处 ─────────────
@@ -4247,6 +4251,32 @@ fn set_the_extent_offset_of_the_pointer_at(bytes: &mut [u8], pointer: usize) {
     bytes[extent_offset..extent_offset + 2].copy_from_slice(&1u16.to_le_bytes());
 }
 
+/// 指针头部里写成非第一版取值的那一段（D19（块指针的结构与宽度预算） 已定项 7 的字段表）：extent 偏移（偏移 32 的 2 字节）写 1，
+/// 或 MAC 第 0 字节写 1（加密关着时 MAC 恒 0，D19 已定项 3）。core 读走读跟随的指针都经
+/// `read_judging_the_encryption_compression_and_extent_offset_fields_from` 两段一并判，GPU 恢复内核两段各一判（第 30 批 30-2 起）。
+#[derive(Clone, Copy, Debug)]
+enum PointerHeadFieldWrittenNonZero {
+    ExtentOffset,
+    FirstMacByte,
+}
+
+/// 偏移 `pointer` 起那条指针头部 `field` 那一段（本来 0）写成非 0。
+fn write_the_pointer_head_field_non_zero_at(
+    bytes: &mut [u8],
+    pointer: usize,
+    field: PointerHeadFieldWrittenNonZero,
+) {
+    match field {
+        PointerHeadFieldWrittenNonZero::ExtentOffset => {
+            set_the_extent_offset_of_the_pointer_at(bytes, pointer);
+        }
+        PointerHeadFieldWrittenNonZero::FirstMacByte => {
+            assert_eq!(bytes[pointer], 0, "加密关着时指针头部的 MAC 恒 0");
+            bytes[pointer] = 1;
+        }
+    }
+}
+
 /// 根记录实例表指针那一处：发布的根记录里实例表指针头部 extent 偏移写 1、重封根记录。池级 checker 走实例表链第 0 片时判它
 /// （`walk_instance_table_chain`），CPU 与 GPU 走读内核都红 I-2.4。core 择根时把这条根当损坏拒（`root_record.rs` 的 `parse_slot` 第五关），
 /// 退回上一条根（看 journal 那一遍再重放回这一版）；GPU 恢复内核择根同拒（`root_record_pointers_carry_no_extent_offset`）。
@@ -4264,30 +4294,34 @@ fn record_new_pool_with_an_extent_offset_on_the_instance_table_pointer_of_the_ro
     flow
 }
 
-/// inode 内部条目子指针那一处：发布那一版 inode 树根（码 2、key 宽 8、条目宽 120）第一条内部条目的子指针头部 extent 偏移写 1、重封到根。
-/// 池级 checker 走 inode 内部条目时判它，CPU 与 GPU 走读内核都红 I-2.4；core 解这条内部条目时拒
+/// inode 内部条目子指针那一处：发布那一版 inode 树根（码 2、key 宽 8、条目宽 120）第一条内部条目的子指针头部 `field` 那一段写成非第一版的取值、
+/// 重封到根。池级 checker 走 inode 内部条目时判它，CPU 与 GPU 走读内核都红 I-2.4；core 解这条内部条目时拒
 /// （`records::parse_inode_internal_entry_or_refusal`），GPU 恢复内核在 `walk_to_file` 读 inode 树那一处同拒。
-fn record_new_pool_with_an_extent_offset_on_an_inode_internal_entry_child_pointer(
+fn record_new_pool_with_a_non_first_version_head_on_an_inode_internal_entry_child_pointer(
     tag: &str,
+    field: PointerHeadFieldWrittenNonZero,
 ) -> RecordedFlow {
+    let name = match field {
+        PointerHeadFieldWrittenNonZero::ExtentOffset => {
+            "new-pool-inode-internal-entry-child-pointer-extent-offset-one"
+        }
+        PointerHeadFieldWrittenNonZero::FirstMacByte => {
+            "new-pool-inode-internal-entry-child-pointer-first-mac-byte-one"
+        }
+    };
     let inode_tree_root = |bytes: &[u8]| {
         unit_class(bytes) == 2
             && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
             && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
     };
-    record_new_pool_with_a_resealed_field(
-        tag,
-        "new-pool-inode-internal-entry-child-pointer-extent-offset-one",
-        &inode_tree_root,
-        false,
-        &|bytes| {
-            let first_entry = index_node_entries_start(INODE_TREE_KEY_BYTES);
-            set_the_extent_offset_of_the_pointer_at(
-                bytes,
-                first_entry + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET,
-            );
-        },
-    )
+    record_new_pool_with_a_resealed_field(tag, name, &inode_tree_root, false, &|bytes| {
+        let first_entry = index_node_entries_start(INODE_TREE_KEY_BYTES);
+        write_the_pointer_head_field_non_zero_at(
+            bytes,
+            first_entry + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET,
+            field,
+        );
+    })
 }
 
 /// extent 上段叶条目里的那条指针（标签之后）在单元里的偏移；条目的标签要是 `expected_tag`。
@@ -4307,49 +4341,58 @@ fn is_an_extent_upper_leaf(bytes: &[u8]) -> bool {
         && index_node_entry_width(bytes) == EXTENT_UPPER_LEAF_ENTRY_BYTES
 }
 
-/// extent 数据指针那一处（内联在上段叶里的那条）：发布那一版唯一的数据单元由上段叶第一条内联指着（标签 2），这条数据指针头部 extent 偏移写 1、
-/// 重封到根。池级 checker 走数据指针时判它（`walk_extent_data_pointer`），CPU 与 GPU 走读内核都红 I-2.4；core 解上段叶条目时拒
-/// （`ExtentUpperLeafEntry::parse`），GPU 恢复内核读上段叶那一处同拒。
-fn record_new_pool_with_an_extent_offset_on_the_inline_data_pointer(tag: &str) -> RecordedFlow {
-    record_new_pool_with_a_resealed_field(
-        tag,
-        "new-pool-inline-data-pointer-extent-offset-one",
-        &is_an_extent_upper_leaf,
-        false,
-        &|bytes| {
-            let pointer = pointer_of_the_first_extent_upper_leaf_entry(
-                bytes,
-                EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT,
-            );
-            set_the_extent_offset_of_the_pointer_at(bytes, pointer);
-        },
-    )
+/// extent 数据指针那一处（内联在上段叶里的那条）：发布那一版唯一的数据单元由上段叶第一条内联指着（标签 2），这条数据指针头部 `field` 那一段
+/// 写成非第一版的取值、重封到根。池级 checker 走数据指针时判它（`walk_extent_data_pointer`），CPU 与 GPU 走读内核都红 I-2.4；
+/// core 解上段叶条目时拒（`ExtentUpperLeafEntry::parse`），GPU 恢复内核读上段叶那一处同拒。
+fn record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
+    tag: &str,
+    field: PointerHeadFieldWrittenNonZero,
+) -> RecordedFlow {
+    let name = match field {
+        PointerHeadFieldWrittenNonZero::ExtentOffset => {
+            "new-pool-inline-data-pointer-extent-offset-one"
+        }
+        PointerHeadFieldWrittenNonZero::FirstMacByte => {
+            "new-pool-inline-data-pointer-first-mac-byte-one"
+        }
+    };
+    record_new_pool_with_a_resealed_field(tag, name, &is_an_extent_upper_leaf, false, &|bytes| {
+        let pointer = pointer_of_the_first_extent_upper_leaf_entry(
+            bytes,
+            EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT,
+        );
+        write_the_pointer_head_field_non_zero_at(bytes, pointer, field);
+    })
 }
 
 /// 分配记录树内部条目子指针那一处：发布那一版分配记录树的根（码 2、key 宽 10、条目宽 96；一棵树里最后写出的内部节点）第一条内部条目的子指针
-/// 头部 extent 偏移写 1、重封到根。池级 checker 读分配记录树的码 2 节点时判它（`read_index_node`），CPU 与 GPU 走读内核都红 I-2.4；
+/// 头部 `field` 那一段写成非第一版的取值、重封到根。池级 checker 读分配记录树的码 2 节点时判它（`read_index_node`），CPU 与 GPU 走读内核都红 I-2.4；
 /// core 读这条内部条目时拒（`allocation_record_tree` 读子指针那一处），GPU 恢复内核 `read_allocation_tree` 同拒。
-fn record_new_pool_with_an_extent_offset_on_an_allocation_record_tree_child_pointer(
+fn record_new_pool_with_a_non_first_version_head_on_an_allocation_record_tree_child_pointer(
     tag: &str,
+    field: PointerHeadFieldWrittenNonZero,
 ) -> RecordedFlow {
+    let name = match field {
+        PointerHeadFieldWrittenNonZero::ExtentOffset => {
+            "new-pool-allocation-record-tree-child-pointer-extent-offset-one"
+        }
+        PointerHeadFieldWrittenNonZero::FirstMacByte => {
+            "new-pool-allocation-record-tree-child-pointer-first-mac-byte-one"
+        }
+    };
     let allocation_internal_node = |bytes: &[u8]| {
         unit_class(bytes) == 2
             && usize::from(index_node_key_width(bytes)) == ALLOCATION_RECORD_TREE_KEY_BYTES
             && index_node_entry_width(bytes) == ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES
     };
-    record_new_pool_with_a_resealed_field(
-        tag,
-        "new-pool-allocation-record-tree-child-pointer-extent-offset-one",
-        &allocation_internal_node,
-        false,
-        &|bytes| {
-            let first_entry = index_node_entries_start(ALLOCATION_RECORD_TREE_KEY_BYTES);
-            set_the_extent_offset_of_the_pointer_at(
-                bytes,
-                first_entry + ALLOCATION_RECORD_TREE_KEY_BYTES,
-            );
-        },
-    )
+    record_new_pool_with_a_resealed_field(tag, name, &allocation_internal_node, false, &|bytes| {
+        let first_entry = index_node_entries_start(ALLOCATION_RECORD_TREE_KEY_BYTES);
+        write_the_pointer_head_field_non_zero_at(
+            bytes,
+            first_entry + ALLOCATION_RECORD_TREE_KEY_BYTES,
+            field,
+        );
+    })
 }
 
 /// 新池新建文件（txg 3）、第一个文件顺序写成三个数据单元（txg 4）：txg 4 那一版 extent 上段叶那一条指一个下段根（标签 1），
@@ -4387,47 +4430,279 @@ fn record_three_unit_file_with_a_resealed_field(
     flow
 }
 
-/// extent 下段根指针那一处：txg 4 那一版上段叶第一条（标签 1）指着的下段根指针头部 extent 偏移写 1、重封到根。池级 checker 读下段根这个码 2 节点时
-/// 判它（`read_index_node`），CPU 与 GPU 走读内核都红 I-2.4；core 解上段叶条目时拒（`ExtentUpperLeafEntry::parse`），GPU 恢复内核读上段叶那一处同拒。
-fn record_three_unit_file_with_an_extent_offset_on_the_lower_segment_root_pointer(
+/// extent 下段根指针那一处：txg 4 那一版上段叶第一条（标签 1）指着的下段根指针头部 `field` 那一段写成非第一版的取值、重封到根。
+/// 池级 checker 读下段根这个码 2 节点时判它（`read_index_node`），CPU 与 GPU 走读内核都红 I-2.4；core 解上段叶条目时拒
+/// （`ExtentUpperLeafEntry::parse`），GPU 恢复内核读上段叶那一处同拒。
+fn record_three_unit_file_with_a_non_first_version_head_on_the_lower_segment_root_pointer(
     tag: &str,
+    field: PointerHeadFieldWrittenNonZero,
 ) -> RecordedFlow {
-    record_three_unit_file_with_a_resealed_field(
-        tag,
-        "three-unit-file-lower-segment-root-pointer-extent-offset-one",
-        &is_an_extent_upper_leaf,
-        &|bytes| {
-            let pointer = pointer_of_the_first_extent_upper_leaf_entry(
-                bytes,
-                EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT,
-            );
-            set_the_extent_offset_of_the_pointer_at(bytes, pointer);
-        },
-    )
+    let name = match field {
+        PointerHeadFieldWrittenNonZero::ExtentOffset => {
+            "three-unit-file-lower-segment-root-pointer-extent-offset-one"
+        }
+        PointerHeadFieldWrittenNonZero::FirstMacByte => {
+            "three-unit-file-lower-segment-root-pointer-first-mac-byte-one"
+        }
+    };
+    record_three_unit_file_with_a_resealed_field(tag, name, &is_an_extent_upper_leaf, &|bytes| {
+        let pointer = pointer_of_the_first_extent_upper_leaf_entry(
+            bytes,
+            EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT,
+        );
+        write_the_pointer_head_field_non_zero_at(bytes, pointer, field);
+    })
 }
 
-/// extent 数据指针那一处（下段叶记录里的那条）：txg 4 那一版下段叶（码 2、key 宽 24、条目宽 112）第一条记录的数据指针头部 extent 偏移写 1、
-/// 重封到根。池级 checker 走数据指针时判它（`walk_extent_data_pointer`），CPU 与 GPU 走读内核都红 I-2.4；core 解这条叶记录时拒
+/// extent 数据指针那一处（下段叶记录里的那条）：txg 4 那一版下段叶（码 2、key 宽 24、条目宽 112）第一条记录的数据指针头部 `field` 那一段
+/// 写成非第一版的取值、重封到根。池级 checker 走数据指针时判它（`walk_extent_data_pointer`），CPU 与 GPU 走读内核都红 I-2.4；core 解这条叶记录时拒
 /// （`records::parse_extent_record`），GPU 恢复内核 `read_lower_segment` 读叶那一处同拒。
-fn record_three_unit_file_with_an_extent_offset_on_a_lower_leaf_data_pointer(
+fn record_three_unit_file_with_a_non_first_version_head_on_a_lower_leaf_data_pointer(
     tag: &str,
+    field: PointerHeadFieldWrittenNonZero,
 ) -> RecordedFlow {
+    let name = match field {
+        PointerHeadFieldWrittenNonZero::ExtentOffset => {
+            "three-unit-file-lower-leaf-data-pointer-extent-offset-one"
+        }
+        PointerHeadFieldWrittenNonZero::FirstMacByte => {
+            "three-unit-file-lower-leaf-data-pointer-first-mac-byte-one"
+        }
+    };
     let extent_lower_leaf = |bytes: &[u8]| {
         unit_class(bytes) == 2
             && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
             && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
     };
-    record_three_unit_file_with_a_resealed_field(
+    record_three_unit_file_with_a_resealed_field(tag, name, &extent_lower_leaf, &|bytes| {
+        let first_record = index_node_entries_start(EXTENT_KEY_BYTES);
+        write_the_pointer_head_field_non_zero_at(bytes, first_record + EXTENT_KEY_BYTES, field);
+    })
+}
+
+// ───────────── 第 30 批 30-2：容器树子指针 extent 偏移（E1）、恢复内核种类 ≥ 32 的重复、恢复内核指针头部加密压缩那几段 ─────────────
+
+/// 墓碑树与谱系树码 2 节点条目里子指针的偏移（分隔 key 8、身份引用 26 之后）与条目宽，key 宽是容器号 8 字节。
+const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET_IN_THE_ENTRY: usize = 34;
+const CONTAINER_TREE_ENTRY_BYTES: u16 = 120;
+const CONTAINER_NUMBER_KEY_BYTES: usize = 8;
+/// 码 2 节点条目里身份引用的类型段（分隔 key 8、出生树 8 之后的 2 字节）：指码 3 容器的条目写容器的记录类型。
+const CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET: usize = 16;
+/// 谱系容器的打包记录类型（`singlefs_core::unit::PACKED_TYPE_LINEAGE`）。
+const PACKED_RECORD_TYPE_LINEAGE: u16 = 5;
+
+/// 一个墓碑树或谱系树的码 2 根（层级 1、key 宽 8、条目宽 120），第一条条目指一片记录类型 `record_type` 的容器。
+/// inode 树根形状相同，靠第一条条目的类型段分开（inode 叶容器的类型是 2）。
+fn is_a_container_tree_root_over_containers_of_record_type(bytes: &[u8], record_type: u16) -> bool {
+    unit_class(bytes) == 2
+        && usize::from(index_node_key_width(bytes)) == CONTAINER_NUMBER_KEY_BYTES
+        && index_node_entry_width(bytes) == CONTAINER_TREE_ENTRY_BYTES
+        && read_u16_at(
+            bytes,
+            index_node_entries_start(CONTAINER_NUMBER_KEY_BYTES)
+                + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
+        ) == record_type
+}
+
+/// 容器树根第一条条目里指向容器的子指针头部 extent 偏移写 1。
+fn set_the_extent_offset_of_the_first_container_pointer(bytes: &mut [u8]) {
+    set_the_extent_offset_of_the_pointer_at(
+        bytes,
+        index_node_entries_start(CONTAINER_NUMBER_KEY_BYTES)
+            + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET_IN_THE_ENTRY,
+    );
+}
+
+/// E1 墓碑树那一处：删除中间态的流上墓碑树根第一条条目里指向墓碑容器的子指针头部 extent 偏移写 1、重封到根。
+/// 池级 checker 走墓碑树时判它（`walk_container_tree_root`），CPU 与 GPU 走读内核都红 I-2.4；core 读容器树的码 2 节点时拒这条子指针
+/// （`read_container_tree_index_nodes`），GPU 恢复内核 `read_container_tree_index_nodes` 同拒，都在找第一个文件那条记录之前。
+fn record_deletion_with_an_extent_offset_on_the_tombstone_container_pointer(
+    tag: &str,
+) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
         tag,
-        "three-unit-file-lower-leaf-data-pointer-extent-offset-one",
-        &extent_lower_leaf,
+        "deletion-tombstone-container-pointer-extent-offset-one",
         &|bytes| {
-            let first_record = index_node_entries_start(EXTENT_KEY_BYTES);
-            set_the_extent_offset_of_the_pointer_at(bytes, first_record + EXTENT_KEY_BYTES);
+            is_a_container_tree_root_over_containers_of_record_type(
+                bytes,
+                PACKED_RECORD_TYPE_TOMBSTONE,
+            )
         },
+        &set_the_extent_offset_of_the_first_container_pointer,
     )
 }
 
+/// E1 谱系树那一处：固定脚本到回退 D 的流上谱系树根第一条条目里指向谱系容器的子指针头部 extent 偏移写 1、重封到根。
+/// 池级 checker 走谱系树时判它，CPU 与 GPU 走读内核都红 I-2.4；core 读容器树的码 2 节点时拒这条子指针（`read_container_tree_index_nodes`），
+/// GPU 恢复内核 `read_container_tree_index_nodes` 同拒（`record_pointer_head_is_the_first_version` 连 extent 偏移一起判）。
+fn record_rollback_with_an_extent_offset_on_the_lineage_container_pointer(
+    tag: &str,
+) -> RecordedFlow {
+    record_rollback_with_a_resealed_field(
+        tag,
+        "rollback-lineage-container-pointer-extent-offset-one",
+        &|bytes| {
+            is_a_container_tree_root_over_containers_of_record_type(
+                bytes,
+                PACKED_RECORD_TYPE_LINEAGE,
+            )
+        },
+        &set_the_extent_offset_of_the_first_container_pointer,
+    )
+}
+
+/// 一个第一版写者写不出的种类码（≥ 32）：GPU 恢复内核的种类位图只装得下 0..31，这一档逐条与前面各条比。
+const TREE_KIND_NOT_BELOW_THIRTY_TWO: u16 = 40;
+
+/// I-9.16 ③ 种类 ≥ 32 那一档（判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L4 第二行）：发布那张树表里
+/// 稀疏旁表与 deadlist 两条（根指针都全零）的种类都改成 40——种类 40 两条、7 与 8 一条都没有。core 的只读恢复在读任何一棵树之前拒
+/// （`tree_table_entries_each_kind_at_most_once` 比整个 u16），GPU 恢复内核对种类 ≥ 32 的条目逐条与前面比、同拒。
+fn record_new_pool_with_two_tree_table_entries_of_a_kind_not_below_thirty_two(
+    tag: &str,
+) -> RecordedFlow {
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-tree-table-two-entries-of-kind-forty",
+        &is_a_tree_table_unit,
+        false,
+        &|bytes| {
+            for from_kind in [TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_DEADLIST] {
+                let entry = only_tree_table_entry_of_kind(bytes, from_kind);
+                let root_pointer = entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
+                assert_eq!(
+                    bytes[root_pointer..root_pointer + NODE_POINTER_BYTES],
+                    [0u8; NODE_POINTER_BYTES],
+                    "种类 {from_kind} 那条的根指针本来全零（这棵树第一版不建根，走读与恢复都不读它）"
+                );
+                let kind = entry + TREE_TABLE_ENTRY_KIND_OFFSET;
+                bytes[kind..kind + 2]
+                    .copy_from_slice(&TREE_KIND_NOT_BELOW_THIRTY_TWO.to_le_bytes());
+            }
+        },
+    )
+}
+
+/// E1 墓碑树：池级 checker 与恢复两半整份比，CPU 红 I-2.4。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_tombstone_container_pointer_carries_an_extent_offset()
+{
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_tombstone_container_pointer_carries_an_extent_offset",
+        &record_deletion_with_an_extent_offset_on_the_tombstone_container_pointer(
+            "crash-judge-gpu-i24-tombstone-container-pointer-extent-offset",
+        ),
+        "I-2.4",
+    );
+}
+
+/// E1 谱系树：池级 checker 与恢复两半整份比，CPU 红 I-2.4。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_pointer_carries_an_extent_offset() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_pointer_carries_an_extent_offset",
+        &record_rollback_with_an_extent_offset_on_the_lineage_container_pointer(
+            "crash-judge-gpu-i24-lineage-container-pointer-extent-offset",
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-9.16 ③：树表里种类 40 有两条（恢复那一半 core 拒、GPU 恢复内核要同拒）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_a_kind_not_below_thirty_two(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_a_kind_not_below_thirty_two",
+        &record_new_pool_with_two_tree_table_entries_of_a_kind_not_below_thirty_two(
+            "crash-judge-gpu-i916-kind-forty-twice",
+        ),
+        "I-9.16",
+    );
+}
+
+/// I-2.4：树表条目里 inode 树根指针的 MAC 第 0 字节非 0（core 解树表条目时拒、GPU 恢复内核同拒）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_root_pointer_carries_a_non_zero_mac_byte() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_root_pointer_carries_a_non_zero_mac_byte",
+        &record_new_pool_with_a_non_first_version_head_on_the_inode_tree_root_pointer(
+            "crash-judge-gpu-i24-tree-table-root-pointer-mac",
+            PointerHeadFieldWrittenNonZero::FirstMacByte,
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-2.4：inode 内部条目子指针的 MAC 第 0 字节非 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_a_non_zero_mac_byte(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_a_non_zero_mac_byte",
+        &record_new_pool_with_a_non_first_version_head_on_an_inode_internal_entry_child_pointer(
+            "crash-judge-gpu-i24-inode-child-pointer-mac",
+            PointerHeadFieldWrittenNonZero::FirstMacByte,
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-2.4：extent 上段叶内联的数据指针的 MAC 第 0 字节非 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_a_non_zero_mac_byte(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_a_non_zero_mac_byte",
+        &record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
+            "crash-judge-gpu-i24-inline-data-pointer-mac",
+            PointerHeadFieldWrittenNonZero::FirstMacByte,
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-2.4：分配记录树内部条目子指针的 MAC 第 0 字节非 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_a_non_zero_mac_byte(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_a_non_zero_mac_byte",
+        &record_new_pool_with_a_non_first_version_head_on_an_allocation_record_tree_child_pointer(
+            "crash-judge-gpu-i24-allocation-child-pointer-mac",
+            PointerHeadFieldWrittenNonZero::FirstMacByte,
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-2.4：extent 上段叶指着的下段根指针的 MAC 第 0 字节非 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_a_non_zero_mac_byte(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_a_non_zero_mac_byte",
+        &record_three_unit_file_with_a_non_first_version_head_on_the_lower_segment_root_pointer(
+            "crash-judge-gpu-i24-lower-segment-root-pointer-mac",
+            PointerHeadFieldWrittenNonZero::FirstMacByte,
+        ),
+        "I-2.4",
+    );
+}
+
+/// I-2.4：extent 下段叶记录的数据指针的 MAC 第 0 字节非 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_a_non_zero_mac_byte(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_a_non_zero_mac_byte",
+        &record_three_unit_file_with_a_non_first_version_head_on_a_lower_leaf_data_pointer(
+            "crash-judge-gpu-i24-lower-leaf-data-pointer-mac",
+            PointerHeadFieldWrittenNonZero::FirstMacByte,
+        ),
+        "I-2.4",
+    );
+}
+
 /// I-9.16 ③：树表里一种种类有两条。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_one_kind() {
@@ -4482,8 +4757,9 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_all
 fn the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset() {
     assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
         "the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset",
-        &record_new_pool_with_an_extent_offset_on_the_inode_tree_root_pointer(
+        &record_new_pool_with_a_non_first_version_head_on_the_inode_tree_root_pointer(
             "crash-judge-gpu-i24-node-pointer-extent-offset",
+            PointerHeadFieldWrittenNonZero::ExtentOffset,
         ),
         "I-2.4",
     );
@@ -4508,8 +4784,9 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_
 ) {
     assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
         "the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_an_extent_offset",
-        &record_new_pool_with_an_extent_offset_on_an_inode_internal_entry_child_pointer(
+        &record_new_pool_with_a_non_first_version_head_on_an_inode_internal_entry_child_pointer(
             "crash-judge-gpu-i24-inode-child-pointer-extent-offset",
+            PointerHeadFieldWrittenNonZero::ExtentOffset,
         ),
         "I-2.4",
     );
@@ -4520,8 +4797,9 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_
 fn the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset() {
     assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
         "the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset",
-        &record_new_pool_with_an_extent_offset_on_the_inline_data_pointer(
+        &record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
             "crash-judge-gpu-i24-inline-data-pointer-extent-offset",
+            PointerHeadFieldWrittenNonZero::ExtentOffset,
         ),
         "I-2.4",
     );
@@ -4533,8 +4811,9 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointe
 ) {
     assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
         "the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_an_extent_offset",
-        &record_new_pool_with_an_extent_offset_on_an_allocation_record_tree_child_pointer(
+        &record_new_pool_with_a_non_first_version_head_on_an_allocation_record_tree_child_pointer(
             "crash-judge-gpu-i24-allocation-child-pointer-extent-offset",
+            PointerHeadFieldWrittenNonZero::ExtentOffset,
         ),
         "I-2.4",
     );
@@ -4546,8 +4825,9 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_c
 ) {
     assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
         "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_an_extent_offset",
-        &record_three_unit_file_with_an_extent_offset_on_the_lower_segment_root_pointer(
+        &record_three_unit_file_with_a_non_first_version_head_on_the_lower_segment_root_pointer(
             "crash-judge-gpu-i24-lower-segment-root-pointer-extent-offset",
+            PointerHeadFieldWrittenNonZero::ExtentOffset,
         ),
         "I-2.4",
     );
@@ -4559,8 +4839,9 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carr
 ) {
     assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
         "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_an_extent_offset",
-        &record_three_unit_file_with_an_extent_offset_on_a_lower_leaf_data_pointer(
+        &record_three_unit_file_with_a_non_first_version_head_on_a_lower_leaf_data_pointer(
             "crash-judge-gpu-i24-lower-leaf-data-pointer-extent-offset",
+            PointerHeadFieldWrittenNonZero::ExtentOffset,
         ),
         "I-2.4",
     );
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index caf128f..56d7882 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -8469,6 +8469,139 @@ fn an_allocation_record_tree_that_does_not_read_back_leaves_the_disjointness_inv
     );
 }
 
+/// I-5.4 按版本（判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2「I-5.4『成立要至少一棵树走到叶』」那一行的零轮收严，
+/// 正推探针 `research/prompts/m3-formal-code-b29-r1-sonnet-model/probes/b29_l4_append.rs` 的 `b29_l2_i54_after_zeroing_each_unit_of_the_second_version`）：
+/// 覆盖写之后（发布 B）把 B 那一版的分配记录树根两份清零——A 那一版的分配记录树照样走到叶，B 那一版自己的走不到；
+/// B 带树、根记录里分配记录树根全零那一格成立而不计进评估，I-5.4 报部分没判、点名 B（改之前 A 的树走到了叶就替 B 计成立，报成立）。
+#[test]
+fn an_allocation_record_tree_of_the_newest_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged_while_the_previous_version_reads_back(
+) {
+    let clean = image_after_the_overwrite("known-bad-disjointness-newest-allocation-root-zeroed");
+    assert_eq!(
+        verdict(&clean, "I-5.4"),
+        InvariantVerdict::Holds,
+        "覆盖写之后的干净镜像上 A、B 两版的分配记录树都走到叶，I-5.4 成立"
+    );
+    let allocation_root_of_the_newest_version = common_pointer_head_corruption::the_slot_of(
+        &common_pointer_head_corruption::tree_table_entry_of_kind(&clean, TREE_KIND_ALLOCATION)
+            .root,
+    );
+    assert_ne!(
+        allocation_root_of_the_newest_version.0, ALLOCATION_RECORD_TREE_ROOT,
+        "B 写出了自己的分配记录树根，不与 A 共用"
+    );
+    let mut image = clean.clone();
+    zero_both_copies_of_the_unit(
+        &mut image,
+        allocation_root_of_the_newest_version.0,
+        usize::try_from(NODE_BYTES).expect("16384"),
+    );
+    assert_eq!(
+        verdict(&image, "I-5.4"),
+        InvariantVerdict::PartiallyJudged(
+            "带树一版根记录里分配记录树根全零那一格成立而没计（那一版自己的分配记录树读不出、走不到叶，别的版本的树走到了叶不替它作保）：根（实例 1、txg 4）"
+                .to_owned()
+        ),
+        "B 那一版的分配记录树走不到叶：A 的树走到了叶不替 B 计成立，I-5.4 报部分没判"
+    );
+}
+
+/// 树表里种类 `kind` 那棵容器树（墓碑树、谱系树）的码 2 根第 0 条条目里指向容器的子指针（条目里分隔 key 8 + 身份引用 26 之后那 86 字节）：
+/// 头部 extent 偏移 0 → 1，根重封、新的整单元校验和沿引用链补到根槽
+/// （攻方 `research/prompts/m3-formal-code-b29-r1-opus-model/tests/b29_opus_tombstone_container_pointer_head.rs` 的造法）。
+fn set_the_extent_offset_of_the_first_container_pointer_in_the_container_tree_root_of_kind(
+    image: &mut MemoryPool,
+    kind: u16,
+) {
+    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
+    let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
+    let node_length = common_pointer_head_corruption::node_bytes();
+    let root = singlefs_core::unit::parse_index_node(
+        &common_pointer_head_corruption::unit_bytes_at(image, root_slot, node_length),
+    )
+    .expect("容器树根解得开");
+    assert_eq!(root.level, 1, "种类 {kind} 那棵容器树的根直接管容器");
+    let first_entry = root.entries.first().expect("根里至少一条条目");
+    let child_pointer = first_entry[CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+        ..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES]
+        .to_vec();
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        image,
+        root_slot,
+        node_length,
+        |unit| {
+            let occurrences: Vec<usize> = (0..=unit.len() - child_pointer.len())
+                .filter(|start| unit[*start..*start + child_pointer.len()] == child_pointer[..])
+                .collect();
+            assert_eq!(occurrences.len(), 1, "根里这条子指针恰好出现一次");
+            let field = occurrences[0] + POINTER_EXTENT_OFFSET_FIELD_OFFSET;
+            assert_eq!(
+                unit[field..field + 2],
+                [0, 0],
+                "第一版指针的 extent 偏移恒 0"
+            );
+            unit[field..field + 2].copy_from_slice(&1u16.to_le_bytes());
+        },
+    );
+}
+
+/// 墓碑树、谱系树码 2 节点条目里子指针的偏移与宽（分隔 key 8 + 身份引用 26 之后的 86 字节码 2 / 码 3 指针）。
+const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: usize = 34;
+const CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES: usize = 86;
+
+/// E1（判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L4 第一行）：unlink 意图之后，墓碑树码 2 根里指向墓碑容器的
+/// 子指针头部 extent 偏移改 1。走读跟随这条指针时按 I-2.4 判 extent 偏移（`.claude/kb/invariants.md` I-2.4 那一行），只红 I-2.4
+/// （改之前 `walk_container_tree_root` 只判 MAC / nonce，I-2.4 报成立、一条都不红）。
+#[test]
+fn an_extent_offset_on_the_container_pointer_of_the_tombstone_tree_root_reddens_only_the_pointer_head_invariant(
+) {
+    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-container-pointer-extent-offset",
+    );
+    assert_eq!(
+        verdict(&clean, "I-2.4"),
+        InvariantVerdict::Holds,
+        "unlink 意图之后的干净镜像上 I-2.4 成立"
+    );
+    let mut image = clean.clone();
+    set_the_extent_offset_of_the_first_container_pointer_in_the_container_tree_root_of_kind(
+        &mut image,
+        TREE_KIND_TOMBSTONE,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-2.4"],
+        "墓碑树根里指向容器的子指针 extent 偏移非 0：只红 I-2.4：{verdicts:?}"
+    );
+}
+
+/// E1 谱系树那一半：回退到 A 之后（回退那次发布追加一条谱系记录、写出谱系树根与一片谱系容器），谱系树码 2 根里指向谱系容器的子指针
+/// 头部 extent 偏移改 1，只红 I-2.4（墓碑树、谱系树走同一个 `walk_container_tree_root`）。
+#[test]
+fn an_extent_offset_on_the_container_pointer_of_the_lineage_tree_root_reddens_only_the_pointer_head_invariant(
+) {
+    let rolled_back = image_after_rolling_back_to_the_first_root(
+        "known-bad-lineage-container-pointer-extent-offset",
+    );
+    assert_eq!(
+        verdict(&rolled_back.image, "I-2.4"),
+        InvariantVerdict::Holds,
+        "回退到 A 之后的干净镜像上 I-2.4 成立"
+    );
+    let mut image = rolled_back.image.clone();
+    set_the_extent_offset_of_the_first_container_pointer_in_the_container_tree_root_of_kind(
+        &mut image,
+        TREE_KIND_LINEAGE,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-2.4"],
+        "谱系树根里指向容器的子指针 extent 偏移非 0：只红 I-2.4：{verdicts:?}"
+    );
+}
+
 /// I-5.5（攻方 d1）：第一个文件的数据指针第二条位置条目从盘 1 改到盘 0——盘 1 上一个数据单元都没有，该盘跳过、不记，
 /// 盘 0 上照判、成立；位置条目不按设备升序，I-2.5 照旧红。
 #[test]
````

### mutations-append.tsv

````text
第 30 批 30-2 池级 checker：墓碑树容器子指针不判 extent 偏移（E1）	crates/singlefs-checker/src/walk.rs	            judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &child, &what);\n            judge_pointer_extent_offset_is_zero(&mut self.judgements, &child, &what);	            judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &child, &what);	-p singlefs-harness --test checker_known_bad_images -- an_extent_offset_on_the_container_pointer_of_the_tombstone_tree_root_reddens_only_the_pointer_head_invariant	an_extent_offset_on_the_container_pointer_of_the_tombstone_tree_root_reddens_only_the_pointer_head_invariant
第 30 批 30-2 池级 checker：谱系树容器子指针不判 extent 偏移（E1）	crates/singlefs-checker/src/walk.rs	            judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &child, &what);\n            judge_pointer_extent_offset_is_zero(&mut self.judgements, &child, &what);	            judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &child, &what);	-p singlefs-harness --test checker_known_bad_images -- an_extent_offset_on_the_container_pointer_of_the_lineage_tree_root_reddens_only_the_pointer_head_invariant	an_extent_offset_on_the_container_pointer_of_the_lineage_tree_root_reddens_only_the_pointer_head_invariant
第 30 批 30-2 池级 checker：I-5.4 带树一版根记录那一格改回任一版的分配记录树走到叶就计成立（L2 零轮收严撤回）	crates/singlefs-checker/src/walk.rs	        if *this_version_allocation_record_tree_walked_to_its_leaves {	        if some_allocation_record_tree_walked_to_its_leaves {	-p singlefs-harness --test checker_known_bad_images -- an_allocation_record_tree_of_the_newest_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged_while_the_previous_version_reads_back	an_allocation_record_tree_of_the_newest_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged_while_the_previous_version_reads_back
第 30 批 30-2 GPU 判器：走读内核墓碑树容器子指针不判 extent 偏移（E1）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_judge_pointer_head(child);\n        w_judge_pointer_extent_offset(child);\n        if (entry_type == 0u) {	        w_judge_pointer_head(child);\n        if (entry_type == 0u) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_tombstone_container_pointer_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_the_tombstone_container_pointer_carries_an_extent_offset
第 30 批 30-2 GPU 判器：走读内核谱系树容器子指针不判 extent 偏移（E1）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_judge_pointer_head(child);\n        w_judge_pointer_extent_offset(child);\n        if (entry_type == 0u) {	        w_judge_pointer_head(child);\n        if (entry_type == 0u) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_pointer_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_pointer_carries_an_extent_offset
第 30 批 30-2 GPU 判器：恢复内核树表种类 ≥ 32 的重复拦不到	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	                if (rd16(tree_table.base, entry_offset(tree_table, earlier) + 10u) == kind) {	                if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_a_kind_not_below_thirty_two	the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_a_kind_not_below_thirty_two
第 30 批 30-2 GPU 判器：恢复内核解树表条目不判树根指针的加密压缩那几段	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        if (!pointer_encryption_and_compression_fields_are_the_first_version(tree_table.base, o + 14u)) {	        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_root_pointer_carries_a_non_zero_mac_byte	the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_root_pointer_carries_a_non_zero_mac_byte
第 30 批 30-2 GPU 判器：恢复内核读 inode 内部条目不判子指针的加密压缩那几段	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        if (!child.head_ok) {\n            return walk_failed(66u);	        if (false) {\n            return walk_failed(66u);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_a_non_zero_mac_byte	the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_a_non_zero_mac_byte
第 30 批 30-2 GPU 判器：恢复内核读 extent 上段叶不判内联数据指针的加密压缩那几段	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	                        // 同上：内联的数据指针（加密、压缩那几段与 extent 偏移各判一次）\n                        if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 25u)) {	                        // 同上：内联的数据指针（加密、压缩那几段与 extent 偏移各判一次）\n                        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_a_non_zero_mac_byte	the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_a_non_zero_mac_byte
第 30 批 30-2 GPU 判器：恢复内核读分配记录树不判内部条目子指针的加密压缩那几段	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        if (!pending_pointer.head_ok) {	        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_a_non_zero_mac_byte	the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_a_non_zero_mac_byte
第 30 批 30-2 GPU 判器：恢复内核读 extent 上段叶不判下段根指针的加密压缩那几段	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	                        // ExtentUpperLeafEntry::parse：下段根指针头部加密、压缩那几段不是第一版的取值，这条条目损坏\n                        if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 25u)) {	                        // ExtentUpperLeafEntry::parse：下段根指针头部加密、压缩那几段不是第一版的取值，这条条目损坏\n                        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_a_non_zero_mac_byte	the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_a_non_zero_mac_byte
第 30 批 30-2 GPU 判器：恢复内核读 extent 下段叶不判数据指针的加密压缩那几段	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	                    if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 24u)) {	                    if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_a_non_zero_mac_byte	the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_a_non_zero_mac_byte
````

### mutations-replacements.tsv

````text
第28批 I-5.4 带树一版根记录那一格不等分配记录树走到叶就计进评估（正推 C2）	crates/singlefs-checker/src/walk.rs	        if *this_version_allocation_record_tree_walked_to_its_leaves {	        if true {	-p singlefs-harness --test checker_known_bad_images -- an_allocation_record_tree_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged	an_allocation_record_tree_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
````

## m3-formal-30-3

### crates.patch

````diff
diff --git a/crates/singlefs-core/src/system_configuration.rs b/crates/singlefs-core/src/system_configuration.rs
index 08c2ea8..33bcb38 100644
--- a/crates/singlefs-core/src/system_configuration.rs
+++ b/crates/singlefs-core/src/system_configuration.rs
@@ -81,8 +81,9 @@ pub const ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION: SlotNumber = SlotNumber(ROOT
 /// D2（RAID 条带策略） 已定项 18：第一版 w_max 与 g 都写 4。
 const MAXIMUM_STRIPE_WIDTH: u8 = 4;
 const GROUP_SIZE: u8 = 4;
-/// D16（发布语义） 已定项 5 的两个可调值。
-const TIME_THRESHOLD_SECONDS: u32 = 5;
+/// D16（发布语义） 已定项 5 的两个可调值。T_time 盘上那 4 字节的单位是毫秒（用户 2026-10-05 定：已定项 5 的可设区间
+/// [200 ms, 5 s] 下端在「秒」里写不出；判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2），默认 5 s 写 5000。
+const TIME_THRESHOLD_MILLISECONDS: u32 = 5000;
 const DIRTY_THRESHOLD_BYTES: u64 = 2 << 30;
 /// 整理低水位 / 高水位 / 停止线三条（D26（后台整理与放置回收） 已定项 1）。
 const COMPACTION_WATERMARK_COUNT: usize = 3;
@@ -285,10 +286,10 @@ impl SystemRuntimeConfiguration {
     /// 这一档在字段表里占的字节数：4 + 8 + 24。
     pub const FIELD_TABLE_BYTES: u64 = 36;
 
-    /// T_time（D16（发布语义） 已定项 5）。
+    /// T_time（D16（发布语义） 已定项 5），单位毫秒：盘上偏移 433 那 4 字节照这个单位写。
     #[must_use]
-    pub fn time_threshold_seconds(self) -> u32 {
-        TIME_THRESHOLD_SECONDS
+    pub fn time_threshold_milliseconds(self) -> u32 {
+        TIME_THRESHOLD_MILLISECONDS
     }
 
     /// T_dirty（D16（发布语义） 已定项 5）；挂载时的有效值另按环长夹取，那不改盘上这 8 字节。
@@ -519,7 +520,7 @@ impl SystemConfiguration {
             writer.put_u32(self.immutable.sizes.fixed_structure_slot_spacing);
         });
         slot.write(SlotFieldMutability::RuntimeConfiguration, |writer| {
-            writer.put_u32(self.runtime.time_threshold_seconds());
+            writer.put_u32(self.runtime.time_threshold_milliseconds());
             writer.put_u64(self.runtime.dirty_threshold_bytes());
             for watermark in self.runtime.compaction_watermarks() {
                 writer.put_u64(watermark);
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index 391f1d3..4b9d90e 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -1351,10 +1351,12 @@ fn byte_ranges_that_do_not_hold<'table>(
         .collect()
 }
 
-/// T_time 的内置默认值，单位秒（D16（发布语义） 已定项 5「`T_time` 默认值 5 s」；`.claude/kb/layout/01-first-txn.md` 系统配置字段表
-/// 「可调值 | T_time | 4 | 5 秒」那一行）。照条款写死，不取被测 core 的常量：期望取自被测者时，core 把默认值改错，挂载写槽写的与期望一起错，
-/// 这一行判不出（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「运行配置『R2 = 内置默认』」那一行，改法 G1）。
-const BUILT_IN_TIME_THRESHOLD_SECONDS_OF_THE_CLAUSES: u32 = 5;
+/// T_time 的内置默认值，单位毫秒：5 s 写 5000（D16（发布语义） 已定项 5「`T_time` 默认值 5 s」、可设区间 [200 ms, 5 s]；
+/// 盘上编码单位毫秒是用户 2026-10-05 定的，判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2，
+/// `.claude/kb/layout/01-first-txn.md` 系统配置字段表「可调值 | T_time | 4」那一行由写回批写明单位毫秒、默认值 5000）。照条款写死，不取被测 core 的常量：
+/// 期望取自被测者时，core 把默认值或单位改错，挂载写槽写的与期望一起错，这一行判不出
+/// （判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「运行配置『R2 = 内置默认』」那一行，改法 G1）。
+const BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES: u32 = 5000;
 /// T_dirty 的内置默认值，单位字节：2 GiB（D16（发布语义） 已定项 5；字段表「可调值 | T_dirty | 8 | 2 GiB」那一行。挂载时的有效值另按环长夹取，
 /// 不改盘上这 8 字节）。
 const BUILT_IN_DIRTY_THRESHOLD_BYTES_OF_THE_CLAUSES: u64 = 2 * 1024 * 1024 * 1024;
@@ -1363,11 +1365,11 @@ const BUILT_IN_DIRTY_THRESHOLD_BYTES_OF_THE_CLAUSES: u64 = 2 * 1024 * 1024 * 102
 const BUILT_IN_COMPACTION_WATERMARKS_OF_THE_CLAUSES: [u64; 3] = [0, 0, 0];
 
 /// 系统运行配置那 36 字节的内置默认值：照字段表的次序（T_time 4、T_dirty 8、整理三水位 8 × 3，小端），值照条款写死
-/// （[`BUILT_IN_TIME_THRESHOLD_SECONDS_OF_THE_CLAUSES`] 那三个常量；今天没有挂载选项通道，写槽总写内置默认）。
+/// （[`BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES`] 那三个常量；今天没有挂载选项通道，写槽总写内置默认）。
 #[must_use]
 pub fn built_in_runtime_configuration_bytes() -> Vec<u8> {
     let mut bytes = Vec::with_capacity(SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION.len());
-    bytes.extend_from_slice(&BUILT_IN_TIME_THRESHOLD_SECONDS_OF_THE_CLAUSES.to_le_bytes());
+    bytes.extend_from_slice(&BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES.to_le_bytes());
     bytes.extend_from_slice(&BUILT_IN_DIRTY_THRESHOLD_BYTES_OF_THE_CLAUSES.to_le_bytes());
     for watermark in BUILT_IN_COMPACTION_WATERMARKS_OF_THE_CLAUSES {
         bytes.extend_from_slice(&watermark.to_le_bytes());
diff --git a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
index 22fce8f..c01b59f 100644
--- a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
+++ b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
@@ -1637,10 +1637,10 @@ fn a_journal_tail_below_the_highest_record_counter_in_the_ring_after_a_mount_and
 }
 
 /// 系统运行配置那一行「R2 = 内置默认」（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「运行配置『R2 = 内置默认』」
-/// 那一行，改法 G1）：带文件的一版上只挂载，判官拿 R2 择到的系统配置 433..469 那 36 字节比照条款写死的内置默认（T_time 5 秒、T_dirty 2 GiB、
+/// 那一行，改法 G1）：带文件的一版上只挂载，判官拿 R2 择到的系统配置 433..469 那 36 字节比照条款写死的内置默认（T_time 5000 毫秒即 5 s、T_dirty 2 GiB、
 /// 整理三水位各 0，D16（发布语义） 已定项 5；`.claude/kb/layout/01-first-txn.md` 系统配置字段表「可调值」那三行），这一样判绿。
-/// 判别力：core 的 T_time 内置默认改成 4（攻方 MUT-A，`crates/mutations.tsv` 一行）时这一条红——判官的期望取自被测 core 时，
-/// 那一改坏下挂载透明 45 条全绿（同一判决同一行）。
+/// 判别力：core 的 T_time 内置默认 5000 改成 4000（攻方 MUT-A，`crates/mutations.tsv` 一行）、或单位退回秒写成 5（第 30 批 30-3 那一行）时
+/// 这一条红——判官的期望取自被测 core 时，那一改坏下挂载透明 45 条全绿（同一判决同一行）。
 #[test]
 fn the_runtime_configuration_after_a_mount_alone_is_the_built_in_default_written_in_the_clauses() {
     let scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
diff --git a/crates/singlefs-harness/tests/system_configuration_mutability_classes.rs b/crates/singlefs-harness/tests/system_configuration_mutability_classes.rs
index 76b4e6b..005ddf0 100644
--- a/crates/singlefs-harness/tests/system_configuration_mutability_classes.rs
+++ b/crates/singlefs-harness/tests/system_configuration_mutability_classes.rs
@@ -86,10 +86,13 @@ fn system_configuration_after_the_new_pool_file_creation() -> SystemConfiguratio
 /// （字段表末尾加回退下界 F 8 字节、incompat 布局身份从位 0 换到位 1，D22 已定项 9、D15 已定项 4）之后按新代码重打，
 /// 与旧字节逐字节比过只差 6 个字节：偏移 6（incompat 位图第一个字节 0x01 → 0x02）与 [155, 159)（整槽校验和的 CRC-32C 那 4 字节）；
 /// F 在这两份样本里是 0，落在原来就是补齐 0 的 [481, 489)，一个字节都没变。
+/// 2026-10-05 T_time 的盘上单位从秒改成毫秒（用户定案，判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2）：
+/// 偏移 [433, 437) 从 5 换成 5000，连带整槽校验和；两份摘要从 `a065c124…`、`c67464f9…` 换成下面这两个（第 30 批 30-3 实现员副本上
+/// 这条断言红出来的 left 值；换之前同一份副本只把 core 那一处退回秒，这一条绿，差异只来自这一改）。
 const SLOT_DIGEST_AT_MKFS: &str =
-    "a065c12485a9ff72117d549852461f5cb4529d664ac2384271f6e0f22d0f167f";
+    "e5cc28abb94e3e1739b85dc7abc2191a1b120908b72a5ab3cb03588feb01c28f";
 const SLOT_DIGEST_AFTER_THE_NEW_POOL_FILE_CREATION: &str =
-    "c67464f93c9021cf354d649f37044041e4b5eec252f23d103a7e8c1328796260";
+    "144c1785dd57555c8539963909817a597c5a32d2d4b8fc3e61f7a8f6233f299e";
 
 #[test]
 fn splitting_the_system_configuration_into_four_mutability_classes_changes_no_byte_on_disk() {
diff --git a/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs b/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
index 58c6cac..310c415 100644
--- a/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
+++ b/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
@@ -308,7 +308,10 @@ fn crc32_of_the_images(devices: &[(DeviceIdentity, SparseBlockDevice)], device_b
 /// （暖机两条、第一个文件一条）的字节：新根段补上实例表指针、分配记录树根指针、卸载记号，fsid 挪到 [463, 471)，点名项整体后挪 176 字节，
 /// 连带 header_csum 与反向链；系统配置槽、根环、单元区逐字节不变（7-2 之前与之后两份副本上同一段流程逐扇区比过，差异只落在 journal 环
 /// 第 0–2 个记录槽里）。钉值随之换成 4048568891（7-2 合入之后的副本上同一段流程现算，与合入验证里这条断言红出来的 left 值相同）。
-const DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32: u32 = 4_048_568_891;
+/// 2026-10-05 T_time 的盘上单位从秒改成毫秒（用户定案，判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2）：
+/// 两块盘每个系统配置槽偏移 [433, 437) 从 5 换成 5000，连带整槽校验和；钉值随之换成 1329175690（第 30 批 30-3 实现员副本上这条断言
+/// 红出来的 left 值；同一份副本只把 core 那一处退回秒时这一条绿，差异只来自这一改）。
+const DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32: u32 = 1_329_175_690;
 
 /// 默认环 768 MiB：mkfs 按环长现算的单元区起点就是 50176（`INSTANCE_TABLE_SLOT`），树表第 0 版在 50178（`TREE_TABLE_GENESIS_SLOT`），
 /// 系统配置偏移 417 的 8 字节写 50176；mkfs 到第一个文件写出的两块盘与钉值那一版（钉值说明里最后一次换钉的那一版）逐字节相同（整盘 CRC-32C 钉值）。
@@ -343,7 +346,7 @@ fn the_default_journal_ring_keeps_the_unit_area_at_slot_50176_and_the_new_pool_f
     assert_eq!(
         crc32_of_the_images(&pool.devices, pool.device_bytes),
         DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32,
-        "默认环下 mkfs 到第一个文件写出的两块盘与钉值那一版（7-2 合入之后）逐字节相同"
+        "默认环下 mkfs 到第一个文件写出的两块盘与钉值那一版（T_time 改毫秒之后）逐字节相同"
     );
 }
 
````

### mutations-append.tsv

````text
第30批 30-3 T_time 盘上单位退回秒：core 的内置默认 5000 毫秒写回 5（挂载透明「R2 = 内置默认」的期望照条款写成毫秒才抓得到）	crates/singlefs-core/src/system_configuration.rs	const TIME_THRESHOLD_MILLISECONDS: u32 = 5000;	const TIME_THRESHOLD_MILLISECONDS: u32 = 5;	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount -- the_runtime_configuration_after_a_mount_alone_is_the_built_in_default_written_in_the_clauses	the_runtime_configuration_after_a_mount_alone_is_the_built_in_default_written_in_the_clauses
````

### mutations-replacements.tsv

````text
第28批 G1 攻方 MUT-A：core 的 T_time 内置默认 5 改 4（挂载透明「R2 = 内置默认」的期望照条款写死才抓得到）	crates/singlefs-core/src/system_configuration.rs	const TIME_THRESHOLD_MILLISECONDS: u32 = 5000;	const TIME_THRESHOLD_MILLISECONDS: u32 = 4000;	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount -- the_runtime_configuration_after_a_mount_alone_is_the_built_in_default_written_in_the_clauses	the_runtime_configuration_after_a_mount_alone_is_the_built_in_default_written_in_the_clauses
````

