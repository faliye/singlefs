# 根环实现第 10 批代码轮：改动全文（2026-10-05）

每份补丁一节，次序就是合入的次序；各节写明补丁目录、实现员报告与 sha256。补丁相对它合入那一刻的主工作区。

## batch10b-core

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10b-core/patch/`
- `crates.patch`：sha256 270bafd02617b0a85050c7b500d287d55dacf20f47a126524dac494d7ddd16be
- `mutations-append.tsv`：sha256 ee81f91994540d1658be43d21b6c993f56344244810d659f9b92f455476f95c9
- `mutations-delete.txt`：sha256 49619cc0a7c33411d376d177ec2ba48cfaa2ded9072103bae20293b162adb4af
- `mutations-replacements.tsv`：sha256 cfe49527553df31dc4d375a8c625c90f9765449fe2c0e3e2d0fb6d0d60bfc1bb
- `report.md`：sha256 60c4b5b094a941fcfc42cdbd2853cd5c1d723426498d4676444911a2eb42220d

### crates.patch

````
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index bf5a31c..f00b7d4 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -46,6 +46,7 @@ use crate::image::{
     LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH, MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
     NAMED_ENTRY_MATCHES_THE_UNIT_HEADER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
     PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, REFERENCE_INTERVALS_ARE_EXACT,
+    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
     TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
     TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
     UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
@@ -5218,10 +5219,10 @@ struct RollbackFloorCeilingBeforeTheRaise {
 }
 
 /// 算一条抬 F 的根 `raising_root` 那一刻的上限（I-7.9（回退下界 F 不高于抬 F 的上限），用户 2026-09-24 定）：
-/// 只用根环里 txg 比它小、同时落在两个一圈窗口里的根（txg > N − 3S）：锚在它自己的 txg 上的那一个（`.claude/kb/invariants.md` I-7.9 那一行
-/// 2026-10-04 写回的字面，照 D16（发布语义） 已定项 1「抬 F 的上限」只在一圈窗口里的根上取），与锚在这张镜像根环里最新有效根上的那一个
-/// （候选过滤那一个，`window_anchored_at_the_newest_root_of_the_image`）——后一个只在镜像比抬 F 那一刻新时更窄，挡的是环上有洞时洞里留下的
-/// 上一圈的根（理由见 [`judge_rollback_floor_raises_against_their_ceilings`]）；有效 = 按**它自己**指着的实例表不被抛弃 ∧ txg ≥ `floor_before_the_raise`（抬之前的 F）；
+/// 只用根环里 txg 比它小、落在锚在这张镜像根环里最新根上的一圈窗口里的根（txg > N − 3S，`window_anchored_at_the_newest_root_of_the_image`），
+/// 挡的是环上有洞时洞里留下的旧根（理由见 [`judge_rollback_floor_raises_against_their_ceilings`]）。抬 F 的上限本身取环里全部有效根
+/// （用户 2026-10-04 定 RGall），原来另取的那个锚在它自己 txg 上的窗口（照上限只在一圈窗口里取）随之拿掉：N 是镜像里最新根的 txg、
+/// 不小于被判那条根的 txg，那个窗口本来就罩着这个窗口，拿掉不改判定。有效 = 按**它自己**指着的实例表不被抛弃 ∧ txg ≥ `floor_before_the_raise`（抬之前的 F）；
 /// 非空 = 按 (txg, 实例) 排，跟前一条有效根比两条用户可见树的根指针，最旧的那条跟 [`UserVisibleTreeRootPointers::ABSENT`] 比。
 /// 它自己指着的实例表读不出、它之前一条有效根都没有 ⇒ None（上限无从算起：抬 F 的入口在这两格上拒，盘上出现这样一条抬 F 的根，
 /// 要么表后来坏了，要么撑它的根已被环盖掉）。
@@ -5236,15 +5237,10 @@ fn rollback_floor_ceiling_before_the_raise(
 ) -> Option<RollbackFloorCeilingBeforeTheRaise> {
     let instance_table_rows_of_the_raising_root =
         instance_table_rows_of_root_without_judging(reader, &raising_root.record_bytes)?;
-    let window_anchored_at_the_raising_root = OneRevolutionWindow {
-        anchor_txg: raising_root.checkpoint_txg,
-        root_ring_slot_count: geometry.regions * geometry.slots_per_region,
-    };
     let mut valid_roots_before: Vec<(u32, &crate::RootView)> = roots
         .iter()
         .filter(|(_, _, root)| {
             root.checkpoint_txg < raising_root.checkpoint_txg
-                && window_anchored_at_the_raising_root.contains(root.checkpoint_txg)
                 && window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)
                 && root.checkpoint_txg >= floor_before_the_raise
                 && !abandoned_by_instance_table_rows(&instance_table_rows_of_the_raising_root, root)
@@ -5331,18 +5327,20 @@ fn rollback_floor_ceiling_before_the_raise(
 /// 「抬 F 之后回退到候选集里的根」上判红（固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`）。
 /// 一次抬 F 推几次空发布、每次都带新 F 时，只有第一条是抬 F 的根（后几条的前一条已带新 F）。
 ///
-/// 上限只用根环里 txg 比它小、落在一圈窗口里的根算（[`rollback_floor_ceiling_before_the_raise`]），
+/// 上限只用根环里 txg 比它小、落在锚在镜像最新根上的一圈窗口里的根算（[`rollback_floor_ceiling_before_the_raise`]），
 /// 不在后来每张镜像上重算：抬之后再回退，后来的根会把当初撑着 F 的非空根判成无效，而 F 不回落。
 /// ⚠️ 后来的镜像上当初撑着上限的根可以已被盖掉：环按 txg 从旧到新被盖时，盖掉的是一段最旧的前缀，重算只会更宽；环上有洞时不是这样——
 /// 洞那一槽留着上一圈的根，比它新的根反倒先被盖掉，按 [`rollback_floor_ceiling_from`] 的单调性重算出来的上限变低、在合法镜像上判红
 /// （里程碑三第 8 批调查的 inject-6 种子 1 那一形）。窗口只锚在被判那条根上挡不住它：洞里那条旧根 t − 3S（t 是洞的 txg、t 晚于抬 F 那条根）
-/// 落在那个窗口里。再按这张镜像的最新有效根 N 取一圈窗口就挡住了：洞里留下的根都 ≤ N − 3S；而窗口里 txg 比抬 F 那条根小的根在抬 F 那一刻
+/// 落在那个窗口里。按这张镜像的最新根 N 取一圈窗口就挡住了：洞里留下的根都 ≤ N − 3S；而窗口里 txg 比抬 F 那条根小的根在抬 F 那一刻
 /// 就在环里、之后不会被盖（盖它要一条 txg ≥ 它 + 3S > N 的根），所以留下的是抬 F 那一刻那批根去掉最旧的一段前缀，重算只会更宽。
+/// 抬 F 那一刻的上限取环里全部有效根（RGall）时同一个道理成立：那一刻环里的根（窗口外洞里的也算）去掉 ≤ N − 3S 的那一段，仍是去掉最旧的一段前缀。
 /// 带卸载记号那一支同一个机理：洞里那条旧根顶上来、比它新的前一条已被盖掉，「txg 比它小的最新那条根」变旧、在合法镜像上判红
-/// （随机历史种子 31045、1037 那两段，9-6 报告第六节 Q1）；两支都取同一个交（用户 2026-10-04 定「乙，两支都取两个窗口」）。
+/// （随机历史种子 31045、1037 那两段，9-6 报告第六节 Q1）；两支都取同一组根（用户 2026-10-04 定「乙，两支都取两个窗口」，
+/// 同日 RGall 之后锚在被判那条根上的那个窗口被锚在镜像最新根上的罩住，只留后者）。
 ///
 /// 按这条根 flags 里的卸载记号分两支（D22（单元原子性怎么合成） 已定项 7，用户 2026-09-26 定 (b)+(c) 合用）：
-/// 带记号的（正常卸载那一串）只判 F 不高于它之前的根（两个窗口的交）里 txg 最大那条的 txg，交是空的不判；不带的（准入抬 F）照下面的准入上限判。
+/// 带记号的（正常卸载那一串）只判 F 不高于它之前的根（锚在镜像最新根上的一圈窗口里）里 txg 最大那条的 txg，一条都没有不判；不带的（准入抬 F）照下面的准入上限判。
 ///
 /// 不带记号那一支三种结局：F ≤ 上限的下沿 ⇒ 成立；F > 上限的上沿 ⇒ 违例；落在两沿之间、或上限无从算起 ⇒ 这条根不判。
 /// 一条根都没判到时整条报不适用并带理由，不报成立。
@@ -5376,20 +5374,15 @@ fn judge_rollback_floor_raises_against_their_ceilings(
         let raised_floor = raising_root.rollback_floor;
         match raising_root.unmount_marker {
             // 带卸载记号那一支：正常卸载把 F 抬到现行那一版的 txg、不另判上限（D16（发布语义） 已定项 1「抬 F 的上限」），
-            // 判 F 不高于它之前的根里 txg 最大的那条的 txg。「之前的根」与准入那一支取同一个交：txg 比它小、落在锚在它自己 txg 上的
-            // 一圈窗口里、也落在锚在这张镜像最新有效根上的一圈窗口里（用户 2026-10-04 定「乙，两支都取两个窗口」）；交是空的
-            // （镜像比它新出一圈以上、它前面那几条已被盖掉）这条根不判。记号是不是只由卸载入口打，盘上没有别的东西核它，归层 0（C557）。
+            // 判 F 不高于它之前的根里 txg 最大的那条的 txg。「之前的根」与准入那一支取同一组：txg 比它小、落在锚在这张镜像最新根上的
+            // 一圈窗口里（用户 2026-10-04 定「乙，两支都取两个窗口」；同日 RGall 之后锚在它自己 txg 上的那一个被这一个罩住、拿掉）；
+            // 一条都没有（镜像比它新出一圈以上、它前面那几条已被盖掉）这条根不判。记号是不是只由卸载入口打，盘上没有别的东西核它，归层 0（C557）。
             crate::UnmountMarkerView::WrittenByTheUnmountSequence => {
-                let window_anchored_at_the_raising_root = OneRevolutionWindow {
-                    anchor_txg: raising_txg,
-                    root_ring_slot_count: geometry.regions * geometry.slots_per_region,
-                };
                 let Some(newest_root_txg_before_the_raise) = roots
                     .iter()
                     .map(|(_, _, root)| root.checkpoint_txg)
                     .filter(|txg| {
                         *txg < raising_txg
-                            && window_anchored_at_the_raising_root.contains(*txg)
                             && window_anchored_at_the_newest_root_of_the_image.contains(*txg)
                     })
                     .max()
@@ -5445,7 +5438,7 @@ fn judge_rollback_floor_raises_against_their_ceilings(
             if raising_roots_not_judged == 0 {
                 "根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（回退与新实例的第一条根不算抬）"
             } else {
-                "根环里抬 F 的根都判不了：它自己指着的实例表读不出、它之前一条有效根都没有，或有效根的树表读不出、上限落在它带的 F 两边，或带卸载记号、两个一圈窗口的交里没有它之前的根"
+                "根环里抬 F 的根都判不了：它自己指着的实例表读不出、它之前一条有效根都没有，或有效根的树表读不出、上限落在它带的 F 两边，或带卸载记号、锚在镜像最新根上的一圈窗口里没有它之前的根"
             },
         );
     }
@@ -5570,8 +5563,8 @@ impl LocationEntryChecksumsForQuarantine<'_> {
 
 /// 收口表第 43 行取丁-defer（用户 2026-09-28 定，被攻过一轮；`research/prompts/abandoned-floor-r1-main-verification.md` M1）：
 /// I-3.1（已分配统计对得上） 的「实际遍历」并上这样的槽——F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的分配记录树里
-/// 罩住它的记录是已释放、释放代高于回收门槛 `reclaim_threshold` = max(F_生效, 一圈窗口里最旧有效根) 的（还在 defer 里，分配器还不许回收，
-/// 记账照算已分配；门槛 2026-10-02 起取一圈窗口，D16（发布语义） 已定项 1）。逐盘交回槽数。
+/// 罩住它的记录是已释放、释放代高于回收门槛 `reclaim_threshold` = max(F_生效, 环里最旧有效根) 的（还在 defer 里，分配器还不许回收，
+/// 记账照算已分配；门槛 2026-10-02 起取一圈窗口，2026-10-04 用户定 RGall 退回整个环，D16（发布语义） 已定项 1）。逐盘交回槽数。
 /// 最新根的树表或分配记录树读不出时一槽都不并（照旧判）。
 fn deferred_slots_referenced_only_below_the_floor_per_device(
     reader: &dyn ImageReader,
@@ -5664,30 +5657,27 @@ fn root_references_are_merged_into_the_deferred_slots_below_the_floor(
     }
 }
 
-/// 丁-defer 的门槛与分配器的回收门槛同一个：max(F_生效, 一圈窗口里最旧有效根)（D16（发布语义） 已定项 1，五处同改之一；
-/// `research/prompts/m3-ring-r3-main-verification.md` 第二节第 5 行）——释放代落在 (F_生效, 窗口里最旧有效根] 的槽分配器已经回收、
-/// 不在「已分配」里，再按 F_生效 并进来就多算。窗口里一条有效根都没有时取 F_生效。
+/// 丁-defer 的门槛与分配器的回收门槛同一个：max(F_生效, 环里最旧有效根)（D16（发布语义） 已定项 1；`research/prompts/m3-ring-r3-main-verification.md`
+/// 第二节第 5 行把它与回收门槛同取一圈窗口，用户 2026-10-04 定 RGall 一起退回整个环，`research/prompts/m3-reclaim-window-r1-main-verification.md`
+/// 第六节 Q1）——释放代落在 (F_生效, 环里最旧有效根] 的槽分配器已经回收、不在「已分配」里，再按 F_生效 并进来就多算。
+/// 环里一条有效根都没有时取 F_生效。
 fn deferred_slots_reclaim_threshold(
     standing_and_txg_of_each_readable_root: impl IntoIterator<
         Item = (RootStandingInTheNewestRootsTable, u64),
     >,
-    window: OneRevolutionWindow,
     effective_rollback_floor: u64,
 ) -> u64 {
     standing_and_txg_of_each_readable_root
         .into_iter()
-        .filter(|(standing, checkpoint_txg)| match standing {
-            RootStandingInTheNewestRootsTable::Valid => window.contains(*checkpoint_txg),
+        .filter(|(standing, _)| match standing {
+            RootStandingInTheNewestRootsTable::Valid => true,
             RootStandingInTheNewestRootsTable::Abandoned => false,
         })
         .map(|(_, checkpoint_txg)| checkpoint_txg)
         .min()
-        .map_or(
-            effective_rollback_floor,
-            |oldest_valid_root_in_the_window| {
-                oldest_valid_root_in_the_window.max(effective_rollback_floor)
-            },
-        )
+        .map_or(effective_rollback_floor, |oldest_valid_root_in_the_ring| {
+            oldest_valid_root_in_the_ring.max(effective_rollback_floor)
+        })
 }
 
 /// 最新根的账里这条记录还在 defer 里：已释放、释放代高于回收门槛（分配器还不许回收，记账照算已分配）。
@@ -7009,11 +6999,11 @@ const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
 /// 根记录里「树表 0 条那一版的分配记录树的根」86 在 342（D22（单元原子性怎么合成） 已定项 7 的偏移表）。
 const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT: std::ops::Range<usize> = 342..428;
 
-/// 一圈窗口（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」，用户 2026-10-02 定）：窗口 = txg > N − 3S（3S 是环长，根环槽数），
-/// 锚 N 由调用方取——候选过滤与 I-3.1 丁-defer 取镜像根环里读得出的最新有效根的 txg；I-7.9（回退下界 F 不高于抬 F 的上限）
-/// 另取一个锚在被判那条抬 F 的根的 txg 上的（`.claude/kb/invariants.md` I-7.9 那一行，2026-10-04 写回），两个一起用。同取这个窗口的五处里 checker 占两处：候选过滤（还有「由记录施加出来的那一版」
-/// 判回收用的最旧有效根）与 I-3.1（已分配统计对得上） 丁-defer 的门槛。窗口之外的有效根是环上有洞时留在洞那一槽里的上一圈的根，
-/// 分配器的回收门槛不再被它压住（`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupancy::oldest_valid_root_in_the_window`）。
+/// 一圈窗口：窗口 = txg > N − 3S（3S 是环长，根环槽数），锚 N 取镜像根环里择到的最新根的 txg。回收门槛、候选过滤、I-3.1（已分配统计对得上）
+/// 丁-defer 门槛、抬 F 的上限等原来同取一圈窗口的六处，用户 2026-10-04 定 RGall 一起退回「环里最旧有效根」
+/// （`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1），checker 里那两处不再用它。剩下的用处只有 I-7.9
+/// （回退下界 F 不高于抬 F 的上限）在后来的镜像上重算上限时挡洞里留下的旧根（[`judge_rollback_floor_raises_against_their_ceilings`]）：
+/// 镜像里 txg ≤ N − 3S 的根只能是洞那一槽留下的，比它新、抬 F 那一刻也在环里的根已被盖掉，拿它重算会把上限算低。
 #[derive(Clone, Copy, Debug)]
 struct OneRevolutionWindow {
     /// 锚 N。
@@ -7057,8 +7047,8 @@ impl VersionAppliedOnlyByRecords {
 ///    记录要带提交标记（D23（journal 的角色与格式） 已定项 7：不带的是恢复要丢的尾巴）。
 /// ② **它的根槽读不出**：根环里自证过的根没有一条是 (i, 这个 txg)。读得出的那一版照常由根环那一路走。
 /// ③ **不低于回退下界**：txg ≥ 最新根带的 F，与根环里的候选根同一条（D16（发布语义） 已定项 1）。
-/// ④ **它换下的单元还没到回收的时候**：一圈窗口（[`OneRevolutionWindow`]）里有一条没被实例表判抛弃的根，txg 比它小。
-///    理由：一个单元只被这一版引用时，它的释放代大于这一版的 txg；回收要「释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」
+/// ④ **它换下的单元还没到回收的时候**：根环里有一条没被实例表判抛弃的根，txg 比它小（RGall：原来只看一圈窗口里的根，用户 2026-10-04 定退回整个环）。
+///    理由：一个单元只被这一版引用时，它的释放代大于这一版的 txg；回收要「释放代 ≤ max(F_生效, 环里最旧有效根)」
 ///    （D3（空间分配） 已定项 7）。环里最旧的有效根比这一版老 ⇒ 最旧有效根 < 这一版 < 释放代，③ 又让 F ≤ 这一版 ⇒ 回收不了，
 ///    它们还算在已分配里；反过来环里最旧的有效根已经比这一版新，这一版里只有它引用的单元释放代都不超过那条根，已可回收复用，
 ///    再把它并进来就是拿已经不归它的槽去比。
@@ -7069,7 +7059,6 @@ fn versions_applied_only_by_records(
     roots: &[(u64, u64, crate::RootView)],
     instance_table_rows: &[InstanceTableRow],
     rollback_floor: u64,
-    window: OneRevolutionWindow,
 ) -> Vec<VersionAppliedOnlyByRecords> {
     let is_abandoned = |instance: u32, checkpoint_txg: u64| {
         instance_table_rows
@@ -7078,10 +7067,7 @@ fn versions_applied_only_by_records(
     };
     let Some(oldest_valid_root_txg) = roots
         .iter()
-        .filter(|(_, _, root)| {
-            !is_abandoned(root.instance, root.checkpoint_txg)
-                && window.contains(root.checkpoint_txg)
-        })
+        .filter(|(_, _, root)| !is_abandoned(root.instance, root.checkpoint_txg))
         .map(|(_, _, root)| root.checkpoint_txg)
         .min()
     else {
@@ -7807,11 +7793,30 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
     );
     // I-8.5（V-池）：全池皆废判红；一块盘交得出可择的系统配置就照常往下判（单盘全废走旁路报告，不进违例通道）。
     // 带越界值的槽、环几何三格不等的槽整池拒挂，上面那两支整池报不适用、不判这一条。
-    judge_some_device_carries_a_self_verified_system_configuration(
-        &system_configuration_slot_readings_of_each_device,
-        &mut root_ring_judgements,
-    );
+    // 一块盘都交不出可择的系统配置、而有槽自证过（incompat 位不认得、R / S 越界这类自证过而不可择的槽）：这不是一个挂得上的镜像，
+    // I-8.5 同别的不变量一起报不适用，不判成成立（`InvariantVerdict` 的「不适用必须带理由，不许静默算成立」）；一个槽都没自证过的照判红。
     if chosen.is_empty() {
+        let mut judgement_of_some_device_carrying_a_self_verified_slot = Judgements::default();
+        judge_some_device_carries_a_self_verified_system_configuration(
+            &system_configuration_slot_readings_of_each_device,
+            &mut judgement_of_some_device_carrying_a_self_verified_slot,
+        );
+        match judgement_of_some_device_carrying_a_self_verified_slot
+            .first_violation_of(SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION)
+        {
+            Some(detail) => {
+                let detail = detail.to_owned();
+                root_ring_judgements.judge(
+                    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
+                    false,
+                    || detail,
+                );
+            }
+            None => root_ring_judgements.not_applicable(
+                SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
+                "池里有系统配置槽自证过，而没有一块盘交得出可择的系统配置：这不是一个挂得上的镜像",
+            ),
+        }
         for invariant in crate::image::IMPLEMENTED_INVARIANTS {
             root_ring_judgements.not_applicable(
                 invariant,
@@ -7820,6 +7825,10 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         }
         return root_ring_judgements.into_report();
     }
+    judge_some_device_carries_a_self_verified_system_configuration(
+        &system_configuration_slot_readings_of_each_device,
+        &mut root_ring_judgements,
+    );
     let geometry = chosen[0].2;
     // 各盘择到的系统配置要属于同一个池：fsid 逐盘相同（一块别的池的旧盘插进来，实例代号可以恰好也是 1，I-7.7 看不出来）。
     for (device, view, _) in &chosen {
@@ -8077,7 +8086,8 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
             highest_floor_on_the_newest_valid_and_the_abandoned_roots,
             u64::max,
         );
-    // 一圈窗口（D16（发布语义） 已定项 1）：N 取最新根（它按自己的实例表恒有效），3S 是根环槽数。
+    // 一圈窗口：N 取最新根（它按自己的实例表恒有效），3S 是根环槽数。只给 I-7.9 挡洞里留下的旧根用（[`OneRevolutionWindow`]）；
+    // 候选过滤与丁-defer 门槛不再看它（RGall）。
     let window = OneRevolutionWindow {
         anchor_txg: newest_txg,
         root_ring_slot_count: geometry.regions * geometry.slots_per_region,
@@ -8085,7 +8095,6 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
     // 掉出遍历的根槽按理由各记一次（几样都占的各边都记）：I-3.1 判红时这几个数就是「遍历为什么少算」的机理标识。
     let mut root_slots_dropped_as_abandoned = 0u64;
     let mut root_slots_dropped_below_floor = 0u64;
-    let mut root_slots_dropped_outside_the_window = 0u64;
     let candidate_indexes: Vec<usize> = roots
         .iter()
         .enumerate()
@@ -8093,10 +8102,9 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
             // 被抛弃：按最新根指着的实例表判（D23（journal 的角色与格式） 已定项 14 候选集）。
             let abandoned = abandoned_by_the_newest_roots_table(root);
             let below_floor = root.checkpoint_txg < effective_rollback_floor;
-            // 一圈窗口之外（D16（发布语义） 已定项 1，五处同改之一）：环上有洞时洞那一槽里留着的上一圈的根。
-            let outside_the_window = !window.contains(root.checkpoint_txg);
-            let walked =
-                *index == newest_index || (!abandoned && !below_floor && !outside_the_window);
+            // 原来还有「一圈窗口之外」一格（环上有洞时洞那一槽里留着的上一圈的根）：候选过滤是原来同取一圈窗口的六处之一，
+            // 用户 2026-10-04 定 RGall 一起退回整个环，那条根照样是候选、照样走。
+            let walked = *index == newest_index || (!abandoned && !below_floor);
             if !walked {
                 if abandoned {
                     root_slots_dropped_as_abandoned += 1;
@@ -8104,9 +8112,6 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
                 if below_floor {
                     root_slots_dropped_below_floor += 1;
                 }
-                if outside_the_window {
-                    root_slots_dropped_outside_the_window += 1;
-                }
             }
             walked
         })
@@ -8149,7 +8154,6 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         &roots,
         &instance_table_rows,
         effective_rollback_floor,
-        window,
     );
     for version in &versions_applied_only_by_records {
         let mismatches_before = walk.judgements.violation_count("I-2.1");
@@ -8410,7 +8414,7 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
     // 不看各段的次序，所以插在「遍历的候选根槽」之后、与它挨着读。
     let mechanism = move || {
         format!(
-            "；机理：根环槽数 {root_ring_slot_count}、最新根 txg {newest_txg}、环里自证过的根槽 {readable_root_slot_count} 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 {walked_root_slot_count} 个、并进遍历的由记录施加出来的版本 {walked_versions_applied_only_by_records} 个、被实例表判抛弃的根槽 {root_slots_dropped_as_abandoned} 个、回退下界 F {effective_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个、一圈窗口之外的根槽 {root_slots_dropped_outside_the_window} 个"
+            "；机理：根环槽数 {root_ring_slot_count}、最新根 txg {newest_txg}、环里自证过的根槽 {readable_root_slot_count} 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 {walked_root_slot_count} 个、并进遍历的由记录施加出来的版本 {walked_versions_applied_only_by_records} 个、被实例表判抛弃的根槽 {root_slots_dropped_as_abandoned} 个、回退下界 F {effective_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"
         )
     };
     // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
@@ -8463,7 +8467,6 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
                 root.checkpoint_txg,
             )
         }),
-        window,
         effective_rollback_floor,
     );
     let deferred_below_the_floor_against_every_walked_version =
@@ -8711,7 +8714,7 @@ mod tests {
         );
     }
 
-    /// 一圈窗口（D16（发布语义） 已定项 1）：N = 35、环长 24 ⇒ 窗口 = txg > 11；N 小于环长时整个环都在窗口里。
+    /// 一圈窗口（I-7.9 挡洞里留下的旧根用的那一个）：N = 35、环长 24 ⇒ 窗口 = txg > 11；N 小于环长时整个环都在窗口里。
     #[test]
     fn the_one_revolution_window_keeps_the_txgs_above_the_newest_minus_the_ring_length() {
         let window = OneRevolutionWindow {
@@ -8733,14 +8736,14 @@ mod tests {
     // 取值域与上界（穷举的范围就是这几条）：
     // - 一块盘（盘 0），单元槽 0..SMALL_WORLD_UNIT_SLOT_COUNT（2 个），每个单元跨 1 槽；一版引用的单元是这 2 槽的任一子集。
     // - 根环 SMALL_WORLD_ROOT_RING_SLOT_COUNT（4）个位置，txg t 的根落在位置 t mod 4（与 `root_ring::target_for_publish` 一样按 txg 取槽），
-    //   一圈窗口照 `OneRevolutionWindow` 取 txg + 4 > N；位置上一条根被同余的新 txg 盖掉就出了环。
+    //   位置上一条根被同余的新 txg 盖掉就出了环（回收门槛与候选集取环里全部根，RGall；小世界里发布不跳号，环里的根本来都在一圈之内）。
     // - 从 mkfs（实例 1、txg 1、引用槽 0、F = 0）起，最多施加 SMALL_WORLD_OPERATION_LIMIT（5）步，每步三种之一：
     //   发布（同一实例 txg + 1，引用换成「现行引用 ∪ 可分配的槽」的任一子集：换下的记成已释放、释放代 = 新 txg，新引用的记成已分配）；
     //   抬 F（F 抬到 (F, 现行 txg] 里任一值：先写系统配置，再推一次带新 F 的空发布）；
-    //   崩溃恢复落到环里一条根 L（按现行那一版的表有效、txg ≥ F、在以现行 txg 为锚的窗口里，含现行那一版自己）：环里 txg 高于 L 的根
+    //   崩溃恢复落到环里一条根 L（按现行那一版的表有效、txg ≥ F，含现行那一版自己）：环里 txg 高于 L 的根
     //   所属的实例各记一行 (实例, L 的 txg)、并进 L 的表；新实例代号 = 用过的最大 + 1，写行 txg = 环里最大 txg + 1，写行那次发布从 L 起，
     //   引用照「发布」那样任选。
-    //   可分配的槽 = 现行没引用、账里没有记录或已释放且释放代 ≤ 回收门槛（max(F, 以现行 txg 为锚的窗口里最旧有效根)）、
+    //   可分配的槽 = 现行没引用、账里没有记录或已释放且释放代 ≤ 回收门槛（max(F, 环里最旧有效根)）、
     //   环里按现行那一版的表被抛弃的根都不引用（影子账隔离）的槽。
     // - 判的那一刻，环里有根的位置任取一个子集读不出（含现行那一版自己的根槽：那时最新根换成读得出的最大那条，账与表都取它的）。
     // - 不在小世界里的：由记录施加出来、根槽从没落盘的那一版（候选集另补的那一种），读得出而走读读到一半断掉的根，
@@ -8751,8 +8754,8 @@ mod tests {
     // 各算一遍丁-defer 并进的槽：选根一次用 `root_references_are_merged_into_the_deferred_slots_below_the_floor`（原式），
     // 一次用「被抛弃 ∨ txg < F」（被抛弃根引用的也并，LR5 的那一式）。两式只在这一形下可能不同：
     // 多并的槽 S 在最新根的账里是已释放、释放代 g，换下它的那一版 r（释放之前的现行那一版，引用 S）的根还在环里、而那一槽读不出——
-    // 一次故障是把根抛弃的那次恢复（读坏或崩溃），另一次是 r 的根槽读不出。r 读得出时：r ≥ F 且在窗口里就被候选集走到；
-    // r < F 原式已并进 S；r 在窗口之外或出了环，窗口里有效的根都 ≥ g、门槛 ≥ g，S 不算 defer。
+    // 一次故障是把根抛弃的那次恢复（读坏或崩溃），另一次是 r 的根槽读不出。r 读得出时：r ≥ F 就被候选集走到；
+    // r < F 原式已并进 S；r 出了环，环里有效的根都 ≥ g、门槛 ≥ g，S 不算 defer。
 
     const SMALL_WORLD_UNIT_SLOT_COUNT: u64 = 2;
     const SMALL_WORLD_ROOT_RING_SLOT_COUNT: u64 = 4;
@@ -8808,19 +8811,11 @@ mod tests {
         }
     }
 
-    fn small_world_window(newest_txg: u64) -> OneRevolutionWindow {
-        OneRevolutionWindow {
-            anchor_txg: newest_txg,
-            root_ring_slot_count: SMALL_WORLD_ROOT_RING_SLOT_COUNT,
-        }
-    }
-
-    /// 写进一条新根之前那一刻可分配的槽：门槛按 `instance_table_rows` 判有效、以 `anchor_txg` 为窗口的锚。
+    /// 写进一条新根之前那一刻可分配的槽：门槛按 `instance_table_rows` 判有效、取环里全部有效根。
     fn small_world_allocatable_unit_slots(
         pool: &SmallWorldPool,
         base: &SmallWorldRoot,
         instance_table_rows: &BTreeMap<u32, u64>,
-        anchor_txg: u64,
     ) -> BTreeSet<u64> {
         let reclaim_threshold = deferred_slots_reclaim_threshold(
             pool.ring.values().map(|root| {
@@ -8829,7 +8824,6 @@ mod tests {
                     root.version.checkpoint_txg,
                 )
             }),
-            small_world_window(anchor_txg),
             pool.system_configuration_floor,
         );
         let isolated_by_the_shadow_ledger: BTreeSet<u64> = pool
@@ -8919,12 +8913,8 @@ mod tests {
             checkpoint_txg: current.version.checkpoint_txg + 1,
         };
         // 发布。
-        let allocatable = small_world_allocatable_unit_slots(
-            pool,
-            current,
-            &current.instance_table_rows,
-            current.version.checkpoint_txg,
-        );
+        let allocatable =
+            small_world_allocatable_unit_slots(pool, current, &current.instance_table_rows);
         let reachable: BTreeSet<u64> = current
             .referenced_unit_slots
             .union(&allocatable)
@@ -8952,7 +8942,6 @@ mod tests {
             ));
         }
         // 崩溃恢复落到环里一条根 L。
-        let window = small_world_window(current.version.checkpoint_txg);
         let highest_txg_in_the_ring = pool
             .ring
             .values()
@@ -8963,8 +8952,7 @@ mod tests {
             let landed_txg = landed.version.checkpoint_txg;
             let can_land = small_world_standing(&current.instance_table_rows, landed.version)
                 == RootStandingInTheNewestRootsTable::Valid
-                && landed_txg >= pool.system_configuration_floor
-                && window.contains(landed_txg);
+                && landed_txg >= pool.system_configuration_floor;
             if !can_land {
                 continue;
             }
@@ -8986,12 +8974,8 @@ mod tests {
                 instance: recovered.highest_instance_used,
                 checkpoint_txg: highest_txg_in_the_ring + 1,
             };
-            let allocatable_after_the_recovery = small_world_allocatable_unit_slots(
-                &recovered,
-                landed,
-                &instance_table_rows,
-                landed_txg,
-            );
+            let allocatable_after_the_recovery =
+                small_world_allocatable_unit_slots(&recovered, landed, &instance_table_rows);
             let reachable_after_the_recovery: BTreeSet<u64> = landed
                 .referenced_unit_slots
                 .union(&allocatable_after_the_recovery)
@@ -9047,14 +9031,12 @@ mod tests {
             })
             .map(|root| root.rollback_floor)
             .fold(pool.system_configuration_floor, u64::max);
-        let window = small_world_window(newest.version.checkpoint_txg);
         let walked_slots: BTreeSet<(u32, u64)> = readable
             .iter()
             .filter(|root| {
                 root.version == newest.version
                     || (standing(root.version) == RootStandingInTheNewestRootsTable::Valid
-                        && root.version.checkpoint_txg >= effective_rollback_floor
-                        && window.contains(root.version.checkpoint_txg))
+                        && root.version.checkpoint_txg >= effective_rollback_floor)
             })
             .flat_map(|root| small_world_slot_keys(&root.referenced_unit_slots))
             .collect();
@@ -9062,7 +9044,6 @@ mod tests {
             readable
                 .iter()
                 .map(|root| (standing(root.version), root.version.checkpoint_txg)),
-            window,
             effective_rollback_floor,
         );
         let released_above_the_reclaim_threshold: BTreeSet<(u32, u64)> = newest
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index a8494c7..8447429 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -21,10 +21,7 @@ use crate::admission::SpaceAdmission;
 use crate::bytes::ByteWriter;
 use crate::instance_table::InstanceTableRecords;
 use crate::journal::slot_after_the_journal_ring;
-use crate::root_ring::{
-    target_for_publish, txg_is_inside_the_one_revolution_window, RootRingSlot,
-    RootRingSlotsPerRegion,
-};
+use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
 use crate::transaction::FrozenPublish;
 use crate::unlink::DeletionSwitches;
 
@@ -943,24 +940,63 @@ impl PlacementOnDevice {
     }
 }
 
-/// 根环一个槽上这会儿住着哪条根。自证不过的槽、读不出而落在一圈窗口之外的槽不在 `RootRingOccupancy` 里：D16（发布语义） 已定项 1 的
-/// 「一圈窗口里最旧有效根」只看自证合法的根与窗口里读不出的槽，读不出的被抛弃根影子账本来就罩不到（`mount::isolate_slots_referenced_only_by_abandoned_roots`）。
+/// 挂载那一刻读坏（设备报读错，或读得出、不是全零而自证不过）的一个根环槽能住的 txg（D16（发布语义） 已定项 1 的「挂载时读不出的槽」，
+/// 用户 2026-10-04 定：读不出、自证不过的根槽在回收门槛里钉在它可能住过的最旧 txg，重读读出有效根或确认为空之前门槛不越过它；
+/// 准入抬 F 的上限照 Q3 按它在 (N − 3S, 新实例第一次发布的 txg − 1] 里能住的最大 txg 并）。读回全零的槽另算「从没写过」，不在这里
+/// （用户 2026-10-04 定）。不变量：最旧的不高于最大的（构造时断言）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct TxgsARingSlotBadAtMountCanHold {
+    oldest_txg_it_can_have_held: CheckpointTxg,
+    highest_txg_it_can_hold: CheckpointTxg,
+}
+
+impl TxgsARingSlotBadAtMountCanHold {
+    /// # Panics
+    /// `oldest_txg_it_can_have_held` 高于 `highest_txg_it_can_hold`：两个数是从同一个槽、同一个 txg 上界算出来的，倒过来说明算错了。
+    #[must_use]
+    pub fn new(
+        oldest_txg_it_can_have_held: CheckpointTxg,
+        highest_txg_it_can_hold: CheckpointTxg,
+    ) -> Self {
+        assert!(
+            oldest_txg_it_can_have_held <= highest_txg_it_can_hold,
+            "一个槽可能住过的最旧 txg {oldest_txg_it_can_have_held:?} 不高于它能住的最大 txg {highest_txg_it_can_hold:?}"
+        );
+        Self {
+            oldest_txg_it_can_have_held,
+            highest_txg_it_can_hold,
+        }
+    }
+
+    /// 落到这个槽上、不晚于新实例第一次发布之前那个 txg 的最小 txg：回收门槛钉在这里（窗口之外留在洞里的旧根也罩得住）。
+    #[must_use]
+    pub fn oldest_txg_it_can_have_held(self) -> CheckpointTxg {
+        self.oldest_txg_it_can_have_held
+    }
+
+    /// 它在 (N − 3S, 新实例第一次发布的 txg − 1] 里能住的最大 txg：准入抬 F 的上限按它并（Q3）。
+    #[must_use]
+    pub fn highest_txg_it_can_hold(self) -> CheckpointTxg {
+        self.highest_txg_it_can_hold
+    }
+}
+
+/// 根环一个槽上这会儿住着哪条根。挂载那一刻读回全零的槽（从没写过）与一个 txg 都住不了的槽不在 `RootRingOccupancy` 里：
+/// 回收门槛取「环里最旧有效根」（D16（发布语义） 已定项 1，用户 2026-10-04 定 RGall，`research/prompts/m3-reclaim-window-r1-main-verification.md`
+/// 第六节 Q1），看自证合法的根与挂载时读坏的槽；读不出的被抛弃根影子账本来就罩不到（`mount::isolate_slots_referenced_only_by_abandoned_roots`）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum RootRingOccupant {
-    /// 挂载那一刻设备报读错的槽，落在挂载时的一圈窗口里（D16（发布语义） 已定项 1「挂载时读不出的槽在窗口里只对应一两个 txg：
-    /// 门槛按最小的算，根环表记最大的」，`research/prompts/m3-ring-r3-main-verification.md` 第二节第 4 行，零轮）：按「那里住着一条 txg 为
-    /// `highest_txg_it_can_hold` 的有效根」进这个进程的回收门槛，直到这个进程把那个槽写掉。它不算「这个进程知道住着根的槽」
-    /// （[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]）：D16 已定项 1「根槽这一次读坏」那一行，挂载那一刻就读不出、之后没写过的槽
-    /// 算上限、判回退候选时当没有根，不因为这里记了它就改成「读坏就拒」。
-    UnreadableAtMountInsideTheWindow {
-        highest_txg_it_can_hold: CheckpointTxg,
-    },
-    /// 挂载那一刻读不出的槽，这个进程在它成了窗口里最小的那个时重读、读出了一条自证过而按实例表仍然有效的根
+    /// 挂载那一刻读坏的槽（设备报读错，或读得出、不是全零而自证不过）：回收门槛钉在它可能住过的最旧 txg、准入抬 F 的上限按它能住的最大 txg
+    /// 并（[`TxgsARingSlotBadAtMountCanHold`]），直到这个进程把那个槽写掉，或进程内记根时重读读出有效根、读出被抛弃的根、读回全零
+    /// （用户 2026-10-04 定）。它不算「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]）：
+    /// D16 已定项 1「根槽这一次读坏」那一行，挂载那一刻就读坏、之后没写过的槽算上限时并进来、判回退候选时当没有根，不因为这里记了它就改成「读坏就拒」。
+    BadAtMount(TxgsARingSlotBadAtMountCanHold),
+    /// 挂载那一刻读坏的槽，这个进程在它成了环里最小的那个时重读、读出了一条自证过而按实例表仍然有效的根
     /// （D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表二之后那一段）：
-    /// 照读出来的 txg 进窗口。它算「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]；D16 已定项 1
+    /// 照读出来的 txg 进环里最旧有效根。它算「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]；D16 已定项 1
     /// 「根槽这一次读坏」那一行，用户 2026-10-04 定，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1）：
     /// 之后那一槽再读坏，算上限、判回退候选时重读一次、仍坏就拒，不按没有根往下走。
-    ValidRootReadWhenTheSlotUnreadableAtMountWasReread { checkpoint_txg: CheckpointTxg },
+    ValidRootReadWhenTheSlotBadAtMountWasReread { checkpoint_txg: CheckpointTxg },
     /// 按实例表判仍然有效的根，txg 在不在 F 之下都算（可再分配谓词取 max(F_生效, 环里最旧有效根)，F 那一半另记）。
     ValidRoot { checkpoint_txg: CheckpointTxg },
     /// 按实例表判被抛弃的根（D23（journal 的角色与格式） 已定项 14），带着它引用的每个落点——挂载那一刻仍被候选根或现行账引用、
@@ -989,7 +1025,7 @@ pub struct RootRingOccupancy {
 }
 
 impl RootRingOccupancy {
-    /// 从盘上读出来的那一刻：`occupants` 是每个读得出的根环槽上的根，`effective_floor` 是恢复后生效的 F，
+    /// 从盘上读出来的那一刻：`occupants` 是每个读得出的根环槽上的根与挂载时读坏的槽，`effective_floor` 是恢复后生效的 F，
     /// `latest_txg_before_the_next_publish` 是这个进程下一条根的 txg 减一（挂载时 = 新实例第一次发布的 txg − 1：
     /// 环里只有记录、没有根的那几个 txg 这时已经跳过，它们的槽上照旧是读出来的内容）。
     #[must_use]
@@ -1017,8 +1053,8 @@ impl RootRingOccupancy {
     /// 这个进程知道住着一条根的根环槽（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）：挂载那一刻读得出、自证过的槽
     /// （有效的与被抛弃的都算），这个进程之后写过、FUA 返回过的槽——发布在落盘之前或落盘途中失败时分配器整个换回发布之前那一份，
     /// 这张表随之退回，记着的只剩根落盘做成的那几次——，与进程内记根时重读读出了有效根的槽
-    /// （[`RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread`]，用户 2026-10-04 定）。挂载那一刻读不出或自证不过、
-    /// 之后又没写过也没重读出有效根的槽不在其内（[`RootRingOccupant::UnreadableAtMountInsideTheWindow`] 记着的槽也不在）。
+    /// （[`RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread`]，用户 2026-10-04 定）。挂载那一刻读坏、
+    /// 之后又没写过也没重读出有效根的槽不在其内（[`RootRingOccupant::BadAtMount`] 记着的槽也不在）。
     #[must_use]
     pub fn ring_slots_known_to_hold_a_root(&self) -> BTreeSet<RootRingSlot> {
         self.occupants
@@ -1026,38 +1062,29 @@ impl RootRingOccupancy {
             .filter(|(_, occupant)| match occupant {
                 RootRingOccupant::ValidRoot { .. }
                 | RootRingOccupant::AbandonedRoot { .. }
-                | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread { .. } => {
-                    true
-                }
-                RootRingOccupant::UnreadableAtMountInsideTheWindow { .. } => false,
+                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => true,
+                RootRingOccupant::BadAtMount(_) => false,
             })
             .map(|(ring_slot, _)| *ring_slot)
             .collect()
     }
 
-    /// 一圈窗口里最旧有效根的 txg（D16（发布语义） 已定项 1）：N = 这个进程记到的最新根的 txg（`latest_recorded_txg`：进程内每记一条根
-    /// 就是这条根），窗口 = txg > N − 3S（3S 是环长）；窗口里的有效根与挂载时窗口里读不出的槽（按它能住的最大 txg）取最小。一条都没有时 `None`。
-    /// 挂载那一刻的门槛不走这里（`mount::rebuilt_allocator` 锚在环里读得出的最新有效根、读不出的槽按最小 txg 算）。
-    fn oldest_valid_root_in_the_window(&self) -> Option<CheckpointTxg> {
+    /// 环里最旧有效根的 txg（D16（发布语义） 已定项 1「环里最旧有效根」，用户 2026-10-04 定 RGall：回收门槛从一圈窗口退回整个环）：
+    /// 这张表里全部有效根与挂载时读坏的槽（钉在它可能住过的最旧 txg，用户 2026-10-04 定）取最小，不按 txg 离这个进程记到的最新根多远滤。
+    /// 一条都没有时 `None`。
+    fn oldest_valid_root_in_the_ring(&self) -> Option<CheckpointTxg> {
         self.occupants
             .values()
             .filter_map(|occupant| match occupant {
                 RootRingOccupant::ValidRoot { checkpoint_txg, .. }
-                | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread {
+                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                     checkpoint_txg,
                 } => Some(*checkpoint_txg),
-                RootRingOccupant::UnreadableAtMountInsideTheWindow {
-                    highest_txg_it_can_hold,
-                } => Some(*highest_txg_it_can_hold),
+                RootRingOccupant::BadAtMount(txgs_it_can_hold) => {
+                    Some(txgs_it_can_hold.oldest_txg_it_can_have_held())
+                }
                 RootRingOccupant::AbandonedRoot { .. } => None,
             })
-            .filter(|checkpoint_txg| {
-                txg_is_inside_the_one_revolution_window(
-                    *checkpoint_txg,
-                    self.latest_recorded_txg,
-                    self.slots_per_region,
-                )
-            })
             .min()
     }
 
@@ -1070,105 +1097,112 @@ impl RootRingOccupancy {
                     referenced_placements,
                 } => referenced_placements.as_slice(),
                 RootRingOccupant::ValidRoot { .. }
-                | RootRingOccupant::UnreadableAtMountInsideTheWindow { .. }
-                | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread { .. } => {
-                    &[]
-                }
+                | RootRingOccupant::BadAtMount(_)
+                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => &[],
             })
             .copied()
             .collect()
     }
 
-    /// 挂载那一刻读不出、还记在表里的槽，各按它能住的最大 txg（[`RootRingOccupant::UnreadableAtMountInsideTheWindow`]），按槽的次序。
+    /// 挂载那一刻读坏、还记在表里的槽（[`RootRingOccupant::BadAtMount`]），各带它能住的 txg，按槽的次序。回收门槛与 E2 重读按其中
+    /// 可能住过的最旧 txg 取，准入抬 F 的上限按其中能住的最大 txg 并（`mount::ring_slots_bad_at_mount_by`）。
     #[must_use]
-    pub fn ring_slots_unreadable_at_mount(&self) -> BTreeMap<RootRingSlot, CheckpointTxg> {
+    pub fn ring_slots_bad_at_mount(
+        &self,
+    ) -> BTreeMap<RootRingSlot, TxgsARingSlotBadAtMountCanHold> {
         self.occupants
             .iter()
             .filter_map(|(ring_slot, occupant)| match occupant {
-                RootRingOccupant::UnreadableAtMountInsideTheWindow {
-                    highest_txg_it_can_hold,
-                } => Some((*ring_slot, *highest_txg_it_can_hold)),
+                RootRingOccupant::BadAtMount(txgs_it_can_hold) => {
+                    Some((*ring_slot, *txgs_it_can_hold))
+                }
                 RootRingOccupant::ValidRoot { .. }
-                | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread { .. }
+                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                 | RootRingOccupant::AbandonedRoot { .. } => None,
             })
             .collect()
     }
 
-    /// 进程内重读那几个槽时判「按实例表仍然有效」用的那张表（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」：按实例表判仍然有效）。
+    /// 挂载那一刻读坏、还记在表里的槽，各钉在它可能住过的最旧 txg：回收门槛与 E2 重读按它取。
+    #[must_use]
+    pub fn ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held(
+        &self,
+    ) -> BTreeMap<RootRingSlot, CheckpointTxg> {
+        self.ring_slots_bad_at_mount()
+            .into_iter()
+            .map(|(ring_slot, txgs_it_can_hold)| {
+                (ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held())
+            })
+            .collect()
+    }
+
+    /// 进程内重读那几个槽时判「按实例表仍然有效」用的那张表（D16（发布语义） 已定项 1「环里最旧有效根」：按实例表判仍然有效）。
     /// 装上根环表时是空表；可写挂载在写行那次发布的表一定下来、第一次记根之前换成它（`mount::establish_instance`）——
     /// 池级 checker 按镜像里最新那条根的实例表判，写行之后最新那条根带的就是写行那一张。挂载装上根环表到换表之间不记根，空表没人读；
-    /// mkfs 同一个进程里一直是空表（表里没有挂载时读不出的槽，用不到它）。
+    /// mkfs 同一个进程里一直是空表（表里没有挂载时读坏的槽，用不到它）。
     #[must_use]
     pub fn instance_table_judging_the_rereads(&self) -> &InstanceTableRecords {
         &self.instance_table_judging_the_rereads
     }
 }
 
-/// 进程内重读一个挂载那一刻读不出的根环槽读到了什么（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定；读得出而自证不过那一格
-/// 用户 2026-10-04 定改法 A，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1）。封闭集合，`match` 不写通配臂。
+/// 进程内重读一个挂载那一刻读坏的根环槽读到了什么（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定；用户 2026-10-04 定：
+/// 重读读出有效根或确认为空之前回收门槛不越过它，盘上全零的根槽另算「从没写过」）。封闭集合，`match` 不写通配臂。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub enum RereadOfARootRingSlotUnreadableAtMount {
-    /// 读得出、自证过，按实例表仍然有效：照读出来的 txg 算（窗口里的进窗口，窗口外的不算）；那一槽从此算「这个进程知道住着根的槽」。
+pub enum RereadOfARootRingSlotBadAtMount {
+    /// 读得出、自证过，按实例表仍然有效：照读出来的 txg 算（回收门槛取整个环，RGall，读出来多旧都算）；那一槽从此算「这个进程知道住着根的槽」。
     ValidRoot { checkpoint_txg: CheckpointTxg },
     /// 读得出、自证过，读出来的根按实例表被抛弃：当没有根。
     AbandonedByTheInstanceTable,
-    /// 仍读不出，或读得出而自证不过（一次读坏的有效根与从没写过的槽这一刻分不开）：照旧按它能住的最大 txg 算。
+    /// 读回全零：从没写过，确认为空、当没有根。
+    AllZeroNeverWritten,
+    /// 仍读不出，或读得出、不是全零而自证不过：照旧钉在它可能住过的最旧 txg。
     StillUnreadableOrNotSelfVerified,
 }
 
-/// 一圈窗口里最旧有效根，挂载那一刻读不出的槽按 E2 并进来（D16（发布语义） 已定项 1，用户 2026-10-03 定）：`valid_root_txgs` 是已知
-/// 住着有效根的 txg，`unreadable_at_mount` 是挂载那一刻读不出、各按能住的最大 txg 记的槽；两者里窗口（锚 `window_anchor`）里最小的那个
-/// 若是一个读不出的槽，先重读那一槽（`reread`）：读出有效根就照它的 txg 换进去、读出被抛弃的根就拿掉，再取最小；
-/// 仍读不出或读得出而自证不过才按最大 txg 算（自证不过那一格用户 2026-10-04 定改法 A）。
+/// 环里最旧有效根，挂载那一刻读坏的槽按 E2 并进来（D16（发布语义） 已定项 1，用户 2026-10-03 定；回收门槛取整个环，用户 2026-10-04 定
+/// RGall；读坏的槽钉在它可能住过的最旧 txg，用户 2026-10-04 定）：`valid_root_txgs` 是已知住着有效根的 txg，`bad_at_mount` 是挂载那一刻
+/// 读坏、各按它可能住过的最旧 txg 钉着的槽；两者里最小的那个若是一个读坏的槽，先重读那一槽（`reread`）：读出有效根就照它的 txg 换进去、
+/// 读出被抛弃的根或读回全零就拿掉，再取最小；仍读不出或读得出而自证不过才按钉着的 txg 算。
 /// 交回最小的 txg（一条都没有时 `None`）与每个重读过的槽读到了什么（按重读的次序）。
 ///
-/// 循环：每轮要么换掉或拿掉一个读不出的槽（`unreadable_at_mount` 少一个），要么在最小的是一个仍读不出的槽、或最小的不是读不出的槽时返回；
-/// 轮数至多 `unreadable_at_mount` 的条数加一。跨轮携带两张表与重读结局。
-pub fn oldest_txg_in_the_window_rereading_slots_unreadable_at_mount(
+/// 循环：每轮要么换掉或拿掉一个读坏的槽（`bad_at_mount` 少一个），要么在最小的是一个仍读坏的槽、或最小的不是读坏的槽时返回；
+/// 轮数至多 `bad_at_mount` 的条数加一。跨轮携带两张表与重读结局。
+pub fn oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
     valid_root_txgs: &[CheckpointTxg],
-    mut unreadable_at_mount: BTreeMap<RootRingSlot, CheckpointTxg>,
-    window_anchor: CheckpointTxg,
-    slots_per_region: RootRingSlotsPerRegion,
-    reread: &mut dyn FnMut(RootRingSlot) -> RereadOfARootRingSlotUnreadableAtMount,
+    mut bad_at_mount: BTreeMap<RootRingSlot, CheckpointTxg>,
+    reread: &mut dyn FnMut(RootRingSlot) -> RereadOfARootRingSlotBadAtMount,
 ) -> (
     Option<CheckpointTxg>,
-    Vec<(RootRingSlot, RereadOfARootRingSlotUnreadableAtMount)>,
+    Vec<(RootRingSlot, RereadOfARootRingSlotBadAtMount)>,
 ) {
-    let is_inside_the_window = |checkpoint_txg: CheckpointTxg| {
-        txg_is_inside_the_one_revolution_window(checkpoint_txg, window_anchor, slots_per_region)
-    };
     let mut valid_root_txgs: Vec<CheckpointTxg> = valid_root_txgs.to_vec();
     let mut rereads = Vec::new();
     loop {
-        let oldest_valid_root = valid_root_txgs
-            .iter()
-            .copied()
-            .filter(|checkpoint_txg| is_inside_the_window(*checkpoint_txg))
-            .min();
-        let oldest_unreadable_slot = unreadable_at_mount
+        let oldest_valid_root = valid_root_txgs.iter().copied().min();
+        let oldest_bad_slot = bad_at_mount
             .iter()
-            .filter(|(_, highest_txg_it_can_hold)| is_inside_the_window(**highest_txg_it_can_hold))
-            .min_by_key(|(_, highest_txg_it_can_hold)| **highest_txg_it_can_hold)
-            .map(|(ring_slot, highest_txg_it_can_hold)| (*ring_slot, *highest_txg_it_can_hold));
-        let Some((ring_slot, highest_txg_it_can_hold)) = oldest_unreadable_slot else {
+            .min_by_key(|(_, pinned_txg)| **pinned_txg)
+            .map(|(ring_slot, pinned_txg)| (*ring_slot, *pinned_txg));
+        let Some((ring_slot, pinned_txg)) = oldest_bad_slot else {
             return (oldest_valid_root, rereads);
         };
-        if oldest_valid_root.is_some_and(|oldest| oldest < highest_txg_it_can_hold) {
+        if oldest_valid_root.is_some_and(|oldest| oldest < pinned_txg) {
             return (oldest_valid_root, rereads);
         }
         let reread_of_the_slot = reread(ring_slot);
         rereads.push((ring_slot, reread_of_the_slot));
         match reread_of_the_slot {
-            RereadOfARootRingSlotUnreadableAtMount::ValidRoot { checkpoint_txg } => {
-                unreadable_at_mount.remove(&ring_slot);
+            RereadOfARootRingSlotBadAtMount::ValidRoot { checkpoint_txg } => {
+                bad_at_mount.remove(&ring_slot);
                 valid_root_txgs.push(checkpoint_txg);
             }
-            RereadOfARootRingSlotUnreadableAtMount::AbandonedByTheInstanceTable => {
-                unreadable_at_mount.remove(&ring_slot);
+            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable
+            | RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten => {
+                bad_at_mount.remove(&ring_slot);
             }
-            RereadOfARootRingSlotUnreadableAtMount::StillUnreadableOrNotSelfVerified => {
-                return (Some(highest_txg_it_can_hold), rereads);
+            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified => {
+                return (Some(pinned_txg), rereads);
             }
         }
     }
@@ -1830,10 +1864,10 @@ impl PoolAllocator {
         let displaced = occupancy
             .occupants
             .insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });
-        // 与挂载时同一个谓词（一处定义）：max(F_生效, 一圈窗口里最旧有效根)。
+        // 与挂载时同一个谓词（一处定义）：max(F_生效, 环里最旧有效根)（RGall）。
         let reclaim_floor = crate::mount::reclaim_floor(
             occupancy.effective_floor,
-            occupancy.oldest_valid_root_in_the_window(),
+            occupancy.oldest_valid_root_in_the_ring(),
         );
         let still_referenced_by_abandoned_roots =
             occupancy.placements_referenced_by_abandoned_roots();
@@ -1846,8 +1880,8 @@ impl PoolAllocator {
             ),
             Some(
                 RootRingOccupant::ValidRoot { .. }
-                | RootRingOccupant::UnreadableAtMountInsideTheWindow { .. }
-                | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread { .. },
+                | RootRingOccupant::BadAtMount(_)
+                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. },
             )
             | None => {}
         }
@@ -1859,14 +1893,16 @@ impl PoolAllocator {
         );
     }
 
-    /// 同 [`Self::record_root_written_by_this_process`]，记之前先照 E2 重读挂载那一刻读不出、记了这条根之后会是窗口里最小的那个槽
-    /// （D16（发布语义） 已定项 1，用户 2026-10-03 定：读得出且自证过就照读出来的根算——按实例表有效的照它的 txg（窗口外的不进窗口），
-    /// 被抛弃的当没有；仍读不出或读得出而自证不过按它能住的最大 txg 算，用户 2026-10-04 定）。`reread` 读盘、判自证与实例表（`mount::reread_of_a_root_ring_slot_unreadable_at_mount`）；
-    /// 这里只定重读哪一槽、按读到的改表。读不出的槽一个都不在窗口里最小的位置上时一个都不读。没装根环时只记根（什么都不做）。
-    pub fn record_root_written_by_this_process_rereading_slots_unreadable_at_mount(
+    /// 同 [`Self::record_root_written_by_this_process`]，记之前先照 E2 重读挂载那一刻读坏、记了这条根之后会是环里最小的那个槽
+    /// （D16（发布语义） 已定项 1，用户 2026-10-03 定；用户 2026-10-04 定：读坏的槽钉在它可能住过的最旧 txg，重读读出有效根或确认为空之前
+    /// 回收门槛不越过它——读得出且自证过就照读出来的根算，按实例表有效的照它的 txg（回收门槛取整个环，RGall，多旧都算），被抛弃的当没有；
+    /// 读回全零的是从没写过，当没有；仍读不出或读得出而自证不过照旧钉着）。
+    /// `reread` 读盘、判全零、自证与实例表（`mount::reread_of_a_root_ring_slot_bad_at_mount`）；
+    /// 这里只定重读哪一槽、按读到的改表。读坏的槽一个都不在最小的位置上时一个都不读。没装根环时只记根（什么都不做）。
+    pub fn record_root_written_by_this_process_rereading_slots_bad_at_mount(
         &mut self,
         checkpoint_txg: CheckpointTxg,
-        reread: &mut dyn FnMut(RootRingSlot) -> RereadOfARootRingSlotUnreadableAtMount,
+        reread: &mut dyn FnMut(RootRingSlot) -> RereadOfARootRingSlotBadAtMount,
     ) {
         if let Some(occupancy) = self.root_ring.as_mut() {
             let ring_slot_of_this_root =
@@ -1880,39 +1916,40 @@ impl PoolAllocator {
                     RootRingOccupant::ValidRoot {
                         checkpoint_txg: txg_of_the_occupant,
                     }
-                    | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread {
+                    | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                         checkpoint_txg: txg_of_the_occupant,
                     } => Some(*txg_of_the_occupant),
-                    RootRingOccupant::UnreadableAtMountInsideTheWindow { .. }
-                    | RootRingOccupant::AbandonedRoot { .. } => None,
+                    RootRingOccupant::BadAtMount(_) | RootRingOccupant::AbandonedRoot { .. } => {
+                        None
+                    }
                 })
                 .chain(std::iter::once(checkpoint_txg))
                 .collect();
-            let mut unreadable_at_mount = occupancy.ring_slots_unreadable_at_mount();
-            unreadable_at_mount.remove(&ring_slot_of_this_root);
-            let (_oldest, rereads) = oldest_txg_in_the_window_rereading_slots_unreadable_at_mount(
+            let mut bad_at_mount =
+                occupancy.ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held();
+            bad_at_mount.remove(&ring_slot_of_this_root);
+            let (_oldest, rereads) = oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
                 &valid_root_txgs,
-                unreadable_at_mount,
-                checkpoint_txg,
-                occupancy.slots_per_region,
+                bad_at_mount,
                 reread,
             );
             for (ring_slot, reread_of_the_slot) in rereads {
                 match reread_of_the_slot {
-                    RereadOfARootRingSlotUnreadableAtMount::ValidRoot {
+                    RereadOfARootRingSlotBadAtMount::ValidRoot {
                         checkpoint_txg: txg_read_on_the_reread,
                     } => {
                         occupancy.occupants.insert(
                             ring_slot,
-                            RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread {
+                            RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                                 checkpoint_txg: txg_read_on_the_reread,
                             },
                         );
                     }
-                    RereadOfARootRingSlotUnreadableAtMount::AbandonedByTheInstanceTable => {
+                    RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable
+                    | RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten => {
                         occupancy.occupants.remove(&ring_slot);
                     }
-                    RereadOfARootRingSlotUnreadableAtMount::StillUnreadableOrNotSelfVerified => {}
+                    RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified => {}
                 }
             }
         }
@@ -2721,48 +2758,99 @@ mod tests {
         }
     }
 
-    /// E2 那一判的取法（D16（发布语义） 已定项 1，用户 2026-10-03 定）：12 槽的环（S = 4），锚 txg 23，窗口 txg > 11；txg 13–23 住着有效根，
-    /// 区域 0 槽 0 挂载那一刻读不出、按它能住的最大 txg 12 记着，是窗口里最小的那个 ⇒ 重读那一槽一次。
-    fn oldest_in_the_window_when_the_slot_unreadable_at_mount_rereads_as(
-        reread_of_the_slot: RereadOfARootRingSlotUnreadableAtMount,
+    /// 环上有洞留下的旧根压住进程内的回收门槛（D16（发布语义） 已定项 1「环里最旧有效根」，用户 2026-10-04 定 RGall）：24 槽的环里
+    /// txg 1 那一槽留着 txg 1 的根（txg 25、49 那两次的根槽都没落），其余 23 槽住着 txg 26–48；这个进程记到 txg 49（没有根的那一个）
+    /// 之后再记 txg 50，盖掉的是 txg 26。txg 1 离最新根已过两圈，仍是环里最旧的有效根：释放代 2 的落点不回收、仍在 defer 里。
+    /// 照一圈窗口取（锚 50，窗口 txg > 26）时 txg 1 被滤掉、门槛跳到 27，那两槽当场回收。
+    #[test]
+    fn root_ring_occupancy_keeps_the_reclaim_floor_at_the_oldest_valid_root_however_far_below_the_newest_it_is(
+    ) {
+        let mut pool = rebuilt_pool(records_on_both_devices(&[(50180, 2, 2, true)]));
+        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
+        let ring_length = 3 * slots_per_region.count();
+        let occupants: Vec<(RootRingSlot, RootRingOccupant)> = std::iter::once(1)
+            .chain(ring_length + 2..=2 * ring_length)
+            .map(|txg| {
+                (
+                    target_for_publish(CheckpointTxg(txg), slots_per_region),
+                    RootRingOccupant::ValidRoot {
+                        checkpoint_txg: CheckpointTxg(txg),
+                    },
+                )
+            })
+            .collect();
+        assert_eq!(
+            occupants.len(),
+            usize::try_from(ring_length).expect("环长"),
+            "每个槽上一条根"
+        );
+        pool.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
+            slots_per_region,
+            occupants,
+            CheckpointTxg(0),
+            CheckpointTxg(2 * ring_length + 1),
+        ));
+        pool.record_root_written_by_this_process(CheckpointTxg(2 * ring_length + 2));
+        for device_map in &pool.devices {
+            assert!(
+                !device_map.is_free(SlotNumber(50180)) && !device_map.is_free(SlotNumber(50181)),
+                "盘 {:?}：txg 1 的根离最新根 txg {} 已过两圈，仍是环里最旧的有效根，释放代 2 的落点不回收",
+                device_map.device,
+                2 * ring_length + 2
+            );
+            assert_eq!(
+                device_map.deferred_slots(),
+                2,
+                "释放代 2 的那两槽仍在 defer 里"
+            );
+        }
+    }
+
+    /// E2 那一判的取法（D16（发布语义） 已定项 1，用户 2026-10-03 定；读坏的槽钉在它可能住过的最旧 txg，用户 2026-10-04 定）：
+    /// 12 槽的环（S = 4）；txg 13–23 住着有效根，区域 0 槽 0 挂载那一刻读坏、钉在它可能住过的最旧 txg 0（它能住的最大 txg 是 12），
+    /// 是环里最小的那个 ⇒ 重读那一槽一次。
+    fn oldest_in_the_ring_when_the_slot_bad_at_mount_rereads_as(
+        reread_of_the_slot: RereadOfARootRingSlotBadAtMount,
     ) -> (
         Option<CheckpointTxg>,
-        Vec<(RootRingSlot, RereadOfARootRingSlotUnreadableAtMount)>,
+        Vec<(RootRingSlot, RereadOfARootRingSlotBadAtMount)>,
     ) {
         let slots_per_region =
             RootRingSlotsPerRegion::from_system_configuration_field(4).expect("4 在区间里");
         let hidden_slot = RootRingSlot { region: 0, slot: 0 };
         assert_eq!(
-            target_for_publish(CheckpointTxg(12), slots_per_region),
-            hidden_slot
+            (
+                target_for_publish(CheckpointTxg(0), slots_per_region),
+                target_for_publish(CheckpointTxg(12), slots_per_region)
+            ),
+            (hidden_slot, hidden_slot)
         );
         let valid_root_txgs: Vec<CheckpointTxg> = (13..=23).map(CheckpointTxg).collect();
-        oldest_txg_in_the_window_rereading_slots_unreadable_at_mount(
+        oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
             &valid_root_txgs,
-            BTreeMap::from([(hidden_slot, CheckpointTxg(12))]),
-            CheckpointTxg(23),
-            slots_per_region,
+            BTreeMap::from([(hidden_slot, CheckpointTxg(0))]),
             &mut |ring_slot| {
-                assert_eq!(ring_slot, hidden_slot, "只重读窗口里最小的那一槽");
+                assert_eq!(ring_slot, hidden_slot, "只重读环里最小的那一槽");
                 reread_of_the_slot
             },
         )
     }
 
-    /// 重读读出的是上一圈的 txg 0（窗口外）：照读出来的根算、那一槽不在窗口里，最旧有效根是 13。
+    /// 重读读出的是上一圈的 txg 0（离锚 23 已过一圈）：回收门槛取整个环（RGall），照读出来的根算，最旧有效根是 0。
     #[test]
-    fn the_slot_unreadable_at_mount_rereading_as_a_root_outside_the_window_leaves_the_window() {
+    fn the_slot_bad_at_mount_rereading_as_a_root_of_the_previous_revolution_counts_by_the_txg_read()
+    {
         assert_eq!(
-            oldest_in_the_window_when_the_slot_unreadable_at_mount_rereads_as(
-                RereadOfARootRingSlotUnreadableAtMount::ValidRoot {
+            oldest_in_the_ring_when_the_slot_bad_at_mount_rereads_as(
+                RereadOfARootRingSlotBadAtMount::ValidRoot {
                     checkpoint_txg: CheckpointTxg(0),
                 }
             ),
             (
-                Some(CheckpointTxg(13)),
+                Some(CheckpointTxg(0)),
                 vec![(
                     RootRingSlot { region: 0, slot: 0 },
-                    RereadOfARootRingSlotUnreadableAtMount::ValidRoot {
+                    RereadOfARootRingSlotBadAtMount::ValidRoot {
                         checkpoint_txg: CheckpointTxg(0),
                     }
                 )]
@@ -2770,42 +2858,56 @@ mod tests {
         );
     }
 
-    /// 重读仍读不出：照旧按它能住的最大 txg 12 算。
+    /// 重读仍读不出：照旧钉在它可能住过的最旧 txg 0，不按它能住的最大 txg 12 算。
     #[test]
-    fn the_slot_unreadable_at_mount_still_unreadable_on_the_reread_counts_by_its_highest_txg() {
+    fn the_slot_bad_at_mount_still_unreadable_on_the_reread_stays_pinned_at_the_oldest_txg_it_can_have_held(
+    ) {
         assert_eq!(
-            oldest_in_the_window_when_the_slot_unreadable_at_mount_rereads_as(
-                RereadOfARootRingSlotUnreadableAtMount::StillUnreadableOrNotSelfVerified
+            oldest_in_the_ring_when_the_slot_bad_at_mount_rereads_as(
+                RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
             ),
             (
-                Some(CheckpointTxg(12)),
+                Some(CheckpointTxg(0)),
                 vec![(
                     RootRingSlot { region: 0, slot: 0 },
-                    RereadOfARootRingSlotUnreadableAtMount::StillUnreadableOrNotSelfVerified
+                    RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
                 )]
             )
         );
     }
 
-    /// 窗口里还有比那一槽能住的最大 txg 更旧的有效根：不重读（重读的口子被调到就断言失败）。
+    /// 重读读回全零：从没写过，确认为空，那一槽拿掉，最旧有效根是 13。
     #[test]
-    fn the_slot_unreadable_at_mount_is_not_reread_while_an_older_valid_root_is_in_the_window() {
-        let slots_per_region =
-            RootRingSlotsPerRegion::from_system_configuration_field(4).expect("4 在区间里");
-        // txg 12 与那一槽同槽，不在有效根里。
-        let valid_root_txgs: Vec<CheckpointTxg> = std::iter::once(11)
-            .chain(13..=22)
+    fn the_slot_bad_at_mount_reading_back_all_zero_on_the_reread_is_taken_as_never_written() {
+        assert_eq!(
+            oldest_in_the_ring_when_the_slot_bad_at_mount_rereads_as(
+                RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten
+            ),
+            (
+                Some(CheckpointTxg(13)),
+                vec![(
+                    RootRingSlot { region: 0, slot: 0 },
+                    RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten
+                )]
+            )
+        );
+    }
+
+    /// 环里还有比那一槽钉着的 txg 更旧的有效根：不重读（重读的口子被调到就断言失败）。
+    #[test]
+    fn the_slot_bad_at_mount_is_not_reread_while_an_older_valid_root_is_in_the_ring() {
+        // 区域 0 槽 1 可能住过的最旧 txg 是 3，环里还有 txg 2 的有效根。
+        let valid_root_txgs: Vec<CheckpointTxg> = std::iter::once(2)
+            .chain(4..=13)
             .map(CheckpointTxg)
             .collect();
         assert_eq!(
-            oldest_txg_in_the_window_rereading_slots_unreadable_at_mount(
+            oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
                 &valid_root_txgs,
-                BTreeMap::from([(RootRingSlot { region: 0, slot: 0 }, CheckpointTxg(12))]),
-                CheckpointTxg(22),
-                slots_per_region,
-                &mut |ring_slot| panic!("窗口里 txg 11 比那一槽的 12 旧，不该重读 {ring_slot:?}"),
+                BTreeMap::from([(RootRingSlot { region: 0, slot: 1 }, CheckpointTxg(3))]),
+                &mut |ring_slot| panic!("环里 txg 2 比那一槽钉着的 3 旧，不该重读 {ring_slot:?}"),
             ),
-            (Some(CheckpointTxg(11)), Vec::new())
+            (Some(CheckpointTxg(2)), Vec::new())
         );
     }
 
diff --git a/crates/singlefs-core/src/instance_table_sweep.rs b/crates/singlefs-core/src/instance_table_sweep.rs
index 5e56557..ab7d7aa 100644
--- a/crates/singlefs-core/src/instance_table_sweep.rs
+++ b/crates/singlefs-core/src/instance_table_sweep.rs
@@ -5,15 +5,18 @@
 //! 次序是先判、后抹（F6）：判根环每个槽都自证过、环里没有该实例的根，判池中每一个落点（[`judge_the_instance_table_sweep`]），
 //! 判过了才抹（FUA 写零头所在的那一个物理块，之后每块盘一道屏障），抹头全部返回之后才发清扫发布
 //! （[`erase_the_heads_and_publish_the_sweep`]）。这次清扫里有一次读失败、设备不全、根环有一槽读不出或全零或重读一次仍自证不过，
-//! 这一轮一行都不删、一个头都不抹。清扫发布的落点被拒时这一轮不删行，挂载照样交回可写（Q4「不单列，被拒就这一轮不删」）。
+//! 这一轮一行都不删、一个头都不抹；抹之前那一遍重读报设备错或读回的不是判的那个头，同样一行都不删、一个头都不抹
+//! （用户 2026-10-04 定 C，`research/prompts/m3-root-ring-code-r3-main-verification.md` 第六节 Q2）。
+//! 清扫发布的落点被拒时这一轮不删行，挂载照样交回可写（Q4「不单列，被拒就这一轮不删」）。
 //!
 //! 谁跑、什么时候跑（Q1）：可写挂载末尾（写行、暖机、抬 F、删除续做之后，交回之前）判一次——开关关着时实例表行数超过阈值就在这次挂载里
 //! 立刻判、抹、发，没超过不扫；开关开着时交给挂着时的后台清扫（[`sweep_the_instance_table_while_mounted`]），后台抹之前在同一串行段里
-//! 重读那一槽的头，仍是候选行的未发布单元才抹（F1）。挂着时什么时候调度后台那一轮，这一批没做（入口在，调度器没有）。
+//! 重读要抹的头，全部仍是候选行的未发布单元才抹（F1；有一个不是，这一轮不删，C）。挂着时什么时候调度后台那一轮，这一批没做（入口在，调度器没有）。
 //!
 //! 池中每一个落点怎么读（Q3「两条都要」、S2-2「按整单元跳」）：每块盘从单元区起点按槽升序读头，先照认领规则认单元（D18 已定项 7：解出一个头
 //! 就按它的整单元跨度把后面的槽认领掉——码 1、码 3 两槽，码 2 一槽），再跳过根环里任一可读根的账里仍分配的槽；认「是单元」只看头校验和，
-//! 不要求载荷校验和过（不比任何读盘判定更严）。
+//! 不要求载荷校验和过（不比任何读盘判定更严）。根环里 txg 低于现行那一版 F 的根跳过它的账（不读、不算），其余的根一条账读不出就这一轮不删
+//! （用户 2026-10-04 定 A2，同一份判决第六节 Q1：F 之下的根回收门槛已许可它独占的槽被复用，它的分配记录树节点可以已被合法盖掉）。
 //!
 //! 第一版不做的：现行那一版树表 0 条时不清扫（[`NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion`]：
 //! 那一版的账住根记录那一项、不在树表里，「跳过根环里可读根的账」那一半读不全；清扫发布在那一版上的形状条款也没写）。
@@ -279,7 +282,7 @@ pub enum NoRowDeletedThisRound {
     SystemConfigurationNotChosen { failure: RecoveryFailure },
     /// 现行那一版的实例表读不出：判不了哪几行是候选。
     InstanceTableOfTheCurrentVersionUnreadable { failure: RecoveryFailure },
-    /// 根环里一条可读根的账读不出：「跳过它的账里仍分配的槽」那一半判不了。
+    /// 根环里一条 txg 不低于现行那一版 F 的可读根的账读不出：「跳过它的账里仍分配的槽」那一半判不了（F 之下的根不读它的账，A2）。
     AccountOfARingRootUnreadable {
         root: RollbackTarget,
         failure: RecoveryFailure,
@@ -293,6 +296,21 @@ pub enum NoRowDeletedThisRound {
     },
     /// 现行那一版树表 0 条：第一版不清扫（模块文档「第一版不做的」）。在任何写之前判出，一个头都不抹。
     SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion,
+    /// 抹之前那一遍重读，一个要抹的头报设备错或读回的不是判的那个头（用户 2026-10-04 定 C）：这一轮不删，一个头都不抹
+    /// （先把要抹的头全部重读一遍、都还是判的那个头才开始抹，[`erase_the_heads_and_publish_the_sweep`]）。
+    AHeadRereadBeforeErasingIsNotTheJudgedUnit {
+        placement: PlacementOfAUnitHead,
+        reread: HeadRereadBeforeErasing,
+    },
+}
+
+/// 抹之前那一遍重读一个要抹的头读到了什么（[`NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum HeadRereadBeforeErasing {
+    /// 读失败（设备报错）。
+    ReadFailed,
+    /// 读得出，而认出来的头不是判的那一刻那个（已抹、被覆盖、读回全零这类认不出头的也在这里），或不再按候选行判未发布。
+    NotTheJudgedUnit,
 }
 
 /// 判的那一半交回的：要删的行、要抹的头，与判的那一刻池里认出来的单元个数（交给用例看扫没扫全）。
@@ -347,7 +365,12 @@ pub fn judge_the_instance_table_sweep(
     let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
     let mut slots_allocated_in_an_account_of_the_ring: BTreeSet<PlacementOfAUnitHead> =
         BTreeSet::new();
-    for root in &roots_of_the_ring {
+    // A2（用户 2026-10-04 定）：txg 低于现行那一版 F 的根不读它的账——正常卸载抬过 F 之后，F 之下的旧根的分配记录树节点被合法复用，
+    // 那条根的账读不出，照「一条读不出就不删」每一轮都停在这里、一行都删不掉（判决第二节 K1-a）。
+    for root in roots_of_the_ring
+        .iter()
+        .filter(|root| root.checkpoint_txg >= current_root.rollback_floor)
+    {
         let records = allocation_records_under_root_in_the_unit_area_starting_at(
             reader,
             unit_area_start,
@@ -478,7 +501,8 @@ fn every_root_of_the_ring_self_verified(
 pub enum InstanceTableSweepOutcome {
     /// 判的那一半就停了，一个头都没抹。
     NoRowDeleted(NoRowDeletedThisRound),
-    /// 抹完头、清扫发布落盘：删了这几行。`heads_erased` 是真抹了的（后台那一支重读之后已经不是候选行未发布单元的不抹，不在里面）。
+    /// 抹完头、清扫发布落盘：删了这几行。`heads_erased` 是抹了的头，就是判的那一半要抹的全部（抹之前那一遍重读有一个不是判的那个头时
+    /// 这一轮不删，走 [`NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit`]）。
     RowsDeleted {
         rows_deleted: Vec<InstanceRow>,
         heads_erased: Vec<PlacementOfAUnitHead>,
@@ -502,8 +526,10 @@ pub enum InstanceTableSweepOutcome {
     },
 }
 
-/// 抹头、过屏障、发清扫发布（判之后那一半）。每个要抹的头抹之前在同一串行段里重读那一槽的头：仍是判的那一刻读出来的那个单元
-/// （类、写序实例、诞生代号、事务号都相同）才抹——判完到抹之间有分配写掉了那一槽的，不抹（F1；挂载末尾那一支两步之间没有别的写，重读只多读一次）。
+/// 抹头、过屏障、发清扫发布（判之后那一半）。抹之前在同一串行段里把要抹的头全部重读一遍：每一个都仍是判的那一刻读出来的那个单元
+/// （类、写序实例、诞生代号、事务号都相同、仍按候选行判未发布）才开始抹；有一个报设备错或不是那个头，这一轮不删、一个头都不抹
+/// （用户 2026-10-04 定 C；F1 原来是「那一个不抹、其余照抹照删」，判完到抹之间有分配写掉了那一槽也走这一支；挂载末尾那一支两步之间没有别的写，
+/// 重读只多读一次）。
 /// 清扫发布接在 `current` 后面：同一实例、事务号 0、不进 W、实例表整链重写成「这一刻现行那一版的表去掉要删的行」、不判挂载准入式子
 /// （一个普通分配都没有的发布不判，`transaction::prepare_the_version_publish`）。发成了 `current` 换成清扫发布那一版。
 pub fn erase_the_heads_and_publish_the_sweep<Device: BlockDevice>(
@@ -535,23 +561,39 @@ pub fn erase_the_heads_and_publish_the_sweep<Device: BlockDevice>(
         .iter()
         .map(|row| (row.instance, *row))
         .collect();
-    let mut heads_erased = Vec::new();
-    // 迭代上界是要抹的头数；跨轮携带的只有抹了的那几个；提前出口是块设备错。
+    // 先重读、后抹（C）：迭代上界是要抹的头数；不跨轮携带状态；提前出口是读失败与不是那个头两种（这一轮不删）。
     for to_erase in &judgement.heads_to_erase {
         let offset = to_erase.placement.slot.to_device_offset();
-        let reread_head =
+        let Some(reread_bytes) =
             PoolReader::read(&*devices, to_erase.placement.device, offset, head_bytes)
-                .as_deref()
-                .and_then(unit_head_read_by_sweep);
-        let still_the_unpublished_unit_judged = reread_head.is_some_and(|head| {
-            head == to_erase.head
-                && rows_by_instance
-                    .get(&head.writer_instance)
-                    .is_some_and(|row| head.is_unpublished_by_the_row(row))
-        });
+        else {
+            return InstanceTableSweepOutcome::NoRowDeleted(
+                NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit {
+                    placement: to_erase.placement,
+                    reread: HeadRereadBeforeErasing::ReadFailed,
+                },
+            );
+        };
+        let still_the_unpublished_unit_judged =
+            unit_head_read_by_sweep(&reread_bytes).is_some_and(|head| {
+                head == to_erase.head
+                    && rows_by_instance
+                        .get(&head.writer_instance)
+                        .is_some_and(|row| head.is_unpublished_by_the_row(row))
+            });
         if !still_the_unpublished_unit_judged {
-            continue;
+            return InstanceTableSweepOutcome::NoRowDeleted(
+                NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit {
+                    placement: to_erase.placement,
+                    reread: HeadRereadBeforeErasing::NotTheJudgedUnit,
+                },
+            );
         }
+    }
+    let mut heads_erased = Vec::new();
+    // 迭代上界是要抹的头数；跨轮携带的只有抹了的那几个；提前出口是块设备错。
+    for to_erase in &judgement.heads_to_erase {
+        let offset = to_erase.placement.slot.to_device_offset();
         let Some((_, device)) = devices
             .iter_mut()
             .find(|(identity, _)| *identity == to_erase.placement.device)
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index d5dbe5c..769b294 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -17,9 +17,10 @@ use crate::allocation_record_tree::{
     node_pointers_as_far_as_readable, AllocationRecordTreeGeometry,
 };
 use crate::allocator::{
-    oldest_txg_in_the_window_rereading_slots_unreadable_at_mount, AllocationRecord, DeviceFreeMap,
+    oldest_txg_in_the_ring_rereading_slots_bad_at_mount, AllocationRecord, DeviceFreeMap,
     Placement, PlacementOnDevice, PlacementRefusal, PoolAllocator, ReclaimedReuse,
-    RereadOfARootRingSlotUnreadableAtMount, RootRingOccupancy, RootRingOccupant, UnitAreaStart,
+    RereadOfARootRingSlotBadAtMount, RootRingOccupancy, RootRingOccupant,
+    TxgsARingSlotBadAtMountCanHold, UnitAreaStart,
 };
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
@@ -65,7 +66,7 @@ use crate::recovery::{
 };
 use crate::recovery::{every_root_ring_slot, read_root_ring_slot, RootRingSlotReading};
 use crate::root_record::{RootRecord, UnmountMarker};
-use crate::root_ring::{target_for_publish, txg_is_inside_the_one_revolution_window, RootRingSlot};
+use crate::root_ring::{target_for_publish, RootRingSlot};
 use crate::system_configuration::SystemImmutableSizes;
 use crate::transaction::{
     acquire_expected_instance, central_mapping_locations_in_the_version,
@@ -756,7 +757,7 @@ pub struct PublishSequenceFailed {
 }
 
 /// 回退目标不在回退候选集里的是哪一条（D23（journal 的角色与格式） 已定项 14「候选集」：根环里按现行那一版的实例表判仍然有效
-/// ∧ txg ≥ F_生效 ∧ txg 落在一圈窗口里（D16（发布语义） 已定项 1「回退候选集」）∧ 带文件的根）。[`roll_back_by_a_forward_publish`] 按成员的次序判，一条目标同时中几条时只报最先判到的那一条。
+/// ∧ txg ≥ F_生效（D16（发布语义） 已定项 1「回退候选集」；一圈窗口那一格用户 2026-10-04 定 RGall 拿掉）∧ 带文件的根）。[`roll_back_by_a_forward_publish`] 按成员的次序判，一条目标同时中几条时只报最先判到的那一条。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum RollbackCandidateExclusion {
     /// 根环里没有那个 (实例, txg) 的可读根槽：候选集是根环里的根。
@@ -765,10 +766,9 @@ pub enum RollbackCandidateExclusion {
     BelowEffectiveFloor,
     /// 现行那一版的实例表里有那个实例的行 (i, Ti, Wi) 且目标的 txg > Ti：被崩溃恢复抛弃的时间线上的根
     /// （「按现行那一版的实例表判仍然有效」反过来，`recovery::root_is_abandoned_by_the_instance_table`）。
+    /// 原来还有一格「txg 落在现行那一版的一圈窗口外」：回退候选集是原来同取一圈窗口的六处之一，用户 2026-10-04 定 RGall 一起退回整个环
+    /// （`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1），那一格拿掉——回收门槛同时退回整个环，窗口外的有效根引用的单元不再被回收。
     OnAbandonedTimeline,
-    /// 目标的 txg 落在现行那一版的一圈窗口外（D16（发布语义） 已定项 1「回退候选集」：txg 落在一圈窗口里；窗口 = txg > N − 3S，
-    /// N 取现行那一版的 txg）：环上有洞时那一槽里留着的上一圈的根，窗口已许可它引用的单元被复用（7-2 报告第四节，用户 2026-10-03 定照做）。
-    OutsideTheOneRevolutionWindow,
     /// 目标那一版树表 0 条（还没发布过文件版本）：候选集只收带文件的根。
     VersionWithoutFile,
 }
@@ -1281,44 +1281,52 @@ fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(
     })
 }
 
-/// 进程内重读一个挂载那一刻读不出的根环槽（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定）：只读一次，不重试；读得出、自证过、
-/// 按 `table` 仍然有效 ⇒ 照它的 txg；读得出、自证过而按 `table` 被抛弃 ⇒ 当没有根；设备报读错或读得出而自证不过 ⇒ 同一个处置，
-/// 按根环表记的最大 txg 算（用户 2026-10-04 定改法 A，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1：
-/// 这一刻读到「自证不过」既可能是一次读坏的有效根，也可能是从没写过的槽，当没有根在前一种上让回收门槛越过盘上一条有效根）。
+/// 进程内重读一个挂载那一刻读坏的根环槽（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定）：只读一次，不重试；读得出、自证过、
+/// 按 `table` 仍然有效 ⇒ 照它的 txg；读得出、自证过而按 `table` 被抛弃 ⇒ 当没有根；读回全零 ⇒ 从没写过，确认为空、当没有根
+/// （用户 2026-10-04 定：盘上全零的根槽另算「从没写过」，重读读出有效根或确认为空之前回收门槛不越过它）；设备报读错或读得出、不是全零而自证不过
+/// ⇒ 照旧钉在它可能住过的最旧 txg（用户 2026-10-04 定改法 A，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1：
+/// 这一刻读到「自证不过」可能是一次读坏的有效根，当没有根在这上面让回收门槛越过盘上一条有效根）。
 #[must_use]
-pub fn reread_of_a_root_ring_slot_unreadable_at_mount<Reader: PoolReader + ?Sized>(
+pub fn reread_of_a_root_ring_slot_bad_at_mount<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     parameters: &MakeFilesystemParameters,
     table: &InstanceTableRecords,
     ring_slot: RootRingSlot,
-) -> RereadOfARootRingSlotUnreadableAtMount {
-    match read_root_ring_slot(
+) -> RereadOfARootRingSlotBadAtMount {
+    match read_root_ring_slot_telling_never_written_apart(
         reader,
         &parameters.region_devices,
         &parameters.geometry,
         &parameters.filesystem_identifier,
         ring_slot,
     ) {
-        RootRingSlotReading::SelfVerified(root) if !abandoned_by_table(&root, table) => {
-            RereadOfARootRingSlotUnreadableAtMount::ValidRoot {
+        RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root)
+            if !abandoned_by_table(&root, table) =>
+        {
+            RereadOfARootRingSlotBadAtMount::ValidRoot {
                 checkpoint_txg: root.checkpoint_txg,
             }
         }
-        RootRingSlotReading::SelfVerified(_) => {
-            RereadOfARootRingSlotUnreadableAtMount::AbandonedByTheInstanceTable
+        RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(_) => {
+            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable
+        }
+        RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => {
+            RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten
+        }
+        RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => {
+            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
         }
-        RootRingSlotReading::Bad(
-            BadRootRingSlotReading::Unreadable | BadRootRingSlotReading::NotSelfVerified,
-        ) => RereadOfARootRingSlotUnreadableAtMount::StillUnreadableOrNotSelfVerified,
     }
 }
 
-/// 这个进程写了一条根：记进分配器（[`PoolAllocator::record_root_written_by_this_process`]），记之前照 E2 重读挂载那一刻读不出、
-/// 记了这条根之后会是窗口里最小的那个槽（[`PoolAllocator::record_root_written_by_this_process_rereading_slots_unreadable_at_mount`]；
-/// D16（发布语义） 已定项 1，用户 2026-10-03 定）。判「按实例表仍然有效」用分配器根环表上记着的那一张
+/// 这个进程写了一条根：记进分配器（[`PoolAllocator::record_root_written_by_this_process`]），记之前照 E2 重读挂载那一刻读坏、
+/// 记了这条根之后会是环里最小的那个槽（[`PoolAllocator::record_root_written_by_this_process_rereading_slots_bad_at_mount`]；
+/// D16（发布语义） 已定项 1，用户 2026-10-03 定；读坏的槽钉在它可能住过的最旧 txg，用户 2026-10-04 定）。名字里的「读不出」是读坏的两类
+/// （读不出、不是全零而自证不过）合起来说的：`transaction` 里两处调用点按这个名字调，这一批不改那份文件。
+/// 判「按实例表仍然有效」用分配器根环表上记着的那一张
 /// （`RootRingOccupancy::instance_table_judging_the_rereads`）。产品路径上记根的每一处都走它：带单元的发布在取完落点之后
 /// （`transaction::publish_admitted`、`transaction::prepare_the_row_publish_on_a_version_without_file`），零单元发布在根落盘之后。
-/// 抬 F 那一串在分配器拷贝上的整串预演（`rehearse_the_publishes_raising_the_floor`）不重读：它按最大 txg 算、回收得只会更少，
+/// 抬 F 那一串在分配器拷贝上的整串预演（`rehearse_the_publishes_raising_the_floor`）不重读：它按钉着的最旧 txg 算、回收得只会更少，
 /// 预演过得去真发就过得去。
 pub fn record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount<
     Reader: PoolReader + ?Sized,
@@ -1332,17 +1340,18 @@ pub fn record_root_written_by_this_process_rereading_root_ring_slots_unreadable_
         || InstanceTableRecords { rows: Vec::new() },
         |occupancy| occupancy.instance_table_judging_the_rereads().clone(),
     );
-    allocator.record_root_written_by_this_process_rereading_slots_unreadable_at_mount(
+    allocator.record_root_written_by_this_process_rereading_slots_bad_at_mount(
         checkpoint_txg,
         &mut |ring_slot| {
-            reread_of_a_root_ring_slot_unreadable_at_mount(reader, parameters, &table, ring_slot)
+            reread_of_a_root_ring_slot_bad_at_mount(reader, parameters, &table, ring_slot)
         },
     );
 }
 
-/// 可再分配谓词的门槛（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」）：根环 R × S 槽（S 读自系统配置），第 0 代根被盖之前
-/// 窗口里最旧有效根恒 0、门槛就是 F_生效；盖掉之后由窗口里最旧的有效根接管。调用方交进来的 `oldest_valid_root` 已经按一圈窗口滤过
-/// （挂载时 `rebuilt_allocator`，进程内 `allocator::RootRingOccupancy`，抬 F 时 `raise_rollback_floor`）。
+/// 可再分配谓词的门槛（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」，用户 2026-10-04 定 RGall：
+/// 从一圈窗口退回整个环，`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）：根环 R × S 槽（S 读自系统配置），第 0 代根被盖之前
+/// 环里最旧有效根恒 0、门槛就是 F_生效；盖掉之后由环里最旧的有效根接管。调用方交进来的 `oldest_valid_root` 取的是环里全部有效根
+/// （挂载时 `rebuilt_allocator`，进程内 `allocator::RootRingOccupancy`，抬 F 时 `raise_rollback_floor`），不按一圈窗口滤。
 #[must_use]
 pub fn reclaim_floor(
     effective_floor: CheckpointTxg,
@@ -1418,7 +1427,7 @@ struct ShadowLedgerComputed {
 #[allow(
     clippy::ptr_arg,
     clippy::too_many_arguments,
-    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、判落在一圈窗口里的谓词、当前账、重读前的快照"
+    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、重读前的快照"
 )]
 fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
@@ -1427,7 +1436,6 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     roots: &[RootRecord],
     is_abandoned: &dyn Fn(&RootRecord) -> bool,
     floor: CheckpointTxg,
-    is_inside_the_window: &dyn Fn(CheckpointTxg) -> bool,
     current_records: &[AllocationRecord],
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<ShadowLedgerComputed, MountError> {
@@ -1436,12 +1444,12 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
         .filter(|record| !record.is_released)
         .map(|record| (record.device.0, record.slot.0))
         .collect();
-    // 候选 = 按实例表有效 ∧ txg ≥ F ∧ 落在一圈窗口里（D16（发布语义） 已定项 1：影子账的候选过滤是窗口五处同改之一）。
-    for root in roots.iter().filter(|root| {
-        !is_abandoned(root)
-            && root.checkpoint_txg >= floor
-            && is_inside_the_window(root.checkpoint_txg)
-    }) {
+    // 候选 = 按实例表有效 ∧ txg ≥ F（D16（发布语义） 已定项 1：影子账的候选过滤是原来同取一圈窗口的六处之一，用户 2026-10-04 定 RGall
+    // 一起退回整个环：窗口外的有效根照样是候选，它引用的槽豁免）。
+    for root in roots
+        .iter()
+        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)
+    {
         if let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) {
             referenced_by_candidates.extend(
                 placements
@@ -1668,77 +1676,111 @@ fn placements_referenced_by_root<Device: BlockDevice>(
     }
 }
 
-/// 挂载那一刻设备报读错（`BadRootRingSlotReading::Unreadable`）、落在一圈窗口里的一个根环槽：它在 (N − 3S, first_txg − 1] 里能住的
-/// 最小与最大 txg（区域内槽位只由 txg 定，D22（单元原子性怎么合成） 已定项 2）。
+/// 挂载重建分配器与进程内重读时读一个根环槽读到了什么：在 [`read_root_ring_slot`] 的分法之上，把读回全零的槽单分出来
+/// （用户 2026-10-04 定：盘上全零的根槽另算「从没写过」，不并进准入抬 F 的上限；它也就没有根可罩，回收门槛不钉它）。
+/// 别的读者（择根、算上限那一遍、清扫）照旧用 [`read_root_ring_slot`]：清扫把全零与读不出同待遇（D18（块里携带什么信息） 已定项 11）。
+#[allow(
+    clippy::large_enum_variant,
+    reason = "一次读一个槽、按值交回马上拆开，装箱只多一次分配"
+)]
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-struct UnreadableRingSlotInsideTheWindow {
+enum RootRingSlotReadingTellingNeverWrittenApart {
+    SelfVerified(RootRecord),
+    /// 设备报读错，或读得出、不是全零而自证不过（校验和不过、fsid 不是本池的）。
+    BadAndNotAllZero,
+    /// 读得出、每个字节都是 0：从没写过。
+    AllZeroNeverWritten,
+}
+
+/// 读一个根环槽，只读一次、不重试：读不出、全零、自证过与否按 [`RootRingSlotReadingTellingNeverWrittenApart`] 分。偏移与盘的取法与
+/// [`read_root_ring_slot`] 相同（区域归属表给出盘、槽距给出偏移）；要看字节是不是全零，所以不经它读（经它再读一次，两次之间的瞬时读错
+/// 会让两个读数对不上）。
+fn read_root_ring_slot_telling_never_written_apart<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    region_devices: &[DeviceIdentity; 3],
+    immutable_sizes: &SystemImmutableSizes,
+    filesystem_identifier: &[u8; 16],
     ring_slot: RootRingSlot,
-    lowest_txg_it_can_hold: CheckpointTxg,
-    highest_txg_it_can_hold: CheckpointTxg,
+) -> RootRingSlotReadingTellingNeverWrittenApart {
+    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
+    let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
+    let offset =
+        crate::root_ring::slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
+    let Some(bytes) = reader.read(device, offset, root_slot_bytes) else {
+        return RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero;
+    };
+    if bytes.iter().all(|byte| *byte == 0) {
+        return RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten;
+    }
+    match RootRecord::parse_slot(&bytes, filesystem_identifier) {
+        Some(root) => RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root),
+        None => RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero,
+    }
 }
 
-/// 挂载时读不出的槽在窗口里只对应一两个 txg：门槛按最小的算，根环表记最大的（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」那一行，
-/// `research/prompts/m3-ring-r3-main-verification.md` 第二节第 4 行，零轮，只在攻方模型上量过）。区间取 (N − 3S, first_txg − 1]：
-/// 读不出的槽里可能住着比环里读得出的最新根还新的根（被藏住的最新根、由记录施加出来的那一版那次发布的根），上沿取新实例第一次发布之前的那个 txg。
-/// 只管设备报读错的槽；读得出而自证不过（没写过、内容坏）的槽照旧当没有根。
+/// 挂载那一刻读坏（设备报读错，或读得出、不是全零而自证不过）的根环槽各能住哪几个 txg（区域内槽位只由 txg 定，
+/// D22（单元原子性怎么合成） 已定项 2）：
+/// - 可能住过的最旧 txg：落到这个槽、小于 `first_txg` 的最小 txg。回收门槛钉在这里（用户 2026-10-04 定：读不出、自证不过的根槽在回收门槛里
+///   钉在它可能住过的最旧 txg，不再只在一圈窗口里取——窗口之外、崩在记录之后根槽之前留在洞里的旧根也可能就住在这一槽里）。
+/// - 能住的最大 txg：(N − 3S, first_txg − 1] 里落到这个槽的最大 txg（D16（发布语义） 已定项 1「根环表记最大的」，
+///   `research/prompts/m3-ring-r3-main-verification.md` 第二节第 4 行）：准入抬 F 的上限照 Q3 按它并。上沿取新实例第一次发布之前的那个 txg：
+///   读坏的槽里可能住着比环里读得出的最新根还新的根（被藏住的最新根、由记录施加出来的那一版那次发布的根）。
 ///
-/// 循环：只走区间头、区间尾两段，各 min(3S, first_txg − 窗口下沿) 个 txg（轮数至多 6S，与 first_txg 多大无关：外来记录的 txg 可以贴着
-/// `u64::MAX`），跨轮只往按槽的表里写最小、最大值；没有提前出口。
-fn unreadable_ring_slots_inside_the_window(
-    ring_slot_readings: &[(RootRingSlot, RootRingSlotReading)],
+/// 一个槽在 [0, first_txg) 里一个 txg 都住不了（年轻的环里还没轮到的槽）就不交回。
+///
+/// 循环：最大 txg 只走区间尾 min(3S, 区间长) 个 txg（与 first_txg 多大无关：外来记录的 txg 可以贴着 `u64::MAX`），最旧 txg 只走
+/// [0, min(3S, first_txg)) ——`target_for_publish` 只看 txg mod 3S，一个槽最小的 txg 落在第一个周期里、区间里最大的 txg 落在区间尾一个周期里；
+/// 轮数至多 6S，跨轮只往按槽的表里写；没有提前出口。
+fn txgs_ring_slots_bad_at_mount_can_hold(
+    bad_ring_slots: &BTreeSet<RootRingSlot>,
     window_anchor: CheckpointTxg,
     first_txg: CheckpointTxg,
     slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
-) -> Vec<UnreadableRingSlotInsideTheWindow> {
-    let unreadable_ring_slots: BTreeSet<RootRingSlot> = ring_slot_readings
-        .iter()
-        .filter_map(|(ring_slot, reading)| match reading {
-            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => Some(*ring_slot),
-            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified)
-            | RootRingSlotReading::SelfVerified(_) => None,
-        })
-        .collect();
+) -> Vec<(RootRingSlot, TxgsARingSlotBadAtMountCanHold)> {
     let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
     let lowest_txg_inside_the_window = window_anchor
         .0
         .checked_sub(ring_length_in_txgs)
         .map_or(0, |below_the_window| below_the_window + 1);
-    // target_for_publish 只看 txg mod 3S（区域 = txg mod R，槽 = (txg div R) mod S），所以一个槽在区间里的最小 txg 落在区间头
-    // min(3S, 区间长) 个 txg 里、最大 txg 落在区间尾同样多个 txg 里，两段之间的 txg 改不动任何一个槽的最小、最大值。
-    // 两段在区间短于 6S 时有重叠：先走区间头、再走区间尾，各自从小到大，第一次碰到的是最小值、最后一次碰到的是最大值。
     let interval_length_in_txgs = first_txg.0.saturating_sub(lowest_txg_inside_the_window);
-    let txgs_walked_at_each_end_of_the_interval = interval_length_in_txgs.min(ring_length_in_txgs);
-    let head_of_the_interval = lowest_txg_inside_the_window
-        ..lowest_txg_inside_the_window + txgs_walked_at_each_end_of_the_interval;
-    let tail_of_the_interval = first_txg.0 - txgs_walked_at_each_end_of_the_interval..first_txg.0;
-    let mut txgs_of_each_unreadable_slot: BTreeMap<RootRingSlot, (CheckpointTxg, CheckpointTxg)> =
-        BTreeMap::new();
-    for txg in head_of_the_interval.chain(tail_of_the_interval) {
+    let txgs_walked_at_the_tail_of_the_interval = interval_length_in_txgs.min(ring_length_in_txgs);
+    let mut highest_txg_of_each_bad_slot: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
+    // 从小到大走，最后一次碰到的是最大值。
+    for txg in first_txg.0 - txgs_walked_at_the_tail_of_the_interval..first_txg.0 {
         let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
-        if !unreadable_ring_slots.contains(&ring_slot) {
-            continue;
+        if bad_ring_slots.contains(&ring_slot) {
+            highest_txg_of_each_bad_slot.insert(ring_slot, CheckpointTxg(txg));
+        }
+    }
+    let mut oldest_txg_of_each_bad_slot: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
+    // 从小到大走，第一次碰到的是最小值。
+    for txg in 0..first_txg.0.min(ring_length_in_txgs) {
+        let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
+        if bad_ring_slots.contains(&ring_slot) {
+            oldest_txg_of_each_bad_slot
+                .entry(ring_slot)
+                .or_insert(CheckpointTxg(txg));
         }
-        txgs_of_each_unreadable_slot
-            .entry(ring_slot)
-            .and_modify(|(_, highest)| *highest = CheckpointTxg(txg))
-            .or_insert((CheckpointTxg(txg), CheckpointTxg(txg)));
     }
-    txgs_of_each_unreadable_slot
+    oldest_txg_of_each_bad_slot
         .into_iter()
-        .map(
-            |(ring_slot, (lowest_txg_it_can_hold, highest_txg_it_can_hold))| {
-                UnreadableRingSlotInsideTheWindow {
-                    ring_slot,
-                    lowest_txg_it_can_hold,
+        .map(|(ring_slot, oldest_txg_it_can_have_held)| {
+            let highest_txg_it_can_hold = *highest_txg_of_each_bad_slot.get(&ring_slot).expect(
+                "这个槽在 [0, first_txg) 里住得下一个 txg，区间 (N − 3S, first_txg − 1] 要么从 0 起、要么长过一圈，它在区间里也住得下一个",
+            );
+            (
+                ring_slot,
+                TxgsARingSlotBadAtMountCanHold::new(
+                    oldest_txg_it_can_have_held,
                     highest_txg_it_can_hold,
-                }
-            },
-        )
+                ),
+            )
+        })
         .collect()
 }
 
 /// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收（当场可用的只到每块盘上最新有效根那一截、某块盘一条有效根都读不出时
-/// 只到一圈窗口里最旧有效根，其余扣到这次挂载的暖机落满每块盘，甲），再把只被被抛弃根引用的槽隔离（影子账：被抛弃的根按最新根指着的
+/// 只到环里最旧有效根（RGall），其余扣到这次挂载的暖机落满每块盘，甲），再把只被被抛弃根引用的槽隔离（影子账：被抛弃的根按最新根指着的
 /// 实例表判，D23（journal 的角色与格式） 已定项 14 射程「影子账与按实例表判抛弃留着，理由是崩溃恢复」）。
 /// 影子账只住内存，所以每次挂载都要重算（步 4 / 步 5 代码三方第一轮云端攻方腿打中：普通重开一次隔离就归零）。
 /// 每块盘的空闲图从择到的系统配置按环长现算的单元区起点起（与偏移 417 那 8 字节相等，择的时候判过；C475（非默认环长下单元区起点取编译期常量））。
@@ -1796,13 +1838,13 @@ fn rebuilt_allocator<Device: BlockDevice>(
         .immutable
         .sizes
         .root_ring_slots_per_region;
-    let ring_slot_readings: Vec<(RootRingSlot, RootRingSlotReading)> =
+    let ring_slot_readings: Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)> =
         every_root_ring_slot(&system_configuration.immutable.sizes)
             .into_iter()
             .map(|ring_slot| {
                 (
                     ring_slot,
-                    read_root_ring_slot(
+                    read_root_ring_slot_telling_never_written_apart(
                         devices,
                         &system_configuration.immutable.region_devices,
                         &system_configuration.immutable.sizes,
@@ -1815,8 +1857,19 @@ fn rebuilt_allocator<Device: BlockDevice>(
     let roots_with_ring_slots: Vec<(RootRingSlot, RootRecord)> = ring_slot_readings
         .iter()
         .filter_map(|(ring_slot, reading)| match reading {
-            RootRingSlotReading::SelfVerified(root) => Some((*ring_slot, *root)),
-            RootRingSlotReading::Bad(_) => None,
+            RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) => {
+                Some((*ring_slot, *root))
+            }
+            RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
+            | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
+        })
+        .collect();
+    let bad_ring_slots_not_all_zero: BTreeSet<RootRingSlot> = ring_slot_readings
+        .iter()
+        .filter_map(|(ring_slot, reading)| match reading {
+            RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => Some(*ring_slot),
+            RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(_)
+            | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
         })
         .collect();
     let roots: Vec<RootRecord> = roots_with_ring_slots
@@ -1824,53 +1877,50 @@ fn rebuilt_allocator<Device: BlockDevice>(
         .map(|(_, root)| *root)
         .collect();
     let is_abandoned = |root: &RootRecord| abandoned_by_table(root, &newest_table);
-    // 一圈窗口（D16（发布语义） 已定项 1）：N = 环里读得出的最新有效根的 txg（挂载重建分配器时取它、不取 first_txg − 1）。
+    // 挂载时读坏的槽能住哪几个 txg（[`txgs_ring_slots_bad_at_mount_can_hold`]）：N = 环里读得出的最新有效根的 txg（D16（发布语义） 已定项 1
+    // 「挂载时读不出的槽」那一句：挂载重建分配器时取它、不取 first_txg − 1），读回全零的槽从没写过、不在其内（用户 2026-10-04 定）。
     let window_anchor = roots
         .iter()
         .filter(|root| !is_abandoned(root))
         .map(|root| root.checkpoint_txg)
         .max();
-    let is_inside_the_window = |checkpoint_txg: CheckpointTxg| {
-        window_anchor.is_none_or(|anchor| {
-            txg_is_inside_the_one_revolution_window(checkpoint_txg, anchor, slots_per_region)
-        })
-    };
-    let unreadable_ring_slots_inside_the_window = window_anchor.map_or_else(Vec::new, |anchor| {
-        unreadable_ring_slots_inside_the_window(
-            &ring_slot_readings,
+    let ring_slots_bad_at_mount = window_anchor.map_or_else(Vec::new, |anchor| {
+        txgs_ring_slots_bad_at_mount_can_hold(
+            &bad_ring_slots_not_all_zero,
             anchor,
             first_txg,
             slots_per_region,
         )
     });
-    // 窗口里有效根的最小 txg；挂载时读不出的槽按它在窗口里能住的最小 txg 算（门槛只会更低）。
-    let oldest_valid_root_in_the_window = roots
+    // 环里有效根的最小 txg（D16（发布语义） 已定项 1「环里最旧有效根」，用户 2026-10-04 定 RGall：不按一圈窗口滤）；
+    // 挂载时读坏的槽钉在它可能住过的最旧 txg（用户 2026-10-04 定）。
+    let oldest_valid_root_in_the_ring = roots
         .iter()
-        .filter(|root| !is_abandoned(root) && is_inside_the_window(root.checkpoint_txg))
+        .filter(|root| !is_abandoned(root))
         .map(|root| root.checkpoint_txg)
         .chain(
-            unreadable_ring_slots_inside_the_window
+            ring_slots_bad_at_mount
                 .iter()
-                .map(|unreadable| unreadable.lowest_txg_it_can_hold),
+                .map(|(_, txgs_it_can_hold)| txgs_it_can_hold.oldest_txg_it_can_have_held()),
         )
         .min();
     // 卸载半途崩溃之后的下一次可写挂载只放开一截（D16（发布语义） 已定项 1「正常卸载」那一行，用户 2026-10-04 选甲，
     // `research/prompts/m3-batch14-code-r1-forks.md` 第 1 行）：卸载那一串先写了系统配置里的新 F、带新 F 的根没落满每块盘就崩时，
     // F_生效 高于某块盘上最新的有效根；恢复择根不看 F，丢掉另一块盘之后择的就是那块盘上最新的根，它引用的单元释放代可以高于它、不高于 F_生效
     // （判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「形 R」那一行）。于是当场可用的只到
-    // reclaim_floor(min(F_生效, m), 一圈窗口里最旧有效根)，m 是每块盘上最新有效根的 txg 取最小（被抛弃的根不算，与卸载那一格同一处定义
+    // reclaim_floor(min(F_生效, m), 环里最旧有效根)（RGall），m 是每块盘上最新有效根的 txg 取最小（被抛弃的根不算，与卸载那一格同一处定义
     // [`newest_valid_root_txg_on_every_device`]）；其余到原门槛为止的那一截扣住，这次挂载的暖机在每块盘上都落了根之后放开（`establish_instance`）。
     // 某块盘一条有效根都读不出（m 取不到）时当场只放到一圈窗口里最旧有效根（用户 2026-10-04 选甲，`research/prompts/m3-batch16-code-r1-forks.md`
-    // 第 1 行；判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「再挂期间某块盘根环整段读坏」那一行）：那块盘上真正最新的根
-    // 这一遍看不到，它引用的单元释放代可以高到 F_生效；窗口里最旧有效根与回收门槛同一个读数（挂载时读不出的槽按它在窗口里能住的最小 txg 并进）。
-    // 这一遍一条有效根都没读出、窗口里最旧有效根也取不到时，「只到窗口里最旧有效根」的那一截是空的：当场一个都不放，整截扣到暖机落满。
+    // 第 1 行；判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「再挂期间某块盘根环整段读坏」那一行），RGall 之后这个量是
+    // 环里最旧有效根：那块盘上真正最新的根这一遍看不到，它引用的单元释放代可以高到 F_生效；它与回收门槛同一个读数（挂载时读坏的槽钉在它可能住过的最旧 txg 并进）。
+    // 这一遍一条有效根都没读出、环里最旧有效根也取不到时，「只到环里最旧有效根」的那一截是空的：当场一个都不放，整截扣到暖机落满。
     let device_of_txg = |txg: CheckpointTxg| {
         let target = target_for_publish(txg, slots_per_region);
         system_configuration.immutable.region_devices
             [usize::try_from(target.region).expect("区域号")]
     };
     let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
-    let floor_of_the_reclaim = reclaim_floor(effective_floor, oldest_valid_root_in_the_window);
+    let floor_of_the_reclaim = reclaim_floor(effective_floor, oldest_valid_root_in_the_ring);
     let floor_released_immediately = match newest_valid_root_txg_on_every_device(
         &roots,
         &newest_table,
@@ -1879,9 +1929,9 @@ fn rebuilt_allocator<Device: BlockDevice>(
     ) {
         Some(newest_valid_root_on_every_device) => Some(reclaim_floor(
             effective_floor.min(newest_valid_root_on_every_device),
-            oldest_valid_root_in_the_window,
+            oldest_valid_root_in_the_ring,
         )),
-        None => oldest_valid_root_in_the_window,
+        None => oldest_valid_root_in_the_ring,
     };
     if let Some(released_immediately_up_to) = floor_released_immediately {
         allocator.reclaim_released_up_to(released_immediately_up_to, ReclaimedReuse::Immediately);
@@ -1898,7 +1948,6 @@ fn rebuilt_allocator<Device: BlockDevice>(
             &roots,
             &is_abandoned,
             effective_floor,
-            &is_inside_the_window,
             &current_records,
             before_the_one_reread,
         )?,
@@ -1911,17 +1960,13 @@ fn rebuilt_allocator<Device: BlockDevice>(
         abandoned_roots_unreadable,
         placements_referenced_by_abandoned_root,
     } = shadow_ledger_computed;
-    // 挂载时读不出的槽在根环表里按它在窗口里能住的最大 txg 记成一条有效根（它在进程内的窗口里待到这个进程写到那个槽为止）；
-    // 同一个槽这一遍读出了根的，读出来的那条在后面、盖掉它（这一遍读的是同一个槽，两者不会同时成立，留着次序只为不依赖这一点）。
-    let occupants: Vec<(RootRingSlot, RootRingOccupant)> = unreadable_ring_slots_inside_the_window
+    // 挂载时读坏的槽在根环表里记成「挂载时读坏」，带它能住的 txg：回收门槛钉在它可能住过的最旧 txg、准入抬 F 的上限按它能住的最大 txg 并，
+    // 直到这个进程写到那个槽、或进程内重读读出有效根或确认为空（用户 2026-10-04 定）。同一个槽这一遍读出了根的，读出来的那条在后面、盖掉它
+    // （这一遍读的是同一个槽，两样不会同时成立，留着次序只为不依赖这一点）。
+    let occupants: Vec<(RootRingSlot, RootRingOccupant)> = ring_slots_bad_at_mount
         .iter()
-        .map(|unreadable| {
-            (
-                unreadable.ring_slot,
-                RootRingOccupant::UnreadableAtMountInsideTheWindow {
-                    highest_txg_it_can_hold: unreadable.highest_txg_it_can_hold,
-                },
-            )
+        .map(|(ring_slot, txgs_it_can_hold)| {
+            (*ring_slot, RootRingOccupant::BadAtMount(*txgs_it_can_hold))
         })
         .chain(roots_with_ring_slots.iter().map(|(ring_slot, root)| {
             let occupant = if is_abandoned(root) {
@@ -2163,17 +2208,16 @@ struct RootsOfTheRingForTheCeiling {
 }
 
 /// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。
-/// 有效 = 自证合法 ∧ 按传进来的实例表不被抛弃 ∧ txg ≥ 今天的 F ∧ 落在一圈窗口里；非空 = 树表里 inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排、
+/// 有效 = 自证合法 ∧ 按传进来的实例表不被抛弃 ∧ txg ≥ 今天的 F；非空 = 树表里 inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排、
 /// 比它小的有效根里最大的那条）的不同（`user_visible_trees_changed`，2026-09-17 用户定案）。
-/// 上限与「非空」只在一圈窗口里的有效根上取（B2，用户 2026-10-03 定，D16 已定项 1 同取这个窗口的第六处；
-/// `research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表二之后那一段）：窗口的锚 N 取环里读得出的、按传进来的实例表有效的
-/// 最新那条根（挂着时就是这个进程记到的最新根），窗口 = txg > N − 3S；窗口外的旧根不当有效根、不读它的树表——窗口已许可它引用的单元被复用，
-/// 读它的树表会读到别的内容或读不出（攻方 K2-b2：没有读故障的合法历史上准入抬 F 被拒）。
+/// 上限与「非空」在环里全部有效根上取：原来同取一圈窗口的六处之一（B2，用户 2026-10-03 定），用户 2026-10-04 定 RGall 一起退回整个环
+/// （`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）。回收门槛同时退回整个环，窗口外的有效根引用的单元不再被回收，
+/// 读它的树表读得到它那一版（B2 当初挡的攻方 K2-b2 那一形——窗口许可复用之后读窗口外旧根的树表读到别的内容——随之没有了）。
 /// 根环怎么读见 [`roots_of_the_ring_for_the_ceiling`]：`ring_slots_known_to_hold_a_root` 取这个进程分配器上那张根环表
 /// （`allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）；分配器上没装根环表的（只有用例自己拼的分配器，
 /// 可写挂载与 `make_filesystem::allocator_after_make_filesystem` 都装）给空集，读坏的槽一律当没有根。
-/// 这一份不知道哪几个槽是挂载那一刻读不出的（给手里没有这个进程的根环表的调用方：实验装置、用例）；这个进程的准入抬 F 走
-/// [`rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount`]。
+/// 这一份不知道哪几个槽是挂载那一刻读坏的（给手里没有这个进程的根环表的调用方：实验装置、用例）；这个进程的准入抬 F 走
+/// [`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]。
 ///
 /// # Errors
 /// 一条有效根一条都没有 ⇒ `RecoveryFailure::NoValidRoot`；要比的一条有效根的树表读不出或解不开 ⇒
@@ -2190,7 +2234,7 @@ pub fn rollback_floor_ceiling<Device: BlockDevice>(
     table: &InstanceTableRecords,
     ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
 ) -> Result<CheckpointTxg, MountError> {
-    rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount(
+    rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
         devices,
         system_configuration,
         current_floor,
@@ -2200,13 +2244,14 @@ pub fn rollback_floor_ceiling<Device: BlockDevice>(
     )
 }
 
-/// 同 [`rollback_floor_ceiling`]，再把窗口里挂载那一刻读不出、之后这个进程没写过的槽当作可能是一条非空有效根，F 不越过它
-/// （D16（发布语义） 已定项 1「抬 F 的上限」与「根槽这一次读坏」两行，用户 2026-10-04 定，
-/// `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS3；与「一圈窗口里最旧有效根」那一行把这些槽并进回收门槛同一个思路）：
-/// `ring_slots_unreadable_since_the_mount` 是这个进程根环表里还记着的那几个槽、各按它能住的最大 txg
-/// （`allocator::RootRingOccupancy::ring_slots_unreadable_at_mount`，[`ring_slots_unreadable_at_mount_by`]）；这一遍读那一槽又读坏
-/// （读不出或自证不过）、那个 txg 不低于今天的 F 且落在一圈窗口里，上限就取不高于它。这一遍读得出的照读出来的根算，不另封顶。
-/// 判「非空」、找「前一条」照旧只在读得出的根上取（读不出的那一条判不了空不空，F 不越过它就罩住了它是第 4 新的那一种）。
+/// 同 [`rollback_floor_ceiling`]，再把挂载那一刻读不出或读得出而自证不过、之后这个进程没写过的槽当作可能是一条非空有效根，F 不越过它
+/// （D16（发布语义） 已定项 1「抬 F 的上限」与「根槽这一次读坏」两行；RS3 用户 2026-10-04 定只并读不出的，
+/// `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节，同日 Q3 重定成两类都并，
+/// `research/prompts/m3-root-ring-code-r3-main-verification.md` 第六节）：`ring_slots_bad_since_the_mount` 是这个进程根环表里还记着的
+/// 那几个槽（[`ring_slots_bad_at_mount_by`]）。两类都只认挂载那一刻在 (N − 3S, 新实例第一次发布的 txg − 1] 里能住一个 txg 的槽
+/// （N = 挂载时环里读得出的最新有效根，`rebuilt_allocator`），各按它在那里能住的最大 txg 并：读不出的那一类与回收门槛记的同一个数，
+/// 自证不过的那一类同一个算法（区域内槽位只由 txg 定）。这一遍读那一槽又读坏（读不出或自证不过）、那个 txg 不低于今天的 F，上限就取不高于它。
+/// 这一遍读得出的照读出来的根算，不另封顶。判「非空」、找「前一条」照旧只在读得出的根上取（读不出的那一条判不了空不空，F 不越过它就罩住了它是第 4 新的那一种）。
 ///
 /// # Errors
 /// 同 [`rollback_floor_ceiling`]。
@@ -2214,15 +2259,13 @@ pub fn rollback_floor_ceiling<Device: BlockDevice>(
     clippy::ptr_arg,
     reason = "user_visible_tree_root_pointers 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
 )]
-pub fn rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount<
-    Device: BlockDevice,
->(
+pub fn rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
     system_configuration: &crate::system_configuration::SystemConfiguration,
     current_floor: CheckpointTxg,
     table: &InstanceTableRecords,
     ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
-    ring_slots_unreadable_since_the_mount: &BTreeMap<RootRingSlot, CheckpointTxg>,
+    ring_slots_bad_since_the_mount: &BTreeMap<RootRingSlot, CheckpointTxg>,
 ) -> Result<CheckpointTxg, MountError> {
     let RootsOfTheRingForTheCeiling {
         roots: readable,
@@ -2232,26 +2275,11 @@ pub fn rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount<
         system_configuration,
         ring_slots_known_to_hold_a_root,
     )?;
-    let slots_per_region = system_configuration
-        .immutable
-        .sizes
-        .root_ring_slots_per_region;
-    let window_anchor = readable
-        .iter()
-        .filter(|root| !abandoned_by_table(root, table))
-        .map(|root| root.checkpoint_txg)
-        .max();
-    let is_inside_the_window = |checkpoint_txg: CheckpointTxg| {
-        window_anchor.is_some_and(|anchor| {
-            txg_is_inside_the_one_revolution_window(checkpoint_txg, anchor, slots_per_region)
-        })
-    };
     let valid: Vec<RootRecord> = readable
         .iter()
         .copied()
         .filter(|root| root.checkpoint_txg >= current_floor)
         .filter(|root| !abandoned_by_table(root, table))
-        .filter(|root| is_inside_the_window(root.checkpoint_txg))
         .collect();
     let mut newest_per_device: std::collections::BTreeMap<DeviceIdentity, CheckpointTxg> =
         std::collections::BTreeMap::new();
@@ -2290,7 +2318,7 @@ pub fn rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount<
             },
         ),
     };
-    // 「前一条」只在有效根里找：被抛弃时间线的根、F 之下的根与一圈窗口外的根不算（D16（发布语义） 已定项 1「非空」从盘上怎么认；B2）。
+    // 「前一条」只在有效根里找：被抛弃时间线的根与 F 之下的根不算（D16（发布语义） 已定项 1「非空」从盘上怎么认；RGall 之后窗口外的有效根照算）。
     let previous_candidates: &[RootRecord] = &valid;
     let mut non_empty: Vec<NonEmptyValidRoot> = Vec::new();
     for root in &valid {
@@ -2320,22 +2348,17 @@ pub fn rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount<
         valid.iter().map(|root| root.checkpoint_txg).min(),
     )
     .ok_or(RecoveryFailure::NoValidRoot)?;
-    let lowest_txg_a_ring_slot_unreadable_since_the_mount_can_hold =
+    let lowest_txg_a_ring_slot_bad_since_the_mount_can_hold =
         ring_slots_bad_on_this_read_taken_as_holding_no_root
             .iter()
-            .filter_map(|ring_slot| ring_slots_unreadable_since_the_mount.get(ring_slot))
+            .filter_map(|ring_slot| ring_slots_bad_since_the_mount.get(ring_slot))
             .copied()
             .filter(|highest_txg_it_can_hold| *highest_txg_it_can_hold >= current_floor)
-            .filter(|highest_txg_it_can_hold| is_inside_the_window(*highest_txg_it_can_hold))
             .min();
-    Ok(
-        match lowest_txg_a_ring_slot_unreadable_since_the_mount_can_hold {
-            Some(highest_txg_it_can_hold) => {
-                ceiling_from_the_roots_read.min(highest_txg_it_can_hold)
-            }
-            None => ceiling_from_the_roots_read,
-        },
-    )
+    Ok(match lowest_txg_a_ring_slot_bad_since_the_mount_can_hold {
+        Some(highest_txg_it_can_hold) => ceiling_from_the_roots_read.min(highest_txg_it_can_hold),
+        None => ceiling_from_the_roots_read,
+    })
 }
 
 /// 一条非空的有效根：它的 (txg, 实例代号) 与它树表里用户可见的两棵树的根指针（去重按后者比）。
@@ -2477,7 +2500,7 @@ enum ReuseOfTheSlotsReclaimedByTheSequence {
 /// - 这一遍读不出、自证不过的槽当没有根（与这一遍回收门槛同一个读数）：恢复同样择不到它；它日后又读得出的话那条根只会更新或更旧——更新的引用的已释放槽更少，
 ///   更旧的轮不到被择。读不出让最小值只会更小，当场放开的只会更少。
 /// - 有一块盘上一条有效根都没读出来：交回 `None`（不按 txg 0 去比，说清是哪一格），两格各自处置：
-///   卸载那一格这一串回收的槽整截扣到生效；重建那一格当场只放到一圈窗口里最旧有效根（挂载时读不出的槽按它在窗口里能住的最小 txg 并进，
+///   卸载那一格这一串回收的槽整截扣到生效；重建那一格当场只放到环里最旧有效根（RGall；挂载时读坏的槽钉在它可能住过的最旧 txg 并进，
 ///   与回收门槛同一个读数），其余扣到这次挂载的暖机落满每块盘（`rebuilt_allocator`，用户 2026-10-04 选甲，
 ///   `research/prompts/m3-batch16-code-r1-forks.md` 第 1 行）。
 fn newest_valid_root_txg_on_every_device(
@@ -2617,17 +2640,21 @@ pub fn unmount<Device: BlockDevice>(
     ))
 }
 
-/// 这个进程分配器上那张根环表里挂载那一刻读不出、之后没写过的槽，各按它能住的最大 txg
-/// （`allocator::RootRingOccupancy::ring_slots_unreadable_at_mount`）；没装根环表的分配器给空表。准入抬 F 的上限按它封顶
-/// （[`rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount`]）。
+/// 这个进程分配器上那张根环表里挂载那一刻读坏（读不出，或读得出、不是全零而自证不过）、之后没写过也没重读出有效根或确认为空的槽，
+/// 各按它能住的最大 txg（`allocator::RootRingOccupancy::ring_slots_bad_at_mount`，Q3 两类都并；读回全零的槽从没写过、不在其内，
+/// 用户 2026-10-04 定）；没装根环表的分配器给空表。准入抬 F 的上限按它封顶（[`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]）。
 #[must_use]
-pub fn ring_slots_unreadable_at_mount_by(
+pub fn ring_slots_bad_at_mount_by(
     allocator: &PoolAllocator,
 ) -> BTreeMap<RootRingSlot, CheckpointTxg> {
-    allocator.root_ring_occupancy().map_or_else(
-        BTreeMap::new,
-        RootRingOccupancy::ring_slots_unreadable_at_mount,
-    )
+    allocator
+        .root_ring_occupancy()
+        .map_or_else(BTreeMap::new, RootRingOccupancy::ring_slots_bad_at_mount)
+        .into_iter()
+        .map(|(ring_slot, txgs_it_can_hold)| {
+            (ring_slot, txgs_it_can_hold.highest_txg_it_can_hold())
+        })
+        .collect()
 }
 
 /// 这个进程分配器上那张根环表里知道住着一条根的槽（`allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）；
@@ -2697,13 +2724,13 @@ fn raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk<Device
     let current_instance_table = instance_table_chain_of_root(&*devices, &current.root)
         .map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?
         .records;
-    let ceiling = rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount(
+    let ceiling = rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
         devices,
         &caller_inputs_agreeing_with_the_disk.system_configuration,
         current.root.rollback_floor,
         &current_instance_table,
         &ring_slots_known_to_hold_a_root_by(allocator),
-        &ring_slots_unreadable_at_mount_by(allocator),
+        &ring_slots_bad_at_mount_by(allocator),
     )?;
     if ceiling <= current.root.rollback_floor {
         return Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling {
@@ -2867,17 +2894,16 @@ pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
     }
 }
 
-/// 抬 F 那一遍的回收门槛里「一圈窗口里最旧有效根」（D16（发布语义） 已定项 1）：这一遍读出来的根里按 `table` 有效的，并上分配器根环表里
-/// 挂载那一刻读不出、这一遍也没在那一槽读出根的槽（各按它能住的最大 txg；RR4，`research/prompts/m3-root-ring-code-r1-main-verification.md`
-/// 第三节表一，攻方 B1），在锚 `window_anchor` 的窗口里取最小；最小的是一个读不出的槽时照 E2 先重读那一槽
-/// （[`oldest_txg_in_the_window_rereading_slots_unreadable_at_mount`]，与进程内记根同一取法）。只读盘，不改分配器。
-fn oldest_valid_root_in_the_window_for_raising_the_floor<Reader: PoolReader + ?Sized>(
+/// 抬 F 那一遍的回收门槛里「环里最旧有效根」（D16（发布语义） 已定项 1，用户 2026-10-04 定 RGall：不按一圈窗口滤）：这一遍读出来的根里
+/// 按 `table` 有效的，并上分配器根环表里挂载那一刻读坏、这一遍也没在那一槽读出根的槽（各钉在它可能住过的最旧 txg，用户 2026-10-04 定；
+/// 并进来是 RR4，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表一，攻方 B1），取最小；最小的是一个读坏的槽时照 E2
+/// 先重读那一槽（[`oldest_txg_in_the_ring_rereading_slots_bad_at_mount`]，与进程内记根同一取法）。只读盘，不改分配器。
+fn oldest_valid_root_in_the_ring_for_raising_the_floor<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     parameters_of_the_pool: &MakeFilesystemParameters,
     allocator: &PoolAllocator,
     roots_read_from_the_ring: &[RootRecord],
     table: &InstanceTableRecords,
-    window_anchor: CheckpointTxg,
 ) -> Option<CheckpointTxg> {
     let slots_per_region = parameters_of_the_pool.geometry.root_ring_slots_per_region;
     let valid_root_txgs: Vec<CheckpointTxg> = roots_read_from_the_ring
@@ -2889,22 +2915,20 @@ fn oldest_valid_root_in_the_window_for_raising_the_floor<Reader: PoolReader + ?S
         .iter()
         .map(|root| target_for_publish(root.checkpoint_txg, slots_per_region))
         .collect();
-    let unreadable_at_mount: BTreeMap<RootRingSlot, CheckpointTxg> = allocator
+    let bad_at_mount: BTreeMap<RootRingSlot, CheckpointTxg> = allocator
         .root_ring_occupancy()
         .map_or_else(
             BTreeMap::new,
-            RootRingOccupancy::ring_slots_unreadable_at_mount,
+            RootRingOccupancy::ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held,
         )
         .into_iter()
         .filter(|(ring_slot, _)| !ring_slots_read_from_the_ring.contains(ring_slot))
         .collect();
-    oldest_txg_in_the_window_rereading_slots_unreadable_at_mount(
+    oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
         &valid_root_txgs,
-        unreadable_at_mount,
-        window_anchor,
-        slots_per_region,
+        bad_at_mount,
         &mut |ring_slot| {
-            reread_of_a_root_ring_slot_unreadable_at_mount(
+            reread_of_a_root_ring_slot_bad_at_mount(
                 reader,
                 parameters_of_the_pool,
                 table,
@@ -2978,13 +3002,13 @@ fn raise_the_floor_through<Device: BlockDevice>(
     }
     let ceiling_judged_by_the_admission = match entry {
         RaiseFloorEntry::Admission => {
-            let ceiling = rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount(
+            let ceiling = rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
                 devices,
                 &system_configuration,
                 current.root.rollback_floor,
                 &table,
                 &ring_slots_known_to_hold_a_root_by(allocator),
-                &ring_slots_unreadable_at_mount_by(allocator),
+                &ring_slots_bad_at_mount_by(allocator),
             )?;
             if new_floor > ceiling {
                 return Err(MountError::RollbackFloorAboveCeiling {
@@ -3030,24 +3054,14 @@ fn raise_the_floor_through<Device: BlockDevice>(
     .map_err(|still_unreadable| {
         MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
     })?;
-    // 一圈窗口（D16（发布语义） 已定项 1）：进程内的锚是这个进程记到的最新根，抬 F 时就是现行那一版的根。
-    let slots_per_region = system_configuration
-        .immutable
-        .sizes
-        .root_ring_slots_per_region;
-    let window_anchor = current.root.checkpoint_txg;
-    let is_inside_the_window = |checkpoint_txg: CheckpointTxg| {
-        txg_is_inside_the_one_revolution_window(checkpoint_txg, window_anchor, slots_per_region)
-    };
-    // 根环表里挂载那一刻读不出、这一遍也没读出根的槽，按它能住的最大 txg 并进窗口里最旧有效根（RR4，攻方 B1；与进程内记根同一取法，
-    // E2 的重读照用：它成了最小的那个时先重读，`oldest_valid_root_in_the_window_for_raising_the_floor`）。
-    let oldest_valid_root_in_the_window = oldest_valid_root_in_the_window_for_raising_the_floor(
+    // 根环表里挂载那一刻读坏、这一遍也没读出根的槽，钉在它可能住过的最旧 txg 并进环里最旧有效根（RR4，攻方 B1；用户 2026-10-04 定钉在最旧 txg；与进程内记根同一取法，
+    // E2 的重读照用：它成了最小的那个时先重读，`oldest_valid_root_in_the_ring_for_raising_the_floor`）。
+    let oldest_valid_root_in_the_ring = oldest_valid_root_in_the_ring_for_raising_the_floor(
         &**devices,
         &parameters_of_the_pool,
         allocator,
         &roots,
         &table,
-        window_anchor,
     );
     // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
     // 不然那个槽回收之后就发得出去（步 4 / 步 5 代码三方第二轮辩方腿：窄读法要按每次挂载与每次抬 F 的候选集现算）。
@@ -3059,7 +3073,6 @@ fn raise_the_floor_through<Device: BlockDevice>(
             &roots,
             &|root| abandoned_by_table(root, &table),
             new_floor,
-            &is_inside_the_window,
             &current.allocation_records,
             &mut BeforeTheOneReread::RereadImmediately,
         ) {
@@ -3074,7 +3087,7 @@ fn raise_the_floor_through<Device: BlockDevice>(
     };
     // 卸载那一格当场放开的那一截（`RaiseFloorEntry::reuse_of_the_reclaimed_slots`，GA）：下面的整串预演就在分配器的拷贝上算上它们；
     // 其余那一截与准入那一格一样扣到生效。分配器记着回收过的槽不重复回收，第二次只回收第一次没回收的那几个。
-    let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_window);
+    let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_ring);
     let reclaimed = match entry.reuse_of_the_reclaimed_slots() {
         ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => allocator
             .reclaim_released_up_to(
@@ -5869,7 +5882,7 @@ fn user_visible_units_through_the_mapping(
 }
 
 /// 回退候选集那一判（D23（journal 的角色与格式） 已定项 14「候选集」）：根环里有它 ∧ txg ≥ F_生效 ∧ 按现行那一版的实例表判仍然有效
-/// ∧ txg 落在一圈窗口里（锚 `window_anchor` = 现行那一版的 txg，D16（发布语义） 已定项 1「回退候选集」）∧ 带文件。
+/// ∧ 带文件（D16（发布语义） 已定项 1「回退候选集」；原来另有「txg 落在一圈窗口里」一格，用户 2026-10-04 定 RGall 退回整个环）。
 /// 按 [`RollbackCandidateExclusion`] 成员的次序判。交回根环里读出来的那条根。
 /// 「根环里有它」与 F_生效 两处读根环都照抬 F 同一个读法（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，实审 A3b 报告 Q7）：
 /// `ring_slots_known_to_hold_a_root`（这个进程分配器上那张根环表，[`ring_slots_known_to_hold_a_root_by`]）里没有的槽读坏当没有根；
@@ -5884,7 +5897,6 @@ fn rollback_candidate<Reader: PoolReader>(
     system_configuration: &crate::system_configuration::SystemConfiguration,
     current_table: &InstanceTableRecords,
     ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
-    window_anchor: CheckpointTxg,
     target: RollbackTarget,
 ) -> Result<RootRecord, RollbackError> {
     let not_a_candidate = |exclusion| RollbackError::TargetNotACandidate { target, exclusion };
@@ -5921,18 +5933,6 @@ fn rollback_candidate<Reader: PoolReader>(
             RollbackCandidateExclusion::OnAbandonedTimeline,
         ));
     }
-    if !txg_is_inside_the_one_revolution_window(
-        target.checkpoint_txg,
-        window_anchor,
-        system_configuration
-            .immutable
-            .sizes
-            .root_ring_slots_per_region,
-    ) {
-        return Err(not_a_candidate(
-            RollbackCandidateExclusion::OutsideTheOneRevolutionWindow,
-        ));
-    }
     match tree_table_has_no_entries(reader, &target_root) {
         Ok(true) => Err(not_a_candidate(
             RollbackCandidateExclusion::VersionWithoutFile,
@@ -6285,7 +6285,6 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         &system_configuration,
         &current_table,
         &ring_slots_known_to_hold_a_root_by(allocator),
-        current.root.checkpoint_txg,
         target,
     )?;
     let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
@@ -6607,19 +6606,17 @@ mod reclaim_floor_tests {
 }
 
 #[cfg(test)]
-mod unreadable_ring_slots_inside_the_window_tests {
-    use super::{unreadable_ring_slots_inside_the_window, UnreadableRingSlotInsideTheWindow};
+mod txgs_ring_slots_bad_at_mount_can_hold_tests {
+    use super::txgs_ring_slots_bad_at_mount_can_hold;
     use crate::address::CheckpointTxg;
-    use crate::recovery::{BadRootRingSlotReading, RootRingSlotReading};
+    use crate::allocator::TxgsARingSlotBadAtMountCanHold;
     use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
     use singlefs_format::ROOT_RING_REGIONS;
     use std::collections::{BTreeMap, BTreeSet};
 
-    /// 环里每个槽都读过一次：两个槽读得出而自证不过，其余的设备报读错。
-    fn ring_slot_readings_with_two_slots_not_self_verified(
-        slots_per_region: RootRingSlotsPerRegion,
-    ) -> Vec<(RootRingSlot, RootRingSlotReading)> {
-        let not_self_verified_ring_slots = [
+    /// 两个槽之外环里每个槽都读坏（两个槽读出了根）。
+    fn bad_ring_slots_but_two(slots_per_region: RootRingSlotsPerRegion) -> BTreeSet<RootRingSlot> {
+        let ring_slots_read = [
             RootRingSlot { region: 0, slot: 0 },
             RootRingSlot { region: 1, slot: 2 },
         ];
@@ -6627,86 +6624,112 @@ mod unreadable_ring_slots_inside_the_window_tests {
             .flat_map(|region| {
                 (0..slots_per_region.count()).map(move |slot| RootRingSlot { region, slot })
             })
-            .map(|ring_slot| {
-                let bad_reading = if not_self_verified_ring_slots.contains(&ring_slot) {
-                    BadRootRingSlotReading::NotSelfVerified
-                } else {
-                    BadRootRingSlotReading::Unreadable
-                };
-                (ring_slot, RootRingSlotReading::Bad(bad_reading))
-            })
+            .filter(|ring_slot| !ring_slots_read.contains(ring_slot))
             .collect()
     }
 
-    /// 参照：区间 (N − 3S, first_txg − 1] 里的 txg 一个一个走（改成只走两段之前的写法），只给区间长度装得下的组用。
-    fn walk_every_txg_of_the_interval(
-        ring_slot_readings: &[(RootRingSlot, RootRingSlotReading)],
+    /// 参照：[0, first_txg) 与区间 (N − 3S, first_txg − 1] 里的 txg 一个一个走（只给区间装得下的组用）。
+    fn walk_every_txg_below_the_first(
+        bad_ring_slots: &BTreeSet<RootRingSlot>,
         window_anchor: CheckpointTxg,
         first_txg: CheckpointTxg,
         slots_per_region: RootRingSlotsPerRegion,
-    ) -> Vec<UnreadableRingSlotInsideTheWindow> {
-        let unreadable_ring_slots: BTreeSet<RootRingSlot> = ring_slot_readings
-            .iter()
-            .filter(|(_, reading)| {
-                *reading == RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable)
-            })
-            .map(|(ring_slot, _)| *ring_slot)
-            .collect();
+    ) -> Vec<(RootRingSlot, TxgsARingSlotBadAtMountCanHold)> {
         let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
         let lowest_txg_inside_the_window = window_anchor
             .0
             .checked_sub(ring_length_in_txgs)
             .map_or(0, |below_the_window| below_the_window + 1);
-        let mut txgs_of_each_unreadable_slot: BTreeMap<
-            RootRingSlot,
-            (CheckpointTxg, CheckpointTxg),
-        > = BTreeMap::new();
-        for txg in lowest_txg_inside_the_window..first_txg.0 {
+        let mut oldest_of_each: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
+        let mut highest_of_each: BTreeMap<RootRingSlot, CheckpointTxg> = BTreeMap::new();
+        for txg in 0..first_txg.0 {
             let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
-            if unreadable_ring_slots.contains(&ring_slot) {
-                txgs_of_each_unreadable_slot
-                    .entry(ring_slot)
-                    .and_modify(|(_, highest)| *highest = CheckpointTxg(txg))
-                    .or_insert((CheckpointTxg(txg), CheckpointTxg(txg)));
+            if !bad_ring_slots.contains(&ring_slot) {
+                continue;
+            }
+            oldest_of_each
+                .entry(ring_slot)
+                .or_insert(CheckpointTxg(txg));
+            if txg >= lowest_txg_inside_the_window {
+                highest_of_each.insert(ring_slot, CheckpointTxg(txg));
             }
         }
-        txgs_of_each_unreadable_slot
+        oldest_of_each
             .into_iter()
-            .map(
-                |(ring_slot, (lowest_txg_it_can_hold, highest_txg_it_can_hold))| {
-                    UnreadableRingSlotInsideTheWindow {
-                        ring_slot,
-                        lowest_txg_it_can_hold,
-                        highest_txg_it_can_hold,
-                    }
-                },
-            )
+            .map(|(ring_slot, oldest)| {
+                (
+                    ring_slot,
+                    TxgsARingSlotBadAtMountCanHold::new(
+                        oldest,
+                        *highest_of_each
+                            .get(&ring_slot)
+                            .expect("住得下就在区间里也住得下"),
+                    ),
+                )
+            })
             .collect()
     }
 
-    /// 区间短于 3S、正好 3S、两段重叠、长于 3S 若干圈与空区间几组逐项比参照；first_txg 是 `u64::MAX` 那一组（外来记录的 txg 是
+    /// 锚 40、first_txg 41（S = 4，区间 [29, 40] 正好一圈）：区域 2 槽 3 能住的最大 txg 是区间里余 11 的 35，
+    /// 可能住过的最旧 txg 是第一圈的 11——回收门槛钉在 11，不在窗口里的 35（用户 2026-10-04 定）。
+    #[test]
+    fn a_bad_slot_is_pinned_at_the_oldest_txg_it_can_have_held_below_the_window() {
+        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
+            .expect("4 是 S 区间 [4, 16] 的下界");
+        let txgs = txgs_ring_slots_bad_at_mount_can_hold(
+            &BTreeSet::from([RootRingSlot { region: 2, slot: 3 }]),
+            CheckpointTxg(40),
+            CheckpointTxg(41),
+            slots_per_region,
+        );
+        assert_eq!(
+            txgs,
+            vec![(
+                RootRingSlot { region: 2, slot: 3 },
+                TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(11), CheckpointTxg(35))
+            )]
+        );
+    }
+
+    /// 年轻的环（first_txg 5，S = 4）：txg 5 起才轮到的槽一个 txg 都住不了，不交回。
+    #[test]
+    fn a_bad_slot_the_young_ring_has_not_reached_holds_no_txg() {
+        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
+            .expect("4 是 S 区间 [4, 16] 的下界");
+        let not_reached = target_for_publish(CheckpointTxg(7), slots_per_region);
+        let reached = target_for_publish(CheckpointTxg(4), slots_per_region);
+        assert_eq!(
+            txgs_ring_slots_bad_at_mount_can_hold(
+                &BTreeSet::from([not_reached, reached]),
+                CheckpointTxg(4),
+                CheckpointTxg(5),
+                slots_per_region,
+            ),
+            vec![(
+                reached,
+                TxgsARingSlotBadAtMountCanHold::new(CheckpointTxg(4), CheckpointTxg(4))
+            )]
+        );
+    }
+
+    /// 区间短于 3S、正好 3S、长于 3S 若干圈、年轻的环与空区间几组逐项比参照；first_txg 是 `u64::MAX` 那一组（外来记录的 txg 是
     /// `u64::MAX − 1`）参照走不完，比的是按 3S 周期手算的期望。
     #[test]
-    fn walking_only_the_two_ends_of_the_interval_matches_walking_every_txg_of_it() {
+    fn walking_one_period_at_each_end_matches_walking_every_txg_below_the_first() {
         for declared_slots_per_region in [4, 5] {
             let slots_per_region =
                 RootRingSlotsPerRegion::from_system_configuration_field(declared_slots_per_region)
                     .expect("4 与 5 落在 S 的区间 [4, 16] 里");
             let ring_length_in_txgs = ROOT_RING_REGIONS * declared_slots_per_region;
-            let ring_slot_readings =
-                ring_slot_readings_with_two_slots_not_self_verified(slots_per_region);
+            let bad_ring_slots = bad_ring_slots_but_two(slots_per_region);
             let anchor_above_one_revolution = 40;
             let lowest_txg_inside_the_window =
                 anchor_above_one_revolution - ring_length_in_txgs + 1;
             let anchor_and_first_txg_of_each_group = [
-                (anchor_above_one_revolution, lowest_txg_inside_the_window),
+                (anchor_above_one_revolution, anchor_above_one_revolution + 1),
                 (
                     anchor_above_one_revolution,
-                    lowest_txg_inside_the_window + ring_length_in_txgs - 5,
-                ),
-                (
-                    anchor_above_one_revolution,
-                    lowest_txg_inside_the_window + ring_length_in_txgs,
+                    lowest_txg_inside_the_window + ring_length_in_txgs + 5,
                 ),
                 (
                     anchor_above_one_revolution,
@@ -6716,19 +6739,21 @@ mod unreadable_ring_slots_inside_the_window_tests {
                     anchor_above_one_revolution,
                     lowest_txg_inside_the_window + 4 * ring_length_in_txgs + 7,
                 ),
+                (5, 6),
                 (5, 9),
                 (5, 5 * ring_length_in_txgs + 3),
+                (0, 0),
             ];
             for (window_anchor, first_txg) in anchor_and_first_txg_of_each_group {
                 assert_eq!(
-                    unreadable_ring_slots_inside_the_window(
-                        &ring_slot_readings,
+                    txgs_ring_slots_bad_at_mount_can_hold(
+                        &bad_ring_slots,
                         CheckpointTxg(window_anchor),
                         CheckpointTxg(first_txg),
                         slots_per_region,
                     ),
-                    walk_every_txg_of_the_interval(
-                        &ring_slot_readings,
+                    walk_every_txg_below_the_first(
+                        &bad_ring_slots,
                         CheckpointTxg(window_anchor),
                         CheckpointTxg(first_txg),
                         slots_per_region,
@@ -6738,17 +6763,16 @@ mod unreadable_ring_slots_inside_the_window_tests {
             }
         }
 
-        // S = 4，3S = 12：txg 模 12 的余数 r 落在区域 r mod 3 的槽 r div 3（D22 已定项 2 的槽位公式在一个周期里展开）。
-        // 2^64 模 12 是 4，所以 u64::MAX 模 12 是 3：区间尾 u64::MAX − 12 ..= u64::MAX − 1 里余数是 r 的那个 txg 是 u64::MAX − k，
-        // k 取 1..=12 里 (3 − k) 模 12 等于 r 的那一个。窗口锚 40 的窗口下沿是 29（模 12 是 5）。
+        // S = 4，3S = 12：txg 模 12 的余数 r 落在区域 r mod 3 的槽 r div 3（D22 已定项 2 的槽位公式在一个周期里展开），
+        // 可能住过的最旧 txg 就是 r。2^64 模 12 是 4，所以 u64::MAX 模 12 是 3：区间尾 u64::MAX − 12 ..= u64::MAX − 1 里余数是 r 的那个
+        // txg 是 u64::MAX − k，k 取 1..=12 里 (3 − k) 模 12 等于 r 的那一个。
         let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
             .expect("4 是 S 区间 [4, 16] 的下界");
         let ring_length_in_txgs = 12;
         assert_eq!(u64::MAX % ring_length_in_txgs, 3);
-        let ring_slot_readings =
-            ring_slot_readings_with_two_slots_not_self_verified(slots_per_region);
-        let lowest_txg_inside_the_window = 29;
-        let mut expected: Vec<UnreadableRingSlotInsideTheWindow> = (0..ring_length_in_txgs)
+        let bad_ring_slots = bad_ring_slots_but_two(slots_per_region);
+        let mut expected: Vec<(RootRingSlot, TxgsARingSlotBadAtMountCanHold)> = (0
+            ..ring_length_in_txgs)
             .map(|residue| {
                 let distance_modulo_the_ring =
                     (3 + ring_length_in_txgs - residue) % ring_length_in_txgs;
@@ -6757,30 +6781,23 @@ mod unreadable_ring_slots_inside_the_window_tests {
                 } else {
                     distance_modulo_the_ring
                 };
-                UnreadableRingSlotInsideTheWindow {
-                    ring_slot: RootRingSlot {
+                (
+                    RootRingSlot {
                         region: residue % 3,
                         slot: residue / 3,
                     },
-                    lowest_txg_it_can_hold: CheckpointTxg(
-                        lowest_txg_inside_the_window
-                            + (residue + ring_length_in_txgs - 5) % ring_length_in_txgs,
+                    TxgsARingSlotBadAtMountCanHold::new(
+                        CheckpointTxg(residue),
+                        CheckpointTxg(u64::MAX - distance_below_u64_maximum),
                     ),
-                    highest_txg_it_can_hold: CheckpointTxg(u64::MAX - distance_below_u64_maximum),
-                }
-            })
-            .filter(|slot_inside_the_window| {
-                ![
-                    RootRingSlot { region: 0, slot: 0 },
-                    RootRingSlot { region: 1, slot: 2 },
-                ]
-                .contains(&slot_inside_the_window.ring_slot)
+                )
             })
+            .filter(|(ring_slot, _)| bad_ring_slots.contains(ring_slot))
             .collect();
-        expected.sort_by_key(|slot_inside_the_window| slot_inside_the_window.ring_slot);
+        expected.sort_by_key(|(ring_slot, _)| *ring_slot);
         assert_eq!(
-            unreadable_ring_slots_inside_the_window(
-                &ring_slot_readings,
+            txgs_ring_slots_bad_at_mount_can_hold(
+                &bad_ring_slots,
                 CheckpointTxg(40),
                 CheckpointTxg(u64::MAX),
                 slots_per_region,
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index 05035ac..70fa7e6 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -549,16 +549,6 @@ pub struct ModelRingPosition {
 const MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 =
     ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM;
 
-/// 一圈窗口（D16（发布语义） 已定项 1）：txg 落不落在锚 `window_anchor` 之下一圈里，txg > N − 3S（写成 txg + 3S > N，N < 3S 时整个环都在窗口里）。
-/// 模型自己算一份，不引实现的那一份（D13（验证路线） 已定项 5）。
-fn is_inside_the_one_revolution_window_of(
-    checkpoint_txg: ModelCheckpointTxg,
-    window_anchor: ModelCheckpointTxg,
-) -> bool {
-    checkpoint_txg.0 + ROOT_RING_REGIONS * MODEL_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM
-        > window_anchor.0
-}
-
 fn ring_position_of(checkpoint_txg: ModelCheckpointTxg) -> ModelRingPosition {
     ModelRingPosition {
         region: checkpoint_txg.0 % ROOT_RING_REGIONS,
@@ -796,9 +786,6 @@ pub enum ModelRefusalReason {
     RollbackTargetBelowEffectiveFloor,
     /// 回退目标在被抛弃的时间线上（D23（journal 的角色与格式） 已定项 14：有行 (i, Ti, Wi) 且 T > Ti）。
     RollbackTargetOnAbandonedTimeline,
-    /// 回退目标的 txg 落在现行那一版的一圈窗口外（D16（发布语义） 已定项 1「回退候选集」：txg 落在一圈窗口里；窗口 = txg > N − 3S，
-    /// N 取现行那一版的 txg，3S 是环长）。
-    RollbackTargetOutsideTheOneRevolutionWindow,
     /// 回退目标那一版树表 0 条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根）。
     RollbackTargetWithoutFile,
     /// 要建立新实例的那一版树表 0 条、而它的实例表已经不是 mkfs 那一片（上一次挂载在这一版上写过行）：
@@ -831,9 +818,6 @@ impl ModelRefusalReason {
             ModelRefusalReason::RollbackTargetNotInRing => "回退目标不在根环里",
             ModelRefusalReason::RollbackTargetBelowEffectiveFloor => "回退目标低于 F_生效",
             ModelRefusalReason::RollbackTargetOnAbandonedTimeline => "回退目标在被抛弃的时间线上",
-            ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow => {
-                "回退目标的 txg 落在现行那一版的一圈窗口外"
-            }
             ModelRefusalReason::RollbackTargetWithoutFile => "回退目标那一版没有文件",
             ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem => {
                 "树表 0 条、而实例表已经不是 mkfs 那一片（条款没写被换下的那一片记在哪）"
@@ -858,7 +842,6 @@ impl ModelRefusalReason {
             | ModelRefusalReason::RollbackTargetNotInRing
             | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
             | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
-            | ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow
             | ModelRefusalReason::RollbackTargetWithoutFile
             | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
             | ModelRefusalReason::FloorAboveCeiling
@@ -1400,7 +1383,7 @@ impl IdealModel {
     }
 
     /// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的有效根, 第 4 新的非空有效根)；非空有效根不足 4 个时取最旧的有效根。
-    /// 有效 = 按最新根的实例表不被抛弃 ∧ txg ≥ 现行的 F ∧ 落在一圈窗口里（「非空」从盘上怎么认，2026-09-17 用户定案；窗口那一项 B2）；现行的 F 取 F_生效（不建崩溃时与
+    /// 有效 = 按最新根的实例表不被抛弃 ∧ txg ≥ 现行的 F（「非空」从盘上怎么认，2026-09-17 用户定案；B2 的窗口那一项 2026-10-04 RGall 拿掉）；现行的 F 取 F_生效（不建崩溃时与
     /// 现行根带的相等）。非空 = inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排）的不同，最旧的有效根与「没有这两棵树」比。
     /// 一块盘上没有有效根时不算它（条款没写这一格；不建崩溃时抬 F 与暖机都推到每块盘都有，走不到）。预想，跟收口表第 ② 行
     /// （候选集下界用哪个 F、被抛弃根带的 F 算不算都在那一行里重议）。
@@ -1408,23 +1391,13 @@ impl IdealModel {
     pub fn rollback_floor_ceiling(&self) -> Option<ModelCheckpointTxg> {
         let current_floor = self.effective_rollback_floor();
         let newest_table = Rc::clone(&self.newest_root().instance_table_rows);
-        // 上限与「非空」只在一圈窗口里的有效根上取（B2，用户 2026-10-03 定，D16 已定项 1 同取这个窗口的第六处）：锚 = 环里最新的有效根。
-        let window_anchor = self
-            .ring
-            .values()
-            .filter(|root| !Self::is_abandoned_by(root, &newest_table))
-            .map(|root| root.key.checkpoint_txg)
-            .max()?;
+        // 上限与「非空」在环里全部有效根上取：B2（用户 2026-10-03 定）原来只取一圈窗口里的，用户 2026-10-04 定 RGall 与回收门槛一起退回整个环。
         let mut valid: Vec<&ModelRoot> = self
             .ring
             .values()
             .filter(|root| {
                 !Self::is_abandoned_by(root, &newest_table)
                     && root.key.checkpoint_txg >= current_floor
-                    && is_inside_the_one_revolution_window_of(
-                        root.key.checkpoint_txg,
-                        window_anchor,
-                    )
             })
             .collect();
         valid.sort_by_key(|root| root.key);
@@ -2401,13 +2374,6 @@ impl IdealModel {
                 if Self::is_abandoned_by(target_root, &current.instance_table_rows) {
                     required_refusals.insert(ModelRefusalReason::RollbackTargetOnAbandonedTimeline);
                 }
-                if !is_inside_the_one_revolution_window_of(
-                    target.checkpoint_txg,
-                    current.key.checkpoint_txg,
-                ) {
-                    required_refusals
-                        .insert(ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow);
-                }
                 if !target_root.contents.has_trees() {
                     required_refusals.insert(ModelRefusalReason::RollbackTargetWithoutFile);
                 }
@@ -2743,7 +2709,6 @@ impl IdealModel {
             | ModelRefusalReason::RollbackTargetNotInRing
             | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
             | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
-            | ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow
             | ModelRefusalReason::RollbackTargetWithoutFile
             | ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem
             | ModelRefusalReason::FloorAboveCeiling
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index 86862b3..8a6d857 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -730,8 +730,8 @@ pub fn refusal_reason_of_placement_refusal(refusal: &PlacementRefusal) -> Observ
 }
 
 /// 回退目标不在候选集里的那一条说的是哪条理由：按字段一对一映射，不看给人看的文字（增补 3 第 2 件代码三方第一轮判决第三节第 2 条）。
-/// 候选集五条（D23（journal 的角色与格式） 已定项 14：根环里、txg ≥ F_生效、按现行那一版的实例表判仍然有效、带文件；D16（发布语义） 已定项 1
-/// 「回退候选集」：txg 落在一圈窗口里），各映射到自己那一条。
+/// 候选集四条（D23（journal 的角色与格式） 已定项 14：根环里、txg ≥ F_生效、按现行那一版的实例表判仍然有效、带文件；D16（发布语义） 已定项 1
+/// 「回退候选集」原来另有「txg 落在一圈窗口里」一条，用户 2026-10-04 定 RGall 拿掉），各映射到自己那一条。
 #[must_use]
 pub fn refusal_reason_of_rollback_candidate_exclusion(
     exclusion: RollbackCandidateExclusion,
@@ -744,9 +744,6 @@ pub fn refusal_reason_of_rollback_candidate_exclusion(
         RollbackCandidateExclusion::OnAbandonedTimeline => {
             ModelRefusalReason::RollbackTargetOnAbandonedTimeline
         }
-        RollbackCandidateExclusion::OutsideTheOneRevolutionWindow => {
-            ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow
-        }
         RollbackCandidateExclusion::VersionWithoutFile => {
             ModelRefusalReason::RollbackTargetWithoutFile
         }
@@ -1289,9 +1286,9 @@ mod tests {
         );
     }
 
-    /// 回退候选集的五条排除各映射到自己那一条理由、互不相同（此前「不在候选集里」映射成「低于 F、被抛弃」两条之一）；
-    /// 「目标那一版树表 0 条」是第五条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根），「落在一圈窗口外」是第四条
-    /// （D16（发布语义） 已定项 1「回退候选集」）。
+    /// 回退候选集的四条排除各映射到自己那一条理由、互不相同（此前「不在候选集里」映射成「低于 F、被抛弃」两条之一）；
+    /// 「目标那一版树表 0 条」是第四条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根）。原来的「落在一圈窗口外」那一条
+    /// 用户 2026-10-04 定 RGall 拿掉（D16（发布语义） 已定项 1「回退候选集」）。
     #[test]
     fn each_rollback_candidate_exclusion_maps_to_its_own_reason() {
         let target = RollbackTarget {
@@ -1302,7 +1299,6 @@ mod tests {
             RollbackCandidateExclusion::NotInRing,
             RollbackCandidateExclusion::BelowEffectiveFloor,
             RollbackCandidateExclusion::OnAbandonedTimeline,
-            RollbackCandidateExclusion::OutsideTheOneRevolutionWindow,
             RollbackCandidateExclusion::VersionWithoutFile,
         ]
         .into_iter()
@@ -1319,9 +1315,6 @@ mod tests {
                 ObservedRefusalReason::Explained(
                     ModelRefusalReason::RollbackTargetOnAbandonedTimeline
                 ),
-                ObservedRefusalReason::Explained(
-                    ModelRefusalReason::RollbackTargetOutsideTheOneRevolutionWindow
-                ),
                 ObservedRefusalReason::Explained(ModelRefusalReason::RollbackTargetWithoutFile),
             ]
         );
diff --git a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
index 569f87b..eb62325 100644
--- a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
+++ b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
@@ -2353,7 +2353,8 @@ fn known_root_slot_still_bad_after_one_reread_refuses_the_floor_raise_before_any
 }
 
 /// 挂载那一刻就读不出、这个进程之后没写过的槽，算准入抬 F 的上限时当作可能是一条非空有效根、F 不越过它
-/// （D16（发布语义） 已定项 1「抬 F 的上限」那一行，用户 2026-10-04 定 RS3；Q5：只并读不出的槽）：同一个池进程退出，(区域 0, 槽 0)
+/// （D16（发布语义） 已定项 1「抬 F 的上限」那一行，用户 2026-10-04 定 RS3；同日 Q3 重定成读不出与自证不过两类都并，这一条是读不出那一类）：
+/// 同一个池进程退出，(区域 0, 槽 0)
 /// 从这时起每一次读都报错，再可写挂载——挂载读根环时它就读不出，这个进程知道住着根的槽里没有它，根环表把它记成窗口里读不出的槽、
 /// 按它能住的最大 txg 记：那一槽在一圈窗口里只能住 txg 0（mkfs 那条根；下一次轮到它是 txg 3S，还没写到）。之后抬 F 到上限：
 /// 读得出的根算出的上限是 1（最旧的有效根是暖机那条 txg 1），并进那一槽的 0 之后上限是 0、不高于现行的 F 0——
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 56effc4..429c73c 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -4931,6 +4931,8 @@ fn overwrite_in_the_second_instance(
 /// 同一段历史在实例 2 里接着发三版（txg 8–10），再把 F 抬到 6：抬 F 生效之后回收释放代 ≤ 6 的落点，
 /// (1, 5) 那一版被写行发布 txg 6 换下的单元（释放代 6）就在其中——不再算已分配。把它照旧并进遍历就是遍历多算，I-3.1 红。
 /// 环里比它老的有效根还在（txg 1–4），第 ④ 条拦不住这一格，拦它的只有第 ③ 条。
+/// 准入抬 F 越得过 5：txg 5 那一槽的根从没落盘、读回全零，盘上全零的根槽另算「从没写过」，不并进抬 F 的上限（用户 2026-10-04 定，
+/// 条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）。照 Q3 把它当自证不过的槽并进来时上限是 5、抬到 6 被拒。
 #[test]
 fn version_applied_only_by_its_journal_record_leaves_the_walk_once_the_rollback_floor_is_raised_past_it(
 ) {
@@ -7793,10 +7795,12 @@ const SYSTEM_CONFIGURATION_REGION_COUNT_OFFSET: usize = 361;
 const REGION_COUNT_PAST_THE_REGION_DEVICE_FIELDS: u8 = 4;
 
 /// I-8.5（tail 槽有效集非空）数的是自证过的槽，「自证过而不可择」的也算（`research/prompts/m3-batch4-code-r1-sonnet-output.md`
-/// 第五节盲区第 9 条）：全池四个槽的 R 都写 4、整槽校验和重封 ⇒ 一槽都择不出、每一槽都自证过 ⇒ I-8.5 真被评估过且成立，
-/// 别的每一条报不适用，一条都不红。
+/// 第五节盲区第 9 条）：全池四个槽的 R 都写 4、整槽校验和重封 ⇒ 一槽都择不出、每一槽都自证过 ⇒ 全池皆废那一格不成立、I-8.5 不判红；
+/// 而一块盘都交不出可择的系统配置，这不是一个挂得上的镜像：I-8.5 与别的每一条一样报不适用、带理由，不判成立
+/// （`InvariantVerdict`「不适用必须带理由，不许静默算成立」；改之前 I-8.5 在 `chosen.is_empty()` 早退之前判成成立）。一条都不红。
 #[test]
-fn self_verified_system_configuration_slots_no_reader_can_choose_still_witness_the_tail() {
+fn self_verified_system_configuration_slots_no_reader_can_choose_leave_every_invariant_not_applicable(
+) {
     let clean = build_pool("tail-slot-witnesses-self-verified-but-not-choosable").memory_pool();
     let mut image = clean.clone();
     for (device, offset) in every_system_configuration_slot(&clean) {
@@ -7810,18 +7814,24 @@ fn self_verified_system_configuration_slots_no_reader_can_choose_still_witness_t
         });
     }
     let verdicts = check_pool_image(&image);
+    let judged: Vec<&(&str, InvariantVerdict)> = verdicts
+        .iter()
+        .filter(|(_, verdict)| !matches!(verdict, InvariantVerdict::NotApplicable(_)))
+        .collect();
     assert_eq!(
-        violated_invariants(&verdicts),
-        Vec::<&str>::new(),
-        "每一槽都自证过：一条都不红：{verdicts:?}"
+        judged,
+        Vec::<&(&str, InvariantVerdict)>::new(),
+        "一槽都择不出：每一条都报不适用，I-8.5 也不判成立"
     );
-    assert_eq!(
-        verdict(
-            &image,
-            SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
+    assert!(
+        matches!(
+            verdict(
+                &image,
+                SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
+            ),
+            InvariantVerdict::NotApplicable(_)
         ),
-        InvariantVerdict::Holds,
-        "自证过而不可择的槽也是 tail 的见证者：I-8.5 真被评估过且成立"
+        "自证过而不可择的槽在：I-8.5 不判红，报不适用"
     );
 }
 
diff --git a/crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs b/crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs
index 4f87495..920d701 100644
--- a/crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs
+++ b/crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs
@@ -26,14 +26,15 @@ use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::instance_table::InstanceRow;
 use singlefs_core::instance_table_sweep::{
     erase_the_heads_and_publish_the_sweep, judge_the_instance_table_sweep,
-    sweep_the_instance_table_while_mounted, unit_head_read_by_sweep,
+    sweep_the_instance_table_while_mounted, unit_head_read_by_sweep, HeadRereadBeforeErasing,
     InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
     InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
     NoRowDeletedThisRound, PlacementOfAUnitHead, SweepPublishPlacements, UnitHeadReadBySweep,
 };
 use singlefs_core::make_filesystem::make_filesystem;
 use singlefs_core::mount::{
-    mount_writable, mount_writable_with_instance_table_sweep_configuration, Mounted, ShadowLedger,
+    mount_writable, mount_writable_with_instance_table_sweep_configuration, unmount, Mounted,
+    ShadowLedger,
 };
 use singlefs_core::recovery::{
     choose_system_configuration, instance_table_chain_of_root, readable_roots, recover,
@@ -605,9 +606,10 @@ fn the_background_sweep_with_a_device_missing_deletes_no_row() {
 }
 
 /// 场景：清扫交给后台。后台那一轮先判（单子上是孤儿），判完到抹之间实例 3 接着覆盖写，直到单子上有一个落点被现行那一版的数据单元占了，再抹、发。
-/// 预期：抹之前重读那一槽的头：已经不是候选行的未发布单元，不抹；单子上别的照抹，实例 1 那一行照删；现行那一版的内容读得回来、那几条不变量不红。
+/// 预期：抹之前那一遍重读到那一槽的头已经不是判的那个单元：这一轮不删、一个头都不抹（C，用户 2026-10-04 定：读回的不是那个头这一轮都不删；
+/// 原来 F1 是那一槽不抹、单子上别的照抹、照删行）；实例 1 那一行还在；现行那一版的内容读得回来、那几条不变量不红。
 #[test]
-fn the_background_sweep_rereads_each_head_before_erasing_it_and_leaves_a_slot_allocated_after_the_judgement(
+fn the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement(
 ) {
     let pool = pool_with_a_dead_instance(None);
     let (mut mounted, mut devices, stream) = sweeping_mount_on(
@@ -675,26 +677,32 @@ fn the_background_sweep_rereads_each_head_before_erasing_it_and_leaves_a_slot_al
         &judgement,
         SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
     );
-    let InstanceTableSweepOutcome::RowsDeleted {
-        rows_deleted,
-        heads_erased,
-    } = outcome
-    else {
-        panic!("这一轮删了行：{outcome:?}");
-    };
     assert!(
-        !heads_erased.contains(&reused),
-        "被现行那一版占了的那一槽没抹"
+        matches!(
+            outcome,
+            InstanceTableSweepOutcome::NoRowDeleted(
+                NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit {
+                    placement,
+                    reread: HeadRereadBeforeErasing::NotTheJudgedUnit,
+                }
+            ) if placement == reused
+        ),
+        "抹之前那一遍重读到那一槽已不是判的那个头：这一轮不删（C）：{outcome:?}"
     );
+    let after = image_after(&pool.image, &stream);
     assert_eq!(
-        rows_deleted
-            .iter()
-            .map(|row| row.instance)
-            .collect::<Vec<_>>(),
-        vec![InstanceGeneration(1)],
-        "实例 1 那一行照删"
+        instances_with_a_row_on(&after),
+        vec![InstanceGeneration(1), InstanceGeneration(2)],
+        "实例 1 那一行还在"
     );
-    let after = image_after(&pool.image, &stream);
+    // 单子上的落点要么还是判的那个孤儿头、要么已被实例 3 后来的写占了（认得出头），没有一个被抹成认不出头。
+    for to_erase in &judgement.heads_to_erase {
+        assert!(
+            unit_head_read_by_sweep(&head_bytes_at(&after, to_erase.placement)).is_some(),
+            "单子上的 {:?} 一个都没抹",
+            to_erase.placement
+        );
+    }
     assert!(
         matches!(
             &recover(&after, JournalPolicy::Consult).outcome,
@@ -705,6 +713,265 @@ fn the_background_sweep_rereads_each_head_before_erasing_it_and_leaves_a_slot_al
     assert_eq!(violations_of(&after), Vec::<String>::new(), "清扫之后");
 }
 
+/// 清扫那次挂载里，第一个孤儿所在的槽判的那一次读照读、第 2 次起（抹之前那一遍重读）按 `read_back` 读坏；
+/// 交回清扫的结局与挂载之后的盘面。
+fn sweep_with_the_reread_of_the_first_orphan_read_bad(
+    pool: &PoolWithADeadInstance,
+    read_back: UnreadableRangeReadBack,
+) -> (InstanceTableSweepAtTheEndOfTheMount, MemoryPool) {
+    let (first_orphan, _) = *pool
+        .orphans_left_on_disk
+        .first()
+        .expect("前提：盘上还留着实例 1 的孤儿");
+    let unreadable = SharedUnreadableRanges::new(
+        vec![UnreadableRange {
+            device: first_orphan.device,
+            offset_in_bytes: first_orphan.slot.to_device_offset().0,
+            length_in_bytes: 512,
+            failing_reads: FailingReadsOfARange::FromTheNthOnward(2),
+        }],
+        read_back,
+    );
+    let stream = SharedStream::retaining_contents();
+    let mut devices = with_unreadable_ranges(recording_of(&pool.image, &stream), &unreadable);
+    let mounted = mount_writable_with_instance_table_sweep_configuration(
+        &parameters_of_the_ring_hole_pools(),
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        sweeping_at_the_end_of_the_mount_on_any_row(),
+    )
+    .expect("读故障只在清扫读的那一槽：挂载照样做成");
+    (
+        mounted.output.instance_table_sweep,
+        image_after(&pool.image, &stream),
+    )
+}
+
+/// 实例 1 的孤儿里头还在盘上的那几个。
+fn orphans_of_instance_one_still_on(
+    pool: &PoolWithADeadInstance,
+    image: &MemoryPool,
+) -> BTreeSet<PlacementOfAUnitHead> {
+    pool.orphans_left_on_disk
+        .iter()
+        .filter(|(placement, head)| {
+            unit_head_read_by_sweep(&head_bytes_at(image, *placement)) == Some(*head)
+        })
+        .map(|(placement, _)| *placement)
+        .collect()
+}
+
+/// 抹之前那一遍重读读坏（C，用户 2026-10-04 定，`research/prompts/m3-root-ring-code-r3-main-verification.md` 第二节 K1-b 与第六节 Q2；
+/// 攻方 `k1-reread` 那两形）：这一轮不删、一个头都不抹；同一份盘面上不注入读错再扫一次，删行生效、实例 1 的孤儿一个不剩。
+/// 改之前那一槽当「已不是那个单元」不抹、照删行，删行之后实例 1 按删前那一行判未发布的孤儿剩 1 个（攻方 `results/reread.log` 的 nth=2 两行）。
+fn the_reread_read_bad_deletes_no_row_and_the_next_sweep_leaves_no_orphan(
+    read_back: UnreadableRangeReadBack,
+    reread_seen: HeadRereadBeforeErasing,
+) {
+    let pool = pool_with_a_dead_instance(None);
+    let (first_orphan, _) = *pool
+        .orphans_left_on_disk
+        .first()
+        .expect("前提：盘上还留着实例 1 的孤儿");
+    let (sweep, after_the_faulty_sweep) =
+        sweep_with_the_reread_of_the_first_orphan_read_bad(&pool, read_back);
+    assert!(
+        matches!(
+            sweep,
+            InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::NoRowDeleted(
+                NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit {
+                    placement,
+                    reread,
+                }
+            )) if placement == first_orphan && reread == reread_seen
+        ),
+        "抹之前那一遍重读读坏：这一轮不删：{sweep:?}"
+    );
+    assert!(
+        instances_with_a_row_on(&after_the_faulty_sweep).contains(&InstanceGeneration(1)),
+        "实例 1 那一行还在"
+    );
+    assert_eq!(
+        orphans_of_instance_one_still_on(&pool, &after_the_faulty_sweep),
+        placements_of(&pool.orphans_left_on_disk),
+        "一个头都没抹"
+    );
+    let (mounted, _devices, stream) = sweeping_mount_on(
+        &after_the_faulty_sweep,
+        sweeping_at_the_end_of_the_mount_on_any_row(),
+    );
+    let next_sweep = mounted.output.instance_table_sweep;
+    assert!(
+        matches!(
+            &next_sweep,
+            InstanceTableSweepAtTheEndOfTheMount::Swept(
+                InstanceTableSweepOutcome::RowsDeleted { .. }
+            )
+        ),
+        "不注入读错的下一次清扫删行：{next_sweep:?}"
+    );
+    let after_the_next_sweep = image_after(&after_the_faulty_sweep, &stream);
+    assert!(
+        !instances_with_a_row_on(&after_the_next_sweep).contains(&InstanceGeneration(1)),
+        "实例 1 那一行删了"
+    );
+    assert_eq!(
+        orphans_of_instance_one_still_on(&pool, &after_the_next_sweep),
+        BTreeSet::<PlacementOfAUnitHead>::new(),
+        "删行生效之后实例 1 的孤儿一个不剩"
+    );
+    assert_eq!(
+        violations_of(&after_the_next_sweep),
+        Vec::<String>::new(),
+        "清扫之后"
+    );
+}
+
+/// 场景：抹之前那一遍重读第一个孤儿那一槽报设备错。
+/// 预期：这一轮不删（报 `ReadFailed`）、一个头都不抹；下一次不注入读错的清扫删行、实例 1 的孤儿一个不剩。
+#[test]
+fn a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(
+) {
+    the_reread_read_bad_deletes_no_row_and_the_next_sweep_leaves_no_orphan(
+        UnreadableRangeReadBack::DeviceError,
+        HeadRereadBeforeErasing::ReadFailed,
+    );
+}
+
+/// 场景：抹之前那一遍重读第一个孤儿那一槽报成功、字节全 0（认不出头）。
+/// 预期：这一轮不删（报 `NotTheJudgedUnit`）、一个头都不抹；下一次不注入读错的清扫删行、实例 1 的孤儿一个不剩。
+#[test]
+fn zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance(
+) {
+    the_reread_read_bad_deletes_no_row_and_the_next_sweep_leaves_no_orphan(
+        UnreadableRangeReadBack::Zeros,
+        HeadRereadBeforeErasing::NotTheJudgedUnit,
+    );
+}
+
+/// 挂载一次（开关关着、有一行就扫），之后覆盖写 `overwrites` 次；`unmounted` 为真时正常卸载，为假时不卸载就退出（崩在最后一次发布之后）。
+/// 交回清扫之后现行那一版的实例表行数与挂载之后的盘面。
+fn one_mount_of_the_periodic_schedule(
+    image: &MemoryPool,
+    overwrites: u64,
+    unmounted: bool,
+) -> (usize, MemoryPool) {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices = devices_of(image);
+    let mounted = mount_writable_with_instance_table_sweep_configuration(
+        &parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        sweeping_at_the_end_of_the_mount_on_any_row(),
+    )
+    .expect("可写挂载");
+    let instance = mounted.output.instance;
+    let mut allocator = mounted.allocator;
+    let mut current = mounted.current;
+    let rows_after_the_sweep = instance_table_chain_of_root(&pool_of(&devices), current.root())
+        .expect("现行那一版的实例表")
+        .records
+        .rows
+        .len();
+    if matches!(current, PoolVersion::WithoutFile(_)) {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        let first_file = publish_first_file(
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
+        .expect("第一个文件");
+        current = PoolVersion::of_a_version_of_trees(first_file);
+    }
+    // 迭代上界：`overwrites` 次覆盖写；跨轮携带的是现行那一版。
+    for _ in 0..overwrites {
+        let previous = current.into_file_version().expect("带文件");
+        let next = publish_an_overwrite(
+            &mut devices,
+            &mut allocator,
+            &previous,
+            &content_of_the_version(previous.root.checkpoint_txg.0 + 1),
+            instance,
+        );
+        current = PoolVersion::of_a_version_of_trees(next);
+    }
+    if unmounted {
+        unmount(
+            &parameters,
+            &mut devices,
+            &mut allocator,
+            &mut current,
+            ShadowLedger::On,
+        )
+        .expect("正常卸载");
+    }
+    (rows_after_the_sweep, pool_of(&devices))
+}
+
+/// 一次挂载之后实例表至多几行：根环 12 槽里住着最近几次挂载的根（一次正常卸载的挂载推 5 次左右的发布、一次不卸载的推 3 次左右），
+/// 根还在环里的实例与这一次挂载自己的那一行加起来不过这个数。
+const INSTANCE_ROWS_AT_MOST_ON_THE_PERIODIC_SCHEDULE: usize = 4;
+/// 周期调度挂几次（攻方 `k1_periodic` 的默认值）。
+const MOUNTS_OF_THE_PERIODIC_SCHEDULE: usize = 60;
+/// 每隔几次挂载跑一次池级 checker（攻方 `PROBE_CHECK_EVERY = 5`）。
+const MOUNTS_BETWEEN_CHECKS_OF_THE_PERIODIC_SCHEDULE: usize = 5;
+
+/// 场景：攻方 `k1_periodic` 那一段无故障合法历史（A2，用户 2026-10-04 定，`research/prompts/m3-root-ring-code-r3-main-verification.md` 第二节 K1-a
+/// 与第六节 Q1）：mkfs 之后第一次挂载写第一个文件、正常卸载；之后每次挂载覆盖写 1 次，正常卸载与不卸载交替，共 60 次挂载，每次挂载末尾有一行就扫。
+/// 正常卸载把 F 抬到现行那一版，F 之下的旧根引用的分配记录树节点被合法复用、那条根的账读不出。
+/// 预期：清扫算账时 txg 低于现行 F 的根跳过它的账，每次挂载之后实例表的行数不超过
+/// [`INSTANCE_ROWS_AT_MOST_ON_THE_PERIODIC_SCHEDULE`]；每 5 次挂载与最后一次挂载之后那几条不变量不红。
+/// 改之前每一次清扫都停在 `AccountOfARingRootUnreadable`，60 次挂载之后 59 行、一行没删（攻方 `results/long-60.log`）。
+#[test]
+fn sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short() {
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
+    let mut image = pool_of(&devices);
+    // 迭代上界：60 次挂载；跨轮携带的是上一次挂载之后的盘面。
+    for mount_index in 0..MOUNTS_OF_THE_PERIODIC_SCHEDULE {
+        let (overwrites, unmounted) = if mount_index == 0 {
+            (0, true)
+        } else {
+            (1, mount_index % 2 == 1)
+        };
+        let (rows_after_the_sweep, after) =
+            one_mount_of_the_periodic_schedule(&image, overwrites, unmounted);
+        assert!(
+            rows_after_the_sweep <= INSTANCE_ROWS_AT_MOST_ON_THE_PERIODIC_SCHEDULE,
+            "第 {mount_index} 次挂载清扫之后实例表 {rows_after_the_sweep} 行"
+        );
+        if mount_index % MOUNTS_BETWEEN_CHECKS_OF_THE_PERIODIC_SCHEDULE == 0
+            || mount_index + 1 == MOUNTS_OF_THE_PERIODIC_SCHEDULE
+        {
+            assert_eq!(
+                violations_of(&after),
+                Vec::<String>::new(),
+                "第 {mount_index} 次挂载之后"
+            );
+        }
+        image = after;
+    }
+}
+
 /// 场景：清扫那次挂载用只供测试的落点开关：清扫发布在一份每个空闲槽都被占掉的分配器拷贝上取落点（真的取落点那一段，被拒）。
 /// 预期：清扫发布被落点拒，这一轮不删行（实例 1 那一行还在）；挂载照样交回可写——接着在交回的分配器上覆盖写一次做成。
 #[test]
diff --git a/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs b/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs
index 0f41d7d..31e7961 100644
--- a/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs
+++ b/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs
@@ -1,50 +1,74 @@
-//! 根环上有洞之后的回收门槛取一圈窗口（D16（发布语义） 已定项 1「可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」，
-//! 用户 2026-10-02 定；`research/prompts/m3-ring-r3-main-verification.md` 第二节第 3–5 行）。
+//! 根环上有洞之后的回收门槛（D16（发布语义） 已定项 1「可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）：2026-10-02 起取一圈窗口，
+//! 用户 2026-10-04 定 RGall 退回整个环（`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）。文件名留着窗口那一轮的叫法，
+//! 盘面就是当初定窗口用的那一形。
 //!
 //! 盘面（与崩溃注入种子 0 第 135 段那一形同形，`/tmp/claude-1000/inject6-findings/report.md` 第三节）：实例 1 在 24 槽的根环上发到 txg 30，
 //! txg 31 那次发布崩在「记录已持久、根槽没持久」（根槽与之后的系统配置轮换都没落：那一槽里留着上一圈的 txg 7 的根）。
 //! 可写挂载（实例 2）恢复由记录施加出 txg 31 那一版，写行 txg 32、暖机 txg 33、34 盖掉 txg 8、9、10 的槽；之后覆盖写 txg 35、36、37。
 //! txg 7 的根一直留在 txg 31 那一槽里，是环里最旧的有效根，而 txg 8–11 已经离开根环。
 //!
-//! - 分配器（进程内每记一条根，锚在这条根）：txg 35 起窗口是 txg > 11，txg 7 在窗口外，门槛跟着窗口里最旧的有效根往前走；
-//!   照「环里最旧有效根」取时门槛卡在 7，只被 txg 8–11 那几版引用、已释放的槽回收不了、仍算已分配，而根环里已没有根引用它们 ⇒ I-3.1 红。
-//! - 池级 checker 的候选过滤：txg 7 在镜像的窗口外（N = 最新根），不当候选走；照旧当候选走时，它引用的槽已被窗口门槛回收、可能已被复用 ⇒
-//!   I-3.1 / I-2.1 / I-7.4 红。
-//!
-//! 每次覆盖写之后在盘上跑池级 checker，判这几条不红。
+//! - 分配器（进程内每记一条根）：门槛卡在 txg 7，txg 7 那一版引用的单元一个都不回收。照一圈窗口取时 txg 35 起 txg 7 在窗口外，
+//!   门槛跟着窗口里最旧的有效根往前走，txg 7 引用、已释放的单元当场回收。
+//! - 重新挂载（重建分配器）与抬 F（抬 F 那一遍的回收）同一个门槛：txg 7 的单元仍不回收。
+//! - 池级 checker：txg 7 是候选、照走；只被 txg 8–11 那几版引用的单元由「由记录施加出来、根槽从没落盘的那一版」那一路走到
+//!   （它们的记录还在 journal 环里），记账与遍历对得上。
+//! - 洞里 txg 7 的根所在那一槽再挂那一刻读不出（txg 7 比 N − 3S 还旧）：读坏的槽在回收门槛里钉在它可能住过的最旧 txg 7
+//!   （用户 2026-10-04 定，条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`），挂载时、进程内与抬 F 那一遍都不越过它。
 mod common;
 
 use common::{
     build_pool, crash_state_devices, memory_pool_of_sparse_devices,
     overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
-    system_configuration_slots_of_every_device, write_back_system_configuration_slots,
-    FIXED_WRITE_TIME_SECONDS,
+    system_configuration_slots_of_every_device, unreadable_root_slot_of, with_unreadable_ranges,
+    write_back_system_configuration_slots, DeviceWithUnreadableRanges, FailingReadsOfARange,
+    SharedUnreadableRanges, UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
 };
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
+use singlefs_core::allocator::{PoolAllocator, TxgsARingSlotBadAtMountCanHold};
 use singlefs_core::block_device::BlockDevice;
-use singlefs_core::mount::mount_writable;
+use singlefs_core::mount::{mount_writable, raise_rollback_floor, ShadowLedger};
+use singlefs_core::recovery::{
+    allocation_records_under_root, choose_system_configuration, readable_roots_with_ring_slots,
+};
+use singlefs_core::root_record::RootRecord;
 use singlefs_core::root_ring::{slot_offset, target_for_publish};
-use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
-use singlefs_harness::SharedStream;
+use singlefs_core::transaction::{
+    publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
+};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
 
 /// 根槽没落的那次发布的 txg：它的槽里留着上一圈 txg 7 的根（24 槽的环，31 − 24 = 7）。
 const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 31;
 /// 洞那一槽里留着的上一圈的根。
 const TXG_OF_THE_STALE_ROOT_IN_THE_HOLE: u64 = 7;
+/// 挂载之后覆盖写到的那一版（txg 35、36、37）。
+const LAST_TXG_OF_INSTANCE_TWO: u64 = 37;
 /// 一次覆盖写的内容长度：一个数据单元装得下。
 const OVERWRITE_BYTES: usize = 3000;
 /// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
-const INVARIANTS_THE_WINDOW_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
+const INVARIANTS_THE_RING_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
+
+type RecordedDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;
 
 fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
     vec![u8::try_from(checkpoint_txg % 251).expect("小于 256"); OVERWRITE_BYTES]
 }
 
-#[test]
-fn reclaiming_after_a_hole_in_the_root_ring_follows_the_one_revolution_window_and_keeps_the_allocated_statistic(
-) {
+/// 挂载之后覆盖写到 txg 37 那一刻：实例 2 的盘、分配器、现行那一版。
+struct PoolAfterTheHole {
+    devices: RecordedDevices,
+    allocator: PoolAllocator,
+    current: TransactionOutput,
+}
+
+/// 造盘面：实例 1 发到 txg 30、txg 31 崩在根槽之前，可写挂载（实例 2）之后覆盖写到 txg `last`；每次覆盖写之后调 `after_each_overwrite`。
+fn pool_after_the_hole_overwritten_through(
+    last: u64,
+    after_each_overwrite: &mut dyn FnMut(u64, &RecordedDevices, &PoolAllocator),
+) -> PoolAfterTheHole {
     let mut pool = build_pool("one-revolution-window-after-a-hole");
     let slots_per_region = parameters().geometry.root_ring_slots_per_region;
     assert_eq!(
@@ -97,7 +121,7 @@ fn reclaiming_after_a_hole_in_the_root_ring_follows_the_one_revolution_window_an
 
     let stream = SharedStream::retaining_contents();
     let mut devices = crash_state_devices(&crash_image, &[], &[], &stream);
-    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
+    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
     assert_eq!(
         (
             mounted.output.instance,
@@ -107,17 +131,19 @@ fn reclaiming_after_a_hole_in_the_root_ring_follows_the_one_revolution_window_an
         (InstanceGeneration(2), CheckpointTxg(31), CheckpointTxg(34)),
         "恢复由记录施加出 txg 31；写行 txg 32、暖机 txg 33、34"
     );
+    let mut allocator = mounted.allocator;
     let PoolVersion::WithFile(mut current) = mounted.current else {
         panic!("txg 31 那一版带文件");
     };
     let publish_parameters = parameters();
-    // 迭代上界：覆盖写三次（txg 35、36、37）；跨轮携带的是现行那一版。
-    for checkpoint_txg in 35..=37u64 {
+    // 迭代上界：覆盖写到 txg `last`；跨轮携带的是现行那一版。
+    while current.root.checkpoint_txg.0 < last {
+        let checkpoint_txg = current.root.checkpoint_txg.0 + 1;
         current = {
             let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
             publish_overwrite(
                 &mut writer,
-                &mut mounted.allocator,
+                &mut allocator,
                 &current,
                 FirstFile {
                     content: &content_of_txg(checkpoint_txg),
@@ -128,21 +154,303 @@ fn reclaiming_after_a_hole_in_the_root_ring_follows_the_one_revolution_window_an
             .expect("覆盖写")
         };
         assert_eq!(current.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
-        let image = memory_pool_of_sparse_devices(&devices);
-        let violations: Vec<String> = check_pool_image(&image)
-            .into_iter()
-            .filter(|(invariant, _)| INVARIANTS_THE_WINDOW_KEEPS_GREEN.contains(invariant))
-            .filter_map(|(invariant, verdict)| match verdict {
-                InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
-                InvariantVerdict::Holds
-                | InvariantVerdict::NotApplicable(_)
-                | InvariantVerdict::PartiallyJudged(_) => None,
-            })
-            .collect();
+        after_each_overwrite(checkpoint_txg, &devices, &allocator);
+    }
+    PoolAfterTheHole {
+        devices,
+        allocator,
+        current,
+    }
+}
+
+fn violations_of(image: &MemoryPool) -> Vec<String> {
+    check_pool_image(image)
+        .into_iter()
+        .filter(|(invariant, _)| INVARIANTS_THE_RING_KEEPS_GREEN.contains(invariant))
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
+        })
+        .collect()
+}
+
+/// 洞那一槽里留着的 txg 7 的根（读得出的那一条）。
+fn stale_root_in_the_hole(image: &MemoryPool) -> RootRecord {
+    let system_configuration = choose_system_configuration(image).expect("系统配置");
+    let immutable = &system_configuration.immutable;
+    readable_roots_with_ring_slots(
+        image,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    )
+    .into_iter()
+    .map(|(_, root)| root)
+    .find(|root| {
+        (root.instance, root.checkpoint_txg)
+            == (
+                InstanceGeneration(1),
+                CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
+            )
+    })
+    .expect("txg 7 的根还在环里、读得出")
+}
+
+/// txg 7 那一版的账里仍分配、而分配器上空着（可再分配）的槽：(盘, 槽)。
+fn slots_of_the_stale_root_left_free(
+    image: &MemoryPool,
+    allocator: &PoolAllocator,
+) -> Vec<(u32, u64)> {
+    let stale_root = stale_root_in_the_hole(image);
+    let records =
+        allocation_records_under_root(image, &stale_root).expect("txg 7 那一版的账读得出");
+    let mut left_free = Vec::new();
+    for record in records.iter().filter(|record| !record.is_released) {
+        let device_map = allocator
+            .devices
+            .iter()
+            .find(|device_map| device_map.device == record.device)
+            .expect("账里的盘在分配器里");
+        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
+            if device_map.is_free(SlotNumber(slot)) {
+                left_free.push((record.device.0, slot));
+            }
+        }
+    }
+    left_free
+}
+
+/// 场景：挂载之后覆盖写 txg 35、36、37（这个进程每记一条根就按环里最旧有效根回收）。
+/// 预期：每次覆盖写之后 txg 7 那一版引用的单元在分配器上一个都不空闲，盘上那几条不变量不红。
+#[test]
+fn reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated(
+) {
+    pool_after_the_hole_overwritten_through(
+        LAST_TXG_OF_INSTANCE_TWO,
+        &mut |checkpoint_txg, devices, allocator| {
+            let image = memory_pool_of_sparse_devices(devices);
+            assert_eq!(
+                slots_of_the_stale_root_left_free(&image, allocator),
+                Vec::<(u32, u64)>::new(),
+                "txg {checkpoint_txg} 之后：txg 7 是环里最旧的有效根，它引用的单元不回收"
+            );
+            assert_eq!(
+                violations_of(&image),
+                Vec::<String>::new(),
+                "txg {checkpoint_txg} 之后"
+            );
+        },
+    );
+}
+
+/// 场景：覆盖写到 txg 37 之后进程退出，再可写挂载（重建分配器：挂载时的门槛锚在环里读得出的最新有效根 37，txg 7 落在它的一圈窗口外）。
+/// 预期：重建出来的分配器上 txg 7 那一版引用的单元一个都不空闲，挂载之后盘上那几条不变量不红。
+#[test]
+fn a_remount_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated() {
+    let pool = pool_after_the_hole_overwritten_through(LAST_TXG_OF_INSTANCE_TWO, &mut |_, _, _| {});
+    let image = memory_pool_of_sparse_devices(&pool.devices);
+    let stream = SharedStream::retaining_contents();
+    let mut devices = crash_state_devices(&image, &[], &[], &stream);
+    let remounted = mount_writable(&parameters(), &mut devices).expect("再可写挂载");
+    let after = memory_pool_of_sparse_devices(&devices);
+    assert_eq!(
+        slots_of_the_stale_root_left_free(&after, &remounted.allocator),
+        Vec::<(u32, u64)>::new(),
+        "再挂之后：txg 7 仍是环里最旧的有效根，重建分配器不回收它引用的单元"
+    );
+    assert_eq!(violations_of(&after), Vec::<String>::new(), "再挂之后");
+}
+
+/// 场景：覆盖写到 txg 37 之后在同一个进程里把 F 抬到 7（不高于上限）。
+/// 预期：抬 F 那一遍的回收门槛 = max(7, 环里最旧有效根 7) = 7，txg 7 那一版引用的单元一个都不空闲，抬完之后盘上那几条不变量不红。
+/// 照一圈窗口取时那一遍的门槛是窗口里最旧的有效根 14，txg 7 引用的单元扣到生效之后放开。
+#[test]
+fn raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated()
+{
+    let mut pool =
+        pool_after_the_hole_overwritten_through(LAST_TXG_OF_INSTANCE_TWO, &mut |_, _, _| {});
+    raise_rollback_floor(
+        &parameters(),
+        &mut pool.devices,
+        &mut pool.allocator,
+        &mut pool.current,
+        CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
+        ShadowLedger::On,
+    )
+    .expect("抬 F 到 7");
+    let image = memory_pool_of_sparse_devices(&pool.devices);
+    assert_eq!(
+        slots_of_the_stale_root_left_free(&image, &pool.allocator),
+        Vec::<(u32, u64)>::new(),
+        "抬 F 到 7 之后：txg 7 引用的单元不回收"
+    );
+    assert_eq!(violations_of(&image), Vec::<String>::new(), "抬 F 之后");
+}
+
+/// 覆盖写到 txg 37 之后进程退出，再可写挂载（实例 3）：洞里 txg 7 的根所在的那一槽（txg 31 的槽）从这一次挂载起按 `slot_reads` 报设备错。
+/// 交回盘（带读故障）、它的读故障、起点盘面、录下的流与挂载交回的分配器、现行那一版。
+struct RemountReadingTheSlotOfTheStaleRootBad {
+    devices: Vec<(
+        DeviceIdentity,
+        DeviceWithUnreadableRanges<RecordingBlockDevice<SparseBlockDevice>>,
+    )>,
+    unreadable_slot: SharedUnreadableRanges,
+    image_before_the_remount: MemoryPool,
+    stream: SharedStream,
+    allocator: PoolAllocator,
+    current: TransactionOutput,
+}
+
+impl RemountReadingTheSlotOfTheStaleRootBad {
+    fn image_now(&self) -> MemoryPool {
+        let mut image = self.image_before_the_remount.clone();
+        image.apply(&self.stream.retained_operations());
+        image
+    }
+}
+
+fn remount_reading_the_slot_of_the_stale_root_bad() -> RemountReadingTheSlotOfTheStaleRootBad {
+    let pool = pool_after_the_hole_overwritten_through(LAST_TXG_OF_INSTANCE_TWO, &mut |_, _, _| {});
+    let image_before_the_remount = memory_pool_of_sparse_devices(&pool.devices);
+    let stream = SharedStream::retaining_contents();
+    let unreadable_slot = SharedUnreadableRanges::new(
+        vec![unreadable_root_slot_of(
+            CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
+            FailingReadsOfARange::Every,
+        )],
+        UnreadableRangeReadBack::DeviceError,
+    );
+    let mut devices = with_unreadable_ranges(
+        crash_state_devices(&image_before_the_remount, &[], &[], &stream),
+        &unreadable_slot,
+    );
+    let remounted = mount_writable(&parameters(), &mut devices).expect("再可写挂载");
+    assert!(
+        unreadable_slot.failed_reads() > 0,
+        "再挂那一次读洞那一槽读错了"
+    );
+    let PoolVersion::WithFile(current) = remounted.current else {
+        panic!("txg 37 那一版带文件");
+    };
+    RemountReadingTheSlotOfTheStaleRootBad {
+        devices,
+        unreadable_slot,
+        image_before_the_remount,
+        stream,
+        allocator: remounted.allocator,
+        current,
+    }
+}
+
+/// 场景：覆盖写到 txg 37 之后再可写挂载，洞里 txg 7 的根所在那一槽再挂那一刻读不出（之后读得出），再覆盖写两次。
+/// txg 7 比 N − 3S = 37 − 24 = 13 还旧：照一圈窗口给读坏的槽划 txg 区间时那一槽只能住 txg 31，门槛取环里读得出的最旧有效根 14，
+/// txg 7 引用、释放代不高于 14 的单元当场回收（实现员报告第 10 批 core 簇第八节第 2 条）。
+/// 预期（用户 2026-10-04 定：读坏的槽钉在它可能住过的最旧 txg）：根环表把那一槽钉在 7（能住的最大 txg 31）；再挂之后与每一次覆盖写之后
+/// txg 7 那一版引用的单元一个都不空闲，盘上那几条不变量不红；挂载之后第一次覆盖写重读那一槽、读出 txg 7 的根。
+#[test]
+fn a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated(
+) {
+    let mut remounted = remount_reading_the_slot_of_the_stale_root_bad();
+    let slot_of_the_stale_root = target_for_publish(
+        CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
+        parameters().geometry.root_ring_slots_per_region,
+    );
+    assert_eq!(
+        remounted
+            .allocator
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表")
+            .ring_slots_bad_at_mount()
+            .get(&slot_of_the_stale_root),
+        Some(&TxgsARingSlotBadAtMountCanHold::new(
+            CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
+            CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST)
+        )),
+        "根环表把那一槽钉在可能住过的最旧 txg 7，能住的最大 txg 31"
+    );
+    remounted.unreadable_slot.lift();
+    let after_the_remount = remounted.image_now();
+    assert_eq!(
+        (
+            slots_of_the_stale_root_left_free(&after_the_remount, &remounted.allocator),
+            violations_of(&after_the_remount)
+        ),
+        (Vec::new(), Vec::<String>::new()),
+        "再挂之后：txg 7 引用的单元一个都不空闲，那几条不变量不红"
+    );
+    let publish_parameters = parameters();
+    let last_txg = remounted.current.root.checkpoint_txg.0 + 2;
+    // 迭代上界：两次覆盖写；跨轮携带的是现行那一版。
+    while remounted.current.root.checkpoint_txg.0 < last_txg {
+        let checkpoint_txg = remounted.current.root.checkpoint_txg.0 + 1;
+        remounted.current = {
+            let mut writer = PoolWriter::new(&publish_parameters, remounted.devices.as_mut_slice());
+            publish_overwrite(
+                &mut writer,
+                &mut remounted.allocator,
+                &remounted.current,
+                FirstFile {
+                    content: &content_of_txg(checkpoint_txg),
+                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + checkpoint_txg,
+                },
+                InstanceGeneration(3),
+            )
+            .expect("覆盖写")
+        };
+        let occupancy = remounted
+            .allocator
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表");
         assert_eq!(
-            violations,
-            Vec::<String>::new(),
-            "txg {checkpoint_txg} 之后：洞那一槽里的 txg 7 在窗口外，回收门槛与候选集都跟着窗口走"
+            (
+                occupancy
+                    .ring_slots_bad_at_mount()
+                    .get(&slot_of_the_stale_root)
+                    .copied(),
+                occupancy
+                    .ring_slots_known_to_hold_a_root()
+                    .contains(&slot_of_the_stale_root)
+            ),
+            (None, true),
+            "txg {checkpoint_txg}：重读读出 txg 7 的根，那一槽算这个进程知道住着根的槽"
+        );
+        let image = remounted.image_now();
+        assert_eq!(
+            (
+                slots_of_the_stale_root_left_free(&image, &remounted.allocator),
+                violations_of(&image)
+            ),
+            (Vec::new(), Vec::<String>::new()),
+            "txg {checkpoint_txg} 之后"
         );
     }
 }
+
+/// 场景：同上再挂，洞里 txg 7 的根所在那一槽一直读不出，在这一次挂载里把 F 抬到 7（不高于上限）。
+/// 预期：抬 F 那一遍的回收门槛 = max(7, 环里最旧有效根)，那一槽重读仍读不出、钉在 7，门槛是 7：txg 7 那一版引用的单元一个都不空闲，
+/// 抬完之后盘上那几条不变量不红。照一圈窗口划区间时那一槽按 31 并、门槛是读得出的最旧有效根 14，txg 7 引用的单元扣到生效之后放开。
+#[test]
+fn raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated(
+) {
+    let mut remounted = remount_reading_the_slot_of_the_stale_root_bad();
+    raise_rollback_floor(
+        &parameters(),
+        &mut remounted.devices,
+        &mut remounted.allocator,
+        &mut remounted.current,
+        CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
+        ShadowLedger::On,
+    )
+    .expect("抬 F 到 7");
+    let image = remounted.image_now();
+    assert_eq!(
+        (
+            slots_of_the_stale_root_left_free(&image, &remounted.allocator),
+            violations_of(&image)
+        ),
+        (Vec::new(), Vec::<String>::new()),
+        "抬 F 到 7 之后：txg 7 引用的单元一个都不空闲，那几条不变量不红"
+    );
+}
diff --git a/crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs b/crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs
index 30c8074..3247b5f 100644
--- a/crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs
+++ b/crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs
@@ -2,6 +2,8 @@
 //! （`.claude/kb/invariants.md` I-7.9 那一行 2026-10-04 写回，照 D16（发布语义） 已定项 1「抬 F 的上限」只在一圈窗口里的根上取）。
 //! 窗口取两个、一起用：锚在被判那条抬 F 的根上的（条款字面），与锚在这张镜像根环里最新有效根上的（候选过滤那一个）；只取前一个时下面三形
 //! 合法镜像照样红（`crates/singlefs-checker/src/walk.rs` 的 `judge_rollback_floor_raises_against_their_ceilings` 写了理由）。
+//! 用户 2026-10-04 定 RGall 之后抬 F 的上限本身取整个环，前一个窗口拿掉（镜像最新根不早于被判那条根，后一个窗口本来就罩着它），
+//! 只剩锚在镜像最新根上的那一个；文件名与用例名留着两个窗口那一轮的叫法。
 //! 环上有洞时洞那一槽留着上一圈的根，后来的发布盖掉的是比它新的根：在后来那张镜像上重算的上限不再「只会更宽」。三形合法镜像，抬 F 那一刻
 //! 实现与 checker 自己的公式算出同一个上限、F 都等于它，再挂之后洞里的旧根顶上来：
 //! - 删除与并发那条线交来的草稿：卸载那一串空发布记录已落、根没落那一形的最小复现（覆盖写切在「记录已落、根没落」再可写挂载）；
diff --git a/crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs b/crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs
index c9f6489..6cbd090 100644
--- a/crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs
+++ b/crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs
@@ -9,6 +9,11 @@
 //!   改之前上限要读它们的树表、读到被复用的槽，准入抬 F 被拒 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（攻方 `rf-14-none.out`）。
 //! - 回收门槛把挂载时读不出的那一槽并进窗口里最旧有效根（RR4，攻方 B1）：X 那一槽第四次挂载起一直读不出，根环表按它能住的最大 txg 记着；
 //!   改之前抬 F 那一遍只在读得出的根上取门槛，门槛越过 X、回收了 X 引用的槽，记账比遍历少（攻方 `rfo3-17-persist.out`：抬完 I-3.1 红）。
+//!
+//! 用户 2026-10-04 定 RGall 之后（上限与回收门槛都取整个环）：洞那几槽里的旧根是有效根，上限与门槛都由它们定，
+//! 分不出上面两样改之前的做法（B2、RR4 两行变异挪走或拿掉，实现员报告）。同日又定读坏的槽在回收门槛里钉在它可能住过的最旧 txg
+//! （条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）：X 那一槽一直读不出时钉在 9（它在 12 槽环上可能住过的最旧 txg），
+//! 比盘上环里最旧的有效根旧，池级 checker 读盘上完好的 X、按它算门槛，I-3.1 在记账多的一边红，直到这个进程写掉那一槽（第二条用例钉这个代价）。
 mod common;
 mod common_root_ring_holes;
 
@@ -24,12 +29,13 @@ use common_root_ring_holes::{
 };
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
 use singlefs_core::allocator::PoolAllocator;
 use singlefs_core::mount::{
     mount_writable, raise_rollback_floor, raise_rollback_floor_to_the_admission_ceiling,
     MountError, RaiseToTheAdmissionCeiling, ShadowLedger,
 };
+use singlefs_core::recovery::allocation_records_under_root;
 use singlefs_core::root_ring::{slot_offset, target_for_publish};
 use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
@@ -197,12 +203,31 @@ fn admission_raising_the_floor_above_holes_at_the_oldest_end_of_the_window_takes
     );
 }
 
+/// I-3.1 那一条红在记账比遍历多的一边：说明里「记账的已分配 Some(a)，遍历全部有效根得到 b」a 大于 b。
+fn allocated_statistic_counts_more_than_the_walk(violation: &str) -> bool {
+    let after = |marker: &str| {
+        violation.split(marker).nth(1).and_then(|rest| {
+            rest.chars()
+                .take_while(char::is_ascii_digit)
+                .collect::<String>()
+                .parse::<u64>()
+                .ok()
+        })
+    };
+    matches!(
+        (after("记账的已分配 Some("), after("遍历全部有效根得到 ")),
+        (Some(counted), Some(walked)) if counted > walked
+    )
+}
+
 /// 场景：实例 1 覆盖写到 txg 17，三次挂载崩在写行根槽之前（洞 18、19、20），X = 21；第四次挂载起 X 那一槽一直读不出，推到锚 29，
-/// 准入抬 F 到 [`FLOOR_RAISED_BELOW_X`]，再覆盖写三次。
-/// 预期：抬 F 做成，抬完与之后每次覆盖写之后盘上那几条不变量不红：X 那一槽按根环表记的最大 txg 并进回收门槛，门槛不越过 X。
-/// 改之前抬完 I-3.1 红（攻方 `rfo3-17-persist.out`：记账的已分配比遍历全部有效根少）。
+/// 准入抬 F 到 [`FLOOR_RAISED_BELOW_X`]，再覆盖写三次（txg 32、33、34；33 写掉 X 那一槽）。
+/// 预期：抬 F 做成；X 那一槽钉在它可能住过的最旧 txg 9（用户 2026-10-04 定），抬 F 那一遍与之后的进程内回收都不越过它：
+/// X 那一版引用的单元在这个进程写掉那一槽之前一个都不空闲，被引用单元那几条一直不红。
+/// 定案的代价：那一槽钉着的 9 比盘上环里最旧的有效根旧，池级 checker 读盘上完好的 X 按它算门槛，那几版 I-3.1 只许红在记账多的一边（安全方向）；
+/// 33 写掉那一槽之后不再钉，那几条全绿。改 RR4 之前抬完 I-3.1 红在记账少的一边（攻方 `rfo3-17-persist.out`）。
 #[test]
-fn admission_raising_the_floor_counts_the_root_ring_slot_unreadable_since_the_mount_into_the_reclaim_threshold(
+fn admission_raising_the_floor_keeps_the_root_ring_slot_unreadable_since_the_mount_pinned_at_the_oldest_txg_it_can_have_held(
 ) {
     let mut pool = pool_just_before_the_admission_raise(17, 3, FailingReadsOfARange::Every);
     assert_eq!(
@@ -211,6 +236,72 @@ fn admission_raising_the_floor_counts_the_root_ring_slot_unreadable_since_the_mo
         "三个洞紧跟着 X"
     );
     let parameters = parameters_of_the_ring_hole_pools();
+    let x_slot = target_for_publish(
+        CheckpointTxg(pool.x_txg),
+        parameters.geometry.root_ring_slots_per_region,
+    );
+    let txg_that_overwrites_the_x_slot = pool.x_txg + 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
+    let x_root = readable_root_at_txg(&pool.image_now(), pool.x_txg).expect("盘上 X 那一槽完好");
+    let slots_x_references: Vec<(DeviceIdentity, SlotNumber)> =
+        allocation_records_under_root(&pool.image_now(), &x_root)
+            .expect("X 那一版的账读得出")
+            .iter()
+            .filter(|record| !record.is_released)
+            .flat_map(|record| {
+                (record.slot.0..record.slot.0 + u64::from(record.span_slots))
+                    .map(move |slot| (record.device, SlotNumber(slot)))
+            })
+            .collect();
+    let slots_of_x_left_free = |allocator: &PoolAllocator| -> Vec<(DeviceIdentity, SlotNumber)> {
+        slots_x_references
+            .iter()
+            .copied()
+            .filter(|(device, slot)| {
+                allocator
+                    .devices
+                    .iter()
+                    .find(|device_map| device_map.device == *device)
+                    .expect("账里的盘在分配器里")
+                    .is_free(*slot)
+            })
+            .collect()
+    };
+    let judge = |pool_being_judged: &PoolJustBeforeTheAdmissionRaise, when: &str| {
+        let txg = pool_being_judged.current.root.checkpoint_txg.0;
+        let violations = violations_of(
+            &pool_being_judged.image_now(),
+            &INVARIANTS_THE_RAISE_KEEPS_GREEN,
+        );
+        if txg < txg_that_overwrites_the_x_slot {
+            assert_eq!(
+                slots_of_x_left_free(&pool_being_judged.allocator),
+                Vec::new(),
+                "{when}（txg {txg}）：X 那一槽钉在 9，X 引用的单元一个都不空闲"
+            );
+            assert!(
+                violations
+                    .iter()
+                    .all(|violation| violation.starts_with("I-3.1：")
+                        && allocated_statistic_counts_more_than_the_walk(violation)),
+                "{when}（txg {txg}）：只许 I-3.1 红在记账多的一边：{violations:?}"
+            );
+        } else {
+            assert_eq!(
+                violations,
+                Vec::<String>::new(),
+                "{when}（txg {txg}）：X 那一槽写掉了，不再钉"
+            );
+        }
+    };
+    assert_eq!(
+        pool.allocator
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表")
+            .ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held()
+            .get(&x_slot),
+        Some(&CheckpointTxg(9)),
+        "前提：X 那一槽钉在它可能住过的最旧 txg 9"
+    );
     let raised = raise_rollback_floor(
         &parameters,
         &mut pool.devices,
@@ -220,11 +311,7 @@ fn admission_raising_the_floor_counts_the_root_ring_slot_unreadable_since_the_mo
         ShadowLedger::On,
     );
     assert!(raised.is_ok(), "准入抬 F 做成：{raised:?}");
-    assert_eq!(
-        violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN),
-        Vec::<String>::new(),
-        "抬完之后：门槛不越过 X 那一槽"
-    );
+    judge(&pool, "抬完之后");
     // 迭代上界：三次覆盖写；跨轮携带的是现行那一版。
     for step in 0..3u64 {
         let salt = 5000 + step;
@@ -242,11 +329,10 @@ fn admission_raising_the_floor_counts_the_root_ring_slot_unreadable_since_the_mo
             )
             .expect("覆盖写")
         };
-        assert_eq!(
-            violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN),
-            Vec::<String>::new(),
-            "抬完之后第 {step} 次覆盖写（txg {}）",
-            pool.current.root.checkpoint_txg.0
-        );
+        judge(&pool, &format!("抬完之后第 {step} 次覆盖写"));
     }
+    assert!(
+        pool.current.root.checkpoint_txg.0 >= txg_that_overwrites_the_x_slot,
+        "前提：覆盖写走到了写掉 X 那一槽的那一版"
+    );
 }
diff --git a/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs b/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs
index 37b2ebf..aecdad3 100644
--- a/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs
+++ b/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs
@@ -1,13 +1,18 @@
-//! 准入抬 F 抬到上限时，窗口里挂载那一刻读不出、之后这个进程没写过的槽当作可能是一条非空有效根，F 不越过它
-//! （D16（发布语义） 已定项 1「抬 F 的上限」与「根槽这一次读坏」两行，用户 2026-10-04 定；代码轮第二轮判决
-//! `research/prompts/m3-root-ring-code-r2-main-verification.md` 第四节 RS3 与第六节；正推腿探针
-//! `research/prompts/m3-root-ring-code-r2-sonnet-model/tests/probe_k1_raise_to_the_ceiling_over_an_unreadable_x.rs` 那一形）。
+//! 准入抬 F 抬到上限时，挂载那一刻读不出或读得出而自证不过、之后这个进程没写过的槽当作可能是一条非空有效根，F 不越过它
+//! （D16（发布语义） 已定项 1「抬 F 的上限」与「根槽这一次读坏」两行：RS3 用户 2026-10-04 定只并读不出的，代码轮第二轮判决
+//! `research/prompts/m3-root-ring-code-r2-main-verification.md` 第四节 RS3 与第六节；同日 Q3 重定成两类都并，代码轮第三轮判决
+//! `research/prompts/m3-root-ring-code-r3-main-verification.md` 第二节 K2 与第六节，攻方 `k2-zeros` 那一形）。
+//! 上限本身取环里全部有效根（用户 2026-10-04 定 RGall，`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）。
+//! 盘上全零的根槽另算「从没写过」，不并进上限（用户 2026-10-04 定，条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）：
+//! 一槽从挂载起读回全零、而盘上住着一条非空有效根时 F 越过它，是定案知情接受的代价（没洞的那一份钉着）。
 //!
-//! 盘面（S = 4 的 12 槽环）：实例 1 覆盖写到 txg 17；三次可写挂载崩在写行那次根槽写之前（洞 18、19、20：那几槽里留着上一圈的根，出了窗口）；
-//! 再一次可写挂载写下 X = 21（写行那一版的根）再空发布两次，正常退出；第四次可写挂载之后空发布推到锚 29，准入抬 F 抬到上限。
-//! 窗口 txg > 17 里读得出的有效根是 22–29 的空发布：X 读不出时最旧的那条 22 比不出前一条、算非空，上限按读得出的根取 22；
-//! X 读得出时 21 是非空、22 起都空，上限取 21。
-//! 抬完之后判的几条不变量不含 I-7.9：池级 checker 的 I-7.9 还按全环算上限（第 9 批池级 checker 簇在改），两种盘面上今天都红。
+//! 两份盘面（S = 4 的 12 槽环）：
+//! - 有洞的那一份（攻方原型那一形）：实例 1 覆盖写到 txg 17；三次可写挂载崩在写行那次根槽写之前（洞 18、19、20：那几槽里留着实例 1 上一圈的
+//!   txg 6、7、8）；再一次可写挂载写下 X = 21（写行那一版的根）再空发布两次，正常退出；第四次可写挂载之后空发布推到 29，准入抬 F 抬到上限。
+//!   RGall 之后洞里那三条旧根是有效根、非空：第 4 新的不同状态落在 txg 6，上限 6，X 读不读得出都一样。
+//! - 没洞的那一份：实例 1 覆盖写到 X = 17，进程退出；可写挂载起 X 那一槽读坏，写行、暖机之后空发布推到 28（X 成了环里最旧的根）。
+//!   读得出的有效根是 18–28：最旧的 18 跟「没有这两棵树」比、算非空，其余都空，照读得出的根算上限是 18；X 那一槽读不出时按它能住的最大 txg 17
+//!   并进来，上限 17；读回全零时不并（从没写过），上限 18、F 越过盘上完好的 X。X 读得出时上限也是 17（X 是最旧的有效根）。
 mod common;
 mod common_root_ring_holes;
 
@@ -28,53 +33,57 @@ use singlefs_core::mount::{
     mount_writable, raise_rollback_floor_to_the_admission_ceiling, RaiseToTheAdmissionCeiling,
     ShadowLedger,
 };
+use singlefs_core::root_ring::RootRingSlot;
 use singlefs_core::root_ring::{slot_offset, target_for_publish};
 use singlefs_core::transaction::{PoolWriter, TransactionOutput};
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
 use singlefs_harness::{RecordingBlockDevice, SharedStream};
 
-/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
-const INVARIANTS_THE_RAISE_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
-/// 实例 1 覆盖写到的 txg。
-const LAST_TXG_OF_INSTANCE_ONE: u64 = 17;
-/// 崩在写行根槽之前的挂载次数（洞 18、19、20）。
+/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）、F 不高于上限（I-7.9）。
+const INVARIANTS_THE_RAISE_KEEPS_GREEN: [&str; 6] =
+    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8", "I-7.9"];
+/// 有洞的那一份：实例 1 覆盖写到的 txg。
+const LAST_TXG_OF_INSTANCE_ONE_BEFORE_THE_HOLES: u64 = 17;
+/// 有洞的那一份：崩在写行根槽之前的挂载次数（洞 18、19、20）。
 const HOLES: usize = 3;
-/// X 那一版之后、正常退出之前的空发布次数。
+/// 有洞的那一份：X 那一版之后、正常退出之前的空发布次数。
 const EMPTY_PUBLISHES_AFTER_X: u64 = 2;
-/// X：洞之后那一次可写挂载写行那一版的 txg。
-const X_TXG: u64 = 21;
-/// X 读得出时准入抬 F 的上限（探针 `PROBEREADABLE_RAISED … ceiling=CheckpointTxg(21)`）。
-const CEILING_WHEN_X_READS: CheckpointTxg = CheckpointTxg(21);
+/// 有洞的那一份：X 的 txg。
+const X_TXG_AFTER_THE_HOLES: u64 = 21;
+/// 有洞的那一份 RGall 之后的上限：洞里留着的 txg 6、7、8 与 22 那四个不同状态里第 4 新的那条。
+const CEILING_OVER_THE_HOLES: CheckpointTxg = CheckpointTxg(6);
+/// 没洞的那一份：X 的 txg（实例 1 最后一次覆盖写）。
+const X_TXG_WITHOUT_HOLES: u64 = 17;
 
-type DevicesOfTheFourthMount = Vec<(
+type DevicesOfTheRaisingMount = Vec<(
     DeviceIdentity,
     DeviceWithUnreadableRanges<RecordingBlockDevice<SparseBlockDevice>>,
 )>;
 
-/// 准入抬 F 之前的那一刻：第四次可写挂载的盘（X 那一槽按 `x_slot_reads` 读坏）、录下的流与它的起点盘面、分配器、现行那一版。
+/// 准入抬 F 之前的那一刻：抬 F 那一次可写挂载的盘（X 那一槽按 `x_slot_reads`、`x_slot_read_back` 读坏）、录下的流与它的起点盘面、分配器、现行那一版。
 struct PoolJustBeforeTheAdmissionRaise {
     holes: Vec<u64>,
     x_txg: u64,
-    image_before_the_fourth_mount: MemoryPool,
+    image_before_the_raising_mount: MemoryPool,
     stream: SharedStream,
-    devices: DevicesOfTheFourthMount,
+    devices: DevicesOfTheRaisingMount,
     allocator: PoolAllocator,
     current: TransactionOutput,
 }
 
 impl PoolJustBeforeTheAdmissionRaise {
-    /// 盘上此刻的样子：起点盘面加上第四次挂载以来录下的全部写（读故障只在读那一侧）。
+    /// 盘上此刻的样子：起点盘面加上抬 F 那一次挂载以来录下的全部写（读故障只在读那一侧）。
     fn image_now(&self) -> MemoryPool {
-        let mut image = self.image_before_the_fourth_mount.clone();
+        let mut image = self.image_before_the_raising_mount.clone();
         image.apply(&self.stream.retained_operations());
         image
     }
 }
 
-fn violations_of(image: &MemoryPool) -> Vec<String> {
+fn violations_of(image: &MemoryPool, invariants_judged: &[&str]) -> Vec<String> {
     check_pool_image(image)
         .into_iter()
-        .filter(|(invariant, _)| INVARIANTS_THE_RAISE_KEEPS_GREEN.contains(invariant))
+        .filter(|(invariant, _)| invariants_judged.contains(invariant))
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
             InvariantVerdict::Holds
@@ -84,37 +93,17 @@ fn violations_of(image: &MemoryPool) -> Vec<String> {
         .collect()
 }
 
-/// 造盘面到准入抬 F 之前（与 `raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` 同一个做法另写一份）。
-fn pool_just_before_the_admission_raise(
+/// 在 `image` 上做抬 F 的那一次可写挂载（X 那一槽从这一次挂载起按 `x_slot_reads`、`x_slot_read_back` 读坏），之后空发布推到 `anchor`。
+fn mount_reading_the_x_slot_bad_and_publish_empty_through(
+    image: MemoryPool,
+    holes: Vec<u64>,
+    x_txg: u64,
     x_slot_reads: FailingReadsOfARange,
+    x_slot_read_back: UnreadableRangeReadBack,
+    anchor: u64,
 ) -> PoolJustBeforeTheAdmissionRaise {
     let parameters = parameters_of_the_ring_hole_pools();
     let slots_per_region = parameters.geometry.root_ring_slots_per_region;
-    let ring_length = 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
-    let mut image = pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE);
-    let mut holes = Vec::new();
-    // 迭代上界：`HOLES` 次挂载，每次一个洞；跨轮携带的是崩溃之后的盘面。
-    for _ in 0..HOLES {
-        let (crashed, row_txg) = crash_a_mount_before_its_row_root(&image);
-        holes.push(row_txg);
-        image = crashed;
-    }
-    let x_txg = {
-        let mut devices = devices_of(&image);
-        let mounted = mount_writable(&parameters, &mut devices).expect("洞之后那一次可写挂载");
-        let x_txg = mounted.output.row_publish.root().checkpoint_txg.0;
-        let instance = mounted.output.instance;
-        let mut allocator = mounted.allocator.clone();
-        let mut current = mounted.current.into_file_version().expect("这一版带文件");
-        // 迭代上界：两次空发布。
-        for _ in 0..EMPTY_PUBLISHES_AFTER_X {
-            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
-            current =
-                publish_empty(&mut writer, &mut allocator, &current, instance).expect("空发布");
-        }
-        image = pool_of(&devices);
-        x_txg
-    };
     assert!(
         readable_root_at_txg(&image, x_txg).is_some(),
         "X 那一版的根落盘了"
@@ -128,15 +117,14 @@ fn pool_just_before_the_admission_raise(
             length_in_bytes: u64::from(parameters.geometry.physical_block_size),
             failing_reads: x_slot_reads,
         }],
-        UnreadableRangeReadBack::DeviceError,
+        x_slot_read_back,
     );
     let stream = SharedStream::retaining_contents();
     let mut devices = with_unreadable_ranges(recording_of(&image, &stream), &unreadable_x_slot);
-    let mounted = mount_writable(&parameters, &mut devices).expect("第四次可写挂载");
+    let mounted = mount_writable(&parameters, &mut devices).expect("抬 F 那一次可写挂载");
     let instance = mounted.output.instance;
     let mut allocator = mounted.allocator.clone();
     let mut current = mounted.current.into_file_version().expect("这一版带文件");
-    let anchor = holes[0] + ring_length - 1;
     // 迭代上界：txg 逐次加一推到锚。
     while current.root.checkpoint_txg.0 < anchor {
         let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
@@ -145,7 +133,7 @@ fn pool_just_before_the_admission_raise(
     PoolJustBeforeTheAdmissionRaise {
         holes,
         x_txg,
-        image_before_the_fourth_mount: image,
+        image_before_the_raising_mount: image,
         stream,
         devices,
         allocator,
@@ -153,6 +141,76 @@ fn pool_just_before_the_admission_raise(
     }
 }
 
+/// 有洞的那一份，造到准入抬 F 之前（与 `raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` 同一个做法另写一份）。
+fn pool_over_the_holes_just_before_the_admission_raise(
+    x_slot_reads: FailingReadsOfARange,
+    x_slot_read_back: UnreadableRangeReadBack,
+) -> PoolJustBeforeTheAdmissionRaise {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let ring_length = 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
+    let mut image = pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE_BEFORE_THE_HOLES);
+    let mut holes = Vec::new();
+    // 迭代上界：`HOLES` 次挂载，每次一个洞；跨轮携带的是崩溃之后的盘面。
+    for _ in 0..HOLES {
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
+    let anchor = holes[0] + ring_length - 1;
+    mount_reading_the_x_slot_bad_and_publish_empty_through(
+        image,
+        holes,
+        x_txg,
+        x_slot_reads,
+        x_slot_read_back,
+        anchor,
+    )
+}
+
+/// 没洞的那一份：实例 1 覆盖写到 X = 17、进程退出，抬 F 那一次可写挂载起 X 那一槽读坏，空发布推到 X + 3S − 1（X 成了环里最旧的根）。
+fn pool_without_holes_just_before_the_admission_raise(
+    x_slot_read_back: UnreadableRangeReadBack,
+) -> PoolJustBeforeTheAdmissionRaise {
+    let ring_length = 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
+    mount_reading_the_x_slot_bad_and_publish_empty_through(
+        pool_overwritten_through(X_TXG_WITHOUT_HOLES),
+        Vec::new(),
+        X_TXG_WITHOUT_HOLES,
+        FailingReadsOfARange::Every,
+        x_slot_read_back,
+        X_TXG_WITHOUT_HOLES + ring_length - 1,
+    )
+}
+
+/// 根环表里 `ring_slot` 按挂载时读坏记着的、它能住的最大 txg（没这样记着时 `None`）。
+fn highest_txg_the_table_records_for(
+    pool: &PoolJustBeforeTheAdmissionRaise,
+    ring_slot: RootRingSlot,
+) -> Option<CheckpointTxg> {
+    pool.allocator
+        .root_ring_occupancy()
+        .expect("可写挂载装了根环表")
+        .ring_slots_bad_at_mount()
+        .get(&ring_slot)
+        .map(|txgs_it_can_hold| txgs_it_can_hold.highest_txg_it_can_hold())
+}
+
 /// 抬到上限，交回抬到的上限与抬完之后现行那一版带的 F。
 fn raise_to_the_admission_ceiling(
     pool: &mut PoolJustBeforeTheAdmissionRaise,
@@ -174,56 +232,158 @@ fn raise_to_the_admission_ceiling(
     (ceiling, pool.current.root.rollback_floor)
 }
 
-/// 场景：X 那一槽第四次挂载起一直读不出（根环表按它能住的最大 txg 21 记着），推到锚 29 之后准入抬 F 抬到上限。
-/// 预期：上限不越过 X：取 21（按读得出的根算是 22），F 抬到 21，抬完那几条不变量不红。
+/// 场景：有洞的那一份，X 那一槽抬 F 那一次挂载起一直读不出（根环表按它能住的最大 txg 21 记着）。
+/// 预期：上限 6（洞里的旧根定的，X 那一槽并进来也不改），F 抬到 6，抬完那几条不变量不红。
+/// 照一圈窗口取上限时洞里的旧根不算，上限是 21（并进 X 那一槽之后）。
 #[test]
 fn admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount() {
-    let mut pool = pool_just_before_the_admission_raise(FailingReadsOfARange::Every);
+    let mut pool = pool_over_the_holes_just_before_the_admission_raise(
+        FailingReadsOfARange::Every,
+        UnreadableRangeReadBack::DeviceError,
+    );
     assert_eq!(
         (pool.holes.clone(), pool.x_txg),
-        (vec![18, 19, 20], X_TXG),
+        (vec![18, 19, 20], X_TXG_AFTER_THE_HOLES),
         "三个洞紧跟着 X"
     );
     let x_slot = target_for_publish(
-        CheckpointTxg(X_TXG),
+        CheckpointTxg(X_TXG_AFTER_THE_HOLES),
         parameters_of_the_ring_hole_pools()
             .geometry
             .root_ring_slots_per_region,
     );
     assert_eq!(
-        pool.allocator
-            .root_ring_occupancy()
-            .expect("可写挂载装了根环表")
-            .ring_slots_unreadable_at_mount()
-            .get(&x_slot),
-        Some(&CheckpointTxg(X_TXG)),
-        "前提：根环表把 X 那一槽记成挂载时读不出、能住的最大 txg 21"
+        highest_txg_the_table_records_for(&pool, x_slot),
+        Some(CheckpointTxg(X_TXG_AFTER_THE_HOLES)),
+        "前提：根环表把 X 那一槽记成挂载时读坏、能住的最大 txg 21"
     );
     assert_eq!(
         raise_to_the_admission_ceiling(&mut pool),
-        (CheckpointTxg(X_TXG), CheckpointTxg(X_TXG)),
-        "上限与抬到的 F 都不越过 X 那一槽能住的 txg 21"
+        (CEILING_OVER_THE_HOLES, CEILING_OVER_THE_HOLES),
+        "上限与抬到的 F 都是洞里旧根定的 6"
     );
     assert_eq!(
-        violations_of(&pool.image_now()),
+        violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN),
         Vec::<String>::new(),
         "抬完之后"
     );
 }
 
-/// 场景：同一盘面，X 那一槽一直读得出。
-/// 预期：上限照读出来的根取 21（X 非空、22 起都空），F 抬到 21，抬完那几条不变量不红。
+/// 场景：有洞的那一份，X 那一槽一直读得出。
+/// 预期：上限照读出来的根取 6（洞里的 txg 6、7、8 与 X 是四个不同状态），F 抬到 6，抬完那几条不变量不红。
 #[test]
 fn admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x() {
-    let mut pool = pool_just_before_the_admission_raise(FailingReadsOfARange::Never);
+    let mut pool = pool_over_the_holes_just_before_the_admission_raise(
+        FailingReadsOfARange::Never,
+        UnreadableRangeReadBack::DeviceError,
+    );
     assert_eq!(
         raise_to_the_admission_ceiling(&mut pool),
-        (CEILING_WHEN_X_READS, CEILING_WHEN_X_READS),
-        "X 读得出时上限是 21"
+        (CEILING_OVER_THE_HOLES, CEILING_OVER_THE_HOLES),
+        "X 读得出时上限也是 6"
     );
     assert_eq!(
-        violations_of(&pool.image_now()),
+        violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN),
         Vec::<String>::new(),
         "抬完之后"
     );
 }
+
+/// 场景：攻方 `k2-zeros` 那一形——有洞的那一份，X 那一槽抬 F 那一次挂载起每次读都报成功、字节全 0。
+/// 预期：读回全零的槽从没写过，根环表不记它、上限不并它（用户 2026-10-04 定）；上限照读出来的根取洞里旧根定的 6，F 抬到 6、不越过 21，
+/// 抬完那几条不变量（含 I-7.9）都不红。这一形上 F 不越过 X 靠的是 RGall 之后洞里那三条旧根，不是并进 X 那一槽：
+/// 攻方当初量到的「上限 22、I-7.9 红」是上限只取一圈窗口那一轮的数（`research/prompts/m3-root-ring-code-r3-opus-model/results/rs3z.log`）；
+/// 全零不并进上限的代价在没洞的那一份上钉着。
+#[test]
+fn admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_since_the_mount_stops_at_the_roots_left_in_the_holes(
+) {
+    let mut pool = pool_over_the_holes_just_before_the_admission_raise(
+        FailingReadsOfARange::Every,
+        UnreadableRangeReadBack::Zeros,
+    );
+    let x_slot = target_for_publish(
+        CheckpointTxg(X_TXG_AFTER_THE_HOLES),
+        parameters_of_the_ring_hole_pools()
+            .geometry
+            .root_ring_slots_per_region,
+    );
+    assert_eq!(
+        highest_txg_the_table_records_for(&pool, x_slot),
+        None,
+        "前提：X 那一槽读回全零，从没写过，根环表不记它"
+    );
+    assert_eq!(
+        raise_to_the_admission_ceiling(&mut pool),
+        (CEILING_OVER_THE_HOLES, CEILING_OVER_THE_HOLES),
+        "上限与抬到的 F 都是洞里旧根定的 6，不越过 X 的 txg 21"
+    );
+    assert_eq!(
+        violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN),
+        Vec::<String>::new(),
+        "抬完之后（含 I-7.9）"
+    );
+}
+
+/// 场景：没洞的那一份，X 那一槽抬 F 那一次挂载起一直读不出（设备报错）。
+/// 预期：上限 17（X 那一槽按它能住的最大 txg 并进来），F 抬到 17；抬完那几条不变量不红——I-7.9 不红（checker 读盘上完好的 X，按它算上限 17）。
+/// 不并 X 那一槽时上限是 18、F 越过 X（I-7.9 红）。抬 F 那一遍的回收门槛并不并 X 那一槽（RR4）在这份盘面上分不出：X 是环里最旧的根，
+/// 抬 F 那一串的空发布先把它那一槽写掉，扣到生效才放开的那几个槽放开时 X 已不在环里。
+#[test]
+fn admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount() {
+    let mut pool =
+        pool_without_holes_just_before_the_admission_raise(UnreadableRangeReadBack::DeviceError);
+    assert_eq!(
+        raise_to_the_admission_ceiling(&mut pool),
+        (
+            CheckpointTxg(X_TXG_WITHOUT_HOLES),
+            CheckpointTxg(X_TXG_WITHOUT_HOLES)
+        ),
+        "上限与抬到的 F 都停在 X 那一槽能住的 txg 17"
+    );
+    assert_eq!(
+        violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN),
+        Vec::<String>::new(),
+        "抬完之后"
+    );
+}
+
+/// 场景：没洞的那一份，X 那一槽抬 F 那一次挂载起每次读都报成功、字节全 0。
+/// 预期：读回全零的槽另算「从没写过」，根环表不记它、准入抬 F 的上限不并它（用户 2026-10-04 定）：上限照读得出的根取 18，F 抬到 18，
+/// 越过盘上完好的非空有效根 X = 17——X 在抬 F 那一串写掉它那一槽之前就出了回退候选集，「保留最近 4 个状态」在这一形上少一个。
+/// 这是定案知情接受的代价（用户 2026-10-04，`records/2026-09-30-里程碑三开工第一轮.md`）。抬完之后池级 checker 看不出它：抬 F 那一串的
+/// 第一次空发布（txg 29 = X + 3S）就写掉了 X 那一槽，镜像里 F 之前的根是 18–28，按它们算的上限是 18，那几条不变量（含 I-7.9）都不红。
+/// 照 Q3（读不出与自证不过两类都并、全零算自证不过）时上限 17、F 停在 X。
+#[test]
+fn admission_raising_the_floor_without_holes_passes_a_slot_reading_back_zeros_since_the_mount_as_the_known_cost(
+) {
+    let mut pool =
+        pool_without_holes_just_before_the_admission_raise(UnreadableRangeReadBack::Zeros);
+    let x_slot = target_for_publish(
+        CheckpointTxg(X_TXG_WITHOUT_HOLES),
+        parameters_of_the_ring_hole_pools()
+            .geometry
+            .root_ring_slots_per_region,
+    );
+    assert_eq!(
+        highest_txg_the_table_records_for(&pool, x_slot),
+        None,
+        "前提：X 那一槽读回全零，从没写过，根环表不记它"
+    );
+    assert!(
+        readable_root_at_txg(&pool.image_now(), X_TXG_WITHOUT_HOLES).is_some(),
+        "前提：抬 F 之前盘上 X 那一槽里是完好的 txg 17 的根"
+    );
+    assert_eq!(
+        raise_to_the_admission_ceiling(&mut pool),
+        (
+            CheckpointTxg(X_TXG_WITHOUT_HOLES + 1),
+            CheckpointTxg(X_TXG_WITHOUT_HOLES + 1)
+        ),
+        "定案的代价：上限与抬到的 F 都是读得出的最旧有效根 18，越过 X 的 txg 17"
+    );
+    assert_eq!(
+        violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN),
+        Vec::<String>::new(),
+        "抬完之后（含 I-7.9：X 那一槽已被抬 F 那一串写掉）"
+    );
+}
diff --git a/crates/singlefs-harness/tests/reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root.rs b/crates/singlefs-harness/tests/reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root.rs
new file mode 100644
index 0000000..517327a
--- /dev/null
+++ b/crates/singlefs-harness/tests/reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root.rs
@@ -0,0 +1,160 @@
+//! 回收门槛取「环里最旧有效根」（D16（发布语义） 已定项 1「可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」；用户 2026-10-04 定 RGall，
+//! `research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1：回收门槛与原来同取一圈窗口的另外五处——checker 的候选过滤、
+//! checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛、影子账的候选过滤、回退候选集、抬 F 的上限——一起退回整个环）。
+//!
+//! 盘面（S = 4 的 12 槽环，攻方腿 `research/prompts/m3-reclaim-window-r1-opus-output.md` 那一形；原型
+//! `research/prompts/m3-reclaim-window-r1-opus-model/probe/main.rs` 的 `crash_a_mount_before_its_row_root` 在 `common_root_ring_holes` 里另写了一份）：
+//! 实例 1 覆盖写到 txg 24；连着几次可写挂载都崩在写行那次根槽写之前（K1 那一类洞：写行那次的单元与记录已持久、根槽没持久，每崩一次 txg 跳一个，
+//! 洞那一槽里留着实例 1 上一圈的根）；再一次可写挂载做完（写行、暖机落满）。
+//!
+//! - 连崩 11 次：写行那次的根一记，进程内的锚跳过 11 个没有根的 txg，实例 1 留在洞里的根（txg 13–23）全落到一圈窗口外。照一圈窗口取门槛时
+//!   它们引用、已释放的单元当场回收（攻方量到挂载之后每盘已分配 136 → 20）；取整个环时一个都不回收。
+//! - 连崩 1 次、只改分配器那一臂（攻方 `arm-RGalloc.diff`）：分配器取整个环、checker 的候选过滤与丁-defer 门槛仍取窗口，
+//!   挂载之后池级 checker 判 I-3.1 红（攻方 S = 4 每段 112 / 120）；六处一起改之后不红。
+mod common_root_ring_holes;
+
+use common_root_ring_holes::{
+    crash_a_mount_before_its_row_root, devices_of, parameters_of_the_ring_hole_pools, pool_of,
+    pool_overwritten_through, SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::mount::{mount_writable, Mounted};
+use singlefs_core::recovery::{
+    allocation_records_under_root, choose_system_configuration, instance_table_chain_of_root,
+    readable_roots_with_ring_slots, root_is_abandoned_by_the_instance_table,
+};
+use singlefs_core::root_record::RootRecord;
+use singlefs_harness::memory_pool::MemoryPool;
+
+/// 实例 1 覆盖写到的 txg（攻方 `C = 24`）。
+const LAST_TXG_OF_INSTANCE_ONE: u64 = 24;
+/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
+const INVARIANTS_THE_MOUNT_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
+
+/// 实例 1 覆盖写到 txg 24，再连着 `crashes` 次可写挂载崩在写行那次根槽写之前。
+fn pool_after_crashes_before_the_row_root(crashes: usize) -> MemoryPool {
+    let mut image = pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE);
+    // 迭代上界：`crashes` 次挂载，每次一个洞；跨轮携带的是崩溃之后的盘面。
+    for _ in 0..crashes {
+        let (crashed, _row_txg) = crash_a_mount_before_its_row_root(&image);
+        image = crashed;
+    }
+    image
+}
+
+/// 盘面上读得出、按 `newest_root` 指着的实例表仍然有效的每一条根。
+fn valid_roots_of_the_ring(image: &MemoryPool, newest_root: &RootRecord) -> Vec<RootRecord> {
+    let system_configuration = choose_system_configuration(image).expect("系统配置");
+    let immutable = &system_configuration.immutable;
+    let table = instance_table_chain_of_root(image, newest_root)
+        .expect("现行那一版的实例表")
+        .records;
+    readable_roots_with_ring_slots(
+        image,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    )
+    .into_iter()
+    .map(|(_, root)| root)
+    .filter(|root| !root_is_abandoned_by_the_instance_table(root, &table))
+    .collect()
+}
+
+/// 环里每条有效根的账里仍分配、而分配器上空着（可再分配）的槽：(根的 txg, 盘, 槽)。
+fn slots_of_valid_ring_roots_left_free(
+    image: &MemoryPool,
+    allocator: &PoolAllocator,
+    roots: &[RootRecord],
+) -> Vec<(u64, u32, u64)> {
+    let mut left_free = Vec::new();
+    for root in roots {
+        let records = allocation_records_under_root(image, root).expect("环里有效根的账读得出");
+        for record in records.iter().filter(|record| !record.is_released) {
+            let device_map = allocator
+                .devices
+                .iter()
+                .find(|device_map| device_map.device == record.device)
+                .expect("账里的盘在分配器里");
+            for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
+                if device_map.is_free(singlefs_core::address::SlotNumber(slot)) {
+                    left_free.push((root.checkpoint_txg.0, record.device.0, slot));
+                }
+            }
+        }
+    }
+    left_free
+}
+
+fn violations_of(image: &MemoryPool) -> Vec<String> {
+    check_pool_image(image)
+        .into_iter()
+        .filter(|(invariant, _)| INVARIANTS_THE_MOUNT_KEEPS_GREEN.contains(invariant))
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
+        })
+        .collect()
+}
+
+/// 场景：连崩 11 次之后可写挂载做完。
+/// 预期：环里实例 1 留在洞里的根有落在一圈窗口外的（前提），挂载之后的分配器上，环里每一条有效根的账里仍分配的槽一个都不空闲；
+/// 挂载之后盘上那几条不变量不红。改之前（回收门槛取一圈窗口）窗口外那几条根引用的单元当场回收、空了出来。
+#[test]
+fn eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated(
+) {
+    let image = pool_after_crashes_before_the_row_root(11);
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices = devices_of(&image);
+    let Mounted {
+        allocator, current, ..
+    } = mount_writable(&parameters, &mut devices).expect("连崩之后那一次可写挂载");
+    let after = pool_of(&devices);
+    let roots = valid_roots_of_the_ring(&after, current.root());
+    let newest_txg = roots
+        .iter()
+        .map(|root| root.checkpoint_txg.0)
+        .max()
+        .expect("环里有根");
+    let ring_length = 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
+    let roots_outside_the_window: Vec<u64> = roots
+        .iter()
+        .map(|root| root.checkpoint_txg.0)
+        .filter(|txg| txg + ring_length <= newest_txg)
+        .collect();
+    assert!(
+        !roots_outside_the_window.is_empty(),
+        "前提：洞里留着的实例 1 的根有落在一圈窗口外的（最新根 txg {newest_txg}）"
+    );
+    assert_eq!(
+        slots_of_valid_ring_roots_left_free(&after, &allocator, &roots),
+        Vec::<(u64, u32, u64)>::new(),
+        "环里有效根（窗口外的 {roots_outside_the_window:?} 也算）引用的单元一个都不回收；每盘已分配 {:?}",
+        allocator
+            .devices
+            .iter()
+            .map(|device_map| device_map.allocated_slots())
+            .collect::<Vec<u64>>()
+    );
+    assert_eq!(violations_of(&after), Vec::<String>::new(), "挂载之后");
+}
+
+/// 场景：连崩 1 次之后可写挂载做完（攻方 RGalloc 臂 I-3.1 误红的那一形：`results/b1-RGalloc-pre0.log` 的 `k=1 j=0`）。
+/// 预期：挂载之后池级 checker 判那几条不变量不红——分配器的门槛与 checker 的候选过滤都取整个环，窗口外那几条根照当候选走、它们引用的单元照算已分配。
+#[test]
+fn one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring(
+) {
+    let image = pool_after_crashes_before_the_row_root(1);
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices = devices_of(&image);
+    mount_writable(&parameters, &mut devices).expect("崩过一次之后那一次可写挂载");
+    assert_eq!(
+        violations_of(&pool_of(&devices)),
+        Vec::<String>::new(),
+        "挂载之后"
+    );
+}
diff --git a/crates/singlefs-harness/tests/rollback_target_outside_the_one_revolution_window_is_not_a_candidate.rs b/crates/singlefs-harness/tests/rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs
similarity index 62%
rename from crates/singlefs-harness/tests/rollback_target_outside_the_one_revolution_window_is_not_a_candidate.rs
rename to crates/singlefs-harness/tests/rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs
index 5d4bd43..5fc7f89 100644
--- a/crates/singlefs-harness/tests/rollback_target_outside_the_one_revolution_window_is_not_a_candidate.rs
+++ b/crates/singlefs-harness/tests/rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs
@@ -1,9 +1,11 @@
-//! 回退候选集只收 txg 落在一圈窗口里的根（D16（发布语义） 已定项 1「回退候选集」：按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ txg 落在一圈窗口里；
-//! D23（journal 的角色与格式） 已定项 14 的挂着时回退取同一个集合；用户 2026-10-03 定照 7-2 报告第四节做）。
+//! 回退候选集取环里全部按现行那一版的实例表判仍然有效、txg ≥ F_生效、带文件的根（D16（发布语义） 已定项 1「回退候选集」；D23（journal 的角色与格式）
+//! 已定项 14 的挂着时回退取同一个集合）。原来另有「txg 落在一圈窗口里」一格（用户 2026-10-03 定照 7-2 报告第四节做），用户 2026-10-04 定 RGall
+//! 与回收门槛一起退回整个环（`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）：回收门槛不再越过窗口外的有效根，
+//! 它引用的单元都还在，它就是候选。
 //!
 //! 盘面同 `one_revolution_window_after_a_hole_in_the_root_ring.rs`：实例 1 在 24 槽的根环上发到 txg 30，txg 31 那次发布崩在「记录已持久、根槽没持久」，
 //! 那一槽里留着上一圈的 txg 7 的根；可写挂载（实例 2）恢复由记录施加出 txg 31，写行 32、暖机 33、34。txg 7 的根读得出、按实例表有效、
-//! txg ≥ F_生效（F 没抬过，是 0）、带文件，只有一样不满足：现行那一版 txg 34 的窗口是 txg > 10，txg 7 在窗口外——窗口已许可它引用的单元被复用。
+//! txg ≥ F_生效（F 没抬过，是 0）、带文件；现行那一版 txg 34 的一圈窗口是 txg > 10，txg 7 在窗口外。
 mod common;
 
 use common::{
@@ -11,12 +13,12 @@ use common::{
     overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
     system_configuration_slots_of_every_device, write_back_system_configuration_slots,
 };
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
 use singlefs_core::block_device::BlockDevice;
-use singlefs_core::mount::{
-    mount_writable, roll_back_by_a_forward_publish, RollbackCandidateExclusion, RollbackError,
-    RollbackTarget,
-};
+use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
+use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
 use singlefs_core::root_ring::{slot_offset, target_for_publish};
 use singlefs_core::transaction::PoolVersion;
 use singlefs_harness::SharedStream;
@@ -27,17 +29,20 @@ const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 31;
 const TXG_OF_THE_STALE_ROOT_IN_THE_HOLE: u64 = 7;
 /// 一次覆盖写的内容长度：一个数据单元装得下。
 const OVERWRITE_BYTES: usize = 3000;
+/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
+const INVARIANTS_THE_ROLLBACK_KEEPS_GREEN: [&str; 5] =
+    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
 
 fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
     vec![u8::try_from(checkpoint_txg % 251).expect("小于 256"); OVERWRITE_BYTES]
 }
 
 /// 场景：上面那份盘面上挂着时回退到环里那条 txg 7 的根（实例 1）。
-/// 预期：在任何写之前拒，报 `TargetNotACandidate { exclusion: OutsideTheOneRevolutionWindow }`；盘上逐字节不变（系统配置槽、根环里自证过的根、录制流步数）。
-/// 改之前候选集不看窗口，txg 7 那条根被当成候选往下走。
+/// 预期：回退做成，向前发一条 txg 35 的根；冷启动择到它、读回 txg 7 那一版的内容；回退之后盘上那几条不变量不红。
+/// 改之前候选集判一圈窗口，在任何写之前拒（`TargetNotACandidate { exclusion: OutsideTheOneRevolutionWindow }`）。
 #[test]
-fn rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_is_refused_before_any_write(
-) {
+fn rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_reads_back_that_version()
+{
     let mut pool = build_pool("rollback-target-outside-the-one-revolution-window");
     let slots_per_region = parameters().geometry.root_ring_slots_per_region;
     // 迭代上界：txg 4 到 30，27 次覆盖写。
@@ -108,9 +113,9 @@ fn rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_is_re
                     CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
                 )
         }),
-        "txg 7 的根还在环里、读得出"
+        "前提：txg 7 的根还在环里、读得出"
     );
-    let refusal = roll_back_by_a_forward_publish(
+    let rolled_back = roll_back_by_a_forward_publish(
         &parameters(),
         &mut devices,
         &mut mounted.allocator,
@@ -119,21 +124,34 @@ fn rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_is_re
             instance: InstanceGeneration(1),
             checkpoint_txg: CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
         },
-    )
-    .expect_err("txg 7 在现行那一版的一圈窗口外，不是候选");
+    );
     assert!(
-        matches!(
-            refusal,
-            RollbackError::TargetNotACandidate {
-                exclusion: RollbackCandidateExclusion::OutsideTheOneRevolutionWindow,
-                ..
-            }
-        ),
-        "报落在一圈窗口外：{refusal:?}"
+        rolled_back.is_ok(),
+        "txg 7 在环里、有效、带文件：是候选，回退做成：{rolled_back:?}"
     );
     assert_eq!(
-        disk_snapshot(&memory_pool_of_sparse_devices(&devices), &stream),
-        before,
-        "在任何写之前拒：盘上逐字节不变"
+        current.root.checkpoint_txg,
+        CheckpointTxg(35),
+        "回退向前发一条 txg 35 的根"
+    );
+    let image = memory_pool_of_sparse_devices(&devices);
+    let violations: Vec<String> = check_pool_image(&image)
+        .into_iter()
+        .filter(|(invariant, _)| INVARIANTS_THE_ROLLBACK_KEEPS_GREEN.contains(invariant))
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
+        })
+        .collect();
+    assert_eq!(violations, Vec::<String>::new(), "回退之后");
+    assert_eq!(
+        recover(&image, JournalPolicy::Consult).outcome,
+        RecoveryOutcome::FileRead {
+            root: (InstanceGeneration(2), CheckpointTxg(35)),
+            content: content_of_txg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
+        },
+        "冷启动择 txg 35 那条根、读回 txg 7 那一版的内容"
     );
 }
diff --git a/crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs b/crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs
index 13d43d0..d8f5600 100644
--- a/crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs
+++ b/crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs
@@ -1,18 +1,20 @@
-//! 进程内记根时重读挂载那一刻读不出的根环槽（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」那一行的 E2 句与「根槽这一次读坏」那一行）：
+//! 进程内记根时重读挂载那一刻读坏的根环槽（D16（发布语义） 已定项 1「环里最旧有效根」那一行的 E2 句与「根槽这一次读坏」那一行）：
 //! 重读那一次的几种结局各记成什么、判「按实例表有效」用哪张表、产品路径上记根的几处各自走到重读。
 //!
-//! - 读得出而自证不过，与仍读不出同一处置，按根环表记的最大 txg 算（用户 2026-10-04 定改法 A，代码轮第二轮判决
+//! 读坏的槽在回收门槛里钉在它可能住过的最旧 txg，重读读出有效根或确认为空之前门槛不越过它；重读时机照 E2（根环表里这一项成了最小的那个时）；
+//! 盘上全零的根槽另算「从没写过」（用户 2026-10-04 定，条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）。钉在最旧 txg 之后
+//! 那一槽挂载那一刻起多半就是根环表里最小的那个，挂载之后第一次记根就重读。
+//!
+//! - 读得出而自证不过，与仍读不出同一处置，照旧钉着（用户 2026-10-04 定改法 A，代码轮第二轮判决
 //!   `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1；攻方 `research/prompts/m3-root-ring-code-r2-opus-model/proto/src/bin/e2_reread_flip.rs`
-//!   那一形：一条有效根所在的槽挂载那一刻读错、它成了窗口里最小的那一刻那一次重读读坏，改之前回收门槛越过它、I-3.1 记账少）；
-//!   代价那一形（同一目录 `front_hole_eio.rs`：从没写过的槽加挂载那一刻读错）判成什么就钉什么。
+//!   那一形：一条有效根所在的槽挂载那一刻读错、重读那一次读坏，改之前回收门槛越过它、I-3.1 记账少）。
+//! - 读回全零：从没写过，确认为空、拿掉（同一目录 `front_hole_eio.rs` 那一形：从没写过的槽加挂载那一刻读错，改法 A 下按记的 txg 多记账的那一版不再有）。
 //! - 读得出、自证过、按实例表有效的照读出来的 txg 算，那一槽从此算「这个进程知道住着根的槽」（用户 2026-10-04 定）；
 //!   读得出、自证过而按实例表被抛弃的当没有根；判有效用的表写行之后是写行那一张（同一份判决第二节 RS2，正推腿探针 p1、p2）。
-//! - 记根的产品路径：带单元的发布（`transaction::publish_admitted`，已有变异行管它）、树表 0 条那一版上的暖机零单元发布
-//!   （`mount::publish_empty_after`）走到重读（探针 q1）。另两处没有会红的用例：第一次可写挂载的零单元写行发布
-//!   （`mount::establish_instance` 里 `rows_written` 为空那一支）走不到重读——那一刻根环里只有 mkfs 写的 txg 0，挂载时读不出的槽只可能是它，
-//!   而它读不出时择不到根、挂载不成；树表 0 条那一版上要写行的写行发布（`transaction::prepare_the_row_publish_on_a_version_without_file`，探针 q3）
-//!   重读的那一槽恰是紧跟着的暖机要盖掉的那一槽（写行 txg + 1），重读改的根环表与回收在暖机那一次被盖成同一个样子，那一版又没有记账树，
-//!   挂载交回之后看不出它重读没重读（交回报告里列着）。
+//! - 记根的产品路径：带单元的发布（`transaction::publish_admitted`，已有变异行管它）、树表 0 条那一版上要写行的写行发布
+//!   （`transaction::prepare_the_row_publish_on_a_version_without_file`）走到重读。第一次可写挂载的零单元写行发布
+//!   （`mount::establish_instance` 里 `rows_written` 为空那一支）走不到重读——那一刻根环里只有 mkfs 写的 txg 0，挂载时读坏的槽只可能是它，
+//!   而它读不出时择不到根、挂载不成。
 mod common;
 mod common_root_ring_holes;
 
@@ -36,7 +38,7 @@ use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
 };
-use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::allocator::{PoolAllocator, TxgsARingSlotBadAtMountCanHold};
 use singlefs_core::block_device::{
     BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
 };
@@ -57,11 +59,11 @@ const INVARIANTS_THE_REREAD_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1"
 /// 攻方那一形：实例 1 覆盖写到 txg 20，有效根 h = 18 所在的槽（区域 0、槽 2）挂载那一刻读错。
 const LAST_TXG_OF_INSTANCE_ONE: u64 = 20;
 const TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT: u64 = 18;
-/// h + 3S − 1：那一槽成了窗口里最小的那个、记这条根时重读它的那一版（S = 4）。
-const TXG_AT_WHICH_THE_SLOT_IS_REREAD: u64 = 29;
+/// h 那一槽（区域 0、槽 2）可能住过的最旧 txg：2 × 3 + 0（S = 4）。
+const OLDEST_TXG_THE_SLOT_OF_THE_VALID_ROOT_CAN_HAVE_HELD: u64 = 6;
 /// 前洞那一形：实例 2 的 txg 9 那次覆盖写崩在根槽写之前（环还没转过来，那一槽从没写过）。
 const TXG_OF_THE_FRONT_HOLE: u64 = 9;
-/// 前洞那一形里那一槽成了窗口里最小的那个的那一版：9 + 3S − 1。
+/// 前洞那一形里那一槽成了根环表里最小的那个的那一版：9 + 3S − 1（txg 8 那一槽被盖掉）。
 const TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD: u64 = 20;
 
 /// 被抛弃根那一形判的几条：同上去掉 I-7.4。被抛弃的根所在的槽挂载那一刻读不出，影子账罩不到它引用的单元（挂载那一刻就读不到那条根，
@@ -354,12 +356,12 @@ impl SessionOnARingHolePool {
         .expect("覆盖写");
     }
 
-    /// 根环表里这一槽按挂载时读不出记着的最大 txg（不再这样记着时 `None`）。
-    fn txg_recorded_as_unreadable_at_mount(&self) -> Option<CheckpointTxg> {
+    /// 根环表里这一槽按挂载时读坏记着的 txg（不再这样记着时 `None`）。
+    fn txgs_recorded_as_bad_at_mount(&self) -> Option<TxgsARingSlotBadAtMountCanHold> {
         self.allocator
             .root_ring_occupancy()
             .expect("可写挂载装了根环表")
-            .ring_slots_unreadable_at_mount()
+            .ring_slots_bad_at_mount()
             .get(&self.ring_slot)
             .copied()
     }
@@ -373,98 +375,103 @@ impl SessionOnARingHolePool {
     }
 }
 
-/// 攻方那一形推到重读的前一版：实例 1 覆盖写到 20；实例 2 可写挂载那一刻读 h = 18 那一槽报读错；之后覆盖写到 28，
-/// 每一版之后那几条不变量都不红。
-fn session_one_publish_before_the_reread() -> SessionOnARingHolePool {
-    let mut session = SessionOnARingHolePool::mounted_with_the_slot_of_txg_unreadable_at_mount(
+/// 攻方那一形挂载之后、重读之前：实例 1 覆盖写到 20；实例 2 可写挂载那一刻读 h = 18 那一槽报读错（挂载里写行、暖机记根时的重读也读错）。
+/// 那一槽钉在它可能住过的最旧 txg 6（能住的最大 txg 18），是根环表里最小的那个：挂载之后第一次记根就重读。
+fn session_right_after_the_mount_before_the_reread() -> SessionOnARingHolePool {
+    let session = SessionOnARingHolePool::mounted_with_the_slot_of_txg_unreadable_at_mount(
         pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE),
         TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT,
     );
     assert_eq!(
-        session.txg_recorded_as_unreadable_at_mount(),
-        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
-        "前提：根环表把那一槽记成挂载时读不出、能住的最大 txg 18"
-    );
-    // 迭代上界：覆盖写到 txg 28；跨轮携带的是现行那一版。
-    while session.current.root.checkpoint_txg.0 < TXG_AT_WHICH_THE_SLOT_IS_REREAD - 1 {
-        session.overwrite_once();
-        assert_eq!(
-            violations_of(&session.image_now()),
-            Vec::<String>::new(),
-            "txg {} 之后",
-            session.current.root.checkpoint_txg.0
-        );
-    }
-    assert_eq!(
-        session.txg_recorded_as_unreadable_at_mount(),
-        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
-        "前提：txg 28 时窗口里还有比 18 旧的 17，那一槽没重读"
+        session.txgs_recorded_as_bad_at_mount(),
+        Some(TxgsARingSlotBadAtMountCanHold::new(
+            CheckpointTxg(OLDEST_TXG_THE_SLOT_OF_THE_VALID_ROOT_CAN_HAVE_HELD),
+            CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)
+        )),
+        "前提：根环表把那一槽记成挂载时读坏：钉在 6，能住的最大 txg 18"
     );
     session
 }
 
-/// 场景（攻方 `e2_reread_flip 4 20 2 flip`，`FLIP_ONLY_AT_ANCHOR=1`）：发 txg 29 那一次覆盖写时只有一次读把那一槽读坏（一个字节取反，盘上不变），
-/// 那一次就是记这条根之前的重读。
-/// 预期：读得出而自证不过与仍读不出同一处置，那一槽照旧按 18 记着，回收门槛停在 18；txg 29 那一版 I-3.1 成立（改之前记账少 12 个槽）。
+/// 场景（攻方 `e2_reread_flip 4 20 2 flip`）：挂载之后第一次覆盖写时只有一次读把那一槽读坏（一个字节取反，盘上不变），
+/// 那一次就是记这条根之前的重读；再覆盖写一次（照读）。
+/// 预期：读得出而自证不过与仍读不出同一处置，那一槽照旧钉在 6，回收门槛不越过盘上 txg 18 那条有效根：被引用单元那几条不红，
+/// 只有 I-3.1 可能红在记账多的一边（钉着的 6 比盘上环里最旧的有效根旧，安全方向，定案的代价）；
+/// 下一次记根再重读、读出 txg 18 的根，那几条不变量都不红。
 #[test]
-fn a_valid_root_read_bad_once_on_the_reread_is_counted_at_the_highest_txg_its_slot_can_hold() {
-    let mut session = session_one_publish_before_the_reread();
+fn a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held(
+) {
+    let mut session = session_right_after_the_mount_before_the_reread();
     session
         .fault
         .set_reads(ReadsOfTheRootRingSlot::OneByteFlippedOnTheNextReadOnly);
     session.overwrite_once();
-    assert_eq!(
-        session.current.root.checkpoint_txg,
-        CheckpointTxg(TXG_AT_WHICH_THE_SLOT_IS_REREAD)
-    );
     assert_eq!(
         session.fault.flipped_reads(),
         1,
-        "发 txg 29 那一次读那一槽、读坏了一次"
+        "挂载之后第一次覆盖写读那一槽、读坏了一次"
     );
     assert_eq!(
-        session.txg_recorded_as_unreadable_at_mount(),
-        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
-        "重读读到自证不过：照旧按能住的最大 txg 18 记着"
+        session.txgs_recorded_as_bad_at_mount(),
+        Some(TxgsARingSlotBadAtMountCanHold::new(
+            CheckpointTxg(OLDEST_TXG_THE_SLOT_OF_THE_VALID_ROOT_CAN_HAVE_HELD),
+            CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)
+        )),
+        "重读读到自证不过：照旧钉在 6"
+    );
+    let violated_after_the_bad_reread: Vec<String> = violations_of(&session.image_now())
+        .iter()
+        .map(|violation| violation.split('：').next().expect("不变量名").to_string())
+        .collect();
+    assert!(
+        violated_after_the_bad_reread
+            .iter()
+            .all(|invariant| invariant == "I-3.1"),
+        "重读读坏那一版：门槛没越过盘上 txg 18 那条有效根，被引用单元那几条不红：{violated_after_the_bad_reread:?}"
+    );
+    session.overwrite_once();
+    assert_eq!(
+        (
+            session.txgs_recorded_as_bad_at_mount(),
+            session.slot_is_known_to_hold_a_root()
+        ),
+        (None, true),
+        "下一次记根再重读、读出 txg 18 的根"
     );
     assert_eq!(
         violations_of(&session.image_now()),
         Vec::<String>::new(),
-        "txg 29 那一版：回收门槛没越过盘上 txg 18 那条有效根"
+        "读出 txg 18 之后那一版"
     );
 }
 
-/// 场景：同一盘面，发 txg 29 那一次重读读得出（读故障只在挂载那一刻）。
-/// 预期：读出 txg 18 的根、按实例表有效、在窗口里：那一槽换成「重读读出的有效根 18」，算这个进程知道住着根的槽；txg 29 那一版 I-3.1 成立。
+/// 场景：同一盘面，挂载之后第一次覆盖写那一次重读读得出（读故障只在挂载那一刻）。
+/// 预期：读出 txg 18 的根、按实例表有效：那一槽换成「重读读出的有效根 18」，算这个进程知道住着根的槽；那一版那几条不变量不红。
 #[test]
-fn a_valid_root_read_on_the_reread_inside_the_window_holds_the_reclaim_threshold_at_its_txg() {
-    let mut session = session_one_publish_before_the_reread();
+fn a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg() {
+    let mut session = session_right_after_the_mount_before_the_reread();
     session.overwrite_once();
-    assert_eq!(
-        session.current.root.checkpoint_txg,
-        CheckpointTxg(TXG_AT_WHICH_THE_SLOT_IS_REREAD)
-    );
     assert_eq!(
         (
-            session.txg_recorded_as_unreadable_at_mount(),
+            session.txgs_recorded_as_bad_at_mount(),
             session.slot_is_known_to_hold_a_root()
         ),
         (None, true),
-        "重读读出有效根：不再按挂载时读不出记，那一槽算这个进程知道住着根的槽"
+        "重读读出有效根：不再按挂载时读坏记，那一槽算这个进程知道住着根的槽"
     );
     assert_eq!(
         violations_of(&session.image_now()),
         Vec::<String>::new(),
-        "txg 29 那一版：门槛照读出来的 18 算"
+        "门槛照读出来的 18 算"
     );
 }
 
-/// 场景：同一盘面，发 txg 29 那一次重读读出 txg 18 的有效根；之后那一槽每次读都报设备错，接着准入抬 F 抬到上限。
+/// 场景：同一盘面，挂载之后第一次覆盖写那一次重读读出 txg 18 的有效根；之后那一槽每次读都报设备错，接着准入抬 F 抬到上限。
 /// 预期：算上限那一遍那一槽读不出、它是这个进程知道住着根的槽：重读一次仍读不出，在任何写之前拒
 /// （`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`），盘上逐字节不变、录制流不多一步。
 #[test]
 fn a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write() {
-    let mut session = session_one_publish_before_the_reread();
+    let mut session = session_right_after_the_mount_before_the_reread();
     session.overwrite_once();
     assert!(
         session.slot_is_known_to_hold_a_root(),
@@ -498,12 +505,12 @@ fn a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_be
     assert!(session.image_now() == image_before, "盘上逐字节不变");
 }
 
-/// 场景（攻方 `front_hole_eio 4 9 mount`，改法 A 的代价那一格）：实例 1 写到 3，实例 2 覆盖写到 8，txg 9 那次覆盖写崩在根槽写之前
-/// （记录已持久；环还没转过来，那一槽从没写过、读出来自证不过）；实例 3 可写挂载那一刻读那一槽报读错，之后照读；覆盖写到 9 + 2 × 3S。
-/// 钉住今天判成的样子：只有 txg 20 那一版（那一槽成了窗口里最小的那个、重读读到自证不过、照旧按 9 算）I-3.1 红——记账比遍历多，安全方向；
-/// 其余每一版那几条不变量都不红，txg 21 起那一槽出了窗口。
+/// 场景（攻方 `front_hole_eio 4 9 mount`）：实例 1 写到 3，实例 2 覆盖写到 8，txg 9 那次覆盖写崩在根槽写之前
+/// （记录已持久；环还没转过来，那一槽从没写过、盘上全零）；实例 3 可写挂载那一刻读那一槽报读错，之后照读；覆盖写到 9 + 2 × 3S。
+/// 预期：那一槽钉在它可能住过的最旧 txg 9；环里还有比 9 旧的有效根时不重读，txg 20 那一版它成了根环表里最小的那个、重读读回全零：
+/// 从没写过，确认为空、拿掉。每一版那几条不变量都不红（改法 A 下这一形重读那一版按 9 多记账、I-3.1 红，全零另算从没写过之后没有了）。
 #[test]
-fn a_front_hole_unreadable_at_mount_counts_more_allocated_only_on_the_version_that_rereads_it() {
+fn a_front_hole_unreadable_at_mount_reread_as_all_zero_is_taken_as_never_written() {
     let parameters = parameters_of_the_ring_hole_pools();
     let image_with_the_front_hole = {
         let mut devices: RingHoleDevices = devices_of(&pool_overwritten_through(3));
@@ -565,13 +572,15 @@ fn a_front_hole_unreadable_at_mount_counts_more_allocated_only_on_the_version_th
         TXG_OF_THE_FRONT_HOLE,
     );
     assert_eq!(
-        session.txg_recorded_as_unreadable_at_mount(),
-        Some(CheckpointTxg(TXG_OF_THE_FRONT_HOLE)),
-        "前提：根环表把那一槽记成挂载时读不出、能住的最大 txg 9"
+        session.txgs_recorded_as_bad_at_mount(),
+        Some(TxgsARingSlotBadAtMountCanHold::new(
+            CheckpointTxg(TXG_OF_THE_FRONT_HOLE),
+            CheckpointTxg(TXG_OF_THE_FRONT_HOLE)
+        )),
+        "前提：根环表把那一槽记成挂载时读坏，钉在 9（能住的最大 txg 也是 9）"
     );
     let ring_length = 3 * parameters.geometry.root_ring_slots_per_region.count();
     let mut versions_with_a_violation = BTreeSet::new();
-    let mut invariants_violated_on_the_reread = Vec::new();
     // 迭代上界：覆盖写到 9 + 2 × 3S；跨轮携带的是现行那一版与红过的版本。
     while session.current.root.checkpoint_txg.0 < TXG_OF_THE_FRONT_HOLE + 2 * ring_length {
         session.overwrite_once();
@@ -579,33 +588,34 @@ fn a_front_hole_unreadable_at_mount_counts_more_allocated_only_on_the_version_th
         if !violations.is_empty() {
             versions_with_a_violation.insert(session.current.root.checkpoint_txg.0);
         }
-        if session.current.root.checkpoint_txg.0 == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD {
-            invariants_violated_on_the_reread = violations
-                .iter()
-                .map(|violation| violation.split('：').next().expect("不变量名").to_string())
-                .collect();
+        let txg = session.current.root.checkpoint_txg.0;
+        if txg == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD - 1 {
+            assert!(
+                session.txgs_recorded_as_bad_at_mount().is_some(),
+                "txg {txg}：环里还有比 9 旧的 txg 8，那一槽没重读、仍钉着"
+            );
+        }
+        if txg == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD {
             assert_eq!(
-                session.txg_recorded_as_unreadable_at_mount(),
-                Some(CheckpointTxg(TXG_OF_THE_FRONT_HOLE)),
-                "txg 20：重读读到自证不过（从没写过），照旧按 9 记着"
+                (
+                    session.txgs_recorded_as_bad_at_mount(),
+                    session.slot_is_known_to_hold_a_root()
+                ),
+                (None, false),
+                "txg 20：重读读回全零，从没写过，拿掉、不算知道住着根的槽"
             );
         }
     }
     assert_eq!(
         versions_with_a_violation,
-        BTreeSet::from([TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD]),
-        "只有重读那一版红"
-    );
-    assert_eq!(
-        invariants_violated_on_the_reread,
-        vec!["I-3.1".to_string()],
-        "红的是 I-3.1（记账比遍历多一截：门槛按 9 算，盘上窗口里最旧的有效根是 10）"
+        BTreeSet::new(),
+        "每一版那几条不变量都不红"
     );
 }
 
 /// 场景：实例 1 覆盖写到 txg 5（最新那条根 a = 5）；崩溃恢复抛弃 a（它的根槽与数据单元一时读不出，择根落到 4，实例 2 写行 6、暖机 7），
 /// 实例 2 再覆盖写两次（8、9）；实例 3 可写挂载那一刻读 a 那一槽报读错，之后照读；覆盖写到 a + 3S − 1 = 28（S = 8 的 24 槽环）。
-/// 预期：记 txg 28 那条根之前 a 那一槽成了窗口里最小的那个，重读读出 txg 5 的根，按写行那一张实例表（实例 1 停在 4）被抛弃：当没有根，
+/// 预期：记 txg 28 那条根之前 a 那一槽成了根环表里最小的那个（txg 0–4 已被盖掉），重读读出 txg 5 的根，按写行那一张实例表（实例 1 停在 4）被抛弃：当没有根，
 /// 那一槽从根环表拿掉、不算这个进程知道住着根的槽；txg 28 那一版那几条不变量（不含 I-7.4，见 [`INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT`]）不红。
 #[test]
 fn an_abandoned_root_read_on_the_reread_is_taken_as_no_root() {
@@ -656,10 +666,7 @@ fn an_abandoned_root_read_on_the_reread_is_taken_as_no_root() {
             .root_ring_occupancy()
             .expect("可写挂载装了根环表");
         (
-            occupancy
-                .ring_slots_unreadable_at_mount()
-                .get(&ring_slot)
-                .copied(),
+            occupancy.ring_slots_bad_at_mount().get(&ring_slot).copied(),
             occupancy
                 .ring_slots_known_to_hold_a_root()
                 .contains(&ring_slot),
@@ -667,8 +674,14 @@ fn an_abandoned_root_read_on_the_reread_is_taken_as_no_root() {
     };
     assert_eq!(
         occupancy_of(&allocator),
-        (Some(CheckpointTxg(txg_of_the_abandoned_root)), false),
-        "前提：根环表把 a 那一槽记成挂载时读不出、能住的最大 txg 5"
+        (
+            Some(TxgsARingSlotBadAtMountCanHold::new(
+                CheckpointTxg(txg_of_the_abandoned_root),
+                CheckpointTxg(txg_of_the_abandoned_root)
+            )),
+            false
+        ),
+        "前提：根环表把 a 那一槽记成挂载时读坏，钉在 5（环还没转过来，能住的最大 txg 也是 5）"
     );
     let mut current = mounted.current.into_file_version().expect("这一版带文件");
     let ring_length = 3 * parameters.geometry.root_ring_slots_per_region.count();
@@ -816,10 +829,12 @@ fn pool_without_file_before_the_mount_writing_its_row_at(
 
 /// 场景：树表 0 条的池（S = 4），mkfs 之后接连可写挂载、从不写文件；txg 14 那次写行崩在根槽写之前（那一槽里留着上一圈 txg 2 的根），
 /// txg 23 那次写行同样崩在根槽写之前；下一次可写挂载写行 24、暖机 25（两块盘各一条）。挂载那一刻（取号之前）txg 14 那一槽读不出，之后照读。
-/// 预期：写行 24 那次窗口里还有比 14 旧的 13，不重读；暖机 25 那次零单元发布记根时那一槽成了窗口里最小的那个，重读读出 txg 2 的根：
-/// 挂载交回的根环表里那一槽换成重读读出的有效根，不再按挂载时读不出记，算这个进程知道住着根的槽。
+/// 预期：那一槽钉在它可能住过的最旧 txg 2，环里没有比它更旧的有效根（txg 23 那个洞里留着的是 11）：写行 24 那次记根之前它是根环表里最小的那个，
+/// 重读读出 txg 2 的根（按写行那一张实例表有效）；挂载交回的根环表里那一槽不再按读坏记、算这个进程知道住着根的槽。
+/// 这一条钉的是树表 0 条那一版上的写行发布（`transaction::prepare_the_row_publish_on_a_version_without_file`）走到 E2 重读。
 #[test]
-fn the_warm_up_on_a_version_without_file_rereads_the_slot_unreadable_at_mount() {
+fn the_row_publish_on_a_version_without_file_rereads_the_slot_unreadable_at_mount_pinned_at_the_oldest_txg_it_can_have_held(
+) {
     let image = pool_without_file_before_the_mount_writing_its_row_at(&[14, 23], 24);
     let parameters = parameters_of_the_ring_hole_pools();
     let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
@@ -854,15 +869,12 @@ fn the_warm_up_on_a_version_without_file_rereads_the_slot_unreadable_at_mount()
         .expect("可写挂载装了根环表");
     assert_eq!(
         (
-            occupancy
-                .ring_slots_unreadable_at_mount()
-                .get(&ring_slot)
-                .copied(),
+            occupancy.ring_slots_bad_at_mount().get(&ring_slot).copied(),
             occupancy
                 .ring_slots_known_to_hold_a_root()
                 .contains(&ring_slot)
         ),
         (None, true),
-        "暖机那次记根之前重读了那一槽"
+        "写行那次记根之前重读那一槽、读出 txg 2 的根"
     );
 }
diff --git a/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs b/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs
index 689c2a9..55dbbd8 100644
--- a/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs
+++ b/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs
@@ -10,6 +10,15 @@
 //!   池级 checker 读盘，那一槽里是窗口外的 txg 0，窗口里最旧的有效根是 25 ⇒ 记账比遍历多算释放代 25 的那几个槽，I-3.1 红。
 //! - E2：记 txg 47 那条根之前那一槽成了窗口里最小的那个，先重读：读出 txg 0 的根（按实例表有效、在窗口外）⇒ 那一槽当没有，门槛取 25，与 checker 对得上。
 //!   txg 46 时窗口里还有 txg 23 的根，比 24 旧，不重读，那一槽仍按 24 记着。
+//!
+//! 回收门槛退回整个环之后（用户 2026-10-04 定 RGall，`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1），读坏的槽在
+//! 回收门槛里钉在它可能住过的最旧 txg，重读读出有效根或确认为空之前门槛不越过它（用户 2026-10-04 定，重读时机照 E2；条文还没写回，
+//! 定案在 `records/2026-09-30-里程碑三开工第一轮.md`）：那一槽可能住过的最旧 txg 是 0，钉在 0，挂载那一刻起就是根环表里最小的那个，
+//! 挂载之后第一次记根就重读、读出 txg 0 的根（按实例表有效），照 0 算，直到 txg 48 这个进程写掉那一槽。txg 0 引用的单元一个都不回收，
+//! 挂载之后每一版那几条不变量都不红。文件名留着「成了窗口里最小的那个才重读」那一轮的叫法。
+//!
+//! 照能住的最大 txg 24 记（第 10 批 core 簇那一版）：txg 25–47 那几版上进程内按 24 回收，门槛越过那一槽里真住着的 txg 0，txg 0 引用的单元
+//! 被回收复用，而 txg 0 是环里读得出的有效根，池级 checker 照当候选走、读到复用的槽（txg 27 起 I-2.1、I-4.8、I-3.1 红）。
 mod common;
 
 use common::{
@@ -21,8 +30,10 @@ use common::{
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::allocator::{PoolAllocator, TxgsARingSlotBadAtMountCanHold};
 use singlefs_core::block_device::BlockDevice;
 use singlefs_core::mount::mount_writable;
+use singlefs_core::recovery::{choose_system_configuration, readable_roots_with_ring_slots};
 use singlefs_core::root_ring::{slot_offset, target_for_publish};
 use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
 use singlefs_harness::memory_pool::MemoryPool;
@@ -30,8 +41,8 @@ use singlefs_harness::SharedStream;
 
 /// 根槽没落的那次发布的 txg：它落到的区域 0 槽 0 里留着 mkfs 写的 txg 0 的根（24 槽的环）。
 const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 24;
-/// 那一槽在窗口里成了最小的那个的那一版：24 + 3S − 1。
-const TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW: u64 = 47;
+/// 这个进程写掉那一槽的那一版：24 + 3S。
+const TXG_THAT_OVERWRITES_THE_SLOT: u64 = 48;
 /// 一次覆盖写的内容长度：一个数据单元装得下。
 const OVERWRITE_BYTES: usize = 3000;
 /// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
@@ -54,8 +65,45 @@ fn violations_of(image: &MemoryPool) -> Vec<String> {
         .collect()
 }
 
+/// mkfs 写的 txg 0 那一版引用的单元（树表、实例表两条指针的两个落点；那一版的账就由这两条指针算，`mount::format_time_allocator`）里
+/// 分配器上空着（可再分配）的槽：(盘, 槽)。
+fn slots_of_the_mkfs_root_left_free(
+    image: &MemoryPool,
+    allocator: &PoolAllocator,
+) -> Vec<(u32, u64)> {
+    let system_configuration = choose_system_configuration(image).expect("系统配置");
+    let immutable = &system_configuration.immutable;
+    let mkfs_root = readable_roots_with_ring_slots(
+        image,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    )
+    .into_iter()
+    .map(|(_, root)| root)
+    .find(|root| root.checkpoint_txg == CheckpointTxg(0))
+    .expect("txg 0 的根还在环里、读得出");
+    [mkfs_root.tree_table, mkfs_root.instance_table]
+        .iter()
+        .flat_map(|pointer| pointer.locations)
+        .filter(|location| {
+            allocator
+                .devices
+                .iter()
+                .find(|device_map| device_map.device == location.device)
+                .expect("落点的盘在分配器里")
+                .is_free(location.slot)
+        })
+        .map(|location| (location.device.0, location.slot.0))
+        .collect()
+}
+
+/// 场景：上面那份盘面上可写挂载（实例 2），那一槽只在挂载那一次读不出；之后覆盖写到 txg 48（这个进程写掉那一槽）。
+/// 预期：挂载交回的根环表把那一槽记成挂载时读坏、钉在可能住过的最旧 txg 0（能住的最大 txg 24）；挂载之后第一次覆盖写记根之前重读，
+/// 读出 txg 0 的根，那一槽不再按读坏记、算这个进程知道住着根的槽。挂载之后与每一次覆盖写之后：txg 0 那一版引用的单元在分配器上
+/// 一个都不空闲，盘上那几条不变量不红。
 #[test]
-fn a_root_ring_slot_unreadable_only_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window_and_the_allocated_statistic_holds(
+fn a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants(
 ) {
     let mut pool = build_pool("root-ring-slot-unreadable-at-mount-reread");
     let slots_per_region = parameters().geometry.root_ring_slots_per_region;
@@ -135,25 +183,40 @@ fn a_root_ring_slot_unreadable_only_at_mount_is_reread_when_it_becomes_the_oldes
         (InstanceGeneration(2), CheckpointTxg(24)),
         "恢复由记录施加出 txg 24"
     );
-    let unreadable_slots_of_the_table =
-        |mounted_allocator: &singlefs_core::allocator::PoolAllocator| {
-            mounted_allocator
-                .root_ring_occupancy()
-                .expect("可写挂载装了根环表")
-                .ring_slots_unreadable_at_mount()
-        };
+    let image_now = || {
+        let mut image = crash_image.clone();
+        image.apply(&stream.retained_operations());
+        image
+    };
     assert_eq!(
-        unreadable_slots_of_the_table(&mounted.allocator).get(&hidden_slot),
-        Some(&CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST)),
-        "根环表把那一槽记成挂载时读不出、能住的最大 txg 24"
+        mounted
+            .allocator
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表")
+            .ring_slots_bad_at_mount()
+            .get(&hidden_slot),
+        Some(&TxgsARingSlotBadAtMountCanHold::new(
+            CheckpointTxg(0),
+            CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST)
+        )),
+        "根环表把那一槽记成挂载时读坏：钉在可能住过的最旧 txg 0，能住的最大 txg 24"
+    );
+    let after_the_mount = image_now();
+    assert_eq!(
+        (
+            slots_of_the_mkfs_root_left_free(&after_the_mount, &mounted.allocator),
+            violations_of(&after_the_mount)
+        ),
+        (Vec::new(), Vec::<String>::new()),
+        "挂载之后：txg 0 引用的单元一个都不空闲，那几条不变量不红"
     );
     let PoolVersion::WithFile(mut current) = mounted.current else {
         panic!("txg 24 那一版带文件");
     };
+    let first_txg_overwritten = current.root.checkpoint_txg.0 + 1;
     let publish_parameters = parameters();
     // 迭代上界：覆盖写到 txg 48（那一槽被这个进程写掉的那一版）；跨轮携带的是现行那一版。
-    while current.root.checkpoint_txg.0 < TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW + 1
-    {
+    while current.root.checkpoint_txg.0 < TXG_THAT_OVERWRITES_THE_SLOT {
         let checkpoint_txg = current.root.checkpoint_txg.0 + 1;
         current = {
             let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
@@ -170,27 +233,32 @@ fn a_root_ring_slot_unreadable_only_at_mount_is_reread_when_it_becomes_the_oldes
             .expect("覆盖写")
         };
         assert_eq!(current.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
-        if checkpoint_txg == TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW - 1 {
-            assert_eq!(
-                unreadable_slots_of_the_table(&mounted.allocator).get(&hidden_slot),
-                Some(&CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST)),
-                "txg 46：窗口里还有比 24 旧的 txg 23，那一槽没重读、仍按 24 记着"
-            );
-        }
-        if checkpoint_txg == TXG_AT_WHICH_THE_SLOT_BECOMES_THE_OLDEST_IN_THE_WINDOW {
+        if checkpoint_txg == first_txg_overwritten {
+            let occupancy = mounted
+                .allocator
+                .root_ring_occupancy()
+                .expect("可写挂载装了根环表");
             assert_eq!(
-                unreadable_slots_of_the_table(&mounted.allocator).get(&hidden_slot),
-                None,
-                "txg 47：那一槽成了窗口里最小的那个，重读出 txg 0 的根，不再按 24 记"
+                (
+                    occupancy.ring_slots_bad_at_mount().get(&hidden_slot).copied(),
+                    occupancy
+                        .ring_slots_known_to_hold_a_root()
+                        .contains(&hidden_slot)
+                ),
+                (None, true),
+                "txg {checkpoint_txg}：挂载之后第一次记根，那一槽钉在 0、是环里最小的那个，重读出 txg 0 的根，算这个进程知道住着根的槽"
             );
         }
-        // 盘上此刻的样子：崩溃状态加上挂载以来录下的全部写（读故障只在读那一侧，写原样落到里面那块盘上）。
-        let mut image = crash_image.clone();
-        image.apply(&stream.retained_operations());
+        let image = image_now();
+        let left_free = if checkpoint_txg < TXG_THAT_OVERWRITES_THE_SLOT {
+            slots_of_the_mkfs_root_left_free(&image, &mounted.allocator)
+        } else {
+            Vec::new()
+        };
         assert_eq!(
-            violations_of(&image),
-            Vec::<String>::new(),
-            "txg {checkpoint_txg} 之后：那一槽重读之后按读出来的根算，回收门槛与 checker 的窗口对得上"
+            (left_free, violations_of(&image)),
+            (Vec::new(), Vec::<String>::new()),
+            "txg {checkpoint_txg} 之后：txg 0 引用的单元一个都不空闲（txg 48 写掉那一槽之前），那几条不变量不红"
         );
     }
 }
diff --git a/crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs b/crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs
index 0c53bfa..2b1dc23 100644
--- a/crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs
+++ b/crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs
@@ -491,7 +491,9 @@ fn a_slot_whose_in_flight_limit_is_off_by_one_and_whose_generation_is_lower_foun
 }
 
 /// 逐档读到的自证过的槽都不可择（4096 处本池的槽 1 也抹掉，8192 处放一份 incompat 位图不认识的槽）：恢复报 incompat 位图不认识、
-/// 点名盘 0；checker 两块盘都择不到，而那一槽自证过，I-8.5（tail 槽有效集非空） 照样成立。
+/// 点名盘 0；checker 两块盘都择不到：这不是一个挂得上的镜像，每一条不变量都报不适用、带理由。I-8.5（tail 槽有效集非空） 也报不适用
+/// （那一槽自证过，全池皆废那一格不成立，不判红；而一块盘都交不出可择的系统配置，也不判成立：`InvariantVerdict` 的「不适用必须带理由，
+/// 不许静默算成立」）。改之前 I-8.5 在早退之前判成成立，坏盘输入战役（checker 档 `bad_disk_input_campaign`）把这类镜像算成三个读者都没看见。
 #[test]
 fn only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery(
 ) {
@@ -529,13 +531,28 @@ fn only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spa
         vec![(0, None), (1, None)],
         "checker 两块盘都择不到"
     );
-    assert_eq!(
+    assert!(
+        matches!(
+            verdict_of(
+                &image,
+                SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
+            ),
+            InvariantVerdict::NotApplicable(_)
+        ),
+        "8192 处那一槽自证过、而一块盘都择不到：I-8.5 报不适用：{:?}",
         verdict_of(
             &image,
             SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
-        ),
-        InvariantVerdict::Holds,
-        "8192 处那一槽自证过：I-8.5 成立"
+        )
+    );
+    let judged: Vec<(&str, InvariantVerdict)> = check_pool_image(&image)
+        .into_iter()
+        .filter(|(_, verdict)| !matches!(verdict, InvariantVerdict::NotApplicable(_)))
+        .collect();
+    assert_eq!(
+        judged,
+        Vec::new(),
+        "一块盘都择不到系统配置：每一条不变量都报不适用"
     );
 }
 
````

### mutations-append.tsv

````
RGall 进程内回收门槛改回一圈窗口（锚取这个进程记到的最新根：上一圈留在洞里的旧根被滤掉，它引用的单元当场回收）	crates/singlefs-core/src/allocator.rs	                RootRingOccupant::AbandonedRoot { .. } => None,\n            })\n            .min()	                RootRingOccupant::AbandonedRoot { .. } => None,\n            })\n            .filter(|checkpoint_txg| {\n                checkpoint_txg.0 + 3 * self.slots_per_region.count() > self.latest_recorded_txg.0\n            })\n            .min()	-p singlefs-core --lib -- root_ring_occupancy_keeps_the_reclaim_floor_at_the_oldest_valid_root_however_far_below_the_newest_it_is	root_ring_occupancy_keeps_the_reclaim_floor_at_the_oldest_valid_root_however_far_below_the_newest_it_is
RGall 进程内回收门槛改回一圈窗口（K1 连崩 11 次之后挂载：实例 1 留在洞里的根引用的单元当场回收）	crates/singlefs-core/src/allocator.rs	                RootRingOccupant::AbandonedRoot { .. } => None,\n            })\n            .min()	                RootRingOccupant::AbandonedRoot { .. } => None,\n            })\n            .filter(|checkpoint_txg| {\n                checkpoint_txg.0 + 3 * self.slots_per_region.count() > self.latest_recorded_txg.0\n            })\n            .min()	-p singlefs-harness --test reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root -- eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated	eleven_crashes_before_the_row_root_then_a_mount_leave_every_unit_of_the_roots_outside_the_one_revolution_window_allocated
RGall 进程内回收门槛改回一圈窗口（洞里 txg 7 的根引用的单元在 txg 35 起被回收）	crates/singlefs-core/src/allocator.rs	                RootRingOccupant::AbandonedRoot { .. } => None,\n            })\n            .min()	                RootRingOccupant::AbandonedRoot { .. } => None,\n            })\n            .filter(|checkpoint_txg| {\n                checkpoint_txg.0 + 3 * self.slots_per_region.count() > self.latest_recorded_txg.0\n            })\n            .min()	-p singlefs-harness --test one_revolution_window_after_a_hole_in_the_root_ring -- reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated	reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated
RGall E2 重读出的有效根照一圈窗口滤（锚取已知有效根里最新的那条：读出上一圈的 txg 0 不算进环里最旧有效根）	crates/singlefs-core/src/allocator.rs	        let oldest_valid_root = valid_root_txgs.iter().copied().min();	        let newest_valid_root = valid_root_txgs.iter().map(|txg| txg.0).max().unwrap_or(0);\n        let oldest_valid_root = valid_root_txgs\n            .iter()\n            .copied()\n            .filter(|txg| txg.0 + 12 > newest_valid_root)\n            .min();	-p singlefs-core --lib -- the_slot_bad_at_mount_rereading_as_a_root_of_the_previous_revolution_counts_by_the_txg_read	the_slot_bad_at_mount_rereading_as_a_root_of_the_previous_revolution_counts_by_the_txg_read
RGall 挂载时的回收门槛改回一圈窗口（锚取环里读得出的最新有效根：再挂时洞里 txg 7 的根被滤掉，它引用的单元当场回收）	crates/singlefs-core/src/mount.rs	    let oldest_valid_root_in_the_ring = roots\n        .iter()\n        .filter(|root| !is_abandoned(root))\n        .map(|root| root.checkpoint_txg)	    let oldest_valid_root_in_the_ring = roots\n        .iter()\n        .filter(|root| {\n            !is_abandoned(root)\n                && window_anchor.is_none_or(|anchor| {\n                    root.checkpoint_txg.0 + 3 * slots_per_region.count() > anchor.0\n                })\n        })\n        .map(|root| root.checkpoint_txg)	-p singlefs-harness --test one_revolution_window_after_a_hole_in_the_root_ring -- a_remount_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated	a_remount_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated
RGall 抬 F 那一遍的回收门槛改回一圈窗口（锚取这一遍读出来的最新根：洞里 txg 7 的根被滤掉，抬 F 生效之后它引用的单元放开）	crates/singlefs-core/src/mount.rs	    let valid_root_txgs: Vec<CheckpointTxg> = roots_read_from_the_ring\n        .iter()\n        .filter(|root| !abandoned_by_table(root, table))\n        .map(|root| root.checkpoint_txg)\n        .collect();	    let newest_txg_read = roots_read_from_the_ring\n        .iter()\n        .map(|root| root.checkpoint_txg.0)\n        .max()\n        .unwrap_or(0);\n    let valid_root_txgs: Vec<CheckpointTxg> = roots_read_from_the_ring\n        .iter()\n        .filter(|root| !abandoned_by_table(root, table))\n        .filter(|root| root.checkpoint_txg.0 + 3 * slots_per_region.count() > newest_txg_read)\n        .map(|root| root.checkpoint_txg)\n        .collect();	-p singlefs-harness --test one_revolution_window_after_a_hole_in_the_root_ring -- raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated	raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated
RGall 池级 checker 候选过滤改回一圈窗口（窗口外、洞里留着的实例 1 的根不当候选走，它们引用的单元照算已分配，I-3.1 红：攻方 RGalloc 那一臂）	crates/singlefs-checker/src/walk.rs	            let walked = *index == newest_index || (!abandoned && !below_floor);	            let walked = *index == newest_index\n                || (!abandoned && !below_floor && window.contains(root.checkpoint_txg));	-p singlefs-harness --test reclaim_floor_takes_the_oldest_valid_root_of_the_whole_ring_after_crashes_before_the_row_root -- one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring	one_crash_before_the_row_root_then_a_mount_keeps_the_allocated_statistic_with_the_reclaim_floor_and_the_checker_candidates_both_on_the_whole_ring
RGall 池级 checker「由记录施加出来的那一版」第 ④ 条改回一圈窗口（洞里 txg 7 之后只被 txg 8–11 引用的单元不并进遍历，I-3.1 红）	crates/singlefs-checker/src/walk.rs	        .filter(|(_, _, root)| !is_abandoned(root.instance, root.checkpoint_txg))	        .filter(|(_, _, root)| {\n            !is_abandoned(root.instance, root.checkpoint_txg)\n                && root.checkpoint_txg + 24\n                    > roots.iter().map(|(_, _, other)| other.checkpoint_txg).max().unwrap_or(0)\n        })	-p singlefs-harness --test one_revolution_window_after_a_hole_in_the_root_ring -- reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated	reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated
RGall 回退候选集改回判一圈窗口（洞里 txg 7 的根在现行那一版的窗口外，回退被拒）	crates/singlefs-core/src/mount.rs	        &ring_slots_known_to_hold_a_root_by(allocator),\n        target,\n    )?;	        &ring_slots_known_to_hold_a_root_by(allocator),\n        target,\n    )?;\n    if target.checkpoint_txg.0\n        + 3 * system_configuration\n            .immutable\n            .sizes\n            .root_ring_slots_per_region\n            .count()\n        <= current.root.checkpoint_txg.0\n    {\n        return Err(RollbackError::TargetNotACandidate {\n            target,\n            exclusion: RollbackCandidateExclusion::NotInRing,\n        });\n    }	-p singlefs-harness --test rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate -- rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_reads_back_that_version	rolling_back_to_a_root_left_in_a_hole_outside_the_one_revolution_window_reads_back_that_version
RGall 抬 F 的上限与「非空」改回只取一圈窗口里的有效根（洞里 txg 6、7、8 不算，上限从 6 抬到 21）	crates/singlefs-core/src/mount.rs	        .filter(|root| !abandoned_by_table(root, table))\n        .collect();\n    let mut newest_per_device	        .filter(|root| !abandoned_by_table(root, table))\n        .filter(|root| {\n            root.checkpoint_txg.0\n                + 3 * system_configuration\n                    .immutable\n                    .sizes\n                    .root_ring_slots_per_region\n                    .count()\n                > readable.iter().map(|other| other.checkpoint_txg.0).max().unwrap_or(0)\n        })\n        .collect();\n    let mut newest_per_device	-p singlefs-harness --test raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount -- admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x	admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x
第9批 RS3（RGall 之后在没洞的盘面上钉）：准入抬 F 的上限不按挂载那一刻读不出、之后没写过的槽封顶（F 越过那一槽能住的 txg 17，抬到 18，I-7.9 红）	crates/singlefs-core/src/mount.rs	        Some(highest_txg_it_can_hold) => ceiling_from_the_roots_read.min(highest_txg_it_can_hold),	        Some(_highest_txg_it_can_hold) => ceiling_from_the_roots_read,	-p singlefs-harness --test raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount -- admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount	admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount
A2 清扫算账不跳低于现行 F 的根的账（正常卸载抬过 F 之后 F 之下旧根的账读不出，每一轮都停在 AccountOfARingRootUnreadable，行数涨上去）	crates/singlefs-core/src/instance_table_sweep.rs	        .filter(|root| root.checkpoint_txg >= current_root.rollback_floor)	        .filter(|root| root.checkpoint_txg.0 < u64::MAX || root.checkpoint_txg >= current_root.rollback_floor)	-p singlefs-harness --test instance_table_sweep_at_the_end_of_a_writable_mount -- sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short	sweeping_a_periodic_schedule_of_unmounted_and_crashed_mounts_keeps_the_instance_table_short
C 抹之前那一遍重读读失败照抹照删（设备报错那一形，删行之后被删实例的孤儿剩 1 个）	crates/singlefs-core/src/instance_table_sweep.rs	        else {\n            return InstanceTableSweepOutcome::NoRowDeleted(\n                NoRowDeletedThisRound::AHeadRereadBeforeErasingIsNotTheJudgedUnit {\n                    placement: to_erase.placement,\n                    reread: HeadRereadBeforeErasing::ReadFailed,\n                },\n            );\n        };	        else {\n            continue;\n        };	-p singlefs-harness --test instance_table_sweep_at_the_end_of_a_writable_mount -- a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance	a_device_error_on_the_reread_before_erasing_deletes_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance
C 抹之前那一遍重读读回的不是那个头也照抹照删（读回全零那一形）	crates/singlefs-core/src/instance_table_sweep.rs	        if !still_the_unpublished_unit_judged {	        if false && !still_the_unpublished_unit_judged {	-p singlefs-harness --test instance_table_sweep_at_the_end_of_a_writable_mount -- zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance	zeros_read_back_on_the_reread_before_erasing_delete_no_row_and_the_next_sweep_leaves_no_orphan_of_the_deleted_instance
RGall I-7.9 不取锚在镜像最新根上的窗口（之前的根按整个环取：卸载那一串的洞上洞里旧根顶上来又红）	crates/singlefs-checker/src/walk.rs	            root.checkpoint_txg < raising_root.checkpoint_txg\n                && window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)	            root.checkpoint_txg < raising_root.checkpoint_txg	-p singlefs-harness --test pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window -- a_remount_over_a_root_ring_hole_left_by_a_journal_applied_version_keeps_an_old_floor_raise_within_its_ceiling	a_remount_over_a_root_ring_hole_left_by_a_journal_applied_version_keeps_an_old_floor_raise_within_its_ceiling
RGall I-7.9 不取锚在镜像最新根上的窗口（由记录施加出来那一类洞，txg 1 顶上来又红）	crates/singlefs-checker/src/walk.rs	            root.checkpoint_txg < raising_root.checkpoint_txg\n                && window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)	            root.checkpoint_txg < raising_root.checkpoint_txg	-p singlefs-harness --test pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window -- a_remount_over_a_root_ring_hole_whose_withheld_root_its_record_applies_keeps_an_old_floor_raise_within_its_ceiling	a_remount_over_a_root_ring_hole_whose_withheld_root_its_record_applies_keeps_an_old_floor_raise_within_its_ceiling
RGall I-7.9 不取锚在镜像最新根上的窗口（K1 类洞，txg 2 顶上来又红）	crates/singlefs-checker/src/walk.rs	            root.checkpoint_txg < raising_root.checkpoint_txg\n                && window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)	            root.checkpoint_txg < raising_root.checkpoint_txg	-p singlefs-harness --test pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window -- a_remount_over_a_root_ring_hole_left_by_a_withheld_row_publish_keeps_an_old_floor_raise_within_its_ceiling	a_remount_over_a_root_ring_hole_left_by_a_withheld_row_publish_keeps_an_old_floor_raise_within_its_ceiling
RGall I-7.9 带卸载记号那一支不取锚在镜像最新根上的窗口（正常卸载之后洞上再挂，卸载那一串的根按洞里留下的 v − 1 判红）	crates/singlefs-checker/src/walk.rs	                        *txg < raising_txg\n                            && window_anchored_at_the_newest_root_of_the_image.contains(*txg)	                        *txg < raising_txg	-p singlefs-harness --test pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window -- an_unmount_sequence_root_after_a_hole_one_revolution_later_is_not_judged_against_the_old_root_left_in_the_hole	an_unmount_sequence_root_after_a_hole_one_revolution_later_is_not_judged_against_the_old_root_left_in_the_hole
第10b批 E2 单测：重读仍读不出时不按钉着的最旧 txg 算（取读得出的最旧有效根）	crates/singlefs-core/src/allocator.rs	                return (Some(pinned_txg), rereads);	                return (oldest_valid_root, rereads);	-p singlefs-core --lib -- allocator::tests::the_slot_bad_at_mount_still_unreadable_on_the_reread_stays_pinned_at_the_oldest_txg_it_can_have_held	the_slot_bad_at_mount_still_unreadable_on_the_reread_stays_pinned_at_the_oldest_txg_it_can_have_held
第10b批 进程内重读读坏也当没有根（读得出而自证不过与读不出都拿出根环表，回收门槛越过盘上 txg 18 的有效根）	crates/singlefs-core/src/mount.rs	        RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => {\n            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified\n        }	        RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => {\n            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable\n        }	-p singlefs-harness --test root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread -- a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held	a_valid_root_read_bad_once_on_the_reread_keeps_the_slot_pinned_at_the_oldest_txg_it_can_have_held
第10b批 E2 进程内记根时挂载时读坏的槽成了根环表里最小的那个也不重读（一直钉在 txg 0，挂载之后第一次覆盖写那一槽仍按读坏记）	crates/singlefs-core/src/allocator.rs	        let reread_of_the_slot = reread(ring_slot);	        let reread_of_the_slot = RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified;	-p singlefs-harness --test root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window -- a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants	a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants
第10b批 E2 树表 0 条那一版上的写行发布记根之前不重读挂载时读坏的槽（那一槽仍按读坏记、不算知道住着根的槽）	crates/singlefs-core/src/allocator.rs	        let reread_of_the_slot = reread(ring_slot);	        let reread_of_the_slot = RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified;	-p singlefs-harness --test root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread -- the_row_publish_on_a_version_without_file_rereads_the_slot_unreadable_at_mount_pinned_at_the_oldest_txg_it_can_have_held	the_row_publish_on_a_version_without_file_rereads_the_slot_unreadable_at_mount_pinned_at_the_oldest_txg_it_can_have_held
第10b批 读坏的槽在回收门槛里钉在能住的最大 txg、不是可能住过的最旧 txg（挂载时门槛按 24 并，mkfs 的 txg 0 引用的单元当场回收）	crates/singlefs-core/src/allocator.rs	        self.oldest_txg_it_can_have_held	        self.highest_txg_it_can_hold	-p singlefs-harness --test root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window -- a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants	a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants
第10b批 挂载时的回收门槛把读坏的槽按能住的最大 txg 并（洞里 txg 7 的根所在那一槽再挂读不出，按 31 并，门槛取读得出的最旧有效根 14，txg 7 引用的单元当场回收）	crates/singlefs-core/src/mount.rs	                .map(|(_, txgs_it_can_hold)| txgs_it_can_hold.oldest_txg_it_can_have_held()),	                .map(|(_, txgs_it_can_hold)| txgs_it_can_hold.highest_txg_it_can_hold()),	-p singlefs-harness --test one_revolution_window_after_a_hole_in_the_root_ring -- a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated	a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated
第10b批 RR4 抬 F 那一遍的回收门槛不并根环表里挂载时读坏的槽（洞里 txg 7 的根所在那一槽一直读不出，抬 F 到 7 之后门槛 14，txg 7 引用的单元放开）	crates/singlefs-core/src/mount.rs	        .filter(|(ring_slot, _)| !ring_slots_read_from_the_ring.contains(ring_slot))	        .filter(|_| false)	-p singlefs-harness --test one_revolution_window_after_a_hole_in_the_root_ring -- raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated	raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated
第10b批 全零改回当读坏记、并进准入抬 F 的上限（没洞的那一份 X = 17 那一槽读回全零，上限并进它、F 停在 17）	crates/singlefs-core/src/mount.rs	        return RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten;	        return RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero;	-p singlefs-harness --test raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount -- admission_raising_the_floor_without_holes_passes_a_slot_reading_back_zeros_since_the_mount_as_the_known_cost	admission_raising_the_floor_without_holes_passes_a_slot_reading_back_zeros_since_the_mount_as_the_known_cost
第10b批 I-8.5 没有可择的系统配置时照旧判成成立（自证过而不可择的槽在，一槽都择不出，I-8.5 报成立、不报不适用）	crates/singlefs-checker/src/walk.rs	            &mut judgement_of_some_device_carrying_a_self_verified_slot,	            &mut root_ring_judgements,	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery	only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery
第10b批 进程内重读读回全零照旧钉着（前洞那一槽 txg 20 重读读回全零，不拿掉）	crates/singlefs-core/src/mount.rs	        RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => {\n            RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten\n        }	        RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => {\n            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified\n        }	-p singlefs-harness --test root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread -- a_front_hole_unreadable_at_mount_reread_as_all_zero_is_taken_as_never_written	a_front_hole_unreadable_at_mount_reread_as_all_zero_is_taken_as_never_written
第10b批 E2 单测：重读读回全零照旧钉着	crates/singlefs-core/src/allocator.rs	            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable\n            | RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten => {\n                bad_at_mount.remove(&ring_slot);\n            }\n            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified => {	            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {\n                bad_at_mount.remove(&ring_slot);\n            }\n            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified\n            | RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten => {	-p singlefs-core --lib -- allocator::tests::the_slot_bad_at_mount_reading_back_all_zero_on_the_reread_is_taken_as_never_written	the_slot_bad_at_mount_reading_back_all_zero_on_the_reread_is_taken_as_never_written
第10b批 读坏的槽可能住过的最旧 txg 改回在窗口里取（锚 40 时区域 2 槽 3 取 35、不取 11）	crates/singlefs-core/src/mount.rs	    for txg in 0..first_txg.0.min(ring_length_in_txgs) {	    for txg in first_txg.0 - txgs_walked_at_the_tail_of_the_interval..first_txg.0 {	-p singlefs-core --lib -- txgs_ring_slots_bad_at_mount_can_hold_tests	a_bad_slot_is_pinned_at_the_oldest_txg_it_can_have_held_below_the_window
第10b批 年轻的环里还没轮到的读坏槽也给一个可能住过的最旧 txg（不按 first_txg 截）	crates/singlefs-core/src/mount.rs	    for txg in 0..first_txg.0.min(ring_length_in_txgs) {	    for txg in 0..ring_length_in_txgs {	-p singlefs-core --lib -- txgs_ring_slots_bad_at_mount_can_hold_tests	a_bad_slot_the_young_ring_has_not_reached_holds_no_txg
第10b批 RGall 抬 F 的上限改回只取一圈窗口里的有效根（有洞的那一份 X 那一槽读回全零：洞里 txg 6、7、8 不算，上限 22，F 越过 21）	crates/singlefs-core/src/mount.rs	        .filter(|root| !abandoned_by_table(root, table))\n        .collect();\n    let mut newest_per_device	        .filter(|root| !abandoned_by_table(root, table))\n        .filter(|root| {\n            root.checkpoint_txg.0\n                + 3 * system_configuration\n                    .immutable\n                    .sizes\n                    .root_ring_slots_per_region\n                    .count()\n                > readable.iter().map(|other| other.checkpoint_txg.0).max().unwrap_or(0)\n        })\n        .collect();\n    let mut newest_per_device	-p singlefs-harness --test raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount -- admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_since_the_mount_stops_at_the_roots_left_in_the_holes	admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_since_the_mount_stops_at_the_roots_left_in_the_holes
````

### mutations-replacements.tsv

````
步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根	crates/singlefs-checker/src/walk.rs	            let walked = *index == newest_index || (!abandoned && !below_floor);	            let walked = *index == newest_index || !below_floor;	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored rolling_back_to_a_missing_root	rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write
步 5：checker 的候选集不按 F 收（F 之下的根照走）	crates/singlefs-checker/src/walk.rs	            let walked = *index == newest_index || (!abandoned && !below_floor);	            let walked = *index == newest_index || !abandoned;	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：影子账的豁免不看候选根的 txg 是否 ≥ F	crates/singlefs-core/src/mount.rs	        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)	        .filter(|root| !is_abandoned(root))	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
实四乙（实三交回 Q4：收口表第 43 行那一形在向前回退下由崩溃恢复造、走得到）：checker 在 I-3.1 的说明文字里少写回退下界那一项机理标识，判读的一方认不出「F 挡掉了环里读得到的根」，那一形归成新发现	crates/singlefs-checker/src/walk.rs	、回退下界 F {effective_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"	、低于 F 的根槽 {root_slots_dropped_below_floor} 个"	-p singlefs-harness --test checker_known_bad_images -- the_allocated_statistic_violation_carries_every_mechanism_field_the_known_red_forms_read	the_allocated_statistic_violation_carries_every_mechanism_field_the_known_red_forms_read
实四乙（C542 普通可写挂载那一支：所选根是最新根时这一串盖不掉它，回收不了这一串自己换下的，预演与真发不分叉）：挂载内每写一条根按这条根自己的 txg 当环里最旧的有效根回收，这一串自己换下的当场回收	crates/singlefs-core/src/allocator.rs	        let reclaim_floor = crate::mount::reclaim_floor(\n            occupancy.effective_floor,\n            occupancy.oldest_valid_root_in_the_ring(),\n        );	        let reclaim_floor =\n            crate::mount::reclaim_floor(occupancy.effective_floor, Some(checkpoint_txg));	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored writable_mount_on_the_newest_root	writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge
第 5 批池级 checker：I-8.5 自证过而不可择的槽不算自证过	crates/singlefs-checker/src/image.rs	        | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => true,\n        SystemConfigurationSlotReading::NotSelfVerified => false,	        => true,\n        SystemConfigurationSlotReading::SelfVerifiedButNotChoosable\n        | SystemConfigurationSlotReading::NotSelfVerified => false,	-p singlefs-harness --test checker_known_bad_images -- self_verified_system_configuration_slots_no_reader_can_choose_leave_every_invariant_not_applicable	self_verified_system_configuration_slots_no_reader_can_choose_leave_every_invariant_not_applicable
挂载时读不出的槽最大值取区间头那一段	crates/singlefs-core/src/mount.rs	    for txg in first_txg.0 - txgs_walked_at_the_tail_of_the_interval..first_txg.0 {	    for txg in lowest_txg_inside_the_window..lowest_txg_inside_the_window + txgs_walked_at_the_tail_of_the_interval {	-p singlefs-core --lib -- txgs_ring_slots_bad_at_mount_can_hold_tests	walking_one_period_at_each_end_matches_walking_every_txg_below_the_first
E2 单测：挂载时读不出的槽成了窗口里最小的那个也不重读	crates/singlefs-core/src/allocator.rs	        let reread_of_the_slot = reread(ring_slot);	        let reread_of_the_slot = RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified;	-p singlefs-core --lib -- allocator::tests::the_slot_bad_at_mount_rereading_as_a_root_of_the_previous_revolution_counts_by_the_txg_read	the_slot_bad_at_mount_rereading_as_a_root_of_the_previous_revolution_counts_by_the_txg_read
E2 进程内重读了挂载时读不出的槽却不按读到的改根环表	crates/singlefs-core/src/allocator.rs	            for (ring_slot, reread_of_the_slot) in rereads {	            for (ring_slot, reread_of_the_slot) in rereads.into_iter().take(0) {	-p singlefs-harness --test root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window -- a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants	a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants
E2 单测：窗口里还有更旧的有效根也重读挂载时读不出的槽	crates/singlefs-core/src/allocator.rs	        if oldest_valid_root.is_some_and(|oldest| oldest < pinned_txg) {	        if false {	-p singlefs-core --lib -- allocator::tests::the_slot_bad_at_mount_is_not_reread_while_an_older_valid_root_is_in_the_ring	the_slot_bad_at_mount_is_not_reread_while_an_older_valid_root_is_in_the_ring
第9批 RS1：重读读出有效根的那一槽不算这个进程知道住着根的槽（之后那一槽读坏，准入抬 F 按没有根往下走、发了写）	crates/singlefs-core/src/allocator.rs	                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => true,\n                RootRingOccupant::BadAtMount(_) => false,	                => true,\n                RootRingOccupant::BadAtMount(_)\n                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => false,	-p singlefs-harness --test root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread -- a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write	a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write
第9批 RS2 探针 p1：进程内重读读得出、自证过、按表有效的根也当没有（门槛越过窗口里那条有效根）	crates/singlefs-core/src/mount.rs	            if !abandoned_by_table(&root, table) =>	            if false && !abandoned_by_table(&root, table) =>	-p singlefs-harness --test root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread -- a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg	a_valid_root_read_on_the_reread_holds_the_reclaim_threshold_at_its_txg
第9批 RS2 探针 p2：进程内重读不按实例表判有效（读出被抛弃的根也照它的 txg 进窗口、算知道住着根的槽）	crates/singlefs-core/src/mount.rs	            if !abandoned_by_table(&root, table) =>	            if true || !abandoned_by_table(&root, table) =>	-p singlefs-harness --test root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread -- an_abandoned_root_read_on_the_reread_is_taken_as_no_root	an_abandoned_root_read_on_the_reread_is_taken_as_no_root
第9批 清扫后台那一支：抹之前不重读那一槽的头（判完之后被现行那一版占了的槽照抹）	crates/singlefs-core/src/instance_table_sweep.rs	        if !still_the_unpublished_unit_judged {	        if false && !still_the_unpublished_unit_judged {	-p singlefs-harness --test instance_table_sweep_at_the_end_of_a_writable_mount -- the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement	the_background_sweep_rereads_every_head_before_erasing_and_deletes_no_row_when_a_slot_was_allocated_after_the_judgement
第9批改钉 RS3（合入后改钉的那一条）：挂载那一刻读不出的槽不封顶准入抬 F 的上限（上限取读得出的根算出的 1、F 抬到 1）	crates/singlefs-core/src/mount.rs	        Some(highest_txg_it_can_hold) => ceiling_from_the_roots_read.min(highest_txg_it_can_hold),	        Some(_highest_txg_it_can_hold) => ceiling_from_the_roots_read,	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- root_slot_already_unreadable_at_mount_is_taken_as_a_possible_root_and_holds_the_admission_ceiling_at_its_txg	root_slot_already_unreadable_at_mount_is_taken_as_a_possible_root_and_holds_the_admission_ceiling_at_its_txg
删除批第 17 批 甲：可写挂载重建分配器时某块盘一条有效根都读不出（m 取不到）照原门槛整截当场放开、不只放到一圈窗口里最旧有效根，再挂期间盘 1 根环每槽读错的窄盘（264 槽）丢盘 0 之后盘 1 读回	crates/singlefs-core/src/mount.rs	        None => oldest_valid_root_in_the_ring,	        None => Some(floor_of_the_reclaim),	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads	after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_reading_no_root_on_device_one_and_crashing_in_its_row_publish_keeps_what_device_one_reads
````

### mutations-delete.txt

````
一圈窗口（D16 已定项 1，五处同改之一：分配器）：进程内的回收门槛不按窗口滤（锚取 0，环里最旧有效根卡在洞里留着的上一圈的根）
一圈窗口（五处同改之一：池级 checker 候选过滤）：窗口之外、洞里留着的上一圈的根照当候选走
E2 进程内记根时挂载时读不出的槽成了窗口里最小的那个也不重读（仍按最大 txg 算，I-3.1 在 24 + 3S − 1 那一版红）
E2 单测：重读仍读不出时不按它能住的最大 txg 算（取窗口里读得出的最旧有效根）
B2 抬 F 的上限与「非空」不只取一圈窗口里的有效根（洞里窗口外旧根的树表读不出，准入抬 F 被拒）
RR4 抬 F 那一遍的回收门槛不并进根环表里挂载时读不出的槽（门槛越过 X，I-3.1 红）
回退候选集不判一圈窗口（洞里窗口外的旧根被当成候选）
回退候选集「落在一圈窗口外」映射成模型的「低于 F_生效」
第9批 RS1：进程内重读读得出而自证不过改回当没有根（一次读坏的有效根被拿出根环表，回收门槛越过它，txg 29 那一版 I-3.1 记账少）
第9批 RS1 代价那一形：进程内重读读得出而自证不过改回当没有根（前洞加挂载读错那一版不再记账多，钉住的判成样子变了）
第9批 RS2 探针 q1：树表 0 条那一版上的暖机零单元发布记根不重读（挂载交回的根环表里那一槽仍按挂载时读不出记着）
第9批 RS3：准入抬 F 的上限不按挂载那一刻读不出、之后没写过的槽封顶（F 越过那一槽能住的 txg，抬到 22）
第9批 I-7.9 一圈窗口：只取锚在被判那条根上的窗口（条款字面），不取锚在镜像最新有效根上的那个（卸载那一串的洞上洞里旧根顶上来又红）
第9批 I-7.9 一圈窗口：只取锚在被判那条根上的窗口（由记录施加出来那一类洞，txg 1 顶上来又红）
第9批 I-7.9 一圈窗口：只取锚在被判那条根上的窗口（K1 类洞，txg 2 顶上来又红）
第9批改钉 I-7.9 带卸载记号那一支：只取锚在被判那条根上的窗口，不取锚在镜像最新有效根上的那个（正常卸载之后洞上再挂，卸载那一串的根按洞里留下的 v − 1 判红）
````

## batch10-checker-tier

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10-checker-tier/patch/`
- `crates.patch`：sha256 ddeb2e0bf8eee0bba0ac8c60bb2136ce77154acf33051e0bf6933a19efa0c8c6
- `mutations-append.tsv`：sha256 0b8efdc8e7a320f4b650a76044901e5b36960081c4a8f1c5f82d341e46a5d5fe
- `report.md`：sha256 daeb80e260c25941e84b831960ca9f87d1ddca223301bf1ad4d702ccf2a60312

### crates.patch

````
diff --git a/crates/singlefs-checker-tier/src/bad_disk_input.rs b/crates/singlefs-checker-tier/src/bad_disk_input.rs
index a51e3c2..7585ef0 100644
--- a/crates/singlefs-checker-tier/src/bad_disk_input.rs
+++ b/crates/singlefs-checker-tier/src/bad_disk_input.rs
@@ -1317,12 +1317,7 @@ pub fn feed_a_damaged_image(
                     | InvariantVerdict::PartiallyJudged(_) => None,
                 })
                 .collect();
-            let judged_nothing = report.iter().all(|(_, verdict)| match verdict {
-                InvariantVerdict::NotApplicable(_) => true,
-                InvariantVerdict::Holds
-                | InvariantVerdict::Violated(_)
-                | InvariantVerdict::PartiallyJudged(_) => false,
-            });
+            let judged_nothing = checker_judged_no_invariant(&report);
             (
                 ReaderOutcome::Finished {
                     how: format!("判红 {} 条", violated.len()),
@@ -1345,6 +1340,16 @@ pub fn feed_a_damaged_image(
     }
 }
 
+/// 池级 checker 一条都没判出来：每一条都报不适用或部分没判（[`BadDiskObservation::checker_judged_nothing`]）。
+/// 部分没判不算判过（`InvariantVerdict::PartiallyJudged`：判到的那一部分对得上、射程里另有一部分这张镜像上判不了，它不是成立，
+/// 代码轮第三轮判决 `research/prompts/m3-root-ring-code-r3-main-verification.md` 第三节）。
+fn checker_judged_no_invariant(report: &[(&'static str, InvariantVerdict)]) -> bool {
+    report.iter().all(|(_, verdict)| match verdict {
+        InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => true,
+        InvariantVerdict::Holds | InvariantVerdict::Violated(_) => false,
+    })
+}
+
 fn describe_read_back(read_back: &ObservedReadBack) -> String {
     match read_back {
         ObservedReadBack::NoFile { root } => format!(
@@ -2930,4 +2935,35 @@ mod tests {
         let kinds: BTreeSet<DamageKind> = EVERY_DAMAGE_KIND.into_iter().collect();
         assert_eq!(kinds.len(), EVERY_DAMAGE_KIND.len(), "清单里没有重复");
     }
+
+    /// 池级 checker 判出来的只有不适用与部分没判：算「一条都没判」（读者拒收整份镜像那一格），部分没判不算判过；
+    /// 有一条成立或违例就不算。
+    #[test]
+    fn a_checker_report_of_only_not_applicable_and_partially_judged_verdicts_counts_as_judging_nothing(
+    ) {
+        let not_applicable_and_partially_judged = [
+            ("I-1.1", InvariantVerdict::NotApplicable("读不出几何")),
+            (
+                "I-9.28",
+                InvariantVerdict::PartiallyJudged(
+                    "盘 0 上的根（实例 1、txg 4）：树表读不出".to_string(),
+                ),
+            ),
+        ];
+        assert!(
+            checker_judged_no_invariant(&not_applicable_and_partially_judged),
+            "只有不适用与部分没判：一条都没判"
+        );
+        for judged in [
+            InvariantVerdict::Holds,
+            InvariantVerdict::Violated("对不上".to_string()),
+        ] {
+            let mut report = not_applicable_and_partially_judged.to_vec();
+            report.push(("I-3.1", judged.clone()));
+            assert!(
+                !checker_judged_no_invariant(&report),
+                "有一条 {judged:?}：判过"
+            );
+        }
+    }
 }
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index b98ebf1..09b3d95 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -24,6 +24,7 @@
 //! 挂载做成的每个崩溃状态上，崩溃镜像的只读恢复（R1）与只挂载之后、挂载加一次发布之后的只读恢复（R2，后者比发布之前那一版）逐字段比，
 //! 挂载途中每个二次崩溃镜像的只读恢复也与 R1 比；可写挂载被拒的，理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的判拒绝本身合不合法。
 
+use std::borrow::Cow;
 use std::collections::{BTreeMap, BTreeSet};
 use std::fmt::Write as _;
 use std::path::{Path, PathBuf};
@@ -39,11 +40,17 @@ use singlefs_core::address::{
 };
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::instance_table::InstanceRow;
 use singlefs_core::instance_table_sweep::{
-    InstanceTableSweepAtTheEndOfTheMount, InstanceTableSweepOutcome,
+    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
+    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
+    SweepPublishPlacements,
 };
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
-use singlefs_core::mount::{mount_writable_with_space_admission, MountError, MountOutput};
+use singlefs_core::mount::{
+    mount_writable_with_instance_table_sweep_configuration, MountError, MountOutput, Mounted,
+    ShadowLedger,
+};
 use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
 use singlefs_core::recovery::{
     choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
@@ -60,10 +67,10 @@ use singlefs_harness::history::{
     choice_from_reproduction_environment, classify_failure, execute_history_with,
     generate_history_with_weights, newest_ring_root_and_slot_count_of,
     raised_floor_lands_only_on_abandoned_roots, record_check_aspects,
-    render_environment_assignments, FailureObservation, FailureSignature, GeneratedHistory,
-    GenerationWeights, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
-    HistorySampling, HistorySeed, ReproductionVariablePrefix, SeededRandomSource, StepPosition,
-    KNOWN_RED_FORMS,
+    render_environment_assignments, ContentChoice, ContentLength, FailureObservation,
+    FailureSignature, GeneratedHistory, GenerationWeights, HistoryEnding, HistoryExecution,
+    HistoryOperation, HistoryOperationKind, HistorySampling, HistorySeed, HistoryStartingPoint,
+    ReproductionVariablePrefix, SeededRandomSource, StepPosition, KNOWN_RED_FORMS,
 };
 use singlefs_harness::memory_pool::{
     newest_persisted_root, root_identity_written_by, some_publish_persisted_without_its_root,
@@ -536,6 +543,9 @@ pub struct CrashInjectionTally {
     /// 挂载与那次发布之后的池上跑的池级 checker 与记录核对器。
     pub checker_runs_after_the_writable_mount: u64,
     pub record_checks_after_the_writable_mount: u64,
+    /// 崩溃后镜像上那次可写挂载末尾的清扫发布删了行的次数，与那几次删行之前一共抹了几个头（清扫那两段摆没摆得到二次崩溃按它们看）。
+    pub writable_mounts_after_the_crash_deleting_instance_table_rows: u64,
+    pub heads_erased_by_the_instance_table_sweep_after_the_crash: u64,
     /// 挂载途中的二次崩溃：摆了几个、按崩在挂载的哪一段分。
     pub second_crash_points: u64,
     pub second_crash_points_by_phase: BTreeMap<WritableMountPhase, u64>,
@@ -567,6 +577,8 @@ pub struct CrashInjectionTally {
     pub mount_transparency_not_judged: BTreeMap<&'static str, u64>,
     pub mount_transparency_violations_by_field: BTreeMap<&'static str, u64>,
     pub mount_transparency_fields_not_judged: BTreeMap<&'static str, u64>,
+    /// 挂载透明判的时候 R2 那一版是清扫发布那一版、把清扫删的行放回去再比的次数（C704（挂载透明不认清扫发布）的处置真走到了）。
+    pub mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back: u64,
     /// 崩溃后镜像上的可写挂载被拒、理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的：拒得合法的、不合法的（判红）。
     pub writable_mount_refusals_on_the_d23_list_legal: u64,
     pub writable_mount_refusals_on_the_d23_list_illegal: u64,
@@ -621,6 +633,8 @@ impl CrashInjectionTally {
             publishes_after_the_writable_mount_refused,
             checker_runs_after_the_writable_mount,
             record_checks_after_the_writable_mount,
+            writable_mounts_after_the_crash_deleting_instance_table_rows,
+            heads_erased_by_the_instance_table_sweep_after_the_crash,
             second_crash_points,
             second_crash_points_by_phase,
             second_crash_recoveries_reading_a_file,
@@ -643,6 +657,7 @@ impl CrashInjectionTally {
             mount_transparency_not_judged,
             mount_transparency_violations_by_field,
             mount_transparency_fields_not_judged,
+            mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back,
             writable_mount_refusals_on_the_d23_list_legal,
             writable_mount_refusals_on_the_d23_list_illegal,
             crash_points_after_a_step_reading_under_injected_faults,
@@ -714,6 +729,10 @@ impl CrashInjectionTally {
             publishes_after_the_writable_mount_refused;
         self.checker_runs_after_the_writable_mount += checker_runs_after_the_writable_mount;
         self.record_checks_after_the_writable_mount += record_checks_after_the_writable_mount;
+        self.writable_mounts_after_the_crash_deleting_instance_table_rows +=
+            writable_mounts_after_the_crash_deleting_instance_table_rows;
+        self.heads_erased_by_the_instance_table_sweep_after_the_crash +=
+            heads_erased_by_the_instance_table_sweep_after_the_crash;
         self.second_crash_points += second_crash_points;
         for (phase, count) in second_crash_points_by_phase {
             *self.second_crash_points_by_phase.entry(*phase).or_insert(0) += count;
@@ -779,6 +798,8 @@ impl CrashInjectionTally {
                 .entry(reason)
                 .or_insert(0) += count;
         }
+        self.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back +=
+            mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back;
         self.writable_mount_refusals_on_the_d23_list_legal +=
             writable_mount_refusals_on_the_d23_list_legal;
         self.writable_mount_refusals_on_the_d23_list_illegal +=
@@ -902,6 +923,12 @@ impl CrashInjectionTally {
             self.checker_runs_after_the_writable_mount,
             self.record_checks_after_the_writable_mount
         );
+        let _ = writeln!(
+            text,
+            "崩溃后那次可写挂载末尾的清扫发布删了行 {} 次，删行之前一共抹了 {} 个头",
+            self.writable_mounts_after_the_crash_deleting_instance_table_rows,
+            self.heads_erased_by_the_instance_table_sweep_after_the_crash
+        );
         let _ = writeln!(
             text,
             "挂载途中的二次崩溃 {} 个：恢复读回文件 {} 次、没有文件 {} 次、失败 {} 次；问模型 {} 次、checker {} 次、记录核对器 {} 次",
@@ -944,7 +971,7 @@ impl CrashInjectionTally {
         );
         let _ = writeln!(
             text,
-            "挂载透明：第一次崩溃镜像上判了 {:?}，二次崩溃镜像上按段判了 {:?}；没判的 {:?}；判红的字段 {:?}；某一样没判的 {:?}",
+            "挂载透明：第一次崩溃镜像上判了 {:?}，二次崩溃镜像上按段判了 {:?}；没判的 {:?}；判红的字段 {:?}；某一样没判的 {:?}；把清扫删的行放回去再比的 {} 次",
             self.mount_transparency_judged
                 .iter()
                 .map(|(comparison, count)| (comparison.name(), *count))
@@ -955,7 +982,8 @@ impl CrashInjectionTally {
                 .collect::<Vec<_>>(),
             self.mount_transparency_not_judged,
             self.mount_transparency_violations_by_field,
-            self.mount_transparency_fields_not_judged
+            self.mount_transparency_fields_not_judged,
+            self.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back
         );
         let _ = writeln!(
             text,
@@ -1134,6 +1162,126 @@ pub fn inject_crashes_into_history_drawing_second_crashes(
     draw: CrashPointDraw,
     second_crash_draw: SecondCrashPointDraw,
     image_directory: Option<&Path>,
+) -> HistoryCrashInjection {
+    inject_crashes_into_history_with_every_switch(
+        history,
+        execution,
+        draw,
+        second_crash_draw,
+        InstanceTableSweepConfiguration::PRODUCT,
+        image_directory,
+    )
+}
+
+/// [`inject_crashes_into_history`]，崩溃后镜像上那次可写挂载与每个二次崩溃镜像上的再挂末尾那一判实例表清扫按 `instance_table_sweep`
+/// （[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`] 这一份让短历史摆得到清扫那两段；别的入口取产品路径 `InstanceTableSweepConfiguration::PRODUCT`）。
+///
+/// # Panics
+/// 同 [`inject_crashes_into_history`]。
+#[must_use]
+pub fn inject_crashes_into_history_with_the_instance_table_sweep_configuration(
+    history: &GeneratedHistory,
+    execution: HistoryExecution,
+    draw: CrashPointDraw,
+    instance_table_sweep: InstanceTableSweepConfiguration,
+    image_directory: Option<&Path>,
+) -> HistoryCrashInjection {
+    inject_crashes_into_history_with_every_switch(
+        history,
+        execution,
+        draw,
+        SecondCrashPointDraw::OneInEachMountPhaseBySeed,
+        instance_table_sweep,
+        image_directory,
+    )
+}
+
+/// [`history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row`] 里实例 2 覆盖写几次。根环 24 槽
+/// （`RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM` 每区 8 槽、三区）里实例 1 的根要全被轮出去：实例 2 的写行、两次暖机与崩溃之前的 24 次
+/// 覆盖写一共 27 条根；最后一次是摆崩溃状态的那一步。
+const OVERWRITES_OF_THE_HISTORY_WHOSE_MOUNT_DELETES_AN_INSTANCE_TABLE_ROW: u64 = 25;
+
+/// 崩溃后那次可写挂载（有一行就扫，[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`]）删得了行的一段历史：第一个文件之后关掉会话、可写挂载成实例 2
+/// （写行把实例 1 那一行写进表），再覆盖写 [`OVERWRITES_OF_THE_HISTORY_WHOSE_MOUNT_DELETES_AN_INSTANCE_TABLE_ROW`] 次——根环里实例 1 的根
+/// 全被轮出去，根环每一槽都是自证过的根，实例 1 那一行是候选行（D18（块里携带什么信息） 已定项 11「回收」一条）。每次发布都做完了，
+/// 盘上没有按实例 1 那一行判未发布的单元：清扫删行、一个头都不抹。
+#[must_use]
+pub fn history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row(
+) -> GeneratedHistory {
+    let mut operations = vec![HistoryOperation::CloseAndMountWritable];
+    operations.extend(
+        (0..OVERWRITES_OF_THE_HISTORY_WHOSE_MOUNT_DELETES_AN_INSTANCE_TABLE_ROW).map(|overwrite| {
+            HistoryOperation::PublishOverwrite(ContentChoice {
+                length: ContentLength::InsideOneDataUnit {
+                    selector: overwrite,
+                },
+                fill_seed: overwrite,
+            })
+        }),
+    );
+    GeneratedHistory {
+        seed: HistorySeed(3),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations,
+    }
+}
+
+/// 在 [`history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row`] 最后一步的开头摆一个崩溃状态（更早的写全落、
+/// 最后一步一个写都没落），崩溃后那次可写挂载与每个二次崩溃镜像上的再挂有一行就扫（[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`]）。
+/// 最后一步的第一个写落在第几段随历史的写数变：先把这段历史跑一遍、按同一条切段规则取出来，不写死段号。
+///
+/// # Panics
+/// 历史没跑到最后一步；最后一步的第一个写不是一段的段首（上一步发布返回之前那道屏障把它与上一步隔开：不是段首说明切段变了，
+/// 「最后一步一个写都没落」那个状态摆不准）。
+#[must_use]
+pub fn inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
+    execution: HistoryExecution,
+) -> HistoryCrashInjection {
+    let history = history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row();
+    let step_before_the_last = StepPosition::Operation(history.operations.len() - 2);
+    let stream = SharedStream::retaining_contents();
+    let mut operations_before_the_last_step = None;
+    let _run = execute_history_with(&history, execution, &stream, &mut |observation| {
+        if observation.position == step_before_the_last {
+            operations_before_the_last_step = Some(stream.operation_count());
+        }
+    });
+    let operations_before_the_last_step = operations_before_the_last_step.expect(
+        "这段历史的每一步都做得成（覆盖写落在 4 GiB 的盘上），倒数第二步之后观察者一定被调过",
+    );
+    let (_writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+        &stream.retained_operations(),
+        &execution.device_width.fixed_geometry(),
+    );
+    let first_write_of_the_last_step = stream_indexes
+        .iter()
+        .position(|stream_index| *stream_index >= operations_before_the_last_step)
+        .expect("最后一步（覆盖写）写过东西");
+    let segment_index = segments
+        .iter()
+        .position(|segment| segment.first() == Some(&first_write_of_the_last_step))
+        .expect("最后一步的第一个写是一段的段首：上一步发布返回之前那道屏障把它与上一步隔开");
+    inject_crashes_into_history_with_the_instance_table_sweep_configuration(
+        &history,
+        execution,
+        CrashPointDraw::ExactlyOneCrashState {
+            segment_index,
+            writes_from_the_segment_on: 1,
+            withheld_among_them: &[0],
+        },
+        INSTANCE_TABLE_SWEPT_ON_ANY_ROW,
+        None,
+    )
+}
+
+/// 崩溃注入的全部开关一处收：第一次崩溃怎么摆、二次崩溃怎么摆、挂载末尾的实例表清扫用哪一份配置。
+fn inject_crashes_into_history_with_every_switch(
+    history: &GeneratedHistory,
+    execution: HistoryExecution,
+    draw: CrashPointDraw,
+    second_crash_draw: SecondCrashPointDraw,
+    instance_table_sweep: InstanceTableSweepConfiguration,
+    image_directory: Option<&Path>,
 ) -> HistoryCrashInjection {
     let stream = SharedStream::retaining_contents();
     let mut committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
@@ -1373,13 +1521,20 @@ pub fn inject_crashes_into_history_drawing_second_crashes(
             ObservedReadBack::NoFile { .. } => Some(None),
             ObservedReadBack::Failed { .. } => None,
         };
-        let mount_run = mount_and_publish_once_on_the_crash_image(
-            &crash_image_materialized,
-            &parameters,
-            execution,
-            history.seed,
-            crash_point,
-        );
+        let mount_run =
+            mount_and_publish_once_with_the_instance_table_sweep_configuration_on_the_crash_image(
+                &crash_image_materialized,
+                &parameters,
+                execution,
+                history.seed,
+                crash_point,
+                instance_table_sweep,
+            );
+        if let Some(rows_deleted) = &mount_run.rows_deleted_by_the_instance_table_sweep {
+            tally.writable_mounts_after_the_crash_deleting_instance_table_rows += 1;
+            tally.heads_erased_by_the_instance_table_sweep_after_the_crash +=
+                u64::try_from(rows_deleted.heads_erased).expect("抹的头数装得进 u64");
+        }
         let (mount_writes, mount_segments, _mount_stream_indexes) =
             writes_and_segments_with_stream_indexes(&mount_run.mount_operations, &geometry);
         let (writes_of_the_publish_after_the_mount, _publish_segments, _publish_stream_indexes) =
@@ -1462,6 +1617,10 @@ pub fn inject_crashes_into_history_drawing_second_crashes(
                     .publishes_before_the_instance_table_sweep,
                 parameters: &parameters,
                 space_admission: execution.space_admission,
+                instance_table_sweep,
+                rows_deleted_by_the_sweep_of_the_uninterrupted_mount: mount_run
+                    .rows_deleted_by_the_instance_table_sweep
+                    .as_ref(),
                 reference_filesystem_at_the_first_crash: &reference_filesystem_judgement,
                 read_faults_in_the_first_crash_state: read_faults_in_the_crash_state,
                 second_crash_draw,
@@ -1670,11 +1829,29 @@ pub enum WritableMountAlone {
 }
 
 /// 在 `pool` 的副本上只起一次可写挂载（建池那一份参数、给定的空间准入开关），不接发布；交回挂载之后的池，或拒的理由、错误与拒之后的池。
+/// 挂载末尾的实例表清扫取产品路径的配置（`InstanceTableSweepConfiguration::PRODUCT`）。
 #[must_use]
 pub fn writable_mount_alone_on(
     pool: &MemoryPool,
     parameters: &MakeFilesystemParameters,
     space_admission: SpaceAdmission,
+) -> WritableMountAlone {
+    writable_mount_alone_with_the_instance_table_sweep_configuration_on(
+        pool,
+        parameters,
+        space_admission,
+        InstanceTableSweepConfiguration::PRODUCT,
+    )
+}
+
+/// 同 [`writable_mount_alone_on`]，挂载末尾那一判实例表清扫的开关与阈值由调用方给（`mount::mount_writable_with_instance_table_sweep_configuration`；
+/// 影子账开着，与产品路径同一臂）。
+#[must_use]
+pub fn writable_mount_alone_with_the_instance_table_sweep_configuration_on(
+    pool: &MemoryPool,
+    parameters: &MakeFilesystemParameters,
+    space_admission: SpaceAdmission,
+    instance_table_sweep: InstanceTableSweepConfiguration,
 ) -> WritableMountAlone {
     let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = pool
         .devices
@@ -1688,11 +1865,16 @@ pub fn writable_mount_alone_on(
             (*identity, device)
         })
         .collect();
-    let mounted_instance =
-        match mount_writable_with_space_admission(parameters, &mut devices, space_admission) {
-            Err(error) => Err(error),
-            Ok(mounted) => Ok(mounted.output.instance),
-        };
+    let mounted_instance = match mount_writable_with_instance_table_sweep_configuration(
+        parameters,
+        &mut devices,
+        space_admission,
+        ShadowLedger::On,
+        instance_table_sweep,
+    ) {
+        Err(error) => Err(error),
+        Ok(mounted) => Ok(mounted.output.instance),
+    };
     let pool_after = MemoryPool {
         devices: devices
             .into_iter()
@@ -1772,8 +1954,61 @@ pub struct WritableMountOnTheCrashImage {
     /// 挂载续做每一步那一版的两棵用户可见树的根指针（挂载透明那一样按它比，用户 2026-10-03 定「续做算挂载的一部分」）；挂载被拒时 None。
     pub user_visible_tree_root_pointers_of_the_continuation:
         Option<UserVisibleTreeRootPointersOfTheContinuation>,
+    /// 挂载末尾的清扫发布删了行时：清扫发布那一版是哪条根、删了哪几行、抹了几个头（挂载透明的实例表行那一样按它放回被删的行再比，
+    /// C704（挂载透明不认清扫发布））；没清扫、清扫没删行、或挂载被拒时 None。
+    pub rows_deleted_by_the_instance_table_sweep: Option<RowsDeletedByTheInstanceTableSweepPublish>,
 }
 
+/// 崩溃后镜像上那次可写挂载末尾的清扫发布删了行（`InstanceTableSweepOutcome::RowsDeleted`）：清扫发布那一版的根身份、删掉的行、
+/// 删行之前抹了几个头。清扫是挂载交回之前最后一件事（`mount::mount_writable` 文档「可写挂载末尾」），清扫发布那一版就是挂载交回的现行版本。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RowsDeletedByTheInstanceTableSweepPublish {
+    pub sweep_publish: (InstanceGeneration, CheckpointTxg),
+    pub rows_deleted: Vec<InstanceRow>,
+    pub heads_erased: usize,
+}
+
+impl RowsDeletedByTheInstanceTableSweepPublish {
+    /// 挂载交回值里清扫删了行时交回删了什么；没删行（没扫、扫了没删、清扫发布没发成）时 None。
+    #[must_use]
+    pub fn of_the_mount(mounted: &Mounted) -> Option<Self> {
+        match &mounted.output.instance_table_sweep {
+            InstanceTableSweepAtTheEndOfTheMount::Swept(
+                InstanceTableSweepOutcome::RowsDeleted {
+                    rows_deleted,
+                    heads_erased,
+                },
+            ) => Some(Self {
+                sweep_publish: (
+                    mounted.current.root().instance,
+                    mounted.current.root().checkpoint_txg,
+                ),
+                rows_deleted: rows_deleted.clone(),
+                heads_erased: heads_erased.len(),
+            }),
+            InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
+            | InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
+            | InstanceTableSweepAtTheEndOfTheMount::Swept(
+                InstanceTableSweepOutcome::NoRowDeleted(_)
+                | InstanceTableSweepOutcome::SweepPublishRefusedForSpace { .. }
+                | InstanceTableSweepOutcome::SweepPublishFailed { .. }
+                | InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing { .. },
+            ) => None,
+        }
+    }
+}
+
+/// 崩溃注入里那次可写挂载（与每个二次崩溃镜像上的再挂）末尾的实例表清扫：有一行就扫（开关关着、阈值
+/// [`InstanceTableRowCountThreshold::ANY_ROW`]、清扫发布照挂载的分配器取落点）。产品路径的阈值占位 369 行（C690（清扫触发阈值的默认值没定）），
+/// 短历史到不了；要摆到清扫那两段（[`WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep`]、[`WritableMountPhase::InstanceTableSweepPublish`]）
+/// 的用例用这一份（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支都能被测试强制进入）。
+pub const INSTANCE_TABLE_SWEPT_ON_ANY_ROW: InstanceTableSweepConfiguration =
+    InstanceTableSweepConfiguration {
+        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
+        row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
+        sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
+    };
+
 /// 挂载之后那一次发布写的内容：随种子与崩溃状态变，装得进一个数据单元（模型只罩一个数据单元的文件）。
 fn content_of_the_publish_after_the_crash(seed: HistorySeed, crash_point: &CrashPoint) -> Vec<u8> {
     let length = 97 + crash_point.segment_index % 64;
@@ -1786,7 +2021,8 @@ fn content_of_the_publish_after_the_crash(seed: HistorySeed, crash_point: &Crash
 /// 在 `crash_image_materialized` 的副本上起可写挂载（建池那一份参数、那一段历史的空间准入开关），做成了就接着发一次布：
 /// 挂载交回的现行版本带文件、第一个文件那条记录还活着时覆盖写，树表 0 条时发第一个文件（与随机历史里这两个入口同一种调法），
 /// 有树而文件集合为空（删完那一版）时、或带文件而第一个文件已删完或正在删（unlink 之后再建过 inode）时建一个 size 0 的 inode
-/// （D8（核心索引结构） 已定项 16 第 4、5 条：写入口拒不在树里与正在删的 inode）。
+/// （D8（核心索引结构） 已定项 16 第 4、5 条：写入口拒不在树里与正在删的 inode）。挂载末尾的实例表清扫取产品路径的配置
+/// （`InstanceTableSweepConfiguration::PRODUCT`）。
 #[must_use]
 pub fn mount_and_publish_once_on_the_crash_image(
     crash_image_materialized: &MemoryPool,
@@ -1794,6 +2030,27 @@ pub fn mount_and_publish_once_on_the_crash_image(
     execution: HistoryExecution,
     seed: HistorySeed,
     crash_point: &CrashPoint,
+) -> WritableMountOnTheCrashImage {
+    mount_and_publish_once_with_the_instance_table_sweep_configuration_on_the_crash_image(
+        crash_image_materialized,
+        parameters,
+        execution,
+        seed,
+        crash_point,
+        InstanceTableSweepConfiguration::PRODUCT,
+    )
+}
+
+/// 同 [`mount_and_publish_once_on_the_crash_image`]，挂载末尾那一判实例表清扫的开关与阈值由调用方给
+/// （`mount::mount_writable_with_instance_table_sweep_configuration`，影子账开着、与产品路径同一臂）。
+#[must_use]
+pub fn mount_and_publish_once_with_the_instance_table_sweep_configuration_on_the_crash_image(
+    crash_image_materialized: &MemoryPool,
+    parameters: &MakeFilesystemParameters,
+    execution: HistoryExecution,
+    seed: HistorySeed,
+    crash_point: &CrashPoint,
+    instance_table_sweep: InstanceTableSweepConfiguration,
 ) -> WritableMountOnTheCrashImage {
     let stream = SharedStream::retaining_contents();
     let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
@@ -1812,9 +2069,18 @@ pub fn mount_and_publish_once_on_the_crash_image(
                 )
             })
             .collect();
-    let mounted =
-        mount_writable_with_space_admission(parameters, &mut devices, execution.space_admission);
+    let mounted = mount_writable_with_instance_table_sweep_configuration(
+        parameters,
+        &mut devices,
+        execution.space_admission,
+        ShadowLedger::On,
+        instance_table_sweep,
+    );
     let mount_operation_count = stream.operation_count();
+    let rows_deleted_by_the_instance_table_sweep = mounted
+        .as_ref()
+        .ok()
+        .and_then(RowsDeletedByTheInstanceTableSweepPublish::of_the_mount);
     let floor_raised_by_the_mount = mounted
         .as_ref()
         .ok()
@@ -1937,6 +2203,7 @@ pub fn mount_and_publish_once_on_the_crash_image(
         publishes_before_the_instance_table_sweep,
         floor_raises_in_the_mount_writes,
         user_visible_tree_root_pointers_of_the_continuation,
+        rows_deleted_by_the_instance_table_sweep,
     }
 }
 
@@ -2424,11 +2691,112 @@ fn violations_of_the_mount_transparency(
     violations
 }
 
-/// 第一次崩溃镜像上一个动作的挂载透明：R1（`before_the_mount`）与动作之后的池（`pool_after_the_action`）上的只读恢复（`after_the_action`）逐字段比；
+/// 挂载透明判红时记进违例的名字：R2 那一版是清扫发布那一版，它的实例表里却仍有清扫说删掉了的行（C704（挂载透明不认清扫发布））。
+pub const MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH: &str =
+    "挂载透明·实例表行（清扫发布那一版里仍有它删掉的行）";
+
+/// R2 那一版的实例表行对清扫发布删的行是什么样（C704（挂载透明不认清扫发布）的处置：判法先跟上，再让战役带清扫配置）。
+#[derive(Debug)]
+enum InstanceTableRowsAgainstTheSweepPublish {
+    /// 挂载没删行、R2 那一版不是清扫发布那一版，或它的实例表读不出（读不出照原样交给逐字段判，实例表行那一样照判红）：照原样比。
+    ComparedAsRead,
+    /// R2 那一版是清扫发布那一版、被删的行一条都不在：把它们放回去之后的 R2（其余的行照 D18 已定项 11「恢复写的行」判）。
+    DeletedRowsPutBack(Box<RecoveryFieldsRead>),
+    /// R2 那一版是清扫发布那一版，被删的行还在：判红。
+    DeletedRowsStillPresent(Vec<InstanceRow>),
+}
+
+/// 按清扫发布那一版的根身份认 R2（只挂载与二次崩溃比恢复落到的那一版、挂载加一次发布比发布之前那一版——清扫是挂载交回之前最后一件事，
+/// 两种比法拿到的都可能是它），判被删的行在不在。
+fn instance_table_rows_against_the_sweep_publish(
+    after_the_action: &RecoveryFieldsRead,
+    rows_deleted_by_the_sweep: Option<&RowsDeletedByTheInstanceTableSweepPublish>,
+) -> InstanceTableRowsAgainstTheSweepPublish {
+    let Some(rows_deleted_by_the_sweep) = rows_deleted_by_the_sweep else {
+        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
+    };
+    let Some(compared) = &after_the_action.compared_version else {
+        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
+    };
+    if (compared.root.instance, compared.root.checkpoint_txg)
+        != rows_deleted_by_the_sweep.sweep_publish
+    {
+        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
+    }
+    let Some(rows_read) = &compared.instance_table_rows else {
+        return InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead;
+    };
+    let deleted_instances: BTreeSet<InstanceGeneration> = rows_deleted_by_the_sweep
+        .rows_deleted
+        .iter()
+        .map(|row| row.instance)
+        .collect();
+    let still_present: Vec<InstanceRow> = rows_read
+        .iter()
+        .filter(|row| deleted_instances.contains(&row.instance))
+        .copied()
+        .collect();
+    if !still_present.is_empty() {
+        return InstanceTableRowsAgainstTheSweepPublish::DeletedRowsStillPresent(still_present);
+    }
+    let rows_with_the_deleted_put_back: Vec<InstanceRow> = rows_read
+        .iter()
+        .chain(&rows_deleted_by_the_sweep.rows_deleted)
+        .copied()
+        .collect();
+    let mut put_back = after_the_action.clone();
+    put_back
+        .compared_version
+        .as_mut()
+        .expect("上面判过 R2 那一版在")
+        .instance_table_rows = Some(rows_with_the_deleted_put_back);
+    InstanceTableRowsAgainstTheSweepPublish::DeletedRowsPutBack(Box::new(put_back))
+}
+
+/// 挂载透明逐字段判（`mount_transparency::judge_mount_transparency`）之前，挂载末尾的清扫发布删了行时按 C704（挂载透明不认清扫发布）先处置 R2：
+/// 那里的逐字段判法按「恢复写的行」比实例表行，不认清扫发布删行。R2 那一版是清扫发布那一版而被删的行一条都不在 ⇒ 交回放回之后的 R2；
+/// 被删的行还在 ⇒ R2 照原样交回、另交一条违例（[`MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH`]，
+/// 计进按字段分的判红计数）；别的照原样交回。被删的是哪几行取挂载交回值（与 F 取挂载交回的抬 F、续做取挂载交回的根指针同一个做法），
+/// 判法不另算哪几行该删。
+fn after_the_action_with_the_rows_deleted_by_the_sweep_put_back<'read>(
+    tally: &mut CrashInjectionTally,
+    after_the_action: &'read RecoveryFieldsRead,
+    rows_deleted_by_the_sweep: Option<&RowsDeletedByTheInstanceTableSweepPublish>,
+) -> (Cow<'read, RecoveryFieldsRead>, Vec<(&'static str, String)>) {
+    match instance_table_rows_against_the_sweep_publish(after_the_action, rows_deleted_by_the_sweep)
+    {
+        InstanceTableRowsAgainstTheSweepPublish::ComparedAsRead => {
+            (Cow::Borrowed(after_the_action), Vec::new())
+        }
+        InstanceTableRowsAgainstTheSweepPublish::DeletedRowsPutBack(put_back) => {
+            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back += 1;
+            (Cow::Owned(*put_back), Vec::new())
+        }
+        InstanceTableRowsAgainstTheSweepPublish::DeletedRowsStillPresent(still_present) => {
+            *tally
+                .mount_transparency_violations_by_field
+                .entry(MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH)
+                .or_insert(0) += 1;
+            (
+                Cow::Borrowed(after_the_action),
+                vec![(
+                    MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH,
+                    format!(
+                        "清扫发布 {:?} 删了 {:?}；那一版的实例表里仍有 {still_present:?}",
+                        rows_deleted_by_the_sweep.map(|deleted| deleted.sweep_publish),
+                        rows_deleted_by_the_sweep.map(|deleted| &deleted.rows_deleted)
+                    ),
+                )],
+            )
+        }
+    }
+}
+
+/// 第一次崩溃镜像上一个动作的挂载透明：R1（`before_the_mount`）与动作之后的池（`pool_after_the_action`）上的只读恢复（`after_the_action_as_read`；挂载末尾的清扫发布删了行时先按 C704 处置）逐字段比；
 /// 判红的落成这一截（[`CrashStateStage::MountTransparencyAfterTheCrash`]）的新发现，镜像留的是动作之后的池。
 #[allow(
     clippy::too_many_arguments,
-    reason = "每个参数各是一样东西：计数与发现、哪个动作、两次恢复与动作之后的池、动作与挂载续做那几步的交回值、这一个崩溃状态落在哪"
+    reason = "每个参数各是一样东西：计数与发现、哪个动作、两次恢复与动作之后的池、动作与挂载续做那几步的交回值、清扫发布删的行、这一个崩溃状态落在哪"
 )]
 fn judge_mount_transparency_of_one_comparison(
     tally: &mut CrashInjectionTally,
@@ -2436,13 +2804,21 @@ fn judge_mount_transparency_of_one_comparison(
     comparison: MountTransparencyComparison,
     before_the_mount: &RecoveryFieldsRead,
     pool_after_the_action: &MemoryPool,
-    after_the_action: &RecoveryFieldsRead,
+    after_the_action_as_read: &RecoveryFieldsRead,
     action: MountTransparencyAction,
     continuation: &UserVisibleTreeRootPointersOfTheContinuation,
+    rows_deleted_by_the_sweep: Option<&RowsDeletedByTheInstanceTableSweepPublish>,
     seed: HistorySeed,
     crash_point: &CrashPoint,
     image_directory: Option<&Path>,
 ) {
+    let (compared_after_the_action, rows_deleted_by_the_sweep_still_present) =
+        after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
+            tally,
+            after_the_action_as_read,
+            rows_deleted_by_the_sweep,
+        );
+    let after_the_action: &RecoveryFieldsRead = &compared_after_the_action;
     let transparency =
         judge_mount_transparency(before_the_mount, after_the_action, action, continuation);
     match &transparency {
@@ -2454,7 +2830,8 @@ fn judge_mount_transparency_of_one_comparison(
         }
         MountTransparency::NotJudged(_) => {}
     }
-    let violations = violations_of_the_mount_transparency(tally, &transparency);
+    let mut violations = violations_of_the_mount_transparency(tally, &transparency);
+    violations.extend(rows_deleted_by_the_sweep_still_present);
     if violations.is_empty() {
         return;
     }
@@ -2528,6 +2905,7 @@ fn judge_mount_transparency_on_the_crash_image(
             after_the_mount,
             MountTransparencyAction::MountAlone(floor),
             continuation_of_a_mount_that_was_made(mount_run),
+            mount_run.rows_deleted_by_the_instance_table_sweep.as_ref(),
             seed,
             crash_point,
             image_directory,
@@ -2554,6 +2932,7 @@ fn judge_mount_transparency_on_the_crash_image(
                 &after_the_publish,
                 MountTransparencyAction::MountAndOnePublish(floor),
                 continuation_of_a_mount_that_was_made(mount_run),
+                mount_run.rows_deleted_by_the_instance_table_sweep.as_ref(),
                 seed,
                 crash_point,
                 image_directory,
@@ -2578,12 +2957,21 @@ fn judge_mount_transparency_on_the_crash_image(
 /// 抬 F 的写有持久的（从第一串把新 F 写进系统配置的那个写起，`persisted_in_the_mount` 与挂载那一段写表逐条对应）时，F 那一样不判
 /// （条款没写落在哪一处）；暖机那一次发布的写崩出来的态照 r5 判决 U5 前半句判 F 不变（代码轮第二轮判决 K2-1）。抬 F 的界在挂载写表里取不出的
 /// （代码轮第三轮判决第三节第 3 条），这一次挂载的二次崩溃整段 F 不判、计数。没崩过的那一次挂载被拒的不判。
+/// 没崩过的那一次挂载末尾的清扫发布删了行时，实例表行那一样照 C704（挂载透明不认清扫发布）的处置判
+/// （[`after_the_action_with_the_rows_deleted_by_the_sweep_put_back`]）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "每个参数各是一样东西：计数、R1 与二次崩溃镜像上的恢复、没崩过的那一次挂载的抬 F 界与续做根指针与清扫删的行、二次崩溃的持久集合与段"
+)]
 fn mount_transparency_violations_of_a_second_crash(
     tally: &mut CrashInjectionTally,
     fields_read_after_the_first_crash: &RecoveryFieldsRead,
     fields_read_after_the_second_crash: &RecoveryFieldsRead,
     floor_raises_in_the_uninterrupted_mount_writes: Option<FloorRaisesInTheMountWrites>,
     continuation_of_the_uninterrupted_mount: Option<&UserVisibleTreeRootPointersOfTheContinuation>,
+    rows_deleted_by_the_sweep_of_the_uninterrupted_mount: Option<
+        &RowsDeletedByTheInstanceTableSweepPublish,
+    >,
     persisted_in_the_mount: &[bool],
     phase: WritableMountPhase,
 ) -> Vec<(&'static str, String)> {
@@ -2599,9 +2987,15 @@ fn mount_transparency_violations_of_a_second_crash(
         "没崩过的那一次挂载做成时抬 F 的界与续做那几步的根指针一起交回（mount_and_publish_once_on_the_crash_image 里同一个 Ok 取的两样）",
     );
     let floor_raises = floor_raises_in_the_mount_writes.at_a_second_crash(persisted_in_the_mount);
+    let (fields_read_after_the_second_crash, rows_deleted_by_the_sweep_still_present) =
+        after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
+            tally,
+            fields_read_after_the_second_crash,
+            rows_deleted_by_the_sweep_of_the_uninterrupted_mount,
+        );
     let transparency = judge_mount_transparency(
         fields_read_after_the_first_crash,
-        fields_read_after_the_second_crash,
+        &fields_read_after_the_second_crash,
         MountTransparencyAction::SecondCrashInsideTheMount(floor_raises),
         continuation,
     );
@@ -2614,39 +3008,60 @@ fn mount_transparency_violations_of_a_second_crash(
         }
         MountTransparency::NotJudged(_) => {}
     }
-    violations_of_the_mount_transparency(tally, &transparency)
+    let mut violations = violations_of_the_mount_transparency(tally, &transparency);
+    violations.extend(rows_deleted_by_the_sweep_still_present);
+    violations
 }
 
 /// 挂载那一段录制流里每一次写落在挂载的哪一段：先是取号的系统配置槽写，第一个别的写起是写行那次发布；
 /// 一次发布的根槽写与紧跟着的系统配置槽写（轮换、抬 F 那一串先写的新 F）之后的第一个别的写起是下一次发布：
 /// 写行之后、第 `publishes_before_the_deletion_queue_continuation` 个根槽写之前的那几次是暖机（连同挂载推的抬 F），
 /// 从它起是续做（没崩过的那一次挂载被拒、交不出这个数时 None：写行之后都算暖机）；第 `publishes_before_the_instance_table_sweep`
-/// 个根槽写之后起是清扫抹头（FUA 写零头所在的物理块、每块盘一道屏障），抹头之后第一个不带 FUA 的单元写或 journal 记录写起是清扫发布
+/// 个根槽写之后起是清扫：其中抹头那一段只有 FUA 写零头所在物理块的单元写（与每块盘一道屏障），第一个别的写（不带 FUA 的单元写、
+/// journal 记录写……）起是清扫发布——一个头都没抹时清扫发布从清扫之前最后一次发布的轮换之后第一个写起，抹头那一段一个写都没有
 /// （没清扫时 None）。
 fn writable_mount_phase_of_each_write(
     writes: &[RetainedWrite],
     publishes_before_the_deletion_queue_continuation: Option<usize>,
     publishes_before_the_instance_table_sweep: Option<usize>,
 ) -> Vec<WritableMountPhase> {
-    let phase_of_the_publish_after = |root_records_written: usize| {
-        if publishes_before_the_instance_table_sweep.is_some_and(|publishes_before_the_sweep| {
-            root_records_written > 0 && root_records_written >= publishes_before_the_sweep
-        }) {
-            return WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep;
-        }
-        match (
-            root_records_written,
-            publishes_before_the_deletion_queue_continuation,
-        ) {
-            (0, _) => WritableMountPhase::RowPublish,
-            (publishes_landed, Some(publishes_before_the_continuation))
-                if publishes_landed >= publishes_before_the_continuation =>
-            {
-                WritableMountPhase::Continuation
+    let phase_of_the_publish_after =
+        |root_records_written: usize, phase_so_far: WritableMountPhase, write: &RetainedWrite| {
+            if publishes_before_the_instance_table_sweep.is_some_and(|publishes_before_the_sweep| {
+                root_records_written > 0 && root_records_written >= publishes_before_the_sweep
+            }) {
+                // 已经在抹头里：往清扫发布换段由下面那几条臂按写的种类判。刚从清扫之前那几段进清扫：头一个写是 FUA 写零（或屏障）才进抹头，
+                // 别的写说明一个头都没抹、直接是清扫发布。
+                if phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {
+                    return WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep;
+                }
+                return match (write.kind, write.is_force_unit_access) {
+                    (StepKind::UnitWrite, true) | (StepKind::Barrier, _) => {
+                        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
+                    }
+                    (StepKind::UnitWrite, false)
+                    | (
+                        StepKind::JournalRecord
+                        | StepKind::RootRecordFua
+                        | StepKind::ZeroFill
+                        | StepKind::SystemConfigurationSlot,
+                        _,
+                    ) => WritableMountPhase::InstanceTableSweepPublish,
+                };
             }
-            (_, Some(_) | None) => WritableMountPhase::WarmUp,
-        }
-    };
+            match (
+                root_records_written,
+                publishes_before_the_deletion_queue_continuation,
+            ) {
+                (0, _) => WritableMountPhase::RowPublish,
+                (publishes_landed, Some(publishes_before_the_continuation))
+                    if publishes_landed >= publishes_before_the_continuation =>
+                {
+                    WritableMountPhase::Continuation
+                }
+                (_, Some(_) | None) => WritableMountPhase::WarmUp,
+            }
+        };
     let mut phase = WritableMountPhase::Acquisition;
     let mut root_records_written = 0;
     // 迭代上界是挂载那一段的写数；跨轮带的是这一次写落在哪一段与已写的根槽数；没有提前出口。
@@ -2693,7 +3108,7 @@ fn writable_mount_phase_of_each_write(
                     | StepKind::RootRecordFua
                     | StepKind::ZeroFill
                     | StepKind::Barrier,
-                ) => phase_of_the_publish_after(root_records_written),
+                ) => phase_of_the_publish_after(root_records_written, phase, write),
             };
             if write.kind == StepKind::RootRecordFua {
                 root_records_written += 1;
@@ -2905,9 +3320,13 @@ struct SecondCrashInputs<'history> {
     /// 没崩过的那一次挂载续做每一步那一版的两根指针（二次崩溃落到挂载写的那一版时挂载透明按它比）；挂载被拒时 None。
     continuation_of_the_uninterrupted_mount:
         Option<&'history UserVisibleTreeRootPointersOfTheContinuation>,
-    /// 二次崩溃之后再挂用的建池参数与空间准入开关（与没崩过的那一次挂载同一份）。
+    /// 二次崩溃之后再挂用的建池参数、空间准入开关与实例表清扫的配置（与没崩过的那一次挂载同一份）。
     parameters: &'history MakeFilesystemParameters,
     space_admission: SpaceAdmission,
+    instance_table_sweep: InstanceTableSweepConfiguration,
+    /// 没崩过的那一次挂载末尾的清扫发布删了哪几行（二次崩溃落到清扫发布那一版时挂载透明按它放回再比，C704）；没删行时 None。
+    rows_deleted_by_the_sweep_of_the_uninterrupted_mount:
+        Option<&'history RowsDeletedByTheInstanceTableSweepPublish>,
     /// RefFS 在第一次崩溃上的判定：判绿时带着以读出的内容重新起头的 RefFS，二次崩溃接着它判。
     reference_filesystem_at_the_first_crash: &'history ReferenceFileSystemJudgementOfACrash,
     /// 第一次崩溃那个态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：二次崩溃之后再挂被拒时判 C670 那一条沿用它。
@@ -2944,6 +3363,8 @@ fn judge_second_crashes_inside_the_writable_mount(
         continuation_of_the_uninterrupted_mount,
         parameters,
         space_admission,
+        instance_table_sweep,
+        rows_deleted_by_the_sweep_of_the_uninterrupted_mount,
         reference_filesystem_at_the_first_crash,
         read_faults_in_the_first_crash_state,
         second_crash_draw,
@@ -3061,6 +3482,7 @@ fn judge_second_crashes_inside_the_writable_mount(
             },
             parameters,
             space_admission,
+            instance_table_sweep,
             second.phase,
             seed,
             crash_point,
@@ -3075,6 +3497,7 @@ fn judge_second_crashes_inside_the_writable_mount(
             &fields_read_after_the_second_crash,
             floor_raises_in_the_uninterrupted_mount_writes,
             continuation_of_the_uninterrupted_mount,
+            rows_deleted_by_the_sweep_of_the_uninterrupted_mount,
             &persisted_in_the_mount,
             second.phase,
         ));
@@ -3105,7 +3528,7 @@ fn judge_second_crashes_inside_the_writable_mount(
 /// （[`judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion`]）；没崩过的那一次挂载被拒（`once` 是没判的理由）时不再挂，两样都不判。
 #[allow(
     clippy::too_many_arguments,
-    reason = "每个参数各是一样东西：两次挂载的输入、计数与发现、这一个二次崩溃落在哪"
+    reason = "每个参数各是一样东西：两次挂载的输入（连同清扫的配置）、计数与发现、这一个二次崩溃落在哪"
 )]
 fn judge_the_remount_after_a_second_crash(
     tally: &mut CrashInjectionTally,
@@ -3114,6 +3537,7 @@ fn judge_the_remount_after_a_second_crash(
     second_crash_image: &RefusalOnTheCrashImage<'_>,
     parameters: &MakeFilesystemParameters,
     space_admission: SpaceAdmission,
+    instance_table_sweep: InstanceTableSweepConfiguration,
     phase: WritableMountPhase,
     seed: HistorySeed,
     crash_point: &CrashPoint,
@@ -3130,10 +3554,11 @@ fn judge_the_remount_after_a_second_crash(
         }
         Ok(content) => content,
     };
-    let remount = writable_mount_alone_on(
+    let remount = writable_mount_alone_with_the_instance_table_sweep_configuration_on(
         second_crash_image.crash_image_materialized,
         parameters,
         space_admission,
+        instance_table_sweep,
     );
     let pool_after_the_remount = match &remount {
         WritableMountAlone::Refused {
@@ -4215,7 +4640,7 @@ pub fn seed_slices(seed_count: u64, worker_threads: usize) -> Vec<std::ops::Rang
 mod tests {
     use super::*;
     use singlefs_core::mount::FloorRaiseFailedAfterTheMountsPublishes;
-    use singlefs_harness::history::{generate_history, HistoryStartingPoint};
+    use singlefs_harness::history::generate_history;
     use singlefs_harness::mount_transparency::WitnessedCounterBranchThatHolds;
 
     const FOUR_CRASH_POINTS: CrashPointDraw = CrashPointDraw::Sampled {
@@ -4641,6 +5066,49 @@ mod tests {
         }
     }
 
+    /// 清扫一个头都没抹（只删行）：清扫之前最后一次发布的轮换之后，第一个写就是清扫发布的单元写，抹头那一段一个写都没有。
+    /// 把清扫发布的第一个写记进抹头那一段的，二次崩溃会说崩在了没写过一个字节的抹头里（假绿）。
+    #[test]
+    fn a_sweep_that_erased_no_head_puts_every_write_after_the_last_publish_before_it_into_the_sweep_publish(
+    ) {
+        let write_of = |kind: StepKind, is_force_unit_access: bool| RetainedWrite {
+            device: DeviceIdentity(0),
+            kind,
+            is_force_unit_access,
+            offset: DeviceOffsetInBytes(0),
+            contents: singlefs_harness::memory_pool::WrittenContents::Bytes(vec![0; 512]),
+        };
+        let writes = vec![
+            write_of(StepKind::SystemConfigurationSlot, false),
+            write_of(StepKind::UnitWrite, false),
+            write_of(StepKind::JournalRecord, false),
+            write_of(StepKind::RootRecordFua, true),
+            write_of(StepKind::SystemConfigurationSlot, false),
+            write_of(StepKind::UnitWrite, false),
+            write_of(StepKind::UnitWrite, false),
+            write_of(StepKind::JournalRecord, false),
+            write_of(StepKind::RootRecordFua, true),
+            write_of(StepKind::SystemConfigurationSlot, false),
+        ];
+        let phases = writable_mount_phase_of_each_write(&writes, None, Some(1));
+        assert_eq!(
+            phases,
+            [
+                WritableMountPhase::Acquisition,
+                WritableMountPhase::RowPublish,
+                WritableMountPhase::RowPublish,
+                WritableMountPhase::RowPublish,
+                WritableMountPhase::RowPublish,
+                WritableMountPhase::InstanceTableSweepPublish,
+                WritableMountPhase::InstanceTableSweepPublish,
+                WritableMountPhase::InstanceTableSweepPublish,
+                WritableMountPhase::InstanceTableSweepPublish,
+                WritableMountPhase::InstanceTableSweepPublish,
+            ],
+            "写行那次的轮换之后没有 FUA 写零：清扫发布从它第一个单元写起，抹头那一段是空的"
+        );
+    }
+
     /// 造写表用：一个 512 字节的写；根槽写的字节里在偏移 24、28 写上根身份（`root_identity_written_by` 的读法）。
     fn synthetic_write(kind: StepKind, root: Option<(u32, u64)>) -> RetainedWrite {
         let mut bytes = vec![0u8; 512];
@@ -5053,7 +5521,9 @@ mod tests {
     /// 挂载每一段都摆得到二次崩溃的两段历史（U4，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：从第一个文件起的
     /// 随机一段（起点就是攻方 `research/prompts/m3-inv-a-r1-opus-output.md` 一·2 的「新池新建文件做完的镜像」），与第一个文件之后经挂着的会话
     /// unlink 它的一段（意图那次 + 最后一批：崩溃状态有落在意图那一版上的，那一版上的可写挂载续做、写出续做那一段，
-    /// 之后那一次发布落在删完那一版上）。两段同一种摆法（短段整段枚举），计数并在一起。
+    /// 之后那一次发布落在删完那一版上）。两段同一种摆法（短段整段枚举），计数并在一起。再并上一段崩溃后那次可写挂载删得了行的历史
+    /// （[`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row`]，有一行就扫）：
+    /// 清扫发布那一段从它来。那段历史上没有孤儿、一个头都不抹，抹头那一段从这三段里都摆不出来。
     fn crash_injections_into_histories_reaching_every_mount_phase(
     ) -> (CrashInjectionTally, Vec<HistoryCrashInjection>) {
         let histories = [
@@ -5068,7 +5538,7 @@ mod tests {
                 operations: vec![HistoryOperation::Unlink],
             },
         ];
-        let injections: Vec<HistoryCrashInjection> = histories
+        let mut injections: Vec<HistoryCrashInjection> = histories
             .iter()
             .map(|history| {
                 inject_crashes_into_history(
@@ -5082,6 +5552,11 @@ mod tests {
                 )
             })
             .collect();
+        injections.push(
+            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
+                HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+            ),
+        );
         let mut tally = CrashInjectionTally::default();
         for injection in &injections {
             tally.absorb(&injection.tally);
@@ -5089,6 +5564,89 @@ mod tests {
         (tally, injections)
     }
 
+    /// 崩溃后那次可写挂载末尾的清扫发布删了行（C704（挂载透明不认清扫发布） 的验收）：挂载做成、删了实例 1 那一行、一个头都没抹；
+    /// 只挂载与挂载加一次发布两个比法拿到的 R2 都是清扫发布那一版，实例表行那一样把删的行放回去再比，挂载透明一样都没红；
+    /// 清扫发布那一段摆了二次崩溃，再挂判了 I-8.4、挂载透明判了；抹头那一段一个二次崩溃都没摆（没抹头时那一段一个写都没有）；
+    /// 一条新发现都没有。判别力：挂载透明不放回删的行时只挂载那一个比法判红（`crates/mutations.tsv` 里点名这条的行）。
+    #[test]
+    fn a_writable_mount_after_the_crash_whose_sweep_publish_deletes_a_row_is_judged_in_the_sweep_publish_phase_without_a_red_field(
+    ) {
+        let injection =
+            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
+                HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+            );
+        let tally = &injection.tally;
+        let rendered = tally.render();
+        assert_eq!(
+            (
+                tally.crash_points,
+                tally.writable_mounts_after_the_crash_succeeded,
+                tally.publishes_after_the_writable_mount
+            ),
+            (1, 1, 1),
+            "前提：摆了一个崩溃状态，那次可写挂载与之后那一次发布都做成：{rendered}"
+        );
+        assert_eq!(
+            (
+                tally.writable_mounts_after_the_crash_deleting_instance_table_rows,
+                tally.heads_erased_by_the_instance_table_sweep_after_the_crash
+            ),
+            (1, 0),
+            "前提：那次挂载末尾的清扫发布删了行、一个头都没抹（这段历史每次发布都做完了，没有孤儿）：{rendered}"
+        );
+        assert_eq!(
+            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back, 2,
+            "只挂载与挂载加一次发布两个比法拿到的 R2 都是清扫发布那一版，各放回一次删的行再比：{rendered}"
+        );
+        let phase_count = |counts: &BTreeMap<WritableMountPhase, u64>, phase| {
+            counts.get(&phase).copied().unwrap_or(0)
+        };
+        assert_eq!(
+            phase_count(
+                &tally.second_crash_points_by_phase,
+                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
+            ),
+            0,
+            "一个头都没抹时抹头那一段一个写都没有，不摆二次崩溃：{rendered}"
+        );
+        for (what, counts) in [
+            ("摆了二次崩溃", &tally.second_crash_points_by_phase),
+            ("再挂判了 I-8.4", &tally.remount_idempotence_judged_by_phase),
+            (
+                "判了挂载透明",
+                &tally.mount_transparency_judged_on_second_crashes_by_phase,
+            ),
+        ] {
+            assert!(
+                phase_count(counts, WritableMountPhase::InstanceTableSweepPublish) >= 1,
+                "清扫发布那一段{what}：{rendered}"
+            );
+        }
+        assert!(
+            tally.mount_transparency_violations_by_field.is_empty(),
+            "挂载透明判红的字段 {:?}：{}",
+            tally.mount_transparency_violations_by_field,
+            injection
+                .new_findings
+                .iter()
+                .map(CrashPointFinding::render)
+                .collect::<String>()
+        );
+        assert_eq!(
+            tally.remounts_after_a_second_crash_reading_different_content, 0,
+            "二次崩溃之后再挂读出的内容与没崩过的那一次不同：{rendered}"
+        );
+        assert!(
+            injection.new_findings.is_empty(),
+            "新发现：{}",
+            injection
+                .new_findings
+                .iter()
+                .map(CrashPointFinding::render)
+                .collect::<String>()
+        );
+    }
+
     /// I-8.4 的 R-换 在真的二次崩溃上不误红：两段历史（[`crash_injections_into_histories_reaching_every_mount_phase`]），
     /// 起点那一段也摆崩溃状态；每个二次崩溃镜像上都再挂了一次，取号、写行、暖机、续做四段各有判过的，
     /// 读出的内容都与没崩过的那一次挂载之后读出的相同（攻方量到实例代号、实例表行、扇区都不同的那种也算相同）。
@@ -5212,9 +5770,7 @@ mod tests {
     /// 关掉空间准入跑第一个文件之后重挂与 12 次覆盖写，再判准入挂载一次。
     #[test]
     fn the_writable_mount_on_a_crash_image_names_the_first_floor_raise_write_after_the_warm_up() {
-        use singlefs_harness::history::{
-            ContentChoice, ContentLength, HistoryDeviceWidth, PerStepChecker,
-        };
+        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
         let width = HistoryDeviceWidth::UnitAreaOf256Slots;
         let mut operations = vec![HistoryOperation::CloseAndMountWritable];
         operations.extend(std::iter::repeat_n(
@@ -5934,9 +6490,7 @@ mod tests {
 
     /// 起点（取号、暖机、第一个文件）之后一次覆盖写、不重挂的历史，与跑它用的执行（单元区 384 槽的小盘，不逐步跑 checker）。
     fn history_of_the_first_file_and_one_overwrite() -> (GeneratedHistory, HistoryExecution) {
-        use singlefs_harness::history::{
-            ContentChoice, ContentLength, HistoryDeviceWidth, PerStepChecker,
-        };
+        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
         (
             GeneratedHistory {
                 seed: HistorySeed(0),
@@ -6178,6 +6732,7 @@ mod tests {
             },
             &execution.device_width.parameters(),
             execution.space_admission,
+            InstanceTableSweepConfiguration::PRODUCT,
             WritableMountPhase::RowPublish,
             history.seed,
             &crash_point,
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs
index 88ab0d2..04a35ee 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs
@@ -38,7 +38,7 @@ use singlefs_core::transaction::{
     publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
 };
 use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
-use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES};
+use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, SLOT_BYTES};
 use singlefs_harness::memory_pool::{
     closed_form_state_count, writes_and_segments,
     writes_and_segments_with_stream_indexes_and_entries, CrashImage, MemoryPool, PublishedVersion,
@@ -127,6 +127,38 @@ const QUICK_TIER_STATES_BY_PUBLISH_THROUGH_THE_UNMOUNT: &str = "instance1_txg1=1
      instance2_txg13=13 instance2_txg14=13 instance2_txg15=21 instance2_txg16=13 instance2_txg17=13 \
      instance2_txg18=21 instance2_txg19=13 after_the_last_root=8 every_write_persisted=1";
 
+/// 回退 D（txg 9）那次发布在零根的谱系树上第一次追加谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条），多写的两个单元的落点：
+/// 谱系容器（码 3 打包记录单元，32768 字节）与谱系树根（码 2 索引节点，16384 字节），两块盘同槽各一份
+/// （根环线第 9 批 9-8 的探针在这条流上量到的槽号）。
+const LINEAGE_CONTAINER_SLOT_OF_THE_ROLLBACK_PUBLISH: u64 = 50352;
+const LINEAGE_TREE_ROOT_SLOT_OF_THE_ROLLBACK_PUBLISH: u64 = 50354;
+
+/// 回退 D 是这条脚本的第几条根（从 0 数：实例 1 的 txg 1–4，实例 2 的 txg 5–8，D 是 txg 9）。
+const ROOT_INDEX_OF_THE_ROLLBACK_PUBLISH: usize = 8;
+
+/// 回退 D 那次发布的单元写里落在 `slot` 起、单元类 `unit_class`、整单元 `unit_bytes` 字节的有几个（每块盘一份就是 2）。
+fn units_of_the_rollback_publish_at(
+    writes: &[RetainedWrite],
+    root_indexes: &[usize],
+    slot: u64,
+    unit_class: u8,
+    unit_bytes: u64,
+) -> usize {
+    let after_the_root_before_it = root_indexes[ROOT_INDEX_OF_THE_ROLLBACK_PUBLISH - 1] + 1;
+    let its_root = root_indexes[ROOT_INDEX_OF_THE_ROLLBACK_PUBLISH];
+    writes[after_the_root_before_it..its_root]
+        .iter()
+        .filter(|write| write.kind == StepKind::UnitWrite)
+        .filter(|write| write.offset.0 == slot * SLOT_BYTES)
+        .filter(|write| {
+            write.bytes().is_some_and(|bytes| {
+                u64::try_from(bytes.len()).expect("一个单元的字节数装得进 u64") == unit_bytes
+                    && bytes[6] == unit_class
+            })
+        })
+        .count()
+}
+
 /// 到 E 之后再覆盖写一次（txg 18）那一版的数据单元落点。
 const TXG_18_DATA_UNIT_SLOT: u64 = 50178;
 /// A（txg 3）的数据单元那一对槽：到 E 之后第二次覆盖写（txg 19）复用它。
@@ -388,6 +420,40 @@ fn prepare(tag: &str, script: Script) -> Prepared {
     }
     let base = pool.memory_pool_after_mkfs();
     let (writes, segments) = writes_and_segments_after_mkfs(&pool);
+    let root_indexes: Vec<usize> = writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
+        .map(|(index, _)| index)
+        .collect();
+    // 谱系那两个单元的槽号先于段序列钉：回退不追加谱系记录时 D 那一段从 24 退回 20，段序列那一条也红，这一条先说清缺的是谱系那两个单元。
+    match script {
+        Script::SecondVersionOnly | Script::ThirdVersion => {}
+        Script::RollbackToFirstVersion
+        | Script::ReuseAfterRaisingFloorThenNormalUnmount
+        | Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => {
+            assert_eq!(
+                (
+                    units_of_the_rollback_publish_at(
+                        &writes,
+                        &root_indexes,
+                        LINEAGE_CONTAINER_SLOT_OF_THE_ROLLBACK_PUBLISH,
+                        UNIT_CLASS_PACKED,
+                        DATA_UNIT_BYTES
+                    ),
+                    units_of_the_rollback_publish_at(
+                        &writes,
+                        &root_indexes,
+                        LINEAGE_TREE_ROOT_SLOT_OF_THE_ROLLBACK_PUBLISH,
+                        UNIT_CLASS_INDEX_NODE,
+                        NODE_BYTES
+                    )
+                ),
+                (2, 2),
+                "回退 D 那次发布写出谱系容器（码 3，槽 {LINEAGE_CONTAINER_SLOT_OF_THE_ROLLBACK_PUBLISH}）与谱系树根（码 2，槽 {LINEAGE_TREE_ROOT_SLOT_OF_THE_ROLLBACK_PUBLISH}），两块盘各一份"
+            );
+        }
+    }
     let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
     // 每次发布：它的单元写、两份 journal 记录、一条根槽写、之后两块盘各一次系统配置槽轮换，共「单元写数 + 5」次写；
     // C577 之后轮换之后、发布返回之前有一道屏障，上一次发布的系统配置槽轮换自成一段，不再与这一次的单元写并成一段
@@ -475,12 +541,6 @@ fn prepare(tag: &str, script: Script) -> Prepared {
             );
         }
     }
-    let root_indexes: Vec<usize> = writes
-        .iter()
-        .enumerate()
-        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
-        .map(|(index, _)| index)
-        .collect();
     let expected_roots = match script {
         Script::SecondVersionOnly => 4,
         Script::ThirdVersion => 8,
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs b/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs
index 7aeb5c8..836ca95 100644
--- a/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs
+++ b/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs
@@ -3,9 +3,11 @@
 //! 在同一份崩溃后镜像上真的起一次可写挂载（取号、写行、暖机）、再发一次布、跑池级 checker；挂载途中再崩一次（二次崩溃），
 //! 取号、写行、暖机、续做各段各摆一个，每个二次崩溃状态上只读恢复、问模型、池级 checker、记录核对器。
 //! 实现在 `singlefs_checker_tier::crash_injection`；随机崩溃注入的快档与大档（`crash_injection_campaign.rs`）
-//! 走的是同一个入口，这里用两段短历史与一段 unlink 的历史钉「每一截都跑到了、四段都摆到了、一条新发现都没有」。
+//! 走的是同一个入口，这里用两段短历史与一段 unlink 的历史钉「每一截都跑到了、四段都摆到了、一条新发现都没有」，
+//! 再加一段崩溃后那次可写挂载删得了实例表行的历史（有一行就扫）：清扫发布那一段的二次崩溃从它来。
 
 use singlefs_checker_tier::crash_injection::{
+    inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row,
     inject_crashes_into_history, CrashInjectionTally, CrashPointDraw, WritableMountPhase,
 };
 use singlefs_core::admission::SpaceAdmission;
@@ -25,7 +27,8 @@ const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
 /// 崩溃状态有落在意图那一版上的，那一版上的可写挂载续做，U4，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：
 /// 每个崩溃状态之后都起了可写挂载、发了一次布（删完那一版上建 inode）、跑了 checker，
 /// 挂载途中的二次崩溃取号、写行、暖机、续做四段都摆到了，二次崩溃状态上恢复、问模型、checker、记录核对器各跑了一次；
-/// 两截上一条新发现都没有，挂载与那次发布一次都没被拒。
+/// 两截上一条新发现都没有，挂载与那次发布一次都没被拒。第四段在一段覆盖写到根环里只剩实例 2 的根的历史最后一步开头崩一次，
+/// 那次可写挂载与二次崩溃之后的再挂有一行就扫（实例 1 那一行删得掉）：清扫发布那一段摆到二次崩溃。
 #[test]
 fn every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase(
 ) {
@@ -58,11 +61,21 @@ fn every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and
     );
     tally.absorb(&unlinking_the_first_file.tally);
     new_findings.extend(unlinking_the_first_file.new_findings);
+    let deleting_an_instance_table_row =
+        inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
+            UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        );
+    tally.absorb(&deleting_an_instance_table_row.tally);
+    new_findings.extend(deleting_an_instance_table_row.new_findings);
     let rendered = tally.render();
     println!("{rendered}");
     assert!(
         tally.crash_points >= 2,
-        "三段历史里摆出了崩溃状态：{rendered}"
+        "四段历史里摆出了崩溃状态：{rendered}"
+    );
+    assert!(
+        tally.writable_mounts_after_the_crash_deleting_instance_table_rows >= 1,
+        "前提：有一次崩溃后的可写挂载末尾的清扫发布删了行：{rendered}"
     );
     assert_eq!(
         tally.writable_mounts_after_the_crash, tally.crash_points,
diff --git a/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs b/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs
index 8312260..d9b7d0c 100644
--- a/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs
+++ b/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs
@@ -4,9 +4,15 @@
 //! 产物是同目录 `results/out/th-*.out`（判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第二节 S2）。
 //!
 //! 另有三形钉第四种判定（用户 2026-10-04 定「树表读不出算部分没判」、加 `InvariantVerdict::PartiallyJudged`；细则主 agent 定）：
-//! 一条根的树表读不出、其余对得上 ⇒ 部分没判且列出那条根；没有一棵树出现在两个不同的树表单元里 ⇒ 不适用；
-//! 有一处对不上、另有一条根的树表读不出 ⇒ 照报违例。
+//! 一条根的树表读不出、其余对得上 ⇒ 部分没判且列出那条根；一条根的树表读得出而条目比字段表短、其余对得上 ⇒ 同样部分没判且列出那条根；
+//! 没有一棵树出现在两个不同的树表单元里 ⇒ 不适用；有一处对不上、另有一条根的树表读不出 ⇒ 照报违例。
 
+mod common;
+mod common_pointer_head_corruption;
+
+use common_pointer_head_corruption::{
+    node_bytes, replace_the_unit_and_propagate_its_checksum, unit_bytes_at,
+};
 use singlefs_checker::image::{
     InvariantVerdict, TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
 };
@@ -14,6 +20,7 @@ use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
 use singlefs_core::allocator::PoolAllocator;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::checksum::crc32_castagnoli;
 use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
 use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
 use singlefs_core::root_ring::RootRingSlotsPerRegion;
@@ -21,6 +28,7 @@ use singlefs_core::system_configuration::SystemImmutableSizes;
 use singlefs_core::transaction::{
     publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput,
 };
+use singlefs_core::unit::seal_header_checksum;
 use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES};
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
 use singlefs_harness::segments::{FixedGeometry, StepKind};
@@ -180,6 +188,41 @@ fn make_the_tree_table_of_the_version_unreadable(
     }
 }
 
+/// 码 2 节点头里 key 宽那 1 字节的偏移（D18（块里携带什么信息） 已定项 7 的字段表）；k 是它的值，下面几个偏移都按它推。
+const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
+
+fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
+    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
+}
+
+/// 一个码 2 节点的自描述三段（声明长度 2 字节在 8、条目数 2 字节在 82 + 2k、条目宽 2 字节在 84 + 2k，条目区从 86 + 2k + 29 起）
+/// 改成「条目宽 = key 宽、只装一条条目」：条目数改 1、声明长度改 k、largest_key（52 + k 起 k 字节）贴到那一条条目的 key 上、
+/// 条目区之后的字节清零。除了条目宽不等于字段表宽，节点处处自洽（与 `checker_known_bad_images.rs` 里 I-1.10 那份坏镜像同一种改法）。
+fn narrow_the_entry_width_to_the_key_width(bytes: &mut [u8]) {
+    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
+    let key_width_field = u16::try_from(key_width).expect("key 宽装得进 u16");
+    let entries_start = 86 + 2 * key_width + 29;
+    let first_key: Vec<u8> = bytes[entries_start..entries_start + key_width].to_vec();
+    set_u16(bytes, 8, key_width_field);
+    set_u16(bytes, 82 + 2 * key_width, 1);
+    set_u16(bytes, 84 + 2 * key_width, key_width_field);
+    bytes[52 + key_width..52 + 2 * key_width].copy_from_slice(&first_key);
+    for byte in &mut bytes[entries_start + key_width..] {
+        *byte = 0;
+    }
+}
+
+/// 码 2 节点重封：载荷 CRC 罩 [86 + 2k, 节点末)、住 76 + 2k，再封头校验和（头 [0, 86 + 2k)）。
+fn reseal_index_node(bytes: &mut [u8]) {
+    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
+    let header_end = 86 + 2 * key_width;
+    let payload_checksum_offset = 76 + 2 * key_width;
+    let payload_checksum = crc32_castagnoli(&bytes[header_end..]);
+    bytes[payload_checksum_offset..payload_checksum_offset + 4]
+        .copy_from_slice(&payload_checksum.to_le_bytes());
+    seal_header_checksum(bytes, header_end);
+}
+
 /// mkfs → 可写挂载实例 1 → 第一个文件 A → 覆盖写两次（B、C）：三片带条目的树表，同一批树 ID、诞生 txg 都相同。交回镜像与 B 那一版。
 fn first_file_and_two_overwrites() -> (MemoryPool, TransactionOutput) {
     let parameters = parameters_of_the_birth_pools();
@@ -582,6 +625,39 @@ fn a_readable_root_whose_tree_table_is_unreadable_while_the_others_agree_is_repo
     }
 }
 
+/// 第一个文件 A、覆盖写 B、C，再把 B 那条根指着的树表改成「条目宽 = key 宽、只装一条条目」（重封，新的整单元校验和补进 B 的根槽，
+/// B 的根照样自证合法）：B 的树表读得出、条目比字段表短、那一条不解字段；A 与 C 两片树表比得上、诞生 txg 相同——报部分没判，
+/// 说明里列出 B 那条根与「树表有条目比字段表短」。
+#[test]
+fn a_readable_root_whose_tree_table_entries_are_shorter_than_the_field_table_while_the_others_agree_is_reported_as_partially_judged_naming_that_root(
+) {
+    let (mut image, second) = first_file_and_two_overwrites();
+    assert_eq!(
+        birth_txg_verdict(&image),
+        InvariantVerdict::Holds,
+        "改窄之前：三片树表比得上、诞生 txg 相同"
+    );
+    let tree_table_slot = second.root.tree_table.locations[0].slot;
+    let mut tree_table = unit_bytes_at(&image, tree_table_slot, node_bytes());
+    narrow_the_entry_width_to_the_key_width(&mut tree_table);
+    reseal_index_node(&mut tree_table);
+    replace_the_unit_and_propagate_its_checksum(&mut image, tree_table_slot, &tree_table);
+    match birth_txg_verdict(&image) {
+        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
+            unjudged.contains(&format!(
+                "（实例 {}、txg {}）：树表有条目比字段表短",
+                second.root.instance.0, second.root.checkpoint_txg.0
+            )),
+            "列出 B 那条根与为什么没判：{unjudged}"
+        ),
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::Violated(_)) => {
+            panic!("B 的树表条目比字段表短、A 与 C 对得上：I-9.28 报部分没判，得到 {other:?}")
+        }
+    }
+}
+
 /// mkfs → 可写挂载实例 1 → 第一个文件：带条目的树表只有第一个文件那一片（mkfs 与暖机那几条根指着的树表 0 条），
 /// 没有一棵树的条目出现在两个不同的树表单元里——报不适用，不报成立。
 #[test]
````

### mutations-append.tsv

````
第10批 I-9.28 条目比字段表短：比字段表短的条目不列进没判到（部分没判退回成立）	crates/singlefs-checker/src/walk.rs	            TreeTableReading::SomeEntryShorterThanTheFieldTable => Some("树表有条目比字段表短"),	            TreeTableReading::SomeEntryShorterThanTheFieldTable => None,	-p singlefs-harness --test pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root -- a_readable_root_whose_tree_table_entries_are_shorter_than_the_field_table_while_the_others_agree_is_reported_as_partially_judged_naming_that_root	a_readable_root_whose_tree_table_entries_are_shorter_than_the_field_table_while_the_others_agree_is_reported_as_partially_judged_naming_that_root
第10批 I-9.28 条目比字段表短：走树表那一遍不记比字段表短（记成条目都读全）	crates/singlefs-checker/src/walk.rs	            references.tree_table_reading = TreeTableReading::SomeEntryShorterThanTheFieldTable;	            references.tree_table_reading = TreeTableReading::EveryEntryRead;	-p singlefs-harness --test pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root -- a_readable_root_whose_tree_table_entries_are_shorter_than_the_field_table_while_the_others_agree_is_reported_as_partially_judged_naming_that_root	a_readable_root_whose_tree_table_entries_are_shorter_than_the_field_table_while_the_others_agree_is_reported_as_partially_judged_naming_that_root
第10批 坏盘输入：判「一条都没判」时把部分没判算成判过	crates/singlefs-checker-tier/src/bad_disk_input.rs	        InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => true,\n        InvariantVerdict::Holds | InvariantVerdict::Violated(_) => false,	        InvariantVerdict::NotApplicable(_) => true,\n        InvariantVerdict::Holds\n        | InvariantVerdict::Violated(_)\n        | InvariantVerdict::PartiallyJudged(_) => false,	-p singlefs-checker-tier --lib -- bad_disk_input::tests::a_checker_report_of_only_not_applicable_and_partially_judged_verdicts_counts_as_judging_nothing	a_checker_report_of_only_not_applicable_and_partially_judged_verdicts_counts_as_judging_nothing
第10批 清扫两段：一个头都没抹时清扫发布的第一个写记进抹头段（抹头那一段假装摆到了二次崩溃）	crates/singlefs-checker-tier/src/crash_injection.rs	                if phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {	                if true || phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {	-p singlefs-checker-tier --lib -- crash_injection::tests::a_sweep_that_erased_no_head_puts_every_write_after_the_last_publish_before_it_into_the_sweep_publish	a_sweep_that_erased_no_head_puts_every_write_after_the_last_publish_before_it_into_the_sweep_publish
第10批 清扫两段（崩溃注入真流）：一个头都没抹时清扫发布的第一个写记进抹头段	crates/singlefs-checker-tier/src/crash_injection.rs	                if phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {	                if true || phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {	-p singlefs-checker-tier --lib -- crash_injection::tests::a_writable_mount_after_the_crash_whose_sweep_publish_deletes_a_row_is_judged_in_the_sweep_publish_phase_without_a_red_field	a_writable_mount_after_the_crash_whose_sweep_publish_deletes_a_row_is_judged_in_the_sweep_publish_phase_without_a_red_field
第10批 C704：挂载透明不放回清扫发布删的行（只挂载那个比法在清扫发布那一版上判实例表行红）	crates/singlefs-checker-tier/src/crash_injection.rs	            (Cow::Owned(*put_back), Vec::new())	            (Cow::Borrowed(after_the_action), Vec::new())	-p singlefs-checker-tier --lib -- crash_injection::tests::a_writable_mount_after_the_crash_whose_sweep_publish_deletes_a_row_is_judged_in_the_sweep_publish_phase_without_a_red_field	a_writable_mount_after_the_crash_whose_sweep_publish_deletes_a_row_is_judged_in_the_sweep_publish_phase_without_a_red_field
第10批 清扫配置：崩溃注入有一行就扫那一份退回产品路径的阈值占位（崩溃后那次挂载不删行）	crates/singlefs-checker-tier/src/crash_injection.rs	        row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,	        row_count_threshold: InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_writable_mount_after_the_crash_whose_sweep_publish_deletes_a_row_is_judged_in_the_sweep_publish_phase_without_a_red_field	a_writable_mount_after_the_crash_whose_sweep_publish_deletes_a_row_is_judged_in_the_sweep_publish_phase_without_a_red_field
第10批 谱系槽号：回退那次发布不追加谱系记录（固定脚本 D 那一段不写谱系容器 50352 与谱系树根 50354）	crates/singlefs-core/src/transaction.rs	            lineage: LineageStepOfThePublish::AdministratorRollback {\n                record: lineage_record,\n            },	            lineage: LineageStepOfThePublish::NoLineageRecord,	-p singlefs-checker-tier --test crash_enumeration_fixed_script_stream -- every_crash_state_of_the_quick_tier_recovers_to_the_version_its_root_claims	every_crash_state_of_the_quick_tier_recovers_to_the_version_its_root_claims
````

## batch10b-checker-tier

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10b-checker-tier/patch/`
- `crates.patch`：sha256 29ae8ccf937d236af5b7dc6e1efbf307291a05e846b3c76ec110ac81c8d476e4
- `mutations-append.tsv`：sha256 3fee1af928d073157ef8b6b830f426310e7010f315b351923423adaa695634eb
- `mutations-replacements.tsv`：sha256 0b4fac3bfa03efb66e78edfde7514a141090fbfb6ba2cec704756907d4e16367
- `report.md`：sha256 03c1d22d37c93337a02a8553effaa52a7b6f2d93376d55dc5cebb2c44bc201ea

### crates.patch

````
diff --git a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
index fd1e241..7e383bb 100644
--- a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
+++ b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
@@ -17913,7 +17913,10 @@ mod fourth_run_segment_two {
                 | HistoryOperation::RollBackWhileMounted(_)
                 | HistoryOperation::ColdStartRecover
                 | HistoryOperation::Unlink
-                | HistoryOperation::CreateInode => format!("{operation:?}"),
+                | HistoryOperation::CreateInode
+                | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
+                    _,
+                ) => format!("{operation:?}"),
             });
         }
         let source_lines: Vec<&str> = body
@@ -19003,6 +19006,11 @@ mod fourth_run_segment_two {
                 HistoryOperation::CreateInode => {
                     not_applicable("CreateInodeIsNotDrivenByThisExperiment")
                 }
+                // 掉电在 journal 记录之前的那次覆盖写只在留孤儿那一组比重里抽（实例表清扫抹头那一段的取样点），这个装置的格子不用那组比重：
+                // 照前提不满足记。
+                HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
+                    _,
+                ) => not_applicable("OverwriteLosingPowerIsNotDrivenByThisExperiment"),
             }
         }
 
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 09b3d95..8044166 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -48,8 +48,7 @@ use singlefs_core::instance_table_sweep::{
 };
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
 use singlefs_core::mount::{
-    mount_writable_with_instance_table_sweep_configuration, MountError, MountOutput, Mounted,
-    ShadowLedger,
+    mount_writable_with_instance_table_sweep_configuration, MountError, Mounted, ShadowLedger,
 };
 use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
 use singlefs_core::recovery::{
@@ -63,14 +62,22 @@ use singlefs_core::transaction::{
 use singlefs_core::write_accounting::{WritesByStructureKind, WrittenStructureKind};
 
 use crate::crash::check_records_against;
+use crate::writable_mount_phases::writable_mount_phase_of_each_write;
+/// 可写挂载的分段与崩溃镜像落池住在 `crate::writable_mount_phases`（不拿随机历史当输入的用例经那一份引）；崩溃注入的用例照旧从这里引。
+pub use crate::writable_mount_phases::{
+    materialized_crash_image, publishes_before_the_instance_table_sweep_of_the_mount_output,
+    writes_of_the_instance_table_sweep_phases, WritableMountPhase,
+};
 use singlefs_harness::history::{
     choice_from_reproduction_environment, classify_failure, execute_history_with,
-    generate_history_with_weights, newest_ring_root_and_slot_count_of,
-    raised_floor_lands_only_on_abandoned_roots, record_check_aspects,
-    render_environment_assignments, ContentChoice, ContentLength, FailureObservation,
-    FailureSignature, GeneratedHistory, GenerationWeights, HistoryEnding, HistoryExecution,
-    HistoryOperation, HistoryOperationKind, HistorySampling, HistorySeed, HistoryStartingPoint,
-    ReproductionVariablePrefix, SeededRandomSource, StepPosition, KNOWN_RED_FORMS,
+    generate_history_with_weights,
+    history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring,
+    newest_ring_root_and_slot_count_of, raised_floor_lands_only_on_abandoned_roots,
+    record_check_aspects, render_environment_assignments, ContentChoice, ContentLength,
+    FailureObservation, FailureSignature, GeneratedHistory, GenerationWeights, HistoryEnding,
+    HistoryExecution, HistoryOperation, HistoryOperationKind, HistorySampling, HistorySeed,
+    HistoryStartingPoint, ReproductionVariablePrefix, SeededRandomSource, StepPosition,
+    KNOWN_RED_FORMS,
 };
 use singlefs_harness::memory_pool::{
     newest_persisted_root, root_identity_written_by, some_publish_persisted_without_its_root,
@@ -306,51 +313,6 @@ impl CrashPoint {
     }
 }
 
-/// 可写挂载的六段（D23（journal 的角色与格式） 已定项 16 取号、D18（块里携带什么信息） 已定项 11 写行、D16（发布语义） 已定项 8 暖机、
-/// D8（核心索引结构） 已定项 16 第 6 条挂载续做、D18 已定项 11「回收」一条的清扫抹头与清扫发布）：挂载那一段录制流里的一次写落在哪一段。
-#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
-pub enum WritableMountPhase {
-    /// 取号：写行那次发布的第一个单元写或 journal 记录写之前的那几次系统配置槽写。
-    Acquisition,
-    /// 写行那次发布：到它的根槽写、连同紧跟着的系统配置槽轮换为止。
-    RowPublish,
-    /// 写行那次之后的暖机空发布（与挂载在暖机之后推的抬 F：摆二次崩溃时两者算一段，F 那一样按抬 F 的写有没有持久分开判，
-    /// [`FloorRaisesInTheMountWrites::at_a_second_crash`]）。
-    WarmUp,
-    /// 挂载续做：暖机与推的抬 F 之后、交回之前把所选那一版的 `deleted_inodes` 走空的那几步删除发布（所选那一版有删除中的文件时才有）。
-    Continuation,
-    /// 实例表清扫抹头：续做之后、清扫发布之前，FUA 写零那几个未发布单元的头、每块盘一道屏障（`singlefs_core::instance_table_sweep`；
-    /// 挂载末尾清扫了才有）。
-    ErasingHeadsOfTheInstanceTableSweep,
-    /// 实例表清扫发布：删行的那一次专用发布（单元、记录、根槽、轮换）。
-    InstanceTableSweepPublish,
-}
-
-impl WritableMountPhase {
-    pub const ALL: [WritableMountPhase; 6] = [
-        WritableMountPhase::Acquisition,
-        WritableMountPhase::RowPublish,
-        WritableMountPhase::WarmUp,
-        WritableMountPhase::Continuation,
-        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
-        WritableMountPhase::InstanceTableSweepPublish,
-    ];
-
-    #[must_use]
-    pub fn name(self) -> &'static str {
-        match self {
-            WritableMountPhase::Acquisition => "acquisition",
-            WritableMountPhase::RowPublish => "row_publish",
-            WritableMountPhase::WarmUp => "warm_up",
-            WritableMountPhase::Continuation => "continuation",
-            WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep => {
-                "erasing_heads_of_the_instance_table_sweep"
-            }
-            WritableMountPhase::InstanceTableSweepPublish => "instance_table_sweep_publish",
-        }
-    }
-}
-
 /// 第一次崩溃镜像上挂载透明比的是哪个动作之后的池（`singlefs_harness::mount_transparency::MountTransparencyAction` 的前两种）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum MountTransparencyComparison {
@@ -1123,7 +1085,10 @@ fn operation_reads_under_injected_faults(kind: HistoryOperationKind) -> bool {
         | HistoryOperationKind::RaiseRollbackFloor
         | HistoryOperationKind::ColdStartRecover
         | HistoryOperationKind::Unlink
-        | HistoryOperationKind::CreateInode => false,
+        | HistoryOperationKind::CreateInode
+        | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
+            false
+        }
     }
 }
 
@@ -1237,11 +1202,43 @@ pub fn history_whose_writable_mount_after_the_crash_deletes_an_instance_table_ro
 pub fn inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row(
     execution: HistoryExecution,
 ) -> HistoryCrashInjection {
-    let history = history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row();
+    inject_a_crash_at_the_start_of_the_last_step_sweeping_on_any_row(
+        &history_whose_writable_mount_after_the_crash_deletes_an_instance_table_row(),
+        execution,
+    )
+}
+
+/// 在 [`history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`] 最后一步的开头摆一个崩溃状态（摆法同
+/// [`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row`]）：那段历史开头那次覆盖写掉电在
+/// journal 记录之前、留下实例 1 的孤儿，之后实例 2 把实例 1 的根全轮出根环；崩溃后那次可写挂载（有一行就扫）末尾的清扫先抹孤儿的头、
+/// 再发清扫发布删实例 1 那一行，抹头那一段（D18（块里携带什么信息） 已定项 11「回收」一条）的二次崩溃从它来。
+///
+/// # Panics
+/// 同 [`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row`]。
+#[must_use]
+pub fn inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
+    execution: HistoryExecution,
+) -> HistoryCrashInjection {
+    inject_a_crash_at_the_start_of_the_last_step_sweeping_on_any_row(
+        &history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring(),
+        execution,
+    )
+}
+
+/// 在 `history` 最后一步的开头摆一个崩溃状态（更早的写全落、最后一步一个写都没落），崩溃后那次可写挂载与每个二次崩溃镜像上的再挂
+/// 有一行就扫（[`INSTANCE_TABLE_SWEPT_ON_ANY_ROW`]）。最后一步的第一个写落在第几段随历史的写数变：先把这段历史跑一遍、
+/// 按同一条切段规则取出来，不写死段号。
+///
+/// # Panics
+/// 历史没跑到最后一步；最后一步的第一个写不是一段的段首。
+fn inject_a_crash_at_the_start_of_the_last_step_sweeping_on_any_row(
+    history: &GeneratedHistory,
+    execution: HistoryExecution,
+) -> HistoryCrashInjection {
     let step_before_the_last = StepPosition::Operation(history.operations.len() - 2);
     let stream = SharedStream::retaining_contents();
     let mut operations_before_the_last_step = None;
-    let _run = execute_history_with(&history, execution, &stream, &mut |observation| {
+    let _run = execute_history_with(history, execution, &stream, &mut |observation| {
         if observation.position == step_before_the_last {
             operations_before_the_last_step = Some(stream.operation_count());
         }
@@ -1262,7 +1259,7 @@ pub fn inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_d
         .position(|segment| segment.first() == Some(&first_write_of_the_last_step))
         .expect("最后一步的第一个写是一段的段首：上一步发布返回之前那道屏障把它与上一步隔开");
     inject_crashes_into_history_with_the_instance_table_sweep_configuration(
-        &history,
+        history,
         execution,
         CrashPointDraw::ExactlyOneCrashState {
             segment_index,
@@ -1710,18 +1707,6 @@ impl FindingsOfOneHistory {
     }
 }
 
-/// 把一份崩溃镜像（基线 + 这一段里持久了的写）落成一个池：之后的可写挂载在它的副本上写。
-#[must_use]
-pub fn materialized_crash_image(image: &CrashImage<'_>) -> MemoryPool {
-    let mut pool = image.base.clone();
-    for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
-        if *is_persisted {
-            pool.apply_writes(std::slice::from_ref(write));
-        }
-    }
-    pool
-}
-
 /// I-8.4（重放幂等）在这里判的那一条的编号：R-换（用户 2026-10-01 定，`research/prompts/m3-inv-a-r1-main-verification.md` 第四节）——
 /// 在二次崩溃上判「恢复加写行那一整段」幂等，等价关系是读出的内容相同。池级 checker 不判它（单份镜像上恒真），判红时记在这一截的违例里。
 pub const REPLAY_IDEMPOTENCE_INVARIANT: &str = "I-8.4";
@@ -2207,41 +2192,6 @@ pub fn mount_and_publish_once_with_the_instance_table_sweep_configuration_on_the
     }
 }
 
-/// 挂载末尾清扫写了东西时，清扫之前这次挂载发了几次（[`WritableMountOnTheCrashImage::publishes_before_the_instance_table_sweep`]）。
-#[must_use]
-pub fn publishes_before_the_instance_table_sweep_of_the_mount_output(
-    output: &MountOutput,
-) -> Option<usize> {
-    let sweep_wrote_something = match &output.instance_table_sweep {
-        InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
-        | InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
-        | InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::NoRowDeleted(_)) => {
-            false
-        }
-        InstanceTableSweepAtTheEndOfTheMount::Swept(
-            InstanceTableSweepOutcome::RowsDeleted { .. }
-            | InstanceTableSweepOutcome::SweepPublishFailed { .. },
-        ) => true,
-        InstanceTableSweepAtTheEndOfTheMount::Swept(
-            InstanceTableSweepOutcome::SweepPublishRefusedForSpace { heads_erased, .. }
-            | InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
-                heads_erased, ..
-            },
-        ) => !heads_erased.is_empty(),
-    };
-    sweep_wrote_something.then(|| {
-        1 + output.warm_up_publishes.len()
-            + output
-                .space_admission
-                .floor_raises()
-                .iter()
-                .chain(&output.floor_raises_pushed_by_the_continuation)
-                .map(|raised| raised.publishes.len())
-                .sum::<usize>()
-            + output.deletion_queue_continuation.len()
-    })
-}
-
 /// 带文件的一版里第一个文件那条记录还活着（在树里、nlink 不是 0）：覆盖写只写第一个文件，写入口拒删完与正在删的
 /// （`transaction::live_record_of_the_first_file`，D8（核心索引结构） 已定项 16 第 4 条）。unlink 之后再建过 inode 的一版仍归带文件的一版
 /// （`TransactionOutput::is_a_version_with_trees_and_no_file` 为假），它的 `inode_record` 是 None。
@@ -3013,131 +2963,6 @@ fn mount_transparency_violations_of_a_second_crash(
     violations
 }
 
-/// 挂载那一段录制流里每一次写落在挂载的哪一段：先是取号的系统配置槽写，第一个别的写起是写行那次发布；
-/// 一次发布的根槽写与紧跟着的系统配置槽写（轮换、抬 F 那一串先写的新 F）之后的第一个别的写起是下一次发布：
-/// 写行之后、第 `publishes_before_the_deletion_queue_continuation` 个根槽写之前的那几次是暖机（连同挂载推的抬 F），
-/// 从它起是续做（没崩过的那一次挂载被拒、交不出这个数时 None：写行之后都算暖机）；第 `publishes_before_the_instance_table_sweep`
-/// 个根槽写之后起是清扫：其中抹头那一段只有 FUA 写零头所在物理块的单元写（与每块盘一道屏障），第一个别的写（不带 FUA 的单元写、
-/// journal 记录写……）起是清扫发布——一个头都没抹时清扫发布从清扫之前最后一次发布的轮换之后第一个写起，抹头那一段一个写都没有
-/// （没清扫时 None）。
-fn writable_mount_phase_of_each_write(
-    writes: &[RetainedWrite],
-    publishes_before_the_deletion_queue_continuation: Option<usize>,
-    publishes_before_the_instance_table_sweep: Option<usize>,
-) -> Vec<WritableMountPhase> {
-    let phase_of_the_publish_after =
-        |root_records_written: usize, phase_so_far: WritableMountPhase, write: &RetainedWrite| {
-            if publishes_before_the_instance_table_sweep.is_some_and(|publishes_before_the_sweep| {
-                root_records_written > 0 && root_records_written >= publishes_before_the_sweep
-            }) {
-                // 已经在抹头里：往清扫发布换段由下面那几条臂按写的种类判。刚从清扫之前那几段进清扫：头一个写是 FUA 写零（或屏障）才进抹头，
-                // 别的写说明一个头都没抹、直接是清扫发布。
-                if phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {
-                    return WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep;
-                }
-                return match (write.kind, write.is_force_unit_access) {
-                    (StepKind::UnitWrite, true) | (StepKind::Barrier, _) => {
-                        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
-                    }
-                    (StepKind::UnitWrite, false)
-                    | (
-                        StepKind::JournalRecord
-                        | StepKind::RootRecordFua
-                        | StepKind::ZeroFill
-                        | StepKind::SystemConfigurationSlot,
-                        _,
-                    ) => WritableMountPhase::InstanceTableSweepPublish,
-                };
-            }
-            match (
-                root_records_written,
-                publishes_before_the_deletion_queue_continuation,
-            ) {
-                (0, _) => WritableMountPhase::RowPublish,
-                (publishes_landed, Some(publishes_before_the_continuation))
-                    if publishes_landed >= publishes_before_the_continuation =>
-                {
-                    WritableMountPhase::Continuation
-                }
-                (_, Some(_) | None) => WritableMountPhase::WarmUp,
-            }
-        };
-    let mut phase = WritableMountPhase::Acquisition;
-    let mut root_records_written = 0;
-    // 迭代上界是挂载那一段的写数；跨轮带的是这一次写落在哪一段与已写的根槽数；没有提前出口。
-    writes
-        .iter()
-        .map(|write| {
-            phase = match (phase, write.kind) {
-                (WritableMountPhase::Acquisition, StepKind::SystemConfigurationSlot) => {
-                    WritableMountPhase::Acquisition
-                }
-                (
-                    WritableMountPhase::RowPublish
-                    | WritableMountPhase::WarmUp
-                    | WritableMountPhase::Continuation
-                    | WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
-                    | WritableMountPhase::InstanceTableSweepPublish,
-                    StepKind::SystemConfigurationSlot,
-                ) => phase,
-                (
-                    WritableMountPhase::InstanceTableSweepPublish,
-                    StepKind::UnitWrite
-                    | StepKind::JournalRecord
-                    | StepKind::RootRecordFua
-                    | StepKind::ZeroFill
-                    | StepKind::Barrier,
-                )
-                | (
-                    WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
-                    StepKind::JournalRecord,
-                ) => WritableMountPhase::InstanceTableSweepPublish,
-                (WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep, StepKind::UnitWrite)
-                    if !write.is_force_unit_access =>
-                {
-                    WritableMountPhase::InstanceTableSweepPublish
-                }
-                (
-                    WritableMountPhase::Acquisition
-                    | WritableMountPhase::RowPublish
-                    | WritableMountPhase::WarmUp
-                    | WritableMountPhase::Continuation
-                    | WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
-                    StepKind::UnitWrite
-                    | StepKind::JournalRecord
-                    | StepKind::RootRecordFua
-                    | StepKind::ZeroFill
-                    | StepKind::Barrier,
-                ) => phase_of_the_publish_after(root_records_written, phase, write),
-            };
-            if write.kind == StepKind::RootRecordFua {
-                root_records_written += 1;
-            }
-            phase
-        })
-        .collect()
-}
-
-/// 清扫两段（抹头、清扫发布）在挂载那一段写表里各是哪几个写（给 checker 档清扫的用例划段用；划法同 [`writable_mount_phase_of_each_write`]）。
-#[must_use]
-pub fn writes_of_the_instance_table_sweep_phases(
-    writes: &[RetainedWrite],
-    publishes_before_the_instance_table_sweep: usize,
-) -> BTreeMap<WritableMountPhase, Vec<usize>> {
-    let mut by_phase: BTreeMap<WritableMountPhase, Vec<usize>> = BTreeMap::new();
-    for (write_index, phase) in writable_mount_phase_of_each_write(
-        writes,
-        None,
-        Some(publishes_before_the_instance_table_sweep),
-    )
-    .into_iter()
-    .enumerate()
-    {
-        by_phase.entry(phase).or_default().push(write_index);
-    }
-    by_phase
-}
-
 /// 挂载途中的一次二次崩溃：挂载那一段的第几段、段内哪几个写持久了（靶向用例摆的那一个从这一段的第一个写起数，可以越过段尾）、崩在挂载的哪一段。
 #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
 struct SecondCrashPoint {
@@ -5523,7 +5348,9 @@ mod tests {
     /// unlink 它的一段（意图那次 + 最后一批：崩溃状态有落在意图那一版上的，那一版上的可写挂载续做、写出续做那一段，
     /// 之后那一次发布落在删完那一版上）。两段同一种摆法（短段整段枚举），计数并在一起。再并上一段崩溃后那次可写挂载删得了行的历史
     /// （[`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row`]，有一行就扫）：
-    /// 清扫发布那一段从它来。那段历史上没有孤儿、一个头都不抹，抹头那一段从这三段里都摆不出来。
+    /// 清扫发布那一段从它来；那段历史上没有孤儿、一个头都不抹。最后并上一段留下实例 1 的孤儿再把实例 1 的根全轮出根环的历史
+    /// （[`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans`]，同一种摆法）：
+    /// 抹头那一段从它来。
     fn crash_injections_into_histories_reaching_every_mount_phase(
     ) -> (CrashInjectionTally, Vec<HistoryCrashInjection>) {
         let histories = [
@@ -5557,6 +5384,11 @@ mod tests {
                 HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
             ),
         );
+        injections.push(
+            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
+                HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+            ),
+        );
         let mut tally = CrashInjectionTally::default();
         for injection in &injections {
             tally.absorb(&injection.tally);
@@ -5647,6 +5479,71 @@ mod tests {
         );
     }
 
+    /// 崩溃后那次可写挂载末尾的清扫先抹孤儿的头、再发清扫发布删行（D18（块里携带什么信息） 已定项 11「回收」一条，抹头那一段）：
+    /// 挂载做成、删了行、抹了至少一个头；抹头那一段摆了二次崩溃、再挂判了 I-8.4、判了挂载透明；挂载透明一样都没红、二次崩溃之后再挂
+    /// 读出的内容与没崩过的那一次相同、一条新发现都没有。判别力：历史开头那次覆盖写掉电不截写（`singlefs_harness::history` 里
+    /// 掉电那一截的判法）时盘上没有孤儿，「抹了至少一个头」那条先红。
+    #[test]
+    fn a_writable_mount_after_the_crash_whose_sweep_erases_heads_of_orphans_is_judged_in_the_erasing_phase_without_a_red_field(
+    ) {
+        let injection =
+            inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
+                HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+            );
+        let tally = &injection.tally;
+        let rendered = tally.render();
+        assert_eq!(
+            (
+                tally.crash_points,
+                tally.writable_mounts_after_the_crash_succeeded,
+                tally.writable_mounts_after_the_crash_deleting_instance_table_rows
+            ),
+            (1, 1, 1),
+            "前提：摆了一个崩溃状态，那次可写挂载做成、末尾的清扫发布删了行：{rendered}"
+        );
+        assert!(
+            tally.heads_erased_by_the_instance_table_sweep_after_the_crash >= 1,
+            "前提：那次挂载末尾的清扫抹了孤儿的头：{rendered}"
+        );
+        let phase_count = |counts: &BTreeMap<WritableMountPhase, u64>, phase| {
+            counts.get(&phase).copied().unwrap_or(0)
+        };
+        for (what, counts) in [
+            ("摆了二次崩溃", &tally.second_crash_points_by_phase),
+            ("再挂判了 I-8.4", &tally.remount_idempotence_judged_by_phase),
+            (
+                "判了挂载透明",
+                &tally.mount_transparency_judged_on_second_crashes_by_phase,
+            ),
+        ] {
+            assert!(
+                phase_count(
+                    counts,
+                    WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
+                ) >= 1,
+                "抹头那一段{what}：{rendered}"
+            );
+        }
+        assert!(
+            tally.mount_transparency_violations_by_field.is_empty(),
+            "挂载透明判红的字段 {:?}：{rendered}",
+            tally.mount_transparency_violations_by_field
+        );
+        assert_eq!(
+            tally.remounts_after_a_second_crash_reading_different_content, 0,
+            "二次崩溃之后再挂读出的内容与没崩过的那一次不同：{rendered}"
+        );
+        assert!(
+            injection.new_findings.is_empty(),
+            "新发现：{}",
+            injection
+                .new_findings
+                .iter()
+                .map(CrashPointFinding::render)
+                .collect::<String>()
+        );
+    }
+
     /// I-8.4 的 R-换 在真的二次崩溃上不误红：两段历史（[`crash_injections_into_histories_reaching_every_mount_phase`]），
     /// 起点那一段也摆崩溃状态；每个二次崩溃镜像上都再挂了一次，取号、写行、暖机、续做四段各有判过的，
     /// 读出的内容都与没崩过的那一次挂载之后读出的相同（攻方量到实例代号、实例表行、扇区都不同的那种也算相同）。
diff --git a/crates/singlefs-checker-tier/src/lib.rs b/crates/singlefs-checker-tier/src/lib.rs
index c7c8ac7..e936b29 100644
--- a/crates/singlefs-checker-tier/src/lib.rs
+++ b/crates/singlefs-checker-tier/src/lib.rs
@@ -1,5 +1,5 @@
 //! checker 档（D13（验证路线） 已定项 15）：崩溃态枚举引擎与记录核对器（`crash`）、断点续跑与双机分片（`layer0_progress`）、
-//! 崩溃注入（`crash_injection`）与坏盘输入（`bad_disk_input`）两场战役、真设备一侧的设备日志比对（`device_log`）与运行模式（`on_device_modes`），
+//! 崩溃注入（`crash_injection`，可写挂载的分段与崩溃镜像落池在 `writable_mount_phases`）与坏盘输入（`bad_disk_input`）两场战役、真设备一侧的设备日志比对（`device_log`）与运行模式（`on_device_modes`），
 //! 崩溃放量（里程碑二「增补 4」）：崩溃点的身份与复用键（`crash_identity`）、先录后核的流水线（`crash_amplification`：崩溃点计划、路径编号、
 //! 按块录入与核对、按权重分工、对账、门禁读一次）、判定存储（`verdict_store`，RocksDB，特性 `verdict-store`）、
 //! 单元两道校验和上 GPU 与 CPU 对拍（`gpu_unit_checks`，特性 `gpu`）、GPU 判器的判法（`crash_judge_gpu`）与它的派活（`crash_judge_dispatch`，不在判法闭包里）。
@@ -27,3 +27,4 @@ pub mod layer0_progress;
 pub mod on_device_modes;
 #[cfg(feature = "verdict-store")]
 pub mod verdict_store;
+pub mod writable_mount_phases;
diff --git a/crates/singlefs-checker-tier/src/writable_mount_phases.rs b/crates/singlefs-checker-tier/src/writable_mount_phases.rs
new file mode 100644
index 0000000..23cae2a
--- /dev/null
+++ b/crates/singlefs-checker-tier/src/writable_mount_phases.rs
@@ -0,0 +1,232 @@
+//! 可写挂载的分段与崩溃镜像落成池：挂载那一段录制流里每一次写落在挂载的哪一段（[`WritableMountPhase`]）、清扫两段各是哪几个写、
+//! 清扫之前这次挂载发了几次，与把一份崩溃镜像落成一个池。
+//!
+//! 从 `crash_injection` 挪出来：实例表清扫的崩溃点用例（`tests/crash_points_of_the_instance_table_sweep.rs`）只用这几样、不拿随机历史当输入，
+//! 经这一份引就不把随机历史脚手架带进它的判法闭包（`crash_identity` 里登记的崩溃枚举用例那一核，按模块引用收文件）。
+//! 所以这一份不引随机历史（`singlefs_harness::history`），也不引崩溃注入那一份；崩溃注入经它引这几样、原样转出。
+
+use std::collections::BTreeMap;
+
+use singlefs_core::instance_table_sweep::{
+    InstanceTableSweepAtTheEndOfTheMount, InstanceTableSweepOutcome,
+};
+use singlefs_core::mount::MountOutput;
+use singlefs_harness::memory_pool::{CrashImage, MemoryPool, RetainedWrite};
+use singlefs_harness::segments::StepKind;
+
+/// 可写挂载的六段（D23（journal 的角色与格式） 已定项 16 取号、D18（块里携带什么信息） 已定项 11 写行、D16（发布语义） 已定项 8 暖机、
+/// D8（核心索引结构） 已定项 16 第 6 条挂载续做、D18 已定项 11「回收」一条的清扫抹头与清扫发布）：挂载那一段录制流里的一次写落在哪一段。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum WritableMountPhase {
+    /// 取号：写行那次发布的第一个单元写或 journal 记录写之前的那几次系统配置槽写。
+    Acquisition,
+    /// 写行那次发布：到它的根槽写、连同紧跟着的系统配置槽轮换为止。
+    RowPublish,
+    /// 写行那次之后的暖机空发布（与挂载在暖机之后推的抬 F：摆二次崩溃时两者算一段，F 那一样按抬 F 的写有没有持久分开判，
+    /// 崩溃注入里 `FloorRaisesInTheMountWrites::at_a_second_crash`）。
+    WarmUp,
+    /// 挂载续做：暖机与推的抬 F 之后、交回之前把所选那一版的 `deleted_inodes` 走空的那几步删除发布（所选那一版有删除中的文件时才有）。
+    Continuation,
+    /// 实例表清扫抹头：续做之后、清扫发布之前，FUA 写零那几个未发布单元的头、每块盘一道屏障（`singlefs_core::instance_table_sweep`；
+    /// 挂载末尾清扫了才有）。
+    ErasingHeadsOfTheInstanceTableSweep,
+    /// 实例表清扫发布：删行的那一次专用发布（单元、记录、根槽、轮换）。
+    InstanceTableSweepPublish,
+}
+
+impl WritableMountPhase {
+    pub const ALL: [WritableMountPhase; 6] = [
+        WritableMountPhase::Acquisition,
+        WritableMountPhase::RowPublish,
+        WritableMountPhase::WarmUp,
+        WritableMountPhase::Continuation,
+        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
+        WritableMountPhase::InstanceTableSweepPublish,
+    ];
+
+    #[must_use]
+    pub fn name(self) -> &'static str {
+        match self {
+            WritableMountPhase::Acquisition => "acquisition",
+            WritableMountPhase::RowPublish => "row_publish",
+            WritableMountPhase::WarmUp => "warm_up",
+            WritableMountPhase::Continuation => "continuation",
+            WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep => {
+                "erasing_heads_of_the_instance_table_sweep"
+            }
+            WritableMountPhase::InstanceTableSweepPublish => "instance_table_sweep_publish",
+        }
+    }
+}
+
+/// 把一份崩溃镜像（基线 + 这一段里持久了的写）落成一个池：之后的可写挂载在它的副本上写。
+#[must_use]
+pub fn materialized_crash_image(image: &CrashImage<'_>) -> MemoryPool {
+    let mut pool = image.base.clone();
+    for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
+        if *is_persisted {
+            pool.apply_writes(std::slice::from_ref(write));
+        }
+    }
+    pool
+}
+
+/// 挂载末尾清扫写了东西时，清扫之前这次挂载发了几次（崩溃注入里那次可写挂载交回的 `publishes_before_the_instance_table_sweep` 取的就是它）。
+#[must_use]
+pub fn publishes_before_the_instance_table_sweep_of_the_mount_output(
+    output: &MountOutput,
+) -> Option<usize> {
+    let sweep_wrote_something = match &output.instance_table_sweep {
+        InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
+        | InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
+        | InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::NoRowDeleted(_)) => {
+            false
+        }
+        InstanceTableSweepAtTheEndOfTheMount::Swept(
+            InstanceTableSweepOutcome::RowsDeleted { .. }
+            | InstanceTableSweepOutcome::SweepPublishFailed { .. },
+        ) => true,
+        InstanceTableSweepAtTheEndOfTheMount::Swept(
+            InstanceTableSweepOutcome::SweepPublishRefusedForSpace { heads_erased, .. }
+            | InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
+                heads_erased, ..
+            },
+        ) => !heads_erased.is_empty(),
+    };
+    sweep_wrote_something.then(|| {
+        1 + output.warm_up_publishes.len()
+            + output
+                .space_admission
+                .floor_raises()
+                .iter()
+                .chain(&output.floor_raises_pushed_by_the_continuation)
+                .map(|raised| raised.publishes.len())
+                .sum::<usize>()
+            + output.deletion_queue_continuation.len()
+    })
+}
+
+/// 挂载那一段录制流里每一次写落在挂载的哪一段：先是取号的系统配置槽写，第一个别的写起是写行那次发布；
+/// 一次发布的根槽写与紧跟着的系统配置槽写（轮换、抬 F 那一串先写的新 F）之后的第一个别的写起是下一次发布：
+/// 写行之后、第 `publishes_before_the_deletion_queue_continuation` 个根槽写之前的那几次是暖机（连同挂载推的抬 F），
+/// 从它起是续做（没崩过的那一次挂载被拒、交不出这个数时 None：写行之后都算暖机）；第 `publishes_before_the_instance_table_sweep`
+/// 个根槽写之后起是清扫：其中抹头那一段只有 FUA 写零头所在物理块的单元写（与每块盘一道屏障），第一个别的写（不带 FUA 的单元写、
+/// journal 记录写……）起是清扫发布——一个头都没抹时清扫发布从清扫之前最后一次发布的轮换之后第一个写起，抹头那一段一个写都没有
+/// （没清扫时 None）。
+pub(crate) fn writable_mount_phase_of_each_write(
+    writes: &[RetainedWrite],
+    publishes_before_the_deletion_queue_continuation: Option<usize>,
+    publishes_before_the_instance_table_sweep: Option<usize>,
+) -> Vec<WritableMountPhase> {
+    let phase_of_the_publish_after =
+        |root_records_written: usize, phase_so_far: WritableMountPhase, write: &RetainedWrite| {
+            if publishes_before_the_instance_table_sweep.is_some_and(|publishes_before_the_sweep| {
+                root_records_written > 0 && root_records_written >= publishes_before_the_sweep
+            }) {
+                // 已经在抹头里：往清扫发布换段由下面那几条臂按写的种类判。刚从清扫之前那几段进清扫：头一个写是 FUA 写零（或屏障）才进抹头，
+                // 别的写说明一个头都没抹、直接是清扫发布。
+                if phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {
+                    return WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep;
+                }
+                return match (write.kind, write.is_force_unit_access) {
+                    (StepKind::UnitWrite, true) | (StepKind::Barrier, _) => {
+                        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
+                    }
+                    (StepKind::UnitWrite, false)
+                    | (
+                        StepKind::JournalRecord
+                        | StepKind::RootRecordFua
+                        | StepKind::ZeroFill
+                        | StepKind::SystemConfigurationSlot,
+                        _,
+                    ) => WritableMountPhase::InstanceTableSweepPublish,
+                };
+            }
+            match (
+                root_records_written,
+                publishes_before_the_deletion_queue_continuation,
+            ) {
+                (0, _) => WritableMountPhase::RowPublish,
+                (publishes_landed, Some(publishes_before_the_continuation))
+                    if publishes_landed >= publishes_before_the_continuation =>
+                {
+                    WritableMountPhase::Continuation
+                }
+                (_, Some(_) | None) => WritableMountPhase::WarmUp,
+            }
+        };
+    let mut phase = WritableMountPhase::Acquisition;
+    let mut root_records_written = 0;
+    // 迭代上界是挂载那一段的写数；跨轮带的是这一次写落在哪一段与已写的根槽数；没有提前出口。
+    writes
+        .iter()
+        .map(|write| {
+            phase = match (phase, write.kind) {
+                (WritableMountPhase::Acquisition, StepKind::SystemConfigurationSlot) => {
+                    WritableMountPhase::Acquisition
+                }
+                (
+                    WritableMountPhase::RowPublish
+                    | WritableMountPhase::WarmUp
+                    | WritableMountPhase::Continuation
+                    | WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
+                    | WritableMountPhase::InstanceTableSweepPublish,
+                    StepKind::SystemConfigurationSlot,
+                ) => phase,
+                (
+                    WritableMountPhase::InstanceTableSweepPublish,
+                    StepKind::UnitWrite
+                    | StepKind::JournalRecord
+                    | StepKind::RootRecordFua
+                    | StepKind::ZeroFill
+                    | StepKind::Barrier,
+                )
+                | (
+                    WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
+                    StepKind::JournalRecord,
+                ) => WritableMountPhase::InstanceTableSweepPublish,
+                (WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep, StepKind::UnitWrite)
+                    if !write.is_force_unit_access =>
+                {
+                    WritableMountPhase::InstanceTableSweepPublish
+                }
+                (
+                    WritableMountPhase::Acquisition
+                    | WritableMountPhase::RowPublish
+                    | WritableMountPhase::WarmUp
+                    | WritableMountPhase::Continuation
+                    | WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
+                    StepKind::UnitWrite
+                    | StepKind::JournalRecord
+                    | StepKind::RootRecordFua
+                    | StepKind::ZeroFill
+                    | StepKind::Barrier,
+                ) => phase_of_the_publish_after(root_records_written, phase, write),
+            };
+            if write.kind == StepKind::RootRecordFua {
+                root_records_written += 1;
+            }
+            phase
+        })
+        .collect()
+}
+
+/// 清扫两段（抹头、清扫发布）在挂载那一段写表里各是哪几个写（给 checker 档清扫的用例划段用；划法同 [`writable_mount_phase_of_each_write`]）。
+#[must_use]
+pub fn writes_of_the_instance_table_sweep_phases(
+    writes: &[RetainedWrite],
+    publishes_before_the_instance_table_sweep: usize,
+) -> BTreeMap<WritableMountPhase, Vec<usize>> {
+    let mut by_phase: BTreeMap<WritableMountPhase, Vec<usize>> = BTreeMap::new();
+    for (write_index, phase) in writable_mount_phase_of_each_write(
+        writes,
+        None,
+        Some(publishes_before_the_instance_table_sweep),
+    )
+    .into_iter()
+    .enumerate()
+    {
+        by_phase.entry(phase).or_default().push(write_index);
+    }
+    by_phase
+}
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs b/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs
index 836ca95..8886f93 100644
--- a/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs
+++ b/crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs
@@ -4,10 +4,12 @@
 //! 取号、写行、暖机、续做各段各摆一个，每个二次崩溃状态上只读恢复、问模型、池级 checker、记录核对器。
 //! 实现在 `singlefs_checker_tier::crash_injection`；随机崩溃注入的快档与大档（`crash_injection_campaign.rs`）
 //! 走的是同一个入口，这里用两段短历史与一段 unlink 的历史钉「每一截都跑到了、四段都摆到了、一条新发现都没有」，
-//! 再加一段崩溃后那次可写挂载删得了实例表行的历史（有一行就扫）：清扫发布那一段的二次崩溃从它来。
+//! 再加一段崩溃后那次可写挂载删得了实例表行的历史（有一行就扫）：清扫发布那一段的二次崩溃从它来；
+//! 再加一段开头那次覆盖写掉电在 journal 记录之前、留下实例 1 的孤儿的历史（同一种摆法）：抹头那一段的二次崩溃从它来。
 
 use singlefs_checker_tier::crash_injection::{
     inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_deletes_an_instance_table_row,
+    inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans,
     inject_crashes_into_history, CrashInjectionTally, CrashPointDraw, WritableMountPhase,
 };
 use singlefs_core::admission::SpaceAdmission;
@@ -28,7 +30,8 @@ const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
 /// 每个崩溃状态之后都起了可写挂载、发了一次布（删完那一版上建 inode）、跑了 checker，
 /// 挂载途中的二次崩溃取号、写行、暖机、续做四段都摆到了，二次崩溃状态上恢复、问模型、checker、记录核对器各跑了一次；
 /// 两截上一条新发现都没有，挂载与那次发布一次都没被拒。第四段在一段覆盖写到根环里只剩实例 2 的根的历史最后一步开头崩一次，
-/// 那次可写挂载与二次崩溃之后的再挂有一行就扫（实例 1 那一行删得掉）：清扫发布那一段摆到二次崩溃。
+/// 那次可写挂载与二次崩溃之后的再挂有一行就扫（实例 1 那一行删得掉）：清扫发布那一段摆到二次崩溃。第五段同一种摆法，那段历史开头那次
+/// 覆盖写掉电在 journal 记录之前、留下实例 1 的孤儿：清扫先抹它们的头，抹头那一段摆到二次崩溃。
 #[test]
 fn every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase(
 ) {
@@ -67,16 +70,26 @@ fn every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and
         );
     tally.absorb(&deleting_an_instance_table_row.tally);
     new_findings.extend(deleting_an_instance_table_row.new_findings);
+    let erasing_heads_of_orphans =
+        inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans(
+            UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        );
+    tally.absorb(&erasing_heads_of_orphans.tally);
+    new_findings.extend(erasing_heads_of_orphans.new_findings);
     let rendered = tally.render();
     println!("{rendered}");
     assert!(
         tally.crash_points >= 2,
-        "四段历史里摆出了崩溃状态：{rendered}"
+        "五段历史里摆出了崩溃状态：{rendered}"
     );
     assert!(
         tally.writable_mounts_after_the_crash_deleting_instance_table_rows >= 1,
         "前提：有一次崩溃后的可写挂载末尾的清扫发布删了行：{rendered}"
     );
+    assert!(
+        tally.heads_erased_by_the_instance_table_sweep_after_the_crash >= 1,
+        "前提：有一次崩溃后的可写挂载末尾的清扫抹了孤儿的头：{rendered}"
+    );
     assert_eq!(
         tally.writable_mounts_after_the_crash, tally.crash_points,
         "每个崩溃状态之后都起一次可写挂载：{rendered}"
diff --git a/crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs b/crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs
index 8cac46f..1ab6b5e 100644
--- a/crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs
+++ b/crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs
@@ -1,4 +1,4 @@
-//! checker 档模块：crash、layer0_progress、crash_injection
+//! checker 档模块：crash、layer0_progress、writable_mount_phases
 //! 实例表清扫的崩溃点（D18（块里携带什么信息） 已定项 11「回收」一条；判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md`
 //! 第三节 F5 与第二节 Q5；C597（实例表行只增不删，删行的条件没有条款） 的判别力自证）。
 //!
@@ -7,7 +7,7 @@
 //! 抹孤儿的头、每块盘一道屏障、发清扫发布删掉实例 1 那一行。基线是实例 3 那次挂载之前的盘面，枚举的是那次挂载的全部写。
 //!
 //! 比已有的流多罩的崩溃状态：已有的层 0 流与崩溃注入里没有一次挂载写过清扫的抹头与清扫发布（产品路径的阈值占位 369 行，那几条流到不了）；
-//! 这一条的挂载末尾两段（`crash_injection::WritableMountPhase` 的抹头与清扫发布）上每个崩溃状态都枚举。
+//! 这一条的挂载末尾两段（`writable_mount_phases::WritableMountPhase` 的抹头与清扫发布）上每个崩溃状态都枚举。
 //! 多跑的一步：清扫两段上的每个崩溃状态再起一次可写挂载（也是有一行就扫），判挂载做成、池级 checker 不红、F5 照判。
 //!
 //! F5（判决第三节 F5，C597 的验收）：删行那一版生效的每个崩溃态里，被删实例按删前那一行判未发布的单元一个不剩——行一删，盘上再没有东西说它们
@@ -28,11 +28,12 @@ use singlefs_checker_tier::crash::{
     enumerate_layer0_in_state_slices, full_expansion, quick_tier_expansion, Layer0ObserverCounts,
     Layer0Parallelism, Layer0SegmentExpansion, Layer0Tally,
 };
-use singlefs_checker_tier::crash_injection::{
+use singlefs_checker_tier::layer0_progress::Layer0Resume;
+// 经 `writable_mount_phases` 引、不经 `crash_injection`：这条流是手写的、不拿随机历史当输入，不该把随机历史脚手架带进判法闭包。
+use singlefs_checker_tier::writable_mount_phases::{
     materialized_crash_image, publishes_before_the_instance_table_sweep_of_the_mount_output,
     writes_of_the_instance_table_sweep_phases, WritableMountPhase,
 };
-use singlefs_checker_tier::layer0_progress::Layer0Resume;
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::allocator::UnitAreaStart;
diff --git a/crates/singlefs-harness/src/fault_injection.rs b/crates/singlefs-harness/src/fault_injection.rs
index fc19fc5..3306a61 100644
--- a/crates/singlefs-harness/src/fault_injection.rs
+++ b/crates/singlefs-harness/src/fault_injection.rs
@@ -1810,9 +1810,10 @@ fn system_configuration_instance_before_a_mount_step(
     match segment {
         FaultedSegment::TheStartingPoint => Some(MKFS_INSTANCE_GENERATION),
         FaultedSegment::Operation { operation_kind, .. } => match operation_kind {
-            // 崩溃恢复抛弃根也是一次可写挂载：取号、写行、暖机。
+            // 崩溃恢复抛弃根也是一次可写挂载：取号、写行、暖机；掉电在 journal 记录之前的那次覆盖写之后也挂一次（掉电那一截不写系统配置）。
             HistoryOperationKind::CloseAndMountWritable
-            | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => {
+            | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
+            | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
                 let index = marks
                     .iter()
                     .position(|mark| mark.position == segment.position())?;
@@ -2509,7 +2510,10 @@ fn inject_one_fault(
             | HistoryOperation::ColdStartRecover
             | HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
             | HistoryOperation::Unlink
-            | HistoryOperation::CreateInode => None,
+            | HistoryOperation::CreateInode
+            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
+                None
+            }
         });
     let (read_back_root, read_back_content): (Option<ModelRootKey>, Option<&[u8]>) =
         match &read_back {
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index f109d09..62404cf 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -17,6 +17,7 @@ use std::cell::{Cell, RefCell};
 use std::collections::{BTreeMap, BTreeSet};
 use std::fmt::Write as _;
 use std::panic::{catch_unwind, AssertUnwindSafe};
+use std::rc::Rc;
 use std::sync::atomic::{AtomicU64, Ordering};
 use std::sync::{Mutex, Once};
 
@@ -51,7 +52,9 @@ use singlefs_core::transaction::{
     PoolWriter, PublishError, TransactionOutput, ZeroUnitPublishPlan,
 };
 use singlefs_core::unit::data_unit_payload_capacity;
-use singlefs_format::{DATA_UNIT_BYTES, EXTENT_TREE_LOWER_LEAF_DATA_UNITS, SLOT_BYTES};
+use singlefs_format::{
+    DATA_UNIT_BYTES, EXTENT_TREE_LOWER_LEAF_DATA_UNITS, JOURNAL_RING_START_SLOT, SLOT_BYTES,
+};
 
 use crate::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
 use crate::memory_pool::{MemoryPool, RecordCheck, SparseBlockDevice};
@@ -496,6 +499,13 @@ pub enum HistoryOperation {
     /// 删完之后只许建 inode、写行与空发布）。只在第一个文件已删完的一版上调（有树而文件集合为空的一版、或在它上面建过 inode 的一版）：
     /// 模型只罩 inode 1 一个文件，建在第一个文件旁边的 inode 会让之后那次最后一批留下别的 inode，模型的删除那一臂罩不到。
     CreateInode,
+    /// 经挂着的会话整份重写文件内容（入口分法同 [`HistoryOperation::PublishOverwrite`]），两块盘在那次改动的第一个 journal 记录写
+    /// 到达时掉电：之前发出的写与屏障都落了盘，从那个写起一个都不落、都报 I/O 错（这一份里的 `DeviceLosingPowerAtTheFirstJournalRecordWrite`），
+    /// 这个进程随之崩掉；之后照 [`HistoryOperation::CloseAndMountWritable`] 可写挂载一次。那次改动写下的单元是这个会话那个实例的孤儿：
+    /// 下一个实例写行时给它写的那一行按所选根判它们未发布，根环里这个实例的根全被轮出去之后，挂载末尾的实例表清扫抹它们的头、
+    /// 再发清扫发布删那一行（D18（块里携带什么信息） 已定项 11「回收」一条：抹头那一段只有这样的孤儿走得到）。
+    /// 掉电截在第一条 journal 记录之前，那次改动没有任何一样能让恢复读到它的内容：交给模型与 RefFS 时与可写挂载相同。
+    OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(ContentChoice),
 }
 
 /// 操作的种类：统计与比重用。
@@ -511,10 +521,11 @@ pub enum HistoryOperationKind {
     CrashRecoveryAbandoningTheNewestRoot,
     Unlink,
     CreateInode,
+    OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,
 }
 
 impl HistoryOperationKind {
-    pub const ALL: [HistoryOperationKind; 10] = [
+    pub const ALL: [HistoryOperationKind; 11] = [
         HistoryOperationKind::PublishFirstFile,
         HistoryOperationKind::PublishOverwrite,
         HistoryOperationKind::PublishWithoutUnits,
@@ -525,6 +536,7 @@ impl HistoryOperationKind {
         HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
         HistoryOperationKind::Unlink,
         HistoryOperationKind::CreateInode,
+        HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,
     ];
 }
 
@@ -544,6 +556,9 @@ impl HistoryOperation {
             }
             HistoryOperation::Unlink => HistoryOperationKind::Unlink,
             HistoryOperation::CreateInode => HistoryOperationKind::CreateInode,
+            HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
+                HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
+            }
         }
     }
 }
@@ -583,6 +598,8 @@ pub enum NamedGenerationWeights {
     TowardTheUnitAreaWall,
     /// [`GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES`]。
     DeletingFilesOfSeveralLowerLeaves,
+    /// [`GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP`]。
+    LeavingOrphansForTheInstanceTableSweep,
 }
 }
 
@@ -607,6 +624,9 @@ impl NamedGenerationWeights {
             NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves => {
                 GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES
             }
+            NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep => {
+                GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP
+            }
         }
     }
 
@@ -620,6 +640,7 @@ impl NamedGenerationWeights {
             NamedGenerationWeights::TowardTheAllocationRecordWall => "wall",
             NamedGenerationWeights::TowardTheUnitAreaWall => "unit-area-wall",
             NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves => "several-leaves",
+            NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep => "orphans",
         }
     }
 }
@@ -900,6 +921,32 @@ impl GenerationWeights {
         overwrite_contents: OverwriteContentDraw::AlsoSpanningSeveralDataUnits,
     };
 
+    /// 实例表清扫抹头那一段（D18（块里携带什么信息） 已定项 11「回收」一条）的取样点：一律从第一个文件起，带文件的版本上
+    /// 偶尔一次覆盖写掉电在第一个 journal 记录写之前、进程崩掉再挂（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]，
+    /// 留下那个实例的孤儿），其余绝大多数是覆盖写（每次一条根，把孤儿那个实例的根从 24 槽的根环里轮出去），夹着少量可写挂载；
+    /// 不抽回退、抬 F、冷启动与 unlink。别的几组比重不抽这一步：它们的种子钉着的取样点照旧。
+    pub const LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP: GenerationWeights = GenerationWeights {
+        name: "偏向留下孤儿实例（清扫抹头那一段的取样点）",
+        named: NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep,
+        starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
+        with_session_closed: &[(HistoryOperationKind::CloseAndMountWritable, 100)],
+        with_session_open_without_file: &[
+            (HistoryOperationKind::PublishFirstFile, 90),
+            (HistoryOperationKind::CloseAndMountWritable, 10),
+        ],
+        with_session_open_with_file: &[
+            (HistoryOperationKind::PublishOverwrite, 88),
+            (
+                HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,
+                8,
+            ),
+            (HistoryOperationKind::CloseAndMountWritable, 4),
+        ],
+        with_session_open_after_the_file_was_deleted: None,
+        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
+        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
+    };
+
     /// 复现入口认得的每一组比重：按 `environment_value` 找回（[`HistorySampling::from_environment`]）。
     /// 照 [`NamedGenerationWeights::ALL`] 逐个经 [`NamedGenerationWeights::weights`] 生成，不手抄常量名：
     /// 新加一组比重要一个新成员，成员不在 `weights` 的穷举 `match` 里补一臂就编不过（代码三方 test-script-align-code-r1 判决 K4）。
@@ -1069,6 +1116,16 @@ fn draw_operation(
         }
         HistoryOperationKind::Unlink => HistoryOperation::Unlink,
         HistoryOperationKind::CreateInode => HistoryOperation::CreateInode,
+        HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
+            HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
+                match overwrite_contents {
+                    OverwriteContentDraw::WithinOneDataUnitOrJustBeyond => draw_content(source),
+                    OverwriteContentDraw::AlsoSpanningSeveralDataUnits => {
+                        draw_content_spanning_several_data_units(source)
+                    }
+                },
+            )
+        }
     }
 }
 
@@ -1087,15 +1144,19 @@ fn expected_session_after(
     kind: HistoryOperationKind,
 ) -> ExpectedSession {
     match kind {
-        // 崩溃恢复抛弃根也是一次可写挂载：落到的前一条根带不带文件，生成时照「这段历史发过文件没有」估，与可写挂载同一个估法。
+        // 崩溃恢复抛弃根也是一次可写挂载：落到的前一条根带不带文件，生成时照「这段历史发过文件没有」估，与可写挂载同一个估法；
+        // 掉电在 journal 记录之前的那次覆盖写之后也是一次可写挂载，落到的是那次改动之前的一版。
         HistoryOperationKind::CloseAndMountWritable
-        | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => match first_file {
-            ExpectedFirstFile::Written => ExpectedSession::OpenWithFile,
-            ExpectedFirstFile::NotWrittenYet => ExpectedSession::OpenWithoutFile,
-            ExpectedFirstFile::DeletedAndFollowedByItsOwnTable => {
-                ExpectedSession::OpenAfterTheFileWasDeleted
+        | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
+        | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
+            match first_file {
+                ExpectedFirstFile::Written => ExpectedSession::OpenWithFile,
+                ExpectedFirstFile::NotWrittenYet => ExpectedSession::OpenWithoutFile,
+                ExpectedFirstFile::DeletedAndFollowedByItsOwnTable => {
+                    ExpectedSession::OpenAfterTheFileWasDeleted
+                }
             }
-        },
+        }
         HistoryOperationKind::ColdStartRecover => ExpectedSession::Closed,
         HistoryOperationKind::PublishFirstFile => match expected {
             ExpectedSession::OpenWithoutFile => ExpectedSession::OpenWithFile,
@@ -1210,6 +1271,44 @@ pub fn generate_history_with_weights(
     }
 }
 
+/// [`history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`] 里掉电之后的那次可写挂载（实例 2）之后覆盖写几次。
+/// 根环 24 槽（`RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM` 每区 8 槽、三区）里实例 1 的根要全被轮出去：实例 2 的写行、两次暖机与
+/// 之后 24 次覆盖写一共 27 条根；第 25 次是崩溃注入摆崩溃状态的那一步。
+pub const OVERWRITES_AFTER_THE_ORPHANS_OF_THE_FIRST_INSTANCE: u64 = 25;
+
+/// 留下实例 1 的孤儿、再把实例 1 的根全轮出根环的一段写死的历史：从第一个文件起（实例 1 的会话开着），一次覆盖写掉电在它的第一个
+/// journal 记录写之前、进程崩掉再可写挂载成实例 2（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]：
+/// 写行把实例 1 那一行写进表，那次覆盖写写下的单元按这一行判未发布），再覆盖写 [`OVERWRITES_AFTER_THE_ORPHANS_OF_THE_FIRST_INSTANCE`] 次。
+/// 跑完之后（或崩在最后一次覆盖写之前）的盘面上，有一行就扫的可写挂载末尾清扫删实例 1 那一行、先抹它的孤儿的头
+/// （D18（块里携带什么信息） 已定项 11「回收」一条）：抹头那一段从这里走得到。
+#[must_use]
+pub fn history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring(
+) -> GeneratedHistory {
+    let mut operations = vec![
+        HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
+            ContentChoice {
+                length: ContentLength::InsideOneDataUnit { selector: 0 },
+                fill_seed: 0,
+            },
+        ),
+    ];
+    operations.extend(
+        (1..=OVERWRITES_AFTER_THE_ORPHANS_OF_THE_FIRST_INSTANCE).map(|overwrite| {
+            HistoryOperation::PublishOverwrite(ContentChoice {
+                length: ContentLength::InsideOneDataUnit {
+                    selector: overwrite,
+                },
+                fill_seed: overwrite,
+            })
+        }),
+    );
+    GeneratedHistory {
+        seed: HistorySeed(3),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations,
+    }
+}
+
 /// 这个进程里开着的可写会话：分配器、现行版本、实例代号，都是挂载（或起点那个进程）交回的。
 struct WritableSession {
     allocator: PoolAllocator,
@@ -2393,7 +2492,11 @@ impl HistoryTally {
             | HistoryOperation::ColdStartRecover
             | HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
             | HistoryOperation::Unlink
-            | HistoryOperation::CreateInode => None,
+            | HistoryOperation::CreateInode
+            // 那次改动掉电在 journal 记录之前、一次都没发布成：内容长度不记进「进过入口」「发布过」（结局是那之后可写挂载的结局）。
+            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
+                None
+            }
         };
         let entrance_was_called = match outcome {
             StepOutcome::Applied(_) | StepOutcome::Refused { .. } => true,
@@ -4384,6 +4487,171 @@ fn apply_crash_recovery_abandoning_the_newest_root(
     )
 }
 
+/// 几块盘此刻通没通电（[`DeviceLosingPowerAtTheFirstJournalRecordWrite`] 的几块盘共用一份）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum PowerOfTheDevices {
+    On,
+    /// 某块盘上第一个落在 journal 环里的写到达时掉的电：那个写与之后每一个写、写零、屏障都不落盘。
+    LostAtTheFirstJournalRecordWrite,
+}
+
+/// 第一个 journal 记录写到达时整池掉电的一块盘（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`] 的造法）：
+/// 之前的写与屏障原样交给里面那块盘（录制流照常记下、落盘）；落在 journal 环里的第一个写把几块盘共用的那份电源状态翻成掉电，
+/// 从它起每一个写、写零与屏障都不交下去、报 I/O 错——录制流里因此只有落了盘的那一串，与崩溃注入截在那个写之前的盘面相同。
+/// 读照样交下去：掉电之后读到的就是落了盘的字节。
+struct DeviceLosingPowerAtTheFirstJournalRecordWrite<
+    Device: singlefs_core::block_device::BlockDevice,
+> {
+    inner: Device,
+    /// journal 环在每块盘上的字节区间（`singlefs_format::JOURNAL_RING_START_SLOT` 起、`journal_ring_bytes` 长；与
+    /// `crate::segments::FixedGeometry::classify` 认 journal 记录写的那一臂同一个区间）。
+    journal_ring_offsets_in_bytes: std::ops::Range<u64>,
+    power: Rc<Cell<PowerOfTheDevices>>,
+}
+
+impl<Device: singlefs_core::block_device::BlockDevice>
+    DeviceLosingPowerAtTheFirstJournalRecordWrite<Device>
+{
+    fn error_after_the_power_loss() -> BlockDeviceError {
+        BlockDeviceError::InputOutput(std::io::Error::other(
+            "掉电：第一个 journal 记录写到达时整池断电，这个写与之后的写、屏障都没落盘",
+        ))
+    }
+
+    fn refuse_once_the_power_is_lost(&self) -> Result<(), BlockDeviceError> {
+        match self.power.get() {
+            PowerOfTheDevices::On => Ok(()),
+            PowerOfTheDevices::LostAtTheFirstJournalRecordWrite => {
+                Err(Self::error_after_the_power_loss())
+            }
+        }
+    }
+}
+
+impl<Device: singlefs_core::block_device::BlockDevice> singlefs_core::block_device::BlockDevice
+    for DeviceLosingPowerAtTheFirstJournalRecordWrite<Device>
+{
+    fn read_at(
+        &self,
+        offset: singlefs_core::address::DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        self.inner.read_at(offset, buffer)
+    }
+
+    fn write_at(
+        &mut self,
+        offset: singlefs_core::address::DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: singlefs_core::block_device::WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        self.refuse_once_the_power_is_lost()?;
+        if self.journal_ring_offsets_in_bytes.contains(&offset.0) {
+            self.power
+                .set(PowerOfTheDevices::LostAtTheFirstJournalRecordWrite);
+            return Err(Self::error_after_the_power_loss());
+        }
+        self.inner.write_at(offset, bytes, durability)
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: singlefs_core::address::DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.refuse_once_the_power_is_lost()?;
+        self.inner.write_zeroes_at(offset, length)
+    }
+
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        self.refuse_once_the_power_is_lost()?;
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
+/// [`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]：前提同覆盖写（[`apply_publish_overwrite`]），
+/// 经挂着的会话发那次整份重写，盘在它的第一个 journal 记录写到达时掉电（[`DeviceLosingPowerAtTheFirstJournalRecordWrite`]）。
+/// 会话的结局不判：进程已经在这次改动里崩掉，没有人收这个结局（掉电之前做完的那几步落了盘、录制流里有它们）。会话连同模型那一侧的会话
+/// 一起丢掉，之后照可写挂载那一步（[`apply_mount_writable`]）挂一次、交模型判：模型看不见这次掉电——那次改动一样都没发布，
+/// 挂载恢复落到的就是改动之前那一版，与这个进程正常退出之后的可写挂载同一个答案。
+fn apply_overwrite_losing_power_before_its_journal_record_then_mount_writable(
+    pool: &mut HistoryPool,
+    parameters: &MakeFilesystemParameters,
+    content_choice: &ContentChoice,
+    write_time_seconds: u64,
+) -> AppliedStep {
+    let Some(open_session) = pool.session.as_ref() else {
+        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
+    };
+    let Some(current_file_version) = open_session.current.file_version() else {
+        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
+    };
+    if current_file_version.inode_record.is_none() {
+        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile);
+    }
+    let WritableSession {
+        allocator,
+        current,
+        instance,
+        shadow_ledger,
+        parameters_and_device_table,
+        ..
+    } = pool.session.take().expect("上面判过这个进程的会话开着");
+    let mut session_that_loses_power = MountedSession {
+        allocator,
+        current,
+        instance,
+        shadow_ledger,
+        parameters_and_device_table,
+    };
+    let content = content_choice.bytes();
+    let file = FirstFile {
+        content: &content,
+        write_time_seconds,
+    };
+    let power = Rc::new(Cell::new(PowerOfTheDevices::On));
+    let journal_ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
+    let journal_ring_offsets_in_bytes =
+        journal_ring_start..journal_ring_start + parameters.geometry.journal_ring_bytes;
+    let mut devices_losing_power: Vec<(
+        DeviceIdentity,
+        DeviceLosingPowerAtTheFirstJournalRecordWrite<HistoryDevice>,
+    )> = std::mem::take(&mut pool.devices)
+        .into_iter()
+        .map(|(identity, device)| {
+            (
+                identity,
+                DeviceLosingPowerAtTheFirstJournalRecordWrite {
+                    inner: device,
+                    journal_ring_offsets_in_bytes: journal_ring_offsets_in_bytes.clone(),
+                    power: Rc::clone(&power),
+                },
+            )
+        })
+        .collect();
+    let _outcome_nobody_receives_after_the_crash = session_that_loses_power.publish_user_change(
+        &mut devices_losing_power,
+        match content_choice.length.whole_file_write_entry() {
+            WholeFileWriteEntry::Overwrite => UserChange::Overwrite(file),
+            WholeFileWriteEntry::SequentialWrite => UserChange::SequentialWrite(file),
+        },
+    );
+    pool.devices = devices_losing_power
+        .into_iter()
+        .map(|(identity, device)| (identity, device.inner))
+        .collect();
+    pool.model.close_session();
+    apply_mount_writable(pool, parameters)
+}
+
 fn apply_rollback_while_mounted(
     pool: &mut HistoryPool,
     parameters: &MakeFilesystemParameters,
@@ -4697,6 +4965,14 @@ fn apply_operation(
         }
         HistoryOperation::Unlink => apply_unlink(pool, write_time_seconds),
         HistoryOperation::CreateInode => apply_create_inode(pool, write_time_seconds),
+        HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(content) => {
+            apply_overwrite_losing_power_before_its_journal_record_then_mount_writable(
+                pool,
+                &parameters,
+                content,
+                write_time_seconds,
+            )
+        }
     }
 }
 
@@ -5614,6 +5890,11 @@ fn simpler_variants(operation: HistoryOperation) -> Vec<HistoryOperation> {
         HistoryOperation::PublishOverwrite(content) => simpler_contents(content)
             .map(HistoryOperation::PublishOverwrite)
             .collect(),
+        HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(content) => {
+            simpler_contents(content)
+                .map(HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable)
+                .collect()
+        }
         HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
             index_from_newest,
         }) => smaller_selectors(index_from_newest)
@@ -6061,6 +6342,129 @@ fn campaign_report_of_finished_histories(
 mod tests {
     use super::*;
 
+    /// 留孤儿那一组比重抽得到「覆盖写掉电在 journal 记录之前再挂」，别的每一组比重一次都抽不到（它们的种子钉着的取样点照旧）：
+    /// 32 个种子、每段 40 步。判别力：留孤儿那一组里这一步的份数是 0 时第一条红。
+    #[test]
+    fn the_orphan_sampling_draws_the_overwrite_losing_power_and_the_broad_sampling_never_does() {
+        let kind =
+            HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable;
+        let steps_drawn_with = |weights: &GenerationWeights| {
+            (0..32u64)
+                .map(|seed_offset| {
+                    generate_history_with_weights(
+                        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset),
+                        40,
+                        weights,
+                    )
+                    .operations
+                    .iter()
+                    .filter(|operation| operation.kind() == kind)
+                    .count()
+                })
+                .sum::<usize>()
+        };
+        assert!(
+            steps_drawn_with(&GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP) >= 1,
+            "留孤儿那一组比重 32 段里一次都没抽到 {kind:?}"
+        );
+        for weights in GenerationWeights::NAMED {
+            if weights.named == NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep {
+                continue;
+            }
+            assert_eq!(
+                steps_drawn_with(&weights),
+                0,
+                "{} 那一组比重抽到了 {kind:?}",
+                weights.name
+            );
+        }
+    }
+
+    /// 掉电在 journal 记录之前的那次覆盖写留下实例 1 的孤儿（D18（块里携带什么信息） 已定项 11「回收」一条）：这段写死的历史
+    /// （[`history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`]）每一步都做成，每一步之后池级 checker 与模型都不红，
+    /// 掉电之后那次可写挂载取到实例 2；之后在跑完的盘面上起一次有一行就扫的可写挂载，清扫删掉实例 1 那一行、抹了孤儿的头。
+    /// 判别力：掉电那一截不截写（那次覆盖写整次发布成）时模型那一判先红（挂载落到的不是它答的那一版），盘上也没有孤儿、清扫一个头都不抹。
+    #[test]
+    fn an_overwrite_losing_power_before_its_journal_record_leaves_orphans_the_sweep_erases_once_its_instance_left_the_ring(
+    ) {
+        use singlefs_core::instance_table_sweep::{
+            InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
+            InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
+            SweepPublishPlacements,
+        };
+        let history =
+            history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring();
+        let execution = HistoryExecution::CHECKED_ON_FOUR_GIBIBYTE_DEVICES;
+        let stream = SharedStream::retaining_contents();
+        let mut image_after_the_last_step = None;
+        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
+            image_after_the_last_step = Some(observation.image.clone());
+        });
+        assert_eq!(
+            run.ending,
+            HistoryEnding::Completed,
+            "这段历史每一步都做成、checker 与模型都不红：{:?}",
+            run.outcomes
+        );
+        assert!(
+            matches!(
+                run.outcomes.first(),
+                Some(StepOutcome::Applied(AppliedEffect::Mounted {
+                    instance: InstanceGeneration(2),
+                    ..
+                }))
+            ),
+            "掉电之后那次可写挂载做成、取到实例 2：{:?}",
+            run.outcomes.first()
+        );
+        let image = image_after_the_last_step.expect("历史跑完，观察者至少被调过一次");
+        let parameters = execution.device_width.parameters();
+        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
+            .devices
+            .iter()
+            .map(|(identity, sectors)| {
+                let mut device = SparseBlockDevice::new(
+                    image.device_size_in_bytes,
+                    PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
+                );
+                device.image = sectors.clone();
+                (*identity, device)
+            })
+            .collect();
+        let mounted = singlefs_core::mount::mount_writable_with_instance_table_sweep_configuration(
+            &parameters,
+            &mut devices,
+            SpaceAdmission::JudgedByTheFormula,
+            ShadowLedger::On,
+            InstanceTableSweepConfiguration {
+                switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
+                row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
+                sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
+            },
+        )
+        .expect("跑完的盘面上可写挂载做得成");
+        let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
+            rows_deleted,
+            heads_erased,
+        }) = &mounted.output.instance_table_sweep
+        else {
+            panic!("清扫删了行：{:?}", mounted.output.instance_table_sweep);
+        };
+        assert_eq!(
+            rows_deleted
+                .iter()
+                .map(|row| row.instance)
+                .collect::<Vec<_>>(),
+            vec![InstanceGeneration(1)],
+            "清扫删的是实例 1 那一行"
+        );
+        assert!(
+            !heads_erased.is_empty(),
+            "清扫抹了实例 1 的孤儿的头：{:?}",
+            mounted.output.instance_table_sweep
+        );
+    }
+
     /// SplitMix64 的参考输出（Vigna 的 splitmix64.c，种子 0）：手写的实现逐位对得上，换一个混合常数这里就红。
     #[test]
     fn seeded_random_source_reproduces_the_splitmix64_reference_sequence() {
diff --git a/crates/singlefs-harness/src/reference_filesystem_translation.rs b/crates/singlefs-harness/src/reference_filesystem_translation.rs
index 731833f..4ca58d0 100644
--- a/crates/singlefs-harness/src/reference_filesystem_translation.rs
+++ b/crates/singlefs-harness/src/reference_filesystem_translation.rs
@@ -404,9 +404,13 @@ impl ReferenceFileSystemTranslation {
             // 可写挂载与抬 F 不改内容：不进 RefFS 的操作序列（H1 只让发布调用、fsync、回退抬 lo），它们写的根映到现行版本。
             // 建 inode 只接在第一个文件删完之后（`HistoryOperation::CreateInode`），翻译到 unlink 写了盘那一步就已经停了；
             // 它也不改第一个文件的内容（新建的 inode 长度 0），照可写挂载与抬 F 那样不进操作序列。
+            // 掉电在第一个 journal 记录写之前的那次覆盖写一样都没发布成（盘上没有能让恢复读到它的记录与根），之后那次可写挂载照可写挂载。
             HistoryOperation::CloseAndMountWritable
             | HistoryOperation::RaiseRollbackFloor(_)
-            | HistoryOperation::CreateInode => EventsOfAStep::nothing(),
+            | HistoryOperation::CreateInode
+            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
+                EventsOfAStep::nothing()
+            }
             HistoryOperation::ColdStartRecover => match &seen.ending {
                 StepEndingSeenByTheTranslation::ReturnedOk {
                     read_out_of_the_recovery_in_the_step: Some(read_out),
diff --git a/crates/singlefs-harness/tests/random_histories.rs b/crates/singlefs-harness/tests/random_histories.rs
index 0b54731..f6b4a88 100644
--- a/crates/singlefs-harness/tests/random_histories.rs
+++ b/crates/singlefs-harness/tests/random_histories.rs
@@ -194,6 +194,16 @@ fn assert_every_path_was_exercised(tally: &HistoryTally) {
             HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => {
                 assert!(operation.refused >= 1, "{kind:?} 一次都没被拒过");
             }
+            // 快档那组比重不抽它（种子钉着的取样点照旧）：它只在留孤儿那一组比重里抽，跑到与留下孤儿由 `singlefs_harness::history`
+            // 的单元测试钉（`the_orphan_sampling_draws_the_overwrite_losing_power_and_the_broad_sampling_never_does` 与
+            // `an_overwrite_losing_power_before_its_journal_record_leaves_orphans_the_sweep_erases_once_its_instance_left_the_ring`）。
+            HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
+                assert_eq!(
+                    operation,
+                    singlefs_harness::history::OperationTally::default(),
+                    "{kind:?} 不在快档那组比重里，却跑到了"
+                );
+            }
             HistoryOperationKind::PublishFirstFile
             | HistoryOperationKind::PublishOverwrite
             | HistoryOperationKind::PublishWithoutUnits
````

### mutations-append.tsv

````
第10批续 留孤儿比重：掉电再挂那一步的份数是 0（随机历史生成器抽不到它）	crates/singlefs-harness/src/history.rs	                HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,\n                8,	                HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,\n                0,	-p singlefs-harness --lib -- history::tests::the_orphan_sampling_draws_the_overwrite_losing_power_and_the_broad_sampling_never_does	the_orphan_sampling_draws_the_overwrite_losing_power_and_the_broad_sampling_never_does
第10批续 掉电截写：落在 journal 环里的第一个写照常交下去（那次覆盖写整次发布成、盘上没有孤儿）	crates/singlefs-harness/src/history.rs	        if self.journal_ring_offsets_in_bytes.contains(&offset.0) {	        if false && self.journal_ring_offsets_in_bytes.contains(&offset.0) {	-p singlefs-harness --lib -- history::tests::an_overwrite_losing_power_before_its_journal_record_leaves_orphans_the_sweep_erases_once_its_instance_left_the_ring	an_overwrite_losing_power_before_its_journal_record_leaves_orphans_the_sweep_erases_once_its_instance_left_the_ring
第10批续 掉电截写（崩溃注入抹头那一段）：落在 journal 环里的第一个写照常交下去（崩溃后那次挂载的清扫一个头都不抹）	crates/singlefs-harness/src/history.rs	        if self.journal_ring_offsets_in_bytes.contains(&offset.0) {	        if false && self.journal_ring_offsets_in_bytes.contains(&offset.0) {	-p singlefs-checker-tier --lib -- crash_injection::tests::a_writable_mount_after_the_crash_whose_sweep_erases_heads_of_orphans_is_judged_in_the_erasing_phase_without_a_red_field	a_writable_mount_after_the_crash_whose_sweep_erases_heads_of_orphans_is_judged_in_the_erasing_phase_without_a_red_field
第10批续 掉电截写（崩溃后可写挂载那条集成用例）：落在 journal 环里的第一个写照常交下去（抹头那一段摆不到二次崩溃）	crates/singlefs-harness/src/history.rs	        if self.journal_ring_offsets_in_bytes.contains(&offset.0) {	        if false && self.journal_ring_offsets_in_bytes.contains(&offset.0) {	-p singlefs-checker-tier --test crash_injection_writable_mount_after_the_crash -- every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase	every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase
第10批续 清扫崩溃点用例经崩溃注入那一份引可写挂载的分段（随机历史脚手架经没登记的库模块进判法闭包）	crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs	use singlefs_checker_tier::writable_mount_phases::{	use singlefs_checker_tier::crash_injection::{	-p singlefs-checker-tier --lib -- crash_identity::tests::every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries	every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries
````

### mutations-replacements.tsv

````
实审 B3b 第 2 条（二次崩溃）：挂载那一段的取号写一律归进写行，二次崩溃摆不到取号那一段	crates/singlefs-checker-tier/src/writable_mount_phases.rs	                (WritableMountPhase::Acquisition, StepKind::SystemConfigurationSlot) => {\n                    WritableMountPhase::Acquisition\n                }	                (WritableMountPhase::Acquisition, StepKind::SystemConfigurationSlot) => {\n                    WritableMountPhase::RowPublish\n                }	-p singlefs-checker-tier --test crash_injection_writable_mount_after_the_crash	every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase
三·2 Q-J：可写挂载的段表里没有续做（续做那一段摆不到二次崩溃）	crates/singlefs-checker-tier/src/writable_mount_phases.rs	    pub const ALL: [WritableMountPhase; 6] = [\n        WritableMountPhase::Acquisition,\n        WritableMountPhase::RowPublish,\n        WritableMountPhase::WarmUp,\n        WritableMountPhase::Continuation,\n        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,\n        WritableMountPhase::InstanceTableSweepPublish,\n    ];	    pub const ALL: [WritableMountPhase; 5] = [\n        WritableMountPhase::Acquisition,\n        WritableMountPhase::RowPublish,\n        WritableMountPhase::WarmUp,\n        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,\n        WritableMountPhase::InstanceTableSweepPublish,\n    ];	-p singlefs-checker-tier --lib -- crash_injection::tests	the_writes_after_the_publishes_before_the_continuation_belong_to_the_continuation_phase_and_get_a_second_crash_point
三·2 Q-J：续做从「续做之前发了几次」那个根槽写之后再晚一次才算起（续做第一步的写归暖机）	crates/singlefs-checker-tier/src/writable_mount_phases.rs	            if publishes_landed >= publishes_before_the_continuation =>	            if publishes_landed > publishes_before_the_continuation =>	-p singlefs-checker-tier --lib -- crash_injection::tests	the_writes_after_the_publishes_before_the_continuation_belong_to_the_continuation_phase_and_get_a_second_crash_point
第9批 清扫两段：抹头之后不按 FUA 分清扫发布（清扫发布的单元写归抹头段）	crates/singlefs-checker-tier/src/writable_mount_phases.rs	                (WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep, StepKind::UnitWrite)\n                    if !write.is_force_unit_access =>	                (WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep, StepKind::UnitWrite)\n                    if false && !write.is_force_unit_access =>	-p singlefs-checker-tier --lib -- crash_injection::tests::the_writes_of_the_instance_table_sweep_split_into_erasing_heads_and_the_sweep_publish_and_each_gets_a_second_crash_point	the_writes_of_the_instance_table_sweep_split_into_erasing_heads_and_the_sweep_publish_and_each_gets_a_second_crash_point
第9批 清扫两段：WritableMountPhase::ALL 里没有抹头与清扫发布（两段摆不到二次崩溃）	crates/singlefs-checker-tier/src/writable_mount_phases.rs	    pub const ALL: [WritableMountPhase; 6] = [\n        WritableMountPhase::Acquisition,\n        WritableMountPhase::RowPublish,\n        WritableMountPhase::WarmUp,\n        WritableMountPhase::Continuation,\n        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,\n        WritableMountPhase::InstanceTableSweepPublish,\n    ];	    pub const ALL: [WritableMountPhase; 4] = [\n        WritableMountPhase::Acquisition,\n        WritableMountPhase::RowPublish,\n        WritableMountPhase::WarmUp,\n        WritableMountPhase::Continuation,\n    ];	-p singlefs-checker-tier --lib -- crash_injection::tests::the_writes_of_the_instance_table_sweep_split_into_erasing_heads_and_the_sweep_publish_and_each_gets_a_second_crash_point	the_writes_of_the_instance_table_sweep_split_into_erasing_heads_and_the_sweep_publish_and_each_gets_a_second_crash_point
第10批 清扫两段：一个头都没抹时清扫发布的第一个写记进抹头段（抹头那一段假装摆到了二次崩溃）	crates/singlefs-checker-tier/src/writable_mount_phases.rs	                if phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {	                if true || phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {	-p singlefs-checker-tier --lib -- crash_injection::tests::a_sweep_that_erased_no_head_puts_every_write_after_the_last_publish_before_it_into_the_sweep_publish	a_sweep_that_erased_no_head_puts_every_write_after_the_last_publish_before_it_into_the_sweep_publish
第10批 清扫两段（崩溃注入真流）：一个头都没抹时清扫发布的第一个写记进抹头段	crates/singlefs-checker-tier/src/writable_mount_phases.rs	                if phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {	                if true || phase_so_far == WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep {	-p singlefs-checker-tier --lib -- crash_injection::tests::a_writable_mount_after_the_crash_whose_sweep_publish_deletes_a_row_is_judged_in_the_sweep_publish_phase_without_a_red_field	a_writable_mount_after_the_crash_whose_sweep_publish_deletes_a_row_is_judged_in_the_sweep_publish_phase_without_a_red_field
````

## batch10b-gpu

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10b-gpu/patch/`
- `crates.patch`：sha256 946d91a3c4cdf24c1f3591fe5d22d798e918e1e4b7c441882d7847bd0ee30974
- `mutations-append.tsv`：sha256 aaa19bac48c21f315fc2cb8cbaaaa9bc4a0cf5af6f67b485cd694f9a9e31d63f
- `mutations-replacements.tsv`：sha256 674632a1f79b95bfb075e648ed1e219753fbea6e9a35a2097a180b4b3cf6e243
- `report.md`：sha256 d2cc945046db30ecac31e91f343ed3c5df12fdf680d6baafd95011a77792f16b

### crates.patch

````
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1532,6 +1532,10 @@
         registered_width = 140u;
     } else if (record_type == 4u) {
         registered_width = 88u;
+    } else if (record_type == PACKED_TYPE_TOMBSTONE) {
+        registered_width = 40u;
+    } else if (record_type == PACKED_TYPE_LINEAGE) {
+        registered_width = 16u;
     }
     let legal = record_type == expected_type && registered_width == width && count * width <= declared && declared <= DATA_UNIT_BYTES - 136u;
     w_judge(INVARIANT_I_1_7, legal);
@@ -1669,13 +1673,11 @@
         w_judge(INVARIANT_I_9_16, true);
         return;
     }
+    // ②：按发号次序 1,2,3,4,6,7,8,9,10,11,12（walk.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER）
     var previous_tree = vec2<u32>(0u, 0u);
     var have_previous = false;
-    for (var k = 0u; k < 7u; k = k + 1u) {
-        var kind = k + 1u;
-        if (k >= 4u) {
-            kind = k + 2u;
-        }
+    for (var k = 0u; k < TREE_KINDS_OF_THE_TREE_TABLE; k = k + 1u) {
+        let kind = tree_kind_in_the_issuing_order(k);
         for (var e = 0u; e < n.entry_count; e = e + 1u) {
             let o = entry_offset(n, e);
             if (rd16(n.base, o + 10u) == kind) {
@@ -2742,7 +2744,8 @@
             }
         }
     }
-    // key 形态：1 extent、2 inode、3 分配记录、4 记账、5 映射；别的种类不走
+    // key 形态（lib.rs 的 key_schema_for_tree_kind）：1 extent、2 inode 与 deleted_inodes、3 分配记录、4 记账、5 映射、墓碑树与谱系树的容器号 [8]
+    // 与 inode 同形（形态 2）；条目字段表宽（walk.rs 的 field_table_width_of_tree_kind）：deleted_inodes 8，墓碑树、谱系树 120；别的种类不走
     var schema = NONE;
     var registered = false;
     var field_table_width = 0u;
@@ -2751,6 +2754,8 @@
     else if (kind == TREE_KIND_ALLOCATION) { schema = 4u; registered = true; field_table_width = 20u; }
     else if (kind == TREE_KIND_ACCOUNTING) { schema = 0u; registered = true; field_table_width = 34u; }
     else if (kind == 5u) { schema = 1u; }
+    else if (kind == TREE_KIND_DELETED_INODES) { schema = 2u; registered = true; field_table_width = DELETED_INODES_ENTRY_BYTES; }
+    else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) { schema = 2u; registered = true; field_table_width = CONTAINER_TREE_ENTRY_BYTES; }
     if (schema == NONE) {
         return;
     }
@@ -2826,6 +2831,116 @@
     if (kind == TREE_KIND_INODE) {
         w_walk_inode_root(read.n, tree, pointer.device0, pointer.slot0, fsid_low);
     }
+    // 墓碑树、谱系树：码 2 根之下是码 3 容器（类型段墓碑 {0, 1}、谱系 {0, 5}）；走到每片容器、判它的头与容器格式
+    if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) {
+        var container_record_type = PACKED_TYPE_TOMBSTONE;
+        if (kind == TREE_KIND_LINEAGE) {
+            container_record_type = PACKED_TYPE_LINEAGE;
+        }
+        w_walk_container_tree_root(read.n, tree, pointer.device0, pointer.slot0, container_record_type, fsid_low);
+    }
+}
+
+// ── 墓碑树或谱系树的码 2 根往下走（walk.rs 的 walk_container_tree_root 与 walk_the_entries_of_a_container_tree_node）
+// 条目 120 字节（分隔 key 8 + 身份引用 26 + 子指针 86）：类型段 = record_type 的走到码 3 容器（判容器头、出生树、身份、出生身份、容器格式，
+// 单元记进这一版走到的落点），类型段 0 的走进下一层码 2 节点（层级 = 父层级 − 1、条目宽 120 才往下走；层级 ≤ 1 的节点里出现类型段 0 记走读失败），
+// 别的类型段记走读失败。往下记的落点都记在正在走的那个码 2 节点下面（w_parent_*）。walk.rs 是递归，这里是同一个先序的显式栈，
+// 深度超过 w_container_tree_frames 的 12 层判不了；两个种类只经这一处调用点
+struct WalkContainerTreeFrame {
+    n: Node,
+    next_entry: u32,
+    device: u32,
+    slot: vec2<u32>,
+};
+
+var<private> w_container_tree_frames: array<WalkContainerTreeFrame, 12>;
+
+fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: vec2<u32>, record_type: u32, fsid_low: vec2<u32>) {
+    let parent_valid = w_parent_valid;
+    let parent_device = w_parent_device;
+    let parent_slot = w_parent_slot;
+    w_container_tree_frames[0].n = root;
+    w_container_tree_frames[0].next_entry = 0u;
+    w_container_tree_frames[0].device = root_device;
+    w_container_tree_frames[0].slot = root_slot;
+    var depth = 1u;
+    w_parent_valid = true;
+    w_parent_device = root_device;
+    w_parent_slot = root_slot;
+    loop {
+        if (depth == 0u) {
+            break;
+        }
+        let top = depth - 1u;
+        let n = w_container_tree_frames[top].n;
+        let e = w_container_tree_frames[top].next_entry;
+        if (e >= n.entry_count) {
+            depth = depth - 1u;
+            if (depth > 0u) {
+                w_parent_device = w_container_tree_frames[depth - 1u].device;
+                w_parent_slot = w_container_tree_frames[depth - 1u].slot;
+            }
+            continue;
+        }
+        w_container_tree_frames[top].next_entry = e + 1u;
+        let o = entry_offset(n, e);
+        let entry_type = rd16(n.base, o + 16u);
+        let child = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
+        w_judge_location_order(child);
+        w_judge_pointer_head(child);
+        if (entry_type == 0u) {
+            if (n.level <= 1u) {
+                w_failures = w_failures + 1u;
+                continue;
+            }
+            let read = w_read_index_node(child, tree, 2u, 0u, true, fsid_low);
+            if (!read.ok) {
+                continue;
+            }
+            w_judge(INVARIANT_I_1_1, read.n.level + 1u == n.level);
+            if (read.n.level + 1u == n.level && read.n.entry_width == CONTAINER_TREE_ENTRY_BYTES) {
+                if (depth >= MAX_FRAMES) {
+                    undecidable = UNDECIDABLE_TOO_DEEP;
+                    break;
+                }
+                w_container_tree_frames[depth].n = read.n;
+                w_container_tree_frames[depth].next_entry = 0u;
+                w_container_tree_frames[depth].device = child.device0;
+                w_container_tree_frames[depth].slot = child.slot0;
+                depth = depth + 1u;
+                w_parent_device = child.device0;
+                w_parent_slot = child.slot0;
+            }
+            continue;
+        }
+        if (entry_type != record_type) {
+            w_failures = w_failures + 1u;
+            continue;
+        }
+        w_note_pointer_references(child, 2u);
+        let s = w_read_referenced_unit(child, DATA_UNIT_BYTES);
+        if (s.version == NONE) {
+            w_failures = w_failures + 1u;
+            continue;
+        }
+        if (!w_visit(child.device0, child.slot0)) {
+            w_merge_below(child.device0, child.slot0);
+            continue;
+        }
+        w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
+        // 条目里的身份引用（出生树、类型段、容器号、容器出生代）与容器头逐段相同，不同记走读失败
+        let identity_holds = eq64(rd64(s.base, 43u), rd64(n.base, o + 8u)) && rd16(s.base, 51u) == entry_type
+            && eq64(rd64(s.base, 53u), rd64(n.base, o + 18u)) && eq64(rd64(s.base, 61u), rd64(n.base, o + 26u));
+        if (!identity_holds) {
+            w_failures = w_failures + 1u;
+        }
+        w_judge_birth_identity(s, child);
+        // 容器格式（类标签、记录类型、记录宽与声明长度）按 I-1.7 判；记录本身的不变量归 I-9.17 起那几条，走读不判
+        _ = w_judge_packed_container(s, record_type, fsid_low);
+    }
+    w_parent_valid = parent_valid;
+    w_parent_device = parent_device;
+    w_parent_slot = parent_slot;
 }
 
 // ── 一版的树表与中央映射树根（walk_tree_table_and_central_mapping_root）
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -148,14 +148,13 @@
 // 第 69 条：I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），扫描内核挨着 I-9.14 判
 const INVARIANT_I_9_28: u32 = 68u;
 
-// I-9.27 读的另外四个种类码（walk.rs 的 TREE_KIND_*；1..8 在 crash_judge_common.wgsl）
-const TREE_KIND_DELETED_INODES: u32 = 9u;
+// 墓碑树、谱系树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_TOMBSTONE / PACKED_TYPE_LINEAGE；登记的记录宽墓碑 40、谱系 16）
+const PACKED_TYPE_TOMBSTONE: u32 = 1u;
 
-const TREE_KIND_TOMBSTONE: u32 = 10u;
+const PACKED_TYPE_LINEAGE: u32 = 5u;
 
-const TREE_KIND_LINEAGE: u32 = 11u;
-
-const TREE_KIND_LOGGED_OPERATIONS: u32 = 12u;
+// deleted_inodes 根的条目：key = inode 号 8 字节、无 value（walk.rs 的 DELETED_INODES_ENTRY_BYTES）
+const DELETED_INODES_ENTRY_BYTES: u32 = 8u;
 
 // 树表条目里头 ID 8 字节的偏移（walk.rs 的 TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET）
 const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: u32 = 116u;
@@ -365,9 +364,11 @@
 
 const C_CANDIDATES: u32 = 166368u;            // 候选根在根表里的下标
 
-const C_CANDIDATE_WORDS: u32 = 40u;          // 0 走全了、1 有树表落点、2 盘、3 槽 lo、4 槽 hi、5 树数、6.. 每树 4 字（树 lo/hi、诞生 lo/hi）
+const C_CANDIDATE_WORDS: u32 = 50u;          // 0 走全了、1 有树表落点、2 盘、3 槽 lo、4 槽 hi、5 树数、6.. 每树 4 字（树 lo/hi、诞生 lo/hi）：6 + 4 × C_MAX_TREES_PER_TABLE
 
-const C_MAX_TREES_PER_TABLE: u32 = 8u;
+// 一张树表最多记几棵不同的树：进树表的种类有 11 个（recovery.rs 与 walk.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER，
+// 第一个文件版本那次就写满 11 条）；合法镜像上一张树表每种至多一条，更多的只出现在坏镜像上，第 12 棵起判不了、交 CPU
+const C_MAX_TREES_PER_TABLE: u32 = 11u;
 
 // I-9.28 在 I-3.9 与 I-9.14 判完之后借这几行，根表里第 k 条根记进第 k 行（只用 1..5 与每树那几字）：根表最多 MAX_ROOTS 条，
 // 不多于 C_MAX_CANDIDATES 行（judge_tree_identifier_birth_txg_across_every_readable_root 开头判，越过就判不了）
@@ -377,61 +378,61 @@
 
 const C_MAX_ALLOC_RECORDS: u32 = 2048u;
 
-const C_ALLOC_RECORDS: u32 = 168336u;
+const C_ALLOC_RECORDS: u32 = 168816u;
 
 const C_R_MAX_SEEN: u32 = 512u;
 
-const C_R_SEEN: u32 = 184720u;                // 盘、槽 lo/hi
+const C_R_SEEN: u32 = 185200u;                // 盘、槽 lo/hi
 
 const C_R_PENDING_WORDS: u32 = 10u;
 
 const C_R_MAX_PENDING: u32 = 512u;
 
-const C_R_PENDING: u32 = 186256u;
+const C_R_PENDING: u32 = 186736u;
 
 const C_MAX_SEEN_NODES: u32 = 256u;
 
-const C_SEEN_NODES: u32 = 191376u;           // 盘、槽 lo/hi、校验和
+const C_SEEN_NODES: u32 = 191856u;           // 盘、槽 lo/hi、校验和
 
 const C_MAX_EXAMINED: u32 = 2048u;
 
-const C_EXAMINED: u32 = 192400u;             // 盘、槽 lo/hi、代 lo/hi
+const C_EXAMINED: u32 = 192880u;             // 盘、槽 lo/hi、代 lo/hi
 
 const C_MAX_SCANNED: u32 = 2048u;
 
-const C_SCANNED: u32 = 202640u;              // 落点下标
+const C_SCANNED: u32 = 203120u;              // 落点下标
 
 const C_R_MAX_ROWS: u32 = 256u;
 
-const C_R_ROWS: u32 = 204688u;               // 实例、txg lo/hi
+const C_R_ROWS: u32 = 205168u;               // 实例、txg lo/hi
 
 const C_VALID_BEFORE_WORDS: u32 = 6u;        // 根下标、有指针、inode 基、inode 偏移、extent 基、extent 偏移
 
-const C_VALID_BEFORE: u32 = 205456u;
+const C_VALID_BEFORE: u32 = 205936u;
 
-const C_NON_EMPTY: u32 = 205744u;            // txg lo/hi、实例、valid_before 下标
+const C_NON_EMPTY: u32 = 206224u;            // txg lo/hi、实例、valid_before 下标
 
-const C_UNDETERMINABLE: u32 = 205936u;       // txg lo/hi、实例
+const C_UNDETERMINABLE: u32 = 206416u;       // txg lo/hi、实例
 
-const C_COUNTED: u32 = 206080u;              // txg lo/hi
+const C_COUNTED: u32 = 206560u;              // txg lo/hi
 
-const C_COUNTED_STATES: u32 = 206176u;       // valid_before 下标
+const C_COUNTED_STATES: u32 = 206656u;       // valid_before 下标
 
-const C_PICKED: u32 = 206224u;               // 选过的标记（非空 48 + 判不了 48）
+const C_PICKED: u32 = 206704u;               // 选过的标记（非空 48 + 判不了 48）
 
 const C_MAX_RELEASED_ABOVE: u32 = 2048u;
 
-const C_RELEASED_ABOVE: u32 = 206320u;       // 盘、槽 lo/hi、跨度
+const C_RELEASED_ABOVE: u32 = 206800u;       // 盘、槽 lo/hi、跨度
 
 const C_MAX_DEFERRED: u32 = 4096u;
 
-const C_DEFERRED: u32 = 214512u;             // 盘、槽 lo/hi
+const C_DEFERRED: u32 = 214992u;             // 盘、槽 lo/hi
 
 const C_MAX_GROUPS: u32 = 2048u;
 
-const C_GROUPS: u32 = 226800u;               // 槽 lo/hi、跨度、盘位掩码
+const C_GROUPS: u32 = 227280u;               // 槽 lo/hi、跨度、盘位掩码
 
-const C_SCRATCH_END_SCAN: u32 = 234992u;
+const C_SCRATCH_END_SCAN: u32 = 235472u;
 
 var<private> w_context: u32;
 
@@ -659,7 +660,7 @@
 // ───────────────────────── 走读内核交给扫描内核的东西（草稿区里的交接段）─────────────────────────
 // 两个内核各自编（驱动编一个 4900 行的内核要 41 分钟、13 GiB；拆开各编各的、并行）；草稿区在同一次派活里两趟共用，
 // 集合都留在草稿区里，只把私有变量里的计数与几何写进这一段。
-const H_BASE: u32 = 234992u;
+const H_BASE: u32 = 235472u;
 const H_VALID: u32 = 0u;                 // 1 = 走读那一批判完、判得了；0 = 扫描内核什么都不做
 const H_ROW_COUNT: u32 = 1u;
 const H_REFERENCE_COUNT: u32 = 2u;
@@ -691,20 +692,20 @@
 const H_BELOW_FLOOR_SLOT_COUNT: u32 = 28u;
 const H_SLOTS_PER_REGION: u32 = 29u;     // 根环每区槽数 S：I-7.9 的一圈窗口宽 3S = H_REGIONS × 它
 const H_MAX_BELOW_FLOOR_SLOTS: u32 = 4096u;
-const H_BELOW_FLOOR_SLOTS: u32 = 235024u;   // 盘、槽 lo/hi
-const H_END: u32 = 247312u;
+const H_BELOW_FLOOR_SLOTS: u32 = 235504u;   // 盘、槽 lo/hi
+const H_END: u32 = 247792u;
 
 // ───────────────────────── 交接段之后：走读内核自己用、扫描内核不读的两张表 ─────────────────────────
 // 主走读（上下文 0）每个走过的单元一行：有没有记下诞生代号、诞生代号 lo/hi（walk.rs 的 birth_txg_of_walked_units，与 C_VISITED0 同下标）。
 // 别的版本再引用它时按这张表判 I-4.9（非最新候选根引用的单元不晚于它）。
 const C_VISITED_BIRTH_WORDS: u32 = 3u;
 
-const C_VISITED_BIRTHS0: u32 = 247312u;
+const C_VISITED_BIRTHS0: u32 = 247792u;
 
 // I-4.1 重放读的那一份环（walk.rs 的 records_the_replay_reads）：版本、计数器 lo/hi、实例，按 (实例, 计数器) 升序
 const C_REPLAY_RECORD_WORDS: u32 = 4u;
 
-const C_REPLAY_RECORDS: u32 = 253456u;
+const C_REPLAY_RECORDS: u32 = 253936u;
 
 // I-3.5（引用区间的精确性）的出现规则读的四张表，只记主走读（上下文 0）走的那几版（最新根、回退候选根、由记录施加出来的版本）。
 // ① 落点表（walk.rs 的 unit_of_each_placement 与 user_visible_tree_of_each_placement）：跟着指针记下的每个落点一行——
@@ -714,7 +715,7 @@
 
 const C_APPEARANCE_MAX_PLACEMENTS: u32 = 2048u;
 
-const C_APPEARANCE_PLACEMENTS: u32 = 255504u;
+const C_APPEARANCE_PLACEMENTS: u32 = 255984u;
 
 // ② 版本表（walk.rs 的 versions_walked_for_the_appearance_rule）：走完的每一版一行——实例、txg lo/hi、走读一条失败都没记（1 / 0）、
 //    这一版引用的落点在 ③ 里的起点与条数、树表里用户可见树的根几棵，之后每棵 5 个字（种类、树 ID lo/hi、根指针出生 txg lo/hi）。
@@ -728,12 +729,12 @@
 
 const C_APPEARANCE_MAX_VERSIONS: u32 = 64u;
 
-const C_APPEARANCE_VERSIONS: u32 = 275984u;
+const C_APPEARANCE_VERSIONS: u32 = 276464u;
 
 // ③ 每一版引用的落点（walk.rs 的 placements_referenced_in_this_version 在这一版走完那一刻的样子）：每个落点一个字，是它在 ① 里的下标。
 const C_APPEARANCE_MAX_VERSION_PLACEMENTS: u32 = 8192u;
 
-const C_APPEARANCE_VERSION_PLACEMENTS: u32 = 279056u;
+const C_APPEARANCE_VERSION_PLACEMENTS: u32 = 279536u;
 
 // ④ 走过的树表（walk.rs 的 tree_table_entries_of_a_walked_tree_table 里 I-3.5 读的那一样）：第一个读得出它的那一版记下树表单元的盘、槽 lo/hi
 //    与它里面用户可见树的根几棵、每棵 5 个字（同 ②）；别的版本共享这片树表时照它取这一版的根。
@@ -743,10 +744,10 @@
 
 const C_APPEARANCE_MAX_WALKED_TREE_TABLES: u32 = 64u;
 
-const C_APPEARANCE_WALKED_TREE_TABLES: u32 = 287248u;
+const C_APPEARANCE_WALKED_TREE_TABLES: u32 = 287728u;
 
 // 每个状态的草稿区到这里为止：crash_judge_gpu.rs 的 SCRATCH_WORDS 与它相等（那一份的单测核）
-const SCRATCH_END: u32 = 290064u;
+const SCRATCH_END: u32 = 290544u;
 
 fn handoff_region_device(region: u32) -> u32 {
     if (region == 0u) {
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -576,6 +576,18 @@
             }
         } else if (kind == TREE_KIND_ACCOUNTING) {
             r_note_every_node_below(node, 0u);
+        } else if (kind == TREE_KIND_DELETED_INODES) {
+            // deleted_inodes 根的条目只是号，不引用别的单元（根自己上面记过）
+        } else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) {
+            // 墓碑树、谱系树（walk.rs 的 TreeKindForReferenceScan::ContainerTree）：根是层级 1 时每条条目的子指针（条目偏移 34 起）指一片容器；
+            // 根长高之后（层级 > 1）这一遍不往下数，记成数不全
+            if (node.level == 1u && node.entry_width >= CONTAINER_TREE_ENTRY_BYTES) {
+                for (var container = 0u; container < node.entry_count; container = container + 1u) {
+                    r_note(parse_node_pointer(node.base, entry_offset(node, container) + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET));
+                }
+            } else {
+                r_complete = false;
+            }
         } else {
             r_complete = false;
         }
@@ -1200,8 +1212,9 @@
     return lt64(anchor_txg, add64_saturating(txg, u64_of(root_ring_slot_count)));
 }
 
-// I-7.9：只判抬 F 的那一条根，拿它之前的根算上限。两支的「之前的根」都只取 txg 比它小、同时落在两个一圈窗口里的根
-// （用户 2026-10-04 定「乙，两支都取两个窗口」）：锚在它自己的 txg 上的那一个，与锚在这张镜像根环里最新那条根的 txg 上的那一个（候选过滤那一个）
+// I-7.9：只判抬 F 的那一条根，拿它之前的根算上限。两支的「之前的根」都只取 txg 比它小、落在锚在这张镜像根环里最新那条根的 txg 上的
+// 一圈窗口里的根（候选过滤那一个；CPU 一侧 walk.rs 同一个口径）。锚在被判那条根自己的 txg 上的那一个窗口不取：最新那条根的 txg 不小于
+// 根表里每一条根的（walk_and_hand_off 择 (txg, 实例) 最大的），锚在它上面的窗口本来就落在另一个里面，两个的交就是它，拿掉不改任何判定
 fn judge_rollback_floor_raises_against_their_ceilings() {
     let device_count = hdr(3u);
     let root_ring_slot_count = sw(H_BASE + H_REGIONS) * sw(H_BASE + H_SLOTS_PER_REGION);
@@ -1224,13 +1237,12 @@
             continue;
         }
         if (raising.unmount) {
-            // 带卸载记号那一支：F 不高于两个窗口里 txg 比它小的最新那条根的 txg；窗口里一条都没有 ⇒ 这条根不判
+            // 带卸载记号那一支：F 不高于窗口里 txg 比它小的最新那条根的 txg；窗口里一条都没有 ⇒ 这条根不判
             var have_newest_before = false;
             var newest_before = vec2<u32>(0u, 0u);
             for (var k = 0u; k < root_count; k = k + 1u) {
                 let txg = checker_root_of(k).txg;
                 if (lt64(txg, raising.txg)
-                    && in_the_one_revolution_window(txg, raising.txg, root_ring_slot_count)
                     && in_the_one_revolution_window(txg, newest_txg_of_the_image, root_ring_slot_count)
                     && (!have_newest_before || lt64(newest_before, txg))) {
                     newest_before = txg;
@@ -1245,12 +1257,11 @@
         if (!r_instance_rows(raising.base)) {
             continue;
         }
-        // 它之前的有效根：txg < 它的、在两个一圈窗口里、≥ 抬之前的 F、按它自己的实例表不被抛弃；按 (txg, 实例) 升序
+        // 它之前的有效根：txg < 它的、在一圈窗口里、≥ 抬之前的 F、按它自己的实例表不被抛弃；按 (txg, 实例) 升序
         var valid_count = 0u;
         for (var k = 0u; k < root_count; k = k + 1u) {
             let root = checker_root_of(k);
             if (!lt64(root.txg, raising.txg)
-                || !in_the_one_revolution_window(root.txg, raising.txg, root_ring_slot_count)
                 || !in_the_one_revolution_window(root.txg, newest_txg_of_the_image, root_ring_slot_count)
                 || lt64(root.txg, floor_before) || r_rows_abandon(root.instance, root.txg)) {
                 continue;
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -117,6 +117,19 @@
 const TREE_KIND_LIVELIST: u32 = 6u;
 const TREE_KIND_SPARSE_SIDE_TABLE: u32 = 7u;
 const TREE_KIND_DEADLIST: u32 = 8u;
+// 第一个文件版本那次零根注册的四棵树（recovery.rs 与 walk.rs 的 TREE_KIND_DELETED_INODES..TREE_KIND_LOGGED_OPERATIONS）
+const TREE_KIND_DELETED_INODES: u32 = 9u;
+const TREE_KIND_TOMBSTONE: u32 = 10u;
+const TREE_KIND_LINEAGE: u32 = 11u;
+const TREE_KIND_LOGGED_OPERATIONS: u32 = 12u;
+// 进树表的种类有几个（中央映射树 5 不进树表）：I-9.16 ② 按发号次序数这几个，见 tree_kind_in_the_issuing_order
+const TREE_KINDS_OF_THE_TREE_TABLE: u32 = 11u;
+// deleted_inodes 根的 key 是 inode 号 8 字节；墓碑树、谱系树码 2 节点的 key 是容器号 8 字节（recovery.rs 的 key_width_for_kind）
+const DELETED_INODES_KEY_WIDTH: u32 = 8u;
+const CONTAINER_NUMBER_KEY_WIDTH: u32 = 8u;
+// 墓碑树、谱系树码 2 节点的条目照 inode 树内部条目：分隔 key 8 + 身份引用 26 + 子指针 86
+const CONTAINER_TREE_ENTRY_BYTES: u32 = 120u;
+const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: u32 = 34u;
 const FIRST_INODE_NUMBER: u32 = 1u;
 const MAGIC_SFSB: u32 = 0x42534653u;
 const MAGIC_SFSR: u32 = 0x52534653u;
@@ -2703,8 +2716,116 @@
     if (kind == TREE_KIND_INODE) { return 8u; }
     if (kind == TREE_KIND_ALLOCATION) { return 10u; }
     if (kind == TREE_KIND_ACCOUNTING) { return 22u; }
+    if (kind == TREE_KIND_DELETED_INODES) { return DELETED_INODES_KEY_WIDTH; }
+    if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) { return CONTAINER_NUMBER_KEY_WIDTH; }
     return NONE;
 }
+// 进树表的第 position 个种类（发号次序，从 0 数）：1、2、3、4，跳过不进树表的 5，再 6..12
+fn tree_kind_in_the_issuing_order(position: u32) -> u32 {
+    if (position < 4u) {
+        return position + 1u;
+    }
+    return position + 2u;
+}
+struct ContainerTreeFrame {
+    n: Node,
+    next_entry: u32,
+};
+var<private> container_tree_frames: array<ContainerTreeFrame, 12>;
+struct ContainerTreeRead {
+    ok: bool,
+    mapped_units: u32,
+};
+// recovery::read_container_tree_index_nodes（墓碑树、谱系树）：从根往下读码 2 节点（提示读不出经映射回退），每个节点照树根同一组核自描述
+// （tree_root_checked：树 ID、key 宽 8、出生身份、fsid、key 区间与首末条目相符）；根层级至少 1、每个节点非空；每条条目宽不窄于 120、
+// 子指针头部是第一版的取值；层级 ℓ > 1 的条目身份引用全 0、指一个码 2 孩子：层级 ℓ − 1、首 key = 条目的分隔 key；层级 1 的条目指容器，只数不读。
+// 交回进映射的单元数：每个码 2 节点（根在内）加每片容器。深度超过 container_tree_frames 的 12 层判不了。
+// 只有一处读节点（根与孩子都经 pending 那一组进去）：每个调用点各展开一份
+fn read_container_tree_index_nodes(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>) -> ContainerTreeRead {
+    var out: ContainerTreeRead;
+    out.ok = false;
+    out.mapped_units = 0u;
+    var nodes = 0u;
+    var containers = 0u;
+    var depth = 0u;
+    var pending_pointer = root_pointer;
+    var pending_pointer_base = root_pointer_base;
+    var pending_pointer_offset = root_pointer_offset;
+    var pending_separator_base = 0u;
+    var pending_separator_offset = 0u;
+    var enter = true;
+    loop {
+        if (enter) {
+            enter = false;
+            let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
+            if (m.failed) {
+                return out;
+            }
+            let n = parse_index_node(m.s);
+            if (!tree_root_checked(n, tree, CONTAINER_NUMBER_KEY_WIDTH, pending_pointer, root, fsid_low)) {
+                return out;
+            }
+            if (n.entry_count == 0u) {
+                return out;
+            }
+            if (depth == 0u) {
+                if (n.level == 0u) {
+                    return out;
+                }
+            } else {
+                let parent_level = container_tree_frames[depth - 1u].n.level;
+                if (n.level + 1u != parent_level
+                    || !bytes_equal(n.base, n.smallest_offset, pending_separator_base, pending_separator_offset, CONTAINER_NUMBER_KEY_WIDTH)) {
+                    return out;
+                }
+            }
+            if (depth >= MAX_FRAMES) {
+                undecidable = UNDECIDABLE_TOO_DEEP;
+                return out;
+            }
+            container_tree_frames[depth].n = n;
+            container_tree_frames[depth].next_entry = 0u;
+            depth = depth + 1u;
+            nodes = nodes + 1u;
+            continue;
+        }
+        if (depth == 0u) {
+            break;
+        }
+        let top = depth - 1u;
+        let n = container_tree_frames[top].n;
+        let e = container_tree_frames[top].next_entry;
+        if (e >= n.entry_count) {
+            depth = depth - 1u;
+            continue;
+        }
+        container_tree_frames[top].next_entry = e + 1u;
+        if (n.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
+            return out;
+        }
+        let o = entry_offset(n, e);
+        if (!record_pointer_head_is_the_first_version(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET)) {
+            return out;
+        }
+        if (n.level == 1u) {
+            containers = containers + 1u;
+            continue;
+        }
+        // 类型段 0：身份引用（出生树 8、类型 2、容器号 8、容器出生代 8）全 0
+        if (!all_zero(n.base, o + 8u, 26u)) {
+            return out;
+        }
+        pending_pointer = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
+        pending_pointer_base = n.base;
+        pending_pointer_offset = o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET;
+        pending_separator_base = n.base;
+        pending_separator_offset = o;
+        enter = true;
+    }
+    out.ok = true;
+    out.mapped_units = nodes + containers;
+    return out;
+}
 fn expected_version_index(instance: u32, txg: vec2<u32>) -> u32 {
     let count = hdr(8u);
     for (var i = 0u; i < count; i = i + 1u) {
@@ -2782,14 +2903,11 @@
             return walk_failed(25u);
         }
     }
-    // I-9.16 ②：按发号次序 1,2,3,4,6,7,8 相邻两棵严格升序
+    // I-9.16 ②：按发号次序 1,2,3,4,6,7,8,9,10,11,12（recovery.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER）相邻两棵严格升序
     var previous_tree = vec2<u32>(0u, 0u);
     var have_previous = false;
-    for (var k = 0u; k < 7u; k = k + 1u) {
-        var kind = k + 1u;
-        if (k >= 4u) {
-            kind = k + 2u;
-        }
+    for (var k = 0u; k < TREE_KINDS_OF_THE_TREE_TABLE; k = k + 1u) {
+        let kind = tree_kind_in_the_issuing_order(k);
         for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
             let o = entry_offset(tree_table, e);
             if (rd16(tree_table.base, o + 10u) == kind) {
@@ -2816,6 +2934,8 @@
     var extent_nodes = 0u;
     var allocation_nodes = 0u;
     var accounting_nodes = 0u;
+    var have_deleted_inodes = false;
+    var container_tree_units = 0u;
     var inode_root = bad_node();
     var inode_root_pointer = zero_pointer();
     for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
@@ -2858,11 +2978,20 @@
             have_accounting = true;
             continue;
         }
+        // 墓碑树与谱系树：从根往下读码 2 节点、数进映射的节点与容器，不读容器（它们的内容不影响读回的文件）
+        if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) {
+            let read = read_container_tree_index_nodes(pointer, tree_table.base, o + 14u, tree, root, fsid_low);
+            if (!read.ok) {
+                return walk_failed(62u);
+            }
+            container_tree_units = container_tree_units + read.mapped_units;
+            continue;
+        }
         let key_width = tree_kind_key_width(kind);
         if (key_width == NONE) {
             return walk_failed(34u);
         }
-        // inode 树根（进映射的树根：提示读不出经映射回退，再核自描述）
+        // inode 树根与 deleted_inodes 根（进映射的树根：提示读不出经映射回退，再核自描述）
         let m = read_via_hint_then_mapping(pointer, tree_table.base, o + 14u, 2u, NODE_BYTES);
         if (m.failed) {
             return walk_failed(35u);
@@ -2876,6 +3005,9 @@
             inode_root_pointer = pointer;
             have_inode = true;
         }
+        if (kind == TREE_KIND_DELETED_INODES) {
+            have_deleted_inodes = true;
+        }
     }
     if (!have_extent || !have_inode || !have_allocation || !have_accounting) {
         return walk_failed(37u);
@@ -2944,8 +3076,10 @@
     if (accounting_count != 3u + 6u * device_count) {
         return walk_failed(40u);
     }
-    // 映射条目数 = 1 + extent 节点 + 分配记录树节点 + 记账树节点 + inode 叶容器数 + 数据单元数
-    if (mapping_count != 1u + extent_nodes + allocation_nodes + accounting_nodes + inode_root.entry_count + data_count) {
+    // 映射条目数 = 1 + extent 节点 + 分配记录树节点 + 记账树节点 + inode 叶容器数 + 数据单元数 + 删除那两棵树与谱系树的节点
+    // （recovery.rs 的 deletion_tree_nodes：deleted_inodes 根有就 1，墓碑树、谱系树各自每个码 2 节点加每片容器）
+    let deletion_tree_nodes = select(0u, 1u, have_deleted_inodes) + container_tree_units;
+    if (mapping_count != 1u + extent_nodes + allocation_nodes + accounting_nodes + inode_root.entry_count + data_count + deletion_tree_nodes) {
         return walk_failed(41u);
     }
     // 分配记录跨度非 0、代不晚于根
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -879,14 +879,14 @@
     #[test]
     fn states_in_one_dispatch_follow_the_scratch_memory_up_to_the_lane_limit() {
         let bytes_per_state = u64::try_from(SCRATCH_WORDS * 4).expect("装得进 u64");
-        assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1850);
+        assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1847);
         assert_eq!(states_per_scratch_lane(1), 1, "再小的上限也带一个");
-        // 12 GiB 的草稿区：12 × 1024 × 1024 × 1024 ÷ 1 160 256 = 11 105 个（向下取整）
+        // 12 GiB 的草稿区：12 × 1024 × 1024 × 1024 ÷ 1 162 176 = 11 086 个（向下取整）
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 12 * 1024),
             usize::try_from(12 * 1024 * 1024 * 1024 / bytes_per_state).expect("装得进 usize")
         );
-        assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 11_105);
+        assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 11_086);
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 0),
             1,
@@ -894,7 +894,7 @@
         );
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 1024 * 1024),
-            1850 * MAXIMUM_SCRATCH_LANES,
+            1847 * MAXIMUM_SCRATCH_LANES,
             "显存再多也只到条数上限"
         );
     }
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -113,7 +113,7 @@
 }
 /// 每个状态的草稿区几个 u32（checker 共用那一半里 `SCRATCH_END`：共用的 `SCRATCH_WORDS_REQUIRED`、走读与扫描的集合、两个内核的交接段、
 /// 交接段之后走读内核自己用的表：两张，加 I-3.5 的出现规则读的四张）。
-pub const SCRATCH_WORDS: usize = 290_064;
+pub const SCRATCH_WORDS: usize = 290_544;
 /// 内核的工作组大小（三个内核都是 `@workgroup_size(64)`）：派 n 个状态要 ⌈n ÷ 64⌉ 个工作组。
 pub const WORKGROUP_SIZE: usize = 64;
 /// 内核的四个绑定（`crash_judge_common.wgsl` 开头）：派活参数、打包的表、结论、草稿区。
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -6,7 +6,8 @@
 //! 改坏那条流上 CPU 有红，GPU 要红在同样的状态、同样的项上——这是 GPU 判器分得出差别的自证；里程碑三二·1 那一批的每一条不变量
 //! 另有一条只改坏它管的那个字段的流（`the_gpu_judge_agrees_with_the_cpu_when_…` 那一组）；I-3.5 的出现规则红与成立各一条，
 //! 恢复施加一条记录取新根段的那几段、施加出来的那一版从记录走实例表与分配记录树、I-8.10 后三段，各有一条同样形态的流；
-//! I-9.28 比到候选集之外的根与树表读不出的根、I-7.8 扫描方向的认领规则、I-7.9 两支只取两个一圈窗口里的根，也各有流。
+//! I-9.28 比到候选集之外的根与树表读不出的根、I-7.8 扫描方向的认领规则、I-7.9 两支只取锚在镜像最新根上的一圈窗口里的根，也各有流；
+//! 树表 11 条与种类 9–11（谱系树的容器记录宽、谱系树根的树 ID、谱系树与 logged_ops 两条的发号次序）各有一条，树表条数不超过扫描内核的上限取卡之前就判。
 //! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
 //! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性；
 //! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（没设不定额度）、
@@ -3768,6 +3769,253 @@
     );
     assert!(
         comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
+
+// ───────────── 树表 11 条与种类 9–11（删除与并发那条线批 6、批 8 的盘面）：GPU 判器跟上之后各有一条分得出的流 ─────────────
+
+/// 一个树表单元（码 2、key 宽 8、条目宽 200）的条目数（索引节点头里 key 区间之后那两字节）。
+fn tree_table_entry_count(bytes: &[u8]) -> usize {
+    let header_end = 86 + 2 * usize::from(index_node_key_width(bytes));
+    usize::from(u16::from_le_bytes(
+        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
+    ))
+}
+
+/// 扫描内核一张树表最多记几棵树：池级 checker 共用那一半里 `const C_MAX_TREES_PER_TABLE: u32 = Nu;` 那一行的 N。
+fn tree_count_the_gpu_scan_kernel_records_per_tree_table() -> usize {
+    const DECLARATION_PREFIX: &str = "const C_MAX_TREES_PER_TABLE: u32 = ";
+    let source = singlefs_checker_tier::crash_judge_gpu::GPU_JUDGE_CHECKER_COMMON_SOURCE;
+    let declarations: Vec<&str> = source
+        .lines()
+        .filter_map(|line| line.strip_prefix(DECLARATION_PREFIX))
+        .collect();
+    assert_eq!(
+        declarations.len(),
+        1,
+        "池级 checker 共用那一半里恰好一行 `{DECLARATION_PREFIX}…`"
+    );
+    declarations[0]
+        .strip_suffix("u;")
+        .expect("声明以 `u;` 结尾")
+        .parse()
+        .expect("上限是十进制数")
+}
+
+/// 新池新建文件与固定脚本到回退 D 两条流写下的每一张树表，条目数都不超过 GPU 扫描内核一张树表能记的树数：核心层的树表再长
+/// （删除与并发那条线批 6 把第一个文件版本的树表从 7 条加到 11 条，扫描内核那时只记 8 棵，第 9 棵起判不了），这条在取卡之前就红，
+/// 不等对拍跑到新池那条流的状态 36。没卡的机器上也判到。
+#[test]
+fn every_tree_table_the_new_pool_and_the_rollback_flows_write_fits_the_tree_count_the_gpu_scan_kernel_records(
+) {
+    let capacity = tree_count_the_gpu_scan_kernel_records_per_tree_table();
+    let is_a_tree_table = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && index_node_key_width(bytes) == 8
+            && index_node_entry_width(bytes) == 200
+    };
+    for flow in [
+        record_new_pool_file_creation("crash-judge-gpu-tree-count-new-pool", "new-pool"),
+        record_fixed_script_through_rollback("crash-judge-gpu-tree-count-rollback"),
+    ] {
+        let entry_counts: Vec<usize> = flow
+            .writes
+            .iter()
+            .filter(|write| write.kind == StepKind::UnitWrite)
+            .filter_map(|write| write.bytes())
+            .filter(|bytes| is_a_tree_table(bytes))
+            .map(tree_table_entry_count)
+            .collect();
+        assert!(!entry_counts.is_empty(), "流 {} 写了树表单元", flow.name);
+        let largest = entry_counts.iter().copied().max().expect("至少一张");
+        assert!(
+            largest <= capacity,
+            "流 {} 写下的树表最多 {largest} 条，GPU 扫描内核一张树表只记 {capacity} 棵树：先把 C_MAX_TREES_PER_TABLE 跟上（草稿区顺移、SCRATCH_WORDS 重算）",
+            flow.name
+        );
+    }
+}
+
+/// 回退流上回退那次发布 D 往谱系树追加的那一片容器（码 3、记录类型 5）：记录宽从 16 改成 17、重封到根。
+/// CPU 池级 checker 走谱系树走到这片容器、按 I-1.7 判它的记录宽不是登记的 16 → 红；恢复不读容器，两遍恢复照常读到文件。
+/// GPU 走读内核不走谱系树、或不认谱系记录的登记宽时这一位对不上。
+fn record_rollback_with_the_lineage_container_record_width_off_its_registered_width(
+    tag: &str,
+) -> RecordedFlow {
+    const LINEAGE_RECORD_TYPE: u16 = 5;
+    const PACKED_RECORD_WIDTH_OFFSET: usize = 71;
+    let lineage_container =
+        |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == LINEAGE_RECORD_TYPE;
+    record_rollback_with_a_resealed_field(
+        tag,
+        "rollback-lineage-container-record-width-seventeen",
+        &lineage_container,
+        &|bytes| {
+            let width = u16::from_le_bytes(
+                bytes[PACKED_RECORD_WIDTH_OFFSET..PACKED_RECORD_WIDTH_OFFSET + 2]
+                    .try_into()
+                    .expect("2"),
+            );
+            assert_eq!(width, 16, "谱系记录宽 16");
+            bytes[PACKED_RECORD_WIDTH_OFFSET..PACKED_RECORD_WIDTH_OFFSET + 2]
+                .copy_from_slice(&(width + 1).to_le_bytes());
+        },
+    )
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width",
+        &record_rollback_with_the_lineage_container_record_width_off_its_registered_width(
+            "crash-judge-gpu-lineage-container",
+        ),
+        "I-1.7",
+    );
+}
+
+/// 回退流上谱系树的根（码 2、key 宽 8、条目宽 120、第一条条目的类型段是谱系记录 5）：头里的树 ID 加 1、重封到根。
+/// 落在 D 上的状态，CPU 恢复读谱系树根时核自描述（树 ID 不是树表条目里那棵）→ 走读失败、读不到文件；CPU 池级 checker 读这个根判 I-1.3 → 红。
+/// GPU 恢复内核不读谱系树根、或读了不核树 ID，GPU 走读内核不读谱系树根时，恢复或这一位对不上。
+fn record_rollback_with_the_lineage_root_naming_another_tree(tag: &str) -> RecordedFlow {
+    const LINEAGE_RECORD_TYPE: u16 = 5;
+    const INDEX_NODE_TREE_IDENTIFIER_OFFSET: usize = 42;
+    const CONTAINER_TREE_ENTRY_TYPE_OFFSET: usize = 16;
+    let lineage_root = |bytes: &[u8]| {
+        if unit_class(bytes) != 2
+            || index_node_key_width(bytes) != 8
+            || index_node_entry_width(bytes) != 120
+        {
+            return false;
+        }
+        let first_entry = 86 + 2 * 8 + 29;
+        u16::from_le_bytes(
+            bytes[first_entry + CONTAINER_TREE_ENTRY_TYPE_OFFSET
+                ..first_entry + CONTAINER_TREE_ENTRY_TYPE_OFFSET + 2]
+                .try_into()
+                .expect("2"),
+        ) == LINEAGE_RECORD_TYPE
+    };
+    record_rollback_with_a_resealed_field(
+        tag,
+        "rollback-lineage-root-names-another-tree",
+        &lineage_root,
+        &|bytes| {
+            bytes[INDEX_NODE_TREE_IDENTIFIER_OFFSET] =
+                bytes[INDEX_NODE_TREE_IDENTIFIER_OFFSET].wrapping_add(1);
+        },
+    )
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree(
+) {
+    let flow =
+        record_rollback_with_the_lineage_root_naming_another_tree("crash-judge-gpu-lineage-root");
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-1.3"),
+        "流 {} 上 CPU 池级 checker 要红 I-1.3（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        comparison.cpu_red > 0,
+        "流 {} 上落在 D 上的状态 CPU 恢复读谱系树根要失败、判器要有红",
+        flow.name
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
+
+/// 新池新建文件那张树表里谱系树（种类 11）与 logged_ops（种类 12）两条互换种类码、重封到根：盘上次序仍按树 ID 升序（I-9.16 ① 成立），
+/// 按发号次序谱系树在 logged_ops 之前、树 ID 却更大（② 不成立）。两条都是零根、头 ID 都该是 0，别的判定不受影响。
+/// 根落了的状态 CPU 恢复在读任何一棵树之前拒（I-9.16）、判器有红，CPU 池级 checker 红 I-9.16；GPU 两处 ② 只数到种类 8 时都对不上。
+fn record_new_pool_with_the_lineage_and_logged_operations_kinds_swapped(tag: &str) -> RecordedFlow {
+    const LINEAGE_KIND: u16 = 11;
+    const LOGGED_OPERATIONS_KIND: u16 = 12;
+    const TREE_TABLE_FIRST_ENTRY_OFFSET: usize = 86 + 2 * 8 + 29;
+    const TREE_TABLE_ENTRY_BYTES: usize = 200;
+    const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
+    let tree_table = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && index_node_key_width(bytes) == 8
+            && index_node_entry_width(bytes) == 200
+    };
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-tree-table-lineage-and-logged-operations-kinds-swapped",
+        &tree_table,
+        false,
+        &|bytes| {
+            let kind_offset_of = |kind: u16| {
+                let offsets: Vec<usize> = (0..tree_table_entry_count(bytes))
+                    .map(|entry| {
+                        TREE_TABLE_FIRST_ENTRY_OFFSET
+                            + entry * TREE_TABLE_ENTRY_BYTES
+                            + TREE_TABLE_ENTRY_KIND_OFFSET
+                    })
+                    .filter(|offset| {
+                        u16::from_le_bytes(bytes[*offset..*offset + 2].try_into().expect("2"))
+                            == kind
+                    })
+                    .collect();
+                assert_eq!(offsets.len(), 1, "树表里种类 {kind} 恰好一条");
+                offsets[0]
+            };
+            let lineage = kind_offset_of(LINEAGE_KIND);
+            let logged_operations = kind_offset_of(LOGGED_OPERATIONS_KIND);
+            assert!(
+                lineage < logged_operations,
+                "谱系树那条在 logged_ops 那条之前（树 ID 小）"
+            );
+            bytes[lineage..lineage + 2].copy_from_slice(&LOGGED_OPERATIONS_KIND.to_le_bytes());
+            bytes[logged_operations..logged_operations + 2]
+                .copy_from_slice(&LINEAGE_KIND.to_le_bytes());
+        },
+    )
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order(
+) {
+    let flow = record_new_pool_with_the_lineage_and_logged_operations_kinds_swapped(
+        "crash-judge-gpu-i916-issuing-order",
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-9.16"),
+        "流 {} 上 CPU 池级 checker 要红 I-9.16（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        comparison.cpu_red > 0,
+        "流 {} 上根落了的状态 CPU 恢复要在读树之前拒、判器要有红",
+        flow.name
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
         "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
         flow.name,
         comparison.mismatches.len(),
````

### mutations-append.tsv

````
第 10 批 10b GPU 判器：扫描内核一张树表只记 8 棵树（改回树表 7 条时的上限，新池第一个文件版本的 11 条第 9 棵起判不了）	crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl	const C_MAX_TREES_PER_TABLE: u32 = 11u;	const C_MAX_TREES_PER_TABLE: u32 = 8u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- every_tree_table_the_new_pool_and_the_rollback_flows_write_fits_the_tree_count_the_gpu_scan_kernel_records	every_tree_table_the_new_pool_and_the_rollback_flows_write_fits_the_tree_count_the_gpu_scan_kernel_records
第 10 批 10b GPU 判器：恢复内核不把谱系树当容器树读（落进只读根的那一支，映射条目数少算谱系树的节点与容器）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) {\n            let read = read_container_tree_index_nodes(	        if (kind == TREE_KIND_TOMBSTONE) {\n            let read = read_container_tree_index_nodes(	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows	the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows
第 10 批 10b GPU 判器：恢复内核映射条目数不算删除那两棵树与谱系树的节点（改回只数五样的式子）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	 + data_count + deletion_tree_nodes) {	 + data_count) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows	the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows
第 10 批 10b GPU 判器：恢复内核读容器树的码 2 节点不核自描述（谱系树根的树 ID 对不上也读到文件）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	            if (!tree_root_checked(n, tree, CONTAINER_NUMBER_KEY_WIDTH, pending_pointer, root, fsid_low)) {\n                return out;\n            }	            // 变异：容器树的码 2 节点不核自描述	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree
第 10 批 10b GPU 判器：恢复内核 I-9.16 ② 只按发号次序数到种类 8（改回 7 个种类）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    for (var k = 0u; k < TREE_KINDS_OF_THE_TREE_TABLE; k = k + 1u) {	    for (var k = 0u; k < 7u; k = k + 1u) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order
第 10 批 10b GPU 判器：走读内核 I-9.16 ② 只按发号次序数到种类 8（改回 7 个种类）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    for (var k = 0u; k < TREE_KINDS_OF_THE_TREE_TABLE; k = k + 1u) {	    for (var k = 0u; k < 7u; k = k + 1u) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order
第 10 批 10b GPU 判器：走读内核读了谱系树根不往下走到容器（改回别的种类不走）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_walk_container_tree_root(read.n, tree, pointer.device0, pointer.slot0, container_record_type, fsid_low);	        // 变异：墓碑树、谱系树不往下走	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width
第 10 批 10b GPU 判器：走读内核谱系记录的登记宽取 17（谱系容器的记录宽不按 16 判）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    } else if (record_type == PACKED_TYPE_LINEAGE) {\n        registered_width = 16u;	    } else if (record_type == PACKED_TYPE_LINEAGE) {\n        registered_width = 17u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width
第 10 批 10b GPU 判器：走读内核不读谱系树根（改回 key 形态只认 1–5、别的种类不走）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) { schema = 2u;	    else if (kind == TREE_KIND_TOMBSTONE) { schema = 2u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree
第 10 批 10b GPU 判器：扫描内核引用集合这一遍把谱系树记成数不全（改回种类 1–4 之外一律数不全）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        } else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) {\n            // 墓碑树、谱系树（walk.rs	        } else if (kind == TREE_KIND_TOMBSTONE) {\n            // 墓碑树、谱系树（walk.rs	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows	the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows
````

### mutations-replacements.tsv

````
第 5 批 GPU 判器：派活给每个状态的草稿区停在交接段末端、没跟上内核新加的两张表	crates/singlefs-checker-tier/src/crash_judge_gpu.rs	pub const SCRATCH_WORDS: usize = 290_544;	pub const SCRATCH_WORDS: usize = 247_792;	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_gpu::tests::scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels	scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels
第 9 批 9-9 GPU 判器：I-7.9 准入那一支不取一圈窗口（改回旧判法，之前的根取根环里全部）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	            if (!lt64(root.txg, raising.txg)\n                || !in_the_one_revolution_window(root.txg, newest_txg_of_the_image, root_ring_slot_count)\n                || lt64(root.txg, floor_before)	            if (!lt64(root.txg, raising.txg)\n                || lt64(root.txg, floor_before)	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_judged	the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_judged
第 9 批 9-9 GPU 判器：I-7.9 准入那一支只取锚在被判那条根上的窗口（不取锚在镜像最新根上的那一个）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	                || !in_the_one_revolution_window(root.txg, newest_txg_of_the_image, root_ring_slot_count)\n	                || !in_the_one_revolution_window(root.txg, raising.txg, root_ring_slot_count)\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_takes_no_root_outside_the_window_anchored_at_the_newest_root_of_the_image	the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_takes_no_root_outside_the_window_anchored_at_the_newest_root_of_the_image
第 9 批 9-9 GPU 判器：I-7.9 带卸载记号那一支不取一圈窗口（改回旧判法，比根环里 txg 比它小的最新那条根）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	                if (lt64(txg, raising.txg)\n                    && in_the_one_revolution_window(txg, newest_txg_of_the_image, root_ring_slot_count)\n                    && (!have_newest_before	                if (lt64(txg, raising.txg)\n                    && (!have_newest_before	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_reddened	the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_reddened
````

## batch10c-core

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10c-core/patch/`
- `crates.patch`：sha256 e0b9fc90a97303d359686567de6beb7e585d7bc72ccb38993fb55c84ae4b19da
- `mutations-append.tsv`：sha256 ef17a5f0886b3cc1c4e4651d2d34f914e17fb81c6891c87ec068c7cad971714d
- `mutations-delete.txt`：sha256 b8019e0d86ec428893f948f320a8d06f52e5f1c5eb563d36a5e86e2e1d4581f8
- `mutations-replacements.tsv`：sha256 24a1e90af009dfc096abe3c891177bdb2ec94d2ed4b8954c7d7232fbc8a35c1b
- `report.md`：sha256 3c91281def69dc4dbc12d7aeed6e9ad6da896a25ca0b01f639fd390f85f9a455

### crates.patch

````
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 4ba0cbf..cc41960 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -942,8 +942,9 @@ impl PlacementOnDevice {
 
 /// 挂载那一刻读坏（设备报读错，或读得出、不是全零而自证不过）的一个根环槽能住的 txg（D16（发布语义） 已定项 1 的「挂载时读不出的槽」，
 /// 用户 2026-10-04 定：读不出、自证不过的根槽在回收门槛里钉在它可能住过的最旧 txg，重读读出有效根或确认为空之前门槛不越过它；
-/// 准入抬 F 的上限照 Q3 按它在 (N − 3S, 新实例第一次发布的 txg − 1] 里能住的最大 txg 并）。读回全零的槽另算「从没写过」，不在这里
-/// （用户 2026-10-04 定）。不变量：最旧的不高于最大的（构造时断言）。
+/// 准入抬 F 的上限照 Q3 按它在 (N − 3S, 新实例第一次发布的 txg − 1] 里能住的最大 txg 并）。读回全零的槽不在这里：它不并进上限
+/// （用户 2026-10-04 定），回收门槛里照「自证不过」那一类钉住（用户 2026-10-05 定乙），记成 [`RootRingOccupant::AllZeroAtMount`]。
+/// 不变量：最旧的不高于最大的（构造时断言）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct TxgsARingSlotBadAtMountCanHold {
     oldest_txg_it_can_have_held: CheckpointTxg,
@@ -981,14 +982,21 @@ impl TxgsARingSlotBadAtMountCanHold {
     }
 }
 
-/// 根环一个槽上这会儿住着哪条根。挂载那一刻读回全零的槽（从没写过）与一个 txg 都住不了的槽不在 `RootRingOccupancy` 里：
+/// 根环一个槽上这会儿住着哪条根。一个 txg 都住不了的槽（年轻的环里还没轮到的）不在 `RootRingOccupancy` 里：
 /// 回收门槛取「环里最旧有效根」（D16（发布语义） 已定项 1，用户 2026-10-04 定 RGall，`research/prompts/m3-reclaim-window-r1-main-verification.md`
-/// 第六节 Q1），看自证合法的根与挂载时读坏的槽；读不出的被抛弃根影子账本来就罩不到（`mount::isolate_slots_referenced_only_by_abandoned_roots`）。
+/// 第六节 Q1），看自证合法的根与挂载时读坏、读回全零的槽；读不出的被抛弃根影子账本来就罩不到（`mount::isolate_slots_referenced_only_by_abandoned_roots`）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum RootRingOccupant {
+    /// 挂载那一刻读回全零的槽：回收门槛照「自证不过」那一类钉在它可能住过的最旧 txg（盘上那一槽可能住着一条有效根、只是这一次读回了全零），
+    /// 准入抬 F 的上限不并它（用户 2026-10-04 定全零不并进上限，2026-10-05 定乙：回收门槛里照旧钉住；条文还没写回，定案在
+    /// `records/2026-09-30-里程碑三开工第一轮.md`）。直到这个进程把那个槽写掉，或进程内记根时重读读出有效根、读出被抛弃的根才解开；
+    /// 重读读回全零不算确认为空，照旧钉着。它不算「这个进程知道住着根的槽」。
+    AllZeroAtMount {
+        oldest_txg_it_can_have_held: CheckpointTxg,
+    },
     /// 挂载那一刻读坏的槽（设备报读错，或读得出、不是全零而自证不过）：回收门槛钉在它可能住过的最旧 txg、准入抬 F 的上限按它能住的最大 txg
-    /// 并（[`TxgsARingSlotBadAtMountCanHold`]），直到这个进程把那个槽写掉，或进程内记根时重读读出有效根、读出被抛弃的根、读回全零
-    /// （用户 2026-10-04 定）。它不算「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]）：
+    /// 并（[`TxgsARingSlotBadAtMountCanHold`]），直到这个进程把那个槽写掉，或进程内记根时重读读出有效根、读出被抛弃的根
+    /// （用户 2026-10-04 定；重读读回全零照旧钉着，2026-10-05 定乙）。它不算「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]）：
     /// D16 已定项 1「根槽这一次读坏」那一行，挂载那一刻就读坏、之后没写过的槽算上限时并进来、判回退候选时当没有根，不因为这里记了它就改成「读坏就拒」。
     BadAtMount(TxgsARingSlotBadAtMountCanHold),
     /// 挂载那一刻读坏的槽，这个进程在它成了环里最小的那个时重读、读出了一条自证过而按实例表仍然有效的根
@@ -1054,7 +1062,7 @@ impl RootRingOccupancy {
     /// （有效的与被抛弃的都算），这个进程之后写过、FUA 返回过的槽——发布在落盘之前或落盘途中失败时分配器整个换回发布之前那一份，
     /// 这张表随之退回，记着的只剩根落盘做成的那几次——，与进程内记根时重读读出了有效根的槽
     /// （[`RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread`]，用户 2026-10-04 定）。挂载那一刻读坏、
-    /// 之后又没写过也没重读出有效根的槽不在其内（[`RootRingOccupant::BadAtMount`] 记着的槽也不在）。
+    /// 之后又没写过也没重读出有效根的槽不在其内（[`RootRingOccupant::BadAtMount`] 与 [`RootRingOccupant::AllZeroAtMount`] 记着的槽也不在）。
     #[must_use]
     pub fn ring_slots_known_to_hold_a_root(&self) -> BTreeSet<RootRingSlot> {
         self.occupants
@@ -1063,14 +1071,15 @@ impl RootRingOccupancy {
                 RootRingOccupant::ValidRoot { .. }
                 | RootRingOccupant::AbandonedRoot { .. }
                 | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => true,
-                RootRingOccupant::BadAtMount(_) => false,
+                RootRingOccupant::BadAtMount(_) | RootRingOccupant::AllZeroAtMount { .. } => false,
             })
             .map(|(ring_slot, _)| *ring_slot)
             .collect()
     }
 
     /// 环里最旧有效根的 txg（D16（发布语义） 已定项 1「环里最旧有效根」，用户 2026-10-04 定 RGall：回收门槛从一圈窗口退回整个环）：
-    /// 这张表里全部有效根与挂载时读坏的槽（钉在它可能住过的最旧 txg，用户 2026-10-04 定）取最小，不按 txg 离这个进程记到的最新根多远滤。
+    /// 这张表里全部有效根与挂载时读坏、读回全零的槽（钉在它可能住过的最旧 txg，用户 2026-10-04 定；读回全零的 2026-10-05 定乙）取最小，
+    /// 不按 txg 离这个进程记到的最新根多远滤。
     /// 一条都没有时 `None`。
     fn oldest_valid_root_in_the_ring(&self) -> Option<CheckpointTxg> {
         self.occupants
@@ -1083,6 +1092,9 @@ impl RootRingOccupancy {
                 RootRingOccupant::BadAtMount(txgs_it_can_hold) => {
                     Some(txgs_it_can_hold.oldest_txg_it_can_have_held())
                 }
+                RootRingOccupant::AllZeroAtMount {
+                    oldest_txg_it_can_have_held,
+                } => Some(*oldest_txg_it_can_have_held),
                 RootRingOccupant::AbandonedRoot { .. } => None,
             })
             .min()
@@ -1098,6 +1110,7 @@ impl RootRingOccupancy {
                 } => referenced_placements.as_slice(),
                 RootRingOccupant::ValidRoot { .. }
                 | RootRingOccupant::BadAtMount(_)
+                | RootRingOccupant::AllZeroAtMount { .. }
                 | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => &[],
             })
             .copied()
@@ -1117,6 +1130,7 @@ impl RootRingOccupancy {
                     Some((*ring_slot, *txgs_it_can_hold))
                 }
                 RootRingOccupant::ValidRoot { .. }
+                | RootRingOccupant::AllZeroAtMount { .. }
                 | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                 | RootRingOccupant::AbandonedRoot { .. } => None,
             })
@@ -1136,6 +1150,39 @@ impl RootRingOccupancy {
             .collect()
     }
 
+    /// 挂载那一刻读回全零、还记在表里的槽（[`RootRingOccupant::AllZeroAtMount`]），各钉在它可能住过的最旧 txg，按槽的次序。
+    /// 准入抬 F 的上限不并它们（[`Self::ring_slots_bad_at_mount`] 里没有它们，用户 2026-10-04 定）。
+    #[must_use]
+    pub fn ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held(
+        &self,
+    ) -> BTreeMap<RootRingSlot, CheckpointTxg> {
+        self.occupants
+            .iter()
+            .filter_map(|(ring_slot, occupant)| match occupant {
+                RootRingOccupant::AllZeroAtMount {
+                    oldest_txg_it_can_have_held,
+                } => Some((*ring_slot, *oldest_txg_it_can_have_held)),
+                RootRingOccupant::ValidRoot { .. }
+                | RootRingOccupant::BadAtMount(_)
+                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
+                | RootRingOccupant::AbandonedRoot { .. } => None,
+            })
+            .collect()
+    }
+
+    /// 回收门槛与 E2 重读看的那几个槽：挂载那一刻读坏的与读回全零的，各钉在它可能住过的最旧 txg（读回全零的照「自证不过」那一类钉住，
+    /// 用户 2026-10-05 定乙）。一个槽在表里只有一项，两类不会同时有它。
+    #[must_use]
+    pub fn ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held(
+        &self,
+    ) -> BTreeMap<RootRingSlot, CheckpointTxg> {
+        let mut pinned = self.ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held();
+        pinned.extend(
+            self.ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held(),
+        );
+        pinned
+    }
+
     /// 进程内重读那几个槽时判「按实例表仍然有效」用的那张表（D16（发布语义） 已定项 1「环里最旧有效根」：按实例表判仍然有效）。
     /// 装上根环表时是空表；可写挂载在写行那次发布的表一定下来、第一次记根之前换成它（`mount::establish_instance`）——
     /// 池级 checker 按镜像里最新那条根的实例表判，写行之后最新那条根带的就是写行那一张。挂载装上根环表到换表之间不记根，空表没人读；
@@ -1146,24 +1193,25 @@ impl RootRingOccupancy {
     }
 }
 
-/// 进程内重读一个挂载那一刻读坏的根环槽读到了什么（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定；用户 2026-10-04 定：
-/// 重读读出有效根或确认为空之前回收门槛不越过它，盘上全零的根槽另算「从没写过」）。封闭集合，`match` 不写通配臂。
+/// 进程内重读一个挂载那一刻读坏或读回全零的根环槽读到了什么（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定；用户 2026-10-04 定：
+/// 重读读出有效根之前回收门槛不越过它；2026-10-05 定乙：读回全零不算确认为空，照「自证不过」那一类钉着）。封闭集合，`match` 不写通配臂。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum RereadOfARootRingSlotBadAtMount {
     /// 读得出、自证过，按实例表仍然有效：照读出来的 txg 算（回收门槛取整个环，RGall，读出来多旧都算）；那一槽从此算「这个进程知道住着根的槽」。
     ValidRoot { checkpoint_txg: CheckpointTxg },
     /// 读得出、自证过，读出来的根按实例表被抛弃：当没有根。
     AbandonedByTheInstanceTable,
-    /// 读回全零：从没写过，确认为空、当没有根。
-    AllZeroNeverWritten,
+    /// 读回全零：不算确认为空（盘上那一槽可能住着一条有效根、这一次又读回了全零），照旧钉在它可能住过的最旧 txg（用户 2026-10-05 定乙）。
+    AllZero,
     /// 仍读不出，或读得出、不是全零而自证不过：照旧钉在它可能住过的最旧 txg。
     StillUnreadableOrNotSelfVerified,
 }
 
 /// 环里最旧有效根，挂载那一刻读坏的槽按 E2 并进来（D16（发布语义） 已定项 1，用户 2026-10-03 定；回收门槛取整个环，用户 2026-10-04 定
-/// RGall；读坏的槽钉在它可能住过的最旧 txg，用户 2026-10-04 定）：`valid_root_txgs` 是已知住着有效根的 txg，`bad_at_mount` 是挂载那一刻
-/// 读坏、各按它可能住过的最旧 txg 钉着的槽；两者里最小的那个若是一个读坏的槽，先重读那一槽（`reread`）：读出有效根就照它的 txg 换进去、
-/// 读出被抛弃的根或读回全零就拿掉，再取最小；仍读不出或读得出而自证不过才按钉着的 txg 算。
+/// RGall；读坏的槽钉在它可能住过的最旧 txg，用户 2026-10-04 定；读回全零的槽同样钉着，用户 2026-10-05 定乙）：`valid_root_txgs` 是已知
+/// 住着有效根的 txg，`bad_at_mount` 是挂载那一刻读坏或读回全零、各按它可能住过的最旧 txg 钉着的槽；两者里最小的那个若是一个钉着的槽，
+/// 先重读那一槽（`reread`）：读出有效根就照它的 txg 换进去、读出被抛弃的根就拿掉，再取最小；仍读不出、读得出而自证不过或读回全零
+/// 才按钉着的 txg 算。
 /// 交回最小的 txg（一条都没有时 `None`）与每个重读过的槽读到了什么（按重读的次序）。
 ///
 /// 循环：每轮要么换掉或拿掉一个读坏的槽（`bad_at_mount` 少一个），要么在最小的是一个仍读坏的槽、或最小的不是读坏的槽时返回；
@@ -1197,11 +1245,11 @@ pub fn oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
                 bad_at_mount.remove(&ring_slot);
                 valid_root_txgs.push(checkpoint_txg);
             }
-            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable
-            | RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten => {
+            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {
                 bad_at_mount.remove(&ring_slot);
             }
-            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified => {
+            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
+            | RereadOfARootRingSlotBadAtMount::AllZero => {
                 return (Some(pinned_txg), rereads);
             }
         }
@@ -1896,6 +1944,7 @@ impl PoolAllocator {
             Some(
                 RootRingOccupant::ValidRoot { .. }
                 | RootRingOccupant::BadAtMount(_)
+                | RootRingOccupant::AllZeroAtMount { .. }
                 | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. },
             )
             | None => {}
@@ -1908,12 +1957,12 @@ impl PoolAllocator {
         );
     }
 
-    /// 同 [`Self::record_root_written_by_this_process`]，记之前先照 E2 重读挂载那一刻读坏、记了这条根之后会是环里最小的那个槽
-    /// （D16（发布语义） 已定项 1，用户 2026-10-03 定；用户 2026-10-04 定：读坏的槽钉在它可能住过的最旧 txg，重读读出有效根或确认为空之前
+    /// 同 [`Self::record_root_written_by_this_process`]，记之前先照 E2 重读挂载那一刻读坏或读回全零、记了这条根之后会是环里最小的那个槽
+    /// （D16（发布语义） 已定项 1，用户 2026-10-03 定；用户 2026-10-04 定：读坏的槽钉在它可能住过的最旧 txg，重读读出有效根之前
     /// 回收门槛不越过它——读得出且自证过就照读出来的根算，按实例表有效的照它的 txg（回收门槛取整个环，RGall，多旧都算），被抛弃的当没有；
-    /// 读回全零的是从没写过，当没有；仍读不出或读得出而自证不过照旧钉着）。
+    /// 仍读不出、读得出而自证不过或读回全零照旧钉着：读回全零不算确认为空，用户 2026-10-05 定乙）。
     /// `reread` 读盘、判全零、自证与实例表（`mount::reread_of_a_root_ring_slot_bad_at_mount`）；
-    /// 这里只定重读哪一槽、按读到的改表。读坏的槽一个都不在最小的位置上时一个都不读。没装根环时只记根（什么都不做）。
+    /// 这里只定重读哪一槽、按读到的改表。钉着的槽一个都不在最小的位置上时一个都不读。没装根环时只记根（什么都不做）。
     pub fn record_root_written_by_this_process_rereading_slots_bad_at_mount(
         &mut self,
         checkpoint_txg: CheckpointTxg,
@@ -1934,14 +1983,14 @@ impl PoolAllocator {
                     | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                         checkpoint_txg: txg_of_the_occupant,
                     } => Some(*txg_of_the_occupant),
-                    RootRingOccupant::BadAtMount(_) | RootRingOccupant::AbandonedRoot { .. } => {
-                        None
-                    }
+                    RootRingOccupant::BadAtMount(_)
+                    | RootRingOccupant::AllZeroAtMount { .. }
+                    | RootRingOccupant::AbandonedRoot { .. } => None,
                 })
                 .chain(std::iter::once(checkpoint_txg))
                 .collect();
             let mut bad_at_mount =
-                occupancy.ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held();
+                occupancy.ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held();
             bad_at_mount.remove(&ring_slot_of_this_root);
             let (_oldest, rereads) = oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
                 &valid_root_txgs,
@@ -1960,11 +2009,11 @@ impl PoolAllocator {
                             },
                         );
                     }
-                    RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable
-                    | RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten => {
+                    RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {
                         occupancy.occupants.remove(&ring_slot);
                     }
-                    RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified => {}
+                    RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
+                    | RereadOfARootRingSlotBadAtMount::AllZero => {}
                 }
             }
         }
@@ -2891,18 +2940,19 @@ mod tests {
         );
     }
 
-    /// 重读读回全零：从没写过，确认为空，那一槽拿掉，最旧有效根是 13。
+    /// 重读读回全零：不算确认为空，照旧钉在它可能住过的最旧 txg 0（用户 2026-10-05 定乙；照「从没写过」拿掉时最旧有效根是 13）。
     #[test]
-    fn the_slot_bad_at_mount_reading_back_all_zero_on_the_reread_is_taken_as_never_written() {
+    fn the_slot_bad_at_mount_reading_back_all_zero_on_the_reread_stays_pinned_at_the_oldest_txg_it_can_have_held(
+    ) {
         assert_eq!(
             oldest_in_the_ring_when_the_slot_bad_at_mount_rereads_as(
-                RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten
+                RereadOfARootRingSlotBadAtMount::AllZero
             ),
             (
-                Some(CheckpointTxg(13)),
+                Some(CheckpointTxg(0)),
                 vec![(
                     RootRingSlot { region: 0, slot: 0 },
-                    RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten
+                    RereadOfARootRingSlotBadAtMount::AllZero
                 )]
             )
         );
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 769b294..ef25199 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1282,8 +1282,8 @@ fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(
 }
 
 /// 进程内重读一个挂载那一刻读坏的根环槽（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定）：只读一次，不重试；读得出、自证过、
-/// 按 `table` 仍然有效 ⇒ 照它的 txg；读得出、自证过而按 `table` 被抛弃 ⇒ 当没有根；读回全零 ⇒ 从没写过，确认为空、当没有根
-/// （用户 2026-10-04 定：盘上全零的根槽另算「从没写过」，重读读出有效根或确认为空之前回收门槛不越过它）；设备报读错或读得出、不是全零而自证不过
+/// 按 `table` 仍然有效 ⇒ 照它的 txg；读得出、自证过而按 `table` 被抛弃 ⇒ 当没有根；读回全零 ⇒ 不算确认为空，照旧钉在它可能住过的最旧 txg
+/// （用户 2026-10-05 定乙：读回全零的槽在回收门槛里照「自证不过」那一类钉住，只是不并进准入抬 F 的上限）；设备报读错或读得出、不是全零而自证不过
 /// ⇒ 照旧钉在它可能住过的最旧 txg（用户 2026-10-04 定改法 A，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1：
 /// 这一刻读到「自证不过」可能是一次读坏的有效根，当没有根在这上面让回收门槛越过盘上一条有效根）。
 #[must_use]
@@ -1311,7 +1311,7 @@ pub fn reread_of_a_root_ring_slot_bad_at_mount<Reader: PoolReader + ?Sized>(
             RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable
         }
         RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => {
-            RereadOfARootRingSlotBadAtMount::AllZeroNeverWritten
+            RereadOfARootRingSlotBadAtMount::AllZero
         }
         RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => {
             RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
@@ -1677,7 +1677,8 @@ fn placements_referenced_by_root<Device: BlockDevice>(
 }
 
 /// 挂载重建分配器与进程内重读时读一个根环槽读到了什么：在 [`read_root_ring_slot`] 的分法之上，把读回全零的槽单分出来
-/// （用户 2026-10-04 定：盘上全零的根槽另算「从没写过」，不并进准入抬 F 的上限；它也就没有根可罩，回收门槛不钉它）。
+/// （用户 2026-10-04 定：盘上全零的根槽不并进准入抬 F 的上限；2026-10-05 定乙：回收门槛里照「自证不过」那一类钉住，
+/// 一次读回全零的槽盘上可能住着一条有效根）。
 /// 别的读者（择根、算上限那一遍、清扫）照旧用 [`read_root_ring_slot`]：清扫把全零与读不出同待遇（D18（块里携带什么信息） 已定项 11）。
 #[allow(
     clippy::large_enum_variant,
@@ -1688,7 +1689,7 @@ enum RootRingSlotReadingTellingNeverWrittenApart {
     SelfVerified(RootRecord),
     /// 设备报读错，或读得出、不是全零而自证不过（校验和不过、fsid 不是本池的）。
     BadAndNotAllZero,
-    /// 读得出、每个字节都是 0：从没写过。
+    /// 读得出、每个字节都是 0：多半从没写过，也可能是一次读坏的有效根（回收门槛照这一种钉住）。
     AllZeroNeverWritten,
 }
 
@@ -1726,7 +1727,8 @@ fn read_root_ring_slot_telling_never_written_apart<Reader: PoolReader + ?Sized>(
 ///   `research/prompts/m3-ring-r3-main-verification.md` 第二节第 4 行）：准入抬 F 的上限照 Q3 按它并。上沿取新实例第一次发布之前的那个 txg：
 ///   读坏的槽里可能住着比环里读得出的最新根还新的根（被藏住的最新根、由记录施加出来的那一版那次发布的根）。
 ///
-/// 一个槽在 [0, first_txg) 里一个 txg 都住不了（年轻的环里还没轮到的槽）就不交回。
+/// 一个槽在 [0, first_txg) 里一个 txg 都住不了（年轻的环里还没轮到的槽）就不交回。挂载时读回全零的槽也经它算（`rebuilt_allocator`），
+/// 那一份只用可能住过的最旧 txg（回收门槛照「自证不过」那一类钉住，用户 2026-10-05 定乙；不并进上限）。
 ///
 /// 循环：最大 txg 只走区间尾 min(3S, 区间长) 个 txg（与 first_txg 多大无关：外来记录的 txg 可以贴着 `u64::MAX`），最旧 txg 只走
 /// [0, min(3S, first_txg)) ——`target_for_publish` 只看 txg mod 3S，一个槽最小的 txg 落在第一个周期里、区间里最大的 txg 落在区间尾一个周期里；
@@ -1872,13 +1874,23 @@ fn rebuilt_allocator<Device: BlockDevice>(
             | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
         })
         .collect();
+    let ring_slots_reading_back_all_zero: BTreeSet<RootRingSlot> = ring_slot_readings
+        .iter()
+        .filter_map(|(ring_slot, reading)| match reading {
+            RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => Some(*ring_slot),
+            RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(_)
+            | RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => None,
+        })
+        .collect();
     let roots: Vec<RootRecord> = roots_with_ring_slots
         .iter()
         .map(|(_, root)| *root)
         .collect();
     let is_abandoned = |root: &RootRecord| abandoned_by_table(root, &newest_table);
     // 挂载时读坏的槽能住哪几个 txg（[`txgs_ring_slots_bad_at_mount_can_hold`]）：N = 环里读得出的最新有效根的 txg（D16（发布语义） 已定项 1
-    // 「挂载时读不出的槽」那一句：挂载重建分配器时取它、不取 first_txg − 1），读回全零的槽从没写过、不在其内（用户 2026-10-04 定）。
+    // 「挂载时读不出的槽」那一句：挂载重建分配器时取它、不取 first_txg − 1）。读回全零的槽按同一个算法另算一份、只取可能住过的最旧 txg：
+    // 回收门槛里照「自证不过」那一类钉住（用户 2026-10-05 定乙），准入抬 F 的上限不并它（用户 2026-10-04 定），能住的最大 txg 用不到。
+    // 一个 txg 都住不了的槽（年轻的环里还没轮到的）两类都不交回。
     let window_anchor = roots
         .iter()
         .filter(|root| !is_abandoned(root))
@@ -1892,8 +1904,23 @@ fn rebuilt_allocator<Device: BlockDevice>(
             slots_per_region,
         )
     });
+    let ring_slots_reading_back_all_zero_at_mount: Vec<(RootRingSlot, CheckpointTxg)> =
+        window_anchor
+            .map_or_else(Vec::new, |anchor| {
+                txgs_ring_slots_bad_at_mount_can_hold(
+                    &ring_slots_reading_back_all_zero,
+                    anchor,
+                    first_txg,
+                    slots_per_region,
+                )
+            })
+            .into_iter()
+            .map(|(ring_slot, txgs_it_can_hold)| {
+                (ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held())
+            })
+            .collect();
     // 环里有效根的最小 txg（D16（发布语义） 已定项 1「环里最旧有效根」，用户 2026-10-04 定 RGall：不按一圈窗口滤）；
-    // 挂载时读坏的槽钉在它可能住过的最旧 txg（用户 2026-10-04 定）。
+    // 挂载时读坏的槽钉在它可能住过的最旧 txg（用户 2026-10-04 定），读回全零的槽同样钉着（用户 2026-10-05 定乙）。
     let oldest_valid_root_in_the_ring = roots
         .iter()
         .filter(|root| !is_abandoned(root))
@@ -1903,6 +1930,11 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 .iter()
                 .map(|(_, txgs_it_can_hold)| txgs_it_can_hold.oldest_txg_it_can_have_held()),
         )
+        .chain(
+            ring_slots_reading_back_all_zero_at_mount
+                .iter()
+                .map(|(_, oldest_txg_it_can_have_held)| *oldest_txg_it_can_have_held),
+        )
         .min();
     // 卸载半途崩溃之后的下一次可写挂载只放开一截（D16（发布语义） 已定项 1「正常卸载」那一行，用户 2026-10-04 选甲，
     // `research/prompts/m3-batch14-code-r1-forks.md` 第 1 行）：卸载那一串先写了系统配置里的新 F、带新 F 的根没落满每块盘就崩时，
@@ -1912,7 +1944,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
     // [`newest_valid_root_txg_on_every_device`]）；其余到原门槛为止的那一截扣住，这次挂载的暖机在每块盘上都落了根之后放开（`establish_instance`）。
     // 某块盘一条有效根都读不出（m 取不到）时当场只放到一圈窗口里最旧有效根（用户 2026-10-04 选甲，`research/prompts/m3-batch16-code-r1-forks.md`
     // 第 1 行；判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「再挂期间某块盘根环整段读坏」那一行），RGall 之后这个量是
-    // 环里最旧有效根：那块盘上真正最新的根这一遍看不到，它引用的单元释放代可以高到 F_生效；它与回收门槛同一个读数（挂载时读坏的槽钉在它可能住过的最旧 txg 并进）。
+    // 环里最旧有效根：那块盘上真正最新的根这一遍看不到，它引用的单元释放代可以高到 F_生效；它与回收门槛同一个读数（挂载时读坏、读回全零的槽钉在它可能住过的最旧 txg 并进）。
     // 这一遍一条有效根都没读出、环里最旧有效根也取不到时，「只到环里最旧有效根」的那一截是空的：当场一个都不放，整截扣到暖机落满。
     let device_of_txg = |txg: CheckpointTxg| {
         let target = target_for_publish(txg, slots_per_region);
@@ -1960,14 +1992,25 @@ fn rebuilt_allocator<Device: BlockDevice>(
         abandoned_roots_unreadable,
         placements_referenced_by_abandoned_root,
     } = shadow_ledger_computed;
-    // 挂载时读坏的槽在根环表里记成「挂载时读坏」，带它能住的 txg：回收门槛钉在它可能住过的最旧 txg、准入抬 F 的上限按它能住的最大 txg 并，
-    // 直到这个进程写到那个槽、或进程内重读读出有效根或确认为空（用户 2026-10-04 定）。同一个槽这一遍读出了根的，读出来的那条在后面、盖掉它
-    // （这一遍读的是同一个槽，两样不会同时成立，留着次序只为不依赖这一点）。
+    // 挂载时读坏的槽在根环表里记成「挂载时读坏」，带它能住的 txg：回收门槛钉在它可能住过的最旧 txg、准入抬 F 的上限按它能住的最大 txg 并；
+    // 读回全零的槽记成「挂载时读回全零」，只在回收门槛里钉在它可能住过的最旧 txg（用户 2026-10-05 定乙）。两类都钉到这个进程写到那个槽、
+    // 或进程内重读读出有效根或被抛弃的根为止（用户 2026-10-04 定）。同一个槽这一遍读出了根的，读出来的那条在后面、盖掉它
+    // （这一遍读的是同一个槽，几样不会同时成立，留着次序只为不依赖这一点）。
     let occupants: Vec<(RootRingSlot, RootRingOccupant)> = ring_slots_bad_at_mount
         .iter()
         .map(|(ring_slot, txgs_it_can_hold)| {
             (*ring_slot, RootRingOccupant::BadAtMount(*txgs_it_can_hold))
         })
+        .chain(ring_slots_reading_back_all_zero_at_mount.iter().map(
+            |(ring_slot, oldest_txg_it_can_have_held)| {
+                (
+                    *ring_slot,
+                    RootRingOccupant::AllZeroAtMount {
+                        oldest_txg_it_can_have_held: *oldest_txg_it_can_have_held,
+                    },
+                )
+            },
+        ))
         .chain(roots_with_ring_slots.iter().map(|(ring_slot, root)| {
             let occupant = if is_abandoned(root) {
                 RootRingOccupant::AbandonedRoot {
@@ -2640,9 +2683,9 @@ pub fn unmount<Device: BlockDevice>(
     ))
 }
 
-/// 这个进程分配器上那张根环表里挂载那一刻读坏（读不出，或读得出、不是全零而自证不过）、之后没写过也没重读出有效根或确认为空的槽，
-/// 各按它能住的最大 txg（`allocator::RootRingOccupancy::ring_slots_bad_at_mount`，Q3 两类都并；读回全零的槽从没写过、不在其内，
-/// 用户 2026-10-04 定）；没装根环表的分配器给空表。准入抬 F 的上限按它封顶（[`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]）。
+/// 这个进程分配器上那张根环表里挂载那一刻读坏（读不出，或读得出、不是全零而自证不过）、之后没写过也没重读出有效根或被抛弃的根的槽，
+/// 各按它能住的最大 txg（`allocator::RootRingOccupancy::ring_slots_bad_at_mount`，Q3 两类都并；读回全零的槽不在其内，
+/// 用户 2026-10-04 定，它们只在回收门槛里钉着）；没装根环表的分配器给空表。准入抬 F 的上限按它封顶（[`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]）。
 #[must_use]
 pub fn ring_slots_bad_at_mount_by(
     allocator: &PoolAllocator,
@@ -2895,7 +2938,8 @@ pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
 }
 
 /// 抬 F 那一遍的回收门槛里「环里最旧有效根」（D16（发布语义） 已定项 1，用户 2026-10-04 定 RGall：不按一圈窗口滤）：这一遍读出来的根里
-/// 按 `table` 有效的，并上分配器根环表里挂载那一刻读坏、这一遍也没在那一槽读出根的槽（各钉在它可能住过的最旧 txg，用户 2026-10-04 定；
+/// 按 `table` 有效的，并上分配器根环表里挂载那一刻读坏或读回全零、这一遍也没在那一槽读出根的槽（各钉在它可能住过的最旧 txg，用户 2026-10-04 定，
+/// 读回全零的 2026-10-05 定乙；
 /// 并进来是 RR4，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表一，攻方 B1），取最小；最小的是一个读坏的槽时照 E2
 /// 先重读那一槽（[`oldest_txg_in_the_ring_rereading_slots_bad_at_mount`]，与进程内记根同一取法）。只读盘，不改分配器。
 fn oldest_valid_root_in_the_ring_for_raising_the_floor<Reader: PoolReader + ?Sized>(
@@ -2919,7 +2963,7 @@ fn oldest_valid_root_in_the_ring_for_raising_the_floor<Reader: PoolReader + ?Siz
         .root_ring_occupancy()
         .map_or_else(
             BTreeMap::new,
-            RootRingOccupancy::ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held,
+            RootRingOccupancy::ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held,
         )
         .into_iter()
         .filter(|(ring_slot, _)| !ring_slots_read_from_the_ring.contains(ring_slot))
diff --git a/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs b/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs
index 31e7961..adb6b03 100644
--- a/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs
+++ b/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs
@@ -289,7 +289,8 @@ fn raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_t
     assert_eq!(violations_of(&image), Vec::<String>::new(), "抬 F 之后");
 }
 
-/// 覆盖写到 txg 37 之后进程退出，再可写挂载（实例 3）：洞里 txg 7 的根所在的那一槽（txg 31 的槽）从这一次挂载起按 `slot_reads` 报设备错。
+/// 覆盖写到 txg 37 之后进程退出，再可写挂载（实例 3）：洞里 txg 7 的根所在的那一槽（txg 31 的槽）从这一次挂载起每次读都坏，
+/// 按 `read_back` 报设备错或报成功、字节全 0。
 /// 交回盘（带读故障）、它的读故障、起点盘面、录下的流与挂载交回的分配器、现行那一版。
 struct RemountReadingTheSlotOfTheStaleRootBad {
     devices: Vec<(
@@ -311,7 +312,9 @@ impl RemountReadingTheSlotOfTheStaleRootBad {
     }
 }
 
-fn remount_reading_the_slot_of_the_stale_root_bad() -> RemountReadingTheSlotOfTheStaleRootBad {
+fn remount_reading_the_slot_of_the_stale_root_bad(
+    read_back: UnreadableRangeReadBack,
+) -> RemountReadingTheSlotOfTheStaleRootBad {
     let pool = pool_after_the_hole_overwritten_through(LAST_TXG_OF_INSTANCE_TWO, &mut |_, _, _| {});
     let image_before_the_remount = memory_pool_of_sparse_devices(&pool.devices);
     let stream = SharedStream::retaining_contents();
@@ -320,7 +323,7 @@ fn remount_reading_the_slot_of_the_stale_root_bad() -> RemountReadingTheSlotOfTh
             CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
             FailingReadsOfARange::Every,
         )],
-        UnreadableRangeReadBack::DeviceError,
+        read_back,
     );
     let mut devices = with_unreadable_ranges(
         crash_state_devices(&image_before_the_remount, &[], &[], &stream),
@@ -352,7 +355,8 @@ fn remount_reading_the_slot_of_the_stale_root_bad() -> RemountReadingTheSlotOfTh
 #[test]
 fn a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated(
 ) {
-    let mut remounted = remount_reading_the_slot_of_the_stale_root_bad();
+    let mut remounted =
+        remount_reading_the_slot_of_the_stale_root_bad(UnreadableRangeReadBack::DeviceError);
     let slot_of_the_stale_root = target_for_publish(
         CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
         parameters().geometry.root_ring_slots_per_region,
@@ -434,7 +438,67 @@ fn a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every
 #[test]
 fn raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated(
 ) {
-    let mut remounted = remount_reading_the_slot_of_the_stale_root_bad();
+    let mut remounted =
+        remount_reading_the_slot_of_the_stale_root_bad(UnreadableRangeReadBack::DeviceError);
+    raise_rollback_floor(
+        &parameters(),
+        &mut remounted.devices,
+        &mut remounted.allocator,
+        &mut remounted.current,
+        CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
+        ShadowLedger::On,
+    )
+    .expect("抬 F 到 7");
+    let image = remounted.image_now();
+    assert_eq!(
+        (
+            slots_of_the_stale_root_left_free(&image, &remounted.allocator),
+            violations_of(&image)
+        ),
+        (Vec::new(), Vec::<String>::new()),
+        "抬 F 到 7 之后：txg 7 引用的单元一个都不空闲，那几条不变量不红"
+    );
+}
+
+/// 场景：同上再挂，洞里 txg 7 的根所在那一槽从再挂起每次读都报成功、字节全 0（盘上那一槽里是 txg 7 的根），在这一次挂载里把 F 抬到 7。
+/// 预期：那一槽在根环表里记成挂载时读回全零、钉在它可能住过的最旧 txg 7（用户 2026-10-05 定乙：回收门槛里照「自证不过」那一类钉住），
+/// 不进准入抬 F 的上限要并的槽；抬 F 那一遍的回收门槛 = max(7, 环里最旧有效根)，重读读回全零照旧钉在 7，门槛是 7：txg 7 那一版引用的单元
+/// 一个都不空闲，抬完之后盘上那几条不变量不红。抬 F 那一遍不并读回全零的槽时门槛是读得出的最旧有效根 14，txg 7 引用的单元放开。
+#[test]
+fn raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated(
+) {
+    let mut remounted =
+        remount_reading_the_slot_of_the_stale_root_bad(UnreadableRangeReadBack::Zeros);
+    let slot_of_the_stale_root = target_for_publish(
+        CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
+        parameters().geometry.root_ring_slots_per_region,
+    );
+    let occupancy = remounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("可写挂载装了根环表");
+    assert_eq!(
+        (
+            occupancy
+                .ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held()
+                .get(&slot_of_the_stale_root)
+                .copied(),
+            occupancy
+                .ring_slots_bad_at_mount()
+                .contains_key(&slot_of_the_stale_root)
+        ),
+        (Some(CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE)), false),
+        "根环表把那一槽记成挂载时读回全零、钉在 7，不进上限要并的槽"
+    );
+    let after_the_remount = remounted.image_now();
+    assert_eq!(
+        (
+            slots_of_the_stale_root_left_free(&after_the_remount, &remounted.allocator),
+            violations_of(&after_the_remount)
+        ),
+        (Vec::new(), Vec::<String>::new()),
+        "再挂之后：挂载时的回收门槛钉在 7，txg 7 引用的单元一个都不空闲，那几条不变量不红"
+    );
     raise_rollback_floor(
         &parameters(),
         &mut remounted.devices,
diff --git a/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs b/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs
index aecdad3..71669d2 100644
--- a/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs
+++ b/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs
@@ -3,8 +3,9 @@
 //! `research/prompts/m3-root-ring-code-r2-main-verification.md` 第四节 RS3 与第六节；同日 Q3 重定成两类都并，代码轮第三轮判决
 //! `research/prompts/m3-root-ring-code-r3-main-verification.md` 第二节 K2 与第六节，攻方 `k2-zeros` 那一形）。
 //! 上限本身取环里全部有效根（用户 2026-10-04 定 RGall，`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）。
-//! 盘上全零的根槽另算「从没写过」，不并进上限（用户 2026-10-04 定，条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）：
-//! 一槽从挂载起读回全零、而盘上住着一条非空有效根时 F 越过它，是定案知情接受的代价（没洞的那一份钉着）。
+//! 盘上全零的根槽不并进上限（用户 2026-10-04 定，条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）：
+//! 一槽从挂载起读回全零、而盘上住着一条非空有效根时 F 越过它，是定案知情接受的代价（没洞的那一份钉着）。回收门槛里它照「自证不过」那一类
+//! 钉在可能住过的最旧 txg（用户 2026-10-05 定乙）：上限的数不变，抬 F 之前那一槽里那条根引用的单元不回收。
 //!
 //! 两份盘面（S = 4 的 12 槽环）：
 //! - 有洞的那一份（攻方原型那一形）：实例 1 覆盖写到 txg 17；三次可写挂载崩在写行那次根槽写之前（洞 18、19、20：那几槽里留着实例 1 上一圈的
@@ -12,7 +13,7 @@
 //!   RGall 之后洞里那三条旧根是有效根、非空：第 4 新的不同状态落在 txg 6，上限 6，X 读不读得出都一样。
 //! - 没洞的那一份：实例 1 覆盖写到 X = 17，进程退出；可写挂载起 X 那一槽读坏，写行、暖机之后空发布推到 28（X 成了环里最旧的根）。
 //!   读得出的有效根是 18–28：最旧的 18 跟「没有这两棵树」比、算非空，其余都空，照读得出的根算上限是 18；X 那一槽读不出时按它能住的最大 txg 17
-//!   并进来，上限 17；读回全零时不并（从没写过），上限 18、F 越过盘上完好的 X。X 读得出时上限也是 17（X 是最旧的有效根）。
+//!   并进来，上限 17；读回全零时不并，上限 18、F 越过盘上完好的 X。X 读得出时上限也是 17（X 是最旧的有效根）。
 mod common;
 mod common_root_ring_holes;
 
@@ -27,12 +28,13 @@ use common_root_ring_holes::{
 };
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
 use singlefs_core::allocator::PoolAllocator;
 use singlefs_core::mount::{
     mount_writable, raise_rollback_floor_to_the_admission_ceiling, RaiseToTheAdmissionCeiling,
     ShadowLedger,
 };
+use singlefs_core::recovery::allocation_records_under_root;
 use singlefs_core::root_ring::RootRingSlot;
 use singlefs_core::root_ring::{slot_offset, target_for_publish};
 use singlefs_core::transaction::{PoolWriter, TransactionOutput};
@@ -211,6 +213,45 @@ fn highest_txg_the_table_records_for(
         .map(|txgs_it_can_hold| txgs_it_can_hold.highest_txg_it_can_hold())
 }
 
+/// 根环表里 `ring_slot` 按挂载时读回全零记着、钉着的 txg（没这样记着时 `None`）。
+fn txg_the_table_pins_for_the_slot_reading_back_all_zero(
+    pool: &PoolJustBeforeTheAdmissionRaise,
+    ring_slot: RootRingSlot,
+) -> Option<CheckpointTxg> {
+    pool.allocator
+        .root_ring_occupancy()
+        .expect("可写挂载装了根环表")
+        .ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held()
+        .get(&ring_slot)
+        .copied()
+}
+
+/// txg `checkpoint_txg` 那一版的账里仍分配、而分配器上空着（可再分配）的槽：(盘, 槽)。
+fn slots_of_the_version_left_free(
+    pool: &PoolJustBeforeTheAdmissionRaise,
+    checkpoint_txg: u64,
+) -> Vec<(DeviceIdentity, SlotNumber)> {
+    let image = pool.image_now();
+    let root = readable_root_at_txg(&image, checkpoint_txg).expect("那一版的根在盘上、读得出");
+    allocation_records_under_root(&image, &root)
+        .expect("那一版的账读得出")
+        .iter()
+        .filter(|record| !record.is_released)
+        .flat_map(|record| {
+            (record.slot.0..record.slot.0 + u64::from(record.span_slots))
+                .map(move |slot| (record.device, SlotNumber(slot)))
+        })
+        .filter(|(device, slot)| {
+            pool.allocator
+                .devices
+                .iter()
+                .find(|device_map| device_map.device == *device)
+                .expect("账里的盘在分配器里")
+                .is_free(*slot)
+        })
+        .collect()
+}
+
 /// 抬到上限，交回抬到的上限与抬完之后现行那一版带的 F。
 fn raise_to_the_admission_ceiling(
     pool: &mut PoolJustBeforeTheAdmissionRaise,
@@ -308,9 +349,17 @@ fn admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_sin
             .root_ring_slots_per_region,
     );
     assert_eq!(
-        highest_txg_the_table_records_for(&pool, x_slot),
-        None,
-        "前提：X 那一槽读回全零，从没写过，根环表不记它"
+        (
+            highest_txg_the_table_records_for(&pool, x_slot),
+            txg_the_table_pins_for_the_slot_reading_back_all_zero(&pool, x_slot)
+        ),
+        (
+            None,
+            Some(CheckpointTxg(
+                X_TXG_AFTER_THE_HOLES % (3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS)
+            ))
+        ),
+        "前提：X 那一槽读回全零，不进上限要并的槽；回收门槛里钉在它可能住过的最旧 txg 9"
     );
     assert_eq!(
         raise_to_the_admission_ceiling(&mut pool),
@@ -348,11 +397,14 @@ fn admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_th
 }
 
 /// 场景：没洞的那一份，X 那一槽抬 F 那一次挂载起每次读都报成功、字节全 0。
-/// 预期：读回全零的槽另算「从没写过」，根环表不记它、准入抬 F 的上限不并它（用户 2026-10-04 定）：上限照读得出的根取 18，F 抬到 18，
+/// 预期：读回全零的槽准入抬 F 的上限不并它（用户 2026-10-04 定）：上限照读得出的根取 18，F 抬到 18，
 /// 越过盘上完好的非空有效根 X = 17——X 在抬 F 那一串写掉它那一槽之前就出了回退候选集，「保留最近 4 个状态」在这一形上少一个。
 /// 这是定案知情接受的代价（用户 2026-10-04，`records/2026-09-30-里程碑三开工第一轮.md`）。抬完之后池级 checker 看不出它：抬 F 那一串的
 /// 第一次空发布（txg 29 = X + 3S）就写掉了 X 那一槽，镜像里 F 之前的根是 18–28，按它们算的上限是 18，那几条不变量（含 I-7.9）都不红。
 /// 照 Q3（读不出与自证不过两类都并、全零算自证不过）时上限 17、F 停在 X。
+/// 回收门槛里那一槽照「自证不过」那一类钉在它可能住过的最旧 txg 5（用户 2026-10-05 定乙），上限的数不跟着变：抬 F 之前（txg 28）X 那一版
+/// 引用的单元一个都不空闲。照「从没写过」不钉时（第 10b 批那一版）挂载之后的空发布把门槛推到读得出的最旧有效根 18，X 那一版里释放代 18 的
+/// 那几个单元在抬 F 之前就回收了。
 #[test]
 fn admission_raising_the_floor_without_holes_passes_a_slot_reading_back_zeros_since_the_mount_as_the_known_cost(
 ) {
@@ -365,14 +417,27 @@ fn admission_raising_the_floor_without_holes_passes_a_slot_reading_back_zeros_si
             .root_ring_slots_per_region,
     );
     assert_eq!(
-        highest_txg_the_table_records_for(&pool, x_slot),
-        None,
-        "前提：X 那一槽读回全零，从没写过，根环表不记它"
+        (
+            highest_txg_the_table_records_for(&pool, x_slot),
+            txg_the_table_pins_for_the_slot_reading_back_all_zero(&pool, x_slot)
+        ),
+        (
+            None,
+            Some(CheckpointTxg(
+                X_TXG_WITHOUT_HOLES % (3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS)
+            ))
+        ),
+        "前提：X 那一槽读回全零，不进上限要并的槽；回收门槛里钉在它可能住过的最旧 txg 5"
     );
     assert!(
         readable_root_at_txg(&pool.image_now(), X_TXG_WITHOUT_HOLES).is_some(),
         "前提：抬 F 之前盘上 X 那一槽里是完好的 txg 17 的根"
     );
+    assert_eq!(
+        slots_of_the_version_left_free(&pool, X_TXG_WITHOUT_HOLES),
+        Vec::new(),
+        "抬 F 之前：X 那一槽钉在 5，X 那一版引用的单元一个都不空闲"
+    );
     assert_eq!(
         raise_to_the_admission_ceiling(&mut pool),
         (
diff --git a/crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs b/crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs
index d8f5600..3c14834 100644
--- a/crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs
+++ b/crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs
@@ -1,14 +1,16 @@
 //! 进程内记根时重读挂载那一刻读坏的根环槽（D16（发布语义） 已定项 1「环里最旧有效根」那一行的 E2 句与「根槽这一次读坏」那一行）：
 //! 重读那一次的几种结局各记成什么、判「按实例表有效」用哪张表、产品路径上记根的几处各自走到重读。
 //!
-//! 读坏的槽在回收门槛里钉在它可能住过的最旧 txg，重读读出有效根或确认为空之前门槛不越过它；重读时机照 E2（根环表里这一项成了最小的那个时）；
-//! 盘上全零的根槽另算「从没写过」（用户 2026-10-04 定，条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）。钉在最旧 txg 之后
-//! 那一槽挂载那一刻起多半就是根环表里最小的那个，挂载之后第一次记根就重读。
+//! 读坏的槽在回收门槛里钉在它可能住过的最旧 txg，重读读出有效根或被抛弃的根之前门槛不越过它；重读时机照 E2（根环表里这一项成了最小的那个时）；
+//! 读回全零的根槽在回收门槛里照「自证不过」那一类钉住，只是不并进准入抬 F 的上限（用户 2026-10-04 定全零不并进上限，2026-10-05 定乙；
+//! 条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）。钉在最旧 txg 之后那一槽挂载那一刻起多半就是根环表里最小的那个，
+//! 挂载之后第一次记根就重读。
 //!
 //! - 读得出而自证不过，与仍读不出同一处置，照旧钉着（用户 2026-10-04 定改法 A，代码轮第二轮判决
 //!   `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1；攻方 `research/prompts/m3-root-ring-code-r2-opus-model/proto/src/bin/e2_reread_flip.rs`
 //!   那一形：一条有效根所在的槽挂载那一刻读错、重读那一次读坏，改之前回收门槛越过它、I-3.1 记账少）。
-//! - 读回全零：从没写过，确认为空、拿掉（同一目录 `front_hole_eio.rs` 那一形：从没写过的槽加挂载那一刻读错，改法 A 下按记的 txg 多记账的那一版不再有）。
+//! - 读回全零：不算确认为空，照旧钉着（用户 2026-10-05 定乙）。同一目录 `front_hole_eio.rs` 那一形（从没写过的槽加挂载那一刻读错）上，
+//!   改法 A 认下的「重读那一版按记的 txg 多记账、I-3.1 红在记账多的一边」因此回来了，到这个进程写掉那一槽为止（一版）。
 //! - 读得出、自证过、按实例表有效的照读出来的 txg 算，那一槽从此算「这个进程知道住着根的槽」（用户 2026-10-04 定）；
 //!   读得出、自证过而按实例表被抛弃的当没有根；判有效用的表写行之后是写行那一张（同一份判决第二节 RS2，正推腿探针 p1、p2）。
 //! - 记根的产品路径：带单元的发布（`transaction::publish_admitted`，已有变异行管它）、树表 0 条那一版上要写行的写行发布
@@ -19,7 +21,7 @@ mod common;
 mod common_root_ring_holes;
 
 use std::cell::RefCell;
-use std::collections::BTreeSet;
+use std::collections::BTreeMap;
 use std::rc::Rc;
 
 use common::{
@@ -508,9 +510,12 @@ fn a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_be
 /// 场景（攻方 `front_hole_eio 4 9 mount`）：实例 1 写到 3，实例 2 覆盖写到 8，txg 9 那次覆盖写崩在根槽写之前
 /// （记录已持久；环还没转过来，那一槽从没写过、盘上全零）；实例 3 可写挂载那一刻读那一槽报读错，之后照读；覆盖写到 9 + 2 × 3S。
 /// 预期：那一槽钉在它可能住过的最旧 txg 9；环里还有比 9 旧的有效根时不重读，txg 20 那一版它成了根环表里最小的那个、重读读回全零：
-/// 从没写过，确认为空、拿掉。每一版那几条不变量都不红（改法 A 下这一形重读那一版按 9 多记账、I-3.1 红，全零另算从没写过之后没有了）。
+/// 不算确认为空，照旧钉在 9（用户 2026-10-05 定乙），txg 21 这个进程写掉那一槽才解开。被引用单元那几条每一版都不红；只有 txg 20 那一版
+/// I-3.1 红、只许红在记账多的一边——钉着的 9 比盘上环里最旧的有效根 10 旧，池级 checker 读盘上全零的那一槽、按 10 算门槛，安全方向，
+/// 定案的代价（与改法 A 当初认下的是同一版）。照「从没写过」拿掉时 txg 20 那一版也不红。
 #[test]
-fn a_front_hole_unreadable_at_mount_reread_as_all_zero_is_taken_as_never_written() {
+fn a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it(
+) {
     let parameters = parameters_of_the_ring_hole_pools();
     let image_with_the_front_hole = {
         let mut devices: RingHoleDevices = devices_of(&pool_overwritten_through(3));
@@ -580,13 +585,13 @@ fn a_front_hole_unreadable_at_mount_reread_as_all_zero_is_taken_as_never_written
         "前提：根环表把那一槽记成挂载时读坏，钉在 9（能住的最大 txg 也是 9）"
     );
     let ring_length = 3 * parameters.geometry.root_ring_slots_per_region.count();
-    let mut versions_with_a_violation = BTreeSet::new();
+    let mut violations_of_each_version: BTreeMap<u64, Vec<String>> = BTreeMap::new();
     // 迭代上界：覆盖写到 9 + 2 × 3S；跨轮携带的是现行那一版与红过的版本。
     while session.current.root.checkpoint_txg.0 < TXG_OF_THE_FRONT_HOLE + 2 * ring_length {
         session.overwrite_once();
         let violations = violations_of(&session.image_now());
         if !violations.is_empty() {
-            versions_with_a_violation.insert(session.current.root.checkpoint_txg.0);
+            violations_of_each_version.insert(session.current.root.checkpoint_txg.0, violations);
         }
         let txg = session.current.root.checkpoint_txg.0;
         if txg == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD - 1 {
@@ -601,18 +606,63 @@ fn a_front_hole_unreadable_at_mount_reread_as_all_zero_is_taken_as_never_written
                     session.txgs_recorded_as_bad_at_mount(),
                     session.slot_is_known_to_hold_a_root()
                 ),
-                (None, false),
-                "txg 20：重读读回全零，从没写过，拿掉、不算知道住着根的槽"
+                (
+                    Some(TxgsARingSlotBadAtMountCanHold::new(
+                        CheckpointTxg(TXG_OF_THE_FRONT_HOLE),
+                        CheckpointTxg(TXG_OF_THE_FRONT_HOLE)
+                    )),
+                    false
+                ),
+                "txg 20：重读读回全零，不算确认为空，照旧钉在 9、不算知道住着根的槽"
+            );
+        }
+        if txg == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD + 1 {
+            assert_eq!(
+                (
+                    session.txgs_recorded_as_bad_at_mount(),
+                    session.slot_is_known_to_hold_a_root()
+                ),
+                (None, true),
+                "txg 21：这个进程写掉那一槽，不再钉着、算知道住着根的槽"
             );
         }
     }
     assert_eq!(
-        versions_with_a_violation,
-        BTreeSet::new(),
-        "每一版那几条不变量都不红"
+        violations_of_each_version
+            .keys()
+            .copied()
+            .collect::<Vec<u64>>(),
+        vec![TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD],
+        "只有重读读回全零、照旧钉着的 txg 20 那一版红：{violations_of_each_version:?}"
+    );
+    assert!(
+        violations_of_each_version
+            .values()
+            .flatten()
+            .all(|violation| violation.starts_with("I-3.1：")
+                && allocated_statistic_counts_more_than_the_walk(violation)),
+        "txg 20 那一版只许 I-3.1 红在记账多的一边：{violations_of_each_version:?}"
     );
 }
 
+/// I-3.1 那一条红在记账比遍历多的一边：说明里「记账的已分配 Some(a)，遍历全部有效根得到 b」a 大于 b
+/// （与 `raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` 同一个判法另写一份）。
+fn allocated_statistic_counts_more_than_the_walk(violation: &str) -> bool {
+    let after = |marker: &str| {
+        violation.split(marker).nth(1).and_then(|rest| {
+            rest.chars()
+                .take_while(char::is_ascii_digit)
+                .collect::<String>()
+                .parse::<u64>()
+                .ok()
+        })
+    };
+    matches!(
+        (after("记账的已分配 Some("), after("遍历全部有效根得到 ")),
+        (Some(counted), Some(walked)) if counted > walked
+    )
+}
+
 /// 场景：实例 1 覆盖写到 txg 5（最新那条根 a = 5）；崩溃恢复抛弃 a（它的根槽与数据单元一时读不出，择根落到 4，实例 2 写行 6、暖机 7），
 /// 实例 2 再覆盖写两次（8、9）；实例 3 可写挂载那一刻读 a 那一槽报读错，之后照读；覆盖写到 a + 3S − 1 = 28（S = 8 的 24 槽环）。
 /// 预期：记 txg 28 那条根之前 a 那一槽成了根环表里最小的那个（txg 0–4 已被盖掉），重读读出 txg 5 的根，按写行那一张实例表（实例 1 停在 4）被抛弃：当没有根，
diff --git a/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs b/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs
index 55dbbd8..551dc0f 100644
--- a/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs
+++ b/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs
@@ -19,6 +19,11 @@
 //!
 //! 照能住的最大 txg 24 记（第 10 批 core 簇那一版）：txg 25–47 那几版上进程内按 24 回收，门槛越过那一槽里真住着的 txg 0，txg 0 引用的单元
 //! 被回收复用，而 txg 0 是环里读得出的有效根，池级 checker 照当候选走、读到复用的槽（txg 27 起 I-2.1、I-4.8、I-3.1 红）。
+//!
+//! 同一份盘面上那一槽从挂载起一直读回全零（报成功、字节全 0，盘上那一槽里其实是 txg 0 的根）：读回全零的槽在回收门槛里照「自证不过」那一类
+//! 钉在它可能住过的最旧 txg，只是不并进准入抬 F 的上限；进程内重读读回全零不算确认为空，照旧钉着（用户 2026-10-05 定乙，条文还没写回，
+//! 依据 `records/2026-09-30-里程碑三开工第一轮.md`「第 10b 批 core 修订交回」那一条的②）。照「从没写过」不钉时（第 10b 批那一版），
+//! 挂载那一刻门槛就越过 txg 0，txg 0 引用的单元当场回收。
 mod common;
 
 use common::{
@@ -34,7 +39,7 @@ use singlefs_core::allocator::{PoolAllocator, TxgsARingSlotBadAtMountCanHold};
 use singlefs_core::block_device::BlockDevice;
 use singlefs_core::mount::mount_writable;
 use singlefs_core::recovery::{choose_system_configuration, readable_roots_with_ring_slots};
-use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
 use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
 use singlefs_harness::memory_pool::MemoryPool;
 use singlefs_harness::SharedStream;
@@ -98,14 +103,11 @@ fn slots_of_the_mkfs_root_left_free(
         .collect()
 }
 
-/// 场景：上面那份盘面上可写挂载（实例 2），那一槽只在挂载那一次读不出；之后覆盖写到 txg 48（这个进程写掉那一槽）。
-/// 预期：挂载交回的根环表把那一槽记成挂载时读坏、钉在可能住过的最旧 txg 0（能住的最大 txg 24）；挂载之后第一次覆盖写记根之前重读，
-/// 读出 txg 0 的根，那一槽不再按读坏记、算这个进程知道住着根的槽。挂载之后与每一次覆盖写之后：txg 0 那一版引用的单元在分配器上
-/// 一个都不空闲，盘上那几条不变量不红。
-#[test]
-fn a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants(
-) {
-    let mut pool = build_pool("root-ring-slot-unreadable-at-mount-reread");
+/// 上面那份盘面（崩溃状态）与 txg 24 落到的那一槽（区域 0 槽 0，里面留着 mkfs 写的 txg 0 的根）。`tag` 给池的镜像起名。
+fn crash_state_with_the_mkfs_root_left_in_the_slot_of_txg_24(
+    tag: &str,
+) -> (MemoryPool, RootRingSlot) {
+    let mut pool = build_pool(tag);
     let slots_per_region = parameters().geometry.root_ring_slots_per_region;
     assert_eq!(
         3 * slots_per_region.count(),
@@ -156,6 +158,19 @@ fn a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldes
         .write(offset_of_the_slot, &stale_root_slot);
     write_back_system_configuration_slots(&mut crash_image, &system_configuration_before);
 
+    (crash_image, hidden_slot)
+}
+
+/// 场景：上面那份盘面上可写挂载（实例 2），那一槽只在挂载那一次读不出；之后覆盖写到 txg 48（这个进程写掉那一槽）。
+/// 预期：挂载交回的根环表把那一槽记成挂载时读坏、钉在可能住过的最旧 txg 0（能住的最大 txg 24）；挂载之后第一次覆盖写记根之前重读，
+/// 读出 txg 0 的根，那一槽不再按读坏记、算这个进程知道住着根的槽。挂载之后与每一次覆盖写之后：txg 0 那一版引用的单元在分配器上
+/// 一个都不空闲，盘上那几条不变量不红。
+#[test]
+fn a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_and_every_version_keeps_the_invariants(
+) {
+    let (crash_image, hidden_slot) = crash_state_with_the_mkfs_root_left_in_the_slot_of_txg_24(
+        "root-ring-slot-unreadable-at-mount-reread",
+    );
     let stream = SharedStream::retaining_contents();
     let unreadable_at_mount = SharedUnreadableRanges::new(
         vec![unreadable_root_slot_of(
@@ -262,3 +277,126 @@ fn a_root_ring_slot_unreadable_only_at_mount_pins_the_reclaim_floor_at_the_oldes
         );
     }
 }
+
+/// 场景：同一份盘面上可写挂载（实例 2），那一槽从挂载起每次读都报成功、字节全 0（盘上那一槽里是 txg 0 的根）；之后覆盖写到 txg 48
+/// （这个进程写掉那一槽）。
+/// 预期：挂载交回的根环表把那一槽记成挂载时读回全零、钉在它可能住过的最旧 txg 0，不进准入抬 F 的上限要并的那几个槽；挂载之后每次记根
+/// 重读那一槽都读回全零，不算确认为空，照旧钉着、不算这个进程知道住着根的槽。挂载之后与每一次覆盖写之后：txg 48 写掉那一槽之前，txg 0
+/// 那一版引用的单元在分配器上一个都不空闲，盘上那几条不变量不红（池级 checker 读盘上完好的 txg 0，环里最旧有效根是 0，与钉着的 0 对得上）。
+#[test]
+fn a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it(
+) {
+    let (crash_image, hidden_slot) = crash_state_with_the_mkfs_root_left_in_the_slot_of_txg_24(
+        "root-ring-slot-reading-back-all-zero-since-the-mount",
+    );
+    let stream = SharedStream::retaining_contents();
+    let reading_back_all_zero = SharedUnreadableRanges::new(
+        vec![unreadable_root_slot_of(
+            CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
+            FailingReadsOfARange::Every,
+        )],
+        UnreadableRangeReadBack::Zeros,
+    );
+    let mut devices = with_unreadable_ranges(
+        crash_state_devices(&crash_image, &[], &[], &stream),
+        &reading_back_all_zero,
+    );
+    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
+    assert!(
+        reading_back_all_zero.failed_reads() > 0,
+        "挂载那一次读那一槽读回了全零"
+    );
+    assert_eq!(
+        (
+            mounted.output.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        (InstanceGeneration(2), CheckpointTxg(24)),
+        "恢复由记录施加出 txg 24"
+    );
+    let image_now = || {
+        let mut image = crash_image.clone();
+        image.apply(&stream.retained_operations());
+        image
+    };
+    // 那一槽在根环表里怎么记着：(读回全零钉着的 txg, 是否进准入抬 F 的上限要并的槽, 是否算这个进程知道住着根的槽)。
+    let how_the_table_records_the_slot = |allocator: &PoolAllocator| {
+        let occupancy = allocator.root_ring_occupancy().expect("可写挂载装了根环表");
+        (
+            occupancy
+                .ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held()
+                .get(&hidden_slot)
+                .copied(),
+            occupancy.ring_slots_bad_at_mount().contains_key(&hidden_slot),
+            occupancy
+                .ring_slots_known_to_hold_a_root()
+                .contains(&hidden_slot),
+        )
+    };
+    assert_eq!(
+        how_the_table_records_the_slot(&mounted.allocator),
+        (Some(CheckpointTxg(0)), false, false),
+        "根环表把那一槽记成挂载时读回全零：钉在可能住过的最旧 txg 0，不进上限要并的槽"
+    );
+    let after_the_mount = image_now();
+    assert_eq!(
+        (
+            slots_of_the_mkfs_root_left_free(&after_the_mount, &mounted.allocator),
+            violations_of(&after_the_mount)
+        ),
+        (Vec::new(), Vec::<String>::new()),
+        "挂载之后：txg 0 引用的单元一个都不空闲，那几条不变量不红"
+    );
+    let PoolVersion::WithFile(mut current) = mounted.current else {
+        panic!("txg 24 那一版带文件");
+    };
+    let publish_parameters = parameters();
+    // 迭代上界：覆盖写到 txg 48（那一槽被这个进程写掉的那一版）；跨轮携带的是现行那一版。
+    while current.root.checkpoint_txg.0 < TXG_THAT_OVERWRITES_THE_SLOT {
+        let checkpoint_txg = current.root.checkpoint_txg.0 + 1;
+        let reads_of_the_slot_before = reading_back_all_zero.reads_of_range(0);
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
+        let image = image_now();
+        if checkpoint_txg < TXG_THAT_OVERWRITES_THE_SLOT {
+            assert_eq!(
+                (
+                    reading_back_all_zero.reads_of_range(0) > reads_of_the_slot_before,
+                    how_the_table_records_the_slot(&mounted.allocator)
+                ),
+                (true, (Some(CheckpointTxg(0)), false, false)),
+                "txg {checkpoint_txg}：记根之前重读那一槽、读回全零，照旧钉在 0"
+            );
+            assert_eq!(
+                (
+                    slots_of_the_mkfs_root_left_free(&image, &mounted.allocator),
+                    violations_of(&image)
+                ),
+                (Vec::new(), Vec::<String>::new()),
+                "txg {checkpoint_txg} 之后：txg 0 引用的单元一个都不空闲，那几条不变量不红"
+            );
+        } else {
+            assert_eq!(
+                (
+                    how_the_table_records_the_slot(&mounted.allocator),
+                    violations_of(&image)
+                ),
+                ((None, false, true), Vec::<String>::new()),
+                "txg {checkpoint_txg}：这个进程写掉那一槽，不再钉着、算知道住着根的槽；那几条不变量不红"
+            );
+        }
+    }
+}
````

### mutations-append.tsv

````
第10c批 挂载时的回收门槛不钉读回全零的槽（洞里 txg 7 的根所在那一槽再挂起读回全零：门槛取读得出的最旧有效根 14，txg 7 引用的单元当场回收）	crates/singlefs-core/src/mount.rs	                .map(|(_, oldest_txg_it_can_have_held)| *oldest_txg_it_can_have_held),	                .filter(|_| false)\n                .map(|(_, oldest_txg_it_can_have_held)| *oldest_txg_it_can_have_held),	-p singlefs-harness --test one_revolution_window_after_a_hole_in_the_root_ring -- raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated	raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated
第10c批 进程内回收门槛不钉读回全零的槽（mkfs 的 txg 0 留在那一槽里、读回全零：挂载之后第一次记根就越过 txg 0）	crates/singlefs-core/src/allocator.rs	                RootRingOccupant::AllZeroAtMount {\n                    oldest_txg_it_can_have_held,\n                } => Some(*oldest_txg_it_can_have_held),	                RootRingOccupant::AllZeroAtMount {\n                    oldest_txg_it_can_have_held: _,\n                } => None,	-p singlefs-harness --test root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window -- a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it	a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it
第10c批 读回全零的槽挂载时不记进根环表（只在挂载那一刻钉着，挂载之后第一次记根就越过 txg 0）	crates/singlefs-core/src/mount.rs	        .chain(ring_slots_reading_back_all_zero_at_mount.iter().map(	        .chain(ring_slots_reading_back_all_zero_at_mount.iter().filter(|_| false).map(	-p singlefs-harness --test root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window -- a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it	a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it
第10c批 进程内记根时重读读回全零就把那一槽拿出根环表（挂载之后第一次记根就越过 txg 0）	crates/singlefs-core/src/allocator.rs	                    RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {\n                        occupancy.occupants.remove(&ring_slot);\n                    }\n                    RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified\n                    | RereadOfARootRingSlotBadAtMount::AllZero => {}	                    RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable\n                    | RereadOfARootRingSlotBadAtMount::AllZero => {\n                        occupancy.occupants.remove(&ring_slot);\n                    }\n                    RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified => {}	-p singlefs-harness --test root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window -- a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it	a_root_ring_slot_reading_back_all_zero_since_the_mount_pins_the_reclaim_floor_at_the_oldest_txg_it_can_have_held_until_the_process_overwrites_it
第10c批 进程内重读读回全零当确认为空、拿掉（前洞那一槽 txg 20 重读读回全零，不再钉着）	crates/singlefs-core/src/mount.rs	        RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => {\n            RereadOfARootRingSlotBadAtMount::AllZero\n        }	        RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => {\n            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable\n        }	-p singlefs-harness --test root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread -- a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it	a_front_hole_unreadable_at_mount_reread_as_all_zero_stays_pinned_until_the_process_overwrites_it
第10c批 E2 单测：重读读回全零拿掉	crates/singlefs-core/src/allocator.rs	            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {\n                bad_at_mount.remove(&ring_slot);\n            }\n            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified\n            | RereadOfARootRingSlotBadAtMount::AllZero => {	            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable\n            | RereadOfARootRingSlotBadAtMount::AllZero => {\n                bad_at_mount.remove(&ring_slot);\n            }\n            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified => {	-p singlefs-core --lib -- allocator::tests::the_slot_bad_at_mount_reading_back_all_zero_on_the_reread_stays_pinned_at_the_oldest_txg_it_can_have_held	the_slot_bad_at_mount_reading_back_all_zero_on_the_reread_stays_pinned_at_the_oldest_txg_it_can_have_held
第10c批 抬 F 那一遍的回收门槛不并读回全零的槽（洞里 txg 7 的根所在那一槽一直读回全零，抬 F 到 7 之后门槛 14，txg 7 引用的单元放开）	crates/singlefs-core/src/mount.rs	            RootRingOccupancy::ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held,	            RootRingOccupancy::ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held,	-p singlefs-harness --test one_revolution_window_after_a_hole_in_the_root_ring -- raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated	raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated
第10c批 回收门槛与 E2 重读看的槽漏了读回全零的那一类（洞里 txg 7 的根所在那一槽一直读回全零，抬 F 那一遍不重读、门槛 14，txg 7 引用的单元放开）	crates/singlefs-core/src/allocator.rs	        pinned.extend(\n            self.ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held(),\n        );	        pinned.extend(BTreeMap::<RootRingSlot, CheckpointTxg>::new());	-p singlefs-harness --test one_revolution_window_after_a_hole_in_the_root_ring -- raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated	raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated
第10c批 进程内回收门槛不钉读回全零的槽（没洞的那一份 X = 17 那一槽读回全零：空发布把门槛推到 18，抬 F 之前 X 那一版引用的单元回收）	crates/singlefs-core/src/allocator.rs	                RootRingOccupant::AllZeroAtMount {\n                    oldest_txg_it_can_have_held,\n                } => Some(*oldest_txg_it_can_have_held),	                RootRingOccupant::AllZeroAtMount {\n                    oldest_txg_it_can_have_held: _,\n                } => None,	-p singlefs-harness --test raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount -- admission_raising_the_floor_without_holes_passes_a_slot_reading_back_zeros_since_the_mount_as_the_known_cost	admission_raising_the_floor_without_holes_passes_a_slot_reading_back_zeros_since_the_mount_as_the_known_cost
````

### mutations-replacements.tsv

````
第9批 RS1：重读读出有效根的那一槽不算这个进程知道住着根的槽（之后那一槽读坏，准入抬 F 按没有根往下走、发了写）	crates/singlefs-core/src/allocator.rs	                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => true,\n                RootRingOccupant::BadAtMount(_) | RootRingOccupant::AllZeroAtMount { .. } => false,	                => true,\n                RootRingOccupant::BadAtMount(_)\n                | RootRingOccupant::AllZeroAtMount { .. }\n                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => false,	-p singlefs-harness --test root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread -- a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write	a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write
````

### mutations-delete.txt

````
第10b批 进程内重读读回全零照旧钉着（前洞那一槽 txg 20 重读读回全零，不拿掉）
第10b批 E2 单测：重读读回全零照旧钉着
````

## batch10d-record-seam

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10d-record-seam/patch/`
- `crates.patch`：sha256 2306d7c25db53bed01fd17a5f4cdfbaceef677cd6604b2a8e4e0d4b52000ab79
- `mutations-append.tsv`：sha256 794eececfa93f0016744ef8fb330244e4b42d7f78330ddddbf4ac361aeb0fd86
- `report.md`：sha256 bf231b352e894e3e27fe0a43a2abd752e6b6658fc779f4f76176c1e631dc4c22

### crates.patch

````
diff --git a/crates/singlefs-harness/src/memory_pool.rs b/crates/singlefs-harness/src/memory_pool.rs
index f088d64..641e0cb 100644
--- a/crates/singlefs-harness/src/memory_pool.rs
+++ b/crates/singlefs-harness/src/memory_pool.rs
@@ -933,58 +933,121 @@ pub struct PublishWrites {
     pub checkpoint_txg: u64,
 }
 
-/// 交给记录核对器的写表是怎么来的：一条录制流从头到尾，还是一条被崩溃截断的录制流后面接上崩溃之后在那份崩溃后镜像上写出的流
-/// （[`publishes_in`] 按它在接缝处断开）。
+/// 写表里的一道接缝：从这一项起是另一个进程写出的录制流，之前那一截到这里就断了（[`publishes_in`] 在这里断开，两截各分各的）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub enum RecordStreamContinuity {
-    /// 一条录制流从头到尾（层 0 与崩溃注入第一截）。
-    OneRecording,
-    /// 写表前 `first_write_after_the_crash` 项是被一次崩溃截断的那条录制流（整条录下来的，截断处之后的写都记成没持久），
-    /// 从这一项起是崩溃之后在那份崩溃后镜像上写出的流（崩溃注入第二、三截：可写挂载那一段）。
-    ResumedAfterACrash {
+pub enum RecordStreamSeam {
+    /// 一段历史里的一次掉电（`crate::history` 里覆盖写掉电在第一个 journal 记录写之前、再可写挂载的那一步）：之前那一截是掉电之前录下的
+    /// （录制器只录交下去、落了盘的写；掉电那个写与之后的写一个都没交下去），从 `first_write_after_the_power_loss` 起是掉电之后
+    /// 那次可写挂载另起的实例写的流。交这道接缝的调用方保证之前那一截在持久集合里全落了（被判的崩溃状态落在掉电之后）。
+    AfterAPowerLoss {
+        first_write_after_the_power_loss: usize,
+    },
+    /// 之前那一截是被一次崩溃截断的录制流（整条录下来的，截断处之后的写都记成没持久），从 `first_write_after_the_crash` 起是
+    /// 崩溃之后在那份崩溃后镜像上写出的流（崩溃注入第二、三截：可写挂载那一段）。
+    AfterACrash {
         first_write_after_the_crash: usize,
         /// 那次崩溃之后只读恢复落到的那一版（恢复失败是 None）：截断处在飞的那次发布只在恢复落到它上（由它的记录重建）时算发生过。
         version_landed_on_after_the_crash: Option<(InstanceGeneration, CheckpointTxg)>,
     },
 }
 
+impl RecordStreamSeam {
+    /// 接缝之后第一个写在写表里的下标。
+    #[must_use]
+    pub fn first_write_after_the_seam(&self) -> usize {
+        match self {
+            Self::AfterAPowerLoss {
+                first_write_after_the_power_loss,
+            } => *first_write_after_the_power_loss,
+            Self::AfterACrash {
+                first_write_after_the_crash,
+                ..
+            } => *first_write_after_the_crash,
+        }
+    }
+}
+
+/// 交给记录核对器的写表是怎么来的：一条录制流从头到尾，还是在几道接缝处断开的几截录制流（[`publishes_in`] 按它分发布）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RecordStreamContinuity<'seams> {
+    /// 一条录制流从头到尾（层 0；崩溃注入第一截在崩溃之前没掉过电的历史上）。
+    OneRecording,
+    /// 写表在每道接缝处断开，接缝按写表下标从小到大；最后一道接缝之后那一截是一条录制流。崩溃注入第一截给崩溃之前那几道掉电接缝；
+    /// 第二、三截给那几道掉电接缝，再加历史与挂载之间那一道崩溃接缝。
+    BrokenAtSeams(&'seams [RecordStreamSeam]),
+}
+
 /// 写表里的每次发布。一条录制流里按根槽写分：一次根槽写之前、上一次根槽写之后的单元写与 journal 记录写都归它。
 ///
-/// 写表是崩溃截断的流接上崩溃之后写出的流（[`RecordStreamContinuity::ResumedAfterACrash`]）时，在接缝处断开，两条流各分各的：
-/// - 截断的那条流末尾没等到根槽写的单元写与记录写，不归接缝之后的第一次发布（那次发布是崩溃之后另起的实例写的）；
-/// - 截断的那条流里只留崩溃之后那条时间线上的发布：根槽写落了盘的（`persisted`），与那次崩溃之后恢复落到的那一版
-///   （由记录重建时它的根槽写在截断处之后、没落盘）。截断处之后才写根槽写、恢复又没落到的发布没发生过：崩溃之后的挂载另起实例、
-///   txg 从盘上最大的往上接，与它们的 (实例, txg) 撞得上，挂载写的实例表也不带挂载自己实例的行，留着就会被当成恢复自称的那一版该有的发布。
+/// 写表在接缝处断开（[`RecordStreamContinuity::BrokenAtSeams`]）时，每道接缝前后各分各的：
+/// - 接缝之前那一截末尾没等到根槽写的单元写与记录写，不归接缝之后的第一次发布（那次发布是另起的实例写的）。掉电那道接缝
+///   （[`RecordStreamSeam::AfterAPowerLoss`]）之前那一截就是这样：掉电截在第一个 journal 记录写上，那次改动的单元写落了盘、
+///   根与记录一个都没写，接成一条来分会把它们归进掉电之后可写挂载写行那次发布，那次发布之后复用了它们落点的写就被判成盖掉了一次发布的单元；
+/// - 崩溃那道接缝（[`RecordStreamSeam::AfterACrash`]）之前那一截里只留崩溃之后那条时间线上的发布：根槽写落了盘的（`persisted`），
+///   与那次崩溃之后恢复落到的那一版（由记录重建时它的根槽写在截断处之后、没落盘）。截断处之后才写根槽写、恢复又没落到的发布没发生过：
+///   崩溃之后的挂载另起实例、txg 从盘上最大的往上接，与它们的 (实例, txg) 撞得上，挂载写的实例表也不带挂载自己实例的行，
+///   留着就会被当成恢复自称的那一版该有的发布。掉电那道接缝之前那一截的根槽写都落了盘（调用方保证），不必剔。
 ///
 /// 两条流接成一条来分（B3b 报告第二节第 4 条）会假红：第一次崩溃截在一次发布中间时，只接到截断处为止，那次发布没落根的单元写与记录写
 /// 归进挂载写行那次发布；整条接上，截断处之后的历史发布与挂载的发布撞了身份。
+///
+/// # Panics
+/// 接缝不是按写表下标从小到大、或越过写表；掉电那道接缝之前那一截有一个写在持久集合里没落。
 pub fn publishes_in(
     writes: &[RetainedWrite],
     persisted: &[bool],
-    continuity: RecordStreamContinuity,
+    continuity: RecordStreamContinuity<'_>,
 ) -> Vec<PublishWrites> {
     match continuity {
         RecordStreamContinuity::OneRecording => publishes_of_one_recording(writes, 0),
-        RecordStreamContinuity::ResumedAfterACrash {
-            first_write_after_the_crash,
-            version_landed_on_after_the_crash,
-        } => {
-            let (cut_recording, resumed_recording) = writes.split_at(first_write_after_the_crash);
-            publishes_of_one_recording(cut_recording, 0)
-                .into_iter()
-                .filter(|publish| {
-                    persisted[publish.root]
-                        || version_landed_on_after_the_crash
-                            == Some((
-                                InstanceGeneration(publish.instance),
-                                CheckpointTxg(publish.checkpoint_txg),
-                            ))
-                })
-                .chain(publishes_of_one_recording(
-                    resumed_recording,
-                    first_write_after_the_crash,
-                ))
-                .collect()
+        RecordStreamContinuity::BrokenAtSeams(seams) => {
+            let mut publishes = Vec::new();
+            let mut first_write_of_the_recording = 0;
+            for seam in seams {
+                let first_write_after_the_seam = seam.first_write_after_the_seam();
+                assert!(
+                    first_write_of_the_recording <= first_write_after_the_seam
+                        && first_write_after_the_seam <= writes.len(),
+                    "接缝按写表下标从小到大、不越过写表：上一截起在 {first_write_of_the_recording}，这道接缝在 {first_write_after_the_seam}，写表 {} 项",
+                    writes.len()
+                );
+                let publishes_of_the_recording_before_the_seam = publishes_of_one_recording(
+                    &writes[first_write_of_the_recording..first_write_after_the_seam],
+                    first_write_of_the_recording,
+                );
+                match seam {
+                    RecordStreamSeam::AfterAPowerLoss { .. } => {
+                        assert!(
+                            persisted[first_write_of_the_recording..first_write_after_the_seam]
+                                .iter()
+                                .all(|is_persisted| *is_persisted),
+                            "掉电那道接缝（写表下标 {first_write_after_the_seam}）之前那一截在持久集合里全落了：被判的崩溃状态落在掉电之后"
+                        );
+                        publishes.extend(publishes_of_the_recording_before_the_seam);
+                    }
+                    RecordStreamSeam::AfterACrash {
+                        version_landed_on_after_the_crash,
+                        ..
+                    } => publishes.extend(
+                        publishes_of_the_recording_before_the_seam
+                            .into_iter()
+                            .filter(|publish| {
+                                persisted[publish.root]
+                                    || *version_landed_on_after_the_crash
+                                        == Some((
+                                            InstanceGeneration(publish.instance),
+                                            CheckpointTxg(publish.checkpoint_txg),
+                                        ))
+                            }),
+                    ),
+                }
+                first_write_of_the_recording = first_write_after_the_seam;
+            }
+            publishes.extend(publishes_of_one_recording(
+                &writes[first_write_of_the_recording..],
+                first_write_of_the_recording,
+            ));
+            publishes
         }
     }
 }
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index 2b7616b..efce7be 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -1428,6 +1428,8 @@ struct HistoryPool {
     stream: SharedStream,
     /// mkfs 写完时录制流里有几步（崩溃注入的基线从这里起）。
     operations_written_by_make_filesystem: usize,
+    /// 每次掉电之后录制流里第一步的下标，按次序（交回 [`HistoryRun::first_stream_index_after_each_power_loss`]）。
+    first_stream_index_after_each_power_loss: Vec<usize>,
 }
 
 /// 模型对一步的判定：对不上的那一格（没有就是 None）与这一步比了多少格。
@@ -1670,6 +1672,7 @@ impl HistoryPool {
                 model,
                 stream: stream.clone(),
                 operations_written_by_make_filesystem,
+                first_stream_index_after_each_power_loss: Vec::new(),
             },
             verdict,
         ))
@@ -2771,6 +2774,11 @@ pub struct HistoryRun {
     /// 观察者那一路看不到这一份：判出失败的那一步不调观察者，而失败常常出在模型已经认下那一版之后（checker 在那一步判红）。
     /// 内容拷成 `Vec<u8>` 不留 `Rc`：`HistoryRun` 要跨线程送（`run_history_campaign` 的那张表），`Rc` 不是 `Send`。
     pub committed_versions_at_the_end: Vec<(ModelRootKey, Option<Vec<u8>>)>,
+    /// 每次掉电（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]）之后录制流里第一步的下标，按次序：
+    /// 掉电之前那一截与之后可写挂载另起的实例写的流在这里断开。崩溃注入把它交给记录核对器
+    /// （`crate::memory_pool::RecordStreamSeam::AfterAPowerLoss`）：那次改动掉电之前落了盘的单元写没有根，
+    /// 不归掉电之后的第一次发布。
+    pub first_stream_index_after_each_power_loss: Vec<usize>,
 }
 
 /// 每一步之后交给观察者的东西：第几步、这一步是什么、结局、此刻的整份镜像、跟到这一步的理想模型。
@@ -4758,6 +4766,10 @@ fn apply_overwrite_losing_power_before_its_journal_record_then_mount_writable(
         .into_iter()
         .map(|(identity, device)| (identity, device.inner))
         .collect();
+    // 这个进程到这里就没了（掉电截在第一个 journal 记录写上；那次改动在那之前就被拒的，进程同样不再往下写），
+    // 之后录下的是可写挂载另起的实例写的流。
+    pool.first_stream_index_after_each_power_loss
+        .push(pool.stream.operation_count());
     pool.model.close_session();
     apply_mount_writable(pool, parameters)
 }
@@ -5990,6 +6002,11 @@ pub fn execute_history_with_faults(
                 .map(|(key, content)| (key, content.map(|content| content.to_vec())))
                 .collect()
         }),
+        first_stream_index_after_each_power_loss: pool_slot
+            .as_ref()
+            .map_or_else(Vec::new, |pool| {
+                pool.first_stream_index_after_each_power_loss.clone()
+            }),
     }
 }
 
@@ -6850,6 +6867,7 @@ mod tests {
             tally: HistoryTally::default(),
             operations_written_by_make_filesystem: 0,
             committed_versions_at_the_end: Vec::new(),
+            first_stream_index_after_each_power_loss: Vec::new(),
         };
         let finished = vec![
             (
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 1cef15b..20ecb76 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -82,7 +82,7 @@ use singlefs_harness::history::{
 use singlefs_harness::memory_pool::{
     newest_persisted_root, root_identity_written_by, some_publish_persisted_without_its_root,
     writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, RecordCheck,
-    RecordStreamContinuity, RetainedWrite, SparseBlockDevice, SECTOR_BYTES,
+    RecordStreamContinuity, RecordStreamSeam, RetainedWrite, SparseBlockDevice, SECTOR_BYTES,
 };
 use singlefs_harness::model::{
     crash_recovery_disagreement, ModelDisagreement, ModelDisagreementAspect, ModelRefusalReason,
@@ -1053,6 +1053,35 @@ impl StreamMarks {
     }
 }
 
+/// 每次掉电之后第一个写在写表里的下标，按次序：录制流下标（[`singlefs_harness::history::HistoryRun::first_stream_index_after_each_power_loss`]）
+/// 换成写表下标，取录制流下标不小于它的第一个写（写表不算屏障，`stream_indexes` 与写表逐条对应）；掉电之后一个写都没录下的取写表长度。
+fn first_write_after_each_power_loss_in_the_write_table(
+    first_stream_index_after_each_power_loss: &[usize],
+    stream_indexes: &[usize],
+) -> Vec<usize> {
+    first_stream_index_after_each_power_loss
+        .iter()
+        .map(|first_stream_index_after_the_power_loss| {
+            stream_indexes
+                .iter()
+                .position(|stream_index| *stream_index >= *first_stream_index_after_the_power_loss)
+                .unwrap_or(stream_indexes.len())
+        })
+        .collect()
+}
+
+/// 写表下标上的几道掉电接缝（[`RecordStreamSeam::AfterAPowerLoss`]），次序照给的。
+fn power_loss_seams(first_write_after_each_power_loss: &[usize]) -> Vec<RecordStreamSeam> {
+    first_write_after_each_power_loss
+        .iter()
+        .map(
+            |first_write_after_the_power_loss| RecordStreamSeam::AfterAPowerLoss {
+                first_write_after_the_power_loss: *first_write_after_the_power_loss,
+            },
+        )
+        .collect()
+}
+
 /// 崩溃窗口末尾那个写在录制流里的下标：窗口从写表第 `first_write_of_the_segment` 个写起数 `writes_from_the_segment_on` 个
 /// （靶向用例的窗口可以越过段尾），末尾是写表第 `first_write_of_the_segment + writes_from_the_segment_on - 1` 个写。
 /// 读故障的认定（[`StreamMarks::read_faults_in_a_crash_state`]）按录制流下标与每一步跑完时流里的步数比；写表下标不算屏障，
@@ -1327,6 +1356,10 @@ fn inject_crashes_into_history_with_every_switch(
     };
     let (writes, segments, stream_indexes) =
         writes_and_segments_with_stream_indexes(&operations, &geometry);
+    let first_write_after_each_power_loss = first_write_after_each_power_loss_in_the_write_table(
+        &run.first_stream_index_after_each_power_loss,
+        &stream_indexes,
+    );
     // 崩溃状态摆在最后一个跑完的操作之前，停下那一步不进 RefFS（与 O1 那一路同一个范围）。
     let reference_filesystem_history =
         reference_filesystem_translation.translate(&operations, &geometry);
@@ -1456,12 +1489,27 @@ fn inject_crashes_into_history_with_every_switch(
         // 更早的段里发出的单元写与 journal 记录写要算进一次发布里；恢复落到由 journal 记录重建的一版时，那一版的根槽写在更晚的段里，
         // 只给到这一段为止的前缀就找不到它、读不出它的实例表，被它抛弃的发布的豁免整条落空（D23（journal 的角色与格式） 已定项 15）。
         // 持久集合与整条流逐条对应：更早的段整段持久、当前段按这个崩溃点的子集、更晚的段一个都没持久（D13（验证路线） 已定项 7 的第四样入参）。
+        // 这个崩溃状态之前掉过电的，在每道掉电接缝处断开（[`RecordStreamSeam::AfterAPowerLoss`]）。
+        let first_write_after_each_power_loss_before_the_crash: Vec<usize> =
+            first_write_after_each_power_loss
+                .iter()
+                .copied()
+                .filter(|first_write_after_the_power_loss| {
+                    *first_write_after_the_power_loss <= first_write_of_the_segment
+                })
+                .collect();
+        let power_loss_seams_before_the_crash =
+            power_loss_seams(&first_write_after_each_power_loss_before_the_crash);
         let record_check = check_records_against(
             &image,
             &image,
             &writes,
             &persisted,
-            RecordStreamContinuity::OneRecording,
+            if power_loss_seams_before_the_crash.is_empty() {
+                RecordStreamContinuity::OneRecording
+            } else {
+                RecordStreamContinuity::BrokenAtSeams(&power_loss_seams_before_the_crash)
+            },
             report.effective_root,
         );
         tally.record_checks += 1;
@@ -1541,6 +1589,7 @@ fn inject_crashes_into_history_with_every_switch(
         let records_across_the_crash = HistoryThenWritableMountRecords::new(
             &writes,
             &persisted,
+            &first_write_after_each_power_loss_before_the_crash,
             report.effective_root,
             &mount_writes,
             &writes_of_the_publish_after_the_mount,
@@ -2324,7 +2373,8 @@ fn publishes_before_the_continuation_of_a_mount_that_handed_back_an_error(
 }
 
 /// 第二、三截交给记录核对器的记录流：整条历史录制流接上那份崩溃后镜像上可写挂载那一段录制流、再接上挂载之后那一次发布的录制流，
-/// 在历史与挂载的接缝处断开（[`RecordStreamContinuity::ResumedAfterACrash`]：历史那一段只留第一次崩溃之后那条时间线上的发布）。
+/// 在历史与挂载的接缝处断开（[`RecordStreamSeam::AfterACrash`]：历史那一段只留第一次崩溃之后那条时间线上的发布）；
+/// 历史在第一次崩溃之前掉过电的，也在每道掉电接缝处断开（[`RecordStreamSeam::AfterAPowerLoss`]）。
 /// 历史那一段的持久集合是第一次崩溃那个状态的，与第一截交的同一张。只交挂载那一段时，历史那几次发布在挂载之后的池与二次崩溃镜像上
 /// 还在不在没人核，二次崩溃落到由记录重建的写行那一版时它的根槽写也不在表里（B3c-1 报告第六节第 2 条、B3b 报告第二节第 4、5 条）。
 /// 挂载之后那一次发布的写也要在表里：它会复用历史发布换下、已可回收的槽，第二截的池上那些单元被它合法盖掉，表里没有它就解释不了。
@@ -2337,15 +2387,21 @@ pub struct HistoryThenWritableMountRecords {
     writes_of_the_mount: usize,
     /// 第一次崩溃之后只读恢复落到的那一版（恢复失败是 None）。
     version_landed_on_after_the_first_crash: Option<(InstanceGeneration, CheckpointTxg)>,
+    /// 交给记录核对器的接缝：第一次崩溃之前的几道掉电接缝，再加历史与挂载之间那一道崩溃接缝。
+    seams: Vec<RecordStreamSeam>,
 }
 
 impl HistoryThenWritableMountRecords {
+    /// `first_write_after_each_power_loss_before_the_crash` 是第一次崩溃之前每次掉电之后第一个写在历史写表里的下标，从小到大
+    /// （历史在那次崩溃之前没掉过电的给空表）。
+    ///
     /// # Panics
-    /// `persisted_in_the_history` 与 `history_writes` 不一样长。
+    /// `persisted_in_the_history` 与 `history_writes` 不一样长；有一道掉电接缝越过历史写表。
     #[must_use]
     pub fn new(
         history_writes: &[RetainedWrite],
         persisted_in_the_history: &[bool],
+        first_write_after_each_power_loss_before_the_crash: &[usize],
         version_landed_on_after_the_first_crash: Option<(InstanceGeneration, CheckpointTxg)>,
         mount_writes: &[RetainedWrite],
         writes_of_the_publish_after_the_mount: &[RetainedWrite],
@@ -2355,6 +2411,20 @@ impl HistoryThenWritableMountRecords {
             history_writes.len(),
             "历史那一段的持久集合与历史录制流逐条对应"
         );
+        assert!(
+            first_write_after_each_power_loss_before_the_crash
+                .iter()
+                .all(|first_write_after_the_power_loss| {
+                    *first_write_after_the_power_loss <= history_writes.len()
+                }),
+            "掉电接缝落在历史写表里：{first_write_after_each_power_loss_before_the_crash:?}，历史写表 {} 项",
+            history_writes.len()
+        );
+        let mut seams = power_loss_seams(first_write_after_each_power_loss_before_the_crash);
+        seams.push(RecordStreamSeam::AfterACrash {
+            first_write_after_the_crash: history_writes.len(),
+            version_landed_on_after_the_crash: version_landed_on_after_the_first_crash,
+        });
         Self {
             writes: history_writes
                 .iter()
@@ -2365,6 +2435,7 @@ impl HistoryThenWritableMountRecords {
             persisted_in_the_history: persisted_in_the_history.to_vec(),
             writes_of_the_mount: mount_writes.len(),
             version_landed_on_after_the_first_crash,
+            seams,
         }
     }
 
@@ -2375,11 +2446,8 @@ impl HistoryThenWritableMountRecords {
         &self.writes[first_write_of_the_mount..first_write_of_the_mount + self.writes_of_the_mount]
     }
 
-    fn continuity(&self) -> RecordStreamContinuity {
-        RecordStreamContinuity::ResumedAfterACrash {
-            first_write_after_the_crash: self.persisted_in_the_history.len(),
-            version_landed_on_after_the_crash: self.version_landed_on_after_the_first_crash,
-        }
+    fn continuity(&self) -> RecordStreamContinuity<'_> {
+        RecordStreamContinuity::BrokenAtSeams(&self.seams)
     }
 
     /// 历史那一段的持久集合，接上挂载那一段的（`persisted_in_the_mount`，与挂载那一段逐条对应），
diff --git a/crates/singlefs-checker-tier/src/crash.rs b/crates/singlefs-checker-tier/src/crash.rs
index dc53ef8..dc35154 100644
--- a/crates/singlefs-checker-tier/src/crash.rs
+++ b/crates/singlefs-checker-tier/src/crash.rs
@@ -63,7 +63,7 @@ pub fn check_records(
 }
 
 /// 同一条判据，两份崩溃后镜像、被核的记录流与它的持久集合分开给（`persisted` 与 `writes` 逐条对应；`continuity` 说这张写表是
-/// 一条录制流，还是崩溃截断的流接上崩溃之后写出的流，[`publishes_in`] 按它分发布）。
+/// 一条录制流，还是在掉电、崩溃那几道接缝处断开的几截录制流，[`publishes_in`] 按它分发布）。
 ///
 /// 崩溃后镜像是两份（D13（验证路线） 已定项 7）：`crash_state_image` 是崩溃态镜像，判择根与前缀——第一条判据里根槽写与记录写
 /// 「在盘上」看它；`recovered_image` 是实现恢复后的镜像，判在不在——第二条判据里单元副本缺不缺席、恢复落到的那一版的实例表
@@ -74,10 +74,11 @@ pub fn check_records(
 /// - 层 0（[`check_records`]）：两份镜像同一份；写表是枚举用的那一张（录制流里的写在前，原地覆写的撕裂镜像与重放接在后面，
 ///   见 [`CrashImage`]），一条录制流。
 /// - 崩溃注入第一截（增补 3 第 3 件，`crash_injection.rs`）：两份镜像都是「更早的段整段持久 + 当前段一个子集」那份崩溃镜像；
-///   写表是这段历史的整条录制流，一条录制流——`persisted` 里更早的段整段持久、当前段按这个崩溃点的子集、更晚的段一个都没持久；
-///   恢复落到由记录重建的一版时，那一版的根槽写在更晚的段里，也在这张表里。
+///   写表是这段历史的整条录制流——`persisted` 里更早的段整段持久、当前段按这个崩溃点的子集、更晚的段一个都没持久；
+///   恢复落到由记录重建的一版时，那一版的根槽写在更晚的段里，也在这张表里。这个崩溃状态之前掉过电的，在每道掉电接缝处断开
+///   （`RecordStreamSeam::AfterAPowerLoss`：掉电之前那次改动落了盘、没有根的单元写不归掉电之后的第一次发布），没掉过电的是一条录制流。
 /// - 崩溃注入第二截（崩溃后镜像上可写挂载、再发一次布之后的池）：崩溃态镜像是第一截那份崩溃镜像，恢复后的镜像是挂载与那次发布
-///   之后的池；写表是整条历史录制流接上挂载那一段录制流、再接上那次发布的录制流，在历史与挂载的接缝处断开——历史那一段的持久集合
+///   之后的池；写表是整条历史录制流接上挂载那一段录制流、再接上那次发布的录制流，在历史与挂载的接缝处断开（第一截那几道掉电接缝照给）——历史那一段的持久集合
 ///   同第一截，挂载与那次发布全落了；`effective_root` 取挂载与那次发布写出的最新那条根（一条根都没写出时取第一次崩溃之后
 ///   恢复落到的那一版）。
 /// - 崩溃注入第三截（挂载途中的二次崩溃）：两份镜像都是二次崩溃镜像；写表同第二截，挂载那一段的持久集合按二次崩溃——
@@ -113,7 +114,7 @@ pub fn check_records_against<
     recovered_image: &RecoveredImage,
     writes: &[RetainedWrite],
     persisted: &[bool],
-    continuity: RecordStreamContinuity,
+    continuity: RecordStreamContinuity<'_>,
     effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
 ) -> RecordCheck {
     assert_eq!(
@@ -287,7 +288,7 @@ fn rollback_floor_written_by(bytes: &[u8]) -> CheckpointTxg {
 ///   （下界过得了它）、要么比它新而它自己的根还在环里（本来就不许复用）。
 ///
 /// 按这三个界判，合法的复用不会被判成过不了；漏的是释放代比 `earlier_publish_txg + 1` 晚得多、又晚过门槛的那一类。
-/// 崩溃注入第二、三截的写表（崩溃截断的历史流接上崩溃之后写出的流，[`RecordStreamContinuity::ResumedAfterACrash`]）里，
+/// 崩溃注入第二、三截的写表（崩溃截断的历史流接上崩溃之后写出的流，`RecordStreamSeam::AfterACrash`）里，
 /// 截断处之后那一截历史写没发生过，接缝之后的写取「这次写之前」时照样把它们算进去：那一截的根比截断处之前的 txg 都大、
 /// 带的 F 不比之前的小，算进去只会把两个界往「过得了」那边推，不会把合法的复用判成过不了（推的，没造状态量过松了多少）。
 /// 前提有两条，写表的来路都满足：基镜像里的根带的 F 为 0（层 0 的基是 mkfs 之后，崩溃注入的基是空池），
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs b/crates/singlefs-checker-tier/tests/crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs
index 2e0c349..16956f9 100644
--- a/crates/singlefs-checker-tier/tests/crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs
+++ b/crates/singlefs-checker-tier/tests/crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs
@@ -2,7 +2,7 @@
 //! 崩溃注入第二、三截交给记录核对器的记录流（代码审阅第 2 条「崩溃注入层补可写挂载」的收尾）。
 //!
 //! 第二截（崩溃后镜像上可写挂载、再发一次布之后的池）与第三截（挂载途中的二次崩溃）交的都是整条历史录制流接上挂载那一段录制流，
-//! 在接缝处断开（`crash::RecordStreamContinuity::ResumedAfterACrash`，`crash_injection::HistoryThenWritableMountRecords`）；
+//! 在接缝处断开（`memory_pool::RecordStreamSeam::AfterACrash`，`crash_injection::HistoryThenWritableMountRecords`）；
 //! 崩溃后镜像是两份：崩溃态镜像判择根与前缀，实现恢复后的镜像判在不在（D13（验证路线） 已定项 7）。
 //! 此前第三截只交挂载那一段到当前段为止的前缀，历史那几次发布在二次崩溃镜像上还在不在没人核（B3c-1 报告第六节第 2 条）；
 //! 第二截只跑 checker，没接记录核对器（B3b 报告第二节第 5 条）；直接把历史写表接在挂载前面会假红（B3b 报告第二节第 4 条）。
@@ -34,7 +34,7 @@ use singlefs_harness::history::{
 use singlefs_harness::memory_pool::{
     root_identity_written_by, some_publish_persisted_without_its_root, writes_and_segments,
     writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, RecordCheck,
-    RecordStreamContinuity, RetainedWrite, WrittenContents,
+    RecordStreamContinuity, RecordStreamSeam, RetainedWrite, WrittenContents,
 };
 use singlefs_harness::segments::StepKind;
 use singlefs_harness::SharedStream;
@@ -172,10 +172,10 @@ fn publishes_cut_off_by_the_crash_are_on_the_resumed_timeline_only_if_their_root
                 pool,
                 &writes,
                 persisted_set,
-                RecordStreamContinuity::ResumedAfterACrash {
+                RecordStreamContinuity::BrokenAtSeams(&[RecordStreamSeam::AfterACrash {
                     first_write_after_the_crash,
                     version_landed_on_after_the_crash: Some(landed),
-                },
+                }]),
                 Some(root_identity(2, 2)),
             )
         };
@@ -229,10 +229,10 @@ fn writes_left_without_a_root_at_the_end_of_the_cut_recording_do_not_join_the_fi
             &image,
             &writes,
             &persisted,
-            RecordStreamContinuity::ResumedAfterACrash {
+            RecordStreamContinuity::BrokenAtSeams(&[RecordStreamSeam::AfterACrash {
                 first_write_after_the_crash: 3,
                 version_landed_on_after_the_crash: Some(root_identity(1, 1)),
-            },
+            }]),
             Some(root_identity(2, 2)),
         ),
         RecordCheck::default(),
@@ -387,6 +387,8 @@ fn crash_then_mount(
     let records = HistoryThenWritableMountRecords::new(
         &recorded.writes,
         &persisted,
+        // 这几段历史里没有掉电步：第一次崩溃之前一道掉电接缝都没有。
+        &[],
         landed_version,
         &mount_writes,
         &publish_writes,
@@ -943,10 +945,10 @@ fn the_pool_after_the_mount_exempts_what_the_newest_mount_version_abandons_by_it
         &crash_image,
         &writes,
         &persisted,
-        RecordStreamContinuity::ResumedAfterACrash {
+        RecordStreamContinuity::BrokenAtSeams(&[RecordStreamSeam::AfterACrash {
             first_write_after_the_crash: recorded.writes.len(),
             version_landed_on_after_the_crash: state.landed_version,
-        },
+        }]),
         newest_mount_root,
     );
     println!(
diff --git a/crates/singlefs-harness/tests/publishes_of_a_history_broken_at_the_power_loss_seam.rs b/crates/singlefs-harness/tests/publishes_of_a_history_broken_at_the_power_loss_seam.rs
new file mode 100644
index 0000000..f4344aa
--- /dev/null
+++ b/crates/singlefs-harness/tests/publishes_of_a_history_broken_at_the_power_loss_seam.rs
@@ -0,0 +1,210 @@
+//! 掉电那道接缝（`memory_pool::RecordStreamSeam::AfterAPowerLoss`）。留孤儿那段历史
+//! （`history::history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`：第一个文件之后一次覆盖写掉电在它的第一个
+//! journal 记录写之前、进程崩掉再可写挂载成实例 2，之后覆盖写到实例 1 的根全轮出根环）上，执行器记下掉电之后录制流里第一步的下标；
+//! 分发布时在那里断开，实例 1 那次改动落了盘、没有根的单元写不归掉电之后可写挂载写行那次发布。
+//! 接成一条来分时它们归进写行那次发布 (2, 4)，第 1 步覆盖写复用了它们的落点，记录核对器第二条判据就判 (2, 4) 缺了单元；
+//! 记录核对器用这道接缝判的那一半在 checker 档（`crates/singlefs-checker-tier/tests/record_checker_breaks_the_history_at_the_power_loss_seam.rs`）。
+
+use singlefs_core::address::InstanceGeneration;
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_harness::history::{
+    execute_history_with,
+    history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring,
+    HistoryDeviceWidth, HistoryEnding, HistoryExecution, PerStepChecker,
+};
+use singlefs_harness::memory_pool::{
+    publishes_in, root_identity_of_write, writes_and_segments_with_stream_indexes, PublishWrites,
+    RecordStreamContinuity, RecordStreamSeam, RetainedWrite,
+};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::SharedStream;
+
+/// 历史本身不跑每一步的 checker（这段历史每一步都做得成、checker 与模型都不红，由 `history.rs` 里的用例钉），两块 4 GiB 的盘。
+const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
+    per_step_checker: PerStepChecker::Skipped,
+    device_width: HistoryDeviceWidth::FourGibibytes,
+    space_admission: SpaceAdmission::JudgedByTheFormula,
+};
+
+/// 留孤儿那段历史跑完之后的写表，与执行器记下的那几道掉电接缝换成的写表下标。
+struct HistoryLeavingOrphans {
+    writes: Vec<RetainedWrite>,
+    first_write_after_each_power_loss: Vec<usize>,
+}
+
+fn record_the_history_leaving_orphans() -> HistoryLeavingOrphans {
+    let stream = SharedStream::retaining_contents();
+    let run = execute_history_with(
+        &history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring(),
+        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        &stream,
+        &mut |_| {},
+    );
+    assert_eq!(
+        run.ending,
+        HistoryEnding::Completed,
+        "这段历史每一步都做得成"
+    );
+    let (writes, _segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+        &stream.retained_operations(),
+        &UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
+            .device_width
+            .fixed_geometry(),
+    );
+    let first_write_after_each_power_loss = run
+        .first_stream_index_after_each_power_loss
+        .iter()
+        .map(|first_stream_index_after_the_power_loss| {
+            stream_indexes
+                .iter()
+                .position(|stream_index| *stream_index >= *first_stream_index_after_the_power_loss)
+                .expect("掉电之后那次可写挂载与之后的覆盖写都写了东西")
+        })
+        .collect();
+    HistoryLeavingOrphans {
+        writes,
+        first_write_after_each_power_loss,
+    }
+}
+
+/// 掉电之前最后一条根槽写在写表里的下标（实例 1 的最后一次发布）。
+fn last_root_write_before(
+    writes: &[RetainedWrite],
+    first_write_after_the_power_loss: usize,
+) -> usize {
+    (0..first_write_after_the_power_loss)
+        .rev()
+        .find(|write_index| writes[*write_index].kind == StepKind::RootRecordFua)
+        .expect("从第一个文件起：掉电之前写表里有实例 1 的根")
+}
+
+/// 掉电之前最后一条根槽写之后、掉电接缝之前的单元写：那次掉电的改动落了盘、没有根的那几个。
+fn unrooted_unit_writes_before_the_power_loss(
+    writes: &[RetainedWrite],
+    first_write_after_the_power_loss: usize,
+) -> Vec<usize> {
+    (last_root_write_before(writes, first_write_after_the_power_loss) + 1
+        ..first_write_after_the_power_loss)
+        .filter(|write_index| writes[*write_index].kind == StepKind::UnitWrite)
+        .collect()
+}
+
+#[test]
+fn the_power_loss_step_records_one_seam_between_the_unrooted_unit_writes_of_instance_one_and_the_mount_of_instance_two(
+) {
+    let history = record_the_history_leaving_orphans();
+    assert_eq!(
+        history.first_write_after_each_power_loss.len(),
+        1,
+        "这段历史掉了一次电，执行器记下一道接缝"
+    );
+    let seam = history.first_write_after_each_power_loss[0];
+    let last_root_before_the_power_loss = last_root_write_before(&history.writes, seam);
+    assert_eq!(
+        root_identity_of_write(&history.writes[last_root_before_the_power_loss]).0,
+        InstanceGeneration(1),
+        "接缝之前最后一条根是实例 1 的"
+    );
+    let writes_between_the_last_root_and_the_seam =
+        &history.writes[last_root_before_the_power_loss + 1..seam];
+    assert!(
+        writes_between_the_last_root_and_the_seam
+            .iter()
+            .all(|write| write.kind != StepKind::JournalRecord
+                && write.kind != StepKind::RootRecordFua),
+        "掉电截在第一个 journal 记录写上：接缝之前那次改动一条记录、一条根都没写下"
+    );
+    assert!(
+        !unrooted_unit_writes_before_the_power_loss(&history.writes, seam).is_empty(),
+        "接缝之前那次改动的单元写落了盘"
+    );
+    let first_root_after_the_seam = (seam..history.writes.len())
+        .find(|write_index| history.writes[*write_index].kind == StepKind::RootRecordFua)
+        .expect("掉电之后的可写挂载写了根");
+    assert_eq!(
+        root_identity_of_write(&history.writes[first_root_after_the_seam]).0,
+        InstanceGeneration(2),
+        "接缝之后第一条根是掉电之后可写挂载另起的实例 2 写的"
+    );
+}
+
+#[test]
+fn publishes_broken_at_the_power_loss_seam_put_the_unrooted_unit_writes_of_instance_one_in_no_publish(
+) {
+    let history = record_the_history_leaving_orphans();
+    let seam = history.first_write_after_each_power_loss[0];
+    let unrooted_unit_writes = unrooted_unit_writes_before_the_power_loss(&history.writes, seam);
+    let every_write_persisted = vec![true; history.writes.len()];
+    let first_publish_rooted_after = |publishes: &[PublishWrites]| {
+        publishes
+            .iter()
+            .position(|publish| publish.root >= seam)
+            .expect("掉电之后的可写挂载发过布")
+    };
+
+    let as_one_recording = publishes_in(
+        &history.writes,
+        &every_write_persisted,
+        RecordStreamContinuity::OneRecording,
+    );
+    let row_publish_as_one_recording =
+        &as_one_recording[first_publish_rooted_after(&as_one_recording)];
+    assert!(
+        unrooted_unit_writes
+            .iter()
+            .all(|unit| row_publish_as_one_recording.units.contains(unit)),
+        "前提：接成一条来分，实例 1 那几个没有根的单元写归进掉电之后的第一次发布 ({}, {})",
+        row_publish_as_one_recording.instance,
+        row_publish_as_one_recording.checkpoint_txg
+    );
+
+    let broken_at_the_seam = publishes_in(
+        &history.writes,
+        &every_write_persisted,
+        RecordStreamContinuity::BrokenAtSeams(&[RecordStreamSeam::AfterAPowerLoss {
+            first_write_after_the_power_loss: seam,
+        }]),
+    );
+    for publish in &broken_at_the_seam {
+        assert!(
+            unrooted_unit_writes
+                .iter()
+                .all(|unit| !publish.units.contains(unit)),
+            "在掉电接缝处断开：实例 1 那几个没有根的单元写不归任何一次发布，却归进了 ({}, {})",
+            publish.instance,
+            publish.checkpoint_txg
+        );
+    }
+    let row_publish = &broken_at_the_seam[first_publish_rooted_after(&broken_at_the_seam)];
+    assert_eq!(
+        row_publish.instance, 2,
+        "掉电之后的第一次发布是实例 2 写行那次"
+    );
+    assert!(
+        row_publish.units.iter().all(|unit| *unit >= seam),
+        "掉电之后的第一次发布只有接缝之后的单元写：{:?}",
+        row_publish.units
+    );
+    assert_eq!(
+        broken_at_the_seam.len(),
+        as_one_recording.len(),
+        "断开只改单元写归哪次发布，发布还是那几次"
+    );
+}
+
+#[test]
+#[should_panic(expected = "掉电那道接缝")]
+fn a_power_loss_seam_whose_earlier_recording_has_a_write_withheld_by_the_persisted_set_is_refused()
+{
+    let history = record_the_history_leaving_orphans();
+    let seam = history.first_write_after_each_power_loss[0];
+    let mut persisted = vec![true; history.writes.len()];
+    persisted[seam - 1] = false;
+    let _publishes = publishes_in(
+        &history.writes,
+        &persisted,
+        RecordStreamContinuity::BrokenAtSeams(&[RecordStreamSeam::AfterAPowerLoss {
+            first_write_after_the_power_loss: seam,
+        }]),
+    );
+}
diff --git a/crates/singlefs-checker-tier/tests/record_checker_breaks_the_history_at_the_power_loss_seam.rs b/crates/singlefs-checker-tier/tests/record_checker_breaks_the_history_at_the_power_loss_seam.rs
new file mode 100644
index 0000000..f5b0814
--- /dev/null
+++ b/crates/singlefs-checker-tier/tests/record_checker_breaks_the_history_at_the_power_loss_seam.rs
@@ -0,0 +1,207 @@
+//! checker 档模块：crash
+//! 记录核对器第二条判据在掉电那道接缝处断开（`memory_pool::RecordStreamSeam::AfterAPowerLoss`）。留孤儿那段历史
+//! （`history::history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`）最后一步开头摆一个崩溃状态
+//! （更早的写全落、最后一步一个都没落，崩溃注入 `inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans`
+//! 第一截的同一种摆法），只读恢复落到最后一次发布。掉电那次覆盖写落了盘、没有根的单元写接成一条来分时归进掉电之后写行那次发布 (2, 4)，
+//! 第 1 步覆盖写合法复用了它们的落点，回收谓词却按 (2, 4) 判那次复用证得出不合法，于是判 (2, 4) 缺了单元（假红）；
+//! 在接缝处断开，那几个写不归任何一次发布，不红。阳性对照：断开之后把最后一次发布的一个单元在镜像上盖掉，照样红。
+//! 分发布那一半（接缝记在哪、断开之后那几个写归不归发布）在 harness 档
+//! （`crates/singlefs-harness/tests/publishes_of_a_history_broken_at_the_power_loss_seam.rs`）。
+
+use singlefs_checker_tier::crash::check_records_against;
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::recovery::{recover, JournalPolicy};
+use singlefs_harness::history::{
+    execute_history_with,
+    history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring,
+    HistoryDeviceWidth, HistoryEnding, HistoryExecution, PerStepChecker, StepPosition,
+};
+use singlefs_harness::memory_pool::{
+    root_identity_of_write, writes_and_segments_with_stream_indexes, MemoryPool, RecordCheck,
+    RecordStreamContinuity, RecordStreamSeam, RetainedWrite, WrittenContents,
+};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::SharedStream;
+
+/// 历史本身不跑每一步的 checker，两块 4 GiB 的盘（与崩溃注入快档同一种跑法）。
+const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
+    per_step_checker: PerStepChecker::Skipped,
+    device_width: HistoryDeviceWidth::FourGibibytes,
+    space_admission: SpaceAdmission::JudgedByTheFormula,
+};
+
+/// 最后一步开头那个崩溃状态：写表、持久集合（最后一步第一个写之前全落、之后一个都没落）、崩溃镜像（基线加落了的写）、
+/// 只读恢复落到的那一版、掉电接缝在写表里的下标、最后一步之前最后一条根槽写的下标。
+struct CrashAtTheStartOfTheLastStep {
+    writes: Vec<RetainedWrite>,
+    persisted: Vec<bool>,
+    first_write_of_the_last_step: usize,
+    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
+    first_write_after_the_power_loss: usize,
+    last_root_write_before_the_crash: usize,
+}
+
+fn crash_at_the_start_of_the_last_step() -> CrashAtTheStartOfTheLastStep {
+    let history = history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring();
+    let step_before_the_last = StepPosition::Operation(history.operations.len() - 2);
+    let stream = SharedStream::retaining_contents();
+    let mut operations_before_the_last_step = None;
+    let run = execute_history_with(
+        &history,
+        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
+        &stream,
+        &mut |observation| {
+            if observation.position == step_before_the_last {
+                operations_before_the_last_step = Some(stream.operation_count());
+            }
+        },
+    );
+    assert_eq!(
+        run.ending,
+        HistoryEnding::Completed,
+        "这段历史每一步都做得成"
+    );
+    let operations_before_the_last_step =
+        operations_before_the_last_step.expect("倒数第二步之后观察者被调过");
+    let (writes, _segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+        &stream.retained_operations(),
+        &UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
+            .device_width
+            .fixed_geometry(),
+    );
+    let first_write_of_the_last_step = stream_indexes
+        .iter()
+        .position(|stream_index| *stream_index >= operations_before_the_last_step)
+        .expect("最后一步（覆盖写）写过东西");
+    assert_eq!(
+        run.first_stream_index_after_each_power_loss.len(),
+        1,
+        "这段历史掉了一次电"
+    );
+    let first_write_after_the_power_loss = stream_indexes
+        .iter()
+        .position(|stream_index| *stream_index >= run.first_stream_index_after_each_power_loss[0])
+        .expect("掉电之后那次可写挂载写了东西");
+    let last_root_write_before_the_crash = (0..first_write_of_the_last_step)
+        .rev()
+        .find(|write_index| writes[*write_index].kind == StepKind::RootRecordFua)
+        .expect("最后一步之前写过根");
+    let mut persisted = vec![false; writes.len()];
+    persisted[..first_write_of_the_last_step].fill(true);
+    let effective_root = recover(
+        &crash_image(&writes, first_write_of_the_last_step, &[]),
+        JournalPolicy::Consult,
+    )
+    .effective_root;
+    CrashAtTheStartOfTheLastStep {
+        writes,
+        persisted,
+        first_write_of_the_last_step,
+        effective_root,
+        first_write_after_the_power_loss,
+        last_root_write_before_the_crash,
+    }
+}
+
+/// 写表前 `first_write_withheld` 个写落在两块空盘上，再叠上 `damage`。
+fn crash_image(
+    writes: &[RetainedWrite],
+    first_write_withheld: usize,
+    damage: &[RetainedWrite],
+) -> MemoryPool {
+    let mut image = MemoryPool::with_devices(
+        &[DeviceIdentity(0), DeviceIdentity(1)],
+        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
+            .device_width
+            .device_bytes(),
+    );
+    image.apply_writes(&writes[..first_write_withheld]);
+    image.apply_writes(damage);
+    image
+}
+
+fn power_loss_seam_of(crash: &CrashAtTheStartOfTheLastStep) -> [RecordStreamSeam; 1] {
+    [RecordStreamSeam::AfterAPowerLoss {
+        first_write_after_the_power_loss: crash.first_write_after_the_power_loss,
+    }]
+}
+
+#[test]
+fn the_crash_at_the_start_of_the_last_step_of_the_history_leaving_orphans_misses_no_unit_once_broken_at_the_power_loss_seam(
+) {
+    let crash = crash_at_the_start_of_the_last_step();
+    assert_eq!(
+        crash.effective_root,
+        Some(root_identity_of_write(
+            &crash.writes[crash.last_root_write_before_the_crash]
+        )),
+        "前提：只读恢复落到最后一步之前那次发布"
+    );
+    let image = crash_image(&crash.writes, crash.first_write_of_the_last_step, &[]);
+    let as_one_recording = check_records_against(
+        &image,
+        &image,
+        &crash.writes,
+        &crash.persisted,
+        RecordStreamContinuity::OneRecording,
+        crash.effective_root,
+    );
+    assert!(
+        as_one_recording.claimed_state_missing_unit,
+        "前提：接成一条来分时，掉电那次改动没有根的单元写归进写行那次发布，判它缺了单元"
+    );
+    assert_eq!(
+        check_records_against(
+            &image,
+            &image,
+            &crash.writes,
+            &crash.persisted,
+            RecordStreamContinuity::BrokenAtSeams(&power_loss_seam_of(&crash)),
+            crash.effective_root,
+        ),
+        RecordCheck::default(),
+        "在掉电接缝处断开：那几个写不归任何一次发布，记录核对器两条判据都不红"
+    );
+}
+
+#[test]
+fn a_unit_of_the_newest_publish_damaged_on_the_crash_image_is_still_missing_once_broken_at_the_power_loss_seam(
+) {
+    let crash = crash_at_the_start_of_the_last_step();
+    let first_unit_of_the_newest_publish = (0..crash.last_root_write_before_the_crash)
+        .rev()
+        .take_while(|write_index| crash.writes[*write_index].kind != StepKind::RootRecordFua)
+        .filter(|write_index| crash.writes[*write_index].kind == StepKind::UnitWrite)
+        .min()
+        .expect("最后一步之前那次发布（覆盖写）写了单元");
+    let unit = &crash.writes[first_unit_of_the_newest_publish];
+    let unit_bytes = usize::try_from(unit.length_in_bytes()).expect("写长装得进 usize");
+    let damage_on_both_devices: Vec<RetainedWrite> = [DeviceIdentity(0), DeviceIdentity(1)]
+        .into_iter()
+        .map(|device| RetainedWrite {
+            device,
+            kind: StepKind::UnitWrite,
+            is_force_unit_access: false,
+            offset: unit.offset,
+            contents: WrittenContents::Bytes(vec![0xA5; unit_bytes]),
+        })
+        .collect();
+    let damaged_image = crash_image(
+        &crash.writes,
+        crash.first_write_of_the_last_step,
+        &damage_on_both_devices,
+    );
+    assert!(
+        check_records_against(
+            &damaged_image,
+            &damaged_image,
+            &crash.writes,
+            &crash.persisted,
+            RecordStreamContinuity::BrokenAtSeams(&power_loss_seam_of(&crash)),
+            crash.effective_root,
+        )
+        .claimed_state_missing_unit,
+        "断开之后最后一次发布的单元两份都盖掉了：照样判缺单元"
+    );
+}
````

### mutations-append.tsv

````
掉电步不记接缝（掉电之后录制流里第一步的下标没交回）	crates/singlefs-harness/src/history.rs	    pool.first_stream_index_after_each_power_loss\n        .push(pool.stream.operation_count());	    let _ = pool.stream.operation_count();	-p singlefs-harness --test publishes_of_a_history_broken_at_the_power_loss_seam -- the_power_loss_step_records_one_seam	the_power_loss_step_records_one_seam_between_the_unrooted_unit_writes_of_instance_one_and_the_mount_of_instance_two
掉电接缝落在写表开头（掉电之前那一截不断开，没有根的单元写归进掉电之后的第一次发布）	crates/singlefs-harness/src/memory_pool.rs	            } => *first_write_after_the_power_loss,	            } => *first_write_after_the_power_loss * 0,	-p singlefs-harness --test publishes_of_a_history_broken_at_the_power_loss_seam -- publishes_broken_at_the_power_loss_seam	publishes_broken_at_the_power_loss_seam_put_the_unrooted_unit_writes_of_instance_one_in_no_publish
掉电接缝之前那一截不核持久集合（崩溃落在掉电之前也照断）	crates/singlefs-harness/src/memory_pool.rs	                                .all(|is_persisted| *is_persisted),	                                .all(|_is_persisted| true),	-p singlefs-harness --test publishes_of_a_history_broken_at_the_power_loss_seam -- a_power_loss_seam_whose_earlier_recording	a_power_loss_seam_whose_earlier_recording_has_a_write_withheld_by_the_persisted_set_is_refused
记录核对器的掉电接缝落在写表开头（留孤儿那段历史最后一步开头判 (2, 4) 缺单元）	crates/singlefs-harness/src/memory_pool.rs	            } => *first_write_after_the_power_loss,	            } => *first_write_after_the_power_loss * 0,	-p singlefs-checker-tier --test record_checker_breaks_the_history_at_the_power_loss_seam -- the_crash_at_the_start	the_crash_at_the_start_of_the_last_step_of_the_history_leaving_orphans_misses_no_unit_once_broken_at_the_power_loss_seam
崩溃注入三截都不给掉电接缝（第一截起就接成一条来分）	crates/singlefs-checker-tier/src/crash_injection.rs	                    *first_write_after_the_power_loss <= first_write_of_the_segment	                    *first_write_after_the_power_loss <= first_write_of_the_segment && false	-p singlefs-checker-tier --lib -- crash_injection::tests::a_writable_mount_after_the_crash_whose_sweep_erases_heads_of_orphans_is_judged_in_the_erasing_phase_without_a_red_field	a_writable_mount_after_the_crash_whose_sweep_erases_heads_of_orphans_is_judged_in_the_erasing_phase_without_a_red_field
崩溃注入第二、三截不给掉电接缝（只在历史与挂载之间断开）	crates/singlefs-checker-tier/src/crash_injection.rs	        let mut seams = power_loss_seams(first_write_after_each_power_loss_before_the_crash);	        let mut seams = power_loss_seams(&first_write_after_each_power_loss_before_the_crash[..0]);	-p singlefs-checker-tier --lib -- crash_injection::tests::a_writable_mount_after_the_crash_whose_sweep_erases_heads_of_orphans_is_judged_in_the_erasing_phase_without_a_red_field	a_writable_mount_after_the_crash_whose_sweep_erases_heads_of_orphans_is_judged_in_the_erasing_phase_without_a_red_field
````

## batch10d-anchors

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10d-anchors/patch/`
- `mutations-replacements.tsv`：sha256 99fabb1d67d7b6abff121d7cd4e7f24d8151e424872dce9842fede4e1ae3373a
- `report.md`：sha256 08879aaf4298720e3f33664a1a418680606a852a34160e6069a9859cd6e9084b

### mutations-replacements.tsv

````
增补 3 第 3 件（用户 2026-09-20 定案第 3 条）：崩溃状态上不跑记录核对器	crates/singlefs-checker-tier/src/crash_injection.rs	        let record_check = check_records_against(\n            &image,\n            &image,\n            &writes,\n            &persisted,\n            if power_loss_seams_before_the_crash.is_empty() {\n                RecordStreamContinuity::OneRecording\n            } else {\n                RecordStreamContinuity::BrokenAtSeams(&power_loss_seams_before_the_crash)\n            },\n            report.effective_root,\n        );\n        tally.record_checks += 1;	        let record_check = RecordCheck::default();	-p singlefs-checker-tier --lib -- crash_injection::tests::crash_points_are_reproducible	crash_points_are_reproducible_proper_subsets_sorted_by_segment
实审 B3c-1：崩溃注入第一截交给记录核对器的写表退回崩溃点所在段为止的前缀（恢复落到由记录重建的一版时找不到它的根槽写、读不出它的实例表，被它抛弃的发布的豁免落空）	crates/singlefs-checker-tier/src/crash_injection.rs	            &writes,\n            &persisted,\n            if power_loss_seams_before_the_crash.is_empty() {	            &writes[..first_write_of_the_segment + writes_in_the_segment],\n            &persisted[..first_write_of_the_segment + writes_in_the_segment],\n            if power_loss_seams_before_the_crash.is_empty() {	-p singlefs-checker-tier --test crash_injection_campaign -- crash_state_landing_on_a_version_rebuilt_from_records	crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons
实审 B3c-2 第 1 件：接缝处不剔截断的流里没发生过的发布（截断处之后才写根槽写、恢复又没落到的发布也算恢复自称的那一版该有的）	crates/singlefs-harness/src/memory_pool.rs	                            .filter(|publish| {\n                                persisted[publish.root]\n	                            .filter(|publish| {\n                                true || persisted[publish.root]\n	-p singlefs-checker-tier --test crash_injection_record_checker_sees_every_publish_across_a_second_crash -- publishes_cut_off_by_the_crash_are_on_the_resumed_timeline_only_if_their_root_landed_or_the_recovery_landed_on_them	publishes_cut_off_by_the_crash_are_on_the_resumed_timeline_only_if_their_root_landed_or_the_recovery_landed_on_them
实审 B3c-2 第 1 件：接缝处不剔截断的流里没发生过的发布（真实历史：截断处之后历史重开写的发布与挂载写的撞了身份）	crates/singlefs-harness/src/memory_pool.rs	                            .filter(|publish| {\n                                persisted[publish.root]\n	                            .filter(|publish| {\n                                true || persisted[publish.root]\n	-p singlefs-checker-tier --test crash_injection_record_checker_sees_every_publish_across_a_second_crash -- first_crashes_cut_inside_publishes_leave_every_second_crash_and_the_pool_after_the_mount_clean	first_crashes_cut_inside_publishes_leave_every_second_crash_and_the_pool_after_the_mount_clean
实审 B3c-2 第 1 件：接缝处不认第一次崩溃之后恢复落到的那一版（由记录重建、根槽写在截断处之后的那次发布被剔掉，它缺单元不判）	crates/singlefs-harness/src/memory_pool.rs	                                    || *version_landed_on_after_the_crash\n	                                    || false && *version_landed_on_after_the_crash\n	-p singlefs-checker-tier --test crash_injection_record_checker_sees_every_publish_across_a_second_crash -- publishes_cut_off_by_the_crash_are_on_the_resumed_timeline_only_if_their_root_landed_or_the_recovery_landed_on_them	publishes_cut_off_by_the_crash_are_on_the_resumed_timeline_only_if_their_root_landed_or_the_recovery_landed_on_them
实审 B3c-2 第 1 件：接缝处连根槽写落了盘的发布也剔掉（截断的那条流里只留恢复落到的那一版）	crates/singlefs-harness/src/memory_pool.rs	                                persisted[publish.root]\n                                    || *version_landed_on_after_the_crash	                                false\n                                    || *version_landed_on_after_the_crash	-p singlefs-checker-tier --test crash_injection_record_checker_sees_every_publish_across_a_second_crash -- publishes_cut_off_by_the_crash_are_on_the_resumed_timeline_only_if_their_root_landed_or_the_recovery_landed_on_them	publishes_cut_off_by_the_crash_are_on_the_resumed_timeline_only_if_their_root_landed_or_the_recovery_landed_on_them
实审 B3c-2 第 1 件：两条流接成一条来分（截断的那条流末尾没等到根槽写的写归进挂载的第一次发布）	crates/singlefs-harness/src/memory_pool.rs	            for seam in seams {\n	            for seam in seams\n                .iter()\n                .filter(|seam| matches!(seam, RecordStreamSeam::AfterAPowerLoss { .. }))\n            {\n	-p singlefs-checker-tier --test crash_injection_record_checker_sees_every_publish_across_a_second_crash -- writes_left_without_a_root_at_the_end_of_the_cut_recording_do_not_join_the_first_publish_after_the_crash	writes_left_without_a_root_at_the_end_of_the_cut_recording_do_not_join_the_first_publish_after_the_crash
增补 3 第 3 件第 1 题：崩溃镜像那一截交给记录核对器的恢复落到的那一版换成 None（第二条判据的前置恒不成立，缺单元的靶向崩溃状态判不出）	crates/singlefs-checker-tier/src/crash_injection.rs	                RecordStreamContinuity::BrokenAtSeams(&power_loss_seams_before_the_crash)\n            },\n            report.effective_root,\n        );	                RecordStreamContinuity::BrokenAtSeams(&power_loss_seams_before_the_crash)\n            },\n            None,\n        );	-p singlefs-checker-tier --test crash_injection_campaign -- a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker	a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker
````

## batch10d-row927

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10d-row927/patch/`
- `crates.patch`：sha256 64b4306cc74fd5a2f39f2bcbfab73768ffe553f7a28b3e9a994a29248ef7ff15
- `mutations-append.tsv`：sha256 5eca5b0b0b6c03916f32dc8280b9eeab75c85632f29e4f519fbf6bb7bdb82553
- `mutations-delete.txt`：sha256 a653b20e6c0fc8cc0d6b81b3b7598b75602b1f2a67cc11c96cb969576a1cf708
- `report.md`：sha256 4cc0f7ac1fbc2ebff263a1ef4ecbda4a5a171ba376821ba3cee6ae95ff2e9450

### crates.patch

````
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs b/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs
index 99fcf502..f3e1a5b7 100644
--- a/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs
+++ b/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs
@@ -11,7 +11,7 @@ use std::collections::BTreeMap;
 use std::io::Write as _;
 
 use singlefs_checker::walk::InstanceTableOfRootRecord;
-use singlefs_checker_tier::crash::check_records;
+use singlefs_checker_tier::crash::{check_records, check_records_against};
 use singlefs_checker_tier::crash_injection::{
     crash_injection_campaign_from_environment, inject_crashes_into_history,
     run_crash_injection_campaign, CrashInjectionAssertions, CrashInjectionCampaign,
@@ -33,7 +33,7 @@ use singlefs_harness::history::{
 };
 use singlefs_harness::memory_pool::{
     root_identity_written_by, writes_and_segments, CrashImage, MemoryPool, RecordCheck,
-    RetainedWrite,
+    RecordStreamContinuity, RetainedWrite,
 };
 use singlefs_harness::numeric_environment_variable::process_environment_variable;
 use singlefs_harness::segments::StepKind;
@@ -1601,6 +1601,121 @@ fn a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_findi
     );
 }
 
+/// 崩溃注入第一截交给记录核对器的写表取整条历史录制流（`crash_injection.rs` 第一截那一次 `check_records_against`），
+/// 与只取到崩溃点所在段为止的前缀，判出来不同只在两种崩溃状态上（推的：前缀之后的写一个都没持久，缺席判定与回收谓词只用得到
+/// 持久的后写与更早的写；整条流多出来的只有根槽写在前缀之后的那几次发布，与恢复落到的那一版的根槽写）：
+/// - 恢复落到由记录重建的一版、它的实例表抛弃了某次单元已被复用的发布：前缀里没有那一版的根槽写，实例表读不出，豁免落空。
+///   C554 乙之后这一形摆不出来（[`crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons`]、
+///   [`units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`] 钉着）。
+/// - 恢复落到由记录重建的一版、那一版自己的某个单元两份都不在：前缀里没有它的根槽写，那次发布归不出来，第二条判据罩不到它。
+///   这一条造的就是它：第二次覆盖写 (1, 5) 的第一个数据单元两份（第 20 段头两个写）扣下，第 20 段别的单元写与第 21 段两份 journal 记录都持久，
+///   第 22 段 (1, 5) 的根槽写在窗口之后、没落。验点名单元的恢复（`JournalPolicy::Consult`，崩溃注入用的就是它）不施加那条记录、落回 (1, 4)，
+///   两种交法同判；只有不验点名单元的恢复（只供测试强制进入的 `JournalPolicy::ConsultWithoutNamedVerification`）落到 (1, 5)，整条流判红、前缀不判。
+///
+/// 所以在崩溃注入摆得出的每个崩溃状态上两种交法同值，把第一截的写表退回前缀那一条变异是等价变异，这一条给它留档
+/// （`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」）；C554 乙那一判或点名验证哪天不在了，两种交法才会分开。
+/// 第二条判据不罩恰好是恢复落到的那一版的那次发布（`crates/mutations.tsv`），整条流那一判不红，这一条红。
+#[test]
+fn a_version_rebuilt_from_records_missing_a_unit_is_judged_only_on_the_whole_stream_and_the_verifying_recovery_never_lands_on_it(
+) {
+    let history = history_of_a_first_file_and_two_overwrites();
+    let (writes, segments) = writes_and_segments_of(&history);
+    let root_of_the_second_overwrite = segments[22][0];
+    assert_eq!(
+        root_identity_of_a_root_slot_write(&writes[root_of_the_second_overwrite]),
+        Some((InstanceGeneration(1), CheckpointTxg(5))),
+        "第 22 段的头一个写是第二次覆盖写 (1, 5) 的根槽写"
+    );
+    assert_eq!(
+        segments[21]
+            .iter()
+            .map(|index| writes[*index].kind)
+            .collect::<Vec<_>>(),
+        vec![StepKind::JournalRecord, StepKind::JournalRecord],
+        "第 21 段是 (1, 5) 的两份 journal 记录"
+    );
+    let first_write = segments[20][0];
+    let withheld_copies = &writes[first_write..first_write + 2];
+    assert!(
+        withheld_copies
+            .iter()
+            .all(|write| write.kind == StepKind::UnitWrite
+                && write.length_in_bytes() == DATA_UNIT_BYTES
+                && write.offset == withheld_copies[0].offset)
+            && withheld_copies[0].device != withheld_copies[1].device,
+        "第 20 段的头两个写是 (1, 5) 一个数据单元在两块盘上的两份：{withheld_copies:?}"
+    );
+    let end_of_the_window = segments[21][0] + segments[21].len();
+    assert_eq!(
+        end_of_the_window, root_of_the_second_overwrite,
+        "窗口（第 20、21 段）截在 (1, 5) 的根槽写之前"
+    );
+
+    let persisted_within_the_window: Vec<bool> = (first_write..end_of_the_window)
+        .map(|index| index >= first_write + 2)
+        .collect();
+    let persisted =
+        persisted_over_the_whole_stream(writes.len(), first_write, &persisted_within_the_window);
+    let base = pool_of_the_persisted_writes_before_the_segment(
+        &writes[..first_write],
+        &persisted,
+        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
+            .device_width
+            .device_bytes(),
+    );
+    let image = CrashImage {
+        base: &base,
+        writes: &writes[first_write..end_of_the_window],
+        persisted: persisted_within_the_window,
+    };
+    // 第一截的交法：两份镜像同一份、一条录制流（这段历史没掉过电），写表与持久集合取到 `end_of_table` 为止。
+    let record_check_on_the_table_up_to =
+        |end_of_table: usize, effective_root: Option<(InstanceGeneration, CheckpointTxg)>| {
+            check_records_against(
+                &image,
+                &image,
+                &writes[..end_of_table],
+                &persisted[..end_of_table],
+                RecordStreamContinuity::OneRecording,
+                effective_root,
+            )
+        };
+
+    let verifying = recover(&image, JournalPolicy::Consult).effective_root;
+    assert_eq!(
+        verifying,
+        Some((InstanceGeneration(1), CheckpointTxg(4))),
+        "验点名单元的恢复不施加 (1, 5) 的记录（它点名的数据单元两份都不在），落回 (1, 4)"
+    );
+    assert_eq!(
+        record_check_on_the_table_up_to(writes.len(), verifying),
+        RecordCheck::default(),
+        "落回 (1, 4)：整条流上记录核对器一条都不判"
+    );
+    assert_eq!(
+        record_check_on_the_table_up_to(end_of_the_window, verifying),
+        RecordCheck::default(),
+        "落回 (1, 4)：前缀上也一条都不判，两种交法同值"
+    );
+
+    let unverifying =
+        recover(&image, JournalPolicy::ConsultWithoutNamedVerification).effective_root;
+    assert_eq!(
+        unverifying,
+        Some((InstanceGeneration(1), CheckpointTxg(5))),
+        "不验点名单元的恢复施加 (1, 5) 的记录，落到由记录重建的 (1, 5)"
+    );
+    assert!(
+        record_check_on_the_table_up_to(writes.len(), unverifying).claimed_state_missing_unit,
+        "整条流里有 (1, 5) 的根槽写，那次发布归得出来，它的数据单元两份都不在：判「恢复自称新态而单元缺席」"
+    );
+    assert_eq!(
+        record_check_on_the_table_up_to(end_of_the_window, unverifying),
+        RecordCheck::default(),
+        "前缀里没有 (1, 5) 的根槽写，那次发布归不出来，第二条判据罩不到它"
+    );
+}
+
 /// 第一个文件之后接着覆盖写的次数：根环三个区域各 8 槽，第 24 次覆盖写 (1, 27) 的根槽写盖掉 (1, 3) 的那一槽。
 const OVERWRITES_THAT_TURN_THE_ROOT_RING_PAST_THE_FIRST_FILE: u64 = 24;
 
````

### mutations-append.tsv

````
实审 B3c-1：崩溃注入第一截交给记录核对器的写表少了崩溃窗口末尾那个写（窗口里落了盘的根槽写不在表里，根在而记录一条都不在的靶向崩溃状态判不出）	crates/singlefs-checker-tier/src/crash_injection.rs	            &writes,\n            &persisted,\n            if power_loss_seams_before_the_crash.is_empty() {	            &writes[..first_write_of_the_segment + writes_in_the_segment - 1],\n            &persisted[..first_write_of_the_segment + writes_in_the_segment - 1],\n            if power_loss_seams_before_the_crash.is_empty() {	-p singlefs-checker-tier --test crash_injection_campaign -- a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone	a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone
实审 B3c-1 第一截写表整条流与前缀等价的留档：记录核对器第二条判据不罩恰好是恢复落到的那一版的那次发布（由记录重建、自己的数据单元两份都不在的 (1, 5) 在整条流上也判不出）	crates/singlefs-checker-tier/src/crash.rs	txg.0 >= publish.checkpoint_txg	txg.0 > publish.checkpoint_txg	-p singlefs-checker-tier --test crash_injection_campaign -- a_version_rebuilt_from_records_missing_a_unit	a_version_rebuilt_from_records_missing_a_unit_is_judged_only_on_the_whole_stream_and_the_verifying_recovery_never_lands_on_it
````

### mutations-delete.txt

````
实审 B3c-1：崩溃注入第一截交给记录核对器的写表退回崩溃点所在段为止的前缀（恢复落到由记录重建的一版时找不到它的根槽写、读不出它的实例表，被它抛弃的发布的豁免落空）
````

## batch10e-reread-ring

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10e-reread-ring/patch/`
- `crates.patch`：sha256 09495e7d2640707cd0da7f07d778989368b99587ef944b29313c7cb39a6fe84b
- `mutations-append.tsv`：sha256 fe0c098ab36130c2a676d7231430680969a59a7e40ee68e545448d9dcd0de33d
- `mutations-replacements.tsv`：sha256 fa627518ab76b942005811ebdb63fea2ee8442f673113533786414591621832c
- `report.md`：sha256 1dc9b11260e1bb95eb86250d0cd40e78ce2bf6ab00e592c0bc920e03d350eb59

### crates.patch

````
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 1c7f58f..ef85e7c 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -504,6 +504,7 @@ impl BeforeTheOneReread<'_> {
 pub struct RereadsOfThisMount {
     pub read_stage: ReadStageSettled,
     pub instance_table_of_the_newest_root: InstanceTableOfTheNewestRootRead,
+    pub root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead,
 }
 
 /// 读阶段在哪一遍判完（判据 N-配置 为假的那一遍就是往下走用的那一遍：它的所选根、记录与所选那一版）。
@@ -526,6 +527,14 @@ pub enum InstanceTableOfTheNewestRootRead {
     OnTheOneReread,
 }
 
+/// 重建分配器读根环（根环表、回收门槛与影子账用的那一遍读数）在哪一遍读出不被抛弃的自证有效根：第一遍一条都没读出就整环重读一次
+/// （用户 2026-10-05 定「重读」），重读也没有就拒（`MountError::Recovery(RecoveryFailure::NoValidRoot)`，不交回这个值）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RootRingOfTheRebuiltAllocatorRead {
+    OnTheFirstRead,
+    OnTheOneReread,
+}
+
 /// 到了顶的是哪一个编号、它现在的值（`MountError::SequenceNumberPastTheTopOfItsRange`）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum SequenceNumberAtTheTopOfItsRange {
@@ -945,7 +954,7 @@ pub struct MountOutput {
     pub space_admission: MountSpaceAdmission,
     /// 这次挂载认下的池（盘上那份参数、核过的盘表）：挂着的会话只用它（实审 A1b Q2、Q3）。
     pub parameters_and_device_table: ParametersAndDeviceTableOfTheMount,
-    /// 这次挂载重读过哪几样（C554 乙）：读阶段在哪一遍判完、最新那条根的实例表在哪一遍读出来。
+    /// 这次挂载重读过哪几样（C554 乙）：读阶段在哪一遍判完、最新那条根的实例表在哪一遍读出来、重建分配器读根环在哪一遍读出有效根。
     pub rereads: RereadsOfThisMount,
     /// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）发成的每一步，按先后：接在写行、暖机与推的抬 F 之后，走到了空时最后一个就是
     /// `Mounted::current`；所选那一版的 `deleted_inodes` 是空的（常态）时空。
@@ -1396,6 +1405,7 @@ struct RebuiltAllocator {
     effective_floor: CheckpointTxg,
     abandoned_roots_unreadable: u64,
     instance_table_of_the_newest_root: InstanceTableOfTheNewestRootRead,
+    root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead,
 }
 
 /// 重建分配器判哪几条根被抛弃用的那张表：最新那条根（`choose_root` 现择）指着的实例表（代码审阅第 22 条）。
@@ -1816,7 +1826,9 @@ fn txgs_ring_slots_bad_at_mount_can_hold(
 ///
 /// # Errors
 /// 最新那条根的实例表读不出、重读一次仍读不出（[`instance_table_of_the_newest_root_read_at_most_twice`]）——在读阶段之后、取号之前；
-/// 树表 0 条那一版的账重建不出（`allocator_of_version_without_file`）。
+/// 树表 0 条那一版的账重建不出（`allocator_of_version_without_file`）；
+/// 读根环那一遍一条不被抛弃的自证有效根都没读出、整环重读一次仍没有 ⇒ `Recovery(RecoveryFailure::NoValidRoot)`（用户 2026-10-05 定「重读」）。
+/// 都在取号之前，盘上逐字节不变。
 #[allow(
     clippy::ptr_arg,
     reason = "choose_root 等走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
@@ -1865,22 +1877,65 @@ fn rebuilt_allocator<Device: BlockDevice>(
         .immutable
         .sizes
         .root_ring_slots_per_region;
-    let ring_slot_readings: Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)> =
-        every_root_ring_slot(&system_configuration.immutable.sizes)
-            .into_iter()
-            .map(|ring_slot| {
-                (
-                    ring_slot,
-                    read_root_ring_slot_telling_never_written_apart(
-                        devices,
-                        &system_configuration.immutable.region_devices,
-                        &system_configuration.immutable.sizes,
-                        &system_configuration.immutable.filesystem_identifier,
+    let read_every_root_ring_slot_once =
+        || -> Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)> {
+            every_root_ring_slot(&system_configuration.immutable.sizes)
+                .into_iter()
+                .map(|ring_slot| {
+                    (
                         ring_slot,
-                    ),
+                        read_root_ring_slot_telling_never_written_apart(
+                            devices,
+                            &system_configuration.immutable.region_devices,
+                            &system_configuration.immutable.sizes,
+                            &system_configuration.immutable.filesystem_identifier,
+                            ring_slot,
+                        ),
+                    )
+                })
+                .collect()
+        };
+    // 窗口锚：这一遍读出的不被抛弃的自证有效根的最大 txg（下面算挂载时读坏、读回全零的槽能住哪几个 txg 用它）。
+    let newest_valid_root_txg_of_the_readings =
+        |readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)]| {
+            readings
+                .iter()
+                .filter_map(|(_, reading)| match reading {
+                    RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) => {
+                        (!abandoned_by_table(root, &newest_table)).then_some(root.checkpoint_txg)
+                    }
+                    RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
+                    | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
+                })
+                .max()
+        };
+    // 这一遍一条不被抛弃的自证有效根都没读出（择根那几遍读得出）就把根环整环再读一遍，根环表、回收门槛与影子账都用重读的读数；
+    // 重读还是一条都没有，就在任何写之前拒这次可写挂载（用户 2026-10-05 定「重读」，岔路单 `research/prompts/m3-batch17-code-r1-forks.md`
+    // 第 2 行；判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第二节「N1 甲二」那一行与第四节）。不重读、给空表往下走时，
+    // 写行那次记根算回收门槛只看得到它自己，释放代高于 F_生效 的一截当场放开、被暖机写掉，盘上环里更旧的有效根还引用着（D16（发布语义） 已定项 1
+    // 「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）。拒在取号之前，盘上逐字节不变。
+    let first_readings = read_every_root_ring_slot_once();
+    let (ring_slot_readings, window_anchor, root_ring_of_the_rebuilt_allocator) =
+        match newest_valid_root_txg_of_the_readings(&first_readings) {
+            Some(window_anchor) => (
+                first_readings,
+                window_anchor,
+                RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,
+            ),
+            None => {
+                let readings_of_the_reread = read_every_root_ring_slot_once();
+                let Some(window_anchor) =
+                    newest_valid_root_txg_of_the_readings(&readings_of_the_reread)
+                else {
+                    return Err(MountError::Recovery(RecoveryFailure::NoValidRoot));
+                };
+                (
+                    readings_of_the_reread,
+                    window_anchor,
+                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,
                 )
-            })
-            .collect();
+            }
+        };
     let roots_with_ring_slots: Vec<(RootRingSlot, RootRecord)> = ring_slot_readings
         .iter()
         .filter_map(|(ring_slot, reading)| match reading {
@@ -1915,35 +1970,25 @@ fn rebuilt_allocator<Device: BlockDevice>(
     // 挂载时读坏的槽能住哪几个 txg（[`txgs_ring_slots_bad_at_mount_can_hold`]）：N = 环里读得出的最新有效根的 txg（D16（发布语义） 已定项 1
     // 「挂载时读不出的槽」那一句：挂载重建分配器时取它、不取 first_txg − 1）。读回全零的槽按同一个算法另算一份、只取可能住过的最旧 txg：
     // 回收门槛里照「自证不过」那一类钉住（用户 2026-10-05 定乙），准入抬 F 的上限不并它（用户 2026-10-04 定），能住的最大 txg 用不到。
-    // 一个 txg 都住不了的槽（年轻的环里还没轮到的）两类都不交回。
-    let window_anchor = roots
-        .iter()
-        .filter(|root| !is_abandoned(root))
-        .map(|root| root.checkpoint_txg)
-        .max();
-    let ring_slots_bad_at_mount = window_anchor.map_or_else(Vec::new, |anchor| {
+    // 一个 txg 都住不了的槽（年轻的环里还没轮到的）两类都不交回。窗口锚上面取到了（取不到就重读、仍取不到就拒），两类不会因为它是空表。
+    let ring_slots_bad_at_mount = txgs_ring_slots_bad_at_mount_can_hold(
+        &bad_ring_slots_not_all_zero,
+        window_anchor,
+        first_txg,
+        slots_per_region,
+    );
+    let ring_slots_reading_back_all_zero_at_mount: Vec<(RootRingSlot, CheckpointTxg)> =
         txgs_ring_slots_bad_at_mount_can_hold(
-            &bad_ring_slots_not_all_zero,
-            anchor,
+            &ring_slots_reading_back_all_zero,
+            window_anchor,
             first_txg,
             slots_per_region,
         )
-    });
-    let ring_slots_reading_back_all_zero_at_mount: Vec<(RootRingSlot, CheckpointTxg)> =
-        window_anchor
-            .map_or_else(Vec::new, |anchor| {
-                txgs_ring_slots_bad_at_mount_can_hold(
-                    &ring_slots_reading_back_all_zero,
-                    anchor,
-                    first_txg,
-                    slots_per_region,
-                )
-            })
-            .into_iter()
-            .map(|(ring_slot, txgs_it_can_hold)| {
-                (ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held())
-            })
-            .collect();
+        .into_iter()
+        .map(|(ring_slot, txgs_it_can_hold)| {
+            (ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held())
+        })
+        .collect();
     // 环里有效根的最小 txg（D16（发布语义） 已定项 1「环里最旧有效根」，用户 2026-10-04 定 RGall：不按一圈窗口滤）；
     // 挂载时读坏的槽钉在它可能住过的最旧 txg（用户 2026-10-04 定），读回全零的槽同样钉着（用户 2026-10-05 定乙）。
     let oldest_valid_root_in_the_ring = roots
@@ -1970,7 +2015,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
     // 某块盘一条有效根都读不出（m 取不到）时当场只放到一圈窗口里最旧有效根（用户 2026-10-04 选甲，`research/prompts/m3-batch16-code-r1-forks.md`
     // 第 1 行；判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「再挂期间某块盘根环整段读坏」那一行），RGall 之后这个量是
     // 环里最旧有效根：那块盘上真正最新的根这一遍看不到，它引用的单元释放代可以高到 F_生效；它与回收门槛同一个读数（挂载时读坏、读回全零的槽钉在它可能住过的最旧 txg 并进）。
-    // 这一遍一条有效根都没读出、环里最旧有效根也取不到时，「只到环里最旧有效根」的那一截是空的：当场一个都不放，整截扣到暖机落满。
+    // 这一遍（或它的重读）至少读出了一条有效根（上面取窗口锚那一步：两遍都读不出就拒），环里最旧有效根恒取得到。
     let device_of_txg = |txg: CheckpointTxg| {
         let target = target_for_publish(txg, slots_per_region);
         system_configuration.immutable.region_devices
@@ -2071,6 +2116,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
         effective_floor,
         abandoned_roots_unreadable,
         instance_table_of_the_newest_root,
+        root_ring_of_the_rebuilt_allocator,
     })
 }
 
@@ -5719,6 +5765,7 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
         effective_floor,
         abandoned_roots_unreadable,
         instance_table_of_the_newest_root,
+        root_ring_of_the_rebuilt_allocator,
     } = rebuilt?;
     allocator.set_deletion_switches(deletion_switches);
     let previous_row = PreviousInstanceRow {
@@ -5748,6 +5795,7 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
             rereads: RereadsOfThisMount {
                 read_stage: read_stage_settled,
                 instance_table_of_the_newest_root,
+                root_ring_of_the_rebuilt_allocator,
             },
             instance_table_sweep,
         },
diff --git a/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs b/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
index 3d16c34..701abb9 100644
--- a/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
+++ b/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
@@ -30,8 +30,8 @@ use singlefs_core::mount::{
     mount_writable, mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
     BeforeTheOneReread, InstanceRow, InstanceTableOfTheNewestRootRead, MountError,
     NewerPublishWitness, ReadStageSettled, RereadsOfThisMount, RollbackTarget,
-    SelectedVersionAgainstTheWitness, ShadowLedger, StillUnreadableAfterOneReread,
-    WitnessedCounterComparison,
+    RootRingOfTheRebuiltAllocatorRead, SelectedVersionAgainstTheWitness, ShadowLedger,
+    StillUnreadableAfterOneReread, WitnessedCounterComparison,
 };
 use singlefs_core::mounted_read::mount_read_only;
 use singlefs_core::pointer::NodePointer;
@@ -294,6 +294,7 @@ fn mount_with_the_newest_root_unreadable_until_the_one_reread(
                 reread: selected_against_its_last_record(&newest, newest.record.counter),
             },
             instance_table_of_the_newest_root: InstanceTableOfTheNewestRootRead::OnTheFirstRead,
+            root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,
         },
         "{read_back:?}：第一遍落到 B、判据为真；重读落到 C、判据为假，往下走用重读那一遍"
     );
@@ -512,6 +513,7 @@ fn the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_n
                 },
             },
             instance_table_of_the_newest_root: InstanceTableOfTheNewestRootRead::OnTheFirstRead,
+            root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,
         }
     );
 }
diff --git a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
new file mode 100644
index 0000000..ca72aff
--- /dev/null
+++ b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
@@ -0,0 +1,795 @@
+//! 重建分配器读根环那一遍一条不被抛弃的自证有效根都没读出（择根那几遍读得出）时，把根环整环再读一遍，根环表、回收门槛与影子账用重读的读数；
+//! 重读还是一条都没有，就在任何写之前拒这次可写挂载（`MountError::Recovery(RecoveryFailure::NoValidRoot)`）。用户 2026-10-05 定「重读」：
+//! 岔路单 `research/prompts/m3-batch17-code-r1-forks.md` 第 2 行，判决 `research/prompts/m3-batch17-code-r1-main-verification.md`
+//! 第二节「N1 甲二」那一行与第四节。不重读、给空表往下走时，写行那次记根算回收门槛只看得到它自己，释放代高于 F_生效 的一截当场放开、被暖机写掉，
+//! 盘上环里更旧的有效根还引用着（攻方 `research/prompts/m3-batch17-code-r1-opus-output.md` 第 1.3 节）。
+//!
+//! 历史照攻方那一段（`264:6:4:0` n = 15，同 `the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` 第三条用例的起点与卸载刀）：
+//! 不补环窗口、准入关着造的窄盘（单元区 264 槽），第一个文件之后覆盖写 6 次、顺序写长到 4 个单元，在这个进程里发 unlink 的意图就崩；
+//! 重开可写（准入判着、影子账开、批大小 1）续做走完；卸载落下 15 个写就崩；两块盘都在、再可写挂载。
+//! 读故障由这里的 [`DeviceFailingPassesOfTheRootRing`] 造：根环每个槽按它自己被读的次数从 1 数，点名的那几次读报块设备错（盘上内容不动）。
+//! 一次不注错的再挂里根环整遍读若干遍、每遍每槽读一次，第 6 遍（从 1 数）是重建分配器那一遍（攻方用回溯栈认过：第 0 遍择根……第 5 遍
+//! 重建分配器，从 0 数）；这一认法由 [`RereadsOfThisMount::root_ring_of_the_rebuilt_allocator`] 现核：只坏那一遍时它报重读过。
+
+mod common_admission;
+
+use common_admission::{
+    checker_violations_on, plain_devices_on,
+    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
+    PoolUnderTest, OVERWRITE_BYTES,
+};
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
+};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::{
+    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+};
+use singlefs_core::container_tree::ContainerTreeNodeEntries;
+use singlefs_core::lineage_tree::LineageContainerCapacity;
+use singlefs_core::mount::{
+    mount_writable_with_test_only_switches_and_deletion_switches, unmount,
+    InstanceTableOfTheNewestRootRead, MountError, Mounted, RereadsOfThisMount,
+    RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
+};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::mounted_session::MountedSession;
+use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
+use singlefs_core::recovery::RecoveryFailure;
+use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
+use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
+use singlefs_core::transaction::{publish_unlink_intent, PoolWriter, FIRST_INODE_NUMBER};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_core::unlink::{
+    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
+};
+use singlefs_harness::fault_injection::{
+    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
+    FaultSchedule, InjectedFault, NamedRootRingSlots, RootRingSlotTarget, SharedFaultPlan,
+};
+use singlefs_harness::history::HistoryDeviceWidth;
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use std::cell::RefCell;
+use std::collections::BTreeMap;
+use std::ops::RangeInclusive;
+use std::rc::Rc;
+
+/// 一次不注错的再挂里，根环每个槽第几次被读是重建分配器读根环那一遍（从 1 数；见文件头）。
+const READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD: u64 = 6;
+
+/// 这段历史上一次不注错的再挂里根环每个槽被读的次数（攻方量过不注错的再挂整遍读 15 遍；这里由不注错那条用例现量、钉住）。
+const READS_OF_EACH_ROOT_RING_SLOT_BY_A_REMOUNT_WITHOUT_FAULTS: u64 = 15;
+
+/// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
+fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
+    DeletionSwitches {
+        batch_size: UnlinkBatchSize::CappedForTests(
+            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
+        ),
+        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
+        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+    }
+}
+
+/// 根环每个槽被读的次数与哪几次读坏（几块盘共用一份）。
+struct RootRingReadsState {
+    target: RootRingSlotTarget,
+    /// 每个槽按它自己被读的次数从 1 数，落在这个区间里的那几次读报块设备错；空区间是一次都不坏。
+    failing_reads_of_each_slot: RangeInclusive<u64>,
+    reads_of_each_slot: BTreeMap<RootRingSlot, u64>,
+    failed_reads: u64,
+}
+
+#[derive(Clone)]
+struct SharedRootRingReads(Rc<RefCell<RootRingReadsState>>);
+
+impl SharedRootRingReads {
+    fn new(target: RootRingSlotTarget, failing_reads_of_each_slot: RangeInclusive<u64>) -> Self {
+        Self(Rc::new(RefCell::new(RootRingReadsState {
+            target,
+            failing_reads_of_each_slot,
+            reads_of_each_slot: BTreeMap::new(),
+            failed_reads: 0,
+        })))
+    }
+
+    /// 这一次读落在哪个根环槽里就给那个槽记一次，交回这一次要不要读坏。
+    fn note_a_read_and_tell_whether_it_fails(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+    ) -> bool {
+        let mut state = self.0.borrow_mut();
+        let Some(ring_slot) = state.target.slot_covering(device, offset) else {
+            return false;
+        };
+        let reads_of_this_slot = {
+            let reads = state.reads_of_each_slot.entry(ring_slot).or_insert(0);
+            *reads += 1;
+            *reads
+        };
+        let fails_this_time = state
+            .failing_reads_of_each_slot
+            .contains(&reads_of_this_slot);
+        if fails_this_time {
+            state.failed_reads += 1;
+        }
+        fails_this_time
+    }
+
+    /// 根环每个槽被读了几次（没被读过的槽不在表里）。
+    fn reads_of_each_slot(&self) -> BTreeMap<RootRingSlot, u64> {
+        self.0.borrow().reads_of_each_slot.clone()
+    }
+
+    fn failed_reads(&self) -> u64 {
+        self.0.borrow().failed_reads
+    }
+}
+
+/// 读的时候按 [`SharedRootRingReads`] 让根环槽的某几次读报块设备错的盘；写、写零、屏障原样交给里面那块盘。
+struct DeviceFailingPassesOfTheRootRing<Inner: BlockDevice> {
+    identity: DeviceIdentity,
+    inner: Inner,
+    root_ring_reads: SharedRootRingReads,
+}
+
+impl<Inner: BlockDevice> BlockDevice for DeviceFailingPassesOfTheRootRing<Inner> {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        if self
+            .root_ring_reads
+            .note_a_read_and_tell_whether_it_fails(self.identity, offset)
+        {
+            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
+                "用例点名的这一次根环槽读坏",
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
+/// 再挂那一次根环哪几遍读坏（两块盘全部槽）。
+#[derive(Clone, Copy, Debug)]
+enum RootRingPassesFailingDuringTheRemount {
+    NoneOfThePasses,
+    OnlyTheAllocatorRebuild,
+    TheAllocatorRebuildAndItsReread,
+}
+
+impl RootRingPassesFailingDuringTheRemount {
+    fn failing_reads_of_each_slot(self) -> RangeInclusive<u64> {
+        let rebuild = READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD;
+        match self {
+            // 空区间：一次都不坏，只数。
+            RootRingPassesFailingDuringTheRemount::NoneOfThePasses => RangeInclusive::new(1, 0),
+            RootRingPassesFailingDuringTheRemount::OnlyTheAllocatorRebuild => rebuild..=rebuild,
+            RootRingPassesFailingDuringTheRemount::TheAllocatorRebuildAndItsReread => {
+                rebuild..=rebuild + 1
+            }
+        }
+    }
+}
+
+/// 卸载半途崩溃之后的那份盘面，与卸载之前的盘面、这个池的参数（再挂与读回要用）。
+struct NarrowPoolAfterACrashedUnmount {
+    pool: PoolUnderTest<SparseBlockDevice>,
+    image_before_the_unmount: MemoryPool,
+    image_after_the_crashed_unmount: MemoryPool,
+}
+
+impl NarrowPoolAfterACrashedUnmount {
+    /// 两块盘全部根环槽。
+    fn every_root_ring_slot(&self) -> RootRingSlotTarget {
+        RootRingSlotTarget {
+            named_slots: NamedRootRingSlots::every_slot_in_the_ring(
+                self.pool.parameters.geometry.root_ring_slots_per_region,
+            ),
+            region_devices: self.pool.parameters.region_devices,
+            fixed_structure_slot_spacing: self
+                .pool
+                .parameters
+                .geometry
+                .fixed_structure_slot_spacing,
+        }
+    }
+
+    fn root_ring_slot_count(&self) -> u64 {
+        u64::from(self.every_root_ring_slot().named_slots.count())
+    }
+
+    /// 卸载半途崩溃之后的盘面拷一份、包上根环读故障。
+    fn devices_failing<Inner: BlockDevice>(
+        &self,
+        root_ring_reads: &SharedRootRingReads,
+        wrap: impl Fn(DeviceIdentity, SparseBlockDevice) -> Inner,
+    ) -> Vec<(DeviceIdentity, DeviceFailingPassesOfTheRootRing<Inner>)> {
+        plain_devices_on(&self.image_after_the_crashed_unmount)
+            .into_iter()
+            .map(|(identity, device)| {
+                (
+                    identity,
+                    DeviceFailingPassesOfTheRootRing {
+                        identity,
+                        inner: wrap(identity, device),
+                        root_ring_reads: root_ring_reads.clone(),
+                    },
+                )
+            })
+            .collect()
+    }
+
+    /// 再可写挂载（准入判着、影子账开、批大小 1）。
+    fn mount_writable<Device: BlockDevice>(
+        &self,
+        devices: &mut Vec<(DeviceIdentity, Device)>,
+    ) -> Result<Mounted, MountError> {
+        mount_writable_with_test_only_switches_and_deletion_switches(
+            &self.pool.parameters,
+            devices,
+            SpaceAdmission::JudgedByTheFormula,
+            ShadowLedger::On,
+            deletion_switches_with_batches_of_one(),
+        )
+    }
+
+    /// 卸载半途崩溃之后的盘面上再可写挂载一次，根环照 `passes_failing` 读坏；交回挂载的结局、挂载之后的盘面与根环读数。
+    fn remount(
+        &self,
+        passes_failing: RootRingPassesFailingDuringTheRemount,
+    ) -> (Result<Mounted, MountError>, MemoryPool, SharedRootRingReads) {
+        let root_ring_reads = SharedRootRingReads::new(
+            self.every_root_ring_slot(),
+            passes_failing.failing_reads_of_each_slot(),
+        );
+        let (mounted, image) = self.remount_reading_the_root_ring_through(&root_ring_reads);
+        (mounted, image, root_ring_reads)
+    }
+
+    /// `device` 那几个区域的每个根环槽。
+    fn root_ring_slots_of(&self, device: DeviceIdentity) -> RootRingSlotTarget {
+        let every_slot = self.every_root_ring_slot();
+        let ring_slots_of_the_device: Vec<RootRingSlot> = every_slot
+            .named_slots
+            .named_slots()
+            .into_iter()
+            .filter(|ring_slot| {
+                self.pool.parameters.region_devices
+                    [usize::try_from(ring_slot.region).expect("区域号")]
+                    == device
+            })
+            .collect();
+        RootRingSlotTarget {
+            named_slots: NamedRootRingSlots::naming(&ring_slots_of_the_device),
+            region_devices: every_slot.region_devices,
+            fixed_structure_slot_spacing: every_slot.fixed_structure_slot_spacing,
+        }
+    }
+
+    /// 卸载半途崩溃之后的盘面上再可写挂载一次，根环照 `root_ring_reads` 读坏；交回挂载的结局与挂载之后的盘面。
+    fn remount_reading_the_root_ring_through(
+        &self,
+        root_ring_reads: &SharedRootRingReads,
+    ) -> (Result<Mounted, MountError>, MemoryPool) {
+        let mut devices = self.devices_failing(root_ring_reads, |_, device| device);
+        let mounted = self.mount_writable(&mut devices);
+        let image = MemoryPool {
+            devices: devices
+                .iter()
+                .map(|(identity, device)| (*identity, device.inner.image.clone()))
+                .collect(),
+            device_size_in_bytes: self.image_after_the_crashed_unmount.device_size_in_bytes,
+        };
+        (mounted, image)
+    }
+}
+
+/// 整池从第 `first_failing_write` 次写起每次写都报错的注错计划（掉电那一形：盘停在之前那几个写之后）。
+fn plan_failing_every_write_from(
+    device_width: HistoryDeviceWidth,
+    first_failing_write: u64,
+) -> SharedFaultPlan {
+    SharedFaultPlan::armed(
+        device_width.fixed_geometry(),
+        FaultSchedule {
+            fault: InjectedFault::WriteFails,
+            device: FaultDeviceSelector::EveryDevice,
+            placement: FaultPlacement::AnyOffset,
+            counting: FaultCounting::AcrossThePool,
+            occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(first_failing_write),
+        },
+    )
+}
+
+/// 攻方那一段的起点（见文件头）：卸载落下 15 个写就崩（两块盘的系统配置都带了新 F、带新 F 的根一条都没落）。
+fn narrow_pool_after_a_crashed_unmount() -> NarrowPoolAfterACrashedUnmount {
+    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
+    let overwrites_before_the_file_grows = 6;
+    let data_units_of_the_file = 4;
+    let unmount_writes_landed = 15;
+    let mut pool =
+        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+            device_width,
+            264,
+        );
+    for overwrite_index in 0..overwrites_before_the_file_grows {
+        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
+        });
+    }
+    pool.sequential_write((data_units_of_the_file - 1) * data_unit_payload_capacity() + 100)
+        .unwrap_or_else(|refusal| panic!("顺序写把文件长到 4 个单元：{refusal:?}"));
+    {
+        let PoolUnderTest {
+            session,
+            devices,
+            parameters,
+            write_time_seconds,
+            ..
+        } = &mut pool;
+        let session = session.as_mut().expect("这一刻有可写会话");
+        let current = session
+            .current
+            .version_of_trees()
+            .expect("第一个文件之后有树")
+            .clone();
+        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
+        publish_unlink_intent(
+            &mut writer,
+            &mut session.allocator,
+            &current,
+            InodeNumber(FIRST_INODE_NUMBER),
+            *write_time_seconds,
+            session.instance,
+        )
+        .expect("意图那次");
+    }
+    pool.session = None;
+    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
+        &pool.parameters,
+        &mut pool.devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        deletion_switches_with_batches_of_one(),
+    )
+    .unwrap_or_else(|error| panic!("崩了之后重开可写、续做：{error:?}"));
+    let (_, session_of_the_continued_mount) = MountedSession::of_the_mount(mounted);
+    pool.session = Some(session_of_the_continued_mount);
+    let image_before_the_unmount = pool.image();
+    let write_plan = plan_failing_every_write_from(device_width, unmount_writes_landed + 1);
+    let mut unmount_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
+        plain_devices_on(&image_before_the_unmount)
+            .into_iter()
+            .map(|(identity, device)| {
+                (
+                    identity,
+                    FaultInjectingBlockDevice::new(identity, device, write_plan.clone()),
+                )
+            })
+            .collect();
+    let image_after_the_crashed_unmount = {
+        let session = pool.session();
+        let mut allocator = session.allocator.clone();
+        let mut current = session.current.clone();
+        let unmounted = unmount(
+            &pool.parameters,
+            &mut unmount_devices,
+            &mut allocator,
+            &mut current,
+            session.shadow_ledger,
+        );
+        assert!(
+            unmounted.is_err(),
+            "卸载在第 {} 个写上掉电、报错：{:?}",
+            unmount_writes_landed + 1,
+            unmounted.as_ref().ok().map(|_| "做成了")
+        );
+        MemoryPool {
+            devices: unmount_devices
+                .iter()
+                .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
+                .collect(),
+            device_size_in_bytes: image_before_the_unmount.device_size_in_bytes,
+        }
+    };
+    NarrowPoolAfterACrashedUnmount {
+        pool,
+        image_before_the_unmount,
+        image_after_the_crashed_unmount,
+    }
+}
+
+/// 只读挂载、把挂载态里每个活着的文件从头读到尾：走到的根 txg 与每个活文件（inode 号、全部字节）。删除中的 inode（nlink = 0）不读。
+#[derive(Debug, PartialEq, Eq)]
+struct WholeReadOfTheLiveFiles {
+    root_txg: CheckpointTxg,
+    live_files: Vec<(u64, Vec<u8>)>,
+}
+
+/// 只读挂载与整读每个活文件；哪一步读不出就交回那一步的错（只读挂载、打开、读）。
+fn whole_read_of_the_live_files(image: &MemoryPool) -> Result<WholeReadOfTheLiveFiles, String> {
+    let read_only = mount_read_only(image).map_err(|failure| format!("只读挂载：{failure:?}"))?;
+    let mut live_files = Vec::new();
+    // 迭代上界是这一版 inode 树的记录数；提前出口只有读不出的那三种错。
+    for record in read_only.mounted.inode_records() {
+        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
+            continue;
+        }
+        let file = read_only
+            .mounted
+            .open_file(image, InodeNumber(record.inode))
+            .map_err(|failure| format!("打开 inode {}：{failure:?}", record.inode))?;
+        let read = file
+            .read_at(image, FileOffsetInBytes(0), record.size)
+            .map_err(|failure| format!("读 inode {}：{failure:?}", record.inode))?;
+        live_files.push((record.inode, read.bytes));
+    }
+    Ok(WholeReadOfTheLiveFiles {
+        root_txg: read_only.effective_root.checkpoint_txg,
+        live_files,
+    })
+}
+
+/// 一份镜像去掉一块盘。
+fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
+    MemoryPool {
+        devices: image
+            .devices
+            .iter()
+            .filter(|(identity, _)| **identity != lost)
+            .map(|(identity, device)| (*identity, device.clone()))
+            .collect(),
+        device_size_in_bytes: image.device_size_in_bytes,
+    }
+}
+
+/// 只坏重建分配器那一遍（两块盘全部根环槽报读错）：重读那一遍读得出，挂载照常可写，交回与不注错的再挂同一份盘面（判定不变）；
+/// 两块盘都在时池级 checker 一条都不判违例（I-7.4「近 K 代块未被复用」在内）。
+/// 判别力：删掉重读（第一遍读不出就拒）时第一项红（`Recovery(NoValidRoot)`）；不重读、照旧给空表往下走时，写行那次记根把释放代高于 F_生效 的一截
+/// 当场放开、暖机写进盘上环里更旧的有效根（txg 20）还引用的单元，写行之后那次准入抬 F 读不出它的树表，挂载落了盘之后报
+/// `FloorRaiseFailedAfterTheMountsPublishes`（攻方第 1.3 节那一形；变异表里这一刀量过，`UnitUnreadable { slot: SlotNumber(1442) }`），第一项同样红。
+#[test]
+fn only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over(
+) {
+    let history = narrow_pool_after_a_crashed_unmount();
+    let (remounted, image_after_the_remount, root_ring_reads) =
+        history.remount(RootRingPassesFailingDuringTheRemount::OnlyTheAllocatorRebuild);
+    let mounted = remounted.unwrap_or_else(|error| {
+        panic!("只坏重建分配器那一遍：重读读得出，照常可写挂载：{error:?}")
+    });
+    assert_eq!(
+        checker_violations_on(&image_after_the_remount),
+        Vec::<String>::new(),
+        "只坏重建分配器那一遍的再挂之后两块盘都在：池级 checker 一条都不判违例（I-7.4 在内）"
+    );
+    let (remounted_without_faults, image_after_the_remount_without_faults, _) =
+        history.remount(RootRingPassesFailingDuringTheRemount::NoneOfThePasses);
+    remounted_without_faults
+        .unwrap_or_else(|error| panic!("同一份盘面不注错地再可写挂载：{error:?}"));
+    assert!(
+        image_after_the_remount == image_after_the_remount_without_faults,
+        "只坏重建分配器那一遍的再挂与不注错的再挂交回逐字节相同的盘面（重读的读数与不注错时相同，判定不变）"
+    );
+    assert_eq!(
+        mounted.output.rereads.root_ring_of_the_rebuilt_allocator,
+        RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,
+        "重建分配器读根环那一遍一条有效根都没读出、重读读出：读坏的确是重建分配器那一遍（第 {READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD} 次读每个槽）"
+    );
+    assert_eq!(
+        mounted.output.rereads.instance_table_of_the_newest_root,
+        InstanceTableOfTheNewestRootRead::OnTheFirstRead,
+        "最新那条根的实例表第一遍就读出（读坏的不是它那一遍）"
+    );
+    assert_eq!(
+        root_ring_reads.failed_reads(),
+        history.root_ring_slot_count(),
+        "读坏的恰是一整遍：每个根环槽一次"
+    );
+    let reads_of_each_slot = root_ring_reads.reads_of_each_slot();
+    assert!(
+        reads_of_each_slot.len() == usize::try_from(history.root_ring_slot_count()).expect("槽数")
+            && reads_of_each_slot.values().all(|reads| *reads
+                == READS_OF_EACH_ROOT_RING_SLOT_BY_A_REMOUNT_WITHOUT_FAULTS + 1),
+        "每个根环槽比不注错的再挂多读恰一次（重读那一遍）：{reads_of_each_slot:?}"
+    );
+}
+
+/// 重建分配器那一遍与它的重读都坏（两块盘全部根环槽报读错）：在任何写之前拒（`Recovery(NoValidRoot)`），盘面与挂载之前逐字节相同。
+/// 判别力：重读还读不出时不拒、照旧给空表往下写，挂载写了盘（写行、暖机）之后报 `FloorRaiseFailedAfterTheMountsPublishes`（变异表里这一刀量过），
+/// 第一项红。
+#[test]
+fn the_allocator_rebuild_pass_and_its_reread_both_failing_refuse_the_writable_mount_before_any_write(
+) {
+    let history = narrow_pool_after_a_crashed_unmount();
+    let (remounted, image_after_the_refusal, root_ring_reads) =
+        history.remount(RootRingPassesFailingDuringTheRemount::TheAllocatorRebuildAndItsReread);
+    assert!(
+        matches!(
+            remounted,
+            Err(MountError::Recovery(RecoveryFailure::NoValidRoot))
+        ),
+        "重建分配器那一遍与重读都一条有效根读不出：拒这次可写挂载（Recovery(NoValidRoot)）：{:?}",
+        remounted.as_ref().map(|_| "做成了")
+    );
+    assert!(
+        image_after_the_refusal == history.image_after_the_crashed_unmount,
+        "拒在任何写之前：两块盘与挂载之前逐字节相同"
+    );
+    assert_eq!(
+        root_ring_reads.failed_reads(),
+        2 * history.root_ring_slot_count(),
+        "读坏的恰是两整遍（重建那一遍与它的重读）：每个根环槽两次"
+    );
+    let reads_of_each_slot = root_ring_reads.reads_of_each_slot();
+    assert!(
+        reads_of_each_slot.len() == usize::try_from(history.root_ring_slot_count()).expect("槽数")
+            && reads_of_each_slot
+                .values()
+                .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD + 1),
+        "重读那一遍之后根环再没被读过（拒在重读之后、取号之前）：{reads_of_each_slot:?}"
+    );
+}
+
+/// 不注错的对照：同一份盘面再可写挂载，一次都不拒、不重读，根环每个槽读的次数与改之前相同（攻方量的 15 遍），两块盘都在时池级 checker 不判违例。
+#[test]
+fn a_remount_without_faults_does_not_reread_the_root_ring_and_reads_each_slot_as_often_as_before() {
+    let history = narrow_pool_after_a_crashed_unmount();
+    let (remounted, image_after_the_remount, root_ring_reads) =
+        history.remount(RootRingPassesFailingDuringTheRemount::NoneOfThePasses);
+    let mounted = remounted.unwrap_or_else(|error| panic!("不注错的再挂照常可写：{error:?}"));
+    let RereadsOfThisMount {
+        read_stage: _,
+        instance_table_of_the_newest_root,
+        root_ring_of_the_rebuilt_allocator,
+    } = mounted.output.rereads;
+    assert_eq!(
+        (
+            instance_table_of_the_newest_root,
+            root_ring_of_the_rebuilt_allocator
+        ),
+        (
+            InstanceTableOfTheNewestRootRead::OnTheFirstRead,
+            RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead
+        ),
+        "不注错时实例表与重建分配器读根环都第一遍就读出"
+    );
+    assert_eq!(root_ring_reads.failed_reads(), 0, "不注错：一次都没读坏");
+    let reads_of_each_slot = root_ring_reads.reads_of_each_slot();
+    assert!(
+        reads_of_each_slot.len() == usize::try_from(history.root_ring_slot_count()).expect("槽数")
+            && reads_of_each_slot
+                .values()
+                .all(|reads| *reads == READS_OF_EACH_ROOT_RING_SLOT_BY_A_REMOUNT_WITHOUT_FAULTS),
+        "不注错的再挂里每个根环槽读 {READS_OF_EACH_ROOT_RING_SLOT_BY_A_REMOUNT_WITHOUT_FAULTS} 次（没有多出的重读）：{reads_of_each_slot:?}"
+    );
+    assert_eq!(
+        checker_violations_on(&image_after_the_remount),
+        Vec::<String>::new(),
+        "不注错的再挂之后两块盘都在：池级 checker 一条都不判违例"
+    );
+}
+
+/// 攻方第 1.3 节那一段具体的历史：只坏重建分配器那一遍的再挂，写行的根落在盘 1、暖机写了单元，暖机落到盘 0 的那条根写下去之前掉电；丢盘 1。
+/// 剩下的盘 0 只读挂载、整读每个活文件，与卸载之前盘 0 上读出的相同（走到的根 txg 与内容）。
+/// 判别力：不重读、照旧给空表往下走时，暖机的单元写进盘 0 上最新的根还引用的槽（攻方量过 `UnitUnreadable { slot: SlotNumber(1441) }`），最后一项红。
+#[test]
+fn only_the_allocator_rebuild_pass_failing_and_a_crash_before_the_warm_up_root_lands_on_device_zero_keep_what_device_zero_reads(
+) {
+    let history = narrow_pool_after_a_crashed_unmount();
+    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
+    let slots_per_region = history.pool.parameters.geometry.root_ring_slots_per_region;
+    let device_of_the_root_of = |txg: CheckpointTxg| {
+        history.pool.parameters.region_devices
+            [usize::try_from(target_for_publish(txg, slots_per_region).region).expect("区域号")]
+    };
+    // 先在一份拷贝上不注错地整做一次再挂，认出写行与暖机各条根的 txg、暖机里第一条落在盘 0 上的根（这几次发布的 txg 只由新实例第一次发布的 txg
+    // 与发布的先后定，读不读坏根环都一样；不拿读坏的那一次认：不重读时那一次在暖机之后报错，交不回暖机的根）。
+    let (whole_remount, _, _) =
+        history.remount(RootRingPassesFailingDuringTheRemount::NoneOfThePasses);
+    let whole_remount =
+        whole_remount.unwrap_or_else(|error| panic!("不注错的再挂照常可写：{error:?}"));
+    let row_txg = whole_remount.output.row_publish.root().checkpoint_txg;
+    assert_eq!(
+        device_of_the_root_of(row_txg),
+        DeviceIdentity(1),
+        "写行那次（txg {row_txg:?}）的根落在盘 1（攻方那一段的样子）"
+    );
+    let warm_up_txg_on_device_zero = whole_remount
+        .output
+        .warm_up_publishes
+        .iter()
+        .map(|version| version.root().checkpoint_txg)
+        .find(|txg| device_of_the_root_of(*txg) == DeviceIdentity(0))
+        .expect("暖机在每块盘上都落一条根：有一条落在盘 0");
+    let root_slot_offset_on_device_zero = slot_offset(
+        target_for_publish(warm_up_txg_on_device_zero, slots_per_region),
+        history
+            .pool
+            .parameters
+            .geometry
+            .fixed_structure_slot_spacing,
+    );
+    // 再在一份拷贝上同样读坏地再挂，只让那条根的写报错，认出它是整池第几次写。
+    let write_ordinal_of_the_warm_up_root_on_device_zero = {
+        let root_write_plan = SharedFaultPlan::armed(
+            device_width.fixed_geometry(),
+            FaultSchedule {
+                fault: InjectedFault::WriteFails,
+                device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(0)),
+                placement: FaultPlacement::OffsetExactly(root_slot_offset_on_device_zero),
+                counting: FaultCounting::AcrossThePool,
+                occurrence: FaultOccurrence::TheNthMatchingCall(1),
+            },
+        );
+        let root_ring_reads = SharedRootRingReads::new(
+            history.every_root_ring_slot(),
+            RootRingPassesFailingDuringTheRemount::OnlyTheAllocatorRebuild
+                .failing_reads_of_each_slot(),
+        );
+        let mut devices = history.devices_failing(&root_ring_reads, |identity, device| {
+            FaultInjectingBlockDevice::new(identity, device, root_write_plan.clone())
+        });
+        let remounted = history.mount_writable(&mut devices);
+        assert!(
+            remounted.is_err(),
+            "暖机落到盘 0 的那条根（txg {warm_up_txg_on_device_zero:?}）写报错：挂载报错"
+        );
+        let fired = root_write_plan.fired();
+        assert_eq!(fired.len(), 1, "那条根的写恰被拦下一次：{fired:?}");
+        fired[0].call_ordinal_across_the_pool
+    };
+    // 真那一次：从那条根的写起每次写都报错（盘停在它之前的写之后），丢盘 1。
+    let crash_plan = plan_failing_every_write_from(
+        device_width,
+        write_ordinal_of_the_warm_up_root_on_device_zero,
+    );
+    let root_ring_reads = SharedRootRingReads::new(
+        history.every_root_ring_slot(),
+        RootRingPassesFailingDuringTheRemount::OnlyTheAllocatorRebuild.failing_reads_of_each_slot(),
+    );
+    let mut devices = history.devices_failing(&root_ring_reads, |identity, device| {
+        FaultInjectingBlockDevice::new(identity, device, crash_plan.clone())
+    });
+    let remounted = history.mount_writable(&mut devices);
+    assert!(
+        remounted.is_err(),
+        "暖机落到盘 0 的那条根写下去之前掉电：挂载报错"
+    );
+    drop(remounted);
+    let image_after_the_crash = MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.inner.wrapped_device().image.clone()))
+            .collect(),
+        device_size_in_bytes: history.image_after_the_crashed_unmount.device_size_in_bytes,
+    };
+    let before_the_unmount = whole_read_of_the_live_files(&image_without(
+        &history.image_before_the_unmount,
+        DeviceIdentity(1),
+    ));
+    assert!(
+        before_the_unmount.is_ok(),
+        "卸载之前盘 0 只读挂载、整读每个活文件做得成：{before_the_unmount:?}"
+    );
+    assert_eq!(
+        whole_read_of_the_live_files(&image_without(&image_after_the_crash, DeviceIdentity(1))),
+        before_the_unmount,
+        "丢盘 1 之后剩下的盘 0 走到的根 txg 与每个活文件的内容与卸载之前盘 0 上读出的相同：再挂那一次当场放开的槽没有一个被盘 0 上最新的根引用"
+    );
+}
+
+/// 重读那一遍只有一部分槽读坏（交主 agent 的那一格，能造的一形）：重建分配器那一遍两块盘全部根环槽报读错，重读那一遍盘 1 的根环槽仍报读错、
+/// 盘 0 的读得出。重读的读数与「重建那一遍本身只坏盘 1」同等对待：挂载做成、交回的盘面与只坏盘 1 那一遍的再挂（不重读）逐字节相同，
+/// 两块盘都在时池级 checker 不判违例。
+/// 判别力：删掉重读时第一项红（`Recovery(NoValidRoot)`）；不重读、照旧给空表往下走时同上一形，挂载落了盘之后报
+/// `FloorRaiseFailedAfterTheMountsPublishes`，第一项红。
+#[test]
+fn a_reread_still_failing_on_device_one_hands_over_what_a_rebuild_pass_failing_only_on_device_one_hands_over(
+) {
+    let history = narrow_pool_after_a_crashed_unmount();
+    let rebuild = READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD;
+    // 外层数每一次根环读、只坏重建那一遍；里层只数外层放过去的读，它的第 6 次是外层的第 7 次（重读那一遍），只坏盘 1 的槽。
+    let reads_of_every_slot =
+        SharedRootRingReads::new(history.every_root_ring_slot(), rebuild..=rebuild);
+    let reads_of_device_one_let_through = SharedRootRingReads::new(
+        history.root_ring_slots_of(DeviceIdentity(1)),
+        rebuild..=rebuild,
+    );
+    let mut devices = history.devices_failing(&reads_of_every_slot, |identity, device| {
+        DeviceFailingPassesOfTheRootRing {
+            identity,
+            inner: device,
+            root_ring_reads: reads_of_device_one_let_through.clone(),
+        }
+    });
+    let remounted = history.mount_writable(&mut devices);
+    let mounted = remounted
+        .unwrap_or_else(|error| panic!("重读那一遍读得出盘 0 的根：照常可写挂载：{error:?}"));
+    let image_after_the_remount = MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.inner.inner.image.clone()))
+            .collect(),
+        device_size_in_bytes: history.image_after_the_crashed_unmount.device_size_in_bytes,
+    };
+    let reads_failing_only_on_device_one = SharedRootRingReads::new(
+        history.root_ring_slots_of(DeviceIdentity(1)),
+        rebuild..=rebuild,
+    );
+    let (remounted_failing_only_on_device_one, image_after_the_remount_failing_only_on_device_one) =
+        history.remount_reading_the_root_ring_through(&reads_failing_only_on_device_one);
+    let mounted_failing_only_on_device_one = remounted_failing_only_on_device_one
+        .unwrap_or_else(|error| panic!("重建那一遍只坏盘 1：照常可写挂载：{error:?}"));
+    assert!(
+        image_after_the_remount == image_after_the_remount_failing_only_on_device_one,
+        "重读那一遍只坏盘 1 的再挂与重建那一遍只坏盘 1 的再挂交回逐字节相同的盘面（重读的读数与第一遍的读数同等对待）"
+    );
+    assert_eq!(
+        (
+            mounted.output.rereads.root_ring_of_the_rebuilt_allocator,
+            mounted_failing_only_on_device_one
+                .output
+                .rereads
+                .root_ring_of_the_rebuilt_allocator
+        ),
+        (
+            RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,
+            RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead
+        ),
+        "前一次重读过、后一次第一遍就读出盘 0 的根"
+    );
+    assert_eq!(
+        (
+            reads_of_every_slot.failed_reads(),
+            reads_of_device_one_let_through.failed_reads()
+        ),
+        (
+            history.root_ring_slot_count(),
+            u64::from(
+                history
+                    .root_ring_slots_of(DeviceIdentity(1))
+                    .named_slots
+                    .count()
+            )
+        ),
+        "重建那一遍每个槽坏一次，重读那一遍盘 1 的每个槽坏一次"
+    );
+    assert_eq!(
+        checker_violations_on(&image_after_the_remount),
+        Vec::<String>::new(),
+        "两块盘都在：池级 checker 一条都不判违例"
+    );
+}
````

### mutations-append.tsv

````
第10e批 重建分配器读根环那一遍一条有效根都没读出时不重读、当场拒（删掉重读那一遍）	crates/singlefs-core/src/mount.rs	            None => {\n                let readings_of_the_reread = read_every_root_ring_slot_once();\n                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_reread)\n                else {\n                    return Err(MountError::Recovery(RecoveryFailure::NoValidRoot));\n                };\n                (\n                    readings_of_the_reread,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )\n            }	            None => return Err(MountError::Recovery(RecoveryFailure::NoValidRoot)),	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over
第10e批 重读那一遍还读不出有效根时不拒、照旧给空表往下写	crates/singlefs-core/src/mount.rs	                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_reread)\n                else {\n                    return Err(MountError::Recovery(RecoveryFailure::NoValidRoot));\n                };\n                (\n                    readings_of_the_reread,	                let (readings_of_the_reread, window_anchor) =\n                    match newest_valid_root_txg_of_the_readings(&readings_of_the_reread) {\n                        Some(window_anchor) => (readings_of_the_reread, window_anchor),\n                        None => (Vec::new(), CheckpointTxg(0)),\n                    };\n                (\n                    readings_of_the_reread,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	the_allocator_rebuild_pass_and_its_reread_both_failing_refuse_the_writable_mount_before_any_write
第10e批 重建分配器读根环那一遍一条有效根都没读出时照第 17 批那样不重读、给空表往下走（攻方 1.3 甲二：暖机写进盘 0 上最新的根还引用的槽）	crates/singlefs-core/src/mount.rs	            None => {\n                let readings_of_the_reread = read_every_root_ring_slot_once();\n                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_reread)\n                else {\n                    return Err(MountError::Recovery(RecoveryFailure::NoValidRoot));\n                };\n                (\n                    readings_of_the_reread,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )\n            }	            None => (\n                Vec::new(),\n                CheckpointTxg(0),\n                RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,\n            ),	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	only_the_allocator_rebuild_pass_failing_and_a_crash_before_the_warm_up_root_lands_on_device_zero_keep_what_device_zero_reads
第10e批 重读那一遍读出了有效根却报第一遍就读出（MountOutput::rereads 看不出重读过）	crates/singlefs-core/src/mount.rs	                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over
````

### mutations-replacements.tsv

````
C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）	crates/singlefs-core/src/mount.rs	    establish_instance(\n        &parameters_of_the_pool,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger,\n            system_configuration,\n            space_admission,\n            selected_version_journal_position,\n            rereads: RereadsOfThisMount {\n                read_stage: read_stage_settled,\n                instance_table_of_the_newest_root,\n                root_ring_of_the_rebuilt_allocator,\n            },\n            instance_table_sweep,\n        },\n    )\n}\n	    let instance_before_acquisition = previous_row.instance;\n    let outcome = establish_instance(\n        &parameters_of_the_pool,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger,\n            system_configuration,\n            space_admission,\n            selected_version_journal_position,\n            rereads: RereadsOfThisMount {\n                read_stage: read_stage_settled,\n                instance_table_of_the_newest_root,\n                root_ring_of_the_rebuilt_allocator,\n            },\n            instance_table_sweep,\n        },\n    );\n    if let Err(MountError::Publish(_)) = &outcome {\n        let mut rollback_writer = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());\n        let _ = rollback_writer.perform(\n            crate::transaction::CommitStep::RotateSystemConfigurationSlots {\n                journal_tail: 0,\n                journal_instance: instance_before_acquisition,\n            },\n        );\n    }\n    outcome\n}\n	-p singlefs-harness --test fault_injection_fast_tier -- write_error_after_the_acquisition	write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it
````

## batch10f-reread-hook

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10f-reread-hook/patch/`
- `crates.patch`：sha256 ae6f514afda0b2643da7cc3a7c263e1d03d0eb16da64c21985a4800fb068abf7
- `mutations-append.tsv`：sha256 20a3ce55fd1497752ede32cc8aeef89f05a893fd3a3ee6ab54411b439f4da00b
- `mutations-replacements.tsv`：sha256 741b1e830d3e33a4a1d72de9a74a9168ac2942b11908543744578b1c4c2d16c3
- `report.md`：sha256 a5f4ee8553852f790f675b9fb718521aa6e9598b34def17d0b44a9f0d2deeeac

### crates.patch

````
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index ef85e7c..4bdfd8a 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -483,8 +483,11 @@ pub enum WitnessedCounterComparison {
 
 /// 可写挂载在「重读一次」之前做什么（D16（发布语义） 已定项 1「根槽这一次读坏」那一行：这一次读不出就重读一次，仍坏就拒）。
 /// 产品路径立即重读，两次读之间不等、不另设间隔。只供测试的那一臂在重读之前调用例给的钩子
-/// （`.claude/rules/fs-design.md` 五条硬要求第 2 条：用例在钩子里撤掉暂时的读故障，强制进入「重读读得出」那一支）。
-/// 一次挂载至多调两次：读阶段那一次（判据 N-配置），重建分配器读最新那条根的实例表那一次（代码审阅第 22 条）。
+/// （`.claude/rules/fs-design.md` 五条硬要求第 2 条：用例在钩子里撤掉暂时的读故障，强制进入「重读读得出」那一支；
+/// 或在钩子里让重读那一遍也读坏，强制进入「重读仍读不出」那一支）。
+/// 每处「重读一次」之前各调一次：读阶段那一次（判据 N-配置），重建分配器读最新那条根的实例表那一次（代码审阅第 22 条），
+/// 重建分配器读根环一条不被抛弃的自证有效根都没读出、整环重读那一次（用户 2026-10-05 定「重读」），影子账里一条被抛弃根的账与从映射现算都不成、
+/// 重读它那一次（C393，每条这样的根各一次）。
 pub enum BeforeTheOneReread<'hook> {
     RereadImmediately,
     CallTheTestOnlyHookFirst(&'hook mut dyn FnMut()),
@@ -1909,7 +1912,8 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 })
                 .max()
         };
-    // 这一遍一条不被抛弃的自证有效根都没读出（择根那几遍读得出）就把根环整环再读一遍，根环表、回收门槛与影子账都用重读的读数；
+    // 这一遍一条不被抛弃的自证有效根都没读出（择根那几遍读得出）就把根环整环再读一遍（重读之前调 `before_the_one_reread`，与另几处重读一样），
+    // 根环表、回收门槛与影子账都用重读的读数；
     // 重读还是一条都没有，就在任何写之前拒这次可写挂载（用户 2026-10-05 定「重读」，岔路单 `research/prompts/m3-batch17-code-r1-forks.md`
     // 第 2 行；判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第二节「N1 甲二」那一行与第四节）。不重读、给空表往下走时，
     // 写行那次记根算回收门槛只看得到它自己，释放代高于 F_生效 的一截当场放开、被暖机写掉，盘上环里更旧的有效根还引用着（D16（发布语义） 已定项 1
@@ -1923,6 +1927,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,
             ),
             None => {
+                before_the_one_reread.before_rereading();
                 let readings_of_the_reread = read_every_root_ring_slot_once();
                 let Some(window_anchor) =
                     newest_valid_root_txg_of_the_readings(&readings_of_the_reread)
diff --git a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
index ca72aff..420a8b6 100644
--- a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
+++ b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
@@ -28,7 +28,8 @@ use singlefs_core::block_device::{
 use singlefs_core::container_tree::ContainerTreeNodeEntries;
 use singlefs_core::lineage_tree::LineageContainerCapacity;
 use singlefs_core::mount::{
-    mount_writable_with_test_only_switches_and_deletion_switches, unmount,
+    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
+    mount_writable_with_test_only_switches_and_deletion_switches, unmount, BeforeTheOneReread,
     InstanceTableOfTheNewestRootRead, MountError, Mounted, RereadsOfThisMount,
     RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
 };
@@ -126,6 +127,11 @@ impl SharedRootRingReads {
     fn failed_reads(&self) -> u64 {
         self.0.borrow().failed_reads
     }
+
+    /// 从这一刻起每个根环槽的每次读都报块设备错（用例在「重读一次」之前的钩子里调）。
+    fn fail_every_later_read_of_each_slot(&self) {
+        self.0.borrow_mut().failing_reads_of_each_slot = 1..=u64::MAX;
+    }
 }
 
 /// 读的时候按 [`SharedRootRingReads`] 让根环槽的某几次读报块设备错的盘；写、写零、屏障原样交给里面那块盘。
@@ -563,6 +569,76 @@ fn the_allocator_rebuild_pass_and_its_reread_both_failing_refuse_the_writable_mo
     );
 }
 
+/// 只坏重建分配器那一遍（两块盘全部根环槽报读错），重读之前的钩子（`BeforeTheOneReread::CallTheTestOnlyHookFirst`）让根环此后每次读都坏：
+/// 重读那一遍也一条有效根读不出，在任何写之前拒（`Recovery(NoValidRoot)`），两块盘与挂载之前逐字节相同。钩子恰调一次，调的那一刻每个根环槽
+/// 恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次（重建那一遍之后、重读之前）。
+/// 这个挂载入口不收删除开关、取产品值：拒在建分配器那一步，删除开关在它之后才装上，挂载读根环的次序与别的用例相同（读次数那两条断言现核）。
+/// 判别力：重建分配器整环重读之前不调钩子时，重读那一遍照常读得出、挂载做成，第一项红。
+#[test]
+fn a_hook_before_the_root_ring_reread_failing_that_reread_too_refuses_the_writable_mount_before_any_write(
+) {
+    let history = narrow_pool_after_a_crashed_unmount();
+    let root_ring_reads = SharedRootRingReads::new(
+        history.every_root_ring_slot(),
+        RootRingPassesFailingDuringTheRemount::OnlyTheAllocatorRebuild.failing_reads_of_each_slot(),
+    );
+    let mut devices = history.devices_failing(&root_ring_reads, |_, device| device);
+    let failing_from_the_hook = root_ring_reads.clone();
+    let mut reads_of_each_slot_at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>> = Vec::new();
+    let mut fail_the_reread_too = || {
+        reads_of_each_slot_at_each_hook_call.push(failing_from_the_hook.reads_of_each_slot());
+        failing_from_the_hook.fail_every_later_read_of_each_slot();
+    };
+    let remounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
+        &history.pool.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut fail_the_reread_too),
+    );
+    let image_after_the_refusal = MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.inner.image.clone()))
+            .collect(),
+        device_size_in_bytes: history.image_after_the_crashed_unmount.device_size_in_bytes,
+    };
+    assert!(
+        matches!(
+            remounted,
+            Err(MountError::Recovery(RecoveryFailure::NoValidRoot))
+        ),
+        "钩子让重读那一遍也读坏：重建分配器那一遍与重读都一条有效根读不出，拒这次可写挂载（Recovery(NoValidRoot)）：{:?}",
+        remounted.as_ref().map(|_| "做成了")
+    );
+    assert!(
+        image_after_the_refusal == history.image_after_the_crashed_unmount,
+        "拒在任何写之前：两块盘与挂载之前逐字节相同"
+    );
+    let slot_count = usize::try_from(history.root_ring_slot_count()).expect("槽数");
+    assert!(
+        reads_of_each_slot_at_each_hook_call.len() == 1
+            && reads_of_each_slot_at_each_hook_call[0].len() == slot_count
+            && reads_of_each_slot_at_each_hook_call[0]
+                .values()
+                .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD),
+        "钩子恰调一次，在重建那一遍之后、重读之前（每个根环槽恰读过 {READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD} 次）：{reads_of_each_slot_at_each_hook_call:?}"
+    );
+    assert_eq!(
+        root_ring_reads.failed_reads(),
+        2 * history.root_ring_slot_count(),
+        "读坏的恰是两整遍（重建那一遍与它的重读）：每个根环槽两次"
+    );
+    let reads_of_each_slot = root_ring_reads.reads_of_each_slot();
+    assert!(
+        reads_of_each_slot.len() == slot_count
+            && reads_of_each_slot
+                .values()
+                .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD + 1),
+        "重读那一遍之后根环再没被读过（拒在重读之后、取号之前）：{reads_of_each_slot:?}"
+    );
+}
+
 /// 不注错的对照：同一份盘面再可写挂载，一次都不拒、不重读，根环每个槽读的次数与改之前相同（攻方量的 15 遍），两块盘都在时池级 checker 不判违例。
 #[test]
 fn a_remount_without_faults_does_not_reread_the_root_ring_and_reads_each_slot_as_often_as_before() {
````

### mutations-append.tsv

````
第10f批 重建分配器整环重读根环之前不调「重读一次」的钩子（拿掉钩子调用：用例造不出重读那一遍也读坏）	crates/singlefs-core/src/mount.rs	                before_the_one_reread.before_rereading();\n                let readings_of_the_reread = read_every_root_ring_slot_once();	                let readings_of_the_reread = read_every_root_ring_slot_once();	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	a_hook_before_the_root_ring_reread_failing_that_reread_too_refuses_the_writable_mount_before_any_write
````

### mutations-replacements.tsv

````
C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）	crates/singlefs-core/src/mount.rs	    establish_instance(\n        &parameters_of_the_pool,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger,\n            system_configuration,\n            space_admission,\n            selected_version_journal_position,\n            rereads: RereadsOfThisMount {\n                read_stage: read_stage_settled,\n                instance_table_of_the_newest_root,\n                root_ring_of_the_rebuilt_allocator,\n            },\n            instance_table_sweep,\n        },\n    )\n}\n	    let instance_before_acquisition = previous_row.instance;\n    let outcome = establish_instance(\n        &parameters_of_the_pool,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger,\n            system_configuration,\n            space_admission,\n            selected_version_journal_position,\n            rereads: RereadsOfThisMount {\n                read_stage: read_stage_settled,\n                instance_table_of_the_newest_root,\n                root_ring_of_the_rebuilt_allocator,\n            },\n            instance_table_sweep,\n        },\n    );\n    if let Err(MountError::Publish(_)) = &outcome {\n        let mut rollback_writer = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());\n        let _ = rollback_writer.perform_commit_step(\n            crate::transaction::CommitStep::RotateSystemConfigurationSlots {\n                journal_tail: 0,\n                journal_instance: instance_before_acquisition,\n            },\n        );\n    }\n    outcome\n}\n	-p singlefs-harness --test fault_injection_fast_tier -- write_error_after_the_acquisition	write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it
第10e批 重建分配器读根环那一遍一条有效根都没读出时不重读、当场拒（删掉重读那一遍）	crates/singlefs-core/src/mount.rs	            None => {\n                before_the_one_reread.before_rereading();\n                let readings_of_the_reread = read_every_root_ring_slot_once();\n                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_reread)\n                else {\n                    return Err(MountError::Recovery(RecoveryFailure::NoValidRoot));\n                };\n                (\n                    readings_of_the_reread,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )\n            }	            None => return Err(MountError::Recovery(RecoveryFailure::NoValidRoot)),	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over
第10e批 重建分配器读根环那一遍一条有效根都没读出时照第 17 批那样不重读、给空表往下走（攻方 1.3 甲二：暖机写进盘 0 上最新的根还引用的槽）	crates/singlefs-core/src/mount.rs	            None => {\n                before_the_one_reread.before_rereading();\n                let readings_of_the_reread = read_every_root_ring_slot_once();\n                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_reread)\n                else {\n                    return Err(MountError::Recovery(RecoveryFailure::NoValidRoot));\n                };\n                (\n                    readings_of_the_reread,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )\n            }	            None => (\n                Vec::new(),\n                CheckpointTxg(0),\n                RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,\n            ),	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	only_the_allocator_rebuild_pass_failing_and_a_crash_before_the_warm_up_root_lands_on_device_zero_keep_what_device_zero_reads
````

