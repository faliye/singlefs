# 根环实现第 10x 批：代码轮第一轮改动全文（2026-10-07）

实现员交的 `crates.patch` 原样，底是「主工作区 + 删除线第 33 批 S」（S 已先合进主工作区）。变异表追加、替换、删除三份附在后面；删除的 20 行测的是去掉的跨进程账那一套，点名的测试已删。

````diff
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
index 2404ae1..d95ce59 100644
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -1,56 +1,43 @@
-//! GPU 卡上的显存怎么分：一个判器一次派活最多带几个状态（草稿区按这条流的状态数封顶），与 GPU 用例共用的卡池怎么记账。
+//! GPU 卡上的显存怎么分：一个判器一次派活最多带几个状态（草稿区按这条流的状态数封顶），与这个进程里 GPU 用例共用的卡池怎么记账。
 //! 单放在 harness 档、只用 `std`：checker 档的 `crash_judge_dispatch`（草稿区封顶）与 `gpu_unit_checks::gpu_test_start` 的卡池
-//! （按卡记账、放不下就等）都调它；这几样不碰 wgpu、不要显卡，单测在 harness 档随时跑得到（checker 档的单测只在提交时跑）。
-//! 挑卡照卡在账里的先后（配置的优先级）找第一张放得下的，前一张放满了才用下一张
+//! 都调它；这几样不碰 wgpu、不要显卡，单测在 harness 档随时跑得到（checker 档的单测只在提交时跑）。
+//! 挑卡照卡在池里的先后（配置的优先级）找第一张放得下的，前一张放满了才用下一张
 //! （用户 2026-09-29 定「一个显卡 任务满了才去分另外一个」「一边占一点绝对不合理」）；
 //! 账只拦放不下的，不限同时借几份、也不限一台机器上同时跑几个 GPU 测试进程（用户 2026-10-06 定「开到最大的并发和gpu」
 //! 「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」）。
-//! 崩溃放量产品路径在起过卡池的进程里也走这份账：第 ① 段每张装了状态的卡按装的状态数借草稿区（[`scratch_states_to_book_on_each_card`]，
-//! 只记在那张卡上），等满了照 GPU 用例那一路报「本次未跑」
-//! （[`scratch_booking_refusal_outcome`]，同一个记号 [`GPU_NOT_RUN_MARKER`]）；没起过卡池的进程照旧不进账，是哪几种调用方写在
-//! `singlefs_checker_tier::crash_amplification` 的文件头。
 //!
-//! 产品路径几张卡一起借（第 10w 批，主 agent 定）：
-//! - 一张借不到不拿着别的卡干等：登记驱动那一份（[`GpuMemoryLedger::register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room`]）
-//!   与借草稿区（[`GpuMemoryLedger::book_states_on_the_cards_waiting_only_when_none_has_room`]）都按卡的先后每张当场试一次、不等；
-//!   借不到的卡不装状态，它那一份留给后面的卡（[`place_states_on_the_cards_that_book_now`]）；一张都借不到才等，等满报本次未跑。
-//! - 估计在压与借之间变小：按状态数借的那一笔在这张卡的锁里照此刻账本里的估计压状态数（[`WhatIsBookedOnOneCard`]），
-//!   调用方压过之后别的进程开账把估计改小了，这里照新的重压一次再借，不判成放不下。
-//! - 第 ② 段核对与单元校验和在卡上建的缓冲照草稿区那一套进账，量按 [`mebibytes_of_buffers_held_at_once`] 算：
-//!   上卡一次、整段都在的，加每一趟派活建、派完就丢的乘同时还活着的趟数。两样各自的字节数在
-//!   `singlefs_checker_tier::crash_amplification` 的文件头第 5 条（哪几张表、每趟几个缓冲照缓冲的大小推的，没上卡量过）。
+//! 只记这个进程借的（用户 2026-10-07 定第 10t 批代码轮判决岔路 G1「二：去掉跨进程账」）：没有账本文件、不锁文件、
+//! 不判别的进程活没活，也不等。每一次借都在这个进程的锁里现读一次每张卡此刻的显存（[`CardMemoryReading`]：空闲、总量、
+//! 这个进程实际占着的；卡池用 `nvidia-smi` 读），照这个进程在这张卡上已经借着的算这一笔放不放得下（[`card_room_for_this_process`]）。
 //!
-//! 账跨进程：每张卡一份账本文件，放在工作根下的 [`GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME`]，读写前对同名的锁文件 `flock`，
-//! 改完写到临时文件再改名换上（进程在写的半路被杀，账本还是改之前那一份）。账本里记三样：记账的进程合起来能在这张卡上占多少显存
-//! （[`memory_all_registered_processes_may_hold_mebibytes`] 的估计；那一行的键沿用 `free_memory_after_quiescing_mebibytes`，
-//! 账本的写法与第 10s 批相同，新旧两版的测试二进制读写得了同一份账本）、在这张卡上登记了的每个进程（各在卡上付一份驱动的固定开销）、
-//! 每一笔借出去的显存。每条记录带记账进程的进程号与启动时刻，读账时照 [`process_is_alive_by_its_entry_in_the_process_filesystem`] 判活
-//! （被杀而父进程还没收尸的僵尸也判成不在：驱动在进程退出时就还了它的显存），死了的那几条当没记、下一次写账时删掉
-//! （进程被杀时它借的显存靠这个回收）。
+//! 口径（第 10x 批，主 agent 定「三者取最小」照第 10t、10v 批）：设 F 是现读的空闲，B 是这个进程在这张卡上借着的，
+//! D 是驱动的固定开销（每个进程在每张卡上一份，起设备时付），M 是这个进程此刻能在这张卡上占的
+//! （[`memory_this_process_may_hold_mebibytes`]：F 加上这个进程实际占着的里属于它借出去的那一部分）。这一笔能借的是
+//! `card_ledger_capacity_mebibytes(…, M, D) − B`，展开就是下面几项里最小的：
+//! - 额度变量 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` − B（设了才有这一项）；
+//! - 配置给卡的额度 − D − B；没配显卡优先级时由 M 的 3/4 − D − B 代替（这几项都是这个进程在这张卡上一共借的连驱动那一份不超过额度）；
+//! - F − 这个进程借了还没实际占的 − D（这个进程还没在这张卡上起设备时）。「还没实际占的」是借着的连驱动那一份减实际占着的；
+//!   实际占着的读不到（`nvidia-smi` 没列这个进程）时当借着的一点都没建、起了设备的驱动那一份已经在卡上，只会少借。
 //!
-//! 估计怎么来（第 10t 批）：每个进程开账时（[`GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock`]，卡池起池时一次）
-//! 在每张卡的锁里现读一次空闲显存 F（这个进程自己在这张卡上还没起设备），同一把锁里数账上活着的进程已经记了的 K
-//! （每一笔借出去的，加每个登记了的进程各一份驱动开销）。记账的进程合起来真能占的是 F 加上「K 里已经实际占在卡上的那部分」A，
-//! 0 ≤ A ≤ K，所以真值落在 [F, F + K] 里：账本里已有前一个进程算的估计，就把它夹进这个区间；没有就取 F。
-//! - 外人（不经这份账的 GPU 用户）后来占了卡：F 变小，上界 F + K 把旧估计压下来（K 都已实际占着时正好压到真值）；
-//! - 外人后来走了：F 变大，下界 F 把旧估计抬上去（把 A 多扣了一遍，偏保守、只会少借）；
-//! - 卡被先起的进程订满：F 接近 0、K 接近整份估计，旧估计还在区间里、照旧，后起的进程等着借，不当成卡不够；
-//! - 两个进程先后开账、后开的那个读到的 F 里已经少了先登记的进程的驱动那一份：那一份在 K 里，上界把它补回来，只记一份。
+//! 借的要比这张卡空着时能借的（同一个算法拿卡的总量当 M、B 当 0）还大，等多久都放不下，调用方判红（配了 GPU 就一定用 GPU）；
+//! 比此刻能借的大而空着时放得下，是借不到：不等，调用方按借不到走下一条（几张卡一起借的换后面的卡，一张都借不到报本次未跑，
+//! [`scratch_booking_refusal_outcome`]、[`GPU_NOT_RUN_MARKER`]）。
+//! 账不防的：两个进程同一刻读到同一份空闲、一起借超了（各自只记自己借的）。这一形由运行时兜底：内存包装
+//! `research/scripts/run-with-memory-cap.sh` 认出内核日志里的 `NVRM … NV_ERR_NO_MEMORY` 退 245，wgpu 报 Out of Memory 时
+//! 崩溃放量产品路径把这一块报本次未跑（原因写显存不够，[`reason_for_running_out_of_gpu_memory`]）。
 //!
-//! 没有活着的记账进程时 K = 0，估计就是现读的 F。之后每一次借都照账本里此刻的估计算容量（[`GpuMemoryLedger::book_waiting_for_room`]），
-//! 别的进程开账时把估计改小了，早开账的进程下一笔也照小的借。
-//! 推的（没上卡量过）：A 读不到，只知道它在 [0, K] 里。夹出来的估计在一种情形下大于真值：外人在两次开账之间占了显存，
-//! 同时账上有借出去而还没在卡上建缓冲的借据（K 里 A 之外的那部分）把上界撑开；大出来的不超过这两样里小的那个。
-//! 驱动开销按常数记：真开销比常数大时上界偏小，只会偏保守。
-//!
-//! 先登记、后建设备：开账只读空闲、不登记。一个进程在一张卡上第一次借（[`GpuMemoryLedger::book_waiting_for_room`]）、
-//! 或只要登记驱动那一份（[`GpuMemoryLedger::register_this_process_on_the_card_waiting_for_room`]）时，这一笔连同它的驱动那一份
-//! 在账上放得下才记、才把这个进程登记到这张卡上；调用方拿到之后才在这张卡上起设备（卡池照这个次序做）。
+//! 先登记、后建设备（这个进程里）：一个进程在一张卡上第一次借时，这一笔连同它的驱动那一份放得下才记、才把这张卡记成登记了；
+//! 调用方拿到借据之后才在这张卡上起设备（[`GpuMemoryLedger::check_registered_before_starting_the_device`]），
+//! 起好了记一笔（[`GpuMemoryLedger::record_the_device_started_on_the_card`]），从此驱动那一份算实际占在卡上。
+//! 崩溃放量产品路径在起过卡池的进程里也走这份账：第 ① 段每张装了状态的卡按装的状态数借草稿区（[`scratch_states_to_book_on_each_card`]，
+//! 只记在那张卡上），几张卡当场各试一次（[`GpuMemoryLedger::book_states_on_the_cards_with_room`]），每张在锁里照此刻能借的压状态数，
+//! 借不到的卡不装状态、它那一份留给后面的卡（[`place_states_on_the_cards_that_book_now`]）；第 ② 段核对与单元校验和在卡上建的缓冲
+//! 照 [`mebibytes_of_buffers_held_at_once`] 算，借第一张放得下的。没起过卡池的进程照旧不进账，是哪几种调用方写在
+//! `singlefs_checker_tier::crash_amplification` 的文件头。
 
-use std::path::{Path, PathBuf};
+use std::collections::BTreeMap;
 use std::sync::atomic::{AtomicU64, Ordering};
-use std::sync::{Condvar, Mutex, PoisonError};
-use std::time::{Duration, Instant};
+use std::sync::{Mutex, MutexGuard, PoisonError};
 
 /// 一个判器一次派活最多带几个状态：显存装得下的、调用方要的、这条流一共几个状态，三者取小，至少 1。
 /// 草稿区按它开：状态数之外开的一个状态都用不上（一条 45 个状态的流按显存开满一张空着的 5090 是约 21 GiB，用得上的约 54 MiB）。
@@ -67,19 +54,19 @@ pub fn states_in_one_dispatch_capped_by_the_state_count(
         .max(1)
 }
 
-/// 一张卡的账能借给这个进程多少（MiB，这个进程自己的驱动那一份已经扣掉）。
+/// 一张卡上这个进程一共能占多少（MiB，这个进程自己的驱动那一份已经扣掉；减去它借着的才是这一笔能借的，文件头「口径」）。
 /// 设了 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（`card_quota_from_the_environment_mebibytes`）就取三者里最小的：这个额度、
-/// 配置给卡的额度减驱动的固定开销、记账进程合起来能占的显存减驱动的固定开销（第 10t 批 K4-1：额度变量不越过配置给卡的额度）。
+/// 配置给卡的额度减驱动的固定开销、这个进程此刻能在这张卡上占的减驱动的固定开销（第 10t 批 K4-1：额度变量不越过配置给卡的额度）。
 /// 配置给卡的额度：`GPU_CARDS_BY_PRIORITY` 配给这张卡的（`configured_card_quota_mebibytes`）；没配显卡优先级（`None`）时由
 /// 没配额度时默认的那一份代替，即 `budget_mebibytes`（那份显存的 3/4，第 10v 批：主 agent 定没配优先级时 3/4 那一份也并进最小值）。
-/// 没设额度变量照旧是这张卡给崩溃放量的额度（`budget_mebibytes`，配置的额度或那份显存的 3/4）减驱动的固定开销
-/// （主 agent 2026-10-06 定：它是每张卡账的上限，不是每条用例的上限，不判红）。
+/// 没设额度变量照旧是这张卡给崩溃放量的额度（`budget_mebibytes`，配置的额度与这个进程能占的里小的那个，或这个进程能占的 3/4）
+/// 减驱动的固定开销（主 agent 2026-10-06 定：它是每张卡的上限，不是每条用例的上限，不判红）。
 #[must_use]
 pub fn card_ledger_capacity_mebibytes(
     card_quota_from_the_environment_mebibytes: Option<u64>,
     configured_card_quota_mebibytes: Option<u64>,
     budget_mebibytes: u64,
-    memory_all_registered_processes_may_hold_mebibytes: u64,
+    memory_this_process_may_hold_mebibytes: u64,
     mebibytes_left_for_the_driver: u64,
 ) -> u64 {
     let default_share_without_a_configured_card_quota_mebibytes =
@@ -95,7 +82,7 @@ pub fn card_ledger_capacity_mebibytes(
                 })
                 .min(default_share_without_a_configured_card_quota_mebibytes)
                 .min(
-                    memory_all_registered_processes_may_hold_mebibytes
+                    memory_this_process_may_hold_mebibytes
                         .saturating_sub(mebibytes_left_for_the_driver),
                 ),
         ),
@@ -103,71 +90,132 @@ pub fn card_ledger_capacity_mebibytes(
     }
 }
 
-/// 记账的进程合起来能在一张卡上占多少显存（MiB）的新估计（判法在文件头「估计怎么来」）：`free_memory_read_now_mebibytes` 是
-/// 这一刻在这张卡的锁里读的空闲显存，`booked_by_live_processes_mebibytes` 是同一刻账上活着的进程已经记了的（借出去的加每个登记了的
-/// 进程一份驱动开销）；`previous_estimate_mebibytes` 是账本里已有的那一份。真值在 [现读的空闲, 现读的空闲 + 已记的] 里，
-/// 旧估计夹进这个区间，没有旧估计取现读的空闲。
+/// 这一刻读到的一张卡的显存（MiB）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CardMemoryReading {
+    pub free_mebibytes: u64,
+    pub total_mebibytes: u64,
+    /// 这个进程此刻实际占在这张卡上的（卡池照 `nvidia-smi` 按进程号报的）；读不到（没列这个进程、读不了）是 `None`。
+    pub occupied_by_this_process_mebibytes: Option<u64>,
+}
+
+/// 这个进程的驱动那一份在一张卡上走到哪一步。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DriverShareOnTheCard {
+    /// 这个进程还没在这张卡上借过：下一笔连驱动那一份一起要放得下。
+    NotRegistered,
+    /// 借过、记成登记了，设备还没起好：驱动那一份许出去了、还没占在卡上。
+    RegisteredDeviceNotStarted,
+    /// 设备起好了：驱动那一份占在卡上。
+    DeviceStarted,
+}
+
+/// 这个进程在一张卡上记着的：借着的合计（MiB）与驱动那一份走到哪一步。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct BookedByThisProcessOnACard {
+    pub leases_mebibytes: u64,
+    pub driver_share: DriverShareOnTheCard,
+}
+
+/// 这个进程此刻能在一张卡上占多少（MiB，文件头「口径」里的 M）：现读的空闲，加上这个进程实际占着的里属于它许出去的那一部分
+/// （借着的，加登记了时驱动那一份）。实际占着的比许出去的多（分配器按块拿、别的入口起的设备）只算到许出去的那么多；
+/// 读不到实际占着的，借着的当一点都没建，起好了设备的驱动那一份当已经在卡上。
 #[must_use]
-pub fn memory_all_registered_processes_may_hold_mebibytes(
-    previous_estimate_mebibytes: Option<u64>,
-    free_memory_read_now_mebibytes: u64,
-    booked_by_live_processes_mebibytes: u64,
+pub fn memory_this_process_may_hold_mebibytes(
+    reading: CardMemoryReading,
+    booked: BookedByThisProcessOnACard,
+    mebibytes_left_for_the_driver: u64,
 ) -> u64 {
-    let lowest = free_memory_read_now_mebibytes;
-    let highest = free_memory_read_now_mebibytes.saturating_add(booked_by_live_processes_mebibytes);
-    match previous_estimate_mebibytes {
-        Some(previous) => previous.clamp(lowest, highest),
-        None => lowest,
+    let promised_mebibytes = match booked.driver_share {
+        DriverShareOnTheCard::NotRegistered => booked.leases_mebibytes,
+        DriverShareOnTheCard::RegisteredDeviceNotStarted | DriverShareOnTheCard::DeviceStarted => {
+            booked
+                .leases_mebibytes
+                .saturating_add(mebibytes_left_for_the_driver)
+        }
+    };
+    let occupied_and_promised_mebibytes = match reading.occupied_by_this_process_mebibytes {
+        Some(occupied) => occupied.min(promised_mebibytes),
+        None => match booked.driver_share {
+            DriverShareOnTheCard::DeviceStarted => {
+                mebibytes_left_for_the_driver.min(promised_mebibytes)
+            }
+            DriverShareOnTheCard::NotRegistered
+            | DriverShareOnTheCard::RegisteredDeviceNotStarted => 0,
+        },
+    };
+    reading
+        .free_mebibytes
+        .saturating_add(occupied_and_promised_mebibytes)
+}
+
+/// 一张卡此刻对这个进程：这一笔最多能借多少，与这张卡空着时最多能借多少（MiB）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CardRoomForThisProcess {
+    pub room_now_mebibytes: u64,
+    /// 同一个算法拿卡的总量当这个进程能占的、借着的当 0：要的比它大，等多久都放不下。
+    pub ceiling_mebibytes: u64,
+}
+
+/// 一张卡此刻能借给这个进程的这一笔（文件头「口径」）：`capacity_of_memory_this_process_may_hold` 把这个进程能占的
+/// （[`memory_this_process_may_hold_mebibytes`]）换成它一共能占的（卡池照 [`card_ledger_capacity_mebibytes`] 算），减去借着的。
+#[must_use]
+pub fn card_room_for_this_process(
+    reading: CardMemoryReading,
+    booked: BookedByThisProcessOnACard,
+    mebibytes_left_for_the_driver: u64,
+    capacity_of_memory_this_process_may_hold: &dyn Fn(u64) -> u64,
+) -> CardRoomForThisProcess {
+    let may_hold_mebibytes =
+        memory_this_process_may_hold_mebibytes(reading, booked, mebibytes_left_for_the_driver);
+    CardRoomForThisProcess {
+        room_now_mebibytes: capacity_of_memory_this_process_may_hold(may_hold_mebibytes)
+            .saturating_sub(booked.leases_mebibytes),
+        ceiling_mebibytes: capacity_of_memory_this_process_may_hold(reading.total_mebibytes),
     }
 }
 
-/// 账上借一次显存的结局（[`decide_booking_by_priority`]）。
+/// 借一次显存的结局（[`decide_booking_by_priority`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum BookingDecision {
-    /// 记在这张卡上：照卡在账里的先后，第一张「已记的 + 要的 ≤ 容量」的卡（值是它在账里的次序）。
+    /// 记在这张卡上：照卡的先后，第一张此刻放得下的卡（值是它在给的表里的次序）。
     BookOnTheCardAt { card_position: usize },
-    /// 此刻每张卡都放不下，而至少一张卡空着时放得下：等别的借据还回来。
-    WaitForMemoryToBeReturned,
-    /// 要的比每张卡的容量都大（一张卡都没有也算）：等多久都放不下。
-    NeverFits { largest_capacity_mebibytes: u64 },
+    /// 此刻每张卡都放不下，而至少一张卡空着时放得下：借不到（不等）。
+    NoRoomNow,
+    /// 要的比每张卡空着时能借的都大（一张卡都没有也算）：等多久都放不下。
+    NeverFits { largest_ceiling_mebibytes: u64 },
 }
 
-/// 一次借 `need_mebibytes` 记在哪张卡上：`capacities_mebibytes` 是每张卡的容量、`booked_mebibytes` 是每张卡此刻已记的，次序就是优先级。
-///
-/// # Panics
-/// 两张表的卡数不同。
+/// 一次借 `need_mebibytes` 记在哪张卡上：`rooms` 是每张卡此刻对这个进程的，次序就是优先级。
 #[must_use]
 pub fn decide_booking_by_priority(
-    capacities_mebibytes: &[u64],
-    booked_mebibytes: &[u64],
+    rooms: &[CardRoomForThisProcess],
     need_mebibytes: u64,
 ) -> BookingDecision {
-    assert_eq!(
-        capacities_mebibytes.len(),
-        booked_mebibytes.len(),
-        "容量表与已记表要逐张卡对上"
-    );
-    let largest_capacity_mebibytes = capacities_mebibytes.iter().copied().max().unwrap_or(0);
-    if need_mebibytes > largest_capacity_mebibytes {
+    let largest_ceiling_mebibytes = rooms
+        .iter()
+        .map(|room| room.ceiling_mebibytes)
+        .max()
+        .unwrap_or(0);
+    if need_mebibytes > largest_ceiling_mebibytes {
         return BookingDecision::NeverFits {
-            largest_capacity_mebibytes,
+            largest_ceiling_mebibytes,
         };
     }
-    match capacities_mebibytes
+    match rooms
         .iter()
-        .zip(booked_mebibytes)
-        .position(|(capacity, booked)| booked.saturating_add(need_mebibytes) <= *capacity)
+        .position(|room| need_mebibytes <= room.room_now_mebibytes)
     {
         Some(card_position) => BookingDecision::BookOnTheCardAt { card_position },
-        None => BookingDecision::WaitForMemoryToBeReturned,
+        None => BookingDecision::NoRoomNow,
     }
 }
 
-/// 要的比每张卡的容量都大：配了 GPU 就一定用 GPU，调用方据此判红，不悄悄换小。
+/// 要的比每张卡空着时能借的都大：配了 GPU 就一定用 GPU，调用方据此判红，不悄悄换小。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct NeedLargerThanEveryCard {
     pub need_mebibytes: u64,
-    pub largest_capacity_mebibytes: u64,
+    pub largest_ceiling_mebibytes: u64,
 }
 
 /// 在 `0..=most_states` 里找最多的状态数，它要的显存（`mebibytes_for_states`，MiB）不超过 `capacity_mebibytes`：
@@ -195,21 +243,22 @@ pub fn most_states_whose_need_fits_the_capacity(
     fitting
 }
 
-/// 产品路径在一张卡上当场（不等）借草稿区的结局（[`place_states_on_the_cards_that_book_now`]）。
+/// 产品路径在一张卡上当场借草稿区的结局（[`place_states_on_the_cards_that_book_now`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum ScratchAttemptOnACard {
-    /// 前面的卡已经装下了全部要判的状态（或这张卡按它的额度一个状态都带不了）：没去借。
+    /// 前面的卡已经装下了全部要判的状态（或这张卡一个状态都带不了）：没去借。
     NothingToPlace,
-    /// 借到了 `states` 个状态的草稿区（在锁里照此刻的估计压过，不多于要装的）。
+    /// 借到了 `states` 个状态的草稿区（在锁里照此刻能借的压过，不多于要装的）。
     Booked { states: u64 },
-    /// 此刻放不下（别的借据占着）：这张卡这一趟不装状态，不等它；`need_mebibytes` 是压过之后要借的量。
+    /// 此刻连一个状态都放不下（这个进程别的借据或卡上别的用户占着），空着时放得下：这张卡这一趟不装状态；
+    /// `need_mebibytes` 是一个状态要借的量。
     NoRoomNow { need_mebibytes: u64 },
-    /// 这张卡的账连一个状态的草稿区都放不下，等多久都一样。
-    CannotHoldOneState { capacity_mebibytes: u64 },
+    /// 这张卡空着时连一个状态的草稿区都放不下，等多久都一样。
+    CannotHoldOneState { ceiling_mebibytes: u64 },
 }
 
 /// 产品路径按卡的先后把 `pending_states` 个状态装到此刻借得到的卡上：每张卡要装的是剩下的与它带得了的（`states_each_card_holds`）
-/// 里小的那个，交 `attempt_now(卡在表里的次序, 要装的状态数)` 当场借一次、不等；借到几个就从剩下的里减几个，
+/// 里小的那个，交 `attempt_now(卡在表里的次序, 要装的状态数)` 当场借一次；借到几个就从剩下的里减几个，
 /// 借不到的卡不装，它那一份留给后面的卡。装满了的卡之后剩下的状态排队等前面的卡判完（不算在任何一张卡上），
 /// 后面的卡要装的是 0，不去借。交回每张卡的结局，与 `states_each_card_holds` 逐张对上。
 ///
@@ -249,21 +298,21 @@ pub fn place_states_on_the_cards_that_book_now(
 /// 每张卡当场试过一次之后怎么办（[`decide_after_attempting_every_card`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum ScratchPlacementDecision {
-    /// 至少一张卡借到了：就在借到的那几张上判，借不到的不等。
+    /// 至少一张卡借到了：就在借到的那几张上判。
     JudgeOnTheCardsBooked,
-    /// 一张都没借到，而至少一张卡空着时放得下：等别的借据还（`need_mebibytes` 是第一张放不下的卡要借的量）。
-    WaitForRoom { need_mebibytes: u64 },
-    /// 一张都没借到，每张卡的账连一个状态都放不下：等多久都一样，调用方判红。
-    NoCardHoldsOneState { largest_capacity_mebibytes: u64 },
+    /// 一张都没借到，而至少一张卡空着时放得下：借不到（不等），调用方报本次未跑（`need_mebibytes` 是第一张放不下的卡要借的量）。
+    NoRoomOnAnyCardNow { need_mebibytes: u64 },
+    /// 一张都没借到，每张卡空着时连一个状态都放不下：等多久都一样，调用方判红。
+    NoCardHoldsOneState { largest_ceiling_mebibytes: u64 },
 }
 
-/// 一张借到了就判；一张都没借到时，有放不下而空着时放得下的就等，没有就判成没有一张卡装得下一个状态。
+/// 一张借到了就判；一张都没借到时，有空着时放得下的就是此刻借不到，没有就判成没有一张卡装得下一个状态。
 #[must_use]
 pub fn decide_after_attempting_every_card(
     attempts: &[ScratchAttemptOnACard],
 ) -> ScratchPlacementDecision {
     let mut first_need_without_room = None;
-    let mut largest_capacity_mebibytes = 0u64;
+    let mut largest_ceiling_mebibytes = 0u64;
     for attempt in attempts {
         match attempt {
             ScratchAttemptOnACard::Booked { .. } => {
@@ -272,16 +321,16 @@ pub fn decide_after_attempting_every_card(
             ScratchAttemptOnACard::NoRoomNow { need_mebibytes } => {
                 first_need_without_room = first_need_without_room.or(Some(*need_mebibytes));
             }
-            ScratchAttemptOnACard::CannotHoldOneState { capacity_mebibytes } => {
-                largest_capacity_mebibytes = largest_capacity_mebibytes.max(*capacity_mebibytes);
+            ScratchAttemptOnACard::CannotHoldOneState { ceiling_mebibytes } => {
+                largest_ceiling_mebibytes = largest_ceiling_mebibytes.max(*ceiling_mebibytes);
             }
             ScratchAttemptOnACard::NothingToPlace => {}
         }
     }
     match first_need_without_room {
-        Some(need_mebibytes) => ScratchPlacementDecision::WaitForRoom { need_mebibytes },
+        Some(need_mebibytes) => ScratchPlacementDecision::NoRoomOnAnyCardNow { need_mebibytes },
         None => ScratchPlacementDecision::NoCardHoldsOneState {
-            largest_capacity_mebibytes,
+            largest_ceiling_mebibytes,
         },
     }
 }
@@ -311,7 +360,7 @@ pub enum GpuCardsSource {
     /// 这一趟没有一类计算走 GPU，不要卡。
     NoCardNeeded,
     /// 这个进程起过 GPU 用例共用的卡池（`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::GpuCardPool`）：照池里的卡用，
-    /// 每张卡这个进程只有一个设备，驱动那一份在跨进程的显存账上登记了；第 ① 段判器开的草稿区按装到这张卡上的状态数借
+    /// 每张卡这个进程只有一个设备、只在借到的那张卡上起；第 ① 段判器开的草稿区按装到这张卡上的状态数借
     /// （第 10v 批：主 agent 定照 state_count 记）。
     TheCardPoolOfThisProcess,
     /// 这个进程没起过卡池：这一趟自己在点名的卡上起设备（`gpu_unit_checks::gpu_cards_by_priority` 或 `usable_gpu_cards`），不进显存账。
@@ -370,29 +419,37 @@ pub fn gpu_not_run_line(test_name: &str, reason: &str) -> String {
     format!("{GPU_NOT_RUN_MARKER} test={test_name} reason={reason}")
 }
 
-/// 等满了还借不到的那一句（GPU 用例借卡与产品路径借草稿区同一句）。
+/// 此刻借不到的那一句（GPU 用例借卡与产品路径借缓冲同一句）。
 #[must_use]
-pub fn reason_for_a_booking_that_waited_past_the_longest_wait(
+pub fn reason_for_a_booking_without_room_now(
     need_mebibytes: u64,
-    waited: Duration,
-    card_capacities_mebibytes: &[u64],
+    card_rooms_mebibytes: &[u64],
 ) -> String {
     format!(
-        "等了 {} 秒，池里没有一张卡空得出 {need_mebibytes} MiB（每张卡的账 {card_capacities_mebibytes:?}；这台机器上别的 GPU 用例或测试进程一直占着）",
-        waited.as_secs()
+        "显存不够：此刻池里没有一张卡借得出 {need_mebibytes} MiB（这个进程看每张卡此刻能借的 {card_rooms_mebibytes:?}：\
+         现读的空闲减这个进程借了还没占的；卡上别的 GPU 用户或这个进程里别的用例占着）"
+    )
+}
+
+/// wgpu 在卡上报 Out of Memory 时「本次未跑」那一句（`what_ran_out` 说是哪一段、哪张卡）：账只记这个进程借的，
+/// 两个进程同一刻读到同一份空闲、一起借超了的那一形由这一句兜底（文件头）。
+#[must_use]
+pub fn reason_for_running_out_of_gpu_memory(what_ran_out: &str) -> String {
+    format!(
+        "显存不够：{what_ran_out}上 wgpu 报了 Out of Memory（账只记这个进程借的，卡上别的进程同一刻借走了；判过的块在库里）"
     )
 }
 
-/// 产品路径借草稿区借不到的结局（[`scratch_booking_refusal_outcome`]）。
+/// 产品路径借缓冲借不到的结局（[`scratch_booking_refusal_outcome`]）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum ScratchBookingRefusalOutcome {
-    /// 放得下而等满了：打这一行，这一趟照产品路径的「本次未跑」交回（判过的块在库里，没判的留给下一趟）。
+    /// 空着时放得下而此刻借不到：打这一行，这一趟照产品路径的「本次未跑」交回（判过的块在库里，没判的留给下一趟）。
     NotRun { not_run_line: String },
-    /// 要的比那张卡的账大、账本读写不了：判红。
+    /// 要的比那张卡空着时能借的都大、卡上的显存读不出：判红。
     Red { reason: String },
 }
 
-/// 产品路径借草稿区被拒时怎么报：等满了照 GPU 对拍用例那一路报「本次未跑」（同一个记号、同一句），别的判红。
+/// 产品路径借缓冲被拒时怎么报：此刻借不到照 GPU 对拍用例那一路报「本次未跑」（同一个记号、同一句），别的判红。
 #[must_use]
 pub fn scratch_booking_refusal_outcome(
     run_name: &str,
@@ -400,1054 +457,442 @@ pub fn scratch_booking_refusal_outcome(
     refusal: &BookingRefusal,
 ) -> ScratchBookingRefusalOutcome {
     match refusal {
-        BookingRefusal::WaitedPastTheLongestWait {
+        BookingRefusal::NoCardHasRoomNow {
             need_mebibytes,
-            waited,
-            card_capacities_mebibytes,
+            card_rooms_mebibytes,
         } => ScratchBookingRefusalOutcome::NotRun {
             not_run_line: gpu_not_run_line(
                 run_name,
-                &reason_for_a_booking_that_waited_past_the_longest_wait(
-                    *need_mebibytes,
-                    *waited,
-                    card_capacities_mebibytes,
-                ),
+                &reason_for_a_booking_without_room_now(*need_mebibytes, card_rooms_mebibytes),
             ),
         },
         BookingRefusal::NeedLargerThanEveryCard(refusal) => ScratchBookingRefusalOutcome::Red {
             reason: format!(
-                "{run_name}：第 {card_position} 张卡的草稿区要借 {} MiB，那张卡的账只有 {} MiB",
-                refusal.need_mebibytes, refusal.largest_capacity_mebibytes
+                "{run_name}：第 {card_position} 张卡要借 {} MiB，那张卡空着时也只借得出 {} MiB",
+                refusal.need_mebibytes, refusal.largest_ceiling_mebibytes
             ),
         },
-        BookingRefusal::LedgerFileUnusable(error) => ScratchBookingRefusalOutcome::Red {
-            reason: format!(
-                "{run_name}：显存账本 {} 用不了：{}",
-                error.path.display(),
-                error.reason
-            ),
+        BookingRefusal::CardMemoryUnreadable { reason } => ScratchBookingRefusalOutcome::Red {
+            reason: format!("{run_name}：卡上的显存读不出：{reason}"),
         },
     }
 }
 
-/// 一笔借不到、等满 [`LONGEST_WAIT_FOR_GPU_MEMORY`] 才放弃：一条 GPU 用例在卡上判一条流以分钟计，等半小时还空不出来，
-/// 说明这台机器上别的 GPU 测试进程占得太久，这条用例报「本次未跑」，不干等到整趟被外面的限时掐掉（推的，没量过用例最长占多久）。
-pub const LONGEST_WAIT_FOR_GPU_MEMORY: Duration = Duration::from_secs(30 * 60);
-/// 等别的进程还显存时隔多久重读一次账本：别的进程还了叫不醒这个进程里等着的线程（同一个进程里还的当场叫醒，不等这一下）。
-pub const POLL_INTERVAL_FOR_MEMORY_RETURNED_BY_OTHER_PROCESSES: Duration = Duration::from_secs(1);
-/// 工作根下放账本的子目录。
-pub const GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME: &str = "gpu-memory-ledgers";
-/// 工作根的环境变量与它没设时在家目录下的默认位置：定义只在 `.claude/hooks/lib_work_root.py` 一处（`WORK_ROOT_VARIABLE`、
-/// `DEFAULT_WORK_ROOT`），这里照它取，两边要一起改。
-pub const WORK_ROOT_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_WORK_ROOT";
-pub const DEFAULT_WORK_ROOT_UNDER_THE_HOME_DIRECTORY: &str = ".local/share/singlefs/work";
-
-/// 账本放在哪个目录：工作根（`work_root_variable` 是 [`WORK_ROOT_ENVIRONMENT_VARIABLE`] 的值，设了而不空就用它，开头的 `~` 照家目录展开；
-/// 没设是家目录下的 [`DEFAULT_WORK_ROOT_UNDER_THE_HOME_DIRECTORY`]）下的 [`GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME`]。
-/// 要家目录而 `home_directory` 没给，交 `None`。
-#[must_use]
-pub fn gpu_memory_ledger_directory(
-    work_root_variable: Option<&std::ffi::OsStr>,
-    home_directory: Option<&std::ffi::OsStr>,
-) -> Option<PathBuf> {
-    let work_root = match work_root_variable.filter(|value| !value.is_empty()) {
-        Some(value) => {
-            let text = value.to_string_lossy();
-            match text.strip_prefix('~') {
-                Some(rest) if rest.is_empty() || rest.starts_with('/') => {
-                    PathBuf::from(home_directory?).join(rest.trim_start_matches('/'))
-                }
-                Some(_) | None => PathBuf::from(value),
-            }
-        }
-        None => PathBuf::from(home_directory?).join(DEFAULT_WORK_ROOT_UNDER_THE_HOME_DIRECTORY),
-    };
-    Some(work_root.join(GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME))
-}
-
-/// 这个进程的账本目录（照 [`gpu_memory_ledger_directory`]，读进程的 `SINGLEFS_WORK_ROOT` 与 `HOME`）。
-#[must_use]
-pub fn gpu_memory_ledger_directory_of_this_process() -> Option<PathBuf> {
-    gpu_memory_ledger_directory(
-        std::env::var_os(WORK_ROOT_ENVIRONMENT_VARIABLE).as_deref(),
-        std::env::var_os("HOME").as_deref(),
-    )
-}
-
-/// 账本里记的一个进程：进程号加它的启动时刻（`/proc/<进程号>/stat` 第 22 项，开机以来的时钟滴答数）；
-/// 进程号会被后起的进程重用，两样都对上才是同一个进程。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub struct RecordedProcess {
-    pub process_identifier: u32,
-    pub start_time_in_clock_ticks: u64,
-}
-
-impl RecordedProcess {
-    /// 跑这段代码的进程；读不到 `/proc/self/stat`（不是 Linux）交 `None`。
-    #[must_use]
-    pub fn this_process() -> Option<Self> {
-        let status_line = std::fs::read_to_string("/proc/self/stat").ok()?;
-        Some(Self {
-            process_identifier: std::process::id(),
-            start_time_in_clock_ticks: start_time_in_clock_ticks_from_the_process_status_line(
-                &status_line,
-            )?,
-        })
-    }
-}
-
-/// `/proc/<进程号>/stat` 的原文里进程的启动时刻（第 22 项）。第 2 项是括号里的进程名，名字里可以有空白与括号，从最后一个 `)` 之后数。
-#[must_use]
-pub fn start_time_in_clock_ticks_from_the_process_status_line(status_line: &str) -> Option<u64> {
-    let (_, after_the_name) = status_line.rsplit_once(')')?;
-    // 最后一个 `)` 之后第一项是第 3 项（进程状态），第 22 项是之后的第 20 项
-    after_the_name.split_whitespace().nth(19)?.parse().ok()
-}
-
-/// 照 `/proc/<进程号>/stat` 的原文判记账的那个进程还在不在：状态（第 3 项，最后一个 `)` 之后第一项）是 `Z`（退出了或被杀了、
-/// 父进程还没收尸）或 `X`（正在被收走）就是不在了（驱动在进程退出时已经还了它的显存，不必等收尸）；启动时刻对不上
-/// （进程号被别的进程重用了）也是不在了。认不出状态或启动时刻的当它还在（判不了就照记的算，宁可多等）。
-#[must_use]
-pub fn process_is_alive_judged_from_its_status_line(
-    status_line: &str,
-    process: RecordedProcess,
-) -> bool {
-    let state = status_line
-        .rsplit_once(')')
-        .and_then(|(_, after_the_name)| after_the_name.split_whitespace().next());
-    match state {
-        Some("Z" | "X") => false,
-        Some(_) | None => start_time_in_clock_ticks_from_the_process_status_line(status_line)
-            .is_none_or(|start_time| start_time == process.start_time_in_clock_ticks),
-    }
-}
-
-/// 记账的那个进程还在不在：`/proc/<进程号>/stat` 不在就是不在了；在，照 [`process_is_alive_judged_from_its_status_line`] 判；
-/// 读的时候出了别的错，当它还在（判不了就照记的算，宁可多等，不把别人还占着的显存算成空的）。
-#[must_use]
-pub fn process_is_alive_by_its_entry_in_the_process_filesystem(process: RecordedProcess) -> bool {
-    let path = format!("/proc/{}/stat", process.process_identifier);
-    match std::fs::read_to_string(&path) {
-        Ok(status_line) => process_is_alive_judged_from_its_status_line(&status_line, process),
-        Err(error) => error.kind() != std::io::ErrorKind::NotFound,
-    }
-}
-
-/// 账本里的一笔借出去的显存。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-struct RecordedLease {
-    process: RecordedProcess,
-    lease_identifier: u64,
-    mebibytes: u64,
-}
-
-/// 一张卡的账本文件里记的。
-#[derive(Clone, Debug, Default, PartialEq, Eq)]
-struct CardLedgerRecords {
-    /// 记账的进程合起来能在这张卡上占多少显存（MiB）：最近一个开账的进程照 [`memory_all_registered_processes_may_hold_mebibytes`] 算的。
-    memory_all_registered_processes_may_hold_mebibytes: Option<u64>,
-    /// 在这张卡上登记了的进程：每个各在卡上起了（或马上要起）一个设备。
-    registered_processes: Vec<RecordedProcess>,
-    leases: Vec<RecordedLease>,
-}
-
-/// 账本里估计那一行的键：沿用第 10s 批的写法（那时记的是头一个起池的进程读到的空闲显存），新旧两版读写得了同一份账本。
-const MEMORY_ESTIMATE_LINE_KEY: &str = "free_memory_after_quiescing_mebibytes";
-const PROCESS_LINE_KEY: &str = "process";
-const LEASE_LINE_KEY: &str = "lease";
-
-impl CardLedgerRecords {
-    /// 账本文件的原文：一行一条，`free_memory_after_quiescing_mebibytes <MiB>`、`process <进程号> <启动时刻>`、
-    /// `lease <进程号> <启动时刻> <借据号> <MiB>`。
-    fn parse(text: &str) -> Result<Self, String> {
-        let mut records = Self::default();
-        for line in text.lines().filter(|line| !line.trim().is_empty()) {
-            let fields: Vec<&str> = line.split_whitespace().collect();
-            let number = |position: usize| -> Result<u64, String> {
-                fields
-                    .get(position)
-                    .and_then(|field| field.parse().ok())
-                    .ok_or_else(|| format!("账本的这一行认不出：「{line}」"))
-            };
-            let process = |first: usize| -> Result<RecordedProcess, String> {
-                Ok(RecordedProcess {
-                    process_identifier: u32::try_from(number(first)?)
-                        .map_err(|_| format!("进程号装不进 u32：「{line}」"))?,
-                    start_time_in_clock_ticks: number(first + 1)?,
-                })
-            };
-            match (fields[0], fields.len()) {
-                (MEMORY_ESTIMATE_LINE_KEY, 2) => {
-                    records.memory_all_registered_processes_may_hold_mebibytes = Some(number(1)?);
-                }
-                (PROCESS_LINE_KEY, 3) => records.registered_processes.push(process(1)?),
-                (LEASE_LINE_KEY, 5) => records.leases.push(RecordedLease {
-                    process: process(1)?,
-                    lease_identifier: number(3)?,
-                    mebibytes: number(4)?,
-                }),
-                (_, _) => return Err(format!("账本的这一行认不出：「{line}」")),
-            }
-        }
-        Ok(records)
-    }
-
-    fn to_text(&self) -> String {
-        let mut text = String::new();
-        if let Some(estimate) = self.memory_all_registered_processes_may_hold_mebibytes {
-            text.push_str(&format!("{MEMORY_ESTIMATE_LINE_KEY} {estimate}\n"));
-        }
-        for process in &self.registered_processes {
-            text.push_str(&format!(
-                "{PROCESS_LINE_KEY} {} {}\n",
-                process.process_identifier, process.start_time_in_clock_ticks
-            ));
-        }
-        for lease in &self.leases {
-            text.push_str(&format!(
-                "{LEASE_LINE_KEY} {} {} {} {}\n",
-                lease.process.process_identifier,
-                lease.process.start_time_in_clock_ticks,
-                lease.lease_identifier,
-                lease.mebibytes
-            ));
-        }
-        text
-    }
-
-    /// 去掉记账进程已经不在了的那几条（它起的池与借的显存都当没记）。
-    fn without_the_records_of_dead_processes(
-        mut self,
-        is_alive: &dyn Fn(RecordedProcess) -> bool,
-    ) -> Self {
-        self.registered_processes
-            .retain(|process| is_alive(*process));
-        self.leases.retain(|lease| is_alive(lease.process));
-        self
-    }
-
-    /// `viewer` 这个进程看这张卡已经记了多少（MiB）：每一笔借出去的，加别的每个登记了的进程各一份驱动的固定开销
-    /// （`viewer` 自己那一份已经从它的容量里扣过）。
-    fn booked_mebibytes_seen_by(
-        &self,
-        viewer: RecordedProcess,
-        mebibytes_each_process_keeps_on_a_card: u64,
-    ) -> u64 {
-        let other_processes = self
-            .registered_processes
-            .iter()
-            .filter(|process| **process != viewer)
-            .count();
-        self.leases
-            .iter()
-            .map(|lease| lease.mebibytes)
-            .sum::<u64>()
-            .saturating_add(
-                mebibytes_each_process_keeps_on_a_card
-                    .saturating_mul(u64::try_from(other_processes).expect("进程数装得进 u64")),
-            )
-    }
-
-    /// 账上活着的进程一共记了多少（MiB）：`viewer` 看到的，加上 `viewer` 自己登记了时它的那一份驱动开销。
-    fn booked_mebibytes_by_every_live_process(
-        &self,
-        viewer: RecordedProcess,
-        mebibytes_each_process_keeps_on_a_card: u64,
-    ) -> u64 {
-        let own_share = if self.registered_processes.contains(&viewer) {
-            mebibytes_each_process_keeps_on_a_card
-        } else {
-            0
-        };
-        self.booked_mebibytes_seen_by(viewer, mebibytes_each_process_keeps_on_a_card)
-            .saturating_add(own_share)
-    }
-}
-
-/// 账本文件读写出错：哪个文件、什么错。
-#[derive(Clone, Debug, PartialEq, Eq)]
-pub struct LedgerFileError {
-    pub path: PathBuf,
-    pub reason: String,
-}
-
-/// 记账的设置：账本放哪、这个进程是谁、怎么判记账进程还在不在（测试换成假的）、每个进程各在卡上付多少驱动的固定开销、
-/// 等别的进程还显存时隔多久重读、最多等多久。
-pub struct LedgerSettings {
-    pub directory: PathBuf,
-    pub this_process: RecordedProcess,
-    pub is_alive: Box<dyn Fn(RecordedProcess) -> bool + Send + Sync>,
-    pub mebibytes_each_process_keeps_on_a_card: u64,
-    pub poll_interval_for_memory_returned_by_other_processes: Duration,
-    pub longest_wait: Duration,
-}
-
-/// 一张卡看这个进程的容量（MiB）：`(卡在账里的次序, 账本里此刻的估计)` → 容量（这个进程自己的驱动那一份已经扣掉，
-/// 卡池照 [`card_ledger_capacity_mebibytes`] 算）。
-pub type CardCapacityForThisProcess = Box<dyn Fn(usize, u64) -> u64 + Send + Sync>;
-
-/// 开账不成：账本文件读写不了，或那张卡的空闲显存读不出（哪张卡的账本、为什么）。
-#[derive(Clone, Debug, PartialEq, Eq)]
-pub enum LedgerOpeningRefusal {
-    LedgerFileUnusable(LedgerFileError),
-    FreeMemoryUnreadable { file_stem: String, reason: String },
-}
-
 /// 借不到的几种结局。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum BookingRefusal {
-    /// 要的比每张卡的容量都大：调用方判红。
+    /// 要的比每张卡空着时能借的都大：调用方判红。
     NeedLargerThanEveryCard(NeedLargerThanEveryCard),
-    /// 放得下而等满了 [`LedgerSettings::longest_wait`] 还没空出来：调用方报「本次未跑」。带着最后一次判的时候每张卡的容量。
-    WaitedPastTheLongestWait {
+    /// 空着时放得下而此刻每张卡都借不出：调用方报「本次未跑」（不等）。带着这一刻每张卡能借的。
+    NoCardHasRoomNow {
         need_mebibytes: u64,
-        waited: Duration,
-        card_capacities_mebibytes: Vec<u64>,
+        card_rooms_mebibytes: Vec<u64>,
     },
-    /// 账本文件读写不了：调用方判红。
-    LedgerFileUnusable(LedgerFileError),
+    /// 卡上的显存读不出、读到的卡数与池对不上：调用方判红。
+    CardMemoryUnreadable { reason: String },
 }
 
-/// 一次记账记什么。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-enum WhatIsBooked {
-    /// 一笔借据；这个进程在那张卡上还没登记的，连它的驱动那一份一起登记。
-    LeaseOf { need_mebibytes: u64 },
-    /// 不借，只把这个进程登记到那张卡上（记它的驱动那一份）。
-    DriverShareOnly,
-}
+/// 读每张卡此刻的显存（次序与卡池一样）：卡池用 `nvidia-smi` 读，单测换成假的。
+pub type CardMemoryReader = Box<dyn Fn() -> Result<Vec<CardMemoryReading>, String> + Send + Sync>;
 
-/// 在一张卡的锁里记什么：照调用方给的量记，或按状态数借、在锁里照此刻的估计压过再记。
-#[derive(Clone, Copy)]
-enum WhatIsBookedOnOneCard<'need> {
-    AsGiven(WhatIsBooked),
-    /// 借 `want_states` 个状态的草稿区（第 ① 段的判器），`mebibytes_for_states` 是几个状态要多少显存（随状态数单调不减）：
-    /// 在锁里照此刻这张卡看这个进程的容量压（[`most_states_whose_need_fits_the_capacity`]），压出来的状态数要的放得下才记。
-    StatesCompressedToTheCapacity {
-        want_states: u64,
-        mebibytes_for_states: &'need dyn Fn(u64) -> u64,
-    },
+/// 一张卡上这个进程一共能占多少（MiB）：`(卡在池里的次序, 这个进程能占的)` → 容量（这个进程自己的驱动那一份已经扣掉，
+/// 卡池照 [`card_ledger_capacity_mebibytes`] 算）。
+pub type CardCapacityForThisProcess = Box<dyn Fn(usize, u64) -> u64 + Send + Sync>;
+
+/// 这个进程在一张卡上记着的：每笔借据（借据号 → MiB）与驱动那一份走到哪一步。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct BooksOfACard {
+    leases: BTreeMap<u64, u64>,
+    driver_share: DriverShareOnTheCard,
 }
 
-impl WhatIsBookedOnOneCard<'_> {
-    /// 照这张卡此刻看这个进程的容量，这一笔要记多少（MiB）、按状态数借的借几个状态。一个状态都放不下时照一个状态算：
-    /// 要的比容量大，判成永远放不下。
-    fn need_within_the_capacity(self, capacity_mebibytes: u64) -> (u64, Option<u64>) {
-        match self {
-            Self::AsGiven(WhatIsBooked::LeaseOf { need_mebibytes }) => (need_mebibytes, None),
-            Self::AsGiven(WhatIsBooked::DriverShareOnly) => (0, None),
-            Self::StatesCompressedToTheCapacity {
-                want_states,
-                mebibytes_for_states,
-            } => {
-                let states_that_fit = most_states_whose_need_fits_the_capacity(
-                    usize::try_from(want_states).unwrap_or(usize::MAX),
-                    capacity_mebibytes,
-                    &|states| {
-                        mebibytes_for_states(u64::try_from(states).expect("状态数装得进 u64"))
-                    },
-                );
-                let states_to_book = u64::try_from(states_that_fit)
-                    .expect("压出来的不多于要借的，装得进 u64")
-                    .max(1);
-                (mebibytes_for_states(states_to_book), Some(states_to_book))
-            }
+impl BooksOfACard {
+    fn booked(&self) -> BookedByThisProcessOnACard {
+        BookedByThisProcessOnACard {
+            leases_mebibytes: self.leases.values().sum(),
+            driver_share: self.driver_share,
         }
     }
-}
 
-/// 一张卡此刻（在它的锁里）记得记不上（[`GpuMemoryLedger::book_on_the_card_if_it_still_fits`]）。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-enum CardBookingNow {
-    Booked(BookedOnACard),
-    /// 别的借据占着，此刻放不下。
-    NoRoomNow {
-        need_mebibytes: u64,
-    },
-    /// 要的比这张卡看这个进程的容量大，等多久都放不下。
-    NeverFits {
-        capacity_mebibytes: u64,
-    },
+    /// 记一笔借据；这个进程还没在这张卡上登记的，一并登记（设备还没起）。
+    fn book_a_lease(&mut self, lease_identifier: u64, mebibytes: u64) {
+        let previous = self.leases.insert(lease_identifier, mebibytes);
+        assert!(
+            previous.is_none(),
+            "借据号 {lease_identifier} 记了两次：借据号由原子计数器发，一个号只发一次"
+        );
+        match self.driver_share {
+            DriverShareOnTheCard::NotRegistered => {
+                self.driver_share = DriverShareOnTheCard::RegisteredDeviceNotStarted;
+            }
+            DriverShareOnTheCard::RegisteredDeviceNotStarted
+            | DriverShareOnTheCard::DeviceStarted => {}
+        }
+    }
 }
 
-/// 记在哪几张卡里挑。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-enum WhichCards {
-    /// 照优先级第一张放得下的。
-    FirstThatFitsByPriority,
-    /// 只这一张。
-    OnlyTheCardAt { card_position: usize },
+/// 这个进程里几张卡的显存账（MiB）：借的时候在锁里现读每张卡的显存、照 [`card_room_for_this_process`] 判，放得下就记上，
+/// 放不下不等、交借不到；借据丢掉时销账。只记这个进程借的（文件头）。
+pub struct GpuMemoryLedger {
+    read_card_memory: CardMemoryReader,
+    capacity_for_this_process: CardCapacityForThisProcess,
+    mebibytes_each_process_keeps_on_a_card: u64,
+    books: Mutex<Vec<BooksOfACard>>,
+    next_lease_identifier: AtomicU64,
 }
 
-/// 记上的那一笔：在账里第几张卡、借据号（只登记驱动那一份的没有）、记了多少（MiB）、按状态数借的借了几个状态、
-/// 记的那一刻这张卡看这个进程的容量。
+/// 记上的那一笔：在池里第几张卡、借据号、记了多少（MiB）、按状态数借的借了几个状态、记的那一刻这张卡能借给这个进程的。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct BookedOnACard {
     card_position: usize,
-    lease_identifier: Option<u64>,
+    lease_identifier: u64,
     mebibytes_booked: u64,
     states_booked: Option<u64>,
-    capacity_mebibytes: u64,
-}
-
-/// 一张卡此刻看这个进程：容量与已经记了的（都在这张卡的锁里读）。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-struct CardSeenByThisProcess {
-    capacity_mebibytes: u64,
-    booked_mebibytes: u64,
-}
-
-/// 几张卡各一本跨进程的显存账（MiB）：借的时候照 [`decide_booking_by_priority`] 记上，放不下就等别的借据还，借据丢掉时销账。
-pub struct GpuMemoryLedger {
-    settings: LedgerSettings,
-    card_file_stems: Vec<String>,
-    capacity_for_this_process: CardCapacityForThisProcess,
-    next_lease_identifier: AtomicU64,
-    /// 这个进程里还过几次：等着的线程记下看到的次数，次数变了就不睡（还在它判完、睡下之前时不漏叫）。
-    returns_in_this_process: Mutex<u64>,
-    memory_returned: Condvar,
-}
-
-/// 锁住一张卡的账本、读出来（去掉死了的进程记的）、交 `change` 改，改过的写回去；交回 `change` 的结局。
-fn with_the_card_ledger_locked<Outcome>(
-    settings: &LedgerSettings,
-    file_stem: &str,
-    change: impl FnOnce(&mut CardLedgerRecords) -> Outcome,
-) -> Result<Outcome, LedgerFileError> {
-    let unusable = |path: &Path, reason: String| LedgerFileError {
-        path: path.to_path_buf(),
-        reason,
-    };
-    std::fs::create_dir_all(&settings.directory)
-        .map_err(|error| unusable(&settings.directory, format!("建不了目录：{error}")))?;
-    let lock_path = settings.directory.join(format!("{file_stem}.lock"));
-    let ledger_path = settings.directory.join(format!("{file_stem}.ledger"));
-    let lock = std::fs::OpenOptions::new()
-        .create(true)
-        .truncate(false)
-        .read(true)
-        .write(true)
-        .open(&lock_path)
-        .map_err(|error| unusable(&lock_path, format!("打不开锁文件：{error}")))?;
-    lock.lock()
-        .map_err(|error| unusable(&lock_path, format!("锁不上：{error}")))?;
-    let text = match std::fs::read_to_string(&ledger_path) {
-        Ok(text) => text,
-        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
-        Err(error) => return Err(unusable(&ledger_path, format!("读不了：{error}"))),
-    };
-    let as_read =
-        CardLedgerRecords::parse(&text).map_err(|reason| unusable(&ledger_path, reason))?;
-    let mut records = as_read
-        .clone()
-        .without_the_records_of_dead_processes(&settings.is_alive);
-    let outcome = change(&mut records);
-    if records != as_read {
-        let temporary_path = settings.directory.join(format!(
-            "{file_stem}.ledger.{}.writing",
-            settings.this_process.process_identifier
-        ));
-        std::fs::write(&temporary_path, records.to_text())
-            .map_err(|error| unusable(&temporary_path, format!("写不了：{error}")))?;
-        std::fs::rename(&temporary_path, &ledger_path)
-            .map_err(|error| unusable(&ledger_path, format!("换不上：{error}")))?;
-    }
-    drop(lock);
-    Ok(outcome)
-}
-
-/// 账本文件名的主干只许字母、数字、`.`、`-`。
-fn is_a_ledger_file_stem(file_stem: &str) -> bool {
-    !file_stem.is_empty()
-        && file_stem.chars().all(|character| {
-            character.is_ascii_alphanumeric() || character == '.' || character == '-'
-        })
+    room_mebibytes_when_booked: u64,
 }
 
 impl GpuMemoryLedger {
-    /// 开这个进程的账：在每张卡（`card_file_stems`，次序就是优先级）的账本锁里调 `read_free_memory_mebibytes(卡的次序)` 现读一次空闲显存，
-    /// 照 [`memory_all_registered_processes_may_hold_mebibytes`] 改账本里的估计；不登记这个进程（第一次借的时候才登记）。
-    /// 之后每一次借，容量都照 `capacity_for_this_process(卡的次序, 账本里此刻的估计)`。
-    ///
-    /// # Errors
-    /// 账本文件读写不了、认不出；某张卡的空闲显存读不出。
-    ///
-    /// # Panics
-    /// 文件名主干里有字母、数字、`.`、`-` 之外的字符。
-    pub fn open_reading_the_free_memory_inside_each_card_lock(
-        settings: LedgerSettings,
-        card_file_stems: Vec<String>,
-        read_free_memory_mebibytes: &dyn Fn(usize) -> Result<u64, String>,
+    /// 开这个进程的账：`card_count` 张卡（次序就是优先级），每次借照 `read_card_memory` 现读、照 `capacity_for_this_process` 算容量，
+    /// 每个进程在每张卡上付 `mebibytes_each_process_keeps_on_a_card` 的驱动开销。不读卡、不登记。
+    #[must_use]
+    pub fn new(
+        card_count: usize,
+        read_card_memory: CardMemoryReader,
         capacity_for_this_process: CardCapacityForThisProcess,
-    ) -> Result<Self, LedgerOpeningRefusal> {
-        for (position, file_stem) in card_file_stems.iter().enumerate() {
-            assert!(
-                is_a_ledger_file_stem(file_stem),
-                "账本文件名主干只许字母、数字、`.`、`-`：「{file_stem}」"
-            );
-            let this_process = settings.this_process;
-            let driver = settings.mebibytes_each_process_keeps_on_a_card;
-            with_the_card_ledger_locked(&settings, file_stem, |records| {
-                let free_now = read_free_memory_mebibytes(position)?;
-                records.memory_all_registered_processes_may_hold_mebibytes =
-                    Some(memory_all_registered_processes_may_hold_mebibytes(
-                        records.memory_all_registered_processes_may_hold_mebibytes,
-                        free_now,
-                        records.booked_mebibytes_by_every_live_process(this_process, driver),
-                    ));
-                Ok(())
-            })
-            .map_err(LedgerOpeningRefusal::LedgerFileUnusable)?
-            .map_err(|reason| LedgerOpeningRefusal::FreeMemoryUnreadable {
-                file_stem: file_stem.clone(),
-                reason,
-            })?;
-        }
-        Ok(Self {
-            settings,
-            card_file_stems,
+        mebibytes_each_process_keeps_on_a_card: u64,
+    ) -> Self {
+        Self {
+            read_card_memory,
             capacity_for_this_process,
+            mebibytes_each_process_keeps_on_a_card,
+            books: Mutex::new(
+                (0..card_count)
+                    .map(|_| BooksOfACard {
+                        leases: BTreeMap::new(),
+                        driver_share: DriverShareOnTheCard::NotRegistered,
+                    })
+                    .collect(),
+            ),
             next_lease_identifier: AtomicU64::new(0),
-            returns_in_this_process: Mutex::new(0),
-            memory_returned: Condvar::new(),
-        })
+        }
     }
 
-    /// 这张卡此刻看这个进程的容量与已记的（在锁里读）。账本里没有估计（开账之后被人删了）当容量 0：借什么都放不下，调用方判红。
-    fn card_seen_by_this_process(
-        records: &CardLedgerRecords,
-        settings: &LedgerSettings,
-        capacity_for_this_process: &CardCapacityForThisProcess,
-        card_position: usize,
-    ) -> CardSeenByThisProcess {
-        CardSeenByThisProcess {
-            capacity_mebibytes: records
-                .memory_all_registered_processes_may_hold_mebibytes
-                .map_or(0, |estimate| {
-                    capacity_for_this_process(card_position, estimate)
-                }),
-            booked_mebibytes: records.booked_mebibytes_seen_by(
-                settings.this_process,
-                settings.mebibytes_each_process_keeps_on_a_card,
-            ),
-        }
+    /// 账的锁：持锁的那几段只改这几张表、不 panic 到一半（借据号重了才断言，那时账已经坏了），别的线程 panic 留下的毒照用。
+    fn books_locked(&self) -> MutexGuard<'_, Vec<BooksOfACard>> {
+        self.books.lock().unwrap_or_else(PoisonError::into_inner)
     }
 
-    /// 这几张卡此刻各看这个进程的容量与已记的。
-    fn cards_seen_by_this_process(
+    /// 在锁里现读每张卡，算这几张卡此刻各能借给这个进程的（与 `card_positions` 逐张对上）。
+    fn rooms_of_the_cards(
         &self,
+        books: &[BooksOfACard],
         card_positions: &[usize],
-    ) -> Result<Vec<CardSeenByThisProcess>, LedgerFileError> {
-        card_positions
+    ) -> Result<Vec<CardRoomForThisProcess>, BookingRefusal> {
+        let readings = (self.read_card_memory)()
+            .map_err(|reason| BookingRefusal::CardMemoryUnreadable { reason })?;
+        if readings.len() != books.len() {
+            return Err(BookingRefusal::CardMemoryUnreadable {
+                reason: format!(
+                    "读到 {} 张卡的显存，池里是 {} 张",
+                    readings.len(),
+                    books.len()
+                ),
+            });
+        }
+        Ok(card_positions
             .iter()
             .map(|card_position| {
-                with_the_card_ledger_locked(
-                    &self.settings,
-                    &self.card_file_stems[*card_position],
-                    |records| {
-                        Self::card_seen_by_this_process(
-                            records,
-                            &self.settings,
-                            &self.capacity_for_this_process,
-                            *card_position,
-                        )
+                card_room_for_this_process(
+                    readings[*card_position],
+                    books[*card_position].booked(),
+                    self.mebibytes_each_process_keeps_on_a_card,
+                    &|may_hold_mebibytes| {
+                        (self.capacity_for_this_process)(*card_position, may_hold_mebibytes)
                     },
                 )
             })
-            .collect()
+            .collect())
+    }
+
+    /// 在锁里记一笔借据，交回记上的那一笔。
+    fn book_in_the_lock(
+        &self,
+        books: &mut [BooksOfACard],
+        card_position: usize,
+        mebibytes: u64,
+        states_booked: Option<u64>,
+        room_mebibytes_when_booked: u64,
+    ) -> BookedOnACard {
+        let lease_identifier = self.next_lease_identifier.fetch_add(1, Ordering::Relaxed);
+        books[card_position].book_a_lease(lease_identifier, mebibytes);
+        BookedOnACard {
+            card_position,
+            lease_identifier,
+            mebibytes_booked: mebibytes,
+            states_booked,
+            room_mebibytes_when_booked,
+        }
     }
 
-    /// 这个进程此刻看每张卡的容量（MiB），次序就是优先级；照账本里此刻的估计算。
+    /// 这几张卡此刻各能借给这个进程多少（MiB，现读一次，与 `card_positions` 逐张对上）。
     ///
     /// # Errors
-    /// 账本文件读写不了、认不出。
-    pub fn card_capacities_mebibytes(&self) -> Result<Vec<u64>, LedgerFileError> {
-        let every_card: Vec<usize> = (0..self.card_file_stems.len()).collect();
+    /// 卡上的显存读不出（[`BookingRefusal::CardMemoryUnreadable`]）。
+    pub fn card_rooms_mebibytes_now(
+        &self,
+        card_positions: &[usize],
+    ) -> Result<Vec<u64>, BookingRefusal> {
+        let books = self.books_locked();
         Ok(self
-            .cards_seen_by_this_process(&every_card)?
+            .rooms_of_the_cards(&books, card_positions)?
             .iter()
-            .map(|card| card.capacity_mebibytes)
+            .map(|room| room.room_now_mebibytes)
             .collect())
     }
 
-    /// 这个进程在这张卡上登记了没有（第一次在它上面借、或 [`Self::register_this_process_on_the_card_waiting_for_room`] 之后就登记了）。
+    /// 借 `need_mebibytes`：照卡的优先级记在第一张此刻放得下的卡上（这个进程在那张卡上还没登记的，它的驱动那一份也要放得下，
+    /// 一并登记），交回借据（丢掉时销账）；此刻每张都放不下就交借不到、什么都不记，不等。
     ///
     /// # Errors
-    /// 账本文件读写不了、认不出。
-    pub fn is_this_process_registered_on_the_card(
+    /// 见 [`BookingRefusal`]。
+    ///
+    /// # Panics
+    /// `need_mebibytes` 是 0（借 0 MiB 照样要登记驱动那一份，这一笔说不清要什么）。
+    pub fn book_on_the_first_card_with_room(
         &self,
-        card_position: usize,
-    ) -> Result<bool, LedgerFileError> {
-        with_the_card_ledger_locked(
-            &self.settings,
-            &self.card_file_stems[card_position],
-            |records| {
-                records
-                    .registered_processes
-                    .contains(&self.settings.this_process)
-            },
-        )
+        need_mebibytes: u64,
+    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
+        let every_card: Vec<usize> = (0..self.books_locked().len()).collect();
+        self.book_on_one_of_these_cards(&every_card, need_mebibytes)
     }
 
-    /// 锁住这张卡的账本重读一遍，此刻还放得下就记上（这个进程在这张卡上还没登记的一并登记），交回记上的那一笔；
-    /// 读完到锁上之间别的进程记了一笔、放不下了，交 [`CardBookingNow::NoRoomNow`]；要的比此刻的容量大，交
-    /// [`CardBookingNow::NeverFits`]。按状态数借的照锁里此刻的估计压（[`WhatIsBookedOnOneCard::need_within_the_capacity`]）。
-    fn book_on_the_card_if_it_still_fits(
+    /// 借 `need_mebibytes`，只记在第 `card_position` 张卡上（卡由调用方定、不由账挑）；别的同 [`Self::book_on_the_first_card_with_room`]。
+    ///
+    /// # Errors
+    /// 见 [`BookingRefusal`]。
+    ///
+    /// # Panics
+    /// 同 [`Self::book_on_the_first_card_with_room`]。
+    pub fn book_on_the_card(
         &self,
         card_position: usize,
-        what: WhatIsBookedOnOneCard<'_>,
-    ) -> Result<CardBookingNow, LedgerFileError> {
-        with_the_card_ledger_locked(
-            &self.settings,
-            &self.card_file_stems[card_position],
-            |records| {
-                let seen = Self::card_seen_by_this_process(
-                    records,
-                    &self.settings,
-                    &self.capacity_for_this_process,
-                    card_position,
-                );
-                let (need_mebibytes, states_booked) =
-                    what.need_within_the_capacity(seen.capacity_mebibytes);
-                match decide_booking_by_priority(
-                    &[seen.capacity_mebibytes],
-                    &[seen.booked_mebibytes],
-                    need_mebibytes,
-                ) {
-                    BookingDecision::BookOnTheCardAt { .. } => {
-                        let lease_identifier = match what {
-                            WhatIsBookedOnOneCard::AsGiven(WhatIsBooked::LeaseOf { .. })
-                            | WhatIsBookedOnOneCard::StatesCompressedToTheCapacity { .. } => {
-                                let lease_identifier =
-                                    self.next_lease_identifier.fetch_add(1, Ordering::Relaxed);
-                                records.leases.push(RecordedLease {
-                                    process: self.settings.this_process,
-                                    lease_identifier,
-                                    mebibytes: need_mebibytes,
-                                });
-                                Some(lease_identifier)
-                            }
-                            WhatIsBookedOnOneCard::AsGiven(WhatIsBooked::DriverShareOnly) => None,
-                        };
-                        if !records
-                            .registered_processes
-                            .contains(&self.settings.this_process)
-                        {
-                            records
-                                .registered_processes
-                                .push(self.settings.this_process);
-                        }
-                        CardBookingNow::Booked(BookedOnACard {
-                            card_position,
-                            lease_identifier,
-                            mebibytes_booked: need_mebibytes,
-                            states_booked,
-                            capacity_mebibytes: seen.capacity_mebibytes,
-                        })
-                    }
-                    BookingDecision::WaitForMemoryToBeReturned => {
-                        CardBookingNow::NoRoomNow { need_mebibytes }
-                    }
-                    BookingDecision::NeverFits {
-                        largest_capacity_mebibytes,
-                    } => CardBookingNow::NeverFits {
-                        capacity_mebibytes: largest_capacity_mebibytes,
-                    },
-                }
-            },
-        )
-    }
-
-    /// 计数器上只做加一，持锁的那一段不会 panic 到一半：别的线程 panic 留下的毒不影响它，照用。
-    fn returns_seen_in_this_process(&self) -> u64 {
-        *self
-            .returns_in_this_process
-            .lock()
-            .unwrap_or_else(PoisonError::into_inner)
-    }
-
-    /// 睡到这个进程里有一笔还回来（还的当场叫醒）、或到重读账本的间隔（别的进程还的叫不醒这里），不越过最长的等待；
-    /// `returns_seen` 是判之前看到的还过几次：判完到睡下之间有人还了就不睡。
-    fn sleep_until_a_return_or_the_poll_interval(&self, returns_seen: u64, waited: Duration) {
-        let returns = self
-            .returns_in_this_process
-            .lock()
-            .unwrap_or_else(PoisonError::into_inner);
-        if *returns == returns_seen {
-            let sleep = self
-                .settings
-                .poll_interval_for_memory_returned_by_other_processes
-                .min(self.settings.longest_wait.saturating_sub(waited));
-            drop(
-                self.memory_returned
-                    .wait_timeout(returns, sleep)
-                    .unwrap_or_else(PoisonError::into_inner),
-            );
-        }
-    }
-
-    /// 几张卡一起借的那几条路等一轮：从 `started` 起已经等满最长的等待交 `Err(等了多久)`，没等满就睡一下
-    /// （[`Self::sleep_until_a_return_or_the_poll_interval`]）。
-    fn wait_once_more_unless_the_longest_wait_is_over(
-        &self,
-        started: Instant,
-        returns_seen: u64,
-    ) -> Result<(), Duration> {
-        let waited_so_far = started.elapsed();
-        if self.settings.longest_wait <= waited_so_far {
-            return Err(waited_so_far);
-        }
-        self.sleep_until_a_return_or_the_poll_interval(returns_seen, waited_so_far);
-        Ok(())
+        need_mebibytes: u64,
+    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
+        self.book_on_one_of_these_cards(&[card_position], need_mebibytes)
     }
 
-    /// 等满了还借不到：带上这几张卡此刻看这个进程的容量。
-    fn refusal_after_the_longest_wait(
+    /// 照 `card_positions` 的先后挑第一张此刻放得下的卡记 `need_mebibytes`（在一次锁里读、判、记）。
+    fn book_on_one_of_these_cards(
         &self,
         card_positions: &[usize],
         need_mebibytes: u64,
-        waited: Duration,
-    ) -> BookingRefusal {
-        match self.cards_seen_by_this_process(card_positions) {
-            Ok(cards) => BookingRefusal::WaitedPastTheLongestWait {
+    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
+        assert!(
+            need_mebibytes > 0,
+            "借 0 MiB 照样要登记驱动那一份：调用方按实际要建的缓冲借，至少 1 MiB"
+        );
+        let mut books = self.books_locked();
+        let rooms = self.rooms_of_the_cards(&books, card_positions)?;
+        match decide_booking_by_priority(&rooms, need_mebibytes) {
+            BookingDecision::BookOnTheCardAt { card_position } => {
+                let booked = self.book_in_the_lock(
+                    &mut books,
+                    card_positions[card_position],
+                    need_mebibytes,
+                    None,
+                    rooms[card_position].room_now_mebibytes,
+                );
+                drop(books);
+                Ok(self.booking_of(booked))
+            }
+            BookingDecision::NoRoomNow => Err(BookingRefusal::NoCardHasRoomNow {
                 need_mebibytes,
-                waited,
-                card_capacities_mebibytes: cards
-                    .iter()
-                    .map(|card| card.capacity_mebibytes)
-                    .collect(),
-            },
-            Err(error) => BookingRefusal::LedgerFileUnusable(error),
-        }
-    }
-
-    /// 记上的一笔借据做成借据（丢掉时销账）。
-    ///
-    /// # Panics
-    /// 记的不是借据（只登记驱动那一份的没有借据号）。
-    fn booking_of(&self, booked: BookedOnACard) -> GpuMemoryBooking<'_> {
-        GpuMemoryBooking {
-            ledger: self,
-            card_position: booked.card_position,
-            mebibytes: booked.mebibytes_booked,
-            lease_identifier: booked
-                .lease_identifier
-                .expect("记的是一笔借据（借一个量或按状态数借），记上就有借据号"),
-            card_capacity_mebibytes_when_booked: booked.capacity_mebibytes,
+                card_rooms_mebibytes: rooms.iter().map(|room| room.room_now_mebibytes).collect(),
+            }),
+            BookingDecision::NeverFits {
+                largest_ceiling_mebibytes,
+            } => Err(BookingRefusal::NeedLargerThanEveryCard(
+                NeedLargerThanEveryCard {
+                    need_mebibytes,
+                    largest_ceiling_mebibytes,
+                },
+            )),
         }
     }
 
-    /// 把这个进程登记到 `card_positions`（次序就是优先级）里此刻放得下它的驱动那一份的每张卡上（已经登记了的照算），每张当场试一次、不等；
-    /// 一张都登记不上才等别的借据还，等满放弃（第 10w 批：主 agent 定几张卡一张借不到时不拿着别的卡干等）。
-    /// 交回每张卡登记了没有（与 `card_positions` 逐张对上，至少一张登记了）；调用方只在登记了的卡上起设备。
+    /// 崩溃放量产品路径第 ① 段借草稿区：按卡的先后（`card_positions`）把 `pending_states` 个状态装到此刻借得到的卡上
+    /// （[`place_states_on_the_cards_that_book_now`]，每张卡最多装 `states_each_card_holds` 个，与 `card_positions` 逐张对上），
+    /// 在一次锁里现读每张卡，每张照此刻能借给这个进程的压状态数再借（几个状态要多少显存照 `mebibytes_for_states`，随状态数单调不减、
+    /// 一个状态至少 1 MiB）；借不到的卡不装。交回每张卡借到的（与 `card_positions` 逐张对上，没借到的是 `None`，至少一张借到了）。
     ///
     /// # Errors
-    /// 一张都登记不上而等满了、账本读写不了（见 [`BookingRefusal`]）。
+    /// 一张都没借到而空着时放得下（[`BookingRefusal::NoCardHasRoomNow`]，要的是一个状态的量）；每张卡空着时连一个状态都放不下
+    /// （[`BookingRefusal::NeedLargerThanEveryCard`]）；卡上的显存读不出。出错时什么都没记。
     ///
     /// # Panics
-    /// `card_positions` 是空的。
-    pub fn register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(
+    /// 两张表的卡数不同；同一张卡给了两次；一个状态要 0 MiB。
+    pub fn book_states_on_the_cards_with_room(
         &self,
         card_positions: &[usize],
-    ) -> Result<Vec<bool>, BookingRefusal> {
-        assert!(
-            !card_positions.is_empty(),
-            "至少给一张卡：一张卡都没有就只能等满"
+        pending_states: u64,
+        states_each_card_holds: &[u64],
+        mebibytes_for_states: &dyn Fn(u64) -> u64,
+    ) -> Result<Vec<Option<StatesBookedOnACard<'_>>>, BookingRefusal> {
+        assert_eq!(
+            card_positions.len(),
+            states_each_card_holds.len(),
+            "卡表与每张卡带得了几个状态的表要逐张对上"
         );
-        let started = Instant::now();
-        loop {
-            let returns_seen = self.returns_seen_in_this_process();
-            let registered = card_positions
+        assert!(
+            card_positions
                 .iter()
-                .map(|card_position| {
-                    if self.is_this_process_registered_on_the_card(*card_position)? {
-                        return Ok(true);
-                    }
-                    match self.book_on_the_card_if_it_still_fits(
-                        *card_position,
-                        WhatIsBookedOnOneCard::AsGiven(WhatIsBooked::DriverShareOnly),
-                    )? {
-                        CardBookingNow::Booked(_) => Ok(true),
-                        CardBookingNow::NoRoomNow { .. } | CardBookingNow::NeverFits { .. } => {
-                            Ok(false)
-                        }
-                    }
-                })
-                .collect::<Result<Vec<bool>, LedgerFileError>>()
-                .map_err(BookingRefusal::LedgerFileUnusable)?;
-            if registered.contains(&true) {
-                return Ok(registered);
-            }
-            if let Err(waited) =
-                self.wait_once_more_unless_the_longest_wait_is_over(started, returns_seen)
-            {
-                return Err(self.refusal_after_the_longest_wait(
-                    card_positions,
-                    self.settings.mebibytes_each_process_keeps_on_a_card,
-                    waited,
-                ));
-            }
-        }
-    }
-
-    /// 崩溃放量产品路径第 ① 段借草稿区：按卡的先后（`card_positions`）把 `pending_states` 个状态装到此刻借得到的卡上
-    /// （[`place_states_on_the_cards_that_book_now`]，每张卡最多装 `states_each_card_holds` 个，与 `card_positions` 逐张对上），
-    /// 每张卡在它的锁里照此刻的估计压状态数再借（[`WhatIsBookedOnOneCard::StatesCompressedToTheCapacity`]，几个状态要多少显存照
-    /// `mebibytes_for_states`）；借不到的卡不装、不等。一张都没借到才等别的借据还，等满放弃。
-    /// 交回每张卡借到的（与 `card_positions` 逐张对上，没借到的是 `None`，至少一张借到了）。
-    ///
-    /// # Errors
-    /// 一张都没借到而等满了；每张卡的账连一个状态都放不下（[`BookingRefusal::NeedLargerThanEveryCard`]，要的是一个状态的量）；
-    /// 账本读写不了。出错时已经借到的都还了。
-    ///
-    /// # Panics
-    /// 两张表的卡数不同。
-    pub fn book_states_on_the_cards_waiting_only_when_none_has_room(
-        &self,
-        card_positions: &[usize],
-        pending_states: u64,
-        states_each_card_holds: &[u64],
-        mebibytes_for_states: &dyn Fn(u64) -> u64,
-    ) -> Result<Vec<Option<StatesBookedOnACard<'_>>>, BookingRefusal> {
-        assert_eq!(
-            card_positions.len(),
-            states_each_card_holds.len(),
-            "卡表与每张卡带得了几个状态的表要逐张对上"
+                .enumerate()
+                .all(|(slot, position)| !card_positions[..slot].contains(position)),
+            "同一张卡给了两次：{card_positions:?}"
         );
-        let started = Instant::now();
-        loop {
-            let returns_seen = self.returns_seen_in_this_process();
-            let mut bookings: Vec<Option<StatesBookedOnACard<'_>>> =
-                card_positions.iter().map(|_| None).collect();
-            let mut ledger_error = None;
-            let attempts = place_states_on_the_cards_that_book_now(
-                pending_states,
-                states_each_card_holds,
-                &mut |card_slot, states_to_place| {
-                    if ledger_error.is_some() {
-                        // 这一轮已经作废（下面整轮交错误、借到的都还），后面的卡不再去借
-                        return ScratchAttemptOnACard::NothingToPlace;
-                    }
-                    let booked_now = self.book_on_the_card_if_it_still_fits(
+        let need_of_one_state_mebibytes = mebibytes_for_states(1);
+        assert!(
+            need_of_one_state_mebibytes > 0,
+            "一个状态要 0 MiB：判器按状态数借的那一族至少有表与一个状态的草稿区"
+        );
+        let mut books = self.books_locked();
+        let rooms = self.rooms_of_the_cards(&books, card_positions)?;
+        let mut booked_on_each_card: Vec<Option<BookedOnACard>> =
+            card_positions.iter().map(|_| None).collect();
+        let attempts = place_states_on_the_cards_that_book_now(
+            pending_states,
+            states_each_card_holds,
+            &mut |card_slot, states_to_place| {
+                let room = rooms[card_slot];
+                let states_that_fit = most_states_whose_need_fits_the_capacity(
+                    usize::try_from(states_to_place).unwrap_or(usize::MAX),
+                    room.room_now_mebibytes,
+                    &|states| {
+                        mebibytes_for_states(u64::try_from(states).expect("状态数装得进 u64"))
+                    },
+                );
+                let states =
+                    u64::try_from(states_that_fit).expect("压出来的不多于要装的，装得进 u64");
+                if states > 0 {
+                    booked_on_each_card[card_slot] = Some(self.book_in_the_lock(
+                        &mut books,
                         card_positions[card_slot],
-                        WhatIsBookedOnOneCard::StatesCompressedToTheCapacity {
-                            want_states: states_to_place,
-                            mebibytes_for_states,
-                        },
-                    );
-                    match booked_now {
-                        Ok(CardBookingNow::Booked(booked)) => {
-                            let states = booked
-                                .states_booked
-                                .expect("按状态数借的那一笔记上就带着借了几个状态");
-                            bookings[card_slot] = Some(StatesBookedOnACard {
-                                states,
-                                booking: self.booking_of(booked),
-                            });
-                            ScratchAttemptOnACard::Booked { states }
-                        }
-                        Ok(CardBookingNow::NoRoomNow { need_mebibytes }) => {
-                            ScratchAttemptOnACard::NoRoomNow { need_mebibytes }
-                        }
-                        Ok(CardBookingNow::NeverFits { capacity_mebibytes }) => {
-                            ScratchAttemptOnACard::CannotHoldOneState { capacity_mebibytes }
-                        }
-                        Err(error) => {
-                            ledger_error = Some(error);
-                            ScratchAttemptOnACard::NothingToPlace
-                        }
-                    }
-                },
-            );
-            if let Some(error) = ledger_error {
-                return Err(BookingRefusal::LedgerFileUnusable(error));
-            }
-            match decide_after_attempting_every_card(&attempts) {
-                ScratchPlacementDecision::JudgeOnTheCardsBooked => return Ok(bookings),
-                ScratchPlacementDecision::NoCardHoldsOneState {
-                    largest_capacity_mebibytes,
-                } => {
-                    return Err(BookingRefusal::NeedLargerThanEveryCard(
-                        NeedLargerThanEveryCard {
-                            need_mebibytes: mebibytes_for_states(1),
-                            largest_capacity_mebibytes,
-                        },
+                        mebibytes_for_states(states),
+                        Some(states),
+                        room.room_now_mebibytes,
                     ));
+                    return ScratchAttemptOnACard::Booked { states };
                 }
-                ScratchPlacementDecision::WaitForRoom { need_mebibytes } => {
-                    if let Err(waited) =
-                        self.wait_once_more_unless_the_longest_wait_is_over(started, returns_seen)
-                    {
-                        return Err(self.refusal_after_the_longest_wait(
-                            card_positions,
-                            need_mebibytes,
-                            waited,
-                        ));
-                    }
-                }
-            }
-        }
-    }
-
-    /// 照 `which` 挑卡记 `what`：记得上就交回记上的那一笔；此刻放不下就等别的借据还（这个进程里还的当场叫醒，
-    /// 别的进程还的隔 [`LedgerSettings::poll_interval_for_memory_returned_by_other_processes`] 重读一次看到），
-    /// 等满 [`LedgerSettings::longest_wait`] 还放不下就放弃；永远放不下交错误、什么都不记。
-    fn book_waiting(
-        &self,
-        which: WhichCards,
-        what: WhatIsBooked,
-    ) -> Result<BookedOnACard, BookingRefusal> {
-        let need_mebibytes = match what {
-            WhatIsBooked::LeaseOf { need_mebibytes } => need_mebibytes,
-            WhatIsBooked::DriverShareOnly => 0,
-        };
-        let card_positions: Vec<usize> = match which {
-            WhichCards::FirstThatFitsByPriority => (0..self.card_file_stems.len()).collect(),
-            WhichCards::OnlyTheCardAt { card_position } => vec![card_position],
-        };
-        let started = Instant::now();
-        loop {
-            let returns_seen = self.returns_seen_in_this_process();
-            let cards = self
-                .cards_seen_by_this_process(&card_positions)
-                .map_err(BookingRefusal::LedgerFileUnusable)?;
-            let capacities: Vec<u64> = cards.iter().map(|card| card.capacity_mebibytes).collect();
-            let booked: Vec<u64> = cards.iter().map(|card| card.booked_mebibytes).collect();
-            match decide_booking_by_priority(&capacities, &booked, need_mebibytes) {
-                BookingDecision::BookOnTheCardAt { card_position } => {
-                    match self
-                        .book_on_the_card_if_it_still_fits(
-                            card_positions[card_position],
-                            WhatIsBookedOnOneCard::AsGiven(what),
-                        )
-                        .map_err(BookingRefusal::LedgerFileUnusable)?
-                    {
-                        CardBookingNow::Booked(booked_on_the_card) => {
-                            return Ok(booked_on_the_card);
-                        }
-                        // 读完到锁上之间别的进程记了一笔、或开账把估计改小了：照此刻的账重判
-                        CardBookingNow::NoRoomNow { .. } | CardBookingNow::NeverFits { .. } => {}
-                    }
-                }
-                BookingDecision::WaitForMemoryToBeReturned => {
-                    let waited = started.elapsed();
-                    if waited >= self.settings.longest_wait {
-                        return Err(BookingRefusal::WaitedPastTheLongestWait {
-                            need_mebibytes,
-                            waited,
-                            card_capacities_mebibytes: capacities,
-                        });
-                    }
-                    self.sleep_until_a_return_or_the_poll_interval(returns_seen, waited);
+                if need_of_one_state_mebibytes > room.ceiling_mebibytes {
+                    return ScratchAttemptOnACard::CannotHoldOneState {
+                        ceiling_mebibytes: room.ceiling_mebibytes,
+                    };
                 }
-                BookingDecision::NeverFits {
-                    largest_capacity_mebibytes,
-                } => {
-                    return Err(BookingRefusal::NeedLargerThanEveryCard(
-                        NeedLargerThanEveryCard {
-                            need_mebibytes,
-                            largest_capacity_mebibytes,
-                        },
-                    ))
+                ScratchAttemptOnACard::NoRoomNow {
+                    need_mebibytes: need_of_one_state_mebibytes,
                 }
+            },
+        );
+        drop(books);
+        match decide_after_attempting_every_card(&attempts) {
+            ScratchPlacementDecision::JudgeOnTheCardsBooked => Ok(booked_on_each_card
+                .into_iter()
+                .map(|booked| {
+                    booked.map(|booked| StatesBookedOnACard {
+                        states: booked
+                            .states_booked
+                            .expect("按状态数借的那一笔记上就带着借了几个状态"),
+                        booking: self.booking_of(booked),
+                    })
+                })
+                .collect()),
+            ScratchPlacementDecision::NoRoomOnAnyCardNow { need_mebibytes } => {
+                Err(BookingRefusal::NoCardHasRoomNow {
+                    need_mebibytes,
+                    card_rooms_mebibytes: rooms
+                        .iter()
+                        .map(|room| room.room_now_mebibytes)
+                        .collect(),
+                })
             }
+            ScratchPlacementDecision::NoCardHoldsOneState {
+                largest_ceiling_mebibytes,
+            } => Err(BookingRefusal::NeedLargerThanEveryCard(
+                NeedLargerThanEveryCard {
+                    need_mebibytes: need_of_one_state_mebibytes,
+                    largest_ceiling_mebibytes,
+                },
+            )),
         }
     }
 
-    /// 借 `need_mebibytes`：照卡的优先级记在第一张放得下的卡上（这个进程在那张卡上还没登记的，它的驱动那一份也要放得下，一并登记），
-    /// 记得上就交回借据（丢掉时销账）；放不下就等，等满放弃；永远放不下交错误、什么都不记（同 [`Self::book_waiting`]）。
-    ///
-    /// # Errors
-    /// 见 [`BookingRefusal`]。
-    pub fn book_waiting_for_room(
-        &self,
-        need_mebibytes: u64,
-    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
-        self.book_a_lease_waiting(WhichCards::FirstThatFitsByPriority, need_mebibytes)
+    /// 记上的一笔借据做成借据（丢掉时销账）。
+    fn booking_of(&self, booked: BookedOnACard) -> GpuMemoryBooking<'_> {
+        GpuMemoryBooking {
+            ledger: self,
+            card_position: booked.card_position,
+            mebibytes: booked.mebibytes_booked,
+            lease_identifier: booked.lease_identifier,
+            card_room_mebibytes_when_booked: booked.room_mebibytes_when_booked,
+        }
     }
 
-    /// 照 `which` 挑卡借 `need_mebibytes`（[`Self::book_waiting`]），记上的那一笔做成借据。
-    fn book_a_lease_waiting(
-        &self,
-        which: WhichCards,
-        need_mebibytes: u64,
-    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
-        let booked = self.book_waiting(which, WhatIsBooked::LeaseOf { need_mebibytes })?;
-        Ok(self.booking_of(booked))
+    /// 这个进程在这张卡上登记了没有（在它上面借过一笔就登记了）。
+    #[must_use]
+    pub fn is_this_process_registered_on_the_card(&self, card_position: usize) -> bool {
+        match self.books_locked()[card_position].driver_share {
+            DriverShareOnTheCard::NotRegistered => false,
+            DriverShareOnTheCard::RegisteredDeviceNotStarted
+            | DriverShareOnTheCard::DeviceStarted => true,
+        }
     }
 
-    /// 借 `need_mebibytes`，只记在第 `card_position` 张卡上（崩溃放量产品路径按每张卡装的状态借草稿区，卡由派活定、不由账挑）；
-    /// 别的同 [`Self::book_waiting_for_room`]：这个进程在这张卡上还没登记的，驱动那一份一并登记；放不下就等，等满放弃；
-    /// 永远放不下（要的比这张卡的容量大）交错误、什么都不记。
+    /// 在第 `card_position` 张卡上起设备之前调（先登记、后建设备）：这个进程要已经在这张卡上借过一笔。
     ///
-    /// # Errors
-    /// 见 [`BookingRefusal`]。
-    pub fn book_on_the_card_waiting_for_room(
-        &self,
-        card_position: usize,
-        need_mebibytes: u64,
-    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
-        self.book_a_lease_waiting(WhichCards::OnlyTheCardAt { card_position }, need_mebibytes)
+    /// # Panics
+    /// 这个进程还没在这张卡上登记（起设备的次序被打乱了）。
+    pub fn check_registered_before_starting_the_device(&self, card_position: usize) {
+        assert!(
+            self.is_this_process_registered_on_the_card(card_position),
+            "第 {card_position} 张卡：这个进程在它的账上登记之前就要起设备（先登记、后建设备）"
+        );
     }
 
-    /// 把这个进程登记到第 `card_position` 张卡上（记它的驱动那一份），不借：已经登记了的当场交回；没登记的要账上放得下这一份才登记，
-    /// 放不下就等、等满放弃（同 [`Self::book_waiting`]）。调用方拿到 `Ok` 之后才在这张卡上起设备。
+    /// 第 `card_position` 张卡上的设备起好了：从此这个进程的驱动那一份算实际占在卡上（读不到实际占着的时候照它算）。
     ///
-    /// # Errors
-    /// 见 [`BookingRefusal`]。
-    pub fn register_this_process_on_the_card_waiting_for_room(
-        &self,
-        card_position: usize,
-    ) -> Result<(), BookingRefusal> {
-        if self
-            .is_this_process_registered_on_the_card(card_position)
-            .map_err(BookingRefusal::LedgerFileUnusable)?
-        {
-            return Ok(());
-        }
-        self.book_waiting(
-            WhichCards::OnlyTheCardAt { card_position },
-            WhatIsBooked::DriverShareOnly,
-        )
-        .map(|_registered| ())
+    /// # Panics
+    /// 同 [`Self::check_registered_before_starting_the_device`]。
+    pub fn record_the_device_started_on_the_card(&self, card_position: usize) {
+        self.check_registered_before_starting_the_device(card_position);
+        self.books_locked()[card_position].driver_share = DriverShareOnTheCard::DeviceStarted;
     }
 
-    /// 从这张卡的账本上删掉这个进程的这一笔。
-    fn remove_the_lease(
-        &self,
-        card_position: usize,
-        lease_identifier: u64,
-    ) -> Result<(), LedgerFileError> {
-        with_the_card_ledger_locked(
-            &self.settings,
-            &self.card_file_stems[card_position],
-            |records| {
-                records.leases.retain(|lease| {
-                    lease.process != self.settings.this_process
-                        || lease.lease_identifier != lease_identifier
-                });
-            },
-        )
+    /// 借据号是 `lease_identifier` 的那一笔此刻还记在第 `card_position` 张卡上没有（还了就不在了）。
+    #[must_use]
+    pub fn is_the_lease_on_the_books(&self, card_position: usize, lease_identifier: u64) -> bool {
+        self.books_locked()[card_position]
+            .leases
+            .contains_key(&lease_identifier)
     }
 }
 
-/// 按状态数在一张卡上借到的草稿区：借了几个状态（在锁里照此刻的估计压过）与那一笔借据（丢掉时还）。
+/// 按状态数在一张卡上借到的草稿区：借了几个状态（在锁里照此刻能借的压过）与那一笔借据（丢掉时还）。
 pub struct StatesBookedOnACard<'ledger> {
     pub states: u64,
     pub booking: GpuMemoryBooking<'ledger>,
 }
 
-/// 账上的一笔：记在哪张卡、多少 MiB、记的那一刻这张卡看这个进程的容量；丢掉时销账、叫醒这个进程里等着借的。
+/// 账上的一笔：记在哪张卡、多少 MiB、记的那一刻这张卡能借给这个进程的；丢掉时销账。
 pub struct GpuMemoryBooking<'ledger> {
     ledger: &'ledger GpuMemoryLedger,
     card_position: usize,
     mebibytes: u64,
     lease_identifier: u64,
-    card_capacity_mebibytes_when_booked: u64,
+    card_room_mebibytes_when_booked: u64,
 }
 
 impl GpuMemoryBooking<'_> {
-    /// 记在账里第几张卡上。
+    /// 记在池里第几张卡上。
     #[must_use]
     pub fn card_position(&self) -> usize {
         self.card_position
@@ -1458,509 +903,591 @@ impl GpuMemoryBooking<'_> {
         self.mebibytes
     }
 
-    /// 记的那一刻这张卡看这个进程的容量（MiB）。
+    /// 这一笔的借据号（[`GpuMemoryLedger::is_the_lease_on_the_books`] 照它查）。
     #[must_use]
-    pub fn card_capacity_mebibytes_when_booked(&self) -> u64 {
-        self.card_capacity_mebibytes_when_booked
+    pub fn lease_identifier(&self) -> u64 {
+        self.lease_identifier
+    }
+
+    /// 记的那一刻这张卡能借给这个进程的（MiB，这一笔之前）。
+    #[must_use]
+    pub fn card_room_mebibytes_when_booked(&self) -> u64 {
+        self.card_room_mebibytes_when_booked
+    }
+
+    /// 这一笔此刻还记在账上没有（[`GpuMemoryLedger::is_the_lease_on_the_books`]；借据活着时恒为真，核账用）。
+    #[must_use]
+    pub fn is_on_the_books(&self) -> bool {
+        self.ledger
+            .is_the_lease_on_the_books(self.card_position, self.lease_identifier)
     }
 }
 
 impl Drop for GpuMemoryBooking<'_> {
     fn drop(&mut self) {
-        // 销不了账只打一行：这一笔留在账本上，等这个进程退出、判活判成不在了时回收
-        if let Err(error) = self
-            .ledger
-            .remove_the_lease(self.card_position, self.lease_identifier)
-        {
-            eprintln!(
-                "GPU_MEMORY_LEDGER_RETURN_FAILED path={} reason={}",
-                error.path.display(),
-                error.reason
-            );
-        }
-        *self
-            .ledger
-            .returns_in_this_process
-            .lock()
-            .unwrap_or_else(PoisonError::into_inner) += 1;
-        self.ledger.memory_returned.notify_all();
+        let removed = self.ledger.books_locked()[self.card_position]
+            .leases
+            .remove(&self.lease_identifier);
+        // 别处已经在 panic（丢借据是收尾的一部分）时不再断言：两次 panic 会让整个进程中止
+        assert!(
+            removed.is_some() || std::thread::panicking(),
+            "第 {} 张卡上借据号 {} 不在账上：一笔借据只在它丢掉时销一次账",
+            self.card_position,
+            self.lease_identifier
+        );
     }
 }
 
 #[cfg(test)]
 mod tests {
     use super::{
-        card_ledger_capacity_mebibytes, decide_after_attempting_every_card,
-        decide_booking_by_priority, gpu_memory_ledger_directory, mebibytes_of_buffers_held_at_once,
-        memory_all_registered_processes_may_hold_mebibytes,
+        card_ledger_capacity_mebibytes, card_room_for_this_process,
+        decide_after_attempting_every_card, decide_booking_by_priority,
+        mebibytes_of_buffers_held_at_once, memory_this_process_may_hold_mebibytes,
         most_states_whose_need_fits_the_capacity, place_states_on_the_cards_that_book_now,
-        process_is_alive_by_its_entry_in_the_process_filesystem, scratch_booking_refusal_outcome,
-        scratch_states_to_book_on_each_card,
-        start_time_in_clock_ticks_from_the_process_status_line,
-        states_in_one_dispatch_capped_by_the_state_count, BookingDecision, BookingRefusal,
-        GpuCardsSource, GpuMemoryLedger, LedgerSettings, NeedLargerThanEveryCard, RecordedProcess,
+        scratch_booking_refusal_outcome, scratch_states_to_book_on_each_card,
+        states_in_one_dispatch_capped_by_the_state_count, BookedByThisProcessOnACard,
+        BookingDecision, BookingRefusal, CardMemoryReading, CardRoomForThisProcess,
+        DriverShareOnTheCard, GpuCardsSource, GpuMemoryLedger, NeedLargerThanEveryCard,
         ScratchAttemptOnACard, ScratchBookingRefusalOutcome, ScratchPlacementDecision,
-        ScratchStatesToBook, GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME,
+        ScratchStatesToBook,
     };
-    use std::ffi::OsStr;
-    use std::path::{Path, PathBuf};
-    use std::time::{Duration, Instant};
+    use std::sync::{Arc, Mutex};
 
     /// 驱动的固定开销，照卡池用的那个数（`crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER`）。
     const DRIVER: u64 = 2560;
-
-    /// 这条用例自己的账本目录（临时目录下，带用例名与进程号）：先删掉上一趟留下的，用例收尾时删。
-    struct LedgerDirectory {
-        path: PathBuf,
-    }
-
-    impl LedgerDirectory {
-        fn for_the_test(test_name: &str) -> Self {
-            let path = std::env::temp_dir().join(format!(
-                "singlefs-gpu-memory-ledger-{test_name}-{}",
-                std::process::id()
-            ));
-            let _ = std::fs::remove_dir_all(&path);
-            Self { path }
+    /// 一张 5090 的显存总量（`nvidia-smi` 报的 memory.total）。
+    const TOTAL_OF_AN_RTX_5090: u64 = 32_607;
+
+    /// 一张 5090 此刻空着 `free_mebibytes`，这个进程实际占着的读不到。
+    fn reading_of_an_rtx_5090(free_mebibytes: u64) -> CardMemoryReading {
+        CardMemoryReading {
+            free_mebibytes,
+            total_mebibytes: TOTAL_OF_AN_RTX_5090,
+            occupied_by_this_process_mebibytes: None,
         }
     }
 
-    impl Drop for LedgerDirectory {
-        fn drop(&mut self) {
-            let _ = std::fs::remove_dir_all(&self.path);
-        }
-    }
+    /// 换得了的假读数：几张卡此刻的显存，用例中途改它就是卡上的空闲变了。
+    type ReplaceableReadings = Arc<Mutex<Vec<CardMemoryReading>>>;
 
-    /// 一个假的记账进程（进程号就是给的数，启动时刻固定 1）。
-    fn fake_process(process_identifier: u32) -> RecordedProcess {
-        RecordedProcess {
-            process_identifier,
-            start_time_in_clock_ticks: 1,
-        }
+    fn replaceable_readings(readings: Vec<CardMemoryReading>) -> ReplaceableReadings {
+        Arc::new(Mutex::new(readings))
     }
 
-    /// 一个假进程的记账设置：判活只认 `alive` 里的进程号，别的进程各在卡上付 `mebibytes_each_process_keeps_on_a_card`。
-    fn settings_of_a_fake_process(
-        directory: &Path,
-        process_identifier: u32,
-        alive: &'static [u32],
-        mebibytes_each_process_keeps_on_a_card: u64,
-        poll_interval_for_memory_returned_by_other_processes: Duration,
-        longest_wait: Duration,
-    ) -> LedgerSettings {
-        LedgerSettings {
-            directory: directory.to_path_buf(),
-            this_process: fake_process(process_identifier),
-            is_alive: Box::new(move |process: RecordedProcess| {
-                alive.contains(&process.process_identifier)
-            }),
-            mebibytes_each_process_keeps_on_a_card,
-            poll_interval_for_memory_returned_by_other_processes,
-            longest_wait,
-        }
-    }
-
-    /// 一个假进程在 `directory` 下开一张卡（文件名主干 `card-test`）的账，容量不看估计、恒为 `capacity_mebibytes`
-    /// （只测借与等的用例用它；开账读到的空闲随便给一个数）。
-    fn ledger_of_a_fake_process(
-        directory: &Path,
-        process_identifier: u32,
-        alive: &'static [u32],
-        capacity_mebibytes: u64,
-        mebibytes_each_process_keeps_on_a_card: u64,
-        poll_interval_for_memory_returned_by_other_processes: Duration,
-        longest_wait: Duration,
-    ) -> GpuMemoryLedger {
-        GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
-            settings_of_a_fake_process(
-                directory,
-                process_identifier,
-                alive,
-                mebibytes_each_process_keeps_on_a_card,
-                poll_interval_for_memory_returned_by_other_processes,
-                longest_wait,
-            ),
-            vec!["card-test".to_string()],
-            &|_position| Ok(1000),
-            Box::new(move |_position, _estimate| capacity_mebibytes),
-        )
-        .expect("临时目录下的账本读写得了")
-    }
-
-    /// 一个假进程在 `directory` 下开一张卡（文件名主干 `card-test`）的账：开账时读到的空闲是 `free_memory_read_mebibytes`，
-    /// 容量照卡池没配额度、没设额度变量时的算法里不打 3/4 折扣的那一份：账本里的估计减驱动那一份 [`DRIVER`]。
-    fn ledger_reading_the_free_memory(
-        directory: &Path,
-        process_identifier: u32,
-        alive: &'static [u32],
-        free_memory_read_mebibytes: u64,
-        longest_wait: Duration,
-    ) -> GpuMemoryLedger {
-        GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
-            settings_of_a_fake_process(
-                directory,
-                process_identifier,
-                alive,
-                DRIVER,
-                Duration::from_millis(20),
-                longest_wait,
-            ),
-            vec!["card-test".to_string()],
-            &|_position| Ok(free_memory_read_mebibytes),
-            Box::new(|_position, estimate| {
-                card_ledger_capacity_mebibytes(None, None, estimate, estimate, DRIVER)
+    /// 把第 `card_position` 张卡此刻的读数换成 `reading`。
+    fn replace_the_reading(
+        readings: &ReplaceableReadings,
+        card_position: usize,
+        reading: CardMemoryReading,
+    ) {
+        readings.lock().expect("用例里只有一条线程改读数")[card_position] = reading;
+    }
+
+    /// 照假读数记账的一本账：容量照卡池没配额度、没设额度变量时的算法里不打 3/4 折扣的那一份
+    /// （这个进程能占的减驱动那一份 [`DRIVER`]）。
+    fn ledger_reading(readings: &ReplaceableReadings) -> GpuMemoryLedger {
+        let card_count = readings.lock().expect("还没有别的线程").len();
+        let shared = Arc::clone(readings);
+        GpuMemoryLedger::new(
+            card_count,
+            Box::new(move || Ok(shared.lock().expect("用例里只有一条线程改读数").clone())),
+            Box::new(|_position, may_hold_mebibytes| {
+                card_ledger_capacity_mebibytes(
+                    None,
+                    None,
+                    may_hold_mebibytes,
+                    may_hold_mebibytes,
+                    DRIVER,
+                )
             }),
+            DRIVER,
         )
-        .expect("临时目录下的账本读写得了")
     }
 
-    /// 借一笔，等不到就放弃（等的上限是开账时给的）：交回借据或借不到的结局。
-    fn booking_outcome(
+    /// 借一笔：借到了交借的量（借据不还，留在账上），借不到交结局。
+    fn booking_kept_on_the_books(
         ledger: &GpuMemoryLedger,
         need_mebibytes: u64,
     ) -> Result<u64, BookingRefusal> {
-        ledger.book_waiting_for_room(need_mebibytes).map(|booking| {
-            let mebibytes = booking.mebibytes();
-            std::mem::forget(booking);
-            mebibytes
-        })
+        ledger
+            .book_on_the_first_card_with_room(need_mebibytes)
+            .map(|booking| {
+                let mebibytes = booking.mebibytes();
+                std::mem::forget(booking);
+                mebibytes
+            })
     }
 
-    /// 估计是旧估计夹进 [现读的空闲, 现读的空闲 + 活进程已记的]：没有旧估计取现读的；外人占了卡（现读的小）压到上界，
-    /// 外人走了（现读的大）抬到下界，订满了（现读的近 0、已记的近满）照旧。
+    /// 第 10x 批（G1 二）：这个进程借了 1 000 之后，一个不经这份账的 GPU 用户占了 20 000（卡上空闲从 30 000 掉到 9 000）。
+    /// 下一笔照这一刻现读的借：9 000 减这个进程借了还没占的 1 000、减还没起设备的驱动那一份，正好 5 440，多 1 MiB 就借不到。
     #[test]
-    fn the_memory_estimate_is_the_previous_one_clamped_between_the_free_memory_read_now_and_that_plus_the_live_bookings(
-    ) {
+    fn a_booking_after_an_outsider_took_memory_follows_the_free_memory_read_at_that_booking() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+        let ledger = ledger_reading(&readings);
         assert_eq!(
-            memory_all_registered_processes_may_hold_mebibytes(None, 30_000, 0),
-            30_000,
-            "没有旧估计：取现读的空闲"
+            booking_kept_on_the_books(&ledger, 1_000),
+            Ok(1_000),
+            "空着的卡借得到"
         );
+        replace_the_reading(&readings, 0, reading_of_an_rtx_5090(9_000));
         assert_eq!(
-            memory_all_registered_processes_may_hold_mebibytes(Some(30_000), 7_340, 2_660),
-            10_000,
-            "外人后来占了 20 000：压到现读的加已记的"
+            booking_kept_on_the_books(&ledger, 5_441),
+            Err(BookingRefusal::NoCardHasRoomNow {
+                need_mebibytes: 5_441,
+                card_rooms_mebibytes: vec![9_000 - 1_000 - DRIVER],
+            }),
+            "外人占了 20 000：这一笔照现读的 9 000 算，借不到（不判红、不等）"
         );
         assert_eq!(
-            memory_all_registered_processes_may_hold_mebibytes(Some(10_000), 25_440, 4_560),
-            25_440,
-            "外人后来走了：抬到现读的"
+            booking_kept_on_the_books(&ledger, 5_440),
+            Ok(5_440),
+            "现读的空闲减借了还没占的、减驱动那一份，正好借得到"
         );
+    }
+
+    /// 第 10x 批（G1 二）：外人占着卡（空 5 000）时借 10 000 借不到，不判红（卡空着时借得出 30 047）；
+    /// 外人走了（空 30 000），下一笔照现读的借到。
+    #[test]
+    fn a_booking_after_an_outsider_left_gets_the_memory_it_returned() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(5_000)]);
+        let ledger = ledger_reading(&readings);
         assert_eq!(
-            memory_all_registered_processes_may_hold_mebibytes(Some(30_000), 0, 30_000),
-            30_000,
-            "卡被订满：旧估计在区间里，照旧"
+            booking_kept_on_the_books(&ledger, 10_000),
+            Err(BookingRefusal::NoCardHasRoomNow {
+                need_mebibytes: 10_000,
+                card_rooms_mebibytes: vec![5_000 - DRIVER],
+            }),
+            "外人占着：借不到，不当成比卡还大"
+        );
+        replace_the_reading(&readings, 0, reading_of_an_rtx_5090(30_000));
+        assert_eq!(
+            booking_kept_on_the_books(&ledger, 10_000),
+            Ok(10_000),
+            "外人走了：下一笔照现读的借到"
         );
     }
 
-    /// 第 10t 批 K4-1：配置给这张卡 8 G、额度变量 20 000：容量不越过配置的额度（减驱动那一份），额度变量更小时取额度变量；
-    /// 没设额度变量照旧。数照一张清场后空 30 000 MiB 的 5090。
+    /// 第 10x 批（G1 二）：这个进程在同一张卡上先后两笔，卡上读数一直是空 30 000（借的都还没建）：第一笔 20 000，
+    /// 第二笔最多 7 440，两笔加驱动那一份正好是现读的空闲，多 1 MiB 就借不到。
     #[test]
-    fn an_environment_quota_larger_than_the_configured_card_quota_stays_within_the_configured_card_quota(
-    ) {
-        const CONFIGURED_CARD_QUOTA: u64 = 8 * 1024;
-        let budget = CONFIGURED_CARD_QUOTA.min(30_000);
-        let capacity = card_ledger_capacity_mebibytes(
-            Some(20_000),
-            Some(CONFIGURED_CARD_QUOTA),
-            budget,
+    fn two_bookings_of_this_process_on_one_card_stay_within_the_free_memory_read_now() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+        let ledger = ledger_reading(&readings);
+        assert_eq!(booking_kept_on_the_books(&ledger, 20_000), Ok(20_000));
+        assert!(
+            matches!(
+                booking_kept_on_the_books(&ledger, 7_441),
+                Err(BookingRefusal::NoCardHasRoomNow { .. })
+            ),
+            "两笔加驱动那一份超过现读的空闲：借不到"
+        );
+        assert_eq!(booking_kept_on_the_books(&ledger, 7_440), Ok(7_440));
+        assert_eq!(
+            20_000 + 7_440 + DRIVER,
             30_000,
-            DRIVER,
+            "两笔加驱动那一份正好是现读的空闲"
         );
-        assert!(
-            capacity <= CONFIGURED_CARD_QUOTA,
-            "额度变量 20 000 不越过配置给卡的 8 G：实际 {capacity}"
+    }
+
+    /// 第 10x 批：这个进程能占的是现读的空闲加它实际占着的里属于它许出去的那一部分：没登记、登记了没起设备都只算空闲；
+    /// 起了设备而读不到实际占着的，驱动那一份当已经在卡上；读得到的只算到许出去的那么多（分配器多拿的块不算）。
+    #[test]
+    fn the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised() {
+        let free = reading_of_an_rtx_5090(20_000);
+        let booked = |leases_mebibytes: u64, driver_share: DriverShareOnTheCard| {
+            BookedByThisProcessOnACard {
+                leases_mebibytes,
+                driver_share,
+            }
+        };
+        assert_eq!(
+            memory_this_process_may_hold_mebibytes(
+                free,
+                booked(0, DriverShareOnTheCard::NotRegistered),
+                DRIVER
+            ),
+            20_000,
+            "没登记：只有现读的空闲"
         );
         assert_eq!(
-            capacity,
-            CONFIGURED_CARD_QUOTA - DRIVER,
-            "取配置额度减驱动那一份"
+            memory_this_process_may_hold_mebibytes(
+                free,
+                booked(1_000, DriverShareOnTheCard::RegisteredDeviceNotStarted),
+                DRIVER
+            ),
+            20_000,
+            "登记了、设备没起、实际占着的读不到：借的当一点都没建"
         );
         assert_eq!(
-            card_ledger_capacity_mebibytes(
-                Some(4_000),
-                Some(CONFIGURED_CARD_QUOTA),
-                budget,
-                30_000,
+            memory_this_process_may_hold_mebibytes(
+                free,
+                booked(1_000, DriverShareOnTheCard::DeviceStarted),
                 DRIVER
             ),
-            4_000,
-            "额度变量更小：取额度变量"
+            20_000 + DRIVER,
+            "起了设备、实际占着的读不到：驱动那一份当已经在卡上"
+        );
+        let occupied = |mebibytes: u64| CardMemoryReading {
+            occupied_by_this_process_mebibytes: Some(mebibytes),
+            ..free
+        };
+        assert_eq!(
+            memory_this_process_may_hold_mebibytes(
+                occupied(DRIVER + 500),
+                booked(1_000, DriverShareOnTheCard::DeviceStarted),
+                DRIVER
+            ),
+            20_000 + DRIVER + 500,
+            "读得到实际占着的：驱动那一份与借的里建了的 500"
+        );
+        assert_eq!(
+            memory_this_process_may_hold_mebibytes(
+                occupied(9_000),
+                booked(1_000, DriverShareOnTheCard::DeviceStarted),
+                DRIVER
+            ),
+            20_000 + DRIVER + 1_000,
+            "实际占着的比许出去的多：只算到许出去的那么多"
         );
+    }
+
+    /// 第 10x 批 K4-3：额度变量 20 000、配置给卡 24 G 都设了，这个进程此刻能占的只有 10 000：容量照能占的那一项压到
+    /// 10 000 减驱动那一份，不取两个额度里小的那个。
+    #[test]
+    fn the_memory_this_process_may_hold_caps_the_capacity_even_when_both_quotas_are_set() {
+        const CONFIGURED_CARD_QUOTA: u64 = 24 * 1024;
+        const MAY_HOLD: u64 = 10_000;
         assert_eq!(
             card_ledger_capacity_mebibytes(
-                None,
+                Some(20_000),
                 Some(CONFIGURED_CARD_QUOTA),
-                budget,
-                30_000,
+                CONFIGURED_CARD_QUOTA.min(MAY_HOLD),
+                MAY_HOLD,
                 DRIVER
             ),
-            CONFIGURED_CARD_QUOTA - DRIVER,
-            "没设额度变量：照旧是给崩溃放量的额度减驱动那一份"
+            MAY_HOLD - DRIVER,
+            "能占的那一项最小：取它"
         );
     }
 
-    /// 第 10t 批 K4-2：子进程被杀、父进程（跑用例的这个进程）还没收尸时它是僵尸，`/proc/<进程号>/stat` 还在、启动时刻也对得上，
-    /// 判成不在（驱动已经还了它的显存）；收了尸照样不在。
+    /// 第 10x 批（先登记、后建设备，这个进程里）：没在一张卡上借过就起设备要断言；借过之后起得了，起好了驱动那一份算在卡上，
+    /// 读不到实际占着的时候这一张能借的多出驱动那一份。
     #[test]
-    fn a_killed_process_that_its_parent_has_not_reaped_yet_is_judged_not_alive() {
-        let state_of = |process_identifier: u32| -> Option<String> {
-            let status_line =
-                std::fs::read_to_string(format!("/proc/{process_identifier}/stat")).ok()?;
-            let (_, after_the_name) = status_line.rsplit_once(')')?;
-            after_the_name.split_whitespace().next().map(str::to_string)
-        };
-        let mut child = std::process::Command::new("sleep")
-            .arg("600")
-            .spawn()
-            .expect("起得了 sleep");
-        let process_identifier = child.id();
-        let recorded = RecordedProcess {
-            process_identifier,
-            start_time_in_clock_ticks: start_time_in_clock_ticks_from_the_process_status_line(
-                &std::fs::read_to_string(format!("/proc/{process_identifier}/stat"))
-                    .expect("刚起的子进程在 /proc 里"),
-            )
-            .expect("认得出启动时刻"),
-        };
+    fn the_device_starts_only_on_a_card_this_process_booked_and_then_counts_the_driver_share_as_on_the_card(
+    ) {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+        let ledger = ledger_reading(&readings);
         assert!(
-            process_is_alive_by_its_entry_in_the_process_filesystem(recorded),
-            "还在跑的子进程判成还在"
-        );
-        child.kill().expect("杀得了自己起的子进程");
-        let deadline = Instant::now() + Duration::from_secs(30);
-        while state_of(process_identifier).as_deref() != Some("Z") {
-            assert!(
-                Instant::now() < deadline,
-                "被杀的子进程 30 秒内没变成僵尸：状态 {:?}",
-                state_of(process_identifier)
-            );
-            std::thread::sleep(Duration::from_millis(10));
-        }
+            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+                ledger.check_registered_before_starting_the_device(0);
+            }))
+            .is_err(),
+            "没借过就起设备：先登记、后建设备的次序被打乱，要断言"
+        );
+        assert!(!ledger.is_this_process_registered_on_the_card(0));
+        let booking = ledger.book_on_the_card(0, 1_000).expect("空着的卡借得到");
         assert!(
-            !process_is_alive_by_its_entry_in_the_process_filesystem(recorded),
-            "被杀而没收尸的僵尸判成不在"
+            ledger.is_this_process_registered_on_the_card(0),
+            "借过就登记了"
         );
-        child.wait().expect("收得了尸");
-        assert!(
-            !process_is_alive_by_its_entry_in_the_process_filesystem(recorded),
-            "收了尸照样不在"
+        assert_eq!(
+            ledger.card_rooms_mebibytes_now(&[0]),
+            Ok(vec![30_000 - 1_000 - DRIVER]),
+            "设备没起：驱动那一份还要从空闲里扣"
+        );
+        ledger.check_registered_before_starting_the_device(0);
+        ledger.record_the_device_started_on_the_card(0);
+        assert_eq!(
+            ledger.card_rooms_mebibytes_now(&[0]),
+            Ok(vec![30_000 - 1_000]),
+            "设备起好了：驱动那一份已经在卡上，现读的空闲里少了它，不再扣"
         );
+        drop(booking);
     }
 
-    /// 第 10t 批 K4-2：先开账的进程 100 借了 1 000、起了设备，后开账的进程 200 读到的空闲（26 440）里已经少了进程 100 的
-    /// 驱动那一份与它借的：账照样只记一份。进程 200 能借的正好是真空闲减它自己的驱动那一份（23 880），多 1 MiB 就要等。
+    /// 第 10x 批（第 10t 批正推探针 G 那一形）：卡上只空 1 000，不到一份驱动开销：借 1 MiB 也借不到，这个进程不登记（调用方不起设备）。
     #[test]
-    fn a_process_whose_free_memory_reading_already_lacks_the_driver_share_of_a_registered_process_counts_that_share_once(
-    ) {
-        let directory = LedgerDirectory::for_the_test("driver-share-once");
-        let first = ledger_reading_the_free_memory(
-            &directory.path,
-            100,
-            &[100, 200],
-            30_000,
-            Duration::from_secs(60),
-        );
-        let first_booking = booking_outcome(&first, 1_000).expect("空着的卡放得下 1 000");
-        assert_eq!(first_booking, 1_000);
-        let true_free_when_the_second_opens = 30_000 - DRIVER - 1_000;
-        let second = ledger_reading_the_free_memory(
-            &directory.path,
-            200,
-            &[100, 200],
-            true_free_when_the_second_opens,
-            Duration::from_millis(200),
-        );
+    fn a_card_with_less_free_memory_than_one_driver_share_lends_nothing_and_registers_nothing() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(1_000)]);
+        let ledger = ledger_reading(&readings);
         assert_eq!(
-            booking_outcome(&second, true_free_when_the_second_opens - DRIVER),
-            Ok(true_free_when_the_second_opens - DRIVER),
-            "进程 100 的驱动那一份只记一份：进程 200 借得到真空闲减它自己的驱动那一份"
+            booking_kept_on_the_books(&ledger, 1),
+            Err(BookingRefusal::NoCardHasRoomNow {
+                need_mebibytes: 1,
+                card_rooms_mebibytes: vec![0],
+            })
         );
         assert!(
-            matches!(
-                booking_outcome(&second, 1),
-                Err(BookingRefusal::WaitedPastTheLongestWait { .. })
-            ),
-            "再多 1 MiB 就超过了真空闲：等满放弃"
+            !ledger.is_this_process_registered_on_the_card(0),
+            "借不到就不登记"
         );
     }
 
-    /// 第 10t 批 K4-3 撞显存那一向：进程 100 开账时卡上空 30 000、借了 100；之后一个不经账的 GPU 用户占了 20 000；
-    /// 进程 200 开账时读到 7 340。进程 200 能再借的加它自己的驱动那一份不超过真空闲（4 780 + 2 560 = 7 340），
-    /// 进程 100 的下一笔也照改小了的估计借。
+    /// 第 10x 批 K4-1 用的核账：一笔借据丢掉之前一直在账上，丢掉就不在了。
     #[test]
-    fn an_outsider_taking_memory_after_the_first_process_opened_its_ledger_caps_a_later_process_by_the_free_memory_it_reads(
-    ) {
-        let directory = LedgerDirectory::for_the_test("outsider-took-memory");
-        let first = ledger_reading_the_free_memory(
-            &directory.path,
-            100,
-            &[100, 200],
-            30_000,
-            Duration::from_millis(200),
-        );
-        booking_outcome(&first, 100).expect("空着的卡放得下 100");
-        let true_free_when_the_second_opens = 30_000 - DRIVER - 100 - 20_000;
-        let second = ledger_reading_the_free_memory(
-            &directory.path,
-            200,
-            &[100, 200],
-            true_free_when_the_second_opens,
-            Duration::from_millis(200),
-        );
-        let room_of_the_second = true_free_when_the_second_opens - DRIVER;
+    fn a_lease_is_on_the_books_until_it_is_dropped() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+        let ledger = ledger_reading(&readings);
+        let booking = ledger.book_on_the_card(0, 1_000).expect("空着的卡借得到");
+        let lease_identifier = booking.lease_identifier();
+        assert!(ledger.is_the_lease_on_the_books(0, lease_identifier));
+        drop(booking);
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, lease_identifier),
+            "丢掉就销账"
+        );
         assert_eq!(
-            booking_outcome(&second, room_of_the_second),
-            Ok(room_of_the_second),
-            "进程 200 借得到真空闲减它自己的驱动那一份"
+            ledger.card_rooms_mebibytes_now(&[0]),
+            Ok(vec![30_000 - DRIVER]),
+            "销了账，这一笔不再从空闲里扣"
         );
-        assert!(
-            matches!(
-                booking_outcome(&second, 1),
-                Err(BookingRefusal::WaitedPastTheLongestWait { .. })
-            ),
-            "进程 200 再多借 1 MiB 就超过了真空闲：等满放弃"
+    }
+
+    /// 第 10x 批（第 10t 批正推探针 C、攻方 S3 那一形）：卡上空 20 000，这个进程已经借着 12 000（没建）。按状态数借 30 个、
+    /// 每个 1 000 MiB：照此刻能借的 5 440 压成 5 个，当场借到，不按卡的整份容量压成 17 个再判放不下。
+    #[test]
+    fn a_scratch_booking_compresses_its_states_to_the_room_now_and_books_them() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(20_000)]);
+        let ledger = ledger_reading(&readings);
+        assert_eq!(booking_kept_on_the_books(&ledger, 12_000), Ok(12_000));
+        let bookings = ledger
+            .book_states_on_the_cards_with_room(&[0], 30, &[30], &|states| states * 1_000)
+            .expect("此刻放得下 5 个状态");
+        let booked = bookings[0].as_ref().expect("只有这一张卡，借到了");
+        assert_eq!(
+            booked.states, 5,
+            "照此刻能借的 20 000 − 12 000 − 驱动那一份压"
         );
+        assert_eq!(booked.booking.mebibytes(), 5_000);
+    }
+
+    /// 第 10x 批（第 10t 批攻方 S4b、正推探针 D 那一形）：两张卡，第一张这个进程借着 12 000（空 20 000），第二张空 16 000。
+    /// 30 个状态、每个 1 000 MiB、每张带得了 30 个：第一张照此刻能借的装 5 个，剩下的 25 个里第二张装得下 13 个，两张都用上。
+    #[test]
+    fn the_states_a_card_cannot_take_now_go_to_the_next_card_with_room() {
+        let readings = replaceable_readings(vec![
+            reading_of_an_rtx_5090(20_000),
+            reading_of_an_rtx_5090(16_000),
+        ]);
+        let ledger = ledger_reading(&readings);
+        std::mem::forget(ledger.book_on_the_card(0, 12_000).expect("第一张借得到"));
+        let bookings = ledger
+            .book_states_on_the_cards_with_room(&[0, 1], 30, &[30, 30], &|states| states * 1_000)
+            .expect("两张都借得到");
+        let states_booked: Vec<Option<u64>> = bookings
+            .iter()
+            .map(|booked| booked.as_ref().map(|booked| booked.states))
+            .collect();
+        assert_eq!(states_booked, vec![Some(5), Some(13)]);
+    }
+
+    /// 第 10x 批：要的比卡空着时能借的（总量减驱动那一份，30 047）还大，判成比每张卡都大（调用方判红）；不超过它、只是此刻
+    /// 放不下的是借不到（调用方报本次未跑）。按状态数借时一个状态都放不下同理。
+    #[test]
+    fn a_need_larger_than_the_card_when_empty_never_fits_and_a_smaller_one_has_no_room_now() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(5_000)]);
+        let ledger = ledger_reading(&readings);
+        let ceiling = TOTAL_OF_AN_RTX_5090 - DRIVER;
+        assert_eq!(
+            booking_kept_on_the_books(&ledger, ceiling + 1),
+            Err(BookingRefusal::NeedLargerThanEveryCard(
+                NeedLargerThanEveryCard {
+                    need_mebibytes: ceiling + 1,
+                    largest_ceiling_mebibytes: ceiling,
+                }
+            ))
+        );
+        assert!(matches!(
+            booking_kept_on_the_books(&ledger, ceiling),
+            Err(BookingRefusal::NoCardHasRoomNow { .. })
+        ));
+        assert!(matches!(
+            ledger.book_states_on_the_cards_with_room(&[0], 3, &[3], &|states| states
+                * (ceiling + 1)),
+            Err(BookingRefusal::NeedLargerThanEveryCard(_))
+        ));
+        assert!(matches!(
+            ledger.book_states_on_the_cards_with_room(&[0], 3, &[3], &|states| states * 6_000),
+            Err(BookingRefusal::NoCardHasRoomNow {
+                need_mebibytes: 6_000,
+                ..
+            })
+        ));
         assert!(
-            matches!(
-                booking_outcome(&first, 1),
-                Err(BookingRefusal::WaitedPastTheLongestWait { .. })
-            ),
-            "进程 100 的下一笔照改小了的估计借：卡上已经没有空的了"
+            !ledger.is_this_process_registered_on_the_card(0),
+            "一笔都没借到：不登记"
         );
     }
 
-    /// 第 10t 批 K4-3 等满报本次未跑那一向：进程 100 开账时卡被外人占着、读到 10 000，借了 2 000；外人走了，进程 200 开账时
-    /// 读到 25 440。进程 100 还借着 2 000 时，进程 200 借 6 000、再借 3 000 都当场借到，不等。
+    /// 读不出卡上的显存、读到的卡数与池对不上：判红（[`BookingRefusal::CardMemoryUnreadable`]），什么都不记。
     #[test]
-    fn an_outsider_leaving_after_the_first_process_opened_its_ledger_lets_a_later_process_book_what_it_now_reads(
-    ) {
-        let directory = LedgerDirectory::for_the_test("outsider-left");
-        let first = ledger_reading_the_free_memory(
-            &directory.path,
-            100,
-            &[100, 200],
-            10_000,
-            Duration::from_secs(60),
-        );
-        booking_outcome(&first, 2_000).expect("外人占着时卡上也放得下 2 000");
-        let second = ledger_reading_the_free_memory(
-            &directory.path,
-            200,
-            &[100, 200],
-            30_000 - DRIVER - 2_000,
-            Duration::from_millis(200),
+    fn an_unreadable_card_memory_is_refused_and_nothing_is_booked() {
+        let unreadable = GpuMemoryLedger::new(
+            1,
+            Box::new(|| Err("nvidia-smi 起不来".to_string())),
+            Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes),
+            DRIVER,
         );
         assert_eq!(
-            booking_outcome(&second, 6_000),
-            Ok(6_000),
-            "外人走了，卡上空着两万多：6 000 当场借到"
+            booking_kept_on_the_books(&unreadable, 1),
+            Err(BookingRefusal::CardMemoryUnreadable {
+                reason: "nvidia-smi 起不来".to_string()
+            })
         );
-        assert_eq!(
-            booking_outcome(&second, 3_000),
-            Ok(3_000),
-            "进程 100 还借着 2 000：3 000 也当场借到，不等"
+        let too_few = GpuMemoryLedger::new(
+            2,
+            Box::new(|| Ok(vec![reading_of_an_rtx_5090(30_000)])),
+            Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes),
+            DRIVER,
         );
+        assert!(matches!(
+            booking_kept_on_the_books(&too_few, 1),
+            Err(BookingRefusal::CardMemoryUnreadable { .. })
+        ));
+        assert!(!unreadable.is_this_process_registered_on_the_card(0));
+        assert!(!too_few.is_this_process_registered_on_the_card(0));
     }
 
-    /// 第 10t 批 K4-3 正推那一形：进程 100 把卡订满（借了它的整份容量），进程 200 开账时读到的空闲是 0。进程 200 借 1 000
-    /// 要等、不当成卡不够（不交「比每张卡都大」）；进程 100 还了，进程 200 重读账本时借到。
+    /// 第 10x 批「估计在压与借之间变小」那一形换了形态：调用方照空 30 000 时能借的压出 274 个每个 100 MiB 的状态；
+    /// 借之前外人占了卡（空 10 000），借的时候在锁里照现读的重压成 74 个（7 400 ≤ 10 000 − 驱动那一份），借到。
     #[test]
-    fn a_card_the_first_process_booked_full_makes_a_later_process_wait_instead_of_refusing_the_card(
-    ) {
-        let directory = LedgerDirectory::for_the_test("booked-full");
-        let first = ledger_reading_the_free_memory(
-            &directory.path,
-            100,
-            &[100, 200],
-            30_000,
-            Duration::from_secs(60),
-        );
-        let whole_card = first
-            .book_waiting_for_room(30_000 - DRIVER)
-            .expect("空着的卡放得下它的整份容量");
-        // 等着借的那条线程拿的是账的借用；判红时它可能一直等着，账交给 'static、线程不回收，判红不卡住整个用例
-        let second: &'static GpuMemoryLedger = Box::leak(Box::new(ledger_reading_the_free_memory(
-            &directory.path,
-            200,
-            &[100, 200],
-            0,
-            Duration::from_secs(3600),
-        )));
-        let (sender, receiver) = std::sync::mpsc::channel();
-        std::thread::spawn(move || {
-            sender
-                .send(booking_outcome(second, 1_000))
-                .expect("用例还在等这一条");
-        });
+    fn states_compressed_before_an_outsider_took_memory_are_compressed_again_in_the_booking() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+        let ledger = ledger_reading(&readings);
+        let need_of = |states: u64| states * 100;
+        let room_when_compressed = ledger.card_rooms_mebibytes_now(&[0]).expect("读得到")[0];
+        let states_compressed =
+            most_states_whose_need_fits_the_capacity(1_000, room_when_compressed, &|states| {
+                need_of(u64::try_from(states).expect("装得进 u64"))
+            });
+        assert_eq!(states_compressed, 274, "照 30 000 − 驱动那一份压出 274 个");
+        replace_the_reading(&readings, 0, reading_of_an_rtx_5090(10_000));
+        let bookings = ledger
+            .book_states_on_the_cards_with_room(&[0], 274, &[274], &need_of)
+            .expect("照现读的重压一次，借得到");
+        let booked = bookings[0].as_ref().expect("只有这一张卡，借到了");
+        assert_eq!(booked.states, 74, "照 10 000 − 驱动那一份重压成 74 个");
+        assert_eq!(booked.booking.mebibytes(), 7_400);
+    }
+
+    /// 第 10v 批：点名一张卡借，只记在那张卡上：第一张空着也不挪过去；要的比点名的那张空着时能借的大、比第一张小的，
+    /// 判成比卡大（判红），不记到第一张上。
+    #[test]
+    fn a_booking_on_a_named_card_stays_on_that_card_even_when_an_earlier_card_has_room() {
+        let ledger = GpuMemoryLedger::new(
+            2,
+            Box::new(|| {
+                Ok(vec![
+                    reading_of_an_rtx_5090(30_000),
+                    reading_of_an_rtx_5090(30_000),
+                ])
+            }),
+            Box::new(|position, may_hold_mebibytes| match position {
+                0 => may_hold_mebibytes.min(29_000),
+                _ => may_hold_mebibytes.min(12_000),
+            }),
+            DRIVER,
+        );
+        let booking = ledger
+            .book_on_the_card(1, 4_000)
+            .expect("第二张卡空着，借得到");
+        assert_eq!(booking.card_position(), 1, "点名第二张卡就记在第二张卡上");
+        drop(booking);
+        let refusal = ledger
+            .book_on_the_card(1, 20_000)
+            .map(|too_large_booking| too_large_booking.card_position())
+            .expect_err("要的比第二张卡空着时能借的大");
         assert!(
-            receiver.recv_timeout(Duration::from_millis(300)).is_err(),
-            "卡被进程 100 订满：进程 200 要等，不当场借到、也不当成卡不够"
+            matches!(
+                scratch_booking_refusal_outcome("crash_amplification::run", 1, &refusal),
+                ScratchBookingRefusalOutcome::Red { .. }
+            ),
+            "比点名的那张大要判红，不挪到第一张卡上：{refusal:?}"
         );
-        drop(whole_card);
-        assert_eq!(
-            receiver.recv_timeout(Duration::from_secs(30)),
-            Ok(Ok(1_000)),
-            "进程 100 还了之后，进程 200 借到"
+    }
+
+    /// 第 10x 批：产品路径此刻借不到照 GPU 对拍用例那一路报「本次未跑」，同一个记号、原因写显存不够；比卡大、读不出卡判红。
+    #[test]
+    fn a_booking_without_room_now_is_reported_not_run_with_the_gpu_not_run_marker() {
+        let no_room = BookingRefusal::NoCardHasRoomNow {
+            need_mebibytes: 4_000,
+            card_rooms_mebibytes: vec![1_000],
+        };
+        let ScratchBookingRefusalOutcome::NotRun { not_run_line } =
+            scratch_booking_refusal_outcome("crash_amplification::run", 0, &no_room)
+        else {
+            panic!("此刻借不到要报本次未跑");
+        };
+        assert!(
+            not_run_line.starts_with(
+                "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run reason=显存不够："
+            ),
+            "本次未跑那一行用 GPU 用例的记号（门禁 feature-gated-tests 格认的就是这一串）、原因写显存不够：{not_run_line}"
         );
+        for red in [
+            BookingRefusal::NeedLargerThanEveryCard(NeedLargerThanEveryCard {
+                need_mebibytes: 40_000,
+                largest_ceiling_mebibytes: 30_047,
+            }),
+            BookingRefusal::CardMemoryUnreadable {
+                reason: "nvidia-smi 起不来".to_string(),
+            },
+        ] {
+            assert!(
+                matches!(
+                    scratch_booking_refusal_outcome("crash_amplification::run", 0, &red),
+                    ScratchBookingRefusalOutcome::Red { .. }
+                ),
+                "{red:?} 判红"
+            );
+        }
     }
 
-    /// 第 10t 批 K4-3 先登记、后建设备：开账不登记；第一次借时连这个进程的驱动那一份一起登记；只登记驱动那一份的，放得下才登记，
-    /// 放不下等着、等满放弃，一直不登记。卡的估计 10 000：进程 100 登记并借 4 880（连驱动 7 440），进程 200 登记（连上 10 000），
-    /// 进程 300 的驱动那一份放不下。
+    /// 第 10t 批 K4-1：配置给这张卡 8 G、额度变量 20 000：容量不越过配置的额度（减驱动那一份），额度变量更小时取额度变量；
+    /// 没设额度变量照旧。数照一张清场后空 30 000 MiB 的 5090。
     #[test]
-    fn a_process_is_registered_on_a_card_only_once_its_first_booking_or_its_driver_share_fits() {
-        let directory = LedgerDirectory::for_the_test("register-before-the-device");
-        let first = ledger_reading_the_free_memory(
-            &directory.path,
-            100,
-            &[100, 200, 300],
-            10_000,
-            Duration::from_secs(60),
+    fn an_environment_quota_larger_than_the_configured_card_quota_stays_within_the_configured_card_quota(
+    ) {
+        const CONFIGURED_CARD_QUOTA: u64 = 8 * 1024;
+        let budget = CONFIGURED_CARD_QUOTA.min(30_000);
+        let capacity = card_ledger_capacity_mebibytes(
+            Some(20_000),
+            Some(CONFIGURED_CARD_QUOTA),
+            budget,
+            30_000,
+            DRIVER,
         );
-        assert_eq!(
-            first.is_this_process_registered_on_the_card(0),
-            Ok(false),
-            "开账只读空闲，不登记"
+        assert!(
+            capacity <= CONFIGURED_CARD_QUOTA,
+            "额度变量 20 000 不越过配置给卡的 8 G：实际 {capacity}"
         );
-        const FIRST_LEASE: u64 = 10_000 - 2 * DRIVER;
-        booking_outcome(&first, FIRST_LEASE).expect("空着的卡放得下");
         assert_eq!(
-            first.is_this_process_registered_on_the_card(0),
-            Ok(true),
-            "第一次借时登记"
-        );
-        let second = ledger_reading_the_free_memory(
-            &directory.path,
-            200,
-            &[100, 200, 300],
-            10_000 - DRIVER - FIRST_LEASE,
-            Duration::from_millis(200),
+            capacity,
+            CONFIGURED_CARD_QUOTA - DRIVER,
+            "取配置额度减驱动那一份"
         );
         assert_eq!(
-            second.register_this_process_on_the_card_waiting_for_room(0),
-            Ok(()),
-            "进程 200 的驱动那一份正好放得下"
-        );
-        assert_eq!(second.is_this_process_registered_on_the_card(0), Ok(true));
-        let third = ledger_reading_the_free_memory(
-            &directory.path,
-            300,
-            &[100, 200, 300],
-            0,
-            Duration::from_millis(200),
-        );
-        assert!(
-            matches!(
-                third.register_this_process_on_the_card_waiting_for_room(0),
-                Err(BookingRefusal::WaitedPastTheLongestWait { .. })
+            card_ledger_capacity_mebibytes(
+                Some(4_000),
+                Some(CONFIGURED_CARD_QUOTA),
+                budget,
+                30_000,
+                DRIVER
             ),
-            "进程 300 的驱动那一份放不下：等满放弃"
+            4_000,
+            "额度变量更小：取额度变量"
         );
         assert_eq!(
-            third.is_this_process_registered_on_the_card(0),
-            Ok(false),
-            "放不下就不登记（调用方不起设备）"
+            card_ledger_capacity_mebibytes(
+                None,
+                Some(CONFIGURED_CARD_QUOTA),
+                budget,
+                30_000,
+                DRIVER
+            ),
+            CONFIGURED_CARD_QUOTA - DRIVER,
+            "没设额度变量：照旧是给崩溃放量的额度减驱动那一份"
         );
     }
 
@@ -2003,16 +1530,15 @@ mod tests {
         );
     }
 
-    /// 设了卡的额度变量：账的容量取额度与「清场后的空闲显存 − 驱动那一份」里小的那个（额度小取额度，额度大取空闲显存那一边）；
-    /// 没设：照旧是这张卡给崩溃放量的额度减驱动那一份，与空闲显存无关。数照一张 5090（清场后空 30 000 MiB，3/4 是 22 500）。
+    /// 设了卡的额度变量：容量取额度与「这个进程能占的 − 驱动那一份」里小的那个（额度小取额度，额度大取能占的那一边）；
+    /// 没设：照旧是这张卡给崩溃放量的额度减驱动那一份。数照一张 5090（空 30 000 MiB，3/4 是 22 500）。
     #[test]
     fn a_card_quota_from_the_environment_caps_each_card_ledger_and_without_it_the_capacity_is_unchanged(
     ) {
-        const DRIVER: u64 = 2560;
         assert_eq!(
             card_ledger_capacity_mebibytes(Some(4096), None, 22_500, 30_000, DRIVER),
             4096,
-            "额度比空闲显存减驱动那一份小：取额度"
+            "额度比能占的减驱动那一份小：取额度"
         );
         assert_eq!(
             card_ledger_capacity_mebibytes(Some(40_000), None, 22_500, 30_000, DRIVER),
@@ -2026,18 +1552,18 @@ mod tests {
         );
     }
 
-    /// 第 10v 批：没配显卡优先级、设了额度变量 20 000、清场后空 30 000 MiB：没配额度时默认的 3/4 那一份（22 500）代替配置的额度，
+    /// 第 10v 批：没配显卡优先级、设了额度变量 20 000、能占 30 000 MiB：没配额度时默认的 3/4 那一份（22 500）代替配置的额度，
     /// 并进最小值，容量不超过它减驱动那一份。
     #[test]
     fn an_environment_quota_without_a_card_priority_configured_stays_within_the_default_three_quarter_share(
     ) {
-        const FREE_MEMORY_AFTER_QUIESCING: u64 = 30_000;
-        let three_quarter_share = FREE_MEMORY_AFTER_QUIESCING / 4 * 3;
+        const MAY_HOLD: u64 = 30_000;
+        let three_quarter_share = MAY_HOLD / 4 * 3;
         let capacity = card_ledger_capacity_mebibytes(
             Some(20_000),
             None,
             three_quarter_share,
-            FREE_MEMORY_AFTER_QUIESCING,
+            MAY_HOLD,
             DRIVER,
         );
         assert!(
@@ -2090,87 +1616,6 @@ mod tests {
         );
     }
 
-    /// 第 10v 批：卡被别的进程订满，产品路径在这张卡上借草稿区等满了：照 GPU 对拍用例那一路报「本次未跑」，同一个记号、同一句。
-    #[test]
-    fn a_scratch_booking_that_waits_past_the_longest_wait_is_reported_not_run_with_the_gpu_not_run_marker(
-    ) {
-        let directory = LedgerDirectory::for_the_test("scratch-waits-past-the-longest-wait");
-        let first = ledger_of_a_fake_process(
-            &directory.path,
-            1,
-            &[1, 2],
-            10_000,
-            0,
-            Duration::from_millis(20),
-            Duration::from_secs(5),
-        );
-        let held = first
-            .book_on_the_card_waiting_for_room(0, 10_000)
-            .expect("空着的卡上借得到整份");
-        let second = ledger_of_a_fake_process(
-            &directory.path,
-            2,
-            &[1, 2],
-            10_000,
-            0,
-            Duration::from_millis(20),
-            Duration::from_millis(100),
-        );
-        let refusal = second
-            .book_on_the_card_waiting_for_room(0, 4_000)
-            .map(|booking| booking.mebibytes())
-            .expect_err("卡被第一个进程订满，等满 100 毫秒借不到");
-        let outcome = scratch_booking_refusal_outcome("crash_amplification::run", 0, &refusal);
-        drop(held);
-        let ScratchBookingRefusalOutcome::NotRun { not_run_line } = &outcome else {
-            panic!("等满了借不到要报本次未跑，判成了 {outcome:?}");
-        };
-        assert!(
-            not_run_line
-                .starts_with("NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run reason=等了 "),
-            "本次未跑那一行用 GPU 用例的记号（门禁 feature-gated-tests 格认的就是这一串）与同一句：{not_run_line}"
-        );
-    }
-
-    /// 第 10v 批：点名一张卡借，只记在那张卡上：第一张卡空着也不挪过去；要的比点名的那张大、比第一张小的，判成放不下（判红），不记到第一张上。
-    #[test]
-    fn a_booking_on_a_named_card_stays_on_that_card_even_when_an_earlier_card_has_room() {
-        let directory = LedgerDirectory::for_the_test("booking-on-a-named-card");
-        let ledger = GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
-            settings_of_a_fake_process(
-                &directory.path,
-                1,
-                &[1],
-                0,
-                Duration::from_millis(20),
-                Duration::from_millis(100),
-            ),
-            vec!["card-first".to_string(), "card-second".to_string()],
-            &|_position| Ok(1000),
-            Box::new(|position, _estimate| match position {
-                0 => 29_000,
-                _ => 12_000,
-            }),
-        )
-        .expect("临时目录下的账本读写得了");
-        let booking = ledger
-            .book_on_the_card_waiting_for_room(1, 4_000)
-            .expect("第二张卡空着，借得到");
-        assert_eq!(booking.card_position(), 1, "点名第二张卡就记在第二张卡上");
-        drop(booking);
-        let refusal = ledger
-            .book_on_the_card_waiting_for_room(1, 20_000)
-            .map(|too_large_booking| too_large_booking.card_position())
-            .expect_err("要的比第二张卡的容量大");
-        assert!(
-            matches!(
-                scratch_booking_refusal_outcome("crash_amplification::run", 1, &refusal),
-                ScratchBookingRefusalOutcome::Red { .. }
-            ),
-            "放不下点名的那张卡要判红，不挪到第一张卡上：{refusal:?}"
-        );
-    }
-
     /// 第 10v 批：按判器那一族的算法（表 1 MiB，每个状态 1 261 616 字节）在 56 MiB 里装得下 45 个状态（45 个要 56、46 个要 57）；
     /// 上限内全放得下就取上限，一个都放不下取 0。
     #[test]
@@ -2197,37 +1642,10 @@ mod tests {
         );
     }
 
-    /// 两张卡的账（容量照 5090 与 5060 Ti 的量级，不看估计）：第一张 29 000、第二张 12 000 MiB，各登记驱动那一份
-    /// `driver_share_mebibytes`；进程号 1 是别的进程、2 是这个进程，都活着。
-    fn two_card_ledger_of_a_fake_process(
-        directory: &Path,
-        process_identifier: u32,
-        driver_share_mebibytes: u64,
-        longest_wait: Duration,
-    ) -> GpuMemoryLedger {
-        GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
-            settings_of_a_fake_process(
-                directory,
-                process_identifier,
-                &[1, 2],
-                driver_share_mebibytes,
-                Duration::from_millis(20),
-                longest_wait,
-            ),
-            vec!["card-first".to_string(), "card-second".to_string()],
-            &|_position| Ok(1000),
-            Box::new(|position, _estimate| match position {
-                0 => 29_000,
-                _ => 12_000,
-            }),
-        )
-        .expect("临时目录下的账本读写得了")
-    }
-
-    /// 第 10w 批「几张卡一张借不到时不拿着别的卡干等」：100 个状态、两张卡各带得了 60 个；第一张当场借到 60 个，第二张被订满。
-    /// 第二张只试一次、要装的是剩下的 40 个，借不到就不装它，判成就在第一张上判（剩下的 40 个排队等第一张），不等第二张。
+    /// 第 10w 批「几张卡一张借不到时不拿着别的卡干等」：100 个状态、两张卡各带得了 60 个；第一张当场借到 60 个，第二张此刻借不到。
+    /// 第二张只试一次、要装的是剩下的 40 个，借不到就不装它，判成就在第一张上判（剩下的 40 个排队等第一张）。
     #[test]
-    fn a_second_card_booked_full_leaves_every_state_to_the_first_card_without_waiting() {
+    fn a_second_card_without_room_leaves_every_state_to_the_first_card() {
         let mut attempts_made = Vec::new();
         let attempts = place_states_on_the_cards_that_book_now(
             100,
@@ -2262,12 +1680,12 @@ mod tests {
         assert_eq!(
             decide_after_attempting_every_card(&attempts),
             ScratchPlacementDecision::JudgeOnTheCardsBooked,
-            "第一张借到了：就在第一张上判，不等第二张"
+            "第一张借到了：就在第一张上判"
         );
     }
 
     /// 第 10w 批：借不到的卡那一份留给后面的卡。第一张此刻借不到，100 个状态里第二张装满它带得了的 60 个；
-    /// 第一张借到的比要装的少（在锁里照变小的估计压过，60 个里借到 30 个），第二张装的是剩下的 70 个与 60 里小的那个。
+    /// 第一张借到的比要装的少（在锁里照此刻能借的压过，60 个里借到 30 个），第二张装的是剩下的 70 个与 60 里小的那个。
     #[test]
     fn the_states_a_card_did_not_book_go_to_the_next_card() {
         let first_card_without_room = place_states_on_the_cards_that_book_now(
@@ -2307,13 +1725,14 @@ mod tests {
         );
     }
 
-    /// 第 10w 批：一张都没借到时，有空着时放得下的卡就等（要的是第一张放不下的卡那一笔），每张卡连一个状态都放不下就判红。
+    /// 第 10x 批：一张都没借到时，有空着时放得下的卡就是此刻借不到（要的是第一张放不下的卡那一笔，调用方报本次未跑），
+    /// 每张卡空着时连一个状态都放不下就判红。
     #[test]
-    fn with_no_card_booked_a_card_that_fits_when_empty_means_waiting_and_none_means_red() {
+    fn with_no_card_booked_a_card_that_fits_when_empty_means_no_room_now_and_none_means_red() {
         assert_eq!(
             decide_after_attempting_every_card(&[
                 ScratchAttemptOnACard::CannotHoldOneState {
-                    capacity_mebibytes: 40
+                    ceiling_mebibytes: 40
                 },
                 ScratchAttemptOnACard::NoRoomNow {
                     need_mebibytes: 3_000
@@ -2322,124 +1741,24 @@ mod tests {
                     need_mebibytes: 1_000
                 },
             ]),
-            ScratchPlacementDecision::WaitForRoom {
+            ScratchPlacementDecision::NoRoomOnAnyCardNow {
                 need_mebibytes: 3_000
             },
-            "第二、三张空着时放得下：等"
+            "第二、三张空着时放得下：此刻借不到"
         );
         assert_eq!(
             decide_after_attempting_every_card(&[
                 ScratchAttemptOnACard::CannotHoldOneState {
-                    capacity_mebibytes: 40
+                    ceiling_mebibytes: 40
                 },
                 ScratchAttemptOnACard::CannotHoldOneState {
-                    capacity_mebibytes: 90
+                    ceiling_mebibytes: 90
                 },
             ]),
             ScratchPlacementDecision::NoCardHoldsOneState {
-                largest_capacity_mebibytes: 90
+                largest_ceiling_mebibytes: 90
             },
-            "每张都连一个状态都放不下：判红，带最大的那张的容量"
-        );
-    }
-
-    /// 第 10w 批：真账本上两张卡，第二张被别的进程订满。这个进程借 400 个状态（每个 50 MiB、每张带得了 300 个）：
-    /// 第一张借到 300 个，第二张借不到就不装，当场交回、不等（最长的等待给 30 秒，判红时才等满）。
-    #[test]
-    fn a_scratch_booking_does_not_wait_for_a_card_booked_full_when_another_card_has_room() {
-        let directory = LedgerDirectory::for_the_test("scratch-skips-a-full-card");
-        let other =
-            two_card_ledger_of_a_fake_process(&directory.path, 1, 0, Duration::from_secs(1));
-        let held = other
-            .book_on_the_card_waiting_for_room(1, 12_000)
-            .expect("第二张卡空着，别的进程借得到整份");
-        let this =
-            two_card_ledger_of_a_fake_process(&directory.path, 2, 0, Duration::from_secs(30));
-        let started = Instant::now();
-        let bookings = this
-            .book_states_on_the_cards_waiting_only_when_none_has_room(
-                &[0, 1],
-                400,
-                &[300, 300],
-                &|states| states * 50,
-            )
-            .expect("第一张空着，借得到");
-        let elapsed = started.elapsed();
-        let states_booked: Vec<Option<u64>> = bookings
-            .iter()
-            .map(|booked| booked.as_ref().map(|booked| booked.states))
-            .collect();
-        drop(bookings);
-        drop(held);
-        assert_eq!(
-            states_booked,
-            vec![Some(300), None],
-            "第一张借到 300 个，被订满的第二张不装"
-        );
-        assert!(
-            elapsed < Duration::from_secs(10),
-            "第一张借到了就不等第二张：用了 {elapsed:?}"
-        );
-    }
-
-    /// 第 10w 批「估计在压与借之间变小 ⇒ 重压后借到」：进程 100 开账时空 30 000 MiB，照它的账（30 000 − 驱动那一份）压出 274 个
-    /// 每个 100 MiB 的状态；还没借，进程 200 开账读到只空 10 000（外人占了卡），把估计改成 10 000。进程 100 借那 274 个：
-    /// 在锁里照新的估计重压成 74 个（7 400 ≤ 10 000 − 驱动那一份），借到，不判成放不下。
-    #[test]
-    fn an_estimate_lowered_between_compressing_and_booking_is_compressed_again_and_the_booking_goes_through(
-    ) {
-        let directory = LedgerDirectory::for_the_test("estimate-lowered-before-booking");
-        let need_of = |states: u64| states * 100;
-        let first = ledger_reading_the_free_memory(
-            &directory.path,
-            100,
-            &[100, 200],
-            30_000,
-            Duration::from_millis(200),
-        );
-        let capacity_when_compressed = first.card_capacities_mebibytes().expect("账本读得了")[0];
-        let states_compressed =
-            most_states_whose_need_fits_the_capacity(1_000, capacity_when_compressed, &|states| {
-                need_of(u64::try_from(states).expect("装得进 u64"))
-            });
-        assert_eq!(states_compressed, 274, "照 30 000 − 驱动那一份压出 274 个");
-        let _second = ledger_reading_the_free_memory(
-            &directory.path,
-            200,
-            &[100, 200],
-            10_000,
-            Duration::from_millis(200),
-        );
-        let bookings = first
-            .book_states_on_the_cards_waiting_only_when_none_has_room(&[0], 274, &[274], &need_of)
-            .expect("估计变小之后照新的估计重压一次，借得到");
-        let booked = bookings[0].as_ref().expect("只有这一张卡，借到了");
-        assert_eq!(booked.states, 74, "照 10 000 − 驱动那一份重压成 74 个");
-        assert_eq!(booked.booking.mebibytes(), 7_400);
-    }
-
-    /// 第 10w 批：登记驱动那一份同样不拿着别的卡干等。第二张被别的进程订满（连这个进程的驱动那一份 1 000 都放不下），
-    /// 这个进程只登记在第一张上、当场交回，不等第二张。
-    #[test]
-    fn registering_on_the_cards_does_not_wait_for_a_card_booked_full_when_another_card_has_room() {
-        let directory = LedgerDirectory::for_the_test("register-skips-a-full-card");
-        let other =
-            two_card_ledger_of_a_fake_process(&directory.path, 1, 1_000, Duration::from_secs(1));
-        let held = other
-            .book_on_the_card_waiting_for_room(1, 12_000)
-            .expect("第二张卡空着，别的进程借得到整份");
-        let this =
-            two_card_ledger_of_a_fake_process(&directory.path, 2, 1_000, Duration::from_secs(30));
-        let started = Instant::now();
-        let registered = this
-            .register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(&[0, 1])
-            .expect("第一张空着，登记得上");
-        let elapsed = started.elapsed();
-        drop(held);
-        assert_eq!(registered, vec![true, false], "只登记在第一张上");
-        assert!(
-            elapsed < Duration::from_secs(10),
-            "第一张登记上了就不等第二张：用了 {elapsed:?}"
+            "每张都连一个状态都放不下：判红，带最大的那张空着时能借的"
         );
     }
 
@@ -2461,315 +1780,73 @@ mod tests {
         assert_eq!(mebibytes_of_buffers_held_at_once(0, 0, 0), 0);
     }
 
-    /// 两张卡（容量照 5090 与 5060 Ti 的量级）：前一张放得下就一直记在前一张上，前一张放不下这一笔才记到下一张。
+    /// 两张卡：前一张此刻放得下就一直记在前一张上，前一张放不下这一笔才记到下一张；每张都放不下而空着时放得下的是此刻借不到，
+    /// 比每张空着时都大的等多久都放不下。
     #[test]
     fn bookings_fill_the_first_card_before_the_next_card_takes_any() {
-        let capacities = [29_000, 12_000];
+        let room = |room_now_mebibytes: u64, ceiling_mebibytes: u64| CardRoomForThisProcess {
+            room_now_mebibytes,
+            ceiling_mebibytes,
+        };
         assert_eq!(
-            decide_booking_by_priority(&capacities, &[0, 0], 600),
+            decide_booking_by_priority(&[room(29_000, 29_000), room(12_000, 12_000)], 600),
             BookingDecision::BookOnTheCardAt { card_position: 0 },
             "两张都空着：记在优先的那张"
         );
         assert_eq!(
-            decide_booking_by_priority(&capacities, &[28_400, 0], 600),
+            decide_booking_by_priority(&[room(600, 29_000), room(12_000, 12_000)], 600),
             BookingDecision::BookOnTheCardAt { card_position: 0 },
-            "前一张剩的正好放得下：照旧记在前一张，不因为后一张更空就换过去"
+            "前一张剩的正好放得下：照旧记在前一张"
         );
         assert_eq!(
-            decide_booking_by_priority(&capacities, &[28_500, 0], 600),
+            decide_booking_by_priority(&[room(500, 29_000), room(12_000, 12_000)], 600),
             BookingDecision::BookOnTheCardAt { card_position: 1 },
             "前一张放不下这一笔：才记到下一张"
         );
-    }
-
-    /// 每张卡此刻都放不下而空着时放得下的等；比每张卡的容量都大的永远放不下（不等，交错误，账上什么都不记）。
-    #[test]
-    fn a_need_that_fits_an_empty_card_waits_and_one_larger_than_every_card_never_fits() {
-        let capacities = [100, 50];
         assert_eq!(
-            decide_booking_by_priority(&capacities, &[80, 40], 30),
-            BookingDecision::WaitForMemoryToBeReturned,
-            "两张都放不下 30，而第一张空着时放得下：等"
+            decide_booking_by_priority(&[room(20, 100), room(10, 50)], 30),
+            BookingDecision::NoRoomNow,
+            "两张此刻都放不下 30，而第一张空着时放得下：此刻借不到"
         );
         assert_eq!(
-            decide_booking_by_priority(&capacities, &[0, 0], 101),
+            decide_booking_by_priority(&[room(100, 100), room(50, 50)], 101),
             BookingDecision::NeverFits {
-                largest_capacity_mebibytes: 100
+                largest_ceiling_mebibytes: 100
             },
-            "比最大的那张还大：永远放不下"
+            "比最大的那张空着时还大：永远放不下"
         );
         assert_eq!(
-            decide_booking_by_priority(&[], &[], 1),
+            decide_booking_by_priority(&[], 1),
             BookingDecision::NeverFits {
-                largest_capacity_mebibytes: 0
+                largest_ceiling_mebibytes: 0
             },
             "一张卡都没有：永远放不下"
         );
-        let directory = LedgerDirectory::for_the_test("never-fits");
-        let ledger = ledger_of_a_fake_process(
-            &directory.path,
-            100,
-            &[100],
-            100,
-            0,
-            Duration::from_millis(20),
-            Duration::from_secs(60),
-        );
-        assert_eq!(
-            ledger.book_waiting_for_room(101).err(),
-            Some(BookingRefusal::NeedLargerThanEveryCard(
-                NeedLargerThanEveryCard {
-                    need_mebibytes: 101,
-                    largest_capacity_mebibytes: 100
-                }
-            )),
-            "账上借永远放不下的：当场交错误，不等"
-        );
-        let whole_card = ledger
-            .book_waiting_for_room(100)
-            .expect("交错误的那一次什么都没记：卡还是空的");
-        assert_eq!(whole_card.card_position(), 0);
     }
 
-    /// 同一个进程里：一张 100 MiB 的卡被一笔记满，另一条线程再借 30，还没还之前它一直等着，前一笔还回来它就记上同一张卡。
-    /// 重读账本的间隔设成 10 分钟：前一笔还的时候要当场叫醒等着的线程，不靠重读。
+    /// 一张卡此刻能借给这个进程的照 [`card_room_for_this_process`]：容量拿这个进程能占的算、减借着的；空着时能借的拿总量算、
+    /// 不减借着的。
     #[test]
-    fn a_booking_that_does_not_fit_waits_until_an_earlier_one_is_returned() {
-        let directory = LedgerDirectory::for_the_test("same-process");
-        // 等着借的那条线程拿的是账的借用；判红时它可能永远等着，账交给 'static、线程不回收，判红不卡住整个用例
-        let ledger: &'static GpuMemoryLedger = Box::leak(Box::new(ledger_of_a_fake_process(
-            &directory.path,
-            100,
-            &[100],
-            100,
-            0,
-            Duration::from_secs(600),
-            Duration::from_secs(3600),
-        )));
-        let first = ledger
-            .book_waiting_for_room(100)
-            .expect("空着的卡放得下 100");
-        let (sender, receiver) = std::sync::mpsc::channel();
-        std::thread::spawn(move || {
-            let second = ledger
-                .book_waiting_for_room(30)
-                .expect("30 比卡的容量小，放得下");
-            sender
-                .send((second.card_position(), second.mebibytes()))
-                .expect("用例还在等这一条");
-        });
-        assert!(
-            receiver
-                .recv_timeout(std::time::Duration::from_millis(300))
-                .is_err(),
-            "卡被前一笔记满了：后一笔要等，不许当场记上"
-        );
-        drop(first);
-        assert_eq!(
-            receiver.recv_timeout(std::time::Duration::from_secs(30)),
-            Ok((0, 30)),
-            "前一笔还回来之后，等着的那一笔记上同一张卡"
-        );
-    }
-
-    /// 两个进程（两本账、两个假进程号，同一个账本目录）各起一次池、各记一份：容量 100，第一个进程借 60，
-    /// 第二个进程看到的是 60 加第一个进程的驱动那一份 10，再借 40 放不下，等；第一个进程还了，第二个进程重读账本时记上。
-    #[test]
-    fn a_second_process_whose_booking_does_not_fit_waits_until_the_first_process_returns_its_booking(
-    ) {
-        let directory = LedgerDirectory::for_the_test("two-processes");
-        let first_process = ledger_of_a_fake_process(
-            &directory.path,
-            100,
-            &[100, 200],
-            100,
-            10,
-            Duration::from_millis(20),
-            Duration::from_secs(3600),
-        );
-        let second_process: &'static GpuMemoryLedger =
-            Box::leak(Box::new(ledger_of_a_fake_process(
-                &directory.path,
-                200,
-                &[100, 200],
-                100,
-                10,
-                Duration::from_millis(20),
-                Duration::from_secs(3600),
-            )));
-        let first = first_process
-            .book_waiting_for_room(60)
-            .expect("空着的卡放得下 60");
-        let (sender, receiver) = std::sync::mpsc::channel();
-        std::thread::spawn(move || {
-            let second = second_process
-                .book_waiting_for_room(40)
-                .expect("40 比卡的容量小，放得下");
-            sender
-                .send((second.card_position(), second.mebibytes()))
-                .expect("用例还在等这一条");
-        });
-        assert!(
-            receiver
-                .recv_timeout(std::time::Duration::from_millis(300))
-                .is_err(),
-            "第一个进程借的 60 与它的驱动那一份 10 加上要的 40 放不下：第二个进程要等"
+    fn the_room_now_subtracts_the_leases_and_the_ceiling_takes_the_total_memory() {
+        let room = card_room_for_this_process(
+            CardMemoryReading {
+                free_mebibytes: 20_000,
+                total_mebibytes: TOTAL_OF_AN_RTX_5090,
+                occupied_by_this_process_mebibytes: None,
+            },
+            BookedByThisProcessOnACard {
+                leases_mebibytes: 3_000,
+                driver_share: DriverShareOnTheCard::DeviceStarted,
+            },
+            DRIVER,
+            &|may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER),
         );
-        drop(first);
         assert_eq!(
-            receiver.recv_timeout(std::time::Duration::from_secs(30)),
-            Ok((0, 40)),
-            "第一个进程还了之后，第二个进程重读账本时记上"
-        );
-    }
-
-    /// 别的进程一直占着、等满最长的等待还空不出来：放弃，交「等满了」（调用方据此报本次未跑），账上什么都不记。
-    #[test]
-    fn a_booking_that_cannot_get_room_before_the_longest_wait_gives_up() {
-        let directory = LedgerDirectory::for_the_test("gives-up");
-        let first_process = ledger_of_a_fake_process(
-            &directory.path,
-            100,
-            &[100, 200],
-            100,
-            0,
-            Duration::from_millis(20),
-            Duration::from_secs(3600),
-        );
-        let second_process: &'static GpuMemoryLedger =
-            Box::leak(Box::new(ledger_of_a_fake_process(
-                &directory.path,
-                200,
-                &[100, 200],
-                100,
-                0,
-                Duration::from_millis(20),
-                Duration::from_millis(200),
-            )));
-        let _held = first_process
-            .book_waiting_for_room(100)
-            .expect("空着的卡放得下 100");
-        let (sender, receiver) = std::sync::mpsc::channel();
-        std::thread::spawn(move || {
-            let outcome = second_process
-                .book_waiting_for_room(30)
-                .map(|booking| booking.mebibytes());
-            sender.send(outcome).expect("用例还在等这一条");
-        });
-        match receiver.recv_timeout(std::time::Duration::from_secs(30)) {
-            Ok(Err(BookingRefusal::WaitedPastTheLongestWait {
-                need_mebibytes,
-                waited,
-                ..
-            })) => {
-                assert_eq!(need_mebibytes, 30);
-                assert!(
-                    waited >= Duration::from_millis(200),
-                    "等满了才放弃：只等了 {waited:?}"
-                );
+            room,
+            CardRoomForThisProcess {
+                room_now_mebibytes: 20_000 - 3_000,
+                ceiling_mebibytes: TOTAL_OF_AN_RTX_5090 - DRIVER,
             }
-            other => panic!("等满最长的等待要放弃，交「等满了」；实际 {other:?}"),
-        }
-    }
-
-    /// 记账进程已经不在了，它记的那一份不算：进程 300 借满 90（借据丢掉不还，就是它被杀了），还当它活着的进程 400 借 90 等到放弃；
-    /// 判活判成它不在了的进程 500 当场借到，账本里 300 记的那几行跟着删掉。
-    #[test]
-    fn a_booking_left_by_a_process_that_is_no_longer_alive_is_not_counted() {
-        let directory = LedgerDirectory::for_the_test("dead-process");
-        let killed_process = ledger_of_a_fake_process(
-            &directory.path,
-            300,
-            &[300, 400, 500],
-            100,
-            0,
-            Duration::from_millis(20),
-            Duration::from_secs(3600),
-        );
-        std::mem::forget(
-            killed_process
-                .book_waiting_for_room(90)
-                .expect("空着的卡放得下 90"),
-        );
-        let sees_it_alive = ledger_of_a_fake_process(
-            &directory.path,
-            400,
-            &[300, 400],
-            100,
-            0,
-            Duration::from_millis(20),
-            Duration::from_millis(200),
-        );
-        assert!(
-            matches!(
-                sees_it_alive.book_waiting_for_room(90).err(),
-                Some(BookingRefusal::WaitedPastTheLongestWait { .. })
-            ),
-            "进程 300 还活着时它借的 90 照算：再借 90 放不下"
-        );
-        let sees_it_dead = ledger_of_a_fake_process(
-            &directory.path,
-            500,
-            &[500],
-            100,
-            0,
-            Duration::from_millis(20),
-            Duration::from_millis(200),
-        );
-        let booking = sees_it_dead
-            .book_waiting_for_room(90)
-            .expect("进程 300 不在了，它借的那一份不算：90 当场放得下");
-        assert_eq!(booking.card_position(), 0);
-        let ledger_text =
-            std::fs::read_to_string(directory.path.join("card-test.ledger")).expect("账本在");
-        assert!(
-            !ledger_text
-                .lines()
-                .any(|line| line.split_whitespace().nth(1) == Some("300")),
-            "不在了的进程 300 记的行在下一次写账时删掉：\n{ledger_text}"
-        );
-    }
-
-    /// 跑这条用例的进程判成还在；同一个进程号配上另一个启动时刻（进程号被后起的进程重用）判成不在。
-    #[test]
-    fn the_process_running_the_test_is_alive_and_its_identifier_with_another_start_time_is_not() {
-        let this_process = RecordedProcess::this_process().expect("Linux 上读得到 /proc/self/stat");
-        assert!(
-            process_is_alive_by_its_entry_in_the_process_filesystem(this_process),
-            "跑用例的进程还在"
-        );
-        assert!(
-            !process_is_alive_by_its_entry_in_the_process_filesystem(RecordedProcess {
-                process_identifier: this_process.process_identifier,
-                start_time_in_clock_ticks: this_process.start_time_in_clock_ticks + 1,
-            }),
-            "进程号对上、启动时刻对不上：是别的进程，原来记账的那个不在了"
         );
     }
-
-    /// 账本目录在工作根下：设了 `SINGLEFS_WORK_ROOT` 照它（开头的 `~` 照家目录展开），设成空串与没设一样落到家目录下的默认工作根，
-    /// 要家目录而没有家目录交 `None`。
-    #[test]
-    fn the_ledger_directory_is_under_the_work_root_variable_or_the_default_work_root_under_home() {
-        let home = Some(OsStr::new("/home/someone"));
-        assert_eq!(
-            gpu_memory_ledger_directory(Some(OsStr::new("/srv/work")), home),
-            Some(PathBuf::from("/srv/work").join(GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME))
-        );
-        assert_eq!(
-            gpu_memory_ledger_directory(Some(OsStr::new("~/work")), home),
-            Some(PathBuf::from("/home/someone/work").join(GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME))
-        );
-        for unset in [None, Some(OsStr::new(""))] {
-            assert_eq!(
-                gpu_memory_ledger_directory(unset, home),
-                Some(
-                    PathBuf::from("/home/someone/.local/share/singlefs/work")
-                        .join(GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME)
-                ),
-                "工作根没设（{unset:?}）：家目录下的默认工作根"
-            );
-        }
-        assert_eq!(gpu_memory_ledger_directory(None, None), None);
-    }
 }
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index e12d371..e7780fc 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -80,8 +80,8 @@ pub struct GpuCard {
     pub pci_bus_identifier: String,
     /// `nvidia-smi` 给这张卡的序号（配置按它点名）。
     pub index: u32,
-    /// 起上下文之前 `nvidia-smi` 报的空闲显存（MiB；卡池里的卡是开账时在账本锁里读的那一份）。
-    /// 卡池借出去的卡改成借的量加驱动那一份：账判过卡上放得下这么多，不让起池那一刻的读数把草稿区压得比借的小。
+    /// 起上下文之前 `nvidia-smi` 报的空闲显存（MiB；卡池里的卡是起池时找卡那一刻报的，只当说明，借卡不看它）。
+    /// 卡池借出去的卡改成借的量加驱动那一份：账判过卡上放得下这么多，不让找卡那一刻的读数把草稿区压得比借的小。
     pub free_memory_mebibytes: u64,
     /// 配置给这张卡的显存额度（MiB）：崩溃放量在这张卡上占的显存不超过它；没给额度的是 `None`。
     /// 卡池借出去的卡改成借的量加驱动那一份。
@@ -93,18 +93,19 @@ pub struct GpuCard {
     compiled_pipelines_by_source_digest: std::sync::Arc<CompiledPipelinesBySourceDigest>,
 }
 
-/// `nvidia-smi` 报的一张卡：序号与空闲显存 MiB。
+/// `nvidia-smi` 报的一张卡：序号、空闲显存与显存总量（MiB）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct ReportedCard {
     index: u32,
     free_memory_mebibytes: u64,
+    total_memory_mebibytes: u64,
 }
 
-/// `nvidia-smi` 报的每张卡：PCI 总线号（小写、去掉前导的 0 域）→ 序号与空闲显存。起不来 `nvidia-smi` 就是空表。
+/// `nvidia-smi` 报的每张卡：PCI 总线号（小写、去掉前导的 0 域）→ 序号、空闲显存与显存总量。起不来 `nvidia-smi` 就是空表。
 fn reported_cards_by_pci_bus() -> std::collections::BTreeMap<String, ReportedCard> {
     let Ok(output) = std::process::Command::new("nvidia-smi")
         .args([
-            "--query-gpu=pci.bus_id,index,memory.free",
+            "--query-gpu=pci.bus_id,index,memory.free,memory.total",
             "--format=csv,noheader,nounits",
         ])
         .output()
@@ -118,17 +119,54 @@ fn reported_cards_by_pci_bus() -> std::collections::BTreeMap<String, ReportedCar
             let bus = fields.next()?;
             let index = fields.next()?.trim().parse().ok()?;
             let free_memory_mebibytes = fields.next()?.trim().parse().ok()?;
+            let total_memory_mebibytes = fields.next()?.trim().parse().ok()?;
             Some((
                 normalized_bus(bus),
                 ReportedCard {
                     index,
                     free_memory_mebibytes,
+                    total_memory_mebibytes,
                 },
             ))
         })
         .collect()
 }
 
+/// `nvidia-smi` 按进程报的显存里这个进程占着的：PCI 总线号 → MiB（同一张卡上几行的加起来）。起不来 `nvidia-smi` 交 `None`；
+/// 这个进程在某张卡上没被列出（没起设备，或驱动不按进程报 Vulkan 的上下文），那张卡不在表里。
+fn memory_used_by_this_process_on_each_bus() -> Option<std::collections::BTreeMap<String, u64>> {
+    let output = std::process::Command::new("nvidia-smi")
+        .args([
+            "--query-compute-apps=pid,gpu_bus_id,used_memory",
+            "--format=csv,noheader,nounits",
+        ])
+        .output()
+        .ok()?;
+    if !output.status.success() {
+        return None;
+    }
+    let this_process = std::process::id();
+    let mut used_by_bus = std::collections::BTreeMap::new();
+    for line in String::from_utf8_lossy(&output.stdout).lines() {
+        let mut fields = line.split(',');
+        let (Some(process_identifier), Some(bus), Some(used)) =
+            (fields.next(), fields.next(), fields.next())
+        else {
+            continue;
+        };
+        let (Ok(process_identifier), Ok(used_mebibytes)) = (
+            process_identifier.trim().parse::<u32>(),
+            used.trim().parse::<u64>(),
+        ) else {
+            continue;
+        };
+        if process_identifier == this_process {
+            *used_by_bus.entry(normalized_bus(bus)).or_insert(0u64) += used_mebibytes;
+        }
+    }
+    Some(used_by_bus)
+}
+
 /// 总线号只留 `总线:设备.功能` 那一段、小写：`00000000:01:00.0` 与 `0000:01:00.0` 认成同一张卡。
 fn normalized_bus(text: &str) -> String {
     let lowered = text.trim().to_ascii_lowercase();
@@ -261,7 +299,7 @@ pub fn usable_gpu_cards() -> Vec<GpuCard> {
 
 /// 一张卡给崩溃放量的额度（MiB）：给了额度（`memory_quota_mebibytes`）的取额度与空闲显存里小的那个，
 /// 没给的取空闲显存的 [`SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA`]（[`GpuCard::memory_budget_mebibytes`] 照它算；卡池照它、
-/// 拿账本里记的清场后那一份空闲显存算每张卡的账）。
+/// 拿这个进程此刻能在卡上占的算每张卡的账，`singlefs_harness::gpu_memory_ledger` 文件头「口径」）。
 #[must_use]
 pub fn memory_budget_mebibytes_of(
     memory_quota_mebibytes: Option<u64>,
@@ -927,14 +965,15 @@ pub mod gpu_test_start {
     /// GPU 用例共用的卡池（[`card_pool::GpuCardPool`]）：只开 `gpu` 也编得进（显卡配置的读法在 `crate::gpu_card_configuration`）。
     pub use card_pool::{
         card_pool_started_in_this_process, gpu_card_pool_or_report_not_run,
-        CardsRegisteredForTheProductPath, GpuCardLease, GpuCardPool, ScratchLeaseOnACard,
+        CardOfThePoolForTheProductPath, GpuCardLease, GpuCardPool, ScratchLeaseOnACard,
         CARD_QUOTA_ENVIRONMENT_VARIABLE,
     };
 
     mod card_pool {
         use super::super::{
-            memory_budget_mebibytes_of, nvidia_discrete_cards_reported_by_nvidia_smi,
-            reported_cards_by_pci_bus, start_the_card, DiscoveredCard, GpuCard,
+            memory_budget_mebibytes_of, memory_used_by_this_process_on_each_bus,
+            nvidia_discrete_cards_reported_by_nvidia_smi, reported_cards_by_pci_bus,
+            start_the_card, DiscoveredCard, GpuCard,
         };
         use super::{
             decide_gpu_test_start, enumerate_vulkan_adapters, not_run_line,
@@ -946,40 +985,34 @@ pub mod gpu_test_start {
             CardHost,
         };
         use singlefs_harness::gpu_memory_ledger::{
-            card_ledger_capacity_mebibytes, gpu_memory_ledger_directory_of_this_process,
-            process_is_alive_by_its_entry_in_the_process_filesystem,
-            reason_for_a_booking_that_waited_past_the_longest_wait, BookingRefusal,
-            GpuMemoryBooking, GpuMemoryLedger, LedgerOpeningRefusal, LedgerSettings,
-            RecordedProcess, StatesBookedOnACard, LONGEST_WAIT_FOR_GPU_MEMORY,
-            POLL_INTERVAL_FOR_MEMORY_RETURNED_BY_OTHER_PROCESSES,
+            card_ledger_capacity_mebibytes, reason_for_a_booking_without_room_now, BookingRefusal,
+            CardMemoryReading, GpuMemoryBooking, GpuMemoryLedger, StatesBookedOnACard,
         };
         use singlefs_harness::numeric_environment_variable::{
             number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
         };
 
-        /// 每张卡账的上限（MiB）：设了，每张卡的账取它、配置给这张卡的额度减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]、
-        /// 记账进程合起来能占的显存减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`] 三者里最小的
+        /// 每张卡账的上限（MiB）：设了，每张卡上这个进程一共能占的取它、配置给这张卡的额度减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]、
+        /// 这个进程此刻能占的减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`] 三者里最小的
         /// （`singlefs_harness::gpu_memory_ledger::card_ledger_capacity_mebibytes`）；不是每条用例的上限，借得比它多的照常判「放不下」。
         pub const CARD_QUOTA_ENVIRONMENT_VARIABLE: &str =
             "SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES";
 
-        /// 池里的一张卡：照配置点了名、Vulkan 与 `nvidia-smi` 里都找得到；设备等这个进程在这张卡的账上登记了之后才起
+        /// 池里的一张卡：照配置点了名、Vulkan 与 `nvidia-smi` 里都找得到；设备等这个进程在这张卡上借到一笔之后才起
         /// （[`GpuCardPool::started_card`]，一个进程在一张卡上只起一次）。
         struct CardOfThePool {
             discovered: DiscoveredCard,
             /// `GPU_CARDS_BY_PRIORITY` 配给这张卡的额度（MiB）；没配显卡优先级的是 `None`。
             configured_card_quota_mebibytes: Option<u64>,
-            /// 开账时在这张卡的账本锁里读的空闲显存（MiB）。
-            free_memory_read_when_the_ledger_opened_mebibytes: u64,
             started: std::sync::OnceLock<Result<GpuCard, String>>,
         }
 
-        /// GPU 用例共用的卡：这台机器点名的每张卡各一本跨进程的显存账（[`GpuMemoryLedger`]，账本在工作根下），每张卡这个进程只起一个设备；
+        /// GPU 用例共用的卡：这台机器点名的每张卡在这个进程里一本显存账（[`GpuMemoryLedger`]，只记这个进程借的，
+        /// 每次借现读卡上此刻的显存：`singlefs_harness::gpu_memory_ledger` 文件头），每张卡这个进程只起一个设备；
         /// 用例按自己实际要的量借（[`GpuCardPool::lease_card_or_report_not_run`]），借据丢掉时还。
-        /// 一张卡的账照 [`card_ledger_capacity_mebibytes`]（账本里的估计怎么来见 `singlefs_harness::gpu_memory_ledger` 文件头）：
-        /// 驱动的上下文与编好的内核每个进程在每张卡上付一份 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]，同一台机器上别的进程各再记一份。
-        /// 起池不看卡上此刻空闲多少（被先起的进程订满的卡照样进池），能不能借由账判；这个进程第一次在一张卡上借（连它的驱动那一份）
-        /// 记上了，才在这张卡上起设备。
+        /// 一张卡这个进程一共能占的照 [`card_ledger_capacity_mebibytes`]：驱动的上下文与编好的内核每个进程在每张卡上付一份
+        /// [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]。起池不读卡上此刻空闲多少，能不能借在借的那一刻判；这个进程第一次在一张卡上借到
+        /// （连它的驱动那一份）了，才在这张卡上起设备，没借过的卡不起（第 10t 批代码轮判决 K2-3）。
         pub struct GpuCardPool {
             cards: Vec<CardOfThePool>,
             ledger: GpuMemoryLedger,
@@ -1003,20 +1036,33 @@ pub mod gpu_test_start {
             pub fn mebibytes(&self) -> u64 {
                 self.booking.mebibytes()
             }
+
+            /// 这一笔的借据号（[`GpuCardPool::is_the_lease_on_the_books`] 照它查）。
+            #[must_use]
+            pub fn lease_identifier(&self) -> u64 {
+                self.booking.lease_identifier()
+            }
+
+            /// 这一笔此刻还记在这个进程的账上没有（核账：有卡用例照它判借据活到了用完，第 10t 批代码轮判决 K4-1）。
+            #[must_use]
+            pub fn is_on_the_books(&self) -> bool {
+                self.booking.is_on_the_books()
+            }
         }
 
-        /// 崩溃放量产品路径在一张卡上借到的草稿区：借来的卡与借了几个状态（在账本锁里照此刻的估计压过）。
+        /// 崩溃放量产品路径在一张卡上借到的草稿区：借来的卡与借了几个状态（在账的锁里照此刻能借的压过）。
         pub struct ScratchLeaseOnACard {
             pub lease: GpuCardLease,
             pub states: u64,
         }
 
-        /// 崩溃放量产品路径这一趟用池里哪几张卡（[`GpuCardPool::cards_registered_waiting_only_when_none_has_room`]）。
-        pub enum CardsRegisteredForTheProductPath {
-            /// 此刻登记得上的那几张（按池的优先级，至少一张）：设备起好了。
-            Started(Vec<GpuCard>),
-            /// 一张都登记不上、等满了：调用方报「本次未跑」，`reason` 是等满那一句。
-            NoCardHadRoomBeforeTheLongestWait { reason: String },
+        /// 崩溃放量产品路径看池里的一张卡（还没起设备，[`GpuCardPool::cards_for_the_product_path`]）：序号、名字与适配器的存储缓冲绑定上限
+        /// （设备按适配器的上限要，`start_the_card`）。
+        #[derive(Clone, Debug, PartialEq, Eq)]
+        pub struct CardOfThePoolForTheProductPath {
+            pub index: u32,
+            pub name: String,
+            pub max_storage_buffer_binding_size: u64,
         }
 
         impl Drop for GpuCardLease {
@@ -1050,46 +1096,43 @@ pub mod gpu_test_start {
                     )
             }
 
-            /// 第 `card_position` 张卡起好的设备：第一次要时起（调用方先在这张卡的账上登记了这个进程），之后拿同一个；起不来交一句为什么。
+            /// 借到 `booking` 的那张卡起好的设备：第一次要时起（借到就登记了这个进程，先登记、后建设备），之后拿同一个；
+            /// 起不来交一句为什么。要一笔借据才起得了：没借过的卡起不了设备。
             ///
             /// # Panics
-            /// 这个进程还没在这张卡的账上登记就来起设备（先登记、后建设备的次序被打乱了）；账本读写不了。
-            fn started_card(&self, card_position: usize) -> Result<&GpuCard, String> {
+            /// 这个进程还没在这张卡的账上登记（`GpuMemoryLedger::check_registered_before_starting_the_device`）。
+            fn started_card(
+                &self,
+                booking: &GpuMemoryBooking<'static>,
+            ) -> Result<&GpuCard, String> {
+                let card_position = booking.card_position();
                 let card = &self.cards[card_position];
                 card.started
                     .get_or_init(|| {
-                        assert!(
-                            self.ledger
-                                .is_this_process_registered_on_the_card(card_position)
-                                .unwrap_or_else(|error| panic!(
-                                    "显存账本 {} 用不了：{}",
-                                    error.path.display(),
-                                    error.reason
-                                )),
-                            "序号 {} 的卡：这个进程在它的账上登记之前就要起设备（先登记、后建设备）",
-                            card.discovered.reported.index
-                        );
+                        self.ledger
+                            .check_registered_before_starting_the_device(card_position);
                         let mut started = start_the_card(&card.discovered).ok_or_else(|| {
                             format!(
-                                "序号 {} 的卡（{}）上起不了设备：账上登记了这个进程，卡却起不来上下文（驱动坏了，或卡被不经账的程序占满）",
+                                "序号 {} 的卡（{}）上起不了设备：账上登记了这个进程，卡却起不来上下文（驱动坏了，或卡被别的程序占满）",
                                 card.discovered.reported.index, card.discovered.name
                             )
                         })?;
+                        self.ledger
+                            .record_the_device_started_on_the_card(card_position);
                         started.memory_quota_mebibytes = card.configured_card_quota_mebibytes;
-                        started.free_memory_mebibytes =
-                            card.free_memory_read_when_the_ledger_opened_mebibytes;
                         Ok(started)
                     })
                     .as_ref()
                     .map_err(Clone::clone)
             }
 
-            /// 借 `need_mebibytes`：照卡的优先级记在第一张放得下的卡上（前一张放满了才用下一张）；此刻每张都放不下就等别的用例、
-            /// 别的 GPU 测试进程还，等满 [`LONGEST_WAIT_FOR_GPU_MEMORY`] 还借不到就打 [`not_run_line`] 那一行、交回 `None`（调用的用例当场返回）。
-            /// 每借到一次打一行 `GPU_CARD_LEASE test=… index=… mebibytes=… capacity_mebibytes=…`。
+            /// 借 `need_mebibytes`：照卡的优先级记在第一张此刻放得下的卡上（前一张放满了才用下一张）；此刻每张都放不下
+            /// （这个进程别的用例或卡上别的 GPU 用户占着）不等，打 [`not_run_line`] 那一行（原因写显存不够）、交回 `None`
+            /// （调用的用例当场返回）。每借到一次打一行 `GPU_CARD_LEASE test=… index=… mebibytes=… room_mebibytes=…`。
             ///
             /// # Panics
-            /// 要的比每张卡的账都大（配了 GPU 就一定用 GPU，放不下判红，不悄悄换小）；账本文件读写不了；借到的卡上起不了设备。
+            /// 要的比每张卡空着时能借的都大（配了 GPU 就一定用 GPU，放不下判红，不悄悄换小）；卡上的显存读不出；借到的卡上起不了设备；
+            /// 要 0 MiB。
             #[must_use]
             pub fn lease_card_or_report_not_run(
                 &'static self,
@@ -1098,18 +1141,19 @@ pub mod gpu_test_start {
             ) -> Option<GpuCardLease> {
                 match self.lease_card(test_name, need_mebibytes) {
                     Ok(lease) => Some(lease),
-                    Err(waited) => {
-                        println!("{}", not_run_line(test_name, &waited));
+                    Err(reason) => {
+                        println!("{}", not_run_line(test_name, &reason));
                         None
                     }
                 }
             }
 
-            /// 同 [`Self::lease_card_or_report_not_run`]，等满了判红而不报「本次未跑」：给还没改成报本次未跑的调用方留着
-            /// （`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`）。
+            /// 同 [`Self::lease_card_or_report_not_run`]，此刻借不到判红而不报「本次未跑」：给还没改成报本次未跑的调用方留着
+            /// （`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`）。名字沿用那几处调用方的写法：已经不等了
+            /// （第 10x 批：只记这个进程借的，借不到不等）。
             ///
             /// # Panics
-            /// 同 [`Self::lease_card_or_report_not_run`]；等满 [`LONGEST_WAIT_FOR_GPU_MEMORY`] 还借不到。
+            /// 同 [`Self::lease_card_or_report_not_run`]；此刻每张卡都借不到。
             #[must_use]
             pub fn lease_card_waiting_for_room(
                 &'static self,
@@ -1117,66 +1161,64 @@ pub mod gpu_test_start {
                 need_mebibytes: u64,
             ) -> GpuCardLease {
                 self.lease_card(test_name, need_mebibytes)
-                    .unwrap_or_else(|waited| {
-                        panic!("{test_name}：{waited}（这一处还没改成经 lease_card_or_report_not_run 报本次未跑）")
+                    .unwrap_or_else(|reason| {
+                        panic!("{test_name}：{reason}（这一处还没改成经 lease_card_or_report_not_run 报本次未跑）")
                     })
             }
 
-            /// 借一次：借到交借来的卡，等满了交一句为什么。
+            /// 借一次：借到交借来的卡，此刻借不到交一句为什么。
             fn lease_card(
                 &'static self,
                 test_name: &str,
                 need_mebibytes: u64,
             ) -> Result<GpuCardLease, String> {
-                let booking = match self.ledger.book_waiting_for_room(need_mebibytes) {
+                let booking = match self.ledger.book_on_the_first_card_with_room(need_mebibytes) {
                     Ok(booking) => booking,
                     Err(BookingRefusal::NeedLargerThanEveryCard(refusal)) => panic!(
-                        "{test_name}：要借 {} MiB，池里最大的一张卡的账只有 {} MiB：这台机器的卡放不下这条用例，\
-                         腾出显存、给卡更大的额度（GPU_CARDS_BY_PRIORITY），或缩小用例",
-                        refusal.need_mebibytes, refusal.largest_capacity_mebibytes
+                        "{test_name}：要借 {} MiB，池里最大的一张卡空着时也只借得出 {} MiB：这台机器的卡放不下这条用例，\
+                         给卡更大的额度（GPU_CARDS_BY_PRIORITY），或缩小用例",
+                        refusal.need_mebibytes, refusal.largest_ceiling_mebibytes
                     ),
-                    Err(BookingRefusal::WaitedPastTheLongestWait {
-                        need_mebibytes: waited_for_mebibytes,
-                        waited,
-                        card_capacities_mebibytes,
+                    Err(BookingRefusal::NoCardHasRoomNow {
+                        need_mebibytes: wanted_mebibytes,
+                        card_rooms_mebibytes,
                     }) => {
-                        return Err(reason_for_a_booking_that_waited_past_the_longest_wait(
-                            waited_for_mebibytes,
-                            waited,
-                            &card_capacities_mebibytes,
+                        return Err(reason_for_a_booking_without_room_now(
+                            wanted_mebibytes,
+                            &card_rooms_mebibytes,
                         ))
                     }
-                    Err(BookingRefusal::LedgerFileUnusable(error)) => panic!(
-                        "{test_name}：显存账本 {} 用不了：{}",
-                        error.path.display(),
-                        error.reason
-                    ),
+                    Err(BookingRefusal::CardMemoryUnreadable { reason }) => {
+                        panic!("{test_name}：卡上的显存读不出：{reason}")
+                    }
                 };
                 Ok(self.card_lent_for_the_booking(test_name, booking))
             }
 
-            /// 同 [`Self::lease_card_or_report_not_run`]（照卡的优先级记在第一张放得下的卡上，此刻每张都放不下才等），借不到交账的结局、
+            /// 同 [`Self::lease_card_or_report_not_run`]（照卡的优先级记在第一张此刻放得下的卡上），借不到交账的结局、
             /// 由调用方定怎么报：崩溃放量产品路径在起过卡池的进程里，第 ② 段核对与单元校验和对拍在卡上建的缓冲照它借
-            /// （`singlefs_harness::gpu_memory_ledger::scratch_booking_refusal_outcome`：等满了报本次未跑，别的判红）。
+            /// （`singlefs_harness::gpu_memory_ledger::scratch_booking_refusal_outcome`：此刻借不到报本次未跑，别的判红）。
             ///
             /// # Errors
             /// 见 `singlefs_harness::gpu_memory_ledger::BookingRefusal`。
             ///
             /// # Panics
-            /// 借到的卡上起不了设备。
-            pub fn lease_the_first_card_with_room_waiting(
+            /// 借到的卡上起不了设备；要 0 MiB。
+            pub fn lease_the_first_card_with_room_now(
                 &'static self,
                 run_name: &str,
                 need_mebibytes: u64,
             ) -> Result<GpuCardLease, BookingRefusal> {
-                let booking = self.ledger.book_waiting_for_room(need_mebibytes)?;
+                let booking = self
+                    .ledger
+                    .book_on_the_first_card_with_room(need_mebibytes)?;
                 Ok(self.card_lent_for_the_booking(run_name, booking))
             }
 
             /// 序号是 `index`（`nvidia-smi` 给的）的那张卡在池里排第几。
             ///
             /// # Panics
-            /// 池里没有这张卡（崩溃放量产品路径的卡都取自这个池，[`Self::cards_registered_waiting_only_when_none_has_room`]）。
+            /// 池里没有这张卡（崩溃放量产品路径的卡都取自这个池，[`Self::cards_for_the_product_path`]）。
             fn position_of_the_card_with_index(&self, index: u32) -> usize {
                 self.cards
                     .iter()
@@ -1187,17 +1229,18 @@ pub mod gpu_test_start {
             }
 
             /// 崩溃放量产品路径第 ① 段借草稿区：`card_indices` 是这一趟用的卡（按优先级，都是池里的卡），每张最多装
-            /// `states_each_card_holds` 个状态；把 `pending_states` 个状态装到此刻借得到的卡上，每张卡在账本锁里照此刻的估计压状态数再借
-            /// （几个状态要多少显存照 `mebibytes_for_states`），借不到的卡不装、不等，一张都借不到才等，等满放弃
-            /// （`singlefs_harness::gpu_memory_ledger::GpuMemoryLedger::book_states_on_the_cards_waiting_only_when_none_has_room`）。
-            /// 借到的每张交成借来的卡（额度是借的量加驱动那一份，判器照它开出来的草稿区正好是借的状态数）；交回与 `card_indices` 逐张对上。
+            /// `states_each_card_holds` 个状态；把 `pending_states` 个状态装到此刻借得到的卡上，在账的锁里现读每张卡、照此刻能借的
+            /// 压状态数再借（几个状态要多少显存照 `mebibytes_for_states`），借不到的卡不装
+            /// （`singlefs_harness::gpu_memory_ledger::GpuMemoryLedger::book_states_on_the_cards_with_room`）。
+            /// 借到的每张交成借来的卡（额度是借的量加驱动那一份，判器照它开出来的草稿区正好是借的状态数；借到了才在那张卡上起设备）；
+            /// 交回与 `card_indices` 逐张对上。
             ///
             /// # Errors
             /// 见 `singlefs_harness::gpu_memory_ledger::BookingRefusal`。
             ///
             /// # Panics
             /// 有一张不是池里的卡；借到的卡上起不了设备。
-            pub fn book_the_scratch_of_the_cards_waiting_only_when_none_has_room(
+            pub fn book_the_scratch_of_the_cards_with_room_now(
                 &'static self,
                 run_name: &str,
                 card_indices: &[u32],
@@ -1211,7 +1254,7 @@ pub mod gpu_test_start {
                     .collect();
                 Ok(self
                     .ledger
-                    .book_states_on_the_cards_waiting_only_when_none_has_room(
+                    .book_states_on_the_cards_with_room(
                         &card_positions,
                         pending_states,
                         states_each_card_holds,
@@ -1229,7 +1272,8 @@ pub mod gpu_test_start {
                     .collect())
             }
 
-            /// 记上了的那一笔交成借来的卡：与池里那张共用设备，额度与空闲改成借的量加驱动那一份；打一行 `GPU_CARD_LEASE`。
+            /// 记上了的那一笔交成借来的卡：在那张卡上起设备（第一次借到时，[`Self::started_card`]），与池里那张共用设备，
+            /// 额度与空闲改成借的量加驱动那一份；打一行 `GPU_CARD_LEASE`。
             ///
             /// # Panics
             /// 那张卡上起不了设备。
@@ -1240,41 +1284,62 @@ pub mod gpu_test_start {
             ) -> GpuCardLease {
                 let need_mebibytes = booking.mebibytes();
                 let mut card = self
-                    .started_card(booking.card_position())
+                    .started_card(&booking)
                     .unwrap_or_else(|reason| panic!("{test_name}：{reason}"))
                     .clone();
                 card.memory_quota_mebibytes = Some(need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER);
                 card.free_memory_mebibytes = need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER;
                 eprintln!(
-                    "GPU_CARD_LEASE test={test_name} card={:?} index={} mebibytes={need_mebibytes} capacity_mebibytes={}",
+                    "GPU_CARD_LEASE test={test_name} card={:?} index={} mebibytes={need_mebibytes} room_mebibytes={}",
                     card.name,
                     card.index,
-                    booking.card_capacity_mebibytes_when_booked()
+                    booking.card_room_mebibytes_when_booked()
                 );
                 GpuCardLease { card, booking }
             }
 
-            /// 这个进程此刻看 `card_indices` 这几张卡（都是池里的卡）的账各能借多少（MiB，驱动那一份已经扣掉；与 `card_indices` 逐张对上）。
+            /// 这个进程此刻在 `card_indices` 这几张卡（都是池里的卡）上各能借多少（MiB，现读一次；与 `card_indices` 逐张对上）。
             ///
             /// # Errors
-            /// 账本读写不了。
+            /// 卡上的显存读不出。
             ///
             /// # Panics
             /// 有一张不是池里的卡。
-            pub fn card_capacities_mebibytes_of(
+            pub fn card_rooms_mebibytes_now_of(
                 &self,
                 card_indices: &[u32],
             ) -> Result<Vec<u64>, String> {
-                let capacities_in_pool_order =
-                    self.ledger.card_capacities_mebibytes().map_err(|error| {
-                        format!("显存账本 {} 用不了：{}", error.path.display(), error.reason)
-                    })?;
-                Ok(card_indices
+                let card_positions: Vec<usize> = card_indices
                     .iter()
-                    .map(|index| {
-                        capacities_in_pool_order[self.position_of_the_card_with_index(*index)]
-                    })
-                    .collect())
+                    .map(|index| self.position_of_the_card_with_index(*index))
+                    .collect();
+                self.ledger
+                    .card_rooms_mebibytes_now(&card_positions)
+                    .map_err(|refusal| format!("读不出卡上此刻的显存：{refusal:?}"))
+            }
+
+            /// 序号是 `index` 的那张卡上，借据号 `lease_identifier` 的那一笔此刻还记在这个进程的账上没有（核账，第 10t 批代码轮判决 K4-1）。
+            ///
+            /// # Panics
+            /// 池里没有这张卡。
+            #[must_use]
+            pub fn is_the_lease_on_the_books(&self, index: u32, lease_identifier: u64) -> bool {
+                self.ledger.is_the_lease_on_the_books(
+                    self.position_of_the_card_with_index(index),
+                    lease_identifier,
+                )
+            }
+
+            /// 序号是 `index` 的那张卡上这个进程起了设备没有（只在借到过的卡上起，第 10t 批代码轮判决 K2-3）。
+            ///
+            /// # Panics
+            /// 池里没有这张卡。
+            #[must_use]
+            pub fn is_the_device_started_on_the_card(&self, index: u32) -> bool {
+                self.cards[self.position_of_the_card_with_index(index)]
+                    .started
+                    .get()
+                    .is_some_and(Result::is_ok)
             }
 
             /// 池里每张卡（按优先级）的序号与配置给它的额度（MiB，没配显卡优先级的是 `None`），不起设备。
@@ -1292,59 +1357,22 @@ pub mod gpu_test_start {
             }
 
             /// 崩溃放量产品路径这一趟用池里哪几张卡（`crate::crash_amplification`：这个进程起过卡池时，放量照池里的卡判，不另起一套设备）：
-            /// 这个进程在每张卡的账上登记（只记驱动那一份，不借），每张当场试一次、不等，此刻登记不上的那几张这一趟不用；
-            /// 一张都登记不上才等，等满交 [`CardsRegisteredForTheProductPath::NoCardHadRoomBeforeTheLongestWait`]
-            /// （第 10w 批：主 agent 定几张卡一张借不到时不拿着别的卡干等）。交回的卡是池里那张的克隆（额度是配置给这张卡的、
-            /// 空闲是开账时读的），按池的优先级；放量第 ① 段开的草稿区另按每张卡装的状态数借
-            /// （[`Self::book_the_scratch_of_the_cards_waiting_only_when_none_has_room`]）。
-            ///
-            /// # Errors
-            /// 账本读写不了，或登记上的卡上起不了设备。
-            pub fn cards_registered_waiting_only_when_none_has_room(
-                &'static self,
-            ) -> Result<CardsRegisteredForTheProductPath, String> {
-                let every_card: Vec<usize> = (0..self.cards.len()).collect();
-                let registered = match self
-                    .ledger
-                    .register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(
-                        &every_card,
-                    ) {
-                    Ok(registered) => registered,
-                    Err(BookingRefusal::WaitedPastTheLongestWait {
-                        need_mebibytes,
-                        waited,
-                        card_capacities_mebibytes,
-                    }) => {
-                        return Ok(
-                            CardsRegisteredForTheProductPath::NoCardHadRoomBeforeTheLongestWait {
-                                reason: reason_for_a_booking_that_waited_past_the_longest_wait(
-                                    need_mebibytes,
-                                    waited,
-                                    &card_capacities_mebibytes,
-                                ),
-                            },
-                        );
-                    }
-                    Err(BookingRefusal::LedgerFileUnusable(error)) => {
-                        return Err(format!(
-                            "显存账本 {} 用不了：{}",
-                            error.path.display(),
-                            error.reason
-                        ));
-                    }
-                    Err(BookingRefusal::NeedLargerThanEveryCard(refusal)) => {
-                        return Err(format!(
-                            "登记驱动那一份不借，账却判成要的比每张卡都大（{refusal:?}）：登记那一笔要的是 0"
-                        ));
-                    }
-                };
-                every_card
-                    .into_iter()
-                    .zip(registered)
-                    .filter(|(_, is_registered)| *is_registered)
-                    .map(|(card_position, _)| self.started_card(card_position).cloned())
-                    .collect::<Result<Vec<GpuCard>, String>>()
-                    .map(CardsRegisteredForTheProductPath::Started)
+            /// 池里每张卡，按池的优先级，不登记、不起设备（第 10t 批代码轮判决 K2-3：只在用到的卡上起设备）。放量借到哪张卡
+            /// （[`Self::book_the_scratch_of_the_cards_with_room_now`]、[`Self::lease_the_first_card_with_room_now`]）才在哪张上起。
+            #[must_use]
+            pub fn cards_for_the_product_path(&self) -> Vec<CardOfThePoolForTheProductPath> {
+                self.cards
+                    .iter()
+                    .map(|card| CardOfThePoolForTheProductPath {
+                        index: card.discovered.reported.index,
+                        name: card.discovered.name.clone(),
+                        max_storage_buffer_binding_size: card
+                            .discovered
+                            .adapter
+                            .limits()
+                            .max_storage_buffer_binding_size,
+                    })
+                    .collect()
             }
         }
 
@@ -1367,7 +1395,7 @@ pub mod gpu_test_start {
 
         /// 照显卡配置挑这台机器的卡：`GPU_CARDS_BY_PRIORITY`（环境变量，没设读 `SINGLEFS_GPU_CARDS_CONFIG` 指的显卡配置里的同名键）
         /// 里这台机器（`SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`，没设是本机）的那几项，照配置的次序与额度（与崩溃放量取卡同一个约定）；
-        /// 都没配就是本机每张 NVIDIA 独显，按空闲显存从大到小、不设额度。只找卡、不起设备，也不看此刻空闲多少（能不能借由账判）。
+        /// 都没配就是本机每张 NVIDIA 独显，按空闲显存从大到小、不设额度。只找卡、不起设备，也不看此刻空闲多少（能不能借在借的那一刻判）。
         fn cards_named_for_this_machine() -> CardsNamedForThisMachine {
             let refused = |reason: String| CardsNamedForThisMachine {
                 cards: Vec::new(),
@@ -1443,90 +1471,71 @@ pub mod gpu_test_start {
             }
         }
 
-        /// 一张卡的账本文件名的主干：按 PCI 总线号（换掉 `:`），同一台机器上几个进程认得出是同一张卡。
-        fn ledger_file_stem_of(card: &DiscoveredCard) -> String {
-            format!("card-{}", card.pci_bus_identifier.replace(':', "-"))
+        /// 池里每张卡此刻的显存（与 `buses` 逐张对上，`buses` 是池里每张卡的 PCI 总线号）：一次 `nvidia-smi` 读空闲与总量，
+        /// 一次按进程读这个进程占着的（读不出、这个进程在那张卡上没被列出的是 `None`）。
+        ///
+        /// # Errors
+        /// `nvidia-smi` 里读不到某张卡。
+        fn memory_of_the_cards_on_these_buses(
+            buses: &[String],
+        ) -> Result<Vec<CardMemoryReading>, String> {
+            let reported_by_bus = reported_cards_by_pci_bus();
+            let used_by_this_process = memory_used_by_this_process_on_each_bus();
+            buses
+                .iter()
+                .map(|bus| {
+                    let reported = reported_by_bus
+                        .get(bus)
+                        .ok_or_else(|| format!("nvidia-smi 里读不到总线 {bus} 那张卡的显存"))?;
+                    Ok(CardMemoryReading {
+                        free_mebibytes: reported.free_memory_mebibytes,
+                        total_mebibytes: reported.total_memory_mebibytes,
+                        occupied_by_this_process_mebibytes: used_by_this_process
+                            .as_ref()
+                            .and_then(|used_by_bus| used_by_bus.get(bus).copied()),
+                    })
+                })
+                .collect()
         }
 
-        /// 开点名的那几张卡的账（每张卡在锁里现读一次空闲显存，[`GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock`]），
-        /// 不起设备；交回池。
-        fn open_the_card_pool(
-            named: Vec<(DiscoveredCard, Option<u64>)>,
-        ) -> Result<GpuCardPool, String> {
+        /// 开点名的那几张卡的账（这个进程里，不读卡、不起设备）；交回池。
+        fn open_the_card_pool(named: Vec<(DiscoveredCard, Option<u64>)>) -> GpuCardPool {
             let card_quota = number_from_the_variable_value_panicking_when_unreadable::<u64>(
                 CARD_QUOTA_ENVIRONMENT_VARIABLE,
                 process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),
             );
-            let directory = gpu_memory_ledger_directory_of_this_process().ok_or_else(|| {
-                "显存账本放在工作根下，SINGLEFS_WORK_ROOT 与 HOME 都没设，定不下工作根".to_string()
-            })?;
-            let this_process = RecordedProcess::this_process().ok_or_else(|| {
-                "读不到 /proc/self/stat：账本要按进程号与启动时刻判记账进程还在不在".to_string()
-            })?;
-            let settings = LedgerSettings {
-                directory,
-                this_process,
-                is_alive: Box::new(process_is_alive_by_its_entry_in_the_process_filesystem),
-                mebibytes_each_process_keeps_on_a_card: MEBIBYTES_LEFT_FOR_THE_DRIVER,
-                poll_interval_for_memory_returned_by_other_processes:
-                    POLL_INTERVAL_FOR_MEMORY_RETURNED_BY_OTHER_PROCESSES,
-                longest_wait: LONGEST_WAIT_FOR_GPU_MEMORY,
-            };
-            let file_stems: Vec<String> = named
+            let buses: Vec<String> = named
                 .iter()
-                .map(|(card, _)| ledger_file_stem_of(card))
+                .map(|(card, _)| card.pci_bus_identifier.clone())
                 .collect();
             let configured_card_quotas: Vec<Option<u64>> =
                 named.iter().map(|(_, quota)| *quota).collect();
-            let free_memory_read_in_the_lock: Vec<std::cell::Cell<Option<u64>>> =
-                named.iter().map(|_| std::cell::Cell::new(None)).collect();
-            let read_free_memory_mebibytes = |position: usize| -> Result<u64, String> {
-                let bus = &named[position].0.pci_bus_identifier;
-                let free = reported_cards_by_pci_bus()
-                    .get(bus)
-                    .map(|reported| reported.free_memory_mebibytes)
-                    .ok_or_else(|| format!("nvidia-smi 里读不到总线 {bus} 那张卡的空闲显存"))?;
-                free_memory_read_in_the_lock[position].set(Some(free));
-                Ok(free)
-            };
-            let ledger = GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
-                settings,
-                file_stems,
-                &read_free_memory_mebibytes,
-                Box::new(move |position, estimate| {
+            let ledger = GpuMemoryLedger::new(
+                named.len(),
+                Box::new(move || memory_of_the_cards_on_these_buses(&buses)),
+                Box::new(move |position, may_hold_mebibytes| {
                     let configured = configured_card_quotas[position];
                     card_ledger_capacity_mebibytes(
                         card_quota,
                         configured,
-                        memory_budget_mebibytes_of(configured, estimate),
-                        estimate,
+                        memory_budget_mebibytes_of(configured, may_hold_mebibytes),
+                        may_hold_mebibytes,
                         MEBIBYTES_LEFT_FOR_THE_DRIVER,
                     )
                 }),
-            )
-            .map_err(|refusal| match refusal {
-                LedgerOpeningRefusal::LedgerFileUnusable(error) => {
-                    format!("显存账本 {} 用不了：{}", error.path.display(), error.reason)
-                }
-                LedgerOpeningRefusal::FreeMemoryUnreadable { file_stem, reason } => {
-                    format!("开 {file_stem} 那张卡的账时读不出空闲显存：{reason}")
-                }
-            })?;
+                MEBIBYTES_LEFT_FOR_THE_DRIVER,
+            );
             let cards = named
                 .into_iter()
-                .zip(free_memory_read_in_the_lock)
                 .map(
-                    |((discovered, configured_card_quota_mebibytes), read)| CardOfThePool {
+                    |(discovered, configured_card_quota_mebibytes)| CardOfThePool {
                         discovered,
                         configured_card_quota_mebibytes,
-                        free_memory_read_when_the_ledger_opened_mebibytes: read
-                            .get()
-                            .expect("开账成了，每张卡都在锁里读过一次空闲显存"),
                         started: std::sync::OnceLock::new(),
                     },
                 )
                 .collect();
-            Ok(GpuCardPool { cards, ledger })
+            GpuCardPool { cards, ledger }
         }
 
         fn start_the_gpu_card_pool() -> GpuCardPoolStart {
@@ -1545,10 +1554,7 @@ pub mod gpu_test_start {
             match decision {
                 GpuTestStart::RunOnTheUsableCards => {
                     if named.named_but_not_found.is_empty() {
-                        match open_the_card_pool(named.cards) {
-                            Ok(pool) => GpuCardPoolStart::Started(pool),
-                            Err(reason) => GpuCardPoolStart::CardsPresentButNotUsable { reason },
-                        }
+                        GpuCardPoolStart::Started(open_the_card_pool(named.cards))
                     } else {
                         // 点名的卡少一张都不跑（用户 2026-09-29 定：配了 GPU 就要它干活，不行就明确报错，不接受少用几张的默认降级）
                         GpuCardPoolStart::CardsPresentButNotUsable {
@@ -1574,13 +1580,13 @@ pub mod gpu_test_start {
 
         static GPU_CARD_POOL: std::sync::OnceLock<GpuCardPoolStart> = std::sync::OnceLock::new();
 
-        /// 这个进程的卡池：第一条来要的用例起池（只在这时判一次有没有卡、找一次点名的卡、在每张卡的账本锁里读一次空闲显存，
-        /// 挑卡照 [`cards_named_for_this_machine`]；设备等借到那张卡时才起），之后每条用例拿同一个。本机没有 NVIDIA 卡时每条来要的用例各打一行
+        /// 这个进程的卡池：第一条来要的用例起池（只在这时判一次有没有卡、找一次点名的卡，挑卡照 [`cards_named_for_this_machine`]；
+        /// 设备等借到那张卡时才起），之后每条用例拿同一个。本机没有 NVIDIA 卡时每条来要的用例各打一行
         /// [`not_run_line`]、交回 `None`（调用的用例当场返回）。先判有没有卡、建好表知道要多少显存之后再借（[`GpuCardPool::lease_card_or_report_not_run`]）。
         ///
         /// # Panics
         /// 有卡而用不上（同 [`super::usable_cards_or_report_not_run`]）；显卡配置写错、没给这台机器点卡、点名的卡少一张找不到；
-        /// [`CARD_QUOTA_ENVIRONMENT_VARIABLE`] 设了而认不出；账本目录定不下、账本读写不了、开账时读不出空闲显存。
+        /// [`CARD_QUOTA_ENVIRONMENT_VARIABLE`] 设了而认不出。
         #[must_use]
         pub fn gpu_card_pool_or_report_not_run(test_name: &str) -> Option<&'static GpuCardPool> {
             match GPU_CARD_POOL.get_or_init(start_the_gpu_card_pool) {
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index b25ad07..3476fbe 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -14,17 +14,17 @@
 //!    领块只在内存里，库里不记谁领了，不会有块被一张死卡占着。
 //! 4. **分工与跑法**：块按权重分给几方（一方一台机器的 CPU，或一张卡，权重取空闲显存），各写各的库，最后
 //!    [`crate::verdict_store::VerdictStore::import_from`] 并成一个；跑法由配置的两个开关定：`ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`。
-//! 5. **显存账**（`singlefs_harness::gpu_memory_ledger`，只防这台机器的显存 OOM、不压并发：用户 2026-10-06 定）：这个进程起过 GPU 用例共用的卡池
-//!    （[`GpuCardsSource::TheCardPoolOfThisProcess`]）时，取卡照池里的卡、驱动那一份在账上登记；第 ① 段判器的草稿区按派活装到每张卡上的状态数
-//!    借（照 state_count 封顶，`singlefs_harness::gpu_memory_ledger::scratch_states_to_book_on_each_card`，显存按与 GPU 对拍用例同一份的
-//!    `crate::crash_judge_dispatch::mebibytes_a_judge_needs` 算，只记在那张卡上），每张卡装几个状态先照账能借的量压（借得到才装）；
-//!    等满了借不到，照对拍用例那一路打「本次未跑」那一行（同一个记号），这一趟照叫停的样子交回（`stopped_early`，判过的块在库里，
-//!    分项驱动把这一项记成本次未跑；派出的项全没判完时整趟退 77）。只排派活计划那一趟试判前同样借。
-//!    几张卡一起借（第 10w 批，主 agent 定）：取卡时登记驱动那一份、第 ① 段借草稿区，都按卡的先后每张当场试一次、不等，借不到的卡
-//!    这一趟不用（不装状态，它那一份留给后面的卡），一张都借不到才等、等满报本次未跑；每张卡装几个状态在账本锁里照此刻的估计再压一次
-//!    （压完到借之间别的进程把估计改小了，照新的估计重压再借，不判红）。
+//! 5. **显存账**（`singlefs_harness::gpu_memory_ledger`，只防这台机器的显存 OOM、不压并发：用户 2026-10-06 定；只记这个进程借的、
+//!    每次借现读卡上此刻的显存：用户 2026-10-07 定第 10t 批代码轮判决岔路 G1「二」，口径在那份文件头）：这个进程起过 GPU 用例共用的卡池
+//!    （[`GpuCardsSource::TheCardPoolOfThisProcess`]）时，取卡照池里的卡、不登记也不起设备，借到哪张卡才在哪张上起（第 10t 批代码轮判决 K2-3）；
+//!    第 ① 段判器的草稿区按派活装到每张卡上的状态数借（照 state_count 封顶，`singlefs_harness::gpu_memory_ledger::scratch_states_to_book_on_each_card`，
+//!    显存按与 GPU 对拍用例同一份的 `crate::crash_judge_dispatch::mebibytes_a_judge_needs` 算，只记在那张卡上），每张卡装几个状态照这一刻
+//!    这张卡能借给这个进程的量压（不照起池那一刻的空闲，第 10t 批代码轮攻方 S5）。几张卡按卡的先后每张当场试一次，借不到的卡这一趟
+//!    不用（不装状态，它那一份留给后面的卡），一张都借不到就照对拍用例那一路打「本次未跑」那一行（同一个记号，原因写显存不够），
+//!    这一趟照叫停的样子交回（`stopped_early`，判过的块在库里，分项驱动把这一项记成本次未跑；派出的项全没判完时整趟退 77）。不等。
+//!    只排派活计划那一趟试判前同样借。
 //!    第 ② 段核对（`crash_verify_gpu`）与第 ① 段由 CPU 判时的单元校验和对拍（`gpu_unit_checks`）在卡上建的缓冲同样进账：照卡的先后借第一张
-//!    放得下的（此刻每张都放不下才等），用借来的那张卡算，等满了报本次未跑（判过的块在库里）。量照
+//!    此刻放得下的，用借来的那张卡算，借不到报本次未跑（判过的块在库里）。量照
 //!    `singlefs_harness::gpu_memory_ledger::mebibytes_of_buffers_held_at_once`，下面的字节数是照两份代码建的缓冲推的，没上卡量过。
 //!    第 ② 段（`facts_verifier_buffer_footprint`）：上卡一次的六张事实表（`crate::crash_verify_gpu::GpuFactsVerifier::new`：
 //!    录制流每次写 4 个字、每对重放 2 个、每段 6 个、段里每次写 1 个、每个单元 6 个、每条根 38 个，空表也建一个字），
@@ -33,7 +33,11 @@
 //!    单元校验和（`unit_checksum_buffer_footprint`）：一个状态一趟，这条流里每次单元写各两段
 //!    （`crate::gpu_unit_checks::bytes_crc32c_of_ranges_needs`），核对线程数（[`checking_threads`]）条线程同时派活，
 //!    每条同时还活着的照 `PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD` 算。
-//!    驱动的上下文与编好的管线算在登记时的驱动那一份里，不另记。
+//!    驱动的上下文与编好的管线算在每张卡第一次借时的驱动那一份里，不另记。
+//!    账只记这个进程借的：两个进程同一刻读到同一份空闲、一起借超了，由运行时兜底：第 ① 段卡上建判器、判一块时 wgpu 报 Out of Memory，
+//!    这张卡停下、这一块不入库、这一趟报本次未跑（原因写显存不够，`singlefs_harness::gpu_memory_ledger::reason_for_running_out_of_gpu_memory`）；
+//!    第 ② 段在卡上核时同样。单元校验和对拍几条线程一起派活，wgpu 的错误范围按线程分，一个范围接不住几条线程，这一段没接：
+//!    那里的 Out of Memory 照旧让这一趟红。
 //!    没起过卡池的进程照旧不进账（[`GpuCardsSource::DevicesStartedForThisRun`]：这一趟自己在点名的卡上起设备），调用方是这几种：
 //!    崩溃放量分项驱动（`research/scripts/crash-amplification-item.sh`）起的、只有经 [`run_from_environment`] 跑的用例而没有一条取卡池的测试二进制
 //!    （`tests/crash_amplification_fixed_script_through_rollback.rs`、`…_through_unmount.rs`、`crash_amplification_two_flows_share_their_prefix.rs`、
@@ -53,9 +57,9 @@ use singlefs_core::root_record::ROOT_MAGIC;
 #[cfg(feature = "gpu")]
 use singlefs_harness::gpu_memory_ledger::{
     gpu_not_run_line, mebibytes_of_buffers_held_at_once, most_states_whose_need_fits_the_capacity,
-    scratch_booking_refusal_outcome, scratch_states_to_book_on_each_card, BookingRefusal,
-    NeedLargerThanEveryCard, ScratchBookingRefusalOutcome, ScratchStatesToBook,
-    PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
+    reason_for_running_out_of_gpu_memory, scratch_booking_refusal_outcome,
+    scratch_states_to_book_on_each_card, BookingRefusal, NeedLargerThanEveryCard,
+    ScratchBookingRefusalOutcome, ScratchStatesToBook, PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
 };
 use singlefs_harness::hexadecimal::hexadecimal_text;
 use singlefs_harness::memory_pool::{
@@ -1746,6 +1750,9 @@ pub struct GpuCardOutcome {
     pub seconds_of_the_longest_dispatch: f64,
     /// 起过卡池的进程在显存账上给这张卡的判器借的草稿区（MiB，文件头第 5 条）；不进账（卡是这一趟自己起的、或调用方给的）是 `None`。
     pub scratch_booked_mebibytes: Option<u64>,
+    /// 起过卡池的进程，这张卡的判器建好那一刻核账：草稿区那一笔借据还在账上没有（第 10t 批代码轮判决 K4-1：借了又在判之前还掉要红）；
+    /// 不进账是 `None`。
+    pub scratch_lease_on_the_books_when_the_judge_was_built: Option<bool>,
 }
 
 impl GpuCardOutcome {
@@ -1864,15 +1871,19 @@ enum CardMessage {
     Finished(GpuCardOutcome),
     /// 这张卡判不了领到的块里的某个状态（表越过了内核的容量）：整趟停下、报错，不退回 CPU 判器。
     Refused(String),
+    /// 这张卡建判器或判一块时 wgpu 报了 Out of Memory（文件头第 5 条）：整趟停下，这一块不入库，这一趟报本次未跑；带那一句为什么。
+    OutOfMemory(String),
 }
 
-/// 写库那条线程收完卡上的线程发来的话之后手里的：按块序写进库几块、收工的卡各自的账（按收到的次序）、写库的错、卡拒判的理由。
+/// 写库那条线程收完卡上的线程发来的话之后手里的：按块序写进库几块、收工的卡各自的账（按收到的次序）、写库的错、卡拒判的理由、
+/// 显存不够的那一句。
 #[cfg(feature = "gpu")]
 struct CardsMessagesReceived {
     blocks_stored: usize,
     finished_cards: Vec<GpuCardOutcome>,
     storing_error: Option<String>,
     refusal: Option<String>,
+    out_of_memory: Option<String>,
 }
 
 /// 写库那条线程收卡上的线程发来的话，到发送端全丢掉为止：判完的块按块序交 `store_block` 写库（后面的块先到就等前面那一块，
@@ -1893,6 +1904,7 @@ fn receive_the_cards_messages_and_store_the_blocks_in_order(
         finished_cards: Vec::new(),
         storing_error: None,
         refusal: None,
+        out_of_memory: None,
     };
     // 写库：按块序。后面的块先判完就在这里等，前面那一块到了一起写
     let mut waiting: BTreeMap<usize, GpuJudgedBlock> = BTreeMap::new();
@@ -1916,6 +1928,9 @@ fn receive_the_cards_messages_and_store_the_blocks_in_order(
             ReceivedOrWallClockLineDue::Received(CardMessage::Refused(reason)) => {
                 received.refusal.get_or_insert(reason);
             }
+            ReceivedOrWallClockLineDue::Received(CardMessage::OutOfMemory(reason)) => {
+                received.out_of_memory.get_or_insert(reason);
+            }
             ReceivedOrWallClockLineDue::Received(CardMessage::Block(arrived_block)) => {
                 waiting.insert(arrived_block.pending_index, arrived_block);
                 while let Some(block) = waiting.remove(&received.blocks_stored) {
@@ -1944,6 +1959,8 @@ struct CardsAtWork<'work> {
     states_placed: &'work [u64],
     /// 每张卡的判器在显存账上借的草稿区（MiB）；不进账的是 `None`。
     scratch_booked_mebibytes: &'work [Option<u64>],
+    /// 每张卡草稿区那一笔借据的借据号（判器建好那一刻核账用）；不进账的是 `None`。
+    scratch_lease_identifiers: &'work [Option<u64>],
     next_pending: std::sync::atomic::AtomicUsize,
     /// 写库失败之后置上：卡上的线程判完手里那一块就停。
     stop: std::sync::atomic::AtomicBool,
@@ -1958,9 +1975,34 @@ impl CardsAtWork<'_> {
     }
 }
 
+/// 在 `card` 上做 `work`，同时接住 wgpu 报的 Out of Memory（文件头第 5 条）：没报就交 `work` 的结果；报了交 `Err(那一句)`
+/// （`what_ran_out` 说是哪一段，`work` 里因为缓冲没建成而 panic 的一并接住、不再往外抛）；没报而 `work` panic 了照样往外抛。
+/// wgpu 的错误范围按线程分：`work` 要在调它的这条线程上建缓冲、派活。
+#[cfg(feature = "gpu")]
+fn on_the_card_catching_out_of_memory<Outcome>(
+    card: &crate::gpu_unit_checks::GpuCard,
+    what_ran_out: &str,
+    work: impl FnOnce() -> Outcome,
+) -> Result<Outcome, String> {
+    let out_of_memory_scope = card
+        .device()
+        .push_error_scope(wgpu::ErrorFilter::OutOfMemory);
+    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
+    match (outcome, pollster::block_on(out_of_memory_scope.pop())) {
+        (Ok(outcome), None) => Ok(outcome),
+        (Ok(_) | Err(_), Some(error)) => Err(reason_for_running_out_of_gpu_memory(&format!(
+            "序号 {} 的卡{what_ran_out}（{error}）",
+            card.index
+        ))),
+        (Err(panic), None) => std::panic::resume_unwind(panic),
+    }
+}
+
 /// 一张卡的线程：表上卡、编内核；备好之后，队里还有活、下一张卡也装了状态，就起下一张卡的线程（卡一张一张备，不同时编内核）；
 /// 然后从队头一块一块领、判、把结论字拆成红项、交给写库的线程。有一个状态判不了（结论字带「判不了」），
 /// 就叫所有卡停下、发一条 [`CardMessage::Refused`]，不退回 CPU 判器（用户 2026-09-29 定：配了 GPU 就一定要 GPU 判，做不到就报错）。
+/// 建判器或判一块时 wgpu 报 Out of Memory（[`on_the_card_catching_out_of_memory`]），叫所有卡停下、发一条 [`CardMessage::OutOfMemory`]，
+/// 那一块不交去写库。起过卡池的进程，判器建好那一刻核账：草稿区那一笔借据还在不在（第 10t 批代码轮判决 K4-1）。
 #[cfg(feature = "gpu")]
 fn judge_on_one_card_and_start_the_next<'scope, 'work>(
     scope: &'scope std::thread::Scope<'scope, 'work>,
@@ -1972,11 +2014,28 @@ fn judge_on_one_card_and_start_the_next<'scope, 'work>(
     use std::sync::atomic::Ordering;
     let card: &crate::gpu_unit_checks::GpuCard = &work.cards[position];
     let build_started = Instant::now();
-    let judge = GpuCrashJudge::on_card_with_packed_tables(
-        card,
-        work.tables,
-        usize::try_from(work.states_placed[position]).expect("一次派活的状态数装得进 usize"),
-    );
+    let judge =
+        match on_the_card_catching_out_of_memory(card, "建判器（表与草稿区）时", || {
+            GpuCrashJudge::on_card_with_packed_tables(
+                card,
+                work.tables,
+                usize::try_from(work.states_placed[position])
+                    .expect("一次派活的状态数装得进 usize"),
+            )
+        }) {
+            Ok(judge) => judge,
+            Err(reason) => {
+                work.stop.store(true, Ordering::SeqCst);
+                // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
+                let _ = sender.send(CardMessage::OutOfMemory(reason));
+                return;
+            }
+        };
+    let scratch_lease_on_the_books_when_the_judge_was_built =
+        work.scratch_lease_identifiers[position].map(|lease_identifier| {
+            the_card_pool_this_run_takes_its_cards_from()
+                .is_the_lease_on_the_books(card.index, lease_identifier)
+        });
     let build_seconds = build_started.elapsed().as_secs_f64();
     eprintln!(
         "CRASH_AMPLIFICATION_GPU_JUDGE card={:?} index={} memory_budget_mebibytes={} states_per_dispatch={} scratch_mebibytes={} build_seconds={build_seconds:.1}",
@@ -2005,6 +2064,7 @@ fn judge_on_one_card_and_start_the_next<'scope, 'work>(
         scratch_mebibytes: u64::try_from(judge.scratch_mebibytes()).expect("装得进 u64"),
         build_seconds,
         scratch_booked_mebibytes: work.scratch_booked_mebibytes[position],
+        scratch_lease_on_the_books_when_the_judge_was_built,
         ..GpuCardOutcome::default()
     };
     while !work.must_stop() {
@@ -2015,7 +2075,16 @@ fn judge_on_one_card_and_start_the_next<'scope, 'work>(
         let (point_index, local) = &work.pending[pending_index];
         let point = &work.plan.points[*point_index];
         let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
-        let words = judge.judge_states(ordinals.clone());
+        let words = match on_the_card_catching_out_of_memory(card, "判一块时", || {
+            judge.judge_states(ordinals.clone())
+        }) {
+            Ok(words) => words,
+            Err(reason) => {
+                work.stop.store(true, Ordering::SeqCst);
+                let _ = sender.send(CardMessage::OutOfMemory(reason));
+                return;
+            }
+        };
         let decoding_started = Instant::now();
         if let Some(offset) = first_state_the_gpu_could_not_judge(&words) {
             let state_words = &words[offset];
@@ -2093,7 +2162,29 @@ fn judge_on_one_card_and_start_the_next<'scope, 'work>(
     let _ = sender.send(CardMessage::Finished(outcome));
 }
 
-/// 每张卡按它的显存额度一次派活带得了几个状态（表上卡之后剩下的显存装草稿区），按 `cards` 的次序。
+/// 一张卡照额度 `memory_budget_mebibytes`（MiB）一次派活带得了几个状态（表上卡之后剩下的显存装草稿区）：表比存储缓冲绑定上限
+/// `max_storage_buffer_binding_size` 大、或额度装不下表与驱动那一份，是 0。
+#[cfg(feature = "gpu")]
+fn states_in_flight_within_the_budget(
+    max_storage_buffer_binding_size: u64,
+    memory_budget_mebibytes: u64,
+    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
+) -> u64 {
+    if tables.mebibytes() * 1024 * 1024 > max_storage_buffer_binding_size
+        || memory_budget_mebibytes
+            < tables.mebibytes() + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER
+    {
+        return 0;
+    }
+    u64::try_from(crate::crash_judge_dispatch::states_in_flight_within(
+        max_storage_buffer_binding_size,
+        memory_budget_mebibytes,
+        tables,
+    ))
+    .expect("装得进 u64")
+}
+
+/// 每张卡按它的显存额度一次派活带得了几个状态（[`states_in_flight_within_the_budget`]），按 `cards` 的次序。
 #[cfg(feature = "gpu")]
 fn states_in_flight_of_each_card(
     cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
@@ -2102,24 +2193,103 @@ fn states_in_flight_of_each_card(
     cards
         .iter()
         .map(|card| {
-            let limit = card.device().limits().max_storage_buffer_binding_size;
-            if tables.mebibytes() * 1024 * 1024 > limit
-                || card.memory_budget_mebibytes()
-                    < tables.mebibytes()
-                        + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER
-            {
-                return 0;
-            }
-            u64::try_from(crate::crash_judge_dispatch::states_in_flight_within(
-                limit,
+            states_in_flight_within_the_budget(
+                card.device().limits().max_storage_buffer_binding_size,
                 card.memory_budget_mebibytes(),
                 tables,
-            ))
-            .expect("装得进 u64")
+            )
         })
         .collect()
 }
 
+/// 这一趟本机的一张卡（按优先级）：序号、名字、存储缓冲绑定上限、给判器的额度（MiB），与照额度一次派活带得了几个状态。
+/// 起过卡池的，额度是这一刻这张卡能借给这个进程的加驱动那一份（借来的卡就是这个额度），卡上还没起设备
+/// （[`cards_of_the_pool_with_what_each_lends_now`]）；这一趟自己起的卡照卡的额度。
+#[cfg(feature = "gpu")]
+struct CardCapacityOfThisRun {
+    index: u32,
+    name: String,
+    max_storage_buffer_binding_size: u64,
+    memory_budget_mebibytes: u64,
+    states_in_flight_capacity: u64,
+}
+
+/// 起过卡池的这一趟本机每张卡（池里每张，按池的优先级，不起设备）：额度是这一刻这张卡能借给这个进程的加驱动那一份，照它算一次派活
+/// 带得了几个状态，再按判器按状态数借的量（`crate::crash_judge_dispatch::mebibytes_a_judge_needs`）压到能借的之内（装上的状态才借得到）。
+/// 不照起池那一刻的空闲（第 10t 批代码轮攻方 S5）。
+///
+/// # Errors
+/// 卡上的显存读不出。
+#[cfg(feature = "gpu")]
+fn cards_of_the_pool_with_what_each_lends_now(
+    pool: &crate::gpu_unit_checks::gpu_test_start::GpuCardPool,
+    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
+) -> Result<Vec<CardCapacityOfThisRun>, String> {
+    let cards = pool.cards_for_the_product_path();
+    let card_indices: Vec<u32> = cards.iter().map(|card| card.index).collect();
+    let rooms_mebibytes = pool.card_rooms_mebibytes_now_of(&card_indices)?;
+    Ok(cards
+        .into_iter()
+        .zip(rooms_mebibytes)
+        .map(|(card, room_mebibytes)| {
+            let memory_budget_mebibytes =
+                room_mebibytes + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
+            let states_by_the_budget = states_in_flight_within_the_budget(
+                card.max_storage_buffer_binding_size,
+                memory_budget_mebibytes,
+                tables,
+            );
+            let states_in_flight_capacity =
+                u64::try_from(most_states_whose_need_fits_the_capacity(
+                    usize::try_from(states_by_the_budget).unwrap_or(usize::MAX),
+                    room_mebibytes,
+                    &|states| crate::crash_judge_dispatch::mebibytes_a_judge_needs(tables, states),
+                ))
+                .expect("装得进 u64");
+            CardCapacityOfThisRun {
+                index: card.index,
+                name: card.name,
+                max_storage_buffer_binding_size: card.max_storage_buffer_binding_size,
+                memory_budget_mebibytes,
+                states_in_flight_capacity,
+            }
+        })
+        .collect())
+}
+
+/// 这一趟本机每张卡（按优先级）：起过卡池的照池（[`cards_of_the_pool_with_what_each_lends_now`]，`cards` 是空的），
+/// 这一趟自己起的照 `cards`（[`states_in_flight_of_each_card`]）。
+///
+/// # Errors
+/// 起过卡池的进程读不出卡上的显存。
+#[cfg(feature = "gpu")]
+fn card_capacities_of_this_run(
+    cards_taken_from: GpuCardsSource,
+    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
+    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
+) -> Result<Vec<CardCapacityOfThisRun>, String> {
+    match cards_taken_from {
+        GpuCardsSource::TheCardPoolOfThisProcess => cards_of_the_pool_with_what_each_lends_now(
+            the_card_pool_this_run_takes_its_cards_from(),
+            tables,
+        ),
+        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => Ok(cards
+            .iter()
+            .zip(states_in_flight_of_each_card(cards, tables))
+            .map(|(card, states_in_flight_capacity)| CardCapacityOfThisRun {
+                index: card.index,
+                name: card.name.clone(),
+                max_storage_buffer_binding_size: card
+                    .device()
+                    .limits()
+                    .max_storage_buffer_binding_size,
+                memory_budget_mebibytes: card.memory_budget_mebibytes(),
+                states_in_flight_capacity,
+            })
+            .collect()),
+    }
+}
+
 /// 第 ① 段在卡上。卡按 `cards` 的次序（优先级）用：要判的状态先装第一张卡，装到它的显存额度带得了的状态数为止，
 /// 装不下的才给下一张（[`place_states_on_cards_by_priority`]）；用到的卡一张一张备（表上卡、编内核），备好就从同一队里领块。
 /// 判完的块按块序一块一块写库（后判完的先到就等前面的）：中途被停，写进库的块下一次不再判；
@@ -2128,7 +2298,8 @@ fn states_in_flight_of_each_card(
 /// 每领一块之前先问 `stop_is_requested`：要停就不再领，已经领的块判完、写进库再收尾（领的块按块序连着，库里不留空洞）。
 ///
 /// # Errors
-/// 库读写失败；建不出表；没有一张卡装得下这条流的表与一个状态的草稿区；有状态 GPU 判不了。
+/// 库读写失败；建不出表；没有一张卡装得下这条流的表与一个状态的草稿区；有状态 GPU 判不了；卡上 wgpu 报了 Out of Memory
+/// （「本次未跑」那一行已经打了，报错之前判完的块在库里）。
 ///
 /// # Panics
 /// 卡上的线程 panic（内核没把状态判完、编内核之前量得内核超了预算）。
@@ -2153,9 +2324,8 @@ pub fn judge_my_blocks_on_gpu_cards(
         stop_is_requested,
     )? {
         GpuJudgingOfMyBlocks::Judged { tally, outcome } => Ok((tally, outcome)),
-        GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime => unreachable!(
-            "卡判成调用方自己起的（DevicesStartedForThisRun）就不借草稿区（scratch_states_to_book_on_each_card 交 NotBooked），等不到账这一形走不到"
-        ),
+        // 卡判成调用方自己起的就不借草稿区，走到这里只能是卡上 wgpu 报了 Out of Memory
+        GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { reason, .. } => Err(reason),
     }
 }
 
@@ -2172,8 +2342,12 @@ enum GpuJudgingOfMyBlocks {
         tally: CheckingTally,
         outcome: GpuJudgeOutcome,
     },
-    /// 起过卡池的进程在账上借草稿区等满了：「本次未跑」那一行已经打了，一块都没判（调用方照叫停的样子交回）。
-    NotRunTheScratchWasNotBookedInTime,
+    /// 显存不够：起过卡池的进程此刻在账上借不到草稿区（一块都没判），或卡上 wgpu 报了 Out of Memory（之前判完的块在库里，
+    /// `tally` 是它们）。「本次未跑」那一行已经打了（`reason` 是它的原因），调用方照叫停的样子交回。
+    NotRunTheGpuMemoryWasShort {
+        tally: CheckingTally,
+        reason: String,
+    },
 }
 
 /// 「本次未跑」那一行里产品路径的名字（GPU 用例写用例名的那一格）。
@@ -2192,73 +2366,46 @@ fn the_card_pool_this_run_takes_its_cards_from(
     )
 }
 
-/// 起过卡池的这一趟每张卡装得下几个状态：照卡的额度算的（[`states_in_flight_of_each_card`]）再按这张卡的账此刻能借给这个进程的量压
-/// （判器按装的状态数借 `crate::crash_judge_dispatch::mebibytes_a_judge_needs`，借的量不越过账的容量，装上的状态才借得到）。
-///
-/// # Errors
-/// 账本读写不了。
-///
-/// # Panics
-/// 这一趟有一张卡不是池里的卡（[`gpu_cards_in_use`] 照池交回登记上的那几张）。
-#[cfg(feature = "gpu")]
-fn states_each_card_of_the_pool_lends(
-    pool: &crate::gpu_unit_checks::gpu_test_start::GpuCardPool,
-    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
-    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
-) -> Result<Vec<u64>, String> {
-    let card_indices: Vec<u32> = cards.iter().map(|card| card.index).collect();
-    let ledger_capacities = pool.card_capacities_mebibytes_of(&card_indices)?;
-    Ok(states_in_flight_of_each_card(cards, tables)
-        .into_iter()
-        .zip(ledger_capacities)
-        .map(|(states_by_the_card_budget, ledger_capacity_mebibytes)| {
-            let most_states = usize::try_from(states_by_the_card_budget).unwrap_or(usize::MAX);
-            u64::try_from(most_states_whose_need_fits_the_capacity(
-                most_states,
-                ledger_capacity_mebibytes,
-                &|states| crate::crash_judge_dispatch::mebibytes_a_judge_needs(tables, states),
-            ))
-            .expect("装得进 u64")
-        })
-        .collect())
-}
-
-/// 起过卡池的产品路径在显存账上借不到时怎么报（`what_was_booked` 说借的是什么）：等满了照对拍用例那一路打「本次未跑」那一行
-/// （[`scratch_booking_refusal_outcome`]，同一个记号）、交 `Ok(())`，调用方照叫停的样子交回；要的比每张卡的账都大、账本读写不了判红。
+/// 起过卡池的产品路径在显存账上借不到时怎么报（`what_was_booked` 说借的是什么）：此刻借不到照对拍用例那一路打「本次未跑」那一行
+/// （[`scratch_booking_refusal_outcome`]，同一个记号）、交 `Ok(那一行)`，调用方照叫停的样子交回；要的比每张卡空着时能借的都大、
+/// 卡上的显存读不出判红。
 ///
 /// # Errors
-/// 要的比每张卡的账都大（说法与没有一张卡装得下这条流的表那一句同一类，分项驱动照它认成设备用不上）、账本读写不了。
+/// 要的比每张卡空着时能借的都大（说法与没有一张卡装得下这条流的表那一句同一类，分项驱动照它认成设备用不上）、卡上的显存读不出。
 #[cfg(feature = "gpu")]
 fn print_the_not_run_line_or_refuse(
     what_was_booked: &str,
     refusal: &BookingRefusal,
-) -> Result<(), String> {
+) -> Result<String, String> {
     if let BookingRefusal::NeedLargerThanEveryCard(NeedLargerThanEveryCard {
         need_mebibytes,
-        largest_capacity_mebibytes,
+        largest_ceiling_mebibytes,
     }) = refusal
     {
         return Err(format!(
-            "没有一张卡装得下这条流的表与{what_was_booked}：要借 {need_mebibytes} MiB，池里最大的一张卡的显存账只有 {largest_capacity_mebibytes} MiB。\
+            "没有一张卡装得下这条流的表与{what_was_booked}：要借 {need_mebibytes} MiB，池里最大的一张卡空着时也只借得出 {largest_ceiling_mebibytes} MiB。\
              看配置里每张卡的显存额度与卡上此刻的空闲显存"
         ));
     }
     match scratch_booking_refusal_outcome(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, 0, refusal) {
         ScratchBookingRefusalOutcome::NotRun { not_run_line } => {
             println!("{not_run_line}");
-            Ok(())
+            Ok(not_run_line)
         }
         ScratchBookingRefusalOutcome::Red { reason } => Err(format!("{what_was_booked}：{reason}")),
     }
 }
 
-/// 第 ① 段每张卡上的判器用什么：借了草稿区的那几张换成借来的卡（额度是借的量加驱动那一份，判器照它开出来的草稿区正好是借的状态数），
-/// 每张卡装几个状态（一次派活带几个；0 的这一趟不用），借了多少；按这一趟卡的次序。
+/// 第 ① 段每张卡上的判器用什么（只列装了状态的卡，按这一趟卡的次序）：借了草稿区的是借来的卡（额度是借的量加驱动那一份，
+/// 判器照它开出来的草稿区正好是借的状态数），每张卡装几个状态（一次派活带几个，都大于 0），借了多少、借据号；
+/// 连同这一趟本机每张卡（与 [`CardCapacityOfThisRun`] 的表逐张对上）各装了几个。
 #[cfg(feature = "gpu")]
 struct ScratchOfTheCards {
     cards_for_the_judges: Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>>,
     states_placed: Vec<u64>,
     scratch_booked_mebibytes: Vec<Option<u64>>,
+    scratch_lease_identifiers: Vec<Option<u64>>,
+    states_placed_on_each_card_of_this_run: Vec<u64>,
     /// 借据（不进账时是空的），判器都丢掉之后才还。
     leases: Vec<crate::gpu_unit_checks::gpu_test_start::GpuCardLease>,
 }
@@ -2267,22 +2414,26 @@ struct ScratchOfTheCards {
 #[cfg(feature = "gpu")]
 enum ScratchOfEachCard {
     Ready(ScratchOfTheCards),
-    /// 一张卡都借不到、等满了：「本次未跑」那一行已经打了，什么都没借着。
-    NotRunTheScratchWasNotBookedInTime,
+    /// 一张卡都借不到：「本次未跑」那一行已经打了（`not_run_line`），什么都没借着。
+    NotRunTheGpuMemoryWasShort {
+        not_run_line: String,
+    },
 }
 
-/// 照卡的来路给第 ① 段装状态、借草稿区（[`scratch_states_to_book_on_each_card`]）。`states_each_card_holds` 是每张卡最多装几个
-/// （起过卡池的已照账能借的量压过，[`states_each_card_of_the_pool_lends`]）。起过卡池的：按卡的先后把 `pending_states` 个状态装到此刻借得到的卡上，
-/// 每张在账本锁里照此刻的估计再压一次再借（显存按 `crate::crash_judge_dispatch::mebibytes_a_judge_needs`，与 GPU 对拍用例借卡同一份），
-/// 借不到的卡不装、不等，一张都借不到才等；等满了打「本次未跑」那一行（[`print_the_not_run_line_or_refuse`]）。
-/// 没起过卡池的不借、卡照原样，照 [`place_states_on_cards_by_priority`] 装。
+/// 照卡的来路给第 ① 段装状态、借草稿区（[`scratch_states_to_book_on_each_card`]）。`run_cards` 是这一趟本机每张卡
+/// （[`card_capacities_of_this_run`]），`states_each_card_holds` 是每张卡最多装几个（起过卡池的已照这一刻能借的压过）。
+/// 起过卡池的：按卡的先后把 `pending_states` 个状态装到此刻借得到的卡上，每张在账的锁里现读、照此刻能借的再压一次再借
+/// （显存按 `crate::crash_judge_dispatch::mebibytes_a_judge_needs`，与 GPU 对拍用例借卡同一份），借不到的卡不装；
+/// 一张都借不到就打「本次未跑」那一行（[`print_the_not_run_line_or_refuse`]）。没起过卡池的不借、卡照原样（`cards`，与 `run_cards` 逐张对上），
+/// 照 [`place_states_on_cards_by_priority`] 装。
 ///
 /// # Errors
-/// 每张卡的账连一个状态都放不下、账本读写不了（判红）。
+/// 每张卡空着时连一个状态都放不下、卡上的显存读不出（判红）。
 #[cfg(feature = "gpu")]
 fn scratch_of_each_card_booked_by_where_the_cards_came_from(
     cards_taken_from: GpuCardsSource,
     cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
+    run_cards: &[CardCapacityOfThisRun],
     tables: &crate::crash_judge_dispatch::PackedJudgeTables,
     state_count: u64,
     pending_states: u64,
@@ -2290,23 +2441,35 @@ fn scratch_of_each_card_booked_by_where_the_cards_came_from(
 ) -> Result<ScratchOfEachCard, String> {
     match scratch_states_to_book_on_each_card(cards_taken_from, states_each_card_holds, state_count)
     {
-        ScratchStatesToBook::NotBooked => Ok(ScratchOfEachCard::Ready(ScratchOfTheCards {
-            cards_for_the_judges: cards.to_vec(),
-            states_placed: place_states_on_cards_by_priority(
-                pending_states,
-                states_each_card_holds,
-            ),
-            scratch_booked_mebibytes: vec![None; cards.len()],
-            leases: Vec::new(),
-        })),
+        ScratchStatesToBook::NotBooked => {
+            let states_placed_on_each_card_of_this_run =
+                place_states_on_cards_by_priority(pending_states, states_each_card_holds);
+            let placed: Vec<(&std::sync::Arc<crate::gpu_unit_checks::GpuCard>, u64)> = cards
+                .iter()
+                .zip(&states_placed_on_each_card_of_this_run)
+                .filter(|(_, states_placed)| **states_placed > 0)
+                .map(|(card, states_placed)| (card, *states_placed))
+                .collect();
+            Ok(ScratchOfEachCard::Ready(ScratchOfTheCards {
+                cards_for_the_judges: placed.iter().map(|(card, _)| (*card).clone()).collect(),
+                states_placed: placed
+                    .iter()
+                    .map(|(_, states_placed)| *states_placed)
+                    .collect(),
+                scratch_booked_mebibytes: vec![None; placed.len()],
+                scratch_lease_identifiers: vec![None; placed.len()],
+                states_placed_on_each_card_of_this_run,
+                leases: Vec::new(),
+            }))
+        }
         ScratchStatesToBook::OnEachCard(states_each_card_lends) => {
-            let card_indices: Vec<u32> = cards.iter().map(|card| card.index).collect();
+            let card_indices: Vec<u32> = run_cards.iter().map(|card| card.index).collect();
             let states_each_card_lends: Vec<u64> = states_each_card_lends
                 .into_iter()
                 .map(|states| u64::try_from(states).expect("状态数装得进 u64"))
                 .collect();
             let booked = match the_card_pool_this_run_takes_its_cards_from()
-                .book_the_scratch_of_the_cards_waiting_only_when_none_has_room(
+                .book_the_scratch_of_the_cards_with_room_now(
                     PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                     &card_indices,
                     pending_states,
@@ -2320,14 +2483,17 @@ fn scratch_of_each_card_booked_by_where_the_cards_came_from(
                 ) {
                 Ok(booked) => booked,
                 Err(refusal) => {
-                    print_the_not_run_line_or_refuse("第 ① 段判器的草稿区", &refusal)?;
-                    return Ok(ScratchOfEachCard::NotRunTheScratchWasNotBookedInTime);
+                    let not_run_line =
+                        print_the_not_run_line_or_refuse("第 ① 段判器的草稿区", &refusal)?;
+                    return Ok(ScratchOfEachCard::NotRunTheGpuMemoryWasShort { not_run_line });
                 }
             };
             let mut scratch = ScratchOfTheCards {
-                cards_for_the_judges: cards.to_vec(),
-                states_placed: vec![0; cards.len()],
-                scratch_booked_mebibytes: vec![None; cards.len()],
+                cards_for_the_judges: Vec::new(),
+                states_placed: Vec::new(),
+                scratch_booked_mebibytes: Vec::new(),
+                scratch_lease_identifiers: Vec::new(),
+                states_placed_on_each_card_of_this_run: vec![0; run_cards.len()],
                 leases: Vec::new(),
             };
             for (card_slot, booked_on_the_card) in booked.into_iter().enumerate() {
@@ -2336,10 +2502,17 @@ fn scratch_of_each_card_booked_by_where_the_cards_came_from(
                     states,
                 }) = booked_on_the_card
                 {
-                    scratch.cards_for_the_judges[card_slot] =
-                        std::sync::Arc::new(lease.card().clone());
-                    scratch.states_placed[card_slot] = states;
-                    scratch.scratch_booked_mebibytes[card_slot] = Some(lease.mebibytes());
+                    scratch
+                        .cards_for_the_judges
+                        .push(std::sync::Arc::new(lease.card().clone()));
+                    scratch.states_placed.push(states);
+                    scratch
+                        .scratch_booked_mebibytes
+                        .push(Some(lease.mebibytes()));
+                    scratch
+                        .scratch_lease_identifiers
+                        .push(Some(lease.lease_identifier()));
+                    scratch.states_placed_on_each_card_of_this_run[card_slot] = states;
                     scratch.leases.push(lease);
                 }
             }
@@ -2348,13 +2521,14 @@ fn scratch_of_each_card_booked_by_where_the_cards_came_from(
     }
 }
 
-/// 同 [`judge_my_blocks_on_gpu_cards`]，卡的来路另给：起过卡池的（[`GpuCardsSource::TheCardPoolOfThisProcess`]），每张卡装几个状态
-/// 先照账能借的量压（[`states_each_card_of_the_pool_lends`]），判之前把要判的状态装到此刻借得到草稿区的卡上
-/// （[`scratch_of_each_card_booked_by_where_the_cards_came_from`]：借不到的卡这一趟不用、不等它，一张都借不到才等），等满了借不到交
-/// [`GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime`]、一块都不判；判完先丢判器、再还借据。
+/// 同 [`judge_my_blocks_on_gpu_cards`]，卡的来路另给：起过卡池的（[`GpuCardsSource::TheCardPoolOfThisProcess`]，`cards` 是空的），
+/// 本机每张卡装几个状态照这一刻能借的量压（[`cards_of_the_pool_with_what_each_lends_now`]），判之前把要判的状态装到此刻借得到草稿区的卡上
+/// （[`scratch_of_each_card_booked_by_where_the_cards_came_from`]：借不到的卡这一趟不用，一张都借不到交
+/// [`GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort`]、一块都不判）；判完先丢判器、再还借据。卡上 wgpu 报了 Out of Memory
+/// 同样交 [`GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort`]（打「本次未跑」那一行，之前判完的块在库里）。
 ///
 /// # Errors
-/// 同 [`judge_my_blocks_on_gpu_cards`]；借草稿区时每张卡的账连一个状态都放不下、账本读写不了。
+/// 同 [`judge_my_blocks_on_gpu_cards`]（Out of Memory 那一形除外）；借草稿区时每张卡空着时连一个状态都放不下、卡上的显存读不出。
 ///
 /// # Panics
 /// 同 [`judge_my_blocks_on_gpu_cards`]。
@@ -2413,16 +2587,11 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
         .iter()
         .map(|(_, local)| local.end - local.start)
         .sum();
-    let capacities = match cards_taken_from {
-        GpuCardsSource::TheCardPoolOfThisProcess => states_each_card_of_the_pool_lends(
-            the_card_pool_this_run_takes_its_cards_from(),
-            cards,
-            &tables,
-        )?,
-        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
-            states_in_flight_of_each_card(cards, &tables)
-        }
-    };
+    let run_cards = card_capacities_of_this_run(cards_taken_from, cards, &tables)?;
+    let capacities: Vec<u64> = run_cards
+        .iter()
+        .map(|card| card.states_in_flight_capacity)
+        .collect();
     // 借据活到这个函数收尾：几张卡上的判器在下面的线程里判完就丢，丢完才还
     let scratch = if capacities.iter().all(|capacity| *capacity == 0) {
         None
@@ -2430,33 +2599,39 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
         match scratch_of_each_card_booked_by_where_the_cards_came_from(
             cards_taken_from,
             cards,
+            &run_cards,
             &tables,
             state_count,
             pending_states,
             &capacities,
         )? {
             ScratchOfEachCard::Ready(scratch) => Some(scratch),
-            ScratchOfEachCard::NotRunTheScratchWasNotBookedInTime => {
-                return Ok(GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime);
+            ScratchOfEachCard::NotRunTheGpuMemoryWasShort { not_run_line } => {
+                return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
+                    tally,
+                    reason: not_run_line,
+                });
             }
         }
     };
-    let states_placed = scratch.as_ref().map_or_else(
-        || vec![0; cards.len()],
-        |scratch| scratch.states_placed.clone(),
+    let states_placed_on_each_card_of_this_run = scratch.as_ref().map_or_else(
+        || vec![0; run_cards.len()],
+        |scratch| scratch.states_placed_on_each_card_of_this_run.clone(),
     );
-    for ((card, capacity), placed) in cards.iter().zip(&capacities).zip(&states_placed) {
+    for ((card, capacity), placed) in run_cards
+        .iter()
+        .zip(&capacities)
+        .zip(&states_placed_on_each_card_of_this_run)
+    {
         eprintln!(
             "CRASH_AMPLIFICATION_GPU_PLACEMENT card={:?} index={} memory_budget_mebibytes={} states_in_flight_capacity={capacity} states_placed={placed} pending_states={pending_states} tables_mebibytes={}",
             card.name,
             card.index,
-            card.memory_budget_mebibytes(),
+            card.memory_budget_mebibytes,
             tables.mebibytes()
         );
     }
-    let (Some(scratch), Some(first_position)) =
-        (scratch, states_placed.iter().position(|placed| *placed > 0))
-    else {
+    let Some(scratch) = scratch.filter(|scratch| !scratch.cards_for_the_judges.is_empty()) else {
         return Err(format!(
             "没有一张卡装得下这条流的表（{} MiB）与一个状态的草稿区：看配置里每张卡的显存额度与卡上此刻的空闲显存",
             tables.mebibytes()
@@ -2467,8 +2642,9 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
         pending: &pending,
         tables: &tables,
         cards: &scratch.cards_for_the_judges,
-        states_placed: &states_placed,
+        states_placed: &scratch.states_placed,
         scratch_booked_mebibytes: &scratch.scratch_booked_mebibytes,
+        scratch_lease_identifiers: &scratch.scratch_lease_identifiers,
         next_pending: std::sync::atomic::AtomicUsize::new(0),
         stop: std::sync::atomic::AtomicBool::new(false),
         stop_is_requested,
@@ -2478,7 +2654,7 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
         let (sender, receiver) = std::sync::mpsc::channel::<CardMessage>();
         let work = &work;
         scope.spawn(move || {
-            judge_on_one_card_and_start_the_next(scope, work, first_position, sender);
+            judge_on_one_card_and_start_the_next(scope, work, 0, sender);
         });
         // 卡上判一块、编内核都可能比进度间隔久：等着收的这条线程隔一个间隔打一行按挂钟的
         let clock = || started.elapsed();
@@ -2514,7 +2690,10 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
             },
             &mut |line| eprintln!("{line}"),
         );
-        if received.storing_error.is_none() && received.refusal.is_none() {
+        if received.storing_error.is_none()
+            && received.refusal.is_none()
+            && received.out_of_memory.is_none()
+        {
             // 领走的块（序号连着、从 0 起）每一块都写进了库；没被叫停时领走的就是全部
             let taken = work
                 .next_pending
@@ -2536,8 +2715,15 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
     if let Some(reason) = received.refusal {
         return Err(reason);
     }
+    if let Some(reason) = received.out_of_memory {
+        println!(
+            "{}",
+            gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
+        );
+        return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason });
+    }
     outcome.cards.sort_by_key(|card| {
-        cards
+        run_cards
             .iter()
             .position(|candidate| candidate.index == card.index)
     });
@@ -3509,6 +3695,7 @@ mod tests {
                     states_in_the_widest_dispatch: 0,
                     seconds_of_the_longest_dispatch: 0.0,
                     scratch_booked_mebibytes: None,
+                    scratch_lease_on_the_books_when_the_judge_was_built: None,
                 };
                 card_sender
                     .send(CardMessage::Finished(finished_card))
@@ -3551,6 +3738,55 @@ mod tests {
         assert_eq!(received.refusal, None);
     }
 
+    /// 第 10x 批：卡上 wgpu 报了 Out of Memory（[`CardMessage::OutOfMemory`]，文件头第 5 条）记成显存不够、留第一句，不记成拒判
+    /// （拒判整趟判红，显存不够这一趟报本次未跑）；之前判完的块照写进库。不要卡：卡上的线程换成直接发这几句话。
+    #[cfg(feature = "gpu")]
+    #[test]
+    fn an_out_of_memory_on_a_card_is_kept_apart_from_a_refusal_and_the_blocks_judged_before_it_are_stored(
+    ) {
+        let (card_sender, card_receiver) = std::sync::mpsc::channel::<CardMessage>();
+        card_sender
+            .send(CardMessage::Block(GpuJudgedBlock {
+                pending_index: 0,
+                items_of_each_state: vec![Vec::new()],
+            }))
+            .expect("收的那一头还在");
+        for reason in ["第一张卡报的那一句", "第二张卡报的那一句"] {
+            card_sender
+                .send(CardMessage::OutOfMemory(reason.to_string()))
+                .expect("收的那一头还在");
+        }
+        drop(card_sender);
+        let mut wall_clock_lines =
+            WallClockProgressLines::every_progress_line_interval(std::time::Duration::ZERO);
+        let mut blocks_handed_to_the_store = 0usize;
+        let received = receive_the_cards_messages_and_store_the_blocks_in_order(
+            &card_receiver,
+            &mut wall_clock_lines,
+            &|| std::time::Duration::ZERO,
+            2,
+            &mut |_block| {
+                blocks_handed_to_the_store += 1;
+                Ok(())
+            },
+            &mut |_line| {},
+        );
+        assert_eq!(
+            received.out_of_memory.as_deref(),
+            Some("第一张卡报的那一句"),
+            "显存不够留第一句"
+        );
+        assert_eq!(
+            received.refusal, None,
+            "Out of Memory 不记成拒判：拒判整趟判红，显存不够报本次未跑"
+        );
+        assert_eq!(
+            (received.blocks_stored, blocks_handed_to_the_store),
+            (1, 1),
+            "Out of Memory 之前判完的块 0 照写进库"
+        );
+    }
+
     /// K5：这份文件里每一处按名字读环境变量（`env::var("…")`、`env::var_os("…")`，测试模块之外），名字都在
     /// [`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`] 里：用例的环境守卫照这张表清、还原，漏一个就会读到调用方留下的值。
     #[test]
@@ -4004,9 +4240,10 @@ pub struct EnvironmentRun {
     pub gpu_judge: Option<GpuJudgeOutcome>,
     /// 这一趟只排了派活计划（`SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1`）：录入与导入照做，一块都没判。
     pub placement_only: bool,
-    /// 这一趟被叫停了（[`stop_is_requested_by_the_environment`]），或起过卡池的进程在显存账上借草稿区等满了、打了「本次未跑」那一行
-    /// （文件头第 5 条）：判过的块在库里，第 ②③ 段没做，对账不为空是正常的；分项驱动照它把这一项记成本次未跑。
-    /// 只排派活计划的那一趟借试判的草稿区等满了也置上（`placement_only` 同时为真，那一趟没打 `CRASH_AMPLIFICATION_PLACEMENT` 行）。
+    /// 这一趟被叫停了（[`stop_is_requested_by_the_environment`]），或显存不够（起过卡池的进程在显存账上此刻借不到、卡上 wgpu 报了
+    /// Out of Memory）、打了「本次未跑」那一行（文件头第 5 条）：判过的块在库里，没做完的段留给下一趟，对账不为空是正常的；
+    /// 分项驱动照它把这一项记成本次未跑。只排派活计划的那一趟借不到试判的草稿区也置上（`placement_only` 同时为真，那一趟没打
+    /// `CRASH_AMPLIFICATION_PLACEMENT` 行）。
     pub stopped_early: bool,
     /// 这一趟判定行上的判法版本。
     pub judge_version: JudgeVersion,
@@ -4027,6 +4264,10 @@ pub struct EnvironmentRun {
     pub unit_checksum_buffers_booked_mebibytes: Option<u64>,
     /// 起过卡池的进程第 ② 段在卡上核时，在显存账上为核对的缓冲借的（MiB，文件头第 5 条）；没借是 `None`。
     pub facts_verifier_buffers_booked_mebibytes: Option<u64>,
+    /// 起过卡池的进程单元校验和对拍完那一刻核账：对拍的缓冲那一笔借据还在账上没有（第 10t 批代码轮判决 K4-1）；没借是 `None`。
+    pub unit_checksum_buffers_lease_on_the_books_after_checking: Option<bool>,
+    /// 起过卡池的进程第 ② 段核完那一刻核账：核对的缓冲那一笔借据还在账上没有（同上）；没借是 `None`。
+    pub facts_verifier_buffers_lease_on_the_books_after_verifying: Option<bool>,
 }
 
 /// 一趟用的卡从哪来（定义在 harness 档，草稿区记不记账的判法 `singlefs_harness::gpu_memory_ledger::scratch_states_to_book_on_each_card`
@@ -4438,26 +4679,17 @@ pub fn run_from_environment(
         }
     }
     let seconds_planning_and_recording = planning_started.elapsed().as_secs_f64();
-    // 起过卡池的进程在池里一张卡都登记不上、等满了：「本次未跑」那一行已经打了，下面照叫停的样子交回（文件头第 5 条）
-    let (cards, gpu_cards_taken_from, no_card_of_the_pool_was_registered_in_time) =
-        if routes.need_a_card() {
-            match gpu_cards_in_use(cards_by_priority.as_deref(), host)? {
-                GpuCardsOfThisRun::Taken { cards, taken_from } => (Some(cards), taken_from, false),
-                GpuCardsOfThisRun::NotRunNoCardOfThePoolHadRoomInTime => {
-                    (None, GpuCardsSource::TheCardPoolOfThisProcess, true)
-                }
-            }
-        } else {
-            (None, GpuCardsSource::NoCardNeeded, false)
-        };
+    // 起过卡池的进程取卡不登记、不起设备，借到哪张卡才在哪张上起（文件头第 5 条）
+    let (cards, gpu_cards_taken_from) = if routes.need_a_card() {
+        let GpuCardsOfThisRun { cards, taken_from } =
+            gpu_cards_in_use(cards_by_priority.as_deref(), host)?;
+        (Some(cards), taken_from)
+    } else {
+        (None, GpuCardsSource::NoCardNeeded)
+    };
     if placement_only {
-        let placement = match (
-            routes.crash_judge,
-            &cards,
-            no_card_of_the_pool_was_registered_in_time,
-        ) {
-            (ComputedOn::Gpu, _, true) => PlacementOfThisRun::NotRunTheScratchWasNotBookedInTime,
-            (ComputedOn::Gpu, Some(cards), false) => print_the_placement(
+        let placement = match (routes.crash_judge, &cards) {
+            (ComputedOn::Gpu, Some(cards)) => print_the_placement(
                 &store,
                 flow,
                 &plan,
@@ -4469,7 +4701,7 @@ pub fn run_from_environment(
                 host,
                 states_reused_at_recording,
             )?,
-            (ComputedOn::Gpu, None, false) | (ComputedOn::Cpu, _, _) => {
+            (ComputedOn::Gpu, None) | (ComputedOn::Cpu, _) => {
                 return Err(format!(
                     "只排派活计划排的是 GPU 判器的活：要配置开着 GPU（ENABLE_GPU=1）、{GPU_COMPUTATION_CLASSES_KEY} 里登记 {}",
                     GpuComputationClass::CrashJudgeGpu.name()
@@ -4498,8 +4730,8 @@ pub fn run_from_environment(
             comparison: None,
             gpu_judge: None,
             placement_only: true,
-            // 起过卡池的进程借试判的草稿区等满了：「本次未跑」那一行已经打了，没打 PLACEMENT 行（第 10v 批）
-            stopped_early: placement == PlacementOfThisRun::NotRunTheScratchWasNotBookedInTime,
+            // 起过卡池的进程借不到试判的草稿区：「本次未跑」那一行已经打了，没打 PLACEMENT 行（第 10v 批）
+            stopped_early: placement == PlacementOfThisRun::NotRunTheGpuMemoryWasShort,
             judge_version: version,
             computation_routes: routes,
             expand_up_to,
@@ -4511,79 +4743,81 @@ pub fn run_from_environment(
             gpu_cards_taken_from,
             unit_checksum_buffers_booked_mebibytes: None,
             facts_verifier_buffers_booked_mebibytes: None,
+            unit_checksum_buffers_lease_on_the_books_after_checking: None,
+            facts_verifier_buffers_lease_on_the_books_after_verifying: None,
         });
     }
     // 第 ① 段：登记了 crash_judge_gpu 就由 GPU 判器判这台机器领的块（判不了就报错，不退回 CPU）；没登记由 CPU 判器判，
-    // 那时登记了 gpu_unit_checks 就把单元两道校验和另交一张卡对拍。起过卡池的进程借显存等满了：「本次未跑」那一行已经打了，
-    // 一块都不判，下面照叫停的样子交回（文件头第 5 条）
+    // 那时登记了 gpu_unit_checks 就把单元两道校验和另交一张卡对拍。显存不够（起过卡池的进程此刻借不到、卡上 wgpu 报了 Out of Memory）：
+    // 「本次未跑」那一行已经打了，判过的块在库里，下面照叫停的样子交回（文件头第 5 条）
     let judging_started = std::time::Instant::now();
     let (
         checking,
         gpu_judge,
         unit_checksum_buffers_booked_mebibytes,
-        memory_was_not_booked_in_time,
-    ) = if no_card_of_the_pool_was_registered_in_time {
-        (CheckingTally::default(), None, None, true)
-    } else {
-        match (routes.crash_judge, &cards) {
-            (ComputedOn::Gpu, Some(cards)) => match judge_my_blocks_on_gpu(
-                &mut store,
-                flow,
-                &plan,
-                &|block| selected.contains(&block),
-                &version,
-                cards,
-                gpu_cards_taken_from,
-                &stop_is_requested_by_the_environment,
-            )? {
-                GpuJudgingOfMyBlocks::Judged { tally, outcome } => {
-                    eprintln!("{}", gpu_total_line(&outcome, expand_up_to, exhaustive));
-                    (tally, Some(outcome), None, false)
-                }
-                GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime => {
-                    (CheckingTally::default(), None, None, true)
-                }
-            },
-            (ComputedOn::Gpu, None) => {
-                return Err(format!(
-                    "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回 CPU 判器",
-                    GpuComputationClass::CrashJudgeGpu.name()
-                ));
+        unit_checksum_buffers_lease_on_the_books_after_checking,
+        gpu_memory_was_short,
+    ) = match (routes.crash_judge, &cards) {
+        (ComputedOn::Gpu, Some(cards)) => match judge_my_blocks_on_gpu(
+            &mut store,
+            flow,
+            &plan,
+            &|block| selected.contains(&block),
+            &version,
+            cards,
+            gpu_cards_taken_from,
+            &stop_is_requested_by_the_environment,
+        )? {
+            GpuJudgingOfMyBlocks::Judged { tally, outcome } => {
+                eprintln!("{}", gpu_total_line(&outcome, expand_up_to, exhaustive));
+                (tally, Some(outcome), None, None, false)
             }
-            (ComputedOn::Cpu, _) => match unit_checksums_on(
-                routes.unit_checksums,
-                cards.as_ref(),
-                gpu_cards_taken_from,
-                flow,
-            )? {
-                // 借来的缓冲活到这一段判完
-                UnitChecksumsOfThisRun::Ready {
-                    on,
-                    lease: _buffers_held_until_checked,
-                    buffers_booked_mebibytes,
-                } => (
-                    check_selected_blocks(
-                        &mut store,
-                        flow,
-                        &plan,
-                        &selected,
-                        &on,
-                        &version,
-                        &stop_is_requested_by_the_environment,
-                    )
-                    .map_err(|error| format!("核对失败：{error:?}"))?,
+            GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason: _ } => {
+                (tally, None, None, None, true)
+            }
+        },
+        (ComputedOn::Gpu, None) => {
+            return Err(format!(
+                "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回 CPU 判器",
+                GpuComputationClass::CrashJudgeGpu.name()
+            ));
+        }
+        (ComputedOn::Cpu, _) => match unit_checksums_on(
+            routes.unit_checksums,
+            cards.as_ref(),
+            gpu_cards_taken_from,
+            flow,
+        )? {
+            // 借来的缓冲活到这一段判完；判完那一刻核账（第 10t 批代码轮判决 K4-1）
+            UnitChecksumsOfThisRun::Ready {
+                on,
+                lease: buffers_held_until_checked,
+                buffers_booked_mebibytes,
+            } => {
+                let checking = check_selected_blocks(
+                    &mut store,
+                    flow,
+                    &plan,
+                    &selected,
+                    &on,
+                    &version,
+                    &stop_is_requested_by_the_environment,
+                )
+                .map_err(|error| format!("核对失败：{error:?}"))?;
+                (
+                    checking,
                     None,
                     buffers_booked_mebibytes,
+                    lease_on_the_books_after_its_work(&buffers_held_until_checked),
                     false,
-                ),
-                UnitChecksumsOfThisRun::NotRunTheBuffersWereNotBookedInTime => {
-                    (CheckingTally::default(), None, None, true)
-                }
-            },
-        }
+                )
+            }
+            UnitChecksumsOfThisRun::NotRunTheGpuMemoryWasShort => {
+                (CheckingTally::default(), None, None, None, true)
+            }
+        },
     };
-    let stop_was_requested =
-        memory_was_not_booked_in_time || stop_is_requested_by_the_environment();
+    let stop_was_requested = gpu_memory_was_short || stop_is_requested_by_the_environment();
     let seconds_judging = judging_started.elapsed().as_secs_f64();
     // 第 ② 段：配置开了 GPU 就核，登记了 crash_verify_gpu 在一张卡上核（起过卡池的照账借第一张放得下的，第 ① 段的草稿区已经放掉）、
     // 没登记用 CPU 参照核；没开 GPU 而 SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu 也用 CPU 参照核；被叫停的不核
@@ -4601,7 +4835,8 @@ pub fn run_from_environment(
         verification,
         verifying_cards_used,
         facts_verifier_buffers_booked_mebibytes,
-        facts_were_not_verified_in_time,
+        facts_verifier_buffers_lease_on_the_books_after_verifying,
+        facts_were_not_verified_for_short_gpu_memory,
     ) = match (facts_verifier_route, &cards) {
         (Some(ComputedOn::Gpu), Some(cards)) => match verify_my_blocks_on_the_first_card(
             &mut store,
@@ -4614,8 +4849,15 @@ pub fn run_from_environment(
             FactsVerifiedOnACard::Verified {
                 outcome,
                 buffers_booked_mebibytes,
-            } => (Some(outcome), 1, buffers_booked_mebibytes, false),
-            FactsVerifiedOnACard::NotRunTheBuffersWereNotBookedInTime => (None, 0, None, true),
+                lease_on_the_books_after_verifying,
+            } => (
+                Some(outcome),
+                1,
+                buffers_booked_mebibytes,
+                lease_on_the_books_after_verifying,
+                false,
+            ),
+            FactsVerifiedOnACard::NotRunTheGpuMemoryWasShort => (None, 0, None, None, true),
         },
         (Some(ComputedOn::Gpu), None) => {
             return Err(format!(
@@ -4623,7 +4865,7 @@ pub fn run_from_environment(
                 GpuComputationClass::CrashVerifyGpu.name()
             ));
         }
-        (None, _) => (None, 0, None, false),
+        (None, _) => (None, 0, None, None, false),
         (Some(ComputedOn::Cpu), _) => {
             let on = FactsVerifierOn::Cpu;
             let tally = verify_blocks_from_facts(
@@ -4640,12 +4882,13 @@ pub fn run_from_environment(
                 }),
                 0,
                 None,
+                None,
                 false,
             )
         }
     };
-    if stop_was_requested || facts_were_not_verified_in_time {
-        // 被叫停，或起过卡池的进程借显存等满了（本次未跑）：判过的块已经在库里，没做的段留给接着判的那一趟
+    if stop_was_requested || facts_were_not_verified_for_short_gpu_memory {
+        // 被叫停，或显存不够（本次未跑）：判过的块已经在库里，没做的段留给接着判的那一趟
         let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &version)
             .map_err(|error| format!("对账失败：{error:?}"))?
             .len();
@@ -4684,6 +4927,8 @@ pub fn run_from_environment(
             gpu_cards_taken_from,
             unit_checksum_buffers_booked_mebibytes,
             facts_verifier_buffers_booked_mebibytes,
+            unit_checksum_buffers_lease_on_the_books_after_checking,
+            facts_verifier_buffers_lease_on_the_books_after_verifying,
         });
     }
     // 用了几张卡：第 ① 段在卡上判的按它用的，第 ② 段在卡上核、单元校验和在卡上对拍的各一张
@@ -4760,6 +5005,8 @@ pub fn run_from_environment(
         gpu_cards_taken_from,
         unit_checksum_buffers_booked_mebibytes,
         facts_verifier_buffers_booked_mebibytes,
+        unit_checksum_buffers_lease_on_the_books_after_checking,
+        facts_verifier_buffers_lease_on_the_books_after_verifying,
     })
 }
 
@@ -4813,33 +5060,43 @@ type BufferLeaseOnTheCardPool = Option<crate::gpu_unit_checks::gpu_test_start::G
 #[cfg(not(feature = "gpu"))]
 type BufferLeaseOnTheCardPool = ();
 
-/// 这一趟的卡（[`gpu_cards_in_use`]）。
+/// 一段算完那一刻核账：那一段的缓冲借的那一笔还在这个进程的账上没有（第 10t 批代码轮判决 K4-1：借了又在用完之前还掉要红）；没借是 `None`。
+#[cfg(feature = "gpu")]
+fn lease_on_the_books_after_its_work(lease: &BufferLeaseOnTheCardPool) -> Option<bool> {
+    lease
+        .as_ref()
+        .map(crate::gpu_unit_checks::gpu_test_start::GpuCardLease::is_on_the_books)
+}
+
+/// 没编进 `gpu` 特性时不借，没有可核的。
+#[cfg(not(feature = "gpu"))]
+fn lease_on_the_books_after_its_work(_lease: &BufferLeaseOnTheCardPool) -> Option<bool> {
+    None
+}
+
+/// 这一趟的卡（[`gpu_cards_in_use`]）：这一趟自己起的那几张（起过卡池的是空的：池里的卡借到哪张才在哪张上起设备，
+/// 用的时候照池取，文件头第 5 条），与卡从哪来。
 #[cfg_attr(
     not(feature = "gpu"),
     allow(
         dead_code,
-        reason = "没编进 gpu 特性时取卡那一份只报错，两种结局只在编进来时造"
+        reason = "没编进 gpu 特性时取卡那一份只报错，这个结局只在编进来时造"
     )
 )]
-enum GpuCardsOfThisRun {
-    Taken {
-        cards: CardsInUse,
-        taken_from: GpuCardsSource,
-    },
-    /// 起过卡池的进程在池里一张卡都登记不上、等满了：「本次未跑」那一行已经打了（文件头第 5 条）。
-    NotRunNoCardOfThePoolHadRoomInTime,
+struct GpuCardsOfThisRun {
+    cards: CardsInUse,
+    taken_from: GpuCardsSource,
 }
 
 /// 这个进程用哪几张卡：配置点了名的只用这台机器那几张、照配置的次序与额度；没配就是本机能用的卡按空闲显存从大到小。
 /// 这个进程起过 GPU 用例共用的卡池（[`crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process`]）就照池里的卡用
-/// （池里的卡要与配置对得上，[`the_card_pool_matches_the_configured_cards`]），不另起一套设备；池里此刻登记不上这个进程的卡这一趟不用，
-/// 一张都登记不上才等，等满了打「本次未跑」那一行、交 [`GpuCardsOfThisRun::NotRunNoCardOfThePoolHadRoomInTime`]。没起过池才自己起。
+/// （池里的卡要与配置对得上，[`the_card_pool_matches_the_configured_cards`]），不另起一套设备，此刻也不登记、不起设备：
+/// 借到哪张卡才在哪张上起（第 10t 批代码轮判决 K2-3）。没起过池才自己起。
 #[cfg(feature = "gpu")]
 fn gpu_cards_in_use(
     configured: Option<&[CardByPriority]>,
     host: CardHost,
 ) -> Result<GpuCardsOfThisRun, String> {
-    use crate::gpu_unit_checks::gpu_test_start::CardsRegisteredForTheProductPath;
     if let Some(pool) = crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process()
     {
         the_card_pool_matches_the_configured_cards(
@@ -4847,19 +5104,10 @@ fn gpu_cards_in_use(
             configured,
             host,
         )?;
-        return match pool.cards_registered_waiting_only_when_none_has_room()? {
-            CardsRegisteredForTheProductPath::Started(cards) => Ok(GpuCardsOfThisRun::Taken {
-                cards: cards.into_iter().map(std::sync::Arc::new).collect(),
-                taken_from: GpuCardsSource::TheCardPoolOfThisProcess,
-            }),
-            CardsRegisteredForTheProductPath::NoCardHadRoomBeforeTheLongestWait { reason } => {
-                println!(
-                    "{}",
-                    gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
-                );
-                Ok(GpuCardsOfThisRun::NotRunNoCardOfThePoolHadRoomInTime)
-            }
-        };
+        return Ok(GpuCardsOfThisRun {
+            cards: Vec::new(),
+            taken_from: GpuCardsSource::TheCardPoolOfThisProcess,
+        });
     }
     let cards = match configured {
         Some(configured) => {
@@ -4894,7 +5142,7 @@ fn gpu_cards_in_use(
                 .to_string(),
         );
     }
-    Ok(GpuCardsOfThisRun::Taken {
+    Ok(GpuCardsOfThisRun {
         cards: cards.into_iter().map(std::sync::Arc::new).collect(),
         taken_from: GpuCardsSource::DevicesStartedForThisRun,
     })
@@ -4943,16 +5191,16 @@ enum UnitChecksumsOfThisRun {
         lease: BufferLeaseOnTheCardPool,
         buffers_booked_mebibytes: Option<u64>,
     },
-    /// 起过卡池的进程借对拍的缓冲等满了：「本次未跑」那一行已经打了。
-    NotRunTheBuffersWereNotBookedInTime,
+    /// 起过卡池的进程此刻借不到对拍的缓冲：「本次未跑」那一行已经打了。
+    NotRunTheGpuMemoryWasShort,
 }
 
 /// 第 ① 段由 CPU 判时，单元两道校验和在哪对拍：登记了 `gpu_unit_checks` 交一张卡，没登记只在 CPU 上。
 /// 卡是这一趟自己起的交优先级最高的那张；起过卡池的照对拍的缓冲（`unit_checksum_buffer_footprint`，这条流的单元写数、核对线程数）
-/// 在账上借第一张放得下的（此刻每张都放不下才等），用借来的那张算；等满了打「本次未跑」那一行（文件头第 5 条）。
+/// 在账上借第一张此刻放得下的，用借来的那张算；借不到打「本次未跑」那一行（文件头第 5 条）。
 ///
 /// # Errors
-/// 登记了却没有起好的卡（不退回只在 CPU 上算）；起过卡池的进程借缓冲时要的比每张卡的账都大、账本读写不了。
+/// 登记了却没有起好的卡（不退回只在 CPU 上算）；起过卡池的进程借缓冲时要的比每张卡空着时能借的都大、卡上的显存读不出。
 #[cfg(feature = "gpu")]
 fn unit_checksums_on(
     route: ComputedOn,
@@ -4968,16 +5216,20 @@ fn unit_checksums_on(
     match route {
         ComputedOn::Cpu => Ok(ready_without_a_lease(UnitChecksumsOn::CpuOnly)),
         ComputedOn::Gpu => {
-            let first_card = cards.and_then(|cards| cards.first()).ok_or_else(|| {
+            let no_card = || {
                 format!(
                     "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回只在 CPU 上算",
                     GpuComputationClass::GpuUnitChecks.name()
                 )
-            })?;
+            };
+            let cards = cards.ok_or_else(no_card)?;
             match cards_taken_from {
-                GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => Ok(
-                    ready_without_a_lease(UnitChecksumsOn::Gpu(first_card.clone())),
-                ),
+                GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
+                    let first_card = cards.first().ok_or_else(no_card)?;
+                    Ok(ready_without_a_lease(UnitChecksumsOn::Gpu(
+                        first_card.clone(),
+                    )))
+                }
                 GpuCardsSource::TheCardPoolOfThisProcess => {
                     let unit_writes = flow
                         .writes
@@ -4987,7 +5239,7 @@ fn unit_checksums_on(
                     let need_mebibytes =
                         unit_checksum_buffer_footprint(unit_writes, checking_threads()).mebibytes();
                     match the_card_pool_this_run_takes_its_cards_from()
-                        .lease_the_first_card_with_room_waiting(
+                        .lease_the_first_card_with_room_now(
                             PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                             need_mebibytes,
                         ) {
@@ -4998,7 +5250,7 @@ fn unit_checksums_on(
                         }),
                         Err(refusal) => {
                             print_the_not_run_line_or_refuse("单元校验和对拍的缓冲", &refusal)?;
-                            Ok(UnitChecksumsOfThisRun::NotRunTheBuffersWereNotBookedInTime)
+                            Ok(UnitChecksumsOfThisRun::NotRunTheGpuMemoryWasShort)
                         }
                     }
                 }
@@ -5041,7 +5293,7 @@ fn gpu_cards_in_use(
 }
 
 /// 第 ① 段在卡上的入口（`gpu` 特性编进来时调 [`judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from`]：
-/// 起过卡池的这一趟草稿区进账，等满了借不到交 [`GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime`]）。
+/// 起过卡池的这一趟草稿区进账，显存不够交 [`GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort`]）。
 #[cfg(feature = "gpu")]
 #[allow(
     clippy::too_many_arguments,
@@ -5097,21 +5349,24 @@ fn judge_my_blocks_on_gpu(
     )
 )]
 enum FactsVerifiedOnACard {
-    /// 核完了：`buffers_booked_mebibytes` 是起过卡池的进程为核对的缓冲借的（不借是 `None`）。
+    /// 核完了：`buffers_booked_mebibytes` 是起过卡池的进程为核对的缓冲借的（不借是 `None`），`lease_on_the_books_after_verifying`
+    /// 是核完那一刻核账、那一笔还在不在（不借是 `None`）。
     Verified {
         outcome: VerificationOutcome,
         buffers_booked_mebibytes: Option<u64>,
+        lease_on_the_books_after_verifying: Option<bool>,
     },
-    /// 起过卡池的进程借核对的缓冲等满了：「本次未跑」那一行已经打了，一块都没核。
-    NotRunTheBuffersWereNotBookedInTime,
+    /// 显存不够（起过卡池的进程此刻借不到核对的缓冲，或卡上 wgpu 报了 Out of Memory）：「本次未跑」那一行已经打了，
+    /// 核过的块在库里。
+    NotRunTheGpuMemoryWasShort,
 }
 
 /// 第 ② 段在卡上：这台机器领到的块都在一张卡上核（第 ① 段的草稿区那时已经放掉，这一段一块一次派活、不占草稿区）。
 /// 卡是这一趟自己起的用优先级最高的那张；起过卡池的照这条流的事实表与一块的缓冲（`facts_verifier_buffer_footprint`）在账上借
-/// 第一张放得下的（此刻每张都放不下才等），用借来的那张核，等满了打「本次未跑」那一行（文件头第 5 条）。
+/// 第一张此刻放得下的，用借来的那张核，借不到打「本次未跑」那一行（文件头第 5 条）。核的时候 wgpu 报 Out of Memory 同样报本次未跑。
 ///
 /// # Errors
-/// 库读写失败；起过卡池的进程借缓冲时要的比每张卡的账都大、账本读写不了。
+/// 库读写失败；起过卡池的进程借缓冲时要的比每张卡空着时能借的都大、卡上的显存读不出。
 #[cfg(feature = "gpu")]
 fn verify_my_blocks_on_the_first_card(
     store: &mut VerdictStore,
@@ -5134,15 +5389,14 @@ fn verify_my_blocks_on_the_first_card(
         GpuCardsSource::TheCardPoolOfThisProcess => {
             let need_mebibytes =
                 facts_verifier_buffer_footprint(&facts, STATES_PER_BLOCK).mebibytes();
-            match the_card_pool_this_run_takes_its_cards_from()
-                .lease_the_first_card_with_room_waiting(
-                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
-                    need_mebibytes,
-                ) {
+            match the_card_pool_this_run_takes_its_cards_from().lease_the_first_card_with_room_now(
+                PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
+                need_mebibytes,
+            ) {
                 Ok(lease) => (std::sync::Arc::new(lease.card().clone()), Some(lease)),
                 Err(refusal) => {
                     print_the_not_run_line_or_refuse("第 ② 段核对的缓冲", &refusal)?;
-                    return Ok(FactsVerifiedOnACard::NotRunTheBuffersWereNotBookedInTime);
+                    return Ok(FactsVerifiedOnACard::NotRunTheGpuMemoryWasShort);
                 }
             }
         }
@@ -5155,14 +5409,20 @@ fn verify_my_blocks_on_the_first_card(
         card.memory_budget_mebibytes(),
         selected.len()
     );
-    let on = FactsVerifierOn::Gpu(card);
-    let tally = verify_blocks_from_these_facts(
-        store,
-        plan,
-        &facts,
-        &|block| selected.contains(&block),
-        &on,
-    )?;
+    let on = FactsVerifierOn::Gpu(card.clone());
+    // 核对在这条线程上一块接一块派活（verify_blocks_from_these_facts），wgpu 的错误范围接得住
+    let tally = match on_the_card_catching_out_of_memory(&card, "第 ② 段核对时", || {
+        verify_blocks_from_these_facts(store, plan, &facts, &|block| selected.contains(&block), &on)
+    }) {
+        Ok(tally) => tally?,
+        Err(reason) => {
+            println!(
+                "{}",
+                gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
+            );
+            return Ok(FactsVerifiedOnACard::NotRunTheGpuMemoryWasShort);
+        }
+    };
     Ok(FactsVerifiedOnACard::Verified {
         outcome: VerificationOutcome {
             verifier: "gpu",
@@ -5171,6 +5431,7 @@ fn verify_my_blocks_on_the_first_card(
         buffers_booked_mebibytes: lease
             .as_ref()
             .map(crate::gpu_unit_checks::gpu_test_start::GpuCardLease::mebibytes),
+        lease_on_the_books_after_verifying: lease_on_the_books_after_its_work(&lease),
     })
 }
 
@@ -5199,15 +5460,15 @@ fn verify_my_blocks_on_the_first_card(
 enum PlacementOfThisRun {
     /// 打了 `CRASH_AMPLIFICATION_PLACEMENT` 那一行。
     Printed,
-    /// 起过卡池的进程借试判的草稿区等满了：「本次未跑」那一行已经打了，没打 PLACEMENT 行。
-    NotRunTheScratchWasNotBookedInTime,
+    /// 起过卡池的进程此刻借不到试判的草稿区：「本次未跑」那一行已经打了，没打 PLACEMENT 行。
+    NotRunTheGpuMemoryWasShort,
 }
 
 /// 只排派活计划：这一趟要判多少、配置的每张卡按额度带得了多少、按优先级各装多少；这台机器装了状态的卡各试判几次派活量速度。
 /// 打一行 `CRASH_AMPLIFICATION_PLACEMENT`（带这一趟录入时库里已判、因此不排的态数 `states_reused`）。别的机器上的卡带得了多少按它的额度算
-/// （绑定上限按本机第一张卡估）。起过卡池的这一趟（`cards_taken_from`），本机每张卡装几个状态照账能借的量压，试判之前按装的状态数
-/// 借草稿区（同第 ① 段，[`scratch_of_each_card_booked_by_where_the_cards_came_from`]），等满了借不到交
-/// [`PlacementOfThisRun::NotRunTheScratchWasNotBookedInTime`]。
+/// （绑定上限按本机第一张卡估）。起过卡池的这一趟（`cards_taken_from`），本机的卡照池里的卡、每张装几个状态照这一刻能借的量压，
+/// 试判之前按装的状态数借草稿区（同第 ① 段，[`scratch_of_each_card_booked_by_where_the_cards_came_from`]），借不到交
+/// [`PlacementOfThisRun::NotRunTheGpuMemoryWasShort`]。
 #[cfg(feature = "gpu")]
 #[allow(
     clippy::too_many_arguments,
@@ -5262,21 +5523,14 @@ fn print_the_placement(
             ));
         }
     };
-    let local_capacities = match cards_taken_from {
-        GpuCardsSource::TheCardPoolOfThisProcess => states_each_card_of_the_pool_lends(
-            the_card_pool_this_run_takes_its_cards_from(),
-            cards,
-            &tables,
-        )?,
-        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
-            states_in_flight_of_each_card(cards, &tables)
-        }
-    };
-    let limit = cards
+    let local_cards = card_capacities_of_this_run(cards_taken_from, cards, &tables)?;
+    let local_capacities: Vec<u64> = local_cards
+        .iter()
+        .map(|card| card.states_in_flight_capacity)
+        .collect();
+    let limit = local_cards
         .first()
-        .expect("用的卡至少一张（gpu_cards_in_use 判过）")
-        .device()
-        .limits()
+        .expect("用的卡至少一张（gpu_cards_in_use 判过、起过卡池的池里至少一张）")
         .max_storage_buffer_binding_size;
     // 配置的每张卡一行，按优先级：这台机器的取实际起好的那张（点了名而用不了的带得了 0 个），别的机器的按额度算
     let rows: Vec<(CardHost, u32, u64)> = match configured {
@@ -5284,7 +5538,7 @@ fn print_the_placement(
             .iter()
             .map(|entry| {
                 let capacity = if entry.host == host {
-                    cards
+                    local_cards
                         .iter()
                         .position(|card| card.index == entry.index)
                         .map_or(0, |position| local_capacities[position])
@@ -5301,7 +5555,7 @@ fn print_the_placement(
                 (entry.host, entry.index, capacity)
             })
             .collect(),
-        None => cards
+        None => local_cards
             .iter()
             .zip(&local_capacities)
             .map(|(card, capacity)| (host, card.index, *capacity))
@@ -5317,7 +5571,7 @@ fn print_the_placement(
     {
         let point = &plan.points[*point_index];
         // 本机每张卡装的状态数（按这一趟卡的次序）：试判的草稿区照它借，借据活到试判完
-        let states_placed_on_local_cards: Vec<u64> = cards
+        let states_placed_on_local_cards: Vec<u64> = local_cards
             .iter()
             .map(|card| {
                 rows.iter()
@@ -5334,14 +5588,15 @@ fn print_the_placement(
             match scratch_of_each_card_booked_by_where_the_cards_came_from(
                 cards_taken_from,
                 cards,
+                &local_cards,
                 &tables,
                 states_of_the_flow,
                 states_placed_on_this_host,
                 &states_placed_on_local_cards,
             )? {
                 ScratchOfEachCard::Ready(scratch) => Some(scratch),
-                ScratchOfEachCard::NotRunTheScratchWasNotBookedInTime => {
-                    return Ok(PlacementOfThisRun::NotRunTheScratchWasNotBookedInTime);
+                ScratchOfEachCard::NotRunTheGpuMemoryWasShort { not_run_line: _ } => {
+                    return Ok(PlacementOfThisRun::NotRunTheGpuMemoryWasShort);
                 }
             }
         };
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs b/crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs
index 551c636..aa617e0 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs
@@ -1,9 +1,10 @@
 //! checker 档模块：crash_amplification、gpu_unit_checks、crash_judge_dispatch
-//! 起过 GPU 用例共用的卡池的进程里，崩溃放量产品路径（`crash_amplification::run_from_environment`）在卡上建的缓冲都在跨进程的显存账上借
+//! 起过 GPU 用例共用的卡池的进程里，崩溃放量产品路径（`crash_amplification::run_from_environment`）在卡上建的缓冲都在这个进程的显存账上借
 //! （`crash_amplification` 文件头第 5 条）：第 ① 段判器的草稿区、第 ① 段由 CPU 判时单元校验和对拍的缓冲、第 ② 段核对的缓冲。
-//! 两条用例各在新池新建文件那条流上按环境真跑一趟，看这一趟交回的借的量：哪一样改回不借，对应的那条就红。
+//! 两条用例各在新池新建文件那条流上按环境真跑一趟，看这一趟交回的借的量，再看用的那一刻核账（判器建好、对拍与核对做完时那一笔借据还在账上，
+//! 第 10t 批代码轮判决 K4-1）：哪一样改回不借、或借了又在用之前还掉，对应的那条就红。
 //! 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」（经这个进程的卡池判，`gpu_test_start::gpu_card_pool_or_report_not_run`），
-//! 有适配器而点名的卡找不到判红；账上被别的进程订满、等满了借不到，产品路径打它自己的「本次未跑」那一行、这一趟照叫停的样子交回，
+//! 有适配器而点名的卡找不到判红；卡上此刻借不到、或卡上 wgpu 报了 Out of Memory，产品路径打它自己的「本次未跑」那一行、这一趟照叫停的样子交回，
 //! 这里同样报「本次未跑」。要 `verdict-store,gpu` 两个特性，少一个这份文件是空的。
 #![cfg(all(feature = "verdict-store", feature = "gpu"))]
 
@@ -119,7 +120,7 @@ impl Drop for GpuRunDirectory {
 }
 
 /// 在这个进程的卡池上按环境真跑一趟：配置开 GPU、单机，`GPU_COMPUTATION_CLASSES` 照 `gpu_classes`，库是新建的空库（每个块都要判）。
-/// 本机没卡（卡池报了「本次未跑」）、或产品路径借显存等满了（这一趟照叫停的样子交回），交 `None`，调用的用例报「本次未跑」当场返回。
+/// 本机没卡（卡池报了「本次未跑」）、或产品路径显存不够（这一趟照叫停的样子交回），交 `None`，调用的用例报「本次未跑」当场返回。
 fn amplify_on_the_card_pool(test_name: &str, gpu_classes: &str) -> Option<EnvironmentRun> {
     gpu_card_pool_or_report_not_run(test_name)?;
     let recorded = record_new_pool_file_creation(test_name);
@@ -161,7 +162,7 @@ fn amplify_on_the_card_pool(test_name: &str, gpu_classes: &str) -> Option<Enviro
             "{}",
             not_run_line(
                 test_name,
-                "产品路径在显存账上借缓冲等满了也借不到（上面 test=crash_amplification::run_from_environment 那一行）"
+                "产品路径显存不够：此刻借不到或卡上报了 Out of Memory（上面 test=crash_amplification::run_from_environment 那一行）"
             )
         );
         return None;
@@ -170,7 +171,8 @@ fn amplify_on_the_card_pool(test_name: &str, gpu_classes: &str) -> Option<Enviro
 }
 
 /// 第 ① 段在卡上判（登记 `crash_judge_gpu`）：每张用到的卡上，判器的草稿区都在显存账上借了，判器用的就是借来的那张
-/// （额度是借的量加驱动那一份，开出来的草稿区不超过借的）。哪天这一支改回不借，借的量是 `None`，这条红。
+/// （额度是借的量加驱动那一份，开出来的草稿区不超过借的），判器建好那一刻那一笔借据还在账上。哪天这一支改回不借，借的量是 `None`，
+/// 借了又在判之前还掉，核账是 `Some(false)`，这条都红。
 #[test]
 fn the_product_path_judges_on_scratch_it_booked_on_the_card_pool() {
     let Some(run) = amplify_on_the_card_pool(
@@ -206,11 +208,18 @@ fn the_product_path_judges_on_scratch_it_booked_on_the_card_pool() {
             card.index,
             card.scratch_mebibytes
         );
+        assert_eq!(
+            card.scratch_lease_on_the_books_when_the_judge_was_built,
+            Some(true),
+            "序号 {} 的卡：判器建好那一刻核账，草稿区那一笔借据要还在账上（借了又在判之前还掉要红）",
+            card.index
+        );
     }
 }
 
 /// 第 ① 段由 CPU 判、单元校验和交卡对拍（登记 `gpu_unit_checks`），第 ② 段在卡上核（登记 `crash_verify_gpu`）：
-/// 两样在卡上建的缓冲都在显存账上借了（借的量不为 0）、第 ② 段在卡上核了。哪一样改回不借，对应的借的量是 `None`，这条红。
+/// 两样在卡上建的缓冲都在显存账上借了（借的量不为 0）、做完那一刻那一笔借据还在账上、第 ② 段在卡上核了。哪一样改回不借，
+/// 对应的借的量是 `None`；借了又在用完之前还掉，核账不是 `Some(true)`：这条都红。
 #[test]
 fn the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool() {
     let Some(run) = amplify_on_the_card_pool(
@@ -231,6 +240,16 @@ fn the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_ca
         "第 ② 段核对的缓冲要在显存账上借：{:?}",
         run.facts_verifier_buffers_booked_mebibytes
     );
+    assert_eq!(
+        run.unit_checksum_buffers_lease_on_the_books_after_checking,
+        Some(true),
+        "单元校验和对拍完那一刻核账：对拍的缓冲那一笔借据要还在账上"
+    );
+    assert_eq!(
+        run.facts_verifier_buffers_lease_on_the_books_after_verifying,
+        Some(true),
+        "第 ② 段核完那一刻核账：核对的缓冲那一笔借据要还在账上"
+    );
     assert_eq!(
         run.verification.map(|outcome| outcome.verifier),
         Some("gpu"),
diff --git a/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs b/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
index 536a2f2..e87a1de 100644
--- a/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
@@ -4,11 +4,14 @@
 //! 与进程里共用一池卡的 `gpu_card_pool_or_report_not_run`）：
 //! 没卡的机器上它报「本次未跑」，有卡而用不上它判红。绕开它直接调 `usable_gpu_cards` 或 `gpu_cards_by_priority` 起卡的用例，
 //! 在有卡的机器上与经入口走同一条路，跑用例分不出；这一条扫源码，不要显卡。
-//! 扫的是 `tests/` 下每份 `.rs`（这一份除外）整份，与 `src/` 下每份 `.rs` 里条件含 `test`（不在 `not(…)` 里）的 `#[cfg(…)]` 下的项，
-//! 顶格不顶格都算（`#[cfg(test)]`、`#[cfg(all(test, feature = "gpu"))]`、`impl` 块里缩进的 `#[cfg(test)]`）；
-//! 产品路径（崩溃放量取卡、`gpu_unit_checks` 自己）不在测试代码里，不扫。注释（`//` 到行尾、`/* */`）不扫，
-//! 字符串与字符字面量的内容也不扫（字符串里的 `//` 不当注释起头，同一行后面的代码照扫）。
+//! 扫的是 `tests/` 下每份 `.rs`（这一份除外）整份，与 `src/` 下每份 `.rs` 里的测试代码：`#[test]`、条件含 `test`（不在 `not(…)` 里）的
+//! `#[cfg(…)]` 下的项，顶格不顶格、与别的属性写在同一行、`cfg` 与括号之间带空白都算（`#[cfg(test)]`、`#[cfg(all(test, feature = "gpu"))]`、
+//! `impl` 块里缩进的 `#[cfg(test)]`、`#[allow(…)] #[cfg (test)]`），文件级的 `#![cfg(test)]` 整份算，测试代码里 `mod 名字;` 引进来的
+//! 那份文件也整份算（第 10t 批代码轮判决 K4-2）；产品路径（崩溃放量取卡、`gpu_unit_checks` 自己）不在测试代码里，不扫。
+//! 注释（`//` 到行尾、`/* */`）不扫，字符串（普通、`b"…"`、`c"…"`、原始的 `r#"…"#`、`br#"…"#`、`cr#"…"#`）与字符字面量的内容也不扫
+//! （字符串里的 `//` 不当注释起头，同一行后面的代码照扫）。
 //! 判的是名字出现没出现，不只是带括号的调用：`use … as 别名` 那一行、拿函数当值、包一层改名都留得下这个名字（第 10t 批 K4-4）。
+//! 不扫的（交 review）：`include!` 与 `#[path = "…"]` 引进来的文件、`benches/` 与 `examples/`、产品代码里包一层再从测试调的。
 //! 改走卡池的几份测试文件（[`FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL`]）另判一层：只许经 `gpu_card_pool_or_report_not_run` 取卡、
 //! 按量借，不许再各自拿第一张能用的卡、拿整份卡表或直接在卡上建判器（各按空闲显存的比例开，几个用例并行时互抢显存）。
 
@@ -18,9 +21,15 @@ use std::path::{Path, PathBuf};
 const THIS_FILE_NAME: &str =
     "gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs";
 
-/// 绕开入口直接起卡的函数名：代码里出现就算（调用、`use` 引进来、改名、当值传）。
-const CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY: [&str; 2] =
-    ["usable_gpu_cards", "gpu_cards_by_priority"];
+/// 绕开入口直接起卡的函数名：代码里出现就算（调用、`use` 引进来、改名、当值传）。后三个是直接用 wgpu 起设备
+/// （第 10t 批代码轮判决 K4-2）：`gpu_unit_checks` 自己的产品代码起设备用它们，测试代码里不该有。
+const CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY: [&str; 5] = [
+    "usable_gpu_cards",
+    "gpu_cards_by_priority",
+    "request_adapter",
+    "request_device",
+    "enumerate_adapters",
+];
 
 /// 经入口取卡的函数名（数调用）。
 const CARD_TAKING_FUNCTIONS_THROUGH_THE_ENTRY: [&str; 3] = [
@@ -139,8 +148,8 @@ fn blank_a_string_literal(
     position
 }
 
-/// 一份源码每一行的代码部分，带 1 起的行号：注释（`//` 到行尾、可嵌套的 `/* */`）去掉，字符串（普通、`b"…"`、`r#"…"#`）与
-/// 字符字面量的内容换成空格（引号留着）。字符串里的 `//` 不当注释，写在字符串里的名字也不当代码里的名字。
+/// 一份源码每一行的代码部分，带 1 起的行号：注释（`//` 到行尾、可嵌套的 `/* */`）去掉，字符串（普通、`b"…"`、`c"…"`、`r#"…"#`、
+/// `br#"…"#`、`cr#"…"#`）与字符字面量的内容换成空格（引号留着）。字符串里的 `//` 不当注释，写在字符串里的名字也不当代码里的名字。
 fn code_lines(source: &str) -> Vec<(usize, String)> {
     let characters: Vec<char> = source.chars().collect();
     let mut code = String::with_capacity(source.len());
@@ -187,8 +196,8 @@ fn code_lines(source: &str) -> Vec<(usize, String)> {
             ('"', _) => {
                 position = blank_a_string_literal(&characters, position, None, &mut code);
             }
-            ('r' | 'b', Some(_)) if !follows_an_identifier => {
-                let prefix_length = if character == 'b' && next == Some('r') {
+            ('r' | 'b' | 'c', Some(_)) if !follows_an_identifier => {
+                let prefix_length = if (character == 'b' || character == 'c') && next == Some('r') {
                     2
                 } else {
                     1
@@ -205,7 +214,7 @@ fn code_lines(source: &str) -> Vec<(usize, String)> {
                         );
                     }
                     (false, Some(0)) => {
-                        code.push('b');
+                        code.push(character);
                         position =
                             blank_a_string_literal(&characters, position + 1, None, &mut code);
                     }
@@ -300,10 +309,191 @@ fn cfg_predicate_includes_test_outside_a_not(attribute: &str) -> bool {
     false
 }
 
-/// `src/` 下一份源码里属于测试代码的行：条件含 `test`（不在 `not(…)` 里，[`cfg_predicate_includes_test_outside_a_not`]）的
-/// `#[cfg(…)]` 属性（可以缩进、可以跨几行）连同它下面的那一项（之间别的属性一并收），项到它的 `{…}` 收尾、或在最外层遇到
-/// `;`、`,`、多出来的右括号为止。交回 (测试代码的行, 收了几项)。
-fn test_code_lines_of_a_library_source(lines: &[(usize, String)]) -> (Vec<(usize, String)>, usize) {
+/// 项的关键字：属性下面的东西头上（属性之后、第一个 `{`、`;`、`=`、`,` 之前）有其中之一就是一项（函数、模块、`impl` 块……），
+/// 项到它的 `{…}` 收尾或最外层的 `;` 为止，中间的逗号（泛型参数、`where` 子句）不算收尾。
+const ITEM_KEYWORDS: [&str; 12] = [
+    "fn",
+    "mod",
+    "impl",
+    "struct",
+    "enum",
+    "trait",
+    "const",
+    "static",
+    "type",
+    "use",
+    "union",
+    "macro_rules",
+];
+
+/// 一条属性（去掉注释、字符串内容之后的原文里，从 `#` 起）：是不是内层属性（`#![…]`）、`[` 与配对的 `]` 之间的原文、那个 `]` 的位置。
+struct AttributeInTheCode {
+    is_inner: bool,
+    inside: String,
+    closing_position: usize,
+}
+
+/// 从 `hash_position` 那个 `#` 起是不是一条属性（`#` 与 `!`、`[` 之间可以有空白）；是就交回它。
+fn attribute_at(characters: &[(usize, char)], hash_position: usize) -> Option<AttributeInTheCode> {
+    let after = |position: usize| {
+        (position..characters.len())
+            .find(|candidate| !characters[*candidate].1.is_whitespace())
+            .unwrap_or(characters.len())
+    };
+    let mut position = after(hash_position + 1);
+    let is_inner = characters.get(position).map(|(_, character)| *character) == Some('!');
+    if is_inner {
+        position = after(position + 1);
+    }
+    if characters.get(position).map(|(_, character)| *character) != Some('[') {
+        return None;
+    }
+    let mut bracket_depth = 0_usize;
+    for (offset, (_, character)) in characters[position..].iter().enumerate() {
+        match character {
+            '[' => bracket_depth += 1,
+            ']' => {
+                bracket_depth -= 1;
+                if bracket_depth == 0 {
+                    let closing_position = position + offset;
+                    return Some(AttributeInTheCode {
+                        is_inner,
+                        inside: characters[position + 1..closing_position]
+                            .iter()
+                            .map(|(_, inside_character)| *inside_character)
+                            .collect(),
+                        closing_position,
+                    });
+                }
+            }
+            _ => {}
+        }
+    }
+    None
+}
+
+/// 属性（`[` 与 `]` 之间的原文）让它下面的东西只在测试里编：裸的 `test`（`#[test]`），或条件含 `test`（不在 `not(…)` 里，
+/// [`cfg_predicate_includes_test_outside_a_not`]）的 `cfg(…)`（`cfg` 与括号之间可以有空白）。
+fn attribute_marks_test_code(attribute_inside: &str) -> bool {
+    let trimmed = attribute_inside.trim();
+    if trimmed == "test" {
+        return true;
+    }
+    let Some(after_the_name) = trimmed.strip_prefix("cfg") else {
+        return false;
+    };
+    let predicate = after_the_name.trim_start();
+    predicate.starts_with('(') && cfg_predicate_includes_test_outside_a_not(predicate)
+}
+
+/// 从 `start` 起的那一项（之前的属性已经读过）到哪一个字符收尾，与它是不是 `mod 名字;`（交回名字）：头上有项的关键字的
+/// （[`ITEM_KEYWORDS`]，头上先跳过别的属性）到它的 `{…}` 收尾或最外层的 `;` 为止；别的（字段、枚举成员、语句、表达式、match 的一支）
+/// 另在最外层遇到 `,`、或多出来的右括号时收尾。
+fn the_item_after(characters: &[(usize, char)], start: usize) -> (usize, Option<String>) {
+    let mut head_start = start;
+    loop {
+        head_start = (head_start..characters.len())
+            .find(|candidate| !characters[*candidate].1.is_whitespace())
+            .unwrap_or(characters.len());
+        match characters.get(head_start).map(|(_, character)| *character) {
+            Some('#') => match attribute_at(characters, head_start) {
+                Some(attribute) => head_start = attribute.closing_position + 1,
+                None => break,
+            },
+            Some(_) | None => break,
+        }
+    }
+    let head_end = characters[head_start..]
+        .iter()
+        .position(|(_, character)| matches!(character, '{' | ';' | '=' | ','))
+        .map_or(characters.len(), |offset| head_start + offset);
+    let head: String = characters[head_start..head_end]
+        .iter()
+        .map(|(_, character)| *character)
+        .collect();
+    let is_an_item = ITEM_KEYWORDS
+        .iter()
+        .any(|keyword| mentions_of(&head, keyword) > 0);
+    let module_name = || {
+        let mut words = head
+            .split(|character: char| !is_an_identifier_character(character))
+            .filter(|word| !word.is_empty());
+        words.find(|word| *word == "mod")?;
+        words.next().map(str::to_string)
+    };
+    let mut depth = 0_usize;
+    let mut body_started = false;
+    for (offset, (_, character)) in characters[start..].iter().enumerate() {
+        let here = start + offset;
+        match character {
+            '(' | '[' | '{' => {
+                if *character == '{' && depth == 0 {
+                    body_started = true;
+                }
+                depth += 1;
+            }
+            ')' | ']' | '}' => {
+                if depth == 0 {
+                    return (here, None);
+                }
+                depth -= 1;
+                if depth == 0 && *character == '}' && body_started {
+                    return (here, None);
+                }
+            }
+            ';' if depth == 0 => {
+                return (here, if is_an_item { module_name() } else { None });
+            }
+            ',' if depth == 0 && !is_an_item => return (here, None),
+            _ => {}
+        }
+    }
+    (characters.len() - 1, None)
+}
+
+/// 一份库源码里属于测试代码的部分（[`test_code_of_a_library_source`]）。
+struct TestCodeOfALibrarySource {
+    /// 测试代码的行。
+    lines: Vec<(usize, String)>,
+    /// 收了几项（整份都是测试代码的算一项）。
+    item_count: usize,
+    /// 测试代码里 `mod 名字;` 引进来的另一份文件的模块名：那份文件整份都是测试代码（[`out_of_line_module_file`]）。
+    out_of_line_module_names: Vec<String>,
+}
+
+/// 一份源码的代码行里每一处 `mod 名字;`（模块放在另一份文件里）的名字。
+fn out_of_line_module_names(lines: &[(usize, String)]) -> Vec<String> {
+    let mut tokens: Vec<String> = Vec::new();
+    for (_, code) in lines {
+        let mut word = String::new();
+        for character in code.chars().chain(std::iter::once(' ')) {
+            if is_an_identifier_character(character) {
+                word.push(character);
+                continue;
+            }
+            if !word.is_empty() {
+                tokens.push(std::mem::take(&mut word));
+            }
+            if !character.is_whitespace() {
+                tokens.push(character.to_string());
+            }
+        }
+    }
+    tokens
+        .windows(3)
+        .filter(|window| {
+            window[0] == "mod"
+                && window[2] == ";"
+                && window[1].chars().all(is_an_identifier_character)
+        })
+        .map(|window| window[1].clone())
+        .collect()
+}
+
+/// `src/` 下一份源码里属于测试代码的部分：标着测试的属性（[`attribute_marks_test_code`]：`#[test]`、条件含 `test` 的 `#[cfg(…)]`，
+/// 写在行首、行中、与别的属性同一行、`cfg` 后面带空白都算）连同它下面的那一项（[`the_item_after`]）；文件级的 `#![cfg(test)]` 让整份都是测试代码。
+/// 测试代码里的 `mod 名字;` 记下名字，那份文件另由调用方整份当测试代码扫。
+fn test_code_of_a_library_source(lines: &[(usize, String)]) -> TestCodeOfALibrarySource {
     let characters: Vec<(usize, char)> = lines
         .iter()
         .enumerate()
@@ -315,81 +505,128 @@ fn test_code_lines_of_a_library_source(lines: &[(usize, String)]) -> (Vec<(usize
         .collect();
     let mut collected_line_positions = std::collections::BTreeSet::new();
     let mut item_count = 0;
-    let mut line_position = 0;
-    while line_position < lines.len() {
-        if !lines[line_position].1.trim_start().starts_with("#[cfg(") {
-            line_position += 1;
+    let mut module_names = Vec::new();
+    let mut position = 0;
+    while position < characters.len() {
+        if characters[position].1 != '#' {
+            position += 1;
             continue;
         }
-        let attribute_start = characters
-            .iter()
-            .position(|(position, character)| *position == line_position && *character == '#')
-            .expect("这一行去掉前导空白以 # 起头");
-        let mut bracket_depth = 0_usize;
-        let mut attribute_end = attribute_start;
-        for (offset, (_, character)) in characters[attribute_start..].iter().enumerate() {
-            match character {
-                '[' => bracket_depth += 1,
-                ']' => {
-                    bracket_depth -= 1;
-                    if bracket_depth == 0 {
-                        attribute_end = attribute_start + offset;
-                        break;
-                    }
-                }
-                _ => {}
-            }
-        }
-        let attribute: String = characters[attribute_start..=attribute_end]
-            .iter()
-            .map(|(_, character)| *character)
-            .collect();
-        if !cfg_predicate_includes_test_outside_a_not(&attribute) {
-            line_position = characters[attribute_end].0 + 1;
+        let Some(attribute) = attribute_at(&characters, position) else {
+            position += 1;
+            continue;
+        };
+        if !attribute_marks_test_code(&attribute.inside) {
+            // 同一行后面可以还有属性（`#[allow(…)] #[cfg(test)]`）：从这一条之后接着找
+            position = attribute.closing_position + 1;
             continue;
         }
-        let mut depth = 0_usize;
-        let mut body_started = false;
-        let mut item_end = characters.len() - 1;
-        for (offset, (_, character)) in characters[attribute_end + 1..].iter().enumerate() {
-            let here = attribute_end + 1 + offset;
-            match character {
-                '(' | '[' | '{' => {
-                    if *character == '{' && depth == 0 {
-                        body_started = true;
-                    }
-                    depth += 1;
-                }
-                ')' | ']' | '}' => {
-                    if depth == 0 {
-                        item_end = here;
-                        break;
-                    }
-                    depth -= 1;
-                    if depth == 0 && *character == '}' && body_started {
-                        item_end = here;
-                        break;
-                    }
-                }
-                ';' | ',' if depth == 0 => {
-                    item_end = here;
-                    break;
-                }
-                _ => {}
-            }
+        if attribute.is_inner {
+            // 文件级的 `#![cfg(test)]`：整份都是测试代码
+            return TestCodeOfALibrarySource {
+                lines: lines.to_vec(),
+                item_count: item_count + 1,
+                out_of_line_module_names: out_of_line_module_names(lines),
+            };
         }
+        let (item_end, module_name) = the_item_after(&characters, attribute.closing_position + 1);
         item_count += 1;
-        let last_line_position = characters[item_end].0;
-        collected_line_positions.extend(line_position..=last_line_position);
-        line_position = last_line_position + 1;
+        collected_line_positions.extend(characters[position].0..=characters[item_end].0);
+        module_names.extend(module_name);
+        position = item_end + 1;
     }
-    (
-        collected_line_positions
+    TestCodeOfALibrarySource {
+        lines: collected_line_positions
             .into_iter()
-            .map(|position| lines[position].clone())
+            .map(|line_position| lines[line_position].clone())
             .collect(),
         item_count,
-    )
+        out_of_line_module_names: module_names,
+    }
+}
+
+/// `parent` 这份源码里 `mod 名字;` 引进来的那份文件：`lib.rs`、`main.rs`、`mod.rs` 里的在同一目录，别的在与它同名的子目录；
+/// `名字.rs` 或 `名字/mod.rs`，都不在交 `None`（编不过的写法，不归这里判）。`#[path = "…"]` 指的位置不认（属性的字符串内容扫之前已经抹掉）。
+fn out_of_line_module_file(parent: &Path, module_name: &str) -> Option<PathBuf> {
+    let directory = match parent.file_name().and_then(|name| name.to_str()) {
+        Some("lib.rs" | "main.rs" | "mod.rs") => parent.parent()?.to_path_buf(),
+        Some(_) | None => parent.with_extension(""),
+    };
+    [
+        directory.join(format!("{module_name}.rs")),
+        directory.join(module_name).join("mod.rs"),
+    ]
+    .into_iter()
+    .find(|candidate| candidate.is_file())
+}
+
+/// `src/` 下扫出来的：绕开入口起卡的那几行、测试代码收了几项、测试代码里经入口取卡几处。
+struct LibraryScan {
+    bypasses: Vec<BypassingLine>,
+    test_item_count: usize,
+    entry_calls_in_test_code: usize,
+}
+
+/// 扫 `source_directory`（`src/`）下每份 `.rs` 的测试代码（[`test_code_of_a_library_source`]），连同测试代码里 `mod 名字;` 引进来的文件
+/// （整份都是测试代码，它们再引的照样）；文件名照 `package_root` 起的相对路径记。同一行只记一次。
+fn scan_the_library_sources(source_directory: &Path, package_root: &Path) -> LibraryScan {
+    let relative = |path: &Path| {
+        path.strip_prefix(package_root)
+            .expect("扫的文件都在包根下")
+            .to_string_lossy()
+            .into_owned()
+    };
+    let read = |path: &Path| {
+        std::fs::read_to_string(path)
+            .unwrap_or_else(|error| panic!("{} 读不出：{error}", path.display()))
+    };
+    let mut scan = LibraryScan {
+        bypasses: Vec::new(),
+        test_item_count: 0,
+        entry_calls_in_test_code: 0,
+    };
+    let mut module_files_that_are_test_code: Vec<PathBuf> = Vec::new();
+    for path in rust_source_files_under(source_directory) {
+        let test_code = test_code_of_a_library_source(&code_lines(&read(&path)));
+        scan.test_item_count += test_code.item_count;
+        scan.entry_calls_in_test_code += entry_call_count(&test_code.lines);
+        scan.bypasses.extend(lines_mentioning(
+            &relative(&path),
+            &test_code.lines,
+            &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
+        ));
+        module_files_that_are_test_code.extend(
+            test_code
+                .out_of_line_module_names
+                .iter()
+                .filter_map(|module_name| out_of_line_module_file(&path, module_name)),
+        );
+    }
+    let mut visited = std::collections::BTreeSet::new();
+    while let Some(module_file) = module_files_that_are_test_code.pop() {
+        if !visited.insert(module_file.clone()) {
+            continue;
+        }
+        let lines = code_lines(&read(&module_file));
+        scan.test_item_count += 1;
+        scan.entry_calls_in_test_code += entry_call_count(&lines);
+        scan.bypasses.extend(lines_mentioning(
+            &relative(&module_file),
+            &lines,
+            &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
+        ));
+        module_files_that_are_test_code.extend(
+            out_of_line_module_names(&lines)
+                .iter()
+                .filter_map(|module_name| out_of_line_module_file(&module_file, module_name)),
+        );
+    }
+    scan.bypasses.sort_by(|left, right| {
+        (&left.file_relative_to_the_package, left.line_number)
+            .cmp(&(&right.file_relative_to_the_package, right.line_number))
+    });
+    scan.bypasses.dedup();
+    scan
 }
 
 /// 这几行里经入口取卡的调用有几处。
@@ -452,19 +689,10 @@ fn every_gpu_test_takes_its_cards_through_gpu_test_start() {
             &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
         ));
     }
-    let mut library_test_item_count = 0_usize;
-    let mut entry_calls_in_library_test_code = 0_usize;
-    for path in rust_source_files_under(&package_root.join("src")) {
-        let source = read(&path);
-        let (lines, item_count) = test_code_lines_of_a_library_source(&code_lines(&source));
-        library_test_item_count += item_count;
-        entry_calls_in_library_test_code += entry_call_count(&lines);
-        bypasses.extend(lines_mentioning(
-            &relative(&path),
-            &lines,
-            &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
-        ));
-    }
+    let library = scan_the_library_sources(&package_root.join("src"), package_root);
+    let library_test_item_count = library.test_item_count;
+    let entry_calls_in_library_test_code = library.entry_calls_in_test_code;
+    bypasses.extend(library.bypasses);
     assert!(
         scanned_test_file_count >= 20 && library_test_item_count >= 10,
         "扫到的 tests/ 文件 {scanned_test_file_count} 份、src/ 里的测试项 {library_test_item_count} 个，少得不对，扫法坏了"
@@ -618,11 +846,15 @@ fn test_code_under_a_compound_or_indented_cfg_test_in_a_library_source_is_caught
                   \x20       let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
                   \x20   }\n\
                   }\n";
-    let (lines, item_count) = test_code_lines_of_a_library_source(&code_lines(sample));
-    assert_eq!(item_count, 2, "两项测试代码：{lines:#?}");
+    let test_code = test_code_of_a_library_source(&code_lines(sample));
+    assert_eq!(
+        test_code.item_count, 2,
+        "两项测试代码：{:#?}",
+        test_code.lines
+    );
     let bypasses = lines_mentioning(
         "src/sample.rs",
-        &lines,
+        &test_code.lines,
         &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
     );
     assert_eq!(
@@ -631,3 +863,199 @@ fn test_code_under_a_compound_or_indented_cfg_test_in_a_library_source_is_caught
         "cfg(all(test, …)) 与缩进的 cfg(test) 下的起卡调用要扫得出，cfg(not(test)) 下的不扫：{bypasses:#?}"
     );
 }
+
+/// 一份库源码样本里扫得出的起卡那几行的行号（[`test_code_of_a_library_source`]）。
+fn bypass_line_numbers_in_a_library_sample(sample: &str) -> Vec<usize> {
+    lines_mentioning(
+        "src/sample.rs",
+        &test_code_of_a_library_source(&code_lines(sample)).lines,
+        &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
+    )
+    .iter()
+    .map(|bypass| bypass.line_number)
+    .collect()
+}
+
+/// 第 10t 批代码轮判决 K4-2 红样本一：C 字符串 `c"…"` 与原始 C 字符串 `cr#"…"#` 之后同一行的起卡调用照扫
+/// （以前 `cr` 的 `r` 跟在 `c` 后面不认成原始串，把 `\` 当转义、吞到下一个引号）。
+#[test]
+fn a_card_taking_call_after_a_raw_c_string_on_the_same_line_is_caught() {
+    let sample = r###"fn takes_cards() {
+    let _a = cr#"x" "#; let _cards = usable_gpu_cards(); let _b = cr#"y"#;
+    let _c = c"\\"; let _card = gpu_cards_by_priority(&[]);
+    let _d = cr"\"; let _more = usable_gpu_cards();
+}
+"###;
+    let bypasses = lines_mentioning(
+        "tests/sample.rs",
+        &code_lines(sample),
+        &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
+    );
+    assert_eq!(
+        bypasses
+            .iter()
+            .map(|bypass| bypass.line_number)
+            .collect::<Vec<_>>(),
+        vec![2, 3, 4],
+        "C 字符串之后的起卡调用要扫得出：{bypasses:#?}"
+    );
+}
+
+/// 第 10t 批代码轮判决 K4-2 红样本二：库源码里 `#[cfg(test)]` 下的函数带泛型参数（`<A, B>`）或 `where` 子句，里面的逗号不让这一项在签名里收尾，
+/// 函数体照扫。
+#[test]
+fn test_code_whose_signature_holds_commas_is_scanned_to_the_end_of_its_body() {
+    let sample = "#[cfg(test)]\n\
+                  fn helper<A, B>() {\n\
+                  \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
+                  }\n\
+                  \n\
+                  #[cfg(test)]\n\
+                  fn bounded<T>()\n\
+                  where\n\
+                  \x20   T: Copy,\n\
+                  {\n\
+                  \x20   let _cards = crate::gpu_unit_checks::gpu_cards_by_priority(&[]);\n\
+                  }\n";
+    assert_eq!(
+        bypass_line_numbers_in_a_library_sample(sample),
+        vec![3, 11],
+        "泛型参数、where 子句里的逗号之后的函数体要扫"
+    );
+}
+
+/// 第 10t 批代码轮判决 K4-2 红样本三：库源码里没有 `#[cfg(test)]`、只有裸 `#[test]` 的函数也是测试代码（只在测试里编），照扫。
+#[test]
+fn a_bare_test_function_in_a_library_source_is_scanned() {
+    let sample = "pub fn product() {}\n\
+                  \n\
+                  #[test]\n\
+                  fn takes_cards() {\n\
+                  \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
+                  }\n";
+    assert_eq!(bypass_line_numbers_in_a_library_sample(sample), vec![5]);
+}
+
+/// 第 10t 批代码轮判决 K4-2 红样本四：`cfg(test)` 的几种不在行首、不紧挨着的写法：文件级的 `#![cfg(test)]`（整份都是测试代码，
+/// 下面第二项也算）、与别的属性写在同一行、`cfg` 与括号之间带空白。
+#[test]
+fn cfg_test_written_off_the_start_of_the_line_or_spaced_or_at_the_file_level_is_caught() {
+    let file_level = "#![cfg(test)]\n\
+                      fn first() {\n\
+                      \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
+                      }\n\
+                      fn second() {\n\
+                      \x20   let _cards = crate::gpu_unit_checks::gpu_cards_by_priority(&[]);\n\
+                      }\n";
+    assert_eq!(
+        bypass_line_numbers_in_a_library_sample(file_level),
+        vec![3, 6],
+        "文件级的 #![cfg(test)]：整份都扫"
+    );
+    let same_line = "#[allow(dead_code)] #[cfg(test)] fn helper() {\n\
+                     \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
+                     }\n";
+    assert_eq!(
+        bypass_line_numbers_in_a_library_sample(same_line),
+        vec![2],
+        "与别的属性同一行的 #[cfg(test)]"
+    );
+    let spaced = "#[cfg (test)]\n\
+                  fn helper() {\n\
+                  \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
+                  }\n";
+    assert_eq!(
+        bypass_line_numbers_in_a_library_sample(spaced),
+        vec![3],
+        "cfg 与括号之间带空白"
+    );
+}
+
+/// 第 10t 批代码轮判决 K4-2 红样本五：不经那两个名字、直接用 wgpu 起设备（`request_adapter`、`request_device`、`enumerate_adapters`）照扫。
+#[test]
+fn a_test_that_starts_a_device_straight_through_wgpu_is_caught() {
+    let sample = "fn takes_cards(instance: &wgpu::Instance) {\n\
+                  \x20   let adapter = pollster::block_on(instance.request_adapter(&options));\n\
+                  \x20   let device = pollster::block_on(adapter.request_device(&descriptor));\n\
+                  \x20   let adapters = instance.enumerate_adapters(wgpu::Backends::VULKAN);\n\
+                  }\n";
+    let bypasses = lines_mentioning(
+        "tests/sample.rs",
+        &code_lines(sample),
+        &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
+    );
+    assert_eq!(
+        bypasses
+            .iter()
+            .map(|bypass| bypass.line_number)
+            .collect::<Vec<_>>(),
+        vec![2, 3, 4],
+        "直接用 wgpu 起设备的每一行都要扫得出：{bypasses:#?}"
+    );
+}
+
+/// 一份临时的包目录（临时目录下，带用例名与进程号）：用例收尾时删。
+struct TemporaryPackage {
+    root: PathBuf,
+}
+
+impl TemporaryPackage {
+    /// 建好包根与 `src/`，按 `(相对 src/ 的路径, 内容)` 写进去。
+    fn with_sources(test_name: &str, sources: &[(&str, &str)]) -> Self {
+        let root = std::env::temp_dir().join(format!(
+            "singlefs-gpu-entry-scan-{test_name}-{}",
+            std::process::id()
+        ));
+        let _ = std::fs::remove_dir_all(&root);
+        for (relative_path, content) in sources {
+            let path = root.join("src").join(relative_path);
+            std::fs::create_dir_all(path.parent().expect("src 下的文件有上一层"))
+                .expect("建得了临时目录");
+            std::fs::write(&path, content).expect("写得了临时文件");
+        }
+        Self { root }
+    }
+}
+
+impl Drop for TemporaryPackage {
+    fn drop(&mut self) {
+        let _ = std::fs::remove_dir_all(&self.root);
+    }
+}
+
+/// 第 10t 批代码轮判决 K4-2 红样本六：库源码里 `#[cfg(test)] mod gpu_tests;` 把测试放进另一份文件，那份文件里没有 cfg 属性：
+/// 整份当测试代码扫（它再引的 `mod deeper;` 照样），不当产品代码放过；`lib.rs` 里不在测试代码里的 `mod product;` 不扫。
+#[test]
+fn a_test_module_kept_in_another_file_is_scanned_whole() {
+    let package = TemporaryPackage::with_sources(
+        "test-module-in-another-file",
+        &[
+            ("lib.rs", "mod product;\n#[cfg(test)]\nmod gpu_tests;\n"),
+            (
+                "product.rs",
+                "pub fn product() {\n    let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n}\n",
+            ),
+            (
+                "gpu_tests.rs",
+                "mod deeper;\nfn helper() {\n    let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n}\n",
+            ),
+            (
+                "gpu_tests/deeper.rs",
+                "fn deeper() {\n    let _cards = crate::gpu_unit_checks::gpu_cards_by_priority(&[]);\n}\n",
+            ),
+        ],
+    );
+    let scan = scan_the_library_sources(&package.root.join("src"), &package.root);
+    assert_eq!(
+        scan.bypasses
+            .iter()
+            .map(|bypass| (
+                bypass.file_relative_to_the_package.as_str(),
+                bypass.line_number
+            ))
+            .collect::<Vec<_>>(),
+        vec![("src/gpu_tests.rs", 3), ("src/gpu_tests/deeper.rs", 2)],
+        "另一份文件里的测试模块整份要扫，产品代码的 mod product; 不扫：{:#?}",
+        scan.bypasses
+    );
+}
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
index c242770..07ab308 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
@@ -957,8 +957,8 @@ fn a_run_importing_another_library_reports_the_states_judged_before_the_import_a
 /// L1：只排派活计划的那一趟照实报录入时库里已判的态数，不写 0。同一份库上第一趟（单机 CPU）把整条流判完；第二趟配置开 GPU、
 /// 登记 `crash_judge_gpu`、只排计划：一块都不判，复用数等于整条流的态数。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」
 /// （经这个进程的卡池判有没有卡，`gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run`），有适配器而点名的卡找不到判红。
-/// 只排计划的那一趟一个状态都不上卡判，这条用例不向卡池借显存；那一趟的卡照这个进程的卡池用（第 10t 批 K4-4：每张卡这个进程只起一个设备，
-/// 驱动那一份在显存账上登记了），不经产品路径另起一套不记账的设备。要 `verdict-store,gpu` 两个特性。
+/// 只排计划的那一趟一个状态都不上卡判，这条用例不向卡池借显存；那一趟的卡照这个进程的卡池用（第 10t 批 K4-4：每张卡这个进程只起一个设备；
+/// 第 10x 批：借到哪张卡才在哪张上起，驱动那一份记在这个进程的显存账上），不经产品路径另起一套不记账的设备。要 `verdict-store,gpu` 两个特性。
 #[cfg(feature = "gpu")]
 #[test]
 fn a_placement_only_run_reports_the_states_judged_at_recording_as_reused() {
````

## mutations-append.tsv

````text
第10x批 显存账 借的时候不照现读的空闲（照卡的总量算）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        room_now_mebibytes: capacity_of_memory_this_process_may_hold(may_hold_mebibytes)	        room_now_mebibytes: capacity_of_memory_this_process_may_hold(reading.total_mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_booking_after_an_outsider_took_memory_follows_the_free_memory_read_at_that_booking	a_booking_after_an_outsider_took_memory_follows_the_free_memory_read_at_that_booking
第10x批 显存账 此刻放不下的判成比每张卡都大（判红）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        .map(|room| room.ceiling_mebibytes)	        .map(|room| room.room_now_mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_booking_after_an_outsider_left_gets_the_memory_it_returned	a_booking_after_an_outsider_left_gets_the_memory_it_returned
第10x批 显存账 这个进程借着的不从能借的里扣	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .saturating_sub(booked.leases_mebibytes),	            .saturating_sub(0),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::two_bookings_of_this_process_on_one_card_stay_within_the_free_memory_read_now	two_bookings_of_this_process_on_one_card_stay_within_the_free_memory_read_now
第10x批 显存账 实际占着的不封顶到许出去的（分配器多拿的块也算成能借的）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        Some(occupied) => occupied.min(promised_mebibytes),	        Some(occupied) => occupied,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised	the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised
第10x批 显存账 起好了设备驱动那一份照样从空闲里扣	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::DeviceStarted => {\n                mebibytes_left_for_the_driver.min(promised_mebibytes)\n            }	            DriverShareOnTheCard::DeviceStarted => 0,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised	the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised
第10x批 显存账 还没起设备就把驱动那一份当已经在卡上（空闲不到一份驱动开销也借给）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::NotRegistered\n            | DriverShareOnTheCard::RegisteredDeviceNotStarted => 0,	            DriverShareOnTheCard::NotRegistered\n            | DriverShareOnTheCard::RegisteredDeviceNotStarted => mebibytes_left_for_the_driver,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_with_less_free_memory_than_one_driver_share_lends_nothing_and_registers_nothing	a_card_with_less_free_memory_than_one_driver_share_lends_nothing_and_registers_nothing
第10x批 显存账 K4-3 能占的那一项换成 u64::MAX（额度变量与配置额度都设了）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                .min(\n                    memory_this_process_may_hold_mebibytes\n                        .saturating_sub(mebibytes_left_for_the_driver),\n                ),	                .min(u64::MAX),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_memory_this_process_may_hold_caps_the_capacity_even_when_both_quotas_are_set	the_memory_this_process_may_hold_caps_the_capacity_even_when_both_quotas_are_set
第10x批 显存账 K4-3 没登记就起设备不断言（先登记、后建设备）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            self.is_this_process_registered_on_the_card(card_position),\n            "第 {card_position} 张卡：这个进程在它的账上登记之前就要起设备（先登记、后建设备）"	            card_position < usize::MAX,\n            "第 {card_position} 张卡：这个进程在它的账上登记之前就要起设备（先登记、后建设备）"	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_device_starts_only_on_a_card_this_process_booked_and_then_counts_the_driver_share_as_on_the_card	the_device_starts_only_on_a_card_this_process_booked_and_then_counts_the_driver_share_as_on_the_card
第10x批 显存账 借据丢掉不销账	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .remove(&self.lease_identifier);	            .get(&self.lease_identifier)\n            .copied();	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_is_on_the_books_until_it_is_dropped	a_lease_is_on_the_books_until_it_is_dropped
第10x批 显存账 按状态数借时照空着时能借的压（借超此刻能借的）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    room.room_now_mebibytes,\n                    &|states| {	                    room.ceiling_mebibytes,\n                    &|states| {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_scratch_booking_compresses_its_states_to_the_room_now_and_books_them	a_scratch_booking_compresses_its_states_to_the_room_now_and_books_them
第10x批 显存账 调用方压过之后外人占了卡，借的时候不照现读的重压	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    room.room_now_mebibytes,\n                    &|states| {	                    u64::MAX,\n                    &|states| {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::states_compressed_before_an_outsider_took_memory_are_compressed_again_in_the_booking	states_compressed_before_an_outsider_took_memory_are_compressed_again_in_the_booking
第10x批 显存账 此刻连一个状态都放不下的卡判成空着时也放不下（判红）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                if need_of_one_state_mebibytes > room.ceiling_mebibytes {	                if need_of_one_state_mebibytes > room.room_now_mebibytes {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_need_larger_than_the_card_when_empty_never_fits_and_a_smaller_one_has_no_room_now	a_need_larger_than_the_card_when_empty_never_fits_and_a_smaller_one_has_no_room_now
第10x批 显存账 读到的卡数与池对不上照样借	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if readings.len() != books.len() {	        if false && readings.len() != books.len() {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_unreadable_card_memory_is_refused_and_nothing_is_booked	an_unreadable_card_memory_is_refused_and_nothing_is_booked
第10x批 显存账 一张卡借到了也当整笔借不到	crates/singlefs-harness/src/gpu_memory_ledger.rs	            ScratchAttemptOnACard::Booked { .. } => {\n                return ScratchPlacementDecision::JudgeOnTheCardsBooked;\n            }	            ScratchAttemptOnACard::Booked { .. } => {}	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_second_card_without_room_leaves_every_state_to_the_first_card	a_second_card_without_room_leaves_every_state_to_the_first_card
第10x批 显存账 真账上一张卡借到了也当整笔借不到	crates/singlefs-harness/src/gpu_memory_ledger.rs	            ScratchAttemptOnACard::Booked { .. } => {\n                return ScratchPlacementDecision::JudgeOnTheCardsBooked;\n            }	            ScratchAttemptOnACard::Booked { .. } => {}	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_states_a_card_cannot_take_now_go_to_the_next_card_with_room	the_states_a_card_cannot_take_now_go_to_the_next_card_with_room
第10x批 显存账 一张都没借到而空着时放得下的也判红（不报本次未跑）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        Some(need_mebibytes) => ScratchPlacementDecision::NoRoomOnAnyCardNow { need_mebibytes },	        Some(_) => ScratchPlacementDecision::NoCardHoldsOneState {\n            largest_ceiling_mebibytes,\n        },	-p singlefs-harness --lib -- gpu_memory_ledger::tests::with_no_card_booked_a_card_that_fits_when_empty_means_no_room_now_and_none_means_red	with_no_card_booked_a_card_that_fits_when_empty_means_no_room_now_and_none_means_red
第10x批 显存账 此刻借不到判红（不报本次未跑）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        } => ScratchBookingRefusalOutcome::NotRun {\n            not_run_line: gpu_not_run_line(	        } => ScratchBookingRefusalOutcome::Red {\n            reason: gpu_not_run_line(	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_booking_without_room_now_is_reported_not_run_with_the_gpu_not_run_marker	a_booking_without_room_now_is_reported_not_run_with_the_gpu_not_run_marker
第10x批 显存账 挑卡从优先级最低的那张起	crates/singlefs-harness/src/gpu_memory_ledger.rs	        .position(|room| need_mebibytes <= room.room_now_mebibytes)	        .rposition(|room| need_mebibytes <= room.room_now_mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::bookings_fill_the_first_card_before_the_next_card_takes_any	bookings_fill_the_first_card_before_the_next_card_takes_any
第10x批 显存账 比每张卡空着时都大的当成此刻借不到	crates/singlefs-harness/src/gpu_memory_ledger.rs	    if need_mebibytes > largest_ceiling_mebibytes {	    if false && need_mebibytes > largest_ceiling_mebibytes {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::bookings_fill_the_first_card_before_the_next_card_takes_any	bookings_fill_the_first_card_before_the_next_card_takes_any
第10x批 显存账 空着时能借的照此刻能占的算（外人占着时判成比卡大）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        ceiling_mebibytes: capacity_of_memory_this_process_may_hold(reading.total_mebibytes),	        ceiling_mebibytes: capacity_of_memory_this_process_may_hold(may_hold_mebibytes),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_room_now_subtracts_the_leases_and_the_ceiling_takes_the_total_memory	the_room_now_subtracts_the_leases_and_the_ceiling_takes_the_total_memory
第10x批 崩溃放量 K4-1 第 ① 段草稿区借了又在判之前还掉	crates/singlefs-checker-tier/src/crash_amplification.rs	                    scratch.leases.push(lease);	                    drop(lease);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_judges_on_scratch_it_booked_on_the_card_pool	the_product_path_judges_on_scratch_it_booked_on_the_card_pool
第10x批 崩溃放量 K4-1 单元校验和对拍的缓冲借了又在对拍之前还掉	crates/singlefs-checker-tier/src/crash_amplification.rs	                            buffers_booked_mebibytes: Some(lease.mebibytes()),\n                            lease: Some(lease),	                            buffers_booked_mebibytes: Some(lease.mebibytes()),\n                            lease: None,	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool	the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool
第10x批 崩溃放量 K4-1 第 ② 段核对的缓冲借了当场还掉	crates/singlefs-checker-tier/src/crash_amplification.rs	                Ok(lease) => (std::sync::Arc::new(lease.card().clone()), Some(lease)),	                Ok(lease) => (std::sync::Arc::new(lease.card().clone()), None),	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool	the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool
第10x批 源码扫描 K4-2 原始 C 字符串不认（cr#"…"# 之后的起卡调用绕得过）	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	            ('r' | 'b' | 'c', Some(_)) if !follows_an_identifier => {	            ('r' | 'b', Some(_)) if !follows_an_identifier => {	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- a_card_taking_call_after_a_raw_c_string_on_the_same_line_is_caught	a_card_taking_call_after_a_raw_c_string_on_the_same_line_is_caught
第10x批 源码扫描 K4-2 项的签名里的逗号就当收尾（泛型参数、where 子句之后的函数体不扫）	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	            ',' if depth == 0 && !is_an_item => return (here, None),	            ',' if depth == 0 => return (here, None),	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- test_code_whose_signature_holds_commas_is_scanned_to_the_end_of_its_body	test_code_whose_signature_holds_commas_is_scanned_to_the_end_of_its_body
第10x批 源码扫描 K4-2 库源码里的裸 #[test] 不当测试代码	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	    if trimmed == "test" {\n        return true;\n    }	    if trimmed == "test" && false {\n        return true;\n    }	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- a_bare_test_function_in_a_library_source_is_scanned	a_bare_test_function_in_a_library_source_is_scanned
第10x批 源码扫描 K4-2 文件级的 #![cfg(test)] 只当下面那一项	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	        if attribute.is_inner {	        if false && attribute.is_inner {	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- cfg_test_written_off_the_start_of_the_line_or_spaced_or_at_the_file_level_is_caught	cfg_test_written_off_the_start_of_the_line_or_spaced_or_at_the_file_level_is_caught
第10x批 源码扫描 K4-2 cfg 与括号之间带空白的不认	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	    let predicate = after_the_name.trim_start();	    let predicate = after_the_name;	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- cfg_test_written_off_the_start_of_the_line_or_spaced_or_at_the_file_level_is_caught	cfg_test_written_off_the_start_of_the_line_or_spaced_or_at_the_file_level_is_caught
第10x批 源码扫描 K4-2 一行里前一条属性之后不再找（与别的属性同一行的 #[cfg(test)] 不认）	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	            position = attribute.closing_position + 1;\n            continue;	            position = characters[attribute.closing_position..]\n                .iter()\n                .position(|(_, character)| *character == '\n')\n                .map_or(characters.len(), |offset| attribute.closing_position + offset + 1);\n            continue;	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- cfg_test_written_off_the_start_of_the_line_or_spaced_or_at_the_file_level_is_caught	cfg_test_written_off_the_start_of_the_line_or_spaced_or_at_the_file_level_is_caught
第10x批 源码扫描 K4-2 直接用 wgpu 起设备不扫（request_device 不在名字表里）	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	    "request_device",\n	    "request_device_that_no_test_calls",\n	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- a_test_that_starts_a_device_straight_through_wgpu_is_caught	a_test_that_starts_a_device_straight_through_wgpu_is_caught
第10x批 源码扫描 K4-2 测试代码里 mod 名字; 引进来的另一份文件不扫	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	                .filter_map(|module_name| out_of_line_module_file(&path, module_name)),	                .filter_map(|_module_name| None::<PathBuf>),	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- a_test_module_kept_in_another_file_is_scanned_whole	a_test_module_kept_in_another_file_is_scanned_whole
第10x批 崩溃放量 卡上报 Out of Memory 当成拒判（整趟判红，不报本次未跑）	crates/singlefs-checker-tier/src/crash_amplification.rs	                received.out_of_memory.get_or_insert(reason);	                received.refusal.get_or_insert(reason);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::an_out_of_memory_on_a_card_is_kept_apart_from_a_refusal_and_the_blocks_judged_before_it_are_stored	an_out_of_memory_on_a_card_is_kept_apart_from_a_refusal_and_the_blocks_judged_before_it_are_stored
````

## mutations-replacements.tsv

````text
第10v批 显存账 本次未跑那一行的记号与 GPU 用例的不同	crates/singlefs-harness/src/gpu_memory_ledger.rs	pub const GPU_NOT_RUN_MARKER: &str = "NOT_RUN_NO_GPU_ADAPTER";	pub const GPU_NOT_RUN_MARKER: &str = "NOT_RUN_SCRATCH";	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_booking_without_room_now_is_reported_not_run_with_the_gpu_not_run_marker	a_booking_without_room_now_is_reported_not_run_with_the_gpu_not_run_marker
第10v批 显存账 点名一张卡借时照优先级挑卡	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.book_on_one_of_these_cards(&[card_position], need_mebibytes)	        self.book_on_the_first_card_with_room(need_mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_booking_on_a_named_card_stays_on_that_card_even_when_an_earlier_card_has_room	a_booking_on_a_named_card_stays_on_that_card_even_when_an_earlier_card_has_room
第10t批 显存账 K4-3 借到了也不登记这个进程（驱动那一份不进账）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::NotRegistered => {\n                self.driver_share = DriverShareOnTheCard::RegisteredDeviceNotStarted;\n            }	            DriverShareOnTheCard::NotRegistered => {}	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_device_starts_only_on_a_card_this_process_booked_and_then_counts_the_driver_share_as_on_the_card	the_device_starts_only_on_a_card_this_process_booked_and_then_counts_the_driver_share_as_on_the_card
第10w批 崩溃放量 起过卡池的产品路径第 ① 段草稿区改回不借	crates/singlefs-checker-tier/src/crash_amplification.rs	        match scratch_of_each_card_booked_by_where_the_cards_came_from(\n            cards_taken_from,\n            cards,\n            &run_cards,	        match scratch_of_each_card_booked_by_where_the_cards_came_from(\n            GpuCardsSource::DevicesStartedForThisRun,\n            cards,\n            &run_cards,	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_judges_on_scratch_it_booked_on_the_card_pool	the_product_path_judges_on_scratch_it_booked_on_the_card_pool
第10w批 崩溃放量 起过卡池的产品路径单元校验和对拍的缓冲改回不借	crates/singlefs-checker-tier/src/crash_amplification.rs	            cards.as_ref(),\n            gpu_cards_taken_from,\n            flow,	            cards.as_ref(),\n            GpuCardsSource::DevicesStartedForThisRun,\n            flow,	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool	the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool
````

## mutations-delete.txt

````text
GPU 显存账 挑卡从优先级最低的那张起
GPU 显存账 比每张卡都大的也去等
GPU 显存账 还了不叫醒等着借的
第10s批 显存账 只算自己这个进程借的、看不见别的进程借的
第10s批 显存账 等满最长的等待也不放弃
第10s批 显存账 记账进程不在了它借的那一份照算
第10s批 显存账 判活不比启动时刻（进程号被重用也当原来那个还在）
第10s批 显存账 默认工作根少了 work 那一层（与 lib_work_root.py 的 DEFAULT_WORK_ROOT 对不上）
第10t批 显存账 K4-2 被杀而没收尸的僵尸判成活着
第10t批 显存账 K4-2 开账的上界不算登记进程的驱动那一份（后读的进程把先登记的驱动那一份记两次）
第10t批 显存账 K4-3 估计没有上界（外人后来占了卡，旧估计照用）
第10t批 显存账 K4-3 估计没有下界（外人后来走了，旧估计照用）
第10t批 显存账 K4-3 估计只取现读的空闲（卡被订满时后起的进程判成放不下）
第10t批 显存账 K4-3 估计照抄旧的、不夹进现读的区间
第10v批 显存账 产品路径借草稿区等满了判红（不报本次未跑）
第10w批 显存账 一张卡借到了也等借不到的那张
第10w批 显存账 真账本上借草稿区一张借到了也等被订满的那张
第10w批 显存账 一张都没借到而空着时放得下的也判红（不等）
第10w批 显存账 估计在压与借之间变小也不在锁里重压
第10w批 显存账 登记驱动那一份要每张都登记上才交回（被订满的那张也等）
````
