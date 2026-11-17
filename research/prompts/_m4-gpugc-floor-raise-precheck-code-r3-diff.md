# m4-gpugc-floor-raise-precheck-code-r3：被判的产品代码改动（撤回第 122 批聚簇段那一截，`mounted_session.rs`）

```diff
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -5938,7 +5938,7 @@
     versions_published: VersionsPublishedInThisChange,
 }
 
-/// 一块盘上抬 F 至多放得开的槽，加上卸载重挂放得出的段内槽对，都是分配器增量维护的数（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」：准入路径上不扫单元区）。
+/// 一块盘上抬 F 至多放得开的槽，都是分配器增量维护的数（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」：准入路径上不扫单元区）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct SlotsAFloorRaiseCouldRelease {
     /// defer 窗口里全部已释放的槽：只算槽数、不问这一串的上限够不够得到它们。
@@ -5948,13 +5948,10 @@
     /// 计数上空闲而位上挡着的槽：隔离着没分配的与抬 F 扣住的（`DeviceFreeMap::hold_until_floor_takes_effect`）。
     /// 式子本来不扣扣住的槽，落得下那一判里它们挡着槽对。
     free_but_blocked: u64,
-    /// 这次挂载开过的聚簇段里两槽都没挡的槽对：抬 F 放不出它们，卸载重挂时分配器重建、聚簇段清空，它们就算进段外槽对。
-    /// 槽对那一判的上界算上它们，先算才不会吃掉「推到上限、报被宽护卡住、列卸载重挂」那条出路（D16（发布语义） 已定项 1「准入」那一行，[`StuckBehindTheWideProtection`]）。
-    unblocked_slot_pairs_inside_the_cluster_segments: u64,
 }
 
 impl SlotsAFloorRaiseCouldRelease {
-    fn of_the_device(device_map: &DeviceFreeMap, cluster_segments: &BTreeSet<SlotNumber>) -> Self {
+    fn of_the_device(device_map: &DeviceFreeMap) -> Self {
         Self {
             deferred: device_map.deferred_slots(),
             isolated: device_map.isolated_slots(),
@@ -5962,19 +5959,12 @@
                 .free_slots()
                 .checked_sub(device_map.unblocked_slots())
                 .expect("没挡的槽都没置分配位，分配位没置的都算空闲：没挡的槽数不大于空闲槽数"),
-            unblocked_slot_pairs_inside_the_cluster_segments: device_map
-                .unblocked_slot_pairs_outside_the_cluster_segments(&BTreeSet::new())
-                .checked_sub(
-                    device_map.unblocked_slot_pairs_outside_the_cluster_segments(cluster_segments),
-                )
-                .expect("段外槽对是全部槽对减去那几段各自的数，各段的数都不为负：段外槽对不大于全部槽对"),
         }
     }
 }
 
 /// 一块被判短的盘推抬 F 至多凑得出多少，按判它短的那一判自己的量（宁大勿小：上界算小了会误拒一个推了本来写得进的写）：
-/// 式子那一判是可用加 defer 与隔离的字节；段外槽对那一判是现有的槽对加每个放得开的槽各补成一对（另一半本来就空着时放开一个槽就多一对，所以按一个数据单元的槽数换算），
-/// 再加这次挂载开过的聚簇段里没挡的槽对（卸载重挂之后算进段外槽对，见 [`SlotsAFloorRaiseCouldRelease`] 那一项）。
+/// 式子那一判是可用加 defer 与隔离的字节；段外槽对那一判是现有的槽对加每个放得开的槽各补成一对（另一半本来就空着时放开一个槽就多一对，所以按一个数据单元的槽数换算）。
 /// 提交内生块那一判交 `None`：这一判数的槽抬 F 那一串自己也要占，不在这里算。
 fn most_a_floor_raise_could_make_available(
     short: &DeviceShortOfDemand,
@@ -5992,13 +5982,8 @@
                 slots.deferred + slots.isolated + slots.free_but_blocked;
             let slots_of_a_data_unit =
                 TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots();
-            let slot_pairs_a_remount_frees_from_the_cluster_segments =
-                slots.unblocked_slot_pairs_inside_the_cluster_segments;
             Some(AvailableBytesOnOneDevice(
                 short.available.0
-                    + bytes_of_slots(
-                        slot_pairs_a_remount_frees_from_the_cluster_segments * slots_of_a_data_unit,
-                    )
                     + bytes_of_slots(slots_that_could_complete_a_pair * slots_of_a_data_unit),
             ))
         }
@@ -6041,10 +6026,7 @@
                 .expect("拒绝里的盘来自这个分配器：准入的两份读数都按它的盘建");
             device_short_even_with_everything_a_floor_raise_could_release(
                 short,
-                SlotsAFloorRaiseCouldRelease::of_the_device(
-                    device_map,
-                    allocator.cluster_segments(),
-                ),
+                SlotsAFloorRaiseCouldRelease::of_the_device(device_map),
             )
         })
         .collect()
@@ -8213,7 +8195,6 @@
             deferred: 0,
             isolated: 0,
             free_but_blocked: 0,
-            unblocked_slot_pairs_inside_the_cluster_segments: 0,
         }
     }
 
@@ -8242,7 +8223,6 @@
             deferred: 2,
             isolated: 1,
             free_but_blocked: 0,
-            unblocked_slot_pairs_inside_the_cluster_segments: 0,
         }
     }
 
@@ -8299,7 +8279,6 @@
                     deferred: 0,
                     isolated: 0,
                     free_but_blocked: 8,
-                    unblocked_slot_pairs_inside_the_cluster_segments: 0,
                 },
             )
             .is_some(),
@@ -8333,7 +8312,6 @@
                 deferred: 1,
                 isolated: 0,
                 free_but_blocked: 0,
-                unblocked_slot_pairs_inside_the_cluster_segments: 0,
             }
         ));
     }
@@ -8345,7 +8323,6 @@
                 deferred: 0,
                 isolated: 1,
                 free_but_blocked: 0,
-                unblocked_slot_pairs_inside_the_cluster_segments: 0,
             }
         ));
     }
@@ -8357,7 +8334,6 @@
                 deferred: 0,
                 isolated: 0,
                 free_but_blocked: 1,
-                unblocked_slot_pairs_inside_the_cluster_segments: 0,
             }
         ));
     }
@@ -8369,24 +8345,6 @@
         ));
     }
 
-    /// 卸载重挂把这次挂载开过的聚簇段清空，段里空着的一对槽就算进段外槽对：少算它，被宽护卡住、列卸载重挂的那一形就判成放不下、一串都不推。
-    #[test]
-    fn an_unblocked_slot_pair_inside_the_cluster_segments_counts_toward_the_slot_pair_bound() {
-        assert_eq!(
-            (
-                one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(no_slots()),
-                one_data_unit_short_on_the_slot_pairs_is_pushed_for_with(
-                    SlotsAFloorRaiseCouldRelease {
-                        unblocked_slot_pairs_inside_the_cluster_segments: 1,
-                        ..no_slots()
-                    }
-                ),
-            ),
-            (false, true),
-            "没有它判放不下；聚簇段里空着一对、需求一个数据单元：照推"
-        );
-    }
-
     #[test]
     fn a_device_short_on_the_slots_for_commit_generated_units_is_always_pushed_for() {
         assert_eq!(
```
