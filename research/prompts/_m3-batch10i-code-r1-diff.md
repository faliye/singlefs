# 根环实现第 10i 批代码轮第一轮：改动全文（2026-10-06）

只有一份补丁：第 10i 批，相对它合入那一刻的主工作区。

## batch10i

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10i/patch/`
- `crates.patch`：sha256 8a74a3413cfc75ba7ccbd66e5c12b891227c31c110c45f8a043754aba7f9dafc
- `mutations-append.tsv`：sha256 de76e66587071b7f7c2ccc9c7a6a43935b2212639f0e44de57e5691683635aa7
- `mutations-replacements.tsv`：sha256 4945c48063818cb5d9a47366974813257cfcec645aa8bf3c80acd796ff677e04
- `report.md`：sha256 88ccbc63a4cc1858de5772db33ea131234e9bc27f5396da17a4e99765b29dfef

### crates.patch

````
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 7117703..fb08cd4 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -1005,6 +1005,19 @@ pub enum RootRingOccupant {
     /// 「根槽这一次读坏」那一行，用户 2026-10-04 定，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1）：
     /// 之后那一槽再读坏，算上限、判回退候选时重读一次、仍坏就拒，不按没有根往下走。
     ValidRootReadWhenTheSlotBadAtMountWasReread { checkpoint_txg: CheckpointTxg },
+    /// 重建分配器读根环有一块盘没读出有效根、整环重读了一遍，两遍按槽并起来是一条自证过的根，而另一遍那一槽没自证过（读不出、读得出而
+    /// 自证不过或读回全零）：并出来的照读出来的那条根算（有效的进环里最旧有效根，被抛弃的带着它引用的落点，离开根环时按它清），
+    /// 另记 `txgs_it_can_hold`——这一槽当作挂载那一刻读坏时能住的 txg（[`TxgsARingSlotBadAtMountCanHold`]，按窗口锚与新实例第一次发布的 txg 算，
+    /// 再并进读出来那条根的 txg）。抬 F 那一遍读它读坏、重读一次仍坏时照 [`RootRingOccupant::BadAtMount`] 处置：回收门槛钉在它可能住过的最旧 txg、
+    /// 准入抬 F 的上限按它能住的最大 txg 并，不拒这一次抬（用户 2026-10-06 定「另立一题改」，判决 `research/prompts/m3-batch10-code-r3-main-verification.md`
+    /// 第六节 S1：之前记成 [`RootRingOccupant::ValidRoot`] / [`RootRingOccupant::AbandonedRoot`]，算「这个进程知道住着根的槽」，抬 F 重读仍坏就拒，
+    /// 窄盘上的可写挂载在写行、暖机之后才报 `FloorRaiseFailedAfterTheMountsPublishes`）。管理员回退判候选与算 F_生效 那一遍照旧把它算进
+    /// 「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]：D16（发布语义） 已定项 1「根槽这一次读坏」那一行字面上
+    /// 它是「挂载那一刻读得出」的槽；这一类在那两处怎么处置条文没写，交主 agent）。
+    RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+        root: RootReadOnOneRebuildPass,
+        txgs_it_can_hold: TxgsARingSlotBadAtMountCanHold,
+    },
     /// 按实例表判仍然有效的根，txg 在不在 F 之下都算（可再分配谓词取 max(F_生效, 环里最旧有效根)，F 那一半另记）。
     ValidRoot { checkpoint_txg: CheckpointTxg },
     /// 按实例表判被抛弃的根（D23（journal 的角色与格式） 已定项 14），带着它引用的每个落点——挂载那一刻仍被候选根或现行账引用、
@@ -1015,6 +1028,18 @@ pub enum RootRingOccupant {
     },
 }
 
+/// [`RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`] 那一槽里读出来的根按最新那条根的实例表是哪一类：
+/// 与 [`RootRingOccupant::ValidRoot`]、[`RootRingOccupant::AbandonedRoot`] 两类同一个分法、同样带着的东西。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum RootReadOnOneRebuildPass {
+    ValidRoot {
+        checkpoint_txg: CheckpointTxg,
+    },
+    AbandonedRoot {
+        referenced_placements: Vec<PlacementOnDevice>,
+    },
+}
+
 /// 分配器眼里的根环，只住内存：挂载时从盘上读（`mount::rebuilt_allocator`），mkfs 同一个进程里按 mkfs 刚写下的样子装
 /// （`make_filesystem::root_ring_occupancy_after_make_filesystem`），之后这个进程每写一条根记一条
 /// （`PoolAllocator::record_root_written_by_this_process`）。它回答两件事：环里最旧有效根往前挪了没有（D16（发布语义） 已定项 1 的
@@ -1063,6 +1088,9 @@ impl RootRingOccupancy {
     /// 这张表随之退回，记着的只剩根落盘做成的那几次——，与进程内记根时重读读出了有效根的槽
     /// （[`RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread`]，用户 2026-10-04 定）。挂载那一刻读坏、
     /// 之后又没写过也没重读出有效根的槽不在其内（[`RootRingOccupant::BadAtMount`] 与 [`RootRingOccupant::AllZeroAtMount`] 记着的槽也不在）。
+    /// 重建分配器两遍里只有一遍自证过的槽（[`RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`]）在其内：管理员回退判候选、
+    /// 算 F_生效 读到它仍坏照旧拒；抬 F 那一遍另按 [`Self::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes`] 处置
+    /// （`mount::raise_rollback_floor` 一族把它们从这一份里拿掉）。
     #[must_use]
     pub fn ring_slots_known_to_hold_a_root(&self) -> BTreeSet<RootRingSlot> {
         self.occupants
@@ -1070,13 +1098,37 @@ impl RootRingOccupancy {
             .filter(|(_, occupant)| match occupant {
                 RootRingOccupant::ValidRoot { .. }
                 | RootRingOccupant::AbandonedRoot { .. }
-                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => true,
+                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
+                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => true,
                 RootRingOccupant::BadAtMount(_) | RootRingOccupant::AllZeroAtMount { .. } => false,
             })
             .map(|(ring_slot, _)| *ring_slot)
             .collect()
     }
 
+    /// 重建分配器两遍里只有一遍自证过、还记在表里的槽（[`RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`]），
+    /// 各带它当作挂载那一刻读坏时能住的 txg，按槽的次序：抬 F 那一遍读它读坏、重读一次仍坏时，回收门槛按其中可能住过的最旧 txg 钉、
+    /// 准入抬 F 的上限按其中能住的最大 txg 并（用户 2026-10-06 定，S1）。这个进程写掉那一槽之后就不在其内。
+    #[must_use]
+    pub fn ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes(
+        &self,
+    ) -> BTreeMap<RootRingSlot, TxgsARingSlotBadAtMountCanHold> {
+        self.occupants
+            .iter()
+            .filter_map(|(ring_slot, occupant)| match occupant {
+                RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                    txgs_it_can_hold,
+                    ..
+                } => Some((*ring_slot, *txgs_it_can_hold)),
+                RootRingOccupant::ValidRoot { .. }
+                | RootRingOccupant::BadAtMount(_)
+                | RootRingOccupant::AllZeroAtMount { .. }
+                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
+                | RootRingOccupant::AbandonedRoot { .. } => None,
+            })
+            .collect()
+    }
+
     /// 环里最旧有效根的 txg（D16（发布语义） 已定项 1「环里最旧有效根」，用户 2026-10-04 定 RGall：回收门槛从一圈窗口退回整个环）：
     /// 这张表里全部有效根与挂载时读坏、读回全零的槽（钉在它可能住过的最旧 txg，用户 2026-10-04 定；读回全零的 2026-10-05 定乙）取最小，
     /// 不按 txg 离这个进程记到的最新根多远滤。
@@ -1088,6 +1140,10 @@ impl RootRingOccupancy {
                 RootRingOccupant::ValidRoot { checkpoint_txg, .. }
                 | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                     checkpoint_txg,
+                }
+                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                    root: RootReadOnOneRebuildPass::ValidRoot { checkpoint_txg },
+                    ..
                 } => Some(*checkpoint_txg),
                 RootRingOccupant::BadAtMount(txgs_it_can_hold) => {
                     Some(txgs_it_can_hold.oldest_txg_it_can_have_held())
@@ -1095,7 +1151,11 @@ impl RootRingOccupancy {
                 RootRingOccupant::AllZeroAtMount {
                     oldest_txg_it_can_have_held,
                 } => Some(*oldest_txg_it_can_have_held),
-                RootRingOccupant::AbandonedRoot { .. } => None,
+                RootRingOccupant::AbandonedRoot { .. }
+                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                    root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
+                    ..
+                } => None,
             })
             .min()
     }
@@ -1107,11 +1167,22 @@ impl RootRingOccupancy {
             .flat_map(|occupant| match occupant {
                 RootRingOccupant::AbandonedRoot {
                     referenced_placements,
+                }
+                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                    root:
+                        RootReadOnOneRebuildPass::AbandonedRoot {
+                            referenced_placements,
+                        },
+                    ..
                 } => referenced_placements.as_slice(),
                 RootRingOccupant::ValidRoot { .. }
                 | RootRingOccupant::BadAtMount(_)
                 | RootRingOccupant::AllZeroAtMount { .. }
-                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => &[],
+                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
+                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                    root: RootReadOnOneRebuildPass::ValidRoot { .. },
+                    ..
+                } => &[],
             })
             .copied()
             .collect()
@@ -1132,7 +1203,8 @@ impl RootRingOccupancy {
                 RootRingOccupant::ValidRoot { .. }
                 | RootRingOccupant::AllZeroAtMount { .. }
                 | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
-                | RootRingOccupant::AbandonedRoot { .. } => None,
+                | RootRingOccupant::AbandonedRoot { .. }
+                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
             })
             .collect()
     }
@@ -1165,7 +1237,8 @@ impl RootRingOccupancy {
                 RootRingOccupant::ValidRoot { .. }
                 | RootRingOccupant::BadAtMount(_)
                 | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
-                | RootRingOccupant::AbandonedRoot { .. } => None,
+                | RootRingOccupant::AbandonedRoot { .. }
+                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
             })
             .collect()
     }
@@ -1943,9 +2016,18 @@ impl PoolAllocator {
         let still_referenced_by_abandoned_roots =
             occupancy.placements_referenced_by_abandoned_roots();
         match displaced {
-            Some(RootRingOccupant::AbandonedRoot {
-                referenced_placements,
-            }) => self.clear_isolation_no_abandoned_root_in_the_ring_backs(
+            Some(
+                RootRingOccupant::AbandonedRoot {
+                    referenced_placements,
+                }
+                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                    root:
+                        RootReadOnOneRebuildPass::AbandonedRoot {
+                            referenced_placements,
+                        },
+                    ..
+                },
+            ) => self.clear_isolation_no_abandoned_root_in_the_ring_backs(
                 &referenced_placements,
                 &still_referenced_by_abandoned_roots,
             ),
@@ -1953,7 +2035,11 @@ impl PoolAllocator {
                 RootRingOccupant::ValidRoot { .. }
                 | RootRingOccupant::BadAtMount(_)
                 | RootRingOccupant::AllZeroAtMount { .. }
-                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. },
+                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
+                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                    root: RootReadOnOneRebuildPass::ValidRoot { .. },
+                    ..
+                },
             )
             | None => {}
         }
@@ -1993,10 +2079,21 @@ impl PoolAllocator {
                     }
                     | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                         checkpoint_txg: txg_of_the_occupant,
+                    }
+                    | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                        root:
+                            RootReadOnOneRebuildPass::ValidRoot {
+                                checkpoint_txg: txg_of_the_occupant,
+                            },
+                        ..
                     } => Some(*txg_of_the_occupant),
                     RootRingOccupant::BadAtMount(_)
                     | RootRingOccupant::AllZeroAtMount { .. }
-                    | RootRingOccupant::AbandonedRoot { .. } => None,
+                    | RootRingOccupant::AbandonedRoot { .. }
+                    | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                        root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
+                        ..
+                    } => None,
                 })
                 .chain(std::iter::once(checkpoint_txg))
                 .collect();
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index eafd007..77c2498 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -19,7 +19,7 @@ use crate::allocation_record_tree::{
 use crate::allocator::{
     oldest_txg_in_the_ring_rereading_slots_bad_at_mount, AllocationRecord, DeviceFreeMap,
     Placement, PlacementOnDevice, PlacementRefusal, PoolAllocator, ReclaimedReuse,
-    RereadOfARootRingSlotBadAtMount, RootRingOccupancy, RootRingOccupant,
+    RereadOfARootRingSlotBadAtMount, RootReadOnOneRebuildPass, RootRingOccupancy, RootRingOccupant,
     TxgsARingSlotBadAtMountCanHold, UnitAreaStart,
 };
 use crate::block_device::BlockDevice;
@@ -53,6 +53,7 @@ use crate::recovery::{
     highest_tree_identifier_watermark_in_the_ring, instance_table_chain_of_root,
     instance_table_of_root, instance_table_page_pointers_as_far_as_readable, readable_roots,
     readable_roots_rereading_ring_slots_known_to_hold_a_root_once,
+    readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once,
     rebuild_version_in_the_unit_area_starting_at, replay_journal,
     root_is_abandoned_by_the_instance_table, scan_journal,
     system_configuration_slots_of_one_device_read_once,
@@ -181,6 +182,7 @@ pub enum MountError {
     /// `allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`），这一次读坏（`first_reading`），重读一次仍坏（`reread`）：
     /// 这条根在不在判不了，不按有根或没根猜，这次抬 F 拒绝（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-09-26 定；
     /// C562（抬 F 算上限时根槽读坏当没有根））。在回收、影子账与任何写之前返回：盘上逐字节不变、分配器不动、调用方的现行版本不动。
+    /// 这个进程的准入抬 F 里重建分配器两遍里只有一遍自证过的槽不走这一条（重读仍坏照挂载那一刻读坏并进上限，用户 2026-10-06 定 S1）。
     RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
         ring_slot: RootRingSlot,
         first_reading: BadRootRingSlotReading,
@@ -378,7 +380,8 @@ pub enum StillUnreadableAfterOneReread {
     /// 挂着时抬 F 重算影子账与回收门槛读根环（`recovery::readable_roots_rereading_ring_slots_known_to_hold_a_root_once`）或管理员回退
     /// 判候选、算 F_生效 读根环：`ring_slot` 是这个进程知道住着一条根的槽（挂载那一刻读得出、或这个进程写过且 FUA 返回过），这一次读坏
     /// （`first_reading`：读不出或自证不过），重读一次仍坏（`reread`）——那一槽里的根在不在判不了，不按有根或没根猜（D16（发布语义） 已定项 1
-    /// 「根槽这一次读坏」那一行全句，用户 2026-09-26 定）。挂载那一刻就读坏、之后没写过的槽不走这一条，当没有根。
+    /// 「根槽这一次读坏」那一行全句，用户 2026-09-26 定）。挂载那一刻就读坏、之后没写过的槽不走这一条，当没有根；重建分配器两遍里只有一遍
+    /// 自证过的槽在抬 F 那一遍也不走这一条（照挂载那一刻读坏钉住，用户 2026-10-06 定 S1），管理员回退那一遍照走。
     /// 实审 A3b 报告 Q4：与乙那一族同级，改之前是 `RecoveryFailure::RootRingSlotStillUnreadableAfterOneReread`（那时不分槽知不知道住着根）。
     RootRingSlotKnownToHoldARoot {
         ring_slot: RootRingSlot,
@@ -1829,6 +1832,44 @@ fn merged_readings_of_the_two_root_ring_passes(
         .collect()
 }
 
+/// 重建分配器读根环的两遍里只有一遍自证过的槽（另一遍读不出、读得出而自证不过或读回全零）：并出来是那一遍读出的根
+/// （[`better_reading_of_a_root_ring_slot_over_the_two_passes`]），根环表把它另记一类（用户 2026-10-06 定 S1，
+/// `allocator::RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`）。两遍按同一个次序读同一组槽，槽对不上是这里的 bug、断言。
+fn ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes(
+    first_readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
+    readings_of_the_reread: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
+) -> BTreeSet<RootRingSlot> {
+    use RootRingSlotReadingTellingNeverWrittenApart::{
+        AllZeroNeverWritten, BadAndNotAllZero, SelfVerified,
+    };
+    assert_eq!(
+        first_readings.len(),
+        readings_of_the_reread.len(),
+        "两遍都按 every_root_ring_slot 读整个环，槽数相同"
+    );
+    first_readings
+        .iter()
+        .zip(readings_of_the_reread)
+        .filter_map(
+            |((ring_slot_of_the_first_pass, first_reading), (ring_slot, reading_of_the_reread))| {
+                assert_eq!(
+                    ring_slot_of_the_first_pass, ring_slot,
+                    "两遍都按 every_root_ring_slot 的次序读，第 n 个读数是同一个槽"
+                );
+                match (first_reading, reading_of_the_reread) {
+                    (SelfVerified(_), BadAndNotAllZero | AllZeroNeverWritten)
+                    | (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(_)) => Some(*ring_slot),
+                    (SelfVerified(_), SelfVerified(_))
+                    | (
+                        BadAndNotAllZero | AllZeroNeverWritten,
+                        BadAndNotAllZero | AllZeroNeverWritten,
+                    ) => None,
+                }
+            },
+        )
+        .collect()
+}
+
 /// 挂载那一刻读坏（设备报读错，或读得出、不是全零而自证不过）的根环槽各能住哪几个 txg（区域内槽位只由 txg 定，
 /// D22（单元原子性怎么合成） 已定项 2）：
 /// - 可能住过的最旧 txg：落到这个槽、小于 `first_txg` 的最小 txg。回收门槛钉在这里（用户 2026-10-04 定：读不出、自证不过的根槽在回收门槛里
@@ -2034,6 +2075,10 @@ fn rebuilt_allocator<Device: BlockDevice>(
     let window_anchor_of_the_first_readings =
         newest_valid_root_txg_of_the_readings(&first_readings)
             .filter(|_| a_valid_root_was_read_on_every_device(&first_readings));
+    // 两遍里只有一遍自证过的槽另记一类（用户 2026-10-06 定 S1，`RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`）：
+    // 并出来照读出来的根算，抬 F 那一遍读它仍坏时照挂载那一刻读坏处置、不拒。只读了一遍的没有这一类（保持空集）。
+    let mut ring_slots_self_verified_on_only_one_of_the_two_passes: BTreeSet<RootRingSlot> =
+        BTreeSet::new();
     let (ring_slot_readings, window_anchor, root_ring_of_the_rebuilt_allocator) =
         match window_anchor_of_the_first_readings {
             Some(window_anchor) => (
@@ -2044,6 +2089,11 @@ fn rebuilt_allocator<Device: BlockDevice>(
             None => {
                 before_the_one_reread.before_rereading();
                 let readings_of_the_reread = read_every_root_ring_slot_once();
+                ring_slots_self_verified_on_only_one_of_the_two_passes =
+                    ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes(
+                        &first_readings,
+                        &readings_of_the_reread,
+                    );
                 let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(
                     &first_readings,
                     readings_of_the_reread,
@@ -2188,6 +2238,36 @@ fn rebuilt_allocator<Device: BlockDevice>(
     // 读回全零的槽记成「挂载时读回全零」，只在回收门槛里钉在它可能住过的最旧 txg（用户 2026-10-05 定乙）。两类都钉到这个进程写到那个槽、
     // 或进程内重读读出有效根或被抛弃的根为止（用户 2026-10-04 定）。同一个槽这一遍读出了根的，读出来的那条在后面、盖掉它
     // （这一遍读的是同一个槽，几样不会同时成立，留着次序只为不依赖这一点）。
+    // 两遍里只有一遍自证过的槽当作挂载那一刻读坏时能住的 txg：与读坏的槽同一个算法（[`txgs_ring_slots_bad_at_mount_can_hold`]），
+    // 再并进读出来那条根的 txg——第 0 代根住在落点公式算不到的槽里（`make_filesystem::genesis_root_ring_slots`），公式给不出或给的不含它。
+    let txgs_ring_slots_self_verified_on_only_one_pass_can_hold: BTreeMap<
+        RootRingSlot,
+        TxgsARingSlotBadAtMountCanHold,
+    > = txgs_ring_slots_bad_at_mount_can_hold(
+        &ring_slots_self_verified_on_only_one_of_the_two_passes,
+        window_anchor,
+        first_txg,
+        slots_per_region,
+    )
+    .into_iter()
+    .collect();
+    let txgs_it_can_hold_if_read_bad = |ring_slot: RootRingSlot, root: &RootRecord| {
+        txgs_ring_slots_self_verified_on_only_one_pass_can_hold
+            .get(&ring_slot)
+            .map_or_else(
+                || TxgsARingSlotBadAtMountCanHold::new(root.checkpoint_txg, root.checkpoint_txg),
+                |txgs_it_can_hold| {
+                    TxgsARingSlotBadAtMountCanHold::new(
+                        txgs_it_can_hold
+                            .oldest_txg_it_can_have_held()
+                            .min(root.checkpoint_txg),
+                        txgs_it_can_hold
+                            .highest_txg_it_can_hold()
+                            .max(root.checkpoint_txg),
+                    )
+                },
+            )
+    };
     let occupants: Vec<(RootRingSlot, RootRingOccupant)> = ring_slots_bad_at_mount
         .iter()
         .map(|(ring_slot, txgs_it_can_hold)| {
@@ -2204,18 +2284,39 @@ fn rebuilt_allocator<Device: BlockDevice>(
             },
         ))
         .chain(roots_with_ring_slots.iter().map(|(ring_slot, root)| {
-            let occupant = if is_abandoned(root) {
-                RootRingOccupant::AbandonedRoot {
+            let root_read = if is_abandoned(root) {
+                RootReadOnOneRebuildPass::AbandonedRoot {
                     referenced_placements: placements_referenced_by_abandoned_root
                         .get(&(root.instance, root.checkpoint_txg))
                         .cloned()
                         .unwrap_or_default(),
                 }
             } else {
-                RootRingOccupant::ValidRoot {
+                RootReadOnOneRebuildPass::ValidRoot {
                     checkpoint_txg: root.checkpoint_txg,
                 }
             };
+            let self_verified_on_only_one_of_the_two_passes =
+                ring_slots_self_verified_on_only_one_of_the_two_passes.contains(ring_slot);
+            let occupant = match (self_verified_on_only_one_of_the_two_passes, root_read) {
+                (true, root_read) => {
+                    RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
+                        root: root_read,
+                        txgs_it_can_hold: txgs_it_can_hold_if_read_bad(*ring_slot, root),
+                    }
+                }
+                (false, RootReadOnOneRebuildPass::ValidRoot { checkpoint_txg }) => {
+                    RootRingOccupant::ValidRoot { checkpoint_txg }
+                }
+                (
+                    false,
+                    RootReadOnOneRebuildPass::AbandonedRoot {
+                        referenced_placements,
+                    },
+                ) => RootRingOccupant::AbandonedRoot {
+                    referenced_placements,
+                },
+            };
             (*ring_slot, occupant)
         }))
         .collect();
@@ -2383,14 +2484,18 @@ pub fn user_visible_trees_changed(
 /// 算抬 F 的上限时读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-09-26 定）：每个槽读一次；读坏的槽按这个进程知不知道
 /// 那里住着一条根分两类——`ring_slots_known_to_hold_a_root` 里没有的（挂载那一刻就读不出或自证不过、之后也没写过）当没有根，
 /// 有的（挂载那一刻读得出、或这个进程写过且 FUA 返回过）重读一次，读出来就用重读的那一条，仍坏就拒，不按有根或没根猜。
+/// `ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread` 里的槽（重建分配器两遍里只有一遍自证过的，用户 2026-10-06 定 S1）
+/// 读坏也重读一次、读出来就用重读的那一条，仍坏不拒、当没有根，交给调用方照挂载那一刻读坏处置（上限按它能住的最大 txg 并）。
 /// 交回读得出的每一条根（按根环槽的次序）与这一遍读坏、当没有根的那几个槽。
 ///
 /// # Errors
-/// 知道住着根的槽重读仍坏 ⇒ `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
+/// 知道住着根的槽（不在 `ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread` 里的）重读仍坏 ⇒
+/// `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
 fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     system_configuration: &crate::system_configuration::SystemConfiguration,
     ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
+    ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread: &BTreeSet<RootRingSlot>,
 ) -> Result<RootsOfTheRingForTheCeiling, MountError> {
     let read_once = |ring_slot: RootRingSlot| {
         read_root_ring_slot(
@@ -2412,6 +2517,15 @@ fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
             }
             RootRingSlotReading::Bad(first_reading) => first_reading,
         };
+        if ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread.contains(&ring_slot) {
+            match read_once(ring_slot) {
+                RootRingSlotReading::SelfVerified(root) => roots.push(root),
+                RootRingSlotReading::Bad(_still_bad) => {
+                    ring_slots_bad_on_this_read_taken_as_holding_no_root.insert(ring_slot);
+                }
+            }
+            continue;
+        }
         if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {
             ring_slots_bad_on_this_read_taken_as_holding_no_root.insert(ring_slot);
             continue;
@@ -2439,7 +2553,7 @@ fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
 struct RootsOfTheRingForTheCeiling {
     /// 读得出、自证过的每一条根，按根环槽的次序。
     roots: Vec<RootRecord>,
-    /// 这一遍读不出或自证不过、又不是「这个进程知道住着根的槽」的槽：当没有根。
+    /// 这一遍读不出或自证不过、又不是「这个进程知道住着根的槽」的槽，与重建分配器两遍里只有一遍自证过、这一遍重读一次仍坏的槽：当没有根。
     ring_slots_bad_on_this_read_taken_as_holding_no_root: BTreeSet<RootRingSlot>,
 }
 
@@ -2477,6 +2591,7 @@ pub fn rollback_floor_ceiling<Device: BlockDevice>(
         table,
         ring_slots_known_to_hold_a_root,
         &BTreeMap::new(),
+        &BTreeMap::new(),
     )
 }
 
@@ -2488,6 +2603,9 @@ pub fn rollback_floor_ceiling<Device: BlockDevice>(
 /// （N = 挂载时环里读得出的最新有效根，`rebuilt_allocator`），各按它在那里能住的最大 txg 并：读不出的那一类与回收门槛记的同一个数，
 /// 自证不过的那一类同一个算法（区域内槽位只由 txg 定）。这一遍读那一槽又读坏（读不出或自证不过）、那个 txg 不低于今天的 F，上限就取不高于它。
 /// 这一遍读得出的照读出来的根算，不另封顶。判「非空」、找「前一条」照旧只在读得出的根上取（读不出的那一条判不了空不空，F 不越过它就罩住了它是第 4 新的那一种）。
+/// `ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes` 是这个进程根环表里重建分配器两遍里只有一遍自证过的槽，
+/// 各按它当作挂载那一刻读坏时能住的最大 txg（[`ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by`]，用户 2026-10-06 定 S1）：
+/// 这一遍读坏就重读一次，读出来照读出来的算，仍坏不拒、照挂载那一刻读坏那一类并进上限（它们也在 `ring_slots_known_to_hold_a_root` 里，这里先按它们分）。
 ///
 /// # Errors
 /// 同 [`rollback_floor_ceiling`]。
@@ -2502,6 +2620,10 @@ pub fn rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount<Device:
     table: &InstanceTableRecords,
     ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
     ring_slots_bad_since_the_mount: &BTreeMap<RootRingSlot, CheckpointTxg>,
+    ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes: &BTreeMap<
+        RootRingSlot,
+        CheckpointTxg,
+    >,
 ) -> Result<CheckpointTxg, MountError> {
     let RootsOfTheRingForTheCeiling {
         roots: readable,
@@ -2510,7 +2632,17 @@ pub fn rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount<Device:
         devices,
         system_configuration,
         ring_slots_known_to_hold_a_root,
+        &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes
+            .keys()
+            .copied()
+            .collect(),
     )?;
+    let ring_slots_bad_since_the_mount: BTreeMap<RootRingSlot, CheckpointTxg> =
+        ring_slots_bad_since_the_mount
+            .iter()
+            .chain(ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes)
+            .map(|(ring_slot, highest_txg_it_can_hold)| (*ring_slot, *highest_txg_it_can_hold))
+            .collect();
     let valid: Vec<RootRecord> = readable
         .iter()
         .copied()
@@ -2903,6 +3035,51 @@ pub fn ring_slots_known_to_hold_a_root_by(allocator: &PoolAllocator) -> BTreeSet
     )
 }
 
+/// 这个进程分配器上那张根环表里重建分配器两遍里只有一遍自证过、之后没写过的槽，各带它当作挂载那一刻读坏时能住的 txg
+/// （`allocator::RootRingOccupancy::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes`，用户 2026-10-06 定 S1）；
+/// 没装根环表的分配器给空表。抬 F 那一遍读它们读坏、重读一次仍坏时照挂载那一刻读坏处置、不拒（[`raise_rollback_floor`] 一族）。
+#[must_use]
+pub fn ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(
+    allocator: &PoolAllocator,
+) -> BTreeMap<RootRingSlot, TxgsARingSlotBadAtMountCanHold> {
+    allocator.root_ring_occupancy().map_or_else(
+        BTreeMap::new,
+        RootRingOccupancy::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes,
+    )
+}
+
+/// 这个进程的准入抬 F 的上限（[`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]），三类槽都取这个进程分配器上那张根环表：
+/// 知道住着根的槽、挂载那一刻读坏的槽、重建分配器两遍里只有一遍自证过的槽（后一类按能住的最大 txg 并，用户 2026-10-06 定 S1）。
+///
+/// # Errors
+/// 同 [`rollback_floor_ceiling`]。
+#[allow(
+    clippy::ptr_arg,
+    reason = "rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount 走 &Vec：要一个有大小的读者"
+)]
+fn rollback_floor_ceiling_of_this_process<Device: BlockDevice>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    system_configuration: &crate::system_configuration::SystemConfiguration,
+    current_floor: CheckpointTxg,
+    table: &InstanceTableRecords,
+    allocator: &PoolAllocator,
+) -> Result<CheckpointTxg, MountError> {
+    rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
+        devices,
+        system_configuration,
+        current_floor,
+        table,
+        &ring_slots_known_to_hold_a_root_by(allocator),
+        &ring_slots_bad_at_mount_by(allocator),
+        &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator)
+            .into_iter()
+            .map(|(ring_slot, txgs_it_can_hold)| {
+                (ring_slot, txgs_it_can_hold.highest_txg_it_can_hold())
+            })
+            .collect(),
+    )
+}
+
 /// 一次准入最多推几次发布：B = 4 + 2 k_tol，k_tol = 2 ⇒ 8（D16（发布语义） 已定项 1「准入」那一行）。数的是这次准入推的空发布
 /// 加上这次准入为之判的那一次发布（发布路径是那次用户发布，可写挂载那一处是写行那次）：`df` 的保留池按 B − 1 次空发布留
 /// （同一张表「`df`」那一行：保留池 = 2 × 5 + (B − 1) × c_max）。
@@ -2960,13 +3137,12 @@ fn raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk<Device
     let current_instance_table = instance_table_chain_of_root(&*devices, &current.root)
         .map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?
         .records;
-    let ceiling = rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
+    let ceiling = rollback_floor_ceiling_of_this_process(
         devices,
         &caller_inputs_agreeing_with_the_disk.system_configuration,
         current.root.rollback_floor,
         &current_instance_table,
-        &ring_slots_known_to_hold_a_root_by(allocator),
-        &ring_slots_bad_at_mount_by(allocator),
+        allocator,
     )?;
     if ceiling <= current.root.rollback_floor {
         return Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling {
@@ -3134,16 +3310,104 @@ pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
     }
 }
 
+/// 抬 F 那一遍重算影子账与回收门槛读根环读出来的（[`readable_roots_for_raising_the_floor`]）。
+struct ReadableRootsForRaisingTheFloor {
+    /// 读得出、自证过的每一条根（重读读出来的排在后面）。
+    roots: Vec<RootRecord>,
+    /// 重建分配器两遍里只有一遍自证过的槽，这一遍读坏、重读一次仍坏的：各按它当作挂载那一刻读坏时可能住过的最旧 txg。
+    ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad:
+        BTreeMap<RootRingSlot, CheckpointTxg>,
+}
+
+/// 抬 F 那一遍重算影子账与回收门槛读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）：每个槽读一次；读坏的槽分三类——
+/// 这个进程不知道住着根的当没有根；知道住着根的重读一次，仍坏就拒（C554 乙报告 Q6）；重建分配器两遍里只有一遍自证过的
+/// （[`ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by`]，它们也在「知道住着根的槽」里，这里先按它们分）重读一次，
+/// 读出来照读出来的算，仍坏不拒、交给调用方照挂载那一刻读坏钉在它可能住过的最旧 txg（用户 2026-10-06 定 S1：之前与知道住着根的槽同一处置，
+/// 窄盘上的可写挂载在写行、暖机之后推的抬 F 被拒）。只读盘。
+///
+/// # Errors
+/// 知道住着根的槽（两遍里只有一遍自证过的除外）重读仍坏 ⇒
+/// `NewerStateStillUnreadableAfterOneReread(StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot(..))`。
+fn readable_roots_for_raising_the_floor<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    system_configuration: &crate::system_configuration::SystemConfiguration,
+    allocator: &PoolAllocator,
+) -> Result<ReadableRootsForRaisingTheFloor, MountError> {
+    let ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass =
+        ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator);
+    let ring_slots_whose_reread_still_bad_refuses_the_raise: BTreeSet<RootRingSlot> =
+        ring_slots_known_to_hold_a_root_by(allocator)
+            .into_iter()
+            .filter(|ring_slot| {
+                !ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass
+                    .contains_key(ring_slot)
+            })
+            .collect();
+    let mut roots_with_ring_slots =
+        readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once(
+            reader,
+            &system_configuration.immutable.region_devices,
+            &system_configuration.immutable.sizes,
+            &system_configuration.immutable.filesystem_identifier,
+            &ring_slots_whose_reread_still_bad_refuses_the_raise,
+        )
+        .map_err(|still_unreadable| {
+            MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
+        })?;
+    let ring_slots_read: BTreeSet<RootRingSlot> = roots_with_ring_slots
+        .iter()
+        .map(|(ring_slot, _)| *ring_slot)
+        .collect();
+    let mut ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad =
+        BTreeMap::new();
+    // 迭代上界是两遍里只有一遍自证过的槽数（至多根环的槽数）；跨轮只往两张表里加；没有提前出口。上面那一遍读坏、当没有根的才在这里重读一次。
+    for (ring_slot, txgs_it_can_hold) in
+        &ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass
+    {
+        if ring_slots_read.contains(ring_slot) {
+            continue;
+        }
+        match read_root_ring_slot(
+            reader,
+            &system_configuration.immutable.region_devices,
+            &system_configuration.immutable.sizes,
+            &system_configuration.immutable.filesystem_identifier,
+            *ring_slot,
+        ) {
+            RootRingSlotReading::SelfVerified(root) => {
+                roots_with_ring_slots.push((*ring_slot, root))
+            }
+            RootRingSlotReading::Bad(_still_bad) => {
+                ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad
+                    .insert(*ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held());
+            }
+        }
+    }
+    Ok(ReadableRootsForRaisingTheFloor {
+        roots: roots_with_ring_slots
+            .into_iter()
+            .map(|(_, root)| root)
+            .collect(),
+        ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad,
+    })
+}
+
 /// 抬 F 那一遍的回收门槛里「环里最旧有效根」（D16（发布语义） 已定项 1，用户 2026-10-04 定 RGall：不按一圈窗口滤）：这一遍读出来的根里
 /// 按 `table` 有效的，并上分配器根环表里挂载那一刻读坏或读回全零、这一遍也没在那一槽读出根的槽（各钉在它可能住过的最旧 txg，用户 2026-10-04 定，
 /// 读回全零的 2026-10-05 定乙；
-/// 并进来是 RR4，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表一，攻方 B1），取最小；最小的是一个读坏的槽时照 E2
+/// 并进来是 RR4，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表一，攻方 B1），再并上
+/// `ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad`（重建分配器两遍里只有一遍自证过、这一遍重读一次仍坏的槽，
+/// 照挂载那一刻读坏钉在它可能住过的最旧 txg，用户 2026-10-06 定 S1），取最小；最小的是一个读坏的槽时照 E2
 /// 先重读那一槽（[`oldest_txg_in_the_ring_rereading_slots_bad_at_mount`]，与进程内记根同一取法）。只读盘，不改分配器。
 fn oldest_valid_root_in_the_ring_for_raising_the_floor<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     parameters_of_the_pool: &MakeFilesystemParameters,
     allocator: &PoolAllocator,
     roots_read_from_the_ring: &[RootRecord],
+    ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad: &BTreeMap<
+        RootRingSlot,
+        CheckpointTxg,
+    >,
     table: &InstanceTableRecords,
 ) -> Option<CheckpointTxg> {
     let slots_per_region = parameters_of_the_pool.geometry.root_ring_slots_per_region;
@@ -3164,6 +3428,13 @@ fn oldest_valid_root_in_the_ring_for_raising_the_floor<Reader: PoolReader + ?Siz
         )
         .into_iter()
         .filter(|(ring_slot, _)| !ring_slots_read_from_the_ring.contains(ring_slot))
+        .chain(
+            ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad
+                .iter()
+                .map(|(ring_slot, oldest_txg_it_can_have_held)| {
+                    (*ring_slot, *oldest_txg_it_can_have_held)
+                }),
+        )
         .collect();
     oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
         &valid_root_txgs,
@@ -3243,13 +3514,12 @@ fn raise_the_floor_through<Device: BlockDevice>(
     }
     let ceiling_judged_by_the_admission = match entry {
         RaiseFloorEntry::Admission => {
-            let ceiling = rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
+            let ceiling = rollback_floor_ceiling_of_this_process(
                 devices,
                 &system_configuration,
                 current.root.rollback_floor,
                 &table,
-                &ring_slots_known_to_hold_a_root_by(allocator),
-                &ring_slots_bad_at_mount_by(allocator),
+                allocator,
             )?;
             if new_floor > ceiling {
                 return Err(MountError::RollbackFloorAboveCeiling {
@@ -3284,17 +3554,13 @@ fn raise_the_floor_through<Device: BlockDevice>(
     // 记账按写那条根的那一刻的 F 算还是按它持久之后的 F_生效 算，口径交 alloc-basis 那一轮（预想）。
     // 回收门槛与影子账读的这一遍根环，照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行分两类（与算上限那一遍同一个分法）：
     // 这个进程知道住着根的槽读坏就重读那一槽一次，仍坏就拒这一次抬（C554 乙报告 Q6：按「没有根」往下走，那一槽里的被抛弃根引用的槽
-    // 不隔离、也不计数）；挂载那一刻就读坏、之后没写过的槽当没有根。在动分配器（影子账重算、回收）之前返回，分配器与盘都不动。
-    let roots = readable_roots_rereading_ring_slots_known_to_hold_a_root_once(
-        &**devices,
-        &system_configuration.immutable.region_devices,
-        &system_configuration.immutable.sizes,
-        &system_configuration.immutable.filesystem_identifier,
-        &ring_slots_known_to_hold_a_root_by(allocator),
-    )
-    .map_err(|still_unreadable| {
-        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
-    })?;
+    // 不隔离、也不计数）；挂载那一刻就读坏、之后没写过的槽当没有根；重建分配器两遍里只有一遍自证过的槽读坏也重读一次，仍坏不拒、
+    // 照挂载那一刻读坏钉在它可能住过的最旧 txg（用户 2026-10-06 定 S1，[`readable_roots_for_raising_the_floor`]）。
+    // 在动分配器（影子账重算、回收）之前返回，分配器与盘都不动。
+    let ReadableRootsForRaisingTheFloor {
+        roots,
+        ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad,
+    } = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;
     // 根环表里挂载那一刻读坏、这一遍也没读出根的槽，钉在它可能住过的最旧 txg 并进环里最旧有效根（RR4，攻方 B1；用户 2026-10-04 定钉在最旧 txg；与进程内记根同一取法，
     // E2 的重读照用：它成了最小的那个时先重读，`oldest_valid_root_in_the_ring_for_raising_the_floor`）。
     let oldest_valid_root_in_the_ring = oldest_valid_root_in_the_ring_for_raising_the_floor(
@@ -3302,6 +3568,7 @@ fn raise_the_floor_through<Device: BlockDevice>(
         &parameters_of_the_pool,
         allocator,
         &roots,
+        &ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad,
         &table,
     );
     // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index 39d9f5f..5cc7255 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -865,8 +865,8 @@ pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReaso
         | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
         // 盘上的实例代号或 txg 已到顶：坏镜像才有。
         | MountError::SequenceNumberPastTheTopOfItsRange(_)
-        // 重建分配器读根环有一块盘没读出有效根、整环重读那一遍整个环仍一条都没有（Q3 另起的拒因）：读阶段择得到根，只在读路径上注入故障才有，
-        // 模型没有这一条。
+        // 重建分配器读根环有一块盘没读出有效根、整环重读一遍，两遍逐槽并起来整个环仍一条不被抛弃的自证有效根都没有（Q3 另起的拒因；
+        // R1 按槽并两遍之后，第一遍读出过一条的不拒）：读阶段择得到根，只在读路径上注入故障才有，模型没有这一条。
         | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread
         // 读阶段判完之后（判据 N-配置 为假）逐盘核或取号那一刻系统配置又读不出（C664（挂载拒绝合法复算把两种拒因报成一个）之前并在 N-配置 里、
         // 映射成模型那一条理由）：两遍读之间的瞬时读错或一块盘两槽都坏才有，模型没有这一条，不映射成「见证过比所选那一版新的发布」。
diff --git a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
index d94e5b6..4dccd9a 100644
--- a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
+++ b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
@@ -157,6 +157,8 @@ struct RootRingReadsState {
     slots: Vec<(RootRingSlot, DeviceIdentity, u64)>,
     /// （槽，这个槽第几次被读，从 1 数）→ 这一次读成什么。
     faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
+    /// 槽 →（从这个槽第几次被读起，每一次都读成什么）：坏道从这一刻起一直坏。`faulted_reads_of_each_slot` 点名的那一次先照它。
+    slots_failing_from_a_read_onward: BTreeMap<RootRingSlot, (u64, RootRingSlotReadFault)>,
     reads_of_each_slot: BTreeMap<RootRingSlot, u64>,
     faulted_reads: u64,
 }
@@ -185,9 +187,17 @@ impl SharedRootRingReads {
             *reads += 1;
             *reads
         };
-        let fault = *state
+        let fault = state
             .faulted_reads_of_each_slot
-            .get(&(ring_slot, reads_of_this_slot))?;
+            .get(&(ring_slot, reads_of_this_slot))
+            .copied()
+            .or_else(|| {
+                state
+                    .slots_failing_from_a_read_onward
+                    .get(&ring_slot)
+                    .filter(|(first_failing_read, _)| reads_of_this_slot >= *first_failing_read)
+                    .map(|(_, fault)| *fault)
+            })?;
         state.faulted_reads += 1;
         Some((fault, slot_start))
     }
@@ -283,6 +293,10 @@ enum StartingPoint {
     /// （攻方第一次抽样 B 段的 `264:6:4-live-crash` 起点）；重开可写（准入判着、影子账开）做完就崩：两块盘上最新的根都是那次暖机的，
     /// 带着同一个活文件。
     UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
+    /// `narrow-384-20-2`（第三轮攻方原型 `research/prompts/m3-batch10-code-r3-opus-model/main.rs` 的 `narrow_start(384, 20, 2)`，只抄造法）：
+    /// 不补环窗口、准入关着造的窄盘（单元区 384 槽），第一个文件之后覆盖写 20 次、顺序写长到 2 个单元，进程退出（不再挂）。
+    /// 再可写挂载时准入在取号之前不够，写行与暖机之后推抬 F。
+    UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
 }
 
 /// 再挂之前的盘面，与「卸载前」那一份（丢盘整读拿它比；n = now 那一个起点两份相同）。
@@ -471,6 +485,28 @@ fn pool_before_the_remount(starting_point: StartingPoint) -> PoolBeforeTheRemoun
                 parameters: pool.parameters.clone(),
             }
         }
+        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits => {
+            let mut pool =
+                start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+                    HistoryDeviceWidth::UnitAreaOf384Slots,
+                    384,
+                );
+            // 迭代上界是 20 次覆盖写；没有提前出口。
+            for overwrite_index in 0..20 {
+                pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+                    panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
+                });
+            }
+            pool.sequential_write(data_unit_payload_capacity() + 100)
+                .unwrap_or_else(|refusal| panic!("顺序写把文件长到 2 个单元：{refusal:?}"));
+            pool.session = None;
+            let image = pool.image();
+            PoolBeforeTheRemount {
+                image_before_the_crash: image.clone(),
+                image_to_remount: image,
+                parameters: pool.parameters.clone(),
+            }
+        }
     }
 }
 
@@ -715,6 +751,19 @@ fn remount_through(
 fn root_ring_reads_failing(
     image: &MemoryPool,
     faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
+) -> SharedRootRingReads {
+    root_ring_reads_failing_also_from_a_read_onward(
+        image,
+        faulted_reads_of_each_slot,
+        BTreeMap::new(),
+    )
+}
+
+/// 同 [`root_ring_reads_failing`]，另有几个槽从它第几次被读起每一次都读坏。
+fn root_ring_reads_failing_also_from_a_read_onward(
+    image: &MemoryPool,
+    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
+    slots_failing_from_a_read_onward: BTreeMap<RootRingSlot, (u64, RootRingSlotReadFault)>,
 ) -> SharedRootRingReads {
     let system_configuration = choose_system_configuration(image).expect("系统配置");
     let immutable = system_configuration.immutable;
@@ -732,6 +781,7 @@ fn root_ring_reads_failing(
         slot_spacing_in_bytes: u64::from(immutable.sizes.fixed_structure_slot_spacing),
         slots,
         faulted_reads_of_each_slot,
+        slots_failing_from_a_read_onward,
         reads_of_each_slot: BTreeMap::new(),
         faulted_reads: 0,
     })))
@@ -1096,6 +1146,299 @@ fn on_the_264_slot_pool_with_a_live_file_device_one_failing_the_rebuild_pass_rer
     );
 }
 
+/// 场景：`264:6:4`，重建那一遍盘 0 除最新那条根之外、盘 1 每个根环槽读回全 0（`P5:but1/all:z`；第三轮辩方 K3：新起点只跑过读错一种）。
+/// 预期与判别力同 `…device_zero_failing_the_rebuild_pass…` 那一条（盘 1 一条有效根都没读出，整环重读；变异行
+/// 「第10i批 rr2 用例丢盘整读补全零与乱字节：重读之后的读数只剩最新那条根…」）。
+#[test]
+fn on_the_264_slot_pool_with_a_live_file_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
+) {
+    assert_the_live_file_starting_point_has_live_files_to_compare(
+        DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS,
+    );
+    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
+        lone_survivors_after_overwrites_following_the_remount(
+            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS,
+        ),
+    );
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
+            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "有活文件的起点、只坏重建那一遍、盘 1 读回全 0：整环重读、照常挂上，丢盘整读与之前相同"
+    );
+}
+
+/// 场景：`264:6:4`，重建那一遍盘 0 每个、盘 1 除最新那条根之外的根环槽读回乱字节（`P5:all/but1:g`）。
+/// 预期与判别力同上一条（盘 0 一条有效根都没读出，整环重读）。
+#[test]
+fn on_the_264_slot_pool_with_a_live_file_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
+) {
+    assert_the_live_file_starting_point_has_live_files_to_compare(
+        DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE,
+    );
+    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
+        lone_survivors_after_overwrites_following_the_remount(
+            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE,
+        ),
+    );
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
+            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "有活文件的起点、只坏重建那一遍、盘 0 读回乱字节：整环重读、照常挂上，丢盘整读与之前相同"
+    );
+}
+
+/// 一份镜像上盘 `device` 那几个区域里最新那条自证过的根住的根环槽。
+fn ring_slot_of_the_newest_root_on(image: &MemoryPool, device: DeviceIdentity) -> RootRingSlot {
+    let region_devices = choose_system_configuration(image)
+        .expect("系统配置")
+        .immutable
+        .region_devices;
+    self_verified_roots_in_the_ring(image)
+        .into_iter()
+        .filter(|(ring_slot, _)| {
+            region_devices[usize::try_from(ring_slot.region).expect("区域号")] == device
+        })
+        .max_by_key(|(_, root)| (root.checkpoint_txg, root.instance))
+        .map(|(ring_slot, _)| ring_slot)
+        .expect("这块盘上有一条自证过的根")
+}
+
+/// 那一槽在重建那一遍（第一遍）之后怎么读坏。
+#[derive(Clone, Copy, Debug)]
+enum ReadsOfTheSlotFailingAfterTheRebuildPass {
+    /// 从重读那一遍起每一次都报读错（第三轮攻方 `research/prompts/m3-batch10-code-r3-opus-output.md` 第 1.3 节那一形：只在「从重读起一直坏」上出现）。
+    FromTheRereadOnward,
+    /// 重读那一遍与写行、暖机之后推的抬 F 算上限读它那一次（[`READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE`]）报读错，
+    /// 别的读得出（抬 F 那一遍瞬时读坏）。
+    OnTheRereadAndTheFirstReadOfTheFloorRaiseOnly,
+    /// 只有重读那一遍报读错（对照）。
+    OnlyOnTheReread,
+}
+
+/// `narrow-384-20-2` 上照 [`ReadsOfTheSlotFailingAfterTheRebuildPass`] 再挂时，盘 0 最新那条根住的槽第几次被读是写行与暖机之后推的
+/// 第一次抬 F 算上限读根环那一遍（`mount::roots_of_the_ring_for_the_ceiling`）：现量的（第 10i 批在副本上逐次打出读它的调用栈：
+/// 1–5 读阶段，6、7 重建分配器两遍，8、9 取号判最大实例代号，10–15 先写系统配置那几步算生效 F，16 算上限）。挂载里读根环的次序变了，
+/// 这个数跟着变；「第10i批 S1 抬 F 算上限读它读坏一次就拒、不重读」那一行变异就不再红，门禁复跑变异表时报出来。
+const READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE: u64 = 16;
+
+/// S1 那一形再挂之后看到的那几样（各用例自己断言）。
+#[derive(Debug, PartialEq, Eq)]
+struct RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
+    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
+    root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
+    /// 重读之前的钩子调了几次，每次调的那一刻每个根环槽是不是都恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次。
+    hook_calls_after_exactly_the_rebuild_pass: Vec<bool>,
+    /// 这次挂载写行与暖机之后推了抬 F（取号之前准入不够）。
+    floor_raises_pushed_by_the_mount: bool,
+    /// 那一槽在重读那一遍之后还被读过（抬 F 那一遍读了它）。
+    the_slot_was_read_after_the_reread: bool,
+    /// 交回的盘面与「那一槽只在重读那一遍报读错」的再挂交回的逐字节相同。
+    same_disk_as_a_remount_failing_the_slot_only_on_the_reread: bool,
+    /// 两块盘都在时池级 checker 判违例的那几条。
+    checker_violations_with_both_devices: Vec<String>,
+    /// 丢盘 0、丢盘 1 之后剩下那块盘只读挂载、整读每个活文件，与挂载之前两块盘都在时读出的相同（挂载之前最新那条根只落在盘 0 上，
+    /// 那时丢盘 0 读出的是更旧的一版，拿它比不出这次挂载丢没丢东西）。
+    lone_survivor_reads_what_both_devices_read_before_the_remount: [bool; 2],
+    /// 丢盘 0、丢盘 1 之后，挂载之前环里还在、不低于 F 的每条自证根在剩下那块盘上整读与挂载之前相同。
+    lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [bool; 2],
+}
+
+/// `narrow-384-20-2` 上再可写挂载（准入判着、影子账开）：重建那一遍盘 1 每个根环槽报读错（盘 1 一条有效根都没读出，rr2 整环重读），
+/// 盘 0 最新那条根住的槽第一遍读得出、之后照 `reads_of_the_slot` 读坏；按槽并两遍时那一槽只在第一遍自证过。
+fn remount_of_the_narrow_pool_with_the_newest_root_on_device_zero_self_verified_only_on_the_rebuild_pass(
+    reads_of_the_slot: ReadsOfTheSlotFailingAfterTheRebuildPass,
+) -> RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
+    let before = pool_before_the_remount(
+        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
+    );
+    let slot_of_the_newest_root_on_device_zero =
+        ring_slot_of_the_newest_root_on(&before.image_to_remount, DeviceIdentity(0));
+    let device_one_failing_the_rebuild_pass: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> =
+        faulted_slots_of_the_plan(&before.image_to_remount, DEVICE_ONE_REPORTS_READ_ERRORS)
+            .into_iter()
+            .map(|(ring_slot, fault)| {
+                (
+                    (
+                        ring_slot,
+                        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
+                    ),
+                    fault,
+                )
+            })
+            .collect();
+    let remount_with =
+        |reads_of_the_slot_in_this_remount: ReadsOfTheSlotFailingAfterTheRebuildPass| {
+            let mut faulted_reads = device_one_failing_the_rebuild_pass.clone();
+            let mut slots_failing_from_a_read_onward = BTreeMap::new();
+            match reads_of_the_slot_in_this_remount {
+                ReadsOfTheSlotFailingAfterTheRebuildPass::FromTheRereadOnward => {
+                    slots_failing_from_a_read_onward.insert(
+                        slot_of_the_newest_root_on_device_zero,
+                        (
+                            READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
+                            RootRingSlotReadFault::ReadFails,
+                        ),
+                    );
+                }
+                ReadsOfTheSlotFailingAfterTheRebuildPass::OnTheRereadAndTheFirstReadOfTheFloorRaiseOnly => {
+                    faulted_reads.extend(faulted_reads_of(
+                        &[slot_of_the_newest_root_on_device_zero],
+                        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
+                        RootRingSlotReadFault::ReadFails,
+                    ));
+                    faulted_reads.extend(faulted_reads_of(
+                        &[slot_of_the_newest_root_on_device_zero],
+                        READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE,
+                        RootRingSlotReadFault::ReadFails,
+                    ));
+                }
+                ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread => {
+                    faulted_reads.extend(faulted_reads_of(
+                        &[slot_of_the_newest_root_on_device_zero],
+                        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
+                        RootRingSlotReadFault::ReadFails,
+                    ));
+                }
+            }
+            let root_ring_reads = root_ring_reads_failing_also_from_a_read_onward(
+                &before.image_to_remount,
+                faulted_reads,
+                slots_failing_from_a_read_onward,
+            );
+            let (mounted, image_after_the_remount, reads_at_each_hook_call) =
+                remount_through(&before, &root_ring_reads);
+            (
+                mounted,
+                image_after_the_remount,
+                reads_at_each_hook_call,
+                root_ring_reads.reads_of_each_slot(),
+            )
+        };
+    let (
+        mounted,
+        image_after_the_remount,
+        reads_at_each_hook_call,
+        reads_of_each_slot_by_the_end_of_the_remount,
+    ) = remount_with(reads_of_the_slot);
+    let (mounted_for_reference, image_after_the_reference_remount, _, _) =
+        remount_with(ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread);
+    mounted_for_reference
+        .unwrap_or_else(|error| panic!("前提：那一槽只在重读那一遍报读错的再挂做得成：{error:?}"));
+    RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
+        hook_calls_after_exactly_the_rebuild_pass: reads_at_each_hook_call
+            .iter()
+            .map(|reads_of_each_slot| {
+                !reads_of_each_slot.is_empty()
+                    && reads_of_each_slot
+                        .values()
+                        .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD)
+            })
+            .collect(),
+        floor_raises_pushed_by_the_mount: mounted
+            .as_ref()
+            .is_ok_and(|mounted| !mounted.output.space_admission.floor_raises().is_empty()),
+        root_ring_of_the_rebuilt_allocator: mounted
+            .map(|mounted| mounted.output.rereads.root_ring_of_the_rebuilt_allocator)
+            .map_err(|error| format!("{error:?}")),
+        the_slot_was_read_after_the_reread: reads_of_each_slot_by_the_end_of_the_remount
+            .get(&slot_of_the_newest_root_on_device_zero)
+            .is_some_and(|reads| *reads > READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD),
+        same_disk_as_a_remount_failing_the_slot_only_on_the_reread: image_after_the_remount
+            == image_after_the_reference_remount,
+        checker_violations_with_both_devices: checker_violations_on(&image_after_the_remount),
+        lone_survivor_reads_what_both_devices_read_before_the_remount: [
+            DeviceIdentity(0),
+            DeviceIdentity(1),
+        ]
+        .map(|lost| {
+            live_files_read_whole(&image_without(&image_after_the_remount, lost))
+                == live_files_read_whole(&before.image_to_remount)
+        }),
+        lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [
+            DeviceIdentity(0),
+            DeviceIdentity(1),
+        ]
+        .map(|lost| {
+            let (_, _, read_differently) = roots_still_in_the_ring_read_on_a_lone_survivor(
+                &before.image_to_remount,
+                &image_after_the_remount,
+                lost,
+            );
+            read_differently.is_empty()
+        }),
+    }
+}
+
+/// 场景：`narrow-384-20-2`，重建那一遍盘 1 每个根环槽报读错（rr2 整环重读），盘 0 最新那条根住的槽第一遍读得出、从重读那一遍起每次都报读错
+/// （第三轮攻方第 1.3 节那一形）：按槽并两遍，那一槽并成第一遍读出的根，根环表另记「两遍里只有一遍自证过」那一类。
+/// 预期：照常挂上（重读过一次），写行与暖机之后推的抬 F 读那一槽仍坏、重读一次仍坏，照挂载那一刻读坏处置、不拒（用户 2026-10-06 定 S1）；
+/// 两块盘都在时池级 checker 不判违例，丢任一块盘剩下那块盘整读与挂载之前两块盘读出的相同，挂载之前环里还在、不低于 F 的根丢盘整读也相同；
+/// 交回的盘面与「那一槽只在重读那一遍报读错」的再挂相同（这一形里抬 F 抬到的上限不受那一槽封顶，回收门槛按新 F 取、钉住的最旧 txg 在它之下）。
+/// 判别力：改回记成「这个进程知道住着根的槽」（`crates/mutations.tsv`「第10i批 S1 两遍里只有一遍自证过的根改回记成知道住着根的槽」）时
+/// 抬 F 重读那一槽仍坏就拒，挂载在写行、暖机之后报 `FloorRaiseFailedAfterTheMountsPublishes`，第一项红。
+#[test]
+fn a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount(
+) {
+    let remount = remount_of_the_narrow_pool_with_the_newest_root_on_device_zero_self_verified_only_on_the_rebuild_pass(
+        ReadsOfTheSlotFailingAfterTheRebuildPass::FromTheRereadOnward,
+    );
+    assert_eq!(
+        (
+            &remount.root_ring_of_the_rebuilt_allocator,
+            &remount.hook_calls_after_exactly_the_rebuild_pass,
+            remount.floor_raises_pushed_by_the_mount,
+            remount.the_slot_was_read_after_the_reread,
+            remount.same_disk_as_a_remount_failing_the_slot_only_on_the_reread,
+            &remount.checker_violations_with_both_devices,
+            remount.lone_survivor_reads_what_both_devices_read_before_the_remount,
+            remount.lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount,
+        ),
+        (
+            &Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
+            &vec![true],
+            true,
+            true,
+            true,
+            &Vec::<String>::new(),
+            [true, true],
+            [true, true],
+        ),
+        "两遍里只有一遍自证过的根、从重读起一直读坏：照常挂上，抬 F 读它仍坏不拒，checker 不红、丢盘整读不变（{remount:?}）"
+    );
+}
+
+/// 场景：同上，那一槽只在重读那一遍与写行、暖机之后推的抬 F 算上限读它那一次报读错，之后读得出（抬 F 那一遍瞬时读坏）。
+/// 预期：照旧——抬 F 那一遍读它读坏就重读一次、重读读出有效根就照读出来的算，与「那一槽只在重读那一遍报读错」的再挂同一份盘面；
+/// checker 不红，丢盘整读不变。
+/// 判别力：抬 F 算上限那一遍读它读坏一次就拒、不重读（`crates/mutations.tsv`「第10i批 S1 抬 F 算上限读它读坏一次就拒、不重读」）时
+/// 挂载在写行、暖机之后报 `FloorRaiseFailedAfterTheMountsPublishes`，第一项红。
+#[test]
+fn a_root_self_verified_only_on_the_rebuild_pass_read_bad_once_more_after_the_reread_is_taken_as_read_by_the_floor_raise(
+) {
+    assert_eq!(
+        remount_of_the_narrow_pool_with_the_newest_root_on_device_zero_self_verified_only_on_the_rebuild_pass(
+            ReadsOfTheSlotFailingAfterTheRebuildPass::OnTheRereadAndTheFirstReadOfTheFloorRaiseOnly,
+        ),
+        RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
+            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
+            hook_calls_after_exactly_the_rebuild_pass: vec![true],
+            floor_raises_pushed_by_the_mount: true,
+            the_slot_was_read_after_the_reread: true,
+            same_disk_as_a_remount_failing_the_slot_only_on_the_reread: true,
+            checker_violations_with_both_devices: Vec::new(),
+            lone_survivor_reads_what_both_devices_read_before_the_remount: [true, true],
+            lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [true, true],
+        },
+        "两遍里只有一遍自证过的根、之后瞬时读坏一次：抬 F 重读读出来就照读出来的算"
+    );
+}
+
 /// K2-2 那组历史停在进程退出那一刻：盘面、参数，被抛弃的根 C、它住的根环槽，与 C 写出的每个落点（盘、槽、那时盘上的字节）。
 struct PoolWithAnAbandonedRoot {
     image: MemoryPool,
@@ -1139,12 +1482,20 @@ fn write_on(
         .expect("内存盘写得进");
 }
 
-/// 384 槽两块盘：第一个文件、覆盖写一次（实例 1）；崩了再挂（实例 2），覆盖写一次得 C；照 `tests/common` 的
+/// 两块盘（盘宽 `device_width`：K2-2 那组历史两档，4 GiB 与单元区 384 槽）：第一个文件、覆盖写 `overwrites_before_the_abandoned_root` 次
+/// （实例 1；K2-2 两档，1 次与 3 次）；崩了再挂（实例 2），覆盖写一次得 C；照 `tests/common` 的
 /// `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 那一形抛弃 C——C 的根槽与数据单元暂存、清零，见证 C 的那一次系统配置轮换
 /// 写回成轮换之前的那一代，崩了再挂（实例 3）择根落到 C 之前那一条、写行与暖机，再把暂存的字节写回；进程退出。
-fn pool_with_an_abandoned_root() -> PoolWithAnAbandonedRoot {
-    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
-    pool.overwrite(OVERWRITE_BYTES).expect("实例 1 覆盖写一次");
+fn pool_with_an_abandoned_root(
+    device_width: HistoryDeviceWidth,
+    overwrites_before_the_abandoned_root: usize,
+) -> PoolWithAnAbandonedRoot {
+    let mut pool = start_plain(device_width);
+    // 迭代上界是 overwrites_before_the_abandoned_root；没有提前出口。
+    for overwrite_index in 0..overwrites_before_the_abandoned_root {
+        pool.overwrite(OVERWRITE_BYTES)
+            .unwrap_or_else(|refusal| panic!("实例 1 第 {overwrite_index} 次覆盖写：{refusal:?}"));
+    }
     pool.crash_and_mount_writable().expect("崩了再挂：取号 2");
     pool.overwrite(OVERWRITE_BYTES - 499)
         .expect("实例 2 覆盖写一次得 C");
@@ -1486,9 +1837,11 @@ fn merged_and_keeps_the_abandoned_root_isolated() -> RemountOfThePoolWithAnAband
 
 /// 第一遍盘 1 整段报读错、重读只坏 C 的槽（读成 `fault`）的那一格（攻方 `pass1:dev1-all-e reread:c-*`）。
 fn first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+    device_width: HistoryDeviceWidth,
+    overwrites_before_the_abandoned_root: usize,
     fault: RootRingSlotReadFault,
 ) -> RemountOfThePoolWithAnAbandonedRoot {
-    let history = pool_with_an_abandoned_root();
+    let history = pool_with_an_abandoned_root(device_width, overwrites_before_the_abandoned_root);
     let mut faulted_reads = faulted_reads_of(
         &root_ring_slots_of_device_one(&history),
         READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
@@ -1506,7 +1859,30 @@ fn first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots
     )
 }
 
-/// 场景：K2-2 那组历史，第一遍盘 1 每个根环槽报读错（rr2 整环重读），重读那一遍只有 C 的根槽报读错。
+/// 第一遍盘 1 每个根环槽与 C 的根槽都报读错、重读那一遍都读得出的那一格（K2-2 的反方向，攻方 `pass1:dev1-all-e+c-e reread:-`）。
+fn first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(
+    device_width: HistoryDeviceWidth,
+    overwrites_before_the_abandoned_root: usize,
+) -> RemountOfThePoolWithAnAbandonedRoot {
+    let history = pool_with_an_abandoned_root(device_width, overwrites_before_the_abandoned_root);
+    let mut faulted_reads = faulted_reads_of(
+        &root_ring_slots_of_device_one(&history),
+        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
+        RootRingSlotReadFault::ReadFails,
+    );
+    faulted_reads.extend(faulted_reads_of(
+        &[history.ring_slot_of_the_abandoned_root],
+        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
+        RootRingSlotReadFault::ReadFails,
+    ));
+    remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
+        &history,
+        faulted_reads,
+        BTreeMap::new(),
+    )
+}
+
+/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错（rr2 整环重读），重读那一遍只有 C 的根槽报读错。
 /// 预期：按槽并两遍，C 那一槽留第一遍读出的 C——影子账照常隔离 C 独占的单元，盘面与不注错的再挂相同；接着覆盖写到 C 离开根环，
 /// C 写出的落点一个都没被改写，池级 checker 不红（D23（journal 的角色与格式） 已定项 14「被崩溃恢复抛弃的时间线的根离开根环之前，
 /// 它们引用的单元不许重新分配也不许抹头」）。
@@ -1517,10 +1893,12 @@ fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_
 ) {
     assert_eq!(
         first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            1,
             RootRingSlotReadFault::ReadFails
         ),
         merged_and_keeps_the_abandoned_root_isolated(),
-        "重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
+        "单元区 384 槽、实例 1 覆盖写 1 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
     );
 }
 
@@ -1531,10 +1909,12 @@ fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_unit
 ) {
     assert_eq!(
         first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            1,
             RootRingSlotReadFault::ReadsBackAllZero
         ),
         merged_and_keeps_the_abandoned_root_isolated(),
-        "重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
+        "单元区 384 槽、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
     );
 }
 
@@ -1545,39 +1925,239 @@ fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_un
 ) {
     assert_eq!(
         first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            1,
             RootRingSlotReadFault::ReadsBackGarbage
         ),
         merged_and_keeps_the_abandoned_root_isolated(),
-        "重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
+        "单元区 384 槽、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
     );
 }
 
-/// 场景：K2-2 的反方向（攻方 `pass1:dev1-all-e+c-e reread:-`）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出。
+/// 场景：K2-2 的反方向（攻方 `pass1:dev1-all-e+c-e reread:-`；单元区 384 槽、实例 1 覆盖写 1 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出。
 /// 预期：按槽并两遍，C 那一槽取重读读出的 C，与不注错的再挂同一份盘面，影子账照隔离，C 离开根环之前它的落点不被改写，池级 checker 不红。
 /// 判别力：并的时候第一遍读坏压过重读自证过的根（`crates/mutations.tsv`「第10h批 R1 第一遍读坏压过重读自证过的根」）时 C 掉出影子账，
 /// 隔离那一项起红。
 #[test]
 fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
 ) {
-    let history = pool_with_an_abandoned_root();
-    let mut faulted_reads = faulted_reads_of(
-        &root_ring_slots_of_device_one(&history),
-        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
-        RootRingSlotReadFault::ReadFails,
+    assert_eq!(
+        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            1,
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "单元区 384 槽、实例 1 覆盖写 1 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
     );
-    faulted_reads.extend(faulted_reads_of(
-        &[history.ring_slot_of_the_abandoned_root],
-        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
-        RootRingSlotReadFault::ReadFails,
-    ));
+}
+
+/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽报读错（R1 定案补的盘宽与前置覆盖写两维，
+/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
+/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
+#[test]
+fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
+) {
     assert_eq!(
-        remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
-            &history,
-            faulted_reads,
-            BTreeMap::new(),
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            3,
+            RootRingSlotReadFault::ReadFails
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "单元区 384 槽、实例 1 覆盖写 3 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回全 0（R1 定案补的盘宽与前置覆盖写两维，
+/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
+/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
+#[test]
+fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            3,
+            RootRingSlotReadFault::ReadsBackAllZero
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "单元区 384 槽、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回乱字节（R1 定案补的盘宽与前置覆盖写两维，
+/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
+/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
+#[test]
+fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            3,
+            RootRingSlotReadFault::ReadsBackGarbage
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "单元区 384 槽、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 的反方向（单元区 384 槽、实例 1 覆盖写 3 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出（R1 定案补的两维）。
+/// 预期同 `the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力同上一条（同一行变异的另一个名字：「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）。
+#[test]
+fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            3,
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "单元区 384 槽、实例 1 覆盖写 3 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽报读错（R1 定案补的盘宽与前置覆盖写两维，
+/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
+/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
+#[test]
+fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::FourGibibytes,
+            1,
+            RootRingSlotReadFault::ReadFails
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "4 GiB 盘、实例 1 覆盖写 1 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回全 0（R1 定案补的盘宽与前置覆盖写两维，
+/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
+/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
+#[test]
+fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::FourGibibytes,
+            1,
+            RootRingSlotReadFault::ReadsBackAllZero
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "4 GiB 盘、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 1 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回乱字节（R1 定案补的盘宽与前置覆盖写两维，
+/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
+/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
+#[test]
+fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::FourGibibytes,
+            1,
+            RootRingSlotReadFault::ReadsBackGarbage
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "4 GiB 盘、实例 1 覆盖写 1 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 的反方向（4 GiB 盘、实例 1 覆盖写 1 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出（R1 定案补的两维）。
+/// 预期同 `the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力同上一条（同一行变异的另一个名字：「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）。
+#[test]
+fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(
+            HistoryDeviceWidth::FourGibibytes,
+            1,
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "4 GiB 盘、实例 1 覆盖写 1 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽报读错（R1 定案补的盘宽与前置覆盖写两维，
+/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
+/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
+#[test]
+fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::FourGibibytes,
+            3,
+            RootRingSlotReadFault::ReadFails
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "4 GiB 盘、实例 1 覆盖写 3 次，重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回全 0（R1 定案补的盘宽与前置覆盖写两维，
+/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
+/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
+#[test]
+fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::FourGibibytes,
+            3,
+            RootRingSlotReadFault::ReadsBackAllZero
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "4 GiB 盘、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 那组历史（4 GiB 盘、实例 1 覆盖写 3 次），第一遍盘 1 每个根环槽报读错，重读那一遍只有 C 的根槽读回乱字节（R1 定案补的盘宽与前置覆盖写两维，
+/// 判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K4 ①）。
+/// 预期同 `the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）时 C 掉出影子账，隔离那一项起红。
+#[test]
+fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            HistoryDeviceWidth::FourGibibytes,
+            3,
+            RootRingSlotReadFault::ReadsBackGarbage
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "4 GiB 盘、实例 1 覆盖写 3 次，重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 的反方向（4 GiB 盘、实例 1 覆盖写 3 次）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出（R1 定案补的两维）。
+/// 预期同 `the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring`。
+/// 判别力同上一条（同一行变异的另一个名字：「第10i批 R1 两维：重读之后的读数只剩最新那条根…」）。
+#[test]
+fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_abandoned_roots_slot_and_the_reread_reading_them(
+            HistoryDeviceWidth::FourGibibytes,
+            3,
         ),
         merged_and_keeps_the_abandoned_root_isolated(),
-        "第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
+        "4 GiB 盘、实例 1 覆盖写 3 次，第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
     );
 }
 
@@ -1588,7 +2168,7 @@ fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_kee
 #[test]
 fn a_reread_failing_on_the_whole_ring_after_a_first_pass_that_read_device_zero_mounts_on_the_first_pass_roots(
 ) {
-    let history = pool_with_an_abandoned_root();
+    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 1);
     let ring_slots_of_device_one = root_ring_slots_of_device_one(&history);
     let mut faulted_reads = faulted_reads_of(
         &ring_slots_of_device_one,
diff --git a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
index e074d02..2db80bb 100644
--- a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
+++ b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
@@ -171,14 +171,12 @@ fn verdicts_without_any_violation(
 
 /// 挂载那一刻读坏（撕裂）或读回全零的那个根槽钉在它可能住过的最旧 txg，回收门槛停在那里（用户 2026-10-05 认下的代价：「钉最旧 txg 让 I-3.1
 /// 在合法盘面上红在记账多的一边……用例钉『只许红在记账多的一边』，池级 checker 不放宽」，`records/2026-09-30-里程碑三开工第一轮.md` 第 624 行；
-/// 读回全零照自证不过钉住，同一处第 628 行附近，第 10c 批）：池级 checker 只判 I-3.1 红，记账的已分配比遍历全部有效根多
-/// `expected_bytes_counted_more_than_the_walk`（慢档 4 条调查报告第 3.1 节，判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第三节最后一行：
-/// 多出来的是盘 0 上释放代在 (钉住的 txg, checker 取的环里最旧的自证根] 里的已释放字节）。
-fn verdicts_red_only_by_the_allocated_statistic_counting_more_than_the_walk(
+/// 读回全零照自证不过钉住，同一处第 628 行附近，第 10c 批）：池级 checker 只判 I-3.1 红，而且红在记账多的一边（记账的已分配比遍历全部有效根多）。
+/// 交回多出来的字节数；只核方向，不核大小（用户认的口径是方向加时长，判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第三节第一行）。
+fn bytes_the_allocated_statistic_counts_more_than_the_walk_when_only_it_is_red(
     image: &MemoryPool,
     step: &str,
-    expected_bytes_counted_more_than_the_walk: u64,
-) {
+) -> u64 {
     let violations: Vec<(&'static str, String)> = check_pool_image(image)
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
@@ -204,16 +202,27 @@ fn verdicts_red_only_by_the_allocated_statistic_counting_more_than_the_walk(
         "{step}：I-3.1 红在记账多的一边：{}",
         violations[0].1
     );
+    allocated - walked
+}
+
+/// 同 [`bytes_the_allocated_statistic_counts_more_than_the_walk_when_only_it_is_red`]，再钉多出来的字节数
+/// `expected_bytes_counted_more_than_the_walk`（挂载那一刻：慢档 4 条调查报告第 3.1 节，判决 `research/prompts/m3-batch10-code-r2-main-verification.md`
+/// 第三节最后一行：多出来的是盘 0 上释放代在 (钉住的 txg, checker 取的环里最旧的自证根] 里的已释放字节）。
+fn verdicts_red_only_by_the_allocated_statistic_counting_more_than_the_walk(
+    image: &MemoryPool,
+    step: &str,
+    expected_bytes_counted_more_than_the_walk: u64,
+) {
     assert_eq!(
-        allocated - walked,
+        bytes_the_allocated_statistic_counts_more_than_the_walk_when_only_it_is_red(image, step),
         expected_bytes_counted_more_than_the_walk,
-        "{step}：记账比遍历多的字节数：{}",
-        violations[0].1
+        "{step}：记账比遍历多的字节数"
     );
 }
 
 /// 恢复之后重开的那个进程接着覆盖写（实例 3），一直写到 txg `txg_writing_the_pinned_slot`（它的根槽就是挂载时钉住的那一槽，28 + 一圈）：
-/// 写到它前一次为止 I-3.1 仍只红在记账多的一边，写掉那一槽那一次之后池级 checker 一条违例都没有（钉住只到这个进程把那个槽写掉为止，
+/// 写到它前一次为止，每一次覆盖写之后 I-3.1 都只红在记账多的一边（逐步核方向、不钉大小：多记的字节随覆盖写涨，第三轮两腿量过），
+/// 写掉那一槽那一次之后池级 checker 一条违例都没有（钉住只到这个进程把那个槽写掉为止，
 /// `allocator.rs` 的 `RootRingOccupant::AllZeroAtMount` / `BadAtMount`）。
 fn overwrite_until_the_pinned_root_slot_is_written_and_the_allocated_statistic_holds(
     pool: &mut BuiltPool,
@@ -226,7 +235,7 @@ fn overwrite_until_the_pinned_root_slot_is_written_and_the_allocated_statistic_h
         "txg {txg_writing_the_pinned_slot} 的根槽就是挂载时钉住的那一槽"
     );
     let mut overwrite = 0usize;
-    // 迭代上界：从挂载交回的那一版写到钉住那一槽的前一次（不到一圈）；没有提前出口。
+    // 迭代上界：从挂载交回的那一版写到钉住那一槽的前一次（不到一圈）；没有提前出口（方向不对就断言失败）。
     while pool.output.root.checkpoint_txg.0 + 1 < txg_writing_the_pinned_slot {
         overwrite_in_process(
             pool,
@@ -234,17 +243,17 @@ fn overwrite_until_the_pinned_root_slot_is_written_and_the_allocated_statistic_h
             InstanceGeneration(3),
         );
         overwrite += 1;
+        bytes_the_allocated_statistic_counts_more_than_the_walk_when_only_it_is_red(
+            &pool.memory_pool(),
+            &format!(
+                "{step}：第 {overwrite} 次覆盖写之后（txg {}），那一槽还没被写掉",
+                pool.output.root.checkpoint_txg.0
+            ),
+        );
     }
-    let violated_before_the_slot_is_written: Vec<&str> = check_pool_image(&pool.memory_pool())
-        .into_iter()
-        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
-        .map(|(invariant, _)| invariant)
-        .collect();
-    assert_eq!(
-        violated_before_the_slot_is_written,
-        vec!["I-3.1"],
-        "{step}：写到 txg {} 那一槽还没被写掉，I-3.1 仍红",
-        pool.output.root.checkpoint_txg.0
+    assert!(
+        overwrite > 0,
+        "{step}：挂载交回的那一版到钉住那一槽之间至少有一次覆盖写"
     );
     overwrite_in_process(pool, &content_of(1900, 97), InstanceGeneration(3));
     assert_eq!(
````

### mutations-append.tsv

````
第10i批 S1 两遍里只有一遍自证过的根改回记成知道住着根的槽（窄盘起点抬 F 重读它仍坏就拒，挂载在写行、暖机之后报 FloorRaiseFailedAfterTheMountsPublishes）	crates/singlefs-core/src/mount.rs	            let self_verified_on_only_one_of_the_two_passes =\n                ring_slots_self_verified_on_only_one_of_the_two_passes.contains(ring_slot);	            let self_verified_on_only_one_of_the_two_passes = false;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount
第10i批 S1 抬 F 算上限读它读坏一次就拒、不重读（瞬时读坏一次的那一形也在写行、暖机之后报 FloorRaiseFailedAfterTheMountsPublishes）	crates/singlefs-core/src/mount.rs	        if ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread.contains(&ring_slot) {\n            match read_once(ring_slot) {\n                RootRingSlotReading::SelfVerified(root) => roots.push(root),\n                RootRingSlotReading::Bad(_still_bad) => {\n                    ring_slots_bad_on_this_read_taken_as_holding_no_root.insert(ring_slot);\n                }\n            }\n            continue;\n        }	        if ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread.contains(&ring_slot) {\n            return Err(\n                MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {\n                    ring_slot,\n                    first_reading,\n                    reread: first_reading,\n                },\n            );\n        }	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_root_self_verified_only_on_the_rebuild_pass_read_bad_once_more_after_the_reread_is_taken_as_read_by_the_floor_raise
第10i批 S1 抬 F 算上限读它重读仍坏照拒（按知道住着根的槽处置）	crates/singlefs-core/src/mount.rs	        if ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread.contains(&ring_slot) {	        if false && ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread.contains(&ring_slot) {	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount
第10i批 S1 抬 F 重算回收门槛那一遍读它重读仍坏照拒（不从知道住着根的槽里拿掉）	crates/singlefs-core/src/mount.rs	                !ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass\n                    .contains_key(ring_slot)	                true || !ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass\n                    .contains_key(ring_slot)	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount
第10i批 R1 两维：重读之后的读数只剩最新那条根（单元区 384 槽、覆盖写 3 次，重读只坏 C 的根槽报读错：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
第10i批 R1 两维：重读之后的读数只剩最新那条根（单元区 384 槽、覆盖写 3 次，重读只坏 C 的根槽读回全 0：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
第10i批 R1 两维：重读之后的读数只剩最新那条根（单元区 384 槽、覆盖写 3 次，重读只坏 C 的根槽读回乱字节：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
第10i批 R1 两维：重读之后的读数只剩最新那条根（单元区 384 槽、覆盖写 3 次，反方向：第一遍 C 的根槽也读坏、重读读得出：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 1 次，重读只坏 C 的根槽报读错：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 1 次，重读只坏 C 的根槽读回全 0：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 1 次，重读只坏 C 的根槽读回乱字节：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 1 次，反方向：第一遍 C 的根槽也读坏、重读读得出：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 3 次，重读只坏 C 的根槽报读错：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 3 次，重读只坏 C 的根槽读回全 0：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 3 次，重读只坏 C 的根槽读回乱字节：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 3 次，反方向：第一遍 C 的根槽也读坏、重读读得出：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
第10i批 rr2 用例丢盘整读补全零与乱字节：重读之后的读数只剩最新那条根（264 槽起点、盘 1 读回全 0：当场放开越过环里更旧的有效根）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	on_the_264_slot_pool_with_a_live_file_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before
第10i批 rr2 用例丢盘整读补全零与乱字节：重读之后的读数只剩最新那条根（264 槽起点、盘 0 读回乱字节：当场放开越过环里更旧的有效根）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	on_the_264_slot_pool_with_a_live_file_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before
第10i批 慢档 plain 逐步核方向：进程内回收门槛在 txg 40 那一次越过环里全部有效根（中间一步记账少于遍历，I-3.1 红在少的一边）	crates/singlefs-core/src/allocator.rs	        let reclaim_floor = crate::mount::reclaim_floor(\n            occupancy.effective_floor,\n            occupancy.oldest_valid_root_in_the_ring(),\n        );	        let reclaim_floor = if checkpoint_txg.0 == 40 {\n            checkpoint_txg\n        } else {\n            crate::mount::reclaim_floor(\n                occupancy.effective_floor,\n                occupancy.oldest_valid_root_in_the_ring(),\n            )\n        };	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored plain_overwrite_whose_root_slot_write_zeroes the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root	plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced
第10i批 慢档 torn 逐步核方向：进程内回收门槛在 txg 40 那一次越过环里全部有效根（中间一步记账少于遍历，I-3.1 红在少的一边）	crates/singlefs-core/src/allocator.rs	        let reclaim_floor = crate::mount::reclaim_floor(\n            occupancy.effective_floor,\n            occupancy.oldest_valid_root_in_the_ring(),\n        );	        let reclaim_floor = if checkpoint_txg.0 == 40 {\n            checkpoint_txg\n        } else {\n            crate::mount::reclaim_floor(\n                occupancy.effective_floor,\n                occupancy.oldest_valid_root_in_the_ring(),\n            )\n        };	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored plain_overwrite_whose_root_slot_write_zeroes the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root	the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash
第10i批 慢档 zeroed 逐步核方向：进程内回收门槛在 txg 40 那一次越过环里全部有效根（中间一步记账少于遍历，I-3.1 红在少的一边）	crates/singlefs-core/src/allocator.rs	        let reclaim_floor = crate::mount::reclaim_floor(\n            occupancy.effective_floor,\n            occupancy.oldest_valid_root_in_the_ring(),\n        );	        let reclaim_floor = if checkpoint_txg.0 == 40 {\n            checkpoint_txg\n        } else {\n            crate::mount::reclaim_floor(\n                occupancy.effective_floor,\n                occupancy.oldest_valid_root_in_the_ring(),\n            )\n        };	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored plain_overwrite_whose_root_slot_write_zeroes the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root	the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash
````

### mutations-replacements.tsv

````
第9批 RS1：重读读出有效根的那一槽不算这个进程知道住着根的槽（之后那一槽读坏，准入抬 F 按没有根往下走、发了写）	crates/singlefs-core/src/allocator.rs	                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => true,\n                RootRingOccupant::BadAtMount(_) | RootRingOccupant::AllZeroAtMount { .. } => false,	                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => true,\n                RootRingOccupant::BadAtMount(_)\n                | RootRingOccupant::AllZeroAtMount { .. }\n                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => false,	-p singlefs-harness --test root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread -- a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write	a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write
RGall 进程内回收门槛改回一圈窗口（锚取这个进程记到的最新根：上一圈留在洞里的旧根被滤掉，它引用的单元当场回收）	crates/singlefs-core/src/allocator.rs	                    root: RootReadOnOneRebuildPass::AbandonedRoot { .. },\n                    ..\n                } => None,\n            })\n            .min()	                    root: RootReadOnOneRebuildPass::AbandonedRoot { .. },\n                    ..\n                } => None,\n            })\n            .filter(|checkpoint_txg| {\n                checkpoint_txg.0 + 3 * self.slots_per_region.count() > self.latest_recorded_txg.0\n            })\n            .min()	-p singlefs-core --lib -- root_ring_occupancy_keeps_the_reclaim_floor_at_the_oldest_valid_root_however_far_below_the_newest_it_is	root_ring_occupancy_keeps_the_reclaim_floor_at_the_oldest_valid_root_however_far_below_the_newest_it_is
RGall 进程内回收门槛改回一圈窗口（K1 连崩 11 次之后挂载：实例 1 留在洞里的根引用的单元当场回收）	crates/singlefs-core/src/allocator.rs	                    root: RootReadOnOneRebuildPass::AbandonedRoot { .. },\n                    ..\n                } => None,\n            })\n            .min()	                    root: RootReadOnOneRebuildPass::AbandonedRoot { .. },\n                    ..\n                } => None,\n            })\n            .filter(|checkpoint_txg| {\n                checkpoint_txg.0 + 3 * self.slots_per_region.count() > self.latest_recorded_txg.0\n            })\n            .min()	-p singlefs-harness --test reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root -- eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated	eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated
RGall 进程内回收门槛改回一圈窗口（洞里 txg 7 的根引用的单元在 txg 35 起被回收）	crates/singlefs-core/src/allocator.rs	                    root: RootReadOnOneRebuildPass::AbandonedRoot { .. },\n                    ..\n                } => None,\n            })\n            .min()	                    root: RootReadOnOneRebuildPass::AbandonedRoot { .. },\n                    ..\n                } => None,\n            })\n            .filter(|checkpoint_txg| {\n                checkpoint_txg.0 + 3 * self.slots_per_region.count() > self.latest_recorded_txg.0\n            })\n            .min()	-p singlefs-harness --test one_revolution_window_after_a_hole_in_the_root_ring -- reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated	reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated
第10e批 重建分配器读根环那一遍一条有效根都没读出时不重读、当场拒（删掉重读那一遍）	crates/singlefs-core/src/mount.rs	            None => {\n                before_the_one_reread.before_rereading();\n                let readings_of_the_reread = read_every_root_ring_slot_once();\n                ring_slots_self_verified_on_only_one_of_the_two_passes =\n                    ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes(\n                        &first_readings,\n                        &readings_of_the_reread,\n                    );\n                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );\n                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_two_passes_merged)\n                else {\n                    return Err(\n                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                    );\n                };\n                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )\n            }	            None => {\n                return Err(\n                    MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                )\n            }	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over
第10e批 重建分配器读根环那一遍一条有效根都没读出时照第 17 批那样不重读、给空表往下走（攻方 1.3 甲二：暖机写进盘 0 上最新的根还引用的槽）	crates/singlefs-core/src/mount.rs	            None => {\n                before_the_one_reread.before_rereading();\n                let readings_of_the_reread = read_every_root_ring_slot_once();\n                ring_slots_self_verified_on_only_one_of_the_two_passes =\n                    ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes(\n                        &first_readings,\n                        &readings_of_the_reread,\n                    );\n                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );\n                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_two_passes_merged)\n                else {\n                    return Err(\n                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                    );\n                };\n                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )\n            }	            None => (\n                Vec::new(),\n                CheckpointTxg(0),\n                RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,\n            ),	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	only_the_allocator_rebuild_pass_failing_and_a_crash_before_the_warm_up_root_lands_on_device_zero_keep_what_device_zero_reads
A3b 善后一：可写挂载重建分配器时生效 F 又先自己读一遍最新根的实例表（那一片读次数回到 (5, 3)）	crates/singlefs-core/src/mount.rs	    let current_records = allocator.records().to_vec();\n    let (newest_table, instance_table_of_the_newest_root) =	    let current_records = allocator.records().to_vec();\n    let _ = crate::recovery::effective_rollback_floor(\n        devices,\n        &system_configuration.immutable.region_devices,\n        &system_configuration.immutable.sizes,\n        &system_configuration.immutable.filesystem_identifier,\n    );\n    let (newest_table, instance_table_of_the_newest_root) =	-p singlefs-harness --test a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable	a_newest_instance_table_unreadable_when_the_shadow_ledger_reads_it_is_reread_once_then_refuses_writable_instead_of_turning_the_isolation_off
````
