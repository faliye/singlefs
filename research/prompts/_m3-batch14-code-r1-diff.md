# 删除批第 14 批合入的代码：代码轮 diff（2026-10-03）

两份补丁原样（实现员 P、Q），加变异表合入前后的差。补丁的底是派发那一刻的主仓 crates/；冻结副本 /tmp/claude-1000/m3-batch14-code-r1/tree/ 是两份都打上之后的样子。

## 1、实现员 P（GA、GC、GB 与文档、Q-F 乙、U8、N2、回扫第 3.4 节落在 transaction.rs、mount.rs、model.rs、mutations.tsv 的条目）

````diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index c7d30cd..f10150c 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -933,7 +933,7 @@ pub struct MountOutput {
 
 /// 挂载续做停在哪儿。成员按调用方要做的决定分：照常写；队列还留着——之后会话里每一次用户改动先走空、走不动报空间不够
 /// （`mounted_session::UserChangeRefused::NoSpaceAfterRaisingTheFloor`），正常卸载照常做成：靠的是卸载那一串的预演算上新 F 回收出来的槽
-/// （[`unmount`]，用户 2026-10-03 定丙 1），之后再挂可写、续做接着走。
+/// 里当场放开的那一截（[`unmount`]，用户 2026-10-03 定甲 GA），之后再挂可写、续做接着走。
 #[derive(Debug)]
 pub enum DeletionQueueContinuationOutcome {
     /// 所选那一版的 `deleted_inodes` 是空的（常态），或续做把它走到了空。
@@ -962,7 +962,7 @@ pub enum MountSpaceAdmission {
     },
     /// 推满仍不够（C565（挂载处推满仍不够怎么收尾没定），实现员提的收尾、交主 agent 定）：挂载照样做成——实例已取、行已写、
     /// 暖机与推的那几串都已落盘；`last_refusal` 是最后一次判的结局，`stop` 是不再推的原因。之后的发布照发布路径的准入判，
-    /// 不够就报空间不够；正常卸载（抬 F 到现行那一版、不判上限）照常做成，它的预演算上新 F 回收出来的槽（[`unmount`]，用户 2026-10-03 定丙 1）。
+    /// 不够就报空间不够；正常卸载（抬 F 到现行那一版、不判上限）照常做成，它的预演算上新 F 回收出来、当场放开的那一截（[`unmount`]，用户 2026-10-03 定甲 GA）。
     StillShortAfterTheFloorRaises {
         refusal_before_acquisition: AdmissionRefusedOnSomeDevices,
         floor_raises: Vec<RaisedFloor>,
@@ -2314,7 +2314,8 @@ pub struct UnmountRaisedTheFloor {
     pub abandoned_roots_unreadable: u64,
 }
 
-/// 抬 F 那一串从哪个入口进来（D16（发布语义） 已定项 1「抬 F 那一串」：准入与卸载共用这一串，两个入口只差这两样）。
+/// 抬 F 那一串从哪个入口进来（D16（发布语义） 已定项 1「抬 F 那一串」：准入与卸载共用这一串，两个入口只差三样：
+/// 判不判上限、根带不带卸载记号、回收出来的槽什么时候能发出去）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum RaiseFloorEntry {
     /// 准入抬 F（`raise_rollback_floor`）：新 F 不高于上限（「抬 F 的上限」那一行的准入式子），这一串的根不带卸载记号。
@@ -2331,19 +2332,65 @@ impl RaiseFloorEntry {
         }
     }
 
-    /// 这一串回收出来的槽什么时候能发出去（用户 2026-10-03 定「丙 1 进下一批」，丙 1 的定义在
-    /// `research/prompts/m3-unmount-after-short-continuation-r1-opus-output.md`「丙1」那一节）：
-    /// 准入那一格扣到生效（每块盘上都有带新 F 的根）；卸载那一格不扣，整串预演与真发都能用——先写系统配置、过屏障那一步在第一条带新 F 的根之前，
-    /// 它之后恢复读得出新 F（D16（发布语义） 已定项 1「生效」那一行把系统配置里读得出的 F 算进 F_生效），回收的槽被这一串的单元写掉时，
-    /// 引用它们的那几条根都已低于 F_生效。挂载续做推满仍放不下之后卸载靠这一样做成（槽被续做吃光，扣着的话这一串在预演里就取不到落点）。
-    const fn reuse_of_the_reclaimed_slots(self) -> ReclaimedReuse {
+    /// 这一串回收出来的槽什么时候能发出去（用户 2026-10-03 定「甲 GA」，`research/prompts/m3-unlink-code-r3-forks.md` 丙1 单盘幸存那一格；
+    /// 判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第三节第 1 条）：准入那一格整截扣到生效（每块盘上都有带新 F 的根）；
+    /// 卸载那一格只把释放代不高于「每块盘上最新有效根的 txg」最小者的那一截当场放开，整串预演与真发都能用，其余照准入那一格扣到生效
+    /// （[`ReuseOfTheSlotsReclaimedByTheSequence`]）。
+    /// 为什么只放那一截：恢复择根不看 F（`recovery::choose_root` 照 `(checkpoint_txg, 实例代号)` 取最大），丢掉一块盘之后择的是剩下那块盘上
+    /// 最新的根，它可以低于系统配置里的新 F（判决第二节 N1 那一行）；一个槽的释放代 g 不高于每块盘上最新的有效根，就没有一块盘上最新的根引用它
+    /// （引用它的根都早于 g），丢哪块盘恢复都择不到引用它的根。挂载续做推满仍放不下之后卸载靠这一截做成（槽被续做吃光，整截扣着的话这一串在预演里
+    /// 取不到落点，`admission_raises_the_floor_before_refusing.rs` 里续做推满之后卸载、再挂续做那一条）。
+    const fn reuse_of_the_reclaimed_slots(self) -> ReuseOfTheSlotsReclaimedByTheSequence {
         match self {
-            RaiseFloorEntry::Admission => ReclaimedReuse::HeldUntilFloorTakesEffect,
-            RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,
+            RaiseFloorEntry::Admission => ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect,
+            RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice,
         }
     }
 }
 
+/// 抬 F 那一串回收出来的槽什么时候能发出去（[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum ReuseOfTheSlotsReclaimedByTheSequence {
+    /// 整截扣到生效（带新 F 的根落满每块盘之后 `PoolAllocator::release_reclaim_holds` 才放开）。
+    HeldUntilFloorTakesEffect,
+    /// 释放代不高于 [`newest_valid_root_txg_on_every_device`] 的那一截当场能发，其余扣到生效。
+    ImmediatelyBelowTheNewestValidRootOnEveryDevice,
+}
+
+/// 每块盘上最新的有效根的 txg 取最小（GA，[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）：`roots` 是抬 F 那一串这一遍从根环读出来的根，
+/// 一条根落在哪块盘由根环落点公式定（`device_of_txg`）。
+/// - 被抛弃的根（按现行那一版的实例表，`abandoned_by_table`）不算：回退之后新实例的 txg 大于环里每一条根（D23（journal 的角色与格式）
+///   已定项 14 第 3 条），被抛弃的根都早于新实例的第一条根；一个槽若在回退之后才释放，释放代就高于每一条被抛弃的根，放不放开只由它那块盘上
+///   有效的根定；若在回退目标那一版之前就释放了，被抛弃的那条时间线接着那一版走，也不引用它。
+/// - 这一遍读不出、自证不过的槽当没有根（与这一遍回收门槛同一个读数）：恢复同样择不到它；它日后又读得出的话那条根只会更新或更旧——更新的引用的已释放槽更少，
+///   更旧的轮不到被择。读不出让最小值只会更小，当场放开的只会更少。
+/// - 有一块盘上一条有效根都没读出来：交回 `None`，这一串回收的槽整截扣到生效（不按 txg 0 去比，说清是哪一格）。
+fn newest_valid_root_txg_on_every_device(
+    roots_read_from_the_ring: &[RootRecord],
+    table: &InstanceTableRecords,
+    all_devices: &[DeviceIdentity],
+    device_of_txg: &dyn Fn(CheckpointTxg) -> DeviceIdentity,
+) -> Option<CheckpointTxg> {
+    let newest_on_each_device: Vec<Option<CheckpointTxg>> = all_devices
+        .iter()
+        .map(|device| {
+            roots_read_from_the_ring
+                .iter()
+                .filter(|root| {
+                    !abandoned_by_table(root, table)
+                        && device_of_txg(root.checkpoint_txg) == *device
+                })
+                .map(|root| root.checkpoint_txg)
+                .max()
+        })
+        .collect();
+    newest_on_each_device
+        .into_iter()
+        .collect::<Option<Vec<CheckpointTxg>>>()?
+        .into_iter()
+        .min()
+}
+
 /// 抬 F 那一串做完之后两个入口共有的东西；准入那一格另带它判过的上限。
 struct FloorRaisedThroughTheSequence {
     ceiling_judged_by_the_admission: Option<CheckpointTxg>,
@@ -2406,8 +2453,9 @@ pub fn raise_rollback_floor<Device: BlockDevice>(
 
 /// 正常卸载（D16（发布语义） 已定项 1「正常卸载」，用户 2026-09-25 定 B1）：推一串空发布把 F 抬到现行那一版的 txg、不另判上限，
 /// 走的是准入抬 F 那一串（影子账按新 F 重算、回收、整串预演、先把新 F 写进每块盘的系统配置过一道屏障、推到每块盘上都有一条带新 F 的根），
-/// 这一串写的根在 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）。与准入那一格差在回收出来的槽不扣到生效、预演与真发都能用
-/// （用户 2026-10-03 定丙 1）：挂载续做推满仍放不下、槽被吃光的盘上卸载照样做成，靠的就是这一串的预演算上新 F 回收出来的槽。第一版几何（R = 3、区域归属 0 / 1 / 0）下推 2 或 3 次：
+/// 这一串写的根在 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）。与准入那一格差在回收出来的槽里释放代不高于每块盘上最新有效根的
+/// 那一截不扣到生效、预演与真发都能用，其余照样扣（用户 2026-10-03 定甲 GA，[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）：
+/// 挂载续做推满仍放不下、槽被吃光的盘上卸载照样做成，靠的就是这一串的预演算上那一截。第一版几何（R = 3、区域归属 0 / 1 / 0）下推 2 或 3 次：
 /// 现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次。不写干净关闭标记，下一次挂载照旧走恢复。做完之后这个写入口不再发布。
 /// 现行那一版树表 0 条时一个字节都不写，卸载照常做成（[`Unmounted::NothingWrittenOnAVersionWithoutFile`]）。
 ///
@@ -2738,8 +2786,8 @@ fn oldest_valid_root_in_the_window_for_raising_the_floor<Reader: PoolReader + ?S
     .0
 }
 
-/// 抬 F 那一串（准入与卸载共用，D16（发布语义） 已定项 1「抬 F 那一串」）。两个入口只差 [`RaiseFloorEntry`] 那两样：
-/// 判不判上限、根带不带卸载记号。
+/// 抬 F 那一串（准入与卸载共用，D16（发布语义） 已定项 1「抬 F 那一串」）。两个入口只差 [`RaiseFloorEntry`] 那三样：
+/// 判不判上限、根带不带卸载记号、回收出来的槽什么时候能发出去（[`RaiseFloorEntry::reuse_of_the_reclaimed_slots`]）。
 fn raise_the_floor_through<Device: BlockDevice>(
     entry: RaiseFloorEntry,
     parameters: &MakeFilesystemParameters,
@@ -2836,7 +2884,8 @@ fn raise_the_floor_through<Device: BlockDevice>(
     // 回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事——checker 的 I-3.1（已分配统计对得上） 按那条根自己的 F
     // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。但回收的槽在生效（每块盘都有带新 F 的根）之前
     // 不许发出去：带新 F 的空发布自己就在分配固定点，开放段满了会开到刚回收空的那一段（alloc-basis 第二轮云端攻方腿打中），所以先扣住、
-    // 落满每块盘之后再放开——这是准入那一格；卸载那一格不扣（用户 2026-10-03 定丙 1，`RaiseFloorEntry::reuse_of_the_reclaimed_slots`）。
+    // 落满每块盘之后再放开——这是准入那一格；卸载那一格只扣释放代高于每块盘上最新有效根的那一截（用户 2026-10-03 定甲 GA，
+    // `RaiseFloorEntry::reuse_of_the_reclaimed_slots`）。
     // 记账按写那条根的那一刻的 F 算还是按它持久之后的 F_生效 算，口径交 alloc-basis 那一轮（预想）。
     // 回收门槛与影子账读的这一遍根环，照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行分两类（与算上限那一遍同一个分法）：
     // 这个进程知道住着根的槽读坏就重读那一槽一次，仍坏就拒这一次抬（C554 乙报告 Q6：按「没有根」往下走，那一槽里的被抛弃根引用的槽
@@ -2893,11 +2942,35 @@ fn raise_the_floor_through<Device: BlockDevice>(
     } else {
         0
     };
-    // 卸载那一格回收的槽不扣（`RaiseFloorEntry::reuse_of_the_reclaimed_slots`，丙 1）：下面的整串预演就在分配器的拷贝上算上它们。
-    let reclaimed = allocator.reclaim_released_up_to(
-        reclaim_floor(new_floor, oldest_valid_root_in_the_window),
-        entry.reuse_of_the_reclaimed_slots(),
-    );
+    // 卸载那一格当场放开的那一截（`RaiseFloorEntry::reuse_of_the_reclaimed_slots`，GA）：下面的整串预演就在分配器的拷贝上算上它们；
+    // 其余那一截与准入那一格一样扣到生效。分配器记着回收过的槽不重复回收，第二次只回收第一次没回收的那几个。
+    let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_window);
+    let reclaimed = match entry.reuse_of_the_reclaimed_slots() {
+        ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => allocator
+            .reclaim_released_up_to(
+                floor_of_the_reclaim,
+                ReclaimedReuse::HeldUntilFloorTakesEffect,
+            ),
+        ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice => {
+            let mut reclaimed = match newest_valid_root_txg_on_every_device(
+                &roots,
+                &table,
+                &all_devices,
+                &device_of_txg,
+            ) {
+                Some(newest_valid_root_on_every_device) => allocator.reclaim_released_up_to(
+                    floor_of_the_reclaim.min(newest_valid_root_on_every_device),
+                    ReclaimedReuse::Immediately,
+                ),
+                None => Vec::new(),
+            };
+            reclaimed.extend(allocator.reclaim_released_up_to(
+                floor_of_the_reclaim,
+                ReclaimedReuse::HeldUntilFloorTakesEffect,
+            ));
+            reclaimed
+        }
+    };
     let publishes_in_the_sequence = planned_txgs.len();
     let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
     // 这一串在任何写之前在分配器的一份拷贝上整串预演（与可写挂载取号之前那一串同一个做法，`dry_run_of_the_publishes_after_acquisition`）：
@@ -4709,7 +4782,7 @@ impl DeletionQueueContinuation {
 /// 这次准入已推的数从 0 数起）。推满仍放不下（一次准入的发布数做满、F 已在上限、抬 F 那一串自己也取不到落点或被空间准入拒）时
 /// 挂载照样做成可写、队列留着（用户 2026-10-03 定「照 D16 做成可写」，D16（发布语义） 已定项 1「准入」那一行「挂载处推满仍不够时挂载照样做成可写」）：
 /// 交回 [`DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises`]，`current` 是最后落盘的那一版；之后会话里每一次用户改动先走空、
-/// 走不动报空间不够；正常卸载照常做成，靠的是卸载那一串的预演算上新 F 回收出来的槽（[`unmount`]，用户 2026-10-03 定丙 1）。
+/// 走不动报空间不够；正常卸载照常做成，靠的是卸载那一串的预演算上新 F 回收出来、当场放开的那一截（[`unmount`]，用户 2026-10-03 定甲 GA）。
 /// 迭代上界：每一轮要么走空交回，要么推满仍放不下交回，要么交回错，要么推成一串（推的数加进这次准入，同一次发布的准入在预算 B − 1 次空发布之内停），
 /// 要么这一趟有一步落了盘（队首那个文件少了单元或删掉了，队列与单元数都有限）。跨轮携带的是走到的那一版、已推的数与已落盘的。
 ///
@@ -4908,7 +4981,7 @@ fn no_demand_on_any_device_of(allocator: &PoolAllocator) -> Vec<DemandOnDevice>
 /// （[`push_one_floor_raise_within_the_admission_budget`]，一次准入最多 [`PUBLISHES_PER_ADMISSION_AT_MOST`] 次发布，写行那次算一次）、
 /// 再判，直到够了或不再推。推满仍不够时怎么收尾条款没定（C565（挂载处推满仍不够怎么收尾没定））：这里取实现员提的那一种——
 /// 挂载照样交回做成（[`MountSpaceAdmission::StillShortAfterTheFloorRaises`]），实例已取、行已写，之后的发布照发布路径的准入判，
-/// 正常卸载照常做成（它抬 F 到现行那一版、不判上限，预演算上新 F 回收出来的槽，[`unmount`]，用户 2026-10-03 定丙 1）；交主 agent 定。
+/// 正常卸载照常做成（它抬 F 到现行那一版、不判上限，预演算上新 F 回收出来、当场放开的那一截，[`unmount`]，用户 2026-10-03 定甲 GA）；交主 agent 定。
 ///
 /// 循环：每一轮先判，够了就交回；不够就推一串，推成了（至少一次空发布、`publishes_pushed` 增加）进下一轮，推满了就交回
 /// （抬 F 那一串自己也取不到落点、被空间准入拒，同样是推满了：[`FloorRaiseStop::FloorRaiseRefusedForSpace`]，实审 A1b Q1），
@@ -5462,7 +5535,7 @@ pub enum RollbackError {
     },
     /// 现行那一版的账（它的根指着的分配记录树）从盘上读，有一个节点读不出或解不开（「cur 那一版的账有一个节点读不出」）。
     CurrentAccountUnreadable { failure: RecoveryFailure },
-    /// 回退目标那一版的八棵树的号与现行那一版的不同：新根要取目标的 inode 树与 extent 树、别的树按现行那一版重算，两边号不同时
+    /// 回退目标那一版的十二棵树的号与现行那一版的不同：新根要取目标的 inode 树与 extent 树、别的树按现行那一版重算，两边号不同时
     /// 新根的树表怎么排条款没写（D23（journal 的角色与格式） 已定项 14 只写了取哪几棵树的根指针），第一版不支持。
     /// 合法历史里走不到：按现行那一版的实例表判仍然有效的根都在同一条时间线上，那条线上的第一个文件版本只发一次号。
     TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 05d14fa..d8b0d60 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -60,7 +60,11 @@ pub enum UserChange<'content> {
     NewInodes { count: u64, write_time_seconds: u64 },
     /// 删一个 inode（D8（核心索引结构） 已定项 16）：一次调用 = 先把现行那一版的 `deleted_inodes` 走空 + 意图那次发布 + 从尾部全部分批
     /// （`transaction::publish_unlink_intent`、`transaction::publish_unlink_steps_until_the_queue_is_empty`）。
-    Unlink { inode: InodeNumber },
+    /// `write_time_seconds` 是意图那次写进这条记录三个时间秒的值，各批照抄它（用户 2026-10-03 定 Q-F 乙）。
+    Unlink {
+        inode: InodeNumber,
+        write_time_seconds: u64,
+    },
 }
 
 /// 一次用户改动发成了：新的一版在会话的 `current` 里。
@@ -242,9 +246,17 @@ fn publish_the_change_once<Device: BlockDevice>(
     let mut writer = PoolWriter::new(parameters, devices);
     writer.set_container_tree_capacities(switches.container_tree_capacities());
     match change {
-        UserChange::Unlink { inode } => {
-            publish_unlink_intent(&mut writer, allocator, previous, inode, instance)
-        }
+        UserChange::Unlink {
+            inode,
+            write_time_seconds,
+        } => publish_unlink_intent(
+            &mut writer,
+            allocator,
+            previous,
+            inode,
+            write_time_seconds,
+            instance,
+        ),
         UserChange::Overwrite(file) => {
             publish_overwrite(&mut writer, allocator, previous, file, instance)
         }
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 3945070..19de0f1 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -2,9 +2,9 @@
 //! （D17（实现分层与第三方管道） 已定项 2；`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。
 //! 三条路径都从同一个枚举走：取号 = 逐盘一次系统配置槽写 + 一道屏障（D23（journal 的角色与格式） 已定项 16，C322（取号那一步的屏障怎么放没有条款） 2026-09-14 定案）；
 //! 暖机（D16（发布语义） 已定项 8）= 屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障；
-//! 新池新建文件（D16（发布语义） 已定项 7）= 单元写 × 8 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障。
-//! 覆盖写（里程碑「覆盖写、释放、回退与复用」步 1 / 步 2）走同一条骨架：新数据单元 COW 到新落点、六个提交内生块与树表 COW 出新版本，
-//! 被换下的八个单元在同一次发布里释放（分配记录改写成已释放 + 释放代，条目不删，D3（空间分配） 已定项 7）。
+//! 新池新建文件（D16（发布语义） 已定项 7）= 单元写 × 12（默认几何）→ 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障。
+//! 覆盖写（里程碑「覆盖写、释放、回退与复用」步 1 / 步 2）走同一条骨架：新数据单元 COW 到新落点、十个提交内生块与树表 COW 出新版本，
+//! 被换下的十二个单元在同一次发布里释放（分配记录改写成已释放 + 释放代，条目不删，D3（空间分配） 已定项 7）。
 //! 每一步都经过块设备接口，录制器挂在那层（D17（实现分层与第三方管道） 已定项 5）。
 
 use std::collections::{BTreeMap, BTreeSet};
@@ -2145,8 +2145,8 @@ impl PoolVersion {
     }
 }
 
-/// 新池新建文件写出的八个单元，声明序 = D3（空间分配） 已定项 10 ⑤ 的 bump 次序（按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末），
-/// 也是出生序号的发号次序（D19（块指针的结构与宽度预算） 已定项 9）与字节表七的 t1..t8。
+/// 新池新建文件写出的十二个单元（八个角色，分配记录树那一个角色按位置寻址是五个节点），声明序 = D3（空间分配） 已定项 10 ⑤ 的 bump 次序（按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末），
+/// 也是出生序号的发号次序（D19（块指针的结构与宽度预算） 已定项 9）与字节表零的 t1..t12。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum TransactionUnit {
     /// 文件的一个数据单元（码 1）。带的是它在文件里的单元序号：一个文件跨多个单元时（里程碑「覆盖写、释放、回退与复用」并行线一）
@@ -2741,7 +2741,7 @@ pub struct TransactionOutput {
     /// 进映射的单元各自的映射 key（每个数据单元、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树；映射树与树表豁免）；
     /// 下一次覆盖写按它经映射取落点释放（D19（块指针的结构与宽度预算） 已定项 5 第 1 条）。
     pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
-    /// 这次发布释放的落点（覆盖写换下的上一版八个单元；新池新建文件为空）：逻辑上释放了的都在，
+    /// 这次发布释放的落点（覆盖写换下的上一版十二个单元；新池新建文件为空）：逻辑上释放了的都在，
     /// 其中有一份核出对不上、那块盘上的记录留在已分配的也在（那几份另列在下一个字段）。
     pub released: Vec<Placement>,
     /// 这次发布释放的落点里，释放之前读盘核校验和核出对不上（读不出也算）、隔离了的那几份（D19（块指针的结构与宽度预算） 已定项 5
@@ -2966,12 +2966,12 @@ impl TransactionOutput {
             .root
     }
 
-    /// 树表条目的诞生 txg（七条同一个数：树建起来那次发布）。
+    /// 树表条目的诞生 txg（十一条同一个数：树建起来那次发布）。
     #[must_use]
     pub fn tree_birth_txg(&self) -> CheckpointTxg {
         self.tree_table_entries
             .first()
-            .expect("树表第 1 版起恒有七条")
+            .expect("树表第 1 版起恒有条目")
             .birth_txg
     }
 
@@ -5553,7 +5553,7 @@ fn inode_record_of_file_version(file: &FileVersionPlan<'_>) -> InodeRecord {
     }
 }
 
-/// 一个池里的第一个文件版本（字节表七 t1..t8 + 六 + 七）：分配八个落点、装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
+/// 一个池里的第一个文件版本（字节表零 t1..t12 + 六 + 七）：分配十二个落点、装十二个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
 /// 建在树表 0 条的那一版上（`version_to_build_on`）：照抄它的实例表指针、换下它指着的那片 mkfs 树表。
 ///
 /// txg 与 jsn 都从它接着算——txg = 那一版的 txg + 1、jsn = 上一条记录的 jsn + 1，**不写死 3**：
@@ -5662,7 +5662,7 @@ pub fn publish_first_file<Device: BlockDevice>(
 }
 
 /// 覆盖写（里程碑「覆盖写、释放、回退与复用」步 1 / 步 2）：同一个实例里接在上一次发布之后再发布一版同一个文件——txg、jsn、事务号各加一，
-/// 对象出生代与容器身份不改，改动计数取这次的 txg；上一版的八个落点经映射释放（进 defer 队列）。
+/// 对象出生代与容器身份不改，改动计数取这次的 txg；上一版的十二个落点经映射释放（进 defer 队列）。
 /// 契约是一个数据单元：多个数据单元的内容走 `publish_sequential_write`。
 ///
 /// # Errors
@@ -5839,7 +5839,9 @@ pub fn publish_new_inodes<Device: BlockDevice>(
 /// 墓碑树开放容器追加一条整对象墓碑（死亡 txg = 这次发布的 checkpoint_txg，D18（块里携带什么信息） 已定项 24 第 2 条）；不立 logged_ops。
 /// 数据单元与 extent 树照抄。一个事务（不写用户数据，不切）。改动计数设成这次发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88
 /// 「最后一次改动所在发布的 checkpoint_txg」：用户 2026-10-03 定 Q-F「算改动，改实现」，意图那次与每一批都算一次改动）；
-/// 写入时间照抄这条记录上一版的值（D8（核心索引结构） 已定项 6 写入时间的取值：unlink 的意图与各批照抄这条记录上一版的写入时间）。
+/// 写入时间取调用方给的 `write_time_seconds`（用户 2026-10-03 定 Q-F「乙 意图带时间、三格同改」：偏移 64 起 atime、mtime、ctime 三个秒字段
+/// 共写这一个值，`records::InodeRecord::to_bytes`），各批照抄意图那次写进记录的值、不另取时间（[`publish_unlink_step`]），
+/// 同一份崩溃镜像各自续做的两份盘因此逐字节相同（D8（核心索引结构） 已定项 16 第 7 条 ①b）。
 ///
 /// # Errors
 /// `inode` 不是树里一条活着的记录（没有、或已经 nlink = 0）⇒ `UnlinkOfAnInodeThatIsNotALiveRecord`；其余同 `publish_version`。
@@ -5852,6 +5854,7 @@ pub fn publish_unlink_intent<Device: BlockDevice>(
     allocator: &mut PoolAllocator,
     previous: &TransactionOutput,
     inode: InodeNumber,
+    write_time_seconds: u64,
     instance: InstanceGeneration,
 ) -> Result<TransactionOutput, PublishError> {
     assert!(
@@ -5876,7 +5879,7 @@ pub fn publish_unlink_intent<Device: BlockDevice>(
                 nlink: NLINK_OF_AN_INODE_BEING_DELETED,
                 size: record.size,
                 change_count: change_count_of_the_intent,
-                write_time_seconds: record.write_time_seconds,
+                write_time_seconds,
             },
             tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),
         },
@@ -5936,7 +5939,8 @@ pub fn publish_unlink_step<Device: BlockDevice>(
                 object_birth: record.object_birth,
                 nlink: record.nlink,
                 size: size_after_the_batch_in_bytes,
-                // 每一批算一次改动（用户 2026-10-03 定 Q-F，同意图那次）；写入时间照旧，理由同 [`publish_unlink_intent`]。
+                // 每一批算一次改动（用户 2026-10-03 定 Q-F，同意图那次）；写入时间照抄上一版这条记录的值——意图那次写进去的那一个
+                // （用户 2026-10-03 定 Q-F 乙「各批照抄」）：批不带时间参数，挂载续做里的几批没有调用方给时间。
                 change_count: change_count_of_the_batch,
                 write_time_seconds: record.write_time_seconds,
             },
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index ac1cbfe..5542ee3 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -3536,9 +3536,10 @@ fn floor_raises_and_versions_in_txg_order<'change>(
 }
 
 /// unlink 第一个文件（`HistoryOperation::Unlink`）：经挂着的会话发（意图那次 + 走空，D8（核心索引结构） 已定项 16）。
+/// 意图那次带这一步的写入时间（与覆盖写、建 inode 同一个取法，用户 2026-10-03 定 Q-F 乙），各批照抄。
 /// 前提：现行那一版带文件、第一个文件那条记录在。删除中的那一版照调：会话先走空、把它走成有树而文件集合为空的一版、再拒成
 /// `UserChangeRefused::NoFileVersionToChange`，走空的那几步逐版交模型比（代码轮 U11）。
-fn apply_unlink(pool: &mut HistoryPool) -> AppliedStep {
+fn apply_unlink(pool: &mut HistoryPool, write_time_seconds: u64) -> AppliedStep {
     let HistoryPool {
         devices,
         session: session_slot,
@@ -3579,6 +3580,7 @@ fn apply_unlink(pool: &mut HistoryPool) -> AppliedStep {
             inode: singlefs_core::address::InodeNumber(
                 singlefs_core::transaction::FIRST_INODE_NUMBER,
             ),
+            write_time_seconds,
         },
     );
     let session_after_the_change = session_slot.insert(WritableSession {
@@ -4681,7 +4683,7 @@ fn apply_operation(
         HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => {
             apply_crash_recovery_abandoning_the_newest_root(pool, &parameters)
         }
-        HistoryOperation::Unlink => apply_unlink(pool),
+        HistoryOperation::Unlink => apply_unlink(pool, write_time_seconds),
         HistoryOperation::CreateInode => apply_create_inode(pool, write_time_seconds),
     }
 }
diff --git a/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs b/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs
new file mode 100644
index 0000000..85ce144
--- /dev/null
+++ b/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs
@@ -0,0 +1,353 @@
+//! 卸载那一串掉电之后只剩一块盘：剩下那块盘只读挂载、整读每个活文件，与卸载之前同一块盘上读出的逐项相同（走到的根 txg 与内容）。
+//! 判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第三节第 2 条（GC）；用户 2026-10-03 定甲 GA
+//! （`mount::RaiseFloorEntry::reuse_of_the_reclaimed_slots`：卸载那一格只把释放代不高于每块盘上最新有效根的那一截当场放开）。
+//! 刀摆在卸载那一串第一次空发布的单元写写完之后（先写系统配置那两个写 + 那几个单元写落下，journal 记录与根都没落），丢掉住着现行那一版的根
+//! 的那块盘。恢复择根不看 F（`recovery::choose_root`），剩下那块盘择到它自己上面最新的根，低于系统配置里的新 F。
+//! 两形照攻方（`research/prompts/m3-unlink-code-r3-opus-output.md` 第一节 1.4）：h2 窄盘续做之后卸载，h4 4 GiB 盘卸载—再挂循环之后第二次卸载。
+//! 池级 checker 在单盘镜像上卸载之前就红六七条（判决第二节 N1「第五节跑前判据的字面」那一行），这里不判 checker，判的是读回。
+
+mod common_admission;
+
+use common_admission::{
+    plain_devices_on,
+    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
+    start_plain, PoolUnderTest, OVERWRITE_BYTES,
+};
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::container_tree::ContainerTreeNodeEntries;
+use singlefs_core::lineage_tree::LineageContainerCapacity;
+use singlefs_core::mount::{
+    mount_writable_with_test_only_switches_and_deletion_switches, unmount, MountError,
+    ShadowLedger, Unmounted,
+};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::mounted_session::MountedSession;
+use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
+use singlefs_core::recovery::{recover, JournalPolicy};
+use singlefs_core::root_ring::target_for_publish;
+use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
+use singlefs_core::transaction::{
+    publish_unlink_intent, publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
+};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_core::unlink::{
+    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
+};
+use singlefs_core::write_accounting::WrittenStructureKind;
+use singlefs_harness::fault_injection::{
+    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
+    FaultSchedule, InjectedFault, SharedFaultPlan,
+};
+use singlefs_harness::history::HistoryDeviceWidth;
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+
+/// 抬 F 那一串第一次发布之前先写系统配置那一步的写：每块盘一次，两块盘。
+const SYSTEM_CONFIGURATION_WRITES_OF_THE_SEQUENCE: u64 = 2;
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
+/// 只读挂载、把挂载态里每个活着的文件从头读到尾：走到的根 txg 与每个活文件（inode 号、全部字节）。删除中的 inode（nlink = 0）读者答「没有文件」，不读。
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
+/// 卸载那一串掉电、丢掉一块盘之后要比的几样。
+struct LoneSurvivorAfterTheFirstUnmountPublish {
+    /// 丢掉的盘：住着卸载之前现行那一版的根。
+    lost_device: DeviceIdentity,
+    current_txg_before_the_unmount: CheckpointTxg,
+    /// 卸载那一串第一次空发布写了几个单元（不注错地在一份拷贝上卸载一次数出来的）。
+    unit_writes_of_the_first_publish: u64,
+    /// 剩下那块盘上恢复择到的根（看 journal）。
+    root_chosen_on_the_survivor: Option<CheckpointTxg>,
+    before_the_unmount: Result<WholeReadOfTheLiveFiles, String>,
+    after_the_crash: Result<WholeReadOfTheLiveFiles, String>,
+}
+
+/// 在 `pool` 的会话上卸载：先在一份拷贝上不注错地卸载一次，数清那一串第一次空发布的单元写；再在另一份上从「先写系统配置那两个写 + 那几个单元写」
+/// 之后的第一个写起每个写都报错（掉电那一形：盘停在那几个单元写之后，journal 记录与根都没落）；丢掉住着现行那一版的根的那块盘，
+/// 卸载之前与掉电之后各在剩下那块盘上整读一遍。
+fn lone_survivor_after_the_first_unmount_publish(
+    pool: &PoolUnderTest<SparseBlockDevice>,
+    device_width: HistoryDeviceWidth,
+) -> LoneSurvivorAfterTheFirstUnmountPublish {
+    let image_before_the_unmount = pool.image();
+    let session = pool.session.as_ref().expect("这一刻有可写会话");
+    let current_txg_before_the_unmount = session.current.root().checkpoint_txg;
+    let unit_writes_of_the_first_publish = {
+        let mut allocator = session.allocator.clone();
+        let mut current = session.current.clone();
+        let mut devices = plain_devices_on(&image_before_the_unmount);
+        let unmounted = unmount(
+            &pool.parameters,
+            &mut devices,
+            &mut allocator,
+            &mut current,
+            session.shadow_ledger,
+        );
+        let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
+            panic!("拷贝上卸载做成：{:?}", unmounted.as_ref().err());
+        };
+        let first = &raised.publishes[0];
+        let structure_writes = |kind: WrittenStructureKind| first.writes.of(kind).write_calls;
+        first.writes.total().write_calls
+            - structure_writes(WrittenStructureKind::JournalRecord)
+            - structure_writes(WrittenStructureKind::RootSlot)
+            - structure_writes(WrittenStructureKind::SystemConfigurationSlot)
+    };
+    let plan = SharedFaultPlan::armed(
+        device_width.fixed_geometry(),
+        FaultSchedule {
+            fault: InjectedFault::WriteFails,
+            device: FaultDeviceSelector::EveryDevice,
+            placement: FaultPlacement::AnyOffset,
+            counting: FaultCounting::AcrossThePool,
+            occurrence: FaultOccurrence::EveryMatchingCallFromTheNthOnward(
+                SYSTEM_CONFIGURATION_WRITES_OF_THE_SEQUENCE + unit_writes_of_the_first_publish + 1,
+            ),
+        },
+    );
+    let mut failing_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
+        plain_devices_on(&image_before_the_unmount)
+            .into_iter()
+            .map(|(identity, device)| {
+                (
+                    identity,
+                    FaultInjectingBlockDevice::new(identity, device, plan.clone()),
+                )
+            })
+            .collect();
+    let mut allocator = session.allocator.clone();
+    let mut current = session.current.clone();
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut failing_devices,
+        &mut allocator,
+        &mut current,
+        session.shadow_ledger,
+    );
+    assert!(
+        matches!(
+            unmounted,
+            Err(MountError::RaiseFloorSequencePublishFailed(_))
+        ),
+        "掉电之后那一串第一次空发布写 journal 记录时报错：{:?}",
+        unmounted.as_ref().err()
+    );
+    let crashed = MemoryPool {
+        devices: failing_devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
+            .collect(),
+        device_size_in_bytes: image_before_the_unmount.device_size_in_bytes,
+    };
+    let ring_slot_of_the_current_root = target_for_publish(
+        current_txg_before_the_unmount,
+        pool.parameters.geometry.root_ring_slots_per_region,
+    );
+    let lost_device = pool.parameters.region_devices
+        [usize::try_from(ring_slot_of_the_current_root.region).expect("区域号")];
+    let survivor_after_the_crash = image_without(&crashed, lost_device);
+    LoneSurvivorAfterTheFirstUnmountPublish {
+        lost_device,
+        current_txg_before_the_unmount,
+        unit_writes_of_the_first_publish,
+        root_chosen_on_the_survivor: recover(&survivor_after_the_crash, JournalPolicy::Consult)
+            .effective_root
+            .map(|(_, checkpoint_txg)| checkpoint_txg),
+        before_the_unmount: whole_read_of_the_live_files(&image_without(
+            &image_before_the_unmount,
+            lost_device,
+        )),
+        after_the_crash: whole_read_of_the_live_files(&survivor_after_the_crash),
+    }
+}
+
+/// 判一份 [`LoneSurvivorAfterTheFirstUnmountPublish`]：第一次空发布真写了单元；剩下那块盘择到的是它自己上面低于现行那一版的根；
+/// 卸载之前那块盘读得出；掉电之后读出来的与卸载之前逐项相同。
+fn assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
+    survivor: &LoneSurvivorAfterTheFirstUnmountPublish,
+) {
+    assert!(
+        survivor.unit_writes_of_the_first_publish > 0,
+        "卸载那一串第一次空发布写了单元（这一刀要判的就是它们先于带新 F 的根写下去）"
+    );
+    assert!(
+        survivor
+            .root_chosen_on_the_survivor
+            .is_some_and(|chosen| chosen < survivor.current_txg_before_the_unmount),
+        "丢掉住着现行那一版（txg {:?}）的根的盘 {:?} 之后，剩下那块盘择到它自己上面更早的根：{:?}",
+        survivor.current_txg_before_the_unmount,
+        survivor.lost_device,
+        survivor.root_chosen_on_the_survivor
+    );
+    assert!(
+        survivor.before_the_unmount.is_ok(),
+        "卸载之前剩下那块盘只读挂载、整读每个活文件做得成：{:?}",
+        survivor.before_the_unmount
+    );
+    assert_eq!(
+        survivor.after_the_crash, survivor.before_the_unmount,
+        "剩下那块盘走到的根 txg 与每个活文件的内容与卸载之前同一块盘上读出的相同：当场放开的槽没有一个被它最新的根引用"
+    );
+}
+
+/// h2（攻方第一节 1.4）：不补环窗口、准入关着造的窄盘（单元区 176 槽），不先覆盖写，第一个文件顺序写长到 4 个单元，
+/// 在这个进程里发 unlink 的意图与 3 批（批大小 1）就崩；重开可写（准入判着、影子账开、批大小 1）续做走完，在这次挂载的会话上卸载。
+/// 判别力：卸载那一格回收的槽整截当场放开（丙1，`ReclaimedReuse::Immediately`）时，第一次空发布把单元写进剩下那块盘上最新的根还引用的槽，
+/// 那块盘只读挂载走树读不出（攻方原型上量过 `MappingStillUnreadable`），最后一项红。
+#[test]
+fn after_the_first_unmount_publish_of_a_narrow_pool_whose_deletion_was_continued_a_lone_surviving_device_reads_back_what_it_read_before(
+) {
+    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
+    let data_units_of_the_file = 4;
+    let batches_published_after_the_intent = 3;
+    let mut pool =
+        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+            device_width,
+            176,
+        );
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
+        let mut current = session
+            .current
+            .version_of_trees()
+            .expect("第一个文件之后有树")
+            .clone();
+        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
+        current = publish_unlink_intent(
+            &mut writer,
+            &mut session.allocator,
+            &current,
+            InodeNumber(FIRST_INODE_NUMBER),
+            *write_time_seconds,
+            session.instance,
+        )
+        .expect("意图那次");
+        for batch_index in 0..batches_published_after_the_intent {
+            current = publish_unlink_step(
+                &mut writer,
+                &mut session.allocator,
+                &current,
+                InodeNumber(FIRST_INODE_NUMBER),
+                deletion_switches_with_batches_of_one().batch_size,
+                session.instance,
+            )
+            .unwrap_or_else(|refusal| panic!("第 {batch_index} 批：{refusal:?}"));
+        }
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
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    pool.session = Some(session);
+    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
+        &lone_survivor_after_the_first_unmount_publish(&pool, device_width),
+    );
+}
+
+/// h4（攻方第一节 1.4）：4 GiB 盘（随机历史默认那一档），第一个文件之后崩了再挂 1 次、正常卸载、再挂、覆盖写 2 次，在这次挂载的会话上第二次卸载。
+/// 判别力：卸载那一格回收的槽整截当场放开（丙1）时，第一次空发布把单元写进剩下那块盘上最新的根还引用的槽（攻方原型上量过：活文件的 extent 叶
+/// 与 inode 叶被写掉、只读挂载之后打开文件失败），最后一项红。
+#[test]
+fn after_the_first_publish_of_a_second_unmount_of_a_four_gibibyte_pool_a_lone_surviving_device_reads_back_what_it_read_before(
+) {
+    let device_width = HistoryDeviceWidth::FourGibibytes;
+    let mut pool = start_plain(device_width);
+    pool.crash_and_mount_writable()
+        .expect("第一个文件之后崩了再挂");
+    {
+        let PoolUnderTest {
+            session,
+            devices,
+            parameters,
+            ..
+        } = &mut pool;
+        let session = session.as_mut().expect("这一刻有可写会话");
+        let unmounted = unmount(
+            parameters,
+            devices,
+            &mut session.allocator,
+            &mut session.current,
+            session.shadow_ledger,
+        );
+        assert!(
+            matches!(unmounted, Ok(Unmounted::FloorRaisedToTheCurrentVersion(_))),
+            "第一次正常卸载做成：{:?}",
+            unmounted.as_ref().err()
+        );
+    }
+    pool.crash_and_mount_writable().expect("卸载之后再挂可写");
+    for overwrite_index in 1..=2 {
+        pool.overwrite(OVERWRITE_BYTES)
+            .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写做成：{refusal:?}"));
+    }
+    assert_the_lone_survivor_reads_back_what_it_read_before_the_unmount(
+        &lone_survivor_after_the_first_unmount_publish(&pool, device_width),
+    );
+}
diff --git a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
index c2af4bc..eb1ea48 100644
--- a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
+++ b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
@@ -106,6 +106,7 @@ fn an_unlink_whose_intent_finds_no_unblocked_slot_is_refused_by_the_landing_judg
         .publish(
             UserChange::Unlink {
                 inode: InodeNumber(FIRST_INODE_NUMBER),
+                write_time_seconds: pool.write_time_seconds,
             },
             None,
         )
@@ -159,6 +160,7 @@ fn an_unlink_whose_last_batch_finds_no_unblocked_slot_is_refused_by_the_landing_
         .publish(
             UserChange::Unlink {
                 inode: InodeNumber(FIRST_INODE_NUMBER),
+                write_time_seconds: pool.write_time_seconds,
             },
             None,
         )
@@ -222,6 +224,7 @@ fn an_unlink_whose_last_batch_and_the_floor_raise_pushed_for_it_both_find_no_unb
         .publish(
             UserChange::Unlink {
                 inode: InodeNumber(FIRST_INODE_NUMBER),
+                write_time_seconds: pool.write_time_seconds,
             },
             None,
         )
@@ -334,6 +337,7 @@ fn pool_crashed_between_deletion_batches(
         session,
         devices,
         parameters,
+        write_time_seconds,
         ..
     } = &mut pool;
     let session = session.as_mut().expect("这一刻有可写会话");
@@ -348,6 +352,7 @@ fn pool_crashed_between_deletion_batches(
         &mut session.allocator,
         &current,
         InodeNumber(FIRST_INODE_NUMBER),
+        *write_time_seconds,
         session.instance,
     )
     .expect("意图那次");
@@ -458,10 +463,11 @@ fn a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_
 /// 续做停在 `StillShortAfterTheFloorRaises`、最后那一次被拒是落得下那一判，现行那一版的队列还留着第一个文件；
 /// 挂着时一次用户改动先走空、走不动报空间不够（一版都没发成、盘不动）。挂载写下的 13 次发布（写行、暖机与续做发成的几步）留在盘上：
 /// 只读挂载走的是挂载最后落盘的那一版，第一个文件仍在删除中（打开它答「没有文件」）。
-/// 正常卸载照常做成（用户 2026-10-03 定丙 1，`m3-unmount-after-short-continuation-r1`）：槽被续做吃光了，卸载那一串的预演算上新 F 回收出来的槽，
+/// 正常卸载照常做成（用户 2026-10-03 定甲 GA，`m3-unlink-code-r3-forks`）：槽被续做吃光了，卸载那一串的预演算上新 F 回收出来、当场放开的那一截
+/// （释放代不高于每块盘上最新有效根的那几个槽），
 /// 抬 F 到现行那一版、两次空发布；之后池级 checker 全绿，再挂可写、续做接着走到空（两步）。
 /// 判别力：推满仍放不下照旧交回 `MountError::Publish`（这一条定案之前的做法）、或结局报成 `WalkedToEmpty`，挂载那一步就红；
-/// 卸载回收的槽照旧扣到生效（`ReclaimedReuse::HeldUntilFloorTakesEffect`）时卸载在预演里取不到落点，卸载那一步红。
+/// 卸载回收的槽照旧整截扣到生效（`ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect`）时卸载在预演里取不到落点，卸载那一步红。
 #[test]
 fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue(
 ) {
@@ -538,7 +544,7 @@ fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_ma
         ),
         "第一个文件仍在删除中：打开它答「没有文件」"
     );
-    // 正常卸载照常做成（D16（发布语义） 已定项 1「正常卸载」，丙 1）：这块窄盘的槽被续做吃光了，卸载那一串回收出来的槽不扣到生效，
+    // 正常卸载照常做成（D16（发布语义） 已定项 1「正常卸载」，甲 GA）：这块窄盘的槽被续做吃光了，卸载那一串回收出来的槽里当场放开的那一截不扣到生效，
     // 预演与真发都落在它们上面。
     let current_txg = session.current.root().checkpoint_txg;
     let unmounted = unmount(
@@ -678,7 +684,7 @@ fn after_a_mount_whose_continuation_stays_short_the_next_mount_is_refused_by_the
 /// 续做走到空、F 一次都没抬、可删的单元都还在 defer 里的盘面上卸载（攻方 `m3-unmount-after-short-continuation-r1` 表 A 那 9 格的形：
 /// 卸载被拒不绑队列）：单元区 256 槽、不先覆盖写、14 个单元，崩在意图与两批之后。重开、准入判着、删除批大小 1：取号之前那一判放行、
 /// 没推抬 F，续做 12 步走到空。挂载之后每块盘只剩 9 个没挡的空槽，卸载那一串三次空发布要 18 个——扣着回收出来的槽就在预演里取不到落点；
-/// 丙 1 之下预演算上新 F 回收出来的槽，卸载做成、F 抬到现行那一版，池级 checker 全绿，再挂可写做成。
+/// 甲 GA 之下预演算上新 F 回收出来、当场放开的那一截，卸载做成、F 抬到现行那一版，池级 checker 全绿，再挂可写做成。
 /// 判别力：卸载回收的槽照旧扣到生效时卸载在预演里被拒，卸载那一步红。
 #[test]
 fn a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims(
@@ -939,6 +945,21 @@ fn mount_failing_on_the_first_write_of_a_continuation_step(
     CheckpointTxg,
     MountError,
     (u64, u64),
+) {
+    mount_failing_on_a_write_of_a_continuation_publish(shape, choose_the_failing_step, |_, _| 1)
+}
+
+/// 同 [`mount_failing_on_the_first_write_of_a_continuation_step`]，注错点摆在 txg 为 `failing_publish` 的那一次发布（推的抬 F 那一串的第一次空发布时，
+/// 从它先写系统配置那一步数起）的第 `ordinal_within_the_publish(输出, failing_publish)` 个写上（从 1 数）。
+fn mount_failing_on_a_write_of_a_continuation_publish(
+    shape: NarrowPoolCrashedBetweenDeletionBatches,
+    choose_the_failing_step: impl Fn(&singlefs_core::mount::MountOutput) -> CheckpointTxg,
+    ordinal_within_the_publish: impl Fn(&singlefs_core::mount::MountOutput, CheckpointTxg) -> u64,
+) -> (
+    singlefs_core::mount::MountOutput,
+    CheckpointTxg,
+    MountError,
+    (u64, u64),
 ) {
     let (pool, _) = pool_crashed_between_deletion_batches(shape);
     let image = pool.image();
@@ -957,7 +978,8 @@ fn mount_failing_on_the_first_write_of_a_continuation_step(
     let (mut failing_devices, stream) = fault_injecting_recorded_devices_on(
         &image,
         Some(write_failing_from(
-            writes_before_the_failing_step + 1,
+            writes_before_the_failing_step
+                + ordinal_within_the_publish(&counted.output, failing_step),
             FailingWrites::OnlyThisWrite,
         )),
     );
@@ -1248,12 +1270,209 @@ fn a_floor_raise_failing_on_a_device_error_in_the_continuation_hands_back_the_wr
     );
 }
 
-/// 卸载那一串（丙 1，回收出来的槽不扣）在先写系统配置之后、第一条带新 F 的根之前掉电（攻方 `m3-unmount-after-short-continuation-r1` 丙 1 那一节
-/// 自列没判的第一件）：续做推满仍放不下的那块窄盘（[`a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue`]
-/// 的造法），先在一份拷贝上不注错地卸载一次，数清那一串第一次空发布写几个单元、其中几个落在这一串自己回收出来的槽上；
-/// 再在另一份上从「先写系统配置那 2 个写 + 第一次空发布的单元写」之后的第一个写起每一个写都报错——单元已写进回收出来的槽，journal 记录与根都没落。
+/// 续做里 txg 为 `step_txg` 的那一步的写，按结构分：单元写、journal 记录写、根槽写（一次发布的写依次是单元写 → 屏障 → 记录 → 屏障 → 根槽 FUA →
+/// 系统配置槽轮换，`transaction` 模块头）。
+fn unit_record_and_root_slot_writes_of_the_continuation_step(
+    output: &singlefs_core::mount::MountOutput,
+    step_txg: CheckpointTxg,
+) -> (u64, u64, u64) {
+    let step = output
+        .deletion_queue_continuation
+        .iter()
+        .find(|step| step.root.checkpoint_txg == step_txg)
+        .expect("注错的那一步是续做里的一步");
+    let structure_writes = |kind: WrittenStructureKind| step.writes.of(kind).write_calls;
+    (
+        step.writes.total().write_calls
+            - structure_writes(WrittenStructureKind::JournalRecord)
+            - structure_writes(WrittenStructureKind::RootSlot)
+            - structure_writes(WrittenStructureKind::SystemConfigurationSlot),
+        structure_writes(WrittenStructureKind::JournalRecord),
+        structure_writes(WrittenStructureKind::RootSlot),
+    )
+}
+
+/// 注错点在续做推的那一串之后的第一步、它第一个写之后的某一个写上（代码轮第三轮判决第三节第 6 条 N2：之前的用例只摆在发布的第一个写上）的那一形：
+/// 起点同 [`a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write`]（窄盘崩在第 0 批之后，续做 14 步、
+/// 第 11 步之后推一串两次空发布）。交回那一步之前落盘的发布数（16 份）、注错那一步写了几个单元、注错的错与真落下的写和根槽写。
+fn mount_failing_after_the_first_write_of_the_step_after_the_continuation_raised_the_floor(
+    ordinal_within_the_step: impl Fn(u64, u64) -> u64,
+) -> (usize, u64, MountError, (u64, u64)) {
+    let (output, failing_step, refusal, landed) =
+        mount_failing_on_a_write_of_a_continuation_publish(
+            NarrowPoolCrashedBetweenDeletionBatches {
+                unit_area_slots: UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION,
+                overwrites_before_the_unlink: OVERWRITES_BEFORE_THE_CONTINUATION,
+                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
+                batches_published_after_the_intent: 0,
+            },
+            first_continuation_step_after_the_continuation_raised_the_floor,
+            |output, step_txg| {
+                let (unit_writes, record_writes, _) =
+                    unit_record_and_root_slot_writes_of_the_continuation_step(output, step_txg);
+                ordinal_within_the_step(unit_writes, record_writes)
+            },
+        );
+    let (_, publishes_before_the_failing_step) =
+        writes_and_publishes_of_the_mount_before(&output, failing_step);
+    let (unit_writes, _, _) =
+        unit_record_and_root_slot_writes_of_the_continuation_step(&output, failing_step);
+    (
+        publishes_before_the_failing_step,
+        unit_writes,
+        refusal,
+        landed,
+    )
+}
+
+/// 续做那一步的第二个单元写报块设备错（N2，注错点在发布第一个写之后）：第一个单元写已落盘、属于失败的那一次发布。
+/// 错是 `Publish`、块设备错；已落盘份数 16 = 落下的根槽写数；写账合计（含失败那一次已记的那 1 个单元写）= 落下的写 − 取号两次。
+/// 判别力：续做那一格交回的失败账照旧是空的（不带失败那一次已落下的写）时合计少 1，那一项红。
+#[test]
+fn a_continuation_step_failing_on_its_second_unit_write_hands_back_every_landed_write() {
+    let (
+        publishes_before_the_failing_step,
+        unit_writes,
+        refusal,
+        (landed_writes, landed_root_slot_writes),
+    ) = mount_failing_after_the_first_write_of_the_step_after_the_continuation_raised_the_floor(
+        |_, _| 2,
+    );
+    assert!(unit_writes >= 2, "那一步至少写两个单元：{unit_writes}");
+    assert_eq!(
+        publishes_before_the_failing_step, 16,
+        "写行、暖机两次、续做 11 步、推的两次"
+    );
+    assert_the_continuation_publish_error_accounts_for_every_landed_write(
+        &refusal,
+        landed_writes,
+        landed_root_slot_writes,
+        publishes_before_the_failing_step,
+    );
+}
+
+/// 续做那一步的 journal 记录写报块设备错（N2）：那一步的单元写全落盘、属于失败的那一次发布，记录与根都没落。
+/// 判法同上一条：份数 16 = 落下的根槽写数，写账合计 = 落下的写 − 取号两次。
+/// 判别力：失败账照旧是空的时合计少那一步的单元写数，那一项红。
+#[test]
+fn a_continuation_step_failing_on_its_journal_record_write_hands_back_every_landed_write() {
+    let (publishes_before_the_failing_step, _, refusal, (landed_writes, landed_root_slot_writes)) =
+        mount_failing_after_the_first_write_of_the_step_after_the_continuation_raised_the_floor(
+            |unit_writes, _| unit_writes + 1,
+        );
+    assert_the_continuation_publish_error_accounts_for_every_landed_write(
+        &refusal,
+        landed_writes,
+        landed_root_slot_writes,
+        publishes_before_the_failing_step,
+    );
+}
+
+/// 续做那一步的根槽写报块设备错（N2）：那一步的单元写与 journal 记录写落盘，根没落（只读恢复会靠那条记录多走一次，见
+/// `unlink_mount_continuation_and_the_emptied_version.rs` 设备错那一条的说明；这里只比根槽写，不比只读挂载走到的版）。
+/// 判法同上：份数 16 = 落下的根槽写数，写账合计 = 落下的写 − 取号两次。
+/// 判别力：失败账照旧是空的时合计少那一步的单元写与记录写，那一项红。
+#[test]
+fn a_continuation_step_failing_on_its_root_slot_write_hands_back_every_landed_write() {
+    let (publishes_before_the_failing_step, _, refusal, (landed_writes, landed_root_slot_writes)) =
+        mount_failing_after_the_first_write_of_the_step_after_the_continuation_raised_the_floor(
+            |unit_writes, record_writes| unit_writes + record_writes + 1,
+        );
+    assert_the_continuation_publish_error_accounts_for_every_landed_write(
+        &refusal,
+        landed_writes,
+        landed_root_slot_writes,
+        publishes_before_the_failing_step,
+    );
+}
+
+/// 续做里为空间推的抬 F 先写系统配置那一步报块设备错（N2，判决第三节第 6 条「续做那一处抬 F 先写系统配置报错」）：192 槽那一种盘
+/// （准入推过一串、续做第 7 步之后又推一串，同 [`a_floor_raise_failing_on_a_device_error_in_the_continuation_hands_back_the_writes_of_the_mounts_publishes`]），
+/// 续做推的那一串先写系统配置的第二个写（盘 1 那一个）报错：第一个已落盘、一条带新 F 的根都没发。挂载交 `FloorRaiseFailedAfterTheMountsPublishes`，
+/// `cause` 是那一串的 `RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`（它的写账是已落下的那 1 个系统配置写）；已落盘份数 11 = 落下的根槽写数；
+/// 写账合计（各份 + 推成的那一串先写系统配置那一步 + `cause` 里那一步的写）= 落下的写 − 取号两次。
+/// 判别力：先写系统配置那一步报错时不带已落下的写（写账交空）时合计少 1，那一项红。
+#[test]
+fn a_floor_raise_in_the_continuation_failing_on_its_system_configuration_write_hands_back_every_landed_write(
+) {
+    let (output, failing_publish, refusal, (landed_writes, landed_root_slot_writes)) =
+        mount_failing_on_a_write_of_a_continuation_publish(
+            NarrowPoolCrashedBetweenDeletionBatches {
+                unit_area_slots: 192,
+                overwrites_before_the_unlink: 0,
+                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
+                batches_published_after_the_intent: 1,
+            },
+            |output| {
+                output.floor_raises_pushed_by_the_continuation[0].publishes[0]
+                    .root
+                    .checkpoint_txg
+            },
+            |_, _| 2,
+        );
+    let MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) = &refusal else {
+        panic!("续做推的抬 F 先写系统配置那一步报块设备错：交 FloorRaiseFailedAfterTheMountsPublishes：{refusal:?}");
+    };
+    let MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(failed_step) =
+        &failed.cause
+    else {
+        panic!(
+            "cause 是续做推的那一串先写系统配置那一步的错原样：{:?}",
+            failed.cause
+        );
+    };
+    let (_, publishes_before_the_failing_raise) =
+        writes_and_publishes_of_the_mount_before(&output, failing_publish);
+    assert_eq!(
+        (
+            failed.writes_of_persisted_publishes.len(),
+            failed
+                .floor_raises
+                .iter()
+                .map(|raised| raised.publishes.len())
+                .collect::<Vec<_>>(),
+            failed_step.writes.total().write_calls,
+            publishes_before_the_failing_raise,
+        ),
+        (11, vec![2], 1, 11),
+        "之前落盘的 11 份（写行、暖机、准入推的两次、续做 7 步）、之前推成的是准入那一串、先写系统配置那一步落了 1 个写"
+    );
+    assert_eq!(
+        u64::try_from(failed.writes_of_persisted_publishes.len()).expect("份数"),
+        landed_root_slot_writes,
+        "已落盘份数等于落下的根槽写数"
+    );
+    assert_eq!(
+        write_calls_of(
+            failed
+                .writes_of_persisted_publishes
+                .iter()
+                .chain(
+                    failed
+                        .floor_raises
+                        .iter()
+                        .map(|raised| &raised.system_configuration_writes_before_the_first_publish)
+                )
+                .chain(std::iter::once(&failed_step.writes))
+        ),
+        landed_writes - ACQUISITION_WRITES,
+        "写账合计等于落下的写减去取号那两次"
+    );
+}
+
+/// 卸载那一串（甲 GA：回收出来的槽里释放代不高于每块盘上最新有效根的那一截当场放开，其余扣到生效）在先写系统配置之后、第一条带新 F 的根之前掉电
+/// （攻方 `m3-unmount-after-short-continuation-r1` 丙 1 那一节自列没判的第一件）：续做推满仍放不下的那块窄盘
+/// （[`a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue`]
+/// 的造法），先在一份拷贝上不注错地卸载一次，数清那一串第一次空发布写几个单元、其中落在这一串回收出来的槽上的那几个各是什么释放代；
+/// 再在另一份上从「先写系统配置那 2 个写 + 第一次空发布的单元写」之后的第一个写起每一个写都报错——单元已写进当场放开的槽，journal 记录与根都没落。
+/// GA 的结局（现行那一版 txg 21，第一版区域归属 0 / 1 / 0：盘 0 上最新的根 21、盘 1 上 19，取小 19）：第一次空发布落在回收槽上的单元，
+/// 那几个槽的释放代都不高于 19；回收出来的槽里释放代 20、21 的那几个（txg 19 那条根还引用着）扣到生效，第一次空发布一个都没落上去。
+/// 改钉这一句的理由：原来钉的「有单元落在回收出来的槽上」是丙 1 的说法（回收的槽整截当场能发）；GA 下当场能发的只有释放代不高于每块盘上最新根的那一截
+/// （判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第三节第 1 条，用户 2026-10-03 定甲 GA），钉的就改成落上去的那几个槽的释放代。
 /// 掉电之后：池级 checker 全绿；只读挂载走的是卸载之前现行那一版（根上带旧 F），而系统配置里已是新 F；再挂可写做成、续做接着走到空，池级 checker 全绿。
-/// 判别力：卸载回收的槽照旧扣到生效时拷贝上那一次卸载就在预演里被拒，第一步红。
+/// 判别力：卸载回收的槽照旧整截扣到生效时拷贝上那一次卸载就在预演里被拒，第一步红。整截当场放开（丙 1）时这一形照样绿：这块盘面上那一串的单元
+/// 本来就没落到释放代 20、21 的槽上（判决第二节 N1「X 第 1404 行那一份盘面的交集」坐实为空；本批证红时量过，`crates/mutations.tsv` 不留这一行），
+/// 释放代那一项只钉 GA 的结局、不分辨丙 1；分辨丙 1 的是 `a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs` 那两形。
 #[test]
 fn an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version(
 ) {
@@ -1278,20 +1497,10 @@ fn an_unmount_losing_power_after_its_system_configuration_write_before_any_root_
             panic!("拷贝上卸载做成：{:?}", unmounted.as_ref().err());
         };
         let first = &raised.publishes[0];
-        let first_publish_slots_reclaimed_by_the_sequence = first
-            .units
-            .iter()
-            .filter(|unit| first.rewritten.contains(&unit.identity))
-            .filter(|unit| {
-                raised
-                    .reclaimed
-                    .iter()
-                    .any(|placement| placement.slot == unit.slot)
-            })
-            .count();
-        assert!(
-            first_publish_slots_reclaimed_by_the_sequence > 0,
-            "第一次空发布有单元落在这一串自己回收出来的槽上（这一刀要打的就是它们先于带新 F 的根写下去）"
+        assert_the_first_unmount_publish_lands_only_on_slots_released_no_later_than_the_newest_root_of_every_device(
+            &session.allocator,
+            &current_root,
+            raised,
         );
         let structure_writes = |kind: WrittenStructureKind| first.writes.of(kind).write_calls;
         (
@@ -1394,12 +1603,79 @@ fn an_unmount_losing_power_after_its_system_configuration_write_before_any_root_
     );
 }
 
+/// 续做推满仍放不下的那块窄盘上，卸载之前现行那一版的 txg，与那时每块盘上最新的根的 txg 取小（第一版区域归属 0 / 1 / 0：txg 21 的根在盘 0，
+/// 盘 1 上最新的是 txg 19；这块盘的根从写行起连号写下来、没有回退，没有被抛弃的根）。
+const CURRENT_TXG_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION: CheckpointTxg = CheckpointTxg(21);
+const NEWEST_ROOT_ON_EVERY_DEVICE_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION: CheckpointTxg =
+    CheckpointTxg(19);
+
+/// 甲 GA 在续做推满仍放不下的那块窄盘上的结局（用户 2026-10-03 定，`mount::RaiseFloorEntry::reuse_of_the_reclaimed_slots`）：卸载那一串
+/// 第一次空发布落在这一串回收出来的槽上的单元至少一个（槽被续做吃光，卸载靠当场放开的那一截做成），那几个槽在卸载之前的分配记录里的释放代都不高于
+/// 每块盘上最新的根的 txg 取小；回收出来的槽里另有释放代高于它的（那几条根还引用着，扣到生效），第一次空发布没落上去。
+fn assert_the_first_unmount_publish_lands_only_on_slots_released_no_later_than_the_newest_root_of_every_device(
+    allocator_before_the_unmount: &singlefs_core::allocator::PoolAllocator,
+    current_root: &RootRecord,
+    raised: &singlefs_core::mount::UnmountRaisedTheFloor,
+) {
+    let release_generation_of = |slot: SlotNumber| {
+        allocator_before_the_unmount
+            .records()
+            .iter()
+            .find(|record| {
+                record.device == DeviceIdentity(0) && record.slot == slot && record.is_released
+            })
+            .map(|record| record.generation)
+            .expect("这一串回收的落点在卸载之前的分配记录里是已释放的")
+    };
+    let first = &raised.publishes[0];
+    let release_generations_under_the_first_publish: Vec<CheckpointTxg> = first
+        .units
+        .iter()
+        .filter(|unit| first.rewritten.contains(&unit.identity))
+        .filter(|unit| {
+            raised
+                .reclaimed
+                .iter()
+                .any(|placement| placement.slot == unit.slot)
+        })
+        .map(|unit| release_generation_of(unit.slot))
+        .collect();
+    let reclaimed_released_after_the_newest_root_of_every_device: Vec<CheckpointTxg> = raised
+        .reclaimed
+        .iter()
+        .map(|placement| release_generation_of(placement.slot))
+        .filter(|generation| {
+            *generation > NEWEST_ROOT_ON_EVERY_DEVICE_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION
+        })
+        .collect();
+    assert_eq!(
+        current_root.checkpoint_txg, CURRENT_TXG_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION,
+        "卸载之前现行那一版 txg 21（盘 0 上），盘 1 上最新的根 txg 19"
+    );
+    assert!(
+        !release_generations_under_the_first_publish.is_empty()
+            && release_generations_under_the_first_publish.iter().all(|generation| {
+                *generation <= NEWEST_ROOT_ON_EVERY_DEVICE_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION
+            }),
+        "第一次空发布落在回收槽上的单元有，那几个槽的释放代都不高于 19：{release_generations_under_the_first_publish:?}"
+    );
+    assert!(
+        !reclaimed_released_after_the_newest_root_of_every_device.is_empty(),
+        "回收出来的槽里有释放代高于 19 的（txg 19 那条根还引用着，GA 下扣到生效）"
+    );
+}
+
 /// 同一刀（卸载那一串先写系统配置之后、第一次空发布的单元写完、journal 记录与根都没落）之后一块盘丢掉、只剩一块（攻方丙 1 那一节自列没判的第二件）：
 /// 丢掉的是住着现行那一版的根的盘 0（现行那一版 txg 21，21 mod 3 = 0，第一版区域归属 0 / 1 / 0），剩下的盘 1 上最新的根是 txg 19，
 /// 早于它系统配置里的新 F（21）。择根与重放：恢复择到盘 1 上最新的那条根、journal 没往后施加，只读挂载走 txg 19 那一版、根上带旧 F；
-/// 可写挂载在读任何盘之前就被 w 的下限拒。池级 checker 在这块剩下的盘上不全绿，与「只写了系统配置、单元一个都没写」那一刀的判定逐条相同：
-/// 第一次空发布写进回收出来的槽的那几个单元，没让 checker 多判出一条（红的那几条不是丙 1 带来的，见报告）。
-/// 判别力：卸载回收的槽照旧扣到生效时拷贝上那一次卸载就在预演里被拒（这一刀摆不出来），第一步红。
+/// 可写挂载在读任何盘之前就被 w 的下限拒。
+/// GA 的结局（用户 2026-10-03 定甲 GA）：第一次空发布写下去、落在回收槽上的那几个单元，槽的释放代都不高于 19——txg 19 那条根不引用它们；
+/// 释放代 20、21 的那几个（txg 19 那条根还引用着）扣到生效、没被写。改钉这一句的理由：原来钉的「写进回收出来的槽的那几个单元没让池级 checker
+/// 多判出一条」分不出 GA 与丙 1——单盘镜像上卸载之前 checker 就红六七条，条名已饱和（判决 `research/prompts/m3-unlink-code-r3-main-verification.md`
+/// 第二节 N1「第五节跑前判据的字面」那一行）；分得出的是被写的槽的释放代，与剩下那块盘的读回（`a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs`）。
+/// checker 那一项照旧判（与「只写了系统配置、单元一个都没写」那一刀逐条相同），只记饱和、不靠它分辨。
+/// 判别力：卸载回收的槽照旧整截扣到生效时拷贝上那一次卸载就在预演里被拒（这一刀摆不出来），第一步红。整截当场放开（丙 1）时这一形照样绿
+/// （同 [`an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version`] 的说明）。
 #[test]
 fn after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor(
 ) {
@@ -1423,6 +1699,11 @@ fn after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_survi
         let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
             panic!("拷贝上卸载做成：{:?}", unmounted.as_ref().err());
         };
+        assert_the_first_unmount_publish_lands_only_on_slots_released_no_later_than_the_newest_root_of_every_device(
+            &session.allocator,
+            &current_root,
+            raised,
+        );
         let first = &raised.publishes[0];
         let structure_writes = |kind: WrittenStructureKind| first.writes.of(kind).write_calls;
         first.writes.total().write_calls
@@ -1511,7 +1792,7 @@ fn after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_survi
     assert_eq!(
         checker_violations_on(&survivor),
         checker_violations_on(&survivor_with_only_the_system_configuration_written),
-        "第一次空发布写进回收出来的槽的那几个单元没让池级 checker 多判出一条"
+        "池级 checker 在单盘镜像上卸载之前就饱和（条名与只写了系统配置那一刀逐条相同），不靠它分辨"
     );
 }
 
diff --git a/crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs b/crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs
index c2448a4..944dcb6 100644
--- a/crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs
+++ b/crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs
@@ -214,11 +214,19 @@ fn an_unlink_intent_whose_commit_generated_units_find_no_unblocked_slot_is_refus
     let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
     pool.crash_and_mount_writable()
         .expect("第一个文件之后崩了再挂");
+    let write_time_seconds_of_the_file = pool.write_time_seconds;
     let intent = |writer: &mut PoolWriter<'_, SparseBlockDevice>,
                   allocator: &mut PoolAllocator,
                   current: &TransactionOutput,
                   instance: InstanceGeneration| {
-        publish_unlink_intent(writer, allocator, current, FIRST_FILE, instance)
+        publish_unlink_intent(
+            writer,
+            allocator,
+            current,
+            FIRST_FILE,
+            write_time_seconds_of_the_file,
+            instance,
+        )
     };
     let ordinary_allocation_slots =
         ordinary_allocation_slots_of_the_same_publish_on_a_copy(&pool, intent);
@@ -270,8 +278,16 @@ fn a_deletion_batch_on_the_narrower_pool_whose_commit_generated_units_find_no_un
         .expect("第一个文件之后崩了再挂");
     pool.sequential_write(3 * data_unit_payload_capacity())
         .expect("顺序写成 3 个数据单元");
+    let write_time_seconds_of_the_file = pool.write_time_seconds;
     publish_directly_on_the_session(&mut pool, |writer, allocator, current, instance| {
-        publish_unlink_intent(writer, allocator, current, FIRST_FILE, instance)
+        publish_unlink_intent(
+            writer,
+            allocator,
+            current,
+            FIRST_FILE,
+            write_time_seconds_of_the_file,
+            instance,
+        )
     })
     .expect("unlink 的意图做成");
     let batch_of_one_data_unit = |writer: &mut PoolWriter<'_, SparseBlockDevice>,
diff --git a/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs b/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs
index 2d057b6..ec6f9db 100644
--- a/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs
+++ b/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs
@@ -136,6 +136,10 @@ fn publish_the_intent_in_process(pool: &mut BuiltPool) -> TransactionOutput {
         &mut pool.allocator,
         &previous,
         FIRST_FILE,
+        previous
+            .inode_record
+            .expect("意图之前第一个文件那条记录在")
+            .write_time_seconds,
         INSTANCE_OF_THE_PROCESS,
     )
     .expect("意图");
diff --git a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
index d153fce..37e635f 100644
--- a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
+++ b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
@@ -59,6 +59,9 @@ fn position(level: u8, index_in_level: u32) -> ContainerTreeNodePosition {
     }
 }
 
+/// 建空 inode 那次的写入时间；unlink 它们的意图照传这一个（记录现有的时间，盘上字节与改之前相同）。
+const WRITE_TIME_SECONDS_OF_THE_NEW_INODES: u64 = 1_788_000_100;
+
 /// 新池新建文件之后建 `count` 个空 inode（号从 2 起）。
 fn pool_with_new_inodes(tag: &str, count: u64) -> BuiltPool {
     let mut pool = build_pool(tag);
@@ -71,7 +74,7 @@ fn pool_with_new_inodes(tag: &str, count: u64) -> BuiltPool {
         &mut pool.allocator,
         &previous,
         count,
-        1_788_000_100,
+        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
         INSTANCE_OF_THE_PUBLISHES,
     )
     .expect("建空 inode");
@@ -94,6 +97,7 @@ fn unlink(
         &mut pool.allocator,
         &previous,
         InodeNumber(inode),
+        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
         INSTANCE_OF_THE_PUBLISHES,
     )
     .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
diff --git a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
index 5e2e8e4..6e52a01 100644
--- a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
+++ b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
@@ -1083,6 +1083,7 @@ fn pool_crashed_after_the_unlink_intent_of_fourteen_data_units_on_narrow_devices
         session,
         devices,
         parameters,
+        write_time_seconds,
         ..
     } = &mut pool;
     let session = session.as_mut().expect("这一刻有可写会话");
@@ -1097,6 +1098,7 @@ fn pool_crashed_after_the_unlink_intent_of_fourteen_data_units_on_narrow_devices
         &mut session.allocator,
         &current,
         InodeNumber(FIRST_INODE_NUMBER),
+        *write_time_seconds,
         session.instance,
     )
     .expect("意图那次");
diff --git a/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs
index eec389a..5874aab 100644
--- a/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs
@@ -81,6 +81,10 @@ fn publish_the_intent_in_process(pool: &mut BuiltPool) -> TransactionOutput {
         &mut pool.allocator,
         &previous,
         FIRST_FILE,
+        previous
+            .inode_record
+            .expect("意图之前第一个文件那条记录在")
+            .write_time_seconds,
         INSTANCE_OF_THE_PROCESS,
     )
     .expect("意图");
diff --git a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
index 604838f..076f3f2 100644
--- a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
@@ -12,7 +12,7 @@ use common::{
 };
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::{InodeNumber, InstanceGeneration};
+use singlefs_core::address::{DeviceOffsetInBytes, InodeNumber, InstanceGeneration};
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::container_tree::ContainerTreeNodeEntries;
 use singlefs_core::lineage_tree::LineageContainerCapacity;
@@ -35,7 +35,7 @@ use singlefs_core::unit::{build_index_node, data_unit_payload_capacity, parse_in
 use singlefs_core::unlink::{
     DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
 };
-use singlefs_format::SLOT_BYTES;
+use singlefs_format::{INODE_RECORD_BYTES, PACKED_UNIT_RECORDS_OFFSET, SLOT_BYTES};
 use singlefs_harness::memory_pool::MemoryPool;
 use singlefs_harness::SharedStream;
 
@@ -72,7 +72,25 @@ fn pool_with_a_three_unit_file(tag: &str) -> BuiltPool {
     pool
 }
 
+/// 第一个文件那条记录现有的写入时间（池的当前版是进程内最后发的那一版）：unlink 的意图照传它，盘上字节与 Q-F 乙之前相同。
+fn write_time_seconds_of_the_first_file(pool: &BuiltPool) -> u64 {
+    pool.output
+        .inode_record
+        .expect("这一刻第一个文件那条记录在")
+        .write_time_seconds
+}
+
+/// 在这个进程里发 unlink 的意图，带第一个文件那条记录现有的写入时间。
 fn publish_the_intent_in_process(pool: &mut BuiltPool) -> TransactionOutput {
+    let write_time_seconds = write_time_seconds_of_the_first_file(pool);
+    publish_the_intent_in_process_at(pool, write_time_seconds)
+}
+
+/// 在这个进程里发 unlink 的意图，带调用方给的写入时间（用户 2026-10-03 定 Q-F 乙：意图带时间、三格同改）。
+fn publish_the_intent_in_process_at(
+    pool: &mut BuiltPool,
+    write_time_seconds: u64,
+) -> TransactionOutput {
     let publish_parameters = parameters();
     let previous = pool.output.clone();
     let devices = pool.devices.as_mut().expect("镜像还开着");
@@ -82,6 +100,7 @@ fn publish_the_intent_in_process(pool: &mut BuiltPool) -> TransactionOutput {
         &mut pool.allocator,
         &previous,
         FIRST_FILE,
+        write_time_seconds,
         INSTANCE_OF_THE_PROCESS,
     )
     .expect("意图");
@@ -198,7 +217,13 @@ fn a_session_mounted_with_batches_of_one_unlinks_a_three_unit_file_in_an_intent_
     let (_, mut session) = MountedSession::of_the_mount(mounted);
     let txg_before = session.current.root().checkpoint_txg.0;
     session
-        .publish_user_change(&mut devices, UserChange::Unlink { inode: FIRST_FILE })
+        .publish_user_change(
+            &mut devices,
+            UserChange::Unlink {
+                inode: FIRST_FILE,
+                write_time_seconds: write_time_seconds_of_the_first_file(&pool),
+            },
+        )
         .expect("会话 unlink");
     assert_eq!(
         (
@@ -220,7 +245,13 @@ fn the_session_hands_in_the_intent_and_every_step_of_an_unlink_in_publish_order(
     let (_, mut session) = MountedSession::of_the_mount(mounted);
     let txg_before = session.current.root().checkpoint_txg.0;
     let published = session
-        .publish_user_change(&mut devices, UserChange::Unlink { inode: FIRST_FILE })
+        .publish_user_change(
+            &mut devices,
+            UserChange::Unlink {
+                inode: FIRST_FILE,
+                write_time_seconds: write_time_seconds_of_the_first_file(&pool),
+            },
+        )
         .expect("会话 unlink");
     assert_eq!(
         published
@@ -293,7 +324,13 @@ fn after_the_unlink_the_session_holds_the_emptied_version_and_refuses_an_overwri
     let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
     let (_, mut session) = MountedSession::of_the_mount(mounted);
     session
-        .publish_user_change(&mut devices, UserChange::Unlink { inode: FIRST_FILE })
+        .publish_user_change(
+            &mut devices,
+            UserChange::Unlink {
+                inode: FIRST_FILE,
+                write_time_seconds: write_time_seconds_of_the_first_file(&pool),
+            },
+        )
         .expect("会话 unlink");
     assert_eq!(
         member_name(&session.current),
@@ -337,7 +374,13 @@ fn new_inodes_on_the_emptied_version_are_published_and_the_version_holds_files_a
     let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
     let (_, mut session) = MountedSession::of_the_mount(mounted);
     session
-        .publish_user_change(&mut devices, UserChange::Unlink { inode: FIRST_FILE })
+        .publish_user_change(
+            &mut devices,
+            UserChange::Unlink {
+                inode: FIRST_FILE,
+                write_time_seconds: write_time_seconds_of_the_first_file(&pool),
+            },
+        )
         .expect("会话 unlink");
     session
         .publish_user_change(
@@ -420,7 +463,9 @@ fn record_of_the_first_file(version: &TransactionOutput) -> InodeRecord {
 
 /// 意图那次与中间一批都算一次改动（用户 2026-10-03 定 Q-F「算改动，改实现」；D8（核心索引结构） 已定项 6 偏移 88 改动计数 =
 /// 最后一次改动所在发布的 checkpoint_txg）：三个单元的文件发意图、再发一批（留 2），两次之后第一个文件那条记录的改动计数各等于那一次发布的 txg；
-/// 写入时间不动（记录里没有 ctime，删除步没有确定的来源）。两次之后池级 checker 全绿（I-9.26（inode 改动计数落在出生代与容器诞生代号之间））。
+/// 写入时间另由 [`the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it`] 判（偏移 64 起 atime、mtime、ctime
+/// 三个秒字段共写一个 `write_time_seconds`，`records::InodeRecord::to_bytes`；D8（核心索引结构） 已定项 6 字段表）。
+/// 两次之后池级 checker 全绿（I-9.26（inode 改动计数落在出生代与容器诞生代号之间））。
 /// 判别力：意图那次或那一批照旧带上一版的改动计数时那一项红。
 #[test]
 fn the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish() {
@@ -447,17 +492,6 @@ fn the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_o
         ),
         "意图那次与中间那一批各把改动计数设成自己那次发布的 txg"
     );
-    assert_eq!(
-        (
-            after_the_intent.write_time_seconds,
-            after_the_middle_batch.write_time_seconds,
-        ),
-        (
-            before_the_unlink.write_time_seconds,
-            before_the_unlink.write_time_seconds,
-        ),
-        "写入时间不动"
-    );
     assert_eq!(
         violations(&pool.memory_pool()),
         Vec::<String>::new(),
@@ -465,6 +499,78 @@ fn the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_o
     );
 }
 
+/// 第一个文件那条记录在 `version` 这一版里、两块盘上各读出来的三个时间秒（偏移 64 起 atime、mtime、ctime 各 8 字节，D8（核心索引结构）
+/// 已定项 6 字段表）：按盘读这一版每片 inode 叶容器的单元字节，在码 3 记录区（`PACKED_UNIT_RECORDS_OFFSET` 起、每条 `INODE_RECORD_BYTES`）里
+/// 找 inode 号是第一个文件的那一条。读的是盘，不是内存里那份记录。
+fn time_seconds_of_the_first_file_on_each_device(
+    image: &MemoryPool,
+    version: &TransactionOutput,
+) -> Vec<[u64; 3]> {
+    let record_bytes = usize::try_from(INODE_RECORD_BYTES).expect("140");
+    let records_offset = usize::try_from(PACKED_UNIT_RECORDS_OFFSET).expect("136");
+    let field_at = |record: &[u8], offset: usize| {
+        u64::from_le_bytes(
+            record[offset..offset + 8]
+                .try_into()
+                .expect("切出来的正好 8 字节"),
+        )
+    };
+    image
+        .devices
+        .values()
+        .map(|device| {
+            version
+                .units
+                .iter()
+                .filter(|unit| matches!(unit.identity, TransactionUnit::InodeLeafContainer(_)))
+                .find_map(|unit| {
+                    let unit_bytes = device.read(
+                        DeviceOffsetInBytes(unit.slot.0 * SLOT_BYTES),
+                        usize::try_from(unit.identity.span_slots() * SLOT_BYTES)
+                            .expect("单元字节数"),
+                    );
+                    unit_bytes[records_offset..]
+                        .chunks_exact(record_bytes)
+                        .find(|record| field_at(record, 0) == FIRST_FILE.0)
+                        .map(|record| {
+                            [
+                                field_at(record, 64),
+                                field_at(record, 72),
+                                field_at(record, 80),
+                            ]
+                        })
+                })
+                .expect("这一版的 inode 叶容器里有第一个文件那条记录（意图与中间各批之后 nlink = 0、还在树里）")
+        })
+        .collect()
+}
+
+/// 意图那次带调用方给的写入时间（用户 2026-10-03 定 Q-F「乙 意图带时间、三格同改」）：三个单元的文件发意图，给一个与这条记录上一版不同的时间，
+/// 再发一批（留 2）。意图那一版两块盘上第一个文件那条记录的三个时间秒都等于给的时间；中间那一批照抄意图那次写进去的值、不另取。
+/// 判别力：意图照旧带上一版的时间时第一项红；那一批不照抄上一版这条记录的时间（另取一个）时第二项红。
+#[test]
+fn the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it() {
+    let mut pool = pool_with_a_three_unit_file("write-time-of-the-deletion-steps");
+    let time_before_the_unlink = write_time_seconds_of_the_first_file(&pool);
+    let time_of_the_intent = time_before_the_unlink + 600;
+    let intent = publish_the_intent_in_process_at(&mut pool, time_of_the_intent);
+    let on_disk_after_the_intent =
+        time_seconds_of_the_first_file_on_each_device(&pool.memory_pool(), &intent);
+    let middle_batch = publish_the_next_step_in_process(&mut pool);
+    let on_disk_after_the_middle_batch =
+        time_seconds_of_the_first_file_on_each_device(&pool.memory_pool(), &middle_batch);
+    assert_eq!(
+        on_disk_after_the_intent,
+        vec![[time_of_the_intent; 3]; 2],
+        "意图那一版两块盘上那条记录的 atime、mtime、ctime 秒都是给的时间（上一版是 {time_before_the_unlink}）"
+    );
+    assert_eq!(
+        on_disk_after_the_middle_batch,
+        vec![[time_of_the_intent; 3]; 2],
+        "中间那一批照抄意图那次写进去的时间"
+    );
+}
+
 /// 两块盘的池：标签 6 是全部副本之和，每份一个副本。
 const REPLICAS_OF_THE_TWO_DEVICE_POOL: u64 = 2;
 
@@ -497,6 +603,7 @@ fn the_tombstone_prepay_of_an_overwrite_reads_the_tombstone_tree_height_from_the
             &mut pool.allocator,
             &with_a_second_inode,
             second_inode,
+            FIXED_WRITE_TIME_SECONDS,
             INSTANCE_OF_THE_PROCESS,
         )
         .expect("unlink 那个空 inode 的意图");
@@ -835,7 +942,13 @@ fn an_overwrite_after_rolling_back_to_the_intent_walks_the_queue_and_its_refusal
     let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
     let (_, mut session) = MountedSession::of_the_mount(mounted);
     let unlinked = session
-        .publish_user_change(&mut devices, UserChange::Unlink { inode: FIRST_FILE })
+        .publish_user_change(
+            &mut devices,
+            UserChange::Unlink {
+                inode: FIRST_FILE,
+                write_time_seconds: write_time_seconds_of_the_first_file(&pool),
+            },
+        )
         .expect("会话 unlink");
     let intent_root = unlinked.versions_published[0].root;
     let mut current = session
diff --git a/crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs b/crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs
index e12e7d9..3649e9b 100644
--- a/crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs
+++ b/crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs
@@ -102,6 +102,10 @@ fn a_file_over_two_extent_lower_leaves_unlinked_in_product_batches_rewrites_the_
             &mut pool.allocator,
             &previous,
             FIRST_FILE,
+            previous
+                .inode_record
+                .expect("意图之前第一个文件那条记录在")
+                .write_time_seconds,
             INSTANCE_OF_THE_PROCESS,
         )
         .expect("意图");
diff --git a/crates/singlefs-harness/tests/unlink_single_file_publishes.rs b/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
index 3b190c5..1979905 100644
--- a/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
+++ b/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
@@ -63,11 +63,16 @@ fn publish_the_intent(pool: &mut BuiltPool) -> Result<TransactionOutput, Publish
     let previous = pool.output.clone();
     let devices = pool.devices.as_mut().expect("镜像还开着");
     let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    // 第一个文件那条记录现有的时间（已经删完的那一格没有记录，意图在任何写之前被拒，给的时间不落盘）。
+    let write_time_seconds = previous
+        .inode_record
+        .map_or(FIXED_WRITE_TIME_SECONDS, |record| record.write_time_seconds);
     let output = publish_unlink_intent(
         &mut writer,
         &mut pool.allocator,
         &previous,
         FIRST_FILE,
+        write_time_seconds,
         INSTANCE_OF_THE_PUBLISHES,
     )?;
     pool.output = output.clone();
````

## 2、实现员 Q（回扫第 3.4 节其余文件：注释、消息、常量说明）

````diff
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -1353,7 +1353,7 @@
         allocator
     }
 
-    /// mkfs 写在单元区里的两个单元：每盘一条分配代 0 的分配记录（字节表五：20 条里 m1 / m2 各两条，分配代 0）。
+    /// mkfs 写在单元区里的两个单元：每盘一条分配代 0 的分配记录（字节表五：28 条里 m1 / m2 各两条，分配代 0）。
     /// 单元区之外的固定结构（系统配置、根环、journal 环）不写分配记录、分配器不下探（D3（空间分配） 已定项 10 ④）。
     pub fn mark_format_time_units(&mut self, instance_table: Placement, tree_table: Placement) {
         self.record(instance_table, CheckpointTxg(0));
@@ -2589,7 +2589,7 @@
 
     /// 增补 2 第 20c 行（代码三方第一轮打中）：回落把开放段置空之后，那一段仍是聚簇段、仍不给用户数据
     /// （D3（空间分配） 已定项 8 第 2 条「聚簇段只给提交内生块」；已定项 10 ② 的量词是「任何开放的聚簇段」）。
-    /// 形态：开段 [50240, 50304) 发新池新建文件那七个提交内生块 ⇒ 游标停在 50249；段里游标前方的槽占满、单元区里一个全空段都不剩
+    /// 形态：开段 [50240, 50304) 发七个提交内生块（新池新建文件的前七个，字节表零 t2..t8）⇒ 游标停在 50249；段里游标前方的槽占满、单元区里一个全空段都不剩
     /// ⇒ 下一个提交内生块回落、`open_segment` 置空；再把段里那个两槽容器释放、回收（游标不回头，bump 够不着它）——
     /// 段里于是有一对空的偶数槽 50242–50243，而且它是全池最低的一对，段里 50244–50248 还躺着活着的内生块。
     /// 这时要一个用户数据落点：它必须落在段外面。
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -4218,7 +4218,7 @@
 }
 
 /// 分配记录「每个落点每盘各一条」（两盘同槽、同一批字段）：同一块盘上一个槽只许一条记录，每块盘各自的（槽, 跨度, 代, 已释放）集合相同，
-/// 每盘不少于 10 个落点（mkfs 2 + 新池新建文件 8）。同盘同槽两条记录（代不同）在集合里是两个元素、两盘对称就过——第二轮攻方腿打中，
+/// 每盘不少于 `NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE` 个落点（下限怎么来的见那个常量）。同盘同槽两条记录（代不同）在集合里是两个元素、两盘对称就过——第二轮攻方腿打中，
 /// 走读自己不判 key 严格递增，这里逐盘核槽号不重复。
 #[must_use]
 pub fn allocation_records_are_one_per_device(
@@ -4259,7 +4259,10 @@
         && placement_sets.all(|placements| placements == first_device_placements)
 }
 
-/// mkfs 写在单元区里的 2 个落点加新池新建文件的 8 个落点（字节表五：20 条记录，每盘 10 条），之后每次发布只多不少。
+/// 每盘落点数的下限：mkfs 写在单元区里的 2 个落点，加新池新建文件只写 8 个单元时的 8 个落点。
+/// 新池新建文件在默认几何下写 12 个落点（字节表五：28 条记录，每盘 14 条；提交内生块个数随几何变），之后每次发布只多不少，
+/// 所以这条下限比盘上实有的松。
+/// TODO：下限没随分配记录树按位置寻址（D8（核心索引结构） 已定项 14）抬上去：内生块个数随几何变，取各几何的最小值还是按几何现算，没有条款。
 const NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE: usize = 10;
 
 /// 冷走读里的中央映射树（自举豁免，只按父指针里的位置条目读）：到第一次有树根的提示读不出、要经映射回退时，
@@ -5194,7 +5197,7 @@
         }
     }
 
-    /// 新池新建文件的形状：每盘 10 个落点、同一批槽号；已释放的记录照样算一个落点。
+    /// 按下限造的形状：每盘恰好 `NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE` 个落点、同一批槽号；已释放的记录照样算一个落点。
     fn new_pool_file_creation_shape() -> Vec<AllocationRecord> {
         let mut records = Vec::new();
         for placement_index in 0..NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE {
--- a/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
+++ b/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
@@ -508,7 +508,7 @@
     );
 }
 
-/// 释放判定路径（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：释放一律经映射，不经提示）：A 的八个落点经 A 的映射取出来与 A 自己记的槽号逐个相同；
+/// 释放判定路径（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：释放一律经映射，不经提示）：A 的十二个落点经 A 的映射取出来与 A 自己记的槽号逐个相同；
 /// B 之后 A 的数据单元那把 key 在 B 的映射里查不到（「不在映射」）；把 A 的映射节点重装成少了码 1 那一条的形态再覆盖写 ⇒ 报「不在映射」、
 /// 一个落点都不释放、分配器一条记录都没改写——按提示释放的写法会把它当成一次正常的覆盖写。
 #[test]
--- a/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
+++ b/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
@@ -77,8 +77,8 @@
 use singlefs_core::unit::seal_header_checksum;
 use singlefs_format::{
     JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT,
-    JOURNAL_SAFETY_FACTOR, ROOT_RING_BASE_SLOT, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
-    UNIT_AREA_START_SLOT,
+    JOURNAL_SAFETY_FACTOR, NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES, ROOT_RING_BASE_SLOT,
+    SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
 };
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice, SECTOR_BYTES};
 
@@ -584,12 +584,11 @@
 
 /// 树表条目里种类那 2 字节的偏移（树 ID 8 + 条目长度 2 之后，D8（核心索引结构） 已定项 8）。
 const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
-/// 第一个文件那一版树表里的条目数（七棵进树表的树）。
-const TREE_TABLE_ENTRIES_OF_A_FILE_VERSION: usize = 7;
-
-/// 树表里种类是 `kind` 的那一条的下标。
+/// 树表里种类是 `kind` 的那一条的下标；找遍第一个文件那一版树表的全部条目（格式常量 `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES`）。
 fn index_of_the_entry_of_kind(node: &[u8], kind: u16) -> usize {
-    (0..TREE_TABLE_ENTRIES_OF_A_FILE_VERSION)
+    let tree_table_entries_of_a_file_version =
+        usize::try_from(NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES).expect("树表条目数装得进 usize");
+    (0..tree_table_entries_of_a_file_version)
         .find(|index| {
             let entry = &node[entry_range_in_node(node, *index)];
             u16::from_le_bytes([
--- a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -443,7 +443,7 @@
 /// C511（回退到无文件那一版之后诞生代怎么接） 第 3 步给 `publish_first_file` 新加的两道，都在任何写之前返回：
 /// ① 判「那一版有没有过文件版本」改看它的树表条数，而那一版的树表单元两份都读不出（两盘上 mkfs 那片树表整个写零）
 ///    ⇒ `TreeTableOfTheVersionToBuildOnUnreadable`，带着恢复路径那一格的原因；
-/// ② 八棵树从那一版的水位（盘上读来的 8 字节）起连号发，而水位离 `u64::MAX` 不到八个号
+/// ② 十二棵树从那一版的水位（盘上读来的 8 字节）起连号发，而水位离 `u64::MAX` 不到十二个号
 ///    ⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`，带着那个水位。
 /// 两格都是一个写、一道屏障都没发（录制流一步不多）、分配器不动。
 #[test]
@@ -474,13 +474,13 @@
     let content = file_content();
 
     // ②：水位离 u64::MAX 只剩三个号。
-    let mut root_with_no_room_for_eight_trees = warm_up_root;
-    root_with_no_room_for_eight_trees.tree_identifier_watermark = u64::MAX - 3;
+    let mut root_with_no_room_for_the_file_version_trees = warm_up_root;
+    root_with_no_room_for_the_file_version_trees.tree_identifier_watermark = u64::MAX - 3;
     let recorded_before_the_watermark_case = formatted.stream.operations().len();
     let refused_for_the_watermark = publish_first_file(
         &mut writer,
         &mut allocator,
-        &root_with_no_room_for_eight_trees,
+        &root_with_no_room_for_the_file_version_trees,
         FirstFile {
             content: &content,
             write_time_seconds: FIXED_WRITE_TIME_SECONDS,
@@ -497,7 +497,7 @@
                 }
             )) if tree_identifier_watermark == u64::MAX - 3
         ),
-        "水位离 u64::MAX 不到八个号：{:?}",
+        "水位离 u64::MAX 不到十二个号：{:?}",
         refused_for_the_watermark.as_ref().err()
     );
     assert_eq!(
--- a/crates/singlefs-harness/tests/write_accounting_per_structure.rs
+++ b/crates/singlefs-harness/tests/write_accounting_per_structure.rs
@@ -179,7 +179,7 @@
     expected
 }
 
-/// 带文件的一版（新池新建文件、覆盖写）：字节表零 t1..t8，数据单元与 inode 树叶容器 32768、其余六个角色 16384，
+/// 带文件的一版（新池新建文件、覆盖写）：字节表零 t1..t12，数据单元与 inode 树叶容器 32768、其余六个角色 16384，
 /// 分配记录树那一个角色按位置寻址之后是五个节点（`ALLOCATION_RECORD_TREE_NODES_REWRITTEN`）。
 fn file_version_publish_by_kind() -> Vec<(WrittenStructureKind, WriteCallsAndBytes)> {
     expected_by_kind(&[
````

## 3、变异表 crates/mutations.tsv 合入前后的差

````diff
@@ -43,7 +43,7 @@
 步 4：被抛弃根的树表读不出时不计数	crates/singlefs-core/src/mount.rs	                None => {\n                    unreadable += 1;\n                    match placements_referenced_by_root_computed_from_the_mapping(	                None => {\n                    match placements_referenced_by_root_computed_from_the_mapping(	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored torn_tree_table_of_an_abandoned_root	torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount
 步 5：抬 F 之后不按新候选集重算影子账	crates/singlefs-core/src/mount.rs	    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {\n        match isolate_slots_referenced_only_by_abandoned_roots(	    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::Off {\n        match isolate_slots_referenced_only_by_abandoned_roots(	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5：影子账的豁免不看候选根的 txg 是否 ≥ F	crates/singlefs-core/src/mount.rs	        !is_abandoned(root)\n            && root.checkpoint_txg >= floor\n            && is_inside_the_window(root.checkpoint_txg)	        !is_abandoned(root) && is_inside_the_window(root.checkpoint_txg)	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
-步 5：抬 F 回收的槽不扣住、生效之前就能发出去	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Admission => ReclaimedReuse::HeldUntilFloorTakesEffect,	            RaiseFloorEntry::Admission => ReclaimedReuse::Immediately,	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored slots_reclaimed_by_raising	slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect
+步 5：抬 F 回收的槽不扣住、生效之前就能发出去	crates/singlefs-core/src/mount.rs	        ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => allocator\n            .reclaim_released_up_to(\n                floor_of_the_reclaim,\n                ReclaimedReuse::HeldUntilFloorTakesEffect,\n            ),	        ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => allocator\n            .reclaim_released_up_to(\n                floor_of_the_reclaim,\n                ReclaimedReuse::Immediately,\n            ),	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored slots_reclaimed_by_raising	slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect
 步 5：抬 F 生效之后不放开扣住的槽	crates/singlefs-core/src/mount.rs	    allocator.release_reclaim_holds();\n    Ok(FloorRaisedThroughTheSequence {	    Ok(FloorRaisedThroughTheSequence {	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5：抬 F 重算影子账时读不出账的被抛弃根不计数	crates/singlefs-core/src/mount.rs	        reclaimed,\n        abandoned_roots_unreadable,\n    })	        reclaimed,\n        abandoned_roots_unreadable: 0,\n    })	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_counts_abandoned_roots	raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable
 步 5：提交内生块的 bump 游标不绕开开段之后才隔离或扣住的槽	crates/singlefs-core/src/allocator.rs	        self.allocated[index] || self.isolated[index] || self.held_until_floor_takes_effect[index]	        self.allocated[index]	-p singlefs-core commit_generated_bump_skips	commit_generated_bump_skips_a_slot_isolated_after_the_segment_was_opened
@@ -315,7 +315,6 @@
 增补 2 收口第 54 行（候选 b）第 ② 条去掉：根槽读得出的那一版也从它的记录再并进一次遍历	crates/singlefs-checker/src/walk.rs	        let root_slot_is_readable = roots.iter().any(|(_, _, root)| {	        let root_slot_is_readable = false && roots.iter().any(|(_, _, root)| {	-p singlefs-harness --test checker_known_bad_images -- version_applied_only_by_its_journal_record_is_walked	version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_the_leak_on_top_still_reddens_it
 D8 已定项 8 ②：可写挂载写行那次的树 ID 水位只沿所选那一版带，不取根环与环里记录的 max（原为 C511 第 3 步回退到暖机根那一格；回退改成向前发布之后由崩溃恢复丢掉一次第一个文件版本那一形护）	crates/singlefs-core/src/mount.rs	        ring.max(version_to_build_on.tree_identifier_watermark)	        version_to_build_on.tree_identifier_watermark	-p singlefs-harness --test writable_mount_of_a_formatted_pool -- the_first_file_version_after_a_recovery_dropped	the_first_file_version_after_a_recovery_dropped_an_earlier_one_issues_its_trees_above_the_ring_watermark_and_reads_back
 C511 第 3 步（D8 已定项 8 ②）：回退行那次发布的树 ID 水位沿回退到的那一版带，不取根环里的 max；偏向回退的随机历史抽样判出 I-7.8（这一档的必红是抽样断言，随测试周期的种子基重验）	crates/singlefs-core/src/mount.rs	        ring.max(version_to_build_on.tree_identifier_watermark)	        version_to_build_on.tree_identifier_watermark	-p singlefs-harness --test tree_identifier_watermark_after_the_first_file_root_is_abandoned	the_row_publish_after_the_first_file_root_is_abandoned_carries_the_highest_watermark_in_the_ring
-C511 第 3 步：从水位起连号发八棵树的号不先判装不装得下（盘上读来的水位离 u64::MAX 不到八个号时越界 panic，不交回错误成员）	crates/singlefs-core/src/transaction.rs	            .checked_add(\n                TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS,\n            )\n            .ok_or(TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {\n                tree_identifier_watermark,\n            })?;	            .wrapping_add(\n                TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS,\n            );	-p singlefs-harness --test writable_mount_of_a_formatted_pool -- first_file_version_that_cannot_judge	first_file_version_that_cannot_judge_or_issue_its_trees_is_refused_before_any_write
 C511 第 3 步：publish_first_file 读不出那一版的树表时当成 0 条接着写，不在写之前交回 TreeTableOfTheVersionToBuildOnUnreadable	crates/singlefs-core/src/transaction.rs	        .map_err(|failure| PublishError::TreeTableOfTheVersionToBuildOnUnreadable { failure })?;	        .unwrap_or(0);	-p singlefs-harness --test writable_mount_of_a_formatted_pool -- first_file_version_that_cannot_judge	first_file_version_that_cannot_judge_or_issue_its_trees_is_refused_before_any_write
 E158 root_choice_repair Q3-1 的 V1（is_abandoned）改成要求全部回退事件都抛弃这个根（该数出「第一次回退抛弃的根也要算」不该数）	crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs	    node.rollback_events\n        .iter()\n        .any(|event| event.abandoned.contains(&root))	    node.rollback_events\n        .iter()\n        .all(|event| event.abandoned.contains(&root))	-p singlefs-checker-tier --bin e158_root_choice_repair	is_abandoned_checks_every_recorded_rollback_event_not_only_the_latest
 E158 root_choice_repair Q3-1 的 V2（content_is_off_the_current_timeline）改成取反（内容在时间线上时反而判违例）	crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs	    !node	    node	-p singlefs-checker-tier --bin e158_root_choice_repair	content_is_off_the_current_timeline_is_false_when_the_content_matches_a_root_still_on_the_timeline
@@ -2172,10 +2171,10 @@
 池级 checker 跟上（D23 已定项 15）：重放那一版不走新根段里的实例表指针	crates/singlefs-checker/src/walk.rs	        self.walk_instance_table_chain(&first_page_pointer, None);\n        let allocation_record_tree_root =	        let allocation_record_tree_root =	-p singlefs-harness --test replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment -- the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment	the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment
 池级 checker 跟上（D23 已定项 15）：重放那一版不走新根段里的分配记录树根	crates/singlefs-checker/src/walk.rs	            &version.new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER];\n        if !parse_node_pointer(allocation_record_tree_root).all_zero {	            &version.new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER];\n        if false && !parse_node_pointer(allocation_record_tree_root).all_zero {	-p singlefs-harness --test replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment -- the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment	the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment
 E171 装置 取号那一核「一份自证过的都没有」写成「不多于两份」、不带故障的盘上每次取号都拒（取代取甲时那一行「少于两槽」写成「不多于两槽」；O1-对拍那一栏该红，装置不记红时这一行没红）	crates/singlefs-core/src/transaction.rs	            if slots_of_this_device.self_verified.is_empty() {	            if slots_of_this_device.self_verified.len() <= 2 {	-p singlefs-checker-tier --bin e171_three_oracle_collusion	an_acquire_and_a_warm_up_on_a_fresh_pool_leave_the_comparison_column_without_a_red
-三·2 删除批第 12 批丙 1（用户 2026-10-03 定「丙 1 进下一批」）：卸载那一串回收的槽照旧扣到生效，续做推满仍放不下之后卸载在预演里取不到落点	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,	            RaiseFloorEntry::Unmount => ReclaimedReuse::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue
-三·2 删除批第 12 批丙 1：卸载那一串回收的槽照旧扣到生效，续做走到空、F 没抬、可删的单元在 defer 里的盘上卸载被拒	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,	            RaiseFloorEntry::Unmount => ReclaimedReuse::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims
-三·2 删除批第 12 批丙 1 (a)：卸载那一串回收的槽照旧扣到生效，先写系统配置之后掉电那一刀摆不出来	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,	            RaiseFloorEntry::Unmount => ReclaimedReuse::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version
-三·2 删除批第 12 批丙 1 (b)：卸载那一串回收的槽照旧扣到生效，只剩一块盘那一刀摆不出来	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,	            RaiseFloorEntry::Unmount => ReclaimedReuse::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor
+三·2 删除批第 12 批丙 1（用户 2026-10-03 定「丙 1 进下一批」）：卸载那一串回收的槽照旧扣到生效，续做推满仍放不下之后卸载在预演里取不到落点	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice,	            RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue
+三·2 删除批第 12 批丙 1：卸载那一串回收的槽照旧扣到生效，续做走到空、F 没抬、可删的单元在 defer 里的盘上卸载被拒	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice,	            RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims
+三·2 删除批第 12 批丙 1 (a)：卸载那一串回收的槽照旧扣到生效，先写系统配置之后掉电那一刀摆不出来	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice,	            RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version
+三·2 删除批第 12 批丙 1 (b)：卸载那一串回收的槽照旧扣到生效，只剩一块盘那一刀摆不出来	crates/singlefs-core/src/mount.rs	            RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice,	            RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor
 三·2 删除批第 12 批代码轮 V1：续做里推的抬 F 自己报了不是空间不够的错时原样交回，不装进 FloorRaiseFailedAfterTheMountsPublishes	crates/singlefs-core/src/mount.rs	                return Err(MountError::FloorRaiseFailedAfterTheMountsPublishes(\n                    Box::new(FloorRaiseFailedAfterTheMountsPublishes {\n                        cause,\n                        writes_of_persisted_publishes: [\n                            writes_of_the_publishes_before_the_continuation,\n                            continuation.writes_of_the_persisted_publishes_in_txg_order(),\n                        ]\n                        .concat(),\n                        floor_raises: floor_raises_pushed_by_the_admission\n                            .iter()\n                            .cloned()\n                            .chain(continuation.floor_raises)\n                            .collect(),\n                    }),\n                ));	                return Err(cause);	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_floor_raise_failing_on_a_device_error_in_the_continuation_hands_back_the_writes_of_the_mounts_publishes
 三·2 删除批第 12 批代码轮 V2：续做那一格交 Publish 时推过的每一串先写系统配置那一步不进错（准入推过的起点）	crates/singlefs-core/src/mount.rs	                writes_before_the_first_publish: system_configuration_writes_of_the_floor_raises(\n                    floor_raises_pushed_by_the_admission\n                        .iter()\n                        .chain(&continuation.floor_raises),\n                ),	                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_continuation_step_failing_after_the_admission_raised_the_floor_hands_back_every_landed_write
 三·2 删除批第 12 批代码轮 V2：续做那一格交 Publish 时推过的每一串先写系统配置那一步不进错（续做推过的起点）	crates/singlefs-core/src/mount.rs	                writes_before_the_first_publish: system_configuration_writes_of_the_floor_raises(\n                    floor_raises_pushed_by_the_admission\n                        .iter()\n                        .chain(&continuation.floor_raises),\n                ),	                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,	-p singlefs-harness --test admission_raises_the_floor_before_refusing	a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write
@@ -2212,7 +2211,6 @@
 LR5 丁-defer 选根一律不并：F 之下、按最新根的表有效的根引用的槽也不并进 I-3.1 的实际遍历；小世界穷举里原式与「被抛弃根引用的也并」那一式在换下槽的那一版读得出时就不同	crates/singlefs-checker/src/walk.rs	        RootStandingInTheNewestRootsTable::Valid => checkpoint_txg < effective_rollback_floor,	        RootStandingInTheNewestRootsTable::Valid => checkpoint_txg < effective_rollback_floor && false,	-p singlefs-checker --lib	deferred_selection_merging_the_abandoned_roots_too_selects_the_same_slots_on_every_small_world_but_the_double_fault_form
 C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型答卸载那一串的根带现行那一版自己的 F、不是它的 txg	crates/singlefs-harness/src/model.rs	        let floor_raised_to_the_current_txg = current.key.checkpoint_txg;	        let floor_raised_to_the_current_txg = current.rollback_floor;	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_the_first_file_at_txg_three	unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees
 C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：实现卸载把 F 抬到现行那一版带的 F（原地不动）而不是它的 txg（E171 第 42 行的形态，由模型对拍判）	crates/singlefs-core/src/mount.rs	    let rollback_floor = current_file_version.root.checkpoint_txg;	    let rollback_floor = current_file_version.root.rollback_floor;	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version	unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees
-C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型在树表 0 条的一版上也答卸载（条款没判的那一格不再停下）	crates/singlefs-harness/src/model.rs	        if !current.contents.has_trees() {\n            return Err(ModelDisagreement::new(\n                ModelDisagreementAspect::NotModeled,\n                format!(\n                    "模型里现行版本 {:?} 树表 0 条：卸载推不推空发布条款没判，模型不答",	        if false {\n            return Err(ModelDisagreement::new(\n                ModelDisagreementAspect::NotModeled,\n                format!(\n                    "模型里现行版本 {:?} 树表 0 条：卸载推不推空发布条款没判，模型不答",	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- the_model_does_not_answer_unmount_on_a_version_without_trees	the_model_does_not_answer_unmount_on_a_version_without_trees
 C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：对拍不比根带的 F（卸载那一串带错 F 不报）	crates/singlefs-harness/src/model.rs	            if expected.rollback_floor != observed.rollback_floor {	            if false {	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- an_unmount_whose_roots_carry_the_floor	an_unmount_whose_roots_carry_the_floor_of_the_current_version_instead_of_its_txg_is_reported_at_the_rollback_floor
 C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型答卸载之后会话照旧开着	crates/singlefs-harness/src/model.rs	        answer.session_after_success = ModelSessionAfterSuccess::Closed;	        answer.session_after_success = ModelSessionAfterSuccess::StaysOpen;	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_the_first_file_at_txg_three	unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees
 C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：胶水只交卸载那一串的第一条根	crates/singlefs-harness/src/model_comparison.rs	            .map(observed_root_of_file_version)\n            .collect(),\n        Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => Vec::new(),	            .map(observed_root_of_file_version)\n            .take(1)\n            .collect(),\n        Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => Vec::new(),	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_the_first_file_at_txg_three	unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees
@@ -2272,3 +2270,22 @@
 C673 ⑥ I-5.5 判法：落点判成恒成立（数据单元搬走的坏镜像不红）	crates/singlefs-checker/src/walk.rs	                    judgements.judge(invariant, smallest_slot == expected_slot, || {	                    judgements.judge(invariant, true, || {	-p singlefs-harness --test checker_known_bad_images -- the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target	the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
 C673 ⑤ I-9.27 判法：extent 树、deleted_inodes、墓碑树条目的头 ID 不比 inode 树的树 ID（头 ID 写 0 的坏镜像不红）	crates/singlefs-checker/src/walk.rs	                inode_tree_identifiers.contains(&head_identifier)	                true	-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
 C673 ⑤ I-9.27 判法：extent 树、deleted_inodes、墓碑树条目的头 ID 拿条目自己的树 ID 比（树 ID 从根环水位发的合法镜像上误红）	crates/singlefs-checker/src/walk.rs	                inode_tree_identifiers.contains(&head_identifier)	                head_identifier == tree	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement	a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recovery_keeps_the_head_identifier_relation
+删除批第 14 批 GA：卸载那一格回收的槽整截当场放开（改回丙1，不按每块盘上最新有效根取小），h2 窄盘续做之后单盘幸存读回	crates/singlefs-core/src/mount.rs	                    floor_of_the_reclaim.min(newest_valid_root_on_every_device),	                    floor_of_the_reclaim,	-p singlefs-harness --test a_lone_surviving_device_reads_back_what_it_read_before_the_unmount -- after_the_first_unmount_publish_of_a_narrow_pool	after_the_first_unmount_publish_of_a_narrow_pool_whose_deletion_was_continued_a_lone_surviving_device_reads_back_what_it_read_before
+删除批第 14 批 GA：卸载那一格回收的槽整截当场放开（改回丙1，不按每块盘上最新有效根取小），h4 4 GiB 盘第二次卸载之后单盘幸存读回	crates/singlefs-core/src/mount.rs	                    floor_of_the_reclaim.min(newest_valid_root_on_every_device),	                    floor_of_the_reclaim,	-p singlefs-harness --test a_lone_surviving_device_reads_back_what_it_read_before_the_unmount -- after_the_first_publish_of_a_second_unmount	after_the_first_publish_of_a_second_unmount_of_a_four_gibibyte_pool_a_lone_surviving_device_reads_back_what_it_read_before
+删除批第 14 批 Q-F 乙：unlink 的意图照旧带这条记录上一版的写入时间，不写调用方给的	crates/singlefs-core/src/transaction.rs	                change_count: change_count_of_the_intent,\n                write_time_seconds,\n	                change_count: change_count_of_the_intent,\n                write_time_seconds: record.write_time_seconds,\n	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- the_intent_writes_the_given_time	the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it
+删除批第 14 批 Q-F 乙：unlink 的中间一批另取时间（取这一批的改动计数），不照抄意图那次写进记录的值	crates/singlefs-core/src/transaction.rs	                write_time_seconds: record.write_time_seconds,	                write_time_seconds: change_count_of_the_batch,	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- the_intent_writes_the_given_time	the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it
+删除批第 14 批 U8：重写墓碑树根的那次发布不按这次的形状算 H，照旧读上一版根节点的码 2 头（长高那次 H 少 1）	crates/singlefs-core/src/transaction.rs	    if resolved.tombstone_tree.is_the_root_rewritten() {\n        return resolved.tombstone_tree.internal_levels();\n    }	    if resolved.tombstone_tree.is_the_root_rewritten() && previous.is_none() {\n        return resolved.tombstone_tree.internal_levels();\n    }	-p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full -- the_third_tombstone_container_grows_the_tree	the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place
+删除批第 14 批 N2：续做那一步在第二个单元写上报块设备错时，挂载交回的失败账是空的（不带失败那一次已落下的写）	crates/singlefs-core/src/mount.rs	            (walk, pool.writes_of_failed_publishes().to_vec())	            (walk, Vec::new())	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- a_continuation_step_failing_on_its_second_unit_write_hands_back_every_landed_write	a_continuation_step_failing_on_its_second_unit_write_hands_back_every_landed_write
+删除批第 14 批 N2：续做那一步在journal 记录写上报块设备错时，挂载交回的失败账是空的（不带失败那一次已落下的写）	crates/singlefs-core/src/mount.rs	            (walk, pool.writes_of_failed_publishes().to_vec())	            (walk, Vec::new())	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- a_continuation_step_failing_on_its_journal_record_write_hands_back_every_landed_write	a_continuation_step_failing_on_its_journal_record_write_hands_back_every_landed_write
+删除批第 14 批 N2：续做那一步在根槽写上报块设备错时，挂载交回的失败账是空的（不带失败那一次已落下的写）	crates/singlefs-core/src/mount.rs	            (walk, pool.writes_of_failed_publishes().to_vec())	            (walk, Vec::new())	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- a_continuation_step_failing_on_its_root_slot_write_hands_back_every_landed_write	a_continuation_step_failing_on_its_root_slot_write_hands_back_every_landed_write
+删除批第 14 批 N2：抬 F 先写系统配置那一步在第二块盘的写上报错时，错里的写账是空的（不带已落下的那一个系统配置写）	crates/singlefs-core/src/transaction.rs	            RollbackFloorOfASystemConfigurationWrite::RaisedFloor(raised_floor),\n        ) {\n            return Err(RaisedFloorSystemConfigurationWriteFailed {\n                devices_carrying_the_raised_floor,\n                cause,\n                writes: pool\n                    .writes_by_structure_kind\n                    .since(&writes_before_this_step),\n            });	            RollbackFloorOfASystemConfigurationWrite::RaisedFloor(raised_floor),\n        ) {\n            return Err(RaisedFloorSystemConfigurationWriteFailed {\n                devices_carrying_the_raised_floor,\n                cause,\n                writes: WritesByStructureKind::NOTHING_WRITTEN,\n            });	-p singlefs-harness --test admission_raises_the_floor_before_refusing -- a_floor_raise_in_the_continuation_failing_on_its_system_configuration_write	a_floor_raise_in_the_continuation_failing_on_its_system_configuration_write_hands_back_every_landed_write
+C511 第 3 步：从水位起连号发十二棵树的号不先判装不装得下（盘上读来的水位离 u64::MAX 不到十二个号时越界 panic，不交回错误成员）	crates/singlefs-core/src/transaction.rs	            .checked_add(\n                TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS,\n            )\n            .ok_or(TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {\n                tree_identifier_watermark,\n            })?;	            .wrapping_add(\n                TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS,\n            );	-p singlefs-harness --test writable_mount_of_a_formatted_pool -- first_file_version_that_cannot_judge	first_file_version_that_cannot_judge_or_issue_its_trees_is_refused_before_any_write
+C670（18-1 P1）判别力（攻方 V242u）：只在带单元的发布上先轮换系统配置、后写根槽，第二截那次挂载的写行崩出「轮换落了、根没落」只经二次崩溃摆得出，第三截再挂判拒得不合法	crates/singlefs-core/src/transaction.rs	    writer.perform_commit_step(CommitStep::Barrier)?;\n    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n        &writes.root_slot,\n        writes.journal_tail,\n        writes.journal_instance,\n    )\n}	    writer.perform_commit_step(CommitStep::Barrier)?;\n    if writes.units.is_empty() {\n        return persist_the_root_then_rotate_the_system_configuration(\n            writer,\n            writes.checkpoint_txg,\n            &writes.root_slot,\n            writes.journal_tail,\n            writes.journal_instance,\n        );\n    }\n    writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail: writes.journal_tail,\n        journal_instance: writes.journal_instance,\n    })?;\n    writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {\n        checkpoint_txg: writes.checkpoint_txg,\n        root_slot: &writes.root_slot,\n    })?;\n    writer.perform_commit_step(CommitStep::Barrier)\n}	-p singlefs-checker-tier --lib -- crash_injection::tests::no_remount_after_a_second_crash_without_read_faults_is_refused_writable_for_a_witness_newer_than_the_selected_version	no_remount_after_a_second_crash_without_read_faults_is_refused_writable_for_a_witness_newer_than_the_selected_version
+C670（18-1 P2）接线：第二截调用点把读故障的认定恒交 Injected（没注入读故障的崩溃态也不判见证判据）	crates/singlefs-checker-tier/src/crash_injection.rs	                read_faults: read_faults_in_the_crash_state,	                read_faults: ReadFaultsInTheCrashState::Injected,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash	a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash
+C670（18-1 P2）接线：第二截换回不判 C670 的旧判官 judge_writable_mount_refusal	crates/singlefs-checker-tier/src/crash_injection.rs	            match judge_writable_mount_refusal_of_a_crash_state(\n                refusal_on_the_crash_image.crash_image_materialized,\n                refusal_on_the_crash_image.version_landed_on_after_the_crash,\n                refusal_error,\n                disk,\n                refusal_on_the_crash_image.read_only_read_back,\n                refusal_on_the_crash_image.read_faults,\n            ) {	            match singlefs_harness::mount_transparency::judge_writable_mount_refusal(\n                refusal_on_the_crash_image.crash_image_materialized,\n                refusal_on_the_crash_image.version_landed_on_after_the_crash,\n                refusal_error,\n                disk,\n                refusal_on_the_crash_image.read_only_read_back,\n            ) {	-p singlefs-checker-tier --lib -- crash_injection::tests::a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash	a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash
+C670（18-1 P1）第三截：再挂被拒那一支只记 I-8.4 没判、不判 C670 那一条	crates/singlefs-checker-tier/src/crash_injection.rs	                .entry(RemountIdempotenceNotJudged::TheRemountWasRefused)\n                .or_insert(0) += 1;\n            judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(	                .entry(RemountIdempotenceNotJudged::TheRemountWasRefused)\n                .or_insert(0) += 1;\n            if tally.remounts_after_a_second_crash > 0 {\n                return;\n            }\n            judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(	-p singlefs-checker-tier --lib -- crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal	a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal
+C670（18-1 P1）第三截：判再挂被拒时读故障恒交 Injected（不沿用第一次崩溃那个态的认定）	crates/singlefs-checker-tier/src/crash_injection.rs	        second_crash_image.read_only_read_back,\n        second_crash_image.read_faults,\n    );	        second_crash_image.read_only_read_back,\n        ReadFaultsInTheCrashState::Injected,\n    );	-p singlefs-checker-tier --lib -- crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal	a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal
+C670（18-1 P1）第三截：判再挂被拒换回不判 C670 的旧判官 judge_writable_mount_refusal	crates/singlefs-checker-tier/src/crash_injection.rs	    let verdict = judge_writable_mount_refusal_of_a_crash_state(\n        second_crash_image.crash_image_materialized,\n        second_crash_image.version_landed_on_after_the_crash,\n        refusal,\n        disk,\n        second_crash_image.read_only_read_back,\n        second_crash_image.read_faults,\n    );	    let verdict = singlefs_harness::mount_transparency::judge_writable_mount_refusal(\n        second_crash_image.crash_image_materialized,\n        second_crash_image.version_landed_on_after_the_crash,\n        refusal,\n        disk,\n        second_crash_image.read_only_read_back,\n    );	-p singlefs-checker-tier --lib -- crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal	a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型在树表 0 条的一版上推空发布（用户 2026-10-04 定不推：一个字节都不写、答零条根）	crates/singlefs-harness/src/model.rs	        let roots = if current.contents.has_trees() {	        let roots = if true {	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees	unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：实现在树表 0 条的一版上卸载写了东西（用户 2026-10-04 定一个字节都不写；盘上逐字节比）	crates/singlefs-core/src/mount.rs	        PoolVersion::WithoutFile(current_version_without_file) => {\n            return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {	        PoolVersion::WithoutFile(current_version_without_file) => {\n            devices[0].1.write_at(crate::address::DeviceOffsetInBytes(1 << 30), &[0xFF; 4096], crate::block_device::WriteDurability::Plain).expect("变异");\n            return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees	unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：对拍不比根的条数（树表 0 条的一版上实现推了空发布不报）	crates/singlefs-harness/src/model.rs	        if expected_roots.len() != observed_roots.len() {	        if false {	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- an_unmount_that_pushes_empty_publishes_on_a_version_without_trees_is_reported_at_the_publish_count	an_unmount_that_pushes_empty_publishes_on_a_version_without_trees_is_reported_at_the_publish_count
````
