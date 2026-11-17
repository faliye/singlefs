# m4-gpugc-precheck-b-a-code-r3：被判的改动全文（第二轮判决第三节第 1–4 条，第 131 批补丁）

补丁 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-r2-fixes-writer/patch/crates.patch`（打在别的会话的回落算法那一批之后）。另有主 agent 自己改的一处注释：`mounted_session.rs` 一句函数文档照 `background_compute.rs` 第 1512–1513 行写成两半（「出错之前就挂着请求了、还没交给实现的切换时当场解除；交给了实现、实现还在为它开卡的，照停下记，那一组换上的那一刻才解除」），不属这一题，只是同一份文件里的改动。

```diff
diff --git a/crates/singlefs-core/src/admission.rs b/crates/singlefs-core/src/admission.rs
index 2dd4b2f..c7e3950 100644
--- a/crates/singlefs-core/src/admission.rs
+++ b/crates/singlefs-core/src/admission.rs
@@ -44,7 +44,7 @@ use singlefs_format::{
 
 use crate::address::{CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, InodeNumber};
 use crate::allocation_record_tree::{span_in_slots_at_level, AllocationRecordTreeGeometry};
-use crate::allocator::PoolAllocator;
+use crate::allocator::{PoolAllocator, SlotPairCount};
 use crate::code_two_tree::{
     plan_the_tree_after_this_publish, CodeTwoKeyFieldWidths, CodeTwoTreeKey,
     CodeTwoTreeNodeCapacity, CodeTwoTreeShape,
@@ -977,7 +977,7 @@ pub struct PlacementRoomOnOneDevice {
     pub device: DeviceIdentity,
     /// 数据单元落得下的槽对：两槽都没挡（已分配、影子账隔离、抬 F 扣住三种位都没置）、起点偶数、不在这次挂载开过的任何一个聚簇段里
     /// （已定项 8 第 1、2 条，已定项 10 ②③；`DeviceFreeMap::unblocked_slot_pairs_outside_the_cluster_segments`）。
-    pub slot_pairs_for_data_units: u64,
+    pub slot_pairs_for_data_units: SlotPairCount,
     /// 提交内生块落得下的槽：三种位都没置，聚簇段内外都算——开放段 bump、开全空段、回落（先取这次发布重写集的叶、再取每块盘都空的槽最多的叶）挡的都只是这三种位
     /// （已定项 8 第 2 条、已定项 10 ①⑤；`DeviceFreeMap::unblocked_slots`）。
     pub slots_for_commit_generated_units: u64,
@@ -1074,9 +1074,10 @@ pub fn admit_the_units_landing_on_every_device(
     let devices_where_the_units_cannot_land: Vec<DeviceShortOfDemand> = room
         .iter()
         .filter_map(|room_on_the_device| {
-            if room_on_the_device.slot_pairs_for_data_units < units.data_units {
+            if room_on_the_device.slot_pairs_for_data_units < SlotPairCount(units.data_units) {
                 let slots_of_the_pairs = room_on_the_device
                     .slot_pairs_for_data_units
+                    .0
                     .checked_mul(slots_of_a_data_unit)
                     .expect("槽对的槽数装得进 u64");
                 return Some(DeviceShortOfDemand {
@@ -2494,7 +2495,7 @@ mod tests {
 
     fn room(
         device: u32,
-        slot_pairs_for_data_units: u64,
+        slot_pairs_for_data_units: SlotPairCount,
         slots_for_commit_generated_units: u64,
     ) -> PlacementRoomOnOneDevice {
         PlacementRoomOnOneDevice {
@@ -2519,7 +2520,7 @@ mod tests {
         };
         assert_eq!(
             admit_the_units_landing_on_every_device(
-                &[room(0, 3, 20), room(1, 3, 20)],
+                &[room(0, SlotPairCount(3), 20), room(1, SlotPairCount(3), 20)],
                 three_data_units_and_fourteen_slots
             ),
             Ok(()),
@@ -2527,7 +2528,7 @@ mod tests {
         );
         assert_eq!(
             admit_the_units_landing_on_every_device(
-                &[room(0, 3, 20), room(1, 2, 20)],
+                &[room(0, SlotPairCount(3), 20), room(1, SlotPairCount(2), 20)],
                 three_data_units_and_fourteen_slots
             ),
             Err(AdmissionRefusedOnSomeDevices {
@@ -2542,7 +2543,7 @@ mod tests {
         );
         assert_eq!(
             admit_the_units_landing_on_every_device(
-                &[room(0, 3, 19), room(1, 3, 19)],
+                &[room(0, SlotPairCount(3), 19), room(1, SlotPairCount(3), 19)],
                 UnitsOfAPublishToLand {
                     data_units: 3,
                     commit_generated_slots: 15,
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index e7c4845..28a4f74 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -870,14 +870,16 @@ impl DeviceFreeMap {
     pub fn unblocked_slot_pairs_outside_the_cluster_segments(
         &self,
         cluster_segments: &BTreeSet<SlotNumber>,
-    ) -> u64 {
-        self.unblocked_slot_pairs.0
-            - pairs_inside_the_cluster_segments(
-                self.unit_area_start,
-                &self.unblocked_slot_pairs_per_segment,
-                cluster_segments,
-            )
-            .0
+    ) -> SlotPairCount {
+        SlotPairCount(
+            self.unblocked_slot_pairs.0
+                - pairs_inside_the_cluster_segments(
+                    self.unit_area_start,
+                    &self.unblocked_slot_pairs_per_segment,
+                    cluster_segments,
+                )
+                .0,
+        )
     }
 
     /// defer、影子账隔离、抬 F 扣住三样全放开之后用户数据落得下的槽对数：两槽都不罩在未释放分配记录下、起点偶数、不在 `cluster_segments` 里任何一段
@@ -3911,7 +3913,7 @@ mod tests {
     fn unblocked_slots_and_pairs_outside_the_cluster_segments_by_scan(
         pool: &PoolAllocator,
         device_map: &DeviceFreeMap,
-    ) -> (u64, u64) {
+    ) -> (u64, SlotPairCount) {
         let first_slot = device_map.unit_area_start().slot().0;
         let past_the_last_slot = first_slot + device_map.unit_area_slots();
         let unblocked_slots = u64::try_from(
@@ -3934,6 +3936,7 @@ mod tests {
                 })
                 .count(),
         )
+        .map(SlotPairCount)
         .expect("槽对数装得进 u64");
         (unblocked_slots, pairs_outside)
     }
@@ -3971,7 +3974,7 @@ mod tests {
         check(&pool, "空盘");
         assert_eq!(
             pool.devices[0].unblocked_slot_pairs_outside_the_cluster_segments(&BTreeSet::new()),
-            192,
+            SlotPairCount(192),
             "385 槽：192 对，末尾一槽不成对"
         );
         let mut placements = Vec::new();
@@ -4073,9 +4076,7 @@ mod tests {
         for slot in first_slot..first_slot + device_map.unit_area_slots() {
             device_map.clear_isolation_of_slot(SlotNumber(slot));
         }
-        SlotPairCount(
-            device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments),
-        )
+        device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
     }
 
     /// 一串定种子的伪随机分配、开段、释放、回收、抬 F 扣住与放开、影子账隔离与清隔离、管理员回退复活（两块盘，单元区 385 槽：
@@ -4216,6 +4217,43 @@ mod tests {
         });
     }
 
+    /// 重开时从记录重建分配器（[`PoolAllocator::rebuild_from_records`]）之后，抬 F 之前先算槽对那一判读的三样——逐槽「罩在未释放记录下」位、
+    /// 按段的槽对数、全盘的槽对数——与重建之前逐项相同：三样只随记录未释放与否变，重建照记录走 `mark_allocated` 与 `mark_released`。
+    /// 一串随机分配、释放、回收、扣住、隔离与复活的每一步之后各重建一次比，两块盘都比。
+    /// 判别力：重建时已释放的两槽落点不进 defer（少走一处 `mark_released`），那两槽重建之后仍罩在未释放记录下，三样都对不上。
+    /// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
+    #[test]
+    fn slot_pairs_once_everything_is_released_are_the_same_after_rebuilding_the_allocator_from_its_records(
+    ) {
+        random_history_of_the_allocator(&|pool, step_number, what_the_step_did| {
+            let rebuilt = PoolAllocator::rebuild_from_records(
+                pool.devices
+                    .iter()
+                    .map(|device_map| {
+                        DeviceFreeMap::new(device_map.device, device_map.device_size_in_bytes)
+                    })
+                    .collect(),
+                pool.records().to_vec(),
+            );
+            for (device_map, rebuilt_device_map) in pool.devices.iter().zip(&rebuilt.devices) {
+                assert_eq!(
+                    (
+                        &rebuilt_device_map.is_under_an_unreleased_record,
+                        &rebuilt_device_map.slot_pairs_without_an_unreleased_record_per_segment,
+                        rebuilt_device_map.slot_pairs_without_an_unreleased_record,
+                    ),
+                    (
+                        &device_map.is_under_an_unreleased_record,
+                        &device_map.slot_pairs_without_an_unreleased_record_per_segment,
+                        device_map.slot_pairs_without_an_unreleased_record,
+                    ),
+                    "第 {step_number} 步（{what_the_step_did}）：盘 {:?} 重建之后的（逐槽位、按段槽对数、全盘槽对数）与重建之前相同",
+                    device_map.device
+                );
+            }
+        });
+    }
+
     #[test]
     fn accounting_deltas_are_maintained_incrementally() {
         let mut pool = pool_after_mkfs();
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index f0d1096..9d93a5c 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -655,8 +655,9 @@ pub enum UserChangeRefused {
     /// 空间不够（报给用户就是 ENOSPC），推过的与不再推的原因在 [`NoSpaceAfterRaisingTheFloor`] 里。
     /// 装箱：不装箱 `UserChangeRefused` 就大过 clippy `result_large_err` 的 128 字节（`MountError` 那几个成员同一个做法）。
     NoSpaceAfterRaisingTheFloor(Box<NoSpaceAfterRaisingTheFloor>),
-    /// 空间不够（报给用户就是 ENOSPC），而且推抬 F 放得出的全放出来也放不下：这一次不再推，[`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`] 带上界与需求。
-    /// 这一次判之前已经推成的串、发成的版在里面，头一次被拒就判到这里时它们是空的，盘上逐字节不变。装箱同 `NoSpaceAfterRaisingTheFloor`。
+    /// 空间不够（报给用户就是 ENOSPC），抬 F 之前先算判出这一次不推，两形都在 [`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`] 里：
+    /// 槽对与式子那两判是推抬 F 放得出的全放出来也放不下，带上界与需求；合取 7 那一形是这次调用还一串没推、写之前那一刻全空聚簇段已少于 `N₀`，
+    /// 带 `N₀` 与少于它的盘。这一次判之前已经推成的串、发成的版在里面，头一次被拒就判到这里时它们是空的，盘上逐字节不变。装箱同 `NoSpaceAfterRaisingTheFloor`。
     NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(
         Box<NoSpaceEvenWithEverythingAFloorRaiseCouldRelease>,
     ),
@@ -683,9 +684,10 @@ pub struct NoSpaceAfterRaisingTheFloor {
     pub wide_protection_at_the_ceiling: WideProtectionAtTheCeiling,
 }
 
-/// 推抬 F 放得出的全放出来也放不下：`devices_short` 里每块盘的上界都小于需求，或这次调用推串之前全空聚簇段已少于 `N₀`、被拒的又是合取 7 判的前台写入
-/// （`empty_cluster_segments_below_the_floor`），所以这一次不推（D16（发布语义） 已定项 1「准入」那一行的射程：
-/// 推了也写不进的写不花根环的槽与写盘；已定项 10 射程「抬 F 之前先算」）。别的被拒成员一样带着这次调用已经落盘的：`floor_raises` 与 `refusals_that_pushed_the_floor_raises`
+/// 抬 F 之前先算判出这一次不推（D16（发布语义） 已定项 10 射程「抬 F 之前先算」），两形：
+/// 槽对与式子那两判（`devices_short`）是推抬 F 放得出的全放出来也放不下：每块列出的盘上界都小于需求；
+/// 合取 7 那一形（`empty_cluster_segments_below_the_floor`）是这次调用还一串没推、写之前那一刻全空聚簇段已少于 `N₀`，被拒的又是合取 7 判的前台写入，不推。
+/// 别的被拒成员一样带着这次调用已经落盘的：`floor_raises` 与 `refusals_that_pushed_the_floor_raises`
 /// 一一对应，`versions_published` 是推的抬 F 之外发成的那几版（[`VersionsPublishedInThisChange`]）。
 #[derive(Debug)]
 pub struct NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {
@@ -2934,7 +2936,8 @@ impl MountedSession {
     /// unlink 发成意图之后接着把它的全部批走完。走空里的删除类发布（不判式子那一判、落得下那一判照判，D28（挂载期承诺量） 已定项 1）
     /// 被拒是空间不够时同样推一串抬 F、回收之后接着走（用户 2026-10-02 定案 Q-V「推抬 F 再试」，每一次被拒的删除类发布各算一次准入），
     /// 推满仍不够交回 `NoSpaceAfterRaisingTheFloor`；推的那几串与意图那次推的一起报在 `floor_raises` 里。
-    /// 推抬 F 放得出的全放出来也放不下时不推，交回 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`（`push_one_floor_raise_for_a_refusal_for_space`）。
+    /// 抬 F 之前先算判出不推时交回 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`（`push_one_floor_raise_for_a_refusal_for_space`）：
+    /// 槽对与式子那两判是推抬 F 放得出的全放出来也放不下；合取 7 那一形是这次调用还一串没推、写之前那一刻全空聚簇段已少于 `N₀`。
     /// 删除那几次发布的两个只供测试的开关取分配器上装的（可写挂载的开关链装上去的，`unlink::DeletionSwitches`）。
     /// 每发成一次（走空的每一步、这次改动）`current` 都按 `PoolVersion::of_a_version_of_trees` 重新归成员；现行那一版是有树而文件集合为空的
     /// 一版时只放建 inode（D8（核心索引结构） 已定项 16 第 5 条），别的改动拒成 `NoFileVersionToChange`。
@@ -6825,7 +6828,8 @@ pub enum AcceptRefused {
     PendingChangesTogetherWithThisChangeRefusedByThePublishJudgement(Box<PublishError>),
     /// `judgement` 那一批被空间拒，当场推抬 F 再判（D16（发布语义） 已定项 1「准入」那一行）之后还是没收下，`refusal` 原样：
     /// [`UserChangeRefused::NoSpaceAfterRaisingTheFloor`] 是推到不再推了仍放不下（一次准入的发布数做满、F 已在上限、抬 F 那一串自己放不下，一串都没推成时 `floor_raises` 是空的）；
-    /// [`UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`] 是推抬 F 放得出的全放出来也放不下，这一次不再推（头一次被拒就判到这里时 `floor_raises` 是空的）；
+    /// [`UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`] 是抬 F 之前先算判出这一次不推：槽对与式子那两判是推抬 F 放得出的全放出来也放不下，
+    /// 合取 7 那一形是这次调用还一串没推、写之前那一刻全空聚簇段已少于 `N₀`（头一次被拒就判到这里时 `floor_raises` 是空的）；
     /// [`UserChangeRefused::FloorRaiseFailedWhilePushingForSpace`] 是抬 F 自己报了错（写盘报错时那一次空发布冻结）；
     /// [`UserChangeRefused::Publish`] 是推过之后再判拒成了别的原因。推成的那几串已落盘，现行那一版在它们后面（[`Self::floor_raises_pushed_by_this_accept`]）。
     /// 这一次改动没收进待发；待发里已有的原样留着，冻结的那一次空发布不含它们。
@@ -8636,7 +8640,7 @@ mod everything_a_floor_raise_could_release_tests {
         assert_eq!(
             allocator.devices[0]
                 .unblocked_slot_pairs_outside_the_cluster_segments(allocator.cluster_segments()),
-            0,
+            SlotPairCount(0),
             "三对都挡着：落得下那一判此刻数到 0 对"
         );
         allocator
diff --git a/crates/singlefs-verification/src/bin/e180_admission_reading_versus_writable_bytes.rs b/crates/singlefs-verification/src/bin/e180_admission_reading_versus_writable_bytes.rs
index 0fc579f..59dd828 100644
--- a/crates/singlefs-verification/src/bin/e180_admission_reading_versus_writable_bytes.rs
+++ b/crates/singlefs-verification/src/bin/e180_admission_reading_versus_writable_bytes.rs
@@ -1705,6 +1705,7 @@ fn upper_bound_units_by_the_space_conditions(pool: &Pool, reading: &StateReading
             map.unblocked_slot_pairs_outside_the_cluster_segments(
                 pool.session.allocator.cluster_segments(),
             )
+            .0
         })
         .min()
         .expect("至少一块盘");
@@ -2697,6 +2698,7 @@ fn half_of_the_pairs_outside_the_cluster_segments(pool: &Pool) -> u64 {
             map.unblocked_slot_pairs_outside_the_cluster_segments(
                 pool.session.allocator.cluster_segments(),
             )
+            .0
         })
         .min()
         .expect("至少一块盘")
diff --git a/crates/singlefs-verification/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs b/crates/singlefs-verification/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs
index de5c0be..12ebf3e 100644
--- a/crates/singlefs-verification/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs
+++ b/crates/singlefs-verification/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs
@@ -1354,6 +1354,7 @@ fn passed_the_landing_check_and_the_formula(outcome: &WriteOutcome) -> bool {
 fn data_unit_pairs_outside_the_clusters_of(allocator: &PoolAllocator, device_index: usize) -> u64 {
     allocator.devices[device_index]
         .unblocked_slot_pairs_outside_the_cluster_segments(allocator.cluster_segments())
+        .0
 }
 
 fn data_unit_pairs_outside_the_clusters(pool: &Pool, device_index: usize) -> u64 {
@@ -1787,8 +1788,9 @@ fn run_edge_history(
 fn mark_the_lowest_pairs_outside_the_clusters(allocator: &mut PoolAllocator, percent: u64) {
     let cluster_segments: BTreeSet<SlotNumber> = allocator.cluster_segments().clone();
     for device_map in &mut allocator.devices {
-        let total_pairs =
-            device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments);
+        let total_pairs = device_map
+            .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
+            .0;
         let wanted = total_pairs * percent / 100;
         let mut marked = 0;
         let mut candidate = device_map.unit_area_start().slot().0;
@@ -2235,6 +2237,7 @@ impl RoomReading {
                 .map(|device_map| {
                     device_map
                         .unblocked_slot_pairs_outside_the_cluster_segments(&no_cluster_segments)
+                        .0
                 })
                 .collect(),
             unblocked_slots: allocator
diff --git a/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs b/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
index e62ad19..9be57e5 100644
--- a/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
+++ b/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
@@ -1036,7 +1036,9 @@ fn a_write_short_on_slot_pairs_only_behind_slots_held_for_the_floor_is_not_refus
         let slots = device_map.unit_area_slots();
         // 迭代上界是单元区槽数；提前出口：段外的槽对只剩这么多。
         for slot in (start..start + slots).rev() {
-            if device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
+            if device_map
+                .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
+                .0
                 <= slot_pairs_left
             {
                 break;
@@ -1046,7 +1048,9 @@ fn a_write_short_on_slot_pairs_only_behind_slots_held_for_the_floor_is_not_refus
             }
         }
         assert_eq!(
-            device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments),
+            device_map
+                .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
+                .0,
             slot_pairs_left,
             "阳性对照：扣到现有的槽对加 defer 与隔离比需求少一对"
         );
@@ -1088,7 +1092,9 @@ fn slot_pairs_counting_one_pair_for_every_slot_a_floor_raise_could_release(
     device_map: &DeviceFreeMap,
     cluster_segments: &BTreeSet<SlotNumber>,
 ) -> u64 {
-    device_map.unblocked_slot_pairs_outside_the_cluster_segments(cluster_segments)
+    device_map
+        .unblocked_slot_pairs_outside_the_cluster_segments(cluster_segments)
+        .0
         + device_map.deferred_slots()
         + device_map.isolated_slots()
         + (device_map.free_slots() - device_map.unblocked_slots())
@@ -1233,7 +1239,9 @@ fn an_accept_short_on_slot_pairs_that_releasing_everything_would_not_complete_is
     let mut devices_short_expected = Vec::new();
     for device_map in &session.allocator.devices {
         assert_eq!(
-            device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments),
+            device_map
+                .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
+                .0,
             CLUSTER_SEGMENT_SLOTS / 2,
             "阳性对照：段外没挡的槽对只剩最高那一段的 32 对"
         );
@@ -1319,7 +1327,9 @@ fn hold_the_highest_free_slots_leaving_slot_pairs(
         let slots = device_map.unit_area_slots();
         // 迭代上界是单元区槽数；提前出口：段外没挡的槽对只剩这么多。
         for slot in (start..start + slots).rev() {
-            if device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
+            if device_map
+                .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
+                .0
                 <= slot_pairs_left
             {
                 break;
@@ -1371,7 +1381,9 @@ fn hold_slots_and_raise_the_empty_cluster_segments_floor_above_every_device(
     let cluster_segments = session.allocator.cluster_segments().clone();
     for device_map in &session.allocator.devices {
         assert_eq!(
-            device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments),
+            device_map
+                .unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
+                .0,
             units_of_the_write - 1,
             "阳性对照：段外没挡的槽对比需求少一对，落得下那一判拒"
         );
@@ -1540,7 +1552,8 @@ fn a_publish_of_a_write_while_empty_cluster_segments_are_below_the_floor_is_refu
 
 /// 删除队列没走空、走空那一步被空间拒，而每块盘的全空聚簇段在写之前都已少于 `N₀`：走空的每一步是删除类、合取 7 免，
 /// 推串之前不判合取 7，照推一串抬 F。单元区里空闲的槽都记成已分配，抬 F 那一串自己也放不下，推到不再推了报空间不够，
-/// 报里被拒的是走空那一步、空间不够的那两种之一。
+/// 报里被拒的是走空那一步、空间不够的那两种之一，不再推是因为抬 F 那一串自己放不下；走空的头一步就被拒、抬 F 那一串没推成，
+/// 报里推成的串与发成的版都是空的。
 /// 判别力：走空那一处把被拒的那一步当成合取 7 判的前台写入交下去时，推串之前那一判判出、一串都不推就拒（全放出来也放不下）。
 #[test]
 fn a_walk_step_refused_for_space_while_empty_cluster_segments_are_below_the_floor_is_pushed_for() {
@@ -1589,4 +1602,23 @@ fn a_walk_step_refused_for_space_while_empty_cluster_segments_are_below_the_floo
         "被拒的是走空那一步、空间不够：{:?}",
         no_space.last_refusal
     );
+    assert!(
+        matches!(
+            no_space.stop,
+            FloorRaiseStop::FloorRaiseRefusedForSpace { .. }
+        ),
+        "不再推是因为抬 F 那一串自己也放不下：{:?}",
+        no_space.stop
+    );
+    assert!(
+        no_space.floor_raises.is_empty()
+            && no_space.refusals_that_pushed_the_floor_raises.is_empty(),
+        "抬 F 那一串自己放不下：这次调用一串都没推成，实际推成 {} 串",
+        no_space.floor_raises.len()
+    );
+    assert!(
+        no_space.versions_published.is_empty(),
+        "单元区里一个空闲的槽都没有：走空的头一步就被拒，这次调用一版都没发成，实际发成 {} 版",
+        no_space.versions_published.len()
+    );
 }
diff --git a/crates/singlefs-verification/tests/admission_refusals_name_the_judgement_that_found_each_device_short.rs b/crates/singlefs-verification/tests/admission_refusals_name_the_judgement_that_found_each_device_short.rs
index 7398254..0b5bd36 100644
--- a/crates/singlefs-verification/tests/admission_refusals_name_the_judgement_that_found_each_device_short.rs
+++ b/crates/singlefs-verification/tests/admission_refusals_name_the_judgement_that_found_each_device_short.rs
@@ -11,6 +11,7 @@ use singlefs_core::admission::{
     JudgementThatFoundTheDeviceShort, PlacementRoomOnOneDevice, PoolWideCommitments, ReplicaCount,
     UnitsOfAPublishToLand,
 };
+use singlefs_core::allocator::SlotPairCount;
 
 const THE_ONLY_DEVICE: DeviceIdentity = DeviceIdentity(0);
 
@@ -66,7 +67,7 @@ fn formula_refusal_and_landing_refusal_with_the_same_bytes_are_told_apart_by_the
         admit_the_units_landing_on_every_device(
             &[PlacementRoomOnOneDevice {
                 device: THE_ONLY_DEVICE,
-                slot_pairs_for_data_units: 0,
+                slot_pairs_for_data_units: SlotPairCount(0),
                 slots_for_commit_generated_units: 1,
             }],
             UnitsOfAPublishToLand {
@@ -121,7 +122,7 @@ fn the_landing_judgement_names_whether_the_data_units_or_the_commit_generated_un
         admit_the_units_landing_on_every_device(
             &[PlacementRoomOnOneDevice {
                 device: THE_ONLY_DEVICE,
-                slot_pairs_for_data_units: 0,
+                slot_pairs_for_data_units: SlotPairCount(0),
                 slots_for_commit_generated_units: 2,
             }],
             UnitsOfAPublishToLand {
@@ -135,7 +136,7 @@ fn the_landing_judgement_names_whether_the_data_units_or_the_commit_generated_un
         admit_the_units_landing_on_every_device(
             &[PlacementRoomOnOneDevice {
                 device: THE_ONLY_DEVICE,
-                slot_pairs_for_data_units: 1,
+                slot_pairs_for_data_units: SlotPairCount(1),
                 slots_for_commit_generated_units: 2,
             }],
             UnitsOfAPublishToLand {
```
