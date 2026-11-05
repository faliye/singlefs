# m3-close-b19-code-r2：第一轮判决修补的产品代码改动（2026-10-29）

```diff
diff --git a/crates/singlefs-core/src/instance_table_sweep.rs b/crates/singlefs-core/src/instance_table_sweep.rs
index b1b9494..dd83fb1 100644
--- a/crates/singlefs-core/src/instance_table_sweep.rs
+++ b/crates/singlefs-core/src/instance_table_sweep.rs
@@ -66,8 +66,10 @@ pub enum InstanceTableSweepSwitch {
 pub struct InstanceTableRowCountThreshold(pub u64);
 
 impl InstanceTableRowCountThreshold {
-    /// 产品路径的默认阈值 365 行：用户 2026-10-29 弹窗定（C690（清扫触发阈值的默认值没定））。行数越过它的那一次挂载末尾就扫，
-    /// 链在一片之内（一片装 369 行，370 条记录减去链指针那一条）收回，切换预留不按两片留。
+    /// 产品路径的默认阈值 365 行：用户 2026-10-29 弹窗定（C690（清扫触发阈值的默认值没定））。它是清扫的触发阈值：挂载末尾现行那一版的
+    /// 实例表行数越过它（366 行起）的那一次挂载末尾就扫。它不改每片装几行：每片仍装 369 行（370 条记录减去链指针那一条）。
+    /// 每次挂载写一行、末尾清扫做成时链留在一片之内；清扫这一轮一行不删、一次挂载写几行、或开关交给挂着时的后台清扫时，链照样长出第二片，
+    /// 切换预留照准入式子按片数留（`admission` 里按每片行数取上整的那一项）。
     // 出处：E183 第三段（H5）的读数，θ = 365 不越过一片的上界、θ = 369 越过（`.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md`「H5 的读数」那一条）。
     pub const DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE: Self = Self(365);
     /// 有一行就清扫（用例要在少量行上强制进入清扫那一支，`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index f7b7563..6e21c1f 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -4380,7 +4380,7 @@ pub fn ring_slots_recorded_as_holding_an_abandoned_root_by(
 ///
 /// 今天的 F 取现行那一版的根 `current_root` 带的 F。
 ///
-/// 共享函数里「一条有效根都没有」那一错（`Recovery(NoValidRoot)`）在这里走不到（C724（准入抬 F 上限里同名拒因不在单子上），判决
+/// 共享函数里「一条有效根都没有」那一错（`Recovery(NoValidRoot)`）在本进程准入路径上走不到（C724（准入抬 F 上限里同名拒因不在单子上），判决
 /// `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R2「写明走不到」）：现行那一版的根是这个进程写的，它在有效根里，
 /// 要两条前提，都不是这个函数自己保证的：
 /// 1. 这个进程写出的每一条现行根都经记根（`PoolAllocator::record_root_written_by_this_process`）进了根环表：它的槽是「知道住着根的槽」，
@@ -4388,15 +4388,16 @@ pub fn ring_slots_recorded_as_holding_an_abandoned_root_by(
 ///    不会被判抛弃）；
 /// 2. 根带的 F 不高于它自己的 txg（`valid` 滤的是 txg ≥ F）。
 ///
-/// 第 2 条在这里断言；第 1 条破了时共享函数交回 `NoValidRoot`，这里当场断言失败、不往上交（不在 D23（journal 的角色与格式） 已定项 14 的单子上，
-/// 这一处在写行之后跑，判据在崩溃镜像上复算不了，加进单子是类别错）。共享函数 [`rollback_floor_ceiling`] 留着这个错：
-/// 不带这个进程根环表的调用方（实验装置、用例手拼的分配器）在根环全读坏时合法地走到它。
+/// 第 2 条在这里断言。第 1 条只在分配器是这个进程挂载建的那一份时成立：公开入口（[`raise_rollback_floor_to_the_admission_ceiling`]、
+/// [`raise_rollback_floor`]）收的分配器是调用方给的，不带这个进程根环表的那一种（实验装置、用例手拼的分配器）在根环全读坏时合法地走到
+/// `NoValidRoot`，这里照共享函数原样交回（代码轮 `research/prompts/m3-close-b19-code-r1-main-verification.md` 第二节 T2：共享函数保留这个错，
+/// 公开入口交回它、不 panic）。第 1 条破了时的断言只留在本进程准入路径那一处（[`push_one_floor_raise_within_the_admission_budget`]）。
 ///
 /// # Errors
-/// 同 [`rollback_floor_ceiling`]，除 `Recovery(NoValidRoot)` 之外。
+/// 同 [`rollback_floor_ceiling`]。
 ///
 /// # Panics
-/// `current_root` 带的 F 高于它自己的 txg；共享函数交回 `Recovery(NoValidRoot)`（第 1 条前提破了：有一条现行根没经记根）。
+/// `current_root` 带的 F 高于它自己的 txg。
 #[allow(
     clippy::ptr_arg,
     reason = "rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount 走 &Vec：要一个有大小的读者"
@@ -4415,7 +4416,7 @@ fn rollback_floor_ceiling_of_this_process<Device: BlockDevice>(
         current_root.rollback_floor,
         current_root.checkpoint_txg
     );
-    let ceiling = rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
+    rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
         devices,
         system_configuration,
         current_root.rollback_floor,
@@ -4423,17 +4424,7 @@ fn rollback_floor_ceiling_of_this_process<Device: BlockDevice>(
         &ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(allocator),
         &ring_slots_bad_at_mount_by(allocator),
         &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator),
-    );
-    assert!(
-        !matches!(
-            ceiling,
-            Err(MountError::Recovery(RecoveryFailure::NoValidRoot))
-        ),
-        "这个进程的准入算抬 F 上限读不出一条有效根（现行那一版 txg {:?}）：这个进程写出的现行根没经记根进根环表，它的槽读坏没有重读即拒\
-         （D23（journal 的角色与格式） 已定项 14 射程两条前提之一，C724）",
-        current_root.checkpoint_txg
-    );
-    ceiling
+    )
 }
 
 /// 算抬 F 的上限读根环时，这一遍读坏、重读一次仍坏就拒的槽（[`rollback_floor_ceiling`] 的 `ring_slots_whose_reread_still_bad_refuses_the_ceiling`）：
@@ -4481,7 +4472,8 @@ pub enum RaiseToTheAdmissionCeiling {
 ///
 /// # Errors
 /// 盘表里有身份交了不止一次、调用方的参数或盘表与盘上系统配置不一致（`DeviceIdentitiesHandedInMoreThanOnce`、
-/// `CallerParametersDisagreeWithTheSelectedSystemConfiguration`）；选系统配置、读现行那一版的实例表、算上限的错（都在任何写之前）；
+/// `CallerParametersDisagreeWithTheSelectedSystemConfiguration`）；选系统配置、读现行那一版的实例表、算上限的错（都在任何写之前；
+/// 含一条有效根都没有的 `Recovery(NoValidRoot)`：调用方给的分配器不带这个进程的根环表时，根环全读坏就走到它）；
 /// [`raise_rollback_floor`] 的错原样交回。
 pub fn raise_rollback_floor_to_the_admission_ceiling<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
@@ -4626,6 +4618,11 @@ pub enum FloorRaisePushedWithinTheAdmissionBudget {
 /// 抬 F 自己报的别的错（[`raise_rollback_floor_to_the_admission_ceiling`] 那一半的错，原样；代码审阅第 23 条：块设备错不是空间不够）：
 /// 在任何写之前被拒的，盘上与分配器不动；真发时落盘途中失败的（`MountError::RaiseFloorSequencePublishFailed`），
 /// 前面已落盘的那几次在调用方的现行版本里。
+///
+/// # Panics
+/// 算抬 F 上限交回 `Recovery(NoValidRoot)`：这是本进程准入路径（可写挂载写行之后推、挂着的会话发布被拒时推，分配器都是这个进程挂载建的那一份），
+/// 这一错要「这个进程写出的现行根都经记根进了根环表」那条前提破了才走得到（[`rollback_floor_ceiling_of_this_process`] 的两条前提之一，C724
+/// （准入抬 F 上限里同名拒因不在单子上））。公开入口 [`raise_rollback_floor_to_the_admission_ceiling`] 收调用方给的分配器，同一错照原样交回。
 pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
@@ -4660,13 +4657,25 @@ pub fn push_one_floor_raise_within_the_admission_budget<Device: BlockDevice>(
             },
         ));
     }
-    match raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk(
-        &caller_inputs_agreeing_with_the_disk,
-        devices,
-        allocator,
-        current,
-        shadow_ledger,
-    ) {
+    let raised_to_the_ceiling =
+        raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk(
+            &caller_inputs_agreeing_with_the_disk,
+            devices,
+            allocator,
+            current,
+            shadow_ledger,
+        );
+    // 本进程准入路径：现行根都经记根进了根环表，算上限的那一遍走不到「一条有效根都没有」（文档「# Panics」）。
+    assert!(
+        !matches!(
+            raised_to_the_ceiling,
+            Err(MountError::Recovery(RecoveryFailure::NoValidRoot))
+        ),
+        "这个进程的准入算抬 F 上限读不出一条有效根（现行那一版 txg {:?}）：这个进程写出的现行根没经记根进根环表，它的槽读坏没有重读即拒\
+         （D23（journal 的角色与格式） 已定项 14 射程两条前提之一，C724）",
+        current.root.checkpoint_txg
+    );
+    match raised_to_the_ceiling {
         Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => {
             Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised))
         }
@@ -7097,14 +7106,25 @@ fn establish_instance<Device: BlockDevice>(
         instance_to_acquire,
     )?;
     let placements_planned = Some(placements_taken_by_every_publish);
+    // 丙'：过了补写闸的两槽全废的盘交给取号，取号写两轮把它两槽补写成新号（补写失败照甲' 每块盘回卷一次）。
+    // 逐盘核排在续做预演之前（代码轮 `research/prompts/m3-close-b19-code-r1-main-verification.md` 第二节 T4 改法一）：预演里只进内存那一趟取号
+    // 照真做那一份补写盘表认这几块盘；按空表走的话，那一趟取号读到两槽全废的盘一份自证过的都没有、停在取号，重走到上限在取号之前拒成预演读坏，
+    // 补写闸与它的拒因载荷都走不到（攻方 `research/prompts/m3-close-b19-code-r1-opus-output.md` T4 一节）。逐盘核只读盘、不写，排在前面不改盘上的结局。
+    let devices_rewritten_by_the_acquisition =
+        devices_rewritten_by_the_acquisition_after_checking_each_device(
+            &pool,
+            &start,
+            &units_of_the_selected_version,
+            &read_route,
+        )?;
     // 取号之后那一串（取号、写行、暖机、写行之后推的抬 F）要的、取号之前定下来的那几样：真发与取号之前预演续做读同一份。
-    let mut publishes_planned = PublishesPlannedAfterAcquisition {
+    let publishes_planned = PublishesPlannedAfterAcquisition {
         instance_to_acquire,
         instance_table_rewrite,
         warm_up_publishes_planned,
         placements_planned,
         admission_before_acquisition,
-        devices_rewritten_by_the_acquisition: Vec::new(),
+        devices_rewritten_by_the_acquisition: devices_rewritten_by_the_acquisition.clone(),
     };
     // 挂载续做接着写行与暖机预演到最后的那一版整个预演一遍（续做要推抬 F 时连同取号之后那一串在只进内存的盘上走）：
     // 有一步要开的容器号到顶（C841）就在取号之前拒，盘上逐字节不变；不预演的话这一错在写行那次发布落盘之后才在续做里交回。
@@ -7121,17 +7141,6 @@ fn establish_instance<Device: BlockDevice>(
         &reads_failed_on_the_disk,
         &reads_failing_self_verification,
     )?;
-    // 丙'：过了补写闸的两槽全废的盘交给取号，取号写两轮把它两槽补写成新号（补写失败照甲' 每块盘回卷一次）。
-    publishes_planned.devices_rewritten_by_the_acquisition =
-        devices_rewritten_by_the_acquisition_after_checking_each_device(
-            &pool,
-            &start,
-            &units_of_the_selected_version,
-            &read_route,
-        )?;
-    let devices_rewritten_by_the_acquisition = publishes_planned
-        .devices_rewritten_by_the_acquisition
-        .clone();
     let PublishedBetweenAcquisitionAndTheContinuation {
         instance,
         row_publish,
@@ -7690,7 +7699,7 @@ struct PublishesPlannedAfterAcquisition {
     placements_planned: Option<Vec<PlacementsTakenByOnePublish>>,
     admission_before_acquisition: WritableMountAdmissionBeforeAcquisition,
     /// 取号写两轮顺带补写的盘（丙'）：两槽一份本池自证过的系统配置都没有、过了补写闸的那几块（[`devices_without_the_selected_version`]）。
-    /// 取号之前的逐盘核在预演之后才判，预演那一趟按空表走。
+    /// 取号之前的逐盘核排在续做预演之前判，预演与真做都按它交回的这一份走。
     devices_rewritten_by_the_acquisition: Vec<DeviceIdentity>,
 }
 
```
