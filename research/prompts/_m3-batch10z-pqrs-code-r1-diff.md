# 根环实现第 10z 批第十六至十九块：代码轮第一轮改动全文（2026-10-09）

四块实现员各交的 `crates.patch` 原样，按合入次序：第十六块、第十八块、第十七块、第十九块（第十九块是第十七块之上的增量）。各块的变异表追加、整行替换附在各自后面。

## 第十六块

````diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 5aec647..d7e36bc 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -640,6 +640,15 @@ impl DeviceFreeMap {
         self.isolated_slots
     }
 
+    /// 这一槽的影子账隔离位（[`Self::isolate`] 那一套）置着；单元区之外的槽交 `false`。只给会话数「C 那一圈这一刻隔离着几个槽」读
+    /// （`PoolAllocator::isolated_slots_covered_by`，推满仍不够时报不报「被宽护卡住」），分配与准入的判定不读它。
+    #[must_use]
+    pub fn is_isolated(&self, slot: SlotNumber) -> bool {
+        let lies_in_the_unit_area =
+            (self.unit_area_start.slot().0..self.unit_area_end_slot()).contains(&slot.0);
+        lies_in_the_unit_area && self.isolated[self.index(slot)]
+    }
+
     /// 清掉一个槽的影子账隔离位（`isolate` 置的那一套，D28（挂载期承诺量） 已定项 1 第九项「被抛弃的根被轮转覆写时清零」）：
     /// 只动这一位与它的两个计数，分配位与抬 F 扣住那一位照旧——清完之后它发不发得出去看别的位。这一位没置着就什么都不动。
     pub fn clear_isolation_of_slot(&mut self, slot: SlotNumber) {
@@ -1253,6 +1262,45 @@ impl RootRingOccupancy {
             .collect()
     }
 
+    /// C 那一圈（水位之上没施加的记录点名的落点、那几次发布的账、比所选那一版新的可读根的账，认不出回退时的宽护在其中，[`RootRingOccupancy`]
+    /// 那一项的注释）还没解开、记着落点的槽，各带记着的落点，按槽的次序：这个进程写掉那一槽之前它们一直罩着（回收出来的照补隔离）。
+    /// 会话推环推到哪一槽、推满仍不够时报不报「被宽护卡住」读它（`mounted_session`，用户 2026-10-08 弹窗定「显式推环命令加带出路的错」）。
+    /// 记着空串的槽（凭据放行那一形）不在其内：它什么都不罩。只读内存里的表。
+    #[must_use]
+    pub fn ring_slots_holding_placements_named_by_unapplied_journal_records(
+        &self,
+    ) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
+        self.placements_named_by_unapplied_journal_records
+            .iter()
+            .filter(|(_, placements)| !placements.is_empty())
+            .map(|(ring_slot, placements)| (*ring_slot, placements.clone()))
+            .collect()
+    }
+
+    /// 这一槽上表里记着的有效根的 txg（挂载那一刻读出、按最新那条根的实例表有效的，进程内重读读出的，两遍里只有一遍自证过而有效的，
+    /// 或这个进程写的）；记的是被抛弃根、挂载那一刻读坏或读回全零的槽，或表里没记这一槽，交 `None`。只读内存里的表。
+    #[must_use]
+    pub fn checkpoint_txg_of_the_valid_root_recorded_on(
+        &self,
+        ring_slot: RootRingSlot,
+    ) -> Option<CheckpointTxg> {
+        match self.occupants.get(&ring_slot)? {
+            RootRingOccupant::ValidRoot { checkpoint_txg }
+            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { checkpoint_txg }
+            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                root: RootReadOnOneRebuildPass::ValidRoot { checkpoint_txg },
+                ..
+            } => Some(*checkpoint_txg),
+            RootRingOccupant::AbandonedRoot { .. }
+            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
+                ..
+            }
+            | RootRingOccupant::BadAtMount(_)
+            | RootRingOccupant::AllZeroAtMount { .. } => None,
+        }
+    }
+
     /// 环里还在的被抛弃根引用着的全部落点，加还没解开的、水位之上没施加的记录点名的全部落点：清隔离位之前要看的那一份
     /// （一个槽两边任一边还罩着就不清），回收出来的槽里它们罩着的补隔离。
     fn placements_the_isolation_still_protects(&self) -> Vec<PlacementOnDevice> {
@@ -2180,6 +2228,50 @@ impl PoolAllocator {
         device_map.isolate(slot, span);
     }
 
+    /// 同 [`Self::isolate_abandoned`]，只隔离落点里分配位没置着的槽（[`DeviceFreeMap::isolate_the_slots_not_allocated`]）：被抛弃根引用、
+    /// 现行账里已释放而还在 defer 窗口里的槽分配位置着，算在「已分配」里，D28（挂载期承诺量） 已定项 1 第九项不在这里再算一次
+    /// （第 10z 批第十四块实现员报告第七节第 2 条，与 C 那一圈同一个做法）。它们回收的那一刻由根环表照这条根记着的落点补隔离
+    /// （[`RootRingOccupancy::placements_the_isolation_still_protects`] 里被抛弃根那一份，挂载内记根与抬 F 那一串回收两条路都走它）。
+    /// 只给落点记在根环表里的被抛弃根用：可写挂载那一遍读出的（落点随后记进表里）、抬 F 那一串取表里记着的或借 C 那一圈的；
+    /// 抬 F 那一串读盘得来、表里没记着的那几条照旧 [`Self::isolate_abandoned`]（回收时没有一份补得上它们，`mount::isolate_slots_referenced_only_by_abandoned_roots`）。
+    pub fn isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
+        &mut self,
+        device: DeviceIdentity,
+        slot: SlotNumber,
+        span: u64,
+    ) {
+        let device_map = self
+            .devices
+            .iter_mut()
+            .find(|device_map| device_map.device == device)
+            .expect("被抛弃根的分配记录的盘在池里：盘上读来的记录在 recovery::allocation_records_fit_the_pool_geometry 逐条比过 PoolReader::device_identities，根环表里记着的是同一份");
+        device_map.isolate_the_slots_not_allocated(slot, span);
+    }
+
+    /// `placements` 罩着的槽里这一刻隔离着的槽数，每块盘一个数、按分配器里盘的次序；同一个槽被几个落点罩着只数一次。
+    /// 只给会话报「被宽护卡住」时数 C 那一圈挡着几个槽读（`mounted_session`），分配与准入的判定不读它。
+    #[must_use]
+    pub fn isolated_slots_covered_by(
+        &self,
+        placements: &[PlacementOnDevice],
+    ) -> Vec<(DeviceIdentity, u64)> {
+        self.devices
+            .iter()
+            .map(|device_map| {
+                let isolated_covered: BTreeSet<u64> = placements
+                    .iter()
+                    .filter(|placement| placement.device == device_map.device)
+                    .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
+                    .filter(|slot| device_map.is_isolated(SlotNumber(*slot)))
+                    .collect();
+                (
+                    device_map.device,
+                    u64::try_from(isolated_covered.len()).expect("单元区槽数装得进 u64"),
+                )
+            })
+            .collect()
+    }
+
     /// 装上根环表（`RootRingOccupancy`：挂载时从盘上读出来的，或 mkfs 刚写下的），替掉原来那一张。
     pub fn install_root_ring_occupancy(&mut self, occupancy: RootRingOccupancy) {
         self.root_ring = Some(occupancy);
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 23ff357..3747c50 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1598,8 +1598,31 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
         // 从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报（显式报告）；
         // 现算也不成（映射树读不出、解不开）才重读一次（重读之前调钩子），账与现算都仍不成就在取号之前拒可写、只读照常。
         // 抬 F 那一串先取根环表里记着的那一份（`or_else` 是惰性的：取到就不读盘，读不出的处置也走不到）。
-        let placements = match placements_of_abandoned_roots
-            .recorded_in_the_root_ring_table(root)
+        let recorded_in_the_root_ring_table =
+            placements_of_abandoned_roots.recorded_in_the_root_ring_table(root);
+        // 回收时补不补得上隔离（第 10z 批第十四块实现员报告第七节第 2 条，派发第十六块第 2 条）：根环表里记着这条根的落点（表里的被抛弃根、
+        // 借 C 那一圈的），或可写挂载那一遍（这一遍读出的落点随后整串记进表里，`rebuilt_allocator` 建根环表那一步），回收出来的槽照表补隔离
+        // （`allocator::RootRingOccupancy::placements_the_isolation_still_protects`），分配位置着的槽就不在这里隔离、不与「已分配」双扣；
+        // 抬 F 那一串读盘得来、表里没记着的，回收时没有一份补得上，照旧整段隔离。
+        let reclaim_isolates_what_this_root_references = match (
+            &placements_of_abandoned_roots,
+            &recorded_in_the_root_ring_table,
+        ) {
+            (PlacementsOfAbandonedRoots::ReadFromTheirAccounts, _)
+            | (
+                PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
+                    _,
+                ),
+                Some(_),
+            ) => true,
+            (
+                PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
+                    _,
+                ),
+                None,
+            ) => false,
+        };
+        let placements = match recorded_in_the_root_ring_table
             .or_else(|| placements_referenced_by_root(devices, unit_area_start, root))
         {
             Some(placements) => placements,
@@ -1619,7 +1642,14 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
             if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
                 continue;
             }
-            allocator.isolate_abandoned(device, slot, span_slots);
+            if reclaim_isolates_what_this_root_references {
+                allocator
+                    .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
+                        device, slot, span_slots,
+                    );
+            } else {
+                allocator.isolate_abandoned(device, slot, span_slots);
+            }
         }
         placements_referenced_by_abandoned_root.insert(
             (root.instance, root.checkpoint_txg),
@@ -1630,12 +1660,17 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
         );
     }
     // 根槽这一遍读坏的被抛弃根（H1）：读不出它的根记录，照它挂载时记着的落点补隔离，豁免与读出来的被抛弃根同一个集合。
+    // 落点取自根环表，回收时照表补隔离：分配位置着的槽不在这里隔离（同上一圈）。
     for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad {
         let key = (placement.device.0, placement.slot.0);
         if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
             continue;
         }
-        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);
+        allocator.isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
+            placement.device,
+            placement.slot,
+            placement.span,
+        );
     }
     // C 那一份不豁免候选根引用的槽（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K1 / K2 T1 C 的候选豁免」那一行，
     // 根环线 K4-1）：水位之上没施加的那次发布的根读得出、账读得出时它自己就是候选根（按实例表有效、txg ≥ F），豁免它引用的槽就是不护它的单元——
@@ -4835,8 +4870,8 @@ fn publish_rows_on_file_version<Device: BlockDevice>(
 }
 
 /// 暖机的一次空发布接在带文件的现行那一版后面时的计划：重写固定点单元（记账树存在，D16（发布语义） 已定项 9），不写文件内容、不碰 inode 树、
-/// 实例表照抄。取号之后的发布与取号之前的预演读的是同一张计划。
-fn empty_publish_plan_after_file_version<'plan>(
+/// 实例表照抄。取号之后的发布与取号之前的预演读的是同一张计划；会话的推环（`mounted_session`）发的也是这一张（F 不变）。
+pub(crate) fn empty_publish_plan_after_file_version<'plan>(
     current_file_version: &TransactionOutput,
     instance: InstanceGeneration,
 ) -> PublishPlan<'plan> {
@@ -5867,6 +5902,27 @@ fn caller_inputs_agreeing_with_the_disk<Device: BlockDevice>(
     })
 }
 
+/// 会话的推环入口（`mounted_session::MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`）
+/// 收盘表时走的核：与抬 F 那一串的空发布收盘表同一套（[`caller_inputs_agreeing_with_the_disk`]），交回盘上择到的那份系统配置照它建的参数，
+/// 写入口只用这一份。只读盘。
+///
+/// # Errors
+/// 同 [`caller_inputs_agreeing_with_the_disk`]，装成 `MountError`（与抬 F 那一串交回的同一种）。
+#[allow(
+    clippy::ptr_arg,
+    reason = "caller_inputs_agreeing_with_the_disk 走 &Vec：choose_system_configuration 要一个有大小的读者"
+)]
+pub(crate) fn parameters_of_the_pool_agreeing_with_the_caller_inputs<Device: BlockDevice>(
+    caller_parameters: &MakeFilesystemParameters,
+    devices: &Vec<(DeviceIdentity, Device)>,
+    current: &TransactionOutput,
+) -> Result<MakeFilesystemParameters, MountError> {
+    Ok(
+        caller_inputs_agreeing_with_the_disk(caller_parameters, devices, current)?
+            .parameters_of_the_pool,
+    )
+}
+
 /// 可写挂载准入的第一条合取（D2（RAID 条带策略） 已定项 13「挂载准入」：可写设备数 ≥ w 的下限；D18（块里携带什么信息） 已定项 11
 /// 「可写挂载的顺序」先判这一条）：交进来的不同设备身份数（[`distinct_device_identities_handed_in`]）低于 [`STRIPE_WIDTH_LOWER_BOUND`] 就拒。
 /// 只比两个数，不读盘、不写盘。
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index d8b0d60..180a992 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -17,24 +17,30 @@
 //! 缺根那一半是 C585（会话发布放进发布途中取下的旧快照） 用户定的改法 C）。
 //! 发布路径与抬 F 各自开写入口的做法不变。
 
-use crate::address::{DeviceIdentity, InodeNumber, InstanceGeneration};
-use crate::allocator::PoolAllocator;
+use crate::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
+use crate::allocator::{PlacementOnDevice, PoolAllocator};
 use crate::block_device::BlockDevice;
 use crate::make_filesystem::MakeFilesystemParameters;
 use crate::mount::{
     devices_behind_the_current_version, devices_without_a_self_verified_system_configuration,
+    empty_publish_plan_after_file_version,
     own_device_numbers_on_disk_differing_from_the_identities_handed_in,
+    parameters_of_the_pool_agreeing_with_the_caller_inputs,
     push_one_floor_raise_within_the_admission_budget, DeviceBehindTheCurrentVersion,
     FloorRaisePushedWithinTheAdmissionBudget, FloorRaiseStop, MountError, MountOutput, Mounted,
     OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, ParametersAndDeviceTableOfTheMount,
     RaisedFloor, ShadowLedger,
 };
+use crate::root_record::UnmountMarker;
+use crate::root_ring::{target_for_publish, RootRingSlot};
 use crate::transaction::{
-    publish_new_inodes, publish_overwrite, publish_sequential_write, publish_unlink_intent,
-    walk_the_deletion_queue_until_it_is_empty, DeletionQueueWalk, FirstFile, PoolVersion,
-    PoolWriter, PublishError, TransactionOutput,
+    prepare_the_version_publish, publish_new_inodes, publish_overwrite, publish_sequential_write,
+    publish_unlink_intent, publish_version, walk_the_deletion_queue_until_it_is_empty,
+    DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, PoolVersion, PoolWriter,
+    PublishError, ReleaseChecksumCheck, TransactionOutput,
 };
 use crate::unlink::DeletionSwitches;
+use singlefs_format::ROOT_RING_REGIONS;
 
 /// 一次可写挂载之后的会话：接下来的发布接在 `current` 后面、用这个分配器与实例代号。
 #[derive(Debug)]
@@ -160,6 +166,197 @@ pub struct NoSpaceAfterRaisingTheFloor {
     pub stop: FloorRaiseStop,
     /// 被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）。
     pub versions_published: VersionsPublishedInThisChange,
+    /// 这一次是不是被宽护卡住（[`WideProtectionAtTheCeiling`]）：F 已在上限、C 那一圈还隔离着槽时点名这一形、带上两条出路，
+    /// 别的形是 [`WideProtectionAtTheCeiling::NotWhatStopsTheAdmission`]，照旧只是空间不够。
+    pub wide_protection_at_the_ceiling: WideProtectionAtTheCeiling,
+}
+
+/// 推满仍不够时，挡着准入的是不是「这个进程写掉那一槽才放」的那一份隔离（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」：
+/// 不自动推环，准入因宽护卡住时报一个点名「被宽护卡住」的错，出路写两条）。
+///
+/// 为什么不另开一个 [`UserChangeRefused`] 成员：checker 档的装置（`e158_root_choice_repair`、`e168_delete_ring_window_borrow`）对
+/// `UserChangeRefused` 与 `FloorRaiseStop` 写的是穷举 `match`，多一个成员它们就编不过，而 checker 档这一批不改（第 10z 批第十六块派发）。
+/// 点名这一形的成员住在这里，外层照旧是 `NoSpaceAfterRaisingTheFloor`，交回的已落盘的东西不变。
+#[derive(Debug)]
+pub enum WideProtectionAtTheCeiling {
+    /// 不是这一形：不再推的原因不是「F 已在上限」（预算用完、抬 F 那一串自己被拒），或 C 那一圈这一刻一个槽都没隔离。
+    NotWhatStopsTheAdmission,
+    /// 被宽护卡住（[`StuckBehindTheWideProtection`]）。装箱同 `NoSpaceAfterRaisingTheFloor`。
+    StuckBehindTheWideProtection(Box<StuckBehindTheWideProtection>),
+}
+
+/// 被宽护卡住：准入被拒、会话推抬 F 停在上限（`FloorRaiseStop::FloorAlreadyAtTheCeiling`），而 C 那一圈（水位之上没施加的记录点名的落点、
+/// 那几次发布的账、比所选那一版新的可读根的账；认不出回退时并进 R 那一槽的「现行账里全部已释放的落点」就是宽护，它是这一圈里最大的一份）
+/// 还隔离着槽。这一圈只在这个进程写掉那一槽时解开（`allocator::PoolAllocator::record_root_written_by_this_process`），写被拒就不发布、
+/// 那一槽写不掉，不管它就隔离到进程结束（第 10z 批第十四块实现员报告第六节，384 槽盘 22 形写全被拒）。
+///
+/// 这里分不出 C 那一圈里哪一份是宽护、哪一份是窄的那几份：分得出要在根环表里另记一样（`PoolAllocator` 的 `{:?}` 随之变，checker 档 E171
+/// 记它），这一份不记；两条出路对整圈同样走得通。
+#[derive(Debug)]
+pub struct StuckBehindTheWideProtection {
+    /// 推环要写到的那一槽：C 那一圈还没解开、记着落点的槽里，从现行那一版起按 txg 往后最后一个轮到的（R 那一槽）。
+    pub ring_slot_of_r: RootRingSlot,
+    /// R 的 txg：根环表里那一槽记着的有效根（挂载那一刻按最新那条根的实例表有效、记成有效根，写行之后才被抛弃的那一条）；
+    /// 那一槽记的不是有效根（读坏、读回全零、没有根的记录）时 `None`。
+    pub checkpoint_txg_of_r: Option<CheckpointTxg>,
+    /// 从现行那一版起要发几次 F 不变的空发布才写掉 R 那一槽（不超过环长 3S）。
+    pub empty_publishes_until_the_ring_slot_of_r_is_written: u64,
+    /// C 那一圈每块盘上这一刻隔离着的槽数（按分配器里盘的次序，同一个槽只数一次）：宽护挡着的槽数。
+    pub isolated_slots_on_each_device: Vec<(DeviceIdentity, u64)>,
+}
+
+impl StuckBehindTheWideProtection {
+    /// 两条出路，调用方照抄就能报给用户：卸载重挂；调推环入口（会丢掉最近可退到的状态，调用即确认）。按这个次序。
+    #[must_use]
+    pub fn ways_out(&self) -> [String; 2] {
+        let txg_of_r = self.checkpoint_txg_of_r.map_or_else(
+            || "那一槽没有读得出的有效根".to_string(),
+            |txg| format!("txg {}", txg.0),
+        );
+        [
+            format!(
+                "出路一：正常卸载（mount::unmount）之后再可写挂载一次。再挂时 R（{txg_of_r}）按新实例的实例表判成被抛弃，影子账只护它自己账里引用的落点，宽护不再出现；R 的账再挂时仍读不出就照旧拒可写（C694 的 B）。"
+            ),
+            format!(
+                "出路二：调 MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states（推环到 R 那一槽，根环槽 {}.{}）：发 {} 次 F 不变的空发布写掉那一槽，宽护随之解开、隔离的 {} 个槽放出来；根环里更早的回退候选会被这一串整环写掉，回退候选集掉到少于 4 个不同状态（D16（发布语义） 已定项 1），调用即确认。",
+                self.ring_slot_of_r.region,
+                self.ring_slot_of_r.slot,
+                self.empty_publishes_until_the_ring_slot_of_r_is_written,
+                self.isolated_slots_on_each_device
+                    .iter()
+                    .map(|(_, isolated)| *isolated)
+                    .sum::<u64>(),
+            ),
+        ]
+    }
+}
+
+/// C 那一圈还没解开、记着落点的那几槽，与从 `current_txg` 起推环要推到哪儿：推环入口（[`MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`]）
+/// 与「被宽护卡住」那一报（[`wide_protection_at_the_ceiling`]）读同一份。
+struct RootRingPushTarget {
+    /// 这几槽，按槽的次序。
+    ring_slots: Vec<RootRingSlot>,
+    /// 这几槽记着的全部落点。
+    placements: Vec<PlacementOnDevice>,
+    /// 最后一个轮到的那一槽（R 那一槽）与写掉它要发几次空发布。
+    ring_slot_of_r: RootRingSlot,
+    publishes_until_the_ring_slot_of_r_is_written: u64,
+}
+
+/// [`RootRingPushTarget`]：分配器没装根环表、或 C 那一圈这一刻没有记着落点的槽时 `None`。只读内存。
+///
+/// 每一槽要几次发布：txg 逐个加一、每次发布写 `target_for_publish(txg)` 那一槽（`root_ring::target_for_publish`），环长 3S 个连续的 txg
+/// 里每一槽恰好轮到一次，所以从 `current_txg + 1` 起数到 3S 之内必找到；txg 加上去越过 u64 的那几个当找不到，取满一环 3S
+/// （真发时那一次发布自己就按 txg 到顶拒，`PublishError::NextCheckpointTxgPastTheTopOfItsRange`，在推环的预演里先报出来）。
+fn root_ring_push_target(
+    allocator: &PoolAllocator,
+    current_txg: CheckpointTxg,
+) -> Option<RootRingPushTarget> {
+    let occupancy = allocator.root_ring_occupancy()?;
+    let holding = occupancy.ring_slots_holding_placements_named_by_unapplied_journal_records();
+    let slots_per_region = occupancy.slots_per_region();
+    let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
+    let publishes_until_written = |ring_slot: RootRingSlot| -> u64 {
+        (1..=ring_length_in_txgs)
+            .find(|publishes_ahead| {
+                current_txg
+                    .0
+                    .checked_add(*publishes_ahead)
+                    .is_some_and(|txg| {
+                        target_for_publish(CheckpointTxg(txg), slots_per_region) == ring_slot
+                    })
+            })
+            .unwrap_or(ring_length_in_txgs)
+    };
+    let (publishes_until_the_ring_slot_of_r_is_written, ring_slot_of_r) = holding
+        .keys()
+        .map(|ring_slot| (publishes_until_written(*ring_slot), *ring_slot))
+        .max()?;
+    Some(RootRingPushTarget {
+        ring_slots: holding.keys().copied().collect(),
+        placements: holding.into_values().flatten().collect(),
+        ring_slot_of_r,
+        publishes_until_the_ring_slot_of_r_is_written,
+    })
+}
+
+/// 推满仍不够时报不报「被宽护卡住」（[`WideProtectionAtTheCeiling`]）：`stop` 是 F 已在上限、C 那一圈还有记着落点的槽、它们这一刻至少隔离着一个槽，
+/// 才报；别的都照旧。只读分配器（内存），不读盘：判的那一刻准入已经拒了，这里只定交回哪一种。
+fn wide_protection_at_the_ceiling(
+    allocator: &PoolAllocator,
+    current: &TransactionOutput,
+    stop: &FloorRaiseStop,
+) -> WideProtectionAtTheCeiling {
+    match stop {
+        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}
+        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }
+        | FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {
+            return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
+        }
+    }
+    let Some(target) = root_ring_push_target(allocator, current.root.checkpoint_txg) else {
+        return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
+    };
+    let isolated_slots_on_each_device = allocator.isolated_slots_covered_by(&target.placements);
+    if isolated_slots_on_each_device
+        .iter()
+        .all(|(_, isolated)| *isolated == 0)
+    {
+        return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
+    }
+    let checkpoint_txg_of_r = allocator.root_ring_occupancy().and_then(|occupancy| {
+        occupancy.checkpoint_txg_of_the_valid_root_recorded_on(target.ring_slot_of_r)
+    });
+    WideProtectionAtTheCeiling::StuckBehindTheWideProtection(Box::new(
+        StuckBehindTheWideProtection {
+            ring_slot_of_r: target.ring_slot_of_r,
+            checkpoint_txg_of_r,
+            empty_publishes_until_the_ring_slot_of_r_is_written: target
+                .publishes_until_the_ring_slot_of_r_is_written,
+            isolated_slots_on_each_device,
+        },
+    ))
+}
+
+/// 推环入口（[`MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`]）做成了：
+/// 发成的每一版按先后（最后一版是会话的 `current`），与写掉的那几槽（C 那一圈在这几槽记着的落点都已解开）。
+#[derive(Debug)]
+pub struct RootRingPushed {
+    pub versions_published: Vec<TransactionOutput>,
+    pub ring_slots_written: Vec<RootRingSlot>,
+}
+
+/// 推环入口没做成。成员按调用方要做的决定分：换盘表或换回这个池的盘、这一形第一版不支持、不必推、空间不够推不过去、发布路径的错。
+#[derive(Debug)]
+pub enum RootRingPushRefused {
+    /// 交进来的盘表没过挂着之后收盘表的那一套核（与抬 F 那一串的空发布同一套：身份交了不止一次、参数或盘表与盘上择到的系统配置不一致、
+    /// 有盘不「可见」、有盘落后于现行那一版又缺它的单元或它的根；系统配置择不出来），原样带着那一种 `MountError`。
+    /// 一个写都没发，盘上逐字节不变、会话不动。
+    DevicesHandedIn(Box<MountError>),
+    /// 现行那一版树表 0 条：推环的空发布在这一版上是零单元发布（D16（发布语义） 已定项 9），会话层的推环在这一形上条款没写，第一版不支持。
+    /// 会话在这一版上也不会被宽护卡住（用户改动在这一版上一律 `NoFileVersionToChange`）。在任何读写之前返回，盘上逐字节不变、会话不动。
+    VersionWithoutFileNotSupportedInTheFirstVersion,
+    /// C 那一圈这一刻没有记着落点的槽：这个进程写掉哪一槽都放不出东西，不推。一个写都没发，盘上逐字节不变、会话不动。
+    NothingHeldUntilThisProcessWritesARingSlot,
+    /// 整串空发布在任何写之前的预演里（分配器的一份拷贝上走发布路径落盘之前那一段）第 `refused_publish_in_the_push` 次（从 1 数）被拒：
+    /// 推到半路空发布取不到落点（`PublishError::PlacementRefused`）就是这里。一个写都没发，盘上逐字节不变、分配器与会话不动。
+    RefusedByTheRehearsalBeforeAnyWrite {
+        refused_publish_in_the_push: usize,
+        publishes_in_the_push: usize,
+        cause: PublishError,
+    },
+    /// 预演过了、真发时有一次发不出去（块设备错这一类；预演与真发只在读盘核换下的那一份对不上时分叉）：[`RootRingPushFailedAfterTheRehearsal`]。
+    PublishFailedAfterTheRehearsal(Box<RootRingPushFailedAfterTheRehearsal>),
+}
+
+/// 推环真发时第 `failed_publish_in_the_push` 次（从 1 数）发不出去：那一次没成立（分配器换回它之前，发布路径的做法），
+/// 前面那几次已落盘、在 `versions_published` 里，会话的 `current` 停在最后一次落盘的那一版上。
+#[derive(Debug)]
+pub struct RootRingPushFailedAfterTheRehearsal {
+    pub failed_publish_in_the_push: usize,
+    pub publishes_in_the_push: usize,
+    pub cause: PublishError,
+    pub versions_published: Vec<TransactionOutput>,
 }
 
 /// 为空间不够推抬 F 时抬 F 自己报了错：`cause` 是抬 F 的错原样（`mount::push_one_floor_raise_within_the_admission_budget` 的 `Err`），
@@ -473,6 +670,128 @@ impl MountedSession {
             publishes_pushed += publishes_of_this_raise;
         }
     }
+
+    /// 推环到 R 那一槽（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」那一个管理入口）：在现行那一版后面接连发 F 不变的空发布
+    /// （与可写挂载暖机同一张计划，`mount::empty_publish_plan_after_file_version`：重写固定点单元，不碰用户树、实例表照抄），
+    /// 直到这个进程写掉 C 那一圈还没解开、记着落点的每一槽（R 那一槽最后轮到），宽护随之解开（`allocator::PoolAllocator::record_root_written_by_this_process`
+    /// 摘下那一槽记着的落点、清掉别处不再护着的隔离位）。要几次见 [`StuckBehindTheWideProtection::empty_publishes_until_the_ring_slot_of_r_is_written`]，至多环长 3S。
+    ///
+    /// **调用即确认丢掉最近可退到的状态**：R 那一槽是环里最后轮到的一槽，推到它要把根环里更早的回退候选整环写掉，换上的是同一个用户状态的
+    /// 空发布根；推完之后回退候选集可以掉到少于 4 个不同状态，D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」只在调这个入口时破
+    /// （第 10z 批调查报告 10z-p 第六节第 1 条：384 槽盘那一形推完只剩 2 个）。不自动调：准入被宽护卡住时只报 [`StuckBehindTheWideProtection`]、写明这一条出路。
+    ///
+    /// 整串先在分配器的一份拷贝上预演（发布路径落盘之前那一段，与抬 F 那一串的预演同一个做法）：推到半路空发布取不到落点（没挡的槽不够这一串）
+    /// 时一个写都不发，盘上逐字节不变、分配器与会话不动。预演过了才真发；真发时有一次发不出去，前面那几次已落盘、`current` 停在最后一次落盘的那一版上。
+    /// 这些空发布不过准入两判（没有普通分配，D28（挂载期承诺量） 已定项 1），只受「取得到落点」约束。交进来的盘表照抬 F 那一串的
+    /// 空发布同一套核（`mount::parameters_of_the_pool_agreeing_with_the_caller_inputs`），在任何写之前；写入口只按盘上那一份参数建。
+    ///
+    /// # Errors
+    /// [`RootRingPushRefused`] 的五种。
+    pub fn push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states<
+        Device: BlockDevice,
+    >(
+        &mut self,
+        devices: &mut Vec<(DeviceIdentity, Device)>,
+    ) -> Result<RootRingPushed, RootRingPushRefused> {
+        let MountedSession {
+            allocator,
+            current,
+            instance,
+            shadow_ledger: _,
+            parameters_and_device_table,
+        } = self;
+        let start = match &*current {
+            PoolVersion::WithFile(version_of_trees)
+            | PoolVersion::WithTreesAndNoFile(version_of_trees) => version_of_trees.clone(),
+            PoolVersion::WithoutFile(_) => {
+                return Err(RootRingPushRefused::VersionWithoutFileNotSupportedInTheFirstVersion);
+            }
+        };
+        let parameters_of_the_pool = parameters_of_the_pool_agreeing_with_the_caller_inputs(
+            parameters_and_device_table.parameters_on_disk(),
+            devices,
+            &start,
+        )
+        .map_err(|refusal| RootRingPushRefused::DevicesHandedIn(Box::new(refusal)))?;
+        let parameters = &parameters_of_the_pool;
+        let Some(target) = root_ring_push_target(allocator, start.root.checkpoint_txg) else {
+            return Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot);
+        };
+        let publishes_in_the_push =
+            usize::try_from(target.publishes_until_the_ring_slot_of_r_is_written)
+                .expect("至多环长 3S ≤ 48 次");
+        // 预演：迭代上界是这一串的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）与分配器的拷贝。
+        {
+            let rehearsal_writer = PoolWriter::new(parameters, devices.as_mut_slice());
+            let mut copy = allocator.clone();
+            let mut rehearsed = start.clone();
+            for publish_index in 0..publishes_in_the_push {
+                let (_, next) = prepare_the_version_publish(
+                    &rehearsal_writer,
+                    &mut copy,
+                    &empty_publish_plan_after_file_version(&rehearsed, *instance),
+                    Some(&rehearsed),
+                    rehearsed.tree_identifiers,
+                    ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
+                    UnmountMarker::NotWrittenByTheUnmountSequence,
+                    &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
+                )
+                .map_err(|cause| {
+                    RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite {
+                        refused_publish_in_the_push: publish_index + 1,
+                        publishes_in_the_push,
+                        cause,
+                    }
+                })?;
+                rehearsed = next;
+            }
+        }
+        // 真发：迭代上界同上；跨轮携带的是落盘了的现行那一版与发成的每一版；出口只有发不出去那一次。
+        let mut last_published = start;
+        let mut versions_published = Vec::with_capacity(publishes_in_the_push);
+        for publish_index in 0..publishes_in_the_push {
+            let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
+            match publish_version(
+                &mut writer,
+                allocator,
+                empty_publish_plan_after_file_version(&last_published, *instance),
+                Some(&last_published),
+            ) {
+                Ok(next) => {
+                    versions_published.push(next.clone());
+                    last_published = next;
+                }
+                Err(cause) => {
+                    *current = PoolVersion::of_a_version_of_trees(last_published);
+                    return Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(
+                        Box::new(RootRingPushFailedAfterTheRehearsal {
+                            failed_publish_in_the_push: publish_index + 1,
+                            publishes_in_the_push,
+                            cause,
+                            versions_published,
+                        }),
+                    ));
+                }
+            }
+        }
+        *current = PoolVersion::of_a_version_of_trees(last_published);
+        let still_holding = allocator
+            .root_ring_occupancy()
+            .expect("推环之前读到了 C 那一圈，装着根环表")
+            .ring_slots_holding_placements_named_by_unapplied_journal_records();
+        assert!(
+            target
+                .ring_slots
+                .iter()
+                .all(|ring_slot| !still_holding.contains_key(ring_slot)),
+            "推环写到了 R 那一槽：每次发布 txg 加一、写 target_for_publish(txg) 那一槽并在取完落点之后摘下 C 在那一槽记着的落点，\
+             R 那一槽是这几槽里最后轮到的；还记着说明有一次发布没经分配器记根"
+        );
+        Ok(RootRingPushed {
+            versions_published,
+            ring_slots_written: target.ring_slots,
+        })
+    }
 }
 
 /// 现行那一版没有可改的文件：拒成 [`UserChangeRefused::NoFileVersionToChange`]，带上这次改动里已经落盘的（走空的那几步与推的抬 F）。
@@ -553,6 +872,10 @@ fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
                 last_refusal: refusal,
                 floor_raises,
                 refusals_that_pushed_the_floor_raises,
+                // 先借 `stop` 判这一形，再把它挪进下一个字段（字段按写的次序求值）。
+                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(
+                    allocator, current, &stop,
+                ),
                 stop,
                 versions_published,
             })),
diff --git a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
index db23a5a..496afb8 100644
--- a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -29,6 +29,10 @@
 //! - 认不出时宽护（tri，用户 2026-10-08 弹窗定，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第五节；细节第 10z 批第十二块实现员定，
 //!   被攻过零轮）：回退复活那一形再让 R 写的谱系片、R 写的全部单元、R_old 的根槽、R_old 的账之一读回全零，C 里 R 那一槽退到现行账里全部已释放的落点，
 //!   写到推过抬 F 不红 I-7.4；认得出回退时只多 R_old 账里仍分配、现行账里已释放的那一截；R 不是回退时一槽不多、挂载那一刻隔离数与之前相同。
+//! - 被宽护卡住的报与推环入口（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」；第 10z 批第十六块实现员定形，被攻过零轮）：384 槽盘上
+//!   第 10z 批第十四块报告第六节那几形卡住时，`NoSpaceAfterRaisingTheFloor` 点名「被宽护卡住」、带 R 的 txg、挡着的槽数与两条出路；
+//!   调推环入口之后 R 那一槽写掉、隔离回 0、之后写放得下；卸载重挂那一条出路也走得通；推到半路取不到落点时在任何写之前拒。
+//! - 被抛弃根那一圈遇到 defer 槽不双扣（第十四块报告第七节第 2 条）：挂载那一刻与根槽读坏时的抬 F 都只隔离分配位没置着的槽，回收时照根环表补。
 // 池（384 槽那一档小盘）与 R 那块盘都经 `tests/common_unwitnessed_publish/mod.rs` 起（它转出 `tests/common_admission_pool/mod.rs` 的全部）：
 // 再声明 `common_admission` 就把 `common_admission_pool` 当模块装两遍（clippy `duplicate_mod`）。
 mod common_unwitnessed_publish;
@@ -56,17 +60,21 @@ use singlefs_core::block_device::{
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
 use singlefs_core::mount::{
     mount_writable, mount_writable_with_space_admission, mount_writable_with_test_only_switches,
-    raise_rollback_floor_to_the_admission_ceiling, ring_slots_known_to_hold_a_root_by,
+    raise_rollback_floor, raise_rollback_floor_to_the_admission_ceiling,
+    ring_slots_known_to_hold_a_root_by, ring_slots_recorded_as_holding_an_abandoned_root_by,
     ring_slots_whose_reread_still_bad_refuses_the_ceiling_by, roll_back_by_a_forward_publish,
-    rollback_floor_ceiling, unmount, MountError, RaiseToTheAdmissionCeiling, RollbackTarget,
-    ShadowLedger, StillUnreadableAfterOneReread, Unmounted,
+    rollback_floor_ceiling, unmount, FloorRaiseStop, MountError, RaiseToTheAdmissionCeiling,
+    RollbackTarget, ShadowLedger, StillUnreadableAfterOneReread, Unmounted,
+};
+use singlefs_core::mounted_session::{
+    MountedSession, NoSpaceAfterRaisingTheFloor, RootRingPushRefused, StuckBehindTheWideProtection,
+    UserChangeRefused, WideProtectionAtTheCeiling,
 };
-use singlefs_core::mounted_session::MountedSession;
 use singlefs_core::pointer::{LocationEntry, NodePointer};
 use singlefs_core::recovery::{choose_system_configuration, instance_table_chain_of_root};
 use singlefs_core::root_record::RootRecord;
 use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
-use singlefs_core::transaction::PoolVersion;
+use singlefs_core::transaction::{PoolVersion, PublishError};
 use singlefs_harness::memory_pool::{
     writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
     SparseDevice,
@@ -3246,3 +3254,841 @@ fn revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims
         );
     }
 }
+
+/// 第 10z 批第十四块实现员报告第六节 384 槽盘卡死的几形（调查报告 `~/.local/share/singlefs/work/m3-r1/investigate-10z-p/report.md` 第五节 22 形里取的）：
+/// R 是哪一种、录在几次覆盖写之后，挂载那一刻读回全零的是哪几段（挂上就撤）。
+#[derive(Clone, Copy, Debug)]
+enum FormStuckBehindTheWideProtectionOnThe384SlotDevices {
+    /// 覆盖写 `overwrites_before` 次、回退到倒数第 7 版录 R；R 的账与 R 写的谱系片读回全零（探针 `b6`、`racct,rlin`）。
+    RollbackSixVersionsBack { overwrites_before: u64 },
+    /// 覆盖写 `overwrites_before` 次、建一个 inode 录 R；R 写的全部单元读回全零（探针 `n`、`rall`）。
+    InodeCreation { overwrites_before: u64 },
+    /// 覆盖写 `overwrites_before` 次、建一个 inode、删掉它，录删除的第一次发布得 R；R 的账与 R 写的墓碑片读回全零（探针 `+n` `d`、`racct,rtomb`）。
+    FirstPublishOfADeletion { overwrites_before: u64 },
+}
+
+/// 录下 `form` 的 R，交回录下来的 R 与挂载那一刻读回全零的那几段。
+fn recorded_r_and_ranges_reading_back_zero_of(
+    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
+) -> (RecordedPublish, RangesReadingBackAllZero) {
+    match form {
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
+            overwrites_before,
+        } => {
+            let recorded = record_one_rollback_after_overwrites(
+                overwrites_before,
+                VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
+            )
+            .recorded;
+            let lineage_unit_writes = lineage_unit_writes_of(&recorded);
+            assert!(
+                !lineage_unit_writes.is_empty(),
+                "回退那次发布写了谱系片（阳性对照：故障有处可打）"
+            );
+            let ranges = RangesReadingBackAllZero::default()
+                .with_the_writes(&account_writes_of(&recorded))
+                .with_the_writes(&lineage_unit_writes);
+            (recorded, ranges)
+        }
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
+            overwrites_before,
+        } => {
+            let recorded = record_one_inode_creation_after_overwrites(overwrites_before);
+            let ranges =
+                RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
+            (recorded, ranges)
+        }
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
+            overwrites_before,
+        } => {
+            let recorded =
+                record_the_first_publish_of_one_deletion_after_overwrites(overwrites_before);
+            let tombstone_unit_writes =
+                packed_unit_writes_of_type(&recorded, singlefs_core::unit::PACKED_TYPE_TOMBSTONE);
+            assert!(
+                !tombstone_unit_writes.is_empty(),
+                "删除的第一次发布写了墓碑片（阳性对照：故障有处可打）"
+            );
+            let ranges = RangesReadingBackAllZero::default()
+                .with_the_writes(&account_writes_of(&recorded))
+                .with_the_writes(&tombstone_unit_writes);
+            (recorded, ranges)
+        }
+    }
+}
+
+/// 卡住的那一刻：挂着的池（读回全零已撤）、录下来的 R、准入第一次被拒时报的「被宽护卡住」。
+struct PoolStuckBehindTheWideProtection {
+    recorded: RecordedPublish,
+    pool: PoolUnderTest<SparseBlockDevice>,
+    stuck: StuckBehindTheWideProtection,
+}
+
+/// 录 `form` 的 R、崩在 R 的根槽落盘之后轮换之前，可写挂载那一刻那几段读回全零（挂上就撤），择根退到 P；挂上之后覆盖写一次：
+/// 准入被拒、推抬 F 停在上限（`FloorAlreadyAtTheCeiling`），报「被宽护卡住」。挂上之后与被拒之后池级 checker 都不红 I-7.4、I-4.8。
+fn pool_stuck_behind_the_wide_protection(
+    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
+) -> PoolStuckBehindTheWideProtection {
+    let (recorded, ranges) = recorded_r_and_ranges_reading_back_zero_of(form);
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let lift = ReadBackAllZeroLift::default();
+    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
+    let mounted = mount_writable_with_space_admission(
+        &recorded.pool.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .unwrap_or_else(|refusal| panic!("{form:?}：R 走读不成，可写挂载落到 P、做得成：{refusal:?}"));
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "{form:?}：择根退到 P（阳性对照：故障打到了）"
+    );
+    lift.lift();
+    let image_after_the_mount =
+        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
+    let mut pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
+    let violations_after_the_mount = violations_of_unit_reuse_on(&pool.image());
+    assert!(
+        violations_after_the_mount.is_empty(),
+        "{form:?}：挂上之后 I-7.4、I-4.8 不红：{violations_after_the_mount:?}"
+    );
+    let Err(refusal) = pool.overwrite(OVERWRITE_BYTES) else {
+        panic!("{form:?}：挂上之后第一次覆盖写被宽护卡住（第 10z 批第十四块报告第六节：去掉双扣之后照旧全被拒），而它做成了");
+    };
+    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
+        panic!("{form:?}：被拒是推满仍不够：{refusal:?}");
+    };
+    assert!(
+        matches!(
+            no_space.stop,
+            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
+        ),
+        "{form:?}：推抬 F 停在上限（不再推的原因是 F 已在上限）：{:?}",
+        no_space.stop
+    );
+    let NoSpaceAfterRaisingTheFloor {
+        wide_protection_at_the_ceiling,
+        ..
+    } = *no_space;
+    let WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) =
+        wide_protection_at_the_ceiling
+    else {
+        panic!("{form:?}：F 在上限、C 那一圈还隔离着槽：报「被宽护卡住」，而报的是 {wide_protection_at_the_ceiling:?}");
+    };
+    let violations_after_the_refusal = violations_of_unit_reuse_on(&pool.image());
+    assert!(
+        violations_after_the_refusal.is_empty(),
+        "{form:?}：被拒之后 I-7.4、I-4.8 不红：{violations_after_the_refusal:?}"
+    );
+    PoolStuckBehindTheWideProtection {
+        recorded,
+        pool,
+        stuck: *stuck,
+    }
+}
+
+/// 推环之后接着写几次覆盖写、几次顺序写（每次两个数据单元）。
+const OVERWRITES_AFTER_PUSHING_THE_ROOT_RING: usize = 5;
+const SEQUENTIAL_WRITES_AFTER_PUSHING_THE_ROOT_RING: usize = 2;
+
+/// 被宽护卡住之后的报与推环（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」）：报的点名 R 的 txg、宽护挡着的槽数、两条出路；
+/// 调推环入口之后 R 那一槽写掉、C 那一圈一槽不剩、隔离回到不含宽护的数（这几形根环表里没有被抛弃根，就是 0），之后普通写放得下；
+/// 推之前、推之后与之后每一次写，池级 checker 不红 I-7.4、I-4.8。
+fn wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
+    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
+) {
+    let PoolStuckBehindTheWideProtection {
+        recorded,
+        mut pool,
+        stuck,
+    } = pool_stuck_behind_the_wide_protection(form);
+    assert_eq!(
+        stuck.checkpoint_txg_of_r,
+        Some(recorded.key.1),
+        "{form:?}：报的 R 是录下来的那一条（R 那一槽挂载那一刻记成有效根）"
+    );
+    assert_eq!(
+        stuck.ring_slot_of_r,
+        target_for_publish(
+            recorded.key.1,
+            recorded.pool.parameters.geometry.root_ring_slots_per_region
+        ),
+        "{form:?}：推环要写到的是 R 那一槽"
+    );
+    let isolated_by_the_wide_protection: u64 = stuck
+        .isolated_slots_on_each_device
+        .iter()
+        .map(|(_, isolated)| *isolated)
+        .sum();
+    assert!(
+        isolated_by_the_wide_protection > 0
+            && isolated_by_the_wide_protection <= isolated_slots_of(&pool.session().allocator),
+        "{form:?}：宽护挡着的槽数大于 0、不多于两块盘隔离的槽数：{isolated_by_the_wide_protection}"
+    );
+    let ways_out = stuck.ways_out();
+    assert!(
+        ways_out[0].contains("卸载") && ways_out[0].contains("再可写挂载"),
+        "{form:?}：出路一是卸载重挂：{}",
+        ways_out[0]
+    );
+    assert!(
+        ways_out[1].contains(
+            "push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states"
+        ) && ways_out[1].contains("少于 4 个不同状态"),
+        "{form:?}：出路二是推环入口、写明回退候选集会掉到少于 4 个不同状态：{}",
+        ways_out[1]
+    );
+    assert!(
+        ring_slots_recorded_as_holding_an_abandoned_root_by(&pool.session().allocator).is_empty(),
+        "{form:?}：这几形根环表里没有被抛弃根（隔离全是 C 那一圈的）"
+    );
+    let mut session = pool.session.take().expect("挂着的会话");
+    let pushed = session
+        .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(
+            &mut pool.devices,
+        );
+    pool.session = Some(session);
+    let pushed = pushed.unwrap_or_else(|refusal| panic!("{form:?}：推环做得成：{refusal:?}"));
+    for version in &pushed.versions_published {
+        pool.note_root_of_the_current_content(&version.root);
+    }
+    assert_eq!(
+        u64::try_from(pushed.versions_published.len()).expect("次数"),
+        stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
+        "{form:?}：推环发的空发布次数就是报里写的那个数"
+    );
+    let image_after_the_push = pool.image();
+    assert!(
+        !root_is_in_the_ring(&image_after_the_push, recorded.key),
+        "{form:?}：推环之后 R 那一槽被这个进程写掉、R 离开根环"
+    );
+    let allocator = &pool.session().allocator;
+    assert!(
+        allocator
+            .root_ring_occupancy()
+            .expect("挂载建的分配器装着根环表")
+            .ring_slots_holding_placements_named_by_unapplied_journal_records()
+            .is_empty(),
+        "{form:?}：推环之后 C 那一圈一槽不剩（宽护解开）"
+    );
+    assert_eq!(
+        isolated_slots_of(allocator),
+        0,
+        "{form:?}：推环之后隔离回到不含宽护的数（根环表里没有被抛弃根）"
+    );
+    assert_eq!(
+        slots_counted_both_as_allocated_and_as_isolated(allocator),
+        0,
+        "{form:?}：推环之后没有一个槽同时算进「已分配」与第九项"
+    );
+    let violations_after_the_push = violations_of_unit_reuse_on(&image_after_the_push);
+    assert!(
+        violations_after_the_push.is_empty(),
+        "{form:?}：推环之后 I-7.4、I-4.8 不红：{violations_after_the_push:?}"
+    );
+    for overwrite_number in 1..=OVERWRITES_AFTER_PUSHING_THE_ROOT_RING {
+        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!("{form:?}：推环之后第 {overwrite_number} 次覆盖写放得下：{refusal:?}")
+        });
+        let violations = violations_of_unit_reuse_on(&pool.image());
+        assert!(
+            violations.is_empty(),
+            "{form:?}：推环之后第 {overwrite_number} 次覆盖写之后 I-7.4、I-4.8 不红：{violations:?}"
+        );
+    }
+    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_PUSHING_THE_ROOT_RING {
+        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
+            .unwrap_or_else(|refusal| {
+                panic!("{form:?}：推环之后第 {sequential_write_number} 次顺序写放得下：{refusal:?}")
+            });
+        let violations = violations_of_unit_reuse_on(&pool.image());
+        assert!(
+            violations.is_empty(),
+            "{form:?}：推环之后第 {sequential_write_number} 次顺序写之后 I-7.4、I-4.8 不红：{violations:?}"
+        );
+    }
+}
+
+/// 前缀覆盖写 20 次、回退到倒数第 7 版（第十四块报告第六节 `o×20 | b6` 那一行）。
+#[test]
+fn a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
+) {
+    wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
+            overwrites_before: 20,
+        },
+    );
+}
+
+/// 前缀覆盖写 40 次、建一个 inode（第十四块报告第六节 `o×40 | n | rall` 那一行）。
+#[test]
+fn an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
+) {
+    wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
+            overwrites_before: 40,
+        },
+    );
+}
+
+/// 前缀覆盖写 160 次、建一个 inode 再删掉它（第十四块报告第六节 `o×160+n | d` 那一行）。
+#[test]
+fn a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
+) {
+    wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
+            overwrites_before: 160,
+        },
+    );
+}
+
+/// 卸载重挂之后接着覆盖写几次。
+const OVERWRITES_AFTER_UNMOUNTING_AND_MOUNTING_AGAIN: usize = 5;
+
+/// 出路一（卸载重挂）走得通：主形（覆盖写 40 次、回退到倒数第 7 版，调查报告第一节复现那一形）被宽护卡住之后正常卸载做成，
+/// 再可写挂载一次（读回全零早已撤：R 的账读得出）：R 按新实例的实例表判成被抛弃、根环表记着它自己账里的落点，宽护不再出现（C 那一圈一槽不剩），
+/// 之后覆盖写一次都不拒，池级 checker 不红 I-7.4、I-4.8。
+#[test]
+fn unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through(
+) {
+    let PoolStuckBehindTheWideProtection {
+        recorded, mut pool, ..
+    } = pool_stuck_behind_the_wide_protection(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
+            overwrites_before: OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
+        },
+    );
+    let mut session = pool.session.take().expect("挂着的会话");
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut session.allocator,
+        &mut session.current,
+        ShadowLedger::On,
+    );
+    match unmounted {
+        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
+            for version in &raised.publishes {
+                pool.note_root_of_the_current_content(&version.root);
+            }
+        }
+        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
+            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
+        }
+        Err(refusal) => panic!("被宽护卡住时正常卸载做得成：{refusal:?}"),
+    }
+    pool.crash_and_mount_writable()
+        .expect("卸载之后再可写挂载做得成（R 的账读得出）");
+    let occupancy = pool
+        .session()
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表");
+    let recorded_for_r = occupancy
+        .placements_recorded_for_the_abandoned_root(recorded.key.0, recorded.key.1)
+        .expect("再挂时根环表把 R 记成被抛弃根（按新实例的实例表）");
+    assert!(
+        !recorded_for_r.is_empty(),
+        "再挂时 R 的账读得出：根环表记着它自己账里的落点"
+    );
+    assert!(
+        occupancy
+            .ring_slots_holding_placements_named_by_unapplied_journal_records()
+            .is_empty(),
+        "再挂时宽护不再出现：C 那一圈一槽不剩"
+    );
+    for overwrite_number in 1..=OVERWRITES_AFTER_UNMOUNTING_AND_MOUNTING_AGAIN {
+        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!("卸载重挂之后第 {overwrite_number} 次覆盖写放得下：{refusal:?}")
+        });
+        let violations = violations_of_unit_reuse_on(&pool.image());
+        assert!(
+            violations.is_empty(),
+            "卸载重挂之后第 {overwrite_number} 次覆盖写之后 I-7.4、I-4.8 不红：{violations:?}"
+        );
+    }
+}
+
+/// 推到半路取不到落点那一形人为压到每块盘没挡的槽几个（调查报告第七节第 1 条 `PROBE_FILL_ISO_LEAVE=40`）。
+const UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH: u64 = 40;
+
+/// 推到半路空发布取不到落点（调查报告第七节第 1 条造的那一次）：主形被宽护卡住之后，把每块盘没挡的槽隔离到只剩 40 个（人为的），
+/// 推环要发的那一串在第 7 次左右取不到落点。推环入口在任何写之前拒（预演里被拒，`PlacementRefused`）：两块盘逐字节不变、
+/// 分配器（`{:?}` 整份）与会话的现行那一版都不变——不留半截不一致的状态。
+#[test]
+fn pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write() {
+    let PoolStuckBehindTheWideProtection {
+        mut pool, stuck, ..
+    } = pool_stuck_behind_the_wide_protection(
+        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
+            overwrites_before: OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
+        },
+    );
+    {
+        let session = pool.session.as_mut().expect("挂着的会话");
+        for device_map in &mut session.allocator.devices {
+            let unit_area_start = device_map.unit_area_start().slot().0;
+            // 迭代上界是单元区槽数；跨轮只带没挡的槽数；出口：只剩下要留的那几个。
+            for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
+                if device_map.unblocked_slots()
+                    <= UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH
+                {
+                    break;
+                }
+                if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
+                    device_map.isolate(SlotNumber(slot), 1);
+                }
+            }
+        }
+    }
+    let image_before = pool.image();
+    let allocator_before = format!("{:?}", pool.session().allocator);
+    let current_root_before = *pool.session().current.root();
+    let mut session = pool.session.take().expect("挂着的会话");
+    let pushed = session
+        .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(
+            &mut pool.devices,
+        );
+    pool.session = Some(session);
+    match pushed {
+        Err(RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite {
+            refused_publish_in_the_push,
+            publishes_in_the_push,
+            cause: PublishError::PlacementRefused { .. },
+        }) => {
+            assert!(
+                refused_publish_in_the_push > 1
+                    && refused_publish_in_the_push <= publishes_in_the_push,
+                "推到半路才取不到落点（前面几次放得下，阳性对照）：第 {refused_publish_in_the_push} 次，共 {publishes_in_the_push} 次"
+            );
+            assert_eq!(
+                u64::try_from(publishes_in_the_push).expect("次数"),
+                stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
+                "这一串的次数就是报里写的那个数"
+            );
+        }
+        other => panic!(
+            "推到半路取不到落点：在任何写之前拒成预演被拒、拒因是取不到落点，而交回 {other:?}"
+        ),
+    }
+    assert!(pool.image() == image_before, "推环被拒：两块盘逐字节不变");
+    assert_eq!(
+        format!("{:?}", pool.session().allocator),
+        allocator_before,
+        "推环被拒：分配器整份不变"
+    );
+    assert_eq!(
+        *pool.session().current.root(),
+        current_root_before,
+        "推环被拒：会话的现行那一版不变"
+    );
+}
+
+/// 推满仍不够、F 已在上限而没有 C 那一圈（崩在静止点之后再挂，水位之上没有没施加的记录）：照旧只报空间不够、不报「被宽护卡住」；
+/// 这时调推环入口也不推（C 那一圈没有记着落点的槽），两块盘逐字节不变。池照 `admission_raises_the_floor_before_refusing.rs` 里
+/// 「截断之后同样大小写回」那一条起（单元区 256 槽两块盘，崩了再挂、覆盖写 3 次、顺序写把文件长到会话报空间不够）。
+fn pool_stuck_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written() -> (
+    PoolUnderTest<SparseBlockDevice>,
+    NoSpaceAfterRaisingTheFloor,
+) {
+    let device_width = singlefs_harness::history::HistoryDeviceWidth::UnitAreaOf256Slots;
+    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
+        device_width.parameters(),
+        device_width.device_bytes(),
+        plain_device,
+    );
+    pool.crash_and_mount_writable()
+        .expect("第一个文件之后崩了再挂");
+    for overwrite_number in 1..=3 {
+        pool.overwrite(OVERWRITE_BYTES)
+            .unwrap_or_else(|refusal| panic!("第 {overwrite_number} 次覆盖写：{refusal:?}"));
+    }
+    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
+    let mut units = 0;
+    // 迭代上界 100 个单元；跨轮只带长到的单元数；出口：被拒。
+    let refusal = loop {
+        match pool.sequential_write((units + 1) * unit_bytes) {
+            Ok(_) => units += 1,
+            Err(refusal) => break refusal,
+        }
+        assert!(units < 100, "256 槽的盘长不到 100 个单元");
+    };
+    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
+        panic!("长大被拒是推满仍不够：{refusal:?}");
+    };
+    assert!(
+        matches!(
+            no_space.stop,
+            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
+        ),
+        "推抬 F 停在上限（阳性对照：与被宽护卡住同一种停法）：{:?}",
+        no_space.stop
+    );
+    assert!(
+        pool.session()
+            .allocator
+            .root_ring_occupancy()
+            .expect("挂载建的分配器装着根环表")
+            .ring_slots_holding_placements_named_by_unapplied_journal_records()
+            .is_empty(),
+        "崩在静止点之后再挂：C 那一圈一槽都没有"
+    );
+    (pool, *no_space)
+}
+
+#[test]
+fn no_space_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written_is_not_reported_as_the_wide_protection(
+) {
+    let (_pool, no_space) =
+        pool_stuck_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written();
+    assert!(
+        matches!(
+            no_space.wide_protection_at_the_ceiling,
+            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission
+        ),
+        "没有 C 那一圈：照旧只报空间不够，不报被宽护卡住：{:?}",
+        no_space.wide_protection_at_the_ceiling
+    );
+}
+
+#[test]
+fn pushing_the_root_ring_with_nothing_held_until_a_ring_slot_is_written_writes_nothing() {
+    let (mut pool, _no_space) =
+        pool_stuck_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written();
+    let image_before = pool.image();
+    let mut session = pool.session.take().expect("挂着的会话");
+    let pushed = session
+        .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(
+            &mut pool.devices,
+        );
+    pool.session = Some(session);
+    assert!(
+        matches!(
+            pushed,
+            Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot)
+        ),
+        "C 那一圈没有记着落点的槽：不推，而交回 {pushed:?}"
+    );
+    assert!(pool.image() == image_before, "不推：两块盘逐字节不变");
+}
+
+/// 现行那一版树表 0 条（mkfs 之后可写挂载、还没写文件）：推环入口在任何读写之前拒成第一版不支持的那一种，两块盘逐字节不变。
+#[test]
+fn pushing_the_root_ring_on_a_version_without_file_is_refused_as_not_supported_in_the_first_version(
+) {
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
+    singlefs_core::make_filesystem::make_filesystem(&parameters, &mut devices)
+        .expect("小盘上 mkfs 做得成");
+    let mounted = mount_writable_with_space_admission(
+        &parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("mkfs 之后可写挂载做得成");
+    let (_, mut session) = MountedSession::of_the_mount(mounted);
+    assert!(
+        matches!(session.current, PoolVersion::WithoutFile(_)),
+        "还没写文件：现行那一版树表 0 条（阳性对照）"
+    );
+    let image_of = |devices_of_the_pool: &[(DeviceIdentity, SparseBlockDevice)]| MemoryPool {
+        devices: devices_of_the_pool
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: device_bytes,
+    };
+    let image_before = image_of(&devices);
+    let pushed = session
+        .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(
+            &mut devices,
+        );
+    assert!(
+        matches!(
+            pushed,
+            Err(RootRingPushRefused::VersionWithoutFileNotSupportedInTheFirstVersion)
+        ),
+        "树表 0 条：拒成第一版不支持，而交回 {pushed:?}"
+    );
+    assert!(image_of(&devices) == image_before, "两块盘逐字节不变");
+}
+
+/// 实例 4 里覆盖写几次再把 F 抬到 C 的 txg：之后有 4 个以上不同的非空状态，准入上限高过 C 的 txg（抬 F 只供测试强制进入的那一档）。
+const OVERWRITES_IN_THE_INSTANCE_RAISING_THE_FLOOR_TO_THE_ABANDONED_ROOT: usize = 4;
+
+/// 挂上之后至多覆盖写几次去等 C 离开根环。
+const OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST: usize = 40;
+
+/// 被抛弃根那一圈的 defer 槽（第 10z 批第十四块实现员报告第七节第 2 条；派发第十六块第 2 条）的池：崩溃恢复抛弃了 C 的那份盘面
+/// （[`pool_whose_newest_root_was_abandoned_by_a_crash_recovery`]：实例 3 择根退到 C 之前那一条 B、写行换下 C 与 B 共用的实例 2 那张实例表），
+/// 实例 4 挂上、覆盖写 4 次、把 F 抬到 C 的 txg（B 不再是候选根），停在那里（进程退出）。交回实例 4 退出时的盘面。
+/// 这之后的可写挂载里，C 引用、现行账里已释放（实例 3 的写行换下的）、释放代高于 F 的槽不再被候选根豁免，还在 defer 窗口里（分配位置着）。
+fn image_after_raising_the_floor_to_the_txg_of_the_abandoned_root(
+    built: &PoolWithAnAbandonedRoot,
+) -> MemoryPool {
+    let mut pool = PoolUnderTest {
+        parameters: built.parameters.clone(),
+        device_bytes: built.device_bytes,
+        devices: plain_devices_on(&built.image),
+        session: None,
+        write_time_seconds: built.write_time_seconds + 1000,
+        content_of_the_current_version: Vec::new(),
+        content_of_each_root: BTreeMap::new(),
+    };
+    pool.crash_and_mount_writable()
+        .expect("实例 4 可写挂载做得成（C 的根槽与账读得出）");
+    for overwrite_number in 1..=OVERWRITES_IN_THE_INSTANCE_RAISING_THE_FLOOR_TO_THE_ABANDONED_ROOT {
+        pool.overwrite(OVERWRITE_BYTES)
+            .unwrap_or_else(|refusal| panic!("实例 4 第 {overwrite_number} 次覆盖写：{refusal:?}"));
+    }
+    let mut session = pool.session.take().expect("挂着的会话");
+    let raised = {
+        let PoolVersion::WithFile(current) = &mut session.current else {
+            panic!("覆盖写之后现行那一版带文件");
+        };
+        raise_rollback_floor(
+            &pool.parameters,
+            &mut pool.devices,
+            &mut session.allocator,
+            current,
+            built.abandoned_root.checkpoint_txg,
+            ShadowLedger::On,
+        )
+    };
+    raised.unwrap_or_else(|refusal| panic!("实例 4 把 F 抬到 C 的 txg 做得成：{refusal:?}"));
+    pool.image()
+}
+
+/// `placements` 罩着的槽里，现行账（`allocator` 的记录）里记成已释放、释放代高于 `floor` 的（盘、槽）：挂载那一刻还在 defer 窗口里、
+/// 不被候选根（txg ≥ F）豁免的那一截的候选。从记录算，不经被测的方法。
+fn released_above_the_floor_and_covered_by(
+    allocator: &PoolAllocator,
+    placements: &[PlacementOnDevice],
+    floor: CheckpointTxg,
+) -> Vec<(DeviceIdentity, SlotNumber)> {
+    allocator
+        .records()
+        .iter()
+        .filter(|record| record.is_released && record.generation > floor)
+        .flat_map(|record| {
+            (record.slot.0..record.slot.0 + u64::from(record.span_slots))
+                .map(move |slot| (record.device, SlotNumber(slot)))
+        })
+        .filter(|(device, slot)| {
+            placements.iter().any(|placement| {
+                placement.device == *device
+                    && slot.0 >= placement.slot.0
+                    && slot.0 < placement.slot.0 + placement.span
+            })
+        })
+        .collect()
+}
+
+/// 被抛弃根那一圈挂载那一刻不双扣、回收之后照旧隔离（派发第十六块第 2 条）：实例 4 把 F 抬到 C 的 txg 之后崩了再挂（实例 5，F = C 的 txg）。
+/// 挂载那一刻 C 记成被抛弃根，它引用的槽里有现行账已释放、释放代高于 F（还在 defer 窗口里、分配位置着）、不被候选根豁免的：
+/// 两块盘没有一个槽同时算进「已分配」与第九项。之后覆盖写到 C 离开根环：C 还在环里时它引用的槽一个都不空闲，其中那几个 defer 槽
+/// 至少有一个在 C 离环之前被回收、补上了隔离（回收时照根环表补，阳性对照：走到了回收那一处）；每一步池级 checker 不红 I-7.4、I-4.8。
+/// 被抛弃根那一圈照旧整段隔离时挂载那一刻那一句红。
+#[test]
+fn defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let image = image_after_raising_the_floor_to_the_txg_of_the_abandoned_root(&built);
+    let fault =
+        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
+    let mut devices = devices_with_the_read_fault_on(&image, &fault);
+    // 空间准入关着（只供测试的开关）：会话不为空间推抬 F，一次覆盖写只发一次、txg 只走一格，看得到「C 那一槽写掉之前那一次发布」之后的那一刻。
+    let mounted = mount_writable_with_space_admission(
+        &built.parameters,
+        &mut devices,
+        SpaceAdmission::SkippedByTheTestOnlySwitch,
+    )
+    .expect("实例 5 可写挂载做得成");
+    assert_eq!(
+        mounted.output.effective_root.rollback_floor, built.abandoned_root.checkpoint_txg,
+        "挂载那一刻 F 是 C 的 txg（阳性对照：B 不再是候选根）"
+    );
+    let recorded_for_c = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .placements_recorded_for_the_abandoned_root(
+            built.abandoned_root.instance,
+            built.abandoned_root.checkpoint_txg,
+        )
+        .filter(|placements| !placements.is_empty())
+        .expect("挂载那一刻根环表把 C 记成被抛弃根、带着它引用的落点");
+    let defer_slots_of_c = released_above_the_floor_and_covered_by(
+        &mounted.allocator,
+        &recorded_for_c,
+        built.abandoned_root.checkpoint_txg,
+    );
+    assert!(
+        !defer_slots_of_c.is_empty(),
+        "C 引用的槽里有现行账已释放、释放代高于 F 的（阳性对照：这一形走到了被抛弃根引用的 defer 槽）"
+    );
+    let free_defer_slots_of_c_at_the_mount: Vec<&(DeviceIdentity, SlotNumber)> = defer_slots_of_c
+        .iter()
+        .filter(|(device, slot)| {
+            mounted
+                .allocator
+                .devices
+                .iter()
+                .any(|device_map| device_map.device == *device && device_map.is_free(*slot))
+        })
+        .collect();
+    assert!(
+        free_defer_slots_of_c_at_the_mount.is_empty(),
+        "挂载那一刻这几个 defer 槽分配位置着、不空闲：空闲的 {free_defer_slots_of_c_at_the_mount:?}"
+    );
+    assert_eq!(
+        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
+        0,
+        "挂载那一刻没有一个槽同时算进「已分配」与第九项（D28（挂载期承诺量） 已定项 1 第九项：被抛弃根引用的 defer 槽算在「已分配」里）"
+    );
+    let mut pool = pool_under_test_after_the_mount(
+        &built.parameters,
+        built.device_bytes,
+        devices,
+        mounted,
+        built.write_time_seconds + 3000,
+    );
+    let key_of_c = (
+        built.abandoned_root.instance,
+        built.abandoned_root.checkpoint_txg,
+    );
+    let mut a_defer_slot_was_reclaimed_and_isolated_while_c_was_in_the_ring = false;
+    let mut overwrites = 0;
+    // 迭代上界 OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST；跨轮带计数与「回收补隔离走到了没有」；出口：C 离开根环，
+    // 被拒、双扣、C 引用的槽空闲、checker 红都当场判红（panic）。
+    while root_is_in_the_ring(&pool.image(), key_of_c) {
+        assert!(
+            overwrites < OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST,
+            "{OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST} 次覆盖写之内 C 那一槽写掉、C 离开根环"
+        );
+        pool.overwrite(OVERWRITE_BYTES)
+            .unwrap_or_else(|refusal| panic!("第 {} 次覆盖写被拒：{refusal:?}", overwrites + 1));
+        overwrites += 1;
+        let allocator = &pool.session().allocator;
+        assert_eq!(
+            slots_counted_both_as_allocated_and_as_isolated(allocator),
+            0,
+            "第 {overwrites} 次覆盖写之后没有一个槽同时算进「已分配」与第九项"
+        );
+        if root_is_in_the_ring(&pool.image(), key_of_c) {
+            let free_among_the_placements_of_c = free_slots_covered_by(allocator, &recorded_for_c);
+            assert!(
+                free_among_the_placements_of_c.is_empty(),
+                "第 {overwrites} 次覆盖写之后 C 还在根环里：它引用的槽一个都不空闲，空闲的 {free_among_the_placements_of_c:?}"
+            );
+            a_defer_slot_was_reclaimed_and_isolated_while_c_was_in_the_ring |=
+                defer_slots_of_c.iter().any(|(device, slot)| {
+                    allocator.devices.iter().any(|device_map| {
+                        device_map.device == *device && device_map.is_isolated(*slot)
+                    })
+                });
+        }
+        let violations = violations_of_unit_reuse_on(&pool.image());
+        assert!(
+            violations.is_empty(),
+            "第 {overwrites} 次覆盖写之后 C 引用的单元一个都没复用：{violations:?}"
+        );
+    }
+    assert!(
+        a_defer_slot_was_reclaimed_and_isolated_while_c_was_in_the_ring,
+        "C 离环之前有一个 defer 槽被回收、照根环表补上了隔离（阳性对照：走到了回收时补隔离那一处）"
+    );
+}
+
+/// 根槽这一遍读坏的被抛弃根那一圈同样不双扣（派发第十六块第 2 条）：同一份盘面挂上（实例 5，F = C 的 txg）之后 C 的根槽读报错，
+/// 把 F 抬到它现在的值（只供测试强制进入的那一档，F 不变）：抬 F 那一串读根环时 C 那一槽读坏、照根环表里记着的落点进影子账
+/// （D16（发布语义） 已定项 1「根槽这一次读坏」，H1），那几个 defer 槽释放代高于 F、这一串不回收：抬完之后没有一个槽同时算进「已分配」与第九项，
+/// C 引用的槽一个都不空闲。这一圈照旧整段隔离时抬完那一句红。
+#[test]
+fn defer_slots_an_abandoned_root_whose_ring_slot_reads_bad_references_are_not_counted_twice_by_a_floor_raise(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let image = image_after_raising_the_floor_to_the_txg_of_the_abandoned_root(&built);
+    let fault =
+        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
+    let (mut pool, recorded_for_c) = mounted_while_the_abandoned_root_reads(
+        &PoolWithAnAbandonedRoot {
+            parameters: built.parameters.clone(),
+            device_bytes: built.device_bytes,
+            image,
+            write_time_seconds: built.write_time_seconds + 3000,
+            abandoned_root: built.abandoned_root,
+            newest_valid_root: built.newest_valid_root,
+        },
+        &fault,
+    );
+    let floor = pool.rollback_floor();
+    assert_eq!(
+        floor, built.abandoned_root.checkpoint_txg,
+        "挂载那一刻 F 是 C 的 txg（阳性对照）"
+    );
+    let defer_slots_of_c =
+        released_above_the_floor_and_covered_by(&pool.session().allocator, &recorded_for_c, floor);
+    assert!(
+        !defer_slots_of_c.is_empty(),
+        "C 引用的槽里有现行账已释放、释放代高于 F 的（阳性对照）"
+    );
+    fault.is_active.set(true);
+    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
+    let mut session = pool.session.take().expect("挂着的会话");
+    let raised = {
+        let PoolVersion::WithFile(current) = &mut session.current else {
+            panic!("挂载落到的那一版带文件");
+        };
+        raise_rollback_floor(
+            &pool.parameters,
+            &mut pool.devices,
+            &mut session.allocator,
+            current,
+            floor,
+            ShadowLedger::On,
+        )
+    };
+    pool.session = Some(session);
+    let raised =
+        raised.unwrap_or_else(|refusal| panic!("C 的根槽读坏、F 不变的抬 F 做得成：{refusal:?}"));
+    assert_eq!(
+        raised.abandoned_roots_unreadable, 0,
+        "C 的根槽读坏：照根环表里记着的落点算，不读它的账"
+    );
+    let allocator = &pool.session().allocator;
+    assert_eq!(
+        slots_counted_both_as_allocated_and_as_isolated(allocator),
+        0,
+        "抬完之后没有一个槽同时算进「已分配」与第九项"
+    );
+    let still_allocated_defer_slots: Vec<&(DeviceIdentity, SlotNumber)> = defer_slots_of_c
+        .iter()
+        .filter(|(device, slot)| {
+            allocator.devices.iter().any(|device_map| {
+                device_map.device == *device
+                    && !device_map.is_free(*slot)
+                    && !device_map.is_isolated(*slot)
+            })
+        })
+        .collect();
+    assert!(
+        !still_allocated_defer_slots.is_empty(),
+        "抬完之后这几个 defer 槽还分配着、这一串没回收（阳性对照：这一圈遇到了分配位置着的槽）"
+    );
+    assert_none_of_the_placements_is_free(allocator, &recorded_for_c, "C 记着");
+}
````

### 第十六块 mutations-append.tsv

````
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（回退、前缀 20 次）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    allocator, current, &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- wide_protection	a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（建 inode、前缀 40 次）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    allocator, current, &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through	an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（删除、前缀 160 次）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    allocator, current, &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through	a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（卸载重挂那一条出路）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    allocator, current, &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through	unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through
第10z批第十六块 推满仍不够时不分宽护：F 在上限、C 那一圈隔离着也照旧只报空间不够（推到半路取不到落点那一形）	crates/singlefs-core/src/mounted_session.rs	                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(\n                    allocator, current, &stop,\n                ),	                wide_protection_at_the_ceiling: WideProtectionAtTheCeiling::NotWhatStopsTheAdmission,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write	pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write
第10z批第十六块 推环少推一次：R 那一槽写不掉、宽护不解开	crates/singlefs-core/src/mounted_session.rs	            usize::try_from(target.publishes_until_the_ring_slot_of_r_is_written)	            usize::try_from(target.publishes_until_the_ring_slot_of_r_is_written - 1)	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through	an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through
第10z批第十六块 推环不预演：推到半路取不到落点时前几次已落盘	crates/singlefs-core/src/mounted_session.rs	            let mut rehearsed = start.clone();\n            for publish_index in 0..publishes_in_the_push {	            let mut rehearsed = start.clone();\n            for publish_index in 0..0 {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write	pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write
第10z批第十六块 C 那一圈没有记着落点的槽时照推一整环	crates/singlefs-core/src/mounted_session.rs	        .max()?;	        .max()\n        .or(Some((ring_length_in_txgs, RootRingSlot { region: 0, slot: 0 })))?;	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- pushing_the_root_ring_with_nothing_held_until_a_ring_slot_is_written_writes_nothing	pushing_the_root_ring_with_nothing_held_until_a_ring_slot_is_written_writes_nothing
第10z批第十六块 树表 0 条那一版报成没有要推的（不报第一版不支持）	crates/singlefs-core/src/mounted_session.rs	                return Err(RootRingPushRefused::VersionWithoutFileNotSupportedInTheFirstVersion);	                return Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot);	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- pushing_the_root_ring_on_a_version_without_file_is_refused_as_not_supported_in_the_first_version	pushing_the_root_ring_on_a_version_without_file_is_refused_as_not_supported_in_the_first_version
第10z批第十六块 F 在上限就报被宽护卡住：不看 C 那一圈有没有、隔离着没有	crates/singlefs-core/src/mounted_session.rs	    let Some(target) = root_ring_push_target(allocator, current.root.checkpoint_txg) else {\n        return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;\n    };\n    let isolated_slots_on_each_device = allocator.isolated_slots_covered_by(&target.placements);\n    if isolated_slots_on_each_device\n        .iter()\n        .all(|(_, isolated)| *isolated == 0)\n    {\n        return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;\n    }	    let target = RootRingPushTarget {\n        ring_slots: Vec::new(),\n        placements: Vec::new(),\n        ring_slot_of_r: RootRingSlot { region: 0, slot: 0 },\n        publishes_until_the_ring_slot_of_r_is_written: 0,\n    };\n    let isolated_slots_on_each_device = allocator.isolated_slots_covered_by(&target.placements);	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- no_space_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written_is_not_reported_as_the_wide_protection	no_space_at_the_ceiling_without_isolation_held_until_a_ring_slot_is_written_is_not_reported_as_the_wide_protection
第10z批第十六块 被抛弃根那一圈挂载那一刻照旧整段隔离：分配位置着的 defer 槽与已分配双扣	crates/singlefs-core/src/mount.rs	            if reclaim_isolates_what_this_root_references {	            if false {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed	defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed
第10z批第十六块 根槽读坏的被抛弃根那一圈照旧整段隔离：抬 F 那一串双扣	crates/singlefs-core/src/mount.rs	        allocator.isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n            placement.device,\n            placement.slot,\n            placement.span,\n        );	        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- defer_slots_an_abandoned_root_whose_ring_slot_reads_bad_references_are_not_counted_twice_by_a_floor_raise	defer_slots_an_abandoned_root_whose_ring_slot_reads_bad_references_are_not_counted_twice_by_a_floor_raise
第10z批第十六块 挂载内回收出来、被抛弃根引用的 defer 槽不补隔离（挂载那一刻跳过的那几个）	crates/singlefs-core/src/allocator.rs	            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);\n        self.isolate_reclaimed_slots_the_isolation_still_protects(\n            &reclaimed,\n            &still_protected_by_the_isolation,\n        );	            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);\n        let _ = (&reclaimed, &still_protected_by_the_isolation);	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed	defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed
````

### 第十六块 mutations-replacements.tsv

````
步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	            if reclaim_isolates_what_this_root_references {\n                allocator\n                    .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n                        device, slot, span_slots,\n                    );\n            } else {\n                allocator.isolate_abandoned(device, slot, span_slots);\n            }	            let _ = (device, slot, span_slots, reclaim_isolates_what_this_root_references);	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored with_the_shadow_ledger_a_publish_after_a_recovery	with_the_shadow_ledger_a_publish_after_a_recovery_keeps_off_the_abandoned_data_slot_and_the_abandoned_root_reads_the_third_content
增补 2 收口第 27 行：影子账隔离只铺记录起点那一槽，不按记录的跨度铺位	crates/singlefs-core/src/mount.rs	            if reclaim_isolates_what_this_root_references {\n                allocator\n                    .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n                        device, slot, span_slots,\n                    );\n            } else {\n                allocator.isolate_abandoned(device, slot, span_slots);\n            }	            if reclaim_isolates_what_this_root_references {\n                allocator\n                    .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n                        device, slot, 1,\n                    );\n            } else {\n                allocator.isolate_abandoned(device, slot, 1);\n            }	-p singlefs-harness --test unreadable_abandoned_root_slot	an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_is_neither_isolated_nor_counted_when_its_root_slot_is_unreadable
第10z批第九块 K3 loop-span1 根槽这一遍读坏的被抛弃根记着的落点补隔离只隔起点那一槽（正推 loop-span1：跨两槽的落点后一槽空闲）	crates/singlefs-core/src/mount.rs	        allocator.isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n            placement.device,\n            placement.slot,\n            placement.span,\n        );\n    }\n    // C 那一份不豁免	        allocator.isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n            placement.device,\n            placement.slot,\n            1,\n        );\n    }\n    // C 那一份不豁免	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds
````

## 第十八块

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index edb1393..7ee421b 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -4842,6 +4842,70 @@ mod tests {
             .count()
     }
 
+    /// 没有行为用例的那几层，扫源码钉住它们怎么用收到的叫停（`every_layer_handed_the_stop_request_hands_it_on_or_asks_it`）。
+    enum StopRequestUseThatIsPinned {
+        /// 函数体短：参数行之后到函数收尾（返回类型与函数体，去掉注释行与空白）整个等于这一句。
+        WholeBodyAfterTheParameter(&'static str),
+        /// 函数体太长、整句比不了：用到叫停整名的只有 `line` 这一行（去掉首尾空白），它是函数体顶层的一句；
+        /// `binding` 在这一层只由它 `let` 绑一次，`type_name` 在这一层只出现在它里面。
+        TopLevelStatementOfALongBody {
+            line: &'static str,
+            binding: &'static str,
+            type_name: &'static str,
+        },
+    }
+
+    /// `lines` 每一行起头处在第几层花括号里（第一行起头算第 0 层）：字符串、字符字面量与行注释里的花括号不算，字符串跨行时照跨着数。
+    ///
+    /// # Panics
+    /// 右花括号多于左花括号；碰到原始字符串（`r"…"`、`r#"…"#`，这里不认它，层数会数错）。
+    fn brace_depth_at_the_start_of_each_line(lines: &[&String]) -> Vec<usize> {
+        let mut depth = 0_usize;
+        let mut is_inside_a_string = false;
+        let mut depths = Vec::with_capacity(lines.len());
+        for line in lines {
+            depths.push(depth);
+            let characters: Vec<char> = line.chars().collect();
+            let mut index = 0;
+            while index < characters.len() {
+                let character = characters[index];
+                if is_inside_a_string {
+                    if character == '\\' {
+                        index += 1;
+                    } else if character == '"' {
+                        is_inside_a_string = false;
+                    }
+                } else if character == '"' {
+                    let previous = index.checked_sub(1).map(|before| characters[before]);
+                    assert!(
+                        !(previous == Some('r') || previous == Some('#')),
+                        "这一层里有原始字符串，花括号的层数要先认它再数：{line}"
+                    );
+                    is_inside_a_string = true;
+                } else if character == '\'' {
+                    // 字符字面量（`'x'`、`'\''`、`'\u{7b}'`）整个跳过；生命周期（`'static`）只跳这个引号
+                    if characters.get(index + 1) == Some(&'\\') {
+                        index += 2;
+                        while characters.get(index + 1).is_some_and(|next| *next != '\'') {
+                            index += 1;
+                        }
+                        index += 1;
+                    } else if characters.get(index + 2) == Some(&'\'') {
+                        index += 2;
+                    }
+                } else if character == '/' && characters.get(index + 1) == Some(&'/') {
+                    break;
+                } else if character == '\u{7b}' {
+                    depth += 1;
+                } else if character == '\u{7d}' {
+                    depth = depth.checked_sub(1).expect("右花括号不多于左花括号");
+                }
+                index += 1;
+            }
+        }
+        depths
+    }
+
     /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4：扫源码的用例被领块之前那一问写成 `if false && stop_is_requested()` 骗过，
     /// 攻方 crash-3）：四块要判，叫停在第三次被问时交真：领每一块之前各问一次、问到真就不再领，叫停之前领的两块判完写了，之后的不领，
     /// 叫停一共被问三次。不要卡、不碰库：判一段与写一块都用假的。
@@ -4908,28 +4972,69 @@ mod tests {
     /// （叫停在第二次被问时交真，只判了一块）。另四层换不成不要卡的行为用例：第 ① 段在卡上判那三层要卡，`check_selected_blocks` 要录好的流与库
     /// （只有 `run_from_environment` 调它，经环境跑的那几条用例里叫停文件放上时块都判过了，问不到这一层），这里扫源码：参数行之后到函数收尾的
     /// 顶格右括号，用到叫停整名的行（去掉首尾空白）逐行等于钉住的那几行，往下交的那一行之后紧跟右括号（是调下一层的最后一个实参）。
-    /// 整行比拦得住上面三种写法（多一行、或那一行不是原样）。拦不住的：同一行原样交下去而下一层的函数体里不用（下一层由它自己那一条管，
-    /// 第 ① 段在卡上判那一层里 `stop_request` 往下怎么用由 `the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run`
-    /// 管，也只看文本）；把整次调用换成另一个同签名的函数；在函数上面加 `#[cfg]` 让另一份同名函数编进来；宏展开出来的调用。
+    /// 整行比拦得住上面三种写法（多一行、或那一行不是原样）。
+    /// 第 10z 批第十八块（第 10z 批第十四、十五块代码轮判决 K4，正推 N1：`check_selected_blocks` 里真调用的叫停换成 `&|| false`，钉住的那一行
+    /// 连同它后面的右括号原样留在 `if false { … }` 死分支里的另一次调用里，编得过、整行比照绿）：函数体短的三层
+    /// （`judge_my_blocks_on_gpu_cards`、`check_selected_blocks`、`judge_my_blocks_on_gpu`）改成整句比——参数行之后到函数收尾
+    /// （返回类型与函数体，去掉注释行与空白）整个等于钉住的那一句，死分支、多一次调用、换实参、换被调的函数都红；函数体太长、整句比不了的
+    /// `judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from` 照旧整行比那一行，另要它是函数体顶层的一句
+    /// （不在任何块里：`if false { … }`、`if false && … { … }` 包着的红）、`stop_request` 在这一层只由它 `let` 绑一次
+    /// （后面再 `let` 一次就换掉了记住的叫停）、`StopRequestRememberedInThisRun` 在这一层只出现在它里面（块里 `use` 或另起同名类型会遮住它）。
+    ///
+    /// 拦不住的写法，一种一条：
+    /// - 同一行原样交下去而下一层的函数体里不用（下一层由它自己那一条管；第 ① 段在卡上判那一层里 `stop_request` 往下怎么用由
+    ///   `the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run` 管，也只看文本）；
+    /// - 长的那一层：钉住那一行之前提早返回，或让它之后的路走不到；`stop_request` 经 `let` 之外的绑定（`match` 的臂、闭包参数、`for`、
+    ///   `if let`）在块里换成别的；
+    /// - 被调的那一层自己改了（短的三层整句比钉的是调谁、怎么交，不是被调的函数里做什么）；harness 档的 `StopRequestRememberedInThisRun::new` 改了；
+    /// - 在函数上面加 `#[cfg]` 让另一份同名函数编进来（另一份的叫停参数写成 `_stop_is_requested` 就不进下面六层的名单）；
+    /// - 长的那一层里宏展开出来的调用与绑定。
     #[test]
     fn every_layer_handed_the_stop_request_hands_it_on_or_asks_it() {
         let stop_parameter = concat!("stop_is_", "requested: &(dyn Fn() -> bool + Sync),");
-        let handed_on_as_the_last_argument = concat!("stop_is_", "requested,");
-        let lines_using_the_stop_request_in_the_layers_without_a_behaviour_test: [(&str, &[&str]);
-            4] = [
+        let pinned_layers_without_a_behaviour_test: [(&str, StopRequestUseThatIsPinned); 4] = [
             (
                 "judge_my_blocks_on_gpu_cards",
-                &[handed_on_as_the_last_argument],
+                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
+                    ")->Result<(CheckingTally,GpuJudgeOutcome),String>\u{7b}",
+                    "matchjudge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(",
+                    "store,flow,plan,is_mine,version,cards,GpuCardsSource::DevicesStartedForThisRun,stop_is_",
+                    "requested,)?\u{7b}",
+                    "GpuJudgingOfMyBlocks::Judged\u{7b}tally,outcome\u{7d}=>Ok((tally,outcome)),",
+                    "GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort\u{7b}reason,..\u{7d}=>Err(reason),",
+                    "\u{7d}"
+                )),
             ),
             (
                 "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from",
-                &[concat!(
-                    "let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
-                    "requested);"
-                )],
+                StopRequestUseThatIsPinned::TopLevelStatementOfALongBody {
+                    line: concat!(
+                        "let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
+                        "requested);"
+                    ),
+                    binding: "stop_request",
+                    type_name: "StopRequestRememberedInThisRun",
+                },
+            ),
+            (
+                "check_selected_blocks",
+                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
+                    ")->Result<CheckingTally,VerdictStoreError>\u{7b}",
+                    "check_blocks_where_until(store,flow,plan,",
+                    "&|block_sequence|selected_block_sequences.contains(&block_sequence),",
+                    "unit_checksums,version,stop_is_",
+                    "requested,)"
+                )),
+            ),
+            (
+                "judge_my_blocks_on_gpu",
+                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
+                    ")->Result<GpuJudgingOfMyBlocks,String>\u{7b}",
+                    "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(",
+                    "store,flow,plan,is_mine,version,cards,cards_taken_from,stop_is_",
+                    "requested,)"
+                )),
             ),
-            ("check_selected_blocks", &[handed_on_as_the_last_argument]),
-            ("judge_my_blocks_on_gpu", &[handed_on_as_the_last_argument]),
         ];
         let lines = code_lines_outside_the_tests();
         let mut layers_handed_the_stop_request = Vec::new();
@@ -4953,37 +5058,76 @@ mod tests {
                 .iter()
                 .take_while(|line| line.as_str() != "\u{7d}")
                 .collect();
-            if let Some((_layer, expected_lines)) =
-                lines_using_the_stop_request_in_the_layers_without_a_behaviour_test
-                    .iter()
-                    .find(|(layer_without_a_behaviour_test, _expected_lines)| {
-                        *layer_without_a_behaviour_test == layer.as_str()
-                    })
+            match pinned_layers_without_a_behaviour_test
+                .iter()
+                .find(|(layer_without_a_behaviour_test, _pinned)| {
+                    *layer_without_a_behaviour_test == layer.as_str()
+                })
+                .map(|(_layer, pinned)| pinned)
             {
-                let lines_using_it: Vec<(usize, &str)> = rest_of_the_layer
-                    .iter()
-                    .enumerate()
-                    .filter(|(_offset, line)| whole_name_uses_of_the_stop_request(line) > 0)
-                    .map(|(offset, line)| (offset, line.trim()))
-                    .collect();
-                assert_eq!(
-                    lines_using_it
+                Some(StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(expected_body)) => {
+                    let body_without_spaces: String = rest_of_the_layer
                         .iter()
-                        .map(|(_offset, line)| *line)
-                        .collect::<Vec<&str>>(),
-                    expected_lines.to_vec(),
-                    "{layer} 用到叫停的只有钉住的那几行、原样（只提一次名字而永不叫停的写法多一行或那一行变了）：{rest_of_the_layer:#?}"
-                );
-                for (offset, line) in &lines_using_it {
-                    if *line == handed_on_as_the_last_argument {
-                        assert!(
-                            rest_of_the_layer
-                                .get(offset + 1)
-                                .is_some_and(|next_line| next_line.trim_start().starts_with(')')),
-                            "{layer} 把叫停当调下一层的最后一个实参原样交下去：{rest_of_the_layer:#?}"
-                        );
-                    }
+                        .flat_map(|line| line.chars())
+                        .filter(|character| !character.is_whitespace())
+                        .collect();
+                    assert_eq!(
+                        body_without_spaces, *expected_body,
+                        "{layer} 参数行之后整句等于钉住的那一句（死分支里留一份原样的调用、真调用换实参，都多出或换掉一截）：{rest_of_the_layer:#?}"
+                    );
+                }
+                Some(StopRequestUseThatIsPinned::TopLevelStatementOfALongBody {
+                    line: pinned_line,
+                    binding,
+                    type_name,
+                }) => {
+                    let lines_using_it: Vec<(usize, &str)> = rest_of_the_layer
+                        .iter()
+                        .enumerate()
+                        .filter(|(_offset, line)| whole_name_uses_of_the_stop_request(line) > 0)
+                        .map(|(offset, line)| (offset, line.trim()))
+                        .collect();
+                    assert_eq!(
+                        lines_using_it
+                            .iter()
+                            .map(|(_offset, line)| *line)
+                            .collect::<Vec<&str>>(),
+                        vec![*pinned_line],
+                        "{layer} 用到叫停的只有钉住的那一行、原样（只提一次名字而永不叫停的写法多一行或那一行变了）：{rest_of_the_layer:#?}"
+                    );
+                    let depths = brace_depth_at_the_start_of_each_line(&rest_of_the_layer);
+                    let (pinned_offset, _pinned_line) = lines_using_it[0];
+                    assert_eq!(
+                        depths[pinned_offset], 1,
+                        "{layer} 的 {pinned_line} 是函数体顶层的一句，不在 `if false` 一类的块里（参数行之后那一行开函数体，算第 1 层）"
+                    );
+                    let let_bindings = rest_of_the_layer
+                        .iter()
+                        .filter(|line| {
+                            let statement = line.trim_start();
+                            [
+                                format!("let {binding} "),
+                                format!("let {binding}:"),
+                                format!("let mut {binding}"),
+                            ]
+                            .iter()
+                            .any(|binding_opening| statement.starts_with(binding_opening.as_str()))
+                        })
+                        .count();
+                    assert_eq!(
+                        let_bindings, 1,
+                        "{layer} 里 {binding} 只由钉住的那一行 let 绑一次（后面再 let 一次就换掉了记住的叫停）"
+                    );
+                    let lines_naming_the_type = rest_of_the_layer
+                        .iter()
+                        .filter(|line| line.contains(type_name))
+                        .count();
+                    assert_eq!(
+                        lines_naming_the_type, 1,
+                        "{layer} 里只有钉住的那一行提到 {type_name}（块里 use 或另起同名的类型会遮住它）"
+                    );
                 }
+                None => {}
             }
             layers_handed_the_stop_request.push(layer);
         }
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index 9b2dbd7..d2086c6 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -2408,11 +2408,20 @@ mod tests {
     /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4）：原先只数几段文本各出现一处、闭包三个，被这几种改了行为而编得过的写法骗过：
     /// 方法体最前面 `if true { return; }`（正推 A2）、丢没丢与 poll 两个同型闭包对调（正推 A3）、整个包进 `if false { … }`（攻方 gpu-2）、
     /// 方法体里另起一个同名的 `device_found_idle_by_a_poll` 恒交真遮住外面那个（攻方 gpu-3）；整句比之后这四种都红。
-    /// 整句比也拦不住的（文本都在这个方法体之外）：改 [`device_found_idle_by_a_poll`] 本身（由
-    /// `a_poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty` 管；`card_pool` 模块按名字从上一层引它，模块里再起一个同名的编译不过）；
-    /// 改 `GpuCard::device_condition`、`device_on_the_books` 放进去的是哪一台（`started_card`）、harness 档的 `DeviceStartedOnDemand::started_now`
-    /// 与 `DeviceConditionOfACard::is_lost`；在方法上面加 `#[cfg]` 让另一份同名方法编进来；四个入口不调这一步
-    /// （`the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first` 管，它也只看文本）。
+    /// 整句比管不到（文本都在这个方法体之外）、由别的用例管的：
+    /// - 改 [`device_found_idle_by_a_poll`] 本身：`a_poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty`；
+    /// - 卡池模块里另起模块（攻方 `mod wgpu`、`mod std`）、经 `use` 改名或通配、在条目这一层展开宏，遮住方法体里的 `std`、`wgpu`；去掉 `use`
+    ///   另起一个同名的 `device_found_idle_by_a_poll`（第 10z 批第十八块）：
+    ///   `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call`，它也只看文本；
+    /// - `GpuCard::device`、`GpuCard::device_condition` 交的是哪一台、哪一份（第 10z 批第十八块，正推 K4）：
+    ///   `a_card_hands_out_its_own_device_and_its_own_device_condition`，它也只看文本；
+    /// - 四个入口不调这一步：`the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first`，它也只看文本。
+    ///
+    /// 拦不住的写法，一种一条：
+    /// - `device_on_the_books` 放进去的是哪一台（`started_card`）；
+    /// - `GpuCard` 的 `device`、`device_condition` 两格里放的是哪一台、哪一份（`start_the_card` 起设备时放进去）；
+    /// - harness 档的 `DeviceStartedOnDemand::started_now` 与 `DeviceConditionOfACard::is_lost` 改了；
+    /// - 在方法上面加 `#[cfg]` 让另一份同名方法编进来。
     #[test]
     fn the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking() {
         let source = include_str!("gpu_unit_checks.rs");
@@ -2455,6 +2464,349 @@ mod tests {
         );
     }
 
+    /// `text` 以 `"` 起头时，那一个字符串字面量连两头引号的长度（反斜杠连同它转义的那个字跳过）。
+    fn length_of_the_quoted_literal_at_the_start(text: &str) -> usize {
+        let bytes = text.as_bytes();
+        let mut position = 1;
+        loop {
+            let byte = *bytes.get(position).expect("字符串字面量有收尾的引号");
+            if byte == b'"' {
+                return position + 1;
+            }
+            position += if byte == b'\\' { 2 } else { 1 };
+        }
+    }
+
+    /// `text` 以 `'` 起头时，那一个字符字面量连两头引号的长度；是生命周期（`'static`、`'_`）交 `None`。
+    fn length_of_the_character_literal_at_the_start(text: &str) -> Option<usize> {
+        let after_the_quote = &text[1..];
+        if after_the_quote.starts_with('\\') {
+            // 转义的（`'\''`、`'\n'`、`'\u{7b}'`）：收尾的引号在反斜杠与它后面那个字之后
+            let closing_quote = 3 + text[3..].find('\'').expect("转义的字符字面量有收尾的引号");
+            return Some(closing_quote + 1);
+        }
+        let first_character = after_the_quote.chars().next().expect("引号后面还有字");
+        after_the_quote[first_character.len_utf8()..]
+            .starts_with('\'')
+            .then(|| first_character.len_utf8() + 2)
+    }
+
+    /// `text` 是原始字符串去掉 `r`、`br` 之后那一截（以若干个 `#` 与一个 `"` 起头）时，到收尾的 `"` 与同样多个 `#` 为止的长度。
+    fn length_of_the_raw_string_at_the_start(text: &str) -> usize {
+        let hashes = text.len() - text.trim_start_matches('#').len();
+        let closing = format!("\"{}", "#".repeat(hashes));
+        let body_start = hashes + 1;
+        body_start + text[body_start..].find(&closing).expect("原始字符串有收尾") + closing.len()
+    }
+
+    /// `text` 以 `/*` 起头时，那一段块注释（可以嵌套）的长度。
+    fn length_of_the_block_comment_at_the_start(text: &str) -> usize {
+        let mut depth = 0_usize;
+        let mut position = 0;
+        loop {
+            let rest = &text[position..];
+            if rest.starts_with("/*") {
+                depth += 1;
+                position += 2;
+            } else if rest.starts_with("*/") {
+                depth -= 1;
+                position += 2;
+                if depth == 0 {
+                    return position;
+                }
+            } else {
+                position += rest.chars().next().expect("块注释有收尾").len_utf8();
+            }
+        }
+    }
+
+    /// `code` 拆成的词：标识符与关键字（`r#` 原始标识符取它后面的名字）、生命周期（连着它前面的 `'`）、数字（照标识符那样收一段）、`::`，
+    /// 其余标点一个字一个词；注释（行注释、可以嵌套的块注释）、字符串与字符字面量（普通的、原始的 `r#"…"#`、字节的）不出词。
+    /// 扫源码的用例照它找条目、`use` 引进来的名字与花括号的层数：字面量与注释里的 `mod`、`use`、花括号都不算。
+    /// 非 ASCII 的字只落在注释、字面量与标识符里，切出来的词因此总在字符边界上。
+    fn rust_tokens_without_comments_and_literals(code: &str) -> Vec<&str> {
+        let is_identifier_character =
+            |character: char| character.is_alphanumeric() || character == '_';
+        let mut tokens = Vec::new();
+        let mut position = 0;
+        while position < code.len() {
+            let rest = &code[position..];
+            let first_character = rest.chars().next().expect("没到结尾就还有字");
+            if first_character.is_whitespace() {
+                position += first_character.len_utf8();
+            } else if rest.starts_with("//") {
+                position += rest.find('\n').unwrap_or(rest.len());
+            } else if rest.starts_with("/*") {
+                position += length_of_the_block_comment_at_the_start(rest);
+            } else if first_character == '"' {
+                position += length_of_the_quoted_literal_at_the_start(rest);
+            } else if first_character == '\'' {
+                match length_of_the_character_literal_at_the_start(rest) {
+                    Some(literal_length) => position += literal_length,
+                    None => {
+                        let lifetime_length = 1 + rest[1..]
+                            .find(|character: char| !is_identifier_character(character))
+                            .unwrap_or(rest.len() - 1);
+                        tokens.push(&rest[..lifetime_length]);
+                        position += lifetime_length;
+                    }
+                }
+            } else if is_identifier_character(first_character) {
+                let word_length = rest
+                    .find(|character: char| !is_identifier_character(character))
+                    .unwrap_or(rest.len());
+                let word = &rest[..word_length];
+                let after_the_word = &rest[word_length..];
+                if matches!(word, "r" | "br")
+                    && after_the_word.trim_start_matches('#').starts_with('"')
+                {
+                    position += word_length + length_of_the_raw_string_at_the_start(after_the_word);
+                } else if word == "r" && after_the_word.starts_with('#') {
+                    // 原始标识符 `r#名字`：名字下一圈照标识符收
+                    position += word_length + 1;
+                } else if word == "b" && after_the_word.starts_with('"') {
+                    position +=
+                        word_length + length_of_the_quoted_literal_at_the_start(after_the_word);
+                } else if word == "b" && after_the_word.starts_with('\'') {
+                    position += word_length
+                        + length_of_the_character_literal_at_the_start(after_the_word)
+                            .expect("b 后面紧跟引号的是字节字面量");
+                } else {
+                    tokens.push(word);
+                    position += word_length;
+                }
+            } else if rest.starts_with("::") {
+                tokens.push("::");
+                position += 2;
+            } else {
+                tokens.push(&rest[..first_character.len_utf8()]);
+                position += first_character.len_utf8();
+            }
+        }
+        tokens
+    }
+
+    /// 这份源码（测试那一段之外）里卡池模块（`mod card_pool`）的词，各带它在模块里第几层花括号里：从模块的左花括号之后
+    /// 到与它配对的右花括号之前，模块的条目在第 0 层（左花括号算它外面那一层，右花括号算它收回到的那一层）。
+    fn card_pool_module_tokens_with_their_brace_depth(code: &str) -> Vec<(usize, &str)> {
+        let tokens = rust_tokens_without_comments_and_literals(code);
+        let opening = tokens
+            .windows(3)
+            .position(|window| window == ["mod", "card_pool", "\u{7b}"])
+            .expect("这份源码里有卡池模块")
+            + 3;
+        let mut depth = 0_usize;
+        let mut module_tokens = Vec::new();
+        for token in &tokens[opening..] {
+            if *token == "\u{7b}" {
+                module_tokens.push((depth, *token));
+                depth += 1;
+            } else if *token == "\u{7d}" {
+                let Some(depth_after_the_closing) = depth.checked_sub(1) else {
+                    return module_tokens;
+                };
+                depth = depth_after_the_closing;
+                module_tokens.push((depth, *token));
+            } else {
+                module_tokens.push((depth, *token));
+            }
+        }
+        panic!("卡池模块有与它的左花括号配对的右花括号")
+    }
+
+    /// 一条 `use`（`statement` 从 `use` 那个词到分号，至多一层花括号）引进来的每个名字连同引它的路径：名字是后面紧跟逗号、
+    /// 右花括号或分号的标识符；路径是它前面连着的 `段::` 那几段，在花括号里的再接上左花括号之前那一段，去掉收尾的 `::`。
+    fn names_brought_in_by_a_use_statement<'code>(
+        statement: &[&'code str],
+    ) -> Vec<(String, &'code str)> {
+        let is_identifier = |token: &str| {
+            token.starts_with(|character: char| character.is_alphanumeric() || character == '_')
+        };
+        let opening_brace = statement.iter().position(|token| *token == "\u{7b}");
+        statement
+            .iter()
+            .enumerate()
+            .skip(1)
+            .filter(|(leaf_index, token)| {
+                is_identifier(token)
+                    && statement
+                        .get(leaf_index + 1)
+                        .is_some_and(|next| matches!(*next, "," | "\u{7d}" | ";"))
+            })
+            .map(|(leaf_index, leaf)| {
+                let mut path_start = leaf_index;
+                while path_start >= 3
+                    && statement[path_start - 1] == "::"
+                    && is_identifier(statement[path_start - 2])
+                {
+                    path_start -= 2;
+                }
+                let mut path = statement[path_start..leaf_index].join("");
+                if let Some(brace) = opening_brace.filter(|brace| *brace < leaf_index) {
+                    path = format!("{}{path}", statement[1..brace].join(""));
+                }
+                (path.trim_end_matches("::").to_string(), *leaf)
+            })
+            .collect()
+    }
+
+    /// 第 10z 批第十八块（第 10z 批第十四、十五块代码轮判决 K4，攻方：卡池模块里加 `mod wgpu { … }` 或 `mod std { … }` 遮住方法体里的
+    /// `wgpu::PollType::Poll`、`std::panic::catch_unwind`，方法体一个字不动、`cargo check` 0 警告编过，整句比照绿）：钉住的两段（借之前那一步、
+    /// 借据丢掉时那一步）里按路径首段解析的 `std`、`wgpu` 与按名字调的那几个函数，在卡池模块这一层只落到外部 crate 与 `use` 点名的那一处。
+    /// 这一层能把一个名字换掉的只有这几样，逐样拒：另起模块、`extern crate`、在条目这一层调宏或定义宏（宏展开出来的条目也能叫 `std`、`wgpu`）、
+    /// `use` 改名（`as`）、引 `self`、通配（`*`，通配引进来的模块同样遮住外部 crate）、`use` 引进叫 `std` 或 `wgpu` 的名字；
+    /// 那几个函数各由一条 `use` 从钉住的那一处引进来（引了它，模块里再起同名的条目编译不过；去掉 `use` 另起一个就是这里红）。
+    /// 扫这份源码（测试那一段之外，注释与字面量不算），不要卡。
+    ///
+    /// 拦不住的写法（扫这一个模块的源码看不见），一种一条：
+    /// - 过程宏（派生宏、属性宏）在卡池模块里展开出来的条目；
+    /// - crate 根上改 `std` 指什么（`#![no_std]` 加 `extern crate … as std`）；
+    /// - `Cargo.toml` 里把 `wgpu` 换成另一个包（`package = …`）；
+    /// - 被引的那一处自己换了：`super::super` 那一层的同名函数改了，或改成从别处 `pub use` 进来。
+    #[test]
+    fn the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call()
+    {
+        let source = include_str!("gpu_unit_checks.rs");
+        let code = source
+            .split(TESTS_MODULE_OPENING)
+            .next()
+            .expect("split 至少交一段");
+        let module_tokens = card_pool_module_tokens_with_their_brace_depth(code);
+        let tokens: Vec<&str> = module_tokens.iter().map(|(_depth, token)| *token).collect();
+        let declaring_keywords: Vec<&str> = tokens
+            .iter()
+            .copied()
+            .filter(|token| matches!(*token, "mod" | "extern"))
+            .collect();
+        assert_eq!(
+            declaring_keywords,
+            Vec::<&str>::new(),
+            "卡池模块里不另起模块、不写 extern crate：`mod wgpu`、`mod std`、`extern crate … as std` 都遮住钉住那两段里按路径首段解析的外部 crate"
+        );
+        let item_level_macros: Vec<&str> = module_tokens
+            .windows(2)
+            .filter(|pair| {
+                pair[0].0 == 0
+                    && pair[1].1 == "!"
+                    && pair[0].1.starts_with(|character: char| {
+                        character.is_alphanumeric() || character == '_'
+                    })
+            })
+            .map(|pair| pair[0].1)
+            .collect();
+        assert_eq!(
+            item_level_macros,
+            Vec::<&str>::new(),
+            "卡池模块的条目这一层不调宏、不定义宏：宏展开出来的条目也能叫 std、wgpu"
+        );
+        let use_statements: Vec<&[&str]> = tokens
+            .iter()
+            .enumerate()
+            .filter(|(_index, token)| **token == "use")
+            .map(|(use_index, _token)| {
+                let length = tokens[use_index..]
+                    .iter()
+                    .position(|token| *token == ";")
+                    .expect("use 以分号收尾")
+                    + 1;
+                &tokens[use_index..use_index + length]
+            })
+            .collect();
+        for statement in &use_statements {
+            assert!(
+                !statement
+                    .iter()
+                    .any(|token| matches!(*token, "as" | "self" | "*")),
+                "卡池模块的 use 不改名、不引 self、不通配（这几样都能把一个外部 crate 的名字换成别的模块）：{statement:?}"
+            );
+            assert!(
+                statement.iter().filter(|token| **token == "\u{7b}").count() <= 1,
+                "卡池模块的 use 至多一层花括号（引进来的名字照这一层认）：{statement:?}"
+            );
+        }
+        let brought_in: Vec<(String, &str)> = use_statements
+            .iter()
+            .flat_map(|statement| names_brought_in_by_a_use_statement(statement))
+            .collect();
+        let crate_roots_brought_in: Vec<&(String, &str)> = brought_in
+            .iter()
+            .filter(|(_prefix, name)| matches!(*name, "std" | "wgpu"))
+            .collect();
+        assert_eq!(
+            crate_roots_brought_in,
+            Vec::<&(String, &str)>::new(),
+            "卡池模块不经 use 引进叫 std、wgpu 的名字"
+        );
+        for (expected_prefix, function) in [
+            ("super::super", "device_found_idle_by_a_poll"),
+            ("super::super", "poll_type_before_returning_a_lease"),
+            ("super::super", "device_wait_before_returning_a_lease"),
+            (
+                "singlefs_harness::gpu_memory_ledger",
+                "wait_for_the_device_and_take_in_the_outcome",
+            ),
+        ] {
+            assert_eq!(
+                brought_in
+                    .iter()
+                    .filter(|(_prefix, name)| *name == function)
+                    .map(|(prefix, _name)| prefix.as_str())
+                    .collect::<Vec<&str>>(),
+                vec![expected_prefix],
+                "钉住那两段按名字调的 {function} 由一条 use 从 {expected_prefix} 引进卡池模块（引了它，模块里再起同名的条目编译不过）"
+            );
+        }
+    }
+
+    /// 第 10z 批第十八块（第 10z 批第十四、十五块代码轮正推 K4：钉住的那一句里 `started.device()` 交的是哪一台不在整句里，
+    /// `started.device_condition()` 同理）：一张卡交出去的设备与设备的情况就是它自己那两格。要真卡上起好的设备才造得出 `GpuCard`，
+    /// 换不成不要卡的行为用例，这里扫这份源码（测试那一段之外、去掉注释行与空白）：两个方法各恰好一处，方法体整个是那一格。不要卡。
+    ///
+    /// 拦不住的写法，一种一条：
+    /// - 这两格里放的是哪一台、哪一份（`start_the_card` 起设备时放进去，克隆共用同一台）；
+    /// - 在方法上面加 `#[cfg]`，让签名写法与钉住的不同的另一份同名方法编进来。
+    #[test]
+    fn a_card_hands_out_its_own_device_and_its_own_device_condition() {
+        let source = include_str!("gpu_unit_checks.rs");
+        let code = source
+            .split(TESTS_MODULE_OPENING)
+            .next()
+            .expect("split 至少交一段");
+        for (signature, expected_body) in [
+            (
+                "pub fn device(&self) -> &wgpu::Device \u{7b}",
+                "&self.device",
+            ),
+            (
+                "pub fn device_condition(&self) -> &DeviceConditionOfACard \u{7b}",
+                "&self.device_condition",
+            ),
+        ] {
+            assert_eq!(
+                code.matches(signature).count(),
+                1,
+                "源码里恰好一处 {signature}"
+            );
+            let start = code
+                .find(signature)
+                .unwrap_or_else(|| panic!("源码里有 {signature}"));
+            let after = &code[start + signature.len()..];
+            let body = &after[..after
+                .find("\n    \u{7d}\n")
+                .expect("GpuCard 的方法以缩进四格的右括号收尾")];
+            let body_without_comments_and_spaces: String = body
+                .lines()
+                .filter(|line| !line.trim_start().starts_with("//"))
+                .flat_map(str::chars)
+                .filter(|character| !character.is_whitespace())
+                .collect();
+            assert_eq!(
+                body_without_comments_and_spaces, expected_body,
+                "{signature} 交的是这张卡自己那一格"
+            );
+        }
+    }
+
     /// Cargo.lock 里 wgpu-core 的版本（`name = "wgpu-core"` 下一行的 `version = "…"`）。
     fn wgpu_core_version_in_the_lock(lock: &str) -> Option<String> {
         let mut lines = lock.lines();
````

### 第十八块 mutations-append.tsv

````
第10z批第十八块 GPU 取卡 K4 卡池模块里加同名 mod wgpu 遮住借之前那一步的 poll、改成一直等到设备空（攻方 K4）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	    mod card_pool {\n        use super::super::{	    mod card_pool {\n        #[allow(non_snake_case, non_upper_case_globals, unused_imports, dead_code, reason = "变异：遮住方法体里的 wgpu")]\n        mod wgpu {\n            pub use ::wgpu::*;\n            pub mod PollType {\n                pub const Poll: ::wgpu::PollType = ::wgpu::PollType::wait_indefinitely();\n            }\n        }\n        use super::super::{	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call	the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call
第10z批第十八块 GPU 取卡 K4 卡池模块里加同名 mod std 遮住借之前那一步的 catch_unwind、不 poll 恒交队列空了（攻方 K4）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	    mod card_pool {\n        use super::super::{	    mod card_pool {\n        #[allow(unused_imports, dead_code, reason = "变异：遮住方法体里的 std")]\n        mod std {\n            pub use ::std::*;\n            pub mod panic {\n                pub use ::std::panic::*;\n                pub fn catch_unwind<Work: FnOnce() -> Result<::wgpu::PollStatus, ::wgpu::PollError>>(\n                    _work: AssertUnwindSafe<Work>,\n                ) -> ::std::thread::Result<Result<::wgpu::PollStatus, ::wgpu::PollError>> {\n                    Ok(Ok(::wgpu::PollStatus::QueueEmpty))\n                }\n            }\n        }\n        use super::super::{	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call	the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call
第10z批第十八块 GPU 取卡 K4 卡池模块经 use 改名把上一层的模块引成 std、遮住借之前那一步的 catch_unwind（攻方 K4 那一形换成 use … as）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	    mod card_pool {\n        use super::super::{	    #[allow(unused_imports, dead_code, reason = "变异：卡池模块经 use 改名引成 std")]\n    mod std_shadowing_the_crate {\n        pub use ::std::*;\n        pub mod panic {\n            pub use ::std::panic::*;\n            pub fn catch_unwind<Work: FnOnce() -> Result<::wgpu::PollStatus, ::wgpu::PollError>>(\n                _work: AssertUnwindSafe<Work>,\n            ) -> ::std::thread::Result<Result<::wgpu::PollStatus, ::wgpu::PollError>> {\n                Ok(Ok(::wgpu::PollStatus::QueueEmpty))\n            }\n        }\n    }\n    mod card_pool {\n        use super::std_shadowing_the_crate as std;\n        use super::super::{	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call	the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call
第10z批第十八块 GPU 取卡 K4 卡池模块去掉 device_found_idle_by_a_poll 的 use、另起同名函数恒交真（攻方 gpu-3 那一形挪到模块这一层）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        use super::super::{\n            card_memory_reader_of_the_card_pool, device_found_idle_by_a_poll,\n	        fn device_found_idle_by_a_poll(\n            _polled: &std::thread::Result<Result<wgpu::PollStatus, wgpu::PollError>>,\n        ) -> bool {\n            true\n        }\n        use super::super::{\n            card_memory_reader_of_the_card_pool,\n	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call	the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call
第10z批第十八块 GPU 取卡 K4 GpuCard::device 交这个进程里头一张问过的卡的设备（正推 K4 device() 不在整句里）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	    pub fn device(&self) -> &wgpu::Device {\n        &self.device\n    }	    pub fn device(&self) -> &wgpu::Device {\n        static DEVICE_OF_THE_FIRST_CARD_ASKED: std::sync::OnceLock<wgpu::Device> = std::sync::OnceLock::new();\n        DEVICE_OF_THE_FIRST_CARD_ASKED.get_or_init(|| self.device.clone())\n    }	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::a_card_hands_out_its_own_device_and_its_own_device_condition	a_card_hands_out_its_own_device_and_its_own_device_condition
第10z批第十八块 GPU 取卡 K4 GpuCard::device_condition 交一份从不丢的设备情况	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	    pub fn device_condition(&self) -> &DeviceConditionOfACard {\n        &self.device_condition\n    }	    pub fn device_condition(&self) -> &DeviceConditionOfACard {\n        static CONDITION_THAT_IS_NEVER_LOST: std::sync::OnceLock<DeviceConditionOfACard> = std::sync::OnceLock::new();\n        CONDITION_THAT_IS_NEVER_LOST.get_or_init(DeviceConditionOfACard::default)\n    }	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::a_card_hands_out_its_own_device_and_its_own_device_condition	a_card_hands_out_its_own_device_and_its_own_device_condition
第10z批第十八块 崩溃放量 K4 check_selected_blocks 真调用的叫停换成永不叫停、钉住那一行留在 if false 死分支里（正推 N1）	crates/singlefs-checker-tier/src/crash_amplification.rs	    check_blocks_where_until(\n        store,\n        flow,\n        plan,\n        &|block_sequence| selected_block_sequences.contains(&block_sequence),\n        unit_checksums,\n        version,\n        stop_is_requested,\n    )\n}	    if false {\n        let _ = check_blocks_where_until(\n            store,\n            flow,\n            plan,\n            &|block_sequence| selected_block_sequences.contains(&block_sequence),\n            unit_checksums,\n            version,\n            stop_is_requested,\n        );\n    }\n    check_blocks_where_until(\n        store,\n        flow,\n        plan,\n        &|block_sequence| selected_block_sequences.contains(&block_sequence),\n        unit_checksums,\n        version,\n        &|| false,\n    )\n}	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
第10z批第十八块 崩溃放量 K4 第 ① 段在卡上判记住的叫停换成永不叫停、钉住那一行留在 if false 死分支里（正推 N1 那一形挪到长的那一层）	crates/singlefs-checker-tier/src/crash_amplification.rs	    let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);	    if false {\n        let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);\n        drop(stop_request);\n    }\n    let never_stop = || false;\n    let stop_request = StopRequestRememberedInThisRun::new(&never_stop);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
第10z批第十八块 崩溃放量 K4 第 ① 段在卡上判钉住那一行之后再 let 一次、记住的叫停换成永不叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	    let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);	    let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);\n    let never_stop = || false;\n    let stop_request = StopRequestRememberedInThisRun::new(&never_stop);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
````

## 第十七块

````diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -2620,6 +2620,43 @@
         assert_eq!(map.free_runs(), 1);
     }
 
+    /// [`DeviceFreeMap::slots_counted_both_as_allocated_and_as_isolated`] 的阳性对照（判决 `research/prompts/m3-batch10z-no-code-r1-main-verification.md`
+    /// 第二节 K2 ②：harness 档用例里它全是比 0，它恒交 0 时那些用例照绿）。一块盘上分配三段（起点槽起两槽、第 4 槽起一槽、第 8 槽起两槽），
+    /// 再用不看分配位的整段 [`DeviceFreeMap::isolate`] 罩住第 0 到第 5 槽：第 0、1、4 槽同时置着分配位与隔离位，第 2、3、5 槽只置隔离位，
+    /// 第 8、9 槽只置分配位。访问器交 3，隔离数 6、已分配数 5 各自对得上。
+    #[test]
+    fn isolating_a_whole_span_over_allocated_slots_counts_each_overlapping_slot_both_as_allocated_and_as_isolated(
+    ) {
+        let device_bytes = (UNIT_AREA_START_SLOT + 385) * SLOT_BYTES;
+        let mut map = DeviceFreeMap::new(DeviceIdentity(0), device_bytes);
+        let first_slot = map.unit_area_start().slot();
+        let slot_at = |offset_in_slots: u64| SlotNumber(first_slot.0 + offset_in_slots);
+        map.mark_allocated(slot_at(0), 2);
+        map.mark_allocated(slot_at(4), 1);
+        map.mark_allocated(slot_at(8), 2);
+        assert_eq!(
+            map.slots_counted_both_as_allocated_and_as_isolated(),
+            0,
+            "隔离之前没有一个槽置着隔离位"
+        );
+        map.isolate(slot_at(0), 6);
+        assert_eq!(
+            map.slots_counted_both_as_allocated_and_as_isolated(),
+            3,
+            "整段隔离罩住的第 0、1、4 槽同时置着分配位与隔离位"
+        );
+        assert_eq!(
+            map.isolated_slots(),
+            3 + 3,
+            "隔离着的槽：同时置两位的 3 个加只置隔离位的第 2、3、5 槽"
+        );
+        assert_eq!(
+            map.allocated_slots(),
+            3 + 2,
+            "已分配的槽：同时置两位的 3 个加没被罩住的第 8、9 槽"
+        );
+    }
+
     #[test]
     fn new_pool_file_creation_placements_follow_the_byte_table() {
         let mut pool = pool_after_mkfs();
diff --git a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
--- a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -3255,6 +3255,149 @@
     }
 }
 
+/// 抬 F 那一串卸载那一格当场放开那一截回收时照样补隔离（判决 `research/prompts/m3-batch10z-no-code-r1-main-verification.md`
+/// 第二节 K2 ①；形照攻方 `research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-u1.txt`：384 槽、覆盖写 10 次、回退到倒数第 7 版录 R、
+/// 挂载那一刻 R 的根槽与谱系片读回全零而挂上之后照旧、挂上就卸载）。与 [`revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero`]
+/// 同一个池、同一次挂载，只把挂上之后那一串抬 F 换成正常卸载：R 复活的数据单元在 P 的账里已释放、还在 defer 窗口里，挂载那一刻不隔离
+/// （D28（挂载期承诺量） 已定项 1 第九项不再算一次），卸载那一串回收它们；R 的根槽照旧读回全零，这一串不把 R 当被抛弃根，护着它们的只有 C 那一圈。
+/// 卸载之后 R 还在根环里：C 那一圈罩着的槽一个都不空闲，复活的那几个隔离着，没有一个槽同时算进「已分配」与第九项。
+/// 当场放开那一截回收之后不补隔离（改回 `PoolAllocator::reclaim_released_up_to`）时，复活的槽卸载之后空闲，这一条红。扣到生效那一截改回只回收，
+/// 这一条照绿（第 10z 批第十七块实现员证红时量的），那一截不在这里钉。
+#[test]
+fn unmount_right_after_the_mount_isolates_the_revived_units_it_reclaims_while_the_root_slot_of_the_rollback_reads_back_zero(
+) {
+    let RecordedRollback {
+        recorded, revived, ..
+    } = record_one_rollback_after_overwrites(
+        OVERWRITES_BEFORE_THE_ROLLBACK_WHOSE_PLACEMENTS_A_FLOOR_RAISE_RECLAIMS,
+        VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
+    );
+    assert!(
+        !revived.is_empty(),
+        "回退目标的数据单元 P 不引用：回退复活了它们（阳性对照：这一形走到了复活）"
+    );
+    let lineage_unit_writes = lineage_unit_writes_of(&recorded);
+    assert!(
+        !lineage_unit_writes.is_empty(),
+        "回退那次发布写了谱系片（阳性对照：故障有处可打）"
+    );
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let devices = devices_reading_back_all_zero_on(
+        &crash_image,
+        &RangesReadingBackAllZero::default()
+            .with_the_root_slot_of(&crash_image, recorded.key)
+            .with_the_writes(&lineage_unit_writes),
+    );
+    let mut pool = PoolUnderTest {
+        parameters: recorded.pool.parameters.clone(),
+        device_bytes: recorded.pool.device_bytes,
+        devices,
+        session: None,
+        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
+        content_of_the_current_version: recorded.content.clone(),
+        content_of_each_root: recorded.pool.content_of_each_root.clone(),
+    };
+    let mounted = mount_writable_with_space_admission(
+        &pool.parameters,
+        &mut pool.devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("R 的根槽与谱系片读回全零而记录齐：可写挂载落到 P、做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 的根槽读不出、系统配置没见证 R、记录不施加：择根落到 P（阳性对照：故障打到了）"
+    );
+    let named_by_the_unapplied_records_at_the_mount = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .placements_named_by_unapplied_journal_records();
+    let revived_not_in_c: Vec<&PlacementOnDevice> = revived
+        .iter()
+        .filter(|revived_placement| {
+            !named_by_the_unapplied_records_at_the_mount
+                .iter()
+                .any(|placement| {
+                    placement.device == revived_placement.device
+                        && placement.slot == revived_placement.slot
+                })
+        })
+        .collect();
+    assert!(
+        revived_not_in_c.is_empty(),
+        "C 那一圈并进了 R 的账（记录的新根段读得出），R 复活的数据单元都在里面（阳性对照），缺 {revived_not_in_c:?}"
+    );
+    // R 的根槽与谱系片挂上之后照旧读回全零，到这个进程写掉那几段为止：卸载那一串读不出 R，不把它当被抛弃根算影子账。
+    let (_output, mut session) = MountedSession::of_the_mount(mounted);
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut session.allocator,
+        &mut session.current,
+        ShadowLedger::On,
+    );
+    let reclaimed_by_the_unmount = match unmounted {
+        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => raised.reclaimed,
+        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
+            panic!("挂载落到的 P 带文件，卸载要推一串抬 F：{current:?}")
+        }
+        Err(refusal) => panic!("挂上就卸载做得成：{refusal:?}"),
+    };
+    assert!(
+        revived.iter().any(|revived_placement| {
+            reclaimed_by_the_unmount
+                .iter()
+                .any(|reclaimed| reclaimed.slot == revived_placement.slot)
+        }),
+        "卸载那一串回收了复活的数据单元（阳性对照：卸载那一格回收时补隔离那一处走到了），回收的 {reclaimed_by_the_unmount:?}"
+    );
+    assert!(
+        root_is_in_the_ring(&pool.image(), recorded.key),
+        "卸载之后 R 还在根环里（卸载那一串没写到 R 那一槽）"
+    );
+    let named_by_the_unapplied_records_after_the_unmount = session
+        .allocator
+        .root_ring_occupancy()
+        .expect("卸载之后分配器照旧装着根环表")
+        .placements_named_by_unapplied_journal_records();
+    let free_among_the_named = free_slots_covered_by(
+        &session.allocator,
+        &named_by_the_unapplied_records_after_the_unmount,
+    );
+    assert!(
+        free_among_the_named.is_empty(),
+        "卸载之后 R 还在根环里：C 那一圈罩着的槽一个都不空闲（卸载那一串回收时照 C 那一圈补了隔离），空闲的（盘、槽）{free_among_the_named:?}"
+    );
+    let revived_slots_not_isolated: Vec<(DeviceIdentity, SlotNumber)> = revived
+        .iter()
+        .flat_map(|revived_placement| {
+            let device_map = session
+                .allocator
+                .devices
+                .iter()
+                .find(|device_map| device_map.device == revived_placement.device)
+                .expect("池里有这块盘");
+            (revived_placement.slot.0..revived_placement.slot.0 + revived_placement.span)
+                .map(SlotNumber)
+                .filter(move |slot| !device_map.is_isolated(*slot))
+                .map(move |slot| (revived_placement.device, slot))
+        })
+        .collect();
+    assert!(
+        revived_slots_not_isolated.is_empty(),
+        "卸载之后复活的数据单元（P 的账里已释放、卸载那一串回收了）覆盖的槽都隔离着，没隔离的（盘、槽）{revived_slots_not_isolated:?}"
+    );
+    assert_eq!(
+        slots_counted_both_as_allocated_and_as_isolated(&session.allocator),
+        0,
+        "卸载之后没有一个槽同时算进「已分配」与第九项"
+    );
+}
+
 /// 第 10z 批第十四块实现员报告第六节 384 槽盘卡死的几形（调查报告 `~/.local/share/singlefs/work/m3-r1/investigate-10z-p/report.md` 第五节 22 形里取的）：
 /// R 是哪一种、录在几次覆盖写之后，挂载那一刻读回全零的是哪几段（挂上就撤）。
 #[derive(Clone, Copy, Debug)]
````

### 第十七块 mutations-append.tsv

````
第10z批第十七块 K2 访问器恒交 0（分配位与隔离位同时置着的槽一个都不数）	crates/singlefs-core/src/allocator.rs	                .filter(|(is_allocated, is_isolated)| **is_allocated && **is_isolated)	                .filter(|(_is_allocated, _is_isolated)| false)	-p singlefs-core --lib	isolating_a_whole_span_over_allocated_slots_counts_each_overlapping_slot_both_as_allocated_and_as_isolated
第10z批第十七块 K2 卸载那一格当场放开那一截回收之后不补隔离（改回只回收）	crates/singlefs-core/src/mount.rs	                Some(newest_valid_root_on_every_device) => {\n                    reclaim_released_in_the_sequence_raising_the_floor_up_to(\n                        allocator,\n                        shadow_ledger,\n                        floor_of_the_reclaim.min(newest_valid_root_on_every_device),\n                        ReclaimedReuse::Immediately,\n                    )\n                }	                Some(newest_valid_root_on_every_device) => allocator.reclaim_released_up_to(\n                    floor_of_the_reclaim.min(newest_valid_root_on_every_device),\n                    ReclaimedReuse::Immediately,\n                ),	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- unmount_right_after_the_mount_isolates_the_revived_units_it_reclaims_while_the_root_slot_of_the_rollback_reads_back_zero	unmount_right_after_the_mount_isolates_the_revived_units_it_reclaims_while_the_root_slot_of_the_rollback_reads_back_zero
````

## 第十九块

````diff
diff --git a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
index 13ed0c1..a93d70d 100644
--- a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -71,7 +71,10 @@ use singlefs_core::mounted_session::{
     UserChangeRefused, WideProtectionAtTheCeiling,
 };
 use singlefs_core::pointer::{LocationEntry, NodePointer};
-use singlefs_core::recovery::{choose_system_configuration, instance_table_chain_of_root};
+use singlefs_core::recovery::{
+    choose_system_configuration, every_root_ring_slot, instance_table_chain_of_root,
+    read_root_ring_slot, RootRingSlotReading,
+};
 use singlefs_core::root_record::RootRecord;
 use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
 use singlefs_core::transaction::{PoolVersion, PublishError};
@@ -3262,7 +3265,8 @@ fn revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims
 /// （D28（挂载期承诺量） 已定项 1 第九项不再算一次），卸载那一串回收它们；R 的根槽照旧读回全零，这一串不把 R 当被抛弃根，护着它们的只有 C 那一圈。
 /// 卸载之后 R 还在根环里：C 那一圈罩着的槽一个都不空闲，复活的那几个隔离着，没有一个槽同时算进「已分配」与第九项。
 /// 当场放开那一截回收之后不补隔离（改回 `PoolAllocator::reclaim_released_up_to`）时，复活的槽卸载之后空闲，这一条红。扣到生效那一截改回只回收，
-/// 这一条照绿（第 10z 批第十七块实现员证红时量的），那一截不在这里钉。
+/// 这一条照绿（第 10z 批第十七块实现员证红时量的），那一截由
+/// [`unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held_until_the_floor_takes_effect_while_the_account_of_r_reads_back_zero`] 钉。
 #[test]
 fn unmount_right_after_the_mount_isolates_the_revived_units_it_reclaims_while_the_root_slot_of_the_rollback_reads_back_zero(
 ) {
@@ -3398,6 +3402,274 @@ fn unmount_right_after_the_mount_isolates_the_revived_units_it_reclaims_while_th
     );
 }
 
+/// 镜像根环里每块盘上读得出、自证过的根（`except` 那一条除外）各取 txg 最大的一条，再在各盘之间取最小：卸载那一格当场放开那一截
+/// 放到哪儿（`mount::newest_valid_root_txg_on_every_device`）在这一形上的值，从盘上独立读、不经被测的方法。`except` 是按现行那一版的
+/// 实例表被抛弃的那一条（这一形里只有 R）；有一块盘上一条根都读不出就判红（这一形里每块盘上都有这次挂载落的根）。
+fn smallest_among_the_newest_root_txg_on_each_device(
+    image: &MemoryPool,
+    except: (InstanceGeneration, CheckpointTxg),
+) -> CheckpointTxg {
+    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
+    let immutable = &configuration.immutable;
+    let mut newest_on_each_device: BTreeMap<DeviceIdentity, CheckpointTxg> = BTreeMap::new();
+    // 迭代上界是根环槽数（3 区 × 每区槽数）；跨轮携带的是每块盘上到这一槽为止读到的最大 txg。
+    for ring_slot in every_root_ring_slot(&immutable.sizes) {
+        let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
+        match read_root_ring_slot(
+            image,
+            &immutable.region_devices,
+            &immutable.sizes,
+            &immutable.filesystem_identifier,
+            ring_slot,
+        ) {
+            RootRingSlotReading::SelfVerified(root)
+                if (root.instance, root.checkpoint_txg) != except =>
+            {
+                let newest = newest_on_each_device
+                    .entry(device)
+                    .or_insert(root.checkpoint_txg);
+                *newest = (*newest).max(root.checkpoint_txg);
+            }
+            RootRingSlotReading::SelfVerified(_) | RootRingSlotReading::Bad(_) => {}
+        }
+    }
+    let devices_of_the_pool: std::collections::BTreeSet<DeviceIdentity> =
+        immutable.region_devices.iter().copied().collect();
+    assert_eq!(
+        newest_on_each_device
+            .keys()
+            .copied()
+            .collect::<std::collections::BTreeSet<_>>(),
+        devices_of_the_pool,
+        "每块盘上都读得出一条不被抛弃的根（这次挂载的暖机在每块盘上都落了根）"
+    );
+    newest_on_each_device
+        .into_values()
+        .min()
+        .expect("池里至少一块盘")
+}
+
+/// `placements` 里每个落点覆盖的槽（起点槽起、跨度那么多槽），按落点的次序。
+fn slots_covered_by(placements: &[PlacementOnDevice]) -> Vec<(DeviceIdentity, SlotNumber)> {
+    placements
+        .iter()
+        .flat_map(|placement| {
+            (placement.slot.0..placement.slot.0 + placement.span)
+                .map(move |slot| (placement.device, SlotNumber(slot)))
+        })
+        .collect()
+}
+
+/// `placements` 里每个落点覆盖的槽里此刻隔离位置着的那几个（盘、槽），按落点的次序。
+fn isolated_slots_covered_by(
+    allocator: &PoolAllocator,
+    placements: &[PlacementOnDevice],
+) -> Vec<(DeviceIdentity, SlotNumber)> {
+    slots_covered_by(placements)
+        .into_iter()
+        .filter(|(device, slot)| {
+            allocator
+                .devices
+                .iter()
+                .find(|device_map| device_map.device == *device)
+                .expect("池里有这块盘")
+                .is_isolated(*slot)
+        })
+        .collect()
+}
+
+/// 抬 F 那一串卸载那一格扣到生效那一截回收时照样补隔离（判决 `research/prompts/m3-batch10z-no-code-r1-main-verification.md` 第二节 K2 ②；
+/// 形照第 10z 批第十七块实现员报告「停下交主 agent」推的方向，第 10z 批第十九块实现员定形，被攻过零轮）。池与挂载同丙那一形
+/// （[`the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor`]：
+/// 覆盖写 3 次得 P、录一次建 inode 得 R，崩在 R 的根槽落盘之后、轮换之前，R 的账读回全零、挂上之后照旧）：择根退到 P，C 里 R 那一槽并上
+/// 挂载那一刻现行账里仍分配着的落点（R 与 P 共用的数据单元），它们分配位置着、挂载那一刻不隔离（D28（挂载期承诺量） 已定项 1 第九项不再算一次）。
+/// 挂上之后覆盖写一次：会话自己释放这几个数据单元，释放代是这次覆盖写的 txg，高于各盘最新根取最小（这一 txg 的根只落在一块盘上），
+/// 卸载那一串回收它们落在扣到生效那一截、不在当场放开那一截。卸载之后 R 还在根环里：它们隔离着、一槽都不空闲，C 那一圈罩着的槽一个都不空闲，
+/// 没有一个槽同时算进「已分配」与第九项（第九项「这次挂载里释放、回收时分配器照补隔离」，I-7.4（近 K 代块未被复用））。
+/// 扣到生效那一截回收之后不补隔离（改回 `PoolAllocator::reclaim_released_up_to`）时，卸载末尾放开扣住位之后它们空闲，这一条红。
+#[test]
+fn unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held_until_the_floor_takes_effect_while_the_account_of_r_reads_back_zero(
+) {
+    let recorded = record_one_inode_creation_after_overwrites(
+        OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_R,
+    );
+    let root_of_r = recorded.root();
+    let shared_with_the_previous_version =
+        data_unit_placements_r_shares_with_the_previous_version(&recorded);
+    assert!(
+        !shared_with_the_previous_version.is_empty(),
+        "R 建一个 inode、不写文件的数据单元：它与 P 共用文件的数据单元（阳性对照：有要护的槽）"
+    );
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let mut devices = devices_reading_back_all_zero_on(
+        &crash_image,
+        &RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded)),
+    );
+    let mounted = mount_writable_with_space_admission(
+        &recorded.pool.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 的账读回全零：R 走读不成、择根退到 P（阳性对照：故障打到了）"
+    );
+    let borrowed_at_the_mount = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+            root_of_r.instance,
+            root_of_r.checkpoint_txg,
+        )
+        .expect("C 那一圈在 R 那一槽记着落点");
+    let shared_not_borrowed: Vec<&PlacementOnDevice> = shared_with_the_previous_version
+        .iter()
+        .filter(|shared| {
+            !borrowed_at_the_mount
+                .iter()
+                .any(|borrowed| borrowed.device == shared.device && borrowed.slot == shared.slot)
+        })
+        .collect();
+    assert!(
+        shared_not_borrowed.is_empty(),
+        "挂载那一刻 C 里 R 那一槽并上了现行账里仍分配着的落点（丙）：R 与 P 共用的数据单元都在里面，缺 {shared_not_borrowed:?}"
+    );
+    let shared_free_at_the_mount =
+        free_slots_covered_by(&mounted.allocator, &shared_with_the_previous_version);
+    assert!(
+        shared_free_at_the_mount.is_empty(),
+        "挂载那一刻 R 与 P 共用的数据单元在现行账里仍分配着：一槽都不空闲，空闲的（盘、槽）{shared_free_at_the_mount:?}"
+    );
+    let shared_isolated_at_the_mount =
+        isolated_slots_covered_by(&mounted.allocator, &shared_with_the_previous_version);
+    assert!(
+        shared_isolated_at_the_mount.is_empty(),
+        "挂载那一刻 C 那一圈不隔离分配位置着的槽（阳性对照：卸载之后的隔离是回收那一刻补的），隔离着的（盘、槽）{shared_isolated_at_the_mount:?}"
+    );
+    let (output, session_of_the_mount) = MountedSession::of_the_mount(mounted);
+    let content_of_the_selected_version = recorded
+        .pool
+        .content_of_each_root
+        .get(&(
+            output.effective_root.instance,
+            output.effective_root.checkpoint_txg,
+        ))
+        .cloned()
+        .expect("挂载落到的那一版是录的池写过的一版");
+    let mut pool = PoolUnderTest {
+        parameters: recorded.pool.parameters.clone(),
+        device_bytes: recorded.pool.device_bytes,
+        devices,
+        session: Some(session_of_the_mount),
+        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
+        content_of_the_current_version: content_of_the_selected_version,
+        content_of_each_root: recorded.pool.content_of_each_root.clone(),
+    };
+    let published = pool
+        .overwrite(OVERWRITE_BYTES)
+        .expect("挂上之后第一次覆盖写落得下");
+    assert_eq!(
+        published.floor_raises.len(),
+        0,
+        "挂上之后第一次覆盖写准入够、不推抬 F（共用的数据单元留给卸载那一串回收）"
+    );
+    let txg_of_the_overwrite = pool.session().current.root().checkpoint_txg;
+    let shared_not_released_by_the_overwrite: Vec<&PlacementOnDevice> =
+        shared_with_the_previous_version
+            .iter()
+            .filter(|shared| {
+                !pool.session().allocator.records().iter().any(|record| {
+                    record.device == shared.device
+                        && record.slot == shared.slot
+                        && record.is_released
+                        && record.generation == txg_of_the_overwrite
+                })
+            })
+            .collect();
+    assert!(
+        shared_not_released_by_the_overwrite.is_empty(),
+        "这次覆盖写释放了 R 与 P 共用的数据单元、释放代是它的 txg {txg_of_the_overwrite:?}（阳性对照：释放是这次挂载里会话自己做的），没这样记着的 {shared_not_released_by_the_overwrite:?}"
+    );
+    let released_immediately_up_to =
+        smallest_among_the_newest_root_txg_on_each_device(&pool.image(), recorded.key);
+    assert!(
+        txg_of_the_overwrite > released_immediately_up_to,
+        "释放代 {txg_of_the_overwrite:?} 高于各盘最新根取最小 {released_immediately_up_to:?}：卸载那一串回收它们落在扣到生效那一截（阳性对照：走到要钉的那一截，不在当场放开那一截）"
+    );
+    let mut session = pool.session.take().expect("挂着的会话");
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut session.allocator,
+        &mut session.current,
+        ShadowLedger::On,
+    );
+    let raised = match unmounted {
+        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => raised,
+        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
+            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
+        }
+        Err(refusal) => panic!("挂上之后覆盖写一次再卸载做得成：{refusal:?}"),
+    };
+    assert_eq!(
+        raised.rollback_floor, txg_of_the_overwrite,
+        "卸载把 F 抬到现行那一版（这次覆盖写）的 txg：回收门槛够得着它释放的槽"
+    );
+    let shared_not_reclaimed: Vec<&PlacementOnDevice> = shared_with_the_previous_version
+        .iter()
+        .filter(|shared| {
+            !raised
+                .reclaimed
+                .iter()
+                .any(|reclaimed| reclaimed.slot == shared.slot)
+        })
+        .collect();
+    assert!(
+        shared_not_reclaimed.is_empty(),
+        "卸载那一串回收了这次覆盖写释放的共用数据单元（阳性对照：回收时补隔离那一处走到了），没回收的 {shared_not_reclaimed:?}，回收的 {:?}",
+        raised.reclaimed
+    );
+    assert!(
+        root_is_in_the_ring(&pool.image(), recorded.key),
+        "卸载之后 R 还在根环里（卸载那一串没写到 R 那一槽）"
+    );
+    let shared_free_after_the_unmount =
+        free_slots_covered_by(&session.allocator, &shared_with_the_previous_version);
+    assert!(
+        shared_free_after_the_unmount.is_empty(),
+        "卸载之后 R 还在根环里：扣到生效那一截回收的共用数据单元覆盖的槽一槽都不空闲（回收时照 C 那一圈补了隔离，卸载末尾放开扣住位之后照旧挡着），空闲的（盘、槽）{shared_free_after_the_unmount:?}"
+    );
+    assert_eq!(
+        isolated_slots_covered_by(&session.allocator, &shared_with_the_previous_version),
+        slots_covered_by(&shared_with_the_previous_version),
+        "卸载之后共用数据单元覆盖的槽都隔离着"
+    );
+    let named_by_the_unapplied_records_after_the_unmount = session
+        .allocator
+        .root_ring_occupancy()
+        .expect("卸载之后分配器照旧装着根环表")
+        .placements_named_by_unapplied_journal_records();
+    let free_among_the_named = free_slots_covered_by(
+        &session.allocator,
+        &named_by_the_unapplied_records_after_the_unmount,
+    );
+    assert!(
+        free_among_the_named.is_empty(),
+        "卸载之后 R 还在根环里：C 那一圈罩着的槽一个都不空闲，空闲的（盘、槽）{free_among_the_named:?}"
+    );
+    assert_eq!(
+        slots_counted_both_as_allocated_and_as_isolated(&session.allocator),
+        0,
+        "卸载之后没有一个槽同时算进「已分配」与第九项"
+    );
+}
+
 /// 第 10z 批第十四块实现员报告第六节 384 槽盘卡死的几形（调查报告 `~/.local/share/singlefs/work/m3-r1/investigate-10z-p/report.md` 第五节 22 形里取的）：
 /// R 是哪一种、录在几次覆盖写之后，挂载那一刻读回全零的是哪几段（挂上就撤）。
 #[derive(Clone, Copy, Debug)]
````

### 第十九块 mutations-append.tsv

````
第10z批第十九块 K2 卸载那一格扣到生效那一截回收之后不补隔离（改回只回收）	crates/singlefs-core/src/mount.rs	            reclaimed.extend(reclaim_released_in_the_sequence_raising_the_floor_up_to(\n                allocator,\n                shadow_ledger,\n                floor_of_the_reclaim,\n                ReclaimedReuse::HeldUntilFloorTakesEffect,\n            ));	            reclaimed.extend(\n                allocator.reclaim_released_up_to(floor_of_the_reclaim, ReclaimedReuse::HeldUntilFloorTakesEffect),\n            );	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_one_overwrite_isolates_the_shared_units_it_reclaims_held_until_the_floor_takes_effect_while_the_account_of_r_reads_back_zero
````

