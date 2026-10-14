# 删除线第 62 批改动全文（2026-10-11）

## m3-batch62/planned-import-pins：crates.patch

````diff
--- a/crates/singlefs-checker-tier/src/verdict_store.rs
+++ b/crates/singlefs-checker-tier/src/verdict_store.rs
@@ -1774,6 +1774,20 @@
         other: &VerdictStore,
         planned: &PlannedImportKeys,
     ) -> Result<PlannedImportTally, VerdictStoreError> {
+        self.import_the_planned_rows_in_batches_of(other, planned, PLANNED_IMPORT_BATCH_BYTES)
+    }
+
+    /// 同 [`Self::import_the_planned_rows_from`]，一批攒到 `batch_size_in_bytes` 字节就同步写一次。产品只经那一个入口、批长取
+    /// [`PLANNED_IMPORT_BATCH_BYTES`]；这里的批长单元测试给小的，在几百字节的库上也走多批（换批、跨批的编号接续）。
+    ///
+    /// # Errors
+    /// 同 [`Self::import_the_planned_rows_from`]。
+    fn import_the_planned_rows_in_batches_of(
+        &mut self,
+        other: &VerdictStore,
+        planned: &PlannedImportKeys,
+        batch_size_in_bytes: usize,
+    ) -> Result<PlannedImportTally, VerdictStoreError> {
         if !planned.conclusions.is_empty() {
             self.refuse_unless_the_conclusion_kinds_layout()?;
         }
@@ -1825,9 +1839,10 @@
                         &requested,
                     );
                     tally.conclusion_blocks_written += 1;
-                    if let Some(rows) =
-                        self.write_the_planned_import_batch_when_full(&mut pending)?
-                    {
+                    if let Some(rows) = self.write_the_planned_import_batch_when_full(
+                        &mut pending,
+                        batch_size_in_bytes,
+                    )? {
                         tally.count_a_written_batch(rows);
                     }
                 }
@@ -1885,7 +1900,9 @@
             }
             pending.batch.put(&key_bytes.0, &value_bytes);
             tally.verifier_blocks_written += 1;
-            if let Some(rows) = self.write_the_planned_import_batch_when_full(&mut pending)? {
+            if let Some(rows) =
+                self.write_the_planned_import_batch_when_full(&mut pending, batch_size_in_bytes)?
+            {
                 tally.count_a_written_batch(rows);
             }
         }
@@ -1901,7 +1918,9 @@
                 &facts_bytes,
             );
             tally.flow_facts_written += 1;
-            if let Some(rows) = self.write_the_planned_import_batch_when_full(&mut pending)? {
+            if let Some(rows) =
+                self.write_the_planned_import_batch_when_full(&mut pending, batch_size_in_bytes)?
+            {
                 tally.count_a_written_batch(rows);
             }
         }
@@ -1911,15 +1930,17 @@
         Ok(tally)
     }
 
-    /// 攒着的那一批到了 [`PLANNED_IMPORT_BATCH_BYTES`] 字节就同步写下去、换一批空的接着攒；交回写下的行数，没到交回 None。
+    /// 攒着的那一批到了 `batch_size_in_bytes` 字节（产品取 [`PLANNED_IMPORT_BATCH_BYTES`]）就同步写下去、换一批空的接着攒；
+    /// 交回写下的行数，没到交回 None。
     ///
     /// # Errors
     /// RocksDB 写失败。
     fn write_the_planned_import_batch_when_full(
         &mut self,
         pending: &mut PlannedImportBatch,
+        batch_size_in_bytes: usize,
     ) -> Result<Option<u64>, VerdictStoreError> {
-        if pending.batch.size_in_bytes() < PLANNED_IMPORT_BATCH_BYTES {
+        if pending.batch.size_in_bytes() < batch_size_in_bytes {
             return Ok(None);
         }
         let full_batch = std::mem::replace(pending, PlannedImportBatch::empty());
@@ -4095,6 +4116,194 @@
         );
     }
 
+    /// 一批攒到一个字节就同步写：放完一块一种结论（或一个核对行、一份事实）就换一批，几百字节的库也走多批。
+    const ONE_BYTE_BATCH_WRITES_AFTER_EVERY_PUT_GROUP: usize = 1;
+
+    /// 对方库 B 再加流 F 第 3 块的 GPU 结论，用到又一种 A 没登记过的判定向量：按计划导入时 F2、F3 的 GPU 结论各带一种新向量，
+    /// 一字节一批时落在两批里（第二种的编号要接在上一批登记的那一种后面）。
+    fn build_the_other_library_b_with_a_second_new_verdict_vector(
+        directory: &Path,
+    ) -> VerdictStore {
+        let mut store = build_the_other_library_b(directory);
+        let clean = store
+            .register_verdict_vector(&verdict_vector(b""))
+            .expect("登记");
+        let red_of_block_three = store
+            .register_verdict_vector(&verdict_vector(b"red-f-block-three-on-the-other-host"))
+            .expect("登记");
+        store_conclusion_in_the_test_library(
+            &mut store,
+            &flow_f_block(3),
+            &ConclusionKind::GpuJudge,
+            &GPU_CONCLUSION_VERSION,
+            &conclusion_record(block_of(&[red_of_block_three, clean]), &[0]),
+        );
+        store
+    }
+
+    /// 换批（正推 B1）：两份逐字节相同的本库 A，从同一份 B（比 A 多两种判定向量）按同一份计划导入，一份照产品的批长（一批写完），
+    /// 一份一字节一批（每块一批）。两份的账只差同步写的批数；写下的库逐字节相同，跨批登记的判定向量编号接着往后编（A 登记了 0、1，
+    /// F2 那一批登记 2、F3 那一批登记 3）；重开之后流 F 每块每种结论按内容读回与 B 相同。
+    /// 绝对值：结论 4 块写下（F1 的 GPU 5 行、F2 的 GPU 5 行加新向量 2 行、F2 的观察者 4 行、F3 的 GPU 5 行加新向量 2 行），F1 的 CPU 本库已有、相同；
+    /// F1 的观察者、F2 的 CPU、F3 的 CPU 与观察者对方库里没有；路径 2 行、核对 2 行、事实 1 行，共 28 行。一字节一批的批：四块结论各一批，
+    /// 两条路径与 F1 的核对行一批（路径放完不判批长），F2 的核对行一批，事实一批，共 7 批。
+    #[test]
+    fn planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes(
+    ) {
+        let other_directory = TemporaryLibraryDirectory::new("planned-import-batches-other");
+        let other =
+            build_the_other_library_b_with_a_second_new_verdict_vector(&other_directory.path);
+        let one_batch_directory = TemporaryLibraryDirectory::new("planned-import-batches-one");
+        let mut imported_in_one_batch = build_the_local_library_a(&one_batch_directory.path);
+        let many_batches_directory = TemporaryLibraryDirectory::new("planned-import-batches-many");
+        let mut imported_in_many_batches = build_the_local_library_a(&many_batches_directory.path);
+        assert_eq!(
+            imported_in_one_batch.every_stored_entry(),
+            imported_in_many_batches.every_stored_entry(),
+            "两份本库 A 开头逐字节相同"
+        );
+        let planned = planned_keys_of_flow_f_beyond_this_host_s_share();
+        let one_batch_tally = imported_in_one_batch
+            .import_the_planned_rows_from(&other, &planned)
+            .expect("照产品的批长导入");
+        let many_batches_tally = imported_in_many_batches
+            .import_the_planned_rows_in_batches_of(
+                &other,
+                &planned,
+                ONE_BYTE_BATCH_WRITES_AFTER_EVERY_PUT_GROUP,
+            )
+            .expect("一字节一批导入");
+        assert_eq!(
+            one_batch_tally,
+            PlannedImportTally {
+                conclusion_blocks_written: 4,
+                conclusion_blocks_already_identical: 1,
+                conclusion_blocks_missing_in_the_other_library: 4,
+                paths_written: 2,
+                verifier_blocks_written: 2,
+                flow_facts_written: 1,
+                rows_written: 28,
+                synced_writes: 1,
+            },
+            "照产品的批长：一批写完"
+        );
+        assert_eq!(
+            many_batches_tally,
+            PlannedImportTally {
+                conclusion_blocks_written: 4,
+                conclusion_blocks_already_identical: 1,
+                conclusion_blocks_missing_in_the_other_library: 4,
+                paths_written: 2,
+                verifier_blocks_written: 2,
+                flow_facts_written: 1,
+                rows_written: 28,
+                synced_writes: 7,
+            },
+            "一字节一批：同样的 28 行分 7 批写"
+        );
+        assert_eq!(
+            imported_in_many_batches.every_stored_entry(),
+            imported_in_one_batch.every_stored_entry(),
+            "多批与一批写下的库逐字节相同（跨批登记的判定向量编号接着上一批往后编）"
+        );
+        drop((imported_in_one_batch, imported_in_many_batches));
+        let (reopened_many_batches, _) =
+            VerdictStore::open_existing(&many_batches_directory.path).expect("重开多批那一份");
+        for (number, expected_vector) in [
+            (2, verdict_vector(b"red-f-judged-on-the-other-host")),
+            (3, verdict_vector(b"red-f-block-three-on-the-other-host")),
+        ] {
+            assert_eq!(
+                reopened_many_batches.verdict_vector_of_number(VerdictVectorNumber(number)),
+                Some(&expected_vector),
+                "编号 {number}：F2 那一批登记 2、F3 那一批接着登记 3"
+            );
+        }
+        for block_index in 1..4 {
+            for (kind, conclusion_version) in [
+                (ConclusionKind::GpuJudge, GPU_CONCLUSION_VERSION),
+                (ConclusionKind::CpuJudge, CPU_CONCLUSION_VERSION),
+                (
+                    observer_kind(MOUNT_OBSERVER_NAME),
+                    OBSERVER_CONCLUSION_VERSION,
+                ),
+            ] {
+                let block_key = flow_f_block(block_index);
+                assert_eq!(
+                    conclusion_read_back_by_content(
+                        &reopened_many_batches,
+                        &block_key,
+                        &kind,
+                        &conclusion_version
+                    ),
+                    conclusion_read_back_by_content(&other, &block_key, &kind, &conclusion_version),
+                    "流 F 第 {block_index} 块的 {kind:?}：多批导入重开之后与 B 按内容读回不同"
+                );
+            }
+        }
+    }
+
+    /// 事实行补写（正推 B2）：本库是断在核对行与事实行之间的那一种（流 F 第 1、2 块的核对行与 B 逐字节相同、它们点名的 F 的事实行不在）。
+    /// 照计划重导这几行核对行：核对行逐字节相同、一行不写，它们点名的事实照样随着导，F 的事实行补上，账是写 1 行、1 批。
+    #[test]
+    fn planned_import_writes_the_facts_named_by_verifier_rows_this_library_already_has() {
+        let other_directory = TemporaryLibraryDirectory::new("planned-import-facts-other");
+        let other = build_the_other_library_b(&other_directory.path);
+        let local_directory = TemporaryLibraryDirectory::new("planned-import-facts-local");
+        let mut local = build_the_local_library_a(&local_directory.path);
+        for (block_index, bits) in [(1, vec![0, 1]), (2, vec![1, 0, 0])] {
+            local
+                .store_verifier_block(
+                    &flow_f_block(block_index),
+                    &CPU_VERIFIER_DIGEST,
+                    &FLOW_F_FACTS_DIGEST,
+                    &bits,
+                )
+                .expect("写核对行");
+            assert_eq!(
+                local
+                    .read_verifier_block(&flow_f_block(block_index), &CPU_VERIFIER_DIGEST)
+                    .expect("读核对行"),
+                other
+                    .read_verifier_block(&flow_f_block(block_index), &CPU_VERIFIER_DIGEST)
+                    .expect("读核对行"),
+                "前提：流 F 第 {block_index} 块的核对行与 B 相同"
+            );
+        }
+        assert_eq!(
+            local.read_flow_facts(&FLOW_F_FACTS_DIGEST).expect("读事实"),
+            None,
+            "前提：本库没有 F 的事实行"
+        );
+        let planned_verifier_rows = PlannedImportKeys {
+            conclusions: Vec::new(),
+            paths: Vec::new(),
+            verifier_blocks: planned_keys_of_flow_f_beyond_this_host_s_share().verifier_blocks,
+        };
+        let tally = local
+            .import_the_planned_rows_from(&other, &planned_verifier_rows)
+            .expect("重导");
+        assert_eq!(
+            tally,
+            PlannedImportTally {
+                conclusion_blocks_written: 0,
+                conclusion_blocks_already_identical: 0,
+                conclusion_blocks_missing_in_the_other_library: 0,
+                paths_written: 0,
+                verifier_blocks_written: 0,
+                flow_facts_written: 1,
+                rows_written: 1,
+                synced_writes: 1,
+            },
+            "核对行都已有且相同、一行不写；它们点名的事实行补写 1 行"
+        );
+        assert_eq!(
+            local.read_flow_facts(&FLOW_F_FACTS_DIGEST).expect("读事实"),
+            Some(b"facts of flow f".to_vec()),
+            "F 的事实行补上了"
+        );
+    }
+
     /// 同一条路径再登同一个身份：什么都不写（库的序号不动）；换一个身份照写。
     #[test]
     fn recording_a_path_again_with_the_same_identity_writes_nothing() {
--- a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
@@ -1209,8 +1209,12 @@
 /// 把第 0、2 块分给 0/2、第 1 块（发布那个点）分给 1/2；发布那个点挂一个只数状态的观察者。导入之后这条流一块没判的都没有；
 /// 本机库里每块每种结论（两种判器、观察者）按内容读回，与一台把整条流判完的那份库逐项相同；导入写的结论块数等于第二台那一份库里的
 /// 结论块数，钉成绝对值 2（CPU 判器 1 块、观察者 1 块；没配 GPU，GPU 判器那一种两边都没有）。
+/// 四趟都设 `SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu`（CPU 参照核），第二台那一块只在第二台的库里核过（攻方 P4-1 的 h02 形状）：
+/// 导入那一趟第 ③ 段比对没有一块缺核对行，本机库里每块在 CPU 核对代码下的核对行与它点名的事实行与一台判完的逐项相同。
+/// 再带同一份导入跑一趟（P5-2）：要导的全是本库已有且相同的，`imported_blocks` 照数，与第一次导入相同。
 /// 判别力（`crates/mutations.tsv`）：排键时选块条件反过来（点的是本机那一份），第二台那一块导不进来、对账剩 1 块；
-/// 排键漏掉观察者那一种，导入只写 1 块、本机库里发布那一块没有观察者的结论。
+/// 排键漏掉观察者那一种，导入只写 1 块、本机库里发布那一块没有观察者的结论；排键一行核对行都不点，导入那一趟比对缺 1 块核对行；
+/// 导入块数只数写下的、不数已有且相同的，再导一次报 0。
 #[test]
 fn importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow()
 {
@@ -1245,6 +1249,7 @@
     let other_share_library = directory.temporary.directory.join("other-share-library");
     let this_share_library = directory.library.clone();
     let mut environment = directory.environment();
+    environment.set("SINGLEFS_CRASH_AMPLIFICATION_VERIFY", OsStr::new("cpu"));
     environment.set(
         "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
         one_host_library.as_os_str(),
@@ -1267,8 +1272,31 @@
         other_share_library.as_os_str(),
     );
     let importing_run = amplify_with_observers(&recorded, "import-planned", &observers);
+    let importing_again_run = amplify_with_observers(&recorded, "import-planned", &observers);
     drop(environment);
     assert_eq!(one_host_run.unjudged_blocks, 0, "一台判完整条流");
+    let comparison_of_the_import = importing_run
+        .comparison
+        .as_ref()
+        .expect("设了 SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu：导入那一趟核了、比了");
+    assert_eq!(
+        (
+            comparison_of_the_import.blocks_without_verifier,
+            comparison_of_the_import.blocks_without_judge,
+            comparison_of_the_import.disagreements
+        ),
+        (0, 0, 0),
+        "导入那一趟第 ③ 段：第二台那一块的核对行随结论一起导进来，没有一块缺核对行"
+    );
+    assert_eq!(
+        (
+            importing_again_run.imported_blocks,
+            importing_again_run.checking.blocks_checked,
+            importing_again_run.unjudged_blocks
+        ),
+        (importing_run.imported_blocks, 0, 0),
+        "再导一次：要导的全是本库已有且相同的，导入块数照数、与第一次相同，一块不判"
+    );
     assert_eq!(
         (
             other_share_run.checking.blocks_checked,
@@ -1333,6 +1361,49 @@
             block_key.stream_or_node_name.as_str()
         );
     }
+    let cpu_verifier_digest = FactsVerifierOn::Cpu.verifier_digest();
+    let mut block_keys_of_the_flow: Vec<&VerdictBlockKey> = Vec::new();
+    for (block_key, _, _) in &one_host_conclusions {
+        if !block_keys_of_the_flow.contains(&block_key) {
+            block_keys_of_the_flow.push(block_key);
+        }
+    }
+    assert_eq!(block_keys_of_the_flow.len(), 3, "前提：这条流 3 块");
+    for block_key in block_keys_of_the_flow {
+        let verifier_row_of_one_host = one_host_store
+            .read_verifier_block(block_key, &cpu_verifier_digest)
+            .expect("读得了一台判完的核对行")
+            .unwrap_or_else(|| {
+                panic!(
+                    "一台判完、核了：块 {} 有 CPU 核对代码的核对行",
+                    block_key.stream_or_node_name.as_str()
+                )
+            });
+        assert_eq!(
+            this_share_store
+                .read_verifier_block(block_key, &cpu_verifier_digest)
+                .expect("读得了导入之后的核对行")
+                .as_ref(),
+            Some(&verifier_row_of_one_host),
+            "块 {} 的核对行：导入之后的本机库与一台判完的读回不同",
+            block_key.stream_or_node_name.as_str()
+        );
+        let facts_of_one_host = one_host_store
+            .read_flow_facts(&verifier_row_of_one_host.facts_digest)
+            .expect("读得了一台判完的事实行");
+        assert!(
+            facts_of_one_host.is_some(),
+            "一台判完：核对行点名的事实行在库里"
+        );
+        assert_eq!(
+            this_share_store
+                .read_flow_facts(&verifier_row_of_one_host.facts_digest)
+                .expect("读得了导入之后的事实行"),
+            facts_of_one_host,
+            "块 {} 的核对行点名的事实行：导入之后的本机库与一台判完的读回不同",
+            block_key.stream_or_node_name.as_str()
+        );
+    }
     drop((one_host_store, other_share_store, this_share_store));
     drop(directory);
 }
@@ -1403,8 +1474,9 @@
 /// 一张都不现判。与旧的整库导入对拍（`VerdictStore::import_from` 把 B 整个导进一份空库，再在那份库上起本机这一份、不设导入，判同一批）：
 /// 两边交回的结论逐张相同；按名导入的库里这一项的（块键，种类，版本）与整库导入的那份里这一项的逐项相同、按内容读回逐项相同，
 /// 另一项的一行都没有。账钉成绝对值：导入 4 块（每张表一块 CPU 判器结论；没配 GPU，GPU 判器那一种 B 里没有）、复用 4 张、现判 0 张。
+/// 再带同一份导入判一遍这 4 张表（P5-2）：要导的全是本库已有且相同的，导入块数照数，账与第一次相同。
 /// 判别力（`crates/mutations.tsv`）：归别的机器的表不导（导入条件里的归属反过来），引擎判第一张表就报「导入之后库里还是没有」；
-/// 点名漏掉 CPU 判器那一种，同样报错。
+/// 点名漏掉 CPU 判器那一种，同样报错；导入块数只数写下的、不数已有且相同的，再导一遍报 0。
 #[test]
 fn given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import(
 ) {
@@ -1460,6 +1532,8 @@
     );
     let (after_the_named_import, named_import_run) =
         judge_every_given_states_table(GIVEN_STATES_IMPORTED_ITEM, &tables);
+    let (after_importing_again, importing_again_run) =
+        judge_every_given_states_table(GIVEN_STATES_IMPORTED_ITEM, &tables);
     drop(environment);
     assert_eq!(
         (
@@ -1488,6 +1562,15 @@
         "按名导入：导入 4 块、复用 4 张、现判 0 张、留给别的机器 0 张"
     );
     assert_eq!(
+        tally_counts(&importing_again_run),
+        tally_counts(&named_import_run),
+        "再导一遍：要导的 4 块全是本库已有且相同的，导入块数照数，账与第一次相同"
+    );
+    assert_eq!(
+        after_importing_again, after_the_named_import,
+        "再导一遍之后引擎交回的结论与第一次逐张相同"
+    );
+    assert_eq!(
         after_the_named_import, after_the_whole_import,
         "两种导法之后引擎交回的结论逐张相同"
     );
--- /dev/null
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs
@@ -0,0 +1,476 @@
+//! checker 档模块：layer0_progress、crash_given_states、crash_identity、crash_amplification、verdict_store、gpu_unit_checks、crash_verify_gpu
+//! 配置开了 GPU 时两台分片之后的按计划导入（第 62 批判决 Q4）：流水线那一路点名的键里有 GPU 判器那一种结论与 GPU 核对代码那一把摘要的核对行，
+//! 给定状态表那一路点名的键里有 GPU 判器那一种结论。没配 GPU 的那几条（`crash_amplification_pipeline_records_checks_and_reuses.rs`）
+//! 库里没有这两样，漏点了照绿；这里两条在卡上真判，漏点哪一样对应的那条就红。
+//! 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」（经这个进程的卡池判，`gpu_test_start::gpu_card_pool_or_report_not_run`），
+//! 有适配器而点名的卡找不到判红；产品路径此刻借不到显存（流水线那一趟照叫停的样子交回、给定状态表的判器报运行条件不满足），同样报「本次未跑」。
+//! 经环境跑的用例自己清、自己还原环境（K5，公共模块的环境守卫）。要 `verdict-store,gpu` 两个特性，少一个这份文件是空的。
+#![cfg(all(feature = "verdict-store", feature = "gpu"))]
+
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
+mod common;
+mod common_crash_points;
+
+use std::ffi::OsStr;
+use std::path::{Path, PathBuf};
+
+use common::{build_pool, file_content, geometry};
+use singlefs_checker_tier::crash_amplification::{
+    card_priority_lines_of_the_caller, CrashPointSpan, EnvironmentRun, GivenStatesEngine,
+    GivenStatesItem, GivenStatesRun, GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX,
+};
+use singlefs_checker_tier::crash_given_states::{
+    GivenStatesConclusionsOfATable, GivenStatesJudge, GivenStatesOfATable,
+};
+use singlefs_checker_tier::crash_identity::ConclusionKind;
+use singlefs_checker_tier::crash_verify_gpu::gpu_verifier_digest;
+use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
+    gpu_card_pool_or_report_not_run, not_run_line,
+};
+use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
+use singlefs_checker_tier::verdict_store::VerdictStore;
+use singlefs_harness::memory_pool::{
+    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
+};
+use singlefs_harness::segments::StepKind;
+
+const THIS_FILE: &str =
+    "crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs";
+/// 只展开不超过 6 次写的段：新池新建文件那条流展开的十段共几十个状态，秒级。
+const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;
+/// 给定状态表那一条：一项 4 张表，每张两个状态。
+const GIVEN_STATES_TABLES: usize = 4;
+const GIVEN_STATES_ITEM: &str = "given_states_gpu_import_probe";
+
+fn repository_root() -> PathBuf {
+    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
+        .join("../../Cargo.lock")
+        .canonicalize()
+        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
+        .parent()
+        .expect("Cargo.lock 所在的目录就是仓根")
+        .to_path_buf()
+}
+
+/// 录好的一条流：崩溃枚举要的五样加两个崩溃点。
+struct RecordedFlow {
+    base: MemoryPool,
+    writes: Vec<RetainedWrite>,
+    segments: Vec<Vec<usize>>,
+    judged_root_index: usize,
+    versions: Vec<PublishedVersion>,
+    crash_points: Vec<CrashPointSpan>,
+}
+
+/// mkfs → 取号 → 两次暖机 → 新池新建文件，按 harness 档 `build_pool` 的做法录；崩溃点按录制流里暖机与发布的分界切
+/// （同 `crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs`，设点的文件是这一份）。
+fn record_new_pool_file_creation(tag: &str) -> RecordedFlow {
+    let pool = build_pool(tag);
+    let operations = pool.retained_operations();
+    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[pool.mkfs_operation_count..],
+        &geometry(),
+    );
+    let publish_boundary = pool.warm_up_operation_count - pool.mkfs_operation_count;
+    let first_publish_write = stream_indexes
+        .iter()
+        .position(|stream_index| *stream_index >= publish_boundary)
+        .expect("发布至少一次写");
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("发布有一条根槽写");
+    let versions = vec![PublishedVersion {
+        instance: pool.output.root.instance,
+        checkpoint_txg: pool.output.root.checkpoint_txg,
+        content: file_content(),
+    }];
+    let crash_points = vec![
+        CrashPointSpan {
+            name: "acquire_instance_and_warm_up".to_string(),
+            code_file_in_repository: THIS_FILE.to_string(),
+            writes: 0..first_publish_write,
+            ignored: false,
+        },
+        CrashPointSpan {
+            name: "publish_first_file".to_string(),
+            code_file_in_repository: THIS_FILE.to_string(),
+            writes: first_publish_write..writes.len(),
+            ignored: false,
+        },
+    ];
+    RecordedFlow {
+        base: pool.memory_pool_after_mkfs(),
+        writes,
+        segments,
+        judged_root_index,
+        versions,
+        crash_points,
+    }
+}
+
+/// 临时目录：一份开了 GPU 的多机配置（显卡先后照抄调用方给这个进程的那一份，`card_priority_lines_of_the_caller`；清场之下要清场之后那一份）
+/// 与两份库（第二台的、本机的），用例结束（成功、失败、panic）都删掉。
+struct GpuImportDirectory {
+    directory: PathBuf,
+    configuration: PathBuf,
+    other_host_library: PathBuf,
+    this_host_library: PathBuf,
+}
+
+impl GpuImportDirectory {
+    fn new(tag: &str) -> Self {
+        let directory = std::env::temp_dir().join(format!(
+            "singlefs-crash-amplification-gpu-import-{tag}-{}",
+            std::process::id()
+        ));
+        let _ = std::fs::remove_dir_all(&directory);
+        std::fs::create_dir_all(&directory).expect("建得了临时目录");
+        let configuration = directory.join("multi-host-with-gpu.env");
+        let cards =
+            card_priority_lines_of_the_caller(&repository_root()).expect("本机的多机配置读得了");
+        std::fs::write(&configuration, format!("ENABLE_GPU=1\n{cards}"))
+            .expect("写得了开 GPU 的配置");
+        let other_host_library = directory.join("other-host-library");
+        let this_host_library = directory.join("this-host-library");
+        Self {
+            directory,
+            configuration,
+            other_host_library,
+            this_host_library,
+        }
+    }
+}
+
+impl Drop for GpuImportDirectory {
+    fn drop(&mut self) {
+        let _ = std::fs::remove_dir_all(&self.directory);
+    }
+}
+
+/// 一条录好的流经公共模块按环境跑一趟（展开上限默认 6）。环境由调用方的守卫定。
+fn amplify(recorded: &RecordedFlow, flow_name: &str) -> EnvironmentRun {
+    common_crash_points::amplify_prepared_flow(
+        flow_name,
+        &recorded.base,
+        &recorded.writes,
+        &recorded.segments,
+        recorded.judged_root_index,
+        &recorded.versions,
+        recorded.crash_points.clone(),
+        LARGEST_EXPANDED_SEGMENT_WRITES,
+    )
+}
+
+fn open_the_library(directory: &Path) -> VerdictStore {
+    VerdictStore::open_existing(directory)
+        .unwrap_or_else(|error| panic!("开得了库 {}：{error:?}", directory.display()))
+        .0
+}
+
+/// 流水线那一路（`crash_amplification` 导入那一段）：配置开 GPU，新池新建文件那条流 3 块，平滑加权轮转把第 1 块（发布那个点）分给 1/2。
+/// 第二台（1/2）在自己的一份库里判那一块（GPU 判器、CPU 对照各一份结论）、在卡上核（GPU 核对代码的核对行）；本机（0/2）判第 0、2 块，
+/// 再带 `SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<第二台的库>` 跑一趟。导入之后 GPU 判器那一种一块没判的都没有，导入的结论块数等于第二台库里的
+/// 结论块数（2）；导入那一趟第 ③ 段在卡上核、拿 GPU 核对代码的核对行比，没有一块缺核对行；第二台那一块的 GPU 核对行导进来、读回相同。
+/// 判别力（`crates/mutations.tsv`）：排键时漏掉 GPU 判器那一种，第二台那一块的 GPU 结论导不进来、对账剩 1 块；
+/// 排键时漏掉 GPU 核对代码那一把摘要，导入那一趟比对缺 1 块核对行。
+#[test]
+fn importing_the_other_share_on_the_gpu_brings_its_gpu_judge_conclusion_and_its_gpu_verifier_row() {
+    let test_name =
+        "importing_the_other_share_on_the_gpu_brings_its_gpu_judge_conclusion_and_its_gpu_verifier_row";
+    if gpu_card_pool_or_report_not_run(test_name).is_none() {
+        return;
+    }
+    let recorded = record_new_pool_file_creation("gpu-pipeline-import");
+    let directory = GpuImportDirectory::new("pipeline");
+    let mut environment = common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[
+        (
+            "SINGLEFS_MULTI_HOST_CONFIG",
+            directory.configuration.as_os_str(),
+        ),
+        (
+            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
+            directory.other_host_library.as_os_str(),
+        ),
+        (
+            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY",
+            OsStr::new("1"),
+        ),
+        ("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("1/2")),
+    ]);
+    let other_share_run = amplify(&recorded, "gpu-pipeline-import");
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
+        directory.this_host_library.as_os_str(),
+    );
+    environment.set("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("0/2"));
+    let this_share_run = amplify(&recorded, "gpu-pipeline-import");
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
+        directory.other_host_library.as_os_str(),
+    );
+    let importing_run = amplify(&recorded, "gpu-pipeline-import");
+    drop(environment);
+    if [&other_share_run, &this_share_run, &importing_run]
+        .iter()
+        .any(|run| run.stopped_early)
+    {
+        println!(
+            "{}",
+            not_run_line(
+                test_name,
+                "产品路径显存不够：此刻借不到或卡上报了 Out of Memory（上面 test=crash_amplification::run_from_environment 那一行）"
+            )
+        );
+        return;
+    }
+    assert!(
+        other_share_run.gpu_judge.is_some(),
+        "前提：配置开了 GPU，第二台那一份在卡上判"
+    );
+    assert_eq!(
+        other_share_run
+            .verification
+            .as_ref()
+            .map(|outcome| outcome.verifier),
+        Some("gpu"),
+        "前提：配置开了 GPU，第二台那一份在卡上核"
+    );
+    assert_eq!(
+        (
+            other_share_run.checking.blocks_checked,
+            this_share_run.checking.blocks_checked
+        ),
+        (1, 2),
+        "前提：1/2 判第 1 块，0/2 判第 0、2 块"
+    );
+    assert_eq!(
+        this_share_run.unjudged_blocks, 1,
+        "前提：导入之前 0/2 的库里第二台那一块没判"
+    );
+    let other_share_store = open_the_library(&directory.other_host_library);
+    let other_share_conclusions = other_share_store
+        .stored_conclusion_keys()
+        .expect("列得出第二台那一份的结论");
+    assert_eq!(
+        other_share_conclusions.len(),
+        2,
+        "前提：第二台那一份是发布那一块的 GPU 判器结论与 CPU 对照结论：{other_share_conclusions:?}"
+    );
+    assert!(
+        other_share_conclusions
+            .iter()
+            .any(|(_, kind, _)| *kind == ConclusionKind::GpuJudge),
+        "前提：第二台那一份库里有 GPU 判器那一种结论：{other_share_conclusions:?}"
+    );
+    assert_eq!(
+        importing_run.unjudged_blocks, 0,
+        "导入之后 GPU 判器那一种一块没判的都没有：第二台那一块的 GPU 判器结论导进来了"
+    );
+    assert_eq!(
+        importing_run.imported_blocks,
+        u64::try_from(other_share_conclusions.len()).expect("块数装得进 u64"),
+        "导入的结论块数等于第二台那一份的结论块数（GPU 判器、CPU 对照各一块）"
+    );
+    let comparison_of_the_import = importing_run
+        .comparison
+        .as_ref()
+        .expect("配置开了 GPU：导入那一趟在卡上核了、比了");
+    assert_eq!(
+        (
+            comparison_of_the_import.blocks_without_verifier,
+            comparison_of_the_import.blocks_without_judge,
+            comparison_of_the_import.disagreements
+        ),
+        (0, 0, 0),
+        "导入那一趟第 ③ 段拿 GPU 核对代码的核对行比：第二台那一块的 GPU 核对行导进来了，没有一块缺核对行"
+    );
+    let this_share_store = open_the_library(&directory.this_host_library);
+    let gpu_verifier = gpu_verifier_digest();
+    for (block_key, _, _) in &other_share_conclusions {
+        let verifier_row_of_the_other_share = other_share_store
+            .read_verifier_block(block_key, &gpu_verifier)
+            .expect("读得了第二台的核对行");
+        assert!(
+            verifier_row_of_the_other_share.is_some(),
+            "前提：第二台在卡上核过块 {}",
+            block_key.stream_or_node_name.as_str()
+        );
+        assert_eq!(
+            this_share_store
+                .read_verifier_block(block_key, &gpu_verifier)
+                .expect("读得了导入之后的核对行"),
+            verifier_row_of_the_other_share,
+            "块 {} 的 GPU 核对行：导入之后的本机库与第二台的读回不同",
+            block_key.stream_or_node_name.as_str()
+        );
+    }
+    drop((other_share_store, this_share_store));
+    drop(directory);
+}
+
+/// 新池新建文件那条流上的几张给定状态表：第 k 张两个状态，持久了前 k·写数/张数 次写的与全部持久的（一条录制流的前缀都是合法的崩溃状态），
+/// 表名带 k（同 `crash_amplification_pipeline_records_checks_and_reuses.rs` 那一份）。
+fn given_states_tables_of_the_flow(
+    recorded: &RecordedFlow,
+    table_count: usize,
+) -> Vec<GivenStatesOfATable<'_>> {
+    let write_count = recorded.writes.len();
+    (0..table_count)
+        .map(|table_index| {
+            let persisted_prefix_writes = table_index * write_count / table_count;
+            GivenStatesOfATable {
+                table_name: format!("gpu-import-probe-table-{table_index}"),
+                base: &recorded.base,
+                writes: &recorded.writes,
+                segments: &recorded.segments,
+                seams: &[],
+                persisted_sets: vec![
+                    (0..write_count)
+                        .map(|write_index| write_index < persisted_prefix_writes)
+                        .collect(),
+                    vec![true; write_count],
+                ],
+            }
+        })
+        .collect()
+}
+
+/// 照此刻的环境起一个给定状态表的引擎，判完 `tables` 每一张、收尾，交回每张的结论与这一趟。配了 GPU 而判器此刻借不到显存
+/// （判器报 [`GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX`] 那一行）时收尾、报「本次未跑」，交 `None`。
+fn judge_every_given_states_table_or_report_not_run(
+    test_name: &str,
+    tables: &[GivenStatesOfATable<'_>],
+) -> Option<(Vec<GivenStatesConclusionsOfATable>, GivenStatesRun)> {
+    let engine = GivenStatesEngine::start_from_environment(
+        GivenStatesItem {
+            item_name: GIVEN_STATES_ITEM.to_string(),
+            setting_file: THIS_FILE.to_string(),
+            seeds_and_scale: "new_pool_file_creation gpu_import_probe".to_string(),
+        },
+        &Layer0ToolchainIdentity::of_the_cargo_running_this_test(),
+        &repository_root(),
+    )
+    .expect("配了 GPU 的给定状态表引擎起得来");
+    let mut conclusions = Vec::with_capacity(tables.len());
+    for table in tables {
+        match engine.judge_the_given_states(table) {
+            Ok(conclusions_of_the_table) => conclusions.push(conclusions_of_the_table),
+            Err(reason) if reason.starts_with(GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX) => {
+                drop(engine.finish());
+                println!("{}", not_run_line(test_name, &reason));
+                return None;
+            }
+            Err(reason) => panic!(
+                "项 {GIVEN_STATES_ITEM} 判得了表 {}：{reason}",
+                table.table_name
+            ),
+        }
+    }
+    Some((conclusions, engine.finish()))
+}
+
+/// 给定状态表那一路（`GivenStatesEngine::import_the_table_from_the_other_hosts`）：配置开 GPU，权重 0,1 让每张表都归 1/2 那一台；
+/// 那一台在卡上判这一项的 4 张表（GPU 判器、CPU 对照各存一块）、写一份库；本机这一份（0/2，一张表都不归它）带
+/// `SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<那一台的库>` 判同一批：每张都从那份库点名导入、照库里的复用（复用要 GPU 判器那一种与对照那一种都在），
+/// 一张都不现判。账：导入的块数等于那一台库里的结论块数（4 张 × 两种 = 8）、复用 4 张、现判 0 张、留给别的机器 0 张。
+/// 判别力（`crates/mutations.tsv`）：点名时漏掉 GPU 判器那一种，导入之后库里还是没有这张表 GPU 判器的结论，引擎判第一张表就报
+/// 「导入之后库里还是没有」。
+#[test]
+fn given_states_tables_judged_on_the_gpu_by_another_host_are_imported_with_their_gpu_judge_conclusions(
+) {
+    let test_name =
+        "given_states_tables_judged_on_the_gpu_by_another_host_are_imported_with_their_gpu_judge_conclusions";
+    if gpu_card_pool_or_report_not_run(test_name).is_none() {
+        return;
+    }
+    let recorded = record_new_pool_file_creation("gpu-given-states-import");
+    let tables = given_states_tables_of_the_flow(&recorded, GIVEN_STATES_TABLES);
+    let directory = GpuImportDirectory::new("given-states");
+    let mut environment = common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[
+        (
+            "SINGLEFS_MULTI_HOST_CONFIG",
+            directory.configuration.as_os_str(),
+        ),
+        (
+            "SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS",
+            OsStr::new("0,1"),
+        ),
+        ("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("1/2")),
+        (
+            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
+            directory.other_host_library.as_os_str(),
+        ),
+        (
+            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY",
+            OsStr::new("1"),
+        ),
+    ]);
+    let Some((_, other_host_run)) =
+        judge_every_given_states_table_or_report_not_run(test_name, &tables)
+    else {
+        return;
+    };
+    environment.set("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("0/2"));
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
+        directory.this_host_library.as_os_str(),
+    );
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
+        directory.other_host_library.as_os_str(),
+    );
+    let Some((_, importing_run)) =
+        judge_every_given_states_table_or_report_not_run(test_name, &tables)
+    else {
+        return;
+    };
+    drop(environment);
+    assert_eq!(
+        (
+            other_host_run.tally.tables_judged_here,
+            other_host_run.tally.states_judged_by_the_gpu_here
+        ),
+        (4, 8),
+        "前提：权重 0,1 下每张表都归 1/2，那一台在卡上判了 4 张表、8 个状态"
+    );
+    assert!(
+        other_host_run.red_items().is_empty(),
+        "前提：那一台判得干净（对照比得上、GPU 都判得了）：{:?}",
+        other_host_run.red_items()
+    );
+    let other_host_store = open_the_library(&directory.other_host_library);
+    let other_host_conclusions = other_host_store
+        .stored_conclusion_keys()
+        .expect("列得出那一台的结论");
+    assert_eq!(
+        other_host_conclusions
+            .iter()
+            .filter(|(_, kind, _)| *kind == ConclusionKind::GpuJudge)
+            .count(),
+        GIVEN_STATES_TABLES,
+        "前提：那一台的库里每张表一块 GPU 判器结论"
+    );
+    assert_eq!(
+        other_host_conclusions.len(),
+        2 * GIVEN_STATES_TABLES,
+        "前提：那一台的库里每张表 GPU 判器、CPU 对照各一块"
+    );
+    drop(other_host_store);
+    assert_eq!(
+        (
+            importing_run.tally.imported_blocks,
+            importing_run.tally.tables_reused,
+            importing_run.tally.tables_judged_here,
+            importing_run.tally.tables_left_to_the_other_hosts
+        ),
+        (8, 4, 0, 0),
+        "按名导入：导入 8 块（每张表 GPU 判器、CPU 对照各一块）、复用 4 张、现判 0 张、留给别的机器 0 张"
+    );
+    assert!(
+        importing_run.red_items().is_empty(),
+        "导入那一趟一张都不现判、不红：{:?}",
+        importing_run.red_items()
+    );
+    drop(directory);
+}
````

## m3-batch62/planned-import-pins：mutations-append.tsv（追加到 crates/mutations.tsv 的 9 行）

````tsv
第62批 Q1 流水线按计划导入 排键时一行核对行都不点（两种核对代码都漏；攻方 P4-1 的 mv）	crates/singlefs-checker-tier/src/crash_amplification.rs	                for verifier_digest in &verifier_digests {	                for verifier_digest in verifier_digests.iter().skip(verifier_digests.len()) {	-p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow	importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow
第62批 Q2 流水线按计划导入 导入块数只数写下的、不数本库已有且相同的（攻方 P5-2 的 mi-pipe）	crates/singlefs-checker-tier/src/crash_amplification.rs	            imported_blocks +=\n                tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;	            imported_blocks +=\n                tally.conclusion_blocks_written + 0 * tally.conclusion_blocks_already_identical;	-p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow	importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow
第62批 Q2 给定状态表按名导入 导入块数只数写下的、不数本库已有且相同的（攻方 P5-2 的 mi-given）	crates/singlefs-checker-tier/src/crash_amplification.rs	                imported_blocks +=\n                    tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;	                imported_blocks +=\n                    tally.conclusion_blocks_written + 0 * tally.conclusion_blocks_already_identical;	-p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import	given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import
第62批 Q3 按计划导入 换批判据反过来（一字节一批时一批都不换，正推 B1）	crates/singlefs-checker-tier/src/verdict_store.rs	        if pending.batch.size_in_bytes() < batch_size_in_bytes {	        if pending.batch.size_in_bytes() >= batch_size_in_bytes {	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes	planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes
第62批 Q3 按计划导入 跨批编号重起（写下一批之后登记的判定向量不进内存表，下一批的新向量从导入前的编号重编，正推 B1）	crates/singlefs-checker-tier/src/verdict_store.rs	        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {	        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().take(0) {	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes	planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes
第62批 Q3 按计划导入 核对行点名事实挪到「本库已有且相同就跳过」之后（核对行在、事实行不在的库补不齐，正推 B2）	crates/singlefs-checker-tier/src/verdict_store.rs	            facts_digests_named.insert(facts_digest);\n            let stored_value = self\n                .database\n                .get(&key_bytes.0)\n                .map_err(VerdictStoreError::RocksDatabase)?;\n            if stored_value.as_deref() == Some(value_bytes.as_slice()) {\n                continue;\n            }\n	            let stored_value = self\n                .database\n                .get(&key_bytes.0)\n                .map_err(VerdictStoreError::RocksDatabase)?;\n            if stored_value.as_deref() == Some(value_bytes.as_slice()) {\n                continue;\n            }\n            facts_digests_named.insert(facts_digest);\n	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::planned_import_writes_the_facts_named_by_verifier_rows_this_library_already_has	planned_import_writes_the_facts_named_by_verifier_rows_this_library_already_has
第62批 Q4 流水线按计划导入 排键时漏掉 GPU 判器那一种结论（换成 CPU 判器那一种，重复的一项导入时去重）	crates/singlefs-checker-tier/src/crash_amplification.rs	                    JudgedBy::GpuJudge.conclusion_kind(),\n                    version_of_the_point(&conclusion_versions.gpu_judge_of_each_point),	                    JudgedBy::CpuJudge.conclusion_kind(),\n                    version_of_the_point(&conclusion_versions.cpu_judge_of_each_point),	-p singlefs-checker-tier --release --features verdict-store,gpu --test crash_amplification_planned_import_names_the_gpu_conclusions -- importing_the_other_share_on_the_gpu_brings_its_gpu_judge_conclusion_and_its_gpu_verifier_row	importing_the_other_share_on_the_gpu_brings_its_gpu_judge_conclusion_and_its_gpu_verifier_row
第62批 Q4 流水线按计划导入 排键时漏掉 GPU 核对代码那一把摘要（换成 CPU 那一把，重复的一项导入时去重）	crates/singlefs-checker-tier/src/crash_amplification.rs	            FactsVerifierOn::Cpu.verifier_digest(),\n            crate::crash_verify_gpu::gpu_verifier_digest(),\n        ];	            FactsVerifierOn::Cpu.verifier_digest(),\n            FactsVerifierOn::Cpu.verifier_digest(),\n        ];	-p singlefs-checker-tier --release --features verdict-store,gpu --test crash_amplification_planned_import_names_the_gpu_conclusions -- importing_the_other_share_on_the_gpu_brings_its_gpu_judge_conclusion_and_its_gpu_verifier_row	importing_the_other_share_on_the_gpu_brings_its_gpu_judge_conclusion_and_its_gpu_verifier_row
第62批 Q4 给定状态表按名导入 点名漏掉 GPU 判器那一种结论	crates/singlefs-checker-tier/src/crash_amplification.rs	            conclusions: [JudgedBy::GpuJudge, JudgedBy::CpuJudge]	            conclusions: [JudgedBy::CpuJudge]	-p singlefs-checker-tier --release --features verdict-store,gpu --test crash_amplification_planned_import_names_the_gpu_conclusions -- given_states_tables_judged_on_the_gpu_by_another_host_are_imported_with_their_gpu_judge_conclusions	given_states_tables_judged_on_the_gpu_by_another_host_are_imported_with_their_gpu_judge_conclusions
````

## m3-batch62/planned-import-pins：mutations-replacements.tsv（整行替换 1 行）

````tsv
按计划导入 批长判据反过来（每项都同步写一批）	crates/singlefs-checker-tier/src/verdict_store.rs	        if pending.batch.size_in_bytes() < batch_size_in_bytes {	        if pending.batch.size_in_bytes() >= batch_size_in_bytes {	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows	importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows
````
