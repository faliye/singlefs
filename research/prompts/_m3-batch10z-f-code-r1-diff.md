# 根环实现第 10z 批第六块：代码轮第一轮改动全文（2026-10-08）

实现员交的 `crates.patch` 原样，打在删除线第 45 批合后的主工作区上。变异表追加、整行替换、删除附在后面；合入时主 agent 把追加的两行（`a_lease_is_returned_only_when_the_wait_for_the_device_finished`、`wgpu_core_loses_the_device_with_the_sentence_of_the_hal_error_it_failed_with`）的筛选词改成只写测试名（门禁 mutation-tables 格认模块认错，调度记录第 856 行）。

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index 38e8cf1..b0312e0 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -49,9 +49,12 @@
 //!    （第 10z 批，第 10y 批代码轮判决 K2-2 P2）；剩下的块别的卡判完了这一趟照常交回。别的卡都已收工、还有交回的块没判时，收尾那一步
 //!    再起一轮，只用没报过的卡（照这一刻能借的重新装状态、借草稿区）判剩下的块，判完的块照块序都入库（第 10z 批第三块，第 10z 批代码轮判决
 //!    K2-1 第 5 条：状态全装在第一张卡时也起第二张）；报过的卡之外没有卡能判才报本次未跑。被叫停时卡照样领判完的块在等的那几块（交回的），
-//!    判完的块照块序都入库再照叫停收尾，没判的留给下一趟（第 10z 批第五块，第 10z 批第三块代码轮判决 K1-5）。卡线程那一圈与按块序写库、
-//!    几轮之间带着的与怎么走在 harness 档（`singlefs_harness::gpu_memory_ledger::judge_blocks_on_one_card`、`BlocksStoredInOrder`、
-//!    `RoundsOfTheCards`），不要卡就测得到。
+//!    判完的块照块序都入库再照叫停收尾，没判的留给下一趟（第 10z 批第五块，第 10z 批第三块代码轮判决 K1-5）；照叫停收尾而这一趟有卡报过
+//!    Out of Memory，头一句照「本次未跑」那一行的形态打出来（第 10z 批第六块，第 10z 批第五块代码轮判决 K1-3）。叫停是叫停文件在不在，
+//!    一趟里见过一次就记住（`singlefs_harness::gpu_memory_ledger::StopRequestRememberedInThisRun`，第 10z 批第五块代码轮判决 K2-2：
+//!    外层循环、卡线程、一轮收尾、整趟收尾都问它，文件中途被删也不各读各的）。卡线程那一圈、起下一张卡、活交回那一刻记设备与按块序写库、
+//!    几轮之间带着的与怎么走在 harness 档（`singlefs_harness::gpu_memory_ledger::judge_blocks_on_one_card`、`next_card_to_start`、
+//!    `work_caught_noting_the_device_when_it_returned`、`BlocksStoredInOrder`、`RoundsOfTheCards`），不要卡就测得到。
 //!    单元校验和对拍时一块的几段在同一个设备上并发，认一段是不是显存不够之前等同块的几段都收完（第 10z 批第三块，攻方：一段因设备丢了先失败，
 //!    同块另一段范围里接到的 Out of Memory 还没记进设备时不判红）。
 //!    没起过卡池的进程照旧不进账（[`GpuCardsSource::DevicesStartedForThisRun`]：这一趟自己在点名的卡上起设备），调用方是这几种：
@@ -81,11 +84,13 @@ use singlefs_harness::gpu_memory_ledger::{
 };
 #[cfg(feature = "gpu")]
 use singlefs_harness::gpu_memory_ledger::{
-    judge_blocks_on_one_card, AfterARoundOfTheCards, BlockHandedToTheStore, BlockJudgedOnACard,
-    BlocksForTheCards, BlocksStoredInOrder, CardLeftTheBlocks, RoundsOfTheCards, StopOfTheCards,
-    StopOfTheRun,
+    judge_blocks_on_one_card, next_card_to_start, work_caught_noting_the_device_when_it_returned,
+    AfterARoundOfTheCards, BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards,
+    BlocksStoredInOrder, CardLeftTheBlocks, RoundsOfTheCards, StopOfTheCards,
+};
+use singlefs_harness::gpu_memory_ledger::{
+    StopRequestRememberedInThisRun, WorkEndingTogether, WorksEndingTogetherOnADevice,
 };
-use singlefs_harness::gpu_memory_ledger::{WorkEndingTogether, WorksEndingTogetherOnADevice};
 use singlefs_harness::hexadecimal::hexadecimal_text;
 use singlefs_harness::memory_pool::{
     root_identity_of_write, root_identity_written_by, CrashImage, MemoryPool, PublishedVersion,
@@ -2111,7 +2116,8 @@ struct CardsAtWork<'work> {
     /// 写库失败、有卡拒判之后置上：卡上的线程判完手里那一块就停、一块都不再领。一张卡报 Out of Memory 不置（只停那一张，第 10z 批）。
     stop: std::sync::atomic::AtomicBool,
     /// 调用方要停（每领一块之前问一次）：不再领新的块，只领判完的块在等的那几块（第 10z 批第五块），手里那一块判完照旧交去写库。
-    stop_is_requested: &'work (dyn Fn() -> bool + Sync),
+    /// 一趟里见过一次就记住（第 10z 批第六块，第 10z 批第五块代码轮判决 K2-2），与一轮收尾问的是同一个。
+    stop_request: &'work StopRequestRememberedInThisRun<'work>,
 }
 
 #[cfg(feature = "gpu")]
@@ -2121,7 +2127,7 @@ impl CardsAtWork<'_> {
     fn stop_of_the_cards(&self) -> StopOfTheCards {
         if self.stop.load(std::sync::atomic::Ordering::SeqCst) {
             StopOfTheCards::TheRunFailed
-        } else if (self.stop_is_requested)() {
+        } else if self.stop_request.is_requested() {
             StopOfTheCards::RequestedByTheCaller
         } else {
             StopOfTheCards::NotRequested
@@ -2195,9 +2201,10 @@ fn on_the_card_catching_out_of_memory_ending_together<Outcome>(
     let out_of_memory_scope = card
         .device()
         .push_error_scope(wgpu::ErrorFilter::OutOfMemory);
-    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
-    // 活交回的这一刻设备丢没丢：之后等同组几段时别的段丢了设备，不把这段活真正的 panic 说成显存不够（第 10z 批第五块）
-    let device_when_the_work_returned = card.device_condition().device_when_the_work_returns();
+    // 活交回的这一刻设备丢没丢：之后等同组几段时别的段丢了设备，不把这段活真正的 panic 说成显存不够（第 10z 批第五块；
+    // 接住与记设备抽在 harness 档，第 10z 批第六块）
+    let (outcome, device_when_the_work_returned) =
+        work_caught_noting_the_device_when_it_returned(card.device_condition(), work);
     let out_of_memory_in_the_scope =
         pollster::block_on(out_of_memory_scope.pop()).map(|error| error.to_string());
     let out_of_memory_on_the_device = card
@@ -2461,15 +2468,17 @@ fn start_the_next_card_with_states<'scope, 'work>(
     position: usize,
     sender: &std::sync::mpsc::Sender<CardMessage>,
 ) {
-    let next_position =
-        (position + 1..work.cards.len()).find(|candidate| work.states_placed[*candidate] > 0);
-    if let Some(next_position) = next_position {
-        if work.blocks.any_left_to_take(work.stop_of_the_cards()) {
-            let sender = sender.clone();
-            scope.spawn(move || {
-                judge_on_one_card_and_start_the_next(scope, work, next_position, sender);
-            });
-        }
+    // 起哪一张、看不看叫停抽在 harness 档（第 10z 批第六块，第 10z 批第五块代码轮判决 K3）
+    if let Some(next_position) = next_card_to_start(
+        work.states_placed,
+        position,
+        &work.blocks,
+        work.stop_of_the_cards(),
+    ) {
+        let sender = sender.clone();
+        scope.spawn(move || {
+            judge_on_one_card_and_start_the_next(scope, work, next_position, sender);
+        });
     }
 }
 
@@ -2934,6 +2943,8 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
     // 几轮之间带着的（harness 档，不要卡就测得到）：按块序写库的那一份（上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写）、
     // 下一轮的那一队块、报过 Out of Memory 的卡与头一句、收过工的卡
     let mut rounds: RoundsOfTheCards<GpuJudgedBlock> = RoundsOfTheCards::new(pending.len());
+    // 卡线程与一轮收尾问的是同一个叫停，见过一次就记住（第 10z 批第六块；调用方 `run_from_environment` 交的已经是记住的那一个）
+    let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);
     // 这一趟本机每张卡的优先级（头一轮那一份，后面几轮只少不多）：收尾时每张卡的账照它排
     let mut card_indices_by_priority: Vec<u32> = Vec::new();
     let started = Instant::now();
@@ -3044,7 +3055,7 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
             scratch_lease_identifiers: &scratch.scratch_lease_identifiers,
             blocks: rounds.take_the_blocks_of_the_round(),
             stop: std::sync::atomic::AtomicBool::new(false),
-            stop_is_requested,
+            stop_request: &stop_request,
         };
         let received = std::thread::scope(|scope| {
             let (sender, receiver) = std::sync::mpsc::channel::<CardMessage>();
@@ -3102,11 +3113,7 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
             .map(|card| card.index)
             .collect();
         outcome.cards.extend(received.finished_cards);
-        let stop = if stop_is_requested() {
-            StopOfTheRun::Requested
-        } else {
-            StopOfTheRun::NotRequested
-        };
+        let stop = stop_request.stop_of_the_run();
         let run_card_indices: Vec<u32> = run_cards.iter().map(|card| card.index).collect();
         // 收工之后的不变量（没有卡报 Out of Memory 时判完的块都写进了库、没叫停时块都领走了）也在这一步里判
         match rounds.after_the_round(
@@ -3125,7 +3132,15 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
                 }
                 break;
             }
-            AfterARoundOfTheCards::StoppedOnRequest => break,
+            AfterARoundOfTheCards::StoppedOnRequest => {
+                // 叫停加有卡报过 Out of Memory：显存不够的缘由照本次未跑那一行打出来（第 10z 批第五块代码轮判决 K1-3）
+                if let Some(not_run_line) = rounds.not_run_line_of_a_stop_after_an_out_of_memory(
+                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
+                ) {
+                    println!("{not_run_line}");
+                }
+                break;
+            }
             AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory => {
                 eprintln!(
                     "CRASH_AMPLIFICATION_GPU_ANOTHER_ROUND 报了 Out of Memory 的卡 {:?} 收工之后还有 {} 块没写进库，收尾再起一轮、用别的卡判剩下的 {} 块",
@@ -4579,6 +4594,73 @@ mod tests {
         }
     }
 
+    /// 这份源文件测试模块之外的代码行（注释行不算）。
+    fn code_lines_outside_the_tests() -> Vec<String> {
+        let source = include_str!("crash_amplification.rs");
+        let tests_start = source
+            .find(concat!("#[cfg(test)]", "\n", "mod tests {"))
+            .expect("这份文件有测试模块");
+        let tests_end = tests_start
+            + source[tests_start..]
+                .find("\n}\n")
+                .expect("测试模块在顶格的右括号处收尾");
+        format!("{}{}", &source[..tests_start], &source[tests_end..])
+            .lines()
+            .filter(|line| !line.trim_start().starts_with("//"))
+            .map(str::to_string)
+            .collect()
+    }
+
+    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K2-2，攻方 F2）：叫停文件只在一处读进这一趟记住的那一个叫停
+    /// （`StopRequestRememberedInThisRun::new`），外层循环、卡线程、一轮收尾与整趟收尾不再各自现读文件。扫这份源码，不要卡。
+    #[test]
+    fn the_run_reads_the_stop_file_only_into_the_stop_request_it_remembers() {
+        let lines_naming_the_stop_file_reader: Vec<String> = code_lines_outside_the_tests()
+            .into_iter()
+            .filter(|line| line.contains(concat!("stop_is_requested_", "by_the_environment")))
+            .map(|line| line.trim().to_string())
+            .collect();
+        assert_eq!(
+            lines_naming_the_stop_file_reader,
+            vec![
+                concat!(
+                    "pub fn stop_is_requested_",
+                    "by_the_environment() -> bool {"
+                )
+                .to_string(),
+                concat!(
+                    "let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_",
+                    "by_the_environment);"
+                )
+                .to_string(),
+            ],
+            "读叫停文件的只有定义它的那一处与这一趟记住叫停的那一处"
+        );
+    }
+
+    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K1-3）：第 ① 段在卡上判、照叫停收尾的那一支打头一句 Out of Memory 那一行
+    /// （`RoundsOfTheCards::not_run_line_of_a_stop_after_an_out_of_memory`，harness 档钉它的形态）。扫这份源码，不要卡。
+    #[test]
+    fn the_gpu_rounds_stopped_on_request_print_the_first_out_of_memory_in_the_not_run_line() {
+        let lines = code_lines_outside_the_tests();
+        let arm = lines
+            .iter()
+            .position(|line| line.trim() == "AfterARoundOfTheCards::StoppedOnRequest => {")
+            .expect("第 ① 段几轮卡的收尾里有照叫停收尾那一支");
+        let arm_body = lines[arm..]
+            .iter()
+            .take_while(|line| line.trim() != "break;")
+            .cloned()
+            .collect::<Vec<String>>()
+            .join("\n");
+        assert!(
+            arm_body.contains(".not_run_line_of_a_stop_after_an_out_of_memory(")
+                && arm_body.contains("PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE")
+                && arm_body.contains("println!(\"{not_run_line}\");"),
+            "照叫停收尾那一支要打头一句 Out of Memory 那一行：{arm_body}"
+        );
+    }
+
     fn registered(classes: &[GpuComputationClass]) -> BTreeSet<GpuComputationClass> {
         classes.iter().copied().collect()
     }
@@ -5507,6 +5589,10 @@ pub fn run_from_environment(
     // 那时登记了 gpu_unit_checks 就把单元两道校验和另交一张卡对拍。显存不够（起过卡池的进程此刻借不到、卡上 wgpu 报了 Out of Memory）：
     // 「本次未跑」那一行已经打了，判过的块在库里，下面照叫停的样子交回（文件头第 5 条）
     let judging_started = std::time::Instant::now();
+    // 这一趟的叫停：第 ① 段的外层循环、卡线程、一轮收尾与下面整趟收尾都问这一个，见过一次就记住（第 10z 批第六块，
+    // 第 10z 批第五块代码轮判决 K2-2 攻方 F2：叫停文件中途被删时各自现读，轮末断言 panic、或整趟没记成提早收尾）
+    let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);
+    let stop_is_requested = || stop_request.is_requested();
     let (
         checking,
         gpu_judge,
@@ -5522,7 +5608,7 @@ pub fn run_from_environment(
             &version,
             cards,
             gpu_cards_taken_from,
-            &stop_is_requested_by_the_environment,
+            &stop_is_requested,
         )? {
             GpuJudgingOfMyBlocks::Judged { tally, outcome } => {
                 eprintln!("{}", gpu_total_line(&outcome, expand_up_to, exhaustive));
@@ -5557,7 +5643,7 @@ pub fn run_from_environment(
                     &selected,
                     &on,
                     &version,
-                    &stop_is_requested_by_the_environment,
+                    &stop_is_requested,
                 )
                 .map_err(|error| format!("核对失败：{error:?}"))?;
                 // 对拍时卡上报了 Out of Memory：打「本次未跑」那一行，照叫停的样子交回（第 10z 批，文件头第 5 条）
@@ -5588,7 +5674,7 @@ pub fn run_from_environment(
             }
         },
     };
-    let stop_was_requested = gpu_memory_was_short || stop_is_requested_by_the_environment();
+    let stop_was_requested = gpu_memory_was_short || stop_is_requested();
     let seconds_judging = judging_started.elapsed().as_secs_f64();
     // 第 ② 段：配置开了 GPU 就核，登记了 crash_verify_gpu 在一张卡上核（起过卡池的照账借第一张放得下的，第 ① 段的草稿区已经放掉）、
     // 没登记用 CPU 参照核；没开 GPU 而 SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu 也用 CPU 参照核；被叫停的不核
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index 4d61e24..aaa6012 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -17,10 +17,30 @@ pub use singlefs_harness::gpu_memory_ledger::DeviceConditionOfACard;
 use singlefs_harness::gpu_memory_ledger::{
     card_memory_reader_of_one_report_per_reading, pci_bus_identifier_without_the_domain,
     BusesWithDevicesStartedOutsideTheBooks, CardMemoryReader, CauseOfTheDeviceLoss,
-    NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
+    DeviceWaitBeforeReturningTheLease, NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
 };
 use singlefs_harness::sha256::DIGEST_BYTES;
 
+/// 卡池的借据丢掉时最多等设备多久（E174 跑前登记 `research/prompts/e174-preregistration.md` 第五节「等待」那一臂：
+/// `PollType::Wait { submission_index: None, timeout: Some(30 秒) }`；E174 两段里这一臂每次都等完了，`wait_always_returns=true`）。
+const WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE: std::time::Duration =
+    std::time::Duration::from_secs(30);
+
+/// 借据丢掉时那一次等（`polled` 是 `poll(Wait)` 的结局，wgpu panic 了是外层的 `Err`）算没算等完：队列空了或等到了才算；
+/// 超时、wgpu 报错、panic 都算没等成。`Wait` 不交 [`wgpu::PollStatus::Poll`]（那是 `PollType::Poll` 的结局），交了照没等成算。
+fn device_wait_before_returning_a_lease(
+    polled: &std::thread::Result<Result<wgpu::PollStatus, wgpu::PollError>>,
+) -> DeviceWaitBeforeReturningTheLease {
+    match polled {
+        Ok(Ok(wgpu::PollStatus::QueueEmpty | wgpu::PollStatus::WaitSucceeded)) => {
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished
+        }
+        Ok(Ok(wgpu::PollStatus::Poll) | Err(_)) | Err(_) => {
+            DeviceWaitBeforeReturningTheLease::TheWaitFailed
+        }
+    }
+}
+
 /// 空闲显存低于这个数（MiB）的卡不用：本机有卡被别的服务占着时只剩几百 MiB，起上下文都难（E163 M2 那张卡 141 MiB 起不来）。
 /// 只管 [`usable_gpu_cards`] 与 [`gpu_cards_by_priority`]；卡池按账判，此刻被别的进程占着的卡照样进池，借的那一刻借不到就报本次未跑（不等）。
 pub const MINIMUM_FREE_MEMORY_MEBIBYTES: u64 = 2048;
@@ -1123,9 +1143,11 @@ pub mod gpu_test_start {
 
     mod card_pool {
         use super::super::{
-            card_memory_reader_of_the_card_pool, memory_budget_mebibytes_of,
-            nvidia_discrete_cards_reported_by_nvidia_smi, nvidia_smi_memory_and_processes_report,
-            start_the_card, CardStartFailure, DeviceOwnerOnTheCard, DiscoveredCard, GpuCard,
+            card_memory_reader_of_the_card_pool, device_wait_before_returning_a_lease,
+            memory_budget_mebibytes_of, nvidia_discrete_cards_reported_by_nvidia_smi,
+            nvidia_smi_memory_and_processes_report, start_the_card, CardStartFailure,
+            DeviceOwnerOnTheCard, DiscoveredCard, GpuCard,
+            WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE,
         };
         use super::{
             decide_gpu_test_start, enumerate_vulkan_adapters, not_run_line,
@@ -1139,7 +1161,8 @@ pub mod gpu_test_start {
         use singlefs_harness::gpu_memory_ledger::{
             card_ledger_capacity_mebibytes, reason_for_a_booking_without_room_now,
             reason_for_running_out_of_gpu_memory, BookingRefusal, CardRoomForThisProcess,
-            DeviceStartedOnDemand, GpuMemoryBooking, GpuMemoryLedger, StatesBookedOnACard,
+            DeviceStartedOnDemand, DeviceWaitBeforeReturningTheLease, GpuMemoryBooking,
+            GpuMemoryLedger, StatesBookedOnACard,
         };
         use singlefs_harness::numeric_environment_variable::{
             number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
@@ -1183,7 +1206,8 @@ pub mod gpu_test_start {
         }
 
         /// 借来的一张卡：[`GpuCardLease::card`] 与池里那张共用设备、编好的内核，显存额度与空闲改成借的量加 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]
-        /// （判器照「额度 − 表 − 那一份」开草稿区，开出来正好是借的那么多）。丢掉时先让设备收掉已丢的缓冲，再销账。
+        /// （判器照「额度 − 表 − 那一份」开草稿区，开出来正好是借的那么多）。丢掉时先等设备把这张卡上提交了的活做完、丢掉的缓冲交回驱动，
+        /// 再销账（最多等 [`WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE`]；等不成照旧记在账上）。
         pub struct GpuCardLease {
             card: GpuCard,
             booking: GpuMemoryBooking<'static>,
@@ -1231,12 +1255,31 @@ pub mod gpu_test_start {
 
         impl Drop for GpuCardLease {
             fn drop(&mut self) {
-                // wgpu 丢掉的缓冲要等设备下一次收拾才还给驱动：销账之前收一次，免得账上还了、卡上还占着，下一个借的建缓冲时撞满
-                if let Err(error) = self.card.device().poll(wgpu::PollType::Poll) {
-                    eprintln!(
-                        "GPU_CARD_LEASE_RETURN_POLL_FAILED index={} error={error:?}",
-                        self.card.index
-                    );
+                // wgpu 丢掉的缓冲要等设备做完这张卡上提交了的活才交回驱动：销账之前等完（用户 2026-10-08 定第 10z 批代码轮岔路 G2「改用乙」，
+                // E174 两张卡上等完之后下一次读数里那笔缓冲都已不在），等不成（超时、wgpu 报错或 panic）这一笔照旧记在账上
+                // （`singlefs_harness::gpu_memory_ledger::GpuMemoryBooking::take_in_the_wait_before_returning`）。
+                // wgpu 把等的时候设备上的错当致命、panic：先把这一笔记成没等成再照样往外抛，不让丢借据那一步把它销了账
+                let device = self.card.device();
+                let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+                    device.poll(wgpu::PollType::Wait {
+                        submission_index: None,
+                        timeout: Some(WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE),
+                    })
+                }));
+                let wait = device_wait_before_returning_a_lease(&polled);
+                match wait {
+                    DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished => {}
+                    DeviceWaitBeforeReturningTheLease::TheWaitFailed => eprintln!(
+                        "GPU_CARD_LEASE_RETURN_WAIT_FAILED index={} lease={} mebibytes={} polled={:?}：这一笔照旧记在账上，到这张卡的设备丢了重起",
+                        self.card.index,
+                        self.booking.lease_identifier(),
+                        self.booking.mebibytes(),
+                        polled.as_ref().map_err(|_panic| "等的时候 wgpu panic 了")
+                    ),
+                }
+                self.booking.take_in_the_wait_before_returning(wait);
+                if let Err(panic) = polled {
+                    std::panic::resume_unwind(panic);
                 }
             }
         }
@@ -1792,12 +1835,15 @@ mod tests {
         PciNvidiaDisplayControllers, PCI_DEVICES_SUBDIRECTORY_NAME,
     };
     use super::{
-        card_memory_reader_of_the_card_pool, cause_of_the_device_loss, failure_of_a_device_request,
+        card_memory_reader_of_the_card_pool, cause_of_the_device_loss,
+        device_wait_before_returning_a_lease, failure_of_a_device_request,
         handle_an_uncaptured_wgpu_error, has_devices_started_outside_the_card_pool_on_the_bus,
         record_the_loss_of_the_device, started_with_its_owner_recorded, CardStartFailure,
         DeviceConditionOfACard, DeviceOwnerOnTheCard,
     };
-    use singlefs_harness::gpu_memory_ledger::{CauseOfTheDeviceLoss, OccupiedByThisProcess};
+    use singlefs_harness::gpu_memory_ledger::{
+        CauseOfTheDeviceLoss, DeviceWaitBeforeReturningTheLease, OccupiedByThisProcess,
+    };
 
     /// 假的 sysfs 里 PCI 总线那一截的样子。
     enum FakePciBus<'devices> {
@@ -2056,11 +2102,13 @@ mod tests {
     /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-4 攻方甲形，wgpu-core 30.0.1 `src/device/resource.rs` 第 706 行、wgpu-hal 30.0.1
     /// `src/lib.rs` 第 382 行）：wgpu 因显存不够丢设备时给的那一句认成显存不够，别的认成别的原因；回调照它记，之后一段因设备丢了而失败的活、
     /// 不在同一组而这段活开始之后一条 Out of Memory 都没记过，也说成显存不够。不要卡。
+    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K3，攻方 F5）：期望的那一句写死成 wgpu-hal 30.0.1 `DeviceError::OutOfMemory` 的
+    /// `#[error("Out of memory")]`，不与产品代码取自同一个 `to_string()`：wgpu 改了这一句，这里先红。
     #[test]
     fn a_device_lost_for_lack_of_memory_explains_a_failure_on_it_as_gpu_memory_short() {
-        let out_of_memory = wgpu::hal::DeviceError::OutOfMemory.to_string();
+        let out_of_memory = "Out of memory";
         assert_eq!(
-            cause_of_the_device_loss(&out_of_memory),
+            cause_of_the_device_loss(out_of_memory),
             CauseOfTheDeviceLoss::GpuMemoryShort
         );
         for other in [
@@ -2077,7 +2125,7 @@ mod tests {
         let condition = DeviceConditionOfACard::default();
         let work = singlefs_harness::gpu_memory_ledger::WorkOnTheCardOfThisThread::begin();
         let reports_before = condition.out_of_memory_reports();
-        record_the_loss_of_the_device(&condition, wgpu::DeviceLostReason::Unknown, &out_of_memory);
+        record_the_loss_of_the_device(&condition, wgpu::DeviceLostReason::Unknown, out_of_memory);
         assert!(
             condition
                 .out_of_memory_explaining_a_failure_of_the_work(
@@ -2124,4 +2172,115 @@ mod tests {
             "照归卡池起设备的只有卡池那一处"
         );
     }
+
+    /// 第 10z 批第六块（G2 乙）：借据丢掉时那一次等，队列空了或等到了才算等完、照常销账；超时、wgpu 报错、等的时候 panic、`Wait` 交回
+    /// `Poll`（那是不等的结局）都算没等成，这一笔照旧记在账上。不要卡。
+    #[test]
+    fn a_lease_is_returned_only_when_the_wait_for_the_device_finished() {
+        let finished = DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished;
+        let failed = DeviceWaitBeforeReturningTheLease::TheWaitFailed;
+        assert_eq!(
+            device_wait_before_returning_a_lease(&Ok(Ok(wgpu::PollStatus::QueueEmpty))),
+            finished
+        );
+        assert_eq!(
+            device_wait_before_returning_a_lease(&Ok(Ok(wgpu::PollStatus::WaitSucceeded))),
+            finished
+        );
+        assert_eq!(
+            device_wait_before_returning_a_lease(&Ok(Ok(wgpu::PollStatus::Poll))),
+            failed,
+            "Wait 交回 Poll：没说等完"
+        );
+        assert_eq!(
+            device_wait_before_returning_a_lease(&Ok(Err(wgpu::PollError::Timeout))),
+            failed,
+            "超时"
+        );
+        assert_eq!(
+            device_wait_before_returning_a_lease(&Err(Box::new("等的时候 wgpu panic 了"))),
+            failed,
+            "等的时候 panic"
+        );
+    }
+
+    /// Cargo.lock 里 wgpu-core 的版本（`name = "wgpu-core"` 下一行的 `version = "…"`）。
+    fn wgpu_core_version_in_the_lock(lock: &str) -> Option<String> {
+        let mut lines = lock.lines();
+        lines.find(|line| line.trim() == "name = \"wgpu-core\"")?;
+        let version_line = lines.next()?.trim();
+        Some(
+            version_line
+                .strip_prefix("version = \"")?
+                .strip_suffix('"')?
+                .to_string(),
+        )
+    }
+
+    /// cargo 缓存（`CARGO_HOME`，没设是 `~/.cargo`）的 `registry/src/*/wgpu-core-<版本>/src/device/resource.rs`；取不到是 `None`。
+    fn wgpu_core_device_source_in_the_cargo_registry(version: &str) -> Option<String> {
+        let cargo_home = std::env::var_os("CARGO_HOME")
+            .map(std::path::PathBuf::from)
+            .or_else(|| {
+                std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".cargo"))
+            })?;
+        std::fs::read_dir(cargo_home.join("registry").join("src"))
+            .ok()?
+            .filter_map(Result::ok)
+            .map(|sources_of_one_registry| {
+                sources_of_one_registry
+                    .path()
+                    .join(format!("wgpu-core-{version}"))
+                    .join("src")
+                    .join("device")
+                    .join("resource.rs")
+            })
+            .find_map(|path| std::fs::read_to_string(path).ok())
+    }
+
+    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K3，攻方 F5）：wgpu-core 因 hal 的错丢设备时，交给设备丢失回调的那一句是那个 hal 错的
+    /// `to_string()`（[`cause_of_the_device_loss`] 照「Out of memory」那一句比，上一条用例钉 hal 那一句）：扫 cargo 缓存里这一版 wgpu-core 的源码，
+    /// `handle_hal_error` 里 `OutOfMemory` 那一支走 `self.lose(&error.to_string());`。版本取仓根 Cargo.lock；源码取不到打「本次未跑」那一行、
+    /// 不报绿。不要卡。
+    #[test]
+    fn wgpu_core_loses_the_device_with_the_sentence_of_the_hal_error_it_failed_with() {
+        const TEST_NAME: &str = "gpu_unit_checks::tests::wgpu_core_loses_the_device_with_the_sentence_of_the_hal_error_it_failed_with";
+        const LOSS_WITH_THE_SENTENCE_OF_THE_HAL_ERROR: &str = "self.lose(&error.to_string());";
+        let lock = std::fs::read_to_string(
+            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock"),
+        )
+        .expect("仓根有 Cargo.lock");
+        let version =
+            wgpu_core_version_in_the_lock(&lock).expect("Cargo.lock 里有 wgpu-core（wgpu 依赖它）");
+        let Some(source) = wgpu_core_device_source_in_the_cargo_registry(&version) else {
+            println!(
+                "{}",
+                singlefs_harness::gpu_memory_ledger::gpu_not_run_line(
+                    TEST_NAME,
+                    &format!(
+                        "cargo 缓存里没有 wgpu-core {version} 的 src/device/resource.rs（CARGO_HOME 或 ~/.cargo 下的 registry/src）"
+                    )
+                )
+            );
+            return;
+        };
+        let handle_start = source
+            .find("pub fn handle_hal_error(")
+            .expect("wgpu-core 的设备上有 handle_hal_error：没有就是这一版改了丢设备的路，要重看");
+        let handle_body = &source[handle_start..];
+        let handle_body = &handle_body[..handle_body
+            .find("\n    }\n")
+            .expect("handle_hal_error 以缩进四格的右括号收尾")];
+        assert!(
+            handle_body.contains("hal::DeviceError::OutOfMemory"),
+            "wgpu-core {version} 的 handle_hal_error 不再认 hal 的 OutOfMemory：{handle_body}"
+        );
+        assert_eq!(
+            handle_body
+                .matches(LOSS_WITH_THE_SENTENCE_OF_THE_HAL_ERROR)
+                .count(),
+            1,
+            "wgpu-core {version} 丢设备时交给回调的不再是 hal 错的那一句：{handle_body}"
+        );
+    }
 }
diff --git a/crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs b/crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs
new file mode 100644
index 0000000..77726ff
--- /dev/null
+++ b/crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs
@@ -0,0 +1,104 @@
+//! checker 档模块：gpu_unit_checks
+//! 卡池的借据丢掉时先等设备做完、再销账（第 10z 批第六块，用户 2026-10-08 定第 10z 批代码轮岔路 G2「改用乙」）：在本机的卡上借一笔、
+//! 建两张 1 GiB 的缓冲、提交一次清零（不等），丢掉缓冲与借据之后读一次 `nvidia-smi -q -d MEMORY,PIDS`，这个进程在那张卡上占着的已经不含
+//! 那笔缓冲。读法照 E174（借据丢掉时等待与显存回还）跑前登记 `research/prompts/e174-preregistration.md` 第一节：建缓冲之前读三份取中位数
+//! U_pre，τ 是那三份的极差加 2 MiB；缓冲活着时读一份，比 U_pre 多出的不少于 S − τ（阳性对照：读数分得出这笔缓冲）；借据丢掉之后读一份，
+//! 比 U_pre 多出的不超过 τ。要 `gpu` 特性；本机一张 NVIDIA 独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
+//! （`gpu_unit_checks::gpu_test_start`）。
+#![cfg(feature = "gpu")]
+
+use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
+use singlefs_harness::gpu_memory_ledger::{
+    card_memory_readings_of_one_nvidia_smi_report, OccupiedByThisProcess,
+    NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
+};
+
+const TEST_NAME: &str =
+    "a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading";
+/// 一张缓冲 1 GiB（E174 第五节：S 大于 1 GiB 的建 ⌈S / 1 GiB⌉ 张、每张 1 GiB）。
+const MEBIBYTES_OF_ONE_BUFFER: u64 = 1024;
+const BUFFERS: u64 = 2;
+/// 两份各自取整到 MiB 的读数，相减最多差出不到 2 MiB（E174 第六节 τ 的来由）。
+const ROUNDING_OF_TWO_READINGS_MEBIBYTES: u64 = 2;
+const BYTES_IN_A_MEBIBYTE: u64 = 1024 * 1024;
+
+/// 这个进程此刻在总线号 `bus`（去掉域的写法）那张卡上占着的（MiB；报告里没列这个进程是 0）：跑一次 `nvidia-smi -q -d MEMORY,PIDS`。
+fn this_process_occupancy_on_the_card_mebibytes(bus: &str) -> u64 {
+    let output = std::process::Command::new("nvidia-smi")
+        .args(NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS)
+        .output()
+        .expect("借到了卡的机器上起得了 nvidia-smi");
+    assert!(output.status.success(), "nvidia-smi 退了 {}", output.status);
+    let readings = card_memory_readings_of_one_nvidia_smi_report(
+        &String::from_utf8_lossy(&output.stdout),
+        std::process::id(),
+        &[bus.to_string()],
+        &|_bus| false,
+    )
+    .expect("报告里有借到的那张卡");
+    match readings[0].occupied_by_this_process {
+        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => mebibytes,
+        OccupiedByThisProcess::NotListed => 0,
+        OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes } => {
+            panic!("这条用例不起卡池之外的设备，读到分不出的 {mebibytes} MiB")
+        }
+    }
+}
+
+#[test]
+fn a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading() {
+    let Some(pool) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
+        return;
+    };
+    let size_mebibytes = MEBIBYTES_OF_ONE_BUFFER * BUFFERS;
+    let Some(lease) = pool.lease_card_or_report_not_run(TEST_NAME, size_mebibytes) else {
+        return;
+    };
+    let index = lease.card().index;
+    let lease_identifier = lease.lease_identifier();
+    let bus = lease.card().pci_bus_identifier.clone();
+    let mut readings_before: Vec<u64> = (0..3)
+        .map(|_| this_process_occupancy_on_the_card_mebibytes(&bus))
+        .collect();
+    readings_before.sort_unstable();
+    let occupied_before = readings_before[1];
+    let tolerance = readings_before[2] - readings_before[0] + ROUNDING_OF_TWO_READINGS_MEBIBYTES;
+    let device = lease.card().device();
+    let buffers: Vec<wgpu::Buffer> = (0..BUFFERS)
+        .map(|_| {
+            device.create_buffer(&wgpu::BufferDescriptor {
+                label: Some(TEST_NAME),
+                size: MEBIBYTES_OF_ONE_BUFFER * BYTES_IN_A_MEBIBYTE,
+                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
+                mapped_at_creation: false,
+            })
+        })
+        .collect();
+    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
+        label: Some(TEST_NAME),
+    });
+    for buffer in &buffers {
+        encoder.clear_buffer(buffer, 0, None);
+    }
+    lease.card().queue().submit([encoder.finish()]);
+    let occupied_while_the_buffers_live = this_process_occupancy_on_the_card_mebibytes(&bus);
+    assert!(
+        occupied_while_the_buffers_live + tolerance >= occupied_before + size_mebibytes,
+        "阳性对照：缓冲活着时读数里有它（建之前 {occupied_before}、活着时 {occupied_while_the_buffers_live}、S {size_mebibytes}、τ {tolerance}，MiB）"
+    );
+    drop(buffers);
+    drop(lease);
+    assert!(
+        !pool.is_the_lease_on_the_books(index, lease_identifier),
+        "借据丢掉、等完设备之后销了账"
+    );
+    let occupied_after_the_lease_was_returned = this_process_occupancy_on_the_card_mebibytes(&bus);
+    println!(
+        "GPU_CARD_LEASE_RETURNED index={index} occupied_before={occupied_before} occupied_while_the_buffers_live={occupied_while_the_buffers_live} \
+         occupied_after_the_lease_was_returned={occupied_after_the_lease_was_returned} tolerance={tolerance}"
+    );
+    assert!(
+        occupied_after_the_lease_was_returned <= occupied_before + tolerance,
+        "借据丢掉、等完设备之后下一次读数里那笔缓冲已经不在（建之前 {occupied_before}、之后 {occupied_after_the_lease_was_returned}、τ {tolerance}，MiB）"
+    );
+}
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
index 72fb503..e8837d6 100644
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -20,17 +20,16 @@
 //! - 额度变量 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` − B（设了才有这一项）；
 //! - 配置给卡的额度 − D − B；没配显卡优先级时由 M 的 3/4 − D − B 代替（这几项都是这个进程在这张卡上一共借的连驱动那一份不超过额度）；
 //! - F − 这个进程许出去而还没占上卡的那一份：许出去的是 B 连驱动那一份 D（还没登记时 B 是 0、D 照样要放得下），
-//!   减实际占着的里属于许出去的那部分。实际占着的先扣掉这个进程已经销账、卡上可能还没还的那一份
-//!   （[`BookedByThisProcessOnACard::returned_possibly_still_on_the_card_mebibytes`]，第 10x 批代码轮判决 K1-1：不当成别的借据已经建上的）；
-//!   那一份每读一次只压到读数里这个进程此刻实际占着的为止（[`returned_possibly_still_on_the_card_after_a_reading`]），不再往下减驱动那一份、
-//!   也不减借据建上的（用户 2026-10-08 定第 10z 批代码轮岔路 G2「先回甲」：读数只给这个进程的合计，压得比合计更低就要猜每笔借据此刻在卡上
-//!   占多少，猜错一边多给——第 10z 批代码轮判决 K2-1 第 2 条，活着的借据缓冲缩回驱动、同时一笔销了账的卡上还没还；照驱动那一份整份在卡上压是
-//!   第 10y 批代码轮判决 K1-1 攻方丙形，驱动实占的 x 比 D 小时多给 D − x）。
-//!   代价是少借：有借据建着或设备起着时，已经还给驱动的那一份要到这个进程占着的比它还少才压得掉（第 10y 批代码轮判决 K2-2 P1 那一形，
-//!   长借据建着时每还一笔短借据多扣一份），借不到报本次未跑。
+//!   减实际占着的里属于许出去的那部分。账上只有活着的借据：借据丢掉时卡池先等这张卡的设备把提交了的活都做完、丢掉的缓冲交回驱动，
+//!   再销账（[`GpuMemoryBooking::take_in_the_wait_before_returning`]；用户 2026-10-08 定第 10z 批代码轮岔路 G2「改用乙」，E174 在两张卡上量过：
+//!   等完之后下一次读数里那笔缓冲都已不在），销了账的那一份不再记、也不从空闲里扣（第 10z 批「甲」那一份「销了账、卡上可能还没还的」整个拿掉）。
+//!   等失败（超时或报错）的那一笔照旧记在账上，直到这张卡的设备丢了、账照重起记（`BooksOfACard::take_back_the_driver_share_of_a_lost_device`），
+//!   只会少借。
 //!   读不到实际占着的（`nvidia-smi` 没列这个进程），或这个进程在这张卡上另有不走这份账的设备（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]，
 //!   分不出哪一份是借据建的），借着的当一点都没建、驱动那一份也当还没占上卡（第 10y 批代码轮判决 K1-1 攻方乙附：当它整份在卡上，实占的 x 比 D 小时
-//!   多给 D − x），只会少借。
+//!   多给 D − x），只会少借。这张卡上起好过的设备丢了、账照重起记了之后，到那一刻活着的借据都还了之前也这样算
+//!   （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]、[`DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain`]：
+//!   那几笔拿着旧设备，旧设备连它上面的缓冲可能还占着卡，这个进程占着的分不出哪一份是新设备的）。
 //!   不走这份账的设备在起之前就记下它在哪张卡上（[`BusesWithDevicesStartedOutsideTheBooks::start_a_device_marked_first`]，第 10y 批代码轮判决 K1-1
 //!   攻方乙形：起好了才记，起设备到记下之间那一段读数里已经有那个设备的占用，被当成借据建上的）。
 //!
@@ -49,7 +48,7 @@
 //! 先登记、后建设备（这个进程里）：一个进程在一张卡上第一次借时，这一笔连同它的驱动那一份放得下才记、才把这张卡记成登记了；
 //! 调用方拿到借据之后才在这张卡上起设备（[`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`]），
 //! 起好了记一笔（[`GpuMemoryLedger::record_the_device_started_on_the_card`]），从此驱动那一份算实际占在卡上。设备丢了要重起时，驱动那一份记回
-//! 许出去而还没占上卡、旧设备那一份当销了账而卡上可能还没还的，此刻放不下就不起（第 10y 批代码轮判决 K1-3：重起不另登记驱动那一份，
+//! 许出去而还没占上卡、这个进程占着的照分不出算（上一段），此刻放不下就不起（第 10y 批代码轮判决 K1-3：重起不另登记驱动那一份，
 //! 新设备的上下文没留地方）；被拒之后新设备起好之前，每一次要起都照此刻再判一次（第 10z 批代码轮判决 K1-3 正推 S6：被拒之前已借到的
 //! 并发用例再来起设备时不直接放行）。
 //!
@@ -65,7 +64,7 @@
 use std::cell::Cell;
 use std::collections::{BTreeMap, BTreeSet};
 use std::marker::PhantomData;
-use std::sync::atomic::{AtomicU64, Ordering};
+use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
 use std::sync::{Mutex, MutexGuard, PoisonError};
 
 /// 一个判器一次派活最多带几个状态：显存装得下的、调用方要的、这条流一共几个状态，三者取小，至少 1。
@@ -151,17 +150,20 @@ pub enum DriverShareOnTheCard {
     DeviceStarted,
     /// 起好过的设备丢了、新设备还没起好（重起时此刻放不下被拒过，或还没起）：驱动那一份许出去、还没占在卡上；
     /// 每一次要起新设备都照此刻再判一次放不放得下（[`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`]）。
+    /// 丢了的设备连它上面的缓冲可能还占着卡，这个进程占着的分不出哪一份是谁的：照读不到实际占着的算
+    /// （[`occupied_part_of_what_this_process_promised_mebibytes`] 交 0）。
     DeviceLostNotStartedAgain,
+    /// 丢了的设备重起好了，而设备丢了、账照重起记的那一刻活着的借据还有没还的：那几笔拿着旧设备，旧设备连它上面的缓冲可能还占着卡，
+    /// 照读不到实际占着的算；那几笔都还了才回到 [`Self::DeviceStarted`]（第 10z 批第六块，G2 乙：拿掉「销了账、卡上可能还没还的」那一份之后，
+    /// 旧设备留在卡上的不再记成一份）。
+    DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain,
 }
 
-/// 这个进程在一张卡上记着的：借着的合计（MiB）、驱动那一份走到哪一步，与已经销账而卡上可能还没还的那一份。
+/// 这个进程在一张卡上记着的：活着的借据合计（MiB）与驱动那一份走到哪一步。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct BookedByThisProcessOnACard {
     pub leases_mebibytes: u64,
     pub driver_share: DriverShareOnTheCard,
-    /// 丢掉的借据建的缓冲：wgpu 要等设备收拾才还给驱动，销了账卡上可能还占着（MiB，上界；每读一次照实际占着的往下压，
-    /// [`GpuMemoryLedger`] 的 `rooms_of_the_cards`）。实际占着的先扣掉它，才算借着的已经建上的（第 10x 批代码轮判决 K1-1，正推冲突 B）。
-    pub returned_possibly_still_on_the_card_mebibytes: u64,
 }
 
 /// 这个进程此刻能在一张卡上占多少（MiB，文件头「口径」里的 M）：现读的空闲，加上这个进程实际占着的里属于它许出去的那一部分
@@ -181,9 +183,11 @@ pub fn memory_this_process_may_hold_mebibytes(
         ))
 }
 
-/// 这个进程实际占在一张卡上的里，属于它许出去的那一部分（MiB；许出去的是借着的，加登记了时驱动那一份）：实际占着的先扣掉已经销账、
-/// 卡上可能还没还的那一份；比许出去的多（分配器按块拿）只算到许出去的那么多；读不到实际占着的、或这个进程在这张卡上另有不走这份账的设备，
-/// 是 0：借着的当一点都没建，驱动那一份也当还没占上卡（驱动实占多少分不出）。
+/// 这个进程实际占在一张卡上的里，属于它许出去的那一部分（MiB；许出去的是活着的借据，加登记了时驱动那一份）：比许出去的多（分配器按块拿）
+/// 只算到许出去的那么多；读不到实际占着的、这个进程在这张卡上另有不走这份账的设备、或这张卡上丢了的设备可能还占着
+/// （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]、[`DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain`]），
+/// 是 0：借着的当一点都没建，驱动那一份也当还没占上卡（驱动实占多少、哪一份是旧设备的都分不出）。
+/// 销了账的借据不在许出去的里，它的缓冲在销账之前已经等设备交回驱动（文件头「口径」，G2 乙），不再从实际占着的里扣。
 #[must_use]
 pub fn occupied_part_of_what_this_process_promised_mebibytes(
     reading: CardMemoryReading,
@@ -192,38 +196,21 @@ pub fn occupied_part_of_what_this_process_promised_mebibytes(
 ) -> u64 {
     let promised_mebibytes = match booked.driver_share {
         DriverShareOnTheCard::NotRegistered => booked.leases_mebibytes,
-        DriverShareOnTheCard::RegisteredDeviceNotStarted
-        | DriverShareOnTheCard::DeviceStarted
-        | DriverShareOnTheCard::DeviceLostNotStartedAgain => booked
-            .leases_mebibytes
-            .saturating_add(mebibytes_left_for_the_driver),
+        DriverShareOnTheCard::RegisteredDeviceNotStarted | DriverShareOnTheCard::DeviceStarted => {
+            booked
+                .leases_mebibytes
+                .saturating_add(mebibytes_left_for_the_driver)
+        }
+        // 旧设备留在卡上的那一堆读数里与新设备的分不出：当一点都不是许出去的（只会少借）
+        DriverShareOnTheCard::DeviceLostNotStartedAgain
+        | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => return 0,
     };
     match reading.occupied_by_this_process {
-        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => mebibytes
-            .saturating_sub(booked.returned_possibly_still_on_the_card_mebibytes)
-            .min(promised_mebibytes),
-        // 驱动那一份当整份在卡上，实占的 x 比 D 小时多给 D − x（第 10y 批代码轮判决 K1-1 攻方乙附）：驱动之后还会长到 D
-        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => 0,
-    }
-}
-
-/// 销了账、卡上可能还没还的那一份照这一次读数往下压（MiB）：它在卡上的那一截是这个进程实际占着的一部分，压到不超过读数里这个进程
-/// 此刻占着的；读不到实际占着的、或另有不走这份账的设备，照旧（分不出就不压）。
-/// 不往下减驱动那一份、也不减借据建上的（用户 2026-10-08 定第 10z 批代码轮岔路 G2「先回甲」，文件头「口径」）：读数只给这个进程的合计，
-/// 驱动实占的 x 比 D 小时减 D 压过头、多给 D − x（第 10y 批代码轮判决 K1-1 攻方丙形）；照早先见到的借据建上的减，借据的缓冲缩回驱动时压过头、
-/// 多给（第 10z 批代码轮判决 K2-1 第 2 条）。压过头的那截在下一笔里被当成借据已经建上的。
-#[must_use]
-pub fn returned_possibly_still_on_the_card_after_a_reading(
-    returned_possibly_still_on_the_card_mebibytes: u64,
-    occupied_by_this_process: OccupiedByThisProcess,
-) -> u64 {
-    match occupied_by_this_process {
         OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => {
-            returned_possibly_still_on_the_card_mebibytes.min(mebibytes)
-        }
-        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {
-            returned_possibly_still_on_the_card_mebibytes
+            mebibytes.min(promised_mebibytes)
         }
+        // 驱动那一份当整份在卡上，实占的 x 比 D 小时多给 D − x（第 10y 批代码轮判决 K1-1 攻方乙附）：驱动之后还会长到 D
+        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => 0,
     }
 }
 
@@ -828,13 +815,18 @@ pub fn card_memory_reader_of_one_report_per_reading(
 /// 卡池照 [`card_ledger_capacity_mebibytes`] 算）。
 pub type CardCapacityForThisProcess = Box<dyn Fn(usize, u64) -> u64 + Send + Sync>;
 
-/// 这个进程在一张卡上记着的：每笔借据（借据号 → MiB）与驱动那一份走到哪一步。
+/// 这个进程在一张卡上记着的：每笔活着的借据（借据号 → MiB）、驱动那一份走到哪一步、设备丢了那一刻活着的借据里还没还的，
+/// 与等设备失败而照旧记着的借据。
 #[derive(Clone, Debug, PartialEq, Eq)]
 struct BooksOfACard {
     leases: BTreeMap<u64, u64>,
     driver_share: DriverShareOnTheCard,
-    /// 销了账、卡上可能还没还的（[`BookedByThisProcessOnACard::returned_possibly_still_on_the_card_mebibytes`]）。
-    returned_possibly_still_on_the_card_mebibytes: u64,
+    /// 这张卡上起好过的设备丢了、账照重起记（[`Self::take_back_the_driver_share_of_a_lost_device`]）那一刻记着的借据里还没还的：
+    /// 不空时这个进程占着的照分不出算（[`DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain`]）。
+    leases_alive_when_the_device_was_lost: BTreeSet<u64>,
+    /// 丢掉时等设备没等成（超时或报错）、照旧记在 `leases` 里的借据（[`GpuMemoryBooking::take_in_the_wait_before_returning`]）：
+    /// 设备丢了、账照重起记的那一刻销账。
+    leases_kept_after_a_failed_wait: BTreeSet<u64>,
 }
 
 impl BooksOfACard {
@@ -843,48 +835,51 @@ impl BooksOfACard {
         Self {
             leases: BTreeMap::new(),
             driver_share: DriverShareOnTheCard::NotRegistered,
-            returned_possibly_still_on_the_card_mebibytes: 0,
+            leases_alive_when_the_device_was_lost: BTreeSet::new(),
+            leases_kept_after_a_failed_wait: BTreeSet::new(),
         }
     }
 
-    /// 读到一次这张卡的显存：销了账、卡上可能还没还的那一份照它往下压（[`returned_possibly_still_on_the_card_after_a_reading`]）。
-    fn take_in_a_reading(&mut self, reading: CardMemoryReading) {
-        self.returned_possibly_still_on_the_card_mebibytes =
-            returned_possibly_still_on_the_card_after_a_reading(
-                self.returned_possibly_still_on_the_card_mebibytes,
-                reading.occupied_by_this_process,
-            );
-    }
-
-    /// 这张卡上起好过的设备丢了、要重起：驱动那一份记回许出去而还没占上卡，旧设备那一份（`mebibytes_left_for_the_driver`）当销了账而卡上
-    /// 可能还没还的。
-    fn take_back_the_driver_share_of_a_lost_device(&mut self, mebibytes_left_for_the_driver: u64) {
+    /// 这张卡上起好过的设备丢了、要重起：驱动那一份记回许出去而还没占上卡；等设备没等成而照旧记着的那几笔这时销账
+    /// （它们的缓冲在旧设备上，旧设备留在卡上的照分不出算，不再按借据记）；此刻记着的借据记成设备丢了那一刻活着的。
+    fn take_back_the_driver_share_of_a_lost_device(&mut self) {
         self.driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;
-        self.returned_possibly_still_on_the_card_mebibytes = self
-            .returned_possibly_still_on_the_card_mebibytes
-            .saturating_add(mebibytes_left_for_the_driver);
+        for kept_lease in std::mem::take(&mut self.leases_kept_after_a_failed_wait) {
+            self.leases.remove(&kept_lease);
+            self.leases_alive_when_the_device_was_lost
+                .remove(&kept_lease);
+        }
+        self.leases_alive_when_the_device_was_lost
+            .extend(self.leases.keys().copied());
     }
 
     fn booked(&self) -> BookedByThisProcessOnACard {
         BookedByThisProcessOnACard {
             leases_mebibytes: self.leases.values().sum(),
             driver_share: self.driver_share,
-            returned_possibly_still_on_the_card_mebibytes: self
-                .returned_possibly_still_on_the_card_mebibytes,
         }
     }
 
-    /// 销一笔借据（借据丢掉时）：它建的缓冲记进卡上可能还没还的那一份。交回销没销到。
+    /// 销一笔借据（借据丢掉、等完设备时）：它的缓冲已经交回驱动，账上不再记它。设备丢了那一刻活着的借据都还了，重起好的设备回到
+    /// [`DriverShareOnTheCard::DeviceStarted`]。交回销没销到。
     fn return_a_lease(&mut self, lease_identifier: u64) -> bool {
-        match self.leases.remove(&lease_identifier) {
-            Some(mebibytes) => {
-                self.returned_possibly_still_on_the_card_mebibytes = self
-                    .returned_possibly_still_on_the_card_mebibytes
-                    .saturating_add(mebibytes);
-                true
+        if self.leases.remove(&lease_identifier).is_none() {
+            return false;
+        }
+        self.leases_alive_when_the_device_was_lost
+            .remove(&lease_identifier);
+        if self.leases_alive_when_the_device_was_lost.is_empty() {
+            match self.driver_share {
+                DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => {
+                    self.driver_share = DriverShareOnTheCard::DeviceStarted;
+                }
+                DriverShareOnTheCard::NotRegistered
+                | DriverShareOnTheCard::RegisteredDeviceNotStarted
+                | DriverShareOnTheCard::DeviceStarted
+                | DriverShareOnTheCard::DeviceLostNotStartedAgain => {}
             }
-            None => false,
         }
+        true
     }
 
     /// 记一笔借据；这个进程还没在这张卡上登记的，一并登记（设备还没起）。
@@ -900,7 +895,8 @@ impl BooksOfACard {
             }
             DriverShareOnTheCard::RegisteredDeviceNotStarted
             | DriverShareOnTheCard::DeviceStarted
-            | DriverShareOnTheCard::DeviceLostNotStartedAgain => {}
+            | DriverShareOnTheCard::DeviceLostNotStartedAgain
+            | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => {}
         }
     }
 }
@@ -949,10 +945,10 @@ impl GpuMemoryLedger {
         self.books.lock().unwrap_or_else(PoisonError::into_inner)
     }
 
-    /// 在锁里现读每张卡（次序与池一样），每张卡的账照这一次读数记（`BooksOfACard::take_in_a_reading`：销了账、卡上可能还没还的那一份往下压）。
-    fn readings_taken_into_the_books(
+    /// 在锁里现读每张卡（次序与池一样，`books` 是锁里的账）：读到的卡数与池对不上是读不出。
+    fn readings_of_every_card(
         &self,
-        books: &mut [BooksOfACard],
+        books: &[BooksOfACard],
     ) -> Result<Vec<CardMemoryReading>, BookingRefusal> {
         let readings = (self.read_card_memory)()
             .map_err(|reason| BookingRefusal::CardMemoryUnreadable { reason })?;
@@ -965,19 +961,16 @@ impl GpuMemoryLedger {
                 ),
             });
         }
-        for (books_of_the_card, reading) in books.iter_mut().zip(&readings) {
-            books_of_the_card.take_in_a_reading(*reading);
-        }
         Ok(readings)
     }
 
-    /// 在锁里现读每张卡（[`Self::readings_taken_into_the_books`]），算这几张卡此刻各能借给这个进程的（与 `card_positions` 逐张对上）。
+    /// 在锁里现读每张卡（[`Self::readings_of_every_card`]），算这几张卡此刻各能借给这个进程的（与 `card_positions` 逐张对上）。
     fn rooms_of_the_cards(
         &self,
-        books: &mut [BooksOfACard],
+        books: &[BooksOfACard],
         card_positions: &[usize],
     ) -> Result<Vec<CardRoomForThisProcess>, BookingRefusal> {
-        let readings = self.readings_taken_into_the_books(books)?;
+        let readings = self.readings_of_every_card(books)?;
         Ok(card_positions
             .iter()
             .map(|card_position| {
@@ -1036,8 +1029,8 @@ impl GpuMemoryLedger {
         &self,
         card_positions: &[usize],
     ) -> Result<Vec<CardRoomForThisProcess>, BookingRefusal> {
-        let mut books = self.books_locked();
-        self.rooms_of_the_cards(&mut books, card_positions)
+        let books = self.books_locked();
+        self.rooms_of_the_cards(&books, card_positions)
     }
 
     /// 借 `need_mebibytes`：照卡的优先级记在第一张此刻放得下的卡上（这个进程在那张卡上还没登记的，它的驱动那一份也要放得下，
@@ -1082,7 +1075,7 @@ impl GpuMemoryLedger {
             "借 0 MiB 照样要登记驱动那一份：调用方按实际要建的缓冲借，至少 1 MiB"
         );
         let mut books = self.books_locked();
-        let rooms = self.rooms_of_the_cards(&mut books, card_positions)?;
+        let rooms = self.rooms_of_the_cards(&books, card_positions)?;
         match decide_booking_by_priority(&rooms, need_mebibytes) {
             BookingDecision::BookOnTheCardAt { card_position } => {
                 let booked = self.book_in_the_lock(
@@ -1146,7 +1139,7 @@ impl GpuMemoryLedger {
             "一个状态要 0 MiB：判器按状态数借的那一族至少有表与一个状态的草稿区"
         );
         let mut books = self.books_locked();
-        let rooms = self.rooms_of_the_cards(&mut books, card_positions)?;
+        let rooms = self.rooms_of_the_cards(&books, card_positions)?;
         let mut booked_on_each_card: Vec<Option<BookedOnACard>> =
             card_positions.iter().map(|_| None).collect();
         let attempts = place_states_on_the_cards_that_book_now(
@@ -1223,6 +1216,7 @@ impl GpuMemoryLedger {
             mebibytes: booked.mebibytes_booked,
             lease_identifier: booked.lease_identifier,
             card_room_mebibytes_when_booked: booked.room_mebibytes_when_booked,
+            when_dropped: LeaseWhenTheBookingIsDropped::ReturnedToTheDriverAndOffTheBooks,
         }
     }
 
@@ -1233,7 +1227,8 @@ impl GpuMemoryLedger {
             DriverShareOnTheCard::NotRegistered => false,
             DriverShareOnTheCard::RegisteredDeviceNotStarted
             | DriverShareOnTheCard::DeviceStarted
-            | DriverShareOnTheCard::DeviceLostNotStartedAgain => true,
+            | DriverShareOnTheCard::DeviceLostNotStartedAgain
+            | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => true,
         }
     }
 
@@ -1250,8 +1245,8 @@ impl GpuMemoryLedger {
 
     /// 在第 `card_position` 张卡上起设备之前调（先登记、后建设备）：这个进程要已经在这张卡上借过一笔。设备是头一回起（或上一回没起来），
     /// 驱动那一份在登记的那一笔里一起借过，照旧；起过的设备丢了要重起（驱动那一份记着已经占在卡上），把它记回许出去而还没占上卡、
-    /// 旧设备那一份当销了账而卡上可能还没还的（`BooksOfACard::take_back_the_driver_share_of_a_lost_device`），在锁里现读一次：
-    /// 借着的连新设备的驱动那一份此刻放不下，交借不到，调用方不起（第 10y 批代码轮判决 K1-3 攻方：重起不另登记驱动那一份，
+    /// 这个进程占着的照分不出算（`BooksOfACard::take_back_the_driver_share_of_a_lost_device`：旧设备连它上面的缓冲可能还占着），
+    /// 在锁里现读一次：借着的连新设备的驱动那一份照此刻的空闲放不下，交借不到，调用方不起（第 10y 批代码轮判决 K1-3 攻方：重起不另登记驱动那一份，
     /// 触发重起的那一笔是按驱动那一份已在卡上借的，新设备的上下文没留地方）。交借不到之后驱动那一份照旧记着许出去而没占上卡
     /// （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]），新设备起好之前每一次再来都照此刻再判一次（第 10z 批代码轮判决 K1-3 正推 S6：
     /// 被拒之前已借到、借据还在手上的并发用例是按丢之前的账借的，再来起设备时不直接放行）。
@@ -1271,14 +1266,13 @@ impl GpuMemoryLedger {
                 "第 {card_position} 张卡：这个进程在它的账上登记之前就要起设备（先登记、后建设备）"
             ),
             DriverShareOnTheCard::RegisteredDeviceNotStarted => Ok(()),
-            DriverShareOnTheCard::DeviceStarted => {
-                books[card_position].take_back_the_driver_share_of_a_lost_device(
-                    self.mebibytes_each_process_keeps_on_a_card,
-                );
-                self.driver_share_of_a_new_device_fits_now(&mut books, card_position)
+            DriverShareOnTheCard::DeviceStarted
+            | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => {
+                books[card_position].take_back_the_driver_share_of_a_lost_device();
+                self.driver_share_of_a_new_device_fits_now(&books, card_position)
             }
             DriverShareOnTheCard::DeviceLostNotStartedAgain => {
-                self.driver_share_of_a_new_device_fits_now(&mut books, card_position)
+                self.driver_share_of_a_new_device_fits_now(&books, card_position)
             }
         }
     }
@@ -1289,10 +1283,10 @@ impl GpuMemoryLedger {
     /// 同 [`Self::prepare_the_driver_share_before_starting_the_device`]。
     fn driver_share_of_a_new_device_fits_now(
         &self,
-        books: &mut [BooksOfACard],
+        books: &[BooksOfACard],
         card_position: usize,
     ) -> Result<(), BookingRefusal> {
-        let readings = self.readings_taken_into_the_books(books)?;
+        let readings = self.readings_of_every_card(books)?;
         let booked = books[card_position].booked();
         let capacity_mebibytes = (self.capacity_for_this_process)(
             card_position,
@@ -1313,13 +1307,24 @@ impl GpuMemoryLedger {
         Ok(())
     }
 
-    /// 第 `card_position` 张卡上的设备起好了：从此这个进程的驱动那一份算实际占在卡上（读到实际占着的时候照它算）。
+    /// 第 `card_position` 张卡上的设备起好了：从此这个进程的驱动那一份算实际占在卡上（读到实际占着的时候照它算）；起的是丢了的设备的替身、
+    /// 设备丢了那一刻活着的借据还有没还的，到它们都还了之前照分不出算
+    /// （[`DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain`]）。
     ///
     /// # Panics
     /// 同 [`Self::check_registered_before_starting_the_device`]。
     pub fn record_the_device_started_on_the_card(&self, card_position: usize) {
         self.check_registered_before_starting_the_device(card_position);
-        self.books_locked()[card_position].driver_share = DriverShareOnTheCard::DeviceStarted;
+        let mut books = self.books_locked();
+        let books_of_the_card = &mut books[card_position];
+        books_of_the_card.driver_share = if books_of_the_card
+            .leases_alive_when_the_device_was_lost
+            .is_empty()
+        {
+            DriverShareOnTheCard::DeviceStarted
+        } else {
+            DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain
+        };
     }
 
     /// 借据号是 `lease_identifier` 的那一笔此刻还记在第 `card_position` 张卡上没有（还了就不在了）。
@@ -1337,13 +1342,33 @@ pub struct StatesBookedOnACard<'ledger> {
     pub booking: GpuMemoryBooking<'ledger>,
 }
 
-/// 账上的一笔：记在哪张卡、多少 MiB、记的那一刻这张卡能借给这个进程的；丢掉时销账。
+/// 账上的一笔：记在哪张卡、多少 MiB、记的那一刻这张卡能借给这个进程的；丢掉时销账（等设备没等成的照旧记着，
+/// [`Self::take_in_the_wait_before_returning`]）。
 pub struct GpuMemoryBooking<'ledger> {
     ledger: &'ledger GpuMemoryLedger,
     card_position: usize,
     mebibytes: u64,
     lease_identifier: u64,
     card_room_mebibytes_when_booked: u64,
+    when_dropped: LeaseWhenTheBookingIsDropped,
+}
+
+/// 借据丢掉之前等设备的结局（卡池的借据丢掉时等这张卡的设备把提交了的活都做完，`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::GpuCardLease`）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DeviceWaitBeforeReturningTheLease {
+    /// 提交了的活都做完了：丢掉的缓冲已经交回驱动，可以销账。
+    TheSubmittedWorkFinished,
+    /// 没等成（超时、wgpu 报错或 panic）：缓冲可能还占着卡，这一笔不销账（文件头「口径」）。
+    TheWaitFailed,
+}
+
+/// 一笔借据丢掉时账上怎么办。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum LeaseWhenTheBookingIsDropped {
+    /// 销账（等完设备了，或这一笔没在卡上建过缓冲）。
+    ReturnedToTheDriverAndOffTheBooks,
+    /// 等设备没等成：照旧记着，设备丢了、账照重起记的那一刻销（`BooksOfACard::leases_kept_after_a_failed_wait`）。
+    KeptOnTheBooksAfterAFailedWait,
 }
 
 impl GpuMemoryBooking<'_> {
@@ -1376,19 +1401,38 @@ impl GpuMemoryBooking<'_> {
         self.ledger
             .is_the_lease_on_the_books(self.card_position, self.lease_identifier)
     }
+
+    /// 丢掉这一笔之前等过设备（卡池的借据丢掉时调，第 10z 批第六块，用户 2026-10-08 定 G2「改用乙」）：等完了，丢掉时照常销账；
+    /// 没等成，这一笔丢掉时不销账、照旧算借着，直到这张卡的设备丢了、账照重起记（文件头「口径」：账不多给，代价是少借这一笔）。
+    pub fn take_in_the_wait_before_returning(&mut self, wait: DeviceWaitBeforeReturningTheLease) {
+        match wait {
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished => {}
+            DeviceWaitBeforeReturningTheLease::TheWaitFailed => {
+                self.ledger.books_locked()[self.card_position]
+                    .leases_kept_after_a_failed_wait
+                    .insert(self.lease_identifier);
+                self.when_dropped = LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait;
+            }
+        }
+    }
 }
 
 impl Drop for GpuMemoryBooking<'_> {
     fn drop(&mut self) {
-        let removed =
-            self.ledger.books_locked()[self.card_position].return_a_lease(self.lease_identifier);
-        // 别处已经在 panic（丢借据是收尾的一部分）时不再断言：两次 panic 会让整个进程中止
-        assert!(
-            removed || std::thread::panicking(),
-            "第 {} 张卡上借据号 {} 不在账上：一笔借据只在它丢掉时销一次账",
-            self.card_position,
-            self.lease_identifier
-        );
+        match self.when_dropped {
+            LeaseWhenTheBookingIsDropped::ReturnedToTheDriverAndOffTheBooks => {
+                let removed = self.ledger.books_locked()[self.card_position]
+                    .return_a_lease(self.lease_identifier);
+                // 别处已经在 panic（丢借据是收尾的一部分）时不再断言：两次 panic 会让整个进程中止
+                assert!(
+                    removed || std::thread::panicking(),
+                    "第 {} 张卡上借据号 {} 不在账上：一笔借据只在它丢掉时销一次账",
+                    self.card_position,
+                    self.lease_identifier
+                );
+            }
+            LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait => {}
+        }
     }
 }
 
@@ -1776,6 +1820,19 @@ impl DeviceConditionOfACard {
     }
 }
 
+/// 在卡上做一段活 `work`、接住它的 panic，并在它交回的那一刻（接住之后、做别的任何事之前）记下设备丢没丢
+/// （[`DeviceConditionOfACard::device_when_the_work_returns`]）：崩溃放量接 Out of Memory 的那一层
+/// （`singlefs_checker_tier::crash_amplification::on_the_card_catching_out_of_memory_ending_together`）照它认这段活的失败
+/// （第 10z 批第五块代码轮判决 K3，攻方 F4：这一步抽到 harness 档，交回那一刻的读法与位置不要卡就测得到）。
+pub fn work_caught_noting_the_device_when_it_returned<Outcome>(
+    condition: &DeviceConditionOfACard,
+    work: impl FnOnce() -> Outcome,
+) -> (std::thread::Result<Outcome>, DeviceWhenTheWorkReturned) {
+    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
+    let device_when_the_work_returned = condition.device_when_the_work_returns();
+    (outcome, device_when_the_work_returned)
+}
+
 /// 同一个设备上一起开的几段活（崩溃放量单元校验和对拍时一块切成的几段，`singlefs_checker_tier::crash_amplification`）：每段收完报到一次，
 /// 认它是不是显存不够之前等其余几段都报到（[`DeviceConditionOfACard::end_the_work_and_explain_its_failure`]）。先把每一段的报到都领出来
 /// （[`Self::join`]），再起那几段：领出来的没报到就丢掉（那一段 panic 了）也算报到，别的段不会等它一辈子。
@@ -2067,6 +2124,26 @@ pub fn judge_blocks_on_one_card<Judged>(
     CardLeftTheBlocks::NothingLeftOrStopped
 }
 
+/// 第 `position` 张卡备好了（或建判器时报了 Out of Memory、不判）之后起哪一张卡的线程（崩溃放量产品路径第 ① 段，卡一张一张备、不同时编内核）：
+/// 后面第一张装了状态的卡（`states_placed` 按这一轮卡的次序，0 是没装），而且队里照此刻的 `stop` 还有领得的块
+/// （[`BlocksForTheCards::any_left_to_take`]：叫停了只剩判完的块不在等的，不起）；否则不起（第 10z 批第五块代码轮判决 K3：
+/// 这一步抽到 harness 档，起下一张卡时看不看叫停不要卡就测得到）。
+#[must_use]
+pub fn next_card_to_start(
+    states_placed: &[u64],
+    position: usize,
+    blocks: &BlocksForTheCards,
+    stop: StopOfTheCards,
+) -> Option<usize> {
+    let next_position =
+        (position + 1..states_placed.len()).find(|candidate| states_placed[*candidate] > 0)?;
+    if blocks.any_left_to_take(stop) {
+        Some(next_position)
+    } else {
+        None
+    }
+}
+
 /// 判完的块按块序写库（判定向量的编号按块序登记，与用了几张卡无关）：块号从 0 起连着，后面的块先到就等着，前面那一块到了一起写；
 /// 写库失败之后到的不再写。几轮卡（[`after_a_round_of_the_cards`]）共用一份：上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写，
 /// 不再判一遍。
@@ -2138,6 +2215,48 @@ pub enum StopOfTheRun {
     NotRequested,
 }
 
+/// 一趟里调用方叫没叫停，见过一次就记住（第 10z 批第五块代码轮判决 K2-2，攻方 F2：叫停是一个文件在不在，外层循环、卡线程、一轮收尾、
+/// 整趟收尾各自现读时文件中途被删，四处读到的不一样，轮末断言 panic、或照叫停收尾而整趟没记成提早收尾）。这一趟里问叫停的每一处都问它；
+/// 头一次读到叫停之后不再读 `read_the_request_now`，之后恒答叫停。几条线程一起问（卡线程与收写库那一条）。
+pub struct StopRequestRememberedInThisRun<'reader> {
+    read_the_request_now: &'reader (dyn Fn() -> bool + Sync),
+    has_seen_the_request: AtomicBool,
+}
+
+impl<'reader> StopRequestRememberedInThisRun<'reader> {
+    /// 这一趟开头：还没见过叫停，要问时照 `read_the_request_now` 现读（崩溃放量产品路径是叫停文件在不在）。
+    #[must_use]
+    pub fn new(read_the_request_now: &'reader (dyn Fn() -> bool + Sync)) -> Self {
+        Self {
+            read_the_request_now,
+            has_seen_the_request: AtomicBool::new(false),
+        }
+    }
+
+    /// 这一刻叫没叫停：这一趟里见过就是叫了；没见过就现读一次，读到叫停记下。
+    #[must_use]
+    pub fn is_requested(&self) -> bool {
+        if self.has_seen_the_request.load(Ordering::SeqCst) {
+            return true;
+        }
+        if (self.read_the_request_now)() {
+            self.has_seen_the_request.store(true, Ordering::SeqCst);
+            return true;
+        }
+        false
+    }
+
+    /// 同 [`Self::is_requested`]，写成一轮收尾交 [`RoundsOfTheCards::after_the_round`] 的那一种。
+    #[must_use]
+    pub fn stop_of_the_run(&self) -> StopOfTheRun {
+        if self.is_requested() {
+            StopOfTheRun::Requested
+        } else {
+            StopOfTheRun::NotRequested
+        }
+    }
+}
+
 /// 一轮卡都收工之后这一趟怎么走（[`after_a_round_of_the_cards`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum AfterARoundOfTheCards {
@@ -2268,6 +2387,16 @@ impl<Block> RoundsOfTheCards<Block> {
         self.first_out_of_memory.as_deref()
     }
 
+    /// 照叫停收尾（[`AfterARoundOfTheCards::StoppedOnRequest`]）时要打的那一行：这一趟有卡报过 Out of Memory，头一句照「本次未跑」那一行的
+    /// 形态（[`gpu_not_run_line`]，`run_name` 是那一格的名字）；没有卡报过是 `None`（第 10z 批第五块代码轮判决 K1-3：叫停加有卡报
+    /// Out of Memory 而照叫停收尾时，这一行不打，显存不够的缘由外面读不到）。
+    #[must_use]
+    pub fn not_run_line_of_a_stop_after_an_out_of_memory(&self, run_name: &str) -> Option<String> {
+        self.first_out_of_memory
+            .as_deref()
+            .map(|reason| gpu_not_run_line(run_name, reason))
+    }
+
     /// 收过工的卡有几张：一张卡在几轮里各收工一次只算一张（第 10z 批代码轮判决 K1-5：`cards_used` 算两遍）。
     #[must_use]
     pub fn cards_used(&self) -> usize {
@@ -2335,9 +2464,8 @@ mod tests {
         attempt_on_a_card_without_room_for_one_state, card_memory_reader_of_one_report_per_reading,
         card_memory_readings_of_one_nvidia_smi_report, decide_scratch_before_booking,
         gpu_not_run_line, pci_bus_identifier_without_the_domain,
-        reason_for_running_out_of_gpu_memory, returned_possibly_still_on_the_card_after_a_reading,
-        BusesWithDevicesStartedOutsideTheBooks, DeviceConditionOfACard, WorkOnTheCardOfThisThread,
-        GPU_MEMORY_SHORT_REASON_PREFIX,
+        reason_for_running_out_of_gpu_memory, BusesWithDevicesStartedOutsideTheBooks,
+        DeviceConditionOfACard, WorkOnTheCardOfThisThread, GPU_MEMORY_SHORT_REASON_PREFIX,
     };
     use super::{
         card_ledger_capacity_mebibytes, card_room_for_this_process,
@@ -2352,6 +2480,10 @@ mod tests {
         ScratchBeforeBooking, ScratchBookingRefusalOutcome, ScratchPlacementDecision,
         ScratchStatesToBook,
     };
+    use super::{
+        next_card_to_start, work_caught_noting_the_device_when_it_returned,
+        DeviceWaitBeforeReturningTheLease, StopRequestRememberedInThisRun,
+    };
     use std::sync::atomic::{AtomicUsize, Ordering};
     use std::sync::{Arc, Mutex};
 
@@ -2503,7 +2635,6 @@ mod tests {
             BookedByThisProcessOnACard {
                 leases_mebibytes,
                 driver_share,
-                returned_possibly_still_on_the_card_mebibytes: 0,
             }
         };
         assert_eq!(
@@ -3270,7 +3401,6 @@ mod tests {
             BookedByThisProcessOnACard {
                 leases_mebibytes: 3_000,
                 driver_share: DriverShareOnTheCard::DeviceStarted,
-                returned_possibly_still_on_the_card_mebibytes: 0,
             },
             DRIVER,
             &|may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER),
@@ -3495,7 +3625,6 @@ mod tests {
         let booked = BookedByThisProcessOnACard {
             leases_mebibytes: 4_000,
             driver_share: DriverShareOnTheCard::DeviceStarted,
-            returned_possibly_still_on_the_card_mebibytes: 0,
         };
         let reading = |occupied_by_this_process| CardMemoryReading {
             free_mebibytes: 15_000,
@@ -3531,55 +3660,61 @@ mod tests {
         );
     }
 
-    /// 第 10x 批代码轮判决 K1-1（正推冲突 B）：第一笔 4 000 建上卡之后丢掉（销了账），卡上还占着；第二笔 4 000 借着还没建。
-    /// 第三笔能借的照「卡上真空着的减第二笔还要建的」算（23 440 − 4 000），不把还没还的 4 000 当成第二笔已经建上的。
-    /// 卡上把第一笔还了之后，那一份照读数压掉，能借的照旧不少算。
+    /// 第 10z 批第六块（G2 乙，用户 2026-10-08 定「改用乙」）：第一笔 4 000 建上卡，第二笔 3 000 借着还没建；第一笔丢掉之前等完设备
+    /// （缓冲交回驱动，读数里这个进程只剩驱动那一份）再销账：之后能借的是卡上空着的减第二笔还要建的、减驱动那一份，
+    /// 销了账的 4 000 不再从空闲里扣（甲在这里多扣驱动那一份 2 560）。
     #[test]
-    fn a_returned_lease_still_on_the_card_is_not_taken_for_a_live_lease_already_built() {
-        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+    fn a_lease_returned_after_the_device_finished_is_no_longer_taken_off_the_free_memory() {
+        const EMPTY: u64 = 30_000;
+        const FIRST_LEASE: u64 = 4_000;
+        const SECOND_LEASE: u64 = 3_000;
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(EMPTY)]);
         let ledger = ledger_reading(&readings);
-        let first = ledger.book_on_the_card(0, 4_000).expect("空着的卡借得到");
+        let mut first = ledger
+            .book_on_the_card(0, FIRST_LEASE)
+            .expect("空着的卡借得到");
         ledger.record_the_device_started_on_the_card(0);
-        let first_built = CardMemoryReading {
-            free_mebibytes: 30_000 - DRIVER - 4_000,
-            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
-            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
-                mebibytes: DRIVER + 4_000,
+        replace_the_reading(
+            &readings,
+            0,
+            CardMemoryReading {
+                free_mebibytes: EMPTY - DRIVER - FIRST_LEASE,
+                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
+                occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
+                    mebibytes: DRIVER + FIRST_LEASE,
+                },
             },
-        };
-        replace_the_reading(&readings, 0, first_built);
-        drop(first);
+        );
         let second = ledger
-            .book_on_the_card(0, 4_000)
+            .book_on_the_card(0, SECOND_LEASE)
             .expect("卡上空着 23 440，借得到");
-        assert_eq!(
-            ledger.card_rooms_mebibytes_now(&[0]),
-            Ok(vec![30_000 - DRIVER - 4_000 - 4_000]),
-            "还没还的 4 000 不算第二笔已经建上的：第二笔还要从空闲里拿 4 000"
-        );
-        assert!(
-            matches!(
-                ledger.book_on_the_card(0, 30_000 - DRIVER - 4_000 - 4_000 + 1),
-                Err(BookingRefusal::NoCardHasRoomNow { .. })
-            ),
-            "多 1 MiB 就借不到"
+        first.take_in_the_wait_before_returning(
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
         );
+        drop(first);
         replace_the_reading(
             &readings,
             0,
             CardMemoryReading {
-                free_mebibytes: 30_000 - DRIVER,
+                free_mebibytes: EMPTY - DRIVER,
                 total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
                 occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                     mebibytes: DRIVER,
                 },
             },
         );
+        let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
         assert_eq!(
-            ledger.card_rooms_mebibytes_now(&[0]),
-            Ok(vec![30_000 - DRIVER - 4_000 - DRIVER]),
-            "卡上还了第一笔：那一份只压到这个进程占着的驱动那一份 2 560 为止，照旧多扣这 2 560（第 10z 批代码轮岔路 G2 甲认的少借），\
-             只扣第二笔还要建的是 24 880"
+            room_now + SECOND_LEASE + DRIVER,
+            EMPTY,
+            "能借的 {room_now} 加第二笔还要建的、加驱动那一份，是卡上空着的：销了账的第一笔不再扣"
+        );
+        assert!(
+            matches!(
+                ledger.book_on_the_card(0, room_now + 1),
+                Err(BookingRefusal::NoCardHasRoomNow { .. })
+            ),
+            "多 1 MiB 就借不到"
         );
         drop(second);
     }
@@ -3812,7 +3947,6 @@ mod tests {
             BookedByThisProcessOnACard {
                 leases_mebibytes: LEASES,
                 driver_share: DriverShareOnTheCard::DeviceStarted,
-                returned_possibly_still_on_the_card_mebibytes: 0,
             },
             DRIVER,
             &|may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER),
@@ -3824,11 +3958,11 @@ mod tests {
         );
     }
 
-    /// 第 10z 批（第 10y 批代码轮判决 K1-1 攻方丙形，数照攻方模型那一行：5090 空卡 32 110，外人占 10 000，借 4 000、设备起好、建满之后丢掉，
-    /// 驱动实占 600，那 4 000 卡上还没还）：销了账的那一份只压到这个进程占着的 4 600 为止，不再减驱动那一份，下一笔能借的是卡上此刻真能再给的
-    /// 15 550（空 17 510 减驱动还要长的 1 960）；减驱动那一份压是 17 510，多给 D − x（第 10z 批代码轮岔路 G2 甲：压到低于这个进程占着的要红）。
+    /// 第 10z 批第六块（G2 乙；数照第 10y 批代码轮判决 K1-1 攻方丙形那一行：5090 空卡 32 110，外人占 10 000，借 4 000、设备起好、建满，
+    /// 驱动实占 600）：那 4 000 丢掉之前等完设备、交回驱动再销账，读数里这个进程只剩驱动实占的 600；下一笔能借的是卡上此刻真能再给的
+    /// 19 550（空 21 510 减驱动还要长的 1 960），不把驱动还没长到的那一截当成能借的（多给 D − x）。
     #[test]
-    fn a_returned_lease_is_pressed_only_to_what_this_process_occupies_not_below_by_a_whole_driver_share(
+    fn a_lease_returned_after_the_device_finished_leaves_room_only_for_what_the_driver_still_grows_to(
     ) {
         const EMPTY: u64 = 32_110;
         const OUTSIDER: u64 = 10_000;
@@ -3854,10 +3988,13 @@ mod tests {
             }),
             DRIVER,
         );
-        let lease = ledger.book_on_the_card(0, LEASE).expect("卡上空着 22 110");
+        let mut lease = ledger.book_on_the_card(0, LEASE).expect("卡上空着 22 110");
         ledger.record_the_device_started_on_the_card(0);
+        lease.take_in_the_wait_before_returning(
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
         drop(lease);
-        let free_now = EMPTY - OUTSIDER - DRIVER_HOLDS_NOW - LEASE;
+        let free_now = EMPTY - OUTSIDER - DRIVER_HOLDS_NOW;
         replace_the_reading(
             &readings,
             0,
@@ -3865,30 +4002,28 @@ mod tests {
                 free_mebibytes: free_now,
                 total_less_reserved_mebibytes: EMPTY,
                 occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
-                    mebibytes: DRIVER_HOLDS_NOW + LEASE,
+                    mebibytes: DRIVER_HOLDS_NOW,
                 },
             },
         );
-        let truly_usable = free_now - (DRIVER - DRIVER_HOLDS_NOW);
+        let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
         assert_eq!(
-            ledger.card_rooms_mebibytes_now(&[0]),
-            Ok(vec![truly_usable]),
-            "那 4 000 卡上还没还，驱动还要长 1 960：能借的是 15 550"
+            room_now + (DRIVER - DRIVER_HOLDS_NOW),
+            free_now,
+            "能借的 {room_now} 加驱动还要长的 1 960 是卡上空着的 21 510：不多给"
         );
         assert_eq!(
             ledger.card_rooms_mebibytes_now(&[0]),
-            Ok(vec![truly_usable]),
+            Ok(vec![room_now]),
             "再读一次照旧"
         );
     }
 
-    /// 第 10z 批（第 10y 批代码轮判决 K2-2 P1，数照正推在真账上造出的那一组 S10）：一笔 10 000 的长借据建满之后，十笔 1 000 的短借据各借、建、丢，
-    /// 丢了之后卡上马上还给驱动（这个进程占的回到驱动那一份加 10 000）。销了账的那一份只压到这个进程占着的 12 560 为止
-    /// （第 10z 批代码轮岔路 G2 甲），十笔 10 000 都压不掉：第 k 笔还了之后能借的是 19 550 − 1 000k，真空闲一直是 19 550，第 10 笔之后
-    /// 要 18 550 放得下却借不到（能借的 9 550），报本次未跑。这是甲认的代价（少借，不多给），钉住它：压得比这个进程占着的更低就会多出来。
+    /// 第 10z 批第六块（G2 乙；数照第 10y 批代码轮判决 K2-2 P1、正推在真账上造出的那一组 S10）：一笔 10 000 的长借据建满之后，十笔 1 000 的短借据
+    /// 各借、建、等完设备再丢（丢了之后读数里这个进程占的回到驱动那一份加 10 000）。销了账的不再记：第 k 笔还了之后能借的照旧是真空闲 19 550，
+    /// 第 10 笔之后要 18 550 照样借得到（甲在这里每笔多扣 1 000，第 10 笔之后只给 9 550、报本次未跑）。
     #[test]
-    fn returned_short_leases_while_a_long_lease_stays_built_each_stay_booked_as_possibly_on_the_card(
-    ) {
+    fn returned_short_leases_while_a_long_lease_stays_built_leave_the_room_as_it_was() {
         const EMPTY: u64 = 32_110;
         const LONG_LEASE: u64 = 10_000;
         const SHORT_LEASE: u64 = 1_000;
@@ -3931,7 +4066,7 @@ mod tests {
             "长借据建满：能借的是真空闲"
         );
         for short_lease_number in 1..=10 {
-            let short_lease = ledger
+            let mut short_lease = ledger
                 .book_on_the_card(0, SHORT_LEASE)
                 .expect("卡上空着 19 550");
             replace_the_reading(
@@ -3942,26 +4077,27 @@ mod tests {
                     DRIVER + LONG_LEASE + SHORT_LEASE,
                 ),
             );
+            short_lease.take_in_the_wait_before_returning(
+                DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+            );
             drop(short_lease);
             replace_the_reading(
                 &readings,
                 0,
                 occupied(free_while_the_long_lease_is_built, DRIVER + LONG_LEASE),
             );
+            let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
             assert_eq!(
-                ledger.card_rooms_mebibytes_now(&[0]),
-                Ok(vec![free_while_the_long_lease_is_built - SHORT_LEASE * short_lease_number]),
-                "第 {short_lease_number} 笔短借据还了、卡上已经还给驱动：这个进程占着的 12 560 压不掉还了的那几笔，每笔多扣 1 000（甲的代价）"
+                room_now + DRIVER + LONG_LEASE,
+                EMPTY,
+                "第 {short_lease_number} 笔短借据等完设备、还了：能借的 {room_now} 加长借据与驱动那一份是空卡，销了账的几笔不再扣"
             );
         }
         let need = free_while_the_long_lease_is_built - SHORT_LEASE;
         assert_eq!(
             booking_kept_on_the_books(&ledger, need),
-            Err(BookingRefusal::NoCardHasRoomNow {
-                need_mebibytes: need,
-                card_rooms_mebibytes: vec![free_while_the_long_lease_is_built - SHORT_LEASE * 10],
-            }),
-            "要 18 550、真空闲 19 550：放得下，账上只给 9 550，借不到（甲的代价，报本次未跑）"
+            Ok(need),
+            "要 18 550、真空闲 19 550：借得到"
         );
         drop(long_lease);
     }
@@ -4014,15 +4150,16 @@ mod tests {
         assert!(ledger.is_this_process_registered_on_the_card(0));
     }
 
-    /// 第 10z 批：同上，第二笔只有 10 000：再起的设备连驱动那一份放得下，起；之后能借的照新设备的驱动那一份还没占上卡算
-    /// （17 440 减驱动那一份减还没建的 10 000），不照丢了的那一份当已在卡上。
+    /// 第 10z 批：同上，第二笔只有 2 000：再起的设备连驱动那一份照此刻的空闲放得下，起；之后能借的照新设备的驱动那一份还没占上卡、
+    /// 这个进程占着的分不出算（第 10z 批第六块 G2 乙：17 440 减驱动那一份、减借着的 12 000，建上了的 10 000 也照没建扣），
+    /// 不照丢了的那一份当已在卡上。
     #[test]
     fn a_lost_device_started_again_books_its_driver_share_as_not_yet_on_the_card() {
         let (ledger, _readings) =
-            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(10_000);
+            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(2_000);
         assert_eq!(
             ledger.card_rooms_mebibytes_now(&[0]),
-            Ok(vec![30_000 - DRIVER - 10_000 - 10_000]),
+            Ok(vec![30_000 - DRIVER - 10_000 - 2_000]),
             "丢之前：驱动那一份在卡上"
         );
         assert_eq!(
@@ -4030,10 +4167,11 @@ mod tests {
             Ok(()),
             "新设备连驱动那一份放得下"
         );
+        let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
         assert_eq!(
-            ledger.card_rooms_mebibytes_now(&[0]),
-            Ok(vec![30_000 - DRIVER - 10_000 - DRIVER - 10_000]),
-            "重起之前：新设备的驱动那一份还要从空闲里扣"
+            room_now + DRIVER + 10_000 + 2_000,
+            30_000 - DRIVER - 10_000,
+            "重起之前：能借的 {room_now} 加新设备的驱动那一份、加借着的两笔，是此刻空着的 17 440"
         );
     }
 
@@ -4240,36 +4378,6 @@ mod tests {
         );
     }
 
-    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X2）：销了账、卡上可能还没还的那一份只在读数分得出这个进程走账的占用时往下压；
-    /// 这个进程另有不走账的设备、或没列这个进程，照旧不压。
-    #[test]
-    fn a_reading_that_cannot_tell_this_process_apart_leaves_the_returned_share_unpressed() {
-        assert_eq!(
-            returned_possibly_still_on_the_card_after_a_reading(
-                4_000,
-                OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 1_000 }
-            ),
-            1_000,
-            "都走这份账：压到占着的那么多"
-        );
-        assert_eq!(
-            returned_possibly_still_on_the_card_after_a_reading(
-                4_000,
-                OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 1_000 }
-            ),
-            4_000,
-            "另有不走账的设备：分不出，不压"
-        );
-        assert_eq!(
-            returned_possibly_still_on_the_card_after_a_reading(
-                4_000,
-                OccupiedByThisProcess::NotListed
-            ),
-            4_000,
-            "没列这个进程：不压"
-        );
-    }
-
     /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X4）：一个状态要的恰好等于空着时能借的，是此刻借不到（报本次未跑），不是放不下；
     /// 多 1 MiB 才是空着时也放不下。
     #[test]
@@ -4292,18 +4400,19 @@ mod tests {
         );
     }
 
-    /// 第 10z 批第三块（第 10z 批代码轮判决 K2-1 第 2 条，数照攻方模型「丁」：5090 空卡 32 110，驱动实占 600）：长借据 10 000、C 8 000、V 3 000 都建满、
-    /// 读过一次；C 销了账、卡上还没还，V 这一趟派完、缓冲缩回驱动（V 的借据还活着）。销了账的那一份只压到这个进程占着的 18 600 为止、照旧 8 000，
-    /// 能借的是卡上此刻真能再给的 8 550（空 13 510 减驱动还要长的 1 960、减 V 还要长回的 3 000）；照活着的借据都建满往下压是 10 950，
-    /// 照早先见到的那一份压是 11 550，照账借满卡上就超了（第 10z 批代码轮岔路 G2 甲：压到低于这个进程占着的要红）。
+    /// 第 10z 批第六块（G2 乙，等失败那一支；数照第 10z 批代码轮判决 K2-1 第 2 条攻方模型「丁」：5090 空卡 32 110，驱动实占 600）：长借据 10 000、
+    /// C 8 000、V 3 000 都建满、读过一次；C 丢掉之前等设备没等成（超时或报错），卡上还没还，V 这一趟派完、缓冲缩回驱动（V 的借据还活着）。
+    /// C 照旧记在账上：能借的是配给卡的额度 32 000 减驱动那一份、减三笔借据的 8 440，不超过卡上此刻真能再给的 8 550（空 13 510 减驱动还要长的
+    /// 1 960、减 V 还要长回的 3 000）；C 照常销了账是 13 510，照账借满卡上就超了。这张卡的设备丢了、账照重起记，C 才销账。
     #[test]
-    fn a_returned_lease_still_on_the_card_is_not_pressed_away_when_a_live_lease_shrinks_back_to_the_driver(
-    ) {
+    fn a_lease_whose_wait_failed_stays_booked_until_the_device_is_taken_as_lost_and_gives_no_more()
+    {
         const EMPTY: u64 = 32_110;
         const DRIVER_HOLDS_NOW: u64 = 600;
         const LONG_LEASE: u64 = 10_000;
         const RETURNED_LEASE: u64 = 8_000;
         const SHRINKING_LEASE: u64 = 3_000;
+        const CONFIGURED_CARD_QUOTA: u64 = 32_000;
         let readings = replaceable_readings(vec![CardMemoryReading {
             free_mebibytes: EMPTY,
             total_less_reserved_mebibytes: EMPTY,
@@ -4316,8 +4425,8 @@ mod tests {
             Box::new(|_position, may_hold_mebibytes| {
                 card_ledger_capacity_mebibytes(
                     None,
-                    Some(32_000),
-                    may_hold_mebibytes.min(32_000),
+                    Some(CONFIGURED_CARD_QUOTA),
+                    may_hold_mebibytes.min(CONFIGURED_CARD_QUOTA),
                     may_hold_mebibytes,
                     DRIVER,
                 )
@@ -4331,9 +4440,10 @@ mod tests {
         };
         let long_lease = ledger.book_on_the_card(0, LONG_LEASE).expect("空卡借得到");
         ledger.record_the_device_started_on_the_card(0);
-        let returned_lease = ledger
+        let mut returned_lease = ledger
             .book_on_the_card(0, RETURNED_LEASE)
             .expect("卡上空着 19 440");
+        let returned_lease_identifier = returned_lease.lease_identifier();
         let shrinking_lease = ledger
             .book_on_the_card(0, SHRINKING_LEASE)
             .expect("卡上空着 11 440");
@@ -4346,6 +4456,8 @@ mod tests {
             ledger.card_rooms_mebibytes_now(&[0]).is_ok(),
             "三笔都建满时读一次"
         );
+        returned_lease
+            .take_in_the_wait_before_returning(DeviceWaitBeforeReturningTheLease::TheWaitFailed);
         drop(returned_lease);
         replace_the_reading(
             &readings,
@@ -4353,11 +4465,24 @@ mod tests {
             occupied(DRIVER_HOLDS_NOW + LONG_LEASE + RETURNED_LEASE),
         );
         let free_now = EMPTY - DRIVER_HOLDS_NOW - LONG_LEASE - RETURNED_LEASE;
-        let truly_lendable = free_now - (DRIVER - DRIVER_HOLDS_NOW) - SHRINKING_LEASE;
+        let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
         assert_eq!(
-            ledger.card_rooms_mebibytes_now(&[0]),
-            Ok(vec![truly_lendable]),
-            "C 卡上还没还、V 缩回驱动：能借的是卡上此刻真能再给的 8 550，不多给"
+            room_now + DRIVER + LONG_LEASE + RETURNED_LEASE + SHRINKING_LEASE,
+            CONFIGURED_CARD_QUOTA,
+            "C 没等成、照旧记着：能借的 {room_now} 加驱动那一份与三笔借据，是配给卡的额度"
+        );
+        assert!(
+            room_now + (DRIVER - DRIVER_HOLDS_NOW) + SHRINKING_LEASE <= free_now,
+            "C 卡上还没还，V 缩回驱动：能借的 {room_now} 加驱动还要长的、V 还要长回的，不超过卡上空着的 13 510（不多给）"
+        );
+        assert!(
+            ledger.is_the_lease_on_the_books(0, returned_lease_identifier),
+            "没等成的那一笔丢掉之后照旧记在账上"
+        );
+        let _restart_judged = ledger.prepare_the_driver_share_before_starting_the_device(0);
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, returned_lease_identifier),
+            "设备丢了、账照重起记：没等成的那一笔这时销账"
         );
         drop(shrinking_lease);
         drop(long_lease);
@@ -4400,17 +4525,124 @@ mod tests {
         assert!(ledger.is_this_process_registered_on_the_card(0));
     }
 
-    /// 第 10z 批第三块（第 10z 批代码轮正推 N10、攻方第三节第 1310 行）：重起时借着的连新设备的驱动那一份恰好放得下（此刻这个进程能占的 24 880，
-    /// 借着的 10 000 + 14 880），起，不拒。
+    /// 第 10z 批第三块（第 10z 批代码轮正推 N10、攻方第三节第 1310 行；第 10z 批第六块 G2 乙改数）：重起时借着的连新设备的驱动那一份照此刻的空闲
+    /// 恰好放得下（空 17 440 = 驱动那一份 2 560 + 借着的 10 000 + 4 880，建上了的那 10 000 照没建算），起，不拒；多 1 MiB 就拒。
     #[test]
     fn a_lost_device_started_again_fits_when_its_driver_share_fits_exactly() {
         let (ledger, _readings) =
-            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(14_880);
+            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(4_880);
+        assert_eq!(DRIVER + 10_000 + 4_880, 30_000 - DRIVER - 10_000);
         assert_eq!(
             ledger.prepare_the_driver_share_before_starting_the_device(0),
             Ok(()),
             "恰好放得下：起"
         );
+        let (ledger_one_mebibyte_short, _readings_one_mebibyte_short) =
+            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(4_881);
+        assert!(
+            ledger_one_mebibyte_short
+                .prepare_the_driver_share_before_starting_the_device(0)
+                .is_err(),
+            "多 1 MiB：不起"
+        );
+    }
+
+    /// 第 10z 批第六块（G2 乙：「销了账、卡上可能还没还的」那一份拿掉之后，丢了的设备留在卡上的不再记成一份）：空 30 000 的卡上借 A 10 000、
+    /// 设备起好、建满（驱动实占 600）；设备丢了，B 5 000 借到、照重起记，新设备起好（也实占 600）。A 还拿着旧设备：旧设备的 600 与 A 的 10 000
+    /// 留在卡上，读数分不出哪一份是新设备的，能借的照分不出算（空 18 800 减驱动那一份、减 A、B，1 240），不超过卡上此刻真能再给的
+    /// 11 840（空 18 800 减新设备驱动还要长的 1 960、减 B 还要建的 5 000）；把旧设备的 600 当新设备的驱动那一份算是 12 440，多给 600。
+    /// A 等完设备还了、旧设备没了，B 是设备丢了那一刻就借着的，照旧照分不出算（卡池里 B 那一笔起的重起，B 还了之前旧设备可能还在池里那一格）；
+    /// 重起之后借 C 3 000、B 也还了，回到照读数分得出算：能借的加 C 与驱动那一份是空卡。
+    #[test]
+    fn a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement() {
+        const EMPTY: u64 = 30_000;
+        const DRIVER_HOLDS_NOW: u64 = 600;
+        const LEASE_ON_THE_LOST_DEVICE: u64 = 10_000;
+        const LEASE_STARTING_THE_REPLACEMENT: u64 = 5_000;
+        const LEASE_AFTER_THE_RESTART: u64 = 3_000;
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(EMPTY)]);
+        let ledger = ledger_reading(&readings);
+        let occupied = |mebibytes: u64| CardMemoryReading {
+            free_mebibytes: EMPTY - mebibytes,
+            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
+            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
+        };
+        let mut lease_on_the_lost_device = ledger
+            .book_on_the_card(0, LEASE_ON_THE_LOST_DEVICE)
+            .expect("空着的卡借得到");
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(())
+        );
+        ledger.record_the_device_started_on_the_card(0);
+        replace_the_reading(
+            &readings,
+            0,
+            occupied(DRIVER_HOLDS_NOW + LEASE_ON_THE_LOST_DEVICE),
+        );
+        let mut lease_starting_the_replacement = ledger
+            .book_on_the_card(0, LEASE_STARTING_THE_REPLACEMENT)
+            .expect("卡上空着 19 400");
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(()),
+            "设备丢了要重起：空 19 400 放得下驱动那一份与两笔"
+        );
+        ledger.record_the_device_started_on_the_card(0);
+        let occupied_with_the_lost_device_left =
+            DRIVER_HOLDS_NOW + LEASE_ON_THE_LOST_DEVICE + DRIVER_HOLDS_NOW;
+        replace_the_reading(&readings, 0, occupied(occupied_with_the_lost_device_left));
+        let free_now = EMPTY - occupied_with_the_lost_device_left;
+        let room_while_the_lost_device_remains =
+            ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
+        assert_eq!(
+            room_while_the_lost_device_remains
+                + DRIVER
+                + LEASE_ON_THE_LOST_DEVICE
+                + LEASE_STARTING_THE_REPLACEMENT,
+            free_now,
+            "旧设备还在卡上：能借的 {room_while_the_lost_device_remains} 加驱动那一份与两笔，是空着的 18 800"
+        );
+        assert!(
+            room_while_the_lost_device_remains
+                + (DRIVER - DRIVER_HOLDS_NOW)
+                + LEASE_STARTING_THE_REPLACEMENT
+                <= free_now,
+            "不超过卡上此刻真能再给的（不多给）"
+        );
+        lease_on_the_lost_device.take_in_the_wait_before_returning(
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+        drop(lease_on_the_lost_device);
+        replace_the_reading(&readings, 0, occupied(DRIVER_HOLDS_NOW));
+        let room_while_the_lease_starting_the_replacement_lives =
+            ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
+        assert_eq!(
+            room_while_the_lease_starting_the_replacement_lives
+                + DRIVER
+                + LEASE_STARTING_THE_REPLACEMENT,
+            EMPTY - DRIVER_HOLDS_NOW,
+            "A 还了，B（设备丢了那一刻已经借着）还在：照旧照分不出算，能借的 \
+             {room_while_the_lease_starting_the_replacement_lives} 加驱动那一份与 B 是空着的 29 400"
+        );
+        let mut lease_after_the_restart = ledger
+            .book_on_the_card(0, LEASE_AFTER_THE_RESTART)
+            .expect("卡上空着 29 400");
+        lease_starting_the_replacement.take_in_the_wait_before_returning(
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+        drop(lease_starting_the_replacement);
+        let room_after_the_leases_of_the_loss_are_returned =
+            ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
+        assert_eq!(
+            room_after_the_leases_of_the_loss_are_returned + LEASE_AFTER_THE_RESTART + DRIVER,
+            EMPTY,
+            "设备丢了那一刻活着的都还了：照读数分得出算，能借的 {room_after_the_leases_of_the_loss_are_returned} \
+             加重起之后借的与驱动那一份是空卡"
+        );
+        lease_after_the_restart.take_in_the_wait_before_returning(
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
     }
 
     /// 第 10z 批第三块（第 10z 批代码轮正推 N1）：同一条线程上活里又起一段活，里面那段丢掉之后，这条线程上记的 Out of Memory 算回外面那段。
@@ -5060,8 +5292,6 @@ mod tests {
         others_mebibytes: u64,
         /// 活着的借据建上卡的合计。
         built_mebibytes: u64,
-        /// 销了账、卡上还没还的。
-        returned_still_on_the_card_mebibytes: u64,
         /// 丢了的设备留在卡上的。
         left_by_a_lost_device_mebibytes: u64,
     }
@@ -5073,7 +5303,6 @@ mod tests {
             } else {
                 0
             }) + self.built_mebibytes
-                + self.returned_still_on_the_card_mebibytes
                 + self.left_by_a_lost_device_mebibytes
         }
 
@@ -5142,12 +5371,12 @@ mod tests {
         card.others_mebibytes = others_mebibytes;
     }
 
-    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M01，`probes/killers.rs`）：长借据建满又还了、卡上还给驱动之后，新借一笔建满，
-    /// 能借的照读数把还了的那一份压到这个进程占着的为止：新借的建上的算进占着的，少的只是驱动那一份（甲的代价）。读数不进账就只剩 16 990。
+    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M01，`probes/killers.rs`；第 10z 批第六块 G2 乙改数）：长借据建满、等完设备还了
+    /// （卡上还给驱动），新借一笔建满：能借的照读数算，新借的建上的只扣一次，还了的那一笔不再扣（甲在这里多扣驱动那一份，21 990）。
     #[test]
-    fn a_reading_after_the_card_took_a_returned_lease_back_lets_a_later_lease_count_once() {
+    fn a_lease_booked_after_another_was_returned_counts_its_built_buffers_once() {
         let (card, ledger) = ledger_on_a_card_under_the_ledger();
-        let first = ledger.book_on_the_card(0, 10_000).expect("空卡借得到");
+        let mut first = ledger.book_on_the_card(0, 10_000).expect("空卡借得到");
         {
             let mut card = card.lock().expect("假卡不 panic");
             card.device_started = true;
@@ -5155,23 +5384,19 @@ mod tests {
             card.built_mebibytes = 10_000;
         }
         ledger.record_the_device_started_on_the_card(0);
+        first.take_in_the_wait_before_returning(
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+        card.lock().expect("假卡不 panic").built_mebibytes = 0;
         drop(first);
-        {
-            let mut card = card.lock().expect("假卡不 panic");
-            card.built_mebibytes = 0;
-            card.returned_still_on_the_card_mebibytes = 10_000;
-        }
-        let _ = room_now(&ledger);
-        card.lock()
-            .expect("假卡不 panic")
-            .returned_still_on_the_card_mebibytes = 0;
         let _ = room_now(&ledger);
         let _second = ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
         card.lock().expect("假卡不 panic").built_mebibytes = 5_000;
+        let room_after_the_second_is_built = room_now(&ledger);
         assert_eq!(
-            room_now(&ledger),
-            AN_EMPTY_RTX_5090_HAS - DRIVER - 5_000 - DRIVER,
-            "第二笔建满：能借的 21 990（读数不压就是 16 990）"
+            room_after_the_second_is_built + 5_000 + DRIVER,
+            AN_EMPTY_RTX_5090_HAS,
+            "第二笔建满：能借的 {room_after_the_second_is_built} 加第二笔与驱动那一份是空卡"
         );
     }
 
@@ -5225,10 +5450,15 @@ mod tests {
         );
     }
 
-    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M10）：设备丢了要重起，借着的 10 000 连新设备的驱动那一份恰好放得下就起，差 1 MiB 就拒。
+    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M10；第 10z 批第六块 G2 乙改数）：设备丢了要重起，借着的 10 000 连新设备的驱动那一份
+    /// 照此刻的空闲恰好放得下就起，差 1 MiB 就拒（丢了的设备与建上的 10 000 留在卡上、不从读数里分出来：空 32 110 − 6 990 − 12 560 = 12 560）。
     #[test]
     fn a_restart_is_allowed_when_it_fits_exactly_and_refused_one_mebibyte_short() {
-        for (others_mebibytes, fits) in [(16_990u64, true), (16_991, false)] {
+        assert_eq!(
+            AN_EMPTY_RTX_5090_HAS - 6_990 - (DRIVER + 10_000),
+            DRIVER + 10_000
+        );
+        for (others_mebibytes, fits) in [(6_990u64, true), (6_991, false)] {
             let (card, ledger) = ledger_on_a_card_under_the_ledger();
             let _lease = ten_thousand_built_on_a_started_device(&card, &ledger);
             the_device_is_lost_with_others_taking(&card, others_mebibytes);
@@ -5668,4 +5898,355 @@ mod tests {
         assert_eq!(judged.judged_times, vec![1, 1, 0, 0]);
         assert_eq!(judged.blocks_left_for_the_next_run, vec![2, 3]);
     }
+
+    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K2-2，攻方 F2）：一趟里见过一次叫停就记住：叫停文件头一次读到之后被删，之后每一处问都答叫停，
+    /// 也不再读文件；没见过叫停时每问一次现读一次。
+    #[test]
+    fn a_stop_request_seen_once_stays_requested_after_the_request_is_gone() {
+        let reads = AtomicUsize::new(0);
+        let request_is_there_on_the_second_read = || reads.fetch_add(1, Ordering::SeqCst) == 1;
+        let stop = StopRequestRememberedInThisRun::new(&request_is_there_on_the_second_read);
+        assert!(!stop.is_requested(), "头一次读：叫停文件还没有");
+        assert!(stop.is_requested(), "第二次读：叫停文件在");
+        assert_eq!(
+            stop.stop_of_the_run(),
+            StopOfTheRun::Requested,
+            "叫停文件已经删了：见过一次就记住"
+        );
+        assert!(stop.is_requested(), "再问照旧");
+        assert_eq!(reads.load(Ordering::SeqCst), 2, "见过叫停之后不再读文件");
+    }
+
+    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K1-3）：叫停、这一轮有卡报了 Out of Memory、判完的块都入了库而照叫停收尾：头一句 Out of Memory
+    /// 照「本次未跑」那一行的形态打得出来；这一趟没有卡报过的，照叫停收尾不打这一行。
+    #[test]
+    fn a_stop_after_an_out_of_memory_prints_the_first_out_of_memory_in_the_not_run_line() {
+        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
+        let blocks = rounds.take_the_blocks_of_the_round();
+        assert_eq!(blocks.take_a_block(StopOfTheCards::NotRequested), Some(0));
+        rounds.stored_in_order().arrive(0, 0, &mut |_block| Ok(()));
+        assert_eq!(
+            rounds.after_the_round(
+                &blocks,
+                &[0, 1],
+                [1],
+                &[(0, "序号 0 的卡判一块时报了 Out of Memory".to_string())],
+                StopOfTheRun::Requested
+            ),
+            AfterARoundOfTheCards::StoppedOnRequest
+        );
+        assert_eq!(
+            rounds
+                .not_run_line_of_a_stop_after_an_out_of_memory(
+                    "crash_amplification::run_from_environment"
+                )
+                .as_deref(),
+            Some(
+                "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=序号 0 的卡判一块时报了 Out of Memory"
+            )
+        );
+        let mut rounds_without_out_of_memory: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
+        let blocks_without_out_of_memory =
+            rounds_without_out_of_memory.take_the_blocks_of_the_round();
+        assert_eq!(
+            rounds_without_out_of_memory.after_the_round(
+                &blocks_without_out_of_memory,
+                &[0],
+                [0],
+                &[],
+                StopOfTheRun::Requested
+            ),
+            AfterARoundOfTheCards::StoppedOnRequest
+        );
+        assert_eq!(
+            rounds_without_out_of_memory.not_run_line_of_a_stop_after_an_out_of_memory(
+                "crash_amplification::run_from_environment"
+            ),
+            None,
+            "没有卡报过 Out of Memory：照叫停收尾不打本次未跑那一行"
+        );
+    }
+
+    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K3，攻方 F4）：一段活交回的那一刻记设备丢没丢：活里设备丢了，交回那一刻记成丢了；
+    /// 活 panic 时设备没丢、之后才丢，交回那一刻记成没丢（之后丢的不把这段活真正的 panic 说成显存不够）。
+    #[test]
+    fn a_work_notes_the_device_at_the_moment_it_returns_not_before_and_not_after() {
+        let lost_during_the_work = DeviceConditionOfACard::default();
+        let (outcome, device_when_the_work_returned) =
+            work_caught_noting_the_device_when_it_returned(&lost_during_the_work, || {
+                lost_during_the_work.record_the_device_lost(
+                    "Unknown：Out of memory".to_string(),
+                    CauseOfTheDeviceLoss::GpuMemoryShort,
+                );
+                7
+            });
+        assert!(matches!(outcome, Ok(7)));
+        assert_eq!(
+            device_when_the_work_returned,
+            DeviceWhenTheWorkReturned::Lost,
+            "活里设备丢了：交回那一刻记成丢了"
+        );
+        let lost_after_the_work = DeviceConditionOfACard::default();
+        let (panicked_outcome, device_when_the_panicked_work_returned) =
+            work_caught_noting_the_device_when_it_returned(&lost_after_the_work, || -> u32 {
+                panic!("这段活自己真 panic 了")
+            });
+        lost_after_the_work.record_the_device_lost(
+            "Unknown：Out of memory".to_string(),
+            CauseOfTheDeviceLoss::GpuMemoryShort,
+        );
+        assert!(panicked_outcome.is_err(), "panic 接住了");
+        assert_eq!(
+            device_when_the_panicked_work_returned,
+            DeviceWhenTheWorkReturned::NotLost,
+            "活 panic 时设备没丢、之后才丢：交回那一刻记成没丢"
+        );
+    }
+
+    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K3：起下一张卡的看叫停没有用例）：后面装了状态的那一张，队里照此刻的叫停还有领得的块才起；
+    /// 叫停了、判完的块都不在等，不起；判红了不起；后面没有装了状态的卡，不起。
+    #[test]
+    fn the_next_card_starts_only_when_it_holds_states_and_a_block_is_left_to_take_under_the_stop() {
+        let blocks = BlocksForTheCards::new([0usize, 1, 2]);
+        assert_eq!(
+            next_card_to_start(&[1, 0, 1], 0, &blocks, StopOfTheCards::NotRequested),
+            Some(2),
+            "跳过没装状态的那一张"
+        );
+        assert_eq!(
+            next_card_to_start(&[1, 0, 1], 0, &blocks, StopOfTheCards::RequestedByTheCaller),
+            None,
+            "叫停了、还没有判完的块：没有判完的块在等的，不起"
+        );
+        assert_eq!(
+            next_card_to_start(&[1, 0, 1], 0, &blocks, StopOfTheCards::TheRunFailed),
+            None,
+            "判红了：不起"
+        );
+        assert_eq!(
+            next_card_to_start(&[1, 0, 1], 2, &blocks, StopOfTheCards::NotRequested),
+            None,
+            "后面没有卡了"
+        );
+        assert_eq!(
+            next_card_to_start(&[1, 0], 0, &blocks, StopOfTheCards::NotRequested),
+            None,
+            "后面的卡没装状态"
+        );
+    }
+
+    /// 接住一段会 panic 的活，交它 panic 了没有。
+    fn panics_when_run(work: impl FnOnce()) -> bool {
+        std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).is_err()
+    }
+
+    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K3，正推 `probes/killers_new.rs` M05）：三张卡各领一块 5、6、7；7 先判完、5 后判完、
+    /// 6 最后报 Out of Memory 交回。叫停之后下一张卡领得到 6：判完的块里最大的是 7（在等 6），不是最后判完的 5。
+    #[test]
+    fn the_largest_judged_block_not_the_last_judged_lets_a_stopped_card_take_the_block_it_waits_for(
+    ) {
+        let blocks = BlocksForTheCards::new([5usize, 6, 7]);
+        let (block_five_taken_sender, block_five_taken) = std::sync::mpsc::channel::<()>();
+        let (block_six_taken_sender, block_six_taken) = std::sync::mpsc::channel::<()>();
+        let (block_seven_taken_sender, block_seven_taken) = std::sync::mpsc::channel::<()>();
+        let (finish_block_five_sender, finish_block_five) = std::sync::mpsc::channel::<()>();
+        let (finish_block_six_sender, finish_block_six) = std::sync::mpsc::channel::<()>();
+        let (finish_block_seven_sender, finish_block_seven) = std::sync::mpsc::channel::<()>();
+        let (block_five_stored_sender, block_five_stored) = std::sync::mpsc::channel::<()>();
+        let (block_seven_stored_sender, block_seven_stored) = std::sync::mpsc::channel::<()>();
+        let enough_for_any_step = std::time::Duration::from_secs(20);
+        // 每张卡只领一块：头一次照常领，之后判红（不再领）
+        let one_block_then_stop = || {
+            let is_the_first_ask = std::cell::Cell::new(true);
+            move || {
+                if is_the_first_ask.replace(false) {
+                    StopOfTheCards::NotRequested
+                } else {
+                    StopOfTheCards::TheRunFailed
+                }
+            }
+        };
+        let card_left_with_block_six = std::thread::scope(|scope| {
+            let blocks = &blocks;
+            scope.spawn(move || {
+                let stop_of_the_cards = one_block_then_stop();
+                let _left = judge_blocks_on_one_card(
+                    blocks,
+                    &stop_of_the_cards,
+                    &mut |block| {
+                        assert_eq!(block, 5);
+                        block_five_taken_sender.send(()).expect("主线程等着");
+                        finish_block_five.recv().expect("主线程会放行");
+                        BlockJudgedOnACard::Judged(block)
+                    },
+                    &mut |_block, _judged: usize| {
+                        block_five_stored_sender.send(()).expect("主线程等着");
+                        BlockHandedToTheStore::Handed
+                    },
+                );
+            });
+            block_five_taken
+                .recv_timeout(enough_for_any_step)
+                .expect("块 5 领走了");
+            let card_with_block_six = scope.spawn(move || {
+                let stop_of_the_cards = one_block_then_stop();
+                judge_blocks_on_one_card(
+                    blocks,
+                    &stop_of_the_cards,
+                    &mut |block| {
+                        assert_eq!(block, 6);
+                        block_six_taken_sender.send(()).expect("主线程等着");
+                        finish_block_six.recv().expect("主线程会放行");
+                        BlockJudgedOnACard::<usize>::OutOfMemory {
+                            reason: "块 6 上报了 Out of Memory".to_string(),
+                        }
+                    },
+                    &mut |_block, _judged: usize| BlockHandedToTheStore::Handed,
+                )
+            });
+            block_six_taken
+                .recv_timeout(enough_for_any_step)
+                .expect("块 6 领走了");
+            scope.spawn(move || {
+                let stop_of_the_cards = one_block_then_stop();
+                let _left = judge_blocks_on_one_card(
+                    blocks,
+                    &stop_of_the_cards,
+                    &mut |block| {
+                        assert_eq!(block, 7);
+                        block_seven_taken_sender.send(()).expect("主线程等着");
+                        finish_block_seven.recv().expect("主线程会放行");
+                        BlockJudgedOnACard::Judged(block)
+                    },
+                    &mut |_block, _judged: usize| {
+                        block_seven_stored_sender.send(()).expect("主线程等着");
+                        BlockHandedToTheStore::Handed
+                    },
+                );
+            });
+            block_seven_taken
+                .recv_timeout(enough_for_any_step)
+                .expect("块 7 领走了");
+            finish_block_seven_sender.send(()).expect("块 7 那张卡等着");
+            block_seven_stored
+                .recv_timeout(enough_for_any_step)
+                .expect("块 7 判完");
+            finish_block_five_sender.send(()).expect("块 5 那张卡等着");
+            block_five_stored
+                .recv_timeout(enough_for_any_step)
+                .expect("块 5 判完");
+            finish_block_six_sender.send(()).expect("块 6 那张卡等着");
+            card_with_block_six.join().expect("块 6 那张卡不 panic")
+        });
+        assert!(matches!(
+            card_left_with_block_six,
+            CardLeftTheBlocks::OutOfMemory { .. }
+        ));
+        assert_eq!(
+            blocks.take_a_block(StopOfTheCards::RequestedByTheCaller),
+            Some(6),
+            "7 判完了在等 6：叫停之后下一张卡领得到 6"
+        );
+    }
+
+    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M10）：叫停、一张卡报了 Out of Memory、判完的块只有 1 块在等：还有别的卡就再起一轮，
+    /// 没有别的卡是本次未跑，都不是照叫停收尾；一块都不在等才照叫停收尾。
+    #[test]
+    fn a_stop_with_one_judged_block_waiting_needs_another_round_or_is_not_run() {
+        assert_eq!(
+            after_a_round_of_the_cards(3, 1, 1, StopOfTheRun::Requested, 1),
+            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor
+        );
+        assert_eq!(
+            after_a_round_of_the_cards(3, 1, 1, StopOfTheRun::Requested, 0),
+            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
+        );
+        assert_eq!(
+            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::Requested, 1),
+            AfterARoundOfTheCards::StoppedOnRequest
+        );
+    }
+
+    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M13）：没有卡报 Out of Memory 而判完的块还有在等前面那一块的：轮末断言 panic（叫停了也一样）。
+    #[test]
+    fn a_round_without_out_of_memory_leaving_a_judged_block_waiting_panics() {
+        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
+        let blocks = rounds.take_the_blocks_of_the_round();
+        assert_eq!(blocks.take_a_block(StopOfTheCards::NotRequested), Some(0));
+        rounds.stored_in_order().arrive(2, 2, &mut |_block| Ok(()));
+        assert_eq!(rounds.blocks_waiting(), 1);
+        assert!(panics_when_run(|| {
+            let _after =
+                rounds.after_the_round(&blocks, &[0], [0u32], &[], StopOfTheRun::Requested);
+        }));
+    }
+
+    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M14）：没人叫停、没有卡报 Out of Memory，队里却还有没领走的块：轮末断言 panic；叫停了就可以留。
+    #[test]
+    fn a_round_without_a_stop_leaving_a_block_untaken_panics() {
+        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
+        let blocks = rounds.take_the_blocks_of_the_round();
+        assert!(panics_when_run(|| {
+            let _after =
+                rounds.after_the_round(&blocks, &[0], [0u32], &[], StopOfTheRun::NotRequested);
+        }));
+        let mut stopped_rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
+        let stopped_blocks = stopped_rounds.take_the_blocks_of_the_round();
+        assert_eq!(
+            stopped_rounds.after_the_round(
+                &stopped_blocks,
+                &[0],
+                [0u32],
+                &[],
+                StopOfTheRun::Requested
+            ),
+            AfterARoundOfTheCards::StoppedOnRequest
+        );
+    }
+
+    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M17）：这一趟头一句 Out of Memory 是第一张报的那一句，后面报的不盖掉它。
+    #[test]
+    fn the_first_out_of_memory_of_the_run_stays_the_first_reported() {
+        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(2);
+        let blocks = rounds.take_the_blocks_of_the_round();
+        let _after = rounds.after_the_round(
+            &blocks,
+            &[0, 1],
+            [],
+            &[(0, "第一句".to_string()), (1, "第二句".to_string())],
+            StopOfTheRun::NotRequested,
+        );
+        assert_eq!(rounds.first_out_of_memory(), Some("第一句"));
+    }
+
+    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M22）：一块交回两次要 panic。
+    #[test]
+    fn giving_a_block_back_twice_panics() {
+        let blocks = BlocksForTheCards::new([0usize, 1]);
+        let first = blocks
+            .take_a_block(StopOfTheCards::NotRequested)
+            .expect("队里有块");
+        blocks.give_back(first);
+        assert!(panics_when_run(|| blocks.give_back(first)));
+    }
+
+    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M23）：同一段活上记了两次 Out of Memory，认出来的是最后那一次。
+    #[test]
+    fn the_last_out_of_memory_recorded_on_a_work_is_the_one_told() {
+        let condition = DeviceConditionOfACard::default();
+        let work = WorkOnTheCardOfThisThread::begin();
+        let reports_before = condition.out_of_memory_reports();
+        condition.record_an_out_of_memory("先报的".to_string());
+        condition.record_an_out_of_memory("后报的".to_string());
+        assert_eq!(
+            condition
+                .out_of_memory_explaining_a_failure_of_the_work(
+                    work.identifier(),
+                    reports_before,
+                    DeviceWhenTheWorkReturned::NotLost
+                )
+                .as_deref(),
+            Some("后报的")
+        );
+    }
 }
diff --git a/crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs b/crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs
new file mode 100644
index 0000000..6ffa0bc
--- /dev/null
+++ b/crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs
@@ -0,0 +1,582 @@
+//! 崩溃放量产品路径第 ① 段几轮卡的叫停（第 10z 批第六块，第 10z 批第五块代码轮判决 K2-2、攻方 F2）：叫停是一个文件在不在，
+//! 文件出现之后中途被删，外层循环、卡线程、一轮收尾、整趟收尾都问这一趟的同一个 [`StopRequestRememberedInThisRun`]，见过一次叫停就记住。
+//! 攻方模型 `research/prompts/m3-batch10z-e-code-r1-opus-model/attack_10ze_rounds.rs` 的 `stop-file-removed` 一臂改到这个接口：checker 档那一圈
+//! （`singlefs_checker_tier::crash_amplification` 的外层循环、`judge_on_one_card_and_start_the_next`、`start_the_next_card_with_states`、
+//! `CardsAtWork::stop_of_the_cards`、整趟收尾那一句）照次序抄成不碰卡的一份，harness 档的 `BlocksForTheCards`、`judge_blocks_on_one_card`、
+//! `next_card_to_start`、`RoundsOfTheCards` 原样调用。线程先后由按种子取的调度器定：卡线程每一步（建判器、问叫停、判一块、交写库）之前停下，
+//! 调度器在停着的里挑一条放行；同一种子同一段历史。
+use singlefs_harness::gpu_memory_ledger::{
+    judge_blocks_on_one_card, next_card_to_start, AfterARoundOfTheCards, BlockHandedToTheStore,
+    BlockJudgedOnACard, BlocksForTheCards, CardLeftTheBlocks, RoundsOfTheCards, StopOfTheCards,
+    StopRequestRememberedInThisRun,
+};
+use std::collections::{BTreeMap, BTreeSet};
+use std::panic::{catch_unwind, AssertUnwindSafe};
+use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
+use std::sync::mpsc::{channel, Sender};
+use std::sync::{Condvar, Mutex};
+
+/// 这一臂跑几段脚本、每段几个种子（几千段：攻方 200 000 段里改前两样结局合计 4 168 段，按比例这里改前各有几十段）。
+const SCRIPTS: usize = 1_500;
+const SEEDS_OF_EACH_SCRIPT: u64 = 4;
+/// 攻方 `stop-file-removed` 一臂的头一个种子。
+const FIRST_SEED: u64 = 0x10E5_0002;
+
+/// SplitMix64：按种子定的伪随机数。
+struct SplitMixRandom {
+    state: u64,
+}
+
+impl SplitMixRandom {
+    fn next_value(&mut self) -> u64 {
+        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
+        let mut mixed = self.state;
+        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
+        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
+        mixed ^ (mixed >> 31)
+    }
+
+    fn below(&mut self, bound: u64) -> u64 {
+        self.next_value() % bound
+    }
+}
+
+/// 一张假卡这一趟怎么走。
+#[derive(Clone, Debug)]
+struct CardScript {
+    index: u32,
+    /// 第 r 轮装得下几个状态（0 是这一轮不上卡）；超出的轮取最后一个。
+    states_placed_in_each_round: Vec<u64>,
+    /// 第几轮建判器时报 Out of Memory。
+    out_of_memory_building_the_judge_in_round: Option<usize>,
+    /// 这张卡判的第几块（几轮合计，从 1 数）报 Out of Memory。
+    out_of_memory_at_its_block: Option<usize>,
+}
+
+impl CardScript {
+    fn states_placed(&self, round: usize) -> u64 {
+        *self
+            .states_placed_in_each_round
+            .get(round)
+            .or(self.states_placed_in_each_round.last())
+            .unwrap_or(&1)
+    }
+}
+
+/// 一趟的脚本：几块、几张卡、叫停文件在第几个事件出现、第几个事件被删。
+#[derive(Clone, Debug)]
+struct RunScript {
+    blocks: usize,
+    cards: Vec<CardScript>,
+    stop_file_appears_at_event: Option<usize>,
+    stop_file_removed_at_event: Option<usize>,
+}
+
+/// 叫停文件在第 `event` 个事件那一刻在不在。
+fn stop_file_is_there(script: &RunScript, event: usize) -> bool {
+    script
+        .stop_file_appears_at_event
+        .is_some_and(|appears| event >= appears)
+        && !script
+            .stop_file_removed_at_event
+            .is_some_and(|removed| event >= removed)
+}
+
+struct SchedulerState {
+    threads_alive: usize,
+    threads_parked: BTreeSet<usize>,
+    thread_granted: Option<usize>,
+}
+
+/// 按种子放行的调度器：每条卡线程每一步之前停下，主线程在停着的里挑一条放行。
+struct Scheduler {
+    state: Mutex<SchedulerState>,
+    changed: Condvar,
+}
+
+impl Scheduler {
+    fn new() -> Self {
+        Self {
+            state: Mutex::new(SchedulerState {
+                threads_alive: 0,
+                threads_parked: BTreeSet::new(),
+                thread_granted: None,
+            }),
+            changed: Condvar::new(),
+        }
+    }
+
+    fn locked(&self) -> std::sync::MutexGuard<'_, SchedulerState> {
+        self.state
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+    }
+
+    fn register_a_thread(&self) {
+        self.locked().threads_alive += 1;
+    }
+
+    fn wait_for_the_grant(&self, thread_number: usize) {
+        let mut state = self.locked();
+        state.threads_parked.insert(thread_number);
+        self.changed.notify_all();
+        while state.thread_granted != Some(thread_number) {
+            state = self
+                .changed
+                .wait(state)
+                .unwrap_or_else(std::sync::PoisonError::into_inner);
+        }
+        state.thread_granted = None;
+        state.threads_parked.remove(&thread_number);
+    }
+
+    fn a_thread_exits(&self) {
+        let mut state = self.locked();
+        state.threads_alive -= 1;
+        self.changed.notify_all();
+    }
+}
+
+/// 卡线程收工（正常收或 panic 了）就不再算活着。
+struct ThreadExitsTheScheduler<'scheduler>(&'scheduler Scheduler);
+
+impl Drop for ThreadExitsTheScheduler<'_> {
+    fn drop(&mut self) {
+        self.0.a_thread_exits();
+    }
+}
+
+enum CardMessage {
+    Block(usize),
+    Finished(u32),
+    OutOfMemory(u32, String),
+    CardPanicked(String),
+}
+
+/// 同 `CardsAtWork`：这一轮上卡的卡、一队块、卡自己置的停与这一趟的叫停。
+struct CardsAtWork<'work> {
+    round: usize,
+    cards: Vec<&'work CardScript>,
+    blocks: BlocksForTheCards,
+    the_run_failed: AtomicBool,
+    stop_request: &'work StopRequestRememberedInThisRun<'work>,
+    scheduler: Scheduler,
+    next_thread_number: AtomicUsize,
+    events: &'work AtomicUsize,
+    sender: Sender<CardMessage>,
+    judged_times: &'work Mutex<BTreeMap<usize, usize>>,
+    blocks_judged_by_each_card: &'work Mutex<BTreeMap<u32, usize>>,
+}
+
+impl CardsAtWork<'_> {
+    fn step(&self, thread_number: usize) {
+        self.scheduler.wait_for_the_grant(thread_number);
+        self.events.fetch_add(1, Ordering::SeqCst);
+    }
+
+    /// 同 `CardsAtWork::stop_of_the_cards`：判红了一块都不再领，叫停了只领判完的块在等的。
+    fn stop_of_the_cards(&self) -> StopOfTheCards {
+        if self.the_run_failed.load(Ordering::SeqCst) {
+            StopOfTheCards::TheRunFailed
+        } else if self.stop_request.is_requested() {
+            StopOfTheCards::RequestedByTheCaller
+        } else {
+            StopOfTheCards::NotRequested
+        }
+    }
+}
+
+/// 同 `start_the_next_card_with_states`：照 [`next_card_to_start`] 起下一张。
+fn start_the_next_card<'scope, 'work>(
+    scope: &'scope std::thread::Scope<'scope, 'work>,
+    work: &'work CardsAtWork<'work>,
+    position: usize,
+) {
+    let states_placed: Vec<u64> = work
+        .cards
+        .iter()
+        .map(|card| card.states_placed(work.round))
+        .collect();
+    if let Some(next_position) = next_card_to_start(
+        &states_placed,
+        position,
+        &work.blocks,
+        work.stop_of_the_cards(),
+    ) {
+        let thread_number = work.next_thread_number.fetch_add(1, Ordering::SeqCst);
+        work.scheduler.register_a_thread();
+        scope.spawn(move || card_thread(scope, work, next_position, thread_number));
+    }
+}
+
+fn card_thread<'scope, 'work>(
+    scope: &'scope std::thread::Scope<'scope, 'work>,
+    work: &'work CardsAtWork<'work>,
+    position: usize,
+    thread_number: usize,
+) {
+    let _exits = ThreadExitsTheScheduler(&work.scheduler);
+    let body = catch_unwind(AssertUnwindSafe(|| {
+        judge_on_one_card_and_start_the_next(scope, work, position, thread_number);
+    }));
+    if let Err(panic) = body {
+        let text = panic
+            .downcast_ref::<String>()
+            .cloned()
+            .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_string()))
+            .unwrap_or_default();
+        let _sent = work.sender.send(CardMessage::CardPanicked(text));
+    }
+}
+
+/// 同 `judge_on_one_card_and_start_the_next` 的次序：建判器（可报 Out of Memory）、起下一张、判块的那一圈、收工报。
+fn judge_on_one_card_and_start_the_next<'scope, 'work>(
+    scope: &'scope std::thread::Scope<'scope, 'work>,
+    work: &'work CardsAtWork<'work>,
+    position: usize,
+    thread_number: usize,
+) {
+    let card = work.cards[position];
+    work.step(thread_number);
+    if card.out_of_memory_building_the_judge_in_round == Some(work.round) {
+        let _sent = work.sender.send(CardMessage::OutOfMemory(
+            card.index,
+            format!("卡 {} 建判器时", card.index),
+        ));
+        start_the_next_card(scope, work, position);
+        return;
+    }
+    start_the_next_card(scope, work, position);
+    let card_left = judge_blocks_on_one_card(
+        &work.blocks,
+        &|| {
+            work.step(thread_number);
+            work.stop_of_the_cards()
+        },
+        &mut |pending_index| {
+            work.step(thread_number);
+            let mut blocks_judged = work
+                .blocks_judged_by_each_card
+                .lock()
+                .expect("假卡不 panic 在锁里");
+            let blocks_judged_by_this_card = blocks_judged.entry(card.index).or_insert(0);
+            *blocks_judged_by_this_card += 1;
+            if card.out_of_memory_at_its_block == Some(*blocks_judged_by_this_card) {
+                return BlockJudgedOnACard::OutOfMemory {
+                    reason: format!("卡 {} 判块 {pending_index} 时", card.index),
+                };
+            }
+            *work
+                .judged_times
+                .lock()
+                .expect("假卡不 panic 在锁里")
+                .entry(pending_index)
+                .or_insert(0) += 1;
+            BlockJudgedOnACard::Judged(pending_index)
+        },
+        &mut |pending_index, _judged| {
+            work.step(thread_number);
+            match work.sender.send(CardMessage::Block(pending_index)) {
+                Ok(()) => BlockHandedToTheStore::Handed,
+                Err(_) => BlockHandedToTheStore::TheStoreIsGone,
+            }
+        },
+    );
+    match card_left {
+        CardLeftTheBlocks::NothingLeftOrStopped | CardLeftTheBlocks::TheStoreIsGone => {
+            let _sent = work.sender.send(CardMessage::Finished(card.index));
+        }
+        CardLeftTheBlocks::OutOfMemory { reason } => {
+            let _sent = work
+                .sender
+                .send(CardMessage::OutOfMemory(card.index, reason));
+        }
+        CardLeftTheBlocks::Refused { .. } => {
+            work.the_run_failed.store(true, Ordering::SeqCst);
+        }
+    }
+}
+
+/// 一趟的结局。
+#[derive(Clone, Debug)]
+enum EndOfTheRun {
+    EveryBlockStored,
+    StoppedOnRequest,
+    NotRun,
+    /// 轮末断言 panic（这一趟判红）。
+    AssertPanicked(String),
+    CardPanicked(String),
+    NoCardHoldsAState,
+    TooManyRounds,
+}
+
+#[derive(Clone, Debug)]
+struct RunResult {
+    end: EndOfTheRun,
+    /// 整趟收尾时 `stopped_early`：显存不够那一支恒真，别的照这一趟的叫停（实现第 5591 行那一句，改到 [`StopRequestRememberedInThisRun`]）。
+    stopped_early: bool,
+    /// 整趟收尾那一刻叫停文件已经不在、这一趟却见过叫停（这一臂要钉的窗口真走到了）。
+    stop_seen_and_the_file_gone_at_the_end: bool,
+}
+
+/// 同外层循环：每轮挑没报过的卡、上卡的是装得下的那几张，第 0 张起、一张带一张；收工之后照 `RoundsOfTheCards::after_the_round` 走。
+fn run_once(script: &RunScript, seed: u64) -> RunResult {
+    let mut random = SplitMixRandom { state: seed };
+    let events = AtomicUsize::new(0);
+    let read_the_stop_file_now = || stop_file_is_there(script, events.load(Ordering::SeqCst));
+    let stop_request = StopRequestRememberedInThisRun::new(&read_the_stop_file_now);
+    let judged_times = Mutex::new(BTreeMap::new());
+    let blocks_judged_by_each_card = Mutex::new(BTreeMap::new());
+    let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(script.blocks);
+    let finish = |end: EndOfTheRun, gpu_memory_was_short: bool| {
+        events.fetch_add(1, Ordering::SeqCst);
+        let stopped_early = gpu_memory_was_short || stop_request.is_requested();
+        RunResult {
+            end,
+            stopped_early,
+            stop_seen_and_the_file_gone_at_the_end: stop_request.is_requested()
+                && !read_the_stop_file_now(),
+        }
+    };
+    let mut round = 0usize;
+    loop {
+        if round > script.cards.len() + 1 {
+            return finish(EndOfTheRun::TooManyRounds, false);
+        }
+        let cards_of_the_round: Vec<&CardScript> = script
+            .cards
+            .iter()
+            .filter(|card| !rounds.is_short_of_memory(card.index))
+            .collect();
+        let cards_placed: Vec<&CardScript> = cards_of_the_round
+            .iter()
+            .copied()
+            .filter(|card| card.states_placed(round) > 0)
+            .collect();
+        if cards_placed.is_empty() {
+            if rounds.first_out_of_memory().is_some() {
+                return finish(EndOfTheRun::NotRun, true);
+            }
+            return finish(EndOfTheRun::NoCardHoldsAState, false);
+        }
+        let (sender, receiver) = channel::<CardMessage>();
+        let work = CardsAtWork {
+            round,
+            cards: cards_placed,
+            blocks: rounds.take_the_blocks_of_the_round(),
+            the_run_failed: AtomicBool::new(false),
+            stop_request: &stop_request,
+            scheduler: Scheduler::new(),
+            next_thread_number: AtomicUsize::new(1),
+            events: &events,
+            sender,
+            judged_times: &judged_times,
+            blocks_judged_by_each_card: &blocks_judged_by_each_card,
+        };
+        let mut cards_finished: Vec<u32> = Vec::new();
+        let mut cards_out_of_memory: Vec<(u32, String)> = Vec::new();
+        let mut card_panic: Option<String> = None;
+        let mut receive =
+            |rounds_receiving: &mut RoundsOfTheCards<usize>, message: CardMessage| match message {
+                CardMessage::Block(pending_index) => {
+                    rounds_receiving.stored_in_order().arrive(
+                        pending_index,
+                        pending_index,
+                        &mut |_block| Ok(()),
+                    );
+                }
+                CardMessage::Finished(index) => cards_finished.push(index),
+                CardMessage::OutOfMemory(index, reason) => {
+                    cards_out_of_memory.push((index, reason))
+                }
+                CardMessage::CardPanicked(text) => card_panic = Some(text),
+            };
+        std::thread::scope(|scope| {
+            let work = &work;
+            work.scheduler.register_a_thread();
+            scope.spawn(move || card_thread(scope, work, 0, 0));
+            loop {
+                {
+                    let mut state = work.scheduler.locked();
+                    while !(state.thread_granted.is_none()
+                        && state.threads_parked.len() == state.threads_alive)
+                    {
+                        state = work
+                            .scheduler
+                            .changed
+                            .wait(state)
+                            .unwrap_or_else(std::sync::PoisonError::into_inner);
+                    }
+                    if state.threads_alive == 0 {
+                        break;
+                    }
+                    let parked = u64::try_from(state.threads_parked.len()).expect("几条线程");
+                    let pick = *state
+                        .threads_parked
+                        .iter()
+                        .nth(usize::try_from(random.below(parked)).expect("比线程数小"))
+                        .expect("挑的是停着的里的一条");
+                    state.thread_granted = Some(pick);
+                    work.scheduler.changed.notify_all();
+                }
+                while let Ok(message) = receiver.try_recv() {
+                    receive(&mut rounds, message);
+                }
+            }
+        });
+        while let Ok(message) = receiver.try_recv() {
+            receive(&mut rounds, message);
+        }
+        if let Some(text) = card_panic {
+            return finish(EndOfTheRun::CardPanicked(text), false);
+        }
+        // 一轮收尾：问这一趟的叫停
+        events.fetch_add(1, Ordering::SeqCst);
+        let stop = stop_request.stop_of_the_run();
+        let card_indices_of_the_round: Vec<u32> =
+            cards_of_the_round.iter().map(|card| card.index).collect();
+        let after = catch_unwind(AssertUnwindSafe(|| {
+            rounds.after_the_round(
+                &work.blocks,
+                &card_indices_of_the_round,
+                cards_finished.clone(),
+                &cards_out_of_memory,
+                stop,
+            )
+        }));
+        match after {
+            Err(panic) => {
+                let text = panic.downcast_ref::<String>().cloned().unwrap_or_default();
+                return finish(EndOfTheRun::AssertPanicked(text), false);
+            }
+            Ok(AfterARoundOfTheCards::EveryBlockStored) => {
+                return finish(EndOfTheRun::EveryBlockStored, false);
+            }
+            Ok(AfterARoundOfTheCards::StoppedOnRequest) => {
+                return finish(EndOfTheRun::StoppedOnRequest, false);
+            }
+            Ok(AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort) => {
+                return finish(EndOfTheRun::NotRun, true);
+            }
+            Ok(
+                AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
+                | AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor,
+            ) => {
+                round += 1;
+            }
+        }
+    }
+}
+
+/// 同攻方 `random_script`（`stop-file-removed` 一臂：叫停文件出现之后再过 1 到 20 个事件被删）。
+fn random_script_with_the_stop_file_removed(random: &mut SplitMixRandom) -> RunScript {
+    let blocks = 3 + usize::try_from(random.below(5)).expect("小于 5");
+    let card_count = 2 + random.below(2);
+    let cards = (0..card_count)
+        .map(|index| CardScript {
+            index: u32::try_from(index).expect("几张卡"),
+            states_placed_in_each_round: if random.below(4) == 0 {
+                vec![1, 0]
+            } else {
+                vec![1]
+            },
+            out_of_memory_building_the_judge_in_round: match random.below(10) {
+                0 => Some(0),
+                1 => Some(1),
+                _ => None,
+            },
+            out_of_memory_at_its_block: if random.below(2) == 0 {
+                None
+            } else {
+                Some(1 + usize::try_from(random.below(3)).expect("小于 3"))
+            },
+        })
+        .collect();
+    let stop_file_appears_at_event = if random.below(4) == 0 {
+        None
+    } else {
+        Some(1 + usize::try_from(random.below(40)).expect("小于 40"))
+    };
+    let stop_file_removed_at_event = stop_file_appears_at_event
+        .map(|appears| appears + 1 + usize::try_from(random.below(20)).expect("小于 20"));
+    RunScript {
+        blocks,
+        cards,
+        stop_file_appears_at_event,
+        stop_file_removed_at_event,
+    }
+}
+
+/// 攻方 `stop-file-removed` 一臂（判决第三节第 1 条要它的两样结局都为 0）：叫停文件出现、中途被删，固定种子跑 6 000 段。
+/// 轮末断言 panic 的段数、照叫停收尾而整趟没记成提早收尾的段数都是 0；这一臂要钉的窗口（见过叫停、收尾时文件已经不在）至少走到一段。
+#[test]
+fn a_stop_file_removed_mid_run_neither_panics_the_round_nor_leaves_the_stopped_run_unmarked() {
+    std::panic::set_hook(Box::new(|_| {}));
+    let mut random = SplitMixRandom { state: FIRST_SEED };
+    let mut ends: BTreeMap<String, usize> = BTreeMap::new();
+    let mut asserts_panicked = 0usize;
+    let mut cards_panicked = 0usize;
+    let mut stopped_on_request_but_run_not_marked_stopped_early = 0usize;
+    let mut stop_seen_and_the_file_gone_at_the_end = 0usize;
+    let mut first_example: Option<String> = None;
+    for _ in 0..SCRIPTS {
+        let script = random_script_with_the_stop_file_removed(&mut random);
+        for seed_number in 0..SEEDS_OF_EACH_SCRIPT {
+            let seed = random.next_value() ^ seed_number;
+            let result = run_once(&script, seed);
+            *ends
+                .entry(
+                    format!("{:?}", result.end)
+                        .split('(')
+                        .next()
+                        .unwrap_or("")
+                        .to_string(),
+                )
+                .or_insert(0) += 1;
+            let failure = match &result.end {
+                EndOfTheRun::AssertPanicked(text) => {
+                    asserts_panicked += 1;
+                    Some(format!("轮末断言 panic（{text}）"))
+                }
+                EndOfTheRun::CardPanicked(text) => {
+                    cards_panicked += 1;
+                    Some(format!("卡线程 panic（{text}）"))
+                }
+                EndOfTheRun::StoppedOnRequest if !result.stopped_early => {
+                    stopped_on_request_but_run_not_marked_stopped_early += 1;
+                    Some("照叫停收尾而整趟没记成提早收尾".to_string())
+                }
+                EndOfTheRun::EveryBlockStored
+                | EndOfTheRun::StoppedOnRequest
+                | EndOfTheRun::NotRun
+                | EndOfTheRun::NoCardHoldsAState
+                | EndOfTheRun::TooManyRounds => None,
+            };
+            if let (Some(failure), None) = (failure, &first_example) {
+                first_example = Some(format!(
+                    "{failure}：seed={seed} script={script:?} result={result:?}"
+                ));
+            }
+            if result.stop_seen_and_the_file_gone_at_the_end {
+                stop_seen_and_the_file_gone_at_the_end += 1;
+            }
+        }
+    }
+    let _default_hook = std::panic::take_hook();
+    println!(
+        "STOP_FILE_REMOVED runs={} ends={ends:?} asserts_panicked={asserts_panicked} cards_panicked={cards_panicked} \
+         stopped_on_request_but_run_not_marked_stopped_early={stopped_on_request_but_run_not_marked_stopped_early} \
+         stop_seen_and_the_file_gone_at_the_end={stop_seen_and_the_file_gone_at_the_end}",
+        SCRIPTS * usize::try_from(SEEDS_OF_EACH_SCRIPT).expect("几个种子")
+    );
+    assert_eq!(asserts_panicked, 0, "轮末断言 panic：{first_example:?}");
+    assert_eq!(cards_panicked, 0, "卡线程 panic：{first_example:?}");
+    assert_eq!(
+        stopped_on_request_but_run_not_marked_stopped_early, 0,
+        "照叫停收尾而整趟没记成提早收尾：{first_example:?}"
+    );
+    assert!(
+        stop_seen_and_the_file_gone_at_the_end > 0,
+        "见过叫停、收尾时文件已经不在的段一段都没走到：这一臂没钉到要钉的窗口"
+    );
+}
````

## 变异表追加（mutations-append.tsv）

````tsv
第10z批第六块 显存账 G2 乙 销了账之后照旧从实际占着的里扣一份驱动那么大（甲的少借回来）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            mebibytes.min(promised_mebibytes)	            mebibytes.saturating_sub(mebibytes_left_for_the_driver).min(promised_mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_returned_after_the_device_finished_is_no_longer_taken_off_the_free_memory	a_lease_returned_after_the_device_finished_is_no_longer_taken_off_the_free_memory
第10z批第六块 显存账 G2 乙 销了账之后照旧扣一份驱动那么大（驱动实占比 D 小时那一笔）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            mebibytes.min(promised_mebibytes)	            mebibytes.saturating_sub(mebibytes_left_for_the_driver).min(promised_mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_returned_after_the_device_finished_leaves_room_only_for_what_the_driver_still_grows_to	a_lease_returned_after_the_device_finished_leaves_room_only_for_what_the_driver_still_grows_to
第10z批第六块 显存账 G2 乙 销了账之后照旧扣一份驱动那么大（长借据建着、短借据一笔笔还）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            mebibytes.min(promised_mebibytes)	            mebibytes.saturating_sub(mebibytes_left_for_the_driver).min(promised_mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::returned_short_leases_while_a_long_lease_stays_built_leave_the_room_as_it_was	returned_short_leases_while_a_long_lease_stays_built_leave_the_room_as_it_was
第10z批第六块 显存账 G2 乙 销了账之后照旧扣一份驱动那么大（还了一笔再借一笔建满）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            mebibytes.min(promised_mebibytes)	            mebibytes.saturating_sub(mebibytes_left_for_the_driver).min(promised_mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_booked_after_another_was_returned_counts_its_built_buffers_once	a_lease_booked_after_another_was_returned_counts_its_built_buffers_once
第10z批第六块 显存账 G2 乙 等设备没等成照样销账（等失败那一支多给）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                self.when_dropped = LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait;	                self.when_dropped = LeaseWhenTheBookingIsDropped::ReturnedToTheDriverAndOffTheBooks;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_whose_wait_failed_stays_booked_until_the_device_is_taken_as_lost_and_gives_no_more	a_lease_whose_wait_failed_stays_booked_until_the_device_is_taken_as_lost_and_gives_no_more
第10z批第六块 显存账 G2 乙 设备丢了、账照重起记时没等成的那几笔不销账	crates/singlefs-harness/src/gpu_memory_ledger.rs	            self.leases.remove(&kept_lease);	            let _ = &kept_lease;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_whose_wait_failed_stays_booked_until_the_device_is_taken_as_lost_and_gives_no_more	a_lease_whose_wait_failed_stays_booked_until_the_device_is_taken_as_lost_and_gives_no_more
第10z批第六块 显存账 G2 乙 丢了的设备还可能占着卡时照读数分（旧设备那一份当成新设备的驱动那一份）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => return 0,	        | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => booked\n            .leases_mebibytes\n            .saturating_add(mebibytes_left_for_the_driver),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement	a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
第10z批第六块 显存账 G2 乙 重起好了就当旧设备已经不在（设备丢了那一刻活着的借据还在）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .leases_alive_when_the_device_was_lost\n            .is_empty()\n        {	            .leases_alive_when_the_device_was_lost\n            .is_empty()\n            || true\n        {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement	a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
第10z批第六块 显存账 G2 乙 设备丢了那一刻活着的借据都还了也不回到照读数分	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    self.driver_share = DriverShareOnTheCard::DeviceStarted;	                    self.driver_share =\n                        DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement	a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
第10z批第六块 显存账 G2 乙 设备丢了、账照重起记时不记那一刻活着的借据	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .extend(self.leases.keys().copied());	            .extend(std::iter::empty::<u64>());	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement	a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
第10z批第六块 叫停 K2-2 见过叫停不记住（每问一次现读）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            self.has_seen_the_request.store(true, Ordering::SeqCst);	            let _ = &self.has_seen_the_request;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_stop_request_seen_once_stays_requested_after_the_request_is_gone	a_stop_request_seen_once_stays_requested_after_the_request_is_gone
第10z批第六块 叫停 K2-2 见过叫停不记住（攻方 stop-file-removed 一臂：轮末断言 panic、整趟没记成提早收尾）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            self.has_seen_the_request.store(true, Ordering::SeqCst);	            let _ = &self.has_seen_the_request;	-p singlefs-harness --test gpu_card_rounds_remember_a_stop_request_removed_mid_run -- a_stop_file_removed_mid_run_neither_panics_the_round_nor_leaves_the_stopped_run_unmarked	a_stop_file_removed_mid_run_neither_panics_the_round_nor_leaves_the_stopped_run_unmarked
第10z批第六块 叫停 K1-3 照叫停收尾时不给头一句 Out of Memory 那一行	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.first_out_of_memory\n            .as_deref()\n            .map(|reason| gpu_not_run_line(run_name, reason))	        None::<&str>.map(|reason| gpu_not_run_line(run_name, reason))	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_stop_after_an_out_of_memory_prints_the_first_out_of_memory_in_the_not_run_line	a_stop_after_an_out_of_memory_prints_the_first_out_of_memory_in_the_not_run_line
第10z批第六块 K3 F4 活交回那一刻记设备挪到接住之前	crates/singlefs-harness/src/gpu_memory_ledger.rs	    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));\n    let device_when_the_work_returned = condition.device_when_the_work_returns();	    let device_when_the_work_returned = condition.device_when_the_work_returns();\n    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_work_notes_the_device_at_the_moment_it_returns_not_before_and_not_after	a_work_notes_the_device_at_the_moment_it_returns_not_before_and_not_after
第10z批第六块 K3 起下一张卡不看叫停与队里领不领得	crates/singlefs-harness/src/gpu_memory_ledger.rs	    if blocks.any_left_to_take(stop) {	    if true || blocks.any_left_to_take(stop) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_next_card_starts_only_when_it_holds_states_and_a_block_is_left_to_take_under_the_stop	the_next_card_starts_only_when_it_holds_states_and_a_block_is_left_to_take_under_the_stop
第10z批第六块 K3 M05 判完块号记最后一块而不是最大	crates/singlefs-harness/src/gpu_memory_ledger.rs	largest.max(pending_index)	pending_index	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_largest_judged_block_not_the_last_judged_lets_a_stopped_card_take_the_block_it_waits_for	the_largest_judged_block_not_the_last_judged_lets_a_stopped_card_take_the_block_it_waits_for
第10z批第六块 K3 M10 叫停、在等的块边界 ==0 改 <2	crates/singlefs-harness/src/gpu_memory_ledger.rs	StopOfTheRun::Requested if blocks_waiting == 0 =>	StopOfTheRun::Requested if blocks_waiting < 2 =>	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_stop_with_one_judged_block_waiting_needs_another_round_or_is_not_run	a_stop_with_one_judged_block_waiting_needs_another_round_or_is_not_run
第10z批第六块 K3 M13 轮末第一条断言（在等的块是 0）去掉	crates/singlefs-harness/src/gpu_memory_ledger.rs	            assert_eq!(\n                self.stored_in_order.blocks_waiting(),\n                0,	            assert_ne!(\n                self.stored_in_order.blocks_waiting(),\n                usize::MAX,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_round_without_out_of_memory_leaving_a_judged_block_waiting_panics	a_round_without_out_of_memory_leaving_a_judged_block_waiting_panics
第10z批第六块 K3 M14 轮末第二条断言（没叫停块都领走）去掉	crates/singlefs-harness/src/gpu_memory_ledger.rs	                !blocks.any_left() || stop == StopOfTheRun::Requested,	                true,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_round_without_a_stop_leaving_a_block_untaken_panics	a_round_without_a_stop_leaving_a_block_untaken_panics
第10z批第六块 K3 M17 头一句 Out of Memory 取最后一句	crates/singlefs-harness/src/gpu_memory_ledger.rs	            self.first_out_of_memory\n                .get_or_insert_with(|| reason.clone());	            self.first_out_of_memory = Some(reason.clone());	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_first_out_of_memory_of_the_run_stays_the_first_reported	the_first_out_of_memory_of_the_run_stays_the_first_reported
第10z批第六块 K3 M22 交回的块再交回不拦	crates/singlefs-harness/src/gpu_memory_ledger.rs	            is_newly_given_back,\n            "块 {pending_index} 交回时已经在队里	            true || is_newly_given_back,\n            "块 {pending_index} 交回时已经在队里	-p singlefs-harness --lib -- gpu_memory_ledger::tests::giving_a_block_back_twice_panics	giving_a_block_back_twice_panics
第10z批第六块 K3 M23 认 Out of Memory 取最早一条而不是最后	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .iter()\n            .rev()\n            .find(|report| report.works_it_explains.contains(&work))	            .iter()\n            .find(|report| report.works_it_explains.contains(&work))	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_last_out_of_memory_recorded_on_a_work_is_the_one_told	the_last_out_of_memory_recorded_on_a_work_is_the_one_told
第10z批第六块 GPU 取卡 G2 乙 借据丢掉时改回只收一次（Poll，不等设备做完）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                    device.poll(wgpu::PollType::Wait {\n                        submission_index: None,\n                        timeout: Some(WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE),\n                    })	                    device.poll(wgpu::PollType::Poll)	-p singlefs-checker-tier --features gpu --test gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card -- a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading	a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading
第10z批第六块 GPU 取卡 G2 乙 没等成也当等完了（照常销账）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        Ok(Ok(wgpu::PollStatus::Poll) | Err(_)) | Err(_) => {\n            DeviceWaitBeforeReturningTheLease::TheWaitFailed	        Ok(Ok(wgpu::PollStatus::Poll) | Err(_)) | Err(_) => {\n            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::a_lease_is_returned_only_when_the_wait_for_the_device_finished	a_lease_is_returned_only_when_the_wait_for_the_device_finished
第10z批第六块 GPU 取卡 K3 F5 扫 wgpu-core 丢设备那一句找的不是 self.lose(&error.to_string())	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        const LOSS_WITH_THE_SENTENCE_OF_THE_HAL_ERROR: &str = "self.lose(&error.to_string());";	        const LOSS_WITH_THE_SENTENCE_OF_THE_HAL_ERROR: &str = "self.lose(error.to_string());";	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::wgpu_core_loses_the_device_with_the_sentence_of_the_hal_error_it_failed_with	wgpu_core_loses_the_device_with_the_sentence_of_the_hal_error_it_failed_with
第10z批第六块 崩溃放量 K2-2 整趟收尾改回现读叫停文件	crates/singlefs-checker-tier/src/crash_amplification.rs	    let stop_was_requested = gpu_memory_was_short || stop_is_requested();	    let stop_was_requested = gpu_memory_was_short || stop_is_requested_by_the_environment();	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::the_run_reads_the_stop_file_only_into_the_stop_request_it_remembers	the_run_reads_the_stop_file_only_into_the_stop_request_it_remembers
第10z批第六块 崩溃放量 K1-3 照叫停收尾那一支不打头一句 Out of Memory 那一行	crates/singlefs-checker-tier/src/crash_amplification.rs	                if let Some(not_run_line) = rounds.not_run_line_of_a_stop_after_an_out_of_memory(\n                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,\n                ) {\n                    println!("{not_run_line}");\n                }\n	                // 变异：照叫停收尾不打头一句 Out of Memory 那一行\n	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::the_gpu_rounds_stopped_on_request_print_the_first_out_of_memory_in_the_not_run_line	the_gpu_rounds_stopped_on_request_print_the_first_out_of_memory_in_the_not_run_line
````

## 整行替换（mutations-replacements.tsv）

````tsv
第10x批 显存账 实际占着的不封顶到许出去的（分配器多拿的块也算成能借的）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            mebibytes.min(promised_mebibytes)	            mebibytes	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised	the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised
第10x批 显存账 借据丢掉不销账	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if self.leases.remove(&lease_identifier).is_none() {	        if self.leases.get(&lease_identifier).is_none() {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_is_on_the_books_until_it_is_dropped	a_lease_is_on_the_books_until_it_is_dropped
第10y批 显存账 K1-1 另有不走账的设备也当都走这份账（占着的当成借据建上的，攻方乙形）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => {\n            mebibytes.min(promised_mebibytes)\n        }	        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes }\n        | OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes } => {\n            mebibytes.min(promised_mebibytes)\n        }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::memory_of_this_process_outside_the_books_is_not_taken_for_leases_already_built	memory_of_this_process_outside_the_books_is_not_taken_for_leases_already_built
第10y批 显存账 K1-1 估计只取现读的空闲（实际占着的里借据建上的一份都不算，第10t批 K4-3 那一形）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            mebibytes.min(promised_mebibytes)	            0 * mebibytes.min(promised_mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised	the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised
第10z批 显存账 K1-3 设备丢了重起不重新登记驱动那一份	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::RegisteredDeviceNotStarted => Ok(()),\n            DriverShareOnTheCard::DeviceStarted\n	            DriverShareOnTheCard::RegisteredDeviceNotStarted | DriverShareOnTheCard::DeviceStarted => Ok(()),\n            DriverShareOnTheCard::DeviceStarted\n	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now	a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now
第10z批 显存账 K1-3 重起时旧设备那一份不当销了账而卡上可能还没还的	crates/singlefs-harness/src/gpu_memory_ledger.rs	        | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => return 0,	        | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => booked\n            .leases_mebibytes\n            .saturating_add(mebibytes_left_for_the_driver),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now	a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now
第10z批 显存账 K1-3 设备丢了重起不重新登记驱动那一份（重起之后能借的照旧当它在卡上）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::RegisteredDeviceNotStarted => Ok(()),\n            DriverShareOnTheCard::DeviceStarted\n	            DriverShareOnTheCard::RegisteredDeviceNotStarted | DriverShareOnTheCard::DeviceStarted => Ok(()),\n            DriverShareOnTheCard::DeviceStarted\n	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_started_again_books_its_driver_share_as_not_yet_on_the_card	a_lost_device_started_again_books_its_driver_share_as_not_yet_on_the_card
第10z批第三块 显存账 K1-3 重起被拒之后再来直接放行（正推 S6）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::DeviceLostNotStartedAgain => {\n                self.driver_share_of_a_new_device_fits_now(&books, card_position)\n            }	            DriverShareOnTheCard::DeviceLostNotStartedAgain => Ok(()),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_refused_a_restart_is_judged_again_on_every_later_attempt	a_lost_device_refused_a_restart_is_judged_again_on_every_later_attempt
第10z批第五块 显存账 K3 重起被拒之后借一笔把驱动那一份记回登记了、没起（正推 M13）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::RegisteredDeviceNotStarted\n            | DriverShareOnTheCard::DeviceStarted\n            | DriverShareOnTheCard::DeviceLostNotStartedAgain\n            | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => {}\n        }\n    }	            DriverShareOnTheCard::RegisteredDeviceNotStarted\n            | DriverShareOnTheCard::DeviceStarted\n            | DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => {}\n            DriverShareOnTheCard::DeviceLostNotStartedAgain => {\n                self.driver_share = DriverShareOnTheCard::RegisteredDeviceNotStarted;\n            }\n        }\n    }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_booked_after_a_refused_restart_keeps_the_restart_judged_again	a_lease_booked_after_a_refused_restart_keeps_the_restart_judged_again
````

## 删除（mutations-delete.txt）

````
第10y批 显存账 K1-1 销了账、卡上还没还的不记（当成别的借据已经建上的，正推冲突 B）
第10y批 显存账 K1-1 销了账、卡上还没还的那一份不照读数往下压（卡上还了照样少借）
第10z批 显存账 K1-1 销了账的那一份照驱动那一份整份在卡上往下压（实占 x < D 多给 D − x，攻方丙形）
第10z批 显存账 K3 往下压时另有不走账的设备也当都走账（正推 X2）
第10z批第三块 显存账 G2 甲 销了账的那一份照活着的借据都建满往下压（压到低于这个进程占着的，攻方丁、正推 S3 多给）
第10z批第三块 显存账 G2 甲 销了账的那一份每读一次整份压掉（第 10z 批「不多扣」那一形，S10 钉的甲的数变）
第10z批第五块 显存账 K3 读数不进账（正推 M01）
````
