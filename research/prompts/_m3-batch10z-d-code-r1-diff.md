# 根环实现第 10z 批第四块：代码轮第一轮改动全文（2026-10-08）

实现员交的 `crates.patch` 原样，打在删除线第 42、43 批合后的主工作区上（第四块先、第五块后）。变异表追加、替换附在后面。

````diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index bc5914f..2d62a04 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -382,7 +382,9 @@ pub enum StillUnreadableAfterOneReread {
     /// 判候选、算 F_生效 读根环：`ring_slot` 是这个进程知道住着一条根的槽（挂载那一刻读得出、或这个进程写过且 FUA 返回过），这一次读坏
     /// （`first_reading`：读不出或自证不过），重读一次仍坏（`reread`）——那一槽里的根在不在判不了，不按有根或没根猜（D16（发布语义） 已定项 1
     /// 「根槽这一次读坏」那一行全句，用户 2026-09-26 定）。挂载那一刻就读坏、之后没写过的槽不走这一条，当没有根；重建分配器两遍里只有一遍
-    /// 自证过的槽在抬 F 那一遍也不走这一条（拿重建那一遍读出、记在根环表里的那条根顶上，用户 2026-10-06 定 T1 甲），管理员回退那一遍照走（同日定 T2：照今天拒）。
+    /// 自证过的槽在抬 F 那一遍也不走这一条（拿重建那一遍读出、记在根环表里的那条根顶上，用户 2026-10-06 定 T1 甲），管理员回退那一遍照走（同日定 T2：照今天拒）；
+    /// 根环表里记成被抛弃根的槽在抬 F 那一串（准入算上限、抬 F 重算影子账与回收门槛，卸载同一串）也不走这一条（照表里记着的落点进影子账、
+    /// 不进上限，用户 2026-10-08 定 H1「被抛弃根的槽不拒」），管理员回退那一遍照走。
     /// 实审 A3b 报告 Q4：与乙那一族同级，改之前是 `RecoveryFailure::RootRingSlotStillUnreadableAfterOneReread`（那时不分槽知不知道住着根）。
     RootRingSlotKnownToHoldARoot {
         ring_slot: RootRingSlot,
@@ -1483,9 +1485,11 @@ enum PlacementsOfAbandonedRoots<'recorded> {
     /// （「崩了再可写挂载」那一道照 B 拒，不变）。
     ReadFromTheirAccounts,
     /// 抬 F 那一串（准入与卸载）：按 (实例代号, txg) 认、根环表里记着同一条被抛弃根的（`allocator::RootRingOccupancy::placements_recorded_for_the_abandoned_root`），
-    /// 照挂载时算出、记着的那一串落点算，不再从盘上读它的账，读不出也不拒。表里没记着这条根的（挂载之后才按现行那一版的实例表判成被抛弃的根——
-    /// 挂载那一刻它按最新那条根的表判有效、记成有效根；挂载那一刻读坏、读回全零的槽这一遍读出的被抛弃根；进程内重读读出被抛弃根、已从表里拿掉的槽）
-    /// 条款没写它们取什么，照可写挂载那一遍读盘，交主 agent（第 10z 批第二块报告）。
+    /// 照挂载时算出、记着的那一串落点算，不再从盘上读它的账，读不出也不拒。挂载之后才按现行那一版的实例表判成被抛弃的根（挂载那一刻它按最新那条根的表
+    /// 判有效、记成有效根）借 C 那一圈在那一槽记着的落点（那一槽记的是有效根且 txg 相同，
+    /// `allocator::RootRingOccupancy::placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot`，用户 2026-10-08 定 H2），
+    /// 同样不读账、读不出不拒。两样都没记着的（C 在那一槽也没记的；挂载那一刻读坏、读回全零的槽这一遍读出的被抛弃根；
+    /// 进程内重读读出被抛弃根、已从表里拿掉的槽）照可写挂载那一遍读盘（「C 也没记着的照今天读盘」）。
     RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
         &'recorded BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<PlacementOnDevice>>,
     ),
@@ -1533,10 +1537,14 @@ impl PlacementsOfAbandonedRoots<'_> {
 /// （`PoolAllocator::record_root_written_by_this_process`，按交回的每条根引用的落点与分配器根环表记着的那一份）。
 /// 被抛弃根引用的落点从哪来由 `placements_of_abandoned_roots` 定（[`PlacementsOfAbandonedRoots`]）：可写挂载那一遍读盘，
 /// 抬 F 那一串取根环表里记着的那一份，表里没记着这条根的才读盘（上面那一串读不出的处置只走在这一支上）。
+/// `placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad` 是抬 F 那一遍根槽读坏、没读出来的被抛弃根挂载时记在根环表里的落点
+/// （D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定 H1「被抛弃根的槽不拒」，[`readable_roots_for_raising_the_floor`]）：
+/// 与读出来的被抛弃根同一种豁免（候选集里的根引用的、当前账里还分配着的不隔离），不计进 `abandoned_roots_unreadable`（它们的账一个字都没读）；
+/// 可写挂载那一遍给空。
 #[allow(
     clippy::ptr_arg,
     clippy::too_many_arguments,
-    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、水位之上没施加的记录点名的落点、被抛弃根的落点从哪来、重读前的快照"
+    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、水位之上没施加的记录点名的落点、被抛弃根的落点从哪来、重读前的快照、根槽这一遍读坏的被抛弃根记着的落点"
 )]
 fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
@@ -1549,6 +1557,7 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     placements_named_by_unapplied_journal_records: &[PlacementOnDevice],
     placements_of_abandoned_roots: PlacementsOfAbandonedRoots<'_>,
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
+    placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[PlacementOnDevice],
 ) -> Result<ShadowLedgerComputed, MountError> {
     let referenced_by_the_current_account: BTreeSet<(u32, u64)> = current_records
         .iter()
@@ -1611,6 +1620,14 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
                 .collect(),
         );
     }
+    // 根槽这一遍读坏的被抛弃根（H1）：读不出它的根记录，照它挂载时记着的落点补隔离，豁免与读出来的被抛弃根同一个集合。
+    for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad {
+        let key = (placement.device.0, placement.slot.0);
+        if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
+            continue;
+        }
+        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);
+    }
     // C 那一份不豁免候选根引用的槽（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K1 / K2 T1 C 的候选豁免」那一行，
     // 根环线 K4-1）：水位之上没施加的那次发布的根读得出、账读得出时它自己就是候选根（按实例表有效、txg ≥ F），豁免它引用的槽就是不护它的单元——
     // 它比所选那一版新，它新分配的单元不在现行账里、回收门槛也罩不到，挂载之后当空闲槽发出去。只豁免现行账里仍分配着的槽：它们这时发不出去，
@@ -2689,6 +2706,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 .collect::<Vec<PlacementOnDevice>>(),
             PlacementsOfAbandonedRoots::ReadFromTheirAccounts,
             before_the_one_reread,
+            &[],
         )?,
         ShadowLedger::Off => ShadowLedgerComputed {
             abandoned_roots_unreadable: 0,
@@ -3226,7 +3244,8 @@ pub struct RaisedFloor {
     /// 生效之后回收的落点（两盘同槽，按盘 0 报）。
     pub reclaimed: Vec<Placement>,
     /// 抬 F 重算影子账时读不出账、被跳过的被抛弃根的条数（与 `MountOutput::abandoned_roots_unreadable` 同一个量；
-    /// 代码三方第三轮云端攻方腿打中：抬 F 那一处此前把计数丢掉了）。根环表里记着落点的被抛弃根这一遍不读盘，不计在内。
+    /// 代码三方第三轮云端攻方腿打中：抬 F 那一处此前把计数丢掉了）。根环表里记着落点的被抛弃根、借 C 那一圈落点的被抛弃根与根槽这一遍读坏的
+    /// 被抛弃根这一遍都不读盘，不计在内。
     pub abandoned_roots_unreadable: u64,
 }
 
@@ -3487,8 +3506,24 @@ pub fn ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passe
     )
 }
 
+/// 这个进程分配器上那张根环表里记成被抛弃根的槽与那条根挂载时记着的落点
+/// （`allocator::RootRingOccupancy::ring_slots_recorded_as_holding_an_abandoned_root`）；没装根环表的分配器给空表。
+/// 抬 F 那一串（准入算上限、抬 F 重算影子账与回收门槛，卸载同一串）读它们读坏、重读仍坏不拒：上限本来不看被抛弃根，影子账照这里记着的落点算
+/// （用户 2026-10-08 定 H1「被抛弃根的槽不拒」）。
+#[must_use]
+pub fn ring_slots_recorded_as_holding_an_abandoned_root_by(
+    allocator: &PoolAllocator,
+) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
+    allocator.root_ring_occupancy().map_or_else(
+        BTreeMap::new,
+        RootRingOccupancy::ring_slots_recorded_as_holding_an_abandoned_root,
+    )
+}
+
 /// 这个进程的准入抬 F 的上限（[`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]），三类槽都取这个进程分配器上那张根环表：
 /// 知道住着根的槽、挂载那一刻读坏的槽、重建分配器两遍里只有一遍自证过的槽（后一类读坏重读仍坏时拿记着的那条根顶上，用户 2026-10-06 定 T1 甲）。
+/// 知道住着根的槽里记成被抛弃根的那几槽不算「重读仍坏就拒」的槽（用户 2026-10-08 定 H1「被抛弃根的槽不拒」）：这一遍读坏就当没有根，
+/// 上限只在有效根上取、本来不看它们；它们也不是挂载那一刻读坏的槽，不并进上限。
 ///
 /// # Errors
 /// 同 [`rollback_floor_ceiling`]。
@@ -3503,12 +3538,22 @@ fn rollback_floor_ceiling_of_this_process<Device: BlockDevice>(
     table: &InstanceTableRecords,
     allocator: &PoolAllocator,
 ) -> Result<CheckpointTxg, MountError> {
+    let ring_slots_recorded_as_holding_an_abandoned_root =
+        ring_slots_recorded_as_holding_an_abandoned_root_by(allocator);
+    let ring_slots_known_to_hold_a_root_whose_reread_still_bad_refuses_the_ceiling: BTreeSet<
+        RootRingSlot,
+    > = ring_slots_known_to_hold_a_root_by(allocator)
+        .into_iter()
+        .filter(|ring_slot| {
+            !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)
+        })
+        .collect();
     rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
         devices,
         system_configuration,
         current_floor,
         table,
-        &ring_slots_known_to_hold_a_root_by(allocator),
+        &ring_slots_known_to_hold_a_root_whose_reread_still_bad_refuses_the_ceiling,
         &ring_slots_bad_at_mount_by(allocator),
         &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator),
     )
@@ -3748,19 +3793,24 @@ pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
 /// 这个进程不知道住着根的当没有根；知道住着根的重读一次，仍坏就拒（C554 乙报告 Q6）；重建分配器两遍里只有一遍自证过的
 /// （[`ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by`]，它们也在「知道住着根的槽」里，这里先按它们分）重读一次，
 /// 读出来照读出来的算，仍坏不拒、拿重建那一遍读出、记在根环表里的那条根顶上（用户 2026-10-06 定 T1 甲：有效的照它本身进回收门槛与候选集，
-/// 被抛弃的照它引用的落点进影子账；有效还是被抛弃由调用方按同一张实例表判）。交回读得出（或顶上）的每一条根（重读读出来的与顶上的排在后面）。
-/// 只读盘。
+/// 被抛弃的照它引用的落点进影子账；有效还是被抛弃由调用方按同一张实例表判）。根环表里记成被抛弃根的槽（它们也在「知道住着根的槽」里，
+/// 这里同样先按它们分，[`ring_slots_recorded_as_holding_an_abandoned_root_by`]）这一遍读坏不拒、不重读：读不出它的根记录，交回它挂载时
+/// 记在根环表里的落点，由影子账照它补隔离、不进回收门槛（用户 2026-10-08 定 H1「被抛弃根的槽不拒」）。重读读得出也是同一条被抛弃根、
+/// 影子账同样取表里记着的那一份（[`PlacementsOfAbandonedRoots`]），所以不重读结局相同。交回读得出（或顶上）的每一条根（重读读出来的与
+/// 顶上的排在后面）与那几槽记着的落点。只读盘。
 ///
 /// # Errors
-/// 知道住着根的槽（两遍里只有一遍自证过的除外）重读仍坏 ⇒
+/// 知道住着根的槽（两遍里只有一遍自证过的、记成被抛弃根的除外）重读仍坏 ⇒
 /// `NewerStateStillUnreadableAfterOneReread(StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot(..))`。
 fn readable_roots_for_raising_the_floor<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     system_configuration: &crate::system_configuration::SystemConfiguration,
     allocator: &PoolAllocator,
-) -> Result<Vec<RootRecord>, MountError> {
+) -> Result<RootsReadForRaisingTheFloor, MountError> {
     let ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass =
         ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator);
+    let ring_slots_recorded_as_holding_an_abandoned_root =
+        ring_slots_recorded_as_holding_an_abandoned_root_by(allocator);
     let ring_slots_whose_reread_still_bad_refuses_the_raise: BTreeSet<RootRingSlot> =
         ring_slots_known_to_hold_a_root_by(allocator)
             .into_iter()
@@ -3768,6 +3818,9 @@ fn readable_roots_for_raising_the_floor<Reader: PoolReader + ?Sized>(
                 !ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass
                     .contains_key(ring_slot)
             })
+            .filter(|ring_slot| {
+                !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)
+            })
             .collect();
     let mut roots_with_ring_slots =
         readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once(
@@ -3805,10 +3858,27 @@ fn readable_roots_for_raising_the_floor<Reader: PoolReader + ?Sized>(
                 .push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),
         }
     }
-    Ok(roots_with_ring_slots
-        .into_iter()
-        .map(|(_, root)| root)
-        .collect())
+    let placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: Vec<PlacementOnDevice> =
+        ring_slots_recorded_as_holding_an_abandoned_root
+            .into_iter()
+            .filter(|(ring_slot, _)| !ring_slots_read.contains(ring_slot))
+            .flat_map(|(_, recorded_placements)| recorded_placements)
+            .collect();
+    Ok(RootsReadForRaisingTheFloor {
+        roots: roots_with_ring_slots
+            .into_iter()
+            .map(|(_, root)| root)
+            .collect(),
+        placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
+    })
+}
+
+/// 抬 F 那一遍读根环读出来的（[`readable_roots_for_raising_the_floor`]）。
+struct RootsReadForRaisingTheFloor {
+    /// 读得出、自证过的每一条根，与两遍里只有一遍自证过、这一遍重读仍坏的槽记着的那条根。
+    roots: Vec<RootRecord>,
+    /// 根环表里记成被抛弃根、这一遍读坏的槽各记着的落点（摊平）：影子账照它补隔离（H1）。
+    placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: Vec<PlacementOnDevice>,
 }
 
 /// 抬 F 那一遍的回收门槛里「环里最旧有效根」（D16（发布语义） 已定项 1，用户 2026-10-04 定 RGall：不按一圈窗口滤）：这一遍读出来的根里
@@ -3965,7 +4035,11 @@ fn raise_the_floor_through<Device: BlockDevice>(
     // 拿重建那一遍读出、记在根环表里的那条根顶上（用户 2026-10-06 定 T1 甲，[`readable_roots_for_raising_the_floor`]）：被抛弃的照它引用的落点
     // 进下面的影子账重算（不顶上时它引用、挂载那一刻被候选根豁免的槽按新 F 回收之后发得出去，判决第二节 K1 攻方 K1-1，I-7.4 红），
     // 有效的照它本身进回收门槛与候选集。在动分配器（影子账重算、回收）之前返回，分配器与盘都不动。
-    let roots = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;
+    // 根环表里记成被抛弃根的槽这一遍读坏不拒（用户 2026-10-08 定 H1）：交回它们记着的落点，下面的影子账照它补隔离。
+    let RootsReadForRaisingTheFloor {
+        roots,
+        placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
+    } = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;
     // 根环表里挂载那一刻读坏、这一遍也没读出根的槽，钉在它可能住过的最旧 txg 并进环里最旧有效根（RR4，攻方 B1；用户 2026-10-04 定钉在最旧 txg；与进程内记根同一取法，
     // E2 的重读照用：它成了最小的那个时先重读，`oldest_valid_root_in_the_ring_for_raising_the_floor`）。
     let oldest_valid_root_in_the_ring = oldest_valid_root_in_the_ring_for_raising_the_floor(
@@ -3986,7 +4060,9 @@ fn raise_the_floor_through<Device: BlockDevice>(
     // `research/prompts/m3-batch10y-code-r1-main-verification.md` 第三节）：这一遍读出的、按现行那一版的实例表被抛弃的根，根环表里那一槽记着的
     // 是同一条（实例代号与 txg 都对上）就照挂载时算出、记着的落点进影子账，不再从盘上读它的账、读不出也不拒（不走 C694 的 B）。
     // 改之前照盘上重读，挂上之后它的账读坏就照 B 拒，正常卸载与会话写都被拒（攻方 `research/prompts/m3-batch10y-code-r1-opus-output.md` 「K4-2」）。
-    // 表里没记着的照旧读盘（[`PlacementsOfAbandonedRoots`] 那一臂的注释）。
+    // 表里没记着这条被抛弃根、挂载那一刻那一槽记成同一条有效根（写行之后才被抛弃）的，借 C 那一圈在那一槽记着的落点
+    // （用户 2026-10-08 定 H2「用 C 那一圈记着的落点」，判决 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第三节）：
+    // 同样不读账、读不出不拒。两样都没记着的照旧读盘（[`PlacementsOfAbandonedRoots`] 那一臂的注释）。
     let placements_recorded_for_abandoned_roots: BTreeMap<
         (InstanceGeneration, CheckpointTxg),
         Vec<PlacementOnDevice>,
@@ -4002,6 +4078,13 @@ fn raise_the_floor_through<Device: BlockDevice>(
                             root.instance,
                             root.checkpoint_txg,
                         )
+                        .or_else(|| {
+                            occupancy
+                                .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+                                    root.instance,
+                                    root.checkpoint_txg,
+                                )
+                        })
                         .map(|recorded| ((root.instance, root.checkpoint_txg), recorded))
                 })
                 .collect()
@@ -4021,6 +4104,7 @@ fn raise_the_floor_through<Device: BlockDevice>(
                 &placements_recorded_for_abandoned_roots,
             ),
             &mut BeforeTheOneReread::RereadImmediately,
+            &placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
         ) {
             Ok(computed) => computed.abandoned_roots_unreadable,
             Err(error) => {
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 1f8fd62..f3d6d71 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -1143,6 +1143,77 @@ impl RootRingOccupancy {
         }
     }
 
+    /// 抬 F 那一串（准入与卸载）重算影子账时，一条根环表里没记着落点的被抛弃根借 C 那一圈记着的落点（D16（发布语义） 已定项 1「根槽这一次读坏」
+    /// 那一行，用户 2026-10-08 定 H2「用 C 那一圈记着的落点」，判决 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第三节）：
+    /// 可写挂载自己写行之后才按现行那一版的实例表判成被抛弃的根，挂载那一刻按最新那条根的表判有效、记成有效根，不带落点。
+    /// 这条根（`instance`、`checkpoint_txg`）的 txg 在根环里对应的那一槽上，表里记着的是一条有效根、txg 与它相同（实例代号记着的就也比：
+    /// 两遍里只有一遍自证过的有效那一半带整条根记录），而 C 那一圈（水位之上没施加的记录点名的落点、那几次发布的账、比所选那一版新的可读根的账，
+    /// [`RootRingOccupancy`] 那一项）在那一槽还记着、没解开，就交回那一槽记着的落点；那一槽记的是被抛弃根、挂载时读坏或读回全零的槽、
+    /// 有效根而 txg 不同（同一槽差一整圈的另一次发布），或 C 在那一槽什么都没记（这个进程写掉那一槽时解开了，或挂载时就没记），交 `None`，
+    /// 调用方照今天读盘。只读内存里的表，不读盘。
+    #[must_use]
+    pub fn placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+        &self,
+        instance: InstanceGeneration,
+        checkpoint_txg: CheckpointTxg,
+    ) -> Option<Vec<PlacementOnDevice>> {
+        let ring_slot = target_for_publish(checkpoint_txg, self.slots_per_region);
+        let is_the_same_root_recorded_valid = match self.occupants.get(&ring_slot)? {
+            RootRingOccupant::ValidRoot {
+                checkpoint_txg: recorded_checkpoint_txg,
+            }
+            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
+                checkpoint_txg: recorded_checkpoint_txg,
+            } => *recorded_checkpoint_txg == checkpoint_txg,
+            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                root: RootReadOnOneRebuildPass::ValidRoot { .. },
+                root_record_self_verified_on_one_rebuild_pass,
+            } => {
+                (
+                    root_record_self_verified_on_one_rebuild_pass.instance,
+                    root_record_self_verified_on_one_rebuild_pass.checkpoint_txg,
+                ) == (instance, checkpoint_txg)
+            }
+            RootRingOccupant::AbandonedRoot { .. }
+            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
+                ..
+            }
+            | RootRingOccupant::BadAtMount(_)
+            | RootRingOccupant::AllZeroAtMount { .. } => false,
+        };
+        if !is_the_same_root_recorded_valid {
+            return None;
+        }
+        self.placements_named_by_unapplied_journal_records
+            .get(&ring_slot)
+            .cloned()
+    }
+
+    /// 根环表里记成被抛弃根的槽（[`RootRingOccupant::AbandonedRoot`]）与那条根挂载时记着的落点，按槽的次序：抬 F 那一串读根环时这几槽读坏、
+    /// 重读仍坏不拒，照这里记着的落点进影子账、不进上限（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定 H1
+    /// 「被抛弃根的槽不拒」）。两遍里只有一遍自证过的槽不在其内：抬 F 那一遍拿它记着的整条根记录顶上（用户 2026-10-06 定 T1 甲）。
+    /// 这个进程写掉那一槽之后就不在其内。
+    #[must_use]
+    pub fn ring_slots_recorded_as_holding_an_abandoned_root(
+        &self,
+    ) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
+        self.occupants
+            .iter()
+            .filter_map(|(ring_slot, occupant)| match occupant {
+                RootRingOccupant::AbandonedRoot {
+                    referenced_placements,
+                    ..
+                } => Some((*ring_slot, referenced_placements.clone())),
+                RootRingOccupant::ValidRoot { .. }
+                | RootRingOccupant::BadAtMount(_)
+                | RootRingOccupant::AllZeroAtMount { .. }
+                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
+                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
+            })
+            .collect()
+    }
+
     /// 环里还在的被抛弃根引用着的全部落点，加还没解开的、水位之上没施加的记录点名的全部落点：清隔离位之前要看的那一份
     /// （一个槽两边任一边还罩着就不清），回收出来的槽里它们罩着的补隔离。
     fn placements_the_isolation_still_protects(&self) -> Vec<PlacementOnDevice> {
@@ -3125,6 +3196,206 @@ mod tests {
         }
     }
 
+    /// 实例 2、txg `checkpoint_txg` 的一条根记录（只有 (实例, txg) 有用，指针都是空的）：两遍里只有一遍自证过的槽记着的那整条根。
+    fn root_record_of_instance_2_at(checkpoint_txg: CheckpointTxg) -> RootRecord {
+        RootRecord {
+            filesystem_identifier: [5u8; 16],
+            unmount_marker: crate::root_record::UnmountMarker::NotWrittenByTheUnmountSequence,
+            instance: InstanceGeneration(2),
+            checkpoint_txg,
+            tree_table: crate::pointer::NodePointer::empty_root(),
+            tree_identifier_watermark: singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+            rollback_floor: CheckpointTxg(0),
+            instance_table: crate::pointer::NodePointer::empty_root(),
+            mapping_root: crate::pointer::NodePointer::empty_root(),
+            allocation_record_tree_root: crate::pointer::NodePointer::empty_root(),
+        }
+    }
+
+    /// 24 槽的环：txg 11 那一槽挂载时读坏，12 读回全零，13 挂载时读坏、进程内重读读出有效根，14 两遍里只有一遍自证过、那一遍读出的是有效根
+    /// （实例 2），15 是有效根，16 是实例 2 的被抛弃根；C 那一圈在 11 到 16 每一槽都记着一个落点（槽号 50200 + txg）。
+    fn occupancy_with_every_kind_of_slot_from_txg_11_through_16_and_unapplied_records_on_each(
+    ) -> RootRingOccupancy {
+        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
+        let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
+        let mut occupancy = RootRingOccupancy::read_from_the_ring(
+            slots_per_region,
+            vec![
+                (
+                    ring_slot_of(11),
+                    RootRingOccupant::BadAtMount(TxgsARingSlotBadAtMountCanHold::new(
+                        CheckpointTxg(11),
+                        CheckpointTxg(11),
+                    )),
+                ),
+                (
+                    ring_slot_of(12),
+                    RootRingOccupant::AllZeroAtMount {
+                        oldest_txg_it_can_have_held: CheckpointTxg(12),
+                    },
+                ),
+                (
+                    ring_slot_of(13),
+                    RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
+                        checkpoint_txg: CheckpointTxg(13),
+                    },
+                ),
+                (
+                    ring_slot_of(14),
+                    RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                        root: RootReadOnOneRebuildPass::ValidRoot {
+                            checkpoint_txg: CheckpointTxg(14),
+                        },
+                        root_record_self_verified_on_one_rebuild_pass: Box::new(
+                            root_record_of_instance_2_at(CheckpointTxg(14)),
+                        ),
+                    },
+                ),
+                (
+                    ring_slot_of(15),
+                    RootRingOccupant::ValidRoot {
+                        checkpoint_txg: CheckpointTxg(15),
+                    },
+                ),
+                (
+                    ring_slot_of(16),
+                    RootRingOccupant::AbandonedRoot {
+                        instance: InstanceGeneration(2),
+                        checkpoint_txg: CheckpointTxg(16),
+                        referenced_placements: Vec::new(),
+                    },
+                ),
+            ],
+            CheckpointTxg(0),
+            CheckpointTxg(16),
+        );
+        occupancy.placements_named_by_unapplied_journal_records = (11..=16)
+            .map(|txg| {
+                (
+                    ring_slot_of(txg),
+                    vec![PlacementOnDevice {
+                        device: DeviceIdentity(0),
+                        slot: SlotNumber(50200 + txg),
+                        span: 1,
+                    }],
+                )
+            })
+            .collect();
+        occupancy
+    }
+
+    /// 抬 F 那一串取被抛弃根记着的落点（[`RootRingOccupancy::placements_recorded_for_the_abandoned_root`]）只认记成被抛弃根的槽：
+    /// 挂载时读坏、读回全零、读坏后进程内重读读出有效根、两遍里只有一遍自证过而那一遍是有效根这四类槽，这一遍读出一条被抛弃根时交 `None`
+    /// （照今天读盘），不交一串空落点——交空落点就是「这条根什么都不引用」，不隔离也不读盘，它的单元掉出影子账（正推 m5、m6、m8、m20）。
+    #[test]
+    fn placements_recorded_for_an_abandoned_root_answer_none_on_slots_recorded_bad_all_zero_or_valid_at_mount(
+    ) {
+        let occupancy =
+            occupancy_with_every_kind_of_slot_from_txg_11_through_16_and_unapplied_records_on_each(
+            );
+        for (txg, kind_of_slot) in [
+            (11, "挂载时读坏"),
+            (12, "挂载时读回全零"),
+            (13, "挂载时读坏、进程内重读读出有效根"),
+            (14, "两遍里只有一遍自证过、那一遍是有效根"),
+            (15, "有效根"),
+        ] {
+            assert_eq!(
+                occupancy.placements_recorded_for_the_abandoned_root(
+                    InstanceGeneration(2),
+                    CheckpointTxg(txg)
+                ),
+                None,
+                "txg {txg} 那一槽记的是「{kind_of_slot}」：没记着这条被抛弃根的落点，交 None 照今天读盘，不交空落点"
+            );
+        }
+    }
+
+    /// 根环表里没记着落点的被抛弃根借 C 那一圈记着的落点（用户 2026-10-08 定 H2）：认同条件不只按槽号——那一槽记的是有效根且 txg 相同
+    /// （两遍里只有一遍自证过的那一类还比实例代号）。同一条根交回 C 在那一槽记着的落点；同一槽上 txg 差一整圈的另一次发布、实例代号不同、
+    /// 那一槽记的是被抛弃根或挂载时读坏、读回全零的槽，交 `None`；C 在那一槽解开之后（这个进程写掉了那一槽）也交 `None`。
+    #[test]
+    fn placements_named_by_unapplied_journal_records_for_a_root_recorded_valid_answer_only_for_the_same_root_on_its_ring_slot(
+    ) {
+        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
+        let ring_length = 3 * slots_per_region.count();
+        let mut occupancy =
+            occupancy_with_every_kind_of_slot_from_txg_11_through_16_and_unapplied_records_on_each(
+            );
+        for txg in [13, 14, 15] {
+            assert_eq!(
+                occupancy.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+                    InstanceGeneration(2),
+                    CheckpointTxg(txg)
+                ),
+                Some(vec![PlacementOnDevice {
+                    device: DeviceIdentity(0),
+                    slot: SlotNumber(50200 + txg),
+                    span: 1,
+                }]),
+                "txg {txg} 那一槽记的是同一条有效根：交回 C 在那一槽记着的落点"
+            );
+        }
+        for (instance, checkpoint_txg, why) in [
+            (
+                InstanceGeneration(2),
+                CheckpointTxg(15 + ring_length),
+                "同一槽差一整圈的另一次发布",
+            ),
+            (
+                InstanceGeneration(2),
+                CheckpointTxg(13 + ring_length),
+                "同一槽差一整圈的另一次发布（进程内重读读出的有效根那一槽）",
+            ),
+            (
+                InstanceGeneration(2),
+                CheckpointTxg(14 + ring_length),
+                "同一槽差一整圈的另一次发布（两遍里只有一遍自证过那一槽）",
+            ),
+            (
+                InstanceGeneration(3),
+                CheckpointTxg(14),
+                "实例代号与那一遍读出的整条根记录不同",
+            ),
+            (InstanceGeneration(2), CheckpointTxg(11), "那一槽挂载时读坏"),
+            (
+                InstanceGeneration(2),
+                CheckpointTxg(12),
+                "那一槽挂载时读回全零",
+            ),
+            (
+                InstanceGeneration(2),
+                CheckpointTxg(16),
+                "那一槽记的是被抛弃根（它记着的落点归另一个方法）",
+            ),
+            (
+                InstanceGeneration(2),
+                CheckpointTxg(17),
+                "那一槽表里什么都没记",
+            ),
+        ] {
+            assert_eq!(
+                occupancy.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+                    instance,
+                    checkpoint_txg
+                ),
+                None,
+                "({instance:?}, {checkpoint_txg:?})：{why}，不借 C 的落点"
+            );
+        }
+        occupancy
+            .placements_named_by_unapplied_journal_records
+            .remove(&target_for_publish(CheckpointTxg(15), slots_per_region));
+        assert_eq!(
+            occupancy.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+                InstanceGeneration(2),
+                CheckpointTxg(15)
+            ),
+            None,
+            "C 在那一槽解开之后没有可借的落点：交 None，调用方照今天读盘"
+        );
+    }
+
     /// 环上有洞留下的旧根压住进程内的回收门槛（D16（发布语义） 已定项 1「环里最旧有效根」，用户 2026-10-04 定 RGall）：24 槽的环里
     /// txg 1 那一槽留着 txg 1 的根（txg 25、49 那两次的根槽都没落），其余 23 槽住着 txg 26–48；这个进程记到 txg 49（没有根的那一个）
     /// 之后再记 txg 50，盖掉的是 txg 26。txg 1 离最新根已过两圈，仍是环里最旧的有效根：释放代 2 的落点不回收、仍在 defer 里。
diff --git a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
index d45d2e7..2424f8a 100644
--- a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
+++ b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
@@ -22848,8 +22848,14 @@ mod tests {
 
     /// op1 第三种变体：探测出的 floor 目标落在 [F+1, 上限] 内时，不注入任何故障应当成功
     /// （`error_member` 为 `None`）；换成对着某条被抛弃可读根的分配记录树节点注入故障（两份都读失败），
-    /// `raise_rollback_floor` 必须做成、且它交回的 `RaisedFloor::abandoned_roots_unreadable` > 0——证明抬 F 那一步的影子账真的读到了
-    /// 这组故障（不是被前面那一步「先不注入的 mount_writable」悄悄吸收掉，也不是抬 F 算上限那一步先拒了）。
+    /// `raise_rollback_floor` 必须做成。那条被抛弃根在先不注入的 `mount_writable` 那一刻读得出，根环表把它记成被抛弃根、带着它引用的落点；
+    /// 抬 F 那一串照记着的落点算影子账、不再从盘上读它的账（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，根环线第 10z 批），
+    /// 所以交回的 `RaisedFloor::abandoned_roots_unreadable` 是 0（注入时与不注入时都是），抬 F 之后它记着的落点一槽都不空闲
+    /// （隔离着，或现行账里还分配着）——照记着的落点隔离，没被注入的读故障挡住，也没漏掉。
+    /// 根环表里没记着、账读不出时照读盘那一支（计数 1、从映射现算）由 harness 档
+    /// `an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 的
+    /// `an_unmount_meeting_an_abandoned_root_not_in_the_root_ring_table_whose_account_is_unreadable_recomputes_it_from_the_mapping`
+    /// 与 `reuse_after_raising_the_floor.rs` 的慢档 `raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable` 钉。
     /// 被抛弃根取自第 2 次跑那一节 H1c 的抛弃步（`abandonment_step_nodes`；跑前登记 `research/prompts/e158-r2-prereg.md`
     /// 第一节处理表：故障改打 H1c 里一条被抛弃根的分配记录树节点、删掉「或者报了错也算」那一半）。
     #[test]
@@ -22907,13 +22913,74 @@ mod tests {
                 "被抛弃根的分配记录树节点两份都读失败时抬 F 照样做成（账读不出只计数），得到 {:?}",
                 with_fault.error_member
             );
+            assert_eq!(
+                (
+                    with_fault.abandoned_roots_unreadable,
+                    without_fault.abandoned_roots_unreadable
+                ),
+                (Some(0), Some(0)),
+                "这条被抛弃根挂载那一刻记在根环表里：抬 F 照记着的落点算、不读它的账，注入与不注入都不计（注入时、不注入时）"
+            );
+            let mut devices_of_the_mount = devices_from_pool(&pool, IMAGE_BYTES);
+            let mounted = mount_writable(&parameters, &mut devices_of_the_mount)
+                .expect("先不注入的可写挂载做得成（上面那一次就做成了）");
+            let recorded_placements = mounted
+                .allocator
+                .root_ring_occupancy()
+                .expect("挂载建的分配器装着根环表")
+                .placements_recorded_for_the_abandoned_root(record.instance, record.checkpoint_txg)
+                .filter(|placements| !placements.is_empty())
+                .expect("挂载那一刻这条被抛弃根读得出：根环表把它记成被抛弃根、带着它引用的落点");
+            let PoolVersion::WithFile(mut current) = mounted.current else {
+                panic!("先不注入的可写挂载交回的现行那一版带文件（上面那一次抬 F 做成了）");
+            };
+            let mut allocator = mounted.allocator;
+            let mut faulted_devices: Vec<(
+                DeviceIdentity,
+                FaultInjectingBlockDevice<SparseBlockDevice>,
+            )> = Vec::new();
+            for (identity, device) in devices_of_the_mount {
+                let plan = SharedFaultPlan::unarmed(fixed_geometry);
+                if let Some((_, offset)) = fault_targets
+                    .iter()
+                    .find(|(target_device, _)| *target_device == identity)
+                {
+                    plan.arm(FaultSchedule {
+                        fault: InjectedFault::ReadFails,
+                        device: FaultDeviceSelector::OnlyDevice(identity),
+                        placement: FaultPlacement::OffsetExactly(*offset),
+                        counting: FaultCounting::AcrossThePool,
+                        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
+                    });
+                }
+                faulted_devices.push((
+                    identity,
+                    FaultInjectingBlockDevice::new(identity, device, plan),
+                ));
+            }
+            raise_rollback_floor(
+                &parameters,
+                &mut faulted_devices,
+                &mut allocator,
+                &mut current,
+                floor,
+                ShadowLedger::On,
+            )
+            .expect("注入同一组故障再抬一次：照样做成");
+            let free_recorded_placements: Vec<_> = recorded_placements
+                .iter()
+                .filter(|placement| {
+                    allocator
+                        .devices
+                        .iter()
+                        .find(|device_map| device_map.device == placement.device)
+                        .expect("池里有这块盘")
+                        .is_free(placement.slot)
+                })
+                .collect();
             assert!(
-                with_fault
-                    .abandoned_roots_unreadable
-                    .is_some_and(|count| count > 0),
-                "抬 F 重算影子账要数到这条读不出账的被抛弃根，得到 {:?}（不注入时 {:?}）",
-                with_fault.abandoned_roots_unreadable,
-                without_fault.abandoned_roots_unreadable
+                free_recorded_placements.is_empty(),
+                "抬 F 之后这条被抛弃根记着的落点一槽都不空闲（照记着的落点隔离）：空闲的 {free_recorded_placements:?}"
             );
             tested = true;
             break;
diff --git a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
index b1a5488..caea814 100644
--- a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -8,29 +8,45 @@
 //! 之后可写挂载一次（实例 4）：挂载那一刻 C 的账读得出，根环表里把 C 记成被抛弃根、带着它引用的落点。挂上之后 C 的账那几个单元
 //! （树表、中央映射树根、分配记录树根的每条位置条目）从此读不出（这几块盘上的读报错）。
 //! 改之前抬 F 那一遍照盘上重读 C 的账，读不出、从映射现算不成、重读一次仍不成、现行账里又没有 B 的凭据，就拒：正常卸载与会话写都被拒。
-mod common_admission;
+//! 抬 F 之后断言 C 记着的落点一槽都不空闲（隔离着，或现行账里还分配着）：记着的落点被削（只交一部分或交空）时，此前靠 C 之前那一版豁免、
+//! 现行账里已释放的那几个（C 之前那一版的实例表片这类）回收出去，这一条红。
+//!
+//! 同一个池上另三形（用户 2026-10-08 定，判决 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第三节）：
+//! - H1「被抛弃根的槽不拒」：挂上之后 C 的**根槽**读报错（账读得出），抬 F 那一串（准入算上限、抬 F 重算影子账，卸载同一串）不拒，
+//!   照根环表里记着的 C 的落点进影子账、不进上限；有效根的槽读坏、重读仍坏照拒。
+//! - H2「用 C 那一圈记着的落点」：可写挂载自己写行之后才判成被抛弃的根 R（`tests/common_unwitnessed_publish/mod.rs` 那块盘，挂载那一刻
+//!   R 的账之外写出的单元读回全零、择根退到 P，R 记成有效根），挂上之后 R 的账读报错：抬 F 照 C 那一圈在 R 那一槽记着的落点算、不读账、不拒。
+//! - 挂载那一刻凭据放行的被抛弃根，根环表里记着的是空串：抬 F 照空串算、不再读它的账（攻方 M7）。
+// 池（384 槽那一档小盘）与 R 那块盘都经 `tests/common_unwitnessed_publish/mod.rs` 起（它转出 `tests/common_admission_pool/mod.rs` 的全部）：
+// 再声明 `common_admission` 就把 `common_admission_pool` 当模块装两遍（clippy `duplicate_mod`）。
+mod common_unwitnessed_publish;
 
 use std::cell::Cell;
 use std::rc::Rc;
 
-use common_admission::{
-    checker_violations_on, start_plain, DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES,
+use common_unwitnessed_publish::{
+    checker_violations_on, device_bytes_of_the_384_slot_small_devices,
+    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
+    parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on, pool_after_the_mount,
+    record_one_publish_after_overwrites, DeviceOverASparseImage, PoolUnderTest, PublishRecordedAsR,
+    RangesReadingBackAllZero, OVERWRITE_BYTES,
 };
 use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
 use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::allocator::{AllocationRecord, PlacementOnDevice, PoolAllocator};
 use singlefs_core::block_device::{
     BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
 };
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
 use singlefs_core::mount::{
-    mount_writable_with_space_admission, unmount, MountError, RollbackTarget, ShadowLedger,
-    StillUnreadableAfterOneReread, Unmounted,
+    mount_writable, mount_writable_with_space_admission, mount_writable_with_test_only_switches,
+    unmount, MountError, RollbackTarget, ShadowLedger, StillUnreadableAfterOneReread, Unmounted,
 };
 use singlefs_core::mounted_session::MountedSession;
-use singlefs_core::pointer::NodePointer;
+use singlefs_core::pointer::{LocationEntry, NodePointer};
 use singlefs_core::root_record::RootRecord;
-use singlefs_core::root_ring::{slot_offset, target_for_publish};
-use singlefs_harness::history::HistoryDeviceWidth;
+use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
+use singlefs_core::transaction::PoolVersion;
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
 use std::collections::BTreeMap;
 
@@ -50,6 +66,8 @@ struct PoolWithAnAbandonedRoot {
     image: MemoryPool,
     write_time_seconds: u64,
     abandoned_root: RootRecord,
+    /// 实例 3 退出时现行那一版的根（按实例 3 的表有效）：下一次可写挂载择到它，根环表把它记成有效根。
+    newest_valid_root: RootRecord,
 }
 
 fn read_bytes_of(
@@ -88,7 +106,11 @@ fn write_bytes_to(
 
 /// 攻方原型 `abandoned_start` 照搬（之前覆盖写 1 次）：见文件头。
 fn pool_whose_newest_root_was_abandoned_by_a_crash_recovery() -> PoolWithAnAbandonedRoot {
-    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
+    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
+        parameters_of_the_384_slot_small_devices(),
+        device_bytes_of_the_384_slot_small_devices(),
+        plain_device,
+    );
     pool.overwrite(OVERWRITE_BYTES).expect("实例 1 覆盖写一次");
     pool.crash_and_mount_writable().expect("崩了再挂：取号 2");
     pool.overwrite(CONTENT_BYTES_OF_THE_ABANDONED_VERSION)
@@ -169,6 +191,7 @@ fn pool_whose_newest_root_was_abandoned_by_a_crash_recovery() -> PoolWithAnAband
     for (device, offset, bytes) in &hidden_bytes {
         write_bytes_to(&mut pool, *device, *offset, bytes);
     }
+    let newest_valid_root = *pool.session().current.root();
     pool.session = None;
     PoolWithAnAbandonedRoot {
         parameters: pool.parameters.clone(),
@@ -176,18 +199,40 @@ fn pool_whose_newest_root_was_abandoned_by_a_crash_recovery() -> PoolWithAnAband
         image: pool.image(),
         write_time_seconds: pool.write_time_seconds,
         abandoned_root: abandoned_version.root,
+        newest_valid_root,
     }
 }
 
-/// C 的账那几个单元的读故障：`is_active` 为真时，读到 `ranges` 里任一段（盘、起点字节、终点字节）就报错。几块盘共用一份。
+/// 挑出来的几段的读故障（一条根的账那几个单元，或一条根的根槽）：`is_active` 为真时，读到 `ranges` 里任一段（盘、起点字节、终点字节）
+/// 就报错，盘上的字节不动。几块盘共用一份。
 #[derive(Clone)]
-struct AccountReadFault {
+struct ReadFaultOnChosenSlots {
     ranges: Rc<Vec<(DeviceIdentity, u64, u64)>>,
     is_active: Rc<Cell<bool>>,
 }
 
-impl AccountReadFault {
-    /// C 的树表、中央映射树根、分配记录树根（空指针不算）的每条位置条目各一个槽。故障先关着。
+impl ReadFaultOnChosenSlots {
+    /// 这条根住的那个根环槽（它的 txg 在根环里对应的那一槽，一个物理块）。故障先关着。
+    fn on_the_root_ring_slot_of(root: &RootRecord, parameters: &MakeFilesystemParameters) -> Self {
+        let ring_slot = target_for_publish(
+            root.checkpoint_txg,
+            parameters.geometry.root_ring_slots_per_region,
+        );
+        let device = parameters.region_devices
+            [usize::try_from(ring_slot.region).expect("区域号装得进 usize")];
+        let start_in_bytes =
+            slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing).0;
+        Self {
+            ranges: Rc::new(vec![(
+                device,
+                start_in_bytes,
+                start_in_bytes + u64::from(parameters.geometry.physical_block_size),
+            )]),
+            is_active: Rc::new(Cell::new(false)),
+        }
+    }
+
+    /// 这条根的树表、中央映射树根、分配记录树根（空指针不算）的每条位置条目各一个槽。故障先关着。
     fn on_the_account_of(root: &RootRecord) -> Self {
         let ranges = [
             root.tree_table,
@@ -222,13 +267,13 @@ impl AccountReadFault {
     }
 }
 
-struct DeviceWhoseReadsOfTheAbandonedAccountFail {
+struct DeviceWithReadFaultsOnChosenSlots {
     identity: DeviceIdentity,
     inner: SparseBlockDevice,
-    fault: AccountReadFault,
+    fault: ReadFaultOnChosenSlots,
 }
 
-impl BlockDevice for DeviceWhoseReadsOfTheAbandonedAccountFail {
+impl BlockDevice for DeviceWithReadFaultsOnChosenSlots {
     fn read_at(
         &self,
         offset: DeviceOffsetInBytes,
@@ -237,7 +282,7 @@ impl BlockDevice for DeviceWhoseReadsOfTheAbandonedAccountFail {
         let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
         if self.fault.refuses(self.identity, offset.0, length_in_bytes) {
             return Err(BlockDeviceError::InputOutput(std::io::Error::other(
-                "C 的账那几个单元从这一刻起读不出",
+                "挑出来的那几段从这一刻起读不出",
             )));
         }
         self.inner.read_at(offset, buffer)
@@ -273,22 +318,22 @@ impl BlockDevice for DeviceWhoseReadsOfTheAbandonedAccountFail {
     }
 }
 
-impl DeviceOverASparseImage for DeviceWhoseReadsOfTheAbandonedAccountFail {
+impl DeviceOverASparseImage for DeviceWithReadFaultsOnChosenSlots {
     fn sparse_image(&self) -> &SparseDevice {
         &self.inner.image
     }
 }
 
-fn devices_with_the_account_fault_on(
+fn devices_with_the_read_fault_on(
     image: &MemoryPool,
-    fault: &AccountReadFault,
-) -> Vec<(DeviceIdentity, DeviceWhoseReadsOfTheAbandonedAccountFail)> {
-    common_admission::plain_devices_on(image)
+    fault: &ReadFaultOnChosenSlots,
+) -> Vec<(DeviceIdentity, DeviceWithReadFaultsOnChosenSlots)> {
+    plain_devices_on(image)
         .into_iter()
         .map(|(identity, inner)| {
             (
                 identity,
-                DeviceWhoseReadsOfTheAbandonedAccountFail {
+                DeviceWithReadFaultsOnChosenSlots {
                     identity,
                     inner,
                     fault: fault.clone(),
@@ -298,21 +343,42 @@ fn devices_with_the_account_fault_on(
         .collect()
 }
 
-/// 可写挂载一次（实例 4，空间准入判着、影子账开着），故障关着：C 的账读得出、根环表把 C 记成被抛弃根。交回挂着的池与那份故障开关。
-fn mounted_while_the_account_of_the_abandoned_root_reads(
+/// 挂着的池（盘上带着 `fault` 那一层）：会话是 `mounted` 那一次挂载的。
+fn pool_under_test_after_the_mount(
+    parameters: &MakeFilesystemParameters,
+    device_bytes: u64,
+    devices: Vec<(DeviceIdentity, DeviceWithReadFaultsOnChosenSlots)>,
+    mounted: singlefs_core::mount::Mounted,
+    write_time_seconds: u64,
+) -> PoolUnderTest<DeviceWithReadFaultsOnChosenSlots> {
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    PoolUnderTest {
+        parameters: parameters.clone(),
+        device_bytes,
+        devices,
+        session: Some(session),
+        write_time_seconds,
+        content_of_the_current_version: Vec::new(),
+        content_of_each_root: BTreeMap::new(),
+    }
+}
+
+/// 可写挂载一次（实例 4，空间准入判着、影子账开着），`fault` 关着：C 的根槽与账都读得出、根环表把 C 记成被抛弃根。
+/// 交回挂着的池与挂载那一刻根环表里记着的 C 的落点（非空）。
+fn mounted_while_the_abandoned_root_reads(
     built: &PoolWithAnAbandonedRoot,
+    fault: &ReadFaultOnChosenSlots,
 ) -> (
-    PoolUnderTest<DeviceWhoseReadsOfTheAbandonedAccountFail>,
-    AccountReadFault,
+    PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
+    Vec<PlacementOnDevice>,
 ) {
-    let fault = AccountReadFault::on_the_account_of(&built.abandoned_root);
-    let mut devices = devices_with_the_account_fault_on(&built.image, &fault);
+    let mut devices = devices_with_the_read_fault_on(&built.image, fault);
     let mounted = mount_writable_with_space_admission(
         &built.parameters,
         &mut devices,
         SpaceAdmission::JudgedByTheFormula,
     )
-    .expect("C 的账读得出：可写挂载做得成");
+    .expect("C 的根槽与账读得出：可写挂载做得成");
     let recorded = mounted
         .allocator
         .root_ring_occupancy()
@@ -320,29 +386,47 @@ fn mounted_while_the_account_of_the_abandoned_root_reads(
         .placements_recorded_for_the_abandoned_root(
             built.abandoned_root.instance,
             built.abandoned_root.checkpoint_txg,
-        );
+        )
+        .filter(|placements| !placements.is_empty())
+        .expect("挂载那一刻根环表把 C 记成被抛弃根、带着它引用的落点");
+    let pool = pool_under_test_after_the_mount(
+        &built.parameters,
+        built.device_bytes,
+        devices,
+        mounted,
+        built.write_time_seconds + 1000,
+    );
+    (pool, recorded)
+}
+
+/// 抬 F 之后 `placements` 里每个落点在分配器里都不空闲：隔离着，或现行账里还分配着。被抛弃根离开根环之前它引用的单元不许发出去
+/// （D23（journal 的角色与格式） 已定项 14 回退段主句）；抬 F 越过一条候选根之后，此前靠它豁免、现行账里已释放的落点只靠影子账这一遍补的隔离护着。
+fn assert_none_of_the_placements_is_free(
+    allocator: &PoolAllocator,
+    placements: &[PlacementOnDevice],
+    whose: &str,
+) {
+    let free_placements: Vec<&PlacementOnDevice> = placements
+        .iter()
+        .filter(|placement| {
+            allocator
+                .devices
+                .iter()
+                .find(|device_map| device_map.device == placement.device)
+                .expect("池里有这块盘")
+                .is_free(placement.slot)
+        })
+        .collect();
     assert!(
-        recorded
-            .as_ref()
-            .is_some_and(|placements| !placements.is_empty()),
-        "挂载那一刻根环表把 C 记成被抛弃根、带着它引用的落点：{recorded:?}"
+        free_placements.is_empty(),
+        "抬 F 之后{whose}的落点一槽都不空闲（隔离着或现行账里还分配着）：空闲的 {free_placements:?}（共 {} 个落点）",
+        placements.len()
     );
-    let (_, session) = MountedSession::of_the_mount(mounted);
-    let pool = PoolUnderTest {
-        parameters: built.parameters.clone(),
-        device_bytes: built.device_bytes,
-        devices,
-        session: Some(session),
-        write_time_seconds: built.write_time_seconds + 1000,
-        content_of_the_current_version: Vec::new(),
-        content_of_each_root: BTreeMap::new(),
-    };
-    (pool, fault)
 }
 
 /// 故障开着之后 C 的树表那一槽确实读不出（阳性对照：用例不是在读得出的盘上空转）。
 fn assert_the_tree_table_of_the_abandoned_root_is_unreadable(
-    pool: &PoolUnderTest<DeviceWhoseReadsOfTheAbandonedAccountFail>,
+    pool: &PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
     abandoned_root: &RootRecord,
 ) {
     let location = abandoned_root.tree_table.locations[0];
@@ -367,15 +451,12 @@ fn violations_of_unit_reuse_on(image: &MemoryPool) -> Vec<String> {
         .collect()
 }
 
-/// 挂上之后 C 的账读不出：正常卸载照根环表里记着的 C 的落点算影子账，做成；卸载之后池级 checker 不红 I-7.4（近 K 代块未被复用）与 I-4.8。
-/// 改之前这一次卸载拒（`NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`）。
-#[test]
-fn unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
-) {
-    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
-    let (mut pool, fault) = mounted_while_the_account_of_the_abandoned_root_reads(&built);
-    fault.is_active.set(true);
-    assert_the_tree_table_of_the_abandoned_root_is_unreadable(&pool, &built.abandoned_root);
+/// 正常卸载一次（影子账开着），要它做成（推一串抬 F 到现行那一版）。交回这一串的 `abandoned_roots_unreadable` 与卸载之后的分配器。
+/// `why` 只用在断言消息里，说是哪一形。
+fn unmount_expecting_the_floor_raised(
+    pool: &mut PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
+    why: &str,
+) -> (u64, PoolAllocator) {
     let mut session = pool.session.take().expect("挂着的会话");
     let unmounted = unmount(
         &pool.parameters,
@@ -385,15 +466,34 @@ fn unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_rec
         ShadowLedger::On,
     );
     match unmounted {
-        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => assert_eq!(
-            raised.abandoned_roots_unreadable, 0,
-            "C 的落点取根环表里记着的那一份，这一遍不读它的账"
-        ),
+        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
+            (raised.abandoned_roots_unreadable, session.allocator)
+        }
         Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
             panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
         }
-        Err(refusal) => panic!("C 的账读不出，卸载照根环表里记着的落点做成，而被拒：{refusal:?}"),
+        Err(refusal) => panic!("{why}：卸载照根环表里记着的落点做成，而被拒：{refusal:?}"),
     }
+}
+
+/// 挂上之后 C 的账读不出：正常卸载照根环表里记着的 C 的落点算影子账，做成；卸载之后 C 记着的落点一槽都不空闲，
+/// 池级 checker 不红 I-7.4（近 K 代块未被复用）与 I-4.8。
+/// 改之前这一次卸载拒（`NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`）。
+#[test]
+fn unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let fault = ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root);
+    let (mut pool, recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
+    fault.is_active.set(true);
+    assert_the_tree_table_of_the_abandoned_root_is_unreadable(&pool, &built.abandoned_root);
+    let (abandoned_roots_unreadable, allocator_after_the_unmount) =
+        unmount_expecting_the_floor_raised(&mut pool, "C 的账读不出");
+    assert_eq!(
+        abandoned_roots_unreadable, 0,
+        "C 的落点取根环表里记着的那一份，这一遍不读它的账"
+    );
+    assert_none_of_the_placements_is_free(&allocator_after_the_unmount, &recorded, "C 记着");
     let violations = violations_of_unit_reuse_on(&pool.image());
     assert!(
         violations.is_empty(),
@@ -403,7 +503,7 @@ fn unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_rec
 
 /// 这一刻根环表里还把 C 记成被抛弃根（这个进程还没写掉 C 那一槽）。
 fn the_root_ring_table_still_records_the_abandoned_root(
-    pool: &PoolUnderTest<DeviceWhoseReadsOfTheAbandonedAccountFail>,
+    pool: &PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
     abandoned_root: &RootRecord,
 ) -> bool {
     pool.session()
@@ -417,16 +517,12 @@ fn the_root_ring_table_still_records_the_abandoned_root(
         .is_some()
 }
 
-/// 挂上之后 C 的账读不出：会话接着覆盖写到准入推出第一串抬 F（推的那一刻 C 还在根环里），再写几次，一次都不拒——准入推的抬 F
-/// 照根环表里记着的 C 的落点算影子账。写完池级 checker 不红 I-7.4 与 I-4.8。
-/// 改之前推抬 F 的那一次覆盖写拒（`FloorRaiseFailedWhilePushingForSpace`，拒因点名 C 的账）。
-#[test]
-fn session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
+/// 会话接着覆盖写，直到准入推出第一串抬 F（推的那一刻 C 还在根环里），一次都不许拒。`why` 只用在断言消息里。
+fn overwrite_until_the_admission_pushes_the_first_floor_raise(
+    pool: &mut PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
+    abandoned_root: &RootRecord,
+    why: &str,
 ) {
-    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
-    let (mut pool, fault) = mounted_while_the_account_of_the_abandoned_root_reads(&built);
-    fault.is_active.set(true);
-    assert_the_tree_table_of_the_abandoned_root_is_unreadable(&pool, &built.abandoned_root);
     let mut overwrites_before_the_first_floor_raise = 0;
     // 迭代上界 OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST；跨轮不带状态（计数之外）；出口：推出抬 F 的那一次 break，被拒就当场判红（panic）。
     loop {
@@ -435,10 +531,10 @@ fn session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_
             "{OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST} 次覆盖写之内准入推出抬 F（384 槽小盘）"
         );
         let abandoned_root_was_in_the_ring =
-            the_root_ring_table_still_records_the_abandoned_root(&pool, &built.abandoned_root);
+            the_root_ring_table_still_records_the_abandoned_root(pool, abandoned_root);
         let published = pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
             panic!(
-                "第 {} 次覆盖写：C 的账读不出，准入抬 F 照根环表里记着的落点做成，而被拒：{refusal:?}",
+                "第 {} 次覆盖写：{why}，准入抬 F 照根环表里记着的落点做成，而被拒：{refusal:?}",
                 overwrites_before_the_first_floor_raise + 1
             )
         });
@@ -448,9 +544,28 @@ fn session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_
                 abandoned_root_was_in_the_ring,
                 "准入推抬 F 的那一刻 C 还在根环里（不然这一串里没有被抛弃根要算，没走到要判的那一处）"
             );
-            break;
+            return;
         }
     }
+}
+
+/// 挂上之后 C 的账读不出：会话接着覆盖写到准入推出第一串抬 F（推的那一刻 C 还在根环里），再写几次，一次都不拒——准入推的抬 F
+/// 照根环表里记着的 C 的落点算影子账；推过那一刻 C 记着的落点一槽都不空闲。写完池级 checker 不红 I-7.4 与 I-4.8。
+/// 改之前推抬 F 的那一次覆盖写拒（`FloorRaiseFailedWhilePushingForSpace`，拒因点名 C 的账）。
+#[test]
+fn session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let fault = ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root);
+    let (mut pool, recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
+    fault.is_active.set(true);
+    assert_the_tree_table_of_the_abandoned_root_is_unreadable(&pool, &built.abandoned_root);
+    overwrite_until_the_admission_pushes_the_first_floor_raise(
+        &mut pool,
+        &built.abandoned_root,
+        "C 的账读不出",
+    );
+    assert_none_of_the_placements_is_free(&pool.session().allocator, &recorded, "C 记着");
     for overwrite_number in 1..=OVERWRITES_AFTER_THE_FIRST_FLOOR_RAISE {
         pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
             panic!("推过抬 F 之后第 {overwrite_number} 次覆盖写被拒：{refusal:?}")
@@ -469,9 +584,9 @@ fn session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_
 fn writable_mount_after_a_crash_still_refuses_while_the_account_of_an_abandoned_root_is_unreadable_without_the_credential(
 ) {
     let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
-    let fault = AccountReadFault::on_the_account_of(&built.abandoned_root);
+    let fault = ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root);
     fault.is_active.set(true);
-    let mut devices = devices_with_the_account_fault_on(&built.image, &fault);
+    let mut devices = devices_with_the_read_fault_on(&built.image, &fault);
     let refused = mount_writable_with_space_admission(
         &built.parameters,
         &mut devices,
@@ -507,3 +622,364 @@ fn writable_mount_after_a_crash_still_refuses_while_the_account_of_an_abandoned_
         "在取号与任何写之前拒：两块盘逐字节不变"
     );
 }
+
+/// 故障开着之后这条根的根槽确实读不出（阳性对照）。
+fn assert_the_root_ring_slot_is_unreadable(
+    pool: &PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
+    root: &RootRecord,
+) {
+    let ring_slot = target_for_publish(
+        root.checkpoint_txg,
+        pool.parameters.geometry.root_ring_slots_per_region,
+    );
+    let device = pool.parameters.region_devices
+        [usize::try_from(ring_slot.region).expect("区域号装得进 usize")];
+    let (_, block_device) = pool
+        .devices
+        .iter()
+        .find(|(identity, _)| *identity == device)
+        .expect("根槽所在的盘");
+    let mut bytes = vec![
+        0u8;
+        usize::try_from(pool.parameters.geometry.physical_block_size)
+            .expect("物理块装得进 usize")
+    ];
+    assert!(
+        block_device
+            .read_at(
+                slot_offset(
+                    ring_slot,
+                    pool.parameters.geometry.fixed_structure_slot_spacing
+                ),
+                &mut bytes
+            )
+            .is_err(),
+        "故障开着：({:?}, {:?}) 那一条根的根槽读不出",
+        root.instance,
+        root.checkpoint_txg
+    );
+}
+
+/// H1（用户 2026-10-08 定「被抛弃根的槽不拒」）：挂上之后 C 的根槽读报错、账读得出。正常卸载那一串重算影子账读根环时 C 那一槽读坏、
+/// 重读仍坏不拒，照根环表里记着的 C 的落点进影子账：卸载做成、这一遍不读 C 的账（计数 0）、C 记着的落点一槽都不空闲，
+/// 池级 checker 不红 I-7.4 与 I-4.8。改之前那一槽算「这个进程知道住着根的槽」，重读仍坏就拒
+/// （`NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot)`，攻方 `research/prompts/m3-batch10z-b-code-r1-opus-output.md` K1-1）。
+#[test]
+fn unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let fault =
+        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
+    let (mut pool, recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
+    fault.is_active.set(true);
+    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
+    let (abandoned_roots_unreadable, allocator_after_the_unmount) =
+        unmount_expecting_the_floor_raised(&mut pool, "C 的根槽读不出");
+    assert_eq!(
+        abandoned_roots_unreadable, 0,
+        "C 的根槽读不出：照根环表里记着的落点算，不读它的账"
+    );
+    assert_none_of_the_placements_is_free(&allocator_after_the_unmount, &recorded, "C 记着");
+    let violations = violations_of_unit_reuse_on(&pool.image());
+    assert!(
+        violations.is_empty(),
+        "卸载之后 C 引用的单元一个都没复用：{violations:?}"
+    );
+}
+
+/// H1：挂上之后 C 的根槽读报错，会话覆盖写到准入推出第一串抬 F（推的那一刻 C 还在根环里）：准入算上限那一遍与抬 F 重算影子账那一遍
+/// 读到 C 那一槽读坏都不拒（上限本来不看被抛弃根；影子账照记着的落点算），推过那一刻 C 记着的落点一槽都不空闲；之后再写几次也不拒，
+/// 池级 checker 不红 I-7.4 与 I-4.8。改之前推抬 F 的那一次覆盖写拒（`FloorRaiseFailedWhilePushingForSpace`，拒因
+/// `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，点名 C 那一槽）。
+#[test]
+fn session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let fault =
+        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
+    let (mut pool, recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
+    fault.is_active.set(true);
+    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
+    overwrite_until_the_admission_pushes_the_first_floor_raise(
+        &mut pool,
+        &built.abandoned_root,
+        "C 的根槽读不出",
+    );
+    assert_none_of_the_placements_is_free(&pool.session().allocator, &recorded, "C 记着");
+    let violations = violations_of_unit_reuse_on(&pool.image());
+    assert!(
+        violations.is_empty(),
+        "推过抬 F 之后 C 引用的单元一个都没复用：{violations:?}"
+    );
+}
+
+/// H1 的另一半：有效根的槽照「这个进程知道住着根的槽」那一句，读坏重读仍坏就拒。挂上之后这次挂载择到的那条有效根（实例 3 退出时现行那一版，
+/// 根环表记成有效根）的根槽读报错：正常卸载拒成 `NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot)`、点名那一槽，
+/// 在任何写之前，两块盘逐字节不变。
+#[test]
+fn unmount_after_the_root_ring_slot_of_a_valid_root_turns_unreadable_is_still_refused_before_any_write(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let fault = ReadFaultOnChosenSlots::on_the_root_ring_slot_of(
+        &built.newest_valid_root,
+        &built.parameters,
+    );
+    let (mut pool, _recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
+    fault.is_active.set(true);
+    assert_the_root_ring_slot_is_unreadable(&pool, &built.newest_valid_root);
+    let expected_ring_slot: RootRingSlot = target_for_publish(
+        built.newest_valid_root.checkpoint_txg,
+        built.parameters.geometry.root_ring_slots_per_region,
+    );
+    let image_before_the_unmount = pool.image();
+    let mut session = pool.session.take().expect("挂着的会话");
+    let refused = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut session.allocator,
+        &mut session.current,
+        ShadowLedger::On,
+    );
+    match refused {
+        Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) => {
+            assert!(
+                matches!(
+                    *still_unreadable,
+                    StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { ring_slot, .. }
+                        if ring_slot == expected_ring_slot
+                ),
+                "拒因点名那条有效根的根槽 {expected_ring_slot:?}：{still_unreadable:?}"
+            );
+        }
+        Err(other) => panic!("拒因是有效根的根槽重读仍坏，而是：{other:?}"),
+        Ok(_) => panic!("有效根的根槽读坏、重读仍坏：卸载照拒，而做成了"),
+    }
+    assert!(
+        pool.image() == image_before_the_unmount,
+        "在任何写之前拒：两块盘逐字节不变"
+    );
+}
+
+/// H2 那块盘之前覆盖写几次（攻方 `research/prompts/m3-batch10z-b-code-r1-opus-output.md` K2-2 原样那一格 k = 5）。
+const OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH: u64 = 5;
+
+/// H2（用户 2026-10-08 定「用 C 那一圈记着的落点」）：`tests/common_unwitnessed_publish/mod.rs` 那块盘（覆盖写 5 次之后录一次覆盖写 R，
+/// 崩在 R 的根槽落盘之后、轮换之前），可写挂载那一刻 R 的账（树表、映射根、分配记录树根）之外写出的单元读回全零：R 走读不成、择根退到 P，
+/// R 的根槽与账读得出，按最新那条根（R 自己）的表
+/// 判有效、根环表记成有效根、不带落点；写行之后 R 按现行那一版的表被抛弃。挂上之后 R 的账（树表、映射根、分配记录树根）读报错：
+/// 正常卸载那一串照 C 那一圈在 R 那一槽记着的落点算（那一槽记的是有效根、txg 与 R 相同），不读 R 的账、不拒（计数 0），那些落点一槽都不空闲，
+/// 池级 checker 不红 I-7.4 与 I-4.8。改之前表里没记着 R 就读盘：账与映射都读不出、现行账里没有凭据，卸载拒
+/// （`NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`，两腿都量到的白拒）。
+#[test]
+fn unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH,
+        PublishRecordedAsR::Overwrite,
+    );
+    let root_of_r = recorded.root();
+    let account_locations_of_r: Vec<LocationEntry> = [
+        root_of_r.tree_table,
+        root_of_r.mapping_root,
+        root_of_r.allocation_record_tree_root,
+    ]
+    .into_iter()
+    .filter(|pointer| *pointer != NodePointer::empty_root())
+    .flat_map(|pointer| pointer.locations)
+    .collect();
+    let unit_writes_of_r_outside_its_account: Vec<&singlefs_harness::memory_pool::RetainedWrite> =
+        recorded
+            .unit_writes()
+            .into_iter()
+            .filter(|write| {
+                !account_locations_of_r.iter().any(|location| {
+                    location.device == write.device
+                        && location.slot.to_device_offset() == write.offset
+                })
+            })
+            .collect();
+    assert!(
+        !unit_writes_of_r_outside_its_account.is_empty(),
+        "R 那次覆盖写写出了账之外的单元（数据单元、两棵树的节点）"
+    );
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let ranges =
+        RangesReadingBackAllZero::default().with_the_writes(&unit_writes_of_r_outside_its_account);
+    let mut mount_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let mounted = mount_writable_with_space_admission(
+        &recorded.pool.parameters,
+        &mut mount_devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("R 的账之外写出的单元读回全零：可写挂载做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 走读不成：择根退到 P"
+    );
+    let root_ring_table = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表");
+    assert_eq!(
+        root_ring_table.placements_recorded_for_the_abandoned_root(
+            root_of_r.instance,
+            root_of_r.checkpoint_txg
+        ),
+        None,
+        "挂载那一刻 R 记成有效根：根环表里没记着它的被抛弃根落点"
+    );
+    let named_on_the_ring_slot_of_r = root_ring_table
+        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+            root_of_r.instance,
+            root_of_r.checkpoint_txg,
+        )
+        .filter(|placements| !placements.is_empty())
+        .expect("C 那一圈在 R 那一槽记着 R（R 的记录点名的、R 的账里的落点）");
+    let image_after_the_mount =
+        image_of_devices_reading_back_all_zero(&mount_devices, recorded.pool.device_bytes);
+    let fault = ReadFaultOnChosenSlots::on_the_account_of(&root_of_r);
+    let devices = devices_with_the_read_fault_on(&image_after_the_mount, &fault);
+    let mut pool = pool_under_test_after_the_mount(
+        &recorded.pool.parameters,
+        recorded.pool.device_bytes,
+        devices,
+        mounted,
+        recorded.pool.write_time_seconds + 10_000,
+    );
+    fault.is_active.set(true);
+    assert_the_tree_table_of_the_abandoned_root_is_unreadable(&pool, &root_of_r);
+    let (abandoned_roots_unreadable, allocator_after_the_unmount) =
+        unmount_expecting_the_floor_raised(&mut pool, "R 的账读不出");
+    assert_eq!(
+        abandoned_roots_unreadable, 0,
+        "R 的落点借 C 那一圈在 R 那一槽记着的那一份，这一遍不读它的账"
+    );
+    assert_none_of_the_placements_is_free(
+        &allocator_after_the_unmount,
+        &named_on_the_ring_slot_of_r,
+        "C 那一圈在 R 那一槽记着",
+    );
+    let violations = violations_of_unit_reuse_on(&pool.image());
+    assert!(
+        violations.is_empty(),
+        "卸载之后 R 引用的单元一个都没复用：{violations:?}"
+    );
+}
+
+/// 凭据放行那一形里，挂载之后至多再覆盖写几次去等 R 的树表槽与映射根槽都被重新分配（384 槽小盘一圈根环 24 次发布之内）。
+const OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST: u64 = 24;
+
+/// 现行账里有一条分配代大于 `root` 的 txg、罩住这一槽的记录（C694 的 B 那道凭据的一条位置条目）。
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
+/// 攻方 M7（`research/prompts/m3-batch10z-b-code-r1-opus-output.md` 第三节）：挂载那一刻凭据放行的被抛弃根，根环表里记着的是空串，
+/// 抬 F 那一串照空串算、不再读它的账。造法同 `abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs`
+/// 「已被占的那一形」：M1 里 R 的根槽与单元读回全零、影子账关着，之后覆盖写到 R 的树表槽与映射根槽都被重新分配；再可写挂载一次（M2）：
+/// R 的根槽读得出、按实例表被抛弃，账与映射都读不出、现行账里有凭据——放行，根环表把 R 记成被抛弃根、记着空串，挂载计数 1。
+/// M2 正常卸载：照记着的空串算，不读 R 的账（计数 0），做成。照读盘时这一遍又数到 R（计数 1）。
+#[test]
+fn unmount_takes_the_empty_placements_recorded_for_an_abandoned_root_the_credential_passed_at_mount_without_reading_its_account(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH,
+        PublishRecordedAsR::Overwrite,
+    );
+    let root_of_r = recorded.root();
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&[recorded.root_slot_write()])
+        .with_the_writes(&recorded.unit_writes());
+    let mut devices_of_the_first_mount = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let first_mount = mount_writable_with_test_only_switches(
+        &recorded.pool.parameters,
+        &mut devices_of_the_first_mount,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::Off,
+    )
+    .expect("R 的根槽与单元读回全零：可写挂载落到 P、做得成");
+    let image_after_the_first_mount = image_of_devices_reading_back_all_zero(
+        &devices_of_the_first_mount,
+        recorded.pool.device_bytes,
+    );
+    let mut pool = pool_after_the_mount(&recorded, &image_after_the_first_mount, first_mount);
+    let account_slots_of_r: Vec<LocationEntry> = root_of_r
+        .tree_table
+        .locations
+        .iter()
+        .chain(root_of_r.mapping_root.locations.iter())
+        .copied()
+        .collect();
+    let mut overwrites_after_the_first_mount = 0;
+    // 迭代上界 OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST；每一轮看一次现行账，不够再覆盖写一次。
+    while !account_slots_of_r.iter().all(|location| {
+        a_later_record_covers(
+            &pool.current_file_version().allocation_records,
+            &root_of_r,
+            location,
+        )
+    }) {
+        assert!(
+            overwrites_after_the_first_mount < OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST,
+            "{OVERWRITES_UNTIL_THE_ACCOUNT_SLOTS_ARE_TAKEN_AT_MOST} 次覆盖写之内 R 的树表槽与映射根槽都被重新分配"
+        );
+        pool.overwrite(OVERWRITE_BYTES)
+            .expect("384 槽小盘上这一串覆盖写都落得下");
+        overwrites_after_the_first_mount += 1;
+    }
+    let image = pool.image();
+    let mut devices_of_the_second_mount = plain_devices_on(&image);
+    let second_mount = mount_writable(&recorded.pool.parameters, &mut devices_of_the_second_mount)
+        .expect("R 的账读不出、现行账里有凭据：再可写挂载做得成");
+    assert_eq!(
+        second_mount.output.abandoned_roots_unreadable, 1,
+        "再可写挂载那一遍读不出 R 的账、凭据放行，计数往上报"
+    );
+    assert_eq!(
+        second_mount
+            .allocator
+            .root_ring_occupancy()
+            .expect("挂载建的分配器装着根环表")
+            .placements_recorded_for_the_abandoned_root(
+                root_of_r.instance,
+                root_of_r.checkpoint_txg
+            ),
+        Some(Vec::new()),
+        "凭据放行：根环表把 R 记成被抛弃根，记着的落点是空串"
+    );
+    let (_, mut session) = MountedSession::of_the_mount(second_mount);
+    let PoolVersion::WithFile(_) = &session.current else {
+        panic!("覆盖写之后现行那一版带文件");
+    };
+    let unmounted = unmount(
+        &recorded.pool.parameters,
+        &mut devices_of_the_second_mount,
+        &mut session.allocator,
+        &mut session.current,
+        ShadowLedger::On,
+    );
+    match unmounted {
+        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => assert_eq!(
+            raised.abandoned_roots_unreadable, 0,
+            "R 记着的是空串：照它算，这一遍不再读 R 的账"
+        ),
+        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
+            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
+        }
+        Err(refusal) => panic!("R 记着空串：卸载做得成，而被拒：{refusal:?}"),
+    }
+}
diff --git a/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs b/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
index 4db0cc3..4ab88fb 100644
--- a/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
+++ b/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
@@ -653,7 +653,8 @@ fn restore_the_root_slot(pool: &mut BuiltPool, hidden: &HiddenRootSlot) {
 /// 分配器整个换回抬 F 之前那一份（补上的隔离撤掉）。
 /// 抬 F 那一串里根环表记着的被抛弃根照记着的落点算、读不出也不拒（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，用户 2026-10-07 定，
 /// 根环线第 10z 批）：要走到影子账的拒，排在后面的那一条得是根环表里没记着的——再可写挂载那一刻它的根槽读回全零（挂载之后写回），
-/// 根环表把那一槽记成「挂载时读回全零」，抬 F 那一遍读出它、照盘上读它的账（这一形条款没写，照改之前，交主 agent）。
+/// 根环表把那一槽记成「挂载时读回全零」，抬 F 那一遍读出它、照盘上读它的账（那一槽记的不是有效根，C 那一圈也不借：两样都没记着的照今天读盘，
+/// D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定）。
 /// 判别力：影子账交 `Err` 时不换回分配器，前一条补上的隔离留在分配器上，「与抬 F 之前逐项相同」那一项红。
 #[test]
 fn a_floor_raise_whose_shadow_ledger_meets_an_abandoned_root_unreadable_by_account_and_mapping_is_refused_and_restores_the_allocator(
@@ -768,9 +769,12 @@ fn a_floor_raise_whose_shadow_ledger_meets_an_abandoned_root_unreadable_by_accou
 /// 被抛弃根的实例表有两片时，账读不出、从中央映射现算的那一臂把两片都算进它引用的落点（代码轮 LR2，攻方 `m3-leftover-code-r1` Z4 ①
 /// 「现算只认实例表第 0 片」放过：用例的池都只有一片实例表）。
 /// 造法：A、B（实例 1）之后连取 370 次号再可写挂载（实例 372），写行把 1 到 371 的 371 行写进两片实例表；覆盖写 C、崩溃恢复抛弃 C
-/// （实例 373 写行，换下两片）；C 的树表两份抹零（账读不出）再可写挂载（实例 374）、覆盖写 [`OVERWRITES_BEFORE_THE_RAISE`] 次、抬 F 到上限：新 F 越过 C 的前一版，
-/// C 那两片实例表此刻只被 C 引用、在现行那一版的账里已释放，抬 F 的影子账从映射现算、两片都隔离，回收之后照样不空闲。
-/// 判别力：现算只认实例表第 0 片时第 1 片不隔离、回收之后空闲，那一项红。
+/// （实例 373 写行，换下两片）；C 的树表两份抹零（账读不出）再可写挂载（实例 374）：挂载那一遍读不出 C 的账、从映射现算出它引用的落点（两片都在内），
+/// 根环表把 C 记成被抛弃根、记着这一份。覆盖写 [`OVERWRITES_BEFORE_THE_RAISE`] 次、抬 F 到上限：新 F 越过 C 的前一版，C 那两片实例表此刻只被 C 引用、
+/// 在现行那一版的账里已释放；抬 F 那一串照根环表里记着的那一份算（不再从盘上读 C 的账、也不再现算，D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，
+/// 根环线第 10z 批），两片都隔离，回收之后照样不空闲。
+/// 判别力：现算只认实例表第 0 片时挂载那一遍就只记下第 0 片，抬 F 照它算，第 1 片不隔离、回收之后空闲，那一项红；
+/// 抬 F 那一串记着的落点被削（只交一部分或交空）时同样红。
 #[test]
 fn an_abandoned_root_with_a_two_page_instance_table_and_an_unreadable_account_isolates_both_pages_from_the_mapping(
 ) {
@@ -906,3 +910,103 @@ fn an_abandoned_root_with_a_two_page_instance_table_and_an_unreadable_account_is
         "C 那两片实例表的落点抬 F 回收之后一槽都不空闲（从映射现算两片都隔离）"
     );
 }
+
+/// 抬 F 那一串里根环表没记着落点的被抛弃根照今天读盘，账读不出就从中央映射现算、照样隔离、计进 `abandoned_roots_unreadable`
+/// （C393 取 (b)-从映射现算；攻方 M11、正推 m19，判决 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第二节 K3：
+/// 这一批之前默认档里没有一条用例走到抬 F 那一遍现算做成的这一支，只有一条慢档用例）。
+/// 造法：崩溃恢复抛弃 C (2, 8) 之后再可写挂载（实例 4）那一刻 C 的根槽读回全零（挂载之后写回）：根环表把那一槽记成「挂载时读回全零」，
+/// 没记着 C 的落点，C 那一圈在那一槽也不借（那一槽记的不是有效根）。挂上之后 C 的树表两份抹零（账读不出、中央映射读得出），正常卸载：
+/// 抬 F 那一遍读出 C、按现行那一版的表被抛弃，读它的账读不出，从映射现算——卸载做成、计数 1，C 写出的单元一个都不空闲。
+/// 判别力：抬 F 那一遍不从映射现算（直接重读、看凭据）时，C 的树表槽没被后来的写占、没有凭据，卸载拒成 `AccountOfAnAbandonedRoot`，这一条红。
+#[test]
+fn an_unmount_meeting_an_abandoned_root_not_in_the_root_ring_table_whose_account_is_unreadable_recomputes_it_from_the_mapping(
+) {
+    let (mut pool, abandoned_c) =
+        pool_after_a_recovery_abandoned_c("m11-unmount-recomputes-from-the-mapping");
+    let hidden_root_slot = hide_the_root_slot_of(&mut pool, &abandoned_c.root);
+    let mut devices_of_the_mount = pool.reopen_recorded();
+    let mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices_of_the_mount)
+        .expect("C 的根槽读回全零：照常可写挂载");
+    pool.devices = Some(devices_of_the_mount);
+    pool.allocator = mounted.allocator;
+    pool.output = mounted.current.into_file_version().expect("带文件");
+    restore_the_root_slot(&mut pool, &hidden_root_slot);
+    let root_ring_table = pool
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表");
+    assert_eq!(
+        root_ring_table.placements_recorded_for_the_abandoned_root(
+            abandoned_c.root.instance,
+            abandoned_c.root.checkpoint_txg
+        ),
+        None,
+        "挂载那一刻 C 的根槽读回全零：根环表里没记着 C 的落点"
+    );
+    assert_eq!(
+        root_ring_table
+            .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+                abandoned_c.root.instance,
+                abandoned_c.root.checkpoint_txg
+            ),
+        None,
+        "C 那一槽记的不是有效根：不借 C 那一圈的落点"
+    );
+    {
+        use singlefs_core::block_device::{BlockDevice, WriteDurability};
+        let mut devices_to_zero_the_tree_table = pool.reopen_recorded();
+        let tree_table_of_c = abandoned_c.unit(TransactionUnit::TreeTable);
+        for (_, device) in &mut devices_to_zero_the_tree_table {
+            device
+                .write_at(
+                    DeviceOffsetInBytes(tree_table_of_c.slot.0 * SLOT_BYTES),
+                    &vec![0u8; tree_table_of_c.bytes.len()],
+                    WriteDurability::Plain,
+                )
+                .expect("C 的树表两份抹零");
+        }
+        pool.devices = Some(devices_to_zero_the_tree_table);
+    }
+    let mut version_to_unmount =
+        singlefs_core::transaction::PoolVersion::WithFile(pool.output.clone());
+    let mut devices_of_the_unmount = pool.reopen_recorded();
+    let unmounted = singlefs_core::mount::unmount(
+        &parameters(),
+        &mut devices_of_the_unmount,
+        &mut pool.allocator,
+        &mut version_to_unmount,
+        ShadowLedger::On,
+    );
+    pool.devices = Some(devices_of_the_unmount);
+    match unmounted {
+        Ok(singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
+            assert_eq!(
+                raised.abandoned_roots_unreadable, 1,
+                "抬 F 那一遍读不出 C 的账：从映射现算，照样计数往上报"
+            );
+        }
+        Ok(singlefs_core::mount::Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
+            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
+        }
+        Err(refusal) => {
+            panic!("C 的账读不出、映射读得出：卸载从映射现算做成，而被拒：{refusal:?}")
+        }
+    }
+    let free_units_of_c: Vec<(u32, u64)> = abandoned_c
+        .units
+        .iter()
+        .flat_map(|unit| {
+            pool.allocator
+                .devices
+                .iter()
+                .filter(|device_map| device_map.is_free(unit.slot))
+                .map(|device_map| (device_map.device.0, unit.slot.0))
+                .collect::<Vec<(u32, u64)>>()
+        })
+        .collect();
+    assert_eq!(
+        free_units_of_c,
+        Vec::<(u32, u64)>::new(),
+        "卸载之后 C 写出的单元一个都不空闲（从映射现算的落点都隔离着）"
+    );
+}
````

## 变异表追加（`mutations-append.tsv` 原样）

````tsv
第10z批第四块 H1 抬 F 重算影子账那一遍被抛弃根的槽也算重读仍坏就拒（挂上之后 C 的根槽读报错：正常卸载拒）	crates/singlefs-core/src/mount.rs	            .filter(|ring_slot| {\n                !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)\n            })\n            .collect();\n    let mut roots_with_ring_slots =	            .collect();\n    let mut roots_with_ring_slots =	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds
第10z批第四块 H1 准入算上限那一遍被抛弃根的槽也算重读仍坏就拒（挂上之后 C 的根槽读报错：会话写到推抬 F 那一次拒）	crates/singlefs-core/src/mount.rs	        .filter(|ring_slot| {\n            !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)\n        })\n        .collect();\n    rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(	        .collect();\n    rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements
第10z批第四块 H1 根槽这一遍读坏的被抛弃根记着的落点不进影子账（不拒，但前一版豁免、现行账已释放的落点回收出去）	crates/singlefs-core/src/mount.rs	    for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad {	    for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad\n        .iter()\n        .take(0)\n    {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds
第10z批第四块 H1 有效根的槽读坏重读仍坏也不拒（抬 F 那一遍知道住着根的槽一个都不按拒处置）	crates/singlefs-core/src/mount.rs	        ring_slots_known_to_hold_a_root_by(allocator)\n            .into_iter()\n            .filter(|ring_slot| {\n                !ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass	        BTreeSet::<RootRingSlot>::new()\n            .into_iter()\n            .filter(|ring_slot| {\n                !ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_the_root_ring_slot_of_a_valid_root_turns_unreadable_is_still_refused_before_any_write
第10z批第四块 H2 抬 F 那一串不借 C 那一圈的落点（写行之后才被抛弃的 R 的账读不出：正常卸载照读盘拒）	crates/singlefs-core/src/mount.rs	                        .or_else(|| {\n                            occupancy\n                                .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(\n                                    root.instance,\n                                    root.checkpoint_txg,\n                                )\n                        })\n		-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot
第10z批第四块 H2 借 C 那一圈落点只按槽号认（那一槽记的有效根 txg 不同、同一槽差一整圈的另一次发布也借）	crates/singlefs-core/src/allocator.rs	            } => *recorded_checkpoint_txg == checkpoint_txg,	            } => true || *recorded_checkpoint_txg == checkpoint_txg,	-p singlefs-core --lib -- placements_named_by_unapplied_journal_records_for_a_root_recorded_valid	placements_named_by_unapplied_journal_records_for_a_root_recorded_valid_answer_only_for_the_same_root_on_its_ring_slot
第10z批第四块 H2 两遍里只有一遍自证过的有效根那一槽借 C 那一圈落点不比实例代号与 txg	crates/singlefs-core/src/allocator.rs	                ) == (instance, checkpoint_txg)\n            }\n            RootRingOccupant::AbandonedRoot { .. }	                ) == (instance, checkpoint_txg)\n                    || true\n            }\n            RootRingOccupant::AbandonedRoot { .. }	-p singlefs-core --lib -- placements_named_by_unapplied_journal_records_for_a_root_recorded_valid	placements_named_by_unapplied_journal_records_for_a_root_recorded_valid_answer_only_for_the_same_root_on_its_ring_slot
第10z批第四块 K3 m1 抬 F 那一串取根环表记着的被抛弃根落点交空（前一版豁免、现行账已释放的落点卸载回收出去）	crates/singlefs-core/src/mount.rs	                    placements\n                        .iter()\n                        .map(|placement| (placement.device, placement.slot, placement.span))	                    placements\n                        .iter()\n                        .take(0)\n                        .map(|placement| (placement.device, placement.slot, placement.span))	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds
第10z批第四块 K3 m2 抬 F 那一串取根环表记着的被抛弃根落点只交第一个	crates/singlefs-core/src/mount.rs	                    placements\n                        .iter()\n                        .map(|placement| (placement.device, placement.slot, placement.span))	                    placements\n                        .iter()\n                        .take(1)\n                        .map(|placement| (placement.device, placement.slot, placement.span))	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds
第10z批第四块 K3 m5 取被抛弃根记着的落点对挂载时读坏的槽交空落点（不读盘也不隔离）	crates/singlefs-core/src/allocator.rs	            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,	            RootRingOccupant::BadAtMount(_) => Some(Vec::new()),\n            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,	-p singlefs-core --lib -- placements_recorded_for_an_abandoned_root_answer_none	placements_recorded_for_an_abandoned_root_answer_none_on_slots_recorded_bad_all_zero_or_valid_at_mount
第10z批第四块 K3 m6 取被抛弃根记着的落点对挂载时读回全零的槽交空落点（不读盘也不隔离）	crates/singlefs-core/src/allocator.rs	            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,	            RootRingOccupant::AllZeroAtMount { .. } => Some(Vec::new()),\n            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,	-p singlefs-core --lib -- placements_recorded_for_an_abandoned_root_answer_none	placements_recorded_for_an_abandoned_root_answer_none_on_slots_recorded_bad_all_zero_or_valid_at_mount
第10z批第四块 K3 m20 取被抛弃根记着的落点对读坏后进程内重读读出有效根的槽交空落点	crates/singlefs-core/src/allocator.rs	            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,	            RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => Some(Vec::new()),\n            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,	-p singlefs-core --lib -- placements_recorded_for_an_abandoned_root_answer_none	placements_recorded_for_an_abandoned_root_answer_none_on_slots_recorded_bad_all_zero_or_valid_at_mount
第10z批第四块 K3 m8 取被抛弃根记着的落点对两遍里只有一遍自证过的有效半边交空落点	crates/singlefs-core/src/allocator.rs	            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,	            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => Some(Vec::new()),\n            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => None,	-p singlefs-core --lib -- placements_recorded_for_an_abandoned_root_answer_none	placements_recorded_for_an_abandoned_root_answer_none_on_slots_recorded_bad_all_zero_or_valid_at_mount
第10z批第四块 K3 M7 抬 F 那一串表里记着空串（挂载时凭据放行）就不用它、照读盘	crates/singlefs-core/src/mount.rs	                .get(&(root.instance, root.checkpoint_txg))\n                .map(|placements| {	                .get(&(root.instance, root.checkpoint_txg))\n                .filter(|placements| !placements.is_empty())\n                .map(|placements| {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_takes_the_empty_placements_recorded_for_an_abandoned_root_the_credential_passed_at_mount_without_reading_its_account
第10z批第四块 K3 M11 / m19 被抛弃根的账读不出时不从中央映射现算（直接重读、看凭据：抬 F 那一遍表里没记着的被抛弃根照拒）	crates/singlefs-core/src/mount.rs	    if let Some(computed) =\n        placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)\n    {\n        return Ok(computed);\n    }\n    before_the_one_reread.before_rereading();\n    if let Some(placements) =\n        placements_referenced_by_root(devices, unit_area_start, root).or_else(|| {\n            placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)\n        })\n    {\n        return Ok(placements);\n    }	    before_the_one_reread.before_rereading();\n    if let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) {\n        return Ok(placements);\n    }	-p singlefs-harness --test an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping -- an_unmount_meeting_an_abandoned_root_not_in_the_root_ring_table	an_unmount_meeting_an_abandoned_root_not_in_the_root_ring_table_whose_account_is_unreadable_recomputes_it_from_the_mapping
````
