# m4-close-batch2-code-r3 被判的改动（2026-11-10）

代码轮第二轮判决之后的修补（B2、H3、H2、两处补用例、mounted_read.rs 注释），产品代码部分；补丁 `~/.local/share/singlefs/work/m4-batch2-r2-fixes-writer/patch/crates.patch`。

```diff
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index 6c3bb79..e2797cf 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -1026,7 +1026,8 @@ impl OpenFileForRead<'_> {
             &mut observation,
         );
         // 读到一半报错（后面一个单元两份都坏、映射里没有、自描述对不上……）的那一次，报错之前已经发出的读、已经读到的只坏一份照样并进打开之后的累计：
-        // 读是真发到盘上的、坏的那一份是真读到的，计不计不看这一次调用成没成。这个累计不去重：一次调用里一个单元只读一遍，跨调用相加。
+        // 读是真发到盘上的、坏的那一份是真读到的，计不计不看这一次调用成没成。这个累计不去重：一次调用里一个单元先按父指针的位置提示读，
+        // 提示不算数时再按中央映射读一遍（[`Self::dereference_data_unit`]），两遍发出的读都照计；跨调用相加。
         // 这一次读自己的观测随报错丢掉，只留在累计里。
         self.mount.add_to_observation_since_open(observation);
         Ok(FileReadOutput {
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 7dc7474..2ee7290 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -514,6 +514,9 @@ pub enum BackgroundCompactionTrigger {
     /// 整理因一批出错停着而不补跑（D26（后台整理与放置回收） 已定项 13 第 4 条），现行那一版里还有活意图（补跑那几批签下的、挂载续做的、停下那一刻没删成的）：
     /// 这一趟只单发一次删意图的发布（同显式入口一趟开头判出停着那一段，[`drop_the_live_intent_on_a_stop`]），停在
     /// [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；删完之后的调用照旧不起跑。预算照后台入口别的起跑给：那一份扣完而前台写得进就不起跑。
+    /// 那一次删意图是整理发布，开关照水位判时发之前过回退门（D26（后台整理与放置回收） 已定项 13 第 5 条）：门关着这一趟停在
+    /// [`CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates`]、一个写都不发，意图与 R 排他留到门开之后那一次调用再删。
+    /// 这一趟不跨调用接着推：放锁的入口排空待发被拒时不记成没完的一趟，下一次调用照停没停重判（解除之后照普通整理判起跑与触发）。
     LiveIntentWhileStoppedAfterAComputeError,
 }
 
@@ -3236,14 +3239,16 @@ impl MountedSession {
             .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
     }
 
-    /// 整理的后台入口（D26（后台整理与放置回收） 已定项 13 第 5、6 条；用户 2026-10-26 定 GC「挪出调用线程，补齐三样保证」）：调用方在它自己的线程上
+    /// 整理的后台入口（D26（后台整理与放置回收） 已定项 13 第 5、6 条：GC 挪出用户改动的调用线程，三样保证照旧）：调用方在它自己的线程上
     /// 一次次调，核心层不起线程、不读钟（钟与前台的 fsync 由 `caller` 答）。每次调用先看起不起跑，依次：
     /// 1. 整理因一批出错停着（第 4 条）而池子每块盘的已分配占它的容量都不超过补跑阈值：现行那一版里有活意图（补跑那几批签下的、挂载续做的、
     ///    停下那一刻没删成的）而上一次用户改动给的那一份 `B_gc` 还剩（或全空段低于底额、前台写不进）⇒ 起跑一趟只单发一次删意图的发布
     ///    （[`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`]，同显式入口一趟开头判出停着那一段），停在
-    ///    [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；没有活意图、或那一份扣完而前台写得进 ⇒ 不起跑，「该跑了」留着，
+    ///    [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；那一次删意图也是整理发布，开关照水位判时发之前同样过回退门（第 5 条），
+    ///    门关着就停在 [`CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates`]、一个写都不发，意图与 R 排他留到门开；
+    ///    没有活意图、或那一份扣完而前台写得进 ⇒ 不起跑，「该跑了」留着，
     ///    一个写都不发（删完意图之后的调用都走这一支）。挂着切到一张卡都用不上的那一组、手里没有活意图时，这一次就在任何写之前报停着；
-    ///    超过阈值就照下面几条照常判起不起跑，起跑之后每一批在 CPU 上补跑（用户 2026-10-28 定案）；
+    ///    超过阈值就照下面几条照常判起不起跑，起跑之后每一批在 CPU 上补跑（第 4 条）；
     /// 2. 用户改动之后起跑判据记下了「该跑了」⇒ 取走，照用户改动之后那一趟推（判决 G8 丁：至多一批）；
     /// 3. 没有 ⇒ 看定时兜底（`pacing.timer_fallback`）：关着、或离上一次查还不到一个间隔就不起跑；到了就再查一次起跑判据，为真就推一趟到停；
     /// 4. 用户改动之后那一趟与定时兜底那一趟都从上一次用户改动给的那一份 `B_gc` 里扣（岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 第 18 行甲，
@@ -3251,7 +3256,8 @@ impl MountedSession {
     ///    （[`BackgroundCompactionNotStarted::PublishBudgetOfTheLastUserChangeSpent`]），等下一次用户改动才再有。闲置时整理发布因此合计不过
     ///    一份 `B_gc`，不随定时兜底查了几次涨。全空段低于底额、前台写不进时破例：那一份扣完也起跑、这一趟至少发 1 次
     ///    （[`CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite`]）；
-    /// 5. 开关照水位判（[`CompactionRunForcing::FollowTheWatermarks`]）时另有三样：头一次判起跑把挂载那一份预算从根环读（挂载不给新预算）；上一趟不发布地停下之后
+    /// 5. 开关照水位判（[`CompactionRunForcing::FollowTheWatermarks`]）时另有三样：头一次判起跑把挂载那一份预算从根环读（挂载不给新预算；排在第 1 条那一判之前，
+    ///    停着时起跑的删意图那一趟同样吃这一份）；上一趟不发布地停下之后
     ///    现行那一版与开关都没变就不重判（[`BackgroundCompactionNotStarted::UnchangedSinceAStopThatPublishedNothing`]；全空段低于底额时停在回退门上的那一次不算）；每一次整理发布之前判回退门
     ///    （[`CompactionRunStop::CompactionPublishWouldLeaveFewerThanFourRollbackStates`]）。
     ///
@@ -3391,20 +3397,12 @@ impl MountedSession {
             .put_the_switch_requested_from_the_next_batch_in_effect();
         let parameters = parameters_and_device_table.parameters_on_disk();
         let background_compute = parameters_and_device_table.background_compute_of_this_mount();
-        if compaction_stays_stopped(allocator, background_compute) {
-            return Ok(start_while_the_compaction_stays_stopped(
-                allocator,
-                current,
-                parameters.geometry.root_ring_slots_per_region.count(),
-                compaction.publishes_since_the_last_user_change,
-            ));
-        }
         let is_judged_as_the_product_does = matches!(
             allocator.compaction().switches.run_forcing,
             CompactionRunForcing::FollowTheWatermarks
         );
         // 挂载不给新预算（D26（后台整理与放置回收） 已定项 13 第 5 条的加门）：这次挂载头一次判后台起跑时，已经过去的发布数取根环里最新那个用户状态之后的根数
-        // （写行、暖机、上一次挂载的整理都在里面）。读不通当那一份已扣完。
+        // （写行、暖机、上一次挂载的整理都在里面）。读不通当那一份已扣完。排在停没停那一判之前：停着而有活意图时起跑的那一趟同样吃这一份。
         if is_judged_as_the_product_does
             && compaction.take_the_reading_of_the_publishes_of_the_mount_from_the_ring()
         {
@@ -3424,6 +3422,14 @@ impl MountedSession {
                 .publishes_since_the_last_user_change
                 .max(publishes_read_from_the_ring);
         }
+        if compaction_stays_stopped(allocator, background_compute) {
+            return Ok(start_while_the_compaction_stays_stopped(
+                allocator,
+                current,
+                parameters.geometry.root_ring_slots_per_region.count(),
+                compaction.publishes_since_the_last_user_change,
+            ));
+        }
         let foreground_cannot_write = foreground_cannot_write_under_the_switches(allocator);
         // 放锁的后台入口上一次调用之后还没完的那一趟接着推：一趟只判一次起跑。整理停着的那一判在上面：停着时这一趟留着，解除之后接着推。
         // 那一份预算扣完就不再推整理，只有两样例外：删除续做不占预算，队列没走空照走；这一趟的破例还没用掉。
@@ -4311,7 +4317,7 @@ fn compaction_stays_stopped(
 
 /// 后台入口判出整理停着而不补跑（[`compaction_stays_stopped`]）之后起不起跑：现行那一版里有活意图，而上一次用户改动给的那一份 `B_gc`
 /// 还剩（或全空段低于底额、前台写不进时破例至少 1 次，同后台入口别的起跑）⇒ 起跑一趟
-/// （[`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`]），那一趟开头判出同一个停着，只单发一次删意图
+/// （[`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`]），那一趟开头判出同一个停着，过了回退门才单发一次删意图
 /// （[`drop_the_live_intent_on_a_stop`]），之后停在 [`CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError`]；
 /// 没有活意图、或那一份扣完而前台写得进 ⇒ 不起跑、一个写都不发，「该跑了」留着。只判，分配器与盘一样都不动。
 fn start_while_the_compaction_stays_stopped(
@@ -4481,6 +4487,7 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
     // D26（后台整理与放置回收） 已定项 13 第 4 条：整理因一批出错停着，等一次通过校验的切换请求解除；停着而池子每块盘的已分配占它的容量都不超过补跑阈值时
     // 删除续做不走：还有活意图（补跑那几批推着的、挂载续做的、停下那一刻没删成的）而这一趟的预算没用完，就单发一次只删意图的发布
     // （[`drop_the_live_intent_on_a_stop`]），之后一个写都不发、当场停；没有活意图或预算用完就一个写都不发。超过阈值就照常推、每一批在 CPU 上补跑。
+    // 删意图排在删除续做之前，队列不空时那一版照抄队列，续做等解除之后。后台入口照水位判时那一次删意图先过回退门（下面那一判）。
     // 挂着切到一张卡都用不上的那一组时，先在这里换上它（一趟开头不在一批之内），与挂载那一刻一样当场判出停着。
     background_compute.put_the_switch_requested_from_the_next_batch_in_effect();
     let budget = match budget_rule {
@@ -4502,8 +4509,32 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
             .max(1)
         }
     };
+    // 回退门只在后台入口、开关照水位判时判（显式入口没有产品调用方，可写挂载续做只推盘上那一条，发布数有上界）。
+    let is_the_rollback_gate_judged = matches!(pacing, PacingOfTheRun::PacedByTheCaller { .. })
+        && matches!(
+            switches.run_forcing,
+            CompactionRunForcing::FollowTheWatermarks
+        );
     if compaction_stays_stopped(allocator, background_compute) {
         let stop = CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError;
+        // 停着时单发的删意图也是一次整理发布，同样过回退门（D26（后台整理与放置回收） 已定项 13 第 5 条）：门关着就不发，意图与 R 排他留到门开；
+        // 全空段低于底额、前台写不进时这一次破例，同循环里每趟的第一次发布。
+        let is_this_drop_the_exception =
+            budget_rule == CompactionRunBudget::AtLeastOnePublishWhileTheForegroundCannotWrite;
+        if budget != 0 && is_the_rollback_gate_judged && !is_this_drop_the_exception {
+            if let Some(stop_of_the_gate) =
+                stop_of_the_rollback_gate(parameters, &*devices, allocator, current)
+            {
+                return finish(
+                    compaction,
+                    current.root().checkpoint_txg,
+                    versions_published,
+                    background_compute_of_each_batch,
+                    0,
+                    CompactionRunOutcome::Stopped(stop_of_the_gate),
+                );
+            }
+        }
         let dropped_on_the_stop = if budget == 0 {
             Ok(CompactionStep::Stopped(stop))
         } else {
@@ -4623,12 +4654,6 @@ fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockD
             ),
         );
     }
-    // 回退门只在后台入口、开关照水位判时判（显式入口没有产品调用方，可写挂载续做只推盘上那一条，发布数有上界）。
-    let is_the_rollback_gate_judged = matches!(pacing, PacingOfTheRun::PacedByTheCaller { .. })
-        && matches!(
-            switches.run_forcing,
-            CompactionRunForcing::FollowTheWatermarks
-        );
     let mut publishes_of_the_run: u64 = 0;
     let mut batches_published: u64 = 0;
     // 循环：每一轮发成一次整理发布或停。迭代上界是预算（每发成一次计一次，到 budget 停）；跨轮携带的是现行那一版、已发的次数与批数。
@@ -7373,7 +7398,8 @@ impl MountedSessionWithPendingChanges {
 
     /// 同 [`Self::run_compaction_in_the_background`]，只差至多一次发布就返回（D26（后台整理与放置回收） 已定项 13 第 5 条：前台与后台经一把锁，每次发布之间放锁）：
     /// 这一趟没完时停因是 [`CompactionRunStop::OnePublishOfTheTripDone`]（或节奏截住），记在会话上，下一次调用不重判起跑、接着推。整理之前的删除续做同样一次调用至多走一步。
-    /// 全空段低于底额时为前台 fsync 让路的那一趟不记（下一趟照样破例）。
+    /// 全空段低于底额时为前台 fsync 让路的那一趟不记（下一趟照样破例）；整理停着时删意图的那一趟
+    /// （[`BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`]）排空待发被拒也不记，下一次调用照停没停重判。
     /// 两次调用之间前台收下的待发在下一次调用先排空。挂着的会话这一层不起线程、不持锁，放锁是调用方的事（运行库 `singlefs_mount_runtime`）。
     ///
     /// # Errors
@@ -7491,6 +7517,11 @@ impl MountedSessionWithPendingChanges {
             Ok(drained) => drained,
             Err(refusal) => {
                 match span {
+                    // 停着删意图那一趟不记成没完：下一次调用照停没停重判（停着就再判删不删，解除了就照普通整理判起跑），
+                    // 不顶着这个触发名接着推普通整理。
+                    SpanOfTheBackgroundCall::AtMostOnePublish
+                        if trigger
+                            == BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError => {}
                     SpanOfTheBackgroundCall::AtMostOnePublish => {
                         self.session.compaction.keep_the_background_trip_under_way(
                             BackgroundTripUnderWay {
```
