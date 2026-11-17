# m4-gpugc-precheck-b-a-code-r1：被判的改动全文（改法乙、甲，第 126 批两份补丁）

产品代码（被判）：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/mounted_session.rs`，来自补丁 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-writer/patch/crates.patch`。

```diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index ab74127..eadb81b 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -404,6 +404,15 @@ pub struct DeviceFreeMap {
     unblocked_slot_pairs_per_segment: Vec<u64>,
     /// 上一项全部段之和。
     unblocked_slot_pairs: u64,
+    /// 每一槽是不是罩在未释放的分配记录下（`unreleased_per_segment` 数的那几槽，逐槽一位）：分配与复活时置、释放时清，回收不动。
+    /// 不罩在未释放记录下的槽只有四种：空闲的、defer 里的、影子账隔离的、抬 F 扣住的——抬 F 能放开的就是后三种。
+    is_under_an_unreleased_record: Vec<bool>,
+    /// 每个 64 槽聚簇段里两槽都不罩在未释放分配记录下的偶数起点槽对数：defer、影子账隔离、抬 F 扣住三样全放开之后用户数据落得下的槽对
+    /// （抬 F 之前先算槽对那一判的上界读它，`mounted_session::most_a_floor_raise_could_make_available`），
+    /// 随 `is_under_an_unreleased_record` 每变一槽增量维护。一对不跨段、单元区末尾凑不成一对的那一槽不算，同 `unblocked_slot_pairs_per_segment`。
+    slot_pairs_without_an_unreleased_record_per_segment: Vec<u64>,
+    /// 上一项全部段之和。
+    slot_pairs_without_an_unreleased_record: u64,
     /// 分配记录树每片叶里三种位都没置的单元区槽数（叶序号 = 槽号 ÷ [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]，下标 = 叶序号 − 单元区起点所在那片叶的序号；
     /// 单元区首尾那两片叶只数单元区里的那一截）。回落取空槽最多的叶时拿它当每片叶「每块盘都空的槽数」的上界
     /// （[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]），同上增量维护。
@@ -481,6 +490,38 @@ pub fn unit_area_slots_of_device(device_bytes: u64) -> u64 {
     .expect("盘末尾不在单元区起点之前：mkfs 判过 UnitAreaBeyondDevice，可写挂载在读盘之前判过 every_device_reaches_the_unit_area_start")
 }
 
+/// 按段维护的槽对数（`pairs_per_segment`，下标是从 `unit_area_start` 起的段号）里落在 `cluster_segments` 那几段的和：
+/// 段外槽对数 = 总数减它（[`DeviceFreeMap::unblocked_slot_pairs_outside_the_cluster_segments`] 与
+/// [`DeviceFreeMap::slot_pairs_without_an_unreleased_record_outside_the_cluster_segments`] 共用）。
+///
+/// # Panics
+/// `cluster_segments` 里有一段的起点不在这块盘单元区里的段边界上：那几段都是各盘一致答出来的全空段起点
+/// （[`DeviceFreeMap::lowest_empty_segment`]），每块盘上都是整段、在单元区里。
+fn pairs_inside_the_cluster_segments(
+    unit_area_start: UnitAreaStart,
+    pairs_per_segment: &[u64],
+    cluster_segments: &BTreeSet<SlotNumber>,
+) -> u64 {
+    cluster_segments
+        .iter()
+        .map(|segment_start| {
+            let offset_in_slots = segment_start
+                .0
+                .checked_sub(unit_area_start.slot().0)
+                .expect("聚簇段在单元区里：它是各盘一致答出来的全空段起点");
+            assert!(
+                offset_in_slots.is_multiple_of(CLUSTER_SEGMENT_SLOTS),
+                "聚簇段起点在段边界上：{segment_start:?}"
+            );
+            let segment =
+                usize::try_from(offset_in_slots / CLUSTER_SEGMENT_SLOTS).expect("段号装得进 usize");
+            *pairs_per_segment
+                .get(segment)
+                .expect("聚簇段在这块盘的单元区里：各盘一致答出来的整段")
+        })
+        .sum()
+}
+
 impl DeviceFreeMap {
     /// 单元区从默认环长下的起点（[`UnitAreaStart::of_the_default_journal_ring`]，槽 50176）起到设备末尾；mkfs 占的槽由调用方标上。
     /// 实例表、树表的落点（`make_filesystem::INSTANCE_TABLE_SLOT`）、恢复判分配记录落点、系统配置里写的单元区起点今天都还按这个数，
@@ -557,6 +598,15 @@ impl DeviceFreeMap {
             held_per_segment: vec![0; usize::try_from(segments).expect("段数")],
             unblocked_slots: unit_area_slots,
             unblocked_slot_pairs: unblocked_slot_pairs_per_segment.iter().sum(),
+            // 起步一槽都不罩在未释放记录下：每段的数与没挡的槽对数相同。
+            is_under_an_unreleased_record: vec![
+                false;
+                usize::try_from(unit_area_slots)
+                    .expect("单元区槽数")
+            ],
+            slot_pairs_without_an_unreleased_record: unblocked_slot_pairs_per_segment.iter().sum(),
+            slot_pairs_without_an_unreleased_record_per_segment: unblocked_slot_pairs_per_segment
+                .clone(),
             unblocked_slot_pairs_per_segment,
             unblocked_slots_per_allocation_record_tree_leaf,
             segments_reclaimed_not_yet_discarded: BTreeSet::new(),
@@ -714,26 +764,32 @@ impl DeviceFreeMap {
         &self,
         cluster_segments: &BTreeSet<SlotNumber>,
     ) -> u64 {
-        let pairs_inside_the_cluster_segments: u64 = cluster_segments
-            .iter()
-            .map(|segment_start| {
-                let offset_in_slots = segment_start
-                    .0
-                    .checked_sub(self.unit_area_start.slot().0)
-                    .expect("聚簇段在单元区里：它是各盘一致答出来的全空段起点");
-                assert!(
-                    offset_in_slots.is_multiple_of(CLUSTER_SEGMENT_SLOTS),
-                    "聚簇段起点在段边界上：{segment_start:?}"
-                );
-                let segment = usize::try_from(offset_in_slots / CLUSTER_SEGMENT_SLOTS)
-                    .expect("段号装得进 usize");
-                *self
-                    .unblocked_slot_pairs_per_segment
-                    .get(segment)
-                    .expect("聚簇段在这块盘的单元区里：各盘一致答出来的整段")
-            })
-            .sum();
-        self.unblocked_slot_pairs - pairs_inside_the_cluster_segments
+        self.unblocked_slot_pairs
+            - pairs_inside_the_cluster_segments(
+                self.unit_area_start,
+                &self.unblocked_slot_pairs_per_segment,
+                cluster_segments,
+            )
+    }
+
+    /// defer、影子账隔离、抬 F 扣住三样全放开之后用户数据落得下的槽对数：两槽都不罩在未释放分配记录下、起点偶数、不在 `cluster_segments` 里任何一段
+    /// （抬 F 之前先算槽对那一判的上界，D16（发布语义） 已定项 10 射程「抬 F 之前先算」）。不问这一串的上限够不够得到 defer 里的那几槽、
+    /// 也不问推串时聚簇段会不会多开：只会比真放开之后成的对数多，不会少。读法同 [`Self::unblocked_slot_pairs_outside_the_cluster_segments`]，
+    /// O(段数)、不扫单元区（准入是运行时决策路径）。
+    ///
+    /// # Panics
+    /// 同 [`Self::unblocked_slot_pairs_outside_the_cluster_segments`]。
+    #[must_use]
+    pub fn slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
+        &self,
+        cluster_segments: &BTreeSet<SlotNumber>,
+    ) -> u64 {
+        self.slot_pairs_without_an_unreleased_record
+            - pairs_inside_the_cluster_segments(
+                self.unit_area_start,
+                &self.slot_pairs_without_an_unreleased_record_per_segment,
+                cluster_segments,
+            )
     }
 
     #[must_use]
@@ -783,6 +839,9 @@ impl DeviceFreeMap {
         for index in start..end {
             let segment = index / slots_per_segment;
             let is_a_full_segment = (segment + 1) * slots_per_segment <= self.allocated.len();
+            self.note_one_slot_under_an_unreleased_record_in_the_slot_pair_counts(
+                index, segment, transition,
+            );
             match transition {
                 UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord => {
                     if self.unreleased_per_segment[segment] == 0 && is_a_full_segment {
@@ -802,6 +861,43 @@ impl DeviceFreeMap {
         }
     }
 
+    /// 位图下标 `index` 所在的那一对（从偶数下标起）两槽都不罩在未释放分配记录下；单元区末尾凑不成一对的那一槽答否。
+    fn is_pair_without_an_unreleased_record_at_index(&self, index: usize) -> bool {
+        let first_index_of_the_pair = index - index % 2;
+        first_index_of_the_pair + 1 < self.is_under_an_unreleased_record.len()
+            && !self.is_under_an_unreleased_record[first_index_of_the_pair]
+            && !self.is_under_an_unreleased_record[first_index_of_the_pair + 1]
+    }
+
+    /// 一槽进出未释放分配记录：改它那一位，它所在那一对在「两槽都不罩在未释放记录下」上翻过去时，那一段的数与总数跟着加一或减一。
+    fn note_one_slot_under_an_unreleased_record_in_the_slot_pair_counts(
+        &mut self,
+        index: usize,
+        segment: usize,
+        transition: UnreleasedRecordTransition,
+    ) {
+        let pair_was_without_an_unreleased_record =
+            self.is_pair_without_an_unreleased_record_at_index(index);
+        self.is_under_an_unreleased_record[index] = match transition {
+            UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord => true,
+            UnreleasedRecordTransition::SlotsLeaveTheirUnreleasedRecord => false,
+        };
+        match (
+            pair_was_without_an_unreleased_record,
+            self.is_pair_without_an_unreleased_record_at_index(index),
+        ) {
+            (true, false) => {
+                self.slot_pairs_without_an_unreleased_record_per_segment[segment] -= 1;
+                self.slot_pairs_without_an_unreleased_record -= 1;
+            }
+            (false, true) => {
+                self.slot_pairs_without_an_unreleased_record_per_segment[segment] += 1;
+                self.slot_pairs_without_an_unreleased_record += 1;
+            }
+            (true, true) | (false, false) => {}
+        }
+    }
+
     /// 把一个仍分配的落点放进 defer 队列：槽仍占着（分配器不许再发它），只是记账上从「仍分配」挪到「待释放」；
     /// 记录改写成已释放，按记录口径它的槽从此不罩在未释放的记录下（[`Self::empty_segments`]）。
     pub fn mark_released(&mut self, slot: SlotNumber, span: u64) {
@@ -3759,6 +3855,197 @@ mod tests {
         );
     }
 
+    /// 按记录逐槽扫出来的「defer、影子账隔离、抬 F 扣住三样全放开之后段外成的槽对数」：一槽罩在未释放的分配记录下就放不开，
+    /// 别的（空闲的、罩在已释放记录下的、隔离或扣住而没有未释放记录罩着的）都放得开；只读记录表、跨度与聚簇段集合，
+    /// 不读增量维护的逐槽位与计数。
+    fn slot_pairs_once_everything_is_released_by_the_records(
+        pool: &PoolAllocator,
+        device_map: &DeviceFreeMap,
+    ) -> u64 {
+        let first_slot = device_map.unit_area_start().slot().0;
+        let past_the_last_slot = first_slot + device_map.unit_area_slots();
+        let is_under_an_unreleased_record_by_the_records = |slot: u64| {
+            pool.records().iter().any(|record| {
+                record.device == device_map.device
+                    && !record.is_released
+                    && slot >= record.slot.0
+                    && slot < record.slot.0 + u64::from(record.span_slots)
+            })
+        };
+        u64::try_from(
+            (first_slot..past_the_last_slot)
+                .step_by(2)
+                .filter(|pair_start| {
+                    pair_start + 1 < past_the_last_slot
+                        && !is_under_an_unreleased_record_by_the_records(*pair_start)
+                        && !is_under_an_unreleased_record_by_the_records(pair_start + 1)
+                        && !pool.cluster_segments().iter().any(|segment_start| {
+                            *pair_start >= segment_start.0
+                                && *pair_start < segment_start.0 + CLUSTER_SEGMENT_SLOTS
+                        })
+                })
+                .count(),
+        )
+        .expect("槽对数装得进 u64")
+    }
+
+    /// 真把三样都放开之后段外没挡的槽对数：在一份拷贝上回收全部已释放的、清掉每一槽的影子账隔离、放开抬 F 扣住，再读没挡的段外槽对。
+    fn slot_pairs_after_really_releasing_everything(
+        pool: &PoolAllocator,
+        device: DeviceIdentity,
+    ) -> u64 {
+        let mut released = pool.clone();
+        released.reclaim_released_up_to(CheckpointTxg(u64::MAX), ReclaimedReuse::Immediately);
+        released.release_reclaim_holds();
+        let cluster_segments = released.cluster_segments().clone();
+        let device_map = released
+            .devices
+            .iter_mut()
+            .find(|device_map| device_map.device == device)
+            .expect("盘在池里");
+        let first_slot = device_map.unit_area_start().slot().0;
+        for slot in first_slot..first_slot + device_map.unit_area_slots() {
+            device_map.clear_isolation_of_slot(SlotNumber(slot));
+        }
+        device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
+    }
+
+    /// 一串定种子的伪随机分配、开段、释放、回收、抬 F 扣住与放开、影子账隔离与清隔离、管理员回退复活（两块盘，单元区 385 槽：
+    /// 末尾一槽不成对、末尾一段不满 64 槽），每一步之后把分配器交给 `check_after_the_step`（步号与这一步做了什么）。
+    fn random_history_of_the_allocator(check_after_the_step: &dyn Fn(&PoolAllocator, usize, &str)) {
+        let device_bytes = (UNIT_AREA_START_SLOT + 385) * SLOT_BYTES;
+        let mut pool = PoolAllocator::new(vec![
+            DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
+            DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
+        ]);
+        let mut random_state: u64 = 0x9E37_79B9_7F4A_7C15;
+        let mut next_random = |below: u64| {
+            random_state ^= random_state << 13;
+            random_state ^= random_state >> 7;
+            random_state ^= random_state << 17;
+            random_state % below
+        };
+        let mut generation = 1u64;
+        let mut placements_allocated: Vec<Placement> = Vec::new();
+        let mut placements_in_the_defer_queue: Vec<(Placement, CheckpointTxg)> = Vec::new();
+        check_after_the_step(&pool, 0, "空盘");
+        for step_number in 1..=600 {
+            let what_the_step_did = match next_random(9) {
+                0 | 1 => {
+                    if let Some(placement) = pool.allocate_user_data(CheckpointTxg(generation)) {
+                        placements_allocated.push(placement);
+                    }
+                    "分配一个数据单元"
+                }
+                2 => {
+                    let footprint = if next_random(2) == 0 {
+                        UnitFootprint::OneSlot
+                    } else {
+                        UnitFootprint::TwoSlotsAligned
+                    };
+                    if let Some(placement) =
+                        pool.allocate_commit_generated(footprint, CheckpointTxg(generation))
+                    {
+                        placements_allocated.push(placement);
+                    }
+                    "开段或 bump 一个提交内生块"
+                }
+                3 | 4 => {
+                    if !placements_allocated.is_empty() {
+                        let position = usize::try_from(next_random(
+                            u64::try_from(placements_allocated.len()).expect("个数"),
+                        ))
+                        .expect("下标");
+                        let placement = placements_allocated.swap_remove(position);
+                        generation += 1;
+                        pool.release(placement, CheckpointTxg(generation));
+                        placements_in_the_defer_queue.push((placement, CheckpointTxg(generation)));
+                    }
+                    "释放一个落点进 defer"
+                }
+                5 => {
+                    let reuse = if next_random(2) == 0 {
+                        ReclaimedReuse::Immediately
+                    } else {
+                        ReclaimedReuse::HeldUntilFloorTakesEffect
+                    };
+                    pool.reclaim_released_up_to(CheckpointTxg(generation), reuse);
+                    placements_in_the_defer_queue.clear();
+                    "回收 defer 里的全部（立即可用或抬 F 扣住）"
+                }
+                6 => {
+                    pool.release_reclaim_holds();
+                    "抬 F 生效、放开扣住"
+                }
+                7 => {
+                    let slot = SlotNumber(UNIT_AREA_START_SLOT + next_random(384));
+                    if next_random(2) == 0 {
+                        pool.isolate_abandoned(DeviceIdentity(0), slot, 1 + next_random(2));
+                    } else {
+                        pool.devices[0].clear_isolation_of_slot(slot);
+                    }
+                    "盘 0 隔离或清隔离"
+                }
+                _ => {
+                    if let Some((placement, _)) = placements_in_the_defer_queue.pop() {
+                        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+                            pool.resurrect_released_record(
+                                device,
+                                placement,
+                                CheckpointTxg(generation),
+                            );
+                        }
+                        placements_allocated.push(placement);
+                    }
+                    "管理员回退复活一个 defer 里的落点"
+                }
+            };
+            check_after_the_step(&pool, step_number, what_the_step_did);
+        }
+    }
+
+    /// 抬 F 之前先算槽对那一判的上界读的数（D16（发布语义） 已定项 10 射程「抬 F 之前先算」）是增量维护的：
+    /// 一串随机操作的每一步之后，拿按记录逐槽扫出来的同一个量对，两块盘都要对得上。
+    /// 判别力：释放时不清那一槽的「罩在未释放记录下」位，defer 里的槽就少算。
+    #[test]
+    fn slot_pairs_once_everything_is_released_are_maintained_incrementally_and_match_a_scan_of_the_records(
+    ) {
+        random_history_of_the_allocator(&|pool, step_number, what_the_step_did| {
+            for device_map in &pool.devices {
+                assert_eq!(
+                    device_map.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
+                        pool.cluster_segments()
+                    ),
+                    slot_pairs_once_everything_is_released_by_the_records(pool, device_map),
+                    "第 {step_number} 步（{what_the_step_did}）：盘 {:?} 的增量数与按记录逐槽扫的相同",
+                    device_map.device
+                );
+            }
+        });
+    }
+
+    /// 上界不许算小（算小了会误拒一个推了本来写得进的写）：一串随机操作的每一步之后，增量维护的数不小于在拷贝上真把 defer、隔离、扣住都放开之后
+    /// 段外没挡的槽对数。
+    #[test]
+    fn slot_pairs_once_everything_is_released_never_fall_below_the_pairs_after_really_releasing_everything(
+    ) {
+        random_history_of_the_allocator(&|pool, step_number, what_the_step_did| {
+            for device_map in &pool.devices {
+                let upper_bound = device_map
+                    .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
+                        pool.cluster_segments(),
+                    );
+                let really_released =
+                    slot_pairs_after_really_releasing_everything(pool, device_map.device);
+                assert!(
+                    upper_bound >= really_released,
+                    "第 {step_number} 步（{what_the_step_did}）：盘 {:?} 上界 {upper_bound} 小于真放开之后的 {really_released}",
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
index 4370897..6c920d1 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -4,7 +4,7 @@
 //! 先推一串抬 F 的空发布（`mount::push_one_floor_raise_within_the_admission_budget`：F 抬到准入上限；上限卡在挂载时读坏或读回全零的槽上时
 //! 推一次不抬 F 的空发布，D16（发布语义） 已定项 1「抬 F 的上限」那一行；一次准入最多 `mount::PUBLISHES_PER_ADMISSION_AT_MOST` 次发布，
 //! 两种推的都算、再算上这次用户发布），再重判；做满仍不够、或抬不动了，才报空间不够。推抬 F 放得出的全放出来也放不下时不推，当场报空间不够
-//! （`UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`）。
+//! （`UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`）；推串之前全空聚簇段已少于 `N₀`、被拒的又是前台写入时同样不推、同一个成员。
 //! 写行那次发布之前不推：会话在可写挂载交回之后才有，写行那次早已发完（可写挂载那一处的推归 `mount::establish_instance`）。
 //!
 //! 为什么要这一层：发布路径（`transaction::publish_overwrite` 这一族）只拿到一个写入口（`PoolWriter`），拿不到抬 F 要的设备表与参数
@@ -31,8 +31,10 @@ use crate::address::{
     InodeNumber, InstanceGeneration, SlotNumber, TreeIdentifier,
 };
 use crate::admission::{
-    AdmissionRefusedOnSomeDevices, AvailableBytesOnOneDevice, BytesOnOneDevice,
-    DeviceShortOfDemand, JudgementThatFoundTheDeviceShort,
+    admit_the_empty_cluster_segments_floor_on_every_device, AdmissionRefusedOnSomeDevices,
+    AvailableBytesOnOneDevice, BytesOnOneDevice, DeviceShortOfDemand,
+    EmptyClusterSegmentsBelowTheFloorOnSomeDevices, JudgementThatFoundTheDeviceShort,
+    SpaceAdmission,
 };
 use crate::allocator::{
     AllocationRecord, DeviceFreeMap, Placement, PlacementOnDevice, PoolAllocator,
@@ -681,13 +683,18 @@ pub struct NoSpaceAfterRaisingTheFloor {
     pub wide_protection_at_the_ceiling: WideProtectionAtTheCeiling,
 }
 
-/// 推抬 F 放得出的全放出来也放不下：`devices_short` 里每块盘的上界都小于需求，所以这一次不推（D16（发布语义） 已定项 1「准入」那一行的射程：
-/// 推了也写不进的写不花根环的槽与写盘）。别的被拒成员一样带着这次调用已经落盘的：`floor_raises` 与 `refusals_that_pushed_the_floor_raises`
+/// 推抬 F 放得出的全放出来也放不下：`devices_short` 里每块盘的上界都小于需求，或这次调用推串之前全空聚簇段已少于 `N₀`、被拒的又是合取 7 判的前台写入
+/// （`empty_cluster_segments_below_the_floor`），所以这一次不推（D16（发布语义） 已定项 1「准入」那一行的射程：
+/// 推了也写不进的写不花根环的槽与写盘；已定项 10 射程「抬 F 之前先算」）。别的被拒成员一样带着这次调用已经落盘的：`floor_raises` 与 `refusals_that_pushed_the_floor_raises`
 /// 一一对应，`versions_published` 是推的抬 F 之外发成的那几版（[`VersionsPublishedInThisChange`]）。
 #[derive(Debug)]
 pub struct NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {
+    /// 上界小于需求的盘；先判它们，判出了 `empty_cluster_segments_below_the_floor` 就是 `None`。
     pub devices_short: Vec<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease>,
-    /// 判出这几块盘的那一次被拒（`SpaceAdmissionRefused`）：这一次发布没有推抬 F。
+    /// 没有上界小于需求的盘、合取 7 那一判在推串之前判出时：`N₀` 与全空聚簇段少于它的每一块盘（合取 7 自己报的那一份）；这时 `devices_short` 是空的。
+    pub empty_cluster_segments_below_the_floor:
+        Option<EmptyClusterSegmentsBelowTheFloorOnSomeDevices>,
+    /// 判出这几块盘的那一次被拒（`SpaceAdmissionRefused`，合取 7 那一判判出时也可以是 `PlacementRefused`）：这一次发布没有推抬 F。
     pub last_refusal: PublishError,
     pub floor_raises: Vec<RaisedFloor>,
     pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
@@ -3835,6 +3842,7 @@ impl MountedSession {
                     version_to_change,
                     shadow_ledger,
                     RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
+                    RefusedPublishUnderTheEmptyClusterSegmentsConjunct::of_the_user_change(change),
                     publishes_pushed,
                     published_in_this_change,
                     refusal,
@@ -6041,26 +6049,27 @@ struct SlotsAFloorRaiseCouldRelease {
     deferred: u64,
     /// 隔离着的槽（式子第九项）：被抛弃根离开根环、C 那一圈解开时回来，推环这条出路也靠它们（[`StuckBehindTheWideProtection`]）。
     isolated: u64,
-    /// 计数上空闲而位上挡着的槽：隔离着没分配的与抬 F 扣住的（`DeviceFreeMap::hold_until_floor_takes_effect`）。
-    /// 式子本来不扣扣住的槽，落得下那一判里它们挡着槽对。
-    free_but_blocked: u64,
+    /// 这次挂载开过的聚簇段之外、两槽都不罩在未释放分配记录下的偶数起点槽对：defer、影子账隔离、抬 F 扣住三样全放开之后数据单元落得下的槽对，
+    /// 现有没挡的槽对也在里面（`DeviceFreeMap::slot_pairs_without_an_unreleased_record_outside_the_cluster_segments`，分配器按段增量维护）。
+    /// defer 里一个数据单元的两槽放开只成一对，另一半罩在未释放记录下的槽放开了也不成对。
+    slot_pairs_once_everything_is_released: u64,
 }
 
 impl SlotsAFloorRaiseCouldRelease {
-    fn of_the_device(device_map: &DeviceFreeMap) -> Self {
+    fn of_the_device(device_map: &DeviceFreeMap, cluster_segments: &BTreeSet<SlotNumber>) -> Self {
         Self {
             deferred: device_map.deferred_slots(),
             isolated: device_map.isolated_slots(),
-            free_but_blocked: device_map
-                .free_slots()
-                .checked_sub(device_map.unblocked_slots())
-                .expect("没挡的槽都没置分配位，分配位没置的都算空闲：没挡的槽数不大于空闲槽数"),
+            slot_pairs_once_everything_is_released: device_map
+                .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
+                    cluster_segments,
+                ),
         }
     }
 }
 
 /// 一块被判短的盘推抬 F 至多凑得出多少，按判它短的那一判自己的量（宁大勿小：上界算小了会误拒一个推了本来写得进的写）：
-/// 式子那一判是可用加 defer 与隔离的字节；段外槽对那一判是现有的槽对加每个放得开的槽各补成一对（另一半本来就空着时放开一个槽就多一对，所以按一个数据单元的槽数换算）。
+/// 式子那一判是可用加 defer 与隔离的字节；段外槽对那一判是三样全放开之后段外成的槽对（现有的在里面，不再另加可用），按一个数据单元的槽数换算。
 /// 提交内生块那一判交 `None`：这一判数的槽抬 F 那一串自己也要占，不在这里算。
 fn most_a_floor_raise_could_make_available(
     short: &DeviceShortOfDemand,
@@ -6074,14 +6083,11 @@ fn most_a_floor_raise_could_make_available(
             ))
         }
         JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits => {
-            let slots_that_could_complete_a_pair =
-                slots.deferred + slots.isolated + slots.free_but_blocked;
             let slots_of_a_data_unit =
                 TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots();
-            Some(AvailableBytesOnOneDevice(
-                short.available.0
-                    + bytes_of_slots(slots_that_could_complete_a_pair * slots_of_a_data_unit),
-            ))
+            Some(AvailableBytesOnOneDevice(bytes_of_slots(
+                slots.slot_pairs_once_everything_is_released * slots_of_a_data_unit,
+            )))
         }
         JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,
     }
@@ -6122,15 +6128,134 @@ fn devices_short_even_with_everything_a_floor_raise_could_release(
                 .expect("拒绝里的盘来自这个分配器：准入的两份读数都按它的盘建");
             device_short_even_with_everything_a_floor_raise_could_release(
                 short,
-                SlotsAFloorRaiseCouldRelease::of_the_device(device_map),
+                SlotsAFloorRaiseCouldRelease::of_the_device(
+                    device_map,
+                    allocator.cluster_segments(),
+                ),
             )
         })
         .collect()
 }
 
-/// 全放出来也放不下的那个成员，把这次调用已经落盘的都带上。
+/// 被拒的那一次发布是不是合取 7（`全空聚簇段数 ≥ N₀`，D3（空间分配） 已定项 7 登记表第 7 行）一定判的前台写入类：抬 F 之前先算的合取 7 那一判
+/// （[`empty_cluster_segments_below_the_floor_before_any_series`]）只对这一种判。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum RefusedPublishUnderTheEmptyClusterSegmentsConjunct {
+    /// 写文件内容、建 inode、并进一次发布的数据写类：带普通分配的前台写入类，合取 7 判它（`transaction::reach_of_the_empty_cluster_segments_conjunct`）。
+    ForegroundWriteTheConjunctJudges,
+    /// 删除类（unlink、删快照的意图，走空里的每一步）免合取 7；改 size 发之前分不出是截断的意图（免）还是变长与只改记录（判），
+    /// 建快照带不带普通分配发之前不知道：这几种都不先判，照推。
+    ExemptOrNotKnownBeforeThePublish,
+}
+
+impl RefusedPublishUnderTheEmptyClusterSegmentsConjunct {
+    /// 会话发这一个改动、被拒的就是这个改动自己那一次发布时（不是先走空里的一步）。
+    fn of_the_user_change(change: UserChange<'_>) -> Self {
+        match change {
+            UserChange::Overwrite(_)
+            | UserChange::SequentialWrite(_)
+            | UserChange::NewInodes { .. }
+            | UserChange::OverwriteOfAnInode { .. }
+            | UserChange::SequentialWriteOfAnInode { .. }
+            | UserChange::WriteAtOffset { .. }
+            | UserChange::DataChangesPublishedTogether(_) => Self::ForegroundWriteTheConjunctJudges,
+            UserChange::Unlink { .. }
+            | UserChange::DestroySnapshot { .. }
+            | UserChange::Truncate { .. }
+            | UserChange::CreateSnapshot => Self::ExemptOrNotKnownBeforeThePublish,
+        }
+    }
+}
+
+/// 抬 F 之前先算的合取 7 那一判（D16（发布语义） 已定项 10 射程「抬 F 之前先算」）：这次调用还一串都没推（`series_pushed_in_this_change` 是 0）、
+/// 被拒是空间不够的那两种（[`refusal_is_short_of_space`]）、被拒的是合取 7 判的前台写入类、准入判着时，读每块盘此刻的全空聚簇段数
+/// （`DeviceFreeMap::empty_segments`，合取 7 自己读的那一个）与整理开关里的 `N₀`，照合取 7 同一个判法
+/// （[`admit_the_empty_cluster_segments_floor_on_every_device`]）判，有一块少于 `N₀` 就交回那几块盘。判的是写之前那一刻的数，不另估这次写之后：
+/// 抬 F 回收 defer 里的槽不改这个数（defer 里的已释放槽本来就不算占用），推了这次写也是被合取 7 拒。只读分配器（内存）。
+fn empty_cluster_segments_below_the_floor_before_any_series(
+    allocator: &PoolAllocator,
+    refusal: &PublishError,
+    refused_publish: RefusedPublishUnderTheEmptyClusterSegmentsConjunct,
+    series_pushed_in_this_change: usize,
+) -> Option<EmptyClusterSegmentsBelowTheFloorOnSomeDevices> {
+    let is_judged_by_the_conjunct = match refused_publish {
+        RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges => {
+            true
+        }
+        RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ExemptOrNotKnownBeforeThePublish => {
+            false
+        }
+    };
+    let is_admission_judged = match allocator.space_admission() {
+        SpaceAdmission::JudgedByTheFormula => true,
+        SpaceAdmission::SkippedByTheTestOnlySwitch => false,
+    };
+    if series_pushed_in_this_change > 0
+        || !refusal_is_short_of_space(refusal)
+        || !is_judged_by_the_conjunct
+        || !is_admission_judged
+    {
+        return None;
+    }
+    let empty_cluster_segments_on_each_device: Vec<(DeviceIdentity, u64)> = allocator
+        .devices
+        .iter()
+        .map(|device_map| (device_map.device, device_map.empty_segments()))
+        .collect();
+    admit_the_empty_cluster_segments_floor_on_every_device(
+        &empty_cluster_segments_on_each_device,
+        allocator
+            .compaction()
+            .switches
+            .empty_cluster_segments_floor
+            .segments(),
+    )
+    .err()
+}
+
+/// 抬 F 之前先算（D16（发布语义） 已定项 10 射程「抬 F 之前先算」）：推串的三处（[`MountedSession::publish_user_change`]、
+/// [`MountedSessionWithPendingChanges::accept`]、走空）都经 [`push_one_floor_raise_for_a_refusal_for_space`] 在推每一串之前走这一段。
+/// 推抬 F 放得出的全放出来也放不下的盘（[`devices_short_even_with_everything_a_floor_raise_could_release`]）先判，判出就交回它们；
+/// 没判出、这次调用还一串没推而全空聚簇段已少于 `N₀`、被拒的又是合取 7 判的前台写入（[`empty_cluster_segments_below_the_floor_before_any_series`]）的，
+/// 交回那几块盘的全空段数。两种都是 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，带上这次调用已经落盘的；
+/// 都没判出原样交回已落盘的与这一次被拒，由调用方接着推。只读分配器（内存），一个盘都不读、一个写都不发。
+fn judge_before_pushing_a_floor_raise_for_a_refusal(
+    allocator: &PoolAllocator,
+    refused_publish: RefusedPublishUnderTheEmptyClusterSegmentsConjunct,
+    published_in_this_change: PublishedInThisChange,
+    refusal: PublishError,
+) -> Result<(PublishedInThisChange, PublishError), UserChangeRefused> {
+    let devices_short =
+        devices_short_even_with_everything_a_floor_raise_could_release(allocator, &refusal);
+    if !devices_short.is_empty() {
+        return Err(no_space_even_with_everything_a_floor_raise_could_release(
+            devices_short,
+            None,
+            refusal,
+            published_in_this_change,
+        ));
+    }
+    if let Some(below_the_floor) = empty_cluster_segments_below_the_floor_before_any_series(
+        allocator,
+        &refusal,
+        refused_publish,
+        published_in_this_change.floor_raises.len(),
+    ) {
+        return Err(no_space_even_with_everything_a_floor_raise_could_release(
+            Vec::new(),
+            Some(below_the_floor),
+            refusal,
+            published_in_this_change,
+        ));
+    }
+    Ok((published_in_this_change, refusal))
+}
+
+/// 全放出来也放不下的那个成员，把这次调用已经落盘的都带上。判出它的是放不下的盘（`devices_short` 非空、`empty_cluster_segments_below_the_floor` 是 `None`）
+/// 或推串之前合取 7 那一判（`devices_short` 空、`empty_cluster_segments_below_the_floor` 是 `Some`）。
 fn no_space_even_with_everything_a_floor_raise_could_release(
     devices_short: Vec<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease>,
+    empty_cluster_segments_below_the_floor: Option<EmptyClusterSegmentsBelowTheFloorOnSomeDevices>,
     last_refusal: PublishError,
     published_in_this_change: PublishedInThisChange,
 ) -> UserChangeRefused {
@@ -6142,6 +6267,7 @@ fn no_space_even_with_everything_a_floor_raise_could_release(
     UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(Box::new(
         NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {
             devices_short,
+            empty_cluster_segments_below_the_floor,
             last_refusal,
             floor_raises,
             refusals_that_pushed_the_floor_raises,
@@ -6151,7 +6277,8 @@ fn no_space_even_with_everything_a_floor_raise_could_release(
 }
 
 /// 一次发布被拒之后会话怎么接（D16（发布语义） 已定项 1「准入」那一行）：不是空间不够的那两种（[`refusal_is_short_of_space`]）原样交回 `Publish`；
-/// 推抬 F 放得出的全放出来也放不下（[`devices_short_even_with_everything_a_floor_raise_could_release`]）交回 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，一个写都不发；
+/// 抬 F 之前先算判出放不下（[`judge_before_pushing_a_floor_raise_for_a_refusal`]，`refused_publish` 说被拒的那一次归不归合取 7 管）交回
+/// `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，一个写都不发；
 /// 别的就为被拒的那一次发布推一串抬 F 的空发布（这次准入已推 `publishes_pushed_in_this_admission` 次，预算照
 /// `mount::push_one_floor_raise_within_the_admission_budget`），推成交回记上这一串之后的 `published_in_this_change` 与这一串推了几次空发布，
 /// 不再推交回 `NoSpaceAfterRaisingTheFloor`（报不报「被宽护卡住」，报的话记下这次改动剩下的 `remainder`，要出路时照它预演，
@@ -6160,7 +6287,7 @@ fn no_space_even_with_everything_a_floor_raise_could_release(
 /// （[`Arc::make_mut`]），版本列表里那一份不动。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、影子账那一臂、这次改动还剩什么、预算已用的数、已落盘的、这一次被拒，不成一个概念"
+    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、影子账那一臂、这次改动还剩什么、被拒的那一次归不归合取 7 管、预算已用的数、已落盘的、这一次被拒，不成一个概念"
 )]
 fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
@@ -6169,19 +6296,17 @@ fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
     current: &mut Arc<TransactionOutput>,
     shadow_ledger: ShadowLedger,
     remainder: RemainderOfTheUserChange<'_>,
+    refused_publish: RefusedPublishUnderTheEmptyClusterSegmentsConjunct,
     publishes_pushed_in_this_admission: usize,
     published_in_this_change: PublishedInThisChange,
     refusal: PublishError,
 ) -> Result<(PublishedInThisChange, usize), UserChangeRefused> {
-    let devices_short =
-        devices_short_even_with_everything_a_floor_raise_could_release(allocator, &refusal);
-    if !devices_short.is_empty() {
-        return Err(no_space_even_with_everything_a_floor_raise_could_release(
-            devices_short,
-            refusal,
-            published_in_this_change,
-        ));
-    }
+    let (published_in_this_change, refusal) = judge_before_pushing_a_floor_raise_for_a_refusal(
+        allocator,
+        refused_publish,
+        published_in_this_change,
+        refusal,
+    )?;
     let PublishedInThisChange {
         mut floor_raises,
         mut refusals_that_pushed_the_floor_raises,
@@ -6295,6 +6420,8 @@ fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<Device: Block
                 current,
                 shadow_ledger,
                 remainder,
+                // 走空里被拒的是删除类的一步，合取 7 免。
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ExemptOrNotKnownBeforeThePublish,
                 publishes_pushed_in_this_admission,
                 published_in_this_change,
                 refusal,
@@ -7100,16 +7227,18 @@ impl MountedSessionWithPendingChanges {
             }
             let (whole_write_together, new_inodes_together) =
                 self.pending.together_with_this_change(data_change);
-            let remainder =
-                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(match judgement {
-                    BatchJudgedAtTheAccept::ThisChangeAlone => change,
-                    BatchJudgedAtTheAccept::PendingChangesTogetherWithThisChange => {
-                        UserChange::DataChangesPublishedTogether(DataChangesPublishedTogether {
-                            whole_write: whole_write_together,
-                            new_inodes: &new_inodes_together,
-                        })
-                    }
-                });
+            let change_of_the_refused_batch = match judgement {
+                BatchJudgedAtTheAccept::ThisChangeAlone => change,
+                BatchJudgedAtTheAccept::PendingChangesTogetherWithThisChange => {
+                    UserChange::DataChangesPublishedTogether(DataChangesPublishedTogether {
+                        whole_write: whole_write_together,
+                        new_inodes: &new_inodes_together,
+                    })
+                }
+            };
+            let remainder = RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(
+                change_of_the_refused_batch,
+            );
             let MountedSession {
                 allocator,
                 current,
@@ -7130,6 +7259,10 @@ impl MountedSessionWithPendingChanges {
                 &mut version_pushed,
                 *shadow_ledger,
                 remainder,
+                // `accept` 不先走空（队列不空时在任何读写之前拒），被拒的就是判的那一批自己。
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::of_the_user_change(
+                    change_of_the_refused_batch,
+                ),
                 publishes_pushed_in_this_admission,
                 published_in_this_change,
                 cause,
@@ -8288,9 +8421,13 @@ mod tests {
 #[cfg(test)]
 mod everything_a_floor_raise_could_release_tests {
     use super::*;
+    use crate::admission::{
+        admit_the_units_landing_on_every_device, PlacementRoomOnOneDevice, UnitsOfAPublishToLand,
+    };
+    use crate::allocator::ReclaimedReuse;
     use crate::mount::FloorOfThePushedPublishes;
     use crate::write_accounting::WritesByStructureKind;
-    use singlefs_format::SLOT_BYTES;
+    use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};
 
     const DEVICE_OF_THE_TEST: DeviceIdentity = DeviceIdentity(0);
 
@@ -8298,7 +8435,7 @@ mod everything_a_floor_raise_could_release_tests {
         SlotsAFloorRaiseCouldRelease {
             deferred: 0,
             isolated: 0,
-            free_but_blocked: 0,
+            slot_pairs_once_everything_is_released: 0,
         }
     }
 
@@ -8326,7 +8463,7 @@ mod everything_a_floor_raise_could_release_tests {
         SlotsAFloorRaiseCouldRelease {
             deferred: 2,
             isolated: 1,
-            free_but_blocked: 0,
+            slot_pairs_once_everything_is_released: 0,
         }
     }
 
@@ -8382,71 +8519,381 @@ mod everything_a_floor_raise_could_release_tests {
                 SlotsAFloorRaiseCouldRelease {
                     deferred: 0,
                     isolated: 0,
-                    free_but_blocked: 8,
+                    slot_pairs_once_everything_is_released: 8,
                 },
             )
             .is_some(),
-            "扣住的槽式子本来不扣：加进上界就多算"
+            "段外放开之后成的槽对是落得下那一判的量，式子不扣它：加进上界就多算"
         );
     }
 
-    /// 段外槽对那一判：一个槽对都没有，需求一个数据单元，这一样放得开的只有一个槽：放开它就可能多一对。
-    fn one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(
-        slots: SlotsAFloorRaiseCouldRelease,
-    ) -> bool {
-        let bytes_of_a_data_unit = BytesOnOneDevice::of_slots(
-            TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots(),
+    /// 单元区恰好一个 64 槽聚簇段的一块盘：32 个槽对。
+    fn allocator_of_one_cluster_segment() -> PoolAllocator {
+        PoolAllocator::new(vec![DeviceFreeMap::new(
+            DEVICE_OF_THE_TEST,
+            (UNIT_AREA_START_SLOT + CLUSTER_SEGMENT_SLOTS) * SLOT_BYTES,
+        )])
+    }
+
+    /// 把这块盘的段外槽对分完（数据单元落点按次序取最低的偶数槽对），交回分到的落点，按槽号。
+    fn allocate_every_slot_pair(allocator: &mut PoolAllocator) -> Vec<Placement> {
+        // 迭代上界：每一轮分走一个槽对，单元区只有 32 对。
+        let mut placements = Vec::new();
+        while let Some(placement) = allocator.allocate_user_data(CheckpointTxg(1)) {
+            placements.push(placement);
+        }
+        assert_eq!(placements.len(), 32, "一段 64 槽分成 32 个数据单元");
+        placements
+    }
+
+    /// 落得下那一判（`admit_the_units_landing_on_every_device`，发布路径判的同一个函数）对这块盘此刻的分配器判 `data_units` 个数据单元、
+    /// `commit_generated_slots` 个提交内生块槽，判不过交回发布路径交回的那一种被拒。
+    fn refusal_of_the_landing_judgement(
+        allocator: &PoolAllocator,
+        data_units: u64,
+        commit_generated_slots: u64,
+    ) -> PublishError {
+        PublishError::SpaceAdmissionRefused(
+            admit_the_units_landing_on_every_device(
+                &PlacementRoomOnOneDevice::of_every_device_of_the_allocator(allocator),
+                UnitsOfAPublishToLand {
+                    data_units,
+                    commit_generated_slots,
+                },
+            )
+            .expect_err("用例造的盘落不下这么多"),
+        )
+    }
+
+    fn bytes_of_slot_pairs(pairs: u64) -> u64 {
+        BytesOnOneDevice::of_slots(
+            pairs * TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots(),
         )
-        .0;
-        device_short_even_with_everything_a_floor_raise_could_release(
-            &short_by(
-                JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
+        .0
+    }
+
+    fn nothing_published_in_this_change() -> PublishedInThisChange {
+        PublishedInThisChange {
+            floor_raises: Vec::new(),
+            refusals_that_pushed_the_floor_raises: Vec::new(),
+            versions_published: Vec::new(),
+        }
+    }
+
+    /// 段外槽对分完、一个数据单元（两槽）进了 defer：放开它只成一对。写两个数据单元，推了也只多一对，先算判放不下、一串都不推。
+    /// 判别力：上界照每个放得开的槽各补一对算（defer 两槽算两对），恰好等于需求，照推。
+    #[test]
+    fn a_deferred_data_unit_completes_one_slot_pair_so_a_write_of_two_data_units_is_not_pushed_for()
+    {
+        let mut allocator = allocator_of_one_cluster_segment();
+        let placements = allocate_every_slot_pair(&mut allocator);
+        allocator.release(placements[7], CheckpointTxg(2));
+        let refusal = refusal_of_the_landing_judgement(&allocator, 2, 0);
+        let Err(UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space)) =
+            judge_before_pushing_a_floor_raise_for_a_refusal(
+                &allocator,
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ExemptOrNotKnownBeforeThePublish,
+                nothing_published_in_this_change(),
+                refusal,
+            )
+        else {
+            panic!("defer 里一个数据单元放开只成一对，两个数据单元推了也放不下：不推");
+        };
+        assert_eq!(
+            (
+                no_space.devices_short,
+                no_space.empty_cluster_segments_below_the_floor.is_none()
+            ),
+            (
+                vec![DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
+                    device: DEVICE_OF_THE_TEST,
+                    judged_short_by:
+                        JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
+                    most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(i128::from(
+                        bytes_of_slot_pairs(1)
+                    )),
+                    demand: BytesOnOneDevice(bytes_of_slot_pairs(2)),
+                }],
+                true
+            ),
+            "点名这块盘：上界一对、需求两对，判出它的是槽对那一判"
+        );
+    }
+
+    /// 段外槽对分完之后，一个数据单元隔离着（回收之后被影子账隔离）、一个抬 F 扣住、一个在 defer 里：三样全放开成三对。
+    fn allocator_whose_slot_pairs_are_blocked_once_by_each_of_the_three() -> PoolAllocator {
+        let mut allocator = allocator_of_one_cluster_segment();
+        let placements = allocate_every_slot_pair(&mut allocator);
+        allocator.release(placements[3], CheckpointTxg(2));
+        allocator.reclaim_released_up_to(CheckpointTxg(2), ReclaimedReuse::Immediately);
+        allocator.isolate_abandoned(DEVICE_OF_THE_TEST, placements[3].slot, placements[3].span);
+        allocator.release(placements[11], CheckpointTxg(3));
+        allocator
+            .reclaim_released_up_to(CheckpointTxg(3), ReclaimedReuse::HeldUntilFloorTakesEffect);
+        allocator.release(placements[20], CheckpointTxg(4));
+        assert_eq!(
+            allocator.devices[0]
+                .unblocked_slot_pairs_outside_the_cluster_segments(allocator.cluster_segments()),
+            0,
+            "三对都挡着：落得下那一判此刻数到 0 对"
+        );
+        allocator
+    }
+
+    /// 上界恰好等于需求算放得下：三样各挡一对、写三个数据单元，照推。
+    /// 判别力：上界只数此刻没挡的槽对（0 对），或漏数 defer、隔离、扣住任一样，都判放不下。
+    #[test]
+    fn slot_pairs_blocked_only_by_the_defer_queue_the_isolation_and_the_floor_hold_are_pushed_for()
+    {
+        let allocator = allocator_whose_slot_pairs_are_blocked_once_by_each_of_the_three();
+        let devices_short = devices_short_even_with_everything_a_floor_raise_could_release(
+            &allocator,
+            &refusal_of_the_landing_judgement(&allocator, 3, 0),
+        );
+        assert!(
+            devices_short.is_empty(),
+            "三样全放开成三对，恰好够三个数据单元：照推，实际点名 {devices_short:?}"
+        );
+    }
+
+    /// 同上，写四个数据单元：三对不够，点名这块盘，上界是三对。
+    #[test]
+    fn slot_pairs_blocked_by_the_three_fall_one_pair_short_of_four_data_units() {
+        let allocator = allocator_whose_slot_pairs_are_blocked_once_by_each_of_the_three();
+        assert_eq!(
+            devices_short_even_with_everything_a_floor_raise_could_release(
+                &allocator,
+                &refusal_of_the_landing_judgement(&allocator, 4, 0),
+            ),
+            vec![DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
+                device: DEVICE_OF_THE_TEST,
+                judged_short_by:
+                    JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
+                most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(i128::from(
+                    bytes_of_slot_pairs(3)
+                )),
+                demand: BytesOnOneDevice(bytes_of_slot_pairs(4)),
+            }],
+            "上界三对、需求四对"
+        );
+    }
+
+    /// 此刻没挡的槽对已经在三样全放开之后的槽对里：段外只剩一对空着、别的都罩在未释放记录下，写两个数据单元，上界一对，不再另加此刻的可用。
+    /// 判别力：上界再加一遍落得下那一判报的可用（一对），恰好等于需求，照推。
+    #[test]
+    fn slot_pairs_free_right_now_are_counted_once_in_the_bound() {
+        let mut allocator = allocator_of_one_cluster_segment();
+        let placements = allocate_every_slot_pair(&mut allocator);
+        allocator.release(placements[5], CheckpointTxg(2));
+        allocator.reclaim_released_up_to(CheckpointTxg(2), ReclaimedReuse::Immediately);
+        assert_eq!(
+            devices_short_even_with_everything_a_floor_raise_could_release(
+                &allocator,
+                &refusal_of_the_landing_judgement(&allocator, 2, 0),
+            ),
+            vec![DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
+                device: DEVICE_OF_THE_TEST,
+                judged_short_by:
+                    JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
+                most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(i128::from(
+                    bytes_of_slot_pairs(1)
+                )),
+                demand: BytesOnOneDevice(bytes_of_slot_pairs(2)),
+            }],
+            "上界一对（空着的那一对）、需求两对"
+        );
+    }
+
+    /// 一个数据单元罩在未释放的记录下（全空聚簇段 0 个，`N₀` 是第一版的 1），被拒的是提交内生块那一判（这一判不算上界，盘那一判判不出）。
+    fn allocator_without_an_empty_cluster_segment_and_its_refusal() -> (PoolAllocator, PublishError)
+    {
+        let mut allocator = allocator_of_one_cluster_segment();
+        allocator
+            .allocate_user_data(CheckpointTxg(1))
+            .expect("空盘分得到一个数据单元");
+        assert_eq!(
+            allocator.devices[0].empty_segments(),
+            0,
+            "唯一那一段罩着一条未释放的记录"
+        );
+        let refusal = refusal_of_the_landing_judgement(&allocator, 0, CLUSTER_SEGMENT_SLOTS);
+        (allocator, refusal)
+    }
+
+    /// 推串之前全空聚簇段已少于 `N₀`、被拒的是前台写入：先算判放不下、一串都不推，报合取 7 那一份。
+    /// 判别力：关掉推串之前判合取 7 那一判，照推。
+    #[test]
+    fn a_foreground_write_refused_while_empty_cluster_segments_are_below_the_floor_is_not_pushed_for(
+    ) {
+        let (allocator, refusal) = allocator_without_an_empty_cluster_segment_and_its_refusal();
+        let Err(UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space)) =
+            judge_before_pushing_a_floor_raise_for_a_refusal(
+                &allocator,
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges,
+                nothing_published_in_this_change(),
+                refusal,
+            )
+        else {
+            panic!("全空聚簇段少于 N₀ 的前台写入推了也被合取 7 拒：不推");
+        };
+        assert_eq!(
+            (
+                no_space.devices_short.len(),
+                no_space.empty_cluster_segments_below_the_floor,
+                no_space.floor_raises.len(),
+            ),
+            (
+                0,
+                Some(EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
+                    floor_in_segments: 1,
+                    short_devices: vec![
+                        crate::admission::DeviceWithEmptyClusterSegmentsBelowTheFloor {
+                            device: DEVICE_OF_THE_TEST,
+                            empty_cluster_segments: 0,
+                        }
+                    ],
+                }),
                 0,
-                bytes_of_a_data_unit,
             ),
-            slots,
-        )
-        .is_none()
+            "没有上界小于需求的盘；合取 7 那一份：N₀ 是 1、这块盘 0 段；一串没推"
+        );
     }
 
+    /// 推串之前合取 7 那一判只拦合取 7 判的前台写入：被拒的是删除类、改 size 或建快照（发之前不知道合取 7 判不判）时照推。
     #[test]
-    fn one_deferred_slot_can_complete_a_slot_pair_for_one_data_unit() {
-        assert!(one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(
-            SlotsAFloorRaiseCouldRelease {
-                deferred: 1,
-                isolated: 0,
-                free_but_blocked: 0,
-            }
-        ));
+    fn a_publish_exempt_from_the_empty_cluster_segments_conjunct_is_pushed_for_below_the_floor() {
+        let (allocator, refusal) = allocator_without_an_empty_cluster_segment_and_its_refusal();
+        assert!(
+            judge_before_pushing_a_floor_raise_for_a_refusal(
+                &allocator,
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ExemptOrNotKnownBeforeThePublish,
+                nothing_published_in_this_change(),
+                refusal,
+            )
+            .is_ok(),
+            "合取 7 免的发布不先判它：照推"
+        );
     }
 
+    /// 推串之前合取 7 那一判只在这次调用还一串没推时判：推过一串之后照推（推完照旧由发布路径判合取 7）。
     #[test]
-    fn one_isolated_slot_can_complete_a_slot_pair_for_one_data_unit() {
-        assert!(one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(
-            SlotsAFloorRaiseCouldRelease {
-                deferred: 0,
-                isolated: 1,
-                free_but_blocked: 0,
-            }
-        ));
+    fn after_a_series_was_pushed_the_empty_cluster_segments_conjunct_is_not_judged_before_the_next()
+    {
+        let (allocator, refusal) = allocator_without_an_empty_cluster_segment_and_its_refusal();
+        assert!(
+            judge_before_pushing_a_floor_raise_for_a_refusal(
+                &allocator,
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges,
+                PublishedInThisChange {
+                    floor_raises: vec![raised_floor_without_publishes()],
+                    refusals_that_pushed_the_floor_raises: vec![refusal_of_the_test()],
+                    versions_published: Vec::new(),
+                },
+                refusal,
+            )
+            .is_ok(),
+            "这次调用已推过一串：不先判合取 7"
+        );
     }
 
+    /// 推串之前合取 7 那一判只对空间不够的那两种被拒判：别的被拒原样交给调用方（它交回 `Publish`），不报空间不够。
     #[test]
-    fn one_free_but_blocked_slot_can_complete_a_slot_pair_for_one_data_unit() {
-        assert!(one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(
-            SlotsAFloorRaiseCouldRelease {
-                deferred: 0,
-                isolated: 0,
-                free_but_blocked: 1,
-            }
-        ));
+    fn a_refusal_not_short_of_space_is_not_judged_against_the_empty_cluster_segments_floor() {
+        let (allocator, _) = allocator_without_an_empty_cluster_segment_and_its_refusal();
+        let refusal_not_short_of_space = PublishError::EmptyClusterSegmentsBelowTheFloor(
+            EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
+                floor_in_segments: 1,
+                short_devices: Vec::new(),
+            },
+        );
+        assert!(
+            judge_before_pushing_a_floor_raise_for_a_refusal(
+                &allocator,
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges,
+                nothing_published_in_this_change(),
+                refusal_not_short_of_space,
+            )
+            .is_ok(),
+            "不是空间不够：原样交回"
+        );
     }
 
+    /// 只供测试的开关关掉准入时合取 7 也不判（`transaction` 判它与式子那一判同一个开关）：推串之前同样不判。
     #[test]
-    fn no_slot_to_release_leaves_a_device_short_on_the_slot_pairs_named() {
-        assert!(!one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(
-            no_slots()
-        ));
+    fn with_the_space_admission_switched_off_the_empty_cluster_segments_floor_is_not_judged_before_pushing(
+    ) {
+        let (mut allocator, refusal) = allocator_without_an_empty_cluster_segment_and_its_refusal();
+        allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
+        assert!(
+            judge_before_pushing_a_floor_raise_for_a_refusal(
+                &allocator,
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges,
+                nothing_published_in_this_change(),
+                refusal,
+            )
+            .is_ok(),
+            "准入关着：合取 7 不判，推串之前也不判"
+        );
+    }
+
+    /// 被拒的那一次归不归合取 7 管，按会话发的改动分：写文件内容、建 inode 是前台写入；unlink、删快照、改 size、建快照不先判。
+    #[test]
+    fn data_writes_and_new_inodes_are_foreground_writes_and_the_other_changes_are_not_judged_before_pushing(
+    ) {
+        let content = [7_u8; 31];
+        let foreground_writes = [
+            UserChange::Overwrite(FirstFile {
+                content: &content,
+                write_time_seconds: 1_000_003,
+            }),
+            UserChange::SequentialWrite(FirstFile {
+                content: &content,
+                write_time_seconds: 1_000_007,
+            }),
+            UserChange::NewInodes {
+                count: 2,
+                write_time_seconds: 1_000_011,
+            },
+            UserChange::OverwriteOfAnInode {
+                inode: InodeNumber(4_099),
+                content: &content,
+                write_time_seconds: 1_000_013,
+            },
+            UserChange::SequentialWriteOfAnInode {
+                inode: InodeNumber(4_099),
+                content: &content,
+                write_time_seconds: 1_000_017,
+            },
+        ];
+        let not_judged_before_pushing = [
+            UserChange::Unlink {
+                inode: InodeNumber(4_099),
+                write_time_seconds: 1_000_019,
+            },
+            UserChange::CreateSnapshot,
+            UserChange::DestroySnapshot {
+                snapshot: TreeIdentifier(65_551),
+            },
+            UserChange::Truncate {
+                inode: InodeNumber(4_099),
+                new_size: 0,
+                write_time_seconds: 1_000_023,
+            },
+        ];
+        for change in foreground_writes {
+            assert_eq!(
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::of_the_user_change(change),
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges,
+                "{change:?}"
+            );
+        }
+        for change in not_judged_before_pushing {
+            assert_eq!(
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::of_the_user_change(change),
+                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ExemptOrNotKnownBeforeThePublish,
+                "{change:?}"
+            );
+        }
     }
 
     #[test]
@@ -8490,6 +8937,7 @@ mod everything_a_floor_raise_could_release_tests {
     fn refusal_after_one_series_was_pushed() -> UserChangeRefused {
         no_space_even_with_everything_a_floor_raise_could_release(
             Vec::new(),
+            None,
             refusal_of_the_test(),
             PublishedInThisChange {
                 floor_raises: vec![raised_floor_without_publishes()],
```

真池用例（测试代码，不被判，给腿读）：补丁 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-pool-tests-writer/patch/crates.patch`。

```diff
diff --git a/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs b/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
index 3b43c142e..a874f75f0 100644
--- a/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
+++ b/crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs
@@ -1,8 +1,9 @@
 //! `accept` 被空间拒时当场推抬 F 再判（D16（发布语义） 已定项 1「准入」那一行）的几格，核心层的攒批会话上判：
-//! 待发里已收下的改动不丢、不重复发（推成、抬 F 写盘报错冻结、排空时抬 F 写盘报错冻结）；推到不再推了还放不下、真满没有可放的报拒不崩，拒绝里带推了几串，盘上状态挂得上、读得出；推抬 F 全放出来也放不下的不推，拒绝里带这次调用已落盘的。
+//! 待发里已收下的改动不丢、不重复发（推成、抬 F 写盘报错冻结、排空时抬 F 写盘报错冻结）；推到不再推了还放不下、真满没有可放的报拒不崩，拒绝里带推了几串，盘上状态挂得上、读得出；推抬 F 全放出来也放不下的不推（短在式子那一判、短在段外槽对那一判而三样全放开之后真成的对数也不够、推串之前全空聚簇段已少于 `N₀` 三形），拒绝里带这次调用已落盘的。
 //! 真写满的池上的那一族与运行库一层在 `crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs`。
 mod common_admission_pool;
 
+use std::collections::BTreeSet;
 use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
 use std::sync::Arc;
 use std::time::Duration;
@@ -11,28 +12,38 @@ use common_admission_pool::{
     content_of, parameters_of_the_384_slot_small_devices, DeviceOverASparseImage, PoolUnderTest,
     FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
 };
-use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InodeNumber, SlotNumber};
-use singlefs_core::admission::JudgementThatFoundTheDeviceShort;
-use singlefs_core::allocator::{PoolAllocator, UnitAreaStart};
+use singlefs_core::address::{
+    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InodeNumber,
+    SlotNumber,
+};
+use singlefs_core::admission::{
+    AvailableBytesOnOneDevice, BytesOnOneDevice, DeviceWithEmptyClusterSegmentsBelowTheFloor,
+    EmptyClusterSegmentsBelowTheFloorOnSomeDevices, JudgementThatFoundTheDeviceShort,
+};
+use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, UnitAreaStart};
 use singlefs_core::block_device::{
     BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
 };
-use singlefs_core::compaction_plan::CallerMonotonicClockReading;
+use singlefs_core::compaction_plan::{
+    CallerMonotonicClockReading, CompactionSwitches, EmptyClusterSegmentsFloor,
+    EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION,
+};
 use singlefs_core::mount::{FloorRaiseStop, PUBLISHES_PER_ADMISSION_AT_MOST};
 use singlefs_core::mounted_session::{
-    refusal_is_short_of_space, AcceptRefused, BatchJudgedAtTheAccept, HowTheChangeWasTaken,
-    MountedSession, MountedSessionWithPendingChanges, ReasonOfThePublish, UserChange,
-    UserChangeRefused,
+    refusal_is_short_of_space, AcceptRefused, BatchJudgedAtTheAccept,
+    DeviceShortEvenWithEverythingAFloorRaiseCouldRelease, HowTheChangeWasTaken, MountedSession,
+    MountedSessionWithPendingChanges, NoSpaceEvenWithEverythingAFloorRaiseCouldRelease,
+    ReasonOfThePublish, UserChange, UserChangeRefused,
 };
 use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
 use singlefs_core::system_configuration::PublishTriggerThresholds;
 use singlefs_core::transaction::{
     journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write,
     DataChangesPublishedTogether, FirstFile, KindOfTheFrozenPublish, NewInodesOfOneChange,
-    PoolWriter, PublishError, WholeWriteOfOneInode, FIRST_INODE_NUMBER,
+    PoolWriter, PublishError, TransactionUnit, WholeWriteOfOneInode, FIRST_INODE_NUMBER,
 };
 use singlefs_core::unit::data_unit_payload_capacity;
-use singlefs_format::{INODE_LEAF_RECORDS, SLOT_BYTES};
+use singlefs_format::{CLUSTER_SEGMENT_SLOTS, INODE_LEAF_RECORDS, SLOT_BYTES};
 use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
 
 /// 时间那一支不该先到：一小时。
@@ -1045,3 +1056,375 @@ fn a_write_short_on_slot_pairs_only_behind_slots_held_for_the_floor_is_not_refus
         );
     }
 }
+
+/// 一个数据单元的槽数换成 `pairs` 个槽对的字节（落得下那一判与抬 F 之前先算的槽对那一判都按它换）。
+fn bytes_of_slot_pairs(pairs: u64) -> u64 {
+    BytesOnOneDevice::of_slots(
+        pairs * TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots(),
+    )
+    .0
+}
+
+/// 槽对那一判的上界照「每个放得开的槽各补一对」数：段外没挡的槽对，加 defer、隔离与空闲而挡着（隔离或抬 F 扣住）的槽各算一对。
+/// 一个数据单元两槽进 defer 算两对、聚簇段里的 defer 槽也算，比三样全放开之后段外真成的对数多；用例拿它核需求落在两者之间（阳性对照）。
+fn slot_pairs_counting_one_pair_for_every_slot_a_floor_raise_could_release(
+    device_map: &DeviceFreeMap,
+    cluster_segments: &BTreeSet<SlotNumber>,
+) -> u64 {
+    device_map.unblocked_slot_pairs_outside_the_cluster_segments(cluster_segments)
+        + device_map.deferred_slots()
+        + device_map.isolated_slots()
+        + (device_map.free_slots() - device_map.unblocked_slots())
+}
+
+/// 这块盘单元区里最高的、整段 64 槽都没分配过（`segment_is_empty`）、又不是这次挂载开过的聚簇段的那一段的起点槽号。
+/// 第一个文件与覆盖写都落在低处，单元区大半空着。
+fn highest_unused_segment_outside_the_cluster_segments(
+    device_map: &DeviceFreeMap,
+    cluster_segments: &BTreeSet<SlotNumber>,
+) -> u64 {
+    let start = device_map.unit_area_start().slot().0;
+    let full_segments = device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS;
+    // 迭代上界是单元区的整段数；提前出口：找到最高那一段。
+    (0..full_segments)
+        .rev()
+        .find(|segment| {
+            let segment_start = start + segment * CLUSTER_SEGMENT_SLOTS;
+            device_map.segment_is_empty(usize::try_from(*segment).expect("段号装得进 usize"))
+                && !cluster_segments.contains(&SlotNumber(segment_start))
+        })
+        .map(|segment| start + segment * CLUSTER_SEGMENT_SLOTS)
+        .expect("单元区大半空着：有没分配过的整段")
+}
+
+/// 每块盘上段外两槽都空的槽对，除了最高那一整段没分配过的（[`highest_unused_segment_outside_the_cluster_segments`]），都把上面那一槽记成已分配
+/// （盘上不动）：那几对罩进未释放的记录、放开什么都不再成对；下面那一槽照旧没挡，留给抬 F 那一串的提交内生块。最高那一段原样留着：
+/// 它的 32 对是段外仅剩没挡的槽对，它全空让合取 7（`N₀` 取第一版的值）照旧放行，推不推只由槽对那一判的上界定。
+fn take_the_upper_slot_of_every_free_slot_pair_outside_the_cluster_segments_but_the_highest_unused_segment(
+    session: &mut MountedSession,
+) {
+    let cluster_segments = session.allocator.cluster_segments().clone();
+    for device_map in &mut session.allocator.devices {
+        let start = device_map.unit_area_start().slot().0;
+        let end = start + device_map.unit_area_slots();
+        let kept_segment_start =
+            highest_unused_segment_outside_the_cluster_segments(device_map, &cluster_segments);
+        // 迭代上界是单元区槽数的一半：每一轮往后两槽，跨轮只改这一对上面那一槽的分配位。
+        for first_slot_of_the_pair in (start..end - 1).step_by(2) {
+            let segment_start = start
+                + (first_slot_of_the_pair - start) / CLUSTER_SEGMENT_SLOTS * CLUSTER_SEGMENT_SLOTS;
+            let is_kept_or_inside_a_cluster_segment = segment_start == kept_segment_start
+                || cluster_segments.contains(&SlotNumber(segment_start));
+            if !is_kept_or_inside_a_cluster_segment
+                && device_map.is_free(SlotNumber(first_slot_of_the_pair))
+                && device_map.is_free(SlotNumber(first_slot_of_the_pair + 1))
+            {
+                device_map.mark_allocated(SlotNumber(first_slot_of_the_pair + 1), 1);
+            }
+        }
+    }
+}
+
+/// 这一次单独那一批被判全放出来也放不下：交回那个成员。
+fn refused_outright_for_this_change_alone(
+    refused: &AcceptRefused,
+) -> &NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {
+    let AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
+        judgement: BatchJudgedAtTheAccept::ThisChangeAlone,
+        refusal,
+    } = refused
+    else {
+        panic!("判的是这一次单独那一批：{refused:?}");
+    };
+    let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) = &**refusal
+    else {
+        panic!("全放出来也放不下：不推，实际 {refusal:?}");
+    };
+    no_space
+}
+
+/// 被拒的这一次一串都没推、一版都没发：F 没动、盘上逐字节不变、待发空；进程退出再可写挂载读得出旧内容。
+fn nothing_was_pushed_and_the_old_content_reads_back_after_a_crash(
+    pool: &mut PoolUnderTest<DeviceFailingAWriteOnRequest>,
+    session: MountedSessionWithPendingChanges,
+    refused: &AcceptRefused,
+    image_before: &MemoryPool,
+    floor_before: CheckpointTxg,
+) {
+    let no_space = refused_outright_for_this_change_alone(refused);
+    assert!(
+        refused.floor_raises_pushed_by_this_accept().is_empty()
+            && no_space.versions_published.is_empty()
+            && no_space.refusals_that_pushed_the_floor_raises.is_empty(),
+        "头一次被拒就判到这里：一串都没推、一版都没发"
+    );
+    assert_eq!(
+        session.session().current.root().rollback_floor,
+        floor_before,
+        "F 没动"
+    );
+    assert!(pool.image() == *image_before, "盘上逐字节不变");
+    assert_eq!(session.pending_changes_accepted(), 0, "待发空");
+    drop(session);
+    let content_before = pool.content_of_the_current_version.clone();
+    pool.crash_and_mount_writable()
+        .expect("崩了再可写挂载做得成：这一次没动盘");
+    assert_eq!(
+        first_file_read_back(&pool.image()),
+        content_before,
+        "再挂载读得出旧内容"
+    );
+}
+
+/// 短在段外槽对那一判，需求落在「三样全放开之后段外真成的槽对」与「每个放得开的槽各补一对」之间：段外没挡的槽对只剩最高那一段的 32 对，
+/// defer 里是覆盖写换下的数据单元（两槽放开只成一对）与聚簇段里的提交内生块（放开不在段外成对）。真成的对数不够，一串都不推就拒，
+/// 点名两块盘、上界是真成的对数；盘上逐字节不变、待发空，崩了再挂载读得出旧内容。
+/// 判别力：上界照每个放得开的槽各补一对算时够得上需求，推一串抬 F，推完照旧放不下。
+#[test]
+fn an_accept_short_on_slot_pairs_that_releasing_everything_would_not_complete_is_refused_without_a_series(
+) {
+    let PoolWithTheFloorBelowItsCeiling {
+        mut pool,
+        mut session,
+        fault: _,
+    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
+    take_the_upper_slot_of_every_free_slot_pair_outside_the_cluster_segments_but_the_highest_unused_segment(
+        &mut session,
+    );
+    let cluster_segments = session.allocator.cluster_segments().clone();
+    let units_of_the_write = session
+        .allocator
+        .devices
+        .iter()
+        .map(|device_map| {
+            device_map.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
+                &cluster_segments,
+            )
+        })
+        .max()
+        .expect("池里有盘")
+        + 1;
+    let mut devices_short_expected = Vec::new();
+    for device_map in &session.allocator.devices {
+        assert_eq!(
+            device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments),
+            CLUSTER_SEGMENT_SLOTS / 2,
+            "阳性对照：段外没挡的槽对只剩最高那一段的 32 对"
+        );
+        assert!(
+            device_map.empty_segments() >= EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION,
+            "阳性对照：最高那一段全空，合取 7 那一判放行"
+        );
+        assert!(
+            device_map.deferred_slots() > 0,
+            "阳性对照：覆盖写过，defer 里有槽"
+        );
+        let slot_pairs_once_everything_is_released = device_map
+            .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
+                &cluster_segments,
+            );
+        let slot_pairs_counting_one_pair_for_every_slot =
+            slot_pairs_counting_one_pair_for_every_slot_a_floor_raise_could_release(
+                device_map,
+                &cluster_segments,
+            );
+        assert!(
+            slot_pairs_once_everything_is_released < units_of_the_write
+                && units_of_the_write <= slot_pairs_counting_one_pair_for_every_slot,
+            "阳性对照：需求 {units_of_the_write} 对落在全放开之后真成的 {slot_pairs_once_everything_is_released} 对\
+             与每个放得开的槽各补一对的 {slot_pairs_counting_one_pair_for_every_slot} 对之间"
+        );
+        devices_short_expected.push(DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
+            device: device_map.device,
+            judged_short_by:
+                JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
+            most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(i128::from(
+                bytes_of_slot_pairs(slot_pairs_once_everything_is_released),
+            )),
+            demand: BytesOnOneDevice(bytes_of_slot_pairs(units_of_the_write)),
+        });
+    }
+    let floor_before = session.current.root().rollback_floor;
+    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
+    let mut session = batching(session, journal_ring_bytes);
+    let content = vec![
+        0x5A_u8;
+        usize::try_from(units_of_the_write).expect("单元数装得进 usize")
+            * data_unit_payload_capacity()
+    ];
+    let image_before = pool.image();
+    let refused = session
+        .accept(
+            &mut pool.devices,
+            UserChange::SequentialWrite(FirstFile {
+                content: &content,
+                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1709,
+            }),
+            reading_zero(),
+        )
+        .expect_err("全放开之后段外真成的槽对也比需求少：收不下");
+    let no_space = refused_outright_for_this_change_alone(&refused);
+    assert!(
+        no_space.empty_cluster_segments_below_the_floor.is_none(),
+        "判出它的是槽对那一判，不是合取 7：{:?}",
+        no_space.empty_cluster_segments_below_the_floor
+    );
+    assert_eq!(
+        no_space.devices_short, devices_short_expected,
+        "两块盘都点名：判它短的是槽对那一判，上界是三样全放开之后段外真成的槽对，需求是这次的数据单元"
+    );
+    nothing_was_pushed_and_the_old_content_reads_back_after_a_crash(
+        &mut pool,
+        session,
+        &refused,
+        &image_before,
+        floor_before,
+    );
+}
+
+/// 每块盘从单元区最高处往下把空闲的槽记成抬 F 扣住（只动扣住位，不写盘），到段外没挡的槽对只剩 `slot_pairs_left` 对为止：
+/// 扣住的槽不罩在未释放的记录下，三样全放开之后照旧成对。
+fn hold_the_highest_free_slots_leaving_slot_pairs(
+    session: &mut MountedSession,
+    slot_pairs_left: u64,
+) {
+    let cluster_segments = session.allocator.cluster_segments().clone();
+    for device_map in &mut session.allocator.devices {
+        let start = device_map.unit_area_start().slot().0;
+        let slots = device_map.unit_area_slots();
+        // 迭代上界是单元区槽数；提前出口：段外没挡的槽对只剩这么多。
+        for slot in (start..start + slots).rev() {
+            if device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments)
+                <= slot_pairs_left
+            {
+                break;
+            }
+            if device_map.is_free(SlotNumber(slot)) {
+                device_map.hold_until_floor_takes_effect(SlotNumber(slot), 1);
+            }
+        }
+    }
+}
+
+/// 会话装上只供测试的 `N₀`（`EmptyClusterSegmentsFloor::ForcedForTests`），整理的别的开关照产品值。
+fn force_the_empty_cluster_segments_floor(session: &mut MountedSession, floor_in_segments: u64) {
+    let CompactionSwitches {
+        batch_size,
+        forced_region,
+        run_forcing,
+        region_exclusion,
+        destination_policy,
+        release_checksum_mismatch_after_issue,
+        empty_cluster_segments_floor: _,
+        relocation_publish_order,
+    } = CompactionSwitches::PRODUCT;
+    session
+        .allocator
+        .set_compaction_switches(CompactionSwitches {
+            batch_size,
+            forced_region,
+            run_forcing,
+            region_exclusion,
+            destination_policy,
+            release_checksum_mismatch_after_issue,
+            empty_cluster_segments_floor: EmptyClusterSegmentsFloor::ForcedForTests(
+                floor_in_segments,
+            ),
+            relocation_publish_order,
+        });
+}
+
+/// 落得下那一判先拒（段外没挡的槽对比需求少一对，差额压在抬 F 扣住的槽上），槽对那一判的上界够得上需求，而每块盘的全空聚簇段都少于 `N₀`：
+/// 推了也是被合取 7 拒，一串都不推就拒，带上合取 7 那一份（`N₀` 与每块盘的全空段数）、不点名槽对短的盘；盘上逐字节不变、待发空，
+/// 崩了再挂载读得出旧内容。这个池单元区大半空着，全空段多，`N₀` 经只供测试的开关抬到比每块盘的全空段数都多一段。
+/// 判别力：推串之前不判合取 7 时推一串抬 F，推完照旧被拒。
+#[test]
+fn an_accept_while_empty_cluster_segments_are_below_the_floor_is_refused_without_a_series() {
+    let PoolWithTheFloorBelowItsCeiling {
+        mut pool,
+        mut session,
+        fault: _,
+    } = pool_with_the_floor_below_its_ceiling(UNIT_AREA_SLOTS_OF_THE_SMALLER_DEVICES);
+    let units_of_the_write =
+        u64::try_from(UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT).expect("单元数装得进 u64");
+    hold_the_highest_free_slots_leaving_slot_pairs(&mut session, units_of_the_write - 1);
+    let cluster_segments = session.allocator.cluster_segments().clone();
+    for device_map in &session.allocator.devices {
+        assert_eq!(
+            device_map.unblocked_slot_pairs_outside_the_cluster_segments(&cluster_segments),
+            units_of_the_write - 1,
+            "阳性对照：段外没挡的槽对比需求少一对，落得下那一判拒"
+        );
+        assert!(
+            device_map.slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
+                &cluster_segments
+            ) >= units_of_the_write,
+            "阳性对照：扣住的槽放得开，槽对那一判的上界够得上需求"
+        );
+    }
+    let empty_cluster_segments_on_each_device: Vec<DeviceWithEmptyClusterSegmentsBelowTheFloor> =
+        session
+            .allocator
+            .devices
+            .iter()
+            .map(|device_map| DeviceWithEmptyClusterSegmentsBelowTheFloor {
+                device: device_map.device,
+                empty_cluster_segments: device_map.empty_segments(),
+            })
+            .collect();
+    let floor_in_segments = empty_cluster_segments_on_each_device
+        .iter()
+        .map(|device| device.empty_cluster_segments)
+        .max()
+        .expect("池里有盘")
+        + 1;
+    force_the_empty_cluster_segments_floor(&mut session, floor_in_segments);
+    let floor_before = session.current.root().rollback_floor;
+    let journal_ring_bytes = pool.parameters.geometry.journal_ring_bytes;
+    let mut session = batching(session, journal_ring_bytes);
+    let content =
+        vec![0x5A_u8; UNITS_OF_A_BATCH_WITHIN_THE_RECORD_LIMIT * data_unit_payload_capacity()];
+    let image_before = pool.image();
+    let refused = session
+        .accept(
+            &mut pool.devices,
+            UserChange::SequentialWrite(FirstFile {
+                content: &content,
+                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1710,
+            }),
+            reading_zero(),
+        )
+        .expect_err("全空聚簇段少于 N₀：收不下");
+    let no_space = refused_outright_for_this_change_alone(&refused);
+    assert!(
+        no_space.devices_short.is_empty(),
+        "没有上界小于需求的盘：{:?}",
+        no_space.devices_short
+    );
+    assert_eq!(
+        no_space.empty_cluster_segments_below_the_floor,
+        Some(EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
+            floor_in_segments,
+            short_devices: empty_cluster_segments_on_each_device,
+        }),
+        "合取 7 那一份：N₀ 与每块盘的全空段数"
+    );
+    let PublishError::SpaceAdmissionRefused(admission_refusal) = &no_space.last_refusal else {
+        panic!("被拒的是落得下那一判：{:?}", no_space.last_refusal);
+    };
+    assert!(
+        !admission_refusal.short_devices.is_empty()
+            && admission_refusal.short_devices.iter().all(|short| {
+                short.judged_short_by
+                    == JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits
+            }),
+        "推串之前被拒的是段外槽对那一判：{admission_refusal:?}"
+    );
+    nothing_was_pushed_and_the_old_content_reads_back_after_a_crash(
+        &mut pool,
+        session,
+        &refused,
+        &image_before,
+        floor_before,
+    );
+}
```

变异行：`crates/mutations.tsv` 追加 15 行（第一份补丁 13 行、第二份 2 行），删 8 行（`~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-writer/patch/mutations-delete.txt`）。
