# d3-fallback-richest-leaf-code-r1 被判的改动（2026-11-12）

D3（空间分配） 已定项 8 第 2 条的回落政策改成「先取重写集的叶、不够取空槽最多的叶」，产品代码部分（`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/transaction.rs`）；补丁 `~/.local/share/singlefs/work/d3-fallback-richest-leaf-writer/patch/crates.patch` 的前两段，测试两段在那份补丁的第 941 行起。主工作区的 `allocator.rs` 在这份补丁之后又被另一份补丁（整理预检那一题，归它自己的代码轮）改过，这一轮只判下面的块。

```diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 436629517de84e2d8e2b17384fc40eb95d945e0f..e3f35997c9cfaede19ca2fb5725abc6a53a93ec2 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -4,15 +4,17 @@
 //! 不在任何开放的聚簇段里、不在 `R` 保护期内（第一版 R 为空）。
 //! 提交内生块（D3（空间分配） 已定项 5 / 已定项 10）：从开放聚簇段 bump，开放段 = 单元区内最低的 64 槽对齐全空段，bump 只在内存；
 //! 码 3 容器按数据单元那一档取落点（起点 32768 对齐），码 2 节点取最低空槽。
-//! 开放段装不下、又没有全空段时回落（D3（空间分配） 已定项 8 ②）：该设备内槽号最小的空槽，与 bump 游标绕开同一套位（已分配、影子账隔离、抬 F 扣住）。
+//! 开放段装不下、又没有全空段时回落（D3（空间分配） 已定项 8 ②）：先取这次发布分配记录树重写集里的叶（[`RewriteSetLeaves`]，每块盘都重写的那几片）里
+//! 每块盘都空的最低落点，那些叶里一个都没有再取每块盘都空的槽最多的那片叶（一样多取叶序号小的）里的最低落点；两步都排除 `R`，
+//! 与 bump 游标绕开同一套位（已分配、影子账隔离、抬 F 扣住）。每片叶没挡的槽数随三种挡位增量维护，取空槽最多的叶不扫单元区的槽。
 //! 落点按设备取（D3（空间分配） 已定项 8 第 1 条「在每一块被选中的设备上各自取」）：每块盘按自己的空闲图各答一个，各盘一致才分配；
 //! 有的盘答不出、或各盘答得不同，在动任何状态之前拒绝（`PlacementRefusal`）——`Placement` 两盘同槽，各盘不同槽第一版不支持。
 //! 记账（D5（快照 / 空间记账机制） 已定项 7 / 已定项 8）：已分配 = 落点之和、容量 = 单元区、runs = 空闲槽的连续段数（逐设备）、
 //! 逐段的已用槽数、全空聚簇段数（逐设备，`DeviceFreeMap::empty_segments`）——这几样在分配那一刻增量维护，运行时不扫盘（`.claude/rules/fs-design.md` 第一格）。
 
 use singlefs_format::{
-    ALLOCATION_RECORD_BYTES, CLUSTER_SEGMENT_SLOTS, JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS,
-    SLOT_BYTES,
+    ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_TREE_LEAF_SLOTS, CLUSTER_SEGMENT_SLOTS,
+    JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
 };
 
 use std::collections::{BTreeMap, BTreeSet};
@@ -187,10 +189,57 @@ impl UnitFootprint {
 pub enum CommitGeneratedDeviceAnswer {
     /// 这块盘上最低的全空段的起点：在这里开新段（D3（空间分配） 已定项 10 ①）。
     OpenEmptySegment(SlotNumber),
-    /// 这块盘上没有全空段：回落到这块盘上槽号最小的空槽（D3（空间分配） 已定项 8 ②）。
+    /// 这块盘上没有全空段、要回落（D3（空间分配） 已定项 8 ②），带的是这块盘上槽号最小的空槽：各盘只按自己的空闲图答到这一步。
+    /// 每块盘都答回落时，池里按回落政策取一个每块盘都空的落点（[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]），
+    /// 把每块盘的答案都换成它；取不到时各盘这一步的答案原样去判一致。
     FallBackToLowestFreeSlot(SlotNumber),
 }
 
+/// 这次发布分配记录树重写集里的叶（回落先取的去处，D3（空间分配） 已定项 8 第 2 条）：叶序号 = 槽号 ÷ [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]，
+/// 只含池里每块盘上都在重写集里的叶——提交内生块的落点各盘同槽，落在这样的叶里，每块盘上都不多弄脏一片叶。
+/// 空集 = 没有重写集可先取，回落直接取空槽最多的叶。发布路径按这次发布的重写角色算（演练与真发读同一串角色，得同一个集合）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RewriteSetLeaves {
+    leaf_indices: BTreeSet<u64>,
+}
+
+impl RewriteSetLeaves {
+    /// 空集：没有重写集可先取（不在发布路径上取落点的调用方，与重写集里还没有叶的那一轮）。
+    #[must_use]
+    pub const fn none() -> Self {
+        Self {
+            leaf_indices: BTreeSet::new(),
+        }
+    }
+
+    /// 每块盘各自重写集里的叶序号（`leaf_indices_per_device` 一块盘一项）取交集：一片叶只有在池里每块盘上都重写，才算数。
+    #[must_use]
+    pub fn present_on_every_device(leaf_indices_per_device: &[BTreeSet<u64>]) -> Self {
+        let Some((leaf_indices_of_one_device, leaf_indices_of_the_other_devices)) =
+            leaf_indices_per_device.split_first()
+        else {
+            return Self::none();
+        };
+        Self {
+            leaf_indices: leaf_indices_of_one_device
+                .iter()
+                .copied()
+                .filter(|leaf_index| {
+                    leaf_indices_of_the_other_devices
+                        .iter()
+                        .all(|leaf_indices| leaf_indices.contains(leaf_index))
+                })
+                .collect(),
+        }
+    }
+
+    /// 集合里的叶序号，升序。
+    #[must_use]
+    pub fn leaf_indices(&self) -> &BTreeSet<u64> {
+        &self.leaf_indices
+    }
+}
+
 /// 分配器拒绝一次分配的原因。拒绝发生在动任何状态之前：记录、位图、计数、开放段与 bump 游标都照旧。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum PlacementRefusal {
@@ -355,6 +404,10 @@ pub struct DeviceFreeMap {
     unblocked_slot_pairs_per_segment: Vec<u64>,
     /// 上一项全部段之和。
     unblocked_slot_pairs: u64,
+    /// 分配记录树每片叶里三种位都没置的单元区槽数（叶序号 = 槽号 ÷ [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]，下标 = 叶序号 − 单元区起点所在那片叶的序号；
+    /// 单元区首尾那两片叶只数单元区里的那一截）。回落取空槽最多的叶时拿它当每片叶「每块盘都空的槽数」的上界
+    /// （[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]），同上增量维护。
+    unblocked_slots_per_allocation_record_tree_leaf: Vec<u64>,
     /// 回收过槽、还没发过 discard 的段（段号）：回收那一刻记（[`Self::mark_reclaimed`]），发布返回之后整段可再分配时发 discard 并摘掉
     /// 只住内存。
     segments_reclaimed_not_yet_discarded: BTreeSet<usize>,
@@ -463,6 +516,23 @@ impl DeviceFreeMap {
                 (unit_area_slots - first_slot_of_the_segment).min(CLUSTER_SEGMENT_SLOTS) / 2
             })
             .collect();
+        // 起步整个单元区都空：每片叶的数是这片叶与单元区相交那一截的槽数。
+        let unit_area_end_slot = unit_area_start.slot().0 + unit_area_slots;
+        let unblocked_slots_per_allocation_record_tree_leaf: Vec<u64> = if unit_area_slots == 0 {
+            Vec::new()
+        } else {
+            (unit_area_start.slot().0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS
+                ..=(unit_area_end_slot - 1) / ALLOCATION_RECORD_TREE_LEAF_SLOTS)
+                .map(|leaf_index| {
+                    let past_the_leaf = ((leaf_index + 1) * ALLOCATION_RECORD_TREE_LEAF_SLOTS)
+                        .min(unit_area_end_slot);
+                    let first_slot_of_the_leaf_in_the_unit_area = (leaf_index
+                        * ALLOCATION_RECORD_TREE_LEAF_SLOTS)
+                        .max(unit_area_start.slot().0);
+                    past_the_leaf - first_slot_of_the_leaf_in_the_unit_area
+                })
+                .collect()
+        };
         Self {
             device,
             device_size_in_bytes: device_bytes,
@@ -488,6 +558,7 @@ impl DeviceFreeMap {
             unblocked_slots: unit_area_slots,
             unblocked_slot_pairs: unblocked_slot_pairs_per_segment.iter().sum(),
             unblocked_slot_pairs_per_segment,
+            unblocked_slots_per_allocation_record_tree_leaf,
             segments_reclaimed_not_yet_discarded: BTreeSet::new(),
         }
     }
@@ -540,9 +611,16 @@ impl DeviceFreeMap {
                 self.held_until_floor_takes_effect[index] = is_set;
             }
         }
+        let leaf = self.allocation_record_tree_leaf_position_of_index(index);
         match (slot_was_unblocked, self.is_unblocked_at_index(index)) {
-            (true, false) => self.unblocked_slots -= 1,
-            (false, true) => self.unblocked_slots += 1,
+            (true, false) => {
+                self.unblocked_slots -= 1;
+                self.unblocked_slots_per_allocation_record_tree_leaf[leaf] -= 1;
+            }
+            (false, true) => {
+                self.unblocked_slots += 1;
+                self.unblocked_slots_per_allocation_record_tree_leaf[leaf] += 1;
+            }
             (true, true) | (false, false) => {}
         }
         let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
@@ -559,6 +637,65 @@ impl DeviceFreeMap {
         }
     }
 
+    /// 位图下标 → 它所在那片分配记录树叶在 `unblocked_slots_per_allocation_record_tree_leaf` 里的下标。
+    fn allocation_record_tree_leaf_position_of_index(&self, index: usize) -> usize {
+        let slot =
+            self.unit_area_start.slot().0 + u64::try_from(index).expect("位图下标装得进 u64");
+        usize::try_from(
+            slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS
+                - self.unit_area_start.slot().0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS,
+        )
+        .expect("叶下标装得进 usize")
+    }
+
+    /// 分配记录树第 `leaf_index` 片叶（罩槽 `[leaf_index × W, (leaf_index + 1) × W)`，W = [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]）里
+    /// 三种位都没置的单元区槽数；叶整片在单元区外是 0。增量维护，读一次 O(1)。
+    #[must_use]
+    pub fn unblocked_slots_in_allocation_record_tree_leaf(&self, leaf_index: u64) -> u64 {
+        let first_leaf_index = self.unit_area_start.slot().0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS;
+        leaf_index
+            .checked_sub(first_leaf_index)
+            .and_then(|position| usize::try_from(position).ok())
+            .and_then(|position| {
+                self.unblocked_slots_per_allocation_record_tree_leaf
+                    .get(position)
+                    .copied()
+            })
+            .unwrap_or(0)
+    }
+
+    /// 单元区罩到的分配记录树叶序号（首尾含）；单元区是空的交回 `None`。
+    #[must_use]
+    pub fn allocation_record_tree_leaves_of_the_unit_area(&self) -> Option<(u64, u64)> {
+        if self.unit_area_slots == 0 {
+            return None;
+        }
+        Some((
+            self.unit_area_start.slot().0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS,
+            (self.unit_area_end_slot() - 1) / ALLOCATION_RECORD_TREE_LEAF_SLOTS,
+        ))
+    }
+
+    /// 从 `candidate` 起的整个跨度在这块盘的单元区里、每一槽都不被挡（[`Self::is_blocked_for_commit_generated`]）、也不在活整理意图的 `R` 里：
+    /// 回落在池里取落点时逐盘认一个落点的判法（槽号可以在这块盘的单元区外，那时答否）。
+    #[must_use]
+    fn is_whole_span_open_for_commit_generated(
+        &self,
+        candidate: u64,
+        footprint: UnitFootprint,
+        excluded_region: Option<CompactionRegion>,
+    ) -> bool {
+        let is_inside_the_unit_area = candidate >= self.unit_area_start.slot().0
+            && candidate + footprint.slots() <= self.unit_area_end_slot();
+        is_inside_the_unit_area
+            && (candidate..candidate + footprint.slots()).all(|slot_of_the_span| {
+                let is_outside_the_excluded_region = !excluded_region
+                    .is_some_and(|region| region.contains_slot(SlotNumber(slot_of_the_span)));
+                is_outside_the_excluded_region
+                    && !self.is_blocked_for_commit_generated(SlotNumber(slot_of_the_span))
+            })
+    }
+
     /// 三种位都没置的槽数（字段 `unblocked_slots`）：提交内生块落得下的槽，聚簇段内外都算。
     #[must_use]
     pub fn unblocked_slots(&self) -> u64 {
@@ -1021,9 +1158,10 @@ impl DeviceFreeMap {
             })
     }
 
-    /// 提交内生块的回落落点（D3（空间分配） 已定项 8 ②「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」）：
-    /// 这块盘单元区里起点槽号最小、整个跨度都不被挡的落点——挡的是 bump 游标绕开的同一套位（已分配、影子账隔离、抬 F 扣住），
-    /// 回落不是绕开影子账或扣住的第二条路；两槽的码 3 容器照数据单元那一档起点 32768 对齐（D3（空间分配） 已定项 10 ⑤）。第一版 `R` 为空。
+    /// 这块盘按自己的空闲图答的回落落点（[`CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot`] 带的那个槽；池里真取的回落落点见
+    /// [`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]）：这块盘单元区里起点槽号最小、整个跨度都不被挡的落点——
+    /// 挡的是 bump 游标绕开的同一套位（已分配、影子账隔离、抬 F 扣住），回落不是绕开影子账或扣住的第二条路；两槽的码 3 容器照数据单元那一档
+    /// 起点 32768 对齐（D3（空间分配） 已定项 10 ⑤）。第一版 `R` 为空。
     #[must_use]
     pub fn lowest_commit_generated_fallback_slot(
         &self,
@@ -2573,15 +2711,35 @@ impl PoolAllocator {
             .ok()
     }
 
+    /// 提交内生块：见 [`Self::try_allocate_commit_generated_preferring_the_rewrite_set`]，没有重写集可先取（回落直接取空槽最多的叶）。
+    /// 不在发布路径上取落点的调用方走这里（新池新建文件的预演、测试）。
+    ///
+    /// # Errors
+    /// 同 [`Self::try_allocate_commit_generated_preferring_the_rewrite_set`]。
+    pub fn try_allocate_commit_generated(
+        &mut self,
+        footprint: UnitFootprint,
+        generation: CheckpointTxg,
+    ) -> Result<Placement, PlacementRefusal> {
+        self.try_allocate_commit_generated_preferring_the_rewrite_set(
+            footprint,
+            generation,
+            &RewriteSetLeaves::none(),
+        )
+    }
+
     /// 提交内生块：从开放段 bump；容器按 32768 对齐档，节点取游标处最低空槽。开放段没有或装不下时每块盘按自己的空闲图答一个去处——
-    /// 最低的全空段（开新段），没有全空段就回落到槽号最小的空槽（D3（空间分配） 已定项 8 ②）——各盘答得一致才分配。
+    /// 最低的全空段（开新段），没有全空段就回落（D3（空间分配） 已定项 8 ②）。每块盘都答回落时池里取一个每块盘都空的回落落点
+    /// （[`Self::commit_generated_fallback_slot_open_on_every_device`]：先 `rewrite_set_leaves` 里的叶，再空槽最多的叶，都排除 `R`），
+    /// 每块盘的答案都换成它；有的盘答开段、有的盘答回落，或池里取不到每块盘都空的回落落点，照各盘自己的答案判一致——各盘答得一致才分配。
     ///
     /// # Errors
     /// 拒绝时分配器一样都没动（装不下的开放段也照旧开着）：全部盘都没有去处、有的盘没有、各盘的去处不同。
-    pub fn try_allocate_commit_generated(
+    pub fn try_allocate_commit_generated_preferring_the_rewrite_set(
         &mut self,
         footprint: UnitFootprint,
         generation: CheckpointTxg,
+        rewrite_set_leaves: &RewriteSetLeaves,
     ) -> Result<Placement, PlacementRefusal> {
         if let Some(open) = self.open_segment {
             if let Some(slot) = self.bump_slot_in_open_segment(open, footprint) {
@@ -2602,6 +2760,35 @@ impl PoolAllocator {
                 )
             })
             .collect();
+        let every_device_falls_back = answer_per_device.iter().all(|(_, answer)| {
+            matches!(
+                answer,
+                Some(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(_))
+            )
+        });
+        let answer_per_device = match every_device_falls_back
+            .then(|| {
+                self.commit_generated_fallback_slot_open_on_every_device(
+                    footprint,
+                    excluded_region,
+                    rewrite_set_leaves,
+                )
+            })
+            .flatten()
+        {
+            Some(fallback_slot_on_every_device) => answer_per_device
+                .into_iter()
+                .map(|(device, _)| {
+                    (
+                        device,
+                        Some(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(
+                            fallback_slot_on_every_device,
+                        )),
+                    )
+                })
+                .collect(),
+            None => answer_per_device,
+        };
         let answer = match agreement_across_devices(&answer_per_device) {
             DeviceAgreement::Agreed(answer) => answer,
             DeviceAgreement::NoAnswerOnAnyDevice => {
@@ -2645,6 +2832,138 @@ impl PoolAllocator {
         }
     }
 
+    /// 每块盘都答回落之后池里取的回落落点（D3（空间分配） 已定项 8 第 2 条）：先 `rewrite_set_leaves` 里的叶（叶序号升序），取第一片叶里
+    /// 每块盘都空的最低落点；那些叶里一个都没有，再取每块盘都空的单槽最多的那片叶（一样多取叶序号小的）里每块盘都空的最低落点——
+    /// 那片叶里放不下这个跨度（两槽的单元要一对偶数起点的空槽）就让给下一片。「每块盘都空」= 整个跨度在每块盘的单元区里、
+    /// 三种位都没置、不在活 `R` 里（[`DeviceFreeMap::is_whole_span_open_for_commit_generated`]）。一片叶罩的槽数是偶数、起点是偶数，
+    /// 两槽的单元不跨叶。哪一步都取不到就 `None`。只读。
+    fn commit_generated_fallback_slot_open_on_every_device(
+        &self,
+        footprint: UnitFootprint,
+        excluded_region: Option<CompactionRegion>,
+        rewrite_set_leaves: &RewriteSetLeaves,
+    ) -> Option<SlotNumber> {
+        rewrite_set_leaves
+            .leaf_indices
+            .iter()
+            .find_map(|leaf_index| {
+                self.lowest_slot_open_on_every_device_in_the_leaf(
+                    *leaf_index,
+                    footprint,
+                    excluded_region,
+                )
+            })
+            .or_else(|| {
+                self.lowest_slot_in_the_leaf_with_the_most_slots_open_on_every_device(
+                    footprint,
+                    excluded_region,
+                )
+            })
+    }
+
+    /// 回落的后一步：每块盘都空的单槽最多的那片叶（一样多取叶序号小的）里每块盘都空的最低落点，放不下这个跨度的叶让给下一片。
+    /// 每片叶先读每块盘增量维护的没挡槽数（[`DeviceFreeMap::unblocked_slots_in_allocation_record_tree_leaf`]），取各盘里最小的那个当上界：
+    /// 每块盘都空的槽一定在每块盘上都没挡。叶序号升序走一遍，上界大于眼下最多的那片叶的数才逐槽数这片叶（数的是那片叶的槽，不扫单元区），
+    /// 数出来更多、又放得下这个跨度才换成它；上界不大于眼下最多的，这片叶数出来也不会更多，一样多时叶序号小的那片已经在手里。
+    fn lowest_slot_in_the_leaf_with_the_most_slots_open_on_every_device(
+        &self,
+        footprint: UnitFootprint,
+        excluded_region: Option<CompactionRegion>,
+    ) -> Option<SlotNumber> {
+        let (first_leaf_index, last_leaf_index) =
+            self.devices
+                .iter()
+                .fold(None, |leaves_so_far: Option<(u64, u64)>, device| {
+                    match (
+                        leaves_so_far,
+                        device.allocation_record_tree_leaves_of_the_unit_area(),
+                    ) {
+                        (None, leaves_of_the_device) => leaves_of_the_device,
+                        (Some(leaves), None) => Some(leaves),
+                        (Some((first, last)), Some((first_of_the_device, last_of_the_device))) => {
+                            Some((first.min(first_of_the_device), last.max(last_of_the_device)))
+                        }
+                    }
+                })?;
+        let mut most_open_slots_so_far: u64 = 0;
+        let mut slot_in_the_leaf_with_the_most_open_slots: Option<SlotNumber> = None;
+        // 迭代上界是单元区罩到的叶数；跨轮带的是眼下最多的那片叶的数与它里面的落点，没有提前出口。
+        for leaf_index in first_leaf_index..=last_leaf_index {
+            let upper_bound_of_the_open_slots = self
+                .devices
+                .iter()
+                .map(|device| device.unblocked_slots_in_allocation_record_tree_leaf(leaf_index))
+                .min()
+                .unwrap_or(0);
+            if upper_bound_of_the_open_slots <= most_open_slots_so_far {
+                continue;
+            }
+            let open_slots =
+                self.slots_open_on_every_device_in_the_leaf(leaf_index, excluded_region);
+            if open_slots <= most_open_slots_so_far {
+                continue;
+            }
+            if let Some(slot) = self.lowest_slot_open_on_every_device_in_the_leaf(
+                leaf_index,
+                footprint,
+                excluded_region,
+            ) {
+                most_open_slots_so_far = open_slots;
+                slot_in_the_leaf_with_the_most_open_slots = Some(slot);
+            }
+        }
+        slot_in_the_leaf_with_the_most_open_slots
+    }
+
+    /// 分配记录树第 `leaf_index` 片叶里每块盘都空的单槽数（单元区外、活 `R` 里的不算）。
+    fn slots_open_on_every_device_in_the_leaf(
+        &self,
+        leaf_index: u64,
+        excluded_region: Option<CompactionRegion>,
+    ) -> u64 {
+        let first_slot_of_the_leaf = leaf_index * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
+        let open_slots = (first_slot_of_the_leaf
+            ..first_slot_of_the_leaf + ALLOCATION_RECORD_TREE_LEAF_SLOTS)
+            .filter(|slot| {
+                self.devices.iter().all(|device| {
+                    device.is_whole_span_open_for_commit_generated(
+                        *slot,
+                        UnitFootprint::OneSlot,
+                        excluded_region,
+                    )
+                })
+            })
+            .count();
+        u64::try_from(open_slots).expect("一片叶的槽数装得进 u64")
+    }
+
+    /// 分配记录树第 `leaf_index` 片叶里起点最低、整个跨度每块盘都空的落点（起点按跨度的对齐档步进；叶起点是偶数）；没有就 `None`。
+    fn lowest_slot_open_on_every_device_in_the_leaf(
+        &self,
+        leaf_index: u64,
+        footprint: UnitFootprint,
+        excluded_region: Option<CompactionRegion>,
+    ) -> Option<SlotNumber> {
+        let first_slot_of_the_leaf = leaf_index * ALLOCATION_RECORD_TREE_LEAF_SLOTS;
+        let past_the_last_slot_of_the_leaf =
+            first_slot_of_the_leaf + ALLOCATION_RECORD_TREE_LEAF_SLOTS;
+        let mut candidate = first_slot_of_the_leaf;
+        // 迭代上界是一片叶的槽数 ÷ 对齐步长；没有跨轮携带的状态，找到就出。
+        while candidate + footprint.slots() <= past_the_last_slot_of_the_leaf {
+            if self.devices.iter().all(|device| {
+                device.is_whole_span_open_for_commit_generated(
+                    candidate,
+                    footprint,
+                    excluded_region,
+                )
+            }) {
+                return Some(SlotNumber(candidate));
+            }
+            candidate += footprint.alignment_in_slots();
+        }
+        None
+    }
+
     /// 开放段里从游标起第一个整跨度在每块盘上都不被挡的对齐起点；段里装不下就 `None`。只读，不挪游标。
     fn bump_slot_in_open_segment(
         &self,
@@ -4016,6 +4335,300 @@ mod tests {
         }
     }
 
+    /// 开段 [50240, 50304) 的第一槽已发出、段里其余槽占满、别的全空段各占掉段首一槽 ⇒ 一个全空段都没有、开放段装不下，
+    /// 下一个提交内生块回落。段首之外各片分配记录树叶全空：叶 62（[50344, 51156)）罩 13 个段首、空 799 槽，
+    /// 叶 63（[51156, 51968)）罩 12 个段首、空 800 槽，是单元区里空槽最多、叶序号最小的那片。
+    fn pool_after_mkfs_whose_next_commit_generated_block_falls_back() -> PoolAllocator {
+        let mut pool = pool_after_mkfs();
+        let opening = pool
+            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
+            .expect("开段");
+        assert_eq!(opening.slot, SlotNumber(50240));
+        fill_free_slots(&mut pool, 50241, 50304);
+        for device_map in &mut pool.devices {
+            let full_segments = device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS;
+            for segment_index in 0..full_segments {
+                if device_map.segment_is_empty(usize::try_from(segment_index).expect("段号")) {
+                    device_map.mark_allocated(
+                        SlotNumber(UNIT_AREA_START_SLOT + segment_index * CLUSTER_SEGMENT_SLOTS),
+                        1,
+                    );
+                }
+            }
+        }
+        pool
+    }
+
+    /// 两块盘上都在重写集里的叶。
+    fn rewrite_set_leaves_of_both_devices(leaf_indices: &[u64]) -> RewriteSetLeaves {
+        let on_each_device: BTreeSet<u64> = leaf_indices.iter().copied().collect();
+        RewriteSetLeaves::present_on_every_device(&[on_each_device.clone(), on_each_device])
+    }
+
+    /// 每块盘上把单元区里还空着的槽都占掉，只留 `kept_free_per_device` 里点名这块盘的那几个（只动空闲图，不写分配记录：造形态用）。
+    fn fill_every_free_slot_except(pool: &mut PoolAllocator, kept_free_per_device: &[&[u64]]) {
+        for (device_map, kept_free) in pool.devices.iter_mut().zip(kept_free_per_device) {
+            for slot in UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + device_map.unit_area_slots() {
+                if device_map.is_free(SlotNumber(slot)) && !kept_free.contains(&slot) {
+                    device_map.mark_allocated(SlotNumber(slot), 1);
+                }
+            }
+        }
+    }
+
+    /// 两块盘留同一组空槽：叶 70（[56840, 57652)）里 3 个、叶 90（[73080, 73892)）里 5 个、叶 100（[81200, 82012)）里 5 个，别的全占掉。
+    /// 没有全空段、没有开放段 ⇒ 提交内生块回落；槽号最小的空槽在叶 70，空槽最多的叶是 90 与 100（一样多，90 的叶序号小）。
+    fn pool_with_free_slots_left_in_three_leaves() -> PoolAllocator {
+        let kept_free: Vec<u64> = (56_850..56_853)
+            .chain(73_180..73_185)
+            .chain(81_300..81_305)
+            .collect();
+        let mut pool = pool_after_mkfs();
+        fill_every_free_slot_except(&mut pool, &[kept_free.as_slice(), kept_free.as_slice()]);
+        pool
+    }
+
+    /// 回落先取重写集的叶：叶 62 在两块盘上都重写，取它里面最低的空槽 50344，不取空槽最多的叶 63。
+    /// 没有重写集取空槽最多的叶 63 里最低的空槽 51156（槽号最小的空槽 50179 在叶 61，不取）；只在一块盘上重写的叶不算（交集），同样取 51156。
+    #[test]
+    fn fallback_takes_a_free_slot_in_the_rewrite_set_leaf_before_the_leaf_with_the_most_free_slots()
+    {
+        let mut preferring = pool_after_mkfs_whose_next_commit_generated_block_falls_back();
+        let placement = preferring
+            .try_allocate_commit_generated_preferring_the_rewrite_set(
+                UnitFootprint::OneSlot,
+                CheckpointTxg(4),
+                &rewrite_set_leaves_of_both_devices(&[62]),
+            )
+            .expect("回落");
+        assert_eq!(
+            placement.slot,
+            SlotNumber(50344),
+            "重写集的叶 62 里最低的空槽"
+        );
+        assert_eq!(preferring.open_segment(), None, "回落不开段");
+        assert_eq!(
+            preferring.commit_generated_placements_that_fell_back_to_the_lowest_free_slot(),
+            1
+        );
+
+        let mut without_a_rewrite_set =
+            pool_after_mkfs_whose_next_commit_generated_block_falls_back();
+        for device_map in &without_a_rewrite_set.devices {
+            assert_eq!(
+                device_map.lowest_commit_generated_fallback_slot(UnitFootprint::OneSlot),
+                Some(SlotNumber(50179)),
+                "这块盘自己答的回落落点是槽号最小的空槽"
+            );
+        }
+        let placement_without_a_rewrite_set = without_a_rewrite_set
+            .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
+            .expect("回落");
+        assert_eq!(
+            placement_without_a_rewrite_set.slot,
+            SlotNumber(51156),
+            "空槽最多的叶 63 里最低的空槽"
+        );
+
+        let only_on_one_device =
+            RewriteSetLeaves::present_on_every_device(&[BTreeSet::from([62]), BTreeSet::new()]);
+        assert!(
+            only_on_one_device.leaf_indices().is_empty(),
+            "只在一块盘上重写的叶不算"
+        );
+        let mut preferring_a_leaf_of_one_device =
+            pool_after_mkfs_whose_next_commit_generated_block_falls_back();
+        let placement_preferring_a_leaf_of_one_device = preferring_a_leaf_of_one_device
+            .try_allocate_commit_generated_preferring_the_rewrite_set(
+                UnitFootprint::OneSlot,
+                CheckpointTxg(4),
+                &only_on_one_device,
+            )
+            .expect("回落");
+        assert_eq!(
+            placement_preferring_a_leaf_of_one_device.slot,
+            SlotNumber(51156)
+        );
+    }
+
+    /// 重写集的叶里没有空槽（叶 75 全占着）就取空槽最多的叶：叶 90 与叶 100 各 5 个空槽、叶 70 只有 3 个 ⇒ 取叶 90 里最低的 73180，
+    /// 不取槽号最小的 56850（叶 70），一样多时不取叶序号大的叶 100。两槽的单元取叶 90 里最低的一对偶数起点空槽 73180–73181。
+    #[test]
+    fn fallback_without_room_in_the_rewrite_set_leaves_takes_the_leaf_with_the_most_free_slots() {
+        for (footprint, expected) in [
+            (
+                UnitFootprint::OneSlot,
+                Placement {
+                    slot: SlotNumber(73_180),
+                    span: 1,
+                },
+            ),
+            (
+                UnitFootprint::TwoSlotsAligned,
+                Placement {
+                    slot: SlotNumber(73_180),
+                    span: 2,
+                },
+            ),
+        ] {
+            let mut pool = pool_with_free_slots_left_in_three_leaves();
+            for device_map in &pool.devices {
+                assert_eq!(
+                    device_map.lowest_empty_segment(),
+                    None,
+                    "造的形态没有全空段"
+                );
+                assert_eq!(
+                    device_map.unblocked_slots_in_allocation_record_tree_leaf(90),
+                    5
+                );
+                assert_eq!(
+                    device_map.unblocked_slots_in_allocation_record_tree_leaf(70),
+                    3
+                );
+            }
+            let placement = pool
+                .try_allocate_commit_generated_preferring_the_rewrite_set(
+                    footprint,
+                    CheckpointTxg(4),
+                    &rewrite_set_leaves_of_both_devices(&[75]),
+                )
+                .expect("回落");
+            assert_eq!(placement, expected, "{footprint:?}");
+        }
+        let mut preferring_the_poorer_leaf = pool_with_free_slots_left_in_three_leaves();
+        let placement = preferring_the_poorer_leaf
+            .try_allocate_commit_generated_preferring_the_rewrite_set(
+                UnitFootprint::OneSlot,
+                CheckpointTxg(4),
+                &rewrite_set_leaves_of_both_devices(&[70]),
+            )
+            .expect("回落");
+        assert_eq!(
+            placement.slot,
+            SlotNumber(56_850),
+            "重写集的叶里有空槽就先取它，哪怕它空槽少"
+        );
+    }
+
+    /// 活整理意图的 `R`（[73152, 73216)，罩着叶 90 那 5 个空槽）两步都排除：重写集的叶 90 里的空槽全在 R 里 ⇒ 让给空槽最多的叶；
+    /// 叶 90 刨掉 R 之后一个空槽都不剩 ⇒ 空槽最多的是叶 100，取 81300。
+    #[test]
+    fn fallback_excludes_the_live_compaction_region_from_the_rewrite_set_leaves_and_from_the_leaf_with_the_most_free_slots(
+    ) {
+        let mut pool = pool_with_free_slots_left_in_three_leaves();
+        pool.compaction_mut()
+            .issue(crate::logged_operations::CompactionIntent {
+                region: CompactionRegion::of_one_cluster_segment(
+                    DeviceIdentity(0),
+                    SlotNumber(73_152),
+                )
+                .expect("73152 是 64 的倍数"),
+                allocation_generation_at_issue: CheckpointTxg(3),
+                empty_cluster_segments_at_issue: 0,
+            });
+        assert!(pool.region_excluded_from_placement().is_some(), "R 排他");
+        let placement = pool
+            .try_allocate_commit_generated_preferring_the_rewrite_set(
+                UnitFootprint::OneSlot,
+                CheckpointTxg(4),
+                &rewrite_set_leaves_of_both_devices(&[90]),
+            )
+            .expect("回落");
+        assert_eq!(placement.slot, SlotNumber(81_300));
+    }
+
+    /// 两块盘的空闲图不同：盘 0 留叶 70 的 56850–56851、叶 90 的 73180–73184、叶 100 的 81300–81303；盘 1 留 56851、73182–73186、81300–81303。
+    /// 各盘自己答的回落落点不同（56850 对 56851）；空槽最多的叶按两块盘都空的槽数：叶 90 每块盘各 5 个、两块盘都空的只有 73182–73184 三个，
+    /// 叶 100 四个 ⇒ 取叶 100 的 81300，两块盘上各记一条同槽的记录。
+    #[test]
+    fn fallback_on_two_devices_counts_the_slots_free_on_every_device_and_records_one_slot_on_both()
+    {
+        let mut pool = pool_after_mkfs();
+        let kept_free_on_device_zero: Vec<u64> = (56_850..56_852)
+            .chain(73_180..73_185)
+            .chain(81_300..81_304)
+            .collect();
+        let kept_free_on_device_one: Vec<u64> = std::iter::once(56_851)
+            .chain(73_182..73_187)
+            .chain(81_300..81_304)
+            .collect();
+        fill_every_free_slot_except(
+            &mut pool,
+            &[
+                kept_free_on_device_zero.as_slice(),
+                kept_free_on_device_one.as_slice(),
+            ],
+        );
+        assert_eq!(
+            pool.devices
+                .iter()
+                .map(|device_map| device_map
+                    .lowest_commit_generated_fallback_slot(UnitFootprint::OneSlot))
+                .collect::<Vec<_>>(),
+            vec![Some(SlotNumber(56_850)), Some(SlotNumber(56_851))],
+            "各盘自己答的回落落点不同"
+        );
+        let placement = pool
+            .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
+            .expect("两块盘都空的槽还有：回落各盘同槽");
+        assert_eq!(placement.slot, SlotNumber(81_300));
+        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+            let record = pool
+                .record_for(device, SlotNumber(81_300))
+                .unwrap_or_else(|| panic!("盘 {device:?} 上没有 81300 的记录"));
+            assert_eq!(record.generation, CheckpointTxg(4));
+        }
+    }
+
+    /// 每片分配记录树叶的没挡槽数随三种挡位增量维护，与逐槽数出来的相等：分配、隔离、扣住、回收各走一遍，单元区首尾那两片叶只数单元区里的一截。
+    #[test]
+    fn unblocked_slots_per_allocation_record_tree_leaf_are_maintained_incrementally_and_match_a_scan(
+    ) {
+        fn unblocked_slots_in_the_leaf_by_scan(device_map: &DeviceFreeMap, leaf_index: u64) -> u64 {
+            let first = (leaf_index * ALLOCATION_RECORD_TREE_LEAF_SLOTS).max(UNIT_AREA_START_SLOT);
+            let past = ((leaf_index + 1) * ALLOCATION_RECORD_TREE_LEAF_SLOTS)
+                .min(UNIT_AREA_START_SLOT + device_map.unit_area_slots());
+            let unblocked = (first..past)
+                .filter(|slot| !device_map.is_blocked_for_commit_generated(SlotNumber(*slot)))
+                .count();
+            u64::try_from(unblocked).expect("一片叶的槽数")
+        }
+        let mut pool = pool_after_mkfs_whose_next_commit_generated_block_falls_back();
+        pool.isolate_abandoned(DeviceIdentity(1), SlotNumber(51_160), 3);
+        let fell_back = pool
+            .allocate_commit_generated(UnitFootprint::TwoSlotsAligned, CheckpointTxg(4))
+            .expect("回落");
+        pool.devices[0].hold_until_floor_takes_effect(SlotNumber(51_170), 2);
+        pool.release(fell_back, CheckpointTxg(5));
+        assert_eq!(
+            pool.reclaim_released_up_to(CheckpointTxg(5), ReclaimedReuse::Immediately),
+            vec![fell_back]
+        );
+        for device_map in &pool.devices {
+            let (first_leaf, last_leaf) = device_map
+                .allocation_record_tree_leaves_of_the_unit_area()
+                .expect("单元区不空");
+            assert_eq!((first_leaf, last_leaf), (61, 322));
+            for leaf_index in first_leaf..=last_leaf {
+                assert_eq!(
+                    device_map.unblocked_slots_in_allocation_record_tree_leaf(leaf_index),
+                    unblocked_slots_in_the_leaf_by_scan(device_map, leaf_index),
+                    "盘 {:?} 叶 {leaf_index}",
+                    device_map.device
+                );
+            }
+            assert_eq!(
+                device_map.unblocked_slots_in_allocation_record_tree_leaf(first_leaf - 1),
+                0
+            );
+            assert_eq!(
+                device_map.unblocked_slots_in_allocation_record_tree_leaf(last_leaf + 1),
+                0
+            );
+        }
+    }
+
     /// 回落把开放段置空之后，那一段仍是聚簇段、仍不给用户数据
     /// （D3（空间分配） 已定项 8 第 2 条「聚簇段只给提交内生块」；已定项 10 ② 的量词是「任何开放的聚簇段」）。
     /// 形态：开段 [50240, 50304) 发七个提交内生块（新池新建文件的前七个，字节表零 t2..t8）⇒ 游标停在 50249；段里游标前方的槽占满、单元区里一个全空段都不剩
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 11de38bd1706145ad96162563928baf0ffaad7c0..732bccd5835c832319908e95022c40cbbed7ef54 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -50,7 +50,7 @@ use crate::allocation_record_tree::{
     AllocationRecordTreeNodePosition, AllocationRecordTreePlan, AllocationRecordTreeVersion,
 };
 use crate::allocator::{
-    AllocationRecord, Placement, PlacementRefusal, PoolAllocator, UnitFootprint,
+    AllocationRecord, Placement, PlacementRefusal, PoolAllocator, RewriteSetLeaves, UnitFootprint,
 };
 use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
 use crate::checksum::crc32_castagnoli;
@@ -2051,8 +2051,9 @@ fn settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(
             nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));
             continue;
         }
+        let rewrite_set_leaves = rewrite_set_leaves_of_the_roles(&rewritten_roles, allocator);
         for identity in &rewritten_roles {
-            allocate_placement_for_role(&mut rehearsal, *identity, txg)?;
+            allocate_placement_for_role(&mut rehearsal, *identity, txg, &rewrite_set_leaves)?;
         }
         let changed =
             nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
@@ -2206,8 +2207,10 @@ pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
     // 实例表链的各片最前、尾片先（D3（空间分配） 已定项 10 ⑤），分配记录树的节点在它们之后按位置先叶后根取：
     // 这些节点自己的分配记录也要进这棵树，所以落点都取完才装节点。
     let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
+    let rewrite_set_leaves = rewrite_set_leaves_of_the_roles(&settled.rewritten_roles, allocator);
     for identity in &settled.rewritten_roles {
-        let placement = allocate_placement_for_role(allocator, *identity, txg)?;
+        let placement =
+            allocate_placement_for_role(allocator, *identity, txg, &rewrite_set_leaves)?;
         slots.insert(*identity, placement.slot);
     }
     // 落点取完、这次不再分配：记下这条根盖掉的根环槽（同 `publish_admitted` 那一处；失败时分配器由调用方整个换回去），
@@ -2482,7 +2485,8 @@ pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
     ))
 }
 
-/// 一个角色这次发布的落点：用户数据按政策函数，其余是提交内生块（码 3 容器按数据单元那一档，D3（空间分配） 已定项 10 ⑤）。
+/// 一个角色这次发布的落点：用户数据按政策函数，其余是提交内生块（码 3 容器按数据单元那一档，D3（空间分配） 已定项 10 ⑤）；
+/// 提交内生块回落时先取 `rewrite_set_leaves`（这次发布分配记录树重写集里的叶）里的空槽，再取空槽最多的叶（D3（空间分配） 已定项 8 第 2 条）。
 /// 取不到时带上是哪个角色、分配器给的原因。
 ///
 /// # Errors
@@ -2491,12 +2495,16 @@ fn allocate_placement_for_role(
     allocator: &mut PoolAllocator,
     identity: TransactionUnit,
     txg: CheckpointTxg,
+    rewrite_set_leaves: &RewriteSetLeaves,
 ) -> Result<Placement, PublishError> {
     let placement = match identity.placement() {
         PlacementRule::UserData => allocator.try_allocate_user_data(txg),
-        PlacementRule::CommitGenerated(footprint) => {
-            allocator.try_allocate_commit_generated(footprint, txg)
-        }
+        PlacementRule::CommitGenerated(footprint) => allocator
+            .try_allocate_commit_generated_preferring_the_rewrite_set(
+                footprint,
+                txg,
+                rewrite_set_leaves,
+            ),
     };
     placement.map_err(|refusal| PublishError::PlacementRefused {
         unit: identity,
@@ -2504,6 +2512,36 @@ fn allocate_placement_for_role(
     })
 }
 
+/// 这次发布重写的角色里分配记录树的叶（层级 0），池里每块盘上都重写的那几片的序号（[`RewriteSetLeaves::present_on_every_device`]）：
+/// 提交内生块回落先取这些叶里的空槽（[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]）。
+/// 固定点的演练与真发读的是同一串重写角色（定下来那一轮的角色就是真发的角色），所以两边得同一个集合、取同一串落点。
+fn rewrite_set_leaves_of_the_roles(
+    rewritten_roles: &[TransactionUnit],
+    allocator: &PoolAllocator,
+) -> RewriteSetLeaves {
+    let leaf_indices_per_device: Vec<BTreeSet<u64>> = allocator
+        .devices
+        .iter()
+        .map(|device_map| {
+            rewritten_roles
+                .iter()
+                .filter_map(|role| match allocation_record_tree_node_of_role(*role) {
+                    Some(AllocationRecordTreeNode::BelowTheRoot(position))
+                        if position.level == 0 && position.device == device_map.device =>
+                    {
+                        Some(position.index_in_device)
+                    }
+                    Some(
+                        AllocationRecordTreeNode::BelowTheRoot(_) | AllocationRecordTreeNode::Root,
+                    )
+                    | None => None,
+                })
+                .collect()
+        })
+        .collect();
+    RewriteSetLeaves::present_on_every_device(&leaf_indices_per_device)
+}
+
 /// 池里现行的那一版：还没发布过文件版本（树表 0 条，零单元发布或写行那次发布写出的根）、带文件的一版，
 /// 或有树而文件集合为空的一版（D8（核心索引结构） 已定项 16 第 5 条，判定见 [`TransactionOutput::is_a_version_with_trees_and_no_file`]）。
 /// 有树的两种由 [`PoolVersion::of_a_version_of_trees`] 按同一个判定分，不由调用方挑。
@@ -13749,10 +13787,11 @@ struct RelocationOfThePublish {
 /// （D26（后台整理与放置回收） 已定项 5 第 2 条、D3（空间分配） 已定项 7「分配代」口径），活 R 由分配器排除。
 ///
 /// # Errors
-/// 取不到 ⇒ `PlacementRefused`，带着释放判定用的那个角色。
+/// 取不到 ⇒ `PlacementRefused`，带着释放判定用的那个角色。提交内生块的搬迁回落时同 [`allocate_placement_for_role`]，先取 `rewrite_set_leaves` 里的空槽。
 fn allocate_relocation_destination(
     allocator: &mut PoolAllocator,
     relocation: &RelocationOfThePublish,
+    rewrite_set_leaves: &RewriteSetLeaves,
 ) -> Result<Placement, PublishError> {
     let rule = relocation.role_for_the_checks.placement();
     let placement = match (allocator.compaction().switches.destination_policy, rule) {
@@ -13763,7 +13802,11 @@ fn allocate_relocation_destination(
         (
             RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
             PlacementRule::CommitGenerated(footprint),
-        ) => allocator.try_allocate_commit_generated(footprint, relocation.birth_txg),
+        ) => allocator.try_allocate_commit_generated_preferring_the_rewrite_set(
+            footprint,
+            relocation.birth_txg,
+            rewrite_set_leaves,
+        ),
         (
             RelocationDestinationPolicy::LowestUnblockedSlotForEveryUnitForTests,
             PlacementRule::UserData,
@@ -13925,10 +13968,14 @@ fn role_standing_for_the_unit_of_a_deadlist_entry(
 /// 真发时走的是同一串释放与取落点，得出同一份记录（释放之前读盘核出对不上的那几份记录留在已分配，只会让变了的节点更少）。
 /// 没有上一版（第一个文件版本）时这棵树的树号换成这次发出来的（树表 0 条那一版的分配记录树是树 0），节点全部重写、不照抄。
 ///
-/// **走得完**：产品路径上释放只改写记录、不在这次发布里空出槽（回收要等根记下之后），取落点时分配记录树的节点之后只剩单槽的提交内生块、
-/// 按同一串槽位往后取——多一个角色只把这一串拉长，有记录的节点只多不少，重写集也只增不减，都以池里的节点数为上界。
-/// 节点数可以比 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 大，这个上界不保证轮数够：释放链沿活跃叶连成一长串时，一轮只走一跳就要走满轮数，
-/// 所以每次并进重写集都一次走到闭包，释放链不占轮数，轮数只剩取落点碰到新叶那一半。这一半的上界没有推出来，E206（发布固定点轮数的代价）第一段 16 个取样点量到至多 5 轮，这几个点都有全空段、走不到回落；回落到最低空槽那一格没有取样点。
+/// **走不走得完，推出来的只有一半**：重写集只增不减（只经 [`merge_into_the_rewrite_set_walking_the_release_chain`] 并节点），以池里的节点数为上界；
+/// 一轮没定下来，要么并进了新节点（重写集严格变大），要么重写集没变、只是这一版的节点换成走出来的那一批。后一种轮的个数推不出上界：
+/// 提交内生块回落时先取这一轮重写集里的叶（[`rewrite_set_leaves_of_the_roles`]），取光了再取空槽最多的叶（D3（空间分配） 已定项 8 第 2 条），
+/// 计划里的节点一变、先取的叶跟着变，这一轮的落点不是上一轮那一串的延长——装着记录的节点可以变少，这一版的节点不单调，同一个重写集下可以来回换。
+/// 推的、没证：回落的落点只进两种叶，重写集里的叶与空槽最多的那片叶；后者装进这次的落点就变了内容、下一轮并进重写集、之后先取它，
+/// 所以每进一片新叶都是重写集里的叶取光之后的那一次、重写集跟着变大。这一句只说回落那一支；开段与 bump 的落点按段走，与重写集无关。
+/// 节点数可以比 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 大，节点数这个上界也不保证轮数够：释放链沿活跃叶连成一长串时，一轮只走一跳就要走满轮数，
+/// 所以每次并进重写集都一次走到闭包，释放链不占轮数，轮数只剩取落点碰到新叶与这一版的节点来回换那两样。E206（发布固定点轮数的代价）第一段 16 个取样点量到至多 5 轮，这几个点都有全空段、走不到回落。
 /// 只供测试的复用窗口置 0 开关（`ReuseWindow::ForcedToZero`）让释放当场空出槽、回落可能取回它们，那一格的单调性也推不出来。
 /// 走满 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 轮走不完，交回 `AllocationRecordTreeRewriteSetDidNotSettle`（条款没写这一格怎么办）。
 ///
@@ -14041,11 +14088,13 @@ fn settle_the_allocation_record_tree(
             nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));
             continue;
         }
+        let rewrite_set_leaves =
+            rewrite_set_leaves_of_the_roles(&resolved.rewritten_roles, allocator);
         for relocation in &relocations_in_effect {
-            allocate_relocation_destination(&mut rehearsal, relocation)?;
+            allocate_relocation_destination(&mut rehearsal, relocation, &rewrite_set_leaves)?;
         }
         for identity in &resolved.rewritten_roles {
-            allocate_placement_for_role(&mut rehearsal, *identity, plan.txg)?;
+            allocate_placement_for_role(&mut rehearsal, *identity, plan.txg, &rewrite_set_leaves)?;
         }
         let changed =
             nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
@@ -14106,8 +14155,9 @@ fn merge_into_the_rewrite_set_walking_the_release_chain(
     }
 }
 
-/// 分配记录树重写集的固定点最多走几轮（[`settle_the_allocation_record_tree`]）。释放链每次并进重写集都走到闭包，产品路径上轮数只剩取落点碰到新叶那一半，
-/// E206（发布固定点轮数的代价）全部取样点量到至多 5 轮；这一半与只供测试的复用窗口置 0 开关下的单调性都推不出上界，这个数给它们一个边。
+/// 分配记录树重写集的固定点最多走几轮（[`settle_the_allocation_record_tree`]）。释放链每次并进重写集都走到闭包，产品路径上轮数只剩取落点碰到新叶
+/// 与回落先取重写集的叶让这一版的节点来回换那两样，E206（发布固定点轮数的代价）全部取样点量到至多 5 轮；这两样与只供测试的复用窗口置 0 开关下的单调性
+/// 都推不出上界，这个数给它们一个边。
 pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;
 
 /// 一次发布切成几条 journal 记录、每条点名哪几个角色（D23（journal 的角色与格式） 已定项 17，C491（多条记录时共享内生块在哪条点名没定）
@@ -15701,10 +15751,12 @@ fn publish_admitted<Device: BlockDevice>(
     // 整理的一批：源落点上面已经随释放进了 defer；目的落点在这次重写的角色之前取（固定点里走的是同一串，`settle_the_allocation_record_tree`），
     // 分配代写单元的诞生代。目的地定下之后，上一版拷一份、把搬走的单元的映射位置换到目的地（`take_the_mapping_positions_over_the_location_hints`）：
     // 照抄这些单元的角色、装映射条目、重写它们的父节点与树表时，读的都是这一份。
+    let rewrite_set_leaves = rewrite_set_leaves_of_the_roles(rewritten, allocator);
     let mut relocated_copies: Vec<(PublishedUnit, NamedUnit, UnitRelocatedByThisPublish)> =
         Vec::with_capacity(relocations.len());
     for relocation in relocations {
-        let destination = allocate_relocation_destination(allocator, relocation)?;
+        let destination =
+            allocate_relocation_destination(allocator, relocation, &rewrite_set_leaves)?;
         let destination_locations = pool.location_entries(destination.slot, &relocation.bytes);
         let mut key_tail = [0u8; 10];
         key_tail.copy_from_slice(&relocation.mapping_key[17..27]);
@@ -15786,7 +15838,8 @@ fn publish_admitted<Device: BlockDevice>(
     // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。
     let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
     for identity in rewritten {
-        let placement = allocate_placement_for_role(allocator, *identity, txg)?;
+        let placement =
+            allocate_placement_for_role(allocator, *identity, txg, &rewrite_set_leaves)?;
         slots.insert(*identity, placement.slot);
     }
     // 这次的根要盖掉根环里的一个槽：落点都取完了、这次不再分配，在装记账行之前记下它——盖掉的是被抛弃的根就清它的隔离位，
```
