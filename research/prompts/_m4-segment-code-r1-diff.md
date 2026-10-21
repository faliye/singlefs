# m4-segment-code-r1：被判产品代码相对 master（c1e9b324）的改动全文（2026-10-16）

已跟踪文件 `git diff master`，新文件 `git diff --no-index /dev/null`；测试代码（`tests/`、harness、checker、checker 档）不在这里，见冻结副本。

````diff
diff --git a/crates/singlefs-core/src/admission.rs b/crates/singlefs-core/src/admission.rs
index f7cb9b32..0af2f06a 100644
--- a/crates/singlefs-core/src/admission.rs
+++ b/crates/singlefs-core/src/admission.rs
@@ -819,6 +819,62 @@ pub fn admit_on_every_device(
     }
 }
 
+/// 合取 7（D3（空间分配） 已定项 7 登记表第 7 行）管不管这一次发布：只拦前台写入类发布，删除类、整理批、挂载与回退的发布免
+/// （用户 2026-10-13 定「只拦前台写入类发布」）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum EmptyClusterSegmentsConjunctReach {
+    /// 前台写入类：写文件内容（第一个文件、覆盖写、顺序写）、建 inode、建快照——条款列出的四类免除之外、带普通分配的发布。
+    ForegroundWrite,
+    /// 免这一条：删除类（unlink 与删快照的意图与各批）、整理批、挂载（写行、暖机）与回退、抬 F 的空发布。
+    Exempt,
+}
+
+/// 合取 7 被拒时全空聚簇段少于 `N₀` 的一块盘：它此刻的全空聚簇段数（分配器增量维护的那一个，`DeviceFreeMap::empty_segments`）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct DeviceWithEmptyClusterSegmentsBelowTheFloor {
+    pub device: DeviceIdentity,
+    pub empty_cluster_segments: u64,
+}
+
+/// 合取 7 被拒（`transaction::PublishError::EmptyClusterSegmentsBelowTheFloor`）：`N₀` 与全空聚簇段少于它的每一块盘（按交进来的次序）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
+    pub floor_in_segments: u64,
+    pub short_devices: Vec<DeviceWithEmptyClusterSegmentsBelowTheFloor>,
+}
+
+/// 合取 7 `全空聚簇段数 ≥ N₀`（D26（后台整理与放置回收） 已定项 3 的第二合取式；D3（空间分配） 已定项 7 登记表第 7 行）逐设备判：
+/// `empty_cluster_segments_on_each_device` 是每块盘此刻的全空聚簇段数（分配器增量维护的那一个，`DeviceFreeMap::empty_segments`）。
+/// 一块盘少于 `floor_in_segments` 就拒。这一条约束的是形状、不是标量（D26 已定项 3），拒绝按段数报、不换成字节，
+/// 与空间准入不够分开（调用方要做的决定不同：腾整段靠整理，不是删东西）。
+///
+/// # Errors
+/// [`EmptyClusterSegmentsBelowTheFloorOnSomeDevices`]：至少一块盘全空段少于 `N₀`。
+pub fn admit_the_empty_cluster_segments_floor_on_every_device(
+    empty_cluster_segments_on_each_device: &[(DeviceIdentity, u64)],
+    floor_in_segments: u64,
+) -> Result<(), EmptyClusterSegmentsBelowTheFloorOnSomeDevices> {
+    let short_devices: Vec<DeviceWithEmptyClusterSegmentsBelowTheFloor> =
+        empty_cluster_segments_on_each_device
+            .iter()
+            .filter(|(_, empty_segments)| *empty_segments < floor_in_segments)
+            .map(
+                |(device, empty_segments)| DeviceWithEmptyClusterSegmentsBelowTheFloor {
+                    device: *device,
+                    empty_cluster_segments: *empty_segments,
+                },
+            )
+            .collect();
+    if short_devices.is_empty() {
+        Ok(())
+    } else {
+        Err(EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
+            floor_in_segments,
+            short_devices,
+        })
+    }
+}
+
 /// 「这次的单元落得下」那一判的读数，一块盘一条（C545（空间准入罩不住分裂与聚簇段层），用户 2026-09-27 定准入先拒）：
 /// 这块盘此刻按分配器今天的落点规则（D3（空间分配） 已定项 8、已定项 10）还落得下多少。两个数都是分配器随三种挡位增量维护的
 /// （`DeviceFreeMap::set_blocking_bit`），读它们 O(这次挂载开过的聚簇段数)，不扫单元区（`.claude/rules/fs-design.md`「记账是事务的副产品」）。
@@ -1106,8 +1162,12 @@ pub fn checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
                             MultiLevelCodeTwoTree::CentralMapping,
                         ),
                     ),
-                    u64::try_from(file_version.mapped_units.len())
-                        .expect("进映射的单元数装得进 u64"),
+                    // 映射条目数 = 进映射的单元 + 被活快照钉住、留存的映射条目（判决 S3 的 α）：空发布两样都照抄。
+                    u64::try_from(
+                        file_version.mapped_units.len()
+                            + file_version.mapping_entries_kept_for_the_snapshots.len(),
+                    )
+                    .expect("映射条目数装得进 u64"),
                     CentralMappingEntryChangesOfAnEmptyPublish {
                         deleted_at_most: mapped_changes_with_accounting_nodes(
                             accounting_tree_nodes_of_this_version,
@@ -1427,6 +1487,7 @@ pub const fn space_budget_of_role(role: TransactionUnit) -> SpaceBudgetOfARole {
         // 删除那两棵树的节点只在删除类发布里重写：落点那一判照判（普通分配），式子那一判删除类发布整次不判
         // （D28（挂载期承诺量） 已定项 1：它们的空间罩在环窗口与墓碑预付里，`transaction::prepare_the_version_publish`）。
         TransactionUnit::DeletedInodesRoot
+        | TransactionUnit::LoggedOperationsRoot
         | TransactionUnit::TombstoneLeafContainer(_)
         | TransactionUnit::TombstoneRoot
         | TransactionUnit::TombstoneNodeBelowTheRoot(_) => SpaceBudgetOfARole::OrdinaryAllocation,
@@ -1435,6 +1496,12 @@ pub const fn space_budget_of_role(role: TransactionUnit) -> SpaceBudgetOfARole {
         TransactionUnit::RetirementLeafContainer(_)
         | TransactionUnit::RetirementRoot
         | TransactionUnit::RetirementNodeBelowTheRoot(_) => SpaceBudgetOfARole::OrdinaryAllocation,
+        // deadlist 树的节点只在杀了被活快照引用的块的那次（覆盖写、删除批）与删快照的批里重写（判决
+        // `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3、S4）：条款没写它归哪一格，照删除那两棵树读——落点那一判照判，
+        // 式子那一判跟着它所在的那次发布（删快照的三种与 unlink 同准入、整次不判）。不进 ckpt_cost 的 Σ 名单：空发布不重写它。
+        TransactionUnit::DeadlistTreeNodeBelowTheRoot(_) | TransactionUnit::DeadlistTree => {
+            SpaceBudgetOfARole::OrdinaryAllocation
+        }
         // 谱系容器与谱系树根归固定点、进 ckpt_cost 的 Σ 名单（D28（挂载期承诺量） 已定项 1 接线），只在管理员回退那次重写：
         // 回退不判式子，空间从 checkpoint 保留池里出（[`checkpoint_cost_of_one_lineage_container_cow`]）。
         TransactionUnit::LineageLeafContainer(_)
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 1329943e..dc4e9cdc 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -19,8 +19,10 @@ use std::collections::{BTreeMap, BTreeSet};
 use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
 use crate::admission::SpaceAdmission;
 use crate::bytes::ByteWriter;
+use crate::compaction_plan::CompactionSwitchesAndTheExcludedRegion;
 use crate::instance_table::InstanceTableRecords;
 use crate::journal::slot_after_the_journal_ring;
+use crate::logged_operations::CompactionRegion;
 use crate::root_record::RootRecord;
 use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
 use crate::transaction::FrozenPublish;
@@ -252,8 +254,15 @@ pub struct DeviceFreeMap {
     unit_area_slots: u64,
     /// 每个 16 KiB 槽一位：true = 已分配。
     allocated: Vec<bool>,
-    /// 每个 64 槽聚簇段里已分配的槽数（增量维护全空段数用）。
+    /// 每个 64 槽聚簇段里分配位置着的槽数（仍分配的与已释放、还在 defer 窗口里的都算）：开段、discard 判整段可再分配用。
     used_per_segment: Vec<u64>,
+    /// 每个 64 槽聚簇段里罩在未释放分配记录下的槽数：仍分配的算，已释放、还在 defer 窗口里的不算（D3（空间分配） 已定项 10 ① 的
+    /// 记录口径，C372（全空聚簇段数两份定义相反））。与 `used_per_segment` 只差 defer 里的那几槽。分配与复活时加、释放时减，
+    /// 回收不动（回收的本来就是已释放的）。
+    unreleased_per_segment: Vec<u64>,
+    /// 单元区里凑满 64 槽的整段中 `unreleased_per_segment` 为 0 的段数：记账第 11 项「全空聚簇段数」（[`Self::empty_segments`]），
+    /// 随 `unreleased_per_segment` 每变一次增量维护，不扫段（`.claude/rules/fs-design.md`「记账是事务的副产品」：合取 7 在准入路径上读它）。
+    full_segments_without_an_unreleased_record: u64,
     /// 空闲槽的连续段数（增量维护）。
     free_runs: u64,
     /// 占着的槽数：仍分配的加上已释放、还在 defer 窗口里的——它们仍被根环里的有效根引用、仍占着空间
@@ -287,6 +296,18 @@ pub struct DeviceFreeMap {
     unblocked_slot_pairs_per_segment: Vec<u64>,
     /// 上一项全部段之和。
     unblocked_slot_pairs: u64,
+    /// 回收过槽、还没发过 discard 的段（段号）：回收那一刻记（[`Self::mark_reclaimed`]），发布返回之后整段可再分配时发 discard 并摘掉
+    /// （判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G5；[`Self::take_the_segments_to_discard`]）。只住内存。
+    segments_reclaimed_not_yet_discarded: BTreeSet<usize>,
+}
+
+/// 一段槽进出未释放分配记录的方向（[`DeviceFreeMap::empty_segments`] 的记录口径按它增量维护）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum UnreleasedRecordTransition {
+    /// 分配，或管理员回退把 defer 里的已释放记录复活成已分配。
+    SlotsComeUnderAnUnreleasedRecord,
+    /// 释放：记录改写成已释放、槽进 defer 队列。
+    SlotsLeaveTheirUnreleasedRecord,
 }
 
 /// 挡住一个槽的三种位（[`DeviceFreeMap`] 的三张位图）：一个都没置的槽才发得出去。
@@ -390,6 +411,8 @@ impl DeviceFreeMap {
             unit_area_slots,
             allocated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
             used_per_segment: vec![0; usize::try_from(segments).expect("段数")],
+            unreleased_per_segment: vec![0; usize::try_from(segments).expect("段数")],
+            full_segments_without_an_unreleased_record: unit_area_slots / CLUSTER_SEGMENT_SLOTS,
             free_runs: u64::from(unit_area_slots > 0),
             allocated_slots: 0,
             free_slots: unit_area_slots,
@@ -406,6 +429,7 @@ impl DeviceFreeMap {
             unblocked_slots: unit_area_slots,
             unblocked_slot_pairs: unblocked_slot_pairs_per_segment.iter().sum(),
             unblocked_slot_pairs_per_segment,
+            segments_reclaimed_not_yet_discarded: BTreeSet::new(),
         }
     }
 
@@ -551,7 +575,39 @@ impl DeviceFreeMap {
         self.deferred_slots
     }
 
-    /// 把一个仍分配的落点放进 defer 队列：槽仍占着（分配器不许再发它），只是记账上从「仍分配」挪到「待释放」。
+    /// 位图下标 `[start, end)` 的槽进出未释放分配记录：逐槽改 `unreleased_per_segment`，一段的数在 0 与非 0 之间翻过去、
+    /// 那一段又是凑满 64 槽的整段时，记录口径的全空整段数跟着减一或加一（[`Self::empty_segments`]）。
+    fn note_slots_under_an_unreleased_record(
+        &mut self,
+        start: usize,
+        end: usize,
+        transition: UnreleasedRecordTransition,
+    ) {
+        let slots_per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
+        for index in start..end {
+            let segment = index / slots_per_segment;
+            let is_a_full_segment = (segment + 1) * slots_per_segment <= self.allocated.len();
+            match transition {
+                UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord => {
+                    if self.unreleased_per_segment[segment] == 0 && is_a_full_segment {
+                        self.full_segments_without_an_unreleased_record -= 1;
+                    }
+                    self.unreleased_per_segment[segment] += 1;
+                }
+                UnreleasedRecordTransition::SlotsLeaveTheirUnreleasedRecord => {
+                    self.unreleased_per_segment[segment] = self.unreleased_per_segment[segment]
+                        .checked_sub(1)
+                        .expect("释放的槽此前罩在未释放的记录下：mark_released 先断言过这些槽分配位置着，分配与复活都给它们加过");
+                    if self.unreleased_per_segment[segment] == 0 && is_a_full_segment {
+                        self.full_segments_without_an_unreleased_record += 1;
+                    }
+                }
+            }
+        }
+    }
+
+    /// 把一个仍分配的落点放进 defer 队列：槽仍占着（分配器不许再发它），只是记账上从「仍分配」挪到「待释放」；
+    /// 记录改写成已释放，按记录口径它的槽从此不罩在未释放的记录下（[`Self::empty_segments`]）。
     pub fn mark_released(&mut self, slot: SlotNumber, span: u64) {
         let start = self.index(slot);
         let end = start + usize::try_from(span).expect("跨度");
@@ -559,6 +615,11 @@ impl DeviceFreeMap {
             self.allocated[start..end].iter().all(|taken| *taken),
             "释放的跨度里有没分配的槽"
         );
+        self.note_slots_under_an_unreleased_record(
+            start,
+            end,
+            UnreleasedRecordTransition::SlotsLeaveTheirUnreleasedRecord,
+        );
         self.deferred_slots += span;
     }
     /// 管理员回退复活一个还在 defer 队列里的落点（`PoolAllocator::resurrect_released_record`）：槽本来就占着，
@@ -574,6 +635,11 @@ impl DeviceFreeMap {
             .deferred_slots
             .checked_sub(span)
             .expect("复活的落点在 defer 里，defer 槽数不小于它的跨度");
+        self.note_slots_under_an_unreleased_record(
+            start,
+            end,
+            UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord,
+        );
     }
 
     /// 隔离一个只被被抛弃根引用的落点：不进已分配、不进 defer、不动空闲计数，只让分配器绕开它（用户数据落点与开放段都不落在它上面）。
@@ -722,25 +788,67 @@ impl DeviceFreeMap {
             self.set_blocking_bit(index, SlotBlockingBit::Allocated, false);
             let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
             self.used_per_segment[segment] -= 1;
+            self.segments_reclaimed_not_yet_discarded.insert(segment);
         }
         self.allocated_slots -= span;
         self.deferred_slots -= span;
         self.free_slots += span;
     }
 
+    /// 段号 → 段起点的槽号。
+    fn segment_start_slot(&self, segment_index: usize) -> SlotNumber {
+        SlotNumber(
+            self.unit_area_start.slot().0
+                + u64::try_from(segment_index).expect("段号") * CLUSTER_SEGMENT_SLOTS,
+        )
+    }
+
+    /// 这一段是不是整段都可再分配（判决 G5「可再分配」谓词逐槽为真）：没有已分配（仍分配与 defer 窗口里的都算占着）、没有影子账隔离、
+    /// 没有抬 F 扣住（扣住 = F 还没在每块盘上生效）。单元区末尾凑不满 64 槽的那一截不是整段。
+    #[must_use]
+    pub fn segment_is_reusable_as_a_whole(&self, segment_index: usize) -> bool {
+        let per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
+        let is_a_full_segment = (segment_index + 1) * per_segment <= self.allocated.len();
+        is_a_full_segment
+            && self.used_per_segment[segment_index] == 0
+            && self.isolated_per_segment[segment_index] == 0
+            && self.held_per_segment[segment_index] == 0
+    }
+
+    /// 发布返回之后要发 discard 的段起点（判决 G5：可再分配谓词为真的槽按 64 槽对齐的整段发）：回收过槽、还没发过的段里，
+    /// 整段可再分配的交出去并摘掉；段里又有槽被分配了的也摘掉（之后再回收会重新记上）；只差扣住或隔离的留着，等它们放开。
+    /// 只看回收时记下的段，不扫单元区（`.claude/rules/fs-design.md`「记账是事务的副产品」：discard 不在决策路径上，也不为它遍历）。
+    pub fn take_the_segments_to_discard(&mut self) -> Vec<SlotNumber> {
+        let noted: Vec<usize> = self
+            .segments_reclaimed_not_yet_discarded
+            .iter()
+            .copied()
+            .collect();
+        let mut segments_to_discard = Vec::new();
+        for segment_index in noted {
+            if self.segment_is_reusable_as_a_whole(segment_index) {
+                segments_to_discard.push(self.segment_start_slot(segment_index));
+                self.segments_reclaimed_not_yet_discarded
+                    .remove(&segment_index);
+            } else if self.used_per_segment[segment_index] > 0 {
+                self.segments_reclaimed_not_yet_discarded
+                    .remove(&segment_index);
+            }
+        }
+        segments_to_discard
+    }
+
     #[must_use]
     pub fn free_runs(&self) -> u64 {
         self.free_runs
     }
+    /// 全空聚簇段数（记账第 11 项）：单元区里凑满 64 槽的整段中、段内 64 槽都没有未释放分配记录的段数（D3（空间分配） 已定项 10 ①、
+    /// D26（后台整理与放置回收） 已定项 1 第 1 条收口的记录口径，C372（全空聚簇段数两份定义相反））。defer 窗口里的已释放槽不算占用：
+    /// 一段里只剩 defer 槽时这里算它全空，[`Self::segment_is_empty`] 与开段（[`Self::lowest_empty_segment`]）照旧算它占着。
+    /// 隔离位与抬 F 扣住位不看（「可开聚簇段」是另一个量，C379）。增量维护，读一次 O(1)。
     #[must_use]
     pub fn empty_segments(&self) -> u64 {
-        u64::try_from(
-            self.used_per_segment
-                .iter()
-                .filter(|used| **used == 0)
-                .count(),
-        )
-        .expect("段数")
+        self.full_segments_without_an_unreleased_record
     }
 
     /// 段内是不是整段空闲。
@@ -775,6 +883,11 @@ impl DeviceFreeMap {
             let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
             self.used_per_segment[segment] += 1;
         }
+        self.note_slots_under_an_unreleased_record(
+            start,
+            end,
+            UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord,
+        );
         self.allocated_slots += span;
         self.free_slots -= span;
     }
@@ -785,6 +898,17 @@ impl DeviceFreeMap {
     pub fn lowest_user_data_slot(
         &self,
         cluster_segments: &BTreeSet<SlotNumber>,
+    ) -> Option<SlotNumber> {
+        self.lowest_user_data_slot_outside_the_region(cluster_segments, None)
+    }
+
+    /// 同 [`Self::lowest_user_data_slot`]，另加一条合取「候选落点不在活整理意图的 `R` 里」（D3（空间分配） 已定项 7 登记表第 5 行、
+    /// D26（后台整理与放置回收） 已定项 1 第 3 条）。`excluded_region` 是 `None` 时与它相同。
+    #[must_use]
+    pub fn lowest_user_data_slot_outside_the_region(
+        &self,
+        cluster_segments: &BTreeSet<SlotNumber>,
+        excluded_region: Option<CompactionRegion>,
     ) -> Option<SlotNumber> {
         // 起点在聚簇段边界上（`UnitAreaStart` 的不变量）⇒ 是偶数槽，按 2 槽步进的候选都 32768 对齐。
         let mut candidate = self.unit_area_start.slot().0;
@@ -793,7 +917,10 @@ impl DeviceFreeMap {
             let inside_cluster_segment = cluster_segments.iter().any(|segment| {
                 candidate >= segment.0 && candidate < segment.0 + CLUSTER_SEGMENT_SLOTS
             });
+            let inside_the_excluded_region =
+                excluded_region.is_some_and(|region| region.contains_slot(SlotNumber(candidate)));
             if !inside_cluster_segment
+                && !inside_the_excluded_region
                 && self.is_free(SlotNumber(candidate))
                 && self.is_free(SlotNumber(candidate + 1))
             {
@@ -807,6 +934,15 @@ impl DeviceFreeMap {
     /// 单元区内最低的 64 槽对齐全空段的起点。
     #[must_use]
     pub fn lowest_empty_segment(&self) -> Option<SlotNumber> {
+        self.lowest_empty_segment_outside_the_region(None)
+    }
+
+    /// 同 [`Self::lowest_empty_segment`]，活整理意图的 `R` 那一段不开（D26（后台整理与放置回收） 已定项 1 第 3 条）。
+    #[must_use]
+    pub fn lowest_empty_segment_outside_the_region(
+        &self,
+        excluded_region: Option<CompactionRegion>,
+    ) -> Option<SlotNumber> {
         let per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
         let full_segments = self.allocated.len() / per_segment;
         (0..full_segments)
@@ -814,6 +950,9 @@ impl DeviceFreeMap {
                 self.used_per_segment[*segment] == 0
                     && self.isolated_per_segment[*segment] == 0
                     && self.held_per_segment[*segment] == 0
+                    && !excluded_region.is_some_and(|region| {
+                        region.contains_slot(self.segment_start_slot(*segment))
+                    })
             })
             .map(|segment| {
                 SlotNumber(
@@ -830,12 +969,24 @@ impl DeviceFreeMap {
     pub fn lowest_commit_generated_fallback_slot(
         &self,
         footprint: UnitFootprint,
+    ) -> Option<SlotNumber> {
+        self.lowest_commit_generated_fallback_slot_outside_the_region(footprint, None)
+    }
+
+    /// 同 [`Self::lowest_commit_generated_fallback_slot`]，排除活整理意图的 `R`（D3（空间分配） 已定项 8 ②「同样排除 `R`」）。
+    #[must_use]
+    pub fn lowest_commit_generated_fallback_slot_outside_the_region(
+        &self,
+        footprint: UnitFootprint,
+        excluded_region: Option<CompactionRegion>,
     ) -> Option<SlotNumber> {
         let unit_area_end = self.unit_area_end_slot();
         let mut candidate = self.unit_area_start.slot().0;
         while candidate + footprint.slots() <= unit_area_end {
-            let is_whole_span_unblocked = (candidate..candidate + footprint.slots())
-                .all(|slot| !self.is_blocked_for_commit_generated(SlotNumber(slot)));
+            let is_whole_span_unblocked = (candidate..candidate + footprint.slots()).all(|slot| {
+                !self.is_blocked_for_commit_generated(SlotNumber(slot))
+                    && !excluded_region.is_some_and(|region| region.contains_slot(SlotNumber(slot)))
+            });
             if is_whole_span_unblocked {
                 return Some(SlotNumber(candidate));
             }
@@ -850,12 +1001,25 @@ impl DeviceFreeMap {
         &self,
         footprint: UnitFootprint,
     ) -> Option<CommitGeneratedDeviceAnswer> {
-        match self.lowest_empty_segment() {
+        self.commit_generated_answer_without_open_segment_outside_the_region(footprint, None)
+    }
+
+    /// 同 [`Self::commit_generated_answer_without_open_segment`]，开段与回落都排除活整理意图的 `R`。
+    #[must_use]
+    pub fn commit_generated_answer_without_open_segment_outside_the_region(
+        &self,
+        footprint: UnitFootprint,
+        excluded_region: Option<CompactionRegion>,
+    ) -> Option<CommitGeneratedDeviceAnswer> {
+        match self.lowest_empty_segment_outside_the_region(excluded_region) {
             Some(segment_start) => {
                 Some(CommitGeneratedDeviceAnswer::OpenEmptySegment(segment_start))
             }
             None => self
-                .lowest_commit_generated_fallback_slot(footprint)
+                .lowest_commit_generated_fallback_slot_outside_the_region(
+                    footprint,
+                    excluded_region,
+                )
                 .map(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot),
         }
     }
@@ -1618,6 +1782,10 @@ pub struct PoolAllocator {
     /// 删除那几次发布的两个只供测试的开关（`unlink::DeletionSwitches`）。只住内存：`new` 与 `rebuild_from_records` 起步是产品值；
     /// 可写挂载按调用方给的装（`mount::mount_writable_with_test_only_switches_and_deletion_switches`），挂载续做与会话照它。
     deletion_switches: DeletionSwitches,
+    /// 整理在分配器这一侧的状态（`compaction_plan::CompactionSwitchesAndTheExcludedRegion`：只供测试的开关、活意图的 R 排他、
+    /// 发布路径隔离过还没交给会话的槽）。R 排他就是分配器的一条合取（D26（后台整理与放置回收） 已定项 1 第 3 条）；不合格段表、常驻隔离集与观测
+    /// 住会话（`compaction_plan::CompactionOfThisMount`）。`new` 与 `rebuild_from_records` 起步是产品值、没有活意图。
+    compaction: CompactionSwitchesAndTheExcludedRegion,
 }
 
 impl PoolAllocator {
@@ -1640,9 +1808,95 @@ impl PoolAllocator {
             instance_rows_after_this_mounts_row_publish: 0,
             space_admission: SpaceAdmission::JudgedByTheFormula,
             deletion_switches: DeletionSwitches::PRODUCT,
+            compaction: CompactionSwitchesAndTheExcludedRegion::of_a_new_mount(),
         }
     }
 
+    /// 装上整理的只供测试的开关（`compaction_plan::CompactionSwitches`）。产品路径一处都不调它。
+    pub fn set_compaction_switches(
+        &mut self,
+        switches: crate::compaction_plan::CompactionSwitches,
+    ) {
+        self.compaction.switches = switches;
+    }
+
+    /// 整理在分配器这一侧的状态（开关、活意图的 R 排他、发布路径隔离过还没交给会话的槽）。
+    #[must_use]
+    pub fn compaction(&self) -> &CompactionSwitchesAndTheExcludedRegion {
+        &self.compaction
+    }
+
+    /// 同 [`Self::compaction`]，可改：会话签发、删、丢、暂停意图时装上与撤掉 R 排他，挂载按盘上的活意图装上，发布路径记隔离都经它。
+    pub fn compaction_mut(&mut self) -> &mut CompactionSwitchesAndTheExcludedRegion {
+        &mut self.compaction
+    }
+
+    /// 落点此刻要排除的 R（活整理意图且排他时；D3（空间分配） 已定项 7 登记表第 5 行）。
+    #[must_use]
+    pub fn region_excluded_from_placement(&self) -> Option<CompactionRegion> {
+        self.compaction.region_excluded_from_placement()
+    }
+
+    /// 只供测试的目的地政策（`compaction_plan::RelocationDestinationPolicy::LowestUnblockedSlotForEveryUnitForTests`）：每块盘取单元区里
+    /// 起点最小、整跨度没挡、不在活 R 里的落点，各盘一致才分配，记一条分配代为 `generation` 的记录。
+    ///
+    /// # Errors
+    /// 同 [`Self::try_allocate_commit_generated`]：拒绝时分配器一样都没动。
+    pub fn try_allocate_at_the_lowest_unblocked_slot_for_tests(
+        &mut self,
+        footprint: UnitFootprint,
+        generation: CheckpointTxg,
+    ) -> Result<Placement, PlacementRefusal> {
+        let excluded_region = self.region_excluded_from_placement();
+        let slot_per_device: Vec<(DeviceIdentity, Option<SlotNumber>)> = self
+            .devices
+            .iter()
+            .map(|device| {
+                (
+                    device.device,
+                    device.lowest_commit_generated_fallback_slot_outside_the_region(
+                        footprint,
+                        excluded_region,
+                    ),
+                )
+            })
+            .collect();
+        let slot = match agreement_across_devices(&slot_per_device) {
+            DeviceAgreement::Agreed(slot) => slot,
+            DeviceAgreement::NoAnswerOnAnyDevice => {
+                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);
+            }
+            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {
+                return Err(
+                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },
+                );
+            }
+            DeviceAgreement::AnswersDiffer(slot_per_device) => {
+                return Err(
+                    PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
+                        slot_per_device,
+                    },
+                );
+            }
+        };
+        let placement = Placement {
+            slot,
+            span: footprint.slots(),
+        };
+        self.record(placement, generation);
+        Ok(placement)
+    }
+
+    /// 发布返回之后每块盘要发 discard 的段起点（[`DeviceFreeMap::take_the_segments_to_discard`]）：按盘，每块盘自己的那几段。
+    pub fn take_the_segments_to_discard_on_every_device(
+        &mut self,
+    ) -> Vec<(DeviceIdentity, Vec<SlotNumber>)> {
+        self.devices
+            .iter_mut()
+            .map(|device_map| (device_map.device, device_map.take_the_segments_to_discard()))
+            .collect()
+    }
+
     /// 装上删除那几次发布的只供测试的开关（`unlink::DeletionSwitches`）。产品路径一处都不调它。
     pub fn set_deletion_switches(&mut self, deletion_switches: DeletionSwitches) {
         self.deletion_switches = deletion_switches;
@@ -1952,13 +2206,17 @@ impl PoolAllocator {
         generation: CheckpointTxg,
     ) -> Result<Placement, PlacementRefusal> {
         let cluster_segments = &self.cluster_segments;
+        let excluded_region = self.compaction.region_excluded_from_placement();
         let slot_per_device: Vec<(DeviceIdentity, Option<SlotNumber>)> = self
             .devices
             .iter()
             .map(|device| {
                 (
                     device.device,
-                    device.lowest_user_data_slot(cluster_segments),
+                    device.lowest_user_data_slot_outside_the_region(
+                        cluster_segments,
+                        excluded_region,
+                    ),
                 )
             })
             .collect();
@@ -2011,13 +2269,17 @@ impl PoolAllocator {
                 return Ok(self.record_bumped(slot, footprint, generation));
             }
         }
+        let excluded_region = self.compaction.region_excluded_from_placement();
         let answer_per_device: Vec<(DeviceIdentity, Option<CommitGeneratedDeviceAnswer>)> = self
             .devices
             .iter()
             .map(|device| {
                 (
                     device.device,
-                    device.commit_generated_answer_without_open_segment(footprint),
+                    device.commit_generated_answer_without_open_segment_outside_the_region(
+                        footprint,
+                        excluded_region,
+                    ),
                 )
             })
             .collect();
@@ -2857,6 +3119,68 @@ mod tests {
         assert_eq!(pool.devices[1].free_runs(), 4, "两盘同构");
     }
 
+    /// 全空聚簇段数按记录口径（D3（空间分配） 已定项 10 ①、D26（后台整理与放置回收） 已定项 1 第 1 条，C372（全空聚簇段数两份定义相反））：
+    /// 一段里只剩已释放、还在 defer 窗口里的槽时它算全空，而那几槽照旧占着（[`DeviceFreeMap::segment_is_empty`] 不收它）；
+    /// 复活把它减回去，回收不动这个数。单元区末尾凑不满 64 槽的那一截不是聚簇段：开头不数它，往里分配也不动这个数。
+    #[test]
+    fn a_segment_left_with_only_deferred_slots_counts_as_empty_by_the_records_while_its_slots_stay_occupied(
+    ) {
+        let full_segments = 3u64;
+        let slots_past_the_last_full_segment = 10u64;
+        let device_bytes = (UNIT_AREA_START_SLOT
+            + full_segments * CLUSTER_SEGMENT_SLOTS
+            + slots_past_the_last_full_segment)
+            * SLOT_BYTES;
+        let mut map = DeviceFreeMap::new(DeviceIdentity(0), device_bytes);
+        assert_eq!(
+            map.empty_segments(),
+            full_segments,
+            "末尾那 10 槽凑不满一段，不数"
+        );
+        let segment_index = 1usize;
+        let slot_in_the_segment =
+            SlotNumber(map.unit_area_start().slot().0 + CLUSTER_SEGMENT_SLOTS + 4);
+        map.mark_allocated(slot_in_the_segment, 2);
+        assert_eq!(
+            map.empty_segments(),
+            full_segments - 1,
+            "分配进第 1 段：那一段罩着未释放的记录"
+        );
+        map.mark_released(slot_in_the_segment, 2);
+        assert_eq!(
+            map.empty_segments(),
+            full_segments,
+            "释放之后第 1 段只剩 defer 槽：按记录口径全空"
+        );
+        assert!(
+            !map.segment_is_empty(segment_index),
+            "defer 槽仍占着：按占用第 1 段不空"
+        );
+        map.mark_resurrected_out_of_the_defer_queue(slot_in_the_segment, 2);
+        assert_eq!(
+            map.empty_segments(),
+            full_segments - 1,
+            "复活之后那条记录又是未释放的"
+        );
+        map.mark_released(slot_in_the_segment, 2);
+        map.mark_reclaimed(slot_in_the_segment, 2);
+        assert_eq!(
+            map.empty_segments(),
+            full_segments,
+            "回收的是已释放的记录，不动记录口径的数"
+        );
+        assert!(map.segment_is_empty(segment_index), "回收之后按占用也空了");
+        map.mark_allocated(
+            SlotNumber(map.unit_area_start().slot().0 + full_segments * CLUSTER_SEGMENT_SLOTS),
+            2,
+        );
+        assert_eq!(
+            map.empty_segments(),
+            full_segments,
+            "分配进末尾那一截不动全空聚簇段数"
+        );
+    }
+
     #[test]
     fn releasing_a_placement_keeps_the_slots_occupied_and_moves_them_into_the_defer_queue() {
         let mut pool = pool_after_mkfs();
diff --git a/crates/singlefs-core/src/block_device.rs b/crates/singlefs-core/src/block_device.rs
index cde55803..eaf896ec 100644
--- a/crates/singlefs-core/src/block_device.rs
+++ b/crates/singlefs-core/src/block_device.rs
@@ -1,4 +1,4 @@
-//! 块设备抽象：读、写、屏障、FUA 写、探测 `physical_block_size`、写零。
+//! 块设备抽象：读、写、屏障、FUA 写、探测 `physical_block_size`、写零、discard。
 //!
 //! 第一版两个后端（D17（实现分层与第三方管道） 已定项 5）：文件后端（测试镜像）与 `O_DIRECT` 后端（虚机里的真块设备）。
 //! 两个后端的持久语义相同：屏障 = `sync_data`（块设备上是一次 FLUSH），FUA 写 = 写完立刻 `sync_data`。
@@ -14,6 +14,12 @@
 //! 读回全 0、文件长度不变与真写 0 相同；持久同样由后面那次屏障给：打洞是元数据改动，ext4 的 `ext4_punch_hole`
 //! 把它记进 `fdatasync` 要等的那个日志事务（本机 6.17 内核源码 `fs/ext4/inode.c` 里调 `ext4_update_inode_fsync_trans(handle, inode, 1)`）。
 //! `O_DIRECT` 后端照旧真写 0：真设备上的块层计数与设备日志比对（checker 档 `device_log.rs` 的折叠）靠的是那些写。
+//!
+//! discard 是第七个动作（`research/prompts/m4-gc-r1-main-verification.md` 第三节 G5「trait 形态」；D26（后台整理与放置回收）
+//! 未定项 9「还缺一个前提」：盘的 FTL 只有收到 discard 才知道哪些块不用了）。它与写零的差别在**结局由设备定**：
+//! 持久之后那一段读回 0 还是旧字节由设备定，模型（崩溃点重放）按「范围写零」枚举两态（D13（验证路线） 已定项 4：持久 → 0、没持久 → 旧字节）。
+//! 后端不认 discard 时交回 [`DiscardOutcome::Unsupported`]、一个字节不动；文件后端复用打洞，`O_DIRECT` 后端第一版不认（`BLKDISCARD` ioctl 留到真盘线）。
+//! 谁发、什么时候发（分配器的可发谓词、发布返回之后发）不在这一层。
 
 use std::fs::{File, OpenOptions};
 use std::io::{self, Seek, SeekFrom};
@@ -32,6 +38,38 @@ pub enum WriteDurability {
     ForceUnitAccess,
 }
 
+/// 一次 [`BlockDevice::discard_at`] 在这块盘上落成了什么（`.claude/rules/fs-design.md`「五条硬要求」第 4 条：分支可观测，调用方按它数）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DiscardOutcome {
+    /// 后端把这一段交给了底层：之后读这一段可能是 0、也可能还是旧字节，持久之后是哪一个由设备定。
+    Applied,
+    /// 后端不认 discard：这一段一个字节没动，读回仍是旧字节。不是错——discard 只是告诉盘「这一段不用了」。
+    Unsupported,
+}
+
+/// 一个句柄上的 discard 各交回了哪一种结局：两项之和就是这个句柄上交回 `Ok` 的 discard 次数。
+#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
+pub struct DiscardOutcomeCounts {
+    pub applied_calls: u64,
+    pub unsupported_calls: u64,
+}
+
+impl DiscardOutcomeCounts {
+    /// 一次也没交回过。
+    pub const NONE: DiscardOutcomeCounts = DiscardOutcomeCounts {
+        applied_calls: 0,
+        unsupported_calls: 0,
+    };
+
+    /// 记下一次交回的结局。
+    pub fn note(&mut self, outcome: DiscardOutcome) {
+        match outcome {
+            DiscardOutcome::Applied => self.applied_calls += 1,
+            DiscardOutcome::Unsupported => self.unsupported_calls += 1,
+        }
+    }
+}
+
 /// 挂载时探测到的物理块大小；不许硬编码（D20（承重面：单元的原子性与自包含） 推论三）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct PhysicalBlockSizeInBytes(pub u32);
@@ -111,7 +149,10 @@ impl std::error::Error for BlockDeviceError {}
 /// 比它大只再快 1–2 ms 而暂存区从 4 MiB 长到 64 MiB（`O_DIRECT` 那条路每次调用要一块对齐暂存区）。
 pub const ZERO_FILL_CHUNK_BYTES: u64 = 4 * 1024 * 1024;
 
-/// 六个动作，只有这六个。介质是什么这里看不见。
+/// 七个动作，只有这七个（与文件头那一行同一份清单：读、写、屏障、FUA 写、探测物理块大小、写零、discard）。介质是什么这里看不见。
+///
+/// 第七个是 [`BlockDevice::discard_at`]：持久之后那一段读回 0 还是旧字节由设备定（不像写零那样必定是 0），
+/// 模型按「范围写零」枚举（D13（验证路线） 已定项 4 的两态：持久 → 0、没持久 → 旧字节）。
 pub trait BlockDevice {
     fn read_at(
         &self,
@@ -138,6 +179,38 @@ pub trait BlockDevice {
         offset: DeviceOffsetInBytes,
         length: u64,
     ) -> Result<(), BlockDeviceError>;
+    /// discard：告诉盘 `[offset, offset + length)` 这一段不用了。偏移与长度同样要按物理块对齐、不越过设备末尾，不合就报
+    /// [`BlockDeviceError::Unaligned`] / [`BlockDeviceError::OutOfRange`]、一个字节不动。
+    ///
+    /// 交回 [`DiscardOutcome::Applied`] 之后，那一段在持久之前读回旧字节或 0 都可能，持久之后读回 0 还是旧字节由设备定；
+    /// 调用方不许指望读回什么（D13（验证路线） 已定项 4：模型按「范围写零」枚举持久 / 没持久两态）。
+    /// 与写零一样没有 [`WriteDurability`] 参数：持久由后面的屏障给。
+    ///
+    /// **长度 0 没有动作**：核过请求（偏移照样要对齐、不越过末尾）就交回 [`DiscardOutcome::Applied`]、一个字节不动、
+    /// 不记进 [`DiscardOutcomeCounts::applied_calls`]；默认实现、文件后端、`O_DIRECT` 后端与 harness 的内存盘结局相同
+    /// （主 agent 2026-10-13 定，判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S5）。
+    ///
+    /// **默认实现只给不认 discard 的后端**：核过请求就交回 [`DiscardOutcome::Unsupported`]（长度 0 除外，见上），一个字节不动。
+    /// 默认实现没有状态、记不了数：「交回了几次不支持」由调用方按返回值数，或由覆盖了它的后端自己数（[`DiscardOutcomeCounts`]）。
+    /// 产品后端必须覆盖它（文件后端、`O_DIRECT` 后端都覆盖了）；包在别的设备外面的包装要转给里面那块盘，
+    /// 不覆盖就把里面那块盘认得的 discard 吞成「不支持」。
+    fn discard_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<DiscardOutcome, BlockDeviceError> {
+        check_aligned_and_in_range(
+            offset,
+            length,
+            self.probe_physical_block_size(),
+            self.size_in_bytes(),
+        )?;
+        if length == 0 {
+            // 默认实现：长度 0 没有动作，与认 discard 的后端结局相同。
+            return Ok(DiscardOutcome::Applied);
+        }
+        Ok(DiscardOutcome::Unsupported)
+    }
     /// 屏障：之前发出的写全部持久之后才返回。
     fn barrier(&mut self) -> Result<(), BlockDeviceError>;
     fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes;
@@ -175,6 +248,25 @@ pub enum ZeroFillLanding {
     },
 }
 
+/// 文件后端的 discard 怎么落到镜像文件上：复用写零的打洞（`punch_hole_keeping_the_file_length`）与同一段按 errno 分路的判定
+/// （`classify_hole_punch_failure`）；与写零不同的只在「不支持打洞」那一路：discard 不退回逐块写 0，交回 [`DiscardOutcome::Unsupported`]。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DiscardLanding {
+    /// 打洞；报不支持打洞交回 [`DiscardOutcome::Unsupported`]、一个字节不动，被信号打断原样重打，报别的错照常交回
+    /// [`BlockDeviceError::InputOutput`]。
+    PunchHoleOrAnswerUnsupported,
+    /// 只供测试的开关（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：不调 `fallocate`，当作它报了 `EOPNOTSUPP`，
+    /// 与真失败走同一段判定，落进「不支持」那条路。
+    ActAsIfPunchingWereUnsupported,
+    /// 只供测试的开关：不调 `fallocate`，当作它报了 `EIO`，落进照常报错那条路。
+    ActAsIfPunchingFailedWithAnInputOutputError,
+    /// 只供测试的开关：这一次 discard 的前 `punches_interrupted_before_the_real_one` 次打洞不调 `fallocate`，每次都当作它报了 `EINTR`；
+    /// 之后那一次照 [`DiscardLanding::PunchHoleOrAnswerUnsupported`] 真调 `fallocate`。
+    ActAsIfTheFirstPunchesWereInterruptedBySignal {
+        punches_interrupted_before_the_real_one: u64,
+    },
+}
+
 /// 打洞失败之后走哪条路，按 `fallocate` 交回的 errno 分（用户 2026-09-30 定「只在不支持时退」）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum HolePunchFailure {
@@ -215,8 +307,15 @@ pub struct FileBackedBlockDevice {
     physical_block_size: PhysicalBlockSizeInBytes,
     zero_fill_landing: ZeroFillLanding,
     zero_fill_landing_counts: ZeroFillLandingCounts,
+    discard_landing: DiscardLanding,
+    discard_outcome_counts: DiscardOutcomeCounts,
+    /// discard 的打洞被信号打断、原样重打的次数（按打洞的次数数，不按 discard 的次数数）。
+    discard_punch_attempts_interrupted_by_signal: u64,
 }
 
+/// 两个构造入口起步时一样：discard 先打洞。
+const INITIAL_DISCARD_LANDING: DiscardLanding = DiscardLanding::PunchHoleOrAnswerUnsupported;
+
 /// 两个构造入口起步时一样：先打洞，计数从 0 起。
 const INITIAL_ZERO_FILL_LANDING: ZeroFillLanding =
     ZeroFillLanding::PunchHoleOrWriteZeroBytesIfUnsupported;
@@ -237,6 +336,26 @@ impl FileBackedBlockDevice {
             physical_block_size: self.physical_block_size,
             zero_fill_landing: landing,
             zero_fill_landing_counts: self.zero_fill_landing_counts,
+            discard_landing: self.discard_landing,
+            discard_outcome_counts: self.discard_outcome_counts,
+            discard_punch_attempts_interrupted_by_signal: self
+                .discard_punch_attempts_interrupted_by_signal,
+        }
+    }
+
+    /// 只供测试：换掉 discard 的落地方式（[`DiscardLanding`] 的三个 `ActAsIf…` 开关各强制走一条路）。
+    #[must_use]
+    pub fn with_test_only_discard_landing(self, landing: DiscardLanding) -> Self {
+        Self {
+            file: self.file,
+            size_in_bytes: self.size_in_bytes,
+            physical_block_size: self.physical_block_size,
+            zero_fill_landing: self.zero_fill_landing,
+            zero_fill_landing_counts: self.zero_fill_landing_counts,
+            discard_landing: landing,
+            discard_outcome_counts: self.discard_outcome_counts,
+            discard_punch_attempts_interrupted_by_signal: self
+                .discard_punch_attempts_interrupted_by_signal,
         }
     }
 
@@ -245,6 +364,18 @@ impl FileBackedBlockDevice {
         self.zero_fill_landing_counts
     }
 
+    /// 这个句柄上的 discard 各交回了哪一种结局（`.claude/rules/fs-design.md`「五条硬要求」第 4 条：分支可观测）。
+    #[must_use]
+    pub fn discard_outcome_counts(&self) -> DiscardOutcomeCounts {
+        self.discard_outcome_counts
+    }
+
+    /// 这个句柄上 discard 的打洞被信号打断、原样重打了几次。
+    #[must_use]
+    pub fn discard_punch_attempts_interrupted_by_signal(&self) -> u64 {
+        self.discard_punch_attempts_interrupted_by_signal
+    }
+
     /// 排他地建出一个镜像文件并把它撑到 `size_in_bytes`；稀疏文件，不真占盘。
     /// 同名文件已经在那儿了就报 [`BlockDeviceError::ImageFileAlreadyExists`]，不接着用旧镜像：
     /// 测试的镜像名带进程号，而进程号会被系统复用、上一轮被杀留下的文件不会被清——接着用就是拿上一轮的盘面当这一轮的起点，
@@ -276,6 +407,9 @@ impl FileBackedBlockDevice {
             physical_block_size,
             zero_fill_landing: INITIAL_ZERO_FILL_LANDING,
             zero_fill_landing_counts: NO_ZERO_FILL_LANDED_YET,
+            discard_landing: INITIAL_DISCARD_LANDING,
+            discard_outcome_counts: DiscardOutcomeCounts::NONE,
+            discard_punch_attempts_interrupted_by_signal: 0,
         })
     }
 
@@ -309,6 +443,9 @@ impl FileBackedBlockDevice {
             physical_block_size,
             zero_fill_landing: INITIAL_ZERO_FILL_LANDING,
             zero_fill_landing_counts: NO_ZERO_FILL_LANDED_YET,
+            discard_landing: INITIAL_DISCARD_LANDING,
+            discard_outcome_counts: DiscardOutcomeCounts::NONE,
+            discard_punch_attempts_interrupted_by_signal: 0,
         })
     }
 
@@ -399,8 +536,8 @@ fn punch_hole_keeping_the_file_length(
         libc::off_t::try_from(length).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
     // SAFETY: `fallocate` 只读它的四个整数参数，不碰进程内存；描述符取自 `file`，
     // 而 `file` 借用到这次调用返回，调用期间不会被关掉或换成别的文件。偏移与长度由上面的 `try_from`
-    // 转成非负的 `off_t`，放不下就在转换处拒；越界的请求到不了这里：唯一的调用方 `write_zeroes_at` 先过 `check_request`
-    // （对齐、落在设备范围内），长度 0 也在 `write_zeroes_at` 里先交回、不到这里。
+    // 转成非负的 `off_t`，放不下就在转换处拒；越界的请求到不了这里：两个调用方 `write_zeroes_at` 与 `discard_at` 都先过 `check_request`
+    // （对齐、落在设备范围内），长度 0 也在它们里面先交回、不到这里。
     // 跨过文件末尾的打洞，文件内那一半照常打成 0，文件外的部分内核不做，都不碰内存。返回值在下面查了。
     let return_code = unsafe {
         libc::fallocate(
@@ -514,6 +651,68 @@ impl BlockDevice for FileBackedBlockDevice {
         }
     }
 
+    /// 复用写零的打洞：打成洞之后读这一段是全 0，文件长度不变（D13（验证路线） 已定项 4 的「持久 → 0」那一态在镜像文件上就是它）。
+    /// 文件系统不支持打洞时交回 [`DiscardOutcome::Unsupported`]、不退回逐块写 0（discard 不承诺读回 0，写 0 只是白写）；
+    /// 被信号打断原样重打；别的错照常报 [`BlockDeviceError::InputOutput`]。
+    fn discard_at(
+        &mut self,
+        discarded_offset: DeviceOffsetInBytes,
+        discarded_length: u64,
+    ) -> Result<DiscardOutcome, BlockDeviceError> {
+        self.check_request(discarded_offset, discarded_length)?;
+        // 长度 0 没有动作（[`BlockDevice::discard_at`] 的文档）：`fallocate(2)` 对它报 `EINVAL`，不调；照「交给了底层」交回、不记数。
+        if discarded_length == 0 {
+            return Ok(DiscardOutcome::Applied);
+        }
+        // 循环：每一轮打一次洞。跨轮的状态只有这一次 discard 里被信号打断的次数（只增）；
+        // 出口三个：打成洞、报不支持、报别的错。被打断就原样再打，轮数的上界是这期间送到的信号数。
+        let mut interrupted_in_this_discard: u64 = 0;
+        loop {
+            let punch_result = match self.discard_landing {
+                DiscardLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal {
+                    punches_interrupted_before_the_real_one,
+                } if interrupted_in_this_discard < punches_interrupted_before_the_real_one => {
+                    Err(io::Error::from_raw_os_error(libc::EINTR))
+                }
+                DiscardLanding::PunchHoleOrAnswerUnsupported
+                | DiscardLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal { .. } => {
+                    punch_hole_keeping_the_file_length(
+                        &self.file,
+                        discarded_offset,
+                        discarded_length,
+                    )
+                }
+                DiscardLanding::ActAsIfPunchingWereUnsupported => {
+                    Err(io::Error::from_raw_os_error(libc::EOPNOTSUPP))
+                }
+                DiscardLanding::ActAsIfPunchingFailedWithAnInputOutputError => {
+                    Err(io::Error::from_raw_os_error(libc::EIO))
+                }
+            };
+            let discard_punch_error = match punch_result {
+                Ok(()) => {
+                    self.discard_outcome_counts.note(DiscardOutcome::Applied);
+                    return Ok(DiscardOutcome::Applied);
+                }
+                Err(discard_punch_error) => discard_punch_error,
+            };
+            match classify_hole_punch_failure(&discard_punch_error) {
+                HolePunchFailure::PunchingUnsupported => {
+                    self.discard_outcome_counts
+                        .note(DiscardOutcome::Unsupported);
+                    return Ok(DiscardOutcome::Unsupported);
+                }
+                HolePunchFailure::InterruptedBySignal => {
+                    interrupted_in_this_discard += 1;
+                    self.discard_punch_attempts_interrupted_by_signal += 1;
+                }
+                HolePunchFailure::OtherFailure => {
+                    return Err(BlockDeviceError::InputOutput(discard_punch_error));
+                }
+            }
+        }
+    }
+
     fn barrier(&mut self) -> Result<(), BlockDeviceError> {
         self.file.sync_data().map_err(BlockDeviceError::InputOutput)
     }
@@ -618,6 +817,7 @@ pub struct DirectInputOutputBlockDevice {
     size_in_bytes: u64,
     physical_block_size: PhysicalBlockSizeInBytes,
     policy: PageCachePolicy,
+    discard_outcome_counts: DiscardOutcomeCounts,
 }
 
 impl DirectInputOutputBlockDevice {
@@ -652,8 +852,15 @@ impl DirectInputOutputBlockDevice {
             size_in_bytes,
             physical_block_size,
             policy,
+            discard_outcome_counts: DiscardOutcomeCounts::NONE,
         })
     }
+
+    /// 这个句柄上的 discard 各交回了哪一种结局；第一版每一次都记「不支持」，长度 0 的不记（[`BlockDevice::discard_at`] 在这个后端上的文档）。
+    #[must_use]
+    pub fn discard_outcome_counts(&self) -> DiscardOutcomeCounts {
+        self.discard_outcome_counts
+    }
 }
 
 impl BlockDevice for DirectInputOutputBlockDevice {
@@ -734,6 +941,25 @@ impl BlockDevice for DirectInputOutputBlockDevice {
         })
     }
 
+    /// 第一版不认 discard：核过请求就交回 [`DiscardOutcome::Unsupported`]，一个字节不动、不发系统调用，两臂（绕开 / 走页缓存）一样；
+    /// 长度 0 的照 trait 文档交回 [`DiscardOutcome::Applied`]、不记数。
+    /// 真块设备上的 discard 是 `BLKDISCARD` ioctl，留到真盘线（`research/prompts/m4-gc-r1-main-verification.md` 第三节 G5「真盘 ioctl 留到真盘线」）；
+    /// 在那之前虚机里的设备侧日志里不会有 Discard 条目，宿主重跑那一侧要照这个结局投（checker 档 `device_log.rs`）。
+    fn discard_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<DiscardOutcome, BlockDeviceError> {
+        check_aligned_and_in_range(offset, length, self.physical_block_size, self.size_in_bytes)?;
+        if length == 0 {
+            // `O_DIRECT` 后端：长度 0 没有动作，交回「交给了底层」、不记数（[`BlockDevice::discard_at`] 的文档）。
+            return Ok(DiscardOutcome::Applied);
+        }
+        self.discard_outcome_counts
+            .note(DiscardOutcome::Unsupported);
+        Ok(DiscardOutcome::Unsupported)
+    }
+
     fn barrier(&mut self) -> Result<(), BlockDeviceError> {
         self.file.sync_data().map_err(BlockDeviceError::InputOutput)
     }
@@ -1592,4 +1818,266 @@ mod tests {
             assert_eq!(scratch.as_slice().len(), length);
         }
     }
+
+    /// discard 用例的镜像：64 KiB、整个先填成 0xC3（稀疏镜像本来就读回全 0，不先弄脏就分不出「打过洞」与「从来没写过」），
+    /// 换上 `landing` 那一种 discard 落地方式。
+    const DISCARD_IMAGE_BYTES: u64 = 65536;
+    const DISCARD_DIRT_BYTE: u8 = 0xC3;
+
+    fn dirty_image_for_discards(
+        tag: &str,
+        landing: DiscardLanding,
+    ) -> (FileBackedBlockDevice, ImageRemovedOnDrop) {
+        let path = temporary_image_path(tag);
+        let mut device = FileBackedBlockDevice::create_image_file_exclusively(
+            &path,
+            DISCARD_IMAGE_BYTES,
+            PhysicalBlockSizeInBytes(512),
+        )
+        .expect("建镜像");
+        let dirt = vec![DISCARD_DIRT_BYTE; usize::try_from(DISCARD_IMAGE_BYTES).expect("64 KiB")];
+        device
+            .write_at(DeviceOffsetInBytes(0), &dirt, WriteDurability::Plain)
+            .expect("填脏");
+        (
+            device.with_test_only_discard_landing(landing),
+            ImageRemovedOnDrop(path),
+        )
+    }
+
+    fn sector_at(device: &FileBackedBlockDevice, offset: u64) -> Vec<u8> {
+        let mut sector = vec![0x5Au8; 512];
+        device
+            .read_at(DeviceOffsetInBytes(offset), &mut sector)
+            .expect("读一扇区");
+        sector
+    }
+
+    /// 文件后端的 discard 复用打洞：段内读回全 0、段外一个字节不动、文件长度不变，交回「交给了底层」并记一次。
+    #[test]
+    fn a_discard_on_the_file_backend_punches_the_range_back_to_zero_and_leaves_the_neighbours() {
+        let (mut device, _removed) = dirty_image_for_discards(
+            "discard-punched",
+            DiscardLanding::PunchHoleOrAnswerUnsupported,
+        );
+        let outcome = device
+            .discard_at(DeviceOffsetInBytes(8192), 16384)
+            .expect("discard");
+        assert_eq!(outcome, DiscardOutcome::Applied);
+        for offset in [8192u64, 8192 + 16384 - 512] {
+            assert!(
+                sector_at(&device, offset).iter().all(|byte| *byte == 0),
+                "偏移 {offset} 在 discard 段内，打过洞读回全 0"
+            );
+        }
+        for offset in [8192u64 - 512, 8192 + 16384] {
+            assert!(
+                sector_at(&device, offset)
+                    .iter()
+                    .all(|byte| *byte == DISCARD_DIRT_BYTE),
+                "偏移 {offset} 在 discard 段外，不该被动"
+            );
+        }
+        assert_eq!(
+            device.file.metadata().expect("读文件元数据").len(),
+            DISCARD_IMAGE_BYTES,
+            "打洞不改文件长度"
+        );
+        assert_eq!(
+            device.discard_outcome_counts(),
+            DiscardOutcomeCounts {
+                applied_calls: 1,
+                unsupported_calls: 0
+            }
+        );
+        assert_eq!(
+            device.zero_fill_landing_counts(),
+            NO_ZERO_FILL_LANDED_YET,
+            "discard 不记进写零的落地计数"
+        );
+    }
+
+    /// 文件系统不支持打洞：交回「不支持」、一个 0 都不写（段里还是旧字节），记一次「不支持」。
+    #[test]
+    fn a_discard_where_hole_punching_is_unsupported_answers_unsupported_and_leaves_the_old_bytes() {
+        let (mut device, _removed) = dirty_image_for_discards(
+            "discard-unsupported",
+            DiscardLanding::ActAsIfPunchingWereUnsupported,
+        );
+        let outcome = device
+            .discard_at(DeviceOffsetInBytes(8192), 16384)
+            .expect("discard");
+        assert_eq!(outcome, DiscardOutcome::Unsupported);
+        assert!(
+            sector_at(&device, 8192)
+                .iter()
+                .all(|byte| *byte == DISCARD_DIRT_BYTE),
+            "不支持时一个字节不动"
+        );
+        assert_eq!(
+            device.discard_outcome_counts(),
+            DiscardOutcomeCounts {
+                applied_calls: 0,
+                unsupported_calls: 1
+            }
+        );
+    }
+
+    /// 打洞报 `EIO`：照常交回块设备 I/O 错，不吞成「不支持」，段里还是旧字节，两种结局都不记。
+    #[test]
+    fn a_discard_whose_hole_punch_fails_with_an_input_output_error_is_reported_and_changes_nothing()
+    {
+        let (mut device, _removed) = dirty_image_for_discards(
+            "discard-input-output-error",
+            DiscardLanding::ActAsIfPunchingFailedWithAnInputOutputError,
+        );
+        let outcome = device.discard_at(DeviceOffsetInBytes(8192), 16384);
+        assert!(
+            matches!(&outcome, Err(BlockDeviceError::InputOutput(error)) if error.raw_os_error() == Some(libc::EIO)),
+            "EIO 照常报错：{outcome:?}"
+        );
+        assert!(sector_at(&device, 8192)
+            .iter()
+            .all(|byte| *byte == DISCARD_DIRT_BYTE));
+        assert_eq!(device.discard_outcome_counts(), DiscardOutcomeCounts::NONE);
+    }
+
+    /// 打洞连着被信号打断两次：原样重打到第三次，打成洞、交回「交给了底层」，被打断的次数记 2。
+    #[test]
+    fn a_discard_interrupted_by_a_signal_twice_in_a_row_is_punched_a_third_time() {
+        let (mut device, _removed) = dirty_image_for_discards(
+            "discard-interrupted",
+            DiscardLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal {
+                punches_interrupted_before_the_real_one: 2,
+            },
+        );
+        let outcome = device
+            .discard_at(DeviceOffsetInBytes(8192), 16384)
+            .expect("被打断之后重打成功");
+        assert_eq!(outcome, DiscardOutcome::Applied);
+        assert!(sector_at(&device, 8192).iter().all(|byte| *byte == 0));
+        assert_eq!(device.discard_punch_attempts_interrupted_by_signal(), 2);
+        assert_eq!(device.discard_outcome_counts().applied_calls, 1);
+    }
+
+    /// 没对齐、越过末尾的 discard 在动文件之前就拒，与读写同一条判据；段里还是旧字节。
+    #[test]
+    fn a_discard_off_the_block_boundaries_or_past_the_end_is_refused_before_touching_the_file() {
+        let (mut device, _removed) = dirty_image_for_discards(
+            "discard-refused",
+            DiscardLanding::PunchHoleOrAnswerUnsupported,
+        );
+        assert!(matches!(
+            device.discard_at(DeviceOffsetInBytes(100), 512),
+            Err(BlockDeviceError::Unaligned { .. })
+        ));
+        assert!(matches!(
+            device.discard_at(DeviceOffsetInBytes(DISCARD_IMAGE_BYTES - 512), 1024),
+            Err(BlockDeviceError::OutOfRange { .. })
+        ));
+        assert!(sector_at(&device, DISCARD_IMAGE_BYTES - 512)
+            .iter()
+            .all(|byte| *byte == DISCARD_DIRT_BYTE));
+        assert_eq!(device.discard_outcome_counts(), DiscardOutcomeCounts::NONE);
+    }
+
+    /// 文件后端上长度 0 的 discard 没有动作：交回「交给了底层」、不调 `fallocate`、两种结局都不记，段里还是旧字节
+    /// （判决 m4-batch2-code-r1 S5，四个后端同一结局）。
+    #[test]
+    fn a_zero_length_discard_on_the_file_backend_is_applied_without_being_counted() {
+        let (mut device, _removed) = dirty_image_for_discards(
+            "discard-zero-length",
+            DiscardLanding::PunchHoleOrAnswerUnsupported,
+        );
+        let outcome = device
+            .discard_at(DeviceOffsetInBytes(8192), 0)
+            .expect("长度 0 的 discard");
+        assert_eq!(outcome, DiscardOutcome::Applied, "长度 0 交回交给了底层");
+        assert_eq!(
+            device.discard_outcome_counts(),
+            DiscardOutcomeCounts::NONE,
+            "长度 0 没有动作，不记数"
+        );
+        assert!(sector_at(&device, 8192)
+            .iter()
+            .all(|byte| *byte == DISCARD_DIRT_BYTE));
+    }
+
+    /// `O_DIRECT` 后端上长度 0 的 discard 没有动作：两臂都交回「交给了底层」、不记「不支持」（判决 m4-batch2-code-r1 S5）。
+    #[test]
+    fn a_zero_length_discard_on_the_direct_backend_is_applied_without_being_counted_on_both_arms() {
+        for policy in [
+            PageCachePolicy::BypassWithDirectInputOutput,
+            PageCachePolicy::GoThroughPageCache,
+        ] {
+            let path = temporary_image_path("direct-discard-zero-length");
+            let _removed = ImageRemovedOnDrop(path.clone());
+            let mut file = std::fs::File::create(&path).expect("建文件");
+            std::io::Write::write_all(&mut file, &[DISCARD_DIRT_BYTE; 65536]).expect("填脏");
+            drop(file);
+            let mut device = DirectInputOutputBlockDevice::open_existing(
+                &path,
+                policy,
+                PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(512)),
+            )
+            .expect("打开（临时目录在 ext4 上）");
+            assert_eq!(
+                device
+                    .discard_at(DeviceOffsetInBytes(8192), 0)
+                    .expect("长度 0 的 discard"),
+                DiscardOutcome::Applied,
+                "{policy:?}"
+            );
+            assert_eq!(
+                device.discard_outcome_counts(),
+                DiscardOutcomeCounts::NONE,
+                "{policy:?}：长度 0 没有动作，不记数"
+            );
+        }
+    }
+
+    /// `O_DIRECT` 后端第一版不认 discard：两臂都交回「不支持」、一个字节不动、每次记一次；没对齐的照样拒。
+    #[test]
+    fn the_direct_backend_answers_every_discard_unsupported_on_both_arms_and_counts_it() {
+        for policy in [
+            PageCachePolicy::BypassWithDirectInputOutput,
+            PageCachePolicy::GoThroughPageCache,
+        ] {
+            let path = temporary_image_path("direct-discard");
+            let _removed = ImageRemovedOnDrop(path.clone());
+            let mut file = std::fs::File::create(&path).expect("建文件");
+            std::io::Write::write_all(&mut file, &[DISCARD_DIRT_BYTE; 65536]).expect("填脏");
+            drop(file);
+            let mut device = DirectInputOutputBlockDevice::open_existing(
+                &path,
+                policy,
+                PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(512)),
+            )
+            .expect("打开（临时目录在 ext4 上）");
+            assert_eq!(
+                device
+                    .discard_at(DeviceOffsetInBytes(8192), 16384)
+                    .expect("discard"),
+                DiscardOutcome::Unsupported,
+                "{policy:?}"
+            );
+            assert!(matches!(
+                device.discard_at(DeviceOffsetInBytes(100), 512),
+                Err(BlockDeviceError::Unaligned { .. })
+            ));
+            let mut sector = vec![0u8; 512];
+            device
+                .read_at(DeviceOffsetInBytes(8192), &mut sector)
+                .expect("读");
+            assert!(sector.iter().all(|byte| *byte == DISCARD_DIRT_BYTE));
+            assert_eq!(
+                device.discard_outcome_counts(),
+                DiscardOutcomeCounts {
+                    applied_calls: 0,
+                    unsupported_calls: 1
+                },
+                "{policy:?}"
+            );
+        }
+    }
 }
diff --git a/crates/singlefs-core/src/code_two_tree.rs b/crates/singlefs-core/src/code_two_tree.rs
index 941fc9da..ae79058d 100644
--- a/crates/singlefs-core/src/code_two_tree.rs
+++ b/crates/singlefs-core/src/code_two_tree.rs
@@ -38,6 +38,10 @@ impl CodeTwoKeyFieldWidths {
     /// 中央映射 key 27：类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 尾段 6（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10：
     /// 码 1 的尾段是事务号低 48 位；码 2 / 码 3 是出生序号 4 + 补零 2，按 6 字节整数比与按出生序号比同序）。
     pub const CENTRAL_MAPPING: Self = Self(&[1, 8, 8, 4, 6]);
+    /// deadlist 条目的 key 43：头的树 ID 8 + 纪元 8 + 中央映射 key 的五段（D5（快照 / 空间记账机制） 已定项 12；
+    /// 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）。按头、纪元、映射 key 逐字段比，
+    /// 同一个头同一段纪元的条目挨着（「按头 / 按快照删一段」那一读法要的次序）。
+    pub const DEADLIST: Self = Self(&[8, 8, 1, 8, 8, 4, 6]);
 
     /// key 宽：各字段宽之和。
     #[must_use]
@@ -704,6 +708,27 @@ pub fn plan_the_tree_after_this_publish(
     })
 }
 
+/// 同 [`plan_the_tree_after_this_publish`]，另收这一版一把 key 都不剩的那一格：这次之后零根（形状一个节点都没有），
+/// 上一版的每个节点都换下（deadlist 树：删快照放掉最后几条之后零根，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3、S4；
+/// 树表条目的根指针置零，同 day-1 注册的那一形）。上一版也是零根、这一版也一把 key 都没有时一个节点都不写。
+///
+/// # Errors
+/// 同 [`plan_the_tree_after_this_publish`]。
+pub fn plan_the_tree_after_this_publish_down_to_a_zero_root(
+    previous: &CodeTwoTreeShape,
+    keys_after_this_publish: &BTreeSet<CodeTwoTreeKey>,
+    capacity: CodeTwoTreeNodeCapacity,
+) -> Result<CodeTwoTreePlan, CodeTwoTreeRefusal> {
+    if !keys_after_this_publish.is_empty() {
+        return plan_the_tree_after_this_publish(previous, keys_after_this_publish, capacity);
+    }
+    Ok(CodeTwoTreePlan {
+        shape: CodeTwoTreeShape { nodes: Vec::new() },
+        origins: Vec::new(),
+        replaced_previous_nodes: previous.nodes().iter().map(|node| node.position).collect(),
+    })
+}
+
 /// 一版树在内存里的样子：形状加每个节点这一版的指针（与 `shape.nodes()` 同序）。节点的字节住
 /// `TransactionOutput::units` 里它那个角色的单元（`transaction::TransactionUnit` 的记账树 / 中央映射树两族）。
 #[derive(Clone, Debug, Default, PartialEq, Eq)]
@@ -1217,6 +1242,74 @@ mod tests {
             .collect()
     }
 
+    /// deadlist 树按节点格式算的容量：叶 376、内部 125（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3：
+    /// key 43、value 空，叶条目宽 = key 宽；内部条目 = key 43 + 子指针 86）。
+    #[test]
+    fn a_deadlist_node_holds_three_hundred_seventy_six_leaf_entries_or_one_hundred_twenty_five_children(
+    ) {
+        let key_width = CodeTwoKeyFieldWidths::DEADLIST.key_width_in_bytes();
+        assert_eq!(key_width, 43, "key 43：头 8 + 纪元 8 + 映射 key 27");
+        assert_eq!(
+            CodeTwoTreeNodeCapacity::of_the_node_format(key_width, key_width),
+            CodeTwoTreeNodeCapacity {
+                leaf_entries: 376,
+                internal_entries: 125,
+            }
+        );
+    }
+
+    /// 这一版一把 key 都不剩：零根，上一版的节点全部换下；上一版也是零根时一个节点都不写也不换。
+    #[test]
+    fn a_tree_whose_last_key_is_deleted_plans_a_zero_root_and_replaces_every_previous_node() {
+        let previous = two_leaves_under_a_root(5);
+        let plan = plan_the_tree_after_this_publish_down_to_a_zero_root(
+            &previous,
+            &BTreeSet::new(),
+            CodeTwoTreeNodeCapacity {
+                leaf_entries: 2,
+                internal_entries: 2,
+            },
+        )
+        .expect("零根那一格不拒");
+        assert!(plan.shape.nodes().is_empty(), "零根：一个节点都没有");
+        assert_eq!(
+            plan.replaced_previous_nodes,
+            previous
+                .nodes()
+                .iter()
+                .map(|node| node.position)
+                .collect::<Vec<_>>(),
+            "上一版的三个节点都换下"
+        );
+        let still_zero = plan_the_tree_after_this_publish_down_to_a_zero_root(
+            &CodeTwoTreeShape::default(),
+            &BTreeSet::new(),
+            CodeTwoTreeNodeCapacity {
+                leaf_entries: 2,
+                internal_entries: 2,
+            },
+        )
+        .expect("零根接零根");
+        assert!(
+            still_zero.shape.nodes().is_empty() && still_zero.replaced_previous_nodes.is_empty(),
+            "零根接零根：不写也不换"
+        );
+        let with_keys = plan_the_tree_after_this_publish_down_to_a_zero_root(
+            &CodeTwoTreeShape::default(),
+            &keys([4]),
+            CodeTwoTreeNodeCapacity {
+                leaf_entries: 2,
+                internal_entries: 2,
+            },
+        )
+        .expect("零根上插一把");
+        assert_eq!(
+            with_keys.shape.nodes().len(),
+            1,
+            "有 key 时照常规划：一片叶"
+        );
+    }
+
     #[test]
     fn internal_entries_are_the_key_plus_an_eighty_six_byte_child_pointer() {
         use singlefs_format::{
diff --git a/crates/singlefs-core/src/lib.rs b/crates/singlefs-core/src/lib.rs
index 4525d8f3..00196609 100644
--- a/crates/singlefs-core/src/lib.rs
+++ b/crates/singlefs-core/src/lib.rs
@@ -17,6 +17,7 @@ pub mod block_device;
 pub mod bytes;
 pub mod checksum;
 pub mod code_two_tree;
+pub mod compaction_plan;
 pub mod container_tree;
 pub mod deleted_inodes;
 pub mod extent_tree;
@@ -25,6 +26,7 @@ pub mod instance_table;
 pub mod instance_table_sweep;
 pub mod journal;
 pub mod lineage_tree;
+pub mod logged_operations;
 pub mod make_filesystem;
 pub mod mount;
 pub mod mounted_read;
@@ -32,6 +34,7 @@ pub mod mounted_session;
 pub mod pointer;
 pub mod records;
 pub mod recovery;
+pub mod region_scan;
 pub mod retirement_tree;
 pub mod root_record;
 pub mod root_ring;
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 8699bbec..3101b2a7 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -24,17 +24,22 @@ use crate::allocator::{
 };
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
+use crate::compaction_plan::{CompactionOfThisMount, CompactionSwitches};
 use crate::instance_table_sweep::{
     sweep_the_instance_table_at_the_end_of_the_mount, InstanceTableSweepAtTheEndOfTheMount,
     InstanceTableSweepConfiguration,
 };
 use crate::journal::back_chain_of;
+use crate::logged_operations::live_compaction_intent_of_the_entries;
 use crate::make_filesystem::MakeFilesystemParameters;
-use crate::mounted_session::refusal_is_short_of_space;
+use crate::mounted_session::{
+    refusal_is_short_of_space, run_compaction_on, CompactionRun, CompactionRunReach,
+};
 use crate::pointer::LocationEntry;
 use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
 use crate::records::{
     LineageRecord, TreeTableEntry, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE,
+    TREE_KIND_SNAPSHOT_INODE_ROOT,
 };
 use crate::recovery::read_unit_via_locations;
 use crate::recovery::{
@@ -1006,6 +1011,20 @@ pub struct MountOutput {
     /// 或这次挂载里判、抹、发到了哪儿（[`crate::instance_table_sweep::sweep_the_instance_table_at_the_end_of_the_mount`]）。
     /// 发成了清扫发布时 `Mounted::current` 是清扫发布那一版。
     pub instance_table_sweep: InstanceTableSweepAtTheEndOfTheMount,
+    /// 盘上那一条未完成的整理意图的续做（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1；D8（核心索引结构） 已定项 5
+    /// 第 2 条），连同这次挂载会话这一侧的整理状态（会话接着用，`mounted_session::MountedSession::of_the_mount`）。
+    pub compaction_continuation: CompactionContinuationAtTheMount,
+}
+
+/// 可写挂载续做整理意图（[`MountOutput::compaction_continuation`]）：次序在删除续做之后（判决 G8「删除续做 → 为用户请求推的抬 F → GC 批」，
+/// 挂载里没有用户请求）、实例表清扫之前；只推盘上那一条，不签新的。
+#[derive(Debug)]
+pub struct CompactionContinuationAtTheMount {
+    /// 推了的那一趟（停在哪、发了哪几版：发了的那几版接在删除续做之后，清扫没发布时最后一个就是 `Mounted::current`）。
+    /// 所选那一版的 logged_ops 没有活意图（常态）、或删除续做没走空（整理排在它后面）时 `None`，一个写都不发。
+    pub run: Option<CompactionRun>,
+    /// 这次挂载会话这一侧的整理状态（不合格段表、常驻隔离集、`B_gc` 数的发布数、观测）：续做时建的，会话接着用。
+    pub state_of_this_mount: CompactionOfThisMount,
 }
 
 /// 挂载续做停在哪儿。成员按调用方要做的决定分：照常写；队列还留着——之后会话里每一次用户改动先走空、走不动报空间不够
@@ -3382,7 +3401,9 @@ fn format_time_allocator<Device: BlockDevice>(
 }
 
 /// 一条有效根的用户可见状态有没有变（D16（发布语义） 已定项 1「非空」从盘上怎么认，2026-09-17 用户定案）：它树表里 inode 树与 extent 树的
-/// 根指针，与它前一条有效根树表里的，逐字节比；两棵树任一棵不同就算非空。最旧的有效根没有前一条，拿 `UserVisibleTreeRootPointers::ABSENT` 比。
+/// 根指针、加上只读快照条目（码 14 / 15）的根指针按盘上次序排成的那一串（用户 2026-10-14 定「建快照那次发布算非空」），与它前一条有效根
+/// 树表里的逐字节比；三样任一样不同就算非空。最旧的有效根没有前一条，拿 `UserVisibleTreeRootPointers::ABSENT` 比。
+/// 只比根指针，不比 flags：删快照的意图那次（只写「销毁中」的记号、传前驱）与只放 deadlist 条目的批不算非空，删掉那一对的最后一批算。
 #[must_use]
 pub fn user_visible_trees_changed(
     root_pointers: &UserVisibleTreeRootPointers,
@@ -3390,6 +3411,7 @@ pub fn user_visible_trees_changed(
 ) -> bool {
     root_pointers.inode_tree != previous_valid_root_pointers.inode_tree
         || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree
+        || root_pointers.snapshot_tree_roots != previous_valid_root_pointers.snapshot_tree_roots
 }
 
 /// 算抬 F 的上限时读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-09-26 定）：每个槽读一次；读坏的槽按这个进程知不知道
@@ -6400,6 +6422,29 @@ fn establish_instance<Device: BlockDevice>(
         writes_of_the_publishes_before_the_continuation,
         space_admission.floor_raises(),
     )?;
+    // 整理意图的续做（判决 G1；D8（核心索引结构） 已定项 5 第 2 条「续做是每次恢复都跑的常规动作」；判决 G8 次序：删除续做 → 抬 F → 整理）：
+    // 现行那一版 logged_ops 里有一条未完成的意图、删除队列走空了，就照分配器上的整理开关推它，推到它删了、停机原因或预算用完，不签新的。
+    // 整理自己不让挂载失败（提示类意图不挡挂载，D8 已定项 5），走到哪儿记进交回值；产品路径批大小条款没写，签任何东西、发任何写之前停下。
+    let mut compaction_state_of_this_mount = CompactionOfThisMount::of_a_new_mount();
+    let live_intent_to_continue = match &deletion_queue_continuation_outcome {
+        DeletionQueueContinuationOutcome::WalkedToEmpty => current
+            .version_of_trees()
+            .and_then(|version| live_compaction_intent_of_the_entries(&version.logged_operations)),
+        DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. } => None,
+    };
+    let compaction_run = if live_intent_to_continue.is_some() {
+        Some(run_compaction_on(
+            parameters,
+            devices.as_mut_slice(),
+            &mut allocator,
+            &mut compaction_state_of_this_mount,
+            &mut current,
+            instance,
+            CompactionRunReach::ContinueTheLiveIntentAtTheMount,
+        ))
+    } else {
+        None
+    };
     // 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」，用户 2026-10-03 定）：写行、暖机、抬 F、删除续做之后，
     // 交回之前判一次。清扫自己不让挂载失败（落点被拒这一轮不删、挂载照样交回可写，Q4），走到哪儿记进交回值。
     let instance_table_sweep = sweep_the_instance_table_at_the_end_of_the_mount(
@@ -6436,6 +6481,10 @@ fn establish_instance<Device: BlockDevice>(
             refusals_that_pushed_the_continuation_floor_raises,
             deletion_queue_continuation_outcome,
             instance_table_sweep,
+            compaction_continuation: CompactionContinuationAtTheMount {
+                run: compaction_run,
+                state_of_this_mount: compaction_state_of_this_mount,
+            },
         },
         allocator,
         current,
@@ -6519,7 +6568,12 @@ fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
         PoolVersion::WithoutFile(_) => return Ok(continuation),
         PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
     };
-    if current_version_of_trees.deleted_inodes.is_empty() {
+    // 销毁中的快照（判决 S4：崩在意图与最后一批之间，下一次可写挂载续做）与 `deleted_inodes` 同一个走空走它们。
+    if current_version_of_trees.deleted_inodes.is_empty()
+        && current_version_of_trees
+            .snapshots_being_destroyed
+            .is_empty()
+    {
         return Ok(continuation);
     }
     let switches = allocator.deletion_switches();
@@ -7218,6 +7272,7 @@ pub fn mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread<
         space_admission,
         shadow_ledger,
         DeletionSwitches::PRODUCT,
+        CompactionSwitches::PRODUCT,
         before_the_one_reread,
         InstanceTableSweepConfiguration::PRODUCT,
     )
@@ -7242,6 +7297,30 @@ pub fn mount_writable_with_test_only_switches_and_deletion_switches<Device: Bloc
         space_admission,
         shadow_ledger,
         deletion_switches,
+        CompactionSwitches::PRODUCT,
+        BeforeTheOneReread::RereadImmediately,
+        InstanceTableSweepConfiguration::PRODUCT,
+    )
+}
+
+/// 同 [`mount_writable_with_test_only_switches`]，整理的只供测试的开关（`compaction_plan::CompactionSwitches`，判决 G6）也由调用方给：
+/// 装在交回的分配器上，挂载续做盘上那一条未完成的整理意图（[`MountOutput::compaction_continuation`]）与这次挂载之后的会话照它。
+/// 产品路径走 [`mount_writable`]，恒 `CompactionSwitches::PRODUCT`（批大小条款没写，续做在签任何东西之前停下）。
+///
+/// # Errors
+/// 同 [`mount_writable_with_test_only_switches`]：续做自己不报错（提示类意图不挡挂载，D8（核心索引结构） 已定项 5），走到哪儿记进交回值。
+pub fn mount_writable_with_compaction_switches<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    compaction_switches: CompactionSwitches,
+) -> Result<Mounted, MountError> {
+    mount_writable_with_every_test_only_switch(
+        parameters,
+        devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        DeletionSwitches::PRODUCT,
+        compaction_switches,
         BeforeTheOneReread::RereadImmediately,
         InstanceTableSweepConfiguration::PRODUCT,
     )
@@ -7267,18 +7346,24 @@ pub fn mount_writable_with_instance_table_sweep_configuration<Device: BlockDevic
         space_admission,
         shadow_ledger,
         DeletionSwitches::PRODUCT,
+        CompactionSwitches::PRODUCT,
         BeforeTheOneReread::RereadImmediately,
         instance_table_sweep,
     )
 }
 
-/// 可写挂载的全部只供测试的开关一处收：空间准入、影子账、删除开关、「重读一次」之前的钩子、实例表清扫的配置。
+/// 可写挂载的全部只供测试的开关一处收：空间准入、影子账、删除开关、整理开关、「重读一次」之前的钩子、实例表清扫的配置。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样只供测试的开关或挂载的输入，不成一个概念"
+)]
 fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
     space_admission: SpaceAdmission,
     shadow_ledger: ShadowLedger,
     deletion_switches: DeletionSwitches,
+    compaction_switches: CompactionSwitches,
     mut before_the_one_reread: BeforeTheOneReread<'_>,
     instance_table_sweep: InstanceTableSweepConfiguration,
 ) -> Result<Mounted, MountError> {
@@ -7417,6 +7502,15 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         root_ring_of_the_rebuilt_allocator,
     } = rebuilt?;
     allocator.set_deletion_switches(deletion_switches);
+    allocator.set_compaction_switches(compaction_switches);
+    // 重建时读回活意图、恢复 R 排他（D26（后台整理与放置回收） 已定项 1 第 3 条：意图住 logged_ops，分配器读得到）：在写行、暖机、抬 F 与任何
+    // 续做之前装上，这次挂载里每一次发布的新落点都不进 R（暂停是会话那一侧的事，挂载里不暂停）。
+    if let Some((_issued_txg, intent)) = previous
+        .version_of_trees()
+        .and_then(|version| live_compaction_intent_of_the_entries(&version.logged_operations))
+    {
+        allocator.compaction_mut().issue(intent);
+    }
     let previous_row = PreviousInstanceRow {
         instance: effective_root.instance,
         selected_root_txg: effective_root.checkpoint_txg,
@@ -7495,6 +7589,13 @@ pub enum RollbackError {
     TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
         target: RollbackTarget,
     },
+    /// 现行那一版或回退目标那一版的树表里有只读快照条目（码 14 / 15）：第一版有快照就拒回退（用户 2026-10-13 定「第一版有快照就拒」，
+    /// 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S8 与第七节岔路 3）。`version` 说是哪一版先被认出带着快照
+    /// （现行那一版在任何读之前判，目标那一版在重建出来之后判），`snapshots` 是那一版码 14 的条数。在任何写之前拒，盘上逐字节不变。
+    SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
+        version: VersionOfTheRollback,
+        snapshots: usize,
+    },
     /// 回退目标引用的一个用户可见单元，在现行那一版的账里这块盘上没有它的记录（没有起点是这个槽、跨度相同的那一条）。
     UserVisibleUnitWithoutItsRecordInTheCurrentAccount {
         unit: TransactionUnit,
@@ -7537,6 +7638,22 @@ pub enum RollbackError {
     Publish(Box<PublishSequenceFailed>),
 }
 
+/// 回退牵涉的两版里的哪一版（[`RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum VersionOfTheRollback {
+    Current,
+    Target,
+}
+
+/// 一版树表里码 14（只读快照的 inode 根）的条数。
+fn snapshots_in_the_tree_table(version: &TransactionOutput) -> usize {
+    version
+        .tree_table_entries
+        .iter()
+        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
+        .count()
+}
+
 /// 复活的一份：哪个单元、哪块盘、落点、写回的分配代（回退目标那一版账里的）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct ResurrectedCopy {
@@ -7574,14 +7691,17 @@ pub const fn is_a_user_visible_unit(identity: TransactionUnit) -> bool {
         | TransactionUnit::AllocationTree
         | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
         | TransactionUnit::AccountingTree
+        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+        | TransactionUnit::DeadlistTree
         | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
         | TransactionUnit::MappingTree
         | TransactionUnit::TreeTable
         | TransactionUnit::InstanceTable
         | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
         // `deleted_inodes` 根不在 D23 那个记号里（新根照样取 R_old 的，见 [`is_a_unit_taken_from_the_rollback_target`]）；
-        // 墓碑树、谱系树与退役树随现行那一版（退役树：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）。
+        // 墓碑树、谱系树、logged_ops 与退役树随现行那一版（退役树：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）。
         TransactionUnit::DeletedInodesRoot
+        | TransactionUnit::LoggedOperationsRoot
         | TransactionUnit::TombstoneLeafContainer(_)
         | TransactionUnit::TombstoneRoot
         | TransactionUnit::TombstoneNodeBelowTheRoot(_)
@@ -7612,6 +7732,8 @@ pub const fn is_a_unit_taken_from_the_rollback_target(identity: TransactionUnit)
         | TransactionUnit::AllocationTree
         | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
         | TransactionUnit::AccountingTree
+        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+        | TransactionUnit::DeadlistTree
         | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
         | TransactionUnit::MappingTree
         | TransactionUnit::TreeTable
@@ -7624,6 +7746,7 @@ pub const fn is_a_unit_taken_from_the_rollback_target(identity: TransactionUnit)
         | TransactionUnit::LineageRoot
         | TransactionUnit::LineageNodeBelowTheRoot(_)
         | TransactionUnit::RetirementLeafContainer(_)
+        | TransactionUnit::LoggedOperationsRoot
         | TransactionUnit::RetirementRoot
         | TransactionUnit::RetirementNodeBelowTheRoot(_) => is_a_user_visible_unit(identity),
     }
@@ -8040,6 +8163,7 @@ fn version_to_publish_the_rollback_after(
             } else {
                 entry.root
             },
+            previous_snapshot_txg: entry.previous_snapshot_txg,
             birth_txg: entry.birth_txg,
             head_identifier: entry.head_identifier,
         })
@@ -8061,11 +8185,20 @@ fn version_to_publish_the_rollback_after(
         accounting_entries: current.accounting_entries.clone(),
         accounting_tree: current.accounting_tree.clone(),
         central_mapping_tree: current.central_mapping_tree.clone(),
+        // 两版都不带快照（回退之前判过，判决 S8）：deadlist 与留存的映射条目照现行那一版（删完全部快照之后是空的）。
+        deadlist_entries: current.deadlist_entries.clone(),
+        deadlist_tree: current.deadlist_tree.clone(),
+        mapping_entries_kept_for_the_snapshots: current
+            .mapping_entries_kept_for_the_snapshots
+            .clone(),
+        snapshots_being_destroyed: current.snapshots_being_destroyed.clone(),
         tree_table_entries,
         tree_identifiers: current.tree_identifiers,
         inode_record: target_version.inode_record,
         // `deleted_inodes` 取 R_old 的（树表那一条上面换了），墓碑树随现行那一版（D23（journal 的角色与格式） 已定项 14）。
         deleted_inodes: target_version.deleted_inodes.clone(),
+        // logged_ops 随现行那一版（D23（journal 的角色与格式） 已定项 14「墓碑树、谱系树、logged_ops 随现行那一版」）：未完成的整理意图跨回退留着、接着推。
+        logged_operations: current.logged_operations.clone(),
         tombstone_containers: current.tombstone_containers.clone(),
         tombstone_tree_index_nodes: current.tombstone_tree_index_nodes.clone(),
         // 谱系树同样随现行那一版（D18（块里携带什么信息） 已定项 24 第 5 条「随现行那一版、只增不删」）：回退这次往它上面追加。
@@ -8079,6 +8212,7 @@ fn version_to_publish_the_rollback_after(
         mapped_units,
         released: Vec::new(),
         quarantined_after_release_checksum_mismatch: Vec::new(),
+        units_relocated_by_this_publish: Vec::new(),
         key_order_mismatches: 0,
         writes: WritesByStructureKind::NOTHING_WRITTEN,
         highest_transaction_number_in_this_instance: current
@@ -8121,6 +8255,16 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
             },
         );
     }
+    // 第一版有快照就拒回退（判决 S8、岔路 3）：现行那一版在内存里，在任何读之前判；目标那一版重建出来之后判。
+    let snapshots_in_the_current_version = snapshots_in_the_tree_table(current);
+    if snapshots_in_the_current_version > 0 {
+        return Err(
+            RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
+                version: VersionOfTheRollback::Current,
+                snapshots: snapshots_in_the_current_version,
+            },
+        );
+    }
     // 调用方的参数与盘表照可写挂载同一套核，写入口只按盘上那一份建（实审 A1b 留下的这一处）。
     let CallerInputsAgreeingWithTheDisk {
         system_configuration,
@@ -8191,6 +8335,15 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
     ) {
         return Err(RollbackError::CurrentAccountUnreadable { failure });
     }
+    let snapshots_in_the_target_version = snapshots_in_the_tree_table(&target_version);
+    if snapshots_in_the_target_version > 0 {
+        return Err(
+            RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses {
+                version: VersionOfTheRollback::Target,
+                snapshots: snapshots_in_the_target_version,
+            },
+        );
+    }
     if target_version.tree_identifiers != current.tree_identifiers {
         return Err(
             RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
@@ -8428,10 +8581,49 @@ mod non_empty_root_tests {
         );
     }
 
+    /// 建快照那次发布算非空（用户 2026-10-14 定，D16（发布语义） 已定项 1「「非空」从盘上怎么认」）：它与前一条有效根只差快照条目的根指针
+    /// （活头两棵树照抄），照样算非空；去重的元组也带快照根，两条根只差一对快照条目时是两个不同状态。删快照的意图那次只改 flags、
+    /// 快照根指针不变，不算非空。
+    #[test]
+    fn a_root_that_only_adds_a_snapshot_pair_is_non_empty_and_counts_as_a_state_of_its_own() {
+        let before_the_snapshot = pointers(1, 2);
+        let with_the_snapshot = UserVisibleTreeRootPointers {
+            inode_tree: Some(vec![1; 86]),
+            extent_tree: Some(vec![2; 86]),
+            snapshot_tree_roots: vec![vec![1; 86], vec![2; 86]],
+        };
+        assert!(
+            user_visible_trees_changed(&with_the_snapshot, &before_the_snapshot),
+            "活头两棵树照抄、多了一对快照根：非空"
+        );
+        assert!(
+            !user_visible_trees_changed(&with_the_snapshot, &with_the_snapshot.clone()),
+            "快照根也照抄（删快照的意图那次、只放 deadlist 的批）：不算非空"
+        );
+        let distinct = newest_first_distinct_states(vec![
+            NonEmptyValidRoot {
+                checkpoint_txg: CheckpointTxg(5),
+                instance: InstanceGeneration(2),
+                user_visible_tree_root_pointers: before_the_snapshot,
+            },
+            NonEmptyValidRoot {
+                checkpoint_txg: CheckpointTxg(6),
+                instance: InstanceGeneration(2),
+                user_visible_tree_root_pointers: with_the_snapshot,
+            },
+        ]);
+        assert_eq!(
+            distinct,
+            vec![CheckpointTxg(6), CheckpointTxg(5)],
+            "只差一对快照根的两条根是两个不同状态"
+        );
+    }
+
     fn pointers(inode_tree: u8, extent_tree: u8) -> UserVisibleTreeRootPointers {
         UserVisibleTreeRootPointers {
             inode_tree: Some(vec![inode_tree; 86]),
             extent_tree: Some(vec![extent_tree; 86]),
+            snapshot_tree_roots: Vec::new(),
         }
     }
 
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index f989c7f5..80962f2d 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -434,7 +434,11 @@ pub fn open_pool_for_read(
             TreeTableEntriesRefusal::EntryMalformed => {
                 OpenPoolForReadFailure::TreeTableEntryMalformed
             }
-            TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice => {
+            TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice
+            | TreeTableEntriesRefusal::SnapshotInodeAndExtentRootEntryCountsDiffer
+            | TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry
+            | TreeTableEntriesRefusal::MoreSnapshotsThanTheFirstVersionAllows
+            | TreeTableEntriesRefusal::SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree => {
                 OpenPoolForReadFailure::Walk(RecoveryFailure::from(refusal))
             }
         })?;
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 0d822163..e7fa2ac9 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -17,15 +17,32 @@
 //! 缺根那一半是 C585（会话发布放进发布途中取下的旧快照） 用户定的改法 C）。
 //! 发布路径与抬 F 各自开写入口的做法不变。
 
-use std::collections::BTreeMap;
+use std::collections::{BTreeMap, BTreeSet};
+
+use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};
 
 use crate::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration,
+    SlotNumber, TreeIdentifier,
 };
 use crate::allocator::{PlacementOnDevice, PoolAllocator};
 use crate::block_device::{
-    check_aligned_and_in_range, BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes,
-    WriteDurability,
+    check_aligned_and_in_range, BlockDevice, BlockDeviceError, DiscardOutcome,
+    PhysicalBlockSizeInBytes, WriteDurability,
+};
+use crate::compaction_plan::{
+    choose_region, compaction_starts_after_a_user_change, empty_cluster_segments_by_the_records,
+    holes_admit_the_issue, holes_outside_the_region_in_segments_holding_user_data, plan_batch,
+    publish_budget_of_a_compaction_run, region_holds_a_quarantined_slot,
+    region_is_empty_by_the_records, region_scans_of_the_segments_without_an_unreleased_record,
+    the_completed_intent_yielded, CompactionBatchSize, CompactionBatchUnits, CompactionOfThisMount,
+    CompactionRunForcing, CompactionRunStop, PlannedBatch, PlannedRelocation, RegionChoice,
+    RegionChoiceCriterion, UnitAreaOfOneDevice,
+};
+use crate::instance_table::InstanceRow;
+use crate::instance_table_sweep::unit_head_read_by_sweep;
+use crate::logged_operations::{
+    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion,
 };
 use crate::make_filesystem::MakeFilesystemParameters;
 use crate::mount::{
@@ -38,14 +55,20 @@ use crate::mount::{
     OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, ParametersAndDeviceTableOfTheMount,
     RaisedFloor, ShadowLedger,
 };
-use crate::root_record::UnmountMarker;
+use crate::recovery::{instance_table_chain_of_root, PoolReader};
+use crate::region_scan::{scan_region, AllocationInRegion, CentralMappingEntry, RegionScan};
+use crate::root_record::{RootRecord, UnmountMarker};
 use crate::root_ring::{target_for_publish, RootRingSlot};
 use crate::transaction::{
-    prepare_the_version_publish, publish_new_inodes, publish_overwrite, publish_sequential_write,
-    publish_unlink_intent, publish_version, walk_the_deletion_queue_until_it_is_empty,
-    DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, PoolVersion, PoolWriter,
-    PublishError, ReleaseChecksumCheck, TransactionOutput,
+    central_mapping_locations_in_the_version, prepare_the_version_publish, publish_compaction_step,
+    publish_create_snapshot, publish_new_inodes, publish_overwrite, publish_sequential_write,
+    publish_snapshot_destruction_intent, publish_unlink_intent, publish_version,
+    walk_the_deletion_queue_until_it_is_empty, CompactionIntentStep,
+    DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, MappingLookup, PoolVersion,
+    PoolWriter, PublishError, ReleaseChecksumCheck, RelocationBatchRefused, TransactionOutput,
+    TransactionUnit,
 };
+use crate::unit::{unit_filesystem_identifier, UNIT_CLASS_DATA};
 use crate::unlink::DeletionSwitches;
 use singlefs_format::ROOT_RING_REGIONS;
 
@@ -60,6 +83,10 @@ pub struct MountedSession {
     /// 这次挂载认下的池（`MountOutput::parameters_and_device_table`；没经过可写挂载的会话读盘造，
     /// `ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`）：写入口只按它的参数建，交进来的盘表与它的逐项比。
     pub parameters_and_device_table: ParametersAndDeviceTableOfTheMount,
+    /// 整理在这次挂载里会话这一侧的状态（`compaction_plan::CompactionOfThisMount`：不合格段表、常驻隔离集、`B_gc` 数的发布数、观测；
+    /// 丁 1 交判第 2 点）：生命周期一次挂载。可写挂载续做意图时建的那一份接着用（`mount::MountOutput::compaction_continuation`）；
+    /// 没经过可写挂载的会话从 `CompactionOfThisMount::of_a_new_mount` 起。活意图不在这里：它在现行那一版的 logged_ops 里。
+    pub compaction: CompactionOfThisMount,
 }
 
 /// 用户的一次改动，经会话发布（发布路径那三种接在带文件的一版后面的改动）。
@@ -78,6 +105,16 @@ pub enum UserChange<'content> {
         inode: InodeNumber,
         write_time_seconds: u64,
     },
+    /// 建一个只读快照（`transaction::publish_create_snapshot`，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2）：
+    /// 先把现行那一版的 `deleted_inodes` 走空（同别的改动，D8（核心索引结构） 已定项 16 第 6 条），再发一次独立发布，不与别的改动合在一起。
+    /// 现行那一版有树而文件集合为空时同覆盖写拒成 `NoFileVersionToChange`（第 5 条：删完之后只许建 inode、写行与空发布）。
+    CreateSnapshot,
+    /// 删一个只读快照（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4 与第七节岔路 2、4）：快照按它码 14 条目的树号点名
+    /// （岔路 5）。一次调用 = 先把现行那一版的队列走空 + 意图那次发布（`transaction::publish_snapshot_destruction_intent`）+ 全部批
+    /// （会话的走空接着走它们，`transaction::walk_the_deletion_queue_until_it_is_empty`），与 unlink 同形、同准入：被空间拒推抬 F 再试，
+    /// 不对「先走空」开例外。现行那一版有树而文件集合为空时照样删（删快照不依赖活头里有没有文件：文件删光之后快照照样删得掉，
+    /// 丙 2 交判第 5 条，主 agent 定是缺陷；D8（核心索引结构） 已定项 16 第 5 条那一句管的是写文件的改动）。
+    DestroySnapshot { snapshot: TreeIdentifier },
 }
 
 /// 一次用户改动发成了：新的一版在会话的 `current` 里。
@@ -89,6 +126,66 @@ pub struct UserChangePublished {
     pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
     /// 这次调用里推的抬 F 之外发成的每一版，按先后（[`VersionsPublishedInThisChange`]）。
     pub versions_published: VersionsPublishedInThisChange,
+    /// 改动发成之后水位越线跑的那至多一批整理（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G8 丁；次序在删除续做、
+    /// 为这次请求推的抬 F 之后）。没起跑是 `None`；起跑了的那一趟停在哪、发了哪几版在里面。整理没发成不让这次改动算没发成。
+    pub compaction_after_the_change: Option<CompactionRun>,
+    /// 这次调用的发布都返回之后发的 discard（判决 G5，[`discard_the_segments_reusable_as_a_whole`]）。
+    pub discards_after_the_change: DiscardsAfterThePublishes,
+}
+
+/// 发布返回之后发的 discard（判决 G5）：每块盘按 64 槽对齐的整段发，各段交回了哪一种结局。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct DiscardsAfterThePublishes {
+    /// 交回 `Applied` 的段：哪块盘、段起点。
+    pub applied: Vec<(DeviceIdentity, SlotNumber)>,
+    /// 交回 `Unsupported` 的段（后端不认 discard，一个字节没动）。
+    pub unsupported: Vec<(DeviceIdentity, SlotNumber)>,
+    /// 块设备报错的段：discard 只是告诉盘「这一段不用了」，报错不让发布算没发成，记下来报出。
+    pub failed: Vec<(DeviceIdentity, SlotNumber)>,
+    /// 装着还没清扫的孤儿头、没发的段（[`discard_the_segments_reusable_as_a_whole`]：discard 不当擦除，孤儿头留给实例表清扫抹）。
+    pub withheld_holding_unswept_orphan_heads: Vec<(DeviceIdentity, SlotNumber)>,
+    /// 判孤儿头时读不出实例表或段里的一个槽、没发的段（读不到不等于没有，按装着算）。
+    pub withheld_after_a_failed_read: Vec<(DeviceIdentity, SlotNumber)>,
+}
+
+/// 一次整理运行（显式入口 [`MountedSession::run_compaction`]，或用户改动之后的那至多一批）做成了什么。
+#[derive(Debug)]
+pub struct CompactionRun {
+    /// 这一趟发成的每一版，按先后：先走空的删除续做、再是整理的各批。最后一个就是会话的 `current`。
+    pub versions_published: Vec<TransactionOutput>,
+    /// 其中整理批的个数。
+    pub batches_published: u64,
+    pub outcome: CompactionRunOutcome,
+    /// 这一趟的发布都返回之后发的 discard（只有显式入口在这里发；用户改动之后那一趟的归改动那一份）。
+    pub discards: DiscardsAfterThePublishes,
+}
+
+/// 一次整理运行停在哪：照判据停（[`CompactionRunStop`]），或一次发布没发成。
+#[derive(Debug)]
+pub enum CompactionRunOutcome {
+    Stopped(CompactionRunStop),
+    Failed(CompactionRunFailure),
+}
+
+/// 整理运行里一次发布没发成（被空间拒不在这里：那是暂停，[`CompactionRunStop::PausedShortOfSpace`]）。已发成的那几版留在盘上、`current` 停在最后那一版。
+#[derive(Debug)]
+pub enum CompactionRunFailure {
+    /// 先走空的删除续做有一步没发成。
+    DeletionContinuation(PublishError),
+    /// 一批没发成（计划对不上上一版，或发布路径的错）。
+    Batch(RelocationBatchRefused),
+}
+
+/// 整理运行在任何读写之前就拒了。
+#[derive(Debug)]
+pub enum CompactionRunRefused {
+    /// 同 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`]：交进来的盘表与这次挂载核过的那一份不同。
+    DeviceTableOtherThanTheOneOfTheMount {
+        device_table_of_the_mount: Vec<DeviceIdentity>,
+        device_table_handed_in: Vec<DeviceIdentity>,
+    },
+    /// 现行那一版树表 0 条：没有可搬的单元。
+    NoVersionOfTrees,
 }
 
 /// 一次用户改动里推的抬 F 之外发成的每一版，按先后：先走空的删除步（D8（核心索引结构） 已定项 16 第 6 条）、这次改动本身
@@ -537,6 +634,10 @@ enum UserChangeKeptByTheRefusal {
         inode: InodeNumber,
         write_time_seconds: u64,
     },
+    CreateSnapshot,
+    DestroySnapshot {
+        snapshot: TreeIdentifier,
+    },
 }
 
 impl UserChangeKeptByTheRefusal {
@@ -564,6 +665,10 @@ impl UserChangeKeptByTheRefusal {
                 inode,
                 write_time_seconds,
             },
+            UserChange::CreateSnapshot => UserChangeKeptByTheRefusal::CreateSnapshot,
+            UserChange::DestroySnapshot { snapshot } => {
+                UserChangeKeptByTheRefusal::DestroySnapshot { snapshot }
+            }
         }
     }
 
@@ -597,6 +702,12 @@ impl UserChangeKeptByTheRefusal {
                 inode: *inode,
                 write_time_seconds: *write_time_seconds,
             },
+            UserChangeKeptByTheRefusal::CreateSnapshot => UserChange::CreateSnapshot,
+            UserChangeKeptByTheRefusal::DestroySnapshot { snapshot } => {
+                UserChange::DestroySnapshot {
+                    snapshot: *snapshot,
+                }
+            }
         }
     }
 }
@@ -910,7 +1021,10 @@ pub struct FloorRaiseFailedWhilePushingForSpace {
 pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {
     match refusal {
         PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,
-        PublishError::MultiLevelCodeTwoTreeRefused { .. }
+        // 合取 7（全空聚簇段少于 `N₀`）：全空段数按记录口径数（`allocator::DeviceFreeMap::empty_segments`，defer 里已释放的槽本来就不算占用），
+        // 抬 F 把 defer 里的槽回收不改这个数，推了只多发几次空发布；原样交回这个成员，调用方分得出「先整理」与「真没空间」。
+        PublishError::EmptyClusterSegmentsBelowTheFloor(_)
+        | PublishError::MultiLevelCodeTwoTreeRefused { .. }
         | PublishError::ReleaseNotInMapping { .. }
         | PublishError::ReleaseTargetNotAllocated { .. }
         | PublishError::ReleaseTargetAlreadyReleased { .. }
@@ -936,6 +1050,11 @@ pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {
         | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
         // `deleted_inodes` 装不下：类型化错误（D8（核心索引结构） 已定项 16 第 6 条），与空间无关。
         | PublishError::DeletedInodesQueueRefused(_)
+        // 快照已到第一版的上限、树 ID 水位到顶：推抬 F 回收不出快照的名额，也抬不低水位。
+        | PublishError::SnapshotCreationRefused(_)
+        // 删快照点名的快照不在、正在删、已有别的在删：推抬 F 改变不了树表。deadlist 条目留存的映射条目没有、落点核不过：盘上的事，与空间无关。
+        | PublishError::SnapshotDestructionRefused(_)
+        | PublishError::DeadlistEntryNotReleasable { .. }
         | PublishError::BlockDevice(_) => false,
     }
 }
@@ -1002,6 +1121,16 @@ fn publish_the_change_once<Device: BlockDevice>(
             write_time_seconds,
             instance,
         ),
+        UserChange::CreateSnapshot => {
+            publish_create_snapshot(&mut writer, allocator, previous, instance)
+        }
+        UserChange::DestroySnapshot { snapshot } => publish_snapshot_destruction_intent(
+            &mut writer,
+            allocator,
+            previous,
+            snapshot,
+            instance,
+        ),
     }
 }
 
@@ -1020,6 +1149,7 @@ impl MountedSession {
             instance: output.instance,
             shadow_ledger: output.shadow_ledger,
             parameters_and_device_table: output.parameters_and_device_table.clone(),
+            compaction: output.compaction_continuation.state_of_this_mount.clone(),
         };
         (output, session)
     }
@@ -1057,6 +1187,7 @@ impl MountedSession {
             instance,
             shadow_ledger,
             parameters_and_device_table,
+            compaction,
         } = self;
         if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
             return Err(UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount {
@@ -1111,7 +1242,7 @@ impl MountedSession {
                 },
             );
         }
-        Self::publish_the_change_raising_the_floor_for_space(
+        let mut published = Self::publish_the_change_raising_the_floor_for_space(
             parameters,
             devices,
             allocator,
@@ -1119,7 +1250,80 @@ impl MountedSession {
             *instance,
             *shadow_ledger,
             change,
-        )
+        )?;
+        // 改动发成：整理「已经过去的发布数」从这里重新数（判决 G8 `B_gc`）。之后水位越线就跑至多一批（判决 G8 丁），
+        // 次序在这次改动的删除续做与为它推的抬 F 之后；整理没发成不让这次改动算没发成。
+        compaction.publishes_since_the_last_user_change = 0;
+        if compaction_starts_after_a_user_change(allocator.compaction().switches.run_forcing) {
+            published.compaction_after_the_change = Some(run_compaction_on(
+                parameters,
+                devices.as_mut_slice(),
+                allocator,
+                compaction,
+                current,
+                *instance,
+                CompactionRunReach::AtMostOneBatchAfterAUserChange,
+            ));
+        }
+        published.discards_after_the_change = discard_the_segments_reusable_as_a_whole(
+            devices.as_mut_slice(),
+            allocator,
+            parameters,
+            current.root(),
+        );
+        Ok(published)
+    }
+
+    /// 整理的显式入口（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G8 甲）：先把删除续做走空，再一次一次推整理意图——
+    /// 没有活意图就选段、签发前判、发一次把意图写进 logged_ops 的发布（判决 G1）；有就对 R 重扫（不复用上一批的扫描，丁 1 交判第 3 点）、
+    /// R 里有隔离槽就发一次删意图的发布丢掉它（攻方 K-c）、计划一批、发这一批，最后一批同一次发布删意图（D8（核心索引结构） 已定项 5）；
+    /// 全局差值为假就不签下一条。这一趟至多发 `B_gc` 次整理发布（[`publish_budget_of_a_compaction_run`]，签发、搬一批、删意图各算一次）；
+    /// 一次整理发布被空间拒就暂停（撤 R 排他、意图留在盘上）、不推抬 F。发布都返回之后发 discard（判决 G5）。
+    /// 批大小条款没写，产品路径在签任何意图之前停下（`CompactionRunStop::BatchSizeNotDecidedByTheClauses`），只供测试的开关给出批大小才推得动。
+    ///
+    /// # Errors
+    /// 盘表与挂载核过的不同、现行那一版树表 0 条 ⇒ [`CompactionRunRefused`]，在任何读写之前。
+    pub fn run_compaction<Device: BlockDevice>(
+        &mut self,
+        devices: &mut Vec<(DeviceIdentity, Device)>,
+    ) -> Result<CompactionRun, CompactionRunRefused> {
+        let MountedSession {
+            allocator,
+            current,
+            instance,
+            shadow_ledger: _,
+            parameters_and_device_table,
+            compaction,
+        } = self;
+        if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
+            return Err(CompactionRunRefused::DeviceTableOtherThanTheOneOfTheMount {
+                device_table_of_the_mount: parameters_and_device_table
+                    .device_identities_in_table_order()
+                    .to_vec(),
+                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
+            });
+        }
+        match &*current {
+            PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {}
+            PoolVersion::WithoutFile(_) => return Err(CompactionRunRefused::NoVersionOfTrees),
+        }
+        let parameters = parameters_and_device_table.parameters_on_disk();
+        let mut run = run_compaction_on(
+            parameters,
+            devices.as_mut_slice(),
+            allocator,
+            compaction,
+            current,
+            *instance,
+            CompactionRunReach::UntilAStopOrTheBudget,
+        );
+        run.discards = discard_the_segments_reusable_as_a_whole(
+            devices.as_mut_slice(),
+            allocator,
+            parameters,
+            current.root(),
+        );
+        Ok(run)
     }
 
     /// [`Self::publish_user_change`] 核过盘表之后那一半：先走空、再发这次改动（unlink 连同它之后的走空），被拒是空间不够就推抬 F 再判。
@@ -1151,7 +1355,12 @@ impl MountedSession {
             versions_published: Vec::new(),
         };
         // 队列空（任何一版都是常态）时一个写都不发、不复制；不空时在一份复制上走，走到哪一步（连同推的抬 F）`current` 就是哪一版。
-        if !current_version_of_trees.deleted_inodes.is_empty() {
+        // 销毁中的快照与 `deleted_inodes` 一起走空（判决 S4：删快照照 unlink 的过程做，第七节岔路 4）。
+        if !current_version_of_trees.deleted_inodes.is_empty()
+            || !current_version_of_trees
+                .snapshots_being_destroyed
+                .is_empty()
+        {
             let mut walked = current_version_of_trees.clone();
             let walk = publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                 parameters,
@@ -1172,10 +1381,13 @@ impl MountedSession {
         let version_to_change = match current {
             PoolVersion::WithFile(file_version) => file_version,
             PoolVersion::WithTreesAndNoFile(emptied_version) => match change {
-                UserChange::NewInodes { .. } => emptied_version,
+                UserChange::NewInodes { .. } | UserChange::DestroySnapshot { .. } => {
+                    emptied_version
+                }
                 UserChange::Overwrite(_)
                 | UserChange::SequentialWrite(_)
-                | UserChange::Unlink { .. } => {
+                | UserChange::Unlink { .. }
+                | UserChange::CreateSnapshot => {
                     return Err(no_file_version_to_change(published_in_this_change));
                 }
             },
@@ -1218,6 +1430,8 @@ impl MountedSession {
                         floor_raises,
                         refusals_that_pushed_the_floor_raises,
                         versions_published,
+                        compaction_after_the_change: None,
+                        discards_after_the_change: DiscardsAfterThePublishes::default(),
                     });
                 }
                 Err(refusal) => refusal,
@@ -1275,6 +1489,7 @@ impl MountedSession {
             instance,
             shadow_ledger: _,
             parameters_and_device_table,
+            compaction: _,
         } = self;
         let is_the_device_table_handed_in_the_one_of_the_mount =
             parameters_and_device_table.is_the_device_table_of_the_mount(devices);
@@ -1500,6 +1715,816 @@ impl MountedSession {
     }
 }
 
+/// 发布返回之后发 discard（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G5，判决 `research/prompts/m4-batch2-code-r1-main-verification.md`
+/// 第二节 S6）：「可再分配」谓词为真（已回收、没被抬 F 扣住、没被隔离、F 已生效）的槽按 64 槽对齐的整段发——只发回收过槽、整段可再分配的段
+/// （`allocator::PoolAllocator::take_the_segments_to_discard_on_every_device`），每块盘发它自己那几段。discard 取几态、设备日志怎么判不在这里
+/// （归设备侧与崩溃点重放）。长度恒为一段（64 × 16384 字节），不发长度 0 的。
+///
+/// 装着还没清扫的孤儿头的段不发（主 agent 2026-10-15 定，零轮）：discard 的结局是三态之一（D13（验证路线） 已定项 4），不能拿它当擦除——
+/// 抹孤儿头归实例表清扫（`instance_table_sweep`），清扫判过、抹过之前段里的孤儿头要原样留着。孤儿头 = 段里某个槽开头认得出本池的单元头
+/// （清扫认头的那一判，`instance_table_sweep::unit_head_read_by_sweep`），它的写序实例在现行那一版的实例表里还有一行、按那一行判未发布
+/// （`UnitHeadReadBySweep::is_unpublished_by_the_row`）。判之前读不出实例表、读不出段里的一个槽，这一段同样不发（读不到不等于没有）。
+pub fn discard_the_segments_reusable_as_a_whole<Device: BlockDevice>(
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    parameters: &MakeFilesystemParameters,
+    current_root: &RootRecord,
+) -> DiscardsAfterThePublishes {
+    let mut discards = DiscardsAfterThePublishes::default();
+    let segments_to_discard = allocator.take_the_segments_to_discard_on_every_device();
+    if segments_to_discard
+        .iter()
+        .all(|(_, segment_starts)| segment_starts.is_empty())
+    {
+        return discards;
+    }
+    let rows_of_the_instance_table: Option<BTreeMap<InstanceGeneration, InstanceRow>> =
+        instance_table_chain_of_root(&*devices, current_root)
+            .ok()
+            .map(|chain| {
+                chain
+                    .records
+                    .rows
+                    .iter()
+                    .map(|row| (row.instance, *row))
+                    .collect()
+            });
+    let filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
+    for (device_identity, segment_starts) in segments_to_discard {
+        for segment_start in segment_starts {
+            let holding = match &rows_of_the_instance_table {
+                Some(rows) => segment_holding_unswept_orphan_heads(
+                    &*devices,
+                    device_identity,
+                    segment_start,
+                    rows,
+                    filesystem_identifier,
+                ),
+                None => SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed,
+            };
+            match holding {
+                SegmentHoldingUnsweptOrphanHeads::No => {}
+                SegmentHoldingUnsweptOrphanHeads::Yes => {
+                    discards
+                        .withheld_holding_unswept_orphan_heads
+                        .push((device_identity, segment_start));
+                    continue;
+                }
+                SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed => {
+                    discards
+                        .withheld_after_a_failed_read
+                        .push((device_identity, segment_start));
+                    continue;
+                }
+            }
+            let Some((_, device)) = devices
+                .iter_mut()
+                .find(|(identity, _)| *identity == device_identity)
+            else {
+                continue;
+            };
+            match device.discard_at(
+                segment_start.to_device_offset(),
+                CLUSTER_SEGMENT_SLOTS * SLOT_BYTES,
+            ) {
+                Ok(DiscardOutcome::Applied) => {
+                    discards.applied.push((device_identity, segment_start));
+                }
+                Ok(DiscardOutcome::Unsupported) => {
+                    discards.unsupported.push((device_identity, segment_start));
+                }
+                Err(_cause) => discards.failed.push((device_identity, segment_start)),
+            }
+        }
+    }
+    discards
+}
+
+/// 一段装没装着还没清扫的孤儿头（[`discard_the_segments_reusable_as_a_whole`]）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum SegmentHoldingUnsweptOrphanHeads {
+    No,
+    Yes,
+    /// 段里一个槽读不出：判不了，按装着算（不发）。
+    NotJudgedReadFailed,
+}
+
+/// `device` 上从 `segment_start` 起的那一段：逐槽读一整槽，开头认得出本池的单元头、写序实例在 `rows` 里有一行、按那一行判未发布的，就是还没清扫的孤儿头。
+/// 认出一个头就按它的整单元跨度跳（同清扫的认领规则）。迭代上界是一段 64 槽；提前出口是认出一个孤儿头与读失败。
+fn segment_holding_unswept_orphan_heads<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    device: DeviceIdentity,
+    segment_start: SlotNumber,
+    rows: &BTreeMap<InstanceGeneration, InstanceRow>,
+    filesystem_identifier: u64,
+) -> SegmentHoldingUnsweptOrphanHeads {
+    let slot_bytes = usize::try_from(SLOT_BYTES).expect("一槽 16384 字节");
+    let mut slot = segment_start.0;
+    while slot < segment_start.0 + CLUSTER_SEGMENT_SLOTS {
+        let Some(bytes) = reader.read(device, SlotNumber(slot).to_device_offset(), slot_bytes)
+        else {
+            return SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed;
+        };
+        let Some(head) = unit_head_read_by_sweep(&bytes) else {
+            slot += 1;
+            continue;
+        };
+        slot += head.span_in_slots();
+        if head.filesystem_identifier == filesystem_identifier
+            && rows
+                .get(&head.writer_instance)
+                .is_some_and(|row| head.is_unpublished_by_the_row(row))
+        {
+            return SegmentHoldingUnsweptOrphanHeads::Yes;
+        }
+    }
+    SegmentHoldingUnsweptOrphanHeads::No
+}
+
+/// 一次整理运行推到哪为止。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub(crate) enum CompactionRunReach {
+    /// 显式入口：推到停机原因或发布预算用完。
+    UntilAStopOrTheBudget,
+    /// 用户改动之后（判决 G8 丁）：至多一批（签发与删意图的发布不算批）。
+    AtMostOneBatchAfterAUserChange,
+    /// 可写挂载续做（D8（核心索引结构） 已定项 5 第 2 条；判决 G1）：只推盘上那一条未完成的意图，推到它删了、停机原因或预算用完，不签新的。
+    ContinueTheLiveIntentAtTheMount,
+}
+
+/// 整理的一次发布做的是哪一种（[`CompactionStep::Published`]）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum CompactionPublishKind {
+    /// 签发那次发布：意图写进 logged_ops。
+    IntentIssued,
+    /// 搬一批，意图照抄。
+    Batch,
+    /// 最后一批：搬完 R 按记录全空，同一次发布删意图（D8（核心索引结构） 已定项 5）。
+    BatchCompletingTheIntent,
+    /// R 已按记录全空（别的发布腾空的），单独一次发布删意图。
+    IntentCompletedWithoutABatch,
+    /// 丢意图：单独一次发布删它（攻方 K-c 的隔离槽、区域扫描拒了 R）。
+    IntentDropped,
+    /// 丢意图：R 里一个能搬的都没有、却还有未释放的记录；丢完这一趟停（[`CompactionRunStop::NothingMovableButTheRegionIsNotEmpty`]）。
+    IntentDroppedWithNothingMovable,
+}
+
+/// 推一步整理的结局：发成一次整理发布，或照判据停。
+enum CompactionStep {
+    Published {
+        version: Box<TransactionOutput>,
+        kind: CompactionPublishKind,
+    },
+    Stopped(CompactionRunStop),
+}
+
+/// 整理一趟（显式入口、用户改动之后、可写挂载续做共用）：先走空删除续做（G8 次序），再照预算一次一次推（[`advance_compaction_by_one_publish`]）。
+/// 预算 `B_gc` 数的是整理的每一次发布（签发、搬一批、删意图：每一次都用掉一个根槽）。discard 由调用方在这一趟之后发。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止"
+)]
+pub(crate) fn run_compaction_on<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    compaction: &mut CompactionOfThisMount,
+    current: &mut PoolVersion,
+    instance: InstanceGeneration,
+    reach: CompactionRunReach,
+) -> CompactionRun {
+    compaction.observations.runs_started += 1;
+    let mut versions_published: Vec<TransactionOutput> = Vec::new();
+    let finish = |compaction_of_the_run: &mut CompactionOfThisMount,
+                  versions_published_by_the_run: Vec<TransactionOutput>,
+                  batches_published: u64,
+                  outcome: CompactionRunOutcome| {
+        if let CompactionRunOutcome::Stopped(stop) = &outcome {
+            compaction_of_the_run.observations.last_stop = Some(*stop);
+        }
+        CompactionRun {
+            versions_published: versions_published_by_the_run,
+            batches_published,
+            outcome,
+            discards: DiscardsAfterThePublishes::default(),
+        }
+    };
+    let switches = allocator.compaction().switches;
+    match switches.run_forcing {
+        CompactionRunForcing::ForcedToStopForTests => {
+            return finish(
+                compaction,
+                versions_published,
+                0,
+                CompactionRunOutcome::Stopped(CompactionRunStop::ForcedToStopForTests),
+            );
+        }
+        CompactionRunForcing::FollowTheWatermarks | CompactionRunForcing::ForcedToStartForTests => {
+        }
+    }
+    let batch_units = match switches.batch_size {
+        CompactionBatchSize::NotDecidedByTheClauses => {
+            return finish(
+                compaction,
+                versions_published,
+                0,
+                CompactionRunOutcome::Stopped(CompactionRunStop::BatchSizeNotDecidedByTheClauses),
+            );
+        }
+        CompactionBatchSize::ForcedForTests(batch_units) => batch_units,
+    };
+    // 次序（判决 G8）：删除续做在整理批之前。
+    let queue_is_empty = match &*current {
+        PoolVersion::WithFile(version_of_trees)
+        | PoolVersion::WithTreesAndNoFile(version_of_trees) => {
+            version_of_trees.deleted_inodes.is_empty()
+                && version_of_trees.snapshots_being_destroyed.is_empty()
+        }
+        PoolVersion::WithoutFile(_) => true,
+    };
+    if !queue_is_empty {
+        let mut walked = current
+            .version_of_trees()
+            .expect("队列不空的一版是有树的一版")
+            .clone();
+        let walk = publish_the_queue_until_it_is_empty(
+            parameters,
+            devices,
+            allocator,
+            &mut walked,
+            instance,
+        );
+        *current = PoolVersion::of_a_version_of_trees(walked);
+        versions_published.extend(walk.steps_published);
+        if let Err(cause) = walk.stopped_by {
+            return finish(
+                compaction,
+                versions_published,
+                0,
+                CompactionRunOutcome::Failed(CompactionRunFailure::DeletionContinuation(cause)),
+            );
+        }
+    }
+    let budget = publish_budget_of_a_compaction_run(
+        parameters.geometry.root_ring_slots_per_region.count(),
+        compaction.publishes_since_the_last_user_change,
+    );
+    let mut publishes_of_the_run: u64 = 0;
+    let mut batches_published: u64 = 0;
+    // 循环：每一轮发成一次整理发布或停。迭代上界是预算（每发成一次计一次，到 budget 停）；跨轮携带的是现行那一版、上一条完成的意图、
+    // 已发的次数与批数。
+    let mut last_completed_intent: Option<CompactionIntent> = None;
+    loop {
+        if publishes_of_the_run >= budget {
+            return finish(
+                compaction,
+                versions_published,
+                batches_published,
+                CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget }),
+            );
+        }
+        if reach == CompactionRunReach::AtMostOneBatchAfterAUserChange && batches_published == 1 {
+            return finish(
+                compaction,
+                versions_published,
+                batches_published,
+                CompactionRunOutcome::Stopped(CompactionRunStop::OneBatchAfterTheUserChangeDone),
+            );
+        }
+        match advance_compaction_by_one_publish(
+            parameters,
+            devices,
+            allocator,
+            compaction,
+            current,
+            instance,
+            batch_units,
+            reach,
+            &mut last_completed_intent,
+        ) {
+            Ok(CompactionStep::Published { version, kind }) => {
+                publishes_of_the_run += 1;
+                compaction.publishes_since_the_last_user_change += 1;
+                compaction.observations.root_ring_slots_used += 1;
+                match kind {
+                    CompactionPublishKind::Batch
+                    | CompactionPublishKind::BatchCompletingTheIntent => {
+                        batches_published += 1;
+                        compaction.observations.batches_published += 1;
+                        let relocated =
+                            u64::try_from(version.units_relocated_by_this_publish.len())
+                                .expect("一批的单元数");
+                        compaction.observations.units_relocated += relocated;
+                        compaction.observations.units_relocated_by_the_last_batch = relocated;
+                    }
+                    CompactionPublishKind::IntentIssued
+                    | CompactionPublishKind::IntentCompletedWithoutABatch
+                    | CompactionPublishKind::IntentDropped
+                    | CompactionPublishKind::IntentDroppedWithNothingMovable => {}
+                }
+                versions_published.push(*version);
+                if kind == CompactionPublishKind::IntentDroppedWithNothingMovable {
+                    return finish(
+                        compaction,
+                        versions_published,
+                        batches_published,
+                        CompactionRunOutcome::Stopped(
+                            CompactionRunStop::NothingMovableButTheRegionIsNotEmpty,
+                        ),
+                    );
+                }
+            }
+            Ok(CompactionStep::Stopped(stop)) => {
+                return finish(
+                    compaction,
+                    versions_published,
+                    batches_published,
+                    CompactionRunOutcome::Stopped(stop),
+                );
+            }
+            Err(refused) => {
+                return finish(
+                    compaction,
+                    versions_published,
+                    batches_published,
+                    CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),
+                );
+            }
+        }
+    }
+}
+
+/// 一版里整理要读的几样：每个进映射单元的映射条目（按 key 在这一版的映射树里查，连同留存的），钉住的槽（豁免映射的三类：映射树各节点、
+/// 树表、实例表链——根记录直接指着它们），码 1 数据单元落着的槽（签发前判「只装提交内生块的段不算非全空」用）。
+struct VersionAsTheCompactionReadsIt {
+    mapping_entries: Vec<CentralMappingEntry>,
+    pinned_slots: BTreeSet<SlotNumber>,
+    slots_holding_user_data: BTreeSet<SlotNumber>,
+}
+
+fn version_as_the_compaction_reads_it(
+    version: &TransactionOutput,
+) -> VersionAsTheCompactionReadsIt {
+    let mut mapping_entries: Vec<CentralMappingEntry> = version
+        .mapped_units
+        .iter()
+        .filter_map(
+            |(_, key)| match central_mapping_locations_in_the_version(version, key) {
+                MappingLookup::Found(locations) => Some(CentralMappingEntry {
+                    key: key.clone(),
+                    locations,
+                }),
+                MappingLookup::NodeMalformedOrNoEntryWithThisKey
+                | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
+            },
+        )
+        .collect();
+    mapping_entries.extend(version.mapping_entries_kept_for_the_snapshots.iter().map(
+        |(key, locations)| CentralMappingEntry {
+            key: key.clone(),
+            locations: *locations,
+        },
+    ));
+    let mut pinned_slots = BTreeSet::new();
+    for unit in &version.units {
+        let is_exempt_from_the_mapping = match unit.identity {
+            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
+            | TransactionUnit::MappingTree
+            | TransactionUnit::TreeTable
+            | TransactionUnit::InstanceTable
+            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => true,
+            TransactionUnit::Data(_)
+            | TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+            | TransactionUnit::ExtentRoot
+            | TransactionUnit::InodeLeafContainer(_)
+            | TransactionUnit::InodeRoot
+            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AllocationTree
+            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AccountingTree
+            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+            | TransactionUnit::DeadlistTree
+            | TransactionUnit::DeletedInodesRoot
+            | TransactionUnit::LoggedOperationsRoot
+            | TransactionUnit::TombstoneLeafContainer(_)
+            | TransactionUnit::TombstoneRoot
+            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
+            | TransactionUnit::LineageLeafContainer(_)
+            | TransactionUnit::LineageRoot
+            | TransactionUnit::LineageNodeBelowTheRoot(_)
+            | TransactionUnit::RetirementLeafContainer(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_) => false,
+        };
+        if is_exempt_from_the_mapping {
+            for slot in unit.slot.0..unit.slot.0 + unit.identity.span_slots() {
+                pinned_slots.insert(SlotNumber(slot));
+            }
+        }
+    }
+    let mut slots_holding_user_data = BTreeSet::new();
+    for entry in &mapping_entries {
+        if entry.key.first() == Some(&UNIT_CLASS_DATA) {
+            let slot = entry.locations[0].slot;
+            slots_holding_user_data.insert(slot);
+            slots_holding_user_data.insert(SlotNumber(slot.0 + 1));
+        }
+    }
+    VersionAsTheCompactionReadsIt {
+        mapping_entries,
+        pinned_slots,
+        slots_holding_user_data,
+    }
+}
+
+/// 在一个新开的写入口上发整理的一次发布（[`publish_compaction_step`]），接在 `previous` 后面。
+fn publish_the_compaction_step_once<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    planned: &[PlannedRelocation],
+    intent_step: CompactionIntentStep,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, RelocationBatchRefused> {
+    let switches = allocator.deletion_switches();
+    let mut writer = PoolWriter::new(parameters, devices);
+    writer.set_container_tree_capacities(switches.container_tree_capacities());
+    publish_compaction_step(
+        &mut writer,
+        allocator,
+        previous,
+        planned,
+        intent_step,
+        instance,
+    )
+}
+
+/// 整理的一次发布被拒是不是空间不够的那几种（[`refusal_is_short_of_space`]）：是就暂停、不推抬 F（判决 G8 g1）。
+fn compaction_publish_refused_short_of_space(refused: &RelocationBatchRefused) -> bool {
+    match refused {
+        RelocationBatchRefused::Publish(cause) => refusal_is_short_of_space(cause),
+        RelocationBatchRefused::PlannedUnitNotInTheVersion { .. }
+        | RelocationBatchRefused::PinnedUnitInThePlan { .. }
+        | RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition { .. }
+        | RelocationBatchRefused::UnitBytesNotAvailable { .. }
+        | RelocationBatchRefused::CompactionIntentAlreadyLive { .. }
+        | RelocationBatchRefused::NoLiveCompactionIntent
+        | RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
+            ..
+        } => false,
+    }
+}
+
+/// 推整理一次发布（判决 G1–G4、G8 的整串；意图住 logged_ops，判决 G1）。活意图读现行那一版的 logged_ops（盘上的权威），
+/// 分配器这一侧的 R 排他与它一致（会话签发、删意图时装上与撤掉，挂载按 logged_ops 装上）。先把发布路径隔离过的槽并进常驻隔离集。
+/// 1. 没有活意图：可写挂载续做停在「没有活意图可续」（只续做、不签新的）；上一条完成的意图全局差值为假就停（攻方 K-h）；选段（强制选的那一段，
+///    或 greedy 先扫后选），选不出就停；签发前判 R 外非全空段的空洞 ≥ R 里要搬的单元数（攻方 K-g），不够就停；签发：先在分配器上排他 R
+///    （签发那次发布自己的新落点也不进 R），发一次只把意图写进 logged_ops 的发布（`c0` = 现行那一版的 txg，`segs0` = R 那块盘按记录口径的全空段数）；
+///    被空间拒就撤排他、停（意图没签成）。
+/// 2. 有活意图：暂停过的重新排他；R 里出现隔离槽（直接查常驻隔离集，钉住与隔离同时成立也算，丁 1 交判第 4 点）、区域扫描拒了 R、扫出隔离类，
+///    都发一次只删意图的发布丢掉它（攻方 K-c）；对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）、计划一批：要搬就发——这一批搬完 R 里每一条未释放的记录时
+///    同一次发布删意图（D8（核心索引结构） 已定项 5；发布在算好的账上核 R 按记录全空，核不过照这一批不删意图再发）；R 已按记录全空就发一次只删意图的发布；
+///    一个能搬的都没有而 R 不空就发一次只删意图的发布丢掉它、这一趟停。整理的发布被空间拒就暂停（撤 R 排他、意图留在盘上，攻方 K-k）、停。
+///
+/// 一次调用至多发两次（意图达成那一支核不过时重发一次），选段那一圈的上界是单元区的整段数。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、批大小、推到哪为止、上一条完成的意图"
+)]
+fn advance_compaction_by_one_publish<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    compaction: &mut CompactionOfThisMount,
+    current: &mut PoolVersion,
+    instance: InstanceGeneration,
+    batch_units: CompactionBatchUnits,
+    reach: CompactionRunReach,
+    last_completed_intent: &mut Option<CompactionIntent>,
+) -> Result<CompactionStep, RelocationBatchRefused> {
+    compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());
+    let version = current
+        .version_of_trees()
+        .expect("整理只在有树的一版上推：入口判过，整理的发布与删除续做都交回有树的一版")
+        .clone();
+    let live_on_disk = live_compaction_intent_of_the_entries(&version.logged_operations);
+    assert_eq!(
+        allocator
+            .compaction()
+            .live_intent()
+            .map(|live| live.intent),
+        live_on_disk.map(|(_, intent)| intent),
+        "分配器这一侧的活意图就是现行那一版 logged_ops 里那一条：会话签发、删意图时装上与撤掉，挂载按 logged_ops 装上"
+    );
+    let unit_area = {
+        let device_map = allocator
+            .devices
+            .first()
+            .expect("池里至少一块盘：两盘同槽，选段与判类按第一块盘的记录做（判决 G1 攻方 K-n）");
+        UnitAreaOfOneDevice {
+            device: device_map.device,
+            start: device_map.unit_area_start(),
+            slots: device_map.unit_area_slots(),
+        }
+    };
+    let read_of_the_version = version_as_the_compaction_reads_it(&version);
+    let quarantined_slots = compaction.quarantined_slots().clone();
+    let scan_of = |region: CompactionRegion, devices_to_read: &[(DeviceIdentity, Device)]| {
+        scan_region(
+            region,
+            &version.allocation_records,
+            &read_of_the_version.mapping_entries,
+            &read_of_the_version.pinned_slots,
+            &quarantined_slots,
+            |device, slot, bytes| devices_to_read.read(device, slot.to_device_offset(), bytes),
+        )
+    };
+    let Some((_issued_txg, intent)) = live_on_disk else {
+        if reach == CompactionRunReach::ContinueTheLiveIntentAtTheMount {
+            return Ok(CompactionStep::Stopped(
+                CompactionRunStop::NoLiveIntentLeftToContinue,
+            ));
+        }
+        if let Some(completed) = *last_completed_intent {
+            let empty_now =
+                empty_cluster_segments_by_the_records(&version.allocation_records, unit_area);
+            if !the_completed_intent_yielded(empty_now, completed.empty_cluster_segments_at_issue) {
+                return Ok(CompactionStep::Stopped(
+                    CompactionRunStop::NoYieldFromTheLastIntent {
+                        empty_cluster_segments_now: empty_now,
+                        empty_cluster_segments_at_issue: completed.empty_cluster_segments_at_issue,
+                    },
+                ));
+            }
+        }
+        // 钉住的槽与常驻隔离集里的槽都让一段没资格：带隔离槽的段签了也立刻要丢（攻方 K-c）。
+        let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> = read_of_the_version
+            .pinned_slots
+            .union(&quarantined_slots)
+            .copied()
+            .collect();
+        let chosen_scan: Option<RegionScan> = match allocator.compaction().switches.forced_region {
+            Some(forced_region) => {
+                let is_disqualified = allocator
+                    .cluster_segments()
+                    .contains(&forced_region.start_slot())
+                    || slots_that_disqualify_a_segment
+                        .range(forced_region.start_slot()..forced_region.end_slot())
+                        .next()
+                        .is_some()
+                    || compaction
+                        .segments_refused_by_the_region_scan
+                        .contains(forced_region.start_slot());
+                match (is_disqualified, scan_of(forced_region, &*devices)) {
+                    (false, Ok(scan)) if scan.movable_units().next().is_some() => Some(scan),
+                    (false, Ok(_)) | (true, _) => None,
+                    (false, Err(refusal)) => {
+                        compaction
+                            .segments_refused_by_the_region_scan
+                            .note_refusal(forced_region, refusal);
+                        None
+                    }
+                }
+            }
+            None => {
+                let mut region_scans: Vec<RegionScan> =
+                    region_scans_of_the_segments_without_an_unreleased_record(
+                        &version.allocation_records,
+                        unit_area,
+                    );
+                // 迭代上界：每答一次「先扫」下一次就多一段有结论（扫成了进 region_scans，被拒了进不合格段表），不超过单元区的整段数。
+                loop {
+                    match choose_region(
+                        &version.allocation_records,
+                        unit_area,
+                        allocator.cluster_segments(),
+                        None,
+                        &slots_that_disqualify_a_segment,
+                        &region_scans,
+                        &compaction.segments_refused_by_the_region_scan,
+                        RegionChoiceCriterion::Greedy,
+                    ) {
+                        RegionChoice::Chosen(region) => {
+                            break region_scans.into_iter().find(|scan| scan.region == region);
+                        }
+                        RegionChoice::ScanBeforeChoosing(region) => {
+                            match scan_of(region, &*devices) {
+                                Ok(scan) => region_scans.push(scan),
+                                Err(refusal) => compaction
+                                    .segments_refused_by_the_region_scan
+                                    .note_refusal(region, refusal),
+                            }
+                        }
+                        RegionChoice::NoEligibleSegment => break None,
+                    }
+                }
+            }
+        };
+        let Some(scan) = chosen_scan else {
+            return Ok(CompactionStep::Stopped(
+                CompactionRunStop::NoEligibleSegment,
+            ));
+        };
+        let units_to_move = u64::try_from(scan.movable_units().count()).expect("一段 64 槽");
+        let device_map = allocator
+            .devices
+            .first()
+            .expect("池里至少一块盘（上面取单元区时判过）");
+        let holes = holes_outside_the_region_in_segments_holding_user_data(
+            &version.allocation_records,
+            &read_of_the_version.slots_holding_user_data,
+            unit_area,
+            scan.region,
+            |slot| device_map.is_free(slot),
+        );
+        if !holes_admit_the_issue(holes, units_to_move) {
+            return Ok(CompactionStep::Stopped(
+                CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove {
+                    holes,
+                    units_to_move,
+                },
+            ));
+        }
+        let issued = CompactionIntent {
+            region: scan.region,
+            allocation_generation_at_issue: version.root.checkpoint_txg,
+            empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(
+                &version.allocation_records,
+                unit_area,
+            ),
+        };
+        // R 从签发那次发布起就排他：那次发布自己的新落点也不进 R。没发成就撤掉（盘上没有这条意图）。
+        allocator.compaction_mut().issue(issued);
+        return match publish_the_compaction_step_once(
+            parameters,
+            devices,
+            allocator,
+            &version,
+            &[],
+            CompactionIntentStep::Issue(issued),
+            instance,
+        ) {
+            Ok(published) => {
+                compaction.observations.intents_issued += 1;
+                *current = PoolVersion::of_a_version_of_trees(published.clone());
+                Ok(CompactionStep::Published {
+                    version: Box::new(published),
+                    kind: CompactionPublishKind::IntentIssued,
+                })
+            }
+            Err(refused) => {
+                allocator.compaction_mut().withdraw_the_live_intent();
+                if compaction_publish_refused_short_of_space(&refused) {
+                    Ok(CompactionStep::Stopped(
+                        CompactionRunStop::IssueRefusedShortOfSpace,
+                    ))
+                } else {
+                    Err(refused)
+                }
+            }
+        };
+    };
+    let is_the_region_excluded = allocator
+        .compaction()
+        .live_intent()
+        .is_some_and(|live| live.is_the_region_excluded_from_placement);
+    if !is_the_region_excluded {
+        allocator.compaction_mut().resume_the_live_intent();
+    }
+    let region = intent.region;
+    let scan = if region_holds_a_quarantined_slot(region, &quarantined_slots) {
+        None
+    } else {
+        match scan_of(region, &*devices) {
+            Ok(scan)
+                if !scan
+                    .allocations
+                    .iter()
+                    .any(|allocation| allocation.class == AllocationInRegion::Quarantined) =>
+            {
+                Some(scan)
+            }
+            Ok(_scan_with_a_quarantined_slot) => None,
+            Err(refusal) => {
+                compaction
+                    .segments_refused_by_the_region_scan
+                    .note_refusal(region, refusal);
+                None
+            }
+        }
+    };
+    // 这一次发布要搬哪几个、对意图做什么、发成了是哪一种。
+    let (planned, intent_step, kind) = match scan
+        .as_ref()
+        .map(|scan| (scan, plan_batch(scan, batch_units)))
+    {
+        // R 里有隔离槽、区域扫描拒了 R、扫出隔离类：丢意图（攻方 K-c）。
+        None => (
+            Vec::new(),
+            CompactionIntentStep::Drop,
+            CompactionPublishKind::IntentDropped,
+        ),
+        Some((scan, PlannedBatch::Relocate(planned))) => {
+            // 这一批搬完 R 里每一条未释放的记录：最后一批，同一次发布删意图（D8（核心索引结构） 已定项 5）。
+            if planned.len() == scan.allocations.len() {
+                (
+                    planned,
+                    CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
+                    CompactionPublishKind::BatchCompletingTheIntent,
+                )
+            } else {
+                (
+                    planned,
+                    CompactionIntentStep::CarryTheIntent,
+                    CompactionPublishKind::Batch,
+                )
+            }
+        }
+        Some((_scan, PlannedBatch::RegionEmptied)) => {
+            assert!(
+                region_is_empty_by_the_records(&version.allocation_records, region),
+                "扫描里一条未释放记录都没有，R 就按记录全空：扫描与停机谓词读的是同一份记录"
+            );
+            (
+                Vec::new(),
+                CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
+                CompactionPublishKind::IntentCompletedWithoutABatch,
+            )
+        }
+        Some((_scan, PlannedBatch::NothingMovableButTheRegionIsNotEmpty)) => (
+            Vec::new(),
+            CompactionIntentStep::Drop,
+            CompactionPublishKind::IntentDroppedWithNothingMovable,
+        ),
+    };
+    let first_try = publish_the_compaction_step_once(
+        parameters,
+        devices,
+        allocator,
+        &version,
+        &planned,
+        intent_step,
+        instance,
+    );
+    let (published, kind) = match first_try {
+        Ok(published) => (published, kind),
+        // 最后一批在算好的账上 R 还不按记录全空（只供测试的开关关掉排他、这一批的新落点落进了 R）：这一批不删意图，照搬。
+        Err(RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
+            ..
+        }) if kind == CompactionPublishKind::BatchCompletingTheIntent => {
+            match publish_the_compaction_step_once(
+                parameters,
+                devices,
+                allocator,
+                &version,
+                &planned,
+                CompactionIntentStep::CarryTheIntent,
+                instance,
+            ) {
+                Ok(published) => (published, CompactionPublishKind::Batch),
+                Err(refused) => return pause_or_hand_up(allocator, refused),
+            }
+        }
+        Err(refused) => return pause_or_hand_up(allocator, refused),
+    };
+    match kind {
+        CompactionPublishKind::BatchCompletingTheIntent
+        | CompactionPublishKind::IntentCompletedWithoutABatch => {
+            allocator.compaction_mut().withdraw_the_live_intent();
+            compaction.observations.intents_completed += 1;
+            *last_completed_intent = Some(intent);
+        }
+        CompactionPublishKind::IntentDropped
+        | CompactionPublishKind::IntentDroppedWithNothingMovable => {
+            allocator.compaction_mut().withdraw_the_live_intent();
+            compaction.observations.intents_dropped += 1;
+        }
+        CompactionPublishKind::Batch | CompactionPublishKind::IntentIssued => {}
+    }
+    *current = PoolVersion::of_a_version_of_trees(published.clone());
+    Ok(CompactionStep::Published {
+        version: Box::new(published),
+        kind,
+    })
+}
+
+/// 整理的一次发布没发成（活意图在盘上）：空间不够就暂停——撤 R 排他、意图留着（攻方 K-k），不推抬 F（判决 G8 g1）；别的原样交回。
+fn pause_or_hand_up(
+    allocator: &mut PoolAllocator,
+    refused: RelocationBatchRefused,
+) -> Result<CompactionStep, RelocationBatchRefused> {
+    if compaction_publish_refused_short_of_space(&refused) {
+        allocator.compaction_mut().pause_the_live_intent();
+        Ok(CompactionStep::Stopped(
+            CompactionRunStop::PausedShortOfSpace,
+        ))
+    } else {
+        Err(refused)
+    }
+}
+
 /// 现行那一版没有可改的文件：拒成 [`UserChangeRefused::NoFileVersionToChange`]，带上这次改动里已经落盘的（走空的那几步与推的抬 F）。
 fn no_file_version_to_change(published_in_this_change: PublishedInThisChange) -> UserChangeRefused {
     let PublishedInThisChange {
@@ -2311,6 +3336,10 @@ mod tests {
                 inode: InodeNumber(4_099),
                 write_time_seconds: 1_000_013,
             },
+            UserChange::CreateSnapshot,
+            UserChange::DestroySnapshot {
+                snapshot: TreeIdentifier(65_551),
+            },
         ];
         for change_handed_in in changes_handed_in {
             let kept = UserChangeKeptByTheRefusal::kept_from(change_handed_in);
diff --git a/crates/singlefs-core/src/records.rs b/crates/singlefs-core/src/records.rs
index 8b51a5dc..aa6233c4 100644
--- a/crates/singlefs-core/src/records.rs
+++ b/crates/singlefs-core/src/records.rs
@@ -430,6 +430,120 @@ pub const TREE_KIND_LINEAGE: u16 = 11;
 pub const TREE_KIND_LOGGED_OPERATIONS: u16 = 12;
 /// 退役树：码 13，接在 logged_ops 之后追加（D8（核心索引结构） 已定项 9；判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。
 pub const TREE_KIND_RETIREMENT: u16 = 13;
+/// 只读快照的 inode 根：码 14，接在退役树之后追加（D8（核心索引结构） 已定项 9 只追加不重用；判决
+/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1）。一个快照一条，字段取值：根指针 = 建快照那一版活头 inode 树的根；
+/// 诞生 txg = S.txg（建快照那次独立发布的 `checkpoint_txg`，同一判决 S2）；`previous_snapshot_txg` = 捕获那一刻活头 inode 树条目里的值；
+/// 头 ID = 活头 inode 树的树 ID；flags 位 0 是「销毁中」（同一判决 S4：删快照的意图写在这里，[`SnapshotDestructionMark`]）。
+/// 一张树表里可以有多条（每个快照一条），与 [`TREE_KIND_SNAPSHOT_EXTENT_ROOT`] 一一配对。
+pub const TREE_KIND_SNAPSHOT_INODE_ROOT: u16 = 14;
+/// 只读快照的 extent 根：码 15，接在码 14 之后追加（同上 S1）。根指针 = 建快照那一版活头 extent 树的根；头 ID = 与它配对的那条
+/// [`TREE_KIND_SNAPSHOT_INODE_ROOT`] 条目的树 ID；`previous_snapshot_txg` 写 0（0 = 不适用，D5（快照 / 空间记账机制） 已定项 9）。
+pub const TREE_KIND_SNAPSHOT_EXTENT_ROOT: u16 = 15;
+/// 第一版同时活着的只读快照至多几个（一张树表里码 14 的条数上界）：用户 2026-10-13 定「10」（判决
+/// `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1；单节点树表能装 34 个，取 10 是用户给的上界）。
+pub const SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION: usize = 10;
+
+/// 树表条目 flags 的位 0「销毁中」：只在码 14 条目上有含义（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4 与第七节岔路 2：
+/// 删快照的意图写在被删那个快照自己的码 14 条目上，只占这一位，预留字节不写）。别的位、别的种类上的这一位仍按 D8（核心索引结构） 已定项 8 ㊁拒收。
+pub const TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED: u16 = 1;
+
+/// 一条码 14 条目带不带「销毁中」的记号（flags 位 0，[`TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SnapshotDestructionMark {
+    NotBeingDestroyed,
+    BeingDestroyed,
+}
+
+impl SnapshotDestructionMark {
+    /// 这个记号在 flags 那 2 字节里的值。
+    #[must_use]
+    pub const fn flags(self) -> u16 {
+        match self {
+            SnapshotDestructionMark::NotBeingDestroyed => 0,
+            SnapshotDestructionMark::BeingDestroyed => {
+                TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED
+            }
+        }
+    }
+}
+
+/// deadlist 条目的 key 宽：头的树 ID 8 + 纪元 8 + 中央映射 key 27 = 43，value 空（D5（快照 / 空间记账机制） 已定项 12；
+/// 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3：deadlist 是第三棵多层码 2 树，种类 8）。
+pub const DEADLIST_ENTRY_KEY_BYTES: usize = 8 + 8 + 27;
+
+/// 中央映射 key 里出生 txg 那 8 字节的起点：类标签 1 + 出生树 8（[`mapping_key_for_data`]、[`mapping_key_for_node`]）。
+const MAPPING_KEY_BIRTH_TXG_OFFSET_IN_BYTES: usize = 1 + 8;
+
+/// 一把中央映射 key 里的出生 txg（D5（快照 / 空间记账机制） 已定项 19 的 `birth`：判决 S3「birth 从映射 key 的出生 txg 段取」）。
+/// key 不是 27 字节时交回 `None`（盘上读来的 key 由调用方当取不到处置：D5 已定项 14「判据的输入取不到时默认进 deadlist」）。
+#[must_use]
+pub fn birth_txg_of_a_mapping_key(mapping_key: &[u8]) -> Option<CheckpointTxg> {
+    if mapping_key.len() != usize::try_from(MAPPING_KEY_BYTES).expect("27") {
+        return None;
+    }
+    let mut reader = ByteReader::at(mapping_key, MAPPING_KEY_BIRTH_TXG_OFFSET_IN_BYTES);
+    Some(CheckpointTxg(reader.get_u64()))
+}
+
+/// 一条 deadlist 条目（D5（快照 / 空间记账机制） 已定项 12「丙-区间」）：key = 头的树 ID + 纪元 + 中央映射 key，value 空——key 即事实。
+/// 纪元 = 块被杀那一刻头的 `previous_snapshot_txg`；哪个快照的那一份按纪元落在哪一段区间认，建快照与删快照都不改条目的键。
+/// 不派生全序：盘上的序是逐字段按小端无符号整数比（`code_two_tree::CodeTwoKeyFieldWidths::DEADLIST`），映射 key 那一段按字节比与它不同序。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct DeadlistEntry {
+    pub head: TreeIdentifier,
+    pub epoch: CheckpointTxg,
+    pub mapping_key: Vec<u8>,
+}
+
+impl DeadlistEntry {
+    /// 盘上 43 字节（叶条目就是 key）。
+    ///
+    /// # Panics
+    /// `mapping_key` 不是 27 字节：条目的映射 key 取自一版的 `mapped_units`（写者现算的）或 [`Self::parse`] 判过宽度的盘上字节。
+    #[must_use]
+    pub fn key_bytes(&self) -> Vec<u8> {
+        let mut writer = ByteWriter::new(DEADLIST_ENTRY_KEY_BYTES);
+        writer.put_u64(self.head.0);
+        writer.put_u64(self.epoch.0);
+        assert_eq!(
+            self.mapping_key.len(),
+            usize::try_from(MAPPING_KEY_BYTES).expect("27"),
+            "deadlist 条目里的映射 key 是 27 字节：取自写者现算的 key 或读者判过宽度的字节"
+        );
+        writer.put(&self.mapping_key);
+        writer.assert_position(
+            u64::try_from(DEADLIST_ENTRY_KEY_BYTES).expect("43"),
+            "deadlist 条目",
+        );
+        writer.into_bytes()
+    }
+
+    /// 读者：条目宽窄于 43 交回 `None`（`parse_index_node` 只判了条目宽 ≥ key 宽）。
+    #[must_use]
+    pub fn parse(bytes: &[u8]) -> Option<Self> {
+        if bytes.len() < DEADLIST_ENTRY_KEY_BYTES {
+            return None;
+        }
+        let mut reader = ByteReader::at(bytes, 0);
+        let head = TreeIdentifier(reader.get_u64());
+        let epoch = CheckpointTxg(reader.get_u64());
+        Some(Self {
+            head,
+            epoch,
+            mapping_key: bytes[16..DEADLIST_ENTRY_KEY_BYTES].to_vec(),
+        })
+    }
+
+    /// 这条条目指的那个单元的出生 txg（映射 key 的出生 txg 段）。
+    ///
+    /// # Panics
+    /// 映射 key 不是 27 字节：同 [`Self::key_bytes`]。
+    #[must_use]
+    pub fn birth_txg(&self) -> CheckpointTxg {
+        birth_txg_of_a_mapping_key(&self.mapping_key)
+            .expect("deadlist 条目里的映射 key 是 27 字节：取自写者现算的 key 或读者判过宽度的字节")
+    }
+}
 
 /// 树表条目里根指针的起点：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2（`singlefs_format::TREE_TABLE_ENTRY_BYTES` 的字段表）。
 const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES: usize = 8 + 2 + 2 + 2;
@@ -440,25 +554,43 @@ pub struct TreeTableEntry {
     pub kind: u16,
     pub tree: TreeIdentifier,
     pub root: NodePointer,
+    /// D5（快照 / 空间记账机制） 已定项 9：写在两处——活头 inode 树那条条目（= 最近一个快照的 S.txg，还没打过快照写 0）与每条码 14
+    /// 快照条目（= 捕获那一刻活头的值）；其余条目写 0（0 = 不适用）。`parse` 照盘上读进来、`to_bytes` 照它写。
+    pub previous_snapshot_txg: CheckpointTxg,
     pub birth_txg: CheckpointTxg,
     /// D5（快照 / 空间记账机制） 已定项 9：这棵树归哪个可写头；无归属写 0。
     pub head_identifier: u64,
 }
 
 impl TreeTableEntry {
+    /// flags 写 0（不带「销毁中」的记号）。
     #[must_use]
     pub fn to_bytes(&self) -> Vec<u8> {
+        self.to_bytes_with_destruction_mark(SnapshotDestructionMark::NotBeingDestroyed)
+    }
+
+    /// flags 写 `mark` 的值（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4：删快照的意图只占码 14 条目 flags 位 0）。
+    ///
+    /// # Panics
+    /// 给不是码 14 的条目写「销毁中」：读者对别的种类上的这一位拒收（[`Self::parse_with_destruction_mark`]），写者只给被删的那条码 14 写。
+    #[must_use]
+    pub fn to_bytes_with_destruction_mark(&self, mark: SnapshotDestructionMark) -> Vec<u8> {
+        assert!(
+            mark == SnapshotDestructionMark::NotBeingDestroyed
+                || self.kind == TREE_KIND_SNAPSHOT_INODE_ROOT,
+            "「销毁中」只写在码 14 条目上（判决 S4）"
+        );
         let mut writer = ByteWriter::new(usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"));
         writer.put_u64(self.tree.0);
         writer.put_u16(u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"));
         writer.put_u16(self.kind);
-        writer.put_u16(0);
+        writer.put_u16(mark.flags());
         writer.assert_position(
             u64::try_from(TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES).expect("14"),
             "树表条目的根指针",
         );
         self.root.write_to(&mut writer);
-        writer.put_u64(0); // previous_snapshot_txg：0 = 不适用
+        writer.put_u64(self.previous_snapshot_txg.0);
         writer.put_u64(self.birth_txg.0);
         writer.put_u64(self.head_identifier);
         writer.skip(76);
@@ -476,8 +608,16 @@ impl TreeTableEntry {
     /// 读者：条目长度、flags 未知位（D8（核心索引结构） 已定项 8 ㊁：一律拒收）、预留 76 非零、根指针头部有一段不是第一版的取值
     /// （MAC / nonce 非 0、算法类型不是未加密、压缩算法码不认识、码 0 而压后长度非 0、extent 偏移非 0：D19（块指针的结构与宽度预算） 已定项 3、
     /// D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据），判该指针所在的结构——这条条目——损坏）都判不可用。
+    /// 码 14 条目 flags 位 0（销毁中）收下、丢掉记号；要记号的读者用 [`Self::parse_with_destruction_mark`]。
     #[must_use]
     pub fn parse(bytes: &[u8]) -> Option<Self> {
+        Self::parse_with_destruction_mark(bytes).map(|(entry, _mark)| entry)
+    }
+
+    /// 同 [`Self::parse`]，连同 flags 读出来的「销毁中」记号：flags 是 0 ⇒ `NotBeingDestroyed`；码 14 条目上 flags 恰是位 0 ⇒ `BeingDestroyed`；
+    /// 别的取值（别的位、别的种类上的位 0）照 D8（核心索引结构） 已定项 8 ㊁拒收。
+    #[must_use]
+    pub fn parse_with_destruction_mark(bytes: &[u8]) -> Option<(Self, SnapshotDestructionMark)> {
         if bytes.len() != usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200")
             || bytes[bytes.len() - 76..].iter().any(|byte| *byte != 0)
         {
@@ -489,24 +629,34 @@ impl TreeTableEntry {
             return None;
         }
         let kind = reader.get_u16();
-        if reader.get_u16() != 0 {
-            return None;
-        }
+        let mark = match reader.get_u16() {
+            0 => SnapshotDestructionMark::NotBeingDestroyed,
+            TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED
+                if kind == TREE_KIND_SNAPSHOT_INODE_ROOT =>
+            {
+                SnapshotDestructionMark::BeingDestroyed
+            }
+            _flags_this_reader_refuses => return None,
+        };
         let root =
             NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                 &mut reader,
             )
             .ok()?;
-        let _previous_snapshot_txg = reader.get_u64();
+        let previous_snapshot_txg = CheckpointTxg(reader.get_u64());
         let birth_txg = CheckpointTxg(reader.get_u64());
         let head_identifier = reader.get_u64();
-        Some(Self {
-            kind,
-            tree,
-            root,
-            birth_txg,
-            head_identifier,
-        })
+        Some((
+            Self {
+                kind,
+                tree,
+                root,
+                previous_snapshot_txg,
+                birth_txg,
+                head_identifier,
+            },
+            mark,
+        ))
     }
 }
 
@@ -759,6 +909,7 @@ mod tests {
             kind: TREE_KIND_EXTENT,
             tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
             root: pointer,
+            previous_snapshot_txg: CheckpointTxg(0),
             birth_txg: CheckpointTxg(3),
             head_identifier: TREE_IDENTIFIER_INODE,
         };
@@ -769,6 +920,140 @@ mod tests {
         );
     }
 
+    /// D5（快照 / 空间记账机制） 已定项 9：`previous_snapshot_txg` 写在条目偏移 100 那 8 字节（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 +
+    /// 根指针 86 之后、诞生 txg 之前），`to_bytes` 照结构里的值写、`parse` 照盘上读回来；两个 txg 字段各自独立，换一个不动另一个。
+    #[test]
+    fn previous_snapshot_txg_is_written_at_offset_one_hundred_and_read_back_apart_from_the_birth_txg(
+    ) {
+        let entry = TreeTableEntry {
+            kind: TREE_KIND_INODE,
+            tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
+            root: NodePointer::empty_root(),
+            previous_snapshot_txg: CheckpointTxg(0x0102_0304),
+            birth_txg: CheckpointTxg(0x0506),
+            head_identifier: TREE_IDENTIFIER_INODE,
+        };
+        let bytes = entry.to_bytes();
+        let field_at = |offset: usize| {
+            u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
+        };
+        assert_eq!(
+            field_at(100),
+            0x0102_0304,
+            "偏移 100：previous_snapshot_txg"
+        );
+        assert_eq!(field_at(108), 0x0506, "偏移 108：诞生 txg");
+        assert_eq!(
+            TreeTableEntry::parse(&bytes).map(|parsed| parsed.previous_snapshot_txg),
+            Some(CheckpointTxg(0x0102_0304)),
+            "读回来是写进去的值，不是 0"
+        );
+        assert_eq!(TreeTableEntry::parse(&bytes), Some(entry), "逐字段读回相等");
+    }
+
+    /// 删快照的意图只占码 14 条目 flags 位 0（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4）：写进偏移 12 那 2 字节、
+    /// 读回来记号相同、别的字段不动；码 14 上别的位、别的种类上的位 0 读者拒收。
+    #[test]
+    fn the_destruction_mark_sits_in_flags_bit_zero_of_a_snapshot_inode_root_entry_only() {
+        let snapshot_inode_root = TreeTableEntry {
+            kind: TREE_KIND_SNAPSHOT_INODE_ROOT,
+            tree: TreeIdentifier(65549),
+            root: NodePointer::empty_root(),
+            previous_snapshot_txg: CheckpointTxg(4),
+            birth_txg: CheckpointTxg(9),
+            head_identifier: TREE_IDENTIFIER_INODE,
+        };
+        let marked = snapshot_inode_root
+            .to_bytes_with_destruction_mark(SnapshotDestructionMark::BeingDestroyed);
+        assert_eq!(
+            u16::from_le_bytes([marked[12], marked[13]]),
+            TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED,
+            "偏移 12：flags 位 0"
+        );
+        assert_eq!(
+            TreeTableEntry::parse_with_destruction_mark(&marked),
+            Some((snapshot_inode_root, SnapshotDestructionMark::BeingDestroyed)),
+            "读回来带「销毁中」，别的字段与写进去的相同"
+        );
+        assert_eq!(
+            TreeTableEntry::parse_with_destruction_mark(&snapshot_inode_root.to_bytes()),
+            Some((
+                snapshot_inode_root,
+                SnapshotDestructionMark::NotBeingDestroyed
+            )),
+            "不带记号的那一份读回来不带"
+        );
+        let mut another_bit = marked.clone();
+        another_bit[12] = 2;
+        assert_eq!(
+            TreeTableEntry::parse(&another_bit),
+            None,
+            "码 14 上 flags 别的位：拒收"
+        );
+        let snapshot_extent_root = TreeTableEntry {
+            kind: TREE_KIND_SNAPSHOT_EXTENT_ROOT,
+            tree: TreeIdentifier(65550),
+            root: NodePointer::empty_root(),
+            previous_snapshot_txg: CheckpointTxg(0),
+            birth_txg: CheckpointTxg(9),
+            head_identifier: 65549,
+        };
+        let mut bit_zero_on_another_kind = snapshot_extent_root.to_bytes();
+        bit_zero_on_another_kind[12] = 1;
+        assert_eq!(
+            TreeTableEntry::parse(&bit_zero_on_another_kind),
+            None,
+            "码 15 上 flags 位 0：拒收"
+        );
+    }
+
+    /// deadlist 条目 43 字节：头的树 ID 8 + 纪元 8 + 中央映射 key 27（D5（快照 / 空间记账机制） 已定项 12），读回来逐字段相等；
+    /// 出生 txg 取映射 key 偏移 9 那 8 字节（判决 S3），不取纪元。
+    #[test]
+    fn a_deadlist_entry_is_the_head_the_epoch_and_the_mapping_key_and_reads_back() {
+        let mapping_key = mapping_key_for_node(
+            2,
+            NodePointer {
+                head: PointerHead {
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                    birth_txg: CheckpointTxg(7),
+                },
+                locations: NodePointer::empty_root().locations,
+                instance: InstanceGeneration(1),
+                birth_sequence: BirthSequence(3),
+            },
+        );
+        let entry = DeadlistEntry {
+            head: TreeIdentifier(TREE_IDENTIFIER_INODE),
+            epoch: CheckpointTxg(11),
+            mapping_key: mapping_key.clone(),
+        };
+        let bytes = entry.key_bytes();
+        assert_eq!(bytes.len(), DEADLIST_ENTRY_KEY_BYTES, "43 字节");
+        assert_eq!(
+            &bytes[..8],
+            &TREE_IDENTIFIER_INODE.to_le_bytes(),
+            "偏移 0：头的树 ID"
+        );
+        assert_eq!(&bytes[8..16], &11u64.to_le_bytes(), "偏移 8：纪元");
+        assert_eq!(
+            &bytes[16..],
+            mapping_key.as_slice(),
+            "偏移 16：中央映射 key"
+        );
+        assert_eq!(
+            DeadlistEntry::parse(&bytes),
+            Some(entry.clone()),
+            "读回来逐字段相等"
+        );
+        assert_eq!(
+            entry.birth_txg(),
+            CheckpointTxg(7),
+            "出生 txg 取映射 key 那一段"
+        );
+        assert_eq!(DeadlistEntry::parse(&bytes[..42]), None, "窄于 43：解不开");
+    }
+
     /// 墓碑记录五个字段落在 `.claude/kb/layout/03-third-txn.md`「二、墓碑记录」那张表的偏移上，读回来逐字段相等。
     #[test]
     fn tombstone_record_fields_sit_at_the_offsets_of_the_layout_table_and_read_back() {
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 0ea26180..1617be28 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -41,7 +41,7 @@ use crate::allocator::{
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
 use crate::code_two_tree::{
-    read_code_two_tree, CodeTwoTreeHeaderJudgement, CodeTwoTreeReadFromDisk,
+    read_code_two_tree, CodeTwoTreeHeaderJudgement, CodeTwoTreeReadFromDisk, CodeTwoTreeVersion,
 };
 use crate::container_tree::{
     ContainerTreeNodePosition, ContainerTreeShape, RecordContainer, IDENTITY_OF_A_CODE_TWO_CHILD,
@@ -61,6 +61,10 @@ use crate::journal::{
     JournalRecordPlaceInPublish, FIRST_JOURNAL_COUNTER,
 };
 use crate::lineage_tree::LineageContainerIndexInTree;
+use crate::logged_operations::{
+    logged_operation_entries_of_the_root_entries, LoggedOperationEntry,
+    LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES,
+};
 use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
 use crate::pointer::{
     location_entries_ascend_by_device, DataPointer, LocationEntry, NodePointer,
@@ -70,11 +74,13 @@ use crate::records::{
     mapping_key_for_data, mapping_key_for_node, parse_inode_internal_entry_or_refusal,
     parse_mapping_entry, AccountingEntry, InodeRecord, LineageRecord, PointerCarryingEntryRefusal,
     RetirementRecord, TombstoneRecord, TreeTableEntry, NLINK_OF_AN_INODE_BEING_DELETED,
-    STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
-    TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE,
-    TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT,
+    SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION, STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING,
+    TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT,
+    TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS,
+    TREE_KIND_RETIREMENT, TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT,
     TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
 };
+use crate::records::{DeadlistEntry, SnapshotDestructionMark, DEADLIST_ENTRY_KEY_BYTES};
 use crate::retirement_tree::RetirementContainerIndexInTree;
 use crate::root_record::RootRecord;
 use crate::root_ring::{region_start, slot_offset, RootRingSlot, RootRingSlotsPerRegionOutOfRange};
@@ -87,7 +93,8 @@ use crate::system_configuration::{
 use crate::tombstone_tree::TombstoneContainerIndexInTree;
 use crate::tombstone_tree::CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES;
 use crate::transaction::{
-    role_of_allocation_record_tree_node, role_of_extent_upper_node, ContainerTreeIndexNodesVersion,
+    role_of_allocation_record_tree_node, role_of_extent_upper_node,
+    take_the_mapping_positions_over_the_location_hints, ContainerTreeIndexNodesVersion,
     FileVersionTreeIdentifiers, InodeLeafContainerVersion, LineageContainerVersion,
     MultiLevelCodeTwoTree, PublishedUnit, RetirementContainerVersion, TombstoneContainerVersion,
     TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
@@ -2322,13 +2329,40 @@ pub fn tree_table_entry_count<Reader: PoolReader + ?Sized>(
 /// 树表里同一种树出现两条时，读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
 pub const TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE: &str = "树表里同一种树有两条";
 
+/// 树表里码 14（快照 inode 根）与码 15（快照 extent 根）的条数不相等时，读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
+pub const TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER: &str =
+    "树表里快照 inode 根与快照 extent 根的条数不相等";
+
+/// 树表里有一条码 15（快照 extent 根）的头 ID 不是任何一条码 14（快照 inode 根）的树 ID，或它指的那条码 14 前面已经有一条码 15 配上了：
+/// 读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
+pub const TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY: &str =
+    "树表里有一条快照 extent 根配不上一条只属于它的快照 inode 根";
+
+/// 树表里码 14（快照 inode 根）多于 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条：读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，
+/// `what` 取这一句。
+pub const TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS: &str =
+    "树表里的只读快照多于第一版允许的个数";
+
+/// 树表里有一条码 14（快照 inode 根）的头 ID 不等于同一张树表里活头 inode 树（码 2）条目的树 ID（表里没有码 2 条目时也报这一句）：
+/// 读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
+pub const TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE: &str =
+    "树表里有一条快照 inode 根的头 ID 不是活头 inode 树的树 ID";
+
 /// 树表的条目读不下去是哪一种（[`tree_table_entries_each_kind_at_most_once`] 交回）。封闭集合，`match` 不写通配臂。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub(crate) enum TreeTableEntriesRefusal {
     /// 一条条目解不开（条目长度、flags 未知位、预留 76 非零、根指针头部 extent 偏移非 0，`TreeTableEntry::parse`）。
     EntryMalformed,
-    /// 同一种树有两条。
+    /// 同一种树有两条（码 14、15 两种快照条目除外）。
     OneKindOfTreeAppearsTwice,
+    /// 码 14 与码 15 的条数不相等。
+    SnapshotInodeAndExtentRootEntryCountsDiffer,
+    /// 一条码 15 的头 ID 不是任何一条码 14 的树 ID，或它指的那条码 14 已经被前面一条码 15 配上了。
+    SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,
+    /// 码 14 多于 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条。
+    MoreSnapshotsThanTheFirstVersionAllows,
+    /// 一条码 14 的头 ID 不等于这张树表里码 2 条目的树 ID（或表里没有码 2 条目）。
+    SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree,
 }
 
 impl From<TreeTableEntriesRefusal> for RecoveryFailure {
@@ -2340,17 +2374,110 @@ impl From<TreeTableEntriesRefusal> for RecoveryFailure {
             TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice => RecoveryFailure::UnitMalformed {
                 what: TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE,
             },
+            TreeTableEntriesRefusal::SnapshotInodeAndExtentRootEntryCountsDiffer => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER,
+                }
+            }
+            TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY,
+                }
+            }
+            TreeTableEntriesRefusal::MoreSnapshotsThanTheFirstVersionAllows => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS,
+                }
+            }
+            TreeTableEntriesRefusal::SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE,
+                }
+            }
         }
     }
 }
 
+/// 一个只读快照在树表里的那一对条目：码 14（快照 inode 根）与头 ID 指着它的那条码 15（快照 extent 根），字段取值见
+/// [`TREE_KIND_SNAPSHOT_INODE_ROOT`] 与 [`TREE_KIND_SNAPSHOT_EXTENT_ROOT`]。重建只把它们读出来，不解它们指着的根。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct SnapshotTreeTableEntries {
+    pub inode_root: TreeTableEntry,
+    pub extent_root: TreeTableEntry,
+}
+
+/// 树表里的快照条目（码 14、15）配成对，按码 14 的盘上次序（即树 ID 升序，I-9.16 ①）交回，并判四样（判决
+/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1 与第七节岔路 1）：两种条数相等；每条码 15 的头 ID 恰是一条码 14 的树 ID、
+/// 且那条码 14 只被这一条码 15 配上（两样合起来是一一配对）；码 14 至多 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条；每条码 14 的头 ID
+/// 等于这张树表里码 2（活头 inode 树）条目的树 ID。没有快照条目的树表交空表，一样都不判。
+///
+/// # Errors
+/// 按上面的次序第一样不成立的那一个：`SnapshotInodeAndExtentRootEntryCountsDiffer`、`SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry`、
+/// `MoreSnapshotsThanTheFirstVersionAllows`、`SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree`。
+pub(crate) fn snapshot_tree_table_entries_paired_one_to_one(
+    tree_table_entries: &[TreeTableEntry],
+) -> Result<Vec<SnapshotTreeTableEntries>, TreeTableEntriesRefusal> {
+    let snapshot_inode_root_entries: Vec<&TreeTableEntry> = tree_table_entries
+        .iter()
+        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
+        .collect();
+    let snapshot_extent_root_entries: Vec<&TreeTableEntry> = tree_table_entries
+        .iter()
+        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT)
+        .collect();
+    if snapshot_inode_root_entries.len() != snapshot_extent_root_entries.len() {
+        return Err(TreeTableEntriesRefusal::SnapshotInodeAndExtentRootEntryCountsDiffer);
+    }
+    // 每条码 14 配上的那条码 15，下标与 `snapshot_inode_root_entries` 一一对应。
+    let mut extent_root_paired_with: Vec<Option<TreeTableEntry>> =
+        vec![None; snapshot_inode_root_entries.len()];
+    for extent_root_entry in &snapshot_extent_root_entries {
+        // 头 ID 指着它、又还没被前面哪条码 15 配上的那条码 14。
+        let paired_inode_root_position = snapshot_inode_root_entries
+            .iter()
+            .zip(&extent_root_paired_with)
+            .position(|(inode_root_entry, extent_root_already_paired)| {
+                inode_root_entry.tree.0 == extent_root_entry.head_identifier
+                    && extent_root_already_paired.is_none()
+            })
+            .ok_or(
+                TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,
+            )?;
+        extent_root_paired_with[paired_inode_root_position] = Some(**extent_root_entry);
+    }
+    if snapshot_inode_root_entries.len() > SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
+        return Err(TreeTableEntriesRefusal::MoreSnapshotsThanTheFirstVersionAllows);
+    }
+    let live_inode_tree = tree_table_entries
+        .iter()
+        .find(|entry| entry.kind == TREE_KIND_INODE)
+        .map(|entry| entry.tree);
+    if snapshot_inode_root_entries.iter().any(|inode_root_entry| {
+        Some(TreeIdentifier(inode_root_entry.head_identifier)) != live_inode_tree
+    }) {
+        return Err(TreeTableEntriesRefusal::SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree);
+    }
+    Ok(snapshot_inode_root_entries
+        .into_iter()
+        .zip(extent_root_paired_with)
+        .map(|(inode_root_entry, extent_root_entry)| SnapshotTreeTableEntries {
+            inode_root: *inode_root_entry,
+            extent_root: extent_root_entry.expect(
+                "两种条数相等、每条码 15 配上一条还没配过的码 14（上面两判）：鸽巢下每条码 14 都配上了",
+            ),
+        })
+        .collect())
+}
+
 /// 树表单元的条目逐条解开，并判每一种树至多一条（审阅第 27 条）。写者每种树只写一条；同一种出现两条时取哪一条条款没有写，
 /// 取首条、取末条、取首条非空各有一个读者照着读过，同一张树表在几处读出不同的树 ⇒ 判损坏，一条都不取。
+/// 码 14、15 两种快照条目例外：每个快照一对，可以有多条，改判 [`snapshot_tree_table_entries_paired_one_to_one`] 那四样。
 /// 读树表取条目的每一处都经这一处：[`rebuild_version`]、[`allocation_records_under_root`]、[`walk_to_file`]、
 /// [`user_visible_tree_root_pointers`]、`mounted_read::open_pool_for_read`。
 ///
 /// # Errors
-/// 按条目次序第一处读不下去的：解不开 ⇒ `EntryMalformed`；它的种类前面已经有一条 ⇒ `OneKindOfTreeAppearsTwice`。
+/// 按条目次序第一处读不下去的：解不开 ⇒ `EntryMalformed`；它的种类（码 14、15 除外）前面已经有一条 ⇒ `OneKindOfTreeAppearsTwice`；
+/// 条目都过了之后快照条目配不成对 ⇒ [`snapshot_tree_table_entries_paired_one_to_one`] 的那四个成员。
 pub(crate) fn tree_table_entries_each_kind_at_most_once(
     entry_bytes_of_the_tree_table: &[Vec<u8>],
 ) -> Result<Vec<TreeTableEntry>, TreeTableEntriesRefusal> {
@@ -2359,11 +2486,18 @@ pub(crate) fn tree_table_entries_each_kind_at_most_once(
     for entry_bytes in entry_bytes_of_the_tree_table {
         let entry =
             TreeTableEntry::parse(entry_bytes).ok_or(TreeTableEntriesRefusal::EntryMalformed)?;
+        let is_a_snapshot_entry = entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
+            || entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT;
+        if is_a_snapshot_entry {
+            entries.push(entry);
+            continue;
+        }
         if !kinds_seen.insert(entry.kind) {
             return Err(TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice);
         }
         entries.push(entry);
     }
+    snapshot_tree_table_entries_paired_one_to_one(&entries)?;
     Ok(entries)
 }
 
@@ -2518,12 +2652,42 @@ impl From<RecoveryFailure> for RebuildVersionFailure {
 /// 记账行里没有 inode 号水位那一行 ⇒ `InodeNumberWatermarkRowMissingFromTheAccountingTree`。
 ///
 /// 单元区从 `unit_area_start` 起（择到的系统配置按环长现算的那个）：这一版的分配记录落不落在单元区里按它判。
+/// 树表里的只读快照条目（码 14、15）照盘上原样在交回那一版的 `tree_table_entries` 里（发布路径接着这一版发时照它们续上，
+/// `transaction` 装树表那一步）；要它们配成对的用 [`rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at`]。
 pub fn rebuild_version_in_the_unit_area_starting_at(
     reader: &dyn PoolReader,
     unit_area_start: UnitAreaStart,
     root: &RootRecord,
     record_standing_for_root: Option<JournalRecord>,
 ) -> Result<RebuiltVersion, RebuildVersionFailure> {
+    rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
+        reader,
+        unit_area_start,
+        root,
+        record_standing_for_root,
+    )
+    .map(|rebuilt| rebuilt.version)
+}
+
+/// 重建出来的一版，连同它树表里的只读快照条目（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RebuiltVersionWithItsSnapshotEntries {
+    pub version: RebuiltVersion,
+    /// 码 14 与配对的码 15，按码 14 的树 ID 升序；树表里没有快照条目（含树表 0 条）时为空。只读出条目，不解它们指着的根。
+    pub snapshot_entries: Vec<SnapshotTreeTableEntries>,
+}
+
+/// 同 [`rebuild_version_in_the_unit_area_starting_at`]（判全、报错都与它相同），另把树表里码 14、15 的条目配成对交出来
+/// （[`snapshot_tree_table_entries_paired_one_to_one`]，配不成对的树表在 [`tree_table_entries_each_kind_at_most_once`] 那一步就拒了）。
+///
+/// # Errors
+/// 同 [`rebuild_version_in_the_unit_area_starting_at`]。
+pub fn rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
+    reader: &dyn PoolReader,
+    unit_area_start: UnitAreaStart,
+    root: &RootRecord,
+    record_standing_for_root: Option<JournalRecord>,
+) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
     let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
     let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
     // 树表 0 条的一版也要判：可写挂载接着它发的零单元发布照抄根记录里的实例表与树表指针。
@@ -2537,13 +2701,20 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         })?;
     tree_table_key_width_is_the_tree_tables(&tree_table)?;
     if tree_table.entries.is_empty() {
-        return Ok(RebuiltVersion::WithoutFile);
+        return Ok(RebuiltVersionWithItsSnapshotEntries {
+            version: RebuiltVersion::WithoutFile,
+            snapshot_entries: Vec::new(),
+        });
     }
     let record =
         record_standing_for_root.ok_or(RebuildVersionFailure::NoRecordStandingForFileVersion)?;
     let record_bytes = record.to_bytes();
     let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)
         .map_err(RecoveryFailure::from)?;
+    let snapshot_entries = snapshot_tree_table_entries_paired_one_to_one(&tree_table_entries)
+        .expect(
+            "tree_table_entries_each_kind_at_most_once 交回之前对同一串条目判过快照条目一一配对",
+        );
     // 可写挂载接着这一版发布，写者按种类装树表条目、断言树 ID 升序：盘上的次序与种类在这里先判（I-9.16），
     // 走得到那条断言的镜像在任何写之前拒。
     tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(&tree_table_entries)?;
@@ -2793,6 +2964,45 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             inode_numbers_of_the_deleted_inodes_root_entries(&node.entries)
         }
     };
+    // logged_ops 树（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）：根是码 2 层级 0，条目 44 字节，逐条照字段表解
+    // （R 不对齐、槽数不是 64、补零非零、种类码保留、整理意图多于一条都拒：盘上读来的坏条目当损坏处置，可写挂载拒）。
+    let logged_operations_root_pointer = pointer_of(TREE_KIND_LOGGED_OPERATIONS)?;
+    let logged_operations_root_bytes =
+        if logged_operations_root_pointer == NodePointer::empty_root() {
+            None
+        } else {
+            Some(read_mapped_tree_node_via_hint_then_central_mapping(
+                reader,
+                &logged_operations_root_pointer,
+                MappedTreeNodeClass::IndexNode,
+                &central_mapping_locations_of_key,
+                &mut stale_location_hint_hops_not_exposed_by_this_reader,
+            )?)
+        };
+    let logged_operations: Vec<LoggedOperationEntry> = match &logged_operations_root_bytes {
+        None => Vec::new(),
+        Some(bytes) => {
+            let node = tree_root_checked_against_its_pointer(
+                bytes,
+                tree_identifiers.logged_operations,
+                LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES,
+                &logged_operations_root_pointer,
+                root,
+                expected_filesystem_identifier,
+            )?;
+            if node.level != 0 || node.entries.is_empty() {
+                return Err(RecoveryFailure::UnitMalformed {
+                    what: LOGGED_OPERATIONS_ROOT_IS_NOT_A_NON_EMPTY_LEVEL_ZERO_NODE,
+                }
+                .into());
+            }
+            logged_operation_entries_of_the_root_entries(&node.entries).map_err(|_refusal| {
+                RecoveryFailure::UnitMalformed {
+                    what: LOGGED_OPERATIONS_ROOT_ENTRIES_REFUSED_BY_THE_FIELD_TABLE,
+                }
+            })?
+        }
+    };
     // 墓碑树与谱系树（D18（块里携带什么信息） 已定项 24 第 1、5 条）：根恒为码 2，条目照 inode 树内部节点的 120 字节形态；根满长高之后
     // 根之下是类型段 0 的码 2 节点，层级 1 的节点的孩子是打包记录容器（墓碑类型 1、谱系类型 5）。两棵树读法相同，只差下面这几样。
     let mut read_container_tree_node = |pointer: &NodePointer, class: MappedTreeNodeClass| {
@@ -2941,6 +3151,44 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
     }) {
         return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());
     }
+    // deadlist 树（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3：第三棵多层码 2 树）：树表条目根指针非零时整棵读回来，
+    // 同记账树（节点进映射、提示读不出经映射回退、照冷走读判全）；零根时一条条目都没有。接着这一版发布要按它规划、删快照的批要从它取条目。
+    let deadlist_pointer = pointer_of(TREE_KIND_DEADLIST)?;
+    let deadlist_tree = if deadlist_pointer == NodePointer::empty_root() {
+        CodeTwoTreeReadFromDisk {
+            version: CodeTwoTreeVersion::default(),
+            node_bytes: Vec::new(),
+            leaf_entries_in_key_order: Vec::new(),
+        }
+    } else {
+        read_code_two_tree(
+            &deadlist_pointer,
+            &MultiLevelCodeTwoTree::Deadlist.read_expectation(tree_identifiers.deadlist),
+            CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
+            root,
+            expected_filesystem_identifier,
+            &mut |pointer: &NodePointer| {
+                read_mapped_tree_node_via_hint_then_central_mapping(
+                    reader,
+                    pointer,
+                    MappedTreeNodeClass::IndexNode,
+                    &central_mapping_locations_of_key,
+                    &mut stale_location_hint_hops_not_exposed_by_this_reader,
+                )
+            },
+        )?
+    };
+    let deadlist_entries = deadlist_entries_parsed(&deadlist_tree.leaf_entries_in_key_order)?;
+    // 删快照的意图写在被删那条码 14 自己身上（判决 S4，flags 位 0）：读回来，接着这一版的发布照它续写记号、走批。
+    let snapshots_being_destroyed: Vec<TreeIdentifier> = tree_table
+        .entries
+        .iter()
+        .filter_map(|entry_bytes| {
+            TreeTableEntry::parse_with_destruction_mark(entry_bytes.as_slice())
+        })
+        .filter(|(_, mark)| *mark == SnapshotDestructionMark::BeingDestroyed)
+        .map(|(entry, _)| entry.tree)
+        .collect();
     // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。
     let mapping_tree = central_mapping_root.into_tree()?;
     mapping_entry_count_is_one_per_mapped_unit(
@@ -2949,9 +3197,12 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             extent_tree_nodes: first_file_extents.nodes_in_bump_order.len(),
             allocation_record_tree_nodes: allocation_tree_read.nodes.len(),
             accounting_tree_nodes: accounting_tree.version.node_count(),
+            deadlist_tree_nodes: deadlist_tree.version.node_count(),
+            mapping_entries_kept_for_the_snapshots: deadlist_entries.len(),
             inode_leaf_containers: inode_root_node.entries.len(),
             data_units: data_pointers.len(),
             deletion_tree_nodes: usize::from(deleted_inodes_root_bytes.is_some())
+                + usize::from(logged_operations_root_bytes.is_some())
                 + tombstone_tree_read
                     .as_ref()
                     .map_or(0, ContainerTreeRead::mapped_units)
@@ -2965,15 +3216,39 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
     )?;
     let mut mapping_keys: Vec<Vec<u8>> =
         Vec::with_capacity(mapping_tree.leaf_entries_in_key_order.len());
+    // 被活快照钉住的单元留存的映射条目（判决 S3 的 α）：每条 deadlist 条目的映射 key 一条，按映射 key 的序（叶条目本来的次序）。
+    let deadlist_mapping_keys: BTreeSet<&[u8]> = deadlist_entries
+        .iter()
+        .map(|entry| entry.mapping_key.as_slice())
+        .collect();
+    let mut mapping_entries_kept_for_the_snapshots: Vec<(Vec<u8>, [LocationEntry; 2])> =
+        Vec::with_capacity(deadlist_entries.len());
+    // 每条映射条目的位置（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「重建时映射条目的位置覆盖提示」）：
+    // 这一版装完之后，指向进映射单元的每一处指针换成它（`transaction::take_the_mapping_positions_over_the_location_hints`）。
+    let mut mapping_position_of_key: BTreeMap<Vec<u8>, [LocationEntry; 2]> = BTreeMap::new();
     for entry in &mapping_tree.leaf_entries_in_key_order {
-        let (key, _locations) =
+        let (key, locations) =
             parse_mapping_entry(entry).ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
                 what: "映射条目",
                 entry_bytes: entry.len(),
                 field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
             })?;
+        if deadlist_mapping_keys.contains(key.as_slice()) {
+            mapping_entries_kept_for_the_snapshots.push((key.clone(), locations));
+        }
+        mapping_position_of_key.insert(key.clone(), locations);
         mapping_keys.push(key);
     }
+    // deadlist 条目按 key 在中央映射树里查不到：C300（deadlist 条目按 key 查映射查不到没有条款）没有条款，第一版拒这一版（删快照的批要从
+    // 那条映射条目取落点，取不到就放不了），在交给发布路径之前。
+    if mapping_entries_kept_for_the_snapshots.len() != deadlist_mapping_keys.len()
+        || deadlist_mapping_keys.len() != deadlist_entries.len()
+    {
+        return Err(RecoveryFailure::UnitMalformed {
+            what: "deadlist 条目的映射 key 在中央映射树里查不到，或两条 deadlist 条目同一把映射 key（C300 没有条款）",
+        }
+        .into());
+    }
     let data_role_of_position = |position: usize| {
         TransactionUnit::Data(DataUnitIndexInFile(
             u64::try_from(position).expect("单元序号"),
@@ -3023,41 +3298,63 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
         ));
     }
-    // 删除那两棵树接在记账树后面（按树 ID），与发布路径的 `mapped_units` 同序。
+    // deadlist 树的每个节点接在记账树后面（按树 ID），与发布路径的 `mapped_units` 同序。
+    let deadlist_shape = &deadlist_tree.version.shape;
+    for (node, pointer) in deadlist_shape
+        .nodes()
+        .iter()
+        .zip(&deadlist_tree.version.pointers)
+    {
+        mapped_units.push((
+            MultiLevelCodeTwoTree::Deadlist.role_of_node(node.position, deadlist_shape),
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
+        ));
+    }
+    // 删除那两棵树接在 deadlist 树后面（按树 ID），与发布路径的 `mapped_units` 同序。
     if deleted_inodes_root_bytes.is_some() {
         mapped_units.push((
             TransactionUnit::DeletedInodesRoot,
             mapping_key_for_node(UNIT_CLASS_INDEX_NODE, deleted_inodes_root_pointer),
         ));
     }
-    // 墓碑树、谱系树：每片容器、根之下的码 2 节点（bump 次序）、根，与发布路径的 `mapped_units` 同序。
-    for (tree_read, roles) in [
-        (
-            tombstone_tree_read
-                .as_ref()
-                .map(ContainerTreeRead::units_in_bump_order),
-            CONTAINER_TREE_ROLES_OF_THE_TOMBSTONE_TREE,
-        ),
-        (
-            lineage_tree_read
-                .as_ref()
-                .map(ContainerTreeRead::units_in_bump_order),
-            CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
-        ),
-        (
-            retirement_tree_read
-                .as_ref()
-                .map(ContainerTreeRead::units_in_bump_order),
-            CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE,
-        ),
-    ] {
-        for (unit_of_the_tree, pointer, _) in tree_read.unwrap_or_default() {
-            mapped_units.push((
-                roles.of(unit_of_the_tree),
-                mapping_key_for_node(unit_of_the_tree.unit_class(), pointer),
-            ));
-        }
+    // 墓碑树、谱系树、logged_ops 根、退役树（按树 ID）：容器树每片容器、根之下的码 2 节点（bump 次序）、根，与发布路径的 `mapped_units` 同序。
+    let mapped_units_of_a_container_tree =
+        |tree_read: Option<Vec<(UnitOfAContainerTree, NodePointer, Vec<u8>)>>,
+         roles: ContainerTreeRolesOfOneTree| {
+            tree_read
+                .unwrap_or_default()
+                .into_iter()
+                .map(move |(unit_of_the_tree, pointer, _)| {
+                    (
+                        roles.of(unit_of_the_tree),
+                        mapping_key_for_node(unit_of_the_tree.unit_class(), pointer),
+                    )
+                })
+        };
+    mapped_units.extend(mapped_units_of_a_container_tree(
+        tombstone_tree_read
+            .as_ref()
+            .map(ContainerTreeRead::units_in_bump_order),
+        CONTAINER_TREE_ROLES_OF_THE_TOMBSTONE_TREE,
+    ));
+    mapped_units.extend(mapped_units_of_a_container_tree(
+        lineage_tree_read
+            .as_ref()
+            .map(ContainerTreeRead::units_in_bump_order),
+        CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
+    ));
+    if logged_operations_root_bytes.is_some() {
+        mapped_units.push((
+            TransactionUnit::LoggedOperationsRoot,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, logged_operations_root_pointer),
+        ));
     }
+    mapped_units.extend(mapped_units_of_a_container_tree(
+        retirement_tree_read
+            .as_ref()
+            .map(ContainerTreeRead::units_in_bump_order),
+        CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE,
+    ));
     let unit =
         |identity: TransactionUnit, locations: &[LocationEntry; 2], bytes: Vec<u8>| PublishedUnit {
             slot: locations[0].slot,
@@ -3127,20 +3424,34 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
                 .map(ContainerTreeRead::units_in_bump_order),
             CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
         ),
-        (
-            retirement_tree_read
-                .as_ref()
-                .map(ContainerTreeRead::units_in_bump_order),
-            CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE,
-        ),
     ] {
         for (unit_of_the_tree, pointer, bytes) in tree_read.unwrap_or_default() {
             deletion_tree_units.push(unit(roles.of(unit_of_the_tree), &pointer.locations, bytes));
         }
     }
+    // logged_ops 根按树 ID 排在谱系树之后、退役树之前（与发布路径装出来的 `units` 同序）。
+    if let Some(bytes) = logged_operations_root_bytes {
+        deletion_tree_units.push(unit(
+            TransactionUnit::LoggedOperationsRoot,
+            &logged_operations_root_pointer.locations,
+            bytes,
+        ));
+    }
+    for (unit_of_the_tree, pointer, bytes) in retirement_tree_read
+        .as_ref()
+        .map(ContainerTreeRead::units_in_bump_order)
+        .unwrap_or_default()
+    {
+        deletion_tree_units.push(unit(
+            CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE.of(unit_of_the_tree),
+            &pointer.locations,
+            bytes,
+        ));
+    }
     let mut deletion_tree_units_not_yet_placed = Some(deletion_tree_units);
     for (tree, read) in [
         (MultiLevelCodeTwoTree::Accounting, &accounting_tree),
+        (MultiLevelCodeTwoTree::Deadlist, &deadlist_tree),
         (MultiLevelCodeTwoTree::CentralMapping, &mapping_tree),
     ] {
         if tree == MultiLevelCodeTwoTree::CentralMapping {
@@ -3177,7 +3488,7 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         ),
     ]);
     let transaction_number_on_the_record_standing_for_this_root = record.transaction;
-    let rebuilt = TransactionOutput {
+    let mut rebuilt = TransactionOutput {
         root: *root,
         record,
         record_bytes,
@@ -3192,10 +3503,15 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         accounting_entries,
         accounting_tree: accounting_tree.version,
         central_mapping_tree: mapping_tree.version,
+        deadlist_entries,
+        deadlist_tree: deadlist_tree.version,
+        mapping_entries_kept_for_the_snapshots,
+        snapshots_being_destroyed,
         tree_table_entries,
         tree_identifiers,
         inode_record,
         deleted_inodes,
+        logged_operations,
         tombstone_containers: tombstone_tree_read.as_ref().map_or_else(Vec::new, |read| {
             read.containers
                 .iter()
@@ -3239,6 +3555,7 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         mapped_units,
         released: Vec::new(),
         quarantined_after_release_checksum_mismatch: Vec::new(),
+        units_relocated_by_this_publish: Vec::new(),
         key_order_mismatches: 0,
         writes: WritesByStructureKind::NOTHING_WRITTEN,
         // 重建出来的这一版属于**旧**实例。今天每条恢复路径（普通挂载、回退、切换）之后都要取新实例代号，
@@ -3251,17 +3568,32 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         highest_transaction_number_in_this_instance:
             transaction_number_on_the_record_standing_for_this_root,
     };
+    // I-2.5 先判盘上读来的提示、再盖映射位置：盖过之后提示就没了，盘上逆序的提示被映射位置盖掉，池级 checker 照盘上字节判坏的镜像
+    // 可写挂载却会收下。
+    pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;
+    take_the_mapping_positions_over_the_location_hints(&mut rebuilt, |key| {
+        mapping_position_of_key.get(key).copied()
+    });
+    // 盖进来的映射位置同样按 I-2.5 判（它的射程含中央映射条目的 value）：下一次发布重写父节点时把它们写进指针，写者在那里断言升序。
     pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;
-    Ok(RebuiltVersion::of_a_version_of_trees(rebuilt))
+    Ok(RebuiltVersionWithItsSnapshotEntries {
+        version: RebuiltVersion::of_a_version_of_trees(rebuilt),
+        snapshot_entries,
+    })
 }
 
 /// 一条根的树表里用户可见的两棵树（inode 树、extent 树）的根指针，取树表条目里那 86 字节的盘上原样；树表里没有那棵树的条目时 `None`。
 /// 抬 F 的上限要数「非空」的有效根：一条有效根算非空 ⟺ 这两样与它前一条有效根的不同（D16（发布语义） 已定项 1「非空」从盘上怎么认，
 /// 2026-09-17 用户定案）；树表单元自己的落点每次发布都变，不拿它比。
+/// 只读快照的两种条目（码 14 快照 inode 根、码 15 快照 extent 根）也是用户可见的根（主 agent 2026-10-13 定：码 14 / 15 是用户可见角色，
+/// 判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S2）：它们的根指针按盘上次序记在 `snapshot_tree_roots`；
+/// 「非空」那一判连它一起比（用户 2026-10-14 定「建快照那次发布算非空」，`mount::user_visible_trees_changed`）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct UserVisibleTreeRootPointers {
     pub inode_tree: Option<Vec<u8>>,
     pub extent_tree: Option<Vec<u8>>,
+    /// 树表里每一条码 14、码 15 条目的根指针盘上原样，按条目的盘上次序（即树 ID 升序）；没有快照的树表为空。
+    pub snapshot_tree_roots: Vec<Vec<u8>>,
 }
 
 impl UserVisibleTreeRootPointers {
@@ -3269,6 +3601,7 @@ impl UserVisibleTreeRootPointers {
     pub const ABSENT: Self = Self {
         inode_tree: None,
         extent_tree: None,
+        snapshot_tree_roots: Vec::new(),
     };
 }
 
@@ -3303,6 +3636,13 @@ pub fn user_visible_tree_root_pointers(
             | TREE_KIND_LINEAGE
             | TREE_KIND_LOGGED_OPERATIONS
             | TREE_KIND_RETIREMENT => continue,
+            // 只读快照的两种条目：一个快照一对、可以有多条，按盘上次序记下（配对在 `tree_table_entries_each_kind_at_most_once` 判过）。
+            TREE_KIND_SNAPSHOT_INODE_ROOT | TREE_KIND_SNAPSHOT_EXTENT_ROOT => {
+                pointers
+                    .snapshot_tree_roots
+                    .push(TreeTableEntry::root_pointer_bytes(entry_bytes).to_vec());
+                continue;
+            }
             _ => {
                 return Err(RecoveryFailure::UnitMalformed {
                     what: "树的种类没登记",
@@ -3828,6 +4168,14 @@ pub(crate) fn root_assembled_from_the_new_root_segment_of_the_record(
     }
 }
 
+/// 重建一版时 logged_ops 根不是非空的层级 0 节点（`RecoveryFailure::UnitMalformed` 的 `what`，用例按它认）。
+pub const LOGGED_OPERATIONS_ROOT_IS_NOT_A_NON_EMPTY_LEVEL_ZERO_NODE: &str =
+    "logged_ops 根不是层级 0 的非空节点";
+/// 重建一版时 logged_ops 根里有条目过不了字段表（宽不是 44、种类码保留、整理意图的 R 不对齐或槽数不是 64、补零非零、整理意图多于一条、
+/// 不按 key 严格升序；`logged_operations::logged_operation_entries_of_the_root_entries`）：可写挂载拒（判决 G1「R 的解码拒不对齐的值」）。
+pub const LOGGED_OPERATIONS_ROOT_ENTRIES_REFUSED_BY_THE_FIELD_TABLE: &str =
+    "logged_ops 根里有条目过不了字段表";
+
 /// 每棵树的 key 宽（码 2 头里的自述 key 宽要与它相符）；day-1 只注册的三棵没有节点要解。
 fn key_width_for_kind(kind: u16) -> Option<usize> {
     match kind {
@@ -3841,11 +4189,12 @@ fn key_width_for_kind(kind: u16) -> Option<usize> {
         TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {
             Some(CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES)
         }
-        // logged_ops 的字段表留到第一个写者落地时定（D8（核心索引结构） 已定项 5、C364（整理意图记录没有字段表））。
-        TREE_KIND_LIVELIST
-        | TREE_KIND_SPARSE_SIDE_TABLE
-        | TREE_KIND_DEADLIST
-        | TREE_KIND_LOGGED_OPERATIONS => None,
+        // logged_ops 的 key = 种类 2 + 签发 txg 8（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1 字段表）。
+        TREE_KIND_LOGGED_OPERATIONS => Some(LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES),
+        TREE_KIND_LIVELIST | TREE_KIND_SPARSE_SIDE_TABLE | TREE_KIND_DEADLIST => None,
+        // 只读快照的两种条目指着的是建快照那一版活头 inode 树、extent 树的根：由快照的读路径按那两棵树解（这一批还没有），
+        // 冷走读不按种类码读它们（`walk_to_file` 对它们 `continue`）。
+        TREE_KIND_SNAPSHOT_INODE_ROOT | TREE_KIND_SNAPSHOT_EXTENT_ROOT => None,
         _ => None,
     }
 }
@@ -4812,6 +5161,10 @@ struct MappedUnitCounts {
     extent_tree_nodes: usize,
     allocation_record_tree_nodes: usize,
     accounting_tree_nodes: usize,
+    /// deadlist 树的节点数（零根时 0，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）。
+    deadlist_tree_nodes: usize,
+    /// 被活快照钉住、留存的映射条目数 = deadlist 条目数（判决 S3 的 α：每条 deadlist 条目的映射 key 一条）。
+    mapping_entries_kept_for_the_snapshots: usize,
     /// inode 树的叶容器数 = 它的根的条目数（根恒是层级 1，下面每条条目一片容器；层级另判）。
     inode_leaf_containers: usize,
     /// 数据单元数 = extent 树里第一个文件的数据指针数（下段叶记录条数，或内联的那一个）。
@@ -4830,18 +5183,39 @@ fn mapping_entry_count_is_one_per_mapped_unit(
         + counts.extent_tree_nodes
         + counts.allocation_record_tree_nodes
         + counts.accounting_tree_nodes
+        + counts.deadlist_tree_nodes
+        + counts.mapping_entries_kept_for_the_snapshots
         + counts.inode_leaf_containers
         + counts.data_units
         + counts.deletion_tree_nodes;
     if mapping_entries != expected {
         return Err(RecoveryFailure::InvariantViolated {
             invariant: "E142 走读同款",
-            detail: "映射条目数不是 1 + extent 树、分配记录树、记账树的节点数 + inode 叶容器数 + 数据单元数",
+            detail: "映射条目数不是 1 + extent 树、分配记录树、记账树、deadlist 树的节点数 + deadlist 条目数 + inode 叶容器数 + 数据单元数",
         });
     }
     Ok(())
 }
 
+/// deadlist 树叶里的条目逐条解开（43 字节，`records::DeadlistEntry`），次序照叶里的（key 升序）。
+///
+/// # Errors
+/// 有一条窄于 43 ⇒ `EntryNarrowerThanItsFieldTable`。
+fn deadlist_entries_parsed(
+    leaf_entries_in_key_order: &[Vec<u8>],
+) -> Result<Vec<DeadlistEntry>, RecoveryFailure> {
+    leaf_entries_in_key_order
+        .iter()
+        .map(|bytes| {
+            DeadlistEntry::parse(bytes).ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
+                what: "deadlist 条目",
+                entry_bytes: bytes.len(),
+                field_table_bytes: DEADLIST_ENTRY_KEY_BYTES,
+            })
+        })
+        .collect()
+}
+
 /// 分配记录的跨度不是 0、代不晚于根。已释放的记录合法（D3（空间分配） 已定项 7：改写不删），它的代是释放代，同样不许晚于根。
 fn allocation_record_generations_and_spans_are_judged(
     records: &[AllocationRecord],
@@ -5139,6 +5513,7 @@ fn pointers_of_the_rebuilt_version_ascend_by_device(
                 .map(|(_, pointer)| pointer),
         )
         .chain(output.accounting_tree.pointers.iter())
+        .chain(output.deadlist_tree.pointers.iter())
         .chain(output.central_mapping_tree.pointers.iter());
     for pointer in node_pointers {
         pointer
@@ -5234,10 +5609,18 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
     let mut by_kind: BTreeMap<u16, IndexNodeHeader> = BTreeMap::new();
     let mut mapped_units_of_each_container_tree: BTreeMap<u16, usize> = BTreeMap::new();
     let mut accounting_tree: Option<CodeTwoTreeReadFromDisk> = None;
+    let mut deadlist_tree: Option<CodeTwoTreeReadFromDisk> = None;
     let mut allocation_tree: Option<AllocationRecordTreeReadFromDisk> = None;
     let mut extent_tree: Option<(TreeIdentifier, ExtentsOfTheFirstFile)> = None;
     for entry in &entries {
         tree_identifier_of_the_entry_is_below_the_watermark(entry, root)?;
+        // 只读快照的两种条目（码 14、15）：它们的根由快照的读路径解（判决 `research/prompts/m4-batch2-code-r1-main-verification.md`
+        // 第二节 S2；这一批不解），冷走读读的是活头那一版的文件，跳过它们。
+        if entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
+            || entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT
+        {
+            continue;
+        }
         if entry.root == NodePointer::empty_root() {
             continue;
         }
@@ -5305,6 +5688,27 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
             )?);
             continue;
         }
+        // deadlist 树（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：多层码 2 树，同记账树整棵读回来；
+        // 它的节点与它的条目各占映射条目（条目留存的那几条，判决 S3 的 α），数映射条目时要它们。
+        if entry.kind == TREE_KIND_DEADLIST {
+            deadlist_tree = Some(read_code_two_tree(
+                &entry.root,
+                &MultiLevelCodeTwoTree::Deadlist.read_expectation(entry.tree),
+                CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
+                root,
+                expected_filesystem_identifier,
+                &mut |pointer: &NodePointer| {
+                    read_mapped_tree_node_via_hint_then_central_mapping(
+                        reader,
+                        pointer,
+                        MappedTreeNodeClass::IndexNode,
+                        &central_mapping_locations_of_key,
+                        mapping_fallbacks,
+                    )
+                },
+            )?);
+            continue;
+        }
         // 墓碑树与谱系树：从根往下读码 2 节点（根满长高之后不止一层）、数进映射的节点与容器，不读容器（它们的内容不影响读回的文件）。
         if entry.kind == TREE_KIND_TOMBSTONE
             || entry.kind == TREE_KIND_LINEAGE
@@ -5360,6 +5764,7 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
             .unwrap_or(0)
     };
     let deletion_tree_nodes = usize::from(by_kind.contains_key(&TREE_KIND_DELETED_INODES))
+        + usize::from(by_kind.contains_key(&TREE_KIND_LOGGED_OPERATIONS))
         + container_tree_nodes(TREE_KIND_TOMBSTONE)
         + container_tree_nodes(TREE_KIND_LINEAGE)
         + container_tree_nodes(TREE_KIND_RETIREMENT);
@@ -5412,6 +5817,12 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
             extent_tree_nodes: roots.extent.nodes_in_bump_order.len(),
             allocation_record_tree_nodes: roots.allocation.nodes.len(),
             accounting_tree_nodes: roots.accounting.version.node_count(),
+            deadlist_tree_nodes: deadlist_tree
+                .as_ref()
+                .map_or(0, |read| read.version.node_count()),
+            mapping_entries_kept_for_the_snapshots: deadlist_tree
+                .as_ref()
+                .map_or(0, |read| read.leaf_entries_in_key_order.len()),
             inode_leaf_containers: roots.inode.entries.len(),
             data_units: roots.extent.data_pointers.len(),
             deletion_tree_nodes,
@@ -5499,6 +5910,156 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
     Ok(Some(content))
 }
 
+/// 读一个只读快照里的第一个文件（[`read_the_first_file_of_a_snapshot`]）读出了什么。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum SnapshotFileReading {
+    /// 这条根的树表里没有树号是那个号的码 14 条目。
+    NoSnapshotWithThisTree,
+    /// 快照那一刻没有第一个文件（inode 1 没有记录，或已在删除中、nlink = 0）。
+    NoFile,
+    /// 第一个文件的内容。
+    File(Vec<u8>),
+}
+
+/// 读这条根的树表里只读快照 `snapshot`（码 14 的树号，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 5）指着的
+/// 第一个文件：从它那一对码 14 / 15 条目的两条根指针往下读 inode 树与 extent 树，节点按根指针头部的出生树核（判决 S1「读快照指着的根按根指针头部
+/// 的出生树比、不按条目自己的树号」），单元的位置提示读不出时经**这条根**的中央映射回退——快照引用、活头已换下的单元靠的就是留存在这里的
+/// 映射条目（判决 S3 的 α）。判定与冷走读那几道相同（inode 树根层级 1、叶容器与条目对得上、单元个数对得上 size、数据单元按查找路径核）。
+///
+/// # Errors
+/// 树表读不出、解不开、条目不合 I-9.16 或配不成对；快照的两棵树、叶容器或数据单元读不出（提示与映射都读不出）、解不开或判红。
+pub fn read_the_first_file_of_a_snapshot(
+    reader: &dyn PoolReader,
+    root: &RootRecord,
+    snapshot: TreeIdentifier,
+) -> Result<SnapshotFileReading, RecoveryFailure> {
+    let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
+    let tree_table_bytes = read_unit_via_locations(
+        reader,
+        &root.tree_table.locations,
+        usize::try_from(NODE_BYTES).expect("16384"),
+    )?;
+    let tree_table =
+        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "树表单元",
+        })?;
+    tree_table_key_width_is_the_tree_tables(&tree_table)?;
+    let entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;
+    let Some(pair) = snapshot_tree_table_entries_paired_one_to_one(&entries)?
+        .into_iter()
+        .find(|pair| pair.inode_root.tree == snapshot)
+    else {
+        return Ok(SnapshotFileReading::NoSnapshotWithThisTree);
+    };
+    let central_mapping_tree = CentralMappingTreeReadOnFirstUse {
+        reader,
+        root,
+        expected_filesystem_identifier,
+        tree: OnceCell::new(),
+    };
+    let central_mapping_locations_of_key =
+        |mapping_key: &[u8]| central_mapping_tree.locations_of_key(mapping_key);
+    let mut stale_location_hint_hops_not_exposed_by_this_reader = 0usize;
+    let extent_root_pointer = pair.extent_root.root;
+    let extents = extents_of_the_first_file(read_extent_tree(
+        &ExtentTreeReading {
+            tree: extent_root_pointer.head.birth_tree,
+            judgement: ExtentTreeHeaderJudgement::EveryHeaderAgainstItsReference,
+            root,
+            expected_filesystem_identifier,
+        },
+        &extent_root_pointer,
+        &mut |pointer: &NodePointer| {
+            read_mapped_tree_node_via_hint_then_central_mapping(
+                reader,
+                pointer,
+                MappedTreeNodeClass::IndexNode,
+                &central_mapping_locations_of_key,
+                &mut stale_location_hint_hops_not_exposed_by_this_reader,
+            )
+        },
+    )?)?;
+    let inode_root_pointer = pair.inode_root.root;
+    let inode_root_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
+        reader,
+        &inode_root_pointer,
+        MappedTreeNodeClass::IndexNode,
+        &central_mapping_locations_of_key,
+        &mut stale_location_hint_hops_not_exposed_by_this_reader,
+    )?;
+    let inode_root = tree_root_checked_against_its_pointer(
+        &inode_root_bytes,
+        inode_root_pointer.head.birth_tree,
+        key_width_for_kind(TREE_KIND_INODE).expect("inode 树的 key 宽登记过"),
+        &inode_root_pointer,
+        root,
+        expected_filesystem_identifier,
+    )?;
+    inode_tree_root_level_is_one(&inode_root)?;
+    let mut inode_record: Option<InodeRecord> = None;
+    for entry in &inode_root.entries {
+        let (separator_key, identity, child) = inode_internal_entry_parsed_and_judged(entry)?;
+        let leaf_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
+            reader,
+            &child,
+            MappedTreeNodeClass::PackedRecordUnit,
+            &central_mapping_locations_of_key,
+            &mut stale_location_hint_hops_not_exposed_by_this_reader,
+        )?;
+        let leaf =
+            parse_packed_unit(&leaf_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+                what: "inode 叶容器",
+            })?;
+        let records = inode_leaf_container_judged_against_its_entry(
+            &leaf,
+            InodeInternalEntry {
+                separator_key,
+                identity,
+                child,
+            },
+            root,
+            expected_filesystem_identifier,
+        )?;
+        if let Some(record) = records
+            .into_iter()
+            .find(|record| record.inode == FIRST_INODE_NUMBER)
+        {
+            inode_record = Some(record);
+        }
+    }
+    let Some(inode_record) = inode_record else {
+        return Ok(SnapshotFileReading::NoFile);
+    };
+    if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
+        return Ok(SnapshotFileReading::NoFile);
+    }
+    data_unit_count_matches_the_inode_size(extents.data_pointers.len(), &inode_record)?;
+    let mut content: Vec<u8> =
+        Vec::with_capacity(usize::try_from(inode_record.size).expect("文件长度装得进 usize"));
+    // 迭代次数的上界是这个文件的单元数（上面核过等于 inode size 除出来的单元数）；跨轮携带的只有已经拼出来的内容。
+    for (position, pointer) in extents.data_pointers.iter().enumerate() {
+        let bytes = read_data_unit_via_hint_then_central_mapping(
+            reader,
+            pointer,
+            &central_mapping_locations_of_key,
+            &mut stale_location_hint_hops_not_exposed_by_this_reader,
+        )?
+        .into_content()?;
+        let payload = data_unit_judged_against_its_pointer_and_the_inode_record(
+            &bytes,
+            DataUnitOfTheFirstFile {
+                position,
+                pointer,
+                extent_tree: extent_root_pointer.head.birth_tree,
+            },
+            &inode_record,
+            expected_filesystem_identifier,
+        )?;
+        content.extend_from_slice(payload);
+    }
+    Ok(SnapshotFileReading::File(content))
+}
+
 /// 同 [`walk_to_file_in_the_unit_area_starting_at`]，单元区起点先择一次系统配置现算（[`choose_system_configuration`]：
 /// 多读每块盘的两个系统配置槽）。给手里没有择好的系统配置的调用方（用例）；[`recover`] 走带起点的那一份。
 ///
@@ -5535,6 +6096,24 @@ pub fn rebuild_version(
     )
 }
 
+/// 同 [`rebuild_version`]（单元区起点先择一次系统配置现算），另交出树表里的只读快照条目
+/// （[`rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at`]）。给手里没有择好的系统配置的调用方（用例）。
+///
+/// # Errors
+/// 同 [`rebuild_version`]。
+pub fn rebuild_version_with_its_snapshot_entries(
+    reader: &dyn PoolReader,
+    root: &RootRecord,
+    record_standing_for_root: Option<JournalRecord>,
+) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
+    rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
+        reader,
+        unit_area_start_of_the_pool(reader)?,
+        root,
+        record_standing_for_root,
+    )
+}
+
 /// 所选那一版走读失败之后退回哪一版（F2 乙，用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md`
 /// 第三节第 7 条）：系统配置没见证过所选那一版时，退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根——交回的不一定是
 /// 见证到的那一版本身，是这一段里最新的那条（`F2_NEWEST`，
@@ -6290,7 +6869,7 @@ mod allocation_record_pool_geometry_tests {
 }
 
 /// 读侧认得删除新写出的三棵树的 key 宽（里程碑三·2 第 7 条「读侧补新种类的 key 宽」）：根指针不全零时冷走读按它读根、核根的自述 key 宽；
-/// 不认得的种类在那一步拒成「树的种类没登记」。logged_ops 还没有字段表，照旧不认。
+/// 不认得的种类在那一步拒成「树的种类没登记」。logged_ops 的 key 是种类 2 + 签发 txg 8（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）。
 #[cfg(test)]
 mod tree_kind_key_width_tests {
     use super::key_width_for_kind;
@@ -6300,8 +6879,8 @@ mod tree_kind_key_width_tests {
     };
 
     #[test]
-    fn deleted_inodes_tombstone_and_lineage_trees_have_eight_byte_keys_and_logged_operations_none()
-    {
+    fn deleted_inodes_tombstone_lineage_and_retirement_trees_have_eight_byte_keys_and_logged_operations_ten(
+    ) {
         assert_eq!(
             key_width_for_kind(TREE_KIND_DELETED_INODES),
             Some(8),
@@ -6324,8 +6903,8 @@ mod tree_kind_key_width_tests {
         );
         assert_eq!(
             key_width_for_kind(TREE_KIND_LOGGED_OPERATIONS),
-            None,
-            "logged_ops 没有字段表"
+            Some(10),
+            "logged_ops 的 key = 种类 2 + 签发 txg 8"
         );
         assert_eq!(key_width_for_kind(TREE_KIND_INODE), Some(8));
     }
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 43b23bf5..20089dba 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -12,7 +12,7 @@ use std::collections::{BTreeMap, BTreeSet};
 use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, DELETED_INODES_ENTRY_BYTES, EXTENT_KEY_BYTES, INODE_INTERNAL_ENTRY,
     INODE_RECORD_BYTES, INSTANCE_ROW_BYTES, JOURNAL_NAMED_ENTRIES_PER_RECORD, LINEAGE_RECORD_BYTES,
-    MAPPING_ENTRY_BYTES, RETIREMENT_RECORD_BYTES, SLOT_BYTES,
+    LOGGED_OPERATIONS_ENTRY_BYTES, MAPPING_ENTRY_BYTES, RETIREMENT_RECORD_BYTES, SLOT_BYTES,
     SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, TOMBSTONE_RECORD_BYTES, TREE_IDENTIFIER_ACCOUNTING,
     TREE_IDENTIFIER_ALLOCATION_RECORDS, TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST,
     TREE_IDENTIFIER_DELETED_INODES, TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE,
@@ -28,13 +28,15 @@ use crate::address::{
 };
 use crate::admission::{
     admission_reading_before_a_publish_on_a_ring_of, admit_on_every_device,
+    admit_the_empty_cluster_segments_floor_on_every_device,
     admit_the_units_landing_on_every_device,
     checkpoint_cost_of_the_version_to_build_on_with_node_capacities,
     committed_reservation_for_tombstones_and_retirement_records,
     demand_of_the_roles_on_each_device, inode_records_of_the_inode_tree,
     live_objects_of_the_inode_tree, publish_has_an_ordinary_allocation, units_of_a_publish_to_land,
-    AdmissionRefusedOnSomeDevices, BytesSummedOverAllReplicas, PlacementRoomOnOneDevice,
-    ReplicaCount, SpaceAdmission, TombstoneAndRetirementPrepayBasis,
+    AdmissionRefusedOnSomeDevices, BytesSummedOverAllReplicas,
+    EmptyClusterSegmentsBelowTheFloorOnSomeDevices, EmptyClusterSegmentsConjunctReach,
+    PlacementRoomOnOneDevice, ReplicaCount, SpaceAdmission, TombstoneAndRetirementPrepayBasis,
 };
 use crate::allocation_record_tree::{
     build_allocation_record_tree_node, children_of, nodes_holding_records,
@@ -49,10 +51,14 @@ use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
 use crate::checksum::crc32_castagnoli;
 use crate::code_two_tree::{
     build_internal_entry, internal_entry_width_in_bytes, leaf_position_routed_to,
-    plan_the_tree_after_this_publish, CodeTwoKeyFieldWidths, CodeTwoTreeKey,
-    CodeTwoTreeNodeCapacity, CodeTwoTreeNodeContents, CodeTwoTreeNodeOrigin,
-    CodeTwoTreeNodePosition, CodeTwoTreePlan, CodeTwoTreeReadExpectation, CodeTwoTreeRefusal,
-    CodeTwoTreeShape, CodeTwoTreeVersion,
+    plan_the_tree_after_this_publish, plan_the_tree_after_this_publish_down_to_a_zero_root,
+    CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodeCapacity, CodeTwoTreeNodeContents,
+    CodeTwoTreeNodeOrigin, CodeTwoTreeNodePosition, CodeTwoTreePlan, CodeTwoTreeReadExpectation,
+    CodeTwoTreeRefusal, CodeTwoTreeShape, CodeTwoTreeVersion,
+};
+use crate::compaction_plan::{
+    region_is_empty_by_the_records, PlannedRelocation, RelocationDestinationPolicy,
+    RelocationPublishOrder,
 };
 use crate::container_tree::{
     ContainerTreeAfterThisPublish, ContainerTreeCapacities, ContainerTreeNodeOrigin,
@@ -84,6 +90,11 @@ use crate::lineage_tree::{
     append_lineage_record_to_the_open_container, LineageContainer, LineageContainerIndexInTree,
     LineageTreeAfterThisPublish,
 };
+use crate::logged_operations::{
+    live_compaction_intent_of_the_entries, logged_operations_root_entries, CompactionIntent,
+    CompactionRegion, LoggedOperation, LoggedOperationEntry, LoggedOperationKind,
+    LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES,
+};
 use crate::make_filesystem::{
     location_entries, MakeFilesystemParameters, MKFS_INSTANCE_GENERATION, TREE_TABLE_KEY_WIDTH,
 };
@@ -91,25 +102,30 @@ use crate::pointer::{
     slot_shared_by_both_location_entries, BirthSequence, DataPointer,
     LocationEntriesOnDifferentSlots, LocationEntry, NodePointer, PointerHead,
 };
+use crate::records::{
+    birth_txg_of_a_mapping_key, DeadlistEntry, SnapshotDestructionMark, DEADLIST_ENTRY_KEY_BYTES,
+};
 use crate::records::{
     build_extent_record, build_inode_internal_entry, build_mapping_entry, data_key_tail,
     mapping_key_for_data, mapping_key_for_node, node_key_tail, parse_mapping_entry,
     AccountingEntry, InodeRecord, LineageRecord, RetirementRecord, TombstoneRecord, TreeTableEntry,
     ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF, NLINK_OF_AN_INODE_BEING_DELETED,
-    NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES, STATISTIC_ALLOCATED_BYTES,
-    STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
+    NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES, SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION,
+    STATISTIC_ALLOCATED_BYTES, STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
     STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_FREE_BYTES,
     STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION, STATISTIC_PENDING_DELETE_BYTES,
     STATISTIC_UNRECLAIMABLE_BYTES, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
     TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE,
     TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT,
-    TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
+    TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT, TREE_KIND_SPARSE_SIDE_TABLE,
+    TREE_KIND_TOMBSTONE,
 };
 use crate::recovery::{
     effective_rollback_floor_rereading_unreadable_reads_once, highest_root_instance,
+    snapshot_tree_table_entries_paired_one_to_one,
     system_configuration_slots_of_one_device_read_once,
     system_configuration_witness_is_undecidable, tree_table_entry_count,
-    verified_system_configuration_slots, PoolReader, RecoveryFailure,
+    verified_system_configuration_slots, PoolReader, RecoveryFailure, SnapshotTreeTableEntries,
     SystemConfigurationSlotsOfOneDevice,
 };
 use crate::retirement_tree::{
@@ -1974,7 +1990,10 @@ pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
             | TransactionUnit::InodeRoot
             | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
+            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+            | TransactionUnit::DeadlistTree
             | TransactionUnit::DeletedInodesRoot
+            | TransactionUnit::LoggedOperationsRoot
             | TransactionUnit::TombstoneLeafContainer(_)
             | TransactionUnit::TombstoneRoot
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
@@ -2297,6 +2316,11 @@ pub enum TransactionUnit {
     AccountingTreeNodeBelowTheRoot(CodeTwoTreeNodePosition),
     /// 记账树的根（只有一个节点时它就是根兼叶）。
     AccountingTree,
+    /// deadlist 树里根之下的一个节点（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3：第三棵多层码 2 树），
+    /// 同 `AccountingTreeNodeBelowTheRoot`。
+    DeadlistTreeNodeBelowTheRoot(CodeTwoTreeNodePosition),
+    /// deadlist 树的根（只有一个节点时它就是根兼叶）：树里有条目的一版才有它，没有条目时树表条目的根指针置零、这个角色不在那一版里。
+    DeadlistTree,
     /// `deleted_inodes` 树的根（码 2、层级 0，D8（核心索引结构） 已定项 16）：队列非空的一版才有它，队列空时树表条目的根指针置零、
     /// 这个角色不在那一版里。任何一版至多一条 key（第 6 条），根之下不长层。
     DeletedInodesRoot,
@@ -2314,6 +2338,10 @@ pub enum TransactionUnit {
     LineageRoot,
     /// 谱系树根之下的一个码 2 节点（根满长高之后才有），同 `TombstoneNodeBelowTheRoot`。
     LineageNodeBelowTheRoot(ContainerTreeNodePosition),
+    /// logged_ops 树的根（码 2、层级 0，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1：单树、定宽联合条目 44 字节，
+    /// `crate::logged_operations`）：树里有条目的一版才有它，没有条目时树表条目的根指针置零、这个角色不在那一版里。全池活整理意图
+    /// 至多一条（e1），一个根装 369 条，根之下不长层。只有整理签发意图、删意图的那次发布重写它（[`publish_compaction_step`]），别的发布照抄。
+    LoggedOperationsRoot,
     /// 退役树的一片叶容器（码 3 打包记录类型 3，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）：带它在树里的叶序。
     /// 只有身份消失（删空摘掉、被左邻吸收）的那次发布追加记录、重写它；别的发布照抄。
     RetirementLeafContainer(RetirementContainerIndexInTree),
@@ -2338,11 +2366,13 @@ pub enum TransactionUnit {
     InstanceTablePageAfterTheFirst(InstanceTablePageIndex),
 }
 
-/// 多层码 2 树里哪一棵：记账树或中央映射树（D8（核心索引结构） 已定项 11 的两棵照分裂做的树）。
+/// 多层码 2 树里哪一棵：记账树、中央映射树（D8（核心索引结构） 已定项 11 的两棵照分裂做的树）或 deadlist 树
+/// （判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3：第三棵多层码 2 树，种类 8，key 43、value 空）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum MultiLevelCodeTwoTree {
     Accounting,
     CentralMapping,
+    Deadlist,
 }
 
 impl MultiLevelCodeTwoTree {
@@ -2362,10 +2392,14 @@ impl MultiLevelCodeTwoTree {
             (MultiLevelCodeTwoTree::CentralMapping, false) => {
                 TransactionUnit::MappingTreeNodeBelowTheRoot(position)
             }
+            (MultiLevelCodeTwoTree::Deadlist, true) => TransactionUnit::DeadlistTree,
+            (MultiLevelCodeTwoTree::Deadlist, false) => {
+                TransactionUnit::DeadlistTreeNodeBelowTheRoot(position)
+            }
         }
     }
 
-    /// 叶条目的字段表宽：记账条目 34、映射条目 55。
+    /// 叶条目的字段表宽：记账条目 34、映射条目 55、deadlist 条目 43（value 空，条目就是 key）。
     #[must_use]
     pub fn leaf_entry_width_in_bytes(self) -> usize {
         match self {
@@ -2375,6 +2409,7 @@ impl MultiLevelCodeTwoTree {
             MultiLevelCodeTwoTree::CentralMapping => {
                 usize::try_from(MAPPING_ENTRY_BYTES).expect("55")
             }
+            MultiLevelCodeTwoTree::Deadlist => DEADLIST_ENTRY_KEY_BYTES,
         }
     }
 
@@ -2383,6 +2418,7 @@ impl MultiLevelCodeTwoTree {
         match self {
             MultiLevelCodeTwoTree::Accounting => CodeTwoKeyFieldWidths::ACCOUNTING,
             MultiLevelCodeTwoTree::CentralMapping => CodeTwoKeyFieldWidths::CENTRAL_MAPPING,
+            MultiLevelCodeTwoTree::Deadlist => CodeTwoKeyFieldWidths::DEADLIST,
         }
     }
 
@@ -2395,11 +2431,12 @@ impl MultiLevelCodeTwoTree {
             internal_entry_name: match self {
                 MultiLevelCodeTwoTree::Accounting => "记账树内部条目",
                 MultiLevelCodeTwoTree::CentralMapping => "中央映射树内部条目",
+                MultiLevelCodeTwoTree::Deadlist => "deadlist 树内部条目",
             },
         }
     }
 
-    /// 格式算出来的节点容量：记账树叶 477、内部 150，中央映射树叶 294、内部 143。
+    /// 格式算出来的节点容量：记账树叶 477、内部 150，中央映射树叶 294、内部 143，deadlist 树叶 376、内部 125。
     #[must_use]
     pub fn node_capacity_of_the_node_format(self) -> CodeTwoTreeNodeCapacity {
         CodeTwoTreeNodeCapacity::of_the_node_format(
@@ -2452,6 +2489,11 @@ impl CodeTwoTreeNodeCapacities {
                 },
                 MultiLevelCodeTwoTree::CentralMapping,
             ) => central_mapping,
+            // deadlist 树不随这个开关压小：开关只罩记账树与中央映射树（它的两个字段），deadlist 树的分裂、收缩走的是同一段
+            // `code_two_tree` 规划（那两棵树压小之后测过的那几条写序）。
+            (CodeTwoTreeNodeCapacities::CappedForTests { .. }, MultiLevelCodeTwoTree::Deadlist) => {
+                MultiLevelCodeTwoTree::Deadlist.node_capacity_of_the_node_format()
+            }
         }
     }
 }
@@ -2511,6 +2553,11 @@ impl TransactionUnit {
                 format!("t6@{}.{}", position.level, position.index_in_level)
             }
             TransactionUnit::AccountingTree => "t6".to_string(),
+            // deadlist 树两种角色字节表七没有登记：写成「tdl@层级.同层序号」「tdl」，只出现在报错与用例消息里。
+            TransactionUnit::DeadlistTreeNodeBelowTheRoot(position) => {
+                format!("tdl@{}.{}", position.level, position.index_in_level)
+            }
+            TransactionUnit::DeadlistTree => "tdl".to_string(),
             // 删除新写出的三种角色字节表七没有登记（layout/03-third-txn.md 只登记形态、不编步号）：写成「td」「tt@叶序」「tt」，
             // 只出现在报错与用例消息里。
             TransactionUnit::DeletedInodesRoot => "td".to_string(),
@@ -2525,6 +2572,8 @@ impl TransactionUnit {
             TransactionUnit::LineageNodeBelowTheRoot(position) => {
                 format!("tl^{}.{}", position.level, position.index_in_level)
             }
+            // logged_ops 根同样没有登记：写成「tlo」。
+            TransactionUnit::LoggedOperationsRoot => "tlo".to_string(),
             // 退役树三种角色同样没有登记：写成「tr@叶序」「tr」「tr^层级.同层序号」。
             TransactionUnit::RetirementLeafContainer(index) => format!("tr@{}", index.0),
             TransactionUnit::RetirementRoot => "tr".to_string(),
@@ -2569,7 +2618,10 @@ impl TransactionUnit {
             | TransactionUnit::AllocationTree
             | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
+            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+            | TransactionUnit::DeadlistTree
             | TransactionUnit::DeletedInodesRoot
+            | TransactionUnit::LoggedOperationsRoot
             | TransactionUnit::TombstoneLeafContainer(_)
             | TransactionUnit::TombstoneRoot
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
@@ -2603,7 +2655,10 @@ impl TransactionUnit {
             | TransactionUnit::AllocationTree
             | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
+            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+            | TransactionUnit::DeadlistTree
             | TransactionUnit::DeletedInodesRoot
+            | TransactionUnit::LoggedOperationsRoot
             | TransactionUnit::TombstoneRoot
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
             | TransactionUnit::LineageRoot
@@ -2630,6 +2685,9 @@ impl TransactionUnit {
             | TransactionUnit::AllocationTree => trees.allocation_records,
             TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree => trees.accounting,
+            TransactionUnit::DeadlistTreeNodeBelowTheRoot(_) | TransactionUnit::DeadlistTree => {
+                trees.deadlist
+            }
             TransactionUnit::DeletedInodesRoot => trees.deleted_inodes,
             TransactionUnit::TombstoneLeafContainer(_)
             | TransactionUnit::TombstoneRoot
@@ -2637,6 +2695,7 @@ impl TransactionUnit {
             TransactionUnit::LineageLeafContainer(_)
             | TransactionUnit::LineageRoot
             | TransactionUnit::LineageNodeBelowTheRoot(_) => trees.lineage,
+            TransactionUnit::LoggedOperationsRoot => trees.logged_operations,
             TransactionUnit::RetirementLeafContainer(_)
             | TransactionUnit::RetirementRoot
             | TransactionUnit::RetirementNodeBelowTheRoot(_) => trees.retirement,
@@ -2672,7 +2731,10 @@ impl TransactionUnit {
             | TransactionUnit::AllocationTree
             | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
+            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+            | TransactionUnit::DeadlistTree
             | TransactionUnit::DeletedInodesRoot
+            | TransactionUnit::LoggedOperationsRoot
             | TransactionUnit::TombstoneRoot
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
             | TransactionUnit::LineageRoot
@@ -2864,6 +2926,18 @@ pub struct TransactionOutput {
     pub accounting_tree: CodeTwoTreeVersion,
     /// 这一版中央映射树的形状与每个节点的指针，同上；根指针同时住根记录。
     pub central_mapping_tree: CodeTwoTreeVersion,
+    /// 这一版 deadlist 树的条目，按 key 升序（D5（快照 / 空间记账机制） 已定项 12；判决 `research/prompts/m4-snapshot-r1-main-verification.md`
+    /// 第三节 S3）：被活快照引用的块被杀时进这里，不释放；删快照的批放掉其中纪元落在 [lo, hi) 里、birth > P 的那几条。空 ⇒ 零根。
+    pub deadlist_entries: Vec<DeadlistEntry>,
+    /// 这一版 deadlist 树的形状与每个节点的指针，同记账树那一份；零根时一个节点都没有。
+    pub deadlist_tree: CodeTwoTreeVersion,
+    /// 被活快照钉住的单元留在中央映射树里的那几条映射条目（判决 S3「被钉单元的映射条目留在中央映射树里（α）」）：每条 deadlist 条目的
+    /// 映射 key 一条，带它的两条位置项（释放判定路径经映射释放，D19（块指针的结构与宽度预算） 已定项 5 第 1 条：删快照的批从这里取落点）。
+    /// 按映射 key 的字段序升序。它们不在 `mapped_units` 里（这一版的树不引用它们）。
+    pub mapping_entries_kept_for_the_snapshots: Vec<(Vec<u8>, [LocationEntry; 2])>,
+    /// 这一版树表里带「销毁中」记号的码 14 条目的树号（判决 S4：删快照的意图写在那条码 14 自己身上，flags 位 0），按树号升序。
+    /// 第一版删快照串行（第七节岔路 2），至多一个。
+    pub snapshots_being_destroyed: Vec<TreeIdentifier>,
     pub tree_table_entries: Vec<TreeTableEntry>,
     /// 这一版那十二棵树各自的号：第一个文件版本那次从水位发出来，之后每一版照抄（从盘上重建的版本按树表条目的种类与根记录里
     /// 中央映射树根指针的出生树读回来）。
@@ -2875,6 +2949,10 @@ pub struct TransactionOutput {
     /// 这一版 `deleted_inodes` 的号（D8（核心索引结构） 已定项 16：队列 = inode 树里 nlink = 0 的号集合，任何一版至多一条），升序。
     /// 空 ⇒ 树表条目的根指针置零、这一版没有 [`TransactionUnit::DeletedInodesRoot`]。
     pub deleted_inodes: Vec<InodeNumber>,
+    /// 这一版 logged_ops 树的条目，按 key 升序（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1：第一版只装整理意图，
+    /// 全池活整理意图至多一条）。空 ⇒ 树表条目的根指针置零、这一版没有 [`TransactionUnit::LoggedOperationsRoot`]。
+    /// 随现行那一版：管理员回退不取 R_old 的（D23（journal 的角色与格式） 已定项 14「墓碑树、谱系树、logged_ops 随现行那一版」）。
+    pub logged_operations: Vec<LoggedOperationEntry>,
     /// 这一版墓碑树的叶容器，按叶序（= 容器号升序）：每片的内容与这一版它的指针（D18（块里携带什么信息） 已定项 24）。
     /// 空 ⇒ 墓碑树零根。
     pub tombstone_containers: Vec<TombstoneContainerVersion>,
@@ -2904,6 +2982,9 @@ pub struct TransactionOutput {
     /// 落盘即跨重挂。从盘上重建的版本不是这个进程发布的，是空的。
     pub quarantined_after_release_checksum_mismatch:
         Vec<CopyQuarantinedAfterReleaseChecksumMismatch>,
+    /// 这次发布搬走的单元（整理的一批，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3，[`publish_relocation_batch`]）：
+    /// 映射 key、它属于这一版的哪个角色（或是被快照钉住、留存映射条目的那一种）、源与目的地的两条位置项。别的发布与从盘上重建的版本都是空的。
+    pub units_relocated_by_this_publish: Vec<UnitRelocatedByThisPublish>,
     /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数：同一请求内取号序与 key 序不一致的次数，新池新建文件恒 0。
     pub key_order_mismatches: u64,
     /// 这次发布交给设备的写，按结构种类（增补 1）；从盘上重建的版本不是这个进程写出的，是空账。
@@ -2913,6 +2994,123 @@ pub struct TransactionOutput {
     pub highest_transaction_number_in_this_instance: u64,
 }
 
+/// 一个被搬走的单元归谁（[`UnitRelocatedByThisPublish`]）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RelocatedUnitHolder {
+    /// 这一版的树引用着它：它是这个角色。搬迁不改角色，只改它的落点。
+    RoleOfThisVersion(TransactionUnit),
+    /// 只被活快照引用、映射条目留存在 `mapping_entries_kept_for_the_snapshots` 里（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
+    /// 第三节 S3 的 α；判决 G3「被快照引用的块照样能搬」）：这一版的树不引用它，它没有角色。
+    KeptForTheSnapshots,
+}
+
+/// 整理的一批搬走的一个单元：K 个单元原样字节写目的地（头不盖戳、birth 不变），映射条目换到目的地，源落点进 defer
+/// （判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct UnitRelocatedByThisPublish {
+    pub mapping_key: Vec<u8>,
+    pub holder: RelocatedUnitHolder,
+    /// 搬之前映射条目里的两条位置项（源落点，两盘同槽）。
+    pub source: [LocationEntry; 2],
+    /// 搬之后映射条目里的两条位置项（目的落点，由这次发布的分配器按 D3（空间分配） 已定项 8 现取）。
+    pub destination: [LocationEntry; 2],
+}
+
+/// 一版里每一处指向进映射单元的指针，换成那个单元在映射里的位置（判决 G3「写路径改成以映射为准」）：`position_of_key(映射 key)` 交回
+/// 那个单元此刻的映射位置，交 `None` 的不动。指针按类型找 key：数据指针取码 1 key，inode 叶容器与三棵容器树的叶容器取码 3 key，
+/// 其余节点指针（extent 树、分配记录树、记账树、deadlist 树、容器树根之下的节点、树表条目里各棵树的根）取码 2 key；零根不动。
+/// 映射树自己的节点、树表、实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8），不在这里。`units` 里进映射角色的落点同步换成映射位置。
+///
+/// 内存里的这一版因此记的是映射位置：父节点下一次重写时写进它的就是映射位置（判决 G3「父节点重写时写映射位置」，C417 第一版政策
+/// 「随重写刷新」），照抄的单元装映射条目时取的就是上一版的映射位置；盘上没重写的父节点里仍是旧提示。两处调它：
+/// 整理的一批在装单元之前对上一版的拷贝调（目的地交给它），从盘上重建一版之后对重建出来的那一版调（映射条目交给它，
+/// 判决 G3「重建时映射条目的位置覆盖提示」）。
+pub fn take_the_mapping_positions_over_the_location_hints<PositionOfKey>(
+    version: &mut TransactionOutput,
+    position_of_key: PositionOfKey,
+) where
+    PositionOfKey: Fn(&[u8]) -> Option<[LocationEntry; 2]>,
+{
+    let refresh_node_pointer = |pointer: &mut NodePointer, unit_class: u8| {
+        if *pointer == NodePointer::empty_root() {
+            return;
+        }
+        if let Some(locations) = position_of_key(&mapping_key_for_node(unit_class, *pointer)) {
+            pointer.locations = locations;
+        }
+    };
+    for data_pointer in &mut version.data_pointers {
+        if let Some(locations) = position_of_key(&mapping_key_for_data(
+            data_pointer.head,
+            data_pointer.write_order,
+        )) {
+            data_pointer.locations = locations;
+        }
+    }
+    for (_, pointer) in &mut version.extent_tree.upper_nodes {
+        refresh_node_pointer(pointer, UNIT_CLASS_INDEX_NODE);
+    }
+    for (_, pointer) in &mut version.extent_tree.lower_nodes {
+        refresh_node_pointer(pointer, UNIT_CLASS_INDEX_NODE);
+    }
+    for container in &mut version.inode_leaf_containers {
+        refresh_node_pointer(&mut container.pointer, UNIT_CLASS_PACKED);
+    }
+    for (_, pointer) in &mut version.allocation_record_tree.nodes {
+        refresh_node_pointer(pointer, UNIT_CLASS_INDEX_NODE);
+    }
+    for pointer in version
+        .accounting_tree
+        .pointers
+        .iter_mut()
+        .chain(version.deadlist_tree.pointers.iter_mut())
+    {
+        refresh_node_pointer(pointer, UNIT_CLASS_INDEX_NODE);
+    }
+    for container in &mut version.tombstone_containers {
+        refresh_node_pointer(&mut container.pointer, UNIT_CLASS_PACKED);
+    }
+    for container in &mut version.lineage_containers {
+        refresh_node_pointer(&mut container.pointer, UNIT_CLASS_PACKED);
+    }
+    for container in &mut version.retirement_containers {
+        refresh_node_pointer(&mut container.pointer, UNIT_CLASS_PACKED);
+    }
+    for (_, pointer) in version
+        .tombstone_tree_index_nodes
+        .pointers_below_the_root
+        .iter_mut()
+        .chain(
+            version
+                .lineage_tree_index_nodes
+                .pointers_below_the_root
+                .iter_mut(),
+        )
+        .chain(
+            version
+                .retirement_tree_index_nodes
+                .pointers_below_the_root
+                .iter_mut(),
+        )
+    {
+        refresh_node_pointer(pointer, UNIT_CLASS_INDEX_NODE);
+    }
+    for entry in &mut version.tree_table_entries {
+        refresh_node_pointer(&mut entry.root, UNIT_CLASS_INDEX_NODE);
+    }
+    for (identity, key) in &version.mapped_units {
+        if let Some(locations) = position_of_key(key) {
+            if let Some(unit) = version
+                .units
+                .iter_mut()
+                .find(|unit| unit.identity == *identity)
+            {
+                unit.slot = locations[0].slot;
+            }
+        }
+    }
+}
+
 /// 释放之前按位置项读盘核校验和、这个单元有一份核出对不上（读不出也算）而隔离的一份：哪个角色、哪块盘、那一份的落点
 /// （槽取位置项里的，跨度取角色的）、那一份自己读出来的样子。隔离就是那块盘上这个落点的分配记录留在已分配；
 /// 一个单元只要有一份对不上，每块盘上那一份都隔离（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-25 定：各盘的账保持对称）。
@@ -3118,6 +3316,7 @@ impl TransactionOutput {
         let root_role = match tree {
             MultiLevelCodeTwoTree::Accounting => TransactionUnit::AccountingTree,
             MultiLevelCodeTwoTree::CentralMapping => TransactionUnit::MappingTree,
+            MultiLevelCodeTwoTree::Deadlist => TransactionUnit::DeadlistTree,
         };
         let root_header = parse_index_node(&self.unit(root_role).bytes)
             .expect("根单元是这个进程装的，或重建时解过、核过的");
@@ -3158,9 +3357,24 @@ impl TransactionOutput {
         match tree {
             MultiLevelCodeTwoTree::Accounting => &self.accounting_tree,
             MultiLevelCodeTwoTree::CentralMapping => &self.central_mapping_tree,
+            MultiLevelCodeTwoTree::Deadlist => &self.deadlist_tree,
         }
     }
 
+    /// 活头 inode 树那条条目里的 `previous_snapshot_txg`（D5（快照 / 空间记账机制） 已定项 9；已定项 19 判定里的 `p`）：
+    /// 还没打过快照是 0。
+    ///
+    /// # Panics
+    /// 树表里没有码 2 条目：带文件的一版恒有活头 inode 树（发布路径装十二条，重建时 `tree_of(TREE_KIND_INODE)` 判过）。
+    #[must_use]
+    pub fn previous_snapshot_txg_of_the_live_head(&self) -> CheckpointTxg {
+        self.tree_table_entries
+            .iter()
+            .find(|entry| entry.kind == TREE_KIND_INODE)
+            .expect("带文件的一版恒有活头 inode 树那条条目")
+            .previous_snapshot_txg
+    }
+
     /// 树表里某棵树的根指针（照抄没重写的角色时用）。
     #[must_use]
     pub fn tree_root_pointer(&self, tree: u64) -> NodePointer {
@@ -3171,6 +3385,18 @@ impl TransactionOutput {
             .root
     }
 
+    /// 这一版树表里的只读快照（码 14 与配对的码 15），按码 14 的树号升序（`tree_table_entries` 里照盘上原样有它们：发布路径续上的、
+    /// 重建时读回来的）。
+    ///
+    /// # Panics
+    /// 快照条目配不成对：发布路径照上一版原样续、建快照一次追加一对；从盘上重建的版本读树表时判过一一配对（`recovery` 读树表那一步）。
+    #[must_use]
+    pub fn snapshot_entries(&self) -> Vec<SnapshotTreeTableEntries> {
+        snapshot_tree_table_entries_paired_one_to_one(&self.tree_table_entries).expect(
+            "这一版的快照条目是发布路径成对写的、或从盘上读回时判过一一配对的（`tree_table_entries_each_kind_at_most_once`）",
+        )
+    }
+
     /// 树表条目的诞生 txg（十二条同一个数：树建起来那次发布）。
     #[must_use]
     pub fn tree_birth_txg(&self) -> CheckpointTxg {
@@ -3268,9 +3494,31 @@ pub fn placements_to_release_via_mapping(
     previous: &TransactionOutput,
     allocator: &PoolAllocator,
     roles: &[TransactionUnit],
+) -> Result<Vec<Placement>, PublishError> {
+    placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(
+        previous,
+        allocator,
+        roles,
+        &[],
+    )
+}
+
+/// 同 [`placements_to_release_via_mapping`]，`kept_for_the_snapshots` 里的角色（被活快照引用、这次进 deadlist 的那几个，判决
+/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）不释放、不核：它们的落点不进 defer 队列，映射条目留存。
+///
+/// # Errors
+/// 同 [`placements_to_release_via_mapping`]。
+pub fn placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(
+    previous: &TransactionOutput,
+    allocator: &PoolAllocator,
+    roles: &[TransactionUnit],
+    kept_for_the_snapshots: &[TransactionUnit],
 ) -> Result<Vec<Placement>, PublishError> {
     let mut placements = Vec::new();
-    for identity in roles_replaced_via_mapping(previous, roles) {
+    for identity in roles_replaced_via_mapping(previous, roles)
+        .into_iter()
+        .filter(|identity| !kept_for_the_snapshots.contains(identity))
+    {
         let locations = match identity {
             TransactionUnit::Data(_)
             | TransactionUnit::ExtentLowerNode(_)
@@ -3282,7 +3530,10 @@ pub fn placements_to_release_via_mapping(
             | TransactionUnit::AllocationTree
             | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
+            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+            | TransactionUnit::DeadlistTree
             | TransactionUnit::DeletedInodesRoot
+            | TransactionUnit::LoggedOperationsRoot
             | TransactionUnit::TombstoneLeafContainer(_)
             | TransactionUnit::TombstoneRoot
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
@@ -3421,6 +3672,8 @@ fn has_a_placement_in_the_previous_version(
             .is_some(),
         // 删除的两棵树：队列空的一版没有 `deleted_inodes` 根，墓碑树零根的一版没有根与容器（意图那次新写出来的在上一版里没有落点）。
         TransactionUnit::DeletedInodesRoot => !previous.deleted_inodes.is_empty(),
+        // logged_ops：没有条目的一版没有根（签发那次新写出来的在上一版里没有落点）。
+        TransactionUnit::LoggedOperationsRoot => !previous.logged_operations.is_empty(),
         TransactionUnit::TombstoneLeafContainer(index) => {
             index.position() < previous.tombstone_containers.len()
         }
@@ -3473,6 +3726,8 @@ fn has_a_placement_in_the_previous_version(
         | TransactionUnit::AllocationTree
         | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
         | TransactionUnit::AccountingTree
+        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+        | TransactionUnit::DeadlistTree
         | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
         | TransactionUnit::MappingTree
         | TransactionUnit::TreeTable
@@ -3524,9 +3779,11 @@ pub fn central_mapping_locations_in_the_version(
 fn mapped_roles_released_by_this_publish(
     previous: &TransactionOutput,
     roles: &[TransactionUnit],
+    kept_for_the_snapshots: &[TransactionUnit],
 ) -> Vec<TransactionUnit> {
     roles_replaced_via_mapping(previous, roles)
         .into_iter()
+        .filter(|identity| !kept_for_the_snapshots.contains(identity))
         .filter(|identity| match identity {
             TransactionUnit::Data(_)
             | TransactionUnit::ExtentLowerNode(_)
@@ -3538,7 +3795,10 @@ fn mapped_roles_released_by_this_publish(
             | TransactionUnit::AllocationTree
             | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
+            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+            | TransactionUnit::DeadlistTree
             | TransactionUnit::DeletedInodesRoot
+            | TransactionUnit::LoggedOperationsRoot
             | TransactionUnit::TombstoneLeafContainer(_)
             | TransactionUnit::TombstoneRoot
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
@@ -3591,9 +3851,10 @@ fn mapping_locations_of_a_released_unit(
 pub fn refuse_mapping_entries_that_do_not_name_two_pool_devices(
     previous: &TransactionOutput,
     roles: &[TransactionUnit],
+    kept_for_the_snapshots: &[TransactionUnit],
     pool_devices: &[DeviceIdentity],
 ) -> Result<(), PublishError> {
-    for identity in mapped_roles_released_by_this_publish(previous, roles) {
+    for identity in mapped_roles_released_by_this_publish(previous, roles, kept_for_the_snapshots) {
         let locations = mapping_locations_of_a_released_unit(previous, identity);
         refuse_locations_that_do_not_name_two_pool_devices(identity, &locations, pool_devices)?;
     }
@@ -3651,12 +3912,18 @@ pub(crate) fn refuse_locations_that_do_not_name_two_pool_devices(
 pub fn copies_failing_the_release_checksum_check<Reader: PoolReader + ?Sized>(
     previous: &TransactionOutput,
     roles: &[TransactionUnit],
+    kept_for_the_snapshots: &[TransactionUnit],
     reader: &Reader,
 ) -> Result<Vec<CopyQuarantinedAfterReleaseChecksumMismatch>, PublishError> {
     let pool_devices = reader.device_identities();
-    refuse_mapping_entries_that_do_not_name_two_pool_devices(previous, roles, &pool_devices)?;
+    refuse_mapping_entries_that_do_not_name_two_pool_devices(
+        previous,
+        roles,
+        kept_for_the_snapshots,
+        &pool_devices,
+    )?;
     let mut quarantined = Vec::new();
-    for identity in mapped_roles_released_by_this_publish(previous, roles) {
+    for identity in mapped_roles_released_by_this_publish(previous, roles, kept_for_the_snapshots) {
         let locations = mapping_locations_of_a_released_unit(previous, identity);
         quarantined.extend(
             copies_of_a_released_unit_failing_the_release_checksum_check(
@@ -3877,6 +4144,12 @@ pub enum PublishError {
     /// （`admission::admit_the_units_landing_on_every_device`；那一判里不够的每一块报的是短的那一种单元，见它的文档）。准入不够时先推空发布抬 F 再判（D16（发布语义） 已定项 1，C283（准入失败时不先推发布就报 ENOSPC））
     /// 归挂着的会话（`mounted_session`）：直接调发布路径的调用方收到的就是这一次的拒绝。
     SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices),
+    /// 合取 7 `全空聚簇段数 ≥ N₀` 不成立（D26（后台整理与放置回收） 已定项 3 的第二合取式；D3（空间分配） 已定项 7 登记表第 7 行，用户定只拦前台写入类发布）：
+    /// 带着全空聚簇段少于 `N₀` 的每一块盘。与 `SpaceAdmissionRefused` 分开报，因为调用方要做的决定不同：这一条约束的是空间的形状
+    /// （有没有整段空着），不是字节够不够——`df` 可以还有空闲，腾出整段要靠整理（或推抬 F 让 defer 里的段回收），不是删东西
+    /// （`.claude/singlefs-ai-sop/rules/code-discipline.md`「错误」一节：成员按调用方要做的决定分）。在算定这次发布的样子之后、
+    /// 式子那一判、读盘核被换下的单元与动分配器之前返回，一个写都没发、盘上逐字节不变。
+    EmptyClusterSegmentsBelowTheFloor(EmptyClusterSegmentsBelowTheFloorOnSomeDevices),
     /// 记账树或中央映射树这次之后的形状算不出来（`code_two_tree::plan_the_tree_after_this_publish`）：要长到 257 层、
     /// 码 2 头的层级 1 字节写不下（多层之后映射树容量准入剩下的唯一一条，D19（块指针的结构与宽度预算） 已定项 5），
     /// 或上一版（从盘上重建的）树按分隔 key 走不到它自己叶里的一把 key。在动分配器、发任何一个写之前返回，盘上逐字节不变。
@@ -4001,10 +4274,74 @@ pub enum PublishError {
     UnlinkOfAnInodeThatIsNotALiveRecord { inode: u64 },
     /// `deleted_inodes` 根装不下这一版的号（`crate::deleted_inodes`，D8（核心索引结构） 已定项 16 第 6 条：类型化错误、不是空间不够）。
     DeletedInodesQueueRefused(DeletedInodesQueueRefusal),
+    /// 建快照那次发布（[`publish_create_snapshot`]）建不了（[`SnapshotCreationRefusal`]）：在任何读写之前返回，盘上逐字节不变。
+    SnapshotCreationRefused(SnapshotCreationRefusal),
+    /// 删快照的意图或某一批发不了（[`SnapshotDestructionRefusal`]）：在任何读写之前返回，盘上逐字节不变。
+    SnapshotDestructionRefused(SnapshotDestructionRefusal),
+    /// 删快照的一批要放的一条 deadlist 条目放不了（`refusal` 说为什么）：在任何落盘动作之前返回，盘上逐字节不变。
+    DeadlistEntryNotReleasable {
+        mapping_key: Vec<u8>,
+        refusal: DeadlistEntryReleaseRefusal,
+    },
     /// 块设备报的错。落盘阶段报的（带文件的一版、树表 0 条的一版上写行）：这次发布已冻结在分配器上等原样重发。
     BlockDevice(BlockDeviceError),
 }
 
+/// 建快照那次发布（[`publish_create_snapshot`]）为什么建不了。封闭集合，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SnapshotCreationRefusal {
+    /// 上一版树表里已经有 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 个只读快照（码 14 的条数 `snapshots`）：第一版同时活着的至多 10 个
+    /// （用户 2026-10-13 定，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1）。
+    MoreSnapshotsThanTheFirstVersionAllows { snapshots: usize },
+    /// 从上一版的树 ID 水位起连发两个号（快照 inode 根、快照 extent 根）越过 `u64::MAX`：水位是盘上读来的 8 字节，坏镜像、外来镜像才走得到。
+    TreeIdentifierWatermarkLeavesNoRoomForTheSnapshotTrees { tree_identifier_watermark: u64 },
+    /// 上一版里有快照 `being_destroyed` 还在销毁中：删快照串行，删完再接别的改动（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
+    /// 第七节岔路 2）。会话与挂载续做先走空（连销毁中的快照一起），走到这里的是直接调发布函数的调用方。
+    SnapshotBeingDestroyed { being_destroyed: TreeIdentifier },
+}
+
+/// 删快照（[`publish_snapshot_destruction_intent`]、[`publish_snapshot_destruction_step`]）为什么发不了。封闭集合，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SnapshotDestructionRefusal {
+    /// 树表里没有树号是 `snapshot` 的码 14 条目（快照按码 14 的树号点名，判决第七节岔路 5）。
+    NoSnapshotWithThisTree { snapshot: TreeIdentifier },
+    /// 要写意图的快照已经带着「销毁中」：它的意图那次已经发过，接着走批（会话与挂载续做的走空做这件事）。
+    SnapshotAlreadyBeingDestroyed { snapshot: TreeIdentifier },
+    /// 另一个快照 `being_destroyed` 还在销毁中：删快照串行（用户 2026-10-13 定「删完再接别的改动」，判决第七节岔路 2），先把它走完。
+    AnotherSnapshotIsBeingDestroyed { being_destroyed: TreeIdentifier },
+    /// 要走批的快照不带「销毁中」：它的意图还没发。
+    SnapshotNotBeingDestroyed { snapshot: TreeIdentifier },
+}
+
+/// 删快照的一批要放的一条 deadlist 条目为什么放不了。
+#[derive(Debug)]
+pub enum DeadlistEntryReleaseRefusal {
+    /// 它留存的映射条目在上一版里没有（C300（deadlist 条目按 key 查映射查不到没有条款）：第一版不处置）。
+    KeptMappingEntryMissing,
+    /// 它映射 key 的类标签不是数据单元、码 2 节点、码 3 容器之一（盘上读来的一字节）：跨度判不了。
+    UnitClassOutsideTheUserVisibleOnes { unit_class: u8 },
+    /// 落点核不过（两条位置项不同槽、某块盘的记录不在册 / 已释放 / 跨度不对、位置项指池外的盘或两条指同一块盘）：`cause` 原样是
+    /// 释放判定路径的那一种，里面的角色是按类标签取的那一类用户可见角色（[`user_visible_role_standing_for_a_unit_class`]）。
+    PlacementNotReleasable(Box<PublishError>),
+}
+
+/// 一个按映射 key 认的单元（deadlist 条目留存的那几个）按它的类标签归到哪一类用户可见角色上去取跨度、读盘核、报错：数据单元 → 第一个数据单元
+/// （跨度 2）；码 3 容器 → 第一片 inode 叶容器（跨度 2）；码 2 节点 → extent 树根（跨度 1）。六类用户可见角色里同一个类标签的跨度相同
+/// （[`TransactionUnit::span_slots`]），取哪一个都一样；别的类标签交回 `None`。
+#[must_use]
+pub const fn user_visible_role_standing_for_a_unit_class(
+    unit_class: u8,
+) -> Option<TransactionUnit> {
+    match unit_class {
+        UNIT_CLASS_DATA => Some(TransactionUnit::Data(DataUnitIndexInFile::FIRST)),
+        UNIT_CLASS_PACKED => Some(TransactionUnit::InodeLeafContainer(
+            InodeLeafContainerIndexInTree::LEFTMOST,
+        )),
+        UNIT_CLASS_INDEX_NODE => Some(TransactionUnit::ExtentRoot),
+        _other_unit_class => None,
+    }
+}
+
 /// 接在 `version_to_build_on` 后面那一次发布的 checkpoint_txg：它加一（D16（发布语义） 已定项 6）。
 ///
 /// # Errors
@@ -4254,18 +4591,434 @@ pub struct DeletionAndLineageOfThePublish {
     /// 墓碑树与谱系树追加时的容量（写入口上装的只供测试的开关，产品路径按格式）。
     pub container_tree_capacities: ContainerTreeCapacities,
     pub lineage: LineageStepOfThePublish,
+    /// 这次往树表里加不加一个只读快照（只有建快照那次发布加，[`publish_create_snapshot`]）。并在这里而不并进 [`PublishPlan`]，
+    /// 理由同 [`DeletionStepOfThePublish`]。
+    pub snapshot: SnapshotStepOfThePublish,
+    /// 这次往 logged_ops 树里签发还是删一条意图（只有整理的发布做，[`publish_compaction_step`]）。并在这里而不并进 [`PublishPlan`]，
+    /// 理由同 [`DeletionStepOfThePublish`]。
+    pub logged_operations: LoggedOperationsStepOfThePublish,
+}
+
+/// 一次发布对 logged_ops 树做什么（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3；D26（后台整理与放置回收） 已定项 2；
+/// D8（核心索引结构） 已定项 5「删除意图必须与最后一批工作在同一个事务里提交」）：只有整理的发布签发或删意图，别的发布照抄上一版的条目。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum LoggedOperationsStepOfThePublish {
+    CarryTheLoggedOperations,
+    /// 签发：追加这一条整理意图，签发 txg 写这次发布的 txg。上一版已有一条活整理意图时调用方不许走到这里（全池至多一条，判决 G1 e1）。
+    IssueTheCompactionIntent(CompactionIntent),
+    /// 删签发 txg 为 `issued_txg` 的那一条整理意图（意图达成时与最后一批同一次发布，或丢意图时单独一次发布）。
+    DeleteTheCompactionIntent {
+        issued_txg: CheckpointTxg,
+    },
+}
+
+impl LoggedOperationsStepOfThePublish {
+    /// 这一步之后 logged_ops 的条目（按 key 升序）：签发加一条（签发 txg = `txg`）、删掉一条、别的照抄。
+    ///
+    /// # Panics
+    /// 签发时上一版已有一条整理意图、删的那一条不在上一版里：两样都由整理的入口（`publish_compaction_step`）在任何读写之前拒过。
+    #[must_use]
+    pub fn logged_operations_after_this_publish(
+        self,
+        before: &[LoggedOperationEntry],
+        txg: CheckpointTxg,
+    ) -> Vec<LoggedOperationEntry> {
+        let mut entries = before.to_vec();
+        match self {
+            LoggedOperationsStepOfThePublish::CarryTheLoggedOperations => {}
+            LoggedOperationsStepOfThePublish::IssueTheCompactionIntent(intent) => {
+                assert!(
+                    live_compaction_intent_of_the_entries(before).is_none(),
+                    "全池活整理意图至多一条（判决 G1 e1）：签发入口判过上一版没有活意图"
+                );
+                entries.push(LoggedOperationEntry {
+                    issued_txg: txg,
+                    operation: LoggedOperation::Compaction(intent),
+                });
+                entries.sort_by_key(|entry| entry.key().sort_key());
+            }
+            LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent { issued_txg } => {
+                let entries_before = entries.len();
+                entries.retain(|entry| {
+                    !(entry.issued_txg == issued_txg
+                        && entry.operation.kind() == LoggedOperationKind::Compaction)
+                });
+                assert_eq!(
+                    entries.len() + 1,
+                    entries_before,
+                    "删的那一条整理意图在上一版里：删意图入口判过"
+                );
+            }
+        }
+        entries
+    }
 }
 
 impl DeletionAndLineageOfThePublish {
-    /// 既不是删除、也不是管理员回退的发布：墓碑树与谱系树照抄，容量不会被读到。
+    /// 既不是删除、也不是管理员回退、也不是建快照的发布：墓碑树与谱系树照抄，容量不会被读到；树表照上一版的快照条目续上。
     pub const NEITHER_A_DELETION_NOR_A_ROLLBACK: DeletionAndLineageOfThePublish =
         DeletionAndLineageOfThePublish {
             step: DeletionStepOfThePublish::NotADeletion,
             container_tree_capacities: ContainerTreeCapacities::FROM_THE_FORMAT,
             lineage: LineageStepOfThePublish::NoLineageRecord,
+            snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
+            logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
         };
 }
 
+/// 一次发布往树表里加不加一个只读快照（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1、S2）。
+/// 每一次带树表的发布都把上一版树表里的快照条目（码 14、15）原样续在十二条之后（判决 `research/prompts/m4-batch2-code-r1-main-verification.md`
+/// 第二节 S2），只有建快照那次独立发布再追加一对。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SnapshotStepOfThePublish {
+    NoNewSnapshot,
+    /// 建快照那次发布（[`publish_create_snapshot`]）：续上的快照条目之后追加一对——码 14 树号 `snapshot_inode_root_tree`、码 15 树号
+    /// `snapshot_extent_root_tree`（这次从水位连发的两个号，D8（核心索引结构） 已定项 8 ②），字段取值见 [`TREE_KIND_SNAPSHOT_INODE_ROOT`] 与
+    /// [`TREE_KIND_SNAPSHOT_EXTENT_ROOT`]；活头 inode 树条目的 `previous_snapshot_txg` 换成这次的 checkpoint_txg（S.txg，D5（快照 / 空间记账机制）
+    /// 已定项 9、已定项 14）。
+    CreateSnapshot {
+        snapshot_inode_root_tree: TreeIdentifier,
+        snapshot_extent_root_tree: TreeIdentifier,
+    },
+    /// 删快照的意图那次发布（[`publish_snapshot_destruction_intent`]，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4）：
+    /// 码 14 条目 `snapshot` 的 flags 位 0 写成「销毁中」，它的前驱 P 传给那一刻的下一侧（活头 inode 条目或下一条码 14 的 `previous_snapshot_txg`，
+    /// D5（快照 / 空间记账机制） 已定项 12「prev(S) 在写意图的那个事务里传给那一刻的下一侧」）。一条 deadlist 条目都不动。
+    DestructionIntent {
+        snapshot: TreeIdentifier,
+    },
+    /// 删快照意图之后、不是最后一批的一批（[`publish_snapshot_destruction_step`]）：放掉 `snapshot` 那一段里能放的条目
+    /// （纪元落在 [lo, hi) 里、birth > P，[`deadlist_entries_the_destruction_of_a_snapshot_releases`]）按 key 升序的前 `entries_released` 条。
+    DestructionBatch {
+        snapshot: TreeIdentifier,
+        entries_released: usize,
+    },
+    /// 删快照的最后一批：放掉剩下能放的全部条目，同一次发布删掉 `snapshot` 那一对码 14、15 条目。
+    DestructionFinalBatch {
+        snapshot: TreeIdentifier,
+    },
+}
+
+impl SnapshotStepOfThePublish {
+    /// 这一步是不是删快照的三种发布之一（与 unlink 的三种同准入：式子那一判不判、落点那一判照判，判决第七节岔路 4）。
+    #[must_use]
+    pub const fn is_a_snapshot_destruction(self) -> bool {
+        match self {
+            SnapshotStepOfThePublish::NoNewSnapshot
+            | SnapshotStepOfThePublish::CreateSnapshot { .. } => false,
+            SnapshotStepOfThePublish::DestructionIntent { .. }
+            | SnapshotStepOfThePublish::DestructionBatch { .. }
+            | SnapshotStepOfThePublish::DestructionFinalBatch { .. } => true,
+        }
+    }
+}
+
+/// 一个快照在派生链上的下一侧（D5（快照 / 空间记账机制） 已定项 12「下一侧怎么找」：从树表按（诞生 txg, 树号）排的码 14 链现取，零字节）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum NextSideOfASnapshot {
+    /// 下一条码 14（条目原样）。
+    Snapshot(TreeTableEntry),
+    /// 它是最后一个快照：下一侧是活头。
+    LiveHead,
+}
+
+/// `snapshot` 那条码 14 在派生链上的下一侧：树表里的码 14 按（诞生 txg, 树号）升序排，排在它后面的第一条；没有就是活头。
+/// 链上不在乎「销毁中」的记号：第一版删快照串行，带记号的只有正在删的这一条（判决第七节岔路 2）。`snapshot` 不在表里时交回 `None`。
+#[must_use]
+pub fn next_side_of_the_snapshot(
+    tree_table_entries: &[TreeTableEntry],
+    snapshot: TreeIdentifier,
+) -> Option<NextSideOfASnapshot> {
+    let mut chain: Vec<&TreeTableEntry> = tree_table_entries
+        .iter()
+        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
+        .collect();
+    chain.sort_by_key(|entry| (entry.birth_txg, entry.tree));
+    let position = chain.iter().position(|entry| entry.tree == snapshot)?;
+    Some(match chain.get(position + 1) {
+        Some(next) => NextSideOfASnapshot::Snapshot(**next),
+        None => NextSideOfASnapshot::LiveHead,
+    })
+}
+
+/// `snapshot` 那条码 14 条目（树表里按树号找）；不在表里交回 `None`。
+#[must_use]
+pub fn snapshot_inode_root_entry(
+    tree_table_entries: &[TreeTableEntry],
+    snapshot: TreeIdentifier,
+) -> Option<TreeTableEntry> {
+    tree_table_entries
+        .iter()
+        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT && entry.tree == snapshot)
+        .copied()
+}
+
+/// 删快照 `snapshot` 时能放掉的 deadlist 条目，按 key 升序（D5（快照 / 空间记账机制） 已定项 12 定案表「销毁 S」那一行，判决 S4）：
+/// 头是活头 inode 树、纪元落在 [lo, hi) 里、birth > P——lo = 它那条码 14 的诞生 txg，P = 它的前驱，hi 现取：下一侧是快照时取那条码 14 的
+/// 诞生 txg；下一侧是活头时条款取「写意图那次发布的 txg」，它不落盘（判决 S4「续做点不存」）——这里取 `hi_when_the_next_side_is_the_live_head`
+/// （调用方给这一批自己的 txg）：删快照串行（第七节岔路 2），意图之后到最后一批之间的发布只有这次销毁的批与挂载自己的写行、暖机、抬 F，
+/// 都不杀用户可见单元，纪元 ≥ 写意图那次 txg 的条目一条都不会有，两种取法放的是同一组。余下的条目键不动，它们按纪元落进下一侧的那一段
+/// （约定 B，已定项 16：下一侧的前驱在意图那次已经换成 P）。`snapshot` 不在树表里交回空表。
+#[must_use]
+pub fn deadlist_entries_the_destruction_of_a_snapshot_releases(
+    version: &TransactionOutput,
+    snapshot: TreeIdentifier,
+    hi_when_the_next_side_is_the_live_head: CheckpointTxg,
+) -> Vec<DeadlistEntry> {
+    let Some(snapshot_entry) = snapshot_inode_root_entry(&version.tree_table_entries, snapshot)
+    else {
+        return Vec::new();
+    };
+    let lo = snapshot_entry.birth_txg;
+    let predecessor = snapshot_entry.previous_snapshot_txg;
+    let hi = match next_side_of_the_snapshot(&version.tree_table_entries, snapshot) {
+        Some(NextSideOfASnapshot::Snapshot(next)) => next.birth_txg,
+        Some(NextSideOfASnapshot::LiveHead) | None => hi_when_the_next_side_is_the_live_head,
+    };
+    let head = TreeIdentifier(snapshot_entry.head_identifier);
+    version
+        .deadlist_entries
+        .iter()
+        .filter(|entry| {
+            entry.head == head
+                && (lo..hi).contains(&entry.epoch)
+                && birth_txg_of_a_mapping_key(&entry.mapping_key)
+                    .is_some_and(|birth| birth > predecessor)
+        })
+        .cloned()
+        .collect()
+}
+
+/// 这次发布对 deadlist 与留存映射条目做的事（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3、S4）：
+/// 这次换下的用户可见单元里哪几个被活快照引用（进 deadlist、不释放、映射条目留存），删快照这一批放掉哪几条，这次之后的条目与留存的映射条目。
+#[derive(Clone, Debug, Default)]
+pub struct DeadlistChangesOfThisPublish {
+    /// 这次换下、经映射走释放判定的用户可见单元（六类角色，上一版里的角色），按换下的次序：释放的与进 deadlist 的合起来。
+    pub user_visible_roles_replaced: Vec<TransactionUnit>,
+    /// 其中被活快照引用、进 deadlist 不释放的那几个（D5（快照 / 空间记账机制） 已定项 19：birth ≤ 活头的 `previous_snapshot_txg`，或取不到 birth）。
+    pub user_visible_roles_kept_for_the_snapshots: Vec<TransactionUnit>,
+    /// 删快照这一批放掉的条目，连同它们留存的映射条目的两条位置项（释放从这里取落点）。
+    pub entries_released_by_the_destruction: Vec<(DeadlistEntry, [LocationEntry; 2])>,
+    /// 这次之后的 deadlist 条目，按 key 的字段序升序。
+    pub entries_after_this_publish: Vec<DeadlistEntry>,
+    /// 这次之后留存的映射条目，按映射 key 的字段序升序。
+    pub kept_mapping_entries_after_this_publish: Vec<(Vec<u8>, [LocationEntry; 2])>,
+}
+
+/// 这次发布对 deadlist 与留存映射条目做的事（[`DeadlistChangesOfThisPublish`]）。`candidate_roles` 是这次换下的用户可见角色的来源
+/// （这次重写的数据单元、inode 叶容器与根，没有同名后继的那几个，extent 树按计划换下的节点）：按释放判定路径同一个口径展开
+/// （文件变短时尾部的数据单元，`roles_replaced_via_mapping`）、只留六类角色与上一版里有落点的，逐个按映射 key 的出生 txg 与活头的
+/// `previous_snapshot_txg` 判（[`snapshot_lifetime_verdict`]）。进 deadlist 的条目：头 = 活头 inode 树、纪元 = 活头此刻的 `previous_snapshot_txg`
+/// （D5 已定项 12「纪元 = 块被杀那一刻头的 previous_snapshot_txg」）、映射 key 照上一版；它的映射条目照上一版那两条位置项留存。
+/// 删快照的批放掉的条目按 [`deadlist_entries_the_destruction_of_a_snapshot_releases`] 取（最后一批取全部、别的批取前 `entries_released` 条），
+/// 落点取它留存的那条映射条目。没有上一版（第一个文件版本）时什么都不做。
+///
+/// # Errors
+/// 进 deadlist 的单元在上一版的映射里查不到、条目窄于字段表 ⇒ `ReleaseNotInMapping` / `MappingEntryNarrowerThanItsFieldTable`（同释放判定路径）；
+/// 删快照这一批要放的条目在上一版留存的映射条目里没有 ⇒ `DeadlistEntryNotReleasable { refusal: KeptMappingEntryMissing }`
+/// （C300（deadlist 条目按 key 查映射查不到没有条款）：第一版不处置，在任何落盘动作之前拒）。
+fn deadlist_changes_of_this_publish(
+    previous: Option<&TransactionOutput>,
+    trees: &FileVersionTreeIdentifiers,
+    candidate_roles: &[TransactionUnit],
+    snapshot: SnapshotStepOfThePublish,
+    txg: CheckpointTxg,
+) -> Result<DeadlistChangesOfThisPublish, PublishError> {
+    let Some(previous_version) = previous else {
+        return Ok(DeadlistChangesOfThisPublish::default());
+    };
+    let user_visible_roles_replaced: Vec<TransactionUnit> =
+        roles_replaced_via_mapping(previous_version, candidate_roles)
+            .into_iter()
+            .filter(|identity| asks_the_snapshot_lifetime_judgement(*identity))
+            .filter(|identity| has_a_placement_in_the_previous_version(previous_version, *identity))
+            .collect();
+    let previous_snapshot_txg_of_the_live_head =
+        previous_version.previous_snapshot_txg_of_the_live_head();
+    let mut user_visible_roles_kept_for_the_snapshots = Vec::new();
+    let mut entries_killed_this_publish: Vec<DeadlistEntry> = Vec::new();
+    let mut mapping_entries_kept_this_publish: Vec<(Vec<u8>, [LocationEntry; 2])> = Vec::new();
+    for identity in &user_visible_roles_replaced {
+        let mapping_key = mapping_key_carried_from(previous_version, *identity);
+        match snapshot_lifetime_verdict(&mapping_key, previous_snapshot_txg_of_the_live_head) {
+            SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue => {}
+            SnapshotLifetimeVerdict::KeptOnTheDeadlist => {
+                let locations =
+                    match mapping_locations_of_a_mapped_unit(previous_version, *identity) {
+                        MappingLookup::Found(locations) => locations,
+                        MappingLookup::NodeMalformedOrNoEntryWithThisKey => {
+                            return Err(PublishError::ReleaseNotInMapping { unit: *identity })
+                        }
+                        MappingLookup::EntryNarrowerThanItsFieldTable { entry_bytes } => {
+                            return Err(PublishError::MappingEntryNarrowerThanItsFieldTable {
+                                unit: *identity,
+                                entry_bytes,
+                                field_table_bytes: usize::try_from(MAPPING_ENTRY_BYTES)
+                                    .expect("55"),
+                            })
+                        }
+                    };
+                user_visible_roles_kept_for_the_snapshots.push(*identity);
+                entries_killed_this_publish.push(DeadlistEntry {
+                    head: trees.inode,
+                    epoch: previous_snapshot_txg_of_the_live_head,
+                    mapping_key: mapping_key.clone(),
+                });
+                mapping_entries_kept_this_publish.push((mapping_key, locations));
+            }
+        }
+    }
+    let entries_released: Vec<DeadlistEntry> = match snapshot {
+        SnapshotStepOfThePublish::DestructionBatch {
+            snapshot,
+            entries_released,
+        } => {
+            deadlist_entries_the_destruction_of_a_snapshot_releases(previous_version, snapshot, txg)
+                .into_iter()
+                .take(entries_released)
+                .collect()
+        }
+        SnapshotStepOfThePublish::DestructionFinalBatch { snapshot } => {
+            deadlist_entries_the_destruction_of_a_snapshot_releases(previous_version, snapshot, txg)
+        }
+        SnapshotStepOfThePublish::NoNewSnapshot
+        | SnapshotStepOfThePublish::CreateSnapshot { .. }
+        | SnapshotStepOfThePublish::DestructionIntent { .. } => Vec::new(),
+    };
+    let mut entries_released_by_the_destruction = Vec::with_capacity(entries_released.len());
+    for entry in entries_released {
+        let locations = previous_version
+            .mapping_entries_kept_for_the_snapshots
+            .iter()
+            .find(|(key, _)| *key == entry.mapping_key)
+            .map(|(_, locations)| *locations)
+            .ok_or_else(|| PublishError::DeadlistEntryNotReleasable {
+                mapping_key: entry.mapping_key.clone(),
+                refusal: DeadlistEntryReleaseRefusal::KeptMappingEntryMissing,
+            })?;
+        entries_released_by_the_destruction.push((entry, locations));
+    }
+    let mut entries_after: BTreeMap<CodeTwoTreeKey, DeadlistEntry> = previous_version
+        .deadlist_entries
+        .iter()
+        .map(|entry| {
+            (
+                CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::DEADLIST),
+                entry.clone(),
+            )
+        })
+        .collect();
+    let mut kept_mapping_after: BTreeMap<CodeTwoTreeKey, (Vec<u8>, [LocationEntry; 2])> =
+        previous_version
+            .mapping_entries_kept_for_the_snapshots
+            .iter()
+            .map(|(key, locations)| {
+                (
+                    CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
+                    (key.clone(), *locations),
+                )
+            })
+            .collect();
+    for (entry, _) in &entries_released_by_the_destruction {
+        entries_after.remove(&CodeTwoTreeKey::new(
+            &entry.key_bytes(),
+            CodeTwoKeyFieldWidths::DEADLIST,
+        ));
+        kept_mapping_after.remove(&CodeTwoTreeKey::new(
+            &entry.mapping_key,
+            CodeTwoKeyFieldWidths::CENTRAL_MAPPING,
+        ));
+    }
+    for entry in entries_killed_this_publish {
+        let previous_entry = entries_after.insert(
+            CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::DEADLIST),
+            entry,
+        );
+        assert!(
+            previous_entry.is_none(),
+            "一个单元只被杀一次：这次进 deadlist 的条目在上一版的 deadlist 里没有（它的映射 key 上一版还被这一版的树引用）"
+        );
+    }
+    for (key, locations) in mapping_entries_kept_this_publish {
+        kept_mapping_after.insert(
+            CodeTwoTreeKey::new(&key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
+            (key, locations),
+        );
+    }
+    Ok(DeadlistChangesOfThisPublish {
+        user_visible_roles_replaced,
+        user_visible_roles_kept_for_the_snapshots,
+        entries_released_by_the_destruction,
+        entries_after_this_publish: entries_after.into_values().collect(),
+        kept_mapping_entries_after_this_publish: kept_mapping_after.into_values().collect(),
+    })
+}
+
+/// 六类用户可见子树的角色（判决 S3：只有它们问 D5（快照 / 空间记账机制） 已定项 19 的判定，其余 14 类一律直接进 defer）。
+#[must_use]
+pub const fn asks_the_snapshot_lifetime_judgement(identity: TransactionUnit) -> bool {
+    match identity {
+        TransactionUnit::Data(_)
+        | TransactionUnit::ExtentLowerNode(_)
+        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+        | TransactionUnit::ExtentRoot
+        | TransactionUnit::InodeLeafContainer(_)
+        | TransactionUnit::InodeRoot => true,
+        TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
+        | TransactionUnit::AllocationTree
+        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+        | TransactionUnit::AccountingTree
+        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+        | TransactionUnit::DeadlistTree
+        | TransactionUnit::DeletedInodesRoot
+        | TransactionUnit::LoggedOperationsRoot
+        | TransactionUnit::TombstoneLeafContainer(_)
+        | TransactionUnit::TombstoneRoot
+        | TransactionUnit::TombstoneNodeBelowTheRoot(_)
+        | TransactionUnit::LineageLeafContainer(_)
+        | TransactionUnit::LineageRoot
+        | TransactionUnit::LineageNodeBelowTheRoot(_)
+        | TransactionUnit::RetirementLeafContainer(_)
+        | TransactionUnit::RetirementRoot
+        | TransactionUnit::RetirementNodeBelowTheRoot(_)
+        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
+        | TransactionUnit::MappingTree
+        | TransactionUnit::TreeTable
+        | TransactionUnit::InstanceTable
+        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
+    }
+}
+
+/// D5（快照 / 空间记账机制） 已定项 19 的判定（第一版不带「n > 1」那一臂：稀疏旁表恒 0 条，判决 S9）：birth > p ⇒ 立即释放（进 defer）；
+/// 否则（含 birth 取不到，已定项 14「判据的输入取不到时默认进 deadlist」）⇒ 进 deadlist。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SnapshotLifetimeVerdict {
+    ReleasedIntoTheDeferQueue,
+    KeptOnTheDeadlist,
+}
+
+/// 一个被杀的单元按它映射 key 里的出生 txg 与活头的 `previous_snapshot_txg` 判（[`SnapshotLifetimeVerdict`]）。
+#[must_use]
+pub fn snapshot_lifetime_verdict(
+    mapping_key: &[u8],
+    previous_snapshot_txg_of_the_live_head: CheckpointTxg,
+) -> SnapshotLifetimeVerdict {
+    match birth_txg_of_a_mapping_key(mapping_key) {
+        Some(birth) if birth > previous_snapshot_txg_of_the_live_head => {
+            SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue
+        }
+        Some(_) | None => SnapshotLifetimeVerdict::KeptOnTheDeadlist,
+    }
+}
+
+/// 这次之后 logged_ops 根从哪来（同 [`DeletedInodesRootOrigin`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum LoggedOperationsRootOrigin {
+    /// 没有条目：树表条目的根指针置零，这一版没有这个角色。
+    NoRoot,
+    CarriedFromThePreviousVersion,
+    RewrittenThisPublish,
+}
+
 /// 这次之后 `deleted_inodes` 根从哪来。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum DeletedInodesRootOrigin {
@@ -4562,8 +5315,13 @@ pub struct ResolvedPublish {
     pub allocation_record_tree: AllocationRecordTreePlan,
     /// 这次之后记账树长什么样（D8（核心索引结构） 已定项 11 的分裂与收缩）。
     pub accounting_tree: CodeTwoTreePlan,
-    /// 这次之后中央映射树长什么样，同上；按下面那一份映射 key 算。
+    /// 这次之后中央映射树长什么样，同上；按下面那一份映射 key 与留存的映射条目（`deadlist.kept_mapping_entries_after_this_publish`）算。
     pub central_mapping_tree: CodeTwoTreePlan,
+    /// 这次之后 deadlist 树长什么样（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：按 `deadlist.entries_after_this_publish` 算，
+    /// 一条都不剩时零根。
+    pub deadlist_tree: CodeTwoTreePlan,
+    /// 这次发布对 deadlist 与留存映射条目做的事。
+    pub deadlist: DeadlistChangesOfThisPublish,
     /// 这一版每个进映射的单元与它的映射 key，按 bump 次序（每个数据单元、extent 根、每片 inode 叶容器、inode 根、分配记录树、
     /// 记账树的每个节点）：这次重写的按这次要发的出生身份现算（出生序号按 bump 次序发，D19（块指针的结构与宽度预算） 已定项 9），
     /// 照抄的取上一版的。装映射节点那一段（`publish_admitted`）核装出来的与它逐项相等。
@@ -4580,6 +5338,9 @@ pub struct ResolvedPublish {
     pub lineage_tree: LineageTreeAfterThisPublish,
     /// 这次之后退役树的样子（有 inode 叶容器身份消失的那次追加，别的照抄）。
     pub retirement_tree: RetirementTreeAfterThisPublish,
+    /// 这次之后 logged_ops 的条目（按 key 升序）与它的根从哪来。
+    pub logged_operations: Vec<LoggedOperationEntry>,
+    pub logged_operations_root: LoggedOperationsRootOrigin,
     /// 上一版里有、这一版没有同一个单元接着的角色（删除批截掉的尾部数据单元、身份消失的 inode 叶容器——按它在上一版里的叶序、
     /// 删空的 `deleted_inodes` 根），照样被这次换下。
     pub previous_roles_without_a_successor: Vec<TransactionUnit>,
@@ -4610,6 +5371,7 @@ impl ResolvedPublish {
         match tree {
             MultiLevelCodeTwoTree::Accounting => &self.accounting_tree,
             MultiLevelCodeTwoTree::CentralMapping => &self.central_mapping_tree,
+            MultiLevelCodeTwoTree::Deadlist => &self.deadlist_tree,
         }
     }
 
@@ -4692,6 +5454,7 @@ impl ResolvedPublish {
         );
         for tree in [
             MultiLevelCodeTwoTree::Accounting,
+            MultiLevelCodeTwoTree::Deadlist,
             MultiLevelCodeTwoTree::CentralMapping,
         ] {
             let previous_shape = &previous.multi_level_tree(tree).shape;
@@ -4719,6 +5482,9 @@ pub const fn multi_level_tree_of_role(identity: TransactionUnit) -> Option<Multi
         TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
             Some(MultiLevelCodeTwoTree::CentralMapping)
         }
+        TransactionUnit::DeadlistTreeNodeBelowTheRoot(_) | TransactionUnit::DeadlistTree => {
+            Some(MultiLevelCodeTwoTree::Deadlist)
+        }
         TransactionUnit::Data(_)
         | TransactionUnit::ExtentLowerNode(_)
         | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
@@ -4728,6 +5494,7 @@ pub const fn multi_level_tree_of_role(identity: TransactionUnit) -> Option<Multi
         | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
         | TransactionUnit::AllocationTree
         | TransactionUnit::DeletedInodesRoot
+        | TransactionUnit::LoggedOperationsRoot
         | TransactionUnit::TombstoneLeafContainer(_)
         | TransactionUnit::TombstoneRoot
         | TransactionUnit::TombstoneNodeBelowTheRoot(_)
@@ -4774,7 +5541,10 @@ pub const fn allocation_record_tree_node_of_role(
         | TransactionUnit::InodeRoot
         | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
         | TransactionUnit::AccountingTree
+        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+        | TransactionUnit::DeadlistTree
         | TransactionUnit::DeletedInodesRoot
+        | TransactionUnit::LoggedOperationsRoot
         | TransactionUnit::TombstoneLeafContainer(_)
         | TransactionUnit::TombstoneRoot
         | TransactionUnit::TombstoneNodeBelowTheRoot(_)
@@ -5142,12 +5912,14 @@ struct DeletionTreesOfThisPublish<'plans> {
     deleted_inodes_root: DeletedInodesRootOrigin,
     tombstone_tree: &'plans TombstoneTreeAfterThisPublish,
     lineage_tree: &'plans LineageTreeAfterThisPublish,
+    logged_operations_root: LoggedOperationsRootOrigin,
     retirement_tree: &'plans RetirementTreeAfterThisPublish,
 }
 
 /// 这张计划要写的记账树与中央映射树，在一切落盘动作之前算完：记账树按这一版的记账 key，中央映射树按这一版每个进映射单元的 key。
 struct MultiLevelTreesOfThisPublish {
     accounting_tree: CodeTwoTreePlan,
+    deadlist_tree: CodeTwoTreePlan,
     central_mapping_tree: CodeTwoTreePlan,
     mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
 }
@@ -5167,6 +5939,9 @@ struct OrdinaryAllocationPartOfThePublish {
     tombstone_tree: TombstoneTreeAfterThisPublish,
     /// 这次之后退役树的样子：有 inode 叶容器身份消失（删空摘掉、被左邻吸收）的那次往开放容器追加，别的照抄。
     retirement_tree: RetirementTreeAfterThisPublish,
+    /// 这次之后 logged_ops 的条目（按 key 升序）与根从哪来。
+    logged_operations: Vec<LoggedOperationEntry>,
+    logged_operations_root: LoggedOperationsRootOrigin,
 }
 
 impl OrdinaryAllocationPartOfThePublish {
@@ -5189,10 +5964,23 @@ impl OrdinaryAllocationPartOfThePublish {
             roles.push(TransactionUnit::InodeRoot);
         }
         roles.extend(self.deletion_roles_rewritten());
+        roles.extend(self.logged_operations_roles_rewritten());
         roles.extend(self.retirement_roles_rewritten());
         roles
     }
 
+    /// logged_ops 这次重写的角色：普通分配（同 `deleted_inodes` 根，只在整理签发、删意图的那次发布重写），按树 ID 排在谱系树之后、
+    /// 退役树之前（D3（空间分配） 已定项 10 ⑤）。
+    fn logged_operations_roles_rewritten(&self) -> Vec<TransactionUnit> {
+        match self.logged_operations_root {
+            LoggedOperationsRootOrigin::RewrittenThisPublish => {
+                vec![TransactionUnit::LoggedOperationsRoot]
+            }
+            LoggedOperationsRootOrigin::NoRoot
+            | LoggedOperationsRootOrigin::CarriedFromThePreviousVersion => Vec::new(),
+        }
+    }
+
     /// 退役树这次重写的角色（树内先叶后根）：普通分配（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条，同墓碑），
     /// 按树 ID 排在谱系树之后、中央映射树之前（D3（空间分配） 已定项 10 ⑤）。
     fn retirement_roles_rewritten(&self) -> Vec<TransactionUnit> {
@@ -5255,6 +6043,12 @@ impl OrdinaryAllocationPartOfThePublish {
         {
             roles.push(TransactionUnit::DeletedInodesRoot);
         }
+        // 删光意图之后的 logged_ops 根（意图达成或丢掉的那一次）：这一版没有它，上一版的那一个照样换下、经映射释放。
+        if self.logged_operations_root == LoggedOperationsRootOrigin::NoRoot
+            && !previous.logged_operations.is_empty()
+        {
+            roles.push(TransactionUnit::LoggedOperationsRoot);
+        }
         roles
     }
 }
@@ -5441,6 +6235,21 @@ impl PublishPlan<'_> {
                 .retirement_container,
             deletion_and_lineage.container_tree_capacities.node_entries,
         );
+        // logged_ops（判决 G1）：整理的发布签发一条或删一条，别的照抄；装不下一个根走不到（全池活整理意图至多一条）。
+        let logged_operations_before: &[LoggedOperationEntry] = match previous {
+            Some(version) => &version.logged_operations,
+            None => &[],
+        };
+        let logged_operations = deletion_and_lineage
+            .logged_operations
+            .logged_operations_after_this_publish(logged_operations_before, self.txg);
+        let logged_operations_root = if logged_operations.is_empty() {
+            LoggedOperationsRootOrigin::NoRoot
+        } else if logged_operations == logged_operations_before {
+            LoggedOperationsRootOrigin::CarriedFromThePreviousVersion
+        } else {
+            LoggedOperationsRootOrigin::RewrittenThisPublish
+        };
         Ok(OrdinaryAllocationPartOfThePublish {
             inode_tree,
             extent_tree,
@@ -5450,6 +6259,8 @@ impl PublishPlan<'_> {
             deleted_inodes_root,
             tombstone_tree,
             retirement_tree,
+            logged_operations,
+            logged_operations_root,
         })
     }
 
@@ -5458,11 +6269,16 @@ impl PublishPlan<'_> {
     /// `device_identities` 是记账行按盘展开的那几块盘（与装记账行时的分配器同一批），`capacities` 是两棵多层码 2 树的节点容量
     /// （写入口上那个只供测试的开关，产品路径按节点格式算）。`allocation_record_tree` 是这次之后分配记录树的样子：它重写哪几个节点
     /// 要在分配器的拷贝上走到固定点才知道（取落点会改记录），由调用方定下来交进来（`settle_the_allocation_record_tree`），
-    /// 这里按它排角色、发映射 key。
+    /// 这里按它排角色、发映射 key。`mapping_keys_moved_by_this_publish` 是整理的一批搬走的单元的映射 key（别的发布给空的）：
+    /// 它们 key 不变、条目的位置换了，中央映射树里装着它们的叶连同到根的路这次重写（[`plan_rewriting_the_paths_to_the_moved_entries`]）。
     ///
     /// # Errors
     /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）；
     /// 记账树或中央映射树这次之后的形状算不出来 ⇒ `MultiLevelCodeTwoTreeRefused`。
+    #[allow(
+        clippy::too_many_arguments,
+        reason = "参数各是一样东西：计划、上一版、树号、记账行的盘、节点容量、分配记录树、删除这一步、这次搬走的映射 key"
+    )]
     pub fn resolve(
         &self,
         previous: Option<&TransactionOutput>,
@@ -5471,9 +6287,11 @@ impl PublishPlan<'_> {
         capacities: CodeTwoTreeNodeCapacities,
         allocation_record_tree: AllocationRecordTreePlan,
         deletion_and_lineage: &DeletionAndLineageOfThePublish,
+        mapping_keys_moved_by_this_publish: &[Vec<u8>],
     ) -> Result<ResolvedPublish, PublishError> {
         let ordinary_part = self.ordinary_allocation_part(previous, trees, deletion_and_lineage)?;
         let deletion_roles_rewritten = ordinary_part.deletion_roles_rewritten();
+        let logged_operations_roles_rewritten = ordinary_part.logged_operations_roles_rewritten();
         let retirement_roles_rewritten = ordinary_part.retirement_roles_rewritten();
         let lineage_tree =
             self.lineage_tree_after_this_publish(previous, trees, deletion_and_lineage);
@@ -5492,6 +6310,8 @@ impl PublishPlan<'_> {
             deleted_inodes_root,
             tombstone_tree,
             retirement_tree,
+            logged_operations,
+            logged_operations_root,
         } = ordinary_part;
         let mut rewritten_roles: Vec<TransactionUnit> = file_content_transactions
             .iter()
@@ -5509,18 +6329,50 @@ impl PublishPlan<'_> {
         if inode_tree.is_the_root_rewritten() {
             rewritten_roles.push(TransactionUnit::InodeRoot);
         }
-        rewritten_roles.extend(
-            allocation_record_tree
-                .rewritten_nodes()
-                .into_iter()
-                .map(role_of_allocation_record_tree_node),
-        );
-        let MultiLevelTreesOfThisPublish {
-            accounting_tree,
-            central_mapping_tree,
-            mapped_units,
-        } = self.plan_the_multi_level_trees(
-            previous,
+        // 释放判定分流（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：这次换下的用户可见单元——这次重写的数据单元、
+        // inode 叶容器与根，没有同名后继的那几个，extent 树按计划换下的节点——逐个问 D5（快照 / 空间记账机制） 已定项 19 的判定，
+        // 被活快照引用的进 deadlist、不释放、映射条目留存；删快照这一批放掉的条目也在这里定。要在中央映射树与 deadlist 树规划之前定：
+        // 两棵树的 key 都要它。释放判定路径（`placements_released_by_this_publish`）核它与按全部换下角色走出来的那一份相同。
+        let user_visible_roles_rewritten_or_without_a_successor: Vec<TransactionUnit> =
+            rewritten_roles
+                .iter()
+                .copied()
+                .filter(|identity| {
+                    matches!(
+                        identity,
+                        TransactionUnit::Data(_)
+                            | TransactionUnit::InodeLeafContainer(_)
+                            | TransactionUnit::InodeRoot
+                    )
+                })
+                .chain(
+                    previous_roles_without_a_successor
+                        .iter()
+                        .copied()
+                        .filter(|identity| asks_the_snapshot_lifetime_judgement(*identity)),
+                )
+                .chain(extent_tree.replaced_previous_roles.iter().copied())
+                .collect();
+        let deadlist = deadlist_changes_of_this_publish(
+            previous,
+            trees,
+            &user_visible_roles_rewritten_or_without_a_successor,
+            deletion_and_lineage.snapshot,
+            self.txg,
+        )?;
+        rewritten_roles.extend(
+            allocation_record_tree
+                .rewritten_nodes()
+                .into_iter()
+                .map(role_of_allocation_record_tree_node),
+        );
+        let MultiLevelTreesOfThisPublish {
+            accounting_tree,
+            deadlist_tree,
+            central_mapping_tree,
+            mapped_units,
+        } = self.plan_the_multi_level_trees(
+            previous,
             trees,
             &MappedPositionAddressedTrees {
                 extent: &extent_tree,
@@ -5532,10 +6384,13 @@ impl PublishPlan<'_> {
                 deleted_inodes_root,
                 tombstone_tree: &tombstone_tree,
                 lineage_tree: &lineage_tree,
+                logged_operations_root,
                 retirement_tree: &retirement_tree,
             },
+            &deadlist,
             device_identities,
             capacities,
+            mapping_keys_moved_by_this_publish,
         )?;
         // 树内先叶后根、同层按 key 升序（D3（空间分配） 已定项 10 ⑤），记账树（树 65539）在前、删除的两棵（65544、65545）接着、
         // 谱系树（65546）再接着、中央映射树倒数第二、树表最末。谱系树的节点归固定点（D28（挂载期承诺量） 已定项 1 接线「谱系容器归固定点、
@@ -5548,11 +6403,22 @@ impl PublishPlan<'_> {
                     MultiLevelCodeTwoTree::Accounting.role_of_node(position, &accounting_tree.shape)
                 }),
         );
+        // deadlist 树（树 65543）按树 ID 排在记账树之后、删除那两棵树之前。
+        rewritten_roles.extend(
+            deadlist_tree
+                .rewritten_positions()
+                .into_iter()
+                .map(|position| {
+                    MultiLevelCodeTwoTree::Deadlist.role_of_node(position, &deadlist_tree.shape)
+                }),
+        );
         rewritten_roles.extend(deletion_roles_rewritten);
         rewritten_roles.extend(container_tree_roles_rewritten(
             &lineage_tree,
             ContainerTreeRoles::OF_THE_LINEAGE_TREE,
         ));
+        // logged_ops 根（树 ID 接在谱系树之后，判决 G1）：普通分配，按树 ID 排在谱系树之后、退役树之前。
+        rewritten_roles.extend(logged_operations_roles_rewritten);
         // 退役树（树 ID 接在 logged_ops 之后，判决 m3-retire-r1 第三节第 1 条）：普通分配，按树 ID 排在谱系树之后。
         rewritten_roles.extend(retirement_roles_rewritten);
         rewritten_roles.extend(central_mapping_tree.rewritten_positions().into_iter().map(
@@ -5568,6 +6434,8 @@ impl PublishPlan<'_> {
             allocation_record_tree,
             accounting_tree,
             central_mapping_tree,
+            deadlist_tree,
+            deadlist,
             mapped_units,
             rewritten_roles,
             carried_data_units,
@@ -5576,6 +6444,8 @@ impl PublishPlan<'_> {
             tombstone_tree,
             lineage_tree,
             retirement_tree,
+            logged_operations,
+            logged_operations_root,
             previous_roles_without_a_successor,
         })
     }
@@ -5622,7 +6492,8 @@ impl PublishPlan<'_> {
     /// 记账树与中央映射树这次之后的形状，与这一版每个进映射单元的映射 key（`resolve` 的后一半）。
     #[allow(
         clippy::too_many_arguments,
-        reason = "参数各是一样东西：计划、上一版、树号、两棵按位置寻址的树、inode 树、删除那两棵树、记账行的盘、节点容量"
+        reason = "参数各是一样东西：计划、上一版、树号、两棵按位置寻址的树、inode 树、删除那两棵树、deadlist 的改动、记账行的盘、节点容量、\
+                  这次搬走的映射 key"
     )]
     fn plan_the_multi_level_trees(
         &self,
@@ -5631,8 +6502,10 @@ impl PublishPlan<'_> {
         position_addressed_trees: &MappedPositionAddressedTrees<'_>,
         inode_tree: &InodeLeafContainersAfterThisPublish,
         deletion_trees: &DeletionTreesOfThisPublish<'_>,
+        deadlist: &DeadlistChangesOfThisPublish,
         device_identities: &[DeviceIdentity],
         capacities: CodeTwoTreeNodeCapacities,
+        mapping_keys_moved_by_this_publish: &[Vec<u8>],
     ) -> Result<MultiLevelTreesOfThisPublish, PublishError> {
         let txg = self.txg;
         let instance = self.instance;
@@ -5667,6 +6540,18 @@ impl PublishPlan<'_> {
             capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
         )
         .map_err(refused(MultiLevelCodeTwoTree::Accounting))?;
+        // deadlist 树（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：按这次之后的条目规划，一条都不剩时零根。
+        let deadlist_keys: BTreeSet<CodeTwoTreeKey> = deadlist
+            .entries_after_this_publish
+            .iter()
+            .map(|entry| CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::DEADLIST))
+            .collect();
+        let deadlist_tree = plan_the_tree_after_this_publish_down_to_a_zero_root(
+            previous_shape_of(MultiLevelCodeTwoTree::Deadlist),
+            &deadlist_keys,
+            capacities.of_tree(MultiLevelCodeTwoTree::Deadlist),
+        )
+        .map_err(refused(MultiLevelCodeTwoTree::Deadlist))?;
 
         // 这一版每个进映射的单元的 key：重写的现算，照抄的取上一版的。出生序号在各自那棵树里按 bump 次序从 0 发（数据单元不发，
         // 码 1 的 key 带写序）：extent 树下段先叶后根、再上段先叶后根，inode 树先叶后根，分配记录树按位置先叶后根。
@@ -5849,6 +6734,36 @@ impl PublishPlan<'_> {
                 }
             }
         }
+        // deadlist 树的每个节点都进映射（码 2 树节点，同记账树）：按 bump 次序，记账树之后、删除那两棵树之前（按树 ID）。
+        let previous_deadlist_shape = previous_shape_of(MultiLevelCodeTwoTree::Deadlist);
+        let mut deadlist_birth_sequence = 0;
+        for (node, origin) in deadlist_tree
+            .shape
+            .nodes()
+            .iter()
+            .zip(&deadlist_tree.origins)
+        {
+            let identity =
+                MultiLevelCodeTwoTree::Deadlist.role_of_node(node.position, &deadlist_tree.shape);
+            match origin {
+                CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
+                    mapped_units.push((
+                        identity,
+                        node_key(
+                            UNIT_CLASS_INDEX_NODE,
+                            trees.deadlist,
+                            deadlist_birth_sequence,
+                        ),
+                    ));
+                    deadlist_birth_sequence += 1;
+                }
+                CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => {
+                    let previous_identity = MultiLevelCodeTwoTree::Deadlist
+                        .role_of_node(*previous_position, previous_deadlist_shape);
+                    mapped_units.push((identity, carried(previous_identity)));
+                }
+            }
+        }
         // 删除那两棵树（记账树之后、按树 ID）：`deleted_inodes` 根（队列非空时），墓碑树的每片容器先、根后。
         match deletion_trees.deleted_inodes_root {
             DeletedInodesRootOrigin::NoRoot => {}
@@ -5874,35 +6789,123 @@ impl PublishPlan<'_> {
             &|unit_class, birth_sequence| node_key(unit_class, trees.lineage, birth_sequence),
             &carried,
         ));
+        // logged_ops 根（谱系树之后、退役树之前，按树 ID）：有条目时一个码 2 根。
+        match deletion_trees.logged_operations_root {
+            LoggedOperationsRootOrigin::NoRoot => {}
+            LoggedOperationsRootOrigin::CarriedFromThePreviousVersion => mapped_units.push((
+                TransactionUnit::LoggedOperationsRoot,
+                carried(TransactionUnit::LoggedOperationsRoot),
+            )),
+            LoggedOperationsRootOrigin::RewrittenThisPublish => mapped_units.push((
+                TransactionUnit::LoggedOperationsRoot,
+                node_key(UNIT_CLASS_INDEX_NODE, trees.logged_operations, 0),
+            )),
+        }
         mapped_units.extend(mapped_units_of_a_container_tree(
             deletion_trees.retirement_tree,
             ContainerTreeRoles::OF_THE_RETIREMENT_TREE,
             &|unit_class, birth_sequence| node_key(unit_class, trees.retirement, birth_sequence),
             &carried,
         ));
+        // 被活快照钉住的单元的映射条目留在树里（判决 S3 的 α）：它们的 key 与这一版的单元的 key 一起排。
         let mapping_keys: BTreeSet<CodeTwoTreeKey> = mapped_units
             .iter()
-            .map(|(_, key)| CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING))
+            .map(|(_, key)| key)
+            .chain(
+                deadlist
+                    .kept_mapping_entries_after_this_publish
+                    .iter()
+                    .map(|(key, _)| key),
+            )
+            .map(|key| CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING))
             .collect();
         assert_eq!(
             mapping_keys.len(),
-            mapped_units.len(),
-            "进映射的单元各一把 key，互不相撞（D19（块指针的结构与宽度预算） 已定项 6：出生身份各不相同）"
+            mapped_units.len() + deadlist.kept_mapping_entries_after_this_publish.len(),
+            "进映射的单元与留存的映射条目各一把 key，互不相撞（D19（块指针的结构与宽度预算） 已定项 6：出生身份各不相同；留存的是上一版里\
+             被杀的单元的 key，这一版的单元不再引用它们）"
         );
-        let central_mapping_tree = plan_the_tree_after_this_publish(
+        let central_mapping_tree = plan_rewriting_the_paths_to_the_moved_entries(
+            plan_the_tree_after_this_publish(
+                previous_shape_of(MultiLevelCodeTwoTree::CentralMapping),
+                &mapping_keys,
+                capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
+            )
+            .map_err(refused(MultiLevelCodeTwoTree::CentralMapping))?,
             previous_shape_of(MultiLevelCodeTwoTree::CentralMapping),
-            &mapping_keys,
-            capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
-        )
-        .map_err(refused(MultiLevelCodeTwoTree::CentralMapping))?;
+            mapping_keys_moved_by_this_publish
+                .iter()
+                .map(|key| CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING))
+                .filter(|key| mapping_keys.contains(key)),
+        );
         Ok(MultiLevelTreesOfThisPublish {
             accounting_tree,
+            deadlist_tree,
             central_mapping_tree,
             mapped_units,
         })
     }
 }
 
+/// 整理的一批换了位置的映射条目（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「映射条目换位置」）：`plan` 里装着
+/// `moved_keys` 每一把的那片叶、连同从根下来那条路上的每个节点，都改成这次重写（D8（核心索引结构） 已定项 11 ①「路径上的节点整条 COW」）。
+/// [`plan_the_tree_after_this_publish`] 只按 key 进出定哪些节点重写；搬迁的 key 不变、条目的位置变了，不在这里标，装着它的叶就照抄上一版、
+/// 盘上的条目留在已经释放的源落点，重建按映射覆盖提示时把单元指回源落点。形状不变，只改节点的来处；上一版换下的节点按新的来处重算。
+/// `moved_keys` 里每一把都在这一版的 key 里（调用方按这一版的 key 集合筛过）。
+fn plan_rewriting_the_paths_to_the_moved_entries(
+    mut plan: CodeTwoTreePlan,
+    previous_shape: &CodeTwoTreeShape,
+    moved_keys: impl Iterator<Item = CodeTwoTreeKey>,
+) -> CodeTwoTreePlan {
+    for moved_key in moved_keys {
+        let mut position = plan
+            .shape
+            .root()
+            .expect("这一版的中央映射树装着搬走的这把 key：有根")
+            .position;
+        // 迭代上界：每一轮下降一层，至多树高那么多轮；走到叶就停。路由同 `leaf_position_routed_to`。
+        loop {
+            let index = plan
+                .shape
+                .nodes()
+                .binary_search_by_key(&position, |node| node.position)
+                .expect("位置取自同一份形状里的子引用");
+            plan.origins[index] = CodeTwoTreeNodeOrigin::RewrittenThisPublish;
+            match &plan.shape.nodes()[index].contents {
+                CodeTwoTreeNodeContents::Leaf { keys } => {
+                    assert!(
+                        keys.contains(&moved_key),
+                        "按分隔 key 走到的叶装着这把 key：调用方只交这一版里有的 key，规划出来的形状按分隔 key 路由"
+                    );
+                    break;
+                }
+                CodeTwoTreeNodeContents::Internal { children } => {
+                    let child_index = children
+                        .iter()
+                        .rposition(|child| child.separator_key <= moved_key)
+                        .unwrap_or(0);
+                    position = children[child_index].child;
+                }
+            }
+        }
+    }
+    let carried: BTreeSet<CodeTwoTreeNodePosition> = plan
+        .origins
+        .iter()
+        .filter_map(|origin| match origin {
+            CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => Some(*previous_position),
+            CodeTwoTreeNodeOrigin::RewrittenThisPublish => None,
+        })
+        .collect();
+    plan.replaced_previous_nodes = previous_shape
+        .nodes()
+        .iter()
+        .map(|node| node.position)
+        .filter(|position| !carried.contains(position))
+        .collect();
+    plan
+}
+
 /// 文件版本那条 inode 记录（D8（核心索引结构） 已定项 6 的字段表）：size = 这次写的内容长度；
 /// blocks 由 size 现算（⌈size ÷ 512⌉，[`InodeRecord::to_bytes`]），不按这个对象分到几个数据单元填。
 fn inode_record_of_file_version(file: &FileVersionPlan<'_>) -> InodeRecord {
@@ -6202,6 +7205,526 @@ pub fn publish_new_inodes<Device: BlockDevice>(
     )
 }
 
+/// 建一个只读快照那次发布（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1、S2）：**独立发布**，不与别的用户改动合在一个事务里，
+/// 接在 `previous` 后面；文件内容与 inode 树都不动（`file` 空、不建 inode），只重写固定点角色与树表。S.txg := 这次发布的 checkpoint_txg。
+/// 树号从上一版的树 ID 水位起连发两个（码 14、码 15，水位 + 2，D8（核心索引结构） 已定项 8 ②），树表在续上的快照条目之后追加这一对，
+/// 活头 inode 树条目的 `previous_snapshot_txg` 换成 S.txg（字段取值见 [`SnapshotStepOfThePublish::CreateSnapshot`]）。deadlist 一条不动。
+/// 一个事务（不写用户数据，不切；事务号接着实例内计数，同建 inode）。
+///
+/// 被快照引用的块这一批不改释放：之后的覆盖写照旧经映射释放被换下的角色（释放分流、deadlist、映射条目留存与删快照归下一批）。
+///
+/// # Errors
+/// 上一版已有 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 个快照 ⇒ `SnapshotCreationRefused(MoreSnapshotsThanTheFirstVersionAllows)`；
+/// 水位加二越过 `u64::MAX` ⇒ `SnapshotCreationRefused(TreeIdentifierWatermarkLeavesNoRoomForTheSnapshotTrees)`；两样都在任何读写之前返回，
+/// 盘上逐字节不变。其余同 [`publish_version`]。
+pub fn publish_create_snapshot<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, PublishError> {
+    let snapshots_before_this_publish = previous
+        .tree_table_entries
+        .iter()
+        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
+        .count();
+    if let Some(being_destroyed) = previous.snapshots_being_destroyed.first().copied() {
+        return Err(PublishError::SnapshotCreationRefused(
+            SnapshotCreationRefusal::SnapshotBeingDestroyed { being_destroyed },
+        ));
+    }
+    if snapshots_before_this_publish >= SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
+        return Err(PublishError::SnapshotCreationRefused(
+            SnapshotCreationRefusal::MoreSnapshotsThanTheFirstVersionAllows {
+                snapshots: snapshots_before_this_publish,
+            },
+        ));
+    }
+    let watermark_before_this_publish = previous.root.tree_identifier_watermark;
+    let watermark_after_this_publish = watermark_before_this_publish.checked_add(2).ok_or(
+        PublishError::SnapshotCreationRefused(
+            SnapshotCreationRefusal::TreeIdentifierWatermarkLeavesNoRoomForTheSnapshotTrees {
+                tree_identifier_watermark: watermark_before_this_publish,
+            },
+        ),
+    )?;
+    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
+    let transaction_of_the_snapshot = previous.highest_transaction_number_in_this_instance + 1;
+    let snapshot = SnapshotStepOfThePublish::CreateSnapshot {
+        snapshot_inode_root_tree: TreeIdentifier(watermark_before_this_publish),
+        snapshot_extent_root_tree: TreeIdentifier(watermark_before_this_publish + 1),
+    };
+    publish_version_of_trees(
+        pool,
+        allocator,
+        PublishPlan {
+            txg,
+            counter: previous.record.counter + 1,
+            transaction: transaction_of_the_snapshot,
+            highest_transaction_number_before_this_publish: previous
+                .highest_transaction_number_in_this_instance,
+            instance,
+            back_chain: back_chain_of(&previous.record_bytes),
+            // 文件内容不动：数据单元、extent 树与 inode 树照抄上一版。
+            file: None,
+            new_inode_records: &[],
+            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
+            tree_birth_txg: previous.tree_birth_txg(),
+            tree_identifier_watermark: watermark_after_this_publish,
+            rollback_floor: previous.root.rollback_floor,
+        },
+        Some(previous),
+        previous.tree_identifiers,
+        UnmountMarker::NotWrittenByTheUnmountSequence,
+        &DeletionAndLineageOfThePublish {
+            step: DeletionStepOfThePublish::NotADeletion,
+            container_tree_capacities: pool.container_tree_capacities(),
+            lineage: LineageStepOfThePublish::NoLineageRecord,
+            snapshot,
+            logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
+        },
+    )
+}
+
+/// 删快照 `snapshot` 的意图那次发布（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4 与第七节岔路 2、4）：
+/// 它那条码 14 条目的 flags 位 0 写成「销毁中」、预留字节不写；它的前驱 P 传给那一刻的下一侧（[`SnapshotStepOfThePublish::DestructionIntent`]）；
+/// 一条 deadlist 条目都不动、一个单元都不释放。照 unlink 的意图那次同一张计划（不写文件内容、不建 inode、实例表照抄、不发树号、一个事务），
+/// 同等准入（式子那一判不判、落点那一判照判）。从这一刻起被删的快照不再钉住块（D5（快照 / 空间记账机制） 已定项 20 的例外）；
+/// 之后的批由 [`publish_snapshot_destruction_step`] 走，会话与挂载续做的走空（[`walk_the_deletion_queue_until_it_is_empty`]）接着走它们。
+///
+/// # Errors
+/// 已经有快照在销毁中（删快照串行，判决第七节岔路 2）⇒ `SnapshotDestructionRefused(SnapshotAlreadyBeingDestroyed / AnotherSnapshotIsBeingDestroyed)`；
+/// 树表里没有这个快照 ⇒ `SnapshotDestructionRefused(NoSnapshotWithThisTree)`；都在任何读写之前。其余同 `publish_version`。
+pub fn publish_snapshot_destruction_intent<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    snapshot: TreeIdentifier,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, PublishError> {
+    if let Some(being_destroyed) = previous.snapshots_being_destroyed.first().copied() {
+        return Err(PublishError::SnapshotDestructionRefused(
+            if being_destroyed == snapshot {
+                SnapshotDestructionRefusal::SnapshotAlreadyBeingDestroyed { snapshot }
+            } else {
+                SnapshotDestructionRefusal::AnotherSnapshotIsBeingDestroyed { being_destroyed }
+            },
+        ));
+    }
+    if snapshot_inode_root_entry(&previous.tree_table_entries, snapshot).is_none() {
+        return Err(PublishError::SnapshotDestructionRefused(
+            SnapshotDestructionRefusal::NoSnapshotWithThisTree { snapshot },
+        ));
+    }
+    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
+    let deletion_and_lineage = DeletionAndLineageOfThePublish {
+        step: DeletionStepOfThePublish::NotADeletion,
+        container_tree_capacities: pool.container_tree_capacities(),
+        lineage: LineageStepOfThePublish::NoLineageRecord,
+        snapshot: SnapshotStepOfThePublish::DestructionIntent { snapshot },
+        logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
+    };
+    publish_deletion_step(
+        pool,
+        allocator,
+        previous,
+        txg,
+        &deletion_and_lineage,
+        instance,
+    )
+}
+
+/// 删快照意图之后的下一批（判决 S4）：这一版 `snapshot` 那一段里能放的条目（[`deadlist_entries_the_destruction_of_a_snapshot_releases`]，
+/// 下一侧是活头时 hi 取这一批自己的 txg）多于一批就放按 key 升序的前一批（[`crate::unlink::deadlist_entries_per_snapshot_destruction_batch`]），
+/// 不多于一批就是最后一批：放掉剩下的全部、同一次发布删掉那一对码 14、15 条目。每条放之前按它留存的映射条目读盘核、逐盘核落点，
+/// 删 deadlist 条目、删留存的映射条目、落点进 defer 队列。下一批只由现行那一版定，续做与一路做下来同一个计划；续做点不存（条目被放掉就是进度）。
+///
+/// # Errors
+/// `snapshot` 不带「销毁中」⇒ `SnapshotDestructionRefused(SnapshotNotBeingDestroyed)`，在任何读写之前；条目放不了 ⇒ `DeadlistEntryNotReleasable`，
+/// 在任何落盘动作之前；其余同 `publish_version`。
+pub fn publish_snapshot_destruction_step<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    snapshot: TreeIdentifier,
+    batch_size: UnlinkBatchSize,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, PublishError> {
+    if !previous.snapshots_being_destroyed.contains(&snapshot) {
+        return Err(PublishError::SnapshotDestructionRefused(
+            SnapshotDestructionRefusal::SnapshotNotBeingDestroyed { snapshot },
+        ));
+    }
+    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
+    let releasable =
+        deadlist_entries_the_destruction_of_a_snapshot_releases(previous, snapshot, txg).len();
+    let entries_per_batch =
+        crate::unlink::deadlist_entries_per_snapshot_destruction_batch(batch_size);
+    let step = if releasable <= entries_per_batch {
+        SnapshotStepOfThePublish::DestructionFinalBatch { snapshot }
+    } else {
+        SnapshotStepOfThePublish::DestructionBatch {
+            snapshot,
+            entries_released: entries_per_batch,
+        }
+    };
+    let deletion_and_lineage = DeletionAndLineageOfThePublish {
+        step: DeletionStepOfThePublish::NotADeletion,
+        container_tree_capacities: pool.container_tree_capacities(),
+        lineage: LineageStepOfThePublish::NoLineageRecord,
+        snapshot: step,
+        logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
+    };
+    publish_deletion_step(
+        pool,
+        allocator,
+        previous,
+        txg,
+        &deletion_and_lineage,
+        instance,
+    )
+}
+
+/// 整理的一批为什么没发：计划与上一版对不上（调用方按现行那一版重扫再计划，丁 1 交判第 3 点「每批之前对 R 重扫」），或发布路径的错。
+/// 计划那几种都在任何读写之前交回，盘上逐字节不变。
+#[derive(Debug)]
+pub enum RelocationBatchRefused {
+    /// 计划里这把映射 key 不是上一版里进映射的单元，也不是留存的映射条目。
+    PlannedUnitNotInTheVersion {
+        mapping_key: Vec<u8>,
+    },
+    /// 映射树自己的节点、树表、实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8），整理不搬它们（区域扫描判钉住）。
+    PinnedUnitInThePlan {
+        mapping_key: Vec<u8>,
+        unit: TransactionUnit,
+    },
+    /// 计划的源落点不是这个单元在上一版映射里的位置，或跨度与它的类不符。
+    PlannedSourceIsNotTheMappingPosition {
+        mapping_key: Vec<u8>,
+        planned_source: Placement,
+        mapping_locations: Option<[LocationEntry; 2]>,
+    },
+    /// 这个单元的字节不在手里：从盘上重建时提示与映射都读不出的数据单元（D19 已定项 5：照抄位置项、不读内容），
+    /// 或留存条目指的单元两份都读不出、核不上位置项里的校验和。原样字节写目的地（判决 G3）做不到，不搬。
+    UnitBytesNotAvailable {
+        mapping_key: Vec<u8>,
+    },
+    /// 签发时上一版已有一条活整理意图（全池至多一条，判决 G1 e1）：调用方先推完或丢掉那一条。
+    CompactionIntentAlreadyLive {
+        issued_txg: CheckpointTxg,
+    },
+    /// 删意图时上一版没有活整理意图。
+    NoLiveCompactionIntent,
+    /// 意图达成那一支（[`CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords`]）在这次发布算好的账上 R 里还有未释放的记录：
+    /// 停机谓词「删意图那一刻 R 按记录全空」（判决 G4）不成立，不删。只在 R 不排他（只供测试的开关关掉排他）、这次发布的新落点落进 R 时走得到；
+    /// 调用方照这一批不删意图再发一次（意图留着，下一批接着搬）。
+    RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
+        region: CompactionRegion,
+    },
+    Publish(PublishError),
+}
+
+impl From<PublishError> for RelocationBatchRefused {
+    fn from(cause: PublishError) -> Self {
+        RelocationBatchRefused::Publish(cause)
+    }
+}
+
+/// 整理的一批（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3，用户定「重构」取乙）：第三个发布入口，额外参数不并进
+/// [`PublishPlan`]（照 [`publish_unlink_step`]）。`planned` 里每一项（[`crate::compaction_plan::plan_batch`] 交出的映射 key 与源落点）
+/// 原样字节写到目的地（头不盖戳、birth 不变）、映射条目换到目的地、源落点写释放代进 defer、目的地记一条分配代 = 诞生代的已分配记录
+/// （D26（后台整理与放置回收） 已定项 5 第 2 条），记账、分配记录树、中央映射树与树表照常重写；journal 点名项 = 搬迁副本 + 新写的节点，
+/// 一个事务号。目的落点由这次发布时的分配器现取（判决 G2，不缓存），活 R 由分配器排除。被快照引用的块照样搬（留存的映射条目跟着换位置）。
+/// 这次重写本来就换下的角色（记账树、分配记录树里内容变了的节点）不另搬：它们的新落点本来就不在活 R 里。
+/// logged_ops 照抄上一版（意图的签发与删在 [`publish_compaction_step`]，这一个入口就是它的「只搬、意图照抄」那一支）。
+/// 照删除的三种发布同一张计划：不写文件内容、不建 inode、实例表照抄、不发树号；准入同删除类（式子那一判照判，合取 7 免）。
+///
+/// # Errors
+/// 计划对不上上一版 ⇒ [`RelocationBatchRefused`] 那四种，在任何读写之前；其余同 `publish_version`（`Publish`）。
+///
+/// # Panics
+/// `planned` 是空的：调用方按 `PlannedBatch::Relocate` 才调（至少一项）。
+pub fn publish_relocation_batch<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    planned: &[PlannedRelocation],
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, RelocationBatchRefused> {
+    assert!(
+        !planned.is_empty(),
+        "一批至少搬一个单元：调用方按 PlannedBatch::Relocate 才调"
+    );
+    publish_compaction_step(
+        pool,
+        allocator,
+        previous,
+        planned,
+        CompactionIntentStep::CarryTheIntent,
+        instance,
+    )
+}
+
+/// 整理的一次发布对意图做什么（[`publish_compaction_step`]；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3、G4；
+/// D26（后台整理与放置回收） 已定项 2；D8（核心索引结构） 已定项 5「删除意图必须与最后一批工作在同一个事务里提交」）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionIntentStep {
+    /// 只搬一批，logged_ops 照抄。
+    CarryTheIntent,
+    /// 签发：logged_ops 追加这一条意图（签发 txg = 这次发布的 txg）。上一版不许已有活整理意图（全池至多一条，判决 G1 e1）。
+    Issue(CompactionIntent),
+    /// 意图达成：删掉上一版那一条活意图。删之前在这次发布算好的账上核 R 按记录全空（停机谓词「删意图那一刻 R 按记录全空」，判决 G4）——
+    /// 带着一批时就是最后一批同发布删。
+    DeleteOnceTheRegionIsEmptyByTheRecords,
+    /// 丢意图（攻方 K-c：R 里有隔离槽；区域扫描拒了 R；R 里一个能搬的都没有）：删掉上一版那一条活意图，不核 R。
+    Drop,
+}
+
+/// 整理的一次发布（[`publish_relocation_batch`] 的一般形）：搬 `planned` 里那几个单元（可以一个都没有），同一次发布对意图做 `intent_step`
+/// （判决 G1：意图条目写进 logged_ops 树，签发那次发布写、最后一批同发布删；D8（核心索引结构） 已定项 5 三态表：签发之前崩 = 没有意图，
+/// 签发之后、最后一批之前崩 = 意图还在、挂载续做，最后一批之后 = 意图没了且目标态达成）。别的同 [`publish_relocation_batch`]。
+///
+/// # Errors
+/// 计划对不上上一版 ⇒ [`RelocationBatchRefused`] 那四种计划错；签发时上一版已有活意图 ⇒ `CompactionIntentAlreadyLive`；删时上一版没有活意图 ⇒
+/// `NoLiveCompactionIntent`；意图达成那一支在这次发布算好的账上 R 还有未释放的记录 ⇒ `RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted`。
+/// 这几种都在任何写之前交回（最后一种在算账之后、落盘之前：分配器换回进来时那一份），盘上逐字节不变；其余同 `publish_version`（`Publish`）。
+///
+/// # Panics
+/// `planned` 是空的而 `intent_step` 是 `CarryTheIntent`：这样的发布什么都不做，调用方不许走到这里。
+pub fn publish_compaction_step<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    planned: &[PlannedRelocation],
+    intent_step: CompactionIntentStep,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, RelocationBatchRefused> {
+    assert!(
+        !(planned.is_empty() && intent_step == CompactionIntentStep::CarryTheIntent),
+        "整理的一次发布要么搬单元、要么签发或删意图：调用方不发什么都不做的发布"
+    );
+    let live_intent = live_compaction_intent_of_the_entries(&previous.logged_operations);
+    let (logged_operations, region_to_be_empty) = match (intent_step, live_intent) {
+        (CompactionIntentStep::CarryTheIntent, _) => (
+            LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
+            None,
+        ),
+        (CompactionIntentStep::Issue(intent), None) => (
+            LoggedOperationsStepOfThePublish::IssueTheCompactionIntent(intent),
+            None,
+        ),
+        (CompactionIntentStep::Issue(_), Some((issued_txg, _))) => {
+            return Err(RelocationBatchRefused::CompactionIntentAlreadyLive { issued_txg })
+        }
+        (
+            CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords
+            | CompactionIntentStep::Drop,
+            None,
+        ) => return Err(RelocationBatchRefused::NoLiveCompactionIntent),
+        (
+            CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
+            Some((issued_txg, intent)),
+        ) => (
+            LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent { issued_txg },
+            Some(intent.region),
+        ),
+        (CompactionIntentStep::Drop, Some((issued_txg, _))) => (
+            LoggedOperationsStepOfThePublish::DeleteTheCompactionIntent { issued_txg },
+            None,
+        ),
+    };
+    let mut relocations = Vec::with_capacity(planned.len());
+    for planned_relocation in planned {
+        relocations.push(relocation_checked_against_the_version(
+            &*pool.devices,
+            previous,
+            planned_relocation,
+        )?);
+    }
+    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
+    // 整理的每一次发布（搬一批、签发、删意图）都是一个事务，事务号接着这个实例（同建快照、删除的意图那一次：不写数据单元也取下一个号）。
+    let transaction_of_this_publish = previous.highest_transaction_number_in_this_instance + 1;
+    let deletion_and_lineage = DeletionAndLineageOfThePublish {
+        step: DeletionStepOfThePublish::NotADeletion,
+        container_tree_capacities: pool.container_tree_capacities(),
+        lineage: LineageStepOfThePublish::NoLineageRecord,
+        snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
+        logged_operations,
+    };
+    publish_version_of_trees_relocating_units_after_a_check(
+        pool,
+        allocator,
+        PublishPlan {
+            txg,
+            counter: previous.record.counter + 1,
+            transaction: transaction_of_this_publish,
+            highest_transaction_number_before_this_publish: previous
+                .highest_transaction_number_in_this_instance,
+            instance,
+            back_chain: back_chain_of(&previous.record_bytes),
+            file: None,
+            new_inode_records: &[],
+            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
+            tree_birth_txg: previous.tree_birth_txg(),
+            tree_identifier_watermark: previous.root.tree_identifier_watermark,
+            rollback_floor: previous.root.rollback_floor,
+        },
+        Some(previous),
+        previous.tree_identifiers,
+        UnmountMarker::NotWrittenByTheUnmountSequence,
+        &deletion_and_lineage,
+        &relocations,
+        |version_after_this_publish| match region_to_be_empty {
+            Some(region)
+                if !region_is_empty_by_the_records(
+                    &version_after_this_publish.allocation_records,
+                    region,
+                ) =>
+            {
+                Err(
+                    RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
+                        region,
+                    },
+                )
+            }
+            Some(_) | None => Ok(()),
+        },
+    )
+}
+
+/// 计划里的一项对着上一版核过、带上搬它要的东西（[`publish_relocation_batch`]）：它是这一版进映射的单元（取它的角色与内存里的字节），
+/// 或是留存的映射条目（字节从盘上按映射位置读、核位置项里的校验和，两份取先核得上的那一份）；源落点要就是映射位置、跨度合它的类。
+///
+/// # Errors
+/// [`RelocationBatchRefused`] 的四种计划错。
+fn relocation_checked_against_the_version<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    previous: &TransactionOutput,
+    planned: &PlannedRelocation,
+) -> Result<RelocationOfThePublish, RelocationBatchRefused> {
+    let mapping_key = planned.mapping_key.clone();
+    let holder_and_locations = match previous
+        .mapped_units
+        .iter()
+        .find(|(_, key)| *key == mapping_key)
+    {
+        Some((role, _)) => {
+            let locations = match central_mapping_locations_in_the_version(previous, &mapping_key) {
+                MappingLookup::Found(locations) => Some(locations),
+                MappingLookup::NodeMalformedOrNoEntryWithThisKey
+                | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
+            };
+            Some((
+                RelocatedUnitHolder::RoleOfThisVersion(*role),
+                *role,
+                locations,
+            ))
+        }
+        None => previous
+            .mapping_entries_kept_for_the_snapshots
+            .iter()
+            .find(|(key, _)| *key == mapping_key)
+            .and_then(|(_, locations)| {
+                user_visible_role_standing_for_a_unit_class(
+                    mapping_key.first().copied().unwrap_or(0),
+                )
+                .map(|role| {
+                    (
+                        RelocatedUnitHolder::KeptForTheSnapshots,
+                        role,
+                        Some(*locations),
+                    )
+                })
+            }),
+    };
+    let Some((holder, role_for_the_checks, mapping_locations)) = holder_and_locations else {
+        return Err(RelocationBatchRefused::PlannedUnitNotInTheVersion { mapping_key });
+    };
+    match role_for_the_checks {
+        TransactionUnit::MappingTreeNodeBelowTheRoot(_)
+        | TransactionUnit::MappingTree
+        | TransactionUnit::TreeTable
+        | TransactionUnit::InstanceTable
+        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
+            return Err(RelocationBatchRefused::PinnedUnitInThePlan {
+                mapping_key,
+                unit: role_for_the_checks,
+            })
+        }
+        TransactionUnit::Data(_)
+        | TransactionUnit::ExtentLowerNode(_)
+        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+        | TransactionUnit::ExtentRoot
+        | TransactionUnit::InodeLeafContainer(_)
+        | TransactionUnit::InodeRoot
+        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
+        | TransactionUnit::AllocationTree
+        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+        | TransactionUnit::AccountingTree
+        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+        | TransactionUnit::DeadlistTree
+        | TransactionUnit::DeletedInodesRoot
+        | TransactionUnit::LoggedOperationsRoot
+        | TransactionUnit::TombstoneLeafContainer(_)
+        | TransactionUnit::TombstoneRoot
+        | TransactionUnit::TombstoneNodeBelowTheRoot(_)
+        | TransactionUnit::LineageLeafContainer(_)
+        | TransactionUnit::LineageRoot
+        | TransactionUnit::LineageNodeBelowTheRoot(_)
+        | TransactionUnit::RetirementLeafContainer(_)
+        | TransactionUnit::RetirementRoot
+        | TransactionUnit::RetirementNodeBelowTheRoot(_) => {}
+    }
+    let Some(source_locations) = mapping_locations.filter(|locations| {
+        locations
+            .iter()
+            .all(|location| location.slot == planned.source.slot)
+            && planned.source.span == role_for_the_checks.span_slots()
+    }) else {
+        return Err(
+            RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition {
+                mapping_key,
+                planned_source: planned.source,
+                mapping_locations,
+            },
+        );
+    };
+    let bytes = match holder {
+        RelocatedUnitHolder::RoleOfThisVersion(role) => previous.unit(role).bytes.clone(),
+        RelocatedUnitHolder::KeptForTheSnapshots => {
+            let unit_bytes = usize::try_from(planned.source.span * SLOT_BYTES)
+                .expect("一个单元两槽以内，字节数装得进 usize");
+            source_locations
+                .iter()
+                .find_map(|location| {
+                    reader
+                        .read(
+                            location.device,
+                            location.slot.to_device_offset(),
+                            unit_bytes,
+                        )
+                        .filter(|copy| crc32_castagnoli(copy) == location.unit_checksum)
+                })
+                .unwrap_or_default()
+        }
+    };
+    if bytes.is_empty() {
+        return Err(RelocationBatchRefused::UnitBytesNotAvailable { mapping_key });
+    }
+    let Some(birth_txg) = birth_txg_of_a_mapping_key(&mapping_key) else {
+        return Err(RelocationBatchRefused::PlannedUnitNotInTheVersion { mapping_key });
+    };
+    Ok(RelocationOfThePublish {
+        mapping_key,
+        holder,
+        role_for_the_checks,
+        source_locations,
+        source: planned.source,
+        bytes,
+        birth_txg,
+    })
+}
+
 /// unlink 的意图那次发布（D8（核心索引结构） 已定项 16 第 1 条）：`inode` 那条记录写成 nlink = 0、size 不变，`deleted_inodes` 加 key = `inode`，
 /// 墓碑树开放容器追加一条整对象墓碑（死亡 txg = 这次发布的 checkpoint_txg，D18（块里携带什么信息） 已定项 24 第 2 条）；不立 logged_ops。
 /// 数据单元与 extent 树照抄。一个事务（不写用户数据，不切）。改动计数设成这次发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88
@@ -6252,6 +7775,8 @@ pub fn publish_unlink_intent<Device: BlockDevice>(
         },
         container_tree_capacities: pool.container_tree_capacities(),
         lineage: LineageStepOfThePublish::NoLineageRecord,
+        snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
+        logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
     };
     publish_deletion_step(
         pool,
@@ -6319,6 +7844,8 @@ pub fn publish_unlink_step<Device: BlockDevice>(
         step,
         container_tree_capacities: pool.container_tree_capacities(),
         lineage: LineageStepOfThePublish::NoLineageRecord,
+        snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
+        logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
     };
     publish_deletion_step(
         pool,
@@ -6366,15 +7893,27 @@ pub fn walk_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
     instance: InstanceGeneration,
 ) -> DeletionQueueWalk {
     let mut steps_published = Vec::new();
-    // 迭代上界：每一步要么让队首那个文件少至少一个单元、要么删掉队首那个号（最后一批），队列与单元数都有限。
+    // 迭代上界：每一步要么让队首那个文件少至少一个单元、要么删掉队首那个号（最后一批），要么放掉销毁中那个快照至少一条 deadlist 条目、
+    // 要么删掉它那一对条目（最后一批）：队列、单元数与条目数都有限。`deleted_inodes` 先走空，再走销毁中的快照（树号升序）。
     loop {
-        let Some(inode) = current.deleted_inodes.first().copied() else {
-            return DeletionQueueWalk {
-                steps_published,
-                stopped_by: Ok(()),
-            };
+        let next_step = match (
+            current.deleted_inodes.first().copied(),
+            current.snapshots_being_destroyed.first().copied(),
+        ) {
+            (Some(inode), _) => {
+                publish_unlink_step(pool, allocator, current, inode, batch_size, instance)
+            }
+            (None, Some(snapshot)) => publish_snapshot_destruction_step(
+                pool, allocator, current, snapshot, batch_size, instance,
+            ),
+            (None, None) => {
+                return DeletionQueueWalk {
+                    steps_published,
+                    stopped_by: Ok(()),
+                }
+            }
         };
-        match publish_unlink_step(pool, allocator, current, inode, batch_size, instance) {
+        match next_step {
             Ok(next) => {
                 *current = next;
                 steps_published.push(current.clone());
@@ -6599,6 +8138,8 @@ pub fn publish_version_appending_a_lineage_record<Device: BlockDevice>(
                 record: lineage_record,
                 inode_number_watermark_after_the_rollback,
             },
+            snapshot: SnapshotStepOfThePublish::NoNewSnapshot,
+            logged_operations: LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
         },
     )
 }
@@ -6614,11 +8155,47 @@ fn publish_version_of_trees<Device: BlockDevice>(
     unmount_marker: UnmountMarker,
     deletion_and_lineage: &DeletionAndLineageOfThePublish,
 ) -> Result<TransactionOutput, PublishError> {
+    publish_version_of_trees_relocating_units(
+        pool,
+        allocator,
+        plan,
+        previous,
+        trees,
+        unmount_marker,
+        deletion_and_lineage,
+        &[],
+    )
+}
+
+/// 同 [`publish_version_of_trees_relocating_units`]，算完账、落盘之前先交 `check_before_persisting` 核这一版（整理删意图那一支核 R 按记录全空，
+/// [`publish_compaction_step`]）：核不过就把分配器换回进来时那一份、一个写都不发，原样交回核出来的拒绝。
+///
+/// # Errors
+/// `check_before_persisting` 的拒绝；其余同 [`publish_version_of_trees_relocating_units`]（经 `From<PublishError>` 换成 `Refusal`）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、卸载记号、删除这一步、这次要搬的单元、落盘之前的核"
+)]
+fn publish_version_of_trees_relocating_units_after_a_check<Device: BlockDevice, Refusal, Check>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    plan: PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
+    trees: FileVersionTreeIdentifiers,
+    unmount_marker: UnmountMarker,
+    deletion_and_lineage: &DeletionAndLineageOfThePublish,
+    relocations: &[RelocationOfThePublish],
+    check_before_persisting: Check,
+) -> Result<TransactionOutput, Refusal>
+where
+    Refusal: From<PublishError>,
+    Check: FnOnce(&TransactionOutput) -> Result<(), Refusal>,
+{
     // 第一道：冻结着一次没重发的发布就在任何读写之前拒（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）。
     refuse_while_a_publish_is_frozen(allocator)?;
     // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
     let allocator_before_this_publish = allocator.clone();
-    let built = prepare_the_version_publish(
+    let built = prepare_the_version_publish_relocating_units(
         pool,
         allocator,
         &plan,
@@ -6627,14 +8204,19 @@ fn publish_version_of_trees<Device: BlockDevice>(
         ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt,
         unmount_marker,
         deletion_and_lineage,
+        relocations,
     );
     let (writes, output) = match built {
         Ok(built) => built,
         Err(refusal) => {
             *allocator = allocator_before_this_publish;
-            return Err(refusal);
+            return Err(Refusal::from(refusal));
         }
     };
+    if let Err(refused) = check_before_persisting(&output) {
+        *allocator = allocator_before_this_publish;
+        return Err(refused);
+    }
     // 落盘那几步里失败：分配器换回去、这次发布冻结在它上面等原样重发（`FrozenPublish`）。
     persist_the_publish_or_freeze_it(
         pool,
@@ -6648,6 +8230,35 @@ fn publish_version_of_trees<Device: BlockDevice>(
             .into_version_of_trees()
             .expect("交给落盘的是有树的一版，交回的就是它（`PoolVersion::with_writes` 不换成员）")
     })
+    .map_err(Refusal::from)
+}
+
+/// 同 [`publish_version_of_trees`]，另带这次要搬的单元（[`prepare_the_version_publish_relocating_units`]）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、卸载记号、删除这一步、这次要搬的单元"
+)]
+fn publish_version_of_trees_relocating_units<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    plan: PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
+    trees: FileVersionTreeIdentifiers,
+    unmount_marker: UnmountMarker,
+    deletion_and_lineage: &DeletionAndLineageOfThePublish,
+    relocations: &[RelocationOfThePublish],
+) -> Result<TransactionOutput, PublishError> {
+    publish_version_of_trees_relocating_units_after_a_check(
+        pool,
+        allocator,
+        plan,
+        previous,
+        trees,
+        unmount_marker,
+        deletion_and_lineage,
+        relocations,
+        |_version_after_this_publish| Ok(()),
+    )
 }
 
 /// 释放之前读不读盘核被换下的每一份的校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）。
@@ -6683,6 +8294,65 @@ pub fn prepare_the_version_publish<Device: BlockDevice>(
     release_checksum_check: ReleaseChecksumCheck,
     unmount_marker: UnmountMarker,
     deletion_and_lineage: &DeletionAndLineageOfThePublish,
+) -> Result<(PublishWrites, TransactionOutput), PublishError> {
+    prepare_the_version_publish_relocating_units(
+        pool,
+        allocator,
+        plan,
+        previous,
+        trees,
+        release_checksum_check,
+        unmount_marker,
+        deletion_and_lineage,
+        &[],
+    )
+}
+
+/// 合取 7（`全空聚簇段数 ≥ N₀`）管不管这一次发布（[`EmptyClusterSegmentsConjunctReach`]，D3（空间分配） 已定项 7 登记表第 7 行）：
+/// 写文件内容、建 inode、建快照的发布是前台写入类；管理员回退、删除类（unlink 与删快照）、整理批（`relocations` 非空）与
+/// 别的不写文件内容也不建 inode 的发布（写行、暖机、抬 F 的空发布）免。
+fn reach_of_the_empty_cluster_segments_conjunct(
+    plan: &PublishPlan<'_>,
+    deletion_and_lineage: &DeletionAndLineageOfThePublish,
+    relocations: &[RelocationOfThePublish],
+) -> EmptyClusterSegmentsConjunctReach {
+    let is_a_rollback = match deletion_and_lineage.lineage {
+        LineageStepOfThePublish::NoLineageRecord => false,
+        LineageStepOfThePublish::AdministratorRollback { .. } => true,
+    };
+    let creates_a_snapshot = match deletion_and_lineage.snapshot {
+        SnapshotStepOfThePublish::CreateSnapshot { .. } => true,
+        SnapshotStepOfThePublish::NoNewSnapshot
+        | SnapshotStepOfThePublish::DestructionIntent { .. }
+        | SnapshotStepOfThePublish::DestructionBatch { .. }
+        | SnapshotStepOfThePublish::DestructionFinalBatch { .. } => false,
+    };
+    let writes_in_the_foreground =
+        plan.file.is_some() || !plan.new_inode_records.is_empty() || creates_a_snapshot;
+    if writes_in_the_foreground && !is_a_rollback && relocations.is_empty() {
+        EmptyClusterSegmentsConjunctReach::ForegroundWrite
+    } else {
+        EmptyClusterSegmentsConjunctReach::Exempt
+    }
+}
+
+/// 同 [`prepare_the_version_publish`]，另带这次要搬的单元（整理的一批，[`publish_relocation_batch`]；别的发布给空的）：
+/// 固定点与真发都按「释放（换下的与搬走的源落点）→ 取目的地 → 取这次重写的角色的落点」走同一串，源落点照释放判定路径逐盘核、
+/// 读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）；这次重写要换下的角色不另搬（它的新落点本来就不在活 R 里）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：写入口、分配器、计划、上一版、树号、读不读盘核、卸载记号、删除这一步、这次要搬的单元"
+)]
+fn prepare_the_version_publish_relocating_units<Device: BlockDevice>(
+    pool: &PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    plan: &PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
+    trees: FileVersionTreeIdentifiers,
+    release_checksum_check: ReleaseChecksumCheck,
+    unmount_marker: UnmountMarker,
+    deletion_and_lineage: &DeletionAndLineageOfThePublish,
+    relocations: &[RelocationOfThePublish],
 ) -> Result<(PublishWrites, TransactionOutput), PublishError> {
     // 先算这次之后每棵树是什么样、这次重写哪些角色、换下哪些：只读（固定点在分配器的拷贝上走），条款没写的那几格与多层码 2 树的拒绝、
     // 释放判定的错、取不到落点都在这里交回，盘上逐字节不变。记账行按分配器里的那几块盘展开（装记账行时读的是同一批）。
@@ -6729,16 +8399,46 @@ pub fn prepare_the_version_publish<Device: BlockDevice>(
         pool.code_two_tree_node_capacities(),
         allocator,
         deletion_and_lineage,
+        relocations,
     )?;
     // 这次发布切出来的记录条数（代码审阅第 26 条）：角色定下来就数得出，在准入、读盘核与动分配器之前判。
     refuse_a_publish_of_more_journal_records_than_the_limit(
-        roles_named_by_each_record_of_the_publish(
+        records_of_a_publish_naming_the_relocated_copies(
             &settled.resolved.rewritten_roles,
+            settled.relocations_in_effect.len(),
             pool.journal_record_named_entry_capacity(),
-        )
-        .len(),
+        ),
         pool.parameters.geometry.journal_ring_bytes,
     )?;
+    // 合取 7（D3（空间分配） 已定项 7 登记表第 7 行：`全空聚簇段数 ≥ N₀`，只拦前台写入类发布）：判不判与式子那一判同一个开关、
+    // 同样只判带普通分配的发布；在式子那一判、读盘核与动分配器之前拒，盘上逐字节不变。`N₀` 取分配器上整理开关里的那一档（产品路径是第一版的 1）。
+    let empty_cluster_segments_conjunct_is_judged = match (
+        allocator.space_admission(),
+        reach_of_the_empty_cluster_segments_conjunct(plan, deletion_and_lineage, relocations),
+    ) {
+        (
+            SpaceAdmission::JudgedByTheFormula,
+            EmptyClusterSegmentsConjunctReach::ForegroundWrite,
+        ) => publish_has_an_ordinary_allocation(&settled.resolved.rewritten_roles),
+        (SpaceAdmission::JudgedByTheFormula, EmptyClusterSegmentsConjunctReach::Exempt)
+        | (SpaceAdmission::SkippedByTheTestOnlySwitch, _) => false,
+    };
+    if empty_cluster_segments_conjunct_is_judged {
+        let empty_cluster_segments_on_each_device: Vec<(DeviceIdentity, u64)> = allocator
+            .devices
+            .iter()
+            .map(|device_map| (device_map.device, device_map.empty_segments()))
+            .collect();
+        admit_the_empty_cluster_segments_floor_on_every_device(
+            &empty_cluster_segments_on_each_device,
+            allocator
+                .compaction()
+                .switches
+                .empty_cluster_segments_floor
+                .segments(),
+        )
+        .map_err(PublishError::EmptyClusterSegmentsBelowTheFloor)?;
+    }
     // 空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取）：读数取分配器此刻的计数——这次的释放与取落点都还没做——，
     // 实例切换的预留按下一次可写挂载的 rows0（已定项 3「发布路径用 rows0 + 1」）；需求 = 这次新写的全部角色（普通分配加固定点，
     // 不加换下的槽）；一个普通分配都没有的发布（空发布、写行、暖机、抬 F）不判，它们的空间在保留池与切换预留里
@@ -6749,9 +8449,11 @@ pub fn prepare_the_version_publish<Device: BlockDevice>(
         &device_identities_of_the_accounting_rows,
     );
     // 删除类发布（意图、各批、最后一批）不判式子那一判、落点那一判照判（D28（挂载期承诺量） 已定项 1：它们的空间罩在环窗口里）。
+    // 删快照的三种发布与 unlink 同准入（判决第七节岔路 4「删快照照 unlink 的过程做、同等准入」）：式子那一判同样不判。
     let demand_is_judged = match (allocator.space_admission(), deletion_and_lineage.step) {
         (SpaceAdmission::JudgedByTheFormula, DeletionStepOfThePublish::NotADeletion) => {
-            publish_has_an_ordinary_allocation(&settled.resolved.rewritten_roles)
+            !deletion_and_lineage.snapshot.is_a_snapshot_destruction()
+                && publish_has_an_ordinary_allocation(&settled.resolved.rewritten_roles)
         }
         (
             SpaceAdmission::JudgedByTheFormula,
@@ -6779,11 +8481,16 @@ pub fn prepare_the_version_publish<Device: BlockDevice>(
     }
     // 经映射核到的那几个单元，释放之前再按位置项读盘核一次校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）：只读，
     // 读不到就在动分配器、发任何一个写之前返回。第一个文件版本换下的 mkfs 那片树表与树表 0 条那一版的分配记录树节点不经映射，不核。
-    let quarantine = match (previous, release_checksum_check) {
+    let kept_for_the_snapshots = &settled
+        .resolved
+        .deadlist
+        .user_visible_roles_kept_for_the_snapshots;
+    let mut quarantine = match (previous, release_checksum_check) {
         (Some(previous_version), ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt) => {
             copies_failing_the_release_checksum_check(
                 previous_version,
                 &settled.previous_roles_replaced,
+                kept_for_the_snapshots,
                 &*pool.devices,
             )?
         }
@@ -6792,12 +8499,83 @@ pub fn prepare_the_version_publish<Device: BlockDevice>(
             refuse_mapping_entries_that_do_not_name_two_pool_devices(
                 previous_version,
                 &settled.previous_roles_replaced,
+                kept_for_the_snapshots,
                 &device_identities_of_the_accounting_rows,
             )?;
             Vec::new()
         }
         (None, _) => Vec::new(),
     };
+    // 删快照这一批放的条目同样先按留存的映射条目读盘核（判决 S4「每条先核还在、读盘核校验和」；条目还在由 `resolve` 从上一版的 deadlist 取保证）。
+    for (entry, locations) in &settled
+        .resolved
+        .deadlist
+        .entries_released_by_the_destruction
+    {
+        let role = role_standing_for_the_unit_of_a_deadlist_entry(entry)?;
+        let not_releasable = |cause: PublishError| PublishError::DeadlistEntryNotReleasable {
+            mapping_key: entry.mapping_key.clone(),
+            refusal: DeadlistEntryReleaseRefusal::PlacementNotReleasable(Box::new(cause)),
+        };
+        refuse_locations_that_do_not_name_two_pool_devices(
+            role,
+            locations,
+            &device_identities_of_the_accounting_rows,
+        )
+        .map_err(not_releasable)?;
+        match release_checksum_check {
+            ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt => {
+                quarantine.extend(
+                    copies_of_a_released_unit_failing_the_release_checksum_check(
+                        role,
+                        locations,
+                        &*pool.devices,
+                    ),
+                );
+            }
+            ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition => {}
+        }
+    }
+    // 搬走的源落点释放之前同样按映射位置项读盘核（判决 G3「源落点写释放代进 defer」走的是同一条释放判定路径）。签发之后强制一次
+    // 「释放前核出对不上」那个只供测试的开关（判决 G6）落在这一批第一个源落点上：两份都当核出对不上，隔离（D19 已定项 5）。
+    for (position, relocation) in settled.relocations_in_effect.iter().enumerate() {
+        refuse_locations_that_do_not_name_two_pool_devices(
+            relocation.role_for_the_checks,
+            &relocation.source_locations,
+            &device_identities_of_the_accounting_rows,
+        )?;
+        match release_checksum_check {
+            ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt => {
+                let is_forced_to_mismatch = position == 0
+                    && allocator
+                        .compaction_mut()
+                        .take_the_forced_release_checksum_mismatch();
+                if is_forced_to_mismatch {
+                    quarantine.extend(relocation.source_locations.iter().enumerate().map(
+                        |(copy_position, location)| CopyQuarantinedAfterReleaseChecksumMismatch {
+                            unit: relocation.role_for_the_checks,
+                            device: location.device,
+                            placement: relocation.source,
+                            reading: if copy_position == 0 {
+                                QuarantinedCopyReading::ChecksumMismatch
+                            } else {
+                                QuarantinedCopyReading::IntactButAnotherCopyFailed
+                            },
+                        },
+                    ));
+                } else {
+                    quarantine.extend(
+                        copies_of_a_released_unit_failing_the_release_checksum_check(
+                            relocation.role_for_the_checks,
+                            &relocation.source_locations,
+                            &*pool.devices,
+                        ),
+                    );
+                }
+            }
+            ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition => {}
+        }
+    }
     publish_admitted(
         pool,
         allocator,
@@ -6809,14 +8587,82 @@ pub fn prepare_the_version_publish<Device: BlockDevice>(
         trees,
         unmount_marker,
         deletion_and_lineage.lineage,
+        deletion_and_lineage.snapshot,
+        &settled.relocations_in_effect,
     )
 }
 
+/// 这次发布切出来的记录条数：没有搬迁副本时同 [`roles_named_by_each_record_of_the_publish`] 的项数；整理的一批（不重写数据单元，
+/// 一个事务）点名项 = 搬迁副本 + 这次重写的角色，按一条记录装几个点名项切（判决 G3「journal 点名项 = 搬迁副本 + 新写的节点，一个事务号」）。
+fn records_of_a_publish_naming_the_relocated_copies(
+    rewritten: &[TransactionUnit],
+    relocated_copies: usize,
+    named_entry_capacity: JournalRecordNamedEntryCapacity,
+) -> usize {
+    if relocated_copies == 0 {
+        return roles_named_by_each_record_of_the_publish(rewritten, named_entry_capacity).len();
+    }
+    (relocated_copies + rewritten.len()).div_ceil(named_entry_capacity.named_entries_per_record())
+}
+
+/// 整理的一批要搬的一个单元，交给发布路径之前验过（[`publish_relocation_batch`]）：映射 key、它归谁、释放判定与写账用哪个角色
+/// （留存条目没有角色，按类标签取 [`user_visible_role_standing_for_a_unit_class`]）、源落点（映射位置）、原样的字节、诞生代号。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct RelocationOfThePublish {
+    mapping_key: Vec<u8>,
+    holder: RelocatedUnitHolder,
+    role_for_the_checks: TransactionUnit,
+    source_locations: [LocationEntry; 2],
+    source: Placement,
+    bytes: Vec<u8>,
+    birth_txg: CheckpointTxg,
+}
+
+/// 一个搬迁单元的目的落点（判决 G2：由这一批发布时的分配器现取，不缓存）：照开关里的目的地政策取，分配代写单元的诞生代
+/// （D26（后台整理与放置回收） 已定项 5 第 2 条、D3（空间分配） 已定项 7「分配代」口径），活 R 由分配器排除。
+///
+/// # Errors
+/// 取不到 ⇒ `PlacementRefused`，带着释放判定用的那个角色。
+fn allocate_relocation_destination(
+    allocator: &mut PoolAllocator,
+    relocation: &RelocationOfThePublish,
+) -> Result<Placement, PublishError> {
+    let rule = relocation.role_for_the_checks.placement();
+    let placement = match (allocator.compaction().switches.destination_policy, rule) {
+        (
+            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
+            PlacementRule::UserData,
+        ) => allocator.try_allocate_user_data(relocation.birth_txg),
+        (
+            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
+            PlacementRule::CommitGenerated(footprint),
+        ) => allocator.try_allocate_commit_generated(footprint, relocation.birth_txg),
+        (
+            RelocationDestinationPolicy::LowestUnblockedSlotForEveryUnitForTests,
+            PlacementRule::UserData,
+        ) => allocator.try_allocate_at_the_lowest_unblocked_slot_for_tests(
+            UnitFootprint::TwoSlotsAligned,
+            relocation.birth_txg,
+        ),
+        (
+            RelocationDestinationPolicy::LowestUnblockedSlotForEveryUnitForTests,
+            PlacementRule::CommitGenerated(footprint),
+        ) => allocator
+            .try_allocate_at_the_lowest_unblocked_slot_for_tests(footprint, relocation.birth_txg),
+    };
+    placement.map_err(|refusal| PublishError::PlacementRefused {
+        unit: relocation.role_for_the_checks,
+        refusal,
+    })
+}
+
 /// 这次发布算定了的样子：每棵树这次之后的形状与重写的角色、这次换下的上一版角色、要释放的落点（按 bump 次序）。
 struct SettledPublish {
     resolved: ResolvedPublish,
     previous_roles_replaced: Vec<TransactionUnit>,
     release: Vec<Placement>,
+    /// 这次真搬的那几个（要搬的里面去掉这次重写本来就换下的角色），按计划的次序。
+    relocations_in_effect: Vec<RelocationOfThePublish>,
 }
 
 /// 这次发布换下的上一版角色与要释放的落点（释放判定路径，只查不改，D19（块指针的结构与宽度预算） 已定项 5 第 1 条）：
@@ -6837,12 +8683,36 @@ fn placements_released_by_this_publish(
         }
         None => Vec::new(),
     };
+    // 释放判定分流（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：切口在这一张清单出来之后、`publish_admitted`
+    // 落释放之前。`resolve` 按换下的用户可见单元定了哪几个进 deadlist；这里核它定的那一组就是按全部换下角色走出来的用户可见那一组
+    // （两边换下口径不同步就会漏判、多判），再把进 deadlist 的那几个从释放里拿掉，删快照这一批放的条目接在最后。
+    let kept_for_the_snapshots = &resolved.deadlist.user_visible_roles_kept_for_the_snapshots;
+    if let Some(previous_version) = previous {
+        let user_visible_roles_released_or_kept: BTreeSet<TransactionUnit> =
+            mapped_roles_released_by_this_publish(previous_version, &previous_roles_replaced, &[])
+                .into_iter()
+                .filter(|identity| asks_the_snapshot_lifetime_judgement(*identity))
+                .collect();
+        assert_eq!(
+            user_visible_roles_released_or_kept,
+            resolved
+                .deadlist
+                .user_visible_roles_replaced
+                .iter()
+                .copied()
+                .collect::<BTreeSet<TransactionUnit>>(),
+            "resolve 判生死的那一组用户可见单元，就是释放判定路径按这次换下的全部角色走出来的用户可见那一组"
+        );
+    }
     let release = match previous {
-        Some(previous_version) => placements_to_release_via_mapping(
-            previous_version,
-            allocator,
-            &previous_roles_replaced,
-        )?,
+        Some(previous_version) => {
+            placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(
+                previous_version,
+                allocator,
+                &previous_roles_replaced,
+                kept_for_the_snapshots,
+            )?
+        }
         // 第一个文件版本没有上一版的内存态：它重写树表时换下的是 mkfs 那片第 0 版树表单元，照样进 defer 队列
         // （D3（空间分配） 已定项 7；不释放它，txg 0 的根离开候选集之后这一槽就永远占着，I-3.1（已分配统计对得上） 在抬 F 之后红）。
         None => format_time_tree_table_to_release(allocator, &resolved.rewritten_roles)?,
@@ -6865,9 +8735,51 @@ fn placements_released_by_this_publish(
         }
         InstanceTablePlan::Carry(_) => release,
     };
+    // 删快照这一批放的条目：落点取它留存的映射条目（释放一律经映射，D19（块指针的结构与宽度预算） 已定项 5 第 1 条），照映射那一路逐盘核。
+    let mut release = release;
+    for (entry, locations) in &resolved.deadlist.entries_released_by_the_destruction {
+        release.push(placement_of_a_deadlist_entry_to_release(
+            entry, locations, allocator,
+        )?);
+    }
     Ok((previous_roles_replaced, release))
 }
 
+/// 删快照的一批放的一条 deadlist 条目的落点：按它映射 key 的类标签取跨度（[`user_visible_role_standing_for_a_unit_class`]），
+/// 照映射那一路逐盘核（[`placement_to_release_after_checking_every_device`]）。
+///
+/// # Errors
+/// 类标签不认识、落点核不过 ⇒ `DeadlistEntryNotReleasable`。
+fn placement_of_a_deadlist_entry_to_release(
+    entry: &DeadlistEntry,
+    locations: &[LocationEntry; 2],
+    allocator: &PoolAllocator,
+) -> Result<Placement, PublishError> {
+    let role = role_standing_for_the_unit_of_a_deadlist_entry(entry)?;
+    placement_to_release_after_checking_every_device(role, locations, allocator).map_err(|cause| {
+        PublishError::DeadlistEntryNotReleasable {
+            mapping_key: entry.mapping_key.clone(),
+            refusal: DeadlistEntryReleaseRefusal::PlacementNotReleasable(Box::new(cause)),
+        }
+    })
+}
+
+/// 一条 deadlist 条目指的单元按类标签归到的那一类用户可见角色（[`user_visible_role_standing_for_a_unit_class`]）。
+///
+/// # Errors
+/// 映射 key 的类标签不认识 ⇒ `DeadlistEntryNotReleasable { refusal: UnitClassOutsideTheUserVisibleOnes }`。
+fn role_standing_for_the_unit_of_a_deadlist_entry(
+    entry: &DeadlistEntry,
+) -> Result<TransactionUnit, PublishError> {
+    let unit_class = entry.mapping_key.first().copied().unwrap_or(0);
+    user_visible_role_standing_for_a_unit_class(unit_class).ok_or_else(|| {
+        PublishError::DeadlistEntryNotReleasable {
+            mapping_key: entry.mapping_key.clone(),
+            refusal: DeadlistEntryReleaseRefusal::UnitClassOutsideTheUserVisibleOnes { unit_class },
+        }
+    })
+}
+
 /// 分配记录树这次重写哪几个节点、这一版有哪几个节点（D8（核心索引结构） 已定项 14：按位置寻址，只重写内容变了的叶与它们的祖先，
 /// 没有记录的一段不写节点）。它们要在取落点之前定下来（角色清单、出生序号、映射 key、中央映射树的形状都要它），而取落点又会改记录——
 /// 这次写的每个单元、连分配记录树自己的节点，每盘各记一条；换下的每一份把它那条改成已释放（D3（空间分配） 已定项 7；这就是释放链）。
@@ -6885,6 +8797,10 @@ fn placements_released_by_this_publish(
 /// # Errors
 /// 同 [`PublishPlan::resolve`]、[`placements_released_by_this_publish`]，与拷贝上取不到落点（`PlacementRefused`）——
 /// 真发时同一个落点同样取不到；轮数用完 ⇒ `AllocationRecordTreeRewriteSetDidNotSettle`。都在任何落盘动作之前交回。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：计划、上一版、树号、记账行的盘、节点容量、分配器、删除这一步、这次要搬的单元"
+)]
 fn settle_the_allocation_record_tree(
     plan: &PublishPlan<'_>,
     previous: Option<&TransactionOutput>,
@@ -6893,8 +8809,13 @@ fn settle_the_allocation_record_tree(
     capacities: CodeTwoTreeNodeCapacities,
     allocator: &PoolAllocator,
     deletion_and_lineage: &DeletionAndLineageOfThePublish,
+    relocations: &[RelocationOfThePublish],
 ) -> Result<SettledPublish, PublishError> {
     let geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
+    let mapping_keys_moved_by_this_publish: Vec<Vec<u8>> = relocations
+        .iter()
+        .map(|relocation| relocation.mapping_key.clone())
+        .collect();
     let (previous_nodes, previous_records): (
         BTreeSet<AllocationRecordTreeNode>,
         &[AllocationRecord],
@@ -6925,9 +8846,33 @@ fn settle_the_allocation_record_tree(
             capacities,
             allocation_plan,
             deletion_and_lineage,
+            &mapping_keys_moved_by_this_publish,
         )?;
-        let (previous_roles_replaced, release) =
+        let (previous_roles_replaced, mut release) =
             placements_released_by_this_publish(plan, previous, &resolved, allocator)?;
+        // 这次重写本来就换下的角色不另搬：它的旧落点经 COW 释放、新落点不在活 R 里（判决 G3 的一批只搬「照抄」的单元）。
+        let relocations_in_effect: Vec<RelocationOfThePublish> = relocations
+            .iter()
+            .filter(|relocation| match relocation.holder {
+                RelocatedUnitHolder::RoleOfThisVersion(role) => {
+                    !previous_roles_replaced.contains(&role)
+                }
+                RelocatedUnitHolder::KeptForTheSnapshots => true,
+            })
+            .cloned()
+            .collect();
+        for relocation in &relocations_in_effect {
+            let source = placement_to_release_after_checking_every_device(
+                relocation.role_for_the_checks,
+                &relocation.source_locations,
+                allocator,
+            )?;
+            assert_eq!(
+                source, relocation.source,
+                "搬迁的源落点就是映射位置：publish_relocation_batch 按同一个上一版的映射核过"
+            );
+            release.push(source);
+        }
         let mut rehearsal = allocator.clone();
         for placement in &release {
             rehearsal.release(*placement, plan.txg);
@@ -6943,6 +8888,9 @@ fn settle_the_allocation_record_tree(
             nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));
             continue;
         }
+        for relocation in &relocations_in_effect {
+            allocate_relocation_destination(&mut rehearsal, relocation)?;
+        }
         for identity in &resolved.rewritten_roles {
             allocate_placement_for_role(&mut rehearsal, *identity, plan.txg)?;
         }
@@ -6954,6 +8902,7 @@ fn settle_the_allocation_record_tree(
                 resolved,
                 previous_roles_replaced,
                 release,
+                relocations_in_effect,
             });
         }
         rewritten.extend(changed);
@@ -7707,6 +9656,8 @@ struct DeletionTreeUnits {
     deleted_inodes_root: Option<DeletionTreeNodeUnit>,
     tombstone: ContainerTreeUnits<TombstoneContainerIndexInTree, TombstoneRecord>,
     lineage: ContainerTreeUnits<LineageContainerIndexInTree, LineageRecord>,
+    /// logged_ops 根（没有条目时没有，判决 G1）。
+    logged_operations_root: Option<DeletionTreeNodeUnit>,
     retirement: ContainerTreeUnits<RetirementContainerIndexInTree, RetirementRecord>,
 }
 
@@ -7966,6 +9917,47 @@ fn build_deletion_tree_units(
             })
         }
     };
+    // logged_ops 根（判决 G1）：码 2 层级 0，条目 44 字节 = key 10 + value 34，key 区间是首末两条的 key。
+    let logged_operations_tree = checkpoint.trees.logged_operations;
+    let logged_operations_root = match resolved.logged_operations_root {
+        LoggedOperationsRootOrigin::NoRoot => None,
+        LoggedOperationsRootOrigin::CarriedFromThePreviousVersion => Some(DeletionTreeNodeUnit {
+            pointer: carried_previous().tree_root_pointer(logged_operations_tree.0),
+            rewritten: None,
+        }),
+        LoggedOperationsRootOrigin::RewrittenThisPublish => {
+            let birth_sequence = sequences.next(logged_operations_tree, txg, instance);
+            let entries = logged_operations_root_entries(&resolved.logged_operations);
+            let key_of = |entry: &Vec<u8>| entry[..LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES].to_vec();
+            let smallest_key = key_of(entries.first().expect("有条目才有根"));
+            let largest_key = key_of(entries.last().expect("有条目才有根"));
+            let bytes = build_index_node(
+                logged_operations_tree,
+                0,
+                LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES,
+                &smallest_key,
+                &largest_key,
+                txg,
+                checkpoint.filesystem_identifier,
+                instance,
+                birth_sequence,
+                u16::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44"),
+                &entries,
+            );
+            Some(DeletionTreeNodeUnit {
+                pointer: pointer_of(
+                    logged_operations_tree,
+                    TransactionUnit::LoggedOperationsRoot,
+                    &bytes,
+                    birth_sequence,
+                ),
+                rewritten: Some(RewrittenDeletionTreeNode {
+                    bytes,
+                    birth_sequence,
+                }),
+            })
+        }
+    };
     let carried_tombstone_container_pointer = |position: usize| {
         carried_previous()
             .tombstone_containers
@@ -8065,6 +10057,7 @@ fn build_deletion_tree_units(
         deleted_inodes_root,
         tombstone,
         lineage,
+        logged_operations_root,
         retirement,
     }
 }
@@ -8109,6 +10102,8 @@ fn publish_admitted<Device: BlockDevice>(
     trees: FileVersionTreeIdentifiers,
     unmount_marker: UnmountMarker,
     lineage: LineageStepOfThePublish,
+    snapshot: SnapshotStepOfThePublish,
+    relocations: &[RelocationOfThePublish],
 ) -> Result<(PublishWrites, TransactionOutput), PublishError> {
     let rewritten = &resolved.rewritten_roles;
     let txg = plan.txg;
@@ -8120,8 +10115,6 @@ fn publish_admitted<Device: BlockDevice>(
         roles_named_by_each_record_of_the_publish(rewritten, named_entry_capacity);
     let transaction_offset_of_each_record =
         transaction_offset_of_each_record_of_the_publish(rewritten, named_entry_capacity);
-    let records_in_this_publish =
-        u64::try_from(roles_named_by_each_record.len()).expect("记录条数");
     // 一个数据单元一个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案）：第 k 个事务是第 `plan.transaction + k` 号，
     // 共享的提交内生块在最后一个事务的记录里点名 ⇒ 它们头里的写序取最后那个事务的号（数据单元各带自己那个事务的号）。
     let last_transaction_of_this_publish = plan.transaction
@@ -8157,6 +10150,96 @@ fn publish_admitted<Device: BlockDevice>(
     if previous.is_none() {
         allocator.forget_the_allocation_record_tree_of_the_version_without_file();
     }
+    // 隔离的那几份记进这次挂载的常驻隔离集（整理签发之后 R 里出现隔离槽就丢意图，攻方 K-c，`compaction_plan::region_holds_a_quarantined_slot`）。
+    for copy in quarantine {
+        allocator
+            .compaction_mut()
+            .note_quarantined_placement(copy.placement.slot, copy.placement.span);
+    }
+    // 整理的一批：源落点上面已经随释放进了 defer；目的落点在这次重写的角色之前取（固定点里走的是同一串，`settle_the_allocation_record_tree`），
+    // 分配代写单元的诞生代。目的地定下之后，上一版拷一份、把搬走的单元的映射位置换到目的地（`take_the_mapping_positions_over_the_location_hints`）：
+    // 照抄这些单元的角色、装映射条目、重写它们的父节点与树表时，读的都是这一份。
+    let mut relocated_copies: Vec<(PublishedUnit, NamedUnit, UnitRelocatedByThisPublish)> =
+        Vec::with_capacity(relocations.len());
+    for relocation in relocations {
+        let destination = allocate_relocation_destination(allocator, relocation)?;
+        let destination_locations = pool.location_entries(destination.slot, &relocation.bytes);
+        let mut key_tail = [0u8; 10];
+        key_tail.copy_from_slice(&relocation.mapping_key[17..27]);
+        let birth_tree = TreeIdentifier(u64::from_le_bytes(
+            relocation.mapping_key[1..9]
+                .try_into()
+                .expect("映射 key 27 字节：类标签 1 + 出生树 8"),
+        ));
+        relocated_copies.push((
+            PublishedUnit {
+                slot: destination.slot,
+                identity: relocation.role_for_the_checks,
+                bytes: relocation.bytes.clone(),
+            },
+            NamedUnit {
+                locations: destination_locations,
+                unit_class: relocation.mapping_key[0],
+                birth_tree,
+                birth_txg: relocation.birth_txg,
+                key_tail,
+            },
+            UnitRelocatedByThisPublish {
+                mapping_key: relocation.mapping_key.clone(),
+                holder: relocation.holder,
+                source: relocation.source_locations,
+                destination: destination_locations,
+            },
+        ));
+    }
+    // 映射条目换到目的地的那几把 key（只供测试的必红开关让映射留在源落点上时是空的）。
+    let destination_of_key: BTreeMap<Vec<u8>, [LocationEntry; 2]> =
+        match allocator.compaction().switches.relocation_publish_order {
+            RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping => {
+                relocated_copies
+                    .iter()
+                    .map(|(_, _, relocated)| (relocated.mapping_key.clone(), relocated.destination))
+                    .collect()
+            }
+            RelocationPublishOrder::SourceReleasedWhileTheMappingStillNamesItForTests => {
+                BTreeMap::new()
+            }
+        };
+    // 留存的映射条目（被快照钉住的单元）照这次之后的那一份，搬走的换到目的地：装中央映射树与交回的这一版都读它。
+    let kept_mapping_entries_after_this_publish: Vec<(Vec<u8>, [LocationEntry; 2])> = resolved
+        .deadlist
+        .kept_mapping_entries_after_this_publish
+        .iter()
+        .map(|(key, locations)| {
+            (
+                key.clone(),
+                destination_of_key.get(key).copied().unwrap_or(*locations),
+            )
+        })
+        .collect();
+    let previous_with_the_relocated_units_at_their_destinations: Option<TransactionOutput> =
+        match (previous, destination_of_key.is_empty()) {
+            (Some(previous_version), false) => {
+                let mut relocated_version = previous_version.clone();
+                take_the_mapping_positions_over_the_location_hints(&mut relocated_version, |key| {
+                    destination_of_key.get(key).copied()
+                });
+                for (key, locations) in
+                    &mut relocated_version.mapping_entries_kept_for_the_snapshots
+                {
+                    if let Some(destination) = destination_of_key.get(key) {
+                        *locations = *destination;
+                    }
+                }
+                Some(relocated_version)
+            }
+            (Some(_), true) | (None, _) => None,
+        };
+    let previous: Option<&TransactionOutput> =
+        match &previous_with_the_relocated_units_at_their_destinations {
+            Some(relocated_version) => Some(relocated_version),
+            None => previous,
+        };
 
     // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。
     let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
@@ -8422,6 +10505,30 @@ fn publish_admitted<Device: BlockDevice>(
         &mut sequences,
     );
 
+    // deadlist 树（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：叶条目就是 43 字节的 key（value 空），按计划装；
+    // 这次没有条目进出时整棵照抄，一条都不剩时零根（一个节点都不装）。
+    let deadlist_built = build_multi_level_tree(
+        MultiLevelCodeTwoTree::Deadlist,
+        trees.deadlist,
+        &resolved.deadlist_tree,
+        previous,
+        &resolved
+            .deadlist
+            .entries_after_this_publish
+            .iter()
+            .map(|entry| {
+                let key_bytes = entry.key_bytes();
+                (
+                    CodeTwoTreeKey::new(&key_bytes, CodeTwoKeyFieldWidths::DEADLIST),
+                    key_bytes,
+                )
+            })
+            .collect(),
+        &multi_level_tree_context,
+        &slots,
+        &mut sequences,
+    );
+
     // 删除那两棵树（D8（核心索引结构） 已定项 16、D18（块里携带什么信息） 已定项 24）：不是删除的发布全部照抄。
     let deletion_tree_units =
         build_deletion_tree_units(&checkpoint, resolved, previous, &slots, &mut sequences);
@@ -8460,6 +10567,12 @@ fn publish_admitted<Device: BlockDevice>(
     };
     let allocation_pointer = allocation_built.version.root_pointer();
     let accounting_pointer = accounting_built.version.root_pointer();
+    let deadlist_pointer = deadlist_built
+        .version
+        .pointers
+        .last()
+        .copied()
+        .unwrap_or_else(NodePointer::empty_root);
 
     // t7 中央映射树（字节表三·二）：码 1 每个数据单元一条 + 码 2 / 码 3 的 extent 树每个节点、每片 inode 叶容器、inode 根、
     // 分配记录树每个节点、记账树的每个节点各一条；映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
@@ -8535,6 +10648,20 @@ fn publish_admitted<Device: BlockDevice>(
             pointer.locations,
         ));
     }
+    for (node, pointer) in deadlist_built
+        .version
+        .shape
+        .nodes()
+        .iter()
+        .zip(&deadlist_built.version.pointers)
+    {
+        mapped_units_with_locations.push((
+            MultiLevelCodeTwoTree::Deadlist
+                .role_of_node(node.position, &deadlist_built.version.shape),
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
+            pointer.locations,
+        ));
+    }
     if let Some(root) = &deletion_tree_units.deleted_inodes_root {
         mapped_units_with_locations.push((
             TransactionUnit::DeletedInodesRoot,
@@ -8584,6 +10711,13 @@ fn publish_admitted<Device: BlockDevice>(
             root.pointer.locations,
         ));
     }
+    if let Some(root) = &deletion_tree_units.logged_operations_root {
+        mapped_units_with_locations.push((
+            TransactionUnit::LoggedOperationsRoot,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, root.pointer),
+            root.pointer.locations,
+        ));
+    }
     for container in &deletion_tree_units.retirement.containers {
         mapped_units_with_locations.push((
             TransactionUnit::RetirementLeafContainer(container.index),
@@ -8613,9 +10747,16 @@ fn publish_admitted<Device: BlockDevice>(
         mapped_units, resolved.mapped_units,
         "中央映射树的形状按 resolve 现算的那一份映射 key 算过：这里装出来的 key 要逐项相等（出生序号按 bump 次序发，D19 已定项 9）"
     );
+    // 被活快照钉住的单元的映射条目留在树里（判决 S3 的 α）：key 与两条位置项照上一版，这一版的树不引用它们。
     let mapping_entries_by_key: BTreeMap<CodeTwoTreeKey, Vec<u8>> = mapped_units_with_locations
         .iter()
-        .map(|(_, key, locations)| {
+        .map(|(_, key, locations)| (key, locations))
+        .chain(
+            kept_mapping_entries_after_this_publish
+                .iter()
+                .map(|(key, locations)| (key, locations)),
+        )
+        .map(|(key, locations)| {
             (
                 CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
                 build_mapping_entry(key, *locations),
@@ -8640,15 +10781,73 @@ fn publish_admitted<Device: BlockDevice>(
 
     // t8 树表单元：十二条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
     // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：inode 树写自己，extent 树、`deleted_inodes` 与墓碑树写它服务的头（inode 树的树 ID），其余 0。
+    // `previous_snapshot_txg`（D5（快照 / 空间记账机制） 已定项 9）只写在活头 inode 树那一条与码 14 上：活头那一条照上一版的值续上
+    // （没有上一版、或还没打过快照是 0），建快照那次换成这次的 checkpoint_txg（S.txg，已定项 14）；其余十一条写 0。
+    let previous_snapshot_txg_of_the_live_inode_tree_before_this_publish = previous
+        .and_then(|previous_version| {
+            previous_version
+                .tree_table_entries
+                .iter()
+                .find(|entry| entry.kind == TREE_KIND_INODE)
+        })
+        .map_or(CheckpointTxg(0), |entry| entry.previous_snapshot_txg);
+    // 删快照的意图那次：被删的那个快照的前驱 P 传给那一刻的下一侧（D5（快照 / 空间记账机制） 已定项 12、已定项 13「prev 要跟着传递」，
+    // 不许拆到以后的发布）——下一侧是活头就换活头 inode 条目的前驱，是下一条码 14 就换那一条的。
+    let predecessor_passed_on_by_the_destruction_intent: Option<(
+        NextSideOfASnapshot,
+        CheckpointTxg,
+    )> = match snapshot {
+        SnapshotStepOfThePublish::DestructionIntent {
+            snapshot: snapshot_being_destroyed,
+        } => {
+            let previous_version = previous.expect(
+                "删快照的意图接在有那个快照的一版之后：publish_snapshot_destruction_intent 判过",
+            );
+            let destroyed_entry = snapshot_inode_root_entry(
+                &previous_version.tree_table_entries,
+                snapshot_being_destroyed,
+            )
+            .expect("被删的快照在上一版树表里：publish_snapshot_destruction_intent 判过");
+            let next_side = next_side_of_the_snapshot(
+                &previous_version.tree_table_entries,
+                snapshot_being_destroyed,
+            )
+            .expect("被删的快照在上一版树表里：同上");
+            Some((next_side, destroyed_entry.previous_snapshot_txg))
+        }
+        SnapshotStepOfThePublish::NoNewSnapshot
+        | SnapshotStepOfThePublish::CreateSnapshot { .. }
+        | SnapshotStepOfThePublish::DestructionBatch { .. }
+        | SnapshotStepOfThePublish::DestructionFinalBatch { .. } => None,
+    };
+    let previous_snapshot_txg_of_the_live_inode_tree = match snapshot {
+        SnapshotStepOfThePublish::CreateSnapshot { .. } => txg,
+        SnapshotStepOfThePublish::NoNewSnapshot
+        | SnapshotStepOfThePublish::DestructionIntent { .. }
+        | SnapshotStepOfThePublish::DestructionBatch { .. }
+        | SnapshotStepOfThePublish::DestructionFinalBatch { .. } => {
+            match predecessor_passed_on_by_the_destruction_intent {
+                Some((NextSideOfASnapshot::LiveHead, predecessor)) => predecessor,
+                Some((NextSideOfASnapshot::Snapshot(_), _)) | None => {
+                    previous_snapshot_txg_of_the_live_inode_tree_before_this_publish
+                }
+            }
+        }
+    };
     let table_entry =
         |kind: u16, tree: TreeIdentifier, root: NodePointer, head_identifier: u64| TreeTableEntry {
             kind,
             tree,
             root,
+            previous_snapshot_txg: if kind == TREE_KIND_INODE {
+                previous_snapshot_txg_of_the_live_inode_tree
+            } else {
+                CheckpointTxg(0)
+            },
             birth_txg: plan.tree_birth_txg,
             head_identifier,
         };
-    let tree_table_entries = vec![
+    let mut tree_table_entries = vec![
         table_entry(
             TREE_KIND_EXTENT,
             trees.extent,
@@ -8685,15 +10884,11 @@ fn publish_admitted<Device: BlockDevice>(
             NodePointer::empty_root(),
             0,
         ),
-        table_entry(
-            TREE_KIND_DEADLIST,
-            trees.deadlist,
-            NodePointer::empty_root(),
-            0,
-        ),
+        // deadlist 树一条条目都没有时零根（同 day-1 注册的那一形，D5（快照 / 空间记账机制） 已定项 11）。
+        table_entry(TREE_KIND_DEADLIST, trees.deadlist, deadlist_pointer, 0),
         // 第一个文件版本那次零根注册的四棵（D8（核心索引结构） 已定项 16 第 3 条、D18（块里携带什么信息） 已定项 24 第 1 / 5 / 6 条）：
         // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：`deleted_inodes` 与墓碑树写 inode 树的树 ID，谱系树写 0（全池一份），
-        // logged_ops 写 0（字段表未定）。
+        // logged_ops 写 0（全池一份）。
         // 队列空 / 墓碑树零根时根指针置零（D8（核心索引结构） 已定项 16 第 2、5 条）。
         table_entry(
             TREE_KIND_DELETED_INODES,
@@ -8725,10 +10920,14 @@ fn publish_admitted<Device: BlockDevice>(
                 .map_or_else(NodePointer::empty_root, |root| root.pointer),
             0,
         ),
+        // logged_ops（判决 G1）：没有条目时根指针置零；签发那次指着新写的根，之后照抄，删光那次置零（随现行那一版）。头 ID 写 0（全池一份）。
         table_entry(
             TREE_KIND_LOGGED_OPERATIONS,
             trees.logged_operations,
-            NodePointer::empty_root(),
+            deletion_tree_units
+                .logged_operations_root
+                .as_ref()
+                .map_or_else(NodePointer::empty_root, |root| root.pointer),
             0,
         ),
         // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）：day-1 零根注册、排在最后，
@@ -8750,13 +10949,126 @@ fn publish_admitted<Device: BlockDevice>(
             .all(|pair| pair[0].tree < pair[1].tree),
         "树表条目按树 ID 升序（D8（核心索引结构） 已定项 8 排序契约）：十三个号连号发、中央映射树不进树表，剩下十二条仍按发号次序升序"
     );
+    // 只读快照的条目（码 14、15）随每一次发布续上（判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S2）：
+    // 上一版树表里的照盘上原样、照原次序接在十二条之后（它们的树号是建快照那次从水位发的，比十二棵树的号都大）；建快照这一次再追加一对。
+    // 删快照的最后一批同一次发布删掉被删那一对（判决 S4）；意图那次下一侧是下一条码 14 时换它的前驱。
+    let snapshot_whose_entries_the_final_batch_deletes = match snapshot {
+        SnapshotStepOfThePublish::DestructionFinalBatch {
+            snapshot: snapshot_being_destroyed,
+        } => Some(snapshot_being_destroyed),
+        SnapshotStepOfThePublish::NoNewSnapshot
+        | SnapshotStepOfThePublish::CreateSnapshot { .. }
+        | SnapshotStepOfThePublish::DestructionIntent { .. }
+        | SnapshotStepOfThePublish::DestructionBatch { .. } => None,
+    };
+    let snapshot_entries_carried_from_the_previous_version: Vec<TreeTableEntry> = previous
+        .map(|previous_version| {
+            previous_version
+                .tree_table_entries
+                .iter()
+                .filter(|entry| {
+                    entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
+                        || entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT
+                })
+                .filter(|entry| {
+                    snapshot_whose_entries_the_final_batch_deletes.is_none_or(|deleted| {
+                        let is_the_deleted_inode_root =
+                            entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT && entry.tree == deleted;
+                        let is_the_deleted_extent_root = entry.kind
+                            == TREE_KIND_SNAPSHOT_EXTENT_ROOT
+                            && entry.head_identifier == deleted.0;
+                        !is_the_deleted_inode_root && !is_the_deleted_extent_root
+                    })
+                })
+                .map(
+                    |entry| match predecessor_passed_on_by_the_destruction_intent {
+                        Some((NextSideOfASnapshot::Snapshot(next), predecessor))
+                            if entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
+                                && entry.tree == next.tree =>
+                        {
+                            TreeTableEntry {
+                                kind: entry.kind,
+                                tree: entry.tree,
+                                root: entry.root,
+                                previous_snapshot_txg: predecessor,
+                                birth_txg: entry.birth_txg,
+                                head_identifier: entry.head_identifier,
+                            }
+                        }
+                        Some(_) | None => *entry,
+                    },
+                )
+                .collect()
+        })
+        .unwrap_or_default();
+    tree_table_entries.extend(snapshot_entries_carried_from_the_previous_version);
+    // 这一版带「销毁中」记号的码 14：上一版的，加意图那次的，减最后一批的（判决 S4：意图写在条目自己身上，删掉那一对时记号跟着没了）。
+    let mut snapshots_being_destroyed: Vec<TreeIdentifier> = previous
+        .map(|previous_version| previous_version.snapshots_being_destroyed.clone())
+        .unwrap_or_default();
+    match snapshot {
+        SnapshotStepOfThePublish::DestructionIntent {
+            snapshot: snapshot_being_destroyed,
+        } => {
+            snapshots_being_destroyed.push(snapshot_being_destroyed);
+            snapshots_being_destroyed.sort_unstable();
+        }
+        SnapshotStepOfThePublish::DestructionFinalBatch {
+            snapshot: snapshot_being_destroyed,
+        } => snapshots_being_destroyed.retain(|tree| *tree != snapshot_being_destroyed),
+        SnapshotStepOfThePublish::NoNewSnapshot
+        | SnapshotStepOfThePublish::CreateSnapshot { .. }
+        | SnapshotStepOfThePublish::DestructionBatch { .. } => {}
+    }
+    match snapshot {
+        SnapshotStepOfThePublish::NoNewSnapshot
+        | SnapshotStepOfThePublish::DestructionIntent { .. }
+        | SnapshotStepOfThePublish::DestructionBatch { .. }
+        | SnapshotStepOfThePublish::DestructionFinalBatch { .. } => {}
+        SnapshotStepOfThePublish::CreateSnapshot {
+            snapshot_inode_root_tree,
+            snapshot_extent_root_tree,
+        } => {
+            // 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1：根指针逐字抄这一版活头的两个根；诞生 txg = S.txg = 这次的
+            // checkpoint_txg；码 14 的 `previous_snapshot_txg` = 捕获那一刻活头的值（换成 S.txg 之前那一个）、头 ID = 活头 inode 树的树号；
+            // 码 15 的头 ID = 配对的码 14 的树号、`previous_snapshot_txg` 0；flags 0（位 0「销毁中」这一批不写）。
+            tree_table_entries.push(TreeTableEntry {
+                kind: TREE_KIND_SNAPSHOT_INODE_ROOT,
+                tree: snapshot_inode_root_tree,
+                root: inode_root_pointer,
+                previous_snapshot_txg:
+                    previous_snapshot_txg_of_the_live_inode_tree_before_this_publish,
+                birth_txg: txg,
+                head_identifier: trees.inode.0,
+            });
+            tree_table_entries.push(TreeTableEntry {
+                kind: TREE_KIND_SNAPSHOT_EXTENT_ROOT,
+                tree: snapshot_extent_root_tree,
+                root: extent_pointer,
+                previous_snapshot_txg: CheckpointTxg(0),
+                birth_txg: txg,
+                head_identifier: snapshot_inode_root_tree.0,
+            });
+        }
+    }
+    assert!(
+        tree_table_entries
+            .windows(2)
+            .all(|pair| pair[0].tree < pair[1].tree),
+        "快照条目的树号在建它那次从水位发、只增不减（D8（核心索引结构） 已定项 8 ②）：续上与追加之后树表仍按树 ID 严格升序"
+    );
     let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
     let tree_table_unit = build_index_node(
         TreeIdentifier(TREE_IDENTIFIER_NONE),
         0,
         TREE_TABLE_KEY_WIDTH,
         &trees.extent.0.to_le_bytes(),
-        &trees.retirement.0.to_le_bytes(),
+        &tree_table_entries
+            .last()
+            .expect("十二条之后才续快照条目：树表不空")
+            .tree
+            .0
+            .to_le_bytes(),
         txg,
         filesystem_identifier,
         instance,
@@ -8764,7 +11076,16 @@ fn publish_admitted<Device: BlockDevice>(
         u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"),
         &tree_table_entries
             .iter()
-            .map(TreeTableEntry::to_bytes)
+            .map(|entry| {
+                let mark = if entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
+                    && snapshots_being_destroyed.contains(&entry.tree)
+                {
+                    SnapshotDestructionMark::BeingDestroyed
+                } else {
+                    SnapshotDestructionMark::NotBeingDestroyed
+                };
+                entry.to_bytes_with_destruction_mark(mark)
+            })
             .collect::<Vec<_>>(),
     );
     let tree_table_pointer = node_pointer(
@@ -8837,6 +11158,7 @@ fn publish_admitted<Device: BlockDevice>(
         );
     }
     units.extend(accounting_built.units.iter().cloned());
+    units.extend(deadlist_built.units.iter().cloned());
     let deletion_tree_unit =
         |identity: TransactionUnit, node: &DeletionTreeNodeUnit| match &node.rewritten {
             Some(rewritten_node) => rewritten_unit(identity, rewritten_node.bytes.clone()),
@@ -8888,6 +11210,12 @@ fn publish_admitted<Device: BlockDevice>(
     if let Some(root) = &deletion_tree_units.lineage.root {
         units.push(deletion_tree_unit(TransactionUnit::LineageRoot, root));
     }
+    if let Some(root) = &deletion_tree_units.logged_operations_root {
+        units.push(deletion_tree_unit(
+            TransactionUnit::LoggedOperationsRoot,
+            root,
+        ));
+    }
     for container in &deletion_tree_units.retirement.containers {
         units.push(deletion_tree_unit(
             TransactionUnit::RetirementLeafContainer(container.index),
@@ -8925,164 +11253,179 @@ fn publish_admitted<Device: BlockDevice>(
     }
 
     // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的每个角色一项，key 尾段与映射 key 共用；照抄的不点名。
-    let key_tail_of = |identity: TransactionUnit| -> [u8; 10] {
-        match identity {
-            TransactionUnit::Data(index) => data_key_tail(
-                data_pointers
-                    .get(usize::try_from(index.0).expect("单元序号"))
-                    .expect("点名的数据单元是这次装的那几个之一")
-                    .write_order,
-            ),
-            TransactionUnit::ExtentLowerNode(_)
-            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
-            | TransactionUnit::ExtentRoot => node_key_tail(
-                instance,
-                *extent_built
-                    .birth_sequences_of_rewritten_nodes
-                    .get(&identity)
-                    .expect("点名的 extent 树节点是这次重写的"),
-            ),
-            TransactionUnit::InodeLeafContainer(index) => node_key_tail(
-                instance,
-                inode_tree_units
-                    .leaf_containers
-                    .get(index.position())
-                    .expect("点名的叶容器是这次重写的那几片之一")
-                    .pointer
-                    .birth_sequence,
-            ),
-            TransactionUnit::InodeRoot => node_key_tail(
-                instance,
-                inode_tree_units
-                    .rewritten_root
-                    .as_ref()
-                    .expect("点名 inode 根的发布一定重写了它")
-                    .birth_sequence,
-            ),
-            TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
-            | TransactionUnit::AllocationTree => node_key_tail(
-                instance,
-                *allocation_built
-                    .birth_sequences_of_rewritten_nodes
-                    .get(&identity)
-                    .expect("点名的分配记录树节点是这次重写的"),
-            ),
-            TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
-            | TransactionUnit::AccountingTree => node_key_tail(
-                instance,
-                accounting_built.birth_sequence_of_a_rewritten_node(identity),
-            ),
-            TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
-                node_key_tail(
+    let key_tail_of =
+        |identity: TransactionUnit| -> [u8; 10] {
+            match identity {
+                TransactionUnit::Data(index) => data_key_tail(
+                    data_pointers
+                        .get(usize::try_from(index.0).expect("单元序号"))
+                        .expect("点名的数据单元是这次装的那几个之一")
+                        .write_order,
+                ),
+                TransactionUnit::ExtentLowerNode(_)
+                | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+                | TransactionUnit::ExtentRoot => node_key_tail(
                     instance,
-                    mapping_built.birth_sequence_of_a_rewritten_node(identity),
-                )
-            }
-            TransactionUnit::DeletedInodesRoot => node_key_tail(
-                instance,
-                deletion_tree_units
-                    .deleted_inodes_root
-                    .as_ref()
-                    .and_then(|root| root.rewritten.as_ref())
-                    .expect("点名 deleted_inodes 根的发布一定重写了它")
-                    .birth_sequence,
-            ),
-            TransactionUnit::TombstoneLeafContainer(index) => node_key_tail(
-                instance,
-                deletion_tree_units
-                    .tombstone
-                    .containers
-                    .get(index.position())
-                    .and_then(|container| container.node.rewritten.as_ref())
-                    .expect("点名的墓碑容器是这次重写的")
-                    .birth_sequence,
-            ),
-            TransactionUnit::TombstoneRoot => node_key_tail(
-                instance,
-                deletion_tree_units
-                    .tombstone
-                    .root
-                    .as_ref()
-                    .and_then(|root| root.rewritten.as_ref())
-                    .expect("点名墓碑树根的发布一定重写了它")
-                    .birth_sequence,
-            ),
-            TransactionUnit::TombstoneNodeBelowTheRoot(_) => node_key_tail(
-                instance,
-                deletion_tree_units
-                    .tombstone
-                    .nodes_below_the_root
-                    .iter()
-                    .find(|node| node.identity == identity)
-                    .and_then(|node| node.node.rewritten.as_ref())
-                    .expect("点名的墓碑树根之下节点是这次重写的")
-                    .birth_sequence,
-            ),
-            TransactionUnit::LineageLeafContainer(index) => node_key_tail(
-                instance,
-                deletion_tree_units
-                    .lineage
-                    .containers
-                    .get(index.position())
-                    .and_then(|container| container.node.rewritten.as_ref())
-                    .expect("点名的谱系容器是这次重写的")
-                    .birth_sequence,
-            ),
-            TransactionUnit::LineageRoot => node_key_tail(
-                instance,
-                deletion_tree_units
-                    .lineage
-                    .root
-                    .as_ref()
-                    .and_then(|root| root.rewritten.as_ref())
-                    .expect("点名谱系树根的发布一定重写了它")
-                    .birth_sequence,
-            ),
-            TransactionUnit::LineageNodeBelowTheRoot(_) => node_key_tail(
-                instance,
-                deletion_tree_units
-                    .lineage
-                    .nodes_below_the_root
-                    .iter()
-                    .find(|node| node.identity == identity)
-                    .and_then(|node| node.node.rewritten.as_ref())
-                    .expect("点名的谱系树根之下节点是这次重写的")
-                    .birth_sequence,
-            ),
-            TransactionUnit::RetirementLeafContainer(index) => node_key_tail(
-                instance,
-                deletion_tree_units
-                    .retirement
-                    .containers
-                    .get(index.position())
-                    .and_then(|container| container.node.rewritten.as_ref())
-                    .expect("点名的退役容器是这次重写的")
-                    .birth_sequence,
-            ),
-            TransactionUnit::RetirementRoot => node_key_tail(
-                instance,
-                deletion_tree_units
-                    .retirement
-                    .root
-                    .as_ref()
-                    .and_then(|root| root.rewritten.as_ref())
-                    .expect("点名退役树根的发布一定重写了它")
-                    .birth_sequence,
-            ),
-            TransactionUnit::RetirementNodeBelowTheRoot(_) => node_key_tail(
-                instance,
-                deletion_tree_units
-                    .retirement
-                    .nodes_below_the_root
-                    .iter()
-                    .find(|node| node.identity == identity)
-                    .and_then(|node| node.node.rewritten.as_ref())
-                    .expect("点名的退役树根之下节点是这次重写的")
-                    .birth_sequence,
-            ),
-            TransactionUnit::TreeTable => node_key_tail(instance, tree_table_sequence),
-            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
-                node_key_tail(
+                    *extent_built
+                        .birth_sequences_of_rewritten_nodes
+                        .get(&identity)
+                        .expect("点名的 extent 树节点是这次重写的"),
+                ),
+                TransactionUnit::InodeLeafContainer(index) => node_key_tail(
+                    instance,
+                    inode_tree_units
+                        .leaf_containers
+                        .get(index.position())
+                        .expect("点名的叶容器是这次重写的那几片之一")
+                        .pointer
+                        .birth_sequence,
+                ),
+                TransactionUnit::InodeRoot => node_key_tail(
+                    instance,
+                    inode_tree_units
+                        .rewritten_root
+                        .as_ref()
+                        .expect("点名 inode 根的发布一定重写了它")
+                        .birth_sequence,
+                ),
+                TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
+                | TransactionUnit::AllocationTree => node_key_tail(
+                    instance,
+                    *allocation_built
+                        .birth_sequences_of_rewritten_nodes
+                        .get(&identity)
+                        .expect("点名的分配记录树节点是这次重写的"),
+                ),
+                TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+                | TransactionUnit::AccountingTree => node_key_tail(
+                    instance,
+                    accounting_built.birth_sequence_of_a_rewritten_node(identity),
+                ),
+                TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
+                | TransactionUnit::DeadlistTree => node_key_tail(
+                    instance,
+                    deadlist_built.birth_sequence_of_a_rewritten_node(identity),
+                ),
+                TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
+                    node_key_tail(
+                        instance,
+                        mapping_built.birth_sequence_of_a_rewritten_node(identity),
+                    )
+                }
+                TransactionUnit::DeletedInodesRoot => node_key_tail(
+                    instance,
+                    deletion_tree_units
+                        .deleted_inodes_root
+                        .as_ref()
+                        .and_then(|root| root.rewritten.as_ref())
+                        .expect("点名 deleted_inodes 根的发布一定重写了它")
+                        .birth_sequence,
+                ),
+                TransactionUnit::LoggedOperationsRoot => node_key_tail(
+                    instance,
+                    deletion_tree_units
+                        .logged_operations_root
+                        .as_ref()
+                        .and_then(|root| root.rewritten.as_ref())
+                        .expect("点名 logged_ops 根的发布一定重写了它")
+                        .birth_sequence,
+                ),
+                TransactionUnit::TombstoneLeafContainer(index) => node_key_tail(
+                    instance,
+                    deletion_tree_units
+                        .tombstone
+                        .containers
+                        .get(index.position())
+                        .and_then(|container| container.node.rewritten.as_ref())
+                        .expect("点名的墓碑容器是这次重写的")
+                        .birth_sequence,
+                ),
+                TransactionUnit::TombstoneRoot => node_key_tail(
+                    instance,
+                    deletion_tree_units
+                        .tombstone
+                        .root
+                        .as_ref()
+                        .and_then(|root| root.rewritten.as_ref())
+                        .expect("点名墓碑树根的发布一定重写了它")
+                        .birth_sequence,
+                ),
+                TransactionUnit::TombstoneNodeBelowTheRoot(_) => node_key_tail(
+                    instance,
+                    deletion_tree_units
+                        .tombstone
+                        .nodes_below_the_root
+                        .iter()
+                        .find(|node| node.identity == identity)
+                        .and_then(|node| node.node.rewritten.as_ref())
+                        .expect("点名的墓碑树根之下节点是这次重写的")
+                        .birth_sequence,
+                ),
+                TransactionUnit::LineageLeafContainer(index) => node_key_tail(
+                    instance,
+                    deletion_tree_units
+                        .lineage
+                        .containers
+                        .get(index.position())
+                        .and_then(|container| container.node.rewritten.as_ref())
+                        .expect("点名的谱系容器是这次重写的")
+                        .birth_sequence,
+                ),
+                TransactionUnit::LineageRoot => node_key_tail(
+                    instance,
+                    deletion_tree_units
+                        .lineage
+                        .root
+                        .as_ref()
+                        .and_then(|root| root.rewritten.as_ref())
+                        .expect("点名谱系树根的发布一定重写了它")
+                        .birth_sequence,
+                ),
+                TransactionUnit::LineageNodeBelowTheRoot(_) => node_key_tail(
+                    instance,
+                    deletion_tree_units
+                        .lineage
+                        .nodes_below_the_root
+                        .iter()
+                        .find(|node| node.identity == identity)
+                        .and_then(|node| node.node.rewritten.as_ref())
+                        .expect("点名的谱系树根之下节点是这次重写的")
+                        .birth_sequence,
+                ),
+                TransactionUnit::RetirementLeafContainer(index) => node_key_tail(
+                    instance,
+                    deletion_tree_units
+                        .retirement
+                        .containers
+                        .get(index.position())
+                        .and_then(|container| container.node.rewritten.as_ref())
+                        .expect("点名的退役容器是这次重写的")
+                        .birth_sequence,
+                ),
+                TransactionUnit::RetirementRoot => node_key_tail(
+                    instance,
+                    deletion_tree_units
+                        .retirement
+                        .root
+                        .as_ref()
+                        .and_then(|root| root.rewritten.as_ref())
+                        .expect("点名退役树根的发布一定重写了它")
+                        .birth_sequence,
+                ),
+                TransactionUnit::RetirementNodeBelowTheRoot(_) => node_key_tail(
+                    instance,
+                    deletion_tree_units
+                        .retirement
+                        .nodes_below_the_root
+                        .iter()
+                        .find(|node| node.identity == identity)
+                        .and_then(|node| node.node.rewritten.as_ref())
+                        .expect("点名的退役树根之下节点是这次重写的")
+                        .birth_sequence,
+                ),
+                TransactionUnit::TreeTable => node_key_tail(instance, tree_table_sequence),
+                TransactionUnit::InstanceTable
+                | TransactionUnit::InstanceTablePageAfterTheFirst(_) => node_key_tail(
                     instance,
                     instance_table_rewrite
                         .iter()
@@ -9090,18 +11433,18 @@ fn publish_admitted<Device: BlockDevice>(
                         .expect("点名实例表那一片的发布一定重写了整条链")
                         .pointer
                         .birth_sequence,
-                )
+                ),
             }
-        }
-    };
-    let written_units: Vec<&PublishedUnit> = rewritten
+        };
+    let written_units: Vec<&PublishedUnit> = relocated_copies
         .iter()
-        .map(|identity| {
+        .map(|(copy, _, _)| copy)
+        .chain(rewritten.iter().map(|identity| {
             units
                 .iter()
                 .find(|unit| unit.identity == *identity)
                 .expect("重写的角色都装了单元")
-        })
+        }))
         .collect();
     let named_unit_of = |identity: TransactionUnit| -> NamedUnit {
         let unit = units
@@ -9135,11 +11478,42 @@ fn publish_admitted<Device: BlockDevice>(
         // 两处都写就成了同一个量的两份手抄。`root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero` 钉住。
         allocation_record_tree_root: NodePointer::empty_root(),
     };
+    // 每条记录点名哪几项：整理的一批（不重写数据单元，一个事务）点名搬迁副本在前、这次重写的角色在后，按一条记录装几个点名项切
+    // （判决 G3「journal 点名项 = 搬迁副本 + 新写的节点，一个事务号」）；别的发布照角色切法。
+    let named_units_of_each_record: Vec<Vec<NamedUnit>> = if relocated_copies.is_empty() {
+        roles_named_by_each_record
+            .iter()
+            .map(|named_roles| named_roles.iter().copied().map(named_unit_of).collect())
+            .collect()
+    } else {
+        assert!(
+            transaction_offset_of_each_record
+                .iter()
+                .all(|transaction_offset| *transaction_offset == 0),
+            "整理的一批不重写数据单元：整次一个事务"
+        );
+        let named_units: Vec<NamedUnit> = relocated_copies
+            .iter()
+            .map(|(_, named, _)| *named)
+            .chain(rewritten.iter().copied().map(named_unit_of))
+            .collect();
+        named_units
+            .chunks(named_entry_capacity.named_entries_per_record())
+            .map(<[NamedUnit]>::to_vec)
+            .collect()
+    };
+    let transaction_offset_of_each_record: Vec<u64> = if relocated_copies.is_empty() {
+        transaction_offset_of_each_record
+    } else {
+        vec![0; named_units_of_each_record.len()]
+    };
+    let records_in_this_publish =
+        u64::try_from(named_units_of_each_record.len()).expect("记录条数");
     let mut written_records: Vec<WrittenJournalRecord> =
-        Vec::with_capacity(roles_named_by_each_record.len());
+        Vec::with_capacity(named_units_of_each_record.len());
     // 迭代次数的上界是这次的记录条数（≥ 1）；跨轮携带的只有已经装好的记录（下一条的反向链要罩前一条的头）。
-    for (record_offset_in_this_publish, named_roles) in
-        roles_named_by_each_record.iter().enumerate()
+    for (record_offset_in_this_publish, named_units) in
+        named_units_of_each_record.iter().enumerate()
     {
         let offset = u64::try_from(record_offset_in_this_publish).expect("记录序号");
         let back_chain = match written_records.last() {
@@ -9181,7 +11555,7 @@ fn publish_admitted<Device: BlockDevice>(
             new_instance_table: root.instance_table,
             new_allocation_record_tree_root: root.allocation_record_tree_root,
             new_unmount_marker: root.unmount_marker,
-            named: named_roles.iter().copied().map(named_unit_of).collect(),
+            named: named_units.clone(),
         };
         let bytes = record.to_bytes();
         written_records.push(WrittenJournalRecord { record, bytes });
@@ -9226,10 +11600,15 @@ fn publish_admitted<Device: BlockDevice>(
         accounting_entries,
         accounting_tree: accounting_built.version,
         central_mapping_tree: mapping_built.version,
+        deadlist_entries: resolved.deadlist.entries_after_this_publish.clone(),
+        deadlist_tree: deadlist_built.version,
+        mapping_entries_kept_for_the_snapshots: kept_mapping_entries_after_this_publish,
+        snapshots_being_destroyed,
         tree_table_entries,
         tree_identifiers: trees,
         inode_record,
         deleted_inodes: resolved.deleted_inodes.clone(),
+        logged_operations: resolved.logged_operations.clone(),
         tombstone_containers: deletion_tree_units
             .tombstone
             .containers
@@ -9295,6 +11674,10 @@ fn publish_admitted<Device: BlockDevice>(
         mapped_units,
         released: release.to_vec(),
         quarantined_after_release_checksum_mismatch: quarantine.to_vec(),
+        units_relocated_by_this_publish: relocated_copies
+            .into_iter()
+            .map(|(_, _, relocated)| relocated)
+            .collect(),
         key_order_mismatches,
         highest_transaction_number_in_this_instance: plan
             .highest_transaction_number_before_this_publish
@@ -9471,6 +11854,86 @@ fn build_multi_level_tree(
 #[cfg(test)]
 mod tests {
     use super::*;
+
+    /// D5（快照 / 空间记账机制） 已定项 14 的左边界：birth == S.txg 时快照引用它（进 deadlist），birth == S.txg + 1 时不引用（释放）；
+    /// 映射 key 不是 27 字节（birth 取不到）默认进 deadlist（「判据的输入取不到时默认进 deadlist」）。没打过快照（p = 0）时一律释放。
+    #[test]
+    fn a_unit_born_at_the_snapshot_txg_is_kept_and_one_born_after_it_is_released() {
+        let key_born_at = |birth: u64| {
+            crate::records::mapping_key_for_node(
+                UNIT_CLASS_INDEX_NODE,
+                NodePointer {
+                    head: PointerHead {
+                        birth_tree: TreeIdentifier(65_536),
+                        birth_txg: CheckpointTxg(birth),
+                    },
+                    locations: NodePointer::empty_root().locations,
+                    instance: InstanceGeneration(1),
+                    birth_sequence: BirthSequence(0),
+                },
+            )
+        };
+        assert_eq!(
+            snapshot_lifetime_verdict(&key_born_at(9), CheckpointTxg(9)),
+            SnapshotLifetimeVerdict::KeptOnTheDeadlist,
+            "birth == S.txg：快照引用它"
+        );
+        assert_eq!(
+            snapshot_lifetime_verdict(&key_born_at(10), CheckpointTxg(9)),
+            SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue,
+            "birth == S.txg + 1：比快照新，释放"
+        );
+        assert_eq!(
+            snapshot_lifetime_verdict(&key_born_at(3), CheckpointTxg(0)),
+            SnapshotLifetimeVerdict::ReleasedIntoTheDeferQueue,
+            "没打过快照：一律释放"
+        );
+        assert_eq!(
+            snapshot_lifetime_verdict(&key_born_at(10)[..26], CheckpointTxg(9)),
+            SnapshotLifetimeVerdict::KeptOnTheDeadlist,
+            "birth 取不到：默认进 deadlist"
+        );
+    }
+
+    /// 判决 S3：只有六类用户可见子树的角色问生死判定，池级元数据的 14 类（连 deadlist 树自己）一律直接进 defer。
+    #[test]
+    fn only_the_six_user_visible_roles_ask_the_snapshot_lifetime_judgement() {
+        let position = CodeTwoTreeNodePosition {
+            level: 0,
+            index_in_level: 0,
+        };
+        for asking in [
+            TransactionUnit::Data(DataUnitIndexInFile::FIRST),
+            TransactionUnit::ExtentRoot,
+            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
+            TransactionUnit::InodeRoot,
+        ] {
+            assert!(
+                asks_the_snapshot_lifetime_judgement(asking),
+                "{asking:?}：用户可见，问判定"
+            );
+        }
+        for not_asking in [
+            TransactionUnit::AllocationTree,
+            TransactionUnit::AccountingTree,
+            TransactionUnit::AccountingTreeNodeBelowTheRoot(position),
+            TransactionUnit::DeadlistTree,
+            TransactionUnit::DeadlistTreeNodeBelowTheRoot(position),
+            TransactionUnit::DeletedInodesRoot,
+            TransactionUnit::TombstoneRoot,
+            TransactionUnit::LineageRoot,
+            TransactionUnit::RetirementRoot,
+            TransactionUnit::MappingTree,
+            TransactionUnit::MappingTreeNodeBelowTheRoot(position),
+            TransactionUnit::TreeTable,
+            TransactionUnit::InstanceTable,
+        ] {
+            assert!(
+                !asks_the_snapshot_lifetime_judgement(not_asking),
+                "{not_asking:?}：池级元数据，直接进 defer"
+            );
+        }
+    }
     use singlefs_format::{INODE_LEAF_RECORDS, NEW_POOL_FILE_CREATION_TXG};
 
     #[test]
diff --git a/crates/singlefs-core/src/unlink.rs b/crates/singlefs-core/src/unlink.rs
index 9a1d7b59..a22b1f0f 100644
--- a/crates/singlefs-core/src/unlink.rs
+++ b/crates/singlefs-core/src/unlink.rs
@@ -102,6 +102,17 @@ impl UnlinkBatchSize {
     }
 }
 
+/// 删快照的一批放几条 deadlist 条目（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4：批大小 K = 144、测试开关压小，
+/// 「`DeletionSwitches` 旁」）：与 unlink 每批的数据单元数共用同一个开关（[`UnlinkBatchSize`]），产品 144、压小时取压的那个数——
+/// 删快照照 unlink 的过程做（第七节岔路 4），两件事的批同一个大小，开关不另立一个。
+///
+/// # Panics
+/// 批大小装不进 `usize`：产品 144、压小的至多 144。
+#[must_use]
+pub fn deadlist_entries_per_snapshot_destruction_batch(batch_size: UnlinkBatchSize) -> usize {
+    usize::try_from(batch_size.data_units_per_batch()).expect("批大小至多 144")
+}
+
 /// unlink 意图之后的一步（D8（核心索引结构） 已定项 16 第 2 条）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum UnlinkStep {
diff --git a/crates/singlefs-core/src/write_accounting.rs b/crates/singlefs-core/src/write_accounting.rs
index 5c0d50e4..1f19e1cd 100644
--- a/crates/singlefs-core/src/write_accounting.rs
+++ b/crates/singlefs-core/src/write_accounting.rs
@@ -18,12 +18,16 @@ pub enum WrittenStructureKind {
     InodeTreeRoot,
     AllocationRecordTreeNode,
     AccountingTreeNode,
+    /// deadlist 树的节点（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3：第三棵多层码 2 树）：按树数，不按层数。
+    DeadlistTreeNode,
     /// `deleted_inodes` 树的根（D8（核心索引结构） 已定项 16）。
     DeletedInodesTreeNode,
     /// 墓碑树的容器与根（D18（块里携带什么信息） 已定项 24）：按树数，不按容器与根分。
     TombstoneTreeNode,
     /// 谱系树的容器与根（D18（块里携带什么信息） 已定项 24 第 5 条）：另立一种、不并进墓碑树那一种（用户 2026-10-02 定「另立一种」），按树数。
     LineageTreeNode,
+    /// logged_ops 树的根（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1：单树、只有一个码 2 根）：另立一种，按树数。
+    LoggedOperationsTreeNode,
     /// 退役树的容器与根（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）：照谱系树另立一种，按树数。
     RetirementTreeNode,
     CentralMappingTreeNode,
@@ -37,16 +41,18 @@ pub enum WrittenStructureKind {
 impl WrittenStructureKind {
     /// 报数的次序：单元按 bump 次序（实例表单元在末尾，同 `TransactionOutput::units`），固定结构按持久顺序（D16（发布语义） 已定项 7）。
     /// 合计不经过这张清单（`WritesByStructureKind::total` 只从记过的账里加）：清单漏一种，报出的各种之和就对不上合计。
-    pub const IN_REPORT_ORDER: [WrittenStructureKind; 16] = [
+    pub const IN_REPORT_ORDER: [WrittenStructureKind; 18] = [
         WrittenStructureKind::DataUnit,
         WrittenStructureKind::ExtentTreeNode,
         WrittenStructureKind::InodeTreeLeafContainer,
         WrittenStructureKind::InodeTreeRoot,
         WrittenStructureKind::AllocationRecordTreeNode,
         WrittenStructureKind::AccountingTreeNode,
+        WrittenStructureKind::DeadlistTreeNode,
         WrittenStructureKind::DeletedInodesTreeNode,
         WrittenStructureKind::TombstoneTreeNode,
         WrittenStructureKind::LineageTreeNode,
+        WrittenStructureKind::LoggedOperationsTreeNode,
         WrittenStructureKind::RetirementTreeNode,
         WrittenStructureKind::CentralMappingTreeNode,
         WrittenStructureKind::TreeTableUnit,
@@ -72,6 +78,9 @@ impl WrittenStructureKind {
             // 多层之后根之下的节点与根同一种：按树数，不按层数（增补 1 的账按结构种类记）。
             TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree => WrittenStructureKind::AccountingTreeNode,
+            TransactionUnit::DeadlistTreeNodeBelowTheRoot(_) | TransactionUnit::DeadlistTree => {
+                WrittenStructureKind::DeadlistTreeNode
+            }
             TransactionUnit::DeletedInodesRoot => WrittenStructureKind::DeletedInodesTreeNode,
             TransactionUnit::TombstoneLeafContainer(_) | TransactionUnit::TombstoneRoot => {
                 WrittenStructureKind::TombstoneTreeNode
@@ -84,6 +93,7 @@ impl WrittenStructureKind {
                 WrittenStructureKind::TombstoneTreeNode
             }
             TransactionUnit::LineageNodeBelowTheRoot(_) => WrittenStructureKind::LineageTreeNode,
+            TransactionUnit::LoggedOperationsRoot => WrittenStructureKind::LoggedOperationsTreeNode,
             TransactionUnit::RetirementLeafContainer(_)
             | TransactionUnit::RetirementRoot
             | TransactionUnit::RetirementNodeBelowTheRoot(_) => {
@@ -109,9 +119,11 @@ impl WrittenStructureKind {
             WrittenStructureKind::InodeTreeRoot => "inode_tree_root",
             WrittenStructureKind::AllocationRecordTreeNode => "allocation_record_tree_node",
             WrittenStructureKind::AccountingTreeNode => "accounting_tree_node",
+            WrittenStructureKind::DeadlistTreeNode => "deadlist_tree_node",
             WrittenStructureKind::DeletedInodesTreeNode => "deleted_inodes_tree_node",
             WrittenStructureKind::TombstoneTreeNode => "tombstone_tree_node",
             WrittenStructureKind::LineageTreeNode => "lineage_tree_node",
+            WrittenStructureKind::LoggedOperationsTreeNode => "logged_operations_tree_node",
             WrittenStructureKind::RetirementTreeNode => "retirement_tree_node",
             WrittenStructureKind::CentralMappingTreeNode => "central_mapping_tree_node",
             WrittenStructureKind::TreeTableUnit => "tree_table_unit",
@@ -286,9 +298,11 @@ mod tests {
             TransactionUnit::InodeRoot,
             TransactionUnit::AllocationTree,
             TransactionUnit::AccountingTree,
+            TransactionUnit::DeadlistTree,
             TransactionUnit::DeletedInodesRoot,
             TransactionUnit::TombstoneRoot,
             TransactionUnit::LineageRoot,
+            TransactionUnit::LoggedOperationsRoot,
             TransactionUnit::RetirementRoot,
             TransactionUnit::MappingTree,
             TransactionUnit::TreeTable,
@@ -312,7 +326,7 @@ mod tests {
             assert_eq!(
                 every_kind_once.of(kind).write_calls,
                 1,
-                "{kind:?}：十三个单元角色各归一种、三种固定结构各一种，每种恰好一笔"
+                "{kind:?}：十五个单元角色各归一种、三种固定结构各一种，每种恰好一笔"
             );
         }
         let reported = WrittenStructureKind::IN_REPORT_ORDER
@@ -328,10 +342,10 @@ mod tests {
         assert_eq!(
             every_kind_once.total(),
             WriteCallsAndBytes {
-                write_calls: 16,
-                written_bytes: 65535
+                write_calls: 18,
+                written_bytes: 262_143
             },
-            "十六笔，长度 1 到 32768 各一次"
+            "十八笔，长度 1 到 131072 各一次"
         );
         let mut names: Vec<&str> = WrittenStructureKind::IN_REPORT_ORDER
             .iter()
@@ -339,6 +353,6 @@ mod tests {
             .collect();
         names.sort_unstable();
         names.dedup();
-        assert_eq!(names.len(), 16, "结果行的字段名前缀互不相同");
+        assert_eq!(names.len(), 18, "结果行的字段名前缀互不相同");
     }
 }
diff --git a/crates/singlefs-format/src/lib.rs b/crates/singlefs-format/src/lib.rs
index d8b5f331..dd21f5c0 100644
--- a/crates/singlefs-format/src/lib.rs
+++ b/crates/singlefs-format/src/lib.rs
@@ -185,6 +185,21 @@ pub const EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES: u64 = 113;
 /// extent 树上段一片叶罩几个 inode 号：(16384 − 163) ÷ 113 的整数部分（一个 inode 号至多一条上段叶条目，叶永远装得下）。
 pub const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 143;
 
+/// logged_ops 树条目的 key：种类 2 + 签发 txg 8，按逐字段无符号整数、字段自左向右比较（D8（核心索引结构） 已定项 11；
+/// 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1「单树、定宽联合条目」）。format-const: LOGGED_OPERATIONS_KEY_BYTES
+pub const LOGGED_OPERATIONS_KEY_BYTES: u64 = 10;
+
+/// logged_ops 树条目的 value，按种类解、定宽：种类 1（整理）= R 的设备 4 + 起始槽 6 + 槽数 2、`c0` 8、`segs0` 8、补零 6；
+/// 别的种类码保留（同一判决 G1）。format-const: LOGGED_OPERATIONS_VALUE_BYTES
+pub const LOGGED_OPERATIONS_VALUE_BYTES: u64 = 34;
+
+/// logged_ops 树条目：key 10 + value 34（同一判决 G1）。format-const: LOGGED_OPERATIONS_ENTRY_BYTES
+pub const LOGGED_OPERATIONS_ENTRY_BYTES: u64 = 44;
+
+/// 一个 logged_ops 码 2 节点装几条：(16384 − 头 `86 + 2 × 10 + 29`) ÷ 44 的整数部分（同一判决 G1「一节点 369 条」）。
+/// format-const: LOGGED_OPERATIONS_ENTRIES_PER_NODE
+pub const LOGGED_OPERATIONS_ENTRIES_PER_NODE: u64 = 369;
+
 /// 新池新建文件这次发布写出的记账行数（D5（快照 / 空间记账机制） 已定项 8，2026-09-14 用户定案 15 行）。
 pub const NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS: u64 = 15;
 
@@ -635,6 +650,35 @@ mod tests {
         );
     }
 
+    /// logged_ops 树条目的四个宽写成整数字面量，原来的算式留在这里，一个数不差（判决 `m4-gc-r1` 第三节 G1）：
+    /// key = 种类 2 + 签发 txg 8；整理那一种的 value = R 的设备 4 + 起始槽 6 + 槽数 2 + `c0` 8 + `segs0` 8 + 补零 6；
+    /// 一节点条数 = 节点里 key 宽 10 的码 2 头之后装得下几条 44。
+    #[test]
+    fn logged_operations_widths_equal_the_field_sums_and_the_node_capacity_they_stand_for() {
+        assert_eq!(
+            LOGGED_OPERATIONS_KEY_BYTES,
+            2 + 8,
+            "key = 种类 2 + 签发 txg 8"
+        );
+        assert_eq!(
+            LOGGED_OPERATIONS_VALUE_BYTES,
+            4 + 6 + 2 + 8 + 8 + 6,
+            "整理的 value = R 的设备 4 + 起始槽 6 + 槽数 2 + c0 8 + segs0 8 + 补零 6"
+        );
+        assert_eq!(
+            LOGGED_OPERATIONS_ENTRY_BYTES,
+            LOGGED_OPERATIONS_KEY_BYTES + LOGGED_OPERATIONS_VALUE_BYTES,
+            "条目 = key + value"
+        );
+        assert_eq!(
+            LOGGED_OPERATIONS_ENTRIES_PER_NODE,
+            (NODE_BYTES - index_node_header_bytes(LOGGED_OPERATIONS_KEY_BYTES))
+                / LOGGED_OPERATIONS_ENTRY_BYTES,
+            "一节点条数 = (16384 − key 宽 10 的码 2 头) ÷ 44 的整数部分"
+        );
+        assert_eq!(LOGGED_OPERATIONS_ENTRIES_PER_NODE, 369, "判决 G1 写的 369");
+    }
+
     /// 系统配置的 489 与它所在的槽：槽宽 4096 时余 3607（D22 已定项 9「槽内余 3607」），装得下下一条 86 字节指针；
     /// 按此前的 512 槽算只余 23、塞不进（D22 已定项 2 重开的理由之一）。
     #[test]
diff --git a/crates/singlefs-reference-filesystem/src/lib.rs b/crates/singlefs-reference-filesystem/src/lib.rs
index 905dfab6..263e3169 100644
--- a/crates/singlefs-reference-filesystem/src/lib.rs
+++ b/crates/singlefs-reference-filesystem/src/lib.rs
@@ -15,6 +15,8 @@
 //! 只管串行提交；任意子集的丢法不做（r5 判决第四节第 8 条：串行提交下出局）。自动触发的攒批发布落地之前只接显式发布点（r6 判决第二节 U2 一行）。
 #![forbid(unsafe_code)]
 
+use std::collections::BTreeMap;
+
 /// 一个文件的全部字节。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct FileContent(pub Vec<u8>);
@@ -26,6 +28,15 @@ pub enum VisibleState {
     OneFile(FileContent),
 }
 
+/// 一个只读快照的名字：调用方按实现点名它的那个号交入（实现按码 14 条目的树号点名，判决
+/// `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 5）。这个包不解释它，只拿它当键。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct SnapshotName(pub u64);
+
+/// 活着的只读快照：每个快照名对应建它那一刻 POSIX 面上看得见的状态（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
+/// 第三节 S7「RefFS 加 CreateSnapshot、DestroySnapshot、读快照，状态加快照集合」）。
+pub type Snapshots = BTreeMap<SnapshotName, VisibleState>;
+
 /// 第一版的操作。崩溃不在这里，它是 [`ReferenceFileSystem::crash_and_recover`]。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum Operation {
@@ -38,6 +49,10 @@ pub enum Operation {
     /// 回退到第 m 版的内容；m 由调用方按实现报的回退根在「根 → 版本」表里查出来，选得对不对这里不判。
     /// 回退是挂着时的一次向前发布：返回成功时抬起下界 lo，它的下标是发布末尾，它之后的内容是一个新版本。
     RollBackToVersion(VersionNumber),
+    /// 建一个只读快照，名字是 `SnapshotName`：捕获这一刻的状态。它是一次独立发布（判决 S2），返回成功时抬起下界 lo。
+    CreateSnapshot(SnapshotName),
+    /// 删一个只读快照（一次调用 = 意图 + 全部批，判决第七节岔路 2、4）：返回成功时它不在了，抬起下界 lo。POSIX 面上的文件不变。
+    DestroySnapshot(SnapshotName),
 }
 
 /// 调用方交入的结局。
@@ -93,6 +108,14 @@ pub enum OperationNotAccepted {
         requested: VersionNumber,
         versions_so_far: usize,
     },
+    /// 还没有文件时建快照：实现在没有活文件的一版上拒建快照（会话拒成 `NoFileVersionToChange`），条款没写它在 POSIX 面上是什么。
+    CreatingASnapshotWithoutAFileIsNotInTheFirstVersion,
+    /// 建快照用了一个已经活着的快照名。
+    SnapshotNameAlreadyTaken { name: SnapshotName },
+    /// 删一个不活着的快照。
+    NoSnapshotWithThisName { name: SnapshotName },
+    /// 有活快照时回退：第一版有快照就拒回退（用户 2026-10-13 定，判决第三节 S8 与第七节岔路 3）。
+    RollBackWithSnapshotsIsNotInTheFirstVersion,
 }
 
 /// 允许集合里的一个状态与它的下标。
@@ -151,6 +174,8 @@ enum EnteredKind {
     ExplicitPublish,
     Fsync,
     RollBack,
+    CreateSnapshot,
+    DestroySnapshot,
 }
 
 impl EnteredKind {
@@ -162,6 +187,8 @@ impl EnteredKind {
             EnteredKind::ExplicitPublish | EnteredKind::Fsync => true,
             // 回退是挂着时的一次向前发布：D23（journal 的角色与格式） .claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
             EnteredKind::RollBack => true,
+            // 建快照是一次独立发布（判决 S2）；删快照一次调用发意图与全部批，返回成功时最后一批已落盘（判决第七节岔路 2、4）。
+            EnteredKind::CreateSnapshot | EnteredKind::DestroySnapshot => true,
             EnteredKind::ContentChange => false,
         }
     }
@@ -172,6 +199,8 @@ struct EnteredOperation {
     kind: EnteredKind,
     outcome: EnteredOutcome,
     state_after: VisibleState,
+    /// 这个操作施加之后活着的快照（不建、不删快照的操作照抄前一个的）。
+    snapshots_after: Snapshots,
 }
 
 /// 序列里已确认的发布（返回成功的发布调用、fsync 与回退，原样重发成功的也算）各自发布之后的内容，按历史次序。
@@ -261,6 +290,8 @@ pub struct ReferenceFileSystem {
     operations_since_the_last_recovery: Vec<EnteredOperation>,
     /// 上一次恢复为止的版本，按历史次序：起点、之前各轮里已确认的发布之后的内容、各次恢复读出的那一份；第 m 版在位置 m − 1。
     versions_up_to_the_last_recovery: Vec<VisibleState>,
+    /// S_0 那一刻活着的快照：上一次恢复以它重新起头的那一份（还没崩溃过时没有快照）。
+    snapshots_at_the_last_recovery: Snapshots,
 }
 
 impl ReferenceFileSystem {
@@ -270,9 +301,24 @@ impl ReferenceFileSystem {
             operations_since_the_last_recovery: Vec::new(),
             // 起点是第 1 版：mkfs 写的根与 mkfs 之后可写挂载写的根映到它（r6 用户定案 H1-收严「m 的编号覆盖每一个写根的入口」）
             versions_up_to_the_last_recovery: vec![VisibleState::NoFile],
+            snapshots_at_the_last_recovery: Snapshots::new(),
         }
     }
 
+    /// 序列里的操作全都做成时活着的快照。
+    pub fn snapshots(&self) -> &Snapshots {
+        self.operations_since_the_last_recovery
+            .last()
+            .map_or(&self.snapshots_at_the_last_recovery, |operation| {
+                &operation.snapshots_after
+            })
+    }
+
+    /// 读快照 `name`：它活着时交回建它那一刻的状态，不活着交回 `None`（序列里的操作全都做成时）。
+    pub fn read_snapshot(&self, name: SnapshotName) -> Option<&VisibleState> {
+        self.snapshots().get(&name)
+    }
+
     /// 到这一刻为止的版本，按历史次序（第 m 版在位置 m − 1）。
     fn versions_so_far(&self) -> Vec<&VisibleState> {
         self.versions_up_to_the_last_recovery
@@ -346,19 +392,76 @@ impl ReferenceFileSystem {
             OperationOutcome::FailedAfterWriting => EnteredOutcome::FailedAfterWriting,
         };
         let state_so_far = self.state_after_every_operation_so_far();
-        let (kind, state_after) = match operation {
+        let snapshots_so_far = self.snapshots();
+        let (kind, state_after, snapshots_after) = match operation {
             Operation::CreateTheFirstFile(content) => match state_so_far {
-                VisibleState::NoFile => (EnteredKind::ContentChange, VisibleState::OneFile(content)),
+                VisibleState::NoFile => (
+                    EnteredKind::ContentChange,
+                    VisibleState::OneFile(content),
+                    snapshots_so_far.clone(),
+                ),
                 VisibleState::OneFile(_) => return Err(OperationNotAccepted::CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion),
             },
             Operation::OverwriteTheFile(content) => match state_so_far {
                 VisibleState::NoFile => return Err(OperationNotAccepted::OverwritingWithoutAFileIsNotInTheFirstVersion),
-                VisibleState::OneFile(_) => (EnteredKind::ContentChange, VisibleState::OneFile(content)),
+                VisibleState::OneFile(_) => (
+                    EnteredKind::ContentChange,
+                    VisibleState::OneFile(content),
+                    snapshots_so_far.clone(),
+                ),
             },
-            Operation::ExplicitPublish => (EnteredKind::ExplicitPublish, state_so_far.clone()),
-            Operation::Fsync => (EnteredKind::Fsync, state_so_far.clone()),
+            Operation::ExplicitPublish => (
+                EnteredKind::ExplicitPublish,
+                state_so_far.clone(),
+                snapshots_so_far.clone(),
+            ),
+            Operation::Fsync => (
+                EnteredKind::Fsync,
+                state_so_far.clone(),
+                snapshots_so_far.clone(),
+            ),
             Operation::RollBackToVersion(number) => {
-                (EnteredKind::RollBack, self.content_of_version(number)?)
+                // 第一版有快照就拒回退（判决 S8）：实现在任何写之前拒，调用方交的是「写之前拒」，走不到这里；走到了就是输入不合法。
+                if !snapshots_so_far.is_empty() {
+                    return Err(OperationNotAccepted::RollBackWithSnapshotsIsNotInTheFirstVersion);
+                }
+                (
+                    EnteredKind::RollBack,
+                    self.content_of_version(number)?,
+                    snapshots_so_far.clone(),
+                )
+            }
+            Operation::CreateSnapshot(name) => {
+                if snapshots_so_far.contains_key(&name) {
+                    return Err(OperationNotAccepted::SnapshotNameAlreadyTaken { name });
+                }
+                match state_so_far {
+                    VisibleState::NoFile => {
+                        return Err(
+                            OperationNotAccepted::CreatingASnapshotWithoutAFileIsNotInTheFirstVersion,
+                        )
+                    }
+                    VisibleState::OneFile(_) => {}
+                }
+                let mut snapshots_after = snapshots_so_far.clone();
+                snapshots_after.insert(name, state_so_far.clone());
+                (
+                    EnteredKind::CreateSnapshot,
+                    state_so_far.clone(),
+                    snapshots_after,
+                )
+            }
+            Operation::DestroySnapshot(name) => {
+                if !snapshots_so_far.contains_key(&name) {
+                    return Err(OperationNotAccepted::NoSnapshotWithThisName { name });
+                }
+                let mut snapshots_after = snapshots_so_far.clone();
+                snapshots_after.remove(&name);
+                (
+                    EnteredKind::DestroySnapshot,
+                    state_so_far.clone(),
+                    snapshots_after,
+                )
             }
         };
         self.operations_since_the_last_recovery
@@ -366,6 +469,7 @@ impl ReferenceFileSystem {
                 kind,
                 outcome: entered_outcome,
                 state_after,
+                snapshots_after,
             });
         Ok(SubmittedOperation::Entered(OperationNumber(
             self.operations_since_the_last_recovery.len(),
@@ -448,6 +552,19 @@ impl ReferenceFileSystem {
         Ok(())
     }
 
+    /// S_j 那一刻活着的快照（S_0 是上一次恢复以它起头的那一份）。
+    fn snapshots_at(&self, index: StateIndex) -> Snapshots {
+        match index.0.checked_sub(1) {
+            None => self.snapshots_at_the_last_recovery.clone(),
+            Some(position) => self
+                .operations_since_the_last_recovery
+                .get(position)
+                .map_or_else(Snapshots::new, |operation| {
+                    operation.snapshots_after.clone()
+                }),
+        }
+    }
+
     fn restart_from_the_recovered_state(
         &mut self,
         crash_falls_in: OperationNumber,
@@ -463,8 +580,24 @@ impl ReferenceFileSystem {
         // 恢复读出、以它重新起头的那一份也是一个版本：之后可写挂载写的根映到它（r6 用户定案 H1-收严）
         self.versions_up_to_the_last_recovery
             .push(recovered.clone());
+        // 恢复之后活着的快照：允许集合里与读出的内容相同、下标最大的那个状态那一刻的快照（这一版的崩溃判定只比文件，
+        // 快照集合不比——建快照与删快照都是发布末尾，读出的文件定不了它落在哪一个末尾时取最新的那个；第一版的随机历史与崩溃流
+        // 还不交快照操作，各状态的快照集合相同）。
+        let snapshots_after_the_recovery = self
+            .allowed_set_for_a_crash_in(crash_falls_in)
+            .ok()
+            .and_then(|allowed| {
+                allowed
+                    .allowed
+                    .iter()
+                    .rev()
+                    .find(|allowed_state| allowed_state.state == recovered)
+                    .map(|allowed_state| self.snapshots_at(allowed_state.index))
+            })
+            .unwrap_or_else(|| self.snapshots_at(StateIndex(0)));
         // 每次恢复之后以读出的内容为起点接着走（r5 判决第二节 U2 ②）
         self.state_at_the_last_recovery = recovered;
+        self.snapshots_at_the_last_recovery = snapshots_after_the_recovery;
         self.operations_since_the_last_recovery.clear();
     }
 }
@@ -508,6 +641,103 @@ mod tests {
             .collect()
     }
 
+    /// 建快照捕获那一刻的文件，之后覆盖写不动它；读快照读出建它那一刻的内容；删掉之后读不到，活头的文件不变
+    /// （判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S7）。
+    #[test]
+    fn a_snapshot_reads_the_file_at_its_creation_until_it_is_destroyed() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(65_549)));
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        assert_eq!(
+            model.read_snapshot(SnapshotName(65_549)),
+            Some(&file_with("A")),
+            "快照读出建它那一刻的 A"
+        );
+        succeed(&mut model, Operation::DestroySnapshot(SnapshotName(65_549)));
+        assert_eq!(
+            model.read_snapshot(SnapshotName(65_549)),
+            None,
+            "删掉之后读不到"
+        );
+        assert!(model.snapshots().is_empty(), "快照集合空了");
+        assert_eq!(
+            model.state_after_every_operation_so_far(),
+            &file_with("B"),
+            "删快照不动活头的文件"
+        );
+    }
+
+    /// 建快照、删快照各是发布末尾（返回成功时抬起下界）：崩在建快照之后，允许集合的下界就是它；恢复读出那一刻的文件，快照接着活着。
+    #[test]
+    fn a_crash_after_a_snapshot_returned_keeps_the_snapshot_and_starts_from_it() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        let creation = succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        let crash = model.crash_after_every_operation_returned();
+        let allowed = model
+            .crash_and_recover(crash, file_with("A"))
+            .expect("A 在允许集合里");
+        assert_eq!(
+            allowed.lower_bound,
+            creation.state_after(),
+            "建快照返回成功：下界是它的末尾"
+        );
+        assert_eq!(
+            model.read_snapshot(SnapshotName(7)),
+            Some(&file_with("A")),
+            "恢复之后快照接着活着"
+        );
+    }
+
+    /// 不收的四种：没有文件时建快照、快照名已经活着、删一个不活着的快照、有活快照时回退；返回时模型不动。
+    #[test]
+    fn snapshot_operations_the_first_version_does_not_take_are_not_accepted_and_leave_the_model_unchanged(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        assert_eq!(
+            model.submit(
+                Operation::CreateSnapshot(SnapshotName(1)),
+                OperationOutcome::Succeeded
+            ),
+            Err(OperationNotAccepted::CreatingASnapshotWithoutAFileIsNotInTheFirstVersion),
+            "没有文件时建快照"
+        );
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(1)));
+        let before = model.clone();
+        assert_eq!(
+            model.submit(
+                Operation::CreateSnapshot(SnapshotName(1)),
+                OperationOutcome::Succeeded
+            ),
+            Err(OperationNotAccepted::SnapshotNameAlreadyTaken {
+                name: SnapshotName(1)
+            }),
+            "快照名已经活着"
+        );
+        assert_eq!(
+            model.submit(
+                Operation::DestroySnapshot(SnapshotName(2)),
+                OperationOutcome::Succeeded
+            ),
+            Err(OperationNotAccepted::NoSnapshotWithThisName {
+                name: SnapshotName(2)
+            }),
+            "删一个不活着的快照"
+        );
+        assert_eq!(
+            model.submit(
+                Operation::RollBackToVersion(VersionNumber(1)),
+                OperationOutcome::Succeeded
+            ),
+            Err(OperationNotAccepted::RollBackWithSnapshotsIsNotInTheFirstVersion),
+            "有活快照时回退"
+        );
+        assert_eq!(model, before, "不收的操作不动模型");
+    }
+
     #[test]
     fn an_overwrite_refused_before_any_write_did_not_happen_so_the_publish_after_it_does_not_allow_its_content(
     ) {
diff --git a/crates/singlefs-replay-prefix/src/lib.rs b/crates/singlefs-replay-prefix/src/lib.rs
index f3b9badb..95b72f7e 100644
--- a/crates/singlefs-replay-prefix/src/lib.rs
+++ b/crates/singlefs-replay-prefix/src/lib.rs
@@ -20,6 +20,8 @@
 //! 另两个纯函数核也在这里：择根（`choose_root_slot`，`recovery::choose_root` 取 `(checkpoint_txg, 实例代号)` 最大的那一截）与
 //! 扫环建表（`first_copy_of_each_record_key`，`recovery::scan_journal` 同一 (实例代号, 计数器) 先到先得的那一截）；
 //! 择根与取前缀的合判（C650 的 Verus 一半）写成推论 `corollary_recovery_replays_from_the_newest_self_verified_root`。
+//! 整理的选段核与计划核及其四条性质（里程碑四二·4）在模块 [`compaction_plan_proofs`]：`singlefs-core` 不调它们，它的
+//! `compaction_plan` 单测与它们逐项对拍。
 //! Verus 的必红世界登记在同目录的 `verus-must-red-worlds.tsv`。
 #![forbid(unsafe_code)]
 
@@ -27,6 +29,8 @@ use std::collections::BTreeMap;
 
 use vstd::prelude::*;
 
+pub mod compaction_plan_proofs;
+
 verus! {
 
 // ===== 核的数据 =====
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
new file mode 100644
index 00000000..8cf74dc9
--- /dev/null
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -0,0 +1,1228 @@
+//! 整理的选段与计划（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G2）：三层里的后两层，都是纯函数——
+//! 不读设备、不发写、不动分配器。`choose_region` 按记录给候选段排序、按扫描分类定资格（没扫过的先让调用方经
+//! `crate::region_scan::scan_region` 扫），`plan_batch` 从扫描结果里取下一批要搬的（映射 key, 源落点）。**目的落点不在计划里**：由那一批发布时的分配器按 D3（空间分配） 已定项 8 现取
+//! （判决 G2，攻方 H-G2-1：缓存目的落点的臂撞上映射重复）。
+//!
+//! 判据显式（`.claude/rules/fs-design.md` 五条硬要求第 1 条）：第一版只有 greedy 一种（[`RegionChoiceCriterion::Greedy`]），
+//! 打分是 [`greedy_score`]；cost-benefit 与轮转留给 E10（老化：结构抗老化 vs 事后整理） 的臂。
+
+use std::collections::{BTreeMap, BTreeSet};
+use std::num::NonZeroUsize;
+
+use singlefs_format::CLUSTER_SEGMENT_SLOTS;
+
+use crate::address::{DeviceIdentity, SlotNumber};
+use crate::allocator::{AllocationRecord, Placement, UnitAreaStart};
+use crate::logged_operations::{CompactionIntent, CompactionRegion};
+use crate::region_scan::{RegionScan, RegionScanRefusal};
+
+/// 选段判据。封闭集合：加一种判据就加一个成员（判决 G2 的 S-b、S-c 由 E10 定要不要加）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RegionChoiceCriterion {
+    /// 活槽最少的段先（判决 G2 第一版）。
+    Greedy,
+}
+
+/// 一块盘的单元区：从哪个槽起、有几个槽。选段只在它的整段里挑（末尾凑不满 64 槽的那一截不是聚簇段，同 `lowest_empty_segment`）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct UnitAreaOfOneDevice {
+    pub device: DeviceIdentity,
+    pub start: UnitAreaStart,
+    pub slots: u64,
+}
+
+/// greedy 的打分：段里的活槽数（未释放记录罩住的槽数），分越低越先选。
+#[must_use]
+pub const fn greedy_score(live_slots_in_segment: u64) -> u64 {
+    live_slots_in_segment
+}
+
+/// 这一次挂载里区域扫描整次拒过的聚簇段（段起点 → 拒的理由）：[`choose_region`] 不再选它们。只增不减，随挂载会话一起丢。
+/// 拒整次扫描的是盘上的坏记录（跨段的记录只有盘上读来的才有，归 C411（分配记录的跨度段与单元类别不符没检查） 的域），
+/// 整理不替它修（主 agent 2026-10-13 定，判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S4）。
+/// 第一版两盘同槽：按段起点的槽号记（判决 G1 攻方 K-n）。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct SegmentsRefusedByTheRegionScan {
+    refused: BTreeMap<SlotNumber, RegionScanRefusal>,
+}
+
+impl SegmentsRefusedByTheRegionScan {
+    /// 记下 `crate::region_scan::scan_region` 对 `region` 整次拒了（同一段再拒一次留第一次的理由）。
+    pub fn note_refusal(&mut self, region: CompactionRegion, refusal: RegionScanRefusal) {
+        self.refused.entry(region.start_slot()).or_insert(refusal);
+    }
+
+    /// 起点是 `segment_start` 的段这次挂载里被扫描拒过没有。
+    #[must_use]
+    pub fn contains(&self, segment_start: SlotNumber) -> bool {
+        self.refused.contains_key(&segment_start)
+    }
+}
+
+/// [`choose_region`] 这一次的答案。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RegionChoice {
+    /// 选中这一段：它的扫描结果里至少一个能搬的单元。
+    Chosen(CompactionRegion),
+    /// 按判据排在最前的候选段还没扫过（`region_scans` 里没有它）：调用方先对它跑 `crate::region_scan::scan_region`——
+    /// 扫成了把结果并进 `region_scans`，被拒了记进 [`SegmentsRefusedByTheRegionScan`]——再选一次。
+    /// 每答一次这个，下一次的输入就多一段有结论，所以连着答的次数不超过单元区的整段数。
+    ScanBeforeChoosing(CompactionRegion),
+    /// 一段都没资格。
+    NoEligibleSegment,
+}
+
+/// 挑一个整理区域 R（判决 G2 `choose_region`）。资格五条全成立的段才算：不在这次挂载开过的聚簇段里（`cluster_segments`，段起点）、
+/// 不是活意图的 R、罩住的槽一个都不在 `pinned_slots` 里、这次挂载里没被区域扫描整次拒过（`segments_refused_by_the_region_scan`）、
+/// R 里至少一个能搬的单元（按 `region_scans` 里这一段的扫描分类取，判决 G2「R 里至少一个能搬的单元」，攻方 K-b；
+/// 判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S4）。
+/// 按判据打分（活槽数：R 那块盘上未释放记录罩住的槽）排候选，同分取起点槽号最小的；排在最前的那一段扫过就交回
+/// [`RegionChoice::Chosen`]，没扫过就交回 [`RegionChoice::ScanBeforeChoosing`]；扫过而一个能搬的都没有的段不是候选。
+/// `region_scans` 要是对这一份 `allocation_records` 扫出来的（记录变了就重扫），不是 `unit_area.device` 那块盘上的不算。
+/// 第一版两盘同槽：活槽数按 `unit_area.device` 那块盘的记录数，`cluster_segments`、活 R、`pinned_slots` 与被拒的段都只比槽号（判决 G1 攻方 K-n）。
+#[must_use]
+#[allow(
+    clippy::too_many_arguments,
+    reason = "八个输入各是判决 G2 / S4 点名的一样，没有哪几个合起来是一个带不变量的概念（code-discipline.md「参数不按个数限」）"
+)]
+pub fn choose_region(
+    allocation_records: &[AllocationRecord],
+    unit_area: UnitAreaOfOneDevice,
+    cluster_segments: &BTreeSet<SlotNumber>,
+    live_intent_region: Option<CompactionRegion>,
+    pinned_slots: &BTreeSet<SlotNumber>,
+    region_scans: &[RegionScan],
+    segments_refused_by_the_region_scan: &SegmentsRefusedByTheRegionScan,
+    criterion: RegionChoiceCriterion,
+) -> RegionChoice {
+    // 按段起点索引一次（同一段有几份时取最前那一份）：下面每个候选段查一次，不逐份线性找。
+    let mut scans_by_start_slot: BTreeMap<SlotNumber, &RegionScan> = BTreeMap::new();
+    for scan in region_scans
+        .iter()
+        .filter(|scan| scan.region.device() == unit_area.device)
+    {
+        scans_by_start_slot
+            .entry(scan.region.start_slot())
+            .or_insert(scan);
+    }
+    let scan_of_the_segment =
+        |start_slot: SlotNumber| scans_by_start_slot.get(&start_slot).copied();
+    let unit_area_start = unit_area.start.slot().0;
+    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
+    let segment_slots = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
+    let mut live_slots_per_segment =
+        vec![0u64; usize::try_from(full_segments).expect("一块盘的段数装得进 usize")];
+    for record in allocation_records
+        .iter()
+        .filter(|record| record.device == unit_area.device && !record.is_released)
+    {
+        // 迭代上界是这条记录的跨度；只数落在整段里的槽。
+        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
+            let Some(offset_in_unit_area) = slot.checked_sub(unit_area_start) else {
+                continue;
+            };
+            let segment = usize::try_from(offset_in_unit_area / CLUSTER_SEGMENT_SLOTS)
+                .expect("段号装得进 usize");
+            if let Some(live_slots) = live_slots_per_segment.get_mut(segment) {
+                *live_slots += 1;
+            }
+        }
+    }
+    let eligible_segments = live_slots_per_segment
+        .iter()
+        .enumerate()
+        .map(|(segment, live_slots)| {
+            let start_slot = SlotNumber(
+                unit_area_start + u64::try_from(segment * segment_slots).expect("槽号装得进 u64"),
+            );
+            (start_slot, *live_slots)
+        })
+        .filter(|(start_slot, _live_slots)| {
+            let end_slot = SlotNumber(start_slot.0 + CLUSTER_SEGMENT_SLOTS);
+            let opened_in_this_mount = cluster_segments.contains(start_slot);
+            let is_the_live_intent_region =
+                live_intent_region.is_some_and(|region| region.start_slot() == *start_slot);
+            let holds_a_pinned_slot = pinned_slots.range(*start_slot..end_slot).next().is_some();
+            let refused_by_the_region_scan =
+                segments_refused_by_the_region_scan.contains(*start_slot);
+            // 没扫过的段还不知道有没有能搬的单元：留作候选，轮到它时让调用方先扫。
+            let may_hold_a_movable_unit = match scan_of_the_segment(*start_slot) {
+                Some(scan) => scan.movable_units().next().is_some(),
+                None => true,
+            };
+            !opened_in_this_mount
+                && !is_the_live_intent_region
+                && !holds_a_pinned_slot
+                && !refused_by_the_region_scan
+                && may_hold_a_movable_unit
+        });
+    let first_start_slot = match criterion {
+        RegionChoiceCriterion::Greedy => eligible_segments
+            .min_by_key(|(start_slot, live_slots)| (greedy_score(*live_slots), *start_slot))
+            .map(|(start_slot, _live_slots)| start_slot),
+    };
+    let Some(first_start_slot) = first_start_slot else {
+        return RegionChoice::NoEligibleSegment;
+    };
+    let first_region = CompactionRegion::of_one_cluster_segment(unit_area.device, first_start_slot)
+        .expect(
+            "单元区起点在聚簇段边界上（UnitAreaStart 的不变量），整段的起点都是 64 的倍数；槽号来自一块盘的单元区，装得进 6 字节",
+        );
+    match scan_of_the_segment(first_start_slot) {
+        Some(_scan_with_a_movable_unit) => RegionChoice::Chosen(first_region),
+        None => RegionChoice::ScanBeforeChoosing(first_region),
+    }
+}
+
+/// 一批至多搬几个单元（判决 G2 的 K）。至少 1；取值归 E10（老化：结构抗老化 vs 事后整理），这里只是参数。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CompactionBatchUnits(NonZeroUsize);
+
+impl CompactionBatchUnits {
+    /// 0 交回 `None`：一批一个都不搬，意图永远推不完。
+    #[must_use]
+    pub fn new(units: usize) -> Option<Self> {
+        NonZeroUsize::new(units).map(Self)
+    }
+
+    #[must_use]
+    pub const fn units(self) -> usize {
+        self.0.get()
+    }
+}
+
+/// 计划里的一项：搬哪个单元（映射 key）、它现在在哪（源落点，两盘同槽）。目的落点不在这里（模块文档）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct PlannedRelocation {
+    pub mapping_key: Vec<u8>,
+    pub source: Placement,
+}
+
+/// [`plan_batch`] 的答案：有要搬的，或者一个能搬的都没有时分「搬空」与「搬不动」（判决 m4-batch2-code-r1 S4）。封闭集合。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum PlannedBatch {
+    /// 这一批要搬的，按槽升序，至少一项、至多 K 项。
+    Relocate(Vec<PlannedRelocation>),
+    /// R 里一条未释放的记录都没有了：R 按记录全空（判决 G4 的停机谓词「删意图那一刻 R 按记录全空」看的就是这个）。
+    RegionEmptied,
+    /// R 里一个能搬的都没有，却还有未释放的记录（钉住、隔离、已死、不一致的）：再推下去也空不了。
+    NothingMovableButTheRegionIsNotEmpty,
+}
+
+/// 下一批要搬的：扫描结果里可搬的单元按槽升序取前 `batch` 个（判决 G2 `plan_batch`）。钉住、隔离、已死、不一致的一个都不进。
+/// 这一批发布之后它们离开 R，下一次扫描就不再有它们：各批互不相交，全部批的并集是 R 里可搬的 key 集合。
+/// 一个能搬的都没有时不交空表：R 里没有未释放的记录交 [`PlannedBatch::RegionEmptied`]，还有交 [`PlannedBatch::NothingMovableButTheRegionIsNotEmpty`]。
+///
+/// # Panics
+/// 扫描结果里能搬的映射 key 有相同的：计划的输入只来自 [`crate::region_scan::scan_region`] 交回的 `Ok`，它见到同一把 key 判成可搬两次
+/// 就拒这个 R（`RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey`），所以这里收到的 key 两两不同，一批里的 key 也就互异
+/// （判决 G7 的 P2；Verus 证的核把它写成蕴含的前提 `movable_mapping_keys_are_distinct`）。
+#[must_use]
+pub fn plan_batch(region_scan: &RegionScan, batch: CompactionBatchUnits) -> PlannedBatch {
+    assert!(
+        region_scan.movable_mapping_keys_are_pairwise_distinct(),
+        "扫描结果里能搬的映射 key 两两不同：scan_region 见到同一把 key 判成可搬两次就拒这个 R，计划只收它交回的 Ok"
+    );
+    let relocations: Vec<PlannedRelocation> = region_scan
+        .movable_units()
+        .take(batch.units())
+        .map(|(source, unit)| PlannedRelocation {
+            mapping_key: unit.mapping_key.clone(),
+            source,
+        })
+        .collect();
+    if !relocations.is_empty() {
+        return PlannedBatch::Relocate(relocations);
+    }
+    if region_scan.allocations.is_empty() {
+        PlannedBatch::RegionEmptied
+    } else {
+        PlannedBatch::NothingMovableButTheRegionIsNotEmpty
+    }
+}
+
+/// 一批至多搬几个单元从哪来。封闭集合。产品路径的批大小条款没写（判决 G2 的 K 归 E10（老化：结构抗老化 vs 事后整理））：
+/// 产品路径照 [`CompactionBatchSize::NotDecidedByTheClauses`]，整理一次运行在签任何意图、发任何一个写之前停下，停机原因
+/// [`CompactionRunStop::BatchSizeNotDecidedByTheClauses`]；只供测试的开关给出一个数才走得进去。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionBatchSize {
+    NotDecidedByTheClauses,
+    /// 只供测试（判决 G6 `CompactionSwitches` 的「批大小」）。
+    ForcedForTests(CompactionBatchUnits),
+}
+
+/// 整理什么时候起跑。封闭集合。产品路径照水位（[`CompactionRunForcing::FollowTheWatermarks`]）；另两种只供测试（判决 G6「强制起跑 / 停」）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionRunForcing {
+    /// 用户改动发成之后按水位判起不起跑（判决 G8 丁）。第一版三条水位恒 0、0 = 关（判决 G4 V-a；内置默认改非零不在这一批），
+    /// 所以这一支在产品路径上不起跑；显式入口（`mounted_session::MountedSession::run_compaction`）不看水位、照常跑。
+    FollowTheWatermarks,
+    /// 只供测试：用户改动发成之后不看水位，照样跑至多一批。
+    ForcedToStartForTests,
+    /// 只供测试：两个入口都不起跑，停机原因 [`CompactionRunStop::ForcedToStopForTests`]。
+    ForcedToStopForTests,
+}
+
+/// 签发之后 R 排不排他（D26（后台整理与放置回收） 已定项 1 第 3 条：`R` 在意图期内不可作分配目的地）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionRegionExclusion {
+    ExcludedWhileTheIntentIsLive,
+    /// 只供测试（判决 G6「关 R 排他」）：活意图的 R 照样当分配目的地。
+    SwitchedOffForTests,
+}
+
+/// 搬迁的目的落点按哪条政策取。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RelocationDestinationPolicy {
+    /// 判决 G2 的 D-a：码 1 走用户数据落点函数（D3（空间分配） 已定项 8 第 1 条），节点与打包容器走开放段 bump（已定项 10 ⑤），都不落在活 R 里。
+    UserDataPlacementForDataAndOpenSegmentForTheRest,
+    /// 只供测试（判决 G6「强制目的地政策」）：每个单元都取单元区里起点最小、整跨度没挡的落点（提交内生块回落那一个函数），仍不落在活 R 里。
+    LowestUnblockedSlotForEveryUnitForTests,
+}
+
+/// 签发之后强制一次「释放前核出对不上」（判决 G6）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ForcedReleaseChecksumMismatchAfterIssue {
+    NotForced,
+    /// 只供测试：签发之后第一批的第一个源落点当作读盘核两份都核出对不上（每块盘一份都隔离，D19（块指针的结构与宽度预算） 已定项 5），
+    /// 整个挂载只强制这一次。
+    OnceOnTheFirstSourceOfTheFirstBatchForTests,
+}
+
+/// 合取 7 `全空聚簇段数 ≥ N₀` 的 `N₀`（D3（空间分配） 已定项 7 登记表第 7 行）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum EmptyClusterSegmentsFloor {
+    /// 第一版的值 [`EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION`]（算式等计数模型验过再定，E10（老化：结构抗老化 vs 事后整理））。
+    OfTheFirstVersion,
+    /// 只供测试（判决 G6「强制合取 7 的 `N₀`」）。
+    ForcedForTests(u64),
+}
+
+/// D3（空间分配） 已定项 7 登记表第 7 行：「`N₀` 第一版取 1」。
+pub const EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION: u64 = 1;
+
+impl EmptyClusterSegmentsFloor {
+    /// 这一档的 `N₀`。
+    #[must_use]
+    pub const fn segments(self) -> u64 {
+        match self {
+            EmptyClusterSegmentsFloor::OfTheFirstVersion => {
+                EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION
+            }
+            EmptyClusterSegmentsFloor::ForcedForTests(segments) => segments,
+        }
+    }
+}
+
+/// 一批搬迁的源落点什么时候释放。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RelocationPublishOrder {
+    /// 判决 G3：源落点的释放与映射条目换位置在同一次发布里。
+    SourceReleasedInThePublishThatMovesTheMapping,
+    /// 只供测试（`.claude/kb/milestone/04-fourth-txn.md` 二·3 的必红世界「源落点在映射发布之前就被释放」）：这一批照样释放源落点、
+    /// 写目的地与它的分配记录，映射条目与各处指针却留在源落点上。产品路径一处都不走它。
+    SourceReleasedWhileTheMappingStillNamesItForTests,
+}
+
+/// 整理只供测试的开关（判决 G6 `CompactionSwitches`；`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）。
+/// 产品路径是 [`CompactionSwitches::PRODUCT`]；只住内存，装在分配器上（`allocator::PoolAllocator::set_compaction_switches`），
+/// 平常构造（`PoolAllocator::new`、挂载重建的分配器）一律是产品值；可写挂载续做意图要开关时由调用方交给挂载
+/// （`mount::mount_writable_with_compaction_switches`）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CompactionSwitches {
+    pub batch_size: CompactionBatchSize,
+    /// 只供测试（判决 G6「强制选段」）：签发时不按判据选、取这一段（资格那五条照判，不合格就停）。`None` 是产品路径。
+    pub forced_region: Option<CompactionRegion>,
+    pub run_forcing: CompactionRunForcing,
+    pub region_exclusion: CompactionRegionExclusion,
+    pub destination_policy: RelocationDestinationPolicy,
+    pub release_checksum_mismatch_after_issue: ForcedReleaseChecksumMismatchAfterIssue,
+    pub empty_cluster_segments_floor: EmptyClusterSegmentsFloor,
+    pub relocation_publish_order: RelocationPublishOrder,
+}
+
+impl CompactionSwitches {
+    pub const PRODUCT: CompactionSwitches = CompactionSwitches {
+        batch_size: CompactionBatchSize::NotDecidedByTheClauses,
+        forced_region: None,
+        run_forcing: CompactionRunForcing::FollowTheWatermarks,
+        region_exclusion: CompactionRegionExclusion::ExcludedWhileTheIntentIsLive,
+        destination_policy:
+            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
+        release_checksum_mismatch_after_issue: ForcedReleaseChecksumMismatchAfterIssue::NotForced,
+        empty_cluster_segments_floor: EmptyClusterSegmentsFloor::OfTheFirstVersion,
+        relocation_publish_order:
+            RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping,
+    };
+}
+
+/// 一次整理运行停在哪（判决 G8 观测「停机原因」）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionRunStop {
+    /// 产品路径的批大小条款没写（[`CompactionBatchSize::NotDecidedByTheClauses`]）：一个意图都没签、一个写都没发。
+    BatchSizeNotDecidedByTheClauses,
+    /// 只供测试的开关叫停（[`CompactionRunForcing::ForcedToStopForTests`]）。
+    ForcedToStopForTests,
+    /// 一段都没资格（[`RegionChoice::NoEligibleSegment`]），或强制选的那一段不合格。
+    NoEligibleSegment,
+    /// 签发前判不过（D26（后台整理与放置回收） 已定项 1 第 2 条，攻方 K-g）：R 外非全空段的空洞少于 R 里要搬的单元数，不签。
+    HolesOutsideTheRegionFewerThanItsUnitsToMove { holes: u64, units_to_move: u64 },
+    /// 上一条意图完成了，全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为假（攻方 K-h）：不签下一条。
+    NoYieldFromTheLastIntent {
+        empty_cluster_segments_now: u64,
+        empty_cluster_segments_at_issue: u64,
+    },
+    /// 这次运行的发布预算用完了（判决 G8 `B_gc`）。
+    PublishBudgetSpent { budget: u64 },
+    /// 一批被空间拒（准入或落点）：暂停——撤掉 R 排他、意图留着（攻方 K-k），不推抬 F（判决 G8 g1）。
+    PausedShortOfSpace,
+    /// 签发那次发布被空间拒（准入或落点）：意图没签成、盘上没有它，R 排他跟着撤；不推抬 F（判决 G8 g1）。
+    IssueRefusedShortOfSpace,
+    /// 可写挂载续做（只续做盘上那一条未完成的意图、不签新的，D8（核心索引结构） 已定项 5 第 2 条「续做是每次恢复都跑的常规动作」）：
+    /// 那一条删了（达成或丢掉），或本来就没有活意图。
+    NoLiveIntentLeftToContinue,
+    /// R 里一个能搬的都没有、却还有未释放的记录（[`PlannedBatch::NothingMovableButTheRegionIsNotEmpty`]）：意图连同 R 排他丢掉。
+    NothingMovableButTheRegionIsNotEmpty,
+    /// 用户改动之后那一次只跑至多一批（判决 G8 丁），跑完了。
+    OneBatchAfterTheUserChangeDone,
+}
+
+impl CompactionRunStop {
+    /// 运行时报出停在哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
+    #[must_use]
+    pub const fn name(self) -> &'static str {
+        match self {
+            CompactionRunStop::BatchSizeNotDecidedByTheClauses => {
+                "compaction_stopped_batch_size_not_decided_by_the_clauses"
+            }
+            CompactionRunStop::ForcedToStopForTests => "compaction_stopped_by_the_test_only_switch",
+            CompactionRunStop::NoEligibleSegment => "compaction_stopped_no_eligible_segment",
+            CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove { .. } => {
+                "compaction_stopped_holes_outside_the_region_too_few"
+            }
+            CompactionRunStop::NoYieldFromTheLastIntent { .. } => {
+                "compaction_stopped_no_yield_from_the_last_intent"
+            }
+            CompactionRunStop::PublishBudgetSpent { .. } => {
+                "compaction_stopped_publish_budget_spent"
+            }
+            CompactionRunStop::PausedShortOfSpace => "compaction_paused_short_of_space",
+            CompactionRunStop::IssueRefusedShortOfSpace => {
+                "compaction_stopped_the_issue_refused_short_of_space"
+            }
+            CompactionRunStop::NoLiveIntentLeftToContinue => {
+                "compaction_stopped_no_live_intent_left_to_continue"
+            }
+            CompactionRunStop::NothingMovableButTheRegionIsNotEmpty => {
+                "compaction_stopped_nothing_movable_in_the_region"
+            }
+            CompactionRunStop::OneBatchAfterTheUserChangeDone => {
+                "compaction_stopped_after_one_batch_following_the_user_change"
+            }
+        }
+    }
+}
+
+/// 一条活意图在这次挂载里的样子：意图本身与它的 R 排没排他（暂停 = 撤掉 R 排他、意图留着，攻方 K-k）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct LiveCompactionIntent {
+    pub intent: CompactionIntent,
+    pub is_the_region_excluded_from_placement: bool,
+}
+
+/// 整理的运行时观测（判决 G8「起跑次数、批数、停机原因、用掉的根槽数」；D26（后台整理与放置回收） 已定项 6 的运行时计数器那两样：
+/// 一批整理的搬迁量；runs、全空聚簇段数、回落计数、不可回收字节由分配器与记账给，整理进度不存）。只住内存、只增，丢了不影响判定。
+#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
+pub struct CompactionObservations {
+    pub runs_started: u64,
+    pub intents_issued: u64,
+    pub intents_completed: u64,
+    pub intents_dropped: u64,
+    pub batches_published: u64,
+    /// 整理的发布用掉的根槽数：一次发布写一个根槽，所以就是整理发布（签发、搬一批、删意图）的次数。
+    pub root_ring_slots_used: u64,
+    pub units_relocated: u64,
+    /// 最近一批搬了几个单元（D26 已定项 6「一批整理的搬迁量」）。
+    pub units_relocated_by_the_last_batch: u64,
+    pub last_stop: Option<CompactionRunStop>,
+}
+
+/// 分配器这一侧的整理状态：只供测试的开关与活意图的 R 排他（D26（后台整理与放置回收） 已定项 1 第 3 条「`R` 在意图期内不可作分配目的地」：
+/// 分配路径多一条合取，意图住 logged_ops 树、分配器读得到）。活意图的权威在现行那一版的 logged_ops 里（`transaction::TransactionOutput::logged_operations`）；
+/// 这里记的是分配器要排除的那一份：会话签发那次发布之前装上（签发那次发布自己的新落点也不进 R）、删意图之后撤掉，可写挂载按挂载那一版的
+/// logged_ops 装上（`mount`）。另带两样只有发布路径看得见的：签发之后强制一次「释放前核出对不上」那个开关用过没有，与这次挂载里发布路径
+/// 隔离过、还没交给会话的槽（会话把它们并进常驻隔离集，[`CompactionOfThisMount::take_the_quarantines_noted_by_the_publishes`]）。
+/// 随分配器只住内存，生命周期一次挂载。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct CompactionSwitchesAndTheExcludedRegion {
+    pub switches: CompactionSwitches,
+    live_intent: Option<LiveCompactionIntent>,
+    has_the_forced_release_checksum_mismatch_been_used: bool,
+    quarantines_noted_by_the_publishes: Vec<(SlotNumber, u64)>,
+}
+
+impl CompactionSwitchesAndTheExcludedRegion {
+    /// 产品路径起步：开关是产品值，没有活意图。
+    #[must_use]
+    pub fn of_a_new_mount() -> Self {
+        Self {
+            switches: CompactionSwitches::PRODUCT,
+            live_intent: None,
+            has_the_forced_release_checksum_mismatch_been_used: false,
+            quarantines_noted_by_the_publishes: Vec::new(),
+        }
+    }
+
+    #[must_use]
+    pub fn live_intent(&self) -> Option<LiveCompactionIntent> {
+        self.live_intent
+    }
+
+    /// 分配器这一侧的签发：记下活意图，R 从此排他（开关关掉 R 排他时不排）。盘上那一条由签发那次发布写进 logged_ops
+    /// （`transaction::publish_compaction_step`）；挂载按 logged_ops 里那一条装上。全池活意图至多一条（判决 G1 e1）。
+    ///
+    /// # Panics
+    /// 已经有一条活意图：调用方先判过（会话按现行那一版的 logged_ops 判，挂载在新建的分配器上装）。
+    pub fn issue(&mut self, intent: CompactionIntent) {
+        assert!(
+            self.live_intent.is_none(),
+            "全池活整理意图至多一条（判决 G1 e1）：签发之前调用方判过没有活意图"
+        );
+        self.live_intent = Some(LiveCompactionIntent {
+            intent,
+            is_the_region_excluded_from_placement: match self.switches.region_exclusion {
+                CompactionRegionExclusion::ExcludedWhileTheIntentIsLive => true,
+                CompactionRegionExclusion::SwitchedOffForTests => false,
+            },
+        });
+    }
+
+    /// 意图删了（达成或丢掉，删它的那次发布已落盘），或签发那次发布没发成：撤掉活意图连同 R 排他，交回撤掉的那一条。
+    pub fn withdraw_the_live_intent(&mut self) -> Option<CompactionIntent> {
+        self.live_intent.take().map(|live| live.intent)
+    }
+
+    /// 暂停：撤掉 R 排他、意图留着（攻方 K-k）。下一次运行接着推它时重新排他。
+    pub fn pause_the_live_intent(&mut self) {
+        if let Some(live) = self.live_intent.as_mut() {
+            live.is_the_region_excluded_from_placement = false;
+        }
+    }
+
+    /// 接着推一条暂停过的意图：照开关重新排他。
+    pub fn resume_the_live_intent(&mut self) {
+        let is_excluded = match self.switches.region_exclusion {
+            CompactionRegionExclusion::ExcludedWhileTheIntentIsLive => true,
+            CompactionRegionExclusion::SwitchedOffForTests => false,
+        };
+        if let Some(live) = self.live_intent.as_mut() {
+            live.is_the_region_excluded_from_placement = is_excluded;
+        }
+    }
+
+    /// 分配器此刻不许当目的地的 R（活意图且排他时）。
+    #[must_use]
+    pub fn region_excluded_from_placement(&self) -> Option<CompactionRegion> {
+        self.live_intent
+            .filter(|live| live.is_the_region_excluded_from_placement)
+            .map(|live| live.intent.region)
+    }
+
+    /// 发布路径记下一次释放前核出对不上而隔离的落点（跨度里的每一个槽，两盘同槽只记槽号）：等会话取走、并进常驻隔离集。
+    /// 发布没成立时分配器换回进来时那一份，这一笔跟着没了。
+    pub fn note_quarantined_placement(&mut self, slot: SlotNumber, span: u64) {
+        self.quarantines_noted_by_the_publishes.push((slot, span));
+    }
+
+    /// 交出发布路径记下、会话还没取走的隔离落点（取走之后清空）。
+    pub fn take_the_quarantines_noted_by_the_publishes(&mut self) -> Vec<(SlotNumber, u64)> {
+        std::mem::take(&mut self.quarantines_noted_by_the_publishes)
+    }
+
+    /// 签发之后强制一次「释放前核出对不上」这一回该不该用上（用上就记下，整个挂载只一次）。
+    pub fn take_the_forced_release_checksum_mismatch(&mut self) -> bool {
+        match self.switches.release_checksum_mismatch_after_issue {
+            ForcedReleaseChecksumMismatchAfterIssue::NotForced => false,
+            ForcedReleaseChecksumMismatchAfterIssue::OnceOnTheFirstSourceOfTheFirstBatchForTests => {
+                if self.has_the_forced_release_checksum_mismatch_been_used
+                    || self.live_intent.is_none()
+                {
+                    return false;
+                }
+                self.has_the_forced_release_checksum_mismatch_been_used = true;
+                true
+            }
+        }
+    }
+}
+
+/// 会话这一侧的整理状态（丁 1 交判第 2 点：不合格段表挂会话；生命周期一次挂载）：这次挂载里被区域扫描拒过的段、这次挂载里释放前核出对不上
+/// 而隔离的槽（常驻隔离集，攻方 K-c 读它）、上一次用户改动之后整理发了几次布（`B_gc`）、观测。住在挂着的会话上
+/// （`mounted_session::MountedSession::compaction`），可写挂载续做意图时先建一份、交给会话接着用（`mount::MountOutput::compaction_continuation`）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct CompactionOfThisMount {
+    pub segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,
+    quarantined_slots: BTreeSet<SlotNumber>,
+    /// 上一次用户改动之后整理发了几次布（判决 G8 `B_gc` 式子里「已经过去的发布数」）：会话发成一次用户改动就归零。
+    pub publishes_since_the_last_user_change: u64,
+    pub observations: CompactionObservations,
+}
+
+impl CompactionOfThisMount {
+    /// 一次挂载起步：三样都空。
+    #[must_use]
+    pub fn of_a_new_mount() -> Self {
+        Self {
+            segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan::default(),
+            quarantined_slots: BTreeSet::new(),
+            publishes_since_the_last_user_change: 0,
+            observations: CompactionObservations::default(),
+        }
+    }
+
+    /// 这次挂载里释放前核出对不上而隔离的槽（常驻隔离集）。
+    #[must_use]
+    pub fn quarantined_slots(&self) -> &BTreeSet<SlotNumber> {
+        &self.quarantined_slots
+    }
+
+    /// 把发布路径记在分配器上、还没取走的隔离落点并进常驻隔离集（跨度里的每一个槽）。
+    pub fn take_the_quarantines_noted_by_the_publishes(
+        &mut self,
+        on_the_allocator: &mut CompactionSwitchesAndTheExcludedRegion,
+    ) {
+        for (slot, span) in on_the_allocator.take_the_quarantines_noted_by_the_publishes() {
+            for covered in slot.0..slot.0 + span {
+                self.quarantined_slots.insert(SlotNumber(covered));
+            }
+        }
+    }
+}
+
+/// 一块盘上按记录口径的全空聚簇段数（D3（空间分配） 已定项 10 ① 与 D26（后台整理与放置回收） 已定项 1 第 1 条收口的那一句：
+/// 段内 64 槽都没有未释放分配记录的段数，C372）。只数单元区里的整段。整理（签发时的 `segs0`、一条意图完成之后「无产出就停」）
+/// 从这一版的分配记录现扫它；记账第 11 项与合取 7 读分配器按同一个定义增量维护的那一份（`allocator::DeviceFreeMap::empty_segments`）：
+/// 定义相同，算法不同（准入路径上不许扫，整理在后台可以扫）。
+#[must_use]
+pub fn empty_cluster_segments_by_the_records(
+    allocation_records: &[AllocationRecord],
+    unit_area: UnitAreaOfOneDevice,
+) -> u64 {
+    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
+    let unit_area_start = unit_area.start.slot().0;
+    let mut segments_with_an_unreleased_record: BTreeSet<u64> = BTreeSet::new();
+    for record in allocation_records
+        .iter()
+        .filter(|record| record.device == unit_area.device && !record.is_released)
+    {
+        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
+            if let Some(offset_in_unit_area) = slot.checked_sub(unit_area_start) {
+                let segment = offset_in_unit_area / CLUSTER_SEGMENT_SLOTS;
+                if segment < full_segments {
+                    segments_with_an_unreleased_record.insert(segment);
+                }
+            }
+        }
+    }
+    full_segments - u64::try_from(segments_with_an_unreleased_record.len()).expect("段数")
+}
+
+/// 签发前判（D26（后台整理与放置回收） 已定项 1 第 2 条，攻方 K-g）的左边：R 那块盘上、R 之外、「非全空」的段里没挡的槽数。
+/// 「非全空」按记录口径；只装提交内生块的段不算非全空——段里至少一个槽罩在 `slots_holding_user_data`（码 1 数据单元的落点，
+/// 两盘同槽只比槽号）里才算。`is_unblocked(槽)` 说那一槽此刻能不能当目的地（已分配、隔离、抬 F 扣住三种位一个都没置）。
+#[must_use]
+pub fn holes_outside_the_region_in_segments_holding_user_data<IsUnblocked>(
+    allocation_records: &[AllocationRecord],
+    slots_holding_user_data: &BTreeSet<SlotNumber>,
+    unit_area: UnitAreaOfOneDevice,
+    region: CompactionRegion,
+    is_unblocked: IsUnblocked,
+) -> u64
+where
+    IsUnblocked: Fn(SlotNumber) -> bool,
+{
+    let unit_area_start = unit_area.start.slot().0;
+    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
+    let mut segments_holding_user_data: BTreeSet<u64> = BTreeSet::new();
+    for record in allocation_records
+        .iter()
+        .filter(|record| record.device == unit_area.device && !record.is_released)
+    {
+        let holds_user_data = (record.slot.0..record.slot.0 + u64::from(record.span_slots))
+            .any(|slot| slots_holding_user_data.contains(&SlotNumber(slot)));
+        if !holds_user_data {
+            continue;
+        }
+        if let Some(offset_in_unit_area) = record.slot.0.checked_sub(unit_area_start) {
+            let segment = offset_in_unit_area / CLUSTER_SEGMENT_SLOTS;
+            if segment < full_segments {
+                segments_holding_user_data.insert(segment);
+            }
+        }
+    }
+    segments_holding_user_data
+        .into_iter()
+        .map(|segment| unit_area_start + segment * CLUSTER_SEGMENT_SLOTS)
+        .filter(|segment_start| !region.contains_slot(SlotNumber(*segment_start)))
+        .map(|segment_start| {
+            u64::try_from(
+                (segment_start..segment_start + CLUSTER_SEGMENT_SLOTS)
+                    .filter(|slot| is_unblocked(SlotNumber(*slot)))
+                    .count(),
+            )
+            .expect("一段 64 槽")
+        })
+        .sum()
+}
+
+/// 单元区里没有一条未释放记录罩到的整段，各交一份空的扫描结果（[`crate::region_scan::scan_region`] 对这样的段交回的就是它：
+/// 没有起点在段里的记录，也没有跨进段里的记录）。选段之前带上它们，[`choose_region`] 就不为每个全空段各要一次「先扫」——
+/// 否则一次签发要扫单元区的每一个全空段、每扫一段再选一遍。只读记录，不读设备。
+#[must_use]
+pub fn region_scans_of_the_segments_without_an_unreleased_record(
+    allocation_records: &[AllocationRecord],
+    unit_area: UnitAreaOfOneDevice,
+) -> Vec<RegionScan> {
+    let unit_area_start = unit_area.start.slot().0;
+    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
+    let mut segments_touched_by_an_unreleased_record: BTreeSet<u64> = BTreeSet::new();
+    for record in allocation_records
+        .iter()
+        .filter(|record| record.device == unit_area.device && !record.is_released)
+    {
+        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
+            if let Some(offset_in_unit_area) = slot.checked_sub(unit_area_start) {
+                segments_touched_by_an_unreleased_record
+                    .insert(offset_in_unit_area / CLUSTER_SEGMENT_SLOTS);
+            }
+        }
+    }
+    (0..full_segments)
+        .filter(|segment| !segments_touched_by_an_unreleased_record.contains(segment))
+        .map(|segment| RegionScan {
+            region: CompactionRegion::of_one_cluster_segment(
+                unit_area.device,
+                SlotNumber(unit_area_start + segment * CLUSTER_SEGMENT_SLOTS),
+            )
+            .expect("单元区起点在聚簇段边界上（UnitAreaStart 的不变量），整段的起点都是 64 的倍数"),
+            allocations: Vec::new(),
+        })
+        .collect()
+}
+
+/// 签发前判（D26（后台整理与放置回收） 已定项 1 第 2 条）：空洞数 ≥ R 里要搬的单元数才签。
+#[must_use]
+pub const fn holes_admit_the_issue(holes: u64, units_to_move: u64) -> bool {
+    holes >= units_to_move
+}
+
+/// 停机谓词（D26（后台整理与放置回收） 已定项 1 第 2 条，判决 G4）：删意图那一刻 R 按记录全空——R 那块盘上起点在 R 里的未释放记录一条都没有。
+#[must_use]
+pub fn region_is_empty_by_the_records(
+    allocation_records: &[AllocationRecord],
+    region: CompactionRegion,
+) -> bool {
+    !allocation_records.iter().any(|record| {
+        record.device == region.device() && !record.is_released && region.contains_slot(record.slot)
+    })
+}
+
+/// 一条意图完成之后签不签下一条（攻方 K-h）：全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为真才签。
+#[must_use]
+pub const fn the_completed_intent_yielded(
+    empty_cluster_segments_now: u64,
+    empty_cluster_segments_at_issue: u64,
+) -> bool {
+    empty_cluster_segments_now > empty_cluster_segments_at_issue
+}
+
+/// 签发之后 R 里出现了隔离槽没有（攻方 K-c；丁 1 交判第 4 点：一个槽既钉住又在隔离集里，区域扫描判「钉住」照旧，这一判另外直接查隔离集，
+/// 两者同时成立也算有）。
+#[must_use]
+pub fn region_holds_a_quarantined_slot(
+    region: CompactionRegion,
+    quarantined_slots: &BTreeSet<SlotNumber>,
+) -> bool {
+    quarantined_slots
+        .range(region.start_slot()..region.end_slot())
+        .next()
+        .is_some()
+}
+
+/// 一次整理运行的发布预算（判决 G8 `B_gc`，正推 O11：整理的发布不改用户可见状态，连发会把回退候选集挤成 1 个状态）：
+/// `3S − 4 − 已经过去的发布数`，至少 1。`S` 是每区根槽数。
+#[must_use]
+pub fn publish_budget_of_a_compaction_run(
+    root_ring_slots_per_region: u64,
+    publishes_already_passed: u64,
+) -> u64 {
+    (3 * root_ring_slots_per_region)
+        .saturating_sub(4)
+        .saturating_sub(publishes_already_passed)
+        .max(1)
+}
+
+/// 用户改动发成之后起不起跑整理（判决 G8 丁：水位越线再跑至多一批）。第一版三条水位恒 0（关），产品路径恒不起跑；只供测试的开关强制。
+#[must_use]
+pub const fn compaction_starts_after_a_user_change(forcing: CompactionRunForcing) -> bool {
+    match forcing {
+        CompactionRunForcing::FollowTheWatermarks | CompactionRunForcing::ForcedToStopForTests => {
+            false
+        }
+        CompactionRunForcing::ForcedToStartForTests => true,
+    }
+}
+
+/// 与 Verus 证过的选段核、计划核逐项对拍（`.claude/kb/milestone/04-fourth-txn.md` 二·4；判决 `research/prompts/m4-gc-r1-main-verification.md`
+/// 第三节 G7 的 P1、P2、P3 证在 `singlefs_replay_prefix::compaction_plan_proofs` 的两个核上）：小输入上每一种形状都喂给这里的
+/// [`choose_region`]、[`plan_batch`] 与核，答案逐项相同。核那一侧的入参由用例按形状直接写出，不经这里的代码算，
+/// 所以两边对上就是这里的实现在这些输入上合核证过的性质。核心层不调核：`crates/mutations.tsv` 里锚在这里现在写法上的变异行要原样留着。
+#[cfg(test)]
+mod tests {
+    use std::collections::{BTreeMap, BTreeSet};
+
+    use singlefs_format::CLUSTER_SEGMENT_SLOTS;
+    use singlefs_replay_prefix::compaction_plan_proofs::{
+        choose_compaction_segment, plan_compaction_batch, CompactionKernelAllocationClass,
+        CompactionKernelPlannedBatch, CompactionKernelRegionChoice,
+        CompactionKernelScannedAllocation, CompactionKernelSegment, CompactionKernelSegmentScan,
+    };
+
+    use super::{
+        choose_region, plan_batch, CompactionBatchUnits, PlannedBatch, PlannedRelocation,
+        RegionChoice, RegionChoiceCriterion, SegmentsRefusedByTheRegionScan, UnitAreaOfOneDevice,
+    };
+    use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+    use crate::allocator::{AllocationRecord, Placement, UnitAreaStart};
+    use crate::logged_operations::CompactionRegion;
+    use crate::region_scan::{
+        AllocationInRegion, MappedUnitClass, MovableUnit, RegionScan, RegionScanInconsistency,
+        RegionScanRefusal, ScannedAllocation,
+    };
+
+    const DEVICE_OF_THE_UNIT_AREA: DeviceIdentity = DeviceIdentity(0);
+    /// 另一块盘上的记录：选段只数单元区那块盘上的活槽，核的入参里没有它们。
+    const ANOTHER_DEVICE: DeviceIdentity = DeviceIdentity(1);
+    const MOST_SEGMENTS_IN_THE_UNIT_AREA: usize = 3;
+    const MOST_LIVE_RECORDS_IN_A_SEGMENT: u64 = 2;
+    const MOST_ALLOCATIONS_IN_A_REGION: usize = 4;
+    const LARGEST_BATCH_UNITS: usize = 3;
+    /// 每段里一条已释放的记录落在这个偏移上：选段不数它，核的入参里也没有它。
+    const RELEASED_RECORD_OFFSET_IN_THE_SEGMENT: u64 = 40;
+    /// 钉住的槽落在段里的这个偏移上。
+    const PINNED_SLOT_OFFSET_IN_THE_SEGMENT: u64 = 50;
+    /// 计划用例里一条记录的跨度（码 1 数据单元）。
+    const SPAN_OF_A_PLANNED_ALLOCATION: u64 = 2;
+
+    /// 一段不合资格的那一条（每段至多一条：几条一起不合格时核与这里都只看合取，各条单独对得上就够）。
+    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
+    enum SegmentExclusion {
+        NotExcluded,
+        OpenedInThisMount,
+        HoldsAPinnedSlot,
+        RefusedByTheRegionScan,
+    }
+
+    const SEGMENT_EXCLUSIONS: [SegmentExclusion; 4] = [
+        SegmentExclusion::NotExcluded,
+        SegmentExclusion::OpenedInThisMount,
+        SegmentExclusion::HoldsAPinnedSlot,
+        SegmentExclusion::RefusedByTheRegionScan,
+    ];
+
+    const SEGMENT_SCANS: [CompactionKernelSegmentScan; 3] = [
+        CompactionKernelSegmentScan::NotScanned,
+        CompactionKernelSegmentScan::ScannedWithAMovableUnit,
+        CompactionKernelSegmentScan::ScannedWithoutAMovableUnit,
+    ];
+
+    #[derive(Clone, Copy, Debug)]
+    struct SegmentShape {
+        live_records: u64,
+        exclusion: SegmentExclusion,
+        scan: CompactionKernelSegmentScan,
+    }
+
+    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
+    enum AllocationShape {
+        Movable,
+        Pinned,
+        Quarantined,
+        Dead,
+        Inconsistent,
+    }
+
+    const ALLOCATION_SHAPES: [AllocationShape; 5] = [
+        AllocationShape::Movable,
+        AllocationShape::Pinned,
+        AllocationShape::Quarantined,
+        AllocationShape::Dead,
+        AllocationShape::Inconsistent,
+    ];
+
+    /// `alphabet_size` 个字母组成的、长 `length` 的全部序列（每个位置一个字母下标）。
+    fn every_sequence(alphabet_size: usize, length: usize) -> Vec<Vec<usize>> {
+        let mut sequences: Vec<Vec<usize>> = vec![Vec::new()];
+        for _position in 0..length {
+            sequences = sequences
+                .into_iter()
+                .flat_map(|prefix| {
+                    (0..alphabet_size).map(move |letter| {
+                        let mut longer = prefix.clone();
+                        longer.push(letter);
+                        longer
+                    })
+                })
+                .collect();
+        }
+        sequences
+    }
+
+    fn unit_area_start_slot() -> u64 {
+        UnitAreaStart::of_the_default_journal_ring().slot().0
+    }
+
+    fn segment_start_slot(segment_index: usize) -> u64 {
+        unit_area_start_slot()
+            + u64::try_from(segment_index).expect("至多三段") * CLUSTER_SEGMENT_SLOTS
+    }
+
+    fn region_of_the_segment(segment_index: usize) -> CompactionRegion {
+        CompactionRegion::of_one_cluster_segment(
+            DEVICE_OF_THE_UNIT_AREA,
+            SlotNumber(segment_start_slot(segment_index)),
+        )
+        .expect("默认单元区起点在聚簇段边界上，整段起点都是 64 的倍数")
+    }
+
+    fn record(device: DeviceIdentity, slot: u64, is_released: bool) -> AllocationRecord {
+        AllocationRecord {
+            device,
+            slot: SlotNumber(slot),
+            span_slots: 1,
+            generation: CheckpointTxg(1),
+            is_released,
+        }
+    }
+
+    fn movable_unit(mapping_key: Vec<u8>) -> MovableUnit {
+        MovableUnit {
+            mapping_key,
+            unit_class: MappedUnitClass::Data,
+            birth_txg: CheckpointTxg(1),
+        }
+    }
+
+    /// 一种形状喂给这里的 `choose_region` 交回的答案。
+    fn choice_of_this_implementation(
+        shapes: &[SegmentShape],
+        live_intent_segment: Option<usize>,
+    ) -> RegionChoice {
+        let mut allocation_records = Vec::new();
+        let mut cluster_segments = BTreeSet::new();
+        let mut pinned_slots = BTreeSet::new();
+        let mut region_scans = Vec::new();
+        let mut segments_refused = SegmentsRefusedByTheRegionScan::default();
+        for (segment_index, shape) in shapes.iter().enumerate() {
+            let start_slot = segment_start_slot(segment_index);
+            for live_record in 0..shape.live_records {
+                allocation_records.push(record(
+                    DEVICE_OF_THE_UNIT_AREA,
+                    start_slot + 2 * live_record,
+                    false,
+                ));
+                allocation_records.push(record(
+                    ANOTHER_DEVICE,
+                    start_slot + 2 * live_record + 1,
+                    false,
+                ));
+            }
+            allocation_records.push(record(
+                DEVICE_OF_THE_UNIT_AREA,
+                start_slot + RELEASED_RECORD_OFFSET_IN_THE_SEGMENT,
+                true,
+            ));
+            let region = region_of_the_segment(segment_index);
+            match shape.exclusion {
+                SegmentExclusion::NotExcluded => {}
+                SegmentExclusion::OpenedInThisMount => {
+                    cluster_segments.insert(SlotNumber(start_slot));
+                }
+                SegmentExclusion::HoldsAPinnedSlot => {
+                    pinned_slots.insert(SlotNumber(start_slot + PINNED_SLOT_OFFSET_IN_THE_SEGMENT));
+                }
+                SegmentExclusion::RefusedByTheRegionScan => segments_refused.note_refusal(
+                    region,
+                    RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
+                        device: DEVICE_OF_THE_UNIT_AREA,
+                        slot: SlotNumber(start_slot),
+                        span_slots: 2,
+                    },
+                ),
+            }
+            let scanned_class = match shape.scan {
+                CompactionKernelSegmentScan::NotScanned => None,
+                CompactionKernelSegmentScan::ScannedWithAMovableUnit => {
+                    Some(AllocationInRegion::Movable(movable_unit(vec![
+                        1,
+                        u8::try_from(segment_index).expect("至多三段"),
+                    ])))
+                }
+                CompactionKernelSegmentScan::ScannedWithoutAMovableUnit => {
+                    Some(AllocationInRegion::Dead)
+                }
+            };
+            if let Some(class) = scanned_class {
+                region_scans.push(RegionScan {
+                    region,
+                    allocations: vec![ScannedAllocation {
+                        placement: Placement {
+                            slot: SlotNumber(start_slot),
+                            span: 1,
+                        },
+                        class,
+                    }],
+                });
+            }
+        }
+        choose_region(
+            &allocation_records,
+            UnitAreaOfOneDevice {
+                device: DEVICE_OF_THE_UNIT_AREA,
+                start: UnitAreaStart::of_the_default_journal_ring(),
+                slots: u64::try_from(shapes.len()).expect("至多三段") * CLUSTER_SEGMENT_SLOTS,
+            },
+            &cluster_segments,
+            live_intent_segment.map(region_of_the_segment),
+            &pinned_slots,
+            &region_scans,
+            &segments_refused,
+            RegionChoiceCriterion::Greedy,
+        )
+    }
+
+    /// 同一种形状直接写成核的入参，核交回的下标换回段。
+    fn choice_of_the_verified_kernel(
+        shapes: &[SegmentShape],
+        live_intent_segment: Option<usize>,
+    ) -> RegionChoice {
+        let segments: Vec<CompactionKernelSegment> = shapes
+            .iter()
+            .enumerate()
+            .map(|(segment_index, shape)| CompactionKernelSegment {
+                live_slots: shape.live_records,
+                is_opened_in_this_mount: shape.exclusion == SegmentExclusion::OpenedInThisMount,
+                is_the_live_intent_region: live_intent_segment == Some(segment_index),
+                holds_a_pinned_slot: shape.exclusion == SegmentExclusion::HoldsAPinnedSlot,
+                is_refused_by_the_region_scan: shape.exclusion
+                    == SegmentExclusion::RefusedByTheRegionScan,
+                scan: shape.scan,
+            })
+            .collect();
+        match choose_compaction_segment(&segments) {
+            CompactionKernelRegionChoice::Chosen(segment_index) => {
+                RegionChoice::Chosen(region_of_the_segment(segment_index))
+            }
+            CompactionKernelRegionChoice::ScanBeforeChoosing(segment_index) => {
+                RegionChoice::ScanBeforeChoosing(region_of_the_segment(segment_index))
+            }
+            CompactionKernelRegionChoice::NoEligibleSegment => RegionChoice::NoEligibleSegment,
+        }
+    }
+
+    /// 一到三段、每段活槽 0 到 2、至多一条不合资格、三种扫描结果、活意图的 R 是哪一段（或没有），全部组合：
+    /// 这里的选段与证过 P1 的核逐项相同；三种答案都至少出现过一次（对拍不空转）。
+    #[test]
+    fn choose_region_agrees_with_the_verified_segment_choice_on_every_small_unit_area() {
+        let shapes: Vec<SegmentShape> = (0..=MOST_LIVE_RECORDS_IN_A_SEGMENT)
+            .flat_map(|live_records| {
+                SEGMENT_EXCLUSIONS.into_iter().flat_map(move |exclusion| {
+                    SEGMENT_SCANS.into_iter().map(move |scan| SegmentShape {
+                        live_records,
+                        exclusion,
+                        scan,
+                    })
+                })
+            })
+            .collect();
+        let mut answers_by_kind: BTreeMap<&'static str, u64> = BTreeMap::new();
+        for segments in 1..=MOST_SEGMENTS_IN_THE_UNIT_AREA {
+            for shape_indices in every_sequence(shapes.len(), segments) {
+                let segment_shapes: Vec<SegmentShape> = shape_indices
+                    .iter()
+                    .map(|shape_index| shapes[*shape_index])
+                    .collect();
+                let live_intent_choices = std::iter::once(None).chain((0..segments).map(Some));
+                for live_intent_segment in live_intent_choices {
+                    let this_implementation =
+                        choice_of_this_implementation(&segment_shapes, live_intent_segment);
+                    let verified_kernel =
+                        choice_of_the_verified_kernel(&segment_shapes, live_intent_segment);
+                    assert_eq!(
+                        this_implementation, verified_kernel,
+                        "段的形状 {segment_shapes:?}、活意图在段 {live_intent_segment:?}：这里的选段与证过的核不同"
+                    );
+                    let kind = match this_implementation {
+                        RegionChoice::Chosen(_) => "chosen",
+                        RegionChoice::ScanBeforeChoosing(_) => "scan_before_choosing",
+                        RegionChoice::NoEligibleSegment => "no_eligible_segment",
+                    };
+                    *answers_by_kind.entry(kind).or_insert(0) += 1;
+                }
+            }
+        }
+        assert_eq!(
+            answers_by_kind.len(),
+            3,
+            "三种答案都该出现过，实际 {answers_by_kind:?}"
+        );
+    }
+
+    /// 一种形状的扫描结果：第 `position` 项的源落点在段起点之后 `2 × position`，能搬的 key 两两不同。
+    fn region_scan_of(shapes: &[AllocationShape]) -> RegionScan {
+        let start_slot = segment_start_slot(0);
+        RegionScan {
+            region: region_of_the_segment(0),
+            allocations: shapes
+                .iter()
+                .enumerate()
+                .map(|(position, shape)| ScannedAllocation {
+                    placement: Placement {
+                        slot: SlotNumber(
+                            start_slot
+                                + SPAN_OF_A_PLANNED_ALLOCATION
+                                    * u64::try_from(position).expect("至多四项"),
+                        ),
+                        span: SPAN_OF_A_PLANNED_ALLOCATION,
+                    },
+                    class: match shape {
+                        AllocationShape::Movable => {
+                            AllocationInRegion::Movable(movable_unit(vec![
+                                1,
+                                u8::try_from(position).expect("至多四项"),
+                            ]))
+                        }
+                        AllocationShape::Pinned => AllocationInRegion::Pinned,
+                        AllocationShape::Quarantined => AllocationInRegion::Quarantined,
+                        AllocationShape::Dead => AllocationInRegion::Dead,
+                        AllocationShape::Inconsistent => AllocationInRegion::Inconsistent(
+                            RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
+                        ),
+                    },
+                })
+                .collect(),
+        }
+    }
+
+    /// 同一份扫描结果写成核的入参（映射 key 换成身份：同一把 key 同一个数），核交回的下标换回（映射 key, 源落点）。
+    fn plan_of_the_verified_kernel(region_scan: &RegionScan, batch_units: usize) -> PlannedBatch {
+        let mut identities_of_the_keys: BTreeMap<Vec<u8>, u64> = BTreeMap::new();
+        let allocations: Vec<CompactionKernelScannedAllocation> = region_scan
+            .allocations
+            .iter()
+            .map(|allocation| CompactionKernelScannedAllocation {
+                source_slot: allocation.placement.slot.0,
+                class: match &allocation.class {
+                    AllocationInRegion::Movable(unit) => {
+                        let next_identity =
+                            u64::try_from(identities_of_the_keys.len()).expect("至多四项");
+                        CompactionKernelAllocationClass::Movable {
+                            mapping_key_identity: *identities_of_the_keys
+                                .entry(unit.mapping_key.clone())
+                                .or_insert(next_identity),
+                        }
+                    }
+                    AllocationInRegion::Pinned => CompactionKernelAllocationClass::Pinned,
+                    AllocationInRegion::Quarantined => CompactionKernelAllocationClass::Quarantined,
+                    AllocationInRegion::Dead => CompactionKernelAllocationClass::Dead,
+                    AllocationInRegion::Inconsistent(_) => {
+                        CompactionKernelAllocationClass::Inconsistent
+                    }
+                },
+            })
+            .collect();
+        match plan_compaction_batch(&allocations, batch_units) {
+            CompactionKernelPlannedBatch::Relocate(positions) => PlannedBatch::Relocate(
+                positions
+                    .into_iter()
+                    .map(|position| {
+                        let allocation = &region_scan.allocations[position];
+                        let AllocationInRegion::Movable(unit) = &allocation.class else {
+                            panic!("核证过：计划里每一项都是能搬的（P2），这里是第 {position} 项 {allocation:?}");
+                        };
+                        PlannedRelocation {
+                            mapping_key: unit.mapping_key.clone(),
+                            source: allocation.placement,
+                        }
+                    })
+                    .collect(),
+            ),
+            CompactionKernelPlannedBatch::RegionEmptied => PlannedBatch::RegionEmptied,
+            CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty => {
+                PlannedBatch::NothingMovableButTheRegionIsNotEmpty
+            }
+        }
+    }
+
+    /// R 里零到四条记录、每条五种分类之一、K 取 1 到 3，全部组合：这里的计划与证过 P2、P3 的核逐项相同；
+    /// 三种答案都至少出现过一次（对拍不空转）。
+    #[test]
+    fn plan_batch_agrees_with_the_verified_batch_plan_on_every_small_region_scan() {
+        let mut answers_by_kind: BTreeMap<&'static str, u64> = BTreeMap::new();
+        for allocations in 0..=MOST_ALLOCATIONS_IN_A_REGION {
+            for shape_indices in every_sequence(ALLOCATION_SHAPES.len(), allocations) {
+                let shapes: Vec<AllocationShape> = shape_indices
+                    .iter()
+                    .map(|shape_index| ALLOCATION_SHAPES[*shape_index])
+                    .collect();
+                let region_scan = region_scan_of(&shapes);
+                assert!(
+                    region_scan.movable_mapping_keys_are_pairwise_distinct(),
+                    "核证的 P2「一批里的 key 互异」是蕴含，前提是扫描结果里能搬的 key 两两不同（scan_region 交回 Ok 时成立）：\
+                     喂给对拍的形状 {shapes:?} 也要满足它"
+                );
+                for batch_units in 1..=LARGEST_BATCH_UNITS {
+                    let this_implementation = plan_batch(
+                        &region_scan,
+                        CompactionBatchUnits::new(batch_units).expect("K 从 1 起"),
+                    );
+                    let verified_kernel = plan_of_the_verified_kernel(&region_scan, batch_units);
+                    assert_eq!(
+                        this_implementation, verified_kernel,
+                        "扫描结果的形状 {shapes:?}、K = {batch_units}：这里的计划与证过的核不同"
+                    );
+                    let kind = match this_implementation {
+                        PlannedBatch::Relocate(_) => "relocate",
+                        PlannedBatch::RegionEmptied => "region_emptied",
+                        PlannedBatch::NothingMovableButTheRegionIsNotEmpty => {
+                            "nothing_movable_but_the_region_is_not_empty"
+                        }
+                    };
+                    *answers_by_kind.entry(kind).or_insert(0) += 1;
+                }
+            }
+        }
+        assert_eq!(
+            answers_by_kind.len(),
+            3,
+            "三种答案都该出现过，实际 {answers_by_kind:?}"
+        );
+    }
+
+    /// 两项能搬、映射 key 相同的扫描结果（`scan_region` 交回 `Ok` 时走不到：它见到这样两项就拒这个 R）进计划就断言，
+    /// 不让同一把 key 进同一批两次（判决 G7 的 P2）。
+    #[test]
+    #[should_panic(expected = "扫描结果里能搬的映射 key 两两不同")]
+    fn plan_batch_refuses_a_region_scan_holding_two_movable_units_under_one_mapping_key() {
+        let mut region_scan = region_scan_of(&[AllocationShape::Movable, AllocationShape::Movable]);
+        let AllocationInRegion::Movable(first_unit) = region_scan.allocations[0].class.clone()
+        else {
+            panic!("第 0 项照形状是能搬的");
+        };
+        region_scan.allocations[1].class = AllocationInRegion::Movable(first_unit);
+        let _planned = plan_batch(
+            &region_scan,
+            CompactionBatchUnits::new(2).expect("K 从 1 起"),
+        );
+    }
+}
diff --git a/crates/singlefs-core/src/logged_operations.rs b/crates/singlefs-core/src/logged_operations.rs
new file mode 100644
index 00000000..927d0d41
--- /dev/null
+++ b/crates/singlefs-core/src/logged_operations.rs
@@ -0,0 +1,366 @@
+//! logged_ops 树的条目（D8（核心索引结构） 已定项 5 的意图树；字段表是判决 `research/prompts/m4-gc-r1-main-verification.md`
+//! 第三节 G1「单树、定宽联合条目」）：key 10 = 种类 2 + 签发 txg 8，value 34 定宽、按种类解，条目 44，一个码 2 节点装 369 条。
+//! 第一版只装整理意图（种类 1）：R（设备 4 + 起始槽 6 + 槽数 2）、`c0` 8、`segs0` 8、补零 6；别的种类码保留，读者拒收
+//! （照树表条目 flags「未知位一律拒收」的政策，D8（核心索引结构） 已定项 8）。
+//!
+//! **不存续做点、不存暂停标志**（判决 G1，攻方 F1 / F4）：暂停就是意图留着不推进；续做每批照目标态从 R 起点重扫，
+//! 删意图之前对整个 R 全量核目标态。
+//!
+//! 和 `crate::lineage_tree`、`crate::tombstone_tree` 一样只算、不发一个写：这里只有字段表的读写，签发、推进、删意图归发布入口。
+
+use singlefs_format::{
+    CLUSTER_SEGMENT_SLOTS, LOGGED_OPERATIONS_ENTRY_BYTES, LOGGED_OPERATIONS_KEY_BYTES,
+};
+
+use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+use crate::bytes::{ByteReader, ByteWriter};
+use crate::unit::index_node_entry_capacity;
+
+/// logged_ops 树的 key 宽：种类 2 + 签发 txg 8（判决 G1）。码 2 节点头里的自述 key 宽要与它相符。
+pub const LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES: usize = 10;
+
+/// logged_ops 条目的种类码：1 = 整理（判决 G1）。别的码保留：读者拒收，不当未来版本的条目猜着读。
+pub const LOGGED_OPERATION_KIND_COMPACTION: u16 = 1;
+
+/// 整理意图 value 末尾的补零：34 − (4 + 6 + 2 + 8 + 8)。读者遇到非零一律拒收（同树表条目预留位的政策）。
+const COMPACTION_INTENT_PADDING_BYTES: usize = 6;
+
+/// 槽号在盘上占 6 字节（D3（空间分配） 已定项 7）：起始槽要小于它才写得下。
+const SIX_BYTE_SLOT_NUMBER_LIMIT: u64 = 1 << 48;
+
+/// logged_ops 条目的种类。封闭集合：加一种意图就加一个成员，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
+pub enum LoggedOperationKind {
+    /// 整理意图（D26（后台整理与放置回收） 已定项 2；提示类，D8（核心索引结构） 已定项 5）。
+    Compaction,
+}
+
+impl LoggedOperationKind {
+    /// 写进 key 前 2 字节的种类码。
+    #[must_use]
+    pub const fn code(self) -> u16 {
+        match self {
+            LoggedOperationKind::Compaction => LOGGED_OPERATION_KIND_COMPACTION,
+        }
+    }
+
+    /// 盘上读到的种类码认不认得：认不得的（保留码）交回 `None`。
+    #[must_use]
+    pub const fn of_code(kind_code: u16) -> Option<Self> {
+        match kind_code {
+            LOGGED_OPERATION_KIND_COMPACTION => Some(LoggedOperationKind::Compaction),
+            _reserved_kind_code => None,
+        }
+    }
+}
+
+/// 整理意图的区域 R：一块盘上一个 64 槽对齐的聚簇段（判决 G1：起始槽 64 对齐、槽数 = 64；第一版两盘同槽时 R 罩住每块盘上的同号槽，
+/// 攻方 K-n）。只从 [`CompactionRegion::of_one_cluster_segment`] 进来，所以槽数不存：恒是 [`CLUSTER_SEGMENT_SLOTS`]。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
+pub struct CompactionRegion {
+    device: DeviceIdentity,
+    start_slot: SlotNumber,
+}
+
+/// 造不出一个整理区域的两种起始槽。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionRegionRefusal {
+    /// 起始槽不是 64 的倍数：R 不对齐（判决 G1「R 的解码拒不对齐的值」）。
+    StartOffAClusterSegmentBoundary { start_slot: SlotNumber },
+    /// 起始槽装不进盘上 6 字节的槽号。
+    StartBeyondTheSixByteSlotNumber { start_slot: SlotNumber },
+}
+
+impl CompactionRegion {
+    /// `device` 上从 `start_slot` 起的那一个聚簇段。
+    ///
+    /// # Errors
+    /// 起始槽不是 64 的倍数 ⇒ `StartOffAClusterSegmentBoundary`；装不进 6 字节 ⇒ `StartBeyondTheSixByteSlotNumber`。
+    pub fn of_one_cluster_segment(
+        device: DeviceIdentity,
+        start_slot: SlotNumber,
+    ) -> Result<Self, CompactionRegionRefusal> {
+        if !start_slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {
+            return Err(CompactionRegionRefusal::StartOffAClusterSegmentBoundary { start_slot });
+        }
+        if start_slot.0 >= SIX_BYTE_SLOT_NUMBER_LIMIT {
+            return Err(CompactionRegionRefusal::StartBeyondTheSixByteSlotNumber { start_slot });
+        }
+        Ok(Self { device, start_slot })
+    }
+
+    #[must_use]
+    pub const fn device(self) -> DeviceIdentity {
+        self.device
+    }
+
+    #[must_use]
+    pub const fn start_slot(self) -> SlotNumber {
+        self.start_slot
+    }
+
+    /// 槽数：恒是一个聚簇段。
+    #[must_use]
+    pub const fn slot_count(self) -> u64 {
+        CLUSTER_SEGMENT_SLOTS
+    }
+
+    /// R 之后的第一个槽（开区间的右端）。
+    #[must_use]
+    pub const fn end_slot(self) -> SlotNumber {
+        SlotNumber(self.start_slot.0 + CLUSTER_SEGMENT_SLOTS)
+    }
+
+    /// 这个槽号在不在 R 里（只看槽号：两盘同槽时 R 罩住每块盘上的同号槽）。
+    #[must_use]
+    pub const fn contains_slot(self, slot: SlotNumber) -> bool {
+        slot.0 >= self.start_slot.0 && slot.0 < self.start_slot.0 + CLUSTER_SEGMENT_SLOTS
+    }
+}
+
+/// 一条整理意图（D26（后台整理与放置回收） 已定项 1 / 已定项 2；字段表判决 G1）。三个字段都是值，不是增量
+/// （D8（核心索引结构） 已定项 5 硬要求 1）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CompactionIntent {
+    pub region: CompactionRegion,
+    /// `c0`：签发那一刻的分配代；目标态是「R 内不存在分配代 ≤ `c0` 且未释放的落点」（D26 已定项 2）。
+    pub allocation_generation_at_issue: CheckpointTxg,
+    /// `segs0`：签发那次发布写下的记账第 11 项（全空聚簇段数）里 R 所在盘的值；只作观测量，停机不靠它（判决 G1 / G4）。
+    pub empty_cluster_segments_at_issue: u64,
+}
+
+/// logged_ops 条目的 value，按种类分。种类码从成员推出来，key 与 value 对不上的条目造不出来。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum LoggedOperation {
+    Compaction(CompactionIntent),
+}
+
+impl LoggedOperation {
+    #[must_use]
+    pub const fn kind(&self) -> LoggedOperationKind {
+        match self {
+            LoggedOperation::Compaction(_intent) => LoggedOperationKind::Compaction,
+        }
+    }
+}
+
+/// logged_ops 条目的 key：种类 + 签发 txg。全序按逐字段无符号整数比（D8（核心索引结构） 已定项 11：小端存储不构成 memcmp 序），
+/// 先按种类、再按签发先后。一次发布至多签发一条（判决 G1 e1），所以 key 不带序号。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
+pub struct LoggedOperationKey {
+    pub kind: LoggedOperationKind,
+    pub issued_txg: CheckpointTxg,
+}
+
+impl LoggedOperationKey {
+    #[must_use]
+    pub fn sort_key(&self) -> (u16, u64) {
+        (self.kind.code(), self.issued_txg.0)
+    }
+}
+
+/// logged_ops 树里的一条：签发那次发布的 txg 与它声明的意图。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct LoggedOperationEntry {
+    pub issued_txg: CheckpointTxg,
+    pub operation: LoggedOperation,
+}
+
+/// 读者拒收一条 logged_ops 条目的原因。条目是盘上读来的字节、不是我们的不变量：调用方把它当损坏处置。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum LoggedOperationEntryRefusal {
+    /// 条目宽（码 2 节点头里的盘上字段）不是字段表的 44。
+    WidthIsNotTheFieldTable { entry_bytes: usize },
+    /// 种类码是保留码。
+    UnknownKind { kind_code: u16 },
+    /// 整理意图的 R 起始槽不对齐、或装不进 6 字节。
+    CompactionRegionRefused(CompactionRegionRefusal),
+    /// 整理意图的 R 槽数不是一个聚簇段（64）。
+    CompactionRegionSlotCountIsNotOneClusterSegment { slot_count: u16 },
+    /// 整理意图 value 末尾那 6 字节补零有非零字节。
+    CompactionPaddingNotZero,
+}
+
+fn entry_bytes() -> usize {
+    usize::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44")
+}
+
+impl LoggedOperationEntry {
+    #[must_use]
+    pub const fn key(&self) -> LoggedOperationKey {
+        LoggedOperationKey {
+            kind: self.operation.kind(),
+            issued_txg: self.issued_txg,
+        }
+    }
+
+    /// 条目 44：种类 0 | 签发 txg 2 | 整理：R 的设备 10 | R 的起始槽 14 | R 的槽数 20 | `c0` 22 | `segs0` 30 | 补零 38，小端。
+    #[must_use]
+    pub fn to_bytes(&self) -> Vec<u8> {
+        let mut writer = ByteWriter::new(entry_bytes());
+        writer.put_u16(self.operation.kind().code());
+        writer.put_u64(self.issued_txg.0);
+        writer.assert_position(LOGGED_OPERATIONS_KEY_BYTES, "logged_ops 条目的 key");
+        match &self.operation {
+            LoggedOperation::Compaction(intent) => {
+                writer.put_u32(intent.region.device().0);
+                writer.assert_position(14, "整理意图 R 的起始槽");
+                writer.put_six_byte_unsigned(intent.region.start_slot().0);
+                writer.assert_position(20, "整理意图 R 的槽数");
+                writer.put_u16(
+                    u16::try_from(intent.region.slot_count())
+                        .expect("一个聚簇段 64 槽装得进 2 字节"),
+                );
+                writer.assert_position(22, "整理意图的 c0");
+                writer.put_u64(intent.allocation_generation_at_issue.0);
+                writer.assert_position(30, "整理意图的 segs0");
+                writer.put_u64(intent.empty_cluster_segments_at_issue);
+                writer.assert_position(38, "整理意图的补零");
+                writer.skip(COMPACTION_INTENT_PADDING_BYTES);
+            }
+        }
+        writer.assert_position(LOGGED_OPERATIONS_ENTRY_BYTES, "logged_ops 条目");
+        writer.into_bytes()
+    }
+
+    /// logged_ops 条目 44 的读者。
+    ///
+    /// # Errors
+    /// 宽不是 44 ⇒ `WidthIsNotTheFieldTable`；种类码保留 ⇒ `UnknownKind`；整理意图的 R 起始槽不是 64 的倍数 ⇒
+    /// `CompactionRegionRefused`；R 槽数不是 64 ⇒ `CompactionRegionSlotCountIsNotOneClusterSegment`；补零非零 ⇒ `CompactionPaddingNotZero`。
+    pub fn parse(bytes: &[u8]) -> Result<Self, LoggedOperationEntryRefusal> {
+        if bytes.len() != entry_bytes() {
+            return Err(LoggedOperationEntryRefusal::WidthIsNotTheFieldTable {
+                entry_bytes: bytes.len(),
+            });
+        }
+        let mut reader = ByteReader::at(bytes, 0);
+        let kind_code = reader.get_u16();
+        let issued_txg = CheckpointTxg(reader.get_u64());
+        let kind = LoggedOperationKind::of_code(kind_code)
+            .ok_or(LoggedOperationEntryRefusal::UnknownKind { kind_code })?;
+        let operation = match kind {
+            LoggedOperationKind::Compaction => {
+                LoggedOperation::Compaction(parse_compaction_intent(&mut reader)?)
+            }
+        };
+        Ok(Self {
+            issued_txg,
+            operation,
+        })
+    }
+}
+
+/// 整理意图那 34 字节 value（读者已在 key 之后）。
+fn parse_compaction_intent(
+    reader: &mut ByteReader<'_>,
+) -> Result<CompactionIntent, LoggedOperationEntryRefusal> {
+    let device = DeviceIdentity(reader.get_u32());
+    let start_slot = SlotNumber(reader.get_six_byte_unsigned());
+    let slot_count = reader.get_u16();
+    let allocation_generation_at_issue = CheckpointTxg(reader.get_u64());
+    let empty_cluster_segments_at_issue = reader.get_u64();
+    let padding = reader.take(COMPACTION_INTENT_PADDING_BYTES);
+    if u64::from(slot_count) != CLUSTER_SEGMENT_SLOTS {
+        return Err(
+            LoggedOperationEntryRefusal::CompactionRegionSlotCountIsNotOneClusterSegment {
+                slot_count,
+            },
+        );
+    }
+    if padding.iter().any(|byte| *byte != 0) {
+        return Err(LoggedOperationEntryRefusal::CompactionPaddingNotZero);
+    }
+    let region = CompactionRegion::of_one_cluster_segment(device, start_slot)
+        .map_err(LoggedOperationEntryRefusal::CompactionRegionRefused)?;
+    Ok(CompactionIntent {
+        region,
+        allocation_generation_at_issue,
+        empty_cluster_segments_at_issue,
+    })
+}
+
+/// 一个 logged_ops 码 2 根（层级 0）装得下几条：(16384 − 含预留位的头 135) ÷ 44 的整数部分 = 369（判决 G1「一节点 369 条」）。
+#[must_use]
+pub fn logged_operations_one_root_node_holds() -> usize {
+    index_node_entry_capacity(LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES, entry_bytes())
+}
+
+/// logged_ops 根要装的条目：每条 44 字节（[`LoggedOperationEntry::to_bytes`]），按 key 的全序升序。
+/// logged_ops 第一版只有一个码 2 根、根之下不长层：全池活整理意图至多一条（判决 G1 e1），一个根装 369 条。
+///
+/// # Panics
+/// `entries` 不是按 key 严格升序、或多于一个根装得下的：条目由发布路径按「签发加一条、删掉一条」维护，至多一条活整理意图，
+/// 乱序或装不下说明调用方拼错了。
+#[must_use]
+pub fn logged_operations_root_entries(entries: &[LoggedOperationEntry]) -> Vec<Vec<u8>> {
+    assert!(
+        entries
+            .windows(2)
+            .all(|pair| pair[0].key().sort_key() < pair[1].key().sort_key()),
+        "logged_ops 的条目按 key 升序、不重复：发布路径按集合维护它"
+    );
+    assert!(
+        entries.len() <= logged_operations_one_root_node_holds(),
+        "logged_ops 至多一条活整理意图，一个根装得下"
+    );
+    entries.iter().map(LoggedOperationEntry::to_bytes).collect()
+}
+
+/// 从一个 logged_ops 根的条目读回意图（条目是盘上读来的字节，逐条照字段表解）。
+///
+/// # Errors
+/// 有一条过不了 [`LoggedOperationEntry::parse`]（宽不是 44、种类码保留、R 不对齐或槽数不是 64、补零非零）、
+/// 几条不按 key 严格升序、整理意图多于一条（全池至多一条，判决 G1 e1） ⇒ [`LoggedOperationsRootRefusal`]。
+pub fn logged_operation_entries_of_the_root_entries(
+    entries: &[Vec<u8>],
+) -> Result<Vec<LoggedOperationEntry>, LoggedOperationsRootRefusal> {
+    let parsed: Vec<LoggedOperationEntry> = entries
+        .iter()
+        .map(|entry| LoggedOperationEntry::parse(entry))
+        .collect::<Result<_, _>>()
+        .map_err(LoggedOperationsRootRefusal::EntryRefused)?;
+    if !parsed
+        .windows(2)
+        .all(|pair| pair[0].key().sort_key() < pair[1].key().sort_key())
+    {
+        return Err(LoggedOperationsRootRefusal::EntriesNotInStrictlyAscendingKeyOrder);
+    }
+    let compaction_intents = parsed
+        .iter()
+        .filter(|entry| entry.operation.kind() == LoggedOperationKind::Compaction)
+        .count();
+    if compaction_intents > 1 {
+        return Err(LoggedOperationsRootRefusal::MoreThanOneCompactionIntent {
+            compaction_intents,
+        });
+    }
+    Ok(parsed)
+}
+
+/// 读者拒收一个 logged_ops 根的原因：根里的字节是盘上读来的，调用方把它当损坏处置（可写挂载拒）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum LoggedOperationsRootRefusal {
+    EntryRefused(LoggedOperationEntryRefusal),
+    EntriesNotInStrictlyAscendingKeyOrder,
+    MoreThanOneCompactionIntent { compaction_intents: usize },
+}
+
+/// 一版 logged_ops 里那一条活整理意图（全池至多一条，判决 G1 e1）：签发它的那次发布的 txg 与意图本身。没有交回 `None`。
+///
+/// # Panics
+/// 有两条以上的整理意图：发布路径只在没有活意图时签发，盘上读回的条目由 [`logged_operation_entries_of_the_root_entries`] 拒过多于一条的。
+#[must_use]
+pub fn live_compaction_intent_of_the_entries(
+    entries: &[LoggedOperationEntry],
+) -> Option<(CheckpointTxg, CompactionIntent)> {
+    let mut intents = entries.iter().map(|entry| match entry.operation {
+        LoggedOperation::Compaction(intent) => (entry.issued_txg, intent),
+    });
+    let live = intents.next();
+    assert!(
+        intents.next().is_none(),
+        "全池活整理意图至多一条（判决 G1 e1）：发布路径只在没有活意图时签发"
+    );
+    live
+}
diff --git a/crates/singlefs-core/src/region_scan.rs b/crates/singlefs-core/src/region_scan.rs
new file mode 100644
index 00000000..fe1839a2
--- /dev/null
+++ b/crates/singlefs-core/src/region_scan.rs
@@ -0,0 +1,977 @@
+//! 区域扫描（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G9）：对一个整理区域 R 按槽升序看每条未释放的分配记录，
+//! 读单元头反查映射 key、查中央映射核位置，把它分成可搬、钉住、隔离、已死、不一致五类；已释放或没有记录的槽跳过。
+//! 「已死」第一版只报告不动（里程碑五的打包容器那一支不写，只保证这一类在）。
+//!
+//! 它在三层里打头（G2：`scan_region` → `crate::compaction_plan::choose_region` → `crate::compaction_plan::plan_batch`）：
+//! 读单元的那一步交给调用方给的闭包，这里只经 `crate::unit` 的三个 parse 解头，不发一个写、不动分配器。
+//!
+//! 第一版两盘同槽（D2（RAID 条带策略） 已定项 10），每个落点逐盘一条记录、两盘对称（D3（空间分配） 已定项 7；释放前核出对不上时
+//! 两盘一起留在已分配，D19（块指针的结构与宽度预算） 已定项 5）：分类按 R 那块盘上的记录做，钉住与隔离按槽号登记
+//! （R 罩住每块盘上的同号槽，判决 G1 攻方 K-n）。
+
+use std::collections::{BTreeMap, BTreeSet};
+
+use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES};
+
+use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+use crate::allocator::{AllocationRecord, Placement};
+use crate::logged_operations::CompactionRegion;
+use crate::pointer::{LocationEntry, NodePointer, PointerHead};
+use crate::records::{mapping_key_for_data, mapping_key_for_node};
+use crate::unit::{
+    parse_data_unit, parse_index_node, parse_packed_unit, UnitError, UNIT_CLASS_DATA,
+    UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
+};
+
+/// 中央映射里的一条：27 字节 key 加两条位置条目（D19（块指针的结构与宽度预算） 已定项 10）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct CentralMappingEntry {
+    pub key: Vec<u8>,
+    pub locations: [LocationEntry; 2],
+}
+
+impl CentralMappingEntry {
+    /// 这条映射的位置条目里有没有一条指着 `device` 上的 `slot`。
+    #[must_use]
+    pub fn points_at(&self, device: DeviceIdentity, slot: SlotNumber) -> bool {
+        self.locations
+            .iter()
+            .any(|location| location.device == device && location.slot == slot)
+    }
+}
+
+/// 进映射的三类单元（D19（块指针的结构与宽度预算） 已定项 8）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum MappedUnitClass {
+    /// 码 1 数据单元。
+    Data,
+    /// 码 2 索引节点。
+    IndexNode,
+    /// 码 3 打包记录单元。
+    Packed,
+}
+
+impl MappedUnitClass {
+    /// 单元类型标签，也是映射 key 的第一个字节。
+    #[must_use]
+    pub const fn tag(self) -> u8 {
+        match self {
+            MappedUnitClass::Data => UNIT_CLASS_DATA,
+            MappedUnitClass::IndexNode => UNIT_CLASS_INDEX_NODE,
+            MappedUnitClass::Packed => UNIT_CLASS_PACKED,
+        }
+    }
+}
+
+/// 一个可搬的单元：头里反查出的映射 key 在映射里、映射的位置就是这个槽（G9「可搬」）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct MovableUnit {
+    pub mapping_key: Vec<u8>,
+    pub unit_class: MappedUnitClass,
+    /// 单元头里的诞生代号（搬迁不改它，D26（后台整理与放置回收） 已定项 5 第 1 条）。
+    pub birth_txg: CheckpointTxg,
+}
+
+/// 单元头为什么读不出。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum UnitHeaderUnreadable {
+    /// 读单元的闭包交回读不到。
+    DeviceReadFailed,
+    /// 记录的跨度不是 1（码 2）也不是 2（码 1 / 码 3）：三个 parse 都不收这样长的单元，不去读。
+    SpanIsNotAUnitFootprint { span_slots: u16 },
+    /// 读到了，parse 拒收（校验和、类标签、结构字段）。
+    RefusedByTheParser(UnitError),
+}
+
+/// 两个来源（单元头、中央映射）对不上的三种（G9「不一致」）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RegionScanInconsistency {
+    HeaderUnreadable(UnitHeaderUnreadable),
+    /// 指着这个槽的映射条目的 key 打头的类标签与单元头的类不符。
+    UnitClassDisagreesWithTheMappingKey {
+        header_unit_class: MappedUnitClass,
+        mapping_key_class_tag: u8,
+    },
+    /// 头里反查出的 key 在映射里的位置不是这个槽，或指着这个槽的是别的 key。
+    HeaderKeyDisagreesWithTheMappingLocation,
+}
+
+/// 一条未释放的分配记录分进哪一类。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum AllocationInRegion {
+    Movable(MovableUnit),
+    /// 罩住的槽里有钉住的（豁免三类或被根直接指着）：不搬。
+    Pinned,
+    /// 罩住的槽里有在隔离集里的（释放前核出对不上，D19 已定项 5 硬规则 1），不论映射里有没有条目指着它：搬不动，
+    /// 单列给丢意图那一步（G2 攻方 K-c）。
+    Quarantined,
+    /// 映射里没有条目指着它、头里的 key 映射里也没有、不在隔离集里：第一版只报告。
+    Dead,
+    Inconsistent(RegionScanInconsistency),
+}
+
+/// R 里一条未释放的分配记录与它的分类。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct ScannedAllocation {
+    /// 记录的落点（两盘同槽）。
+    pub placement: Placement,
+    pub class: AllocationInRegion,
+}
+
+/// 一次区域扫描的结果：R 里每条未释放的记录一项，按槽升序。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RegionScan {
+    pub region: CompactionRegion,
+    pub allocations: Vec<ScannedAllocation>,
+}
+
+impl RegionScan {
+    /// 可搬的那几项，按槽升序：落点与单元。
+    pub fn movable_units(&self) -> impl Iterator<Item = (Placement, &MovableUnit)> {
+        self.allocations
+            .iter()
+            .filter_map(|allocation| match &allocation.class {
+                AllocationInRegion::Movable(unit) => Some((allocation.placement, unit)),
+                AllocationInRegion::Pinned
+                | AllocationInRegion::Quarantined
+                | AllocationInRegion::Dead
+                | AllocationInRegion::Inconsistent(_) => None,
+            })
+    }
+
+    /// 按槽升序头一对判成可搬、映射 key 相同的记录的起点（先遇到的在前）；可搬的 key 两两不同时交 `None`。
+    #[must_use]
+    pub fn first_two_movable_units_sharing_a_mapping_key(&self) -> Option<[SlotNumber; 2]> {
+        let mut slot_of_each_movable_key: BTreeMap<&[u8], SlotNumber> = BTreeMap::new();
+        for (placement, unit) in self.movable_units() {
+            if let Some(earlier_slot) =
+                slot_of_each_movable_key.insert(unit.mapping_key.as_slice(), placement.slot)
+            {
+                return Some([earlier_slot, placement.slot]);
+            }
+        }
+        None
+    }
+
+    /// 能搬的那几项映射 key 两两不同（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7 的 P2「key 互异」的前提，
+    /// `singlefs_replay_prefix::compaction_plan_proofs::movable_mapping_keys_are_distinct`）。[`scan_region`] 交回 `Ok` 的扫描结果上恒真。
+    #[must_use]
+    pub fn movable_mapping_keys_are_pairwise_distinct(&self) -> bool {
+        self.first_two_movable_units_sharing_a_mapping_key()
+            .is_none()
+    }
+}
+
+/// 扫不了这个 R 的情形。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RegionScanRefusal {
+    /// R 那块盘上有一条未释放的记录跨着 R 的边界（起点在 R 外而罩进 R，或起点在 R 里而罩出 R）。
+    /// 判决 G9 没定这样的记录归哪一类；分配器自己发的落点不跨段（单元区起点在段边界上、两槽单元起点偶数），
+    /// 只有盘上读来的坏记录才有（`recovery::allocation_records_fit_the_pool_geometry` 不判跨度与单元类别，C411）：第一版不支持。
+    RecordStraddlesTheRegionBoundaryUnsupported {
+        device: DeviceIdentity,
+        slot: SlotNumber,
+        span_slots: u16,
+    },
+    /// R 里两条起点不同的记录都判成可搬、单元头反查出的映射 key 相同（`slots` 是头一对的起点，按槽升序）。判「可搬」要指着这一槽的映射条目
+    /// 都带头里那把 key，而一把 key 在映射里只有一条条目：走到这里，那条条目在 R 那块盘上指着两个槽——两条位置项落在同一块盘上，
+    /// D19（块指针的结构与宽度预算） 已定项 5（用户 2026-09-25 定）把它当映射条目损坏、在任何写之前拒绝（发布路径的同一判是
+    /// `transaction::refuse_mapping_entries_that_do_not_name_two_pool_devices`）。读盘重建时只有一部分映射条目经它覆盖的指针按 I-2.5 核过
+    /// 位置项设备升序（`recovery::pointers_of_the_rebuilt_version_ascend_by_device` 列的那几类），留存给快照的那几条
+    /// （`mapping_entries_kept_for_the_snapshots`）不经任何指针、没有经它核，盘上这样一条条目走得到这里（推的，没造过这样的盘）。
+    /// 照那一句拒这个 R，一条都不交：不拒的话同一把 key 进同一批两次，判决 G7 的 P2「一批里的 key 互异」不成立。
+    TwoRecordsMovableUnderOneMappingKey {
+        device: DeviceIdentity,
+        slots: [SlotNumber; 2],
+    },
+}
+
+/// 单元头里反查出来的东西。
+struct UnitHeaderIdentity {
+    unit_class: MappedUnitClass,
+    mapping_key: Vec<u8>,
+    birth_txg: CheckpointTxg,
+}
+
+/// 扫一个 R：`allocation_records` 里 R 那块盘上起点在 R 里、未释放的每条记录分一类，按槽升序交回。
+/// `read_unit_bytes(盘, 槽, 字节数)` 读那个落点上的单元，读不到交 `None`。
+///
+/// 交回 `Ok` 时能搬的那几项映射 key 两两不同（[`RegionScan::movable_mapping_keys_are_pairwise_distinct`]）。
+///
+/// # Errors
+/// R 那块盘上有未释放的记录跨着 R 的边界 ⇒ [`RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported`]，一条都不分类；
+/// 两条记录判成可搬而映射 key 相同 ⇒ [`RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey`]，一条都不交。
+pub fn scan_region<ReadUnitBytes>(
+    region: CompactionRegion,
+    allocation_records: &[AllocationRecord],
+    mapping_entries: &[CentralMappingEntry],
+    pinned_slots: &BTreeSet<SlotNumber>,
+    quarantined_slots: &BTreeSet<SlotNumber>,
+    mut read_unit_bytes: ReadUnitBytes,
+) -> Result<RegionScan, RegionScanRefusal>
+where
+    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
+{
+    let device = region.device();
+    let mut records_in_region: Vec<&AllocationRecord> = Vec::new();
+    for record in allocation_records
+        .iter()
+        .filter(|record| record.device == device && !record.is_released)
+    {
+        let record_end = record.slot.0 + u64::from(record.span_slots);
+        let starts_inside = region.contains_slot(record.slot);
+        let reaches_in_from_below =
+            record.slot.0 < region.start_slot().0 && record_end > region.start_slot().0;
+        let reaches_out_above = starts_inside && record_end > region.end_slot().0;
+        if reaches_in_from_below || reaches_out_above {
+            return Err(
+                RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
+                    device,
+                    slot: record.slot,
+                    span_slots: record.span_slots,
+                },
+            );
+        }
+        if starts_inside {
+            records_in_region.push(record);
+        }
+    }
+    records_in_region.sort_by_key(|record| record.slot);
+    let allocations = records_in_region
+        .into_iter()
+        .map(|record| ScannedAllocation {
+            placement: Placement {
+                slot: record.slot,
+                span: u64::from(record.span_slots),
+            },
+            class: classify_allocation(
+                record,
+                mapping_entries,
+                pinned_slots,
+                quarantined_slots,
+                &mut read_unit_bytes,
+            ),
+        })
+        .collect();
+    let scan = RegionScan {
+        region,
+        allocations,
+    };
+    if let Some(slots) = scan.first_two_movable_units_sharing_a_mapping_key() {
+        return Err(RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey { device, slots });
+    }
+    Ok(scan)
+}
+
+/// 一条未释放记录的分类，次序：钉住 → 隔离 → 读头 → 类标签 → key 与位置（判决 m4-batch2-code-r1 S3）。
+/// 钉住与隔离都看记录罩住的每一个槽，不只看起点槽；隔离不看映射有没有条目指着它。
+fn classify_allocation<ReadUnitBytes>(
+    record: &AllocationRecord,
+    mapping_entries: &[CentralMappingEntry],
+    pinned_slots: &BTreeSet<SlotNumber>,
+    quarantined_slots: &BTreeSet<SlotNumber>,
+    read_unit_bytes: &mut ReadUnitBytes,
+) -> AllocationInRegion
+where
+    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
+{
+    let covered_slots = record.slot..SlotNumber(record.slot.0 + u64::from(record.span_slots));
+    if pinned_slots.range(covered_slots.clone()).next().is_some() {
+        return AllocationInRegion::Pinned;
+    }
+    // 隔离不看映射：隔离 = 释放前核出校验和对不上，映射照旧指着它时搬它就是把坏副本搬走（主 agent 2026-10-13 定，判决 m4-batch2-code-r1 S3）。
+    if quarantined_slots.range(covered_slots).next().is_some() {
+        return AllocationInRegion::Quarantined;
+    }
+    let entries_pointing_here: Vec<&CentralMappingEntry> = mapping_entries
+        .iter()
+        .filter(|entry| entry.points_at(record.device, record.slot))
+        .collect();
+    let header = match read_unit_header(record, read_unit_bytes) {
+        Ok(header) => header,
+        Err(unreadable) => {
+            return AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
+                unreadable,
+            ))
+        }
+    };
+    if let Some(entry) = entries_pointing_here
+        .iter()
+        .find(|entry| entry.key.first() != Some(&header.unit_class.tag()))
+    {
+        return AllocationInRegion::Inconsistent(
+            RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
+                header_unit_class: header.unit_class,
+                mapping_key_class_tag: entry.key.first().copied().unwrap_or_default(),
+            },
+        );
+    }
+    let header_key_is_mapped = mapping_entries
+        .iter()
+        .any(|entry| entry.key == header.mapping_key);
+    if entries_pointing_here.is_empty() && !header_key_is_mapped {
+        return AllocationInRegion::Dead;
+    }
+    let every_entry_pointing_here_carries_the_header_key = entries_pointing_here
+        .iter()
+        .all(|entry| entry.key == header.mapping_key);
+    if !entries_pointing_here.is_empty() && every_entry_pointing_here_carries_the_header_key {
+        return AllocationInRegion::Movable(MovableUnit {
+            mapping_key: header.mapping_key,
+            unit_class: header.unit_class,
+            birth_txg: header.birth_txg,
+        });
+    }
+    AllocationInRegion::Inconsistent(
+        RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
+    )
+}
+
+/// 读一条记录落点上的单元、解头、反查映射 key：跨度 1 当码 2 读 16 KiB，跨度 2 读 32 KiB 先当码 1、类标签是 3 再当码 3。
+/// key 的各段照写者的取法（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 8）：码 1 = 树 ID + 诞生代号 + 写序，
+/// 码 2 = 树 ID + 诞生代号 + 实例代号 + 出生序号，码 3 = 出生树 + 诞生代号 + 写序里的实例代号 + 出生序号。
+fn read_unit_header<ReadUnitBytes>(
+    record: &AllocationRecord,
+    read_unit_bytes: &mut ReadUnitBytes,
+) -> Result<UnitHeaderIdentity, UnitHeaderUnreadable>
+where
+    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
+{
+    let node_pointer_of = |birth_tree, birth_txg, instance, birth_sequence| NodePointer {
+        head: PointerHead {
+            birth_tree,
+            birth_txg,
+        },
+        locations: NodePointer::empty_root().locations,
+        instance,
+        birth_sequence,
+    };
+    match record.span_slots {
+        1 => {
+            let bytes = read_unit_bytes(
+                record.device,
+                record.slot,
+                usize::try_from(NODE_BYTES).expect("16384"),
+            )
+            .ok_or(UnitHeaderUnreadable::DeviceReadFailed)?;
+            let header =
+                parse_index_node(&bytes).map_err(UnitHeaderUnreadable::RefusedByTheParser)?;
+            Ok(UnitHeaderIdentity {
+                unit_class: MappedUnitClass::IndexNode,
+                mapping_key: mapping_key_for_node(
+                    UNIT_CLASS_INDEX_NODE,
+                    node_pointer_of(
+                        header.tree,
+                        header.birth_txg,
+                        header.instance,
+                        header.birth_sequence,
+                    ),
+                ),
+                birth_txg: header.birth_txg,
+            })
+        }
+        2 => {
+            let bytes = read_unit_bytes(
+                record.device,
+                record.slot,
+                usize::try_from(DATA_UNIT_BYTES).expect("32768"),
+            )
+            .ok_or(UnitHeaderUnreadable::DeviceReadFailed)?;
+            match parse_data_unit(&bytes) {
+                Ok(header) => Ok(UnitHeaderIdentity {
+                    unit_class: MappedUnitClass::Data,
+                    mapping_key: mapping_key_for_data(
+                        PointerHead {
+                            birth_tree: header.identity.tree,
+                            birth_txg: header.birth_txg,
+                        },
+                        header.write_order,
+                    ),
+                    birth_txg: header.birth_txg,
+                }),
+                Err(UnitError::WrongUnitClass {
+                    found: UNIT_CLASS_PACKED,
+                    ..
+                }) => {
+                    let header = parse_packed_unit(&bytes)
+                        .map_err(UnitHeaderUnreadable::RefusedByTheParser)?;
+                    Ok(UnitHeaderIdentity {
+                        unit_class: MappedUnitClass::Packed,
+                        mapping_key: mapping_key_for_node(
+                            UNIT_CLASS_PACKED,
+                            node_pointer_of(
+                                header.identity.birth_tree,
+                                header.birth_txg,
+                                header.write_order.instance,
+                                header.birth_sequence,
+                            ),
+                        ),
+                        birth_txg: header.birth_txg,
+                    })
+                }
+                Err(refusal) => Err(UnitHeaderUnreadable::RefusedByTheParser(refusal)),
+            }
+        }
+        span_slots => Err(UnitHeaderUnreadable::SpanIsNotAUnitFootprint { span_slots }),
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use std::collections::BTreeMap;
+
+    use singlefs_format::{TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, UNIT_AREA_START_SLOT};
+
+    use crate::address::{InstanceGeneration, TreeIdentifier};
+    use crate::pointer::BirthSequence;
+    use crate::unit::{
+        build_data_unit, build_index_node, build_packed_unit, DataUnitIdentity, PackedIdentity,
+        WriteOrder, PACKED_TYPE_INODE,
+    };
+
+    const FILESYSTEM_IDENTIFIER: [u8; 16] = [7u8; 16];
+    const BIRTH_TXG: CheckpointTxg = CheckpointTxg(7);
+    const WRITING_INSTANCE: InstanceGeneration = InstanceGeneration(2);
+    const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
+    const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);
+
+    /// 单元区里起点 50240 的那个聚簇段 [50240, 50304)。
+    fn region_under_test() -> CompactionRegion {
+        CompactionRegion::of_one_cluster_segment(DEVICE_ZERO, SlotNumber(UNIT_AREA_START_SLOT + 64))
+            .expect("50240 = 785 × 64")
+    }
+
+    fn slot_in_region(offset_in_slots: u64) -> SlotNumber {
+        SlotNumber(region_under_test().start_slot().0 + offset_in_slots)
+    }
+
+    /// 一块镜像：两块盘上每个落点各一份字节、各一条记录（两盘同槽），加中央映射的条目。
+    #[derive(Default)]
+    struct PoolUnderTest {
+        units: BTreeMap<(DeviceIdentity, SlotNumber), Vec<u8>>,
+        records: Vec<AllocationRecord>,
+        mapping_entries: Vec<CentralMappingEntry>,
+    }
+
+    impl PoolUnderTest {
+        fn place(&mut self, slot: SlotNumber, span_slots: u16, unit: Option<&[u8]>) {
+            for device in [DEVICE_ZERO, DEVICE_ONE] {
+                self.records.push(AllocationRecord {
+                    device,
+                    slot,
+                    span_slots,
+                    generation: BIRTH_TXG,
+                    is_released: false,
+                });
+                if let Some(bytes) = unit {
+                    self.units.insert((device, slot), bytes.to_vec());
+                }
+            }
+        }
+
+        fn release(&mut self, slot: SlotNumber) {
+            for record in self.records.iter_mut().filter(|record| record.slot == slot) {
+                record.is_released = true;
+            }
+        }
+
+        fn map(&mut self, key: Vec<u8>, slot: SlotNumber) {
+            self.mapping_entries.push(CentralMappingEntry {
+                key,
+                locations: locations_at(slot),
+            });
+        }
+
+        fn scan(
+            &self,
+            pinned_slots: &BTreeSet<SlotNumber>,
+            quarantined_slots: &BTreeSet<SlotNumber>,
+        ) -> Result<RegionScan, RegionScanRefusal> {
+            scan_region(
+                region_under_test(),
+                &self.records,
+                &self.mapping_entries,
+                pinned_slots,
+                quarantined_slots,
+                |device, slot, unit_bytes| {
+                    self.units
+                        .get(&(device, slot))
+                        .filter(|bytes| bytes.len() == unit_bytes)
+                        .cloned()
+                },
+            )
+        }
+    }
+
+    fn locations_at(slot: SlotNumber) -> [LocationEntry; 2] {
+        [DEVICE_ZERO, DEVICE_ONE].map(|device| LocationEntry {
+            device,
+            slot,
+            unit_checksum: 0,
+        })
+    }
+
+    /// 一个码 1 数据单元与写者给它的映射 key（`transaction` 里 `mapping_key_for_data` 那一处的取法：出生树 = extent 树、
+    /// 出生 txg = 这次发布、写序 = 实例代号与事务号）。
+    fn data_unit(transaction: u64) -> (Vec<u8>, Vec<u8>) {
+        let write_order = WriteOrder {
+            instance: WRITING_INSTANCE,
+            transaction,
+        };
+        let bytes = build_data_unit(
+            DataUnitIdentity {
+                tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                object: 1,
+                object_birth: CheckpointTxg(3),
+                anchor_offset: 0,
+            },
+            BIRTH_TXG,
+            &FILESYSTEM_IDENTIFIER,
+            write_order,
+            &[0x5A; 100],
+        );
+        let key = mapping_key_for_data(
+            PointerHead {
+                birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                birth_txg: BIRTH_TXG,
+            },
+            write_order,
+        );
+        (bytes, key)
+    }
+
+    fn node_pointer(tree: u64, birth_sequence: u32) -> NodePointer {
+        NodePointer {
+            head: PointerHead {
+                birth_tree: TreeIdentifier(tree),
+                birth_txg: BIRTH_TXG,
+            },
+            locations: NodePointer::empty_root().locations,
+            instance: WRITING_INSTANCE,
+            birth_sequence: BirthSequence(birth_sequence),
+        }
+    }
+
+    /// 一个码 2 节点与写者给它的映射 key（类标签 2、出生树 = 节点的树、出生 txg、实例代号、出生序号）。
+    fn index_node(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
+        let bytes = build_index_node(
+            TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+            0,
+            24,
+            &[0u8; 24],
+            &[0u8; 24],
+            BIRTH_TXG,
+            &FILESYSTEM_IDENTIFIER,
+            WRITING_INSTANCE,
+            BirthSequence(birth_sequence),
+            112,
+            &[],
+        );
+        let key = mapping_key_for_node(
+            UNIT_CLASS_INDEX_NODE,
+            node_pointer(TREE_IDENTIFIER_EXTENT, birth_sequence),
+        );
+        (bytes, key)
+    }
+
+    /// 一个码 3 inode 叶容器与写者给它的映射 key（类标签 3、出生树 = inode 树、出生 txg、实例代号、出生序号）。
+    fn packed_unit(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
+        let bytes = build_packed_unit(
+            PackedIdentity {
+                birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
+                record_type: PACKED_TYPE_INODE,
+                container: 1,
+                container_birth: BIRTH_TXG,
+            },
+            140,
+            &[vec![1u8; 140]],
+            BIRTH_TXG,
+            &FILESYSTEM_IDENTIFIER,
+            WriteOrder {
+                instance: WRITING_INSTANCE,
+                transaction: 9,
+            },
+            BirthSequence(birth_sequence),
+        );
+        let key = mapping_key_for_node(
+            UNIT_CLASS_PACKED,
+            node_pointer(TREE_IDENTIFIER_INODE, birth_sequence),
+        );
+        (bytes, key)
+    }
+
+    fn scanned(offset_in_slots: u64, span: u64, class: AllocationInRegion) -> ScannedAllocation {
+        ScannedAllocation {
+            placement: Placement {
+                slot: slot_in_region(offset_in_slots),
+                span,
+            },
+            class,
+        }
+    }
+
+    fn inconsistent_by_key() -> AllocationInRegion {
+        AllocationInRegion::Inconsistent(
+            RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
+        )
+    }
+
+    /// 判决 G9 的五类在一个 R 里各放至少一个，逐项钉分类、落点与可搬单元的 key / 类 / 诞生代；已释放的记录、R 外的记录、
+    /// 另一块盘上的镜像记录都不出现。
+    #[test]
+    fn every_class_of_the_region_scan_is_pinned_down_slot_by_slot() {
+        let mut pool = PoolUnderTest::default();
+        let (movable_data, movable_data_key) = data_unit(1);
+        pool.place(slot_in_region(0), 2, Some(&movable_data));
+        pool.map(movable_data_key.clone(), slot_in_region(0));
+        let (pinned_node, _pinned_node_key) = index_node(0);
+        pool.place(slot_in_region(2), 1, Some(&pinned_node));
+        let (movable_node, movable_node_key) = index_node(1);
+        pool.place(slot_in_region(3), 1, Some(&movable_node));
+        pool.map(movable_node_key.clone(), slot_in_region(3));
+        let (movable_packed, movable_packed_key) = packed_unit(0);
+        pool.place(slot_in_region(4), 2, Some(&movable_packed));
+        pool.map(movable_packed_key.clone(), slot_in_region(4));
+        let (quarantined_data, _quarantined_key) = data_unit(2);
+        pool.place(slot_in_region(6), 2, Some(&quarantined_data));
+        let (dead_data, _dead_key) = data_unit(3);
+        pool.place(slot_in_region(8), 2, Some(&dead_data));
+        let (mismatched_data, _mismatched_key) = data_unit(4);
+        let (_other_data, other_key) = data_unit(5);
+        pool.place(slot_in_region(10), 2, Some(&mismatched_data));
+        pool.map(other_key, slot_in_region(10));
+        let (released_data, released_key) = data_unit(6);
+        pool.place(slot_in_region(12), 2, Some(&released_data));
+        pool.map(released_key, slot_in_region(12));
+        pool.release(slot_in_region(12));
+        let (outside_data, outside_key) = data_unit(7);
+        pool.place(slot_in_region(64), 2, Some(&outside_data));
+        pool.map(outside_key, slot_in_region(64));
+
+        let scan = pool
+            .scan(
+                &BTreeSet::from([slot_in_region(2)]),
+                &BTreeSet::from([slot_in_region(6)]),
+            )
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(scan.region, region_under_test());
+        assert_eq!(
+            scan.allocations,
+            vec![
+                scanned(
+                    0,
+                    2,
+                    AllocationInRegion::Movable(MovableUnit {
+                        mapping_key: movable_data_key,
+                        unit_class: MappedUnitClass::Data,
+                        birth_txg: BIRTH_TXG,
+                    })
+                ),
+                scanned(2, 1, AllocationInRegion::Pinned),
+                scanned(
+                    3,
+                    1,
+                    AllocationInRegion::Movable(MovableUnit {
+                        mapping_key: movable_node_key,
+                        unit_class: MappedUnitClass::IndexNode,
+                        birth_txg: BIRTH_TXG,
+                    })
+                ),
+                scanned(
+                    4,
+                    2,
+                    AllocationInRegion::Movable(MovableUnit {
+                        mapping_key: movable_packed_key,
+                        unit_class: MappedUnitClass::Packed,
+                        birth_txg: BIRTH_TXG,
+                    })
+                ),
+                scanned(6, 2, AllocationInRegion::Quarantined),
+                scanned(8, 2, AllocationInRegion::Dead),
+                scanned(10, 2, inconsistent_by_key()),
+            ],
+            "槽 +0 / +3 / +4 可搬（码 1 / 码 2 / 码 3），+2 钉住，+6 隔离，+8 已死，+10 头的 key 与指着它的映射条目不同；\
+             +12 已释放、+64 在 R 外，都不出现"
+        );
+        assert_eq!(scan.movable_units().count(), 3, "可搬三个");
+    }
+
+    /// 映射里这个 key 的位置挪到了 R 里另一个槽，原来那个槽上的单元头照旧：没有条目指着它，但头里的 key 在映射里，
+    /// 两个来源对不上，判「不一致」，不判「已死」（G9「头里的 key 与映射里的位置对不上」）。
+    #[test]
+    fn a_unit_whose_mapping_entry_points_at_another_slot_is_inconsistent_not_dead() {
+        let mut pool = PoolUnderTest::default();
+        let (data, key) = data_unit(1);
+        pool.place(slot_in_region(0), 2, Some(&data));
+        pool.map(key, slot_in_region(20));
+
+        let scan = pool
+            .scan(&BTreeSet::new(), &BTreeSet::new())
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(scan.allocations, vec![scanned(0, 2, inconsistent_by_key())]);
+    }
+
+    /// 映射条目指着这个槽，而单元头里反查出的 key 是另一把（头的写序与映射 key 的不同）：判「不一致」，不判可搬。
+    #[test]
+    fn a_unit_whose_header_key_differs_from_the_mapping_entry_pointing_at_it_is_inconsistent() {
+        let mut pool = PoolUnderTest::default();
+        let (_mapped_data, mapped_key) = data_unit(1);
+        let (data_with_another_write_order, _its_own_key) = data_unit(2);
+        pool.place(slot_in_region(0), 2, Some(&data_with_another_write_order));
+        pool.map(mapped_key, slot_in_region(0));
+
+        let scan = pool
+            .scan(&BTreeSet::new(), &BTreeSet::new())
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(scan.allocations, vec![scanned(0, 2, inconsistent_by_key())]);
+    }
+
+    /// 「不一致」的另外两种：头读不出（读不到、头校验和对不上、跨度不是单元的大小）与类标签和映射 key 不符。
+    #[test]
+    fn an_unreadable_header_or_a_class_tag_unlike_the_mapping_key_is_inconsistent() {
+        let mut pool = PoolUnderTest::default();
+        let (_never_written, unwritten_key) = data_unit(1);
+        pool.place(slot_in_region(0), 2, None);
+        pool.map(unwritten_key, slot_in_region(0));
+        let (mut damaged, damaged_key) = data_unit(2);
+        damaged[60] ^= 1;
+        pool.place(slot_in_region(2), 2, Some(&damaged));
+        pool.map(damaged_key, slot_in_region(2));
+        let (data, _data_key) = data_unit(3);
+        let (_node, node_key) = index_node(0);
+        pool.place(slot_in_region(4), 2, Some(&data));
+        pool.map(node_key, slot_in_region(4));
+        let (_three_slot_unit, three_slot_key) = data_unit(4);
+        pool.place(slot_in_region(6), 3, None);
+        pool.map(three_slot_key, slot_in_region(6));
+
+        let scan = pool
+            .scan(&BTreeSet::new(), &BTreeSet::new())
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(
+            scan.allocations,
+            vec![
+                scanned(
+                    0,
+                    2,
+                    AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
+                        UnitHeaderUnreadable::DeviceReadFailed
+                    ))
+                ),
+                scanned(
+                    2,
+                    2,
+                    AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
+                        UnitHeaderUnreadable::RefusedByTheParser(UnitError::HeaderChecksumMismatch)
+                    ))
+                ),
+                scanned(
+                    4,
+                    2,
+                    AllocationInRegion::Inconsistent(
+                        RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
+                            header_unit_class: MappedUnitClass::Data,
+                            mapping_key_class_tag: UNIT_CLASS_INDEX_NODE,
+                        }
+                    )
+                ),
+                scanned(
+                    6,
+                    3,
+                    AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
+                        UnitHeaderUnreadable::SpanIsNotAUnitFootprint { span_slots: 3 }
+                    ))
+                ),
+            ]
+        );
+    }
+
+    /// 隔离槽上的单元头完好、映射条目照旧指着它、key 也对得上：照样判「隔离」，不判可搬（判决 m4-batch2-code-r1 S3：
+    /// 隔离 = 释放前核出校验和对不上，搬它就是把坏副本搬走；隔离排在读头与核映射之前）。
+    #[test]
+    fn a_quarantined_slot_the_mapping_still_points_at_is_quarantined_not_movable() {
+        let mut pool = PoolUnderTest::default();
+        let (data, key) = data_unit(1);
+        pool.place(slot_in_region(0), 2, Some(&data));
+        pool.map(key, slot_in_region(0));
+
+        let scan = pool
+            .scan(&BTreeSet::new(), &BTreeSet::from([slot_in_region(0)]))
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(
+            scan.allocations,
+            vec![scanned(0, 2, AllocationInRegion::Quarantined)],
+            "映射指着隔离槽也判隔离"
+        );
+        assert_eq!(scan.movable_units().count(), 0, "隔离槽不进可搬");
+    }
+
+    /// 两条映射条目指着同一个槽：一条是单元头里反查出的 key，另一条是别的 key（类标签相同）。要每一条都是头里那把 key 才可搬，
+    /// 这里判「不一致」（判决 m4-batch2-code-r1 S3，攻方翻过的 `all` → `any`）。
+    #[test]
+    fn a_slot_two_mapping_entries_point_at_with_only_one_carrying_the_header_key_is_inconsistent() {
+        let mut pool = PoolUnderTest::default();
+        let (data, own_key) = data_unit(1);
+        let (_other_data, other_key) = data_unit(2);
+        pool.place(slot_in_region(0), 2, Some(&data));
+        pool.map(own_key, slot_in_region(0));
+        pool.map(other_key, slot_in_region(0));
+
+        let scan = pool
+            .scan(&BTreeSet::new(), &BTreeSet::new())
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(
+            scan.allocations,
+            vec![scanned(0, 2, inconsistent_by_key())],
+            "另一把 key 也指着这个槽：两个来源对不上"
+        );
+    }
+
+    /// 单元头里的 key 在映射里，映射的两条位置条目槽号与这个槽相同、却在另两块盘上（不是 R 那块盘）：没有条目指着 R 那块盘上的这个槽，
+    /// 判「不一致」，不判可搬（判决 m4-batch2-code-r1 S3，攻方翻过的「位置条目不比盘」）。
+    #[test]
+    fn a_mapping_entry_at_the_same_slot_number_on_other_devices_does_not_point_at_the_region_device(
+    ) {
+        let mut pool = PoolUnderTest::default();
+        let (data, key) = data_unit(1);
+        pool.place(slot_in_region(0), 2, Some(&data));
+        pool.mapping_entries.push(CentralMappingEntry {
+            key,
+            locations: [DeviceIdentity(2), DeviceIdentity(3)].map(|device| LocationEntry {
+                device,
+                slot: slot_in_region(0),
+                unit_checksum: 0,
+            }),
+        });
+
+        let scan = pool
+            .scan(&BTreeSet::new(), &BTreeSet::new())
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(
+            scan.allocations,
+            vec![scanned(0, 2, inconsistent_by_key())],
+            "同槽号、别的盘上的位置条目不算指着这里"
+        );
+    }
+
+    /// 两槽单元的第二个槽钉住（起点槽没钉住）：整条记录判「钉住」，不判可搬（判决 m4-batch2-code-r1 S3，攻方翻过的「钉住只看起点槽」）。
+    #[test]
+    fn a_two_slot_unit_whose_second_slot_is_pinned_is_pinned() {
+        let mut pool = PoolUnderTest::default();
+        let (data, key) = data_unit(1);
+        pool.place(slot_in_region(0), 2, Some(&data));
+        pool.map(key, slot_in_region(0));
+
+        let scan = pool
+            .scan(&BTreeSet::from([slot_in_region(1)]), &BTreeSet::new())
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(
+            scan.allocations,
+            vec![scanned(0, 2, AllocationInRegion::Pinned)],
+            "罩住的第二个槽钉住，整条记录钉住"
+        );
+    }
+
+    /// R 那块盘上一条未释放的记录跨着 R 的边界（从下面罩进来、从里面罩出去）：判决 G9 没定它归哪一类，整次拒、一条都不分类。
+    #[test]
+    fn a_record_straddling_the_region_boundary_is_refused_before_any_classification() {
+        let (inside, inside_key) = data_unit(1);
+        let slot_below_the_region = SlotNumber(region_under_test().start_slot().0 - 1);
+
+        let mut reaching_in = PoolUnderTest::default();
+        reaching_in.place(slot_in_region(0), 2, Some(&inside));
+        reaching_in.map(inside_key.clone(), slot_in_region(0));
+        reaching_in.place(slot_below_the_region, 2, None);
+        assert_eq!(
+            reaching_in.scan(&BTreeSet::new(), &BTreeSet::new()),
+            Err(
+                RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
+                    device: DEVICE_ZERO,
+                    slot: slot_below_the_region,
+                    span_slots: 2,
+                }
+            ),
+            "起点在 R 之下一槽、跨 2：罩进 R"
+        );
+
+        let mut reaching_out = PoolUnderTest::default();
+        reaching_out.place(slot_in_region(0), 2, Some(&inside));
+        reaching_out.map(inside_key, slot_in_region(0));
+        reaching_out.place(slot_in_region(63), 2, None);
+        assert_eq!(
+            reaching_out.scan(&BTreeSet::new(), &BTreeSet::new()),
+            Err(
+                RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
+                    device: DEVICE_ZERO,
+                    slot: slot_in_region(63),
+                    span_slots: 2,
+                }
+            ),
+            "起点在 R 末槽、跨 2：罩出 R"
+        );
+    }
+
+    /// 同一个单元的两份副本（搬迁原样拷，头里反查出的 key 相同）落在 R 的两个槽上。映射条目健康（两条位置项分在两块盘、指同一个槽）时
+    /// 只有它指着的那一份可搬，另一份不一致；映射条目两条位置项都落在 R 那块盘上、各指一份（D19（块指针的结构与宽度预算） 已定项 5 当它损坏）时
+    /// 两份都会判成可搬、key 相同：整次拒、一条都不交（判决 G7 的 P2「一批里的 key 互异」靠它）。
+    #[test]
+    fn two_records_movable_under_one_mapping_key_refuse_the_region() {
+        let (copy_of_the_unit, key) = data_unit(1);
+
+        let mut healthy_mapping = PoolUnderTest::default();
+        healthy_mapping.place(slot_in_region(0), 2, Some(&copy_of_the_unit));
+        healthy_mapping.place(slot_in_region(2), 2, Some(&copy_of_the_unit));
+        healthy_mapping.map(key.clone(), slot_in_region(2));
+        let scan = healthy_mapping
+            .scan(&BTreeSet::new(), &BTreeSet::new())
+            .expect("映射条目健康：一把 key 只判一份可搬");
+        assert!(
+            scan.movable_mapping_keys_are_pairwise_distinct(),
+            "能搬的 key 两两不同"
+        );
+        assert_eq!(
+            scan.allocations
+                .iter()
+                .map(|allocation| allocation.class.clone())
+                .collect::<Vec<_>>(),
+            vec![
+                inconsistent_by_key(),
+                AllocationInRegion::Movable(MovableUnit {
+                    mapping_key: key.clone(),
+                    unit_class: MappedUnitClass::Data,
+                    birth_txg: BIRTH_TXG,
+                }),
+            ],
+            "映射指着的那一份可搬，另一份不一致"
+        );
+
+        let mut damaged_mapping = PoolUnderTest::default();
+        damaged_mapping.place(slot_in_region(0), 2, Some(&copy_of_the_unit));
+        damaged_mapping.place(slot_in_region(2), 2, Some(&copy_of_the_unit));
+        damaged_mapping.mapping_entries.push(CentralMappingEntry {
+            key,
+            locations: [slot_in_region(0), slot_in_region(2)].map(|slot| LocationEntry {
+                device: DEVICE_ZERO,
+                slot,
+                unit_checksum: 0,
+            }),
+        });
+        assert_eq!(
+            damaged_mapping.scan(&BTreeSet::new(), &BTreeSet::new()),
+            Err(RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey {
+                device: DEVICE_ZERO,
+                slots: [slot_in_region(0), slot_in_region(2)],
+            }),
+            "两条位置项都在盘 0、各指一份：两份都会判可搬、key 相同，整次拒"
+        );
+    }
+}
diff --git a/crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs b/crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs
new file mode 100644
index 00000000..31b48638
--- /dev/null
+++ b/crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs
@@ -0,0 +1,1178 @@
+//! 整理的选段与计划的 Verus 核与证明（里程碑四二·4；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7 的 V-1：
+//! 放进这个 crate 的一个模块）。`singlefs-core` 的 `compaction_plan::choose_region` 与 `compaction_plan::plan_batch` 去掉读记录、
+//! 读扫描结果之后剩下的判定写成两个可执行的核（[`choose_compaction_segment`]、[`plan_compaction_batch`]），证判决 G7 点名的四条：
+//! - P1 选段健全（先不证最优）：选出的段合 G2 的资格——不是这次挂载开过的段、不是活意图的 R、没装钉住的槽、没被区域扫描拒过、
+//!   扫过而且至少一个能搬的单元；「一段都没资格」时真没有一段是候选。
+//! - P2 计划健全：一批至多 K 项、每一项是能搬的单元、各项互异；扫描结果里能搬的单元映射 key 互异时，一批里的 key 互异。
+//!   目的地那一半（不在 R 内、两两不重叠、跨度相同）按判决 G2 是分配器的性质，计划函数不交目的地，这里只证 key 一侧。
+//! - P3 进展：交「搬」时至少一项；不搬时分「搬空」（R 里一条未释放记录都没有）与「搬不动」（有记录、一个能搬的都没有）。
+//! - P4 抽象恒等引理：映射（key → 落点）加落点上的字节是具体状态，每个 key 经映射读到的字节是抽象状态。按计划搬一批（源字节原样写到
+//!   目的地、映射条目换到目的地）不改任何一个 key 读到的字节；在前提「R 内每条未释放记录都有映射 key 且不在隔离集里」（攻方 H-G7-1）下，
+//!   一批批搬到 R 空也不改。
+//!
+//! 核心层不调这两个核（`crates/mutations.tsv` 里有十二行变异锚在 `compaction_plan` 现在的写法上，换成调核那几行原文就没了）：
+//! `singlefs-core` 的 `compaction_plan` 单测在小输入上把它的 `choose_region`、`plan_batch` 与这两个核逐项对拍。
+//! 执行层（发布路径是不是把字节原样写到目的地、写序、崩溃）不在 Verus 里，靠 N6、模型对拍与必出反例的世界（判决 G7）。
+//! 必红世界与 `src/lib.rs` 的同登记在 `verus-must-red-worlds.tsv`。
+
+use vstd::pervasive::unreached;
+use vstd::prelude::*;
+
+verus! {
+
+// ===== 选段核的数据 =====
+// 单元区里一个整段在选段那一刻的样子：胶水（singlefs-core 的 compaction_plan::choose_region 那一截）按分配记录、
+// 这次挂载开过的段、活意图的 R、钉住的槽、被拒的段与扫描结果逐段算好，核只判。入参按段起点槽号升序排，下标就是段在单元区里的序。
+// 扫过的段分两种：扫描结果里至少一个能搬的单元，与一个能搬的都没有。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionKernelSegmentScan {
+    NotScanned,
+    ScannedWithAMovableUnit,
+    ScannedWithoutAMovableUnit,
+}
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CompactionKernelSegment {
+    // greedy 的打分就是它（singlefs-core 的 compaction_plan::greedy_score）：段里未释放记录罩住的槽数。
+    pub live_slots: u64,
+    pub is_opened_in_this_mount: bool,
+    pub is_the_live_intent_region: bool,
+    pub holds_a_pinned_slot: bool,
+    pub is_refused_by_the_region_scan: bool,
+    pub scan: CompactionKernelSegmentScan,
+}
+
+// 交回的下标指入参里的段。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionKernelRegionChoice {
+    Chosen(usize),
+    ScanBeforeChoosing(usize),
+    NoEligibleSegment,
+}
+
+// ===== 选段的 spec（可信：照判决 G2 的资格写）=====
+// 候选：资格里除「扫过、至少一个能搬的单元」之外的四条都成立，而且不是「扫过而一个能搬的都没有」（没扫过的还不知道，留作候选）。
+pub open spec fn segment_is_a_candidate(segment: CompactionKernelSegment) -> bool {
+    &&& !segment.is_opened_in_this_mount
+    &&& !segment.is_the_live_intent_region
+    &&& !segment.holds_a_pinned_slot
+    &&& !segment.is_refused_by_the_region_scan
+    &&& !(segment.scan is ScannedWithoutAMovableUnit)
+}
+
+// P1：选中的段合资格五条；要先扫的段是候选、还没扫过；「一段都没资格」时没有一段是候选。
+pub open spec fn region_choice_is_sound(
+    segments: Seq<CompactionKernelSegment>,
+    choice: CompactionKernelRegionChoice,
+) -> bool {
+    match choice {
+        CompactionKernelRegionChoice::Chosen(chosen) => {
+            &&& chosen < segments.len()
+            &&& segment_is_a_candidate(segments[chosen as int])
+            &&& segments[chosen as int].scan is ScannedWithAMovableUnit
+        },
+        CompactionKernelRegionChoice::ScanBeforeChoosing(to_scan) => {
+            &&& to_scan < segments.len()
+            &&& segment_is_a_candidate(segments[to_scan as int])
+            &&& segments[to_scan as int].scan is NotScanned
+        },
+        CompactionKernelRegionChoice::NoEligibleSegment => forall|position: int|
+            0 <= position < segments.len() ==> !segment_is_a_candidate(#[trigger] segments[position]),
+    }
+}
+
+// singlefs-core 的 compaction_plan::choose_region 按记录与集合算完每段之后的那一截：候选里活槽数最少的先，同分取起点槽号最小的
+// （入参里最前的）；排在最前的那一段扫过就选它，没扫过就要调用方先扫。
+pub fn choose_compaction_segment(segments: &[CompactionKernelSegment]) -> (choice:
+    CompactionKernelRegionChoice)
+    ensures
+        region_choice_is_sound(segments@, choice),
+{
+    let mut best: Option<usize> = None;
+    let mut best_live_slots: u64 = 0;
+    let mut position: usize = 0;
+    while position < segments.len()
+        invariant
+            position <= segments.len(),
+            best matches Some(best_position) ==> best_position < position
+                && segment_is_a_candidate(segments@[best_position as int]),
+            best is None ==> forall|earlier: int|
+                0 <= earlier < position ==> !segment_is_a_candidate(#[trigger] segments@[earlier]),
+        decreases segments.len() - position,
+    {
+        let segment = segments[position];
+        let is_scanned_without_a_movable_unit = match segment.scan {
+            CompactionKernelSegmentScan::ScannedWithoutAMovableUnit => true,
+            CompactionKernelSegmentScan::NotScanned
+            | CompactionKernelSegmentScan::ScannedWithAMovableUnit => false,
+        };
+        let is_a_candidate = !segment.is_opened_in_this_mount
+            && !segment.is_the_live_intent_region
+            && !segment.holds_a_pinned_slot
+            && !segment.is_refused_by_the_region_scan
+            && !is_scanned_without_a_movable_unit;
+        if is_a_candidate {
+            let scores_lower_than_the_best = match best {
+                None => true,
+                Some(_) => segment.live_slots < best_live_slots,
+            };
+            if scores_lower_than_the_best {
+                best = Some(position);
+                best_live_slots = segment.live_slots;
+            }
+        }
+        position += 1;
+    }
+    match best {
+        None => CompactionKernelRegionChoice::NoEligibleSegment,
+        Some(best_position) => match segments[best_position].scan {
+            CompactionKernelSegmentScan::ScannedWithAMovableUnit => CompactionKernelRegionChoice::Chosen(
+                best_position,
+            ),
+            CompactionKernelSegmentScan::NotScanned => CompactionKernelRegionChoice::ScanBeforeChoosing(
+                best_position,
+            ),
+            // 循环不变式：留下的那一段是候选，候选不是「扫过而一个能搬的都没有」（Verus 证这一臂走不到）。
+            CompactionKernelSegmentScan::ScannedWithoutAMovableUnit => unreached(),
+        },
+    }
+}
+
+// ===== 计划核的数据 =====
+// 区域扫描（singlefs-core 的 region_scan::scan_region）对 R 里一条未释放记录的分类。映射 key 的身份：胶水给每一把不同的 key
+// 一个不同的数、同一把 key 同一个数，核只比相等。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionKernelAllocationClass {
+    Movable { mapping_key_identity: u64 },
+    Pinned,
+    Quarantined,
+    Dead,
+    Inconsistent,
+}
+
+// R 里一条未释放记录：源落点的起点槽号与它的分类。入参按槽升序（区域扫描交回的次序）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CompactionKernelScannedAllocation {
+    pub source_slot: u64,
+    pub class: CompactionKernelAllocationClass,
+}
+
+// 「搬」交的是要搬的那几项在入参里的下标，升序。
+pub enum CompactionKernelPlannedBatch {
+    Relocate(Vec<usize>),
+    RegionEmptied,
+    NothingMovableButTheRegionIsNotEmpty,
+}
+
+// ===== 计划的 spec（可信：照判决 G2 与 G7 的 P2、P3 写）=====
+pub open spec fn allocation_is_movable(allocation: CompactionKernelScannedAllocation) -> bool {
+    allocation.class is Movable
+}
+
+// 不能搬的几类没有映射 key；这里给 0 只为让函数是全的，用到它的地方都先判过能搬。
+pub open spec fn mapping_key_identity_of(allocation: CompactionKernelScannedAllocation) -> u64 {
+    match allocation.class {
+        CompactionKernelAllocationClass::Movable { mapping_key_identity } => mapping_key_identity,
+        CompactionKernelAllocationClass::Pinned
+        | CompactionKernelAllocationClass::Quarantined
+        | CompactionKernelAllocationClass::Dead
+        | CompactionKernelAllocationClass::Inconsistent => 0,
+    }
+}
+
+// 入参 [0, end) 里能搬的那几项的下标，升序。
+pub open spec fn movable_positions_before(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    end: int,
+) -> Seq<int>
+    decreases end,
+{
+    if end <= 0 {
+        Seq::empty()
+    } else {
+        let earlier = movable_positions_before(allocations, end - 1);
+        if allocation_is_movable(allocations[end - 1]) {
+            earlier.push(end - 1)
+        } else {
+            earlier
+        }
+    }
+}
+
+pub open spec fn movable_positions(allocations: Seq<CompactionKernelScannedAllocation>) -> Seq<
+    int,
+> {
+    movable_positions_before(allocations, allocations.len() as int)
+}
+
+// 区域扫描判「可搬」时，头里的 key 在映射里、映射的位置就是这一槽；映射里同一把 key 只有一个位置，所以这一条在健康的映射上成立。
+// 计划核不判它：坏盘上它不成立时计划怎么办，条款没写（报告交主 agent），这里只在它成立时证 key 互异。
+pub open spec fn movable_mapping_keys_are_distinct(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+) -> bool {
+    forall|one: int, other: int|
+        #![trigger allocations[one], allocations[other]]
+        0 <= one < allocations.len() && 0 <= other < allocations.len() && one != other
+            && allocation_is_movable(allocations[one]) && allocation_is_movable(allocations[other])
+            ==> mapping_key_identity_of(allocations[one]) != mapping_key_identity_of(
+            allocations[other],
+        )
+}
+
+pub open spec fn positions_as_ints(positions: Seq<usize>) -> Seq<int> {
+    Seq::new(positions.len(), |index: int| positions[index] as int)
+}
+
+pub open spec fn planned_positions(planned: CompactionKernelPlannedBatch) -> Seq<int> {
+    match planned {
+        CompactionKernelPlannedBatch::Relocate(positions) => positions_as_ints(positions@),
+        CompactionKernelPlannedBatch::RegionEmptied
+        | CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty => Seq::empty(),
+    }
+}
+
+// 一批搬几个：能搬的不到 K 个就全搬，否则 K 个。
+pub open spec fn units_in_the_batch(movable_units: int, batch_units: int) -> int {
+    if movable_units < batch_units {
+        movable_units
+    } else {
+        batch_units
+    }
+}
+
+// P2：至多 K 项；每一项是入参里能搬的一项；下标严格升序（各项互异）；能搬的单元映射 key 互异时一批里的 key 互异。
+pub open spec fn planned_batch_is_sound(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    batch_units: usize,
+    planned: CompactionKernelPlannedBatch,
+) -> bool {
+    let positions = planned_positions(planned);
+    &&& positions.len() <= batch_units
+    &&& forall|index: int|
+        0 <= index < positions.len() ==> 0 <= #[trigger] positions[index] < allocations.len()
+            && allocation_is_movable(allocations[positions[index]])
+    &&& forall|earlier: int, later: int|
+        #![trigger positions[earlier], positions[later]]
+        0 <= earlier < later < positions.len() ==> positions[earlier] < positions[later]
+    &&& movable_mapping_keys_are_distinct(allocations) ==> forall|earlier: int, later: int|
+        #![trigger positions[earlier], positions[later]]
+        0 <= earlier < later < positions.len() ==> mapping_key_identity_of(
+            allocations[positions[earlier]],
+        ) != mapping_key_identity_of(allocations[positions[later]])
+}
+
+// P3：「搬」至少一项；「搬空」时 R 里一条未释放记录都没有；「搬不动」时有记录、一个能搬的都没有。
+// 合起来：R 里有能搬的单元，计划就是「搬」。
+pub open spec fn planned_batch_makes_progress(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    planned: CompactionKernelPlannedBatch,
+) -> bool {
+    match planned {
+        CompactionKernelPlannedBatch::Relocate(positions) => positions@.len() >= 1,
+        CompactionKernelPlannedBatch::RegionEmptied => allocations.len() == 0,
+        CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty => {
+            &&& allocations.len() > 0
+            &&& forall|position: int|
+                0 <= position < allocations.len() ==> !allocation_is_movable(
+                    #[trigger] allocations[position],
+                )
+        },
+    }
+}
+
+// 判决 G2 的 plan_batch：能搬的单元按槽升序取前 K 个（各批互不相交、并起来是 R 里能搬的全部，靠的就是每批都取最前的那几个）。
+pub open spec fn planned_batch_takes_the_first_movable_units(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    batch_units: usize,
+    planned: CompactionKernelPlannedBatch,
+) -> bool {
+    let movable = movable_positions(allocations);
+    planned_positions(planned) == movable.take(
+        units_in_the_batch(movable.len() as int, batch_units as int),
+    )
+}
+
+// ===== 计划的引理（不可信件：写错了只会证不出来）=====
+proof fn lemma_movable_positions_are_the_movable_allocations_in_order(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    end: int,
+)
+    requires
+        0 <= end <= allocations.len(),
+    ensures
+        ({
+            let movable = movable_positions_before(allocations, end);
+            &&& movable.len() <= end
+            &&& forall|index: int|
+                0 <= index < movable.len() ==> 0 <= #[trigger] movable[index] < end
+                    && allocation_is_movable(allocations[movable[index]])
+            &&& forall|earlier: int, later: int|
+                #![trigger movable[earlier], movable[later]]
+                0 <= earlier < later < movable.len() ==> movable[earlier] < movable[later]
+            &&& forall|position: int|
+                0 <= position < end && allocation_is_movable(#[trigger] allocations[position])
+                    ==> exists|index: int| 0 <= index < movable.len() && movable[index] == position
+        }),
+    decreases end,
+{
+    if end > 0 {
+        lemma_movable_positions_are_the_movable_allocations_in_order(allocations, end - 1);
+        let earlier = movable_positions_before(allocations, end - 1);
+        let movable = movable_positions_before(allocations, end);
+        if allocation_is_movable(allocations[end - 1]) {
+            assert(movable == earlier.push(end - 1));
+            assert forall|position: int|
+                0 <= position < end && allocation_is_movable(
+                    #[trigger] allocations[position],
+                ) implies exists|index: int|
+                0 <= index < movable.len() && movable[index] == position by {
+                if position == end - 1 {
+                    assert(movable[earlier.len() as int] == position);
+                } else {
+                    let index = choose|index: int|
+                        0 <= index < earlier.len() && earlier[index] == position;
+                    assert(movable[index] == position);
+                }
+            }
+        } else {
+            assert(movable == earlier);
+        }
+    }
+}
+
+proof fn lemma_every_position_is_movable_when_every_allocation_is(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    end: int,
+)
+    requires
+        0 <= end <= allocations.len(),
+        forall|position: int|
+            0 <= position < allocations.len() ==> allocation_is_movable(
+                #[trigger] allocations[position],
+            ),
+    ensures
+        movable_positions_before(allocations, end) == Seq::new(end as nat, |index: int| index),
+    decreases end,
+{
+    if end > 0 {
+        lemma_every_position_is_movable_when_every_allocation_is(allocations, end - 1);
+        assert(movable_positions_before(allocations, end) =~= Seq::new(
+            end as nat,
+            |index: int| index,
+        ));
+    } else {
+        assert(movable_positions_before(allocations, end) =~= Seq::new(
+            end as nat,
+            |index: int| index,
+        ));
+    }
+}
+
+// ===== 计划核 =====
+// singlefs-core 的 compaction_plan::plan_batch：扫描结果里能搬的按槽升序取前 batch_units 个；一个都没取到时，R 里没有未释放记录
+// 交「搬空」，还有交「搬不动」。
+pub fn plan_compaction_batch(
+    allocations: &[CompactionKernelScannedAllocation],
+    batch_units: usize,
+) -> (planned: CompactionKernelPlannedBatch)
+    requires
+        batch_units >= 1,
+    ensures
+        planned_batch_is_sound(allocations@, batch_units, planned),
+        planned_batch_makes_progress(allocations@, planned),
+        planned_batch_takes_the_first_movable_units(allocations@, batch_units, planned),
+{
+    let mut positions: Vec<usize> = Vec::new();
+    let mut position: usize = 0;
+    while position < allocations.len()
+        invariant
+            position <= allocations.len(),
+            batch_units >= 1,
+            positions_as_ints(positions@) == movable_positions_before(
+                allocations@,
+                position as int,
+            ).take(
+                units_in_the_batch(
+                    movable_positions_before(allocations@, position as int).len() as int,
+                    batch_units as int,
+                ),
+            ),
+        decreases allocations.len() - position,
+    {
+        let ghost movable_before = movable_positions_before(allocations@, position as int);
+        let ghost positions_before = positions@;
+        proof {
+            lemma_movable_positions_are_the_movable_allocations_in_order(
+                allocations@,
+                position as int,
+            );
+        }
+        let is_movable = match allocations[position].class {
+            CompactionKernelAllocationClass::Movable { .. } => true,
+            CompactionKernelAllocationClass::Pinned
+            | CompactionKernelAllocationClass::Quarantined
+            | CompactionKernelAllocationClass::Dead
+            | CompactionKernelAllocationClass::Inconsistent => false,
+        };
+        if is_movable && positions.len() < batch_units {
+            positions.push(position);
+        }
+        proof {
+            let movable_after = movable_positions_before(allocations@, position as int + 1);
+            if is_movable {
+                assert(movable_after == movable_before.push(position as int));
+                if movable_before.len() < batch_units {
+                    assert(movable_before.take(movable_before.len() as int) =~= movable_before);
+                    assert(positions@ == positions_before.push(position));
+                    assert(movable_after.take(movable_after.len() as int) =~= movable_after);
+                    assert(positions_as_ints(positions@) =~= movable_after.take(
+                        units_in_the_batch(movable_after.len() as int, batch_units as int),
+                    ));
+                } else {
+                    assert(positions@ == positions_before);
+                    assert(movable_after.take(batch_units as int) =~= movable_before.take(
+                        batch_units as int,
+                    ));
+                }
+            } else {
+                assert(movable_after == movable_before);
+            }
+        }
+        position += 1;
+    }
+    proof {
+        lemma_movable_positions_are_the_movable_allocations_in_order(
+            allocations@,
+            allocations@.len() as int,
+        );
+        assert(allocations@.subrange(0, allocations@.len() as int) =~= allocations@);
+    }
+    if !positions.is_empty() {
+        return CompactionKernelPlannedBatch::Relocate(positions);
+    }
+    proof {
+        assert(movable_positions(allocations@).len() == 0);
+        assert(movable_positions(allocations@).take(0) =~= Seq::<int>::empty());
+    }
+    if allocations.is_empty() {
+        CompactionKernelPlannedBatch::RegionEmptied
+    } else {
+        CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty
+    }
+}
+
+// ===== P4：抽象恒等引理（执行层的模型，不是执行层）=====
+// 具体状态：映射（key 的身份 → 落点起点槽号），加上每个槽号上的字节。抽象状态：每个 key 经映射读到的字节。
+pub open spec fn bytes_read_through_the_mapping(
+    mapping: Map<u64, u64>,
+    storage: spec_fn(u64) -> Seq<u8>,
+    mapping_key_identity: u64,
+) -> Seq<u8> {
+    storage(mapping[mapping_key_identity])
+}
+
+// 搬一个单元的写：源落点上的字节原样写到目的落点（判决 G3：头不盖戳、birth 不变），别的槽不动。
+pub open spec fn storage_after_copying(
+    storage: spec_fn(u64) -> Seq<u8>,
+    source_slot: u64,
+    destination_slot: u64,
+) -> spec_fn(u64) -> Seq<u8> {
+    |slot: u64|
+        if slot == destination_slot {
+            storage(source_slot)
+        } else {
+            storage(slot)
+        }
+}
+
+// 按次序搬前 count 项：第 j 项把 keys[j] 的映射条目换到 destinations[j]。
+pub open spec fn mapping_after_relocating(
+    mapping: Map<u64, u64>,
+    keys: Seq<u64>,
+    destinations: Seq<u64>,
+    count: int,
+) -> Map<u64, u64>
+    decreases count,
+{
+    if count <= 0 {
+        mapping
+    } else {
+        mapping_after_relocating(mapping, keys, destinations, count - 1).insert(
+            keys[count - 1],
+            destinations[count - 1],
+        )
+    }
+}
+
+// 按次序搬前 count 项：第 j 项把 sources[j] 上的字节写到 destinations[j]。
+pub open spec fn storage_after_relocating(
+    storage: spec_fn(u64) -> Seq<u8>,
+    sources: Seq<u64>,
+    destinations: Seq<u64>,
+    count: int,
+) -> spec_fn(u64) -> Seq<u8>
+    decreases count,
+{
+    if count <= 0 {
+        storage
+    } else {
+        storage_after_copying(
+            storage_after_relocating(storage, sources, destinations, count - 1),
+            sources[count - 1],
+            destinations[count - 1],
+        )
+    }
+}
+
+// 目的地是分配器的性质（判决 G2）：分配器交的落点此刻没被映射里任何一个 key 占着（「目的地不在 R 内」是它的一个特例：R 里能搬的单元
+// 都在映射里）。
+pub open spec fn destination_is_free_in_the_mapping(mapping: Map<u64, u64>, destination: u64) -> bool {
+    forall|mapping_key_identity: u64|
+        #[trigger] mapping.contains_key(mapping_key_identity) ==> mapping[mapping_key_identity]
+            != destination
+}
+
+// 前 count 个目的地两两不重叠、都不被映射占着。
+pub open spec fn destinations_are_distinct_and_free(
+    mapping: Map<u64, u64>,
+    destinations: Seq<u64>,
+    count: int,
+) -> bool {
+    &&& count <= destinations.len()
+    &&& forall|earlier: int, later: int|
+        #![trigger destinations[earlier], destinations[later]]
+        0 <= earlier < later < count ==> destinations[earlier] != destinations[later]
+    &&& forall|index: int|
+        0 <= index < count ==> destination_is_free_in_the_mapping(
+            mapping,
+            #[trigger] destinations[index],
+        )
+}
+
+// 区域扫描的「可搬」：key 在映射里、映射的位置就是这一项的源落点。
+pub open spec fn scan_agrees_with_the_mapping(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    mapping: Map<u64, u64>,
+) -> bool {
+    forall|position: int|
+        0 <= position < allocations.len() && allocation_is_movable(#[trigger] allocations[position])
+            ==> mapping.contains_key(mapping_key_identity_of(allocations[position]))
+            && mapping[mapping_key_identity_of(allocations[position])]
+            == allocations[position].source_slot
+}
+
+pub open spec fn planned_mapping_keys(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    positions: Seq<int>,
+) -> Seq<u64> {
+    positions.map_values(|position: int| mapping_key_identity_of(allocations[position]))
+}
+
+pub open spec fn planned_sources(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    positions: Seq<int>,
+) -> Seq<u64> {
+    positions.map_values(|position: int| allocations[position].source_slot)
+}
+
+proof fn lemma_relocating_in_order_keeps_every_read(
+    mapping: Map<u64, u64>,
+    storage: spec_fn(u64) -> Seq<u8>,
+    keys: Seq<u64>,
+    sources: Seq<u64>,
+    destinations: Seq<u64>,
+    count: int,
+)
+    requires
+        0 <= count <= keys.len(),
+        count <= sources.len(),
+        forall|earlier: int, later: int|
+            #![trigger keys[earlier], keys[later]]
+            0 <= earlier < later < count ==> keys[earlier] != keys[later],
+        forall|index: int|
+            0 <= index < count ==> mapping.contains_key(#[trigger] keys[index]) && mapping[keys[index]]
+                == sources[index],
+        destinations_are_distinct_and_free(mapping, destinations, count),
+    ensures
+        ({
+            let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
+            let moved_storage = storage_after_relocating(storage, sources, destinations, count);
+            &&& moved_mapping.dom() == mapping.dom()
+            &&& forall|mapping_key_identity: u64|
+                #[trigger] mapping.contains_key(mapping_key_identity) ==> bytes_read_through_the_mapping(
+                    moved_mapping,
+                    moved_storage,
+                    mapping_key_identity,
+                ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity)
+            &&& forall|index: int|
+                0 <= index < count ==> moved_mapping[#[trigger] keys[index]] == destinations[index]
+            &&& forall|mapping_key_identity: u64|
+                #[trigger] mapping.contains_key(mapping_key_identity) && (forall|index: int|
+                    0 <= index < count ==> keys[index] != mapping_key_identity)
+                    ==> moved_mapping[mapping_key_identity] == mapping[mapping_key_identity]
+            &&& forall|mapping_key_identity: u64|
+                #[trigger] moved_mapping.contains_key(mapping_key_identity) ==> moved_mapping[mapping_key_identity]
+                    == mapping[mapping_key_identity] || exists|index: int|
+                    0 <= index < count && moved_mapping[mapping_key_identity] == destinations[index]
+            &&& forall|slot: u64|
+                (forall|index: int| 0 <= index < count ==> destinations[index] != slot) ==> #[trigger] moved_storage(slot)
+                    == storage(slot)
+        }),
+    decreases count,
+{
+    if count > 0 {
+        let previous_count = count - 1;
+        lemma_relocating_in_order_keeps_every_read(
+            mapping,
+            storage,
+            keys,
+            sources,
+            destinations,
+            previous_count,
+        );
+        let previous_mapping = mapping_after_relocating(mapping, keys, destinations, previous_count);
+        let previous_storage = storage_after_relocating(storage, sources, destinations, previous_count);
+        let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
+        let moved_storage = storage_after_relocating(storage, sources, destinations, count);
+        let moved_key = keys[previous_count];
+        let source = sources[previous_count];
+        let destination = destinations[previous_count];
+        assert(moved_mapping == previous_mapping.insert(moved_key, destination));
+        assert(moved_storage == storage_after_copying(previous_storage, source, destination));
+        assert(mapping.contains_key(moved_key) && mapping[moved_key] == source);
+        assert forall|index: int| 0 <= index < previous_count implies keys[index] != moved_key by {
+            assert(keys[index] != keys[previous_count]);
+        }
+        assert(previous_mapping[moved_key] == mapping[moved_key]);
+        assert forall|index: int| 0 <= index < previous_count implies destinations[index] != source by {
+            assert(destination_is_free_in_the_mapping(mapping, destinations[index]));
+        }
+        assert(previous_storage(source) == storage(source));
+        assert(destination_is_free_in_the_mapping(mapping, destination));
+        assert(moved_mapping.dom() =~= mapping.dom());
+        assert forall|mapping_key_identity: u64| #[trigger]
+            moved_mapping.contains_key(mapping_key_identity) implies moved_mapping[mapping_key_identity]
+            == mapping[mapping_key_identity] || exists|index: int|
+            0 <= index < count && moved_mapping[mapping_key_identity] == destinations[index] by {
+            if mapping_key_identity == moved_key {
+                assert(moved_mapping[mapping_key_identity] == destinations[previous_count]);
+            } else {
+                assert(previous_mapping.contains_key(mapping_key_identity));
+            }
+        }
+        assert forall|mapping_key_identity: u64| #[trigger]
+            mapping.contains_key(mapping_key_identity) implies bytes_read_through_the_mapping(
+            moved_mapping,
+            moved_storage,
+            mapping_key_identity,
+        ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity) by {
+            if mapping_key_identity == moved_key {
+                assert(moved_mapping[mapping_key_identity] == destination);
+                assert(moved_storage(destination) == previous_storage(source));
+            } else {
+                assert(previous_mapping.contains_key(mapping_key_identity));
+                let previous_slot = previous_mapping[mapping_key_identity];
+                assert(moved_mapping[mapping_key_identity] == previous_slot);
+                if previous_slot == mapping[mapping_key_identity] {
+                    assert(previous_slot != destination);
+                } else {
+                    let index = choose|index: int|
+                        0 <= index < previous_count && previous_slot == destinations[index];
+                    assert(destinations[index] != destinations[previous_count]);
+                }
+                assert(moved_storage(previous_slot) == previous_storage(previous_slot));
+                assert(bytes_read_through_the_mapping(
+                    previous_mapping,
+                    previous_storage,
+                    mapping_key_identity,
+                ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity));
+            }
+        }
+        assert forall|index: int| 0 <= index < count implies moved_mapping[#[trigger] keys[index]]
+            == destinations[index] by {
+            if index < previous_count {
+                assert(keys[index] != keys[previous_count]);
+            }
+        }
+        assert forall|slot: u64|
+            (forall|index: int| 0 <= index < count ==> destinations[index] != slot) implies #[trigger] moved_storage(
+            slot,
+        ) == storage(slot) by {
+            assert(destinations[previous_count] != slot);
+            assert forall|index: int| 0 <= index < previous_count implies destinations[index] != slot by {
+                assert(0 <= index < count);
+            }
+        }
+    }
+}
+
+// P4，一批：计划合 P2（key 互异来自扫描结果里能搬的 key 互异）、扫描的「可搬」与映射对得上、目的地两两不重叠且没被映射占着
+// （分配器的性质），按计划搬完这一批，每个 key 读到的字节不变，映射里的 key 一个不多一个不少。
+pub proof fn lemma_a_planned_batch_keeps_every_read(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    batch_units: usize,
+    planned: CompactionKernelPlannedBatch,
+    mapping: Map<u64, u64>,
+    storage: spec_fn(u64) -> Seq<u8>,
+    destinations: Seq<u64>,
+)
+    requires
+        planned_batch_is_sound(allocations, batch_units, planned),
+        movable_mapping_keys_are_distinct(allocations),
+        scan_agrees_with_the_mapping(allocations, mapping),
+        destinations_are_distinct_and_free(
+            mapping,
+            destinations,
+            planned_positions(planned).len() as int,
+        ),
+    ensures
+        ({
+            let positions = planned_positions(planned);
+            let moved_mapping = mapping_after_relocating(
+                mapping,
+                planned_mapping_keys(allocations, positions),
+                destinations,
+                positions.len() as int,
+            );
+            let moved_storage = storage_after_relocating(
+                storage,
+                planned_sources(allocations, positions),
+                destinations,
+                positions.len() as int,
+            );
+            &&& moved_mapping.dom() == mapping.dom()
+            &&& forall|mapping_key_identity: u64|
+                #[trigger] mapping.contains_key(mapping_key_identity) ==> bytes_read_through_the_mapping(
+                    moved_mapping,
+                    moved_storage,
+                    mapping_key_identity,
+                ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity)
+        }),
+{
+    let positions = planned_positions(planned);
+    let keys = planned_mapping_keys(allocations, positions);
+    let sources = planned_sources(allocations, positions);
+    assert forall|earlier: int, later: int|
+        #![trigger keys[earlier], keys[later]]
+        0 <= earlier < later < positions.len() implies keys[earlier] != keys[later] by {
+        assert(positions[earlier] < positions[later]);
+        assert(allocation_is_movable(allocations[positions[earlier]]));
+        assert(allocation_is_movable(allocations[positions[later]]));
+    }
+    assert forall|index: int| 0 <= index < positions.len() implies mapping.contains_key(
+        #[trigger] keys[index],
+    ) && mapping[keys[index]] == sources[index] by {
+        assert(0 <= positions[index] < allocations.len());
+        assert(allocation_is_movable(allocations[positions[index]]));
+    }
+    lemma_relocating_in_order_keeps_every_read(
+        mapping,
+        storage,
+        keys,
+        sources,
+        destinations,
+        positions.len() as int,
+    );
+}
+
+// ===== P4，搬到 R 空 =====
+// 前提「R 内每条未释放记录都有映射 key 且不在隔离集里」（攻方 H-G7-1）在扫描结果上就是每一项都能搬：钉住的豁免三类没有映射 key，
+// 已死的映射不指它，隔离的在隔离集里，不一致的是映射与单元头对不上。
+pub open spec fn every_allocation_in_the_region_is_movable(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+) -> bool {
+    forall|position: int|
+        0 <= position < allocations.len() ==> allocation_is_movable(#[trigger] allocations[position])
+}
+
+pub open spec fn mapping_key_identities(allocations: Seq<CompactionKernelScannedAllocation>) -> Seq<
+    u64,
+> {
+    allocations.map_values(
+        |allocation: CompactionKernelScannedAllocation| mapping_key_identity_of(allocation),
+    )
+}
+
+pub open spec fn source_slots(allocations: Seq<CompactionKernelScannedAllocation>) -> Seq<u64> {
+    allocations.map_values(|allocation: CompactionKernelScannedAllocation| allocation.source_slot)
+}
+
+// 一批批搬到 R 空：每一批搬剩下的最前 min(K, 剩下几项) 项（前提下计划就交这几项，lemma_under_the_premise_a_plan_takes_the_leading_allocations），
+// 目的地按次序从 destinations 里取；这一批发布之后它们离开 R，下一次扫描就是剩下的那几项（singlefs-core 的 compaction_plan::plan_batch 文档）。
+pub open spec fn mapping_after_compacting_the_region(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    batch_units: int,
+    mapping: Map<u64, u64>,
+    destinations: Seq<u64>,
+) -> Map<u64, u64>
+    decreases allocations.len(),
+{
+    if allocations.len() == 0 || batch_units <= 0 {
+        mapping
+    } else {
+        let count = units_in_the_batch(allocations.len() as int, batch_units);
+        mapping_after_compacting_the_region(
+            allocations.skip(count),
+            batch_units,
+            mapping_after_relocating(mapping, mapping_key_identities(allocations), destinations, count),
+            destinations.skip(count),
+        )
+    }
+}
+
+pub open spec fn storage_after_compacting_the_region(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    batch_units: int,
+    storage: spec_fn(u64) -> Seq<u8>,
+    destinations: Seq<u64>,
+) -> spec_fn(u64) -> Seq<u8>
+    decreases allocations.len(),
+{
+    if allocations.len() == 0 || batch_units <= 0 {
+        storage
+    } else {
+        let count = units_in_the_batch(allocations.len() as int, batch_units);
+        storage_after_compacting_the_region(
+            allocations.skip(count),
+            batch_units,
+            storage_after_relocating(storage, source_slots(allocations), destinations, count),
+            destinations.skip(count),
+        )
+    }
+}
+
+// 前提下，计划交的就是剩下的最前 min(K, 剩下几项) 项，而且是「搬」（P3：R 不空就有进展）。
+pub proof fn lemma_under_the_premise_a_plan_takes_the_leading_allocations(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    batch_units: usize,
+    planned: CompactionKernelPlannedBatch,
+)
+    requires
+        batch_units >= 1,
+        allocations.len() > 0,
+        every_allocation_in_the_region_is_movable(allocations),
+        planned_batch_makes_progress(allocations, planned),
+        planned_batch_takes_the_first_movable_units(allocations, batch_units, planned),
+    ensures
+        planned is Relocate,
+        planned_positions(planned) == Seq::new(
+            units_in_the_batch(allocations.len() as int, batch_units as int) as nat,
+            |index: int| index,
+        ),
+{
+    lemma_every_position_is_movable_when_every_allocation_is(allocations, allocations.len() as int);
+    let count = units_in_the_batch(allocations.len() as int, batch_units as int);
+    assert(movable_positions(allocations).take(count) =~= Seq::new(
+        count as nat,
+        |index: int| index,
+    ));
+    assert(allocation_is_movable(allocations[0]));
+}
+
+// P4：前提下按计划一批批搬到 R 空，每个 key 读到的字节不变，映射里的 key 一个不多一个不少。
+pub proof fn corollary_compacting_the_region_until_it_is_empty_keeps_every_read(
+    allocations: Seq<CompactionKernelScannedAllocation>,
+    batch_units: int,
+    mapping: Map<u64, u64>,
+    storage: spec_fn(u64) -> Seq<u8>,
+    destinations: Seq<u64>,
+)
+    requires
+        batch_units >= 1,
+        every_allocation_in_the_region_is_movable(allocations),
+        movable_mapping_keys_are_distinct(allocations),
+        scan_agrees_with_the_mapping(allocations, mapping),
+        destinations_are_distinct_and_free(mapping, destinations, allocations.len() as int),
+    ensures
+        ({
+            let moved_mapping = mapping_after_compacting_the_region(
+                allocations,
+                batch_units,
+                mapping,
+                destinations,
+            );
+            let moved_storage = storage_after_compacting_the_region(
+                allocations,
+                batch_units,
+                storage,
+                destinations,
+            );
+            &&& moved_mapping.dom() == mapping.dom()
+            &&& forall|mapping_key_identity: u64|
+                #[trigger] mapping.contains_key(mapping_key_identity) ==> bytes_read_through_the_mapping(
+                    moved_mapping,
+                    moved_storage,
+                    mapping_key_identity,
+                ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity)
+        }),
+    decreases allocations.len(),
+{
+    if allocations.len() > 0 {
+        let count = units_in_the_batch(allocations.len() as int, batch_units);
+        let keys = mapping_key_identities(allocations);
+        let sources = source_slots(allocations);
+        assert forall|earlier: int, later: int|
+            #![trigger keys[earlier], keys[later]]
+            0 <= earlier < later < count implies keys[earlier] != keys[later] by {
+            assert(allocation_is_movable(allocations[earlier]));
+            assert(allocation_is_movable(allocations[later]));
+        }
+        assert forall|index: int| 0 <= index < count implies mapping.contains_key(
+            #[trigger] keys[index],
+        ) && mapping[keys[index]] == sources[index] by {
+            assert(allocation_is_movable(allocations[index]));
+        }
+        lemma_relocating_in_order_keeps_every_read(
+            mapping,
+            storage,
+            keys,
+            sources,
+            destinations,
+            count,
+        );
+        let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
+        let moved_storage = storage_after_relocating(storage, sources, destinations, count);
+        let remaining = allocations.skip(count);
+        let remaining_destinations = destinations.skip(count);
+        assert forall|position: int| 0 <= position < remaining.len() implies allocation_is_movable(
+            #[trigger] remaining[position],
+        ) by {
+            assert(remaining[position] == allocations[position + count]);
+        }
+        assert forall|one: int, other: int|
+            #![trigger remaining[one], remaining[other]]
+            0 <= one < remaining.len() && 0 <= other < remaining.len() && one != other
+                && allocation_is_movable(remaining[one]) && allocation_is_movable(remaining[other])
+                implies mapping_key_identity_of(remaining[one]) != mapping_key_identity_of(
+            remaining[other],
+        ) by {
+            assert(remaining[one] == allocations[one + count]);
+            assert(remaining[other] == allocations[other + count]);
+        }
+        assert forall|position: int|
+            0 <= position < remaining.len() && allocation_is_movable(
+                #[trigger] remaining[position],
+            ) implies moved_mapping.contains_key(mapping_key_identity_of(remaining[position]))
+            && moved_mapping[mapping_key_identity_of(remaining[position])]
+            == remaining[position].source_slot by {
+            let original = position + count;
+            assert(remaining[position] == allocations[original]);
+            assert(allocation_is_movable(allocations[original]));
+            let key = mapping_key_identity_of(allocations[original]);
+            assert(mapping.contains_key(key));
+            assert forall|index: int| 0 <= index < count implies keys[index] != key by {
+                assert(allocation_is_movable(allocations[index]));
+                assert(keys[index] == mapping_key_identity_of(allocations[index]));
+            }
+            assert(moved_mapping[key] == mapping[key]);
+        }
+        assert forall|earlier: int, later: int|
+            #![trigger remaining_destinations[earlier], remaining_destinations[later]]
+            0 <= earlier < later < remaining.len() implies remaining_destinations[earlier]
+            != remaining_destinations[later] by {
+            assert(remaining_destinations[earlier] == destinations[earlier + count]);
+            assert(remaining_destinations[later] == destinations[later + count]);
+        }
+        assert forall|index: int| 0 <= index < remaining.len() implies destination_is_free_in_the_mapping(
+            moved_mapping,
+            #[trigger] remaining_destinations[index],
+        ) by {
+            let original = index + count;
+            assert(remaining_destinations[index] == destinations[original]);
+            assert(destination_is_free_in_the_mapping(mapping, destinations[original]));
+            assert forall|mapping_key_identity: u64| #[trigger]
+                moved_mapping.contains_key(mapping_key_identity) implies moved_mapping[mapping_key_identity]
+                != destinations[original] by {
+                if moved_mapping[mapping_key_identity] != mapping[mapping_key_identity] {
+                    let moved_index = choose|moved_index: int|
+                        0 <= moved_index < count && moved_mapping[mapping_key_identity]
+                            == destinations[moved_index];
+                    assert(destinations[moved_index] != destinations[original]);
+                }
+            }
+        }
+        corollary_compacting_the_region_until_it_is_empty_keeps_every_read(
+            remaining,
+            batch_units,
+            moved_mapping,
+            moved_storage,
+            remaining_destinations,
+        );
+    }
+}
+
+// ===== spec 自己的冒烟（防 spec 恒真或恒假）=====
+// 一项能搬、K = 1：交「搬」那一项合 P2、P3 与「取最前的」；交「搬空」不合 P3。
+proof fn one_movable_allocation_is_planned_and_not_reported_emptied(
+    only: CompactionKernelScannedAllocation,
+    positions: Vec<usize>,
+)
+    requires
+        allocation_is_movable(only),
+        positions@ == seq![0usize],
+    ensures
+        planned_batch_takes_the_first_movable_units(
+            seq![only],
+            1,
+            CompactionKernelPlannedBatch::Relocate(positions),
+        ),
+        !planned_batch_makes_progress(seq![only], CompactionKernelPlannedBatch::RegionEmptied),
+{
+    let allocations = seq![only];
+    assert(allocations[0] == only);
+    assert(movable_positions_before(allocations, 0) =~= Seq::<int>::empty());
+    assert(movable_positions(allocations) =~= seq![0int]);
+    assert(movable_positions(allocations).take(1) =~= seq![0int]);
+    assert(positions_as_ints(positions@) =~= seq![0int]);
+}
+
+// 选段：唯一一段装着钉住的槽，「选中它」不合 P1。
+proof fn a_segment_holding_a_pinned_slot_is_not_a_sound_choice(only: CompactionKernelSegment)
+    requires
+        only.holds_a_pinned_slot,
+    ensures
+        !region_choice_is_sound(seq![only], CompactionKernelRegionChoice::Chosen(0)),
+{
+}
+
+} // verus!
+
+// 平时的 cargo test 跑的几条：Verus 证的是 spec，这几条钉住编进这个 crate 的两个核在手算得出的小例子上交的答案
+// （crates/mutations.tsv 里点名它们的变异改坏核，它们红）。核与 singlefs-core 的 compaction_plan 逐项对拍在那边的单测里。
+#[cfg(test)]
+mod tests {
+    use super::{
+        choose_compaction_segment, plan_compaction_batch, CompactionKernelAllocationClass,
+        CompactionKernelPlannedBatch, CompactionKernelRegionChoice,
+        CompactionKernelScannedAllocation, CompactionKernelSegment, CompactionKernelSegmentScan,
+    };
+
+    fn segment(live_slots: u64, scan: CompactionKernelSegmentScan) -> CompactionKernelSegment {
+        CompactionKernelSegment {
+            live_slots,
+            is_opened_in_this_mount: false,
+            is_the_live_intent_region: false,
+            holds_a_pinned_slot: false,
+            is_refused_by_the_region_scan: false,
+            scan,
+        }
+    }
+
+    fn allocation(
+        source_slot: u64,
+        class: CompactionKernelAllocationClass,
+    ) -> CompactionKernelScannedAllocation {
+        CompactionKernelScannedAllocation { source_slot, class }
+    }
+
+    fn movable(mapping_key_identity: u64) -> CompactionKernelAllocationClass {
+        CompactionKernelAllocationClass::Movable {
+            mapping_key_identity,
+        }
+    }
+
+    fn relocated_positions(planned: CompactionKernelPlannedBatch) -> Vec<usize> {
+        match planned {
+            CompactionKernelPlannedBatch::Relocate(positions) => positions,
+            CompactionKernelPlannedBatch::RegionEmptied => {
+                panic!("期望「搬」，计划交了「搬空」")
+            }
+            CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty => {
+                panic!("期望「搬」，计划交了「搬不动」")
+            }
+        }
+    }
+
+    /// 三段：活槽 5（扫过、有能搬的）、活槽 2（扫过、有能搬的）、活槽 2（没扫过）⇒ 选活槽最少、同分起点最前的那一段（下标 1）。
+    #[test]
+    fn the_fewest_live_slots_win_and_a_tie_goes_to_the_earlier_segment() {
+        let segments = [
+            segment(5, CompactionKernelSegmentScan::ScannedWithAMovableUnit),
+            segment(2, CompactionKernelSegmentScan::ScannedWithAMovableUnit),
+            segment(2, CompactionKernelSegmentScan::NotScanned),
+        ];
+        assert_eq!(
+            choose_compaction_segment(&segments),
+            CompactionKernelRegionChoice::Chosen(1),
+            "活槽 2 的两段同分，取起点最前的下标 1；它扫过、有能搬的单元，直接选中"
+        );
+    }
+
+    /// 活槽最少的那一段装着钉住的槽、次少的被这次挂载开过 ⇒ 都不是候选，剩下没扫过的那一段要先扫。
+    #[test]
+    fn excluded_segments_are_skipped_and_an_unscanned_candidate_is_scanned_first() {
+        let mut pinned = segment(0, CompactionKernelSegmentScan::ScannedWithAMovableUnit);
+        pinned.holds_a_pinned_slot = true;
+        let mut opened = segment(1, CompactionKernelSegmentScan::ScannedWithAMovableUnit);
+        opened.is_opened_in_this_mount = true;
+        let segments = [
+            pinned,
+            opened,
+            segment(9, CompactionKernelSegmentScan::NotScanned),
+        ];
+        assert_eq!(
+            choose_compaction_segment(&segments),
+            CompactionKernelRegionChoice::ScanBeforeChoosing(2),
+            "钉住的段与这次挂载开过的段都不是候选，只剩没扫过的下标 2：先扫它"
+        );
+    }
+
+    /// 扫过而一个能搬的都没有的段、活意图的 R、被扫描拒过的段都不是候选 ⇒ 一段都没资格。
+    #[test]
+    fn no_segment_is_eligible_when_each_one_is_excluded() {
+        let mut live_intent = segment(1, CompactionKernelSegmentScan::ScannedWithAMovableUnit);
+        live_intent.is_the_live_intent_region = true;
+        let mut refused = segment(1, CompactionKernelSegmentScan::NotScanned);
+        refused.is_refused_by_the_region_scan = true;
+        let segments = [
+            segment(0, CompactionKernelSegmentScan::ScannedWithoutAMovableUnit),
+            live_intent,
+            refused,
+        ];
+        assert_eq!(
+            choose_compaction_segment(&segments),
+            CompactionKernelRegionChoice::NoEligibleSegment,
+            "三段各有一条不合资格，一段都不该选"
+        );
+    }
+
+    /// 能搬、钉住、能搬、已死、能搬，K = 2 ⇒ 搬下标 0 与 2，跳过钉住与已死、不取第三个能搬的。
+    #[test]
+    fn a_batch_takes_the_first_movable_units_up_to_the_batch_size() {
+        let allocations = [
+            allocation(64, movable(10)),
+            allocation(65, CompactionKernelAllocationClass::Pinned),
+            allocation(66, movable(11)),
+            allocation(68, CompactionKernelAllocationClass::Dead),
+            allocation(69, movable(12)),
+        ];
+        assert_eq!(
+            relocated_positions(plan_compaction_batch(&allocations, 2)),
+            vec![0, 2],
+            "K = 2：按槽升序取前两个能搬的（下标 0、2），钉住与已死的不进"
+        );
+    }
+
+    /// R 里没有记录 ⇒ 搬空；只有隔离与不一致的 ⇒ 搬不动。
+    #[test]
+    fn an_empty_region_is_emptied_and_a_region_without_a_movable_unit_is_not() {
+        assert!(
+            matches!(
+                plan_compaction_batch(&[], 1),
+                CompactionKernelPlannedBatch::RegionEmptied
+            ),
+            "R 里一条未释放记录都没有：搬空"
+        );
+        let stuck = [
+            allocation(64, CompactionKernelAllocationClass::Quarantined),
+            allocation(66, CompactionKernelAllocationClass::Inconsistent),
+        ];
+        assert!(
+            matches!(
+                plan_compaction_batch(&stuck, 3),
+                CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty
+            ),
+            "R 里有隔离与不一致的记录、一个能搬的都没有：搬不动，不是搬空"
+        );
+    }
+}
````
