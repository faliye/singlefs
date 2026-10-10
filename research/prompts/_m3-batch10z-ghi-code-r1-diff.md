# 根环实现第 10z 批第七、八、九块：代码轮第一轮改动全文（2026-10-08）

三块实现员各交的 `crates.patch` 原样，按合入次序：第七块、第九块、第八块（第七块打在第六块合入之后，第九块打在删除线第 45 批合入之后，第八块打在第九块合入之后）。各块的变异表追加、整行替换附在各自后面。

## 第七块

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index b0312e0..98f29c6 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -2460,7 +2460,7 @@ fn scratch_step_before_booking(
 }
 
 /// 第 `position` 张卡备好了（或建判器时报了 Out of Memory、不判）：后面还有装了状态的卡、队里照此刻的叫停还有领得的块
-/// （[`CardsAtWork::stop_of_the_cards`]），就起下一张的线程（卡一张一张备，不同时编内核）。
+/// （[`CardsAtWork::stop_of_the_cards`]，只在后面有装了状态的卡时问一次），就起下一张的线程（卡一张一张备，不同时编内核）。
 #[cfg(feature = "gpu")]
 fn start_the_next_card_with_states<'scope, 'work>(
     scope: &'scope std::thread::Scope<'scope, 'work>,
@@ -2468,13 +2468,12 @@ fn start_the_next_card_with_states<'scope, 'work>(
     position: usize,
     sender: &std::sync::mpsc::Sender<CardMessage>,
 ) {
-    // 起哪一张、看不看叫停抽在 harness 档（第 10z 批第六块，第 10z 批第五块代码轮判决 K3）
-    if let Some(next_position) = next_card_to_start(
-        work.states_placed,
-        position,
-        &work.blocks,
-        work.stop_of_the_cards(),
-    ) {
+    // 起哪一张、看不看叫停抽在 harness 档（第 10z 批第六块，第 10z 批第五块代码轮判决 K3）；叫停交成闭包，没有下一张卡不问（第 10z 批第七块）
+    if let Some(next_position) =
+        next_card_to_start(work.states_placed, position, &work.blocks, || {
+            work.stop_of_the_cards()
+        })
+    {
         let sender = sender.clone();
         scope.spawn(move || {
             judge_on_one_card_and_start_the_next(scope, work, next_position, sender);
@@ -2716,22 +2715,39 @@ fn the_card_pool_this_run_takes_its_cards_from(
 
 /// 起过卡池的产品路径向卡池借不到时怎么报（`what_was_booked` 说借的是什么）：此刻借不到、借到的卡上起设备时显存不够，
 /// 照对拍用例那一路打「本次未跑」那一行（同一个记号、原因写显存不够）、交 `Ok(那一行)`，调用方照叫停的样子交回；
-/// 要的比每张卡空着时能借的都大、卡上的显存读不出判红。
+/// 要的比每张卡空着时能借的都大、卡上的显存读不出判红。那一行怎么造、哪几种判红在 [`not_run_line_or_refusal_of_a_lease`]，
+/// 这里只多打印那一行。
 ///
 /// # Errors
-/// 要的比每张卡空着时能借的都大（说法与没有一张卡装得下这条流的表那一句同一类，分项驱动照它认成设备用不上）、卡上的显存读不出。
+/// 同 [`not_run_line_or_refusal_of_a_lease`]。
 #[cfg(feature = "gpu")]
 fn print_the_not_run_line_or_refuse(
     what_was_booked: &str,
     refusal: &crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal,
+) -> Result<String, String> {
+    let not_run_line = not_run_line_or_refusal_of_a_lease(what_was_booked, refusal)?;
+    println!("{not_run_line}");
+    Ok(not_run_line)
+}
+
+/// 起过卡池的产品路径向卡池借不到时那一行「本次未跑」或判红的那一句，不打印（第 10z 批第七块：造那一行与打印分开，不要卡的用例只调这一半，
+/// 测试段的标准输出里不出现门禁会收成本次未跑的那一行）：此刻借不到、借到的卡上起设备时显存不够交 `Ok(那一行)`。
+///
+/// # Errors
+/// 要的比每张卡空着时能借的都大（说法与没有一张卡装得下这条流的表那一句同一类，分项驱动照它认成设备用不上）、卡上的显存读不出。
+#[cfg(feature = "gpu")]
+fn not_run_line_or_refusal_of_a_lease(
+    what_was_booked: &str,
+    refusal: &crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal,
 ) -> Result<String, String> {
     use crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal;
     let refusal = match refusal {
         CardPoolLeaseRefusal::Booking(refusal) => refusal,
         CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice { reason } => {
-            let not_run_line = gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, reason);
-            println!("{not_run_line}");
-            return Ok(not_run_line);
+            return Ok(gpu_not_run_line(
+                PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
+                reason,
+            ));
         }
     };
     if let BookingRefusal::NeedLargerThanEveryCard(NeedLargerThanEveryCard {
@@ -2745,10 +2761,7 @@ fn print_the_not_run_line_or_refuse(
         ));
     }
     match scratch_booking_refusal_outcome(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, 0, refusal) {
-        ScratchBookingRefusalOutcome::NotRun { not_run_line } => {
-            println!("{not_run_line}");
-            Ok(not_run_line)
-        }
+        ScratchBookingRefusalOutcome::NotRun { not_run_line } => Ok(not_run_line),
         ScratchBookingRefusalOutcome::Red { reason } => Err(format!("{what_was_booked}：{reason}")),
     }
 }
@@ -3783,13 +3796,14 @@ mod tests {
     }
 
     /// 第 10z 批（第 10y 批代码轮正推 K3 列的第 5 处）：产品路径向卡池借不到时，借到的卡上起设备显存不够与此刻借不到都打本次未跑那一行、
-    /// 交 `Ok`；要的比每张卡空着时都大、卡上的显存读不出判红。不要卡：结局照卡池交回的造。
+    /// 交 `Ok`；要的比每张卡空着时都大、卡上的显存读不出判红。不要卡：结局照卡池交回的造；只调不打印的那一半
+    /// （第 10z 批第七块：打印那一半会把造出来的那一行打进测试段的标准输出，门禁收成本次未跑）。
     #[cfg(feature = "gpu")]
     #[test]
     fn a_device_short_of_gpu_memory_on_the_card_lent_is_reported_not_run_like_no_room_now() {
         use crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal;
         assert!(
-            print_the_not_run_line_or_refuse(
+            not_run_line_or_refusal_of_a_lease(
                 "单元校验和对拍的缓冲",
                 &CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice {
                     reason: "显存不够：起设备时".to_string()
@@ -3800,7 +3814,7 @@ mod tests {
             "起设备时显存不够：本次未跑"
         );
         assert!(
-            print_the_not_run_line_or_refuse(
+            not_run_line_or_refusal_of_a_lease(
                 "单元校验和对拍的缓冲",
                 &CardPoolLeaseRefusal::Booking(BookingRefusal::NoCardHasRoomNow {
                     need_mebibytes: 900,
@@ -3811,7 +3825,7 @@ mod tests {
             "此刻借不到：本次未跑"
         );
         assert!(
-            print_the_not_run_line_or_refuse(
+            not_run_line_or_refusal_of_a_lease(
                 "单元校验和对拍的缓冲",
                 &CardPoolLeaseRefusal::Booking(BookingRefusal::NeedLargerThanEveryCard(
                     NeedLargerThanEveryCard {
@@ -3824,7 +3838,7 @@ mod tests {
             "比每张卡空着时都大：判红"
         );
         assert!(
-            print_the_not_run_line_or_refuse(
+            not_run_line_or_refusal_of_a_lease(
                 "单元校验和对拍的缓冲",
                 &CardPoolLeaseRefusal::Booking(BookingRefusal::CardMemoryUnreadable {
                     reason: "nvidia-smi 起不来".to_string()
@@ -3835,6 +3849,48 @@ mod tests {
         );
     }
 
+    /// 第 10z 批第七块：产品路径借不到时那一行照旧打进标准输出（造那一行与打印分开之后，打印那一半交 `Ok` 之前打它；
+    /// 不要卡的用例调的是不打印的那一半，钉不住这一句）。扫这份源码，不要卡。
+    #[test]
+    fn the_product_path_prints_the_not_run_line_of_a_lease_it_could_not_get() {
+        let lines = code_lines_outside_the_tests();
+        let start = lines
+            .iter()
+            .position(|line| line.trim() == "fn print_the_not_run_line_or_refuse(")
+            .expect("产品路径有打印借不到那一行的那一半");
+        let body = lines[start..]
+            .iter()
+            .take_while(|line| line.as_str() != "}")
+            .map(|line| line.trim().to_string())
+            .collect::<Vec<String>>();
+        let line_made = body
+            .iter()
+            .position(|line| {
+                line == "let not_run_line = not_run_line_or_refusal_of_a_lease(what_was_booked, refusal)?;"
+            })
+            .unwrap_or_else(|| panic!("打印那一半先照不打印的那一半造那一行：{body:?}"));
+        let line_printed = body
+            .iter()
+            .position(|line| line == "println!(\"{not_run_line}\");")
+            .unwrap_or_else(|| panic!("打印那一半要把造出来的那一行打进标准输出：{body:?}"));
+        let line_handed_back = body
+            .iter()
+            .position(|line| line == "Ok(not_run_line)")
+            .unwrap_or_else(|| panic!("打印那一半交回造出来的那一行：{body:?}"));
+        assert!(
+            line_made < line_printed && line_printed < line_handed_back,
+            "先造、再打、再交回：{body:?}"
+        );
+        assert_eq!(
+            lines
+                .iter()
+                .filter(|line| line.contains("print_the_not_run_line_or_refuse("))
+                .count(),
+            4,
+            "产品路径的三处借不到都经打印的那一半（加上它自己的定义一处）"
+        );
+    }
+
     /// 一块里第一个判不了的状态：恢复内核的「判不了」位或 checker 内核的「判不了」字，哪一样都算；全判得了是 `None`。
     #[cfg(feature = "gpu")]
     #[test]
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
index e8837d6..adb7858 100644
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -2125,19 +2125,21 @@ pub fn judge_blocks_on_one_card<Judged>(
 }
 
 /// 第 `position` 张卡备好了（或建判器时报了 Out of Memory、不判）之后起哪一张卡的线程（崩溃放量产品路径第 ① 段，卡一张一张备、不同时编内核）：
-/// 后面第一张装了状态的卡（`states_placed` 按这一轮卡的次序，0 是没装），而且队里照此刻的 `stop` 还有领得的块
+/// 后面第一张装了状态的卡（`states_placed` 按这一轮卡的次序，0 是没装），而且队里照此刻的叫停还有领得的块
 /// （[`BlocksForTheCards::any_left_to_take`]：叫停了只剩判完的块不在等的，不起）；否则不起（第 10z 批第五块代码轮判决 K3：
 /// 这一步抽到 harness 档，起下一张卡时看不看叫停不要卡就测得到）。
+/// 叫停只在后面有装了状态的卡时才问 `stop_of_the_cards_now`、问一次，没有下一张卡一次都不问（第 10z 批第七块：
+/// 抽出来那一版不管有没有下一张卡都先问，每起一张卡多问一次；调用方的叫停可以按「问第几次」变，多问一次卡上就少领一块）。
 #[must_use]
 pub fn next_card_to_start(
     states_placed: &[u64],
     position: usize,
     blocks: &BlocksForTheCards,
-    stop: StopOfTheCards,
+    stop_of_the_cards_now: impl FnOnce() -> StopOfTheCards,
 ) -> Option<usize> {
     let next_position =
         (position + 1..states_placed.len()).find(|candidate| states_placed[*candidate] > 0)?;
-    if blocks.any_left_to_take(stop) {
+    if blocks.any_left_to_take(stop_of_the_cards_now()) {
         Some(next_position)
     } else {
         None
@@ -6009,32 +6011,64 @@ mod tests {
     fn the_next_card_starts_only_when_it_holds_states_and_a_block_is_left_to_take_under_the_stop() {
         let blocks = BlocksForTheCards::new([0usize, 1, 2]);
         assert_eq!(
-            next_card_to_start(&[1, 0, 1], 0, &blocks, StopOfTheCards::NotRequested),
+            next_card_to_start(&[1, 0, 1], 0, &blocks, || StopOfTheCards::NotRequested),
             Some(2),
             "跳过没装状态的那一张"
         );
         assert_eq!(
-            next_card_to_start(&[1, 0, 1], 0, &blocks, StopOfTheCards::RequestedByTheCaller),
+            next_card_to_start(&[1, 0, 1], 0, &blocks, || {
+                StopOfTheCards::RequestedByTheCaller
+            }),
             None,
             "叫停了、还没有判完的块：没有判完的块在等的，不起"
         );
         assert_eq!(
-            next_card_to_start(&[1, 0, 1], 0, &blocks, StopOfTheCards::TheRunFailed),
+            next_card_to_start(&[1, 0, 1], 0, &blocks, || StopOfTheCards::TheRunFailed),
             None,
             "判红了：不起"
         );
         assert_eq!(
-            next_card_to_start(&[1, 0, 1], 2, &blocks, StopOfTheCards::NotRequested),
+            next_card_to_start(&[1, 0, 1], 2, &blocks, || StopOfTheCards::NotRequested),
             None,
             "后面没有卡了"
         );
         assert_eq!(
-            next_card_to_start(&[1, 0], 0, &blocks, StopOfTheCards::NotRequested),
+            next_card_to_start(&[1, 0], 0, &blocks, || StopOfTheCards::NotRequested),
             None,
             "后面的卡没装状态"
         );
     }
 
+    /// 第 10z 批第七块：起下一张卡时叫停只在后面有装了状态的卡时问、问一次；后面没有卡、后面的卡都没装状态，一次都不问
+    /// （崩溃放量产品路径的叫停可以按「问第几次」变，多问一次卡上就少领一块）。
+    #[test]
+    fn the_next_card_asks_for_the_stop_once_only_when_a_later_card_holds_states() {
+        let blocks = BlocksForTheCards::new([0usize, 1, 2]);
+        let times_asked = std::cell::Cell::new(0usize);
+        let stop_counting_the_asks = || {
+            times_asked.set(times_asked.get() + 1);
+            StopOfTheCards::NotRequested
+        };
+        assert_eq!(
+            next_card_to_start(&[1, 0, 1], 2, &blocks, stop_counting_the_asks),
+            None,
+            "最后一张卡之后没有卡"
+        );
+        assert_eq!(times_asked.get(), 0, "后面没有卡：叫停一次都不问");
+        assert_eq!(
+            next_card_to_start(&[1, 0, 0], 0, &blocks, stop_counting_the_asks),
+            None,
+            "后面的卡都没装状态"
+        );
+        assert_eq!(times_asked.get(), 0, "后面的卡都没装状态：叫停一次都不问");
+        assert_eq!(
+            next_card_to_start(&[1, 0, 1], 0, &blocks, stop_counting_the_asks),
+            Some(2),
+            "后面有装了状态的卡、没叫停：起它"
+        );
+        assert_eq!(times_asked.get(), 1, "后面有装了状态的卡：叫停恰好问一次");
+    }
+
     /// 接住一段会 panic 的活，交它 panic 了没有。
     fn panics_when_run(work: impl FnOnce()) -> bool {
         std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).is_err()
diff --git a/crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs b/crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs
index 6ffa0bc..275de16 100644
--- a/crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs
+++ b/crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs
@@ -197,12 +197,9 @@ fn start_the_next_card<'scope, 'work>(
         .iter()
         .map(|card| card.states_placed(work.round))
         .collect();
-    if let Some(next_position) = next_card_to_start(
-        &states_placed,
-        position,
-        &work.blocks,
-        work.stop_of_the_cards(),
-    ) {
+    if let Some(next_position) = next_card_to_start(&states_placed, position, &work.blocks, || {
+        work.stop_of_the_cards()
+    }) {
         let thread_number = work.next_thread_number.fetch_add(1, Ordering::SeqCst);
         work.scheduler.register_a_thread();
         scope.spawn(move || card_thread(scope, work, next_position, thread_number));
````

### 第七块 mutations-append.tsv

````
第10z批第七块 K3 起下一张卡不管后面有没有装了状态的卡都先问叫停（第六块合入那一形，每起一张卡多问一次）	crates/singlefs-harness/src/gpu_memory_ledger.rs	    let next_position =\n        (position + 1..states_placed.len()).find(|candidate| states_placed[*candidate] > 0)?;\n    if blocks.any_left_to_take(stop_of_the_cards_now()) {	    let stop = stop_of_the_cards_now();\n    let next_position =\n        (position + 1..states_placed.len()).find(|candidate| states_placed[*candidate] > 0)?;\n    if blocks.any_left_to_take(stop) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_next_card_asks_for_the_stop_once_only_when_a_later_card_holds_states	the_next_card_asks_for_the_stop_once_only_when_a_later_card_holds_states
第10z批第七块 崩溃放量 起下一张卡之前先问叫停（第六块合入那一形：只用一张卡时叫停之前少领一块）	crates/singlefs-checker-tier/src/crash_amplification.rs	        next_card_to_start(work.states_placed, position, &work.blocks, || {\n            work.stop_of_the_cards()\n        })	        next_card_to_start(work.states_placed, position, &work.blocks, {\n            let stop = work.stop_of_the_cards();\n            move || stop\n        })	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest	the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest
第10z批第七块 崩溃放量 产品路径借不到时不打本次未跑那一行（打印那一半去掉 println）	crates/singlefs-checker-tier/src/crash_amplification.rs	    let not_run_line = not_run_line_or_refusal_of_a_lease(what_was_booked, refusal)?;\n    println!("{not_run_line}");\n	    let not_run_line = not_run_line_or_refusal_of_a_lease(what_was_booked, refusal)?;\n	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::the_product_path_prints_the_not_run_line_of_a_lease_it_could_not_get	the_product_path_prints_the_not_run_line_of_a_lease_it_could_not_get
````

### 第七块 mutations-replacements.tsv

````
第10z批第六块 K3 起下一张卡不看叫停与队里领不领得	crates/singlefs-harness/src/gpu_memory_ledger.rs	    if blocks.any_left_to_take(stop_of_the_cards_now()) {	    if true || blocks.any_left_to_take(stop_of_the_cards_now()) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_next_card_starts_only_when_it_holds_states_and_a_block_is_left_to_take_under_the_stop	the_next_card_starts_only_when_it_holds_states_and_a_block_is_left_to_take_under_the_stop
第10z批 崩溃放量 K3 借到的卡上起设备显存不够判红（不报本次未跑）	crates/singlefs-checker-tier/src/crash_amplification.rs	            return Ok(gpu_not_run_line(\n                PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,\n                reason,\n            ));	            return Err(reason.clone());	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::a_device_short_of_gpu_memory_on_the_card_lent_is_reported_not_run_like_no_room_now	a_device_short_of_gpu_memory_on_the_card_lent_is_reported_not_run_like_no_room_now
````

## 第九块

````diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 5460d26..2ffa836 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1915,7 +1915,8 @@ fn placements_in_the_accounts_of_unapplied_publishes<Device: BlockDevice>(
 /// 有效（`is_abandoned` 判否）、读得出的根——F2 退回之后被退掉的那条根就是这样：它的根槽读得出、它自己的实例表里它有效，影子账只管被抛弃的根、
 /// 不管它；它的记录又全读不出时 C 那两份（记录点名的、记录拼根读的账）都是空的。照影子账同一份读法（[`placements_referenced_by_root`]）读它的账，
 /// 账里的落点按它的 txg 在根环里对应的槽（[`target_for_publish`]）归组交回，调用方并进 C 的集合、同一个时机解开。账读不出时照没施加的发布那一份办：
-/// 那次发布读得出的记录不齐就从中央映射现算（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]，仍不成拒），记录齐就不交（交 F1 乙）；
+/// 那次发布读得出的记录不齐就从中央映射现算（[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]，仍不成拒），记录齐就不交账（交 F1 乙）、
+/// 改交挂载那一刻现行账（`current_records`）里仍分配着的落点（丙，用户 2026-10-08 定，函数体里那一支的注释）；
 /// 一条记录都读不出的，映射现算得出就交；现算不成时，`journal_instance_in_the_system_configuration`（挂载择到的系统配置里取过号的实例代号，
 /// `SystemRuntimeQuantities::journal_instance`）不比那条根新就照记录不齐那一支办（仍不成拒），比它新就不交也不拒（函数体里那一支的注释）。
 /// `roots` 是重建分配器读根环那一遍读出的根（影子账、根环表用的同一份）。
@@ -1997,7 +1998,14 @@ fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
                             before_the_one_reread,
                         )?
                     } else {
-                        continue;
+                        // 记录齐：账不交（交 F1 乙），C 那一槽另并上挂载那一刻现行账里仍分配着的落点（丙，D16（发布语义） 已定项 1
+                        // 「根槽这一次读坏」那一行末句，用户 2026-10-08 定「改成丙」，判决
+                        // `research/prompts/m3-batch10z-d-code-r1-main-verification.md` 第三节；攻方改法丙只在攻方模型上量过、被攻过零轮）。
+                        // 这条根建在所选那一版上：它引用的单元要么是它自己写的（记录点名了，C 那一份里有），要么在所选那一版的账里，
+                        // 也就是现行账里这一截。它们这时分配着、影子账照豁免（第九项不数两遍）；这次挂载里释放之后，抬 F 借 C 那一槽
+                        // （写行之后这条根被抛弃，H2）与进程内回收（`allocator::PoolAllocator::record_root_written_by_this_process`）照这一份补隔离。
+                        // 不并它时，抬 F 只借到记录点名的那几个，它从所选那一版继承、被会话释放的单元回收出去（攻方 K2-2，I-7.4 红）。
+                        placements_still_allocated_in_the_account(current_records)
                     }
                 }
             };
@@ -2013,6 +2021,17 @@ fn placements_in_the_accounts_of_newer_readable_roots<Device: BlockDevice>(
     Ok(in_the_accounts_of_newer_roots_by_ring_slot)
 }
 
+/// 一份账里仍分配着的每个落点（盘、起点槽、跨度），按记录的次序。只读内存。
+fn placements_still_allocated_in_the_account(
+    records: &[AllocationRecord],
+) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
+    records
+        .iter()
+        .filter(|record| !record.is_released)
+        .map(|record| (record.device, record.slot, u64::from(record.span_slots)))
+        .collect()
+}
+
 /// 水位之上一次没施加的发布（或比所选那一版新的可读根）的账读不出、那次发布读得出的记录又不齐时，它引用的落点照被抛弃根那一份办
 /// （`C_MAPPING` 收窄，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1、H3 两行与第三节第 1 条）：
 /// 从中央映射现算、重读一次、看 C694 的 B 那道凭据（[`placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread`]，
@@ -3095,16 +3114,17 @@ struct RootsOfTheRingForTheCeiling {
 /// 上限与「非空」在环里全部有效根上取：原来同取一圈窗口的六处之一（B2，用户 2026-10-03 定），用户 2026-10-04 定 RGall 一起退回整个环
 /// （`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）。回收门槛同时退回整个环，窗口外的有效根引用的单元不再被回收，
 /// 读它的树表读得到它那一版（B2 当初挡的攻方 K2-b2 那一形——窗口许可复用之后读窗口外旧根的树表读到别的内容——随之没有了）。
-/// 根环怎么读见 [`roots_of_the_ring_for_the_ceiling`]：`ring_slots_known_to_hold_a_root` 取这个进程分配器上那张根环表
-/// （`allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）；分配器上没装根环表的（只有用例自己拼的分配器，
-/// 可写挂载与 `make_filesystem::allocator_after_make_filesystem` 都装）给空集，读坏的槽一律当没有根。
+/// 根环怎么读见 [`roots_of_the_ring_for_the_ceiling`]：`ring_slots_whose_reread_still_bad_refuses_the_ceiling` 是这一遍读坏、重读一次仍坏就拒的槽，
+/// 取这个进程分配器上那张根环表里知道住着根、又没记成被抛弃根的槽（[`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`]，
+/// 与抬 F 那一串同一个口径：被抛弃根的槽读坏不拒、当没有根、不进上限，D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定 H1）；
+/// 分配器上没装根环表的（只有用例自己拼的分配器，可写挂载与 `make_filesystem::allocator_after_make_filesystem` 都装）给空集，读坏的槽一律当没有根。
 /// 这一份不知道哪几个槽是挂载那一刻读坏的（给手里没有这个进程的根环表的调用方：实验装置、用例）；这个进程的准入抬 F 走
 /// [`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]。
 ///
 /// # Errors
 /// 一条有效根一条都没有 ⇒ `RecoveryFailure::NoValidRoot`；要比的一条有效根的树表读不出或解不开 ⇒
-/// `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（读不出要走修复，不按空或非空猜）；知道住着根的根环槽重读仍坏 ⇒
-/// `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
+/// `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（读不出要走修复，不按空或非空猜）；
+/// `ring_slots_whose_reread_still_bad_refuses_the_ceiling` 里的根环槽重读仍坏 ⇒ `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
 #[allow(
     clippy::ptr_arg,
     reason = "user_visible_tree_root_pointers 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
@@ -3114,14 +3134,14 @@ pub fn rollback_floor_ceiling<Device: BlockDevice>(
     system_configuration: &crate::system_configuration::SystemConfiguration,
     current_floor: CheckpointTxg,
     table: &InstanceTableRecords,
-    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
+    ring_slots_whose_reread_still_bad_refuses_the_ceiling: &BTreeSet<RootRingSlot>,
 ) -> Result<CheckpointTxg, MountError> {
     rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
         devices,
         system_configuration,
         current_floor,
         table,
-        ring_slots_known_to_hold_a_root,
+        ring_slots_whose_reread_still_bad_refuses_the_ceiling,
         &BTreeMap::new(),
         &BTreeMap::new(),
     )
@@ -3551,7 +3571,8 @@ pub fn ring_slots_bad_at_mount_by(
 }
 
 /// 这个进程分配器上那张根环表里知道住着一条根的槽（`allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）；
-/// 没装根环表的分配器（只有用例自己拼的）给空集。算抬 F 的上限读根环按它分两类（[`rollback_floor_ceiling`]）。
+/// 没装根环表的分配器（只有用例自己拼的）给空集。挂着时管理员回退判候选读根环按它分两类；算抬 F 的上限除去记成被抛弃根的槽
+/// （[`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`]）。
 #[must_use]
 pub fn ring_slots_known_to_hold_a_root_by(allocator: &PoolAllocator) -> BTreeSet<RootRingSlot> {
     allocator.root_ring_occupancy().map_or_else(
@@ -3606,27 +3627,36 @@ fn rollback_floor_ceiling_of_this_process<Device: BlockDevice>(
     table: &InstanceTableRecords,
     allocator: &PoolAllocator,
 ) -> Result<CheckpointTxg, MountError> {
-    let ring_slots_recorded_as_holding_an_abandoned_root =
-        ring_slots_recorded_as_holding_an_abandoned_root_by(allocator);
-    let ring_slots_known_to_hold_a_root_whose_reread_still_bad_refuses_the_ceiling: BTreeSet<
-        RootRingSlot,
-    > = ring_slots_known_to_hold_a_root_by(allocator)
-        .into_iter()
-        .filter(|ring_slot| {
-            !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)
-        })
-        .collect();
     rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount(
         devices,
         system_configuration,
         current_floor,
         table,
-        &ring_slots_known_to_hold_a_root_whose_reread_still_bad_refuses_the_ceiling,
+        &ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(allocator),
         &ring_slots_bad_at_mount_by(allocator),
         &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator),
     )
 }
 
+/// 算抬 F 的上限读根环时，这一遍读坏、重读一次仍坏就拒的槽（[`rollback_floor_ceiling`] 的 `ring_slots_whose_reread_still_bad_refuses_the_ceiling`）：
+/// 这个进程分配器上那张根环表里知道住着根的槽（[`ring_slots_known_to_hold_a_root_by`]），除去记成被抛弃根的那几槽
+/// （[`ring_slots_recorded_as_holding_an_abandoned_root_by`]；D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-10-08 定 H1
+/// 「被抛弃根的槽不拒」：读坏就当没有根，上限只在有效根上取、本来不看它们）。这个进程的准入抬 F（[`rollback_floor_ceiling_of_this_process`]）
+/// 与自算上限的调用方（实验装置、checker 档的运行模式）都取这一份。没装根环表的分配器给空集。
+#[must_use]
+pub fn ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(
+    allocator: &PoolAllocator,
+) -> BTreeSet<RootRingSlot> {
+    let ring_slots_recorded_as_holding_an_abandoned_root =
+        ring_slots_recorded_as_holding_an_abandoned_root_by(allocator);
+    ring_slots_known_to_hold_a_root_by(allocator)
+        .into_iter()
+        .filter(|ring_slot| {
+            !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)
+        })
+        .collect()
+}
+
 /// 一次准入最多推几次发布：B = 4 + 2 k_tol，k_tol = 2 ⇒ 8（D16（发布语义） 已定项 1「准入」那一行）。数的是这次准入推的空发布
 /// 加上这次准入为之判的那一次发布（发布路径是那次用户发布，可写挂载那一处是写行那次）：`df` 的保留池按 B − 1 次空发布留
 /// （同一张表「`df`」那一行：保留池 = 2 × 5 + (B − 1) × c_max）。
@@ -4130,7 +4160,9 @@ fn raise_the_floor_through<Device: BlockDevice>(
     // 改之前照盘上重读，挂上之后它的账读坏就照 B 拒，正常卸载与会话写都被拒（攻方 `research/prompts/m3-batch10y-code-r1-opus-output.md` 「K4-2」）。
     // 表里没记着这条被抛弃根、挂载那一刻那一槽记成同一条有效根（写行之后才被抛弃）的，借 C 那一圈在那一槽记着的落点
     // （用户 2026-10-08 定 H2「用 C 那一圈记着的落点」，判决 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第三节）：
-    // 同样不读账、读不出不拒。两样都没记着的照旧读盘（[`PlacementsOfAbandonedRoots`] 那一臂的注释）。
+    // 同样不读账、读不出不拒。两样都没记着的照旧读盘（[`PlacementsOfAbandonedRoots`] 那一臂的注释）。那条根挂载那一刻账读不出而记录齐时，
+    // C 那一槽另带着挂载那一刻现行账里仍分配着的落点（丙，用户 2026-10-08 定，[`placements_in_the_accounts_of_newer_readable_roots`]）。
+    // C 那一槽有条目而落点是空的（凭据放行那一形），照空串算、不读账（「照 C 那一圈记着的落点算」，与根环表里记着空串同一处置）。
     let placements_recorded_for_abandoned_roots: BTreeMap<
         (InstanceGeneration, CheckpointTxg),
         Vec<PlacementOnDevice>,
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index f3d6d71..c331fee 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -1150,7 +1150,8 @@ impl RootRingOccupancy {
     /// 两遍里只有一遍自证过的有效那一半带整条根记录），而 C 那一圈（水位之上没施加的记录点名的落点、那几次发布的账、比所选那一版新的可读根的账，
     /// [`RootRingOccupancy`] 那一项）在那一槽还记着、没解开，就交回那一槽记着的落点；那一槽记的是被抛弃根、挂载时读坏或读回全零的槽、
     /// 有效根而 txg 不同（同一槽差一整圈的另一次发布），或 C 在那一槽什么都没记（这个进程写掉那一槽时解开了，或挂载时就没记），交 `None`，
-    /// 调用方照今天读盘。只读内存里的表，不读盘。
+    /// 调用方照今天读盘。C 在那一槽有条目而落点是空的（凭据放行那一形）交 `Some(空)`：条目在就是记着了，调用方照空串算、不读账。
+    /// 只读内存里的表，不读盘。
     #[must_use]
     pub fn placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
         &self,
@@ -3396,6 +3397,34 @@ mod tests {
         );
     }
 
+    /// C 那一圈在那一槽有条目、落点是空的（正推 `research/prompts/m3-batch10z-d-code-r1-sonnet-output.md` K1-2：较新可读根或没施加的发布的账
+    /// 读不出、记录不齐、映射也现算不成，而现行账里有 C694 的 B 那道凭据时，`mount::placements_of_an_unapplied_publish_whose_account_is_unreadable`
+    /// 交空串、照样记进 C 那一槽）：借到的是 `Some(空)`，不是 `None`。D16（发布语义） 已定项 1「根槽这一次读坏」那一行：C 那一圈在那一槽记着
+    /// 同一条根时「照 C 那一圈记着的落点算、不读账、读不出不拒」，「C 也没记着的照今天读盘」——条目在就是记着了，空串照空串算，
+    /// 与根环表里记着空串（挂载时凭据放行，攻方 M7）同一处置；凭据说的是那条根引用的单元已经没有要护的。
+    #[test]
+    fn placements_named_by_unapplied_journal_records_for_a_root_recorded_valid_lend_an_empty_entry_as_some_empty(
+    ) {
+        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
+        let mut occupancy =
+            occupancy_with_every_kind_of_slot_from_txg_11_through_16_and_unapplied_records_on_each(
+            );
+        occupancy
+            .placements_named_by_unapplied_journal_records
+            .insert(
+                target_for_publish(CheckpointTxg(15), slots_per_region),
+                Vec::new(),
+            );
+        assert_eq!(
+            occupancy.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+                InstanceGeneration(2),
+                CheckpointTxg(15)
+            ),
+            Some(Vec::new()),
+            "那一槽记的是同一条有效根、C 在那一槽有条目而落点是空的：借到空串（照它算、不读账），不交 None（照今天读盘）"
+        );
+    }
+
     /// 环上有洞留下的旧根压住进程内的回收门槛（D16（发布语义） 已定项 1「环里最旧有效根」，用户 2026-10-04 定 RGall）：24 槽的环里
     /// txg 1 那一槽留着 txg 1 的根（txg 25、49 那两次的根槽都没落），其余 23 槽住着 txg 26–48；这个进程记到 txg 49（没有根的那一个）
     /// 之后再记 txg 50，盖掉的是 txg 26。txg 1 离最新根已过两圈，仍是环里最旧的有效根：释放代 2 的落点不回收、仍在 defer 里。
diff --git a/crates/singlefs-checker-tier/src/on_device_modes.rs b/crates/singlefs-checker-tier/src/on_device_modes.rs
index 77a956d..c43c076 100644
--- a/crates/singlefs-checker-tier/src/on_device_modes.rs
+++ b/crates/singlefs-checker-tier/src/on_device_modes.rs
@@ -10,8 +10,8 @@ use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator, RootRingOccupancy,
 use singlefs_core::block_device::BlockDevice;
 use singlefs_core::make_filesystem::{MakeFilesystemParameters, ROLLBACK_FLOOR_AT_MKFS};
 use singlefs_core::mount::{
-    raise_rollback_floor, ring_slots_known_to_hold_a_root_by, rollback_floor_ceiling, MountError,
-    RaisedFloor, ShadowLedger,
+    raise_rollback_floor, ring_slots_whose_reread_still_bad_refuses_the_ceiling_by,
+    rollback_floor_ceiling, MountError, RaisedFloor, ShadowLedger,
 };
 use singlefs_core::recovery::{choose_system_configuration, instance_table_chain_of_root};
 use singlefs_core::root_ring::RootRingSlot;
@@ -310,7 +310,8 @@ pub fn publish_the_fourth_version<Device: BlockDevice>(
 /// `current_version` 随之换成最后落盘的那一次空发布（失败时是最后落盘的那一次，见 [`MountError::RaiseFloorSequencePublishFailed`]）。
 ///
 /// 上限照 D16（发布语义） 已定项 1「抬 F 的上限」由 core 的 [`rollback_floor_ceiling`] 现算（min(每块盘上最新的持久有效根,
-/// 第 4 新的非空持久有效根)），入参与 [`raise_rollback_floor`] 自己算上限时的一样：现行那一版的根指着的实例表、现行的 F、盘上读得到的根。
+/// 第 4 新的非空持久有效根)），入参与 [`raise_rollback_floor`] 自己算上限时的一样：现行那一版的根指着的实例表、现行的 F、盘上读得到的根；
+/// 读坏、重读仍坏就拒的槽与抬 F 那一串同一个口径（[`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`]：被抛弃根的槽读坏不拒、不进上限）。
 /// 这里算完到 `raise_rollback_floor` 再算一次之间没有写，两次读的是同一份盘上状态；对不上时 `raise_rollback_floor`
 /// 在任何写之前拒绝（`RollbackFloorAboveCeiling`）。
 ///
@@ -333,7 +334,7 @@ pub fn raise_the_rollback_floor_to_its_ceiling<Device: BlockDevice>(
         &system_configuration,
         current_version.root.rollback_floor,
         &instance_table,
-        &ring_slots_known_to_hold_a_root_by(allocator),
+        &ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(allocator),
     )?;
     raise_rollback_floor(
         parameters,
diff --git a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
index 7361f07..ae4748e 100644
--- a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
+++ b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
@@ -31,7 +31,7 @@ use singlefs_core::make_filesystem::{
     allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
 };
 use singlefs_core::mount::{
-    mount_writable, raise_rollback_floor, ring_slots_known_to_hold_a_root_by,
+    mount_writable, raise_rollback_floor, ring_slots_whose_reread_still_bad_refuses_the_ceiling_by,
     roll_back_by_a_forward_publish, rollback_floor_ceiling, Mounted, RollbackTarget, ShadowLedger,
 };
 
@@ -1962,7 +1962,8 @@ fn floor_targets_between(
 /// op1 第三种变体的目标 floor 枚举：先不注入地做一次 `mount_writable`（试探性，用完即弃，只为量出
 /// 这一步的 F 与上限——它本身不是受注入的一步），从它交回的 `current.root.rollback_floor` 起算 F，
 /// 再用 `crates/` 已有的 `rollback_floor_ceiling`（与 `raise_rollback_floor` 内部用的是同一个函数）
-/// 算出上限。交回空 `Vec`：探测步骤失败（挂载不成功、没有文件、系统配置或实例表读不出、
+/// 算出上限（读坏、重读仍坏就拒的槽与抬 F 那一串同一个口径，`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`：
+/// 被抛弃根的槽读坏不拒、不进上限）。交回空 `Vec`：探测步骤失败（挂载不成功、没有文件、系统配置或实例表读不出、
 /// `rollback_floor_ceiling` 报错），或 F 已经等于上限（没有余量可抬）——两种都不是「该测到却漏了」，
 /// 是这条历史在这一格没有 op1 第三种变体可跑，调用方按 `raise_floor_history_nodes_without_room`
 /// 单独计一次，不归进任何触发计数。
@@ -1989,7 +1990,7 @@ fn raise_floor_targets_for(
         &system_configuration,
         current_floor,
         &table.records,
-        &ring_slots_known_to_hold_a_root_by(&mounted.allocator),
+        &ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(&mounted.allocator),
     ) else {
         return Vec::new();
     };
@@ -22958,6 +22959,26 @@ mod tests {
                     FaultInjectingBlockDevice::new(identity, device, plan),
                 ));
             }
+            // 注入确实打在抬 F 那一遍要读的地方（正推 `research/prompts/m3-batch10z-d-code-r1-sonnet-output.md` K1-3）：抬 F 之前，
+            // 经同一组带故障的盘读这条被抛弃根的分配记录树节点的每一份（落点照 `locate_allocation_record_tree` 从盘上现找，不取 `fault_targets`），
+            // 都读不出——抬 F 那一遍要是读它的账，就撞上注入。注入落空（偏移不对、故障没装上、严重度给成空集）时这里读得出，这一条红。
+            for location in &allocation_locations {
+                let (_, faulted_device) = faulted_devices
+                    .iter()
+                    .find(|(identity, _)| *identity == location.device)
+                    .expect("池里有这块盘");
+                let mut node_bytes = vec![
+                    0u8;
+                    usize::try_from(singlefs_format::SLOT_BYTES)
+                        .expect("槽宽装得进 usize")
+                ];
+                assert!(
+                    faulted_device
+                        .read_at(location.slot.to_device_offset(), &mut node_bytes)
+                        .is_err(),
+                    "注入打在这条被抛弃根的分配记录树节点 {location:?} 上：抬 F 那一遍去读它就读不出"
+                );
+            }
             raise_rollback_floor(
                 &parameters,
                 &mut faulted_devices,
diff --git a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
index caea814..801757e 100644
--- a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -17,6 +17,10 @@
 //! - H2「用 C 那一圈记着的落点」：可写挂载自己写行之后才判成被抛弃的根 R（`tests/common_unwitnessed_publish/mod.rs` 那块盘，挂载那一刻
 //!   R 的账之外写出的单元读回全零、择根退到 P，R 记成有效根），挂上之后 R 的账读报错：抬 F 照 C 那一圈在 R 那一槽记着的落点算、不读账、不拒。
 //! - 挂载那一刻凭据放行的被抛弃根，根环表里记着的是空串：抬 F 照空串算、不再读它的账（攻方 M7）。
+//! - H2 之上的丙（用户 2026-10-08 定「改成丙」，判决 `research/prompts/m3-batch10z-d-code-r1-main-verification.md` 第三节）：R 换成建一个 inode
+//!   （与所选那一版 P 共用文件的数据单元），挂载那一刻 R 的账读回全零而记录齐，C 那一槽另并上那一刻现行账里仍分配着的落点；
+//!   会话覆盖写释放了那几个共用的数据单元、推抬 F、再顺序写，池级 checker 不红 I-7.4。只有 H2、没有丙时这一形红（攻方 K2-2）。
+//! - 公开的抬 F 上限（`rollback_floor_ceiling`）照抬 F 那一串的口径取拒集（正推 K1-1）：C 的根槽读坏不拒、不进上限。
 // 池（384 槽那一档小盘）与 R 那块盘都经 `tests/common_unwitnessed_publish/mod.rs` 起（它转出 `tests/common_admission_pool/mod.rs` 的全部）：
 // 再声明 `common_admission` 就把 `common_admission_pool` 当模块装两遍（clippy `duplicate_mod`）。
 mod common_unwitnessed_publish;
@@ -26,12 +30,14 @@ use std::rc::Rc;
 
 use common_unwitnessed_publish::{
     checker_violations_on, device_bytes_of_the_384_slot_small_devices,
-    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
+    devices_reading_back_all_zero_on, devices_reading_back_all_zero_on_until_lifted,
+    fixed_geometry_of_the_384_slot_small_devices, image_of_devices_reading_back_all_zero,
     parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on, pool_after_the_mount,
-    record_one_publish_after_overwrites, DeviceOverASparseImage, PoolUnderTest, PublishRecordedAsR,
-    RangesReadingBackAllZero, OVERWRITE_BYTES,
+    record_one_publish_after_overwrites, root_is_in_the_ring, units_of_r_overwritten_on,
+    DeviceOverASparseImage, DeviceReadingBackAllZeroOnRanges, PoolUnderTest, PublishRecordedAsR,
+    RangesReadingBackAllZero, ReadBackAllZeroLift, RecordedPublish, OVERWRITE_BYTES,
 };
-use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::allocator::{AllocationRecord, PlacementOnDevice, PoolAllocator};
 use singlefs_core::block_device::{
@@ -40,14 +46,22 @@ use singlefs_core::block_device::{
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
 use singlefs_core::mount::{
     mount_writable, mount_writable_with_space_admission, mount_writable_with_test_only_switches,
-    unmount, MountError, RollbackTarget, ShadowLedger, StillUnreadableAfterOneReread, Unmounted,
+    ring_slots_known_to_hold_a_root_by, ring_slots_whose_reread_still_bad_refuses_the_ceiling_by,
+    rollback_floor_ceiling, unmount, MountError, RollbackTarget, ShadowLedger,
+    StillUnreadableAfterOneReread, Unmounted,
 };
 use singlefs_core::mounted_session::MountedSession;
 use singlefs_core::pointer::{LocationEntry, NodePointer};
+use singlefs_core::recovery::{choose_system_configuration, instance_table_chain_of_root};
 use singlefs_core::root_record::RootRecord;
 use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
 use singlefs_core::transaction::PoolVersion;
-use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
+use singlefs_harness::memory_pool::{
+    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
+    SparseDevice,
+};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
 use std::collections::BTreeMap;
 
 /// C 那一次覆盖写的内容长度（攻方原型的 2500 字节）：与之前之后的覆盖写长度不同，读回哪一版看得出。
@@ -399,29 +413,64 @@ fn mounted_while_the_abandoned_root_reads(
     (pool, recorded)
 }
 
-/// 抬 F 之后 `placements` 里每个落点在分配器里都不空闲：隔离着，或现行账里还分配着。被抛弃根离开根环之前它引用的单元不许发出去
-/// （D23（journal 的角色与格式） 已定项 14 回退段主句）；抬 F 越过一条候选根之后，此前靠它豁免、现行账里已释放的落点只靠影子账这一遍补的隔离护着。
+/// 抬 F 之后 `placements` 里每个落点覆盖的每一槽（起点槽起、跨度那么多槽）在分配器里都不空闲：隔离着，或现行账里还分配着。
+/// 被抛弃根离开根环之前它引用的单元不许发出去（D23（journal 的角色与格式） 已定项 14 回退段主句）；抬 F 越过一条候选根之后，
+/// 此前靠它豁免、现行账里已释放的落点只靠影子账这一遍补的隔离护着。只核起点槽时，补隔离只隔起点那一槽（跨度给成 1）照绿
+/// （正推 `research/prompts/m3-batch10z-d-code-r1-sonnet-output.md` K3 `loop-span1`）。
 fn assert_none_of_the_placements_is_free(
     allocator: &PoolAllocator,
     placements: &[PlacementOnDevice],
     whose: &str,
 ) {
-    let free_placements: Vec<&PlacementOnDevice> = placements
+    let free_slots = free_slots_covered_by(allocator, placements);
+    assert!(
+        free_slots.is_empty(),
+        "抬 F 之后{whose}的落点覆盖的槽一槽都不空闲（隔离着或现行账里还分配着）：空闲的（盘、槽）{free_slots:?}（共 {} 个落点）",
+        placements.len()
+    );
+}
+
+/// `placements` 里每个落点覆盖的槽（起点槽起、跨度那么多槽）里此刻在分配器里空闲的那几个（盘、槽），按落点的次序。
+fn free_slots_covered_by(
+    allocator: &PoolAllocator,
+    placements: &[PlacementOnDevice],
+) -> Vec<(DeviceIdentity, SlotNumber)> {
+    placements
         .iter()
-        .filter(|placement| {
-            allocator
+        .flat_map(|placement| {
+            let device_map = allocator
                 .devices
                 .iter()
                 .find(|device_map| device_map.device == placement.device)
-                .expect("池里有这块盘")
-                .is_free(placement.slot)
+                .expect("池里有这块盘");
+            (placement.slot.0..placement.slot.0 + placement.span)
+                .map(SlotNumber)
+                .filter(move |slot| device_map.is_free(*slot))
+                .map(move |slot| (placement.device, slot))
         })
-        .collect();
-    assert!(
-        free_placements.is_empty(),
-        "抬 F 之后{whose}的落点一槽都不空闲（隔离着或现行账里还分配着）：空闲的 {free_placements:?}（共 {} 个落点）",
-        placements.len()
-    );
+        .collect()
+}
+
+/// R 自己写出的每个单元的落点（盘、起点槽、跨度：写长按槽宽折成槽数）：从录下来的写算，不经被测的方法。
+fn placements_written_by(recorded: &RecordedPublish) -> Vec<PlacementOnDevice> {
+    recorded
+        .unit_writes()
+        .into_iter()
+        .map(|write| {
+            assert_eq!(
+                write.offset.0 % singlefs_format::SLOT_BYTES,
+                0,
+                "单元写落在槽边界上：{write:?}"
+            );
+            PlacementOnDevice {
+                device: write.device,
+                slot: SlotNumber(write.offset.0 / singlefs_format::SLOT_BYTES),
+                span: write
+                    .length_in_bytes()
+                    .div_ceil(singlefs_format::SLOT_BYTES),
+            }
+        })
+        .collect()
 }
 
 /// 故障开着之后 C 的树表那一槽确实读不出（阳性对照：用例不是在读得出的盘上空转）。
@@ -713,6 +762,76 @@ fn session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable
     );
 }
 
+/// 公开的抬 F 上限（`rollback_floor_ceiling`），读坏、重读仍坏就拒的槽取 `ring_slots`：系统配置、现行那一版的实例表与 F 都照这一刻的盘与会话现读。
+fn public_rollback_floor_ceiling_of(
+    pool: &PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
+    ring_slots: &std::collections::BTreeSet<RootRingSlot>,
+) -> Result<singlefs_core::address::CheckpointTxg, MountError> {
+    let system_configuration = choose_system_configuration(&pool.devices).expect("系统配置读得出");
+    let table = instance_table_chain_of_root(&pool.devices, &pool.current_file_version().root)
+        .expect("现行那一版的实例表读得出")
+        .records;
+    rollback_floor_ceiling(
+        &pool.devices,
+        &system_configuration,
+        pool.rollback_floor(),
+        &table,
+        ring_slots,
+    )
+}
+
+/// K1-1（正推 `research/prompts/m3-batch10z-d-code-r1-sonnet-output.md` K1-1 末段公开上限函数那一形）：挂上之后 C 的根槽读报错。自算上限的调用方
+/// （checker 档 `raise-rollback-floor` 模式、e158 装置）照抬 F 那一串的口径取读坏重读仍坏就拒的槽
+/// （`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`：除去根环表记成被抛弃根的槽），公开的 `rollback_floor_ceiling` 不拒，
+/// C 那一槽当没有根、不进上限：算出的上限与故障关着时相同。阳性对照：同一刻照旧口径传「知道住着根的槽」整集就拒、点名 C 那一槽
+/// （`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，改之前 checker 档那两处调用方就是这样）。
+#[test]
+fn the_public_rollback_floor_ceiling_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_equals_the_ceiling_while_it_reads(
+) {
+    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
+    let fault =
+        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
+    let (pool, _recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
+    let ring_slots_whose_reread_still_bad_refuses_the_ceiling =
+        ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(&pool.session().allocator);
+    let ceiling_while_the_slot_reads = public_rollback_floor_ceiling_of(
+        &pool,
+        &ring_slots_whose_reread_still_bad_refuses_the_ceiling,
+    )
+    .expect("故障关着：上限算得出");
+    fault.is_active.set(true);
+    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
+    let ring_slot_of_the_abandoned_root = target_for_publish(
+        built.abandoned_root.checkpoint_txg,
+        built.parameters.geometry.root_ring_slots_per_region,
+    );
+    match public_rollback_floor_ceiling_of(
+        &pool,
+        &ring_slots_known_to_hold_a_root_by(&pool.session().allocator),
+    ) {
+        Err(MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
+            ring_slot,
+            ..
+        }) => assert_eq!(
+            ring_slot, ring_slot_of_the_abandoned_root,
+            "旧口径（知道住着根的槽整集）拒、点名 C 那一槽"
+        ),
+        other => panic!("阳性对照：旧口径下 C 的根槽读坏重读仍坏就拒，而是：{other:?}"),
+    }
+    match public_rollback_floor_ceiling_of(
+        &pool,
+        &ring_slots_whose_reread_still_bad_refuses_the_ceiling,
+    ) {
+        Ok(ceiling) => assert_eq!(
+            ceiling, ceiling_while_the_slot_reads,
+            "被抛弃根的槽读坏当没有根、不进上限：上限与故障关着时相同"
+        ),
+        Err(refusal) => {
+            panic!("被抛弃根的槽读坏不拒（与抬 F 那一串同一口径），而拒了：{refusal:?}")
+        }
+    }
+}
+
 /// H1 的另一半：有效根的槽照「这个进程知道住着根的槽」那一句，读坏重读仍坏就拒。挂上之后这次挂载择到的那条有效根（实例 3 退出时现行那一版，
 /// 根环表记成有效根）的根槽读报错：正常卸载拒成 `NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot)`、点名那一槽，
 /// 在任何写之前，两块盘逐字节不变。
@@ -832,13 +951,15 @@ fn unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unr
         None,
         "挂载那一刻 R 记成有效根：根环表里没记着它的被抛弃根落点"
     );
-    let named_on_the_ring_slot_of_r = root_ring_table
-        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
-            root_of_r.instance,
-            root_of_r.checkpoint_txg,
-        )
-        .filter(|placements| !placements.is_empty())
-        .expect("C 那一圈在 R 那一槽记着 R（R 的记录点名的、R 的账里的落点）");
+    assert!(
+        root_ring_table
+            .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+                root_of_r.instance,
+                root_of_r.checkpoint_txg,
+            )
+            .is_some_and(|placements| !placements.is_empty()),
+        "C 那一圈在 R 那一槽记着 R（R 的记录点名的、R 的账里的落点）"
+    );
     let image_after_the_mount =
         image_of_devices_reading_back_all_zero(&mount_devices, recorded.pool.device_bytes);
     let fault = ReadFaultOnChosenSlots::on_the_account_of(&root_of_r);
@@ -858,10 +979,11 @@ fn unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unr
         abandoned_roots_unreadable, 0,
         "R 的落点借 C 那一圈在 R 那一槽记着的那一份，这一遍不读它的账"
     );
+    // 期望从录下来的 R 那次发布的写独立算（正推 K1-2 末段：取自被测方法时，借来的那一份被削成一半照绿）。
     assert_none_of_the_placements_is_free(
         &allocator_after_the_unmount,
-        &named_on_the_ring_slot_of_r,
-        "C 那一圈在 R 那一槽记着",
+        &placements_written_by(&recorded),
+        "R 自己写出的单元",
     );
     let violations = violations_of_unit_reuse_on(&pool.image());
     assert!(
@@ -983,3 +1105,288 @@ fn unmount_takes_the_empty_placements_recorded_for_an_abandoned_root_the_credent
         Err(refusal) => panic!("R 记着空串：卸载做得成，而被拒：{refusal:?}"),
     }
 }
+
+/// 丙那一形之前覆盖写几次（攻方 `research/prompts/m3-batch10z-d-code-r1-opus-output.md` 第一节原样那一格 k = 3）。
+const OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_R: u64 = 3;
+
+/// 丙那一形推过第一串抬 F 之后再顺序写几次（每次两个数据单元；攻方原样那一格 `Wss` 的两个 s）。
+const SEQUENTIAL_WRITES_AFTER_THE_FIRST_FLOOR_RAISE: usize = 2;
+
+/// 照 `record_one_publish_after_overwrites` 的录法录 R（崩在 R 的根槽落盘之后、系统配置轮换之前那一份写表），R 换成建一个 inode：
+/// 文件的数据单元不动，R 与它之前那一版 P 共用它们（攻方 `research/prompts/m3-batch10z-d-code-r1-opus-model/src/opus10zd-probe/tests/opus10zd_h2_borrowed.rs`
+/// 的 `record_r` 那一种 `n`）。
+fn record_one_inode_creation_after_overwrites(overwrites_before: u64) -> RecordedPublish {
+    let stream = SharedStream::retaining_contents();
+    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
+        parameters_of_the_384_slot_small_devices(),
+        device_bytes_of_the_384_slot_small_devices(),
+        |identity, device| {
+            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
+        },
+    );
+    for _overwrite in 0..overwrites_before {
+        pool.overwrite(OVERWRITE_BYTES)
+            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
+    }
+    let previous_root = *pool.session().current.root();
+    let previous_content = pool.content_of_the_current_version.clone();
+    let base = pool.image();
+    let operations_before_r = stream.operation_count();
+    pool.create_one_inode().expect("录的那次建 inode 落得下");
+    let root = *pool.session().current.root();
+    let operations = stream.retained_operations();
+    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[operations_before_r..],
+        &fixed_geometry_of_the_384_slot_small_devices(),
+    );
+    let root_indices: Vec<usize> = writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
+        .map(|(index, _)| index)
+        .collect();
+    assert_eq!(
+        root_indices.len(),
+        1,
+        "录的那一次建 inode 只写一条根（准入够，没推抬 F）"
+    );
+    RecordedPublish {
+        content: pool.content_of_the_current_version.clone(),
+        pool,
+        base,
+        writes,
+        root_index: root_indices[0],
+        key: (root.instance, root.checkpoint_txg),
+        previous_key: (previous_root.instance, previous_root.checkpoint_txg),
+        previous_content,
+    }
+}
+
+/// R 的账（树表、中央映射树根、分配记录树根；空指针不算）那几个单元的写，每块盘一份。
+fn account_writes_of(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
+    let root_of_r = recorded.root();
+    [
+        root_of_r.tree_table,
+        root_of_r.mapping_root,
+        root_of_r.allocation_record_tree_root,
+    ]
+    .into_iter()
+    .filter(|pointer| *pointer != NodePointer::empty_root())
+    .flat_map(|pointer| recorded.unit_writes_at(&pointer))
+    .collect()
+}
+
+/// R 引用、R 自己没写的文件数据单元的落点（R 从所选那一版 P 继承的）：录下来的会话里 R 那一版的文件数据指针，
+/// 每条位置条目里 R 一份都没写过的那几个。从录下来的会话与写表算，不经被测的方法。
+fn data_unit_placements_r_shares_with_the_previous_version(
+    recorded: &RecordedPublish,
+) -> Vec<PlacementOnDevice> {
+    let written_by_r = placements_written_by(recorded);
+    recorded
+        .pool
+        .current_file_version()
+        .data_pointers
+        .iter()
+        .flat_map(|pointer| pointer.locations)
+        .map(|location| PlacementOnDevice {
+            device: location.device,
+            slot: location.slot,
+            span: singlefs_format::DATA_UNIT_BYTES.div_ceil(singlefs_format::SLOT_BYTES),
+        })
+        .filter(|placement| {
+            !written_by_r
+                .iter()
+                .any(|written| written.device == placement.device && written.slot == placement.slot)
+        })
+        .collect()
+}
+
+impl DeviceOverASparseImage for DeviceReadingBackAllZeroOnRanges<SparseBlockDevice> {
+    fn sparse_image(&self) -> &SparseDevice {
+        &self.inner.image
+    }
+}
+
+/// 丙那一形挂上之后，R 的账那几段还读不读回全零。
+#[derive(Clone, Copy, Debug)]
+enum AccountOfRAfterTheMount {
+    /// 挂上之后撤掉读回全零（攻方 a2、a3 那几组）。
+    ReadBackAllZeroLifted,
+    /// 挂上之后照旧读回全零，写到哪一段那一段才照读（攻方 a4「故障不撤」）。
+    StillReadingBackAllZero,
+}
+
+/// 丙那一形（攻方 `research/prompts/m3-batch10z-d-code-r1-opus-output.md` 第一节，原样那一格 k = 3、R 建一个 inode、故障 `acct`、动作 `Wss`）：
+/// 第一个文件之后覆盖写 3 次得 P，录一次建 inode 得 R（R 与 P 共用文件的数据单元），崩在 R 的根槽落盘之后、轮换之前；可写挂载那一刻
+/// R 的账（树表、中央映射树根、分配记录树根）读回全零：R 走读不成、择根退到 P，R 的记录齐，C 那两份账都不加，R 的根槽读得出、记成有效根，
+/// 写行之后 R 按现行那一版的表被抛弃。挂上之后照 `after_the_mount` 撤或不撤故障；会话覆盖写到准入推出第一串抬 F（覆盖写释放了 R 与 P 共用的
+/// 数据单元，推的那一刻 R 还在根环里），再顺序写两次。每一步之后池级 checker 不红 I-7.4（近 K 代块未被复用）与 I-4.8，R 写的单元一份都没被写掉；
+/// 推过抬 F 那一刻 R 共用的数据单元与 R 写的单元覆盖的槽一槽都不空闲（期望从录下来的会话与写表独立算）；最后核挂载那一刻 C 那一槽借得到
+/// R 共用的那几个数据单元（丙并上的那一截）。只有 H2、没有丙时抬 F 只借到 R 的记录点名的落点，共用的数据单元回收、被顺序写发出去，I-7.4 红。
+fn the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor(
+    after_the_mount: AccountOfRAfterTheMount,
+) {
+    let recorded = record_one_inode_creation_after_overwrites(
+        OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_R,
+    );
+    let root_of_r = recorded.root();
+    let shared_with_the_previous_version =
+        data_unit_placements_r_shares_with_the_previous_version(&recorded);
+    assert!(
+        !shared_with_the_previous_version.is_empty(),
+        "R 建一个 inode、不写文件的数据单元：它与 P 共用文件的数据单元"
+    );
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let ranges = RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded));
+    let lift = ReadBackAllZeroLift::default();
+    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
+    let mounted = mount_writable_with_space_admission(
+        &recorded.pool.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 走读不成：择根退到 P"
+    );
+    let root_ring_table = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表");
+    assert_eq!(
+        root_ring_table.placements_recorded_for_the_abandoned_root(
+            root_of_r.instance,
+            root_of_r.checkpoint_txg
+        ),
+        None,
+        "挂载那一刻 R 记成有效根：根环表里没记着它的被抛弃根落点（抬 F 那一串借 C 那一圈，H2）"
+    );
+    let borrowed_at_the_mount = root_ring_table
+        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+            root_of_r.instance,
+            root_of_r.checkpoint_txg,
+        )
+        .expect("C 那一圈在 R 那一槽记着 R 的记录点名的落点");
+    match after_the_mount {
+        AccountOfRAfterTheMount::ReadBackAllZeroLifted => lift.lift(),
+        AccountOfRAfterTheMount::StillReadingBackAllZero => {}
+    }
+    let (output, session) = MountedSession::of_the_mount(mounted);
+    let content_of_the_selected_version = recorded
+        .pool
+        .content_of_each_root
+        .get(&(
+            output.effective_root.instance,
+            output.effective_root.checkpoint_txg,
+        ))
+        .cloned()
+        .expect("挂载落到的那一版是录的池写过的一版");
+    let mut pool = PoolUnderTest {
+        parameters: recorded.pool.parameters.clone(),
+        device_bytes: recorded.pool.device_bytes,
+        devices,
+        session: Some(session),
+        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
+        content_of_the_current_version: content_of_the_selected_version,
+        content_of_each_root: recorded.pool.content_of_each_root.clone(),
+    };
+    let assert_nothing_of_r_reused = |pool_after_the_step: &PoolUnderTest<_>, after_what: &str| {
+        let image = pool_after_the_step.image();
+        if root_is_in_the_ring(&image, recorded.key) {
+            let violations = violations_of_unit_reuse_on(&image);
+            assert!(
+                violations.is_empty(),
+                "{after_what}（{after_the_mount:?}），R 的根还在根环里：R 引用的单元一个都没复用：{violations:?}"
+            );
+            assert_eq!(
+                units_of_r_overwritten_on(&recorded, &image),
+                Vec::<(DeviceIdentity, u64)>::new(),
+                "{after_what}（{after_the_mount:?}），R 的根还在根环里：R 写的单元一份都没被写掉"
+            );
+        }
+    };
+    let mut overwrites_before_the_first_floor_raise = 0;
+    // 迭代上界 OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST；跨轮不带状态（计数之外）；出口：推出抬 F 的那一次 break，被拒就当场判红（panic）。
+    loop {
+        assert!(
+            overwrites_before_the_first_floor_raise < OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST,
+            "{OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST} 次覆盖写之内准入推出抬 F（384 槽小盘）"
+        );
+        let root_of_r_was_in_the_ring = root_is_in_the_ring(&pool.image(), recorded.key);
+        let published = pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!(
+                "第 {} 次覆盖写（{after_the_mount:?}）被拒：{refusal:?}",
+                overwrites_before_the_first_floor_raise + 1
+            )
+        });
+        overwrites_before_the_first_floor_raise += 1;
+        assert_nothing_of_r_reused(&pool, "覆盖写之后");
+        if !published.floor_raises.is_empty() {
+            assert!(
+                root_of_r_was_in_the_ring,
+                "准入推抬 F 的那一刻 R 还在根环里（不然这一串里没有被抛弃根要借 C，没走到要判的那一处）"
+            );
+            break;
+        }
+    }
+    // 推过抬 F 那一刻的空闲槽先记下、顺序写之后再判：池级 checker 那一道先判（回收出来的槽被顺序写发出去才红得出），这一道接着判。
+    let free_among_the_shared_after_the_raise =
+        free_slots_covered_by(&pool.session().allocator, &shared_with_the_previous_version);
+    let free_among_the_written_after_the_raise =
+        free_slots_covered_by(&pool.session().allocator, &placements_written_by(&recorded));
+    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_THE_FIRST_FLOOR_RAISE {
+        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
+            .unwrap_or_else(|refusal| {
+                panic!(
+                    "推过抬 F 之后第 {sequential_write_number} 次顺序写（{after_the_mount:?}）被拒：{refusal:?}"
+                )
+            });
+        assert_nothing_of_r_reused(&pool, "推过抬 F 之后顺序写");
+    }
+    assert!(
+        free_among_the_shared_after_the_raise.is_empty(),
+        "推过抬 F 那一刻（{after_the_mount:?}）R 与 P 共用的数据单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_shared_after_the_raise:?}"
+    );
+    assert!(
+        free_among_the_written_after_the_raise.is_empty(),
+        "推过抬 F 那一刻（{after_the_mount:?}）R 自己写出的单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_written_after_the_raise:?}"
+    );
+    let shared_not_borrowed: Vec<&PlacementOnDevice> = shared_with_the_previous_version
+        .iter()
+        .filter(|shared| {
+            !borrowed_at_the_mount
+                .iter()
+                .any(|borrowed| borrowed.device == shared.device && borrowed.slot == shared.slot)
+        })
+        .collect();
+    assert!(
+        shared_not_borrowed.is_empty(),
+        "挂载那一刻 C 那一槽并上了现行账里仍分配着的落点（丙）：R 与 P 共用的数据单元都在借得到的那一份里，缺 {shared_not_borrowed:?}"
+    );
+}
+
+/// 丙（用户 2026-10-08 定「改成丙」）：挂上之后撤掉 R 的账那几段的读回全零（攻方 a2、a3 那几组的取法），流程与断言见
+/// [`the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor`]。
+/// 判别力：撤掉丙（变异「第10z批第九块 丙 较新可读根账读不出而记录齐时 C 那一槽不并现行账里仍分配着的落点」）时 I-7.4 红。
+#[test]
+fn units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again(
+) {
+    the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor(
+        AccountOfRAfterTheMount::ReadBackAllZeroLifted,
+    );
+}
+
+/// 丙：挂上之后 R 的账那几段照旧读回全零（攻方 a4「故障不撤」），流程与断言同上。
+/// 判别力：同上那一条变异时 I-7.4 红。
+#[test]
+fn units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero(
+) {
+    the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor(
+        AccountOfRAfterTheMount::StillReadingBackAllZero,
+    );
+}
````

### 第九块 mutations-append.tsv

````
第10z批第九块 丙 较新可读根账读不出而记录齐时 C 那一槽不并现行账里仍分配着的落点（R 建 inode、与 P 共用的数据单元：挂上之后撤了故障，抬 F 回收、顺序写发出去）	crates/singlefs-core/src/mount.rs	placements_still_allocated_in_the_account(current_records)	placements_still_allocated_in_the_account(&[])	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again
第10z批第九块 丙 较新可读根账读不出而记录齐时 C 那一槽不并现行账里仍分配着的落点（R 建 inode、与 P 共用的数据单元：故障不撤，抬 F 回收、顺序写发出去）	crates/singlefs-core/src/mount.rs	placements_still_allocated_in_the_account(current_records)	placements_still_allocated_in_the_account(&[])	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero
第10z批第九块 K1-1 公开上限函数的调用方取的拒集不除去被抛弃根的槽（挂上之后 C 的根槽读报错：照抬 F 那一串口径算的公开上限也拒）	crates/singlefs-core/src/mount.rs	        .filter(|ring_slot| {\n            !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)\n        })\n        .collect()\n}	        .collect()\n}	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	the_public_rollback_floor_ceiling_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_equals_the_ceiling_while_it_reads
第10z批第九块 K1-2 C 那一槽有条目而落点是空的就不借、交 None（照读盘）	crates/singlefs-core/src/allocator.rs	        self.placements_named_by_unapplied_journal_records\n            .get(&ring_slot)\n            .cloned()\n    }	        self.placements_named_by_unapplied_journal_records\n            .get(&ring_slot)\n            .filter(|placements| !placements.is_empty())\n            .cloned()\n    }	-p singlefs-core --lib -- placements_named_by_unapplied_journal_records_for_a_root_recorded_valid_lend_an_empty_entry	placements_named_by_unapplied_journal_records_for_a_root_recorded_valid_lend_an_empty_entry_as_some_empty
第10z批第九块 K1-3 e158 抬 F 那一遍注入落空（被抛弃根分配记录树节点的故障严重度给成空集）	crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs	            let fault_targets = fault_targets_for(&allocation_locations, FaultSeverity::Both);	            let fault_targets = fault_targets_for(&allocation_locations, FaultSeverity::Empty);	-p singlefs-checker-tier --bin e158_root_choice_repair -- mount_writable_then_raise_floor_reaches_the_injected_fault	mount_writable_then_raise_floor_reaches_the_injected_fault
第10z批第九块 K3 loop-span1 根槽这一遍读坏的被抛弃根记着的落点补隔离只隔起点那一槽（正推 loop-span1：跨两槽的落点后一槽空闲）	crates/singlefs-core/src/mount.rs	        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);\n    }\n    // C 那一份不豁免	        allocator.isolate_abandoned(placement.device, placement.slot, 1);\n    }\n    // C 那一份不豁免	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds
第10z批第九块 K3 h2-first-half-only 借 C 那一槽只交前一半（丙并上的那一截在后半：丙那一形挂载那一刻借得到的那一份缺 R 与 P 共用的数据单元）	crates/singlefs-core/src/allocator.rs	        self.placements_named_by_unapplied_journal_records\n            .get(&ring_slot)\n            .cloned()\n    }	        self.placements_named_by_unapplied_journal_records\n            .get(&ring_slot)\n            .map(|placements| placements[..placements.len() / 2].to_vec())\n    }	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again
第10z批第九块 K3 h2-slot-off-by-one 借 C 那一圈按 txg 加一的槽认（R 那一槽借不到：端到端前提「C 在 R 那一槽记着」红）	crates/singlefs-core/src/allocator.rs	        let ring_slot = target_for_publish(checkpoint_txg, self.slots_per_region);\n        let is_the_same_root_recorded_valid = match	        let ring_slot = target_for_publish(CheckpointTxg(checkpoint_txg.0 + 1), self.slots_per_region);\n        let is_the_same_root_recorded_valid = match	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot
````

### 第九块 mutations-replacements.tsv

````
第10z批第四块 H1 准入算上限那一遍被抛弃根的槽也算重读仍坏就拒（挂上之后 C 的根槽读报错：会话写到推抬 F 那一次拒）	crates/singlefs-core/src/mount.rs	        .filter(|ring_slot| {\n            !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)\n        })\n        .collect()\n}	        .collect()\n}	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements
````

## 第八块

````diff
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -3575,6 +3575,13 @@
 mod tests {
     use super::*;
 
+    // 扫源码的用例里，串里的花括号一律写成 \u 转义：门禁 code-source-discipline 的 mutation-tables 格配花括号时不跳字符串，
+    // 串里落单的半边括号会让它把后面的用例算到别的模块下（第 10z 批第八块）。
+    /// 这份文件测试模块的头两行（扫源码的用例从这里切掉测试模块）。
+    const TESTS_MODULE_OPENING: &str = "#[cfg(test)]\nmod tests \u{7b}";
+    /// 顶格的右括号自成一行（测试模块在这里收尾）。
+    const TOP_LEVEL_CLOSING_BRACE_LINE: &str = "\n\u{7d}\n";
+
     /// 配置点名的卡少了哪几张：按配置的次序列出用不上的序号；都在就是空。
     #[test]
     fn configured_cards_that_are_not_usable_are_named_in_the_configured_order() {
@@ -3860,7 +3867,7 @@
             .expect("产品路径有打印借不到那一行的那一半");
         let body = lines[start..]
             .iter()
-            .take_while(|line| line.as_str() != "}")
+            .take_while(|line| line.as_str() != "\u{7d}")
             .map(|line| line.trim().to_string())
             .collect::<Vec<String>>();
         let line_made = body
@@ -4623,11 +4630,11 @@
         )
         .expect("读得了这份源文件");
         let tests_start = source
-            .find("#[cfg(test)]\nmod tests {")
+            .find(TESTS_MODULE_OPENING)
             .expect("这份文件有测试模块");
         let tests_end = tests_start
             + source[tests_start..]
-                .find("\n}\n")
+                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
                 .expect("测试模块在顶格的右括号处收尾");
         let outside_the_tests = format!("{}{}", &source[..tests_start], &source[tests_end..]);
         let mut names_read = Vec::new();
@@ -4654,11 +4661,11 @@
     fn code_lines_outside_the_tests() -> Vec<String> {
         let source = include_str!("crash_amplification.rs");
         let tests_start = source
-            .find(concat!("#[cfg(test)]", "\n", "mod tests {"))
+            .find(TESTS_MODULE_OPENING)
             .expect("这份文件有测试模块");
         let tests_end = tests_start
             + source[tests_start..]
-                .find("\n}\n")
+                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
                 .expect("测试模块在顶格的右括号处收尾");
         format!("{}{}", &source[..tests_start], &source[tests_end..])
             .lines()
@@ -4681,7 +4688,7 @@
             vec![
                 concat!(
                     "pub fn stop_is_requested_",
-                    "by_the_environment() -> bool {"
+                    "by_the_environment() -> bool \u{7b}"
                 )
                 .to_string(),
                 concat!(
@@ -4701,7 +4708,7 @@
         let lines = code_lines_outside_the_tests();
         let arm = lines
             .iter()
-            .position(|line| line.trim() == "AfterARoundOfTheCards::StoppedOnRequest => {")
+            .position(|line| line.trim() == "AfterARoundOfTheCards::StoppedOnRequest => \u{7b}")
             .expect("第 ① 段几轮卡的收尾里有照叫停收尾那一支");
         let arm_body = lines[arm..]
             .iter()
@@ -4712,8 +4719,67 @@
         assert!(
             arm_body.contains(".not_run_line_of_a_stop_after_an_out_of_memory(")
                 && arm_body.contains("PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE")
-                && arm_body.contains("println!(\"{not_run_line}\");"),
-            "照叫停收尾那一支要打头一句 Out of Memory 那一行：{arm_body}"
+                && arm_body
+                    .lines()
+                    .any(|line| line.trim() == "println!(\"{not_run_line}\");"),
+            "照叫停收尾那一支要把头一句 Out of Memory 那一行打进标准输出（整行比，`eprintln!` 不算）：{arm_body}"
+        );
+    }
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K4，正推第 3 点：checker 档接线五种改回去，两条扫源码用例照绿）：这一趟记住的那一个叫停
+    /// 一路交到一轮收尾（A）、起下一张卡（B）、卡线程判块那一圈（C）与整趟收尾（D）；E（照叫停收尾那一支打进标准输出）在上一条。扫这份源码，不要卡。
+    #[test]
+    fn the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run(
+    ) {
+        let lines: Vec<String> = code_lines_outside_the_tests()
+            .iter()
+            .map(|line| line.trim().to_string())
+            .collect();
+        let count = |expected: &str| {
+            lines
+                .iter()
+                .filter(|line| line.as_str() == expected)
+                .count()
+        };
+        for (expected, what) in [
+            (
+                concat!("let stop = stop_request.", "stop_of_the_run();"),
+                "一轮收尾问这一趟记住的叫停（A）",
+            ),
+            (
+                concat!("&|| work.", "stop_of_the_cards(),"),
+                "卡线程判块那一圈问卡上的叫停（C）",
+            ),
+            (
+                concat!(
+                    "let stop_is_requested = || stop_",
+                    "request.is_requested();"
+                ),
+                "整趟收尾问的是记住的那一个（D）",
+            ),
+            (
+                concat!(
+                    "let stop_was_requested = gpu_memory_was_short || stop_",
+                    "is_requested();"
+                ),
+                "整趟收尾照它判提早收尾（D）",
+            ),
+        ] {
+            assert_eq!(count(expected), 1, "{what}：要有一行 {expected}");
+        }
+        let start = lines
+            .iter()
+            .position(|line| line.starts_with("fn start_the_next_card_with_states<"))
+            .expect("有起下一张卡的那一个函数");
+        let next_card_body: Vec<&String> = lines[start..]
+            .iter()
+            .take_while(|line| line.as_str() != "\u{7d}")
+            .collect();
+        assert!(
+            next_card_body
+                .iter()
+                .any(|line| line.as_str() == concat!("work.", "stop_of_the_cards()")),
+            "起下一张卡照卡上的叫停问（B）：{next_card_body:?}"
         );
     }
 
@@ -6097,6 +6163,10 @@
         reason = "没编进 gpu 特性时单元校验和只在 CPU 上，借不到缓冲那一形只在编进来时造"
     )
 )]
+#[allow(
+    clippy::large_enum_variant,
+    reason = "一趟只造一个、交给调用方就拆开，不进集合：卡池的借据多带一份账上的设备（第 10z 批第八块）之后过了门槛，装箱省不下什么"
+)]
 enum UnitChecksumsOfThisRun {
     /// `on` 是在哪对拍；`lease` 是起过卡池的进程为对拍的缓冲借的那一笔（活到这一段判完），`buffers_booked_mebibytes` 是借的量。
     Ready {
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -17,7 +17,8 @@
 use singlefs_harness::gpu_memory_ledger::{
     card_memory_reader_of_one_report_per_reading, pci_bus_identifier_without_the_domain,
     BusesWithDevicesStartedOutsideTheBooks, CardMemoryReader, CauseOfTheDeviceLoss,
-    DeviceWaitBeforeReturningTheLease, NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
+    DeviceStartedOnTheBooks, DeviceWaitBeforeReturningTheLease,
+    NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
 };
 use singlefs_harness::sha256::DIGEST_BYTES;
 
@@ -41,6 +42,26 @@
     }
 }
 
+/// 卡池的借据丢掉时那一次等：等这张卡的设备上到此刻为止提交的活都做完（`submission_index: None`，E174 跑前登记第五节「等待」那一臂），
+/// 最多等 [`WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE`]（不设上限会让丢借据的那条线程卡在一台出了事的设备上，设 0 等于不等）。
+fn poll_type_before_returning_a_lease() -> wgpu::PollType {
+    wgpu::PollType::Wait {
+        submission_index: None,
+        timeout: Some(WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE),
+    }
+}
+
+/// 卡池借之前对一台设备 poll 一次（`PollType::Poll`，不等，wgpu panic 了是外层的 `Err`）交回的算不算那台设备上提交的活都做完了：
+/// 交队列空了才算（wgpu-core 30 `Device::maintain` 先收完做完的提交、丢掉的缓冲交回驱动，再看队列空没空）；还有在飞的、报错、panic 都不算。
+fn device_found_idle_by_a_poll(
+    polled: &std::thread::Result<Result<wgpu::PollStatus, wgpu::PollError>>,
+) -> bool {
+    match polled {
+        Ok(Ok(wgpu::PollStatus::QueueEmpty)) => true,
+        Ok(Ok(wgpu::PollStatus::WaitSucceeded | wgpu::PollStatus::Poll) | Err(_)) | Err(_) => false,
+    }
+}
+
 /// 空闲显存低于这个数（MiB）的卡不用：本机有卡被别的服务占着时只剩几百 MiB，起上下文都难（E163 M2 那张卡 141 MiB 起不来）。
 /// 只管 [`usable_gpu_cards`] 与 [`gpu_cards_by_priority`]；卡池按账判，此刻被别的进程占着的卡照样进池，借的那一刻借不到就报本次未跑（不等）。
 pub const MINIMUM_FREE_MEMORY_MEBIBYTES: u64 = 2048;
@@ -124,6 +145,10 @@
     compiled_pipelines_by_source_digest: std::sync::Arc<CompiledPipelinesBySourceDigest>,
     /// 这个设备上 wgpu 报过的 Out of Memory 与设备丢没丢，几个克隆共用（[`start_the_card`] 装的回调往里记）。
     device_condition: std::sync::Arc<DeviceConditionOfACard>,
+    /// 卡池起的设备在账上的那一台（[`gpu_test_start::GpuCardPool`] 起好时放进来，不走卡池的是 `None`），几个克隆共用：最后一个克隆丢掉时
+    /// 这一台放掉（`singlefs_harness::gpu_memory_ledger::DeviceStartedOnTheBooks`）。放在最后一个字段：字段照声明的次序丢，
+    /// 这一份最后丢的时候，同一个克隆里的设备、队列与编好的管线已经先丢了（第 10z 批第八块）。
+    device_on_the_books: Option<std::sync::Arc<DeviceStartedOnTheBooks<'static>>>,
 }
 
 /// wgpu 没接住的错误（装在每个设备上，[`start_the_card`]）：Out of Memory 先记进这个设备的情况，再与别的错一样照 wgpu 默认的做法 panic
@@ -404,6 +429,7 @@
         pipeline,
         compiled_pipelines_by_source_digest: std::sync::Arc::default(),
         device_condition,
+        device_on_the_books: None,
     })
 }
 
@@ -1143,11 +1169,11 @@
 
     mod card_pool {
         use super::super::{
-            card_memory_reader_of_the_card_pool, device_wait_before_returning_a_lease,
-            memory_budget_mebibytes_of, nvidia_discrete_cards_reported_by_nvidia_smi,
-            nvidia_smi_memory_and_processes_report, start_the_card, CardStartFailure,
+            card_memory_reader_of_the_card_pool, device_found_idle_by_a_poll,
+            device_wait_before_returning_a_lease, memory_budget_mebibytes_of,
+            nvidia_discrete_cards_reported_by_nvidia_smi, nvidia_smi_memory_and_processes_report,
+            poll_type_before_returning_a_lease, start_the_card, CardStartFailure,
             DeviceOwnerOnTheCard, DiscoveredCard, GpuCard,
-            WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE,
         };
         use super::{
             decide_gpu_test_start, enumerate_vulkan_adapters, not_run_line,
@@ -1160,9 +1186,9 @@
         };
         use singlefs_harness::gpu_memory_ledger::{
             card_ledger_capacity_mebibytes, reason_for_a_booking_without_room_now,
-            reason_for_running_out_of_gpu_memory, BookingRefusal, CardRoomForThisProcess,
-            DeviceStartedOnDemand, DeviceWaitBeforeReturningTheLease, GpuMemoryBooking,
-            GpuMemoryLedger, StatesBookedOnACard,
+            reason_for_running_out_of_gpu_memory, wait_for_the_device_and_take_in_the_outcome,
+            BookingRefusal, CardRoomForThisProcess, DeviceStartedOnDemand, DeviceStartedOnTheBooks,
+            GpuMemoryBooking, GpuMemoryLedger, StatesBookedOnACard,
         };
         use singlefs_harness::numeric_environment_variable::{
             number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
@@ -1207,9 +1233,11 @@
 
         /// 借来的一张卡：[`GpuCardLease::card`] 与池里那张共用设备、编好的内核，显存额度与空闲改成借的量加 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]
         /// （判器照「额度 − 表 − 那一份」开草稿区，开出来正好是借的那么多）。丢掉时先等设备把这张卡上提交了的活做完、丢掉的缓冲交回驱动，
-        /// 再销账（最多等 [`WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE`]；等不成照旧记在账上）。
+        /// 再销账（照 [`poll_type_before_returning_a_lease`] 等；等不成照旧记在账上）。字段照声明的次序丢：卡、账上的那一台、借据。
         pub struct GpuCardLease {
             card: GpuCard,
+            /// 借据那张卡上的设备在账上的那一台（`card` 里那一份的克隆，借据等的就是它）。
+            device_on_the_books: std::sync::Arc<DeviceStartedOnTheBooks<'static>>,
             booking: GpuMemoryBooking<'static>,
         }
 
@@ -1256,31 +1284,26 @@
         impl Drop for GpuCardLease {
             fn drop(&mut self) {
                 // wgpu 丢掉的缓冲要等设备做完这张卡上提交了的活才交回驱动：销账之前等完（用户 2026-10-08 定第 10z 批代码轮岔路 G2「改用乙」，
-                // E174 两张卡上等完之后下一次读数里那笔缓冲都已不在），等不成（超时、wgpu 报错或 panic）这一笔照旧记在账上
-                // （`singlefs_harness::gpu_memory_ledger::GpuMemoryBooking::take_in_the_wait_before_returning`）。
-                // wgpu 把等的时候设备上的错当致命、panic：先把这一笔记成没等成再照样往外抛，不让丢借据那一步把它销了账
+                // E174 两张卡上等完之后下一次读数里那笔缓冲都已不在），等不成（超时、wgpu 报错或 panic）这一笔照旧记在账上；wgpu 把等的时候
+                // 设备上的错当致命、panic：接住、记成没等成，不在展开中照样往外抛、已经在展开中不再抛
+                // （`singlefs_harness::gpu_memory_ledger::wait_for_the_device_and_take_in_the_outcome`，第 10z 批第八块）
                 let device = self.card.device();
-                let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
-                    device.poll(wgpu::PollType::Wait {
-                        submission_index: None,
-                        timeout: Some(WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE),
-                    })
-                }));
-                let wait = device_wait_before_returning_a_lease(&polled);
-                match wait {
-                    DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished => {}
-                    DeviceWaitBeforeReturningTheLease::TheWaitFailed => eprintln!(
-                        "GPU_CARD_LEASE_RETURN_WAIT_FAILED index={} lease={} mebibytes={} polled={:?}：这一笔照旧记在账上，到这张卡的设备丢了重起",
-                        self.card.index,
-                        self.booking.lease_identifier(),
-                        self.booking.mebibytes(),
-                        polled.as_ref().map_err(|_panic| "等的时候 wgpu panic 了")
-                    ),
-                }
-                self.booking.take_in_the_wait_before_returning(wait);
-                if let Err(panic) = polled {
-                    std::panic::resume_unwind(panic);
-                }
+                let index = self.card.index;
+                let lease_identifier = self.booking.lease_identifier();
+                let mebibytes = self.booking.mebibytes();
+                wait_for_the_device_and_take_in_the_outcome(
+                    &mut self.booking,
+                    &self.device_on_the_books,
+                    || device.poll(poll_type_before_returning_a_lease()),
+                    device_wait_before_returning_a_lease,
+                    |polled| {
+                        eprintln!(
+                            "GPU_CARD_LEASE_RETURN_WAIT_FAILED index={index} lease={lease_identifier} mebibytes={mebibytes} polled={:?}：\
+                             这一笔照旧记在账上，到那台设备上提交的活做完了、或那台设备记成丢了、放掉了才销",
+                            polled.as_ref().map_err(|_panic| "等的时候 wgpu panic 了")
+                        );
+                    },
+                );
             }
         }
 
@@ -1303,6 +1326,31 @@
                     )
             }
 
+            /// 借之前：账上有等设备没等成而照旧记着的借据的卡，对池里那张卡留着的设备 poll 一次（不等）；交回队列空了，那台设备上提交的活
+            /// 都做完了、丢掉的缓冲交回了驱动，那几笔销账（第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：等超时的那一笔不永久少借，
+            /// 两次超时之间没有别的等成时也出得来）。留着的设备丢了的不 poll（wgpu 在丢了的设备上 poll 会 panic）：那几笔到重起时照重起记销。
+            /// 不在账的锁里 poll：队列空了是在 poll 那一刻，那几笔丢掉的缓冲早在那之前提交、丢掉，之后什么时候销都不多给。
+            fn take_in_the_devices_found_idle(&self) {
+                for card_position in self.ledger.cards_with_leases_kept_after_a_failed_wait() {
+                    let Some(started) = self.cards[card_position].started.started_now() else {
+                        continue;
+                    };
+                    if started.device_condition().is_lost() {
+                        continue;
+                    }
+                    let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+                        started.device().poll(wgpu::PollType::Poll)
+                    }));
+                    if device_found_idle_by_a_poll(&polled) {
+                        started
+                            .device_on_the_books
+                            .as_ref()
+                            .expect("卡池起的设备起好时放进了账上的那一台（started_card）")
+                            .take_in_that_the_submitted_work_finished();
+                    }
+                }
+            }
+
             /// 借到 `booking` 的那张卡起好的设备：留着的设备没丢就拿它，没有（第一次要、上一次起不来）或丢了就起一个
             /// （借到就登记了这个进程，先登记、后建设备）；起不来交为什么、不留着。要一笔借据才起得了：没借过的卡起不了设备。
             /// 丢了的设备重起之前，驱动那一份在账上重新登记，此刻放不下就不起、交显存不够
@@ -1311,7 +1359,7 @@
             /// # Panics
             /// 这个进程还没在这张卡的账上登记（`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`）。
             fn started_card(
-                &self,
+                &'static self,
                 booking: &GpuMemoryBooking<'static>,
             ) -> Result<GpuCard, CardStartFailure> {
                 let card_position = booking.card_position();
@@ -1344,9 +1392,11 @@
                             })?;
                         let mut started =
                             start_the_card(&card.discovered, DeviceOwnerOnTheCard::TheCardPool)?;
-                        self.ledger
+                        let device_on_the_books = self
+                            .ledger
                             .record_the_device_started_on_the_card(card_position);
                         started.memory_quota_mebibytes = card.configured_card_quota_mebibytes;
+                        started.device_on_the_books = Some(std::sync::Arc::new(device_on_the_books));
                         Ok(started)
                     },
                 )
@@ -1382,6 +1432,7 @@
                 test_name: &str,
                 need_mebibytes: u64,
             ) -> Result<GpuCardLease, String> {
+                self.take_in_the_devices_found_idle();
                 let booking = match self.ledger.book_on_the_first_card_with_room(need_mebibytes) {
                     Ok(booking) => booking,
                     Err(BookingRefusal::NeedLargerThanEveryCard(refusal)) => panic!(
@@ -1419,6 +1470,7 @@
                 run_name: &str,
                 need_mebibytes: u64,
             ) -> Result<GpuCardLease, CardPoolLeaseRefusal> {
+                self.take_in_the_devices_found_idle();
                 let booking = self
                     .ledger
                     .book_on_the_first_card_with_room(need_mebibytes)
@@ -1466,6 +1518,7 @@
                     .iter()
                     .map(|index| self.position_of_the_card_with_index(*index))
                     .collect();
+                self.take_in_the_devices_found_idle();
                 let booked = self
                     .ledger
                     .book_states_on_the_cards_with_room(
@@ -1521,6 +1574,11 @@
                         panic!("{test_name}：{reason}：账上登记了这个进程，卡却起不来上下文（驱动坏了）")
                     }
                 };
+                let device_on_the_books = std::sync::Arc::clone(
+                    card.device_on_the_books
+                        .as_ref()
+                        .expect("卡池起的设备起好时放进了账上的那一台（started_card）"),
+                );
                 card.memory_quota_mebibytes = Some(need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER);
                 card.free_memory_mebibytes = need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER;
                 eprintln!(
@@ -1529,7 +1587,11 @@
                     card.index,
                     booking.card_room_mebibytes_when_booked()
                 );
-                Ok(GpuCardLease { card, booking })
+                Ok(GpuCardLease {
+                    card,
+                    device_on_the_books,
+                    booking,
+                })
             }
 
             /// 这个进程此刻与这几张卡空着时，在 `card_indices` 这几张卡（都是池里的卡）上各能借多少（MiB，现读一次；与 `card_indices` 逐张对上）。
@@ -1547,6 +1609,7 @@
                     .iter()
                     .map(|index| self.position_of_the_card_with_index(*index))
                     .collect();
+                self.take_in_the_devices_found_idle();
                 self.ledger
                     .card_rooms_now_and_when_empty(&card_positions)
                     .map_err(|refusal| format!("读不出卡上此刻的显存：{refusal:?}"))
@@ -1835,16 +1898,24 @@
         PciNvidiaDisplayControllers, PCI_DEVICES_SUBDIRECTORY_NAME,
     };
     use super::{
-        card_memory_reader_of_the_card_pool, cause_of_the_device_loss,
+        card_memory_reader_of_the_card_pool, cause_of_the_device_loss, device_found_idle_by_a_poll,
         device_wait_before_returning_a_lease, failure_of_a_device_request,
         handle_an_uncaptured_wgpu_error, has_devices_started_outside_the_card_pool_on_the_bus,
-        record_the_loss_of_the_device, started_with_its_owner_recorded, CardStartFailure,
-        DeviceConditionOfACard, DeviceOwnerOnTheCard,
+        poll_type_before_returning_a_lease, record_the_loss_of_the_device,
+        started_with_its_owner_recorded, CardStartFailure, DeviceConditionOfACard,
+        DeviceOwnerOnTheCard,
     };
     use singlefs_harness::gpu_memory_ledger::{
         CauseOfTheDeviceLoss, DeviceWaitBeforeReturningTheLease, OccupiedByThisProcess,
     };
 
+    // 扫源码的用例里，串里的花括号一律写成 \u 转义：门禁 code-source-discipline 的 mutation-tables 格配花括号时不跳字符串，
+    // 串里落单的半边括号会让它把后面的用例算到别的模块下（第 10z 批第八块）。
+    /// 这份文件测试模块的头两行（扫源码的用例从这里切掉测试模块）。
+    const TESTS_MODULE_OPENING: &str = "#[cfg(test)]\nmod tests \u{7b}";
+    /// 顶格的右括号自成一行（函数体在这里收尾）。
+    const TOP_LEVEL_CLOSING_BRACE_LINE: &str = "\n\u{7d}\n";
+
     /// 假的 sysfs 里 PCI 总线那一截的样子。
     enum FakePciBus<'devices> {
         /// `<根>/bus` 在，`<根>/bus/pci` 不在。
@@ -2144,7 +2215,7 @@
     fn the_entries_outside_the_card_pool_start_their_devices_as_outside_the_card_pool() {
         let source = include_str!("gpu_unit_checks.rs");
         let code = source
-            .split(concat!("#[cfg(test)]", "\n", "mod tests {"))
+            .split(TESTS_MODULE_OPENING)
             .next()
             .expect("split 至少交一段");
         let body_of = |signature: &str| {
@@ -2152,7 +2223,9 @@
                 .find(signature)
                 .unwrap_or_else(|| panic!("源码里有 {signature}"));
             let after = &code[start + signature.len()..];
-            &after[..after.find("\n}\n").expect("函数体以顶格的 } 收尾")]
+            &after[..after
+                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
+                .expect("函数体以顶格的右括号收尾")]
         };
         let outside_the_pool = concat!(
             "start_the_card(discovered, DeviceOwnerOnTheCard::",
@@ -2204,6 +2277,111 @@
         );
     }
 
+    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K4，攻方 H6：超时 30 秒改成 0 或不设都照绿）：借据丢掉时那一次等的是到此刻为止的全部提交，
+    /// 最多等 E174 跑前登记的 30 秒（`research/prompts/e174-preregistration.md` 第五节「等待」那一臂）。不要卡。
+    #[test]
+    fn a_lease_waits_for_every_submission_so_far_for_at_most_the_thirty_seconds_registered_by_e174()
+    {
+        match poll_type_before_returning_a_lease() {
+            wgpu::PollType::Wait {
+                submission_index: None,
+                timeout: Some(timeout),
+            } => assert_eq!(
+                timeout,
+                std::time::Duration::from_secs(30),
+                "最多等 E174 登记的 30 秒：不设会让丢借据的线程卡在出了事的设备上，0 等于不等"
+            ),
+            wgpu::PollType::Wait {
+                submission_index: Some(_),
+                ..
+            }
+            | wgpu::PollType::Wait { timeout: None, .. }
+            | wgpu::PollType::Poll => {
+                panic!(
+                    "借据丢掉时等到此刻为止的全部提交、有上限：{:?}",
+                    poll_type_before_returning_a_lease()
+                )
+            }
+        }
+    }
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K1-2 的出路）：卡池借之前对有没等成借据的那张卡 poll 一次，交回队列空了才算那台设备上
+    /// 提交的活都做完了（销那几笔）；还有在飞的、`Wait` 才交的那一种、报错、panic 都不算。不要卡。
+    #[test]
+    fn a_poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty() {
+        assert!(device_found_idle_by_a_poll(&Ok(Ok(
+            wgpu::PollStatus::QueueEmpty
+        ))));
+        assert!(
+            !device_found_idle_by_a_poll(&Ok(Ok(wgpu::PollStatus::Poll))),
+            "还有在飞的提交"
+        );
+        assert!(
+            !device_found_idle_by_a_poll(&Ok(Ok(wgpu::PollStatus::WaitSucceeded))),
+            "Poll 不交这一种，交了也说不清全部提交做完了"
+        );
+        assert!(
+            !device_found_idle_by_a_poll(&Ok(Err(wgpu::PollError::Timeout))),
+            "报错"
+        );
+        assert!(
+            !device_found_idle_by_a_poll(&Err(Box::new("poll 的时候 wgpu panic 了"))),
+            "panic"
+        );
+    }
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K4，攻方 H6、正推第 4 点：`GpuCardLease::drop` 等不等、记没记结局没有不要卡的测试）：
+    /// 借据丢掉时那一步整个交给 harness 档的 `wait_for_the_device_and_take_in_the_outcome`（等、接住 panic、记结局、展开中不再抛都在那边测），
+    /// 等的是 [`poll_type_before_returning_a_lease`]、结局照 [`device_wait_before_returning_a_lease`] 判、记进这一笔借据；卡池的四个入口借之前
+    /// 都先 poll 没等成的那几张卡（K1-2 的出路）。扫这份源码（测试那一段之外），不要卡。
+    #[test]
+    fn the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first() {
+        let source = include_str!("gpu_unit_checks.rs");
+        let code = source
+            .split(TESTS_MODULE_OPENING)
+            .next()
+            .expect("split 至少交一段");
+        let body_until = |signature: &str, closing_line: &str| -> String {
+            let start = code
+                .find(signature)
+                .unwrap_or_else(|| panic!("源码里有 {signature}"));
+            let after = &code[start + signature.len()..];
+            after[..after
+                .find(closing_line)
+                .unwrap_or_else(|| panic!("{signature} 收得了尾"))]
+                .to_string()
+        };
+        let drop_body = body_until("impl Drop for GpuCardLease \u{7b}", "\n        \u{7d}\n");
+        for expected in [
+            "wait_for_the_device_and_take_in_the_outcome(",
+            "&mut self.booking,",
+            "&self.device_on_the_books,",
+            "|| device.poll(poll_type_before_returning_a_lease()),",
+            "device_wait_before_returning_a_lease,",
+        ] {
+            assert_eq!(
+                drop_body.matches(expected).count(),
+                1,
+                "借据丢掉时那一步要有一处 {expected}：{drop_body}"
+            );
+        }
+        for entry in [
+            "fn lease_card(",
+            "pub fn lease_the_first_card_with_room_now(",
+            "pub fn book_the_scratch_of_the_cards_with_room_now(",
+            "pub fn card_rooms_now_and_when_empty_of(",
+        ] {
+            let body = body_until(entry, "\n            \u{7d}\n");
+            let polled = body
+                .find("self.take_in_the_devices_found_idle();")
+                .unwrap_or_else(|| panic!("{entry} 借之前先 poll 没等成的那几张卡：{body}"));
+            let booked = body
+                .find("ledger")
+                .unwrap_or_else(|| panic!("{entry} 照账借：{body}"));
+            assert!(polled < booked, "{entry} 先 poll、再照账借：{body}");
+        }
+    }
+
     /// Cargo.lock 里 wgpu-core 的版本（`name = "wgpu-core"` 下一行的 `version = "…"`）。
     fn wgpu_core_version_in_the_lock(lock: &str) -> Option<String> {
         let mut lines = lock.lines();
@@ -2269,7 +2447,7 @@
             .expect("wgpu-core 的设备上有 handle_hal_error：没有就是这一版改了丢设备的路，要重看");
         let handle_body = &source[handle_start..];
         let handle_body = &handle_body[..handle_body
-            .find("\n    }\n")
+            .find("\n    \u{7d}\n")
             .expect("handle_hal_error 以缩进四格的右括号收尾")];
         assert!(
             handle_body.contains("hal::DeviceError::OutOfMemory"),
--- a/crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs
@@ -1,8 +1,10 @@
 //! checker 档模块：gpu_unit_checks
 //! 卡池的借据丢掉时先等设备做完、再销账（第 10z 批第六块，用户 2026-10-08 定第 10z 批代码轮岔路 G2「改用乙」）：在本机的卡上借一笔、
-//! 建两张 1 GiB 的缓冲、提交一次清零（不等），丢掉缓冲与借据之后读一次 `nvidia-smi -q -d MEMORY,PIDS`，这个进程在那张卡上占着的已经不含
-//! 那笔缓冲。读法照 E174（借据丢掉时等待与显存回还）跑前登记 `research/prompts/e174-preregistration.md` 第一节：建缓冲之前读三份取中位数
-//! U_pre，τ 是那三份的极差加 2 MiB；缓冲活着时读一份，比 U_pre 多出的不少于 S − τ（阳性对照：读数分得出这笔缓冲）；借据丢掉之后读一份，
+//! 建两张 1 GiB 的缓冲，提交一次清零连一段空转约 250 毫秒的内核（活在飞），紧接着丢掉缓冲与借据，之后读一次 `nvidia-smi -q -d MEMORY,PIDS`，
+//! 这个进程在那张卡上占着的已经不含那笔缓冲（第 10z 批第八块，第 10z 批第六块代码轮攻方 F-H4：提交之后先读一次显存再丢，丢的时候活早做完了，
+//! 照 E174「已做完」档改回只 poll 一次也读不到那笔缓冲，分不出等没等；照 E174「在飞·中」改成活在飞时丢，不等的那一臂第一份读数含整笔）。
+//! 读法照 E174（借据丢掉时等待与显存回还）跑前登记 `research/prompts/e174-preregistration.md` 第一节：建缓冲之前读三份取中位数 U_pre，
+//! τ 是那三份的极差加 2 MiB；缓冲建好、提交之前读一份，比 U_pre 多出的不少于 S − τ（阳性对照：读数分得出这笔缓冲）；借据丢掉之后读一份，
 //! 比 U_pre 多出的不超过 τ。要 `gpu` 特性；本机一张 NVIDIA 独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
 //! （`gpu_unit_checks::gpu_test_start`）。
 #![cfg(feature = "gpu")]
@@ -21,6 +23,32 @@
 /// 两份各自取整到 MiB 的读数，相减最多差出不到 2 MiB（E174 第六节 τ 的来由）。
 const ROUNDING_OF_TWO_READINGS_MEBIBYTES: u64 = 2;
 const BYTES_IN_A_MEBIBYTE: u64 = 1024 * 1024;
+/// 空转内核至少跑多久（E174 修订 1「在飞·中」的空转目标 250 毫秒：比读一次显存的约 0.05 秒长得多，丢借据时活还在飞）。
+const SPIN_AT_LEAST: std::time::Duration = std::time::Duration::from_millis(250);
+/// 标定空转内核时头一次试的循环次数（E174 装置同一个数）与最多试几次。
+const FIRST_GUESS_OF_SPIN_ITERATIONS: u32 = 2_000_000;
+const SPIN_CALIBRATION_ATTEMPTS: u32 = 8;
+/// 空转内核绑的那一截（第一张缓冲的头 256 字节）。
+const SPIN_KERNEL_BINDING_BYTES: u64 = 256;
+/// 提交到丢借据之间最多占空转时长的几成，活才算在飞（E174 登记第九节 K9 的 0.8）。
+const IN_FLIGHT_SHARE_PERCENT: u128 = 80;
+
+/// 空转内核：单线程做 `iteration_count` 次相依的整数运算，结果写进第一张缓冲的第一个字，编译器消不掉（照 E174 装置的那一个）。
+const SPIN_SHADER_SOURCE: &str = r#"
+override iteration_count: u32 = 1u;
+
+@group(0) @binding(0) var<storage, read_write> output_words: array<u32>;
+
+@compute @workgroup_size(1)
+fn spin() {
+    var state: u32 = 12345u;
+    for (var step: u32 = 0u; step < iteration_count; step = step + 1u) {
+        state = state * 1664525u + 1013904223u;
+        state = state ^ (state >> 11u);
+    }
+    output_words[0] = state;
+}
+"#;
 
 /// 这个进程此刻在总线号 `bus`（去掉域的写法）那张卡上占着的（MiB；报告里没列这个进程是 0）：跑一次 `nvidia-smi -q -d MEMORY,PIDS`。
 fn this_process_occupancy_on_the_card_mebibytes(bus: &str) -> u64 {
@@ -45,6 +73,104 @@
     }
 }
 
+/// 循环 `iteration_count` 次的空转管线。
+fn spin_pipeline(
+    device: &wgpu::Device,
+    module: &wgpu::ShaderModule,
+    iteration_count: u32,
+) -> wgpu::ComputePipeline {
+    let constants = [("iteration_count", f64::from(iteration_count))];
+    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
+        label: Some(TEST_NAME),
+        layout: None,
+        module,
+        entry_point: Some("spin"),
+        compilation_options: wgpu::PipelineCompilationOptions {
+            constants: &constants,
+            zero_initialize_workgroup_memory: true,
+        },
+        cache: None,
+    })
+}
+
+/// 一次提交：清每张缓冲，再在第一张的头 [`SPIN_KERNEL_BINDING_BYTES`] 上派一次空转内核。
+fn submit_clearing_and_spinning(
+    device: &wgpu::Device,
+    queue: &wgpu::Queue,
+    buffers: &[wgpu::Buffer],
+    pipeline: &wgpu::ComputePipeline,
+) -> wgpu::SubmissionIndex {
+    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
+        label: Some(TEST_NAME),
+    });
+    for buffer in buffers {
+        encoder.clear_buffer(buffer, 0, None);
+    }
+    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
+        label: Some(TEST_NAME),
+        layout: &pipeline.get_bind_group_layout(0),
+        entries: &[wgpu::BindGroupEntry {
+            binding: 0,
+            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
+                buffer: &buffers[0],
+                offset: 0,
+                size: std::num::NonZeroU64::new(SPIN_KERNEL_BINDING_BYTES),
+            }),
+        }],
+    });
+    {
+        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
+            label: Some(TEST_NAME),
+            timestamp_writes: None,
+        });
+        compute_pass.set_pipeline(pipeline);
+        compute_pass.set_bind_group(0, &bind_group, &[]);
+        compute_pass.dispatch_workgroups(1, 1, 1);
+    }
+    queue.submit([encoder.finish()])
+}
+
+/// 标定一条至少空转 [`SPIN_AT_LEAST`] 的管线（在一张 1 MiB 的缓冲上派、等完量挂钟，不够就照比例加循环次数再试）：交管线与量到的时长。
+/// 标定用的缓冲等完就丢了，不留在之后的读数里。
+fn spin_pipeline_running_at_least_the_target(
+    device: &wgpu::Device,
+    queue: &wgpu::Queue,
+) -> (wgpu::ComputePipeline, std::time::Duration) {
+    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
+        label: Some(TEST_NAME),
+        source: wgpu::ShaderSource::Wgsl(SPIN_SHADER_SOURCE.into()),
+    });
+    let calibration_buffers = [device.create_buffer(&wgpu::BufferDescriptor {
+        label: Some(TEST_NAME),
+        size: BYTES_IN_A_MEBIBYTE,
+        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
+        mapped_at_creation: false,
+    })];
+    let mut iteration_count = FIRST_GUESS_OF_SPIN_ITERATIONS;
+    for _attempt in 0..SPIN_CALIBRATION_ATTEMPTS {
+        let pipeline = spin_pipeline(device, &module, iteration_count);
+        let started = std::time::Instant::now();
+        let submission =
+            submit_clearing_and_spinning(device, queue, &calibration_buffers, &pipeline);
+        device
+            .poll(wgpu::PollType::Wait {
+                submission_index: Some(submission),
+                timeout: Some(std::time::Duration::from_secs(30)),
+            })
+            .expect("标定空转内核时等得到");
+        let spun = started.elapsed();
+        if spun >= SPIN_AT_LEAST {
+            return (pipeline, spun);
+        }
+        // 照比例加到目标的一倍半（再短也至少翻一倍），下一次多半就够
+        let scaled = u128::from(iteration_count) * SPIN_AT_LEAST.as_micros() * 3
+            / (2 * spun.as_micros().max(1));
+        iteration_count =
+            u32::try_from(scaled.max(u128::from(iteration_count) * 2)).unwrap_or(u32::MAX);
+    }
+    panic!("试了 {SPIN_CALIBRATION_ATTEMPTS} 次标定不出至少空转 {SPIN_AT_LEAST:?} 的内核（最后一次 {iteration_count} 次循环）")
+}
+
 #[test]
 fn a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading() {
     let Some(pool) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
@@ -57,13 +183,15 @@
     let index = lease.card().index;
     let lease_identifier = lease.lease_identifier();
     let bus = lease.card().pci_bus_identifier.clone();
+    let device = lease.card().device();
+    let queue = lease.card().queue();
+    let (pipeline, spin_duration) = spin_pipeline_running_at_least_the_target(device, queue);
     let mut readings_before: Vec<u64> = (0..3)
         .map(|_| this_process_occupancy_on_the_card_mebibytes(&bus))
         .collect();
     readings_before.sort_unstable();
     let occupied_before = readings_before[1];
     let tolerance = readings_before[2] - readings_before[0] + ROUNDING_OF_TWO_READINGS_MEBIBYTES;
-    let device = lease.card().device();
     let buffers: Vec<wgpu::Buffer> = (0..BUFFERS)
         .map(|_| {
             device.create_buffer(&wgpu::BufferDescriptor {
@@ -74,28 +202,32 @@
             })
         })
         .collect();
-    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
-        label: Some(TEST_NAME),
-    });
-    for buffer in &buffers {
-        encoder.clear_buffer(buffer, 0, None);
-    }
-    lease.card().queue().submit([encoder.finish()]);
     let occupied_while_the_buffers_live = this_process_occupancy_on_the_card_mebibytes(&bus);
     assert!(
         occupied_while_the_buffers_live + tolerance >= occupied_before + size_mebibytes,
-        "阳性对照：缓冲活着时读数里有它（建之前 {occupied_before}、活着时 {occupied_while_the_buffers_live}、S {size_mebibytes}、τ {tolerance}，MiB）"
+        "阳性对照：缓冲建好、提交之前读数里有它（建之前 {occupied_before}、建好 {occupied_while_the_buffers_live}、S {size_mebibytes}、τ {tolerance}，MiB）"
     );
+    let submitted = std::time::Instant::now();
+    let _in_flight = submit_clearing_and_spinning(device, queue, &buffers, &pipeline);
     drop(buffers);
+    let submitted_to_the_lease_dropped = submitted.elapsed();
     drop(lease);
     assert!(
+        submitted_to_the_lease_dropped.as_micros() * 100
+            < spin_duration.as_micros() * IN_FLIGHT_SHARE_PERCENT,
+        "丢借据时活还在飞（提交到丢借据 {submitted_to_the_lease_dropped:?}，空转 {spin_duration:?}）：不在飞的话分不出等没等"
+    );
+    assert!(
         !pool.is_the_lease_on_the_books(index, lease_identifier),
         "借据丢掉、等完设备之后销了账"
     );
     let occupied_after_the_lease_was_returned = this_process_occupancy_on_the_card_mebibytes(&bus);
     println!(
         "GPU_CARD_LEASE_RETURNED index={index} occupied_before={occupied_before} occupied_while_the_buffers_live={occupied_while_the_buffers_live} \
-         occupied_after_the_lease_was_returned={occupied_after_the_lease_was_returned} tolerance={tolerance}"
+         occupied_after_the_lease_was_returned={occupied_after_the_lease_was_returned} tolerance={tolerance} \
+         spin_microseconds={} submitted_to_the_lease_dropped_microseconds={}",
+        spin_duration.as_micros(),
+        submitted_to_the_lease_dropped.as_micros()
     );
     assert!(
         occupied_after_the_lease_was_returned <= occupied_before + tolerance,
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -23,13 +23,17 @@
 //!   减实际占着的里属于许出去的那部分。账上只有活着的借据：借据丢掉时卡池先等这张卡的设备把提交了的活都做完、丢掉的缓冲交回驱动，
 //!   再销账（[`GpuMemoryBooking::take_in_the_wait_before_returning`]；用户 2026-10-08 定第 10z 批代码轮岔路 G2「改用乙」，E174 在两张卡上量过：
 //!   等完之后下一次读数里那笔缓冲都已不在），销了账的那一份不再记、也不从空闲里扣（第 10z 批「甲」那一份「销了账、卡上可能还没还的」整个拿掉）。
-//!   等失败（超时或报错）的那一笔照旧记在账上，直到这张卡的设备丢了、账照重起记（`BooksOfACard::take_back_the_driver_share_of_a_lost_device`），
-//!   只会少借。
+//!   等失败（超时或报错）的那一笔照旧记在账上，只会少借；它等的那台设备上提交的活后来做完了就销（卡池借之前对那张卡 poll 一次交队列空了、
+//!   或同一台设备上另一笔借据等成了，[`DeviceStartedOnTheBooks::take_in_that_the_submitted_work_finished`]），那台设备记成丢了、或放掉了也销
+//!   （第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：两腿都量到等超时的那几笔累加到整张卡借不出，到设备丢了才回来）。
 //!   读不到实际占着的（`nvidia-smi` 没列这个进程），或这个进程在这张卡上另有不走这份账的设备（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]，
 //!   分不出哪一份是借据建的），借着的当一点都没建、驱动那一份也当还没占上卡（第 10y 批代码轮判决 K1-1 攻方乙附：当它整份在卡上，实占的 x 比 D 小时
-//!   多给 D − x），只会少借。这张卡上起好过的设备丢了、账照重起记了之后，到那一刻活着的借据都还了之前也这样算
-//!   （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]、[`DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain`]：
-//!   那几笔拿着旧设备，旧设备连它上面的缓冲可能还占着卡，这个进程占着的分不出哪一份是新设备的）。
+//!   多给 D − x），只会少借。这张卡上起好过的设备丢了、账照重起记了之后，到丢了的那几台都放掉之前也这样算
+//!   （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]、[`DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains`]：
+//!   丢了的设备连它上面的缓冲可能还占着卡，这个进程占着的分不出哪一份是新设备的）。账上的每台设备是一个 [`DeviceStartedOnTheBooks`]，
+//!   卡池把它放进那台设备的每一个克隆：最后一个克隆丢掉（这个进程里那台设备的句柄都没了，wgpu 收掉设备、它上面的缓冲交回驱动）才算放掉
+//!   （第 10z 批第八块，第 10z 批第六块代码轮判决 K1-3：丢了那一刻活着的借据没等成时这一段出不来；按「那一刻活着的借据都还了」判，
+//!   旧设备另有不在账上的句柄时会多给它占着的那一份）。
 //!   不走这份账的设备在起之前就记下它在哪张卡上（[`BusesWithDevicesStartedOutsideTheBooks::start_a_device_marked_first`]，第 10y 批代码轮判决 K1-1
 //!   攻方乙形：起好了才记，起设备到记下之间那一段读数里已经有那个设备的占用，被当成借据建上的）。
 //!
@@ -153,10 +157,10 @@
     /// 丢了的设备连它上面的缓冲可能还占着卡，这个进程占着的分不出哪一份是谁的：照读不到实际占着的算
     /// （[`occupied_part_of_what_this_process_promised_mebibytes`] 交 0）。
     DeviceLostNotStartedAgain,
-    /// 丢了的设备重起好了，而设备丢了、账照重起记的那一刻活着的借据还有没还的：那几笔拿着旧设备，旧设备连它上面的缓冲可能还占着卡，
-    /// 照读不到实际占着的算；那几笔都还了才回到 [`Self::DeviceStarted`]（第 10z 批第六块，G2 乙：拿掉「销了账、卡上可能还没还的」那一份之后，
-    /// 旧设备留在卡上的不再记成一份）。
-    DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain,
+    /// 丢了的设备重起好了，而丢了的那几台还有没放掉的（[`DeviceStartedOnTheBooks`] 还有克隆在：借据的卡、卡池那一格、调用方留着的卡）：
+    /// 丢了的设备连它上面的缓冲可能还占着卡，照读不到实际占着的算；都放掉了才回到 [`Self::DeviceStarted`]（第 10z 批第六块，G2 乙：
+    /// 拿掉「销了账、卡上可能还没还的」那一份之后，旧设备留在卡上的不再记成一份；第 10z 批第八块改按放没放掉判，不按那一刻活着的借据还没还判）。
+    DeviceStartedAgainWhileTheLostDeviceRemains,
 }
 
 /// 这个进程在一张卡上记着的：活着的借据合计（MiB）与驱动那一份走到哪一步。
@@ -185,9 +189,10 @@
 
 /// 这个进程实际占在一张卡上的里，属于它许出去的那一部分（MiB；许出去的是活着的借据，加登记了时驱动那一份）：比许出去的多（分配器按块拿）
 /// 只算到许出去的那么多；读不到实际占着的、这个进程在这张卡上另有不走这份账的设备、或这张卡上丢了的设备可能还占着
-/// （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]、[`DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain`]），
+/// （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]、[`DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains`]），
 /// 是 0：借着的当一点都没建，驱动那一份也当还没占上卡（驱动实占多少、哪一份是旧设备的都分不出）。
-/// 销了账的借据不在许出去的里，它的缓冲在销账之前已经等设备交回驱动（文件头「口径」，G2 乙），不再从实际占着的里扣。
+/// 销了账的借据不在许出去的里，不再从实际占着的里扣：它的缓冲在销账之前已经交回驱动（等完了设备，或那台设备上提交的活后来做完了，
+/// 文件头「口径」，G2 乙），或者它拿着的是丢了还没放掉的设备，那时这里本来就交 0、放掉之前缓冲交回驱动。
 #[must_use]
 pub fn occupied_part_of_what_this_process_promised_mebibytes(
     reading: CardMemoryReading,
@@ -203,7 +208,7 @@
         }
         // 旧设备留在卡上的那一堆读数里与新设备的分不出：当一点都不是许出去的（只会少借）
         DriverShareOnTheCard::DeviceLostNotStartedAgain
-        | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => return 0,
+        | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => return 0,
     };
     match reading.occupied_by_this_process {
         OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => {
@@ -815,18 +820,27 @@
 /// 卡池照 [`card_ledger_capacity_mebibytes`] 算）。
 pub type CardCapacityForThisProcess = Box<dyn Fn(usize, u64) -> u64 + Send + Sync>;
 
-/// 这个进程在一张卡上记着的：每笔活着的借据（借据号 → MiB）、驱动那一份走到哪一步、设备丢了那一刻活着的借据里还没还的，
+/// 账上一台设备在它那张卡上是第几台（这个进程在每张卡上起好的设备各从 0 数，[`GpuMemoryLedger::record_the_device_started_on_the_card`] 发号）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+struct DeviceNumberOnTheCard(u64);
+
+/// 这个进程在一张卡上记着的：每笔活着的借据（借据号 → MiB）、驱动那一份走到哪一步、账上的设备（在用的那一台、丢了还没放掉的那几台），
 /// 与等设备失败而照旧记着的借据。
 #[derive(Clone, Debug, PartialEq, Eq)]
 struct BooksOfACard {
     leases: BTreeMap<u64, u64>,
     driver_share: DriverShareOnTheCard,
-    /// 这张卡上起好过的设备丢了、账照重起记（[`Self::take_back_the_driver_share_of_a_lost_device`]）那一刻记着的借据里还没还的：
-    /// 不空时这个进程占着的照分不出算（[`DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain`]）。
-    leases_alive_when_the_device_was_lost: BTreeSet<u64>,
-    /// 丢掉时等设备没等成（超时或报错）、照旧记在 `leases` 里的借据（[`GpuMemoryBooking::take_in_the_wait_before_returning`]）：
-    /// 设备丢了、账照重起记的那一刻销账。
-    leases_kept_after_a_failed_wait: BTreeSet<u64>,
+    /// 下一台起好的设备记成第几台。
+    next_device_number: DeviceNumberOnTheCard,
+    /// 起好了、还没记成丢了、也还没放掉的那一台。
+    device_in_use: Option<DeviceNumberOnTheCard>,
+    /// 记成丢了（[`Self::take_back_the_driver_share_of_a_lost_device`]）、还没放掉（[`DeviceStartedOnTheBooks`] 还有克隆在）的那几台：
+    /// 它们连上面的缓冲可能还占着卡，不空时这个进程占着的照分不出算（[`DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains`]）。
+    lost_devices_not_released: BTreeSet<DeviceNumberOnTheCard>,
+    /// 丢掉时等设备没等成（超时或报错）、照旧记在 `leases` 里的借据（借据号 → 它等的那一台，[`GpuMemoryBooking::take_in_the_wait_before_returning`]）：
+    /// 那一台上提交的活后来做完了（[`Self::return_the_leases_kept_after_a_failed_wait_on`]）、那一台记成丢了、或放掉了，销账。
+    /// 只记在用的那一台上的：等的是丢了还没放掉的那一台而没等成的，照常销账（那时这个进程占着的照分不出算，放掉之前它的缓冲交回驱动）。
+    leases_kept_after_a_failed_wait: BTreeMap<u64, DeviceNumberOnTheCard>,
 }
 
 impl BooksOfACard {
@@ -835,22 +849,23 @@
         Self {
             leases: BTreeMap::new(),
             driver_share: DriverShareOnTheCard::NotRegistered,
-            leases_alive_when_the_device_was_lost: BTreeSet::new(),
-            leases_kept_after_a_failed_wait: BTreeSet::new(),
+            next_device_number: DeviceNumberOnTheCard(0),
+            device_in_use: None,
+            lost_devices_not_released: BTreeSet::new(),
+            leases_kept_after_a_failed_wait: BTreeMap::new(),
         }
     }
 
-    /// 这张卡上起好过的设备丢了、要重起：驱动那一份记回许出去而还没占上卡；等设备没等成而照旧记着的那几笔这时销账
-    /// （它们的缓冲在旧设备上，旧设备留在卡上的照分不出算，不再按借据记）；此刻记着的借据记成设备丢了那一刻活着的。
+    /// 这张卡上起好过的设备丢了、要重起：驱动那一份记回许出去而还没占上卡；在用的那一台记成丢了（放掉之前这个进程占着的照分不出算）；
+    /// 在它上面等设备没等成而照旧记着的那几笔这时销账（它们的缓冲在丢了的那一台上，放掉之前照分不出算，不再按借据记）。
     fn take_back_the_driver_share_of_a_lost_device(&mut self) {
         self.driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;
-        for kept_lease in std::mem::take(&mut self.leases_kept_after_a_failed_wait) {
+        if let Some(lost_device) = self.device_in_use.take() {
+            self.lost_devices_not_released.insert(lost_device);
+        }
+        for kept_lease in std::mem::take(&mut self.leases_kept_after_a_failed_wait).into_keys() {
             self.leases.remove(&kept_lease);
-            self.leases_alive_when_the_device_was_lost
-                .remove(&kept_lease);
         }
-        self.leases_alive_when_the_device_was_lost
-            .extend(self.leases.keys().copied());
     }
 
     fn booked(&self) -> BookedByThisProcessOnACard {
@@ -860,17 +875,37 @@
         }
     }
 
-    /// 销一笔借据（借据丢掉、等完设备时）：它的缓冲已经交回驱动，账上不再记它。设备丢了那一刻活着的借据都还了，重起好的设备回到
-    /// [`DriverShareOnTheCard::DeviceStarted`]。交回销没销到。
+    /// 销一笔借据（借据丢掉时）：它的缓冲已经交回驱动（等完了设备），或它拿着的是丢了还没放掉的设备（这个进程占着的照分不出算），
+    /// 账上不再记它。交回销没销到。
     fn return_a_lease(&mut self, lease_identifier: u64) -> bool {
-        if self.leases.remove(&lease_identifier).is_none() {
-            return false;
+        self.leases.remove(&lease_identifier).is_some()
+    }
+
+    /// 第 `device` 台上到某一刻为止提交的活都做完了：在它上面等设备没等成而照旧记着的那几笔销账（它们丢掉的缓冲在那之前提交过、
+    /// 丢掉过，wgpu 收完那几次提交就交回驱动）。
+    fn return_the_leases_kept_after_a_failed_wait_on(&mut self, device: DeviceNumberOnTheCard) {
+        let leases = &mut self.leases;
+        self.leases_kept_after_a_failed_wait
+            .retain(|kept_lease, kept_on_the_device| {
+                if *kept_on_the_device == device {
+                    leases.remove(kept_lease);
+                    return false;
+                }
+                true
+            });
+    }
+
+    /// 第 `device` 台放掉了（这个进程里它的句柄都丢了，wgpu 收掉设备、它上面的缓冲交回驱动）：在它上面没等成的那几笔销账；
+    /// 它记成丢了的，不再让这张卡照分不出算，丢了的都放掉了、新设备已经起好，回到 [`DriverShareOnTheCard::DeviceStarted`]。
+    fn release_a_device(&mut self, device: DeviceNumberOnTheCard) {
+        self.return_the_leases_kept_after_a_failed_wait_on(device);
+        self.lost_devices_not_released.remove(&device);
+        if self.device_in_use == Some(device) {
+            self.device_in_use = None;
         }
-        self.leases_alive_when_the_device_was_lost
-            .remove(&lease_identifier);
-        if self.leases_alive_when_the_device_was_lost.is_empty() {
+        if self.lost_devices_not_released.is_empty() {
             match self.driver_share {
-                DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => {
+                DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => {
                     self.driver_share = DriverShareOnTheCard::DeviceStarted;
                 }
                 DriverShareOnTheCard::NotRegistered
@@ -879,7 +914,6 @@
                 | DriverShareOnTheCard::DeviceLostNotStartedAgain => {}
             }
         }
-        true
     }
 
     /// 记一笔借据；这个进程还没在这张卡上登记的，一并登记（设备还没起）。
@@ -896,7 +930,7 @@
             DriverShareOnTheCard::RegisteredDeviceNotStarted
             | DriverShareOnTheCard::DeviceStarted
             | DriverShareOnTheCard::DeviceLostNotStartedAgain
-            | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => {}
+            | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => {}
         }
     }
 }
@@ -1228,7 +1262,7 @@
             DriverShareOnTheCard::RegisteredDeviceNotStarted
             | DriverShareOnTheCard::DeviceStarted
             | DriverShareOnTheCard::DeviceLostNotStartedAgain
-            | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => true,
+            | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => true,
         }
     }
 
@@ -1244,8 +1278,9 @@
     }
 
     /// 在第 `card_position` 张卡上起设备之前调（先登记、后建设备）：这个进程要已经在这张卡上借过一笔。设备是头一回起（或上一回没起来），
-    /// 驱动那一份在登记的那一笔里一起借过，照旧；起过的设备丢了要重起（驱动那一份记着已经占在卡上），把它记回许出去而还没占上卡、
-    /// 这个进程占着的照分不出算（`BooksOfACard::take_back_the_driver_share_of_a_lost_device`：旧设备连它上面的缓冲可能还占着），
+    /// 驱动那一份在登记的那一笔里一起借过，照旧；起过的设备丢了要重起（驱动那一份记着已经占在卡上；重起好了、丢了的还没放掉时又丢一次也是），
+    /// 把它记回许出去而还没占上卡、在用的那一台记成丢了、这个进程占着的照分不出算
+    /// （`BooksOfACard::take_back_the_driver_share_of_a_lost_device`：丢了的设备连它上面的缓冲可能还占着，放掉之前一直这样算），
     /// 在锁里现读一次：借着的连新设备的驱动那一份照此刻的空闲放不下，交借不到，调用方不起（第 10y 批代码轮判决 K1-3 攻方：重起不另登记驱动那一份，
     /// 触发重起的那一笔是按驱动那一份已在卡上借的，新设备的上下文没留地方）。交借不到之后驱动那一份照旧记着许出去而没占上卡
     /// （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]），新设备起好之前每一次再来都照此刻再判一次（第 10z 批代码轮判决 K1-3 正推 S6：
@@ -1267,7 +1302,7 @@
             ),
             DriverShareOnTheCard::RegisteredDeviceNotStarted => Ok(()),
             DriverShareOnTheCard::DeviceStarted
-            | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => {
+            | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => {
                 books[card_position].take_back_the_driver_share_of_a_lost_device();
                 self.driver_share_of_a_new_device_fits_now(&books, card_position)
             }
@@ -1308,23 +1343,51 @@
     }
 
     /// 第 `card_position` 张卡上的设备起好了：从此这个进程的驱动那一份算实际占在卡上（读到实际占着的时候照它算）；起的是丢了的设备的替身、
-    /// 设备丢了那一刻活着的借据还有没还的，到它们都还了之前照分不出算
-    /// （[`DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain`]）。
+    /// 丢了的那几台还有没放掉的，到它们都放掉之前照分不出算（[`DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains`]）。
+    /// 交回账上的这一台：调用方把它放进这台设备的每一个克隆，最后一个克隆丢掉时放掉（[`DeviceStartedOnTheBooks`]）。
     ///
     /// # Panics
-    /// 同 [`Self::check_registered_before_starting_the_device`]。
-    pub fn record_the_device_started_on_the_card(&self, card_position: usize) {
+    /// 同 [`Self::check_registered_before_starting_the_device`]；这张卡上在用的那一台还没记成丢了、也没放掉（起设备只在没起过、
+    /// 上一次没起来、或丢了时）。
+    #[must_use]
+    pub fn record_the_device_started_on_the_card(
+        &self,
+        card_position: usize,
+    ) -> DeviceStartedOnTheBooks<'_> {
         self.check_registered_before_starting_the_device(card_position);
         let mut books = self.books_locked();
         let books_of_the_card = &mut books[card_position];
-        books_of_the_card.driver_share = if books_of_the_card
-            .leases_alive_when_the_device_was_lost
-            .is_empty()
-        {
+        assert_eq!(
+            books_of_the_card.device_in_use, None,
+            "第 {card_position} 张卡：在用的那一台没记成丢了就又起了一台（起设备只在没起过、上一次没起来、或丢了时）"
+        );
+        let device = books_of_the_card.next_device_number;
+        books_of_the_card.next_device_number = DeviceNumberOnTheCard(device.0 + 1);
+        books_of_the_card.device_in_use = Some(device);
+        books_of_the_card.driver_share = if books_of_the_card.lost_devices_not_released.is_empty() {
             DriverShareOnTheCard::DeviceStarted
         } else {
-            DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain
+            DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains
         };
+        DeviceStartedOnTheBooks {
+            ledger: self,
+            card_position,
+            device,
+        }
+    }
+
+    /// 哪几张卡上有等设备没等成而照旧记着的借据（卡池借之前对它们留着的设备 poll 一次，
+    /// [`DeviceStartedOnTheBooks::take_in_that_the_submitted_work_finished`]）。
+    #[must_use]
+    pub fn cards_with_leases_kept_after_a_failed_wait(&self) -> Vec<usize> {
+        self.books_locked()
+            .iter()
+            .enumerate()
+            .filter(|(_card_position, books_of_the_card)| {
+                !books_of_the_card.leases_kept_after_a_failed_wait.is_empty()
+            })
+            .map(|(card_position, _books_of_the_card)| card_position)
+            .collect()
     }
 
     /// 借据号是 `lease_identifier` 的那一笔此刻还记在第 `card_position` 张卡上没有（还了就不在了）。
@@ -1358,16 +1421,16 @@
 pub enum DeviceWaitBeforeReturningTheLease {
     /// 提交了的活都做完了：丢掉的缓冲已经交回驱动，可以销账。
     TheSubmittedWorkFinished,
-    /// 没等成（超时、wgpu 报错或 panic）：缓冲可能还占着卡，这一笔不销账（文件头「口径」）。
+    /// 没等成（超时、wgpu 报错或 panic）：缓冲可能还占着卡，等的是在用的那一台时这一笔先不销账（文件头「口径」）。
     TheWaitFailed,
 }
 
 /// 一笔借据丢掉时账上怎么办。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum LeaseWhenTheBookingIsDropped {
-    /// 销账（等完设备了，或这一笔没在卡上建过缓冲）。
+    /// 销账（等完设备了，这一笔没在卡上建过缓冲，或等的是丢了还没放掉的那一台）。
     ReturnedToTheDriverAndOffTheBooks,
-    /// 等设备没等成：照旧记着，设备丢了、账照重起记的那一刻销（`BooksOfACard::leases_kept_after_a_failed_wait`）。
+    /// 等在用的那一台没等成：照旧记着，那一台上提交的活后来做完了、记成丢了、或放掉了时销（`BooksOfACard::leases_kept_after_a_failed_wait`）。
     KeptOnTheBooksAfterAFailedWait,
 }
 
@@ -1402,16 +1465,44 @@
             .is_the_lease_on_the_books(self.card_position, self.lease_identifier)
     }
 
-    /// 丢掉这一笔之前等过设备（卡池的借据丢掉时调，第 10z 批第六块，用户 2026-10-08 定 G2「改用乙」）：等完了，丢掉时照常销账；
-    /// 没等成，这一笔丢掉时不销账、照旧算借着，直到这张卡的设备丢了、账照重起记（文件头「口径」：账不多给，代价是少借这一笔）。
-    pub fn take_in_the_wait_before_returning(&mut self, wait: DeviceWaitBeforeReturningTheLease) {
+    /// 丢掉这一笔之前等过设备（卡池的借据丢掉时调，第 10z 批第六块，用户 2026-10-08 定 G2「改用乙」；`device` 是这笔借据那张卡上的设备）：
+    /// 等完了，丢掉时照常销账，同一台上之前没等成而照旧记着的那几笔也一起销（这一次等的是到此刻为止的全部提交，它们的活也做完了）；
+    /// 没等成而等的是在用的那一台，这一笔丢掉时不销账、照旧算借着，到那一台上提交的活后来做完了、或那一台记成丢了、放掉了才销
+    /// （文件头「口径」：账不多给，代价是这一段少借这一笔）；没等成而等的是丢了还没放掉的那一台，照常销账（这个进程占着的那时照分不出算，
+    /// 它的缓冲在那一台放掉之前交回驱动，第 10z 批第八块，第 10z 批第六块代码轮判决 K1-3）。
+    ///
+    /// # Panics
+    /// `device` 不是这本账、这张卡上的（别处已经在 panic 时不断言：丢借据是收尾的一部分，两次 panic 会让整个进程中止）。
+    pub fn take_in_the_wait_before_returning(
+        &mut self,
+        device: &DeviceStartedOnTheBooks<'_>,
+        wait: DeviceWaitBeforeReturningTheLease,
+    ) {
+        assert!(
+            (std::ptr::eq(device.ledger, self.ledger) && device.card_position == self.card_position)
+                || std::thread::panicking(),
+            "第 {} 张卡上的借据号 {} 等的设备记在第 {} 张卡上（或别的账上）：借据只等它那张卡上的设备",
+            self.card_position,
+            self.lease_identifier,
+            device.card_position
+        );
+        let mut books = self.ledger.books_locked();
+        let books_of_the_card = &mut books[self.card_position];
         match wait {
-            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished => {}
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished => {
+                books_of_the_card.return_the_leases_kept_after_a_failed_wait_on(device.device);
+            }
             DeviceWaitBeforeReturningTheLease::TheWaitFailed => {
-                self.ledger.books_locked()[self.card_position]
-                    .leases_kept_after_a_failed_wait
-                    .insert(self.lease_identifier);
-                self.when_dropped = LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait;
+                if !books_of_the_card
+                    .lost_devices_not_released
+                    .contains(&device.device)
+                {
+                    books_of_the_card
+                        .leases_kept_after_a_failed_wait
+                        .insert(self.lease_identifier, device.device);
+                    self.when_dropped =
+                        LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait;
+                }
             }
         }
     }
@@ -1436,6 +1527,58 @@
     }
 }
 
+/// 账上一台起好的设备（[`GpuMemoryLedger::record_the_device_started_on_the_card`] 交回）：卡池把它放进这台设备的每一个克隆
+/// （`singlefs_checker_tier::gpu_unit_checks::GpuCard` 的最后一个字段，借据的卡、卡池那一格、调用方留着的卡都是克隆），
+/// 最后一个克隆丢掉时这一台放掉：这个进程里它的句柄都没了，wgpu 收掉设备、它上面的缓冲交回驱动（wgpu-core 30 的 `Queue::drop` 先等设备空了
+/// 再收拾），它记成丢了的不再让这张卡照分不出算，在它上面没等成的借据销账（第 10z 批第八块，第 10z 批第六块代码轮判决 K1-3）。
+/// 管不到的：调用方把 wgpu 的设备、队列、缓冲从克隆里另拿出来、比卡活得久（卡池这边没有这种拿法，借据先于它的缓冲丢的那一条同一个前提）。
+pub struct DeviceStartedOnTheBooks<'ledger> {
+    ledger: &'ledger GpuMemoryLedger,
+    card_position: usize,
+    device: DeviceNumberOnTheCard,
+}
+
+impl DeviceStartedOnTheBooks<'_> {
+    /// 这台设备上到此刻为止提交的活都做完了（卡池借之前对它 poll 一次交回队列空了：wgpu 收完那几次提交、丢掉的缓冲交回了驱动）：
+    /// 在它上面等设备没等成而照旧记着的那几笔销账（第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：两次超时之间没有别的等成时也出得来）。
+    pub fn take_in_that_the_submitted_work_finished(&self) {
+        self.ledger.books_locked()[self.card_position]
+            .return_the_leases_kept_after_a_failed_wait_on(self.device);
+    }
+}
+
+impl Drop for DeviceStartedOnTheBooks<'_> {
+    fn drop(&mut self) {
+        self.ledger.books_locked()[self.card_position].release_a_device(self.device);
+    }
+}
+
+/// 卡池的一笔借据丢掉时那一步（`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::GpuCardLease` 的 `drop` 只调它）：
+/// 照 `wait_for_the_device` 等这笔借据那张卡上的设备 `device`（wgpu 在等的时候 panic 了接住），`wait_of` 照等的结局判等没等成，
+/// 没等成时先交 `report_the_failed_wait` 打一行，再把结局记进这一笔（[`GpuMemoryBooking::take_in_the_wait_before_returning`]）。
+/// 等的时候 panic 了：不在展开中，记完照样往外抛；已经在展开中（这笔借据是别处 panic 收尾时丢掉的），只记没等成、不再抛——展开中再抛会让
+/// 整个进程中止，同一进程里别的用例都没了结局（第 10z 批第八块，第 10z 批第六块代码轮判决 K1-1 攻方 H3，改法 F-H3）。
+pub fn wait_for_the_device_and_take_in_the_outcome<Polled>(
+    booking: &mut GpuMemoryBooking<'_>,
+    device: &DeviceStartedOnTheBooks<'_>,
+    wait_for_the_device: impl FnOnce() -> Polled,
+    wait_of: impl FnOnce(&std::thread::Result<Polled>) -> DeviceWaitBeforeReturningTheLease,
+    report_the_failed_wait: impl FnOnce(&std::thread::Result<Polled>),
+) {
+    let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(wait_for_the_device));
+    let wait = wait_of(&polled);
+    match wait {
+        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished => {}
+        DeviceWaitBeforeReturningTheLease::TheWaitFailed => report_the_failed_wait(&polled),
+    }
+    booking.take_in_the_wait_before_returning(device, wait);
+    if let Err(panic) = polled {
+        if !std::thread::panicking() {
+            std::panic::resume_unwind(panic);
+        }
+    }
+}
+
 /// 一张卡上这个进程的设备（卡池每张卡一格）：要时有留着的、没丢的就拿它；没有（第一次要、上一次起不来）或丢了就起一个，
 /// 起好了留着，起不来交为什么、不留着（第 10x 批代码轮判决 K1-3：起设备失败不缓存成这个进程永久的错；设备丢了之后借这张卡的
 /// 不拿丢了的那个）。不碰 wgpu：设备、怎么起、怎么算丢了由卡池给，单测换成假的。
@@ -1488,6 +1631,15 @@
             .unwrap_or_else(PoisonError::into_inner)
             .is_some()
     }
+
+    /// 留着的设备的克隆（丢了的也交；没起过、上一次没起来是 `None`）；不起设备（卡池借之前 poll 留着的设备用）。
+    #[must_use]
+    pub fn started_now(&self) -> Option<Device> {
+        self.started
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .clone()
+    }
 }
 
 /// 这个进程在哪几张卡（PCI 总线号，[`pci_bus_identifier_without_the_domain`] 写法）上起过不走这份账的设备（崩溃放量没起卡池时自己起的、
@@ -2483,7 +2635,8 @@
         ScratchStatesToBook,
     };
     use super::{
-        next_card_to_start, work_caught_noting_the_device_when_it_returned,
+        next_card_to_start, wait_for_the_device_and_take_in_the_outcome,
+        work_caught_noting_the_device_when_it_returned, DeviceStartedOnTheBooks,
         DeviceWaitBeforeReturningTheLease, StopRequestRememberedInThisRun,
     };
     use std::sync::atomic::{AtomicUsize, Ordering};
@@ -2543,6 +2696,14 @@
         )
     }
 
+    /// 第 `card_position` 张卡上的设备起好了，账上这一台一直不放掉（卡池那一格一直留着它；用例不看它放掉之后的事）。
+    fn record_the_device_started_and_kept_by_the_pool(
+        ledger: &GpuMemoryLedger,
+        card_position: usize,
+    ) {
+        std::mem::forget(ledger.record_the_device_started_on_the_card(card_position));
+    }
+
     /// 借一笔：借到了交借的量（借据不还，留在账上），借不到交结局。
     fn booking_kept_on_the_books(
         ledger: &GpuMemoryLedger,
@@ -2739,7 +2900,7 @@
             Ok(()),
             "头一回起：驱动那一份在登记的那一笔里借过"
         );
-        ledger.record_the_device_started_on_the_card(0);
+        let _device = ledger.record_the_device_started_on_the_card(0);
         assert_eq!(
             ledger.card_rooms_mebibytes_now(&[0]),
             Ok(vec![30_000 - 1_000 - DRIVER]),
@@ -3602,7 +3763,7 @@
             30_000 - DRIVER,
             "第一笔照第一份报告：空 30 000，减还没起设备的驱动那一份"
         );
-        ledger.record_the_device_started_on_the_card(0);
+        let _device = ledger.record_the_device_started_on_the_card(0);
         let second = ledger
             .book_on_the_card(0, 1_000)
             .expect("第二份报告里还空着 24 440");
@@ -3675,7 +3836,7 @@
         let mut first = ledger
             .book_on_the_card(0, FIRST_LEASE)
             .expect("空着的卡借得到");
-        ledger.record_the_device_started_on_the_card(0);
+        let device = ledger.record_the_device_started_on_the_card(0);
         replace_the_reading(
             &readings,
             0,
@@ -3691,6 +3852,7 @@
             .book_on_the_card(0, SECOND_LEASE)
             .expect("卡上空着 23 440，借得到");
         first.take_in_the_wait_before_returning(
+            &device,
             DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
         );
         drop(first);
@@ -3991,8 +4153,9 @@
             DRIVER,
         );
         let mut lease = ledger.book_on_the_card(0, LEASE).expect("卡上空着 22 110");
-        ledger.record_the_device_started_on_the_card(0);
+        let device = ledger.record_the_device_started_on_the_card(0);
         lease.take_in_the_wait_before_returning(
+            &device,
             DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
         );
         drop(lease);
@@ -4055,7 +4218,7 @@
             occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
         };
         let long_lease = ledger.book_on_the_card(0, LONG_LEASE).expect("空卡借得到");
-        ledger.record_the_device_started_on_the_card(0);
+        let device = ledger.record_the_device_started_on_the_card(0);
         let free_while_the_long_lease_is_built = EMPTY - DRIVER - LONG_LEASE;
         replace_the_reading(
             &readings,
@@ -4080,6 +4243,7 @@
                 ),
             );
             short_lease.take_in_the_wait_before_returning(
+                &device,
                 DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
             );
             drop(short_lease);
@@ -4116,7 +4280,7 @@
             ledger.prepare_the_driver_share_before_starting_the_device(0),
             Ok(())
         );
-        ledger.record_the_device_started_on_the_card(0);
+        record_the_device_started_and_kept_by_the_pool(&ledger, 0);
         replace_the_reading(
             &readings,
             0,
@@ -4441,7 +4605,7 @@
             occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
         };
         let long_lease = ledger.book_on_the_card(0, LONG_LEASE).expect("空卡借得到");
-        ledger.record_the_device_started_on_the_card(0);
+        let device = ledger.record_the_device_started_on_the_card(0);
         let mut returned_lease = ledger
             .book_on_the_card(0, RETURNED_LEASE)
             .expect("卡上空着 19 440");
@@ -4458,8 +4622,10 @@
             ledger.card_rooms_mebibytes_now(&[0]).is_ok(),
             "三笔都建满时读一次"
         );
-        returned_lease
-            .take_in_the_wait_before_returning(DeviceWaitBeforeReturningTheLease::TheWaitFailed);
+        returned_lease.take_in_the_wait_before_returning(
+            &device,
+            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
+        );
         drop(returned_lease);
         replace_the_reading(
             &readings,
@@ -4523,7 +4689,7 @@
             Ok(()),
             "旧设备还给驱动之后：借着的 27 440 连新设备的驱动那一份放得下，起"
         );
-        ledger.record_the_device_started_on_the_card(0);
+        let _replacement = ledger.record_the_device_started_on_the_card(0);
         assert!(ledger.is_this_process_registered_on_the_card(0));
     }
 
@@ -4549,12 +4715,12 @@
         );
     }
 
-    /// 第 10z 批第六块（G2 乙：「销了账、卡上可能还没还的」那一份拿掉之后，丢了的设备留在卡上的不再记成一份）：空 30 000 的卡上借 A 10 000、
-    /// 设备起好、建满（驱动实占 600）；设备丢了，B 5 000 借到、照重起记，新设备起好（也实占 600）。A 还拿着旧设备：旧设备的 600 与 A 的 10 000
-    /// 留在卡上，读数分不出哪一份是新设备的，能借的照分不出算（空 18 800 减驱动那一份、减 A、B，1 240），不超过卡上此刻真能再给的
-    /// 11 840（空 18 800 减新设备驱动还要长的 1 960、减 B 还要建的 5 000）；把旧设备的 600 当新设备的驱动那一份算是 12 440，多给 600。
-    /// A 等完设备还了、旧设备没了，B 是设备丢了那一刻就借着的，照旧照分不出算（卡池里 B 那一笔起的重起，B 还了之前旧设备可能还在池里那一格）；
-    /// 重起之后借 C 3 000、B 也还了，回到照读数分得出算：能借的加 C 与驱动那一份是空卡。
+    /// 第 10z 批第六块（G2 乙：「销了账、卡上可能还没还的」那一份拿掉之后，丢了的设备留在卡上的不再记成一份；第 10z 批第八块改按放没放掉判）：
+    /// 空 30 000 的卡上借 A 10 000、设备起好、建满（驱动实占 600）；设备丢了，B 5 000 借到、照重起记，新设备起好（也实占 600），卡池那一格换成新设备。
+    /// A 的卡还拿着旧设备：旧设备的 600 与 A 的 10 000 留在卡上，读数分不出哪一份是新设备的，能借的照分不出算（空 18 800 减驱动那一份、减 A、B，
+    /// 1 240），不超过卡上此刻真能再给的 11 840（空 18 800 减新设备驱动还要长的 1 960、减 B 还要建的 5 000）；把旧设备的 600 当新设备的驱动那一份算是
+    /// 12 440，多给 600。A 等完设备、它的卡丢掉，旧设备的句柄都没了（wgpu 收掉、卡上还给驱动），这一台放掉：照读数分得出算，能借的加 B 与驱动那一份
+    /// 是空卡（B 是设备丢了那一刻就借着的，照「那一刻活着的借据都还了」判时要等 B 也还了，少给 600）。之后借 C 3 000、B 也还了，照旧。
     #[test]
     fn a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement() {
         const EMPTY: u64 = 30_000;
@@ -4576,7 +4742,9 @@
             ledger.prepare_the_driver_share_before_starting_the_device(0),
             Ok(())
         );
-        ledger.record_the_device_started_on_the_card(0);
+        let lost_device_held_by_the_pool =
+            Arc::new(ledger.record_the_device_started_on_the_card(0));
+        let lost_device_held_by_the_lease = Arc::clone(&lost_device_held_by_the_pool);
         replace_the_reading(
             &readings,
             0,
@@ -4590,7 +4758,8 @@
             Ok(()),
             "设备丢了要重起：空 19 400 放得下驱动那一份与两笔"
         );
-        ledger.record_the_device_started_on_the_card(0);
+        let replacement = ledger.record_the_device_started_on_the_card(0);
+        drop(lost_device_held_by_the_pool);
         let occupied_with_the_lost_device_left =
             DRIVER_HOLDS_NOW + LEASE_ON_THE_LOST_DEVICE + DRIVER_HOLDS_NOW;
         replace_the_reading(&readings, 0, occupied(occupied_with_the_lost_device_left));
@@ -4613,40 +4782,633 @@
             "不超过卡上此刻真能再给的（不多给）"
         );
         lease_on_the_lost_device.take_in_the_wait_before_returning(
+            &lost_device_held_by_the_lease,
             DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
         );
-        drop(lease_on_the_lost_device);
         replace_the_reading(&readings, 0, occupied(DRIVER_HOLDS_NOW));
-        let room_while_the_lease_starting_the_replacement_lives =
+        drop(lost_device_held_by_the_lease);
+        drop(lease_on_the_lost_device);
+        let room_after_the_lost_device_is_released =
             ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
         assert_eq!(
-            room_while_the_lease_starting_the_replacement_lives
-                + DRIVER
-                + LEASE_STARTING_THE_REPLACEMENT,
-            EMPTY - DRIVER_HOLDS_NOW,
-            "A 还了，B（设备丢了那一刻已经借着）还在：照旧照分不出算，能借的 \
-             {room_while_the_lease_starting_the_replacement_lives} 加驱动那一份与 B 是空着的 29 400"
+            room_after_the_lost_device_is_released + DRIVER + LEASE_STARTING_THE_REPLACEMENT,
+            EMPTY,
+            "丢了的那一台放掉了：照读数分得出算，能借的 {room_after_the_lost_device_is_released} 加驱动那一份与 B 是空卡"
         );
         let mut lease_after_the_restart = ledger
             .book_on_the_card(0, LEASE_AFTER_THE_RESTART)
             .expect("卡上空着 29 400");
         lease_starting_the_replacement.take_in_the_wait_before_returning(
+            &replacement,
             DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
         );
         drop(lease_starting_the_replacement);
-        let room_after_the_leases_of_the_loss_are_returned =
+        let room_after_the_lease_starting_the_replacement_is_returned =
             ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
         assert_eq!(
-            room_after_the_leases_of_the_loss_are_returned + LEASE_AFTER_THE_RESTART + DRIVER,
+            room_after_the_lease_starting_the_replacement_is_returned
+                + LEASE_AFTER_THE_RESTART
+                + DRIVER,
             EMPTY,
-            "设备丢了那一刻活着的都还了：照读数分得出算，能借的 {room_after_the_leases_of_the_loss_are_returned} \
-             加重起之后借的与驱动那一份是空卡"
+            "B 也还了：能借的 {room_after_the_lease_starting_the_replacement_is_returned} 加重起之后借的与驱动那一份是空卡"
         );
         lease_after_the_restart.take_in_the_wait_before_returning(
+            &replacement,
             DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
         );
     }
 
+    /// [`CardUnderTheLedger`] 此刻真能再借给这个进程的（MiB）：空闲减起着的设备驱动还要长到 [`DRIVER`] 的、减活着的借据还没建的
+    /// （`unbuilt_mebibytes`，由用例给）。
+    fn room_truly_left(card: &Mutex<CardUnderTheLedger>, unbuilt_mebibytes: u64) -> u64 {
+        let card = card.lock().expect("假卡不 panic");
+        let driver_still_grows = if card.device_started {
+            DRIVER - card.driver_occupied_mebibytes
+        } else {
+            DRIVER
+        };
+        card.reading()
+            .free_mebibytes
+            .saturating_sub(driver_still_grows)
+            .saturating_sub(unbuilt_mebibytes)
+    }
+
+    /// 借一笔建满 `mebibytes`、等设备超时（设备上还排着别的活）、丢掉：交回它的借据号；缓冲还在卡上，由调用方后来还给驱动。
+    fn a_lease_built_and_dropped_after_a_timed_out_wait(
+        card: &Mutex<CardUnderTheLedger>,
+        ledger: &GpuMemoryLedger,
+        device: &DeviceStartedOnTheBooks<'_>,
+        mebibytes: u64,
+    ) -> u64 {
+        let mut lease = ledger.book_on_the_card(0, mebibytes).expect("借得到");
+        let lease_identifier = lease.lease_identifier();
+        card.lock().expect("假卡不 panic").built_mebibytes += mebibytes;
+        lease.take_in_the_wait_before_returning(
+            device,
+            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
+        );
+        drop(lease);
+        lease_identifier
+    }
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K1-2：两腿都量到等超时的那几笔永久少借，正推每笔 4 000、第 7 笔起借不到）：设备起好（驱动实占 600），
+    /// 一笔笔借 4 000、建满、等设备超时、丢掉；活后来做完、缓冲还给驱动，卡池借下一笔之前对这张卡 poll 一次交回队列空了：没等成的那一笔销账，
+    /// 能借的回到卡上真能再给的（空卡减驱动那一份）。八笔笔笔借得到，中间没有别的等成。
+    #[test]
+    fn a_lease_whose_wait_timed_out_leaves_the_books_when_a_poll_finds_its_device_idle() {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
+        let device = ledger.record_the_device_started_on_the_card(0);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = 600;
+        }
+        first.take_in_the_wait_before_returning(
+            &device,
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+        drop(first);
+        for timed_out_lease_number in 1..=8 {
+            let timed_out_lease =
+                a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 4_000);
+            assert!(
+                ledger.is_the_lease_on_the_books(0, timed_out_lease),
+                "第 {timed_out_lease_number} 笔没等成：缓冲可能还在卡上，照旧记着"
+            );
+            assert!(
+                room_now(&ledger) <= room_truly_left(&card, 0),
+                "第 {timed_out_lease_number} 笔的缓冲还在卡上时不多给"
+            );
+            assert_eq!(
+                ledger.cards_with_leases_kept_after_a_failed_wait(),
+                vec![0],
+                "第 {timed_out_lease_number} 笔：这张卡上有没等成而照旧记着的，卡池借之前要 poll 它"
+            );
+            card.lock().expect("假卡不 panic").built_mebibytes = 0;
+            device.take_in_that_the_submitted_work_finished();
+            assert!(
+                !ledger.is_the_lease_on_the_books(0, timed_out_lease),
+                "第 {timed_out_lease_number} 笔：poll 交回队列空了，那一笔销账"
+            );
+            assert_eq!(
+                room_now(&ledger),
+                room_truly_left(&card, 0),
+                "第 {timed_out_lease_number} 笔销了账：能借的是卡上真能再给的"
+            );
+        }
+        assert!(ledger
+            .cards_with_leases_kept_after_a_failed_wait()
+            .is_empty());
+    }
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮攻方改法 F-K1）：没等成的那一笔记着，同一台设备上后来另一笔借据等成了（等的是到那一刻为止的全部提交）：
+    /// 没等成的那一笔一起销账。
+    #[test]
+    fn a_lease_whose_wait_timed_out_leaves_the_books_when_another_lease_on_its_device_finishes_its_wait(
+    ) {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let mut finishing_later = ledger.book_on_the_card(0, 2_000).expect("空卡借得到");
+        let device = ledger.record_the_device_started_on_the_card(0);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = 600;
+            card.built_mebibytes = 2_000;
+        }
+        let timed_out_lease =
+            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 6_000);
+        card.lock().expect("假卡不 panic").built_mebibytes = 0;
+        finishing_later.take_in_the_wait_before_returning(
+            &device,
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, timed_out_lease),
+            "同一台上另一笔等成了：没等成的那一笔的活也做完了，销账"
+        );
+        drop(finishing_later);
+        assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
+    }
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K1-3，正推：触发重起的那一笔拿的是新设备）：A 建满在旧设备上，设备丢了；B 借到、照重起记、
+    /// 新设备起好、B 建满，B 等新设备超时、丢掉（缓冲还在卡上）。A 在旧设备上等成了：不销 B（B 等的是新设备）。旧设备放掉之后照读数分，
+    /// B 照旧记着，不多给；新设备后来 poll 交队列空了、B 的缓冲还给驱动，B 才销账。
+    #[test]
+    fn a_finished_wait_on_a_lost_device_leaves_a_lease_kept_on_its_replacement_on_the_books() {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let (mut lease_on_the_lost_device, lost_device) =
+            ten_thousand_built_with_the_device_held(&card, &ledger);
+        let mut lease_starting_the_replacement =
+            ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
+        let starting_the_replacement = lease_starting_the_replacement.lease_identifier();
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = false;
+            card.built_mebibytes = 0;
+            card.left_by_a_lost_device_mebibytes = DRIVER + 10_000;
+        }
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(())
+        );
+        let replacement = ledger.record_the_device_started_on_the_card(0);
+        {
+            // 新设备的驱动还没长满（实占 600）：B 的缓冲要是被当成驱动还要长的那一份，就看得出多给
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = 600;
+            card.built_mebibytes = 5_000;
+        }
+        lease_starting_the_replacement.take_in_the_wait_before_returning(
+            &replacement,
+            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
+        );
+        drop(lease_starting_the_replacement);
+        lease_on_the_lost_device.take_in_the_wait_before_returning(
+            &lost_device,
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+        assert!(
+            ledger.is_the_lease_on_the_books(0, starting_the_replacement),
+            "旧设备上等成了，说明不了新设备上 B 的活做完了：B 照旧记着"
+        );
+        card.lock()
+            .expect("假卡不 panic")
+            .left_by_a_lost_device_mebibytes = 0;
+        drop(lost_device);
+        drop(lease_on_the_lost_device);
+        assert!(
+            room_now(&ledger) <= room_truly_left(&card, 0),
+            "旧设备放掉、照读数分：B 的缓冲还在卡上，不多给（{} 对 {}）",
+            room_now(&ledger),
+            room_truly_left(&card, 0)
+        );
+        card.lock().expect("假卡不 panic").built_mebibytes = 0;
+        replacement.take_in_that_the_submitted_work_finished();
+        assert!(!ledger.is_the_lease_on_the_books(0, starting_the_replacement));
+        assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
+    }
+
+    /// 借一笔 10 000、起设备、建满（驱动实占 [`DRIVER`]），交回借据与账上的这一台（`Arc`：借据的卡拿着一份；卡池那一格的那一份
+    /// 由调用方丢的时候另克隆，这里不留）。
+    fn ten_thousand_built_with_the_device_held<'ledger>(
+        card: &Mutex<CardUnderTheLedger>,
+        ledger: &'ledger GpuMemoryLedger,
+    ) -> (
+        GpuMemoryBooking<'ledger>,
+        Arc<DeviceStartedOnTheBooks<'ledger>>,
+    ) {
+        let lease = ledger
+            .book_on_the_card(0, 10_000)
+            .expect("空卡借得到 10 000");
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(())
+        );
+        let device = Arc::new(ledger.record_the_device_started_on_the_card(0));
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = DRIVER;
+            card.built_mebibytes = 10_000;
+        }
+        (lease, device)
+    }
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K1-3、攻方 H2 与改法 F-K3 自报的风险）：旧设备上借着 A 10 000（建满，驱动实占 600），
+    /// 设备丢了；L 5 000 借到、照重起记、新设备起好、L 建满，卡池那一格换成新设备。A 等旧设备没等成、丢掉：照常销账（这时照分不出算）。
+    /// 调用方另留着旧设备的一个克隆（产品路径的 `Arc<GpuCard>` 一类）：旧设备连 A 的缓冲还占着卡，L 也还了，这张卡照旧照分不出算，不多给
+    /// （照「那一刻活着的借据都还了」判，这时回到照读数分，旧设备的那一份被当成新借据建上的）。那个克隆也丢了、旧设备放掉，照读数分：
+    /// 长借据 Q 15 000 建满之后再借 10 000，借得到，能借的就是卡上真能再给的（攻方 `k1_3` 那六段的形状）。
+    #[test]
+    fn a_lost_device_still_held_outside_the_leases_keeps_the_card_unsplittable_until_it_is_released(
+    ) {
+        const DRIVER_HOLDS_NOW: u64 = 600;
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let mut lease_a = ledger.book_on_the_card(0, 10_000).expect("空卡借得到");
+        let lease_a_identifier = lease_a.lease_identifier();
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(())
+        );
+        let held_by_the_pool = Arc::new(ledger.record_the_device_started_on_the_card(0));
+        let held_by_lease_a = Arc::clone(&held_by_the_pool);
+        let held_by_the_caller = Arc::clone(&held_by_the_pool);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
+            card.built_mebibytes = 10_000;
+        }
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = false;
+            card.built_mebibytes = 0;
+            card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW + 10_000;
+        }
+        let mut lease_l = ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(()),
+            "重起放得下"
+        );
+        let replacement = ledger.record_the_device_started_on_the_card(0);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
+            card.built_mebibytes = 5_000;
+        }
+        drop(held_by_the_pool);
+        lease_a.take_in_the_wait_before_returning(
+            &held_by_lease_a,
+            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
+        );
+        drop(held_by_lease_a);
+        drop(lease_a);
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, lease_a_identifier),
+            "A 等的是丢了还没放掉的那一台：没等成也照常销账，不卡在账上"
+        );
+        assert!(
+            room_now(&ledger) <= room_truly_left(&card, 0),
+            "旧设备还被调用方拿着、连 A 的缓冲占着卡：不多给（{} 对 {}）",
+            room_now(&ledger),
+            room_truly_left(&card, 0)
+        );
+        card.lock().expect("假卡不 panic").built_mebibytes = 0;
+        lease_l.take_in_the_wait_before_returning(
+            &replacement,
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+        drop(lease_l);
+        assert!(
+            room_now(&ledger) <= room_truly_left(&card, 0),
+            "L 也还了、旧设备还在：照旧照分不出算，不多给（{} 对 {}）",
+            room_now(&ledger),
+            room_truly_left(&card, 0)
+        );
+        card.lock()
+            .expect("假卡不 panic")
+            .left_by_a_lost_device_mebibytes = 0;
+        drop(held_by_the_caller);
+        assert_eq!(
+            room_now(&ledger),
+            room_truly_left(&card, 0),
+            "旧设备放掉了：照读数分，能借的是卡上真能再给的"
+        );
+        let _long_lease = ledger.book_on_the_card(0, 15_000).expect("借得到 15 000");
+        card.lock().expect("假卡不 panic").built_mebibytes = 15_000;
+        assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
+        assert!(
+            ledger.book_on_the_card(0, 10_000).is_ok(),
+            "Q 建满之后卡上放得下 10 000：借得到"
+        );
+    }
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮正推探针 P5、攻方 H5 的 X7：重起好了、旧设备还在时又丢一次，不重新记账）：第一台丢了、调用方还拿着它，
+    /// 第二台起好、L 在它上面建满之后也丢了，M 借到、照重起记、第三台起好。第一台放掉了，第二台还被 L 的卡与调用方拿着、连 L 的缓冲占着卡：
+    /// 照旧照分不出算，不多给；L 还了、第二台也放掉，照读数分。
+    #[test]
+    fn a_second_loss_while_the_first_lost_device_remains_keeps_the_card_unsplittable_until_both_are_released(
+    ) {
+        const DRIVER_HOLDS_NOW: u64 = 600;
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let mut lease_on_the_first = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(())
+        );
+        let first_device = ledger.record_the_device_started_on_the_card(0);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
+        }
+        lease_on_the_first.take_in_the_wait_before_returning(
+            &first_device,
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+        drop(lease_on_the_first);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = false;
+            card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW;
+        }
+        let mut lease_l = ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(()),
+            "第一次重起"
+        );
+        let second_device = Arc::new(ledger.record_the_device_started_on_the_card(0));
+        let second_held_by_lease_l = Arc::clone(&second_device);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
+            card.built_mebibytes = 5_000;
+        }
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = false;
+            card.built_mebibytes = 0;
+            card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW + DRIVER_HOLDS_NOW + 5_000;
+        }
+        let _lease_m = ledger.book_on_the_card(0, 1_000).expect("借得到 1 000");
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(()),
+            "第二次重起：第一台还没放掉时又丢一次"
+        );
+        let _third_device = ledger.record_the_device_started_on_the_card(0);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
+            card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW + 5_000;
+        }
+        drop(first_device);
+        let unbuilt_lease_m = 1_000;
+        assert!(
+            room_now(&ledger) <= room_truly_left(&card, unbuilt_lease_m),
+            "第一台放掉了、第二台连 L 的缓冲还在卡上：照分不出算，不多给（{} 对 {}）",
+            room_now(&ledger),
+            room_truly_left(&card, unbuilt_lease_m)
+        );
+        card.lock()
+            .expect("假卡不 panic")
+            .left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW;
+        lease_l.take_in_the_wait_before_returning(
+            &second_held_by_lease_l,
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+        drop(second_held_by_lease_l);
+        drop(lease_l);
+        assert!(
+            room_now(&ledger) <= room_truly_left(&card, unbuilt_lease_m),
+            "L 还了、第二台还被调用方拿着：不多给"
+        );
+        card.lock()
+            .expect("假卡不 panic")
+            .left_by_a_lost_device_mebibytes = 0;
+        drop(second_device);
+        assert_eq!(
+            room_now(&ledger),
+            room_truly_left(&card, unbuilt_lease_m),
+            "丢了的两台都放掉了：照读数分，能借的是卡上真能再给的"
+        );
+    }
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮正推探针 P3、攻方 H5 的 X1 那一形：重起时没等成的借据要一起出账，不然这张卡到下一次丢设备都少给）：
+    /// C 在设备上没等成、照旧记着（缓冲还在卡上）；设备丢了、重起：C 这时销账；旧设备连 C 的缓冲放掉之后，能借的就是卡上真能再给的。
+    #[test]
+    fn a_lease_kept_after_a_failed_wait_leaves_the_books_at_the_restart_and_the_card_is_not_left_short(
+    ) {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let (lease, lost_device) = ten_thousand_built_with_the_device_held(&card, &ledger);
+        let kept_lease =
+            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &lost_device, 4_000);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = false;
+            card.built_mebibytes = 0;
+            card.left_by_a_lost_device_mebibytes = DRIVER + 10_000 + 4_000;
+        }
+        let mut lease_starting_the_replacement =
+            ledger.book_on_the_card(0, 1_000).expect("借得到 1 000");
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(())
+        );
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, kept_lease),
+            "设备丢了、账照重起记：没等成的那一笔这时销账"
+        );
+        let replacement = ledger.record_the_device_started_on_the_card(0);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = DRIVER;
+            card.left_by_a_lost_device_mebibytes = 0;
+        }
+        drop(lost_device);
+        drop(lease);
+        assert_eq!(
+            room_now(&ledger),
+            room_truly_left(&card, 1_000),
+            "旧设备放掉了：能借的是卡上真能再给的，没有一笔卡在账上"
+        );
+        lease_starting_the_replacement.take_in_the_wait_before_returning(
+            &replacement,
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+    }
+
+    /// 借据丢掉时那几条用例的等法：`true` 是等完了、`false` 是没等成；等的时候 panic 了也是没等成。
+    fn wait_of_a_fake_device(
+        polled: &std::thread::Result<bool>,
+    ) -> DeviceWaitBeforeReturningTheLease {
+        if matches!(polled, Ok(true)) {
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished
+        } else {
+            DeviceWaitBeforeReturningTheLease::TheWaitFailed
+        }
+    }
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K4，攻方 H6、正推第 4 点：`GpuCardLease::drop` 记没记结局没有不要卡的测试）：
+    /// 借据丢掉时那一步没等成，打一行、这一笔照旧记着；等完了不打、照常销账，之前没等成的那一笔也一起销。
+    #[test]
+    fn the_wait_before_returning_a_lease_reports_a_failed_wait_and_takes_in_either_outcome() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+        let ledger = ledger_reading(&readings);
+        let mut timed_out = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
+        let timed_out_identifier = timed_out.lease_identifier();
+        let device = ledger.record_the_device_started_on_the_card(0);
+        let mut finished = ledger.book_on_the_card(0, 1_000).expect("借得到");
+        let finished_identifier = finished.lease_identifier();
+        let reports = std::cell::Cell::new(0);
+        wait_for_the_device_and_take_in_the_outcome(
+            &mut timed_out,
+            &device,
+            || false,
+            wait_of_a_fake_device,
+            |_polled| reports.set(reports.get() + 1),
+        );
+        drop(timed_out);
+        assert_eq!(reports.get(), 1, "没等成：打一行");
+        assert!(
+            ledger.is_the_lease_on_the_books(0, timed_out_identifier),
+            "没等成：结局记进了这一笔，丢掉之后照旧记着"
+        );
+        wait_for_the_device_and_take_in_the_outcome(
+            &mut finished,
+            &device,
+            || true,
+            wait_of_a_fake_device,
+            |_polled| reports.set(reports.get() + 1),
+        );
+        drop(finished);
+        assert_eq!(reports.get(), 1, "等完了：不打");
+        assert!(!ledger.is_the_lease_on_the_books(0, finished_identifier));
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, timed_out_identifier),
+            "同一台上等完了：之前没等成的那一笔一起销"
+        );
+    }
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K4，攻方 H6：`catch_unwind` 拿掉照绿）：不在展开中，等的时候 wgpu panic 了，
+    /// 这一笔记成没等成（丢掉之后照旧记着），那个 panic 照样往外抛。
+    #[test]
+    fn a_panic_while_waiting_before_returning_a_lease_is_taken_as_a_failed_wait_and_goes_on() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+        let ledger = ledger_reading(&readings);
+        let mut lease = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
+        let lease_identifier = lease.lease_identifier();
+        let device = ledger.record_the_device_started_on_the_card(0);
+        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+            wait_for_the_device_and_take_in_the_outcome(
+                &mut lease,
+                &device,
+                || -> bool { panic!("等的时候 wgpu panic 了") },
+                wait_of_a_fake_device,
+                |_polled| {},
+            );
+        }));
+        let panic = unwound.expect_err("不在展开中：等的时候的 panic 照样往外抛");
+        assert_eq!(
+            panic.downcast_ref::<&str>(),
+            Some(&"等的时候 wgpu panic 了"),
+            "抛出去的是等的时候那一个"
+        );
+        drop(lease);
+        assert!(
+            ledger.is_the_lease_on_the_books(0, lease_identifier),
+            "等的时候 panic 了：记成没等成，丢掉之后照旧记着"
+        );
+    }
+
+    /// 子进程跑 [`a_lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again`] 那一形时设的变量。
+    const LEASE_DROPPED_WHILE_UNWINDING_CHILD_VARIABLE: &str =
+        "SINGLEFS_HARNESS_LEASE_DROPPED_WHILE_UNWINDING_CHILD";
+
+    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K1-1 攻方 H3，改法 F-H3，照攻方 `unwind_in_drop.rs` 写，不要卡）：用例本身 panic 了，
+    /// 借据在展开中丢掉，等的时候 wgpu 又 panic：只记没等成、不再抛（展开中再抛整个进程中止）。在子进程里跑那一形（中止的是子进程，
+    /// 这条用例照常判红），子进程要正常退出、这一笔记成没等成。
+    #[test]
+    fn a_lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again()
+    {
+        const TEST_FUNCTION: &str =
+            "a_lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again";
+        if std::env::var_os(LEASE_DROPPED_WHILE_UNWINDING_CHILD_VARIABLE).is_some() {
+            struct LeaseReturnedWhenDropped<'ledger, 'device> {
+                lease: Option<GpuMemoryBooking<'ledger>>,
+                device: &'device DeviceStartedOnTheBooks<'ledger>,
+            }
+            impl Drop for LeaseReturnedWhenDropped<'_, '_> {
+                fn drop(&mut self) {
+                    if let Some(mut lease) = self.lease.take() {
+                        wait_for_the_device_and_take_in_the_outcome(
+                            &mut lease,
+                            self.device,
+                            || -> bool { panic!("等的时候 wgpu panic 了：设备丢了") },
+                            wait_of_a_fake_device,
+                            |_polled| {},
+                        );
+                    }
+                }
+            }
+            let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+            let ledger = ledger_reading(&readings);
+            let lease = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
+            let lease_identifier = lease.lease_identifier();
+            let device = ledger.record_the_device_started_on_the_card(0);
+            let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+                let _returned_when_dropped = LeaseReturnedWhenDropped {
+                    lease: Some(lease),
+                    device: &device,
+                };
+                panic!("用例本身 panic 了");
+            }));
+            assert!(unwound.is_err(), "用例本身那一个 panic 照样展开到外面");
+            println!(
+                "LEASE_DROPPED_WHILE_UNWINDING kept_on_the_books={}",
+                ledger.is_the_lease_on_the_books(0, lease_identifier)
+            );
+            return;
+        }
+        let test_name = format!(
+            "{}::{TEST_FUNCTION}",
+            module_path!()
+                .split_once("::")
+                .expect("模块路径以包名起头")
+                .1
+        );
+        let child = std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))
+            .args([
+                test_name.as_str(),
+                "--exact",
+                "--nocapture",
+                "--test-threads",
+                "1",
+            ])
+            .env(LEASE_DROPPED_WHILE_UNWINDING_CHILD_VARIABLE, "1")
+            .output()
+            .expect("起得了子进程");
+        let standard_output = String::from_utf8_lossy(&child.stdout);
+        assert!(
+            child.status.success(),
+            "展开中丢借据、等的时候又 panic：子进程 {}（中止就是展开中又抛了）：{}",
+            child.status,
+            String::from_utf8_lossy(&child.stderr)
+        );
+        assert!(
+            standard_output.contains("LEASE_DROPPED_WHILE_UNWINDING kept_on_the_books=true"),
+            "这一笔记成没等成、丢掉之后照旧记着：{standard_output}"
+        );
+    }
+
     /// 第 10z 批第三块（第 10z 批代码轮正推 N1）：同一条线程上活里又起一段活，里面那段丢掉之后，这条线程上记的 Out of Memory 算回外面那段。
     #[test]
     fn a_work_begun_inside_another_hands_the_thread_back_to_the_outer_work_when_it_ends() {
@@ -5357,7 +6119,7 @@
             card.driver_occupied_mebibytes = DRIVER;
             card.built_mebibytes = 10_000;
         }
-        ledger.record_the_device_started_on_the_card(0);
+        record_the_device_started_and_kept_by_the_pool(ledger, 0);
         lease
     }
 
@@ -5385,8 +6147,9 @@
             card.driver_occupied_mebibytes = DRIVER;
             card.built_mebibytes = 10_000;
         }
-        ledger.record_the_device_started_on_the_card(0);
+        let device = ledger.record_the_device_started_on_the_card(0);
         first.take_in_the_wait_before_returning(
+            &device,
             DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
         );
         card.lock().expect("假卡不 panic").built_mebibytes = 0;
````

### 第八块 mutations-append.tsv

````
第10z批第八块 显存账 K1-2 poll 交回队列空了也不销没等成的那几笔	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .return_the_leases_kept_after_a_failed_wait_on(self.device);	            .leases_kept_after_a_failed_wait\n            .clear();	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_whose_wait_timed_out_leaves_the_books_when_a_poll_finds_its_device_idle	a_lease_whose_wait_timed_out_leaves_the_books_when_a_poll_finds_its_device_idle
第10z批第八块 显存账 K1-2 借之前列不出有没等成借据的卡	crates/singlefs-harness/src/gpu_memory_ledger.rs	                !books_of_the_card.leases_kept_after_a_failed_wait.is_empty()	                books_of_the_card.leases_kept_after_a_failed_wait.is_empty()	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_whose_wait_timed_out_leaves_the_books_when_a_poll_finds_its_device_idle	a_lease_whose_wait_timed_out_leaves_the_books_when_a_poll_finds_its_device_idle
第10z批第八块 显存账 K1-2 同一台上另一笔等成了也不销没等成的那几笔（攻方 F-K1）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                books_of_the_card.return_the_leases_kept_after_a_failed_wait_on(device.device);	                let _ = device.device;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_whose_wait_timed_out_leaves_the_books_when_another_lease_on_its_device_finishes_its_wait	a_lease_whose_wait_timed_out_leaves_the_books_when_another_lease_on_its_device_finishes_its_wait
第10z批第八块 显存账 K1-3 等成了连别的设备上没等成的也销（触发重起那一笔拿的是新设备）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                if *kept_on_the_device == device {	                if *kept_on_the_device == device || true {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_finished_wait_on_a_lost_device_leaves_a_lease_kept_on_its_replacement_on_the_books	a_finished_wait_on_a_lost_device_leaves_a_lease_kept_on_its_replacement_on_the_books
第10z批第八块 显存账 K1-3 等的是丢了还没放掉的那一台而没等成也照旧记着（攻方 H2 那一形卡在账上）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                if !books_of_the_card\n                    .lost_devices_not_released\n                    .contains(&device.device)\n                {	                if true {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_still_held_outside_the_leases_keeps_the_card_unsplittable_until_it_is_released	a_lost_device_still_held_outside_the_leases_keeps_the_card_unsplittable_until_it_is_released
第10z批第八块 显存账 K1-3 丢了的设备另有句柄时重起好了就照读数分（攻方 F-K3 自报的多给那一形）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        books_of_the_card.driver_share = if books_of_the_card.lost_devices_not_released.is_empty() {	        books_of_the_card.driver_share = if books_of_the_card.lost_devices_not_released.is_empty() || true {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_still_held_outside_the_leases_keeps_the_card_unsplittable_until_it_is_released	a_lost_device_still_held_outside_the_leases_keeps_the_card_unsplittable_until_it_is_released
第10z批第八块 显存账 K1-3 放掉一台就当丢了的都放掉了（第二次丢的那一台还在）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.lost_devices_not_released.remove(&device);	        self.lost_devices_not_released.clear();	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_second_loss_while_the_first_lost_device_remains_keeps_the_card_unsplittable_until_both_are_released	a_second_loss_while_the_first_lost_device_remains_keeps_the_card_unsplittable_until_both_are_released
第10z批第八块 显存账 K1-3 重起好了、丢了的还没放掉时又丢一次不重新记账（正推 P5、攻方 X7）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::DeviceStarted\n            | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => {\n                books[card_position].take_back_the_driver_share_of_a_lost_device();	            DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => {\n                books[card_position].driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;\n                self.driver_share_of_a_new_device_fits_now(&books, card_position)\n            }\n            DriverShareOnTheCard::DeviceStarted => {\n                books[card_position].take_back_the_driver_share_of_a_lost_device();	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_second_loss_while_the_first_lost_device_remains_keeps_the_card_unsplittable_until_both_are_released	a_second_loss_while_the_first_lost_device_remains_keeps_the_card_unsplittable_until_both_are_released
第10z批第八块 显存账 K1-3 重起时没等成的那几笔不出账（正推 P3、攻方 X1 那一形）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            self.leases.remove(&kept_lease);	            let _ = &kept_lease;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_kept_after_a_failed_wait_leaves_the_books_at_the_restart_and_the_card_is_not_left_short	a_lease_kept_after_a_failed_wait_leaves_the_books_at_the_restart_and_the_card_is_not_left_short
第10z批第八块 显存账 K4 借据丢掉时那一步不把结局记进借据（GpuCardLease::drop 记没记结局）	crates/singlefs-harness/src/gpu_memory_ledger.rs	    booking.take_in_the_wait_before_returning(device, wait);	    let _ = (&booking, device, wait);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_wait_before_returning_a_lease_reports_a_failed_wait_and_takes_in_either_outcome	the_wait_before_returning_a_lease_reports_a_failed_wait_and_takes_in_either_outcome
第10z批第八块 显存账 K4 借据丢掉时那一步没等成不打那一行	crates/singlefs-harness/src/gpu_memory_ledger.rs	        DeviceWaitBeforeReturningTheLease::TheWaitFailed => report_the_failed_wait(&polled),	        DeviceWaitBeforeReturningTheLease::TheWaitFailed => {}	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_wait_before_returning_a_lease_reports_a_failed_wait_and_takes_in_either_outcome	the_wait_before_returning_a_lease_reports_a_failed_wait_and_takes_in_either_outcome
第10z批第八块 显存账 K4 借据丢掉时那一步等的时候 panic 不接住（攻方 H6 的 catch_unwind）	crates/singlefs-harness/src/gpu_memory_ledger.rs	    let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(wait_for_the_device));	    let polled: std::thread::Result<Polled> = Ok(wait_for_the_device());	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_panic_while_waiting_before_returning_a_lease_is_taken_as_a_failed_wait_and_goes_on	a_panic_while_waiting_before_returning_a_lease_is_taken_as_a_failed_wait_and_goes_on
第10z批第八块 显存账 H3 借据丢掉时那一步展开中照样再抛（攻方 F-H3）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if !std::thread::panicking() {\n            std::panic::resume_unwind(panic);	        if true {\n            std::panic::resume_unwind(panic);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again	a_lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again
第10z批第八块 GPU 取卡 K4 借据丢掉时等的超时改成 0（攻方 H6）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	    std::time::Duration::from_secs(30);	    std::time::Duration::from_secs(0);	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::a_lease_waits_for_every_submission_so_far_for_at_most_the_thirty_seconds_registered_by_e174	a_lease_waits_for_every_submission_so_far_for_at_most_the_thirty_seconds_registered_by_e174
第10z批第八块 GPU 取卡 K4 借据丢掉时等的不设超时（攻方 H6）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        timeout: Some(WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE),	        timeout: None,	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::a_lease_waits_for_every_submission_so_far_for_at_most_the_thirty_seconds_registered_by_e174	a_lease_waits_for_every_submission_so_far_for_at_most_the_thirty_seconds_registered_by_e174
第10z批第八块 GPU 取卡 K1-2 借之前 poll 交回还有在飞的也当空闲	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        Ok(Ok(wgpu::PollStatus::QueueEmpty)) => true,	        Ok(Ok(wgpu::PollStatus::QueueEmpty | wgpu::PollStatus::Poll)) => true,	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::a_poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty	a_poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty
第10z批第八块 GPU 取卡 K1-2 卡池借之前不 poll 没等成的那几张卡	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                self.take_in_the_devices_found_idle();\n                let booking = match self.ledger.book_on_the_first_card_with_room(need_mebibytes) {	                let booking = match self.ledger.book_on_the_first_card_with_room(need_mebibytes) {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first	the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first
第10z批第八块 GPU 取卡 K4 借据丢掉时那一步改回只 poll 一次（扫源码那一条）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                    || device.poll(poll_type_before_returning_a_lease()),	                    || device.poll(wgpu::PollType::Poll),	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first	the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first
第10z批第八块 崩溃放量 K4 一轮收尾不问记住的叫停（正推改回去 A）	crates/singlefs-checker-tier/src/crash_amplification.rs	        let stop = stop_request.stop_of_the_run();	        let stop = singlefs_harness::gpu_memory_ledger::StopOfTheRun::NotRequested;	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run	the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run
第10z批第八块 崩溃放量 K4 起下一张卡不看叫停（正推改回去 B）	crates/singlefs-checker-tier/src/crash_amplification.rs	            work.stop_of_the_cards()\n        })	            StopOfTheCards::NotRequested\n        })	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run	the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run
第10z批第八块 崩溃放量 K4 卡线程判块那一圈不看叫停（正推改回去 C）	crates/singlefs-checker-tier/src/crash_amplification.rs	        &|| work.stop_of_the_cards(),	        &|| StopOfTheCards::NotRequested,	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run	the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run
第10z批第八块 崩溃放量 K4 整趟收尾的叫停闭包改成永远不叫停（正推改回去 D）	crates/singlefs-checker-tier/src/crash_amplification.rs	    let stop_is_requested = || stop_request.is_requested();	    let stop_is_requested = || false;	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run	the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run
第10z批第八块 崩溃放量 K4 照叫停收尾那一支把头一句 Out of Memory 那一行打进标准错误（正推改回去 E）	crates/singlefs-checker-tier/src/crash_amplification.rs	                    println!("{not_run_line}");\n                }\n                break;	                    eprintln!("{not_run_line}");\n                }\n                break;	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_gpu_rounds_stopped_on_request_print_the_first_out_of_memory_in_the_not_run_line	the_gpu_rounds_stopped_on_request_print_the_first_out_of_memory_in_the_not_run_line
````

### 第八块 mutations-replacements.tsv

````
第10z批第六块 显存账 G2 乙 等设备没等成照样销账（等失败那一支多给）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    self.when_dropped =\n                        LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait;	                    self.when_dropped =\n                        LeaseWhenTheBookingIsDropped::ReturnedToTheDriverAndOffTheBooks;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_whose_wait_failed_stays_booked_until_the_device_is_taken_as_lost_and_gives_no_more	a_lease_whose_wait_failed_stays_booked_until_the_device_is_taken_as_lost_and_gives_no_more
第10z批第六块 显存账 G2 乙 丢了的设备还可能占着卡时照读数分（旧设备那一份当成新设备的驱动那一份）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => return 0,	        | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => booked\n            .leases_mebibytes\n            .saturating_add(mebibytes_left_for_the_driver),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement	a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
第10z批第六块 显存账 G2 乙 重起好了就当旧设备已经不在（设备丢了那一刻活着的借据还在）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        books_of_the_card.driver_share = if books_of_the_card.lost_devices_not_released.is_empty() {	        books_of_the_card.driver_share = if books_of_the_card.lost_devices_not_released.is_empty() || true {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement	a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
第10z批第六块 显存账 G2 乙 设备丢了那一刻活着的借据都还了也不回到照读数分	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    self.driver_share = DriverShareOnTheCard::DeviceStarted;	                    self.driver_share =\n                        DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement	a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
第10z批第六块 显存账 G2 乙 设备丢了、账照重起记时不记那一刻活着的借据	crates/singlefs-harness/src/gpu_memory_ledger.rs	            self.lost_devices_not_released.insert(lost_device);	            let _ = lost_device;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement	a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
第10z批第六块 GPU 取卡 G2 乙 借据丢掉时改回只收一次（Poll，不等设备做完）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	    wgpu::PollType::Wait {\n        submission_index: None,\n        timeout: Some(WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE),\n    }	    wgpu::PollType::Poll	-p singlefs-checker-tier --features gpu --test gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card -- a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading	a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading
第10z批第六块 GPU 取卡 G2 乙 没等成也当等完了（照常销账）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        Ok(Ok(wgpu::PollStatus::Poll) | Err(_)) | Err(_) => {\n            DeviceWaitBeforeReturningTheLease::TheWaitFailed	        Ok(Ok(wgpu::PollStatus::Poll) | Err(_)) | Err(_) => {\n            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::a_lease_is_returned_only_when_the_wait_for_the_device_finished	a_lease_is_returned_only_when_the_wait_for_the_device_finished
第10z批第六块 GPU 取卡 K3 F5 扫 wgpu-core 丢设备那一句找的不是 self.lose(&error.to_string())	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        const LOSS_WITH_THE_SENTENCE_OF_THE_HAL_ERROR: &str = "self.lose(&error.to_string());";	        const LOSS_WITH_THE_SENTENCE_OF_THE_HAL_ERROR: &str = "self.lose(error.to_string());";	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::wgpu_core_loses_the_device_with_the_sentence_of_the_hal_error_it_failed_with	wgpu_core_loses_the_device_with_the_sentence_of_the_hal_error_it_failed_with
第10x批 显存账 借据丢掉不销账	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.leases.remove(&lease_identifier).is_some()	        self.leases.get(&lease_identifier).is_some()	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_is_on_the_books_until_it_is_dropped	a_lease_is_on_the_books_until_it_is_dropped
第10z批 显存账 K1-3 重起时旧设备那一份不当销了账而卡上可能还没还的	crates/singlefs-harness/src/gpu_memory_ledger.rs	        | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => return 0,	        | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => booked\n            .leases_mebibytes\n            .saturating_add(mebibytes_left_for_the_driver),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now	a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now
第10z批第五块 显存账 K3 重起被拒之后借一笔把驱动那一份记回登记了、没起（正推 M13）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::RegisteredDeviceNotStarted\n            | DriverShareOnTheCard::DeviceStarted\n            | DriverShareOnTheCard::DeviceLostNotStartedAgain\n            | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => {}\n        }\n    }	            DriverShareOnTheCard::RegisteredDeviceNotStarted\n            | DriverShareOnTheCard::DeviceStarted\n            | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => {}\n            DriverShareOnTheCard::DeviceLostNotStartedAgain => {\n                self.driver_share = DriverShareOnTheCard::RegisteredDeviceNotStarted;\n            }\n        }\n    }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_booked_after_a_refused_restart_keeps_the_restart_judged_again	a_lease_booked_after_a_refused_restart_keeps_the_restart_judged_again
````
