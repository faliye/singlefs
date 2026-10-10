# 根环实现第 10 批代码轮第二轮：改动全文（2026-10-05）

只有一份补丁：第 10g 批，相对它合入那一刻的主工作区（删除与并发那条线第 21 批合后）。

## batch10g

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10g/patch/`
- `crates.patch`：sha256 820822ec65ff005ed0a85a074da4165774756c460f8e43915292c42ec4fdf35b
- `mutations-append.tsv`：sha256 77c359e46fbdfc3cc09abbc11bf6e9ce6a0a5e8f2d4cf09e769617d9150fa748
- `mutations-replacements.tsv`：sha256 b62561679785e6e1a84c7f3f571e6eb71719f83764aa3749da63be7f08b613d3
- `mutations-delete.txt`：sha256 1c74adcacf5745aad7bfd4ec9c5a20b0a993d62de12eb6451864ad414f72c3a4
- `report.md`：sha256 7226f601c3206ba101d9afa9cef5d94f794c1ef7628362047f0c9f1e0d5a4957

### crates.patch

````
diff --git a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
index 3dd188c..7549400 100644
--- a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
+++ b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
@@ -3477,6 +3477,9 @@ fn refusal_condition_text(condition: &RefusalConditionNotMet) -> String {
             "system_configuration_unreadable_after_the_read_stage_criterion_true_without_read_faults"
                 .to_string()
         }
+        RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults => {
+            "no_valid_root_in_the_root_ring_criterion_true_without_read_faults".to_string()
+        }
     }
 }
 
diff --git a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
index 30a6b83..a3e7786 100644
--- a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
+++ b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
@@ -849,7 +849,9 @@ where
                 // 读到更新的东西读不出、重读一次仍读不出（C554 乙）：取号之前拒，一个写都没发。
                 | MountError::NewerStateStillUnreadableAfterOneReread(_)
                 // 读阶段之后逐盘核或取号那一刻系统配置又读不出（C664）：第一个取号写之前拒，一个写都没发。
-                | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
+                | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
+                // 重建分配器读根环重读那一遍仍一条有效根都没有（Q3）：取号之前拒，一个写都没发。
+                | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => {
                     vec![describe_run_failure("reopen_and_writable_mount", &cause)]
                 }
             };
@@ -1264,9 +1266,10 @@ fn raise_the_rollback_floor_and_describe<Inner: BlockDevice>(
                 // 现行那一版 txg 到顶：算这一串的 txg 时拒，一个写都没发；可写挂载那一处的抬 F 报错成员抬 F 自己报不出来。
                 | MountError::SequenceNumberPastTheTopOfItsRange(_)
                 | MountError::FloorRaiseFailedAfterTheMountsPublishes(_)
-                // 重读一次仍读不出、读阶段之后系统配置又读不出只由可写挂载交回，抬 F 报不出来。
+                // 重读一次仍读不出、读阶段之后系统配置又读不出、重建分配器读根环重读仍一条有效根都没有只由可写挂载交回，抬 F 报不出来。
                 | MountError::NewerStateStillUnreadableAfterOneReread(_)
-                | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
+                | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
+                | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => {
                     vec![describe_run_failure("raise_rollback_floor", &cause)]
                 }
             };
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index e5a0109..51ff5da 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -535,6 +535,11 @@ pub struct CrashInjectionTally {
     pub remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage: u64,
     pub remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults:
         u64,
+    /// 二次崩溃之后再挂被拒、拒因是「重建分配器读根环整环重读一次仍一条不被抛弃的自证有效根都没有」的（第三截判 Q3 那一条，与 G4 同形）：
+    /// 判过的、崩溃态没注入读故障而那一拒的判据在二次崩溃镜像上复算为真、拒得不合法的（判红）。
+    pub remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator: u64,
+    pub remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults:
+        u64,
     /// 每一截（崩溃镜像本身、挂载与一次发布之后、二次崩溃）判到已知红与新发现的状态数之和。
     pub crash_states_ending_known_red: BTreeMap<usize, u64>,
     pub crash_states_ending_new_finding: u64,
@@ -621,6 +626,8 @@ impl CrashInjectionTally {
             remount_refusals_with_the_witness_criterion_true_without_read_faults,
             remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage,
             remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults,
+            remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
+            remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults,
             crash_states_ending_known_red,
             crash_states_ending_new_finding,
             mount_transparency_judged,
@@ -739,6 +746,10 @@ impl CrashInjectionTally {
             remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage;
         self.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults +=
             remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults;
+        self.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator +=
+            remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator;
+        self.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults +=
+            remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults;
         for (form, count) in crash_states_ending_known_red {
             *self.crash_states_ending_known_red.entry(*form).or_insert(0) += count;
         }
@@ -950,6 +961,12 @@ impl CrashInjectionTally {
             self.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage,
             self.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults
         );
+        let _ = writeln!(
+            text,
+            "二次崩溃之后再挂被拒、拒因是重建分配器读根环整环重读一次仍一条有效根都没有的（第三截判 Q3 那一条）：{} 次，崩溃态没注入读故障而判据复算为真、拒得不合法 {}",
+            self.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
+            self.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults
+        );
         let _ = writeln!(
             text,
             "挂载透明：第一次崩溃镜像上判了 {:?}，二次崩溃镜像上按段判了 {:?}；没判的 {:?}；判红的字段 {:?}；某一样没判的 {:?}；把清扫删的行放回去再比的 {} 次",
@@ -2358,7 +2375,8 @@ fn publishes_before_the_continuation_of_a_mount_that_failed(error: &MountError)
         | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
         | MountError::SequenceNumberPastTheTopOfItsRange(_)
         | MountError::NewerStateStillUnreadableAfterOneReread(_)
-        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => None,
+        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
+        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => None,
     }
 }
 
@@ -3619,6 +3637,9 @@ struct RemountRefusedAfterASecondCrash<'remount> {
 /// 改法 G4（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「新拒因少一条 C670 同形的判法（J2-b）」那一行）同一个接法：
 /// 再挂的拒因是「读阶段判完之后系统配置又读不出」、崩溃态没注入读故障、那一拒的判据在二次崩溃镜像上复算为真 ⇒ 拒得不合法，报红
 /// （判官的 [`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）。
+/// Q3（用户 2026-10-05 定，判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）同一个接法：再挂的拒因是「重建分配器读根环
+/// 整环重读一次仍一条不被抛弃的自证有效根都没有」、崩溃态没注入读故障、那一拒的判据在二次崩溃镜像上复算为真 ⇒ 拒得不合法，报红
+/// （判官的 [`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）。
 /// 拒因是实例表读不出的不判。
 fn judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_without_read_faults(
     tally: &mut CrashInjectionTally,
@@ -3693,9 +3714,11 @@ fn judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_with
     );
 }
 
-/// 第三截那一判的计数与取舍：拒因是判据 N-配置 或「读阶段判完之后系统配置又读不出」的，各记一次判过；判官列出的没满足的条件里有
+/// 第三截那一判的计数与取舍：拒因是判据 N-配置、「读阶段判完之后系统配置又读不出」或「重建分配器读根环整环重读一次仍一条有效根都没有」的，
+/// 各记一次判过；判官列出的没满足的条件里有
 /// 没注入读故障时判据复算为真的那一条（N-配置 的 C670：[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]；
-/// 另一拒的改法 G4：[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）时
+/// 另一拒的改法 G4：[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]；
+/// Q3：[`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）时
 /// 再各记一次不合法，交回拒因与全部没满足的条件（报红用）。拒因是实例表读不出的、那一条不成立（注入过读故障、判据复算为假）、只差 U6 那几条的，交回 None。
 fn reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
     tally: &mut CrashInjectionTally,
@@ -3742,6 +3765,22 @@ fn reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_with
                 conditions_not_met,
             )
         }
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
+        ) => {
+            tally.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator += 1;
+            return None;
+        }
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
+            conditions_not_met,
+        } => {
+            tally.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator += 1;
+            (
+                RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
+                conditions_not_met,
+            )
+        }
     };
     let mut criterion_true_without_read_faults = false;
     // 迭代上界是判官列出的条件数（至多五条）；跨轮携带的只有上面那一个布尔；没有提前出口。
@@ -3755,6 +3794,10 @@ fn reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_with
                 tally.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults += 1;
                 criterion_true_without_read_faults = true;
             }
+            RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults => {
+                tally.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults += 1;
+                criterion_true_without_read_faults = true;
+            }
             RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
             | RefusalConditionNotMet::DiskChangedByTheRefusedMount
             | RefusalConditionNotMet::ReadOnlyReadBackOutsideTheAllowedSet => {}
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs b/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs
index f3e1a5b..f58e7a3 100644
--- a/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs
+++ b/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs
@@ -1612,8 +1612,12 @@ fn a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_findi
 ///   第 22 段 (1, 5) 的根槽写在窗口之后、没落。验点名单元的恢复（`JournalPolicy::Consult`，崩溃注入用的就是它）不施加那条记录、落回 (1, 4)，
 ///   两种交法同判；只有不验点名单元的恢复（只供测试强制进入的 `JournalPolicy::ConsultWithoutNamedVerification`）落到 (1, 5)，整条流判红、前缀不判。
 ///
-/// 所以在崩溃注入摆得出的每个崩溃状态上两种交法同值，把第一截的写表退回前缀那一条变异是等价变异，这一条给它留档
-/// （`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」）；C554 乙那一判或点名验证哪天不在了，两种交法才会分开。
+/// 所以把第一截的写表退回前缀那一条变异不是等价变异：这一条自己就造出了两种交法不同值的输入（不验点名单元的恢复落到 (1, 5)，整条流判红、前缀不判）。
+/// 它是第三类「取样点不敏感」（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」）：崩溃注入的取值域里摆不出能分辨两种交法的盘面——
+/// 崩溃注入恢复只用验点名单元的那一种（`JournalPolicy::Consult`），C554 乙那一判又让另一形摆不出来；补不上敏感的取样点，是因为要给崩溃注入
+/// 加换恢复策略的开关（改被测代码，判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第五节「崩溃注入换恢复策略的开关」那一行），
+/// 不是因为两种交法在所有输入上同值。这一条钉住取值域外那一支的两种交法确实分开、取值域里那一形两种交法同判；C554 乙那一判或点名验证哪天不在了，
+/// 崩溃注入里两种交法也会分开。
 /// 第二条判据不罩恰好是恢复落到的那一版的那次发布（`crates/mutations.tsv`），整条流那一判不红，这一条红。
 #[test]
 fn a_version_rebuilt_from_records_missing_a_unit_is_judged_only_on_the_whole_stream_and_the_verifying_recovery_never_lands_on_it(
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 4bdfd8a..74a1d3d 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -97,6 +97,11 @@ pub use crate::instance_table::{InstanceRow, InstanceTableRecords};
 #[derive(Debug)]
 pub enum MountError {
     Recovery(RecoveryFailure),
+    /// 重建分配器读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出（被抛弃按最新那条根指着的实例表判），整环重读一次，
+    /// 重读那一遍整个环仍一条都没有：在取号之前拒可写，盘上逐字节不变，只读挂载照常（D23（journal 的角色与格式） 已定项 14 那张单子上的一行，
+    /// 用户 2026-10-05 定「另起拒因」，岔路单 `research/prompts/m3-batch10-code-r1-forks.md` Q3；重读的触发照 Q4 rr2）。
+    /// 与读阶段择不到根的 `Recovery(RecoveryFailure::NoValidRoot)` 分开报：那一拒只读挂载同样挂不上，这一拒读阶段择得到根。
+    NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
     /// 所选根下面有文件版本，而环里一条自证过的记录都没有：从盘上重建上一版要一条记录顶着，拿不出来
     /// （树表 0 条的根用不到记录，刚 mkfs 的池不走这里）。
     FileVersionWithoutAnyJournalRecord,
@@ -486,7 +491,7 @@ pub enum WitnessedCounterComparison {
 /// （`.claude/rules/fs-design.md` 五条硬要求第 2 条：用例在钩子里撤掉暂时的读故障，强制进入「重读读得出」那一支；
 /// 或在钩子里让重读那一遍也读坏，强制进入「重读仍读不出」那一支）。
 /// 每处「重读一次」之前各调一次：读阶段那一次（判据 N-配置），重建分配器读最新那条根的实例表那一次（代码审阅第 22 条），
-/// 重建分配器读根环一条不被抛弃的自证有效根都没读出、整环重读那一次（用户 2026-10-05 定「重读」），影子账里一条被抛弃根的账与从映射现算都不成、
+/// 重建分配器读根环有一块盘一条不被抛弃的自证有效根都没读出、整环重读那一次（用户 2026-10-05 定「重读」与 Q4 rr2），影子账里一条被抛弃根的账与从映射现算都不成、
 /// 重读它那一次（C393，每条这样的根各一次）。
 pub enum BeforeTheOneReread<'hook> {
     RereadImmediately,
@@ -530,8 +535,9 @@ pub enum InstanceTableOfTheNewestRootRead {
     OnTheOneReread,
 }
 
-/// 重建分配器读根环（根环表、回收门槛与影子账用的那一遍读数）在哪一遍读出不被抛弃的自证有效根：第一遍一条都没读出就整环重读一次
-/// （用户 2026-10-05 定「重读」），重读也没有就拒（`MountError::Recovery(RecoveryFailure::NoValidRoot)`，不交回这个值）。
+/// 重建分配器读根环（根环表、回收门槛与影子账用的那一遍读数）在哪一遍读出不被抛弃的自证有效根：第一遍有一块盘一条都没读出就整环重读一次
+/// （用户 2026-10-05 定「重读」，Q4 定 rr2：不只两块盘都没读出时），重读那一遍整个环一条都没有就拒
+/// （`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，不交回这个值）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum RootRingOfTheRebuiltAllocatorRead {
     OnTheFirstRead,
@@ -1830,7 +1836,8 @@ fn txgs_ring_slots_bad_at_mount_can_hold(
 /// # Errors
 /// 最新那条根的实例表读不出、重读一次仍读不出（[`instance_table_of_the_newest_root_read_at_most_twice`]）——在读阶段之后、取号之前；
 /// 树表 0 条那一版的账重建不出（`allocator_of_version_without_file`）；
-/// 读根环那一遍一条不被抛弃的自证有效根都没读出、整环重读一次仍没有 ⇒ `Recovery(RecoveryFailure::NoValidRoot)`（用户 2026-10-05 定「重读」）。
+/// 读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，重读那一遍整个环仍一条都没有 ⇒
+/// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2）。
 /// 都在取号之前，盘上逐字节不变。
 #[allow(
     clippy::ptr_arg,
@@ -1912,15 +1919,46 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 })
                 .max()
         };
-    // 这一遍一条不被抛弃的自证有效根都没读出（择根那几遍读得出）就把根环整环再读一遍（重读之前调 `before_the_one_reread`，与另几处重读一样），
-    // 根环表、回收门槛与影子账都用重读的读数；
-    // 重读还是一条都没有，就在任何写之前拒这次可写挂载（用户 2026-10-05 定「重读」，岔路单 `research/prompts/m3-batch17-code-r1-forks.md`
-    // 第 2 行；判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第二节「N1 甲二」那一行与第四节）。不重读、给空表往下走时，
+    // 一条根落在哪块盘由根环落点公式定（下面判「每块盘都读出了有效根」与当场放开到哪儿都用它）。
+    let device_of_txg = |txg: CheckpointTxg| {
+        let target = target_for_publish(txg, slots_per_region);
+        system_configuration.immutable.region_devices
+            [usize::try_from(target.region).expect("区域号")]
+    };
+    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
+    // 这一遍每块盘都读出了一条不被抛弃的自证有效根（与下面当场放开到哪儿那一步取 m 同一处定义 [`newest_valid_root_txg_on_every_device`]）。
+    let a_valid_root_was_read_on_every_device =
+        |readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)]| {
+            let roots_of_the_readings: Vec<RootRecord> = readings
+                .iter()
+                .filter_map(|(_, reading)| match reading {
+                    RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) => Some(*root),
+                    RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
+                    | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
+                })
+                .collect();
+            newest_valid_root_txg_on_every_device(
+                &roots_of_the_readings,
+                &newest_table,
+                &all_devices,
+                &device_of_txg,
+            )
+            .is_some()
+        };
+    // 这一遍有一块盘一条不被抛弃的自证有效根都没读出（择根那几遍读得出）就把根环整环再读一遍（用户 2026-10-05 定 Q4 rr2，
+    // 岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 第 4 行：改之前只在整个环一条都没读出时重读，只坏这一遍、只有一块盘没读出的那一形
+    // m 取不到、当场只放到环里最旧有效根，窄盘上取不到落点被拒；重读之前调 `before_the_one_reread`，与另几处重读一样），
+    // 根环表、回收门槛与影子账都用重读的读数（重读那一遍仍有一块盘没读出的，照 m 取不到那一支往下走）；
+    // 重读那一遍整个环还是一条都没有，就在任何写之前拒这次可写挂载（用户 2026-10-05 定「重读」，岔路单 `research/prompts/m3-batch17-code-r1-forks.md`
+    // 第 2 行；判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第二节「N1 甲二」那一行与第四节；拒因另起成员，Q3）。不重读、给空表往下走时，
     // 写行那次记根算回收门槛只看得到它自己，释放代高于 F_生效 的一截当场放开、被暖机写掉，盘上环里更旧的有效根还引用着（D16（发布语义） 已定项 1
     // 「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）。拒在取号之前，盘上逐字节不变。
     let first_readings = read_every_root_ring_slot_once();
+    let window_anchor_of_the_first_readings =
+        newest_valid_root_txg_of_the_readings(&first_readings)
+            .filter(|_| a_valid_root_was_read_on_every_device(&first_readings));
     let (ring_slot_readings, window_anchor, root_ring_of_the_rebuilt_allocator) =
-        match newest_valid_root_txg_of_the_readings(&first_readings) {
+        match window_anchor_of_the_first_readings {
             Some(window_anchor) => (
                 first_readings,
                 window_anchor,
@@ -1932,7 +1970,9 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 let Some(window_anchor) =
                     newest_valid_root_txg_of_the_readings(&readings_of_the_reread)
                 else {
-                    return Err(MountError::Recovery(RecoveryFailure::NoValidRoot));
+                    return Err(
+                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
+                    );
                 };
                 (
                     readings_of_the_reread,
@@ -1975,7 +2015,8 @@ fn rebuilt_allocator<Device: BlockDevice>(
     // 挂载时读坏的槽能住哪几个 txg（[`txgs_ring_slots_bad_at_mount_can_hold`]）：N = 环里读得出的最新有效根的 txg（D16（发布语义） 已定项 1
     // 「挂载时读不出的槽」那一句：挂载重建分配器时取它、不取 first_txg − 1）。读回全零的槽按同一个算法另算一份、只取可能住过的最旧 txg：
     // 回收门槛里照「自证不过」那一类钉住（用户 2026-10-05 定乙），准入抬 F 的上限不并它（用户 2026-10-04 定），能住的最大 txg 用不到。
-    // 一个 txg 都住不了的槽（年轻的环里还没轮到的）两类都不交回。窗口锚上面取到了（取不到就重读、仍取不到就拒），两类不会因为它是空表。
+    // 一个 txg 都住不了的槽（年轻的环里还没轮到的）两类都不交回。窗口锚上面取到了（有一块盘没读出就重读、重读那一遍整个环仍取不到就拒），
+    // 两类不会因为它是空表。
     let ring_slots_bad_at_mount = txgs_ring_slots_bad_at_mount_can_hold(
         &bad_ring_slots_not_all_zero,
         window_anchor,
@@ -2017,16 +2058,12 @@ fn rebuilt_allocator<Device: BlockDevice>(
     // （判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「形 R」那一行）。于是当场可用的只到
     // reclaim_floor(min(F_生效, m), 环里最旧有效根)（RGall），m 是每块盘上最新有效根的 txg 取最小（被抛弃的根不算，与卸载那一格同一处定义
     // [`newest_valid_root_txg_on_every_device`]）；其余到原门槛为止的那一截扣住，这次挂载的暖机在每块盘上都落了根之后放开（`establish_instance`）。
-    // 某块盘一条有效根都读不出（m 取不到）时当场只放到一圈窗口里最旧有效根（用户 2026-10-04 选甲，`research/prompts/m3-batch16-code-r1-forks.md`
-    // 第 1 行；判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「再挂期间某块盘根环整段读坏」那一行），RGall 之后这个量是
-    // 环里最旧有效根：那块盘上真正最新的根这一遍看不到，它引用的单元释放代可以高到 F_生效；它与回收门槛同一个读数（挂载时读坏、读回全零的槽钉在它可能住过的最旧 txg 并进）。
-    // 这一遍（或它的重读）至少读出了一条有效根（上面取窗口锚那一步：两遍都读不出就拒），环里最旧有效根恒取得到。
-    let device_of_txg = |txg: CheckpointTxg| {
-        let target = target_for_publish(txg, slots_per_region);
-        system_configuration.immutable.region_devices
-            [usize::try_from(target.region).expect("区域号")]
-    };
-    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
+    // 某块盘一条有效根都读不出（m 取不到）时当场只放到环里最旧有效根（用户 2026-10-04 选甲，`research/prompts/m3-batch16-code-r1-forks.md`
+    // 第 1 行；判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「再挂期间某块盘根环整段读坏」那一行；用户 2026-10-04 定 RGall
+    // 之后取整个环里的有效根、不按一圈窗口滤）：那块盘上真正最新的根这一遍看不到，它引用的单元释放代可以高到 F_生效；它与回收门槛同一个读数
+    // （挂载时读坏、读回全零的槽钉在它可能住过的最旧 txg 并进）。第一遍有一块盘没读出就整环重读（Q4 rr2），走到这一支的只剩重读那一遍
+    // 仍有一块盘一条有效根都没读出的那一形。这一遍（或它的重读）至少读出了一条有效根（上面取窗口锚那一步：重读那一遍也一条都读不出就拒），
+    // 环里最旧有效根恒取得到。
     let floor_of_the_reclaim = reclaim_floor(effective_floor, oldest_valid_root_in_the_ring);
     let floor_released_immediately = match newest_valid_root_txg_on_every_device(
         &roots,
@@ -2932,7 +2969,9 @@ fn floor_raise_refused_for_space(refusal: &MountError) -> bool {
         // 重读一次仍读不出只由可写挂载的读阶段与重建分配器交回（C554 乙），读阶段之后系统配置又读不出只由可写挂载取号之前与取号交回：
         // 抬 F 报不出来，也不是空间不够。
         | MountError::NewerStateStillUnreadableAfterOneReread(_)
-        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => false,
+        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
+        // 重建分配器读根环重读那一遍仍一条有效根都没有只由可写挂载的重建分配器交回：抬 F 报不出来，也不是空间不够。
+        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => false,
     }
 }
 
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index 87267a6..23655c1 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -2945,6 +2945,9 @@ fn mount_error_member(error: &MountError) -> String {
         MountError::Recovery(failure) => {
             return format!("MountError::Recovery({})", recovery_failure_member(failure))
         }
+        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => {
+            "NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread"
+        }
         MountError::FileVersionWithoutAnyJournalRecord => "FileVersionWithoutAnyJournalRecord",
         MountError::InstanceTableMalformed => "InstanceTableMalformed",
         MountError::Acquisition(acquisition) => {
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index 5e0d252..9734b07 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -865,6 +865,9 @@ pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReaso
         | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
         // 盘上的实例代号或 txg 已到顶：坏镜像才有。
         | MountError::SequenceNumberPastTheTopOfItsRange(_)
+        // 重建分配器读根环有一块盘没读出有效根、整环重读那一遍整个环仍一条都没有（Q3 另起的拒因）：读阶段择得到根，只在读路径上注入故障才有，
+        // 模型没有这一条。
+        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread
         // 读阶段判完之后（判据 N-配置 为假）逐盘核或取号那一刻系统配置又读不出（C664（挂载拒绝合法复算把两种拒因报成一个）之前并在 N-配置 里、
         // 映射成模型那一条理由）：两遍读之间的瞬时读错或一块盘两槽都坏才有，模型没有这一条，不映射成「见证过比所选那一版新的发布」。
         | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
@@ -921,7 +924,8 @@ pub fn root_ring_slot_still_bad_after_one_reread_of_mount_error(
         | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
         | MountError::SequenceNumberPastTheTopOfItsRange(_)
         | MountError::NewerStateStillUnreadableAfterOneReread(_)
-        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => None,
+        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
+        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => None,
     }
 }
 
@@ -996,7 +1000,8 @@ pub fn reported_ceiling_of_mount_error(error: &MountError) -> Option<ModelCheckp
         | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
         | MountError::SequenceNumberPastTheTopOfItsRange(_)
         | MountError::NewerStateStillUnreadableAfterOneReread(_)
-        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => None,
+        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
+        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => None,
     }
 }
 
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index 4b9d90e..d1e18fc 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -2224,6 +2224,10 @@ pub enum RefusalReasonOnTheD23List {
     /// 读阶段判完、判据 N-配置 为假之后，取号之前的逐盘核或取号那一刻读系统配置判不出（R2）或某块盘一份自证过的都没有
     /// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`；C664（挂载拒绝合法复算把两种拒因报成一个）之前并在 N-配置 里报）。
     SystemConfigurationUnreadableAfterTheReadStage,
+    /// 重建分配器读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，重读那一遍整个环仍一条都没有
+    /// （`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`；用户 2026-10-05 定 Q3「另起拒因、单子加一行」，
+    /// 岔路单 `research/prompts/m3-batch10-code-r1-forks.md`；之前报 `Recovery(NoValidRoot)`、与读阶段择不到根同名，不在单子上）。
+    NoValidRootInTheRootRingOfTheRebuiltAllocator,
 }
 
 /// 拒可写的理由在不在 D23 已定项 14 那张单子上。别的理由（单元区墙、坏盘上才有的那几种……）照崩溃注入第二截今天的写法判。
@@ -2247,6 +2251,10 @@ pub fn refusal_reason_on_the_d23_list(error: &MountError) -> Option<RefusalReaso
         MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
             Some(RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage)
         }
+        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => {
+            Some(RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator)
+        }
+        // 恢复报的错（读阶段择不到根那一拒在内，只读挂载同样挂不上）：不在单子上。
         MountError::Recovery(_)
         | MountError::FileVersionWithoutAnyJournalRecord
         | MountError::InstanceTableMalformed
@@ -2519,6 +2527,39 @@ fn instance_table_of_the_newest_root_unreadable_on_the_crash_image(
         .is_none_or(|newest| instance_table_of_root(crash_image, &newest).is_none())
 }
 
+/// 「重建分配器读根环整环重读一次仍一条不被抛弃的自证有效根都没有」（Q3）在崩溃镜像上复算：按最新那条根（`choose_root`）指着的实例表，
+/// 根环里一条不被抛弃的自证有效根都没有。被抛弃照 D23（journal 的角色与格式） 已定项 14 候选集那一格在这里另写一份：表里有那条根的实例的行
+/// (i, Ti, Wi) 且它的 txg > Ti（不调实现的 `recovery::root_is_abandoned_by_the_instance_table`）。择不到系统配置、择不到根都按真；
+/// 最新那条根的实例表读不出时没有「被抛弃」可判，不按表滤（读得出的根都算有效，与 F 生效值那一处的取法同向），择得到根就为假。
+/// 没有读故障时最新那条根按它自己指着的表恒不被抛弃（表里的行只记更早的实例，推的），所以择得到根的崩溃镜像上这条判据为假。
+fn no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(
+    crash_image: &dyn PoolReader,
+) -> bool {
+    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
+        return true;
+    };
+    let Some(newest) = choose_root(crash_image, &system_configuration) else {
+        return true;
+    };
+    let Some(table_of_the_newest_root) = instance_table_of_root(crash_image, &newest) else {
+        return false;
+    };
+    let immutable = &system_configuration.immutable;
+    !readable_roots(
+        crash_image,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    )
+    .iter()
+    .any(|root| {
+        !table_of_the_newest_root
+            .rows
+            .iter()
+            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
+    })
+}
+
 /// 拒之后盘上逐字节同不同（调用方比拒之前与拒之后的整份镜像）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum DiskAfterTheRefusal {
@@ -2556,7 +2597,8 @@ pub enum ReadFaultsUnderTheRefusedMount {
 }
 
 impl ReadFaultsUnderTheRefusedMount {
-    /// 「读阶段判完之后系统配置又读不出」那一拒的判据在崩溃镜像上复算为假，能不能是被拒那次挂载里一次瞬时读错（复算不出）。
+    /// 「读阶段判完之后系统配置又读不出」与「重建分配器读根环整环重读一次仍一条有效根都没有」（Q3）那两拒的判据在崩溃镜像上复算为假，
+    /// 能不能是被拒那次挂载里一次瞬时读错（复算不出）。
     fn can_hide_a_transient_read_error_from_the_crash_image(self) -> bool {
         match self {
             Self::TheRefusedMountReadTheCrashImageWithoutFaults => false,
@@ -2579,6 +2621,11 @@ pub enum RefusalConditionNotMet {
     /// 崩溃态没注入读故障，拒因是「读阶段判完之后系统配置又读不出」，那一拒的判据在崩溃镜像上复算为真（某块盘一份本池自证过的系统配置都没有、
     /// 一份都择不到，或见证判不出）——没有读故障的历史留不下这份盘面，正确实现在它上面永远拒可写；与 C670 那一条同形，拒是对的、造出这份盘面的那一段不对。
     SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
+    /// 与改法 G4 同形（用户 2026-10-05 定 Q3，判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）：崩溃态没注入读故障，
+    /// 拒因是「重建分配器读根环整环重读一次仍一条不被抛弃的自证有效根都没有」，那一拒的判据在崩溃镜像上复算为真（按最新那条根的实例表，
+    /// 环里一条不被抛弃的自证有效根都没有）——没有读故障的历史留不下这份盘面（读阶段择得到根、它按自己的实例表不被抛弃），正确实现在它上面
+    /// 永远拒可写；拒是对的、造出这份盘面的那一段不对。
+    NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,
 }
 
 /// 可写挂载被拒，拒得合不合法（r5 判决第二节 U6：拒绝合法 ⇔ 理由在 D23 已定项 14 那张单子上、那一条判据在崩溃镜像上复算为真、
@@ -2647,10 +2694,12 @@ pub fn judge_writable_mount_refusal_of_a_crash_state(
 /// （`read_faults`）、理由是判据 N-配置，而判据在崩溃镜像上复算、撕裂的系统配置槽跳过之后仍为真（[`WitnessedCounterBranchThatHolds`]），
 /// 拒得不合法（[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）；与它同形的改法 G4——崩溃态没注入读故障、
 /// 理由是「读阶段判完之后系统配置又读不出」，而那一拒的判据在崩溃镜像上复算为真，拒得不合法
-/// （[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）。注入过读故障的崩溃态
-/// 这两条都不判（D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。理由是「读阶段判完之后系统配置又读不出」而被拒的那次挂载自己在读故障下跑的
-/// （`read_faults_under_the_refused_mount`），判据在崩溃镜像上复算为假不算不合法（两遍读之间的瞬时读错复算不出，C664（挂载拒绝合法复算把两种拒因
-/// 报成一个））；被拒的那次挂载不带故障层的，崩溃态注没注入过读故障都照 U6 复算（判据为假即不合法，改法 G3）。
+/// （[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）；与 G4 同形的 Q3——
+/// 崩溃态没注入读故障、理由是「重建分配器读根环整环重读一次仍一条不被抛弃的自证有效根都没有」，而那一拒的判据在崩溃镜像上复算为真，拒得不合法
+/// （[`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）。注入过读故障的崩溃态
+/// 这三条都不判（D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。理由是「读阶段判完之后系统配置又读不出」或 Q3 那一拒而被拒的那次挂载
+/// 自己在读故障下跑的（`read_faults_under_the_refused_mount`），判据在崩溃镜像上复算为假不算不合法（两遍读之间的瞬时读错复算不出，C664（挂载拒绝合法复算
+/// 把两种拒因报成一个））；被拒的那次挂载不带故障层的，崩溃态注没注入过读故障都照 U6 复算（判据为假即不合法，改法 G3）。
 #[must_use]
 pub fn judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
     crash_image: &dyn PoolReader,
@@ -2699,9 +2748,23 @@ pub fn judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_und
                 );
             }
         }
+        // Q3 与 G4 同形：没有读故障时设备不报读错，读阶段择得到的根按它自己的实例表不被抛弃——这一拒的判据为真只能是盘面本身如此。
+        (
+            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ) => {
+            if no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(
+                crash_image,
+            ) {
+                conditions_not_met.push(
+                    RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,
+                );
+            }
+        }
         (
             RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
-            | RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+            | RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
+            | RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
             ReadFaultsInTheCrashState::Injected,
         )
         | (
@@ -2712,8 +2775,10 @@ pub fn judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_und
     // 读阶段之后系统配置又读不出、被拒的那次挂载自己在读故障下跑：两遍读之间的瞬时读错在崩溃镜像上复算不出，判据复算为假不算拒得不合法
     // （C664（挂载拒绝合法复算把两种拒因报成一个）的还清条件：带读故障的档里取号那一刻瞬时读错的态，分开报之后判合法）；
     // 盘上逐字节不变与只读读出照判。被拒的那次挂载不带故障层的照 U6 复算（判据为假即不合法）。
+    // 重建分配器读根环重读仍一条有效根都没有（Q3）同一个办法：重建那一遍与它的重读读坏是这次挂载里的瞬时读错，崩溃镜像上复算不出。
     match reason {
-        RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage => {
+        RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
+        | RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator => {
             if read_faults_under_the_refused_mount
                 .can_hide_a_transient_read_error_from_the_crash_image()
             {
@@ -2747,6 +2812,11 @@ fn conditions_not_met_by_a_refusal_on_the_d23_list(
         RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage => {
             system_configuration_unreadable_after_the_read_stage_on_the_crash_image(crash_image)
         }
+        RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator => {
+            no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(
+                crash_image,
+            )
+        }
     };
     let mut conditions_not_met = Vec::new();
     if !criterion_holds_on_the_crash_image {
diff --git a/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs b/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
index 701abb9..1b888b4 100644
--- a/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
+++ b/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
@@ -490,7 +490,9 @@ fn without_the_last_record_of_the_selected_version_the_record_at_the_witnessed_c
     }
 }
 
-/// 判据的第一支：只做过 mkfs 的池两块盘的系统配置 tail 都是 0 ⇒ 没有见证，判假、不重读。
+/// 判据的第一支：只做过 mkfs 的池两块盘的系统配置 tail 都是 0 ⇒ 没有见证，判假、读阶段不重读。
+/// 重建分配器读根环那一遍照 Q4 rr2（用户 2026-10-05 定，岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 第 4 行）整环重读一次：
+/// 只做过 mkfs 的环里只有 mkfs 那一条根，落在一块盘上，另一块盘一条有效根都没有。
 #[test]
 fn the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread() {
     let mut formatted = format_pool("c554-yi-formatted-pool-nothing-witnessed");
@@ -513,7 +515,7 @@ fn the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_n
                 },
             },
             instance_table_of_the_newest_root: InstanceTableOfTheNewestRootRead::OnTheFirstRead,
-            root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,
+            root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,
         }
     );
 }
diff --git a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
index 420a8b6..0a1c347 100644
--- a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
+++ b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
@@ -1,5 +1,7 @@
 //! 重建分配器读根环那一遍一条不被抛弃的自证有效根都没读出（择根那几遍读得出）时，把根环整环再读一遍，根环表、回收门槛与影子账用重读的读数；
-//! 重读还是一条都没有，就在任何写之前拒这次可写挂载（`MountError::Recovery(RecoveryFailure::NoValidRoot)`）。用户 2026-10-05 定「重读」：
+//! 重读还是一条都没有，就在任何写之前拒这次可写挂载（`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，用户 2026-10-05 定
+//! Q3 另起拒因，之前报 `Recovery(NoValidRoot)`）。只有一块盘没读出时也整环重读（Q4 rr2）的用例在
+//! `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs`。用户 2026-10-05 定「重读」：
 //! 岔路单 `research/prompts/m3-batch17-code-r1-forks.md` 第 2 行，判决 `research/prompts/m3-batch17-code-r1-main-verification.md`
 //! 第二节「N1 甲二」那一行与第四节。不重读、给空表往下走时，写行那次记根算回收门槛只看得到它自己，释放代高于 F_生效 的一截当场放开、被暖机写掉，
 //! 盘上环里更旧的有效根还引用着（攻方 `research/prompts/m3-batch17-code-r1-opus-output.md` 第 1.3 节）。
@@ -36,7 +38,7 @@ use singlefs_core::mount::{
 use singlefs_core::mounted_read::mount_read_only;
 use singlefs_core::mounted_session::MountedSession;
 use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
-use singlefs_core::recovery::RecoveryFailure;
+use singlefs_core::recovery::{recover, JournalPolicy};
 use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
 use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
 use singlefs_core::transaction::{publish_unlink_intent, PoolWriter, FIRST_INODE_NUMBER};
@@ -50,6 +52,11 @@ use singlefs_harness::fault_injection::{
 };
 use singlefs_harness::history::HistoryDeviceWidth;
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use singlefs_harness::mount_transparency::{
+    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount,
+    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadFaultsUnderTheRefusedMount,
+    ReadOnlyReadBackOfTheCrashImage, RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
+};
 use std::cell::RefCell;
 use std::collections::BTreeMap;
 use std::ops::RangeInclusive;
@@ -484,7 +491,7 @@ fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
 
 /// 只坏重建分配器那一遍（两块盘全部根环槽报读错）：重读那一遍读得出，挂载照常可写，交回与不注错的再挂同一份盘面（判定不变）；
 /// 两块盘都在时池级 checker 一条都不判违例（I-7.4「近 K 代块未被复用」在内）。
-/// 判别力：删掉重读（第一遍读不出就拒）时第一项红（`Recovery(NoValidRoot)`）；不重读、照旧给空表往下走时，写行那次记根把释放代高于 F_生效 的一截
+/// 判别力：删掉重读（第一遍读不出就拒）时第一项红（Q3 那一拒）；不重读、照旧给空表往下走时，写行那次记根把释放代高于 F_生效 的一截
 /// 当场放开、暖机写进盘上环里更旧的有效根（txg 20）还引用的单元，写行之后那次准入抬 F 读不出它的树表，挂载落了盘之后报
 /// `FloorRaiseFailedAfterTheMountsPublishes`（攻方第 1.3 节那一形；变异表里这一刀量过，`UnitUnreadable { slot: SlotNumber(1442) }`），第一项同样红。
 #[test]
@@ -533,9 +540,11 @@ fn only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_
     );
 }
 
-/// 重建分配器那一遍与它的重读都坏（两块盘全部根环槽报读错）：在任何写之前拒（`Recovery(NoValidRoot)`），盘面与挂载之前逐字节相同。
+/// 重建分配器那一遍与它的重读都坏（两块盘全部根环槽报读错）：在任何写之前拒（Q3 另起的拒因
+/// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`），盘面与挂载之前逐字节相同；拒因在 D23 已定项 14 那张单子上，
+/// 被拒的那次挂载在读故障下跑，崩溃镜像上判据复算为假不算不合法，判官判它合法。
 /// 判别力：重读还读不出时不拒、照旧给空表往下写，挂载写了盘（写行、暖机）之后报 `FloorRaiseFailedAfterTheMountsPublishes`（变异表里这一刀量过），
-/// 第一项红。
+/// 第一项红；拒因照旧报 `Recovery(NoValidRoot)` 时第一项红，判官把它排在单子外、最后一项也红。
 #[test]
 fn the_allocator_rebuild_pass_and_its_reread_both_failing_refuse_the_writable_mount_before_any_write(
 ) {
@@ -545,9 +554,9 @@ fn the_allocator_rebuild_pass_and_its_reread_both_failing_refuse_the_writable_mo
     assert!(
         matches!(
             remounted,
-            Err(MountError::Recovery(RecoveryFailure::NoValidRoot))
+            Err(MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread)
         ),
-        "重建分配器那一遍与重读都一条有效根读不出：拒这次可写挂载（Recovery(NoValidRoot)）：{:?}",
+        "重建分配器那一遍与重读都一条有效根读不出：拒这次可写挂载（Q3 那一拒）：{:?}",
         remounted.as_ref().map(|_| "做成了")
     );
     assert!(
@@ -567,10 +576,28 @@ fn the_allocator_rebuild_pass_and_its_reread_both_failing_refuse_the_writable_mo
                 .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD + 1),
         "重读那一遍之后根环再没被读过（拒在重读之后、取号之前）：{reads_of_each_slot:?}"
     );
+    let Err(refusal) = remounted else {
+        panic!("上面判过这次挂载拒了");
+    };
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
+            &history.image_after_the_crashed_unmount,
+            recover(&history.image_after_the_crashed_unmount, JournalPolicy::Consult).effective_root,
+            &refusal,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+            ReadFaultsInTheCrashState::Injected,
+            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadThroughInjectedFaults,
+        ),
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator
+        ),
+        "拒因在 D23 已定项 14 那张单子上；被拒的那次挂载在读故障下跑，崩溃镜像上判据复算为假不算不合法：判合法"
+    );
 }
 
 /// 只坏重建分配器那一遍（两块盘全部根环槽报读错），重读之前的钩子（`BeforeTheOneReread::CallTheTestOnlyHookFirst`）让根环此后每次读都坏：
-/// 重读那一遍也一条有效根读不出，在任何写之前拒（`Recovery(NoValidRoot)`），两块盘与挂载之前逐字节相同。钩子恰调一次，调的那一刻每个根环槽
+/// 重读那一遍也一条有效根读不出，在任何写之前拒（Q3 那一拒），两块盘与挂载之前逐字节相同。钩子恰调一次，调的那一刻每个根环槽
 /// 恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次（重建那一遍之后、重读之前）。
 /// 这个挂载入口不收删除开关、取产品值：拒在建分配器那一步，删除开关在它之后才装上，挂载读根环的次序与别的用例相同（读次数那两条断言现核）。
 /// 判别力：重建分配器整环重读之前不调钩子时，重读那一遍照常读得出、挂载做成，第一项红。
@@ -606,9 +633,9 @@ fn a_hook_before_the_root_ring_reread_failing_that_reread_too_refuses_the_writab
     assert!(
         matches!(
             remounted,
-            Err(MountError::Recovery(RecoveryFailure::NoValidRoot))
+            Err(MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread)
         ),
-        "钩子让重读那一遍也读坏：重建分配器那一遍与重读都一条有效根读不出，拒这次可写挂载（Recovery(NoValidRoot)）：{:?}",
+        "钩子让重读那一遍也读坏：重建分配器那一遍与重读都一条有效根读不出，拒这次可写挂载（Q3 那一拒）：{:?}",
         remounted.as_ref().map(|_| "做成了")
     );
     assert!(
@@ -788,12 +815,12 @@ fn only_the_allocator_rebuild_pass_failing_and_a_crash_before_the_warm_up_root_l
 }
 
 /// 重读那一遍只有一部分槽读坏（交主 agent 的那一格，能造的一形）：重建分配器那一遍两块盘全部根环槽报读错，重读那一遍盘 1 的根环槽仍报读错、
-/// 盘 0 的读得出。重读的读数与「重建那一遍本身只坏盘 1」同等对待：挂载做成、交回的盘面与只坏盘 1 那一遍的再挂（不重读）逐字节相同，
-/// 两块盘都在时池级 checker 不判违例。
-/// 判别力：删掉重读时第一项红（`Recovery(NoValidRoot)`）；不重读、照旧给空表往下走时同上一形，挂载落了盘之后报
+/// 盘 0 的读得出。重读的读数与「重建那一遍与重读都只坏盘 1」（rr2 之后只坏盘 1 的那一遍也整环重读，Q4）同等对待：挂载做成、交回的盘面与
+/// 两遍都只坏盘 1 的再挂逐字节相同（两者都重读过、重读那一遍都只读出盘 0 的根，照 m 取不到那一支往下走），两块盘都在时池级 checker 不判违例。
+/// 判别力：删掉重读时第一项红（Q3 那一拒）；不重读、照旧给空表往下走时同上一形，挂载落了盘之后报
 /// `FloorRaiseFailedAfterTheMountsPublishes`，第一项红。
 #[test]
-fn a_reread_still_failing_on_device_one_hands_over_what_a_rebuild_pass_failing_only_on_device_one_hands_over(
+fn a_reread_still_failing_on_device_one_hands_over_what_both_passes_failing_only_on_device_one_hand_over(
 ) {
     let history = narrow_pool_after_a_crashed_unmount();
     let rebuild = READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD;
@@ -823,15 +850,15 @@ fn a_reread_still_failing_on_device_one_hands_over_what_a_rebuild_pass_failing_o
     };
     let reads_failing_only_on_device_one = SharedRootRingReads::new(
         history.root_ring_slots_of(DeviceIdentity(1)),
-        rebuild..=rebuild,
+        rebuild..=rebuild + 1,
     );
     let (remounted_failing_only_on_device_one, image_after_the_remount_failing_only_on_device_one) =
         history.remount_reading_the_root_ring_through(&reads_failing_only_on_device_one);
     let mounted_failing_only_on_device_one = remounted_failing_only_on_device_one
-        .unwrap_or_else(|error| panic!("重建那一遍只坏盘 1：照常可写挂载：{error:?}"));
+        .unwrap_or_else(|error| panic!("重建那一遍与重读都只坏盘 1：照常可写挂载：{error:?}"));
     assert!(
         image_after_the_remount == image_after_the_remount_failing_only_on_device_one,
-        "重读那一遍只坏盘 1 的再挂与重建那一遍只坏盘 1 的再挂交回逐字节相同的盘面（重读的读数与第一遍的读数同等对待）"
+        "重读那一遍只坏盘 1 的再挂与两遍都只坏盘 1 的再挂交回逐字节相同的盘面（重读的读数一样，判定一样）"
     );
     assert_eq!(
         (
@@ -843,9 +870,9 @@ fn a_reread_still_failing_on_device_one_hands_over_what_a_rebuild_pass_failing_o
         ),
         (
             RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,
-            RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead
+            RootRingOfTheRebuiltAllocatorRead::OnTheOneReread
         ),
-        "前一次重读过、后一次第一遍就读出盘 0 的根"
+        "两次都重读过（后一次第一遍盘 1 没读出有效根，rr2 照样重读）"
     );
     assert_eq!(
         (
diff --git b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
new file mode 100644
index 0000000..6631322
--- /dev/null
+++ b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
@@ -0,0 +1,785 @@
+//! 重建分配器读根环那一遍只要有一块盘一条不被抛弃的自证有效根都没读出，就把根环整环再读一遍（重读之前照旧调 `BeforeTheOneReread` 钩子），
+//! 根环表、回收门槛与影子账用重读的读数（用户 2026-10-05 定 Q4「rr2」，岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 第 4 行，
+//! 判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）。改之前只在整个环一条都没读出时重读：只坏重建那一遍、
+//! 只有一块盘没读出的那一形 m 取不到、当场只放到环里最旧有效根，窄盘上写行取不到落点，在取号之前被拒
+//! （`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`；攻方第二次抽样 `research/prompts/m3-batch10-code-r1-opus-s2-output.md`
+//! K2-3 与「改法」一节：拒 36 → 28，降的正是这 8 段）。
+//!
+//! 起点照攻方那两个（造法照攻方原型 `research/prompts/m3-batch17-code-r1-opus-model/probe/main.rs` 的 `narrow_start`，只抄造法）：
+//! - `192:9:3:1` n = 8：不补环窗口、准入关着造的窄盘（单元区 192 槽），第一个文件之后覆盖写 9 次、顺序写长到 3 个单元，发 unlink 的意图、
+//!   删一批就崩；重开可写（准入判着、影子账开、批大小 1）续做走完；卸载落下 8 个写就崩；两块盘都在、再可写挂载。
+//! - `216:7:5:4` n = now：单元区 216 槽，覆盖写 7 次、长到 5 个单元，意图之后删 4 批就崩；重开可写续做走完那一刻就崩（不卸载）；再可写挂载。
+//!
+//! 读坏计划照攻方那四份（`P5:all/-:e`、`P5:-/all:e`、`P5:but1/all:z`、`P5:all/but1:g`）：只坏重建分配器读根环那一遍（根环每个槽按它自己
+//! 被读的次数从 1 数，第 6 次是那一遍：钩子调的那一刻每个槽恰读过 6 次现核），点名的槽这一次报读错（e）、读回全 0（z）或读回不是全 0 而自证不过的
+//! 乱字节（g）；`all` 是那块盘上的每个根环槽，`but1` 是除了那块盘上最新那条自证过的根所在的槽之外的每个根环槽。
+
+mod common_admission;
+
+use common_admission::{
+    checker_violations_on, plain_devices_on,
+    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
+    PoolUnderTest, OVERWRITE_BYTES,
+};
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
+};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::{
+    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+};
+use singlefs_core::container_tree::ContainerTreeNodeEntries;
+use singlefs_core::lineage_tree::LineageContainerCapacity;
+use singlefs_core::mount::{
+    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
+    mount_writable_with_test_only_switches_and_deletion_switches, unmount, BeforeTheOneReread,
+    MountError, Mounted, RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
+};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::mounted_session::MountedSession;
+use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
+use singlefs_core::recovery::{
+    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, RootRingSlotReading,
+};
+use singlefs_core::root_ring::{slot_offset, RootRingSlot};
+use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
+use singlefs_core::transaction::{
+    publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
+};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_core::unlink::{
+    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
+};
+use singlefs_harness::fault_injection::{
+    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
+    FaultSchedule, InjectedFault, SharedFaultPlan,
+};
+use singlefs_harness::history::HistoryDeviceWidth;
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use std::cell::RefCell;
+use std::collections::BTreeMap;
+use std::rc::Rc;
+
+/// 一次不注错的再挂里，根环每个槽第几次被读是重建分配器读根环那一遍（从 1 数；与
+/// `rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs` 那段历史同一个数，这里由钩子现核）。
+const READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD: u64 = 6;
+
+/// 读回乱字节那一类填的字节（攻方原型 `k.rs` 同一个值）：不是全 0，自证不过。
+const GARBAGE_BYTE: u8 = 0xA5;
+
+/// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
+fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
+    DeletionSwitches {
+        batch_size: UnlinkBatchSize::CappedForTests(
+            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
+        ),
+        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
+        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+    }
+}
+
+/// 重建那一遍点名的槽这一次读成什么。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum RootRingSlotReadFault {
+    /// 报块设备错（攻方计划的 `e`）。
+    ReadFails,
+    /// 报成功、读回全 0（`z`）。
+    ReadsBackAllZero,
+    /// 报成功、读回不是全 0 而自证不过的乱字节（`g`）。
+    ReadsBackGarbage,
+}
+
+/// 一块盘上重建那一遍点名哪几个根环槽。
+#[derive(Clone, Copy, Debug)]
+enum RootRingSlotsOfOneDevice {
+    /// `-`：一个都不点名。
+    NoSlot,
+    /// `all`：每一个。
+    Every,
+    /// `but1`：除了这块盘上最新那条自证过的根所在的槽之外的每一个。
+    EveryButTheSlotOfTheNewestRoot,
+}
+
+/// 攻方的一份读坏计划：两块盘各点名哪几个槽、点名的槽读成什么。
+#[derive(Clone, Copy, Debug)]
+struct RebuildPassPlan {
+    device_zero: RootRingSlotsOfOneDevice,
+    device_one: RootRingSlotsOfOneDevice,
+    fault: RootRingSlotReadFault,
+}
+
+/// `P5:all/-:e`
+const DEVICE_ZERO_REPORTS_READ_ERRORS: RebuildPassPlan = RebuildPassPlan {
+    device_zero: RootRingSlotsOfOneDevice::Every,
+    device_one: RootRingSlotsOfOneDevice::NoSlot,
+    fault: RootRingSlotReadFault::ReadFails,
+};
+/// `P5:-/all:e`
+const DEVICE_ONE_REPORTS_READ_ERRORS: RebuildPassPlan = RebuildPassPlan {
+    device_zero: RootRingSlotsOfOneDevice::NoSlot,
+    device_one: RootRingSlotsOfOneDevice::Every,
+    fault: RootRingSlotReadFault::ReadFails,
+};
+/// `P5:but1/all:z`
+const DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS: RebuildPassPlan =
+    RebuildPassPlan {
+        device_zero: RootRingSlotsOfOneDevice::EveryButTheSlotOfTheNewestRoot,
+        device_one: RootRingSlotsOfOneDevice::Every,
+        fault: RootRingSlotReadFault::ReadsBackAllZero,
+    };
+/// `P5:all/but1:g`
+const DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE: RebuildPassPlan =
+    RebuildPassPlan {
+        device_zero: RootRingSlotsOfOneDevice::Every,
+        device_one: RootRingSlotsOfOneDevice::EveryButTheSlotOfTheNewestRoot,
+        fault: RootRingSlotReadFault::ReadsBackGarbage,
+    };
+
+/// 根环每个槽被读的次数与重建那一遍点名的槽（几块盘共用一份）。
+struct RootRingReadsState {
+    slot_spacing_in_bytes: u64,
+    /// 每个根环槽在哪块盘、盘内从哪个偏移起。
+    slots: Vec<(RootRingSlot, DeviceIdentity, u64)>,
+    faulted_slots: BTreeMap<RootRingSlot, RootRingSlotReadFault>,
+    reads_of_each_slot: BTreeMap<RootRingSlot, u64>,
+    faulted_reads: u64,
+}
+
+#[derive(Clone)]
+struct SharedRootRingReads(Rc<RefCell<RootRingReadsState>>);
+
+impl SharedRootRingReads {
+    /// 这一次读落在哪个根环槽里就给那个槽记一次；是重建那一遍点名的槽，交回这一次读成什么与那个槽在盘内的起点。
+    fn note_a_read_and_tell_its_fault(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+    ) -> Option<(RootRingSlotReadFault, u64)> {
+        let mut state = self.0.borrow_mut();
+        let spacing = state.slot_spacing_in_bytes;
+        let (ring_slot, slot_start) = state
+            .slots
+            .iter()
+            .find(|(_, slot_device, start)| {
+                *slot_device == device && offset.0 >= *start && offset.0 < *start + spacing
+            })
+            .map(|(ring_slot, _, start)| (*ring_slot, *start))?;
+        let reads_of_this_slot = {
+            let reads = state.reads_of_each_slot.entry(ring_slot).or_insert(0);
+            *reads += 1;
+            *reads
+        };
+        if reads_of_this_slot != READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD {
+            return None;
+        }
+        let fault = *state.faulted_slots.get(&ring_slot)?;
+        state.faulted_reads += 1;
+        Some((fault, slot_start))
+    }
+
+    fn reads_of_each_slot(&self) -> BTreeMap<RootRingSlot, u64> {
+        self.0.borrow().reads_of_each_slot.clone()
+    }
+
+    fn faulted_reads(&self) -> u64 {
+        self.0.borrow().faulted_reads
+    }
+}
+
+/// 读的时候按 [`SharedRootRingReads`] 让重建那一遍点名的根环槽读坏的盘；写、写零、屏障原样交给里面那块盘。
+struct DeviceFailingTheRebuildPass {
+    identity: DeviceIdentity,
+    inner: SparseBlockDevice,
+    root_ring_reads: SharedRootRingReads,
+}
+
+impl BlockDevice for DeviceFailingTheRebuildPass {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let fault = self
+            .root_ring_reads
+            .note_a_read_and_tell_its_fault(self.identity, offset);
+        let filler = match fault {
+            None => return self.inner.read_at(offset, buffer),
+            Some((RootRingSlotReadFault::ReadFails, _)) => {
+                return Err(BlockDeviceError::InputOutput(std::io::Error::other(
+                    "用例点名的这一次根环槽读坏",
+                )));
+            }
+            Some((RootRingSlotReadFault::ReadsBackAllZero, slot_start)) => (0u8, slot_start),
+            Some((RootRingSlotReadFault::ReadsBackGarbage, slot_start)) => {
+                (GARBAGE_BYTE, slot_start)
+            }
+        };
+        self.inner.read_at(offset, buffer)?;
+        let (byte, slot_start) = filler;
+        let spacing = self.root_ring_reads.0.borrow().slot_spacing_in_bytes;
+        let first_byte_in_the_buffer = usize::try_from(slot_start.saturating_sub(offset.0))
+            .expect("缓冲区里的偏移装得进 usize");
+        let end_in_the_buffer = usize::try_from(
+            (slot_start + spacing - offset.0).min(u64::try_from(buffer.len()).expect("缓冲区长度")),
+        )
+        .expect("缓冲区里的偏移装得进 usize");
+        buffer[first_byte_in_the_buffer..end_in_the_buffer].fill(byte);
+        Ok(())
+    }
+
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        self.inner.write_at(offset, bytes, durability)
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.inner.write_zeroes_at(offset, length)
+    }
+
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        self.inner.barrier()
+    }
+
+    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+        self.inner.probe_physical_block_size()
+    }
+
+    fn size_in_bytes(&self) -> u64 {
+        self.inner.size_in_bytes()
+    }
+}
+
+/// 攻方的两个起点。
+#[derive(Clone, Copy, Debug)]
+enum StartingPoint {
+    /// `192:9:3:1` n = 8。
+    UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
+    /// `216:7:5:4` n = now。
+    UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
+}
+
+/// 再挂之前的盘面，与「卸载前」那一份（丢盘整读拿它比；n = now 那一个起点两份相同）。
+struct PoolBeforeTheRemount {
+    image_before_the_crash: MemoryPool,
+    image_to_remount: MemoryPool,
+    parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
+}
+
+/// 整池从第 `first_failing_write` 次写起每次写都报错的注错计划（掉电那一形：盘停在之前那几个写之后）。
+fn plan_failing_every_write_from(
+    device_width: HistoryDeviceWidth,
+    first_failing_write: u64,
+) -> SharedFaultPlan {
+    SharedFaultPlan::armed(
+        device_width.fixed_geometry(),
+        FaultSchedule {
+            fault: InjectedFault::WriteFails,
+            device: FaultDeviceSelector::EveryDevice,
+            placement: FaultPlacement::AnyOffset,
+            counting: FaultCounting::AcrossThePool,
+            occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(first_failing_write),
+        },
+    )
+}
+
+/// 窄盘：第一个文件之后覆盖写 `overwrites` 次、顺序写长到 `data_units_of_the_file` 个单元，在这个进程里发 unlink 的意图、
+/// 再删 `batches_after_the_intent` 批就崩；重开可写（准入判着、影子账开、批大小 1）续做走完，会话挂回池上。
+fn narrow_pool_whose_deletion_was_continued(
+    unit_area_slots: u64,
+    overwrites: u64,
+    data_units_of_the_file: usize,
+    batches_after_the_intent: u64,
+) -> PoolUnderTest<SparseBlockDevice> {
+    let mut pool =
+        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            unit_area_slots,
+        );
+    for overwrite_index in 0..overwrites {
+        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
+        });
+    }
+    pool.sequential_write((data_units_of_the_file - 1) * data_unit_payload_capacity() + 100)
+        .unwrap_or_else(|refusal| {
+            panic!("顺序写把文件长到 {data_units_of_the_file} 个单元：{refusal:?}")
+        });
+    {
+        let PoolUnderTest {
+            session,
+            devices,
+            parameters,
+            write_time_seconds,
+            ..
+        } = &mut pool;
+        let session = session.as_mut().expect("这一刻有可写会话");
+        let mut current = session
+            .current
+            .version_of_trees()
+            .expect("第一个文件之后有树")
+            .clone();
+        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
+        current = publish_unlink_intent(
+            &mut writer,
+            &mut session.allocator,
+            &current,
+            InodeNumber(FIRST_INODE_NUMBER),
+            *write_time_seconds,
+            session.instance,
+        )
+        .expect("意图那次");
+        // 迭代上界是 `batches_after_the_intent`；跨轮携带的是现行那一版。
+        for batch_index in 0..batches_after_the_intent {
+            current = publish_unlink_step(
+                &mut writer,
+                &mut session.allocator,
+                &current,
+                InodeNumber(FIRST_INODE_NUMBER),
+                deletion_switches_with_batches_of_one().batch_size,
+                session.instance,
+            )
+            .unwrap_or_else(|refusal| panic!("第 {batch_index} 批：{refusal:?}"));
+        }
+    }
+    pool.session = None;
+    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
+        &pool.parameters,
+        &mut pool.devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    )
+    .unwrap_or_else(|error| panic!("崩了之后重开可写、续做：{error:?}"));
+    let (_, session_of_the_continued_mount) = MountedSession::of_the_mount(mounted);
+    pool.session = Some(session_of_the_continued_mount);
+    pool
+}
+
+fn pool_before_the_remount(starting_point: StartingPoint) -> PoolBeforeTheRemount {
+    match starting_point {
+        StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites => {
+            let unmount_writes_landed = 8;
+            let pool = narrow_pool_whose_deletion_was_continued(192, 9, 3, 1);
+            let image_before_the_unmount = pool.image();
+            let write_plan = plan_failing_every_write_from(
+                HistoryDeviceWidth::UnitAreaOf384Slots,
+                unmount_writes_landed + 1,
+            );
+            let mut unmount_devices: Vec<(
+                DeviceIdentity,
+                FaultInjectingBlockDevice<SparseBlockDevice>,
+            )> = plain_devices_on(&image_before_the_unmount)
+                .into_iter()
+                .map(|(identity, device)| {
+                    (
+                        identity,
+                        FaultInjectingBlockDevice::new(identity, device, write_plan.clone()),
+                    )
+                })
+                .collect();
+            let session = pool.session();
+            let mut allocator = session.allocator.clone();
+            let mut current = session.current.clone();
+            let unmounted = unmount(
+                &pool.parameters,
+                &mut unmount_devices,
+                &mut allocator,
+                &mut current,
+                session.shadow_ledger,
+            );
+            assert!(
+                unmounted.is_err(),
+                "前提：卸载在第 {} 个写上掉电、报错",
+                unmount_writes_landed + 1
+            );
+            let image_after_the_crashed_unmount = MemoryPool {
+                devices: unmount_devices
+                    .iter()
+                    .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
+                    .collect(),
+                device_size_in_bytes: image_before_the_unmount.device_size_in_bytes,
+            };
+            PoolBeforeTheRemount {
+                image_before_the_crash: image_before_the_unmount,
+                image_to_remount: image_after_the_crashed_unmount,
+                parameters: pool.parameters.clone(),
+            }
+        }
+        StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount => {
+            let pool = narrow_pool_whose_deletion_was_continued(216, 7, 5, 4);
+            let image = pool.image();
+            PoolBeforeTheRemount {
+                image_before_the_crash: image.clone(),
+                image_to_remount: image,
+                parameters: pool.parameters.clone(),
+            }
+        }
+    }
+}
+
+/// 照计划点名的根环槽（`but1` 按这份盘面上每块盘最新那条自证过的根认）。
+fn faulted_slots_of_the_plan(
+    image: &MemoryPool,
+    plan: RebuildPassPlan,
+) -> BTreeMap<RootRingSlot, RootRingSlotReadFault> {
+    let system_configuration = choose_system_configuration(image).expect("系统配置");
+    let immutable = system_configuration.immutable;
+    let device_of = |ring_slot: RootRingSlot| {
+        immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")]
+    };
+    let ring_slots = every_root_ring_slot(&immutable.sizes);
+    let mut slot_of_the_newest_root_on_each_device: BTreeMap<
+        DeviceIdentity,
+        (CheckpointTxg, RootRingSlot),
+    > = BTreeMap::new();
+    // 迭代上界是根环的槽数；没有提前出口。
+    for ring_slot in &ring_slots {
+        if let RootRingSlotReading::SelfVerified(root) = read_root_ring_slot(
+            image,
+            &immutable.region_devices,
+            &immutable.sizes,
+            &immutable.filesystem_identifier,
+            *ring_slot,
+        ) {
+            let newest = slot_of_the_newest_root_on_each_device
+                .entry(device_of(*ring_slot))
+                .or_insert((root.checkpoint_txg, *ring_slot));
+            if root.checkpoint_txg > newest.0 {
+                *newest = (root.checkpoint_txg, *ring_slot);
+            }
+        }
+    }
+    ring_slots
+        .into_iter()
+        .filter(|ring_slot| {
+            let device = device_of(*ring_slot);
+            let slots_of_the_device = if device == DeviceIdentity(0) {
+                plan.device_zero
+            } else {
+                plan.device_one
+            };
+            match slots_of_the_device {
+                RootRingSlotsOfOneDevice::NoSlot => false,
+                RootRingSlotsOfOneDevice::Every => true,
+                RootRingSlotsOfOneDevice::EveryButTheSlotOfTheNewestRoot => {
+                    slot_of_the_newest_root_on_each_device
+                        .get(&device)
+                        .is_none_or(|(_, newest_slot)| newest_slot != ring_slot)
+                }
+            }
+        })
+        .map(|ring_slot| (ring_slot, plan.fault))
+        .collect()
+}
+
+/// 只读挂载、把挂载态里每个活着的文件从头读到尾：每个活文件（inode 号、全部字节）。删除中的 inode（nlink = 0）不读。
+fn live_files_read_whole(image: &MemoryPool) -> Result<Vec<(u64, Vec<u8>)>, String> {
+    let read_only = mount_read_only(image).map_err(|failure| format!("只读挂载：{failure:?}"))?;
+    let mut live_files = Vec::new();
+    // 迭代上界是这一版 inode 树的记录数；提前出口只有读不出的那三种错。
+    for record in read_only.mounted.inode_records() {
+        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
+            continue;
+        }
+        let file = read_only
+            .mounted
+            .open_file(image, InodeNumber(record.inode))
+            .map_err(|failure| format!("打开 inode {}：{failure:?}", record.inode))?;
+        let read = file
+            .read_at(image, FileOffsetInBytes(0), record.size)
+            .map_err(|failure| format!("读 inode {}：{failure:?}", record.inode))?;
+        live_files.push((record.inode, read.bytes));
+    }
+    Ok(live_files)
+}
+
+/// 一份镜像去掉一块盘。
+fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
+    MemoryPool {
+        devices: image
+            .devices
+            .iter()
+            .filter(|(identity, _)| **identity != lost)
+            .map(|(identity, device)| (*identity, device.clone()))
+            .collect(),
+        device_size_in_bytes: image.device_size_in_bytes,
+    }
+}
+
+/// 照计划只坏重建那一遍的再挂，看到的那几样（各用例自己断言）。
+#[derive(Debug, PartialEq, Eq)]
+struct RemountUnderThePlan {
+    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
+    root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
+    /// 重读之前的钩子调了几次，每次调的那一刻每个根环槽是不是都恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次。
+    hook_calls_after_exactly_the_rebuild_pass: Vec<bool>,
+    /// 这一次读坏的恰是计划点名的那几个槽、各一次。
+    faulted_reads_are_the_named_slots_once: bool,
+    /// 交回的盘面与同一份盘面上不注错的再挂交回的逐字节相同。
+    same_disk_as_a_remount_without_faults: bool,
+    /// 两块盘都在时池级 checker 判违例的那几条。
+    checker_violations_with_both_devices: Vec<String>,
+    /// 丢盘 0、丢盘 1 之后剩下那块盘只读挂载、整读每个活文件，与崩溃之前同一块盘上读出的相同。
+    lone_survivor_reads_what_it_read_before_the_crash: [bool; 2],
+}
+
+/// 再挂一次（准入判着、影子账开），读盘经 `root_ring_reads`；重读之前调钩子记下那一刻每个根环槽读过几次。
+fn remount_through(
+    before: &PoolBeforeTheRemount,
+    root_ring_reads: &SharedRootRingReads,
+) -> (
+    Result<Mounted, MountError>,
+    MemoryPool,
+    Vec<BTreeMap<RootRingSlot, u64>>,
+) {
+    let mut devices: Vec<(DeviceIdentity, DeviceFailingTheRebuildPass)> =
+        plain_devices_on(&before.image_to_remount)
+            .into_iter()
+            .map(|(identity, device)| {
+                (
+                    identity,
+                    DeviceFailingTheRebuildPass {
+                        identity,
+                        inner: device,
+                        root_ring_reads: root_ring_reads.clone(),
+                    },
+                )
+            })
+            .collect();
+    let reads_seen_by_the_hook = root_ring_reads.clone();
+    let mut reads_of_each_slot_at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>> = Vec::new();
+    let mut note_the_reads = || {
+        reads_of_each_slot_at_each_hook_call.push(reads_seen_by_the_hook.reads_of_each_slot());
+    };
+    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
+        &before.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut note_the_reads),
+    );
+    let image = MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.inner.image.clone()))
+            .collect(),
+        device_size_in_bytes: before.image_to_remount.device_size_in_bytes,
+    };
+    (mounted, image, reads_of_each_slot_at_each_hook_call)
+}
+
+fn root_ring_reads_failing(
+    image: &MemoryPool,
+    faulted_slots: BTreeMap<RootRingSlot, RootRingSlotReadFault>,
+) -> SharedRootRingReads {
+    let system_configuration = choose_system_configuration(image).expect("系统配置");
+    let immutable = system_configuration.immutable;
+    let slots = every_root_ring_slot(&immutable.sizes)
+        .into_iter()
+        .map(|ring_slot| {
+            (
+                ring_slot,
+                immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
+                slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing).0,
+            )
+        })
+        .collect();
+    SharedRootRingReads(Rc::new(RefCell::new(RootRingReadsState {
+        slot_spacing_in_bytes: u64::from(immutable.sizes.fixed_structure_slot_spacing),
+        slots,
+        faulted_slots,
+        reads_of_each_slot: BTreeMap::new(),
+        faulted_reads: 0,
+    })))
+}
+
+fn remount_under_the_plan(
+    starting_point: StartingPoint,
+    plan: RebuildPassPlan,
+) -> RemountUnderThePlan {
+    let before = pool_before_the_remount(starting_point);
+    let faulted_slots = faulted_slots_of_the_plan(&before.image_to_remount, plan);
+    let faulted_slot_count = u64::try_from(faulted_slots.len()).expect("槽数");
+    let root_ring_reads = root_ring_reads_failing(&before.image_to_remount, faulted_slots);
+    let (mounted, image_after_the_remount, reads_at_each_hook_call) =
+        remount_through(&before, &root_ring_reads);
+    let (mounted_without_faults, image_after_a_remount_without_faults, _) = remount_through(
+        &before,
+        &root_ring_reads_failing(&before.image_to_remount, BTreeMap::new()),
+    );
+    mounted_without_faults
+        .unwrap_or_else(|error| panic!("前提：同一份盘面不注错地再可写挂载做得成：{error:?}"));
+    let lone_survivor_reads_what_it_read_before_the_crash = [DeviceIdentity(0), DeviceIdentity(1)]
+        .map(|lost| {
+            let before_the_crash =
+                live_files_read_whole(&image_without(&before.image_before_the_crash, lost));
+            before_the_crash.is_ok()
+                && live_files_read_whole(&image_without(&image_after_the_remount, lost))
+                    == before_the_crash
+        });
+    RemountUnderThePlan {
+        root_ring_of_the_rebuilt_allocator: mounted
+            .map(|mounted| mounted.output.rereads.root_ring_of_the_rebuilt_allocator)
+            .map_err(|error| format!("{error:?}")),
+        hook_calls_after_exactly_the_rebuild_pass: reads_at_each_hook_call
+            .iter()
+            .map(|reads_of_each_slot| {
+                !reads_of_each_slot.is_empty()
+                    && reads_of_each_slot
+                        .values()
+                        .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD)
+            })
+            .collect(),
+        faulted_reads_are_the_named_slots_once: faulted_slot_count > 0
+            && root_ring_reads.faulted_reads() == faulted_slot_count,
+        same_disk_as_a_remount_without_faults: image_after_the_remount
+            == image_after_a_remount_without_faults,
+        checker_violations_with_both_devices: checker_violations_on(&image_after_the_remount),
+        lone_survivor_reads_what_it_read_before_the_crash,
+    }
+}
+
+/// 只坏重建那一遍的计划在这两个起点上的预期：照常挂上、重读过一次（钩子恰调一次，在重建那一遍之后），交回与不注错的再挂同一份盘面，
+/// 两块盘都在时池级 checker 不判违例，丢盘整读与崩溃之前相同。
+fn rereads_and_mounts_like_a_remount_without_faults() -> RemountUnderThePlan {
+    RemountUnderThePlan {
+        root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
+        hook_calls_after_exactly_the_rebuild_pass: vec![true],
+        faulted_reads_are_the_named_slots_once: true,
+        same_disk_as_a_remount_without_faults: true,
+        checker_violations_with_both_devices: Vec::new(),
+        lone_survivor_reads_what_it_read_before_the_crash: [true, true],
+    }
+}
+
+/// 场景：`192:9:3:1` n = 8，重建那一遍盘 0 的每个根环槽报读错（`P5:all/-:e`）。
+/// 预期：盘 0 一条有效根都没读出，整环重读一次（钩子恰调一次），重读读得出，照常可写挂载，交回与不注错的再挂同一份盘面；
+/// 两块盘都在时池级 checker 不判违例；丢盘整读与卸载之前相同。
+/// 判别力：拿掉「一块盘没读出也重读」（`crates/mutations.tsv`「第10g批 Q4 rr2…」）时 m 取不到、当场只放到环里最旧有效根，
+/// 窄盘上写行取不到落点、取号之前拒，第一项红。
+#[test]
+fn on_the_192_slot_pool_after_a_crashed_unmount_device_zero_failing_the_rebuild_pass_rereads_and_mounts(
+) {
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
+            DEVICE_ZERO_REPORTS_READ_ERRORS
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "只坏重建那一遍、盘 0 报读错：整环重读、照常挂上"
+    );
+}
+
+/// 场景：`192:9:3:1` n = 8，重建那一遍盘 1 的每个根环槽报读错（`P5:-/all:e`）。
+/// 预期与判别力同上一条。
+#[test]
+fn on_the_192_slot_pool_after_a_crashed_unmount_device_one_failing_the_rebuild_pass_rereads_and_mounts(
+) {
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
+            DEVICE_ONE_REPORTS_READ_ERRORS
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "只坏重建那一遍、盘 1 报读错：整环重读、照常挂上"
+    );
+}
+
+/// 场景：`192:9:3:1` n = 8，重建那一遍盘 0 除最新那条根之外、盘 1 每个根环槽读回全 0（`P5:but1/all:z`）。
+/// 预期与判别力同上（盘 1 一条有效根都没读出）。
+#[test]
+fn on_the_192_slot_pool_after_a_crashed_unmount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts(
+) {
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
+            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "只坏重建那一遍、盘 1 读回全 0：整环重读、照常挂上"
+    );
+}
+
+/// 场景：`192:9:3:1` n = 8，重建那一遍盘 0 每个、盘 1 除最新那条根之外的根环槽读回乱字节（`P5:all/but1:g`）。
+/// 预期与判别力同上（盘 0 一条有效根都没读出）。
+#[test]
+fn on_the_192_slot_pool_after_a_crashed_unmount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts(
+) {
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
+            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "只坏重建那一遍、盘 0 读回乱字节：整环重读、照常挂上"
+    );
+}
+
+/// 场景：`216:7:5:4` n = now，重建那一遍盘 0 的每个根环槽报读错（`P5:all/-:e`）。
+/// 预期与判别力同第一条。
+#[test]
+fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_failing_the_rebuild_pass_rereads_and_mounts(
+) {
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
+            DEVICE_ZERO_REPORTS_READ_ERRORS
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "只坏重建那一遍、盘 0 报读错：整环重读、照常挂上"
+    );
+}
+
+/// 场景：`216:7:5:4` n = now，重建那一遍盘 1 的每个根环槽报读错（`P5:-/all:e`）。
+/// 预期与判别力同第一条。
+#[test]
+fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_failing_the_rebuild_pass_rereads_and_mounts(
+) {
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
+            DEVICE_ONE_REPORTS_READ_ERRORS
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "只坏重建那一遍、盘 1 报读错：整环重读、照常挂上"
+    );
+}
+
+/// 场景：`216:7:5:4` n = now，重建那一遍盘 0 除最新那条根之外、盘 1 每个根环槽读回全 0（`P5:but1/all:z`）。
+/// 预期与判别力同第一条。
+#[test]
+fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts(
+) {
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
+            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "只坏重建那一遍、盘 1 读回全 0：整环重读、照常挂上"
+    );
+}
+
+/// 场景：`216:7:5:4` n = now，重建那一遍盘 0 每个、盘 1 除最新那条根之外的根环槽读回乱字节（`P5:all/but1:g`）。
+/// 预期与判别力同第一条。
+#[test]
+fn on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts(
+) {
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
+            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "只坏重建那一遍、盘 0 读回乱字节：整环重读、照常挂上"
+    );
+}
diff --git b/crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs b/crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs
new file mode 100644
index 0000000..6c219af
--- /dev/null
+++ b/crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs
@@ -0,0 +1,503 @@
+//! 原来同取一圈窗口的六处里另两处，用户 2026-10-04 定 RGall 一起退回整个环（D16（发布语义） 已定项 1；
+//! `research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）：影子账的候选过滤（`mount::isolate_slots_referenced_only_by_abandoned_roots`：
+//! 按实例表有效 ∧ txg ≥ F 的根都是候选，窗口外的有效根照样豁免它引用的槽）与池级 checker I-3.1（已分配统计对得上） 丁-defer 的门槛
+//! （`walk::deferred_slots_reclaim_threshold`：max(F_生效, 环里最旧有效根)）。改回一圈窗口时这两处没有会红的用例
+//! （判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第二节 K1「影子账候选、I-3.1 丁-defer 门槛改回窗口」那一行，正推 K1-2）。
+//!
+//! 盘面是 S = 4 的 12 槽小环（`common_root_ring_holes`）。两形都先造一条被抛弃的根（实例 1 最新那次发布的系统配置轮换没落、下一次可写挂载时它的
+//! 根槽与数据单元一时读不出，择根落到前一条根 B、实例 2 写行，之后它读得出、按实例表被抛弃），再让一次可写挂载崩在写行那次根槽写之前
+//! （K1 那一类洞：txg 跳一个，洞那一槽里留着上一圈的根），之后的发布把锚往前推，洞里那条根落到一圈窗口外、仍是有效根。
+//! - 丁-defer（`raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs` 那一形搬到有洞的小环上）：B 那一槽在洞里留着，
+//!   F 抬到 B 与被抛弃的那条根之间（B 在 F 之下）。B 引用的实例 1 那张实例表在实例 2 写行那次被整条重写，释放代高于 F（还在 defer 里，分配器照算已分配），
+//!   F 之上的有效根都不引用它，被抛弃的那条根不走、它的记录也不算被施加过。门槛取整个环是 max(F, 环里最旧有效根 B) = F，并进遍历；
+//!   照一圈窗口取时门槛是窗口里最旧的有效根（高于那个释放代），那几槽不并，I-3.1 红在记账多的一边。
+//!   不用 K1 那一形的「实例 1 连崩之后 F 抬到 24」：那里 txg 24 那一版由记录施加出来、照候选走（`walk::versions_applied_only_by_records`），
+//!   实例 1 那张实例表由它走到，丁-defer 门槛取什么都一样，那一形钉不住门槛。
+//! - 影子账：先造一条被抛弃的根——实例 1 最新的根 txg 24 那次发布的系统配置轮换没落、下一次可写挂载时它的根槽与数据单元一时读不出，择根落到 23、
+//!   实例 2 写行（行 (1, 23)），之后 24 读得出、按实例表被抛弃。再连崩到 txg 36 那一槽也被跳过（24 那一槽不被盖掉）、做完一次可写挂载，
+//!   再挂时影子账按候选集算：实例 1 那张实例表被被抛弃的 24 引用，也被洞里实例 1 的有效根（在窗口外）引用，豁免、不隔离；
+//!   照一圈窗口滤候选时窗口外那几条不豁免，那几槽被隔离。
+
+mod common_root_ring_holes;
+
+use common_root_ring_holes::{
+    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
+    parameters_of_the_ring_hole_pools, pool_of, RingHoleDevices,
+    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
+    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
+use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
+use singlefs_core::make_filesystem::make_filesystem;
+use singlefs_core::mount::{mount_writable, raise_rollback_floor, Mounted, ShadowLedger};
+use singlefs_core::recovery::{
+    allocation_records_under_root, choose_root, choose_system_configuration,
+    effective_rollback_floor, instance_table_of_root, readable_roots,
+};
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_core::transaction::{
+    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
+};
+use singlefs_format::{
+    DATA_UNIT_BYTES, ROOT_RING_REGIONS, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
+    SYSTEM_CONFIGURATION_SLOT_BYTES,
+};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use std::collections::{BTreeMap, BTreeSet};
+
+/// 影子账那一形实例 1 覆盖写到的 txg（被抛弃的就是这一条）。
+const LAST_TXG_OF_INSTANCE_ONE: u64 = 24;
+/// 丁-defer 那一形实例 1 发到的 txg（被抛弃的那一条；它前一条 7 就是 B）。
+const ABANDONED_TXG_UNDER_THE_RAISED_FLOOR: u64 = 8;
+/// 影子账那一形造出被抛弃的根之后连崩的次数：跳过的 txg 要盖住 24 那一槽的下一圈 txg 36（24 + 12），不让它被盖掉。
+const CRASHES_BEFORE_THE_ROW_ROOT_AFTER_THE_ABANDONMENT: usize = 11;
+
+/// 一圈窗口的长度（3S）。
+fn ring_length_in_txgs() -> u64 {
+    ROOT_RING_REGIONS * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS
+}
+
+/// 盘面上读得出的根、最新那条根指着的实例表、按它判被抛弃。
+struct RootsOfTheRing {
+    newest: RootRecord,
+    valid: Vec<RootRecord>,
+    abandoned: Vec<RootRecord>,
+}
+
+fn roots_of_the_ring(image: &MemoryPool) -> RootsOfTheRing {
+    let system_configuration = choose_system_configuration(image).expect("系统配置");
+    let newest = choose_root(image, &system_configuration).expect("择得到根");
+    let table = instance_table_of_root(image, &newest).expect("最新那条根的实例表读得出");
+    let immutable = &system_configuration.immutable;
+    let (abandoned, valid): (Vec<RootRecord>, Vec<RootRecord>) = readable_roots(
+        image,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    )
+    .into_iter()
+    .partition(|root| {
+        table
+            .rows
+            .iter()
+            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
+    });
+    RootsOfTheRing {
+        newest,
+        valid,
+        abandoned,
+    }
+}
+
+/// 一条根在不在锚在 `anchor` 上的一圈窗口里（txg > N − 3S）。
+fn inside_the_one_revolution_window(root: &RootRecord, anchor: CheckpointTxg) -> bool {
+    root.checkpoint_txg.0 + ring_length_in_txgs() > anchor.0
+}
+
+/// 盘面上池级 checker 判 I-3.1 违例的那几块盘（名字加细节）。
+fn allocated_statistic_violations(image: &MemoryPool) -> Vec<String> {
+    check_pool_image(image)
+        .into_iter()
+        .filter(|(invariant, _)| *invariant == "I-3.1")
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
+        })
+        .collect()
+}
+
+/// 场景：实例 1 发到 txg 8，8 被崩溃恢复抛弃（实例 2 写行 (1, 7)，B = 7）；再挂一次（实例 3）覆盖写到 txg 18；下一次可写挂载崩在写行 txg 19
+/// 那次根槽写之前（19 与 B 同槽：B 留在洞里）；再一次可写挂载做完；同一个进程里把 F 抬到 8。
+/// 预期（前提）：B 还在环里、有效、在 F 之下、在锚在最新根上的一圈窗口外，窗口里最旧的有效根高于 F；
+/// 池级 checker 不判 I-3.1——丁-defer 的门槛取整个环：max(8, 环里最旧有效根 7) = 8，B 引用、释放代高于 8 的槽并进遍历。
+/// 判别力：丁-defer 门槛改回一圈窗口（`crates/mutations.tsv`「第10g批 丁-defer 门槛改回一圈窗口…」）时门槛是窗口里最旧的有效根，
+/// 那几槽不并、记账多于遍历，最后一项红。
+#[test]
+fn raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring(
+) {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let root_before_the_abandoned_one = ABANDONED_TXG_UNDER_THE_RAISED_FLOOR - 1;
+    let floor = CheckpointTxg(ABANDONED_TXG_UNDER_THE_RAISED_FLOOR);
+    let image =
+        pool_with_the_newest_root_of_instance_one_abandoned(ABANDONED_TXG_UNDER_THE_RAISED_FLOOR);
+    let image_before_the_hole = {
+        let mut devices = devices_of(&image);
+        let Mounted {
+            output,
+            mut allocator,
+            current,
+        } = mount_writable(&parameters, &mut devices).expect("被抛弃的根造出来之后再挂一次");
+        let PoolVersion::WithFile(mut current) = current else {
+            panic!("B 那一版带文件");
+        };
+        // 迭代上界：覆盖写到 B 那一槽下一圈的 txg 之前那一个；跨轮携带的是现行那一版。
+        while current.root.checkpoint_txg.0 + 1
+            < root_before_the_abandoned_one + ring_length_in_txgs()
+        {
+            let salt = current.root.checkpoint_txg.0 + 1;
+            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+            current = publish_overwrite(
+                &mut writer,
+                &mut allocator,
+                &current,
+                FirstFile {
+                    content: &content_of_the_version(salt),
+                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
+                },
+                output.instance,
+            )
+            .expect("覆盖写");
+        }
+        pool_of(&devices)
+    };
+    let (image_with_the_hole, row_txg_of_the_crashed_mount) =
+        crash_a_mount_before_its_row_root(&image_before_the_hole);
+    assert_eq!(
+        row_txg_of_the_crashed_mount,
+        root_before_the_abandoned_one + ring_length_in_txgs(),
+        "前提：崩在写行之前的那次挂载要写的根槽就是 B 那一槽"
+    );
+    let mut devices = devices_of(&image_with_the_hole);
+    let Mounted {
+        mut allocator,
+        current,
+        ..
+    } = mount_writable(&parameters, &mut devices).expect("洞之后的可写挂载做完");
+    let PoolVersion::WithFile(mut current) = current else {
+        panic!("现行那一版带文件");
+    };
+    raise_rollback_floor(
+        &parameters,
+        &mut devices,
+        &mut allocator,
+        &mut current,
+        floor,
+        ShadowLedger::On,
+    )
+    .expect("F 抬到 8（不高于上限）");
+    let image_after_the_raise = pool_of(&devices);
+    let roots = roots_of_the_ring(&image_after_the_raise);
+    let below_the_floor_outside_the_window: Vec<CheckpointTxg> = roots
+        .valid
+        .iter()
+        .filter(|root| {
+            root.checkpoint_txg < floor
+                && !inside_the_one_revolution_window(root, roots.newest.checkpoint_txg)
+        })
+        .map(|root| root.checkpoint_txg)
+        .collect();
+    let oldest_valid_root_inside_the_window = roots
+        .valid
+        .iter()
+        .filter(|root| inside_the_one_revolution_window(root, roots.newest.checkpoint_txg))
+        .map(|root| root.checkpoint_txg)
+        .min();
+    assert!(
+        below_the_floor_outside_the_window == vec![CheckpointTxg(root_before_the_abandoned_one)]
+            && oldest_valid_root_inside_the_window.is_some_and(|oldest| oldest > floor),
+        "前提：F 之下、一圈窗口外的有效根恰是 B {below_the_floor_outside_the_window:?}；窗口里最旧的有效根 {oldest_valid_root_inside_the_window:?} 高于 F（最新根 {:?}）",
+        roots.newest.checkpoint_txg
+    );
+    assert_eq!(
+        allocated_statistic_violations(&image_after_the_raise),
+        Vec::<String>::new(),
+        "丁-defer 门槛取整个环：B 引用、还在 defer 里的槽并进遍历，记账与遍历对得上"
+    );
+}
+
+/// 实例 1 覆盖写到 `last` 那一刻的盘与它最后那次发布；另交最后那次发布之前每块盘两槽系统配置的字节（盘号、偏移、字节）。
+struct PoolOfInstanceOne {
+    devices: RingHoleDevices,
+    last_publish: TransactionOutput,
+    system_configuration_before_the_last_publish:
+        Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)>,
+}
+
+/// 照 `common_root_ring_holes::pool_overwritten_through` 同一个做法（mkfs → 可写挂载（实例 1）→ 第一个文件 → 覆盖写到 txg `last`），
+/// 多留最后那次发布与它之前的系统配置槽。
+fn pool_of_instance_one_overwritten_through(last: u64) -> PoolOfInstanceOne {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices: RingHoleDevices = (0..2u32)
+        .map(|number| {
+            (
+                DeviceIdentity(number),
+                SparseBlockDevice::new(
+                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
+                    PhysicalBlockSizeInBytes(512),
+                ),
+            )
+        })
+        .collect();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let Mounted {
+        output,
+        mut allocator,
+        current,
+    } = mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
+    let instance = output.instance;
+    let mut current: TransactionOutput = {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut allocator,
+            current.root(),
+            FirstFile {
+                content: &content_of_the_version(0),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
+            },
+            instance,
+            current.record_bytes(),
+        )
+        .expect("第一个文件")
+    };
+    let mut system_configuration_before_the_last_publish = Vec::new();
+    // 迭代上界：txg 逐次加一，到 `last` 为止。
+    while current.root.checkpoint_txg.0 < last {
+        let salt = current.root.checkpoint_txg.0 + 1;
+        if salt == last {
+            system_configuration_before_the_last_publish = system_configuration_slots_of(&devices);
+        }
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        current = publish_overwrite(
+            &mut writer,
+            &mut allocator,
+            &current,
+            FirstFile {
+                content: &content_of_the_version(salt),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
+            },
+            instance,
+        )
+        .expect("覆盖写");
+    }
+    PoolOfInstanceOne {
+        devices,
+        last_publish: current,
+        system_configuration_before_the_last_publish,
+    }
+}
+
+/// 每块盘两个系统配置槽的全部字节。
+fn system_configuration_slots_of(
+    devices: &RingHoleDevices,
+) -> Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> {
+    let spacing = u64::from(
+        parameters_of_the_ring_hole_pools()
+            .geometry
+            .fixed_structure_slot_spacing,
+    );
+    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
+    let mut saved = Vec::new();
+    // 迭代上界是盘数 × 每块盘两槽；没有提前出口。
+    for (identity, device) in devices {
+        for slot in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
+            let offset = DeviceOffsetInBytes(slot * spacing);
+            let mut bytes = vec![0u8; slot_bytes];
+            device.read_at(offset, &mut bytes).expect("读系统配置槽");
+            saved.push((*identity, offset, bytes));
+        }
+    }
+    saved
+}
+
+fn write_bytes(
+    devices: &mut RingHoleDevices,
+    device: DeviceIdentity,
+    offset: DeviceOffsetInBytes,
+    bytes: &[u8],
+) {
+    devices
+        .iter_mut()
+        .find(|(identity, _)| *identity == device)
+        .expect("池里有这块盘")
+        .1
+        .write_at(offset, bytes, WriteDurability::Plain)
+        .expect("写进内存盘");
+}
+
+/// 崩溃恢复造一条被抛弃的根（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 同一形，换到 12 槽小环上）：
+/// 实例 1 最新那次发布（txg `last`）的系统配置轮换没落（每块盘两槽退回它之前的字节）；下一次可写挂载时它的根槽与数据单元一时读不出（清零），
+/// 择根落到前一条根、它那条记录施加前验点名单元失败、不施加，实例 2 写行与暖机；挂载交回之后把它的根槽原样写回（数据单元不写回：那几槽实例 2 可能已经用了）。
+/// 交回之后的盘面：txg `last` 的根读得出、按实例 2 的实例表被抛弃。
+fn pool_with_the_newest_root_of_instance_one_abandoned(last: u64) -> MemoryPool {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let PoolOfInstanceOne {
+        mut devices,
+        last_publish,
+        system_configuration_before_the_last_publish,
+    } = pool_of_instance_one_overwritten_through(last);
+    for (device, offset, bytes) in &system_configuration_before_the_last_publish {
+        write_bytes(&mut devices, *device, *offset, bytes);
+    }
+    let target = target_for_publish(
+        last_publish.root.checkpoint_txg,
+        parameters.geometry.root_ring_slots_per_region,
+    );
+    let root_slot_device =
+        parameters.region_devices[usize::try_from(target.region).expect("区域号")];
+    let root_slot_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
+    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
+    let mut root_slot = vec![0u8; root_slot_bytes];
+    devices
+        .iter()
+        .find(|(identity, _)| *identity == root_slot_device)
+        .expect("池里有这块盘")
+        .1
+        .read_at(root_slot_offset, &mut root_slot)
+        .expect("暂存最新那条根的根槽");
+    write_bytes(
+        &mut devices,
+        root_slot_device,
+        root_slot_offset,
+        &vec![0u8; root_slot_bytes],
+    );
+    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
+    for location in last_publish
+        .data_pointers
+        .iter()
+        .flat_map(|pointer| pointer.locations)
+    {
+        write_bytes(
+            &mut devices,
+            location.device,
+            location.slot.to_device_offset(),
+            &vec![0u8; data_unit_bytes],
+        );
+    }
+    let mounted =
+        mount_writable(&parameters, &mut devices).expect("择根落到前一条根，照常可写挂载");
+    assert_eq!(
+        (
+            mounted.output.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        (InstanceGeneration(2), CheckpointTxg(last - 1)),
+        "前提：恢复落到 txg {} 那一版，实例 2",
+        last - 1
+    );
+    write_bytes(&mut devices, root_slot_device, root_slot_offset, &root_slot);
+    pool_of(&devices)
+}
+
+/// 场景：实例 1 覆盖写到 24，24 那次发布被崩溃恢复抛弃（实例 2 写行 (1, 23)）；再连崩 11 次在写行根槽之前（跳过的 txg 盖住 36，24 那一槽留着）、
+/// 一次可写挂载做完；再可写挂载一次，看重建分配器影子账隔离了几槽。
+/// 预期（前提）：盘上只有一条被抛弃的根 24；有被它引用、也只被锚在最新有效根上的一圈窗口外的有效根引用（窗口里的有效根与现行那一版的账都不引用）的槽。
+/// 影子账按候选集（按实例表有效 ∧ txg ≥ F，不按窗口滤）豁免：每块盘隔离的槽数 = 被抛弃的根引用的槽里，候选根与现行那一版的账都不引用的那几槽的跨度之和。
+/// 判别力：影子账的候选过滤改回一圈窗口（`crates/mutations.tsv`「第10g批 影子账候选过滤改回一圈窗口…」）时窗口外那几条根不豁免，
+/// 它们与被抛弃的根同引用的槽被隔离，隔离数变多，最后一项红。
+#[test]
+fn a_remount_isolates_no_slot_an_abandoned_root_shares_with_valid_roots_left_in_the_holes_outside_the_one_revolution_window(
+) {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut image = pool_with_the_newest_root_of_instance_one_abandoned(LAST_TXG_OF_INSTANCE_ONE);
+    // 迭代上界：连崩的次数，每次一个洞；跨轮携带的是崩溃之后的盘面。
+    for _ in 0..CRASHES_BEFORE_THE_ROW_ROOT_AFTER_THE_ABANDONMENT {
+        let (crashed, _row_txg) = crash_a_mount_before_its_row_root(&image);
+        image = crashed;
+    }
+    let mut devices = devices_of(&image);
+    mount_writable(&parameters, &mut devices).expect("连崩之后的可写挂载做完");
+    let image_before_the_remount = pool_of(&devices);
+
+    let roots = roots_of_the_ring(&image_before_the_remount);
+    let effective_floor = {
+        let system_configuration =
+            choose_system_configuration(&image_before_the_remount).expect("系统配置");
+        let immutable = system_configuration.immutable;
+        effective_rollback_floor(
+            &image_before_the_remount,
+            &immutable.region_devices,
+            &immutable.sizes,
+            &immutable.filesystem_identifier,
+        )
+    };
+    let start_slots_allocated_under = |root: &RootRecord| -> Vec<(DeviceIdentity, u64, u64)> {
+        allocation_records_under_root(&image_before_the_remount, root)
+            .expect("这一版的账读得出")
+            .into_iter()
+            .filter(|record| !record.is_released)
+            .map(|record| (record.device, record.slot.0, u64::from(record.span_slots)))
+            .collect()
+    };
+    let newest_valid_root = roots
+        .valid
+        .iter()
+        .map(|root| root.checkpoint_txg)
+        .max()
+        .expect("有有效根");
+    let candidates: Vec<&RootRecord> = roots
+        .valid
+        .iter()
+        .filter(|root| root.checkpoint_txg >= effective_floor)
+        .collect();
+    let referenced_by = |roots_referencing: &[&RootRecord]| -> BTreeSet<(DeviceIdentity, u64)> {
+        roots_referencing
+            .iter()
+            .flat_map(|root| start_slots_allocated_under(root))
+            .map(|(device, slot, _)| (device, slot))
+            .collect()
+    };
+    let current_version = choose_root(
+        &image_before_the_remount,
+        &choose_system_configuration(&image_before_the_remount).expect("系统配置"),
+    )
+    .expect("现行那一版的根");
+    let allocated_in_the_current_version: BTreeSet<(DeviceIdentity, u64)> =
+        referenced_by(&[&current_version]);
+    let referenced_by_candidates = referenced_by(&candidates);
+    let candidates_inside_the_window: Vec<&RootRecord> = candidates
+        .iter()
+        .copied()
+        .filter(|root| inside_the_one_revolution_window(root, newest_valid_root))
+        .collect();
+    let referenced_by_candidates_inside_the_window = referenced_by(&candidates_inside_the_window);
+    let mut expected_isolated_per_device: BTreeMap<DeviceIdentity, u64> = BTreeMap::new();
+    let mut shared_only_with_candidates_outside_the_window: BTreeSet<(DeviceIdentity, u64)> =
+        BTreeSet::new();
+    let mut counted: BTreeSet<(DeviceIdentity, u64)> = BTreeSet::new();
+    // 迭代上界是被抛弃的根的条数乘以它们账里的记录数；没有提前出口。
+    for abandoned in &roots.abandoned {
+        for (device, slot, span) in start_slots_allocated_under(abandoned) {
+            let key = (device, slot);
+            if allocated_in_the_current_version.contains(&key) {
+                continue;
+            }
+            if referenced_by_candidates.contains(&key) {
+                if !referenced_by_candidates_inside_the_window.contains(&key) {
+                    shared_only_with_candidates_outside_the_window.insert(key);
+                }
+                continue;
+            }
+            if counted.insert(key) {
+                *expected_isolated_per_device.entry(device).or_insert(0) += span;
+            }
+        }
+    }
+    assert!(
+        roots.abandoned.iter().map(|root| (root.instance, root.checkpoint_txg)).collect::<Vec<_>>()
+            == vec![(InstanceGeneration(1), CheckpointTxg(LAST_TXG_OF_INSTANCE_ONE))]
+            && !shared_only_with_candidates_outside_the_window.is_empty(),
+        "前提：只有被抛弃的根 txg 24；它与窗口外的有效根同引用、窗口里的有效根与现行那一版的账都不引用的槽 {shared_only_with_candidates_outside_the_window:?}（最新有效根 {newest_valid_root:?}，F {effective_floor:?}）"
+    );
+
+    let mut devices_of_the_remount = devices_of(&image_before_the_remount);
+    let remounted = mount_writable(&parameters, &mut devices_of_the_remount).expect("再可写挂载");
+    let isolated_per_device: BTreeMap<DeviceIdentity, u64> = remounted
+        .allocator
+        .devices
+        .iter()
+        .map(|device_map| (device_map.device, device_map.isolated_slots()))
+        .filter(|(_, isolated)| *isolated > 0)
+        .collect();
+    assert_eq!(
+        isolated_per_device, expected_isolated_per_device,
+        "影子账按整个环的候选集豁免：窗口外的有效根与被抛弃的根同引用的 {shared_only_with_candidates_outside_the_window:?} 不隔离"
+    );
+}
diff --git b/crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs b/crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs
new file mode 100644
index 0000000..7c5fed1
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs
@@ -0,0 +1,187 @@
+//! 重建分配器读根环有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，重读那一遍整个环仍一条都没有：拒可写的拒因另起成员
+//! （`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`），不复用读阶段择不到根的 `Recovery(NoValidRoot)`；
+//! D23（journal 的角色与格式） 已定项 14 那张单子加一行（用户 2026-10-05 定 Q3，岔路单 `research/prompts/m3-batch10-code-r1-forks.md`，
+//! 判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）。判法在崩溃镜像上复算「按最新那条根的实例表，环里一条不被抛弃的
+//! 自证有效根都没有」，与改法 G4 同形：注入了读故障的崩溃态上不判那一条；没注入读故障的崩溃态上复算为真，拒得不合法
+//! （`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`）。
+//! 盘面是 24 槽根环的小池（`common::build_pool`，第一个文件之后）；手工盘面是把它根环三个区域的每一个槽都清成全 0。
+
+mod common;
+
+use common::build_pool;
+use singlefs_core::mount::MountError;
+use singlefs_core::recovery::{
+    choose_system_configuration, every_root_ring_slot, recover, JournalPolicy, RecoveryFailure,
+};
+use singlefs_core::root_ring::slot_offset;
+use singlefs_harness::memory_pool::MemoryPool;
+use singlefs_harness::mount_transparency::{
+    judge_writable_mount_refusal_of_a_crash_state,
+    judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount,
+    refusal_reason_on_the_d23_list, DiskAfterTheRefusal, ReadFaultsInTheCrashState,
+    ReadFaultsUnderTheRefusedMount, ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet,
+    RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
+};
+
+/// 第一个文件之后的小池盘面。
+fn image_after_the_first_file(tag: &str) -> MemoryPool {
+    build_pool(tag).memory_pool()
+}
+
+/// 手工盘面：根环三个区域的每一个槽都清成全 0（静默损坏，不是崩溃摆得出来的）。
+fn image_with_every_root_ring_slot_zeroed_by_hand(image: &MemoryPool) -> MemoryPool {
+    let system_configuration =
+        choose_system_configuration(image).expect("清根环之前择得到系统配置");
+    let immutable = system_configuration.immutable;
+    let root_slot_bytes = usize::try_from(immutable.sizes.physical_block_size).expect("根槽宽");
+    let mut zeroed = image.clone();
+    // 迭代上界是根环的槽数；没有提前出口。
+    for ring_slot in every_root_ring_slot(&immutable.sizes) {
+        let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
+        zeroed
+            .devices
+            .get_mut(&device)
+            .expect("区域归属表里的盘在池里")
+            .write(
+                slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing),
+                &vec![0u8; root_slot_bytes],
+            );
+    }
+    zeroed
+}
+
+/// 场景：两个拒因各交一次给「在不在单子上」的判法。
+/// 预期：重建分配器读根环重读仍一条有效根都没有那一拒在单子上（Q3 那一行）；读阶段择不到根的 `Recovery(NoValidRoot)` 照旧不在单子上。
+/// 判别力：单子不收新成员时第一项红。
+#[test]
+fn the_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring_is_on_the_d23_list_and_the_read_stage_one_is_not(
+) {
+    assert_eq!(
+        refusal_reason_on_the_d23_list(
+            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread
+        ),
+        Some(RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator),
+        "重建分配器读根环整环重读一次仍一条有效根都没有：在 D23 已定项 14 那张单子上"
+    );
+    assert_eq!(
+        refusal_reason_on_the_d23_list(&MountError::Recovery(RecoveryFailure::NoValidRoot)),
+        None,
+        "读阶段择不到根（只读挂载同样挂不上）：不在单子上"
+    );
+}
+
+/// 场景：手工把根环每一个槽清成全 0 的盘面，崩溃态没注入读故障；拿 Q3 那一拒交给判官（盘上逐字节不变、只读读出在允许集合里）。
+/// 预期：判据在镜像上复算为真（择不到根），U6 那四条都满足；没有读故障的历史留不下这份盘面，拒得不合法，没满足的恰是 Q3 那一条。
+/// 判别力：判官不复算 Q3 那一条（`crates/mutations.tsv`「第10g批 Q3 判官不复算…」）时判成合法，这一项红。
+#[test]
+fn a_refusal_for_no_valid_root_on_a_crash_image_without_read_faults_whose_root_ring_was_zeroed_by_hand_is_illegal(
+) {
+    let image = image_with_every_root_ring_slot_zeroed_by_hand(&image_after_the_first_file(
+        "no-valid-root-in-the-ring-zeroed-without-read-faults",
+    ));
+    assert_eq!(
+        recover(&image, JournalPolicy::Consult).effective_root,
+        None,
+        "前提：根环全 0，只读恢复也择不到根"
+    );
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &image,
+            None,
+            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults
+            ],
+        },
+        "没注入读故障而判据在镜像上复算为真：拒是对的、造出这份盘面的那一段不对，判不合法"
+    );
+}
+
+/// 场景：同一份手工盘面，崩溃态注入过读故障。
+/// 预期：Q3 那一条不判（D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态），判据复算为真、U6 那四条都满足，拒得合法。
+/// 判别力：注入过读故障的态上也判 Q3 那一条时这一项红。
+#[test]
+fn a_refusal_for_no_valid_root_on_a_read_faulted_crash_image_whose_root_ring_was_zeroed_by_hand_is_legal(
+) {
+    let image = image_with_every_root_ring_slot_zeroed_by_hand(&image_after_the_first_file(
+        "no-valid-root-in-the-ring-zeroed-with-read-faults",
+    ));
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &image,
+            None,
+            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+            ReadFaultsInTheCrashState::Injected,
+        ),
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator
+        ),
+        "注入过读故障的崩溃态上 Q3 那一条不判，判据为真：拒得合法"
+    );
+}
+
+/// 场景：没动过的盘面（根环读得出），崩溃态没注入读故障，被拒的那次挂载读的就是这份镜像、不带故障层；拿 Q3 那一拒交给判官。
+/// 预期：判据在镜像上复算为假（读阶段择得到的根按它自己的实例表不被抛弃），拒的理由是错的，没满足的恰是「判据复算为假」那一条
+/// （Q3 那一条只在判据为真时才有）。
+/// 判别力：判官把 Q3 那一拒的判据恒按真复算时这一项红。
+#[test]
+fn a_refusal_for_no_valid_root_on_a_crash_image_whose_root_ring_reads_back_is_illegal_because_its_criterion_is_false(
+) {
+    let image = image_after_the_first_file("no-valid-root-in-the-ring-readable");
+    let selected_version = recover(&image, JournalPolicy::Consult).effective_root;
+    assert!(
+        selected_version.is_some(),
+        "前提：根环读得出，只读恢复择得到根"
+    );
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &image,
+            selected_version,
+            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            ],
+        },
+        "判据在镜像上复算为假：拒得不合法，理由是错的"
+    );
+}
+
+/// 场景：同一份读得出的盘面，崩溃态注入过读故障、被拒的那次挂载自己的读经故障层（重建那一遍与它的重读读坏是这次挂载里的瞬时读错）。
+/// 预期：判据在镜像上复算为假不算不合法（与「读阶段判完之后系统配置又读不出」那一拒同一个办法，C664），盘上逐字节不变、只读读出在允许集合里，
+/// 拒得合法。
+/// 判别力：被拒的挂载在读故障下跑时也照 U6 判「判据复算为假」时这一项红。
+#[test]
+fn a_refusal_for_no_valid_root_by_a_mount_reading_through_injected_faults_is_legal_though_the_crash_image_reads_back(
+) {
+    let image = image_after_the_first_file("no-valid-root-in-the-ring-readable-transient");
+    let selected_version = recover(&image, JournalPolicy::Consult).effective_root;
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(
+            &image,
+            selected_version,
+            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+            ReadFaultsInTheCrashState::Injected,
+            ReadFaultsUnderTheRefusedMount::TheRefusedMountReadThroughInjectedFaults,
+        ),
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator
+        ),
+        "被拒的那次挂载在读故障下跑：两遍读之间的瞬时读错复算不出，判据复算为假不算不合法"
+    );
+}
````

### mutations-append.tsv

````
第10g批 Q4 rr2 拿掉「一块盘没读出也重读」（192 槽起点只坏重建那一遍、盘 0 报读错：m 取不到、当场只放到环里最旧有效根，窄盘上取号之前拒）	crates/singlefs-core/src/mount.rs	        newest_valid_root_txg_of_the_readings(&first_readings)\n            .filter(|_| a_valid_root_was_read_on_every_device(&first_readings));	        newest_valid_root_txg_of_the_readings(&first_readings);	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	on_the_192_slot_pool_after_a_crashed_unmount_device_zero_failing_the_rebuild_pass_rereads_and_mounts
第10g批 Q4 rr2 拿掉「一块盘没读出也重读」（216 槽起点只坏重建那一遍、盘 1 读回全 0：m 取不到、当场只放到环里最旧有效根，窄盘上取号之前拒）	crates/singlefs-core/src/mount.rs	        newest_valid_root_txg_of_the_readings(&first_readings)\n            .filter(|_| a_valid_root_was_read_on_every_device(&first_readings));	        newest_valid_root_txg_of_the_readings(&first_readings);	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts
第10g批 Q3 重建分配器读根环重读那一遍仍一条有效根都没有照旧报 Recovery(NoValidRoot)（与读阶段择不到根同名）	crates/singlefs-core/src/mount.rs	                    return Err(\n                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                    );	                    return Err(MountError::Recovery(RecoveryFailure::NoValidRoot));	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	the_allocator_rebuild_pass_and_its_reread_both_failing_refuse_the_writable_mount_before_any_write
第10g批 Q3 D23 已定项 14 那张单子不收重建分配器读根环重读仍一条有效根都没有那一拒	crates/singlefs-harness/src/mount_transparency.rs	        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => {\n            Some(RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator)\n        }	        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => None,	-p singlefs-harness --test writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring	the_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring_is_on_the_d23_list_and_the_read_stage_one_is_not
第10g批 Q3 判官不复算没注读故障时判据为真那一条（拿掉新成员的复算：手工清零根环的盘面上判成合法）	crates/singlefs-harness/src/mount_transparency.rs	            if no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(\n                crash_image,\n            ) {\n                conditions_not_met.push(\n                    RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,\n                );\n            }		-p singlefs-harness --test writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring	a_refusal_for_no_valid_root_on_a_crash_image_without_read_faults_whose_root_ring_was_zeroed_by_hand_is_illegal
第10g批 Q3 注入过读故障的崩溃态上也判没注读故障时判据为真那一条	crates/singlefs-harness/src/mount_transparency.rs	            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,\n            ReadFaultsInTheCrashState::NoneInjected,\n        ) => {	            RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,\n            ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,\n        ) => {	-p singlefs-harness --test writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring	a_refusal_for_no_valid_root_on_a_read_faulted_crash_image_whose_root_ring_was_zeroed_by_hand_is_legal
第10g批 Q3 那一拒的判据在崩溃镜像上恒按真复算	crates/singlefs-harness/src/mount_transparency.rs	        RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator => {\n            no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(\n                crash_image,\n            )\n        }	        RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator => true,	-p singlefs-harness --test writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring	a_refusal_for_no_valid_root_on_a_crash_image_whose_root_ring_reads_back_is_illegal_because_its_criterion_is_false
第10g批 Q3 被拒的挂载在读故障下跑时也照 U6 判「判据复算为假」（只给「读阶段判完之后系统配置又读不出」那一拒豁免）	crates/singlefs-harness/src/mount_transparency.rs	            if read_faults_under_the_refused_mount\n                .can_hide_a_transient_read_error_from_the_crash_image()	            if reason == RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage\n                && read_faults_under_the_refused_mount\n                    .can_hide_a_transient_read_error_from_the_crash_image()	-p singlefs-harness --test writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring	a_refusal_for_no_valid_root_by_a_mount_reading_through_injected_faults_is_legal_though_the_crash_image_reads_back
第10g批 影子账候选过滤改回一圈窗口（锚取这一遍读出的最新有效根、长 3S：洞里窗口外的有效根不豁免它与被抛弃的根同引用的槽，隔离数变多）	crates/singlefs-core/src/mount.rs	    for root in roots\n        .iter()\n        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)\n    {	    let window_anchor_of_the_candidates = roots\n        .iter()\n        .filter(|root| !is_abandoned(root))\n        .map(|root| root.checkpoint_txg.0)\n        .max()\n        .unwrap_or(0);\n    let ring_length_of_the_candidates = ROOT_RING_REGIONS\n        * crate::recovery::choose_system_configuration(devices)\n            .expect("系统配置")\n            .immutable\n            .sizes\n            .root_ring_slots_per_region\n            .count();\n    for root in roots.iter().filter(|root| {\n        !is_abandoned(root)\n            && root.checkpoint_txg >= floor\n            && root.checkpoint_txg.0 + ring_length_of_the_candidates > window_anchor_of_the_candidates\n    }) {	-p singlefs-harness --test shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring	a_remount_isolates_no_slot_an_abandoned_root_shares_with_valid_roots_left_in_the_holes_outside_the_one_revolution_window
第10g批 丁-defer 门槛改回一圈窗口（只取锚在镜像最新根上的窗口里的有效根：洞里 F 之下的 B 引用、释放代在 (F, 窗口里最旧有效根] 的槽不并，I-3.1 红在记账多的一边）	crates/singlefs-checker/src/walk.rs	    let deferred_threshold = deferred_slots_reclaim_threshold(\n        roots.iter().map(|(_, _, root)| {	    let deferred_threshold = deferred_slots_reclaim_threshold(\n        roots\n            .iter()\n            .filter(|(_, _, root)| window.contains(root.checkpoint_txg))\n            .map(|(_, _, root)| {	-p singlefs-harness --test shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring	raising_the_floor_over_a_root_left_in_a_hole_below_an_abandoned_root_keeps_the_allocated_statistic_with_the_deferred_threshold_on_the_whole_ring
实审 B3c-1 第一截写表整条流与前缀在崩溃注入的取值域里取样点不敏感（第三类：取值域外不验点名单元的恢复上两种交法不同值，补取样点要给崩溃注入加换恢复策略的开关；不是等价变异）：记录核对器第二条判据不罩恰好是恢复落到的那一版的那次发布（由记录重建、自己的数据单元两份都不在的 (1, 5) 在整条流上也判不出）	crates/singlefs-checker-tier/src/crash.rs	txg.0 >= publish.checkpoint_txg	txg.0 > publish.checkpoint_txg	-p singlefs-checker-tier --test crash_injection_campaign -- a_version_rebuilt_from_records_missing_a_unit	a_version_rebuilt_from_records_missing_a_unit_is_judged_only_on_the_whole_stream_and_the_verifying_recovery_never_lands_on_it
````

### mutations-replacements.tsv

````
第10e批 重建分配器读根环那一遍一条有效根都没读出时不重读、当场拒（删掉重读那一遍）	crates/singlefs-core/src/mount.rs	            None => {\n                before_the_one_reread.before_rereading();\n                let readings_of_the_reread = read_every_root_ring_slot_once();\n                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_reread)\n                else {\n                    return Err(\n                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                    );\n                };\n                (\n                    readings_of_the_reread,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )\n            }	            None => {\n                return Err(\n                    MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                )\n            }	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over
第10e批 重读那一遍还读不出有效根时不拒、照旧给空表往下写	crates/singlefs-core/src/mount.rs	                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_reread)\n                else {\n                    return Err(\n                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                    );\n                };\n                (\n                    readings_of_the_reread,	                let (readings_of_the_reread, window_anchor) =\n                    match newest_valid_root_txg_of_the_readings(&readings_of_the_reread) {\n                        Some(window_anchor) => (readings_of_the_reread, window_anchor),\n                        None => (Vec::new(), CheckpointTxg(0)),\n                    };\n                (\n                    readings_of_the_reread,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	the_allocator_rebuild_pass_and_its_reread_both_failing_refuse_the_writable_mount_before_any_write
第10e批 重建分配器读根环那一遍一条有效根都没读出时照第 17 批那样不重读、给空表往下走（攻方 1.3 甲二：暖机写进盘 0 上最新的根还引用的槽）	crates/singlefs-core/src/mount.rs	            None => {\n                before_the_one_reread.before_rereading();\n                let readings_of_the_reread = read_every_root_ring_slot_once();\n                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_reread)\n                else {\n                    return Err(\n                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                    );\n                };\n                (\n                    readings_of_the_reread,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )\n            }	            None => (\n                Vec::new(),\n                CheckpointTxg(0),\n                RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,\n            ),	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	only_the_allocator_rebuild_pass_failing_and_a_crash_before_the_warm_up_root_lands_on_device_zero_keep_what_device_zero_reads
````

### mutations-delete.txt

````
实审 B3c-1 第一截写表整条流与前缀等价的留档：记录核对器第二条判据不罩恰好是恢复落到的那一版的那次发布（由记录重建、自己的数据单元两份都不在的 (1, 5) 在整条流上也判不出）
````
