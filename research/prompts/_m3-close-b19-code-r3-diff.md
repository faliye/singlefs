# m3-close-b19-code-r3：第二轮判决修补的产品代码改动（2026-10-29）

```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -4620,7 +4620,10 @@
 /// 前面已落盘的那几次在调用方的现行版本里。
 ///
 /// # Panics
-/// 算抬 F 上限交回 `Recovery(NoValidRoot)`：这是本进程准入路径（可写挂载写行之后推、挂着的会话发布被拒时推，分配器都是这个进程挂载建的那一份），
+/// 算抬 F 上限交回 `Recovery(NoValidRoot)`：这是本进程准入路径，调用处三个——可写挂载真做时写行之后与挂载续做里推、
+/// 可写挂载取号之前只进内存那一趟预演把同样两处再走一遍时推（[`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`]，
+/// 分配器是这个进程挂载建的那一份的拷贝）、挂着的会话发布被拒时推；分配器都是这个进程挂载建的那一份或它的拷贝，
+/// 读的都是这个进程写出的现行根（代码轮 `research/prompts/m3-close-b19-code-r2-main-verification.md` 第二节 U3 补列），
 /// 这一错要「这个进程写出的现行根都经记根进了根环表」那条前提破了才走得到（[`rollback_floor_ceiling_of_this_process`] 的两条前提之一，C724
 /// （准入抬 F 上限里同名拒因不在单子上））。公开入口 [`raise_rollback_floor_to_the_admission_ceiling`] 收调用方给的分配器，同一错照原样交回。
 pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
@@ -5633,6 +5636,9 @@
 /// 从头重走（岔路单 `research/prompts/m3-b44-code-r3-forks.md` 岔路二「没走完就重走」那一行），读错的块本来没记下、读坏的块读坏那一刻已从
 /// 共用的那一份里丢掉，重走读到它们穿到盘上；走到 [`CONTINUATION_REHEARSALS_BEFORE_ACQUISITION_AT_MOST`] 趟仍是这样，在取号之前拒，
 /// 交最后那一趟最先的那一次读错，没有读错交最先的那一次读坏（两本账同一判，岔路一「两样都做」那一行）。
+/// 读坏账里读的是补写盘（`publishes_planned.devices_rewritten_by_the_acquisition`）系统配置槽的那几条不算进这一判
+/// （[`read_back_a_system_configuration_slot_of_a_device_rewritten_by_the_acquisition`]；代码轮 `research/prompts/m3-close-b19-code-r2-main-verification.md`
+/// 第二节 U2）：逐盘核已判过它们两槽全废、取号会补写，只进内存那一趟取号读它们必然读坏，算进去会把别处的读坏或别的停法的拒因盖成补写盘的槽。
 /// 两本账都空的照旧留给续做、真做时报它自己的（岔路单那一行只定了 C841，[`rehearsed_continuation_refusal_is_returned_before_acquisition`]）：
 /// 只进内存那一趟与真做读同一份字节、走同一段，它报的错真做走到那一步照样报——当场重读那几读（预演与真做各自再读一次盘）
 /// 与真做取号写之前那几次读、取号写与轮换写算世代号那一读（真做读盘、预演读共用的那一份）除外，它们这一刻读到的可以与预演不同；
@@ -5668,6 +5674,7 @@
     // 迭代上界是 CONTINUATION_REHEARSALS_BEFORE_ACQUISITION_AT_MOST 趟；跨轮带着已走了几趟（两本账每一趟开头清空：只看这一趟记下的，
     // 之前读的——所选那一版的单元清单、写行与暖机那一串的预演、上一趟——有读错读坏也已各自判过）；
     // 出口：走到结局或两本账都空（照旧留给续做）、C841（拒）、到上限仍没走完而账不空（拒）。
+    let slot_spacing_in_bytes = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
     let mut rehearsals_walked: usize = 0;
     let refusal = loop {
         rehearsals_walked += 1;
@@ -5693,10 +5700,19 @@
             | ContinuationRehearsedBeforeAcquisition::StoppedBeforeTheEndOfTheContinuation => {}
         }
         let first_read_failed_on_the_disk = reads_failed_on_the_disk.borrow_mut().drain(..).next();
+        // 补写盘两槽的读坏不拿来判重走与拒（代码轮 `research/prompts/m3-close-b19-code-r2-main-verification.md` 第二节 U2，攻方改法乙）：
+        // 逐盘核已判过这几块盘两槽全废、取号会补写它们，只进内存那一趟取号读它们两槽必然读坏；拿它们判，会让两本账在这几条之外本来是空的那一形
+        // 重走、拒，拒因点名补写盘的系统配置槽，盖住真出故障的那一读。读错账不剔：逐盘核只收两槽都读得出的盘，之后那两槽报读错是新的故障。
         let first_read_failing_self_verification = reads_failing_self_verification
             .borrow_mut()
             .drain(..)
-            .next();
+            .find(|read_failing_self_verification| {
+                !read_back_a_system_configuration_slot_of_a_device_rewritten_by_the_acquisition(
+                    read_failing_self_verification,
+                    &publishes_planned.devices_rewritten_by_the_acquisition,
+                    slot_spacing_in_bytes,
+                )
+            });
         let is_the_last_rehearsal =
             rehearsals_walked >= CONTINUATION_REHEARSALS_BEFORE_ACQUISITION_AT_MOST;
         match (
@@ -5704,7 +5720,7 @@
             first_read_failing_self_verification,
             is_the_last_rehearsal,
         ) {
-            // 这一趟读盘没出过读错、也没读坏：照旧留给续做（岔路单那一行只定了 C841）。
+            // 这一趟读盘没出过读错、补写盘系统配置槽之外也没读坏：照旧留给续做（岔路单那一行只定了 C841）。
             (None, None, _) => return Ok(()),
             (Some(read_failed_on_the_disk), _, true) => {
                 break PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(
@@ -5728,6 +5744,22 @@
     })))
 }
 
+/// 一条读坏是不是读的逐盘核交给取号补写的那几块盘（丙'，`devices_rewritten_by_the_acquisition`）之一的一个系统配置槽：
+/// 盘号在补写盘表里，读的恰是那块盘两槽之一整槽（取号读见证 `recovery::system_configuration_slots_of_one_device_read_once` 那一读的偏移与长度）。
+/// 取号之前预演续做判两本账时剔掉这几条（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）。
+fn read_back_a_system_configuration_slot_of_a_device_rewritten_by_the_acquisition(
+    read_failing_self_verification: &ReadFailingSelfVerification,
+    devices_rewritten_by_the_acquisition: &[DeviceIdentity],
+    slot_spacing_in_bytes: u64,
+) -> bool {
+    devices_rewritten_by_the_acquisition.contains(&read_failing_self_verification.device)
+        && read_failing_self_verification.length_in_bytes
+            == singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES
+        && (0..singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).any(|slot_index| {
+            read_failing_self_verification.offset.0 == slot_index * slot_spacing_in_bytes
+        })
+}
+
 /// 取号之前预演续做走一趟（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）：
 /// 在取号之前那一串预演到最后的那一版与那一份分配器拷贝上，
 /// 照续做取的批大小与容器树容量（分配器上装的删除开关，续做读同一份）把删除队列走到空的每一步都只算不写
```
