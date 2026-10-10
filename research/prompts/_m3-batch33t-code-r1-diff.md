# 删除批第 33 批 T 合入的改动原样（2026-10-07）

## 实现员 t：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch33/t/patch/crates.patch）

````diff
diff --git a/crates/singlefs-core/src/admission.rs b/crates/singlefs-core/src/admission.rs
index 1474bc3..5fbc17e 100644
--- a/crates/singlefs-core/src/admission.rs
+++ b/crates/singlefs-core/src/admission.rs
@@ -480,9 +480,11 @@ pub struct DeviceAdmissionTerms {
     /// 式子里的「挂载期承诺量」按设备摊的成员：实例切换的预留（D28（挂载期承诺量） 已定项 3，
     /// `instance_switch_reserve_on_one_device`）。它的池级成员 checkpoint 保留池在式子里单列，这里不重复扣。
     pub mount_time_commitment: BytesOnOneDevice,
-    /// 式子里的「被抛弃根独占量」（D28（挂载期承诺量） 已定项 1 第九项，C318（影子账隔离的单元没进准入不等式））：
-    /// 只被被抛弃根引用的槽，与分配器实际隔离的是同一个集合。
-    pub abandoned_root_exclusive: BytesOnOneDevice,
+    /// 式子第九项（D28（挂载期承诺量） 已定项 1，C318（影子账隔离的单元没进准入不等式））：分配器实际隔离的槽，是同一个集合——
+    /// 只被被抛弃根引用的槽，加可写挂载那一刻水位之上没施加的 journal 记录点名、不被回退候选集里的根（与现行账）引用、还没解开的槽
+    /// （C694 的 C，判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 3 条：第九项的定义随 C 扩成分配器实际隔离的集合）。
+    /// 读法不变：取 `DeviceFreeMap::isolated_slots`。
+    pub isolated_for_abandoned_roots_and_unapplied_journal_records: BytesOnOneDevice,
     /// 式子里的「环窗口」（D28（挂载期承诺量） 已定项 1 末项）：删除各批新写的兜底，常驻扣项、只住内存、每块盘各扣这一份
     /// （[`RingWindowOfTheReading`]）。
     pub ring_window: BytesOnOneDevice,
@@ -556,7 +558,7 @@ impl AdmissionReading {
     }
 
     /// 从分配器此刻的计数取逐盘各项：容量 = 单元区槽数、已分配 = 占着的槽数（含已释放还在窗口里的）、
-    /// 被抛弃根独占量 = 影子账隔离的槽数，不可回收取 [`UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED`]；副本数按
+    /// 第九项 = 分配器隔离的槽数（影子账与水位之上没施加的记录点名的两部分），不可回收取 [`UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED`]；副本数按
     /// [`ReplicaCount::of_every_device_in_the_pool`]。挂载期承诺量与环窗口（每块盘同一个数）、全池三项由调用方给。
     ///
     /// D28（挂载期承诺量） 已定项 2 要的是「已发布统计量 − 在飞已批准」。这几个计数就是每次发布写进记账行的那几个数
@@ -580,7 +582,8 @@ impl AdmissionReading {
                 allocated: BytesOnOneDevice::of_slots(device_map.allocated_slots()),
                 unreclaimable: UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED,
                 mount_time_commitment: mount_time_commitment_on_each_device,
-                abandoned_root_exclusive: BytesOnOneDevice::of_slots(device_map.isolated_slots()),
+                isolated_for_abandoned_roots_and_unapplied_journal_records:
+                    BytesOnOneDevice::of_slots(device_map.isolated_slots()),
                 ring_window: ring_window_on_each_device,
             })
             .collect();
@@ -624,7 +627,7 @@ impl AdmissionReading {
                     terms.allocated,
                     terms.unreclaimable,
                     terms.mount_time_commitment,
-                    terms.abandoned_root_exclusive,
+                    terms.isolated_for_abandoned_roots_and_unapplied_journal_records,
                     terms.ring_window,
                 ]
                 .map(|term| i128::from(term.0));
@@ -1459,7 +1462,7 @@ mod tests {
             allocated: BytesOnOneDevice::ZERO,
             unreclaimable: BytesOnOneDevice::ZERO,
             mount_time_commitment: BytesOnOneDevice::ZERO,
-            abandoned_root_exclusive: BytesOnOneDevice::ZERO,
+            isolated_for_abandoned_roots_and_unapplied_journal_records: BytesOnOneDevice::ZERO,
             ring_window: BytesOnOneDevice::ZERO,
         }
     }
@@ -1489,7 +1492,7 @@ mod tests {
                     allocated: slots(100),
                     unreclaimable: slots(1),
                     mount_time_commitment: slots(7),
-                    abandoned_root_exclusive: slots(3),
+                    isolated_for_abandoned_roots_and_unapplied_journal_records: slots(3),
                     ring_window: slots(13),
                 },
                 DeviceAdmissionTerms {
@@ -1498,7 +1501,8 @@ mod tests {
                     allocated: slots(90),
                     unreclaimable: BytesOnOneDevice::ZERO,
                     mount_time_commitment: slots(7),
-                    abandoned_root_exclusive: BytesOnOneDevice::ZERO,
+                    isolated_for_abandoned_roots_and_unapplied_journal_records:
+                        BytesOnOneDevice::ZERO,
                     ring_window: slots(13),
                 },
             ],
@@ -1913,7 +1917,7 @@ mod tests {
             allocated: slots(3),
             unreclaimable: BytesOnOneDevice::ZERO,
             mount_time_commitment,
-            abandoned_root_exclusive: slots(isolated_slots),
+            isolated_for_abandoned_roots_and_unapplied_journal_records: slots(isolated_slots),
             ring_window: BytesOnOneDevice::ZERO,
         };
         assert_eq!(
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index b64f765..b17f7af 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -593,7 +593,8 @@ impl DeviceFreeMap {
         }
     }
 
-    /// 影子账隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」的这块盘那一项）。
+    /// 这块盘上隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项的这块盘那一项）：只被被抛弃根引用的，加水位之上没施加的 journal 记录点名、
+    /// 还没解开的（C694 的 C）；两部分置的是同一个隔离位，同一个槽只数一次。
     #[must_use]
     pub fn isolated_slots(&self) -> u64 {
         self.isolated_slots
@@ -1055,6 +1056,12 @@ pub struct RootRingOccupancy {
     latest_recorded_txg: CheckpointTxg,
     /// 见 [`RootRingOccupancy::instance_table_judging_the_rereads`]。
     instance_table_judging_the_rereads: InstanceTableRecords,
+    /// 可写挂载那一刻水位之上、没被施加的 journal 记录点名的落点（C694 的 C，判决 `research/prompts/m3-c694-r2-main-verification.md`
+    /// 第三节第 1 条：选法取水位之上全部记录），按那条记录的 txg 在根环里对应的槽归组：那条记录所属的发布若落过根，根就住在那一槽，
+    /// 这个进程写掉那一槽之前那条根还可能被下一次恢复择到。挂载那一刻仍被候选根或现行账引用、因而没隔离的落点也在内：
+    /// 它们之后在这次挂载里被回收时要补隔离。只在 [`PoolAllocator::record_root_written_by_this_process`] 写掉那一槽、那次发布取完落点之后
+    /// 解开（判决同一条的 `ring`）；进程内重读那一槽（D16（发布语义） 已定项 1 读回全零的槽）读出什么都不解开它。
+    placements_named_by_unapplied_journal_records: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>,
 }
 
 impl RootRingOccupancy {
@@ -1074,9 +1081,29 @@ impl RootRingOccupancy {
             effective_floor,
             latest_recorded_txg: latest_txg_before_the_next_publish,
             instance_table_judging_the_rereads: InstanceTableRecords { rows: Vec::new() },
+            placements_named_by_unapplied_journal_records: BTreeMap::new(),
         }
     }
 
+    /// 还没解开的、水位之上没施加的 journal 记录点名的全部落点（[`RootRingOccupancy`] 那一项的注释），按根环槽的次序摊平：
+    /// 抬 F 按新候选集重算隔离时拿它（`mount::raise_rollback_floor` 一族）。
+    #[must_use]
+    pub fn placements_named_by_unapplied_journal_records(&self) -> Vec<PlacementOnDevice> {
+        self.placements_named_by_unapplied_journal_records
+            .values()
+            .flatten()
+            .copied()
+            .collect()
+    }
+
+    /// 环里还在的被抛弃根引用着的全部落点，加还没解开的、水位之上没施加的记录点名的全部落点：清隔离位之前要看的那一份
+    /// （一个槽两边任一边还罩着就不清），回收出来的槽里它们罩着的补隔离。
+    fn placements_the_isolation_still_protects(&self) -> Vec<PlacementOnDevice> {
+        let mut protected = self.placements_referenced_by_abandoned_roots();
+        protected.extend(self.placements_named_by_unapplied_journal_records());
+        protected
+    }
+
     /// 根环每区槽数 S（系统配置偏移 362）：准入式子里环窗口按它算（`admission::ring_window_on_one_device`）。
     #[must_use]
     pub fn slots_per_region(&self) -> RootRingSlotsPerRegion {
@@ -1938,7 +1965,8 @@ impl PoolAllocator {
         }
     }
 
-    /// 回退的影子账：把一个只被被抛弃根引用的落点在它那块盘上隔离。
+    /// 回退的影子账：把一个只被被抛弃根引用的落点，或水位之上没施加的 journal 记录点名、不被候选根与现行账引用的落点（C694 的 C），
+    /// 在它那块盘上隔离。
     pub fn isolate_abandoned(&mut self, device: DeviceIdentity, slot: SlotNumber, span: u64) {
         let device_map = self
             .devices
@@ -1964,6 +1992,22 @@ impl PoolAllocator {
         }
     }
 
+    /// 记下可写挂载那一刻水位之上没施加的 journal 记录点名的落点（按记录的 txg 对应的根环槽归组，C694 的 C）：
+    /// 隔离位由调用方在同一次影子账里置好（`mount::isolate_slots_referenced_only_by_abandoned_roots`），这里只记着什么时候解开
+    /// （[`Self::record_root_written_by_this_process`] 写掉那一槽时）、回收出来的槽要不要补隔离。挂载装上根环表之后、第一次记根之前调一次。
+    ///
+    /// # Panics
+    /// 没装根环表：可写挂载在重建分配器那一步先装上根环表再调它（`mount::rebuilt_allocator`），不装就没有解开的时机。
+    pub fn note_placements_named_by_unapplied_journal_records(
+        &mut self,
+        named_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>,
+    ) {
+        self.root_ring
+            .as_mut()
+            .expect("记水位之上没施加的记录点名的落点之前装了根环表：mount::rebuilt_allocator 先装根环表再记它")
+            .placements_named_by_unapplied_journal_records = named_by_ring_slot;
+    }
+
     /// 这会儿装着的根环；`None` 见 `root_ring` 字段的注释。
     #[must_use]
     pub fn root_ring_occupancy(&self) -> Option<&RootRingOccupancy> {
@@ -2013,8 +2057,13 @@ impl PoolAllocator {
             occupancy.effective_floor,
             occupancy.oldest_valid_root_in_the_ring(),
         );
-        let still_referenced_by_abandoned_roots =
-            occupancy.placements_referenced_by_abandoned_roots();
+        // C694 的 C 只在这里解开（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条 `ring`）：这个进程写掉那条记录的 txg
+        // 对应的根环槽，这次发布的落点已经取完（调用时机是调用方的前置条件，见上）。
+        let released_named_by_unapplied_journal_records = occupancy
+            .placements_named_by_unapplied_journal_records
+            .remove(&ring_slot)
+            .unwrap_or_default();
+        let still_protected_by_the_isolation = occupancy.placements_the_isolation_still_protects();
         match displaced {
             Some(
                 RootRingOccupant::AbandonedRoot {
@@ -2027,9 +2076,9 @@ impl PoolAllocator {
                         },
                     ..
                 },
-            ) => self.clear_isolation_no_abandoned_root_in_the_ring_backs(
+            ) => self.clear_isolation_nothing_still_protected_backs(
                 &referenced_placements,
-                &still_referenced_by_abandoned_roots,
+                &still_protected_by_the_isolation,
             ),
             Some(
                 RootRingOccupant::ValidRoot { .. }
@@ -2043,11 +2092,15 @@ impl PoolAllocator {
             )
             | None => {}
         }
+        self.clear_isolation_nothing_still_protected_backs(
+            &released_named_by_unapplied_journal_records,
+            &still_protected_by_the_isolation,
+        );
         let reclaimed =
             self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);
-        self.isolate_reclaimed_slots_an_abandoned_root_still_references(
+        self.isolate_reclaimed_slots_the_isolation_still_protects(
             &reclaimed,
-            &still_referenced_by_abandoned_roots,
+            &still_protected_by_the_isolation,
         );
     }
 
@@ -2146,8 +2199,9 @@ impl PoolAllocator {
         }
     }
 
-    /// 一条被抛弃根离开根环：它引用的槽里，`still_referenced` 不罩着的清掉隔离位。
-    fn clear_isolation_no_abandoned_root_in_the_ring_backs(
+    /// 一条被抛弃根离开根环，或水位之上没施加的记录点名的一组落点解开：`left_the_ring` 罩的槽里，`still_referenced`
+    /// （环里还在的被抛弃根引用的、还没解开的记录点名的，[`RootRingOccupancy::placements_the_isolation_still_protects`]）不罩着的清掉隔离位。
+    fn clear_isolation_nothing_still_protected_backs(
         &mut self,
         left_the_ring: &[PlacementOnDevice],
         still_referenced: &[PlacementOnDevice],
@@ -2171,8 +2225,9 @@ impl PoolAllocator {
         }
     }
 
-    /// 挂载内回收出来的槽里，环里还在的被抛弃根引用着的补隔离（同一个槽隔离两次不重复计数，`DeviceFreeMap::isolate`）。
-    fn isolate_reclaimed_slots_an_abandoned_root_still_references(
+    /// 挂载内回收出来的槽里，环里还在的被抛弃根引用着的、还没解开的水位之上没施加的记录点名的补隔离
+    /// （同一个槽隔离两次不重复计数，`DeviceFreeMap::isolate`）。
+    fn isolate_reclaimed_slots_the_isolation_still_protects(
         &mut self,
         reclaimed: &[AllocationRecord],
         still_referenced: &[PlacementOnDevice],
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index da0164d..622cf3a 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -58,7 +58,8 @@ use crate::recovery::{
     root_is_abandoned_by_the_instance_table, scan_journal,
     system_configuration_slots_of_one_device_read_once,
     system_configuration_witness_is_undecidable, tree_table_has_no_entries,
-    user_visible_tree_root_pointers, verified_system_configuration_slots, BadRootRingSlotReading,
+    user_visible_tree_root_pointers, verified_system_configuration_slots,
+    version_witnessed_by_the_system_configuration_to_fall_back_to, BadRootRingSlotReading,
     EffectiveFloorReadingStillUnreadableAfterOneReread, InstanceTableChain,
     InstanceTableOfTheNewestRootStillUnreadableAfterOneReread, JournalScanReport, PoolReader,
     RebuildVersionFailure, RebuiltVersion, RecoveryFailure,
@@ -391,6 +392,11 @@ pub enum StillUnreadableAfterOneReread {
     /// 影子账读一条被抛弃根的账（树表或分配记录树）读不出、从中央映射也现算不成，重读一次仍是这样：拒可写
     /// （C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算那一条的最后一支，用户 2026-09-28 定）。
     AccountOfAnAbandonedRoot { abandoned_root: RollbackTarget },
+    /// 可写挂载读阶段：水位之上有一次没施加的发布 `publish`，读得出的那几条记录的「本次发布内序号」不是 1..N 连着或没有一条带末条标志，
+    /// 而它们点名的单元又有一个验不过；在这次挂载的读缓存上重做读阶段一遍仍是这样（F1 乙，用户 2026-10-07 定「缺记录就拒可写」，
+    /// 判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 6 条）。缺的那条记录点名的单元挂载时谁都不知道是哪几个，
+    /// 隔离不到；在取号之前拒，盘上逐字节不变，只读挂载照常。
+    PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { publish: RollbackTarget },
 }
 
 impl From<RootRingSlotKnownToHoldARootStillBadAfterOneReread> for StillUnreadableAfterOneReread {
@@ -959,11 +965,12 @@ pub struct MountOutput {
     /// 这次挂载走的影子账分支名（`ShadowLedger::branch_name`）：两臂在这里永远不同，
     /// 而 `isolated_slots_per_device` 两臂都可能是 0，光看它分不开（五条硬要求第 4 条）。
     pub shadow_ledger_branch: &'static str,
-    /// 影子账隔离的槽数，逐盘（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」，只住内存）：只被被抛弃根引用的槽，
-    /// 仍被候选集里的根引用的不在其内（对用户 2026-09-16 定的窄读法措辞的收严，见 `isolate_slots_referenced_only_by_abandoned_roots`；预想）。
+    /// 分配器隔离的槽数，逐盘（D28（挂载期承诺量） 已定项 1 第九项，只住内存）：只被被抛弃根引用的槽，加水位之上没施加的 journal 记录点名、
+    /// 不被候选集里的根与现行账引用的槽（C694 的 C）；仍被候选集里的根引用的不在其内（对用户 2026-09-16 定的窄读法措辞的收严，
+    /// 见 `isolate_slots_referenced_only_by_abandoned_roots`；预想）。
     pub isolated_slots_per_device: Vec<(DeviceIdentity, u64)>,
-    /// 被抛弃根里树表或分配记录树读不出、解不开的条数：这样的根影子账罩不到，只计数、不拒绝挂载
-    /// （步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败）。
+    /// 被抛弃根里树表或分配记录树读不出、解不开的条数：这样的根从中央映射现算它引用的落点照样隔离（C393）；映射也现算不成、重读一次仍不成时，
+    /// 现行账里有那条根之后的记录罩住它的树表槽（与映射根槽）的照样计数、不隔离（C694 的 B），没有的拒可写（`isolate_slots_referenced_only_by_abandoned_roots`）。
     pub abandoned_roots_unreadable: u64,
     /// 这次挂载的空间准入怎么判的、推了几串抬 F 的空发布（[`MountSpaceAdmission`]）。推的那几串写的根不在 `warm_up_publishes` 里：
     /// 它们接在暖机之后，`Mounted::current` 是最后落盘的那一次。
@@ -1472,13 +1479,21 @@ struct ShadowLedgerComputed {
 /// A 仍豁免 mkfs 实例表那 2 槽，它们被回收、发出去，而被抛弃根 B 还引用着，违反 D23（journal 的角色与格式） 已定项 14 的主句。
 /// 预想、偏离用户定案的措辞，交用户。
 /// 一条根引用的 = 它那一版账里还分配着的落点；账里已释放的是它换下的上一版的，不算它引用。
-/// 树表或分配记录树读不出、解不开的被抛弃根只计数，不拒绝挂载；候选根读不出就当它什么都不豁免（隔离只会多不会少）。
+/// 被抛弃根的树表或分配记录树读不出、解不开时不跳过：从中央映射现算它引用的落点照样隔离，计进 `abandoned_roots_unreadable`
+/// （C393 取 (b)-从映射现算）；映射也现算不成就重读一次（重读之前调钩子），仍不成时看现行账（`current_records`）：现行账里那条根的树表槽
+/// （与中央映射树根槽，映射根是空指针时只看树表槽）每块盘上都有一条分配代大于那条根的 txg、罩住它的记录，就是那几槽已被那条根之后的合法写占了，
+/// 那条根引用的单元已经没有要护的，只计数、不隔离、不拒（C694 的 B，判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 2 条：
+/// 凭据只读内存里的现行账，不读那一槽）；没有这样的记录（那几槽只是这一次读不出）就在任何写之前拒：挂载在取号之前拒可写、抬 F 拒这一次抬
+/// （[`StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot`]）。候选根读不出就当它什么都不豁免（隔离只会多不会少）。
+/// 同一次也隔离水位之上没施加的 journal 记录点名的落点（`placements_named_by_unapplied_journal_records`，C694 的 C）：豁免与被抛弃根那一份
+/// 同一个集合（候选集里的根与现行账引用的槽），D28（挂载期承诺量） 已定项 1 第九项按分配器实际隔离的槽数取，两部分都在里面。
 /// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销；
-/// 清只在被抛弃的根离开根环的那一次发布里清（`PoolAllocator::record_root_written_by_this_process`，按交回的每条根引用的落点）。
+/// 被抛弃根引用的只在它离开根环的那一次发布里清，水位之上的记录点名的只在这个进程写掉那条记录的 txg 对应的根环槽那一次发布里清
+/// （`PoolAllocator::record_root_written_by_this_process`，按交回的每条根引用的落点与分配器根环表记着的那一份）。
 #[allow(
     clippy::ptr_arg,
     clippy::too_many_arguments,
-    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、重读前的快照"
+    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、水位之上没施加的记录点名的落点、重读前的快照"
 )]
 fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
@@ -1488,6 +1503,7 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     is_abandoned: &dyn Fn(&RootRecord) -> bool,
     floor: CheckpointTxg,
     current_records: &[AllocationRecord],
+    placements_named_by_unapplied_journal_records: &[PlacementOnDevice],
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<ShadowLedgerComputed, MountError> {
     let mut referenced_by_candidates: BTreeSet<(u32, u64)> = current_records
@@ -1517,43 +1533,19 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
         // `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）：账（树表或分配记录树）读不出时不跳过，
         // 从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报（显式报告）；
         // 现算也不成（映射树读不出、解不开）才重读一次（重读之前调钩子），账与现算都仍不成就在取号之前拒可写、只读照常。
-        let placements =
-            match placements_referenced_by_root(devices, unit_area_start, root) {
-                Some(placements) => placements,
-                None => {
-                    unreadable += 1;
-                    match placements_referenced_by_root_computed_from_the_mapping(
-                        devices,
-                        unit_area_start,
-                        root,
-                    ) {
-                        Some(computed) => computed,
-                        None => {
-                            before_the_one_reread.before_rereading();
-                            match placements_referenced_by_root(devices, unit_area_start, root)
-                                .or_else(|| {
-                                    placements_referenced_by_root_computed_from_the_mapping(
-                                        devices,
-                                        unit_area_start,
-                                        root,
-                                    )
-                                }) {
-                                Some(placements) => placements,
-                                None => return Err(
-                                    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
-                                        StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
-                                            abandoned_root: RollbackTarget {
-                                                instance: root.instance,
-                                                checkpoint_txg: root.checkpoint_txg,
-                                            },
-                                        },
-                                    )),
-                                ),
-                            }
-                        }
-                    }
-                }
-            };
+        let placements = match placements_referenced_by_root(devices, unit_area_start, root) {
+            Some(placements) => placements,
+            None => {
+                unreadable += 1;
+                placements_of_an_abandoned_root_whose_account_is_unreadable(
+                    devices,
+                    unit_area_start,
+                    root,
+                    current_records,
+                    before_the_one_reread,
+                )?
+            }
+        };
         for (device, slot, span_slots) in placements.iter().copied() {
             let key = (device.0, slot.0);
             if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
@@ -1569,12 +1561,164 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
                 .collect(),
         );
     }
+    for placement in placements_named_by_unapplied_journal_records {
+        let key = (placement.device.0, placement.slot.0);
+        if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
+            continue;
+        }
+        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);
+    }
     Ok(ShadowLedgerComputed {
         abandoned_roots_unreadable: unreadable,
         placements_referenced_by_abandoned_root,
     })
 }
 
+/// 一条被抛弃根的账（树表或分配记录树）这一遍读不出之后，它引用的落点从哪来（C393 取 (b)-从映射现算，用户 2026-09-28 定，被攻过零轮；
+/// C694 的 B，用户 2026-10-07 定）：从中央映射现算；现算也不成就重读一次（重读之前调钩子），账或现算有一样成就用它；仍都不成时，
+/// 现行账里有那条根之后的记录罩住它的树表槽（与映射根槽）就当它什么都不引用（[`the_current_account_records_later_units_over_the_account_slots_of`]），
+/// 没有就拒。
+///
+/// # Errors
+/// 账与映射重读一次仍都不成、现行账里又没有那道凭据 ⇒ `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`（在任何写之前）。
+#[allow(
+    clippy::ptr_arg,
+    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
+)]
+fn placements_of_an_abandoned_root_whose_account_is_unreadable<Device: BlockDevice>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    unit_area_start: UnitAreaStart,
+    root: &RootRecord,
+    current_records: &[AllocationRecord],
+    before_the_one_reread: &mut BeforeTheOneReread<'_>,
+) -> Result<Vec<(DeviceIdentity, SlotNumber, u64)>, MountError> {
+    if let Some(computed) =
+        placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)
+    {
+        return Ok(computed);
+    }
+    before_the_one_reread.before_rereading();
+    if let Some(placements) =
+        placements_referenced_by_root(devices, unit_area_start, root).or_else(|| {
+            placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)
+        })
+    {
+        return Ok(placements);
+    }
+    if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {
+        return Ok(Vec::new());
+    }
+    Err(MountError::NewerStateStillUnreadableAfterOneReread(
+        Box::new(StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
+            abandoned_root: RollbackTarget {
+                instance: root.instance,
+                checkpoint_txg: root.checkpoint_txg,
+            },
+        }),
+    ))
+}
+
+/// C694 的 B 那道凭据（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 2 条）：被抛弃根 `root` 的树表槽、与它的中央映射树根槽
+/// （映射根是空指针时没有这一槽，只看树表槽）的每一条位置条目，在现行账 `current_records` 里那块盘上都有一条分配代大于 `root` 的 txg、
+/// 罩住那一槽的记录（已释放的也算：分配代说的是那一槽在 `root` 之后被重新分配过）。只读内存里的现行账，不读那几槽的内容——读回旧内容的
+/// 瞬时故障下读那一槽判「诞生身份不同」会把活着的根判成已被占（判决第二节 K3 那一行，W1 出局）。
+fn the_current_account_records_later_units_over_the_account_slots_of(
+    root: &RootRecord,
+    current_records: &[AllocationRecord],
+) -> bool {
+    let mapping_root_locations: &[crate::pointer::LocationEntry] =
+        if root.mapping_root == crate::pointer::NodePointer::empty_root() {
+            &[]
+        } else {
+            &root.mapping_root.locations
+        };
+    root.tree_table
+        .locations
+        .iter()
+        .chain(mapping_root_locations)
+        .all(|location| {
+            current_records.iter().any(|record| {
+                record.device == location.device
+                    && record.slot.0 <= location.slot.0
+                    && location.slot.0 < record.slot.0 + u64::from(record.span_slots)
+                    && record.generation > root.checkpoint_txg
+            })
+        })
+}
+
+/// 水位之上、没被施加的 journal 记录点名的落点（C694 的 C，判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条，选法 `all`：
+/// 不只认带末条标志的记录），按那条记录的 txg 在根环里对应的槽（[`target_for_publish`]）归组。水位是所选那一版 `selected_version`
+/// （施加之后的根）的 (实例代号, checkpoint_txg)，按实例代号为主比，与恢复判施加哪一段同一个序（D23（journal 的角色与格式） 已定项 14 注 2）；
+/// 水位之下的记录要么施加过、要么比所选那一版旧，不在其内。一条记录交出的落点：它点名的每个单元的两条位置条目（跨度按单元种类），
+/// 与它新根段里四条指针（树表、中央映射树根、实例表、分配记录树根；空指针不算）的位置条目——一次发布的每条记录都带整次发布的新根段
+/// （已定项 15），末条读不出时前几条的新根段照样说得出那次发布的树表与映射根在哪。落点不在那块盘的单元区里的（坏盘上的记录）不交。
+/// 只读内存里的记录、不读盘；读盘只问盘的字节数（落点在不在单元区）。
+#[allow(
+    clippy::ptr_arg,
+    reason = "placement_lies_in_the_unit_area_of_its_device 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
+)]
+fn placements_named_by_unapplied_journal_records<Device: BlockDevice>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    unit_area_start: UnitAreaStart,
+    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
+    selected_version: &RootRecord,
+    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
+) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
+    let mut named_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> = BTreeMap::new();
+    for record in records.values().filter(|record| {
+        (record.instance, record.checkpoint_txg)
+            > (selected_version.instance, selected_version.checkpoint_txg)
+    }) {
+        let new_root_segment_pointers = [
+            (&record.new_tree_table, TransactionUnit::TreeTable),
+            (&record.new_mapping_root, TransactionUnit::MappingTree),
+            (&record.new_instance_table, TransactionUnit::InstanceTable),
+            (
+                &record.new_allocation_record_tree_root,
+                TransactionUnit::AllocationTree,
+            ),
+        ]
+        .into_iter()
+        .filter(|(pointer, _)| **pointer != crate::pointer::NodePointer::empty_root())
+        .flat_map(|(pointer, unit)| {
+            pointer
+                .locations
+                .iter()
+                .map(move |location| (*location, unit.span_slots()))
+        });
+        let named_units = record.named.iter().flat_map(|named| {
+            let span_slots = crate::recovery::span_in_slots_of_a_named_unit(named);
+            named
+                .locations
+                .iter()
+                .map(move |location| (*location, span_slots))
+        });
+        let placements: Vec<PlacementOnDevice> = named_units
+            .chain(new_root_segment_pointers)
+            .filter(|(location, span_slots)| {
+                placement_lies_in_the_unit_area_of_its_device(
+                    devices,
+                    unit_area_start,
+                    location.device,
+                    location.slot,
+                    *span_slots,
+                )
+                .is_ok()
+            })
+            .map(|(location, span_slots)| PlacementOnDevice {
+                device: location.device,
+                slot: location.slot,
+                span: span_slots,
+            })
+            .collect();
+        named_by_ring_slot
+            .entry(target_for_publish(record.checkpoint_txg, slots_per_region))
+            .or_default()
+            .extend(placements);
+    }
+    named_by_ring_slot
+}
+
 /// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮）：被抛弃根的树表或分配记录树读不出时，
 /// 不读分配记录、不读树表，按这条根现算它引用的全部落点：根记录直接指着的豁免单元（实例表链上认得出的每一片、
 /// 树表、中央映射树的每个节点）加中央映射树里每一条映射条目的落点（数据单元、打包单元、四棵树的节点，含分配记录树自己的节点）。
@@ -1950,11 +2094,17 @@ fn txgs_ring_slots_bad_at_mount_can_hold(
     clippy::ptr_arg,
     reason = "choose_root 等走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
 )]
+#[allow(
+    clippy::too_many_arguments,
+    reason = "每个参数各是一样东西：设备表、系统配置、上一版、新实例第一次发布的 txg、读阶段扫出的记录、所选那一版、影子账开关、重读前的钩子"
+)]
 fn rebuilt_allocator<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
     system_configuration: &crate::system_configuration::SystemConfiguration,
     previous: &PreviousVersion,
     first_txg: CheckpointTxg,
+    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
+    selected_version: &RootRecord,
     shadow_ledger: ShadowLedger,
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<RebuiltAllocator, MountError> {
@@ -2214,6 +2364,18 @@ fn rebuilt_allocator<Device: BlockDevice>(
         floor_of_the_reclaim,
         ReclaimedReuse::HeldUntilFloorTakesEffect,
     );
+    // C694 的 C（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条）：水位之上没施加的记录点名的落点与影子账同一次隔离、
+    // 豁免同一个集合。影子账关着（只供测试）时它也不隔离：那一臂要的是「什么都不护」的对照。
+    let placements_named_by_unapplied_journal_records = match shadow_ledger {
+        ShadowLedger::On => placements_named_by_unapplied_journal_records(
+            devices,
+            unit_area_start,
+            records,
+            selected_version,
+            slots_per_region,
+        ),
+        ShadowLedger::Off => BTreeMap::new(),
+    };
     let shadow_ledger_computed = match shadow_ledger {
         ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(
             devices,
@@ -2223,6 +2385,11 @@ fn rebuilt_allocator<Device: BlockDevice>(
             &is_abandoned,
             effective_floor,
             &current_records,
+            &placements_named_by_unapplied_journal_records
+                .values()
+                .flatten()
+                .copied()
+                .collect::<Vec<PlacementOnDevice>>(),
             before_the_one_reread,
         )?,
         ShadowLedger::Off => ShadowLedgerComputed {
@@ -2305,6 +2472,9 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 .expect("新实例第一次发布的 txg = 环里最大 txg + 1 ≥ 1"),
         ),
     ));
+    allocator.note_placements_named_by_unapplied_journal_records(
+        placements_named_by_unapplied_journal_records,
+    );
     Ok(RebuiltAllocator {
         allocator,
         effective_floor,
@@ -3507,6 +3677,11 @@ fn raise_the_floor_through<Device: BlockDevice>(
     );
     // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
     // 不然那个槽回收之后就发得出去（步 4 / 步 5 代码三方第二轮辩方腿：窄读法要按每次挂载与每次抬 F 的候选集现算）。
+    // 水位之上没施加的记录点名、还没解开的落点（C694 的 C）按新候选集同一次重算：此前靠一条低于新 F 的根豁免的那几个，回收之前补上隔离。
+    let placements_named_by_unapplied_journal_records: Vec<PlacementOnDevice> = allocator
+        .root_ring_occupancy()
+        .map(RootRingOccupancy::placements_named_by_unapplied_journal_records)
+        .unwrap_or_default();
     let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {
         match isolate_slots_referenced_only_by_abandoned_roots(
             devices,
@@ -3516,6 +3691,7 @@ fn raise_the_floor_through<Device: BlockDevice>(
             &|root| abandoned_by_table(root, &table),
             new_floor,
             &current.allocation_records,
+            &placements_named_by_unapplied_journal_records,
             &mut BeforeTheOneReread::RereadImmediately,
         ) {
             Ok(computed) => computed.abandoned_roots_unreadable,
@@ -5785,6 +5961,64 @@ struct ReadStage {
     journal: JournalScanReport,
     effective_root: RootRecord,
     against_the_witness: SelectedVersionAgainstTheWitness,
+    /// 水位之上那几次缺记录、读得出的记录点名的单元又验不过的发布（F1 乙，[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]）。
+    publishes_missing_records_whose_named_units_do_not_verify: Vec<RollbackTarget>,
+}
+
+/// F1 乙（用户 2026-10-07 定「缺记录就拒可写」；判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 6 条）：水位之上、没被施加的
+/// 每一次发布（同一 (实例代号, checkpoint_txg) 的记录），读得出的那几条的「本次发布内序号」不是 1..N 连着（I-8.9：一次发布的序号依次是 1..N），
+/// 或其中没有一条带末条标志，而它们点名的单元又有一个验不过（两份读不出或校验和对不上，与施加前验点名单元同一份判法
+/// `recovery::every_named_unit_of_the_record_verifies`）——交回这样的发布，按 (实例代号, checkpoint_txg) 升序。
+/// 缺的那条记录点名了哪几个单元挂载时没有别的东西记着（C 隔离不到，判决第二节 K2「C 的盲区」那一行），而那次发布可能已落过根、
+/// 可能被下一次恢复择到：调用方重读一次仍这样就在取号之前拒可写。正常崩溃在写记录中途（单元在记录之前落盘，D16（发布语义） 已定项 7）
+/// 读得出的记录点名的单元验得过，不交回。水位与 C 同一个（`placements_named_by_unapplied_journal_records`）。
+fn publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify<
+    Reader: PoolReader + ?Sized,
+>(
+    reader: &Reader,
+    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
+    selected_version: &RootRecord,
+) -> Vec<RollbackTarget> {
+    let mut records_of_each_publish: BTreeMap<
+        (InstanceGeneration, CheckpointTxg),
+        Vec<&crate::journal::JournalRecord>,
+    > = BTreeMap::new();
+    for record in records.values().filter(|record| {
+        (record.instance, record.checkpoint_txg)
+            > (selected_version.instance, selected_version.checkpoint_txg)
+    }) {
+        records_of_each_publish
+            .entry((record.instance, record.checkpoint_txg))
+            .or_default()
+            .push(record);
+    }
+    records_of_each_publish
+        .into_iter()
+        .filter(|(_, records_of_the_publish)| {
+            let mut ordinals: Vec<u32> = records_of_the_publish
+                .iter()
+                .map(|record| record.ordinal_within_publish.0)
+                .collect();
+            ordinals.sort_unstable();
+            let ordinals_run_from_one_without_a_gap = ordinals
+                .iter()
+                .zip(1..)
+                .all(|(ordinal, expected)| *ordinal == expected);
+            let carries_a_last_record = records_of_the_publish.iter().any(|record| {
+                record.place_in_publish
+                    == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
+            });
+            let some_named_unit_does_not_verify = records_of_the_publish.iter().any(|record| {
+                !crate::recovery::every_named_unit_of_the_record_verifies(reader, record)
+            });
+            !(ordinals_run_from_one_without_a_gap && carries_a_last_record)
+                && some_named_unit_does_not_verify
+        })
+        .map(|((instance, checkpoint_txg), _)| RollbackTarget {
+            instance,
+            checkpoint_txg,
+        })
+        .collect()
 }
 
 /// 判据 N-配置（E158 第 3 次跑登记第 347 行）的读数：系统配置槽直接读盘（`system_configuration_slots_of_one_device_read_once`，每块盘两槽），
@@ -5877,6 +6111,12 @@ fn read_stage<Reader: PoolReader>(
         true,
     )?;
     let witness = newer_publish_witness(devices, system_configuration, &effective_root, &records);
+    let publishes_missing_records_whose_named_units_do_not_verify =
+        publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(
+            cache,
+            &records,
+            &effective_root,
+        );
     Ok(ReadStage {
         chosen_root,
         records,
@@ -5889,15 +6129,41 @@ fn read_stage<Reader: PoolReader>(
             },
             witness,
         },
+        publishes_missing_records_whose_named_units_do_not_verify,
     })
 }
 
+/// 一版 `version` 覆盖的最后一条记录（建上一版时拿它当上一版的记录）：所选根覆盖的最后一条记录读得出就是它——同实例、同 checkpoint_txg
+/// 的记录里带「本次发布末条」标志的那一条（D23（journal 的角色与格式） 已定项 14 注 1，读法乙，用户 2026-09-24 定：「那条」按末条标志认；
+/// 一次发布切成多条记录时它是那次发布的末条）。读不出（两份都撕了，或那次发布读得出的几条都不带标志）就拿环里最大 jsn 那条顶着——
+/// 本实例的第一条反向链恒 0、事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。树表 0 条的根用不到记录
+/// （只做过 mkfs 的池环里一条都没有）。
+fn record_standing_for_the_version(
+    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
+    version: &RootRecord,
+) -> Option<crate::journal::JournalRecord> {
+    records
+        .values()
+        .filter(|record| {
+            record.instance == version.instance
+                && record.checkpoint_txg == version.checkpoint_txg
+                && record.place_in_publish
+                    == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
+        })
+        .max_by_key(|record| record.counter)
+        .or_else(|| records.values().max_by_key(|record| record.counter))
+        .cloned()
+}
+
 /// C554 乙（用户 2026-09-27 定，R = 1）：读阶段读一遍，判据 N-配置 为真（系统配置见证过比所选那一版新的发布）就在同一个读缓存上
 /// 重做一遍（重做之前调 `before_the_one_reread`）。哪一遍判假，就交回哪一遍：往下走用它的所选根、记录与所选那一版。
 /// 在 `replay_journal` 交回之后、`rebuild_previous_version` 之前判（E158 第 3 次跑登记 5.1 末段），取号之前。
+/// F1 乙（用户 2026-10-07 定）同一个做法：水位之上有一次缺记录、读得出的记录点名的单元又验不过的发布
+/// （[`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`]）也在同一个读缓存上重做读阶段一遍，仍有就拒。
 ///
 /// # Errors
 /// 重读那一遍仍判真 ⇒ `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`；
+/// 重读那一遍判假而仍有那样一次发布 ⇒ `NewerStateStillUnreadableAfterOneReread(PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify)`；
 /// 任一遍 [`read_stage`] 报的错原样交回。
 fn read_stage_settled_at_most_on_the_one_reread<Reader: PoolReader>(
     devices: &Reader,
@@ -5907,7 +6173,11 @@ fn read_stage_settled_at_most_on_the_one_reread<Reader: PoolReader>(
     let cache = ReadStageCache::empty_over(devices);
     let first = read_stage(&cache, devices, system_configuration)?;
     let first_read = first.against_the_witness;
-    if !first_read.witnesses_a_publish_newer_than_the_selected_version() {
+    if !first_read.witnesses_a_publish_newer_than_the_selected_version()
+        && first
+            .publishes_missing_records_whose_named_units_do_not_verify
+            .is_empty()
+    {
         return Ok((first, ReadStageSettled::OnTheFirstRead { first_read }));
     }
     before_the_one_reread.before_rereading();
@@ -5923,6 +6193,18 @@ fn read_stage_settled_at_most_on_the_one_reread<Reader: PoolReader>(
             ),
         ));
     }
+    if let Some(publish) = reread
+        .publishes_missing_records_whose_named_units_do_not_verify
+        .first()
+    {
+        return Err(MountError::NewerStateStillUnreadableAfterOneReread(
+            Box::new(
+                StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
+                    publish: *publish,
+                },
+            ),
+        ));
+    }
     Ok((
         reread,
         ReadStageSettled::OnTheOneReread {
@@ -6041,6 +6323,7 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
             journal,
             effective_root,
             against_the_witness: _,
+            publishes_missing_records_whose_named_units_do_not_verify: _,
         },
         read_stage_settled,
     ) = read_stage_settled_at_most_on_the_one_reread(
@@ -6048,24 +6331,66 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         &system_configuration,
         &mut before_the_one_reread,
     )?;
-    // 所选根覆盖的最后一条记录读得出就拿它当上一版的记录：同实例、同 checkpoint_txg 的记录里带「本次发布末条」标志的那一条
-    // （D23（journal 的角色与格式） 已定项 14 注 1，读法乙，用户 2026-09-24 定：「那条」按末条标志认；一次发布切成多条记录时它是那次发布的末条）。
-    // 读不出（两份都撕了，或那次发布读得出的几条都不带标志）就拿环里最大 jsn 那条顶着——本实例的第一条反向链恒 0、
-    // 事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。树表 0 条的根用不到记录（只做过 mkfs 的池环里一条都没有）。
-    let own_record = records
-        .values()
-        .filter(|record| {
-            record.instance == effective_root.instance
-                && record.checkpoint_txg == effective_root.checkpoint_txg
-                && record.place_in_publish
-                    == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
-        })
-        .max_by_key(|record| record.counter)
-        .or_else(|| records.values().max_by_key(|record| record.counter))
-        .cloned();
+    let own_record = record_standing_for_the_version(&records, &effective_root);
     let selected_version_journal_position =
         journal_position_of_the_selected_version(&effective_root, own_record.as_ref());
-    let previous = rebuild_previous_version(devices, unit_area_start, &effective_root, own_record)?;
+    // 所选那一版建不出来（走读失败）、系统配置没见证它：退回见证到的那一版建上一版（F2 乙，用户 2026-10-07 定「三处都做」，与恢复、只读挂载
+    // 同一份判法 `recovery::version_witnessed_by_the_system_configuration_to_fall_back_to`）。退过去不重放：所选根与所选那一版都换成它，
+    // 扫环报告里施加的那一截清零（新实例写的上一个实例那一行 W 取 0，与落在一条根上、一条记录都没施加时相同）。
+    // 退过去的那一版也建不出来，照报所选那一版的失败。之后的一切（取号之前逐盘核、写行、影子账与 C）都按退到的那一版。
+    let (chosen_root, effective_root, journal, selected_version_journal_position, previous) =
+        match rebuild_previous_version(devices, unit_area_start, &effective_root, own_record) {
+            Ok(previous) => (
+                chosen_root,
+                effective_root,
+                journal,
+                selected_version_journal_position,
+                previous,
+            ),
+            Err(failure_of_the_selected_version) => {
+                let Some(witnessed_version) =
+                    version_witnessed_by_the_system_configuration_to_fall_back_to(
+                        &*devices,
+                        &system_configuration,
+                        &chosen_root,
+                        &effective_root,
+                        &records,
+                    )
+                else {
+                    return Err(failure_of_the_selected_version);
+                };
+                let record_standing_for_the_witnessed_version =
+                    record_standing_for_the_version(&records, &witnessed_version);
+                let witnessed_version_journal_position = journal_position_of_the_selected_version(
+                    &witnessed_version,
+                    record_standing_for_the_witnessed_version.as_ref(),
+                );
+                match rebuild_previous_version(
+                    devices,
+                    unit_area_start,
+                    &witnessed_version,
+                    record_standing_for_the_witnessed_version,
+                ) {
+                    Ok(previous) => (
+                        witnessed_version,
+                        witnessed_version,
+                        JournalScanReport {
+                            valid_records: journal.valid_records,
+                            above_water: journal.above_water,
+                            prefix_applied: 0,
+                            verification_passed: journal.verification_passed,
+                            verification_failed: journal.verification_failed,
+                            maximum_applied_transaction: 0,
+                        },
+                        witnessed_version_journal_position,
+                        previous,
+                    ),
+                    Err(_failure_of_the_witnessed_version) => {
+                        return Err(failure_of_the_selected_version)
+                    }
+                }
+            }
+        };
     // 环里没有记录时从 1 起（D23（journal 的角色与格式） 已定项 14 第 3 条：计数器全池接着走）。
     let next_counter = records
         .values()
@@ -6085,6 +6410,8 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         &system_configuration,
         &previous,
         first_txg,
+        &records,
+        &effective_root,
         shadow_ledger,
         &mut before_the_one_reread,
     );
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index e9ea14a..6afdc37 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -46,8 +46,9 @@ use crate::recovery::{
     choose_root, choose_system_configuration_naming_devices_without_a_mountable_slot,
     read_mapped_tree_node_via_hint_then_central_mapping, read_mapped_tree_root,
     read_unit_via_locations, replay_journal, scan_journal,
-    tree_table_entries_each_kind_at_most_once, JournalScanReport, MappedTreeNodeClass, PoolReader,
-    RecoveryFailure, TreeTableEntriesRefusal,
+    tree_table_entries_each_kind_at_most_once,
+    version_witnessed_by_the_system_configuration_to_fall_back_to, JournalScanReport,
+    MappedTreeNodeClass, PoolReader, RecoveryFailure, TreeTableEntriesRefusal,
 };
 use crate::root_record::RootRecord;
 use crate::transaction::MultiLevelCodeTwoTree;
@@ -367,6 +368,23 @@ pub fn open_pool_for_read(
     let filesystem_identifier_in_unit_headers =
         unit_filesystem_identifier(&root.filesystem_identifier);
 
+    // 映射根是空指针（mkfs 的第 0 代、暖机根与写行根这几条树表 0 条的根）时先判树表 0 条（C695 甲，用户 2026-10-07 定「先修次序」，
+    // 判决 `research/prompts/m3-c694-r1-main-verification.md` 第二节 K3 那一行）：先读空映射根会把一个两盘都在、树表 0 条的版本报成
+    // 槽 0 读不出。树表有条目而映射根是空指针的，照下面的次序往下走（读映射那一步判红）。
+    if root.mapping_root == NodePointer::empty_root() {
+        let tree_table_bytes =
+            read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
+        let tree_table = parse_index_node(&tree_table_bytes).map_err(|error| {
+            OpenPoolForReadFailure::UnitMalformed {
+                what: "树表单元",
+                error,
+            }
+        })?;
+        if tree_table.entries.is_empty() {
+            return Err(OpenPoolForReadFailure::TreeTableHasNoEntries);
+        }
+    }
+
     // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）；它的节点都不进映射（自举豁免），只按父指针里的位置条目读。
     // 它不进树表，是哪棵树由根记录里它那条根指针的出生树说（第一个文件版本那次从水位发的号）。
     // 多层时整棵读进挂载态（D19（块指针的结构与宽度预算） 已定项 5「挂载态怎么读映射」）：之后解引用查的是这份内存里的条目，
@@ -567,7 +585,9 @@ pub struct MountedReadOnly {
 }
 
 /// 只读挂载：择系统配置 → 择根 → 全环扫描 journal → 按前缀口径施加 → 沿施加之后的根打开挂载态。
-/// 一个字节都不写（并行线二「这条线不写盘」）。
+/// 一个字节都不写（并行线二「这条线不写盘」）。施加之后那一版打不开（树表 0 条除外）、系统配置没见证它时，退回见证到的那一版打开
+/// （F2 乙，[`version_witnessed_by_the_system_configuration_to_fall_back_to`]）：`chosen_root` 与 `effective_root` 都是退到的那一条，
+/// `journal` 仍是对原来所选根扫环、施加那一趟的报告。
 ///
 /// **扫环只在这一步发生一次**：之后每一次 [`OpenFileForRead::read_at`] 都不碰 journal 环。
 /// 层 0 的每个恢复结果上也能跑这一条——`reader` 换成那个崩溃镜像即可（并行线二验收第 3 条；
@@ -591,8 +611,38 @@ pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, Mount
         true,
     )
     .map_err(MountReadOnlyFailure::Recovery)?;
-    let mounted =
-        open_pool_for_read(reader, &effective_root).map_err(MountReadOnlyFailure::Open)?;
+    let (mounted, chosen_root, effective_root) = match open_pool_for_read(reader, &effective_root) {
+        Ok(mounted) => (mounted, chosen_root, effective_root),
+        // 树表 0 条不是走读失败（这一版还没发布过文件版本），不退。
+        Err(OpenPoolForReadFailure::TreeTableHasNoEntries) => {
+            return Err(MountReadOnlyFailure::Open(
+                OpenPoolForReadFailure::TreeTableHasNoEntries,
+            ));
+        }
+        // 所选那一版走读失败、系统配置没见证它：退回见证到的那一版（F2 乙，与恢复、可写挂载同一份判法，
+        // `recovery::version_witnessed_by_the_system_configuration_to_fall_back_to`），不重放、直接打开；打不开照报所选那一版的失败。
+        Err(failure_of_the_selected_version) => {
+            let witnessed_version_opened =
+                version_witnessed_by_the_system_configuration_to_fall_back_to(
+                    reader,
+                    &system_configuration,
+                    &chosen_root,
+                    &effective_root,
+                    &records,
+                )
+                .and_then(|witnessed_version| {
+                    open_pool_for_read(reader, &witnessed_version)
+                        .ok()
+                        .map(|mounted| (mounted, witnessed_version))
+                });
+            match witnessed_version_opened {
+                Some((mounted, witnessed_version)) => {
+                    (mounted, witnessed_version, witnessed_version)
+                }
+                None => return Err(MountReadOnlyFailure::Open(failure_of_the_selected_version)),
+            }
+        }
+    };
     Ok(MountedReadOnly {
         mounted,
         chosen_root,
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index d354f0d..08e2274 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -12,11 +12,11 @@ use std::cell::OnceCell;
 use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{
-    ACCOUNTING_ENTRY_BYTES, DATA_UNIT_BYTES, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
-    INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT,
-    JOURNAL_SAFETY_FACTOR, LINEAGE_RECORD_BYTES, MAPPING_ENTRY_BYTES, NODE_BYTES,
-    ROOT_RECORD_BYTES, ROOT_RING_REGIONS, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
-    SYSTEM_CONFIGURATION_SLOT_BYTES, TOMBSTONE_RECORD_BYTES,
+    ACCOUNTING_ENTRY_BYTES, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES,
+    FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES,
+    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, JOURNAL_SAFETY_FACTOR, LINEAGE_RECORD_BYTES,
+    MAPPING_ENTRY_BYTES, NODE_BYTES, ROOT_RECORD_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
+    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES, TOMBSTONE_RECORD_BYTES,
 };
 use singlefs_replay_prefix::{
     choose_root_slot, first_copy_of_each_record_key, replay_core, CoreFailure, CoreRecord,
@@ -366,6 +366,15 @@ fn unit_bytes_for_class(unit_class: u8) -> Option<usize> {
     }
 }
 
+/// 一条记录点名的一个单元占几个槽（单元种类定的字节数 ÷ 16384）：可写挂载隔离水位之上没施加的记录点名的落点按它
+/// （`mount::placements_named_by_unapplied_journal_records`）。种类不认识的那条点名项（坏盘上的记录：施加前逐项验点名单元时它本来就验不过）
+/// 按最宽的那一档（数据单元的两槽）算，隔离只会多不会少。
+pub(crate) fn span_in_slots_of_a_named_unit(named: &crate::journal::NamedUnit) -> u64 {
+    let unit_bytes = unit_bytes_for_class(named.unit_class)
+        .unwrap_or(usize::try_from(DATA_UNIT_BYTES).expect("32768"));
+    u64::try_from(unit_bytes).expect("单元字节数装得进 u64") / SLOT_BYTES
+}
+
 /// 按指针里的位置条目读一个单元：逐条试，整单元 CRC-32C 等于条目里的校验和才算读到。
 /// 挂载态的读（`crate::mounted_read`）打开时走同一条，不另写一份。
 pub(crate) fn read_unit_via_locations<Reader: PoolReader + ?Sized>(
@@ -2835,6 +2844,8 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
     )?;
     let device_identities = reader.device_identities();
     allocation_records_are_one_placement_per_device_on_every_device(
+        reader,
+        unit_area_start,
         &allocation_tree_read.records,
         &device_identities,
     )?;
@@ -3509,6 +3520,29 @@ enum NewRootSource {
 pub const CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR: &str =
     "所选根的 checkpoint_txg 是 u64 的最大值：接在它后面的那次发布取不到 txg";
 
+/// 一条记录点名的每个单元的两份（两条位置条目）都读得出、整单元 CRC-32C 都等于条目里的（前缀第四条「两份都验过才施加」，
+/// D23（journal 的角色与格式） 已定项 14）；点名项的单元种类不认识就算验不过。施加前验点名单元（[`replay_journal`]）与可写挂载判
+/// 「水位之上一次缺记录的发布点名的单元验不过」（`mount::publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`）同用这一份。
+pub(crate) fn every_named_unit_of_the_record_verifies<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    record: &JournalRecord,
+) -> bool {
+    record.named.iter().all(|named| {
+        let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
+            return false;
+        };
+        named.locations.iter().all(|location| {
+            reader
+                .read(
+                    location.device,
+                    location.slot.to_device_offset(),
+                    unit_bytes,
+                )
+                .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
+        })
+    })
+}
+
 /// 取前缀并施加（D23（journal 的角色与格式） 已定项 14 / 已定项 15）；返回扫描报告与施加之后的根。
 ///
 /// 链首锚在所选根覆盖的最后一条：所选根那次发布里带「本次发布末条」标志的那一条（已定项 14 注 1，读法乙；
@@ -3596,22 +3630,8 @@ pub fn replay_journal(
             BackChainJudgement::Holds | BackChainJudgement::PreviousRecordNotOnDisk => false,
             BackChainJudgement::Broken => true,
         };
-    let named_units_verified = |index: usize| {
-        records_in_key_order[index].named.iter().all(|named| {
-            let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
-                return false;
-            };
-            named.locations.iter().all(|location| {
-                reader
-                    .read(
-                        location.device,
-                        location.slot.to_device_offset(),
-                        unit_bytes,
-                    )
-                    .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
-            })
-        })
-    };
+    let named_units_verified =
+        |index: usize| every_named_unit_of_the_record_verifies(reader, records_in_key_order[index]);
     let named_unit_verification = if verify_named_units {
         NamedUnitVerification::Verify
     } else {
@@ -4280,12 +4300,14 @@ pub(crate) fn extents_of_the_first_file(
 }
 
 /// 分配记录「每个落点每盘各一条」（两盘同槽、同一批字段）：同一块盘上一个槽只许一条记录，每块盘各自的（槽, 跨度, 代, 已释放）集合相同，
-/// 每盘不少于 `NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE` 个落点（下限怎么来的见那个常量）。同盘同槽两条记录（代不同）在集合里是两个元素、两盘对称就过——第二轮攻方腿打中，
+/// 每盘不少于 `placements_on_each_device_at_least` 个落点（走读传的是这套几何下新池新建文件之后每盘的落点数，
+/// [`placements_on_each_device_after_the_new_pool_file_creation`]）。同盘同槽两条记录（代不同）在集合里是两个元素、两盘对称就过——第二轮攻方腿打中，
 /// 走读自己不判 key 严格递增，这里逐盘核槽号不重复。
 #[must_use]
 pub fn allocation_records_are_one_per_device(
     records: &[AllocationRecord],
     device_identities: &[DeviceIdentity],
+    placements_on_each_device_at_least: PlacementsOnEachDevice,
 ) -> bool {
     let mut placements_per_device: BTreeMap<DeviceIdentity, BTreeSet<(u64, u16, u64, bool)>> =
         BTreeMap::new();
@@ -4317,16 +4339,115 @@ pub fn allocation_records_are_one_per_device(
     }
     let mut placement_sets = placements_per_device.values();
     let first_device_placements = placement_sets.next().expect("上面核过每块盘都有记录");
-    first_device_placements.len() >= NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE
+    first_device_placements.len() >= placements_on_each_device_at_least.0
         && placement_sets.all(|placements| placements == first_device_placements)
 }
 
-/// 每盘落点数的下限：mkfs 写在单元区里的 2 个落点，加新池新建文件只写 8 个单元时的 8 个落点。
-/// 新池新建文件在默认几何下写 12 个落点（字节表五：28 条记录，每盘 14 条；提交内生块个数随几何变），之后每次发布只多不少，
-/// 所以这条下限比盘上实有的松。
-/// TODO：下限没随分配记录树按位置寻址（D8（核心索引结构） 已定项 14）抬上去：内生块个数随几何变，取各几何的最小值还是按几何现算，没有条款；
-/// 欠账在 `.claude/kb/checks-owed.md` C692（新池新建文件每盘落点下限偏松），定下取法之后照那一行改常量与它的单测。
-const NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE: usize = 10;
+/// 每块盘上的落点数（分配记录条数按盘数）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct PlacementsOnEachDevice(pub usize);
+
+/// 新池新建文件之后每块盘上的落点数，按这个池的几何现算（C692 取 G2「按几何现算」，用户 2026-10-07 定）：分配记录每盘落点数的下限
+/// （[`allocation_records_are_one_per_device`]），之后每次发布只多不少。几何是每块盘的字节数（`device_sizes_in_bytes`，分配记录树的根层级
+/// 由它定，[`AllocationRecordTreeGeometry::of_device_sizes_in_bytes`]）与单元区起点（系统配置里的环长现算的，`unit_area_start`）。
+/// 算法是在一个空分配器上照新池新建文件那一版分一遍落点（D3（空间分配） 已定项 8 / 已定项 10 的分配器本身，不另写一份落点规则）：
+/// mkfs 写在单元区里的两个单元（实例表第 0 片在单元区起点、两槽，第 0 版树表紧跟其后、一槽，与 `make_filesystem` 同一个摆法），
+/// 一个数据单元（第一个文件至少一个数据单元，`write_request_split::data_unit_count_of_a_sequential_write`），再照提交内生块的 bump 次序
+/// （D3（空间分配） 已定项 10 ⑤：按树 ID 升序、树内先叶后根、中央映射树倒数第二、树表单元最末）分 extent 树根、inode 叶容器、inode 树根、
+/// 分配记录树的节点、记账树根、中央映射树根、树表单元。分配记录树要几个节点取决于这些记录落在几片叶里（按位置寻址，D8（核心索引结构） 已定项 14），
+/// 而节点自己的落点也有记录：节点数从「每块盘一条根到叶的路径加根」起，按分完之后的记录现算的节点数调，到两者相等为止。
+/// 默认几何（4 GiB 盘、默认环）每盘 14（字节表五：28 条分配记录）；1024 MiB 环那套几何（单元区起点 66560）提交内生块多两个、每盘 16。
+///
+/// # Panics
+/// 空分配器上分不出这几个落点（单元区装不下新池新建文件那十几个单元）：mkfs 判过单元区够不够（`UnitAreaBeyondDevice`），
+/// 能挂、能走读的池都装得下；或节点数调了 [`ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS`] 轮还不相等。
+#[must_use]
+pub fn placements_on_each_device_after_the_new_pool_file_creation(
+    device_sizes_in_bytes: &[(DeviceIdentity, u64)],
+    unit_area_start: UnitAreaStart,
+) -> PlacementsOnEachDevice {
+    let geometry = AllocationRecordTreeGeometry::of_device_sizes_in_bytes(device_sizes_in_bytes);
+    let new_pool_file_creation_txg = CheckpointTxg(1);
+    // 新池新建文件的落点都在单元区开头的两段里（mkfs 与数据单元在第一段，提交内生块开在最低的全空段、即第二段）：空闲图只建到那两段为止，
+    // 走读每次都调它，不为整块盘建一张图（4 GiB 盘是二十多万槽）。分配记录树的几何仍按整块盘的字节数（上面那一行）。
+    let rehearsal_bytes_on_each_device =
+        (unit_area_start.slot().0 + 2 * CLUSTER_SEGMENT_SLOTS) * SLOT_BYTES;
+    let mut allocator = crate::allocator::PoolAllocator::new(
+        device_sizes_in_bytes
+            .iter()
+            .map(|(device, device_bytes)| {
+                crate::allocator::DeviceFreeMap::with_unit_area_start(
+                    *device,
+                    (*device_bytes).min(rehearsal_bytes_on_each_device),
+                    unit_area_start,
+                )
+            })
+            .collect(),
+    );
+    let instance_table_span = TransactionUnit::InstanceTable.span_slots();
+    allocator.mark_format_time_units(
+        crate::allocator::Placement {
+            slot: unit_area_start.slot(),
+            span: instance_table_span,
+        },
+        crate::allocator::Placement {
+            slot: SlotNumber(unit_area_start.slot().0 + instance_table_span),
+            span: TransactionUnit::TreeTable.span_slots(),
+        },
+    );
+    let allocate = |rehearsing: &mut crate::allocator::PoolAllocator, unit: TransactionUnit| {
+        let placement = match unit.placement() {
+            crate::transaction::PlacementRule::UserData => {
+                rehearsing.allocate_user_data(new_pool_file_creation_txg)
+            }
+            crate::transaction::PlacementRule::CommitGenerated(footprint) => {
+                rehearsing.allocate_commit_generated(footprint, new_pool_file_creation_txg)
+            }
+        };
+        placement.expect(
+            "空分配器上分得出新池新建文件那十几个单元：mkfs 判过单元区装得下（UnitAreaBeyondDevice），能挂、能走读的池都装得下",
+        );
+    };
+    for unit in [
+        TransactionUnit::Data(DataUnitIndexInFile(0)),
+        TransactionUnit::ExtentRoot,
+        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
+        TransactionUnit::InodeRoot,
+    ] {
+        allocate(&mut allocator, unit);
+    }
+    let units_before_the_allocation_record_tree = allocator.clone();
+    let device_count = device_sizes_in_bytes.len();
+    // 迭代上界 ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS；跨轮只带猜的节点数，每一轮从同一个分配器拷贝重分。
+    let mut allocation_record_tree_nodes = device_count * usize::from(geometry.root_level()) + 1;
+    for _round in 0..ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS {
+        let mut rehearsal = units_before_the_allocation_record_tree.clone();
+        for _node in 0..allocation_record_tree_nodes {
+            allocate(&mut rehearsal, TransactionUnit::AllocationTree);
+        }
+        for unit in [
+            TransactionUnit::AccountingTree,
+            TransactionUnit::MappingTree,
+            TransactionUnit::TreeTable,
+        ] {
+            allocate(&mut rehearsal, unit);
+        }
+        let nodes_holding_the_records =
+            crate::allocation_record_tree::nodes_holding_records(&geometry, rehearsal.records())
+                .len();
+        if nodes_holding_the_records == allocation_record_tree_nodes {
+            return PlacementsOnEachDevice(rehearsal.records().len() / device_count);
+        }
+        allocation_record_tree_nodes = nodes_holding_the_records;
+    }
+    panic!(
+        "分配记录树的节点数调了 {ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS} 轮还不相等：每一轮多分的节点只会让记录多落进一两片叶，几轮就该停"
+    );
+}
+
+/// [`placements_on_each_device_after_the_new_pool_file_creation`] 调分配记录树节点数的轮数上界：猜的节点数每一轮只会因为节点自己的落点
+/// 多落进一片叶而加几个，推的，没量过会不会到第三轮。
+const ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS: usize = 8;
 
 /// 冷走读里的中央映射树（自举豁免，只按父指针里的位置条目读）：到第一次有树根的提示读不出、要经映射回退时，
 /// 或走到 `TreeRoots` 那一步时才读，读过一次就留着。不提前读：提示都读得出的镜像上，走读的读序与判红次序照旧。
@@ -4409,17 +4530,41 @@ fn tree_identifier_of_the_entry_is_below_the_watermark(
     Ok(())
 }
 
-/// 每个落点每盘一条（两盘同槽）：新池新建文件 10 × 盘数，每次覆盖写再加 8 × 盘数（换下的那些改写、不删）。
+/// 每个落点每盘一条（两盘同槽）：新池新建文件之后每盘的落点数按这个池的几何现算（默认几何 14 × 盘数，
+/// [`placements_on_each_device_after_the_new_pool_file_creation`]），之后每次发布只多不少（换下的那些改写、不删）。
 /// 只核总数是盘数的整数倍拦不住「一盘多一条、另一盘少一条」——发布 B 三方第一轮正推腿打中，改成逐盘核同一批（槽, 跨度）
 /// （[`allocation_records_are_one_per_device`]）。
+///
+/// # Panics
+/// 读者列出的盘说不出自己的字节数：每个读者都从同一张表里取（`PoolReader::device_size_in_bytes`）。
 fn allocation_records_are_one_placement_per_device_on_every_device(
+    reader: &dyn PoolReader,
+    unit_area_start: UnitAreaStart,
     records: &[AllocationRecord],
     device_identities: &[DeviceIdentity],
 ) -> Result<(), RecoveryFailure> {
-    if !allocation_records_are_one_per_device(records, device_identities) {
+    let device_sizes_in_bytes: Vec<(DeviceIdentity, u64)> = device_identities
+        .iter()
+        .map(|device| {
+            (
+                *device,
+                reader
+                    .device_size_in_bytes(*device)
+                    .expect("读者列出的盘说得出自己的字节数"),
+            )
+        })
+        .collect();
+    if !allocation_records_are_one_per_device(
+        records,
+        device_identities,
+        placements_on_each_device_after_the_new_pool_file_creation(
+            &device_sizes_in_bytes,
+            unit_area_start,
+        ),
+    ) {
         return Err(RecoveryFailure::InvariantViolated {
             invariant: "E142 走读同款",
-            detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于 10 个落点",
+            detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于这套几何下新池新建文件之后每盘的落点数",
         });
     }
     Ok(())
@@ -5010,6 +5155,8 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
     allocation_records_fit_the_pool_geometry(reader, unit_area_start, &roots.allocation.records)?;
     let allocation_records: &[AllocationRecord] = &roots.allocation.records;
     allocation_records_are_one_placement_per_device_on_every_device(
+        reader,
+        unit_area_start,
         allocation_records,
         &device_identities,
     )?;
@@ -5150,7 +5297,113 @@ pub fn rebuild_version(
     )
 }
 
-/// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。
+/// 所选那一版走读失败之后退回哪一版（F2 乙，用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md`
+/// 第三节第 7 条）：系统配置没见证过所选那一版时，退回根环里比所选根旧、不比系统配置见证到的那一次发布新的最新一条根；见证过、见证判不出、
+/// 或找不到那样一条根时 `None`（照报原来的失败）。恢复（[`recover`]）、可写挂载读阶段之后建上一版那一步（`mount`）、只读挂载择根
+/// （`mounted_read::mount_read_only`）三处都调这一份，退过去之后不重放、直接走那一版。所选那一版是施加过记录、由记录重建的
+/// （`selected_version` 不是 `chosen_root`）时不退：条款写的是「所选根走读失败」，施加过记录的那一版走不通条款没写（函数体第一句）。
+///
+/// 「见证过」的判法取攻方 `strict`（判决同一条）：c_见证 = 每块盘两槽里自证过的系统配置 journal tail 的最大值（与可写挂载判据 N-配置
+/// 同一个读数，`mount::newer_publish_witness`）。所选那一版那次发布带末条标志的记录读得出（任一份）⇒ 它的计数器 ≤ c_见证 即见证过；
+/// 读不出而计数器 = c_见证 的那条记录读得出 ⇒ 它的 (checkpoint_txg, 实例代号) 不比所选那一版旧即见证过；两条都读不出当见证过、不退。
+/// 退到的上限：计数器 = c_见证 的那条记录读得出时取它的 (checkpoint_txg, 实例代号)，读不出时取所选根的。根环里的根按择根同一个序
+/// （checkpoint_txg 为主、实例代号破平局）比。
+///
+/// 系统配置见证判不出（R2：有槽设备报读错、又没有一块盘两槽都自证过，[`system_configuration_witness_is_undecidable`]）时不退：
+/// 条款只写了「系统配置没见证它时退」，判不出算不算没见证没写；这一支照报原来的失败（与今天相同），交主 agent 定。
+///
+/// 代价（判决同一条，用户认下）：已确认的一版在较新的系统配置槽与它的单元同时读回全零时，c_见证 落回它之前，读回的是上一版。
+#[must_use]
+pub fn version_witnessed_by_the_system_configuration_to_fall_back_to<
+    Reader: PoolReader + ?Sized,
+>(
+    reader: &Reader,
+    system_configuration: &SystemConfiguration,
+    chosen_root: &RootRecord,
+    selected_version: &RootRecord,
+    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
+) -> Option<RootRecord> {
+    // 条款写的是「所选根走读失败」：只在走读失败的就是所选根本身（一条记录都没施加）时退。施加过记录、由记录重建的那一版走读失败不退：
+    // 施加之前它点名的单元两份都验过（前缀第四条），走不通说明记录或单元的内容坏了，不是被后来的写盖掉；那一支条款没写，照报原来的失败（与今天相同），交主 agent 定。
+    if (selected_version.instance, selected_version.checkpoint_txg)
+        != (chosen_root.instance, chosen_root.checkpoint_txg)
+    {
+        return None;
+    }
+    let slot_spacing_in_bytes = u64::from(
+        system_configuration
+            .immutable
+            .sizes
+            .fixed_structure_slot_spacing,
+    );
+    let slots_of_each_device: Vec<SystemConfigurationSlotsOfOneDevice> = reader
+        .device_identities()
+        .into_iter()
+        .map(|device| {
+            system_configuration_slots_of_one_device_read_once(
+                reader,
+                device,
+                slot_spacing_in_bytes,
+                &system_configuration.immutable.filesystem_identifier,
+            )
+        })
+        .collect();
+    if system_configuration_witness_is_undecidable(&slots_of_each_device) {
+        return None;
+    }
+    let witnessed_journal_counter = slots_of_each_device
+        .iter()
+        .flat_map(|slots_of_this_device| slots_of_this_device.self_verified.iter())
+        .map(|slot| slot.quantities.journal_tail)
+        .max()
+        .unwrap_or(0);
+    let order_of_the_root_choice = |root: &RootRecord| (root.checkpoint_txg, root.instance);
+    let last_record_counter_of_the_selected_version = records
+        .values()
+        .filter(|record| {
+            record.instance == selected_version.instance
+                && record.checkpoint_txg == selected_version.checkpoint_txg
+                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
+        })
+        .map(|record| record.counter)
+        .max();
+    let record_at_the_witnessed_counter = records
+        .values()
+        .filter(|record| record.counter == witnessed_journal_counter)
+        .map(|record| (record.checkpoint_txg, record.instance))
+        .max();
+    let is_witnessed = match (
+        last_record_counter_of_the_selected_version,
+        record_at_the_witnessed_counter,
+    ) {
+        (Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter,
+        (None, Some(witnessed_publish)) => {
+            witnessed_publish >= order_of_the_root_choice(selected_version)
+        }
+        (None, None) => true,
+    };
+    if is_witnessed {
+        return None;
+    }
+    let chosen = order_of_the_root_choice(chosen_root);
+    let ceiling = record_at_the_witnessed_counter.unwrap_or(chosen);
+    readable_roots(
+        reader,
+        &system_configuration.immutable.region_devices,
+        &system_configuration.immutable.sizes,
+        &system_configuration.immutable.filesystem_identifier,
+    )
+    .into_iter()
+    .filter(|root| {
+        let order = order_of_the_root_choice(root);
+        order < chosen && order <= ceiling
+    })
+    .max_by_key(order_of_the_root_choice)
+}
+
+/// 整条恢复路径。报出去的 `root` 恒是所选的那条根；施加记录之后走的是重建出来的根。所选那一版走读失败、系统配置没见证它时退回见证到的那一版
+/// （[`version_witnessed_by_the_system_configuration_to_fall_back_to`]）：那一版走得通，`root` 与 `effective_root` 都报退到的那一条，
+/// `journal` 仍是对所选根扫环、施加那一趟的报告；走不通照报所选那一版的失败。不看 journal 的那一臂（`JournalPolicy::Ignore`）不退。
 #[must_use]
 pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {
     let mut mapping_fallbacks = 0;
@@ -5215,21 +5468,68 @@ pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport
         }
     };
     let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
-    let outcome = match walk_to_file_in_the_unit_area_starting_at(
+    let (outcome, effective_root) = match walk_to_file_in_the_unit_area_starting_at(
         reader,
         unit_area_start,
         &effective_root,
         &mut mapping_fallbacks,
     ) {
-        Ok(Some(content)) => RecoveryOutcome::FileRead {
-            root: root_key,
-            content,
-        },
-        Ok(None) => RecoveryOutcome::NoFile { root: root_key },
-        Err(failure) => RecoveryOutcome::Failed {
-            root: Some(root_key),
-            failure,
-        },
+        Ok(Some(content)) => (
+            RecoveryOutcome::FileRead {
+                root: root_key,
+                content,
+            },
+            effective_root,
+        ),
+        Ok(None) => (RecoveryOutcome::NoFile { root: root_key }, effective_root),
+        // 走读失败才再扫一遍环拿记录给退回的判法（只读、不写盘；走读做成的恢复不多扫）。
+        Err(failure) => {
+            let witnessed_version_walked = match policy {
+                JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {
+                    version_witnessed_by_the_system_configuration_to_fall_back_to(
+                        reader,
+                        &system_configuration,
+                        &root,
+                        &effective_root,
+                        &scan_journal(reader, &system_configuration),
+                    )
+                }
+                JournalPolicy::Ignore => None,
+            }
+            .and_then(|witnessed_version| {
+                walk_to_file_in_the_unit_area_starting_at(
+                    reader,
+                    unit_area_start,
+                    &witnessed_version,
+                    &mut mapping_fallbacks,
+                )
+                .ok()
+                .map(|content| (witnessed_version, content))
+            });
+            match witnessed_version_walked {
+                Some((witnessed_version, content)) => {
+                    let witnessed_key =
+                        (witnessed_version.instance, witnessed_version.checkpoint_txg);
+                    let outcome = match content {
+                        Some(content) => RecoveryOutcome::FileRead {
+                            root: witnessed_key,
+                            content,
+                        },
+                        None => RecoveryOutcome::NoFile {
+                            root: witnessed_key,
+                        },
+                    };
+                    (outcome, witnessed_version)
+                }
+                None => (
+                    RecoveryOutcome::Failed {
+                        root: Some(root_key),
+                        failure,
+                    },
+                    effective_root,
+                ),
+            }
+        }
     };
     RecoveryReport {
         outcome,
@@ -5243,13 +5543,38 @@ pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport
 #[cfg(test)]
 mod allocation_records_per_device_tests {
     use super::{
-        allocation_records_are_one_per_device, NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE,
+        allocation_records_are_one_per_device,
+        placements_on_each_device_after_the_new_pool_file_creation, PlacementsOnEachDevice,
     };
     use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
-    use crate::allocator::AllocationRecord;
+    use crate::allocator::{AllocationRecord, UnitAreaStart};
 
     const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
 
+    /// 默认几何与 1024 MiB 环那套几何的盘都是 4 GiB（`.claude/kb/layout/01-first-txn.md` 字节表五；
+    /// `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的一 GiB 环那一条）。
+    const FOUR_GIBIBYTES: u64 = 4 << 30;
+    const ONE_GIBIBYTE_JOURNAL_RING_BYTES: u64 = 1 << 30;
+
+    fn both_devices_of_four_gibibytes() -> Vec<(DeviceIdentity, u64)> {
+        BOTH_DEVICES
+            .iter()
+            .map(|device| (*device, FOUR_GIBIBYTES))
+            .collect()
+    }
+
+    fn placements_on_each_device_of_the_default_geometry() -> PlacementsOnEachDevice {
+        placements_on_each_device_after_the_new_pool_file_creation(
+            &both_devices_of_four_gibibytes(),
+            UnitAreaStart::of_the_default_journal_ring(),
+        )
+    }
+
+    fn unit_area_start_after_a_one_gibibyte_journal_ring() -> UnitAreaStart {
+        UnitAreaStart::following_the_journal_ring(ONE_GIBIBYTE_JOURNAL_RING_BYTES)
+            .expect("1024 MiB 环末尾的下一个槽（66560）落在聚簇段边界上")
+    }
+
     fn record(device: DeviceIdentity, slot: u64, is_released: bool) -> AllocationRecord {
         AllocationRecord {
             device,
@@ -5260,11 +5585,14 @@ mod allocation_records_per_device_tests {
         }
     }
 
-    /// 按下限造的形状：每盘恰好 `NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE` 个落点、同一批槽号；已释放的记录照样算一个落点。
-    fn new_pool_file_creation_shape() -> Vec<AllocationRecord> {
+    /// 按下限造的形状：每盘恰好 `placements_on_each_device` 个落点、同一批槽号，从 `first_slot` 起；已释放的记录照样算一个落点。
+    fn shape_with_placements_on_each_device(
+        first_slot: u64,
+        placements_on_each_device: PlacementsOnEachDevice,
+    ) -> Vec<AllocationRecord> {
         let mut records = Vec::new();
-        for placement_index in 0..NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE {
-            let slot = 50176 + 2 * u64::try_from(placement_index).expect("落点序号");
+        for placement_index in 0..placements_on_each_device.0 {
+            let slot = first_slot + 2 * u64::try_from(placement_index).expect("落点序号");
             for device in BOTH_DEVICES {
                 records.push(record(device, slot, placement_index == 0));
             }
@@ -5272,17 +5600,91 @@ mod allocation_records_per_device_tests {
         records
     }
 
+    fn new_pool_file_creation_shape() -> Vec<AllocationRecord> {
+        shape_with_placements_on_each_device(
+            50176,
+            placements_on_each_device_of_the_default_geometry(),
+        )
+    }
+
+    #[test]
+    fn the_default_geometry_has_fourteen_placements_on_each_device_after_the_new_pool_file_creation(
+    ) {
+        assert_eq!(
+            placements_on_each_device_of_the_default_geometry(),
+            PlacementsOnEachDevice(14),
+            "字节表五：28 条分配记录 = m1、m2、t1..t12 十四个单元 × 2 盘"
+        );
+    }
+
     #[test]
-    fn new_pool_file_creation_shape_is_one_record_per_placement_per_device() {
+    fn the_one_gibibyte_journal_ring_geometry_has_sixteen_placements_on_each_device_after_the_new_pool_file_creation(
+    ) {
+        assert_eq!(
+            placements_on_each_device_after_the_new_pool_file_creation(
+                &both_devices_of_four_gibibytes(),
+                unit_area_start_after_a_one_gibibyte_journal_ring(),
+            ),
+            PlacementsOnEachDevice(16),
+            "单元区起点 66560：mkfs 与数据单元在槽 81 那片叶（W = 812），提交内生块在槽 82 那片叶，分配记录树每盘多一片叶、多两个节点，提交内生块 13 个"
+        );
+    }
+
+    #[test]
+    fn exactly_the_placements_of_the_default_geometry_are_accepted_and_one_fewer_is_rejected() {
+        let placements_on_each_device = placements_on_each_device_of_the_default_geometry();
         assert!(allocation_records_are_one_per_device(
             &new_pool_file_creation_shape(),
-            &BOTH_DEVICES
+            &BOTH_DEVICES,
+            placements_on_each_device,
         ));
+        let one_fewer = shape_with_placements_on_each_device(
+            50176,
+            PlacementsOnEachDevice(placements_on_each_device.0 - 1),
+        );
+        assert!(
+            !allocation_records_are_one_per_device(
+                &one_fewer,
+                &BOTH_DEVICES,
+                placements_on_each_device
+            ),
+            "每盘比新池新建文件少一个落点：判红"
+        );
+    }
+
+    #[test]
+    fn exactly_the_placements_of_the_one_gibibyte_journal_ring_geometry_are_accepted_and_one_fewer_is_rejected(
+    ) {
+        let unit_area_start = unit_area_start_after_a_one_gibibyte_journal_ring();
+        let placements_on_each_device = placements_on_each_device_after_the_new_pool_file_creation(
+            &both_devices_of_four_gibibytes(),
+            unit_area_start,
+        );
+        assert!(allocation_records_are_one_per_device(
+            &shape_with_placements_on_each_device(
+                unit_area_start.slot().0,
+                placements_on_each_device
+            ),
+            &BOTH_DEVICES,
+            placements_on_each_device,
+        ));
+        let one_fewer = shape_with_placements_on_each_device(
+            unit_area_start.slot().0,
+            PlacementsOnEachDevice(placements_on_each_device.0 - 1),
+        );
+        assert!(
+            !allocation_records_are_one_per_device(
+                &one_fewer,
+                &BOTH_DEVICES,
+                placements_on_each_device
+            ),
+            "每盘比这套几何下的新池新建文件少一个落点：判红（默认几何的 14 放得过它）"
+        );
     }
 
     #[test]
     fn placement_recorded_on_one_device_only_is_rejected_even_when_the_total_is_even() {
-        // 盘 0 多一条、盘 1 少一条：总数仍是 20，旧的「总数是盘数的整数倍」判定放过它。
+        // 盘 0 多一条、盘 1 少一条：总数仍是盘数的整数倍，旧的「总数是盘数的整数倍」判定放过它。
         let mut records = new_pool_file_creation_shape();
         let moved = records
             .iter()
@@ -5292,11 +5694,12 @@ mod allocation_records_per_device_tests {
         records[moved].slot = SlotNumber(50300);
         assert_eq!(
             records.len(),
-            2 * NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE
+            2 * placements_on_each_device_of_the_default_geometry().0
         );
         assert!(!allocation_records_are_one_per_device(
             &records,
-            &BOTH_DEVICES
+            &BOTH_DEVICES,
+            placements_on_each_device_of_the_default_geometry(),
         ));
     }
 
@@ -5306,7 +5709,8 @@ mod allocation_records_per_device_tests {
         records.push(record(DeviceIdentity(2), 50176, false));
         assert!(!allocation_records_are_one_per_device(
             &records,
-            &BOTH_DEVICES
+            &BOTH_DEVICES,
+            placements_on_each_device_of_the_default_geometry(),
         ));
         let only_device_zero: Vec<AllocationRecord> = new_pool_file_creation_shape()
             .into_iter()
@@ -5314,7 +5718,8 @@ mod allocation_records_per_device_tests {
             .collect();
         assert!(!allocation_records_are_one_per_device(
             &only_device_zero,
-            &BOTH_DEVICES
+            &BOTH_DEVICES,
+            placements_on_each_device_of_the_default_geometry(),
         ));
     }
 
@@ -5329,7 +5734,8 @@ mod allocation_records_per_device_tests {
         records[released_on_device_one].is_released = false;
         assert!(!allocation_records_are_one_per_device(
             &records,
-            &BOTH_DEVICES
+            &BOTH_DEVICES,
+            placements_on_each_device_of_the_default_geometry(),
         ));
     }
 
@@ -5344,19 +5750,8 @@ mod allocation_records_per_device_tests {
         }
         assert!(!allocation_records_are_one_per_device(
             &records,
-            &BOTH_DEVICES
-        ));
-    }
-
-    #[test]
-    fn fewer_than_the_new_pool_file_creation_placements_is_rejected() {
-        let records: Vec<AllocationRecord> = new_pool_file_creation_shape()
-            .into_iter()
-            .filter(|record| record.slot != SlotNumber(50176))
-            .collect();
-        assert!(!allocation_records_are_one_per_device(
-            &records,
-            &BOTH_DEVICES
+            &BOTH_DEVICES,
+            placements_on_each_device_of_the_default_geometry(),
         ));
     }
 }
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index 23655c1..3224776 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -3103,6 +3103,9 @@ fn still_unreadable_after_one_reread_member(
             "RootRingSlotKnownToHoldARoot"
         }
         StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => "AccountOfAnAbandonedRoot",
+        StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
+            "PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify"
+        }
     }
 }
 
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index 5cc7255..c71ab6e 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -837,7 +837,9 @@ pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReaso
                 // 改之前经 `MountError::Recovery` 交出、同样记成没理由）：只在读路径上注入故障才有，模型没有这一条。
                 | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                 | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
-                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => {
+                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. }
+                // 水位之上一次缺记录、读得出的记录点名的单元又验不过的发布（F1 乙）：随机历史不藏 journal 记录，模型没有这一条。
+                | StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => {
                     ObservedRefusalReason::Unexplained
                 }
             }
@@ -944,7 +946,8 @@ pub fn root_ring_slot_still_bad_after_one_reread_of_rollback_error(
                 StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. }
                 | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. }
                 | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
-                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => None,
+                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. }
+                | StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => None,
             }
         }
         RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index ef1bd70..cf253ce 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -2246,7 +2246,9 @@ pub fn refusal_reason_on_the_d23_list(error: &MountError) -> Option<RefusalReaso
                 // 挂着时抬 F、管理员回退那几处读坏，与影子账读一条被抛弃根的账读不出（C393）：不在 D23 已定项 14 那张单子上。
                 StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                 | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
-                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => None,
+                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. }
+                // F1 乙（用户 2026-10-07 定）那一拒：D23 已定项 14 那张单子还没写回它（kb 写回在主 agent），这里照「不在单子上」判。
+                | StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => None,
             }
         }
         MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
diff --git a/crates/singlefs-harness/tests/abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs b/crates/singlefs-harness/tests/abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs
new file mode 100644
index 0000000..821d8cb
--- /dev/null
+++ b/crates/singlefs-harness/tests/abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs
@@ -0,0 +1,228 @@
+//! C694 的 B（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 2 条，用户 2026-10-07 定「C 与 B 都做」）：影子账读一条被抛弃根的账
+//! 读不出、从中央映射也现算不成、重读一次仍不成时（`mount::isolate_slots_referenced_only_by_abandoned_roots` 那一支，改之前一律拒），
+//! 内存里的现行账有一条分配代大于那条根的 txg、罩住它树表槽（与中央映射树根槽）的记录就不再拒；凭据只读现行账，不读那一槽。
+//!
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次覆盖写 R，崩在 R 的根槽落盘之后、
+//! 系统配置轮换之前。
+//! - 已被占的那一形：取新实例的那次可写挂载 M1 里 R 的根槽与单元读回全零，M1 关着影子账（只供测试的开关 `ShadowLedger::Off`，连 C 一起关：
+//!   造出 C 罩不到的那一形——R 的记录也读回全零时 M1 写行那一条根就盖掉 R 的根槽，造不出被抛弃的 R），M1 与之后的覆盖写把 R 的树表槽与
+//!   映射根槽重新分配、写成别的单元（现行账里有那两条记录）；之后 R 的根槽读得出、按实例表被抛弃，账与映射都读不出。三条路都开着影子账。
+//! - 瞬时读不出的那一形（同一个崩溃盘面）：M1 里只有 R 的根槽与单元读回全零（`ur`），C 隔离了 R 的落点、谁都没碰它们；卸载那一刻 R 的树表槽与
+//!   映射根槽读回全零（一次挂载内两遍都是），现行账里没有罩住它们的记录：照拒。
+mod common_unwitnessed_publish;
+
+use common_unwitnessed_publish::{
+    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero, plain_devices_on,
+    pool_after_the_mount, record_one_publish_after_overwrites, PublishRecordedAsR,
+    RangesReadingBackAllZero, RecordedPublish, OVERWRITE_BYTES,
+};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::allocator::AllocationRecord;
+use singlefs_core::mount::{
+    mount_writable, mount_writable_with_test_only_switches,
+    raise_rollback_floor_to_the_admission_ceiling, unmount, MountError, RollbackTarget,
+    ShadowLedger, StillUnreadableAfterOneReread,
+};
+use singlefs_core::pointer::LocationEntry;
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::transaction::PoolVersion;
+use singlefs_format::SLOT_BYTES;
+
+const OVERWRITES_BEFORE_R: u64 = 5;
+
+/// 已被占那一形里，M1 之后最多再覆盖写几次去等 R 的树表槽与映射根槽都被重新分配（384 槽小盘一圈根环 24 次发布之内）。
+const OVERWRITES_AFTER_THE_MOUNT_AT_MOST: u64 = 24;
+
+fn root_of_r(recorded: &RecordedPublish) -> RootRecord {
+    RootRecord::parse_slot(
+        recorded.root_slot_write().bytes().expect("根槽写带字节"),
+        &recorded.pool.parameters.filesystem_identifier,
+    )
+    .expect("R 的根自证得过")
+}
+
+/// R 的树表槽与映射根槽的每一条位置条目。
+fn account_slots_of(root: &RootRecord) -> Vec<LocationEntry> {
+    root.tree_table
+        .locations
+        .iter()
+        .chain(root.mapping_root.locations.iter())
+        .copied()
+        .collect()
+}
+
+/// 现行账里有一条分配代大于 R 的 txg、罩住这一槽的记录。
+fn a_later_record_covers(
+    records: &[AllocationRecord],
+    root: &RootRecord,
+    location: &LocationEntry,
+) -> bool {
+    records.iter().any(|record| {
+        record.device == location.device
+            && record.slot.0 <= location.slot.0
+            && location.slot.0 < record.slot.0 + u64::from(record.span_slots)
+            && record.generation > root.checkpoint_txg
+    })
+}
+
+fn is_the_refusal_on_the_account_of(error: &MountError, root: &RootRecord) -> bool {
+    matches!(
+        error,
+        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
+            if matches!(
+                still_unreadable.as_ref(),
+                StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
+                    abandoned_root: RollbackTarget { instance, checkpoint_txg },
+                } if (*instance, *checkpoint_txg) == (root.instance, root.checkpoint_txg)
+            )
+    )
+}
+
+/// 已被占的那一形：卸载、会话里为空间推的那一串抬 F、再可写挂载三条路都做得成。
+/// 判别力：凭据那一支拿掉（变异「B 的凭据恒不成立」）时三条路都拒成 `AccountOfAnAbandonedRoot`，这一条红。
+#[test]
+fn an_abandoned_root_whose_account_slots_were_taken_by_later_units_no_longer_refuses_unmount_floor_raise_or_remount(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let root = root_of_r(&recorded);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.unit_writes());
+    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let mounted = mount_writable_with_test_only_switches(
+        &recorded.pool.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::Off,
+    )
+    .expect("R 的根槽与单元读回全零：可写挂载落到 P、做得成");
+    let image_after_the_mount =
+        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
+    let mut pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
+    let mut overwrites_after_the_mount = 0;
+    // 迭代上界 OVERWRITES_AFTER_THE_MOUNT_AT_MOST；每一轮看一次现行账，不够再覆盖写一次。
+    while !account_slots_of(&root).iter().all(|location| {
+        a_later_record_covers(
+            &pool.current_file_version().allocation_records,
+            &root,
+            location,
+        )
+    }) {
+        assert!(
+            overwrites_after_the_mount < OVERWRITES_AFTER_THE_MOUNT_AT_MOST,
+            "{OVERWRITES_AFTER_THE_MOUNT_AT_MOST} 次覆盖写之内 R 的树表槽与映射根槽都被重新分配"
+        );
+        pool.overwrite(OVERWRITE_BYTES)
+            .expect("384 槽小盘上这一串覆盖写都落得下");
+        overwrites_after_the_mount += 1;
+    }
+    let image = pool.image();
+    let root_slot_write = recorded.root_slot_write();
+    assert!(
+        root_slot_write
+            .contents
+            .still_on_disk(&image.devices[&root_slot_write.device].read(
+                root_slot_write.offset,
+                usize::try_from(root_slot_write.length_in_bytes()).expect("根槽写长"),
+            )),
+        "R 的根槽还没被这个进程盖掉：R 还在根环里，三条路的影子账都要算它"
+    );
+    let session = pool.session();
+    let PoolVersion::WithFile(current_file_version) = &session.current else {
+        panic!("覆盖写之后现行那一版带文件");
+    };
+    let mut devices_of_the_unmount = plain_devices_on(&image);
+    let mut allocator_of_the_unmount = session.allocator.clone();
+    let mut current_of_the_unmount = session.current.clone();
+    let unmounted = unmount(
+        &recorded.pool.parameters,
+        &mut devices_of_the_unmount,
+        &mut allocator_of_the_unmount,
+        &mut current_of_the_unmount,
+        ShadowLedger::On,
+    );
+    assert!(
+        unmounted.is_ok(),
+        "卸载做得成（R 的账读不出，现行账里有那两槽之后的记录）：{:?}",
+        unmounted.err()
+    );
+    let mut devices_of_the_raise = plain_devices_on(&image);
+    let mut allocator_of_the_raise = session.allocator.clone();
+    let mut current_of_the_raise = current_file_version.clone();
+    let raised = raise_rollback_floor_to_the_admission_ceiling(
+        &recorded.pool.parameters,
+        &mut devices_of_the_raise,
+        &mut allocator_of_the_raise,
+        &mut current_of_the_raise,
+        ShadowLedger::On,
+    );
+    assert!(
+        raised.is_ok(),
+        "会话为空间推的那一串抬 F 做得成：{:?}",
+        raised.err()
+    );
+    let mut devices_of_the_remount = plain_devices_on(&image);
+    let remounted = mount_writable(&recorded.pool.parameters, &mut devices_of_the_remount);
+    assert!(remounted.is_ok(), "再可写挂载做得成：{:?}", remounted.err());
+}
+
+/// 瞬时读不出的那一形（同一个崩溃盘面）：现行账里没有罩住 R 树表槽与映射根槽的记录（C 隔离了 R 的落点，谁都没重新分配它们），
+/// 那两槽这一次读回全零，卸载照拒、点名 R，盘上逐字节不变。判别力：凭据不看现行账、读不出就放（第一轮攻方 A 那一臂）时这一条红。
+#[test]
+fn an_abandoned_root_whose_account_slots_only_read_back_zero_still_refuses_the_unmount_before_any_write(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let root = root_of_r(&recorded);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.unit_writes());
+    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("R 的根槽与单元读回全零：可写挂载落到 P、做得成");
+    let image_after_the_mount =
+        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
+    let pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
+    assert!(
+        !account_slots_of(&root).iter().any(|location| {
+            a_later_record_covers(
+                &pool.current_file_version().allocation_records,
+                &root,
+                location,
+            )
+        }),
+        "C 隔离了 R 的落点：现行账里没有罩住 R 树表槽或映射根槽的记录"
+    );
+    let account_ranges = RangesReadingBackAllZero(
+        account_slots_of(&root)
+            .iter()
+            .map(|location| (location.device, location.slot.0 * SLOT_BYTES, SLOT_BYTES))
+            .collect(),
+    );
+    let image = pool.image();
+    let session = pool.session();
+    let mut devices_of_the_unmount = devices_reading_back_all_zero_on(&image, &account_ranges);
+    let mut allocator_of_the_unmount = session.allocator.clone();
+    let mut current_of_the_unmount = session.current.clone();
+    let Err(refusal) = unmount(
+        &recorded.pool.parameters,
+        &mut devices_of_the_unmount,
+        &mut allocator_of_the_unmount,
+        &mut current_of_the_unmount,
+        ShadowLedger::On,
+    ) else {
+        panic!("R 的账与映射这一次都读回全零、现行账里没有凭据：卸载照拒，而这次卸载做成了");
+    };
+    assert!(
+        is_the_refusal_on_the_account_of(&refusal, &root),
+        "拒因点名 R 的账：{refusal:?}"
+    );
+    assert!(
+        image_of_devices_reading_back_all_zero(&devices_of_the_unmount, recorded.pool.device_bytes)
+            == image,
+        "在任何写之前拒：两块盘逐字节不变"
+    );
+}
diff --git a/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs b/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs
index 55728b4..162f71b 100644
--- a/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs
+++ b/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs
@@ -182,7 +182,7 @@ fn admission_reading(
 }
 
 /// 同一份读数，只把第九项「被抛弃根独占量」逐盘置 0（C318 判别力自证要的那一刀）。
-fn reading_with_the_abandoned_root_exclusive_term_zeroed(
+fn reading_with_the_isolated_for_abandoned_roots_and_unapplied_journal_records_term_zeroed(
     reading: &AdmissionReading,
 ) -> AdmissionReading {
     AdmissionReading::new(
@@ -195,7 +195,7 @@ fn reading_with_the_abandoned_root_exclusive_term_zeroed(
                 allocated: terms.allocated,
                 unreclaimable: terms.unreclaimable,
                 mount_time_commitment: terms.mount_time_commitment,
-                abandoned_root_exclusive: BytesOnOneDevice::ZERO,
+                isolated_for_abandoned_roots_and_unapplied_journal_records: BytesOnOneDevice::ZERO,
                 ring_window: terms.ring_window,
             })
             .collect(),
@@ -295,12 +295,12 @@ fn after_a_recovery_abandoned_a_root_the_admission_reading_with_the_shadow_ledge
                 (*device, *device)
             );
             assert_eq!(
-                terms_with.abandoned_root_exclusive,
+                terms_with.isolated_for_abandoned_roots_and_unapplied_journal_records,
                 BytesOnOneDevice::of_slots(*abandoned_slots),
                 "{supplied:?} 盘 {device:?}：影子账开着，第九项是只被被抛弃根引用的那几个槽"
             );
             assert_eq!(
-                terms_without.abandoned_root_exclusive,
+                terms_without.isolated_for_abandoned_roots_and_unapplied_journal_records,
                 BytesOnOneDevice::ZERO,
                 "{supplied:?} 盘 {device:?}：影子账关着，第九项是 0"
             );
@@ -341,7 +341,10 @@ fn after_a_recovery_abandoned_a_root_the_admission_reading_with_the_shadow_ledge
             );
         }
 
-        let zeroed = reading_with_the_abandoned_root_exclusive_term_zeroed(&reading_with);
+        let zeroed =
+            reading_with_the_isolated_for_abandoned_roots_and_unapplied_journal_records_term_zeroed(
+                &reading_with,
+            );
         for (((device, with), (_, zeroed_available)), (_, abandoned_slots)) in available_with
             .iter()
             .zip(&zeroed.available_on_each_device())
diff --git a/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs b/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs
index ec6f9db..9cd2ad3 100644
--- a/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs
+++ b/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs
@@ -75,7 +75,8 @@ fn available_bytes_the_ring_window_takes_on_each_device(reading: &AdmissionReadi
                 allocated: terms.allocated,
                 unreclaimable: terms.unreclaimable,
                 mount_time_commitment: terms.mount_time_commitment,
-                abandoned_root_exclusive: terms.abandoned_root_exclusive,
+                isolated_for_abandoned_roots_and_unapplied_journal_records: terms
+                    .isolated_for_abandoned_roots_and_unapplied_journal_records,
                 ring_window: BytesOnOneDevice::ZERO,
             })
             .collect(),
diff --git a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
index 1ca3784..1ec1003 100644
--- a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
+++ b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
@@ -14,9 +14,10 @@ mod common;
 mod common_tree_split;
 
 use common::{
-    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
-    crash_state_devices, memory_pool_of_sparse_devices, parameters, publish_overwrite_in_process,
-    BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before,
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_the_shadow_ledger,
+    build_pool, crash_state_devices, memory_pool_of_sparse_devices, parameters,
+    publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
 };
 use common_tree_split::{
     capacities, propagate_the_new_checksum, read_unit_on_device, reseal_unit_by_its_class,
@@ -32,7 +33,7 @@ use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
 use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
 use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT};
-use singlefs_core::mount::mount_writable;
+use singlefs_core::mount::{mount_writable, ShadowLedger};
 use singlefs_core::pointer::BirthSequence;
 use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
 use singlefs_core::unit::{build_index_node, parse_index_node, IndexNodeHeader};
@@ -515,8 +516,13 @@ fn image_after_the_blind_mount_handed_out_a_unit_of_the_abandoned_root(
     tag: &str,
 ) -> (MemoryPool, TransactionOutput, Vec<u64>) {
     let (mut pool, third) = pool_through_the_third_version(tag);
+    // 那次挂载关着影子账（连 C694 的 C 一起关）：C 之后 C 的记录读得出时产品路径看得见 C、隔离它写出的单元，造不出这一形。
     let blind_mount =
-        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
+        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_the_shadow_ledger(
+            &mut pool,
+            &third,
+            ShadowLedger::Off,
+        );
     let instance = blind_mount.output.instance;
     for round in 0..OVERWRITES_IN_THE_BLIND_MOUNT_AT_MOST {
         let round_index = usize::try_from(round).expect("轮次");
diff --git a/crates/singlefs-harness/tests/common/mod.rs b/crates/singlefs-harness/tests/common/mod.rs
index 0ee06e9..a22e7c2 100644
--- a/crates/singlefs-harness/tests/common/mod.rs
+++ b/crates/singlefs-harness/tests/common/mod.rs
@@ -747,6 +747,21 @@ pub fn build_pool(tag: &str) -> BuiltPool {
 pub fn abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(
     pool: &mut BuiltPool,
     newest: &TransactionOutput,
+) -> singlefs_core::mount::Mounted {
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_the_shadow_ledger(
+        pool,
+        newest,
+        singlefs_core::mount::ShadowLedger::On,
+    )
+}
+
+/// 同 [`abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`]，那次挂载的影子账那一臂由调用方给（只供测试的开关）。
+/// `ShadowLedger::Off` 连 C694 的 C（水位之上没施加的记录点名的落点隔离，删除线第 33 批 T）一起关：那次挂载真看不见 `newest`，
+/// 它的分配器把 `newest` 写出的单元当空闲槽——C 之后产品路径上 `newest` 的记录读得出时造不出这一形，checker 那几条要这一形的坏镜像用它造。
+pub fn abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_the_shadow_ledger(
+    pool: &mut BuiltPool,
+    newest: &TransactionOutput,
+    shadow_ledger: singlefs_core::mount::ShadowLedger,
 ) -> singlefs_core::mount::Mounted {
     use singlefs_core::block_device::{BlockDevice, WriteDurability};
     let publish_parameters = parameters();
@@ -842,8 +857,13 @@ pub fn abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(
             .write_at(newest_slot, &rotation_not_written, WriteDurability::Plain)
             .expect("见证 newest 的系统配置槽写回成轮换之前的那一代");
     }
-    let mounted = singlefs_core::mount::mount_writable(&publish_parameters, &mut devices)
-        .expect("最新那条根与它的数据单元读不出：择根落到前一条根，照常可写挂载");
+    let mounted = singlefs_core::mount::mount_writable_with_test_only_switches(
+        &publish_parameters,
+        &mut devices,
+        singlefs_core::admission::SpaceAdmission::JudgedByTheFormula,
+        shadow_ledger,
+    )
+    .expect("最新那条根与它的数据单元读不出：择根落到前一条根，照常可写挂载");
     for (identity, offset, bytes) in &saved {
         let index = index_of_device(&devices, *identity);
         devices[index]
diff --git a/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs b/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs
new file mode 100644
index 0000000..9a26b66
--- /dev/null
+++ b/crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs
@@ -0,0 +1,398 @@
+//! C694、C695 那一簇（删除线第 33 批 T）几份 harness 档用例共用的搭建：单元区 384 槽那一档小盘上第一个文件之后在会话里覆盖写 k 次，
+//! 再经会话录一次发布 R（覆盖写一次，或顺序写三个数据单元、切成三条记录），崩在 R 的根槽落盘之后、它那次系统配置轮换之前
+//! （系统配置没见证 R，D16（发布语义） 已定项 7 的持久顺序）。之后的一次可写挂载里让 R 的根槽、R 写出的单元、R 的某几条记录读回全零
+//! （只这一次挂载：写到哪一段，那一段就不再藏），即攻方 `research/prompts/m3-c694-r2-opus-output.md` 第一、二、五节的 `ur` / `urM` / `urF` /
+//! `urL` / `urJ` 那几形，在入库装置上重造。池本身（mkfs、取号、暖机、第一个文件、会话）在 `tests/common_admission_pool/mod.rs`，这里全部转出去。
+#![allow(dead_code, reason = "几个测试文件各自只用到其中一部分")]
+
+#[path = "../common_admission_pool/mod.rs"]
+mod common_admission_pool;
+
+pub use common_admission_pool::*;
+
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
+use singlefs_core::block_device::{
+    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+};
+use singlefs_core::mount::Mounted;
+use singlefs_core::mounted_session::MountedSession;
+use singlefs_harness::memory_pool::{
+    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
+};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+/// 录的那一次发布 R 是哪一种。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum PublishRecordedAsR {
+    /// 覆盖写一次（一个数据单元，一条记录）：第一轮攻方 K1-2 那一形的 R。
+    Overwrite,
+    /// 顺序写三个数据单元（每个数据单元一个事务、一条记录，末条另点名全部元数据单元）：第二轮攻方第二节那一形的 R。
+    SequentialWriteOfThreeDataUnits,
+}
+
+/// 顺序写三个数据单元的内容长度：两个数据单元的净荷再加一千字节。
+fn content_bytes_of_three_data_units() -> usize {
+    2 * singlefs_core::unit::data_unit_payload_capacity() + 1000
+}
+
+/// 录下来的 R：R 之前那一刻的盘面（`base`）、R 那次发布的全部写（`writes`，按落盘次序）、R 的根槽写在写表里的下标、R 的 (实例, txg)
+/// 与它带的文件内容，加起池那一份（参数、盘宽、每条根带的文件内容；会话停在 R 那一版）。
+pub struct RecordedPublish {
+    pub pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
+    pub base: MemoryPool,
+    pub writes: Vec<RetainedWrite>,
+    pub root_index: usize,
+    pub key: (InstanceGeneration, CheckpointTxg),
+    pub content: Vec<u8>,
+    /// R 之前的最后一版（P）的 (实例, txg) 与文件内容。
+    pub previous_key: (InstanceGeneration, CheckpointTxg),
+    pub previous_content: Vec<u8>,
+}
+
+/// 起池（384 槽那一档、第一个文件），会话里覆盖写 `overwrites_before` 次，再经会话录一次 `publish`。
+///
+/// # Panics
+/// 起池、覆盖写或录的那次发布被拒（这几档参数下都落得下）；录的那次发布不是一条根。
+pub fn record_one_publish_after_overwrites(
+    overwrites_before: u64,
+    publish: PublishRecordedAsR,
+) -> RecordedPublish {
+    let stream = SharedStream::retaining_contents();
+    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
+        parameters_of_the_384_slot_small_devices(),
+        device_bytes_of_the_384_slot_small_devices(),
+        |identity, device| {
+            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
+        },
+    );
+    for _overwrite in 0..overwrites_before {
+        pool.overwrite(OVERWRITE_BYTES)
+            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
+    }
+    let previous_root = *pool.session().current.root();
+    let previous_content = pool.content_of_the_current_version.clone();
+    let base = pool.image();
+    let operations_before_r = stream.operation_count();
+    match publish {
+        PublishRecordedAsR::Overwrite => pool.overwrite(OVERWRITE_BYTES),
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits => {
+            pool.sequential_write(content_bytes_of_three_data_units())
+        }
+    }
+    .expect("录的那次发布落得下");
+    let root = *pool.session().current.root();
+    let operations = stream.retained_operations();
+    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[operations_before_r..],
+        &fixed_geometry_of_the_384_slot_small_devices(),
+    );
+    let root_indices: Vec<usize> = writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
+        .map(|(index, _)| index)
+        .collect();
+    assert_eq!(
+        root_indices.len(),
+        1,
+        "录的那一次发布只写一条根（准入够，没推抬 F）"
+    );
+    RecordedPublish {
+        content: pool.content_of_the_current_version.clone(),
+        pool,
+        base,
+        writes,
+        root_index: root_indices[0],
+        key: (root.instance, root.checkpoint_txg),
+        previous_key: (previous_root.instance, previous_root.checkpoint_txg),
+        previous_content,
+    }
+}
+
+impl RecordedPublish {
+    /// 崩在 R 的根槽落盘之后、系统配置轮换之前：R 的单元、记录与根槽都落了，轮换那两槽没落（写表里根槽之后的都不施加）。
+    #[must_use]
+    pub fn crash_image_with_the_root_persisted_and_the_rotation_lost(&self) -> MemoryPool {
+        let mut image = self.base.clone();
+        image.apply_writes(&self.writes[..=self.root_index]);
+        image
+    }
+
+    /// R 的整次发布都落了（系统配置轮换与末尾屏障都过了：R 已确认）。
+    #[must_use]
+    pub fn image_with_the_whole_publish_persisted(&self) -> MemoryPool {
+        let mut image = self.base.clone();
+        image.apply_writes(&self.writes);
+        image
+    }
+
+    /// R 写出的单元写（每块盘一份）。
+    #[must_use]
+    pub fn unit_writes(&self) -> Vec<&RetainedWrite> {
+        self.writes
+            .iter()
+            .filter(|write| write.kind == StepKind::UnitWrite)
+            .collect()
+    }
+
+    /// R 的第 `ordinal_from_one` 条记录的两份（两块盘同一个偏移）：记录写按落盘次序、按偏移归条，第一个偏移是第 1 条。
+    ///
+    /// # Panics
+    /// R 没有那么多条记录。
+    #[must_use]
+    pub fn record_writes_of_the_record(&self, ordinal_from_one: usize) -> Vec<&RetainedWrite> {
+        let mut offsets: Vec<DeviceOffsetInBytes> = Vec::new();
+        for write in self
+            .writes
+            .iter()
+            .filter(|write| write.kind == StepKind::JournalRecord)
+        {
+            if !offsets.contains(&write.offset) {
+                offsets.push(write.offset);
+            }
+        }
+        let offset = offsets[ordinal_from_one - 1];
+        self.writes
+            .iter()
+            .filter(|write| write.kind == StepKind::JournalRecord && write.offset == offset)
+            .collect()
+    }
+
+    /// R 一共几条记录。
+    #[must_use]
+    pub fn record_count(&self) -> usize {
+        let mut offsets: Vec<DeviceOffsetInBytes> = Vec::new();
+        for write in self
+            .writes
+            .iter()
+            .filter(|write| write.kind == StepKind::JournalRecord)
+        {
+            if !offsets.contains(&write.offset) {
+                offsets.push(write.offset);
+            }
+        }
+        offsets.len()
+    }
+
+    /// R 的根槽写。
+    #[must_use]
+    pub fn root_slot_write(&self) -> &RetainedWrite {
+        &self.writes[self.root_index]
+    }
+}
+
+/// 一次挂载里读回全零的那几段（每段：盘、偏移、长度）。
+#[derive(Clone, Debug, Default)]
+pub struct RangesReadingBackAllZero(pub Vec<(DeviceIdentity, u64, u64)>);
+
+impl RangesReadingBackAllZero {
+    /// 加上这几次写各自落的那一段。
+    #[must_use]
+    pub fn with_the_writes(mut self, writes: &[&RetainedWrite]) -> Self {
+        self.0.extend(
+            writes
+                .iter()
+                .map(|write| (write.device, write.offset.0, write.length_in_bytes())),
+        );
+        self
+    }
+
+    fn on_device(&self, device: DeviceIdentity) -> Vec<(u64, u64)> {
+        self.0
+            .iter()
+            .filter(|(range_device, _, _)| *range_device == device)
+            .map(|(_, offset, length)| (*offset, *length))
+            .collect()
+    }
+}
+
+/// 读点名的几段读回全零（盘上的字节不动）；写到哪一段，那一段之后就照读。设备报读错不在这一层：故障模型是「读回全零」
+/// （`crates/singlefs-harness/src/history.rs` 注入的那一种）。
+pub struct DeviceReadingBackAllZeroOnRanges<Inner: BlockDevice> {
+    pub inner: Inner,
+    ranges: Vec<(u64, u64)>,
+}
+
+fn overlaps(start: u64, length: u64, offset: u64, other_length: u64) -> bool {
+    start < offset + other_length && offset < start + length
+}
+
+impl<Inner: BlockDevice> BlockDevice for DeviceReadingBackAllZeroOnRanges<Inner> {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        self.inner.read_at(offset, buffer)?;
+        let buffer_length = u64::try_from(buffer.len()).expect("读长装得进 u64");
+        for (start, length) in &self.ranges {
+            if overlaps(*start, *length, offset.0, buffer_length) {
+                let from = usize::try_from(start.saturating_sub(offset.0)).expect("缓冲内偏移");
+                let to = usize::try_from((start + length).min(offset.0 + buffer_length) - offset.0)
+                    .expect("缓冲内偏移");
+                buffer[from..to].fill(0);
+            }
+        }
+        Ok(())
+    }
+
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        let written_length = u64::try_from(bytes.len()).expect("写长装得进 u64");
+        self.ranges
+            .retain(|(start, length)| !overlaps(*start, *length, offset.0, written_length));
+        self.inner.write_at(offset, bytes, durability)
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.ranges
+            .retain(|(start, range_length)| !overlaps(*start, *range_length, offset.0, length));
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
+/// 一份镜像交给两块新的稀疏盘，`ranges` 那几段读回全零。
+#[must_use]
+pub fn devices_reading_back_all_zero_on(
+    image: &MemoryPool,
+    ranges: &RangesReadingBackAllZero,
+) -> Vec<(
+    DeviceIdentity,
+    DeviceReadingBackAllZeroOnRanges<SparseBlockDevice>,
+)> {
+    plain_devices_on(image)
+        .into_iter()
+        .map(|(identity, inner)| {
+            (
+                identity,
+                DeviceReadingBackAllZeroOnRanges {
+                    inner,
+                    ranges: ranges.on_device(identity),
+                },
+            )
+        })
+        .collect()
+}
+
+/// 同上，每块盘外面再录一层（录挂载自己的写，拿来摆「挂载写到一半断电」那几个状态）。
+#[must_use]
+pub fn recorded_devices_reading_back_all_zero_on(
+    image: &MemoryPool,
+    ranges: &RangesReadingBackAllZero,
+    stream: &SharedStream,
+) -> Vec<(
+    DeviceIdentity,
+    DeviceReadingBackAllZeroOnRanges<RecordingBlockDevice<SparseBlockDevice>>,
+)> {
+    plain_devices_on(image)
+        .into_iter()
+        .map(|(identity, inner)| {
+            (
+                identity,
+                DeviceReadingBackAllZeroOnRanges {
+                    inner: RecordingBlockDevice::with_shared_stream(
+                        identity,
+                        inner,
+                        stream.clone(),
+                    ),
+                    ranges: ranges.on_device(identity),
+                },
+            )
+        })
+        .collect()
+}
+
+/// 两块盘此刻的盘面（盘上的字节，读回全零那一层不算）。
+#[must_use]
+pub fn image_of_devices_reading_back_all_zero(
+    devices: &[(
+        DeviceIdentity,
+        DeviceReadingBackAllZeroOnRanges<SparseBlockDevice>,
+    )],
+    device_bytes: u64,
+) -> MemoryPool {
+    MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.inner.image.clone()))
+            .collect(),
+        device_size_in_bytes: device_bytes,
+    }
+}
+
+/// 可写挂载做成之后的池：盘换成这次挂载之后的盘面（不再读回全零），会话是这次挂载的，每条根带的文件内容接着录的那一份记。
+#[must_use]
+pub fn pool_after_the_mount(
+    recorded: &RecordedPublish,
+    image_after_the_mount: &MemoryPool,
+    mounted: Mounted,
+) -> PoolUnderTest<SparseBlockDevice> {
+    let (output, session) = MountedSession::of_the_mount(mounted);
+    let content_of_the_current_version = recorded
+        .pool
+        .content_of_each_root
+        .get(&(
+            output.effective_root.instance,
+            output.effective_root.checkpoint_txg,
+        ))
+        .cloned()
+        .expect("挂载落到的那一版是录的池写过的一版");
+    let mut content_of_each_root = recorded.pool.content_of_each_root.clone();
+    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
+        let root = version.root();
+        content_of_each_root.insert(
+            (root.instance, root.checkpoint_txg),
+            content_of_the_current_version.clone(),
+        );
+    }
+    PoolUnderTest {
+        parameters: recorded.pool.parameters.clone(),
+        device_bytes: recorded.pool.device_bytes,
+        devices: plain_devices_on(image_after_the_mount),
+        session: Some(session),
+        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
+        content_of_the_current_version,
+        content_of_each_root,
+    }
+}
+
+/// R 写出的单元里，`image` 上逐字节已不是 R 写的那几份（盘、偏移）。
+#[must_use]
+pub fn units_of_r_overwritten_on(
+    recorded: &RecordedPublish,
+    image: &MemoryPool,
+) -> Vec<(DeviceIdentity, u64)> {
+    recorded
+        .unit_writes()
+        .into_iter()
+        .filter(|write| {
+            let length = usize::try_from(write.length_in_bytes()).expect("单元写长");
+            let read_back = image.devices[&write.device].read(write.offset, length);
+            !write.contents.still_on_disk(&read_back)
+        })
+        .map(|write| (write.device, write.offset.0))
+        .collect()
+}
diff --git a/crates/singlefs-harness/tests/mounted_read_reports_an_empty_tree_table_before_reading_an_empty_mapping_root.rs b/crates/singlefs-harness/tests/mounted_read_reports_an_empty_tree_table_before_reading_an_empty_mapping_root.rs
new file mode 100644
index 0000000..9a978e4
--- /dev/null
+++ b/crates/singlefs-harness/tests/mounted_read_reports_an_empty_tree_table_before_reading_an_empty_mapping_root.rs
@@ -0,0 +1,84 @@
+//! C695 甲（用户 2026-10-07 定「先修次序，选哪一版归 C683」；判决 `research/prompts/m3-c694-r1-main-verification.md` 第二节 K3 那一行）：
+//! 打开挂载态时根记录的中央映射树根是空指针，先判树表 0 条、报 `OpenPoolForReadFailure::TreeTableHasNoEntries`，不先读空映射根
+//! （改之前读那条空指针的位置条目，两盘都在也报 `Walk(UnitUnreadable { slot: SlotNumber(0) })`）。
+//! 盘面：384 槽小盘 mkfs，可写挂载两次（第一次取号 1、暖机；第二次取号 2、在树表 0 条的一版上写行、暖机），一个文件都不写：
+//! 环里读得出的根是 mkfs 的第 0 代根、暖机根与写行根，树表都是 0 条、映射根都是空指针。
+mod common_admission_pool;
+
+use common_admission_pool::{
+    device_bytes_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
+};
+use singlefs_core::address::DeviceIdentity;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::make_filesystem::make_filesystem;
+use singlefs_core::mount::mount_writable;
+use singlefs_core::mounted_read::{
+    mount_read_only, open_pool_for_read, MountReadOnlyFailure, OpenPoolForReadFailure,
+};
+use singlefs_core::pointer::NodePointer;
+use singlefs_core::recovery::{choose_system_configuration, readable_roots};
+use singlefs_harness::memory_pool::SparseBlockDevice;
+
+/// mkfs 之后可写挂载两次、一个文件都不写的两块 384 槽小盘。
+fn devices_of_a_pool_without_a_file_after_two_writable_mounts(
+) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
+    let parameters = parameters_of_the_384_slot_small_devices();
+    let device_bytes = device_bytes_of_the_384_slot_small_devices();
+    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
+        [DeviceIdentity(0), DeviceIdentity(1)]
+            .into_iter()
+            .map(|identity| {
+                (
+                    identity,
+                    SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
+                )
+            })
+            .collect();
+    make_filesystem(&parameters, &mut devices).expect("小盘上 mkfs 做得成");
+    mount_writable(&parameters, &mut devices).expect("mkfs 之后第一次可写挂载");
+    mount_writable(&parameters, &mut devices).expect("树表 0 条的一版上第二次可写挂载（写行）");
+    devices
+}
+
+/// 环里每一条读得出的根（mkfs 的第 0 代、暖机、写行）打开挂载态都报树表 0 条；只读挂载同样报它。
+/// 判别力：先判树表 0 条那一步拿掉（变异「C695 先读空映射根」）时报的是 `Walk(UnitUnreadable { slot: SlotNumber(0) })`，这一条红。
+#[test]
+fn every_root_of_a_pool_without_a_file_opens_to_an_empty_tree_table_instead_of_an_unreadable_slot_zero(
+) {
+    let devices = devices_of_a_pool_without_a_file_after_two_writable_mounts();
+    let system_configuration = choose_system_configuration(&devices).expect("择系统配置");
+    let roots = readable_roots(
+        &devices,
+        &system_configuration.immutable.region_devices,
+        &system_configuration.immutable.sizes,
+        &system_configuration.immutable.filesystem_identifier,
+    );
+    let instances: Vec<u32> = roots.iter().map(|root| root.instance.0).collect();
+    assert!(
+        instances.contains(&0) && instances.contains(&1) && instances.contains(&2),
+        "环里有 mkfs 的第 0 代根、实例 1 的暖机根与实例 2 的写行根：{instances:?}"
+    );
+    for root in &roots {
+        assert_eq!(
+            root.mapping_root,
+            NodePointer::empty_root(),
+            "树表 0 条的根（实例 {:?} txg {:?}）映射根是空指针",
+            root.instance,
+            root.checkpoint_txg
+        );
+        assert_eq!(
+            open_pool_for_read(&devices, root).err(),
+            Some(OpenPoolForReadFailure::TreeTableHasNoEntries),
+            "实例 {:?} txg {:?} 那条根两盘都在：报树表 0 条，不报槽 0 读不出",
+            root.instance,
+            root.checkpoint_txg
+        );
+    }
+    assert_eq!(
+        mount_read_only(&devices).err(),
+        Some(MountReadOnlyFailure::Open(
+            OpenPoolForReadFailure::TreeTableHasNoEntries
+        )),
+        "只读挂载择到的那条根同样报树表 0 条（树表 0 条不是走读失败，F2 乙不退）"
+    );
+}
diff --git a/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs b/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs
new file mode 100644
index 0000000..437e15d
--- /dev/null
+++ b/crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs
@@ -0,0 +1,217 @@
+//! F2 乙（用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 7 条）：所选那一版走读失败、系统配置没见证它时，
+//! 退回系统配置见证到的那一版；恢复（`recovery::recover`）、可写挂载读阶段之后建上一版那一步、只读挂载（`mounted_read::mount_read_only`）三处同一份判法
+//! （`recovery::version_witnessed_by_the_system_configuration_to_fall_back_to`）。「见证过」照攻方 `strict`：所选那一版带末条标志的记录读得出 ⇒
+//! 计数器 ≤ c_见证；读不出 ⇒ 拿计数器 = c_见证 的那条记录比 (txg, 实例)；两条都读不出当见证过、不退。
+//!
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次覆盖写 R（P 是它之前那一版）。
+//! - 第二轮攻方 5.2 两形：崩在 R 的根槽落盘之后、系统配置轮换之前；取新实例的那次挂载里 R 的根槽与单元读回全零（`ur`）或连 R 的记录也读回全零
+//!   （`urJ`），那次挂载写行那次发布的根落盘之前断电，R 的单元已被那次挂载写掉；断电之后三处都读回 P。`ur` 那一形今天的 C 隔离得住 R 的单元，
+//!   这里关着影子账（只供测试的开关 `ShadowLedger::Off`，连 C 一起关）造出 R 的单元被写掉的盘面；`urJ` 那一形 C 本来就看不见。
+//! - 5.3 两形（R 已确认：整次发布连系统配置轮换都落了）：S2——R 的末条（唯一一条）记录与单元读回全零，两条都读不出、当见证过、不退，照报读不出；
+//!   S1——较新的那个系统配置槽（R 那次轮换写的）与 R 的单元读回全零，c_见证 落回 P，R 算没见证、读回 P：用户 2026-10-07 定案接受的代价。
+mod common_unwitnessed_publish;
+
+use common_unwitnessed_publish::{
+    devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
+    image_of_devices_reading_back_all_zero, record_one_publish_after_overwrites,
+    recorded_devices_reading_back_all_zero_on, units_of_r_overwritten_on, PublishRecordedAsR,
+    RangesReadingBackAllZero, RecordedPublish,
+};
+use singlefs_core::address::DeviceIdentity;
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::mount::{mount_writable, mount_writable_with_test_only_switches, ShadowLedger};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
+use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::SharedStream;
+
+const OVERWRITES_BEFORE_R: u64 = 5;
+
+/// 取新实例的那次挂载里 `ranges` 读回全零、影子账按 `shadow_ledger`，写行那次发布的根落盘之前断电：交回断电那一刻的盘面。
+fn image_of_a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance(
+    recorded: &RecordedPublish,
+    ranges: &RangesReadingBackAllZero,
+    shadow_ledger: ShadowLedger,
+) -> MemoryPool {
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let stream = SharedStream::retaining_contents();
+    let mut devices = recorded_devices_reading_back_all_zero_on(&crash_image, ranges, &stream);
+    mount_writable_with_test_only_switches(
+        &recorded.pool.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        shadow_ledger,
+    )
+    .expect("系统配置没见证 R：取新实例的那次可写挂载做得成");
+    let (mount_writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &stream.retained_operations(),
+        &fixed_geometry_of_the_384_slot_small_devices(),
+    );
+    let row_root = mount_writes
+        .iter()
+        .position(|write| write.kind == StepKind::RootRecordFua)
+        .expect("那次挂载写行那次发布有根槽写");
+    let mut image = crash_image;
+    image.apply_writes(&mount_writes[..row_root]);
+    assert_ne!(
+        units_of_r_overwritten_on(recorded, &image),
+        Vec::<(DeviceIdentity, u64)>::new(),
+        "写行那次发布的根落盘之前，那次挂载已写掉 R 的单元（这一形的前提：恢复落到 R 上走读失败）"
+    );
+    image
+}
+
+/// 三处都读回 P：恢复读回 P 的内容、落到 P；只读挂载择到并打开 P；可写挂载建在 P 上。
+fn assert_recovery_and_both_mounts_read_back_the_previous_version(
+    recorded: &RecordedPublish,
+    image: &MemoryPool,
+) {
+    let report = recover(image, JournalPolicy::Consult);
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::FileRead {
+            root: recorded.previous_key,
+            content: recorded.previous_content.clone(),
+        },
+        "恢复：所选那一版 R 走读失败、系统配置没见证它，退回见证到的 P、读回 P 的内容（今天：Failed(UnitUnreadable)）"
+    );
+    assert_eq!(report.effective_root, Some(recorded.previous_key));
+    let read_only = mount_read_only(image).expect("只读挂载退回 P、打得开");
+    assert_eq!(
+        (
+            read_only.effective_root.instance,
+            read_only.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "只读挂载：退回 P"
+    );
+    let mut devices = devices_reading_back_all_zero_on(image, &RangesReadingBackAllZero::default());
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("可写挂载退回 P、建得出上一版");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "可写挂载：建在 P 上"
+    );
+}
+
+/// `ur`：R 的记录读得出（末条，计数器大于 c_见证）。判别力：「末条读得出就当见证过」（变异「F2 末条那一支恒见证过」）时三处都不退、这一条红。
+#[test]
+fn a_root_whose_units_were_overwritten_while_its_record_reads_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.unit_writes());
+    let image = image_of_a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance(
+        &recorded,
+        &ranges,
+        ShadowLedger::Off,
+    );
+    assert_recovery_and_both_mounts_read_back_the_previous_version(&recorded, &image);
+}
+
+/// `urJ`：R 的记录那次也读回全零，取新实例的那次挂载把它的记录槽写成自己的写行记录（计数器接在 P 后面）：R 的末条读不出，
+/// 计数器 = c_见证 的那条是 P 的，(txg, 实例) 比 R 旧 ⇒ 没见证。判别力：「拿 c_见证 那条比」那一支恒见证过
+/// （变异「F2 c_见证 那一支恒见证过」）时三处都不退、这一条红。
+#[test]
+fn a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.unit_writes())
+        .with_the_writes(&recorded.record_writes_of_the_record(1));
+    let image = image_of_a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance(
+        &recorded,
+        &ranges,
+        ShadowLedger::On,
+    );
+    assert_recovery_and_both_mounts_read_back_the_previous_version(&recorded, &image);
+}
+
+/// 5.3 的 S2：R 已确认，R 的记录（末条，也是计数器 = c_见证 的那一条）与单元读回全零：两条都读不出，当见证过、不退——恢复照报读不出、
+/// 只读挂载照报打不开、可写挂载照拒，盘上逐字节不变。判别力：两条都读不出当没见证（变异「F2 两条都读不出就退」，攻方 `loose`）时恢复读回 P、这一条红。
+#[test]
+fn a_confirmed_root_whose_last_record_and_units_read_back_zero_is_not_replaced_by_the_previous_version(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let confirmed = recorded.image_with_the_whole_publish_persisted();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&recorded.unit_writes())
+        .with_the_writes(&recorded.record_writes_of_the_record(1));
+    let devices = devices_reading_back_all_zero_on(&confirmed, &ranges);
+    let report = recover(&devices, JournalPolicy::Consult);
+    assert!(
+        matches!(
+            report.outcome,
+            RecoveryOutcome::Failed { root: Some(root), .. } if root == recorded.key
+        ),
+        "恢复落到已确认的 R、照报走读失败，不悄悄读回 P：{:?}",
+        report.outcome
+    );
+    assert!(
+        mount_read_only(&devices).is_err(),
+        "只读挂载照报打不开 R，不退回 P"
+    );
+    let mut devices_of_the_mount = devices_reading_back_all_zero_on(&confirmed, &ranges);
+    assert!(
+        mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount).is_err(),
+        "可写挂载照拒，不在 P 上取号"
+    );
+    assert!(
+        image_of_devices_reading_back_all_zero(&devices_of_the_mount, recorded.pool.device_bytes)
+            == confirmed,
+        "可写挂载在任何写之前拒：两块盘逐字节不变"
+    );
+}
+
+/// 5.3 的 S1：R 已确认，较新的那个系统配置槽（R 那次轮换写的，每块盘一槽）与 R 的单元读回全零：c_见证 落回 P 的那一次，R 的末条读得出、
+/// 计数器大于它，算没见证，恢复与只读挂载读回 P——用户 2026-10-07 定案接受的代价（F2 乙「三处都做」，交用户时写明：已确认的版本在较新系统配置槽与
+/// 它的单元同时读回全零时悄悄读回上一版）。这一条钉住这个结局；条款改了它跟着改。判别力：同 `ur` 那一条的变异（末条那一支恒见证过）时照报读不出、这一条红。
+#[test]
+fn user_accepted_cost_of_2026_10_07_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_reads_back_the_previous_version(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let confirmed = recorded.image_with_the_whole_publish_persisted();
+    let rotation_of_r: Vec<_> = recorded
+        .writes
+        .iter()
+        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
+        .collect();
+    assert_eq!(
+        rotation_of_r.len(),
+        2,
+        "R 那次发布的系统配置轮换每块盘写一槽"
+    );
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&recorded.unit_writes())
+        .with_the_writes(&rotation_of_r);
+    let devices = devices_reading_back_all_zero_on(&confirmed, &ranges);
+    let report = recover(&devices, JournalPolicy::Consult);
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::FileRead {
+            root: recorded.previous_key,
+            content: recorded.previous_content.clone(),
+        },
+        "恢复读回 P（R 已确认、只是这一次读不出）：用户 2026-10-07 定案接受的代价"
+    );
+    let read_only = mount_read_only(&devices).expect("只读挂载退回 P、打得开");
+    assert_eq!(
+        (
+            read_only.effective_root.instance,
+            read_only.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "只读挂载同样读回 P"
+    );
+}
diff --git a/crates/singlefs-harness/tests/release_checksum_quarantine.rs b/crates/singlefs-harness/tests/release_checksum_quarantine.rs
index 0666bcc..d979668 100644
--- a/crates/singlefs-harness/tests/release_checksum_quarantine.rs
+++ b/crates/singlefs-harness/tests/release_checksum_quarantine.rs
@@ -823,7 +823,7 @@ fn quarantined_copy_counts_as_allocated_in_the_admission_reading_and_adds_no_ter
             quarantined_terms.device
         );
         assert_eq!(
-            quarantined_terms.abandoned_root_exclusive,
+            quarantined_terms.isolated_for_abandoned_roots_and_unapplied_journal_records,
             BytesOnOneDevice::ZERO,
             "不另进式子：被抛弃根独占量那一项是 0"
         );
diff --git a/crates/singlefs-harness/tests/row_publish_record_without_its_root.rs b/crates/singlefs-harness/tests/row_publish_record_without_its_root.rs
index dd2464a..1cc04fd 100644
--- a/crates/singlefs-harness/tests/row_publish_record_without_its_root.rs
+++ b/crates/singlefs-harness/tests/row_publish_record_without_its_root.rs
@@ -12,8 +12,9 @@
 //!   实例表指针是 mkfs 那一片、分配记录树根指针全零。写行那次发布恒是新实例的第一次发布，所以「施加一条写行记录」今天走不到。
 //! - checker 怎么算它们：遍历从根走，谁也不引用这两个单元，一条违例都没有；把它们改坏，判决除 I-8.11（点名项与被指单元头相同）
 //!   之外逐条不变（走读不读它们；I-8.11 按记录 jsn 3 的点名项读它们，改坏之后校验和对不上、没有可比的，从成立转成不适用）。
-//! - 分配器怎么算它们：重开之后的可写挂载从 mkfs 那一版的账重建，这两个落点不在任何一条分配记录里、是空闲槽；
-//!   这次挂载写行那次发布取的落点与崩掉的那次逐槽相同，把它们盖掉。之后 checker 仍一条违例都没有。
+//! - 分配器怎么算它们：重开之后的可写挂载从 mkfs 那一版的账重建，这两个落点不在任何一条分配记录里；记录 jsn 3 在水位之上、
+//!   没被施加，C694 的 C（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条，删除线第 33 批 T）把它点名的六个落点隔离，
+//!   这次挂载写行那次发布取的是别的槽、一个都不碰它们（改 C 之前取的正是这六个槽、把它们盖掉）。之后 checker 仍一条违例都没有。
 //!
 //! 条款怎么定（新根段加这两样 / 这种发布整次不施加 / 别的）交用户；这里只记今天的结局。
 
@@ -63,7 +64,7 @@ fn assert_no_invariant_is_violated(image: &MemoryPool, step: &str) {
 }
 
 #[test]
-fn c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_are_free_slots_that_the_next_mount_overwrites(
+fn c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_stay_isolated_from_the_next_mount(
 ) {
     let mut formatted = format_pool("c533-row-publish-record-without-root");
     let mut first = formatted.reopen_recorded();
@@ -270,7 +271,7 @@ fn c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_nev
         "两个孤单元两块盘上都改坏之后，I-8.11 之外 checker 的判决逐条不变"
     );
 
-    // 重开可写挂载：账从 mkfs 那一版重建，那两个落点是空闲槽；写行那次发布取到同样的两个落点、把它们盖掉。
+    // 重开可写挂载：账从 mkfs 那一版重建，那两个落点不在账里；记录 jsn 3 在水位之上没施加，C 把它点名的六个落点隔离，写行那次发布取别的槽。
     let third_mount = mount_writable(&parameters(), &mut devices).expect("崩溃之后重开可写挂载");
     assert_eq!(
         third_mount.output.instance,
@@ -304,32 +305,36 @@ fn c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_nev
         (CheckpointTxg(4), 4),
         "txg 与 jsn 都从环里那条孤记录（txg 3、jsn 3）之后接"
     );
-    assert_eq!(
-        unit_slots_of_the_row_publish(&rewritten_row_publish.record.named),
-        slots_of_the_crashed_row_publish,
-        "分配器不认得崩掉那次写的六个落点：这次写行取到的正是它们，次序也一样"
+    let slots_of_the_rewritten_row_publish =
+        unit_slots_of_the_row_publish(&rewritten_row_publish.record.named);
+    assert!(
+        slots_of_the_rewritten_row_publish
+            .iter()
+            .all(|slot| !slots_of_the_crashed_row_publish.contains(slot)),
+        "这次写行取的落点 {slots_of_the_rewritten_row_publish:?} 一个都不是崩掉那次写的 {slots_of_the_crashed_row_publish:?}（C 隔离了它们）"
     );
-    let mut orphan_placements_in_the_rebuilt_records = third_mount
-        .allocator
-        .records()
-        .iter()
-        .filter(|record| slots_of_the_crashed_row_publish.contains(&record.slot))
-        .map(|record| (record.device, record.slot, record.is_released))
-        .collect::<Vec<_>>();
-    orphan_placements_in_the_rebuilt_records.sort();
-    let mut expected_records: Vec<(DeviceIdentity, SlotNumber, bool)> = BOTH_DEVICES
-        .iter()
-        .flat_map(|device| {
+    assert!(
+        third_mount
+            .allocator
+            .records()
+            .iter()
+            .all(|record| !slots_of_the_crashed_row_publish.contains(&record.slot)),
+        "挂载之后没有一条分配记录罩着崩掉那次写的六个槽"
+    );
+    for device in BOTH_DEVICES {
+        let device_map = third_mount
+            .allocator
+            .devices
+            .iter()
+            .find(|device_map| device_map.device == device)
+            .expect("池里有这块盘");
+        assert!(
             slots_of_the_crashed_row_publish
                 .iter()
-                .map(move |slot| (*device, *slot, false))
-        })
-        .collect();
-    expected_records.sort();
-    assert_eq!(
-        orphan_placements_in_the_rebuilt_records, expected_records,
-        "挂载之后罩着这六个槽的只有这次写行新记的那几条（两盘各六条、都未释放）"
-    );
+                .all(|slot| !device_map.is_free(*slot)),
+            "盘 {device:?} 上崩掉那次写的六个槽都隔离着、不空闲"
+        );
+    }
     assert_no_invariant_is_violated(
         &memory_pool_of_sparse_devices(&devices),
         "崩溃之后重开可写挂载",
diff --git a/crates/singlefs-harness/tests/writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten.rs b/crates/singlefs-harness/tests/writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten.rs
new file mode 100644
index 0000000..b1e038a
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten.rs
@@ -0,0 +1,257 @@
+//! C694 的 C（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条，用户 2026-10-07 定「C 与 B 都做」）：可写挂载读阶段、
+//! 重建分配器之后，把水位之上、没被施加的全部 journal 记录点名的落点隔离（选法 `all`），只在这个进程写掉那条记录的 txg 对应的根环槽、
+//! 那次发布取完落点之后解开（`ring`），进程内重读读回全零的那一槽（D16（发布语义） 已定项 1）读出什么都不解开它；第九项（D28（挂载期承诺量）
+//! 已定项 1）随它扩成分配器实际隔离的集合（判决第三节第 3 条）。
+//!
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次发布 R，崩在 R 的根槽落盘之后、
+//! 系统配置轮换之前；取新实例的那次可写挂载 M1 里 R 的根槽与单元读回全零（第一轮攻方 K1-2 那一形，`ur`），所选那一版落到 R 之前那一版 P，
+//! R 的记录读得出、点名的单元验不过、不施加。
+mod common_unwitnessed_publish;
+
+use common_unwitnessed_publish::{
+    devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
+    image_of_devices_reading_back_all_zero, pool_after_the_mount,
+    record_one_publish_after_overwrites, recorded_devices_reading_back_all_zero_on,
+    units_of_r_overwritten_on, PublishRecordedAsR, RangesReadingBackAllZero, RecordedPublish,
+    OVERWRITE_BYTES,
+};
+use singlefs_core::address::{DeviceIdentity, SlotNumber};
+use singlefs_core::admission::{admission_reading_before_a_publish, BytesOnOneDevice};
+use singlefs_core::journal::JournalRecord;
+use singlefs_core::mount::mount_writable;
+use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
+use singlefs_core::root_ring::target_for_publish;
+use singlefs_core::unit::unit_filesystem_identifier;
+use singlefs_format::SLOT_BYTES;
+use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::SharedStream;
+
+/// 覆盖写几次之后录 R：第一轮攻方 K1-2 与第二轮攻方第一、二节都取 k = 5（384 槽这一档里 R 那次发布新开了一段，P 的账里那一段是空的，
+/// 取新实例那次挂载的提交内生块开到同一段上）。
+const OVERWRITES_BEFORE_R: u64 = 5;
+
+/// M1 里读回全零的：R 的根槽与 R 写出的每个单元（`ur`）。
+fn root_slot_and_units_of_r(recorded: &RecordedPublish) -> RangesReadingBackAllZero {
+    RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.unit_writes())
+}
+
+/// 取新实例那次挂载写行那次发布的根落盘之前断电（第一轮攻方 K1-2）：今天那次挂载取的提交内生块落在 R 的单元上（R 的单元在 P 的账里是空闲槽），
+/// 断电之后恢复择到 R（盘上 txg 最大的根）、走读失败；C 隔离了 R 的记录点名的落点，那次挂载一个都不碰，恢复落到 R 上读回 R 写的内容。
+/// 判别力：C 不隔离（变异「C 一个落点都不隔离」）时 R 的单元被写掉、走读失败，F2 乙再退回 P——落到的不是 R、内容是 P 的，两条断言都红。
+#[test]
+fn a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance_still_reads_back_the_root_whose_units_read_back_zero_at_that_mount(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let stream = SharedStream::retaining_contents();
+    let mut devices = recorded_devices_reading_back_all_zero_on(
+        &crash_image,
+        &root_slot_and_units_of_r(&recorded),
+        &stream,
+    );
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("系统配置没见证 R：取新实例的那次可写挂载做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 的根槽读回全零、点名的单元验不过：所选那一版是 P"
+    );
+    let (mount_writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &stream.retained_operations(),
+        &fixed_geometry_of_the_384_slot_small_devices(),
+    );
+    let row_root = mount_writes
+        .iter()
+        .position(|write| write.kind == StepKind::RootRecordFua)
+        .expect("那次挂载写行那次发布有根槽写");
+    let mut image_before_the_row_root = crash_image.clone();
+    image_before_the_row_root.apply_writes(&mount_writes[..row_root]);
+    assert_eq!(
+        units_of_r_overwritten_on(&recorded, &image_before_the_row_root),
+        Vec::<(DeviceIdentity, u64)>::new(),
+        "写行那次发布的根落盘之前，那次挂载写下的一切都没碰 R 的单元（C 隔离了 R 的记录点名的落点）"
+    );
+    let report = recover(&image_before_the_row_root, JournalPolicy::Consult);
+    assert_eq!(
+        report.effective_root,
+        Some(recorded.key),
+        "写行那次的根没落：恢复择到盘上 txg 最大的根 R"
+    );
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::FileRead {
+            root: recorded.key,
+            content: recorded.content.clone(),
+        },
+        "落到 R 上读回 R 写的内容（今天走读失败：UnitUnreadable）"
+    );
+}
+
+/// 第九项（D28（挂载期承诺量） 已定项 1）随 C 扩成分配器实际隔离的集合（判决第三节第 3 条）：M1 之后每块盘上分配器隔离的槽数，
+/// 就是 R 写出的单元在那块盘上占的槽数（R 的根槽那一次读回全零，环里没有读得出的被抛弃根，影子账那一部分是 0，隔离的全是 C 的），
+/// 准入读数的第九项逐盘等于它。判别力：C 不隔离时两样都是 0，「隔离数等于 R 的单元槽数」那一条红。
+#[test]
+fn the_ninth_admission_term_equals_the_slots_isolated_for_the_records_above_the_water_mark_on_every_device(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let mut devices =
+        devices_reading_back_all_zero_on(&crash_image, &root_slot_and_units_of_r(&recorded));
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("系统配置没见证 R：可写挂载做得成");
+    let reading =
+        admission_reading_before_a_publish(&mounted.allocator, mounted.current.file_version());
+    for device_map in &mounted.allocator.devices {
+        let slots_of_r_on_this_device: u64 = recorded
+            .unit_writes()
+            .into_iter()
+            .filter(|write| write.device == device_map.device)
+            .map(|write| write.length_in_bytes() / SLOT_BYTES)
+            .sum();
+        assert_eq!(
+            device_map.isolated_slots(),
+            slots_of_r_on_this_device,
+            "盘 {:?}：分配器隔离的槽数 = R 写出的单元在这块盘上占的槽数（C 那一部分）",
+            device_map.device
+        );
+        let terms = reading
+            .per_device()
+            .iter()
+            .find(|terms| terms.device == device_map.device)
+            .expect("读数里每块盘一项");
+        assert_eq!(
+            terms.isolated_for_abandoned_roots_and_unapplied_journal_records,
+            BytesOnOneDevice::of_slots(device_map.isolated_slots()),
+            "盘 {:?}：第九项 = 分配器隔离的槽数（含 C 那一部分）",
+            device_map.device
+        );
+    }
+}
+
+/// 第二轮攻方第一节那一形：M1 里 R 的根槽读回全零（钉在回收门槛里），之后照常覆盖写；R 的根槽成了环里最小的那个时，进程内记根先重读那一槽、
+/// 读出 R、按实例表判被抛弃（D16 已定项 1 那一行，今天把它当没有根）。C 隔离的 R 的落点不随这一次重读解开，只在这个进程写掉 R 那一槽、
+/// 那次发布取完落点之后解开：R 的根槽被盖掉之前（含盖掉它的那一次发布）R 的单元一个都不被取到。
+/// 每次覆盖写之后核两样：R 的单元盘上没被写掉；R 那一槽还没被盖掉时，R 写出的每一槽在分配器上不空闲（落点政策取不取到它们看运气，
+/// 不空闲不看运气）。判别力：解开退成随重读解开（变异「C 随 D16 那一行的重读解开」）时，重读出 R 的那一次发布之后 R 的槽就空闲了，「不空闲」那一条红。
+#[test]
+fn the_units_named_by_the_record_of_a_root_reread_as_abandoned_stay_isolated_until_this_process_overwrites_its_root_ring_slot(
+) {
+    let recorded =
+        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let mut devices =
+        devices_reading_back_all_zero_on(&crash_image, &root_slot_and_units_of_r(&recorded));
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("系统配置没见证 R：可写挂载做得成");
+    let image_after_the_mount =
+        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
+    let mut pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
+    let slots_per_region = recorded.pool.parameters.geometry.root_ring_slots_per_region;
+    let ring_slot_of_r = target_for_publish(recorded.key.1, slots_per_region);
+    let overwrites_limit = 4 * slots_per_region.count();
+    let mut overwrites = 0;
+    // 迭代上界 overwrites_limit（根环三区、每区 S 槽，一圈 3S 次发布之内必写到 R 那一槽）；每一轮发一次覆盖写、核一次。
+    loop {
+        pool.overwrite(OVERWRITE_BYTES)
+            .expect("384 槽小盘上这一串覆盖写都落得下");
+        overwrites += 1;
+        let current_txg = pool.session().current.root().checkpoint_txg;
+        assert_eq!(
+            units_of_r_overwritten_on(&recorded, &pool.image()),
+            Vec::<(DeviceIdentity, u64)>::new(),
+            "第 {overwrites} 次覆盖写（txg {current_txg:?}）之后，R 的根槽还没被这个进程盖掉或刚被盖掉：R 的单元一个都没被取到"
+        );
+        if current_txg > recorded.key.1
+            && target_for_publish(current_txg, slots_per_region) == ring_slot_of_r
+        {
+            break;
+        }
+        // R 那一槽还没被这个进程盖掉：R 写出的每个落点在分配器上都不空闲（隔离着，或被候选根与现行账引用着）。
+        let session = pool.session();
+        for write in recorded.unit_writes() {
+            let device_map = session
+                .allocator
+                .devices
+                .iter()
+                .find(|device_map| device_map.device == write.device)
+                .expect("单元写的盘在池里");
+            assert!(
+                !device_map.is_free(SlotNumber(write.offset.0 / SLOT_BYTES)),
+                "第 {overwrites} 次覆盖写（txg {current_txg:?}）之后 R 那一槽还没被盖掉：R 写在盘 {:?} 偏移 {} 的单元那一槽不空闲",
+                write.device,
+                write.offset.0
+            );
+        }
+        assert!(
+            overwrites < overwrites_limit,
+            "{overwrites_limit} 次覆盖写之内这个进程写到 R 那一槽"
+        );
+    }
+    assert!(
+        overwrites > slots_per_region.count(),
+        "走到 R 那一槽被盖掉的那一次发布要转过大半圈根环（R 的根槽在那之前成了环里最小的那个，进程内记根重读过它）"
+    );
+}
+
+/// 选法 `all`（判决第三节第 1 条）：R 是一次三条记录的发布（顺序写三个数据单元），M1 里 R 的根槽与末条记录读回全零、单元照读
+/// （末条读不出、前两条读得出、点名的单元验得过：F1 乙不拒，那是正常崩溃在写记录中途的样子）。前两条记录点名的每个落点 C 都隔离——
+/// M1 之后分配器上它们不空闲。判别力：选法退成只认带末条标志的记录（变异「C 只认末条」）时这次发布一个落点都不隔离，「不空闲」那一条红。
+#[test]
+fn the_units_named_by_the_readable_records_of_a_publish_whose_last_record_reads_back_zero_are_isolated(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let last_record_writes = recorded.record_writes_of_the_record(3);
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&last_record_writes);
+    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("末条读不出、读得出的那两条点名的单元验得过：F1 乙不拒，可写挂载做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 的根槽读回全零、末条读不出：R 整次不施加，所选那一版是 P"
+    );
+    let filesystem_identifier =
+        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier);
+    for ordinal in [1, 2] {
+        let record_bytes = recorded.record_writes_of_the_record(ordinal)[0]
+            .bytes()
+            .expect("记录写带字节");
+        let record =
+            JournalRecord::parse(record_bytes, filesystem_identifier).expect("R 的记录解得开");
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
diff --git a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
index 9e04600..b7700ae 100644
--- a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -2016,8 +2016,16 @@ fn crash_recovery_abandoning_the_unwitnessed_row_publish_of_the_version_without_
         UnreadableRangeReadBack::DeviceError,
     );
     let mut third_with_the_fault = with_unreadable_ranges(crashed, &unreadable);
-    let third_mount = mount_writable(&publish_parameters, &mut third_with_the_fault)
-        .expect("M3：写行那条根读不出、系统配置只见证到 (1, 2)，择根落到 (1, 2)，照常可写挂载");
+    // M3 关着影子账（只供测试的开关，连 C694 的 C 一起关，删除线第 33 批 T）：(2, 3) 的记录读得出、在水位之上没施加，产品路径的 C
+    // 会把它点名的落点隔离、M3 不再是看不见它的那次挂载；这条用例要的是看不见它、把它的槽发出去之后，之后的挂载与抬 F 怎么护。
+    let third_mount = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
+        &publish_parameters,
+        &mut third_with_the_fault,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::Off,
+        BeforeTheOneReread::RereadImmediately,
+    )
+    .expect("M3：写行那条根读不出、系统配置只见证到 (1, 2)，择根落到 (1, 2)，照常可写挂载");
     let mut third = without_unreadable_ranges(third_with_the_fault);
     assert_eq!(
         third_mount.output.rereads.read_stage,
diff --git a/crates/singlefs-harness/tests/writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero.rs b/crates/singlefs-harness/tests/writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero.rs
new file mode 100644
index 0000000..e282e72
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero.rs
@@ -0,0 +1,124 @@
+//! F1 乙（用户 2026-10-07 定「缺记录就拒可写」；判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 6 条）：可写挂载时水位之上一次
+//! 没施加的发布，读得出的那几条记录的「本次发布内序号」不是 1..N 连着（I-8.9）或没有一条带末条标志，而它们点名的单元又验不过，
+//! 重读一次仍这样就在取号之前拒可写（`StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`），
+//! 盘上逐字节不变；读得出的记录点名的单元验得过（正常崩溃在写记录中途）不拒。
+//!
+//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次三条记录的发布 R（顺序写三个数据单元），
+//! 崩在 R 的根槽落盘之后、系统配置轮换之前；挂载时 R 的根槽、R 写出的每个单元与 R 的某一条记录读回全零（第二轮攻方 2.2 那三种藏法：
+//! 中间一条 `urM`、第一条 `urF`、末条 `urL`）。这三形上 C 看不见缺的那条记录点名了哪几个单元（判别子观测不到），不拒的话那几个单元
+//! 第一次覆盖写就被写掉、R 还在根环里（攻方三种藏法各 48 / 73 结点 I-7.4 红）。
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
+use singlefs_harness::memory_pool::RetainedWrite;
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
+/// R 的根槽、R 写出的每个单元与 R 的第 `hidden_record` 条记录读回全零时挂一次可写：拒成 F1 那一个成员、点名 R，盘上逐字节不变。
+fn assert_the_writable_mount_is_refused_before_any_write_when_record_reads_back_zero(
+    hidden_record: usize,
+) {
+    let recorded = recorded_publish_of_three_records();
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.unit_writes())
+        .with_the_writes(&recorded.record_writes_of_the_record(hidden_record));
+    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let refusal = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect_err("R 缺一条记录、读得出的记录点名的单元验不过：重读一次仍这样，拒可写");
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
+        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes) == crash_image,
+        "在取号之前拒：两块盘逐字节不变，R 的单元一个都没被写掉"
+    );
+}
+
+/// `urM`：中间那一条读回全零，读得出的序号是 1、3（不连着）。判别力：判法不认序号缺口（变异「F1 不认序号缺口」）时不拒，这一条红。
+#[test]
+fn a_publish_whose_middle_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write(
+) {
+    assert_the_writable_mount_is_refused_before_any_write_when_record_reads_back_zero(2);
+}
+
+/// `urF`：第一条读回全零，读得出的序号是 2、3（不从 1 起）。判别力：同上一条（变异「F1 不认序号缺口」）。
+#[test]
+fn a_publish_whose_first_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write(
+) {
+    assert_the_writable_mount_is_refused_before_any_write_when_record_reads_back_zero(1);
+}
+
+/// `urL`：末条读回全零，读得出的序号 1、2 连着而没有一条带末条标志。判别力：判法不认缺末条（变异「F1 不认缺末条」）时不拒，这一条红。
+#[test]
+fn a_publish_whose_last_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write(
+) {
+    assert_the_writable_mount_is_refused_before_any_write_when_record_reads_back_zero(3);
+}
+
+/// 正常崩溃在写记录中途：R 的单元都落了（单元在记录之前落盘，D16（发布语义） 已定项 7），记录只落了前两条，根没落、轮换没落。
+/// 读得出的两条序号 1、2、没有末条，而它们点名的单元验得过：不拒，可写挂载照常做成（R 整次不施加，所选那一版是 P）。
+/// 判别力：判法不看点名的单元验不验得过（变异「F1 不看点名单元」）时这里也拒，这一条红。
+#[test]
+fn a_power_loss_while_writing_the_records_of_a_publish_whose_units_verify_still_mounts_writable() {
+    let recorded = recorded_publish_of_three_records();
+    let first_two_records: Vec<&RetainedWrite> = recorded
+        .record_writes_of_the_record(1)
+        .into_iter()
+        .chain(recorded.record_writes_of_the_record(2))
+        .collect();
+    let mut crash_image = recorded.base.clone();
+    let persisted: Vec<RetainedWrite> = recorded
+        .writes
+        .iter()
+        .filter(|write| {
+            write.kind == StepKind::UnitWrite
+                || first_two_records
+                    .iter()
+                    .any(|record_write| record_write == write)
+        })
+        .cloned()
+        .collect();
+    crash_image.apply_writes(&persisted);
+    let mut devices =
+        devices_reading_back_all_zero_on(&crash_image, &RangesReadingBackAllZero::default());
+    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
+        .expect("崩在写记录中途、读得出的记录点名的单元验得过：可写挂载照常做成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 的末条没落：R 整次不施加，所选那一版是 P"
+    );
+}
````

## 追加的变异行（/home/fy5090/.local/share/singlefs/work/m3-batch33/t/patch/mutations-append.tsv）

````tsv
第33批T C694 的 C 一个落点都不隔离（取新实例那次挂载写行根落盘之前断电，R 的单元已被写掉）	crates/singlefs-core/src/mount.rs	    for record in records.values().filter(|record| {\n        (record.instance, record.checkpoint_txg)\n            > (selected_version.instance, selected_version.checkpoint_txg)\n    }) {\n        let new_root_segment_pointers = [	    for record in records.values().filter(|_record| false) {\n        let new_root_segment_pointers = [	-p singlefs-harness --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten -- a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance_still_reads_back_the_root_whose_units_read_back_zero_at_that_mount	a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance_still_reads_back_the_root_whose_units_read_back_zero_at_that_mount
第33批T C694 的 C 一个落点都不隔离（D28 已定项 1 第九项不含 C 那一部分）	crates/singlefs-core/src/mount.rs	    for record in records.values().filter(|record| {\n        (record.instance, record.checkpoint_txg)\n            > (selected_version.instance, selected_version.checkpoint_txg)\n    }) {\n        let new_root_segment_pointers = [	    for record in records.values().filter(|_record| false) {\n        let new_root_segment_pointers = [	-p singlefs-harness --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten -- the_ninth_admission_term_equals_the_slots_isolated_for_the_records_above_the_water_mark_on_every_device	the_ninth_admission_term_equals_the_slots_isolated_for_the_records_above_the_water_mark_on_every_device
第33批T C694 的 C 选法退成只认带末条标志的记录（last）	crates/singlefs-core/src/mount.rs	    for record in records.values().filter(|record| {\n        (record.instance, record.checkpoint_txg)\n            > (selected_version.instance, selected_version.checkpoint_txg)\n    }) {\n        let new_root_segment_pointers = [	    for record in records.values().filter(|record| {\n        (record.instance, record.checkpoint_txg)\n            > (selected_version.instance, selected_version.checkpoint_txg)\n            && record.place_in_publish\n                == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish\n    }) {\n        let new_root_segment_pointers = [	-p singlefs-harness --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten -- the_units_named_by_the_readable_records_of_a_publish_whose_last_record_reads_back_zero_are_isolated	the_units_named_by_the_readable_records_of_a_publish_whose_last_record_reads_back_zero_are_isolated
第33批T C694 的 C 随 D16 已定项 1 读回全零那一行的进程内重读解开（d16）	crates/singlefs-core/src/allocator.rs	                    RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {\n                        occupancy.occupants.remove(&ring_slot);\n                    }	                    RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {\n                        occupancy.occupants.remove(&ring_slot);\n                        let released_on_the_reread = occupancy.placements_named_by_unapplied_journal_records.remove(&ring_slot).unwrap_or_default();\n                        occupancy.placements_named_by_unapplied_journal_records.entry(ring_slot_of_this_root).or_default().extend(released_on_the_reread);\n                    }	-p singlefs-harness --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten -- the_units_named_by_the_record_of_a_root_reread_as_abandoned_stay_isolated_until_this_process_overwrites_its_root_ring_slot	the_units_named_by_the_record_of_a_root_reread_as_abandoned_stay_isolated_until_this_process_overwrites_its_root_ring_slot
第33批T C694 的 B 凭据恒不成立（被抛弃根的账读不出一律照拒）	crates/singlefs-core/src/mount.rs	    if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {	    if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) && false {	-p singlefs-harness --test abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots -- an_abandoned_root_whose_account_slots_were_taken_by_later_units_no_longer_refuses_unmount_floor_raise_or_remount	an_abandoned_root_whose_account_slots_were_taken_by_later_units_no_longer_refuses_unmount_floor_raise_or_remount
第33批T C694 的 B 凭据不看现行账、读不出就放（第一轮攻方 A）	crates/singlefs-core/src/mount.rs	    if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {	    if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) || true {	-p singlefs-harness --test abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots -- an_abandoned_root_whose_account_slots_only_read_back_zero_still_refuses_the_unmount_before_any_write	an_abandoned_root_whose_account_slots_only_read_back_zero_still_refuses_the_unmount_before_any_write
第33批T F1 乙不认序号缺口（中间一条读不出）	crates/singlefs-core/src/mount.rs	            let ordinals_run_from_one_without_a_gap = ordinals\n                .iter()\n                .zip(1..)\n                .all(|(ordinal, expected)| *ordinal == expected);	            let ordinals_run_from_one_without_a_gap = !ordinals.is_empty();	-p singlefs-harness --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero -- a_publish_whose_middle_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write	a_publish_whose_middle_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write
第33批T F1 乙不认序号缺口（第一条读不出）	crates/singlefs-core/src/mount.rs	            let ordinals_run_from_one_without_a_gap = ordinals\n                .iter()\n                .zip(1..)\n                .all(|(ordinal, expected)| *ordinal == expected);	            let ordinals_run_from_one_without_a_gap = !ordinals.is_empty();	-p singlefs-harness --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero -- a_publish_whose_first_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write	a_publish_whose_first_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write
第33批T F1 乙不认缺末条	crates/singlefs-core/src/mount.rs	            let carries_a_last_record = records_of_the_publish.iter().any(|record| {	            let carries_a_last_record = true || records_of_the_publish.iter().any(|record| {	-p singlefs-harness --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero -- a_publish_whose_last_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write	a_publish_whose_last_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write
第33批T F1 乙不看点名单元验不验得过（正常崩溃在写记录中途也拒）	crates/singlefs-core/src/mount.rs	            !(ordinals_run_from_one_without_a_gap && carries_a_last_record)\n                && some_named_unit_does_not_verify	            !(ordinals_run_from_one_without_a_gap && carries_a_last_record)\n                && (some_named_unit_does_not_verify || true)	-p singlefs-harness --test writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero -- a_power_loss_while_writing_the_records_of_a_publish_whose_units_verify_still_mounts_writable	a_power_loss_while_writing_the_records_of_a_publish_whose_units_verify_still_mounts_writable
第33批T F2 乙末条读得出那一支恒见证过（不退）	crates/singlefs-core/src/recovery.rs	        (Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter,	        (Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter || true,	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_root_whose_units_were_overwritten_while_its_record_reads_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts	a_root_whose_units_were_overwritten_while_its_record_reads_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts
第33批T F2 乙末条读得出那一支恒见证过（S1 那一形不退）	crates/singlefs-core/src/recovery.rs	        (Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter,	        (Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter || true,	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- user_accepted_cost_of_2026_10_07_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_reads_back_the_previous_version	user_accepted_cost_of_2026_10_07_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_reads_back_the_previous_version
第33批T F2 乙拿 c_见证 那条比的一支恒见证过（不退）	crates/singlefs-core/src/recovery.rs	            witnessed_publish >= order_of_the_root_choice(selected_version)	            witnessed_publish >= order_of_the_root_choice(selected_version) || true	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts	a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts
第33批T F2 乙两条都读不出就退（攻方 loose）	crates/singlefs-core/src/recovery.rs	        (None, None) => true,	        (None, None) => false,	-p singlefs-harness --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed -- a_confirmed_root_whose_last_record_and_units_read_back_zero_is_not_replaced_by_the_previous_version	a_confirmed_root_whose_last_record_and_units_read_back_zero_is_not_replaced_by_the_previous_version
第33批T C695 甲映射根是空指针时照旧先读映射	crates/singlefs-core/src/mounted_read.rs	    if root.mapping_root == NodePointer::empty_root() {	    if root.mapping_root == NodePointer::empty_root() && false {	-p singlefs-harness --test mounted_read_reports_an_empty_tree_table_before_reading_an_empty_mapping_root -- every_root_of_a_pool_without_a_file_opens_to_an_empty_tree_table_instead_of_an_unreadable_slot_zero	every_root_of_a_pool_without_a_file_opens_to_an_empty_tree_table_instead_of_an_unreadable_slot_zero
第33批T C692 G2 下限退成默认几何的常数 14（1024 MiB 环那套几何每盘少两条判不出）	crates/singlefs-core/src/recovery.rs	            return PlacementsOnEachDevice(rehearsal.records().len() / device_count);	            return PlacementsOnEachDevice(14);	-p singlefs-core --lib -- allocation_records_per_device_tests	the_one_gibibyte_journal_ring_geometry_has_sixteen_placements_on_each_device_after_the_new_pool_file_creation
第33批T C692 G2 下限少算一个落点（默认几何每盘少一条判不出）	crates/singlefs-core/src/recovery.rs	            return PlacementsOnEachDevice(rehearsal.records().len() / device_count);	            return PlacementsOnEachDevice(rehearsal.records().len() / device_count - 1);	-p singlefs-core --lib -- allocation_records_per_device_tests	the_default_geometry_has_fourteen_placements_on_each_device_after_the_new_pool_file_creation
第33批T F2 乙施加过记录、由记录重建的那一版走读失败也退（条款只写了所选根）	crates/singlefs-core/src/recovery.rs	    if (selected_version.instance, selected_version.checkpoint_txg)\n        != (chosen_root.instance, chosen_root.checkpoint_txg)\n    {\n        return None;\n    }	    if (selected_version.instance, selected_version.checkpoint_txg)\n        != (chosen_root.instance, chosen_root.checkpoint_txg)\n        && false\n    {\n        return None;\n    }	-p singlefs-harness --test named_unit_verification_stands_in_for_the_barrier_before_the_record -- without_named_unit_verification_the_same_state_is_a_model_disagreement	without_named_unit_verification_the_same_state_is_a_model_disagreement
````

## 按名字整行替换的已有变异行（/home/fy5090/.local/share/singlefs/work/m3-batch33/t/patch/mutations-replacements.tsv）

````tsv
步 4：被抛弃根的树表读不出时不计数	crates/singlefs-core/src/mount.rs	            None => {\n                unreadable += 1;\n                placements_of_an_abandoned_root_whose_account_is_unreadable(	            None => {\n                placements_of_an_abandoned_root_whose_account_is_unreadable(	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored torn_tree_table_of_an_abandoned_root	torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount
增补 2 第 61 行（C519 已知丢失）：施加前的点名单元验证改成任一条位置条目验过即过（改法 Z2 被悄悄落地时已知丢失那条先红）	crates/singlefs-core/src/recovery.rs	        named.locations.iter().all(|location| {	        named.locations.iter().any(|location| {	-p singlefs-harness --test whole_device_loss_after_warm_up -- c519_known_loss	c519_known_loss_after_the_warm_up_covers_both_devices_losing_the_acknowledged_version_root_device_falls_back_to_the_row_publish_version
C318 实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒 0（影子账隔离的单元又不进不等式）	crates/singlefs-core/src/admission.rs	                isolated_for_abandoned_roots_and_unapplied_journal_records:\n                    BytesOnOneDevice::of_slots(device_map.isolated_slots()),	                isolated_for_abandoned_roots_and_unapplied_journal_records:\n                    BytesOnOneDevice::ZERO,	-p singlefs-harness --test admission_formula_on_the_mounted_state -- after_a_recovery_abandoned_a_root	after_a_recovery_abandoned_a_root_the_admission_reading_with_the_shadow_ledger_is_short_of_the_one_without_it_by_exactly_the_isolated_slots_on_each_device
C318 实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒 0（从分配器取读数那一处）	crates/singlefs-core/src/admission.rs	                isolated_for_abandoned_roots_and_unapplied_journal_records:\n                    BytesOnOneDevice::of_slots(device_map.isolated_slots()),	                isolated_for_abandoned_roots_and_unapplied_journal_records:\n                    BytesOnOneDevice::ZERO,	-p singlefs-core --lib -- admission::tests	reading_of_an_allocator_takes_each_devices_own_capacity_allocated_and_isolated_slots
C318 实现：可用(d) 不扣第九项「被抛弃根独占量」	crates/singlefs-core/src/admission.rs	                    terms.mount_time_commitment,\n                    terms.isolated_for_abandoned_roots_and_unapplied_journal_records,\n	                    terms.mount_time_commitment,\n	-p singlefs-harness --test admission_formula_on_the_mounted_state -- after_a_recovery_abandoned_a_root	after_a_recovery_abandoned_a_root_the_admission_reading_with_the_shadow_ledger_is_short_of_the_one_without_it_by_exactly_the_isolated_slots_on_each_device
C518（一次挂载之内环转过一圈之后不回收）：挂载内回收出来、被抛弃根还引用着的槽不补隔离（D23 已定项 14 主句）	crates/singlefs-core/src/allocator.rs	        self.isolate_reclaimed_slots_the_isolation_still_protects(\n            &reclaimed,\n            &still_protected_by_the_isolation,\n        );	        let _ = (&reclaimed, &still_protected_by_the_isolation);	-p singlefs-harness --test unreadable_abandoned_root_slot -- slot_reclaimed_while_an_abandoned_root_still_references	slot_reclaimed_while_an_abandoned_root_still_references_it_stays_isolated_until_that_root_leaves_the_ring
实三（影子账，理由改指崩溃恢复）：账读不出的被抛弃根不计数	crates/singlefs-core/src/mount.rs	                unreadable += 1;\n                placements_of_an_abandoned_root_whose_account_is_unreadable(	                unreadable += 0;\n                placements_of_an_abandoned_root_whose_account_is_unreadable(	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored torn_tree_table_of_an_abandoned_root	torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount
C554 乙：读阶段第一遍判据 N-配置 为真也不重读、按第一遍往下走（系统配置见证过的最新根暂时读不出照旧被当成被抛弃）	crates/singlefs-core/src/mount.rs	    if !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	    if true || !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	-p singlefs-harness --test a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable -- a_witnessed_newest_root_still_unreadable_after_the_one_reread	a_witnessed_newest_root_still_unreadable_after_the_one_reread_refuses_the_writable_mount_before_any_write_while_the_read_only_mount_reads_the_version_before_it
C554 乙（树表 0 条那一版上）：读阶段第一遍判据 N-配置 为真也不重读、按第一遍往下走	crates/singlefs-core/src/mount.rs	    if !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	    if true || !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	-p singlefs-harness --test writable_mount_of_a_formatted_pool -- crash_recovery_that_cannot_read_the_witnessed_row_publish	crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_chooses_it_when_the_read_fault_is_lifted_before_the_one_reread
实审 A3d：重建不判分配记录每个落点每盘各一条（两盘的账不对称：可写挂载改红在下一道分配记录代的判定上）	crates/singlefs-core/src/recovery.rs	    allocation_records_are_one_placement_per_device_on_every_device(\n        reader,\n        unit_area_start,\n        &allocation_tree_read.records,\n        &device_identities,\n    )?;	    let _ = allocation_records_are_one_placement_per_device_on_every_device(\n        reader,\n        unit_area_start,\n        &allocation_tree_read.records,\n        &device_identities,\n    );	-p singlefs-checker-tier --test bad_disk_input_campaign -- allocation_records_released_on_the_second_device_only	allocation_records_released_on_the_second_device_only_are_refused_at_the_one_record_per_device_judgement_by_both_readers
合入后验证一：C554 乙关掉，崩溃恢复抛弃根又造得出被抛弃的根（落到实例 2）	crates/singlefs-core/src/mount.rs	    if !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	    if true || !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	-p singlefs-checker-tier --test crash_injection_campaign -- units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red	units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red
合入后验证一：C554 乙关掉，种子基 + 2 那段历史里崩溃恢复抛弃根那一步不再被拒	crates/singlefs-core/src/mount.rs	    if !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	    if true || !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	-p singlefs-checker-tier --test crash_injection_campaign -- crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons	crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons
合入后验证一：C554 乙关掉，判别力那一条的历史里崩溃恢复抛弃根那一步不再被拒	crates/singlefs-core/src/mount.rs	    if !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	    if true || !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	-p singlefs-checker-tier --test crash_injection_campaign -- on_the_whole_stream_a_kept_publish_missing_a_unit_under_a_version_rebuilt_from_records_is_still_red	on_the_whole_stream_a_kept_publish_missing_a_unit_under_a_version_rebuilt_from_records_is_still_red
调查三条红（主 agent 定）：C554 乙关掉，读路径那一格（盘 1 每次读都报错）走到逐盘核、不再被乙拒	crates/singlefs-core/src/mount.rs	    if !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	    if true || !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	-p singlefs-harness --test fsync_drop_and_devices_without_the_selected_version -- mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write	mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write
调查三条红（主 agent 定）：C554 乙关掉，盘 0 是空盘那一格走到逐盘核、不再被乙拒	crates/singlefs-core/src/mount.rs	    if !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	    if true || !first_read.witnesses_a_publish_newer_than_the_selected_version()\n        && first	-p singlefs-harness --test fsync_drop_and_devices_without_the_selected_version -- blank_device_refuses_the_writable_mount_by_name_before_any_write	blank_device_refuses_the_writable_mount_by_name_before_any_write
三·2 第 6 条：可用(d) 不扣环窗口（式子末项漏扣）	crates/singlefs-core/src/admission.rs	                    terms.isolated_for_abandoned_roots_and_unapplied_journal_records,\n                    terms.ring_window,\n	                    terms.isolated_for_abandoned_roots_and_unapplied_journal_records,\n	-p singlefs-harness --test admission_ring_window_and_tombstone_prepay_on_the_publish_path	the_publish_reading_on_the_384_slot_small_devices_deducts_twenty_five_times_thirty_slots
遗留改动代码轮 LR4（C393 (b)-从映射现算最后一支）：被抛弃根的账与从映射现算重读一次仍都不成时不拒可写，当它什么都不引用	crates/singlefs-core/src/mount.rs	    if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {\n        return Ok(Vec::new());\n    }\n    Err(MountError::NewerStateStillUnreadableAfterOneReread(\n        Box::new(StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {\n            abandoned_root: RollbackTarget {\n                instance: root.instance,\n                checkpoint_txg: root.checkpoint_txg,\n            },\n        }),\n    ))	    let _ = (root, current_records);\n    Ok(Vec::new())	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area	an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking
第10l批 新类被抛弃那一半被换下时不清隔离（4 GiB、覆盖写 1 次：C 离开根环之后它写出的落点仍隔离、不空闲）	crates/singlefs-core/src/allocator.rs	            Some(\n                RootRingOccupant::AbandonedRoot {\n                    referenced_placements,\n                }\n                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                    root:\n                        RootReadOnOneRebuildPass::AbandonedRoot {\n                            referenced_placements,\n                        },\n                    ..\n                },\n            ) => self.clear_isolation_nothing_still_protected_backs(	            Some(RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::AbandonedRoot { .. },\n                ..\n            }) => {}\n            Some(RootRingOccupant::AbandonedRoot {\n                referenced_placements,\n            }) => self.clear_isolation_nothing_still_protected_backs(	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring
增补 2 第 62 行（C533 复现）：前缀跨实例边界，写行那条孤记录进了水位之上	crates/singlefs-replay-prefix/src/lib.rs	    record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0	    record.instance.0 > root_instance.0 || (record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0)	-p singlefs-harness --test row_publish_record_without_its_root -- c533_row_publish_record_persisted	c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_stay_isolated_from_the_next_mount
C554 乙 模型跟进：可写挂载只写上一个实例那一行、不写中间实例行（烧掉的号那一形护；崩溃恢复写 (2, 0, 0) 那一形乙之后拒可写、checker 那份改走事务层）	crates/singlefs-core/src/mount.rs	    (first_row_instance..instance_to_acquire.0)	    (first_row_instance..first_row_instance + 1)	-p singlefs-harness --test row_publish_record_without_its_root	c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_stay_isolated_from_the_next_mount
````
