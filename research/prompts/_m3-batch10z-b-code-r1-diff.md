# 根环实现第 10z 批第二块：代码轮第一轮改动全文（2026-10-08）

实现员交的 `crates.patch` 原样，打在删除线第 35 批 W、第 36 批 V 合后、根环线 10z 第一块合后的主工作区上。变异表追加、替换两份附在后面。

````diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -16,7 +16,7 @@
 
 use std::collections::{BTreeMap, BTreeSet};
 
-use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
 use crate::admission::SpaceAdmission;
 use crate::bytes::ByteWriter;
 use crate::instance_table::InstanceTableRecords;
@@ -1023,10 +1023,14 @@
     },
     /// 按实例表判仍然有效的根，txg 在不在 F 之下都算（可再分配谓词取 max(F_生效, 环里最旧有效根)，F 那一半另记）。
     ValidRoot { checkpoint_txg: CheckpointTxg },
-    /// 按实例表判被抛弃的根（D23（journal 的角色与格式） 已定项 14），带着它引用的每个落点——挂载那一刻仍被候选根或现行账引用、
+    /// 按实例表判被抛弃的根（D23（journal 的角色与格式） 已定项 14），带着它的 (实例代号, txg) 与它引用的每个落点——挂载那一刻仍被候选根或现行账引用、
     /// 因而没隔离的也在内：它们之后在这次挂载里被回收时要补隔离，这条根离开根环时按它清。
-    /// 账读不出的、影子账关着（只供测试）的，这一串是空的：它什么都没隔离，离开根环时也什么都不清。
+    /// 账读不出（凭据放行）的、影子账关着（只供测试）的，这一串是空的：它什么都没隔离，离开根环时也什么都不清。
+    /// 抬 F 那一串（准入与卸载）重算影子账时，这一槽这一遍读出的被抛弃根与这里记着的是同一条（实例代号与 txg 都对上）就照这一串算、
+    /// 不再从盘上读它的账（[`RootRingOccupancy::placements_recorded_for_the_abandoned_root`]，D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，用户 2026-10-07 定）。
     AbandonedRoot {
+        instance: InstanceGeneration,
+        checkpoint_txg: CheckpointTxg,
         referenced_placements: Vec<PlacementOnDevice>,
     },
 }
@@ -1098,6 +1102,47 @@
             .collect()
     }
 
+    /// 抬 F 那一串（准入与卸载）重算影子账时一条被抛弃根的落点（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句「抬 F 那一串（准入与卸载）里的被抛弃根」，
+    /// 用户 2026-10-07 定）：这条根（`instance`、`checkpoint_txg`）的 txg 在根环里对应的那一槽上，表里记着的是挂载那一刻读出的被抛弃根
+    /// （[`RootRingOccupant::AbandonedRoot`]，或两遍里只有一遍自证过的槽里被抛弃的那一半，按那一遍读出的整条根记录认），而且就是这一条——
+    /// 实例代号与 txg 都对上，不只按槽号对——就交回挂载时算出、记着的那一串落点；记着的是别的根、有效根、挂载时读坏或读回全零的槽，
+    /// 或这个进程已写掉那一槽（换成有效根），交 `None`。只读内存里的表，不读盘。
+    #[must_use]
+    pub fn placements_recorded_for_the_abandoned_root(
+        &self,
+        instance: InstanceGeneration,
+        checkpoint_txg: CheckpointTxg,
+    ) -> Option<Vec<PlacementOnDevice>> {
+        let ring_slot = target_for_publish(checkpoint_txg, self.slots_per_region);
+        match self.occupants.get(&ring_slot)? {
+            RootRingOccupant::AbandonedRoot {
+                instance: recorded_instance,
+                checkpoint_txg: recorded_checkpoint_txg,
+                referenced_placements,
+            } => ((*recorded_instance, *recorded_checkpoint_txg) == (instance, checkpoint_txg))
+                .then(|| referenced_placements.clone()),
+            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                root:
+                    RootReadOnOneRebuildPass::AbandonedRoot {
+                        referenced_placements,
+                    },
+                root_record_self_verified_on_one_rebuild_pass,
+            } => ((
+                root_record_self_verified_on_one_rebuild_pass.instance,
+                root_record_self_verified_on_one_rebuild_pass.checkpoint_txg,
+            ) == (instance, checkpoint_txg))
+                .then(|| referenced_placements.clone()),
+            RootRingOccupant::ValidRoot { .. }
+            | RootRingOccupant::BadAtMount(_)
+            | RootRingOccupant::AllZeroAtMount { .. }
+            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
+            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                root: RootReadOnOneRebuildPass::ValidRoot { .. },
+                ..
+            } => None,
+        }
+    }
+
     /// 环里还在的被抛弃根引用着的全部落点，加还没解开的、水位之上没施加的记录点名的全部落点：清隔离位之前要看的那一份
     /// （一个槽两边任一边还罩着就不清），回收出来的槽里它们罩着的补隔离。
     fn placements_the_isolation_still_protects(&self) -> Vec<PlacementOnDevice> {
@@ -1196,6 +1241,7 @@
             .flat_map(|occupant| match occupant {
                 RootRingOccupant::AbandonedRoot {
                     referenced_placements,
+                    ..
                 }
                 | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                     root:
@@ -2070,6 +2116,7 @@
             Some(
                 RootRingOccupant::AbandonedRoot {
                     referenced_placements,
+                    ..
                 }
                 | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                     root:
@@ -2987,6 +3034,97 @@
         }
     }
 
+    /// 抬 F 那一串取的被抛弃根落点按根认、不只按槽认（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，用户 2026-10-07 定）：
+    /// 24 槽的环里 txg 8 那一槽记着实例 2 的被抛弃根（读出那一遍记成 `AbandonedRoot`），txg 9 那一槽是两遍里只有一遍自证过、被抛弃的
+    /// 实例 2 的根。同一条根交回记着的落点；同一槽上 txg 差一整圈的根、或实例代号不同的根交 `None`；有效根那一槽交 `None`。
+    #[test]
+    fn placements_recorded_for_an_abandoned_root_answer_only_for_the_same_root_not_for_another_root_on_its_ring_slot(
+    ) {
+        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
+        let ring_length = 3 * slots_per_region.count();
+        let recorded_on_txg_8 = vec![PlacementOnDevice {
+            device: DeviceIdentity(0),
+            slot: SlotNumber(50180),
+            span: 2,
+        }];
+        let recorded_on_txg_9 = vec![PlacementOnDevice {
+            device: DeviceIdentity(1),
+            slot: SlotNumber(50190),
+            span: 1,
+        }];
+        let root_on_txg_9 = RootRecord {
+            filesystem_identifier: [5u8; 16],
+            unmount_marker: crate::root_record::UnmountMarker::NotWrittenByTheUnmountSequence,
+            instance: InstanceGeneration(2),
+            checkpoint_txg: CheckpointTxg(9),
+            tree_table: crate::pointer::NodePointer::empty_root(),
+            tree_identifier_watermark: singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+            rollback_floor: CheckpointTxg(0),
+            instance_table: crate::pointer::NodePointer::empty_root(),
+            mapping_root: crate::pointer::NodePointer::empty_root(),
+            allocation_record_tree_root: crate::pointer::NodePointer::empty_root(),
+        };
+        let occupancy = RootRingOccupancy::read_from_the_ring(
+            slots_per_region,
+            vec![
+                (
+                    target_for_publish(CheckpointTxg(8), slots_per_region),
+                    RootRingOccupant::AbandonedRoot {
+                        instance: InstanceGeneration(2),
+                        checkpoint_txg: CheckpointTxg(8),
+                        referenced_placements: recorded_on_txg_8.clone(),
+                    },
+                ),
+                (
+                    target_for_publish(CheckpointTxg(9), slots_per_region),
+                    RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                        root: RootReadOnOneRebuildPass::AbandonedRoot {
+                            referenced_placements: recorded_on_txg_9.clone(),
+                        },
+                        root_record_self_verified_on_one_rebuild_pass: Box::new(root_on_txg_9),
+                    },
+                ),
+                (
+                    target_for_publish(CheckpointTxg(10), slots_per_region),
+                    RootRingOccupant::ValidRoot {
+                        checkpoint_txg: CheckpointTxg(10),
+                    },
+                ),
+            ],
+            CheckpointTxg(0),
+            CheckpointTxg(10),
+        );
+        assert_eq!(
+            occupancy.placements_recorded_for_the_abandoned_root(
+                InstanceGeneration(2),
+                CheckpointTxg(8)
+            ),
+            Some(recorded_on_txg_8),
+            "记成被抛弃根的那一条：交回挂载时记着的落点"
+        );
+        assert_eq!(
+            occupancy.placements_recorded_for_the_abandoned_root(
+                InstanceGeneration(2),
+                CheckpointTxg(9)
+            ),
+            Some(recorded_on_txg_9),
+            "两遍里只有一遍自证过、被抛弃的那一条：按那一遍读出的整条根记录认，交回记着的落点"
+        );
+        for (instance, checkpoint_txg) in [
+            (InstanceGeneration(2), CheckpointTxg(8 + ring_length)),
+            (InstanceGeneration(3), CheckpointTxg(8)),
+            (InstanceGeneration(2), CheckpointTxg(9 + ring_length)),
+            (InstanceGeneration(3), CheckpointTxg(9)),
+            (InstanceGeneration(2), CheckpointTxg(10)),
+        ] {
+            assert_eq!(
+                occupancy.placements_recorded_for_the_abandoned_root(instance, checkpoint_txg),
+                None,
+                "({instance:?}, {checkpoint_txg:?}) 不是那一槽记着的那条被抛弃根（同槽差一整圈、实例代号不同，或那一槽是有效根）：不交落点"
+            );
+        }
+    }
+
     /// 环上有洞留下的旧根压住进程内的回收门槛（D16（发布语义） 已定项 1「环里最旧有效根」，用户 2026-10-04 定 RGall）：24 槽的环里
     /// txg 1 那一槽留着 txg 1 的根（txg 25、49 那两次的根槽都没落），其余 23 槽住着 txg 26–48；这个进程记到 txg 49（没有根的那一个）
     /// 之后再记 txg 50，盖掉的是 txg 26。txg 1 离最新根已过两圈，仍是环里最旧的有效根：释放代 2 的落点不回收、仍在 defer 里。
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1473,6 +1473,41 @@
         BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<PlacementOnDevice>>,
 }
 
+/// 影子账里一条被抛弃根引用的落点从哪来（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句「抬 F 那一串（准入与卸载）里的被抛弃根」、
+/// D23（journal 的角色与格式） 已定项 14 回退段「抬 F 那一串（准入与卸载）不走这一道」，用户 2026-10-07 定）。封闭集合，`match` 不写通配臂。
+enum PlacementsOfAbandonedRoots<'recorded> {
+    /// 可写挂载重建分配器那一遍：从盘上读它的账，读不出照 C393 从映射现算、重读一次、再看 C694 的 B 凭据，仍不成就拒
+    /// （「崩了再可写挂载」那一道照 B 拒，不变）。
+    ReadFromTheirAccounts,
+    /// 抬 F 那一串（准入与卸载）：按 (实例代号, txg) 认、根环表里记着同一条被抛弃根的（`allocator::RootRingOccupancy::placements_recorded_for_the_abandoned_root`），
+    /// 照挂载时算出、记着的那一串落点算，不再从盘上读它的账，读不出也不拒。表里没记着这条根的（挂载之后才按现行那一版的实例表判成被抛弃的根——
+    /// 挂载那一刻它按最新那条根的表判有效、记成有效根；挂载那一刻读坏、读回全零的槽这一遍读出的被抛弃根；进程内重读读出被抛弃根、已从表里拿掉的槽）
+    /// 条款没写它们取什么，照可写挂载那一遍读盘，交主 agent（第 10z 批第二块报告）。
+    RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
+        &'recorded BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<PlacementOnDevice>>,
+    ),
+}
+
+impl PlacementsOfAbandonedRoots<'_> {
+    /// 根环表里记着的这条被抛弃根的落点；读盘那一臂与表里没记着它时 `None`。只读内存。
+    fn recorded_in_the_root_ring_table(
+        &self,
+        root: &RootRecord,
+    ) -> Option<Vec<(DeviceIdentity, SlotNumber, u64)>> {
+        match self {
+            Self::ReadFromTheirAccounts => None,
+            Self::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(recorded) => recorded
+                .get(&(root.instance, root.checkpoint_txg))
+                .map(|placements| {
+                    placements
+                        .iter()
+                        .map(|placement| (placement.device, placement.slot, placement.span))
+                        .collect()
+                }),
+        }
+    }
+}
+
 /// 影子账（D28（挂载期承诺量） 已定项 1 第九项）：只被被抛弃根引用的槽 = 被抛弃根引用的槽 − 候选集里的根引用的槽 − 当前这一版账里
 /// 还分配着的槽；候选 = 可读 ∧ 按实例表有效 ∧ txg ≥ F。用户 2026-09-16 定的窄读法措辞是「仍被有效根引用的槽不在其内」、有效只按实例表判，
 /// 豁免只给候选集里的根（多了 txg ≥ F、抬 F 之后按新 F 重算）是对那句措辞的收严（alloc-basis 那一轮的 G5 臂）：按原措辞，抬 F 到 11 之后
@@ -1484,6 +1519,7 @@
 /// （与中央映射树根槽，映射根是空指针时只看树表槽）每块盘上都有一条分配代大于那条根的 txg、罩住它的记录，就是那几槽已被那条根之后的合法写占了，
 /// 那条根引用的单元已经没有要护的，只计数、不隔离、不拒（C694 的 B，判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 2 条：
 /// 凭据只读内存里的现行账，不读那一槽）；没有这样的记录（那几槽只是这一次读不出）就在任何写之前拒：挂载在取号之前拒可写、抬 F 拒这一次抬
+/// （抬 F 那一串只在根环表里没记着这条根时才读盘、才走到这一拒）
 /// （[`StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot`]）。候选根读不出就当它什么都不豁免（隔离只会多不会少）。
 /// 同一次也隔离水位之上没施加的 journal 记录点名的落点与那几次发布的账里的落点（`placements_named_by_unapplied_journal_records`，C694 的 C）：
 /// 只豁免现行账里仍分配着的槽，不豁免候选集里的根引用的槽（根环线 K4-1，函数体里那一圈的注释），D28（挂载期承诺量） 已定项 1 第九项
@@ -1491,10 +1527,12 @@
 /// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销；
 /// 被抛弃根引用的只在它离开根环的那一次发布里清，水位之上的记录点名的只在这个进程写掉那条记录的 txg 对应的根环槽那一次发布里清
 /// （`PoolAllocator::record_root_written_by_this_process`，按交回的每条根引用的落点与分配器根环表记着的那一份）。
+/// 被抛弃根引用的落点从哪来由 `placements_of_abandoned_roots` 定（[`PlacementsOfAbandonedRoots`]）：可写挂载那一遍读盘，
+/// 抬 F 那一串取根环表里记着的那一份，表里没记着这条根的才读盘（上面那一串读不出的处置只走在这一支上）。
 #[allow(
     clippy::ptr_arg,
     clippy::too_many_arguments,
-    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、水位之上没施加的记录点名的落点、重读前的快照"
+    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、水位之上没施加的记录点名的落点、被抛弃根的落点从哪来、重读前的快照"
 )]
 fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
@@ -1505,6 +1543,7 @@
     floor: CheckpointTxg,
     current_records: &[AllocationRecord],
     placements_named_by_unapplied_journal_records: &[PlacementOnDevice],
+    placements_of_abandoned_roots: PlacementsOfAbandonedRoots<'_>,
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<ShadowLedgerComputed, MountError> {
     let referenced_by_the_current_account: BTreeSet<(u32, u64)> = current_records
@@ -1536,7 +1575,11 @@
         // `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）：账（树表或分配记录树）读不出时不跳过，
         // 从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报（显式报告）；
         // 现算也不成（映射树读不出、解不开）才重读一次（重读之前调钩子），账与现算都仍不成就在取号之前拒可写、只读照常。
-        let placements = match placements_referenced_by_root(devices, unit_area_start, root) {
+        // 抬 F 那一串先取根环表里记着的那一份（`or_else` 是惰性的：取到就不读盘，读不出的处置也走不到）。
+        let placements = match placements_of_abandoned_roots
+            .recorded_in_the_root_ring_table(root)
+            .or_else(|| placements_referenced_by_root(devices, unit_area_start, root))
+        {
             Some(placements) => placements,
             None => {
                 unreadable += 1;
@@ -2475,6 +2518,7 @@
                 .flatten()
                 .copied()
                 .collect::<Vec<PlacementOnDevice>>(),
+            PlacementsOfAbandonedRoots::ReadFromTheirAccounts,
             before_the_one_reread,
         )?,
         ShadowLedger::Off => ShadowLedgerComputed {
@@ -2537,6 +2581,8 @@
                         referenced_placements,
                     },
                 ) => RootRingOccupant::AbandonedRoot {
+                    instance: root.instance,
+                    checkpoint_txg: root.checkpoint_txg,
                     referenced_placements,
                 },
             };
@@ -3011,7 +3057,7 @@
     /// 生效之后回收的落点（两盘同槽，按盘 0 报）。
     pub reclaimed: Vec<Placement>,
     /// 抬 F 重算影子账时读不出账、被跳过的被抛弃根的条数（与 `MountOutput::abandoned_roots_unreadable` 同一个量；
-    /// 代码三方第三轮云端攻方腿打中：抬 F 那一处此前把计数丢掉了）。
+    /// 代码三方第三轮云端攻方腿打中：抬 F 那一处此前把计数丢掉了）。根环表里记着落点的被抛弃根这一遍不读盘，不计在内。
     pub abandoned_roots_unreadable: u64,
 }
 
@@ -3767,6 +3813,31 @@
         .root_ring_occupancy()
         .map(RootRingOccupancy::placements_named_by_unapplied_journal_records)
         .unwrap_or_default();
+    // 抬 F 那一串里的被抛弃根（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，用户 2026-10-07 定，判决
+    // `research/prompts/m3-batch10y-code-r1-main-verification.md` 第三节）：这一遍读出的、按现行那一版的实例表被抛弃的根，根环表里那一槽记着的
+    // 是同一条（实例代号与 txg 都对上）就照挂载时算出、记着的落点进影子账，不再从盘上读它的账、读不出也不拒（不走 C694 的 B）。
+    // 改之前照盘上重读，挂上之后它的账读坏就照 B 拒，正常卸载与会话写都被拒（攻方 `research/prompts/m3-batch10y-code-r1-opus-output.md` 「K4-2」）。
+    // 表里没记着的照旧读盘（[`PlacementsOfAbandonedRoots`] 那一臂的注释）。
+    let placements_recorded_for_abandoned_roots: BTreeMap<
+        (InstanceGeneration, CheckpointTxg),
+        Vec<PlacementOnDevice>,
+    > = allocator
+        .root_ring_occupancy()
+        .map(|occupancy| {
+            roots
+                .iter()
+                .filter(|root| abandoned_by_table(root, &table))
+                .filter_map(|root| {
+                    occupancy
+                        .placements_recorded_for_the_abandoned_root(
+                            root.instance,
+                            root.checkpoint_txg,
+                        )
+                        .map(|recorded| ((root.instance, root.checkpoint_txg), recorded))
+                })
+                .collect()
+        })
+        .unwrap_or_default();
     let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {
         match isolate_slots_referenced_only_by_abandoned_roots(
             devices,
@@ -3777,6 +3848,9 @@
             new_floor,
             &current.allocation_records,
             &placements_named_by_unapplied_journal_records,
+            PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
+                &placements_recorded_for_abandoned_roots,
+            ),
             &mut BeforeTheOneReread::RereadImmediately,
         ) {
             Ok(computed) => computed.abandoned_roots_unreadable,
diff --git a/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs b/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
--- a/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
+++ b/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs
@@ -579,12 +579,81 @@
     pool.devices = Some(devices);
 }
 
+/// 一条根的根槽在池里的位置与原字节：(盘, 偏移, 原来那一槽的字节)。
+struct HiddenRootSlot {
+    device: DeviceIdentity,
+    offset: DeviceOffsetInBytes,
+    original_bytes: Vec<u8>,
+}
+
+/// 把一条根的根槽抹成全零（之后可写挂载那一刻读回全零），交回原字节，挂载之后由 [`restore_the_root_slot`] 写回。
+fn hide_the_root_slot_of(
+    pool: &mut BuiltPool,
+    root: &singlefs_core::root_record::RootRecord,
+) -> HiddenRootSlot {
+    use singlefs_core::block_device::{BlockDevice, WriteDurability};
+    let pool_parameters = parameters();
+    let ring_slot = singlefs_core::root_ring::target_for_publish(
+        root.checkpoint_txg,
+        pool_parameters.geometry.root_ring_slots_per_region,
+    );
+    let device = pool_parameters.region_devices
+        [usize::try_from(ring_slot.region).expect("区域号装得进 usize")];
+    let offset = singlefs_core::root_ring::slot_offset(
+        ring_slot,
+        pool_parameters.geometry.fixed_structure_slot_spacing,
+    );
+    let root_slot_bytes =
+        usize::try_from(pool_parameters.geometry.physical_block_size).expect("物理块装得进 usize");
+    let mut devices = pool.reopen_recorded();
+    let index = devices
+        .iter()
+        .position(|(identity, _)| *identity == device)
+        .expect("根槽所在的盘");
+    let mut original_bytes = vec![0u8; root_slot_bytes];
+    devices[index]
+        .1
+        .read_at(offset, &mut original_bytes)
+        .expect("读根槽");
+    devices[index]
+        .1
+        .write_at(offset, &vec![0u8; root_slot_bytes], WriteDurability::Plain)
+        .expect("抹零根槽");
+    pool.devices = Some(devices);
+    HiddenRootSlot {
+        device,
+        offset,
+        original_bytes,
+    }
+}
+
+fn restore_the_root_slot(pool: &mut BuiltPool, hidden: &HiddenRootSlot) {
+    use singlefs_core::block_device::{BlockDevice, WriteDurability};
+    let mut devices = pool.reopen_recorded();
+    let index = devices
+        .iter()
+        .position(|(identity, _)| *identity == hidden.device)
+        .expect("根槽所在的盘");
+    devices[index]
+        .1
+        .write_at(
+            hidden.offset,
+            &hidden.original_bytes,
+            WriteDurability::Plain,
+        )
+        .expect("写回根槽");
+    pool.devices = Some(devices);
+}
+
 /// 抬 F 那一串在影子账上被拒（代码轮 LR3，攻方 `m3-leftover-code-r1` Z2 ③：此前没有用例走到这一支）：
 /// 崩溃恢复先后抛弃 C (2, 8) 与 E（实例 3 覆盖写 D 之后的那一版），再可写挂载一次、覆盖写 [`OVERWRITES_BEFORE_THE_RAISE`] 次，然后把两条被抛弃根里影子账排在后面的那一条的
 /// 树表与中央映射树根两份都抹零。抬 F 到上限：新 F 越过两条被抛弃根的前一版，排在前面那条被抛弃根引用的槽（前一版那张实例表这类）
 /// 此刻只被它引用，影子账先给它补上隔离；算到后面那条，账与从映射现算都不成、重读一次仍不成，`isolate_slots_referenced_only_by_abandoned_roots`
 /// 交 `Err`——抬 F 拒成 `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`、点名那条根，一个写都不发，
 /// 分配器整个换回抬 F 之前那一份（补上的隔离撤掉）。
+/// 抬 F 那一串里根环表记着的被抛弃根照记着的落点算、读不出也不拒（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，用户 2026-10-07 定，
+/// 根环线第 10z 批）：要走到影子账的拒，排在后面的那一条得是根环表里没记着的——再可写挂载那一刻它的根槽读回全零（挂载之后写回），
+/// 根环表把那一槽记成「挂载时读回全零」，抬 F 那一遍读出它、照盘上读它的账（这一形条款没写，照改之前，交主 agent）。
 /// 判别力：影子账交 `Err` 时不换回分配器，前一条补上的隔离留在分配器上，「与抬 F 之前逐项相同」那一项红。
 #[test]
 fn a_floor_raise_whose_shadow_ledger_meets_an_abandoned_root_unreadable_by_account_and_mapping_is_refused_and_restores_the_allocator(
@@ -610,13 +679,44 @@
     )
     .expect("E");
     abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned_e);
+    let keys = [
+        (abandoned_c.root.instance, abandoned_c.root.checkpoint_txg),
+        (abandoned_e.root.instance, abandoned_e.root.checkpoint_txg),
+    ];
+    let positions = positions_in_the_ring_order(&mut pool, &keys);
+    let (earlier, later) = if positions[0] > positions[1] {
+        (&abandoned_e, &abandoned_c)
+    } else {
+        (&abandoned_c, &abandoned_e)
+    };
+    let hidden_root_slot = hide_the_root_slot_of(&mut pool, &later.root);
     let mut devices_of_the_mount = pool.reopen_recorded();
     let mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices_of_the_mount)
-        .expect("两条被抛弃根都读得出：照常可写挂载");
+        .expect("排在后面那条被抛弃根的根槽读回全零、前面那条读得出：照常可写挂载");
     let instance = mounted.output.instance;
     pool.devices = Some(devices_of_the_mount);
     pool.allocator = mounted.allocator;
     pool.output = mounted.current.into_file_version().expect("带文件");
+    restore_the_root_slot(&mut pool, &hidden_root_slot);
+    let root_ring_table = pool
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表");
+    assert!(
+        root_ring_table
+            .placements_recorded_for_the_abandoned_root(
+                earlier.root.instance,
+                earlier.root.checkpoint_txg
+            )
+            .is_some()
+            && root_ring_table
+                .placements_recorded_for_the_abandoned_root(
+                    later.root.instance,
+                    later.root.checkpoint_txg
+                )
+                .is_none(),
+        "根环表记着排在前面的那条被抛弃根、没记着排在后面的那一条"
+    );
     for overwrite_index in 0..OVERWRITES_BEFORE_THE_RAISE {
         let previous = pool.output.clone();
         pool.output = publish_overwrite_in_process(
@@ -628,16 +728,6 @@
         )
         .expect("这次挂载里覆盖写");
     }
-    let keys = [
-        (abandoned_c.root.instance, abandoned_c.root.checkpoint_txg),
-        (abandoned_e.root.instance, abandoned_e.root.checkpoint_txg),
-    ];
-    let positions = positions_in_the_ring_order(&mut pool, &keys);
-    let later = if positions[0] > positions[1] {
-        &abandoned_c
-    } else {
-        &abandoned_e
-    };
     zero_the_tree_table_and_the_mapping_root_of(&mut pool, &later.root);
     let allocator_before_the_raise = format!("{:?}", pool.allocator);
     let image_before_the_raise = pool.memory_pool();
diff --git a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -0,0 +1,509 @@
+//! 抬 F 那一串（准入与卸载）里的被抛弃根照挂载时记在根环表里的落点算影子账，不再从盘上重读它的账，读不出也不拒
+//! （D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，D23（journal 的角色与格式） 已定项 14 回退段「抬 F 那一串（准入与卸载）不走这一道」，
+//! 用户 2026-10-07 定；判决 `research/prompts/m3-batch10y-code-r1-main-verification.md` 第二节 K4-2 与第三节）。「崩了再可写挂载」那一道照 C694 的 B 拒，不变。
+//!
+//! 造法照攻方 `research/prompts/m3-batch10y-code-r1-opus-model/opus_10y_k4_probe.rs` 的 `abandoned_start`（384 槽小盘、之前覆盖写 1 次）：
+//! 实例 1 第一个文件、覆盖写一次；崩了再挂（实例 2）覆盖写一次得 C；把 C 的根槽与数据单元抹成全零、每块盘的系统配置轮换退回 C 之前，
+//! 崩了再挂（实例 3）择根落到 C 之前那一条、写行与暖机；再把抹掉的字节写回、进程退出。C 留在根环里，按实例 3 的表被抛弃。
+//! 之后可写挂载一次（实例 4）：挂载那一刻 C 的账读得出，根环表里把 C 记成被抛弃根、带着它引用的落点。挂上之后 C 的账那几个单元
+//! （树表、中央映射树根、分配记录树根的每条位置条目）从此读不出（这几块盘上的读报错）。
+//! 改之前抬 F 那一遍照盘上重读 C 的账，读不出、从映射现算不成、重读一次仍不成、现行账里又没有 B 的凭据，就拒：正常卸载与会话写都被拒。
+mod common_admission;
+
+use std::cell::Cell;
+use std::rc::Rc;
+
+use common_admission::{
+    checker_violations_on, start_plain, DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES,
+};
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::{
+    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+};
+use singlefs_core::make_filesystem::MakeFilesystemParameters;
+use singlefs_core::mount::{
+    mount_writable_with_space_admission, unmount, MountError, RollbackTarget, ShadowLedger,
+    StillUnreadableAfterOneReread, Unmounted,
+};
+use singlefs_core::mounted_session::MountedSession;
+use singlefs_core::pointer::NodePointer;
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_harness::history::HistoryDeviceWidth;
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
+use std::collections::BTreeMap;
+
+/// C 那一次覆盖写的内容长度（攻方原型的 2500 字节）：与之前之后的覆盖写长度不同，读回哪一版看得出。
+const CONTENT_BYTES_OF_THE_ABANDONED_VERSION: usize = 2500;
+
+/// 挂着时 C 的账读不出之后，会话至多覆盖写几次去等准入推出第一串抬 F（攻方原型覆盖写到 60 次；改之前推抬 F 的那一次写被拒）。
+const OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST: usize = 60;
+
+/// 准入推过第一串抬 F 之后再覆盖写几次（推过之后的写也不拒）。
+const OVERWRITES_AFTER_THE_FIRST_FLOOR_RAISE: usize = 3;
+
+/// 一份崩溃恢复抛弃了最新那条根 C 之后、进程退出的盘面（还没再挂）。
+struct PoolWithAnAbandonedRoot {
+    parameters: MakeFilesystemParameters,
+    device_bytes: u64,
+    image: MemoryPool,
+    write_time_seconds: u64,
+    abandoned_root: RootRecord,
+}
+
+fn read_bytes_of(
+    pool: &PoolUnderTest<SparseBlockDevice>,
+    device: DeviceIdentity,
+    offset: DeviceOffsetInBytes,
+    length_in_bytes: usize,
+) -> Vec<u8> {
+    let (_, block_device) = pool
+        .devices
+        .iter()
+        .find(|(identity, _)| *identity == device)
+        .expect("池里有这块盘");
+    let mut bytes = vec![0u8; length_in_bytes];
+    block_device
+        .read_at(offset, &mut bytes)
+        .expect("内存稀疏盘上的读做得成");
+    bytes
+}
+
+fn write_bytes_to(
+    pool: &mut PoolUnderTest<SparseBlockDevice>,
+    device: DeviceIdentity,
+    offset: DeviceOffsetInBytes,
+    bytes: &[u8],
+) {
+    let (_, block_device) = pool
+        .devices
+        .iter_mut()
+        .find(|(identity, _)| *identity == device)
+        .expect("池里有这块盘");
+    block_device
+        .write_at(offset, bytes, WriteDurability::Plain)
+        .expect("内存稀疏盘上的写做得成");
+}
+
+/// 攻方原型 `abandoned_start` 照搬（之前覆盖写 1 次）：见文件头。
+fn pool_whose_newest_root_was_abandoned_by_a_crash_recovery() -> PoolWithAnAbandonedRoot {
+    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
+    pool.overwrite(OVERWRITE_BYTES).expect("实例 1 覆盖写一次");
+    pool.crash_and_mount_writable().expect("崩了再挂：取号 2");
+    pool.overwrite(CONTENT_BYTES_OF_THE_ABANDONED_VERSION)
+        .expect("实例 2 覆盖写一次：C");
+    let abandoned_version = pool.current_file_version().clone();
+    let parameters = pool.parameters.clone();
+    let ring_slot_of_the_abandoned_root = target_for_publish(
+        abandoned_version.root.checkpoint_txg,
+        parameters.geometry.root_ring_slots_per_region,
+    );
+    let root_slot_device = parameters.region_devices
+        [usize::try_from(ring_slot_of_the_abandoned_root.region).expect("区域号装得进 usize")];
+    let root_slot_offset = slot_offset(
+        ring_slot_of_the_abandoned_root,
+        parameters.geometry.fixed_structure_slot_spacing,
+    );
+    let root_slot_bytes =
+        usize::try_from(parameters.geometry.physical_block_size).expect("物理块装得进 usize");
+    let data_unit_bytes =
+        usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("数据单元装得进 usize");
+    let mut hidden_places: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)> =
+        vec![(root_slot_device, root_slot_offset, root_slot_bytes)];
+    for pointer in &abandoned_version.data_pointers {
+        for location in pointer.locations {
+            hidden_places.push((
+                location.device,
+                location.slot.to_device_offset(),
+                data_unit_bytes,
+            ));
+        }
+    }
+    let mut hidden_bytes = Vec::new();
+    for (device, offset, length_in_bytes) in &hidden_places {
+        let bytes = read_bytes_of(&pool, *device, *offset, *length_in_bytes);
+        write_bytes_to(&mut pool, *device, *offset, &vec![0u8; *length_in_bytes]);
+        hidden_bytes.push((*device, *offset, bytes));
+    }
+    let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
+    let rotations_rolled_back: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = pool
+        .devices
+        .iter()
+        .map(|(identity, _)| {
+            let mut verified = singlefs_core::recovery::verified_system_configuration_slots(
+                pool.devices.as_slice(),
+                *identity,
+                slot_spacing,
+                &parameters.filesystem_identifier,
+            );
+            verified.sort_by_key(|slot| slot.quantities.slot_generation);
+            let [older, newest] =
+                <[_; 2]>::try_from(verified).expect("每块盘上系统配置两槽都自证得过");
+            let newest_generation = newest.quantities.slot_generation;
+            let mut not_rotated = older;
+            not_rotated.quantities.slot_generation = newest_generation
+                .checked_sub(2)
+                .expect("覆盖写过几次之后槽代号不小于 2");
+            (
+                *identity,
+                DeviceOffsetInBytes(
+                    (newest_generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
+                        * slot_spacing,
+                ),
+                not_rotated.to_slot(),
+            )
+        })
+        .collect();
+    for (device, offset, bytes) in &rotations_rolled_back {
+        write_bytes_to(&mut pool, *device, *offset, bytes);
+    }
+    let abandoning = pool
+        .crash_and_mount_writable()
+        .expect("崩了再挂：择根落到 C 之前那一条、取号 3");
+    assert!(
+        abandoning.effective_root.checkpoint_txg < abandoned_version.root.checkpoint_txg,
+        "择根落到 C 之前那一条：{:?}",
+        abandoning.effective_root
+    );
+    for (device, offset, bytes) in &hidden_bytes {
+        write_bytes_to(&mut pool, *device, *offset, bytes);
+    }
+    pool.session = None;
+    PoolWithAnAbandonedRoot {
+        parameters: pool.parameters.clone(),
+        device_bytes: pool.device_bytes,
+        image: pool.image(),
+        write_time_seconds: pool.write_time_seconds,
+        abandoned_root: abandoned_version.root,
+    }
+}
+
+/// C 的账那几个单元的读故障：`is_active` 为真时，读到 `ranges` 里任一段（盘、起点字节、终点字节）就报错。几块盘共用一份。
+#[derive(Clone)]
+struct AccountReadFault {
+    ranges: Rc<Vec<(DeviceIdentity, u64, u64)>>,
+    is_active: Rc<Cell<bool>>,
+}
+
+impl AccountReadFault {
+    /// C 的树表、中央映射树根、分配记录树根（空指针不算）的每条位置条目各一个槽。故障先关着。
+    fn on_the_account_of(root: &RootRecord) -> Self {
+        let ranges = [
+            root.tree_table,
+            root.mapping_root,
+            root.allocation_record_tree_root,
+        ]
+        .into_iter()
+        .filter(|pointer| *pointer != NodePointer::empty_root())
+        .flat_map(|pointer| pointer.locations)
+        .map(|location| {
+            let start_in_bytes = location.slot.to_device_offset().0;
+            (
+                location.device,
+                start_in_bytes,
+                start_in_bytes + singlefs_format::SLOT_BYTES,
+            )
+        })
+        .collect();
+        Self {
+            ranges: Rc::new(ranges),
+            is_active: Rc::new(Cell::new(false)),
+        }
+    }
+
+    fn refuses(&self, device: DeviceIdentity, offset_in_bytes: u64, length_in_bytes: u64) -> bool {
+        self.is_active.get()
+            && self.ranges.iter().any(|(range_device, start, end)| {
+                *range_device == device
+                    && offset_in_bytes < *end
+                    && offset_in_bytes + length_in_bytes > *start
+            })
+    }
+}
+
+struct DeviceWhoseReadsOfTheAbandonedAccountFail {
+    identity: DeviceIdentity,
+    inner: SparseBlockDevice,
+    fault: AccountReadFault,
+}
+
+impl BlockDevice for DeviceWhoseReadsOfTheAbandonedAccountFail {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
+        if self.fault.refuses(self.identity, offset.0, length_in_bytes) {
+            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
+                "C 的账那几个单元从这一刻起读不出",
+            )));
+        }
+        self.inner.read_at(offset, buffer)
+    }
+
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        self.inner.write_at(offset, bytes, durability)
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
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
+impl DeviceOverASparseImage for DeviceWhoseReadsOfTheAbandonedAccountFail {
+    fn sparse_image(&self) -> &SparseDevice {
+        &self.inner.image
+    }
+}
+
+fn devices_with_the_account_fault_on(
+    image: &MemoryPool,
+    fault: &AccountReadFault,
+) -> Vec<(DeviceIdentity, DeviceWhoseReadsOfTheAbandonedAccountFail)> {
+    common_admission::plain_devices_on(image)
+        .into_iter()
+        .map(|(identity, inner)| {
+            (
+                identity,
+                DeviceWhoseReadsOfTheAbandonedAccountFail {
+                    identity,
+                    inner,
+                    fault: fault.clone(),
+                },
+            )
+        })
+        .collect()
+}
+
+/// 可写挂载一次（实例 4，空间准入判着、影子账开着），故障关着：C 的账读得出、根环表把 C 记成被抛弃根。交回挂着的池与那份故障开关。
+fn mounted_while_the_account_of_the_abandoned_root_reads(
+    built: &PoolWithAnAbandonedRoot,
+) -> (
+    PoolUnderTest<DeviceWhoseReadsOfTheAbandonedAccountFail>,
+    AccountReadFault,
+) {
+    let fault = AccountReadFault::on_the_account_of(&built.abandoned_root);
+    let mut devices = devices_with_the_account_fault_on(&built.image, &fault);
+    let mounted = mount_writable_with_space_admission(
+        &built.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("C 的账读得出：可写挂载做得成");
+    let recorded = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .placements_recorded_for_the_abandoned_root(
+            built.abandoned_root.instance,
+            built.abandoned_root.checkpoint_txg,
+        );
+    assert!(
+        recorded
+            .as_ref()
+            .is_some_and(|placements| !placements.is_empty()),
+        "挂载那一刻根环表把 C 记成被抛弃根、带着它引用的落点：{recorded:?}"
+    );
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    let pool = PoolUnderTest {
+        parameters: built.parameters.clone(),
+        device_bytes: built.device_bytes,
+        devices,
+        session: Some(session),
+        write_time_seconds: built.write_time_seconds + 1000,
+        content_of_the_current_version: Vec::new(),
+        content_of_each_root: BTreeMap::new(),
+    };
+    (pool, fault)
+}
+
+/// 故障开着之后 C 的树表那一槽确实读不出（阳性对照：用例不是在读得出的盘上空转）。
+fn assert_the_tree_table_of_the_abandoned_root_is_unreadable(
+    pool: &PoolUnderTest<DeviceWhoseReadsOfTheAbandonedAccountFail>,
+    abandoned_root: &RootRecord,
+) {
+    let location = abandoned_root.tree_table.locations[0];
+    let (_, block_device) = pool
+        .devices
+        .iter()
+        .find(|(identity, _)| *identity == location.device)
+        .expect("C 的树表所在的盘");
+    let mut bytes = vec![0u8; usize::try_from(singlefs_format::SLOT_BYTES).expect("槽宽")];
+    assert!(
+        block_device
+            .read_at(location.slot.to_device_offset(), &mut bytes)
+            .is_err(),
+        "故障开着：C 的树表那一槽读不出"
+    );
+}
+
+fn violations_of_unit_reuse_on(image: &MemoryPool) -> Vec<String> {
+    checker_violations_on(image)
+        .into_iter()
+        .filter(|violation| violation.starts_with("I-7.4") || violation.starts_with("I-4.8"))
+        .collect()
+}
+
+/// 挂上之后 C 的账读不出：正常卸载照根环表里记着的 C 的落点算影子账，做成；卸载之后池级 checker 不红 I-7.4（近 K 代块未被复用）与 I-4.8。
+/// 改之前这一次卸载拒（`NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`）。
+#[test]
+fn unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let (mut pool, fault) = mounted_while_the_account_of_the_abandoned_root_reads(&built);
+    fault.is_active.set(true);
+    assert_the_tree_table_of_the_abandoned_root_is_unreadable(&pool, &built.abandoned_root);
+    let mut session = pool.session.take().expect("挂着的会话");
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut session.allocator,
+        &mut session.current,
+        ShadowLedger::On,
+    );
+    match unmounted {
+        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => assert_eq!(
+            raised.abandoned_roots_unreadable, 0,
+            "C 的落点取根环表里记着的那一份，这一遍不读它的账"
+        ),
+        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
+            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
+        }
+        Err(refusal) => panic!("C 的账读不出，卸载照根环表里记着的落点做成，而被拒：{refusal:?}"),
+    }
+    let violations = violations_of_unit_reuse_on(&pool.image());
+    assert!(
+        violations.is_empty(),
+        "卸载之后 C 引用的单元一个都没复用：{violations:?}"
+    );
+}
+
+/// 这一刻根环表里还把 C 记成被抛弃根（这个进程还没写掉 C 那一槽）。
+fn the_root_ring_table_still_records_the_abandoned_root(
+    pool: &PoolUnderTest<DeviceWhoseReadsOfTheAbandonedAccountFail>,
+    abandoned_root: &RootRecord,
+) -> bool {
+    pool.session()
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .placements_recorded_for_the_abandoned_root(
+            abandoned_root.instance,
+            abandoned_root.checkpoint_txg,
+        )
+        .is_some()
+}
+
+/// 挂上之后 C 的账读不出：会话接着覆盖写到准入推出第一串抬 F（推的那一刻 C 还在根环里），再写几次，一次都不拒——准入推的抬 F
+/// 照根环表里记着的 C 的落点算影子账。写完池级 checker 不红 I-7.4 与 I-4.8。
+/// 改之前推抬 F 的那一次覆盖写拒（`FloorRaiseFailedWhilePushingForSpace`，拒因点名 C 的账）。
+#[test]
+fn session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let (mut pool, fault) = mounted_while_the_account_of_the_abandoned_root_reads(&built);
+    fault.is_active.set(true);
+    assert_the_tree_table_of_the_abandoned_root_is_unreadable(&pool, &built.abandoned_root);
+    let mut overwrites_before_the_first_floor_raise = 0;
+    // 迭代上界 OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST；跨轮不带状态（计数之外）；出口：推出抬 F 的那一次 break，被拒就当场判红（panic）。
+    loop {
+        assert!(
+            overwrites_before_the_first_floor_raise < OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST,
+            "{OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST} 次覆盖写之内准入推出抬 F（384 槽小盘）"
+        );
+        let abandoned_root_was_in_the_ring =
+            the_root_ring_table_still_records_the_abandoned_root(&pool, &built.abandoned_root);
+        let published = pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!(
+                "第 {} 次覆盖写：C 的账读不出，准入抬 F 照根环表里记着的落点做成，而被拒：{refusal:?}",
+                overwrites_before_the_first_floor_raise + 1
+            )
+        });
+        overwrites_before_the_first_floor_raise += 1;
+        if !published.floor_raises.is_empty() {
+            assert!(
+                abandoned_root_was_in_the_ring,
+                "准入推抬 F 的那一刻 C 还在根环里（不然这一串里没有被抛弃根要算，没走到要判的那一处）"
+            );
+            break;
+        }
+    }
+    for overwrite_number in 1..=OVERWRITES_AFTER_THE_FIRST_FLOOR_RAISE {
+        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!("推过抬 F 之后第 {overwrite_number} 次覆盖写被拒：{refusal:?}")
+        });
+    }
+    let violations = violations_of_unit_reuse_on(&pool.image());
+    assert!(
+        violations.is_empty(),
+        "这一串覆盖写之后 C 引用的单元一个都没复用：{violations:?}"
+    );
+}
+
+/// 「崩了再可写挂载」那一道照 C694 的 B 拒，不变：同一份盘面上 C 的账读不出、现行账里没有罩住它树表槽的凭据，可写挂载在取号之前拒、
+/// 点名 C，两块盘逐字节不变。
+#[test]
+fn writable_mount_after_a_crash_still_refuses_while_the_account_of_an_abandoned_root_is_unreadable_without_the_credential(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let fault = AccountReadFault::on_the_account_of(&built.abandoned_root);
+    fault.is_active.set(true);
+    let mut devices = devices_with_the_account_fault_on(&built.image, &fault);
+    let refused = mount_writable_with_space_admission(
+        &built.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    );
+    let expected_abandoned_root = RollbackTarget {
+        instance: built.abandoned_root.instance,
+        checkpoint_txg: built.abandoned_root.checkpoint_txg,
+    };
+    match refused {
+        Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) => {
+            assert!(
+                matches!(
+                    *still_unreadable,
+                    StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { abandoned_root }
+                        if abandoned_root == expected_abandoned_root
+                ),
+                "拒因点名 C 的账：{still_unreadable:?}"
+            );
+        }
+        Err(other) => panic!("拒因是 C 的账读不出，而是：{other:?}"),
+        Ok(_) => panic!("C 的账读不出、没有凭据：崩了再可写挂载照拒，而挂上了"),
+    }
+    let image_after_the_refusal = MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.sparse_image().clone()))
+            .collect(),
+        device_size_in_bytes: built.device_bytes,
+    };
+    assert!(
+        image_after_the_refusal == built.image,
+        "在取号与任何写之前拒：两块盘逐字节不变"
+    );
+}
````

## 变异表追加（`mutations-append.tsv` 原样）

````tsv
第10z批 抬 F 那一串的被抛弃根改回从盘上重读账（挂上之后 C 的账读不出：正常卸载照 B 拒）	crates/singlefs-core/src/mount.rs	            PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(\n                &placements_recorded_for_abandoned_roots,\n            ),\n            &mut BeforeTheOneReread::RereadImmediately,	            PlacementsOfAbandonedRoots::ReadFromTheirAccounts,\n            &mut BeforeTheOneReread::RereadImmediately,	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds
第10z批 抬 F 那一串只在被抛弃根的账这一遍读得出时取根环表里记着的落点（读不出照旧拒：正常卸载照 B 拒）	crates/singlefs-core/src/mount.rs	                .filter(|root| abandoned_by_table(root, &table))	                .filter(|root| abandoned_by_table(root, &table))\n                .filter(|root| {\n                    placements_referenced_by_root(\n                        &*devices,\n                        unit_area_start_of_the_chosen_system_configuration(&system_configuration),\n                        root,\n                    )\n                    .is_some()\n                })	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds
第10z批 根环表里记着的被抛弃根只按槽号对、不按 (实例代号, txg) 对（同槽差一整圈、实例代号不同的根也交落点）	crates/singlefs-core/src/allocator.rs	            } => ((*recorded_instance, *recorded_checkpoint_txg) == (instance, checkpoint_txg))	            } => (true || (*recorded_instance, *recorded_checkpoint_txg) == (instance, checkpoint_txg))	-p singlefs-core --lib -- placements_recorded_for_an_abandoned_root_answer_only_for_the_same_root	placements_recorded_for_an_abandoned_root_answer_only_for_the_same_root_not_for_another_root_on_its_ring_slot
第10z批 两遍里只有一遍自证过的被抛弃根只按槽号对、不按 (实例代号, txg) 对（同槽差一整圈、实例代号不同的根也交落点）	crates/singlefs-core/src/allocator.rs	                root_record_self_verified_on_one_rebuild_pass.checkpoint_txg,\n            ) == (instance, checkpoint_txg))	                root_record_self_verified_on_one_rebuild_pass.checkpoint_txg,\n            ) == (instance, checkpoint_txg)\n                || true)	-p singlefs-core --lib -- placements_recorded_for_an_abandoned_root_answer_only_for_the_same_root	placements_recorded_for_an_abandoned_root_answer_only_for_the_same_root_not_for_another_root_on_its_ring_slot
````

## 变异表替换（`mutations-replacements.tsv` 原样）

````tsv
第10l批 新类被抛弃那一半不算进环里还被引用的落点（4 GiB、覆盖写 1 次：环里最旧有效根越过 C 那一次回收出来的 C 的实例表单元空闲）	crates/singlefs-core/src/allocator.rs	                RootRingOccupant::AbandonedRoot {\n                    referenced_placements,\n                    ..\n                }\n                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                    root:\n                        RootReadOnOneRebuildPass::AbandonedRoot {\n                            referenced_placements,\n                        },\n                    ..\n                } => referenced_placements.as_slice(),	                RootRingOccupant::AbandonedRoot {\n                    referenced_placements,\n                    ..\n                } => referenced_placements.as_slice(),\n                RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                    root: RootReadOnOneRebuildPass::AbandonedRoot { .. },\n                    ..\n                } => &[],	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring
第10l批 新类被抛弃那一半被换下时不清隔离（4 GiB、覆盖写 1 次：C 离开根环之后它写出的落点仍隔离、不空闲）	crates/singlefs-core/src/allocator.rs	            Some(\n                RootRingOccupant::AbandonedRoot {\n                    referenced_placements,\n                    ..\n                }\n                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                    root:\n                        RootReadOnOneRebuildPass::AbandonedRoot {\n                            referenced_placements,\n                        },\n                    ..\n                },\n            ) => self.clear_isolation_nothing_still_protected_backs(	            Some(RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::AbandonedRoot { .. },\n                ..\n            }) => {}\n            Some(RootRingOccupant::AbandonedRoot {\n                referenced_placements,\n                ..\n            }) => self.clear_isolation_nothing_still_protected_backs(	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring
````
