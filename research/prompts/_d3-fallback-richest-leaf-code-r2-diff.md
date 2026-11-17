# d3-fallback-richest-leaf-code-r2 被判的改动（2026-11-12）

用户数据落点扫描换算法（用户 2026-11-12「改实现 优化算法」），连同第一轮判决 `research/prompts/d3-fallback-richest-leaf-code-r1-main-verification.md` 打中的名字、裸整数、错误成员与注释的改法。只列产品代码（`crates/singlefs-core/src/` 下三份），照三份补丁打进主工作区的次序排；测试与验证包的改动在各补丁目录的 `crates.patch` 里。

## 补丁 `~/.local/share/singlefs/work/user-data-placement-scan-writer/patch/crates.patch`（产品代码部分 3 份文件）

```diff
diff --git a/crates/singlefs-core/src/admission.rs b/crates/singlefs-core/src/admission.rs
index 725ab0c..2dd4b2f 100644
--- a/crates/singlefs-core/src/admission.rs
+++ b/crates/singlefs-core/src/admission.rs
@@ -978,7 +978,7 @@ pub struct PlacementRoomOnOneDevice {
     /// 数据单元落得下的槽对：两槽都没挡（已分配、影子账隔离、抬 F 扣住三种位都没置）、起点偶数、不在这次挂载开过的任何一个聚簇段里
     /// （已定项 8 第 1、2 条，已定项 10 ②③；`DeviceFreeMap::unblocked_slot_pairs_outside_the_cluster_segments`）。
     pub slot_pairs_for_data_units: u64,
-    /// 提交内生块落得下的槽：三种位都没置，聚簇段内外都算——开放段 bump、开全空段、回落到最低空槽挡的都只是这三种位
+    /// 提交内生块落得下的槽：三种位都没置，聚簇段内外都算——开放段 bump、开全空段、回落（先取这次发布重写集的叶、再取每块盘都空的槽最多的叶）挡的都只是这三种位
     /// （已定项 8 第 2 条、已定项 10 ①⑤；`DeviceFreeMap::unblocked_slots`）。
     pub slots_for_commit_generated_units: u64,
 }
@@ -1185,8 +1185,8 @@ impl SpaceAdmission {
 /// 中央映射树的节点容量按节点格式算（产品路径）；只供测试的压小容量下量同一个数用
 /// [`checkpoint_cost_of_the_version_to_build_on_with_node_capacities`]。
 ///
-/// 分配记录树那一项不读这一版有哪些节点：按「每块盘两条叶路径」算，在开放段装不下开新段、没有全空段回落到最低空槽、
-/// 换下很久之前落在别处的节点这三种情形下少扣（量到一块盘改 3–4 片叶），
+/// 分配记录树那一项不读这一版有哪些节点：按「每块盘两条叶路径」算，在开放段装不下开新段、没有全空段回落（先取这次发布重写集的叶，
+/// 那些叶里没有每块盘都空的落点再取每块盘都空的槽最多的叶，落点可以在单元区的任何一片叶里）、换下很久之前落在别处的节点这三种情形下少扣，
 /// 一次发布改几片叶的不动点收不了口，拿得准的上界只有几何上整棵树。代价随单元区线性涨（两块 4 GiB 盘 529 个节点）。
 ///
 /// 同一个数也是挂载期承诺量里暖机那一半的 c_max（已定项 4 末条「按同一个现算的 c_max 取」）。
```

```diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index c9ad4f5..ab4be3b 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -1,7 +1,8 @@
 //! 分配器（里程碑步 2 / 步 3）：内存里一张单元区的空闲图，两条落点政策，分配记录与记账增量在提交时增量维护。
 //!
 //! 用户数据落点（D3（空间分配） 已定项 8 / 已定项 10）：已选设备内起点槽号最小、起点 32768 对齐（偶数槽）、跨度内两槽都空、
-//! 不在任何开放的聚簇段里、不在 `R` 保护期内（第一版 R 为空）。
+//! 不在任何开放的聚簇段里、不在 `R` 保护期内（第一版 R 为空）。取法不从单元区起点逐个候选看：每块盘增量维护「有两槽都没挡的偶数起点槽对」的段号集合，
+//! 按段号升序走这个集合、跳过这次挂载开过的聚簇段与 `R` 那一段（按段起点查），落进第一段里最低的那一对（[`DeviceFreeMap::lowest_user_data_slot_outside_the_region`]）。
 //! 提交内生块（D3（空间分配） 已定项 5 / 已定项 10）：从开放聚簇段 bump，开放段 = 单元区内最低的 64 槽对齐全空段，bump 只在内存；
 //! 码 3 容器按数据单元那一档取落点（起点 32768 对齐），码 2 节点取最低空槽。
 //! 开放段装不下、又没有全空段时回落（D3（空间分配） 已定项 8 ②）：先取这次发布分配记录树重写集里的叶（[`RewriteSetLeaves`]，每块盘都重写的那几片）里
@@ -189,18 +190,48 @@ impl UnitFootprint {
 pub enum CommitGeneratedDeviceAnswer {
     /// 这块盘上最低的全空段的起点：在这里开新段（D3（空间分配） 已定项 10 ①）。
     OpenEmptySegment(SlotNumber),
-    /// 这块盘上没有全空段、要回落（D3（空间分配） 已定项 8 ②），带的是这块盘上槽号最小的空槽：各盘只按自己的空闲图答到这一步。
-    /// 每块盘都答回落时，池里按回落政策取一个每块盘都空的落点（[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]），
-    /// 把每块盘的答案都换成它；取不到时各盘这一步的答案原样去判一致。
-    FallBackToLowestFreeSlot(SlotNumber),
+    /// 这块盘上没有全空段、要回落（D3（空间分配） 已定项 8 第 2 条），带的是回落的落点。各盘只按自己的空闲图答时，它是这块盘单元区里
+    /// 起点最低、整跨度没挡、不在活 `R` 里的落点（[`DeviceFreeMap::lowest_commit_generated_fallback_slot_outside_the_region`]）；
+    /// 每块盘都答回落、池里按回落政策取到一个每块盘都空的落点时（[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]），
+    /// 每块盘的答案都换成池里取的那一个。
+    FallBack(SlotNumber),
 }
 
+/// 分配记录树一片叶的序号：盘上第 k 片叶罩槽 `[k × W, (k + 1) × W)`（W = [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]），即槽号 ÷ W。
+/// 与槽号、槽数同为 `u64` 而含义不同，各用各的类型。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct AllocationRecordTreeLeafIndex(pub u64);
+
+impl AllocationRecordTreeLeafIndex {
+    /// 罩着 `slot` 的那片叶。
+    #[must_use]
+    pub const fn of_slot(slot: SlotNumber) -> Self {
+        Self(slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS)
+    }
+
+    /// 这片叶罩的第一个槽。
+    #[must_use]
+    pub const fn first_slot(self) -> SlotNumber {
+        SlotNumber(self.0 * ALLOCATION_RECORD_TREE_LEAF_SLOTS)
+    }
+
+    /// 这片叶罩的最后一个槽的下一个槽（下一片叶的第一个槽）。
+    #[must_use]
+    pub const fn past_the_last_slot(self) -> SlotNumber {
+        SlotNumber((self.0 + 1) * ALLOCATION_RECORD_TREE_LEAF_SLOTS)
+    }
+}
+
+/// 一片分配记录树叶里的槽数（没挡的槽数、每块盘都空的槽数），至多 [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]；是槽数，不是槽号，也不是槽对数。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct SlotCountInALeaf(pub u64);
+
 /// 这次发布分配记录树重写集里的叶（回落先取的去处，D3（空间分配） 已定项 8 第 2 条）：叶序号 = 槽号 ÷ [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]，
 /// 只含池里每块盘上都在重写集里的叶——提交内生块的落点各盘同槽，落在这样的叶里，每块盘上都不多弄脏一片叶。
 /// 空集 = 没有重写集可先取，回落直接取空槽最多的叶。发布路径按这次发布的重写角色算（演练与真发读同一串角色，得同一个集合）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct RewriteSetLeaves {
-    leaf_indices: BTreeSet<u64>,
+    leaf_indices: BTreeSet<AllocationRecordTreeLeafIndex>,
 }
 
 impl RewriteSetLeaves {
@@ -214,7 +245,9 @@ impl RewriteSetLeaves {
 
     /// 每块盘各自重写集里的叶序号（`leaf_indices_per_device` 一块盘一项）取交集：一片叶只有在池里每块盘上都重写，才算数。
     #[must_use]
-    pub fn present_on_every_device(leaf_indices_per_device: &[BTreeSet<u64>]) -> Self {
+    pub fn present_on_every_device(
+        leaf_indices_per_device: &[BTreeSet<AllocationRecordTreeLeafIndex>],
+    ) -> Self {
         let Some((leaf_indices_of_one_device, leaf_indices_of_the_other_devices)) =
             leaf_indices_per_device.split_first()
         else {
@@ -235,7 +268,7 @@ impl RewriteSetLeaves {
 
     /// 集合里的叶序号，升序。
     #[must_use]
-    pub fn leaf_indices(&self) -> &BTreeSet<u64> {
+    pub fn leaf_indices(&self) -> &BTreeSet<AllocationRecordTreeLeafIndex> {
         &self.leaf_indices
     }
 }
@@ -253,11 +286,42 @@ pub enum PlacementRefusal {
     UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
         slot_per_device: Vec<(DeviceIdentity, SlotNumber)>,
     },
-    /// 开放段装不下之后各盘给的去处不同（一块开段一块回落，或开的段不同）：各盘上的聚簇段要不要对齐是 D3（空间分配） 已定项 8 待办 ①，
-    /// 没有条款；`Placement` 两盘同槽；第一版不支持。
+    /// 开放段装不下之后各盘给的去处不同，而不是每块盘都答回落（一块开段一块回落，或开的段不同）：各盘上的聚簇段要不要对齐是
+    /// D3（空间分配） 已定项 8 待办 ①，没有条款；`Placement` 两盘同槽；第一版不支持。
     CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
         answer_per_device: Vec<(DeviceIdentity, CommitGeneratedDeviceAnswer)>,
     },
+    /// 每块盘都答回落，池里却取不到一个每块盘都空的落点：每块盘自己的空闲图上还有回落的落点（`fallback_slot_per_device`，各盘自己答的那一个），
+    /// 可没有一个槽号在每块盘上都空（D3（空间分配） 已定项 8 末段「池里取不到每块盘都空的落点时，照各盘自己的答案判一致，不一致拒绝」）。
+    /// 与段对齐那一条分开报：这一条不是没有条款的池形状，是各盘的空槽错开了——腾出空间（推抬 F、整理、删）之后同一次写可能落得下。
+    CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice {
+        fallback_slot_per_device: Vec<(DeviceIdentity, SlotNumber)>,
+    },
+}
+
+/// 开放段装不下之后各盘答得不一致（[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]）时报哪一种拒绝：
+/// 每块盘都答回落（池里取不到每块盘都空的落点，各盘自己的回落落点又不同）报
+/// [`PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice`]，别的（有的盘开段）报段对齐那一条。
+fn refusal_of_commit_generated_answers_that_differ(
+    answer_per_device: Vec<(DeviceIdentity, CommitGeneratedDeviceAnswer)>,
+) -> PlacementRefusal {
+    let fallback_slot_per_device: Option<Vec<(DeviceIdentity, SlotNumber)>> = answer_per_device
+        .iter()
+        .map(|(device, answer)| match answer {
+            CommitGeneratedDeviceAnswer::FallBack(slot) => Some((*device, *slot)),
+            CommitGeneratedDeviceAnswer::OpenEmptySegment(_) => None,
+        })
+        .collect();
+    match fallback_slot_per_device {
+        Some(fallback_slot_per_device) => {
+            PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice {
+                fallback_slot_per_device,
+            }
+        }
+        None => PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
+            answer_per_device,
+        },
+    }
 }
 
 /// 各盘各答一个之后合成的结论。
@@ -415,6 +479,10 @@ pub struct DeviceFreeMap {
     unblocked_slot_pairs_per_segment: Vec<SlotPairCount>,
     /// 上一项全部段之和。
     unblocked_slot_pairs: SlotPairCount,
+    /// `unblocked_slot_pairs_per_segment` 不是 0 的那几段的段号（从单元区起点起数）：用户数据落点按段号升序走它
+    /// （[`Self::lowest_user_data_slot_outside_the_region`]），不从单元区起点逐个候选看。那一段的槽对数在 0 与非 0 之间翻过去时
+    /// 跟着加进或摘掉（[`Self::set_blocking_bit`]），与那一项同一处维护。
+    segments_with_an_unblocked_slot_pair: BTreeSet<usize>,
     /// 每一槽是不是罩在未释放的分配记录下（`unreleased_per_segment` 数的那几槽，逐槽一位）：分配与复活时置、释放时清，回收不动。
     /// 不罩在未释放记录下的槽只有四种：空闲的、defer 里的、影子账隔离的、抬 F 扣住的——抬 F 能放开的就是后三种。
     is_under_an_unreleased_record: Vec<bool>,
@@ -427,7 +495,7 @@ pub struct DeviceFreeMap {
     /// 分配记录树每片叶里三种位都没置的单元区槽数（叶序号 = 槽号 ÷ [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]，下标 = 叶序号 − 单元区起点所在那片叶的序号；
     /// 单元区首尾那两片叶只数单元区里的那一截）。回落取空槽最多的叶时拿它当每片叶「每块盘都空的槽数」的上界
     /// （[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]），同上增量维护。
-    unblocked_slots_per_allocation_record_tree_leaf: Vec<u64>,
+    unblocked_slots_per_allocation_record_tree_leaf: Vec<SlotCountInALeaf>,
     /// 回收过槽、还没发过 discard 的段（段号）：回收那一刻记（[`Self::mark_reclaimed`]），发布返回之后整段可再分配时发 discard 并摘掉
     /// 只住内存。
     segments_reclaimed_not_yet_discarded: BTreeSet<usize>,
@@ -572,21 +640,30 @@ impl DeviceFreeMap {
             .collect();
         // 起步整个单元区都空：每片叶的数是这片叶与单元区相交那一截的槽数。
         let unit_area_end_slot = unit_area_start.slot().0 + unit_area_slots;
-        let unblocked_slots_per_allocation_record_tree_leaf: Vec<u64> = if unit_area_slots == 0 {
-            Vec::new()
-        } else {
-            (unit_area_start.slot().0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS
-                ..=(unit_area_end_slot - 1) / ALLOCATION_RECORD_TREE_LEAF_SLOTS)
-                .map(|leaf_index| {
-                    let past_the_leaf = ((leaf_index + 1) * ALLOCATION_RECORD_TREE_LEAF_SLOTS)
-                        .min(unit_area_end_slot);
-                    let first_slot_of_the_leaf_in_the_unit_area = (leaf_index
-                        * ALLOCATION_RECORD_TREE_LEAF_SLOTS)
-                        .max(unit_area_start.slot().0);
-                    past_the_leaf - first_slot_of_the_leaf_in_the_unit_area
-                })
-                .collect()
-        };
+        let unblocked_slots_per_allocation_record_tree_leaf: Vec<SlotCountInALeaf> =
+            if unit_area_slots == 0 {
+                Vec::new()
+            } else {
+                (AllocationRecordTreeLeafIndex::of_slot(unit_area_start.slot()).0
+                    ..=AllocationRecordTreeLeafIndex::of_slot(SlotNumber(unit_area_end_slot - 1)).0)
+                    .map(|leaf_number| {
+                        let leaf_index = AllocationRecordTreeLeafIndex(leaf_number);
+                        let past_the_leaf =
+                            leaf_index.past_the_last_slot().0.min(unit_area_end_slot);
+                        let first_slot_of_the_leaf_in_the_unit_area =
+                            leaf_index.first_slot().0.max(unit_area_start.slot().0);
+                        SlotCountInALeaf(past_the_leaf - first_slot_of_the_leaf_in_the_unit_area)
+                    })
+                    .collect()
+            };
+        // 起步整个单元区都空：有槽对的段就是槽对数不是 0 的段（末尾那一段只有一槽时没有）。
+        let segments_with_an_unblocked_slot_pair: BTreeSet<usize> =
+            unblocked_slot_pairs_per_segment
+                .iter()
+                .enumerate()
+                .filter(|(_, pairs)| pairs.0 > 0)
+                .map(|(segment_index, _)| segment_index)
+                .collect();
         Self {
             device,
             device_size_in_bytes: device_bytes,
@@ -624,6 +701,7 @@ impl DeviceFreeMap {
             slot_pairs_without_an_unreleased_record_per_segment: unblocked_slot_pairs_per_segment
                 .clone(),
             unblocked_slot_pairs_per_segment,
+            segments_with_an_unblocked_slot_pair,
             unblocked_slots_per_allocation_record_tree_leaf,
             segments_reclaimed_not_yet_discarded: BTreeSet::new(),
         }
@@ -666,7 +744,7 @@ impl DeviceFreeMap {
             && self.is_unblocked_at_index(first_index_of_the_pair + 1)
     }
 
-    /// 改一槽的一种挡位，同时增量维护没挡的槽数与没挡的槽对数（每段一份与总数）：三张位图只经这里改。
+    /// 改一槽的一种挡位，同时增量维护没挡的槽数（总数与每片叶一份）、没挡的槽对数（每段一份与总数）与有没挡槽对的段号集合：三张位图只经这里改。
     fn set_blocking_bit(&mut self, index: usize, bit: SlotBlockingBit, is_set: bool) {
         let slot_was_unblocked = self.is_unblocked_at_index(index);
         let pair_was_unblocked = self.is_unblocked_pair_at_index(index);
@@ -681,11 +759,11 @@ impl DeviceFreeMap {
         match (slot_was_unblocked, self.is_unblocked_at_index(index)) {
             (true, false) => {
                 self.unblocked_slots -= 1;
-                self.unblocked_slots_per_allocation_record_tree_leaf[leaf] -= 1;
+                self.unblocked_slots_per_allocation_record_tree_leaf[leaf].0 -= 1;
             }
             (false, true) => {
                 self.unblocked_slots += 1;
-                self.unblocked_slots_per_allocation_record_tree_leaf[leaf] += 1;
+                self.unblocked_slots_per_allocation_record_tree_leaf[leaf].0 += 1;
             }
             (true, true) | (false, false) => {}
         }
@@ -694,10 +772,14 @@ impl DeviceFreeMap {
             (true, false) => {
                 self.unblocked_slot_pairs_per_segment[segment].0 -= 1;
                 self.unblocked_slot_pairs.0 -= 1;
+                if self.unblocked_slot_pairs_per_segment[segment].0 == 0 {
+                    self.segments_with_an_unblocked_slot_pair.remove(&segment);
+                }
             }
             (false, true) => {
                 self.unblocked_slot_pairs_per_segment[segment].0 += 1;
                 self.unblocked_slot_pairs.0 += 1;
+                self.segments_with_an_unblocked_slot_pair.insert(segment);
             }
             (true, true) | (false, false) => {}
         }
@@ -705,40 +787,49 @@ impl DeviceFreeMap {
 
     /// 位图下标 → 它所在那片分配记录树叶在 `unblocked_slots_per_allocation_record_tree_leaf` 里的下标。
     fn allocation_record_tree_leaf_position_of_index(&self, index: usize) -> usize {
-        let slot =
-            self.unit_area_start.slot().0 + u64::try_from(index).expect("位图下标装得进 u64");
+        let slot = SlotNumber(
+            self.unit_area_start.slot().0
+                + u64::try_from(index)
+                    .expect("位图下标小于单元区槽数，单元区槽数本来就是 u64（with_unit_area_start 由它定位图长度）"),
+        );
         usize::try_from(
-            slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS
-                - self.unit_area_start.slot().0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS,
+            AllocationRecordTreeLeafIndex::of_slot(slot).0
+                - AllocationRecordTreeLeafIndex::of_slot(self.unit_area_start.slot()).0,
         )
-        .expect("叶下标装得进 usize")
+        .expect("叶在单元区里的下标不大于位图下标（一片叶至少罩一个槽），位图下标本来就是 usize")
     }
 
     /// 分配记录树第 `leaf_index` 片叶（罩槽 `[leaf_index × W, (leaf_index + 1) × W)`，W = [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]）里
     /// 三种位都没置的单元区槽数；叶整片在单元区外是 0。增量维护，读一次 O(1)。
     #[must_use]
-    pub fn unblocked_slots_in_allocation_record_tree_leaf(&self, leaf_index: u64) -> u64 {
-        let first_leaf_index = self.unit_area_start.slot().0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS;
+    pub fn unblocked_slots_in_allocation_record_tree_leaf(
+        &self,
+        leaf_index: AllocationRecordTreeLeafIndex,
+    ) -> SlotCountInALeaf {
+        let first_leaf_index = AllocationRecordTreeLeafIndex::of_slot(self.unit_area_start.slot());
         leaf_index
-            .checked_sub(first_leaf_index)
+            .0
+            .checked_sub(first_leaf_index.0)
             .and_then(|position| usize::try_from(position).ok())
             .and_then(|position| {
                 self.unblocked_slots_per_allocation_record_tree_leaf
                     .get(position)
                     .copied()
             })
-            .unwrap_or(0)
+            .unwrap_or(SlotCountInALeaf(0))
     }
 
     /// 单元区罩到的分配记录树叶序号（首尾含）；单元区是空的交回 `None`。
     #[must_use]
-    pub fn allocation_record_tree_leaves_of_the_unit_area(&self) -> Option<(u64, u64)> {
+    pub fn allocation_record_tree_leaves_of_the_unit_area(
+        &self,
+    ) -> Option<(AllocationRecordTreeLeafIndex, AllocationRecordTreeLeafIndex)> {
         if self.unit_area_slots == 0 {
             return None;
         }
         Some((
-            self.unit_area_start.slot().0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS,
-            (self.unit_area_end_slot() - 1) / ALLOCATION_RECORD_TREE_LEAF_SLOTS,
+            AllocationRecordTreeLeafIndex::of_slot(self.unit_area_start.slot()),
+            AllocationRecordTreeLeafIndex::of_slot(SlotNumber(self.unit_area_end_slot() - 1)),
         ))
     }
 
@@ -747,14 +838,14 @@ impl DeviceFreeMap {
     #[must_use]
     fn is_whole_span_open_for_commit_generated(
         &self,
-        candidate: u64,
+        candidate: SlotNumber,
         footprint: UnitFootprint,
         excluded_region: Option<CompactionRegion>,
     ) -> bool {
-        let is_inside_the_unit_area = candidate >= self.unit_area_start.slot().0
-            && candidate + footprint.slots() <= self.unit_area_end_slot();
+        let is_inside_the_unit_area = candidate.0 >= self.unit_area_start.slot().0
+            && candidate.0 + footprint.slots() <= self.unit_area_end_slot();
         is_inside_the_unit_area
-            && (candidate..candidate + footprint.slots()).all(|slot_of_the_span| {
+            && (candidate.0..candidate.0 + footprint.slots()).all(|slot_of_the_span| {
                 let is_outside_the_excluded_region = !excluded_region
                     .is_some_and(|region| region.contains_slot(SlotNumber(slot_of_the_span)));
                 is_outside_the_excluded_region
@@ -1216,11 +1307,64 @@ impl DeviceFreeMap {
 
     /// 同 [`Self::lowest_user_data_slot`]，另加一条合取「候选落点不在活整理意图的 `R` 里」（D3（空间分配） 已定项 7 登记表第 5 行、
     /// D26（后台整理与放置回收） 已定项 1 第 3 条）。`excluded_region` 是 `None` 时与它相同。
+    ///
+    /// 取法：按段号升序走有没挡槽对的段（`segments_with_an_unblocked_slot_pair`，随三种挡位增量维护），跳过起点在 `cluster_segments` 里的段
+    /// 与起点在 `R` 里的段，交回第一个没跳过的段里最低的那一对。与从单元区起点按 2 槽逐个看的取法答案相同，依据：一对不跨段
+    /// （单元区起点在段边界上、段长是偶数）；聚簇段的起点是开段时各盘一致答出来的全空段起点（[`Self::lowest_empty_segment`]），`R` 的起点按类型不变量是
+    /// 64 的倍数，单元区起点也是 ⇒ 三者都是同一套 64 槽格子上的整段，一个候选在不在聚簇段或 `R` 里等于它所在那一段的起点在不在。
+    /// 代价：集合里每走过一段，那一段要么在 `cluster_segments` 里、要么是 `R` 那一段、要么就是答案所在的段 ⇒ 至多走过
+    /// 「这次挂载开过的聚簇段数 + 2」段（每段一次集合点查），再在答案那一段里看至多 32 对；不随单元区的槽数、也不随已占的槽数长。
     #[must_use]
     pub fn lowest_user_data_slot_outside_the_region(
         &self,
         cluster_segments: &BTreeSet<SlotNumber>,
         excluded_region: Option<CompactionRegion>,
+    ) -> Option<SlotNumber> {
+        self.segments_with_an_unblocked_slot_pair
+            .iter()
+            .find_map(|segment_index| {
+                let segment_start = self.segment_start_slot(*segment_index);
+                let is_open_to_user_data = !cluster_segments.contains(&segment_start)
+                    && !excluded_region.is_some_and(|region| region.contains_slot(segment_start));
+                if is_open_to_user_data {
+                    self.lowest_unblocked_slot_pair_in_the_segment(*segment_index)
+                } else {
+                    None
+                }
+            })
+    }
+
+    /// 第 `segment_index` 段里起点最低、两槽都没挡的那一对（[`Self::is_unblocked_pair_at_index`]，从段首按 2 槽步进；单元区末尾不成对的那一槽不算）；
+    /// 没有就 `None`。至多看 32 对。
+    fn lowest_unblocked_slot_pair_in_the_segment(
+        &self,
+        segment_index: usize,
+    ) -> Option<SlotNumber> {
+        let slots_per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
+        let first_index_of_the_segment = segment_index * slots_per_segment;
+        let past_the_segment =
+            (first_index_of_the_segment + slots_per_segment).min(self.allocated.len());
+        (first_index_of_the_segment..past_the_segment)
+            .step_by(2)
+            .find(|first_index_of_the_pair| {
+                self.is_unblocked_pair_at_index(*first_index_of_the_pair)
+            })
+            .map(|first_index_of_the_pair| {
+                SlotNumber(
+                    self.unit_area_start.slot().0
+                        + u64::try_from(first_index_of_the_pair)
+                            .expect("位图下标小于单元区槽数，单元区槽数本来就是 u64"),
+                )
+            })
+    }
+
+    /// 用户数据落点的参照取法，只供测试与 [`Self::lowest_user_data_slot_outside_the_region`] 对拍：从单元区起点按 2 槽逐个候选看，
+    /// 每个候选逐段问在不在聚簇段里、问在不在 `R` 里、问两槽空不空，第一个都过的就是答案。
+    #[cfg(test)]
+    fn lowest_user_data_slot_outside_the_region_by_stepping_through_every_candidate(
+        &self,
+        cluster_segments: &BTreeSet<SlotNumber>,
+        excluded_region: Option<CompactionRegion>,
     ) -> Option<SlotNumber> {
         // 起点在聚簇段边界上（`UnitAreaStart` 的不变量）⇒ 是偶数槽，按 2 槽步进的候选都 32768 对齐。
         let mut candidate = self.unit_area_start.slot().0;
@@ -1274,7 +1418,7 @@ impl DeviceFreeMap {
             })
     }
 
-    /// 这块盘按自己的空闲图答的回落落点（[`CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot`] 带的那个槽；池里真取的回落落点见
+    /// 这块盘按自己的空闲图答的回落落点（各盘只按自己的空闲图答时 [`CommitGeneratedDeviceAnswer::FallBack`] 带的那个槽；池里真取的回落落点见
     /// [`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]）：这块盘单元区里起点槽号最小、整个跨度都不被挡的落点——
     /// 挡的是 bump 游标绕开的同一套位（已分配、影子账隔离、抬 F 扣住），回落不是绕开影子账或扣住的第二条路；两槽的码 3 容器照数据单元那一档
     /// 起点 32768 对齐（D3（空间分配） 已定项 10 ⑤）。第一版 `R` 为空。
@@ -1333,7 +1477,7 @@ impl DeviceFreeMap {
                     footprint,
                     excluded_region,
                 )
-                .map(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot),
+                .map(CommitGeneratedDeviceAnswer::FallBack),
         }
     }
 }
@@ -2160,12 +2304,12 @@ pub struct PoolAllocator {
     /// 复用窗口置 0 时在 `release` 里当场回收掉的落点数（两盘同槽的一个落点算一条，就是 `reclaim_released_up_to` 交回的条数）：
     /// 开关走没走到，读这个数就看得出（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
     placements_reclaimed_on_release_by_the_forced_zero_reuse_window: u64,
-    /// 这个分配器实例上提交内生块走回落（`CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot`，D3（空间分配） 已定项 8 ②）的次数，
+    /// 这个分配器实例上提交内生块走回落（`CommitGeneratedDeviceAnswer::FallBack`，D3（空间分配） 已定项 8 ②）的次数，
     /// 只增、只住内存、`new` 与 `rebuild_from_records` 起步是 0。只读，产品路径不按它分支：它是回落那一支的观测
     /// （`.claude/rules/fs-design.md` 五条硬要求第 4 条「分支必须可观测」）。harness 的看门用例在「mkfs 之后连续可写挂载、再写第一个文件」
     /// 那一族上断言它是 0：I-5.5 的开放段认法（池级 checker 拿树表单元所在的段当开放段）假定这一族不走回落，哪天走进回落，用例红、
     /// 回来重看那一格。
-    commit_generated_placements_that_fell_back_to_the_lowest_free_slot: u64,
+    commit_generated_placements_that_fell_back: u64,
     /// 分配器眼里的根环（`RootRingOccupancy`）：挂载时装（`mount::rebuilt_allocator`），mkfs 同一个进程里由建分配器的一方装
     /// （`make_filesystem::root_ring_occupancy_after_make_filesystem`）。`None`：建分配器的一方没装（`new` 与 `rebuild_from_records`
     /// 都不装）。没装的分配器不许记根：`record_root_written_by_this_process` 断言（C551），只做不发布的事：分配，
@@ -2213,7 +2357,7 @@ impl PoolAllocator {
             allocation_record_tree_of_the_version_without_file: None,
             reuse_window: ReuseWindow::GatedByTheRollbackFloor,
             placements_reclaimed_on_release_by_the_forced_zero_reuse_window: 0,
-            commit_generated_placements_that_fell_back_to_the_lowest_free_slot: 0,
+            commit_generated_placements_that_fell_back: 0,
             root_ring: None,
             frozen_publish: None,
             instance_rows_after_this_mounts_row_publish: 0,
@@ -2507,11 +2651,11 @@ impl PoolAllocator {
         self.placements_reclaimed_on_release_by_the_forced_zero_reuse_window
     }
 
-    /// 这个分配器实例上提交内生块走回落（`CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot`）的次数：每取一个回落的落点加一。
+    /// 这个分配器实例上提交内生块走回落（`CommitGeneratedDeviceAnswer::FallBack`）的次数：每取一个回落的落点加一。
     /// 可写挂载交回的分配器（`mount::Mounted::allocator`）带着挂载里写行、暖机、抬 F 与续做那几次发布数下的；重开之后从 0 数起。
     #[must_use]
-    pub fn commit_generated_placements_that_fell_back_to_the_lowest_free_slot(&self) -> u64 {
-        self.commit_generated_placements_that_fell_back_to_the_lowest_free_slot
+    pub fn commit_generated_placements_that_fell_back(&self) -> u64 {
+        self.commit_generated_placements_that_fell_back
     }
 
     /// 重开时从盘上那棵分配记录树重建分配器（D23（journal 的角色与格式） 已定项 14：defer 队列、分配器游标、记账的现行值从所选根那棵账重新载入）：
@@ -2850,7 +2994,9 @@ impl PoolAllocator {
     /// 每块盘的答案都换成它；有的盘答开段、有的盘答回落，或池里取不到每块盘都空的回落落点，照各盘自己的答案判一致——各盘答得一致才分配。
     ///
     /// # Errors
-    /// 拒绝时分配器一样都没动（装不下的开放段也照旧开着）：全部盘都没有去处、有的盘没有、各盘的去处不同。
+    /// 拒绝时分配器一样都没动（装不下的开放段也照旧开着）：全部盘都没有去处、有的盘没有、各盘的去处不同
+    /// （每块盘都答回落而池里取不到每块盘都空的落点时报 [`PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice`]，
+    /// 别的不同报段对齐那一条）。
     pub fn try_allocate_commit_generated_preferring_the_rewrite_set(
         &mut self,
         footprint: UnitFootprint,
@@ -2876,12 +3022,9 @@ impl PoolAllocator {
                 )
             })
             .collect();
-        let every_device_falls_back = answer_per_device.iter().all(|(_, answer)| {
-            matches!(
-                answer,
-                Some(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(_))
-            )
-        });
+        let every_device_falls_back = answer_per_device
+            .iter()
+            .all(|(_, answer)| matches!(answer, Some(CommitGeneratedDeviceAnswer::FallBack(_))));
         let answer_per_device = match every_device_falls_back
             .then(|| {
                 self.commit_generated_fallback_slot_open_on_every_device(
@@ -2897,7 +3040,7 @@ impl PoolAllocator {
                 .map(|(device, _)| {
                     (
                         device,
-                        Some(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(
+                        Some(CommitGeneratedDeviceAnswer::FallBack(
                             fallback_slot_on_every_device,
                         )),
                     )
@@ -2916,11 +3059,9 @@ impl PoolAllocator {
                 );
             }
             DeviceAgreement::AnswersDiffer(answer_per_device) => {
-                return Err(
-                    PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
-                        answer_per_device,
-                    },
-                );
+                return Err(refusal_of_commit_generated_answers_that_differ(
+                    answer_per_device,
+                ));
             }
         };
         match answer {
@@ -2933,11 +3074,11 @@ impl PoolAllocator {
                     .expect("每块盘都答了同一个全空段（已分配、隔离、扣住都是 0，整段在每块盘的单元区里）：64 槽装得下任何一个单元");
                 Ok(self.record_bumped(slot, footprint, generation))
             }
-            CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(slot) => {
+            CommitGeneratedDeviceAnswer::FallBack(slot) => {
                 // 回落之后没有段可 bump 了，但刚才那一段仍是聚簇段、仍装着这次提交的内生块：它留在 `cluster_segments` 里，
                 // 用户数据照旧不许落进去（D3（空间分配） 已定项 8 第 2 条；增补 2 第 20c 行）。
                 self.open_segment = None;
-                self.commit_generated_placements_that_fell_back_to_the_lowest_free_slot += 1;
+                self.commit_generated_placements_that_fell_back += 1;
                 let placement = Placement {
                     slot,
                     span: footprint.slots(),
@@ -2986,31 +3127,36 @@ impl PoolAllocator {
         footprint: UnitFootprint,
         excluded_region: Option<CompactionRegion>,
     ) -> Option<SlotNumber> {
-        let (first_leaf_index, last_leaf_index) =
-            self.devices
-                .iter()
-                .fold(None, |leaves_so_far: Option<(u64, u64)>, device| {
-                    match (
-                        leaves_so_far,
-                        device.allocation_record_tree_leaves_of_the_unit_area(),
-                    ) {
-                        (None, leaves_of_the_device) => leaves_of_the_device,
-                        (Some(leaves), None) => Some(leaves),
-                        (Some((first, last)), Some((first_of_the_device, last_of_the_device))) => {
-                            Some((first.min(first_of_the_device), last.max(last_of_the_device)))
-                        }
+        let (first_leaf_index, last_leaf_index) = self.devices.iter().fold(
+            None,
+            |leaves_so_far: Option<(
+                AllocationRecordTreeLeafIndex,
+                AllocationRecordTreeLeafIndex,
+            )>,
+             device| {
+                match (
+                    leaves_so_far,
+                    device.allocation_record_tree_leaves_of_the_unit_area(),
+                ) {
+                    (None, leaves_of_the_device) => leaves_of_the_device,
+                    (Some(leaves), None) => Some(leaves),
+                    (Some((first, last)), Some((first_of_the_device, last_of_the_device))) => {
+                        Some((first.min(first_of_the_device), last.max(last_of_the_device)))
                     }
-                })?;
-        let mut most_open_slots_so_far: u64 = 0;
+                }
+            },
+        )?;
+        let mut most_open_slots_so_far = SlotCountInALeaf(0);
         let mut slot_in_the_leaf_with_the_most_open_slots: Option<SlotNumber> = None;
         // 迭代上界是单元区罩到的叶数；跨轮带的是眼下最多的那片叶的数与它里面的落点，没有提前出口。
-        for leaf_index in first_leaf_index..=last_leaf_index {
+        for leaf_number in first_leaf_index.0..=last_leaf_index.0 {
+            let leaf_index = AllocationRecordTreeLeafIndex(leaf_number);
             let upper_bound_of_the_open_slots = self
                 .devices
                 .iter()
                 .map(|device| device.unblocked_slots_in_allocation_record_tree_leaf(leaf_index))
                 .min()
-                .unwrap_or(0);
+                .unwrap_or(SlotCountInALeaf(0));
             if upper_bound_of_the_open_slots <= most_open_slots_so_far {
                 continue;
             }
@@ -3034,12 +3180,11 @@ impl PoolAllocator {
     /// 分配记录树第 `leaf_index` 片叶里每块盘都空的单槽数（单元区外、活 `R` 里的不算）。
     fn slots_open_on_every_device_in_the_leaf(
         &self,
-        leaf_index: u64,
+        leaf_index: AllocationRecordTreeLeafIndex,
         excluded_region: Option<CompactionRegion>,
-    ) -> u64 {
-        let first_slot_of_the_leaf = leaf_index * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
-        let open_slots = (first_slot_of_the_leaf
-            ..first_slot_of_the_leaf + ALLOCATION_RECORD_TREE_LEAF_SLOTS)
+    ) -> SlotCountInALeaf {
+        let open_slots = (leaf_index.first_slot().0..leaf_index.past_the_last_slot().0)
+            .map(SlotNumber)
             .filter(|slot| {
                 self.devices.iter().all(|device| {
                     device.is_whole_span_open_for_commit_generated(
@@ -3050,22 +3195,22 @@ impl PoolAllocator {
                 })
             })
             .count();
-        u64::try_from(open_slots).expect("一片叶的槽数装得进 u64")
+        SlotCountInALeaf(u64::try_from(open_slots).expect(
+            "数的是一片叶里的槽，至多 ALLOCATION_RECORD_TREE_LEAF_SLOTS（812）个，装得进 u64",
+        ))
     }
 
     /// 分配记录树第 `leaf_index` 片叶里起点最低、整个跨度每块盘都空的落点（起点按跨度的对齐档步进；叶起点是偶数）；没有就 `None`。
     fn lowest_slot_open_on_every_device_in_the_leaf(
         &self,
-        leaf_index: u64,
+        leaf_index: AllocationRecordTreeLeafIndex,
         footprint: UnitFootprint,
         excluded_region: Option<CompactionRegion>,
     ) -> Option<SlotNumber> {
-        let first_slot_of_the_leaf = leaf_index * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
-        let past_the_last_slot_of_the_leaf =
-            first_slot_of_the_leaf + ALLOCATION_RECORD_TREE_LEAF_SLOTS;
-        let mut candidate = first_slot_of_the_leaf;
+        let past_the_last_slot_of_the_leaf = leaf_index.past_the_last_slot();
+        let mut candidate = leaf_index.first_slot();
         // 迭代上界是一片叶的槽数 ÷ 对齐步长；没有跨轮携带的状态，找到就出。
-        while candidate + footprint.slots() <= past_the_last_slot_of_the_leaf {
+        while candidate.0 + footprint.slots() <= past_the_last_slot_of_the_leaf.0 {
             if self.devices.iter().all(|device| {
                 device.is_whole_span_open_for_commit_generated(
                     candidate,
@@ -3073,9 +3218,9 @@ impl PoolAllocator {
                     excluded_region,
                 )
             }) {
-                return Some(SlotNumber(candidate));
+                return Some(candidate);
             }
-            candidate += footprint.alignment_in_slots();
+            candidate = SlotNumber(candidate.0 + footprint.alignment_in_slots());
         }
         None
     }
@@ -4672,8 +4817,12 @@ mod tests {
     }
 
     /// 两块盘上都在重写集里的叶。
-    fn rewrite_set_leaves_of_both_devices(leaf_indices: &[u64]) -> RewriteSetLeaves {
-        let on_each_device: BTreeSet<u64> = leaf_indices.iter().copied().collect();
+    fn rewrite_set_leaves_of_both_devices(leaf_numbers: &[u64]) -> RewriteSetLeaves {
+        let on_each_device: BTreeSet<AllocationRecordTreeLeafIndex> = leaf_numbers
+            .iter()
+            .copied()
+            .map(AllocationRecordTreeLeafIndex)
+            .collect();
         RewriteSetLeaves::present_on_every_device(&[on_each_device.clone(), on_each_device])
     }
 
@@ -4719,10 +4868,7 @@ mod tests {
             "重写集的叶 62 里最低的空槽"
         );
         assert_eq!(preferring.open_segment(), None, "回落不开段");
-        assert_eq!(
-            preferring.commit_generated_placements_that_fell_back_to_the_lowest_free_slot(),
-            1
-        );
+        assert_eq!(preferring.commit_generated_placements_that_fell_back(), 1);
 
         let mut without_a_rewrite_set =
             pool_after_mkfs_whose_next_commit_generated_block_falls_back();
@@ -4742,8 +4888,10 @@ mod tests {
             "空槽最多的叶 63 里最低的空槽"
         );
 
-        let only_on_one_device =
-            RewriteSetLeaves::present_on_every_device(&[BTreeSet::from([62]), BTreeSet::new()]);
+        let only_on_one_device = RewriteSetLeaves::present_on_every_device(&[
+            BTreeSet::from([AllocationRecordTreeLeafIndex(62)]),
+            BTreeSet::new(),
+        ]);
         assert!(
             only_on_one_device.leaf_indices().is_empty(),
             "只在一块盘上重写的叶不算"
@@ -4763,51 +4911,80 @@ mod tests {
         );
     }
 
-    /// 重写集的叶里没有空槽（叶 75 全占着）就取空槽最多的叶：叶 90 与叶 100 各 5 个空槽、叶 70 只有 3 个 ⇒ 取叶 90 里最低的 73180，
-    /// 不取槽号最小的 56850（叶 70），一样多时不取叶序号大的叶 100。两槽的单元取叶 90 里最低的一对偶数起点空槽 73180–73181。
+    /// [`pool_with_free_slots_left_in_three_leaves`] 造出来的形态：没有全空段；每块盘上叶 90 没挡 5 槽、叶 70 没挡 3 槽。
+    fn assert_the_pool_with_free_slots_left_in_three_leaves_has_no_empty_segment(
+        pool: &PoolAllocator,
+    ) {
+        for device_map in &pool.devices {
+            assert_eq!(
+                device_map.lowest_empty_segment(),
+                None,
+                "造的形态没有全空段"
+            );
+            assert_eq!(
+                device_map.unblocked_slots_in_allocation_record_tree_leaf(
+                    AllocationRecordTreeLeafIndex(90)
+                ),
+                SlotCountInALeaf(5)
+            );
+            assert_eq!(
+                device_map.unblocked_slots_in_allocation_record_tree_leaf(
+                    AllocationRecordTreeLeafIndex(70)
+                ),
+                SlotCountInALeaf(3)
+            );
+        }
+    }
+
+    /// 重写集的叶里没有空槽（叶 75 全占着）就取空槽最多的叶：叶 90 与叶 100 各 5 个空槽、叶 70 只有 3 个 ⇒ 单槽的单元取叶 90 里最低的 73180，
+    /// 不取槽号最小的 56850（叶 70），一样多时不取叶序号大的叶 100。
     #[test]
-    fn fallback_without_room_in_the_rewrite_set_leaves_takes_the_leaf_with_the_most_free_slots() {
-        for (footprint, expected) in [
-            (
+    fn fallback_of_a_one_slot_unit_without_room_in_the_rewrite_set_leaves_takes_the_lowest_slot_of_the_leaf_with_the_most_free_slots(
+    ) {
+        let mut pool = pool_with_free_slots_left_in_three_leaves();
+        assert_the_pool_with_free_slots_left_in_three_leaves_has_no_empty_segment(&pool);
+        let placement = pool
+            .try_allocate_commit_generated_preferring_the_rewrite_set(
                 UnitFootprint::OneSlot,
-                Placement {
-                    slot: SlotNumber(73_180),
-                    span: 1,
-                },
-            ),
-            (
+                CheckpointTxg(4),
+                &rewrite_set_leaves_of_both_devices(&[75]),
+            )
+            .expect("回落");
+        assert_eq!(
+            placement,
+            Placement {
+                slot: SlotNumber(73_180),
+                span: 1,
+            }
+        );
+    }
+
+    /// 同一形态上两槽的单元：取空槽最多、叶序号小的叶 90 里最低的一对偶数起点空槽 73180–73181。
+    #[test]
+    fn fallback_of_a_two_slot_unit_without_room_in_the_rewrite_set_leaves_takes_the_lowest_even_pair_of_the_leaf_with_the_most_free_slots(
+    ) {
+        let mut pool = pool_with_free_slots_left_in_three_leaves();
+        assert_the_pool_with_free_slots_left_in_three_leaves_has_no_empty_segment(&pool);
+        let placement = pool
+            .try_allocate_commit_generated_preferring_the_rewrite_set(
                 UnitFootprint::TwoSlotsAligned,
-                Placement {
-                    slot: SlotNumber(73_180),
-                    span: 2,
-                },
-            ),
-        ] {
-            let mut pool = pool_with_free_slots_left_in_three_leaves();
-            for device_map in &pool.devices {
-                assert_eq!(
-                    device_map.lowest_empty_segment(),
-                    None,
-                    "造的形态没有全空段"
-                );
-                assert_eq!(
-                    device_map.unblocked_slots_in_allocation_record_tree_leaf(90),
-                    5
-                );
-                assert_eq!(
-                    device_map.unblocked_slots_in_allocation_record_tree_leaf(70),
-                    3
-                );
+                CheckpointTxg(4),
+                &rewrite_set_leaves_of_both_devices(&[75]),
+            )
+            .expect("回落");
+        assert_eq!(
+            placement,
+            Placement {
+                slot: SlotNumber(73_180),
+                span: 2,
             }
-            let placement = pool
-                .try_allocate_commit_generated_preferring_the_rewrite_set(
-                    footprint,
-                    CheckpointTxg(4),
-                    &rewrite_set_leaves_of_both_devices(&[75]),
-                )
-                .expect("回落");
-            assert_eq!(placement, expected, "{footprint:?}");
-        }
+        );
+    }
+
+    /// 同一形态上重写集的叶是只有 3 个空槽的叶 70：重写集的叶里有空槽就先取它（56850），哪怕它空槽比叶 90、叶 100 少。
+    #[test]
+    fn fallback_takes_a_free_slot_of_a_rewrite_set_leaf_poorer_than_the_leaf_with_the_most_free_slots(
+    ) {
         let mut preferring_the_poorer_leaf = pool_with_free_slots_left_in_three_leaves();
         let placement = preferring_the_poorer_leaf
             .try_allocate_commit_generated_preferring_the_rewrite_set(
@@ -4897,14 +5074,21 @@ mod tests {
     #[test]
     fn unblocked_slots_per_allocation_record_tree_leaf_are_maintained_incrementally_and_match_a_scan(
     ) {
-        fn unblocked_slots_in_the_leaf_by_scan(device_map: &DeviceFreeMap, leaf_index: u64) -> u64 {
-            let first = (leaf_index * ALLOCATION_RECORD_TREE_LEAF_SLOTS).max(UNIT_AREA_START_SLOT);
-            let past = ((leaf_index + 1) * ALLOCATION_RECORD_TREE_LEAF_SLOTS)
+        fn unblocked_slots_in_the_leaf_by_scan(
+            device_map: &DeviceFreeMap,
+            leaf_index: AllocationRecordTreeLeafIndex,
+        ) -> SlotCountInALeaf {
+            let first = leaf_index.first_slot().0.max(UNIT_AREA_START_SLOT);
+            let past = leaf_index
+                .past_the_last_slot()
+                .0
                 .min(UNIT_AREA_START_SLOT + device_map.unit_area_slots());
             let unblocked = (first..past)
                 .filter(|slot| !device_map.is_blocked_for_commit_generated(SlotNumber(*slot)))
                 .count();
-            u64::try_from(unblocked).expect("一片叶的槽数")
+            SlotCountInALeaf(
+                u64::try_from(unblocked).expect("数的是一片叶里的槽，至多 812 个，装得进 u64"),
+            )
         }
         let mut pool = pool_after_mkfs_whose_next_commit_generated_block_falls_back();
         pool.isolate_abandoned(DeviceIdentity(1), SlotNumber(51_160), 3);
@@ -4921,22 +5105,33 @@ mod tests {
             let (first_leaf, last_leaf) = device_map
                 .allocation_record_tree_leaves_of_the_unit_area()
                 .expect("单元区不空");
-            assert_eq!((first_leaf, last_leaf), (61, 322));
-            for leaf_index in first_leaf..=last_leaf {
+            assert_eq!(
+                (first_leaf, last_leaf),
+                (
+                    AllocationRecordTreeLeafIndex(61),
+                    AllocationRecordTreeLeafIndex(322)
+                )
+            );
+            for leaf_number in first_leaf.0..=last_leaf.0 {
+                let leaf_index = AllocationRecordTreeLeafIndex(leaf_number);
                 assert_eq!(
                     device_map.unblocked_slots_in_allocation_record_tree_leaf(leaf_index),
                     unblocked_slots_in_the_leaf_by_scan(device_map, leaf_index),
-                    "盘 {:?} 叶 {leaf_index}",
+                    "盘 {:?} 叶 {leaf_number}",
                     device_map.device
                 );
             }
             assert_eq!(
-                device_map.unblocked_slots_in_allocation_record_tree_leaf(first_leaf - 1),
-                0
+                device_map.unblocked_slots_in_allocation_record_tree_leaf(
+                    AllocationRecordTreeLeafIndex(first_leaf.0 - 1)
+                ),
+                SlotCountInALeaf(0)
             );
             assert_eq!(
-                device_map.unblocked_slots_in_allocation_record_tree_leaf(last_leaf + 1),
-                0
+                device_map.unblocked_slots_in_allocation_record_tree_leaf(
+                    AllocationRecordTreeLeafIndex(last_leaf.0 + 1)
+                ),
+                SlotCountInALeaf(0)
             );
         }
     }
@@ -5031,6 +5226,296 @@ mod tests {
         );
     }
 
+    /// 每块盘都答回落、两块盘的空槽错开（盘 0 只剩 56850、盘 1 只剩 56851）：池里取不到每块盘都空的落点，
+    /// 报「回落取不到每块盘都空的落点」并带上各盘自己答的回落落点，不报段对齐那一条；分配器一样都没动（记录、回落计数、开放段）。
+    #[test]
+    fn fallback_without_a_slot_free_on_every_device_is_refused_naming_each_devices_own_fallback_slot(
+    ) {
+        let mut pool = pool_after_mkfs();
+        fill_every_free_slot_except(&mut pool, &[[56_850].as_slice(), [56_851].as_slice()]);
+        let records_before = pool.records().to_vec();
+        let refusal = pool
+            .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
+            .expect_err("没有一个槽在两块盘上都空");
+        assert_eq!(
+            refusal,
+            PlacementRefusal::CommitGeneratedFallbackWithoutASlotFreeOnEveryDevice {
+                fallback_slot_per_device: vec![
+                    (DeviceIdentity(0), SlotNumber(56_850)),
+                    (DeviceIdentity(1), SlotNumber(56_851)),
+                ],
+            }
+        );
+        assert_eq!(pool.records(), records_before.as_slice(), "记录一条没动");
+        assert_eq!(pool.commit_generated_placements_that_fell_back(), 0);
+        assert_eq!(pool.open_segment(), None);
+    }
+
+    /// 用户数据落点对拍用的一串定种子伪随机操作（两块盘，单元区各 `unit_area_slots_per_device` 槽，可以不等大）：分配数据单元、
+    /// 开段 / bump / 回落一个提交内生块、释放、回收（立即可用或抬 F 扣住）、放开扣住、直接扣住空槽、影子账隔离与清隔离（两块盘各自）、
+    /// 签发与撤掉活整理意图的 `R`、克隆分配器接着在拷贝上走，各走到。每一步之后把分配器交给 `check_after_the_step`（步号与这一步做了什么）。
+    fn random_history_for_the_user_data_placement(
+        seed: u64,
+        unit_area_slots_per_device: [u64; 2],
+        steps: usize,
+        check_after_the_step: &dyn Fn(&PoolAllocator, usize, &str),
+    ) {
+        let mut pool = PoolAllocator::new(vec![
+            DeviceFreeMap::new(
+                DeviceIdentity(0),
+                (UNIT_AREA_START_SLOT + unit_area_slots_per_device[0]) * SLOT_BYTES,
+            ),
+            DeviceFreeMap::new(
+                DeviceIdentity(1),
+                (UNIT_AREA_START_SLOT + unit_area_slots_per_device[1]) * SLOT_BYTES,
+            ),
+        ]);
+        let mut random_state = seed;
+        let mut next_random = |below: u64| {
+            random_state ^= random_state << 13;
+            random_state ^= random_state >> 7;
+            random_state ^= random_state << 17;
+            random_state % below
+        };
+        let mut generation = 1u64;
+        let mut placements_allocated: Vec<Placement> = Vec::new();
+        check_after_the_step(&pool, 0, "空盘");
+        for step_number in 1..=steps {
+            let what_the_step_did = match next_random(12) {
+                0 | 1 => {
+                    if let Some(placement) = pool.allocate_user_data(CheckpointTxg(generation)) {
+                        placements_allocated.push(placement);
+                    }
+                    "分配一个数据单元"
+                }
+                2 | 3 => {
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
+                    "开段、bump 或回落一个提交内生块"
+                }
+                4 | 5 => {
+                    if !placements_allocated.is_empty() {
+                        let position = usize::try_from(next_random(
+                            u64::try_from(placements_allocated.len()).expect("个数"),
+                        ))
+                        .expect("下标");
+                        let placement = placements_allocated.swap_remove(position);
+                        generation += 1;
+                        pool.release(placement, CheckpointTxg(generation));
+                    }
+                    "释放一个落点进 defer"
+                }
+                6 => {
+                    let reuse = if next_random(2) == 0 {
+                        ReclaimedReuse::Immediately
+                    } else {
+                        ReclaimedReuse::HeldUntilFloorTakesEffect
+                    };
+                    pool.reclaim_released_up_to(CheckpointTxg(generation), reuse);
+                    "回收 defer 里的全部（立即可用或抬 F 扣住）"
+                }
+                7 => {
+                    pool.release_reclaim_holds();
+                    "抬 F 生效、放开扣住"
+                }
+                8 => {
+                    let device_position = usize::try_from(next_random(2)).expect("盘的下标");
+                    let device_map = &mut pool.devices[device_position];
+                    let slot = SlotNumber(
+                        UNIT_AREA_START_SLOT + next_random(device_map.unit_area_slots() - 1),
+                    );
+                    for slot_of_the_span in [slot, SlotNumber(slot.0 + 1)] {
+                        if device_map.is_free(slot_of_the_span) {
+                            device_map.hold_until_floor_takes_effect(slot_of_the_span, 1);
+                        }
+                    }
+                    "一块盘上直接扣住两个空槽"
+                }
+                9 => {
+                    let device = DeviceIdentity(u32::try_from(next_random(2)).expect("盘号"));
+                    let device_position = usize::try_from(device.0).expect("盘的下标");
+                    let unit_area_slots = pool.devices[device_position].unit_area_slots();
+                    let slot = SlotNumber(UNIT_AREA_START_SLOT + next_random(unit_area_slots - 1));
+                    if next_random(2) == 0 {
+                        pool.isolate_abandoned(device, slot, 1 + next_random(2));
+                    } else {
+                        pool.devices[device_position].clear_isolation_of_slot(slot);
+                    }
+                    "一块盘上隔离或清隔离"
+                }
+                10 => {
+                    if pool.compaction().live_intent().is_some() {
+                        pool.compaction_mut().withdraw_the_live_intent();
+                        "撤掉活整理意图的 R"
+                    } else {
+                        let smaller_unit_area =
+                            unit_area_slots_per_device[0].min(unit_area_slots_per_device[1]);
+                        let segment_index = next_random(smaller_unit_area / CLUSTER_SEGMENT_SLOTS);
+                        pool.compaction_mut()
+                            .issue(crate::logged_operations::CompactionIntent {
+                                region: CompactionRegion::of_one_cluster_segment(
+                                    DeviceIdentity(0),
+                                    SlotNumber(
+                                        UNIT_AREA_START_SLOT
+                                            + segment_index * CLUSTER_SEGMENT_SLOTS,
+                                    ),
+                                )
+                                .expect("单元区起点在段边界上，段起点是 64 的倍数"),
+                                allocation_generation_at_issue: CheckpointTxg(generation),
+                                empty_cluster_segments_at_issue: 0,
+                            });
+                        "签发活整理意图的 R"
+                    }
+                }
+                _ => {
+                    let cloned_allocator = pool.clone();
+                    pool = cloned_allocator;
+                    "克隆分配器，接着在拷贝上走"
+                }
+            };
+            check_after_the_step(&pool, step_number, what_the_step_did);
+        }
+    }
+
+    /// 每块盘上按段取的用户数据落点与从单元区起点逐个候选看的参照取法答得一样：带着这次挂载开过的聚簇段与此刻的 `R` 问一次，
+    /// 不带 `R` 再问一次。
+    fn assert_user_data_slot_by_segment_matches_stepping_through_every_candidate(
+        seed: u64,
+        pool: &PoolAllocator,
+        step_number: usize,
+        what_the_step_did: &str,
+    ) {
+        for device_map in &pool.devices {
+            for excluded_region in [pool.region_excluded_from_placement(), None] {
+                assert_eq!(
+                    device_map.lowest_user_data_slot_outside_the_region(
+                        pool.cluster_segments(),
+                        excluded_region
+                    ),
+                    device_map.lowest_user_data_slot_outside_the_region_by_stepping_through_every_candidate(
+                        pool.cluster_segments(),
+                        excluded_region
+                    ),
+                    "种子 {seed:#x} 第 {step_number} 步（{what_the_step_did}）：盘 {:?}、R {excluded_region:?}、聚簇段 {:?}",
+                    device_map.device,
+                    pool.cluster_segments()
+                );
+            }
+        }
+    }
+
+    /// 两块等大的盘、单元区 385 槽（末尾一段只有一槽、不成对）：一串 1500 步的随机操作，每一步之后按段取与逐个候选看答得一样。
+    /// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
+    #[test]
+    fn user_data_slot_found_by_segment_matches_stepping_through_every_candidate_on_two_equal_devices_ending_in_an_unpaired_slot(
+    ) {
+        let seed = 0x5EED_0000_0000_0385;
+        println!("种子 {seed:#x}");
+        random_history_for_the_user_data_placement(
+            seed,
+            [385, 385],
+            1500,
+            &|pool, step_number, what_the_step_did| {
+                assert_user_data_slot_by_segment_matches_stepping_through_every_candidate(
+                    seed,
+                    pool,
+                    step_number,
+                    what_the_step_did,
+                );
+            },
+        );
+    }
+
+    /// 两块不等大的盘（单元区 385 槽与 1000 槽）：大盘上小盘够不着的那几段只在大盘的集合里，按段取照样与逐个候选看答得一样。
+    /// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
+    #[test]
+    fn user_data_slot_found_by_segment_matches_stepping_through_every_candidate_on_two_unequal_devices(
+    ) {
+        let seed = 0x5EED_0000_0385_1000;
+        println!("种子 {seed:#x}");
+        random_history_for_the_user_data_placement(
+            seed,
+            [385, 1000],
+            1500,
+            &|pool, step_number, what_the_step_did| {
+                assert_user_data_slot_by_segment_matches_stepping_through_every_candidate(
+                    seed,
+                    pool,
+                    step_number,
+                    what_the_step_did,
+                );
+            },
+        );
+    }
+
+    /// 两块等大的盘、单元区 2000 槽（31 个整段加 16 槽）：段多、开过的聚簇段攒得多，一串 3000 步的随机操作，每一步之后按段取与逐个候选看答得一样。
+    /// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
+    #[test]
+    fn user_data_slot_found_by_segment_matches_stepping_through_every_candidate_on_two_equal_devices_of_thirty_one_full_segments(
+    ) {
+        let seed = 0x5EED_0000_0000_2000;
+        println!("种子 {seed:#x}");
+        random_history_for_the_user_data_placement(
+            seed,
+            [2000, 2000],
+            3000,
+            &|pool, step_number, what_the_step_did| {
+                assert_user_data_slot_by_segment_matches_stepping_through_every_candidate(
+                    seed,
+                    pool,
+                    step_number,
+                    what_the_step_did,
+                );
+            },
+        );
+    }
+
+    /// 有没挡槽对的段号集合（`segments_with_an_unblocked_slot_pair`）是增量维护的：一串随机操作的每一步之后，拿逐对扫出来的
+    /// 「两槽都空（`is_free`）的偶数起点槽对所在的段」对，两块盘都要对得上。
+    /// 判别力：槽对放开时不把那一段加回集合，放开之后那一段就少了。
+    /// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
+    #[test]
+    fn segments_with_an_unblocked_slot_pair_are_maintained_incrementally_and_match_a_scan() {
+        let seed = 0x5EED_0000_0000_0385;
+        println!("种子 {seed:#x}");
+        random_history_for_the_user_data_placement(
+            seed,
+            [385, 385],
+            1500,
+            &|pool, step_number, what_the_step_did| {
+                for device_map in &pool.devices {
+                    let first_slot = device_map.unit_area_start().slot().0;
+                    let past_the_last_slot = first_slot + device_map.unit_area_slots();
+                    let segments_by_scan: BTreeSet<usize> = (first_slot..past_the_last_slot)
+                        .step_by(2)
+                        .filter(|pair_start| {
+                            pair_start + 1 < past_the_last_slot
+                                && device_map.is_free(SlotNumber(*pair_start))
+                                && device_map.is_free(SlotNumber(pair_start + 1))
+                        })
+                        .map(|pair_start| {
+                            usize::try_from((pair_start - first_slot) / CLUSTER_SEGMENT_SLOTS)
+                                .expect("段号")
+                        })
+                        .collect();
+                    assert_eq!(
+                        device_map.segments_with_an_unblocked_slot_pair, segments_by_scan,
+                        "种子 {seed:#x} 第 {step_number} 步（{what_the_step_did}）：盘 {:?}",
+                        device_map.device
+                    );
+                }
+            },
+        );
+    }
+
     /// 根环 R × S = 24 槽，txg 0–23 各一条有效根，另有一个释放代 1 的落点在 defer 队列里。
     fn pool_whose_root_ring_holds_txg_0_through_23() -> PoolAllocator {
         let mut pool = rebuilt_pool(records_on_both_devices(&[(50180, 2, 1, true)]));
```

```diff
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index ce205c4..f7d8707 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -50,7 +50,8 @@ use crate::allocation_record_tree::{
     AllocationRecordTreeNodePosition, AllocationRecordTreePlan, AllocationRecordTreeVersion,
 };
 use crate::allocator::{
-    AllocationRecord, Placement, PlacementRefusal, PoolAllocator, RewriteSetLeaves, UnitFootprint,
+    AllocationRecord, AllocationRecordTreeLeafIndex, Placement, PlacementRefusal, PoolAllocator,
+    RewriteSetLeaves, UnitFootprint,
 };
 use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
 use crate::checksum::crc32_castagnoli;
@@ -2519,7 +2520,7 @@ fn rewrite_set_leaves_of_the_roles(
     rewritten_roles: &[TransactionUnit],
     allocator: &PoolAllocator,
 ) -> RewriteSetLeaves {
-    let leaf_indices_per_device: Vec<BTreeSet<u64>> = allocator
+    let leaf_indices_per_device: Vec<BTreeSet<AllocationRecordTreeLeafIndex>> = allocator
         .devices
         .iter()
         .map(|device_map| {
@@ -2529,7 +2530,7 @@ fn rewrite_set_leaves_of_the_roles(
                     Some(AllocationRecordTreeNode::BelowTheRoot(position))
                         if position.level == 0 && position.device == device_map.device =>
                     {
-                        Some(position.index_in_device)
+                        Some(AllocationRecordTreeLeafIndex(position.index_in_device))
                     }
                     Some(
                         AllocationRecordTreeNode::BelowTheRoot(_) | AllocationRecordTreeNode::Root,
```

## 补丁 `~/.local/share/singlefs/work/user-data-placement-scan-writer-2/patch/crates.patch`（产品代码部分 1 份文件）

```diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index ab4be3b..f39d9b3 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -5478,6 +5478,36 @@ mod tests {
         );
     }
 
+    /// 对拍的定形取样点（随机历史未必走到这一形）：一块盘三个整段全空，这次挂载开过的聚簇段是第 0、1 两段，
+    /// 段外最低的一对是第 2 段段首。按段取要把两段都跳过、与逐个候选看答得一样；只跳过集合里最后那一段的取法会落进第 0 段。
+    #[test]
+    fn user_data_slot_found_by_segment_skips_every_cluster_segment_of_this_mount_not_only_the_last()
+    {
+        let device_map = DeviceFreeMap::new(
+            DeviceIdentity(0),
+            (UNIT_AREA_START_SLOT + 3 * CLUSTER_SEGMENT_SLOTS) * SLOT_BYTES,
+        );
+        let cluster_segments = BTreeSet::from([
+            SlotNumber(UNIT_AREA_START_SLOT),
+            SlotNumber(UNIT_AREA_START_SLOT + CLUSTER_SEGMENT_SLOTS),
+        ]);
+        let by_stepping_through_every_candidate = device_map
+            .lowest_user_data_slot_outside_the_region_by_stepping_through_every_candidate(
+                &cluster_segments,
+                None,
+            );
+        assert_eq!(
+            by_stepping_through_every_candidate,
+            Some(SlotNumber(UNIT_AREA_START_SLOT + 2 * CLUSTER_SEGMENT_SLOTS)),
+            "参照取法：两段聚簇段都跳过，落在第 2 段段首"
+        );
+        assert_eq!(
+            device_map.lowest_user_data_slot_outside_the_region(&cluster_segments, None),
+            by_stepping_through_every_candidate,
+            "按段取与逐个候选看答得一样：这次挂载开过的两段聚簇段都要跳过，聚簇段 {cluster_segments:?}"
+        );
+    }
+
     /// 有没挡槽对的段号集合（`segments_with_an_unblocked_slot_pair`）是增量维护的：一串随机操作的每一步之后，拿逐对扫出来的
     /// 「两槽都空（`is_free`）的偶数起点槽对所在的段」对，两块盘都要对得上。
     /// 判别力：槽对放开时不把那一段加回集合，放开之后那一段就少了。
```

## 补丁 `~/.local/share/singlefs/work/user-data-placement-scan-writer-3/patch/crates.patch`（产品代码部分 1 份文件）

```diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -5508,12 +5508,66 @@
         );
     }
 
-    /// 有没挡槽对的段号集合（`segments_with_an_unblocked_slot_pair`）是增量维护的：一串随机操作的每一步之后，拿逐对扫出来的
-    /// 「两槽都空（`is_free`）的偶数起点槽对所在的段」对，两块盘都要对得上。
-    /// 判别力：槽对放开时不把那一段加回集合，放开之后那一段就少了。
+    /// 逐对扫出来的「两槽都空（`is_free`）的偶数起点槽对所在的段」：增量维护的 `segments_with_an_unblocked_slot_pair` 拿它对。
+    fn segments_with_an_unblocked_slot_pair_by_scan(device_map: &DeviceFreeMap) -> BTreeSet<usize> {
+        let first_slot = device_map.unit_area_start().slot().0;
+        let past_the_last_slot = first_slot + device_map.unit_area_slots();
+        (first_slot..past_the_last_slot)
+            .step_by(2)
+            .filter(|pair_start| {
+                pair_start + 1 < past_the_last_slot
+                    && device_map.is_free(SlotNumber(*pair_start))
+                    && device_map.is_free(SlotNumber(pair_start + 1))
+            })
+            .map(|pair_start| {
+                usize::try_from((pair_start - first_slot) / CLUSTER_SEGMENT_SLOTS).expect("段号")
+            })
+            .collect()
+    }
+
+    /// 有没挡槽对的段号集合（`segments_with_an_unblocked_slot_pair`）是增量维护的，每一步之后拿逐对扫出来的段集合对。
+    /// 先走一段定形取样点：一块盘三个整段全空，第 0 段前 31 对分配掉、最后一对扣住 ⇒ 第 0 段一对不剩，要离开集合；
+    /// 放开扣住 ⇒ 第 0 段的槽对数从 0 变回 1，要回到集合，用户数据落点落回第 0 段最后那一对。
+    /// 再走一串随机操作，每一步之后两块盘都要对得上。随机历史在 385 槽的盘上未必把哪一段占满，所以两个方向的转移靠定形那一段钉住。
+    /// 判别力：一段最后一对被挡上时不把那一段拿掉，挡上之后那一段多出来；槽对放开时不把那一段加回，放开之后那一段少了、落点跳到第 1 段。
     /// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。
     #[test]
     fn segments_with_an_unblocked_slot_pair_are_maintained_incrementally_and_match_a_scan() {
+        let mut device_map = DeviceFreeMap::new(
+            DeviceIdentity(0),
+            (UNIT_AREA_START_SLOT + 3 * CLUSTER_SEGMENT_SLOTS) * SLOT_BYTES,
+        );
+        let last_pair_of_the_first_segment =
+            SlotNumber(UNIT_AREA_START_SLOT + CLUSTER_SEGMENT_SLOTS - 2);
+        device_map.mark_allocated(SlotNumber(UNIT_AREA_START_SLOT), CLUSTER_SEGMENT_SLOTS - 2);
+        device_map.hold_until_floor_takes_effect(last_pair_of_the_first_segment, 2);
+        assert_eq!(
+            segments_with_an_unblocked_slot_pair_by_scan(&device_map),
+            BTreeSet::from([1, 2]),
+            "定形取样点：第 0 段 31 对分配、最后一对扣住，逐对扫只剩第 1、2 段"
+        );
+        assert_eq!(
+            device_map.segments_with_an_unblocked_slot_pair,
+            segments_with_an_unblocked_slot_pair_by_scan(&device_map),
+            "第 0 段最后一对被挡上：那一段离开集合"
+        );
+        device_map.release_holds();
+        assert_eq!(
+            segments_with_an_unblocked_slot_pair_by_scan(&device_map),
+            BTreeSet::from([0, 1, 2]),
+            "定形取样点：放开扣住之后第 0 段又有一对"
+        );
+        assert_eq!(
+            device_map.segments_with_an_unblocked_slot_pair,
+            segments_with_an_unblocked_slot_pair_by_scan(&device_map),
+            "第 0 段的槽对数从 0 变回 1：那一段回到集合"
+        );
+        assert_eq!(
+            device_map.lowest_user_data_slot(&BTreeSet::new()),
+            Some(last_pair_of_the_first_segment),
+            "用户数据落点落回第 0 段放开的那一对"
+        );
+
         let seed = 0x5EED_0000_0000_0385;
         println!("种子 {seed:#x}");
         random_history_for_the_user_data_placement(
@@ -5522,22 +5576,9 @@
             1500,
             &|pool, step_number, what_the_step_did| {
                 for device_map in &pool.devices {
-                    let first_slot = device_map.unit_area_start().slot().0;
-                    let past_the_last_slot = first_slot + device_map.unit_area_slots();
-                    let segments_by_scan: BTreeSet<usize> = (first_slot..past_the_last_slot)
-                        .step_by(2)
-                        .filter(|pair_start| {
-                            pair_start + 1 < past_the_last_slot
-                                && device_map.is_free(SlotNumber(*pair_start))
-                                && device_map.is_free(SlotNumber(pair_start + 1))
-                        })
-                        .map(|pair_start| {
-                            usize::try_from((pair_start - first_slot) / CLUSTER_SEGMENT_SLOTS)
-                                .expect("段号")
-                        })
-                        .collect();
                     assert_eq!(
-                        device_map.segments_with_an_unblocked_slot_pair, segments_by_scan,
+                        device_map.segments_with_an_unblocked_slot_pair,
+                        segments_with_an_unblocked_slot_pair_by_scan(device_map),
                         "种子 {seed:#x} 第 {step_number} 步（{what_the_step_did}）：盘 {:?}",
                         device_map.device
                     );
```

## 三份补丁之后主工作区上这三份文件又有的改动（不在上面的 diff 里）

- 主 agent 2026-11-12 改 `crates/singlefs-core/src/allocator.rs` 的单测 `segments_with_an_unblocked_slot_pair_are_maintained_incrementally_and_match_a_scan` 闭包里的循环变量 `device_map` → `device_map_in_the_pool`（4 行，`clippy::shadow_unrelated`，任务 14857）。归这一轮。
- 另一个会话的补丁（任务 15040）：`allocator.rs` 的 `unblocked_slot_pairs_outside_the_cluster_segments` 返回值换成 `SlotPairCount`、`allocator::tests` 加一条重建后比计数的用例；`admission.rs` 的 `slot_pairs_for_data_units` 换成 `SlotPairCount`。归那个会话自己的代码轮，这一轮的腿只攻它与上面 diff 的交叉。
