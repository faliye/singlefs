# 第 37 批代码轮第三轮：改动全文（2026-10-09）

相对第二轮冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b37-r2/tree/`，只列第二轮判决第三节改的块；`crates/mutations.tsv` 只列这一轮追加的两行（别的线在表末追加的行不列）。

## `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`

````diff
--- a/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
+++ b/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
@@ -18268,128 +18268,303 @@
     }
 
     /// 判据表每一行都有一条对照且都红（登记 5.4、作废条款 V2；r2 档加的 J6Qn、J7 五行同样，登记 r2 5.5；r5 档加的 J8 五行，登记 r5 5.4；
-    /// T_table、T_row，登记 r2 5.5）。正式跑用的五档（r1、r2、两份表都取的 r2、r3、r5，`research/scripts/replay.sh` 起这个装置的那几条驱动）
-    /// 各自核：这一档跑出对照的判据恰好是登记给这一档的那一份（写死成字面，不从 `is_judged_under` 现算，那是被核的东西），
-    /// 且这一档的读数全红；五档并起来罩全表。不把几档读数并起来判：并起来时一档少跑的对照会被别的档的读数补上
-    /// （判决 `research/prompts/m3-formal-code-b37-r1-main-verification.md` B1-甲）。对抗一族那几行要扫到 n = 3，用上全部核。
+    /// T_table、T_row，登记 r2 5.5）。逐个核正式跑的单元：`research/scripts/replay.sh` 起这个装置的那几条驱动，每条一族、一档、
+    /// 一种建表法、一种盘次序、一种标记奇偶；每个单元跑出对照的判据恰好是那条驱动产物里 `name=control` 的那一份（写死成字面，
+    /// 不从 `is_judged_under`、`Criterion::family` 现算，那是被核的东西），且读数全红；全部单元并起来罩全表。不把几档、几族并起来判：
+    /// 并起来时一个单元少跑的对照会被别的单元的读数补上（判决 `research/prompts/m3-formal-code-b37-r1-main-verification.md` B1-甲、
+    /// `research/prompts/m3-formal-code-b37-r2-main-verification.md` C3-甲）。对抗一族那几行要扫到 n = 3，用上全部核。
     #[test]
     fn every_criterion_has_a_positive_control_and_every_control_is_red() {
         let thread_count = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
-        let parity = MarkerParity::EvenVersionsWrittenByTheUnmountSequence;
-        let legal_and_adversarial = [Family::LegalHistory, Family::Adversarial];
-        let all_families = [Family::LegalHistory, Family::Adversarial, Family::MultiDisk];
-        let criteria_of_every_profile = [
-            Criterion::J0Defined,
-            Criterion::J1ReturnedRoot,
-            Criterion::J2RootPlusDisk,
-            Criterion::J3WithinTheInFlightLimit,
-            Criterion::J4FarthestVersion,
-            Criterion::J5aAppliedSet,
-            Criterion::J5bLandedVersion,
-            Criterion::J5aAndJ5bAreSeparable,
-            Criterion::J6FixedPoint,
-            Criterion::Ia1LargestTxgFails,
-            Criterion::Ia2NoVerificationCounted,
-            Criterion::A0Defined,
-            Criterion::AsWholeTable,
-            Criterion::AsGapContinues,
-            Criterion::AsGapStops,
-            Criterion::AMutual,
-            Criterion::AStructure,
-            Criterion::ALimit,
-        ];
-        let j6qn_and_multi_disk = [
-            Criterion::J6QnFixedPoint,
-            Criterion::MultiDiskAnyCopyIsOnDisk,
-            Criterion::MultiDiskReturnedRoot,
-            Criterion::MultiDiskFarthestVersion,
-            Criterion::MultiDiskAppliedSetAndLandedVersion,
-        ];
-        let j7 = [
-            Criterion::J7aInstanceTable,
-            Criterion::J7aAllocationRecordTreeRoot,
-            Criterion::J7aFilesystemIdentifier,
-            Criterion::J7bUnmountMarkerOfARebuiltRoot,
-            Criterion::J7cUnmountMarkerWhenNothingIsApplied,
-        ];
-        let j8 = [
-            Criterion::J8aFieldsFromTheNewRootSegment,
-            Criterion::J8aUnmountMarkerFromTheNewRootSegment,
-            Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
-            Criterion::J8cNothingAppliedReturnsTheChosenRoot,
-        ];
-        let table_comparison = [
-            Criterion::TableComparisonSeesADifferentTable,
-            Criterion::TableComparisonSeesADifferentRow,
-        ];
-        let split = [Criterion::J6QnSplitByI812];
-        let r2_criteria: Vec<Criterion> =
-            [&criteria_of_every_profile[..], &j6qn_and_multi_disk, &j7].concat();
-        let profiles: [(&str, &[Family], RunProfile, Vec<Criterion>); 5] = [
+        let even = MarkerParity::EvenVersionsWrittenByTheUnmountSequence;
+        let odd = MarkerParity::OddVersionsWrittenByTheUnmountSequence;
+        let labels: std::collections::BTreeSet<&str> = Criterion::ALL
+            .iter()
+            .map(|criterion| criterion.label())
+            .collect();
+        assert_eq!(labels.len(), Criterion::ALL.len(), "判据的标签互不相同");
+        let formal_run_units: [(&str, Family, RunProfile, &[Criterion]); 14] = [
+            // `research/scripts/replay.sh` 第 703、706 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-legal-family.out`，11 条
+            (
+                "r1 合法",
+                Family::LegalHistory,
+                RunProfile::R1,
+                &[
+                    Criterion::J0Defined,
+                    Criterion::J1ReturnedRoot,
+                    Criterion::J2RootPlusDisk,
+                    Criterion::J3WithinTheInFlightLimit,
+                    Criterion::J4FarthestVersion,
+                    Criterion::J5aAppliedSet,
+                    Criterion::J5bLandedVersion,
+                    Criterion::J5aAndJ5bAreSeparable,
+                    Criterion::J6FixedPoint,
+                    Criterion::Ia1LargestTxgFails,
+                    Criterion::Ia2NoVerificationCounted,
+                ],
+            ),
+            // `research/scripts/replay.sh` 第 710 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-adversarial-family.out`，7 条
             (
-                "r1",
-                &legal_and_adversarial,
+                "r1 对抗",
+                Family::Adversarial,
                 RunProfile::R1,
-                criteria_of_every_profile.to_vec(),
+                &[
+                    Criterion::A0Defined,
+                    Criterion::AsWholeTable,
+                    Criterion::AsGapContinues,
+                    Criterion::AsGapStops,
+                    Criterion::AMutual,
+                    Criterion::AStructure,
+                    Criterion::ALimit,
+                ],
+            ),
+            // `research/scripts/replay.sh` 第 738 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-multi-disk.out`，4 条
+            (
+                "r2 多盘",
+                Family::MultiDisk,
+                RunProfile::r2(even),
+                &[
+                    Criterion::MultiDiskAnyCopyIsOnDisk,
+                    Criterion::MultiDiskReturnedRoot,
+                    Criterion::MultiDiskFarthestVersion,
+                    Criterion::MultiDiskAppliedSetAndLandedVersion,
+                ],
+            ),
+            // `research/scripts/replay.sh` 第 741 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-legal-family.out`，19 条
+            (
+                "两份表都取的 r2 合法",
+                Family::LegalHistory,
+                RunProfile::r2(even).with_table_source(RecordTableSource::Both),
+                &[
+                    Criterion::J0Defined,
+                    Criterion::J1ReturnedRoot,
+                    Criterion::J2RootPlusDisk,
+                    Criterion::J3WithinTheInFlightLimit,
+                    Criterion::J4FarthestVersion,
+                    Criterion::J5aAppliedSet,
+                    Criterion::J5bLandedVersion,
+                    Criterion::J5aAndJ5bAreSeparable,
+                    Criterion::J6FixedPoint,
+                    Criterion::Ia1LargestTxgFails,
+                    Criterion::Ia2NoVerificationCounted,
+                    Criterion::J6QnFixedPoint,
+                    Criterion::J7aInstanceTable,
+                    Criterion::J7aAllocationRecordTreeRoot,
+                    Criterion::J7aFilesystemIdentifier,
+                    Criterion::J7bUnmountMarkerOfARebuiltRoot,
+                    Criterion::J7cUnmountMarkerWhenNothingIsApplied,
+                    Criterion::TableComparisonSeesADifferentTable,
+                    Criterion::TableComparisonSeesADifferentRow,
+                ],
+            ),
+            // `research/scripts/replay.sh` 第 744 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-adversarial-family.out`，9 条
+            (
+                "两份表都取的 r2 对抗",
+                Family::Adversarial,
+                RunProfile::r2(even).with_table_source(RecordTableSource::Both),
+                &[
+                    Criterion::A0Defined,
+                    Criterion::AsWholeTable,
+                    Criterion::AsGapContinues,
+                    Criterion::AsGapStops,
+                    Criterion::AMutual,
+                    Criterion::AStructure,
+                    Criterion::ALimit,
+                    Criterion::TableComparisonSeesADifferentTable,
+                    Criterion::TableComparisonSeesADifferentRow,
+                ],
+            ),
+            // `research/scripts/replay.sh` 第 747 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-multi-disk.out`，6 条
+            (
+                "两份表都取的 r2 多盘",
+                Family::MultiDisk,
+                RunProfile::r2(even).with_table_source(RecordTableSource::Both),
+                &[
+                    Criterion::MultiDiskAnyCopyIsOnDisk,
+                    Criterion::MultiDiskReturnedRoot,
+                    Criterion::MultiDiskFarthestVersion,
+                    Criterion::MultiDiskAppliedSetAndLandedVersion,
+                    Criterion::TableComparisonSeesADifferentTable,
+                    Criterion::TableComparisonSeesADifferentRow,
+                ],
+            ),
+            // `research/scripts/replay.sh` 第 750 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-legal-family.out`，19 条
+            (
+                "两份表都取、盘次序反报的 r2 合法",
+                Family::LegalHistory,
+                RunProfile::r2(even)
+                    .with_table_source(RecordTableSource::Both)
+                    .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst),
+                &[
+                    Criterion::J0Defined,
+                    Criterion::J1ReturnedRoot,
+                    Criterion::J2RootPlusDisk,
+                    Criterion::J3WithinTheInFlightLimit,
+                    Criterion::J4FarthestVersion,
+                    Criterion::J5aAppliedSet,
+                    Criterion::J5bLandedVersion,
+                    Criterion::J5aAndJ5bAreSeparable,
+                    Criterion::J6FixedPoint,
+                    Criterion::Ia1LargestTxgFails,
+                    Criterion::Ia2NoVerificationCounted,
+                    Criterion::J6QnFixedPoint,
+                    Criterion::J7aInstanceTable,
+                    Criterion::J7aAllocationRecordTreeRoot,
+                    Criterion::J7aFilesystemIdentifier,
+                    Criterion::J7bUnmountMarkerOfARebuiltRoot,
+                    Criterion::J7cUnmountMarkerWhenNothingIsApplied,
+                    Criterion::TableComparisonSeesADifferentTable,
+                    Criterion::TableComparisonSeesADifferentRow,
+                ],
+            ),
+            // `research/scripts/replay.sh` 第 753 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-multi-disk.out`，6 条
+            (
+                "两份表都取、盘次序反报的 r2 多盘",
+                Family::MultiDisk,
+                RunProfile::r2(even)
+                    .with_table_source(RecordTableSource::Both)
+                    .with_reported_disk_order(ReportedDiskOrder::SecondDeviceBeforeFirst),
+                &[
+                    Criterion::MultiDiskAnyCopyIsOnDisk,
+                    Criterion::MultiDiskReturnedRoot,
+                    Criterion::MultiDiskFarthestVersion,
+                    Criterion::MultiDiskAppliedSetAndLandedVersion,
+                    Criterion::TableComparisonSeesADifferentTable,
+                    Criterion::TableComparisonSeesADifferentRow,
+                ],
+            ),
+            // `research/scripts/replay.sh` 第 763 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-adversarial-family.out`，8 条
+            (
+                "r3 对抗",
+                Family::Adversarial,
+                RunProfile::r3(even),
+                &[
+                    Criterion::A0Defined,
+                    Criterion::AsWholeTable,
+                    Criterion::AsGapContinues,
+                    Criterion::AsGapStops,
+                    Criterion::AMutual,
+                    Criterion::AStructure,
+                    Criterion::ALimit,
+                    Criterion::J6QnSplitByI812,
+                ],
+            ),
+            // `research/scripts/replay.sh` 第 855 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-legal-family.out`，16 条
+            (
+                "r5 合法",
+                Family::LegalHistory,
+                RunProfile::r5(even),
+                &[
+                    Criterion::J0Defined,
+                    Criterion::J1ReturnedRoot,
+                    Criterion::J2RootPlusDisk,
+                    Criterion::J3WithinTheInFlightLimit,
+                    Criterion::J4FarthestVersion,
+                    Criterion::J5aAppliedSet,
+                    Criterion::J5bLandedVersion,
+                    Criterion::J5aAndJ5bAreSeparable,
+                    Criterion::J6FixedPoint,
+                    Criterion::Ia1LargestTxgFails,
+                    Criterion::Ia2NoVerificationCounted,
+                    Criterion::J6QnFixedPoint,
+                    Criterion::J8aFieldsFromTheNewRootSegment,
+                    Criterion::J8aUnmountMarkerFromTheNewRootSegment,
+                    Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
+                    Criterion::J8cNothingAppliedReturnsTheChosenRoot,
+                ],
             ),
+            // `research/scripts/replay.sh` 第 858 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-adversarial-family.out`，8 条
             (
-                "r2",
-                &all_families,
-                RunProfile::r2(parity),
-                r2_criteria.clone(),
+                "r5 对抗",
+                Family::Adversarial,
+                RunProfile::r5(even),
+                &[
+                    Criterion::A0Defined,
+                    Criterion::AsWholeTable,
+                    Criterion::AsGapContinues,
+                    Criterion::AsGapStops,
+                    Criterion::AMutual,
+                    Criterion::AStructure,
+                    Criterion::ALimit,
+                    Criterion::J6QnSplitByI812,
+                ],
             ),
+            // `research/scripts/replay.sh` 第 861 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-multi-disk.out`，4 条
             (
-                "两份表都取的 r2",
-                &all_families,
-                RunProfile::r2(parity).with_table_source(RecordTableSource::Both),
-                [&r2_criteria[..], &table_comparison].concat(),
+                "r5 多盘",
+                Family::MultiDisk,
+                RunProfile::r5(even),
+                &[
+                    Criterion::MultiDiskAnyCopyIsOnDisk,
+                    Criterion::MultiDiskReturnedRoot,
+                    Criterion::MultiDiskFarthestVersion,
+                    Criterion::MultiDiskAppliedSetAndLandedVersion,
+                ],
             ),
+            // `research/scripts/replay.sh` 第 864 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-legal-family.out`，16 条
             (
-                "r3",
-                &all_families,
-                RunProfile::r3(parity),
-                [&r2_criteria[..], &split].concat(),
+                "奇数版本的 r5 合法",
+                Family::LegalHistory,
+                RunProfile::r5(odd),
+                &[
+                    Criterion::J0Defined,
+                    Criterion::J1ReturnedRoot,
+                    Criterion::J2RootPlusDisk,
+                    Criterion::J3WithinTheInFlightLimit,
+                    Criterion::J4FarthestVersion,
+                    Criterion::J5aAppliedSet,
+                    Criterion::J5bLandedVersion,
+                    Criterion::J5aAndJ5bAreSeparable,
+                    Criterion::J6FixedPoint,
+                    Criterion::Ia1LargestTxgFails,
+                    Criterion::Ia2NoVerificationCounted,
+                    Criterion::J6QnFixedPoint,
+                    Criterion::J8aFieldsFromTheNewRootSegment,
+                    Criterion::J8aUnmountMarkerFromTheNewRootSegment,
+                    Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
+                    Criterion::J8cNothingAppliedReturnsTheChosenRoot,
+                ],
             ),
+            // `research/scripts/replay.sh` 第 867 行；产物 `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-adversarial-family.out`，8 条
             (
-                "r5",
-                &all_families,
-                RunProfile::r5(parity),
-                [
-                    &criteria_of_every_profile[..],
-                    &j6qn_and_multi_disk,
-                    &split,
-                    &j8,
-                ]
-                .concat(),
+                "奇数版本的 r5 对抗",
+                Family::Adversarial,
+                RunProfile::r5(odd),
+                &[
+                    Criterion::A0Defined,
+                    Criterion::AsWholeTable,
+                    Criterion::AsGapContinues,
+                    Criterion::AsGapStops,
+                    Criterion::AMutual,
+                    Criterion::AStructure,
+                    Criterion::ALimit,
+                    Criterion::J6QnSplitByI812,
+                ],
             ),
         ];
-        let mut controlled_in_some_profile = std::collections::BTreeSet::new();
-        for (profile_name, families, run_profile, registered_criteria) in profiles {
-            let readings = positive_control_readings(families, run_profile, thread_count);
-            let controlled: std::collections::BTreeSet<&str> = readings
-                .iter()
-                .map(|reading| reading.criterion.label())
-                .collect();
-            let registered: std::collections::BTreeSet<&str> = registered_criteria
-                .iter()
-                .map(|criterion| criterion.label())
-                .collect();
+        let mut controlled_in_some_unit = std::collections::BTreeSet::new();
+        for (unit_name, family, run_profile, registered_criteria) in formal_run_units {
+            let readings = positive_control_readings(&[family], run_profile, thread_count);
+            let controlled: std::collections::BTreeSet<Criterion> =
+                readings.iter().map(|reading| reading.criterion).collect();
+            let registered: std::collections::BTreeSet<Criterion> =
+                registered_criteria.iter().copied().collect();
             assert_eq!(
                 controlled, registered,
-                "{profile_name} 档跑出对照的判据与登记给这一档的那一份不同"
+                "{unit_name} 那个单元跑出对照的判据与那条正式跑产物里的那一份不同"
             );
             for reading in &readings {
                 assert!(
                     reading.is_red_as_required,
-                    "{profile_name} 档对照没红：{reading:?}"
+                    "{unit_name} 那个单元对照没红：{reading:?}"
                 );
             }
-            controlled_in_some_profile.extend(controlled);
+            controlled_in_some_unit.extend(controlled);
         }
         for criterion in Criterion::ALL.iter() {
             assert!(
-                controlled_in_some_profile.contains(criterion.label()),
-                "判据 {} 在正式跑用的五档里都没有对照的读数",
+                controlled_in_some_unit.contains(criterion),
+                "判据 {} 在正式跑的单元里都没有对照的读数",
                 criterion.label()
             );
         }
````

## `crates/mutations.tsv` 这一轮追加的两行（整行原样）

````text
E165 对抗一族八条挪进合法一族（对抗一族的正式跑没有它们的对照，几族并起来判时照过）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            | Self::J6QnSplitByI812 => Family::Adversarial,	            | Self::J6QnSplitByI812 => Family::LegalHistory,	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	every_criterion_has_a_positive_control_and_every_control_is_red
E165 T_table、T_row 只在合法一族跑对照（对抗、多盘两族带两份表的正式跑没有它们的对照）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	    const fn is_judged_in_every_family(self) -> bool {\n        matches!(\n            self,\n            Self::TableComparisonSeesADifferentTable | Self::TableComparisonSeesADifferentRow\n        )\n    }	    const fn is_judged_in_every_family(self) -> bool {\n        false\n    }	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	every_criterion_has_a_positive_control_and_every_control_is_red
````
