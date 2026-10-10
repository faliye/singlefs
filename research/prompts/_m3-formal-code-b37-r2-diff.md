# 第 37 批代码轮第二轮：改动全文（2026-10-08）

相对第一轮冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b37-r1/tree/`，只列这一轮被判的四份文件里第一轮判决第三节改的块。`crates/mutations.tsv` 只列本批四行（第一轮之后别的线在表末追加的行不列）。

## `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`

````diff
--- a/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
+++ b/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
@@ -18267,60 +18267,132 @@
         assert!(parse_the_state_probe(&arguments("--replay-state", "X:1:0")).is_err());
     }
 
-    /// 判据表每一行都有一条对照且都红（登记 5.4、作废条款 V2；r2 档加的 J6Qn、J7 五行同样，登记 r2 5.5）。r1 档只判 r1 时的那几行、
-    /// 也都红；r3、r5 与两份表都取的 r3 三档并起来判全表（r5 档加的 J8 五行，登记 r5 5.4；T_table、T_row，登记 r2 5.5）。
-    /// 对抗一族那几行要扫到 n = 3，用上全部核。
+    /// 判据表每一行都有一条对照且都红（登记 5.4、作废条款 V2；r2 档加的 J6Qn、J7 五行同样，登记 r2 5.5；r5 档加的 J8 五行，登记 r5 5.4；
+    /// T_table、T_row，登记 r2 5.5）。正式跑用的五档（r1、r2、两份表都取的 r2、r3、r5，`research/scripts/replay.sh` 起这个装置的那几条驱动）
+    /// 各自核：这一档跑出对照的判据恰好是登记给这一档的那一份（写死成字面，不从 `is_judged_under` 现算，那是被核的东西），
+    /// 且这一档的读数全红；五档并起来罩全表。不把几档读数并起来判：并起来时一档少跑的对照会被别的档的读数补上
+    /// （判决 `research/prompts/m3-formal-code-b37-r1-main-verification.md` B1-甲）。对抗一族那几行要扫到 n = 3，用上全部核。
     #[test]
     fn every_criterion_has_a_positive_control_and_every_control_is_red() {
         let thread_count = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
-        let families = [Family::LegalHistory, Family::Adversarial];
-        let r1_readings = positive_control_readings(&families, RunProfile::R1, thread_count);
-        for reading in &r1_readings {
-            assert!(reading.is_red_as_required, "r1 档对照没红：{reading:?}");
-        }
-        // r3 档判全表（r2 档的每一行再加 J6Qn 的 I-8.12 拆行）；r2 档那一份另核：r2 档不判拆行那一条判据。
-        let r2_readings = positive_control_readings(
-            &[Family::LegalHistory, Family::Adversarial, Family::MultiDisk],
-            RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
-            thread_count,
-        );
-        assert!(
-            r2_readings
-                .iter()
-                .all(|reading| reading.criterion != Criterion::J6QnSplitByI812),
-            "r2 档不判 J6Qn 的 I-8.12 拆行"
-        );
-        // 全表由三档并起来罩：r3 档判 J7 不判 J8，r5 档判 J8 不判 J7（`judges_j7`、`judges_j8`）；T_table、T_row 只在
-        // 两份记录表都取的那一档判（`compares_the_two_record_tables`，正式跑带 `--table both`）。
+        let parity = MarkerParity::EvenVersionsWrittenByTheUnmountSequence;
+        let legal_and_adversarial = [Family::LegalHistory, Family::Adversarial];
         let all_families = [Family::LegalHistory, Family::Adversarial, Family::MultiDisk];
-        let mut readings = positive_control_readings(
-            &all_families,
-            RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
-            thread_count,
-        );
-        readings.extend(positive_control_readings(
-            &all_families,
-            RunProfile::r5(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
-            thread_count,
-        ));
-        readings.extend(positive_control_readings(
-            &all_families,
-            RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)
-                .with_table_source(RecordTableSource::Both),
-            thread_count,
-        ));
+        let criteria_of_every_profile = [
+            Criterion::J0Defined,
+            Criterion::J1ReturnedRoot,
+            Criterion::J2RootPlusDisk,
+            Criterion::J3WithinTheInFlightLimit,
+            Criterion::J4FarthestVersion,
+            Criterion::J5aAppliedSet,
+            Criterion::J5bLandedVersion,
+            Criterion::J5aAndJ5bAreSeparable,
+            Criterion::J6FixedPoint,
+            Criterion::Ia1LargestTxgFails,
+            Criterion::Ia2NoVerificationCounted,
+            Criterion::A0Defined,
+            Criterion::AsWholeTable,
+            Criterion::AsGapContinues,
+            Criterion::AsGapStops,
+            Criterion::AMutual,
+            Criterion::AStructure,
+            Criterion::ALimit,
+        ];
+        let j6qn_and_multi_disk = [
+            Criterion::J6QnFixedPoint,
+            Criterion::MultiDiskAnyCopyIsOnDisk,
+            Criterion::MultiDiskReturnedRoot,
+            Criterion::MultiDiskFarthestVersion,
+            Criterion::MultiDiskAppliedSetAndLandedVersion,
+        ];
+        let j7 = [
+            Criterion::J7aInstanceTable,
+            Criterion::J7aAllocationRecordTreeRoot,
+            Criterion::J7aFilesystemIdentifier,
+            Criterion::J7bUnmountMarkerOfARebuiltRoot,
+            Criterion::J7cUnmountMarkerWhenNothingIsApplied,
+        ];
+        let j8 = [
+            Criterion::J8aFieldsFromTheNewRootSegment,
+            Criterion::J8aUnmountMarkerFromTheNewRootSegment,
+            Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
+            Criterion::J8cNothingAppliedReturnsTheChosenRoot,
+        ];
+        let table_comparison = [
+            Criterion::TableComparisonSeesADifferentTable,
+            Criterion::TableComparisonSeesADifferentRow,
+        ];
+        let split = [Criterion::J6QnSplitByI812];
+        let r2_criteria: Vec<Criterion> =
+            [&criteria_of_every_profile[..], &j6qn_and_multi_disk, &j7].concat();
+        let profiles: [(&str, &[Family], RunProfile, Vec<Criterion>); 5] = [
+            (
+                "r1",
+                &legal_and_adversarial,
+                RunProfile::R1,
+                criteria_of_every_profile.to_vec(),
+            ),
+            (
+                "r2",
+                &all_families,
+                RunProfile::r2(parity),
+                r2_criteria.clone(),
+            ),
+            (
+                "两份表都取的 r2",
+                &all_families,
+                RunProfile::r2(parity).with_table_source(RecordTableSource::Both),
+                [&r2_criteria[..], &table_comparison].concat(),
+            ),
+            (
+                "r3",
+                &all_families,
+                RunProfile::r3(parity),
+                [&r2_criteria[..], &split].concat(),
+            ),
+            (
+                "r5",
+                &all_families,
+                RunProfile::r5(parity),
+                [
+                    &criteria_of_every_profile[..],
+                    &j6qn_and_multi_disk,
+                    &split,
+                    &j8,
+                ]
+                .concat(),
+            ),
+        ];
+        let mut controlled_in_some_profile = std::collections::BTreeSet::new();
+        for (profile_name, families, run_profile, registered_criteria) in profiles {
+            let readings = positive_control_readings(families, run_profile, thread_count);
+            let controlled: std::collections::BTreeSet<&str> = readings
+                .iter()
+                .map(|reading| reading.criterion.label())
+                .collect();
+            let registered: std::collections::BTreeSet<&str> = registered_criteria
+                .iter()
+                .map(|criterion| criterion.label())
+                .collect();
+            assert_eq!(
+                controlled, registered,
+                "{profile_name} 档跑出对照的判据与登记给这一档的那一份不同"
+            );
+            for reading in &readings {
+                assert!(
+                    reading.is_red_as_required,
+                    "{profile_name} 档对照没红：{reading:?}"
+                );
+            }
+            controlled_in_some_profile.extend(controlled);
+        }
         for criterion in Criterion::ALL.iter() {
             assert!(
-                readings
-                    .iter()
-                    .any(|reading| reading.criterion == *criterion),
-                "判据 {} 没有对照的读数",
+                controlled_in_some_profile.contains(criterion.label()),
+                "判据 {} 在正式跑用的五档里都没有对照的读数",
                 criterion.label()
             );
         }
-        for reading in &readings {
-            assert!(reading.is_red_as_required, "对照没红：{reading:?}");
-        }
     }
 
     /// 钉住的表 `.claude/gate.d/e165-pinned-red-examples.tsv` 第 20、27 行的 ①②③⑦ 原样抄来：拿 ⑦ 重算的指纹要等于 ①。
````

## `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`

````diff
--- a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
+++ b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
@@ -1457,7 +1457,7 @@
     let mut counts_after_acquisition: Option<Vec<DeviceCallCounts>> = None;
     let mut counts_before_new_pool_file_creation: Option<Vec<DeviceCallCounts>> = None;
     // 暖机 warm_up() 已调完、publish_first_file() 还没调的那一处：与 new_pool_file_creation_publish.rs 的
-    // warm_up_operation_count（该文件第 187–188 行）量的是同一个边界，不是从流尾往回数几步
+    // warm_up_operation_count（该文件第 189、201 行）量的是同一个边界，不是从流尾往回数几步
     // （里程碑「覆盖写、释放、回退与复用」收尾批「55 号装置步数」）。
     let mut operation_count_before_new_pool_file_creation: Option<usize> = None;
     // 分段挂钟在这里起表：往下每一段末尾取一次，行先攒着、跑完一起打——
@@ -2441,9 +2441,9 @@
     }
 
     /// 暖机与新池新建文件的分界不是从流尾往回数几步：`ScenarioPoint::BeforeNewPoolFileCreation` 处量到的流长度，
-    /// 与 `new_pool_file_creation_publish.rs` 的 `warm_up_operation_count`（该文件第 187–188 行）量的是同一处
+    /// 与 `new_pool_file_creation_publish.rs` 的 `warm_up_operation_count`（该文件第 189、201 行）量的是同一处
     /// ——暖机 `warm_up()` 已调完、`publish_first_file()` 还没调。这里在内存盘上走同一条路径，用它切出五段，
-    /// 钉住该文件那五处断言的操作数：23、2、20、35、57（C577 之后每次发布末尾轮换之后多一道池屏障、两块盘各记一步：暖机两次发布
+    /// 钉住该文件那五处断言的操作数：25、2、20、35、57（mkfs 清 [0, 根环基址)（C651）之后每块盘多一步整段清零：mkfs 23 → 25；C577 之后每次发布末尾轮换之后多一道池屏障、两块盘各记一步：暖机两次发布
     /// 18 → 20——第一次末尾那一道顶替了第二次开头那道没有写在前的屏障、只多第二次末尾那 2 步——、新池新建文件 33 → 35，
     /// mkfs 之后整条 2 + 20 + 35 = 57）。
     #[test]
````

## `crates/singlefs-checker/src/image.rs`

````diff
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -209,7 +209,7 @@
 /// I-7.15：各盘所选的系统配置槽里的设备数相同（形态抄 I-1.4 的 fsid 那一判）。
 pub const DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE: &str = "I-7.15";
 /// I-4.1：所选根（重放之后那一版）指向的树全可遍历；checker 自己按 D23（journal 的角色与格式） 已定项 14 的规则只读重放（`walk` 的
-/// `replay_the_next_mount_would_do`），一条记录都不施加时报不适用，前缀停在点名单元验证不过（起点含介质损坏）时报不适用。
+/// `replay_the_next_mount_would_do`），一条记录都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验证不过（起点含介质损坏）时报不适用。
 pub const TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE: &str = "I-4.1";
 /// I-8.1：自证过的系统配置槽里在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋、最坏占用 = 在飞上限 × 4096、F = 格式常量 `JOURNAL_SAFETY_FACTOR`
 /// （三格任一不等，整池不作保，同 I-7.13）；同一块盘环里同 (实例代号, checkpoint_txg) 的记录不多于在飞上限条。
````

## `crates/mutations.tsv` 本批四行（冻结副本第 3383、3407、3496、3497 行，整行原样）

````text
E165 基础 18 条在 r5 档判了不跑对照（几档并起来判时被 r3 的读数补上）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            | Self::ALimit => true,	            | Self::ALimit => !run_profile.judges_j8(),	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	every_criterion_has_a_positive_control_and_every_control_is_red
E165 T_table、T_row 在两份表都取的 r2 档判了不跑对照（几档并起来判时被 r3 的读数补上）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	                run_profile.compares_the_two_record_tables()	                run_profile.compares_the_two_record_tables() && run_profile.splits_j6qn_by_i812()	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	every_criterion_has_a_positive_control_and_every_control_is_red
E165 基础 18 条在 r1 档不跑对照（r1 段读数为空也照过）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            | Self::ALimit => true,	            | Self::ALimit => run_profile.judges_j6qn(),	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	every_criterion_has_a_positive_control_and_every_control_is_red
报数次序里已删 inode 树与墓碑树对调（各种之和不变，core 的自检照绿）	crates/singlefs-core/src/write_accounting.rs	        WrittenStructureKind::DeletedInodesTreeNode,\n        WrittenStructureKind::TombstoneTreeNode,\n        WrittenStructureKind::LineageTreeNode,	        WrittenStructureKind::TombstoneTreeNode,\n        WrittenStructureKind::DeletedInodesTreeNode,\n        WrittenStructureKind::LineageTreeNode,	-p singlefs-checker-tier --bin new_pool_file_creation_on_device -- publish_writes_line	publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros
````
