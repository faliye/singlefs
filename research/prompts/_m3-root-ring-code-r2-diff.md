# 第 8 批代码轮第二轮：被判的改动全文（2026-10-03）

两份补丁依次打进主工作区，之后主 agent 派的改钉实现员与主 agent 自己各改了一处；冻结副本 `/tmp/claude-1000/m3-root-ring-code-r2/tree/` 是这些都做完之后的样子。

## 8-1 core 簇（`/tmp/claude-1000/m3-r1/batch8-core/patch/crates.patch`，报告同目录 `report.md`；变异 `mutations-append.tsv`、`mutations-replacements.tsv`、`mutations-delete.txt`）

````diff
diff --git a/crates/singlefs-core/src/allocation_record_tree.rs b/crates/singlefs-core/src/allocation_record_tree.rs
index a4f7289..b972fa2 100644
--- a/crates/singlefs-core/src/allocation_record_tree.rs
+++ b/crates/singlefs-core/src/allocation_record_tree.rs
@@ -27,7 +27,9 @@ use crate::address::{
 };
 use crate::allocator::{absolute_slot_count_of_device, AllocationRecord, PoolAllocator};
 use crate::bytes::{ByteReader, ByteWriter};
-use crate::pointer::{BirthSequence, NodePointer};
+use crate::pointer::{
+    BirthSequence, NodePointer, PointerHeadFieldGroup, PointerHeadFieldsJudgement,
+};
 use crate::recovery::{PoolReader, RecoveryFailure};
 use crate::root_record::RootRecord;
 use crate::unit::{build_index_node, parse_index_node, IndexNodeHeader};
@@ -771,10 +773,19 @@ impl AllocationRecordTreeReading<'_> {
             }
             previous_child = Some(child);
             let mut reader = ByteReader::at(entry, key_bytes_width());
-            let child_pointer = NodePointer::read_judging_the_extent_offset_from(&mut reader)
-                .map_err(|_extent_offset| RecoveryFailure::UnitMalformed {
-                    what:
-                        ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
+            let child_pointer =
+                NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+                    &mut reader,
+                )
+                .map_err(|field| RecoveryFailure::UnitMalformed {
+                    what: match field.field_group() {
+                        PointerHeadFieldGroup::EncryptionOrCompression => {
+                            ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION
+                        }
+                        PointerHeadFieldGroup::ExtentOffset => {
+                            ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO
+                        }
+                    },
                 })?;
             self.read_node_and_its_subtree(
                 child,
@@ -838,10 +849,16 @@ pub fn read_allocation_record_tree(
 /// （C581（extent 偏移非 0 没有判据）：判该指针所在的结构损坏）。
 pub const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO: &str =
     "分配记录树内部条目的子指针头部 extent 偏移不是 0";
+/// 分配记录树内部条目里的子指针头部加密、压缩那几段（MAC / nonce、算法类型、压缩算法码与压后长度）不是第一版的取值时报
+/// [`RecoveryFailure::UnitMalformed`]，`what` 取这一句（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14：判该指针所在的结构损坏）。
+pub const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION:
+    &str = "分配记录树内部条目的子指针头部的加密、压缩那几段不是第一版的取值";
 
 /// 从根指针往下，读得出多少就认多少：交回走得到的每个节点的指针（根在内；读不出、解不开、位置对不上的节点它自己的指针照样交回，
-/// 只是不再往它下面走；子指针头部 extent 偏移非 0 的那一条条目同「解不开」，它指的孩子不认）。影子账认一条被抛弃根引用着哪些单元时用它（`mount`）：这棵树的节点都是这条根引用的单元，
-/// 少认一个，被抛弃根的那一个就不隔离、回退之后可能被发出去——同实例表那条链「认得出的都算上」。
+/// 只是不再往它下面走）。影子账认一条被抛弃根引用着哪些单元时用它（`mount`）：这棵树的节点都是这条根引用的单元，
+/// 少认一个，被抛弃根的那一个就不隔离、回退之后可能被发出去——同实例表那条链「认得出的都算上」。所以子指针头部第一版之外的取值
+/// （加密、压缩那几段与 extent 偏移）这里不判，指针里的位置条目照样认它指的孩子（[`PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger`]；
+/// 走读那一路 `read_allocation_record_tree` 照判）。
 #[must_use]
 pub fn node_pointers_as_far_as_readable(
     root_pointer: &NodePointer,
@@ -874,8 +891,9 @@ pub fn node_pointers_as_far_as_readable(
             if let Some(child) =
                 geometry.child_named_by_entry_key(node, &entry[..key_bytes_width()])
             {
-                if let Ok(child_pointer) = NodePointer::read_judging_the_extent_offset_from(
+                if let Ok(child_pointer) = NodePointer::read_with_head_fields_judgement_from(
                     &mut ByteReader::at(entry, key_bytes_width()),
+                    PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger,
                 ) {
                     pending.push((child, child_pointer));
                 }
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 0b98097..45a076f 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -19,6 +19,7 @@ use std::collections::{BTreeMap, BTreeSet};
 use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
 use crate::admission::SpaceAdmission;
 use crate::bytes::ByteWriter;
+use crate::instance_table::InstanceTableRecords;
 use crate::journal::slot_after_the_journal_ring;
 use crate::root_ring::{
     target_for_publish, txg_is_inside_the_one_revolution_window, RootRingSlot,
@@ -954,6 +955,11 @@ pub enum RootRingOccupant {
     UnreadableAtMountInsideTheWindow {
         highest_txg_it_can_hold: CheckpointTxg,
     },
+    /// 挂载那一刻读不出的槽，这个进程在它成了窗口里最小的那个时重读、读出了一条自证过而按实例表仍然有效的根
+    /// （D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表二之后那一段）：
+    /// 照读出来的 txg 进窗口。与 [`RootRingOccupant::UnreadableAtMountInsideTheWindow`] 一样不算「这个进程知道住着根的槽」：D16 已定项 1
+    /// 「根槽这一次读坏」那一行列的是挂载那一刻读得出的与这个进程写过的两种，进程内重读读出来的不在其内。
+    ValidRootReadWhenTheSlotUnreadableAtMountWasReread { checkpoint_txg: CheckpointTxg },
     /// 按实例表判仍然有效的根，txg 在不在 F 之下都算（可再分配谓词取 max(F_生效, 环里最旧有效根)，F 那一半另记）。
     ValidRoot { checkpoint_txg: CheckpointTxg },
     /// 按实例表判被抛弃的根（D23（journal 的角色与格式） 已定项 14），带着它引用的每个落点——挂载那一刻仍被候选根或现行账引用、
@@ -977,6 +983,8 @@ pub struct RootRingOccupancy {
     /// 谓词里 F 那一半再抬也多收不回一个槽。
     effective_floor: CheckpointTxg,
     latest_recorded_txg: CheckpointTxg,
+    /// 见 [`RootRingOccupancy::instance_table_judging_the_rereads`]。
+    instance_table_judging_the_rereads: InstanceTableRecords,
 }
 
 impl RootRingOccupancy {
@@ -995,6 +1003,7 @@ impl RootRingOccupancy {
             occupants: occupants.into_iter().collect(),
             effective_floor,
             latest_recorded_txg: latest_txg_before_the_next_publish,
+            instance_table_judging_the_rereads: InstanceTableRecords { rows: Vec::new() },
         }
     }
 
@@ -1014,7 +1023,10 @@ impl RootRingOccupancy {
             .iter()
             .filter(|(_, occupant)| match occupant {
                 RootRingOccupant::ValidRoot { .. } | RootRingOccupant::AbandonedRoot { .. } => true,
-                RootRingOccupant::UnreadableAtMountInsideTheWindow { .. } => false,
+                RootRingOccupant::UnreadableAtMountInsideTheWindow { .. }
+                | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread { .. } => {
+                    false
+                }
             })
             .map(|(ring_slot, _)| *ring_slot)
             .collect()
@@ -1027,7 +1039,10 @@ impl RootRingOccupancy {
         self.occupants
             .values()
             .filter_map(|occupant| match occupant {
-                RootRingOccupant::ValidRoot { checkpoint_txg, .. } => Some(*checkpoint_txg),
+                RootRingOccupant::ValidRoot { checkpoint_txg, .. }
+                | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread {
+                    checkpoint_txg,
+                } => Some(*checkpoint_txg),
                 RootRingOccupant::UnreadableAtMountInsideTheWindow {
                     highest_txg_it_can_hold,
                 } => Some(*highest_txg_it_can_hold),
@@ -1052,11 +1067,105 @@ impl RootRingOccupancy {
                     referenced_placements,
                 } => referenced_placements.as_slice(),
                 RootRingOccupant::ValidRoot { .. }
-                | RootRingOccupant::UnreadableAtMountInsideTheWindow { .. } => &[],
+                | RootRingOccupant::UnreadableAtMountInsideTheWindow { .. }
+                | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread { .. } => {
+                    &[]
+                }
             })
             .copied()
             .collect()
     }
+
+    /// 挂载那一刻读不出、还记在表里的槽，各按它能住的最大 txg（[`RootRingOccupant::UnreadableAtMountInsideTheWindow`]），按槽的次序。
+    #[must_use]
+    pub fn ring_slots_unreadable_at_mount(&self) -> BTreeMap<RootRingSlot, CheckpointTxg> {
+        self.occupants
+            .iter()
+            .filter_map(|(ring_slot, occupant)| match occupant {
+                RootRingOccupant::UnreadableAtMountInsideTheWindow {
+                    highest_txg_it_can_hold,
+                } => Some((*ring_slot, *highest_txg_it_can_hold)),
+                RootRingOccupant::ValidRoot { .. }
+                | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread { .. }
+                | RootRingOccupant::AbandonedRoot { .. } => None,
+            })
+            .collect()
+    }
+
+    /// 进程内重读那几个槽时判「按实例表仍然有效」用的那张表（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」：按实例表判仍然有效）。
+    /// 挂载时装的是最新那条根的那一张（`mount::rebuilt_allocator`），写行那次发布的表一定下来就换成它（`mount::establish_instance`）——
+    /// 池级 checker 按镜像里最新那条根的实例表判，写行之后最新那条根带的就是写行那一张。mkfs 同一个进程里装的是空表（表里没有挂载时读不出的槽，用不到它）。
+    #[must_use]
+    pub fn instance_table_judging_the_rereads(&self) -> &InstanceTableRecords {
+        &self.instance_table_judging_the_rereads
+    }
+}
+
+/// 进程内重读一个挂载那一刻读不出的根环槽读到了什么（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定）。封闭集合，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RereadOfARootRingSlotUnreadableAtMount {
+    /// 读得出、自证过，按实例表仍然有效：照读出来的 txg 算（窗口里的进窗口，窗口外的不算）。
+    ValidRoot { checkpoint_txg: CheckpointTxg },
+    /// 读得出而自证不过，或读出来的根按实例表被抛弃：当没有根（与挂载那一刻读得出而自证不过的槽同一个处置）。
+    NoValidRoot,
+    /// 仍读不出：照旧按它能住的最大 txg 算。
+    StillUnreadable,
+}
+
+/// 一圈窗口里最旧有效根，挂载那一刻读不出的槽按 E2 并进来（D16（发布语义） 已定项 1，用户 2026-10-03 定）：`valid_root_txgs` 是已知
+/// 住着有效根的 txg，`unreadable_at_mount` 是挂载那一刻读不出、各按能住的最大 txg 记的槽；两者里窗口（锚 `window_anchor`）里最小的那个
+/// 若是一个读不出的槽，先重读那一槽（`reread`）：读出有效根就照它的 txg 换进去、当没有根就拿掉，再取最小；仍读不出才按最大 txg 算。
+/// 交回最小的 txg（一条都没有时 `None`）与每个重读过的槽读到了什么（按重读的次序）。
+///
+/// 循环：每轮要么换掉或拿掉一个读不出的槽（`unreadable_at_mount` 少一个），要么在最小的是一个仍读不出的槽、或最小的不是读不出的槽时返回；
+/// 轮数至多 `unreadable_at_mount` 的条数加一。跨轮携带两张表与重读结局。
+pub fn oldest_txg_in_the_window_rereading_slots_unreadable_at_mount(
+    valid_root_txgs: &[CheckpointTxg],
+    mut unreadable_at_mount: BTreeMap<RootRingSlot, CheckpointTxg>,
+    window_anchor: CheckpointTxg,
+    slots_per_region: RootRingSlotsPerRegion,
+    reread: &mut dyn FnMut(RootRingSlot) -> RereadOfARootRingSlotUnreadableAtMount,
+) -> (
+    Option<CheckpointTxg>,
+    Vec<(RootRingSlot, RereadOfARootRingSlotUnreadableAtMount)>,
+) {
+    let is_inside_the_window = |checkpoint_txg: CheckpointTxg| {
+        txg_is_inside_the_one_revolution_window(checkpoint_txg, window_anchor, slots_per_region)
+    };
+    let mut valid_root_txgs: Vec<CheckpointTxg> = valid_root_txgs.to_vec();
+    let mut rereads = Vec::new();
+    loop {
+        let oldest_valid_root = valid_root_txgs
+            .iter()
+            .copied()
+            .filter(|checkpoint_txg| is_inside_the_window(*checkpoint_txg))
+            .min();
+        let oldest_unreadable_slot = unreadable_at_mount
+            .iter()
+            .filter(|(_, highest_txg_it_can_hold)| is_inside_the_window(**highest_txg_it_can_hold))
+            .min_by_key(|(_, highest_txg_it_can_hold)| **highest_txg_it_can_hold)
+            .map(|(ring_slot, highest_txg_it_can_hold)| (*ring_slot, *highest_txg_it_can_hold));
+        let Some((ring_slot, highest_txg_it_can_hold)) = oldest_unreadable_slot else {
+            return (oldest_valid_root, rereads);
+        };
+        if oldest_valid_root.is_some_and(|oldest| oldest < highest_txg_it_can_hold) {
+            return (oldest_valid_root, rereads);
+        }
+        let reread_of_the_slot = reread(ring_slot);
+        rereads.push((ring_slot, reread_of_the_slot));
+        match reread_of_the_slot {
+            RereadOfARootRingSlotUnreadableAtMount::ValidRoot { checkpoint_txg } => {
+                unreadable_at_mount.remove(&ring_slot);
+                valid_root_txgs.push(checkpoint_txg);
+            }
+            RereadOfARootRingSlotUnreadableAtMount::NoValidRoot => {
+                unreadable_at_mount.remove(&ring_slot);
+            }
+            RereadOfARootRingSlotUnreadableAtMount::StillUnreadable => {
+                return (Some(highest_txg_it_can_hold), rereads);
+            }
+        }
+    }
 }
 
 /// 池级分配器：每块盘按同一条规则各自取，各盘取得一致才给号（盘可以不等大，D2（RAID 条带策略） 已定项 2）。
@@ -1666,6 +1775,17 @@ impl PoolAllocator {
         self.root_ring = Some(occupancy);
     }
 
+    /// 换上进程内重读挂载时读不出的槽时判「按实例表仍然有效」的那张表（[`RootRingOccupancy::instance_table_judging_the_rereads`]）。
+    /// 没装根环时什么都不做。
+    pub fn note_the_instance_table_judging_the_rereads_of_root_ring_slots(
+        &mut self,
+        table: InstanceTableRecords,
+    ) {
+        if let Some(occupancy) = self.root_ring.as_mut() {
+            occupancy.instance_table_judging_the_rereads = table;
+        }
+    }
+
     /// 这会儿装着的根环；`None` 见 `root_ring` 字段的注释。
     #[must_use]
     pub fn root_ring_occupancy(&self) -> Option<&RootRingOccupancy> {
@@ -1720,7 +1840,8 @@ impl PoolAllocator {
             ),
             Some(
                 RootRingOccupant::ValidRoot { .. }
-                | RootRingOccupant::UnreadableAtMountInsideTheWindow { .. },
+                | RootRingOccupant::UnreadableAtMountInsideTheWindow { .. }
+                | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread { .. },
             )
             | None => {}
         }
@@ -1732,6 +1853,66 @@ impl PoolAllocator {
         );
     }
 
+    /// 同 [`Self::record_root_written_by_this_process`]，记之前先照 E2 重读挂载那一刻读不出、记了这条根之后会是窗口里最小的那个槽
+    /// （D16（发布语义） 已定项 1，用户 2026-10-03 定：读得出且自证过就照读出来的根算——窗口里、按实例表有效的照它的 txg，别的当没有；
+    /// 仍读不出才按它能住的最大 txg 算）。`reread` 读盘、判自证与实例表（`mount::reread_of_a_root_ring_slot_unreadable_at_mount`）；
+    /// 这里只定重读哪一槽、按读到的改表。读不出的槽一个都不在窗口里最小的位置上时一个都不读。没装根环时只记根（什么都不做）。
+    pub fn record_root_written_by_this_process_rereading_slots_unreadable_at_mount(
+        &mut self,
+        checkpoint_txg: CheckpointTxg,
+        reread: &mut dyn FnMut(RootRingSlot) -> RereadOfARootRingSlotUnreadableAtMount,
+    ) {
+        if let Some(occupancy) = self.root_ring.as_mut() {
+            let ring_slot_of_this_root =
+                target_for_publish(checkpoint_txg, occupancy.slots_per_region);
+            // 这条根要盖掉的那一槽不算：记了之后那里住的是这条根。
+            let valid_root_txgs: Vec<CheckpointTxg> = occupancy
+                .occupants
+                .iter()
+                .filter(|(ring_slot, _)| **ring_slot != ring_slot_of_this_root)
+                .filter_map(|(_, occupant)| match occupant {
+                    RootRingOccupant::ValidRoot {
+                        checkpoint_txg: txg_of_the_occupant,
+                    }
+                    | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread {
+                        checkpoint_txg: txg_of_the_occupant,
+                    } => Some(*txg_of_the_occupant),
+                    RootRingOccupant::UnreadableAtMountInsideTheWindow { .. }
+                    | RootRingOccupant::AbandonedRoot { .. } => None,
+                })
+                .chain(std::iter::once(checkpoint_txg))
+                .collect();
+            let mut unreadable_at_mount = occupancy.ring_slots_unreadable_at_mount();
+            unreadable_at_mount.remove(&ring_slot_of_this_root);
+            let (_oldest, rereads) = oldest_txg_in_the_window_rereading_slots_unreadable_at_mount(
+                &valid_root_txgs,
+                unreadable_at_mount,
+                checkpoint_txg,
+                occupancy.slots_per_region,
+                reread,
+            );
+            for (ring_slot, reread_of_the_slot) in rereads {
+                match reread_of_the_slot {
+                    RereadOfARootRingSlotUnreadableAtMount::ValidRoot {
+                        checkpoint_txg: txg_read_on_the_reread,
+                    } => {
+                        occupancy.occupants.insert(
+                            ring_slot,
+                            RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread {
+                                checkpoint_txg: txg_read_on_the_reread,
+                            },
+                        );
+                    }
+                    RereadOfARootRingSlotUnreadableAtMount::NoValidRoot => {
+                        occupancy.occupants.remove(&ring_slot);
+                    }
+                    RereadOfARootRingSlotUnreadableAtMount::StillUnreadable => {}
+                }
+            }
+        }
+        self.record_root_written_by_this_process(checkpoint_txg);
+    }
+
     /// 第一个文件版本要建在上面的那一版（txg `version_to_build_on`，树表 0 条）之前，这个实例接连发过的零单元发布把根写到了
     /// 那个 txg：调用方直接调 `transaction::publish_without_units` 时那几条根没经分配器。它们是同一个实例在这个进程里接着写的
     /// （零单元发布一条接一条、txg 逐个加一，每一条交回 Ok 之后调用方才接着发），所以 [上一条记到的 txg + 1, 那一版的 txg] 里
@@ -2534,6 +2715,94 @@ mod tests {
         }
     }
 
+    /// E2 那一判的取法（D16（发布语义） 已定项 1，用户 2026-10-03 定）：12 槽的环（S = 4），锚 txg 23，窗口 txg > 11；txg 13–23 住着有效根，
+    /// 区域 0 槽 0 挂载那一刻读不出、按它能住的最大 txg 12 记着，是窗口里最小的那个 ⇒ 重读那一槽一次。
+    fn oldest_in_the_window_when_the_slot_unreadable_at_mount_rereads_as(
+        reread_of_the_slot: RereadOfARootRingSlotUnreadableAtMount,
+    ) -> (
+        Option<CheckpointTxg>,
+        Vec<(RootRingSlot, RereadOfARootRingSlotUnreadableAtMount)>,
+    ) {
+        let slots_per_region =
+            RootRingSlotsPerRegion::from_system_configuration_field(4).expect("4 在区间里");
+        let hidden_slot = RootRingSlot { region: 0, slot: 0 };
+        assert_eq!(
+            target_for_publish(CheckpointTxg(12), slots_per_region),
+            hidden_slot
+        );
+        let valid_root_txgs: Vec<CheckpointTxg> = (13..=23).map(CheckpointTxg).collect();
+        oldest_txg_in_the_window_rereading_slots_unreadable_at_mount(
+            &valid_root_txgs,
+            BTreeMap::from([(hidden_slot, CheckpointTxg(12))]),
+            CheckpointTxg(23),
+            slots_per_region,
+            &mut |ring_slot| {
+                assert_eq!(ring_slot, hidden_slot, "只重读窗口里最小的那一槽");
+                reread_of_the_slot
+            },
+        )
+    }
+
+    /// 重读读出的是上一圈的 txg 0（窗口外）：照读出来的根算、那一槽不在窗口里，最旧有效根是 13。
+    #[test]
+    fn the_slot_unreadable_at_mount_rereading_as_a_root_outside_the_window_leaves_the_window() {
+        assert_eq!(
+            oldest_in_the_window_when_the_slot_unreadable_at_mount_rereads_as(
+                RereadOfARootRingSlotUnreadableAtMount::ValidRoot {
+                    checkpoint_txg: CheckpointTxg(0),
+                }
+            ),
+            (
+                Some(CheckpointTxg(13)),
+                vec![(
+                    RootRingSlot { region: 0, slot: 0 },
+                    RereadOfARootRingSlotUnreadableAtMount::ValidRoot {
+                        checkpoint_txg: CheckpointTxg(0),
+                    }
+                )]
+            )
+        );
+    }
+
+    /// 重读仍读不出：照旧按它能住的最大 txg 12 算。
+    #[test]
+    fn the_slot_unreadable_at_mount_still_unreadable_on_the_reread_counts_by_its_highest_txg() {
+        assert_eq!(
+            oldest_in_the_window_when_the_slot_unreadable_at_mount_rereads_as(
+                RereadOfARootRingSlotUnreadableAtMount::StillUnreadable
+            ),
+            (
+                Some(CheckpointTxg(12)),
+                vec![(
+                    RootRingSlot { region: 0, slot: 0 },
+                    RereadOfARootRingSlotUnreadableAtMount::StillUnreadable
+                )]
+            )
+        );
+    }
+
+    /// 窗口里还有比那一槽能住的最大 txg 更旧的有效根：不重读（重读的口子被调到就断言失败）。
+    #[test]
+    fn the_slot_unreadable_at_mount_is_not_reread_while_an_older_valid_root_is_in_the_window() {
+        let slots_per_region =
+            RootRingSlotsPerRegion::from_system_configuration_field(4).expect("4 在区间里");
+        // txg 12 与那一槽同槽，不在有效根里。
+        let valid_root_txgs: Vec<CheckpointTxg> = std::iter::once(11)
+            .chain(13..=22)
+            .map(CheckpointTxg)
+            .collect();
+        assert_eq!(
+            oldest_txg_in_the_window_rereading_slots_unreadable_at_mount(
+                &valid_root_txgs,
+                BTreeMap::from([(RootRingSlot { region: 0, slot: 0 }, CheckpointTxg(12))]),
+                CheckpointTxg(22),
+                slots_per_region,
+                &mut |ring_slot| panic!("窗口里 txg 11 比那一槽的 12 旧，不该重读 {ring_slot:?}"),
+            ),
+            (Some(CheckpointTxg(11)), Vec::new())
+        );
+    }
+
     /// 记到的根 txg 跳了一格（txg 24 那条根没经分配器写出去）：这张表说的环已经不是盘上的环，断言失败，不按猜的环接着回收。
     #[test]
     #[should_panic(expected = "装了根环表的进程写的根 txg 逐个加一")]
diff --git a/crates/singlefs-core/src/extent_tree.rs b/crates/singlefs-core/src/extent_tree.rs
index f945222..9157645 100644
--- a/crates/singlefs-core/src/extent_tree.rs
+++ b/crates/singlefs-core/src/extent_tree.rs
@@ -28,7 +28,10 @@ use singlefs_format::{
 
 use crate::address::{CheckpointTxg, InstanceGeneration, SlotNumber, TreeIdentifier};
 use crate::bytes::{ByteReader, ByteWriter};
-use crate::pointer::{BirthSequence, DataPointer, NodePointer, PointerHeadExtentOffsetNotZero};
+use crate::pointer::{
+    BirthSequence, DataPointer, NodePointer, PointerHeadFieldGroup,
+    PointerHeadFieldOutsideTheFirstVersion,
+};
 use crate::records::{build_extent_record, parse_extent_record, PointerCarryingEntryRefusal};
 use crate::recovery::RecoveryFailure;
 use crate::root_record::RootRecord;
@@ -261,9 +264,9 @@ pub enum ExtentUpperLeafEntryMalformed {
     UnrecognizedTag(u8),
     /// key 的 locality 段或 offset 段不是 0，或载荷里该是零的字节不是零。
     NonZeroBytesWhereTheFieldTableSaysZero,
-    /// 标签 1 / 2 的载荷里那条指针（下段根指针、内联的数据指针）头部 extent 偏移非 0：判这条条目损坏
-    /// （D19（块指针的结构与宽度预算） 已定项 3、C581（extent 偏移非 0 没有判据））。
-    PointerExtentOffsetNotZero(PointerHeadExtentOffsetNotZero),
+    /// 标签 1 / 2 的载荷里那条指针（下段根指针、内联的数据指针）头部有一段不是第一版的取值（MAC / nonce、算法类型、压缩算法码与
+    /// 压后长度、extent 偏移）：判这条条目损坏（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据））。
+    PointerHeadFieldOutsideTheFirstVersion(PointerHeadFieldOutsideTheFirstVersion),
 }
 
 /// extent 上段叶条目里那条指针（下段根指针或内联的数据指针）头部 extent 偏移非 0 时报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
@@ -275,6 +278,30 @@ pub const EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO: &str
 /// extent 叶记录里的数据指针头部 extent 偏移非 0 时报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
 pub const EXTENT_LEAF_RECORD_DATA_POINTER_EXTENT_OFFSET_NOT_ZERO: &str =
     "extent 叶记录里数据指针的头部 extent 偏移不是 0";
+/// extent 上段叶条目里那条指针头部的加密、压缩那几段（MAC / nonce、算法类型、压缩算法码与压后长度）不是第一版的取值时报
+/// [`RecoveryFailure::UnitMalformed`]，`what` 取这一句（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14）。
+pub const EXTENT_UPPER_LEAF_ENTRY_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION:
+    &str = "extent 上段叶条目里那条指针头部的加密、压缩那几段不是第一版的取值";
+/// extent 树内部条目里的子指针头部加密、压缩那几段不是第一版的取值时报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
+pub const EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION:
+    &str = "extent 树内部条目的子指针头部的加密、压缩那几段不是第一版的取值";
+/// extent 叶记录里的数据指针头部加密、压缩那几段不是第一版的取值时报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
+pub const EXTENT_LEAF_RECORD_DATA_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION:
+    &str = "extent 叶记录里数据指针头部的加密、压缩那几段不是第一版的取值";
+
+/// 一条指针头部第一版之外的取值报哪一句：按落在哪一组字段上，在读者给的两句里挑（三类 extent 树读者共用）。
+fn what_of_a_pointer_head_field_outside_the_first_version(
+    field: PointerHeadFieldOutsideTheFirstVersion,
+    what_of_the_encryption_or_compression_fields: &'static str,
+    what_of_the_extent_offset: &'static str,
+) -> &'static str {
+    match field.field_group() {
+        PointerHeadFieldGroup::EncryptionOrCompression => {
+            what_of_the_encryption_or_compression_fields
+        }
+        PointerHeadFieldGroup::ExtentOffset => what_of_the_extent_offset,
+    }
+}
 
 impl ExtentUpperLeafEntry {
     #[must_use]
@@ -304,7 +331,7 @@ impl ExtentUpperLeafEntry {
     /// 上段叶条目的读者：这里是盘上字节进字段表的边界。
     ///
     /// # Errors
-    /// 窄于 113、标签不认识、该是零的字节不是零、载荷里那条指针头部 extent 偏移非 0。
+    /// 窄于 113、标签不认识、该是零的字节不是零、载荷里那条指针头部有一段不是第一版的取值。
     pub fn parse(bytes: &[u8]) -> Result<Self, ExtentUpperLeafEntryMalformed> {
         if bytes.len() < usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113") {
             return Err(ExtentUpperLeafEntryMalformed::NarrowerThanItsFieldTable);
@@ -333,15 +360,17 @@ impl ExtentUpperLeafEntry {
                     );
                 }
                 ExtentUpperLeafTarget::LowerSegmentRoot(
-                    NodePointer::read_judging_the_extent_offset_from(&mut ByteReader::at(
-                        payload, 0,
-                    ))
-                    .map_err(ExtentUpperLeafEntryMalformed::PointerExtentOffsetNotZero)?,
+                    NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+                        &mut ByteReader::at(payload, 0),
+                    )
+                    .map_err(ExtentUpperLeafEntryMalformed::PointerHeadFieldOutsideTheFirstVersion)?,
                 )
             }
             EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT => ExtentUpperLeafTarget::InlineDataUnit(
-                DataPointer::read_judging_the_extent_offset_from(&mut ByteReader::at(payload, 0))
-                    .map_err(ExtentUpperLeafEntryMalformed::PointerExtentOffsetNotZero)?,
+                DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+                    &mut ByteReader::at(payload, 0),
+                )
+                .map_err(ExtentUpperLeafEntryMalformed::PointerHeadFieldOutsideTheFirstVersion)?,
             ),
             unrecognized => {
                 return Err(ExtentUpperLeafEntryMalformed::UnrecognizedTag(unrecognized))
@@ -703,13 +732,16 @@ impl ExtentTreeReading<'_> {
             .entries
             .iter()
             .map(|entry| {
-                NodePointer::read_judging_the_extent_offset_from(&mut ByteReader::at(
-                    entry,
-                    key_width(),
-                ))
+                NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+                    &mut ByteReader::at(entry, key_width()),
+                )
                 .map(|child| (entry[..key_width()].to_vec(), child))
-                .map_err(|_extent_offset| RecoveryFailure::UnitMalformed {
-                    what: EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
+                .map_err(|field| RecoveryFailure::UnitMalformed {
+                    what: what_of_a_pointer_head_field_outside_the_first_version(
+                        field,
+                        EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+                        EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
+                    ),
                 })
             })
             .collect()
@@ -802,9 +834,13 @@ fn read_lower_node_and_its_subtree(
                                 .expect("112"),
                         }
                     }
-                    PointerCarryingEntryRefusal::PointerExtentOffsetNotZero(_extent_offset) => {
+                    PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(field) => {
                         RecoveryFailure::UnitMalformed {
-                            what: EXTENT_LEAF_RECORD_DATA_POINTER_EXTENT_OFFSET_NOT_ZERO,
+                            what: what_of_a_pointer_head_field_outside_the_first_version(
+                                field,
+                                EXTENT_LEAF_RECORD_DATA_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+                                EXTENT_LEAF_RECORD_DATA_POINTER_EXTENT_OFFSET_NOT_ZERO,
+                            ),
                         }
                     }
                 })?;
@@ -915,9 +951,13 @@ fn upper_leaf_entries(
                         detail: "extent 上段叶条目里字段表写零的字节不是零",
                     }
                 }
-                ExtentUpperLeafEntryMalformed::PointerExtentOffsetNotZero(_extent_offset) => {
+                ExtentUpperLeafEntryMalformed::PointerHeadFieldOutsideTheFirstVersion(field) => {
                     RecoveryFailure::UnitMalformed {
-                        what: EXTENT_UPPER_LEAF_ENTRY_POINTER_EXTENT_OFFSET_NOT_ZERO,
+                        what: what_of_a_pointer_head_field_outside_the_first_version(
+                            field,
+                            EXTENT_UPPER_LEAF_ENTRY_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+                            EXTENT_UPPER_LEAF_ENTRY_POINTER_EXTENT_OFFSET_NOT_ZERO,
+                        ),
                     }
                 }
             })?;
diff --git a/crates/singlefs-core/src/instance_table.rs b/crates/singlefs-core/src/instance_table.rs
index 70fe762..9864ef7 100644
--- a/crates/singlefs-core/src/instance_table.rs
+++ b/crates/singlefs-core/src/instance_table.rs
@@ -9,7 +9,7 @@
 use crate::address::{CheckpointTxg, InstanceGeneration, TreeIdentifier};
 use crate::bytes::{ByteReader, ByteWriter};
 use crate::make_filesystem::instance_table_chain_record;
-use crate::pointer::NodePointer;
+use crate::pointer::{NodePointer, PointerHeadFieldsJudgement};
 use crate::transaction::TREE_IDENTIFIER_NONE;
 use crate::unit::{parse_packed_unit, PackedIdentity, PACKED_TYPE_INSTANCE_TABLE};
 use singlefs_format::{INSTANCE_ROW_BYTES, INSTANCE_TABLE_PAGE_RECORDS, NODE_POINTER_BYTES};
@@ -189,10 +189,24 @@ impl InstanceTableChainRecord {
     }
 
     /// 解一条链指针记录。宽不是 88、`kind` 不是 1、「有无下一片」不是 0 或 1、无下一片而位置指针不全零（条款：清零占位）、
-    /// 有下一片而位置指针全零（指不到任何地方）、有下一片而位置指针头部 extent 偏移非 0（C581（extent 偏移非 0 没有判据）：
+    /// 有下一片而位置指针全零（指不到任何地方）、有下一片而位置指针头部有一段不是第一版的取值（MAC / nonce、算法类型、压缩算法码与
+    /// 压后长度、extent 偏移：D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据），
     /// 判该指针所在的结构——这一片——损坏），都是 `None`。
     #[must_use]
     pub fn parse(bytes: &[u8]) -> Option<Self> {
+        Self::parse_with_pointer_head_fields_judgement(
+            bytes,
+            PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion,
+        )
+    }
+
+    /// 同 [`Self::parse`]，链指针头部第一版之外的取值判不判由读者定：沿链读整张表的判，影子账认链上各片的不判
+    /// （[`PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger`]：多隔离、不漏隔离）。
+    #[must_use]
+    pub fn parse_with_pointer_head_fields_judgement(
+        bytes: &[u8],
+        pointer_head_fields: PointerHeadFieldsJudgement,
+    ) -> Option<Self> {
         if bytes.len() != usize::try_from(INSTANCE_ROW_BYTES).expect("88")
             || bytes[0] != INSTANCE_ROW_KIND_CHAIN
         {
@@ -207,10 +221,10 @@ impl InstanceTableChainRecord {
                 if pointer_is_zero {
                     return None;
                 }
-                NodePointer::read_judging_the_extent_offset_from(&mut ByteReader::at(
-                    bytes,
-                    CHAIN_RECORD_POINTER_OFFSET,
-                ))
+                NodePointer::read_with_head_fields_judgement_from(
+                    &mut ByteReader::at(bytes, CHAIN_RECORD_POINTER_OFFSET),
+                    pointer_head_fields,
+                )
                 .ok()
                 .map(Self::NextPage)
             }
@@ -229,14 +243,32 @@ pub struct InstanceTablePage {
 impl InstanceTablePage {
     /// 把一个单元当链上第 `page_index` 片解出来。单元解不开、身份四元组不是 (0, 4, 片序号, 0)（指针指到了别的单元，
     /// 或链的次序乱了、成了环）、最后一条不是合法的链指针记录、它前面有行之外的记录，都是 `None`。
+    /// 链指针头部第一版之外的取值按走读那一种判（[`InstanceTableChainRecord::parse`]）。
     #[must_use]
     pub fn parse(unit_bytes: &[u8], page_index: InstanceTablePageIndex) -> Option<Self> {
+        Self::parse_with_chain_pointer_head_fields_judgement(
+            unit_bytes,
+            page_index,
+            PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion,
+        )
+    }
+
+    /// 同 [`Self::parse`]，链指针头部第一版之外的取值判不判由读者定（[`InstanceTableChainRecord::parse_with_pointer_head_fields_judgement`]）。
+    #[must_use]
+    pub fn parse_with_chain_pointer_head_fields_judgement(
+        unit_bytes: &[u8],
+        page_index: InstanceTablePageIndex,
+        chain_pointer_head_fields: PointerHeadFieldsJudgement,
+    ) -> Option<Self> {
         let unit = parse_packed_unit(unit_bytes).ok()?;
         if unit.identity != page_index.packed_identity() {
             return None;
         }
         let (last, rows_bytes) = unit.records.split_last()?;
-        let chain = InstanceTableChainRecord::parse(last)?;
+        let chain = InstanceTableChainRecord::parse_with_pointer_head_fields_judgement(
+            last,
+            chain_pointer_head_fields,
+        )?;
         let mut rows = Vec::with_capacity(rows_bytes.len());
         for bytes in rows_bytes {
             rows.push(InstanceRow::parse(bytes)?);
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index fd1dc5c..c7d30cd 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -17,8 +17,9 @@ use crate::allocation_record_tree::{
     node_pointers_as_far_as_readable, AllocationRecordTreeGeometry,
 };
 use crate::allocator::{
-    AllocationRecord, DeviceFreeMap, Placement, PlacementOnDevice, PlacementRefusal, PoolAllocator,
-    ReclaimedReuse, RootRingOccupancy, RootRingOccupant, UnitAreaStart,
+    oldest_txg_in_the_window_rereading_slots_unreadable_at_mount, AllocationRecord, DeviceFreeMap,
+    Placement, PlacementOnDevice, PlacementRefusal, PoolAllocator, ReclaimedReuse,
+    RereadOfARootRingSlotUnreadableAtMount, RootRingOccupancy, RootRingOccupant, UnitAreaStart,
 };
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
@@ -751,7 +752,7 @@ pub struct PublishSequenceFailed {
 }
 
 /// 回退目标不在回退候选集里的是哪一条（D23（journal 的角色与格式） 已定项 14「候选集」：根环里按现行那一版的实例表判仍然有效
-/// ∧ txg ≥ F_生效 ∧ 带文件的根）。[`roll_back_by_a_forward_publish`] 按成员的次序判，一条目标同时中几条时只报最先判到的那一条。
+/// ∧ txg ≥ F_生效 ∧ txg 落在一圈窗口里（D16（发布语义） 已定项 1「回退候选集」）∧ 带文件的根）。[`roll_back_by_a_forward_publish`] 按成员的次序判，一条目标同时中几条时只报最先判到的那一条。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum RollbackCandidateExclusion {
     /// 根环里没有那个 (实例, txg) 的可读根槽：候选集是根环里的根。
@@ -761,6 +762,9 @@ pub enum RollbackCandidateExclusion {
     /// 现行那一版的实例表里有那个实例的行 (i, Ti, Wi) 且目标的 txg > Ti：被崩溃恢复抛弃的时间线上的根
     /// （「按现行那一版的实例表判仍然有效」反过来，`recovery::root_is_abandoned_by_the_instance_table`）。
     OnAbandonedTimeline,
+    /// 目标的 txg 落在现行那一版的一圈窗口外（D16（发布语义） 已定项 1「回退候选集」：txg 落在一圈窗口里；窗口 = txg > N − 3S，
+    /// N 取现行那一版的 txg）：环上有洞时那一槽里留着的上一圈的根，窗口已许可它引用的单元被复用（7-2 报告第四节，用户 2026-10-03 定照做）。
+    OutsideTheOneRevolutionWindow,
     /// 目标那一版树表 0 条（还没发布过文件版本）：候选集只收带文件的根。
     VersionWithoutFile,
 }
@@ -1267,6 +1271,64 @@ fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(
     })
 }
 
+/// 进程内重读一个挂载那一刻读不出的根环槽（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定）：只读一次，不重试；读得出、自证过、
+/// 按 `table` 仍然有效 ⇒ 照它的 txg；读得出而自证不过、或按 `table` 被抛弃 ⇒ 当没有根；设备报读错 ⇒ 仍读不出。
+#[must_use]
+pub fn reread_of_a_root_ring_slot_unreadable_at_mount<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    parameters: &MakeFilesystemParameters,
+    table: &InstanceTableRecords,
+    ring_slot: RootRingSlot,
+) -> RereadOfARootRingSlotUnreadableAtMount {
+    match read_root_ring_slot(
+        reader,
+        &parameters.region_devices,
+        &parameters.geometry,
+        &parameters.filesystem_identifier,
+        ring_slot,
+    ) {
+        RootRingSlotReading::SelfVerified(root) if !abandoned_by_table(&root, table) => {
+            RereadOfARootRingSlotUnreadableAtMount::ValidRoot {
+                checkpoint_txg: root.checkpoint_txg,
+            }
+        }
+        RootRingSlotReading::SelfVerified(_)
+        | RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {
+            RereadOfARootRingSlotUnreadableAtMount::NoValidRoot
+        }
+        RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {
+            RereadOfARootRingSlotUnreadableAtMount::StillUnreadable
+        }
+    }
+}
+
+/// 这个进程写了一条根：记进分配器（[`PoolAllocator::record_root_written_by_this_process`]），记之前照 E2 重读挂载那一刻读不出、
+/// 记了这条根之后会是窗口里最小的那个槽（[`PoolAllocator::record_root_written_by_this_process_rereading_slots_unreadable_at_mount`]；
+/// D16（发布语义） 已定项 1，用户 2026-10-03 定）。判「按实例表仍然有效」用分配器根环表上记着的那一张
+/// （`RootRingOccupancy::instance_table_judging_the_rereads`）。产品路径上记根的每一处都走它：带单元的发布在取完落点之后
+/// （`transaction::publish_admitted`、`transaction::prepare_the_row_publish_on_a_version_without_file`），零单元发布在根落盘之后。
+/// 抬 F 那一串在分配器拷贝上的整串预演（`rehearse_the_publishes_raising_the_floor`）不重读：它按最大 txg 算、回收得只会更少，
+/// 预演过得去真发就过得去。
+pub fn record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount<
+    Reader: PoolReader + ?Sized,
+>(
+    reader: &Reader,
+    parameters: &MakeFilesystemParameters,
+    allocator: &mut PoolAllocator,
+    checkpoint_txg: CheckpointTxg,
+) {
+    let table = allocator.root_ring_occupancy().map_or_else(
+        || InstanceTableRecords { rows: Vec::new() },
+        |occupancy| occupancy.instance_table_judging_the_rereads().clone(),
+    );
+    allocator.record_root_written_by_this_process_rereading_slots_unreadable_at_mount(
+        checkpoint_txg,
+        &mut |ring_slot| {
+            reread_of_a_root_ring_slot_unreadable_at_mount(reader, parameters, &table, ring_slot)
+        },
+    );
+}
+
 /// 可再分配谓词的门槛（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」）：根环 R × S 槽（S 读自系统配置），第 0 代根被盖之前
 /// 窗口里最旧有效根恒 0、门槛就是 F_生效；盖掉之后由窗口里最旧的有效根接管。调用方交进来的 `oldest_valid_root` 已经按一圈窗口滤过
 /// （挂载时 `rebuilt_allocator`，进程内 `allocator::RootRingOccupancy`，抬 F 时 `raise_rollback_floor`）。
@@ -1847,6 +1909,8 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 .expect("新实例第一次发布的 txg = 环里最大 txg + 1 ≥ 1"),
         ),
     ));
+    // 进程内重读挂载时读不出的槽（E2）判有效用的表：写行那次发布的表定下来之前先用最新那条根的这一张（`establish_instance` 换掉它）。
+    allocator.note_the_instance_table_judging_the_rereads_of_root_ring_slots(newest_table.clone());
     Ok(RebuiltAllocator {
         allocator,
         effective_floor,
@@ -2044,8 +2108,12 @@ fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
 }
 
 /// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。
-/// 有效 = 自证合法 ∧ 按传进来的实例表不被抛弃 ∧ txg ≥ 今天的 F；非空 = 树表里 inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排、
+/// 有效 = 自证合法 ∧ 按传进来的实例表不被抛弃 ∧ txg ≥ 今天的 F ∧ 落在一圈窗口里；非空 = 树表里 inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排、
 /// 比它小的有效根里最大的那条）的不同（`user_visible_trees_changed`，2026-09-17 用户定案）。
+/// 上限与「非空」只在一圈窗口里的有效根上取（B2，用户 2026-10-03 定，D16 已定项 1 同取这个窗口的第六处；
+/// `research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表二之后那一段）：窗口的锚 N 取环里读得出的、按传进来的实例表有效的
+/// 最新那条根（挂着时就是这个进程记到的最新根），窗口 = txg > N − 3S；窗口外的旧根不当有效根、不读它的树表——窗口已许可它引用的单元被复用，
+/// 读它的树表会读到别的内容或读不出（攻方 K2-b2：没有读故障的合法历史上准入抬 F 被拒）。
 /// 根环怎么读见 [`roots_of_the_ring_for_the_ceiling`]：`ring_slots_known_to_hold_a_root` 取这个进程分配器上那张根环表
 /// （`allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）；分配器上没装根环表的（只有用例自己拼的分配器，
 /// 可写挂载与 `make_filesystem::allocator_after_make_filesystem` 都装）给空集，读坏的槽一律当没有根。
@@ -2070,11 +2138,26 @@ pub fn rollback_floor_ceiling<Device: BlockDevice>(
         system_configuration,
         ring_slots_known_to_hold_a_root,
     )?;
+    let slots_per_region = system_configuration
+        .immutable
+        .sizes
+        .root_ring_slots_per_region;
+    let window_anchor = readable
+        .iter()
+        .filter(|root| !abandoned_by_table(root, table))
+        .map(|root| root.checkpoint_txg)
+        .max();
+    let is_inside_the_window = |checkpoint_txg: CheckpointTxg| {
+        window_anchor.is_some_and(|anchor| {
+            txg_is_inside_the_one_revolution_window(checkpoint_txg, anchor, slots_per_region)
+        })
+    };
     let valid: Vec<RootRecord> = readable
         .iter()
         .copied()
         .filter(|root| root.checkpoint_txg >= current_floor)
         .filter(|root| !abandoned_by_table(root, table))
+        .filter(|root| is_inside_the_window(root.checkpoint_txg))
         .collect();
     let mut newest_per_device: std::collections::BTreeMap<DeviceIdentity, CheckpointTxg> =
         std::collections::BTreeMap::new();
@@ -2113,7 +2196,7 @@ pub fn rollback_floor_ceiling<Device: BlockDevice>(
             },
         ),
     };
-    // 「前一条」只在有效根里找：被抛弃时间线的根与 F 之下的根不算（D16（发布语义） 已定项 1「非空」从盘上怎么认）。
+    // 「前一条」只在有效根里找：被抛弃时间线的根、F 之下的根与一圈窗口外的根不算（D16（发布语义） 已定项 1「非空」从盘上怎么认；B2）。
     let previous_candidates: &[RootRecord] = &valid;
     let mut non_empty: Vec<NonEmptyValidRoot> = Vec::new();
     for root in &valid {
@@ -2607,6 +2690,54 @@ pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
     }
 }
 
+/// 抬 F 那一遍的回收门槛里「一圈窗口里最旧有效根」（D16（发布语义） 已定项 1）：这一遍读出来的根里按 `table` 有效的，并上分配器根环表里
+/// 挂载那一刻读不出、这一遍也没在那一槽读出根的槽（各按它能住的最大 txg；RR4，`research/prompts/m3-root-ring-code-r1-main-verification.md`
+/// 第三节表一，攻方 B1），在锚 `window_anchor` 的窗口里取最小；最小的是一个读不出的槽时照 E2 先重读那一槽
+/// （[`oldest_txg_in_the_window_rereading_slots_unreadable_at_mount`]，与进程内记根同一取法）。只读盘，不改分配器。
+fn oldest_valid_root_in_the_window_for_raising_the_floor<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    parameters_of_the_pool: &MakeFilesystemParameters,
+    allocator: &PoolAllocator,
+    roots_read_from_the_ring: &[RootRecord],
+    table: &InstanceTableRecords,
+    window_anchor: CheckpointTxg,
+) -> Option<CheckpointTxg> {
+    let slots_per_region = parameters_of_the_pool.geometry.root_ring_slots_per_region;
+    let valid_root_txgs: Vec<CheckpointTxg> = roots_read_from_the_ring
+        .iter()
+        .filter(|root| !abandoned_by_table(root, table))
+        .map(|root| root.checkpoint_txg)
+        .collect();
+    let ring_slots_read_from_the_ring: BTreeSet<RootRingSlot> = roots_read_from_the_ring
+        .iter()
+        .map(|root| target_for_publish(root.checkpoint_txg, slots_per_region))
+        .collect();
+    let unreadable_at_mount: BTreeMap<RootRingSlot, CheckpointTxg> = allocator
+        .root_ring_occupancy()
+        .map_or_else(
+            BTreeMap::new,
+            RootRingOccupancy::ring_slots_unreadable_at_mount,
+        )
+        .into_iter()
+        .filter(|(ring_slot, _)| !ring_slots_read_from_the_ring.contains(ring_slot))
+        .collect();
+    oldest_txg_in_the_window_rereading_slots_unreadable_at_mount(
+        &valid_root_txgs,
+        unreadable_at_mount,
+        window_anchor,
+        slots_per_region,
+        &mut |ring_slot| {
+            reread_of_a_root_ring_slot_unreadable_at_mount(
+                reader,
+                parameters_of_the_pool,
+                table,
+                ring_slot,
+            )
+        },
+    )
+    .0
+}
+
 /// 抬 F 那一串（准入与卸载共用，D16（发布语义） 已定项 1「抬 F 那一串」）。两个入口只差 [`RaiseFloorEntry`] 那两样：
 /// 判不判上限、根带不带卸载记号。
 fn raise_the_floor_through<Device: BlockDevice>(
@@ -2729,13 +2860,16 @@ fn raise_the_floor_through<Device: BlockDevice>(
     let is_inside_the_window = |checkpoint_txg: CheckpointTxg| {
         txg_is_inside_the_one_revolution_window(checkpoint_txg, window_anchor, slots_per_region)
     };
-    let oldest_valid_root_in_the_window = roots
-        .iter()
-        .filter(|root| {
-            !abandoned_by_table(root, &table) && is_inside_the_window(root.checkpoint_txg)
-        })
-        .map(|root| root.checkpoint_txg)
-        .min();
+    // 根环表里挂载那一刻读不出、这一遍也没读出根的槽，按它能住的最大 txg 并进窗口里最旧有效根（RR4，攻方 B1；与进程内记根同一取法，
+    // E2 的重读照用：它成了最小的那个时先重读，`oldest_valid_root_in_the_window_for_raising_the_floor`）。
+    let oldest_valid_root_in_the_window = oldest_valid_root_in_the_window_for_raising_the_floor(
+        &**devices,
+        &parameters_of_the_pool,
+        allocator,
+        &roots,
+        &table,
+        window_anchor,
+    );
     // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
     // 不然那个槽回收之后就发得出去（步 4 / 步 5 代码三方第二轮辩方腿：窄读法要按每次挂载与每次抬 F 的候选集现算）。
     let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {
@@ -3118,8 +3252,14 @@ fn publish_empty_after<Device: BlockDevice>(
                     instance,
                 ),
             )?;
-            // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`）。
-            allocator.record_root_written_by_this_process(published.root.checkpoint_txg);
+            // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`），
+            // 记之前照 E2 重读挂载时读不出的那一槽。
+            record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
+                &*pool.devices,
+                pool.parameters,
+                allocator,
+                published.root.checkpoint_txg,
+            );
             Ok(PoolVersion::WithoutFile(published))
         }
     }
@@ -4147,6 +4287,12 @@ fn establish_instance<Device: BlockDevice>(
     allocator.record_instance_rows_after_this_mounts_row_publish(
         u64::try_from(instance_table_rewrite.rows.len()).expect("实例表的行数装得进 u64"),
     );
+    // 写行那次发布之后最新那条根带的就是这张表（池级 checker 按它判有效）：进程内重读挂载时读不出的槽（E2）从写行那次记根起按它判。
+    allocator.note_the_instance_table_judging_the_rereads_of_root_ring_slots(
+        InstanceTableRecords {
+            rows: instance_table_rewrite.rows.clone(),
+        },
+    );
     allocator.set_space_admission(start.space_admission);
     let admission_before_acquisition = match start.space_admission {
         SpaceAdmission::JudgedByTheFormula => match admit_on_every_device(
@@ -4340,8 +4486,14 @@ fn establish_instance<Device: BlockDevice>(
                 },
             )
             .map(|published| {
-                // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`）。
-                allocator.record_root_written_by_this_process(published.root.checkpoint_txg);
+                // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`），
+                // 记之前照 E2 重读挂载时读不出的那一槽。
+                record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
+                    &*pool.devices,
+                    pool.parameters,
+                    &mut allocator,
+                    published.root.checkpoint_txg,
+                );
                 PoolVersion::WithoutFile(published)
             })
         }
@@ -5468,7 +5620,8 @@ fn user_visible_units_through_the_mapping(
 }
 
 /// 回退候选集那一判（D23（journal 的角色与格式） 已定项 14「候选集」）：根环里有它 ∧ txg ≥ F_生效 ∧ 按现行那一版的实例表判仍然有效
-/// ∧ 带文件。按 [`RollbackCandidateExclusion`] 成员的次序判。交回根环里读出来的那条根。
+/// ∧ txg 落在一圈窗口里（锚 `window_anchor` = 现行那一版的 txg，D16（发布语义） 已定项 1「回退候选集」）∧ 带文件。
+/// 按 [`RollbackCandidateExclusion`] 成员的次序判。交回根环里读出来的那条根。
 /// 「根环里有它」与 F_生效 两处读根环都照抬 F 同一个读法（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，实审 A3b 报告 Q7）：
 /// `ring_slots_known_to_hold_a_root`（这个进程分配器上那张根环表，[`ring_slots_known_to_hold_a_root_by`]）里没有的槽读坏当没有根；
 /// 有的重读一次，仍坏就拒——不按没有根往下走（那样目标可以被判成不在环里，F_生效 可以算低、低于真 F 的目标被当成候选）。
@@ -5482,6 +5635,7 @@ fn rollback_candidate<Reader: PoolReader>(
     system_configuration: &crate::system_configuration::SystemConfiguration,
     current_table: &InstanceTableRecords,
     ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
+    window_anchor: CheckpointTxg,
     target: RollbackTarget,
 ) -> Result<RootRecord, RollbackError> {
     let not_a_candidate = |exclusion| RollbackError::TargetNotACandidate { target, exclusion };
@@ -5518,6 +5672,18 @@ fn rollback_candidate<Reader: PoolReader>(
             RollbackCandidateExclusion::OnAbandonedTimeline,
         ));
     }
+    if !txg_is_inside_the_one_revolution_window(
+        target.checkpoint_txg,
+        window_anchor,
+        system_configuration
+            .immutable
+            .sizes
+            .root_ring_slots_per_region,
+    ) {
+        return Err(not_a_candidate(
+            RollbackCandidateExclusion::OutsideTheOneRevolutionWindow,
+        ));
+    }
     match tree_table_has_no_entries(reader, &target_root) {
         Ok(true) => Err(not_a_candidate(
             RollbackCandidateExclusion::VersionWithoutFile,
@@ -5870,6 +6036,7 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         &system_configuration,
         &current_table,
         &ring_slots_known_to_hold_a_root_by(allocator),
+        current.root.checkpoint_txg,
         target,
     )?;
     let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index c8b4776..e9ea14a 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -34,10 +34,13 @@ use crate::extent_tree::{
     extent_key_bytes, find_upper_leaf_entry, read_lower_segment, ExtentTreeHeaderJudgement,
     ExtentTreeReading, ExtentUpperLeafTarget,
 };
-use crate::pointer::{DataPointer, LocationEntry, NodePointer};
+use crate::pointer::{
+    DataPointer, LocationEntry, NodePointer, PointerHeadFieldOutsideTheFirstVersion,
+};
 use crate::records::{
-    mapping_key_for_data, parse_inode_internal_entry, parse_mapping_entry, InodeRecord,
-    NLINK_OF_AN_INODE_BEING_DELETED, TREE_KIND_EXTENT, TREE_KIND_INODE,
+    mapping_key_for_data, parse_inode_internal_entry_or_refusal, parse_mapping_entry, InodeRecord,
+    PointerCarryingEntryRefusal, NLINK_OF_AN_INODE_BEING_DELETED, TREE_KIND_EXTENT,
+    TREE_KIND_INODE,
 };
 use crate::recovery::{
     choose_root, choose_system_configuration_naming_devices_without_a_mountable_slot,
@@ -210,8 +213,16 @@ pub enum OpenPoolForReadFailure {
     InodeLeafContainerDisagreesWithTheInternalEntry { detail: &'static str },
     /// inode 记录解不开（填充 / flags / 预留非零，I-9.7）。
     InodeRecordMalformed,
-    /// 节点里的一条记录 / 条目解不开：盘上自述的条目宽窄于字段表。
+    /// 节点里的一条记录 / 条目解不开：盘上自述的条目宽窄于字段表（这一版的读者不认这一版写的条目布局）。
     RecordMalformed { what: &'static str },
+    /// 节点里的一条条目按字段表解得开，而它带的那条指针头部有一段不是第一版的取值（MAC / nonce 非 0、算法类型不是未加密、
+    /// 压缩算法码不认识、码 0 而压后长度非 0、extent 偏移非 0）：这条指针不能解引用，判该指针所在的结构损坏
+    /// （D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据））。与 `RecordMalformed` 分开报：
+    /// 那一种是条目布局不认，这一种是条目布局认得、指针指不过去（`records/2026-09-30-里程碑三开工第一轮.md` 第 8 批 core 簇第 8 条）。
+    PointerInTheRecordCannotBeDereferenced {
+        what: &'static str,
+        field: PointerHeadFieldOutsideTheFirstVersion,
+    },
 }
 
 impl From<RecoveryFailure> for OpenPoolForReadFailure {
@@ -449,11 +460,21 @@ pub fn open_pool_for_read(
     }
     let mut inode_records: Vec<InodeRecord> = Vec::new();
     for entry_bytes in &inode_root.entries {
-        let (_separator_key, identity_in_the_entry, child) = parse_inode_internal_entry(
+        let (_separator_key, identity_in_the_entry, child) = parse_inode_internal_entry_or_refusal(
             entry_bytes,
         )
-        .ok_or(OpenPoolForReadFailure::RecordMalformed {
-            what: "inode 树内部条目",
+        .map_err(|refusal| match refusal {
+            PointerCarryingEntryRefusal::NarrowerThanItsFieldTable => {
+                OpenPoolForReadFailure::RecordMalformed {
+                    what: "inode 树内部条目",
+                }
+            }
+            PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(field) => {
+                OpenPoolForReadFailure::PointerInTheRecordCannotBeDereferenced {
+                    what: "inode 树内部条目的子指针",
+                    field,
+                }
+            }
         })?;
         let container_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
             reader,
diff --git a/crates/singlefs-core/src/pointer.rs b/crates/singlefs-core/src/pointer.rs
index d13cc3a..a255b77 100644
--- a/crates/singlefs-core/src/pointer.rs
+++ b/crates/singlefs-core/src/pointer.rs
@@ -124,6 +124,48 @@ pub enum PointerHeadFieldOutsideTheFirstVersion {
     ExtentOffsetNotZero(PointerHeadExtentOffsetNotZero),
 }
 
+/// 指针头部第一版之外的取值落在哪一组字段上：读者按它挑报给调用方的那一句（`what`），两组各一句，诊断时分得开。
+/// 两组的处置相同（判该指针所在的结构损坏，不拿这条指针去解引用）。封闭集合，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum PointerHeadFieldGroup {
+    /// MAC / nonce、算法类型、压缩算法码与压后长度（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14）。
+    EncryptionOrCompression,
+    /// extent 偏移（C581（extent 偏移非 0 没有判据））。
+    ExtentOffset,
+}
+
+impl PointerHeadFieldOutsideTheFirstVersion {
+    /// 这一处落在哪一组字段上。
+    #[must_use]
+    pub fn field_group(self) -> PointerHeadFieldGroup {
+        match self {
+            PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero
+            | PointerHeadFieldOutsideTheFirstVersion::AlgorithmTypeNotUnencrypted { .. }
+            | PointerHeadFieldOutsideTheFirstVersion::CompressionAlgorithmCodeNotRecognized {
+                ..
+            }
+            | PointerHeadFieldOutsideTheFirstVersion::CompressedLengthNotZeroWithoutCompression {
+                ..
+            } => PointerHeadFieldGroup::EncryptionOrCompression,
+            PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(_) => {
+                PointerHeadFieldGroup::ExtentOffset
+            }
+        }
+    }
+}
+
+/// 读盘上指针的读者判不判头部里第一版之外的取值（加密、压缩那几段与 extent 偏移）。封闭集合，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum PointerHeadFieldsJudgement {
+    /// 走读与挂载的读者：判，有一段不是第一版的取值就交回 [`PointerHeadFieldOutsideTheFirstVersion`]，调用方判该指针所在的结构损坏
+    /// （D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据））。
+    JudgedAsTheStructureIsCorruptOutsideTheFirstVersion,
+    /// 影子账认一条被抛弃根引用着哪些单元的读者：不判，指针里的位置条目照样认。影子账要的是多隔离、不漏隔离
+    /// （主 agent 2026-10-03 判，`records/2026-09-30-里程碑三开工第一轮.md` 第 8 批 core 簇第 7 条）：
+    /// 判了就少认一个单元，被抛弃根的那一个不隔离、回退之后可能被发出去。
+    NotJudgedByTheShadowLedger,
+}
+
 /// 指针头部 extent 偏移那 2 字节（头部偏移 32）读到非 0。第一版写者恒写 0（[`PointerHead::write_to`]），读者遇到非 0
 /// 判该指针所在的结构损坏，同 MAC / nonce（C581（extent 偏移非 0 没有判据））。**这是盘上读来的字节、不是我们的不变量**。
 /// 带着读到的值。
@@ -194,9 +236,9 @@ impl PointerHeadAsRead {
             .map_err(PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero)
     }
 
-    /// 只判 extent 偏移（C581）：多层码 2 树内部条目与 journal 新根段之外读指针头部的每一个 core 读者走这一判
-    /// （根记录、树表条目、inode 内部条目、extent 树的上段叶条目、内部条目与叶记录、分配记录树内部条目、实例表链指针；
-    /// 判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行），加密与压缩那几段在它们上面还没换成判（实审 A3a 的报告列着）。
+    /// 只判 extent 偏移（C581）：今天只剩根记录（`root_record.rs`）走这一判，加密与压缩那几段在它上面还没换成判；树表条目、inode 内部条目、
+    /// extent 树的上段叶条目、内部条目与叶记录、分配记录树内部条目、实例表链指针已换成 [`Self::judged`]（判决
+    /// `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行、`records/2026-09-30-里程碑三开工第一轮.md` 第 8 批 core 簇第 9 条）。
     fn judged_extent_offset(self) -> Result<PointerHead, PointerHeadExtentOffsetNotZero> {
         if self.extent_offset != 0 {
             return Err(PointerHeadExtentOffsetNotZero {
@@ -339,6 +381,22 @@ impl NodePointer {
         Ok(Self::read_after_the_head(head, reader))
     }
 
+    /// 读一条子指针，判不判头部第一版之外的取值由读者定（[`PointerHeadFieldsJudgement`]）：走读的读者判，影子账的读者不判。
+    ///
+    /// # Errors
+    /// `judgement` 是判的那一种、而头部有一段不是第一版的取值 ⇒ [`PointerHeadFieldOutsideTheFirstVersion`]；不判的那一种恒 `Ok`。
+    pub fn read_with_head_fields_judgement_from(
+        reader: &mut ByteReader<'_>,
+        judgement: PointerHeadFieldsJudgement,
+    ) -> Result<Self, PointerHeadFieldOutsideTheFirstVersion> {
+        match judgement {
+            PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion => {
+                Self::read_judging_the_encryption_compression_and_extent_offset_fields_from(reader)
+            }
+            PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger => Ok(Self::read_from(reader)),
+        }
+    }
+
     fn read_after_the_head(head: PointerHead, reader: &mut ByteReader<'_>) -> Self {
         let locations = [
             LocationEntry::read_from(reader),
diff --git a/crates/singlefs-core/src/records.rs b/crates/singlefs-core/src/records.rs
index 05002fe..f19449e 100644
--- a/crates/singlefs-core/src/records.rs
+++ b/crates/singlefs-core/src/records.rs
@@ -13,7 +13,7 @@ use crate::address::{
 use crate::bytes::{ByteReader, ByteWriter};
 use crate::pointer::{
     BirthSequence, DataPointer, LocationEntry, NodePointer, PointerHead,
-    PointerHeadExtentOffsetNotZero,
+    PointerHeadFieldOutsideTheFirstVersion,
 };
 use crate::unit::{PackedIdentity, WriteOrder, UNIT_CLASS_DATA};
 
@@ -426,8 +426,9 @@ impl TreeTableEntry {
             ..TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES
                 + usize::try_from(NODE_POINTER_BYTES).expect("86")]
     }
-    /// 读者：条目长度、flags 未知位（D8（核心索引结构） 已定项 8 ㊁：一律拒收）、预留 76 非零、根指针头部 extent 偏移非 0
-    /// （C581（extent 偏移非 0 没有判据）：判该指针所在的结构——这条条目——损坏）都判不可用。
+    /// 读者：条目长度、flags 未知位（D8（核心索引结构） 已定项 8 ㊁：一律拒收）、预留 76 非零、根指针头部有一段不是第一版的取值
+    /// （MAC / nonce 非 0、算法类型不是未加密、压缩算法码不认识、码 0 而压后长度非 0、extent 偏移非 0：D19（块指针的结构与宽度预算） 已定项 3、
+    /// D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据），判该指针所在的结构——这条条目——损坏）都判不可用。
     #[must_use]
     pub fn parse(bytes: &[u8]) -> Option<Self> {
         if bytes.len() != usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200")
@@ -444,7 +445,11 @@ impl TreeTableEntry {
         if reader.get_u16() != 0 {
             return None;
         }
-        let root = NodePointer::read_judging_the_extent_offset_from(&mut reader).ok()?;
+        let root =
+            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+                &mut reader,
+            )
+            .ok()?;
         let _previous_snapshot_txg = reader.get_u64();
         let birth_txg = CheckpointTxg(reader.get_u64());
         let head_identifier = reader.get_u64();
@@ -464,14 +469,14 @@ impl TreeTableEntry {
 pub enum PointerCarryingEntryRefusal {
     /// 条目宽（索引节点头里的盘上字段）窄于字段表：调用方报「条目窄于字段表」。
     NarrowerThanItsFieldTable,
-    /// 条目里那条指针的头部 extent 偏移非 0：判该指针所在的结构（这条条目）损坏，同 MAC / nonce
-    /// （D19（块指针的结构与宽度预算） 已定项 3、C581（extent 偏移非 0 没有判据））。
-    PointerExtentOffsetNotZero(PointerHeadExtentOffsetNotZero),
+    /// 条目里那条指针的头部有一段不是第一版的取值（MAC / nonce、算法类型、压缩算法码与压后长度、extent 偏移）：判该指针所在的结构
+    /// （这条条目）损坏（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据））。
+    PointerHeadFieldOutsideTheFirstVersion(PointerHeadFieldOutsideTheFirstVersion),
 }
 
 /// inode 内部条目 120 的读者。条目宽是索引节点头里的一个盘上字段（`parse_index_node` 只判了它 ≥ key 宽 8），
-/// 窄于字段表、或子指针头部 extent 偏移非 0 时返回 `None`：这里是盘上字节进字段表的边界，往里就按 120 信它。
-/// 要分清是哪一种的调用方用 [`parse_inode_internal_entry_or_refusal`]；只要知道解不解得开的（只读挂载打开池，两种都报同一个成员）用这一个。
+/// 窄于字段表、或子指针头部有一段不是第一版的取值时返回 `None`：这里是盘上字节进字段表的边界，往里就按 120 信它。
+/// 要分清是哪一种的调用方（冷启动走读、只读挂载打开池）用 [`parse_inode_internal_entry_or_refusal`]；只要知道解不解得开的（用例）用这一个。
 #[must_use]
 pub fn parse_inode_internal_entry(bytes: &[u8]) -> Option<(u64, PackedIdentity, NodePointer)> {
     parse_inode_internal_entry_or_refusal(bytes).ok()
@@ -480,7 +485,7 @@ pub fn parse_inode_internal_entry(bytes: &[u8]) -> Option<(u64, PackedIdentity,
 /// 同 [`parse_inode_internal_entry`]，解不开时交回是哪一种。
 ///
 /// # Errors
-/// 窄于 120 ⇒ `NarrowerThanItsFieldTable`；子指针头部 extent 偏移非 0 ⇒ `PointerExtentOffsetNotZero`。
+/// 窄于 120 ⇒ `NarrowerThanItsFieldTable`；子指针头部有一段不是第一版的取值 ⇒ `PointerHeadFieldOutsideTheFirstVersion`。
 pub fn parse_inode_internal_entry_or_refusal(
     bytes: &[u8],
 ) -> Result<(u64, PackedIdentity, NodePointer), PointerCarryingEntryRefusal> {
@@ -495,16 +500,18 @@ pub fn parse_inode_internal_entry_or_refusal(
         container: reader.get_u64(),
         container_birth: CheckpointTxg(reader.get_u64()),
     };
-    let child = NodePointer::read_judging_the_extent_offset_from(&mut reader)
-        .map_err(PointerCarryingEntryRefusal::PointerExtentOffsetNotZero)?;
+    let child = NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+        &mut reader,
+    )
+    .map_err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion)?;
     Ok((separator_key, child_identity, child))
 }
 
 /// extent 叶记录 112 的读者：key 24 + 指针 88（同 [`parse_inode_internal_entry_or_refusal`]：`parse_index_node` 只判了条目宽 ≥ key 宽 24，
-/// 切到偏移 112 之前要在这里判一次；数据指针头部的 extent 偏移也在这里判）。
+/// 切到偏移 112 之前要在这里判一次；数据指针头部第一版之外的取值也在这里判）。
 ///
 /// # Errors
-/// 窄于 112 ⇒ `NarrowerThanItsFieldTable`；数据指针头部 extent 偏移非 0 ⇒ `PointerExtentOffsetNotZero`。
+/// 窄于 112 ⇒ `NarrowerThanItsFieldTable`；数据指针头部有一段不是第一版的取值 ⇒ `PointerHeadFieldOutsideTheFirstVersion`。
 pub fn parse_extent_record(
     bytes: &[u8],
 ) -> Result<([u8; 24], DataPointer), PointerCarryingEntryRefusal> {
@@ -512,8 +519,11 @@ pub fn parse_extent_record(
         return Err(PointerCarryingEntryRefusal::NarrowerThanItsFieldTable);
     }
     let key: [u8; 24] = bytes[..24].try_into().expect("24");
-    let pointer = DataPointer::read_judging_the_extent_offset_from(&mut ByteReader::at(bytes, 24))
-        .map_err(PointerCarryingEntryRefusal::PointerExtentOffsetNotZero)?;
+    let pointer =
+        DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+            &mut ByteReader::at(bytes, 24),
+        )
+        .map_err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion)?;
     Ok((key, pointer))
 }
 
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index c9e0bd3..df24cbf 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -61,7 +61,10 @@ use crate::journal::{
 };
 use crate::lineage_tree::LineageContainerIndexInTree;
 use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
-use crate::pointer::{location_entries_ascend_by_device, DataPointer, LocationEntry, NodePointer};
+use crate::pointer::{
+    location_entries_ascend_by_device, DataPointer, LocationEntry, NodePointer,
+    PointerHeadFieldGroup, PointerHeadFieldOutsideTheFirstVersion, PointerHeadFieldsJudgement,
+};
 use crate::records::{
     mapping_key_for_data, mapping_key_for_node, parse_inode_internal_entry_or_refusal,
     parse_mapping_entry, AccountingEntry, InodeRecord, LineageRecord, PointerCarryingEntryRefusal,
@@ -2177,6 +2180,7 @@ struct InstanceTableChainRead {
 fn read_instance_table_chain<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     root: &RootRecord,
+    chain_pointer_head_fields: PointerHeadFieldsJudgement,
 ) -> InstanceTableChainRead {
     let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
     let mut page_pointers = vec![root.instance_table];
@@ -2193,7 +2197,11 @@ fn read_instance_table_chain<Reader: PoolReader + ?Sized>(
                 }
             }
         };
-        let Some(page) = InstanceTablePage::parse(&bytes, page_index) else {
+        let Some(page) = InstanceTablePage::parse_with_chain_pointer_head_fields_judgement(
+            &bytes,
+            page_index,
+            chain_pointer_head_fields,
+        ) else {
             return InstanceTableChainRead {
                 page_pointers,
                 outcome: Err(RecoveryFailure::UnitMalformed {
@@ -2227,7 +2235,11 @@ pub fn instance_table_chain_of_root<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     root: &RootRecord,
 ) -> Result<InstanceTableChain, RecoveryFailure> {
-    let read = read_instance_table_chain(reader, root);
+    let read = read_instance_table_chain(
+        reader,
+        root,
+        PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion,
+    );
     read.outcome.map(|records| InstanceTableChain {
         records,
         page_pointers: read.page_pointers,
@@ -2236,12 +2248,19 @@ pub fn instance_table_chain_of_root<Reader: PoolReader + ?Sized>(
 
 /// 一条根指着的实例表链上认得出的每一片的指针（第 0 片恒在）：读得出的就沿链往下认，读不出、解不开的那一片自己的指针也在，
 /// 它后面的认不出。影子账拿它隔离一条被抛弃根的整条链——读不全的链少隔离一片，被抛弃根在环里被覆写之前那一片就可能被发出去。
+/// 所以链指针头部第一版之外的取值（加密、压缩那几段与 extent 偏移）这里不判（[`PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger`]）：
+/// 指针里的位置条目照样认下一片；沿链读整张表（[`instance_table_chain_of_root`]）照判。
 #[must_use]
 pub fn instance_table_page_pointers_as_far_as_readable<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     root: &RootRecord,
 ) -> Vec<NodePointer> {
-    read_instance_table_chain(reader, root).page_pointers
+    read_instance_table_chain(
+        reader,
+        root,
+        PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger,
+    )
+    .page_pointers
 }
 
 /// 一条根指着的树表是不是 0 条（mkfs 的第 0 代树表，第一次可写挂载的暖机根照抄它）：这条根下面还没有发布过文件版本。
@@ -3713,6 +3732,24 @@ const CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE: ContainerTreeRolesOfOneTree =
 /// （C581（extent 偏移非 0 没有判据）：判该指针所在的结构损坏，与 inode 树内部条目同一条）。
 pub const CONTAINER_TREE_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO: &str =
     "墓碑树或谱系树内部条目的子指针头部 extent 偏移不是 0";
+/// 墓碑树、谱系树码 2 节点条目里的子指针头部加密、压缩那几段（MAC / nonce、算法类型、压缩算法码与压后长度）不是第一版的取值时报
+/// [`RecoveryFailure::UnitMalformed`]，`what` 取这一句（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14，与 inode 树内部条目同一条）。
+pub const CONTAINER_TREE_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION:
+    &str = "墓碑树或谱系树内部条目的子指针头部的加密、压缩那几段不是第一版的取值";
+
+/// 一条带指针的条目里那条指针头部第一版之外的取值报哪一句：按落在哪一组字段上，在读者给的两句里挑。
+fn what_of_a_pointer_head_field_outside_the_first_version(
+    field: PointerHeadFieldOutsideTheFirstVersion,
+    what_of_the_encryption_or_compression_fields: &'static str,
+    what_of_the_extent_offset: &'static str,
+) -> &'static str {
+    match field.field_group() {
+        PointerHeadFieldGroup::EncryptionOrCompression => {
+            what_of_the_encryption_or_compression_fields
+        }
+        PointerHeadFieldGroup::ExtentOffset => what_of_the_extent_offset,
+    }
+}
 
 /// 读一个进映射的树节点（码 2 或码 3，提示读不出经映射回退）：重建那一路收进来的读盘办法。
 type ReadOneMappedTreeNode<'reading> =
@@ -3854,10 +3891,14 @@ fn read_container_tree_index_nodes(
                                         .expect("120"),
                                 }
                             }
-                            PointerCarryingEntryRefusal::PointerExtentOffsetNotZero(
-                                _extent_offset,
+                            PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(
+                                field,
                             ) => RecoveryFailure::UnitMalformed {
-                                what: CONTAINER_TREE_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
+                                what: what_of_a_pointer_head_field_outside_the_first_version(
+                                    field,
+                                    CONTAINER_TREE_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+                                    CONTAINER_TREE_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
+                                ),
                             },
                         },
                     )?;
@@ -4426,9 +4467,14 @@ struct InodeInternalEntry {
 /// （C581（extent 偏移非 0 没有判据）：判该指针所在的结构损坏）。
 pub const INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO: &str =
     "inode 树内部条目的子指针头部 extent 偏移不是 0";
+/// inode 树内部条目里的子指针头部加密、压缩那几段（MAC / nonce、算法类型、压缩算法码与压后长度）不是第一版的取值时报
+/// [`RecoveryFailure::UnitMalformed`]，`what` 取这一句（D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14：判该指针所在的结构损坏）。
+pub const INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION:
+    &str = "inode 树内部条目的子指针头部的加密、压缩那几段不是第一版的取值";
 
 /// 解一条 inode 树内部条目（条目宽是盘上的值，窄于 120 报 `EntryNarrowerThanItsFieldTable`；子指针头部 extent 偏移非 0 报
-/// `UnitMalformed`，`what` 取 [`INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO`]），判类型段是 2（I-9.2（条目身份与子头相符）
+/// `UnitMalformed`，`what` 取 [`INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO`]，加密、压缩那几段不是第一版的取值取
+/// [`INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]），判类型段是 2（I-9.2（条目身份与子头相符）
 /// 的第一句；第一版 inode 树根下面只有叶容器）。
 fn inode_internal_entry_parsed_and_judged(
     entry: &[u8],
@@ -4442,9 +4488,13 @@ fn inode_internal_entry_parsed_and_judged(
                     field_table_bytes: usize::try_from(INODE_INTERNAL_ENTRY).expect("120"),
                 }
             }
-            PointerCarryingEntryRefusal::PointerExtentOffsetNotZero(_extent_offset) => {
+            PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(field) => {
                 RecoveryFailure::UnitMalformed {
-                    what: INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
+                    what: what_of_a_pointer_head_field_outside_the_first_version(
+                        field,
+                        INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+                        INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
+                    ),
                 }
             }
         })?;
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 194e9bd..ddaaa2f 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -1752,8 +1752,14 @@ pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
         let placement = allocate_placement_for_role(allocator, *identity, txg)?;
         slots.insert(*identity, placement.slot);
     }
-    // 落点取完、这次不再分配：记下这条根盖掉的根环槽（同 `publish_admitted` 那一处；失败时分配器由调用方整个换回去）。
-    allocator.record_root_written_by_this_process(txg);
+    // 落点取完、这次不再分配：记下这条根盖掉的根环槽（同 `publish_admitted` 那一处；失败时分配器由调用方整个换回去），
+    // 记之前照 E2 重读挂载时读不出的那一槽（`mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount`）。
+    crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
+        &*pool.devices,
+        pool.parameters,
+        allocator,
+        txg,
+    );
     // 这一次发布的提交内生块（实例表链的各片、分配记录树的节点）都归树 0（这一版还没登记过任何树），在 (txg, 实例) 上连着发出生序号：
     // 实例表各片按 bump 次序（尾片先）在前，分配记录树的节点在后（先叶后根）。
     let mut sequences = BirthSequenceAllocator::default();
@@ -7741,7 +7747,13 @@ fn publish_admitted<Device: BlockDevice>(
     }
     // 这次的根要盖掉根环里的一个槽：落点都取完了、这次不再分配，在装记账行之前记下它——盖掉的是被抛弃的根就清它的隔离位，
     // 环里最旧有效根往前挪了就按谓词回收，记账行按回收之后的数写（与这条根一起说同一件事）。失败时整个分配器由调用方换回去。
-    allocator.record_root_written_by_this_process(txg);
+    // 记之前照 E2 重读挂载时读不出、记了这条根之后会是窗口里最小的那一槽（D16（发布语义） 已定项 1，用户 2026-10-03 定）。
+    crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
+        &*pool.devices,
+        pool.parameters,
+        allocator,
+        txg,
+    );
     let slot_of = |identity: TransactionUnit| slots[&identity];
     let device_identities: Vec<DeviceIdentity> =
         pool.devices.iter().map(|(identity, _)| *identity).collect();
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index da01f95..ec5a973 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -433,6 +433,16 @@ pub struct ModelRingPosition {
 const MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 =
     ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;
 
+/// 一圈窗口（D16（发布语义） 已定项 1）：txg 落不落在锚 `window_anchor` 之下一圈里，txg > N − 3S（写成 txg + 3S > N，N < 3S 时整个环都在窗口里）。
+/// 模型自己算一份，不引实现的那一份（D13（验证路线） 已定项 5）。
+fn is_inside_the_one_revolution_window_of(
+    checkpoint_txg: ModelCheckpointTxg,
+    window_anchor: ModelCheckpointTxg,
+) -> bool {
+    checkpoint_txg.0 + ROOT_RING_REGIONS * MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM
+        > window_anchor.0
+}
+
 fn ring_position_of(checkpoint_txg: ModelCheckpointTxg) -> ModelRingPosition {
     ModelRingPosition {
         region: checkpoint_txg.0 % ROOT_RING_REGIONS,
@@ -669,6 +679,9 @@ pub enum ModelRefusalReason {
     RollbackTargetBelowEffectiveFloor,
     /// 回退目标在被抛弃的时间线上（D23（journal 的角色与格式） 已定项 14：有行 (i, Ti, Wi) 且 T > Ti）。
     RollbackTargetOnAbandonedTimeline,
+    /// 回退目标的 txg 落在现行那一版的一圈窗口外（D16（发布语义） 已定项 1「回退候选集」：txg 落在一圈窗口里；窗口 = txg > N − 3S，
+    /// N 取现行那一版的 txg，3S 是环长）。
+    RollbackTargetOutsideTheOneRevolutionWindow,
     /// 回退目标那一版树表 0 条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根）。
     RollbackTargetWithoutFile,
     /// 要建立新实例的那一版树表 0 条、而它的实例表已经不是 mkfs 那一片（上一次挂载在这一版上写过行）：
@@ -701,6 +714,9 @@ impl ModelRefusalReason {
             ModelRefusalReason::RollbackTargetNotInRing => "回退目标不在根环里",
             ModelRefusalReason::RollbackTargetBelowEffectiveFloor => "回退目标低于 F_生效",
             ModelRefusalReason::RollbackTargetOnAbandonedTimeline => "回退目标在被抛弃的时间线上",
+            ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow => {
+                "回退目标的 txg 落在现行那一版的一圈窗口外"
+            }
             ModelRefusalReason::RollbackTargetWithoutFile => "回退目标那一版没有文件",
             ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem => {
                 "树表 0 条、而实例表已经不是 mkfs 那一片（条款没写被换下的那一片记在哪）"
@@ -725,6 +741,7 @@ impl ModelRefusalReason {
             | ModelRefusalReason::RollbackTargetNotInRing
             | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
             | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
+            | ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow
             | ModelRefusalReason::RollbackTargetWithoutFile
             | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
             | ModelRefusalReason::FloorAboveCeiling
@@ -1265,7 +1282,7 @@ impl IdealModel {
     }
 
     /// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的有效根, 第 4 新的非空有效根)；非空有效根不足 4 个时取最旧的有效根。
-    /// 有效 = 按最新根的实例表不被抛弃 ∧ txg ≥ 现行的 F（「非空」从盘上怎么认，2026-09-17 用户定案）；现行的 F 取 F_生效（不建崩溃时与
+    /// 有效 = 按最新根的实例表不被抛弃 ∧ txg ≥ 现行的 F ∧ 落在一圈窗口里（「非空」从盘上怎么认，2026-09-17 用户定案；窗口那一项 B2）；现行的 F 取 F_生效（不建崩溃时与
     /// 现行根带的相等）。非空 = inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排）的不同，最旧的有效根与「没有这两棵树」比。
     /// 一块盘上没有有效根时不算它（条款没写这一格；不建崩溃时抬 F 与暖机都推到每块盘都有，走不到）。预想，跟收口表第 ② 行
     /// （候选集下界用哪个 F、被抛弃根带的 F 算不算都在那一行里重议）。
@@ -1273,12 +1290,23 @@ impl IdealModel {
     pub fn rollback_floor_ceiling(&self) -> Option<ModelCheckpointTxg> {
         let current_floor = self.effective_rollback_floor();
         let newest_table = Rc::clone(&self.newest_root().instance_table_rows);
+        // 上限与「非空」只在一圈窗口里的有效根上取（B2，用户 2026-10-03 定，D16 已定项 1 同取这个窗口的第六处）：锚 = 环里最新的有效根。
+        let window_anchor = self
+            .ring
+            .values()
+            .filter(|root| !Self::is_abandoned_by(root, &newest_table))
+            .map(|root| root.key.checkpoint_txg)
+            .max()?;
         let mut valid: Vec<&ModelRoot> = self
             .ring
             .values()
             .filter(|root| {
                 !Self::is_abandoned_by(root, &newest_table)
                     && root.key.checkpoint_txg >= current_floor
+                    && is_inside_the_one_revolution_window_of(
+                        root.key.checkpoint_txg,
+                        window_anchor,
+                    )
             })
             .collect();
         valid.sort_by_key(|root| root.key);
@@ -2213,6 +2241,13 @@ impl IdealModel {
                 if Self::is_abandoned_by(target_root, &current.instance_table_rows) {
                     required_refusals.insert(ModelRefusalReason::RollbackTargetOnAbandonedTimeline);
                 }
+                if !is_inside_the_one_revolution_window_of(
+                    target.checkpoint_txg,
+                    current.key.checkpoint_txg,
+                ) {
+                    required_refusals
+                        .insert(ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow);
+                }
                 if !target_root.contents.has_trees() {
                     required_refusals.insert(ModelRefusalReason::RollbackTargetWithoutFile);
                 }
@@ -2497,6 +2532,7 @@ impl IdealModel {
             | ModelRefusalReason::RollbackTargetNotInRing
             | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
             | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
+            | ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow
             | ModelRefusalReason::RollbackTargetWithoutFile
             | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
             | ModelRefusalReason::FloorAboveCeiling
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index e4b6db4..fb61ac4 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -717,7 +717,8 @@ pub fn refusal_reason_of_placement_refusal(refusal: &PlacementRefusal) -> Observ
 }
 
 /// 回退目标不在候选集里的那一条说的是哪条理由：按字段一对一映射，不看给人看的文字（增补 3 第 2 件代码三方第一轮判决第三节第 2 条）。
-/// 候选集四条（D23（journal 的角色与格式） 已定项 14：根环里、txg ≥ F_生效、按现行那一版的实例表判仍然有效、带文件），各映射到自己那一条。
+/// 候选集五条（D23（journal 的角色与格式） 已定项 14：根环里、txg ≥ F_生效、按现行那一版的实例表判仍然有效、带文件；D16（发布语义） 已定项 1
+/// 「回退候选集」：txg 落在一圈窗口里），各映射到自己那一条。
 #[must_use]
 pub fn refusal_reason_of_rollback_candidate_exclusion(
     exclusion: RollbackCandidateExclusion,
@@ -730,6 +731,9 @@ pub fn refusal_reason_of_rollback_candidate_exclusion(
         RollbackCandidateExclusion::OnAbandonedTimeline => {
             ModelRefusalReason::RollbackTargetOnAbandonedTimeline
         }
+        RollbackCandidateExclusion::OutsideTheOneRevolutionWindow => {
+            ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow
+        }
         RollbackCandidateExclusion::VersionWithoutFile => {
             ModelRefusalReason::RollbackTargetWithoutFile
         }
@@ -1272,8 +1276,9 @@ mod tests {
         );
     }
 
-    /// 回退候选集的四条排除各映射到自己那一条理由、互不相同（此前「不在候选集里」映射成「低于 F、被抛弃」两条之一）；
-    /// 「目标那一版树表 0 条」是第四条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根）。
+    /// 回退候选集的五条排除各映射到自己那一条理由、互不相同（此前「不在候选集里」映射成「低于 F、被抛弃」两条之一）；
+    /// 「目标那一版树表 0 条」是第五条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根），「落在一圈窗口外」是第四条
+    /// （D16（发布语义） 已定项 1「回退候选集」）。
     #[test]
     fn each_rollback_candidate_exclusion_maps_to_its_own_reason() {
         let target = RollbackTarget {
@@ -1284,6 +1289,7 @@ mod tests {
             RollbackCandidateExclusion::NotInRing,
             RollbackCandidateExclusion::BelowEffectiveFloor,
             RollbackCandidateExclusion::OnAbandonedTimeline,
+            RollbackCandidateExclusion::OutsideTheOneRevolutionWindow,
             RollbackCandidateExclusion::VersionWithoutFile,
         ]
         .into_iter()
@@ -1300,6 +1306,9 @@ mod tests {
                 ObservedRefusalReason::Explained(
                     ModelRefusalReason::RollbackTargetOnAbandonedTimeline
                 ),
+                ObservedRefusalReason::Explained(
+                    ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow
+                ),
                 ObservedRefusalReason::Explained(ModelRefusalReason::RollbackTargetWithoutFile),
             ]
         );
diff --git a/crates/singlefs-harness/tests/common_pointer_head_corruption/mod.rs b/crates/singlefs-harness/tests/common_pointer_head_corruption/mod.rs
new file mode 100644
index 0000000..134c8d3
--- /dev/null
+++ b/crates/singlefs-harness/tests/common_pointer_head_corruption/mod.rs
@@ -0,0 +1,516 @@
+//! 改坏一条盘上指针的头部（或它所在节点的条目宽）、沿引用链补校验和的搭建：在发布路径写出的一版上找到那条指针所在的单元，
+//! 把它头部的一段由第一版的取值改成别的、重封这个单元，新的整单元校验和沿引用链补上去（父单元里的位置条目、中央映射条目、根槽），
+//! 再把每条根抄进它那几条 journal 记录的新根段。做法与 `pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs`
+//! 里那一份相同，这里按改哪一段字段参数化（那一份只改 extent 偏移）。
+//! 用这一份的测试文件同时声明 `mod common;`（这里引它的建池与改记录那几样）。
+#![allow(dead_code, reason = "每个测试文件各自只用到其中一部分")]
+
+use singlefs_core::address::{
+    DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, SlotNumber,
+};
+use singlefs_core::allocation_record_tree::{
+    node_pointers_as_far_as_readable, AllocationRecordTreeGeometry,
+};
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::bytes::ByteWriter;
+use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
+use singlefs_core::extent_tree::ExtentUpperLeafEntry;
+use singlefs_core::mount::{mount_writable, MountError};
+use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
+use singlefs_core::pointer::{DataPointer, NodePointer};
+use singlefs_core::records::{TreeTableEntry, TREE_KIND_EXTENT};
+use singlefs_core::recovery::{
+    choose_root, choose_system_configuration, every_root_ring_slot, recover, JournalPolicy,
+    RecoveryFailure, RecoveryOutcome,
+};
+use singlefs_core::root_record::{RootRecord, ROOT_CHECKSUM_OFFSET};
+use singlefs_core::root_ring::slot_offset;
+use singlefs_core::transaction::{acquire_instance, PoolWriter, FIRST_INODE_NUMBER};
+use singlefs_core::unit::{
+    data_unit_payload_capacity, parse_data_unit, parse_index_node, parse_packed_unit,
+    seal_header_checksum, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
+};
+use singlefs_format::{
+    DATA_POINTER_BYTES, DATA_UNIT_BYTES, NODE_BYTES, NODE_POINTER_BYTES, SLOT_BYTES,
+    UNIT_AREA_START_SLOT,
+};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use singlefs_harness::scenario::run_new_pool_file_creation;
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+use crate::common::{
+    build_pool, copy_each_root_into_the_new_root_segment_of_its_journal_records,
+    memory_pool_of_sparse_devices, parameters,
+    sequential_write_in_process_as_instance_one_and_advance_the_pool, IMAGE_BYTES,
+};
+
+/// 单元头第 6 个字节是单元类（共同前缀：magic 4、格式版本 2 之后）。
+const UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER: usize = 6;
+/// 根槽宽 = physical_block_size（`common::parameters` 取 512）。
+const ROOT_SLOT_BYTES: usize = 512;
+/// 一片实例表里数据行的上限：370 条记录减去链指针那一条（D18（块里携带什么信息） 已定项 11）。
+const INSTANCE_ROWS_PER_PAGE: u32 = 369;
+
+/// 指针头部里第一版写定值的一段（D19（块指针的结构与宽度预算） 已定项 7 的字段表：MAC 16 / nonce 12 / 算法类型 1 / 压缩算法码 1 / 压后长度 2 /
+/// extent 偏移 2），与把它改成的那个非第一版的值。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum PointerHeadFieldChange {
+    /// MAC 第 0 字节改 1（加密关着时 MAC 恒 0，D19 已定项 3）。
+    FirstMacByteToOne,
+    /// nonce 第 0 字节改 1（加密关着时 nonce 恒 0，D19 已定项 3）。
+    FirstNonceByteToOne,
+    /// 算法类型改 1（登记过的加密算法，第一版不支持加密，D9（加密） 已定项 10、D22（单元原子性怎么合成） 已定项 17）。
+    AlgorithmTypeToOne,
+    /// 压缩算法码改 1（登记表第一版只有码 0，D9（加密） 已定项 14）。
+    CompressionAlgorithmCodeToOne,
+    /// 压缩算法码仍 0、压后长度改 1（D9（加密） 已定项 14「码 0 时它恒 0，非 0 判该指针损坏」）。
+    CompressedLengthToOneWithoutCompression,
+}
+
+impl PointerHeadFieldChange {
+    /// 这一段在指针头部里的偏移与改成的字节。
+    fn offset_and_bytes(self) -> (usize, Vec<u8>) {
+        match self {
+            PointerHeadFieldChange::FirstMacByteToOne => (0, vec![1]),
+            PointerHeadFieldChange::FirstNonceByteToOne => (16, vec![1]),
+            PointerHeadFieldChange::AlgorithmTypeToOne => (28, vec![1]),
+            PointerHeadFieldChange::CompressionAlgorithmCodeToOne => (29, vec![1]),
+            PointerHeadFieldChange::CompressedLengthToOneWithoutCompression => {
+                (30, 1u16.to_le_bytes().to_vec())
+            }
+        }
+    }
+}
+
+pub fn node_bytes() -> usize {
+    usize::try_from(NODE_BYTES).expect("16384")
+}
+
+pub fn data_unit_bytes() -> usize {
+    usize::try_from(DATA_UNIT_BYTES).expect("32768")
+}
+
+/// 盘 0 上从 `slot` 起 `length_in_bytes` 字节（第一版两盘同槽同内容，读一份就够）。
+pub fn unit_bytes_at(image: &MemoryPool, slot: SlotNumber, length_in_bytes: usize) -> Vec<u8> {
+    image
+        .devices
+        .get(&DeviceIdentity(0))
+        .expect("池里有盘 0")
+        .read(slot.to_device_offset(), length_in_bytes)
+}
+
+/// 一个单元按单元类的整单元宽；单元类不认识（槽里不是单元的开头）时是 None。
+fn unit_bytes_of_class(unit_class: u8) -> Option<usize> {
+    match unit_class {
+        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => Some(data_unit_bytes()),
+        UNIT_CLASS_INDEX_NODE => Some(node_bytes()),
+        _not_a_unit_class => None,
+    }
+}
+
+/// 改过的单元重封：载荷 CRC 与头校验和（码 1 罩 [105, 32768)、住 101，头 [0, 105)；码 3 罩 [107, 32768)、住 89，头 [0, 107)；
+/// 码 2 罩 [86 + 2k, 16384)、住 76 + 2k，头 [0, 86 + 2k)，k 是 key 宽、住偏移 51）。
+fn reseal_unit(unit: &mut [u8]) {
+    let (header_end, payload_checksum_offset) = match unit[UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER] {
+        UNIT_CLASS_DATA => (105, 101),
+        UNIT_CLASS_PACKED => (107, 89),
+        UNIT_CLASS_INDEX_NODE => {
+            let key_width = usize::from(unit[51]);
+            (86 + 2 * key_width, 76 + 2 * key_width)
+        }
+        unknown_class => panic!("只重封三类单元，读到类 {unknown_class}"),
+    };
+    let payload_checksum = crc32_castagnoli(&unit[header_end..]);
+    unit[payload_checksum_offset..payload_checksum_offset + 4]
+        .copy_from_slice(&payload_checksum.to_le_bytes());
+    seal_header_checksum(unit, header_end);
+}
+
+/// 单元区里盘 0 上写过的每个单元（起始槽、整单元宽）：按槽号升序逐槽看单元头，按单元类取宽、解得开才算一个单元，32 KiB 的单元跳过它的第二个槽。
+fn units_in_the_unit_area(image: &MemoryPool) -> Vec<(SlotNumber, usize)> {
+    let unit_area_start = DeviceOffsetInBytes(UNIT_AREA_START_SLOT * SLOT_BYTES);
+    let device = image.devices.get(&DeviceIdentity(0)).expect("池里有盘 0");
+    let written_slots: std::collections::BTreeSet<u64> = device
+        .written_sectors_in(unit_area_start, IMAGE_BYTES - unit_area_start.0)
+        .into_iter()
+        .map(|sector| sector * 512 / SLOT_BYTES)
+        .collect();
+    let mut units = Vec::new();
+    let mut first_slot_not_yet_covered = UNIT_AREA_START_SLOT;
+    // 迭代上界是写过的槽数；跨轮携带的是上一个单元之后的第一个槽（32 KiB 单元的第二个槽不当单元的开头看）。
+    for slot in written_slots {
+        if slot < first_slot_not_yet_covered {
+            continue;
+        }
+        let header = unit_bytes_at(image, SlotNumber(slot), 512);
+        let Some(length_in_bytes) =
+            unit_bytes_of_class(header[UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER])
+        else {
+            continue;
+        };
+        let bytes = unit_bytes_at(image, SlotNumber(slot), length_in_bytes);
+        let is_a_unit = match header[UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER] {
+            UNIT_CLASS_DATA => parse_data_unit(&bytes).is_ok(),
+            UNIT_CLASS_PACKED => parse_packed_unit(&bytes).is_ok(),
+            _index_node => parse_index_node(&bytes).is_ok(),
+        };
+        if is_a_unit {
+            units.push((SlotNumber(slot), length_in_bytes));
+            first_slot_not_yet_covered =
+                slot + u64::try_from(length_in_bytes).expect("单元宽") / SLOT_BYTES;
+        }
+    }
+    units
+}
+
+/// 位置条目 14 字节的盘上样子：设备身份 4、槽号 6、整单元校验和 4。
+fn location_entry_bytes(device: DeviceIdentity, slot: SlotNumber, unit_checksum: u32) -> Vec<u8> {
+    [
+        device.0.to_le_bytes().to_vec(),
+        slot.0.to_le_bytes()[..6].to_vec(),
+        unit_checksum.to_le_bytes().to_vec(),
+    ]
+    .concat()
+}
+
+/// 把 `container` 里每一处 `from` 换成 `to`（等长），交回换了几处。
+fn replace_every_occurrence(container: &mut [u8], from: &[u8], to: &[u8]) -> usize {
+    assert_eq!(from.len(), to.len(), "等长替换");
+    let mut replaced = 0;
+    let mut position = 0;
+    // 迭代上界是容器的字节数；跨轮携带的是下一个要比的位置（换过的那一段整段跳过）。
+    while position + from.len() <= container.len() {
+        if container[position..position + from.len()] == *from {
+            container[position..position + from.len()].copy_from_slice(to);
+            replaced += 1;
+            position += from.len();
+        } else {
+            position += 1;
+        }
+    }
+    replaced
+}
+
+/// 根环里每个根槽（盘、偏移）。
+fn root_slots(image: &MemoryPool) -> Vec<(DeviceIdentity, DeviceOffsetInBytes)> {
+    let system_configuration = choose_system_configuration(image).expect("系统配置择得到");
+    every_root_ring_slot(&system_configuration.immutable.sizes)
+        .into_iter()
+        .map(|ring_slot| {
+            (
+                system_configuration.immutable.region_devices
+                    [usize::try_from(ring_slot.region).expect("区域号")],
+                slot_offset(
+                    ring_slot,
+                    system_configuration
+                        .immutable
+                        .sizes
+                        .fixed_structure_slot_spacing,
+                ),
+            )
+        })
+        .collect()
+}
+
+/// `slot` 那个单元（两盘同槽）的整单元校验和由 `old` 换成了 `new`：引用它的每一条位置条目（单元区里的父单元、中央映射条目、根槽）
+/// 换成新的，被换的单元重封、它自己的新校验和再往上补，直到根槽；最后把每条根抄进它那几条 journal 记录的新根段。
+fn propagate_the_new_unit_checksum(image: &mut MemoryPool, slot: SlotNumber, old: u32, new: u32) {
+    let units = units_in_the_unit_area(image);
+    let mut pending = vec![(slot, old, new)];
+    // 迭代上界：每换一处，被换的单元的新校验和入队一次；引用链从叶到根是有限的、不成环（单元只被更晚写出的单元引用）。
+    while let Some((changed_slot, old_checksum, new_checksum)) = pending.pop() {
+        if old_checksum == new_checksum {
+            continue;
+        }
+        for (container_slot, length_in_bytes) in &units {
+            if *container_slot == changed_slot {
+                continue;
+            }
+            let before = unit_bytes_at(image, *container_slot, *length_in_bytes);
+            let mut after = before.clone();
+            let mut replaced = 0;
+            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+                replaced += replace_every_occurrence(
+                    &mut after,
+                    &location_entry_bytes(device, changed_slot, old_checksum),
+                    &location_entry_bytes(device, changed_slot, new_checksum),
+                );
+            }
+            if replaced > 0 {
+                reseal_unit(&mut after);
+                for device in image.devices.values_mut() {
+                    device.write(container_slot.to_device_offset(), &after);
+                }
+                pending.push((
+                    *container_slot,
+                    crc32_castagnoli(&before),
+                    crc32_castagnoli(&after),
+                ));
+            }
+        }
+        for (device, offset) in root_slots(image) {
+            let sparse = image.devices.get_mut(&device).expect("根槽所在的盘在池里");
+            let mut root_slot = sparse.read(offset, ROOT_SLOT_BYTES);
+            let mut replaced = 0;
+            for location_device in [DeviceIdentity(0), DeviceIdentity(1)] {
+                replaced += replace_every_occurrence(
+                    &mut root_slot,
+                    &location_entry_bytes(location_device, changed_slot, old_checksum),
+                    &location_entry_bytes(location_device, changed_slot, new_checksum),
+                );
+            }
+            if replaced > 0 {
+                let digest = wide_checksum_with_field_zeroed(
+                    &root_slot,
+                    ROOT_SLOT_BYTES,
+                    ROOT_CHECKSUM_OFFSET,
+                );
+                root_slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
+                sparse.write(offset, &root_slot);
+            }
+        }
+    }
+    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
+}
+
+/// `container` 那个单元换成 `after`（同宽）：写回两盘，新的整单元校验和沿引用链补上去。`after` 调用方已重封。
+pub fn replace_the_unit_and_propagate_its_checksum(
+    image: &mut MemoryPool,
+    container: SlotNumber,
+    after: &[u8],
+) {
+    let before = unit_bytes_at(image, container, after.len());
+    for device in image.devices.values_mut() {
+        device.write(container.to_device_offset(), after);
+    }
+    propagate_the_new_unit_checksum(
+        image,
+        container,
+        crc32_castagnoli(&before),
+        crc32_castagnoli(after),
+    );
+}
+
+/// `container` 那个单元里恰好一处是 `pointer_bytes`（一条指针的盘上原样）：把它头部的 `change` 那一段改掉、重封、写回两盘，
+/// 新的整单元校验和沿引用链补上去。
+pub fn change_a_pointer_head_field_inside_the_unit(
+    image: &mut MemoryPool,
+    container: SlotNumber,
+    container_bytes: usize,
+    pointer_bytes: &[u8],
+    change: PointerHeadFieldChange,
+) {
+    let before = unit_bytes_at(image, container, container_bytes);
+    let occurrences: Vec<usize> = (0..=before.len() - pointer_bytes.len())
+        .filter(|start| before[*start..*start + pointer_bytes.len()] == *pointer_bytes)
+        .collect();
+    assert_eq!(
+        occurrences.len(),
+        1,
+        "槽 {container:?} 那个单元里这条指针恰好出现一次"
+    );
+    let (offset_in_the_head, new_bytes) = change.offset_and_bytes();
+    let field_at = occurrences[0] + offset_in_the_head;
+    assert!(
+        before[field_at..field_at + new_bytes.len()]
+            .iter()
+            .all(|byte| *byte == 0),
+        "第一版写者在这一段写的是 0"
+    );
+    let mut after = before.clone();
+    after[field_at..field_at + new_bytes.len()].copy_from_slice(&new_bytes);
+    reseal_unit(&mut after);
+    replace_the_unit_and_propagate_its_checksum(image, container, &after);
+}
+
+pub fn node_pointer_bytes(pointer: &NodePointer) -> Vec<u8> {
+    let mut writer = ByteWriter::new(usize::try_from(NODE_POINTER_BYTES).expect("86"));
+    pointer.write_to(&mut writer);
+    writer.into_bytes()
+}
+
+pub fn data_pointer_bytes(pointer: &DataPointer) -> Vec<u8> {
+    let mut writer = ByteWriter::new(usize::try_from(DATA_POINTER_BYTES).expect("88"));
+    pointer.write_to(&mut writer);
+    writer.into_bytes()
+}
+
+pub fn the_slot_of(pointer: &NodePointer) -> SlotNumber {
+    assert_eq!(
+        pointer.locations[0].slot, pointer.locations[1].slot,
+        "第一版两盘同槽"
+    );
+    pointer.locations[0].slot
+}
+
+/// 择根择到的那条根（只读恢复与挂载择的是它）。
+pub fn newest_root(image: &MemoryPool) -> RootRecord {
+    let system_configuration = choose_system_configuration(image).expect("系统配置择得到");
+    choose_root(image, &system_configuration).expect("择得到根")
+}
+
+/// 最新那条根的树表里种类为 `kind` 的那条条目（树表单元按根记录里的树表指针读）。
+pub fn tree_table_entry_of_kind(image: &MemoryPool, kind: u16) -> TreeTableEntry {
+    let root = newest_root(image);
+    let tree_table = parse_index_node(&unit_bytes_at(
+        image,
+        the_slot_of(&root.tree_table),
+        node_bytes(),
+    ))
+    .expect("树表单元解得开");
+    tree_table
+        .entries
+        .iter()
+        .filter_map(|entry| TreeTableEntry::parse(entry))
+        .find(|entry| entry.kind == kind)
+        .expect("树表里有这种树")
+}
+
+/// 一个码 2 节点第 0 条条目。
+pub fn first_entry_of_the_node_at(image: &MemoryPool, pointer: &NodePointer) -> Vec<u8> {
+    parse_index_node(&unit_bytes_at(image, the_slot_of(pointer), node_bytes()))
+        .expect("节点解得开")
+        .entries
+        .first()
+        .expect("节点里至少一条条目")
+        .clone()
+}
+
+/// extent 树根（第一版只有一片上段叶：根兼叶）第 0 条上段叶条目。
+pub fn first_upper_leaf_entry(image: &MemoryPool) -> ExtentUpperLeafEntry {
+    let extent_root = tree_table_entry_of_kind(image, TREE_KIND_EXTENT).root;
+    ExtentUpperLeafEntry::parse(&first_entry_of_the_node_at(image, &extent_root))
+        .expect("上段叶条目解得开")
+}
+
+/// 只读挂载照常挂上，打开第一个文件被拒：交回打开那一步的错。
+pub fn open_the_first_file_refused(image: &MemoryPool) -> OpenFileFailure {
+    let mounted = match mount_read_only(image) {
+        Ok(mounted) => mounted,
+        Err(failure) => panic!("只读挂载那一步不读 extent 树，要挂得上：{failure:?}"),
+    };
+    match mounted
+        .mounted
+        .open_file(image, InodeNumber(FIRST_INODE_NUMBER))
+    {
+        Ok(_file) => panic!("extent 树里那条指针头部不是第一版的取值，打开文件却成功了"),
+        Err(failure) => failure,
+    }
+}
+
+/// 只读挂载、打开第一个文件、读第一个字节都成功：交回读到的字节数。
+pub fn first_byte_of_the_first_file_read(image: &MemoryPool) -> usize {
+    let mounted = match mount_read_only(image) {
+        Ok(mounted) => mounted,
+        Err(failure) => panic!("改之前只读挂载要挂得上：{failure:?}"),
+    };
+    let file = mounted
+        .mounted
+        .open_file(image, InodeNumber(FIRST_INODE_NUMBER))
+        .expect("改之前打开第一个文件");
+    file.read_at(image, FileOffsetInBytes(0), 1)
+        .expect("改之前读第一个字节")
+        .bytes
+        .len()
+}
+
+/// 冷启动恢复报的那个失败成员（恢复沿择到的根走读，走到被改的那条指针）。
+pub fn recovery_failure(image: &MemoryPool) -> RecoveryFailure {
+    match recover(image, JournalPolicy::Consult).outcome {
+        RecoveryOutcome::Failed { failure, .. } => failure,
+        RecoveryOutcome::FileRead { .. } | RecoveryOutcome::NoFile { .. } => {
+            panic!("被改的那条指针在冷启动走读的路上，恢复却成功了")
+        }
+    }
+}
+
+pub type WritableDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;
+
+pub fn writable_devices_of(image: &MemoryPool) -> WritableDevices {
+    image
+        .devices
+        .iter()
+        .map(|(identity, sparse)| {
+            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
+            device.image = sparse.clone();
+            (*identity, device)
+        })
+        .collect()
+}
+
+/// 可写挂载被拒、盘上逐字节不变：交回拒的那个错。
+pub fn writable_mount_refused_before_any_write(image: &MemoryPool) -> MountError {
+    let mut devices = writable_devices_of(image);
+    let refusal = match mount_writable(&parameters(), &mut devices) {
+        Ok(mounted) => panic!("坏镜像被可写挂载收下了：实例 {:?}", mounted.output.instance),
+        Err(refusal) => refusal,
+    };
+    for (identity, device) in &devices {
+        assert!(
+            device.image == image.devices[identity],
+            "可写挂载在任何写之前拒：盘 {identity:?} 逐字节不变（{refusal:?}）"
+        );
+    }
+    refusal
+}
+
+/// 新池新建文件之后再顺序写一版 `data_units` 个数据单元的文件（最后一个装一半）。
+pub fn image_after_a_sequential_write_of(tag: &str, data_units: usize) -> MemoryPool {
+    let mut pool = build_pool(tag);
+    let payload_capacity = data_unit_payload_capacity();
+    let content: Vec<u8> = (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
+        .map(|index| u8::try_from((index * 31 + 7) % 251).expect("小于 256"))
+        .collect();
+    sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
+    pool.memory_pool()
+}
+
+/// 一棵分配记录树从根往下认得出的节点指针（影子账认被抛弃根引用的单元时用的那个读者），节点按指针里第一条位置条目从盘 0 读。
+pub fn allocation_record_tree_nodes_as_far_as_readable(
+    image: &MemoryPool,
+    root_pointer: &NodePointer,
+) -> Vec<NodePointer> {
+    node_pointers_as_far_as_readable(
+        root_pointer,
+        &AllocationRecordTreeGeometry::of_reader(image),
+        &mut |pointer: &NodePointer| {
+            Some(unit_bytes_at(
+                image,
+                pointer.locations[0].slot,
+                node_bytes(),
+            ))
+        },
+    )
+}
+
+type RecordedSparseDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;
+
+/// 两块 4 GiB 内存盘上新池新建文件，再连着 369 次「取号之后崩溃」、可写挂载一次：给 [1, 371) 写 370 行——一片装不下，
+/// 实例表链两片（`instance_table_second_page_write.rs` 同一个造法）。
+pub fn image_whose_instance_table_has_two_pages() -> MemoryPool {
+    let stream = SharedStream::new();
+    let mut devices: RecordedSparseDevices = [DeviceIdentity(0), DeviceIdentity(1)]
+        .into_iter()
+        .map(|identity| {
+            (
+                identity,
+                RecordingBlockDevice::with_shared_stream(
+                    identity,
+                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
+                    stream.clone(),
+                ),
+            )
+        })
+        .collect();
+    run_new_pool_file_creation(&parameters(), &mut devices, &stream, |_, _| {})
+        .expect("新池新建文件");
+    {
+        let publish_parameters = parameters();
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        for _ in 0..INSTANCE_ROWS_PER_PAGE {
+            acquire_instance(&mut writer).expect("取号");
+        }
+    }
+    mount_writable(&parameters(), &mut devices).expect("370 行写成两片，可写挂载要成立");
+    memory_pool_of_sparse_devices(&devices)
+}
diff --git a/crates/singlefs-harness/tests/common_root_ring_holes/mod.rs b/crates/singlefs-harness/tests/common_root_ring_holes/mod.rs
new file mode 100644
index 0000000..f5226c5
--- /dev/null
+++ b/crates/singlefs-harness/tests/common_root_ring_holes/mod.rs
@@ -0,0 +1,248 @@
+//! 根环上有洞的池的搭建（S = 4 的 12 槽小环、1 MiB journal 环、两块 128 MiB 内存稀疏盘）：攻方腿原型
+//! `research/prompts/m3-root-ring-code-r1-opus-model/rr-attack-proto/src/lib.rs` 里 `build_plain`、`crash_a_mount_before_its_row_root`、
+//! `publish_empty` 那几样照同一个做法另写一份（D13（验证路线） 已定项 5：用例不与原型共用代码）。
+#![allow(dead_code, reason = "每个测试文件各自只用到其中一部分")]
+
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
+use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
+use singlefs_core::mount::{mount_writable, Mounted};
+use singlefs_core::recovery::{choose_system_configuration, readable_roots_with_ring_slots};
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::root_ring::RootRingSlotsPerRegion;
+use singlefs_core::system_configuration::SystemImmutableSizes;
+use singlefs_core::transaction::{
+    publish_first_file, publish_overwrite, publish_version, FirstFile, InstanceTablePlan,
+    PoolWriter, PublishError, PublishPlan, TransactionOutput,
+};
+use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
+use singlefs_harness::segments::{FixedGeometry, StepKind};
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+/// 这几份用例池的 fsid。
+pub const FILESYSTEM_IDENTIFIER_OF_THE_RING_HOLE_POOLS: [u8; 16] = *b"RING-HOLES-POOL1";
+/// 每块盘 128 MiB（攻方 `rf-*`、`rfo3-*` 那几组的 `PROTO_IMAGE_BYTES`）。
+pub const DEVICE_BYTES_OF_THE_RING_HOLE_POOLS: u64 = 128 << 20;
+/// journal 环 1 MiB（同那几组的 `PROTO_JOURNAL_BYTES`）。
+pub const JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS: u64 = 1 << 20;
+/// 每区槽数 S = 4：环长 12。
+pub const SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS: u64 = 4;
+/// 写入时间的起点。
+pub const WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS: u64 = 1_788_000_000;
+/// 一次覆盖写的内容长度：一个数据单元装得下。
+const CONTENT_BYTES: usize = 3000;
+
+pub type RingHoleDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;
+pub type RecordedRingHoleDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;
+
+#[must_use]
+pub fn parameters_of_the_ring_hole_pools() -> MakeFilesystemParameters {
+    MakeFilesystemParameters {
+        filesystem_identifier: FILESYSTEM_IDENTIFIER_OF_THE_RING_HOLE_POOLS,
+        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+        geometry: SystemImmutableSizes {
+            physical_block_size: 512,
+            minimum_input_output_bytes: 512,
+            fixed_structure_slot_spacing: 4096,
+            journal_ring_bytes: JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS,
+            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
+                SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
+            )
+            .expect("S = 4 在区间里"),
+        },
+    }
+}
+
+#[must_use]
+pub fn fixed_geometry_of_the_ring_hole_pools() -> FixedGeometry {
+    let parameters = parameters_of_the_ring_hole_pools();
+    FixedGeometry {
+        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
+        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
+        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
+    }
+}
+
+/// 第 `salt` 版的文件内容（3000 字节，各版不同）。
+#[must_use]
+pub fn content_of_the_version(salt: u64) -> Vec<u8> {
+    let salt = usize::try_from(salt % 251).expect("小于 251");
+    (0..CONTENT_BYTES)
+        .map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 256"))
+        .collect()
+}
+
+#[must_use]
+pub fn pool_of(devices: &RingHoleDevices) -> MemoryPool {
+    MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
+    }
+}
+
+#[must_use]
+pub fn devices_of(pool: &MemoryPool) -> RingHoleDevices {
+    pool.devices
+        .iter()
+        .map(|(identity, image)| {
+            let mut device = SparseBlockDevice::new(
+                DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
+                PhysicalBlockSizeInBytes(512),
+            );
+            device.image = image.clone();
+            (*identity, device)
+        })
+        .collect()
+}
+
+/// 这份盘面上的两块盘，外面包录制器、录进 `stream`。
+#[must_use]
+pub fn recording_of(pool: &MemoryPool, stream: &SharedStream) -> RecordedRingHoleDevices {
+    devices_of(pool)
+        .into_iter()
+        .map(|(identity, device)| {
+            (
+                identity,
+                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
+            )
+        })
+        .collect()
+}
+
+/// 带文件的一版上的空发布：文件不变，只换根与记账（准入、抬 F 那几串推的空发布同一个计划）。
+///
+/// # Errors
+/// 发布路径的错原样交回。
+pub fn publish_empty<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, PublishError> {
+    publish_version(
+        pool,
+        allocator,
+        PublishPlan {
+            txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
+            counter: previous.record.counter + 1,
+            transaction: 0,
+            highest_transaction_number_before_this_publish: previous
+                .highest_transaction_number_in_this_instance,
+            instance,
+            back_chain: singlefs_core::journal::back_chain_of(&previous.record_bytes),
+            file: None,
+            new_inode_records: &[],
+            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
+            tree_birth_txg: previous.tree_birth_txg(),
+            tree_identifier_watermark: previous.root.tree_identifier_watermark,
+            rollback_floor: previous.root.rollback_floor,
+        },
+        Some(previous),
+    )
+}
+
+/// mkfs → 可写挂载（实例 1）→ 第一个文件 → 覆盖写到 txg `last`，每次都完整落盘。
+#[must_use]
+pub fn pool_overwritten_through(last: u64) -> MemoryPool {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices: RingHoleDevices = (0..2u32)
+        .map(|number| {
+            (
+                DeviceIdentity(number),
+                SparseBlockDevice::new(
+                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
+                    PhysicalBlockSizeInBytes(512),
+                ),
+            )
+        })
+        .collect();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let Mounted {
+        output,
+        mut allocator,
+        current,
+    } = mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
+    let instance = output.instance;
+    let mut current: TransactionOutput = {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut allocator,
+            current.root(),
+            FirstFile {
+                content: &content_of_the_version(0),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
+            },
+            instance,
+            current.record_bytes(),
+        )
+        .expect("第一个文件")
+    };
+    // 迭代上界：txg 逐次加一，到 `last` 为止。
+    while current.root.checkpoint_txg.0 < last {
+        let salt = current.root.checkpoint_txg.0 + 1;
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        current = publish_overwrite(
+            &mut writer,
+            &mut allocator,
+            &current,
+            FirstFile {
+                content: &content_of_the_version(salt),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
+            },
+            instance,
+        )
+        .expect("覆盖写");
+    }
+    pool_of(&devices)
+}
+
+/// 在 `image` 上做一次可写挂载，崩在写行那次根槽写之前（写行那次的单元与记录已持久、根槽没持久，之后的写都没落）。
+/// 交回崩溃后的盘面与写行那次的 txg：那一槽里留着上一圈的根，环上多一个洞。
+#[must_use]
+pub fn crash_a_mount_before_its_row_root(image: &MemoryPool) -> (MemoryPool, u64) {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let stream = SharedStream::retaining_contents();
+    let mut recording = recording_of(image, &stream);
+    let mounted = mount_writable(&parameters, &mut recording).expect("被截的那次挂载");
+    let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
+    let (writes, _segments) = writes_and_segments(
+        &stream.retained_operations(),
+        &fixed_geometry_of_the_ring_hole_pools(),
+    );
+    let cut = writes
+        .iter()
+        .position(|write| {
+            write.kind == StepKind::RootRecordFua
+                && write
+                    .bytes()
+                    .and_then(|bytes| {
+                        RootRecord::parse_slot(bytes, &FILESYSTEM_IDENTIFIER_OF_THE_RING_HOLE_POOLS)
+                    })
+                    .is_some_and(|root| root.checkpoint_txg.0 == row_txg)
+        })
+        .expect("写行那次的根槽写");
+    let mut crashed = image.clone();
+    crashed.apply_writes(&writes[..cut]);
+    (crashed, row_txg)
+}
+
+/// 环里读得出的、txg 为 `checkpoint_txg` 的那条根。
+#[must_use]
+pub fn readable_root_at_txg(image: &MemoryPool, checkpoint_txg: u64) -> Option<RootRecord> {
+    let system_configuration = choose_system_configuration(image).ok()?;
+    let immutable = &system_configuration.immutable;
+    readable_roots_with_ring_slots(
+        image,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    )
+    .into_iter()
+    .map(|(_, root)| root)
+    .find(|root| root.checkpoint_txg.0 == checkpoint_txg)
+}
diff --git a/crates/singlefs-harness/tests/mounted_read_tells_a_narrow_inode_internal_entry_from_a_child_pointer_it_cannot_dereference.rs b/crates/singlefs-harness/tests/mounted_read_tells_a_narrow_inode_internal_entry_from_a_child_pointer_it_cannot_dereference.rs
new file mode 100644
index 0000000..8c77800
--- /dev/null
+++ b/crates/singlefs-harness/tests/mounted_read_tells_a_narrow_inode_internal_entry_from_a_child_pointer_it_cannot_dereference.rs
@@ -0,0 +1,105 @@
+//! 只读挂载打开池时 inode 树内部条目的两种拒法分成两个错误成员（`records/2026-09-30-里程碑三开工第一轮.md` 第 8 批 core 簇第 8 条）：
+//! 条目宽窄于字段表报 `OpenPoolForReadFailure::RecordMalformed`（这一版的读者不认这一版写的条目布局），子指针头部有一段不是第一版的取值报
+//! `OpenPoolForReadFailure::PointerInTheRecordCannotBeDereferenced`（条目布局认得、指针指不过去，判该指针所在的结构损坏：
+//! D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581（extent 偏移非 0 没有判据））。改之前两种都报 `RecordMalformed`。
+//!
+//! 坏镜像只改 inode 树根那一个节点：在新池新建文件那一版上重写它（条目宽、或第 0 条条目子指针的 MAC），重封、沿引用链补校验和。
+
+mod common;
+mod common_pointer_head_corruption;
+
+use common::{build_pool, parameters};
+use common_pointer_head_corruption::{
+    change_a_pointer_head_field_inside_the_unit, first_entry_of_the_node_at, node_bytes,
+    node_pointer_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of,
+    tree_table_entry_of_kind, unit_bytes_at, PointerHeadFieldChange,
+};
+use singlefs_core::mounted_read::{mount_read_only, MountReadOnlyFailure, OpenPoolForReadFailure};
+use singlefs_core::pointer::PointerHeadFieldOutsideTheFirstVersion;
+use singlefs_core::records::{parse_inode_internal_entry, TREE_KIND_INODE};
+use singlefs_core::unit::{build_index_node, parse_index_node};
+
+/// 场景：新池新建文件那一版，inode 树根按条目宽 119 重写（每条条目截掉最后一个字节：比字段表的 120 窄一个字节）。
+/// 预期：只读挂载报 `RecordMalformed { what: "inode 树内部条目" }`。
+#[test]
+fn a_narrow_inode_internal_entry_is_reported_as_a_malformed_record_by_the_read_only_mount() {
+    let mut image = build_pool("mounted-read-narrow-inode-internal-entry").memory_pool();
+    assert!(mount_read_only(&image).is_ok(), "改之前只读挂载挂得上");
+    let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
+    let node = parse_index_node(&unit_bytes_at(
+        &image,
+        the_slot_of(&inode_root),
+        node_bytes(),
+    ))
+    .expect("inode 树根解得开");
+    let narrower_entries: Vec<Vec<u8>> = node
+        .entries
+        .iter()
+        .map(|entry| entry[..entry.len() - 1].to_vec())
+        .collect();
+    let narrower_root = build_index_node(
+        node.tree,
+        node.level,
+        node.key_width,
+        &node.smallest_key,
+        &node.largest_key,
+        node.birth_txg,
+        &parameters().filesystem_identifier,
+        node.instance,
+        node.birth_sequence,
+        u16::try_from(node.entry_width - 1).expect("条目宽装得进两字节"),
+        &narrower_entries,
+    );
+    replace_the_unit_and_propagate_its_checksum(
+        &mut image,
+        the_slot_of(&inode_root),
+        &narrower_root,
+    );
+    let Err(refusal) = mount_read_only(&image) else {
+        panic!("条目宽窄于字段表，只读挂载却挂上了");
+    };
+    assert!(
+        matches!(
+            refusal,
+            MountReadOnlyFailure::Open(OpenPoolForReadFailure::RecordMalformed {
+                what: "inode 树内部条目"
+            })
+        ),
+        "只读挂载：条目布局不认，报 RecordMalformed，实际 {refusal:?}"
+    );
+}
+
+/// 场景：新池新建文件那一版，inode 树根第 0 条内部条目的子指针 MAC 第 0 字节改成 1。
+/// 预期：只读挂载报 `PointerInTheRecordCannotBeDereferenced { what: "inode 树内部条目的子指针", field: MacOrNonceNotZero }`。
+#[test]
+fn an_inode_internal_entry_whose_child_pointer_cannot_be_dereferenced_is_reported_apart_by_the_read_only_mount(
+) {
+    let mut image = build_pool("mounted-read-inode-internal-entry-child-pointer").memory_pool();
+    assert!(mount_read_only(&image).is_ok(), "改之前只读挂载挂得上");
+    let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
+    let (_separator_key, _identity, child) =
+        parse_inode_internal_entry(&first_entry_of_the_node_at(&image, &inode_root))
+            .expect("改之前 inode 内部条目解得开");
+    change_a_pointer_head_field_inside_the_unit(
+        &mut image,
+        the_slot_of(&inode_root),
+        node_bytes(),
+        &node_pointer_bytes(&child),
+        PointerHeadFieldChange::FirstMacByteToOne,
+    );
+    let Err(refusal) = mount_read_only(&image) else {
+        panic!("子指针 MAC 非 0，只读挂载却挂上了");
+    };
+    assert!(
+        matches!(
+            refusal,
+            MountReadOnlyFailure::Open(
+                OpenPoolForReadFailure::PointerInTheRecordCannotBeDereferenced {
+                    what: "inode 树内部条目的子指针",
+                    field: PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero,
+                }
+            )
+        ),
+        "只读挂载：条目布局认得、指针指不过去，实际 {refusal:?}"
+    );
+}
diff --git a/crates/singlefs-harness/tests/pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader.rs b/crates/singlefs-harness/tests/pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader.rs
new file mode 100644
index 0000000..63af027
--- /dev/null
+++ b/crates/singlefs-harness/tests/pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader.rs
@@ -0,0 +1,346 @@
+//! 指针头部里为加密与压缩留的那几段（MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2）第一版写定值，**读指针头部的每一个 core 走读读者**
+//! 遇到别的值都判该指针所在的结构损坏（D19（块指针的结构与宽度预算） 已定项 3「加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏」；
+//! D9（加密） 已定项 14「读者遇到不认识的码一律判该指针无法解引用（与算法类型那个字段同一条政策）」「码 0 时它恒 0，非 0 判该指针损坏」；
+//! `records/2026-09-30-里程碑三开工第一轮.md` 第 8 批 core 簇第 9 条）。多层码 2 树内部条目与 journal 新根段那两类读者早就判
+//! （`corrupt_on_disk_content_is_refused_instead_of_panicking.rs`）；根记录这一类还没换（`root_record.rs` 不在这一批）。
+//! 这里一类读者一条用例：树表条目的根指针、inode 树内部条目的子指针、extent 上段叶条目里内联的数据指针、extent 叶记录的数据指针、
+//! extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针；五种改法轮着用，每条坏镜像只改一处。
+//! 影子账认单元的那两个读者照认（多隔离、不漏隔离），在分配记录树与实例表链那两条里一并钉住。
+//! 改之前这些读者只判 extent 偏移，加密与压缩那几段跳过、照常读下去。
+
+mod common;
+mod common_pointer_head_corruption;
+
+use common::build_pool;
+use common_pointer_head_corruption::{
+    allocation_record_tree_nodes_as_far_as_readable, change_a_pointer_head_field_inside_the_unit,
+    data_pointer_bytes, data_unit_bytes, first_byte_of_the_first_file_read,
+    first_entry_of_the_node_at, first_upper_leaf_entry, image_after_a_sequential_write_of,
+    image_whose_instance_table_has_two_pages, newest_root, node_bytes, node_pointer_bytes,
+    open_the_first_file_refused, recovery_failure, the_slot_of, tree_table_entry_of_kind,
+    unit_bytes_at, writable_mount_refused_before_any_write, PointerHeadFieldChange,
+};
+use singlefs_core::allocation_record_tree::ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION;
+use singlefs_core::bytes::ByteReader;
+use singlefs_core::extent_tree::{
+    ExtentUpperLeafTarget,
+    EXTENT_LEAF_RECORD_DATA_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+    EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+    EXTENT_UPPER_LEAF_ENTRY_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+};
+use singlefs_core::instance_table::{
+    InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex,
+};
+use singlefs_core::mount::MountError;
+use singlefs_core::mounted_read::{
+    mount_read_only, MountReadOnlyFailure, OpenFileFailure, OpenPoolForReadFailure,
+};
+use singlefs_core::pointer::NodePointer;
+use singlefs_core::records::{
+    parse_extent_record, parse_inode_internal_entry, TREE_KIND_ALLOCATION, TREE_KIND_EXTENT,
+    TREE_KIND_INODE,
+};
+use singlefs_core::recovery::{
+    instance_table_chain_of_root, instance_table_page_pointers_as_far_as_readable, recover,
+    JournalPolicy, RecoveryFailure, RecoveryOutcome,
+    INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+};
+use singlefs_core::unit::parse_index_node;
+
+/// 场景：新池新建文件那一版，树表里 inode 树那条条目的根指针 MAC 第 0 字节改成 1（树表单元重封、根记录补校验和）。
+/// 预期：冷启动恢复与可写挂载都报「树表条目」解不开（`UnitMalformed`），可写挂载在任何写之前拒；只读挂载报树表条目解不开。
+#[test]
+fn a_tree_table_entry_whose_root_pointer_carries_a_non_zero_mac_is_refused() {
+    let mut image = build_pool("pointer-mac-tree-table-entry").memory_pool();
+    assert!(
+        matches!(
+            recover(&image, JournalPolicy::Consult).outcome,
+            RecoveryOutcome::FileRead { .. }
+        ),
+        "改之前冷启动读得出文件"
+    );
+    let tree_table = newest_root(&image).tree_table;
+    let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
+    change_a_pointer_head_field_inside_the_unit(
+        &mut image,
+        the_slot_of(&tree_table),
+        node_bytes(),
+        &node_pointer_bytes(&inode_root),
+        PointerHeadFieldChange::FirstMacByteToOne,
+    );
+    let tree_table_entry_malformed = RecoveryFailure::UnitMalformed {
+        what: "树表条目"
+    };
+    assert_eq!(
+        recovery_failure(&image),
+        tree_table_entry_malformed,
+        "冷启动恢复：树表条目解不开"
+    );
+    assert!(
+        matches!(
+            mount_read_only(&image),
+            Err(MountReadOnlyFailure::Open(
+                OpenPoolForReadFailure::TreeTableEntryMalformed
+            ))
+        ),
+        "只读挂载：打开挂载态时树表条目解不开"
+    );
+    let refusal = writable_mount_refused_before_any_write(&image);
+    assert!(
+        matches!(&refusal, MountError::Recovery(failure) if *failure == tree_table_entry_malformed),
+        "可写挂载：树表条目解不开，实际 {refusal:?}"
+    );
+}
+
+/// 场景：新池新建文件那一版，inode 树根第 0 条内部条目的子指针 nonce 第 0 字节改成 1。
+/// 预期：冷启动恢复报 `UnitMalformed`（[`INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]）。
+/// 只读挂载那一判在 `mounted_read_tells_a_narrow_inode_internal_entry_from_a_child_pointer_it_cannot_dereference.rs`。
+#[test]
+fn an_inode_internal_entry_whose_child_pointer_carries_a_non_zero_nonce_is_refused() {
+    let mut image = build_pool("pointer-nonce-inode-internal-entry").memory_pool();
+    assert_eq!(
+        first_byte_of_the_first_file_read(&image),
+        1,
+        "改之前读得出第一个字节"
+    );
+    let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
+    let (_separator_key, _identity, child) =
+        parse_inode_internal_entry(&first_entry_of_the_node_at(&image, &inode_root))
+            .expect("改之前 inode 内部条目解得开");
+    change_a_pointer_head_field_inside_the_unit(
+        &mut image,
+        the_slot_of(&inode_root),
+        node_bytes(),
+        &node_pointer_bytes(&child),
+        PointerHeadFieldChange::FirstNonceByteToOne,
+    );
+    assert_eq!(
+        recovery_failure(&image),
+        RecoveryFailure::UnitMalformed {
+            what: INODE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+        },
+        "冷启动恢复：inode 内部条目的子指针 nonce 非 0"
+    );
+}
+
+/// 场景：新池新建文件那一版（3000 字节，一个内联的数据单元），extent 树根里那条上段叶条目内联的数据指针算法类型改成 1。
+/// 预期：只读挂载照常挂上，打开文件报 extent 树走读失败（`UnitMalformed`，
+/// [`EXTENT_UPPER_LEAF_ENTRY_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]）；冷启动恢复报同一个成员。
+#[test]
+fn an_extent_upper_leaf_entry_whose_inline_data_pointer_names_an_encryption_algorithm_is_refused() {
+    let mut image = build_pool("pointer-algorithm-inline-data-pointer").memory_pool();
+    assert_eq!(
+        first_byte_of_the_first_file_read(&image),
+        1,
+        "改之前读得出第一个字节"
+    );
+    let ExtentUpperLeafTarget::InlineDataUnit(data_pointer) = first_upper_leaf_entry(&image).target
+    else {
+        panic!("3000 字节的文件是一个内联的数据单元（标签 2）");
+    };
+    let extent_root = tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
+    change_a_pointer_head_field_inside_the_unit(
+        &mut image,
+        the_slot_of(&extent_root),
+        node_bytes(),
+        &data_pointer_bytes(&data_pointer),
+        PointerHeadFieldChange::AlgorithmTypeToOne,
+    );
+    let upper_leaf_entry_refused = RecoveryFailure::UnitMalformed {
+        what: EXTENT_UPPER_LEAF_ENTRY_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+    };
+    assert_eq!(
+        open_the_first_file_refused(&image),
+        OpenFileFailure::ExtentTreeWalk(upper_leaf_entry_refused.clone()),
+        "打开文件：上段叶条目内联的数据指针算法类型不是未加密"
+    );
+    assert_eq!(
+        recovery_failure(&image),
+        upper_leaf_entry_refused,
+        "冷启动恢复：同一个成员"
+    );
+}
+
+/// 场景：新池新建文件之后顺序写成两个数据单元的文件，下段只有一片叶（根兼叶），它第 0 条叶记录的数据指针压缩算法码改成 1。
+/// 预期：打开文件与冷启动恢复都报 `UnitMalformed`（[`EXTENT_LEAF_RECORD_DATA_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]）。
+#[test]
+fn an_extent_leaf_record_whose_data_pointer_names_an_unregistered_compression_algorithm_is_refused()
+{
+    let mut image = image_after_a_sequential_write_of("pointer-compression-leaf-record", 2);
+    assert_eq!(
+        first_byte_of_the_first_file_read(&image),
+        1,
+        "改之前读得出第一个字节"
+    );
+    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(&image).target
+    else {
+        panic!("两个数据单元的文件在上段叶里是一条标签 1 的条目");
+    };
+    let (_key, data_pointer) =
+        parse_extent_record(&first_entry_of_the_node_at(&image, &lower_root))
+            .expect("改之前叶记录解得开");
+    change_a_pointer_head_field_inside_the_unit(
+        &mut image,
+        the_slot_of(&lower_root),
+        node_bytes(),
+        &data_pointer_bytes(&data_pointer),
+        PointerHeadFieldChange::CompressionAlgorithmCodeToOne,
+    );
+    let leaf_record_refused = RecoveryFailure::UnitMalformed {
+        what: EXTENT_LEAF_RECORD_DATA_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+    };
+    assert_eq!(
+        open_the_first_file_refused(&image),
+        OpenFileFailure::ExtentTreeWalk(leaf_record_refused.clone()),
+        "打开文件：叶记录的数据指针压缩算法码不认识"
+    );
+    assert_eq!(
+        recovery_failure(&image),
+        leaf_record_refused,
+        "冷启动恢复：同一个成员"
+    );
+}
+
+/// 下段两层的文件有几个数据单元：比一片下段叶装得下的 144 个多一个（D8（核心索引结构） 已定项 14「实现取值」）。
+const DATA_UNITS_OF_A_FILE_WITH_A_TWO_LEVEL_LOWER_SEGMENT: usize = 145;
+
+/// 场景：145 个数据单元的文件，下段根第 0 条内部条目的子指针压缩算法码仍 0、压后长度改成 1。
+/// 预期：打开文件与冷启动恢复都报 `UnitMalformed`（[`EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]）。
+#[test]
+fn an_extent_internal_entry_whose_child_pointer_carries_a_compressed_length_without_compression_is_refused(
+) {
+    let mut image = image_after_a_sequential_write_of(
+        "pointer-compressed-length-extent-internal-entry",
+        DATA_UNITS_OF_A_FILE_WITH_A_TWO_LEVEL_LOWER_SEGMENT,
+    );
+    assert_eq!(
+        first_byte_of_the_first_file_read(&image),
+        1,
+        "改之前读得出第一个字节"
+    );
+    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(&image).target
+    else {
+        panic!("145 个数据单元的文件在上段叶里是一条标签 1 的条目");
+    };
+    assert_eq!(
+        parse_index_node(&unit_bytes_at(
+            &image,
+            the_slot_of(&lower_root),
+            node_bytes()
+        ))
+        .expect("下段根解得开")
+        .level,
+        1,
+        "145 个单元：下段根在第 1 层"
+    );
+    let first_child = NodePointer::read_from(&mut ByteReader::at(
+        &first_entry_of_the_node_at(&image, &lower_root),
+        24,
+    ));
+    change_a_pointer_head_field_inside_the_unit(
+        &mut image,
+        the_slot_of(&lower_root),
+        node_bytes(),
+        &node_pointer_bytes(&first_child),
+        PointerHeadFieldChange::CompressedLengthToOneWithoutCompression,
+    );
+    let internal_entry_refused = RecoveryFailure::UnitMalformed {
+        what: EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION,
+    };
+    assert_eq!(
+        open_the_first_file_refused(&image),
+        OpenFileFailure::ExtentTreeWalk(internal_entry_refused.clone()),
+        "打开文件：下段内部条目的子指针码 0 而压后长度非 0"
+    );
+    assert_eq!(
+        recovery_failure(&image),
+        internal_entry_refused,
+        "冷启动恢复：同一个成员"
+    );
+}
+
+/// 场景：新池新建文件那一版，分配记录树根第 0 条内部条目的子指针 MAC 第 0 字节改成 1。
+/// 预期：可写挂载在任何写之前拒，报 `UnitMalformed`
+/// （[`ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION`]）；
+/// 影子账认单元的那个读者照认那条条目指的孩子（多隔离、不漏隔离）。
+#[test]
+fn an_allocation_record_tree_internal_entry_whose_child_pointer_carries_a_non_zero_mac_is_refused_by_the_walk(
+) {
+    let mut image = build_pool("pointer-mac-allocation-internal-entry").memory_pool();
+    let allocation_root = tree_table_entry_of_kind(&image, TREE_KIND_ALLOCATION).root;
+    let nodes_before = allocation_record_tree_nodes_as_far_as_readable(&image, &allocation_root);
+    let first_child = NodePointer::read_from(&mut ByteReader::at(
+        &first_entry_of_the_node_at(&image, &allocation_root),
+        10,
+    ));
+    assert!(
+        nodes_before.contains(&first_child),
+        "改之前影子账那个读者认得出第 0 条条目指的孩子"
+    );
+    change_a_pointer_head_field_inside_the_unit(
+        &mut image,
+        the_slot_of(&allocation_root),
+        node_bytes(),
+        &node_pointer_bytes(&first_child),
+        PointerHeadFieldChange::FirstMacByteToOne,
+    );
+    let refusal = writable_mount_refused_before_any_write(&image);
+    assert!(
+        matches!(
+            &refusal,
+            MountError::Recovery(RecoveryFailure::UnitMalformed { what })
+                if *what == ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_ENCRYPTION_OR_COMPRESSION_FIELD_OUTSIDE_THE_FIRST_VERSION
+        ),
+        "可写挂载：分配记录树内部条目的子指针 MAC 非 0，实际 {refusal:?}"
+    );
+    let allocation_root_after = tree_table_entry_of_kind(&image, TREE_KIND_ALLOCATION).root;
+    let nodes_after =
+        allocation_record_tree_nodes_as_far_as_readable(&image, &allocation_root_after);
+    assert_eq!(
+        nodes_after.len(),
+        nodes_before.len(),
+        "影子账那个读者照认那条条目指的孩子：改前改后认得出的节点一样多"
+    );
+}
+
+/// 场景：实例表链两片的那一版，第 0 片末尾那条链指针记录里下一片指针的算法类型改成 1。
+/// 预期：沿链读整张实例表报这一片解不开（`UnitMalformed`）；可写挂载在任何写之前拒（`InstanceTableMalformed`）；
+/// 影子账认链上各片的那个读者照认两片。
+#[test]
+fn an_instance_table_chain_pointer_naming_an_encryption_algorithm_is_refused_by_the_walk() {
+    let mut image = image_whose_instance_table_has_two_pages();
+    let root = newest_root(&image);
+    let first_page = unit_bytes_at(&image, the_slot_of(&root.instance_table), data_unit_bytes());
+    let InstanceTableChainRecord::NextPage(second_page) =
+        InstanceTablePage::parse(&first_page, InstanceTablePageIndex::FIRST)
+            .expect("改之前第 0 片解得开")
+            .chain
+    else {
+        panic!("第 0 片的链指针记录是「有下一片」");
+    };
+    change_a_pointer_head_field_inside_the_unit(
+        &mut image,
+        the_slot_of(&root.instance_table),
+        data_unit_bytes(),
+        &node_pointer_bytes(&second_page),
+        PointerHeadFieldChange::AlgorithmTypeToOne,
+    );
+    let root_after = newest_root(&image);
+    assert!(
+        matches!(
+            instance_table_chain_of_root(&image, &root_after),
+            Err(RecoveryFailure::UnitMalformed { .. })
+        ),
+        "沿链读：第 0 片的链指针记录解不开"
+    );
+    assert_eq!(
+        instance_table_page_pointers_as_far_as_readable(&image, &root_after).len(),
+        2,
+        "影子账那个读者照认两片"
+    );
+    let refusal = writable_mount_refused_before_any_write(&image);
+    assert!(
+        matches!(&refusal, MountError::InstanceTableMalformed),
+        "可写挂载：所选根的实例表沿链有一片解不开，实际 {refusal:?}"
+    );
+}
diff --git a/crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs b/crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs
index 7691235..a66a9ab 100644
--- a/crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs
+++ b/crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs
@@ -3,8 +3,10 @@
 //! C581（extent 偏移非 0 没有判据））。根记录、多层码 2 树内部条目与 journal 新根段那三类读者另有用例
 //! （`pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs`、`corrupt_on_disk_content_is_refused_instead_of_panicking.rs`）；
 //! 这里一类读者一条用例：树表条目的根指针、inode 树内部条目的子指针、extent 上段叶条目里内联的数据指针与下段根指针、
-//! extent 叶记录的数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针（连同影子账认单元的那个读者）、实例表链指针
-//! （连同影子账认链上各片的那个读者）。
+//! extent 叶记录的数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针。
+//! 影子账认单元的那两个读者（分配记录树的 `node_pointers_as_far_as_readable`、实例表链的 `instance_table_page_pointers_as_far_as_readable`）
+//! 遇到 extent 偏移非 0 照旧认它指的孩子：影子账要的是多隔离、不漏隔离（主 agent 2026-10-03 判，`records/2026-09-30-里程碑三开工第一轮.md`
+//! 第 8 批 core 簇第 7 条），在分配记录树与实例表链那两条里一并钉住。
 //!
 //! 坏镜像只改一处：在发布路径写出的一版上找到那条指针所在的单元，把那条指针头部偏移 32 的 extent 偏移由 0 改成 1、重封这个单元，
 //! 新的整单元校验和沿引用链补上去（父单元里的位置条目、中央映射条目、根槽，`propagate_the_new_unit_checksum`），
@@ -42,7 +44,10 @@ use singlefs_core::mount::{mount_writable, MountError};
 use singlefs_core::mounted_read::{
     mount_read_only, MountReadOnlyFailure, OpenFileFailure, OpenPoolForReadFailure,
 };
-use singlefs_core::pointer::{DataPointer, NodePointer};
+use singlefs_core::pointer::{
+    DataPointer, NodePointer, PointerHeadExtentOffsetNotZero,
+    PointerHeadFieldOutsideTheFirstVersion,
+};
 use singlefs_core::records::{
     parse_extent_record, parse_inode_internal_entry, TreeTableEntry, TREE_KIND_ALLOCATION,
     TREE_KIND_EXTENT, TREE_KIND_INODE,
@@ -493,7 +498,8 @@ fn a_tree_table_entry_whose_root_pointer_carries_a_non_zero_extent_offset_is_ref
 }
 
 /// 场景：新池新建文件那一版，inode 树根第 0 条内部条目的子指针头部 extent 偏移改成 1（inode 树根重封，树表、中央映射、根记录补校验和）。
-/// 预期：冷启动恢复报 `UnitMalformed`（[`INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO`]），只读挂载报 inode 树内部条目解不开。
+/// 预期：冷启动恢复报 `UnitMalformed`（[`INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO`]），只读挂载报 inode 树内部条目的子指针
+/// 不能解引用（`PointerInTheRecordCannotBeDereferenced`，与条目宽窄于字段表的 `RecordMalformed` 分开报）。
 /// 改之前两个读者都跳过这 2 字节、照常读出文件。
 #[test]
 fn an_inode_internal_entry_whose_child_pointer_carries_a_non_zero_extent_offset_is_refused() {
@@ -524,12 +530,15 @@ fn an_inode_internal_entry_whose_child_pointer_carries_a_non_zero_extent_offset_
         matches!(
             mount_read_only(&image),
             Err(MountReadOnlyFailure::Open(
-                OpenPoolForReadFailure::RecordMalformed {
-                    what: "inode 树内部条目"
+                OpenPoolForReadFailure::PointerInTheRecordCannotBeDereferenced {
+                    what: "inode 树内部条目的子指针",
+                    field: PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(
+                        PointerHeadExtentOffsetNotZero { extent_offset: 1 }
+                    ),
                 }
             ))
         ),
-        "只读挂载：inode 树内部条目解不开"
+        "只读挂载：inode 树内部条目的子指针不能解引用"
     );
 }
 
@@ -722,7 +731,7 @@ fn allocation_record_tree_nodes_as_far_as_readable(
 /// 场景：新池新建文件那一版，分配记录树根（层级由池几何定，第一版两块 4 GiB 盘上在第 1 层以上）第 0 条内部条目的子指针头部
 /// extent 偏移改成 1。
 /// 预期：可写挂载在任何写之前拒，报 `UnitMalformed`（[`ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO`]）；
-/// 影子账认单元的那个读者把这条条目当解不开，不认它指的孩子（认得出的节点比改之前少）。改之前可写挂载照常挂上。
+/// 影子账认单元的那个读者照旧认它指的孩子（认得出的节点与改之前一样多，那个孩子在其中）。改之前可写挂载照常挂上。
 #[test]
 fn an_allocation_record_tree_internal_entry_whose_child_pointer_carries_a_non_zero_extent_offset_is_refused(
 ) {
@@ -765,11 +774,11 @@ fn an_allocation_record_tree_internal_entry_whose_child_pointer_carries_a_non_ze
     let nodes_after =
         allocation_record_tree_nodes_as_far_as_readable(&image, &allocation_root_after);
     assert!(
-        nodes_after.len() < nodes_before.len()
-            && !nodes_after
+        nodes_after.len() == nodes_before.len()
+            && nodes_after
                 .iter()
                 .any(|pointer| pointer.locations[0].slot == first_child.locations[0].slot),
-        "影子账那个读者不认那条条目指的孩子：改之前 {} 个节点，改之后 {} 个",
+        "影子账那个读者照认那条条目指的孩子（多隔离、不漏隔离）：改之前 {} 个节点，改之后 {} 个",
         nodes_before.len(),
         nodes_after.len()
     );
@@ -808,7 +817,7 @@ fn image_whose_instance_table_has_two_pages() -> MemoryPool {
 }
 
 /// 场景：实例表链两片的那一版，第 0 片末尾那条链指针记录里下一片指针的头部 extent 偏移改成 1（第 0 片重封，引用它的根记录补校验和）。
-/// 预期：沿链读整张实例表报这一片解不开（`UnitMalformed`）；影子账认链上各片的那个读者只认得出第 0 片；
+/// 预期：沿链读整张实例表报这一片解不开（`UnitMalformed`）；影子账认链上各片的那个读者照认两片（多隔离、不漏隔离）；
 /// 可写挂载在任何写之前拒（`InstanceTableMalformed`）。改之前链上两片都认得出、整张表读得出。
 #[test]
 fn an_instance_table_chain_pointer_carrying_a_non_zero_extent_offset_is_refused() {
@@ -842,9 +851,9 @@ fn an_instance_table_chain_pointer_carrying_a_non_zero_extent_offset_is_refused(
         "沿链读：第 0 片的链指针记录解不开"
     );
     assert_eq!(
-        instance_table_page_pointers_as_far_as_readable(&image, &root_after),
-        vec![root_after.instance_table],
-        "影子账那个读者只认得出第 0 片"
+        instance_table_page_pointers_as_far_as_readable(&image, &root_after).len(),
+        2,
+        "影子账那个读者照认两片：第 1 片那条指针的 extent 偏移非 0 也照它的位置条目认"
     );
     let refusal = writable_mount_refused_before_any_write(&image);
     assert!(
diff --git a/crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs b/crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs
new file mode 100644
index 0000000..4d263a4
--- /dev/null
+++ b/crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs
@@ -0,0 +1,250 @@
+//! 准入抬 F 那一遍在一圈窗口最旧端有洞的根环上（D16（发布语义） 已定项 1；`research/prompts/m3-root-ring-code-r1-main-verification.md`
+//! 第二节 K2-b、K2-b2 两格，攻方 `raise_floor_hidden_slot.rs`）。
+//!
+//! 盘面（S = 4 的 12 槽环）：实例 1 覆盖写到 txg `last`；之后几次可写挂载都崩在写行那次根槽写之前（写行那次的单元与记录已持久、根槽没持久：
+//! 那一槽里留着上一圈的根，出了窗口），每次一个洞；再一次可写挂载写行那一版的根 X 落盘、接着两次空发布，正常退出。
+//! 第四次可写挂载之后推空发布到锚 = 第一个洞 + 3S − 1（洞成了窗口最旧的几格），准入抬 F。
+//!
+//! - 上限只取窗口里的有效根（B2，用户 2026-10-03 定）：洞那几槽里上一圈的旧根在窗口外，窗口已许可它们引用的单元被复用；
+//!   改之前上限要读它们的树表、读到被复用的槽，准入抬 F 被拒 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（攻方 `rf-14-none.out`）。
+//! - 回收门槛把挂载时读不出的那一槽并进窗口里最旧有效根（RR4，攻方 B1）：X 那一槽第四次挂载起一直读不出，根环表按它能住的最大 txg 记着；
+//!   改之前抬 F 那一遍只在读得出的根上取门槛，门槛越过 X、回收了 X 引用的槽，记账比遍历少（攻方 `rfo3-17-persist.out`：抬完 I-3.1 红）。
+mod common;
+mod common_root_ring_holes;
+
+use common::{
+    with_unreadable_ranges, DeviceWithUnreadableRanges, FailingReadsOfARange,
+    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
+};
+use common_root_ring_holes::{
+    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
+    parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, publish_empty,
+    readable_root_at_txg, recording_of, SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
+    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::mount::{
+    mount_writable, raise_rollback_floor, raise_rollback_floor_to_the_admission_ceiling,
+    MountError, RaiseToTheAdmissionCeiling, ShadowLedger,
+};
+use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）、F 不高于上限（I-7.9）。
+const INVARIANTS_THE_RAISE_KEEPS_GREEN: [&str; 6] =
+    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8", "I-7.9"];
+/// 回收门槛那一条把 F 抬到的值：攻方 `rfo3-17-persist.out` 那一组准入抬 F 抬到的 6（改 B2 之前那份盘面上的上限），低于 X 的 txg 21——
+/// 门槛由「窗口里最旧有效根」那一半定，读不出的 X 算不算进来分得出两种取法。（抬到 B2 之后的上限 22 时 F 高于 X、门槛由 F 定，分不出。）
+const FLOOR_RAISED_BELOW_X: CheckpointTxg = CheckpointTxg(6);
+/// X 那一版之后、正常退出之前的空发布次数（攻方 `AFTER_X` 的默认值）。
+const EMPTY_PUBLISHES_AFTER_X: u64 = 2;
+
+type DevicesOfTheFourthMount = Vec<(
+    DeviceIdentity,
+    DeviceWithUnreadableRanges<RecordingBlockDevice<SparseBlockDevice>>,
+)>;
+
+/// 准入抬 F 之前的那一刻：第四次可写挂载的盘（X 那一槽按 `x_slot_reads` 读坏）、录下的流与它的起点盘面、分配器、现行那一版。
+struct PoolJustBeforeTheAdmissionRaise {
+    holes: Vec<u64>,
+    x_txg: u64,
+    image_before_the_fourth_mount: MemoryPool,
+    stream: SharedStream,
+    devices: DevicesOfTheFourthMount,
+    allocator: PoolAllocator,
+    current: TransactionOutput,
+    instance: InstanceGeneration,
+}
+
+impl PoolJustBeforeTheAdmissionRaise {
+    /// 盘上此刻的样子：起点盘面加上第四次挂载以来录下的全部写（读故障只在读那一侧）。
+    fn image_now(&self) -> MemoryPool {
+        let mut image = self.image_before_the_fourth_mount.clone();
+        image.apply(&self.stream.retained_operations());
+        image
+    }
+}
+
+fn violations_of(image: &MemoryPool, invariants_judged: &[&str]) -> Vec<String> {
+    check_pool_image(image)
+        .into_iter()
+        .filter(|(invariant, _)| invariants_judged.contains(invariant))
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+        })
+        .collect()
+}
+
+/// 造盘面到准入抬 F 之前：实例 1 覆盖写到 `last`，`hole_count` 次挂载崩在写行根槽之前，第三次（洞之后那一次）挂载写下 X 再空发布两次，
+/// 第四次挂载（X 那一槽按 `x_slot_reads` 读坏）之后空发布推到锚。
+fn pool_just_before_the_admission_raise(
+    last: u64,
+    hole_count: usize,
+    x_slot_reads: FailingReadsOfARange,
+) -> PoolJustBeforeTheAdmissionRaise {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let slots_per_region = parameters.geometry.root_ring_slots_per_region;
+    let ring_length = 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
+    let mut image = pool_overwritten_through(last);
+    let mut holes = Vec::new();
+    // 迭代上界：`hole_count` 次挂载，每次一个洞；跨轮携带的是崩溃之后的盘面。
+    for _ in 0..hole_count {
+        let (crashed, row_txg) = crash_a_mount_before_its_row_root(&image);
+        holes.push(row_txg);
+        image = crashed;
+    }
+    let x_txg = {
+        let mut devices = devices_of(&image);
+        let mounted = mount_writable(&parameters, &mut devices).expect("洞之后那一次可写挂载");
+        let x_txg = mounted.output.row_publish.root().checkpoint_txg.0;
+        let instance = mounted.output.instance;
+        let mut allocator = mounted.allocator.clone();
+        let mut current = mounted.current.into_file_version().expect("这一版带文件");
+        // 迭代上界：两次空发布。
+        for _ in 0..EMPTY_PUBLISHES_AFTER_X {
+            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+            current =
+                publish_empty(&mut writer, &mut allocator, &current, instance).expect("空发布");
+        }
+        image = pool_of(&devices);
+        x_txg
+    };
+    assert!(
+        readable_root_at_txg(&image, x_txg).is_some(),
+        "X 那一版的根落盘了"
+    );
+    let x_slot = target_for_publish(CheckpointTxg(x_txg), slots_per_region);
+    let unreadable_x_slot = SharedUnreadableRanges::new(
+        vec![UnreadableRange {
+            device: parameters.region_devices[usize::try_from(x_slot.region).expect("区域号")],
+            offset_in_bytes: slot_offset(x_slot, parameters.geometry.fixed_structure_slot_spacing)
+                .0,
+            length_in_bytes: u64::from(parameters.geometry.physical_block_size),
+            failing_reads: x_slot_reads,
+        }],
+        UnreadableRangeReadBack::DeviceError,
+    );
+    let stream = SharedStream::retaining_contents();
+    let mut devices = with_unreadable_ranges(recording_of(&image, &stream), &unreadable_x_slot);
+    let mounted = mount_writable(&parameters, &mut devices).expect("第四次可写挂载");
+    let instance = mounted.output.instance;
+    let mut allocator = mounted.allocator.clone();
+    let mut current = mounted.current.into_file_version().expect("这一版带文件");
+    let anchor = holes[0] + ring_length - 1;
+    // 迭代上界：txg 逐次加一推到锚。
+    while current.root.checkpoint_txg.0 < anchor {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        current = publish_empty(&mut writer, &mut allocator, &current, instance).expect("空发布");
+    }
+    PoolJustBeforeTheAdmissionRaise {
+        holes,
+        x_txg,
+        image_before_the_fourth_mount: image,
+        stream,
+        devices,
+        allocator,
+        current,
+        instance,
+    }
+}
+
+/// 场景：实例 1 覆盖写到 txg 14，两次挂载崩在写行根槽之前（洞 15、16），X = 17；第四次挂载没有读故障，推到锚 26，准入抬 F。
+/// 预期：抬 F 做成（不拒 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`），抬完盘上那几条不变量不红。
+/// 改之前上限读洞里窗口外 txg 3 那条旧根的树表，那个槽已被复用、读不出，准入抬 F 被拒（攻方 `rf-14-none.out`）。
+#[test]
+fn admission_raising_the_floor_above_holes_at_the_oldest_end_of_the_window_takes_its_ceiling_only_from_roots_inside_the_window(
+) {
+    let mut pool = pool_just_before_the_admission_raise(14, 2, FailingReadsOfARange::Never);
+    assert_eq!(
+        (pool.holes.clone(), pool.x_txg),
+        (vec![15, 16], 17),
+        "两个洞紧跟着 X"
+    );
+    let raised = raise_rollback_floor_to_the_admission_ceiling(
+        &parameters_of_the_ring_hole_pools(),
+        &mut pool.devices,
+        &mut pool.allocator,
+        &mut pool.current,
+        ShadowLedger::On,
+    );
+    assert!(
+        matches!(
+            raised,
+            Ok(RaiseToTheAdmissionCeiling::Raised(_)
+                | RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { .. })
+        ),
+        "上限只取窗口里的有效根，不读窗口外旧根的树表：{raised:?}"
+    );
+    assert!(
+        !matches!(
+            raised,
+            Err(MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. })
+        ),
+        "没有读故障的合法历史上准入抬 F 不因窗口外旧根的树表被拒"
+    );
+    assert_eq!(
+        violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN),
+        Vec::<String>::new(),
+        "抬完之后"
+    );
+}
+
+/// 场景：实例 1 覆盖写到 txg 17，三次挂载崩在写行根槽之前（洞 18、19、20），X = 21；第四次挂载起 X 那一槽一直读不出，推到锚 29，
+/// 准入抬 F 到 [`FLOOR_RAISED_BELOW_X`]，再覆盖写三次。
+/// 预期：抬 F 做成，抬完与之后每次覆盖写之后盘上那几条不变量不红：X 那一槽按根环表记的最大 txg 并进回收门槛，门槛不越过 X。
+/// 改之前抬完 I-3.1 红（攻方 `rfo3-17-persist.out`：记账的已分配比遍历全部有效根少）。
+#[test]
+fn admission_raising_the_floor_counts_the_root_ring_slot_unreadable_since_the_mount_into_the_reclaim_threshold(
+) {
+    let mut pool = pool_just_before_the_admission_raise(17, 3, FailingReadsOfARange::Every);
+    assert_eq!(
+        (pool.holes.clone(), pool.x_txg),
+        (vec![18, 19, 20], 21),
+        "三个洞紧跟着 X"
+    );
+    let parameters = parameters_of_the_ring_hole_pools();
+    let raised = raise_rollback_floor(
+        &parameters,
+        &mut pool.devices,
+        &mut pool.allocator,
+        &mut pool.current,
+        FLOOR_RAISED_BELOW_X,
+        ShadowLedger::On,
+    );
+    assert!(raised.is_ok(), "准入抬 F 做成：{raised:?}");
+    assert_eq!(
+        violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN),
+        Vec::<String>::new(),
+        "抬完之后：门槛不越过 X 那一槽"
+    );
+    // 迭代上界：三次覆盖写；跨轮携带的是现行那一版。
+    for step in 0..3u64 {
+        let salt = 5000 + step;
+        pool.current = {
+            let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
+            publish_overwrite(
+                &mut writer,
+                &mut pool.allocator,
+                &pool.current,
+                FirstFile {
+                    content: &content_of_the_version(salt),
+                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
+                },
+                pool.instance,
+            )
+            .expect("覆盖写")
+        };
+        assert_eq!(
+            violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN),
+            Vec::<String>::new(),
+            "抬完之后第 {step} 次覆盖写（txg {}）",
+            pool.current.root.checkpoint_txg.0
+        );
+    }
+}
diff --git a/crates/singlefs-harness/tests/rollback_target_outside_the_one_revolution_window_is_not_a_candidate.rs b/crates/singlefs-harness/tests/rollback_target_outside_the_one_revolution_window_is_not_a_candidate.rs
new file mode 100644
index 0000000..5d4bd43
--- /dev/null
+++ b/crates/singlefs-harness/tests/rollback_target_outside_the_one_revolution_window_is_not_a_candidate.rs
@@ -0,0 +1,139 @@
+//! 回退候选集只收 txg 落在一圈窗口里的根（D16（发布语义） 已定项 1「回退候选集」：按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ txg 落在一圈窗口里；
+//! D23（journal 的角色与格式） 已定项 14 的挂着时回退取同一个集合；用户 2026-10-03 定照 7-2 报告第四节做）。
+//!
+//! 盘面同 `one_revolution_window_after_a_hole_in_the_root_ring.rs`：实例 1 在 24 槽的根环上发到 txg 30，txg 31 那次发布崩在「记录已持久、根槽没持久」，
+//! 那一槽里留着上一圈的 txg 7 的根；可写挂载（实例 2）恢复由记录施加出 txg 31，写行 32、暖机 33、34。txg 7 的根读得出、按实例表有效、
+//! txg ≥ F_生效（F 没抬过，是 0）、带文件，只有一样不满足：现行那一版 txg 34 的窗口是 txg > 10，txg 7 在窗口外——窗口已许可它引用的单元被复用。
+mod common;
+
+use common::{
+    build_pool, crash_state_devices, disk_snapshot, memory_pool_of_sparse_devices,
+    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
+    system_configuration_slots_of_every_device, write_back_system_configuration_slots,
+};
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::block_device::BlockDevice;
+use singlefs_core::mount::{
+    mount_writable, roll_back_by_a_forward_publish, RollbackCandidateExclusion, RollbackError,
+    RollbackTarget,
+};
+use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_core::transaction::PoolVersion;
+use singlefs_harness::SharedStream;
+
+/// 根槽没落的那次发布的 txg：它的槽里留着上一圈 txg 7 的根（24 槽的环，31 − 24 = 7）。
+const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 31;
+/// 洞那一槽里留着的上一圈的根：回退的目标。
+const TXG_OF_THE_STALE_ROOT_IN_THE_HOLE: u64 = 7;
+/// 一次覆盖写的内容长度：一个数据单元装得下。
+const OVERWRITE_BYTES: usize = 3000;
+
+fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
+    vec![u8::try_from(checkpoint_txg % 251).expect("小于 256"); OVERWRITE_BYTES]
+}
+
+/// 场景：上面那份盘面上挂着时回退到环里那条 txg 7 的根（实例 1）。
+/// 预期：在任何写之前拒，报 `TargetNotACandidate { exclusion: OutsideTheOneRevolutionWindow }`；盘上逐字节不变（系统配置槽、根环里自证过的根、录制流步数）。
+/// 改之前候选集不看窗口，txg 7 那条根被当成候选往下走。
+#[test]
+fn rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_is_refused_before_any_write(
+) {
+    let mut pool = build_pool("rollback-target-outside-the-one-revolution-window");
+    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
+    // 迭代上界：txg 4 到 30，27 次覆盖写。
+    for checkpoint_txg in 4..TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST {
+        overwrite_in_process_as_instance_one_and_advance_the_pool(
+            &mut pool,
+            &content_of_txg(checkpoint_txg),
+        );
+    }
+    let target = target_for_publish(
+        CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
+        slots_per_region,
+    );
+    assert_eq!(
+        target,
+        target_for_publish(
+            CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
+            slots_per_region
+        ),
+        "根环 24 槽：txg 31 与 txg 7 同槽"
+    );
+    let device_of_the_slot =
+        parameters().region_devices[usize::try_from(target.region).expect("区域号")];
+    let offset_of_the_slot =
+        slot_offset(target, parameters().geometry.fixed_structure_slot_spacing);
+    let mut stale_root_slot =
+        vec![0u8; usize::try_from(parameters().geometry.physical_block_size).expect("根槽宽")];
+    {
+        let devices = pool.devices.as_ref().expect("镜像开着");
+        let (_, device) = devices
+            .iter()
+            .find(|(identity, _)| *identity == device_of_the_slot)
+            .expect("池里有这块盘");
+        device
+            .read_at(offset_of_the_slot, &mut stale_root_slot)
+            .expect("读那个根槽");
+    }
+    let system_configuration_before = system_configuration_slots_of_every_device(&pool);
+    overwrite_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &content_of_txg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
+    );
+    let mut crash_image = pool.memory_pool();
+    crash_image
+        .devices
+        .get_mut(&device_of_the_slot)
+        .expect("有这块盘")
+        .write(offset_of_the_slot, &stale_root_slot);
+    write_back_system_configuration_slots(&mut crash_image, &system_configuration_before);
+
+    let stream = SharedStream::retaining_contents();
+    let mut devices = crash_state_devices(&crash_image, &[], &[], &stream);
+    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
+    let PoolVersion::WithFile(mut current) = mounted.current else {
+        panic!("txg 31 那一版带文件");
+    };
+    assert_eq!(
+        (mounted.output.instance, current.root.checkpoint_txg),
+        (InstanceGeneration(2), CheckpointTxg(34)),
+        "写行 txg 32、暖机 txg 33、34"
+    );
+    let before = disk_snapshot(&memory_pool_of_sparse_devices(&devices), &stream);
+    assert!(
+        before.readable_roots.iter().any(|root| {
+            (root.instance, root.checkpoint_txg)
+                == (
+                    InstanceGeneration(1),
+                    CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
+                )
+        }),
+        "txg 7 的根还在环里、读得出"
+    );
+    let refusal = roll_back_by_a_forward_publish(
+        &parameters(),
+        &mut devices,
+        &mut mounted.allocator,
+        &mut current,
+        RollbackTarget {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
+        },
+    )
+    .expect_err("txg 7 在现行那一版的一圈窗口外，不是候选");
+    assert!(
+        matches!(
+            refusal,
+            RollbackError::TargetNotACandidate {
+                exclusion: RollbackCandidateExclusion::OutsideTheOneRevolutionWindow,
+                ..
+            }
+        ),
+        "报落在一圈窗口外：{refusal:?}"
+    );
+    assert_eq!(
+        disk_snapshot(&memory_pool_of_sparse_devices(&devices), &stream),
+        before,
+        "在任何写之前拒：盘上逐字节不变"
+    );
+}
diff --git a/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs b/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs
new file mode 100644
index 0000000..4d54adb
--- /dev/null
+++ b/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs
@@ -0,0 +1,194 @@
+//! 挂载那一刻读不出的根环槽，进程内成了一圈窗口里最小的那个时先重读（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定；
+//! `research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表二之后那一段，攻方 K2-e `unreadable_slot_i31.rs` 的
+//! `u31-s4-9-mount.out` 那一形搬到 mkfs 默认的 24 槽环上）。
+//!
+//! 盘面：实例 1 发到 txg 23，txg 24 那次发布崩在「记录已持久、根槽与之后的系统配置轮换没持久」：txg 24 落到的区域 0 槽 0 里留着 mkfs 写的
+//! txg 0 的根。可写挂载（实例 2）那一刻区域 0 槽 0 读不出（设备报读错），挂载之后读得出。挂载按记录施加出 txg 24 那一版，根环表把那一槽记成
+//! 「挂载时读不出、能住的最大 txg 是 24」。
+//!
+//! - 照最大 txg 算（改之前）：写 txg 47 那一版时窗口是 txg > 23，txg 25–47 是这个进程写的有效根，那一槽按 24 算、成了最小的，门槛停在 24；
+//!   池级 checker 读盘，那一槽里是窗口外的 txg 0，窗口里最旧的有效根是 25 ⇒ 记账比遍历多算释放代 25 的那几个槽，I-3.1 红。
+//! - E2：记 txg 47 那条根之前那一槽成了窗口里最小的那个，先重读：读出 txg 0 的根（按实例表有效、在窗口外）⇒ 那一槽当没有，门槛取 25，与 checker 对得上。
+//!   txg 46 时窗口里还有 txg 23 的根，比 24 旧，不重读，那一槽仍按 24 记着。
+mod common;
+
+use common::{
+    build_pool, crash_state_devices, overwrite_in_process_as_instance_one_and_advance_the_pool,
+    parameters, system_configuration_slots_of_every_device, unreadable_root_slot_of,
+    with_unreadable_ranges, write_back_system_configuration_slots, FailingReadsOfARange,
+    SharedUnreadableRanges, UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::block_device::BlockDevice;
+use singlefs_core::mount::mount_writable;
+use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
+use singlefs_harness::memory_pool::MemoryPool;
+use singlefs_harness::SharedStream;
+
+/// 根槽没落的那次发布的 txg：它落到的区域 0 槽 0 里留着 mkfs 写的 txg 0 的根（24 槽的环）。
+const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 24;
+/// 那一槽在窗口里成了最小的那个的那一版：24 + 3S − 1。
+const TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW: u64 = 47;
+/// 一次覆盖写的内容长度：一个数据单元装得下。
+const OVERWRITE_BYTES: usize = 3000;
+/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
+const INVARIANTS_THE_REREAD_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
+
+fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
+    vec![u8::try_from(checkpoint_txg % 251).expect("小于 256"); OVERWRITE_BYTES]
+}
+
+fn violations_of(image: &MemoryPool) -> Vec<String> {
+    check_pool_image(image)
+        .into_iter()
+        .filter(|(invariant, _)| INVARIANTS_THE_REREAD_KEEPS_GREEN.contains(invariant))
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+        })
+        .collect()
+}
+
+#[test]
+fn a_root_ring_slot_unreadable_only_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window_and_the_allocated_statistic_holds(
+) {
+    let mut pool = build_pool("root-ring-slot-unreadable-at-mount-reread");
+    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
+    assert_eq!(
+        3 * slots_per_region.count(),
+        TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST,
+        "根环 24 槽：txg 24 与 mkfs 的 txg 0 同槽"
+    );
+    // 迭代上界：txg 4 到 23，20 次覆盖写。
+    for checkpoint_txg in 4..TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST {
+        let output = overwrite_in_process_as_instance_one_and_advance_the_pool(
+            &mut pool,
+            &content_of_txg(checkpoint_txg),
+        );
+        assert_eq!(output.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
+    }
+    let hidden_slot = target_for_publish(
+        CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
+        slots_per_region,
+    );
+    let device_of_the_slot =
+        parameters().region_devices[usize::try_from(hidden_slot.region).expect("区域号")];
+    let offset_of_the_slot = slot_offset(
+        hidden_slot,
+        parameters().geometry.fixed_structure_slot_spacing,
+    );
+    let mut stale_root_slot =
+        vec![0u8; usize::try_from(parameters().geometry.physical_block_size).expect("根槽宽")];
+    {
+        let devices = pool.devices.as_ref().expect("镜像开着");
+        let (_, device) = devices
+            .iter()
+            .find(|(identity, _)| *identity == device_of_the_slot)
+            .expect("池里有这块盘");
+        device
+            .read_at(offset_of_the_slot, &mut stale_root_slot)
+            .expect("读那个根槽");
+    }
+    let system_configuration_before = system_configuration_slots_of_every_device(&pool);
+    overwrite_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &content_of_txg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
+    );
+    // 崩溃状态：txg 24 的单元与记录都落了，根槽与之后的系统配置轮换没落（D16（发布语义） 已定项 7 的持久顺序）。
+    let mut crash_image = pool.memory_pool();
+    crash_image
+        .devices
+        .get_mut(&device_of_the_slot)
+        .expect("有这块盘")
+        .write(offset_of_the_slot, &stale_root_slot);
+    write_back_system_configuration_slots(&mut crash_image, &system_configuration_before);
+
+    let stream = SharedStream::retaining_contents();
+    let unreadable_at_mount = SharedUnreadableRanges::new(
+        vec![unreadable_root_slot_of(
+            CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
+            FailingReadsOfARange::Every,
+        )],
+        UnreadableRangeReadBack::DeviceError,
+    );
+    let mut devices = with_unreadable_ranges(
+        crash_state_devices(&crash_image, &[], &[], &stream),
+        &unreadable_at_mount,
+    );
+    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
+    // 只在挂载那一次读错：挂载之后那一槽读得出。
+    unreadable_at_mount.lift();
+    assert!(
+        unreadable_at_mount.failed_reads() > 0,
+        "挂载那一次读那一槽读错了"
+    );
+    assert_eq!(
+        (
+            mounted.output.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        (InstanceGeneration(2), CheckpointTxg(24)),
+        "恢复由记录施加出 txg 24"
+    );
+    let unreadable_slots_of_the_table =
+        |mounted_allocator: &singlefs_core::allocator::PoolAllocator| {
+            mounted_allocator
+                .root_ring_occupancy()
+                .expect("可写挂载装了根环表")
+                .ring_slots_unreadable_at_mount()
+        };
+    assert_eq!(
+        unreadable_slots_of_the_table(&mounted.allocator).get(&hidden_slot),
+        Some(&CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST)),
+        "根环表把那一槽记成挂载时读不出、能住的最大 txg 24"
+    );
+    let PoolVersion::WithFile(mut current) = mounted.current else {
+        panic!("txg 24 那一版带文件");
+    };
+    let publish_parameters = parameters();
+    // 迭代上界：覆盖写到 txg 48（那一槽被这个进程写掉的那一版）；跨轮携带的是现行那一版。
+    while current.root.checkpoint_txg.0 < TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW + 1
+    {
+        let checkpoint_txg = current.root.checkpoint_txg.0 + 1;
+        current = {
+            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+            publish_overwrite(
+                &mut writer,
+                &mut mounted.allocator,
+                &current,
+                FirstFile {
+                    content: &content_of_txg(checkpoint_txg),
+                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + checkpoint_txg,
+                },
+                InstanceGeneration(2),
+            )
+            .expect("覆盖写")
+        };
+        assert_eq!(current.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
+        if checkpoint_txg == TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW - 1 {
+            assert_eq!(
+                unreadable_slots_of_the_table(&mounted.allocator).get(&hidden_slot),
+                Some(&CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST)),
+                "txg 46：窗口里还有比 24 旧的 txg 23，那一槽没重读、仍按 24 记着"
+            );
+        }
+        if checkpoint_txg == TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW {
+            assert_eq!(
+                unreadable_slots_of_the_table(&mounted.allocator).get(&hidden_slot),
+                None,
+                "txg 47：那一槽成了窗口里最小的那个，重读出 txg 0 的根，不再按 24 记"
+            );
+        }
+        // 盘上此刻的样子：崩溃状态加上挂载以来录下的全部写（读故障只在读那一侧，写原样落到里面那块盘上）。
+        let mut image = crash_image.clone();
+        image.apply(&stream.retained_operations());
+        assert_eq!(
+            violations_of(&image),
+            Vec::<String>::new(),
+            "txg {checkpoint_txg} 之后：那一槽重读之后按读出来的根算，回收门槛与 checker 的窗口对得上"
+        );
+    }
+}
diff --git a/crates/singlefs-harness/tests/unmount_marker_of_a_root_rebuilt_from_the_records_of_the_unmount_sequence.rs b/crates/singlefs-harness/tests/unmount_marker_of_a_root_rebuilt_from_the_records_of_the_unmount_sequence.rs
new file mode 100644
index 0000000..a73cfda
--- /dev/null
+++ b/crates/singlefs-harness/tests/unmount_marker_of_a_root_rebuilt_from_the_records_of_the_unmount_sequence.rs
@@ -0,0 +1,80 @@
+//! 由记录重建那次发布的根时，卸载记号从记录的新根段来、不照抄所选根（D23（journal 的角色与格式） 已定项 15：施加一条记录 = 把所选根的七个字段换成
+//! 记录里的，卸载记号是其中一项；`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表一 RR2，正推腿 H2 的补洞草稿）。
+//!
+//! 盘面：新池新建文件之后覆盖写三版（txg 4、5、6），正常卸载推两次带卸载记号的空发布（txg 7、8），两次的根槽都撕掉、两条记录留着。
+//! 择根落到 txg 6（不带卸载记号），施加 txg 7、8 两条记录：重建出来的根与 txg 8 当初写的逐字段相同，带卸载记号。
+//! 只撕 txg 8 一槽时所选根 txg 7 本身就带记号，照抄所选根也对，分不出两种取法；两槽都撕才分得出。
+mod common;
+
+use common::{
+    build_pool, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
+    tear_the_root_slot_written_for_txg, CONTENT_BYTES_OF_PUBLISH_B,
+};
+use singlefs_core::address::CheckpointTxg;
+use singlefs_core::mount::{unmount, ShadowLedger, Unmounted};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::root_record::UnmountMarker;
+use singlefs_core::transaction::PoolVersion;
+
+/// 场景：上面那份盘面只读挂载。
+/// 预期：所选根是 txg 6、不带卸载记号；恢复施加了两条记录，生效的根与卸载那一串第二次空发布当初写的根逐字段相同（带卸载记号）。
+#[test]
+fn a_root_rebuilt_from_both_records_of_the_unmount_sequence_takes_the_unmount_marker_from_the_record(
+) {
+    let mut pool = build_pool("unmount-marker-of-a-root-rebuilt-from-the-records");
+    // 迭代上界：三次覆盖写（txg 4、5、6）。
+    for fill_byte in [0xB0u8, 0xC0, 0xD0] {
+        overwrite_in_process_as_instance_one_and_advance_the_pool(
+            &mut pool,
+            &vec![fill_byte; CONTENT_BYTES_OF_PUBLISH_B],
+        );
+    }
+    let mut current = PoolVersion::WithFile(pool.output.clone());
+    let unmounted = unmount(
+        &parameters(),
+        pool.devices.as_mut().expect("镜像开着"),
+        &mut pool.allocator,
+        &mut current,
+        ShadowLedger::On,
+    )
+    .expect("正常卸载");
+    let Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
+        panic!("现行那一版带文件，卸载推抬 F 的空发布");
+    };
+    let [first_unmount_publish, second_unmount_publish] =
+        <[_; 2]>::try_from(raised.publishes).expect("txg 6 除以 3 余 0：卸载推两次空发布");
+    assert_eq!(
+        (
+            first_unmount_publish.root.checkpoint_txg,
+            second_unmount_publish.root.checkpoint_txg
+        ),
+        (CheckpointTxg(7), CheckpointTxg(8))
+    );
+    assert_eq!(
+        second_unmount_publish.root.unmount_marker,
+        UnmountMarker::WrittenByTheUnmountSequence
+    );
+    let mut image = pool.memory_pool();
+    tear_the_root_slot_written_for_txg(&mut image, 7);
+    tear_the_root_slot_written_for_txg(&mut image, 8);
+    let mounted = mount_read_only(&image).expect("只读挂载");
+    assert_eq!(
+        (
+            mounted.chosen_root.checkpoint_txg,
+            mounted.chosen_root.unmount_marker
+        ),
+        (
+            CheckpointTxg(6),
+            UnmountMarker::NotWrittenByTheUnmountSequence
+        ),
+        "所选根是 txg 6，不带卸载记号"
+    );
+    assert_eq!(
+        mounted.journal.prefix_applied, 2,
+        "施加了卸载那一串的两条记录"
+    );
+    assert_eq!(
+        mounted.effective_root, second_unmount_publish.root,
+        "重建出来的根与 txg 8 当初写的逐字段相同：卸载记号从记录的新根段来"
+    );
+}
````

## 8-2 checker 簇（`/tmp/claude-1000/m3-r1/batch8-checker/patch/crates.patch`，报告同目录 `report.md`；变异 `mutations-append.tsv`）

````diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 42642a1..e2b5b66 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -69,7 +69,8 @@ pub const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY: &str =
 /// 第 50 条起是里程碑三二·1 那一批（两份判决 `research/prompts/m3-inv-a-r1-main-verification.md`、
 /// `research/prompts/m3-inv-b-r1-main-verification.md`，用户 2026-10-01 定案），**只往末尾追加**：层 0 的违例位按下标 i ↔ 这张表第 i 条，
 /// 前 49 条的下标不动。位宽改到 ≥ 128 之前，下标 64 起的那一条在层 0 的 GPU 判器上没有位（另一个实现员在 checker 档改）。
-pub const IMPLEMENTED_INVARIANTS: [&str; 65] = [
+/// 第 66 条（下标 65）是 I-3.5（里程碑三第 8 批）：GPU 判器内核还没判它（`crash_judge_gpu` 的覆盖表另列）。
+pub const IMPLEMENTED_INVARIANTS: [&str; 66] = [
     "I-1.1",
     "I-1.2",
     "I-1.3",
@@ -135,12 +136,14 @@ pub const IMPLEMENTED_INVARIANTS: [&str; 65] = [
     JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
     ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
     SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
+    REFERENCE_INTERVALS_ARE_EXACT,
 ];
 
 /// I-1.12：走读跟随的一条指针，偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；码 2 取哪个字段随 C289 定，
 /// 不判（inode 内部节点指着叶容器的那一类由 I-9.2 罩住，不重判）。
 pub const POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER: &str = "I-1.12";
-/// I-8.10：带末条标志、同 (实例代号, checkpoint_txg) 只此一条的记录，新根段四段与根环里同 (实例代号, checkpoint_txg) 的根逐字节相同。
+/// I-8.10：带末条标志、同 (实例代号, checkpoint_txg) 只此一条的记录，新根段七段与根环里同 (实例代号, checkpoint_txg) 的根逐字节相同
+/// （树表指针、映射根指针、树 ID 水位、F 四段，加实例表指针、分配记录树根指针、卸载记号三段：用户 2026-10-03 定扩到七段）。
 pub const LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH: &str = "I-8.10";
 /// I-8.11：点名项的一条位置条目所指那一份整单元校验和相符时，单元头的类标签、诞生代号、key 尾段（与码 1 / 码 3 的出生树）与点名项逐字段相同。
 pub const NAMED_ENTRY_MATCHES_THE_UNIT_HEADER: &str = "I-8.11";
@@ -174,6 +177,11 @@ pub const ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH: &str = "I-7.5";
 /// I-8.5（V-池）：全池至少一块盘有一个自证过的系统配置槽，全池皆废判红。单盘全废、哪块盘少于两槽自证过（可写挂载会拒）
 /// 不进违例通道，走旁路报告 [`system_configuration_redundancy_report`]。
 pub const SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION: &str = "I-8.5";
+/// I-3.5（引用区间的精确性）的出现规则：回退候选集按 txg 升序、同 txg 并成一条，相邻两版 (R⁻, R)，R 引用而 R⁻ 不引用的每个单元
+/// 诞生代号大于 R⁻.txg，或落在 R 的一棵整棵继承下来的用户可见树里（按树继承下界）；候选集有洞时按实例表行判接不接得上，
+/// 接不上的那一对只判用户可见树之外的单元（`research/prompts/m3-birth-floor-r3-main-verification.md`，用户 2026-10-03 定丙）。
+/// ③ ④（deadlist）这一版不判。
+pub const REFERENCE_INTERVALS_ARE_EXACT: &str = "I-3.5";
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、整条不适用的理由，以及射程里有一部分判不了的理由。
 #[derive(Default)]
@@ -553,7 +561,8 @@ pub const fn value_refused_by_this_reader(verdict: &crate::Verdict) -> bool {
         | crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval
         | crate::Verdict::EncryptionReservedBytesNotZero
         | crate::Verdict::EntryWidthZeroWithEntries
-        | crate::Verdict::RecordWidthZeroWithRecords => false,
+        | crate::Verdict::RecordWidthZeroWithRecords
+        | crate::Verdict::UnmountMarkerBitsOtherThanBitZero => false,
     }
 }
 
@@ -588,7 +597,8 @@ pub const fn verdict_says_the_slot_does_not_verify_itself(verdict: &crate::Verdi
         | crate::Verdict::RootRingSlotsPerRegionOutsideTheFormatInterval
         | crate::Verdict::EncryptionReservedBytesNotZero
         | crate::Verdict::EntryWidthZeroWithEntries
-        | crate::Verdict::RecordWidthZeroWithRecords => false,
+        | crate::Verdict::RecordWidthZeroWithRecords
+        | crate::Verdict::UnmountMarkerBitsOtherThanBitZero => false,
     }
 }
 
diff --git a/crates/singlefs-checker/src/lib.rs b/crates/singlefs-checker/src/lib.rs
index 3ea25df..aa2a39e 100644
--- a/crates/singlefs-checker/src/lib.rs
+++ b/crates/singlefs-checker/src/lib.rs
@@ -162,6 +162,10 @@ pub enum Verdict {
     /// 系统配置偏移 417 自述的单元区起始槽号不落在聚簇段边界上（不是 `CLUSTER_SEGMENT_SLOTS` 的整数倍，D3（空间分配） 已定项 10 ①）：
     /// 这样的单元区怎么分段条款没写，第一版不支持。
     UnitAreaStartOffTheClusterSegmentBoundary,
+    /// journal 记录新根段末尾卸载记号那 4 字节（记录偏移 459，D23（journal 的角色与格式） 已定项 15）位 0 之外有位为 1：
+    /// 与根记录 flags 同一个编码，读者与恢复把这条记录当损坏（`crates/singlefs-core/src/root_record.rs` 的
+    /// `of_unmount_marker_field_of_a_new_root_segment` 交 `None`），checker 照同一口径不收它（m3-root-ring-code-r1 判决 RR1）。
+    UnmountMarkerBitsOtherThanBitZero,
 }
 
 /// 系统配置槽解出来的几个要紧字段。
@@ -707,6 +711,10 @@ const JOURNAL_BACK_CHAIN_OFFSET: usize = 91;
 const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 95;
 const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: usize = 99;
 const JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET: usize = 463;
+/// 新根段里卸载记号那 4 字节：新根段 99 起，前六段 86 + 86 + 8 + 8 + 86 + 86 = 360 之后（D23（journal 的角色与格式） 已定项 15）。
+const JOURNAL_UNMOUNT_MARKER_OFFSET: usize = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 360;
+/// 卸载记号只许位 0（正常卸载那一串写 1），编码与根记录 flags 位 0 相同（D22（单元原子性怎么合成） 已定项 7）。
+const JOURNAL_UNMOUNT_MARKER_BIT: u32 = 1;
 
 /// 点名项解出来的样子（D23（journal 的角色与格式） 已定项 17）。
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -766,7 +774,8 @@ pub fn back_chain_of_record_header(record: &[u8]) -> u32 {
     crc32_castagnoli_bitwise(&header)
 }
 
-/// 判一条 journal 记录：magic、`header_csum` 罩整条 4096（D23（journal 的角色与格式） 已定项 13）、类型、记录长度、fsid、载荷 CRC、点名项 flags。
+/// 判一条 journal 记录：magic、`header_csum` 罩整条 4096（D23（journal 的角色与格式） 已定项 13）、类型、记录长度、fsid、
+/// 卸载记号位 0 之外全 0、载荷 CRC、点名项 flags。
 pub fn check_journal_record(
     record: &[u8],
     expected_filesystem_identifier: u64,
@@ -790,6 +799,9 @@ pub fn check_journal_record(
     if read_u64(record, JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET) != expected_filesystem_identifier {
         return Err(Verdict::FilesystemIdentifierMismatch);
     }
+    if read_u32(record, JOURNAL_UNMOUNT_MARKER_OFFSET) & !JOURNAL_UNMOUNT_MARKER_BIT != 0 {
+        return Err(Verdict::UnmountMarkerBitsOtherThanBitZero);
+    }
     let named_count = usize::try_from(read_u32(record, 12)).expect("点名项数");
     let header_bytes = usize::try_from(JOURNAL_HEADER_BYTES).expect("487");
     let entry_bytes = usize::try_from(JOURNAL_NAMED_ENTRY_BYTES).expect("56");
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 2f3b455..996bef7 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -42,7 +42,8 @@ use crate::image::{
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
     LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH, MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
     NAMED_ENTRY_MATCHES_THE_UNIT_HEADER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
-    PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
+    PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, REFERENCE_INTERVALS_ARE_EXACT,
+    TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
     UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
 };
 use crate::{
@@ -355,6 +356,250 @@ struct Walk<'reader> {
     birth_txg_of_walked_units: BTreeMap<(u32, u64), u64>,
     /// I-4.9 在不是最新的候选根上判过几个被引用单元：一个都没有（候选集只剩最新那条根）时整条报不适用。
     units_judged_against_an_older_candidate_root: u64,
+    /// 跟着指针记下的每个落点 (设备, 槽) → 它是哪个单元（那条指针第 0 条位置条目的落点，与 `visited_units`、`birth_txg_of_walked_units`
+    /// 同一个键）：I-3.5 比相邻两版引用了哪些单元时，两份副本的落点折成一个单元。
+    unit_of_each_placement: BTreeMap<(u32, u64), (u32, u64)>,
+    /// 跟着指针记下的落点里，记的那一刻走读正在一棵用户可见树（extent、inode、`deleted_inodes`）里往下走的那些 → 那棵树
+    /// （I-3.5 (b) 认「落在 R 树表里某棵用户可见树的子树里」按它）。别的版本共享这棵子树时不重走，照这张表接上。
+    user_visible_tree_of_each_placement: BTreeMap<(u32, u64), UserVisibleTreeOfTheWalk>,
+    /// 走读此刻在哪棵用户可见树里往下走；不在任何一棵里是 `None`。
+    user_visible_tree_being_walked: Option<UserVisibleTreeOfTheWalk>,
+    /// 正在走的这一版树表里每棵根指针非全零的用户可见树，带根指针的出生 txg（I-3.5 (b) 拿它比 R⁻.txg）。
+    user_visible_tree_roots_of_this_version: Vec<UserVisibleTreeRootOfAVersion>,
+    /// 走过的树表单元 (设备, 槽) → 它每一条条目里 I-9.25 与 I-3.5 读的几样：别的版本共享这片树表时照这份判，不重读盘。
+    tree_table_entries_of_a_walked_tree_table: BTreeMap<(u32, u64), Vec<TreeTableEntryOfTheWalk>>,
+    /// 走过的实例表各片（按第 0 条位置条目的落点）→ 头里的诞生代号：I-3.5 要它。不进 `birth_txg_of_walked_units`——
+    /// 那张表 I-4.9 也读，而实例表单元按 I-1.2 那一行的例外不进 I-4.9。
+    birth_txg_of_walked_instance_table_pages: BTreeMap<(u32, u64), u64>,
+    /// 已经走完的每一版（根环里的候选根与由记录施加出来的那几版）：I-3.5 在全部走完之后逐对比。
+    versions_walked_for_the_appearance_rule: Vec<VersionReferencesForTheAppearanceRule>,
+}
+
+/// 用户可见树的种类码：回退取回退目标那一版的几棵（extent、inode、`deleted_inodes`，`crates/singlefs-core/src/mount.rs`
+/// 的 `version_to_publish_the_rollback_after`；D23（journal 的角色与格式） 已定项 14）。I-3.5 (b) 只认这几棵里的单元。
+const USER_VISIBLE_TREE_KINDS: [u16; 3] =
+    [TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_DELETED_INODES];
+
+/// 一棵用户可见树：种类与树 ID。I-3.5 按它给落点打标签。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+struct UserVisibleTreeOfTheWalk {
+    kind: u16,
+    tree: u64,
+}
+
+/// 一版树表里一棵根指针非全零的用户可见树，带根指针头里的出生 txg。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct UserVisibleTreeRootOfAVersion {
+    tree: UserVisibleTreeOfTheWalk,
+    root_pointer_birth_txg: u64,
+}
+
+/// 树表的一条条目里 I-9.25 与 I-3.5 读的几样（D8（核心索引结构） 已定项 8 的字段表：树 ID 0、种类 10、根指针 14..100、
+/// `previous_snapshot_txg` 100）。走读第一次读到这片树表时记下。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct TreeTableEntryOfTheWalk {
+    tree: u64,
+    kind: u16,
+    previous_snapshot_txg: u64,
+    /// 种类是用户可见树、根指针非全零时是那棵树的根；别的条目 `None`。
+    user_visible_root: Option<UserVisibleTreeRootOfAVersion>,
+}
+
+/// 一条树表条目按字段表切出来的几样；条目比字段表窄（走读守卫那一格，`walk_tree_table_entry` 记走读失败）时 `None`。
+fn tree_table_entry_of_the_walk(entry: &[u8]) -> Option<TreeTableEntryOfTheWalk> {
+    if entry.len() < tree_table_entry_bytes() {
+        return None;
+    }
+    let tree = read_u64(entry, 0);
+    let kind = read_u16(entry, 10);
+    let root_pointer = parse_node_pointer(&entry[14..100]);
+    let user_visible_root = (USER_VISIBLE_TREE_KINDS.contains(&kind) && !root_pointer.all_zero)
+        .then_some(UserVisibleTreeRootOfAVersion {
+            tree: UserVisibleTreeOfTheWalk { kind, tree },
+            root_pointer_birth_txg: root_pointer.birth_txg,
+        });
+    Some(TreeTableEntryOfTheWalk {
+        tree,
+        kind,
+        previous_snapshot_txg: read_u64(entry, TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET),
+        user_visible_root,
+    })
+}
+
+/// 走完的一版在 I-3.5 眼里的样子：它是哪一版、引用了哪些单元（各落在哪棵用户可见树里）、树表里几棵用户可见树的根、走读断没断。
+struct VersionReferencesForTheAppearanceRule {
+    instance: u32,
+    checkpoint_txg: u64,
+    /// 这一版引用的单元（落点折成第 0 条位置条目的落点）→ 它落在哪棵用户可见树里（不在任何一棵里是 `None`）。
+    units: BTreeMap<(u32, u64), Option<UserVisibleTreeOfTheWalk>>,
+    user_visible_tree_roots: Vec<UserVisibleTreeRootOfAVersion>,
+    /// 这一版走读一条走读失败都没记：记了就是引用集合数不全，它参与的每一对报不适用。
+    references_are_complete: bool,
+}
+
+/// 回退候选集里相邻两版之间有没有洞（I-3.5 的缺口条款；I-3.1 在短环上洞里的误红归下一批调查，那一处要接同一份判法，
+/// `research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 5 行）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum HoleInTheCandidateSet {
+    /// R.txg = R⁻.txg + 1。
+    NoHole,
+    /// 有洞，但最新根的实例表行证明 R 所在的实例直接接在 R⁻ 那一版上（gap-row）：这一对照常判。
+    HoleBridgedByTheInstanceTableRows,
+    /// 有洞、实例表行接不上：用户可见树子树里的单元不判（N3），别的照 (a) 判。
+    HoleNotBridgedByTheInstanceTableRows,
+}
+
+/// 相邻两版 (R⁻, R) 之间的洞：R⁻ 那一侧是同 txg 并成的一条（几个实例都算 R⁻ 的实例）。接得上 ⟺ 对 R⁻ 的某个实例 i：
+/// 最新根的实例表里有一行 (i, T = R⁻.txg)，R 的实例 v > i，且 i 与 v 之间的每个实例 j 都有一行、都是 (j, 0, 0)。
+/// 迭代上界是实例表的行数（不按实例代号逐个数，代号是盘上读来的 4 字节）。
+fn hole_in_the_candidate_set_between(
+    previous_txg: u64,
+    previous_instances: &BTreeSet<u32>,
+    version_txg: u64,
+    version_instance: u32,
+    instance_table_rows_of_the_newest_root: &[InstanceTableRow],
+) -> HoleInTheCandidateSet {
+    if previous_txg.checked_add(1) == Some(version_txg) {
+        return HoleInTheCandidateSet::NoHole;
+    }
+    let bridged_from = |previous_instance: u32| {
+        if version_instance <= previous_instance {
+            return false;
+        }
+        let row_of_the_previous_version =
+            instance_table_rows_of_the_newest_root.iter().any(|row| {
+                row.instance == previous_instance && row.published_checkpoint_txg == previous_txg
+            });
+        let rows_between: Vec<&InstanceTableRow> = instance_table_rows_of_the_newest_root
+            .iter()
+            .filter(|row| row.instance > previous_instance && row.instance < version_instance)
+            .collect();
+        let instances_between: BTreeSet<u32> =
+            rows_between.iter().map(|row| row.instance).collect();
+        let every_instance_between_has_a_row = u64::try_from(instances_between.len())
+            .expect("实例表行数装得进 u64")
+            == u64::from(version_instance - previous_instance - 1);
+        let every_row_between_published_nothing = rows_between.iter().all(|row| {
+            row.published_checkpoint_txg == 0 && row.applied_transaction_high_water_mark == 0
+        });
+        row_of_the_previous_version
+            && every_instance_between_has_a_row
+            && every_row_between_published_nothing
+    };
+    if previous_instances
+        .iter()
+        .any(|previous_instance| bridged_from(*previous_instance))
+    {
+        return HoleInTheCandidateSet::HoleBridgedByTheInstanceTableRows;
+    }
+    HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows
+}
+
+/// I-3.5（引用区间的精确性） 的出现规则（TF + β + gap + gap-row + N3，`research/prompts/m3-birth-floor-r3-main-verification.md`、
+/// 用户 2026-10-03 定丙；措辞是 `.claude/kb/invariants.md` I-3.5 那一行）：回退候选集（走过的每一版，与 I-7.4 同一份）按 txg 升序，
+/// 同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 逐对判。R 引用而 R⁻ 不引用的每个单元 b
+/// （第一次出现与再次出现都判）满足 (a) birth(b) > R⁻.txg；或 (b) b 落在 R 树表里一棵用户可见树的子树里，那棵树的根指针出生 txg
+/// 严格小于 R⁻.txg，且 R 里那棵子树每个读得出的单元头的诞生代号都不大于那个出生 txg。birth(b) 取单元头里的诞生代号，头读不出的不判。
+/// 有洞（[`hole_in_the_candidate_set_between`]）而实例表行接不上时，用户可见树子树里的单元不判（N3）。R 或 R⁻ 走读记过失败的那一对不判；
+/// 一个单元都没判到时整条报不适用。
+fn judge_the_appearance_rule_of_adjacent_candidate_versions(
+    versions: &[VersionReferencesForTheAppearanceRule],
+    birth_txg_of_walked_units: &BTreeMap<(u32, u64), u64>,
+    birth_txg_of_walked_instance_table_pages: &BTreeMap<(u32, u64), u64>,
+    instance_table_rows_of_the_newest_root: &[InstanceTableRow],
+    judgements: &mut Judgements,
+) {
+    let birth_txg_of = |unit: &(u32, u64)| {
+        birth_txg_of_walked_units
+            .get(unit)
+            .or_else(|| birth_txg_of_walked_instance_table_pages.get(unit))
+            .copied()
+    };
+    let mut versions_of_each_txg: BTreeMap<u64, Vec<&VersionReferencesForTheAppearanceRule>> =
+        BTreeMap::new();
+    for version in versions {
+        versions_of_each_txg
+            .entry(version.checkpoint_txg)
+            .or_default()
+            .push(version);
+    }
+    let groups: Vec<(u64, Vec<&VersionReferencesForTheAppearanceRule>)> =
+        versions_of_each_txg.into_iter().collect();
+    let mut units_judged: u64 = 0;
+    for adjacent in groups.windows(2) {
+        let (previous_txg, previous_versions) = (adjacent[0].0, &adjacent[0].1);
+        let versions_at_this_txg = &adjacent[1].1;
+        let previous_references_are_complete = previous_versions
+            .iter()
+            .all(|previous_version| previous_version.references_are_complete);
+        let units_referenced_by_the_previous_version: BTreeSet<(u32, u64)> = previous_versions
+            .iter()
+            .flat_map(|previous_version| previous_version.units.keys().copied())
+            .collect();
+        let previous_instances: BTreeSet<u32> = previous_versions
+            .iter()
+            .map(|previous_version| previous_version.instance)
+            .collect();
+        for version in versions_at_this_txg {
+            if !previous_references_are_complete || !version.references_are_complete {
+                continue;
+            }
+            let hole = hole_in_the_candidate_set_between(
+                previous_txg,
+                &previous_instances,
+                version.checkpoint_txg,
+                version.instance,
+                instance_table_rows_of_the_newest_root,
+            );
+            let trees_inherited_whole: BTreeSet<UserVisibleTreeOfTheWalk> = version
+                .user_visible_tree_roots
+                .iter()
+                .filter(|root| {
+                    root.root_pointer_birth_txg < previous_txg
+                        && version
+                            .units
+                            .iter()
+                            .filter(|(_, tree)| **tree == Some(root.tree))
+                            .filter_map(|(unit, _)| birth_txg_of(unit))
+                            .all(|birth_txg| birth_txg <= root.root_pointer_birth_txg)
+                })
+                .map(|root| root.tree)
+                .collect();
+            for (unit, user_visible_tree) in &version.units {
+                if units_referenced_by_the_previous_version.contains(unit) {
+                    continue;
+                }
+                if hole == HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows
+                    && user_visible_tree.is_some()
+                {
+                    continue;
+                }
+                let Some(birth_txg) = birth_txg_of(unit) else {
+                    continue;
+                };
+                units_judged += 1;
+                let born_after_the_previous_version = birth_txg > previous_txg;
+                let in_a_tree_inherited_whole =
+                    user_visible_tree.is_some_and(|tree| trees_inherited_whole.contains(&tree));
+                judgements.judge(
+                    REFERENCE_INTERVALS_ARE_EXACT,
+                    born_after_the_previous_version || in_a_tree_inherited_whole,
+                    || {
+                        format!(
+                            "相邻两版 (实例 {previous_instances:?}、txg {previous_txg}) → (实例 {}、txg {})（{hole:?}）：后一版引用、前一版不引用的单元盘 {} 槽 {}（用户可见树 {user_visible_tree:?}）诞生代号 {birth_txg} 不大于前一版的 txg，也不在一棵整棵继承下来的用户可见树里（根指针出生 txg 小于前一版的 txg、子树里读得出的诞生代号都不大于它）",
+                            version.instance, version.checkpoint_txg, unit.0, unit.1
+                        )
+                    },
+                );
+            }
+        }
+    }
+    if units_judged == 0 {
+        judgements.not_applicable(
+            REFERENCE_INTERVALS_ARE_EXACT,
+            "回退候选集里相邻两版之间一个新出现、头读得出的单元都没判到：候选集只剩一版，或相邻两版有一版走读记过失败，或缺口那一对新出现的只有用户可见树里的单元",
+        );
+    }
 }
 
 /// 走读此刻在走哪一版（D16（发布语义） 已定项 1 的回退候选集里的根，或由记录施加出来、根槽从没落盘的那一版），带它的 `checkpoint_txg`。
@@ -655,6 +900,13 @@ impl<'reader> Walk<'reader> {
             },
             birth_txg_of_walked_units: BTreeMap::new(),
             units_judged_against_an_older_candidate_root: 0,
+            unit_of_each_placement: BTreeMap::new(),
+            user_visible_tree_of_each_placement: BTreeMap::new(),
+            user_visible_tree_being_walked: None,
+            user_visible_tree_roots_of_this_version: Vec::new(),
+            tree_table_entries_of_a_walked_tree_table: BTreeMap::new(),
+            birth_txg_of_walked_instance_table_pages: BTreeMap::new(),
+            versions_walked_for_the_appearance_rule: Vec::new(),
         }
     }
 
@@ -716,6 +968,39 @@ impl<'reader> Walk<'reader> {
     fn begin_walking_a_version(&mut self) {
         self.placements_referenced_in_this_version.clear();
         self.walk_parent_unit = None;
+        self.user_visible_tree_being_walked = None;
+        self.user_visible_tree_roots_of_this_version.clear();
+    }
+
+    /// 一版走完：把它引用的单元（落点折成单元，带用户可见树的标签）、树表里几棵用户可见树的根与走读断没断记下，留给 I-3.5 逐对比。
+    /// `walk_failures_before` 是走这一版之前走读失败记了几条。
+    fn note_the_version_just_walked_for_the_appearance_rule(
+        &mut self,
+        instance: u32,
+        checkpoint_txg: u64,
+        walk_failures_before: usize,
+    ) {
+        let units = self
+            .placements_referenced_in_this_version
+            .keys()
+            .map(|placement| {
+                let unit = *self.unit_of_each_placement.get(placement).expect(
+                    "这一版引用过的每个落点都经 `note_references_of_a_pointer_followed_by_the_walk` 记过（这一版或先走的那一版）",
+                );
+                (
+                    unit,
+                    self.user_visible_tree_of_each_placement.get(placement).copied(),
+                )
+            })
+            .collect();
+        self.versions_walked_for_the_appearance_rule
+            .push(VersionReferencesForTheAppearanceRule {
+                instance,
+                checkpoint_txg,
+                units,
+                user_visible_tree_roots: self.user_visible_tree_roots_of_this_version.clone(),
+                references_are_complete: self.walk_failures.len() == walk_failures_before,
+            });
     }
 
     /// 往下走 `unit` 里的指针之前调：之后跟着指针记下的落点都算在 `unit` 下面（`placements_referenced_below_a_walked_unit`）。
@@ -786,9 +1071,16 @@ impl<'reader> Walk<'reader> {
         span: u64,
         what: &str,
     ) {
+        let unit = (pointer.locations[0].device, pointer.locations[0].slot);
         for (index, location) in pointer.locations.iter().enumerate() {
             self.note_reference(location, span, what);
             let placement = (location.device, location.slot);
+            self.unit_of_each_placement.entry(placement).or_insert(unit);
+            if let Some(tree) = self.user_visible_tree_being_walked {
+                self.user_visible_tree_of_each_placement
+                    .entry(placement)
+                    .or_insert(tree);
+            }
             let is_an_earlier_location_entry_of_this_pointer = pointer.locations[..index]
                 .iter()
                 .any(|earlier| (earlier.device, earlier.slot) == placement);
@@ -1080,7 +1372,8 @@ impl<'reader> Walk<'reader> {
     }
 
     /// 一版的树表（树 ID 0）与它下面每一条树表条目，再加中央映射树的根：根环里的根从根记录里取这两条指针，
-    /// 由记录施加出来的那一版从记录的新根段里取（两处都是 86 字节的码 2 指针）。树表读不出时映射根也不走（走读已经断了）。
+    /// 由记录施加出来的那一版从记录的新根段里取（两处都是 86 字节的码 2 指针）。树表读不出时映射根也不走（走读已经断了）；
+    /// 树表别的版本先走过时条目不重走，映射根照样往下走（共享的就并进这一版）。
     /// `version_name` 是这一版的称呼（哪条根、或由哪条记录施加出来），I-9.16 判红时写进第一处违例。
     fn walk_tree_table_and_central_mapping_root(
         &mut self,
@@ -1089,24 +1382,36 @@ impl<'reader> Walk<'reader> {
         version_name: &str,
     ) {
         // 树表单元：不属于任何树（树 ID 0）。
-        let Some(tree_table) = self.read_index_node(
+        let tree_table_pointer_view = parse_node_pointer(tree_table_pointer);
+        let tree_table_placement = (
+            tree_table_pointer_view.locations[0].device,
+            tree_table_pointer_view.locations[0].slot,
+        );
+        let tree_table_walked_by_an_earlier_version =
+            !tree_table_pointer_view.all_zero && self.visited_units.contains(&tree_table_placement);
+        let tree_table = self.read_index_node(
             tree_table_pointer,
             0,
             KEY_SCHEMA_TREE_TABLE,
             "树表单元",
             KeyRangeReading::FirstAndLastEntries,
             EntryFieldTableRegistration::NotRegistered,
-        ) else {
-            return;
-        };
-        judge_tree_table_entries_ordering(&mut self.judgements, &tree_table.entries, version_name);
-        let tree_table_locations = parse_node_pointer(tree_table_pointer).locations;
-        let previous_parent_unit = self
-            .start_walking_below((tree_table_locations[0].device, tree_table_locations[0].slot));
-        for entry in &tree_table.entries {
-            self.walk_tree_table_entry(entry);
+        );
+        match tree_table {
+            Some(tree_table) => self.walk_the_entries_of_a_tree_table_walked_first_by_this_version(
+                &tree_table,
+                tree_table_placement,
+                version_name,
+            ),
+            // 别的版本先走过这片树表（零单元发布、卸载与抬 F 的空发布照抄上一版的树表与映射根）：它下面的落点已经并进这一版
+            // （`read_index_node` 里的 `merge_placements_below_a_unit_walked_by_an_earlier_version`），条目照先走那一版记下的判；
+            // 映射根照样往下并（这一版也引用它，I-3.5 比相邻两版的引用集合要它，`research/prompts/m3-birth-floor-r3-main-verification.md`）。
+            None if tree_table_walked_by_an_earlier_version => {
+                self.judge_the_tree_table_entries_an_earlier_version_walked(tree_table_placement);
+            }
+            // 树表读不出、头用不了：走读已经断了，映射根也不走。
+            None => return,
         }
-        self.stop_walking_below(previous_parent_unit);
         // 中央映射树不进树表，引用它的是那条映射根指针（根记录里的，或由记录施加出来那一版的新根段里的）：
         // I-1.3（块头树 ID 一致） 的「实际引用它的树」按那条指针头部的出生树取（D19（块指针的结构与宽度预算） 已定项 7），
         // 与树表条目里的树 ID 对树表指着的根是同一个读法。号不写死映射树的号（mkfs 那条流上是 65540）：回退到树表 0 条的一版之后再发第一个文件版本，
@@ -1130,6 +1435,40 @@ impl<'reader> Walk<'reader> {
         self.walk_central_mapping_entries(&leaf_entries);
     }
 
+    /// 这一版第一个走到的树表：判条目次序（I-9.16）与每一条的 previous_snapshot_txg（I-9.25），把条目里 I-9.25 与 I-3.5 读的几样记下
+    /// （别的版本共享这片树表时照它判），再逐条往下走；走进用户可见树时给记下的落点打上那棵树的标签（I-3.5）。
+    fn walk_the_entries_of_a_tree_table_walked_first_by_this_version(
+        &mut self,
+        tree_table: &crate::IndexNodeView,
+        tree_table_placement: (u32, u64),
+        version_name: &str,
+    ) {
+        judge_tree_table_entries_ordering(&mut self.judgements, &tree_table.entries, version_name);
+        let entries_of_the_walk: Vec<TreeTableEntryOfTheWalk> = tree_table
+            .entries
+            .iter()
+            .filter_map(|entry| tree_table_entry_of_the_walk(entry))
+            .collect();
+        self.tree_table_entries_of_a_walked_tree_table
+            .insert(tree_table_placement, entries_of_the_walk.clone());
+        for entry in &entries_of_the_walk {
+            self.judge_previous_snapshot_txg_of_a_tree_table_entry(*entry);
+        }
+        self.user_visible_tree_roots_of_this_version = entries_of_the_walk
+            .iter()
+            .filter_map(|entry| entry.user_visible_root)
+            .collect();
+        let previous_parent_unit = self.start_walking_below(tree_table_placement);
+        for entry in &tree_table.entries {
+            self.user_visible_tree_being_walked = tree_table_entry_of_the_walk(entry)
+                .and_then(|entry_of_the_walk| entry_of_the_walk.user_visible_root)
+                .map(|root| root.tree);
+            self.walk_tree_table_entry(entry);
+            self.user_visible_tree_being_walked = None;
+        }
+        self.stop_walking_below(previous_parent_unit);
+    }
+
     /// 中央映射树叶里的条目逐条走（D19（块指针的结构与宽度预算） 已定项 6 / 10：key 27 = 类标签 1 + 出生树 8 + 出生 txg 8 +
     /// 实例代号 4 + 尾段 6，value = 位置条目 14 × 2）。入参的条目宽由走叶的那一步判过 ≥ 55（`EntryWidthInTheWalk::GuardedForTheWalkOnly`），
     /// 这里按字段表的固定偏移切。每一条：
@@ -1461,25 +1800,6 @@ impl<'reader> Walk<'reader> {
         let tree = read_u64(entry, 0);
         let kind = read_u16(entry, 10);
         self.tree_identifiers_in_tables.insert(tree);
-        // I-9.25：`previous_snapshot_txg` 只写在 inode 树那条条目里，0 = 不适用；inode 树那条不大于这一版的 txg
-        // （D5（快照 / 空间记账机制） 已定项 9；「≤ 根 txg」那一半用户 2026-10-01 补定）。树表被几版共享时只在第一个走到它的那一版判
-        // （`visited_units`），共享它的别的版本 txg 更大，判得更松。
-        let previous_snapshot_txg = read_u64(entry, TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET);
-        let version_txg = self.version_being_walked.checkpoint_txg();
-        let previous_snapshot_txg_holds = if kind == TREE_KIND_INODE {
-            previous_snapshot_txg <= version_txg
-        } else {
-            previous_snapshot_txg == 0
-        };
-        self.judgements.judge(
-            PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
-            previous_snapshot_txg_holds,
-            || {
-                format!(
-                    "树 {tree}（种类 {kind}）的树表条目 previous_snapshot_txg {previous_snapshot_txg}：不是 inode 树的条目恒 0，inode 树的不大于这一版的 txg {version_txg}"
-                )
-            },
-        );
         let what = format!("树 {tree}（种类 {kind}）的根");
         let Some(schema) = key_schema_for_tree_kind(kind) else {
             return;
@@ -1595,19 +1915,103 @@ impl<'reader> Walk<'reader> {
         }
         // 墓碑树、谱系树：码 2 根之下是码 3 容器（类型段墓碑 {0, 1}、谱系 {0, 5}，D18（块里携带什么信息） 已定项 24）。
         // 这里只走到每片容器、判它的头与容器格式（单元算进可达与已有统计）；墓碑与谱系的专门不变量（I-9.17–I-9.24）归下一步。
+        let node_locations = parse_node_pointer(&entry[14..100]).locations;
+        let node_placement = (node_locations[0].device, node_locations[0].slot);
         if kind == TREE_KIND_TOMBSTONE {
-            self.walk_container_tree_root(&node, tree, PACKED_TYPE_TOMBSTONE, "墓碑树");
+            self.walk_container_tree_root(
+                &node,
+                node_placement,
+                tree,
+                PACKED_TYPE_TOMBSTONE,
+                "墓碑树",
+            );
         }
         if kind == TREE_KIND_LINEAGE {
-            self.walk_container_tree_root(&node, tree, PACKED_TYPE_LINEAGE, "谱系树");
+            self.walk_container_tree_root(
+                &node,
+                node_placement,
+                tree,
+                PACKED_TYPE_LINEAGE,
+                "谱系树",
+            );
+        }
+    }
+
+    /// I-9.25：`previous_snapshot_txg` 只写在 inode 树那条条目里，0 = 不适用；inode 树那条不大于这一版的 txg
+    /// （D5（快照 / 空间记账机制） 已定项 9；「≤ 根 txg」那一半用户 2026-10-01 补定）。走到的每一版（候选集里每条，含由记录施加出来的）
+    /// 拿自己的 txg 判自己树表里的每一条：树表被几版共享时每一版各判一次（[`Self::judge_the_tree_table_entries_an_earlier_version_walked`]），
+    /// 不只在第一个走到它的那一版判（主 agent 2026-10-03 定改回 kb 字面）。
+    fn judge_previous_snapshot_txg_of_a_tree_table_entry(
+        &mut self,
+        entry: TreeTableEntryOfTheWalk,
+    ) {
+        let version_txg = self.version_being_walked.checkpoint_txg();
+        let TreeTableEntryOfTheWalk {
+            tree,
+            kind,
+            previous_snapshot_txg,
+            ..
+        } = entry;
+        let previous_snapshot_txg_holds = if kind == TREE_KIND_INODE {
+            previous_snapshot_txg <= version_txg
+        } else {
+            previous_snapshot_txg == 0
+        };
+        self.judgements.judge(
+            PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
+            previous_snapshot_txg_holds,
+            || {
+                format!(
+                    "树 {tree}（种类 {kind}）的树表条目 previous_snapshot_txg {previous_snapshot_txg}：不是 inode 树的条目恒 0，inode 树的不大于这一版的 txg {version_txg}"
+                )
+            },
+        );
+    }
+
+    /// 这一版的树表单元别的版本先走过（`visited_units`）：不重读盘，照先走那一版记下的条目判这一版的 I-9.25，
+    /// 这一版树表里几棵用户可见树的根也照它取（I-3.5）。先走那一版头用不了、没记下条目的，这里没有对象。
+    fn judge_the_tree_table_entries_an_earlier_version_walked(
+        &mut self,
+        tree_table_placement: (u32, u64),
+    ) {
+        let Some(entries) = self
+            .tree_table_entries_of_a_walked_tree_table
+            .get(&tree_table_placement)
+            .cloned()
+        else {
+            return;
+        };
+        for entry in &entries {
+            self.judge_previous_snapshot_txg_of_a_tree_table_entry(*entry);
         }
+        self.user_visible_tree_roots_of_this_version = entries
+            .iter()
+            .filter_map(|entry| entry.user_visible_root)
+            .collect();
     }
 
     /// 墓碑树或谱系树的码 2 根（或根长高之后的内部节点）往下走：条目 120 字节（分隔 key 8 + 身份引用 26 + 子指针 86），
     /// 类型段 = `record_type` 的走到码 3 容器（判容器头、出生树、容器格式，单元记进这一版走到的落点），类型段 0 的走进下一层码 2 节点
     /// （层级 = 父层级 − 1），别的类型段记走读失败、不往下走。用 checker 自己的读法，不调 `singlefs_core`（D13（验证路线） 已定项 5）。
     /// 递归的上界：每进一层类型段 0，层级减一（层级 0 的码 2 节点在这两棵树里没有条款定的形状，记走读失败）。
+    ///
+    /// 往下记的落点都记在 `node_placement` 这个节点下面（`placements_referenced_below_a_walked_unit`）：别的版本共享这个节点时
+    /// 照这张表把它下面的容器并进那一版（I-5.1 与 I-3.5 都要），不记在外面那片树表下面。
     fn walk_container_tree_root(
+        &mut self,
+        node: &crate::IndexNodeView,
+        node_placement: (u32, u64),
+        tree: u64,
+        record_type: u16,
+        name: &str,
+    ) {
+        let previous_parent_unit = self.start_walking_below(node_placement);
+        self.walk_the_entries_of_a_container_tree_node(node, tree, record_type, name);
+        self.stop_walking_below(previous_parent_unit);
+    }
+
+    /// [`Self::walk_container_tree_root`] 那一个节点的条目逐条走（循环里有 `continue`，所以与换走读父单元那两步分开写）。
+    fn walk_the_entries_of_a_container_tree_node(
         &mut self,
         node: &crate::IndexNodeView,
         tree: u64,
@@ -1655,7 +2059,13 @@ impl<'reader> Walk<'reader> {
                 if child_node.level + 1 == node.level
                     && child_node.entry_width == inode_internal_entry_bytes()
                 {
-                    self.walk_container_tree_root(&child_node, tree, record_type, name);
+                    self.walk_container_tree_root(
+                        &child_node,
+                        (child.locations[0].device, child.locations[0].slot),
+                        tree,
+                        record_type,
+                        name,
+                    );
                 }
                 continue;
             }
@@ -2150,6 +2560,8 @@ impl<'reader> Walk<'reader> {
             };
             self.judge_pointer_birth_tree_against_the_unit_header(&unit, &pointer, &what);
             if let Some(birth_identity) = birth_identity_in_the_header(&unit) {
+                self.birth_txg_of_walked_instance_table_pages
+                    .insert(page_placement, birth_identity.write_order.birth_txg());
                 self.judge_birth_identity_against_the_pointer(birth_identity, &pointer, &what);
             }
             let identity = (
@@ -5980,11 +6392,59 @@ const NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK: std::ops::Range<usize> = 172..
 const NEW_ROOT_SEGMENT_ROLLBACK_FLOOR: std::ops::Range<usize> = 180..188;
 const NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER: std::ops::Range<usize> = 188..274;
 const NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER: std::ops::Range<usize> = 274..360;
-/// 根记录里与新根段四段同名的四段（D22（单元原子性怎么合成） 已定项 7 的偏移表：树表指针 36、树 ID 水位 122、F 130、映射根指针 256）。
+const NEW_ROOT_SEGMENT_UNMOUNT_MARKER: std::ops::Range<usize> = 360..364;
+/// 根记录里与新根段七段同名的七段（D22（单元原子性怎么合成） 已定项 7 的偏移表：flags 20（位 0 是卸载记号）、树表指针 36、
+/// 树 ID 水位 122、F 130、实例表指针 170、映射根指针 256、树表 0 条那一版的分配记录树根 342）。
+const ROOT_RECORD_FLAGS: std::ops::Range<usize> = 20..24;
 const ROOT_RECORD_TREE_TABLE_POINTER: std::ops::Range<usize> = 36..122;
 const ROOT_RECORD_TREE_IDENTIFIER_WATERMARK: std::ops::Range<usize> = 122..130;
 const ROOT_RECORD_ROLLBACK_FLOOR: std::ops::Range<usize> = 130..138;
+const ROOT_RECORD_INSTANCE_TABLE_POINTER: std::ops::Range<usize> = 170..256;
 const ROOT_RECORD_MAPPING_ROOT_POINTER: std::ops::Range<usize> = 256..342;
+
+/// I-8.10 逐字节比的七对：(这一段叫什么, 新根段里的落点, 根记录里的落点)。卸载记号对的是根记录 flags 整 4 字节：
+/// 两边同一个编码（位 0），根记录 flags 其余位非 0 的根自证不过、进不了这里，新根段那 4 字节位 0 之外非 0 的记录也进不了这里（RR1）。
+const NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT: &[(
+    &str,
+    std::ops::Range<usize>,
+    std::ops::Range<usize>,
+)] = &[
+    (
+        "树表指针",
+        NEW_ROOT_SEGMENT_TREE_TABLE_POINTER,
+        ROOT_RECORD_TREE_TABLE_POINTER,
+    ),
+    (
+        "映射根指针",
+        NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER,
+        ROOT_RECORD_MAPPING_ROOT_POINTER,
+    ),
+    (
+        "树 ID 水位",
+        NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK,
+        ROOT_RECORD_TREE_IDENTIFIER_WATERMARK,
+    ),
+    (
+        "回退下界 F",
+        NEW_ROOT_SEGMENT_ROLLBACK_FLOOR,
+        ROOT_RECORD_ROLLBACK_FLOOR,
+    ),
+    (
+        "实例表指针",
+        NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER,
+        ROOT_RECORD_INSTANCE_TABLE_POINTER,
+    ),
+    (
+        "分配记录树根指针",
+        NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER,
+        ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT,
+    ),
+    (
+        "卸载记号",
+        NEW_ROOT_SEGMENT_UNMOUNT_MARKER,
+        ROOT_RECORD_FLAGS,
+    ),
+];
 /// 根记录里实例代号 4 字节与 `checkpoint_txg` 8 字节的偏移（D22（单元原子性怎么合成） 已定项 7 的偏移表，与 `check_root_slot` 读的同一处）：
 /// I-9.16 判红时拿它们说是哪一版根。
 const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;
@@ -6177,8 +6637,10 @@ fn judge_blocks_referenced_by_abandoned_roots(
 }
 
 /// I-8.10（记录新根段与同代根相同）：逐盘，按 (实例代号, `checkpoint_txg`) 分组，组里带末条标志（记录标志位 0）的恰一条时，
-/// 它的新根段四段与根环里同 (实例代号, `checkpoint_txg`) 的每条自证过的根的同名四段逐字节相同（D23（journal 的角色与格式） 已定项 15）：
-/// 新根段 [0..86) = 根 [36..122)、[86..172) = [256..342)、[172..180) = [122..130)、[180..188) = [130..138)。
+/// 它的新根段七段与根环里同 (实例代号, `checkpoint_txg`) 的每条自证过的根的同名七段逐字节相同（D23（journal 的角色与格式） 已定项 15；
+/// 用户 2026-10-03 定从四段扩到七段）：新根段 [0..86) = 根 [36..122)、[86..172) = [256..342)、[172..180) = [122..130)、
+/// [180..188) = [130..138)、[188..274) = [170..256)、[274..360) = [342..428)、[360..364) = flags [20..24)
+/// （[`NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT`]）。
 /// 末条标志不唯一的组归 I-8.9（一次发布的记录序号连续且只有末条带标志），不判；同 (实例, txg) 有两条以上不同内容的根不另分，逐条比。
 fn judge_new_root_segment_of_the_last_record_against_the_roots(
     records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
@@ -6207,19 +6669,23 @@ fn judge_new_root_segment_of_the_last_record_against_the_roots(
                 }
                 pairs_compared += 1;
                 let root_bytes = &root.record_bytes;
-                let same = segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER]
-                    == root_bytes[ROOT_RECORD_TREE_TABLE_POINTER]
-                    && segment[NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER]
-                        == root_bytes[ROOT_RECORD_MAPPING_ROOT_POINTER]
-                    && segment[NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK]
-                        == root_bytes[ROOT_RECORD_TREE_IDENTIFIER_WATERMARK]
-                    && segment[NEW_ROOT_SEGMENT_ROLLBACK_FLOOR]
-                        == root_bytes[ROOT_RECORD_ROLLBACK_FLOOR];
-                judgements.judge(LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH, same, || {
-                    format!(
-                        "盘 {device} journal 计数器 {counter} 那条带末条标志的记录（实例 {instance}、txg {checkpoint_txg}）的新根段与区域 {region} 槽 {slot} 那条同代根的四段不逐字节相同"
-                    )
-                });
+                let differing_fields: Vec<&str> = NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT
+                    .iter()
+                    .filter(|(_, in_the_segment, in_the_root)| {
+                        segment[in_the_segment.clone()] != root_bytes[in_the_root.clone()]
+                    })
+                    .map(|(name, _, _)| *name)
+                    .collect();
+                judgements.judge(
+                    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
+                    differing_fields.is_empty(),
+                    || {
+                        format!(
+                            "盘 {device} journal 计数器 {counter} 那条带末条标志的记录（实例 {instance}、txg {checkpoint_txg}）的新根段与区域 {region} 槽 {slot} 那条同代根这几段不逐字节相同：{}",
+                            differing_fields.join("、")
+                        )
+                    },
+                );
             }
         }
     }
@@ -6970,6 +7436,11 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
     );
     // 先走最新的根：它的走读断没断就是 I-7.2；记账行只取最新根下面的。
     walk.walk_root(&roots[newest_index].2.record_bytes, true);
+    walk.note_the_version_just_walked_for_the_appearance_rule(
+        roots[newest_index].2.instance,
+        roots[newest_index].2.checkpoint_txg,
+        0,
+    );
     let newest_failures = walk.walk_failures.clone();
     // I-3.11（已分配减 defer 等于最新根走读）要的「从最新有效根走读到的、这块盘上被引用的槽数」：走法与 I-3.1 同一个
     // （`note_reference` 记下的 (设备, 起点槽, 跨度)），只取最新根——这一刻 `references` 里还只有最新根这一遍记下的。
@@ -7088,6 +7559,11 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
             let mismatches_before = walk.judgements.violation_count("I-2.1");
             let failures_before = walk.walk_failures.len();
             walk.walk_root(&root.record_bytes, false);
+            walk.note_the_version_just_walked_for_the_appearance_rule(
+                root.instance,
+                root.checkpoint_txg,
+                failures_before,
+            );
             // 这条候选根引用的单元有一个校验和对不上或头用不了，就是它指着的块被复用或抹头了（I-7.4（近 K 代块未被复用）），
             // 从它出发的遍历也就不自洽（I-4.8（近 K 代根校验和自洽））；两条按每条候选根各判一格。
             let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
@@ -7120,6 +7596,11 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         let mismatches_before = walk.judgements.violation_count("I-2.1");
         let failures_before = walk.walk_failures.len();
         walk.walk_version_applied_only_by_records(version);
+        walk.note_the_version_just_walked_for_the_appearance_rule(
+            version.instance,
+            version.checkpoint_txg,
+            failures_before,
+        );
         let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
             > mismatches_before
             || walk.walk_failures.len() > failures_before;
@@ -7137,6 +7618,13 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
                 )
             });
     }
+    judge_the_appearance_rule_of_adjacent_candidate_versions(
+        &walk.versions_walked_for_the_appearance_rule,
+        &walk.birth_txg_of_walked_units,
+        &walk.birth_txg_of_walked_instance_table_pages,
+        &walk.instance_table_rows,
+        &mut walk.judgements,
+    );
     if walk.units_judged_against_an_older_candidate_root == 0 {
         walk.judgements.not_applicable(
             UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
diff --git a/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs b/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
index c4b6b68..2e231ce 100644
--- a/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
+++ b/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
@@ -1960,9 +1960,9 @@ const FIRST_RUN_MK_PLUS_ROW_NAME: &str = "P9-Mk+";
 const BLIND_PSEUDO_ROOT_STEPS: u64 = 7;
 const BLIND_LOWEST_WATERMARK_STEP: u64 = 1000;
 const BLIND_HIGHEST_WATERMARK_STEP: u64 = 1_000_000;
-/// 记录的点名项区起点 = 记录头字节数（`layout/01-first-txn.md` 六节记录头 311 字节）；新根段起点 = 本次发布内序号 4 + 反向链 4 + 载荷校验和 4
+/// 记录的点名项区起点 = 记录头字节数（D23（journal 的角色与格式） 已定项 4：记录头 487 字节）；新根段起点 = 本次发布内序号 4 + 反向链 4 + 载荷校验和 4
 /// 之后。两个数写成本地常量，`bad-states` 开头与 core 的公开常量回比（S1 同形，回比不过照打、退 3）。
-const BAD_RECORD_NAMED_ITEM_AREA_START: usize = 311;
+const BAD_RECORD_NAMED_ITEM_AREA_START: usize = 487;
 const BAD_RECORD_NEW_ROOT_SEGMENT_START: usize = 99;
 /// 系统配置槽里「根环起点」那 8 字节的偏移（`layout/01-first-txn.md` 一）：core 里是私有常量，补丁之后由 `SystemConfiguration::parse_slot`
 /// 读回 65 来核这个偏移（`bad_s1` 行 `p1_root_ring_base_reads_back`）。
@@ -3105,7 +3105,8 @@ fn bad_local_constant_checks(facts: &BadStreamFacts<'_>, full: &BaseState) -> Ve
         StopCheck::new(
             &format!("{label}_record_named_item_area_start"),
             BAD_RECORD_NAMED_ITEM_AREA_START,
-            usize::try_from(singlefs_format::JOURNAL_HEADER_BYTES).expect("311"),
+            usize::try_from(singlefs_format::JOURNAL_HEADER_BYTES)
+                .expect("记录头 487 字节装得进 usize"),
         ),
         StopCheck::new(
             &format!("{label}_record_new_root_segment_start"),
diff --git a/crates/singlefs-checker-tier/src/crash_identity.rs b/crates/singlefs-checker-tier/src/crash_identity.rs
index 58f50f6..47def7a 100644
--- a/crates/singlefs-checker-tier/src/crash_identity.rs
+++ b/crates/singlefs-checker-tier/src/crash_identity.rs
@@ -1229,9 +1229,12 @@ mod tests {
 
     fn repository_root() -> PathBuf {
         PathBuf::from(env!("CARGO_MANIFEST_DIR"))
-            .join("../..")
+            .join("../../Cargo.lock")
             .canonicalize()
-            .expect("仓根在 crate 往上两层")
+            .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
+            .parent()
+            .expect("Cargo.lock 所在的目录就是仓根")
+            .to_path_buf()
     }
 
     #[test]
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index a861164..fe42d09 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -299,7 +299,11 @@ fn checker_record_valid(s: Slice, fsid_low: vec2<u32>) -> bool {
     if (rd32(base, 0u) != MAGIC_SFSJ || s.c0 != 1u || rd16(base, 4u) != 1u || rd32(base, 8u) != JOURNAL_RECORD_BYTES) {
         return false;
     }
-    if (!eq64(rd64(base, 287u), fsid_low)) {
+    if (!eq64(rd64(base, JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET), fsid_low)) {
+        return false;
+    }
+    // 新根段末尾卸载记号那 4 字节位 0 之外有位为 1：读者与恢复当这条记录损坏，checker 同一口径不收（RR1）
+    if ((rd32(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 360u) & 0xFFFFFFFEu) != 0u) {
         return false;
     }
     if (s.c1 != 1u) {
@@ -667,8 +671,9 @@ fn judge_named_entries_against_the_unit_headers_of_device() {
     }
 }
 
-// I-8.10：同 (实例代号, checkpoint_txg) 带末条标志的记录恰一条时，它的新根段四段（记录偏移 99 起：树表指针 86、映射根指针 86、
-// 树 ID 水位 8、回退下界 F 8）与根环里同 (实例代号, checkpoint_txg) 的每条自证过的根的同名四段（根记录 36、256、122、130）逐字节相同
+// I-8.10：同 (实例代号, checkpoint_txg) 带末条标志的记录恰一条时，它的新根段七段（记录偏移 99 起：树表指针 86、映射根指针 86、
+// 树 ID 水位 8、回退下界 F 8、实例表指针 86、分配记录树根指针 86、卸载记号 4）与根环里同 (实例代号, checkpoint_txg) 的每条自证过的根的
+// 同名七段（根记录 36、256、122、130、170、342，卸载记号对根记录 flags 20 那 4 字节）逐字节相同（用户 2026-10-03 定从四段扩到七段）
 fn judge_new_root_segment_of_the_last_record_against_the_roots_of_device() {
     for (var i = 0u; i < record_count; i = i + 1u) {
         let r = checker_record_at(i);
@@ -694,8 +699,14 @@ fn judge_new_root_segment_of_the_last_record_against_the_roots_of_device() {
             if (root.instance != r.instance || !eq64(root.txg, r.txg)) {
                 continue;
             }
-            let same = bytes_equal(r.base, 99u, root.base, 36u, 86u) && bytes_equal(r.base, 185u, root.base, 256u, 86u)
-                && bytes_equal(r.base, 271u, root.base, 122u, 8u) && bytes_equal(r.base, 279u, root.base, 130u, 8u);
+            let segment = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
+            let same = bytes_equal(r.base, segment, root.base, 36u, 86u)
+                && bytes_equal(r.base, segment + 86u, root.base, 256u, 86u)
+                && bytes_equal(r.base, segment + 172u, root.base, 122u, 8u)
+                && bytes_equal(r.base, segment + 180u, root.base, 130u, 8u)
+                && bytes_equal(r.base, segment + 188u, root.base, 170u, 86u)
+                && bytes_equal(r.base, segment + 274u, root.base, 342u, 86u)
+                && bytes_equal(r.base, segment + 360u, root.base, 20u, 4u);
             if (!same) {
                 violate(INVARIANT_I_8_10);
             }
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index 10db56b..56c538d 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -88,8 +88,13 @@ const NODE_BYTES: u32 = 16384u;
 const DATA_UNIT_BYTES: u32 = 32768u;
 const ROOT_RECORD_BYTES: u32 = 457u;
 const JOURNAL_RECORD_BYTES: u32 = 4096u;
-const JOURNAL_HEADER_BYTES: u32 = 311u;
+const JOURNAL_HEADER_BYTES: u32 = 487u;
 const JOURNAL_NAMED_ENTRY_BYTES: u32 = 56u;
+// 记录头里新根段的起点与宽（D23（journal 的角色与格式） 已定项 4 / 已定项 15：树表指针 86、映射根指针 86、树 ID 水位 8、F 8、
+// 实例表指针 86、分配记录树根指针 86、卸载记号 4），fsid 8 紧跟新根段（偏移 463），头到 487 为止
+const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: u32 = 99u;
+const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u32 = 364u;
+const JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET: u32 = 463u;
 const JOURNAL_RING_START_BYTES: u32 = 16777216u;   // 1024 × 16384
 const SYSTEM_CONFIGURATION_SLOT_BYTES: u32 = 4096u;
 const ROOT_RING_REGION_STRIDE: u32 = 3145728u;     // P × chunk = 3 × 1 MiB
@@ -976,7 +981,7 @@ fn parse_record(s: Slice, fsid_low: vec2<u32>) -> Rec {
     if (!tree_table.head_ok || !mapping_root.head_ok) {
         return invalid_record();
     }
-    if (!eq64(rd64(base, 287u), fsid_low)) {
+    if (!eq64(rd64(base, JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET), fsid_low)) {
         return invalid_record();
     }
     if (s.c1 != 1u) {
@@ -1092,11 +1097,11 @@ fn records_parse_equal(base_a: u32, base_b: u32) -> bool {
     if (!bytes_equal(base_a, 78u, base_b, 78u, 21u)) {
         return false;
     }
-    if (!bytes_equal(base_a, 99u, base_b, 99u, 196u)) {
+    if (!bytes_equal(base_a, JOURNAL_NEW_ROOT_SEGMENT_OFFSET, base_b, JOURNAL_NEW_ROOT_SEGMENT_OFFSET, JOURNAL_NEW_ROOT_SEGMENT_BYTES + 8u)) {
         return false;
     }
     let named = rd32(base_a, 12u);
-    return bytes_equal(base_a, 311u, base_b, 311u, named * JOURNAL_NAMED_ENTRY_BYTES);
+    return bytes_equal(base_a, JOURNAL_HEADER_BYTES, base_b, JOURNAL_HEADER_BYTES, named * JOURNAL_NAMED_ENTRY_BYTES);
 }
 // 反向链判定：0 成立、1 断、2 前一条不在盘上
 fn judge_back_chain(r: Rec, ring_bytes: vec2<u32>) -> u32 {
@@ -1114,7 +1119,7 @@ fn judge_back_chain(r: Rec, ring_bytes: vec2<u32>) -> u32 {
     let slot_in_ring = divmod64(sub64(previous_counter, u64_of(1u)), ring_slots).remainder;
     let offset = add64(u64_of(JOURNAL_RING_START_BYTES), shl64(slot_in_ring, 12u));
     let device_count = hdr(3u);
-    let fsid_low = rd64(previous_base, 287u);
+    let fsid_low = rd64(previous_base, JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET);
     for (var d = 0u; d < device_count; d = d + 1u) {
         let s = read_at(hdr(12u + d), offset, JOURNAL_RECORD_BYTES);
         if (s.version == NONE) {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index 6aa90b3..761b06b 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -58,7 +58,7 @@ const _: () = assert!(
     "违例位读成一个 u128"
 );
 
-/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 65 条——三批 49 条，加里程碑三二·1 那一批 16 条
+/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：前 65 条——三批 49 条，加里程碑三二·1 那一批 16 条
 /// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），对拍逐位比、不屏蔽。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
@@ -66,12 +66,19 @@ pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64,
 ];
 
-// 覆盖表是池级 checker 的每一条、按下标次序：新立一条不变量而内核没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0。
+/// 池级 checker 已经判、GPU 内核还没判的不变量（下标接在覆盖表之后）：下标 65 是 I-3.5（引用区间的精确性，里程碑三第 8 批进池级 checker，
+/// 派发没含 GPU 内核那一半，交主 agent 定）。内核那一位恒 0：对拍逐位比照旧不屏蔽，CPU 在哪个状态上判红它，对拍就在那个状态上报不同；
+/// 崩溃放量由 GPU 判的那几趟判不到它（`crash_amplification` 按违例位取不变量）。
+pub const POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET: &[usize] = &[65];
+
+// 覆盖表加上「内核还没判」那张表是池级 checker 的每一条、按下标次序：新立一条不变量而两张表都没跟上时这里编译不过，
+// 不悄悄让 GPU 那一位恒 0；还没判的那几条要在「内核还没判」那张表里点名。
 const _: () = {
     assert!(
         POOL_CHECKER_COVERED_INVARIANT_INDEXES.len()
+            + POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET.len()
             == singlefs_checker::image::IMPLEMENTED_INVARIANTS.len(),
-        "池级 checker 新立的不变量 GPU 内核还没判：补内核，再把下标补进覆盖表"
+        "池级 checker 新立的不变量 GPU 内核还没判：补内核，再把下标补进覆盖表（暂不补内核的，点名进「内核还没判」那张表）"
     );
     let mut position = 0;
     while position < POOL_CHECKER_COVERED_INVARIANT_INDEXES.len() {
@@ -81,6 +88,15 @@ const _: () = {
         );
         position += 1;
     }
+    let mut position_not_judged = 0;
+    while position_not_judged < POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET.len() {
+        assert!(
+            POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET[position_not_judged]
+                == POOL_CHECKER_COVERED_INVARIANT_INDEXES.len() + position_not_judged,
+            "「内核还没判」那张表接在覆盖表之后、按下标次序"
+        );
+        position_not_judged += 1;
+    }
 };
 
 /// 一个状态结论里池级 checker 的违例位（[`POOL_CHECKER_VIOLATED_BIT_CAPACITY`] 位，位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
diff --git a/crates/singlefs-checker-tier/src/crash_judge_tables.rs b/crates/singlefs-checker-tier/src/crash_judge_tables.rs
index 0339ffd..e92b081 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_tables.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_tables.rs
@@ -737,7 +737,7 @@ fn sector_equality(
 ///
 /// - 系统配置：`[整槽校验和字段对得上, 0, 0, 0]`。
 /// - 根槽：`[整槽校验和字段对得上（罩整个槽宽）, 0, 0, 0]`。
-/// - journal 记录：`[整条校验和字段对得上, 载荷 CRC 对得上（点名项区在 4096 之内）, 反向链值 = CRC-32C(头 311 字节、校验和字段按 0), 0]`。
+/// - journal 记录：`[整条校验和字段对得上, 载荷 CRC 对得上（点名项区在 4096 之内）, 反向链值 = CRC-32C(头 487 字节、校验和字段按 0), 0]`。
 /// - 单元槽：`[前 16 KiB 的 CRC-32C, 前 32 KiB 的 CRC-32C（落点只有 16 KiB 时 0）, 标志位, 0]`；标志位：位 0 头校验和按自述的类过、
 ///   位 1 载荷 CRC 按自述的类过、位 2 落点有 32 KiB、位 3 类身份段之后的 29 字节预留位全 0。类从偏移 6 读，码 2 的 key 宽从偏移 51 读。
 #[must_use]
@@ -760,7 +760,8 @@ pub fn version_checks(kind: JudgeLocationKind, bytes: &[u8]) -> [u32; 4] {
         }
         JudgeLocationKind::JournalRecord => {
             let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
-            let header_bytes = usize::try_from(JOURNAL_HEADER_BYTES).expect("311");
+            let header_bytes =
+                usize::try_from(JOURNAL_HEADER_BYTES).expect("记录头 487 字节装得进 usize");
             let header_holds = wide_checksum_field_holds(bytes, record_bytes, 46);
             let named_count = usize::try_from(u32::from_le_bytes(
                 bytes[12..16].try_into().expect("4 字节"),
diff --git a/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs b/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs
index 5320b30..5635f30 100644
--- a/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs
+++ b/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs
@@ -227,9 +227,12 @@ pub fn amplify_prepared_flow_and_hand_back_its_line(
         crash_points,
     };
     let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
-        .join("../..")
+        .join("../../Cargo.lock")
         .canonicalize()
-        .expect("仓根");
+        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
+        .parent()
+        .expect("Cargo.lock 所在的目录就是仓根")
+        .to_path_buf();
     let judging = PipelineJudgingCode::of_the_judges(
         &root,
         &Layer0ToolchainIdentity::of_the_cargo_running_this_test(),
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs
index f1ac6d7..18f49a9 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs
@@ -224,9 +224,12 @@ fn the_fixed_script_through_the_rollback_is_recorded_checked_in_parallel_and_reu
         crash_points,
     };
     let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
-        .join("../..")
+        .join("../../Cargo.lock")
         .canonicalize()
-        .expect("仓根");
+        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
+        .parent()
+        .expect("Cargo.lock 所在的目录就是仓根")
+        .to_path_buf();
     let judging = PipelineJudgingCode::of_the_judges(
         &root,
         &Layer0ToolchainIdentity::of_the_cargo_running_this_test(),
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
index 62e8c62..90373fb 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
@@ -337,9 +337,12 @@ fn the_whole_fixed_script_through_the_unmount_is_recorded_checked_in_parallel_an
         crash_points,
     };
     let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
-        .join("../..")
+        .join("../../Cargo.lock")
         .canonicalize()
-        .expect("仓根");
+        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
+        .parent()
+        .expect("Cargo.lock 所在的目录就是仓根")
+        .to_path_buf();
     let judging = PipelineJudgingCode::of_the_judges(
         &root,
         &Layer0ToolchainIdentity::of_the_cargo_running_this_test(),
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
index 91af6e7..5592d76 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
@@ -42,9 +42,12 @@ const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;
 
 fn repository_root() -> PathBuf {
     PathBuf::from(env!("CARGO_MANIFEST_DIR"))
-        .join("../..")
+        .join("../../Cargo.lock")
         .canonicalize()
-        .expect("仓根在 crate 往上两层")
+        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
+        .parent()
+        .expect("Cargo.lock 所在的目录就是仓根")
+        .to_path_buf()
 }
 
 fn expansion(_segment_index: usize, segment: &[usize]) -> Layer0SegmentExpansion {
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs b/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs
index 98cbcd7..3cc67fb 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs
@@ -158,9 +158,12 @@ fn record_flow(tag: &str, overwrite_content: &[u8], tail_set_in_common: bool) ->
 
 fn repository_root() -> PathBuf {
     PathBuf::from(env!("CARGO_MANIFEST_DIR"))
-        .join("../..")
+        .join("../../Cargo.lock")
         .canonicalize()
-        .expect("仓根")
+        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
+        .parent()
+        .expect("Cargo.lock 所在的目录就是仓根")
+        .to_path_buf()
 }
 
 /// 前缀的点数：名字不以 `publish_overwrite.` 起头的都是前缀。
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs
index 22bcaf0..544d446 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs
@@ -1,19 +1,19 @@
 //! checker 档模块：crash
 //! 层 0 流 L8（里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 36 行、调度表「实二十」；三方 `m2-treesplit-r1` T4 列的八条流里的最后一条）：
 //! 一次发布多条记录、共享点名跨记录。D23（journal 的角色与格式） 已定项 17：最后一个事务要点名的项多于
-//! 一条记录装得下的 67 项时末条再跨记录，只有真正的最后一条带「本次发布末条」标志；已定项 7：同一事务的记录共享事务号、
+//! 一条记录装得下的 64 项时末条再跨记录，只有真正的最后一条带「本次发布末条」标志；已定项 7：同一事务的记录共享事务号、
 //! 提交标记只在它的最后一条；已定项 14 第六条：恢复按记录标志位 0 认发布边界，末条没到的发布整体不施加。
 //!
 //! 写法照 T4「起点镜像不枚举 + 只录那一次发布」：mkfs → 取号 → 暖机 × 2 → A（新池新建文件）整段做完的镜像当起点、不枚举；
-//! 只把建 N 个 inode 的那一次发布（叶容器 + 叶容器之外的角色 = 68 个单元，一个事务，点名 68 项 ⇒ 两条记录：
-//! 67 项不带提交标记与末条标志、1 项两样都带）录进被枚举的流。叶容器之外的角色有几个不写死，由实现现给
-//! （[`leaf_containers_naming_one_more_entry_than_a_record_holds`]）：今天是 11 个（inode 根、分配记录树两盘各叶 61 与叶 62、
-//! 两盘各第 1 层、根，记账树、中央映射树、树表），叶容器 57 片。它的段序列是 `136+4+1+2`：单元段 68 个单元 × 2 盘，
-//! 记录段两条 × 2 盘，根槽 FUA，系统配置槽轮换 × 2 盘。
+//! 只把建 N 个 inode 的那一次发布（叶容器 + 叶容器之外的角色 = 65 个单元，一个事务，点名 65 项 ⇒ 两条记录：
+//! 64 项不带提交标记与末条标志、1 项两样都带）录进被枚举的流。叶容器之外的角色有几个不写死，由实现现给
+//! （[`leaf_containers_naming_one_more_entry_than_a_record_holds`]；叶容器跨过分配记录树叶 61 的末槽时是 11 个——inode 根、分配记录树两盘各叶 61
+//! 与叶 62、两盘各第 1 层、根，记账树、中央映射树、树表；叶容器片数 = 65 − 角色数，记录头改成 487 字节之后没在这条流上跑过）。它的段序列是
+//! `130+4+1+2`：单元段 65 个单元 × 2 盘，记录段两条 × 2 盘，根槽 FUA，系统配置槽轮换 × 2 盘。
 //!
 //! **枚举了哪些崩溃状态**：记录段、根槽两段逐个子集全展开（15 + 1），系统配置槽那一段的两次轮换是原地覆写、各取三态
 //! （3² − 1 = 8，`crash::TearableInPlaceOverwrites`），加全部持久那一个，共 25 个；
-//! 单元段 136 写的 2^136 − 1 个子集枚举不了，它只以整段持久进入后面的状态。单元段里的状态上这次发布的记录一份都没落
+//! 单元段 130 写的 2^130 − 1 个子集枚举不了，它只以整段持久进入后面的状态。单元段里的状态上这次发布的记录一份都没落
 //! （记录段在一道屏障之后），恢复见到的与起点镜像相同的那一版加上几份没人点名的单元——那一格已有的流都罩着。
 //! **比已有的流多罩了什么**：已有的多记录发布（并行线一的层 0 流）一条记录一个事务、每条都带提交标记；这里一个事务跨两条记录，
 //! 第一条不带提交标记，共享的提交内生块分在两条里点名——记录段 15 个状态里：两条都至少落了一份的 8 个（加上根槽那一段的 1 个）
@@ -89,7 +89,7 @@ fn roles_other_than_leaf_containers_named_by_a_trial_publish(
 }
 
 /// 叶容器取几片，这次发布点名的单元才恰好是 [`NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH`] 项。叶容器之外的角色数随叶容器数变
-/// （叶容器落点越过分配记录树叶 61 的末槽时，两块盘各多改一片叶 62：今天 1 片叶容器时 9 个角色、57 片时 11 个），
+/// （叶容器落点越过分配记录树叶 61 的末槽时，两块盘各多改一片叶 62：1 片叶容器时 9 个角色、跨过末槽之后 11 个），
 /// 所以按「叶容器数 = 项数 − 这么多叶容器时的角色数」迭代到不动点，每一轮的角色数都由试发现数。
 ///
 /// 迭代上界 8 轮：角色数随叶容器数不减，从最多的叶容器数（只留 1 个角色）起往下收；今天第 2 轮就不动。
@@ -146,7 +146,7 @@ fn prepare(tag: &str) -> Prepared {
     // 起点镜像：A 整段做完（系统配置槽轮换在内），不枚举。
     let base = pool.memory_pool();
     let operations_before = pool.retained_operations().len();
-    // 叶容器数由实现现给的角色数定（试发在 `base` 的拷贝上，不进录制流）：点名 68 项，比一条记录装得下的 67 多一项。
+    // 叶容器数由实现现给的角色数定（试发在 `base` 的拷贝上，不进录制流）：点名的项数比一条记录装得下的多一项。
     let leaf_containers =
         leaf_containers_naming_one_more_entry_than_a_record_holds(&base, &pool.allocator, &before);
     let new_inode_count = new_inode_count_filling(leaf_containers);
@@ -162,18 +162,13 @@ fn prepare(tag: &str) -> Prepared {
             FIXED_WRITE_TIME_SECONDS + 120,
             InstanceGeneration(1),
         )
-        .expect("点名 68 项：末条再跨记录")
+        .expect("点名的项数比一条记录装得下的多一项：末条再跨记录")
     };
     assert_eq!(
-        spilling.rewritten.len(),
-        68,
-        "{leaf_containers} 片叶容器 + 叶容器之外的角色（由试发现数），恰好比一条记录装得下的 67 多一项：{:?}",
-        spilling.rewritten
-    );
-    assert_eq!(
-        u64::try_from(spilling.rewritten.len()).expect("68"),
+        u64::try_from(spilling.rewritten.len()).expect("一次发布的单元数装得进 u64"),
         NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH,
-        "68 就是一条记录装得下的项数加一（`JOURNAL_NAMED_ENTRIES_PER_RECORD` + 1）"
+        "{leaf_containers} 片叶容器 + 叶容器之外的角色（由试发现数），恰好是一条记录装得下的项数加一（`JOURNAL_NAMED_ENTRIES_PER_RECORD` + 1）：{:?}",
+        spilling.rewritten
     );
     assert_eq!(
         (
@@ -184,15 +179,26 @@ fn prepare(tag: &str) -> Prepared {
                 .len(),
             spilling.record.named.len()
         ),
-        (1, 67, 1),
-        "两条记录：装满 67 项的一条 + 跨出去的 1 项"
+        (
+            1,
+            usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD)
+                .expect("一条记录的点名项数装得进 usize"),
+            1
+        ),
+        "两条记录：装满一条记录的项数的一条 + 跨出去的 1 项"
     );
     let operations = pool.retained_operations();
     let (writes, segments) = writes_and_segments(&operations[operations_before..], &geometry());
     assert_eq!(
         segments.iter().map(Vec::len).collect::<Vec<_>>(),
-        vec![136, 4, 1, 2],
-        "只录这一次发布：68 个单元 × 2 盘、两条记录 × 2 盘、根槽 FUA、系统配置槽 × 2 盘"
+        vec![
+            2 * usize::try_from(NAMED_ENTRIES_OF_THE_SPILLING_PUBLISH)
+                .expect("点名项数装得进 usize"),
+            4,
+            1,
+            2
+        ],
+        "只录这一次发布：点名的每个单元 × 2 盘、两条记录 × 2 盘、根槽 FUA、系统配置槽 × 2 盘"
     );
     let root_indexes: Vec<usize> = writes
         .iter()
@@ -343,7 +349,7 @@ fn assert_clean(tally: &Layer0Tally) {
 
 /// L8 能枚举的全部状态：记录段（4 写）、根槽（1 写）逐个子集全展开，系统配置槽（2 写，原地覆写各取三态）全展开，
 /// 加全部持久那一个，共 25 个；
-/// 单元段 136 写不展开（2^136 − 1 个子集，只以整段持久进入后面的状态）。每个状态按两条记录落没落核发布边界。
+/// 单元段 130 写不展开（2^130 − 1 个子集，只以整段持久进入后面的状态）。每个状态按两条记录落没落核发布边界。
 #[test]
 fn every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all(
 ) {
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index b033308..422e0e7 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -464,15 +464,18 @@ fn reseal_unit(bytes: &mut [u8]) {
 fn reseal_journal_record(bytes: &mut [u8]) {
     let named = usize::try_from(u32::from_le_bytes(bytes[12..16].try_into().expect("4")))
         .expect("点名项数");
-    let payload_crc = crc32c(&bytes[311..311 + named * 56]);
+    let payload_crc = crc32c(
+        &bytes[JOURNAL_RECORD_HEADER_BYTES
+            ..JOURNAL_RECORD_HEADER_BYTES + named * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD],
+    );
     bytes[95..99].copy_from_slice(&payload_crc.to_le_bytes());
     let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, 4096, 46);
     bytes[46..78].copy_from_slice(&digest);
 }
 
-/// 下一条记录该带的反向链：这一条 311 字节头、校验和字段按 0 参与的 CRC-32C。
+/// 下一条记录该带的反向链：这一条 487 字节头、校验和字段按 0 参与的 CRC-32C。
 fn journal_chain_value(bytes: &[u8]) -> u32 {
-    let mut header = bytes[..311].to_vec();
+    let mut header = bytes[..JOURNAL_RECORD_HEADER_BYTES].to_vec();
     header[46..78].fill(0);
     crc32c(&header)
 }
@@ -919,8 +922,10 @@ const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: usize = 99;
 const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;
 /// 码 2 / 码 3 指针头里出生树的偏移。
 const POINTER_BIRTH_TREE_OFFSET: usize = 34;
-/// journal 记录头 311 字节，点名项从这里起、每项 56 字节（诞生代号在点名项偏移 37）。
-const JOURNAL_RECORD_HEADER_BYTES: usize = 311;
+/// journal 记录头 487 字节（D23（journal 的角色与格式） 已定项 4），点名项从这里起、每项 56 字节（诞生代号在点名项偏移 37）。
+const JOURNAL_RECORD_HEADER_BYTES: usize = 487;
+/// 点名项宽（D23（journal 的角色与格式） 已定项 17）。
+const JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD: usize = 56;
 
 /// 发布的根（`judged_root_index` 那次根槽写）的 (实例代号, checkpoint_txg)：根记录偏移 24、28。
 fn publish_root_identity(flow: &RecordedFlow) -> (u32, u64) {
@@ -1835,14 +1840,17 @@ fn resealed_publish_record_names_units_that_match_on_disk() {
         singlefs_core::checksum::wide_checksum_field_holds(record, 4096, 46),
         "记录头校验和重封过"
     );
-    let payload_crc = crc32c(&record[311..311 + named * 56]);
+    let payload_crc = crc32c(
+        &record[JOURNAL_RECORD_HEADER_BYTES
+            ..JOURNAL_RECORD_HEADER_BYTES + named * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD],
+    );
     assert_eq!(
         payload_crc,
         u32::from_le_bytes(record[95..99].try_into().expect("4")),
         "记录载荷校验和重封过"
     );
     for entry in 0..named {
-        let o = 311 + entry * 56;
+        let o = JOURNAL_RECORD_HEADER_BYTES + entry * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD;
         let class = record[o + 28];
         let length = if class == 2 { 16384 } else { 32768 };
         for copy in 0..2 {
@@ -1940,7 +1948,7 @@ fn verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several(
 /// 在扫描内核读回违例位之后种第 128 位（下标 127）：
 /// 两份种过的内核判出的每个状态，违例位等于没种的那一份判出的再并上种的位（第 65 位每个状态都有，第 128 位只在扫描内核判完的状态上有），
 /// 其余的字逐字相同。违例位的高 64 位在走读内核写出、扫描内核读回又写出、Rust 读出这三处任一处被截掉，这条红。
-/// 池级 checker 今天 65 条（下标 0..=64，内核全部判），种的是第 66 位（下标 65）。
+/// 池级 checker 今天 66 条（下标 0..=65；内核判前 65 条，下标 65 的 I-3.5 内核还没判，`crash_judge_gpu` 的「内核还没判」那张表），种的是第 67 位（下标 66）。
 #[test]
 fn a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels() {
     const FIRST_BIT_OF_THE_HIGH_HALF: usize = 64;
diff --git a/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs b/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
index 844b43b..94a938f 100644
--- a/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
@@ -205,9 +205,12 @@ fn the_pipeline_with_gpu_unit_checks_reports_no_red_and_no_disagreement_on_the_h
         ],
     };
     let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
-        .join("../..")
+        .join("../../Cargo.lock")
         .canonicalize()
-        .expect("仓根");
+        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
+        .parent()
+        .expect("Cargo.lock 所在的目录就是仓根")
+        .to_path_buf();
     let judging = PipelineJudgingCode::of_the_judges(
         &root,
         &Layer0ToolchainIdentity::of_the_cargo_running_this_test(),
diff --git b/crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs b/crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs
new file mode 100644
index 0000000..3f4a3ea
--- /dev/null
+++ b/crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs
@@ -0,0 +1,1001 @@
+//! 池级 checker 里程碑三第 8 批的几处判法，每条一份镜像（干净的，或改一处之后重封、沿引用链补到根槽的）：
+//! - I-3.5（引用区间的精确性）的出现规则：诞生代号下界 (a)、按树继承下界（TF）、再次出现也判（β）、候选集有洞（gap）、
+//!   实例表行接得上就照判（gap-row）、接不上时只放过用户可见树里的单元（N3）——`research/prompts/m3-birth-floor-r3-main-verification.md`，
+//!   用户 2026-10-03 定丙；措辞是 `.claude/kb/invariants.md` I-3.5 那一行；
+//! - I-9.25（树表条目的 previous_snapshot_txg 取值）：树表被几版共享时每一版拿自己的 txg 各判一次（回到 kb 字面）；
+//! - I-8.10（记录新根段与同代根相同）：七段里新加的实例表指针、分配记录树根指针、卸载记号三段，与写行那次发布的记录（RR3 的观察者）；
+//! - 记录新根段卸载记号位 0 之外有位为 1：池级 checker 与恢复收下同一批记录（m3-root-ring-code-r1 判决 RR1）。
+mod common;
+
+use common::{
+    build_pool, format_pool, parameters, publish_overwrite_in_process,
+    rewrite_journal_record_and_rechain_the_records_after_it, tear_the_root_slot_written_for_txg,
+    BuiltPool, FIXED_WRITE_TIME_SECONDS,
+};
+use singlefs_checker::image::{
+    InvariantVerdict, LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
+    PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, REFERENCE_INTERVALS_ARE_EXACT,
+    TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
+};
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
+use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
+use singlefs_core::journal::{back_chain_of, record_offset, JournalRecord};
+use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
+use singlefs_core::pointer::NodePointer;
+use singlefs_core::records::{TREE_KIND_ACCOUNTING, TREE_KIND_EXTENT, TREE_KIND_INODE};
+use singlefs_core::recovery::{
+    choose_system_configuration, readable_roots, scan_journal, PoolReader,
+};
+use singlefs_core::root_record::UnmountMarker;
+use singlefs_core::transaction::{
+    publish_without_units, PoolWriter, TransactionOutput, ZeroUnitPublishPlan,
+};
+use singlefs_core::unit::{
+    unit_filesystem_identifier, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
+    UNIT_MAGIC,
+};
+use singlefs_format::{
+    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES,
+};
+use singlefs_harness::memory_pool::MemoryPool;
+
+const SLOT: u64 = 16384;
+const DEVICES: [u32; 2] = [0, 1];
+/// 单元区第一个槽（`common::build_pool` 那份 4 GiB 两盘池：mkfs 的实例表单元就落在这里）。
+const FIRST_UNIT_SLOT: u64 = 50176;
+/// 找单元时每个槽读开头一个扇区看单元头（内存池的读按扇区对齐，`MemoryPool` 的 `PoolReader::read`）。
+const SECTOR_BYTES_READ_FOR_A_UNIT_HEADER: usize = 512;
+/// 找单元时往后扫几个槽：这几份镜像上写过的单元都在这一段里（最长那段历史是回退那一份，十来次发布）。
+const UNIT_SLOTS_SCANNED: u64 = 2048;
+/// 根记录的自证校验和在 138 起 32 字节，罩整个 512 字节的根槽（D22（单元原子性怎么合成） 已定项 7）。
+const ROOT_SLOT_BYTES: usize = 512;
+const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
+/// journal 记录头校验和在 46 起 32 字节，罩整条 4096（D23（journal 的角色与格式） 已定项 13）。
+const JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET: usize = 46;
+/// 新根段末尾卸载记号那 4 字节在记录里的偏移：新根段 99 起，前六段 360 字节之后（D23（journal 的角色与格式） 已定项 15）。
+const JOURNAL_RECORD_UNMOUNT_MARKER_OFFSET: usize = 99 + 360;
+/// 码 2 节点：key 宽在 51，条目数在 82 + 2k；树表单元（k = 8）的条目区从 131 起、每条 200（D8（核心索引结构） 已定项 8）。
+const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
+const TREE_TABLE_ENTRY_START: usize = 131;
+const TREE_TABLE_ENTRY_BYTES_IN_THE_UNIT: usize = 200;
+const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
+const TREE_TABLE_ENTRY_ROOT_POINTER: std::ops::Range<usize> = 14..100;
+const TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET: usize = 100;
+/// 码 2 / 码 3 指针头里出生 txg 的偏移（D19（块指针的结构与宽度预算） 已定项 11 的偏移表）。
+const POINTER_BIRTH_TXG_OFFSET: usize = 42;
+/// 单元头里诞生代号的偏移（D18（块里携带什么信息） 已定项 7 / 已定项 11）：码 1 在 75，码 3 在 73，码 2 在 52 + 2k。
+const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
+const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
+const INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS: usize = 52;
+/// 恢复择根之上一次发布都不施加时 I-4.1 报的不适用理由。
+const NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT: &str =
+    "所选根之上一次发布都不施加：重放之后那一版就是所选根，它可不可遍历归 I-7.2";
+
+fn read(image: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
+    PoolReader::read(
+        image,
+        DeviceIdentity(device),
+        DeviceOffsetInBytes(offset),
+        length,
+    )
+    .expect("这几份镜像上读的都是盘内的落点")
+}
+
+fn write(image: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
+    image
+        .devices
+        .get_mut(&DeviceIdentity(device))
+        .expect("这几份镜像都是两块盘")
+        .write(DeviceOffsetInBytes(offset), bytes);
+}
+
+fn verdict(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(identifier, _)| *identifier == invariant)
+        .expect("池级 checker 的清单里有这一条")
+        .1
+}
+
+/// 这一条判违例，第一处违例的说明里有 `expected_in_the_detail`。
+fn assert_violated_mentioning(image: &MemoryPool, invariant: &str, expected_in_the_detail: &str) {
+    match verdict(image, invariant) {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.contains(expected_in_the_detail),
+            "{invariant} 判违例，说明里该有「{expected_in_the_detail}」：{detail}"
+        ),
+        other => {
+            panic!("{invariant} 应当判违例（说明里有「{expected_in_the_detail}」），实际 {other:?}")
+        }
+    }
+}
+
+/// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
+fn reseal_unit(bytes: &mut [u8]) {
+    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
+    let (header_end, payload_crc_offset) = match bytes[6] {
+        UNIT_CLASS_DATA => (105usize, 101usize),
+        UNIT_CLASS_PACKED => (107, 89),
+        UNIT_CLASS_INDEX_NODE => (86 + 2 * key_width, 76 + 2 * key_width),
+        other_class => panic!("这几份镜像上只有码 1、2、3 的单元，读到类 {other_class}"),
+    };
+    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
+    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
+    singlefs_core::unit::seal_header_checksum(bytes, header_end);
+}
+
+fn birth_txg_offset_in_the_unit_header(bytes: &[u8]) -> usize {
+    match bytes[6] {
+        UNIT_CLASS_DATA => DATA_UNIT_BIRTH_TXG_OFFSET,
+        UNIT_CLASS_PACKED => PACKED_UNIT_BIRTH_TXG_OFFSET,
+        UNIT_CLASS_INDEX_NODE => {
+            INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS
+                + 2 * usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET])
+        }
+        other_class => panic!("这几份镜像上只有码 1、2、3 的单元，读到类 {other_class}"),
+    }
+}
+
+fn birth_txg_in_the_unit_header(bytes: &[u8]) -> u64 {
+    let offset = birth_txg_offset_in_the_unit_header(bytes);
+    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
+}
+
+fn set_birth_txg_in_the_unit_header(bytes: &mut [u8], birth_txg: u64) {
+    let offset = birth_txg_offset_in_the_unit_header(bytes);
+    bytes[offset..offset + 8].copy_from_slice(&birth_txg.to_le_bytes());
+}
+
+/// 盘 0 单元区开头那一段里写过的每个单元：(起点槽, 字节数)。两盘同槽（D2（RAID 条带策略） 已定项 10），读一块盘就够。
+/// 迭代上界是扫的槽数；码 1 / 码 3 的单元占两槽，第二槽不当成单元头读。
+fn units_on_disk(image: &MemoryPool) -> Vec<(u64, usize)> {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
+    let mut units = Vec::new();
+    let mut slot = FIRST_UNIT_SLOT;
+    while slot < FIRST_UNIT_SLOT + UNIT_SLOTS_SCANNED {
+        let header = read(image, 0, slot * SLOT, SECTOR_BYTES_READ_FOR_A_UNIT_HEADER);
+        if header[..4] != UNIT_MAGIC {
+            slot += 1;
+            continue;
+        }
+        if header[6] == UNIT_CLASS_INDEX_NODE {
+            units.push((slot, node_bytes));
+            slot += 1;
+        } else {
+            units.push((slot, data_unit_bytes));
+            slot += 2;
+        }
+    }
+    units
+}
+
+/// 根环全部槽：(盘, 偏移)。区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，每区 8 槽、槽距 4096。
+fn root_slots() -> Vec<(u32, u64)> {
+    (0..3u64)
+        .flat_map(|region| {
+            (0..8u64).map(move |slot| {
+                (
+                    [0u32, 1, 0][usize::try_from(region).expect("区域号")],
+                    64 * SLOT + region * 3 * (1 << 20) + slot * 4096,
+                )
+            })
+        })
+        .collect()
+}
+
+/// 一条位置条目：设备 4 + 槽号 6 + 整单元校验和 4。
+fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
+    [
+        device.to_le_bytes().to_vec(),
+        slot.to_le_bytes()[..6].to_vec(),
+        checksum.to_le_bytes().to_vec(),
+    ]
+    .concat()
+}
+
+fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
+    let mut changed = false;
+    let mut index = 0;
+    while index + needle.len() <= haystack.len() {
+        if haystack[index..index + needle.len()] == *needle {
+            haystack[index..index + needle.len()].copy_from_slice(replacement);
+            changed = true;
+            index += needle.len();
+        } else {
+            index += 1;
+        }
+    }
+    changed
+}
+
+/// 一个单元（两盘同槽）的整单元校验和从 old 变成 new：把引用它的位置条目逐个补上（单元里的、根槽里的），被补的单元重封、
+/// 再往上补，直到根槽；最后把每条根照抄进同 (实例, txg) 的 journal 记录的新根段（`common` 的那一份），I-8.10 不跟着红。
+/// 迭代上界：每补一个单元，它自己的新校验和进待补的表一次；引用链是有向无环的（子先于父写），补到根槽就停。
+fn propagate(image: &mut MemoryPool, units: &[(u64, usize)], mut changes: Vec<(u64, u32, u32)>) {
+    while let Some((slot, old, new)) = changes.pop() {
+        if old == new {
+            continue;
+        }
+        for (container, length) in units.iter().copied() {
+            if container == slot {
+                continue;
+            }
+            let before = read(image, 0, container * SLOT, length);
+            let mut bytes = before.clone();
+            let mut touched = false;
+            for device in DEVICES {
+                touched |= replace_all(
+                    &mut bytes,
+                    &location_pattern(device, slot, old),
+                    &location_pattern(device, slot, new),
+                );
+            }
+            if touched {
+                reseal_unit(&mut bytes);
+                for device in DEVICES {
+                    write(image, device, container * SLOT, &bytes);
+                }
+                changes.push((
+                    container,
+                    crc32_castagnoli(&before),
+                    crc32_castagnoli(&bytes),
+                ));
+            }
+        }
+        for (device, offset) in root_slots() {
+            let mut bytes = read(image, device, offset, ROOT_SLOT_BYTES);
+            let mut touched = false;
+            for location_device in DEVICES {
+                touched |= replace_all(
+                    &mut bytes,
+                    &location_pattern(location_device, slot, old),
+                    &location_pattern(location_device, slot, new),
+                );
+            }
+            if touched {
+                let digest = wide_checksum_with_field_zeroed(
+                    &bytes,
+                    ROOT_SLOT_BYTES,
+                    ROOT_RECORD_CHECKSUM_OFFSET,
+                );
+                bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + 32]
+                    .copy_from_slice(&digest);
+                write(image, device, offset, &bytes);
+            }
+        }
+    }
+    common::copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
+}
+
+/// 改一个单元（两盘一起改）：改完重封它的两道校验和，新的整单元校验和沿引用链补到根槽。
+fn rewrite_unit_and_propagate(image: &mut MemoryPool, slot: u64, change: impl Fn(&mut Vec<u8>)) {
+    let units = units_on_disk(image);
+    let length = units
+        .iter()
+        .find(|(start, _)| *start == slot)
+        .unwrap_or_else(|| panic!("槽 {slot} 是盘上写过的一个单元的起点"))
+        .1;
+    let before = read(image, 0, slot * SLOT, length);
+    let mut bytes = before.clone();
+    change(&mut bytes);
+    reseal_unit(&mut bytes);
+    for device in DEVICES {
+        write(image, device, slot * SLOT, &bytes);
+    }
+    propagate(
+        image,
+        &units,
+        vec![(slot, crc32_castagnoli(&before), crc32_castagnoli(&bytes))],
+    );
+}
+
+/// 两盘同槽：一条指针两条位置条目落在同一个槽号上，按一个槽号改两盘。
+fn slot_shared_by_both_copies(pointer: &NodePointer) -> u64 {
+    assert_eq!(
+        pointer.locations[0].slot, pointer.locations[1].slot,
+        "两盘同槽：这份镜像的改法按一个槽号改两盘"
+    );
+    pointer.locations[0].slot.0
+}
+
+fn tree_table_slot(version: &TransactionOutput) -> u64 {
+    slot_shared_by_both_copies(&version.root.tree_table)
+}
+
+fn root_slot_of_the_tree(version: &TransactionOutput, kind: u16) -> u64 {
+    let entry = version
+        .tree_table_entries
+        .iter()
+        .find(|entry| entry.kind == kind)
+        .unwrap_or_else(|| panic!("这一版的树表里有种类 {kind} 的那一条"));
+    slot_shared_by_both_copies(&entry.root)
+}
+
+fn data_unit_slot(version: &TransactionOutput) -> u64 {
+    let [pointer] = version.data_pointers.as_slice() else {
+        panic!(
+            "这几版文件都只有一个数据单元：{} 条数据指针",
+            version.data_pointers.len()
+        );
+    };
+    assert_eq!(
+        pointer.locations[0].slot, pointer.locations[1].slot,
+        "两盘同槽：数据单元按一个槽号改两盘"
+    );
+    pointer.locations[0].slot.0
+}
+
+/// 一片树表单元里种类为 `kind` 的那一条条目的起点。
+fn tree_table_entry_offset_of_kind(tree_table: &[u8], kind: u16) -> usize {
+    let key_width = usize::from(tree_table[INDEX_NODE_KEY_WIDTH_OFFSET]);
+    let entry_count_offset = 82 + 2 * key_width;
+    let entry_count = usize::from(u16::from_le_bytes([
+        tree_table[entry_count_offset],
+        tree_table[entry_count_offset + 1],
+    ]));
+    (0..entry_count)
+        .map(|index| TREE_TABLE_ENTRY_START + TREE_TABLE_ENTRY_BYTES_IN_THE_UNIT * index)
+        .find(|offset| {
+            let kind_offset = offset + TREE_TABLE_ENTRY_KIND_OFFSET;
+            u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]]) == kind
+        })
+        .unwrap_or_else(|| panic!("树表里有种类 {kind} 的那一条"))
+}
+
+/// 覆盖写那几版的内容：每版一个数据单元装得下，字节各不相同。
+fn content_of_the_overwrite(seed: usize) -> Vec<u8> {
+    (0..4100usize)
+        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
+        .collect()
+}
+
+/// 新池新建文件 A (1, 3) 之后同一个进程里再覆盖写 `overwrites` 次（B (1, 4)、C (1, 5)……）：交回池与每一版（A 在最前）。
+fn pool_through_overwrites(tag: &str, overwrites: usize) -> (BuiltPool, Vec<TransactionOutput>) {
+    let mut pool = build_pool(tag);
+    let mut versions = vec![pool.output.clone()];
+    for overwrite in 0..overwrites {
+        let previous = versions.last().expect("至少有 A").clone();
+        let next = publish_overwrite_in_process(
+            &mut pool,
+            &previous,
+            &content_of_the_overwrite(overwrite + 3),
+            FIXED_WRITE_TIME_SECONDS + 60 * (u64::try_from(overwrite).expect("次数") + 1),
+            InstanceGeneration(1),
+        )
+        .expect("覆盖写");
+        pool.output = next.clone();
+        versions.push(next);
+    }
+    (pool, versions)
+}
+
+/// 挂着的时候回退到 A 之后的干净镜像：A (1, 3)、B (1, 4) → 重开取号 2、写行、暖机两次、C (2, 8) → 一次向前发布 D (2, 9)，
+/// 新根的 extent 树、inode 树取 A 的，树表、记账、映射另写（D23（journal 的角色与格式） 已定项 14；`checker_known_bad_images.rs` 同一段历史）。
+struct ImageRolledBackToTheFirstVersion {
+    image: MemoryPool,
+    first: TransactionOutput,
+    version_before_the_rollback: TransactionOutput,
+    rollback_publish: TransactionOutput,
+}
+
+fn image_rolled_back_to_the_first_version(tag: &str) -> ImageRolledBackToTheFirstVersion {
+    let (mut pool, versions) = pool_through_overwrites(tag, 1);
+    let first = versions[0].clone();
+    let mut devices_reopened_for_the_second_instance = pool.reopen_recorded();
+    let mounted = mount_writable(&parameters(), &mut devices_reopened_for_the_second_instance)
+        .expect("重开取号 2");
+    pool.devices = Some(devices_reopened_for_the_second_instance);
+    pool.allocator = mounted.allocator;
+    let current = mounted
+        .current
+        .into_file_version()
+        .expect("B 之后重开，现行那一版带文件");
+    let third = publish_overwrite_in_process(
+        &mut pool,
+        &current,
+        &content_of_the_overwrite(11),
+        FIXED_WRITE_TIME_SECONDS + 120,
+        InstanceGeneration(2),
+    )
+    .expect("覆盖写 C");
+    let mut rollback_publish = third.clone();
+    let rollback_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    roll_back_by_a_forward_publish(
+        &rollback_parameters,
+        devices,
+        &mut pool.allocator,
+        &mut rollback_publish,
+        RollbackTarget {
+            instance: first.root.instance,
+            checkpoint_txg: first.root.checkpoint_txg,
+        },
+    )
+    .expect("挂着的时候回退到 A");
+    assert_eq!(
+        rollback_publish.root.checkpoint_txg.0,
+        third.root.checkpoint_txg.0 + 1,
+        "回退那次发布紧接在 C 之后"
+    );
+    assert_eq!(
+        root_slot_of_the_tree(&rollback_publish, TREE_KIND_EXTENT),
+        root_slot_of_the_tree(&first, TREE_KIND_EXTENT),
+        "回退那一版的 extent 树根就是 A 的"
+    );
+    ImageRolledBackToTheFirstVersion {
+        image: pool.memory_pool(),
+        first,
+        version_before_the_rollback: third,
+        rollback_publish,
+    }
+}
+
+/// (a)：A (1, 3) → B (1, 4) 的干净镜像上 I-3.5 成立；B 新写的数据单元诞生代号改小到恰好等于 A 的 txg（J3-c），I-3.5 红，
+/// 违例说的就是那个数据单元。「严格大于」写成「不小于」的这一条红。
+#[test]
+fn a_unit_new_in_an_overwrite_born_at_the_txg_of_the_version_before_reddens_the_reference_interval_invariant(
+) {
+    let (pool, versions) =
+        pool_through_overwrites("reference-interval-born-at-the-previous-txg", 1);
+    let (first, second) = (&versions[0], &versions[1]);
+    let clean = pool.memory_pool();
+    assert_eq!(
+        verdict(&clean, REFERENCE_INTERVALS_ARE_EXACT),
+        InvariantVerdict::Holds,
+        "A → B 的干净镜像上每个新出现的单元都生在前一版之后"
+    );
+    let mut image = clean.clone();
+    let data_unit = data_unit_slot(second);
+    rewrite_unit_and_propagate(&mut image, data_unit, |bytes| {
+        assert_eq!(
+            birth_txg_in_the_unit_header(bytes),
+            second.root.checkpoint_txg.0,
+            "B 的数据单元生在 B 那一版"
+        );
+        set_birth_txg_in_the_unit_header(bytes, first.root.checkpoint_txg.0);
+    });
+    assert_violated_mentioning(
+        &image,
+        REFERENCE_INTERVALS_ARE_EXACT,
+        &format!("槽 {data_unit}"),
+    );
+}
+
+/// TF（按树继承下界）：挂着的时候回退到 A 之后，回退那一版 D 引用 A 的 extent 树与 inode 树（诞生代号都是 3），C 不引用它们；
+/// 它们落在 D 的用户可见树里、根指针出生 txg 3 小于 C 的 txg、子树里诞生代号都不大于 3：合法回退，I-3.5 成立。去掉 (b) 这一条红。
+#[test]
+fn a_rollback_inheriting_the_user_visible_trees_of_its_target_whole_holds_the_reference_interval_invariant(
+) {
+    let rolled_back = image_rolled_back_to_the_first_version("reference-interval-rollback-holds");
+    assert!(
+        rolled_back.first.root.checkpoint_txg.0
+            < rolled_back
+                .version_before_the_rollback
+                .root
+                .checkpoint_txg
+                .0,
+        "回退目标比回退之前那一版老"
+    );
+    assert_eq!(
+        verdict(&rolled_back.image, REFERENCE_INTERVALS_ARE_EXACT),
+        InvariantVerdict::Holds,
+        "回退那一版整棵继承 A 的用户可见树：(b) 罩住它们"
+    );
+}
+
+/// TF 的「严格小于」：A → B 的镜像上把 B 的数据单元、B 的 extent 树根两个单元头的诞生代号与 B 树表里 extent 树那条根指针的出生 txg
+/// 一起改成 A 的 txg（3）：那棵树的根指针出生 txg 等于前一版的 txg，不算整棵继承下来，I-3.5 红。「严格小于」写成「不大于」的这一条红。
+#[test]
+fn a_user_visible_tree_whose_root_pointer_was_born_at_the_txg_of_the_version_before_reddens_the_reference_interval_invariant(
+) {
+    let (pool, versions) =
+        pool_through_overwrites("reference-interval-root-born-at-the-previous-txg", 1);
+    let (first, second) = (&versions[0], &versions[1]);
+    let previous_txg = first.root.checkpoint_txg.0;
+    let mut image = pool.memory_pool();
+    for unit in [
+        data_unit_slot(second),
+        root_slot_of_the_tree(second, TREE_KIND_EXTENT),
+    ] {
+        rewrite_unit_and_propagate(&mut image, unit, |bytes| {
+            set_birth_txg_in_the_unit_header(bytes, previous_txg);
+        });
+    }
+    rewrite_unit_and_propagate(&mut image, tree_table_slot(second), |bytes| {
+        let pointer_birth_txg = tree_table_entry_offset_of_kind(bytes, TREE_KIND_EXTENT)
+            + TREE_TABLE_ENTRY_ROOT_POINTER.start
+            + POINTER_BIRTH_TXG_OFFSET;
+        bytes[pointer_birth_txg..pointer_birth_txg + 8]
+            .copy_from_slice(&previous_txg.to_le_bytes());
+    });
+    assert_violated_mentioning(&image, REFERENCE_INTERVALS_ARE_EXACT, "用户可见树 Some(");
+}
+
+/// TF 的「压得住」：回退到 A 之后，A 的数据单元头里的诞生代号改成 4（大于 extent 树根指针的出生 txg 3、不大于回退之前那一版的 txg）：
+/// 那棵子树里有单元比根新，不算整棵继承下来，I-3.5 红，违例说的就是那个数据单元。去掉「子树里诞生代号都不大于根」那一半这一条红。
+#[test]
+fn a_unit_in_an_inherited_user_visible_tree_born_after_its_root_reddens_the_reference_interval_invariant(
+) {
+    let rolled_back =
+        image_rolled_back_to_the_first_version("reference-interval-inherited-tree-not-held-down");
+    let mut image = rolled_back.image.clone();
+    let data_unit = data_unit_slot(&rolled_back.first);
+    let birth_after_the_root = rolled_back.first.root.checkpoint_txg.0 + 1;
+    assert!(
+        birth_after_the_root
+            < rolled_back
+                .version_before_the_rollback
+                .root
+                .checkpoint_txg
+                .0,
+        "改成的诞生代号不大于回退之前那一版的 txg：(a) 罩不住它"
+    );
+    rewrite_unit_and_propagate(&mut image, data_unit, |bytes| {
+        assert_eq!(
+            birth_txg_in_the_unit_header(bytes),
+            rolled_back.first.root.checkpoint_txg.0,
+            "A 的数据单元生在 A 那一版"
+        );
+        set_birth_txg_in_the_unit_header(bytes, birth_after_the_root);
+    });
+    assert_violated_mentioning(
+        &image,
+        REFERENCE_INTERVALS_ARE_EXACT,
+        &format!("槽 {data_unit}"),
+    );
+}
+
+/// β（再次出现也判）：回退到 A 之后，把回退那一版 D 树表里记账树那条的根指针换成 A 树表里那一条的——A 的记账树根（诞生代号 3）
+/// 在 A 里出现过、在 C 里没有、在 D 里再出现，它不在用户可见树里，I-3.5 红。只判第一次出现（以前哪一版引用过就不判）这一条红。
+#[test]
+fn a_fixed_point_tree_of_the_rollback_pointed_back_at_the_rollback_target_reddens_the_reference_interval_invariant(
+) {
+    let rolled_back =
+        image_rolled_back_to_the_first_version("reference-interval-reappearing-accounting-root");
+    let mut image = rolled_back.image.clone();
+    let first_tree_table = read(
+        &image,
+        0,
+        tree_table_slot(&rolled_back.first) * SLOT,
+        usize::try_from(NODE_BYTES).expect("16384"),
+    );
+    let accounting_root_pointer_of_the_first: Vec<u8> = {
+        let entry = tree_table_entry_offset_of_kind(&first_tree_table, TREE_KIND_ACCOUNTING);
+        first_tree_table
+            [entry + TREE_TABLE_ENTRY_ROOT_POINTER.start..entry + TREE_TABLE_ENTRY_ROOT_POINTER.end]
+            .to_vec()
+    };
+    let accounting_root_of_the_first =
+        root_slot_of_the_tree(&rolled_back.first, TREE_KIND_ACCOUNTING);
+    assert_ne!(
+        root_slot_of_the_tree(&rolled_back.rollback_publish, TREE_KIND_ACCOUNTING),
+        accounting_root_of_the_first,
+        "回退那一版本来另写了记账树"
+    );
+    rewrite_unit_and_propagate(
+        &mut image,
+        tree_table_slot(&rolled_back.rollback_publish),
+        |bytes| {
+            let entry = tree_table_entry_offset_of_kind(bytes, TREE_KIND_ACCOUNTING);
+            bytes[entry + TREE_TABLE_ENTRY_ROOT_POINTER.start
+                ..entry + TREE_TABLE_ENTRY_ROOT_POINTER.end]
+                .copy_from_slice(&accounting_root_pointer_of_the_first);
+        },
+    );
+    assert_violated_mentioning(
+        &image,
+        REFERENCE_INTERVALS_ARE_EXACT,
+        &format!("槽 {accounting_root_of_the_first}（用户可见树 None）"),
+    );
+}
+
+/// A (1, 3) → B (1, 4) → C (1, 5) 之后 B 的根槽改坏：候选集里 A 与 C 之间有洞，只有实例 1、实例表一行都没有，接不上。
+fn image_with_a_hole_the_instance_table_rows_do_not_bridge(
+    tag: &str,
+    change_before_the_root_slot_is_torn: impl Fn(&mut MemoryPool, &[TransactionOutput]),
+) -> MemoryPool {
+    let (pool, versions) = pool_through_overwrites(tag, 2);
+    let mut image = pool.memory_pool();
+    change_before_the_root_slot_is_torn(&mut image, &versions);
+    tear_the_root_slot_written_for_txg(&mut image, versions[1].root.checkpoint_txg.0);
+    image
+}
+
+/// gap（缺口）：同一个改法——C 新写的数据单元诞生代号改小到 A 的 txg——在没有洞的镜像上红（与 B 相邻），在 B 的根槽读不出、
+/// 实例表行接不上的那份上不判（N3 只放过用户可见树里的单元：盲区，`.claude/kb/invariants.md` I-3.5 射程），I-3.5 成立。
+/// 去掉缺口那一条（洞上的用户可见树单元照判）这一条红。
+#[test]
+fn in_a_hole_the_instance_table_rows_do_not_bridge_a_user_visible_unit_born_before_the_hole_is_not_judged(
+) {
+    let lower_the_data_unit_of_the_third_version =
+        |image: &mut MemoryPool, versions: &[TransactionOutput]| {
+            let data_unit = data_unit_slot(&versions[2]);
+            let first_txg = versions[0].root.checkpoint_txg.0;
+            rewrite_unit_and_propagate(image, data_unit, |bytes| {
+                set_birth_txg_in_the_unit_header(bytes, first_txg);
+            });
+        };
+    let (pool, versions) =
+        pool_through_overwrites("reference-interval-gap-user-visible-no-hole", 2);
+    let mut without_the_hole = pool.memory_pool();
+    lower_the_data_unit_of_the_third_version(&mut without_the_hole, &versions);
+    assert_violated_mentioning(
+        &without_the_hole,
+        REFERENCE_INTERVALS_ARE_EXACT,
+        &format!("槽 {}", data_unit_slot(&versions[2])),
+    );
+    let clean_with_the_hole = image_with_a_hole_the_instance_table_rows_do_not_bridge(
+        "reference-interval-gap-clean",
+        |_image, _versions| {},
+    );
+    assert_eq!(
+        verdict(&clean_with_the_hole, REFERENCE_INTERVALS_ARE_EXACT),
+        InvariantVerdict::Holds,
+        "B 的根槽读不出的干净镜像：A → C 那一对照 N3 判，树表、记账那几个单元都生在 A 之后"
+    );
+    let with_the_hole = image_with_a_hole_the_instance_table_rows_do_not_bridge(
+        "reference-interval-gap-user-visible",
+        lower_the_data_unit_of_the_third_version,
+    );
+    assert_eq!(
+        verdict(&with_the_hole, REFERENCE_INTERVALS_ARE_EXACT),
+        InvariantVerdict::Holds,
+        "洞上实例表行接不上：C 的数据单元落在用户可见树里，不判"
+    );
+}
+
+/// N3：同一份有洞、实例表行接不上的镜像上，C 新写的树表单元（不在用户可见树里）诞生代号改小到 A 的 txg：照 (a) 判，I-3.5 红。
+/// 洞上整对都不判这一条红。
+#[test]
+fn in_a_hole_the_instance_table_rows_do_not_bridge_a_fixed_point_unit_born_before_the_hole_reddens_the_reference_interval_invariant(
+) {
+    let tree_table_of_the_third_version = std::cell::Cell::new(0u64);
+    let image = image_with_a_hole_the_instance_table_rows_do_not_bridge(
+        "reference-interval-gap-fixed-point",
+        |image, versions| {
+            let tree_table = tree_table_slot(&versions[2]);
+            tree_table_of_the_third_version.set(tree_table);
+            let first_txg = versions[0].root.checkpoint_txg.0;
+            rewrite_unit_and_propagate(image, tree_table, |bytes| {
+                set_birth_txg_in_the_unit_header(bytes, first_txg);
+            });
+        },
+    );
+    assert_violated_mentioning(
+        &image,
+        REFERENCE_INTERVALS_ARE_EXACT,
+        &format!(
+            "槽 {}（用户可见树 None）",
+            tree_table_of_the_third_version.get()
+        ),
+    );
+}
+
+/// gap-row：A (1, 3) 之后重开取号 2（写行、暖机）、覆盖写 D；实例 2 在 D 之前的几条根槽都改坏。候选集里 A 与 D 之间有洞，
+/// 但 D 指着的实例表有一行 (1, 3)、D 的实例 2 紧接实例 1：接得上，这一对照常判。D 新写的数据单元诞生代号改小到 A 的 txg，I-3.5 红。
+/// 去掉 gap-row（接得上也当接不上、只判用户可见树之外的）这一条红。
+#[test]
+fn a_hole_the_instance_table_rows_bridge_judges_the_user_visible_units_too() {
+    let mut pool = build_pool("reference-interval-gap-row");
+    let first = pool.output.clone();
+    let mut devices_reopened_for_the_second_instance = pool.reopen_recorded();
+    let mounted = mount_writable(&parameters(), &mut devices_reopened_for_the_second_instance)
+        .expect("重开取号 2：写行、暖机");
+    pool.devices = Some(devices_reopened_for_the_second_instance);
+    pool.allocator = mounted.allocator;
+    let current = mounted
+        .current
+        .into_file_version()
+        .expect("A 之后重开，现行那一版带文件");
+    let overwrite = publish_overwrite_in_process(
+        &mut pool,
+        &current,
+        &content_of_the_overwrite(17),
+        FIXED_WRITE_TIME_SECONDS + 120,
+        InstanceGeneration(2),
+    )
+    .expect("实例 2 覆盖写 D");
+    let txgs_of_the_second_instance_before_the_overwrite =
+        first.root.checkpoint_txg.0 + 1..overwrite.root.checkpoint_txg.0;
+    assert!(
+        !txgs_of_the_second_instance_before_the_overwrite.is_empty(),
+        "实例 2 在 D 之前写过行"
+    );
+    let tear_the_second_instance_before_the_overwrite = |image: &mut MemoryPool| {
+        for txg in txgs_of_the_second_instance_before_the_overwrite.clone() {
+            tear_the_root_slot_written_for_txg(image, txg);
+        }
+    };
+    let mut clean = pool.memory_pool();
+    tear_the_second_instance_before_the_overwrite(&mut clean);
+    assert_eq!(
+        verdict(&clean, REFERENCE_INTERVALS_ARE_EXACT),
+        InvariantVerdict::Holds,
+        "实例 2 写行与暖机的根槽读不出的干净镜像：A → D 那一对照常判，D 新写的都生在 A 之后"
+    );
+    let mut image = pool.memory_pool();
+    let data_unit = data_unit_slot(&overwrite);
+    rewrite_unit_and_propagate(&mut image, data_unit, |bytes| {
+        set_birth_txg_in_the_unit_header(bytes, first.root.checkpoint_txg.0);
+    });
+    tear_the_second_instance_before_the_overwrite(&mut image);
+    assert_violated_mentioning(
+        &image,
+        REFERENCE_INTERVALS_ARE_EXACT,
+        &format!("槽 {data_unit}"),
+    );
+}
+
+/// 新池新建文件 A (1, 3) 之后同一实例发一次零单元发布 (1, 4)：它照抄 A 的树表指针与映射根指针，与 A 共享那两棵。交回池、A 与那一版。
+fn pool_with_a_zero_unit_publish_after_the_first_file(
+    tag: &str,
+) -> (
+    BuiltPool,
+    TransactionOutput,
+    singlefs_core::transaction::VersionWithoutFilePublishOutput,
+) {
+    let mut pool = build_pool(tag);
+    let first = pool.output.clone();
+    let publish_parameters = parameters();
+    let zero_unit = {
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        publish_without_units(
+            &mut writer,
+            &first.root,
+            ZeroUnitPublishPlan {
+                txg: CheckpointTxg(first.root.checkpoint_txg.0 + 1),
+                counter: first.record.counter + 1,
+                instance: first.root.instance,
+                back_chain: back_chain_of(&first.record_bytes),
+                rollback_floor: first.root.rollback_floor,
+                tree_identifier_watermark: first.root.tree_identifier_watermark,
+            },
+        )
+        .expect("A 之后一次零单元发布")
+    };
+    assert_eq!(
+        (zero_unit.root.tree_table, zero_unit.root.mapping_root),
+        (first.root.tree_table, first.root.mapping_root),
+        "零单元发布与 A 共享 A 那片树表与映射根"
+    );
+    (pool, first, zero_unit)
+}
+
+/// 共享树表的那一版照样把共享的映射根并进它引用的单元：A 与它之后的零单元发布 (1, 4) 引用同一批单元，相邻两版之间一个新出现的都没有，
+/// I-3.5 成立（新池新建文件那一对判到了）。共享树表时映射根不往下并，(1, 4) 先走、A 后走，A 的引用集合里少了映射树，
+/// (A, (1, 4)) 那一对把映射树当成新出现、诞生代号 3 不大于 3，这一条红。
+#[test]
+fn a_zero_unit_publish_sharing_the_tree_table_and_the_mapping_root_holds_the_reference_interval_invariant(
+) {
+    let (pool, _first, _zero_unit) =
+        pool_with_a_zero_unit_publish_after_the_first_file("reference-interval-shared-tree-table");
+    assert_eq!(
+        verdict(&pool.memory_pool(), REFERENCE_INTERVALS_ARE_EXACT),
+        InvariantVerdict::Holds,
+        "A 与零单元发布那一版引用同一批单元"
+    );
+}
+
+/// I-9.25 回到 kb 字面：A (1, 3) 之后同一实例发一次零单元发布 (1, 4)，它与 A 共享 A 那片树表。树表里 inode 树那条的
+/// previous_snapshot_txg 改成 4：(1, 4) 那一版成立，A 那一版（txg 3）不成立——共享树表的每一版各判一次，I-9.25 红。
+/// 只在第一个走到它的那一版判（最新那条根先走、txg 最大）这一条不红。
+#[test]
+fn a_tree_table_shared_by_two_versions_is_judged_against_the_txg_of_each_version() {
+    let (pool, first, zero_unit) = pool_with_a_zero_unit_publish_after_the_first_file(
+        "previous-snapshot-txg-shared-tree-table",
+    );
+    let clean = pool.memory_pool();
+    assert_eq!(
+        verdict(&clean, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES),
+        InvariantVerdict::Holds,
+        "干净镜像上树表条目的 previous_snapshot_txg 都是 0"
+    );
+    let mut image = clean.clone();
+    let later_txg = zero_unit.root.checkpoint_txg.0;
+    rewrite_unit_and_propagate(&mut image, tree_table_slot(&first), |bytes| {
+        let field = tree_table_entry_offset_of_kind(bytes, TREE_KIND_INODE)
+            + TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET;
+        assert_eq!(
+            u64::from_le_bytes(bytes[field..field + 8].try_into().expect("8 字节")),
+            0,
+            "inode 树那条的 previous_snapshot_txg 本来是 0"
+        );
+        bytes[field..field + 8].copy_from_slice(&later_txg.to_le_bytes());
+    });
+    assert_violated_mentioning(
+        &image,
+        PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
+        &format!("不大于这一版的 txg {}", first.root.checkpoint_txg.0),
+    );
+}
+
+/// I-8.10 新加三段之一：A 那条记录（计数器 3，带末条标志）新根段里实例表指针第一条位置条目的校验和改一位、重封；A 的根照旧。
+#[test]
+fn a_last_record_whose_instance_table_pointer_differs_from_its_root_reddens_the_new_root_segment_invariant(
+) {
+    let pool = build_pool("new-root-segment-instance-table-pointer");
+    let clean = pool.memory_pool();
+    assert_eq!(
+        verdict(
+            &clean,
+            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH
+        ),
+        InvariantVerdict::Holds,
+        "干净镜像上带末条标志的记录与同代根七段都相同"
+    );
+    let mut image = clean.clone();
+    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 3, |record| {
+        record.new_instance_table.locations[0].unit_checksum ^= 1;
+    });
+    assert_violated_mentioning(
+        &image,
+        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
+        "实例表指针",
+    );
+}
+
+/// I-8.10 新加三段之二：A 那条记录新根段里分配记录树根指针（带文件的一版恒全零）换成一条非零指针、重封；A 的根照旧。
+#[test]
+fn a_last_record_whose_allocation_record_tree_root_pointer_differs_from_its_root_reddens_the_new_root_segment_invariant(
+) {
+    let pool = build_pool("new-root-segment-allocation-record-tree-root");
+    let mut image = pool.memory_pool();
+    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 3, |record| {
+        assert_eq!(
+            record.new_allocation_record_tree_root,
+            NodePointer::empty_root(),
+            "带文件的一版，记录新根段里的分配记录树根指针恒全零"
+        );
+        record.new_allocation_record_tree_root = record.new_instance_table;
+    });
+    assert_violated_mentioning(
+        &image,
+        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
+        "分配记录树根指针",
+    );
+}
+
+/// I-8.10 新加三段之三：A 那条记录新根段里卸载记号 0 → 1（位 0 合法，读者照收）、重封；A 的根不带记号。
+#[test]
+fn a_last_record_whose_unmount_marker_differs_from_its_root_reddens_the_new_root_segment_invariant()
+{
+    let pool = build_pool("new-root-segment-unmount-marker");
+    let mut image = pool.memory_pool();
+    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 3, |record| {
+        assert_eq!(
+            record.new_unmount_marker,
+            UnmountMarker::NotWrittenByTheUnmountSequence,
+            "新池新建文件那条记录不带卸载记号"
+        );
+        record.new_unmount_marker = UnmountMarker::WrittenByTheUnmountSequence;
+    });
+    assert_violated_mentioning(
+        &image,
+        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
+        "卸载记号",
+    );
+}
+
+/// RR3 的观察者：只做过 mkfs 的池第一次可写挂载（取号 1、不写行）、第二次可写挂载（取号 2，写行：实例表换了、分配记录树根住根记录）。
+/// 写行那条记录与它的根都在盘上，I-8.10 七段都比得到、成立。写行记录的实例表指针照抄上一版的、或分配记录树根写成空的，这一条红。
+#[test]
+fn the_row_publish_record_carries_every_new_root_segment_field_of_its_root() {
+    let mut formatted = format_pool("new-root-segment-of-the-row-publish");
+    let mut first_mount_devices = formatted.reopen_recorded();
+    mount_writable(&parameters(), &mut first_mount_devices)
+        .expect("第一次可写挂载：取号 1、不写行");
+    drop(first_mount_devices);
+    let mut second_mount_devices = formatted.reopen_recorded();
+    mount_writable(&parameters(), &mut second_mount_devices).expect("第二次可写挂载：取号 2、写行");
+    formatted.devices = Some(second_mount_devices);
+    let image = formatted.memory_pool();
+    let system_configuration = choose_system_configuration(&image).expect("择得出系统配置");
+    let roots = readable_roots(
+        &image,
+        &system_configuration.immutable.region_devices,
+        &system_configuration.immutable.sizes,
+        &system_configuration.immutable.filesystem_identifier,
+    );
+    let row_root = roots
+        .iter()
+        .filter(|root| root.instance == InstanceGeneration(2))
+        .min_by_key(|root| root.checkpoint_txg)
+        .expect("实例 2 的写行那条根读得出");
+    let root_before_the_row = roots
+        .iter()
+        .filter(|root| root.checkpoint_txg < row_root.checkpoint_txg)
+        .max_by_key(|root| (root.checkpoint_txg, root.instance))
+        .expect("写行之前那一版的根读得出");
+    assert_ne!(
+        row_root.instance_table, root_before_the_row.instance_table,
+        "写行那一版换了实例表"
+    );
+    assert_ne!(
+        row_root.allocation_record_tree_root,
+        NodePointer::empty_root(),
+        "树表 0 条的一版写行之后，分配记录树根住根记录"
+    );
+    assert_eq!(
+        verdict(
+            &image,
+            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH
+        ),
+        InvariantVerdict::Holds,
+        "写行那条记录的新根段七段与它的根逐字节相同"
+    );
+}
+
+/// 改 A 那条记录（计数器 3）：新根段树表指针两条位置条目的校验和各翻一位（施加出来那一版走不完，I-4.1 看得见收没收），
+/// 卸载记号那 4 字节写成 `field`，头校验和重封；两盘各一份。
+fn set_the_unmount_marker_field_and_break_the_tree_table_pointer_of_the_first_file_record(
+    image: &mut MemoryPool,
+    field: u32,
+) {
+    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
+    let offset = record_offset(3, JOURNAL_RING_DEFAULT_BYTES);
+    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
+    for device in DEVICES {
+        let mut record = JournalRecord::parse(
+            &read(image, device, offset.0, record_bytes),
+            filesystem_identifier,
+        )
+        .expect("改之前 A 那条记录自证过");
+        for location in &mut record.new_tree_table.locations {
+            location.unit_checksum ^= 1;
+        }
+        let mut bytes = record.to_bytes();
+        bytes[JOURNAL_RECORD_UNMOUNT_MARKER_OFFSET..JOURNAL_RECORD_UNMOUNT_MARKER_OFFSET + 4]
+            .copy_from_slice(&field.to_le_bytes());
+        let digest = wide_checksum_with_field_zeroed(
+            &bytes,
+            record_bytes,
+            JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET,
+        );
+        bytes[JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET..JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET + 32]
+            .copy_from_slice(&digest);
+        write(image, device, offset.0, &bytes);
+    }
+}
+
+/// RR1（m3-root-ring-code-r1 判决第三节表一，正推腿 H1 那一盘面）：新池新建文件、A 的根槽改坏，A 那条记录的卸载记号字段写成 1、2、
+/// 0x8000_0000（树表指针改坏，重放出来那一版走不完）。恢复扫环只收卸载记号位 0 之外全 0 的；池级 checker 收下同一批：
+/// 收下时它重放那条记录、I-4.1 红，不收时所选根之上一次发布都不施加、I-4.1 报不适用。checker 不判那 4 字节时 2 与 0x8000_0000 两份红。
+// harness-test-granularity:one-scenario 同一份盘面上卸载记号字段的三个取值各拷一份镜像比，循环里不建池
+#[test]
+fn a_record_whose_unmount_marker_has_bits_other_than_bit_zero_is_dropped_by_the_checker_as_by_the_recovery(
+) {
+    let pool = build_pool("unmount-marker-bits-other-than-bit-zero");
+    let mut torn = pool.memory_pool();
+    tear_the_root_slot_written_for_txg(&mut torn, 3);
+    for (field, kept_by_the_recovery) in [(1u32, true), (2u32, false), (0x8000_0000u32, false)] {
+        let mut image = torn.clone();
+        set_the_unmount_marker_field_and_break_the_tree_table_pointer_of_the_first_file_record(
+            &mut image, field,
+        );
+        let system_configuration = choose_system_configuration(&image).expect("择得出系统配置");
+        let kept_by_the_scan =
+            scan_journal(&image, &system_configuration).contains_key(&(InstanceGeneration(1), 3));
+        assert_eq!(
+            kept_by_the_scan, kept_by_the_recovery,
+            "卸载记号字段 {field:#x}：恢复扫环收不收 A 那条记录"
+        );
+        let replayed = verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE);
+        if kept_by_the_recovery {
+            assert!(
+                matches!(replayed, InvariantVerdict::Violated(_)),
+                "卸载记号字段 {field:#x}：恢复收下、checker 也收下，重放出来那一版走不完，I-4.1 红：{replayed:?}"
+            );
+        } else {
+            assert_eq!(
+                replayed,
+                InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
+                "卸载记号字段 {field:#x}：恢复不收，checker 也不收，所选根之上一次发布都不施加"
+            );
+        }
+    }
+}
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index daff3ae..5c623b3 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -19,7 +19,8 @@ use singlefs_checker::image::{
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
     LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH, NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
     POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
-    ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES, ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
+    REFERENCE_INTERVALS_ARE_EXACT, ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
+    ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
     SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
     SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
     SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
@@ -1285,6 +1286,28 @@ fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {
                 );
             }),
         ),
+        // I-3.5：A 的数据单元（50180）头里的诞生代号 3 → 2，重封、补到根：A 引用、暖机那一版（txg 2）不引用的单元诞生代号不大于 2，
+        // 也不在整棵继承下来的用户可见树里（extent 树根指针出生 txg 3）。指针、映射 key 与分配代还记 3，I-1.2、I-1.11、I-3.10 跟着红，
+        // 所以放在这一张表里（只判「它红」，不判「只红它」）；按各个组成部分的几份在
+        // `checker_judges_reference_intervals_and_every_new_root_segment_field.rs`。
+        (
+            REFERENCE_INTERVALS_ARE_EXACT,
+            Box::new(|image: &mut MemoryPool| {
+                mutate_unit(
+                    image,
+                    DATA_UNIT,
+                    |bytes| {
+                        assert_eq!(
+                            get_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET),
+                            3,
+                            "A 的数据单元生在 txg 3"
+                        );
+                        set_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET, 2);
+                    },
+                    true,
+                );
+            }),
+        ),
     ]
 }
 
@@ -6229,7 +6252,8 @@ fn known_bad_images_of_pointer_journal_and_generation_fields() -> Vec<(&'static
             }),
         ),
         // I-2.4 指针 extent 偏移那一格（`research/prompts/m3-inv-a-r1-opus-output.md` A6 的造法）：每条自证过的根里实例表指针的 extent 偏移 0 → 1，
-        // 根槽自证校验和重封；实例表指针不在新根段里，记录不跟着改。
+        // 根槽自证校验和重封；实例表指针从第 8 批起也在 I-8.10 比的新根段七段里，同 (实例, txg) 的记录跟着改、反向链接着重算
+        // （`set_the_extent_offset_of_the_instance_table_pointer_in_the_records_of_these_roots`）。
         (
             "I-2.4",
             Box::new(|image: &mut MemoryPool| {
@@ -6253,11 +6277,62 @@ fn known_bad_images_of_pointer_journal_and_generation_fields() -> Vec<(&'static
                     rewritten_roots += 1;
                 }
                 assert!(rewritten_roots > 0, "根环里至少改了一条根");
+                set_the_extent_offset_of_the_instance_table_pointer_in_the_records_of_these_roots(
+                    image,
+                );
             }),
         ),
     ]
 }
 
+/// 记录新根段里实例表指针的起点：新根段 99 起，树表指针 86、映射根指针 86、树 ID 水位 8、F 8 之后（D23（journal 的角色与格式） 已定项 15）。
+const JOURNAL_RECORD_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET: usize = 99 + 188;
+
+/// 环里从计数器 1 起读到第一格没有 magic 的那条为止：同 (实例, txg) 的根读得出（根槽 magic 在、实例表指针 extent 偏移已改成 1）的记录，
+/// 新根段里实例表指针的 extent 偏移也 0 → 1；每一条的反向链按前一条（改后的）记录头重算，头校验和重封，两盘各一份。
+/// 记录不经 `JournalRecord::parse` 改：实现的读者拒 extent 偏移非 0 的指针，解不出这样的记录。
+/// 这份镜像上只有实例 1 写过记录（反向链在实例内逐条接），迭代上界是 [`RECORDS_READ_BACK_AT_MOST`]。
+fn set_the_extent_offset_of_the_instance_table_pointer_in_the_records_of_these_roots(
+    image: &mut MemoryPool,
+) {
+    let roots_with_the_extent_offset_set: std::collections::BTreeSet<(u32, u64)> = root_slots()
+        .into_iter()
+        .map(|(device, offset)| read(image, device, offset, 512))
+        .filter(|bytes| bytes[..4] == singlefs_core::root_record::ROOT_MAGIC)
+        .map(|bytes| (get_u32(&bytes, 24), get_u64(&bytes, 28)))
+        .collect();
+    for device in DEVICES {
+        let mut previous_record: Option<Vec<u8>> = None;
+        for counter in 1..=RECORDS_READ_BACK_AT_MOST {
+            let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES).0;
+            let mut bytes = read(image, device, offset, 4096);
+            if bytes[..4] != singlefs_core::journal::JOURNAL_MAGIC {
+                break;
+            }
+            let identity = (
+                get_u32(&bytes, JOURNAL_RECORD_INSTANCE_OFFSET),
+                get_u64(&bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET),
+            );
+            if roots_with_the_extent_offset_set.contains(&identity) {
+                let field = JOURNAL_RECORD_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
+                    + POINTER_EXTENT_OFFSET_FIELD_OFFSET;
+                bytes[field] = 1;
+            }
+            if let Some(previous) = &previous_record {
+                set_u32(
+                    &mut bytes,
+                    JOURNAL_RECORD_BACK_CHAIN_OFFSET,
+                    back_chain_of(previous),
+                );
+            }
+            let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 46);
+            bytes[46..78].copy_from_slice(&digest);
+            write(image, device, offset, &bytes);
+            previous_record = Some(bytes);
+        }
+    }
+}
+
 /// 上面那批坏镜像各自**只**红在自己那一条上；干净镜像上那几条要真被评估过且成立（不是「不适用」）。
 #[test]
 fn each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant() {
@@ -7458,6 +7533,8 @@ fn an_instance_table_pointer_whose_birth_txg_differs_from_the_instance_table_hea
     let digest = wide_checksum_with_field_zeroed(&root, 512, 138);
     root[138..170].copy_from_slice(&digest);
     write(&mut image, device, offset, &root);
+    // 记录新根段里那份实例表指针跟着改（I-8.10 七段里有它）：盘上留下的是「写者从一开始就写出了这样的根」的样子。
+    copy_each_root_into_the_new_root_segment_of_its_journal_records(&mut image);
     let verdicts = check_pool_image(&image);
     assert_eq!(
         violated_invariants(&verdicts),
diff --git a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
index da58223..bbfea2d 100644
--- a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
+++ b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
@@ -677,10 +677,12 @@ fn an_older_version_naming_again_a_placement_inside_a_subtree_it_shares_with_a_n
     reseal_root_slot(&mut root_slot);
     write_bytes(&mut image, root_device, root_offset, &root_slot);
     let red = violations(&image);
+    // 另红两条，都是改了 F 那条根的实例表指针连带的：F 那条记录新根段里的实例表指针没跟着改（I-8.10 从第 8 批起比七段，有它）；
+    // F 不再引用实例表单元而 N 照旧引用，那个单元在 N 里像是新出现、诞生代号 0 不大于 F 的 txg（I-3.5 的出现规则）。
     assert_eq!(
         violated_names(&red),
-        ["I-5.1"],
-        "F 这一版里 D 被引用两次，只该红 I-5.1：{red:?}"
+        ["I-5.1", "I-8.10", "I-3.5"],
+        "F 这一版里 D 被引用两次，红 I-5.1（另加改实例表指针连带的 I-8.10 与 I-3.5）：{red:?}"
     );
     assert!(
         red[0].1.contains("被同一版引用了两次")
diff --git a/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs b/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
index 6e92cc6..b203acc 100644
--- a/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
+++ b/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
@@ -618,7 +618,7 @@ fn a_device_whose_slot_zero_fails_borrows_the_spacing_another_slot_zero_records_
     let chosen_by_the_checker_on_device_zero = chosen_system_configurations(&image)
         .into_iter()
         .find(|(device, _)| *device == 0)
-        .and_then(|(_, chosen)| chosen)
+        .and_then(|(_, chosen_by_the_checker)| chosen_by_the_checker)
         .map(|(view, geometry)| (geometry.slot_spacing, view.slot_generation));
     assert_eq!(
         chosen_by_the_checker_on_device_zero,
diff --git a/crates/singlefs-harness/tests/instance_table_chain_read.rs b/crates/singlefs-harness/tests/instance_table_chain_read.rs
index fa6faf0..d25d2ac 100644
--- a/crates/singlefs-harness/tests/instance_table_chain_read.rs
+++ b/crates/singlefs-harness/tests/instance_table_chain_read.rs
@@ -21,6 +21,7 @@ use singlefs_core::address::{
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::bytes::ByteWriter;
 use singlefs_core::instance_table::{InstanceRow, InstanceTablePageIndex, InstanceTableRecords};
+use singlefs_core::journal::{back_chain_of, record_offset, JournalRecord};
 use singlefs_core::make_filesystem::{
     instance_table_chain_record, location_entries, make_filesystem, INSTANCE_TABLE_SLOT,
 };
@@ -35,8 +36,8 @@ use singlefs_core::recovery::{
 use singlefs_core::root_record::RootRecord;
 use singlefs_core::root_ring::{slot_offset, target_for_publish};
 use singlefs_core::transaction::{PublishError, TransactionUnit};
-use singlefs_core::unit::{build_packed_unit, parse_packed_unit};
-use singlefs_format::{DATA_UNIT_BYTES, INSTANCE_ROW_BYTES};
+use singlefs_core::unit::{build_packed_unit, parse_packed_unit, unit_filesystem_identifier};
+use singlefs_format::{DATA_UNIT_BYTES, INSTANCE_ROW_BYTES, JOURNAL_RECORD_BYTES};
 use singlefs_harness::memory_pool::SparseBlockDevice;
 use singlefs_harness::{RecordingBlockDevice, SharedStream};
 
@@ -236,12 +237,72 @@ fn write_two_page_chain(
         rewritten_roots += 1;
     }
     assert!(rewritten_roots >= 1, "至少最新那条根指着原来那一片");
+    let rewritten_versions: Vec<(InstanceGeneration, CheckpointTxg)> = roots
+        .iter()
+        .filter(|root| root.instance_table == original_first_page_pointer)
+        .map(|root| (root.instance, root.checkpoint_txg))
+        .collect();
+    point_the_records_of_these_versions_at_the_first_page(
+        devices,
+        &rewritten_versions,
+        &first_page_pointer,
+        system_configuration.immutable.sizes.journal_ring_bytes,
+    );
     TwoPageChain {
         first_page_slot,
         second_page_pointer,
     }
 }
 
+/// 根环里换了实例表指针的那几版，同 (实例, txg) 的 journal 记录新根段里的实例表指针也换成 `first_page_pointer`（I-8.10 从第 8 批起比七段，
+/// 有它；盘上留下「写者从一开始就写出了这样的根」的样子），之后同实例的记录反向链按前一条改后的记录头重算（I-8.6 只链实例内逻辑前一条），
+/// `JournalRecord::to_bytes` 重封，两盘各一份。环里从计数器 1 起读到第一格解不出就停：迭代上界是环的槽数。
+fn point_the_records_of_these_versions_at_the_first_page(
+    devices: &mut Devices,
+    versions: &[(InstanceGeneration, CheckpointTxg)],
+    first_page_pointer: &NodePointer,
+    ring_bytes: u64,
+) {
+    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
+    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
+    for device in DISKS {
+        // 跨轮携带：计数器小 1 的那一条若被改过，记它的实例与改后的字节（下一条的反向链按它重算）。
+        let mut rewritten_previous_record: Option<(InstanceGeneration, Vec<u8>)> = None;
+        for counter in 1..=ring_bytes / JOURNAL_RECORD_BYTES {
+            let offset = record_offset(counter, ring_bytes);
+            let Some(bytes) = PoolReader::read(&*devices, device, offset, record_bytes) else {
+                break;
+            };
+            let Some(record) = JournalRecord::parse(&bytes, filesystem_identifier) else {
+                break;
+            };
+            let mut rewritten = record.clone();
+            if versions.contains(&(record.instance, record.checkpoint_txg)) {
+                rewritten.new_instance_table = *first_page_pointer;
+            }
+            if let Some((previous_instance, previous_bytes)) = &rewritten_previous_record {
+                if *previous_instance == record.instance {
+                    rewritten.back_chain = back_chain_of(previous_bytes);
+                }
+            }
+            rewritten_previous_record = if rewritten == record {
+                None
+            } else {
+                let rewritten_bytes = rewritten.to_bytes();
+                devices
+                    .iter_mut()
+                    .find(|(identity, _)| *identity == device)
+                    .expect("盘在池里")
+                    .1
+                    .wrapped_device_mut()
+                    .image
+                    .write(offset, &rewritten_bytes);
+                Some((record.instance, rewritten_bytes))
+            };
+        }
+    }
+}
+
 /// 最新那一版原来那张表（一片）里的行：实例 1、2、3 各一行。
 fn rows_of_the_newest_table(devices: &Devices) -> Vec<InstanceRow> {
     let rows = instance_table_of_root(devices, &newest_root(devices))
@@ -513,7 +574,8 @@ fn chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant()
     let verdicts = verdicts(&devices);
     assert_eq!(
         violated(&verdicts),
-        vec!["I-1.1", "I-1.2", "I-3.9", "I-4.8", "I-7.2", "I-7.4"],
+        // I-3.5：mkfs 种的那一片（诞生代号 0）被最新那一版的链再次引用，前一版不引用它。
+        vec!["I-1.1", "I-1.2", "I-3.9", "I-4.8", "I-7.2", "I-7.4", "I-3.5"],
         "{verdicts:?}"
     );
     match verdict_of(&verdicts, "I-1.2") {
````

## 改钉（报告 `/tmp/claude-1000/m3-r1/batch8-repin/report.md`）与主 agent 改的通配臂

用例两份的改动（`writable_mount_of_a_formatted_pool.rs` 对的是改钉之前那一版，`checker_judges_reference_intervals_and_every_new_root_segment_field.rs` 对的是 8-2 补丁打完那一版）：

````diff
diff --git acrates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
index e70a6648..66a38b7f 100644
--- acrates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -568,7 +568,10 @@ fn region_device(txg: u64) -> DeviceIdentity {
 /// I-7.9（回退下界 F 不高于抬 F 的上限） 是因为没抬过 F：每条根带的 F 都是 0（这个文件里哪一步都不抬，下面四张表都有它）。
 /// I-4.1（重放后树可遍历） 五张表都有：这个文件里每一步的镜像都是发布完才看的，最新根之上没有记录，重放一次都不施加
 /// （阳性对照与坏镜像在 `checker_known_bad_images.rs` 那份「发布 B 的记录落了、根槽没落」的崩溃镜像上）。
-const NOT_APPLICABLE_WITHOUT_FILE: [&str; 27] = [
+/// I-3.5（引用区间的精确性） 在这里报不适用是因为回退候选集里相邻两版之间一个新出现的单元都没有：第一次可写挂载不写行，
+/// 写行那次发布（txg 1）与暖机（txg 2）都是零单元发布，树表、实例表照抄 mkfs 那一版、映射根全零，后一版引用的单元前一版都引用过，
+/// 出现规则一个单元都判不到（mkfs 之后那一格的理由不同，见 `NOT_APPLICABLE_RIGHT_AFTER_MKFS`）。
+const NOT_APPLICABLE_WITHOUT_FILE: [&str; 28] = [
     "I-1.10",
     "I-3.1",
     "I-3.9",
@@ -596,12 +599,15 @@ const NOT_APPLICABLE_WITHOUT_FILE: [&str; 27] = [
     "I-9.26",
     "I-3.12",
     "I-4.1",
+    "I-3.5",
 ];
 
 /// mkfs 刚写完那一份要多报两条不适用：journal 环整段是 0，一条自证过的记录都没有 ⇒ I-8.6（反向链算法） 与
 /// I-8.9（一次发布的记录序号连续且只有末条带标志） 都没有判的对象。
 /// 可写挂载之后环里有记录（写行 / 暖机的空发布），它们就真被评估过了——所以这两条只在 mkfs 之后那一格。
-const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 31] = [
+/// I-3.5（引用区间的精确性） 两张表都有，这一格的理由是回退候选集只剩 mkfs 的第 0 代根一版（环里没有记录、根环里只有它），
+/// 凑不出相邻的一对。
+const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 32] = [
     "I-1.10",
     "I-3.1",
     "I-3.9",
@@ -633,6 +639,7 @@ const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 31] = [
     "I-3.12",
     "I-8.12",
     "I-4.1",
+    "I-3.5",
 ];
 
 /// 写完第一个文件版本之后仍报不适用的那五条（I-4.1 的理由见 `NOT_APPLICABLE_WITHOUT_FILE`）：
@@ -659,6 +666,8 @@ const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 6] =
 /// I-3.10 在这一格真被评估过；这棵树按位置寻址（D8（核心索引结构） 已定项 14），checker 从根记录那一项按位置走下去、每个节点判条目宽，
 /// I-1.10 也真被评估过。checker 把根记录直接持有的那一棵并进候选根的分配记录（实审 B2）：被换下的 mkfs 那片实例表带释放标志，
 /// I-3.9 判它的释放代；I-5.4 判这棵树里的记录两两不相交——两条都真被评估过且成立。
+/// I-3.5（引用区间的精确性） 也不在这张表里：写行那次发布（实例 2、txg 3）把实例表 COW 到新落点、写了自己那棵分配记录树，
+/// 这几个单元前一版（实例 1 的暖机根，txg 2）不引用、诞生代号 3 大于 2，出现规则真判到了它们且成立。
 const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 21] = [
     "I-3.1",
     "I-3.11",
diff --git acrates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs b/crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs
index 3f4a3eac..61cb74f9 100644
--- acrates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs
+++ b/crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs
@@ -107,7 +107,7 @@ fn assert_violated_mentioning(image: &MemoryPool, invariant: &str, expected_in_t
             detail.contains(expected_in_the_detail),
             "{invariant} 判违例，说明里该有「{expected_in_the_detail}」：{detail}"
         ),
-        other => {
+        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
             panic!("{invariant} 应当判违例（说明里有「{expected_in_the_detail}」），实际 {other:?}")
         }
     }
````

`crates/mutations.tsv` 改钉之后的五行（第 477、1767、1916、1917、1918 行，整行）：

````text
增补 2 收口表第 39 行那一族：发布取完落点之后不记根（不转环、不回收；取号之前的预演与真发走同一段，两边一起不回收）	crates/singlefs-core/src/transaction.rs	    crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(\n        &*pool.devices,\n        pool.parameters,\n        allocator,\n        txg,\n    );\n    let slot_of = |identity: TransactionUnit| slots[&identity];	    let slot_of = |identity: TransactionUnit| slots[&identity];	-p singlefs-harness --test random_histories -- --include-ignored rolling_back_while_mounted_to_the_oldest_ring_root	rolling_back_while_mounted_to_the_oldest_ring_root_on_a_small_device_and_overwriting_again_all_succeed
二·1 I-8.10 判定恒真：带末条标志的记录新根段不与同代根比	crates/singlefs-checker/src/walk.rs	                    differing_fields.is_empty(),	                    true,	-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
第 5 批池级 checker：I-8.10 新根段树表指针不比	crates/singlefs-checker/src/walk.rs	    (\n        "树表指针",\n        NEW_ROOT_SEGMENT_TREE_TABLE_POINTER,\n        ROOT_RECORD_TREE_TABLE_POINTER,\n    ),\n		-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
第 5 批池级 checker：I-8.10 新根段映射根指针不比	crates/singlefs-checker/src/walk.rs	    (\n        "映射根指针",\n        NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER,\n        ROOT_RECORD_MAPPING_ROOT_POINTER,\n    ),\n		-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
第 5 批池级 checker：I-8.10 新根段树 ID 水位不比	crates/singlefs-checker/src/walk.rs	    (\n        "树 ID 水位",\n        NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK,\n        ROOT_RECORD_TREE_IDENTIFIER_WATERMARK,\n    ),\n		-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
````
