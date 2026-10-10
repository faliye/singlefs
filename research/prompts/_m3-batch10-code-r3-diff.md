# 根环实现第 10 批代码轮第三轮：改动全文（2026-10-06）

只有一份补丁：第 10h 批，相对它合入那一刻的主工作区（形式化接入那条线第 34 批合后）。

## batch10h

- 补丁目录：`/tmp/claude-1000/m3-r1/batch10h/patch/`
- `crates.patch`：sha256 ae33b1450fefcad7e00026c9cb607d0785188d4919f3367fa3d05cbf226d1053
- `mutations-append.tsv`：sha256 ba915963d0a1fb2d915c4c51d75b286071a4dfab4ec4767801e6a52c49709d15
- `mutations-replacements.tsv`：sha256 f8440a6c6d0e7cd67923f0735d153702d6a49c3f9d2065daea809ac227f013e6
- `report.md`：sha256 7feb79d386543f2370c0b401ac39e01ae0013c577a7f9d4aaf12a4109b24762c

### crates.patch

````
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -97,9 +97,11 @@
 #[derive(Debug)]
 pub enum MountError {
     Recovery(RecoveryFailure),
-    /// 重建分配器读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出（被抛弃按最新那条根指着的实例表判），整环重读一次，
-    /// 重读那一遍整个环仍一条都没有：在取号之前拒可写，盘上逐字节不变，只读挂载照常（D23（journal 的角色与格式） 已定项 14 那张单子上的一行，
-    /// 用户 2026-10-05 定「另起拒因」，岔路单 `research/prompts/m3-batch10-code-r1-forks.md` Q3；重读的触发照 Q4 rr2）。
+    /// 重建分配器读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出（被抛弃按最新那条根指着的实例表判；一条根归哪块盘按读出它的那个槽所在的区域），
+    /// 整环重读一次，两遍逐槽并起来（`merged_readings_of_the_two_root_ring_passes`）整个环仍一条都没有：在取号之前拒可写，盘上逐字节不变，
+    /// 只读挂载照常（D23（journal 的角色与格式） 已定项 14 那张单子上的一行，用户 2026-10-05 定「另起拒因」，岔路单
+    /// `research/prompts/m3-batch10-code-r1-forks.md` Q3；重读的触发照 Q4 rr2；两遍按槽并照用户 2026-10-05 定 R1「按槽并两遍」，
+    /// 判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第六节）。第一遍读出过一条有效根的，并出来的那一份里就有它，不拒。
     /// 与读阶段择不到根的 `Recovery(RecoveryFailure::NoValidRoot)` 分开报：那一拒只读挂载同样挂不上，这一拒读阶段择得到根。
     NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
     /// 所选根下面有文件版本，而环里一条自证过的记录都没有：从盘上重建上一版要一条记录顶着，拿不出来
@@ -535,9 +537,11 @@
     OnTheOneReread,
 }
 
-/// 重建分配器读根环（根环表、回收门槛与影子账用的那一遍读数）在哪一遍读出不被抛弃的自证有效根：第一遍有一块盘一条都没读出就整环重读一次
-/// （用户 2026-10-05 定「重读」，Q4 定 rr2：不只两块盘都没读出时），重读那一遍整个环一条都没有就拒
+/// 重建分配器读根环（根环表、回收门槛与影子账用的那一份读数）在哪一遍定下：第一遍有一块盘一条不被抛弃的自证有效根都没读出就整环重读一次
+/// （用户 2026-10-05 定「重读」，Q4 定 rr2：不只两块盘都没读出时；一条根归哪块盘按读出它的那个槽所在的区域），用的是两遍逐槽并起来的那一份
+/// （用户 2026-10-05 定 R1「按槽并两遍」）；并起来整个环一条都没有就拒
 /// （`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，不交回这个值）。
+/// 只做过 mkfs、环里只有第 0 代根的池第一遍就够：mkfs 在三个区域的槽 0 各写一份第 0 代根，每块盘都读得出一条。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum RootRingOfTheRebuiltAllocatorRead {
     OnTheFirstRead,
@@ -1763,6 +1767,68 @@
     }
 }
 
+/// 一个根环槽两遍读数里较好的那一份（用户 2026-10-05 定 R1「按槽并两遍」，判决 `research/prompts/m3-batch10-code-r2-main-verification.md`
+/// 第六节）：任一遍自证过的根（两遍都自证过取重读那一条）＞ 任一遍读坏、不是全零 ＞ 全零。
+/// 自证过的根不是读错造得出来的（整槽校验和过、fsid 是本池的），挂载期间又没有别的写者，第一遍读出的那一条重读时仍住在那一槽：
+/// 只拿重读那一遍、丢掉第一遍读出的根，第一遍读得出、重读读坏的被抛弃根会掉出影子账，它独占的单元当场发得出去
+/// （攻方第二次抽样 `research/prompts/m3-batch10-code-r2-opus-s2-output.md` K2-2，违反 D23（journal 的角色与格式） 已定项 14
+/// 「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」）。读坏压过全零：读坏那一类回收门槛钉住、准入抬 F 的上限也并，
+/// 全零那一类只在回收门槛里钉住（用户 2026-10-05 定乙），两类里读坏的约束多。
+fn better_reading_of_a_root_ring_slot_over_the_two_passes(
+    first_reading: RootRingSlotReadingTellingNeverWrittenApart,
+    reading_of_the_reread: RootRingSlotReadingTellingNeverWrittenApart,
+) -> RootRingSlotReadingTellingNeverWrittenApart {
+    use RootRingSlotReadingTellingNeverWrittenApart::{
+        AllZeroNeverWritten, BadAndNotAllZero, SelfVerified,
+    };
+    match (first_reading, reading_of_the_reread) {
+        (SelfVerified(_), SelfVerified(root_of_the_reread)) => SelfVerified(root_of_the_reread),
+        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(root_of_the_reread)) => {
+            SelfVerified(root_of_the_reread)
+        }
+        (SelfVerified(root_of_the_first_pass), BadAndNotAllZero | AllZeroNeverWritten) => {
+            SelfVerified(root_of_the_first_pass)
+        }
+        (BadAndNotAllZero, BadAndNotAllZero) => BadAndNotAllZero,
+        (BadAndNotAllZero, AllZeroNeverWritten) | (AllZeroNeverWritten, BadAndNotAllZero) => {
+            BadAndNotAllZero
+        }
+        (AllZeroNeverWritten, AllZeroNeverWritten) => AllZeroNeverWritten,
+    }
+}
+
+/// 重建分配器读根环的两遍（[`every_root_ring_slot`] 的次序，每槽各读一次）逐槽并成一份
+/// （[`better_reading_of_a_root_ring_slot_over_the_two_passes`]）。两遍按同一个次序读同一组槽，槽对不上是这里的 bug、断言。
+fn merged_readings_of_the_two_root_ring_passes(
+    first_readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
+    readings_of_the_reread: Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)>,
+) -> Vec<(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)> {
+    assert_eq!(
+        first_readings.len(),
+        readings_of_the_reread.len(),
+        "两遍都按 every_root_ring_slot 读整个环，槽数相同"
+    );
+    first_readings
+        .iter()
+        .zip(readings_of_the_reread)
+        .map(
+            |((ring_slot_of_the_first_pass, first_reading), (ring_slot, reading_of_the_reread))| {
+                assert_eq!(
+                    *ring_slot_of_the_first_pass, ring_slot,
+                    "两遍都按 every_root_ring_slot 的次序读，第 n 个读数是同一个槽"
+                );
+                (
+                    ring_slot,
+                    better_reading_of_a_root_ring_slot_over_the_two_passes(
+                        *first_reading,
+                        reading_of_the_reread,
+                    ),
+                )
+            },
+        )
+        .collect()
+}
+
 /// 挂载那一刻读坏（设备报读错，或读得出、不是全零而自证不过）的根环槽各能住哪几个 txg（区域内槽位只由 txg 定，
 /// D22（单元原子性怎么合成） 已定项 2）：
 /// - 可能住过的最旧 txg：落到这个槽、小于 `first_txg` 的最小 txg。回收门槛钉在这里（用户 2026-10-04 定：读不出、自证不过的根槽在回收门槛里
@@ -1836,8 +1902,8 @@
 /// # Errors
 /// 最新那条根的实例表读不出、重读一次仍读不出（[`instance_table_of_the_newest_root_read_at_most_twice`]）——在读阶段之后、取号之前；
 /// 树表 0 条那一版的账重建不出（`allocator_of_version_without_file`）；
-/// 读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，重读那一遍整个环仍一条都没有 ⇒
-/// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2）。
+/// 读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有 ⇒
+/// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2、R1 按槽并两遍）。
 /// 都在取号之前，盘上逐字节不变。
 #[allow(
     clippy::ptr_arg,
@@ -1919,38 +1985,49 @@
                 })
                 .max()
         };
-    // 一条根落在哪块盘由根环落点公式定（下面判「每块盘都读出了有效根」与当场放开到哪儿都用它）。
-    let device_of_txg = |txg: CheckpointTxg| {
-        let target = target_for_publish(txg, slots_per_region);
+    // 一条根归哪块盘按读出它的那个槽所在区域的 `region_devices` 定（下面判「每块盘都读出了有效根」与当场放开到哪儿取 m 都用它，同一处定义）。
+    // 不按 txg 的落点公式（`target_for_publish`）：mkfs 在三个区域的槽 0 各写一份第 0 代根（`make_filesystem::genesis_root_ring_slots`），
+    // 公式把三份都归到盘 0，环里只剩第 0 代根时盘 1 实际读出了有效根、却算成一条都没有，rr2 照样整环重读（判决
+    // `research/prompts/m3-batch10-code-r2-main-verification.md` 第二节 K2 第一、二、三格，攻方第一次抽样 H1）。别的根都按公式写进槽，两种归法只在第 0 代根上不同。
+    let device_of_the_ring_slot = |ring_slot: RootRingSlot| {
         system_configuration.immutable.region_devices
-            [usize::try_from(target.region).expect("区域号")]
+            [usize::try_from(ring_slot.region).expect("区域号")]
+    };
+    let roots_on_the_devices_they_were_read_from = |readings: &[(
+        RootRingSlot,
+        RootRingSlotReadingTellingNeverWrittenApart,
+    )]|
+     -> Vec<(DeviceIdentity, RootRecord)> {
+        readings
+            .iter()
+            .filter_map(|(ring_slot, reading)| match reading {
+                RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) => {
+                    Some((device_of_the_ring_slot(*ring_slot), *root))
+                }
+                RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
+                | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
+            })
+            .collect()
     };
     let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
     // 这一遍每块盘都读出了一条不被抛弃的自证有效根（与下面当场放开到哪儿那一步取 m 同一处定义 [`newest_valid_root_txg_on_every_device`]）。
     let a_valid_root_was_read_on_every_device =
         |readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)]| {
-            let roots_of_the_readings: Vec<RootRecord> = readings
-                .iter()
-                .filter_map(|(_, reading)| match reading {
-                    RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) => Some(*root),
-                    RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero
-                    | RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => None,
-                })
-                .collect();
             newest_valid_root_txg_on_every_device(
-                &roots_of_the_readings,
+                &roots_on_the_devices_they_were_read_from(readings),
                 &newest_table,
                 &all_devices,
-                &device_of_txg,
             )
             .is_some()
         };
     // 这一遍有一块盘一条不被抛弃的自证有效根都没读出（择根那几遍读得出）就把根环整环再读一遍（用户 2026-10-05 定 Q4 rr2，
     // 岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 第 4 行：改之前只在整个环一条都没读出时重读，只坏这一遍、只有一块盘没读出的那一形
     // m 取不到、当场只放到环里最旧有效根，窄盘上取不到落点被拒；重读之前调 `before_the_one_reread`，与另几处重读一样），
-    // 根环表、回收门槛与影子账都用重读的读数（重读那一遍仍有一块盘没读出的，照 m 取不到那一支往下走）；
-    // 重读那一遍整个环还是一条都没有，就在任何写之前拒这次可写挂载（用户 2026-10-05 定「重读」，岔路单 `research/prompts/m3-batch17-code-r1-forks.md`
-    // 第 2 行；判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第二节「N1 甲二」那一行与第四节；拒因另起成员，Q3）。不重读、给空表往下走时，
+    // 根环表、回收门槛与影子账都用两遍逐槽并起来的那一份（用户 2026-10-05 定 R1「按槽并两遍」，[`merged_readings_of_the_two_root_ring_passes`]：
+    // 只拿重读那一遍时，第一遍读出、重读读坏的被抛弃根掉出影子账；并起来那一份仍有一块盘没读出的，照 m 取不到那一支往下走）；
+    // 并起来整个环还是一条都没有，就在任何写之前拒这次可写挂载（用户 2026-10-05 定「重读」，岔路单 `research/prompts/m3-batch17-code-r1-forks.md`
+    // 第 2 行；判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第二节「N1 甲二」那一行与第四节；拒因另起成员，Q3；R1 之后
+    // 第一遍读出过一条有效根的不拒）。不重读、给空表往下走时，
     // 写行那次记根算回收门槛只看得到它自己，释放代高于 F_生效 的一截当场放开、被暖机写掉，盘上环里更旧的有效根还引用着（D16（发布语义） 已定项 1
     // 「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）。拒在取号之前，盘上逐字节不变。
     let first_readings = read_every_root_ring_slot_once();
@@ -1967,15 +2044,19 @@
             None => {
                 before_the_one_reread.before_rereading();
                 let readings_of_the_reread = read_every_root_ring_slot_once();
+                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(
+                    &first_readings,
+                    readings_of_the_reread,
+                );
                 let Some(window_anchor) =
-                    newest_valid_root_txg_of_the_readings(&readings_of_the_reread)
+                    newest_valid_root_txg_of_the_readings(&readings_of_the_two_passes_merged)
                 else {
                     return Err(
                         MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
                     );
                 };
                 (
-                    readings_of_the_reread,
+                    readings_of_the_two_passes_merged,
                     window_anchor,
                     RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,
                 )
@@ -2015,7 +2096,7 @@
     // 挂载时读坏的槽能住哪几个 txg（[`txgs_ring_slots_bad_at_mount_can_hold`]）：N = 环里读得出的最新有效根的 txg（D16（发布语义） 已定项 1
     // 「挂载时读不出的槽」那一句：挂载重建分配器时取它、不取 first_txg − 1）。读回全零的槽按同一个算法另算一份、只取可能住过的最旧 txg：
     // 回收门槛里照「自证不过」那一类钉住（用户 2026-10-05 定乙），准入抬 F 的上限不并它（用户 2026-10-04 定），能住的最大 txg 用不到。
-    // 一个 txg 都住不了的槽（年轻的环里还没轮到的）两类都不交回。窗口锚上面取到了（有一块盘没读出就重读、重读那一遍整个环仍取不到就拒），
+    // 一个 txg 都住不了的槽（年轻的环里还没轮到的）两类都不交回。窗口锚上面取到了（有一块盘没读出就重读、两遍并起来整个环仍取不到就拒），
     // 两类不会因为它是空表。
     let ring_slots_bad_at_mount = txgs_ring_slots_bad_at_mount_can_hold(
         &bad_ring_slots_not_all_zero,
@@ -2061,15 +2142,14 @@
     // 某块盘一条有效根都读不出（m 取不到）时当场只放到环里最旧有效根（用户 2026-10-04 选甲，`research/prompts/m3-batch16-code-r1-forks.md`
     // 第 1 行；判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「再挂期间某块盘根环整段读坏」那一行；用户 2026-10-04 定 RGall
     // 之后取整个环里的有效根、不按一圈窗口滤）：那块盘上真正最新的根这一遍看不到，它引用的单元释放代可以高到 F_生效；它与回收门槛同一个读数
-    // （挂载时读坏、读回全零的槽钉在它可能住过的最旧 txg 并进）。第一遍有一块盘没读出就整环重读（Q4 rr2），走到这一支的只剩重读那一遍
-    // 仍有一块盘一条有效根都没读出的那一形。这一遍（或它的重读）至少读出了一条有效根（上面取窗口锚那一步：重读那一遍也一条都读不出就拒），
-    // 环里最旧有效根恒取得到。
+    // （挂载时读坏、读回全零的槽钉在它可能住过的最旧 txg 并进）。第一遍有一块盘没读出就整环重读（Q4 rr2），走到这一支的只剩两遍并起来
+    // 仍有一块盘一条有效根都没读出的那一形。这一份读数至少有一条有效根（上面取窗口锚那一步：两遍并起来也一条都没有就拒），
+    // 环里最旧有效根恒取得到。m 与上面判要不要重读同一个归盘法（按读出它的槽所在的区域）。
     let floor_of_the_reclaim = reclaim_floor(effective_floor, oldest_valid_root_in_the_ring);
     let floor_released_immediately = match newest_valid_root_txg_on_every_device(
-        &roots,
+        &roots_on_the_devices_they_were_read_from(&ring_slot_readings),
         &newest_table,
         &all_devices,
-        &device_of_txg,
     ) {
         Some(newest_valid_root_on_every_device) => Some(reclaim_floor(
             effective_floor.min(newest_valid_root_on_every_device),
@@ -2646,9 +2726,11 @@
     ImmediatelyBelowTheNewestValidRootOnEveryDevice,
 }
 
-/// 每块盘上最新的有效根的 txg 取最小（GA，[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]；可写挂载重建分配器时当场放开到哪儿也取它，
-/// 甲，`rebuilt_allocator`）：`roots` 是调用方这一遍从根环读出来的根，`table` 是判被抛弃用的那张实例表（卸载那一格是现行那一版的，
-/// 重建那一格是最新那条根指着的），一条根落在哪块盘由根环落点公式定（`device_of_txg`）。
+/// 每块盘上最新的有效根的 txg 取最小（GA，[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]；可写挂载重建分配器时当场放开到哪儿、
+/// 要不要整环重读也取它，甲、Q4 rr2，`rebuilt_allocator`）：`roots_on_their_devices` 是调用方这一遍从根环读出来的根，各带它归哪块盘，
+/// `table` 是判被抛弃用的那张实例表（卸载那一格是现行那一版的，重建那一格是最新那条根指着的）。归盘由调用方定：重建那一格按读出它的那个槽
+/// 所在的区域（mkfs 三个区域槽 0 各一份的第 0 代根各归各的盘，判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第二节 K2），
+/// 卸载那一格按根环落点公式（那一遍的读数不带槽；卸载接在一次可写挂载之后，暖机已在每块盘上落了根，环里不会只剩第 0 代根）。
 /// - 被抛弃的根（按 `table`，`abandoned_by_table`）不算。卸载那一格滤不滤结局相同：卸载接在一次可写挂载之后，那次挂载的暖机先在每块盘上
 ///   落了新实例的根（D16（发布语义） 已定项 8 戊），新实例的 txg 大于环里每一条根（D23（journal 的角色与格式） 已定项 14 第 3 条），
 ///   卸载时各盘最新的根都不是被抛弃的（判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「单盘恢复择中被抛弃的根」那一行，
@@ -2660,21 +2742,19 @@
 ///   与回收门槛同一个读数），其余扣到这次挂载的暖机落满每块盘（`rebuilt_allocator`，用户 2026-10-04 选甲，
 ///   `research/prompts/m3-batch16-code-r1-forks.md` 第 1 行）。
 fn newest_valid_root_txg_on_every_device(
-    roots_read_from_the_ring: &[RootRecord],
+    roots_on_their_devices: &[(DeviceIdentity, RootRecord)],
     table: &InstanceTableRecords,
     all_devices: &[DeviceIdentity],
-    device_of_txg: &dyn Fn(CheckpointTxg) -> DeviceIdentity,
 ) -> Option<CheckpointTxg> {
     let newest_on_each_device: Vec<Option<CheckpointTxg>> = all_devices
         .iter()
         .map(|device| {
-            roots_read_from_the_ring
+            roots_on_their_devices
                 .iter()
-                .filter(|root| {
-                    !abandoned_by_table(root, table)
-                        && device_of_txg(root.checkpoint_txg) == *device
+                .filter(|(device_of_the_root, root)| {
+                    !abandoned_by_table(root, table) && *device_of_the_root == *device
                 })
-                .map(|root| root.checkpoint_txg)
+                .map(|(_, root)| root.checkpoint_txg)
                 .max()
         })
         .collect();
@@ -3256,11 +3336,14 @@
                 ReclaimedReuse::HeldUntilFloorTakesEffect,
             ),
         ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice => {
+            let roots_on_the_devices_of_their_txgs: Vec<(DeviceIdentity, RootRecord)> = roots
+                .iter()
+                .map(|root| (device_of_txg(root.checkpoint_txg), *root))
+                .collect();
             let mut reclaimed = match newest_valid_root_txg_on_every_device(
-                &roots,
+                &roots_on_the_devices_of_their_txgs,
                 &table,
                 &all_devices,
-                &device_of_txg,
             ) {
                 Some(newest_valid_root_on_every_device) => allocator.reclaim_released_up_to(
                     floor_of_the_reclaim.min(newest_valid_root_on_every_device),
@@ -7012,27 +7095,168 @@
             }],
         };
         let both_devices = [DeviceIdentity(0), DeviceIdentity(1)];
+        let on_their_devices = |roots: &[RootRecord]| -> Vec<(DeviceIdentity, RootRecord)> {
+            roots
+                .iter()
+                .map(|root| (device_of_an_even_or_odd_txg(root.checkpoint_txg), *root))
+                .collect()
+        };
         let roots = [root_of(1, 7), root_of(1, 8), root_of(1, 9), root_of(2, 12)];
         assert_eq!(
-            newest_valid_root_txg_on_every_device(
-                &roots,
-                &table,
-                &both_devices,
-                &device_of_an_even_or_odd_txg
-            ),
+            newest_valid_root_txg_on_every_device(&on_their_devices(&roots), &table, &both_devices,),
             Some(CheckpointTxg(7)),
             "盘 0 上最新的有效根 txg 12、盘 1 上 txg 7（txg 9 被抛弃、不算），取最小是 7"
         );
         let only_the_abandoned_root_on_device_one = [root_of(1, 8), root_of(1, 9), root_of(2, 12)];
         assert_eq!(
             newest_valid_root_txg_on_every_device(
-                &only_the_abandoned_root_on_device_one,
+                &on_their_devices(&only_the_abandoned_root_on_device_one),
                 &table,
                 &both_devices,
-                &device_of_an_even_or_odd_txg
             ),
             None,
             "盘 1 上只有被抛弃的 txg 9：那块盘一条有效根都没有"
         );
     }
 }
+
+#[cfg(test)]
+mod merged_readings_of_the_two_root_ring_passes_tests {
+    use super::{
+        better_reading_of_a_root_ring_slot_over_the_two_passes,
+        merged_readings_of_the_two_root_ring_passes, RootRingSlotReadingTellingNeverWrittenApart,
+    };
+    use crate::address::{CheckpointTxg, InstanceGeneration};
+    use crate::pointer::NodePointer;
+    use crate::root_record::{RootRecord, UnmountMarker};
+    use crate::root_ring::RootRingSlot;
+    use RootRingSlotReadingTellingNeverWrittenApart::{
+        AllZeroNeverWritten, BadAndNotAllZero, SelfVerified,
+    };
+
+    fn root_of(instance: u32, checkpoint_txg: u64) -> RootRecord {
+        RootRecord {
+            filesystem_identifier: [0; 16],
+            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+            instance: InstanceGeneration(instance),
+            checkpoint_txg: CheckpointTxg(checkpoint_txg),
+            tree_table: NodePointer::empty_root(),
+            tree_identifier_watermark: 0,
+            rollback_floor: CheckpointTxg(0),
+            instance_table: NodePointer::empty_root(),
+            mapping_root: NodePointer::empty_root(),
+            allocation_record_tree_root: NodePointer::empty_root(),
+        }
+    }
+
+    /// 场景：一个根环槽两遍读数的九种组合（第一遍、重读各取自证过、读坏不是全零、全零三种），两遍都自证过时两遍读出的是两条不同的根
+    /// （只为看出取的是哪一遍的，挂载里同一槽两遍读出的是同一条）。
+    /// 预期：照用户 2026-10-05 定 R1 的次序——任一遍自证过的根（两遍都自证过取重读那一条）＞ 任一遍读坏不是全零 ＞ 全零。
+    /// 判别力：`crates/mutations.tsv` 里「第10h批 R1 …」那几行各改一处判别，各红在点名那一组上。
+    #[test]
+    fn each_pair_of_readings_merges_into_the_self_verified_root_then_the_bad_reading_then_all_zero()
+    {
+        let root_of_the_first_pass = root_of(1, 7);
+        let root_of_the_reread = root_of(1, 31);
+        let pairs_and_merged = [
+            (
+                SelfVerified(root_of_the_first_pass),
+                SelfVerified(root_of_the_reread),
+                SelfVerified(root_of_the_reread),
+                "两遍都自证过：取重读那一条",
+            ),
+            (
+                BadAndNotAllZero,
+                SelfVerified(root_of_the_reread),
+                SelfVerified(root_of_the_reread),
+                "第一遍读坏、重读自证过：取重读那一条",
+            ),
+            (
+                AllZeroNeverWritten,
+                SelfVerified(root_of_the_reread),
+                SelfVerified(root_of_the_reread),
+                "第一遍全零、重读自证过：取重读那一条",
+            ),
+            (
+                SelfVerified(root_of_the_first_pass),
+                BadAndNotAllZero,
+                SelfVerified(root_of_the_first_pass),
+                "第一遍自证过、重读读坏：取第一遍那一条",
+            ),
+            (
+                SelfVerified(root_of_the_first_pass),
+                AllZeroNeverWritten,
+                SelfVerified(root_of_the_first_pass),
+                "第一遍自证过、重读全零：取第一遍那一条",
+            ),
+            (
+                BadAndNotAllZero,
+                BadAndNotAllZero,
+                BadAndNotAllZero,
+                "两遍都读坏：读坏",
+            ),
+            (
+                BadAndNotAllZero,
+                AllZeroNeverWritten,
+                BadAndNotAllZero,
+                "第一遍读坏、重读全零：读坏压过全零",
+            ),
+            (
+                AllZeroNeverWritten,
+                BadAndNotAllZero,
+                BadAndNotAllZero,
+                "第一遍全零、重读读坏：读坏压过全零",
+            ),
+            (
+                AllZeroNeverWritten,
+                AllZeroNeverWritten,
+                AllZeroNeverWritten,
+                "两遍都全零：全零",
+            ),
+        ];
+        // 迭代上界是九种组合；没有提前出口。
+        for (first_reading, reading_of_the_reread, expected, case) in pairs_and_merged {
+            assert_eq!(
+                better_reading_of_a_root_ring_slot_over_the_two_passes(
+                    first_reading,
+                    reading_of_the_reread
+                ),
+                expected,
+                "{case}"
+            );
+        }
+    }
+
+    /// 场景：三个槽的两遍读数整份并（第一遍：自证过、读坏、全零；重读：读坏、自证过、读坏）。
+    /// 预期：逐槽并、次序与槽不变——第一个槽留第一遍的根，第二个槽取重读的根，第三个槽读坏。
+    /// 判别力：并的时候槽错位、或只取其中一遍时这一项红（改回整份换成重读那一遍改的是 `rebuilt_allocator` 里的调用点，由 harness 档用例罩）。
+    #[test]
+    fn the_two_passes_are_merged_slot_by_slot_in_ring_order() {
+        let slots = [
+            RootRingSlot { region: 0, slot: 0 },
+            RootRingSlot { region: 1, slot: 0 },
+            RootRingSlot { region: 2, slot: 0 },
+        ];
+        let root_of_the_first_pass = root_of(2, 8);
+        let root_of_the_reread = root_of(3, 13);
+        let first_readings = vec![
+            (slots[0], SelfVerified(root_of_the_first_pass)),
+            (slots[1], BadAndNotAllZero),
+            (slots[2], AllZeroNeverWritten),
+        ];
+        let readings_of_the_reread = vec![
+            (slots[0], BadAndNotAllZero),
+            (slots[1], SelfVerified(root_of_the_reread)),
+            (slots[2], BadAndNotAllZero),
+        ];
+        assert_eq!(
+            merged_readings_of_the_two_root_ring_passes(&first_readings, readings_of_the_reread),
+            vec![
+                (slots[0], SelfVerified(root_of_the_first_pass)),
+                (slots[1], SelfVerified(root_of_the_reread)),
+                (slots[2], BadAndNotAllZero),
+            ],
+            "逐槽取较好的那一份，次序与槽不变"
+        );
+    }
+}
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -2224,9 +2224,10 @@
     /// 读阶段判完、判据 N-配置 为假之后，取号之前的逐盘核或取号那一刻读系统配置判不出（R2）或某块盘一份自证过的都没有
     /// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`；C664（挂载拒绝合法复算把两种拒因报成一个）之前并在 N-配置 里报）。
     SystemConfigurationUnreadableAfterTheReadStage,
-    /// 重建分配器读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，重读那一遍整个环仍一条都没有
+    /// 重建分配器读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有
     /// （`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`；用户 2026-10-05 定 Q3「另起拒因、单子加一行」，
-    /// 岔路单 `research/prompts/m3-batch10-code-r1-forks.md`；之前报 `Recovery(NoValidRoot)`、与读阶段择不到根同名，不在单子上）。
+    /// 岔路单 `research/prompts/m3-batch10-code-r1-forks.md`；之前报 `Recovery(NoValidRoot)`、与读阶段择不到根同名，不在单子上；
+    /// 两遍按槽并照用户 2026-10-05 定 R1，判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第六节：第一遍读出过一条有效根的不拒）。
     NoValidRootInTheRootRingOfTheRebuiltAllocator,
 }
 
@@ -2527,11 +2528,15 @@
         .is_none_or(|newest| instance_table_of_root(crash_image, &newest).is_none())
 }
 
-/// 「重建分配器读根环整环重读一次仍一条不被抛弃的自证有效根都没有」（Q3）在崩溃镜像上复算：按最新那条根（`choose_root`）指着的实例表，
-/// 根环里一条不被抛弃的自证有效根都没有。被抛弃照 D23（journal 的角色与格式） 已定项 14 候选集那一格在这里另写一份：表里有那条根的实例的行
+/// 「重建分配器读根环整环重读一次、两遍逐槽并起来仍一条不被抛弃的自证有效根都没有」（Q3，并两遍照用户 2026-10-05 定 R1）在崩溃镜像上复算：
+/// 按最新那条根（`choose_root`）指着的实例表，根环里一条不被抛弃的自证有效根都没有。并两遍之后判据照旧只读一遍：崩溃镜像上没有瞬时读错，
+/// 两遍读出的逐槽相同，并起来就是那一遍（任一遍自证过的根留着，两遍都没有才算没有）。被抛弃照 D23（journal 的角色与格式） 已定项 14 候选集那一格在这里另写一份：表里有那条根的实例的行
 /// (i, Ti, Wi) 且它的 txg > Ti（不调实现的 `recovery::root_is_abandoned_by_the_instance_table`）。择不到系统配置、择不到根都按真；
 /// 最新那条根的实例表读不出时没有「被抛弃」可判，不按表滤（读得出的根都算有效，与 F 生效值那一处的取法同向），择得到根就为假。
-/// 没有读故障时最新那条根按它自己指着的表恒不被抛弃（表里的行只记更早的实例，推的），所以择得到根的崩溃镜像上这条判据为假。
+/// 没有读故障时最新那条根按它自己指着的表恒不被抛弃（表里的行只记更早的实例：写行只给比要取的号小的实例写，`mount.rs` 的
+/// `instance_rows_to_write`），所以择得到根的崩溃镜像上这条判据为假；「按表判被抛弃」那半句在合法盘面上不改判据的值，等价性留档在
+/// `writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs`
+/// 的 `a_refusal_for_no_valid_root_on_a_crash_image_holding_an_abandoned_root_is_illegal_because_the_newest_root_is_never_abandoned_by_its_own_table`。
 fn no_valid_root_in_the_root_ring_under_the_newest_roots_instance_table_on_the_crash_image(
     crash_image: &dyn PoolReader,
 ) -> bool {
@@ -2622,7 +2627,7 @@
     /// 一份都择不到，或见证判不出）——没有读故障的历史留不下这份盘面，正确实现在它上面永远拒可写；与 C670 那一条同形，拒是对的、造出这份盘面的那一段不对。
     SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults,
     /// 与改法 G4 同形（用户 2026-10-05 定 Q3，判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）：崩溃态没注入读故障，
-    /// 拒因是「重建分配器读根环整环重读一次仍一条不被抛弃的自证有效根都没有」，那一拒的判据在崩溃镜像上复算为真（按最新那条根的实例表，
+    /// 拒因是「重建分配器读根环整环重读一次、两遍并起来仍一条不被抛弃的自证有效根都没有」，那一拒的判据在崩溃镜像上复算为真（按最新那条根的实例表，
     /// 环里一条不被抛弃的自证有效根都没有）——没有读故障的历史留不下这份盘面（读阶段择得到根、它按自己的实例表不被抛弃），正确实现在它上面
     /// 永远拒可写；拒是对的、造出这份盘面的那一段不对。
     NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,
@@ -2695,7 +2700,7 @@
 /// 拒得不合法（[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）；与它同形的改法 G4——崩溃态没注入读故障、
 /// 理由是「读阶段判完之后系统配置又读不出」，而那一拒的判据在崩溃镜像上复算为真，拒得不合法
 /// （[`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`]）；与 G4 同形的 Q3——
-/// 崩溃态没注入读故障、理由是「重建分配器读根环整环重读一次仍一条不被抛弃的自证有效根都没有」，而那一拒的判据在崩溃镜像上复算为真，拒得不合法
+/// 崩溃态没注入读故障、理由是「重建分配器读根环整环重读一次、两遍并起来仍一条不被抛弃的自证有效根都没有」，而那一拒的判据在崩溃镜像上复算为真，拒得不合法
 /// （[`RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults`]）。注入过读故障的崩溃态
 /// 这三条都不判（D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。理由是「读阶段判完之后系统配置又读不出」或 Q3 那一拒而被拒的那次挂载
 /// 自己在读故障下跑的（`read_faults_under_the_refused_mount`），判据在崩溃镜像上复算为假不算不合法（两遍读之间的瞬时读错复算不出，C664（挂载拒绝合法复算
diff --git a/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs b/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
--- a/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
+++ b/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
@@ -491,8 +491,12 @@
 }
 
 /// 判据的第一支：只做过 mkfs 的池两块盘的系统配置 tail 都是 0 ⇒ 没有见证，判假、读阶段不重读。
-/// 重建分配器读根环那一遍照 Q4 rr2（用户 2026-10-05 定，岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 第 4 行）整环重读一次：
-/// 只做过 mkfs 的环里只有 mkfs 那一条根，落在一块盘上，另一块盘一条有效根都没有。
+/// 重建分配器读根环那一遍也不重读：mkfs 在三个区域的槽 0 各写一份第 0 代根（区域 0、2 在盘 0，区域 1 在盘 1），一条根按读出它的那个槽
+/// 所在的区域归盘，两块盘各读出一条有效根，Q4 rr2（用户 2026-10-05 定，岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 第 4 行）
+/// 「任一块盘没读出」不成立。
+/// 判别力：归盘改回按 txg 的落点公式（`crates/mutations.tsv`「第10h批 rr2 改回按 txg 的落点公式归盘」）时三份第 0 代根都归到盘 0，
+/// 盘 1 算成一条都没有、整环重读一次，最后一项报 `OnTheOneReread`、这一条红（判决
+/// `research/prompts/m3-batch10-code-r2-main-verification.md` 第二节 K2）。
 #[test]
 fn the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread() {
     let mut formatted = format_pool("c554-yi-formatted-pool-nothing-witnessed");
@@ -515,7 +519,7 @@
                 },
             },
             instance_table_of_the_newest_root: InstanceTableOfTheNewestRootRead::OnTheFirstRead,
-            root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,
+            root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,
         }
     );
 }
diff --git a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
--- a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
+++ b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
@@ -1,6 +1,8 @@
-//! 重建分配器读根环那一遍只要有一块盘一条不被抛弃的自证有效根都没读出，就把根环整环再读一遍（重读之前照旧调 `BeforeTheOneReread` 钩子），
-//! 根环表、回收门槛与影子账用重读的读数（用户 2026-10-05 定 Q4「rr2」，岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 第 4 行，
-//! 判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）。改之前只在整个环一条都没读出时重读：只坏重建那一遍、
+//! 重建分配器读根环那一遍只要有一块盘一条不被抛弃的自证有效根都没读出（一条根按读出它的那个槽所在的区域归盘），就把根环整环再读一遍
+//! （重读之前照旧调 `BeforeTheOneReread` 钩子；用户 2026-10-05 定 Q4「rr2」，岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 第 4 行，
+//! 判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节），根环表、回收门槛与影子账用两遍逐槽并起来的那一份读数
+//! （用户 2026-10-05 定 R1「按槽并两遍」：任一遍自证过的根 ＞ 任一遍读坏不是全零 ＞ 全零，判决 `research/prompts/m3-batch10-code-r2-main-verification.md`
+//! 第六节）。改之前只在整个环一条都没读出时重读：只坏重建那一遍、
 //! 只有一块盘没读出的那一形 m 取不到、当场只放到环里最旧有效根，窄盘上写行取不到落点，在取号之前被拒
 //! （`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`；攻方第二次抽样 `research/prompts/m3-batch10-code-r1-opus-s2-output.md`
 //! K2-3 与「改法」一节：拒 36 → 28，降的正是这 8 段）。
@@ -13,16 +15,22 @@
 //! 读坏计划照攻方那四份（`P5:all/-:e`、`P5:-/all:e`、`P5:but1/all:z`、`P5:all/but1:g`）：只坏重建分配器读根环那一遍（根环每个槽按它自己
 //! 被读的次数从 1 数，第 6 次是那一遍：钩子调的那一刻每个槽恰读过 6 次现核），点名的槽这一次报读错（e）、读回全 0（z）或读回不是全 0 而自证不过的
 //! 乱字节（g）；`all` 是那块盘上的每个根环槽，`but1` 是除了那块盘上最新那条自证过的根所在的槽之外的每个根环槽。
+//! 那两个起点丢任一块盘之后都没有活文件（攻方第一次抽样 `research/prompts/m3-batch10-code-r2-opus-output.md` H2：丢盘整读比的是两个空表），
+//! 另加第三个起点 `264:6:4` 有活文件、可写挂载做完就崩；丢盘整读另比挂载之前环里每条不低于 F 的自证根（攻方 H2 的第二个改法）。
+//!
+//! 按槽并两遍的用例（攻方第二次抽样 `research/prompts/m3-batch10-code-r2-opus-s2-output.md` K2-2 那组历史）：384 槽两块盘，实例 1 第一个文件、
+//! 覆盖写一次，崩了再挂（实例 2）覆盖写一次得 C，再照 `tests/common` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`
+//! 那一形抛弃 C（实例 3），进程退出；再可写挂载时第一遍（第 6 次读）盘 1 每个根环槽报读错，重读（第 7 次读）照点名的读坏。
 
 mod common_admission;
 
 use common_admission::{
     checker_violations_on, plain_devices_on,
     start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
-    PoolUnderTest, OVERWRITE_BYTES,
+    start_plain, PoolUnderTest, FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
 };
 use singlefs_core::address::{
-    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, SlotNumber,
 };
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::block_device::{
@@ -35,13 +43,15 @@
     mount_writable_with_test_only_switches_and_deletion_switches, unmount, BeforeTheOneReread,
     MountError, Mounted, RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
 };
-use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read};
 use singlefs_core::mounted_session::MountedSession;
 use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
 use singlefs_core::recovery::{
-    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, RootRingSlotReading,
+    choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
+    verified_system_configuration_slots, RootRingSlotReading,
 };
-use singlefs_core::root_ring::{slot_offset, RootRingSlot};
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
 use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
 use singlefs_core::transaction::{
     publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
@@ -64,6 +74,10 @@
 /// `rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs` 那段历史同一个数，这里由钩子现核）。
 const READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD: u64 = 6;
 
+/// 重建分配器整环重读那一遍是每个槽的第几次读（紧接着重建那一遍）。
+const READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD: u64 =
+    READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD + 1;
+
 /// 读回乱字节那一类填的字节（攻方原型 `k.rs` 同一个值）：不是全 0，自证不过。
 const GARBAGE_BYTE: u8 = 0xA5;
 
@@ -136,12 +150,13 @@
         fault: RootRingSlotReadFault::ReadsBackGarbage,
     };
 
-/// 根环每个槽被读的次数与重建那一遍点名的槽（几块盘共用一份）。
+/// 根环每个槽被读的次数与点名读坏的那几次（几块盘共用一份）。
 struct RootRingReadsState {
     slot_spacing_in_bytes: u64,
     /// 每个根环槽在哪块盘、盘内从哪个偏移起。
     slots: Vec<(RootRingSlot, DeviceIdentity, u64)>,
-    faulted_slots: BTreeMap<RootRingSlot, RootRingSlotReadFault>,
+    /// （槽，这个槽第几次被读，从 1 数）→ 这一次读成什么。
+    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
     reads_of_each_slot: BTreeMap<RootRingSlot, u64>,
     faulted_reads: u64,
 }
@@ -150,7 +165,7 @@
 struct SharedRootRingReads(Rc<RefCell<RootRingReadsState>>);
 
 impl SharedRootRingReads {
-    /// 这一次读落在哪个根环槽里就给那个槽记一次；是重建那一遍点名的槽，交回这一次读成什么与那个槽在盘内的起点。
+    /// 这一次读落在哪个根环槽里就给那个槽记一次；是点名读坏的那一次，交回这一次读成什么与那个槽在盘内的起点。
     fn note_a_read_and_tell_its_fault(
         &self,
         device: DeviceIdentity,
@@ -170,10 +185,9 @@
             *reads += 1;
             *reads
         };
-        if reads_of_this_slot != READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD {
-            return None;
-        }
-        let fault = *state.faulted_slots.get(&ring_slot)?;
+        let fault = *state
+            .faulted_reads_of_each_slot
+            .get(&(ring_slot, reads_of_this_slot))?;
         state.faulted_reads += 1;
         Some((fault, slot_start))
     }
@@ -187,7 +201,7 @@
     }
 }
 
-/// 读的时候按 [`SharedRootRingReads`] 让重建那一遍点名的根环槽读坏的盘；写、写零、屏障原样交给里面那块盘。
+/// 读的时候按 [`SharedRootRingReads`] 让点名的那几次根环槽读读坏的盘；写、写零、屏障原样交给里面那块盘。
 struct DeviceFailingTheRebuildPass {
     identity: DeviceIdentity,
     inner: SparseBlockDevice,
@@ -258,13 +272,17 @@
     }
 }
 
-/// 攻方的两个起点。
+/// 攻方的两个起点，与一个丢任一块盘之后仍有活文件的起点。
 #[derive(Clone, Copy, Debug)]
 enum StartingPoint {
     /// `192:9:3:1` n = 8。
     UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
     /// `216:7:5:4` n = now。
     UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
+    /// `264:6:4`：不补环窗口、准入关着造的窄盘（单元区 264 槽），第一个文件之后覆盖写 6 次、顺序写长到 4 个单元就崩
+    /// （攻方第一次抽样 B 段的 `264:6:4-live-crash` 起点）；重开可写（准入判着、影子账开）做完就崩：两块盘上最新的根都是那次暖机的，
+    /// 带着同一个活文件。
+    UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
 }
 
 /// 再挂之前的盘面，与「卸载前」那一份（丢盘整读拿它比；n = now 那一个起点两份相同）。
@@ -423,6 +441,36 @@
                 parameters: pool.parameters.clone(),
             }
         }
+        StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount => {
+            let mut pool =
+                start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+                    HistoryDeviceWidth::UnitAreaOf384Slots,
+                    264,
+                );
+            // 迭代上界是 6 次覆盖写；没有提前出口。
+            for overwrite_index in 0..6 {
+                pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+                    panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
+                });
+            }
+            pool.sequential_write(3 * data_unit_payload_capacity() + 100)
+                .unwrap_or_else(|refusal| panic!("顺序写把文件长到 4 个单元：{refusal:?}"));
+            pool.session = None;
+            mount_writable_with_test_only_switches_and_deletion_switches(
+                &pool.parameters,
+                &mut pool.devices,
+                SpaceAdmission::JudgedByTheFormula,
+                ShadowLedger::On,
+                deletion_switches_with_batches_of_one(),
+            )
+            .unwrap_or_else(|error| panic!("崩了之后重开可写：{error:?}"));
+            let image = pool.image();
+            PoolBeforeTheRemount {
+                image_before_the_crash: image.clone(),
+                image_to_remount: image,
+                parameters: pool.parameters.clone(),
+            }
+        }
     }
 }
 
@@ -502,6 +550,90 @@
     Ok(live_files)
 }
 
+/// 在 `root` 那一版上打开、整读每个活文件（inode 号、全部字节）；删除中的 inode 不读。
+fn live_files_read_whole_at(
+    image: &MemoryPool,
+    root: &RootRecord,
+) -> Result<Vec<(u64, Vec<u8>)>, String> {
+    let mounted =
+        open_pool_for_read(image, root).map_err(|failure| format!("打开那一版：{failure:?}"))?;
+    let mut live_files = Vec::new();
+    // 迭代上界是这一版 inode 树的记录数；提前出口只有读不出的那两种错。
+    for record in mounted.inode_records() {
+        if record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {
+            continue;
+        }
+        let file = mounted
+            .open_file(image, InodeNumber(record.inode))
+            .map_err(|failure| format!("打开 inode {}：{failure:?}", record.inode))?;
+        let read = file
+            .read_at(image, FileOffsetInBytes(0), record.size)
+            .map_err(|failure| format!("读 inode {}：{failure:?}", record.inode))?;
+        live_files.push((record.inode, read.bytes));
+    }
+    Ok(live_files)
+}
+
+/// 一份镜像的根环里每条自证过的根，带它读出的那个槽。
+fn self_verified_roots_in_the_ring(image: &MemoryPool) -> Vec<(RootRingSlot, RootRecord)> {
+    let immutable = choose_system_configuration(image)
+        .expect("系统配置")
+        .immutable;
+    every_root_ring_slot(&immutable.sizes)
+        .into_iter()
+        .filter_map(|ring_slot| {
+            match read_root_ring_slot(
+                image,
+                &immutable.region_devices,
+                &immutable.sizes,
+                &immutable.filesystem_identifier,
+                ring_slot,
+            ) {
+                RootRingSlotReading::SelfVerified(root) => Some((ring_slot, root)),
+                RootRingSlotReading::Bad(_) => None,
+            }
+        })
+        .collect()
+}
+
+/// 挂载之前环里每条自证过的根，挂载之后还住在同一槽、txg 不低于挂载之后最新那条根带的 F（回退还能落到的），丢 `lost` 之后在剩下那块盘上
+/// 整读每个活文件，与挂载之前丢同一块盘读出的相同（挂载之前就读不出的不比）。交回比了几条根、其中有活文件的几条与读得不一样的那几条的 txg。
+fn roots_still_in_the_ring_read_on_a_lone_survivor(
+    image_before_the_remount: &MemoryPool,
+    image_after_the_remount: &MemoryPool,
+    lost: DeviceIdentity,
+) -> (usize, usize, Vec<u64>) {
+    let roots_after: Vec<(RootRingSlot, RootRecord)> =
+        self_verified_roots_in_the_ring(image_after_the_remount);
+    let floor_after = roots_after
+        .iter()
+        .map(|(_, root)| (root.checkpoint_txg, root.instance, root.rollback_floor))
+        .max()
+        .map_or(CheckpointTxg(0), |(_, _, rollback_floor)| rollback_floor);
+    let before_without = image_without(image_before_the_remount, lost);
+    let after_without = image_without(image_after_the_remount, lost);
+    let mut compared = 0;
+    let mut compared_with_live_files = 0;
+    let mut read_differently = Vec::new();
+    // 迭代上界是根环的槽数；没有提前出口。
+    for (ring_slot, root) in self_verified_roots_in_the_ring(image_before_the_remount) {
+        if root.checkpoint_txg < floor_after || !roots_after.contains(&(ring_slot, root)) {
+            continue;
+        }
+        let Ok(before) = live_files_read_whole_at(&before_without, &root) else {
+            continue;
+        };
+        compared += 1;
+        if !before.is_empty() {
+            compared_with_live_files += 1;
+        }
+        if live_files_read_whole_at(&after_without, &root) != Ok(before) {
+            read_differently.push(root.checkpoint_txg.0);
+        }
+    }
+    (compared, compared_with_live_files, read_differently)
+}
+
 /// 一份镜像去掉一块盘。
 fn image_without(image: &MemoryPool, lost: DeviceIdentity) -> MemoryPool {
     MemoryPool {
@@ -530,6 +662,9 @@
     checker_violations_with_both_devices: Vec<String>,
     /// 丢盘 0、丢盘 1 之后剩下那块盘只读挂载、整读每个活文件，与崩溃之前同一块盘上读出的相同。
     lone_survivor_reads_what_it_read_before_the_crash: [bool; 2],
+    /// 丢盘 0、丢盘 1 之后，挂载之前环里还在、不低于 F 的每条自证根在剩下那块盘上整读与挂载之前相同
+    /// （[`roots_still_in_the_ring_read_on_a_lone_survivor`]）。
+    lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [bool; 2],
 }
 
 /// 再挂一次（准入判着、影子账开），读盘经 `root_ring_reads`；重读之前调钩子记下那一刻每个根环槽读过几次。
@@ -579,7 +714,7 @@
 
 fn root_ring_reads_failing(
     image: &MemoryPool,
-    faulted_slots: BTreeMap<RootRingSlot, RootRingSlotReadFault>,
+    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
 ) -> SharedRootRingReads {
     let system_configuration = choose_system_configuration(image).expect("系统配置");
     let immutable = system_configuration.immutable;
@@ -596,7 +731,7 @@
     SharedRootRingReads(Rc::new(RefCell::new(RootRingReadsState {
         slot_spacing_in_bytes: u64::from(immutable.sizes.fixed_structure_slot_spacing),
         slots,
-        faulted_slots,
+        faulted_reads_of_each_slot,
         reads_of_each_slot: BTreeMap::new(),
         faulted_reads: 0,
     })))
@@ -607,9 +742,22 @@
     plan: RebuildPassPlan,
 ) -> RemountUnderThePlan {
     let before = pool_before_the_remount(starting_point);
-    let faulted_slots = faulted_slots_of_the_plan(&before.image_to_remount, plan);
-    let faulted_slot_count = u64::try_from(faulted_slots.len()).expect("槽数");
-    let root_ring_reads = root_ring_reads_failing(&before.image_to_remount, faulted_slots);
+    let faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> =
+        faulted_slots_of_the_plan(&before.image_to_remount, plan)
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
+    let faulted_slot_count = u64::try_from(faulted_reads_of_each_slot.len()).expect("槽数");
+    let root_ring_reads =
+        root_ring_reads_failing(&before.image_to_remount, faulted_reads_of_each_slot);
     let (mounted, image_after_the_remount, reads_at_each_hook_call) =
         remount_through(&before, &root_ring_reads);
     let (mounted_without_faults, image_after_a_remount_without_faults, _) = remount_through(
@@ -626,6 +774,15 @@
                 && live_files_read_whole(&image_without(&image_after_the_remount, lost))
                     == before_the_crash
         });
+    let lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount =
+        [DeviceIdentity(0), DeviceIdentity(1)].map(|lost| {
+            let (_, _, read_differently) = roots_still_in_the_ring_read_on_a_lone_survivor(
+                &before.image_to_remount,
+                &image_after_the_remount,
+                lost,
+            );
+            read_differently.is_empty()
+        });
     RemountUnderThePlan {
         root_ring_of_the_rebuilt_allocator: mounted
             .map(|mounted| mounted.output.rereads.root_ring_of_the_rebuilt_allocator)
@@ -645,11 +802,12 @@
             == image_after_a_remount_without_faults,
         checker_violations_with_both_devices: checker_violations_on(&image_after_the_remount),
         lone_survivor_reads_what_it_read_before_the_crash,
+        lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount,
     }
 }
 
-/// 只坏重建那一遍的计划在这两个起点上的预期：照常挂上、重读过一次（钩子恰调一次，在重建那一遍之后），交回与不注错的再挂同一份盘面，
-/// 两块盘都在时池级 checker 不判违例，丢盘整读与崩溃之前相同。
+/// 只坏重建那一遍的计划在这几个起点上的预期：照常挂上、重读过一次（钩子恰调一次，在重建那一遍之后），交回与不注错的再挂同一份盘面，
+/// 两块盘都在时池级 checker 不判违例，丢盘整读与崩溃之前相同，挂载之前环里还在、不低于 F 的每条自证根丢盘整读与挂载之前相同。
 fn rereads_and_mounts_like_a_remount_without_faults() -> RemountUnderThePlan {
     RemountUnderThePlan {
         root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
@@ -658,6 +816,7 @@
         same_disk_as_a_remount_without_faults: true,
         checker_violations_with_both_devices: Vec::new(),
         lone_survivor_reads_what_it_read_before_the_crash: [true, true],
+        lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [true, true],
     }
 }
 
@@ -783,3 +942,681 @@
         "只坏重建那一遍、盘 0 读回乱字节：整环重读、照常挂上"
     );
 }
+
+/// `264:6:4` 那个起点上同一份读坏计划之前，先核它真有可比的：挂载之前丢任一块盘，只读挂载择到的那一版有活文件，环里还在、不低于 F 的根里
+/// 也有带活文件的（不然丢盘整读两项比的又是空表）。
+fn assert_the_live_file_starting_point_has_live_files_to_compare(plan: RebuildPassPlan) {
+    let before = pool_before_the_remount(
+        StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
+    );
+    // 迭代上界是两块盘；没有提前出口。
+    for lost in [DeviceIdentity(0), DeviceIdentity(1)] {
+        let live_files =
+            live_files_read_whole(&image_without(&before.image_before_the_crash, lost))
+                .unwrap_or_else(|failure| {
+                    panic!("前提：丢盘 {} 之后只读挂载、整读做得成：{failure}", lost.0)
+                });
+        assert!(
+            live_files.iter().any(|(_, bytes)| !bytes.is_empty()),
+            "前提：丢盘 {} 之后剩下那块盘上有一个不空的活文件（{plan:?}）",
+            lost.0
+        );
+        let (compared, compared_with_live_files, _) =
+            roots_still_in_the_ring_read_on_a_lone_survivor(
+                &before.image_to_remount,
+                &before.image_to_remount,
+                lost,
+            );
+        assert!(
+            compared > 0 && compared_with_live_files > 0,
+            "前提：丢盘 {} 之后环里不低于 F 的根有 {compared} 条读得出，其中 {compared_with_live_files} 条带活文件",
+            lost.0
+        );
+    }
+}
+
+/// 再挂之后接着覆盖写几次（只读挂载择到的那一版之外，环里更旧的有效根引用的单元要在接着写的时候才会被发出去）。264 槽的窄盘上
+/// 第 12 次覆盖写抬过 F 仍放不下（`NoSpaceAfterRaisingTheFloor`，现量），取它之前的 10 次。
+const OVERWRITES_AFTER_THE_REMOUNT: usize = 10;
+
+/// `264:6:4` 那个起点上照计划只坏重建那一遍再挂，之后接着覆盖写 [`OVERWRITES_AFTER_THE_REMOUNT`] 次（准入关着：窄盘扣不起环窗口，起点就是
+/// 关着准入造的，落点是唯一的墙）；交回丢盘 0、丢盘 1 时 [`roots_still_in_the_ring_read_on_a_lone_survivor`] 比的（几条、带活文件的几条、
+/// 读得不一样的 txg）——挂载之前环里那几条根，写到这一刻仍在环里、不低于 F 的。
+fn lone_survivors_after_overwrites_following_the_remount(
+    plan: RebuildPassPlan,
+) -> [(usize, usize, Vec<u64>); 2] {
+    let before = pool_before_the_remount(
+        StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
+    );
+    let faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> =
+        faulted_slots_of_the_plan(&before.image_to_remount, plan)
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
+    let root_ring_reads =
+        root_ring_reads_failing(&before.image_to_remount, faulted_reads_of_each_slot);
+    let (mounted, image_after_the_remount, _) = remount_through(&before, &root_ring_reads);
+    let mounted = mounted.unwrap_or_else(|error| panic!("照计划再可写挂载做得成：{error:?}"));
+    let (_, mut session) = MountedSession::of_the_mount(mounted);
+    session
+        .allocator
+        .set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
+    let mut pool: PoolUnderTest<SparseBlockDevice> = PoolUnderTest {
+        parameters: before.parameters.clone(),
+        device_bytes: before.image_to_remount.device_size_in_bytes,
+        devices: plain_devices_on(&image_after_the_remount),
+        session: Some(session),
+        write_time_seconds: FIRST_WRITE_TIME_SECONDS + 100_000,
+        content_of_the_current_version: Vec::new(),
+        content_of_each_root: BTreeMap::new(),
+    };
+    // 迭代上界是 OVERWRITES_AFTER_THE_REMOUNT；没有提前出口。
+    for overwrite_index in 0..OVERWRITES_AFTER_THE_REMOUNT {
+        pool.overwrite(OVERWRITE_BYTES)
+            .unwrap_or_else(|refusal| panic!("再挂之后第 {overwrite_index} 次覆盖写：{refusal:?}"));
+    }
+    let image_after_the_overwrites = pool.image();
+    [DeviceIdentity(0), DeviceIdentity(1)].map(|lost| {
+        roots_still_in_the_ring_read_on_a_lone_survivor(
+            &before.image_to_remount,
+            &image_after_the_overwrites,
+            lost,
+        )
+    })
+}
+
+/// 接着覆盖写之后的预期：丢任一块盘，挂载之前环里还在、不低于 F 的根都比到了（至少一条、其中至少一条带活文件），没有一条读得不一样。
+fn assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
+    compared_on_each_lone_survivor: [(usize, usize, Vec<u64>); 2],
+) {
+    // 迭代上界是两块盘；没有提前出口。
+    for (lost, (compared, compared_with_live_files, read_differently)) in
+        [0u32, 1].into_iter().zip(compared_on_each_lone_survivor)
+    {
+        assert!(
+            compared > 0 && compared_with_live_files > 0,
+            "丢盘 {lost}：环里还在、不低于 F 的根比了 {compared} 条，其中 {compared_with_live_files} 条带活文件（要比到带活文件的根）"
+        );
+        assert_eq!(
+            read_differently,
+            Vec::<u64>::new(),
+            "丢盘 {lost}：再挂、接着覆盖写之后，挂载之前环里还在、不低于 F 的根在剩下那块盘上整读与挂载之前相同（读得不一样的根的 txg）"
+        );
+    }
+}
+
+/// 场景：`264:6:4`（有活文件，可写挂载做完就崩），重建那一遍盘 0 的每个根环槽报读错（`P5:all/-:e`）。
+/// 预期：整环重读一次、照常挂上，与不注错的再挂同一份盘面；丢任一块盘之后剩下那块盘只读挂载整读与崩溃之前相同（这一回比的是一个不空的活文件），
+/// 挂载之前环里还在、不低于 F 的每条自证根丢盘整读也与挂载之前相同；再接着覆盖写 [`OVERWRITES_AFTER_THE_REMOUNT`] 次之后，
+/// 挂载之前环里仍在、不低于 F 的根丢盘整读还与挂载之前相同。
+/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「第10h批 rr2 用例丢盘整读：重读之后的读数只剩最新那条根…」）时
+/// 回收门槛与当场放开越过环里更旧的有效根，接着覆盖写把它们引用的单元写掉，接着写之后那一项先红（它排在前面；后面那一项里两块盘都在时
+/// 池级 checker 也判 I-3.1 记账少于遍历）。
+#[test]
+fn on_the_264_slot_pool_with_a_live_file_device_zero_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
+) {
+    assert_the_live_file_starting_point_has_live_files_to_compare(DEVICE_ZERO_REPORTS_READ_ERRORS);
+    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
+        lone_survivors_after_overwrites_following_the_remount(DEVICE_ZERO_REPORTS_READ_ERRORS),
+    );
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
+            DEVICE_ZERO_REPORTS_READ_ERRORS
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "有活文件的起点、只坏重建那一遍、盘 0 报读错：整环重读、照常挂上，丢盘整读与之前相同"
+    );
+}
+
+/// 场景：`264:6:4`，重建那一遍盘 1 的每个根环槽报读错（`P5:-/all:e`）。
+/// 预期与判别力同上一条。
+#[test]
+fn on_the_264_slot_pool_with_a_live_file_device_one_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
+) {
+    assert_the_live_file_starting_point_has_live_files_to_compare(DEVICE_ONE_REPORTS_READ_ERRORS);
+    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
+        lone_survivors_after_overwrites_following_the_remount(DEVICE_ONE_REPORTS_READ_ERRORS),
+    );
+    assert_eq!(
+        remount_under_the_plan(
+            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
+            DEVICE_ONE_REPORTS_READ_ERRORS
+        ),
+        rereads_and_mounts_like_a_remount_without_faults(),
+        "有活文件的起点、只坏重建那一遍、盘 1 报读错：整环重读、照常挂上，丢盘整读与之前相同"
+    );
+}
+
+/// K2-2 那组历史停在进程退出那一刻：盘面、参数，被抛弃的根 C、它住的根环槽，与 C 写出的每个落点（盘、槽、那时盘上的字节）。
+struct PoolWithAnAbandonedRoot {
+    image: MemoryPool,
+    parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
+    device_bytes: u64,
+    write_time_seconds: u64,
+    abandoned_root: RootRecord,
+    ring_slot_of_the_abandoned_root: RootRingSlot,
+    places_the_abandoned_root_wrote: Vec<(DeviceIdentity, SlotNumber, Vec<u8>)>,
+}
+
+fn bytes_on(
+    devices: &[(DeviceIdentity, SparseBlockDevice)],
+    device: DeviceIdentity,
+    offset: DeviceOffsetInBytes,
+    length: usize,
+) -> Vec<u8> {
+    let (_, block_device) = devices
+        .iter()
+        .find(|(identity, _)| *identity == device)
+        .expect("池里有这块盘");
+    let mut bytes = vec![0u8; length];
+    block_device
+        .read_at(offset, &mut bytes)
+        .expect("内存盘读得出");
+    bytes
+}
+
+fn write_on(
+    devices: &mut [(DeviceIdentity, SparseBlockDevice)],
+    device: DeviceIdentity,
+    offset: DeviceOffsetInBytes,
+    bytes: &[u8],
+) {
+    let (_, block_device) = devices
+        .iter_mut()
+        .find(|(identity, _)| *identity == device)
+        .expect("池里有这块盘");
+    block_device
+        .write_at(offset, bytes, WriteDurability::Plain)
+        .expect("内存盘写得进");
+}
+
+/// 384 槽两块盘：第一个文件、覆盖写一次（实例 1）；崩了再挂（实例 2），覆盖写一次得 C；照 `tests/common` 的
+/// `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 那一形抛弃 C——C 的根槽与数据单元暂存、清零，见证 C 的那一次系统配置轮换
+/// 写回成轮换之前的那一代，崩了再挂（实例 3）择根落到 C 之前那一条、写行与暖机，再把暂存的字节写回；进程退出。
+fn pool_with_an_abandoned_root() -> PoolWithAnAbandonedRoot {
+    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
+    pool.overwrite(OVERWRITE_BYTES).expect("实例 1 覆盖写一次");
+    pool.crash_and_mount_writable().expect("崩了再挂：取号 2");
+    pool.overwrite(OVERWRITE_BYTES - 499)
+        .expect("实例 2 覆盖写一次得 C");
+    let abandoned = pool.current_file_version().clone();
+    let geometry = pool.parameters.geometry;
+    let ring_slot_of_the_abandoned_root = target_for_publish(
+        abandoned.root.checkpoint_txg,
+        geometry.root_ring_slots_per_region,
+    );
+    let root_slot_device = pool.parameters.region_devices
+        [usize::try_from(ring_slot_of_the_abandoned_root.region).expect("区域号")];
+    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
+    let places_to_hide: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)> = std::iter::once((
+        root_slot_device,
+        slot_offset(
+            ring_slot_of_the_abandoned_root,
+            geometry.fixed_structure_slot_spacing,
+        ),
+        usize::try_from(geometry.physical_block_size).expect("根槽宽"),
+    ))
+    .chain(
+        abandoned
+            .data_pointers
+            .iter()
+            .flat_map(|pointer| pointer.locations)
+            .map(|location| {
+                (
+                    location.device,
+                    location.slot.to_device_offset(),
+                    data_unit_bytes,
+                )
+            }),
+    )
+    .collect();
+    let saved: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = places_to_hide
+        .iter()
+        .map(|(device, offset, length)| {
+            let bytes = bytes_on(&pool.devices, *device, *offset, *length);
+            write_on(&mut pool.devices, *device, *offset, &vec![0u8; *length]);
+            (*device, *offset, bytes)
+        })
+        .collect();
+    // 见证 C 的那一次轮换（每块盘世代号最大的那一槽）写回成轮换之前的那一代：另一槽的内容、世代号减 2（与 `tests/common` 那一形同一个造法）。
+    let slot_spacing_in_bytes = u64::from(geometry.fixed_structure_slot_spacing);
+    let rotations_not_written: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = pool
+        .devices
+        .iter()
+        .map(|(identity, _)| {
+            let mut verified_slots = verified_system_configuration_slots(
+                pool.devices.as_slice(),
+                *identity,
+                slot_spacing_in_bytes,
+                &pool.parameters.filesystem_identifier,
+            );
+            verified_slots.sort_by_key(|slot| slot.quantities.slot_generation);
+            let [older, newest] =
+                <[_; 2]>::try_from(verified_slots).expect("造被抛弃的根之前每块盘两槽都自证得过");
+            let newest_generation = newest.quantities.slot_generation;
+            let mut rotation_not_written = older;
+            rotation_not_written.quantities.slot_generation = newest_generation
+                .checked_sub(2)
+                .expect("C 之前至少还有两次轮换");
+            (
+                *identity,
+                DeviceOffsetInBytes(
+                    (newest_generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
+                        * slot_spacing_in_bytes,
+                ),
+                rotation_not_written.to_slot(),
+            )
+        })
+        .collect();
+    // 迭代上界是两块盘；没有提前出口。
+    for (device, offset, bytes) in &rotations_not_written {
+        write_on(&mut pool.devices, *device, *offset, bytes);
+    }
+    let abandoning = pool
+        .crash_and_mount_writable()
+        .expect("C 的根槽与数据单元读不出：择根落到 C 之前那一条，照常可写挂载（实例 3）");
+    assert!(
+        abandoning.effective_root.checkpoint_txg < abandoned.root.checkpoint_txg,
+        "前提：实例 3 落到 C 之前"
+    );
+    // 迭代上界是暂存的落点数；没有提前出口。
+    for (device, offset, bytes) in &saved {
+        write_on(&mut pool.devices, *device, *offset, bytes);
+    }
+    pool.session = None;
+    let mut places_the_abandoned_root_wrote: Vec<(DeviceIdentity, SlotNumber, Vec<u8>)> =
+        Vec::new();
+    // 迭代上界是 C 写出的单元数 × 两块盘；没有提前出口。
+    for unit in &abandoned.units {
+        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+            let on_disk = bytes_on(
+                &pool.devices,
+                device,
+                unit.slot.to_device_offset(),
+                unit.bytes.len(),
+            );
+            if on_disk == unit.bytes {
+                places_the_abandoned_root_wrote.push((device, unit.slot, on_disk));
+            }
+        }
+    }
+    // 迭代上界是 C 的数据指针数 × 两个位置；没有提前出口。
+    for location in abandoned
+        .data_pointers
+        .iter()
+        .flat_map(|pointer| pointer.locations)
+    {
+        if !places_the_abandoned_root_wrote
+            .iter()
+            .any(|(device, slot, _)| *device == location.device && *slot == location.slot)
+        {
+            let on_disk = bytes_on(
+                &pool.devices,
+                location.device,
+                location.slot.to_device_offset(),
+                data_unit_bytes,
+            );
+            places_the_abandoned_root_wrote.push((location.device, location.slot, on_disk));
+        }
+    }
+    PoolWithAnAbandonedRoot {
+        image: pool.image(),
+        parameters: pool.parameters.clone(),
+        device_bytes: pool.device_bytes,
+        write_time_seconds: pool.write_time_seconds,
+        abandoned_root: abandoned.root,
+        ring_slot_of_the_abandoned_root,
+        places_the_abandoned_root_wrote,
+    }
+}
+
+/// K2-2 那组历史上再可写挂载一次、之后接着覆盖写到 C 离开根环，看到的那几样（各用例自己断言）。
+#[derive(Debug, PartialEq, Eq)]
+struct RemountOfThePoolWithAnAbandonedRoot {
+    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
+    root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
+    /// 重读之前的钩子调了几次，每次调的那一刻每个根环槽是不是都恰读过 [`READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD`] 次。
+    hook_calls_after_exactly_the_rebuild_pass: Vec<bool>,
+    /// 读坏的恰是点名的那几次、各一次。
+    faulted_reads_are_the_named_ones: bool,
+    /// 交回的盘面与同一份盘面上照对照的读坏计划再挂交回的逐字节相同。
+    same_disk_as_the_reference_remount: bool,
+    /// 挂上那一刻两块盘都隔离了只被被抛弃根引用的槽（影子账不为 0）。
+    both_devices_isolate_slots_after_the_mount: bool,
+    /// 挂上那一刻 C 写出的落点没有一个在分配器里是空闲的。
+    no_place_of_the_abandoned_root_is_free_after_the_mount: bool,
+    /// 接着覆盖写到 C 的根槽被盖掉（C 离开根环）为止，每次发布之后 C 写出的每个落点在盘上仍是 C 写的字节。
+    places_of_the_abandoned_root_kept_until_it_leaves_the_ring: bool,
+    /// C 离开根环那一刻两块盘都在时池级 checker 判违例的那几条。
+    checker_violations_when_the_abandoned_root_leaves_the_ring: Vec<String>,
+}
+
+/// 照一份读坏计划再挂一次交回的：挂载的结局、挂载之后的盘面、钩子每次调的那一刻每个根环槽读过几次、这一次读坏了几次。
+struct RemountThroughFaultedReads {
+    mounted: Result<Mounted, MountError>,
+    image_after_the_remount: MemoryPool,
+    reads_of_each_slot_at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>>,
+    faulted_reads: u64,
+}
+
+/// 照 `faulted_reads_of_each_slot` 读坏地再可写挂载一次（准入判着、影子账开），重读之前调钩子记下那一刻每个根环槽读过几次。
+fn remount_the_pool_with_an_abandoned_root(
+    history: &PoolWithAnAbandonedRoot,
+    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
+) -> RemountThroughFaultedReads {
+    let root_ring_reads = root_ring_reads_failing(&history.image, faulted_reads_of_each_slot);
+    let before = PoolBeforeTheRemount {
+        image_before_the_crash: history.image.clone(),
+        image_to_remount: history.image.clone(),
+        parameters: history.parameters.clone(),
+    };
+    let (mounted, image_after_the_remount, reads_of_each_slot_at_each_hook_call) =
+        remount_through(&before, &root_ring_reads);
+    RemountThroughFaultedReads {
+        mounted,
+        image_after_the_remount,
+        reads_of_each_slot_at_each_hook_call,
+        faulted_reads: root_ring_reads.faulted_reads(),
+    }
+}
+
+/// 根环里这一槽这一刻是不是还住着 C。
+fn abandoned_root_still_in_the_ring(image: &MemoryPool, history: &PoolWithAnAbandonedRoot) -> bool {
+    self_verified_roots_in_the_ring(image).contains(&(
+        history.ring_slot_of_the_abandoned_root,
+        history.abandoned_root,
+    ))
+}
+
+/// 在 K2-2 那组历史上照 `faulted_reads_of_each_slot` 再挂、接着覆盖写到 C 离开根环；盘面拿 `reference_faulted_reads_of_each_slot` 那一次再挂比。
+fn remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
+    history: &PoolWithAnAbandonedRoot,
+    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
+    reference_faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
+) -> RemountOfThePoolWithAnAbandonedRoot {
+    let named_faulted_reads = u64::try_from(faulted_reads_of_each_slot.len()).expect("次数");
+    let RemountThroughFaultedReads {
+        mounted,
+        image_after_the_remount,
+        reads_of_each_slot_at_each_hook_call: reads_at_each_hook_call,
+        faulted_reads,
+    } = remount_the_pool_with_an_abandoned_root(history, faulted_reads_of_each_slot);
+    let reference =
+        remount_the_pool_with_an_abandoned_root(history, reference_faulted_reads_of_each_slot);
+    let image_after_the_reference_remount = reference.image_after_the_remount;
+    reference
+        .mounted
+        .unwrap_or_else(|error| panic!("前提：照对照的读坏计划再可写挂载做得成：{error:?}"));
+    let hook_calls_after_exactly_the_rebuild_pass: Vec<bool> = reads_at_each_hook_call
+        .iter()
+        .map(|reads_of_each_slot| {
+            !reads_of_each_slot.is_empty()
+                && reads_of_each_slot
+                    .values()
+                    .all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD)
+        })
+        .collect();
+    let faulted_reads_are_the_named_ones =
+        named_faulted_reads > 0 && faulted_reads == named_faulted_reads;
+    let same_disk_as_the_reference_remount =
+        image_after_the_remount == image_after_the_reference_remount;
+    let mounted = match mounted {
+        Ok(mounted) => mounted,
+        Err(error) => {
+            return RemountOfThePoolWithAnAbandonedRoot {
+                root_ring_of_the_rebuilt_allocator: Err(format!("{error:?}")),
+                hook_calls_after_exactly_the_rebuild_pass,
+                faulted_reads_are_the_named_ones,
+                same_disk_as_the_reference_remount,
+                both_devices_isolate_slots_after_the_mount: false,
+                no_place_of_the_abandoned_root_is_free_after_the_mount: false,
+                places_of_the_abandoned_root_kept_until_it_leaves_the_ring: false,
+                checker_violations_when_the_abandoned_root_leaves_the_ring: Vec::new(),
+            };
+        }
+    };
+    let root_ring_of_the_rebuilt_allocator =
+        Ok(mounted.output.rereads.root_ring_of_the_rebuilt_allocator);
+    let both_devices_isolate_slots_after_the_mount = mounted.output.isolated_slots_per_device.len()
+        == 2
+        && mounted
+            .output
+            .isolated_slots_per_device
+            .iter()
+            .all(|(_, isolated_slots)| *isolated_slots > 0);
+    let no_place_of_the_abandoned_root_is_free_after_the_mount = history
+        .places_the_abandoned_root_wrote
+        .iter()
+        .all(|(device, slot, _)| {
+            !mounted.allocator.devices[usize::try_from(device.0).expect("盘号")].is_free(*slot)
+        });
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    let mut pool: PoolUnderTest<SparseBlockDevice> = PoolUnderTest {
+        parameters: history.parameters.clone(),
+        device_bytes: history.device_bytes,
+        devices: plain_devices_on(&image_after_the_remount),
+        session: Some(session),
+        write_time_seconds: history.write_time_seconds + 100,
+        content_of_the_current_version: Vec::new(),
+        content_of_each_root: BTreeMap::new(),
+    };
+    let root_ring_slot_count = every_root_ring_slot(&history.parameters.geometry).len();
+    let mut places_of_the_abandoned_root_kept_until_it_leaves_the_ring = true;
+    let mut overwrites = 0usize;
+    // 迭代上界是根环的槽数（再挂写了几条根，C 的根槽在一圈之内被盖掉）；提前出口：C 的哪个落点被改写了就停。
+    while abandoned_root_still_in_the_ring(&pool.image(), history) {
+        assert!(
+            overwrites < root_ring_slot_count,
+            "C 的根槽在一圈（{root_ring_slot_count} 次发布）之内被盖掉"
+        );
+        pool.overwrite(OVERWRITE_BYTES)
+            .unwrap_or_else(|refusal| panic!("再挂之后第 {overwrites} 次覆盖写：{refusal:?}"));
+        overwrites += 1;
+        if history
+            .places_the_abandoned_root_wrote
+            .iter()
+            .any(|(device, slot, bytes)| {
+                bytes_on(&pool.devices, *device, slot.to_device_offset(), bytes.len()) != *bytes
+            })
+        {
+            places_of_the_abandoned_root_kept_until_it_leaves_the_ring = false;
+            break;
+        }
+    }
+    RemountOfThePoolWithAnAbandonedRoot {
+        root_ring_of_the_rebuilt_allocator,
+        hook_calls_after_exactly_the_rebuild_pass,
+        faulted_reads_are_the_named_ones,
+        same_disk_as_the_reference_remount,
+        both_devices_isolate_slots_after_the_mount,
+        no_place_of_the_abandoned_root_is_free_after_the_mount,
+        places_of_the_abandoned_root_kept_until_it_leaves_the_ring,
+        checker_violations_when_the_abandoned_root_leaves_the_ring: checker_violations_on(
+            &pool.image(),
+        ),
+    }
+}
+
+/// 盘 1 那几个区域的每个根环槽。
+fn root_ring_slots_of_device_one(history: &PoolWithAnAbandonedRoot) -> Vec<RootRingSlot> {
+    every_root_ring_slot(&history.parameters.geometry)
+        .into_iter()
+        .filter(|ring_slot| {
+            history.parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")]
+                == DeviceIdentity(1)
+        })
+        .collect()
+}
+
+/// 每个点名的槽在第 `read_of_each_slot` 次读时读成 `fault`。
+fn faulted_reads_of(
+    ring_slots: &[RootRingSlot],
+    read_of_each_slot: u64,
+    fault: RootRingSlotReadFault,
+) -> BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> {
+    ring_slots
+        .iter()
+        .map(|ring_slot| ((*ring_slot, read_of_each_slot), fault))
+        .collect()
+}
+
+/// 并两遍之后的预期：重读过一次（钩子恰调一次，在重建那一遍之后）、照常挂上，与对照那一次再挂同一份盘面；两块盘都隔离、C 的落点一个都不空闲，
+/// 接着覆盖写到 C 离开根环之前 C 写出的落点一个都没被改写，那一刻池级 checker 不判违例。
+fn merged_and_keeps_the_abandoned_root_isolated() -> RemountOfThePoolWithAnAbandonedRoot {
+    RemountOfThePoolWithAnAbandonedRoot {
+        root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
+        hook_calls_after_exactly_the_rebuild_pass: vec![true],
+        faulted_reads_are_the_named_ones: true,
+        same_disk_as_the_reference_remount: true,
+        both_devices_isolate_slots_after_the_mount: true,
+        no_place_of_the_abandoned_root_is_free_after_the_mount: true,
+        places_of_the_abandoned_root_kept_until_it_leaves_the_ring: true,
+        checker_violations_when_the_abandoned_root_leaves_the_ring: Vec::new(),
+    }
+}
+
+/// 第一遍盘 1 整段报读错、重读只坏 C 的槽（读成 `fault`）的那一格（攻方 `pass1:dev1-all-e reread:c-*`）。
+fn first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+    fault: RootRingSlotReadFault,
+) -> RemountOfThePoolWithAnAbandonedRoot {
+    let history = pool_with_an_abandoned_root();
+    let mut faulted_reads = faulted_reads_of(
+        &root_ring_slots_of_device_one(&history),
+        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
+        RootRingSlotReadFault::ReadFails,
+    );
+    faulted_reads.extend(faulted_reads_of(
+        &[history.ring_slot_of_the_abandoned_root],
+        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
+        fault,
+    ));
+    remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
+        &history,
+        faulted_reads,
+        BTreeMap::new(),
+    )
+}
+
+/// 场景：K2-2 那组历史，第一遍盘 1 每个根环槽报读错（rr2 整环重读），重读那一遍只有 C 的根槽报读错。
+/// 预期：按槽并两遍，C 那一槽留第一遍读出的 C——影子账照常隔离 C 独占的单元，盘面与不注错的再挂相同；接着覆盖写到 C 离开根环，
+/// C 写出的落点一个都没被改写，池级 checker 不红（D23（journal 的角色与格式） 已定项 14「被崩溃恢复抛弃的时间线的根离开根环之前，
+/// 它们引用的单元不许重新分配也不许抹头」）。
+/// 判别力：改回整份换成重读的读数（`crates/mutations.tsv`「第10h批 R1 改回整份换成重读的读数」）时 C 掉出影子账、两块盘隔离 0，
+/// 第一次覆盖写就复用 C 的单元（攻方量过 I-7.4 红），隔离那一项起红；「第10h批 R1 第一遍自证过压不过重读读坏」同样红。
+#[test]
+fn the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            RootRingSlotReadFault::ReadFails
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "重读只坏 C 的根槽（报读错）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：同上，重读那一遍 C 的根槽读回全 0。
+/// 预期与判别力同上一条（第一遍自证过压过重读全零）。
+#[test]
+fn the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            RootRingSlotReadFault::ReadsBackAllZero
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "重读只坏 C 的根槽（读回全 0）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：同上，重读那一遍 C 的根槽读回乱字节。
+/// 预期与判别力同上一条。
+#[test]
+fn the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
+) {
+    assert_eq!(
+        first_pass_failing_device_one_and_the_reread_failing_only_the_abandoned_roots_slot(
+            RootRingSlotReadFault::ReadsBackGarbage
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "重读只坏 C 的根槽（读回乱字节）：并两遍留第一遍的 C，影子账照隔离"
+    );
+}
+
+/// 场景：K2-2 的反方向（攻方 `pass1:dev1-all-e+c-e reread:-`）：第一遍盘 1 每个根环槽与 C 的根槽都报读错，重读那一遍都读得出。
+/// 预期：按槽并两遍，C 那一槽取重读读出的 C，与不注错的再挂同一份盘面，影子账照隔离，C 离开根环之前它的落点不被改写，池级 checker 不红。
+/// 判别力：并的时候第一遍读坏压过重读自证过的根（`crates/mutations.tsv`「第10h批 R1 第一遍读坏压过重读自证过的根」）时 C 掉出影子账，
+/// 隔离那一项起红。
+#[test]
+fn the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring(
+) {
+    let history = pool_with_an_abandoned_root();
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
+    assert_eq!(
+        remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
+            &history,
+            faulted_reads,
+            BTreeMap::new(),
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "第一遍 C 的根槽也读坏、重读读得出：并两遍取重读的 C，影子账照隔离"
+    );
+}
+
+/// 场景：第一遍盘 1 每个根环槽报读错（rr2 整环重读），重读那一遍整个环都报读错（攻方 `pass1:dev1-all-e reread:all-e`，判决 K2「多拒」那一格）。
+/// 预期：按槽并两遍，第一遍盘 0 读出的根都留着，照常挂上（不是 Q3 那一拒），与「两遍都只坏盘 1」的再挂同一份盘面（两份并出来的读数相同：
+/// 盘 0 读出、盘 1 读坏，照 m 取不到那一支往下走）；影子账照隔离 C，C 离开根环之前它的落点不被改写，池级 checker 不红。
+/// 判别力：改回整份换成重读的读数时重读那一遍一条有效根都没有，拒成 `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，第一项红。
+#[test]
+fn a_reread_failing_on_the_whole_ring_after_a_first_pass_that_read_device_zero_mounts_on_the_first_pass_roots(
+) {
+    let history = pool_with_an_abandoned_root();
+    let ring_slots_of_device_one = root_ring_slots_of_device_one(&history);
+    let mut faulted_reads = faulted_reads_of(
+        &ring_slots_of_device_one,
+        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
+        RootRingSlotReadFault::ReadFails,
+    );
+    faulted_reads.extend(faulted_reads_of(
+        &every_root_ring_slot(&history.parameters.geometry),
+        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
+        RootRingSlotReadFault::ReadFails,
+    ));
+    let mut faulted_reads_only_on_device_one = faulted_reads_of(
+        &ring_slots_of_device_one,
+        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
+        RootRingSlotReadFault::ReadFails,
+    );
+    faulted_reads_only_on_device_one.extend(faulted_reads_of(
+        &ring_slots_of_device_one,
+        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
+        RootRingSlotReadFault::ReadFails,
+    ));
+    assert_eq!(
+        remount_the_pool_with_an_abandoned_root_and_overwrite_until_it_leaves_the_ring(
+            &history,
+            faulted_reads,
+            faulted_reads_only_on_device_one,
+        ),
+        merged_and_keeps_the_abandoned_root_isolated(),
+        "重读整环都坏、第一遍读出了盘 0 的根：并两遍照常挂上"
+    );
+}
diff --git a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
--- a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
+++ b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
@@ -1,5 +1,5 @@
-//! 重建分配器读根环那一遍一条不被抛弃的自证有效根都没读出（择根那几遍读得出）时，把根环整环再读一遍，根环表、回收门槛与影子账用重读的读数；
-//! 重读还是一条都没有，就在任何写之前拒这次可写挂载（`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，用户 2026-10-05 定
+//! 重建分配器读根环那一遍一条不被抛弃的自证有效根都没读出（择根那几遍读得出）时，把根环整环再读一遍，根环表、回收门槛与影子账用两遍逐槽
+//! 并起来的读数（用户 2026-10-05 定 R1「按槽并两遍」；第一遍整环读坏时并起来就是重读那一遍）；并起来还是一条都没有，就在任何写之前拒这次可写挂载（`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，用户 2026-10-05 定
 //! Q3 另起拒因，之前报 `Recovery(NoValidRoot)`）。只有一块盘没读出时也整环重读（Q4 rr2）的用例在
 //! `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs`。用户 2026-10-05 定「重读」：
 //! 岔路单 `research/prompts/m3-batch17-code-r1-forks.md` 第 2 行，判决 `research/prompts/m3-batch17-code-r1-main-verification.md`
@@ -815,7 +815,7 @@
 }
 
 /// 重读那一遍只有一部分槽读坏（交主 agent 的那一格，能造的一形）：重建分配器那一遍两块盘全部根环槽报读错，重读那一遍盘 1 的根环槽仍报读错、
-/// 盘 0 的读得出。重读的读数与「重建那一遍与重读都只坏盘 1」（rr2 之后只坏盘 1 的那一遍也整环重读，Q4）同等对待：挂载做成、交回的盘面与
+/// 盘 0 的读得出。两遍并起来的读数与「重建那一遍与重读都只坏盘 1」（rr2 之后只坏盘 1 的那一遍也整环重读，Q4）逐槽相同：挂载做成、交回的盘面与
 /// 两遍都只坏盘 1 的再挂逐字节相同（两者都重读过、重读那一遍都只读出盘 0 的根，照 m 取不到那一支往下走），两块盘都在时池级 checker 不判违例。
 /// 判别力：删掉重读时第一项红（Q3 那一拒）；不重读、照旧给空表往下走时同上一形，挂载落了盘之后报
 /// `FloorRaiseFailedAfterTheMountsPublishes`，第一项红。
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs b/crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs
--- a/crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs
@@ -1,17 +1,25 @@
-//! 重建分配器读根环有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，重读那一遍整个环仍一条都没有：拒可写的拒因另起成员
+//! 重建分配器读根环有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有：拒可写的拒因另起成员
 //! （`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`），不复用读阶段择不到根的 `Recovery(NoValidRoot)`；
 //! D23（journal 的角色与格式） 已定项 14 那张单子加一行（用户 2026-10-05 定 Q3，岔路单 `research/prompts/m3-batch10-code-r1-forks.md`，
 //! 判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第六节）。判法在崩溃镜像上复算「按最新那条根的实例表，环里一条不被抛弃的
 //! 自证有效根都没有」，与改法 G4 同形：注入了读故障的崩溃态上不判那一条；没注入读故障的崩溃态上复算为真，拒得不合法
 //! （`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`）。
+//! 两遍按槽并（用户 2026-10-05 定 R1，判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第六节）之后这一拒的判据在崩溃镜像上
+//! 照旧只读一遍：崩溃镜像上没有瞬时读错，两遍读出的逐槽相同，并起来就是那一遍；所以复算判据不变。
 //! 盘面是 24 槽根环的小池（`common::build_pool`，第一个文件之后）；手工盘面是把它根环三个区域的每一个槽都清成全 0。
 
 mod common;
 
-use common::build_pool;
+use common::{
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
+    publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS,
+};
+use singlefs_core::address::InstanceGeneration;
 use singlefs_core::mount::MountError;
 use singlefs_core::recovery::{
-    choose_system_configuration, every_root_ring_slot, recover, JournalPolicy, RecoveryFailure,
+    choose_root, choose_system_configuration, every_root_ring_slot, instance_table_of_root,
+    readable_roots, recover, root_is_abandoned_by_the_instance_table, JournalPolicy,
+    RecoveryFailure,
 };
 use singlefs_core::root_ring::slot_offset;
 use singlefs_harness::memory_pool::MemoryPool;
@@ -185,3 +193,109 @@
         "被拒的那次挂载在读故障下跑：两遍读之间的瞬时读错复算不出，判据复算为假不算不合法"
     );
 }
+
+/// 崩溃恢复抛弃了一条根的盘面（与 `a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs` 的
+/// `pool_after_a_recovery_abandoned_the_third_version` 同一段脚本）：A、B（实例 1）→ 重开取号 2、写行（txg 5）、暖机（6、7）→ C（txg 8）→
+/// 崩溃恢复抛弃 C（系统配置没见证 C，实例 3 写行 9、暖机 10），C 的根槽写回、又读得出。
+fn image_with_an_abandoned_root(tag: &str) -> MemoryPool {
+    let mut pool = build_pool(tag);
+    let first = pool.output.clone();
+    pool.output = publish_overwrite_in_process(
+        &mut pool,
+        &first,
+        &[0x5A; 4100],
+        FIXED_WRITE_TIME_SECONDS + 60,
+        InstanceGeneration(1),
+    )
+    .expect("B");
+    let mut devices = pool.reopen_recorded();
+    let mounted = singlefs_core::mount::mount_writable(&common::parameters(), &mut devices)
+        .expect("B 之后重开，可写挂载（实例 2）");
+    pool.devices = Some(devices);
+    pool.allocator = mounted.allocator;
+    pool.output = mounted
+        .current
+        .into_file_version()
+        .expect("B 之后重开，现行那一版带文件");
+    let before_the_abandoned_publish = pool.output.clone();
+    let abandoned = publish_overwrite_in_process(
+        &mut pool,
+        &before_the_abandoned_publish,
+        &[0xC3; 2500],
+        FIXED_WRITE_TIME_SECONDS + 60,
+        InstanceGeneration(2),
+    )
+    .expect("C");
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned);
+    pool.memory_pool()
+}
+
+/// 场景：崩溃恢复抛弃了一条根 C 的盘面，根环里有实例 0（mkfs）到实例 3 的根，最新那条根（实例 3）指着的实例表有实例 1、2 的行；
+/// 崩溃态没注入读故障，被拒的那次挂载读的就是这份镜像；拿 Q3 那一拒交给判官。
+/// 预期：判据在镜像上复算为假（C 按表被抛弃，别的根有效），拒的理由是错的。判官「按最新那条根的实例表判被抛弃」那半句在合法盘面上不改判据的值：
+/// 写行只给比要取的号小的实例写行（`mount.rs` 的 `instance_rows_to_write`），最新那条根所在的实例在它自己的表里没有行，它按表恒不被抛弃，
+/// 判据恒假——前两项断言把这一点钉住。
+/// 留档（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第二行）：正推 `research/prompts/m3-batch10-code-r2-sonnet-output.md` K1-3
+/// 把那半句换成「表里有该实例的行就算被抛弃」、5 条仍全绿，在合法盘面上是等价变异，这一条在它下面照样绿（推的：只在本仓写出来的实例表上成立，
+/// 手工改出一张给最新那条根自己的实例写了行的表才分得开，那不是合法盘面）。换成会改行为的那一条：判官把「有一条不被抛弃的根」写成「每条都不被抛弃」
+/// （`crates/mutations.tsv`「第10h批 Q3 判官把有一条有效根写成每条根都有效」），这一盘面上有被抛弃的 C，判据复算为真，最后一项红。
+#[test]
+fn a_refusal_for_no_valid_root_on_a_crash_image_holding_an_abandoned_root_is_illegal_because_the_newest_root_is_never_abandoned_by_its_own_table(
+) {
+    let image = image_with_an_abandoned_root("no-valid-root-in-the-ring-with-an-abandoned-root");
+    let system_configuration = choose_system_configuration(&image).expect("系统配置");
+    let newest = choose_root(&image, &system_configuration).expect("择得到最新那条根");
+    let table_of_the_newest_root =
+        instance_table_of_root(&image, &newest).expect("最新那条根的实例表读得出");
+    let immutable = &system_configuration.immutable;
+    let roots_in_the_ring = readable_roots(
+        &image,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    );
+    assert!(
+        roots_in_the_ring
+            .iter()
+            .any(|root| root_is_abandoned_by_the_instance_table(root, &table_of_the_newest_root)),
+        "前提：环里有一条按最新那条根的实例表被抛弃的根（C）"
+    );
+    assert!(
+        table_of_the_newest_root
+            .rows
+            .iter()
+            .all(|row| row.instance < newest.instance)
+            && !root_is_abandoned_by_the_instance_table(&newest, &table_of_the_newest_root),
+        "最新那条根（实例 {}）的实例表只有更早的实例的行：{:?}；它按表不被抛弃",
+        newest.instance.0,
+        table_of_the_newest_root.rows
+    );
+    let root_of_an_instance_without_a_row = |root: &singlefs_core::root_record::RootRecord| {
+        !table_of_the_newest_root
+            .rows
+            .iter()
+            .any(|row| row.instance == root.instance)
+    };
+    assert!(
+        roots_in_the_ring.iter().any(root_of_an_instance_without_a_row),
+        "按「表里有该实例的行就算被抛弃」判，也还有一条有效根（最新那条根自己）：两种判法下判据都为假"
+    );
+    let selected_version = recover(&image, JournalPolicy::Consult).effective_root;
+    assert_eq!(
+        judge_writable_mount_refusal_of_a_crash_state(
+            &image,
+            selected_version,
+            &MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
+            DiskAfterTheRefusal::ByteForByteUnchanged,
+            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+            ReadFaultsInTheCrashState::NoneInjected,
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            ],
+        },
+        "环里有被抛弃的 C、也有有效根：判据在镜像上复算为假，拒得不合法"
+    );
+}
diff --git a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
--- a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
+++ b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
@@ -169,6 +169,94 @@
     verdicts
 }
 
+/// 挂载那一刻读坏（撕裂）或读回全零的那个根槽钉在它可能住过的最旧 txg，回收门槛停在那里（用户 2026-10-05 认下的代价：「钉最旧 txg 让 I-3.1
+/// 在合法盘面上红在记账多的一边……用例钉『只许红在记账多的一边』，池级 checker 不放宽」，`records/2026-09-30-里程碑三开工第一轮.md` 第 624 行；
+/// 读回全零照自证不过钉住，同一处第 628 行附近，第 10c 批）：池级 checker 只判 I-3.1 红，记账的已分配比遍历全部有效根多
+/// `expected_bytes_counted_more_than_the_walk`（慢档 4 条调查报告第 3.1 节，判决 `research/prompts/m3-batch10-code-r2-main-verification.md` 第三节最后一行：
+/// 多出来的是盘 0 上释放代在 (钉住的 txg, checker 取的环里最旧的自证根] 里的已释放字节）。
+fn verdicts_red_only_by_the_allocated_statistic_counting_more_than_the_walk(
+    image: &MemoryPool,
+    step: &str,
+    expected_bytes_counted_more_than_the_walk: u64,
+) {
+    let violations: Vec<(&'static str, String)> = check_pool_image(image)
+        .into_iter()
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
+        })
+        .collect();
+    assert_eq!(
+        violations
+            .iter()
+            .map(|(invariant, _)| *invariant)
+            .collect::<Vec<_>>(),
+        vec!["I-3.1"],
+        "{step}：只许 I-3.1 红：{violations:?}"
+    );
+    let (allocated, walked) =
+        singlefs_harness::history::allocated_and_walked_bytes(&violations[0].1)
+            .expect("I-3.1 的说明里带着已分配与遍历两个数");
+    assert!(
+        allocated > walked,
+        "{step}：I-3.1 红在记账多的一边：{}",
+        violations[0].1
+    );
+    assert_eq!(
+        allocated - walked,
+        expected_bytes_counted_more_than_the_walk,
+        "{step}：记账比遍历多的字节数：{}",
+        violations[0].1
+    );
+}
+
+/// 恢复之后重开的那个进程接着覆盖写（实例 3），一直写到 txg `txg_writing_the_pinned_slot`（它的根槽就是挂载时钉住的那一槽，28 + 一圈）：
+/// 写到它前一次为止 I-3.1 仍只红在记账多的一边，写掉那一槽那一次之后池级 checker 一条违例都没有（钉住只到这个进程把那个槽写掉为止，
+/// `allocator.rs` 的 `RootRingOccupant::AllZeroAtMount` / `BadAtMount`）。
+fn overwrite_until_the_pinned_root_slot_is_written_and_the_allocated_statistic_holds(
+    pool: &mut BuiltPool,
+    txg_writing_the_pinned_slot: u64,
+    step: &str,
+) {
+    assert_eq!(
+        root_slot_position_of(txg_writing_the_pinned_slot),
+        root_slot_position_of(28),
+        "txg {txg_writing_the_pinned_slot} 的根槽就是挂载时钉住的那一槽"
+    );
+    let mut overwrite = 0usize;
+    // 迭代上界：从挂载交回的那一版写到钉住那一槽的前一次（不到一圈）；没有提前出口。
+    while pool.output.root.checkpoint_txg.0 + 1 < txg_writing_the_pinned_slot {
+        overwrite_in_process(
+            pool,
+            &content_of(1800 + overwrite, 61 + overwrite),
+            InstanceGeneration(3),
+        );
+        overwrite += 1;
+    }
+    let violated_before_the_slot_is_written: Vec<&str> = check_pool_image(&pool.memory_pool())
+        .into_iter()
+        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
+        .map(|(invariant, _)| invariant)
+        .collect();
+    assert_eq!(
+        violated_before_the_slot_is_written,
+        vec!["I-3.1"],
+        "{step}：写到 txg {} 那一槽还没被写掉，I-3.1 仍红",
+        pool.output.root.checkpoint_txg.0
+    );
+    overwrite_in_process(pool, &content_of(1900, 97), InstanceGeneration(3));
+    assert_eq!(
+        pool.output.root.checkpoint_txg,
+        CheckpointTxg(txg_writing_the_pinned_slot)
+    );
+    verdicts_without_any_violation(
+        &pool.memory_pool(),
+        &format!("{step}：写掉钉住的那一槽（txg {txg_writing_the_pinned_slot}）之后"),
+    );
+}
+
 fn assert_judged_and_holding(
     verdicts: &[(&'static str, InvariantVerdict)],
     invariants: &[&str],
@@ -1528,8 +1616,10 @@
 /// - 进程在重发之前崩了：D 的记录在根槽写之前已持久（D16（发布语义） 已定项 7 的持久顺序），恢复由记录重建 D 的根（已定项 15），
 ///   读回 B 的内容；B 的根槽哪怕被撕坏，回退也已经生效，不用再做一次。
 ///
-/// 两条路上回退都没有丢，C558 说的「退不回去」这两条路上走不到。checker 在重发之后与崩溃之后重开时全绿；崩在重发之前、
-/// B 的根槽被写坏的两格，恢复之前的镜像上 I-3.1（已分配统计对得上） 红，与普通发布同形（见用例里那一段）。
+/// 两条路上回退都没有丢，C558 说的「退不回去」这两条路上走不到。checker 在重发之后全绿；崩在重发之前、
+/// B 的根槽被写坏的两格，恢复之前的镜像上 I-3.1（已分配统计对得上） 红，与普通发布同形（见用例里那一段）；崩溃之后重开时一字节不写那一格全绿，
+/// 撕裂、写成 0 那两格挂载把那一槽钉在它可能住过的最旧 txg（用户 2026-10-05 认下的代价），只许 I-3.1 红在记账多的一边、差 819200，
+/// 这个进程写掉那一槽（txg 52）之后全绿。
 #[test]
 #[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
 fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash(
@@ -1656,7 +1746,31 @@
         "{mode:?}：新实例接在 D 后面"
     );
     crashed.devices = Some(devices);
-    verdicts_without_any_violation(&crashed.memory_pool(), &format!("{mode:?}：崩溃之后重开"));
+    let step = format!("{mode:?}：崩溃之后重开");
+    match mode {
+        RootSlotWriteFailure::RefusedBeforeWriting => {
+            verdicts_without_any_violation(&crashed.memory_pool(), &step);
+        }
+        // B 的根槽（D 那一次写坏的那一槽）挂载时撕裂读坏、或读回全零：钉在它可能住过的最旧 txg 4，回收门槛停在 4，checker 取环里最旧的自证根 8；
+        // 多记账的是盘 0 上释放代 5–8 的已释放字节（慢档调查报告第 3.1 节：Torn、Zeroes 两种都是 819200 = 6406144 − 5586944）。
+        RootSlotWriteFailure::TornAfterTheFirstHalf | RootSlotWriteFailure::ZeroesWritten => {
+            verdicts_red_only_by_the_allocated_statistic_counting_more_than_the_walk(
+                &crashed.memory_pool(),
+                &step,
+                819_200,
+            );
+            crashed.allocator = remounted.allocator;
+            crashed.output = remounted
+                .current
+                .into_file_version()
+                .expect("崩溃之后重开，现行那一版带文件");
+            overwrite_until_the_pinned_root_slot_is_written_and_the_allocated_statistic_holds(
+                &mut crashed,
+                52,
+                &step,
+            );
+        }
+    }
 }
 
 /// C542 剩下的普通可写挂载那一支（`checks-owed.md` C542 行「所选根是最新根时，这次挂载的写行与暖机能不能把环里比它旧的有效根全盖掉」，
@@ -1829,7 +1943,9 @@
 /// 每块盘「记账的已分配 − 遍历全部有效根」= 只被 B 引用的那几个单元的字节数（229 376 = 14 槽）；这几个单元在 C 的账里是已释放、释放代 5
 /// （txg 5 那次换下的），回收要环里最旧的有效根 ≥ 5——B 还在环里时回收不了，C 的账照算它们；B 的根槽被 D 那一次写坏之后遍历里没有 B，
 /// 而 checker 只把「已被某次恢复施加」的记录并进遍历（`walk.rs` 的 `versions_applied_only_by_records`），D 不在其内，用的仍是 C 的账。
-/// D 那一版的账（D 盖掉 B 的槽之后环里最旧的有效根是 5，释放代 5 的已回收）与这个环对得上。恢复（由 D 的记录重建 D）之后重开，全绿。
+/// D 那一版的账（D 盖掉 B 的槽之后环里最旧的有效根是 5，释放代 5 的已回收）与这个环对得上。恢复（由 D 的记录重建 D）之后重开：
+/// 挂载时 B 那一槽读回全零，钉在它可能住过的最旧 txg 4（用户 2026-10-05 认下的代价），只许 I-3.1 红在记账多的一边、差 917504；
+/// 这个进程写掉那一槽（txg 52）之后全绿。
 /// checker 在这一格上该不该判红、要不要把「根槽写坏、记录已持久没施加」那一版并进来，是 checker 判法的事，交主 agent。
 #[test]
 #[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
@@ -1980,7 +2096,23 @@
     );
     drop(cold);
     let mut reopened_devices = pool.reopen_recorded();
-    mount_writable(&parameters(), &mut reopened_devices).expect("恢复之后可写挂载");
+    let remounted = mount_writable(&parameters(), &mut reopened_devices).expect("恢复之后可写挂载");
     pool.devices = Some(reopened_devices);
-    verdicts_without_any_violation(&pool.memory_pool(), "恢复之后重开");
+    // B 的根槽读回全零：钉在它可能住过的最旧 txg 4（用户 2026-10-05 认下的代价，第 10c 批读回全零照自证不过钉住），多记账的是盘 0 上
+    // 释放代 5–8 的已释放字节（慢档调查报告第 3.1 节：917504 = 6455296 − 5537792）；写掉那一槽（txg 52）之后转绿。
+    verdicts_red_only_by_the_allocated_statistic_counting_more_than_the_walk(
+        &pool.memory_pool(),
+        "恢复之后重开",
+        917_504,
+    );
+    pool.allocator = remounted.allocator;
+    pool.output = remounted
+        .current
+        .into_file_version()
+        .expect("恢复之后重开，现行那一版带文件");
+    overwrite_until_the_pinned_root_slot_is_written_and_the_allocated_statistic_holds(
+        &mut pool,
+        52,
+        "恢复之后重开",
+    );
 }
````

### mutations-append.tsv

````
第10h批 rr2 改回按 txg 的落点公式归盘（mkfs 三个区域槽 0 的第 0 代根都归到盘 0，只剩第 0 代根时盘 1 算成一条都没有、整环重读）	crates/singlefs-core/src/mount.rs	                    Some((device_of_the_ring_slot(*ring_slot), *root))	                    Some((device_of_the_ring_slot(target_for_publish(root.checkpoint_txg, slots_per_region)), *root))	-p singlefs-harness --test a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable	the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread
第10h批 R1 改回整份换成重读的读数（第一遍读出、重读报读错的被抛弃根 C 掉出影子账）	crates/singlefs-core/src/mount.rs	                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );	                let readings_of_the_two_passes_merged = readings_of_the_reread;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring
第10h批 R1 改回整份换成重读的读数（重读整环都坏、第一遍读出盘 0 的根：拒成 Q3）	crates/singlefs-core/src/mount.rs	                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );	                let readings_of_the_two_passes_merged = readings_of_the_reread;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_reread_failing_on_the_whole_ring_after_a_first_pass_that_read_device_zero_mounts_on_the_first_pass_roots
第10h批 R1 第一遍自证过压不过重读读坏（重读读回全 0 的被抛弃根 C 掉出影子账）	crates/singlefs-core/src/mount.rs	        (SelfVerified(root_of_the_first_pass), BadAndNotAllZero | AllZeroNeverWritten) => {\n            SelfVerified(root_of_the_first_pass)\n        }	        (SelfVerified(_), BadAndNotAllZero | AllZeroNeverWritten) => reading_of_the_reread,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring
第10h批 R1 第一遍自证过压不过重读读坏（重读读回乱字节的被抛弃根 C 掉出影子账）	crates/singlefs-core/src/mount.rs	        (SelfVerified(root_of_the_first_pass), BadAndNotAllZero | AllZeroNeverWritten) => {\n            SelfVerified(root_of_the_first_pass)\n        }	        (SelfVerified(_), BadAndNotAllZero | AllZeroNeverWritten) => reading_of_the_reread,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring
第10h批 R1 第一遍读坏压过重读自证过的根（第一遍读坏、重读读出的被抛弃根 C 掉出影子账）	crates/singlefs-core/src/mount.rs	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(root_of_the_reread)) => {\n            SelfVerified(root_of_the_reread)\n        }	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(_)) => first_reading,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring
第10h批 R1 两遍都自证过取第一遍那一条	crates/singlefs-core/src/mount.rs	        (SelfVerified(_), SelfVerified(root_of_the_reread)) => SelfVerified(root_of_the_reread),	        (SelfVerified(root_of_the_first_pass), SelfVerified(_)) => SelfVerified(root_of_the_first_pass),	-p singlefs-core --lib	each_pair_of_readings_merges_into_the_self_verified_root_then_the_bad_reading_then_all_zero
第10h批 R1 并时全零压过读坏	crates/singlefs-core/src/mount.rs	        (BadAndNotAllZero, AllZeroNeverWritten) | (AllZeroNeverWritten, BadAndNotAllZero) => {\n            BadAndNotAllZero\n        }	        (BadAndNotAllZero, AllZeroNeverWritten) | (AllZeroNeverWritten, BadAndNotAllZero) => {\n            AllZeroNeverWritten\n        }	-p singlefs-core --lib	each_pair_of_readings_merges_into_the_self_verified_root_then_the_bad_reading_then_all_zero
第10h批 R1 逐槽并时只取第一遍那一份	crates/singlefs-core/src/mount.rs	                    better_reading_of_a_root_ring_slot_over_the_two_passes(\n                        *first_reading,\n                        reading_of_the_reread,\n                    ),	                    *first_reading,	-p singlefs-core --lib	the_two_passes_are_merged_slot_by_slot_in_ring_order
第10h批 rr2 用例丢盘整读：重读之后的读数只剩最新那条根（环里更旧的有效根不进回收门槛与 m，当场放开越过它们，写行与暖机写掉它们引用的单元）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	on_the_264_slot_pool_with_a_live_file_device_zero_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before
第10h批 Q3 判官把「有一条不被抛弃的根」写成「每条根都不被抛弃」（有被抛弃根 C 的盘面上判据复算成真）	crates/singlefs-harness/src/mount_transparency.rs	    .iter()\n    .any(|root| {\n        !table_of_the_newest_root	    .iter()\n    .all(|root| {\n        !table_of_the_newest_root	-p singlefs-harness --test writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring	a_refusal_for_no_valid_root_on_a_crash_image_holding_an_abandoned_root_is_illegal_because_the_newest_root_is_never_abandoned_by_its_own_table
第10h批 慢档 plain：挂载时读回全零的槽不在回收门槛里钉住（钉值撤掉，恢复之后重开 I-3.1 不再红在记账多的一边、差 917504 那一项红）	crates/singlefs-core/src/mount.rs	        .chain(\n            ring_slots_reading_back_all_zero_at_mount\n                .iter()\n                .map(|(_, oldest_txg_it_can_have_held)| *oldest_txg_it_can_have_held),\n        )	        .chain(std::iter::empty::<CheckpointTxg>())	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored plain_overwrite_whose_root_slot_write_zeroes	plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced
第10h批 慢档 zeroed：挂载时读回全零的槽不在回收门槛里钉住（崩溃之后重开差 819200 那一项红）	crates/singlefs-core/src/mount.rs	        .chain(\n            ring_slots_reading_back_all_zero_at_mount\n                .iter()\n                .map(|(_, oldest_txg_it_can_have_held)| *oldest_txg_it_can_have_held),\n        )	        .chain(std::iter::empty::<CheckpointTxg>())	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed	the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash
第10h批 慢档 torn：挂载时读坏的槽不在回收门槛里钉住（崩溃之后重开差 819200 那一项红）	crates/singlefs-core/src/mount.rs	        .chain(\n            ring_slots_bad_at_mount\n                .iter()\n                .map(|(_, txgs_it_can_hold)| txgs_it_can_hold.oldest_txg_it_can_have_held()),\n        )	        .chain(std::iter::empty::<CheckpointTxg>())	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half	the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash
````

### mutations-replacements.tsv

````
删除批第 16 批 GA：每块盘上最新有效根取最小时不滤被抛弃的根（卸载那一格与重建分配器那一格共用的那一处定义）	crates/singlefs-core/src/mount.rs	                .filter(|(device_of_the_root, root)| {\n                    !abandoned_by_table(root, table) && *device_of_the_root == *device\n                })	                .filter(|(device_of_the_root, _)| *device_of_the_root == *device)	-p singlefs-core --lib	an_abandoned_root_newest_on_a_device_does_not_stand_for_that_devices_newest_valid_root
第10e批 重建分配器读根环那一遍一条有效根都没读出时不重读、当场拒（删掉重读那一遍）	crates/singlefs-core/src/mount.rs	            None => {\n                before_the_one_reread.before_rereading();\n                let readings_of_the_reread = read_every_root_ring_slot_once();\n                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );\n                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_two_passes_merged)\n                else {\n                    return Err(\n                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                    );\n                };\n                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )\n            }	            None => {\n                return Err(\n                    MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                )\n            }	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	only_the_allocator_rebuild_pass_failing_rereads_the_ring_and_hands_over_what_a_remount_without_faults_hands_over
第10e批 重读那一遍还读不出有效根时不拒、照旧给空表往下写	crates/singlefs-core/src/mount.rs	                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_two_passes_merged)\n                else {\n                    return Err(\n                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                    );\n                };\n                (\n                    readings_of_the_two_passes_merged,	                let (readings_of_the_two_passes_merged, window_anchor) =\n                    match newest_valid_root_txg_of_the_readings(&readings_of_the_two_passes_merged) {\n                        Some(window_anchor) => (readings_of_the_two_passes_merged, window_anchor),\n                        None => (Vec::new(), CheckpointTxg(0)),\n                    };\n                (\n                    readings_of_the_two_passes_merged,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	the_allocator_rebuild_pass_and_its_reread_both_failing_refuse_the_writable_mount_before_any_write
第10e批 重建分配器读根环那一遍一条有效根都没读出时照第 17 批那样不重读、给空表往下走（攻方 1.3 甲二：暖机写进盘 0 上最新的根还引用的槽）	crates/singlefs-core/src/mount.rs	            None => {\n                before_the_one_reread.before_rereading();\n                let readings_of_the_reread = read_every_root_ring_slot_once();\n                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );\n                let Some(window_anchor) =\n                    newest_valid_root_txg_of_the_readings(&readings_of_the_two_passes_merged)\n                else {\n                    return Err(\n                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,\n                    );\n                };\n                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )\n            }	            None => (\n                Vec::new(),\n                CheckpointTxg(0),\n                RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,\n            ),	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read	only_the_allocator_rebuild_pass_failing_and_a_crash_before_the_warm_up_root_lands_on_device_zero_keep_what_device_zero_reads
````
