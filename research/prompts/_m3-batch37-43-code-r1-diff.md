# 删除线第 37 批 X、第 38 批 Y、第 39 批 Z、第 41–43 批合入的改动原样（2026-10-08）

## 删除线第 37 批 X：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch37/x/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 3928059..68dd9c3 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -540,6 +540,10 @@ pub struct CrashInjectionTally {
     pub remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator: u64,
     pub remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults:
         u64,
+    /// 二次崩溃之后再挂被拒、拒因是「记录有缺口时拒可写」的（D23（journal 的角色与格式） 已定项 14，F1 乙）：判过的次数。这一拒没有 C670 同形的那一条
+    /// （判官不列「没注入读故障时判据复算为真」那一类条件），只差 U6 那几条的照别的拒因不报红。
+    pub remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify:
+        u64,
     /// 每一截（崩溃镜像本身、挂载与一次发布之后、二次崩溃）判到已知红与新发现的状态数之和。
     pub crash_states_ending_known_red: BTreeMap<usize, u64>,
     pub crash_states_ending_new_finding: u64,
@@ -628,6 +632,7 @@ impl CrashInjectionTally {
             remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults,
             remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
             remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults,
+            remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify,
             crash_states_ending_known_red,
             crash_states_ending_new_finding,
             mount_transparency_judged,
@@ -750,6 +755,8 @@ impl CrashInjectionTally {
             remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator;
         self.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults +=
             remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults;
+        self.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify +=
+            remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify;
         for (form, count) in crash_states_ending_known_red {
             *self.crash_states_ending_known_red.entry(*form).or_insert(0) += count;
         }
@@ -967,6 +974,11 @@ impl CrashInjectionTally {
             self.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
             self.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults
         );
+        let _ = writeln!(
+            text,
+            "二次崩溃之后再挂被拒、拒因是记录有缺口时拒可写的（F1 乙）：{} 次",
+            self.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify
+        );
         let _ = writeln!(
             text,
             "挂载透明：第一次崩溃镜像上判了 {:?}，二次崩溃镜像上按段判了 {:?}；没判的 {:?}；判红的字段 {:?}；某一样没判的 {:?}；把清扫删的行放回去再比的 {} 次",
@@ -3719,7 +3731,8 @@ fn judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_with
 /// 没注入读故障时判据复算为真的那一条（N-配置 的 C670：[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]；
 /// 另一拒的改法 G4：[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]；
 /// Q3：[`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）时
-/// 再各记一次不合法，交回拒因与全部没满足的条件（报红用）。拒因是实例表读不出的、那一条不成立（注入过读故障、判据复算为假）、只差 U6 那几条的，交回 None。
+/// 再各记一次不合法，交回拒因与全部没满足的条件（报红用）。拒因是「记录有缺口时拒可写」（F1 乙）的只记一次判过：判官对它不列那一类条件。
+/// 拒因是实例表读不出的、那一条不成立（注入过读故障、判据复算为假）、只差 U6 那几条的，交回 None。
 fn reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
     tally: &mut CrashInjectionTally,
     verdict: WritableMountRefusalVerdict,
@@ -3781,6 +3794,23 @@ fn reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_with
                 conditions_not_met,
             )
         }
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
+        ) => {
+            tally.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify += 1;
+            return None;
+        }
+        WritableMountRefusalVerdict::Illegal {
+            reason:
+                RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
+            conditions_not_met,
+        } => {
+            tally.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify += 1;
+            (
+                RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
+                conditions_not_met,
+            )
+        }
     };
     let mut criterion_true_without_read_faults = false;
     // 迭代上界是判官列出的条件数（至多五条）；跨轮携带的只有上面那一个布尔；没有提前出口。
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index 9cec331..56bd5be 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -462,6 +462,8 @@ fn slice_of_version(version: u32, location: u32) -> Slice {
 // 正推说的「空槽加下一已写槽前 16 KiB」同一形）：CPU 的稀疏读者读出「16 KiB 全零 + 紧接着的下一个落点的前 16 KiB」。GPU 拿全零 16 KiB 那一条的第 8 字
 // （VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES）异或下一个落点那一版的前 16 KiB 的 CRC，拼出整段的 CRC 放进 c1，c2 只留「有 32 KiB」与拼读两位
 // （SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION）：字节只有前 16 KiB 是这一读的，按位置条目比 CRC 对得上时交 CPU 判（slice_crc_matches）。
+// 拼读切片后 16 KiB 没有字节（base 指的是全零 16 KiB 那一条版本字节，再往后不是下一个落点的字节），调用方只能比 CRC 或读前 16 KiB
+// （crash_judge_checker_scan.wgsl 的 c_copy_matches_its_location_entry 比 CRC、读的单元头在前 16 KiB 里）。
 // 下一个落点查不到版本（undecidable 已记）、空段止于盘尾（CPU 越过盘尾读不到）的交 no_slice。「16 KiB 节点加下一槽」不用拼：单元槽落点本来就是
 // 32 KiB 长（盘尾那一槽 16 KiB，CPU 读 32 KiB 越过盘尾），版本字节里后 16 KiB 就是叠上去的下一槽
 fn read_at(device: u32, offset: vec2<u32>, length: u32) -> Slice {
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index 6e1b959..ff7df04 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -20,8 +20,9 @@ use std::collections::BTreeMap;
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
 };
+use singlefs_core::checksum::crc32_castagnoli;
 use singlefs_core::instance_table::InstanceRow;
-use singlefs_core::journal::JournalRecordPlaceInPublish;
+use singlefs_core::journal::{JournalRecord, JournalRecordPlaceInPublish, NamedUnit};
 use singlefs_core::mount::{MountError, MountOutput, RaisedFloor, StillUnreadableAfterOneReread};
 use singlefs_core::records::{
     TreeTableEntry, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT,
@@ -39,8 +40,9 @@ use singlefs_core::recovery::{
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
 use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
 use singlefs_core::transaction::TransactionOutput;
+use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
 use singlefs_format::{
-    SYSTEM_CONFIGURATION_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
+    DATA_UNIT_BYTES, NODE_BYTES, SYSTEM_CONFIGURATION_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
     SYSTEM_CONFIGURATION_SLOT_BYTES,
 };
 
@@ -2251,6 +2253,10 @@ pub enum RefusalReasonOnTheD23List {
     /// 岔路单 `research/prompts/m3-batch10-code-r1-forks.md`；之前报 `Recovery(NoValidRoot)`、与读阶段择不到根同名，不在单子上；
     /// 两遍按槽并照用户 2026-10-05 定 R1，判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第六节：第一遍读出过一条有效根的不拒）。
     NoValidRootInTheRootRingOfTheRebuiltAllocator,
+    /// 「记录有缺口时拒可写」（F1 乙，用户 2026-10-07 定；判据照判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 那一行，`F1_ANY`）：
+    /// 水位之上一次没施加的发布，读得出的那几条记录的本次发布内序号不是 1..N 连着或缺末条，而它们点名的单元里有一个两份副本都验不过，
+    /// 重读一次仍这样（`StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`）。
+    PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
 }
 
 /// 拒可写的理由在不在 D23 已定项 14 那张单子上。别的理由（单元区墙、坏盘上才有的那几种……）照崩溃注入第二截今天的写法判。
@@ -2265,12 +2271,13 @@ pub fn refusal_reason_on_the_d23_list(error: &MountError) -> Option<RefusalReaso
                 StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. } => {
                     Some(RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable)
                 }
+                StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
+                    Some(RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify)
+                }
                 // 挂着时抬 F、管理员回退那几处读坏，与影子账读一条被抛弃根的账读不出（C393）：不在 D23 已定项 14 那张单子上。
                 StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                 | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
-                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. }
-                // F1 乙（用户 2026-10-07 定）那一拒：D23 已定项 14 那张单子还没写回它（kb 写回在主 agent），这里照「不在单子上」判。
-                | StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => None,
+                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => None,
             }
         }
         MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
@@ -2589,6 +2596,88 @@ fn no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_c
     })
 }
 
+/// 「记录有缺口时拒可写」（D23（journal 的角色与格式） 已定项 14，F1 乙）在崩溃镜像上复算，不调实现的
+/// `mount::publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`：水位（`selected_version`，R1 落到的那一版）之上
+/// 按 (实例代号, checkpoint_txg) 分组的每一次发布，读得出的记录的本次发布内序号排好之后不是从 1 起逐个加一、或其中没有一条带末条标志，
+/// 而它们点名的单元里有一个两份副本都验不过（[`named_unit_has_a_copy_that_verifies_on_the_crash_image`] 为假）——有这样一次发布即为真。
+/// 崩溃镜像上没有瞬时读错，重读一次读出的与这一遍相同，判据只算一遍。择不到系统配置按真（与这一族别的复算同向）；
+/// 没有所选那一版时水位无从取，交回假。
+fn publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify_on_the_crash_image(
+    crash_image: &dyn PoolReader,
+    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
+) -> bool {
+    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
+        return true;
+    };
+    let Some(selected_version) = selected_version else {
+        return false;
+    };
+    let mut records_of_each_publish: BTreeMap<
+        (InstanceGeneration, CheckpointTxg),
+        Vec<JournalRecord>,
+    > = BTreeMap::new();
+    for record in scan_journal(crash_image, &system_configuration).into_values() {
+        let publish = (record.instance, record.checkpoint_txg);
+        if publish > selected_version {
+            records_of_each_publish
+                .entry(publish)
+                .or_default()
+                .push(record);
+        }
+    }
+    records_of_each_publish
+        .values()
+        .any(|records_of_the_publish| {
+            let mut ordinals: Vec<u32> = records_of_the_publish
+                .iter()
+                .map(|record| record.ordinal_within_publish.0)
+                .collect();
+            ordinals.sort_unstable();
+            let ordinals_run_from_one_without_a_gap = ordinals
+                .iter()
+                .zip(1u32..)
+                .all(|(ordinal, expected_ordinal)| *ordinal == expected_ordinal);
+            let carries_the_last_record_of_the_publish =
+                records_of_the_publish.iter().any(|record| {
+                    record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
+                });
+            let some_named_unit_fails_on_both_copies = records_of_the_publish
+                .iter()
+                .flat_map(|record| &record.named)
+                .any(|named| {
+                    !named_unit_has_a_copy_that_verifies_on_the_crash_image(crash_image, named)
+                });
+            !(ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish)
+                && some_named_unit_fails_on_both_copies
+        })
+}
+
+/// 一条记录点名的一个单元在崩溃镜像上至少有一份副本验得过：某一条位置条目指的那一份读得出，按单元种类的长度（数据单元、打包单元 32 KiB，
+/// 索引节点 16 KiB）读回的整单元 CRC-32C 等于条目里的。种类不认识的点名项（坏盘上的记录）哪一份都不算验得过。
+fn named_unit_has_a_copy_that_verifies_on_the_crash_image(
+    crash_image: &dyn PoolReader,
+    named: &NamedUnit,
+) -> bool {
+    let unit_length_in_bytes = if named.unit_class == UNIT_CLASS_INDEX_NODE {
+        NODE_BYTES
+    } else if named.unit_class == UNIT_CLASS_DATA || named.unit_class == UNIT_CLASS_PACKED {
+        DATA_UNIT_BYTES
+    } else {
+        return false;
+    };
+    let unit_bytes =
+        usize::try_from(unit_length_in_bytes).expect("单元字节数 16384 或 32768 装得进 usize");
+    named.locations.iter().any(|location| {
+        crash_image
+            .read(
+                location.device,
+                location.slot.to_device_offset(),
+                unit_bytes,
+            )
+            .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
+    })
+}
+
 /// 拒之后盘上逐字节同不同（调用方比拒之前与拒之后的整份镜像）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum DiskAfterTheRefusal {
@@ -2797,7 +2886,8 @@ pub fn judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_und
             ReadFaultsInTheCrashState::Injected,
         )
         | (
-            RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
+            RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable
+            | RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
             ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,
         ) => {}
     }
@@ -2817,8 +2907,11 @@ pub fn judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_und
                 });
             }
         }
+        // 「记录有缺口时拒可写」那一拒没有这一条宽限：被拒的那次挂载在读故障下跑、判据在崩溃镜像上复算为假的照判不合法
+        // （D23 已定项 14 那一段只写判据、没写读故障下这一拒怎么判；取不放宽的一边：复算不出的报红，不藏）。
         RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
-        | RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable => {}
+        | RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable
+        | RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify => {}
     }
     verdict_of_a_refusal_on_the_d23_list(reason, conditions_not_met)
 }
@@ -2846,6 +2939,12 @@ fn conditions_not_met_by_a_refusal_on_the_d23_list(
                 crash_image,
             )
         }
+        RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify => {
+            publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify_on_the_crash_image(
+                crash_image,
+                selected_version,
+            )
+        }
     };
     let mut conditions_not_met = Vec::new();
     if !criterion_holds_on_the_crash_image {
diff --git a/crates/singlefs-harness/tests/random_histories.rs b/crates/singlefs-harness/tests/random_histories.rs
index 2e138f7..0ae8fa9 100644
--- a/crates/singlefs-harness/tests/random_histories.rs
+++ b/crates/singlefs-harness/tests/random_histories.rs
@@ -1185,8 +1185,8 @@ fn turning_the_root_ring_with_overwrites_in_the_make_filesystem_process_runs_to_
 /// 覆盖写（10）、可写挂载（实例 3，txg 11、12）、覆盖写三次（13–15）、抬 F（选择子 8）。旧形态下两次回退各开一个实例、抛弃前一段，
 /// F = 8 落进回退留下的空档，I-3.1 记账多算 12 槽；向前回退不抛弃任何根，这一段没有空档，每一步之后 checker 判绿、跑完。
 /// 清单那一条的形态今天只剩崩溃恢复抛弃的时间线造得出（实四乙第 7 件）；C554 乙之后只剩系统配置没见证到被抛弃那条根的那一形，
-/// 那一形的时间线由 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_is_red_only_on_i_7_4_before_any_floor_raise_in_the_known_double_fault_form`
-/// 造，抬 F 之前已经 I-7.4 红（C583（双故障形：每盘一槽系统配置坏加最新根一时读不出），已知）。
+/// 那一形的时间线由 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units`
+/// 造（C583（双故障形：每盘一槽系统配置坏加最新根一时读不出））：可写挂载时的预隔离护住被抛弃那条根的单元，抬 F 之前池级 checker 不红。
 #[test]
 fn the_history_that_raised_the_floor_into_a_rollback_gap_completes_under_the_forward_rollback() {
     let empty = ContentChoice {
@@ -1317,16 +1317,17 @@ fn violations_on_the_image(image: &MemoryPool) -> Vec<(&'static str, String)> {
         .collect()
 }
 
-/// 双故障形（C583（双故障形：每盘一槽系统配置坏加最新根一时读不出），已知，留里程碑三）：第一个文件 A（txg 3）之后覆盖写 B（4）、C（5），
-/// 崩溃恢复抛弃 C——择根落到 B，实例 2 写行 6、暖机 7；再覆盖写四次（8–11）。那次崩溃恢复看不见 C、把 C 引用的单元又发出去，
-/// 抬 F 之前池级 checker 就恰好红一条 I-7.4（近 K 代块未被复用），说明文字点名实例 1、txg 5 那条被抛弃的根——乙罩不到的那一格
-/// （`research/prompts/m2-impl-c554-yi-implementer-report.md` 第六节 Q1），不是乙的拒。被抛弃的 C 由
-/// [`pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version`] 造（系统配置没见证到 C 的那一形；生成器的那一步 C554 乙之后造不出）。
-/// 原来这一条接着抬 F 到 5、拿观察按「已知红」清单归到第 0 条（收口表第 43 行）；第 0 条要「只有 I-3.1 红」，抬 F 之前已经 I-7.4 红，
-/// 抬 F 与归类那一段归不进第 0 条，删掉（`reuse_after_raising_the_floor.rs` 那一条同形、同样改钉）。
+/// 双故障形（C583（双故障形：每盘一槽系统配置坏加最新根一时读不出））：第一个文件 A（txg 3）之后覆盖写 B（4）、C（5），
+/// 崩溃恢复抛弃 C——择根落到 B，实例 2 写行 6、暖机 7；再覆盖写四次（8–11）。那次挂载时 C 的根槽读不出、系统配置没见证 C（乙罩不到的那一格，
+/// `research/prompts/m2-impl-c554-yi-implementer-report.md` 第六节 Q1），影子账看不见 C；可写挂载时的预隔离（D23（journal 的角色与格式） 已定项 14
+/// 「可写挂载时的预隔离」）把水位之上没施加的 C 的记录点名的落点与 C 的账里的落点隔离到这个进程写掉 txg 5 那一槽为止，四次覆盖写发不出 C 的单元，
+/// 抬 F 之前池级 checker 一条都不红。被抛弃的 C 由 [`pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version`] 造
+/// （系统配置没见证到 C 的那一形；生成器的那一步 C554 乙之后造不出）。
+/// 判别力：挂载时不隔离那一份（变异「第37批 可写挂载时的预隔离不隔离水位之上没施加的记录点名的落点」）时 C 引用的单元又被发出去，
+/// I-7.4（近 K 代块未被复用）红、点名实例 1、txg 5 那条被抛弃的根，这一条红。
 #[test]
 #[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
-fn crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_is_red_only_on_i_7_4_before_any_floor_raise_in_the_known_double_fault_form(
+fn crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(
 ) {
     let (mut pool, abandoned) =
         pool_after_a_crash_recovery_abandoned_an_unwitnessed_third_version("random-history-row-43");
@@ -1344,19 +1345,9 @@ fn crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnes
         "被抛弃的是 C：实例 1、txg 5"
     );
     let violations = violations_on_the_image(&pool.memory_pool());
-    assert_eq!(
-        violations
-            .iter()
-            .map(|(invariant, _)| *invariant)
-            .collect::<Vec<_>>(),
-        vec!["I-7.4"],
-        "抬 F 之前恰好 I-7.4 红：{violations:?}"
-    );
     assert!(
-        violations[0]
-            .1
-            .starts_with("被抛弃的根（实例 1、txg 5）引用的单元已被重新分配或抹头"),
-        "I-7.4 的说明文字点名实例 1、txg 5 那条被抛弃的根：{violations:?}"
+        violations.is_empty(),
+        "抬 F 之前池级 checker 一条都不红（C 的单元被预隔离护着）：{violations:?}"
     );
 }
 
diff --git a/crates/singlefs-harness/tests/writable_mount_keeps_a_released_slot_named_by_an_unapplied_record_isolated_until_its_root_ring_slot_is_overwritten.rs b/crates/singlefs-harness/tests/writable_mount_keeps_a_released_slot_named_by_an_unapplied_record_isolated_until_its_root_ring_slot_is_overwritten.rs
new file mode 100644
index 0000000..9473382
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_keeps_a_released_slot_named_by_an_unapplied_record_isolated_until_its_root_ring_slot_is_overwritten.rs
@@ -0,0 +1,187 @@
+//! 可写挂载时的预隔离（D23（journal 的角色与格式） 已定项 14「可写挂载时的预隔离」）只豁免现行账里仍分配着的槽（D28（挂载期承诺量） 已定项 1
+//! 第九项：「只除去现行账里仍分配着的槽……这次挂载里释放、回收时分配器照补隔离」）：这样的槽在这次挂载里被释放、过了回收门槛，
+//! 而那条被抛弃根的根环槽还没被这个进程写掉时，分配器回收它的那一刻照还没解开的那一份补隔离（`PoolAllocator::record_root_written_by_this_process`
+//! 里回收之后补隔离看的集合并进水位之上没施加的记录点名的落点），它在那一槽写掉之前不被取到。
+//!
+//! 造法（`tests/common/mod.rs`，4 GiB 两块盘，准入宽、不推抬 F：回收只走根环的回收门槛那一条）：第一个文件 A（txg 3）之后同一个进程里覆盖写
+//! B（4）、C（5），崩溃恢复抛弃 C（`abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元在那次挂载里读不出、
+//! 系统配置没见证 C），择根落到 B，实例 2 写行（txg 6）、暖机。C 的记录读得出、不施加：C 的记录新根段与 C 的账里有 B 的现行账里仍分配着的落点，
+//! 挂载那一刻按窄读豁免、不隔离；写行那次发布换掉实例表那一类单元、释放它们。之后实例 2 照常覆盖写：C 的根槽成了环里最小的那个时进程内重读读出 C、
+//! 按实例表判被抛弃、当没有根，回收门槛越过写行那一版——那几个槽在 C 的根环槽被写掉的前一次发布里回收，这时 C 的记录点名的那一份还没解开。
+mod common;
+
+use common::{
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
+    publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
+};
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration, SlotNumber};
+use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, PlacementOnDevice, PoolAllocator};
+use singlefs_core::root_ring::target_for_publish;
+use singlefs_core::transaction::TransactionOutput;
+
+fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
+    (0..length)
+        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
+        .collect()
+}
+
+/// 同一个进程里对建好的池覆盖写一次，`pool.output` 换成这一版。
+fn overwrite_the_built_pool(
+    pool: &mut BuiltPool,
+    content: &[u8],
+    instance: InstanceGeneration,
+) -> TransactionOutput {
+    let previous = pool.output.clone();
+    let output = publish_overwrite_in_process(
+        pool,
+        &previous,
+        content,
+        FIXED_WRITE_TIME_SECONDS + 60,
+        instance,
+    )
+    .expect("4 GiB 两块盘上这一串覆盖写都落得下");
+    pool.output = output.clone();
+    output
+}
+
+/// 分配器记账里罩住 `placement` 起点那一槽的那条记录。
+fn record_covering<'allocator>(
+    allocator: &'allocator PoolAllocator,
+    placement: &PlacementOnDevice,
+) -> Option<&'allocator AllocationRecord> {
+    allocator.records().iter().find(|record| {
+        record.device == placement.device
+            && record.slot.0 <= placement.slot.0
+            && placement.slot.0 < record.slot.0 + u64::from(record.span_slots)
+    })
+}
+
+fn device_map_of<'allocator>(
+    allocator: &'allocator PoolAllocator,
+    placement: &PlacementOnDevice,
+) -> &'allocator DeviceFreeMap {
+    allocator
+        .devices
+        .iter()
+        .find(|device_map| device_map.device == placement.device)
+        .expect("落点的盘在池里")
+}
+
+/// `placement` 跨的每一槽在分配器上都不空闲（已分配、隔离着或被抬 F 扣着）。
+fn every_slot_of_the_placement_is_held(
+    allocator: &PoolAllocator,
+    placement: &PlacementOnDevice,
+) -> bool {
+    let device_map = device_map_of(allocator, placement);
+    (placement.slot.0..placement.slot.0 + placement.span)
+        .all(|slot| !device_map.is_free(SlotNumber(slot)))
+}
+
+/// `placement` 跨的每一槽都已回收（分配位清了、没被抬 F 扣着）：在空闲图的一份拷贝上清掉隔离位之后它就空闲。
+fn every_slot_of_the_placement_is_reclaimed(
+    allocator: &PoolAllocator,
+    placement: &PlacementOnDevice,
+) -> bool {
+    let mut without_the_isolation = device_map_of(allocator, placement).clone();
+    (placement.slot.0..placement.slot.0 + placement.span).all(|slot| {
+        without_the_isolation.clear_isolation_of_slot(SlotNumber(slot));
+        without_the_isolation.is_free(SlotNumber(slot))
+    })
+}
+
+/// C 的记录与账点名的、B 的现行账里仍分配着的槽，写行那次发布释放了它们；C 的根环槽被这个进程写掉之前它们被回收过（过了回收门槛，
+/// 抬 F 一次都没推），而每一次覆盖写之后它们都不空闲、也没被重新分配出去；写掉 C 那一槽的那次发布同样没取到它们。
+/// 判别力：回收之后补隔离不看水位之上没施加的记录点名的落点（变异「第37批 回收时补隔离不看水位之上没施加的记录点名的落点」，删掉
+/// `placements_the_isolation_still_protects` 里那一句 `protected.extend`）时，回收出来的那几个槽当场空闲，「不空闲」那一条红。
+#[test]
+fn a_slot_of_the_current_account_named_by_an_unapplied_record_stays_isolated_after_its_reclaim_until_the_root_ring_slot_of_that_record_is_overwritten(
+) {
+    let mut pool = build_pool("released-slot-named-by-an-unapplied-record");
+    overwrite_the_built_pool(
+        &mut pool,
+        &content_seeded_by(4100, 3),
+        InstanceGeneration(1),
+    );
+    let abandoned = overwrite_the_built_pool(
+        &mut pool,
+        &content_seeded_by(2500, 11),
+        InstanceGeneration(1),
+    );
+    let mounted =
+        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned);
+    assert_eq!(
+        (
+            abandoned.root.checkpoint_txg,
+            mounted.output.effective_root.checkpoint_txg,
+            mounted.output.instance
+        ),
+        (CheckpointTxg(5), CheckpointTxg(4), InstanceGeneration(2)),
+        "C 是 txg 5；恢复落到 B（txg 4），实例 2"
+    );
+    let row_publish_txg = mounted.output.row_publish.root().checkpoint_txg;
+    let floor_at_the_mount = pool.output.root.rollback_floor;
+    let released_by_the_row_publish: Vec<PlacementOnDevice> = pool
+        .allocator
+        .root_ring_occupancy()
+        .expect("可写挂载建的分配器装着根环表")
+        .placements_named_by_unapplied_journal_records()
+        .into_iter()
+        .filter(|placement| {
+            record_covering(&pool.allocator, placement)
+                .is_some_and(|record| record.is_released && record.generation == row_publish_txg)
+        })
+        .collect();
+    assert!(
+        !released_by_the_row_publish.is_empty(),
+        "C 的记录与账点名的落点里有 B 的现行账里仍分配着、写行那次发布（txg {row_publish_txg:?}）释放了的"
+    );
+    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
+    let ring_slot_of_c = target_for_publish(abandoned.root.checkpoint_txg, slots_per_region);
+    let overwrites_limit = 4 * slots_per_region.count();
+    let mut overwrites: u64 = 0;
+    let mut reclaimed_before_the_ring_slot_of_c_was_overwritten = false;
+    // 迭代上界 overwrites_limit（根环三区、每区 S 槽，一圈 3S 次发布之内必写到 C 那一槽）；跨轮携带「回收过」一个布尔；
+    // 出口是写掉 C 那一槽的那一次发布。
+    loop {
+        overwrites += 1;
+        let current_txg = overwrite_the_built_pool(
+            &mut pool,
+            &content_seeded_by(3000, usize::try_from(overwrites).expect("轮数装得进 usize")),
+            InstanceGeneration(2),
+        )
+        .root
+        .checkpoint_txg;
+        assert_eq!(
+            pool.output.root.rollback_floor, floor_at_the_mount,
+            "第 {overwrites} 次覆盖写（txg {current_txg:?}）：没推抬 F（回收只走根环的回收门槛）"
+        );
+        for placement in &released_by_the_row_publish {
+            assert!(
+                !record_covering(&pool.allocator, placement).is_some_and(|record| {
+                    !record.is_released && record.generation > row_publish_txg
+                }),
+                "第 {overwrites} 次覆盖写（txg {current_txg:?}）：C 的根环槽写掉之前（含写掉它的那一次发布）{placement:?} 没被重新分配"
+            );
+        }
+        if target_for_publish(current_txg, slots_per_region) == ring_slot_of_c {
+            break;
+        }
+        for placement in &released_by_the_row_publish {
+            if every_slot_of_the_placement_is_reclaimed(&pool.allocator, placement) {
+                reclaimed_before_the_ring_slot_of_c_was_overwritten = true;
+            }
+            assert!(
+                every_slot_of_the_placement_is_held(&pool.allocator, placement),
+                "第 {overwrites} 次覆盖写（txg {current_txg:?}）之后 C 那一槽还没被盖掉：{placement:?} 不空闲（回收了也照补隔离）"
+            );
+        }
+        assert!(
+            overwrites < overwrites_limit,
+            "{overwrites_limit} 次覆盖写之内这个进程写到 C 那一槽"
+        );
+    }
+    assert!(
+        reclaimed_before_the_ring_slot_of_c_was_overwritten,
+        "C 的根环槽写掉之前写行那次发布释放的槽过了回收门槛、被回收过（这一形才走到回收之后补隔离那一步）"
+    );
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_for_a_publish_missing_records_recomputed_on_the_crash_image.rs b/crates/singlefs-harness/tests/writable_mount_refusal_for_a_publish_missing_records_recomputed_on_the_crash_image.rs
new file mode 100644
index 0000000..2c37fc9
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_for_a_publish_missing_records_recomputed_on_the_crash_image.rs
@@ -0,0 +1,252 @@
+//! 「记录有缺口时拒可写」（D23（journal 的角色与格式） 已定项 14，F1 乙）那一拒在拒绝合法判法里归到单子上
+//! （`singlefs_harness::mount_transparency::RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`），
+//! 判据由判法在崩溃镜像上自己复算（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 那一行）：水位之上某次没施加的发布
+//! 读得出的记录本次发布内序号不连续或缺末条、且某个点名单元两份副本都验不过。
+//!
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次三条记录的发布 R（顺序写三个数据单元）。
+mod common_unwitnessed_publish;
+
+use common_unwitnessed_publish::{
+    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
+    record_one_publish_after_overwrites, PublishRecordedAsR, RangesReadingBackAllZero,
+    RecordedPublish,
+};
+use singlefs_core::journal::JournalRecord;
+use singlefs_core::mount::{
+    mount_writable, MountError, RollbackTarget, StillUnreadableAfterOneReread,
+};
+use singlefs_core::unit::unit_filesystem_identifier;
+use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite};
+use singlefs_harness::mount_transparency::{
+    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
+    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
+    RefusalConditionNotMet, RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
+};
+use singlefs_harness::segments::StepKind;
+
+const OVERWRITES_BEFORE_R: u64 = 5;
+
+fn recorded_publish_of_three_records() -> RecordedPublish {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
+    recorded
+}
+
+/// 写记录中途断电、只落了第一条记录（两份）：R 的单元都落了（单元在记录之前落盘），第二、三条记录、根槽与轮换都没落。
+fn crash_image_with_only_the_first_record_of_the_publish(recorded: &RecordedPublish) -> MemoryPool {
+    let first_record = recorded
+        .writes
+        .iter()
+        .position(|write| write.kind == StepKind::JournalRecord)
+        .expect("R 写了记录");
+    let mut crash_image = recorded.base.clone();
+    crash_image.apply_writes(&recorded.writes[..first_record]);
+    let first_record_writes: Vec<RetainedWrite> = recorded
+        .record_writes_of_the_record(1)
+        .into_iter()
+        .cloned()
+        .collect();
+    crash_image.apply_writes(&first_record_writes);
+    crash_image
+}
+
+/// R 第一条记录点名的第一个单元的两份写（两块盘各一份，按盘序）。
+fn unit_writes_of_the_first_unit_named_by_the_first_record(
+    recorded: &RecordedPublish,
+) -> Vec<&RetainedWrite> {
+    let first_record = JournalRecord::parse(
+        recorded.record_writes_of_the_record(1)[0]
+            .bytes()
+            .expect("记录写带字节"),
+        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier),
+    )
+    .expect("R 的第一条记录解得开");
+    let named = first_record
+        .named
+        .first()
+        .expect("R 的第一条记录点名了单元");
+    named
+        .locations
+        .iter()
+        .map(|location| {
+            recorded
+                .unit_writes()
+                .into_iter()
+                .find(|write| {
+                    write.device == location.device
+                        && write.offset == location.slot.to_device_offset()
+                })
+                .expect("点名的单元是 R 写出的")
+        })
+        .collect()
+}
+
+/// `image` 上 `writes` 那几段盘面真的写成全零（不是读回全零的那一层：崩溃镜像本身就是这样）。
+fn image_with_the_writes_zeroed(image: &MemoryPool, writes: &[&RetainedWrite]) -> MemoryPool {
+    let mut zeroed = image.clone();
+    for write in writes {
+        let length = usize::try_from(write.length_in_bytes()).expect("一次写的长度装得进 usize");
+        zeroed
+            .devices
+            .get_mut(&write.device)
+            .expect("写落在池里的一块盘上")
+            .write(write.offset, &vec![0u8; length]);
+    }
+    zeroed
+}
+
+/// 可写挂载在 `crash_image` 上读到 `ranges` 那几段全零：拒成 F1 那一个成员、点名 R，盘上逐字节不变；交回那一拒。
+fn refusal_of_the_writable_mount_for_the_missing_records_of_r(
+    recorded: &RecordedPublish,
+    crash_image: &MemoryPool,
+    ranges: &RangesReadingBackAllZero,
+) -> MountError {
+    let mut devices = devices_reading_back_all_zero_on(crash_image, ranges);
+    let refusal = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect_err("R 缺记录、读到的点名单元两份都验不过：重读一次仍这样，拒可写");
+    assert!(
+        matches!(
+            &refusal,
+            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
+                if matches!(
+                    still_unreadable.as_ref(),
+                    StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
+                        publish: RollbackTarget { instance, checkpoint_txg },
+                    } if (*instance, *checkpoint_txg) == recorded.key
+                )
+        ),
+        "拒因是 F1 那一个成员、点名 R {:?}：{refusal:?}",
+        recorded.key
+    );
+    assert!(
+        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes)
+            == *crash_image,
+        "在取号之前拒：两块盘逐字节不变"
+    );
+    refusal
+}
+
+/// 崩溃镜像上 R 只落了第一条记录，它点名的一个单元两份盘面都是全零：可写挂载拒成 F1，判法把这一拒归到单子上、在崩溃镜像上复算判据为真
+/// （读得出的序号只有 1、没有末条；那个点名单元两份都验不过），盘上逐字节不变、只读读出在允许集合里，拒得合法。
+/// 判别力：F1 那一拒照旧归「不在单子上」（变异「F1 的拒照旧归到不在 D23 单子上」）时判成 `ReasonNotOnTheD23List`，这一条红。
+#[test]
+fn a_refusal_for_a_publish_missing_records_whose_named_unit_fails_on_both_copies_of_the_crash_image_is_legal(
+) {
+    let recorded = recorded_publish_of_three_records();
+    let both_copies = unit_writes_of_the_first_unit_named_by_the_first_record(&recorded);
+    let crash_image = image_with_the_writes_zeroed(
+        &crash_image_with_only_the_first_record_of_the_publish(&recorded),
+        &both_copies,
+    );
+    let refusal = refusal_of_the_writable_mount_for_the_missing_records_of_r(
+        &recorded,
+        &crash_image,
+        &RangesReadingBackAllZero::default(),
+    );
+    let selected_version = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
+        .report
+        .effective_root;
+    assert_eq!(
+        selected_version,
+        Some(recorded.previous_key),
+        "R 整次不施加：崩溃镜像上只读恢复落到 P"
+    );
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &crash_image,
+            selected_version,
+            &refusal,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ),
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify
+        )
+    );
+}
+
+/// 同一拒（可写挂载读到那个点名单元两份都是全零），崩溃镜像上那两份却验得过：判据在崩溃镜像上复算为假，判法照报拒得不合法。
+/// 判别力：复算不看点名单元验不验得过（变异「F1 判据复算不看点名单元」）时只剩「缺记录」一半、判成合法，这一条红。
+#[test]
+fn a_refusal_for_a_publish_missing_records_is_illegal_on_a_crash_image_whose_named_units_verify() {
+    let recorded = recorded_publish_of_three_records();
+    let crash_image = crash_image_with_only_the_first_record_of_the_publish(&recorded);
+    let both_copies = unit_writes_of_the_first_unit_named_by_the_first_record(&recorded);
+    let refusal = refusal_of_the_writable_mount_for_the_missing_records_of_r(
+        &recorded,
+        &crash_image,
+        &RangesReadingBackAllZero::default().with_the_writes(&both_copies),
+    );
+    let selected_version = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
+        .report
+        .effective_root;
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &crash_image,
+            selected_version,
+            &refusal,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason:
+                RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            ],
+        }
+    );
+}
+
+/// 崩溃镜像上 R 的三条记录都在（序号 1、2、3，末条带标志），根槽与 R 写出的单元两份盘面都是全零：点名的单元验不过，但记录没有缺口。
+/// 拿一次报 F1 的拒去判（实现在这份盘面上不报它：判法不信拒因、自己复算），判据为假，拒得不合法。
+/// 判别力：复算不看序号与末条（变异「F1 判据复算不看记录缺口」）时只剩「点名单元验不过」一半、判成合法，这一条红。
+#[test]
+fn a_refusal_claimed_for_missing_records_is_illegal_on_a_crash_image_holding_every_record_of_the_publish(
+) {
+    let recorded = recorded_publish_of_three_records();
+    let mut zeroed_writes = recorded.unit_writes();
+    zeroed_writes.push(recorded.root_slot_write());
+    let crash_image = image_with_the_writes_zeroed(
+        &recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
+        &zeroed_writes,
+    );
+    let selected_version = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
+        .report
+        .effective_root;
+    assert_eq!(
+        selected_version,
+        Some(recorded.previous_key),
+        "R 的根槽全零、点名的单元验不过：R 不施加，只读恢复落到 P"
+    );
+    let claimed_refusal = MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
+        StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
+            publish: RollbackTarget {
+                instance: recorded.key.0,
+                checkpoint_txg: recorded.key.1,
+            },
+        },
+    ));
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &crash_image,
+            selected_version,
+            &claimed_refusal,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason:
+                RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            ],
+        }
+    );
+}
````

## 删除线第 37 批 X：mutations-append.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch37/x/patch/mutations-append.tsv）

````text
第37批 可写挂载时的预隔离不隔离水位之上没施加的记录点名的落点（双故障形里被抛弃根 C 的单元又被发出去，抬 F 之前 I-7.4 红）	crates/singlefs-core/src/mount.rs	    for placement in placements_named_by_unapplied_journal_records {	    for placement in placements_named_by_unapplied_journal_records.iter().take(0) {	-p singlefs-harness --test random_histories -- --include-ignored --exact crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units	crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units
第37批 回收时补隔离不看水位之上没施加的记录点名的落点（删掉 placements_the_isolation_still_protects 里那一句 protected.extend，现行账里被豁免、这次挂载释放又回收的槽当场空闲）	crates/singlefs-core/src/allocator.rs	        protected.extend(self.placements_named_by_unapplied_journal_records());\n		-p singlefs-harness --test writable_mount_keeps_a_released_slot_named_by_an_unapplied_record_isolated_until_its_root_ring_slot_is_overwritten -- a_slot_of_the_current_account_named_by_an_unapplied_record_stays_isolated_after_its_reclaim_until_the_root_ring_slot_of_that_record_is_overwritten	a_slot_of_the_current_account_named_by_an_unapplied_record_stays_isolated_after_its_reclaim_until_the_root_ring_slot_of_that_record_is_overwritten
第37批 F1 的拒照旧归到不在 D23 单子上（判法交 ReasonNotOnTheD23List，崩溃注入第二截把一次合法的 F1 拒报红）	crates/singlefs-harness/src/mount_transparency.rs	                    Some(RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify)	                    None	-p singlefs-harness --test writable_mount_refusal_for_a_publish_missing_records_recomputed_on_the_crash_image -- a_refusal_for_a_publish_missing_records_whose_named_unit_fails_on_both_copies_of_the_crash_image_is_legal	a_refusal_for_a_publish_missing_records_whose_named_unit_fails_on_both_copies_of_the_crash_image_is_legal
第37批 F1 判据复算不看点名单元（只剩记录缺口一半，崩溃镜像上点名单元验得过的 F1 拒判成合法）	crates/singlefs-harness/src/mount_transparency.rs	            !(ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish)\n                && some_named_unit_fails_on_both_copies\n	            !(ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish)\n	-p singlefs-harness --test writable_mount_refusal_for_a_publish_missing_records_recomputed_on_the_crash_image -- a_refusal_for_a_publish_missing_records_is_illegal_on_a_crash_image_whose_named_units_verify	a_refusal_for_a_publish_missing_records_is_illegal_on_a_crash_image_whose_named_units_verify
第37批 F1 判据复算不看记录缺口（只剩点名单元验不过一半，三条记录都在的崩溃镜像上报 F1 的拒判成合法）	crates/singlefs-harness/src/mount_transparency.rs	            !(ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish)\n                && some_named_unit_fails_on_both_copies\n	            some_named_unit_fails_on_both_copies\n	-p singlefs-harness --test writable_mount_refusal_for_a_publish_missing_records_recomputed_on_the_crash_image -- a_refusal_claimed_for_missing_records_is_illegal_on_a_crash_image_holding_every_record_of_the_publish	a_refusal_claimed_for_missing_records_is_illegal_on_a_crash_image_holding_every_record_of_the_publish
````

## 删除线第 37 批 X：mutations-replacements.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch37/x/patch/mutations-replacements.tsv）

````text
第33批 GPU 全零单元守卫照改之前拿 first_slot + 读的槽数 比空段终点（32 位加法），槽 2³² − 1 回绕、交回全零单元	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        || first_slot > first_slot_past_the_gap) {\n        return NONE;\n    }\n    let slots_left_in_the_gap = first_slot_past_the_gap - first_slot;\n    let whole_data_unit_fits = first_slot < location_word(before, 11u);\n    if (slots_left_in_the_gap < slots_read) {	        ) {\n        return NONE;\n    }\n    let slots_left_in_the_gap = first_slot_past_the_gap - first_slot;\n    let whole_data_unit_fits = first_slot < location_word(before, 11u);\n    if (first_slot_past_the_gap < first_slot + slots_read) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_in_thirty_two_bits_with_the_checksum_of_zeros	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_in_thirty_two_bits_with_the_checksum_of_zeros
第33批 GPU F-α 那一处拿按 16 KiB 读回的切片的 32 KiB 检查字去比（不另按 32 KiB 读），没写过而下一槽写过的槽判对得上 32 KiB 全零的 CRC	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        || slice_whole_read_crc_matches(read_at(device, slot_to_bytes(slot), DATA_UNIT_BYTES), checksum);	        || slice_whole_read_crc_matches(copy, checksum);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes
第32批 GPU 判器 F-零槽：读者对单元区里没写过的槽照旧读不出（墓碑树根条目类型段 7、第 1 条位置条目指着没写过的槽、校验和是全零 16 KiB 的 CRC 那条流上 GPU 多红 I-2.1）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    return select(0u, 1u, !whole_data_unit_fits);\n	    return NONE;\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_with_the_checksum_of_zeros	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_with_the_checksum_of_zeros
````

## 删除线第 38 批 Y：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch38/y/patch/crates.patch）

````diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 05fe473..bc5914f 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -391,6 +391,9 @@ pub enum StillUnreadableAfterOneReread {
     },
     /// 影子账读一条被抛弃根的账（树表或分配记录树）读不出、从中央映射也现算不成，重读一次仍是这样：拒可写
     /// （C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算那一条的最后一支，用户 2026-09-28 定）。
+    /// 可写挂载的 C 读一次没施加的发布（水位之上那几条记录拼的根，或比所选那一版新的可读根）的账读不出、那次发布读得出的记录又不齐时
+    /// 照同一份办，仍不成也报这一个，`abandoned_root` 点名那次发布——这次挂载取号之后它就被新实例抛弃（`C_MAPPING` 收窄，
+    /// 判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第三节第 1 条）。
     AccountOfAnAbandonedRoot { abandoned_root: RollbackTarget },
     /// 可写挂载读阶段：水位之上有一次没施加的发布 `publish`，读得出的那几条记录的「本次发布内序号」不是 1..N 连着或没有一条带末条标志，
     /// 而它们点名的单元又有一个验不过；在这次挂载的读缓存上重做读阶段一遍仍是这样（F1 乙，用户 2026-10-07 定「缺记录就拒可写」，
@@ -1521,7 +1524,8 @@ impl PlacementsOfAbandonedRoots<'_> {
 /// 凭据只读内存里的现行账，不读那一槽）；没有这样的记录（那几槽只是这一次读不出）就在任何写之前拒：挂载在取号之前拒可写、抬 F 拒这一次抬
 /// （抬 F 那一串只在根环表里没记着这条根时才读盘、才走到这一拒）
 /// （[`StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot`]）。候选根读不出就当它什么都不豁免（隔离只会多不会少）。
-/// 同一次也隔离水位之上没施加的 journal 记录点名的落点与那几次发布的账里的落点（`placements_named_by_unapplied_journal_records`，C694 的 C）：
+/// 同一次也隔离水位之上没施加的 journal 记录点名的落点、那几次发布的账里的落点与比所选那一版新的可读根的账里的落点
+/// （`placements_named_by_unapplied_journal_records`，C694 的 C 连同 `C_ACCOUNT`、`C_NEWER_ROOTS`、`C_MAPPING`）：
 /// 只豁免现行账里仍分配着的槽，不豁免候选集里的根引用的槽（根环线 K4-1，函数体里那一圈的注释），D28（挂载期承诺量） 已定项 1 第九项
 /// 按分配器实际隔离的槽数取，两部分都在里面。
 /// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销；
@@ -1775,12 +1779,19 @@ fn placements_named_by_unapplied_journal_records<Device: BlockDevice>(
 /// 水位之上每一次没施加的发布（同一 (实例代号, checkpoint_txg) 的记录），用它读得出的记录里计数器最小的那一条的新根段拼一条根
 /// （每条记录都带整次发布的新根段，D23（journal 的角色与格式） 已定项 15；[`crate::recovery::root_assembled_from_the_new_root_segment_of_the_record`]），
 /// 按影子账同一份读法（[`placements_referenced_by_root`]）读它的账，账里的落点按那次发布的 txg 在根环里对应的槽（[`target_for_publish`]）归组交回，
-/// 调用方并进 C 的集合、与记录点名的那一份同一个时机解开。账读不出、解不开的那次发布不交：那时它的单元多半也验不过，交 F1 乙。
+/// 调用方并进 C 的集合、与记录点名的那一份同一个时机解开。账读不出、解不开的那次发布：读得出的记录齐（序号 1..N 连着、带末条）就不交——
+/// 那时它的单元多半也验不过，交 F1 乙；不齐就照被抛弃根那一份从中央映射现算（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]，
+/// `C_MAPPING` 收窄，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H3 那一行与第三节第 1 条）。
 /// 它补的是 C 只看读得出的记录点名了什么那一处盲区：那次发布的根槽与它的某一条记录（末条、中间一条或第一条）读回全零、单元读得出时，
 /// F1 乙不拒，缺的那条记录单独点名的单元又不在别的记录里；账里是那次发布引用的全部落点。水位与 C 同一个（[`placements_named_by_unapplied_journal_records`]）。
+///
+/// # Errors
+/// 那一支现算也不成、重读一次仍不成、现行账里又没有 C694 的 B 那道凭据 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`
+/// （点名那次发布的 (实例代号, checkpoint_txg)），在任何写之前。
 #[allow(
     clippy::ptr_arg,
-    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
+    clippy::too_many_arguments,
+    reason = "ptr_arg：placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
 )]
 fn placements_in_the_accounts_of_unapplied_publishes<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
@@ -1788,33 +1799,51 @@ fn placements_in_the_accounts_of_unapplied_publishes<Device: BlockDevice>(
     records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
     selected_version: &RootRecord,
     slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
-) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
+    current_records: &[AllocationRecord],
+    before_the_one_reread: &mut BeforeTheOneReread<'_>,
+) -> Result<BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>, MountError> {
     // 记录表按 (实例代号, 计数器) 升序：同一次发布里先遇到的就是计数器最小的那一条。
-    let mut first_readable_record_of_each_publish: BTreeMap<
+    let mut readable_records_of_each_publish: BTreeMap<
         (InstanceGeneration, CheckpointTxg),
-        &crate::journal::JournalRecord,
+        Vec<&crate::journal::JournalRecord>,
     > = BTreeMap::new();
     for record in records.values().filter(|record| {
         (record.instance, record.checkpoint_txg)
             > (selected_version.instance, selected_version.checkpoint_txg)
     }) {
-        first_readable_record_of_each_publish
+        readable_records_of_each_publish
             .entry((record.instance, record.checkpoint_txg))
-            .or_insert(record);
+            .or_default()
+            .push(record);
     }
     let mut in_the_accounts_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> =
         BTreeMap::new();
-    for ((_instance, checkpoint_txg), record) in first_readable_record_of_each_publish {
+    for ((_instance, checkpoint_txg), records_of_the_publish) in readable_records_of_each_publish {
+        let first_readable_record = records_of_the_publish
+            .first()
+            .expect("按发布归组时每一组至少放进了一条记录");
         let root_of_the_publish =
             crate::recovery::root_assembled_from_the_new_root_segment_of_the_record(
-                record,
+                first_readable_record,
                 selected_version.filesystem_identifier,
             );
-        let Some(placements) =
-            placements_referenced_by_root(devices, unit_area_start, &root_of_the_publish)
-        else {
-            continue;
-        };
+        let placements =
+            match placements_referenced_by_root(devices, unit_area_start, &root_of_the_publish) {
+                Some(placements) => placements,
+                None if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(
+                    &records_of_the_publish,
+                ) =>
+                {
+                    placements_of_an_unapplied_publish_whose_account_is_unreadable(
+                        devices,
+                        unit_area_start,
+                        &root_of_the_publish,
+                        current_records,
+                        before_the_one_reread,
+                    )?
+                }
+                None => continue,
+            };
         in_the_accounts_by_ring_slot
             .entry(target_for_publish(checkpoint_txg, slots_per_region))
             .or_default()
@@ -1824,7 +1853,122 @@ fn placements_in_the_accounts_of_unapplied_publishes<Device: BlockDevice>(
                     .map(|(device, slot, span)| PlacementOnDevice { device, slot, span }),
             );
     }
-    in_the_accounts_by_ring_slot
+    Ok(in_the_accounts_by_ring_slot)
+}
+
+/// C 并上的较新可读根的账（`C_NEWER_ROOTS`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1 那一行与第三节第 1 条，
+/// 用户 2026-10-08 定「四形都修」）：根环里比所选那一版 `selected_version` 新（择根的序：checkpoint_txg 为主、实例代号破平局）、按最新那条根的实例表
+/// 有效（`is_abandoned` 判否）、读得出的根——F2 退回之后被退掉的那条根就是这样：它的根槽读得出、它自己的实例表里它有效，影子账只管被抛弃的根、
+/// 不管它；它的记录又全读不出时 C 那两份（记录点名的、记录拼根读的账）都是空的。照影子账同一份读法（[`placements_referenced_by_root`]）读它的账，
+/// 账里的落点按它的 txg 在根环里对应的槽（[`target_for_publish`]）归组交回，调用方并进 C 的集合、同一个时机解开。账读不出时照没施加的发布那一份办：
+/// 那次发布读得出的记录不齐就从中央映射现算（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]，仍不成拒），记录齐就不交（交 F1 乙）；
+/// 一条记录都读不出的，映射现算得出就交、现算不成不交也不拒（条款没写这一格，函数体里那一支的注释）。
+/// `roots` 是重建分配器读根环那一遍读出的根（影子账、根环表用的同一份）。
+///
+/// # Errors
+/// 同 [`placements_in_the_accounts_of_unapplied_publishes`]（点名那条根的 (实例代号, checkpoint_txg)）。
+#[allow(
+    clippy::ptr_arg,
+    clippy::too_many_arguments,
+    reason = "ptr_arg：placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、根环里读出的根、判被抛弃的谓词、读阶段扫出的记录、所选那一版、每区根环槽数、现行账、重读前的钩子"
+)]
+fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    unit_area_start: UnitAreaStart,
+    roots: &[RootRecord],
+    is_abandoned: &dyn Fn(&RootRecord) -> bool,
+    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
+    selected_version: &RootRecord,
+    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
+    current_records: &[AllocationRecord],
+    before_the_one_reread: &mut BeforeTheOneReread<'_>,
+) -> Result<BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>, MountError> {
+    let mut in_the_accounts_of_newer_roots_by_ring_slot: BTreeMap<
+        RootRingSlot,
+        Vec<PlacementOnDevice>,
+    > = BTreeMap::new();
+    for root in roots.iter().filter(|root| {
+        !is_abandoned(root)
+            && (root.checkpoint_txg, root.instance)
+                > (selected_version.checkpoint_txg, selected_version.instance)
+    }) {
+        let placements_in_the_newer_roots_account =
+            match placements_referenced_by_root(devices, unit_area_start, root) {
+                Some(placements) => placements,
+                None => {
+                    let readable_records_of_its_publish: Vec<&crate::journal::JournalRecord> =
+                        records
+                            .values()
+                            .filter(|record| {
+                                (record.instance, record.checkpoint_txg)
+                                    == (root.instance, root.checkpoint_txg)
+                            })
+                            .collect();
+                    if readable_records_of_its_publish.is_empty() {
+                        // 那次发布一条记录都读不出：「读得出的记录不齐」条款没写这一格（交主 agent，第 38 批报告）。照两条验收合起来取：
+                        // 映射现算得出就并进来（攻方 H1 里账读不出的那几格要它），现算不成不拒、不重读——那条根的单元已被后来一次没落根的挂载
+                        // 写掉、记录槽也被它的记录占了时（入库用例 `urJ` 那一形）映射本来就现算不成，拒了就再也做不成可写挂载。
+                        match placements_referenced_by_root_computed_from_the_mapping(
+                            devices,
+                            unit_area_start,
+                            root,
+                        ) {
+                            Some(computed) => computed,
+                            None => continue,
+                        }
+                    } else if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(
+                        &readable_records_of_its_publish,
+                    ) {
+                        placements_of_an_unapplied_publish_whose_account_is_unreadable(
+                            devices,
+                            unit_area_start,
+                            root,
+                            current_records,
+                            before_the_one_reread,
+                        )?
+                    } else {
+                        continue;
+                    }
+                }
+            };
+        in_the_accounts_of_newer_roots_by_ring_slot
+            .entry(target_for_publish(root.checkpoint_txg, slots_per_region))
+            .or_default()
+            .extend(
+                placements_in_the_newer_roots_account
+                    .into_iter()
+                    .map(|(device, slot, span)| PlacementOnDevice { device, slot, span }),
+            );
+    }
+    Ok(in_the_accounts_of_newer_roots_by_ring_slot)
+}
+
+/// 水位之上一次没施加的发布（或比所选那一版新的可读根）的账读不出、那次发布读得出的记录又不齐时，它引用的落点照被抛弃根那一份办
+/// （`C_MAPPING` 收窄，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1、H3 两行与第三节第 1 条）：
+/// 从中央映射现算、重读一次、看 C694 的 B 那道凭据（[`placements_of_an_abandoned_root_whose_account_is_unreadable`] 原样）。
+/// 记录齐的那几次发布不走这里（调用方判）：那时账读不出交 F1 乙，照被抛弃根那一份办会在单元全读不出、映射也现算不成的正常盘面上误拒。
+///
+/// # Errors
+/// 映射现算不成、重读一次仍不成、凭据也没有 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`，`abandoned_root` 点名
+/// `root_of_the_publish` 的 (实例代号, checkpoint_txg)——这次挂载取号之后它就被新实例抛弃，拒因照被抛弃根那一类写；在任何写之前。
+#[allow(
+    clippy::ptr_arg,
+    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
+)]
+fn placements_of_an_unapplied_publish_whose_account_is_unreadable<Device: BlockDevice>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    unit_area_start: UnitAreaStart,
+    root_of_the_publish: &RootRecord,
+    current_records: &[AllocationRecord],
+    before_the_one_reread: &mut BeforeTheOneReread<'_>,
+) -> Result<Vec<(DeviceIdentity, SlotNumber, u64)>, MountError> {
+    placements_of_an_abandoned_root_whose_account_is_unreadable(
+        devices,
+        unit_area_start,
+        root_of_the_publish,
+        current_records,
+        before_the_one_reread,
+    )
 }
 
 /// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮）：被抛弃根的树表或分配记录树读不出时，
@@ -2198,7 +2342,9 @@ fn txgs_ring_slots_bad_at_mount_can_hold(
 /// 最新那条根的实例表读不出、重读一次仍读不出（[`instance_table_of_the_newest_root_read_at_most_twice`]）——在读阶段之后、取号之前；
 /// 树表 0 条那一版的账重建不出（`allocator_of_version_without_file`）；
 /// 读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有 ⇒
-/// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2、R1 按槽并两遍）。
+/// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2、R1 按槽并两遍）；
+/// C 读一次没施加的发布或一条较新可读根的账读不出、那次发布读得出的记录不齐、从中央映射也现算不成、重读一次仍不成
+/// （[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`。
 /// 都在取号之前，盘上逐字节不变。
 #[allow(
     clippy::ptr_arg,
@@ -2476,7 +2622,8 @@ fn rebuilt_allocator<Device: BlockDevice>(
     );
     // C694 的 C（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条）：水位之上没施加的记录点名的落点与影子账同一次隔离、
     // 豁免同一个集合。影子账关着（只供测试）时它也不隔离：那一臂要的是「什么都不护」的对照。
-    // 同一份里再并上那几次没施加的发布的账里的落点（C 的账那一份，[`placements_in_the_accounts_of_unapplied_publishes`]），解开时机相同。
+    // 同一份里再并上那几次没施加的发布的账里的落点（C 的账那一份，[`placements_in_the_accounts_of_unapplied_publishes`]；账读不出、
+    // 读得出的记录不齐时从中央映射现算，仍不成就在这里拒），解开时机相同。
     let placements_named_by_unapplied_journal_records = match shadow_ledger {
         ShadowLedger::On => {
             let mut named_by_ring_slot = placements_named_by_unapplied_journal_records(
@@ -2493,13 +2640,35 @@ fn rebuilt_allocator<Device: BlockDevice>(
                     records,
                     selected_version,
                     slots_per_region,
-                )
+                    &current_records,
+                    before_the_one_reread,
+                )?
             {
                 named_by_ring_slot
                     .entry(ring_slot)
                     .or_default()
                     .extend(placements_in_the_account);
             }
+            // 再并上环里比所选那一版新、按实例表有效、读得出的根自己的账（`C_NEWER_ROOTS`，[`placements_in_the_accounts_of_newer_readable_roots`]）：
+            // F2 退回之后被退掉的那条根，影子账不管它，它的记录又读不出时前两份都是空的。同一个时机解开。
+            for (ring_slot, placements_in_the_newer_roots_account) in
+                placements_in_the_accounts_of_newer_readable_roots(
+                    devices,
+                    unit_area_start,
+                    &roots,
+                    &is_abandoned,
+                    records,
+                    selected_version,
+                    slots_per_region,
+                    &current_records,
+                    before_the_one_reread,
+                )?
+            {
+                named_by_ring_slot
+                    .entry(ring_slot)
+                    .or_default()
+                    .extend(placements_in_the_newer_roots_account);
+            }
             named_by_ring_slot
         }
         ShadowLedger::Off => BTreeMap::new(),
@@ -6156,25 +6325,12 @@ fn publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verif
     records_of_each_publish
         .into_iter()
         .filter(|(_, records_of_the_publish)| {
-            let mut ordinals: Vec<u32> = records_of_the_publish
-                .iter()
-                .map(|record| record.ordinal_within_publish.0)
-                .collect();
-            ordinals.sort_unstable();
-            let ordinals_run_from_one_without_a_gap = ordinals
-                .iter()
-                .zip(1..)
-                .all(|(ordinal, expected)| *ordinal == expected);
-            let carries_a_last_record = records_of_the_publish.iter().any(|record| {
-                record.place_in_publish
-                    == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
-            });
             let some_named_unit_does_not_verify = records_of_the_publish.iter().any(|record| {
                 !crate::recovery::every_named_unit_of_the_record_has_a_copy_that_verifies(
                     reader, record,
                 )
             });
-            !(ordinals_run_from_one_without_a_gap && carries_a_last_record)
+            readable_records_of_the_publish_leave_a_gap_or_miss_the_last(records_of_the_publish)
                 && some_named_unit_does_not_verify
         })
         .map(|((instance, checkpoint_txg), _)| RollbackTarget {
@@ -6184,6 +6340,29 @@ fn publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verif
         .collect()
 }
 
+/// 一次发布读得出的那几条记录（`records_of_the_publish`：同一 (实例代号, checkpoint_txg)、读阶段扫得出的）不齐：「本次发布内序号」不是 1..N 连着
+/// （I-8.9：一次发布的序号依次是 1..N），或其中没有一条带末条标志；一条都读不出也算不齐。F1 乙
+/// （[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]）与 C 的账读不出时从中央映射现算的那一支
+/// （`C_MAPPING` 收窄，[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）用同一份。只读内存里的记录。
+fn readable_records_of_the_publish_leave_a_gap_or_miss_the_last(
+    records_of_the_publish: &[&crate::journal::JournalRecord],
+) -> bool {
+    let mut ordinals: Vec<u32> = records_of_the_publish
+        .iter()
+        .map(|record| record.ordinal_within_publish.0)
+        .collect();
+    ordinals.sort_unstable();
+    let ordinals_run_from_one_without_a_gap = ordinals
+        .iter()
+        .zip(1..)
+        .all(|(ordinal, expected)| *ordinal == expected);
+    let carries_a_last_record = records_of_the_publish.iter().any(|record| {
+        record.place_in_publish
+            == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
+    });
+    !(ordinals_run_from_one_without_a_gap && carries_a_last_record)
+}
+
 /// 判据 N-配置（E158 第 3 次跑登记第 347 行）的读数：系统配置槽直接读盘（`system_configuration_slots_of_one_device_read_once`，每块盘两槽），
 /// 不经读缓存（第 4 次跑登记 5.1 第 1 条：重读那一遍也直接从盘上重读这几槽）；记录取读阶段那一遍扫出来的。
 fn newer_publish_witness<Reader: PoolReader + ?Sized>(
@@ -6377,6 +6556,51 @@ fn read_stage_settled_at_most_on_the_one_reread<Reader: PoolReader>(
     ))
 }
 
+/// F1 乙按 F2 退到的那一版再判一次（`F1_AFTER_F2`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H2 那一行与
+/// 第三节第 1 条）：读阶段判 F1 时的水位是退之前的所选那一版——所选根读得出、走读失败、退回的那一形里，被退掉的那次发布不在那时的水位之上，
+/// 一条都不判；退到 `version_fallen_back_to` 之后它在水位之上了。按退到的那一版当水位判
+/// （[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]，记录取读阶段扫出的那一份、点名单元直接读盘），
+/// 有那样一次发布就重读一次（重读之前调 `before_the_one_reread`，点名单元再直接读一遍盘），仍有就拒，与读阶段那一判同一个拒因。只读盘、不写。
+///
+/// # Errors
+/// 重读一次仍有 ⇒ `NewerStateStillUnreadableAfterOneReread(PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify)`（按 (实例代号, checkpoint_txg)
+/// 最小的那一次），在取号之前、盘上逐字节不变。
+fn refuse_publishes_above_the_version_fallen_back_to_missing_records_whose_named_units_do_not_verify<
+    Reader: PoolReader + ?Sized,
+>(
+    devices: &Reader,
+    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
+    version_fallen_back_to: &RootRecord,
+    before_the_one_reread: &mut BeforeTheOneReread<'_>,
+) -> Result<(), MountError> {
+    if publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(
+        devices,
+        records,
+        version_fallen_back_to,
+    )
+    .is_empty()
+    {
+        return Ok(());
+    }
+    before_the_one_reread.before_rereading();
+    match publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(
+        devices,
+        records,
+        version_fallen_back_to,
+    )
+    .first()
+    {
+        Some(publish) => Err(MountError::NewerStateStillUnreadableAfterOneReread(
+            Box::new(
+                StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
+                    publish: *publish,
+                },
+            ),
+        )),
+        None => Ok(()),
+    }
+}
+
 /// 同 [`mount_writable_with_test_only_switches`]，「重读一次」之前做什么也由调用方给（[`BeforeTheOneReread`]：只供测试的钩子，
 /// 用例在里面撤掉暂时的读故障；产品路径走 [`mount_writable`]，立即重读）。
 ///
@@ -6500,7 +6724,8 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     // 所选那一版建不出来（走读失败）、系统配置没见证它：退回见证到的那一版建上一版（F2 乙，用户 2026-10-07 定「三处都做」，与恢复、只读挂载
     // 同一份判法 `recovery::version_witnessed_by_the_system_configuration_to_fall_back_to`）。退过去不重放：所选根与所选那一版都换成它，
     // 扫环报告里施加的那一截清零（新实例写的上一个实例那一行 W 取 0，与落在一条根上、一条记录都没施加时相同）。
-    // 退过去的那一版也建不出来，照报所选那一版的失败。之后的一切（取号之前逐盘核、写行、影子账与 C）都按退到的那一版。
+    // 退过去的那一版也建不出来，照报所选那一版的失败。之后的一切（F1 乙再判一次、取号之前逐盘核、写行、影子账与 C）都按退到的那一版。
+    // 退到哪一版照 `F2_NEWEST`：[见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（判法在那个函数里）。
     let (chosen_root, effective_root, journal, selected_version_journal_position, previous) =
         match rebuild_previous_version(devices, unit_area_start, &effective_root, own_record) {
             Ok(previous) => (
@@ -6534,20 +6759,29 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
                     &witnessed_version,
                     record_standing_for_the_witnessed_version,
                 ) {
-                    Ok(previous) => (
-                        witnessed_version,
-                        witnessed_version,
-                        JournalScanReport {
-                            valid_records: journal.valid_records,
-                            above_water: journal.above_water,
-                            prefix_applied: 0,
-                            verification_passed: journal.verification_passed,
-                            verification_failed: journal.verification_failed,
-                            maximum_applied_transaction: 0,
-                        },
-                        witnessed_version_journal_position,
-                        previous,
-                    ),
+                    Ok(previous) => {
+                        // F1 乙按退到的那一版的水位再判一次（`F1_AFTER_F2`）：读阶段判 F1 时水位是退之前的所选那一版。
+                        refuse_publishes_above_the_version_fallen_back_to_missing_records_whose_named_units_do_not_verify(
+                            &*devices,
+                            &records,
+                            &witnessed_version,
+                            &mut before_the_one_reread,
+                        )?;
+                        (
+                            witnessed_version,
+                            witnessed_version,
+                            JournalScanReport {
+                                valid_records: journal.valid_records,
+                                above_water: journal.above_water,
+                                prefix_applied: 0,
+                                verification_passed: journal.verification_passed,
+                                verification_failed: journal.verification_failed,
+                                maximum_applied_transaction: 0,
+                            },
+                            witnessed_version_journal_position,
+                            previous,
+                        )
+                    }
                     Err(_failure_of_the_witnessed_version) => {
                         return Err(failure_of_the_selected_version)
                     }
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 84aeade..8689d22 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -5404,18 +5404,21 @@ pub fn rebuild_version(
 }
 
 /// 所选那一版走读失败之后退回哪一版（F2 乙，用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md`
-/// 第三节第 7 条）：系统配置没见证过所选那一版时，退回系统配置见证到的那一版本身——根环里 (checkpoint_txg, 实例代号) 与计数器 = c_见证 的
-/// 那条记录恰好相等、读得出、比所选根旧的那条根（`F2_EXACT`，判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行与
-/// 第三节第 1 条：不退到比它更旧的根，那样已确认的那一版根槽读回全零时会退过它、记录读不出时会退到被新实例抛弃了的根）；见证过、见证判不出、
-/// c_见证 = 0（没有见证到的记录）、那条记录读不出、或那条根读不出时 `None`（照报原来的失败）。恢复（[`recover`]）、可写挂载读阶段之后建上一版那一步（`mount`）、只读挂载择根
+/// 第三节第 7 条）：系统配置没见证过所选那一版时，退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（`F2_NEWEST`，
+/// 判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H4 那一行与第三节第 1 条，用户 2026-10-08 定「四形都修」）——
+/// 见证到的那一版 = 根环里 (checkpoint_txg, 实例代号) 与计数器 = c_见证 的那条记录相等的那一版；不退到比它更旧的根（`F2_EXACT`，
+/// 判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行与第三节第 1 条：那样已确认的那一版根槽读回全零时会退过它、
+/// 记录读不出时会退到被新实例抛弃了的根），也不停在它而跳过它之后一条已确认、读得出的根（P 那次轮换写的较新系统配置槽读回全零时 c_见证 落回 P 之前）。
+/// 所选根的实例表读不出时只认与见证到的那一版恰好相等的那条；c_见证 落在所选那次发布自己的某条记录上时那一段是空的、不退（所选那一版算见证过）。
+/// 见证过、见证判不出、c_见证 = 0（没有见证到的记录）、那条记录读不出、或那一段里没有读得出、不被抛弃的根时 `None`（照报原来的失败）。
+/// 恢复（[`recover`]）、可写挂载读阶段之后建上一版那一步（`mount`）、只读挂载择根
 /// （`mounted_read::mount_read_only`）三处都调这一份，退过去之后不重放、直接走那一版。所选那一版是施加过记录、由记录重建的
 /// （`selected_version` 不是 `chosen_root`）时不退：条款写的是「所选根走读失败」，施加过记录的那一版走不通条款没写（函数体第一句）。
 ///
 /// 「见证过」的判法取攻方 `strict`（判决同一条）：c_见证 = 每块盘两槽里自证过的系统配置 journal tail 的最大值（与可写挂载判据 N-配置
 /// 同一个读数，`mount::newer_publish_witness`）。所选那一版那次发布带末条标志的记录读得出（任一份）⇒ 它的计数器 ≤ c_见证 即见证过；
 /// 读不出而计数器 = c_见证 的那条记录读得出 ⇒ 它的 (checkpoint_txg, 实例代号) 不比所选那一版旧即见证过；两条都读不出当见证过、不退。
-/// 退到哪一条：计数器 = c_见证 的那条记录的 (checkpoint_txg, 实例代号) 恰好相等的那条根。根环里的根按择根同一个序
-/// （checkpoint_txg 为主、实例代号破平局）比。
+/// 退到哪一条：上面那一段里最新的那条。根环里的根按择根同一个序（checkpoint_txg 为主、实例代号破平局）比。
 ///
 /// 系统配置见证判不出（R2：有槽设备报读错、又没有一块盘两槽都自证过，[`system_configuration_witness_is_undecidable`]）时不退：
 /// 条款只写了「系统配置没见证它时退」，判不出算不算没见证没写；这一支照报原来的失败（与今天相同），交主 agent 定。
@@ -5493,10 +5496,14 @@ pub fn version_witnessed_by_the_system_configuration_to_fall_back_to<
     if is_witnessed {
         return None;
     }
-    // 只退到见证到的那一版本身（`F2_EXACT`）：计数器 = c_见证 的那条记录读不出就不退；c_见证 = 0 时也走这一支——计数器从 1 起
-    // （`journal::FIRST_JOURNAL_COUNTER`），环里没有计数器 0 的记录。
+    // 计数器 = c_见证 的那条记录读不出就不退；c_见证 = 0 时也走这一支——计数器从 1 起（`journal::FIRST_JOURNAL_COUNTER`），环里没有计数器 0 的记录。
     let witnessed_publish = record_at_the_witnessed_counter?;
     let chosen = order_of_the_root_choice(chosen_root);
+    // 退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（`F2_NEWEST`）。c_见证 落在所选那次发布自己的某条记录上时
+    // 这一段是空的：所选那一版算见证过、不退。所选根的实例表读不出时滤不了，只认与见证到的那一版恰好相等的那条（`F2_EXACT` 那一取法）。
+    let instance_table_of_the_chosen_root = instance_table_chain_of_root(reader, chosen_root)
+        .ok()
+        .map(|chain| chain.records);
     readable_roots(
         reader,
         &system_configuration.immutable.region_devices,
@@ -5504,14 +5511,22 @@ pub fn version_witnessed_by_the_system_configuration_to_fall_back_to<
         &system_configuration.immutable.filesystem_identifier,
     )
     .into_iter()
-    .find(|root| {
+    .filter(|root| {
         let order = order_of_the_root_choice(root);
-        order == witnessed_publish && order < chosen
+        let kept_by_the_instance_table_of_the_chosen_root = match &instance_table_of_the_chosen_root
+        {
+            Some(table) => !root_is_abandoned_by_the_instance_table(root, table),
+            None => order == witnessed_publish,
+        };
+        witnessed_publish <= order
+            && order < chosen
+            && kept_by_the_instance_table_of_the_chosen_root
     })
+    .max_by_key(order_of_the_root_choice)
 }
 
-/// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。所选那一版走读失败、系统配置没见证它时退回见证到的那一版
-/// （[`version_witnessed_by_the_system_configuration_to_fall_back_to`]）：那一版走得通，`root` 与 `effective_root` 都报退到的那一条，
+/// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。所选那一版走读失败、系统配置没见证它时退到
+/// [见证到的那一版, 所选根) 里读得出、不被抛弃的最新一条根（[`version_witnessed_by_the_system_configuration_to_fall_back_to`]）：那一版走得通，`root` 与 `effective_root` 都报退到的那一条，
 /// `journal` 仍是对所选根扫环、施加那一趟的报告；走不通照报所选那一版的失败。不看 journal 的那一臂（`JournalPolicy::Ignore`）不退。
 #[must_use]
 pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {
diff --git a/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs b/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs
index 2f8d112..a170c26 100644
--- a/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs
+++ b/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs
@@ -468,3 +468,159 @@ pub fn units_of_r_overwritten_on(
         .map(|write| (write.device, write.offset.0))
         .collect()
 }
+
+impl RecordedPublish {
+    /// R 的根记录（R 的根槽写解出来的）。
+    ///
+    /// # Panics
+    /// 根槽写不带字节，或解不开、自证不过。
+    #[must_use]
+    pub fn root(&self) -> singlefs_core::root_record::RootRecord {
+        singlefs_core::root_record::RootRecord::parse_slot(
+            self.root_slot_write().bytes().expect("根槽写带字节"),
+            &self.pool.parameters.filesystem_identifier,
+        )
+        .expect("R 的根自证")
+    }
+
+    /// R 写出的单元写里落在 `pointer` 那两条位置条目上的几份（R 自己写了那个单元时每块盘一份；那个单元是照抄上一版的就一份都没有）。
+    #[must_use]
+    pub fn unit_writes_at(
+        &self,
+        pointer: &singlefs_core::pointer::NodePointer,
+    ) -> Vec<&RetainedWrite> {
+        self.unit_writes()
+            .into_iter()
+            .filter(|write| {
+                pointer.locations.iter().any(|location| {
+                    location.device == write.device
+                        && location.slot.to_device_offset() == write.offset
+                })
+            })
+            .collect()
+    }
+
+    /// R 写出的单元按单元归组：同一个单元的几份字节相同、落在不同的盘上；组按第一份的落盘次序。
+    ///
+    /// # Panics
+    /// 单元写不带字节（单元写按构造是普通写）。
+    #[must_use]
+    pub fn unit_writes_grouped_by_unit(&self) -> Vec<Vec<&RetainedWrite>> {
+        let mut groups: Vec<Vec<&RetainedWrite>> = Vec::new();
+        for write in self.unit_writes() {
+            let bytes = write.bytes().expect("单元写带字节");
+            match groups.iter_mut().find(|group| {
+                group[0].bytes() == Some(bytes)
+                    && group.iter().all(|copy| copy.device != write.device)
+            }) {
+                Some(group) => group.push(write),
+                None => groups.push(vec![write]),
+            }
+        }
+        groups
+    }
+}
+
+/// `image` 的根环里有自证过的、(实例, txg) 为 `key` 的那条根。
+///
+/// # Panics
+/// `image` 上择不到系统配置。
+#[must_use]
+pub fn root_is_in_the_ring(image: &MemoryPool, key: (InstanceGeneration, CheckpointTxg)) -> bool {
+    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
+    let sizes = &configuration.immutable.sizes;
+    every_root_ring_slot(sizes).into_iter().any(|ring_slot| {
+        matches!(
+            read_root_ring_slot(
+                image,
+                &configuration.immutable.region_devices,
+                sizes,
+                &configuration.immutable.filesystem_identifier,
+                ring_slot,
+            ),
+            RootRingSlotReading::SelfVerified(root) if (root.instance, root.checkpoint_txg) == key
+        )
+    })
+}
+
+/// M1 之后依次做的三次用户动作（攻方 `research/prompts/m3-batch35-36-code-r1-opus-output.md` 第一节放开用户动作那一扫的 o、n、s 三种）。
+#[derive(Clone, Copy, Debug)]
+enum UserActionAfterTheMount {
+    Overwrite,
+    CreateOneInode,
+    SequentialWriteOfTwoDataUnits,
+}
+
+/// 取新实例的那次可写挂载 M1（`crash_image` 上 `ranges` 那几段读回全零）：做成就核 R 的单元没被写掉——先核写行那次发布的根落盘之前那一刻
+/// （`crash_image` 上施加 M1 写到那时的全部写）R 的单元一份不少；再在 M1 之后的池上依次覆盖写一次、建一个 inode、顺序写两个数据单元，
+/// 每一步之后 R 的根还在根环里就核 R 的单元一份不少；交回 M1 落到的那一版。被拒就核拒在任何写之前（录下来的写一条都没有），原样交回拒因。
+/// `cell` 只用在断言消息里，说是哪一格。
+///
+/// # Panics
+/// 断言不成立（写掉了 R 的单元，或被拒时已经写了盘）；M1 没有写行那次发布的根槽写；三次用户动作有一次被拒（384 槽小盘上都落得下）。
+pub fn mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
+    recorded: &RecordedPublish,
+    crash_image: &MemoryPool,
+    ranges: &RangesReadingBackAllZero,
+    cell: &str,
+) -> Result<(InstanceGeneration, CheckpointTxg), singlefs_core::mount::MountError> {
+    let stream = SharedStream::retaining_contents();
+    let mut devices = recorded_devices_reading_back_all_zero_on(crash_image, ranges, &stream);
+    let mounted =
+        match singlefs_core::mount::mount_writable(&recorded.pool.parameters, &mut devices) {
+            Ok(mounted) => mounted,
+            Err(refusal) => {
+                assert_eq!(
+                    stream.operation_count(),
+                    0,
+                    "{cell}：可写挂载被拒（{refusal:?}），拒在任何写之前"
+                );
+                return Err(refusal);
+            }
+        };
+    let effective = (
+        mounted.output.effective_root.instance,
+        mounted.output.effective_root.checkpoint_txg,
+    );
+    let (mount_writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &stream.retained_operations(),
+        &fixed_geometry_of_the_384_slot_small_devices(),
+    );
+    let row_root = mount_writes
+        .iter()
+        .position(|write| write.kind == StepKind::RootRecordFua)
+        .expect("M1 写行那次发布有根槽写");
+    let mut image_before_the_row_root = crash_image.clone();
+    image_before_the_row_root.apply_writes(&mount_writes[..row_root]);
+    assert_eq!(
+        units_of_r_overwritten_on(recorded, &image_before_the_row_root),
+        Vec::<(DeviceIdentity, u64)>::new(),
+        "{cell}：写行那次发布的根落盘之前，M1 写下的一切都没碰 R 的单元"
+    );
+    let mut image_after_the_mount = crash_image.clone();
+    image_after_the_mount.apply_writes(&mount_writes);
+    let mut pool = pool_after_the_mount(recorded, &image_after_the_mount, mounted);
+    for user_action in [
+        UserActionAfterTheMount::Overwrite,
+        UserActionAfterTheMount::CreateOneInode,
+        UserActionAfterTheMount::SequentialWriteOfTwoDataUnits,
+    ] {
+        match user_action {
+            UserActionAfterTheMount::Overwrite => pool.overwrite(OVERWRITE_BYTES).map(|_| ()),
+            UserActionAfterTheMount::CreateOneInode => pool.create_one_inode().map(|_| ()),
+            UserActionAfterTheMount::SequentialWriteOfTwoDataUnits => pool
+                .sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
+                .map(|_| ()),
+        }
+        .unwrap_or_else(|refusal| panic!("{cell}：M1 之后{user_action:?}落得下：{refusal:?}"));
+        let image = pool.image();
+        if root_is_in_the_ring(&image, recorded.key) {
+            assert_eq!(
+                units_of_r_overwritten_on(recorded, &image),
+                Vec::<(DeviceIdentity, u64)>::new(),
+                "{cell}：M1 之后{user_action:?}，R 的根还在根环里，R 的单元一份都没被写掉"
+            );
+        }
+    }
+    Ok(effective)
+}
diff --git a/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs b/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs
index b1189a3..8d551b0 100644
--- a/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs
+++ b/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs
@@ -1,9 +1,10 @@
 //! F2 乙（用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 7 条）：所选那一版走读失败、系统配置没见证它时，
 //! 退回系统配置见证到的那一版；恢复（`recovery::recover`）、可写挂载读阶段之后建上一版那一步、只读挂载（`mounted_read::mount_read_only`）三处同一份判法
 //! （`recovery::version_witnessed_by_the_system_configuration_to_fall_back_to`）。「见证过」照攻方 `strict`：所选那一版带末条标志的记录读得出 ⇒
-//! 计数器 ≤ c_见证；读不出 ⇒ 拿计数器 = c_见证 的那条记录比 (txg, 实例)；两条都读不出当见证过、不退。退只退到见证到的那一版本身
-//! （`F2_EXACT`，判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行与第三节第 1 条）：根环里与计数器 = c_见证 的那条记录
-//! (txg, 实例) 恰好相等、读得出的那条根；那条记录读不出、或那条根读不出，都不退。
+//! 计数器 ≤ c_见证；读不出 ⇒ 拿计数器 = c_见证 的那条记录比 (txg, 实例)；两条都读不出当见证过、不退。退到 [见证到的那一版, 所选根) 里读得出、
+//! 不被所选根的实例表抛弃的最新一条根（`F2_NEWEST`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H4 那一行与第三节第 1 条），
+//! 所选根的实例表读不出时只认与见证到的那一版恰好相等的那条；不退到比见证到的那一版更旧的根（`F2_EXACT`，判决
+//! `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行与第三节第 1 条）；计数器 = c_见证 的那条记录读不出、或那一段里没有读得出的根，都不退。
 //!
 //! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次覆盖写 R（P 是它之前那一版，已确认）。
 //! - 第二轮攻方 5.2 两形：崩在 R 的根槽落盘之后、系统配置轮换之前；取新实例的那次挂载里 R 的根槽与单元读回全零（`ur`）或连 R 的记录也读回全零
@@ -14,6 +15,9 @@
 //! - 代码轮攻方（`research/prompts/m3-batch33t-code-r1-opus-output.md` 第一节）两形：A——崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元与 P 的根槽
 //!   读回全零（见证到的就是 P，P 的根读不出）；B——取新实例的那次挂载里 `ur`、崩在那次挂载写行那次发布的根槽之后，下一次读时写行那次发布的单元
 //!   与 P 的末条记录（计数器 = c_见证 的那一条）读回全零。两形都照报读不出，不退到 P 之前那一版、也不退到被新实例抛弃的 R。
+//! - 代码轮攻方（`research/prompts/m3-batch35-36-code-r1-opus-output.md` 第四节）H4：崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元与 P 那次轮换写的
+//!   较新系统配置槽读回全零，c_见证 落回 P 之前那一版；三处都落到已确认的 P，四组（两种 R × 覆盖写 3、5 次）各一条。B 形那一盘上 P 的末条读得出时
+//!   退到的根按 Row 的实例表滤（跳过被抛弃的 R），Row 的实例表读不出时只认恰好相等的 P。
 mod common_unwitnessed_publish;
 
 use common_unwitnessed_publish::{
@@ -22,12 +26,21 @@ use common_unwitnessed_publish::{
     recorded_devices_reading_back_all_zero_on, units_of_r_overwritten_on, PublishRecordedAsR,
     RangesReadingBackAllZero, RecordedPublish,
 };
-use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
 use singlefs_core::admission::SpaceAdmission;
-use singlefs_core::mount::{mount_writable, mount_writable_with_test_only_switches, ShadowLedger};
+use singlefs_core::mount::{
+    mount_writable, mount_writable_with_test_only_switches, MountError, ShadowLedger,
+    StillUnreadableAfterOneReread,
+};
 use singlefs_core::mounted_read::mount_read_only;
-use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
+use singlefs_core::recovery::{
+    choose_system_configuration, recover, JournalPolicy, RecoveryOutcome,
+};
 use singlefs_core::root_record::RootRecord;
+use singlefs_core::system_configuration::SystemConfiguration;
+use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
 use singlefs_harness::memory_pool::{
     writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite,
 };
@@ -337,3 +350,232 @@ fn a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_z
         "只读挂载同样读回 P"
     );
 }
+
+/// 每块盘上较新的那个系统配置槽（两槽里自证过、槽世代号大的那一槽）：盘、偏移、长度。
+fn newer_system_configuration_slot_of_each_device(
+    image: &MemoryPool,
+) -> Vec<(DeviceIdentity, u64, u64)> {
+    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
+    let slot_spacing_in_bytes =
+        u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
+    let slot_bytes =
+        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节");
+    image
+        .devices
+        .iter()
+        .map(|(device, sparse_device)| {
+            let newer_slot_offset = [0, slot_spacing_in_bytes]
+                .into_iter()
+                .filter_map(|offset| {
+                    SystemConfiguration::parse_slot(
+                        &sparse_device.read(DeviceOffsetInBytes(offset), slot_bytes),
+                    )
+                    .ok()
+                    .map(|slot| (slot.quantities.slot_generation, offset))
+                })
+                .max()
+                .map(|(_, offset)| offset)
+                .expect("每块盘至少一槽自证过");
+            (*device, newer_slot_offset, SYSTEM_CONFIGURATION_SLOT_BYTES)
+        })
+        .collect()
+}
+
+/// 攻方 H4（`research/prompts/m3-batch35-36-code-r1-opus-output.md` 第四节）：崩在 R 的根槽之后、轮换之前（R 没见证；P 已确认，它那次轮换写的
+/// 系统配置槽在每块盘上都是较新的那一槽），读的那一刻 R 的单元与每块盘较新的那个系统配置槽读回全零。c_见证 落回 P 之前那一版 Pm 的计数器，
+/// R 走读失败、没见证 ⇒ 退到 [Pm, R) 里读得出、不被 R 的实例表抛弃的最新一条根，就是已确认、单元完好的 P（`F2_NEWEST`），三处都落到 P。
+/// 判别力：只退到见证到的那一版本身（变异「删除线第38批 F2_NEWEST 只退到见证到的那一版本身（H4：落到 Pm、跳过已确认的 P）」）时三处都落到 Pm、这一条红。
+fn assert_the_units_of_r_and_the_newer_system_configuration_slots_reading_back_zero_fall_back_to_the_previous_version(
+    publish: PublishRecordedAsR,
+    overwrites_before_r: u64,
+) {
+    let recorded = record_one_publish_after_overwrites(overwrites_before_r, publish);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let ranges =
+        RangesReadingBackAllZero(newer_system_configuration_slot_of_each_device(&crash_image))
+            .with_the_writes(&recorded.unit_writes());
+    assert_recovery_and_both_mounts_read_back_the_previous_version(
+        &recorded,
+        &crash_image,
+        &ranges,
+    );
+}
+
+/// H4 那一组：一条记录的覆盖写 R、R 之前覆盖写 3 次。
+#[test]
+fn a_root_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_three_overwrites(
+) {
+    assert_the_units_of_r_and_the_newer_system_configuration_slots_reading_back_zero_fall_back_to_the_previous_version(
+        PublishRecordedAsR::Overwrite,
+        3,
+    );
+}
+
+/// H4 那一组：一条记录的覆盖写 R、R 之前覆盖写 5 次。
+#[test]
+fn a_root_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_five_overwrites(
+) {
+    assert_the_units_of_r_and_the_newer_system_configuration_slots_reading_back_zero_fall_back_to_the_previous_version(
+        PublishRecordedAsR::Overwrite,
+        5,
+    );
+}
+
+/// H4 那一组：三条记录的顺序写 R、R 之前覆盖写 3 次。
+#[test]
+fn a_root_of_three_records_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_three_overwrites(
+) {
+    assert_the_units_of_r_and_the_newer_system_configuration_slots_reading_back_zero_fall_back_to_the_previous_version(
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+        3,
+    );
+}
+
+/// H4 那一组：三条记录的顺序写 R、R 之前覆盖写 5 次。
+#[test]
+fn a_root_of_three_records_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_five_overwrites(
+) {
+    assert_the_units_of_r_and_the_newer_system_configuration_slots_reading_back_zero_fall_back_to_the_previous_version(
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+        5,
+    );
+}
+
+/// 攻方 B 形那一盘（取新实例的那次挂载 M1 里 `ur`，M1 落在 P、取号，崩在写行那次发布 Row 的根槽之后、轮换之前），下一次读时只让 Row 写出的
+/// 单元读回全零（`keep_the_instance_table_of_row` 为真时留下 Row 的实例表那一片照读），P 的末条读得出：Row 走读失败、没见证，见证到的是 P
+/// （c_见证 是 M1 取号那一写写进去的 P 的计数器）。[P, Row) 里有 P 与 M1 按实例表抛弃了的 R，R 更新。交回那一刻的盘面与读回全零的几段。
+fn image_after_a_row_publish_abandoning_r_with_the_units_of_row_reading_back_zero(
+    recorded: &RecordedPublish,
+    keep_the_instance_table_of_row: bool,
+) -> (
+    MemoryPool,
+    RangesReadingBackAllZero,
+    (InstanceGeneration, CheckpointTxg),
+) {
+    let ranges_at_the_mount = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.unit_writes());
+    let (mount_writes, row_root) = writes_of_the_mount_that_took_a_new_instance(
+        recorded,
+        &ranges_at_the_mount,
+        ShadowLedger::On,
+    );
+    let mut image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    image.apply_writes(&mount_writes[..=row_root]);
+    let row = RootRecord::parse_slot(
+        mount_writes[row_root].bytes().expect("根槽写带字节"),
+        &recorded.pool.parameters.filesystem_identifier,
+    )
+    .expect("写行那次发布的根自证");
+    let row_unit_writes: Vec<&RetainedWrite> = mount_writes[..row_root]
+        .iter()
+        .filter(|write| write.kind == StepKind::UnitWrite)
+        .filter(|write| {
+            !keep_the_instance_table_of_row
+                || !row.instance_table.locations.iter().any(|location| {
+                    location.device == write.device
+                        && location.slot.to_device_offset() == write.offset
+                })
+        })
+        .collect();
+    (
+        image,
+        RangesReadingBackAllZero::default().with_the_writes(&row_unit_writes),
+        (row.instance, row.checkpoint_txg),
+    )
+}
+
+/// 退到的根按所选根的实例表滤（`F2_NEWEST`，正推「退到的根不按实例表滤」那一处写定）：B 形那一盘上只让 Row 写出的单元（实例表那一片除外）读回全零。
+/// Row 的实例表读得出、里面有 R 那个实例的行，R 的 txg 大于行里的 ⇒ R 被抛弃，[P, Row) 里不被抛弃的最新一条是 P：三处都退到 P，
+/// 不退到被新实例抛弃了的 R。判别力：不按实例表滤（变异「删除线第38批 F2_NEWEST 退到的根不按所选根的实例表滤（退到被抛弃的 R）」）时三处读回 R、这一条红。
+#[test]
+fn a_row_publish_whose_units_read_back_zero_falls_back_past_the_root_its_instance_table_abandoned_to_the_witnessed_version(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let (image, ranges, row_key) =
+        image_after_a_row_publish_abandoning_r_with_the_units_of_row_reading_back_zero(
+            &recorded, true,
+        );
+    let report = recover(
+        &devices_reading_back_all_zero_on(&image, &RangesReadingBackAllZero::default()),
+        JournalPolicy::Consult,
+    );
+    assert!(
+        matches!(report.outcome, RecoveryOutcome::FileRead { root, .. } if root == row_key),
+        "没有读故障时恢复择到 Row、走得通：{:?}",
+        report.outcome
+    );
+    assert_recovery_and_both_mounts_read_back_the_previous_version(&recorded, &image, &ranges);
+}
+
+/// 所选根的实例表读不出时只认恰好相等的那条：B 形那一盘上 Row 写出的单元连同它的实例表都读回全零。滤不了，只认与见证到的 P 恰好相等的那条：
+/// 恢复与只读挂载退到 P；可写挂载判抛弃用的最新那条根（Row）的实例表重读一次仍读不出，在取号之前拒（那一拒在 D23 单子上，盘上逐字节不变）。
+/// 判别力：实例表读不出时照收 [P, Row) 里最新的一条（变异「删除线第38批 F2_NEWEST 所选根的实例表读不出时照收最新的一条（退到被抛弃的 R）」）
+/// 时恢复读回 R、这一条红。
+#[test]
+fn a_row_publish_whose_units_and_instance_table_read_back_zero_falls_back_only_to_the_exactly_witnessed_version(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let (image, ranges, _row_key) =
+        image_after_a_row_publish_abandoning_r_with_the_units_of_row_reading_back_zero(
+            &recorded, false,
+        );
+    let devices = devices_reading_back_all_zero_on(&image, &ranges);
+    let report = recover(&devices, JournalPolicy::Consult);
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::FileRead {
+            root: recorded.previous_key,
+            content: recorded.previous_content.clone(),
+        },
+        "恢复：Row 的实例表读不出，只认与见证到的 P 恰好相等的那条，不退到被抛弃的 R"
+    );
+    let read_only = mount_read_only(&devices).expect("只读挂载退回 P、打得开");
+    assert_eq!(
+        (
+            read_only.effective_root.instance,
+            read_only.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "只读挂载：退回 P"
+    );
+    let mut devices_of_the_mount = devices_reading_back_all_zero_on(&image, &ranges);
+    let refusal = mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount)
+        .expect_err("判抛弃用的最新那条根的实例表读不出：可写挂载拒");
+    assert!(
+        matches!(
+            &refusal,
+            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
+                if matches!(
+                    **still_unreadable,
+                    StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. }
+                )
+        ),
+        "拒因是最新那条根的实例表读不出：{refusal:?}"
+    );
+    assert!(
+        image_of_devices_reading_back_all_zero(&devices_of_the_mount, recorded.pool.device_bytes)
+            == image,
+        "可写挂载在任何写之前拒：两块盘逐字节不变"
+    );
+}
+
+/// 见证记录读不出照旧不退（`F2_NEWEST` 之下）：B 形那一盘上 Row 写出的单元（实例表那一片除外）与 P 的末条记录（计数器 = c_见证 的那一条）读回全零。
+/// Row 的实例表读得出、滤得了，[P, Row) 里不被抛弃的最新一条是 P——可那条见证记录读不出，见证到的是哪一版说不出，三处照报 Row 读不出。
+/// 判别力：见证记录读不出时下限放到最旧（变异「删除线第38批 F2_NEWEST 见证记录读不出时照退（下限放到最旧，退到 P）」）时三处读回 P、这一条红。
+#[test]
+fn a_row_publish_whose_units_and_the_witnessed_last_record_read_back_zero_is_reported_unreadable_even_with_its_instance_table_readable(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let (image, ranges, row_key) =
+        image_after_a_row_publish_abandoning_r_with_the_units_of_row_reading_back_zero(
+            &recorded, true,
+        );
+    let ranges = ranges.with_the_last_record_of_the_publish(&image, recorded.previous_key);
+    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
+        &recorded, &image, &ranges, row_key,
+    );
+}
diff --git a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
index fec11b7..d52703c 100644
--- a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
+++ b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
@@ -1153,15 +1153,15 @@ fn pool_after_a_recovery_abandoned_the_third_version_and_four_overwrites(tag: &s
     pool
 }
 
-/// 双故障形（C583（双故障形：每盘一槽系统配置坏加最新根一时读不出），已知，留里程碑三）：崩溃恢复抛弃 C（实例 1、txg 5）之后实例 2 覆盖写四次，
-/// 那次恢复看不见 C、把 C 引用的单元又发出去，抬 F 之前池级 checker 就恰好红一条 I-7.4（近 K 代块未被复用），说明文字点名实例 1、
-/// txg 5 那条被抛弃的根——C554 乙罩不到的那一格，不是乙的拒。
-/// 原来这一条接着把 F 抬到 5（C 那个 txg）、钉「只有 I-3.1 红、归到『已知红』清单第 0 条（收口表第 43 行）」，另有 F 抬到 6、7 两个对照全绿；
-/// 抬 F 之前已经 I-7.4 红，抬 F、归类与两个对照那一段都立不住，删掉（`random_histories.rs` 那一条同形、同样改钉）。
-/// 清单第 0 条的归类由 `crates/singlefs-harness/src/history.rs` 的库单测钉着。
+/// 双故障形（C583（双故障形：每盘一槽系统配置坏加最新根一时读不出））：崩溃恢复抛弃 C（实例 1、txg 5）之后实例 2 覆盖写四次。
+/// 那次挂载时 C 的根槽读不出、系统配置没见证 C（C554 乙罩不到的那一格），影子账看不见 C；可写挂载时的预隔离（D23（journal 的角色与格式）
+/// 已定项 14「可写挂载时的预隔离」）把水位之上没施加的 C 的记录点名的落点与 C 的账里的落点隔离到这个进程写掉 txg 5 那一槽为止，
+/// 四次覆盖写发不出 C 的单元，抬 F 之前池级 checker 一条都不红（`random_histories.rs` 那一条同形、同样改钉）。
+/// 判别力：挂载时不隔离那一份（变异「删除线第38批 可写挂载时的预隔离不隔离水位之上没施加的记录点名的落点（reuse_after_raising_the_floor 双故障形）」）
+/// 时 C 引用的单元又被发出去，I-7.4（近 K 代块未被复用）红、点名实例 1、txg 5 那条被抛弃的根，这一条红。
 #[test]
 #[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
-fn four_overwrites_after_crash_recovery_abandoned_the_third_version_are_red_only_on_i_7_4_before_any_floor_raise_in_the_known_double_fault_form(
+fn four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units(
 ) {
     let pool =
         pool_after_a_recovery_abandoned_the_third_version_and_four_overwrites("step-five-gap-5");
@@ -1174,19 +1174,9 @@ fn four_overwrites_after_crash_recovery_abandoned_the_third_version_are_red_only
             | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
-    assert_eq!(
-        violations
-            .iter()
-            .map(|(invariant, _)| *invariant)
-            .collect::<Vec<_>>(),
-        vec!["I-7.4"],
-        "抬 F 之前恰好 I-7.4 红：{violations:?}"
-    );
     assert!(
-        violations[0]
-            .1
-            .starts_with("被抛弃的根（实例 1、txg 5）引用的单元已被重新分配或抹头"),
-        "I-7.4 的说明文字点名实例 1、txg 5 那条被抛弃的根：{violations:?}"
+        violations.is_empty(),
+        "抬 F 之前池级 checker 一条都不红（C 的单元被预隔离护着）：{violations:?}"
     );
 }
 
diff --git a/crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs b/crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs
new file mode 100644
index 0000000..6f846a4
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs
@@ -0,0 +1,319 @@
+//! 可写挂载时的预隔离（C）再收严两处（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1、H3 两行与第三节第 1 条，
+//! 用户 2026-10-08 定「四形都修」）：
+//! - `C_NEWER_ROOTS`：C 并上根环里比所选那一版新、按实例表有效、读得出的根自己的账（照影子账同一份读法）。F2 退回之后被退掉的那条根 R
+//!   就是这样：影子账只管被抛弃的根、不管它，它的记录又全读不出时，C 那两份（记录点名的、记录拼根读的账）都是空的。
+//! - `C_MAPPING`（收窄）：C 读一次没施加的发布（记录拼的根，或上面那条较新的根）的账读不出时，只在那次发布读得出的记录不齐（序号缺一条、
+//!   缺末条）时照被抛弃根那一份从中央映射现算账里的落点；仍算不出就在取号之前拒可写（拒因照被抛弃根那一类）。
+//!   记录齐的不走这一支（照被抛弃根那一份不收窄会在 `ur` 那几形上误拒，攻方量到入库 7 条）。较新的根那次发布一条记录都读不出时
+//!   条款没写：映射现算得出就并进来，现算不成不拒（入库 `urJ` 那一形就是这样，拒了它就再也做不成可写挂载）。
+//!
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`，攻方 `research/prompts/m3-batch35-36-code-r1-opus-output.md` 第一、三节那几形）：384 槽小盘、
+//! 第一个文件之后覆盖写 5 次，再录一次发布 R（覆盖写一次、一条记录；或顺序写三个数据单元、三条记录），崩在 R 的根槽落盘之后、系统配置轮换之前；
+//! 取新实例的那次可写挂载 M1 里几段读回全零。每格核：M1 做成时写行那次发布的根落盘之前 R 的单元一份不少，M1 之后覆盖写、建 inode、
+//! 顺序写各一次，R 的根还在根环里时 R 的单元一份不少；M1 被拒时拒在任何写之前。
+mod common_unwitnessed_publish;
+
+use common_unwitnessed_publish::{
+    mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions,
+    record_one_publish_after_overwrites, PublishRecordedAsR, RangesReadingBackAllZero,
+    RecordedPublish,
+};
+use singlefs_core::mount::{MountError, RollbackTarget, StillUnreadableAfterOneReread};
+use singlefs_harness::memory_pool::RetainedWrite;
+
+/// 覆盖写几次之后录 R：攻方第一、三节都取 k = 5。
+const OVERWRITES_BEFORE_R: u64 = 5;
+
+/// R 的每一条记录（两块盘各一份）。
+fn every_record_of_r(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
+    (1..=recorded.record_count())
+        .flat_map(|ordinal| recorded.record_writes_of_the_record(ordinal))
+        .collect()
+}
+
+/// R 自己写的那个单元（`pointer` 指着的），两块盘各一份。
+fn unit_writes_of_r_at<'recorded>(
+    recorded: &'recorded RecordedPublish,
+    pointer: &singlefs_core::pointer::NodePointer,
+    what: &str,
+) -> Vec<&'recorded RetainedWrite> {
+    let writes = recorded.unit_writes_at(pointer);
+    assert_eq!(writes.len(), 2, "R 自己写了{what}，两块盘各一份");
+    writes
+}
+
+/// `C_NEWER_ROOTS`（攻方 H1 最小形）：R 是一条记录的覆盖写，M1 里 R 的中央映射树根与 R 那条记录读回全零，R 的根槽读得出。
+/// R 被择、走读失败，系统配置没见证 R（末条读不出，计数器 = c_见证 的那条是 P 的），退回 P；R 的账（树表、分配记录树）读得出。
+/// 判别力：C 不并较新可读根的账（变异「删除线第38批 C_NEWER_ROOTS C 不并较新可读根的账（覆盖写 R 的映射根与记录读回全零）」）时
+/// C 是空的，写行那次发布的根落盘之前 M1 就写掉 R 的单元（攻方量到 14 份），这一条红。
+#[test]
+fn a_newer_readable_root_whose_mapping_root_and_only_record_read_back_zero_keeps_its_units_through_the_mount_and_three_user_actions(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let root_of_r = recorded.root();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&unit_writes_of_r_at(
+            &recorded,
+            &root_of_r.mapping_root,
+            "中央映射树根",
+        ))
+        .with_the_writes(&every_record_of_r(&recorded));
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let effective =
+        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
+            &recorded,
+            &crash_image,
+            &ranges,
+            "覆盖写 R、映射根与记录读回全零",
+        )
+        .expect("R 的映射根与记录读回全零：可写挂载退到 P、做得成");
+    assert_eq!(
+        effective, recorded.previous_key,
+        "R 走读失败、系统配置没见证它，所选那一版退到 P"
+    );
+}
+
+/// `C_NEWER_ROOTS`（攻方 H1 的 sw3 那一形）：R 是三条记录的顺序写，M1 里 R 的中央映射树根与三条记录都读回全零，R 的根槽读得出、账读得出。
+/// 判别力：同上那一条变异的 sw3 那一行（「……（顺序写 R 的映射根与三条记录读回全零）」）时这一条红。
+#[test]
+fn a_newer_readable_root_of_three_records_whose_mapping_root_and_every_record_read_back_zero_keeps_its_units_through_the_mount_and_three_user_actions(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
+    let root_of_r = recorded.root();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&unit_writes_of_r_at(
+            &recorded,
+            &root_of_r.mapping_root,
+            "中央映射树根",
+        ))
+        .with_the_writes(&every_record_of_r(&recorded));
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let effective =
+        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
+            &recorded,
+            &crash_image,
+            &ranges,
+            "顺序写 R、映射根与三条记录读回全零",
+        )
+        .expect("R 的映射根与三条记录读回全零：可写挂载退到 P、做得成");
+    assert_eq!(effective, recorded.previous_key, "所选那一版退到 P");
+}
+
+/// `C_NEWER_ROOTS` 连同 `C_MAPPING`（攻方 H1 里账读不出的那几格）：R 是一条记录的覆盖写，M1 里 R 的树表与那条记录读回全零，R 的根槽读得出。
+/// 退回 P 之后 R 是较新的可读根，它的账（经树表读）读不出，那次发布一条记录都读不出 ⇒ 映射现算得出就并进 C（这一格条款没写，
+/// 取法见 `mount::placements_in_the_accounts_of_newer_readable_roots` 那一支的注释）。
+/// 判别力：一条记录都读不出时不从映射现算（变异「删除线第38批 C_MAPPING 较新可读根一条记录都读不出时不从映射现算」）时 C 是空的、这一条红。
+#[test]
+fn a_newer_readable_root_whose_tree_table_and_only_record_read_back_zero_keeps_its_units_recomputed_from_the_mapping(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let root_of_r = recorded.root();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&unit_writes_of_r_at(
+            &recorded,
+            &root_of_r.tree_table,
+            "树表",
+        ))
+        .with_the_writes(&every_record_of_r(&recorded));
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let effective =
+        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
+            &recorded,
+            &crash_image,
+            &ranges,
+            "覆盖写 R、树表与记录读回全零",
+        )
+        .expect("R 的账读不出、映射现算得出：可写挂载退到 P、做得成");
+    assert_eq!(effective, recorded.previous_key, "所选那一版退到 P");
+}
+
+/// `C_MAPPING`（攻方 H3 根藏的那一半）：R 是三条记录的顺序写，M1 里 R 的根槽、末条与 R 的树表读回全零。所选那一版是 P，读得出的第 1、2 条
+/// 缺末条、点名的数据单元验得过（F1 乙不拒）；记录拼根读 R 的账读不出 ⇒ 记录不齐，从中央映射现算 R 引用的落点。
+/// 判别力：没施加的发布的账读不出时不从映射现算（变异「删除线第38批 C_MAPPING 没施加的发布的账读不出时不从映射现算」）时末条单独点名的单元
+/// （第三个数据单元与全部元数据单元）没人护，这一条红。
+#[test]
+fn a_publish_whose_root_slot_last_record_and_tree_table_read_back_zero_keeps_its_units_recomputed_from_the_mapping(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    let root_of_r = recorded.root();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.record_writes_of_the_record(3))
+        .with_the_writes(&unit_writes_of_r_at(
+            &recorded,
+            &root_of_r.tree_table,
+            "树表",
+        ));
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let effective =
+        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
+            &recorded,
+            &crash_image,
+            &ranges,
+            "顺序写 R、根槽、末条与树表读回全零",
+        )
+        .expect("R 的账读不出、映射现算得出：可写挂载做得成");
+    assert_eq!(
+        effective, recorded.previous_key,
+        "R 的根槽读不出，所选那一版是 P"
+    );
+}
+
+/// `C_MAPPING`（攻方 H3 根读得出的那一半）：同上，只是 R 的根槽读得出。R 被择、走读失败、退回 P（末条读不出，计数器 = c_见证 的那条是 P 的），
+/// F1 乙按 P 再判一次：读得出的第 1、2 条点名的数据单元验得过、不拒；R 既是没施加的发布、又是较新的可读根，两处都按映射现算。
+#[test]
+fn a_newer_readable_root_whose_last_record_and_tree_table_read_back_zero_keeps_its_units_recomputed_from_the_mapping(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    let root_of_r = recorded.root();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&recorded.record_writes_of_the_record(3))
+        .with_the_writes(&unit_writes_of_r_at(
+            &recorded,
+            &root_of_r.tree_table,
+            "树表",
+        ));
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let effective =
+        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
+            &recorded,
+            &crash_image,
+            &ranges,
+            "顺序写 R、末条与树表读回全零、根槽读得出",
+        )
+        .expect("R 的账读不出、映射现算得出：可写挂载退到 P、做得成");
+    assert_eq!(effective, recorded.previous_key, "所选那一版退到 P");
+}
+
+/// `C_MAPPING` 仍算不出就拒：同「根藏的那一半」，再加 R 的中央映射树根读回全零——账读不出、记录不齐、映射也现算不成，重读一次仍是这样，
+/// 现行账（P 的）里也没有 C694 的 B 那道凭据（R 比 P 新），在取号之前拒可写，拒因照被抛弃根那一类、点名 R。
+/// 判别力：现算不成也不拒（变异「删除线第38批 C_MAPPING 现算不成也不拒」）时可写挂载做成、这一条红。
+#[test]
+fn a_publish_whose_account_and_mapping_read_back_zero_while_its_last_record_is_missing_refuses_the_writable_mount_before_any_write(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    let root_of_r = recorded.root();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.record_writes_of_the_record(3))
+        .with_the_writes(&unit_writes_of_r_at(
+            &recorded,
+            &root_of_r.tree_table,
+            "树表",
+        ))
+        .with_the_writes(&unit_writes_of_r_at(
+            &recorded,
+            &root_of_r.mapping_root,
+            "中央映射树根",
+        ));
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let refusal =
+        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
+            &recorded,
+            &crash_image,
+            &ranges,
+            "顺序写 R、根槽、末条、树表与映射根读回全零",
+        )
+        .expect_err("账与映射都读不出、记录不齐：可写挂载在取号之前拒");
+    assert!(
+        matches!(
+            &refusal,
+            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
+                if matches!(
+                    **still_unreadable,
+                    StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
+                        abandoned_root: RollbackTarget { instance, checkpoint_txg }
+                    } if (instance, checkpoint_txg) == recorded.key
+                )
+        ),
+        "拒因照被抛弃根那一类、点名 R：{refusal:?}"
+    );
+}
+
+/// 攻方 H1 那一族的覆盖写那一形逐格（`C_NEWER_ROOTS` 与 `C_MAPPING` 合用）：R 是一条记录的覆盖写，M1 里 R 那条记录与 R 写出的某一个单元
+/// （逐个取，两块盘的两份）读回全零，R 的根槽读得出。每一格要么做成、R 的单元一份不少，要么在任何写之前拒。
+#[test]
+fn every_unit_of_a_newer_readable_root_of_one_record_reading_back_zero_with_its_record_keeps_its_units(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    assert_every_unit_reading_back_zero_with_every_record_keeps_the_units_of_r(&recorded);
+}
+
+/// 同上，R 是三条记录的顺序写，三条记录都读回全零（攻方第一节 sw3 `rec_hidden=111` 那一行）。
+#[test]
+fn every_unit_of_a_newer_readable_root_of_three_records_reading_back_zero_with_every_record_keeps_its_units(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    assert_every_unit_reading_back_zero_with_every_record_keeps_the_units_of_r(&recorded);
+}
+
+/// R 写出的每一个单元逐个连同 R 的全部记录读回全零（根槽读得出），各挂一次：做成的核 R 的单元一份不少，拒的只许是 C 的映射那一支拒
+/// （点名 R）。打印每一格的结局与合计。
+fn assert_every_unit_reading_back_zero_with_every_record_keeps_the_units_of_r(
+    recorded: &RecordedPublish,
+) {
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let mut outcomes: Vec<String> = Vec::new();
+    for (unit_index, unit_copies) in recorded.unit_writes_grouped_by_unit().iter().enumerate() {
+        let cell = format!("R 的第 {unit_index} 个单元与全部记录读回全零");
+        let ranges = RangesReadingBackAllZero::default()
+            .with_the_writes(unit_copies)
+            .with_the_writes(&every_record_of_r(recorded));
+        match mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
+            recorded,
+            &crash_image,
+            &ranges,
+            &cell,
+        ) {
+            Ok(effective) => outcomes.push(format!("{cell}：做成，落到 {effective:?}")),
+            Err(refusal) => {
+                assert!(
+                    matches!(
+                        &refusal,
+                        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
+                            if matches!(
+                                **still_unreadable,
+                                StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
+                                    abandoned_root: RollbackTarget { instance, checkpoint_txg }
+                                } if (instance, checkpoint_txg) == recorded.key
+                            )
+                    ),
+                    "{cell}：拒只许是 C 的映射那一支拒、点名 R：{refusal:?}"
+                );
+                outcomes.push(format!("{cell}：拒（映射现算不成）"));
+            }
+        }
+    }
+    println!("{}", outcomes.join("\n"));
+    println!(
+        "合计 {} 格：做成 {}，拒 {}",
+        outcomes.len(),
+        outcomes
+            .iter()
+            .filter(|outcome| outcome.contains("做成"))
+            .count(),
+        outcomes
+            .iter()
+            .filter(|outcome| outcome.contains("：拒"))
+            .count()
+    );
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs b/crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs
index 9a22798..e9b7cf4 100644
--- a/crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs
+++ b/crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs
@@ -16,7 +16,9 @@ use common_unwitnessed_publish::{
     RangesReadingBackAllZero, OVERWRITE_BYTES,
 };
 use singlefs_core::address::DeviceIdentity;
+use singlefs_core::journal::JournalRecord;
 use singlefs_core::mount::{mount_writable, unmount, ShadowLedger};
+use singlefs_core::unit::unit_filesystem_identifier;
 use singlefs_harness::memory_pool::MemoryPool;
 
 /// 覆盖写几次之后录 R：攻方那一形量过 k = 1 与 4（两者逐格同数），取 1。
@@ -127,3 +129,64 @@ fn a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_thr
         assert_the_units_of_the_publish_stay_unwritten_through_an_overwrite_and_the_unmount_when_record_reads_back_zero(1);
     println!("M1 隔离槽数（rF）：{isolated_slots:?}");
 }
+
+/// C 选法 `all`（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条：不只认带末条标志的记录）在 `C_ACCOUNT` 之下分得开的取样点
+/// （`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第三类：原来那一格的末条读不出时 `C_ACCOUNT` 读得出 R 的账、把前两条点名的落点
+/// 一并隔离，选法退成只认末条也照样护住）：R 的三条记录都读得出，M1 里 R 的根槽与 R 的树表读回全零。末条点名的树表两份都验不过，R 整次不施加，
+/// 所选那一版是 P；三条记录齐（序号 1..3、带末条），F1 乙不判，C 的账读不出也不从映射现算；记录拼根读 R 的账经树表、读不出，`C_ACCOUNT` 什么都不加
+/// ——前两条点名的落点只有 C 记录那一份护着。
+/// 判别力：选法退成只认带末条标志的记录（变异「第33批T C694 的 C 选法退成只认带末条标志的记录（last）」）时前两条点名的数据单元 M1 之后空闲、这一条红。
+#[test]
+fn the_units_named_by_the_first_two_records_of_a_publish_whose_root_slot_and_tree_table_read_back_zero_are_isolated(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
+    let tree_table_writes = recorded.unit_writes_at(&recorded.root().tree_table);
+    assert_eq!(tree_table_writes.len(), 2, "R 自己写了树表，两块盘各一份");
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&tree_table_writes);
+    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("R 的三条记录齐、树表读不出：F1 乙不判，可写挂载做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 的根槽读回全零、末条点名的树表验不过：R 整次不施加，所选那一版是 P"
+    );
+    let filesystem_identifier =
+        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier);
+    for ordinal in [1, 2] {
+        let record = JournalRecord::parse(
+            recorded.record_writes_of_the_record(ordinal)[0]
+                .bytes()
+                .expect("记录写带字节"),
+            filesystem_identifier,
+        )
+        .expect("R 的记录解得开");
+        assert!(!record.named.is_empty(), "第 {ordinal} 条记录点名了单元");
+        for named in &record.named {
+            for location in &named.locations {
+                let device_map = mounted
+                    .allocator
+                    .devices
+                    .iter()
+                    .find(|device_map| device_map.device == location.device)
+                    .expect("点名项的盘在池里");
+                assert!(
+                    !device_map.is_free(location.slot),
+                    "R 第 {ordinal} 条记录点名的落点（盘 {:?} 槽 {:?}）M1 之后不空闲（C 隔离了它）",
+                    location.device,
+                    location.slot
+                );
+            }
+        }
+    }
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version.rs b/crates/singlefs-harness/tests/writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version.rs
new file mode 100644
index 0000000..190bf96
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version.rs
@@ -0,0 +1,215 @@
+//! F1 乙按 F2 退到的那一版再判一次（`F1_AFTER_F2`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H2 那一行与第三节第 1 条，
+//! 用户 2026-10-08 定「四形都修」）：可写挂载读阶段判 F1 时的水位是退之前的所选那一版；所选根读得出、走读失败、F2 退回见证到的那一版之后，
+//! 被退掉的那次发布到了水位之上，按退到的那一版再判一次 F1（读得出的记录序号不连续或缺末条、点名的单元有一个两份都验不过 ⇒ 重读一次仍这样就拒）。
+//!
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`，攻方 `research/prompts/m3-batch35-36-code-r1-opus-output.md` 第二节那一形）：384 槽小盘、
+//! 第一个文件之后覆盖写 5 次，再录一次三条记录的顺序写 R（每个数据单元一条记录，末条另点名全部元数据单元），崩在 R 的根槽落盘之后、
+//! 系统配置轮换之前；取新实例的那次可写挂载 M1 里 R 的根槽读得出，R 的某几条记录与 R 写出的某一个单元读回全零。
+mod common_unwitnessed_publish;
+
+use common_unwitnessed_publish::{
+    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
+    mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions,
+    record_one_publish_after_overwrites, PublishRecordedAsR, RangesReadingBackAllZero,
+    RecordedPublish,
+};
+use singlefs_core::journal::JournalRecord;
+use singlefs_core::mount::{
+    mount_writable, MountError, RollbackTarget, StillUnreadableAfterOneReread,
+};
+use singlefs_core::unit::unit_filesystem_identifier;
+use singlefs_harness::memory_pool::RetainedWrite;
+use singlefs_harness::segments::StepKind;
+
+const OVERWRITES_BEFORE_R: u64 = 5;
+
+/// 这一拒是 F1 乙、点名 R 那次发布。
+fn is_the_refusal_for_missing_records_of_r(
+    refusal: &MountError,
+    recorded: &RecordedPublish,
+) -> bool {
+    matches!(
+        refusal,
+        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
+            if matches!(
+                **still_unreadable,
+                StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
+                    publish: RollbackTarget { instance, checkpoint_txg }
+                } if (instance, checkpoint_txg) == recorded.key
+            )
+    )
+}
+
+/// 攻方 H2 那一格：M1 里 R 的第 1 条记录与 R 的树表（末条点名的元数据单元，R 的账在它上面）读回全零，R 的根槽读得出。
+/// R 被择、走读失败，系统配置没见证 R（末条读得出、计数器大于 c_见证），退回 P；按 P 当水位，R 读得出的是第 2、3 条（序号不从 1 起），
+/// 第 3 条点名的树表两份都验不过 ⇒ 在取号之前拒可写，盘上逐字节不变。
+/// 判别力：退回之后不再判 F1（变异「删除线第38批 F1_AFTER_F2 F2 退回之后不按退到的那一版再判 F1」）时 M1 做成、这一条红。
+#[test]
+fn a_root_falling_back_to_the_witnessed_version_with_its_first_record_and_tree_table_reading_back_zero_refuses_the_writable_mount_before_any_write(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
+    let tree_table_writes = recorded.unit_writes_at(&recorded.root().tree_table);
+    assert_eq!(tree_table_writes.len(), 2, "R 自己写了树表，两块盘各一份");
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&recorded.record_writes_of_the_record(1))
+        .with_the_writes(&tree_table_writes);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let refusal = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect_err("退回 P 之后 R 缺第 1 条、末条点名的树表两份都验不过：拒可写");
+    assert!(
+        is_the_refusal_for_missing_records_of_r(&refusal, &recorded),
+        "拒因是 F1 乙、点名 R：{refusal:?}"
+    );
+    assert!(
+        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes) == crash_image,
+        "拒在任何写之前：两块盘逐字节不变"
+    );
+}
+
+/// 攻方 H2 那一族逐格：R 的记录子集 {1}、{2}、{1,2} 读回全零 × R 写出的每一个单元逐个读回全零（两份），R 的根槽读得出。
+/// 每一格三种结局之一，各有各的前提：退回 P 之后被 F1 乙拒（点名 R、拒在任何写之前）——读阶段之后那个单元被读得出的某条记录点名；
+/// 退回 P 又做成——那个单元不被读得出的任何一条记录点名（F1 乙只看读得出的记录点名的单元）；不退、挂在 R 上（R 走得通）。
+/// 做成的两种都核 R 的单元一份不少。打印每一格的结局与合计（这一版量到 39 格：被 F1 乙拒 30、退回 P 又做成 0、挂在 R 上 9）。
+/// 判别力：退回之后不再判 F1（变异「删除线第38批 F1_AFTER_F2 F2 退回之后不按退到的那一版再判 F1（缺记录、逐个单元读回全零那一族）」）时
+/// 那 30 格退回 P 又做成、点名单元被读得出的记录点名，这一条红。
+#[test]
+fn every_unit_of_a_root_falling_back_with_a_gap_in_its_records_refuses_the_writable_mount_exactly_when_a_readable_record_names_it(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let mut outcomes: Vec<String> = Vec::new();
+    for hidden_records in [vec![1], vec![2], vec![1, 2]] {
+        for (unit_index, unit_copies) in recorded.unit_writes_grouped_by_unit().iter().enumerate() {
+            let cell = format!("记录 {hidden_records:?} 与 R 的第 {unit_index} 个单元读回全零");
+            let record_writes: Vec<&RetainedWrite> = hidden_records
+                .iter()
+                .flat_map(|ordinal| recorded.record_writes_of_the_record(*ordinal))
+                .collect();
+            let ranges = RangesReadingBackAllZero::default()
+                .with_the_writes(&record_writes)
+                .with_the_writes(unit_copies);
+            let named_by_a_readable_record = unit_is_named_by_a_record_not_reading_back_zero(
+                &recorded,
+                unit_copies,
+                &hidden_records,
+            );
+            match mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
+                &recorded,
+                &crash_image,
+                &ranges,
+                &cell,
+            ) {
+                Ok(effective) if effective == recorded.key => {
+                    outcomes.push(format!("{cell}：没退，挂在 R 上"));
+                }
+                Ok(effective) => {
+                    assert_eq!(effective, recorded.previous_key, "{cell}：退的话退到 P");
+                    assert!(
+                        !named_by_a_readable_record,
+                        "{cell}：退回 P 又做成的，那个单元不被读得出的记录点名（被点名就该被 F1 乙拒）"
+                    );
+                    outcomes.push(format!(
+                        "{cell}：退回 P、做成（单元只被读回全零的记录点名）"
+                    ));
+                }
+                Err(refusal) => {
+                    assert!(
+                        is_the_refusal_for_missing_records_of_r(&refusal, &recorded),
+                        "{cell}：拒因是 F1 乙、点名 R：{refusal:?}"
+                    );
+                    assert!(
+                        named_by_a_readable_record,
+                        "{cell}：被 F1 乙拒的，那个单元被读得出的某条记录点名"
+                    );
+                    outcomes.push(format!("{cell}：退回 P 之后被 F1 乙拒"));
+                }
+            }
+        }
+    }
+    println!("{}", outcomes.join("\n"));
+    println!(
+        "合计 {} 格：被 F1 乙拒 {}，退回 P、做成 {}，挂在 R 上 {}",
+        outcomes.len(),
+        outcomes
+            .iter()
+            .filter(|outcome| outcome.contains("被 F1 乙拒"))
+            .count(),
+        outcomes
+            .iter()
+            .filter(|outcome| outcome.contains("退回 P、做成"))
+            .count(),
+        outcomes
+            .iter()
+            .filter(|outcome| outcome.contains("挂在 R 上"))
+            .count()
+    );
+}
+
+/// R 写出的这个单元（`unit_copies`，两块盘的两份）被 R 的某条没读回全零的记录（`hidden_records` 以外的）点名。
+fn unit_is_named_by_a_record_not_reading_back_zero(
+    recorded: &RecordedPublish,
+    unit_copies: &[&RetainedWrite],
+    hidden_records: &[usize],
+) -> bool {
+    let filesystem_identifier =
+        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier);
+    (1..=recorded.record_count())
+        .filter(|ordinal| !hidden_records.contains(ordinal))
+        .any(|ordinal| {
+            let record = JournalRecord::parse(
+                recorded.record_writes_of_the_record(ordinal)[0]
+                    .bytes()
+                    .expect("记录写带字节"),
+                filesystem_identifier,
+            )
+            .expect("R 的记录解得开");
+            record.named.iter().any(|named| {
+                named.locations.iter().any(|location| {
+                    unit_copies.iter().any(|copy| {
+                        copy.device == location.device
+                            && copy.offset == location.slot.to_device_offset()
+                    })
+                })
+            })
+        })
+}
+
+/// 正常崩溃照不拒：崩在 R 写记录的中途（第 1 条两份都落了、第 2 条还没写），一处读故障都没有。R 的根槽没落，所选那一版是 P，
+/// 读得出的第 1 条缺末条、点名的数据单元验得过（单元在记录之前落盘）：F1 乙不拒，可写挂载做成；R 走不到 F2，`F1_AFTER_F2` 不判。
+#[test]
+fn a_power_loss_after_the_first_record_of_a_three_record_publish_still_mounts_writable() {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    let first_record_offset = recorded.record_writes_of_the_record(1)[0].offset;
+    let last_write_of_the_first_record = recorded
+        .writes
+        .iter()
+        .rposition(|write| {
+            write.kind == StepKind::JournalRecord && write.offset == first_record_offset
+        })
+        .expect("R 写了第 1 条记录");
+    let mut crash_image = recorded.base.clone();
+    crash_image.apply_writes(&recorded.writes[..=last_write_of_the_first_record]);
+    let mut devices =
+        devices_reading_back_all_zero_on(&crash_image, &RangesReadingBackAllZero::default());
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("崩在写记录中途、没有读故障：可写挂载做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 的根槽没落：所选那一版是 P"
+    );
+}
````

## 删除线第 38 批 Y：mutations-append.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch38/y/patch/mutations-append.tsv）

````text
删除线第38批 C_NEWER_ROOTS C 不并较新可读根的账（覆盖写 R 的映射根与记录读回全零）	crates/singlefs-core/src/mount.rs	                    .extend(placements_in_the_newer_roots_account);	                    .extend(placements_in_the_newer_roots_account.into_iter().take(0));	-p singlefs-harness --test writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing -- a_newer_readable_root_whose_mapping_root_and_only_record_read_back_zero_keeps_its_units_through_the_mount_and_three_user_actions	a_newer_readable_root_whose_mapping_root_and_only_record_read_back_zero_keeps_its_units_through_the_mount_and_three_user_actions
删除线第38批 C_NEWER_ROOTS C 不并较新可读根的账（顺序写 R 的映射根与三条记录读回全零）	crates/singlefs-core/src/mount.rs	                    .extend(placements_in_the_newer_roots_account);	                    .extend(placements_in_the_newer_roots_account.into_iter().take(0));	-p singlefs-harness --test writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing -- a_newer_readable_root_of_three_records_whose_mapping_root_and_every_record_read_back_zero_keeps_its_units_through_the_mount_and_three_user_actions	a_newer_readable_root_of_three_records_whose_mapping_root_and_every_record_read_back_zero_keeps_its_units_through_the_mount_and_three_user_actions
删除线第38批 C_NEWER_ROOTS C 不并较新可读根的账（覆盖写 R 逐个单元与记录读回全零那一族）	crates/singlefs-core/src/mount.rs	                    .extend(placements_in_the_newer_roots_account);	                    .extend(placements_in_the_newer_roots_account.into_iter().take(0));	-p singlefs-harness --test writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing -- every_unit_of_a_newer_readable_root_of_one_record_reading_back_zero_with_its_record_keeps_its_units	every_unit_of_a_newer_readable_root_of_one_record_reading_back_zero_with_its_record_keeps_its_units
删除线第38批 C_NEWER_ROOTS C 不并较新可读根的账（顺序写 R 逐个单元与三条记录读回全零那一族）	crates/singlefs-core/src/mount.rs	                    .extend(placements_in_the_newer_roots_account);	                    .extend(placements_in_the_newer_roots_account.into_iter().take(0));	-p singlefs-harness --test writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing -- every_unit_of_a_newer_readable_root_of_three_records_reading_back_zero_with_every_record_keeps_its_units	every_unit_of_a_newer_readable_root_of_three_records_reading_back_zero_with_every_record_keeps_its_units
删除线第38批 C_MAPPING 没施加的发布的账读不出时不从映射现算（根槽、末条与树表读回全零）	crates/singlefs-core/src/mount.rs	                None if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(\n                    &records_of_the_publish,\n                ) =>	                None if false && readable_records_of_the_publish_leave_a_gap_or_miss_the_last(\n                    &records_of_the_publish,\n                ) =>	-p singlefs-harness --test writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing -- a_publish_whose_root_slot_last_record_and_tree_table_read_back_zero_keeps_its_units_recomputed_from_the_mapping	a_publish_whose_root_slot_last_record_and_tree_table_read_back_zero_keeps_its_units_recomputed_from_the_mapping
删除线第38批 C_MAPPING 较新可读根一条记录都读不出时不从映射现算（树表与记录读回全零）	crates/singlefs-core/src/mount.rs	                            Some(computed) => computed,\n                            None => continue,	                            Some(_computed) => continue,\n                            None => continue,	-p singlefs-harness --test writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing -- a_newer_readable_root_whose_tree_table_and_only_record_read_back_zero_keeps_its_units_recomputed_from_the_mapping	a_newer_readable_root_whose_tree_table_and_only_record_read_back_zero_keeps_its_units_recomputed_from_the_mapping
删除线第38批 C_MAPPING 现算不成也不拒（账与映射都读不出、缺末条）	crates/singlefs-core/src/mount.rs	    placements_of_an_abandoned_root_whose_account_is_unreadable(\n        devices,\n        unit_area_start,\n        root_of_the_publish,\n        current_records,\n        before_the_one_reread,\n    )\n}	    placements_of_an_abandoned_root_whose_account_is_unreadable(\n        devices,\n        unit_area_start,\n        root_of_the_publish,\n        current_records,\n        before_the_one_reread,\n    )\n    .or(Ok(Vec::new()))\n}	-p singlefs-harness --test writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing -- a_publish_whose_account_and_mapping_read_back_zero_while_its_last_record_is_missing_refuses_the_writable_mount_before_any_write	a_publish_whose_account_and_mapping_read_back_zero_while_its_last_record_is_missing_refuses_the_writable_mount_before_any_write
删除线第38批 C_MAPPING 那一支什么都不交（根读得出、末条与树表读回全零）	crates/singlefs-core/src/mount.rs	    placements_of_an_abandoned_root_whose_account_is_unreadable(\n        devices,\n        unit_area_start,\n        root_of_the_publish,\n        current_records,\n        before_the_one_reread,\n    )\n}	    let _ = (devices, unit_area_start, root_of_the_publish, current_records, before_the_one_reread);\n    Ok(Vec::new())\n}	-p singlefs-harness --test writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing -- a_newer_readable_root_whose_last_record_and_tree_table_read_back_zero_keeps_its_units_recomputed_from_the_mapping	a_newer_readable_root_whose_last_record_and_tree_table_read_back_zero_keeps_its_units_recomputed_from_the_mapping
删除线第38批 C_MAPPING 不收窄（记录齐的没施加的发布账读不出也现算、仍不成就拒，ur 那一形被误拒）	crates/singlefs-core/src/mount.rs	                None if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(\n                    &records_of_the_publish,\n                ) =>	                None if true || readable_records_of_the_publish_leave_a_gap_or_miss_the_last(\n                    &records_of_the_publish,\n                ) =>	-p singlefs-harness --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten -- a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance_still_reads_back_the_root_whose_units_read_back_zero_at_that_mount	a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance_still_reads_back_the_root_whose_units_read_back_zero_at_that_mount
删除线第38批 C_MAPPING 较新可读根不收窄（记录齐也现算、仍不成就拒，设计里那一退被误拒）	crates/singlefs-core/src/mount.rs	                    } else if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(\n                        &readable_records_of_its_publish,\n                    ) {	                    } else if true || readable_records_of_the_publish_leave_a_gap_or_miss_the_last(\n                        &readable_records_of_its_publish,\n                    ) {	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_root_whose_units_read_back_zero_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts	a_root_whose_units_read_back_zero_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts
删除线第38批 C_MAPPING 较新可读根一条记录都读不出、映射现算不成也拒（urJ 那一形再也做不成可写挂载）	crates/singlefs-core/src/mount.rs	                            Some(computed) => computed,\n                            None => continue,	                            Some(computed) => computed,\n                            None => return Err(MountError::NewerStateStillUnreadableAfterOneReread(Box::new(StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { abandoned_root: RollbackTarget { instance: root.instance, checkpoint_txg: root.checkpoint_txg } }))),	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts	a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts
删除线第38批 F1_AFTER_F2 F2 退回之后不按退到的那一版再判 F1（缺第 1 条、树表读回全零）	crates/singlefs-core/src/mount.rs	                            &mut before_the_one_reread,\n                        )?;\n                        (	                            &mut before_the_one_reread,\n                        ).ok();\n                        (	-p singlefs-harness --test writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version -- a_root_falling_back_to_the_witnessed_version_with_its_first_record_and_tree_table_reading_back_zero_refuses_the_writable_mount_before_any_write	a_root_falling_back_to_the_witnessed_version_with_its_first_record_and_tree_table_reading_back_zero_refuses_the_writable_mount_before_any_write
删除线第38批 F1_AFTER_F2 F2 退回之后不按退到的那一版再判 F1（缺记录、逐个单元读回全零那一族）	crates/singlefs-core/src/mount.rs	                            &mut before_the_one_reread,\n                        )?;\n                        (	                            &mut before_the_one_reread,\n                        ).ok();\n                        (	-p singlefs-harness --test writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version -- every_unit_of_a_root_falling_back_with_a_gap_in_its_records_refuses_the_writable_mount_exactly_when_a_readable_record_names_it	every_unit_of_a_root_falling_back_with_a_gap_in_its_records_refuses_the_writable_mount_exactly_when_a_readable_record_names_it
删除线第38批 F1 乙不看点名单元验不验得过（崩在三条记录的第 1 条之后、没有读故障也拒）	crates/singlefs-core/src/mount.rs	            readable_records_of_the_publish_leave_a_gap_or_miss_the_last(records_of_the_publish)\n                && some_named_unit_does_not_verify	            readable_records_of_the_publish_leave_a_gap_or_miss_the_last(records_of_the_publish)\n                && (some_named_unit_does_not_verify || true)	-p singlefs-harness --test writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version -- a_power_loss_after_the_first_record_of_a_three_record_publish_still_mounts_writable	a_power_loss_after_the_first_record_of_a_three_record_publish_still_mounts_writable
删除线第38批 F2_NEWEST 只退到见证到的那一版本身（H4 覆盖写 R、R 之前覆盖写 3 次：落到 Pm、跳过已确认的 P）	crates/singlefs-core/src/recovery.rs	        witnessed_publish <= order\n            && order < chosen\n            && kept_by_the_instance_table_of_the_chosen_root	        witnessed_publish == order\n            && order < chosen\n            && kept_by_the_instance_table_of_the_chosen_root	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_root_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_three_overwrites	a_root_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_three_overwrites
删除线第38批 F2_NEWEST 只退到见证到的那一版本身（H4 覆盖写 R、R 之前覆盖写 5 次：落到 Pm、跳过已确认的 P）	crates/singlefs-core/src/recovery.rs	        witnessed_publish <= order\n            && order < chosen\n            && kept_by_the_instance_table_of_the_chosen_root	        witnessed_publish == order\n            && order < chosen\n            && kept_by_the_instance_table_of_the_chosen_root	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_root_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_five_overwrites	a_root_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_five_overwrites
删除线第38批 F2_NEWEST 只退到见证到的那一版本身（H4 顺序写 R、R 之前覆盖写 3 次：落到 Pm、跳过已确认的 P）	crates/singlefs-core/src/recovery.rs	        witnessed_publish <= order\n            && order < chosen\n            && kept_by_the_instance_table_of_the_chosen_root	        witnessed_publish == order\n            && order < chosen\n            && kept_by_the_instance_table_of_the_chosen_root	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_root_of_three_records_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_three_overwrites	a_root_of_three_records_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_three_overwrites
删除线第38批 F2_NEWEST 只退到见证到的那一版本身（H4 顺序写 R、R 之前覆盖写 5 次：落到 Pm、跳过已确认的 P）	crates/singlefs-core/src/recovery.rs	        witnessed_publish <= order\n            && order < chosen\n            && kept_by_the_instance_table_of_the_chosen_root	        witnessed_publish == order\n            && order < chosen\n            && kept_by_the_instance_table_of_the_chosen_root	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_root_of_three_records_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_five_overwrites	a_root_of_three_records_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_five_overwrites
删除线第38批 F2_NEWEST 见证记录读不出时照退（下限放到最旧，退到 P）	crates/singlefs-core/src/recovery.rs	    let witnessed_publish = record_at_the_witnessed_counter?;	    let witnessed_publish = record_at_the_witnessed_counter.unwrap_or((CheckpointTxg(0), InstanceGeneration(0)));	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_row_publish_whose_units_and_the_witnessed_last_record_read_back_zero_is_reported_unreadable_even_with_its_instance_table_readable	a_row_publish_whose_units_and_the_witnessed_last_record_read_back_zero_is_reported_unreadable_even_with_its_instance_table_readable
删除线第38批 F2_NEWEST 退到的根不按所选根的实例表滤（退到被抛弃的 R）	crates/singlefs-core/src/recovery.rs	            Some(table) => !root_is_abandoned_by_the_instance_table(root, table),	            Some(_table) => true,	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_row_publish_whose_units_read_back_zero_falls_back_past_the_root_its_instance_table_abandoned_to_the_witnessed_version	a_row_publish_whose_units_read_back_zero_falls_back_past_the_root_its_instance_table_abandoned_to_the_witnessed_version
删除线第38批 F2_NEWEST 所选根的实例表读不出时照收最新的一条（退到被抛弃的 R）	crates/singlefs-core/src/recovery.rs	            None => order == witnessed_publish,	            None => true,	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_row_publish_whose_units_and_instance_table_read_back_zero_falls_back_only_to_the_exactly_witnessed_version	a_row_publish_whose_units_and_instance_table_read_back_zero_falls_back_only_to_the_exactly_witnessed_version
删除线第38批 可写挂载时的预隔离不隔离水位之上没施加的记录点名的落点（reuse_after_raising_the_floor 双故障形，抬 F 之前 I-7.4 红）	crates/singlefs-core/src/mount.rs	    for placement in placements_named_by_unapplied_journal_records {	    for placement in placements_named_by_unapplied_journal_records.iter().take(0) {	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored --exact four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units	four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units
````

## 删除线第 38 批 Y：mutations-replacements.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch38/y/patch/mutations-replacements.tsv）

````text
第33批T C694 的 C 选法退成只认带末条标志的记录（last）	crates/singlefs-core/src/mount.rs	    for record in records.values().filter(|record| {\n        (record.instance, record.checkpoint_txg)\n            > (selected_version.instance, selected_version.checkpoint_txg)\n    }) {\n        let new_root_segment_pointers = [	    for record in records.values().filter(|record| {\n        (record.instance, record.checkpoint_txg)\n            > (selected_version.instance, selected_version.checkpoint_txg)\n            && record.place_in_publish\n                == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish\n    }) {\n        let new_root_segment_pointers = [	-p singlefs-harness --test writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero -- the_units_named_by_the_first_two_records_of_a_publish_whose_root_slot_and_tree_table_read_back_zero_are_isolated	the_units_named_by_the_first_two_records_of_a_publish_whose_root_slot_and_tree_table_read_back_zero_are_isolated
第33批T F1 乙不认序号缺口（中间一条读不出）	crates/singlefs-core/src/mount.rs	    let ordinals_run_from_one_without_a_gap = ordinals\n        .iter()\n        .zip(1..)\n        .all(|(ordinal, expected)| *ordinal == expected);	    let ordinals_run_from_one_without_a_gap = !ordinals.is_empty();	-p singlefs-harness --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero -- a_publish_whose_middle_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write	a_publish_whose_middle_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write
第33批T F1 乙不认序号缺口（第一条读不出）	crates/singlefs-core/src/mount.rs	    let ordinals_run_from_one_without_a_gap = ordinals\n        .iter()\n        .zip(1..)\n        .all(|(ordinal, expected)| *ordinal == expected);	    let ordinals_run_from_one_without_a_gap = !ordinals.is_empty();	-p singlefs-harness --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero -- a_publish_whose_first_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write	a_publish_whose_first_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write
第33批T F1 乙不认缺末条	crates/singlefs-core/src/mount.rs	    let carries_a_last_record = records_of_the_publish.iter().any(|record| {	    let carries_a_last_record = true || records_of_the_publish.iter().any(|record| {	-p singlefs-harness --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero -- a_publish_whose_last_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write	a_publish_whose_last_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write
第33批T F1 乙不看点名单元验不验得过（正常崩溃在写记录中途也拒）	crates/singlefs-core/src/mount.rs	            readable_records_of_the_publish_leave_a_gap_or_miss_the_last(records_of_the_publish)\n                && some_named_unit_does_not_verify	            readable_records_of_the_publish_leave_a_gap_or_miss_the_last(records_of_the_publish)\n                && (some_named_unit_does_not_verify || true)	-p singlefs-harness --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero -- a_power_loss_while_writing_the_records_of_a_publish_whose_units_verify_still_mounts_writable	a_power_loss_while_writing_the_records_of_a_publish_whose_units_verify_still_mounts_writable
删除线第35批 F2_EXACT F2 退到见证上限之下任一条可读根（A 形：见证到的那一版根槽读回全零时退过它）	crates/singlefs-core/src/recovery.rs	        witnessed_publish <= order\n            && order < chosen\n            && kept_by_the_instance_table_of_the_chosen_root	        order < chosen && kept_by_the_instance_table_of_the_chosen_root	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_root_whose_units_and_the_witnessed_versions_root_slot_read_back_zero_is_reported_unreadable_in_recovery_and_both_mounts	a_root_whose_units_and_the_witnessed_versions_root_slot_read_back_zero_is_reported_unreadable_in_recovery_and_both_mounts
````

## 删除线第 38 批 Y：mutations-delete.txt（/home/fy5090/.local/share/singlefs/work/m3-batch38/y/patch/mutations-delete.txt）

````text
删除线第35批 F2_EXACT 退回 F2_EXACT 之前的上限取法（B 形：见证记录读不出时上限取所选根、退到被抛弃的根）
````

## 删除线第 39 批 Z：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch39/z/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -1338,14 +1338,14 @@
         }
     }
 
-    /// 三个内核此刻展开之后的表达式数与离预算的余量（2026-10-08 在删除线第 36 批的仓副本上用 naga =30.0.1 现量的：恢复内核补 F2（F2_EXACT 那一形，
-    /// 走读改成一个调用点走两遍）、`read_at` 拼读空段最后一槽、`read_unit_via_locations` 两条位置条目并成一个 `read_at` 调用点之后；
-    /// 改之前是恢复 320 015、扫描 382 259、走读 337 872）：
+    /// 三个内核此刻展开之后的表达式数与离预算的余量（2026-10-08 在删除线第 39 批的仓副本上用 naga =30.0.1 现量的：恢复内核的 F2 改成 F2_NEWEST 那一形
+    /// （退到 [见证到的那一版, 所选根) 里最新、不被所选根实例表抛弃的根，沿链读所选根的实例表），恢复 318 224 → 328 934，扫描与走读没变；
+    /// 第 36 批之前是恢复 320 015、扫描 382 259、走读 337 872）：
     /// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
     #[test]
     fn each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget() {
         for (name, source, pinned_expressions, pinned_margin) in [
-            ("recovery", recovery_kernel_source(), 318_224, 81_776),
+            ("recovery", recovery_kernel_source(), 328_934, 71_066),
             (
                 "checker-scan",
                 crate::crash_judge_gpu::checker_scan_kernel_source(),
diff --git a/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl b/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
--- a/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
@@ -86,12 +86,56 @@
     found: bool,
     root: RootFields,
 };
-// recovery::version_witnessed_by_the_system_configuration_to_fall_back_to，攻方 F2_EXACT 那一形（判决
-// `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行、第三节第 2 条）：所选那一版（一条记录都没施加的所选根本身）
-// 走读失败、系统配置没见证它时，只退到计数器 = c_见证 的那条记录所属那一版本身——根环里 (txg, 实例) 与它恰好相等、读得出的那条根
-// （R_ROOTS 按根环次序记着 choose_root 自证过的根，取第一条）。见证判不出、见证过、c_见证 = 0、那条记录读不出、那条根读不出都不退。
+// 择根的序（recovery 里 order_of_the_root_choice：checkpoint_txg 为主、实例代号破平局）上 (earlier_txg, earlier_instance) 排在 (later_txg, later_instance) 之前
+fn precedes_in_the_root_choice_order(earlier_txg: vec2<u32>, earlier_instance: u32, later_txg: vec2<u32>, later_instance: u32) -> bool {
+    return lt64(earlier_txg, later_txg) || (eq64(earlier_txg, later_txg) && earlier_instance < later_instance);
+}
+// recovery::instance_table_chain_of_root 读出的整张表的行，写进 R_ROWS（abandoned_by_rows 照它判）：沿链读法与走读判表读不读得出的
+// instance_table_chain_readable 同一份（unit::parse_packed_unit、InstanceTablePage::parse，链指针头部按走读那一种判），只多把每片的行记下来。
+// 一片读不出、解不开交 false；行多过 R_ROWS 装得下的、链长过 256 片的记判不了（CPU 不设这两个上限，那几个状态交 CPU 判）
+fn load_rows_of_the_instance_table_chain(root: RootFields) -> bool {
+    row_count = 0u;
+    var pointer = root.instance_table;
+    var page_index = vec2<u32>(0u, 0u);
+    for (var page = 0u; page < 256u; page = page + 1u) {
+        let page_read = read_unit_via_locations(pointer, DATA_UNIT_BYTES);
+        if (page_read.version == NONE) {
+            return false;
+        }
+        let page_unit = parse_packed_unit(page_read);
+        let parsed_page = parse_instance_table_page(page_unit, page_index);
+        if (parsed_page.verdict == 2u) {
+            return false;
+        }
+        for (var row_index = 0u; row_index + 1u < page_unit.record_count; row_index = row_index + 1u) {
+            if (row_count >= MAX_ROWS) {
+                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
+                return false;
+            }
+            let row_offset = packed_record_offset(page_unit, row_index);
+            sws(R_ROWS + row_count * 3u, rd32(page_unit.base, row_offset + 1u));
+            let row_txg = rd64(page_unit.base, row_offset + 5u);
+            sws(R_ROWS + row_count * 3u + 1u, row_txg.x);
+            sws(R_ROWS + row_count * 3u + 2u, row_txg.y);
+            row_count = row_count + 1u;
+        }
+        if (parsed_page.verdict == 0u) {
+            return true;
+        }
+        pointer = parsed_page.next;
+        page_index = add64(page_index, u64_of(1u));
+    }
+    undecidable = UNDECIDABLE_TOO_DEEP;
+    return false;
+}
+// recovery::version_witnessed_by_the_system_configuration_to_fall_back_to，F2_NEWEST 那一形（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md`
+// 第二节 K1 / K2 W4 H4 行与第三节第 2 条，用户 2026-10-08 定「四形都修」）：所选那一版（一条记录都没施加的所选根本身）走读失败、系统配置没见证它时，
+// 退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（R_ROOTS 按根环次序记着 choose_root 自证过的根；同为最新的几条取根环次序上
+// 最后那条，同 max_by_key）；所选根的实例表读不出、解不开时只认 (txg, 实例) 与见证到的那一版恰好相等的那条。「见证到的那一版」是计数器 = c_见证 的那条记录
+// 所属那一版。见证判不出、见证过、那条记录读不出都不退；c_见证 = 0 不另设守卫，与 CPU 同走「那条记录读不出」：写者的计数器从 1 起，环里没有计数器 0 的记录，
+// 伪造出来的那一条两边一样找得到（判决同一份第二节 K1 V1 行「建模差两处」之二）。退回的那一版不重放、直接走。
 // 「见证过」：所选那一版带末条标志的记录读得出 ⇒ 它的计数器 ≤ c_见证；读不出而 c_见证 那条记录读得出 ⇒ 那条记录的 (txg, 实例) 不比所选那一版旧；
-// 两条都读不出当见证过
+// 两条都读不出当见证过。c_见证 落在所选那次发布自己的某条记录上时下界就是所选那一版、区间是空的，同样不退
 fn version_witnessed_by_the_system_configuration_to_fall_back_to(cfg: SysConfig, chosen: RootFields, selected: RootFields) -> FallbackVersion {
     var out: FallbackVersion;
     out.found = false;
@@ -108,6 +152,10 @@
         var self_verified_slots = 0u;
         for (var slot = 0u; slot < 2u; slot = slot + 1u) {
             let offset = select(0u, cfg.spacing, slot == 1u);
+            // 内存池另把越过盘尾的读报成读错（check_request_against_the_device_contract 的 OutOfRange），这里不另判那一支：走到这一步时所选根已经
+            // 从根环里读出来了（根环起点 ROOT_RING_BASE_BYTES = 1 MiB，CPU 读得出就说明盘至少这么大，池里每块盘一样大），读者收下的槽距不超过
+            // 1 MiB − 4096（parse_system_configuration 的 largest_spacing），两槽 [0, 4096)、[槽距, 槽距 + 4096) 恒在盘内（判决
+            // `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 V1 行「建模差两处」之一：在走得到的状态上两边同判）
             if ((offset % SECTOR_BYTES_OF_THE_MEMORY_POOL) != 0u) {
                 some_slot_reports_a_read_error = true;
                 continue;
@@ -157,21 +205,28 @@
     } else if (has_witnessed_record) {
         is_witnessed = !witnessed_record_is_older_than_the_selected_version;
     }
-    if (is_witnessed || is_zero64(witnessed_counter) || !has_witnessed_record) {
+    if (is_witnessed || !has_witnessed_record) {
         return out;
     }
-    for (var k = 0u; k < root_count; k = k + 1u) {
-        var s: Slice = no_slice();
-        s.version = sw(R_ROOTS + k * 2u);
-        s.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, s.version * VERSION_WORDS + 1u);
-        s.length = cfg.physical_block_size;
-        s.c0 = tab(T_VERSIONS, s.version * VERSION_WORDS + 2u);
-        let root = parse_root(s, cfg.fsid);
-        if (root.instance == witnessed_record_instance && eq64(root.txg, witnessed_record_txg)
-            && (lt64(root.txg, chosen.txg) || (eq64(root.txg, chosen.txg) && root.instance < chosen.instance))) {
+    let chosen_instance_table_is_readable = load_rows_of_the_instance_table_chain(chosen);
+    for (var root_index = 0u; root_index < root_count; root_index = root_index + 1u) {
+        var root_read: Slice = no_slice();
+        root_read.version = sw(R_ROOTS + root_index * 2u);
+        root_read.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, root_read.version * VERSION_WORDS + 1u);
+        root_read.length = cfg.physical_block_size;
+        root_read.c0 = tab(T_VERSIONS, root_read.version * VERSION_WORDS + 2u);
+        let root = parse_root(root_read, cfg.fsid);
+        let is_the_witnessed_version = root.instance == witnessed_record_instance && eq64(root.txg, witnessed_record_txg);
+        var is_admissible = is_the_witnessed_version;
+        if (chosen_instance_table_is_readable) {
+            is_admissible = !abandoned_by_rows(root.instance, root.txg);
+        }
+        let is_in_the_range = !precedes_in_the_root_choice_order(root.txg, root.instance, witnessed_record_txg, witnessed_record_instance)
+            && precedes_in_the_root_choice_order(root.txg, root.instance, chosen.txg, chosen.instance);
+        if (is_in_the_range && is_admissible
+            && (!out.found || !precedes_in_the_root_choice_order(root.txg, root.instance, out.root.txg, out.root.instance))) {
             out.found = true;
             out.root = root;
-            return out;
         }
     }
     return out;
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -62,6 +62,9 @@
 //! （V 的根槽也读不出，报读不出、不退到 V⁻）各一条流；I-3.5 按全部版本走完之后的折叠表取单元（H-S1-fold A 臂）一条流；read_at 拼读那三形
 //! （空段右端一槽按 32 KiB 读的数据指针记判不了回落 CPU、「16 KiB 节点加下一槽」照表判、F-α 的「空槽加下一已写槽前 16 KiB」拿拼出的 CRC 照比）各一条流；
 //! 删完那条流钉 CPU 一个违例都没有，C744 ① 那条流钉两种版本同在的状态至少一个；借卡一律经 `lease_card_or_report_not_run`（借不到报本次未跑）。
+//! 第 39 批（删除线）：GPU 恢复内核的 F2 改成 F2_NEWEST 那一形（退到 [见证到的那一版, 所选根) 里最新、不被所选根实例表抛弃的根）——攻方 H4
+//! （V 那次轮换的系统配置槽读回全零，落已确认的 V、不落 V⁻）、所选根的实例表抛弃 V（落 V⁻）、所选根的实例表读不出（只认恰好相等的 V⁻）、
+//! c_见证 = 0 而环里有一条计数器 0 的伪造记录（两边一样找到它、退到它那一版）各一条流，都整份比。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
 //! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
@@ -11575,8 +11578,9 @@
 
 /// F2 设计里那一退（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行「设计那一退照旧落 V」）：R 的根槽落了、走读失败，
 /// 系统配置没见证 R、见证到 V，CPU 恢复退到 V 本身、读出第二份内容，oracle 判「落到更旧的根」（oracle 不认 F2 那一退，判决第三节第 4 条不改）。
-/// GPU 恢复内核照 `recovery::version_witnessed_by_the_system_configuration_to_fall_back_to` 的 F2_EXACT 那一形退（`crash_judge_recovery.wgsl` 的
-/// 同名函数），每个状态逐位相同、落到同一条根。没有它的 GPU 在这几个状态上报读不出（第 33 批 T 合入之后对拍红的那 52 条流同一个病根）。
+/// GPU 恢复内核照 `recovery::version_witnessed_by_the_system_configuration_to_fall_back_to` 退（`crash_judge_recovery.wgsl` 的同名函数），每个状态逐位相同、
+/// 落到同一条根。没有它的 GPU 在这几个状态上报读不出（第 33 批 T 合入之后对拍红的那 52 条流同一个病根）。F2 改成 F2_NEWEST（第 39 批，判决
+/// `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 / K2 W4 H4 行）之后这一退不变：[V, R) 里最新、不被 R 的实例表抛弃的根就是 V。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_newest_root_does_not_read_back_falls_back_to_the_version_the_system_configuration_witnessed(
 ) {
@@ -11623,9 +11627,9 @@
 }
 
 /// 攻方 A 形（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行，攻方 `research/prompts/m3-batch33t-code-r1-opus-output.md`
-/// 第一节 1）：同上一条流，V 的根槽写自证校验和字段翻一位（V 的根槽读不出），R 走读失败、系统配置见证到 V 的状态上，F2_EXACT 只退到 V 本身、
-/// V 的根读不出就不退——CPU 报读不出（落到的根照旧是 R），一个状态都不退到 V⁻；GPU 恢复内核同判，每个状态逐位相同。
-/// 退到「比所选根旧、不超过见证上限的最新可读根」（第 33 批 T 的写法）会落到 V⁻、把已确认的 V 越过去，这条红。
+/// 第一节 1）：同上一条流，V 的根槽写自证校验和字段翻一位（V 的根槽读不出），R 走读失败、系统配置见证到 V 的状态上，F2_NEWEST（第 39 批）只在
+/// [V, R) 里找：V 的根读不出、区间里再没有别的根，就不退——CPU 报读不出（落到的根照旧是 R），一个状态都不退到 V⁻；GPU 恢复内核同判，每个状态逐位相同。
+/// 退到「比所选根旧、不超过见证上限的最新可读根」（第 33 批 T 的写法）或不设下界，会落到 V⁻、把已确认的 V 越过去，这条红。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_the_witnessed_version_whose_root_does_not_read_back(
 ) {
@@ -11679,7 +11683,7 @@
     );
     assert_eq!(
         states_falling_back_past_v, 0,
-        "流 {}：一个状态都不退到 V⁻ {V_MINUS_OF_THE_F2_FLOWS:?}（F2 只退到见证到的那一版本身）",
+        "流 {}：一个状态都不退到 V⁻ {V_MINUS_OF_THE_F2_FLOWS:?}（F2 只退到见证到的那一版 V 及之后、所选根之前）",
         flow.name
     );
     assert!(
@@ -11691,6 +11695,319 @@
     );
 }
 
+// ─── 第 39 批（删除线）：GPU 恢复内核的 F2 改成 F2_NEWEST 那一形（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节
+// K1 / K2 W4 H4 行、K1 V1 行与第三节第 2 条） ───
+
+/// V 那次发布轮换写的系统配置槽：V 的根槽写之后、R 的根槽写之前的系统配置槽写（每次发布按单元、记录、根槽、系统配置轮换的次序落盘），每块盘一槽。
+fn system_configuration_rotation_writes_of_v(flow: &RecordedFlow) -> Vec<usize> {
+    let root_of_v = root_write_of_txg(flow, V_OF_THE_F2_FLOWS.1);
+    let root_of_r = root_write_of_txg(flow, R_OF_THE_F2_FLOWS.1);
+    let rotation: Vec<usize> = (root_of_v + 1..root_of_r)
+        .filter(|index| flow.writes[*index].kind == StepKind::SystemConfigurationSlot)
+        .collect();
+    assert_eq!(
+        rotation.len(),
+        singlefs_core::recovery::PoolReader::device_identities(&flow.base).len(),
+        "V 那次轮换每块盘写一槽：{rotation:?}"
+    );
+    rotation
+}
+
+/// 攻方 H4 的底（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 / K2 W4 H4 行；攻方
+/// `research/prompts/m3-batch35-36-code-r1-opus-output.md` 第四节，攻方的 Pm、P、R 是这里的 V⁻、V、R）：F2 那两条流的底上，V 那次发布轮换写的系统配置槽
+/// （每块盘一槽，是两槽里较新的那一槽）写成全零。R 的根槽落了、R 的轮换没落的状态上，每块盘读得出的只剩 V⁻ 那次轮换写的那一槽，
+/// c_见证 落回 V⁻ 那次发布末条记录的计数器，系统配置没见证 R；V 已确认、根与单元都读得出。
+fn record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
+    tag: &str,
+    name: &'static str,
+) -> RecordedFlow {
+    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(tag, name);
+    for rotation_write in system_configuration_rotation_writes_of_v(&flow) {
+        let slot_bytes = usize::try_from(flow.writes[rotation_write].length_in_bytes())
+            .expect("系统配置槽写 4096 字节装得进 usize");
+        flow.writes[rotation_write].contents = WrittenContents::Bytes(vec![0u8; slot_bytes]);
+    }
+    flow
+}
+
+/// 看 journal 那一遍有几个状态经 F2 退到 `version`：CPU 报的根是它，oracle 判「落到更旧的根」（所选根 R 落了而走的是更旧的这一版）。
+fn states_where_the_consulted_recovery_falls_back_to(
+    comparison: &Comparison,
+    version: (u32, u64),
+) -> usize {
+    let recovered_to_an_older_root =
+        consulted_oracle_item(Layer0OracleViolationKind::RecoveredToAnOlderRoot);
+    (0..comparison.cpu_red_items_of_each_state.len())
+        .filter(|ordinal| {
+            comparison.cpu_red_items_of_each_state[*ordinal].contains(&recovered_to_an_older_root)
+                && comparison.cpu_consulted_reported_root_of_each_state[*ordinal] == Some(version)
+        })
+        .count()
+}
+
+/// F2 的四条流共用的判法：CPU 看 journal 那一遍至少一个状态退到 `landed_version`（读出它的内容）、一个状态都不退到 `version_not_landed_on`，
+/// GPU 每个状态逐位相同、落到同一条根。本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
+fn assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
+    test_name: &str,
+    flow: &RecordedFlow,
+    landed_version: (u32, u64),
+    version_not_landed_on: (u32, u64),
+) {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
+        return;
+    };
+    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
+        return;
+    };
+    let recovered_to_an_older_root =
+        consulted_oracle_item(Layer0OracleViolationKind::RecoveredToAnOlderRoot);
+    let states_falling_back_to_the_landed_version = (0..comparison
+        .cpu_red_items_of_each_state
+        .len())
+        .filter(|ordinal| {
+            comparison.cpu_red_items_of_each_state[*ordinal].contains(&recovered_to_an_older_root)
+                && comparison.cpu_consulted_reported_root_of_each_state[*ordinal]
+                    == Some(landed_version)
+                && consulted_recovery_lands_on(
+                    &comparison,
+                    *ordinal,
+                    landed_version,
+                    RECOVERY_WORD_CLASS_FILE_READ,
+                )
+        })
+        .count();
+    let states_falling_back_to_the_other_version =
+        states_where_the_consulted_recovery_falls_back_to(&comparison, version_not_landed_on);
+    eprintln!(
+        "CRASH_JUDGE_GPU_F2 flow={} states={} states_falling_back_to_{landed_version:?}={states_falling_back_to_the_landed_version} states_falling_back_to_{version_not_landed_on:?}={states_falling_back_to_the_other_version}",
+        flow.name, comparison.states
+    );
+    assert!(
+        states_falling_back_to_the_landed_version > 0,
+        "流 {}：R 走读失败、系统配置没见证 R 的状态上，CPU 看 journal 那一遍退到 {landed_version:?}、读出它的内容",
+        flow.name
+    );
+    assert_eq!(
+        states_falling_back_to_the_other_version, 0,
+        "流 {}：一个状态都不退到 {version_not_landed_on:?}",
+        flow.name
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
+/// 攻方 H4（底见 [`record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros`]）：
+/// R 走读失败、c_见证 落回 V⁻ 的状态上，F2_NEWEST 退到 [V⁻, R) 里读得出、不被 R 的实例表抛弃的最新一条根——已确认、单元完好的 V；
+/// F2_EXACT 只认 V⁻、把 V 越过去（攻方原样：三处都落 Pm）。CPU 看 journal 那一遍至少一个状态退到 V、一个状态都不退到 V⁻，GPU 逐位相同。
+/// GPU 恢复内核读得出所选根的实例表也只认恰好相等的那条、或把下界换成见证上限（第 33 批 T 的写法）时 GPU 落 V⁻，这条红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_falls_back_to_the_newest_version_from_the_witnessed_one_when_the_newer_system_configuration_slots_read_back_as_zeros(
+) {
+    let flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
+        "crash-judge-gpu-f2-newest-newer-rotation-zeroed",
+        "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed",
+    );
+    assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_falls_back_to_the_newest_version_from_the_witnessed_one_when_the_newer_system_configuration_slots_read_back_as_zeros",
+        &flow,
+        V_OF_THE_F2_FLOWS,
+        V_MINUS_OF_THE_F2_FLOWS,
+    );
+}
+
+/// 实例表那一片单元（码 3、记录类型 4）最前面插一行 (`instance`, `selected_root_txg`, W = 0)：原有的记录往后挪一条，记录数与声明长度各加一条
+/// （单元自己的两道校验和由调用方重封）。
+fn insert_an_instance_table_row_in_front(bytes: &mut [u8], instance: u32, selected_root_txg: u64) {
+    let record_count = usize::from(u16::from_le_bytes(
+        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
+            .try_into()
+            .expect("2 字节"),
+    ));
+    let records_end = PACKED_UNIT_RECORDS_START + record_count * INSTANCE_TABLE_RECORD_BYTES;
+    bytes.copy_within(
+        PACKED_UNIT_RECORDS_START..records_end,
+        PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_RECORD_BYTES,
+    );
+    let row = &mut bytes
+        [PACKED_UNIT_RECORDS_START..PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_RECORD_BYTES];
+    row.fill(0);
+    row[1..5].copy_from_slice(&instance.to_le_bytes());
+    row[5..13].copy_from_slice(&selected_root_txg.to_le_bytes());
+    let new_record_count = record_count + 1;
+    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
+        &u16::try_from(new_record_count)
+            .expect("记录数装得进 2 字节")
+            .to_le_bytes(),
+    );
+    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
+        &u16::try_from(new_record_count * INSTANCE_TABLE_RECORD_BYTES)
+            .expect("声明长度装得进 2 字节")
+            .to_le_bytes(),
+    );
+}
+
+/// 所选根的实例表抛弃了区间里最新的那条根：H4 的底上，实例 1 那张实例表（同一实例里不重写，V⁻、V、R 指着同一张）最前面插一行 (1, 3)、重封到根——
+/// 按 R 的实例表，实例 1 里 txg > 3 的根（V 与 R）在被抛弃的时间线上。R 走读失败、c_见证 落回 V⁻ 的状态上，F2_NEWEST 在 [V⁻, R) 里滤掉 V、落 V⁻。
+/// CPU 看 journal 那一遍至少一个状态退到 V⁻、一个状态都不退到 V，GPU 逐位相同。GPU 恢复内核读得出所选根的实例表而不按它滤时落 V，这条红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons(
+) {
+    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
+        "crash-judge-gpu-f2-newest-abandoned-by-the-chosen-instance-table",
+        "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed-instance-table-abandons-v",
+    );
+    let instance_table = |bytes: &[u8]| {
+        unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INSTANCE_TABLE
+    };
+    let copies = last_unit_copies(&flow, "实例 1 的实例表", &instance_table, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &copies, &|bytes| {
+        insert_an_instance_table_row_in_front(
+            bytes,
+            V_MINUS_OF_THE_F2_FLOWS.0,
+            V_MINUS_OF_THE_F2_FLOWS.1,
+        );
+    });
+    assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons",
+        &flow,
+        V_MINUS_OF_THE_F2_FLOWS,
+        V_OF_THE_F2_FLOWS,
+    );
+}
+
+/// 所选根的实例表读不出：H4 的底上，R 的根槽写里实例表指针两条位置条目的整单元校验和各翻一位、重封根记录（V⁻、V 的根记录不动，它们指着的表照样读得出）。
+/// R 走读失败、c_见证 落回 V⁻ 的状态上，F2_NEWEST 只认与见证到的那一版恰好相等的那条根、落 V⁻。CPU 看 journal 那一遍至少一个状态退到 V⁻、
+/// 一个状态都不退到 V，GPU 逐位相同。GPU 恢复内核读不出所选根的实例表也照取最新时落 V，这条红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_chosen_root_instance_table_does_not_read_back_falls_back_only_to_the_witnessed_version(
+) {
+    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
+        "crash-judge-gpu-f2-newest-chosen-instance-table-unreadable",
+        "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed-chosen-instance-table-unreadable",
+    );
+    let root_of_r = root_write_of_txg(&flow, R_OF_THE_F2_FLOWS.1);
+    rewrite_root_record_and_reseal(&mut flow, root_of_r, &|bytes| {
+        for checksum_offset in POINTER_LOCATION_CHECKSUM_OFFSETS {
+            bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + checksum_offset] ^= 0x01;
+        }
+    });
+    assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_chosen_root_instance_table_does_not_read_back_falls_back_only_to_the_witnessed_version",
+        &flow,
+        V_MINUS_OF_THE_F2_FLOWS,
+        V_OF_THE_F2_FLOWS,
+    );
+}
+
+/// 系统配置槽里 journal tail 的偏移（`system_configuration.rs` 的 TAIL_OFFSET，8 字节）。
+const SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET: usize = 469;
+/// journal 记录的字节数（一个记录槽）。
+const JOURNAL_RECORD_SLOT_BYTES: u64 = 4096;
+/// journal 环的起点（`JOURNAL_RING_START_SLOT` × 16 KiB = 16 MiB）。
+const JOURNAL_RING_START_BYTES: u64 = 1024 * 16384;
+/// 伪造的那条记录放在同一块盘上流里最后一个记录槽之后第几个槽：离开流里写过的槽，仍在环里。
+const RECORD_SLOTS_PAST_THE_LAST_RECORD_OF_THE_FLOW: u64 = 8;
+
+/// c_见证 = 0 而环里有一条计数器 0 的记录（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 V1 行「建模差两处」之二，
+/// 正推 `research/prompts/m3-batch35-36-code-r1-sonnet-output.md` V1 那一格：GPU 原来对 c_见证 = 0 另设守卫、不退，CPU 照找计数器 = 0 的那条记录）：
+/// F2 设计那一退的底上，流里每一次系统配置槽写的 journal tail 改成 0、重封（c_见证 恒为 0）；流尾另起一段，写一条 V 那次发布末条记录的副本，
+/// 计数器改成 0、重封，放在同一块盘上流里最后一个记录槽之后（写者写不出计数器 0 的记录，这是伪造的）。那一段落了的状态上 R 走读失败，
+/// 计数器 = c_见证 = 0 的那条记录读得出、属于 V，CPU 退到 [V, R) 里最新的 V；那一段没落的状态上没有那条记录，照报读不出。
+fn record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_v(
+    tag: &str,
+    name: &'static str,
+) -> RecordedFlow {
+    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(tag, name);
+    let system_configuration_writes = flow
+        .writes
+        .iter()
+        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
+        .count();
+    for nth_from_the_end in 0..system_configuration_writes {
+        rewrite_and_reseal(
+            &mut flow,
+            StepKind::SystemConfigurationSlot,
+            nth_from_the_end,
+            4096,
+            SYSTEM_CONFIGURATION_CHECKSUM_FIELD,
+            &|bytes| {
+                bytes[SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET
+                    ..SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET + 8]
+                    .fill(0);
+            },
+        );
+    }
+    let is_the_last_record_of_v = |write: &RetainedWrite| {
+        write.kind == StepKind::JournalRecord
+            && write.bytes().is_some_and(|bytes| {
+                u32::from_le_bytes(bytes[16..20].try_into().expect("4 字节")) == V_OF_THE_F2_FLOWS.0
+                    && u64::from_le_bytes(bytes[26..34].try_into().expect("8 字节"))
+                        == V_OF_THE_F2_FLOWS.1
+                    && bytes[7] & 1 == 1
+            })
+    };
+    let last_record_of_v = flow
+        .writes
+        .iter()
+        .rposition(is_the_last_record_of_v)
+        .expect("流里有 V 那次发布带末条标志的记录");
+    let device = flow.writes[last_record_of_v].device;
+    let mut forged_record = flow.writes[last_record_of_v]
+        .bytes()
+        .expect("记录写带字节")
+        .to_vec();
+    forged_record[JOURNAL_RECORD_COUNTER_OFFSET..JOURNAL_RECORD_COUNTER_OFFSET + 6].fill(0);
+    reseal_journal_record(&mut forged_record);
+    let last_record_slot_of_the_flow = flow
+        .writes
+        .iter()
+        .filter(|write| write.kind == StepKind::JournalRecord && write.device == device)
+        .map(|write| write.offset.0)
+        .max()
+        .expect("这块盘上有记录写");
+    let forged_record_offset = last_record_slot_of_the_flow
+        + RECORD_SLOTS_PAST_THE_LAST_RECORD_OF_THE_FLOW * JOURNAL_RECORD_SLOT_BYTES;
+    assert!(
+        forged_record_offset + JOURNAL_RECORD_SLOT_BYTES
+            <= JOURNAL_RING_START_BYTES + geometry().journal_ring_bytes,
+        "伪造的记录在环里"
+    );
+    append_a_segment_of_writes(
+        &mut flow,
+        vec![RetainedWrite {
+            device,
+            kind: StepKind::JournalRecord,
+            is_force_unit_access: false,
+            offset: singlefs_core::address::DeviceOffsetInBytes(forged_record_offset),
+            contents: WrittenContents::Bytes(forged_record),
+        }],
+    );
+    flow
+}
+
+/// c_见证 = 0 而环里有一条计数器 0 的记录（底见 [`record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_v`]）：
+/// GPU 恢复内核对 c_见证 = 0 不另设守卫，与 CPU 一样找计数器 = 0 的那条记录、退到它那一版。CPU 看 journal 那一遍至少一个状态退到 V、
+/// 一个状态都不退到 V⁻，GPU 逐位相同。GPU 把守卫加回去（c_见证 = 0 不退）时报读不出、与 CPU 不一致，这条红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_falls_back_to_the_version_of_a_record_carrying_counter_zero(
+) {
+    let flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_v(
+        "crash-judge-gpu-f2-witnessed-counter-zero",
+        "two-overwrites-second-tree-table-unreadable-every-tail-zero-forged-record-of-counter-zero",
+    );
+    assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_falls_back_to_the_version_of_a_record_carrying_counter_zero",
+        &flow,
+        V_OF_THE_F2_FLOWS,
+        V_MINUS_OF_THE_F2_FLOWS,
+    );
+}
+
 /// extent 树的种类码（`singlefs_core::records::TREE_KIND_EXTENT`）。
 const TREE_KIND_OF_THE_EXTENT_TREE: u16 = 1;
 
````

## 删除线第 39 批 Z：mutations-append.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch39/z/patch/mutations-append.tsv）

````text
第39批 Z GPU 恢复内核 F2 读得出所选根的实例表也只认与见证到的那一版恰好相等的根（攻方 H4 落 V⁻、CPU 落 V）	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	            is_admissible = !abandoned_by_rows(root.instance, root.txg);	            is_admissible = is_the_witnessed_version;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_recovery_falls_back_to_the_newest_version_from_the_witnessed_one_when_the_newer_system_configuration_slots_read_back_as_zeros	the_gpu_judge_agrees_with_the_cpu_that_a_recovery_falls_back_to_the_newest_version_from_the_witnessed_one_when_the_newer_system_configuration_slots_read_back_as_zeros
第39批 Z GPU 恢复内核 F2 读得出所选根的实例表而不按它滤（被所选根实例表抛弃的 V 照退、CPU 落 V⁻）	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	            is_admissible = !abandoned_by_rows(root.instance, root.txg);	            is_admissible = true;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons	the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons
第39批 Z GPU 恢复内核 F2 读不出所选根的实例表也照取区间里最新的根（GPU 落 V、CPU 只认恰好相等的 V⁻）	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	        var is_admissible = is_the_witnessed_version;	        var is_admissible = true;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_chosen_root_instance_table_does_not_read_back_falls_back_only_to_the_witnessed_version	the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_chosen_root_instance_table_does_not_read_back_falls_back_only_to_the_witnessed_version
第39批 Z GPU 恢复内核 F2 对 c_见证 = 0 另设守卫、不找计数器 0 的记录（伪造的计数器 0 记录上 GPU 报读不出、CPU 落 V）	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	    if (is_witnessed || !has_witnessed_record) {	    if (is_witnessed || is_zero64(witnessed_counter) || !has_witnessed_record) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_falls_back_to_the_version_of_a_record_carrying_counter_zero	the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_falls_back_to_the_version_of_a_record_carrying_counter_zero
````

## 删除线第 39 批 Z：mutations-replacements.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch39/z/patch/mutations-replacements.tsv）

````text
第36批 V GPU 恢复内核 F2 退到见证上限之下第一条读得出的根	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	        let is_in_the_range = !precedes_in_the_root_choice_order(root.txg, root.instance, witnessed_record_txg, witnessed_record_instance)	        let is_in_the_range = !precedes_in_the_root_choice_order(witnessed_record_txg, witnessed_record_instance, root.txg, root.instance)	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_the_witnessed_version_whose_root_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_the_witnessed_version_whose_root_does_not_read_back
````

## 删除线第 41 批：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch41/a/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl b/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
index 6f85303..6428540 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
@@ -55,7 +55,7 @@ fn recover(consult: bool, cfg_choice: SysChoice) -> Recovery {
         if (walk_round == 1u || walked.outcome != OUTCOME_FAILED || !consult) {
             break;
         }
-        let fallback = version_witnessed_by_the_system_configuration_to_fall_back_to(cfg_choice.cfg, chosen.root, effective);
+        let fallback = newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(cfg_choice.cfg, chosen.root, effective);
         if (!fallback.found) {
             break;
         }
@@ -128,15 +128,15 @@ fn load_rows_of_the_instance_table_chain(root: RootFields) -> bool {
     undecidable = UNDECIDABLE_TOO_DEEP;
     return false;
 }
-// recovery::version_witnessed_by_the_system_configuration_to_fall_back_to，F2_NEWEST 那一形（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md`
+// recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to，F2_NEWEST 那一形（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md`
 // 第二节 K1 / K2 W4 H4 行与第三节第 2 条，用户 2026-10-08 定「四形都修」）：所选那一版（一条记录都没施加的所选根本身）走读失败、系统配置没见证它时，
 // 退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（R_ROOTS 按根环次序记着 choose_root 自证过的根；同为最新的几条取根环次序上
 // 最后那条，同 max_by_key）；所选根的实例表读不出、解不开时只认 (txg, 实例) 与见证到的那一版恰好相等的那条。「见证到的那一版」是计数器 = c_见证 的那条记录
-// 所属那一版。见证判不出、见证过、那条记录读不出都不退；c_见证 = 0 不另设守卫，与 CPU 同走「那条记录读不出」：写者的计数器从 1 起，环里没有计数器 0 的记录，
-// 伪造出来的那一条两边一样找得到（判决同一份第二节 K1 V1 行「建模差两处」之二）。退回的那一版不重放、直接走。
+// 所属那一版。见证判不出、见证过、那条记录读不出、c_见证 = 0 都不退：c_见证 = 0 时环里读得出计数器 0 的记录（写者的计数器从 1 起，那一条只能是
+// 坏盘或伪造的）也不退，与 CPU 同一个守卫（D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」）。退回的那一版不重放、直接走。
 // 「见证过」：所选那一版带末条标志的记录读得出 ⇒ 它的计数器 ≤ c_见证；读不出而 c_见证 那条记录读得出 ⇒ 那条记录的 (txg, 实例) 不比所选那一版旧；
 // 两条都读不出当见证过。c_见证 落在所选那次发布自己的某条记录上时下界就是所选那一版、区间是空的，同样不退
-fn version_witnessed_by_the_system_configuration_to_fall_back_to(cfg: SysConfig, chosen: RootFields, selected: RootFields) -> FallbackVersion {
+fn newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(cfg: SysConfig, chosen: RootFields, selected: RootFields) -> FallbackVersion {
     var out: FallbackVersion;
     out.found = false;
     out.root = invalid_root();
@@ -205,7 +205,7 @@ fn version_witnessed_by_the_system_configuration_to_fall_back_to(cfg: SysConfig,
     } else if (has_witnessed_record) {
         is_witnessed = !witnessed_record_is_older_than_the_selected_version;
     }
-    if (is_witnessed || !has_witnessed_record) {
+    if (is_witnessed || is_zero64(witnessed_counter) || !has_witnessed_record) {
         return out;
     }
     let chosen_instance_table_is_readable = load_rows_of_the_instance_table_chain(chosen);
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 0819837..c5f457a 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -64,7 +64,7 @@
 //! 删完那条流钉 CPU 一个违例都没有，C744 ① 那条流钉两种版本同在的状态至少一个；借卡一律经 `lease_card_or_report_not_run`（借不到报本次未跑）。
 //! 第 39 批（删除线）：GPU 恢复内核的 F2 改成 F2_NEWEST 那一形（退到 [见证到的那一版, 所选根) 里最新、不被所选根实例表抛弃的根）——攻方 H4
 //! （V 那次轮换的系统配置槽读回全零，落已确认的 V、不落 V⁻）、所选根的实例表抛弃 V（落 V⁻）、所选根的实例表读不出（只认恰好相等的 V⁻）、
-//! c_见证 = 0 而环里有一条计数器 0 的伪造记录（两边一样找到它、退到它那一版）各一条流，都整份比。
+//! c_见证 = 0 而环里有一条计数器 0 的伪造记录各一条流，都整份比；第 41 批（删除线）CPU 与 GPU 一起对 c_见证 = 0 设守卫，那一条流改成两边都不退、报读不出。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
 //! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
@@ -11578,7 +11578,7 @@ fn consulted_recovery_lands_on(
 
 /// F2 设计里那一退（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行「设计那一退照旧落 V」）：R 的根槽落了、走读失败，
 /// 系统配置没见证 R、见证到 V，CPU 恢复退到 V 本身、读出第二份内容，oracle 判「落到更旧的根」（oracle 不认 F2 那一退，判决第三节第 4 条不改）。
-/// GPU 恢复内核照 `recovery::version_witnessed_by_the_system_configuration_to_fall_back_to` 退（`crash_judge_recovery.wgsl` 的同名函数），每个状态逐位相同、
+/// GPU 恢复内核照 `recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to` 退（`crash_judge_recovery.wgsl` 的同名函数），每个状态逐位相同、
 /// 落到同一条根。没有它的 GPU 在这几个状态上报读不出（第 33 批 T 合入之后对拍红的那 52 条流同一个病根）。F2 改成 F2_NEWEST（第 39 批，判决
 /// `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 / K2 W4 H4 行）之后这一退不变：[V, R) 里最新、不被 R 的实例表抛弃的根就是 V。
 #[test]
@@ -11914,10 +11914,10 @@ const JOURNAL_RING_START_BYTES: u64 = 1024 * 16384;
 const RECORD_SLOTS_PAST_THE_LAST_RECORD_OF_THE_FLOW: u64 = 8;
 
 /// c_见证 = 0 而环里有一条计数器 0 的记录（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 V1 行「建模差两处」之二，
-/// 正推 `research/prompts/m3-batch35-36-code-r1-sonnet-output.md` V1 那一格：GPU 原来对 c_见证 = 0 另设守卫、不退，CPU 照找计数器 = 0 的那条记录）：
+/// 正推 `research/prompts/m3-batch35-36-code-r1-sonnet-output.md` V1 那一格）：
 /// F2 设计那一退的底上，流里每一次系统配置槽写的 journal tail 改成 0、重封（c_见证 恒为 0）；流尾另起一段，写一条 V 那次发布末条记录的副本，
-/// 计数器改成 0、重封，放在同一块盘上流里最后一个记录槽之后（写者写不出计数器 0 的记录，这是伪造的）。那一段落了的状态上 R 走读失败，
-/// 计数器 = c_见证 = 0 的那条记录读得出、属于 V，CPU 退到 [V, R) 里最新的 V；那一段没落的状态上没有那条记录，照报读不出。
+/// 计数器改成 0、重封，放在同一块盘上流里最后一个记录槽之后（写者写不出计数器 0 的记录，这是伪造的）。R 走读失败的状态上，那一段落没落，
+/// CPU 与 GPU 都按 D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」不退、照报 R 读不出。
 fn record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_v(
     tag: &str,
     name: &'static str,
@@ -11991,20 +11991,62 @@ fn record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witne
 }
 
 /// c_见证 = 0 而环里有一条计数器 0 的记录（底见 [`record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_v`]）：
-/// GPU 恢复内核对 c_见证 = 0 不另设守卫，与 CPU 一样找计数器 = 0 的那条记录、退到它那一版。CPU 看 journal 那一遍至少一个状态退到 V、
-/// 一个状态都不退到 V⁻，GPU 逐位相同。GPU 把守卫加回去（c_见证 = 0 不退）时报读不出、与 CPU 不一致，这条红。
+/// CPU 与 GPU 恢复内核都对 c_见证 = 0 设守卫、不退（D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」）。CPU 看 journal 那一遍至少一个状态
+/// 照报 R 读不出、一个状态都不退到 V 或 V⁻，GPU 逐位相同。GPU 去掉守卫（找计数器 = 0 的那条记录、退到 V）时与 CPU 不一致，这条红；
+/// CPU 去掉守卫时伪造那一段落了的状态退到 V，这条也红。
 #[test]
-fn the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_falls_back_to_the_version_of_a_record_carrying_counter_zero(
+fn the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_does_not_fall_back_even_to_the_version_of_a_record_carrying_counter_zero(
 ) {
+    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_does_not_fall_back_even_to_the_version_of_a_record_carrying_counter_zero";
     let flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_v(
         "crash-judge-gpu-f2-witnessed-counter-zero",
         "two-overwrites-second-tree-table-unreadable-every-tail-zero-forged-record-of-counter-zero",
     );
-    assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
-        "the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_falls_back_to_the_version_of_a_record_carrying_counter_zero",
-        &flow,
-        V_OF_THE_F2_FLOWS,
-        V_MINUS_OF_THE_F2_FLOWS,
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
+        return;
+    };
+    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
+        return;
+    };
+    let read_failed = consulted_oracle_item(Layer0OracleViolationKind::ReadFailed);
+    let states_reporting_r_unreadable = (0..comparison.cpu_red_items_of_each_state.len())
+        .filter(|ordinal| {
+            comparison.cpu_red_items_of_each_state[*ordinal].contains(&read_failed)
+                && comparison.cpu_consulted_reported_root_of_each_state[*ordinal]
+                    == Some(R_OF_THE_F2_FLOWS)
+                && consulted_recovery_lands_on(
+                    &comparison,
+                    *ordinal,
+                    R_OF_THE_F2_FLOWS,
+                    RECOVERY_WORD_CLASS_FAILED,
+                )
+        })
+        .count();
+    let states_falling_back_to_v =
+        states_where_the_consulted_recovery_falls_back_to(&comparison, V_OF_THE_F2_FLOWS);
+    let states_falling_back_to_v_minus =
+        states_where_the_consulted_recovery_falls_back_to(&comparison, V_MINUS_OF_THE_F2_FLOWS);
+    eprintln!(
+        "CRASH_JUDGE_GPU_F2 flow={} states={} states_reporting_r_unreadable={states_reporting_r_unreadable} states_falling_back_to_v={states_falling_back_to_v} states_falling_back_to_v_minus={states_falling_back_to_v_minus}",
+        flow.name, comparison.states
+    );
+    assert!(
+        states_reporting_r_unreadable > 0,
+        "流 {}：R 走读失败、c_见证 = 0 的状态上 CPU 看 journal 那一遍报读不出、报的根是 R {R_OF_THE_F2_FLOWS:?}",
+        flow.name
+    );
+    assert_eq!(
+        (states_falling_back_to_v, states_falling_back_to_v_minus),
+        (0, 0),
+        "流 {}：c_见证 = 0 不退，一个状态都不退到 V {V_OF_THE_F2_FLOWS:?} 或 V⁻ {V_MINUS_OF_THE_F2_FLOWS:?}",
+        flow.name
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
     );
 }
 
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index bc5914f..69d3b43 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -51,15 +51,15 @@ use crate::recovery::{
     effective_rollback_floor_rereading_the_newest_instance_table_once,
     effective_rollback_floor_under_the_newest_roots_table, highest_root_txg,
     highest_tree_identifier_watermark_in_the_ring, instance_table_chain_of_root,
-    instance_table_of_root, instance_table_page_pointers_as_far_as_readable, readable_roots,
-    readable_roots_rereading_ring_slots_known_to_hold_a_root_once,
+    instance_table_of_root, instance_table_page_pointers_as_far_as_readable,
+    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to,
+    readable_roots, readable_roots_rereading_ring_slots_known_to_hold_a_root_once,
     readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once,
     rebuild_version_in_the_unit_area_starting_at, replay_journal,
     root_is_abandoned_by_the_instance_table, scan_journal,
     system_configuration_slots_of_one_device_read_once,
     system_configuration_witness_is_undecidable, tree_table_has_no_entries,
-    user_visible_tree_root_pointers, verified_system_configuration_slots,
-    version_witnessed_by_the_system_configuration_to_fall_back_to, BadRootRingSlotReading,
+    user_visible_tree_root_pointers, verified_system_configuration_slots, BadRootRingSlotReading,
     EffectiveFloorReadingStillUnreadableAfterOneReread, InstanceTableChain,
     InstanceTableOfTheNewestRootStillUnreadableAfterOneReread, JournalScanReport, PoolReader,
     RebuildVersionFailure, RebuiltVersion, RecoveryFailure,
@@ -6721,8 +6721,8 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     let own_record = record_standing_for_the_version(&records, &effective_root);
     let selected_version_journal_position =
         journal_position_of_the_selected_version(&effective_root, own_record.as_ref());
-    // 所选那一版建不出来（走读失败）、系统配置没见证它：退回见证到的那一版建上一版（F2 乙，用户 2026-10-07 定「三处都做」，与恢复、只读挂载
-    // 同一份判法 `recovery::version_witnessed_by_the_system_configuration_to_fall_back_to`）。退过去不重放：所选根与所选那一版都换成它，
+    // 所选那一版建不出来（走读失败）、系统配置没见证它：退到 F2 交回的那一版建上一版（F2 乙，用户 2026-10-07 定「三处都做」，与恢复、只读挂载
+    // 同一份判法 `recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`）。退过去不重放：所选根与所选那一版都换成它，
     // 扫环报告里施加的那一截清零（新实例写的上一个实例那一行 W 取 0，与落在一条根上、一条记录都没施加时相同）。
     // 退过去的那一版也建不出来，照报所选那一版的失败。之后的一切（F1 乙再判一次、取号之前逐盘核、写行、影子账与 C）都按退到的那一版。
     // 退到哪一版照 `F2_NEWEST`：[见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（判法在那个函数里）。
@@ -6736,8 +6736,8 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
                 previous,
             ),
             Err(failure_of_the_selected_version) => {
-                let Some(witnessed_version) =
-                    version_witnessed_by_the_system_configuration_to_fall_back_to(
+                let Some(version_fallen_back_to) =
+                    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
                         &*devices,
                         &system_configuration,
                         &chosen_root,
@@ -6747,29 +6747,30 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
                 else {
                     return Err(failure_of_the_selected_version);
                 };
-                let record_standing_for_the_witnessed_version =
-                    record_standing_for_the_version(&records, &witnessed_version);
-                let witnessed_version_journal_position = journal_position_of_the_selected_version(
-                    &witnessed_version,
-                    record_standing_for_the_witnessed_version.as_ref(),
-                );
+                let record_standing_for_the_version_fallen_back_to =
+                    record_standing_for_the_version(&records, &version_fallen_back_to);
+                let journal_position_of_the_version_fallen_back_to =
+                    journal_position_of_the_selected_version(
+                        &version_fallen_back_to,
+                        record_standing_for_the_version_fallen_back_to.as_ref(),
+                    );
                 match rebuild_previous_version(
                     devices,
                     unit_area_start,
-                    &witnessed_version,
-                    record_standing_for_the_witnessed_version,
+                    &version_fallen_back_to,
+                    record_standing_for_the_version_fallen_back_to,
                 ) {
                     Ok(previous) => {
                         // F1 乙按退到的那一版的水位再判一次（`F1_AFTER_F2`）：读阶段判 F1 时水位是退之前的所选那一版。
                         refuse_publishes_above_the_version_fallen_back_to_missing_records_whose_named_units_do_not_verify(
                             &*devices,
                             &records,
-                            &witnessed_version,
+                            &version_fallen_back_to,
                             &mut before_the_one_reread,
                         )?;
                         (
-                            witnessed_version,
-                            witnessed_version,
+                            version_fallen_back_to,
+                            version_fallen_back_to,
                             JournalScanReport {
                                 valid_records: journal.valid_records,
                                 above_water: journal.above_water,
@@ -6778,11 +6779,11 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
                                 verification_failed: journal.verification_failed,
                                 maximum_applied_transaction: 0,
                             },
-                            witnessed_version_journal_position,
+                            journal_position_of_the_version_fallen_back_to,
                             previous,
                         )
                     }
-                    Err(_failure_of_the_witnessed_version) => {
+                    Err(_failure_of_the_version_fallen_back_to) => {
                         return Err(failure_of_the_selected_version)
                     }
                 }
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index 6afdc37..3ecd9c6 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -44,11 +44,11 @@ use crate::records::{
 };
 use crate::recovery::{
     choose_root, choose_system_configuration_naming_devices_without_a_mountable_slot,
+    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to,
     read_mapped_tree_node_via_hint_then_central_mapping, read_mapped_tree_root,
     read_unit_via_locations, replay_journal, scan_journal,
-    tree_table_entries_each_kind_at_most_once,
-    version_witnessed_by_the_system_configuration_to_fall_back_to, JournalScanReport,
-    MappedTreeNodeClass, PoolReader, RecoveryFailure, TreeTableEntriesRefusal,
+    tree_table_entries_each_kind_at_most_once, JournalScanReport, MappedTreeNodeClass, PoolReader,
+    RecoveryFailure, TreeTableEntriesRefusal,
 };
 use crate::root_record::RootRecord;
 use crate::transaction::MultiLevelCodeTwoTree;
@@ -585,8 +585,8 @@ pub struct MountedReadOnly {
 }
 
 /// 只读挂载：择系统配置 → 择根 → 全环扫描 journal → 按前缀口径施加 → 沿施加之后的根打开挂载态。
-/// 一个字节都不写（并行线二「这条线不写盘」）。施加之后那一版打不开（树表 0 条除外）、系统配置没见证它时，退回见证到的那一版打开
-/// （F2 乙，[`version_witnessed_by_the_system_configuration_to_fall_back_to`]）：`chosen_root` 与 `effective_root` 都是退到的那一条，
+/// 一个字节都不写（并行线二「这条线不写盘」）。施加之后那一版打不开（树表 0 条除外）、系统配置没见证它时，退到 [见证到的那一版, 所选根) 里最新的那条根打开
+/// （F2 乙，[`newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`]）：`chosen_root` 与 `effective_root` 都是退到的那一条，
 /// `journal` 仍是对原来所选根扫环、施加那一趟的报告。
 ///
 /// **扫环只在这一步发生一次**：之后每一次 [`OpenFileForRead::read_at`] 都不碰 journal 环。
@@ -619,25 +619,25 @@ pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, Mount
                 OpenPoolForReadFailure::TreeTableHasNoEntries,
             ));
         }
-        // 所选那一版走读失败、系统配置没见证它：退回见证到的那一版（F2 乙，与恢复、可写挂载同一份判法，
-        // `recovery::version_witnessed_by_the_system_configuration_to_fall_back_to`），不重放、直接打开；打不开照报所选那一版的失败。
+        // 所选那一版走读失败、系统配置没见证它：退到 [见证到的那一版, 所选根) 里最新的那条根（F2 乙，与恢复、可写挂载同一份判法，
+        // `recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`），不重放、直接打开；打不开照报所选那一版的失败。
         Err(failure_of_the_selected_version) => {
-            let witnessed_version_opened =
-                version_witnessed_by_the_system_configuration_to_fall_back_to(
+            let version_fallen_back_to_opened =
+                newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
                     reader,
                     &system_configuration,
                     &chosen_root,
                     &effective_root,
                     &records,
                 )
-                .and_then(|witnessed_version| {
-                    open_pool_for_read(reader, &witnessed_version)
+                .and_then(|version_fallen_back_to| {
+                    open_pool_for_read(reader, &version_fallen_back_to)
                         .ok()
-                        .map(|mounted| (mounted, witnessed_version))
+                        .map(|mounted| (mounted, version_fallen_back_to))
                 });
-            match witnessed_version_opened {
-                Some((mounted, witnessed_version)) => {
-                    (mounted, witnessed_version, witnessed_version)
+            match version_fallen_back_to_opened {
+                Some((mounted, version_fallen_back_to)) => {
+                    (mounted, version_fallen_back_to, version_fallen_back_to)
                 }
                 None => return Err(MountReadOnlyFailure::Open(failure_of_the_selected_version)),
             }
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 8689d22..d4c033f 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -5404,13 +5404,14 @@ pub fn rebuild_version(
 }
 
 /// 所选那一版走读失败之后退回哪一版（F2 乙，用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md`
-/// 第三节第 7 条）：系统配置没见证过所选那一版时，退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（`F2_NEWEST`，
+/// 第三节第 7 条）：系统配置没见证过所选那一版时，退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根——交回的不一定是
+/// 见证到的那一版本身，是这一段里最新的那条（`F2_NEWEST`，
 /// 判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H4 那一行与第三节第 1 条，用户 2026-10-08 定「四形都修」）——
 /// 见证到的那一版 = 根环里 (checkpoint_txg, 实例代号) 与计数器 = c_见证 的那条记录相等的那一版；不退到比它更旧的根（`F2_EXACT`，
 /// 判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行与第三节第 1 条：那样已确认的那一版根槽读回全零时会退过它、
 /// 记录读不出时会退到被新实例抛弃了的根），也不停在它而跳过它之后一条已确认、读得出的根（P 那次轮换写的较新系统配置槽读回全零时 c_见证 落回 P 之前）。
 /// 所选根的实例表读不出时只认与见证到的那一版恰好相等的那条；c_见证 落在所选那次发布自己的某条记录上时那一段是空的、不退（所选那一版算见证过）。
-/// 见证过、见证判不出、c_见证 = 0（没有见证到的记录）、那条记录读不出、或那一段里没有读得出、不被抛弃的根时 `None`（照报原来的失败）。
+/// 见证过、见证判不出、c_见证 = 0（环里读得出计数器 0 的记录也一样）、那条记录读不出、或那一段里没有读得出、不被抛弃的根时 `None`（照报原来的失败）。
 /// 恢复（[`recover`]）、可写挂载读阶段之后建上一版那一步（`mount`）、只读挂载择根
 /// （`mounted_read::mount_read_only`）三处都调这一份，退过去之后不重放、直接走那一版。所选那一版是施加过记录、由记录重建的
 /// （`selected_version` 不是 `chosen_root`）时不退：条款写的是「所选根走读失败」，施加过记录的那一版走不通条款没写（函数体第一句）。
@@ -5425,7 +5426,7 @@ pub fn rebuild_version(
 ///
 /// 代价（判决同一条，用户认下）：已确认的一版在较新的系统配置槽与它的单元同时读回全零时，c_见证 落回它之前，读回的是上一版。
 #[must_use]
-pub fn version_witnessed_by_the_system_configuration_to_fall_back_to<
+pub fn newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to<
     Reader: PoolReader + ?Sized,
 >(
     reader: &Reader,
@@ -5468,6 +5469,11 @@ pub fn version_witnessed_by_the_system_configuration_to_fall_back_to<
         .map(|slot| slot.quantities.journal_tail)
         .max()
         .unwrap_or(0);
+    // c_见证 = 0 不退（D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」）：写者的计数器从 1 起
+    // （`journal::FIRST_JOURNAL_COUNTER`），环里读到计数器 0 的记录只能是坏盘或伪造的，不拿它当见证到的那一版。
+    if witnessed_journal_counter == 0 {
+        return None;
+    }
     let order_of_the_root_choice = |root: &RootRecord| (root.checkpoint_txg, root.instance);
     let last_record_counter_of_the_selected_version = records
         .values()
@@ -5496,7 +5502,7 @@ pub fn version_witnessed_by_the_system_configuration_to_fall_back_to<
     if is_witnessed {
         return None;
     }
-    // 计数器 = c_见证 的那条记录读不出就不退；c_见证 = 0 时也走这一支——计数器从 1 起（`journal::FIRST_JOURNAL_COUNTER`），环里没有计数器 0 的记录。
+    // 计数器 = c_见证 的那条记录读不出就不退。
     let witnessed_publish = record_at_the_witnessed_counter?;
     let chosen = order_of_the_root_choice(chosen_root);
     // 退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（`F2_NEWEST`）。c_见证 落在所选那次发布自己的某条记录上时
@@ -5526,7 +5532,7 @@ pub fn version_witnessed_by_the_system_configuration_to_fall_back_to<
 }
 
 /// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。所选那一版走读失败、系统配置没见证它时退到
-/// [见证到的那一版, 所选根) 里读得出、不被抛弃的最新一条根（[`version_witnessed_by_the_system_configuration_to_fall_back_to`]）：那一版走得通，`root` 与 `effective_root` 都报退到的那一条，
+/// [见证到的那一版, 所选根) 里读得出、不被抛弃的最新一条根（[`newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`]）：那一版走得通，`root` 与 `effective_root` 都报退到的那一条，
 /// `journal` 仍是对所选根扫环、施加那一趟的报告；走不通照报所选那一版的失败。不看 journal 的那一臂（`JournalPolicy::Ignore`）不退。
 #[must_use]
 pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {
@@ -5608,9 +5614,9 @@ pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport
         Ok(None) => (RecoveryOutcome::NoFile { root: root_key }, effective_root),
         // 走读失败才再扫一遍环拿记录给退回的判法（只读、不写盘；走读做成的恢复不多扫）。
         Err(failure) => {
-            let witnessed_version_walked = match policy {
+            let version_fallen_back_to_walked = match policy {
                 JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {
-                    version_witnessed_by_the_system_configuration_to_fall_back_to(
+                    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
                         reader,
                         &system_configuration,
                         &root,
@@ -5620,30 +5626,32 @@ pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport
                 }
                 JournalPolicy::Ignore => None,
             }
-            .and_then(|witnessed_version| {
+            .and_then(|version_fallen_back_to| {
                 walk_to_file_in_the_unit_area_starting_at(
                     reader,
                     unit_area_start,
-                    &witnessed_version,
+                    &version_fallen_back_to,
                     &mut mapping_fallbacks,
                 )
                 .ok()
-                .map(|content| (witnessed_version, content))
+                .map(|content| (version_fallen_back_to, content))
             });
-            match witnessed_version_walked {
-                Some((witnessed_version, content)) => {
-                    let witnessed_key =
-                        (witnessed_version.instance, witnessed_version.checkpoint_txg);
+            match version_fallen_back_to_walked {
+                Some((version_fallen_back_to, content)) => {
+                    let key_fallen_back_to = (
+                        version_fallen_back_to.instance,
+                        version_fallen_back_to.checkpoint_txg,
+                    );
                     let outcome = match content {
                         Some(content) => RecoveryOutcome::FileRead {
-                            root: witnessed_key,
+                            root: key_fallen_back_to,
                             content,
                         },
                         None => RecoveryOutcome::NoFile {
-                            root: witnessed_key,
+                            root: key_fallen_back_to,
                         },
                     };
-                    (outcome, witnessed_version)
+                    (outcome, version_fallen_back_to)
                 }
                 None => (
                     RecoveryOutcome::Failed {
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index ff7df04..49e3d49 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -32,10 +32,10 @@ use singlefs_core::records::{
 use singlefs_core::recovery::{
     choose_root, choose_system_configuration, chosen_system_configuration_slot_bytes,
     effective_rollback_floor, highest_root_instance, highest_root_txg, instance_table_of_root,
+    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to,
     readable_roots, recover, replay_journal, root_record_bytes_of_the_version, scan_journal,
-    tree_table_entry_bytes_of_root, verified_system_configuration_slots,
-    version_witnessed_by_the_system_configuration_to_fall_back_to, walk_to_file, JournalPolicy,
-    PoolReader, RecoveryOutcome, RecoveryReport,
+    tree_table_entry_bytes_of_root, verified_system_configuration_slots, walk_to_file,
+    JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
 };
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
 use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
@@ -133,7 +133,8 @@ pub struct RecoveryFieldsRead {
 /// 对 `reader` 跑一次只读恢复（看 journal），再读挂载透明要比的字段。`compared` 定拿哪一版来比（[`ComparedVersion`]）。
 ///
 /// 拿来比的那一版（与恢复报告里的 `effective_root` 那一版）按恢复同一套读法再算一遍：择根、重放；重放出来的那一版走读失败时，
-/// 照恢复同一份 F2 判法（`recovery::version_witnessed_by_the_system_configuration_to_fall_back_to`）退到系统配置见证到的那一版，
+/// 照恢复同一份 F2 判法（`recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`）退到
+/// [系统配置见证到的那一版, 所选根) 里最新的那条根，
 /// 那一版走得通就是它（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第三节第 1 条：F2 退了就比退到的那一版）。
 ///
 /// # Panics
@@ -221,7 +222,7 @@ pub fn read_recovery_fields(
     let recovered_root = if walks_to_a_file_or_no_file(&replayed_root) {
         replayed_root
     } else {
-        version_witnessed_by_the_system_configuration_to_fall_back_to(
+        newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
             reader,
             &system_configuration,
             &chosen,
@@ -234,7 +235,7 @@ pub fn read_recovery_fields(
     assert_eq!(
         Some((recovered_root.instance, recovered_root.checkpoint_txg)),
         fields.report.effective_root,
-        "同一份盘上再择一次根、再重放一次（走读失败时照 F2 退到见证到的那一版），要落到恢复报告里那一版"
+        "同一份盘上再择一次根、再重放一次（走读失败时照 F2 退到 [见证到的那一版, 所选根) 里最新的那条根），要落到恢复报告里那一版"
     );
     fields.compared_version = match compared {
         ComparedVersion::TheRecoveredVersion => Some(version_fields(
diff --git a/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs b/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs
index a170c26..a872aa3 100644
--- a/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs
+++ b/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs
@@ -281,11 +281,29 @@ impl RangesReadingBackAllZero {
     }
 }
 
-/// 读点名的几段读回全零（盘上的字节不动）；写到哪一段，那一段之后就照读。设备报读错不在这一层：故障模型是「读回全零」
-/// （`crates/singlefs-harness/src/history.rs` 注入的那一种）。
+/// 几块盘共用的一个开关：撤了之后读回全零那一层整个不起作用，盘上的字节照读。用例在可写挂载「重读一次」之前的钩子里撤
+/// （`mount::BeforeTheOneReread::CallTheTestOnlyHookFirst`），强制进入「重读读得出」那一支（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
+#[derive(Clone, Debug, Default)]
+pub struct ReadBackAllZeroLift(std::rc::Rc<std::cell::Cell<bool>>);
+
+impl ReadBackAllZeroLift {
+    /// 撤掉读故障：之后每块盘每一次读都照读盘上的字节。
+    pub fn lift(&self) {
+        self.0.set(true);
+    }
+
+    #[must_use]
+    pub fn is_lifted(&self) -> bool {
+        self.0.get()
+    }
+}
+
+/// 读点名的几段读回全零（盘上的字节不动）；写到哪一段，那一段之后就照读；`lift` 撤了之后哪一段都照读。设备报读错不在这一层：
+/// 故障模型是「读回全零」（`crates/singlefs-harness/src/history.rs` 注入的那一种）。
 pub struct DeviceReadingBackAllZeroOnRanges<Inner: BlockDevice> {
     pub inner: Inner,
     ranges: Vec<(u64, u64)>,
+    lift: ReadBackAllZeroLift,
 }
 
 fn overlaps(start: u64, length: u64, offset: u64, other_length: u64) -> bool {
@@ -299,6 +317,9 @@ impl<Inner: BlockDevice> BlockDevice for DeviceReadingBackAllZeroOnRanges<Inner>
         buffer: &mut [u8],
     ) -> Result<(), BlockDeviceError> {
         self.inner.read_at(offset, buffer)?;
+        if self.lift.is_lifted() {
+            return Ok(());
+        }
         let buffer_length = u64::try_from(buffer.len()).expect("读长装得进 u64");
         for (start, length) in &self.ranges {
             if overlaps(*start, *length, offset.0, buffer_length) {
@@ -354,6 +375,19 @@ pub fn devices_reading_back_all_zero_on(
 ) -> Vec<(
     DeviceIdentity,
     DeviceReadingBackAllZeroOnRanges<SparseBlockDevice>,
+)> {
+    devices_reading_back_all_zero_on_until_lifted(image, ranges, &ReadBackAllZeroLift::default())
+}
+
+/// 同 [`devices_reading_back_all_zero_on`]，几块盘共用 `lift` 这一个开关：调用方撤了它，`ranges` 那几段在每块盘上都不再读回全零。
+#[must_use]
+pub fn devices_reading_back_all_zero_on_until_lifted(
+    image: &MemoryPool,
+    ranges: &RangesReadingBackAllZero,
+    lift: &ReadBackAllZeroLift,
+) -> Vec<(
+    DeviceIdentity,
+    DeviceReadingBackAllZeroOnRanges<SparseBlockDevice>,
 )> {
     plain_devices_on(image)
         .into_iter()
@@ -363,6 +397,7 @@ pub fn devices_reading_back_all_zero_on(
                 DeviceReadingBackAllZeroOnRanges {
                     inner,
                     ranges: ranges.on_device(identity),
+                    lift: lift.clone(),
                 },
             )
         })
@@ -391,6 +426,7 @@ pub fn recorded_devices_reading_back_all_zero_on(
                         stream.clone(),
                     ),
                     ranges: ranges.on_device(identity),
+                    lift: ReadBackAllZeroLift::default(),
                 },
             )
         })
diff --git a/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs b/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs
index 8d551b0..7e8d4bb 100644
--- a/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs
+++ b/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs
@@ -1,6 +1,6 @@
 //! F2 乙（用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 7 条）：所选那一版走读失败、系统配置没见证它时，
 //! 退回系统配置见证到的那一版；恢复（`recovery::recover`）、可写挂载读阶段之后建上一版那一步、只读挂载（`mounted_read::mount_read_only`）三处同一份判法
-//! （`recovery::version_witnessed_by_the_system_configuration_to_fall_back_to`）。「见证过」照攻方 `strict`：所选那一版带末条标志的记录读得出 ⇒
+//! （`recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`）。「见证过」照攻方 `strict`：所选那一版带末条标志的记录读得出 ⇒
 //! 计数器 ≤ c_见证；读不出 ⇒ 拿计数器 = c_见证 的那条记录比 (txg, 实例)；两条都读不出当见证过、不退。退到 [见证到的那一版, 所选根) 里读得出、
 //! 不被所选根的实例表抛弃的最新一条根（`F2_NEWEST`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H4 那一行与第三节第 1 条），
 //! 所选根的实例表读不出时只认与见证到的那一版恰好相等的那条；不退到比见证到的那一版更旧的根（`F2_EXACT`，判决
@@ -18,6 +18,8 @@
 //! - 代码轮攻方（`research/prompts/m3-batch35-36-code-r1-opus-output.md` 第四节）H4：崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元与 P 那次轮换写的
 //!   较新系统配置槽读回全零，c_见证 落回 P 之前那一版；三处都落到已确认的 P，四组（两种 R × 覆盖写 3、5 次）各一条。B 形那一盘上 P 的末条读得出时
 //!   退到的根按 Row 的实例表滤（跳过被抛弃的 R），Row 的实例表读不出时只认恰好相等的 P。
+//! - c_见证 = 0（删除线第 41 批，D23 已定项 14「c_见证 = 0 … 都不退」）：设计里那一退的底上每个系统配置槽的 tail 改 0、环里伪造一条计数器 0 的 P 的记录，
+//!   三处都不退。
 mod common_unwitnessed_publish;
 
 use common_unwitnessed_publish::{
@@ -30,19 +32,21 @@ use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
 };
 use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::journal::{record_offset, JournalRecord, JournalRecordPlaceInPublish};
 use singlefs_core::mount::{
     mount_writable, mount_writable_with_test_only_switches, MountError, ShadowLedger,
     StillUnreadableAfterOneReread,
 };
 use singlefs_core::mounted_read::mount_read_only;
 use singlefs_core::recovery::{
-    choose_system_configuration, recover, JournalPolicy, RecoveryOutcome,
+    choose_system_configuration, recover, scan_journal, JournalPolicy, RecoveryOutcome,
 };
 use singlefs_core::root_record::RootRecord;
 use singlefs_core::system_configuration::SystemConfiguration;
-use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
+use singlefs_core::unit::unit_filesystem_identifier;
+use singlefs_format::{JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
 use singlefs_harness::memory_pool::{
-    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite,
+    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, WrittenContents,
 };
 use singlefs_harness::segments::StepKind;
 use singlefs_harness::SharedStream;
@@ -351,6 +355,116 @@ fn a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_z
     );
 }
 
+/// 伪造的那条记录写进环里哪个槽：环里读得出的最大计数器之后第几条记录的槽（离开 R 与它之前写过的槽，仍在环里）。
+const RECORD_SLOTS_PAST_THE_HIGHEST_COUNTER_FOR_THE_FORGED_RECORD: u64 = 8;
+
+/// `image` 上每块盘两个系统配置槽里自证过的那几份 journal tail 改成 0、重封（c_见证 恒为 0）；再把 `key` 那次发布带末条标志的记录照抄一份，
+/// 计数器改成 0、重封，每块盘写进环里一个空着的记录槽（写者的计数器从 1 起，这条只能是伪造的）。
+fn image_with_every_journal_tail_zero_and_a_counter_zero_copy_of_the_last_record_of(
+    image: &MemoryPool,
+    key: (InstanceGeneration, CheckpointTxg),
+) -> MemoryPool {
+    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
+    let slot_spacing_in_bytes =
+        u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
+    let slot_bytes =
+        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节");
+    let mut writes: Vec<RetainedWrite> = Vec::new();
+    for (device, sparse_device) in &image.devices {
+        for slot_offset in [0, slot_spacing_in_bytes] {
+            if let Ok(mut slot) = SystemConfiguration::parse_slot(
+                &sparse_device.read(DeviceOffsetInBytes(slot_offset), slot_bytes),
+            ) {
+                slot.quantities.journal_tail = 0;
+                writes.push(RetainedWrite {
+                    device: *device,
+                    kind: StepKind::SystemConfigurationSlot,
+                    is_force_unit_access: false,
+                    offset: DeviceOffsetInBytes(slot_offset),
+                    contents: WrittenContents::Bytes(slot.to_slot()),
+                });
+            }
+        }
+    }
+    let records = scan_journal(image, &configuration);
+    let mut forged_record = records
+        .values()
+        .find(|record| {
+            (record.instance, record.checkpoint_txg) == key
+                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
+        })
+        .expect("环里读得出那次发布带末条标志的记录")
+        .clone();
+    forged_record.counter = 0;
+    let highest_counter = records
+        .values()
+        .map(|record| record.counter)
+        .max()
+        .expect("环里有记录");
+    let forged_record_offset = record_offset(
+        highest_counter + RECORD_SLOTS_PAST_THE_HIGHEST_COUNTER_FOR_THE_FORGED_RECORD,
+        configuration.immutable.sizes.journal_ring_bytes,
+    );
+    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("记录 4096 字节");
+    let filesystem_identifier =
+        unit_filesystem_identifier(&configuration.immutable.filesystem_identifier);
+    for (device, sparse_device) in &image.devices {
+        assert!(
+            JournalRecord::parse(
+                &sparse_device.read(forged_record_offset, record_bytes),
+                filesystem_identifier,
+            )
+            .is_none(),
+            "伪造的记录写进的那一槽原来没有读得出的记录"
+        );
+        writes.push(RetainedWrite {
+            device: *device,
+            kind: StepKind::JournalRecord,
+            is_force_unit_access: false,
+            offset: forged_record_offset,
+            contents: WrittenContents::Bytes(forged_record.to_bytes()),
+        });
+    }
+    let mut forged_image = image.clone();
+    forged_image.apply_writes(&writes);
+    forged_image
+}
+
+/// c_见证 = 0 不退（D23（journal 的角色与格式） 已定项 14「那条记录读不出、c_见证 = 0、那条根读不出都不退」）：设计里那一退的底上（R 的根槽落了、
+/// 轮换没落，读的那一刻 R 的单元读回全零），每个系统配置槽的 journal tail 改成 0，环里另有一条 P 那次发布末条记录的副本、计数器改成 0。
+/// 计数器 = c_见证 = 0 的那条记录读得出、属于 P，但 c_见证 = 0 不拿它当见证到的那一版：三处都不退、留在所选的 R 上照报读不出，可写挂载在任何写之前拒。
+/// 判别力：去掉 c_见证 = 0 的守卫（变异「删除线第41批 F2 c_见证 = 0 不另设守卫（计数器 0 的伪造记录上退到 P）」）时三处都退到 P、这一条红。
+#[test]
+fn a_witnessed_counter_of_zero_does_not_fall_back_even_with_a_record_carrying_counter_zero_in_recovery_and_both_mounts(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let image = image_with_every_journal_tail_zero_and_a_counter_zero_copy_of_the_last_record_of(
+        &crash_image,
+        recorded.previous_key,
+    );
+    let configuration = choose_system_configuration(&image).expect("改过的镜像上择得到系统配置");
+    assert_eq!(
+        configuration.quantities.journal_tail, 0,
+        "择到的系统配置 journal tail 是 0"
+    );
+    assert!(
+        scan_journal(&image, &configuration)
+            .values()
+            .any(|record| record.counter == 0
+                && (record.instance, record.checkpoint_txg) == recorded.previous_key),
+        "环里读得出一条属于 P、计数器 0 的记录"
+    );
+    let ranges = RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
+    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
+        &recorded,
+        &image,
+        &ranges,
+        recorded.key,
+    );
+}
+
 /// 每块盘上较新的那个系统配置槽（两槽里自证过、槽世代号大的那一槽）：盘、偏移、长度。
 fn newer_system_configuration_slot_of_each_device(
     image: &MemoryPool,
diff --git a/crates/singlefs-harness/tests/writable_mount_rejudges_f1_after_falling_back_and_accepts_when_the_reread_clears_the_fault.rs b/crates/singlefs-harness/tests/writable_mount_rejudges_f1_after_falling_back_and_accepts_when_the_reread_clears_the_fault.rs
new file mode 100644
index 0000000..888b92a
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_rejudges_f1_after_falling_back_and_accepts_when_the_reread_clears_the_fault.rs
@@ -0,0 +1,76 @@
+//! F1 乙按 F2 退到的那一版再判一次（`F1_AFTER_F2`）之后的「重读一次」读得出的那一支：判出有一次缺记录、点名单元两份都验不过的发布，
+//! 重读之前读故障撤掉，重读时点名单元验得过 ⇒ 不拒，可写挂载照常做成（D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」，
+//! `.claude/rules/fs-design.md` 五条硬要求第 2 条：用重读之前的钩子撤掉读故障，强制进入这一支）。
+//!
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`）与 `writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version.rs`
+//! 第一条同一个盘面：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次三条记录的顺序写 R，崩在 R 的根槽落盘之后、系统配置轮换之前；
+//! 可写挂载里 R 的第 1 条记录与 R 的树表读回全零，R 的根槽读得出。不撤读故障时那一条在取号之前拒。
+mod common_unwitnessed_publish;
+
+use common_unwitnessed_publish::{
+    devices_reading_back_all_zero_on_until_lifted, record_one_publish_after_overwrites,
+    PublishRecordedAsR, RangesReadingBackAllZero, ReadBackAllZeroLift,
+};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::mount::{
+    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread,
+    ReadStageSettled, ShadowLedger,
+};
+
+const OVERWRITES_BEFORE_R: u64 = 5;
+
+/// R 被择、走读失败（树表读回全零），系统配置没见证 R，F2 退回 P；按 P 当水位，R 读得出的是第 2、3 条（序号不从 1 起），第 3 条点名的树表
+/// 两份都验不过 ⇒ 重读一次。重读之前的钩子撤掉读故障，重读时树表验得过：不拒，可写挂载建在 P 上做成；钩子只调了一次，读阶段没有重读
+/// （那一次重读是 `F1_AFTER_F2` 的）。判别力：`F1_AFTER_F2` 重读之前不调钩子（变异「删除线第41批 F1_AFTER_F2 重读之前不调钩子
+/// （读故障撤不掉、照拒）」）时重读照样验不过、在取号之前拒，这一条红。
+#[test]
+fn a_root_falling_back_with_its_first_record_and_tree_table_reading_back_zero_mounts_writable_when_the_reread_clears_the_fault(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
+    let tree_table_writes = recorded.unit_writes_at(&recorded.root().tree_table);
+    assert_eq!(tree_table_writes.len(), 2, "R 自己写了树表，两块盘各一份");
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&recorded.record_writes_of_the_record(1))
+        .with_the_writes(&tree_table_writes);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let lift = ReadBackAllZeroLift::default();
+    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
+    let lifting = lift.clone();
+    let mut hook_calls = 0u32;
+    let mut lift_before_the_reread = || {
+        hook_calls += 1;
+        lifting.lift();
+    };
+    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
+        &recorded.pool.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut lift_before_the_reread),
+    )
+    .expect(
+        "退回 P 之后判出 R 缺记录、点名的树表验不过，重读之前读故障撤掉、重读验得过：可写挂载做成",
+    );
+    assert!(lift.is_lifted(), "读故障在挂载途中撤掉了");
+    assert_eq!(hook_calls, 1, "钩子只在 F1_AFTER_F2 重读之前调了一次");
+    assert!(
+        matches!(
+            mounted.output.rereads.read_stage,
+            ReadStageSettled::OnTheFirstRead { .. }
+        ),
+        "读阶段第一遍就判定了、没有重读（那一次重读不是读阶段的）：{:?}",
+        mounted.output.rereads.read_stage
+    );
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "F2 退回 P，可写挂载建在 P 上"
+    );
+}
````

## 删除线第 41 批：mutations-append.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch41/a/patch/mutations-append.tsv）

````text
删除线第41批 F2 c_见证 = 0 不另设守卫（计数器 0 的伪造记录上三处都退到 P）	crates/singlefs-core/src/recovery.rs	    if witnessed_journal_counter == 0 {\n        return None;\n    }	    if false {\n        return None;\n    }	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_witnessed_counter_of_zero_does_not_fall_back_even_with_a_record_carrying_counter_zero_in_recovery_and_both_mounts	a_witnessed_counter_of_zero_does_not_fall_back_even_with_a_record_carrying_counter_zero_in_recovery_and_both_mounts
删除线第41批 GPU 恢复内核 F2 c_见证 = 0 不另设守卫（计数器 0 的伪造记录上 GPU 退到 V、CPU 报读不出）	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	    if (is_witnessed || is_zero64(witnessed_counter) || !has_witnessed_record) {	    if (is_witnessed || !has_witnessed_record) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_does_not_fall_back_even_to_the_version_of_a_record_carrying_counter_zero	the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_does_not_fall_back_even_to_the_version_of_a_record_carrying_counter_zero
删除线第41批 F1_AFTER_F2 重读之前不调钩子（读故障撤不掉、照拒）	crates/singlefs-core/src/mount.rs	    before_the_one_reread.before_rereading();\n    match publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(	    match publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(	-p singlefs-harness --test writable_mount_rejudges_f1_after_falling_back_and_accepts_when_the_reread_clears_the_fault -- a_root_falling_back_with_its_first_record_and_tree_table_reading_back_zero_mounts_writable_when_the_reread_clears_the_fault	a_root_falling_back_with_its_first_record_and_tree_table_reading_back_zero_mounts_writable_when_the_reread_clears_the_fault
````

## 删除线第 41 批：mutations-delete.txt（/home/fy5090/.local/share/singlefs/work/m3-batch41/a/patch/mutations-delete.txt）

````text
第39批 Z GPU 恢复内核 F2 对 c_见证 = 0 另设守卫、不找计数器 0 的记录（伪造的计数器 0 记录上 GPU 报读不出、CPU 落 V）
````

## 删除线第 42 批：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch42/a/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index c5f457a..5fb0768 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -11851,8 +11851,130 @@ fn insert_an_instance_table_row_in_front(bytes: &mut [u8], instance: u32, select
     );
 }
 
-/// 所选根的实例表抛弃了区间里最新的那条根：H4 的底上，实例 1 那张实例表（同一实例里不重写，V⁻、V、R 指着同一张）最前面插一行 (1, 3)、重封到根——
-/// 按 R 的实例表，实例 1 里 txg > 3 的根（V 与 R）在被抛弃的时间线上。R 走读失败、c_见证 落回 V⁻ 的状态上，F2_NEWEST 在 [V⁻, R) 里滤掉 V、落 V⁻。
+/// 实例表一片的字节数（码 3 打包记录单元，占两槽）。
+const INSTANCE_TABLE_PAGE_BYTES: u64 = 2 * SLOT_BYTES;
+
+/// 只让所选根 R 认一张抛弃 V 的实例表：V⁻、V、R 指着的是同一张单片实例表，它在基镜像里（流里没有写它的单元写，同一实例里不重写）。
+/// 照 R 的实例表指针第 0 条位置条目从基镜像读出它（整单元 CRC 与指针里的相同），最前面插一行 (V⁻ 的实例, V⁻ 的 txg)、重封，
+/// 写进两块盘基镜像里流从没写过、全零的 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]：实例表单元头只带身份 (0, 4, 片号, 0) 与诞生、写序，不带自己的落点，
+/// 换一槽照样自证得过（读者按位置条目读、只比整单元 CRC）。R 的根槽写与 R 那次发布的 journal 记录新根段里的实例表指针（写者照抄根记录里那一条）
+/// 两条位置条目的槽改指这一槽、整单元校验和换成新的，各自重封（记录的反向链一路传下去）。V⁻、V 的根记录与原来那张表一个字节都不动。
+fn point_the_chosen_root_alone_at_an_instance_table_abandoning_v(flow: &mut RecordedFlow) {
+    let root_record_pointer_field = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
+        ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES;
+    let instance_table_pointer_of = |recorded_flow: &RecordedFlow, root_write: usize| {
+        recorded_flow.writes[root_write]
+            .bytes()
+            .expect("根槽写带字节")[root_record_pointer_field.clone()]
+        .to_vec()
+    };
+    let root_of_r = root_write_of_txg(flow, R_OF_THE_F2_FLOWS.1);
+    let pointer_to_the_shared_table = instance_table_pointer_of(flow, root_of_r);
+    assert_eq!(
+        instance_table_pointer_of(flow, root_write_of_txg(flow, V_OF_THE_F2_FLOWS.1)),
+        pointer_to_the_shared_table,
+        "流 {}：V 与 R 的根记录指着同一张实例表",
+        flow.name
+    );
+    let (device_of_the_first_location, slot_of_the_shared_table) =
+        device_and_slot_of_a_pointer_location(
+            &pointer_to_the_shared_table,
+            POINTER_LOCATION_ENTRY_OFFSETS[0],
+        );
+    let shared_table_start = slot_of_the_shared_table * SLOT_BYTES;
+    let shared_table_end = shared_table_start + INSTANCE_TABLE_PAGE_BYTES;
+    assert!(
+        flow.writes.iter().all(|write| write.device.0
+            != device_of_the_first_location
+            || write.offset.0 + write.length_in_bytes() <= shared_table_start
+            || write.offset.0 >= shared_table_end),
+        "流 {}：R 指着的实例表（盘 {device_of_the_first_location} 槽 {slot_of_the_shared_table}）流里没写过，在基镜像里",
+        flow.name
+    );
+    let shared_table = singlefs_core::recovery::PoolReader::read(
+        &flow.base,
+        DeviceIdentity(device_of_the_first_location),
+        singlefs_core::address::DeviceOffsetInBytes(shared_table_start),
+        usize::try_from(INSTANCE_TABLE_PAGE_BYTES).expect("32768"),
+    )
+    .expect("基镜像里读得到 R 指着的实例表");
+    assert_eq!(
+        crc32c(&shared_table),
+        read_u32_at(
+            &pointer_to_the_shared_table,
+            POINTER_LOCATION_ENTRY_OFFSETS[0] + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET
+        ),
+        "流 {}：基镜像里那一槽的整单元 CRC 与 R 的实例表指针里的相同",
+        flow.name
+    );
+    assert!(
+        unit_class(&shared_table) == 3
+            && packed_record_type(&shared_table) == PACKED_RECORD_TYPE_INSTANCE_TABLE,
+        "流 {}：R 指着的是一片实例表（码 3、记录类型 4）",
+        flow.name
+    );
+    let mut table_of_r = shared_table;
+    insert_an_instance_table_row_in_front(
+        &mut table_of_r,
+        V_MINUS_OF_THE_F2_FLOWS.0,
+        V_MINUS_OF_THE_F2_FLOWS.1,
+    );
+    reseal_unit(&mut table_of_r);
+    for device in devices_of_an_instance_table_pointer(&pointer_to_the_shared_table) {
+        assert!(
+            device == DeviceIdentity(0) || device == DeviceIdentity(1),
+            "R 的实例表指针两条位置条目落在盘 0、盘 1 上（基镜像只在这两块盘上写）：盘 {}",
+            device.0
+        );
+    }
+    assert_nothing_in_the_flow_wrote_the_slot(flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
+    write_into_the_base_on_both_devices(flow, SLOT_NOTHING_IN_THE_FLOW_WROTE, &table_of_r);
+    let pointer_to_the_table_of_r = pointer_naming_a_page_at(
+        &pointer_to_the_shared_table,
+        SLOT_NOTHING_IN_THE_FLOW_WROTE,
+        crc32c(&table_of_r),
+    );
+    rewrite_root_record_and_reseal(flow, root_of_r, &|bytes| {
+        bytes[root_record_pointer_field.clone()].copy_from_slice(&pointer_to_the_table_of_r);
+    });
+    let journal_record_pointer_field = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
+        + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
+        ..JOURNAL_NEW_ROOT_SEGMENT_OFFSET
+            + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
+            + NODE_POINTER_BYTES;
+    let journal_records_of_r: Vec<usize> = (0..flow.writes.len())
+        .filter(|write_index| {
+            flow.writes[*write_index].kind == StepKind::JournalRecord
+                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
+                    read_u32_at(bytes, JOURNAL_RECORD_INSTANCE_OFFSET) == R_OF_THE_F2_FLOWS.0
+                        && read_u64_at(bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET)
+                            == R_OF_THE_F2_FLOWS.1
+                })
+        })
+        .collect();
+    assert!(
+        !journal_records_of_r.is_empty(),
+        "流 {}：R 那次发布写了 journal 记录",
+        flow.name
+    );
+    for record in journal_records_of_r {
+        assert_eq!(
+            &flow.writes[record].bytes().expect("记录写带字节")
+                [journal_record_pointer_field.clone()],
+            pointer_to_the_shared_table.as_slice(),
+            "流 {}：R 那次发布的记录新根段里的实例表指针与 R 的根记录原来那一条相同",
+            flow.name
+        );
+        rewrite_journal_record_and_reseal(flow, record, &|bytes| {
+            bytes[journal_record_pointer_field.clone()].copy_from_slice(&pointer_to_the_table_of_r);
+        });
+    }
+}
+
+/// 所选根的实例表抛弃了区间里最新的那条根：H4 的底上，只让 R 认一张最前面多一行 (1, 3) 的实例表
+/// （[`point_the_chosen_root_alone_at_an_instance_table_abandoning_v`]：改好的那一片写进基镜像里流从没写过的槽，R 的根记录与记录新根段改指它）——
+/// 按 R 的实例表，实例 1 里 txg > 3 的根（V 与 R）在被抛弃的时间线上；V⁻、V 认的仍是原来那张表，V 自己照样有效。
+/// R 走读失败、c_见证 落回 V⁻ 的状态上，F2_NEWEST 在 [V⁻, R) 里按 R 的实例表滤掉 V、落 V⁻。
 /// CPU 看 journal 那一遍至少一个状态退到 V⁻、一个状态都不退到 V，GPU 逐位相同。GPU 恢复内核读得出所选根的实例表而不按它滤时落 V，这条红。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons(
@@ -11861,17 +11983,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_ver
         "crash-judge-gpu-f2-newest-abandoned-by-the-chosen-instance-table",
         "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed-instance-table-abandons-v",
     );
-    let instance_table = |bytes: &[u8]| {
-        unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INSTANCE_TABLE
-    };
-    let copies = last_unit_copies(&flow, "实例 1 的实例表", &instance_table, false);
-    corrupt_units_and_reseal_up_to_the_root(&mut flow, &copies, &|bytes| {
-        insert_an_instance_table_row_in_front(
-            bytes,
-            V_MINUS_OF_THE_F2_FLOWS.0,
-            V_MINUS_OF_THE_F2_FLOWS.1,
-        );
-    });
+    point_the_chosen_root_alone_at_an_instance_table_abandoning_v(&mut flow);
     assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
         "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons",
         &flow,
````

## 删除线第 42 批：mutations-append.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch42/a/patch/mutations-append.tsv）

````text
删除线第42批 F2_NEWEST 不按所选根的实例表滤（只有 R 的实例表抛弃 V 的世界上 CPU 退到 V、不落 V⁻）	crates/singlefs-core/src/recovery.rs	            Some(table) => !root_is_abandoned_by_the_instance_table(root, table),	            Some(_table) => true,	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons	the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons
````

## 删除线第 43 批：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch43/a/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 68dd9c3..f747c83 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -544,6 +544,10 @@ pub struct CrashInjectionTally {
     /// （判官不列「没注入读故障时判据复算为真」那一类条件），只差 U6 那几条的照别的拒因不报红。
     pub remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify:
         u64,
+    /// 二次崩溃之后再挂被拒、拒因是 C 读一次没施加的发布的账读不出、读得出的记录不齐、映射也现算不成的（`C_MAPPING` 收窄）：判过的次数。
+    /// 与 F1 乙同样没有 C670 同形的那一条，只差 U6 那几条的照别的拒因不报红。
+    pub remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records:
+        u64,
     /// 每一截（崩溃镜像本身、挂载与一次发布之后、二次崩溃）判到已知红与新发现的状态数之和。
     pub crash_states_ending_known_red: BTreeMap<usize, u64>,
     pub crash_states_ending_new_finding: u64,
@@ -633,6 +637,7 @@ impl CrashInjectionTally {
             remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
             remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults,
             remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify,
+            remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records,
             crash_states_ending_known_red,
             crash_states_ending_new_finding,
             mount_transparency_judged,
@@ -757,6 +762,8 @@ impl CrashInjectionTally {
             remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults;
         self.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify +=
             remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify;
+        self.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records +=
+            remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records;
         for (form, count) in crash_states_ending_known_red {
             *self.crash_states_ending_known_red.entry(*form).or_insert(0) += count;
         }
@@ -979,6 +986,11 @@ impl CrashInjectionTally {
             "二次崩溃之后再挂被拒、拒因是记录有缺口时拒可写的（F1 乙）：{} 次",
             self.remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify
         );
+        let _ = writeln!(
+            text,
+            "二次崩溃之后再挂被拒、拒因是没施加的发布的账读不出而读得出的记录不齐的（C_MAPPING）：{} 次",
+            self.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records
+        );
         let _ = writeln!(
             text,
             "挂载透明：第一次崩溃镜像上判了 {:?}，二次崩溃镜像上按段判了 {:?}；没判的 {:?}；判红的字段 {:?}；某一样没判的 {:?}；把清扫删的行放回去再比的 {} 次",
@@ -3731,7 +3743,8 @@ fn judge_the_remount_refusal_after_a_second_crash_against_the_criteria_true_with
 /// 没注入读故障时判据复算为真的那一条（N-配置 的 C670：[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]；
 /// 另一拒的改法 G4：[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]；
 /// Q3：[`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）时
-/// 再各记一次不合法，交回拒因与全部没满足的条件（报红用）。拒因是「记录有缺口时拒可写」（F1 乙）的只记一次判过：判官对它不列那一类条件。
+/// 再各记一次不合法，交回拒因与全部没满足的条件（报红用）。拒因是「记录有缺口时拒可写」（F1 乙）或 C 读一次没施加的发布的账读不出而记录不齐
+/// （`C_MAPPING`）的只记一次判过：判官对它们不列那一类条件。
 /// 拒因是实例表读不出的、那一条不成立（注入过读故障、判据复算为假）、只差 U6 那几条的，交回 None。
 fn reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
     tally: &mut CrashInjectionTally,
@@ -3811,6 +3824,24 @@ fn reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_with
                 conditions_not_met,
             )
         }
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+                ..
+            },
+        ) => {
+            tally.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records += 1;
+            return None;
+        }
+        WritableMountRefusalVerdict::Illegal {
+            reason:
+                reason @ RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+                    ..
+                },
+            conditions_not_met,
+        } => {
+            tally.remount_refusals_for_an_account_of_an_unapplied_publish_with_incomplete_readable_records += 1;
+            (reason, conditions_not_met)
+        }
     };
     let mut criterion_true_without_read_faults = false;
     // 迭代上界是判官列出的条件数（至多五条）；跨轮携带的只有上面那一个布尔；没有提前出口。
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 69d3b43..b034edf 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -391,10 +391,16 @@ pub enum StillUnreadableAfterOneReread {
     },
     /// 影子账读一条被抛弃根的账（树表或分配记录树）读不出、从中央映射也现算不成，重读一次仍是这样：拒可写
     /// （C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算那一条的最后一支，用户 2026-09-28 定）。
-    /// 可写挂载的 C 读一次没施加的发布（水位之上那几条记录拼的根，或比所选那一版新的可读根）的账读不出、那次发布读得出的记录又不齐时
-    /// 照同一份办，仍不成也报这一个，`abandoned_root` 点名那次发布——这次挂载取号之后它就被新实例抛弃（`C_MAPPING` 收窄，
-    /// 判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第三节第 1 条）。
+    /// 可写挂载的 C 读一次没施加的发布那一拒不报这一个，报 [`Self::AccountOfAnUnappliedPublishWithIncompleteReadableRecords`]。
     AccountOfAnAbandonedRoot { abandoned_root: RollbackTarget },
+    /// 可写挂载的 C 读一次没施加的发布——水位之上那几条记录拼的根，或比所选那一版新、按最新那条根的实例表有效、读得出的根（F2 退回之后
+    /// 被退掉的那条）——的账（树表或分配记录树）读不出、那次发布读得出的记录又不齐（本次发布内序号缺一条，或没有带末条标志的那条），
+    /// 照被抛弃根那一份从中央映射现算也不成，重读一次仍不成，现行账里又没有 C694 的 B 那道凭据（`C_MAPPING` 收窄，判决
+    /// `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K2 W1 / W2 H1、K2 W2 H3 两行与第三节第 1 条）。
+    /// `unapplied_publish` 点名那次发布（那条较新可读根就是它自己那次发布）。与 C393 那一拒（[`Self::AccountOfAnAbandonedRoot`]）分开报：
+    /// 这一拒列在 D23（journal 的角色与格式） 已定项 14 那张单子上，C393 那一拒不在（用户 2026-10-08 定「乙：core 另起错误成员」）。
+    /// 在取号之前拒，盘上逐字节不变，只读挂载照常。
+    AccountOfAnUnappliedPublishWithIncompleteReadableRecords { unapplied_publish: RollbackTarget },
     /// 可写挂载读阶段：水位之上有一次没施加的发布 `publish`，读得出的那几条记录的「本次发布内序号」不是 1..N 连着或没有一条带末条标志，
     /// 而它们点名的单元又有一个验不过；在这次挂载的读缓存上重做读阶段一遍仍是这样（F1 乙，用户 2026-10-07 定「缺记录就拒可写」，
     /// 判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 6 条）。缺的那条记录点名的单元挂载时谁都不知道是哪几个，
@@ -1630,9 +1636,8 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
 }
 
 /// 一条被抛弃根的账（树表或分配记录树）这一遍读不出之后，它引用的落点从哪来（C393 取 (b)-从映射现算，用户 2026-09-28 定，被攻过零轮；
-/// C694 的 B，用户 2026-10-07 定）：从中央映射现算；现算也不成就重读一次（重读之前调钩子），账或现算有一样成就用它；仍都不成时，
-/// 现行账里有那条根之后的记录罩住它的树表槽（与映射根槽）就当它什么都不引用（[`the_current_account_records_later_units_over_the_account_slots_of`]），
-/// 没有就拒。
+/// C694 的 B，用户 2026-10-07 定）：[`placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread`]，
+/// 那一份交 `None` 就拒。
 ///
 /// # Errors
 /// 账与映射重读一次仍都不成、现行账里又没有那道凭据 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`（在任何写之前）。
@@ -1647,10 +1652,46 @@ fn placements_of_an_abandoned_root_whose_account_is_unreadable<Device: BlockDevi
     current_records: &[AllocationRecord],
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<Vec<(DeviceIdentity, SlotNumber, u64)>, MountError> {
+    placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread(
+        devices,
+        unit_area_start,
+        root,
+        current_records,
+        before_the_one_reread,
+    )
+    .ok_or_else(|| {
+        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
+            StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
+                abandoned_root: RollbackTarget {
+                    instance: root.instance,
+                    checkpoint_txg: root.checkpoint_txg,
+                },
+            },
+        ))
+    })
+}
+
+/// 一条根的账（树表或分配记录树）这一遍读不出之后，它引用的落点从哪来，被抛弃根（[`placements_of_an_abandoned_root_whose_account_is_unreadable`]）
+/// 与可写挂载的 C 读一次没施加的发布（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）共用这一份、交 `None` 时各报各的拒因：
+/// 从中央映射现算；现算也不成就重读一次（重读之前调钩子），账或现算有一样成就用它；仍都不成时，现行账里有那条根之后的记录罩住它的树表槽
+/// （与映射根槽）就当它什么都不引用（[`the_current_account_records_later_units_over_the_account_slots_of`]），没有就交 `None`。
+#[allow(
+    clippy::ptr_arg,
+    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
+)]
+fn placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread<
+    Device: BlockDevice,
+>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    unit_area_start: UnitAreaStart,
+    root: &RootRecord,
+    current_records: &[AllocationRecord],
+    before_the_one_reread: &mut BeforeTheOneReread<'_>,
+) -> Option<Vec<(DeviceIdentity, SlotNumber, u64)>> {
     if let Some(computed) =
         placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)
     {
-        return Ok(computed);
+        return Some(computed);
     }
     before_the_one_reread.before_rereading();
     if let Some(placements) =
@@ -1658,19 +1699,12 @@ fn placements_of_an_abandoned_root_whose_account_is_unreadable<Device: BlockDevi
             placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)
         })
     {
-        return Ok(placements);
+        return Some(placements);
     }
     if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {
-        return Ok(Vec::new());
+        return Some(Vec::new());
     }
-    Err(MountError::NewerStateStillUnreadableAfterOneReread(
-        Box::new(StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
-            abandoned_root: RollbackTarget {
-                instance: root.instance,
-                checkpoint_txg: root.checkpoint_txg,
-            },
-        }),
-    ))
+    None
 }
 
 /// C694 的 B 那道凭据（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 2 条）：被抛弃根 `root` 的树表槽、与它的中央映射树根槽
@@ -1786,7 +1820,8 @@ fn placements_named_by_unapplied_journal_records<Device: BlockDevice>(
 /// F1 乙不拒，缺的那条记录单独点名的单元又不在别的记录里；账里是那次发布引用的全部落点。水位与 C 同一个（[`placements_named_by_unapplied_journal_records`]）。
 ///
 /// # Errors
-/// 那一支现算也不成、重读一次仍不成、现行账里又没有 C694 的 B 那道凭据 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`
+/// 那一支现算也不成、重读一次仍不成、现行账里又没有 C694 的 B 那道凭据 ⇒
+/// `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`
 /// （点名那次发布的 (实例代号, checkpoint_txg)），在任何写之前。
 #[allow(
     clippy::ptr_arg,
@@ -1945,12 +1980,14 @@ fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
 
 /// 水位之上一次没施加的发布（或比所选那一版新的可读根）的账读不出、那次发布读得出的记录又不齐时，它引用的落点照被抛弃根那一份办
 /// （`C_MAPPING` 收窄，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1、H3 两行与第三节第 1 条）：
-/// 从中央映射现算、重读一次、看 C694 的 B 那道凭据（[`placements_of_an_abandoned_root_whose_account_is_unreadable`] 原样）。
+/// 从中央映射现算、重读一次、看 C694 的 B 那道凭据（[`placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread`]，
+/// 与被抛弃根那一份共用）。
 /// 记录齐的那几次发布不走这里（调用方判）：那时账读不出交 F1 乙，照被抛弃根那一份办会在单元全读不出、映射也现算不成的正常盘面上误拒。
 ///
 /// # Errors
-/// 映射现算不成、重读一次仍不成、凭据也没有 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`，`abandoned_root` 点名
-/// `root_of_the_publish` 的 (实例代号, checkpoint_txg)——这次挂载取号之后它就被新实例抛弃，拒因照被抛弃根那一类写；在任何写之前。
+/// 映射现算不成、重读一次仍不成、凭据也没有 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`，
+/// `unapplied_publish` 点名 `root_of_the_publish` 的 (实例代号, checkpoint_txg)；与 C393 那一拒分开报（用户 2026-10-08 定「乙：core 另起错误成员」），
+/// 在任何写之前。
 #[allow(
     clippy::ptr_arg,
     reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
@@ -1962,13 +1999,23 @@ fn placements_of_an_unapplied_publish_whose_account_is_unreadable<Device: BlockD
     current_records: &[AllocationRecord],
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<Vec<(DeviceIdentity, SlotNumber, u64)>, MountError> {
-    placements_of_an_abandoned_root_whose_account_is_unreadable(
+    placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread(
         devices,
         unit_area_start,
         root_of_the_publish,
         current_records,
         before_the_one_reread,
     )
+    .ok_or_else(|| {
+        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
+            StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+                unapplied_publish: RollbackTarget {
+                    instance: root_of_the_publish.instance,
+                    checkpoint_txg: root_of_the_publish.checkpoint_txg,
+                },
+            },
+        ))
+    })
 }
 
 /// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮）：被抛弃根的树表或分配记录树读不出时，
@@ -2344,7 +2391,8 @@ fn txgs_ring_slots_bad_at_mount_can_hold(
 /// 读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有 ⇒
 /// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2、R1 按槽并两遍）；
 /// C 读一次没施加的发布或一条较新可读根的账读不出、那次发布读得出的记录不齐、从中央映射也现算不成、重读一次仍不成
-/// （[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`。
+/// （[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）⇒
+/// `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`。
 /// 都在取号之前，盘上逐字节不变。
 #[allow(
     clippy::ptr_arg,
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index 3224776..3150c2b 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -3103,6 +3103,9 @@ fn still_unreadable_after_one_reread_member(
             "RootRingSlotKnownToHoldARoot"
         }
         StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => "AccountOfAnAbandonedRoot",
+        StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+            ..
+        } => "AccountOfAnUnappliedPublishWithIncompleteReadableRecords",
         StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
             "PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify"
         }
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index c71ab6e..c123012 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -838,6 +838,8 @@ pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReaso
                 | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                 | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
                 | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. }
+                // 没施加的发布的账读不出、读得出的记录不齐、映射也现算不成（C_MAPPING）：随机历史不藏 journal 记录，模型没有这一条。
+                | StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { .. }
                 // 水位之上一次缺记录、读得出的记录点名的单元又验不过的发布（F1 乙）：随机历史不藏 journal 记录，模型没有这一条。
                 | StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
                     ObservedRefusalReason::Unexplained
@@ -947,6 +949,7 @@ pub fn root_ring_slot_still_bad_after_one_reread_of_rollback_error(
                 | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. }
                 | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                 | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. }
+                | StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { .. }
                 | StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => None,
             }
         }
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index 49e3d49..0c17252 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -20,14 +20,16 @@ use std::collections::BTreeMap;
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
 };
+use singlefs_core::bytes::ByteReader;
 use singlefs_core::checksum::crc32_castagnoli;
 use singlefs_core::instance_table::InstanceRow;
 use singlefs_core::journal::{JournalRecord, JournalRecordPlaceInPublish, NamedUnit};
 use singlefs_core::mount::{MountError, MountOutput, RaisedFloor, StillUnreadableAfterOneReread};
+use singlefs_core::pointer::NodePointer;
 use singlefs_core::records::{
-    TreeTableEntry, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT,
-    TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS,
-    TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
+    TreeTableEntry, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES,
+    TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST,
+    TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
 };
 use singlefs_core::recovery::{
     choose_root, choose_system_configuration, chosen_system_configuration_slot_bytes,
@@ -40,7 +42,9 @@ use singlefs_core::recovery::{
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
 use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
 use singlefs_core::transaction::TransactionOutput;
-use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
+use singlefs_core::unit::{
+    parse_index_node, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
+};
 use singlefs_format::{
     DATA_UNIT_BYTES, NODE_BYTES, SYSTEM_CONFIGURATION_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
     SYSTEM_CONFIGURATION_SLOT_BYTES,
@@ -2258,6 +2262,13 @@ pub enum RefusalReasonOnTheD23List {
     /// 水位之上一次没施加的发布，读得出的那几条记录的本次发布内序号不是 1..N 连着或缺末条，而它们点名的单元里有一个两份副本都验不过，
     /// 重读一次仍这样（`StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`）。
     PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
+    /// 可写挂载的 C 读一次没施加的发布（水位之上那几条记录拼的根，或比所选那一版新、按实例表有效、读得出的根）的账读不出、那次发布读得出的记录
+    /// 不齐、从中央映射也现算不成，重读一次仍这样（`C_MAPPING` 收窄，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md`
+    /// 第二节 K2 W1 / W2 H1、K2 W2 H3 两行；`StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords`，
+    /// 用户 2026-10-08 定「乙：core 另起错误成员」、列进单子）。`unapplied_publish` 是那一拒点名的 (实例代号, checkpoint_txg)，复算按它找那次发布。
+    AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+        unapplied_publish: (InstanceGeneration, CheckpointTxg),
+    },
 }
 
 /// 拒可写的理由在不在 D23 已定项 14 那张单子上。别的理由（单元区墙、坏盘上才有的那几种……）照崩溃注入第二截今天的写法判。
@@ -2275,6 +2286,16 @@ pub fn refusal_reason_on_the_d23_list(error: &MountError) -> Option<RefusalReaso
                 StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
                     Some(RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify)
                 }
+                StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+                    unapplied_publish,
+                } => Some(
+                    RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+                        unapplied_publish: (
+                            unapplied_publish.instance,
+                            unapplied_publish.checkpoint_txg,
+                        ),
+                    },
+                ),
                 // 挂着时抬 F、管理员回退那几处读坏，与影子账读一条被抛弃根的账读不出（C393）：不在 D23 已定项 14 那张单子上。
                 StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                 | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
@@ -2679,6 +2700,183 @@ fn named_unit_has_a_copy_that_verifies_on_the_crash_image(
     })
 }
 
+/// 「C 读一次没施加的发布的账读不出、读得出的记录不齐、从中央映射也现算不成」（`C_MAPPING` 收窄）那一拒在崩溃镜像上复算，不调实现的
+/// `mount::placements_in_the_accounts_of_unapplied_publishes` 与 `mount::placements_in_the_accounts_of_newer_readable_roots`。
+/// 判据由主 agent 定（2026-10-08，被攻过零轮），三样都成立即为真：
+/// 1. 那一拒点名的 `unapplied_publish` 是水位（`selected_version`，R1 落到的那一版，按实例代号为主比）之上没施加的发布——崩溃镜像上读得出它的记录；
+///    或是根环里比所选那一版新（txg 为主、实例代号破平局）、按最新那条根的实例表有效、读得出的根。被抛弃照 D23（journal 的角色与格式） 已定项 14
+///    候选集那一格另写一份（表里有那条根的实例的行 (i, Ti, Wi) 且它的 txg > Ti）；最新那条根的实例表读不出时不按表滤，与
+///    [`no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image`] 同一取法。
+/// 2. 那次发布在崩溃镜像上读得出的记录不齐：本次发布内序号排好之后不是从 1 起逐个加一，或没有一条带末条标志（一条都读不出也算没有末条）。
+/// 3. 它的账在崩溃镜像上读不出：按新根段里的树表指针找到的账根两份副本都验不过、或账根读不到（[`account_unreadable_on_the_crash_image`]）——
+///    读得出的每条记录的新根段，与第 1 条里那条较新的根的根记录，各找一次，每一份都读不出才算。
+///
+/// 这一格只判必要条件：实现拒之前还要「从中央映射现算不成」，那一半崩溃镜像上不复算——复算为真不说明实现非拒不可，只说拒的理由在镜像上站得住。
+/// 账只看到账根那一个节点：实现读账还读分配记录树根以下的节点，只有那几样读不出时实现照拒、这里复算为假（判不合法）。
+/// 崩溃镜像上没有瞬时读错，重读一次读出的与这一遍相同，判据只算一遍。择不到系统配置按真（与这一族别的复算同向）；没有所选那一版时水位无从取，交回假。
+fn account_of_an_unapplied_publish_with_incomplete_readable_records_on_the_crash_image(
+    crash_image: &dyn PoolReader,
+    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
+    unapplied_publish: (InstanceGeneration, CheckpointTxg),
+) -> bool {
+    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
+        return true;
+    };
+    let Some(selected_version) = selected_version else {
+        return false;
+    };
+    let readable_records_of_the_publish: Vec<JournalRecord> =
+        scan_journal(crash_image, &system_configuration)
+            .into_values()
+            .filter(|record| (record.instance, record.checkpoint_txg) == unapplied_publish)
+            .collect();
+    let is_an_unapplied_publish_above_the_water_mark =
+        unapplied_publish > selected_version && !readable_records_of_the_publish.is_empty();
+    let newer_valid_readable_root = newer_valid_readable_root_on_the_crash_image(
+        crash_image,
+        &system_configuration,
+        selected_version,
+        unapplied_publish,
+    );
+    if !is_an_unapplied_publish_above_the_water_mark && newer_valid_readable_root.is_none() {
+        return false;
+    }
+    let mut ordinals: Vec<u32> = readable_records_of_the_publish
+        .iter()
+        .map(|record| record.ordinal_within_publish.0)
+        .collect();
+    ordinals.sort_unstable();
+    let ordinals_run_from_one_without_a_gap = ordinals
+        .iter()
+        .zip(1u32..)
+        .all(|(ordinal, expected_ordinal)| *ordinal == expected_ordinal);
+    let carries_the_last_record_of_the_publish =
+        readable_records_of_the_publish.iter().any(|record| {
+            record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
+        });
+    if ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish {
+        return false;
+    }
+    readable_records_of_the_publish
+        .iter()
+        .map(|record| {
+            (
+                record.new_tree_table,
+                record.new_allocation_record_tree_root,
+            )
+        })
+        .chain(
+            newer_valid_readable_root
+                .iter()
+                .map(|root| (root.tree_table, root.allocation_record_tree_root)),
+        )
+        .all(
+            |(tree_table, allocation_record_tree_root_in_the_new_root_segment)| {
+                account_unreadable_on_the_crash_image(
+                    crash_image,
+                    &tree_table,
+                    &allocation_record_tree_root_in_the_new_root_segment,
+                )
+            },
+        )
+}
+
+/// 一版的账在崩溃镜像上读不出：账根两份副本都验不过（[`node_has_a_copy_that_verifies_on_the_crash_image`] 为假），或连账根在哪都读不到。
+/// 账根按那一版的树表找（`mount::placements_referenced_by_root` 读账的同一条路，这里另写一份、不调它）：树表两份都验不过、解不开 ⇒
+/// 读不到账根，算读不出；树表 0 条（没写过文件的一版）⇒ 账根是新根段里那条分配记录树根（空指针算读得出：mkfs 的第 0 代与照抄它的暖机根没有这棵树）；
+/// 树表有条目 ⇒ 账根是 `TREE_KIND_ALLOCATION` 那一条的根指针（没有那一条算读得出：账里一条记录都没有）。
+/// 主 agent 定的判据写的是「新根段里的分配记录树根」；带文件的一版，根记录与新根段里那一条恒是空指针（它的分配记录树住树表条目，
+/// D8（核心索引结构） 已定项 8；`transaction.rs` 发布带文件的一版时那一项写 `NodePointer::empty_root()`），不是它的账——
+/// 只看那一条，带文件的一版永远判不成读不出。这里按「新根段里的树表指针找到的账根」读，交主 agent 定（第 43 批报告）。
+/// 只看账根那一个节点：账根以下的节点读不出时实现照拒、这里判读得出。
+fn account_unreadable_on_the_crash_image(
+    crash_image: &dyn PoolReader,
+    tree_table: &NodePointer,
+    allocation_record_tree_root_in_the_new_root_segment: &NodePointer,
+) -> bool {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("节点字节数 16384 装得进 usize");
+    let Some(tree_table_bytes) = tree_table.locations.iter().find_map(|location| {
+        crash_image
+            .read(
+                location.device,
+                location.slot.to_device_offset(),
+                node_bytes,
+            )
+            .filter(|bytes| crc32_castagnoli(bytes) == location.unit_checksum)
+    }) else {
+        return true;
+    };
+    let Ok(tree_table_node) = parse_index_node(&tree_table_bytes) else {
+        return true;
+    };
+    let account_root = if tree_table_node.entries.is_empty() {
+        *allocation_record_tree_root_in_the_new_root_segment
+    } else {
+        match TreeTableEntryRootPointersByKind::of_the_entries(
+            &tree_table_node.entries,
+            &[TREE_KIND_ALLOCATION],
+        )
+        .root_pointer_of_each_kind
+        .get(&TREE_KIND_ALLOCATION)
+        {
+            Some(root_pointer_bytes) => {
+                NodePointer::read_from(&mut ByteReader::at(root_pointer_bytes, 0))
+            }
+            None => return false,
+        }
+    };
+    account_root != NodePointer::empty_root()
+        && !node_has_a_copy_that_verifies_on_the_crash_image(crash_image, &account_root)
+}
+
+/// 根环里那条 (实例代号, checkpoint_txg) 等于 `publish` 的根，它比所选那一版新（择根的序：txg 为主、实例代号破平局）、读得出、按最新那条根
+/// （`choose_root`）的实例表不被抛弃（表里有它的实例的行 (i, Ti, Wi) 且它的 txg > Ti 才算被抛弃，不调实现的
+/// `recovery::root_is_abandoned_by_the_instance_table`；那张表读不出时不按表滤）：交回那条根，没有交 `None`。
+fn newer_valid_readable_root_on_the_crash_image(
+    crash_image: &dyn PoolReader,
+    system_configuration: &SystemConfiguration,
+    selected_version: (InstanceGeneration, CheckpointTxg),
+    publish: (InstanceGeneration, CheckpointTxg),
+) -> Option<RootRecord> {
+    let table_of_the_newest_root = choose_root(crash_image, system_configuration)
+        .and_then(|newest| instance_table_of_root(crash_image, &newest));
+    let immutable = &system_configuration.immutable;
+    readable_roots(
+        crash_image,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    )
+    .into_iter()
+    .find(|root| {
+        let is_abandoned = table_of_the_newest_root.as_ref().is_some_and(|table| {
+            table.rows.iter().any(|row| {
+                row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg
+            })
+        });
+        (root.instance, root.checkpoint_txg) == publish
+            && (root.checkpoint_txg, root.instance) > (selected_version.1, selected_version.0)
+            && !is_abandoned
+    })
+}
+
+/// 一条码 2 / 码 3 指针指的节点在崩溃镜像上至少有一份副本验得过：某一条位置条目指的那一份读得出，16 KiB 读回的整单元 CRC-32C 等于条目里的。
+fn node_has_a_copy_that_verifies_on_the_crash_image(
+    crash_image: &dyn PoolReader,
+    pointer: &NodePointer,
+) -> bool {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("节点字节数 16384 装得进 usize");
+    pointer.locations.iter().any(|location| {
+        crash_image
+            .read(
+                location.device,
+                location.slot.to_device_offset(),
+                node_bytes,
+            )
+            .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
+    })
+}
+
 /// 拒之后盘上逐字节同不同（调用方比拒之前与拒之后的整份镜像）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum DiskAfterTheRefusal {
@@ -2888,7 +3086,10 @@ pub fn judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_und
         )
         | (
             RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable
-            | RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
+            | RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify
+            | RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+                ..
+            },
             ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,
         ) => {}
     }
@@ -2910,9 +3111,13 @@ pub fn judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_und
         }
         // 「记录有缺口时拒可写」那一拒没有这一条宽限：被拒的那次挂载在读故障下跑、判据在崩溃镜像上复算为假的照判不合法
         // （D23 已定项 14 那一段只写判据、没写读故障下这一拒怎么判；取不放宽的一边：复算不出的报红，不藏）。
+        // `C_MAPPING` 那一拒同一个取法（主 agent 2026-10-08 定，被攻过零轮）。
         RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
         | RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable
-        | RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify => {}
+        | RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify
+        | RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+            ..
+        } => {}
     }
     verdict_of_a_refusal_on_the_d23_list(reason, conditions_not_met)
 }
@@ -2946,6 +3151,13 @@ fn conditions_not_met_by_a_refusal_on_the_d23_list(
                 selected_version,
             )
         }
+        RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+            unapplied_publish,
+        } => account_of_an_unapplied_publish_with_incomplete_readable_records_on_the_crash_image(
+            crash_image,
+            selected_version,
+            unapplied_publish,
+        ),
     };
     let mut conditions_not_met = Vec::new();
     if !criterion_holds_on_the_crash_image {
diff --git a/crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs b/crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs
index 6f846a4..7cac344 100644
--- a/crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs
+++ b/crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs
@@ -3,7 +3,7 @@
 //! - `C_NEWER_ROOTS`：C 并上根环里比所选那一版新、按实例表有效、读得出的根自己的账（照影子账同一份读法）。F2 退回之后被退掉的那条根 R
 //!   就是这样：影子账只管被抛弃的根、不管它，它的记录又全读不出时，C 那两份（记录点名的、记录拼根读的账）都是空的。
 //! - `C_MAPPING`（收窄）：C 读一次没施加的发布（记录拼的根，或上面那条较新的根）的账读不出时，只在那次发布读得出的记录不齐（序号缺一条、
-//!   缺末条）时照被抛弃根那一份从中央映射现算账里的落点；仍算不出就在取号之前拒可写（拒因照被抛弃根那一类）。
+//!   缺末条）时照被抛弃根那一份从中央映射现算账里的落点；仍算不出就在取号之前拒可写（拒因另起一个成员，与 C393 那一拒分开报）。
 //!   记录齐的不走这一支（照被抛弃根那一份不收窄会在 `ur` 那几形上误拒，攻方量到入库 7 条）。较新的根那次发布一条记录都读不出时
 //!   条款没写：映射现算得出就并进来，现算不成不拒（入库 `urJ` 那一形就是这样，拒了它就再也做不成可写挂载）。
 //!
@@ -198,7 +198,7 @@ fn a_newer_readable_root_whose_last_record_and_tree_table_read_back_zero_keeps_i
 }
 
 /// `C_MAPPING` 仍算不出就拒：同「根藏的那一半」，再加 R 的中央映射树根读回全零——账读不出、记录不齐、映射也现算不成，重读一次仍是这样，
-/// 现行账（P 的）里也没有 C694 的 B 那道凭据（R 比 P 新），在取号之前拒可写，拒因照被抛弃根那一类、点名 R。
+/// 现行账（P 的）里也没有 C694 的 B 那道凭据（R 比 P 新），在取号之前拒可写，拒因是 `AccountOfAnUnappliedPublishWithIncompleteReadableRecords`、点名 R。
 /// 判别力：现算不成也不拒（变异「删除线第38批 C_MAPPING 现算不成也不拒」）时可写挂载做成、这一条红。
 #[test]
 fn a_publish_whose_account_and_mapping_read_back_zero_while_its_last_record_is_missing_refuses_the_writable_mount_before_any_write(
@@ -236,12 +236,12 @@ fn a_publish_whose_account_and_mapping_read_back_zero_while_its_last_record_is_m
             MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                 if matches!(
                     **still_unreadable,
-                    StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
-                        abandoned_root: RollbackTarget { instance, checkpoint_txg }
+                    StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+                        unapplied_publish: RollbackTarget { instance, checkpoint_txg }
                     } if (instance, checkpoint_txg) == recorded.key
                 )
         ),
-        "拒因照被抛弃根那一类、点名 R：{refusal:?}"
+        "拒因是 C 的映射那一支另起的成员、点名 R：{refusal:?}"
     );
 }
 
@@ -292,8 +292,8 @@ fn assert_every_unit_reading_back_zero_with_every_record_keeps_the_units_of_r(
                         MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                             if matches!(
                                 **still_unreadable,
-                                StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
-                                    abandoned_root: RollbackTarget { instance, checkpoint_txg }
+                                StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+                                    unapplied_publish: RollbackTarget { instance, checkpoint_txg }
                                 } if (instance, checkpoint_txg) == recorded.key
                             )
                     ),
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_for_an_unreadable_account_with_incomplete_records_judged_on_the_d23_list.rs b/crates/singlefs-harness/tests/writable_mount_refusal_for_an_unreadable_account_with_incomplete_records_judged_on_the_d23_list.rs
new file mode 100644
index 0000000..7969e80
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_for_an_unreadable_account_with_incomplete_records_judged_on_the_d23_list.rs
@@ -0,0 +1,204 @@
+//! 可写挂载的 C 读一次没施加的发布、账读不出、读得出的记录不齐、从中央映射也现算不成的那一拒（`C_MAPPING` 收窄，判决
+//! `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K2 W1 / W2 H1、K2 W2 H3 两行）另起错误成员
+//! （`StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords`，用户 2026-10-08 定「乙：core 另起错误成员」），
+//! 列进 D23（journal 的角色与格式） 已定项 14 那张单子（`RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords`）；
+//! 判法在崩溃镜像上复算判据（主 agent 定，被攻过零轮）：点名的是水位之上没施加的发布（或较新的有效可读根），它读得出的记录不齐，
+//! 它的账两份副本都验不过（账根经新根段里的树表指针找）。「映射现算得出」那一半镜像上不复算，这一格只判必要条件。
+//!
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次三条记录的发布 R（顺序写三个数据单元），
+//! 崩在 R 的根槽落盘之后、系统配置轮换之前；崩溃镜像上 R 的根槽、R 的树表与中央映射树根两份盘面写成全零（不是读回全零那一层）。
+//! R 的账按新根段里的树表指针找：树表两份都验不过就读不到账根（新根段里那条分配记录树根在带文件的一版上恒是空指针，
+//! 判据怎么读见 `mount_transparency::account_unreadable_on_the_crash_image` 的注释，交主 agent 定）。
+mod common_unwitnessed_publish;
+
+use common_unwitnessed_publish::{
+    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
+    record_one_publish_after_overwrites, PublishRecordedAsR, RangesReadingBackAllZero,
+    RecordedPublish,
+};
+use singlefs_core::mount::{
+    mount_writable, MountError, RollbackTarget, StillUnreadableAfterOneReread,
+};
+use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite};
+use singlefs_harness::mount_transparency::{
+    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
+    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
+    RefusalConditionNotMet, RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
+};
+
+const OVERWRITES_BEFORE_R: u64 = 5;
+
+fn recorded_publish_of_three_records() -> RecordedPublish {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
+    recorded
+}
+
+/// R 的根槽、树表与中央映射树根的写（两棵树的根两块盘各一份）：R 不在根环里可读、R 的账（经树表读）读不出、映射也现算不成。
+fn root_slot_tree_table_and_mapping_root_writes_of_r(
+    recorded: &RecordedPublish,
+) -> Vec<&RetainedWrite> {
+    let root_of_r = recorded.root();
+    let mut writes = vec![recorded.root_slot_write()];
+    for (pointer, what) in [
+        (&root_of_r.tree_table, "树表"),
+        (&root_of_r.mapping_root, "中央映射树根"),
+    ] {
+        let unit_writes = recorded.unit_writes_at(pointer);
+        assert_eq!(unit_writes.len(), 2, "R 自己写了{what}，两块盘各一份");
+        writes.extend(unit_writes);
+    }
+    writes
+}
+
+/// `image` 上 `writes` 那几段盘面真的写成全零（不是读回全零的那一层：崩溃镜像本身就是这样）。
+fn image_with_the_writes_zeroed(image: &MemoryPool, writes: &[&RetainedWrite]) -> MemoryPool {
+    let mut zeroed = image.clone();
+    for write in writes {
+        let length = usize::try_from(write.length_in_bytes()).expect("一次写的长度装得进 usize");
+        zeroed
+            .devices
+            .get_mut(&write.device)
+            .expect("写落在池里的一块盘上")
+            .write(write.offset, &vec![0u8; length]);
+    }
+    zeroed
+}
+
+/// 崩溃镜像：R 的根槽、树表与映射根全零，末条（第 3 条，两份）也全零——读得出的第 1、2 条缺末条，记录不齐。
+fn crash_image_with_the_last_record_account_and_mapping_of_r_zeroed(
+    recorded: &RecordedPublish,
+) -> MemoryPool {
+    let mut zeroed_writes = root_slot_tree_table_and_mapping_root_writes_of_r(recorded);
+    zeroed_writes.extend(recorded.record_writes_of_the_record(3));
+    image_with_the_writes_zeroed(
+        &recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
+        &zeroed_writes,
+    )
+}
+
+/// 在 `crash_image` 上起一次可写挂载（不带读回全零那一层）：被拒，盘上逐字节不变；交回那一拒。
+fn refusal_of_the_writable_mount_on(
+    recorded: &RecordedPublish,
+    crash_image: &MemoryPool,
+) -> MountError {
+    let mut devices =
+        devices_reading_back_all_zero_on(crash_image, &RangesReadingBackAllZero::default());
+    let refusal = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect_err("R 的账与映射都读不出、记录不齐：重读一次仍这样，拒可写");
+    assert!(
+        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes)
+            == *crash_image,
+        "在取号之前拒：两块盘逐字节不变"
+    );
+    refusal
+}
+
+/// 崩溃镜像上 R 缺末条、树表与映射根两份都是全零：C 读 R 的账读不出、记录不齐、映射也现算不成，拒成另起的那一个成员、点名 R
+/// （不是 C393 那一拒的 `AccountOfAnAbandonedRoot`）。
+/// 判别力：那一拒照旧报被抛弃根那一个成员（变异「删除线第43批 C_MAPPING 那一拒照旧报被抛弃根那一个成员」）时这一条红。
+#[test]
+fn a_publish_whose_account_and_mapping_are_zero_while_its_last_record_is_missing_is_refused_with_its_own_error_member(
+) {
+    let recorded = recorded_publish_of_three_records();
+    let crash_image = crash_image_with_the_last_record_account_and_mapping_of_r_zeroed(&recorded);
+    let refusal = refusal_of_the_writable_mount_on(&recorded, &crash_image);
+    assert!(
+        matches!(
+            &refusal,
+            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
+                if matches!(
+                    still_unreadable.as_ref(),
+                    StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+                        unapplied_publish: RollbackTarget { instance, checkpoint_txg },
+                    } if (*instance, *checkpoint_txg) == recorded.key
+                )
+        ),
+        "拒因是 C_MAPPING 那一个成员、点名 R {:?}：{refusal:?}",
+        recorded.key
+    );
+}
+
+/// 同一份崩溃镜像、同一拒：判法把它归到单子上，在崩溃镜像上复算判据为真（R 在水位之上没施加，读得出的第 1、2 条缺末条，
+/// R 的树表两份都验不过、读不到账根），盘上逐字节不变、只读读出在允许集合里，拒得合法。
+/// 判别力：新成员仍映成不在单子上（变异「删除线第43批 C_MAPPING 那一拒仍映成不在 D23 单子上」）时判成 `ReasonNotOnTheD23List`，这一条红。
+#[test]
+fn a_refusal_for_an_unreadable_account_with_incomplete_records_is_legal_on_the_d23_list_when_the_crash_image_bears_it_out(
+) {
+    let recorded = recorded_publish_of_three_records();
+    let crash_image = crash_image_with_the_last_record_account_and_mapping_of_r_zeroed(&recorded);
+    let refusal = refusal_of_the_writable_mount_on(&recorded, &crash_image);
+    let selected_version = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
+        .report
+        .effective_root;
+    assert_eq!(
+        selected_version,
+        Some(recorded.previous_key),
+        "R 的根槽全零、缺末条：R 整次不施加，崩溃镜像上只读恢复落到 P"
+    );
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &crash_image,
+            selected_version,
+            &refusal,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ),
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+                unapplied_publish: recorded.key,
+            }
+        )
+    );
+}
+
+/// 崩溃镜像上 R 的三条记录都在（序号 1、2、3，末条带标志），根槽、树表与映射根两份盘面全零：账读不出，但记录齐。
+/// 拿一次报 `C_MAPPING` 那一拒的拒去判（实现在这份盘面上不走那一支：记录齐的交 F1 乙；判法不信拒因、自己复算），判据为假，拒得不合法。
+/// 判别力：复算不看记录齐不齐（变异「删除线第43批 C_MAPPING 判据复算不看记录齐不齐」）时只剩「水位之上」与「账验不过」、判成合法，这一条红。
+#[test]
+fn a_refusal_claimed_for_an_unreadable_account_with_incomplete_records_is_illegal_on_a_crash_image_holding_every_record_of_the_publish(
+) {
+    let recorded = recorded_publish_of_three_records();
+    let crash_image = image_with_the_writes_zeroed(
+        &recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
+        &root_slot_tree_table_and_mapping_root_writes_of_r(&recorded),
+    );
+    let selected_version = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
+        .report
+        .effective_root;
+    assert_eq!(
+        selected_version,
+        Some(recorded.previous_key),
+        "R 的根槽全零、末条点名的树表验不过：R 不施加，只读恢复落到 P"
+    );
+    let claimed_refusal = MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
+        StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+            unapplied_publish: RollbackTarget {
+                instance: recorded.key.0,
+                checkpoint_txg: recorded.key.1,
+            },
+        },
+    ));
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &crash_image,
+            selected_version,
+            &claimed_refusal,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
+                unapplied_publish: recorded.key,
+            },
+            conditions_not_met: vec![
+                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            ],
+        }
+    );
+}
````

## 删除线第 43 批：mutations-append.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch43/a/patch/mutations-append.tsv）

````text
删除线第43批 C_MAPPING 那一拒照旧报被抛弃根那一个成员（与 C393 那一拒分不开）	crates/singlefs-core/src/mount.rs	            StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {\n                unapplied_publish: RollbackTarget {	            StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {\n                abandoned_root: RollbackTarget {	-p singlefs-harness --test writable_mount_refusal_for_an_unreadable_account_with_incomplete_records_judged_on_the_d23_list	a_publish_whose_account_and_mapping_are_zero_while_its_last_record_is_missing_is_refused_with_its_own_error_member
删除线第43批 C_MAPPING 那一拒仍映成不在 D23 单子上（判法交 ReasonNotOnTheD23List）	crates/singlefs-harness/src/mount_transparency.rs	                StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {\n                    unapplied_publish,\n                } => Some(\n                    RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {\n                        unapplied_publish: (\n                            unapplied_publish.instance,\n                            unapplied_publish.checkpoint_txg,\n                        ),\n                    },\n                ),\n	                StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { .. } => None,\n	-p singlefs-harness --test writable_mount_refusal_for_an_unreadable_account_with_incomplete_records_judged_on_the_d23_list	a_refusal_for_an_unreadable_account_with_incomplete_records_is_legal_on_the_d23_list_when_the_crash_image_bears_it_out
删除线第43批 C_MAPPING 判据复算不看记录齐不齐（三条记录都在的崩溃镜像上报 C_MAPPING 的拒判成合法）	crates/singlefs-harness/src/mount_transparency.rs	    if ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish {\n        return false;\n    }	    if false && ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish {\n        return false;\n    }	-p singlefs-harness --test writable_mount_refusal_for_an_unreadable_account_with_incomplete_records_judged_on_the_d23_list	a_refusal_claimed_for_an_unreadable_account_with_incomplete_records_is_illegal_on_a_crash_image_holding_every_record_of_the_publish
````

## 删除线第 43 批：mutations-replacements.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch43/a/patch/mutations-replacements.tsv）

````text
遗留改动代码轮 LR4（C393 (b)-从映射现算最后一支）：被抛弃根的账与从映射现算重读一次仍都不成时不拒可写，当它什么都不引用	crates/singlefs-core/src/mount.rs	    )\n    .ok_or_else(|| {\n        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(\n            StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {	    )\n    .or(Some(Vec::new()))\n    .ok_or_else(|| {\n        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(\n            StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area	an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking
删除线第38批 C_MAPPING 现算不成也不拒（账与映射都读不出、缺末条）	crates/singlefs-core/src/mount.rs	    )\n    .ok_or_else(|| {\n        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(\n            StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {	    )\n    .or(Some(Vec::new()))\n    .ok_or_else(|| {\n        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(\n            StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {	-p singlefs-harness --test writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing -- a_publish_whose_account_and_mapping_read_back_zero_while_its_last_record_is_missing_refuses_the_writable_mount_before_any_write	a_publish_whose_account_and_mapping_read_back_zero_while_its_last_record_is_missing_refuses_the_writable_mount_before_any_write
删除线第38批 C_MAPPING 那一支什么都不交（根读得出、末条与树表读回全零）	crates/singlefs-core/src/mount.rs	    placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread(\n        devices,\n        unit_area_start,\n        root_of_the_publish,\n        current_records,\n        before_the_one_reread,\n    )\n    .ok_or_else(|| {\n        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(\n            StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {\n                unapplied_publish: RollbackTarget {\n                    instance: root_of_the_publish.instance,\n                    checkpoint_txg: root_of_the_publish.checkpoint_txg,\n                },\n            },\n        ))\n    })\n}	    let _ = (devices, unit_area_start, root_of_the_publish, current_records, before_the_one_reread);\n    Ok(Vec::new())\n}	-p singlefs-harness --test writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing -- a_newer_readable_root_whose_last_record_and_tree_table_read_back_zero_keeps_its_units_recomputed_from_the_mapping	a_newer_readable_root_whose_last_record_and_tree_table_read_back_zero_keeps_its_units_recomputed_from_the_mapping
````

## 删除线第 41 批：恢复内核钉值（不在 crates.patch 里，随第 41 批打）（/home/fy5090/.local/share/singlefs/work/m3-batch41/a/hunks/crash_judge_dispatch.rs.pin-not-in-crates-patch.patch）

````diff
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs	2026-10-08 00:10:20.894903693 +0000
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs	2026-10-08 00:10:20.902830877 +0000
@@ -1339,13 +1339,13 @@
     }
 
     /// 三个内核此刻展开之后的表达式数与离预算的余量（2026-10-08 在删除线第 39 批的仓副本上用 naga =30.0.1 现量的：恢复内核的 F2 改成 F2_NEWEST 那一形
-    /// （退到 [见证到的那一版, 所选根) 里最新、不被所选根实例表抛弃的根，沿链读所选根的实例表），恢复 318 224 → 328 934，扫描与走读没变；
+    /// （退到 [见证到的那一版, 所选根) 里最新、不被所选根实例表抛弃的根，沿链读所选根的实例表），恢复 318 224 → 328 934，扫描与走读没变；删除线第 41 批 F2 对 c_见证 = 0 加守卫，恢复 328 934 → 328 970；
     /// 第 36 批之前是恢复 320 015、扫描 382 259、走读 337 872）：
     /// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
     #[test]
     fn each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget() {
         for (name, source, pinned_expressions, pinned_margin) in [
-            ("recovery", recovery_kernel_source(), 328_934, 71_066),
+            ("recovery", recovery_kernel_source(), 328_970, 71_030),
             (
                 "checker-scan",
                 crate::crash_judge_gpu::checker_scan_kernel_source(),
````

