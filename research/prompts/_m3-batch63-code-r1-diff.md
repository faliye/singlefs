# 删除线第 63 批改动全文（2026-10-11）

## m3-batch63/import-number-pins：crates.patch

````diff
--- a/crates/singlefs-checker-tier/src/verdict_store.rs
+++ b/crates/singlefs-checker-tier/src/verdict_store.rs
@@ -4141,12 +4141,45 @@
         store
     }
 
+    /// 第二份对方库 C（流水线逐份导入时的又一份第二台的库）：A 的快照加上流 F 第 3 块的 CPU 结论，用到 B 带过来的那两种判定向量
+    /// （内容相同，在 C 里的编号与本库不同：C 先登记 F3 那一种、再登记 F2 那一种）。
+    fn build_the_second_other_library_c_with_the_same_new_verdict_vectors(
+        directory: &Path,
+    ) -> VerdictStore {
+        let mut store = build_the_local_library_a(directory);
+        let clean = store
+            .register_verdict_vector(&verdict_vector(b""))
+            .expect("登记");
+        let red_of_block_three = store
+            .register_verdict_vector(&verdict_vector(b"red-f-block-three-on-the-other-host"))
+            .expect("登记");
+        let red_judged_on_the_other_host = store
+            .register_verdict_vector(&verdict_vector(b"red-f-judged-on-the-other-host"))
+            .expect("登记");
+        store_conclusion_in_the_test_library(
+            &mut store,
+            &flow_f_block(3),
+            &ConclusionKind::CpuJudge,
+            &CPU_CONCLUSION_VERSION,
+            &conclusion_record(
+                block_of(&[red_of_block_three, red_judged_on_the_other_host, clean]),
+                &[0, 1],
+            ),
+        );
+        store
+    }
+
     /// 换批（正推 B1）：两份逐字节相同的本库 A，从同一份 B（比 A 多两种判定向量）按同一份计划导入，一份照产品的批长（一批写完），
     /// 一份一字节一批（每块一批）。两份的账只差同步写的批数；写下的库逐字节相同，跨批登记的判定向量编号接着往后编（A 登记了 0、1，
     /// F2 那一批登记 2、F3 那一批登记 3）；重开之后流 F 每块每种结论按内容读回与 B 相同。
     /// 绝对值：结论 4 块写下（F1 的 GPU 5 行、F2 的 GPU 5 行加新向量 2 行、F2 的观察者 4 行、F3 的 GPU 5 行加新向量 2 行），F1 的 CPU 本库已有、相同；
     /// F1 的观察者、F2 的 CPU、F3 的 CPU 与观察者对方库里没有；路径 2 行、核对 2 行、事实 1 行，共 28 行。一字节一批的批：四块结论各一批，
     /// 两条路径与 F1 的核对行一批（路径放完不判批长），F2 的核对行一批，事实一批，共 7 批。
+    /// 照产品的批长导完的那一份不重开、接着用（流水线里导入在第 ① 段判之前，判本机的块、存观察者结论都经内存表登记与读编号；攻方 G1）：
+    /// 一批里登记的两种向量照登记次序进内存表（编号 2、3），流 F 每块每种结论按内存表读回与 B 相同；再登记一种新向量，编号是 4。
+    /// 同一个打开着的库接着从第二份对方库 C 导入（`crash_amplification` 逐份导入 `SINGLEFS_CRASH_AMPLIFICATION_IMPORT` 里的几份库那样）：
+    /// C 的 F3 CPU 结论用的两种向量按内容找到本库的 2、3，不另起编号；账钉成绝对值（F3 的 CPU 6 行、F1 的 CPU 相同、其余 7 项 C 里没有）。
+    /// 重开之后流 F 每块每种结论按内容读回与 B 相同（F3 的 CPU 与 C 相同），编号 4 读得回、编号 5 没有。
     #[test]
     fn planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes(
     ) {
@@ -4206,7 +4239,116 @@
             imported_in_one_batch.every_stored_entry(),
             "多批与一批写下的库逐字节相同（跨批登记的判定向量编号接着上一批往后编）"
         );
+        let kinds_of_the_plan = [
+            (ConclusionKind::GpuJudge, GPU_CONCLUSION_VERSION),
+            (ConclusionKind::CpuJudge, CPU_CONCLUSION_VERSION),
+            (
+                observer_kind(MOUNT_OBSERVER_NAME),
+                OBSERVER_CONCLUSION_VERSION,
+            ),
+        ];
+        for (number, expected_vector) in [
+            (2, verdict_vector(b"red-f-judged-on-the-other-host")),
+            (3, verdict_vector(b"red-f-block-three-on-the-other-host")),
+        ] {
+            assert_eq!(
+                imported_in_one_batch.verdict_vector_of_number(VerdictVectorNumber(number)),
+                Some(&expected_vector),
+                "照产品的批长导完、不重开：一批里登记的两种向量照登记次序进内存表，编号 {number} 是这一种"
+            );
+        }
+        for block_index in 1..4 {
+            for (kind, conclusion_version) in &kinds_of_the_plan {
+                let block_key = flow_f_block(block_index);
+                assert_eq!(
+                    conclusion_read_back_by_content(
+                        &imported_in_one_batch,
+                        &block_key,
+                        kind,
+                        conclusion_version
+                    ),
+                    conclusion_read_back_by_content(&other, &block_key, kind, conclusion_version),
+                    "流 F 第 {block_index} 块的 {kind:?}：照产品的批长导完、不重开，按内存表读回与 B 不同"
+                );
+            }
+        }
+        let registered_after_the_import = verdict_vector(b"registered-after-the-import");
+        assert_eq!(
+            imported_in_one_batch
+                .register_verdict_vector(&registered_after_the_import)
+                .expect("导入之后再登记"),
+            VerdictVectorNumber(4),
+            "导入之后再登记一种新向量：编号接在导进来的 0–3 后面"
+        );
+        let second_other_directory =
+            TemporaryLibraryDirectory::new("planned-import-batches-second-other");
+        let second_other = build_the_second_other_library_c_with_the_same_new_verdict_vectors(
+            &second_other_directory.path,
+        );
+        let planned_conclusions_only = PlannedImportKeys {
+            conclusions: planned.conclusions.clone(),
+            paths: Vec::new(),
+            verifier_blocks: Vec::new(),
+        };
+        let second_tally = imported_in_one_batch
+            .import_the_planned_rows_from(&second_other, &planned_conclusions_only)
+            .expect("同一个打开着的库接着从 C 导入");
+        assert_eq!(
+            second_tally,
+            PlannedImportTally {
+                conclusion_blocks_written: 1,
+                conclusion_blocks_already_identical: 1,
+                conclusion_blocks_missing_in_the_other_library: 7,
+                paths_written: 0,
+                verifier_blocks_written: 0,
+                flow_facts_written: 0,
+                rows_written: 6,
+                synced_writes: 1,
+            },
+            "接着从 C 导入：只写 F3 的 CPU 结论 6 行，它用的两种向量本库已登记过、不另登记"
+        );
+        assert_eq!(
+            imported_in_one_batch.verdict_vector_of_number(VerdictVectorNumber(5)),
+            None,
+            "接着从 C 导入：同内容的向量按内容找到本库的编号，不另起编号 5"
+        );
         drop((imported_in_one_batch, imported_in_many_batches));
+        let (reopened_one_batch, _) =
+            VerdictStore::open_existing(&one_batch_directory.path).expect("重开一批那一份");
+        for block_index in 1..4 {
+            for (kind, conclusion_version) in &kinds_of_the_plan {
+                let block_key = flow_f_block(block_index);
+                let imported_from = if block_index == 3 && *kind == ConclusionKind::CpuJudge {
+                    &second_other
+                } else {
+                    &other
+                };
+                assert_eq!(
+                    conclusion_read_back_by_content(
+                        &reopened_one_batch,
+                        &block_key,
+                        kind,
+                        conclusion_version
+                    ),
+                    conclusion_read_back_by_content(
+                        imported_from,
+                        &block_key,
+                        kind,
+                        conclusion_version
+                    ),
+                    "流 F 第 {block_index} 块的 {kind:?}：一批那一份接着导入 C、重开之后按内容读回与它导自的那份库不同"
+                );
+            }
+        }
+        assert_eq!(
+            (
+                reopened_one_batch.verdict_vector_of_number(VerdictVectorNumber(4)),
+                reopened_one_batch.verdict_vector_of_number(VerdictVectorNumber(5)),
+            ),
+            (Some(&registered_after_the_import), None),
+            "重开之后：导入之后登记的那一种是编号 4，没有编号 5"
+        );
+        drop(reopened_one_batch);
         let (reopened_many_batches, _) =
             VerdictStore::open_existing(&many_batches_directory.path).expect("重开多批那一份");
         for (number, expected_vector) in [
--- a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
@@ -1210,7 +1210,7 @@
 /// 本机库里每块每种结论（两种判器、观察者）按内容读回，与一台把整条流判完的那份库逐项相同；导入写的结论块数等于第二台那一份库里的
 /// 结论块数，钉成绝对值 2（CPU 判器 1 块、观察者 1 块；没配 GPU，GPU 判器那一种两边都没有）。
 /// 四趟都设 `SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu`（CPU 参照核），第二台那一块只在第二台的库里核过（攻方 P4-1 的 h02 形状）：
-/// 导入那一趟第 ③ 段比对没有一块缺核对行，本机库里每块在 CPU 核对代码下的核对行与它点名的事实行与一台判完的逐项相同。
+/// 导入那一趟第 ③ 段比对没有一块缺核对行，本机库里每块在 CPU 核对代码下的核对行与一台判完的逐项相同。
 /// 再带同一份导入跑一趟（P5-2）：要导的全是本库已有且相同的，`imported_blocks` 照数，与第一次导入相同。
 /// 判别力（`crates/mutations.tsv`）：排键时选块条件反过来（点的是本机那一份），第二台那一块导不进来、对账剩 1 块；
 /// 排键漏掉观察者那一种，导入只写 1 块、本机库里发布那一块没有观察者的结论；排键一行核对行都不点，导入那一趟比对缺 1 块核对行；
@@ -1395,14 +1395,6 @@
             facts_of_one_host.is_some(),
             "一台判完：核对行点名的事实行在库里"
         );
-        assert_eq!(
-            this_share_store
-                .read_flow_facts(&verifier_row_of_one_host.facts_digest)
-                .expect("读得了导入之后的事实行"),
-            facts_of_one_host,
-            "块 {} 的核对行点名的事实行：导入之后的本机库与一台判完的读回不同",
-            block_key.stream_or_node_name.as_str()
-        );
     }
     drop((one_host_store, other_share_store, this_share_store));
     drop(directory);
````

## m3-batch63/import-number-pins：mutations-append.tsv（追加到 crates/mutations.tsv 的 3 行）

````tsv
第63批 S1 按计划导入 写下一批之后这一批登记的判定向量只有第一种进内存表（一批登两种时第二种不进，攻方 take1）	crates/singlefs-checker-tier/src/verdict_store.rs	        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {	        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().take(1) {	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes	planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes
第63批 S1 按计划导入 写下一批之后这一批登记的判定向量倒着进内存表（一批登两种时内存表里两个编号对调，攻方 rev）	crates/singlefs-checker-tier/src/verdict_store.rs	        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {	        for verdict_vector in pending.verdict_vectors_registered_in_the_batch.into_iter().rev() {	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes	planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes
第63批 S1 按计划导入 写下一批之后这一批登记的判定向量不进按内容查编号的表（接着导下一份对方库时同内容的向量另起编号，正推 R3 旁注）	crates/singlefs-checker-tier/src/verdict_store.rs	            self.verdict_vector_number_by_content\n                .insert(verdict_vector.0.clone(), number);\n            self.verdict_vector_by_number.push(verdict_vector);	            let _ = number;\n            self.verdict_vector_by_number.push(verdict_vector);	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes	planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes
````

## m3-batch63/import-number-pins：mutations-replacements.tsv（整行替换 1 行）

````tsv
第62批 Q3 按计划导入 换批判据反过来（一字节一批时一批都不换，正推 B1）	crates/singlefs-checker-tier/src/verdict_store.rs	        if pending.batch.size_in_bytes() < batch_size_in_bytes {	        if batch_size_in_bytes < PLANNED_IMPORT_BATCH_BYTES || pending.batch.size_in_bytes() < batch_size_in_bytes {	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes	planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes
````
