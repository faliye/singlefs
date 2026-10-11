# 根环实现第 10z 批第一块：代码轮第一轮改动全文（2026-10-07）

实现员交的 `crates.patch` 原样，打在删除线第 33 批 T 合后、根环线 10y 合后的主工作区上（删除线 W、V 没合）。变异表追加、替换、删除三份附在后面。

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -37,12 +37,16 @@
 //!    每条同时还活着的照 `PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD` 算。
 //!    驱动的上下文与编好的管线算在每张卡第一次借时的驱动那一份里，不另记。
 //!    账只记这个进程借的：两个进程同一刻读到同一份空闲、一起借超了，由运行时兜底：第 ① 段卡上建判器、判一块时 wgpu 报 Out of Memory，
-//!    这张卡停下、这一块不入库、这一趟报本次未跑（原因写显存不够，`singlefs_harness::gpu_memory_ledger::reason_for_running_out_of_gpu_memory`）；
+//!    这张卡停下、这一块交回给别的卡，判不完才报本次未跑（原因写显存不够，`singlefs_harness::gpu_memory_ledger::reason_for_running_out_of_gpu_memory`）；
 //!    第 ② 段在卡上核时、只排派活计划那一趟试判时、卡池在借到的卡上起设备与编单元校验和管线时同样（第 10x 批代码轮判决 K1-3）。
 //!    判器内核在另起的线程上编，那里的 Out of Memory 不进卡线程的错误范围：每个设备上装了接 wgpu 没接住的错误的回调，Out of Memory
 //!    记在设备上（`crate::gpu_unit_checks::DeviceConditionOfACard`），卡线程上的活 panic 了就照它认是不是显存不够
-//!    （`decide_the_work_on_the_card`）；设备因为 Out of Memory 丢了，卡池下一次借这张卡时重起设备。
-//!    单元校验和对拍几条线程一起派活，那一段没包进接 Out of Memory 的那一层：那里的 Out of Memory 记在设备上、照旧让这一趟红。
+//!    （`decide_the_work_on_the_card`），按这段活认、不按整台设备认（第 10z 批：别的线程一次 Out of Memory 不把这条线程上真正的 panic
+//!    说成显存不够）；设备因为 Out of Memory 丢了，卡池下一次借这张卡时重起设备（驱动那一份在账上重新登记，放不下就报本次未跑）。
+//!    第 ① 段由 CPU 判时单元校验和对拍几条线程一起派活，每条线程那一段各自包进接 Out of Memory 的那一层（第 10z 批）：有一段报了，
+//!    这一块不入库、不再领下一块，这一趟报本次未跑（`CheckingTally::stopped_because_the_gpu_memory_was_short`，之前判完的块在库里）。
+//!    第 ① 段在几张卡上判时，一张卡建判器或判一块时报了 Out of Memory，只停那一张：它手里那一块交回队里，别的卡照判
+//!    （第 10z 批，第 10y 批代码轮判决 K2-2 P2）；剩下的块别的卡判完了这一趟照常交回，判不完（卡都停了）才报本次未跑。
 //!    没起过卡池的进程照旧不进账（[`GpuCardsSource::DevicesStartedForThisRun`]：这一趟自己在点名的卡上起设备），调用方是这几种：
 //!    崩溃放量分项驱动（`scripts/crash/crash-amplification-item.sh`）起的、只有经 [`run_from_environment`] 跑的用例而没有一条取卡池的测试二进制
 //!    （`tests/crash_amplification_fixed_script_through_rollback.rs`、`…_through_unmount.rs`、`crash_amplification_two_flows_share_their_prefix.rs`、
@@ -1156,6 +1160,9 @@
     pub gpu_cpu_disagreements: u64,
     /// 每个红状态的违例正文（点名、块内序号、正文），给门禁与人看；只收前 [`RED_TEXTS_KEPT`] 条。
     pub red_texts: Vec<(String, u64, String)>,
+    /// 单元校验和交卡对拍时卡上 wgpu 报了 Out of Memory（第 10z 批）：那一块不入库、之后的块不领，这一趟报本次未跑；
+    /// 是「本次未跑」那一行的原因（以「显存不够：」起头）。没报是 `None`。
+    pub stopped_because_the_gpu_memory_was_short: Option<String>,
 }
 
 pub const RED_TEXTS_KEPT: usize = 64;
@@ -1189,12 +1196,43 @@
     .get()
 }
 
-/// 一块判完的结果（还没写库）：每个状态判红的各项、红状态的正文、GPU 与 CPU 对不上的状态数。
+/// 一块判完的结果（还没写库）：每个状态判红的各项、红状态的正文、GPU 与 CPU 对不上的状态数；单元校验和交卡对拍时卡上报了
+/// Out of Memory 的，`gpu_memory_was_short` 是「本次未跑」那一行的原因，前几样都是空的（这一段不作数）。
 struct JudgedBlock {
     local: Range<u64>,
     items_of_each_state: Vec<Vec<String>>,
     red_texts: Vec<(u64, String)>,
     gpu_cpu_disagreements: u64,
+    gpu_memory_was_short: Option<String>,
+}
+
+/// 判一块的一段（[`judge_one_block`]）：单元校验和交卡对拍的，这一段包进接 Out of Memory 的那一层（[`on_the_card_catching_out_of_memory`]，
+/// 每条核对线程各认各的），报了就交一段不作数、带着原因的（第 10z 批，第 10y 批代码轮判决 K1-3）。
+fn judge_one_piece_catching_out_of_memory_on_the_card(
+    flow: &CrashFlow<'_>,
+    plan: &CrashPlan,
+    point_index: usize,
+    local: Range<u64>,
+    unit_checksums: &UnitChecksumsOn,
+) -> JudgedBlock {
+    match unit_checksums {
+        UnitChecksumsOn::CpuOnly => judge_one_block(flow, plan, point_index, local, unit_checksums),
+        #[cfg(feature = "gpu")]
+        UnitChecksumsOn::Gpu(card) => {
+            match on_the_card_catching_out_of_memory(card, "单元校验和对拍时", || {
+                judge_one_block(flow, plan, point_index, local.clone(), unit_checksums)
+            }) {
+                Ok(judged) => judged,
+                Err(reason) => JudgedBlock {
+                    local,
+                    items_of_each_state: Vec::new(),
+                    red_texts: Vec::new(),
+                    gpu_cpu_disagreements: 0,
+                    gpu_memory_was_short: Some(reason),
+                },
+            }
+        }
+    }
 }
 
 fn judge_one_block(
@@ -1266,13 +1304,15 @@
         items_of_each_state,
         red_texts,
         gpu_cpu_disagreements,
+        gpu_memory_was_short: None,
     }
 }
 
 /// 核对录入表里还没有判定的块：按分工只核 `owner` 那一方分到的；一块切成几段多线程判（[`checking_threads`]），
 /// 每块在 CPU 上逐状态判（`unit_checksums` 给了卡就把每个状态镜像里被写过的单元两道校验和交 GPU 算、与 CPU 逐个对拍），
 /// 判完一块就按块的次序单线程写回判定块与违例——判定向量的编号按块序登记，与线程数无关，输出与单线程逐字相同。
-/// 同一块两方不会同时领：块的归属由权重表与块序号定死。
+/// 同一块两方不会同时领：块的归属由权重表与块序号定死。单元校验和对拍时卡上报了 Out of Memory（每条核对线程各自接），那一块不写、
+/// 不再领下一块，交回的账带着原因（[`CheckingTally::stopped_because_the_gpu_memory_was_short`]，调用方报本次未跑）。
 ///
 /// # Errors
 /// 库读写失败；判定向量超过 256 种。
@@ -1367,7 +1407,13 @@
                 let range = range.clone();
                 let judged_sender = judged_sender.clone();
                 scope.spawn(move || {
-                    let piece = judge_one_block(flow, plan, *point_index, range, unit_checksums);
+                    let piece = judge_one_piece_catching_out_of_memory_on_the_card(
+                        flow,
+                        plan,
+                        *point_index,
+                        range,
+                        unit_checksums,
+                    );
                     // 发不出去说明收的这条线程已经走了（它 panic 了），这一段的结果没人要
                     let _ = judged_sender.send(piece);
                 });
@@ -1395,6 +1441,11 @@
             "块 {pending_index} 的每一段都判完交回了（核对线程 panic 时上面的 scope 已经往外传）"
         );
         judged.sort_by_key(|piece| piece.local.start);
+        // 单元校验和对拍时有一段报了 Out of Memory：这一块不入库、不再领下一块，判过的块在库里（文件头第 5 条）
+        if let Some(reason) = reason_the_gpu_memory_was_short_in_one_of_the_pieces(&judged) {
+            tally.stopped_because_the_gpu_memory_was_short = Some(reason);
+            break;
+        }
         let mut items_of_each_state: Vec<Vec<String>> =
             Vec::with_capacity(usize::try_from(local.end - local.start).expect("一块装得进 usize"));
         let mut next_expected = local.start;
@@ -1434,6 +1485,13 @@
     Ok(tally)
 }
 
+/// 一块按段序排好的几段里，单元校验和对拍时卡上报了 Out of Memory 的第一段的原因；都没报是 `None`。
+fn reason_the_gpu_memory_was_short_in_one_of_the_pieces(pieces: &[JudgedBlock]) -> Option<String> {
+    pieces
+        .iter()
+        .find_map(|piece| piece.gpu_memory_was_short.clone())
+}
+
 /// 一块正在判的时候按挂钟那一行要报的：哪个点、块从点内第几个状态起、切成几段，这一趟已经判完几块（共几块）与几个状态。
 struct BlockInProgress<'point> {
     point_name: &'point str,
@@ -1970,8 +2028,8 @@
     scratch_booked_mebibytes: &'work [Option<u64>],
     /// 每张卡草稿区那一笔借据的借据号（判器建好那一刻核账用）；不进账的是 `None`。
     scratch_lease_identifiers: &'work [Option<u64>],
-    next_pending: std::sync::atomic::AtomicUsize,
-    /// 写库失败之后置上：卡上的线程判完手里那一块就停。
+    blocks: BlocksForTheCards,
+    /// 写库失败、有卡拒判之后置上：卡上的线程判完手里那一块就停。一张卡报 Out of Memory 不置（只停那一张，第 10z 批）。
     stop: std::sync::atomic::AtomicBool,
     /// 调用方要停（每领一块之前问一次）：不再领新的块，手里那一块判完照旧交去写库。
     stop_is_requested: &'work (dyn Fn() -> bool + Sync),
@@ -1984,6 +2042,69 @@
     }
 }
 
+/// 几张卡一起判的那一队块（要判的块在 `pending` 里的次序）：从队头领；一张卡报了 Out of Memory 把手里那一块交回（第 10z 批，
+/// 第 10y 批代码轮判决 K2-2 P2：一张卡 Out of Memory 不停整趟），交回的先给下一张来领的卡。
+#[cfg(feature = "gpu")]
+struct BlocksForTheCards {
+    pending_blocks: usize,
+    next_pending: std::sync::atomic::AtomicUsize,
+    given_back: std::sync::Mutex<Vec<usize>>,
+}
+
+#[cfg(feature = "gpu")]
+impl BlocksForTheCards {
+    fn new(pending_blocks: usize) -> Self {
+        Self {
+            pending_blocks,
+            next_pending: std::sync::atomic::AtomicUsize::new(0),
+            given_back: std::sync::Mutex::new(Vec::new()),
+        }
+    }
+
+    /// 领一块：先领交回的（序号小的先），没有就从队头领；都领完了是 `None`。
+    fn take_a_block(&self) -> Option<usize> {
+        // 持锁只为取一个序号，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
+        let mut given_back = self
+            .given_back
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner);
+        if let Some(smallest) = given_back.iter().copied().min() {
+            given_back.retain(|pending_index| *pending_index != smallest);
+            return Some(smallest);
+        }
+        drop(given_back);
+        let pending_index = self
+            .next_pending
+            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
+        (pending_index < self.pending_blocks).then_some(pending_index)
+    }
+
+    /// 领了没判完的一块交回队里（报了 Out of Memory 的卡交）。
+    fn give_back(&self, pending_index: usize) {
+        self.given_back
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+            .push(pending_index);
+    }
+
+    /// 还有没领走的块没有（队里的或交回的）。
+    fn any_left(&self) -> bool {
+        self.next_pending.load(std::sync::atomic::Ordering::SeqCst) < self.pending_blocks
+            || !self
+                .given_back
+                .lock()
+                .unwrap_or_else(std::sync::PoisonError::into_inner)
+                .is_empty()
+    }
+
+    /// 从队头领走了几块（不算交回的；序号连着、从 0 起）。
+    fn taken_from_the_head(&self) -> usize {
+        self.next_pending
+            .load(std::sync::atomic::Ordering::SeqCst)
+            .min(self.pending_blocks)
+    }
+}
+
 /// 卡上一段活做完之后的结局（[`decide_the_work_on_the_card`]）。
 #[cfg(feature = "gpu")]
 enum WorkOnTheCard<Outcome> {
@@ -2021,13 +2142,15 @@
 /// 在 `card` 上做 `work`，同时接住 wgpu 报的 Out of Memory（文件头第 5 条）：没报就交 `work` 的结果；报了交 `Err(那一句)`
 /// （`what_ran_out` 说是哪一段，`work` 里因为缓冲没建成而 panic 的一并接住、不再往外抛）；没报而 `work` panic 了照样往外抛。
 /// wgpu 的错误范围按线程分：这条线程上的接在范围里；`work` 另起的线程上没接住的记在设备上（`crate::gpu_unit_checks::DeviceConditionOfACard`），
-/// 照 [`decide_the_work_on_the_card`] 一并认。范围接到的也记在设备上（同一个设备上别的线程照它认）。
+/// 照 [`decide_the_work_on_the_card`] 一并认。设备上记的按这段活认（`singlefs_harness::gpu_memory_ledger::WorkOnTheCardOfThisThread`，
+/// 第 10z 批：同一个设备上别的线程那段活的 Out of Memory 说不成这段活的显存不够，这段活真正的 panic 照样往外抛）。
 #[cfg(feature = "gpu")]
 fn on_the_card_catching_out_of_memory<Outcome>(
     card: &crate::gpu_unit_checks::GpuCard,
     what_ran_out: &str,
     work: impl FnOnce() -> Outcome,
 ) -> Result<Outcome, String> {
+    let work_on_the_card = singlefs_harness::gpu_memory_ledger::WorkOnTheCardOfThisThread::begin();
     let out_of_memory_reports_before = card.device_condition().out_of_memory_reports();
     let out_of_memory_scope = card
         .device()
@@ -2037,7 +2160,10 @@
         pollster::block_on(out_of_memory_scope.pop()).map(|error| error.to_string());
     let out_of_memory_on_the_device = card
         .device_condition()
-        .out_of_memory_explaining_a_failure_since(out_of_memory_reports_before);
+        .out_of_memory_explaining_a_failure_of_the_work(
+            work_on_the_card.identifier(),
+            out_of_memory_reports_before,
+        );
     if let Some(caught) = &out_of_memory_in_the_scope {
         card.device_condition().record_an_out_of_memory(format!(
             "序号 {} 的卡{what_ran_out}报了 {caught}",
@@ -2060,9 +2186,10 @@
 /// 一张卡的线程：表上卡、编内核；备好之后，队里还有活、下一张卡也装了状态，就起下一张卡的线程（卡一张一张备，不同时编内核）；
 /// 然后从队头一块一块领、判、把结论字拆成红项、交给写库的线程。有一个状态判不了（结论字带「判不了」），
 /// 就叫所有卡停下、发一条 [`CardMessage::Refused`]，不退回 CPU 判器（用户 2026-09-29 定：配了 GPU 就一定要 GPU 判，做不到就报错）。
-/// 建判器或判一块时 wgpu 报 Out of Memory（[`on_the_card_catching_out_of_memory`]），叫所有卡停下、发一条 [`CardMessage::OutOfMemory`]，
-/// 那一块不交去写库。起过卡池的进程，判器建好那一刻与这张卡判完那一刻各核一次账：草稿区那一笔借据还在不在（第 10t 批代码轮判决 K4-1、
-/// 第 10x 批代码轮判决 K3-1）。
+/// 建判器或判一块时 wgpu 报 Out of Memory（[`on_the_card_catching_out_of_memory`]），只停这一张卡、发一条 [`CardMessage::OutOfMemory`]：
+/// 判一块时报的，那一块交回队里给别的卡（[`BlocksForTheCards::give_back`]），不交去写库；建判器时报的，下一张卡照起
+/// （第 10z 批，第 10y 批代码轮判决 K2-2 P2：一张卡 Out of Memory 不停整趟）。起过卡池的进程，判器建好那一刻与这张卡判完那一刻
+/// 各核一次账：草稿区那一笔借据还在不在（第 10t 批代码轮判决 K4-1、第 10x 批代码轮判决 K3-1）。
 #[cfg(feature = "gpu")]
 fn judge_on_one_card_and_start_the_next<'scope, 'work>(
     scope: &'scope std::thread::Scope<'scope, 'work>,
@@ -2085,9 +2212,9 @@
         }) {
             Ok(judge) => judge,
             Err(reason) => {
-                work.stop.store(true, Ordering::SeqCst);
                 // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
                 let _ = sender.send(CardMessage::OutOfMemory(reason));
+                start_the_next_card_with_states(scope, work, position, &sender);
                 return;
             }
         };
@@ -2107,16 +2234,7 @@
         judge.states_per_dispatch(),
         judge.scratch_mebibytes()
     );
-    let next_position =
-        (position + 1..work.cards.len()).find(|candidate| work.states_placed[*candidate] > 0);
-    if let Some(next_position) = next_position {
-        if work.next_pending.load(Ordering::SeqCst) < work.pending.len() && !work.must_stop() {
-            let sender = sender.clone();
-            scope.spawn(move || {
-                judge_on_one_card_and_start_the_next(scope, work, next_position, sender);
-            });
-        }
-    }
+    start_the_next_card_with_states(scope, work, position, &sender);
     let judging_started = Instant::now();
     let mut outcome = GpuCardOutcome {
         name: card.name.clone(),
@@ -2130,10 +2248,9 @@
         ..GpuCardOutcome::default()
     };
     while !work.must_stop() {
-        let pending_index = work.next_pending.fetch_add(1, Ordering::SeqCst);
-        if pending_index >= work.pending.len() {
+        let Some(pending_index) = work.blocks.take_a_block() else {
             break;
-        }
+        };
         let (point_index, local) = &work.pending[pending_index];
         let point = &work.plan.points[*point_index];
         let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
@@ -2142,7 +2259,8 @@
         }) {
             Ok(words) => words,
             Err(reason) => {
-                work.stop.store(true, Ordering::SeqCst);
+                // 只停这一张卡：这一块交回给别的卡（第 10z 批）
+                work.blocks.give_back(pending_index);
                 let _ = sender.send(CardMessage::OutOfMemory(reason));
                 return;
             }
@@ -2226,6 +2344,99 @@
     let _ = sender.send(CardMessage::Finished(outcome));
 }
 
+/// 第 ① 段借草稿区之前这一步怎么走（[`scratch_step_before_booking`]）。
+#[cfg(feature = "gpu")]
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum ScratchStepBeforeBooking {
+    /// 至少一张卡此刻带得了一个状态：去借。
+    BookOnTheCards,
+    /// 每张卡此刻都带不了一个状态、至少一张空着时带得了：打这一行，报本次未跑（不借、不判红）。
+    NotRunNoCardHoldsAStateNow { not_run_line: String },
+    /// 空着时也带不了：往下走到没有一张卡装得下那一句，判红。
+    RedNoCardHoldsAStateEvenWhenEmpty,
+}
+
+/// 照借之前的判定（[`decide_scratch_before_booking`]）定这一步：此刻借不到的那一行照对拍用例那一路写（同一个记号、原因写显存不够），
+/// 带每张卡此刻能借的（额度减驱动那一份）与一个状态要借的 `need_of_one_state_mebibytes`。
+#[cfg(feature = "gpu")]
+fn scratch_step_before_booking(
+    decision: ScratchBeforeBooking,
+    run_cards: &[CardCapacityOfThisRun],
+    need_of_one_state_mebibytes: u64,
+) -> ScratchStepBeforeBooking {
+    match decision {
+        ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow => {
+            ScratchStepBeforeBooking::BookOnTheCards
+        }
+        ScratchBeforeBooking::NoCardHoldsAStateNow => {
+            let card_rooms_mebibytes: Vec<u64> = run_cards
+                .iter()
+                .map(|card| {
+                    card.memory_budget_mebibytes
+                        .saturating_sub(crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER)
+                })
+                .collect();
+            ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow {
+                not_run_line: gpu_not_run_line(
+                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
+                    &reason_for_a_booking_without_room_now(
+                        need_of_one_state_mebibytes,
+                        &card_rooms_mebibytes,
+                    ),
+                ),
+            }
+        }
+        ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty => {
+            ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty
+        }
+    }
+}
+
+/// 第 `position` 张卡备好了（或建判器时报了 Out of Memory、不判）：后面还有装了状态的卡、队里还有没领的块、没被叫停，就起下一张的线程
+/// （卡一张一张备，不同时编内核）。
+#[cfg(feature = "gpu")]
+fn start_the_next_card_with_states<'scope, 'work>(
+    scope: &'scope std::thread::Scope<'scope, 'work>,
+    work: &'work CardsAtWork<'work>,
+    position: usize,
+    sender: &std::sync::mpsc::Sender<CardMessage>,
+) {
+    let next_position =
+        (position + 1..work.cards.len()).find(|candidate| work.states_placed[*candidate] > 0);
+    if let Some(next_position) = next_position {
+        if work.blocks.any_left() && !work.must_stop() {
+            let sender = sender.clone();
+            scope.spawn(move || {
+                judge_on_one_card_and_start_the_next(scope, work, next_position, sender);
+            });
+        }
+    }
+}
+
+/// 几张卡都收工之后，有卡报过 Out of Memory 时这一趟怎么交回（[`judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from`]）。
+#[cfg(feature = "gpu")]
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum CardsOutcomeAfterAnOutOfMemory {
+    /// 别的卡把剩下的块都判完写进了库：这一趟照常交回（报 Out of Memory 的那几张只是没判）。
+    EveryBlockJudged,
+    /// 还有块没判（卡都停了，或被叫停）：这一趟报本次未跑，判过的块在库里。
+    NotRunTheGpuMemoryWasShort { reason: String },
+}
+
+/// 几张卡收工之后：写进库 `blocks_stored` 块（共 `pending_blocks` 块），`out_of_memory` 是第一张报 Out of Memory 的卡那一句（没有是 `None`）。
+#[cfg(feature = "gpu")]
+fn cards_outcome_after_an_out_of_memory(
+    blocks_stored: usize,
+    pending_blocks: usize,
+    out_of_memory: Option<String>,
+) -> Option<CardsOutcomeAfterAnOutOfMemory> {
+    let reason = out_of_memory?;
+    if blocks_stored >= pending_blocks {
+        return Some(CardsOutcomeAfterAnOutOfMemory::EveryBlockJudged);
+    }
+    Some(CardsOutcomeAfterAnOutOfMemory::NotRunTheGpuMemoryWasShort { reason })
+}
+
 /// 一张卡照额度 `memory_budget_mebibytes`（MiB）一次派活带得了几个状态（表上卡之后剩下的显存装草稿区）：表比存储缓冲绑定上限
 /// `max_storage_buffer_binding_size` 大、或额度装不下表与驱动那一份，是 0。
 #[cfg(feature = "gpu")]
@@ -2313,24 +2524,40 @@
     Ok(cards
         .into_iter()
         .zip(rooms)
-        .map(|(card, room)| CardCapacityOfThisRun {
-            index: card.index,
-            memory_budget_mebibytes: room.room_now_mebibytes
-                + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER,
-            states_in_flight_capacity: states_in_flight_for_a_room(
-                card.max_storage_buffer_binding_size,
-                room.room_now_mebibytes,
-            ),
-            states_in_flight_capacity_when_the_card_is_empty: states_in_flight_for_a_room(
-                card.max_storage_buffer_binding_size,
-                room.ceiling_mebibytes,
-            ),
-            name: card.name,
-            max_storage_buffer_binding_size: card.max_storage_buffer_binding_size,
+        .map(|(card, room)| {
+            let (states_in_flight_capacity, states_in_flight_capacity_when_the_card_is_empty) =
+                states_in_flight_now_and_when_the_card_is_empty(room, &|room_mebibytes| {
+                    states_in_flight_for_a_room(
+                        card.max_storage_buffer_binding_size,
+                        room_mebibytes,
+                    )
+                });
+            CardCapacityOfThisRun {
+                index: card.index,
+                memory_budget_mebibytes: room.room_now_mebibytes
+                    + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER,
+                states_in_flight_capacity,
+                states_in_flight_capacity_when_the_card_is_empty,
+                name: card.name,
+                max_storage_buffer_binding_size: card.max_storage_buffer_binding_size,
+            }
         })
         .collect())
 }
 
+/// 一张池里的卡此刻与空着时一次派活各带得了几个状态（`states_in_flight_for_a_room` 照能借的 MiB 算）：此刻照这一刻能借的，
+/// 空着时照空着时能借的（[`singlefs_harness::gpu_memory_ledger::CardRoomForThisProcess::ceiling_mebibytes`]，第 10x 批代码轮判决 K2-2）。
+#[cfg(feature = "gpu")]
+fn states_in_flight_now_and_when_the_card_is_empty(
+    room: singlefs_harness::gpu_memory_ledger::CardRoomForThisProcess,
+    states_in_flight_for_a_room: &dyn Fn(u64) -> u64,
+) -> (u64, u64) {
+    (
+        states_in_flight_for_a_room(room.room_now_mebibytes),
+        states_in_flight_for_a_room(room.ceiling_mebibytes),
+    )
+}
+
 /// 这一趟本机每张卡（按优先级）：起过卡池的照池（[`cards_of_the_pool_with_what_each_lends_now`]，`cards` 是空的），
 /// 这一趟自己起的照 `cards`（[`states_in_flight_of_each_card`]）。
 ///
@@ -2374,7 +2601,7 @@
 ///
 /// # Errors
 /// 库读写失败；建不出表；没有一张卡装得下这条流的表与一个状态的草稿区；有状态 GPU 判不了；卡上 wgpu 报了 Out of Memory
-/// （「本次未跑」那一行已经打了，报错之前判完的块在库里）。
+/// 而别的卡没把剩下的块判完（「本次未跑」那一行已经打了，报错之前判完的块在库里；只停报了的那一张卡，第 10z 批）。
 ///
 /// # Panics
 /// 卡上的线程 panic（内核没把状态判完、编内核之前量得内核超了预算）。
@@ -2426,7 +2653,6 @@
 }
 
 /// 「本次未跑」那一行里产品路径的名字（GPU 用例写用例名的那一格）。
-#[cfg(feature = "gpu")]
 const PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE: &str = "crash_amplification::run_from_environment";
 
 /// 这一趟的卡判成照这个进程的卡池用（[`gpu_cards_in_use`] 交 [`GpuCardsSource::TheCardPoolOfThisProcess`]）时的那个池。
@@ -2681,51 +2907,39 @@
         .map(|card| card.states_in_flight_capacity_when_the_card_is_empty)
         .collect();
     // 借据活到这个函数收尾：几张卡上的判器在下面的线程里判完就丢，丢完才还
-    let scratch =
-        match decide_scratch_before_booking(&capacities, &capacities_when_the_cards_are_empty) {
-            ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow => {
-                match scratch_of_each_card_booked_by_where_the_cards_came_from(
-                    cards_taken_from,
-                    cards,
-                    &run_cards,
-                    &tables,
-                    state_count,
-                    pending_states,
-                    &capacities,
-                )? {
-                    ScratchOfEachCard::Ready(scratch) => Some(scratch),
-                    ScratchOfEachCard::NotRunTheGpuMemoryWasShort { not_run_line } => {
-                        return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
-                            tally,
-                            reason: not_run_line,
-                        });
-                    }
+    let scratch = match scratch_step_before_booking(
+        decide_scratch_before_booking(&capacities, &capacities_when_the_cards_are_empty),
+        &run_cards,
+        crate::crash_judge_dispatch::mebibytes_a_judge_needs(&tables, 1),
+    ) {
+        ScratchStepBeforeBooking::BookOnTheCards => {
+            match scratch_of_each_card_booked_by_where_the_cards_came_from(
+                cards_taken_from,
+                cards,
+                &run_cards,
+                &tables,
+                state_count,
+                pending_states,
+                &capacities,
+            )? {
+                ScratchOfEachCard::Ready(scratch) => Some(scratch),
+                ScratchOfEachCard::NotRunTheGpuMemoryWasShort { not_run_line } => {
+                    return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
+                        tally,
+                        reason: not_run_line,
+                    });
                 }
             }
-            ScratchBeforeBooking::NoCardHoldsAStateNow => {
-                let card_rooms_mebibytes: Vec<u64> = run_cards
-                    .iter()
-                    .map(|card| {
-                        card.memory_budget_mebibytes.saturating_sub(
-                            crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER,
-                        )
-                    })
-                    .collect();
-                let not_run_line = gpu_not_run_line(
-                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
-                    &reason_for_a_booking_without_room_now(
-                        crate::crash_judge_dispatch::mebibytes_a_judge_needs(&tables, 1),
-                        &card_rooms_mebibytes,
-                    ),
-                );
-                println!("{not_run_line}");
-                return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
-                    tally,
-                    reason: not_run_line,
-                });
-            }
-            ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty => None,
-        };
+        }
+        ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow { not_run_line } => {
+            println!("{not_run_line}");
+            return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
+                tally,
+                reason: not_run_line,
+            });
+        }
+        ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty => None,
+    };
     let states_placed_on_each_card_of_this_run = scratch.as_ref().map_or_else(
         || vec![0; run_cards.len()],
         |scratch| scratch.states_placed_on_each_card_of_this_run.clone(),
@@ -2757,7 +2971,7 @@
         states_placed: &scratch.states_placed,
         scratch_booked_mebibytes: &scratch.scratch_booked_mebibytes,
         scratch_lease_identifiers: &scratch.scratch_lease_identifiers,
-        next_pending: std::sync::atomic::AtomicUsize::new(0),
+        blocks: BlocksForTheCards::new(pending.len()),
         stop: std::sync::atomic::AtomicBool::new(false),
         stop_is_requested,
     };
@@ -2807,10 +3021,7 @@
             && received.out_of_memory.is_none()
         {
             // 领走的块（序号连着、从 0 起）每一块都写进了库；没被叫停时领走的就是全部
-            let taken = work
-                .next_pending
-                .load(std::sync::atomic::Ordering::SeqCst)
-                .min(pending.len());
+            let taken = work.blocks.taken_from_the_head();
             assert_eq!(received.blocks_stored, taken, "领走的块每一块都写进了库");
             assert!(
                 received.blocks_stored == pending.len() || stop_is_requested(),
@@ -2827,12 +3038,25 @@
     if let Some(reason) = received.refusal {
         return Err(reason);
     }
-    if let Some(reason) = received.out_of_memory {
-        println!(
-            "{}",
-            gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
-        );
-        return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason });
+    match cards_outcome_after_an_out_of_memory(
+        received.blocks_stored,
+        pending.len(),
+        received.out_of_memory,
+    ) {
+        None => {}
+        Some(CardsOutcomeAfterAnOutOfMemory::EveryBlockJudged) => {
+            eprintln!(
+                "CRASH_AMPLIFICATION_GPU_OUT_OF_MEMORY_ON_A_CARD 有卡报了 Out of Memory、只停那一张，别的卡判完了全部 {} 块",
+                pending.len()
+            );
+        }
+        Some(CardsOutcomeAfterAnOutOfMemory::NotRunTheGpuMemoryWasShort { reason }) => {
+            println!(
+                "{}",
+                gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
+            );
+            return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason });
+        }
     }
     outcome.cards.sort_by_key(|card| {
         run_cards
@@ -3239,6 +3463,185 @@
         );
     }
 
+    /// 第 10z 批（第 10y 批代码轮判决 K1-3）：一块的几段里，单元校验和对拍时卡上报了 Out of Memory 的，按段序取第一段的原因
+    /// （那一块不入库、这一趟报本次未跑）；都没报是 `None`。不要卡：几段照核对线程交回的造。
+    #[test]
+    fn the_first_piece_short_of_gpu_memory_stops_the_checking_with_its_reason() {
+        let piece = |start: u64, gpu_memory_was_short: Option<&str>| JudgedBlock {
+            local: start..start + 2,
+            items_of_each_state: Vec::new(),
+            red_texts: Vec::new(),
+            gpu_cpu_disagreements: 0,
+            gpu_memory_was_short: gpu_memory_was_short.map(str::to_string),
+        };
+        assert_eq!(
+            reason_the_gpu_memory_was_short_in_one_of_the_pieces(&[
+                piece(0, None),
+                piece(2, Some("显存不够：第二段")),
+                piece(4, Some("显存不够：第三段")),
+            ]),
+            Some("显存不够：第二段".to_string())
+        );
+        assert_eq!(
+            reason_the_gpu_memory_was_short_in_one_of_the_pieces(&[piece(0, None), piece(2, None)]),
+            None
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K2-2 P2）：几张卡一起领的那一队块，报了 Out of Memory 的卡交回的那一块先给下一张来领的卡；
+    /// 交回的都领走、队里也领完才算没有剩下的。不要卡。
+    #[cfg(feature = "gpu")]
+    #[test]
+    fn a_block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one() {
+        let blocks = BlocksForTheCards::new(3);
+        assert_eq!(blocks.take_a_block(), Some(0));
+        assert_eq!(blocks.take_a_block(), Some(1));
+        blocks.give_back(0);
+        assert!(blocks.any_left());
+        assert_eq!(blocks.take_a_block(), Some(0), "交回的先领");
+        assert_eq!(blocks.take_a_block(), Some(2));
+        assert_eq!(blocks.take_a_block(), None);
+        assert!(!blocks.any_left());
+        blocks.give_back(1);
+        assert!(blocks.any_left(), "队里领完了、还有交回的");
+        assert_eq!(blocks.take_a_block(), Some(1));
+        assert_eq!(blocks.take_a_block(), None);
+        assert_eq!(blocks.taken_from_the_head(), 3);
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K2-2 P2）：有卡报了 Out of Memory，别的卡把剩下的块都判完写进了库，这一趟照常交回；
+    /// 没判完才报本次未跑（带第一张报的卡那一句）；没有卡报就不归这里管。
+    #[cfg(feature = "gpu")]
+    #[test]
+    fn an_out_of_memory_on_one_card_is_not_run_only_when_the_other_cards_left_blocks_unjudged() {
+        assert_eq!(
+            cards_outcome_after_an_out_of_memory(3, 3, Some("显存不够：卡 1".to_string())),
+            Some(CardsOutcomeAfterAnOutOfMemory::EveryBlockJudged)
+        );
+        assert_eq!(
+            cards_outcome_after_an_out_of_memory(2, 3, Some("显存不够：卡 1".to_string())),
+            Some(CardsOutcomeAfterAnOutOfMemory::NotRunTheGpuMemoryWasShort {
+                reason: "显存不够：卡 1".to_string()
+            })
+        );
+        assert_eq!(cards_outcome_after_an_out_of_memory(2, 3, None), None);
+    }
+
+    /// 第 10z 批（第 10y 批代码轮正推 K3 列的第 7 处）：池里一张卡此刻照这一刻能借的算带得了几个状态，空着时照空着时能借的算。不要卡。
+    #[cfg(feature = "gpu")]
+    #[test]
+    fn a_pool_card_holds_states_now_by_its_room_now_and_when_empty_by_its_ceiling() {
+        assert_eq!(
+            states_in_flight_now_and_when_the_card_is_empty(
+                singlefs_harness::gpu_memory_ledger::CardRoomForThisProcess {
+                    room_now_mebibytes: 100,
+                    ceiling_mebibytes: 1_000,
+                },
+                &|room_mebibytes| room_mebibytes / 10
+            ),
+            (10, 100)
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮正推 K3 列的第 6 处）：借草稿区之前每张卡此刻都带不了一个状态、至少一张空着时带得了，这一步是报本次未跑
+    /// （GPU 用例的记号、原因写显存不够、带每张卡此刻能借的）；空着时也带不了才判红；有一张此刻带得了就去借。不要卡。
+    #[cfg(feature = "gpu")]
+    #[test]
+    fn with_no_card_holding_a_state_now_the_step_before_booking_is_a_not_run_line_not_red() {
+        let run_cards = [CardCapacityOfThisRun {
+            index: 0,
+            name: "造的一张卡".to_string(),
+            max_storage_buffer_binding_size: 1 << 30,
+            memory_budget_mebibytes: crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER
+                + 700,
+            states_in_flight_capacity: 0,
+            states_in_flight_capacity_when_the_card_is_empty: 3,
+        }];
+        match scratch_step_before_booking(ScratchBeforeBooking::NoCardHoldsAStateNow, &run_cards, 900)
+        {
+            ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow { not_run_line } => assert!(
+                not_run_line.starts_with(
+                    "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=显存不够："
+                ) && not_run_line.contains("900 MiB")
+                    && not_run_line.contains("[700]"),
+                "{not_run_line}"
+            ),
+            other @ (ScratchStepBeforeBooking::BookOnTheCards
+            | ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty) => {
+                panic!("此刻借不到：要报本次未跑，判成了 {other:?}")
+            }
+        }
+        assert_eq!(
+            scratch_step_before_booking(
+                ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty,
+                &run_cards,
+                900
+            ),
+            ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty
+        );
+        assert_eq!(
+            scratch_step_before_booking(
+                ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow,
+                &run_cards,
+                900
+            ),
+            ScratchStepBeforeBooking::BookOnTheCards
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮正推 K3 列的第 5 处）：产品路径向卡池借不到时，借到的卡上起设备显存不够与此刻借不到都打本次未跑那一行、
+    /// 交 `Ok`；要的比每张卡空着时都大、卡上的显存读不出判红。不要卡：结局照卡池交回的造。
+    #[cfg(feature = "gpu")]
+    #[test]
+    fn a_device_short_of_gpu_memory_on_the_card_lent_is_reported_not_run_like_no_room_now() {
+        use crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal;
+        assert!(
+            print_the_not_run_line_or_refuse(
+                "单元校验和对拍的缓冲",
+                &CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice {
+                    reason: "显存不够：起设备时".to_string()
+                }
+            )
+            .is_ok_and(|line| line.starts_with("NOT_RUN_NO_GPU_ADAPTER ")
+                && line.ends_with("reason=显存不够：起设备时")),
+            "起设备时显存不够：本次未跑"
+        );
+        assert!(
+            print_the_not_run_line_or_refuse(
+                "单元校验和对拍的缓冲",
+                &CardPoolLeaseRefusal::Booking(BookingRefusal::NoCardHasRoomNow {
+                    need_mebibytes: 900,
+                    card_rooms_mebibytes: vec![700],
+                })
+            )
+            .is_ok_and(|line| line.starts_with("NOT_RUN_NO_GPU_ADAPTER ")),
+            "此刻借不到：本次未跑"
+        );
+        assert!(
+            print_the_not_run_line_or_refuse(
+                "单元校验和对拍的缓冲",
+                &CardPoolLeaseRefusal::Booking(BookingRefusal::NeedLargerThanEveryCard(
+                    NeedLargerThanEveryCard {
+                        need_mebibytes: 40_000,
+                        largest_ceiling_mebibytes: 29_550,
+                    }
+                ))
+            )
+            .is_err(),
+            "比每张卡空着时都大：判红"
+        );
+        assert!(
+            print_the_not_run_line_or_refuse(
+                "单元校验和对拍的缓冲",
+                &CardPoolLeaseRefusal::Booking(BookingRefusal::CardMemoryUnreadable {
+                    reason: "nvidia-smi 起不来".to_string()
+                })
+            )
+            .is_err(),
+            "读不出：判红"
+        );
+    }
+
     /// 一块里第一个判不了的状态：恢复内核的「判不了」位或 checker 内核的「判不了」字，哪一样都算；全判得了是 `None`。
     #[cfg(feature = "gpu")]
     #[test]
@@ -3716,6 +4119,7 @@
                         items_of_each_state: vec![Vec::new(), Vec::new()],
                         red_texts: Vec::new(),
                         gpu_cpu_disagreements: 0,
+                        gpu_memory_was_short: None,
                     })
                     .expect("收的那一头还在等");
             });
@@ -4967,12 +5371,27 @@
                     &stop_is_requested_by_the_environment,
                 )
                 .map_err(|error| format!("核对失败：{error:?}"))?;
+                // 对拍时卡上报了 Out of Memory：打「本次未跑」那一行，照叫停的样子交回（第 10z 批，文件头第 5 条）
+                let gpu_memory_was_short = match &checking.stopped_because_the_gpu_memory_was_short
+                {
+                    Some(reason) => {
+                        println!(
+                            "{}",
+                            singlefs_harness::gpu_memory_ledger::gpu_not_run_line(
+                                PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
+                                reason
+                            )
+                        );
+                        true
+                    }
+                    None => false,
+                };
                 (
                     checking,
                     None,
                     buffers_booked_mebibytes,
                     lease_on_the_books_after_its_work(&buffers_held_until_checked),
-                    false,
+                    gpu_memory_was_short,
                 )
             }
             UnitChecksumsOfThisRun::NotRunTheGpuMemoryWasShort => {
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -6,13 +6,18 @@
 //! GPU 用例共用的卡池（[`gpu_test_start::GpuCardPool`]）不看这个门槛，能不能借由这个进程的显存账在借的那一刻判
 //! （`singlefs_harness::gpu_memory_ledger` 文件头：只记这个进程借的，借不到不等）。
 //! 每个设备起的时候装上 wgpu 没接住的错误与设备丢失的回调（[`DeviceConditionOfACard`]）：别的线程上报的 Out of Memory
-//! （判器内核在另起的线程上编）记在设备上，接 Out of Memory 的那一层照它认出是显存不够（第 10x 批代码轮判决 K1-3）。
+//! （判器内核在另起的线程上编）记在设备上，接 Out of Memory 的那一层照它认出是显存不够（第 10x 批代码轮判决 K1-3），
+//! 按记的那条线程上正在做的那段活认（第 10z 批：别的线程上一次 Out of Memory 不把这段活真正的 panic 说成显存不够；
+//! 编判器内核另起的线程算在那一刻在这个设备上编内核的那段活上，[`GpuCard::pipelines_compiled_once_on_this_device`]）。
 
 use singlefs_checker::{
     crc32_castagnoli_table, unit_checksum_layout, UNIT_HEADER_CHECKSUM_BYTE_OFFSET,
 };
+pub use singlefs_harness::gpu_memory_ledger::DeviceConditionOfACard;
 use singlefs_harness::gpu_memory_ledger::{
-    pci_bus_identifier_without_the_domain, NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
+    card_memory_reader_of_one_report_per_reading, pci_bus_identifier_without_the_domain,
+    BusesWithDevicesStartedOutsideTheBooks, CardMemoryReader,
+    NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
 };
 use singlefs_harness::sha256::DIGEST_BYTES;
 
@@ -101,76 +106,6 @@
     device_condition: std::sync::Arc<DeviceConditionOfACard>,
 }
 
-/// 一个设备上 wgpu 报过的 Out of Memory（错误范围接住的与没接住的都记）与设备丢没丢。接 Out of Memory 的那一层
-/// （崩溃放量产品路径 `crate::crash_amplification`）照它认出：活在别的线程上 panic、或设备丢了，是不是显存不够（第 10x 批代码轮判决 K1-3：
-/// wgpu 的错误范围按线程分，判器内核在另起的线程上编，那里的 Out of Memory 范围接不住）。
-#[derive(Default)]
-pub struct DeviceConditionOfACard {
-    out_of_memory_reports: std::sync::atomic::AtomicU64,
-    last_out_of_memory: std::sync::Mutex<Option<String>>,
-    lost: std::sync::Mutex<Option<String>>,
-}
-
-impl DeviceConditionOfACard {
-    /// 记一次 Out of Memory（`what` 说是在哪、wgpu 报的那一句）。
-    pub fn record_an_out_of_memory(&self, what: String) {
-        // 持锁只为换一个字符串，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
-        *self
-            .last_out_of_memory
-            .lock()
-            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(what);
-        self.out_of_memory_reports
-            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
-    }
-
-    /// 到此刻一共记了几次 Out of Memory。
-    #[must_use]
-    pub fn out_of_memory_reports(&self) -> u64 {
-        self.out_of_memory_reports
-            .load(std::sync::atomic::Ordering::SeqCst)
-    }
-
-    /// 记设备丢了（wgpu 的设备丢失回调调它）。
-    pub fn record_the_device_lost(&self, why: String) {
-        *self
-            .lost
-            .lock()
-            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(why);
-    }
-
-    /// 设备丢了的话为什么丢（没丢是 `None`）。
-    #[must_use]
-    pub fn lost_reason(&self) -> Option<String> {
-        self.lost
-            .lock()
-            .unwrap_or_else(std::sync::PoisonError::into_inner)
-            .clone()
-    }
-
-    /// 从记到 `reports_before` 次那一刻起出了一样坏事、能不能说成是显存不够：这之后又记了 Out of Memory，交最后那一句；
-    /// 设备丢了、丢之前报过 Out of Memory（wgpu 把建管线、提交这类调用上的 Out of Memory 当致命、丢掉设备），交两样都说的一句；别的交 `None`。
-    #[must_use]
-    pub fn out_of_memory_explaining_a_failure_since(&self, reports_before: u64) -> Option<String> {
-        let last_out_of_memory = self
-            .last_out_of_memory
-            .lock()
-            .unwrap_or_else(std::sync::PoisonError::into_inner)
-            .clone();
-        if self.out_of_memory_reports() > reports_before {
-            return Some(
-                last_out_of_memory
-                    .unwrap_or_else(|| "wgpu 报了 Out of Memory（那一句没记下）".to_string()),
-            );
-        }
-        match (self.lost_reason(), last_out_of_memory) {
-            (Some(lost), Some(last_out_of_memory)) => Some(format!(
-                "设备丢了（{lost}），丢之前报过 {last_out_of_memory}"
-            )),
-            (None, Some(_) | None) | (Some(_), None) => None,
-        }
-    }
-}
-
 /// wgpu 没接住的错误（装在每个设备上，[`start_the_card`]）：Out of Memory 先记进这个设备的情况，再与别的错一样照 wgpu 默认的做法 panic
 /// （缓冲、管线没建成，往下跑判出来的也不作数）。
 fn handle_an_uncaptured_wgpu_error(condition: &DeviceConditionOfACard, error: wgpu::Error) {
@@ -208,26 +143,54 @@
 }
 
 /// 这一个设备归谁：卡池的设备走这个进程的显存账；别的（[`gpu_cards_by_priority`]、[`usable_gpu_cards`] 起的）不走，
-/// 起了就记下它在哪张卡上（[`has_devices_started_outside_the_card_pool_on_the_bus`]）。
+/// 起之前就记下它在哪张卡上（[`has_devices_started_outside_the_card_pool_on_the_bus`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum DeviceOwnerOnTheCard {
     TheCardPool,
     OutsideTheCardPool,
 }
 
-/// 这个进程在哪几张卡（PCI 总线号）上起过不走卡池账的设备。只增不减：那几个设备什么时候丢掉、卡上什么时候还，这里不知道，
-/// 记着就照「分不出」算（只会少借）。是这个进程的事实，所以是进程里一份（第 10x 批代码轮判决 K1-1，攻方乙形）。
-static BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL: std::sync::Mutex<
-    std::collections::BTreeSet<String>,
-> = std::sync::Mutex::new(std::collections::BTreeSet::new());
+/// 这个进程在哪几张卡（PCI 总线号）上起过不走卡池账的设备（起之前就记，[`started_with_its_owner_recorded`]）。是这个进程的事实，
+/// 所以是进程里一份（第 10x 批代码轮判决 K1-1，攻方乙形）。
+static BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL: BusesWithDevicesStartedOutsideTheBooks =
+    BusesWithDevicesStartedOutsideTheBooks::new();
 
-/// 这个进程在这张卡（PCI 总线号，[`pci_bus_identifier_without_the_domain`] 写法）上起过不走卡池账的设备没有：起过，卡池读这张卡时
+/// 这个进程在这张卡（PCI 总线号，[`pci_bus_identifier_without_the_domain`] 写法）上起过（或正在起）不走卡池账的设备没有：起过，卡池读这张卡时
 /// 这个进程占着的那一份分不出哪些是借据建的（`singlefs_harness::gpu_memory_ledger::OccupiedByThisProcess::AlsoOutsideTheBooks`）。
 fn has_devices_started_outside_the_card_pool_on_the_bus(bus: &str) -> bool {
-    BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL
-        .lock()
-        .unwrap_or_else(std::sync::PoisonError::into_inner)
-        .contains(bus)
+    BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL.has_devices_on_the_bus(bus)
+}
+
+/// 照设备归谁起（`start` 起设备）：卡池的直接起；不归卡池的先在总线 `bus` 上记下、再起（第 10z 批，第 10y 批代码轮判决 K1-1 攻方乙形：
+/// 起好了才记，起设备到记下那一段里卡池读到的这个进程占着的已经有它，被当成借据建上的）。
+///
+/// # Errors
+/// `start` 的错。
+fn started_with_its_owner_recorded<Device>(
+    owner: DeviceOwnerOnTheCard,
+    bus: &str,
+    start: impl FnOnce() -> Result<Device, CardStartFailure>,
+) -> Result<Device, CardStartFailure> {
+    match owner {
+        DeviceOwnerOnTheCard::TheCardPool => start(),
+        DeviceOwnerOnTheCard::OutsideTheCardPool => {
+            BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL.start_a_device_marked_first(bus, start)
+        }
+    }
+}
+
+/// 卡池读 `buses` 这几张卡显存的那一份：每读一次取一份 `run_the_report` 的报告（卡池是跑一次 `nvidia-smi`），按这个进程的进程号认它占着的，
+/// 这个进程在哪张卡上起过不走卡池账的设备照 [`has_devices_started_outside_the_card_pool_on_the_bus`] 认。
+fn card_memory_reader_of_the_card_pool(
+    run_the_report: Box<dyn Fn() -> Result<String, String> + Send + Sync>,
+    buses: Vec<String>,
+) -> CardMemoryReader {
+    card_memory_reader_of_one_report_per_reading(
+        run_the_report,
+        std::process::id(),
+        buses,
+        Box::new(has_devices_started_outside_the_card_pool_on_the_bus),
+    )
 }
 
 /// `nvidia-smi` 报的一张卡：序号与空闲显存（MiB）。
@@ -327,12 +290,21 @@
 }
 
 /// 在找到的这张卡上起设备、编单元校验和的管线；起不来交为什么（显存不够与别的分开，[`CardStartFailure`]）。不看空闲显存（看不看由调用方定）。
-/// 设备上装好接 wgpu 没接住的错误与设备丢失的回调（[`DeviceConditionOfACard`]）；不归卡池的设备起好了记下它在哪张卡上
-/// （[`has_devices_started_outside_the_card_pool_on_the_bus`]）。
+/// 设备上装好接 wgpu 没接住的错误与设备丢失的回调（[`DeviceConditionOfACard`]）；不归卡池的设备起之前记下它在哪张卡上
+/// （[`started_with_its_owner_recorded`]、[`has_devices_started_outside_the_card_pool_on_the_bus`]）。
 fn start_the_card(
     discovered: &DiscoveredCard,
     owner: DeviceOwnerOnTheCard,
 ) -> Result<GpuCard, CardStartFailure> {
+    started_with_its_owner_recorded(owner, &discovered.pci_bus_identifier, || {
+        start_the_device_and_the_pipeline(discovered)
+    })
+}
+
+/// [`start_the_card`] 起设备、编管线的那一段（不管设备归谁）。
+fn start_the_device_and_the_pipeline(
+    discovered: &DiscoveredCard,
+) -> Result<GpuCard, CardStartFailure> {
     let which_card = format!(
         "序号 {} 的卡（{}）",
         discovered.reported.index, discovered.name
@@ -377,15 +349,6 @@
         device_condition.record_an_out_of_memory(reason.clone());
         return Err(CardStartFailure::OutOfGpuMemory { reason });
     }
-    match owner {
-        DeviceOwnerOnTheCard::TheCardPool => {}
-        DeviceOwnerOnTheCard::OutsideTheCardPool => {
-            BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL
-                .lock()
-                .unwrap_or_else(std::sync::PoisonError::into_inner)
-                .insert(discovered.pci_bus_identifier.clone());
-        }
-    }
     Ok(GpuCard {
         name: discovered.name.clone(),
         pci_bus_identifier: discovered.pci_bus_identifier.clone(),
@@ -551,6 +514,8 @@
     /// 这个设备上按 `source_digest` 编好的那组管线：这个设备（这张卡与它的每个克隆）上第一次要这个摘要时调 `compile` 编，
     /// 别的线程同时要同一个摘要就等这一次编完、拿同一组；以后再要直接拿。管线绑在设备上，另起的设备（另一次 [`usable_gpu_cards`]）另有一份。
     /// 冷编一套判器内核要几分钟、驱动编的时候吃几 GiB 主机内存：几十条用例共用一张卡时，每条各编一遍既慢又撞主机内存上限。
+    /// 编的那一段在设备上记着是为这条线程上的哪段活编的（[`DeviceConditionOfACard::compile_for_the_work_of_this_thread`]）：
+    /// `compile` 另起的线程上没接住的 Out of Memory 算那段活的。
     ///
     /// # Panics
     /// `compile` panic：原样传出，这一格照旧空着，下一个要这个摘要的再编一次。
@@ -568,7 +533,12 @@
                 .unwrap_or_else(std::sync::PoisonError::into_inner);
             std::sync::Arc::clone(by_source_digest.entry(source_digest).or_default())
         };
-        compiled_once.get_or_init(compile).clone()
+        compiled_once
+            .get_or_init(|| {
+                self.device_condition
+                    .compile_for_the_work_of_this_thread(compile)
+            })
+            .clone()
     }
 
     /// 一批字节段（每段不超过 32 KiB）在这张卡上算 CRC-32C，按次序交回。
@@ -1129,7 +1099,7 @@
 
     mod card_pool {
         use super::super::{
-            has_devices_started_outside_the_card_pool_on_the_bus, memory_budget_mebibytes_of,
+            card_memory_reader_of_the_card_pool, memory_budget_mebibytes_of,
             nvidia_discrete_cards_reported_by_nvidia_smi, nvidia_smi_memory_and_processes_report,
             start_the_card, CardStartFailure, DeviceOwnerOnTheCard, DiscoveredCard, GpuCard,
         };
@@ -1143,10 +1113,9 @@
             CardHost,
         };
         use singlefs_harness::gpu_memory_ledger::{
-            card_ledger_capacity_mebibytes, card_memory_reader_of_one_report_per_reading,
-            reason_for_a_booking_without_room_now, reason_for_running_out_of_gpu_memory,
-            BookingRefusal, CardRoomForThisProcess, DeviceStartedOnDemand, GpuMemoryBooking,
-            GpuMemoryLedger, StatesBookedOnACard,
+            card_ledger_capacity_mebibytes, reason_for_a_booking_without_room_now,
+            reason_for_running_out_of_gpu_memory, BookingRefusal, CardRoomForThisProcess,
+            DeviceStartedOnDemand, GpuMemoryBooking, GpuMemoryLedger, StatesBookedOnACard,
         };
         use singlefs_harness::numeric_environment_variable::{
             number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
@@ -1269,9 +1238,11 @@
 
             /// 借到 `booking` 的那张卡起好的设备：留着的设备没丢就拿它，没有（第一次要、上一次起不来）或丢了就起一个
             /// （借到就登记了这个进程，先登记、后建设备）；起不来交为什么、不留着。要一笔借据才起得了：没借过的卡起不了设备。
+            /// 丢了的设备重起之前，驱动那一份在账上重新登记，此刻放不下就不起、交显存不够
+            /// （`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`，第 10z 批）。
             ///
             /// # Panics
-            /// 这个进程还没在这张卡的账上登记（`GpuMemoryLedger::check_registered_before_starting_the_device`）。
+            /// 这个进程还没在这张卡的账上登记（`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`）。
             fn started_card(
                 &self,
                 booking: &GpuMemoryBooking<'static>,
@@ -1281,8 +1252,29 @@
                 card.started.started_or_start(
                     |started: &GpuCard| started.device_condition().lost_reason().is_some(),
                     || {
+                        let which_card = format!(
+                            "序号 {} 的卡（{}）",
+                            card.discovered.reported.index, card.discovered.name
+                        );
                         self.ledger
-                            .check_registered_before_starting_the_device(card_position);
+                            .prepare_the_driver_share_before_starting_the_device(card_position)
+                            .map_err(|refusal| match refusal {
+                                BookingRefusal::NoCardHasRoomNow { .. } => {
+                                    CardStartFailure::OutOfGpuMemory {
+                                        reason: format!(
+                                            "{which_card}上丢了的设备要重起，账上此刻放不下新设备的驱动那一份（{refusal:?}）"
+                                        ),
+                                    }
+                                }
+                                BookingRefusal::NeedLargerThanEveryCard(_)
+                                | BookingRefusal::CardMemoryUnreadable { .. } => {
+                                    CardStartFailure::DeviceNotStarted {
+                                        reason: format!(
+                                            "{which_card}上丢了的设备要重起，账上重新登记驱动那一份时 {refusal:?}"
+                                        ),
+                                    }
+                                }
+                            })?;
                         let mut started =
                             start_the_card(&card.discovered, DeviceOwnerOnTheCard::TheCardPool)?;
                         self.ledger
@@ -1679,11 +1671,9 @@
             // 每次借读一份 nvidia-smi 报告，空闲与这个进程占着的出自同一份（第 10x 批代码轮判决 K1-1）
             let ledger = GpuMemoryLedger::new(
                 named.len(),
-                card_memory_reader_of_one_report_per_reading(
+                card_memory_reader_of_the_card_pool(
                     Box::new(nvidia_smi_memory_and_processes_report),
-                    std::process::id(),
                     buses,
-                    Box::new(has_devices_started_outside_the_card_pool_on_the_bus),
                 ),
                 Box::new(move |position, may_hold_mebibytes| {
                     let configured = configured_card_quotas[position];
@@ -1797,9 +1787,12 @@
         PciNvidiaDisplayControllers, PCI_DEVICES_SUBDIRECTORY_NAME,
     };
     use super::{
-        failure_of_a_device_request, handle_an_uncaptured_wgpu_error, CardStartFailure,
-        DeviceConditionOfACard,
+        card_memory_reader_of_the_card_pool, failure_of_a_device_request,
+        handle_an_uncaptured_wgpu_error, has_devices_started_outside_the_card_pool_on_the_bus,
+        started_with_its_owner_recorded, CardStartFailure, DeviceConditionOfACard,
+        DeviceOwnerOnTheCard,
     };
+    use singlefs_harness::gpu_memory_ledger::OccupiedByThisProcess;
 
     /// 假的 sysfs 里 PCI 总线那一截的样子。
     enum FakePciBus<'devices> {
@@ -1969,38 +1962,77 @@
         assert_eq!(condition.out_of_memory_reports(), 1, "校验错不记成显存不够");
     }
 
-    /// 第 10x 批代码轮判决 K1-3（攻方 R2、R4）：从某一刻起又记了 Out of Memory，或设备丢了而丢之前报过 Out of Memory，说得成显存不够；
-    /// 什么都没记、或设备丢了而从没报过 Out of Memory，说不成。
+    /// 第 10z 批（第 10y 批代码轮判决 K1-1 攻方乙形、K3）：不归卡池的设备起之前就在它的总线上记下（起的那一段里卡池已经认得出），
+    /// 卡池的设备不记。总线号是造的，不与本机的卡重。
     #[test]
-    fn a_failure_is_put_down_to_gpu_memory_only_after_an_out_of_memory_on_the_device() {
-        let condition = DeviceConditionOfACard::default();
-        assert_eq!(condition.out_of_memory_explaining_a_failure_since(0), None);
-        condition.record_the_device_lost("Unknown：驱动重置".to_string());
+    fn a_device_outside_the_card_pool_is_marked_on_its_bus_before_it_starts_and_a_pool_device_is_not(
+    ) {
         assert_eq!(
-            condition.out_of_memory_explaining_a_failure_since(0),
-            None,
-            "设备丢了而从没报过 Out of Memory：不是显存不够"
-        );
-        let fresh = DeviceConditionOfACard::default();
-        let reports_before = fresh.out_of_memory_reports();
-        fresh.record_an_out_of_memory("另起的线程上编恢复内核时没接住的 Out of Memory".to_string());
+            started_with_its_owner_recorded(
+                DeviceOwnerOnTheCard::OutsideTheCardPool,
+                "fe:00.0",
+                || {
+                    Ok(has_devices_started_outside_the_card_pool_on_the_bus(
+                        "fe:00.0",
+                    ))
+                }
+            ),
+            Ok(true),
+            "不归卡池：起的那一段里已经记着"
+        );
         assert_eq!(
-            fresh.out_of_memory_explaining_a_failure_since(reports_before),
-            Some("另起的线程上编恢复内核时没接住的 Out of Memory".to_string()),
-            "这之后记了 Out of Memory：说成显存不够"
+            started_with_its_owner_recorded(DeviceOwnerOnTheCard::TheCardPool, "fd:00.0", || {
+                Ok(has_devices_started_outside_the_card_pool_on_the_bus(
+                    "fd:00.0",
+                ))
+            }),
+            Ok(false),
+            "卡池的：不记"
         );
-        let reports_after = fresh.out_of_memory_reports();
+        assert!(!has_devices_started_outside_the_card_pool_on_the_bus(
+            "fd:00.0"
+        ));
+    }
+
+    /// 第 10z 批（第 10y 批代码轮攻方 K3：认出不走卡池设备的接线没有测试）：卡池读显存的那一份照这个进程起过不归卡池设备的总线认，
+    /// 那张卡上这个进程占着的记成分不出；别的卡照都走账。报告是造的，总线号不与本机的卡重。
+    #[test]
+    fn the_card_pool_reads_this_process_as_also_outside_the_books_on_a_bus_with_a_device_outside_the_pool(
+    ) {
         assert_eq!(
-            fresh.out_of_memory_explaining_a_failure_since(reports_after),
-            None,
-            "这之后没再记、设备也没丢：说不成"
+            started_with_its_owner_recorded(
+                DeviceOwnerOnTheCard::OutsideTheCardPool,
+                "fc:00.0",
+                || { Ok(()) }
+            ),
+            Ok(())
         );
-        fresh.record_the_device_lost("Unknown：Out of memory".to_string());
-        assert!(
-            fresh
-                .out_of_memory_explaining_a_failure_since(reports_after)
-                .is_some(),
-            "设备丢了、丢之前报过 Out of Memory：说成显存不够"
+        let this_process = std::process::id();
+        let report = ["00000000:FC:00.0", "00000000:FB:00.0"]
+            .iter()
+            .map(|bus| {
+                format!(
+                    "GPU {bus}\n    FB Memory Usage\n        Total                                          : 16311 MiB\n\
+                     \x20       Reserved                                       : 462 MiB\n\
+                     \x20       Free                                           : 10000 MiB\n    Processes\n\
+                     \x20       Process ID                                     : {this_process}\n\
+                     \x20           Used GPU Memory                            : 1000 MiB\n"
+                )
+            })
+            .collect::<String>();
+        let read = card_memory_reader_of_the_card_pool(
+            Box::new(move || Ok(report.clone())),
+            vec!["fc:00.0".to_string(), "fb:00.0".to_string()],
+        );
+        assert_eq!(
+            read().map(|readings| readings
+                .iter()
+                .map(|reading| reading.occupied_by_this_process)
+                .collect::<Vec<OccupiedByThisProcess>>()),
+            Ok(vec![
+                OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 1000 },
+                OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 1000 },
+            ])
         );
     }
 }
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -22,29 +22,45 @@
 //! - F − 这个进程许出去而还没占上卡的那一份：许出去的是 B 连驱动那一份 D（还没登记时 B 是 0、D 照样要放得下），
 //!   减实际占着的里属于许出去的那部分。实际占着的先扣掉这个进程已经销账、卡上可能还没还的那一份
 //!   （[`BookedByThisProcessOnACard::returned_possibly_still_on_the_card_mebibytes`]，第 10x 批代码轮判决 K1-1：不当成别的借据已经建上的）；
+//!   那一份每读一次照「实际占着的减早先读数见到占在卡上、此刻还活着的那一份」往下压（[`returned_possibly_still_on_the_card_after_a_reading`]），
+//!   不照「驱动那一份整份在卡上」压（第 10y 批代码轮判决 K1-1 攻方丙形：驱动实占的 x 比 D 小时压过头，多给 D − x），也不只看这一次读数
+//!   （第 10y 批代码轮判决 K2-2 P1：有长借据建着时，已经还给驱动的短借据一笔一笔累积下来压不掉，每还一笔多扣一份）。
 //!   读不到实际占着的（`nvidia-smi` 没列这个进程），或这个进程在这张卡上另有不走这份账的设备（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]，
-//!   分不出哪一份是借据建的），借着的当一点都没建、起了设备的驱动那一份当已经在卡上，只会少借。
+//!   分不出哪一份是借据建的），借着的当一点都没建、驱动那一份也当还没占上卡（第 10y 批代码轮判决 K1-1 攻方乙附：当它整份在卡上，实占的 x 比 D 小时
+//!   多给 D − x），只会少借。
+//!   不走这份账的设备在起之前就记下它在哪张卡上（[`BusesWithDevicesStartedOutsideTheBooks::start_a_device_marked_first`]，第 10y 批代码轮判决 K1-1
+//!   攻方乙形：起好了才记，起设备到记下之间那一段读数里已经有那个设备的占用，被当成借据建上的）。
 //!
 //! 借的要比这张卡空着时能借的（同一个算法拿空卡上能用的当 M、B 当 0；空卡上能用的是 `nvidia-smi` 报的总量减驱动保留的那一份，
 //! 2026-10-07 本机两张空卡上逐 MiB 等于报的空闲：5090 是 32607 − 497 = 32110，5060 Ti 是 16311 − 462 = 15849；第 10x 批代码轮判决 K1-1
 //! 「上界不用总量」）还大，等多久都放不下，调用方判红（配了 GPU 就一定用 GPU）；
 //! 比此刻能借的大而空着时放得下，是借不到：不等，调用方按借不到走下一条（几张卡一起借的换后面的卡，一张都借不到报本次未跑，
 //! [`scratch_booking_refusal_outcome`]、[`GPU_NOT_RUN_MARKER`]）。
-//! 账不防的：两个进程同一刻读到同一份空闲、一起借超了（各自只记自己借的）。这一形由运行时兜底：内存包装
+//! 账不防的：两个进程同一刻读到同一份空闲、一起借超了（各自只记自己借的）；一份 `nvidia-smi -q` 报告里空闲与这个进程占着的是驱动的两次查询，
+//! 本机两张卡的两次读显存相隔约 1.3 毫秒（第 10y 批代码轮攻方量过 1347.9 µs），这个进程别的线程恰在两次之间建上的那一笔既在空闲里又在占着的里
+//! （第 10y 批代码轮判决 K1-1：与两进程同读一份空闲同形，归运行时兜底，不另改）。这两形由运行时兜底：内存包装
 //! `scripts/hosts/run-with-memory-cap.sh` 认出内核日志里的 `NVRM … NV_ERR_NO_MEMORY` 退 245，wgpu 报 Out of Memory 时
 //! 崩溃放量产品路径与卡池起设备把这一趟报本次未跑（原因写显存不够，[`GPU_MEMORY_SHORT_REASON_PREFIX`]、
 //! [`reason_for_running_out_of_gpu_memory`]）。
 //!
 //! 先登记、后建设备（这个进程里）：一个进程在一张卡上第一次借时，这一笔连同它的驱动那一份放得下才记、才把这张卡记成登记了；
-//! 调用方拿到借据之后才在这张卡上起设备（[`GpuMemoryLedger::check_registered_before_starting_the_device`]），
-//! 起好了记一笔（[`GpuMemoryLedger::record_the_device_started_on_the_card`]），从此驱动那一份算实际占在卡上。
+//! 调用方拿到借据之后才在这张卡上起设备（[`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`]），
+//! 起好了记一笔（[`GpuMemoryLedger::record_the_device_started_on_the_card`]），从此驱动那一份算实际占在卡上。设备丢了要重起时，驱动那一份记回
+//! 许出去而还没占上卡、旧设备那一份当销了账而卡上可能还没还的，此刻放不下就不起（第 10y 批代码轮判决 K1-3：重起不另登记驱动那一份，
+//! 新设备的上下文没留地方）。
+//!
+//! wgpu 在卡上报的 Out of Memory 记在设备上（[`DeviceConditionOfACard`]），按记的那条线程上正在做的那段活认（[`WorkOnTheCardOfThisThread`]）：
+//! 别的线程上一次 Out of Memory 不把这条线程上真正的 panic 说成显存不够（第 10y 批代码轮判决 K1-3 攻方）；判器内核在活另起的线程上编，
+//! 那几条线程上记的算在那一刻这个设备上在编内核的那几段活上（[`DeviceConditionOfACard::compile_for_the_work_of_this_thread`]）。
 //! 崩溃放量产品路径在起过卡池的进程里也走这份账：第 ① 段每张装了状态的卡按装的状态数借草稿区（[`scratch_states_to_book_on_each_card`]，
 //! 只记在那张卡上），几张卡当场各试一次（[`GpuMemoryLedger::book_states_on_the_cards_with_room`]），每张在锁里照此刻能借的压状态数，
 //! 借不到的卡不装状态、它那一份留给后面的卡（[`place_states_on_the_cards_that_book_now`]）；第 ② 段核对与单元校验和在卡上建的缓冲
 //! 照 [`mebibytes_of_buffers_held_at_once`] 算，借第一张放得下的。没起过卡池的进程照旧不进账，是哪几种调用方写在
 //! `singlefs_checker_tier::crash_amplification` 的文件头。
 
-use std::collections::BTreeMap;
+use std::cell::Cell;
+use std::collections::{BTreeMap, BTreeSet};
+use std::marker::PhantomData;
 use std::sync::atomic::{AtomicU64, Ordering};
 use std::sync::{Mutex, MutexGuard, PoisonError};
 
@@ -142,14 +158,31 @@
 }
 
 /// 这个进程此刻能在一张卡上占多少（MiB，文件头「口径」里的 M）：现读的空闲，加上这个进程实际占着的里属于它许出去的那一部分
-/// （借着的，加登记了时驱动那一份）。实际占着的先扣掉已经销账、卡上可能还没还的那一份；比许出去的多（分配器按块拿）只算到许出去的那么多；
-/// 读不到实际占着的、或这个进程在这张卡上另有不走这份账的设备，借着的当一点都没建，起好了设备的驱动那一份当已经在卡上。
+/// （[`occupied_part_of_what_this_process_promised_mebibytes`]）。
 #[must_use]
 pub fn memory_this_process_may_hold_mebibytes(
     reading: CardMemoryReading,
     booked: BookedByThisProcessOnACard,
     mebibytes_left_for_the_driver: u64,
 ) -> u64 {
+    reading
+        .free_mebibytes
+        .saturating_add(occupied_part_of_what_this_process_promised_mebibytes(
+            reading,
+            booked,
+            mebibytes_left_for_the_driver,
+        ))
+}
+
+/// 这个进程实际占在一张卡上的里，属于它许出去的那一部分（MiB；许出去的是借着的，加登记了时驱动那一份）：实际占着的先扣掉已经销账、
+/// 卡上可能还没还的那一份；比许出去的多（分配器按块拿）只算到许出去的那么多；读不到实际占着的、或这个进程在这张卡上另有不走这份账的设备，
+/// 是 0：借着的当一点都没建，驱动那一份也当还没占上卡（驱动实占多少分不出）。
+#[must_use]
+pub fn occupied_part_of_what_this_process_promised_mebibytes(
+    reading: CardMemoryReading,
+    booked: BookedByThisProcessOnACard,
+    mebibytes_left_for_the_driver: u64,
+) -> u64 {
     let promised_mebibytes = match booked.driver_share {
         DriverShareOnTheCard::NotRegistered => booked.leases_mebibytes,
         DriverShareOnTheCard::RegisteredDeviceNotStarted | DriverShareOnTheCard::DeviceStarted => {
@@ -158,43 +191,30 @@
                 .saturating_add(mebibytes_left_for_the_driver)
         }
     };
-    let occupied_and_promised_mebibytes = match reading.occupied_by_this_process {
+    match reading.occupied_by_this_process {
         OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => mebibytes
             .saturating_sub(booked.returned_possibly_still_on_the_card_mebibytes)
             .min(promised_mebibytes),
-        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {
-            match booked.driver_share {
-                DriverShareOnTheCard::DeviceStarted => {
-                    mebibytes_left_for_the_driver.min(promised_mebibytes)
-                }
-                DriverShareOnTheCard::NotRegistered
-                | DriverShareOnTheCard::RegisteredDeviceNotStarted => 0,
-            }
-        }
-    };
-    reading
-        .free_mebibytes
-        .saturating_add(occupied_and_promised_mebibytes)
+        // 驱动那一份当整份在卡上，实占的 x 比 D 小时多给 D − x（第 10y 批代码轮判决 K1-1 攻方乙附）：驱动之后还会长到 D
+        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => 0,
+    }
 }
 
-/// 销了账、卡上可能还没还的那一份照这一次读数往下压（MiB）：这个进程实际占着的里，起好了设备的驱动那一份之外才可能是它，
-/// 压到不超过那么多；读不到实际占着的、或另有不走这份账的设备，照旧（分不出就不压）。
+/// 销了账、卡上可能还没还的那一份照这一次读数往下压（MiB）：这个进程实际占着的里，早先读数已经见到占在卡上、此刻还算数的那一份
+/// （`seen_on_the_card_at_least_mebibytes`：驱动那一份与那一刻活着、此刻还活着的借据建上的，[`GpuMemoryLedger`] 每读一次记）
+/// 之外才可能是它，压到不超过那么多；读不到实际占着的、或另有不走这份账的设备，照旧（分不出就不压）。
+/// 不照「驱动那一份 D 已经整份在卡上」压：驱动实占的 x 比 D 小时压过头，被压掉的那截在下一笔里当成借据建上的，多给 D − x
+/// （第 10y 批代码轮判决 K1-1 攻方丙形）。
 #[must_use]
 pub fn returned_possibly_still_on_the_card_after_a_reading(
     returned_possibly_still_on_the_card_mebibytes: u64,
     occupied_by_this_process: OccupiedByThisProcess,
-    driver_share: DriverShareOnTheCard,
-    mebibytes_left_for_the_driver: u64,
+    seen_on_the_card_at_least_mebibytes: u64,
 ) -> u64 {
     match occupied_by_this_process {
         OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => {
-            let driver_on_the_card_mebibytes = match driver_share {
-                DriverShareOnTheCard::DeviceStarted => mebibytes_left_for_the_driver,
-                DriverShareOnTheCard::NotRegistered
-                | DriverShareOnTheCard::RegisteredDeviceNotStarted => 0,
-            };
             returned_possibly_still_on_the_card_mebibytes
-                .min(mebibytes.saturating_sub(driver_on_the_card_mebibytes))
+                .min(mebibytes.saturating_sub(seen_on_the_card_at_least_mebibytes))
         }
         OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {
             returned_possibly_still_on_the_card_mebibytes
@@ -810,9 +830,60 @@
     driver_share: DriverShareOnTheCard,
     /// 销了账、卡上可能还没还的（[`BookedByThisProcessOnACard::returned_possibly_still_on_the_card_mebibytes`]）。
     returned_possibly_still_on_the_card_mebibytes: u64,
+    /// 早先一次读数见到占在卡上的、这个进程许出去的那一份（MiB，[`occupied_part_of_what_this_process_promised_mebibytes`]：驱动那一份与借据建上的）
+    /// 里此刻还算数的：那一刻活着的借据（`leases_seen_on_the_card`）销了账就减掉它借的量，那一刻之后借的销账不减。只拿来压销了账、
+    /// 卡上可能还没还的那一份（[`returned_possibly_still_on_the_card_after_a_reading`]，第 10y 批代码轮判决 K2-2 P1）。
+    seen_on_the_card_at_least_mebibytes: u64,
+    leases_seen_on_the_card: BTreeSet<u64>,
 }
 
 impl BooksOfACard {
+    /// 一张还没借过的卡的账。
+    fn empty() -> Self {
+        Self {
+            leases: BTreeMap::new(),
+            driver_share: DriverShareOnTheCard::NotRegistered,
+            returned_possibly_still_on_the_card_mebibytes: 0,
+            seen_on_the_card_at_least_mebibytes: 0,
+            leases_seen_on_the_card: BTreeSet::new(),
+        }
+    }
+
+    /// 读到一次这张卡的显存：销了账、卡上可能还没还的那一份先照它往下压，再看这一次见到占在卡上的许出去的那一份，
+    /// 比记着的多就换成它、连同这一刻活着的借据。
+    fn take_in_a_reading(
+        &mut self,
+        reading: CardMemoryReading,
+        mebibytes_left_for_the_driver: u64,
+    ) {
+        self.returned_possibly_still_on_the_card_mebibytes =
+            returned_possibly_still_on_the_card_after_a_reading(
+                self.returned_possibly_still_on_the_card_mebibytes,
+                reading.occupied_by_this_process,
+                self.seen_on_the_card_at_least_mebibytes,
+            );
+        let seen_now_mebibytes = occupied_part_of_what_this_process_promised_mebibytes(
+            reading,
+            self.booked(),
+            mebibytes_left_for_the_driver,
+        );
+        if seen_now_mebibytes > self.seen_on_the_card_at_least_mebibytes {
+            self.seen_on_the_card_at_least_mebibytes = seen_now_mebibytes;
+            self.leases_seen_on_the_card = self.leases.keys().copied().collect();
+        }
+    }
+
+    /// 这张卡上起过的设备丢了、要重起：驱动那一份记回许出去而还没占上卡，旧设备那一份（`mebibytes_left_for_the_driver`）当销了账而卡上
+    /// 可能还没还的，早先读数见到的作废（见到的是旧设备上的）。
+    fn take_back_the_driver_share_of_a_lost_device(&mut self, mebibytes_left_for_the_driver: u64) {
+        self.driver_share = DriverShareOnTheCard::RegisteredDeviceNotStarted;
+        self.returned_possibly_still_on_the_card_mebibytes = self
+            .returned_possibly_still_on_the_card_mebibytes
+            .saturating_add(mebibytes_left_for_the_driver);
+        self.seen_on_the_card_at_least_mebibytes = 0;
+        self.leases_seen_on_the_card.clear();
+    }
+
     fn booked(&self) -> BookedByThisProcessOnACard {
         BookedByThisProcessOnACard {
             leases_mebibytes: self.leases.values().sum(),
@@ -822,13 +893,18 @@
         }
     }
 
-    /// 销一笔借据（借据丢掉时）：它建的缓冲记进卡上可能还没还的那一份，交回销没销到。
+    /// 销一笔借据（借据丢掉时）：它建的缓冲记进卡上可能还没还的那一份；早先读数见到的那一份里有它的话减掉它借的量。交回销没销到。
     fn return_a_lease(&mut self, lease_identifier: u64) -> bool {
         match self.leases.remove(&lease_identifier) {
             Some(mebibytes) => {
                 self.returned_possibly_still_on_the_card_mebibytes = self
                     .returned_possibly_still_on_the_card_mebibytes
                     .saturating_add(mebibytes);
+                if self.leases_seen_on_the_card.remove(&lease_identifier) {
+                    self.seen_on_the_card_at_least_mebibytes = self
+                        .seen_on_the_card_at_least_mebibytes
+                        .saturating_sub(mebibytes);
+                }
                 true
             }
             None => false,
@@ -886,15 +962,7 @@
             read_card_memory,
             capacity_for_this_process,
             mebibytes_each_process_keeps_on_a_card,
-            books: Mutex::new(
-                (0..card_count)
-                    .map(|_| BooksOfACard {
-                        leases: BTreeMap::new(),
-                        driver_share: DriverShareOnTheCard::NotRegistered,
-                        returned_possibly_still_on_the_card_mebibytes: 0,
-                    })
-                    .collect(),
-            ),
+            books: Mutex::new((0..card_count).map(|_| BooksOfACard::empty()).collect()),
             next_lease_identifier: AtomicU64::new(0),
         }
     }
@@ -904,13 +972,12 @@
         self.books.lock().unwrap_or_else(PoisonError::into_inner)
     }
 
-    /// 在锁里现读每张卡，算这几张卡此刻各能借给这个进程的（与 `card_positions` 逐张对上）；每张卡销了账、卡上可能还没还的那一份
-    /// 先照这一次读数往下压（[`returned_possibly_still_on_the_card_after_a_reading`]）。
-    fn rooms_of_the_cards(
+    /// 在锁里现读每张卡（次序与池一样），每张卡的账照这一次读数记（`BooksOfACard::take_in_a_reading`：销了账、卡上可能还没还的那一份往下压，
+    /// 见到占在卡上的那一份往上换）。
+    fn readings_taken_into_the_books(
         &self,
         books: &mut [BooksOfACard],
-        card_positions: &[usize],
-    ) -> Result<Vec<CardRoomForThisProcess>, BookingRefusal> {
+    ) -> Result<Vec<CardMemoryReading>, BookingRefusal> {
         let readings = (self.read_card_memory)()
             .map_err(|reason| BookingRefusal::CardMemoryUnreadable { reason })?;
         if readings.len() != books.len() {
@@ -923,14 +990,19 @@
             });
         }
         for (books_of_the_card, reading) in books.iter_mut().zip(&readings) {
-            books_of_the_card.returned_possibly_still_on_the_card_mebibytes =
-                returned_possibly_still_on_the_card_after_a_reading(
-                    books_of_the_card.returned_possibly_still_on_the_card_mebibytes,
-                    reading.occupied_by_this_process,
-                    books_of_the_card.driver_share,
-                    self.mebibytes_each_process_keeps_on_a_card,
-                );
+            books_of_the_card
+                .take_in_a_reading(*reading, self.mebibytes_each_process_keeps_on_a_card);
         }
+        Ok(readings)
+    }
+
+    /// 在锁里现读每张卡（[`Self::readings_taken_into_the_books`]），算这几张卡此刻各能借给这个进程的（与 `card_positions` 逐张对上）。
+    fn rooms_of_the_cards(
+        &self,
+        books: &mut [BooksOfACard],
+        card_positions: &[usize],
+    ) -> Result<Vec<CardRoomForThisProcess>, BookingRefusal> {
+        let readings = self.readings_taken_into_the_books(books)?;
         Ok(card_positions
             .iter()
             .map(|card_position| {
@@ -1200,7 +1272,55 @@
         );
     }
 
-    /// 第 `card_position` 张卡上的设备起好了：从此这个进程的驱动那一份算实际占在卡上（读不到实际占着的时候照它算）。
+    /// 在第 `card_position` 张卡上起设备之前调（先登记、后建设备）：这个进程要已经在这张卡上借过一笔。设备是头一回起（或上一回没起来），
+    /// 驱动那一份在登记的那一笔里一起借过，照旧；起过的设备丢了要重起（驱动那一份记着已经占在卡上），把它记回许出去而还没占上卡、
+    /// 旧设备那一份当销了账而卡上可能还没还的（`BooksOfACard::take_back_the_driver_share_of_a_lost_device`），在锁里现读一次：
+    /// 借着的连新设备的驱动那一份此刻放不下，交借不到，调用方不起（第 10y 批代码轮判决 K1-3 攻方：重起不另登记驱动那一份，
+    /// 触发重起的那一笔是按驱动那一份已在卡上借的，新设备的上下文没留地方）。交借不到之后驱动那一份照旧记着许出去而没占上卡。
+    ///
+    /// # Errors
+    /// 重起而此刻放不下（[`BookingRefusal::NoCardHasRoomNow`]，要的是驱动那一份，带着这张卡此刻能借的）；卡上的显存读不出。
+    ///
+    /// # Panics
+    /// 同 [`Self::check_registered_before_starting_the_device`]。
+    pub fn prepare_the_driver_share_before_starting_the_device(
+        &self,
+        card_position: usize,
+    ) -> Result<(), BookingRefusal> {
+        let mut books = self.books_locked();
+        match books[card_position].driver_share {
+            DriverShareOnTheCard::NotRegistered => panic!(
+                "第 {card_position} 张卡：这个进程在它的账上登记之前就要起设备（先登记、后建设备）"
+            ),
+            DriverShareOnTheCard::RegisteredDeviceNotStarted => Ok(()),
+            DriverShareOnTheCard::DeviceStarted => {
+                books[card_position].take_back_the_driver_share_of_a_lost_device(
+                    self.mebibytes_each_process_keeps_on_a_card,
+                );
+                let readings = self.readings_taken_into_the_books(&mut books)?;
+                let booked = books[card_position].booked();
+                let capacity_mebibytes = (self.capacity_for_this_process)(
+                    card_position,
+                    memory_this_process_may_hold_mebibytes(
+                        readings[card_position],
+                        booked,
+                        self.mebibytes_each_process_keeps_on_a_card,
+                    ),
+                );
+                if capacity_mebibytes < booked.leases_mebibytes {
+                    return Err(BookingRefusal::NoCardHasRoomNow {
+                        need_mebibytes: self.mebibytes_each_process_keeps_on_a_card,
+                        card_rooms_mebibytes: vec![
+                            capacity_mebibytes.saturating_sub(booked.leases_mebibytes)
+                        ],
+                    });
+                }
+                Ok(())
+            }
+        }
+    }
+
+    /// 第 `card_position` 张卡上的设备起好了：从此这个进程的驱动那一份算实际占在卡上（读到实际占着的时候照它算）。
     ///
     /// # Panics
     /// 同 [`Self::check_registered_before_starting_the_device`]。
@@ -1333,10 +1453,256 @@
     }
 }
 
+/// 这个进程在哪几张卡（PCI 总线号，[`pci_bus_identifier_without_the_domain`] 写法）上起过不走这份账的设备（崩溃放量没起卡池时自己起的、
+/// 经入口直接起的）：卡池读这张卡时这个进程占着的那一份分不出哪些是借据建的（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]）。
+/// 只增不减：那几个设备什么时候丢掉、卡上什么时候还，这里不知道，记着就照「分不出」算（只会少借）。
+pub struct BusesWithDevicesStartedOutsideTheBooks {
+    buses: Mutex<BTreeSet<String>>,
+}
+
+/// 空着（一个不走账的设备都没起过）就是起点，没有别的不变量。
+impl Default for BusesWithDevicesStartedOutsideTheBooks {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+impl BusesWithDevicesStartedOutsideTheBooks {
+    #[must_use]
+    pub const fn new() -> Self {
+        Self {
+            buses: Mutex::new(BTreeSet::new()),
+        }
+    }
+
+    /// 先把 `bus` 记成起过不走账的设备，再调 `start` 起，交 `start` 的结果：起设备到记下之间那一段，卡池读到的这个进程占着的已经有
+    /// 那个设备的那一份，后记就被当成借据建上的（第 10y 批代码轮判决 K1-1 攻方乙形）。起不来的照样记着（分不出，只会少借）。
+    ///
+    /// # Errors
+    /// `start` 的错。
+    pub fn start_a_device_marked_first<Device, Failure>(
+        &self,
+        bus: &str,
+        start: impl FnOnce() -> Result<Device, Failure>,
+    ) -> Result<Device, Failure> {
+        // 持锁只为插一个总线号，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
+        self.buses
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .insert(bus.to_string());
+        start()
+    }
+
+    /// 这个进程在 `bus` 那张卡上起过（或正在起）不走账的设备没有。
+    #[must_use]
+    pub fn has_devices_on_the_bus(&self, bus: &str) -> bool {
+        self.buses
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .contains(bus)
+    }
+}
+
+/// 卡上一段活的号：接 Out of Memory 的那一层每起一段活发一个（[`WorkOnTheCardOfThisThread::begin`]），进程里不重。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct WorkOnTheCardIdentifier(u64);
+
+static NEXT_WORK_ON_THE_CARD_IDENTIFIER: AtomicU64 = AtomicU64::new(0);
+
+thread_local! {
+    /// 这条线程上此刻正在做的那段卡上的活（[`WorkOnTheCardOfThisThread`]）。
+    static WORK_ON_THE_CARD_OF_THIS_THREAD: Cell<Option<WorkOnTheCardIdentifier>> = const { Cell::new(None) };
+}
+
+/// 这条线程上正在做的一段卡上的活：活着时，这条线程上记的 Out of Memory 算它的（[`DeviceConditionOfACard::record_an_out_of_memory`]）；
+/// 丢掉时这条线程换回原来那一段（没有就是没有）。只在起它的那条线程上丢（不跨线程送）。
+pub struct WorkOnTheCardOfThisThread {
+    identifier: WorkOnTheCardIdentifier,
+    work_of_this_thread_before: Option<WorkOnTheCardIdentifier>,
+    stays_on_the_thread_that_began_it: PhantomData<*const ()>,
+}
+
+impl WorkOnTheCardOfThisThread {
+    /// 在这条线程上起一段新的活（发一个新号）。
+    #[must_use]
+    pub fn begin() -> Self {
+        let identifier = WorkOnTheCardIdentifier(
+            NEXT_WORK_ON_THE_CARD_IDENTIFIER.fetch_add(1, Ordering::Relaxed),
+        );
+        let work_of_this_thread_before =
+            WORK_ON_THE_CARD_OF_THIS_THREAD.with(|work| work.replace(Some(identifier)));
+        Self {
+            identifier,
+            work_of_this_thread_before,
+            stays_on_the_thread_that_began_it: PhantomData,
+        }
+    }
+
+    #[must_use]
+    pub fn identifier(&self) -> WorkOnTheCardIdentifier {
+        self.identifier
+    }
+}
+
+impl Drop for WorkOnTheCardOfThisThread {
+    fn drop(&mut self) {
+        WORK_ON_THE_CARD_OF_THIS_THREAD.with(|work| work.set(self.work_of_this_thread_before));
+    }
+}
+
+/// 一次记下的 Out of Memory：wgpu 报的那一句，与它说得成哪几段活的显存不够。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct OutOfMemoryReport {
+    what: String,
+    works_it_explains: Vec<WorkOnTheCardIdentifier>,
+}
+
+/// 一个设备上 wgpu 报过的 Out of Memory（错误范围接住的与没接住的都记）与设备丢没丢（几个克隆共用一份）。接 Out of Memory 的那一层
+/// （崩溃放量产品路径 `singlefs_checker_tier::crash_amplification`）照它认出：活 panic 了、或设备丢了，是不是这段活的显存不够
+/// （第 10x 批代码轮判决 K1-3：wgpu 的错误范围按线程分，判器内核在另起的线程上编，那里的 Out of Memory 范围接不住）。
+/// 每一次按记的那条线程认是哪段活的（第 10y 批代码轮判决 K1-3 攻方：别的线程上一次 Out of Memory 不把这段活真正的 panic 说成显存不够）：
+/// 那条线程上正在做一段活就是那一段；没有（活另起的编内核的线程）就是那一刻这个设备上在编内核的那几段（[`Self::compile_for_the_work_of_this_thread`]）。
+#[derive(Default)]
+pub struct DeviceConditionOfACard {
+    out_of_memory_reports: Mutex<Vec<OutOfMemoryReport>>,
+    /// 此刻在这个设备上编内核的活，各有几处在编。
+    works_compiling: Mutex<BTreeMap<WorkOnTheCardIdentifier, usize>>,
+    lost: Mutex<Option<String>>,
+}
+
+/// 编内核那一段活着时这个设备上记着「在给这段活编」，丢掉时（编完或 panic）抹掉。
+struct CompilingForAWork<'condition> {
+    works_compiling: &'condition Mutex<BTreeMap<WorkOnTheCardIdentifier, usize>>,
+    work: WorkOnTheCardIdentifier,
+}
+
+impl Drop for CompilingForAWork<'_> {
+    fn drop(&mut self) {
+        // 持锁只为改一个计数，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
+        let mut works_compiling = self
+            .works_compiling
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        let still_compiling = works_compiling.get(&self.work).copied().unwrap_or(0);
+        if still_compiling <= 1 {
+            works_compiling.remove(&self.work);
+        } else {
+            works_compiling.insert(self.work, still_compiling - 1);
+        }
+    }
+}
+
+impl DeviceConditionOfACard {
+    /// 记一次 Out of Memory（`what` 说是在哪、wgpu 报的那一句），算在记的这条线程上正在做的那段活上；这条线程上没有活，
+    /// 算在此刻这个设备上在编内核的那几段活上（都没有就哪段活都说不成）。
+    pub fn record_an_out_of_memory(&self, what: String) {
+        let works_it_explains = match WORK_ON_THE_CARD_OF_THIS_THREAD.with(Cell::get) {
+            Some(work) => vec![work],
+            None => self
+                .works_compiling
+                .lock()
+                .unwrap_or_else(PoisonError::into_inner)
+                .keys()
+                .copied()
+                .collect(),
+        };
+        // 持锁只为追加一条，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
+        self.out_of_memory_reports
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .push(OutOfMemoryReport {
+                what,
+                works_it_explains,
+            });
+    }
+
+    /// 到此刻一共记了几次 Out of Memory。
+    #[must_use]
+    pub fn out_of_memory_reports(&self) -> usize {
+        self.out_of_memory_reports
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .len()
+    }
+
+    /// 在这个设备上为这条线程上正在做的那段活编内核（`compile`，里面另起的线程上记的 Out of Memory 算这段活的），交 `compile` 的结果；
+    /// 这条线程上没有活就只是调 `compile`。`compile` panic 照样往外抛。
+    pub fn compile_for_the_work_of_this_thread<Output>(
+        &self,
+        compile: impl FnOnce() -> Output,
+    ) -> Output {
+        let Some(work) = WORK_ON_THE_CARD_OF_THIS_THREAD.with(Cell::get) else {
+            return compile();
+        };
+        *self
+            .works_compiling
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .entry(work)
+            .or_insert(0) += 1;
+        let _compiling = CompilingForAWork {
+            works_compiling: &self.works_compiling,
+            work,
+        };
+        compile()
+    }
+
+    /// 记设备丢了（wgpu 的设备丢失回调调它）。
+    pub fn record_the_device_lost(&self, why: String) {
+        *self.lost.lock().unwrap_or_else(PoisonError::into_inner) = Some(why);
+    }
+
+    /// 设备丢了的话为什么丢（没丢是 `None`）。
+    #[must_use]
+    pub fn lost_reason(&self) -> Option<String> {
+        self.lost
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .clone()
+    }
+
+    /// 段活 `work` 从记到 `reports_before` 次那一刻起出了一样坏事、能不能说成是它的显存不够：这之后记了算在它上面的 Out of Memory，
+    /// 交最后那一句；设备丢了、这之后记过 Out of Memory（算在哪段活上都行：wgpu 把建管线、提交这类调用上的 Out of Memory 当致命、丢掉设备，
+    /// 丢了设备谁都做不成），交两样都说的一句；别的交 `None`（别的活上的 Out of Memory、这段活开始之前的，都说不成）。
+    #[must_use]
+    pub fn out_of_memory_explaining_a_failure_of_the_work(
+        &self,
+        work: WorkOnTheCardIdentifier,
+        reports_before: usize,
+    ) -> Option<String> {
+        let reports = self
+            .out_of_memory_reports
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        let reports_since = reports.get(reports_before..).unwrap_or(&[]);
+        if let Some(report) = reports_since
+            .iter()
+            .rev()
+            .find(|report| report.works_it_explains.contains(&work))
+        {
+            return Some(report.what.clone());
+        }
+        match (self.lost_reason(), reports_since.last()) {
+            (Some(lost), Some(report)) => {
+                Some(format!("设备丢了（{lost}），丢之前报过 {}", report.what))
+            }
+            (None, Some(_) | None) | (Some(_), None) => None,
+        }
+    }
+}
+
 #[cfg(test)]
 mod tests {
     use super::DeviceStartedOnDemand;
     use super::{
+        attempt_on_a_card_without_room_for_one_state, card_memory_reader_of_one_report_per_reading,
+        card_memory_readings_of_one_nvidia_smi_report, decide_scratch_before_booking,
+        gpu_not_run_line, pci_bus_identifier_without_the_domain,
+        reason_for_running_out_of_gpu_memory, returned_possibly_still_on_the_card_after_a_reading,
+        BusesWithDevicesStartedOutsideTheBooks, DeviceConditionOfACard, WorkOnTheCardOfThisThread,
+        GPU_MEMORY_SHORT_REASON_PREFIX,
+    };
+    use super::{
         card_ledger_capacity_mebibytes, card_room_for_this_process,
         decide_after_attempting_every_card, decide_booking_by_priority,
         mebibytes_of_buffers_held_at_once, memory_this_process_may_hold_mebibytes,
@@ -1348,12 +1714,6 @@
         OccupiedByThisProcess, ScratchAttemptOnACard, ScratchBeforeBooking,
         ScratchBookingRefusalOutcome, ScratchPlacementDecision, ScratchStatesToBook,
     };
-    use super::{
-        card_memory_reader_of_one_report_per_reading,
-        card_memory_readings_of_one_nvidia_smi_report, decide_scratch_before_booking,
-        gpu_not_run_line, pci_bus_identifier_without_the_domain,
-        reason_for_running_out_of_gpu_memory, GPU_MEMORY_SHORT_REASON_PREFIX,
-    };
     use std::sync::atomic::{AtomicUsize, Ordering};
     use std::sync::{Arc, Mutex};
 
@@ -1497,7 +1857,7 @@
     }
 
     /// 第 10x 批：这个进程能占的是现读的空闲加它实际占着的里属于它许出去的那一部分：没登记、登记了没起设备都只算空闲；
-    /// 起了设备而读不到实际占着的，驱动那一份当已经在卡上；读得到的只算到许出去的那么多（分配器多拿的块不算）。
+    /// 起了设备而读不到实际占着的，驱动那一份也不当已经在卡上（第 10z 批：实占多少分不出）；读得到的只算到许出去的那么多（分配器多拿的块不算）。
     #[test]
     fn the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised() {
         let free = reading_of_an_rtx_5090(20_000);
@@ -1532,8 +1892,8 @@
                 booked(1_000, DriverShareOnTheCard::DeviceStarted),
                 DRIVER
             ),
-            20_000 + DRIVER,
-            "起了设备、实际占着的读不到：驱动那一份当已经在卡上"
+            20_000,
+            "起了设备、实际占着的读不到：驱动那一份实占多少分不出，不当已经在卡上"
         );
         let occupied = |mebibytes: u64| CardMemoryReading {
             occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
@@ -1578,8 +1938,8 @@
         );
     }
 
-    /// 第 10x 批（先登记、后建设备，这个进程里）：没在一张卡上借过就起设备要断言；借过之后起得了，起好了驱动那一份算在卡上，
-    /// 读不到实际占着的时候这一张能借的多出驱动那一份。
+    /// 第 10x 批（先登记、后建设备，这个进程里）：没在一张卡上借过就起设备要断言；借过之后起得了，起好了驱动那一份算在卡上：
+    /// 读数里这个进程占着的驱动那一份从空闲里少了，不再从能借的里扣第二次；读不到实际占着的时候照旧扣（第 10z 批：实占多少分不出）。
     #[test]
     fn the_device_starts_only_on_a_card_this_process_booked_and_then_counts_the_driver_share_as_on_the_card(
     ) {
@@ -1603,12 +1963,32 @@
             Ok(vec![30_000 - 1_000 - DRIVER]),
             "设备没起：驱动那一份还要从空闲里扣"
         );
-        ledger.check_registered_before_starting_the_device(0);
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(()),
+            "头一回起：驱动那一份在登记的那一笔里借过"
+        );
         ledger.record_the_device_started_on_the_card(0);
         assert_eq!(
             ledger.card_rooms_mebibytes_now(&[0]),
-            Ok(vec![30_000 - 1_000]),
-            "设备起好了：驱动那一份已经在卡上，现读的空闲里少了它，不再扣"
+            Ok(vec![30_000 - 1_000 - DRIVER]),
+            "设备起好了而读不到实际占着的：驱动那一份照旧扣"
+        );
+        replace_the_reading(
+            &readings,
+            0,
+            CardMemoryReading {
+                free_mebibytes: 30_000 - DRIVER,
+                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
+                occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
+                    mebibytes: DRIVER,
+                },
+            },
+        );
+        assert_eq!(
+            ledger.card_rooms_mebibytes_now(&[0]),
+            Ok(vec![30_000 - DRIVER - 1_000]),
+            "设备起好了、读到占着驱动那一份：现读的空闲里少了它，不再扣第二次"
         );
         drop(booking);
     }
@@ -2260,9 +2640,10 @@
         assert_eq!(
             room,
             CardRoomForThisProcess {
-                room_now_mebibytes: 20_000 - 3_000,
+                room_now_mebibytes: 20_000 - DRIVER - 3_000,
                 ceiling_mebibytes: AN_EMPTY_RTX_5090_HAS - DRIVER,
-            }
+            },
+            "读不到实际占着的：驱动那一份不当已经在卡上，从现读的空闲里扣"
         );
     }
 
@@ -2489,8 +2870,8 @@
                 booked,
                 DRIVER
             ),
-            15_000 + DRIVER,
-            "分不出：借着的 4 000 当一点都没建，只把起好了设备的驱动那一份当在卡上"
+            15_000,
+            "分不出：借着的 4 000 当一点都没建，起好了设备的驱动那一份也不当在卡上（第 10z 批：实占多少分不出）"
         );
         assert_eq!(
             memory_this_process_may_hold_mebibytes(
@@ -2498,7 +2879,7 @@
                 booked,
                 DRIVER
             ),
-            15_000 + DRIVER,
+            15_000,
             "与没列这个进程同样算"
         );
         assert_eq!(
@@ -2750,4 +3131,505 @@
         assert_eq!(starts.load(Ordering::SeqCst), 2);
         assert!(slot.is_started());
     }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K1-1 攻方乙形）：不走账的设备起之前就记下它在哪张卡上：起的那一段里卡池读这张卡已经认得出
+    /// 这个进程另有不走账的设备；没起过的卡不记。
+    #[test]
+    fn a_device_outside_the_books_is_marked_on_its_bus_before_it_starts() {
+        let buses = BusesWithDevicesStartedOutsideTheBooks::new();
+        assert_eq!(
+            buses.start_a_device_marked_first("01:00.0", || {
+                Ok::<bool, String>(buses.has_devices_on_the_bus("01:00.0"))
+            }),
+            Ok(true),
+            "起的那一段里已经记着"
+        );
+        assert!(buses.has_devices_on_the_bus("01:00.0"));
+        assert!(!buses.has_devices_on_the_bus("10:00.0"), "没起过的卡不记");
+    }
+
+    /// 第 10z 批：不走账的设备起不来照样记着（分不出它占没占，只会少借）。
+    #[test]
+    fn a_device_outside_the_books_that_failed_to_start_stays_marked_on_its_bus() {
+        let buses = BusesWithDevicesStartedOutsideTheBooks::new();
+        assert_eq!(
+            buses
+                .start_a_device_marked_first("10:00.0", || Err::<(), String>("起不来".to_string())),
+            Err("起不来".to_string())
+        );
+        assert!(buses.has_devices_on_the_bus("10:00.0"));
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K1-1 攻方乙附，数照攻方模型那一行：空 21 010，借着 8 000 还没建，驱动实占 600）：
+    /// 设备起好了而报告里没列这个进程，驱动那一份不当整份在卡上：能借的是空闲减驱动那一份减借着的 10 450，不超过卡上此刻真能再给的
+    /// 11 050（空闲减驱动还要长的 1 960 减借着的）；当整份在卡上是 13 010，多给 D − x。
+    #[test]
+    fn with_this_process_not_listed_a_started_device_leaves_room_for_its_whole_driver_share() {
+        const FREE: u64 = 21_010;
+        const LEASES: u64 = 8_000;
+        const DRIVER_HOLDS_NOW: u64 = 600;
+        let room = card_room_for_this_process(
+            reading_of_an_rtx_5090(FREE),
+            BookedByThisProcessOnACard {
+                leases_mebibytes: LEASES,
+                driver_share: DriverShareOnTheCard::DeviceStarted,
+                returned_possibly_still_on_the_card_mebibytes: 0,
+            },
+            DRIVER,
+            &|may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER),
+        );
+        assert_eq!(room.room_now_mebibytes, FREE - DRIVER - LEASES);
+        assert!(
+            room.room_now_mebibytes <= FREE - (DRIVER - DRIVER_HOLDS_NOW) - LEASES,
+            "不超过卡上此刻真能再给的：{room:?}"
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K1-1 攻方丙形，数照攻方模型那一行：5090 空卡 32 110，外人占 10 000，借 4 000、设备起好、建满之后丢掉，
+    /// 驱动实占 600，那 4 000 卡上还没还）：销了账的那一份不照「驱动那一份整份在卡上」压，下一笔能借的是卡上此刻真能再给的 15 550
+    /// （空 17 510 减驱动还要长的 1 960）；照 D 压是 17 510，多给 D − x。
+    #[test]
+    fn a_returned_lease_is_pressed_by_what_was_seen_on_the_card_not_by_a_whole_driver_share() {
+        const EMPTY: u64 = 32_110;
+        const OUTSIDER: u64 = 10_000;
+        const DRIVER_HOLDS_NOW: u64 = 600;
+        const LEASE: u64 = 4_000;
+        let readings = replaceable_readings(vec![CardMemoryReading {
+            free_mebibytes: EMPTY - OUTSIDER,
+            total_less_reserved_mebibytes: EMPTY,
+            occupied_by_this_process: OccupiedByThisProcess::NotListed,
+        }]);
+        let shared = Arc::clone(&readings);
+        let ledger = GpuMemoryLedger::new(
+            1,
+            Box::new(move || Ok(shared.lock().expect("用例里只有一条线程改读数").clone())),
+            Box::new(|_position, may_hold_mebibytes| {
+                card_ledger_capacity_mebibytes(
+                    None,
+                    Some(32_000),
+                    may_hold_mebibytes.min(32_000),
+                    may_hold_mebibytes,
+                    DRIVER,
+                )
+            }),
+            DRIVER,
+        );
+        let lease = ledger.book_on_the_card(0, LEASE).expect("卡上空着 22 110");
+        ledger.record_the_device_started_on_the_card(0);
+        drop(lease);
+        let free_now = EMPTY - OUTSIDER - DRIVER_HOLDS_NOW - LEASE;
+        replace_the_reading(
+            &readings,
+            0,
+            CardMemoryReading {
+                free_mebibytes: free_now,
+                total_less_reserved_mebibytes: EMPTY,
+                occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
+                    mebibytes: DRIVER_HOLDS_NOW + LEASE,
+                },
+            },
+        );
+        let truly_usable = free_now - (DRIVER - DRIVER_HOLDS_NOW);
+        assert_eq!(
+            ledger.card_rooms_mebibytes_now(&[0]),
+            Ok(vec![truly_usable]),
+            "那 4 000 卡上还没还，驱动还要长 1 960：能借的是 15 550"
+        );
+        assert_eq!(
+            ledger.card_rooms_mebibytes_now(&[0]),
+            Ok(vec![truly_usable]),
+            "再读一次照旧"
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K2-2 P1，数照正推在真账上造出的那一组）：一笔 10 000 的长借据建满之后，十笔 1 000 的短借据各借、建、丢，
+    /// 丢了之后卡上马上还给驱动（这个进程占的回到驱动那一份加 10 000）。每还一笔，能借的照旧是真空闲 19 550（不再一笔一笔多扣）；
+    /// 要 18 550 放得下，借得到。
+    #[test]
+    fn returned_short_leases_the_card_gave_back_cost_nothing_while_a_long_lease_stays_built() {
+        const EMPTY: u64 = 32_110;
+        const LONG_LEASE: u64 = 10_000;
+        const SHORT_LEASE: u64 = 1_000;
+        let readings = replaceable_readings(vec![CardMemoryReading {
+            free_mebibytes: EMPTY,
+            total_less_reserved_mebibytes: EMPTY,
+            occupied_by_this_process: OccupiedByThisProcess::NotListed,
+        }]);
+        let shared = Arc::clone(&readings);
+        let ledger = GpuMemoryLedger::new(
+            1,
+            Box::new(move || Ok(shared.lock().expect("用例里只有一条线程改读数").clone())),
+            Box::new(|_position, may_hold_mebibytes| {
+                card_ledger_capacity_mebibytes(
+                    None,
+                    Some(40_000),
+                    may_hold_mebibytes.min(40_000),
+                    may_hold_mebibytes,
+                    DRIVER,
+                )
+            }),
+            DRIVER,
+        );
+        let occupied = |free_mebibytes: u64, mebibytes: u64| CardMemoryReading {
+            free_mebibytes,
+            total_less_reserved_mebibytes: EMPTY,
+            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
+        };
+        let long_lease = ledger.book_on_the_card(0, LONG_LEASE).expect("空卡借得到");
+        ledger.record_the_device_started_on_the_card(0);
+        let free_while_the_long_lease_is_built = EMPTY - DRIVER - LONG_LEASE;
+        replace_the_reading(
+            &readings,
+            0,
+            occupied(free_while_the_long_lease_is_built, DRIVER + LONG_LEASE),
+        );
+        assert_eq!(
+            ledger.card_rooms_mebibytes_now(&[0]),
+            Ok(vec![free_while_the_long_lease_is_built]),
+            "长借据建满：能借的是真空闲"
+        );
+        for short_lease_number in 1..=10 {
+            let short_lease = ledger
+                .book_on_the_card(0, SHORT_LEASE)
+                .expect("卡上空着 19 550");
+            replace_the_reading(
+                &readings,
+                0,
+                occupied(
+                    free_while_the_long_lease_is_built - SHORT_LEASE,
+                    DRIVER + LONG_LEASE + SHORT_LEASE,
+                ),
+            );
+            drop(short_lease);
+            replace_the_reading(
+                &readings,
+                0,
+                occupied(free_while_the_long_lease_is_built, DRIVER + LONG_LEASE),
+            );
+            assert_eq!(
+                ledger.card_rooms_mebibytes_now(&[0]),
+                Ok(vec![free_while_the_long_lease_is_built]),
+                "第 {short_lease_number} 笔短借据还了、卡上已经还给驱动：不多扣"
+            );
+        }
+        let need = free_while_the_long_lease_is_built - SHORT_LEASE;
+        assert_eq!(
+            booking_kept_on_the_books(&ledger, need),
+            Ok(need),
+            "要 18 550、真空闲 19 550：放得下，借得到"
+        );
+        drop(long_lease);
+    }
+
+    /// 第 10z 批的两条重起用例共用：一张 5090 上空 30 000，借一笔 10 000、设备起好、建满（卡上空 17 440），再借 `second_lease_mebibytes`
+    /// 还没建。交账与换读数的那一份（卡上的读数停在建满那一刻：丢了的设备还占着）。
+    fn ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(
+        second_lease_mebibytes: u64,
+    ) -> (GpuMemoryLedger, ReplaceableReadings) {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+        let ledger = ledger_reading(&readings);
+        booking_kept_on_the_books(&ledger, 10_000).expect("空着的卡借得到");
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(())
+        );
+        ledger.record_the_device_started_on_the_card(0);
+        replace_the_reading(
+            &readings,
+            0,
+            CardMemoryReading {
+                free_mebibytes: 30_000 - DRIVER - 10_000,
+                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
+                occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
+                    mebibytes: DRIVER + 10_000,
+                },
+            },
+        );
+        booking_kept_on_the_books(&ledger, second_lease_mebibytes)
+            .expect("建满之后卡上还空着 17 440");
+        (ledger, readings)
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K1-3 攻方）：起过的设备丢了要重起，驱动那一份重新登记：卡上空 17 440（丢了的设备还占着），
+    /// 第二笔 17 440 还没建，再起一个设备连它的驱动那一份要 20 000，放不下：交借不到，这个进程照旧登记着（驱动那一份许出去而没占上卡）。
+    #[test]
+    fn a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now() {
+        let (ledger, _readings) =
+            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(
+                30_000 - DRIVER - 10_000,
+            );
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Err(BookingRefusal::NoCardHasRoomNow {
+                need_mebibytes: DRIVER,
+                card_rooms_mebibytes: vec![0],
+            }),
+            "新设备的驱动那一份放不下：不起"
+        );
+        assert!(ledger.is_this_process_registered_on_the_card(0));
+    }
+
+    /// 第 10z 批：同上，第二笔只有 10 000：再起的设备连驱动那一份放得下，起；之后能借的照新设备的驱动那一份还没占上卡算
+    /// （17 440 减驱动那一份减还没建的 10 000），不照丢了的那一份当已在卡上。
+    #[test]
+    fn a_lost_device_started_again_books_its_driver_share_as_not_yet_on_the_card() {
+        let (ledger, _readings) =
+            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(10_000);
+        assert_eq!(
+            ledger.card_rooms_mebibytes_now(&[0]),
+            Ok(vec![30_000 - DRIVER - 10_000 - 10_000]),
+            "丢之前：驱动那一份在卡上"
+        );
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(()),
+            "新设备连驱动那一份放得下"
+        );
+        assert_eq!(
+            ledger.card_rooms_mebibytes_now(&[0]),
+            Ok(vec![30_000 - DRIVER - 10_000 - DRIVER - 10_000]),
+            "重起之前：新设备的驱动那一份还要从空闲里扣"
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K1-3 攻方）：同一个设备上，别的线程上那段活记的 Out of Memory 说不成这段活的显存不够
+    /// （这段活真正的 panic 照样往外抛）；这段活自己这条线程上记的说得成。
+    #[test]
+    fn an_out_of_memory_in_the_work_of_another_thread_does_not_explain_a_failure_of_this_work() {
+        let condition = DeviceConditionOfACard::default();
+        let this_work = WorkOnTheCardOfThisThread::begin();
+        let reports_before = condition.out_of_memory_reports();
+        std::thread::scope(|scope| {
+            scope.spawn(|| {
+                let _another_work = WorkOnTheCardOfThisThread::begin();
+                condition.record_an_out_of_memory("别的用例在这张卡上撞满".to_string());
+            });
+        });
+        assert_eq!(condition.out_of_memory_reports(), reports_before + 1);
+        assert_eq!(
+            condition.out_of_memory_explaining_a_failure_of_the_work(
+                this_work.identifier(),
+                reports_before
+            ),
+            None,
+            "别的线程上那段活的 Out of Memory：说不成这段活的显存不够"
+        );
+        condition.record_an_out_of_memory("这段活自己判一块时".to_string());
+        assert_eq!(
+            condition.out_of_memory_explaining_a_failure_of_the_work(
+                this_work.identifier(),
+                reports_before
+            ),
+            Some("这段活自己判一块时".to_string()),
+            "这段活自己这条线程上记的：说得成"
+        );
+    }
+
+    /// 第 10z 批：判器内核在活另起的线程上编，那几条线程上没有活；在这个设备上为这段活编内核的那一刻记的 Out of Memory 算这段活的，
+    /// 编完之后没有活的线程上记的哪段活都说不成。
+    #[test]
+    fn an_out_of_memory_on_a_thread_without_work_explains_only_the_work_compiling_on_the_device_then(
+    ) {
+        let condition = DeviceConditionOfACard::default();
+        let compiling_work = WorkOnTheCardOfThisThread::begin();
+        let reports_before = condition.out_of_memory_reports();
+        condition.compile_for_the_work_of_this_thread(|| {
+            std::thread::scope(|scope| {
+                scope.spawn(|| {
+                    condition
+                        .record_an_out_of_memory("另起的线程上编恢复内核时没接住的".to_string());
+                });
+            });
+        });
+        assert_eq!(
+            condition.out_of_memory_explaining_a_failure_of_the_work(
+                compiling_work.identifier(),
+                reports_before
+            ),
+            Some("另起的线程上编恢复内核时没接住的".to_string()),
+            "编内核的那一刻另起的线程上记的：算这段活的"
+        );
+        let reports_after_compiling = condition.out_of_memory_reports();
+        std::thread::scope(|scope| {
+            scope.spawn(|| {
+                condition.record_an_out_of_memory("编完之后没有活的线程上记的".to_string());
+            });
+        });
+        assert_eq!(
+            condition.out_of_memory_explaining_a_failure_of_the_work(
+                compiling_work.identifier(),
+                reports_after_compiling
+            ),
+            None,
+            "编完之后没有活的线程上记的：说不成"
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮正推）：设备丢了，而这段活开始之后没记过 Out of Memory（早先一段活上处理过的一次不算），说不成显存不够；
+    /// 这段活开始之后记过（别的活上的也算：丢了设备谁都做不成），说成设备因它丢了。
+    #[test]
+    fn a_lost_device_explains_a_failure_only_with_an_out_of_memory_since_the_work_began() {
+        let condition = DeviceConditionOfACard::default();
+        condition.record_an_out_of_memory("早先处理过的一次".to_string());
+        let work = WorkOnTheCardOfThisThread::begin();
+        let reports_before = condition.out_of_memory_reports();
+        condition.record_the_device_lost("Unknown：驱动重置".to_string());
+        assert_eq!(
+            condition
+                .out_of_memory_explaining_a_failure_of_the_work(work.identifier(), reports_before),
+            None,
+            "设备丢了而这段活开始之后没报过：不是显存不够"
+        );
+        std::thread::scope(|scope| {
+            scope.spawn(|| {
+                let _another_work = WorkOnTheCardOfThisThread::begin();
+                condition.record_an_out_of_memory("别的活上撞满".to_string());
+            });
+        });
+        assert!(
+            condition
+                .out_of_memory_explaining_a_failure_of_the_work(work.identifier(), reports_before)
+                .is_some_and(|reason| reason.starts_with("设备丢了")),
+            "设备丢了、这段活开始之后报过：说成设备因它丢了"
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X16）：报告里这个进程那一行占的量读不出（`N/A`），记成没列（少借），不当成占了极大。
+    #[test]
+    fn an_unreadable_memory_of_this_process_in_the_report_is_not_listed_not_a_huge_occupancy() {
+        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(
+            20_000,
+            &[(ANOTHER_PROCESS, 9_000), (THIS_PROCESS, 3_141)],
+        )])
+        .replace(": 3141 MiB", ": N/A");
+        assert_eq!(
+            card_memory_readings_of_one_nvidia_smi_report(
+                &report,
+                THIS_PROCESS,
+                &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
+                &no_device_outside_the_books
+            )
+            .map(|readings| readings[0].occupied_by_this_process),
+            Ok(OccupiedByThisProcess::NotListed)
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X17）：报告里一行进程号认不出，报错（判红），不悄悄当成 0 号进程。
+    #[test]
+    fn an_unreadable_process_identifier_in_the_report_is_an_error_not_process_zero() {
+        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(
+            20_000,
+            &[(ANOTHER_PROCESS, 9_000), (THIS_PROCESS, 3_141)],
+        )])
+        .replace(": 7777\n", ": abc\n");
+        let readings = card_memory_readings_of_one_nvidia_smi_report(
+            &report,
+            THIS_PROCESS,
+            &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
+            &no_device_outside_the_books,
+        );
+        assert!(
+            readings
+                .as_ref()
+                .is_err_and(|reason| reason.contains("进程号认不出")),
+            "{readings:?}"
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X18）：报告里点名的卡的 Free 读不出，报错（判红），不悄悄当成空 0。
+    #[test]
+    fn an_unreadable_free_memory_in_the_report_is_an_error_not_zero() {
+        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(20_000, &[])])
+            .replace(": 20000 MiB", ": N/A");
+        let readings = card_memory_readings_of_one_nvidia_smi_report(
+            &report,
+            THIS_PROCESS,
+            &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
+            &no_device_outside_the_books,
+        );
+        assert!(
+            readings
+                .as_ref()
+                .is_err_and(|reason| reason.contains("Free 读不出")),
+            "{readings:?}"
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X19）：报告里点名的卡没有 Reserved 那一行（老驱动的样子），报错（判红），不悄悄当成保留 0
+    /// （那样空卡上能用的就成了总量）。
+    #[test]
+    fn a_missing_reserved_memory_in_the_report_is_an_error_not_zero() {
+        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(20_000, &[])])
+            .lines()
+            .filter(|line| !line.trim_start().starts_with("Reserved"))
+            .collect::<Vec<&str>>()
+            .join("\n");
+        let readings = card_memory_readings_of_one_nvidia_smi_report(
+            &report,
+            THIS_PROCESS,
+            &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
+            &no_device_outside_the_books,
+        );
+        assert!(
+            readings
+                .as_ref()
+                .is_err_and(|reason| reason.contains("Reserved 读不出")),
+            "{readings:?}"
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X2）：销了账、卡上可能还没还的那一份只在读数分得出这个进程走账的占用时往下压；
+    /// 这个进程另有不走账的设备、或没列这个进程，照旧不压。
+    #[test]
+    fn a_reading_that_cannot_tell_this_process_apart_leaves_the_returned_share_unpressed() {
+        assert_eq!(
+            returned_possibly_still_on_the_card_after_a_reading(
+                4_000,
+                OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 1_000 },
+                0
+            ),
+            1_000,
+            "都走这份账：压到占着的那么多"
+        );
+        assert_eq!(
+            returned_possibly_still_on_the_card_after_a_reading(
+                4_000,
+                OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 1_000 },
+                0
+            ),
+            4_000,
+            "另有不走账的设备：分不出，不压"
+        );
+        assert_eq!(
+            returned_possibly_still_on_the_card_after_a_reading(
+                4_000,
+                OccupiedByThisProcess::NotListed,
+                0
+            ),
+            4_000,
+            "没列这个进程：不压"
+        );
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X4）：一个状态要的恰好等于空着时能借的，是此刻借不到（报本次未跑），不是放不下；
+    /// 多 1 MiB 才是空着时也放不下。
+    #[test]
+    fn a_state_needing_exactly_what_an_empty_card_lends_is_no_room_now() {
+        let room = CardRoomForThisProcess {
+            room_now_mebibytes: 0,
+            ceiling_mebibytes: 1_000,
+        };
+        assert_eq!(
+            attempt_on_a_card_without_room_for_one_state(1_000, room),
+            ScratchAttemptOnACard::NoRoomNow {
+                need_mebibytes: 1_000
+            }
+        );
+        assert_eq!(
+            attempt_on_a_card_without_room_for_one_state(1_001, room),
+            ScratchAttemptOnACard::CannotHoldOneState {
+                ceiling_mebibytes: 1_000
+            }
+        );
+    }
 }
````

## 变异表追加（`mutations-append.tsv` 原样）

````tsv
第10z批 显存账 K1-1 不走账的设备起好了才记总线号（起设备到记下之间被当成借据建上的，攻方乙形）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.buses\n            .lock()\n            .unwrap_or_else(PoisonError::into_inner)\n            .insert(bus.to_string());\n        start()	        let started = start();\n        self.buses\n            .lock()\n            .unwrap_or_else(PoisonError::into_inner)\n            .insert(bus.to_string());\n        started	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_device_outside_the_books_is_marked_on_its_bus_before_it_starts	a_device_outside_the_books_is_marked_on_its_bus_before_it_starts
第10z批 显存账 K1-1 没列这个进程时起好了设备的驱动那一份当整份在卡上（实占 x < D 多给 D − x，攻方乙附）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => 0,	        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {\n            mebibytes_left_for_the_driver.min(promised_mebibytes)\n        }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::with_this_process_not_listed_a_started_device_leaves_room_for_its_whole_driver_share	with_this_process_not_listed_a_started_device_leaves_room_for_its_whole_driver_share
第10z批 显存账 K1-1 销了账的那一份照驱动那一份整份在卡上往下压（实占 x < D 多给 D − x，攻方丙形）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                .min(mebibytes.saturating_sub(seen_on_the_card_at_least_mebibytes))	                .min(mebibytes.saturating_sub(seen_on_the_card_at_least_mebibytes.max(2560)))	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_returned_lease_is_pressed_by_what_was_seen_on_the_card_not_by_a_whole_driver_share	a_returned_lease_is_pressed_by_what_was_seen_on_the_card_not_by_a_whole_driver_share
第10z批 显存账 K2-2 P1 往下压时不看早先见到占在卡上的（还给驱动的短借据一笔一笔累积压不掉）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                .min(mebibytes.saturating_sub(seen_on_the_card_at_least_mebibytes))	                .min(mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::returned_short_leases_the_card_gave_back_cost_nothing_while_a_long_lease_stays_built	returned_short_leases_the_card_gave_back_cost_nothing_while_a_long_lease_stays_built
第10z批 显存账 K2-2 P1 那一刻之后借的销账也从早先见到的那一份里减	crates/singlefs-harness/src/gpu_memory_ledger.rs	                if self.leases_seen_on_the_card.remove(&lease_identifier) {	                if self.leases_seen_on_the_card.remove(&lease_identifier) || mebibytes > 0 {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::returned_short_leases_the_card_gave_back_cost_nothing_while_a_long_lease_stays_built	returned_short_leases_the_card_gave_back_cost_nothing_while_a_long_lease_stays_built
第10z批 显存账 K2-2 P1 读到的不记成见到占在卡上的	crates/singlefs-harness/src/gpu_memory_ledger.rs	            self.seen_on_the_card_at_least_mebibytes = seen_now_mebibytes;	            let _ = seen_now_mebibytes;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::returned_short_leases_the_card_gave_back_cost_nothing_while_a_long_lease_stays_built	returned_short_leases_the_card_gave_back_cost_nothing_while_a_long_lease_stays_built
第10z批 显存账 K1-3 设备丢了重起不重新登记驱动那一份	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::RegisteredDeviceNotStarted => Ok(()),\n            DriverShareOnTheCard::DeviceStarted => {	            DriverShareOnTheCard::RegisteredDeviceNotStarted | DriverShareOnTheCard::DeviceStarted => Ok(()),\n            DriverShareOnTheCard::DeviceStarted => {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now	a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now
第10z批 显存账 K1-3 重起时旧设备那一份不当销了账而卡上可能还没还的	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .saturating_add(mebibytes_left_for_the_driver);	            .saturating_add(mebibytes_left_for_the_driver.min(0));	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now	a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now
第10z批 显存账 K1-3 别的线程那段活的 Out of Memory 也说成这段活的显存不够（按整台设备认）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .find(|report| report.works_it_explains.contains(&work))	            .find(|_report| true)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_out_of_memory_in_the_work_of_another_thread_does_not_explain_a_failure_of_this_work	an_out_of_memory_in_the_work_of_another_thread_does_not_explain_a_failure_of_this_work
第10z批 显存账 K1-3 没有活的线程上记的不算在设备上正在编内核的那段活上	crates/singlefs-harness/src/gpu_memory_ledger.rs	            None => self\n                .works_compiling\n                .lock()\n                .unwrap_or_else(PoisonError::into_inner)\n                .keys()\n                .copied()\n                .collect(),	            None => Vec::new(),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_out_of_memory_on_a_thread_without_work_explains_only_the_work_compiling_on_the_device_then	an_out_of_memory_on_a_thread_without_work_explains_only_the_work_compiling_on_the_device_then
第10z批 显存账 K1-3 设备丢了时这段活开始之前的 Out of Memory 也算（正推：早先处理过的一次加一次无关的丢失）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        let reports_since = reports.get(reports_before..).unwrap_or(&[]);	        let reports_since = reports.get(..).unwrap_or(&[]);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_explains_a_failure_only_with_an_out_of_memory_since_the_work_began	a_lost_device_explains_a_failure_only_with_an_out_of_memory_since_the_work_began
第10z批 显存账 K3 本进程那一行占的量读不出当极大（正推 X16）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                None | Some(Err(())) => OccupiedByThisProcess::NotListed,	                None => OccupiedByThisProcess::NotListed,\n                Some(Err(())) => OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: u64::MAX },	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_unreadable_memory_of_this_process_in_the_report_is_not_listed_not_a_huge_occupancy	an_unreadable_memory_of_this_process_in_the_report_is_not_listed_not_a_huge_occupancy
第10z批 显存账 K3 进程号认不出悄悄当 0（正推 X17）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                let process_identifier: u32 = value\n                    .parse()\n                    .map_err(|_| format!("nvidia-smi 的报告里一行进程号认不出：{line:?}"))?;	                let process_identifier: u32 = value.parse().unwrap_or(0);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_unreadable_process_identifier_in_the_report_is_an_error_not_process_zero	an_unreadable_process_identifier_in_the_report_is_an_error_not_process_zero
第10z批 显存账 K3 Free 读不出悄悄当 0（正推 X18）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            let free_mebibytes = card.free_mebibytes.ok_or_else(|| unreadable("Free"))?;	            let free_mebibytes = card.free_mebibytes.unwrap_or(0);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_unreadable_free_memory_in_the_report_is_an_error_not_zero	an_unreadable_free_memory_in_the_report_is_an_error_not_zero
第10z批 显存账 K3 Reserved 读不出悄悄当 0（正推 X19）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                .ok_or_else(|| unreadable("Reserved"))?;	                .unwrap_or(0);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_missing_reserved_memory_in_the_report_is_an_error_not_zero	a_missing_reserved_memory_in_the_report_is_an_error_not_zero
第10z批 显存账 K3 往下压时另有不走账的设备也当都走账（正推 X2）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => {\n            returned_possibly_still_on_the_card_mebibytes	        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes }\n        | OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes } => {\n            returned_possibly_still_on_the_card_mebibytes	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_reading_that_cannot_tell_this_process_apart_leaves_the_returned_share_unpressed	a_reading_that_cannot_tell_this_process_apart_leaves_the_returned_share_unpressed
第10z批 显存账 K3 一个状态恰好等于空着时能借的判成放不下（正推 X4）	crates/singlefs-harness/src/gpu_memory_ledger.rs	    if need_of_one_state_mebibytes > room.ceiling_mebibytes {	    if need_of_one_state_mebibytes >= room.ceiling_mebibytes {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_state_needing_exactly_what_an_empty_card_lends_is_no_room_now	a_state_needing_exactly_what_an_empty_card_lends_is_no_room_now
第10z批 GPU 取卡 K1-1 不归卡池的设备起之前不记总线号（攻方乙形）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL.start_a_device_marked_first(bus, start)	            {\n                let _ = bus;\n                start()\n            }	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::a_device_outside_the_card_pool_is_marked_on_its_bus_before_it_starts_and_a_pool_device_is_not	a_device_outside_the_card_pool_is_marked_on_its_bus_before_it_starts_and_a_pool_device_is_not
第10z批 GPU 取卡 K3 卡池读显存不认这个进程起过不归卡池设备的总线（攻方接线）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        Box::new(has_devices_started_outside_the_card_pool_on_the_bus),	        Box::new(|_bus: &str| false),	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_reads_this_process_as_also_outside_the_books_on_a_bus_with_a_device_outside_the_pool	the_card_pool_reads_this_process_as_also_outside_the_books_on_a_bus_with_a_device_outside_the_pool
第10z批 崩溃放量 K1-3 单元校验和对拍时报了 Out of Memory 的那一段不叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	        .find_map(|piece| piece.gpu_memory_was_short.clone())	        .find_map(|_piece| None::<String>)	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::the_first_piece_short_of_gpu_memory_stops_the_checking_with_its_reason	the_first_piece_short_of_gpu_memory_stops_the_checking_with_its_reason
第10z批 崩溃放量 K2-2 P2 一张卡交回的块不再给别的卡领	crates/singlefs-checker-tier/src/crash_amplification.rs	        if let Some(smallest) = given_back.iter().copied().min() {	        if let Some(smallest) = given_back.iter().copied().min().filter(|_| false) {	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::a_block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one	a_block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one
第10z批 崩溃放量 K2-2 P2 一张卡 Out of Memory 别的卡判完了也报本次未跑	crates/singlefs-checker-tier/src/crash_amplification.rs	    if blocks_stored >= pending_blocks {	    if false && blocks_stored >= pending_blocks {	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::an_out_of_memory_on_one_card_is_not_run_only_when_the_other_cards_left_blocks_unjudged	an_out_of_memory_on_one_card_is_not_run_only_when_the_other_cards_left_blocks_unjudged
第10z批 崩溃放量 K3 池里的卡空着时带得了几个状态照此刻能借的算	crates/singlefs-checker-tier/src/crash_amplification.rs	        states_in_flight_for_a_room(room.ceiling_mebibytes),	        states_in_flight_for_a_room(room.room_now_mebibytes),	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::a_pool_card_holds_states_now_by_its_room_now_and_when_empty_by_its_ceiling	a_pool_card_holds_states_now_by_its_room_now_and_when_empty_by_its_ceiling
第10z批 崩溃放量 K3 每张卡此刻带不了一个状态而空着时带得了也判红（借之前那一步接错）	crates/singlefs-checker-tier/src/crash_amplification.rs	        ScratchBeforeBooking::NoCardHoldsAStateNow => {\n            let card_rooms_mebibytes: Vec<u64> = run_cards	        ScratchBeforeBooking::NoCardHoldsAStateNow => {\n            if !run_cards.is_empty() || run_cards.is_empty() {\n                return ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty;\n            }\n            let card_rooms_mebibytes: Vec<u64> = run_cards	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::with_no_card_holding_a_state_now_the_step_before_booking_is_a_not_run_line_not_red	with_no_card_holding_a_state_now_the_step_before_booking_is_a_not_run_line_not_red
第10z批 崩溃放量 K3 借到的卡上起设备显存不够判红（不报本次未跑）	crates/singlefs-checker-tier/src/crash_amplification.rs	            let not_run_line = gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, reason);\n            println!("{not_run_line}");\n            return Ok(not_run_line);	            return Err(reason.clone());	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::a_device_short_of_gpu_memory_on_the_card_lent_is_reported_not_run_like_no_room_now	a_device_short_of_gpu_memory_on_the_card_lent_is_reported_not_run_like_no_room_now
第10z批 显存账 K1-1 不走账的设备起不来就不记总线号	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.buses\n            .lock()\n            .unwrap_or_else(PoisonError::into_inner)\n            .insert(bus.to_string());\n        start()	        let started = start();\n        if started.is_ok() {\n            self.buses\n                .lock()\n                .unwrap_or_else(PoisonError::into_inner)\n                .insert(bus.to_string());\n        }\n        started	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_device_outside_the_books_that_failed_to_start_stays_marked_on_its_bus	a_device_outside_the_books_that_failed_to_start_stays_marked_on_its_bus
第10z批 显存账 K1-3 设备丢了重起不重新登记驱动那一份（重起之后能借的照旧当它在卡上）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::RegisteredDeviceNotStarted => Ok(()),\n            DriverShareOnTheCard::DeviceStarted => {	            DriverShareOnTheCard::RegisteredDeviceNotStarted | DriverShareOnTheCard::DeviceStarted => Ok(()),\n            DriverShareOnTheCard::DeviceStarted => {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_started_again_books_its_driver_share_as_not_yet_on_the_card	a_lost_device_started_again_books_its_driver_share_as_not_yet_on_the_card
````

## 变异表替换（`mutations-replacements.tsv` 原样）

````tsv
第10y批 显存账 K1-1 销了账、卡上还没还的那一份不照读数往下压（卡上还了照样少借）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                .min(mebibytes.saturating_sub(seen_on_the_card_at_least_mebibytes))	                .max(mebibytes.saturating_sub(seen_on_the_card_at_least_mebibytes))	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_returned_lease_still_on_the_card_is_not_taken_for_a_live_lease_already_built	a_returned_lease_still_on_the_card_is_not_taken_for_a_live_lease_already_built
第10y批 GPU 取卡 K1-3 活开始之后记的 Out of Memory 不算（另起的线程上的说不成显存不够）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        let reports_since = reports.get(reports_before..).unwrap_or(&[]);	        let reports_since = reports.get(reports_before + 1..).unwrap_or(&[]);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_out_of_memory_on_a_thread_without_work_explains_only_the_work_compiling_on_the_device_then	an_out_of_memory_on_a_thread_without_work_explains_only_the_work_compiling_on_the_device_then
第10y批 GPU 取卡 K1-3 设备因 Out of Memory 丢了也说不成显存不够	crates/singlefs-harness/src/gpu_memory_ledger.rs	        match (self.lost_reason(), reports_since.last()) {	        match (None::<String>, reports_since.last()) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_explains_a_failure_only_with_an_out_of_memory_since_the_work_began	a_lost_device_explains_a_failure_only_with_an_out_of_memory_since_the_work_began
第10w批 崩溃放量 起过卡池的产品路径第 ① 段草稿区改回不借	crates/singlefs-checker-tier/src/crash_amplification.rs	            match scratch_of_each_card_booked_by_where_the_cards_came_from(\n                cards_taken_from,\n                cards,\n                &run_cards,	            match scratch_of_each_card_booked_by_where_the_cards_came_from(\n                GpuCardsSource::DevicesStartedForThisRun,\n                cards,\n                &run_cards,	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_judges_on_scratch_it_booked_on_the_card_pool	the_product_path_judges_on_scratch_it_booked_on_the_card_pool
第10x批 显存账 还没起设备就把驱动那一份当已经在卡上（空闲不到一份驱动开销也借给）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => 0,	        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {\n            mebibytes_left_for_the_driver\n        }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_with_less_free_memory_than_one_driver_share_lends_nothing_and_registers_nothing	a_card_with_less_free_memory_than_one_driver_share_lends_nothing_and_registers_nothing
````

## 变异表删除（`mutations-delete.txt` 原样）

````text
第10x批 显存账 起好了设备驱动那一份照样从空闲里扣
````
