# 第 37 批代码轮第一轮：改动全文（2026-10-08）

主 agent 自己改的几处，都已在主工作区（未提交）。下面每份 diff 只含这一批的块：前像是主工作区那份文件把这一批的块反着做之后的样子，同一文件里别的线的改动不在这里（例：`image.rs` 第 282 行是根环线 10u 的）。

## `crates/singlefs-checker/src/image.rs`

````diff
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -209,7 +209,7 @@
 /// I-7.15：各盘所选的系统配置槽里的设备数相同（形态抄 I-1.4 的 fsid 那一判）。
 pub const DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE: &str = "I-7.15";
 /// I-4.1：所选根（重放之后那一版）指向的树全可遍历；checker 自己按 D23（journal 的角色与格式） 已定项 14 的规则只读重放（`walk` 的
-/// `version_the_next_mount_applies`），一条记录都不施加时报不适用，前缀停在点名单元验证不过（起点含介质损坏）时报不适用。
+/// `replay_the_next_mount_would_do`），一条记录都不施加时报不适用，前缀停在点名单元验证不过（起点含介质损坏）时报不适用。
 pub const TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE: &str = "I-4.1";
 /// I-8.1：自证过的系统配置槽里在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋、最坏占用 = 在飞上限 × 4096、F = 格式常量 `JOURNAL_SAFETY_FACTOR`
 /// （三格任一不等，整池不作保，同 I-7.13）；同一块盘环里同 (实例代号, checkpoint_txg) 的记录不多于在飞上限条。
````

## `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`

````diff
--- a/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
+++ b/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
@@ -18268,7 +18268,8 @@
     }
 
     /// 判据表每一行都有一条对照且都红（登记 5.4、作废条款 V2；r2 档加的 J6Qn、J7 五行同样，登记 r2 5.5）。r1 档只判 r1 时的那几行、
-    /// 也都红；r2 档判全表。对抗一族那几行要扫到 n = 3，用上全部核。
+    /// 也都红；r3、r5 与两份表都取的 r3 三档并起来判全表（r5 档加的 J8 五行，登记 r5 5.4；T_table、T_row，登记 r2 5.5）。
+    /// 对抗一族那几行要扫到 n = 3，用上全部核。
     #[test]
     fn every_criterion_has_a_positive_control_and_every_control_is_red() {
         let thread_count = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
@@ -18289,11 +18290,25 @@
                 .all(|reading| reading.criterion != Criterion::J6QnSplitByI812),
             "r2 档不判 J6Qn 的 I-8.12 拆行"
         );
-        let readings = positive_control_readings(
-            &[Family::LegalHistory, Family::Adversarial, Family::MultiDisk],
+        // 全表由三档并起来罩：r3 档判 J7 不判 J8，r5 档判 J8 不判 J7（`judges_j7`、`judges_j8`）；T_table、T_row 只在
+        // 两份记录表都取的那一档判（`compares_the_two_record_tables`，正式跑带 `--table both`）。
+        let all_families = [Family::LegalHistory, Family::Adversarial, Family::MultiDisk];
+        let mut readings = positive_control_readings(
+            &all_families,
             RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
             thread_count,
         );
+        readings.extend(positive_control_readings(
+            &all_families,
+            RunProfile::r5(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
+            thread_count,
+        ));
+        readings.extend(positive_control_readings(
+            &all_families,
+            RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
+                .with_table_source(RecordTableSource::Both),
+            thread_count,
+        ));
         for criterion in Criterion::ALL.iter() {
             assert!(
                 readings
````

## `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`

````diff
--- a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
+++ b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
@@ -1977,6 +1977,9 @@
              inode_tree_root_write_calls=0 inode_tree_root_written_bytes=0 \
              allocation_record_tree_node_write_calls=0 allocation_record_tree_node_written_bytes=0 \
              accounting_tree_node_write_calls=0 accounting_tree_node_written_bytes=0 \
+             deleted_inodes_tree_node_write_calls=0 deleted_inodes_tree_node_written_bytes=0 \
+             tombstone_tree_node_write_calls=0 tombstone_tree_node_written_bytes=0 \
+             lineage_tree_node_write_calls=0 lineage_tree_node_written_bytes=0 \
              central_mapping_tree_node_write_calls=0 central_mapping_tree_node_written_bytes=0 \
              tree_table_unit_write_calls=0 tree_table_unit_written_bytes=0 \
              instance_table_unit_write_calls=0 instance_table_unit_written_bytes=0 \
@@ -2470,7 +2473,7 @@
         let lengths: Vec<usize> = paths.into_iter().map(|(_, slice)| slice.len()).collect();
         assert_eq!(
             lengths,
-            vec![23, 2, 20, 35, 57],
+            vec![25, 2, 20, 35, 57],
             "mkfs / instance_acquisition / warm_up / transaction / post_mkfs_stream 五段的操作数：\
              与 new_pool_file_creation_publish.rs 断言的五个数逐字相同"
         );
````

## `crates/mutations.tsv` 追加两行（主工作区现在的第 3383、3407 行，整行原样）

````text
E165 全表对照只取 r3 一档（J8 五行无对照）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	        readings.extend(positive_control_readings(\n            &all_families,\n            RunProfile::r5(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),	        readings.extend(positive_control_readings(\n            &all_families,\n            RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	every_criterion_has_a_positive_control_and_every_control_is_red
E165 全表对照不取两份表都取的那一档（T_table、T_row 无对照）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)\n                .with_table_source(RecordTableSource::Both),\n            thread_count,\n        ));	            RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)\n                .with_table_source(RecordTableSource::Direct),\n            thread_count,\n        ));	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	every_criterion_has_a_positive_control_and_every_control_is_red
````
