# 代码欠账批甲簇：代码轮改动全文（2026-10-14）

## 一、`crates/singlefs-core/src/mount.rs` 相对 HEAD f111c3f3（主工作区合入之后，含全仓改名带进来的注释改动）

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 7eed88ea..8699bbec 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -310,7 +310,7 @@ pub enum MountError {
     /// 都已落盘，写账口径见 [`FloorRaiseFailedAfterTheMountsPublishes`]。
     /// 装箱：不装箱 `MountError` 就大过 clippy `result_large_err` 的 128 字节。
     FloorRaiseFailedAfterTheMountsPublishes(Box<FloorRaiseFailedAfterTheMountsPublishes>),
-    /// C554 乙（用户 2026-09-27 定：重读后再判，R = 1；岔路单 `research/prompts/c554-fix-forks.md` 第 1 行）：
+    /// C554 乙（用户 2026-09-27 定：重读后再判，R = 1；岔路单 `research/prompts/c554-fix-forks.md` #1「挂载时有一条根（或它引用的账）暂时读不出」）：
     /// 可写挂载读到的样子里有一样更新的东西读不出，重读一次（D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」）仍读不出——
     /// 不按读得出的那一版往下走，不把读不出的那一版当成被抛弃，拒可写挂载。是哪一样读不出见 [`StillUnreadableAfterOneReread`]。
     /// 在取号之前返回：一个写都没发，盘上逐字节不变。只读挂载不走这一判，照常（`mounted_read`、`recovery::recover`）。
@@ -355,7 +355,7 @@ pub enum SystemConfigurationUnreadableWhen {
 /// 可写挂载重读一次之后仍读不出的是哪一样（[`MountError::NewerStateStillUnreadableAfterOneReread`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum StillUnreadableAfterOneReread {
-    /// 判据 N-配置（定义取自 E158 第 3 次跑登记 `research/prompts/e158-r3-prereg.md` 第 347 行）：系统配置见证过一次比所选那一版新的发布——
+    /// 判据 N-配置（定义取自 E158 第 3 次跑登记 `research/prompts/e158-r3-prereg.md`「`| **N-配置** |`」那一行）：系统配置见证过一次比所选那一版新的发布——
     /// 系统配置在发布的根槽 FUA 之后才轮换（D16（发布语义） 已定项 7），见证到的发布它的根落过盘——而读阶段（择根、扫 journal、重放）
     /// 交出的所选那一版比它旧；在这次挂载的读缓存上重做读阶段一遍（[`ReadStageCache`]），仍判真。
     PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
@@ -1567,10 +1567,15 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
     placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[PlacementOnDevice],
 ) -> Result<ShadowLedgerComputed, MountError> {
+    // 豁免与判重一律按槽：每份落点罩着的每一槽各进集合（D23（journal 的角色与格式） 已定项 14「只隔离其中只被被抛弃根引用的槽」，
+    // 判决 `research/prompts/m3-c554-c572-r1-main-verification.md` 第二节 B3 那一行）。按起点槽认时，同一起点、跨度不同的两份落点
+    // 窄的那份把宽的整份豁免或判重掉，宽的那份多出来的槽没人罩（攻方 `research/prompts/m3-c554-c572-r1-opus-output.md`「B3」一节的三种形）。
     let referenced_by_the_current_account: BTreeSet<(u32, u64)> = current_records
         .iter()
         .filter(|record| !record.is_released)
-        .map(|record| (record.device.0, record.slot.0))
+        .flat_map(|record| {
+            slots_covered_by_a_placement(record.device, record.slot, u64::from(record.span_slots))
+        })
         .collect();
     let mut referenced_by_candidates: BTreeSet<(u32, u64)> =
         referenced_by_the_current_account.clone();
@@ -1581,11 +1586,11 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
         .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)
     {
         if let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) {
-            referenced_by_candidates.extend(
-                placements
-                    .iter()
-                    .map(|(device, slot, _)| (device.0, slot.0)),
-            );
+            referenced_by_candidates.extend(placements.iter().flat_map(
+                |(device, slot, span_slots)| {
+                    slots_covered_by_a_placement(*device, *slot, *span_slots)
+                },
+            ));
         }
     }
     let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
@@ -1593,7 +1598,7 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     let mut placements_referenced_by_abandoned_root = BTreeMap::new();
     for root in roots.iter().filter(|root| is_abandoned(root)) {
         // C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮；
-        // `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）：账（树表或分配记录树）读不出时不跳过，
+        // `research/prompts/unreadable-at-mount-r2-main-verification.md`「L3　C393」一节）：账（树表或分配记录树）读不出时不跳过，
         // 从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报（显式报告）；
         // 现算也不成（映射树读不出、解不开）才重读一次（重读之前调钩子），账与现算都仍不成就在取号之前拒可写、只读照常。
         // 抬 F 那一串先取根环表里记着的那一份（`or_else` 是惰性的：取到就不读盘，读不出的处置也走不到）。
@@ -1637,17 +1642,23 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
             }
         };
         for (device, slot, span_slots) in placements.iter().copied() {
-            let key = (device.0, slot.0);
-            if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
-                continue;
-            }
-            if reclaim_isolates_what_this_root_references {
-                allocator
-                    .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
-                        device, slot, span_slots,
-                    );
-            } else {
-                allocator.isolate_abandoned(device, slot, span_slots);
+            for slot_to_isolate in slots_of_a_placement_to_isolate_now(
+                device,
+                slot,
+                span_slots,
+                &referenced_by_candidates,
+                &mut isolated,
+            ) {
+                if reclaim_isolates_what_this_root_references {
+                    allocator
+                        .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
+                            device,
+                            slot_to_isolate,
+                            SPAN_OF_ONE_SLOT,
+                        );
+                } else {
+                    allocator.isolate_abandoned(device, slot_to_isolate, SPAN_OF_ONE_SLOT);
+                }
             }
         }
         placements_referenced_by_abandoned_root.insert(
@@ -1661,15 +1672,20 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     // 根槽这一遍读坏的被抛弃根（H1）：读不出它的根记录，照它挂载时记着的落点补隔离，豁免与读出来的被抛弃根同一个集合。
     // 落点取自根环表，回收时照表补隔离：分配位置着的槽不在这里隔离（同上一圈）。
     for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad {
-        let key = (placement.device.0, placement.slot.0);
-        if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
-            continue;
-        }
-        allocator.isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
+        for slot_to_isolate in slots_of_a_placement_to_isolate_now(
             placement.device,
             placement.slot,
             placement.span,
-        );
+            &referenced_by_candidates,
+            &mut isolated,
+        ) {
+            allocator
+                .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
+                    placement.device,
+                    slot_to_isolate,
+                    SPAN_OF_ONE_SLOT,
+                );
+        }
     }
     // C 那一份不豁免候选根引用的槽（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K1 / K2 T1 C 的候选豁免」那一行，
     // 根环线 K4-1）：水位之上没施加的那次发布的根读得出、账读得出时它自己就是候选根（按实例表有效、txg ≥ F），豁免它引用的槽就是不护它的单元——
@@ -1682,16 +1698,20 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     // 分配器照还没解开的这一份补隔离：挂载之内记根时（`PoolAllocator::record_root_written_by_this_process`），抬 F 那一串回收时
     // （`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`，[`raise_rollback_floor`]）。
     for placement in placements_named_by_unapplied_journal_records {
-        let key = (placement.device.0, placement.slot.0);
-        if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {
-            continue;
+        for slot_to_isolate in slots_of_a_placement_to_isolate_now(
+            placement.device,
+            placement.slot,
+            placement.span,
+            &referenced_by_the_current_account,
+            &mut isolated,
+        ) {
+            allocator
+                .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(
+                    placement.device,
+                    slot_to_isolate,
+                    SPAN_OF_ONE_SLOT,
+                );
         }
-        allocator
-            .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(
-                placement.device,
-                placement.slot,
-                placement.span,
-            );
     }
     Ok(ShadowLedgerComputed {
         abandoned_roots_unreadable: unreadable,
@@ -1699,6 +1719,41 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     })
 }
 
+/// 影子账一次隔离一槽时交给分配器的跨度（[`slots_of_a_placement_to_isolate_now`] 按槽交回要隔离的槽）。
+const SPAN_OF_ONE_SLOT: u64 = 1;
+
+/// 一份落点罩着的每一槽，按 (设备号, 槽号)，槽号升序：影子账的豁免集合与判重集合都按它收（[`isolate_slots_referenced_only_by_abandoned_roots`]）。
+///
+/// # Panics
+/// 末槽装不进 u64：进影子账的落点都判过落在那块盘的单元区里（盘上读来的经 `recovery::placement_lies_in_the_unit_area_of_its_device`，
+/// 现行账里的记录经 `recovery::allocation_records_fit_the_pool_geometry`），走不到。
+fn slots_covered_by_a_placement(
+    device: DeviceIdentity,
+    slot: SlotNumber,
+    span_slots: u64,
+) -> impl Iterator<Item = (u32, u64)> {
+    let end_slot = slot.0.checked_add(span_slots).expect(
+        "落点末槽装得进 u64：进影子账的落点都判过落在那块盘的单元区里（recovery::placement_lies_in_the_unit_area_of_its_device、allocation_records_fit_the_pool_geometry）",
+    );
+    (slot.0..end_slot).map(move |covered_slot| (device.0, covered_slot))
+}
+
+/// 一份要护的落点（被抛弃根引用的，或水位之上没施加的记录点名的）里这一次要隔离的槽：它罩着的每一槽里，不在 `exempt` 里、
+/// 也还没进 `isolated` 的那几槽，槽号升序；交回的槽同时记进 `isolated`，`exempt` 里的不记（与改按槽之前的判重同一个次序：先看豁免，再判重）。
+/// 只动两个集合，不碰分配器。
+fn slots_of_a_placement_to_isolate_now(
+    device: DeviceIdentity,
+    slot: SlotNumber,
+    span_slots: u64,
+    exempt: &BTreeSet<(u32, u64)>,
+    isolated: &mut BTreeSet<(u32, u64)>,
+) -> Vec<SlotNumber> {
+    slots_covered_by_a_placement(device, slot, span_slots)
+        .filter(|covered| !exempt.contains(covered) && isolated.insert(*covered))
+        .map(|(_, covered_slot)| SlotNumber(covered_slot))
+        .collect()
+}
+
 /// 一条被抛弃根的账（树表或分配记录树）这一遍读不出之后，它引用的落点从哪来（C393 取 (b)-从映射现算，用户 2026-09-28 定，被攻过零轮；
 /// C694 的 B，用户 2026-10-07 定）：[`placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread`]，
 /// 那一份交 `None` 就拒。
@@ -2871,7 +2926,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
             .is_some()
         };
     // 这一遍有一块盘一条不被抛弃的自证有效根都没读出（择根那几遍读得出）就把根环整环再读一遍（用户 2026-10-05 定 Q4 rr2，
-    // 岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 第 4 行：改之前只在整个环一条都没读出时重读，只坏这一遍、只有一块盘没读出的那一形
+    // 岔路单 `research/prompts/m3-batch10-code-r1-forks.md` Q4 那一行：改之前只在整个环一条都没读出时重读，只坏这一遍、只有一块盘没读出的那一形
     // m 取不到、当场只放到环里最旧有效根，窄盘上取不到落点被拒；重读之前调 `before_the_one_reread`，与另几处重读一样），
     // 根环表、回收门槛与影子账都用两遍逐槽并起来的那一份（用户 2026-10-05 定 R1「按槽并两遍」，[`merged_readings_of_the_two_root_ring_passes`]：
     // 只拿重读那一遍时，第一遍读出、重读读坏的被抛弃根掉出影子账；并起来那一份仍有一块盘没读出的，照 m 取不到那一支往下走）；
@@ -2993,7 +3048,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
         )
         .min();
     // 卸载半途崩溃之后的下一次可写挂载只放开一截（D16（发布语义） 已定项 1「正常卸载」那一行，用户 2026-10-04 选甲，
-    // `research/prompts/m3-batch14-code-r1-forks.md` 第 1 行）：卸载那一串先写了系统配置里的新 F、带新 F 的根没落满每块盘就崩时，
+    // `research/prompts/m3-batch14-code-r1-forks.md`「甲 再挂也只放开一截」那一行）：卸载那一串先写了系统配置里的新 F、带新 F 的根没落满每块盘就崩时，
     // F_生效 高于某块盘上最新的有效根；恢复择根不看 F，丢掉另一块盘之后择的就是那块盘上最新的根，它引用的单元释放代可以高于它、不高于 F_生效
     // （判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「形 R」那一行）。于是当场可用的只到
     // reclaim_floor(min(F_生效, m), 环里最旧有效根)（RGall），m 是每块盘上最新有效根的 txg 取最小（被抛弃的根不算，与卸载那一格同一处定义
@@ -3727,7 +3782,7 @@ enum ReuseOfTheSlotsReclaimedByTheSequence {
 /// - 有一块盘上一条有效根都没读出来：交回 `None`（不按 txg 0 去比，说清是哪一格），两格各自处置：
 ///   卸载那一格这一串回收的槽整截扣到生效；重建那一格当场只放到环里最旧有效根（RGall；挂载时读坏的槽钉在它可能住过的最旧 txg 并进，
 ///   与回收门槛同一个读数），其余扣到这次挂载的暖机落满每块盘（`rebuilt_allocator`，用户 2026-10-04 选甲，
-///   `research/prompts/m3-batch16-code-r1-forks.md` 第 1 行）。
+///   `research/prompts/m3-batch16-code-r1-forks.md`「甲 只放到窗口里最旧有效根」那一行）。
 fn newest_valid_root_txg_on_every_device(
     roots_on_their_devices: &[(DeviceIdentity, RootRecord)],
     table: &InstanceTableRecords,
@@ -7460,6 +7515,14 @@ pub enum RollbackError {
         device: DeviceIdentity,
         slot: SlotNumber,
     },
+    /// 回退目标引用的一个隔离单元（现行账里每块盘上都仍分配、分配代相同，现行那一版却不引用它：释放之前读盘核出过一份对不上、
+    /// 两份都留在已分配，D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）在这块盘上的那一份读不出或整单元 CRC-32C 与映射条目里的对不上
+    /// （D23（journal 的角色与格式） 已定项 14「在任何写之前拒」那一格，用户 2026-10-12 定 C572「隔离单元也进逐盘验」）。
+    IsolatedUnitCopyUnreadableOrMismatched {
+        unit: TransactionUnit,
+        device: DeviceIdentity,
+        slot: SlotNumber,
+    },
     /// 释放现行那一版不再被引用的用户可见单元时，释放判定路径报错（映射里查不到、条目切不动、两条位置条目不同槽、这块盘的记录
     /// 不在册 / 已释放 / 跨度不对、位置项指池外的盘或两条指同一块盘）：`cause` 原样是发布路径的那一种。
     CurrentVersionUnitNotReleasable { cause: PublishError },
@@ -7660,9 +7723,14 @@ fn rollback_candidate<Reader: PoolReader>(
 
 /// 回退目标引用的用户可见单元，逐个逐盘对现行那一版的账（D23（journal 的角色与格式） 已定项 14「在任何写之前拒」前两样与「复活」）：
 /// 每块盘上起点是它的槽、跨度相同的那条记录——没有就拒；仍分配而分配代与回退目标那一版账里的不同就拒；已释放的进复活集
-/// （分配代写回目标那一版账里的）；仍分配、分配代相同的是新旧两版共用的单元，不动。交回复活集，与复活集里有份的单元（逐盘验要读它们每一份）。
+/// （分配代写回目标那一版账里的）；仍分配、分配代相同的不动。交回复活集、与复活集里有份的单元（逐盘验要读它们每一份），
+/// 与隔离单元：每块盘上都仍分配、分配代相同，而现行那一版不引用它（它的位置项的槽里没有一个是现行那一版的单元的槽，
+/// `slots_of_the_units_of_the_current_version`）——现行账里仍分配而现行那一版不引用，只有释放之前读盘核出一份对不上、两份都留在已分配
+/// 这一种来路（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1，`PoolAllocator::release_leaving_the_record_allocated_on`）。
+/// 隔离单元同样逐盘验（D23 已定项 14「在任何写之前拒」那一格「R_old 引用、cur 的账里仍分配而同分配代的单元里被隔离的那些……同样逐盘验」，
+/// 用户 2026-10-12 定 C572「隔离单元也进逐盘验」）；现行那一版引用着的才是新旧两版共用的单元，分配器不会发出去，不验。
 ///
-/// 循环：外层按目标的用户可见单元（轮数 = 单元数），内层按它的两条位置项；跨轮只往两张清单里追加；提前出口都是拒。
+/// 循环：外层按目标的用户可见单元（轮数 = 单元数），内层按它的两条位置项；跨轮只往三张清单里追加；提前出口都是拒。
 ///
 /// # Errors
 /// `UserVisibleUnitWithoutItsRecordInTheCurrentAccount`、`UserVisibleUnitStillAllocatedUnderAnotherGeneration`；
@@ -7672,6 +7740,7 @@ fn resurrection_against_the_current_account(
     target_version: &TransactionOutput,
     target_units: &[(TransactionUnit, [LocationEntry; 2])],
     current_account: &PoolAllocator,
+    slots_of_the_units_of_the_current_version: &BTreeSet<SlotNumber>,
 ) -> Result<Resurrection, RollbackError> {
     let pool_devices: Vec<DeviceIdentity> = current_account
         .devices
@@ -7684,6 +7753,7 @@ fn resurrection_against_the_current_account(
     };
     let mut resurrected = Vec::new();
     let mut units_to_check_on_every_device = Vec::new();
+    let mut isolated_units_to_check_on_every_device = Vec::new();
     for (unit, locations) in target_units {
         refuse_locations_that_do_not_name_two_pool_devices(*unit, locations, &pool_devices)
             .map_err(|_refusal| {
@@ -7740,23 +7810,53 @@ fn resurrection_against_the_current_account(
                         },
                     );
                 }
-                Some(_shared_by_both_versions) => {}
+                Some(_still_allocated_under_the_same_generation) => {}
             }
         }
+        let is_referenced_by_the_current_version = locations
+            .iter()
+            .any(|location| slots_of_the_units_of_the_current_version.contains(&location.slot));
         if is_resurrected_on_some_device {
             units_to_check_on_every_device.push((*unit, *locations));
+        } else if !is_referenced_by_the_current_version {
+            isolated_units_to_check_on_every_device.push((*unit, *locations));
         }
     }
     Ok(Resurrection {
         resurrected,
         units_to_check_on_every_device,
+        isolated_units_to_check_on_every_device,
     })
 }
 
-/// 复活那一判交回的：复活集（逐盘），与复活集里有份的单元（逐盘验要读它们每一份）。
+/// 复活那一判交回的：复活集（逐盘），复活集里有份的单元（逐盘验要读它们每一份），与回退目标引用的隔离单元（同样逐盘验）。
 struct Resurrection {
     resurrected: Vec<ResurrectedCopy>,
     units_to_check_on_every_device: Vec<(TransactionUnit, [LocationEntry; 2])>,
+    isolated_units_to_check_on_every_device: Vec<(TransactionUnit, [LocationEntry; 2])>,
+}
+
+/// 回退目标引用的隔离单元在每块盘上的那一份都要读得出、整单元 CRC-32C 与映射条目里的对得上（D23（journal 的角色与格式） 已定项 14
+/// 「在任何写之前拒」那一格，用户 2026-10-12 定 C572「隔离单元也进逐盘验」）：隔离就是释放之前读盘核出过一份对不上，回退之后新根又引用它，
+/// 不验就把读不出的那一份当成两份都在发出去。读法与复活集那一判相同（[`first_copy_that_does_not_read_back`]），只是拒因不同。
+/// 只读一次，不重读：读错只让这次回退被拒，不留下任何状态。
+///
+/// # Errors
+/// `IsolatedUnitCopyUnreadableOrMismatched`，按单元次序、位置项次序报第一份。
+fn every_copy_of_the_isolated_units_the_target_references_reads_back<
+    Reader: PoolReader + ?Sized,
+>(
+    reader: &Reader,
+    units: &[(TransactionUnit, [LocationEntry; 2])],
+) -> Result<(), RollbackError> {
+    match first_copy_that_does_not_read_back(reader, units) {
+        None => Ok(()),
+        Some((unit, location)) => Err(RollbackError::IsolatedUnitCopyUnreadableOrMismatched {
+            unit,
+            device: location.device,
+            slot: location.slot,
+        }),
+    }
 }
 
 /// 复活集里每个单元在每块盘上的那一份都要读得出、整单元 CRC-32C 与映射条目里的对得上（D23（journal 的角色与格式） 已定项 14
@@ -7769,26 +7869,37 @@ fn every_copy_of_the_resurrected_units_reads_back<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     units: &[(TransactionUnit, [LocationEntry; 2])],
 ) -> Result<(), RollbackError> {
-    for (unit, locations) in units {
-        let unit_bytes = usize::try_from(unit.span_slots() * SLOT_BYTES).expect("一个单元两槽以内");
-        for location in locations {
-            let is_intact = reader
-                .read(
-                    location.device,
-                    location.slot.to_device_offset(),
-                    unit_bytes,
-                )
-                .is_some_and(|copy| crc32_castagnoli(&copy) == location.unit_checksum);
-            if !is_intact {
-                return Err(RollbackError::ResurrectedUnitCopyUnreadableOrMismatched {
-                    unit: *unit,
-                    device: location.device,
-                    slot: location.slot,
-                });
-            }
-        }
+    match first_copy_that_does_not_read_back(reader, units) {
+        None => Ok(()),
+        Some((unit, location)) => Err(RollbackError::ResurrectedUnitCopyUnreadableOrMismatched {
+            unit,
+            device: location.device,
+            slot: location.slot,
+        }),
     }
-    Ok(())
+}
+
+/// `units` 里按单元次序、位置项次序第一份读不出、或整单元 CRC-32C 与位置项里的校验和对不上的那一份；都对得上交 `None`。
+/// 每份只读一次。
+fn first_copy_that_does_not_read_back<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    units: &[(TransactionUnit, [LocationEntry; 2])],
+) -> Option<(TransactionUnit, LocationEntry)> {
+    units.iter().find_map(|(unit, locations)| {
+        let unit_bytes = usize::try_from(unit.span_slots() * SLOT_BYTES).expect("一个单元两槽以内");
+        locations
+            .iter()
+            .find(|location| {
+                !reader
+                    .read(
+                        location.device,
+                        location.slot.to_device_offset(),
+                        unit_bytes,
+                    )
+                    .is_some_and(|copy| crc32_castagnoli(&copy) == location.unit_checksum)
+            })
+            .map(|location| (*unit, *location))
+    })
 }
 
 /// `ring_roots` 里每一条带文件的根，它那一版记账里的 inode 号水位，取最大；从盘上重建那一版读不出的根不算（「环里读得出的根」）。
@@ -8095,16 +8206,25 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
             },
         }
     })?;
+    // 现行那一版的单元的槽（两盘同槽）：现行账里仍分配、同分配代而不在其中的，是隔离单元（[`resurrection_against_the_current_account`]）。
+    let slots_of_the_units_of_the_current_version: BTreeSet<SlotNumber> =
+        current.units.iter().map(|unit| unit.slot).collect();
     let Resurrection {
         resurrected,
         units_to_check_on_every_device,
+        isolated_units_to_check_on_every_device,
     } = resurrection_against_the_current_account(
         target,
         &target_version,
         &target_units,
         allocator,
+        &slots_of_the_units_of_the_current_version,
     )?;
     every_copy_of_the_resurrected_units_reads_back(&*devices, &units_to_check_on_every_device)?;
+    every_copy_of_the_isolated_units_the_target_references_reads_back(
+        &*devices,
+        &isolated_units_to_check_on_every_device,
+    )?;
     // 释放集：cur 引用的用户可见单元里，落点不在 R_old 引用的那几个里的（在的那几个上面判过同槽同分配代，是两版共用的）。
     let slots_referenced_by_the_target: BTreeSet<(DeviceIdentity, SlotNumber)> = target_units
         .iter()
@@ -9547,3 +9667,490 @@ mod tree_identifier_watermark_of_the_rollback_tests {
         );
     }
 }
+
+/// 影子账按槽认豁免与判重（D23（journal 的角色与格式） 已定项 14「只隔离其中只被被抛弃根引用的槽」，判决
+/// `research/prompts/m3-c554-c572-r1-main-verification.md` 第二节 B3 那一行）：攻方 `research/prompts/m3-c554-c572-r1-opus-output.md`
+/// 「B3」一节点名的五处各一条会红的用例，前三种形照攻方原样（同一起点、跨度 1 与跨度 2 的两份落点），候选根、现行账两份豁免集合另各一条
+/// （跨度 2 的豁免落点罩住被抛弃根落点的后一槽）。直调被测函数；两块盘照 mkfs 写成，候选根与「从盘上读账」的被抛弃根读的是 mkfs 写下的那一版，
+/// 其余几条的落点都在内存里给（根环表里记着的、根槽读坏记着的、水位之上没施加的记录点名的），不读盘。
+#[cfg(test)]
+mod shadow_ledger_exempts_and_isolates_slot_by_slot_tests {
+    use super::{
+        isolate_slots_referenced_only_by_abandoned_roots, BeforeTheOneReread,
+        PlacementsOfAbandonedRoots,
+    };
+    use crate::address::{
+        CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
+    };
+    use crate::allocator::{
+        AllocationRecord, DeviceFreeMap, PlacementOnDevice, PoolAllocator, UnitAreaStart,
+    };
+    use crate::block_device::{
+        BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+    };
+    use crate::make_filesystem::{make_filesystem, MakeFilesystemParameters};
+    use crate::root_record::RootRecord;
+    use crate::root_ring::RootRingSlotsPerRegion;
+    use crate::system_configuration::SystemImmutableSizes;
+    use std::collections::BTreeMap;
+
+    /// 每块盘的字节数：与 harness 档建池同一个尺寸（4 GiB，journal 环取默认的 768 MiB），只在内存里按页存写过的字节。
+    const DEVICE_BYTES: u64 = 4 << 30;
+    /// 稀疏盘一页的字节数。
+    const PAGE_BYTES: u64 = 4096;
+    /// 现行账、被抛弃根、水位之上的记录在这几条用例里引用的起点：单元区起点之后第 10 槽，mkfs 写下的实例表（起点、两槽）与树表（起点 + 2、一槽）之外。
+    const START_SLOT_AFTER_THE_UNIT_AREA_START: u64 = 10;
+    /// 被抛弃根的那一版、现行账记录的分配代：这几条用例只看豁免与判重，代取多少都一样。
+    const ALLOCATION_GENERATION: CheckpointTxg = CheckpointTxg(5);
+
+    /// 只在内存里的盘：写过的页才存，没写过的读回全零；越过末尾报越界。
+    struct SparseDeviceInMemory {
+        pages: BTreeMap<u64, Vec<u8>>,
+    }
+
+    impl SparseDeviceInMemory {
+        fn refuse_past_the_end(
+            offset: DeviceOffsetInBytes,
+            length: u64,
+        ) -> Result<(), BlockDeviceError> {
+            if offset.0 + length > DEVICE_BYTES {
+                return Err(BlockDeviceError::OutOfRange {
+                    offset,
+                    length,
+                    device_size: DEVICE_BYTES,
+                });
+            }
+            Ok(())
+        }
+    }
+
+    impl BlockDevice for SparseDeviceInMemory {
+        fn read_at(
+            &self,
+            offset: DeviceOffsetInBytes,
+            buffer: &mut [u8],
+        ) -> Result<(), BlockDeviceError> {
+            let length = u64::try_from(buffer.len()).expect("长度装得进 u64");
+            Self::refuse_past_the_end(offset, length)?;
+            for (index_in_buffer, byte) in buffer.iter_mut().enumerate() {
+                let address = offset.0 + u64::try_from(index_in_buffer).expect("下标装得进 u64");
+                let index_in_page =
+                    usize::try_from(address % PAGE_BYTES).expect("页内偏移装得进 usize");
+                *byte = self
+                    .pages
+                    .get(&(address / PAGE_BYTES))
+                    .map_or(0, |page| page[index_in_page]);
+            }
+            Ok(())
+        }
+        fn write_at(
+            &mut self,
+            offset: DeviceOffsetInBytes,
+            bytes: &[u8],
+            _durability: WriteDurability,
+        ) -> Result<(), BlockDeviceError> {
+            let length = u64::try_from(bytes.len()).expect("长度装得进 u64");
+            Self::refuse_past_the_end(offset, length)?;
+            for (index_in_bytes, byte) in bytes.iter().enumerate() {
+                let address = offset.0 + u64::try_from(index_in_bytes).expect("下标装得进 u64");
+                let index_in_page =
+                    usize::try_from(address % PAGE_BYTES).expect("页内偏移装得进 usize");
+                self.pages
+                    .entry(address / PAGE_BYTES)
+                    .or_insert_with(|| vec![0; usize::try_from(PAGE_BYTES).expect("4096")])
+                    [index_in_page] = *byte;
+            }
+            Ok(())
+        }
+        fn write_zeroes_at(
+            &mut self,
+            offset: DeviceOffsetInBytes,
+            length: u64,
+        ) -> Result<(), BlockDeviceError> {
+            Self::refuse_past_the_end(offset, length)?;
+            assert!(
+                offset.0.is_multiple_of(PAGE_BYTES) && length.is_multiple_of(PAGE_BYTES),
+                "mkfs 清零的段按 4096 对齐：系统配置槽距、根环区域与 journal 环都是 4096 的整数倍"
+            );
+            let first_page = offset.0 / PAGE_BYTES;
+            let pages_zeroed: Vec<u64> = self
+                .pages
+                .range(first_page..first_page + length / PAGE_BYTES)
+                .map(|(page, _)| *page)
+                .collect();
+            for page in pages_zeroed {
+                self.pages.remove(&page);
+            }
+            Ok(())
+        }
+        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+            Ok(())
+        }
+        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+            PhysicalBlockSizeInBytes(512)
+        }
+        fn size_in_bytes(&self) -> u64 {
+            DEVICE_BYTES
+        }
+    }
+
+    /// mkfs 写成的两块盘、它的单元区起点与它写下的第 0 代根。
+    struct FormattedPool {
+        devices: Vec<(DeviceIdentity, SparseDeviceInMemory)>,
+        unit_area_start: UnitAreaStart,
+        make_filesystem_root: RootRecord,
+    }
+
+    fn formatted_pool() -> FormattedPool {
+        let parameters = MakeFilesystemParameters {
+            filesystem_identifier: [7; 16],
+            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+            geometry: SystemImmutableSizes {
+                physical_block_size: 512,
+                minimum_input_output_bytes: 512,
+                fixed_structure_slot_spacing: 4096,
+                journal_ring_bytes: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
+                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
+            },
+        };
+        let mut devices: Vec<(DeviceIdentity, SparseDeviceInMemory)> =
+            [DeviceIdentity(0), DeviceIdentity(1)]
+                .into_iter()
+                .map(|identity| {
+                    (
+                        identity,
+                        SparseDeviceInMemory {
+                            pages: BTreeMap::new(),
+                        },
+                    )
+                })
+                .collect();
+        let output =
+            make_filesystem(&parameters, &mut devices).expect("4 GiB 两盘照默认几何建得成");
+        FormattedPool {
+            devices,
+            unit_area_start: output.unit_area_start,
+            make_filesystem_root: output.root,
+        }
+    }
+
+    fn slot_after_the_unit_area_start(pool: &FormattedPool, slots_after: u64) -> SlotNumber {
+        SlotNumber(pool.unit_area_start.slot().0 + slots_after)
+    }
+
+    fn placement_on_device_zero(slot: SlotNumber, span: u64) -> PlacementOnDevice {
+        PlacementOnDevice {
+            device: DeviceIdentity(0),
+            slot,
+            span,
+        }
+    }
+
+    fn allocated_record_on_device_zero(slot: SlotNumber, span_slots: u16) -> AllocationRecord {
+        AllocationRecord {
+            device: DeviceIdentity(0),
+            slot,
+            span_slots,
+            generation: ALLOCATION_GENERATION,
+            is_released: false,
+        }
+    }
+
+    /// 一条被抛弃根：实例代号与 txg 各取一个 mkfs 没写过的，别的字段照 mkfs 的第 0 代根（它的落点从根环表里记着的那一份取，不读这些指针）。
+    fn abandoned_root(pool: &FormattedPool, instance: u32, checkpoint_txg: u64) -> RootRecord {
+        let make_filesystem_root = pool.make_filesystem_root;
+        RootRecord {
+            filesystem_identifier: make_filesystem_root.filesystem_identifier,
+            unmount_marker: make_filesystem_root.unmount_marker,
+            instance: InstanceGeneration(instance),
+            checkpoint_txg: CheckpointTxg(checkpoint_txg),
+            tree_table: make_filesystem_root.tree_table,
+            tree_identifier_watermark: make_filesystem_root.tree_identifier_watermark,
+            rollback_floor: make_filesystem_root.rollback_floor,
+            instance_table: make_filesystem_root.instance_table,
+            mapping_root: make_filesystem_root.mapping_root,
+            allocation_record_tree_root: make_filesystem_root.allocation_record_tree_root,
+        }
+    }
+
+    /// 喂给被测函数的一次影子账：根环里的根（`abandoned_instances` 里实例代号的算被抛弃）、现行账、根环表里记着的被抛弃根的落点、
+    /// 根槽这一遍读坏的被抛弃根记着的落点、水位之上没施加的记录点名的落点。
+    struct ShadowLedgerInputs<'inputs> {
+        roots: &'inputs [RootRecord],
+        abandoned_instances: &'inputs [InstanceGeneration],
+        current_records: &'inputs [AllocationRecord],
+        recorded_placements_of_abandoned_roots:
+            &'inputs BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<PlacementOnDevice>>,
+        placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad:
+            &'inputs [PlacementOnDevice],
+        placements_named_by_unapplied_journal_records: &'inputs [PlacementOnDevice],
+    }
+
+    /// 在一个只按现行账置好分配位的分配器上跑一次影子账（下界 F 取 0），交回跑完的分配器。
+    fn allocator_after_the_shadow_ledger(
+        pool: &FormattedPool,
+        inputs: &ShadowLedgerInputs<'_>,
+    ) -> PoolAllocator {
+        let mut allocator = PoolAllocator::new(
+            pool.devices
+                .iter()
+                .map(|(identity, _)| {
+                    DeviceFreeMap::with_unit_area_start(
+                        *identity,
+                        DEVICE_BYTES,
+                        pool.unit_area_start,
+                    )
+                })
+                .collect(),
+        );
+        for record in inputs
+            .current_records
+            .iter()
+            .filter(|record| !record.is_released)
+        {
+            allocator
+                .devices
+                .iter_mut()
+                .find(|device_map| device_map.device == record.device)
+                .expect("用例里的记录都在盘 0 上")
+                .mark_allocated(record.slot, u64::from(record.span_slots));
+        }
+        let is_abandoned = |root: &RootRecord| inputs.abandoned_instances.contains(&root.instance);
+        isolate_slots_referenced_only_by_abandoned_roots(
+            &pool.devices,
+            pool.unit_area_start,
+            &mut allocator,
+            inputs.roots,
+            &is_abandoned,
+            CheckpointTxg(0),
+            inputs.current_records,
+            inputs.placements_named_by_unapplied_journal_records,
+            PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
+                inputs.recorded_placements_of_abandoned_roots,
+            ),
+            &mut BeforeTheOneReread::RereadImmediately,
+            inputs.placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad,
+        )
+        .expect("mkfs 写下的那一版读得出、别的落点都在内存里：不拒");
+        allocator
+    }
+
+    /// 盘 0 上 `slot` 那一槽：(隔离着, 发得出去)。
+    fn isolated_and_free_on_device_zero(
+        allocator: &PoolAllocator,
+        slot: SlotNumber,
+    ) -> (bool, bool) {
+        let device_map = &allocator.devices[0];
+        (device_map.is_isolated(slot), device_map.is_free(slot))
+    }
+
+    /// 攻方形一：两条被抛弃根同一起点 s，A1 引用 (s, 跨 1)、A2 引用 (s, 跨 2)。按起点槽判重时窄的先读出就把宽的整份判重掉、s+1 发得出去，
+    /// 宽的先读出两槽都罩着——结局随根环读出次序变。按槽判重两种次序结局相同：两槽都隔离着。
+    #[test]
+    fn two_abandoned_roots_sharing_a_start_slot_isolate_both_slots_of_the_wider_placement_in_either_read_order(
+    ) {
+        let pool = formatted_pool();
+        let start = slot_after_the_unit_area_start(&pool, START_SLOT_AFTER_THE_UNIT_AREA_START);
+        let narrow_root = abandoned_root(&pool, 1, 8);
+        let wide_root = abandoned_root(&pool, 2, 12);
+        let recorded = BTreeMap::from([
+            (
+                (narrow_root.instance, narrow_root.checkpoint_txg),
+                vec![placement_on_device_zero(start, 1)],
+            ),
+            (
+                (wide_root.instance, wide_root.checkpoint_txg),
+                vec![placement_on_device_zero(start, 2)],
+            ),
+        ]);
+        let outcome_in_order = |roots: &[RootRecord]| {
+            let allocator = allocator_after_the_shadow_ledger(
+                &pool,
+                &ShadowLedgerInputs {
+                    roots,
+                    abandoned_instances: &[narrow_root.instance, wide_root.instance],
+                    current_records: &[],
+                    recorded_placements_of_abandoned_roots: &recorded,
+                    placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
+                    placements_named_by_unapplied_journal_records: &[],
+                },
+            );
+            [
+                isolated_and_free_on_device_zero(&allocator, start),
+                isolated_and_free_on_device_zero(&allocator, SlotNumber(start.0 + 1)),
+            ]
+        };
+        let narrow_first = outcome_in_order(&[narrow_root, wide_root]);
+        let wide_first = outcome_in_order(&[wide_root, narrow_root]);
+        assert_eq!(
+            narrow_first,
+            [(true, false), (true, false)],
+            "窄的先读出：s 与 s+1 都隔离着、都发不出去（按起点槽判重时 s+1 发得出去）"
+        );
+        assert_eq!(wide_first, narrow_first, "宽的先读出与窄的先读出结局相同");
+    }
+
+    /// 攻方形二：现行账里 (s, 跨 1) 仍分配，被抛弃根引用 (s, 跨 2)。s 被现行账引用、不隔离（分配着、发不出去）；s+1 只被被抛弃根引用，隔离。
+    /// 按起点槽豁免时整份跳过，s+1 发得出去。
+    #[test]
+    fn a_current_account_record_at_the_same_start_slot_leaves_the_second_slot_of_an_abandoned_roots_wider_placement_isolated(
+    ) {
+        let pool = formatted_pool();
+        let start = slot_after_the_unit_area_start(&pool, START_SLOT_AFTER_THE_UNIT_AREA_START);
+        let root = abandoned_root(&pool, 1, 8);
+        let recorded = BTreeMap::from([(
+            (root.instance, root.checkpoint_txg),
+            vec![placement_on_device_zero(start, 2)],
+        )]);
+        let allocator = allocator_after_the_shadow_ledger(
+            &pool,
+            &ShadowLedgerInputs {
+                roots: &[root],
+                abandoned_instances: &[root.instance],
+                current_records: &[allocated_record_on_device_zero(start, 1)],
+                recorded_placements_of_abandoned_roots: &recorded,
+                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
+                placements_named_by_unapplied_journal_records: &[],
+            },
+        );
+        assert_eq!(
+            isolated_and_free_on_device_zero(&allocator, start),
+            (false, false),
+            "s 被现行账引用：不隔离、分配着"
+        );
+        assert_eq!(
+            isolated_and_free_on_device_zero(&allocator, SlotNumber(start.0 + 1)),
+            (true, false),
+            "s+1 只被被抛弃根引用：隔离、发不出去"
+        );
+    }
+
+    /// 攻方形三：水位之上没施加的记录点名 (s, 跨 2)，现行账里 (s, 跨 1) 仍分配。C 那一圈只豁免现行账引用的槽：s 不隔离，s+1 隔离。
+    #[test]
+    fn a_current_account_record_at_the_same_start_slot_leaves_the_second_slot_of_a_placement_named_by_unapplied_records_isolated(
+    ) {
+        let pool = formatted_pool();
+        let start = slot_after_the_unit_area_start(&pool, START_SLOT_AFTER_THE_UNIT_AREA_START);
+        let allocator = allocator_after_the_shadow_ledger(
+            &pool,
+            &ShadowLedgerInputs {
+                roots: &[],
+                abandoned_instances: &[],
+                current_records: &[allocated_record_on_device_zero(start, 1)],
+                recorded_placements_of_abandoned_roots: &BTreeMap::new(),
+                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
+                placements_named_by_unapplied_journal_records: &[placement_on_device_zero(
+                    start, 2,
+                )],
+            },
+        );
+        assert_eq!(
+            isolated_and_free_on_device_zero(&allocator, start),
+            (false, false),
+            "s 被现行账引用：不隔离、分配着"
+        );
+        assert_eq!(
+            isolated_and_free_on_device_zero(&allocator, SlotNumber(start.0 + 1)),
+            (true, false),
+            "s+1 只被水位之上没施加的记录点名：隔离、发不出去"
+        );
+    }
+
+    /// 根槽这一遍读坏的被抛弃根记着 (s, 跨 2)，现行账里 (s, 跨 1) 仍分配（攻方点名的第四处，形二搬到这一圈）：s 不隔离，s+1 隔离。
+    #[test]
+    fn a_current_account_record_at_the_same_start_slot_leaves_the_second_slot_recorded_for_an_abandoned_root_whose_ring_slot_read_bad_isolated(
+    ) {
+        let pool = formatted_pool();
+        let start = slot_after_the_unit_area_start(&pool, START_SLOT_AFTER_THE_UNIT_AREA_START);
+        let allocator = allocator_after_the_shadow_ledger(
+            &pool,
+            &ShadowLedgerInputs {
+                roots: &[],
+                abandoned_instances: &[],
+                current_records: &[allocated_record_on_device_zero(start, 1)],
+                recorded_placements_of_abandoned_roots: &BTreeMap::new(),
+                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[
+                    placement_on_device_zero(start, 2),
+                ],
+                placements_named_by_unapplied_journal_records: &[],
+            },
+        );
+        assert_eq!(
+            isolated_and_free_on_device_zero(&allocator, start),
+            (false, false),
+            "s 被现行账引用：不隔离、分配着"
+        );
+        assert_eq!(
+            isolated_and_free_on_device_zero(&allocator, SlotNumber(start.0 + 1)),
+            (true, false),
+            "s+1 只被那条被抛弃根引用：隔离、发不出去"
+        );
+    }
+
+    /// 现行账那一份豁免集合按槽收（攻方点名的第一处）：现行账里 (起点 + 1, 跨 2) 仍分配，被抛弃根是从盘上读账的 mkfs 那一版
+    /// （实例表 (起点, 跨 2)、树表 (起点 + 2, 跨 1)，两块盘各一份；根环表里没记着它，整段隔离、分配位置着的槽也隔离）。
+    /// 盘 0 上起点只被被抛弃根引用、隔离；起点 + 1 与起点 + 2 被现行账那条记录罩着、不隔离。只按记录的起点收时起点 + 2 不在豁免里、被隔离。
+    #[test]
+    fn a_current_account_record_exempts_every_slot_it_covers_from_the_placements_of_an_abandoned_root_read_from_its_account(
+    ) {
+        let pool = formatted_pool();
+        let unit_area_start = pool.unit_area_start.slot();
+        let root = abandoned_root(&pool, 1, 8);
+        let allocator = allocator_after_the_shadow_ledger(
+            &pool,
+            &ShadowLedgerInputs {
+                roots: &[root],
+                abandoned_instances: &[root.instance],
+                current_records: &[allocated_record_on_device_zero(
+                    SlotNumber(unit_area_start.0 + 1),
+                    2,
+                )],
+                recorded_placements_of_abandoned_roots: &BTreeMap::new(),
+                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
+                placements_named_by_unapplied_journal_records: &[],
+            },
+        );
+        let isolated_on_device_zero: Vec<bool> = (0..3)
+            .map(|slots_after| {
+                allocator.devices[0].is_isolated(SlotNumber(unit_area_start.0 + slots_after))
+            })
+            .collect();
+        assert_eq!(
+            isolated_on_device_zero,
+            vec![true, false, false],
+            "盘 0：起点只被被抛弃根引用、隔离；起点 + 1、起点 + 2 被现行账 (起点 + 1, 跨 2) 罩着、不隔离"
+        );
+    }
+
+    /// 候选根那一份豁免集合按槽收（攻方点名的第二处）：mkfs 那一版是候选根（按实例表有效、txg 0 ≥ F = 0），它引用实例表 (起点, 跨 2)；
+    /// 一条被抛弃根引用 (起点 + 1, 跨 1)。起点 + 1 被候选根罩着、不隔离；只按候选根落点的起点收时它不在豁免里、被隔离。
+    #[test]
+    fn a_candidate_roots_two_slot_placement_exempts_its_second_slot_from_an_abandoned_roots_placement(
+    ) {
+        let pool = formatted_pool();
+        let second_slot_of_the_instance_table = slot_after_the_unit_area_start(&pool, 1);
+        let root = abandoned_root(&pool, 1, 8);
+        let recorded = BTreeMap::from([(
+            (root.instance, root.checkpoint_txg),
+            vec![placement_on_device_zero(
+                second_slot_of_the_instance_table,
+                1,
+            )],
+        )]);
+        let allocator = allocator_after_the_shadow_ledger(
+            &pool,
+            &ShadowLedgerInputs {
+                roots: &[pool.make_filesystem_root, root],
+                abandoned_instances: &[root.instance],
+                current_records: &[],
+                recorded_placements_of_abandoned_roots: &recorded,
+                placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad: &[],
+                placements_named_by_unapplied_journal_records: &[],
+            },
+        );
+        assert!(
+            !allocator.devices[0].is_isolated(second_slot_of_the_instance_table),
+            "起点 + 1 被候选根（mkfs 那一版）的实例表 (起点, 跨 2) 罩着：不隔离"
+        );
+    }
+}
```

## 二、补丁里 harness 三份（`code-b2-mount/patch/crates.patch` 原样摘出）

```diff
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -3146,6 +3146,9 @@
         RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. } => {
             "ResurrectedUnitCopyUnreadableOrMismatched".to_string()
         }
+        RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. } => {
+            "IsolatedUnitCopyUnreadableOrMismatched".to_string()
+        }
         RollbackError::CurrentVersionUnitNotReleasable { cause } => {
             format!("CurrentVersionUnitNotReleasable({})", publish_error_member(cause))
         }
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -794,6 +794,7 @@
         | RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount { .. }
         | RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration { .. }
         | RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. }
+        | RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. }
         // 判候选集读根环或算 F_生效 重读一次仍读坏（实审 A3b Q7）：读路径上注入故障、或同一个进程里一次根槽写被说谎的设备吞掉
         // （那一槽这个进程知道住着根、盘上却不是它写的那一份）才有，与 `MountError::Recovery(_)` 同一处置。
         | RollbackError::CandidateJudgementStillUnreadableAfterOneReread(_) => {
@@ -985,6 +986,7 @@
         | RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount { .. }
         | RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration { .. }
         | RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. }
+        | RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. }
         | RollbackError::CurrentVersionUnitNotReleasable { .. }
         | RollbackError::Publish(_) => None,
     }
--- /dev/null
+++ b/crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs
@@ -0,0 +1,215 @@
+//! 管理员回退的目标引用一个隔离单元（C572（回退目标引用隔离单元不进逐盘验））：D23（journal 的角色与格式） 已定项 14「在任何写之前拒」那一格——
+//! R_old 引用、cur 的账里仍分配而同分配代的单元里被隔离的那些（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1：读盘核出一份对不上、
+//! 两份都留在已分配）同样逐盘验，有一份读不出或对不上就拒（用户 2026-10-12 定「隔离单元也进逐盘验」）。
+//!
+//! 历史：新池新建文件 A（实例 1，txg 3）；盘 1 上 A 的数据单元那一份改坏；覆盖写 B（txg 4）换掉它，释放之前读盘核出盘 1 那一份对不上，
+//! 两块盘那条记录都留在已分配、分配代照旧 3（隔离）；挂着时回退到 A。A 引用的数据单元在 B 的账里仍分配、同分配代，B 不引用它。
+//! - 盘 1 那一份仍坏：在任何写之前拒成 `IsolatedUnitCopyUnreadableOrMismatched`（盘 1、那个槽），盘上逐字节不变、现行版本与分配器不动。
+//!   不验它（改之前的样子）时回退做成，新根引用一个只有一份读得出的单元。
+//! - 盘 1 那一份写回原样（两份都对得上）：回退照做，冷启动读回 A 的内容——逐盘验只拒对不上的，不拒隔离本身。
+
+mod common;
+
+use common::{
+    build_pool, disk_snapshot, file_content, parameters, publish_overwrite_in_process, BuiltPool,
+    Recorded, FIXED_WRITE_TIME_SECONDS,
+};
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
+use singlefs_core::block_device::{BlockDevice, WriteDurability};
+use singlefs_core::mount::{
+    roll_back_by_a_forward_publish, RollbackError, RollbackTarget, RolledBack,
+};
+use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
+use singlefs_core::transaction::{
+    CopyQuarantinedAfterReleaseChecksumMismatch, QuarantinedCopyReading, TransactionUnit,
+};
+
+/// 覆盖写 B 的内容字节数：与 A 的长度不同，冷启动读回哪一版分得开。
+const SECOND_CONTENT_BYTES: usize = 2700;
+/// 覆盖写 B 比 A 晚这么多秒。
+const SECONDS_BETWEEN_THE_TWO_WRITES: u64 = 60;
+
+fn second_content() -> Vec<u8> {
+    (0..SECOND_CONTENT_BYTES)
+        .map(|index| u8::try_from((index * 11 + 9) % 247).expect("小于 256"))
+        .collect()
+}
+
+/// A 那一版的根：回退目标。
+fn first_root() -> RollbackTarget {
+    RollbackTarget {
+        instance: InstanceGeneration(1),
+        checkpoint_txg: CheckpointTxg(3),
+    }
+}
+
+fn read_copy_at(
+    devices: &[(DeviceIdentity, Recorded)],
+    device: DeviceIdentity,
+    slot: SlotNumber,
+) -> Vec<u8> {
+    let (_, block_device) = devices
+        .iter()
+        .find(|(identity, _)| *identity == device)
+        .expect("池里有这块盘");
+    let mut bytes = vec![0u8; usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768")];
+    block_device
+        .read_at(slot.to_device_offset(), &mut bytes)
+        .expect("读一份数据单元");
+    bytes
+}
+
+fn write_copy_at(
+    devices: &mut [(DeviceIdentity, Recorded)],
+    device: DeviceIdentity,
+    slot: SlotNumber,
+    bytes: &[u8],
+) {
+    let (_, block_device) = devices
+        .iter_mut()
+        .find(|(identity, _)| *identity == device)
+        .expect("池里有这块盘");
+    block_device
+        .write_at(slot.to_device_offset(), bytes, WriteDurability::Plain)
+        .expect("写一份数据单元");
+}
+
+/// 走到「B 发布完、A 的数据单元隔离着」那一刻：交回池（`output` 是 B）、A 的数据单元的槽、盘 1 上那一份改坏之前的字节。
+/// 顺带钉住前提：B 隔离了 A 的数据单元两块盘那两份（盘 1 对不上、盘 0 对得上一起留下），两条记录都仍分配、分配代 3，B 不引用那个槽。
+fn pool_with_the_data_unit_of_the_first_version_isolated(
+    tag: &str,
+) -> (BuiltPool, SlotNumber, Vec<u8>) {
+    let mut pool = build_pool(tag);
+    let first = pool.output.clone();
+    let data_slot = first.data_pointers[0].locations[0].slot;
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let intact_copy_on_device_one = read_copy_at(devices, DeviceIdentity(1), data_slot);
+    let mut corrupted_copy = intact_copy_on_device_one.clone();
+    corrupted_copy[4096] ^= 0xff;
+    write_copy_at(devices, DeviceIdentity(1), data_slot, &corrupted_copy);
+    let second = publish_overwrite_in_process(
+        &mut pool,
+        &first,
+        &second_content(),
+        FIXED_WRITE_TIME_SECONDS + SECONDS_BETWEEN_THE_TWO_WRITES,
+        InstanceGeneration(1),
+    )
+    .expect("只一份对不上时覆盖写照成");
+    let data_placement = singlefs_core::allocator::Placement {
+        slot: data_slot,
+        span: 2,
+    };
+    let data_unit = TransactionUnit::Data(singlefs_core::address::DataUnitIndexInFile::FIRST);
+    assert_eq!(
+        second.quarantined_after_release_checksum_mismatch,
+        vec![
+            CopyQuarantinedAfterReleaseChecksumMismatch {
+                unit: data_unit,
+                device: DeviceIdentity(0),
+                placement: data_placement,
+                reading: QuarantinedCopyReading::IntactButAnotherCopyFailed,
+            },
+            CopyQuarantinedAfterReleaseChecksumMismatch {
+                unit: data_unit,
+                device: DeviceIdentity(1),
+                placement: data_placement,
+                reading: QuarantinedCopyReading::ChecksumMismatch,
+            },
+        ],
+        "前提：B 隔离了 A 的数据单元两份（盘 1 对不上，盘 0 对得上一起留下）"
+    );
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        let record = pool
+            .allocator
+            .record_for(device, data_slot)
+            .expect("隔离只是记录留在已分配，不删");
+        assert!(
+            !record.is_released && record.generation == CheckpointTxg(3),
+            "前提：{device:?} 上那条记录仍分配、分配代照旧 3：{record:?}"
+        );
+    }
+    assert!(
+        second.units.iter().all(|unit| unit.slot != data_slot),
+        "前提：B 不引用 A 的数据单元那个槽"
+    );
+    pool.output = second;
+    (pool, data_slot, intact_copy_on_device_one)
+}
+
+fn roll_back_to_the_first_version(pool: &mut BuiltPool) -> Result<RolledBack, RollbackError> {
+    let publish_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    roll_back_by_a_forward_publish(
+        &publish_parameters,
+        devices,
+        &mut pool.allocator,
+        &mut pool.output,
+        first_root(),
+    )
+}
+
+#[test]
+fn rolling_back_to_the_version_referencing_an_isolated_unit_with_a_copy_that_does_not_verify_is_refused_before_any_write(
+) {
+    let (mut pool, data_slot, _) =
+        pool_with_the_data_unit_of_the_first_version_isolated("rollback-isolated-unit-refused");
+    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
+    let current_before = pool.output.clone();
+    let records_before = pool.allocator.records().to_vec();
+    let refusal = roll_back_to_the_first_version(&mut pool)
+        .expect_err("目标引用的隔离单元盘 1 那一份对不上：在任何写之前拒");
+    assert!(
+        matches!(
+            refusal,
+            RollbackError::IsolatedUnitCopyUnreadableOrMismatched {
+                unit: TransactionUnit::Data(_),
+                device: DeviceIdentity(1),
+                slot,
+            } if slot == data_slot
+        ),
+        "拒成隔离单元那一份对不上（盘 1、A 的数据单元那个槽）：{refusal:?}"
+    );
+    assert_eq!(
+        disk_snapshot(&pool.memory_pool(), &pool.stream),
+        before,
+        "拒之前盘上一个字节都不动（系统配置槽、读得出的根、录制流步数）"
+    );
+    assert_eq!(pool.output, current_before, "现行版本不动");
+    assert_eq!(
+        pool.allocator.records(),
+        records_before.as_slice(),
+        "分配器不动"
+    );
+}
+
+#[test]
+fn rolling_back_to_the_version_referencing_an_isolated_unit_whose_failing_copy_was_written_back_intact_goes_through(
+) {
+    let (mut pool, data_slot, intact_copy_on_device_one) =
+        pool_with_the_data_unit_of_the_first_version_isolated("rollback-isolated-unit-repaired");
+    write_copy_at(
+        pool.devices.as_mut().expect("镜像还开着"),
+        DeviceIdentity(1),
+        data_slot,
+        &intact_copy_on_device_one,
+    );
+    let rolled_back =
+        roll_back_to_the_first_version(&mut pool).expect("隔离单元两份都对得上：回退照做");
+    assert_eq!(rolled_back.target, first_root());
+    assert!(
+        rolled_back
+            .resurrected_user_visible_copies
+            .iter()
+            .all(|copy| copy.placement.slot != data_slot),
+        "隔离单元在账里本来就仍分配，不进复活集"
+    );
+    let cold = recover(&pool.memory_pool(), JournalPolicy::Consult);
+    assert!(
+        matches!(
+            &cold.outcome,
+            RecoveryOutcome::FileRead { content, .. } if *content == file_content()
+        ),
+        "冷启动读回 A 的内容：{:?}",
+        cold.outcome
+    );
+}
```

## 三、变异行：追加（`mutations-append.tsv`）

```text
C572 批 B3 第一处：现行账那一份豁免集合按记录起点收（只收起点槽，不按跨度展开）	crates/singlefs-core/src/mount.rs	        .flat_map(|record| {\n            slots_covered_by_a_placement(record.device, record.slot, u64::from(record.span_slots))\n        })	        .map(|record| (record.device.0, record.slot.0))	-p singlefs-core --lib -- shadow_ledger_exempts_and_isolates_slot_by_slot_tests	a_current_account_record_exempts_every_slot_it_covers_from_the_placements_of_an_abandoned_root_read_from_its_account
C572 批 B3 第二处：候选根那一份豁免集合按落点起点收	crates/singlefs-core/src/mount.rs	            referenced_by_candidates.extend(placements.iter().flat_map(\n                |(device, slot, span_slots)| {\n                    slots_covered_by_a_placement(*device, *slot, *span_slots)\n                },\n            ));	            referenced_by_candidates.extend(placements.iter().map(|(device, slot, _)| (device.0, slot.0)));	-p singlefs-core --lib -- shadow_ledger_exempts_and_isolates_slot_by_slot_tests	a_candidate_roots_two_slot_placement_exempts_its_second_slot_from_an_abandoned_roots_placement
C572 批 B3 第三处：被抛弃根那一圈照旧按起点槽豁免与判重（起点槽被豁免或已隔离就整份跳过）	crates/singlefs-core/src/mount.rs	            for slot_to_isolate in slots_of_a_placement_to_isolate_now(\n                device,\n                slot,\n                span_slots,\n                &referenced_by_candidates,\n                &mut isolated,\n            ) {	            for slot_to_isolate in (referenced_by_candidates.contains(&(device.0, slot.0)) || !isolated.insert((device.0, slot.0))).then(Vec::new).unwrap_or_else(|| (slot.0..slot.0 + span_slots).map(SlotNumber).collect()) {	-p singlefs-core --lib -- shadow_ledger_exempts_and_isolates_slot_by_slot_tests	two_abandoned_roots_sharing_a_start_slot_isolate_both_slots_of_the_wider_placement_in_either_read_order
C572 批 B3 第四处：根槽读坏的被抛弃根那一圈照旧按起点槽豁免与判重	crates/singlefs-core/src/mount.rs	        for slot_to_isolate in slots_of_a_placement_to_isolate_now(\n            placement.device,\n            placement.slot,\n            placement.span,\n            &referenced_by_candidates,\n            &mut isolated,\n        ) {	        for slot_to_isolate in (referenced_by_candidates.contains(&(placement.device.0, placement.slot.0)) || !isolated.insert((placement.device.0, placement.slot.0))).then(Vec::new).unwrap_or_else(|| (placement.slot.0..placement.slot.0 + placement.span).map(SlotNumber).collect()) {	-p singlefs-core --lib -- shadow_ledger_exempts_and_isolates_slot_by_slot_tests	a_current_account_record_at_the_same_start_slot_leaves_the_second_slot_recorded_for_an_abandoned_root_whose_ring_slot_read_bad_isolated
C572 批 B3 第五处：C 那一圈照旧按起点槽豁免与判重	crates/singlefs-core/src/mount.rs	        for slot_to_isolate in slots_of_a_placement_to_isolate_now(\n            placement.device,\n            placement.slot,\n            placement.span,\n            &referenced_by_the_current_account,\n            &mut isolated,\n        ) {	        for slot_to_isolate in (referenced_by_the_current_account.contains(&(placement.device.0, placement.slot.0)) || !isolated.insert((placement.device.0, placement.slot.0))).then(Vec::new).unwrap_or_else(|| (placement.slot.0..placement.slot.0 + placement.span).map(SlotNumber).collect()) {	-p singlefs-core --lib -- shadow_ledger_exempts_and_isolates_slot_by_slot_tests	a_current_account_record_at_the_same_start_slot_leaves_the_second_slot_of_a_placement_named_by_unapplied_records_isolated
C572 批 C572：回退目标引用的隔离单元不逐盘验（去掉「隔离单元也进逐盘验」这一步）	crates/singlefs-core/src/mount.rs	    every_copy_of_the_isolated_units_the_target_references_reads_back(\n        &*devices,\n        &isolated_units_to_check_on_every_device,\n    )?;	    let _ = &isolated_units_to_check_on_every_device;	-p singlefs-harness --test rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references -- rolling_back_to_the_version_referencing_an_isolated_unit_with_a_copy_that_does_not_verify	rolling_back_to_the_version_referencing_an_isolated_unit_with_a_copy_that_does_not_verify_is_refused_before_any_write
```

## 四、变异行：整行替换（`mutations-replacements.tsv`）

```text
步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	                if reclaim_isolates_what_this_root_references {\n                    allocator\n                        .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n                            device,\n                            slot_to_isolate,\n                            SPAN_OF_ONE_SLOT,\n                        );\n                } else {\n                    allocator.isolate_abandoned(device, slot_to_isolate, SPAN_OF_ONE_SLOT);\n                }	                let _ = (device, slot_to_isolate, reclaim_isolates_what_this_root_references);	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored with_the_shadow_ledger_a_publish_after_a_recovery	with_the_shadow_ledger_a_publish_after_a_recovery_keeps_off_the_abandoned_data_slot_and_the_abandoned_root_reads_the_third_content
步 4：影子账不豁免候选根引用的槽（保守读法）	crates/singlefs-core/src/mount.rs	                &referenced_by_candidates,\n                &mut isolated,\n            ) {	                &BTreeSet::new(),\n                &mut isolated,\n            ) {	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored the_plain_remount_after_a_recovery	the_plain_remount_after_a_recovery_that_abandoned_roots_keeps_the_isolation
增补 2 收口第 27 行：影子账隔离只铺记录起点那一槽，不按记录的跨度铺位	crates/singlefs-core/src/mount.rs	                device,\n                slot,\n                span_slots,\n                &referenced_by_candidates,	                device,\n                slot,\n                1,\n                &referenced_by_candidates,	-p singlefs-harness --test unreadable_abandoned_root_slot	an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_is_neither_isolated_nor_counted_when_its_root_slot_is_unreadable
删除线第35批 K4-1 C 那一圈照旧豁免候选根引用的槽（R 的映射根读回全零、R 是候选根，那次挂载根落盘之前写掉 R 的单元）	crates/singlefs-core/src/mount.rs	            &referenced_by_the_current_account,\n            &mut isolated,	            &referenced_by_candidates,\n            &mut isolated,	-p singlefs-harness --test writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten -- a_power_loss_before_the_row_root_of_the_mount_still_reads_back_a_candidate_root_whose_mapping_root_read_back_zero_at_that_mount	a_power_loss_before_the_row_root_of_the_mount_still_reads_back_a_candidate_root_whose_mapping_root_read_back_zero_at_that_mount
第10z批第九块 K3 loop-span1 根槽这一遍读坏的被抛弃根记着的落点补隔离只隔起点那一槽（正推 loop-span1：跨两槽的落点后一槽空闲）	crates/singlefs-core/src/mount.rs	            placement.slot,\n            placement.span,\n            &referenced_by_candidates,	            placement.slot,\n            1,\n            &referenced_by_candidates,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_isolates_the_later_slot_of_its_recorded_two_slot_placement_left_unisolated	unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_isolates_the_later_slot_of_its_recorded_two_slot_placement_left_unisolated
第10z批第十四块 H2 挂载那一刻 C 那一圈照旧隔离分配位置着的槽（1536 槽盘先释放一大批再认不出那一形）	crates/singlefs-core/src/mount.rs	            allocator\n                .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(\n                    placement.device,\n                    slot_to_isolate,\n                    SPAN_OF_ONE_SLOT,\n                );	            allocator.isolate_abandoned(placement.device, slot_to_isolate, SPAN_OF_ONE_SLOT);	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- --include-ignored wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices	wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
第10z批第十四块 H2 挂载那一刻 C 那一圈照旧隔离分配位置着的槽（第九项与已分配双扣）	crates/singlefs-core/src/mount.rs	            allocator\n                .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(\n                    placement.device,\n                    slot_to_isolate,\n                    SPAN_OF_ONE_SLOT,\n                );	            allocator.isolate_abandoned(placement.device, slot_to_isolate, SPAN_OF_ONE_SLOT);	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices	wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices
第10z批第十六块 根槽读坏的被抛弃根那一圈照旧整段隔离：抬 F 那一串双扣	crates/singlefs-core/src/mount.rs	            allocator\n                .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n                    placement.device,\n                    slot_to_isolate,\n                    SPAN_OF_ONE_SLOT,\n                );	            allocator.isolate_abandoned(placement.device, slot_to_isolate, SPAN_OF_ONE_SLOT);	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- defer_slots_an_abandoned_root_whose_ring_slot_reads_bad_references_are_not_counted_twice_by_a_floor_raise	defer_slots_an_abandoned_root_whose_ring_slot_reads_bad_references_are_not_counted_twice_by_a_floor_raise
```
