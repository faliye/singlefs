# 根环实现第 10t–10w 批：代码轮第一轮改动全文（2026-10-07）

四份补丁照实现员交的 `crates.patch` 原样（合入次序 10t → 删除线第 32 批 P → 10u → 10v → 删除线第 33 批 R → 10w）。主 agent 合入 10u 时改了 `crash_judge_dispatch.rs` 的三个内核钉值与 `image.rs` 一句文档，那两段在末尾，是 `~/.local/share/singlefs/work/m3-r1/merge-10u-10v/before/` 与冻结副本之差（冻结副本里这两份之后没被别人改过）。

## 第 10t 批

````diff
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
index 80099c4..14ca2c9 100644
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -7,10 +7,31 @@
 //! 「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」）。
 //!
 //! 账跨进程：每张卡一份账本文件，放在工作根下的 [`GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME`]，读写前对同名的锁文件 `flock`，
-//! 改完写到临时文件再改名换上（进程在写的半路被杀，账本还是改之前那一份）。账本里记三样：头一个起池的进程读到的空闲显存
-//! （这张卡上没有别的 GPU 测试进程时读的那一份，后起的进程照它算账的容量）、起了池的每个进程（各在卡上付一份驱动的固定开销）、
-//! 每一笔借出去的显存。每条记录带记账进程的进程号与启动时刻，读账时照 [`process_is_alive_by_its_entry_in_the_process_filesystem`] 判活，
-//! 死了的那几条当没记、下一次写账时删掉（进程被杀时它借的显存靠这个回收）。
+//! 改完写到临时文件再改名换上（进程在写的半路被杀，账本还是改之前那一份）。账本里记三样：记账的进程合起来能在这张卡上占多少显存
+//! （[`memory_all_registered_processes_may_hold_mebibytes`] 的估计；那一行的键沿用 `free_memory_after_quiescing_mebibytes`，
+//! 账本的写法与第 10s 批相同，新旧两版的测试二进制读写得了同一份账本）、在这张卡上登记了的每个进程（各在卡上付一份驱动的固定开销）、
+//! 每一笔借出去的显存。每条记录带记账进程的进程号与启动时刻，读账时照 [`process_is_alive_by_its_entry_in_the_process_filesystem`] 判活
+//! （被杀而父进程还没收尸的僵尸也判成不在：驱动在进程退出时就还了它的显存），死了的那几条当没记、下一次写账时删掉
+//! （进程被杀时它借的显存靠这个回收）。
+//!
+//! 估计怎么来（第 10t 批）：每个进程开账时（[`GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock`]，卡池起池时一次）
+//! 在每张卡的锁里现读一次空闲显存 F（这个进程自己在这张卡上还没起设备），同一把锁里数账上活着的进程已经记了的 K
+//! （每一笔借出去的，加每个登记了的进程各一份驱动开销）。记账的进程合起来真能占的是 F 加上「K 里已经实际占在卡上的那部分」A，
+//! 0 ≤ A ≤ K，所以真值落在 [F, F + K] 里：账本里已有前一个进程算的估计，就把它夹进这个区间；没有就取 F。
+//! - 外人（不经这份账的 GPU 用户）后来占了卡：F 变小，上界 F + K 把旧估计压下来（K 都已实际占着时正好压到真值）；
+//! - 外人后来走了：F 变大，下界 F 把旧估计抬上去（把 A 多扣了一遍，偏保守、只会少借）；
+//! - 卡被先起的进程订满：F 接近 0、K 接近整份估计，旧估计还在区间里、照旧，后起的进程等着借，不当成卡不够；
+//! - 两个进程先后开账、后开的那个读到的 F 里已经少了先登记的进程的驱动那一份：那一份在 K 里，上界把它补回来，只记一份。
+//!
+//! 没有活着的记账进程时 K = 0，估计就是现读的 F。之后每一次借都照账本里此刻的估计算容量（[`GpuMemoryLedger::book_waiting_for_room`]），
+//! 别的进程开账时把估计改小了，早开账的进程下一笔也照小的借。
+//! 推的（没上卡量过）：A 读不到，只知道它在 [0, K] 里。夹出来的估计在一种情形下大于真值：外人在两次开账之间占了显存，
+//! 同时账上有借出去而还没在卡上建缓冲的借据（K 里 A 之外的那部分）把上界撑开；大出来的不超过这两样里小的那个。
+//! 驱动开销按常数记：真开销比常数大时上界偏小，只会偏保守。
+//!
+//! 先登记、后建设备：开账只读空闲、不登记。一个进程在一张卡上第一次借（[`GpuMemoryLedger::book_waiting_for_room`]）、
+//! 或只要登记驱动那一份（[`GpuMemoryLedger::register_this_process_on_the_card_waiting_for_room`]）时，这一笔连同它的驱动那一份
+//! 在账上放得下才记、才把这个进程登记到这张卡上；调用方拿到之后才在这张卡上起设备（卡池照这个次序做）。
 
 use std::path::{Path, PathBuf};
 use std::sync::atomic::{AtomicU64, Ordering};
@@ -32,24 +53,53 @@ pub fn states_in_one_dispatch_capped_by_the_state_count(
         .max(1)
 }
 
-/// 一张卡的账能借出去多少（MiB）：设了 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（`card_quota_from_the_environment_mebibytes`）
-/// 就取「这个额度」与「这张卡清场后的空闲显存减驱动的固定开销」里小的那个；没设照旧是这张卡给崩溃放量的额度（`budget_mebibytes`，
-/// 配置的额度或空闲显存的一份）减驱动的固定开销（主 agent 2026-10-06 定：它是每张卡账的上限，不是每条用例的上限，不判红）。
+/// 一张卡的账能借给这个进程多少（MiB，这个进程自己的驱动那一份已经扣掉）。
+/// 设了 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（`card_quota_from_the_environment_mebibytes`）就取三者里最小的：这个额度、
+/// `GPU_CARDS_BY_PRIORITY` 配给这张卡的额度减驱动的固定开销（`configured_card_quota_mebibytes`，没配给这张卡额度的没有这一项）、
+/// 记账进程合起来能占的显存减驱动的固定开销（第 10t 批 K4-1：额度变量不越过配置给卡的额度）；
+/// 没设照旧是这张卡给崩溃放量的额度（`budget_mebibytes`，配置的额度或那份显存的 3/4）减驱动的固定开销
+/// （主 agent 2026-10-06 定：它是每张卡账的上限，不是每条用例的上限，不判红）。
 #[must_use]
 pub fn card_ledger_capacity_mebibytes(
     card_quota_from_the_environment_mebibytes: Option<u64>,
+    configured_card_quota_mebibytes: Option<u64>,
     budget_mebibytes: u64,
-    free_memory_after_quiescing_mebibytes: u64,
+    memory_all_registered_processes_may_hold_mebibytes: u64,
     mebibytes_left_for_the_driver: u64,
 ) -> u64 {
     match card_quota_from_the_environment_mebibytes {
         Some(quota) => quota.min(
-            free_memory_after_quiescing_mebibytes.saturating_sub(mebibytes_left_for_the_driver),
+            configured_card_quota_mebibytes
+                .map_or(u64::MAX, |configured| {
+                    configured.saturating_sub(mebibytes_left_for_the_driver)
+                })
+                .min(
+                    memory_all_registered_processes_may_hold_mebibytes
+                        .saturating_sub(mebibytes_left_for_the_driver),
+                ),
         ),
         None => budget_mebibytes.saturating_sub(mebibytes_left_for_the_driver),
     }
 }
 
+/// 记账的进程合起来能在一张卡上占多少显存（MiB）的新估计（判法在文件头「估计怎么来」）：`free_memory_read_now_mebibytes` 是
+/// 这一刻在这张卡的锁里读的空闲显存，`booked_by_live_processes_mebibytes` 是同一刻账上活着的进程已经记了的（借出去的加每个登记了的
+/// 进程一份驱动开销）；`previous_estimate_mebibytes` 是账本里已有的那一份。真值在 [现读的空闲, 现读的空闲 + 已记的] 里，
+/// 旧估计夹进这个区间，没有旧估计取现读的空闲。
+#[must_use]
+pub fn memory_all_registered_processes_may_hold_mebibytes(
+    previous_estimate_mebibytes: Option<u64>,
+    free_memory_read_now_mebibytes: u64,
+    booked_by_live_processes_mebibytes: u64,
+) -> u64 {
+    let lowest = free_memory_read_now_mebibytes;
+    let highest = free_memory_read_now_mebibytes.saturating_add(booked_by_live_processes_mebibytes);
+    match previous_estimate_mebibytes {
+        Some(previous) => previous.clamp(lowest, highest),
+        None => lowest,
+    }
+}
+
 /// 账上借一次显存的结局（[`decide_booking_by_priority`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum BookingDecision {
@@ -173,14 +223,31 @@ pub fn start_time_in_clock_ticks_from_the_process_status_line(status_line: &str)
     after_the_name.split_whitespace().nth(19)?.parse().ok()
 }
 
-/// 记账的那个进程还在不在：`/proc/<进程号>/stat` 不在、或在而启动时刻对不上（进程号被别的进程重用了）就是不在了；
-/// 读得到文件却认不出、或读的时候出了别的错，当它还在（判不了就照记的算，宁可多等，不把别人还占着的显存算成空的）。
+/// 照 `/proc/<进程号>/stat` 的原文判记账的那个进程还在不在：状态（第 3 项，最后一个 `)` 之后第一项）是 `Z`（退出了或被杀了、
+/// 父进程还没收尸）或 `X`（正在被收走）就是不在了（驱动在进程退出时已经还了它的显存，不必等收尸）；启动时刻对不上
+/// （进程号被别的进程重用了）也是不在了。认不出状态或启动时刻的当它还在（判不了就照记的算，宁可多等）。
+#[must_use]
+pub fn process_is_alive_judged_from_its_status_line(
+    status_line: &str,
+    process: RecordedProcess,
+) -> bool {
+    let state = status_line
+        .rsplit_once(')')
+        .and_then(|(_, after_the_name)| after_the_name.split_whitespace().next());
+    match state {
+        Some("Z" | "X") => false,
+        Some(_) | None => start_time_in_clock_ticks_from_the_process_status_line(status_line)
+            .is_none_or(|start_time| start_time == process.start_time_in_clock_ticks),
+    }
+}
+
+/// 记账的那个进程还在不在：`/proc/<进程号>/stat` 不在就是不在了；在，照 [`process_is_alive_judged_from_its_status_line`] 判；
+/// 读的时候出了别的错，当它还在（判不了就照记的算，宁可多等，不把别人还占着的显存算成空的）。
 #[must_use]
 pub fn process_is_alive_by_its_entry_in_the_process_filesystem(process: RecordedProcess) -> bool {
     let path = format!("/proc/{}/stat", process.process_identifier);
     match std::fs::read_to_string(&path) {
-        Ok(status_line) => start_time_in_clock_ticks_from_the_process_status_line(&status_line)
-            .is_none_or(|start_time| start_time == process.start_time_in_clock_ticks),
+        Ok(status_line) => process_is_alive_judged_from_its_status_line(&status_line, process),
         Err(error) => error.kind() != std::io::ErrorKind::NotFound,
     }
 }
@@ -196,14 +263,15 @@ struct RecordedLease {
 /// 一张卡的账本文件里记的。
 #[derive(Clone, Debug, Default, PartialEq, Eq)]
 struct CardLedgerRecords {
-    /// 头一个起池的进程读到的空闲显存（MiB）：那时这张卡上没有别的进程起着池。
-    free_memory_after_quiescing_mebibytes: Option<u64>,
-    /// 起了池的进程：每个各在卡上起了一个设备。
+    /// 记账的进程合起来能在这张卡上占多少显存（MiB）：最近一个开账的进程照 [`memory_all_registered_processes_may_hold_mebibytes`] 算的。
+    memory_all_registered_processes_may_hold_mebibytes: Option<u64>,
+    /// 在这张卡上登记了的进程：每个各在卡上起了（或马上要起）一个设备。
     registered_processes: Vec<RecordedProcess>,
     leases: Vec<RecordedLease>,
 }
 
-const FREE_MEMORY_LINE_KEY: &str = "free_memory_after_quiescing_mebibytes";
+/// 账本里估计那一行的键：沿用第 10s 批的写法（那时记的是头一个起池的进程读到的空闲显存），新旧两版读写得了同一份账本。
+const MEMORY_ESTIMATE_LINE_KEY: &str = "free_memory_after_quiescing_mebibytes";
 const PROCESS_LINE_KEY: &str = "process";
 const LEASE_LINE_KEY: &str = "lease";
 
@@ -228,8 +296,8 @@ impl CardLedgerRecords {
                 })
             };
             match (fields[0], fields.len()) {
-                (FREE_MEMORY_LINE_KEY, 2) => {
-                    records.free_memory_after_quiescing_mebibytes = Some(number(1)?);
+                (MEMORY_ESTIMATE_LINE_KEY, 2) => {
+                    records.memory_all_registered_processes_may_hold_mebibytes = Some(number(1)?);
                 }
                 (PROCESS_LINE_KEY, 3) => records.registered_processes.push(process(1)?),
                 (LEASE_LINE_KEY, 5) => records.leases.push(RecordedLease {
@@ -245,8 +313,8 @@ impl CardLedgerRecords {
 
     fn to_text(&self) -> String {
         let mut text = String::new();
-        if let Some(free) = self.free_memory_after_quiescing_mebibytes {
-            text.push_str(&format!("{FREE_MEMORY_LINE_KEY} {free}\n"));
+        if let Some(estimate) = self.memory_all_registered_processes_may_hold_mebibytes {
+            text.push_str(&format!("{MEMORY_ESTIMATE_LINE_KEY} {estimate}\n"));
         }
         for process in &self.registered_processes {
             text.push_str(&format!(
@@ -277,12 +345,12 @@ impl CardLedgerRecords {
         self
     }
 
-    /// `viewer` 这个进程看这张卡已经记了多少（MiB）：每一笔借出去的，加别的每个起了池的进程各一份驱动的固定开销
+    /// `viewer` 这个进程看这张卡已经记了多少（MiB）：每一笔借出去的，加别的每个登记了的进程各一份驱动的固定开销
     /// （`viewer` 自己那一份已经从它的容量里扣过）。
     fn booked_mebibytes_seen_by(
         &self,
         viewer: RecordedProcess,
-        mebibytes_each_other_process_keeps_on_a_card: u64,
+        mebibytes_each_process_keeps_on_a_card: u64,
     ) -> u64 {
         let other_processes = self
             .registered_processes
@@ -294,10 +362,25 @@ impl CardLedgerRecords {
             .map(|lease| lease.mebibytes)
             .sum::<u64>()
             .saturating_add(
-                mebibytes_each_other_process_keeps_on_a_card
+                mebibytes_each_process_keeps_on_a_card
                     .saturating_mul(u64::try_from(other_processes).expect("进程数装得进 u64")),
             )
     }
+
+    /// 账上活着的进程一共记了多少（MiB）：`viewer` 看到的，加上 `viewer` 自己登记了时它的那一份驱动开销。
+    fn booked_mebibytes_by_every_live_process(
+        &self,
+        viewer: RecordedProcess,
+        mebibytes_each_process_keeps_on_a_card: u64,
+    ) -> u64 {
+        let own_share = if self.registered_processes.contains(&viewer) {
+            mebibytes_each_process_keeps_on_a_card
+        } else {
+            0
+        };
+        self.booked_mebibytes_seen_by(viewer, mebibytes_each_process_keeps_on_a_card)
+            .saturating_add(own_share)
+    }
 }
 
 /// 账本文件读写出错：哪个文件、什么错。
@@ -307,21 +390,26 @@ pub struct LedgerFileError {
     pub reason: String,
 }
 
-/// 记账的设置：账本放哪、这个进程是谁、怎么判记账进程还在不在（测试换成假的）、别的进程各在卡上付多少驱动的固定开销、
+/// 记账的设置：账本放哪、这个进程是谁、怎么判记账进程还在不在（测试换成假的）、每个进程各在卡上付多少驱动的固定开销、
 /// 等别的进程还显存时隔多久重读、最多等多久。
 pub struct LedgerSettings {
     pub directory: PathBuf,
     pub this_process: RecordedProcess,
     pub is_alive: Box<dyn Fn(RecordedProcess) -> bool + Send + Sync>,
-    pub mebibytes_each_other_process_keeps_on_a_card: u64,
+    pub mebibytes_each_process_keeps_on_a_card: u64,
     pub poll_interval_for_memory_returned_by_other_processes: Duration,
     pub longest_wait: Duration,
 }
 
-/// 要记账的一张卡：账本文件名的主干（只许字母、数字、`.`、`-`）与这个进程起池时读到的空闲显存。
-pub struct CardToRegister {
-    pub file_stem: String,
-    pub free_memory_read_by_this_process_mebibytes: u64,
+/// 一张卡看这个进程的容量（MiB）：`(卡在账里的次序, 账本里此刻的估计)` → 容量（这个进程自己的驱动那一份已经扣掉，
+/// 卡池照 [`card_ledger_capacity_mebibytes`] 算）。
+pub type CardCapacityForThisProcess = Box<dyn Fn(usize, u64) -> u64 + Send + Sync>;
+
+/// 开账不成：账本文件读写不了，或那张卡的空闲显存读不出（哪张卡的账本、为什么）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum LedgerOpeningRefusal {
+    LedgerFileUnusable(LedgerFileError),
+    FreeMemoryUnreadable { file_stem: String, reason: String },
 }
 
 /// 借不到的几种结局。
@@ -329,20 +417,54 @@ pub struct CardToRegister {
 pub enum BookingRefusal {
     /// 要的比每张卡的容量都大：调用方判红。
     NeedLargerThanEveryCard(NeedLargerThanEveryCard),
-    /// 放得下而等满了 [`LedgerSettings::longest_wait`] 还没空出来：调用方报「本次未跑」。
+    /// 放得下而等满了 [`LedgerSettings::longest_wait`] 还没空出来：调用方报「本次未跑」。带着最后一次判的时候每张卡的容量。
     WaitedPastTheLongestWait {
         need_mebibytes: u64,
         waited: Duration,
+        card_capacities_mebibytes: Vec<u64>,
     },
     /// 账本文件读写不了：调用方判红。
     LedgerFileUnusable(LedgerFileError),
 }
 
+/// 一次记账记什么。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum WhatIsBooked {
+    /// 一笔借据；这个进程在那张卡上还没登记的，连它的驱动那一份一起登记。
+    LeaseOf { need_mebibytes: u64 },
+    /// 不借，只把这个进程登记到那张卡上（记它的驱动那一份）。
+    DriverShareOnly,
+}
+
+/// 记在哪几张卡里挑。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum WhichCards {
+    /// 照优先级第一张放得下的。
+    FirstThatFitsByPriority,
+    /// 只这一张。
+    OnlyTheCardAt { card_position: usize },
+}
+
+/// 记上的那一笔：在账里第几张卡、借据号（只登记驱动那一份的没有）、记的那一刻这张卡看这个进程的容量。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct BookedOnACard {
+    card_position: usize,
+    lease_identifier: Option<u64>,
+    capacity_mebibytes: u64,
+}
+
+/// 一张卡此刻看这个进程：容量与已经记了的（都在这张卡的锁里读）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct CardSeenByThisProcess {
+    capacity_mebibytes: u64,
+    booked_mebibytes: u64,
+}
+
 /// 几张卡各一本跨进程的显存账（MiB）：借的时候照 [`decide_booking_by_priority`] 记上，放不下就等别的借据还，借据丢掉时销账。
 pub struct GpuMemoryLedger {
     settings: LedgerSettings,
     card_file_stems: Vec<String>,
-    capacities_mebibytes: Vec<u64>,
+    capacity_for_this_process: CardCapacityForThisProcess,
     next_lease_identifier: AtomicU64,
     /// 这个进程里还过几次：等着的线程记下看到的次数，次数变了就不睡（还在它判完、睡下之前时不漏叫）。
     returns_in_this_process: Mutex<u64>,
@@ -397,108 +519,192 @@ fn with_the_card_ledger_locked<Outcome>(
     Ok(outcome)
 }
 
+/// 账本文件名的主干只许字母、数字、`.`、`-`。
+fn is_a_ledger_file_stem(file_stem: &str) -> bool {
+    !file_stem.is_empty()
+        && file_stem.chars().all(|character| {
+            character.is_ascii_alphanumeric() || character == '.' || character == '-'
+        })
+}
+
 impl GpuMemoryLedger {
-    /// 这个进程在每张卡的账本上记一笔「起了池」，交回账。这张卡上此刻没有别的进程起着池时，账本记下这个进程读到的空闲显存
-    /// （就是清场之后那一份）；有，就照账本里记的那一份。`capacity_for_judges(卡的次序, 那一份空闲显存)` 是这个进程看这张卡的容量。
+    /// 开这个进程的账：在每张卡（`card_file_stems`，次序就是优先级）的账本锁里调 `read_free_memory_mebibytes(卡的次序)` 现读一次空闲显存，
+    /// 照 [`memory_all_registered_processes_may_hold_mebibytes`] 改账本里的估计；不登记这个进程（第一次借的时候才登记）。
+    /// 之后每一次借，容量都照 `capacity_for_this_process(卡的次序, 账本里此刻的估计)`。
     ///
     /// # Errors
-    /// 账本文件读写不了、认不出。
+    /// 账本文件读写不了、认不出；某张卡的空闲显存读不出。
     ///
     /// # Panics
     /// 文件名主干里有字母、数字、`.`、`-` 之外的字符。
-    pub fn register_this_process_on_the_cards(
+    pub fn open_reading_the_free_memory_inside_each_card_lock(
         settings: LedgerSettings,
-        cards: &[CardToRegister],
-        capacity_for_judges: impl Fn(usize, u64) -> u64,
-    ) -> Result<Self, LedgerFileError> {
-        let mut capacities_mebibytes = Vec::with_capacity(cards.len());
-        for (position, card) in cards.iter().enumerate() {
+        card_file_stems: Vec<String>,
+        read_free_memory_mebibytes: &dyn Fn(usize) -> Result<u64, String>,
+        capacity_for_this_process: CardCapacityForThisProcess,
+    ) -> Result<Self, LedgerOpeningRefusal> {
+        for (position, file_stem) in card_file_stems.iter().enumerate() {
             assert!(
-                !card.file_stem.is_empty()
-                    && card.file_stem.chars().all(|character| {
-                        character.is_ascii_alphanumeric() || character == '.' || character == '-'
-                    }),
-                "账本文件名主干只许字母、数字、`.`、`-`：「{}」",
-                card.file_stem
+                is_a_ledger_file_stem(file_stem),
+                "账本文件名主干只许字母、数字、`.`、`-`：「{file_stem}」"
             );
             let this_process = settings.this_process;
-            let free_after_quiescing =
-                with_the_card_ledger_locked(&settings, &card.file_stem, |records| {
-                    records
-                        .registered_processes
-                        .retain(|process| *process != this_process);
-                    if records.registered_processes.is_empty()
-                        || records.free_memory_after_quiescing_mebibytes.is_none()
-                    {
-                        records.free_memory_after_quiescing_mebibytes =
-                            Some(card.free_memory_read_by_this_process_mebibytes);
-                    }
-                    records.registered_processes.push(this_process);
-                    records
-                        .free_memory_after_quiescing_mebibytes
-                        .expect("上面刚记过")
-                })?;
-            capacities_mebibytes.push(capacity_for_judges(position, free_after_quiescing));
+            let driver = settings.mebibytes_each_process_keeps_on_a_card;
+            with_the_card_ledger_locked(&settings, file_stem, |records| {
+                let free_now = read_free_memory_mebibytes(position)?;
+                records.memory_all_registered_processes_may_hold_mebibytes =
+                    Some(memory_all_registered_processes_may_hold_mebibytes(
+                        records.memory_all_registered_processes_may_hold_mebibytes,
+                        free_now,
+                        records.booked_mebibytes_by_every_live_process(this_process, driver),
+                    ));
+                Ok(())
+            })
+            .map_err(LedgerOpeningRefusal::LedgerFileUnusable)?
+            .map_err(|reason| LedgerOpeningRefusal::FreeMemoryUnreadable {
+                file_stem: file_stem.clone(),
+                reason,
+            })?;
         }
         Ok(Self {
             settings,
-            card_file_stems: cards.iter().map(|card| card.file_stem.clone()).collect(),
-            capacities_mebibytes,
+            card_file_stems,
+            capacity_for_this_process,
             next_lease_identifier: AtomicU64::new(0),
             returns_in_this_process: Mutex::new(0),
             memory_returned: Condvar::new(),
         })
     }
 
-    /// 这个进程看每张卡的容量（MiB），次序就是优先级。
-    #[must_use]
-    pub fn card_capacities_mebibytes(&self) -> &[u64] {
-        &self.capacities_mebibytes
+    /// 这张卡此刻看这个进程的容量与已记的（在锁里读）。账本里没有估计（开账之后被人删了）当容量 0：借什么都放不下，调用方判红。
+    fn card_seen_by_this_process(
+        records: &CardLedgerRecords,
+        settings: &LedgerSettings,
+        capacity_for_this_process: &CardCapacityForThisProcess,
+        card_position: usize,
+    ) -> CardSeenByThisProcess {
+        CardSeenByThisProcess {
+            capacity_mebibytes: records
+                .memory_all_registered_processes_may_hold_mebibytes
+                .map_or(0, |estimate| {
+                    capacity_for_this_process(card_position, estimate)
+                }),
+            booked_mebibytes: records.booked_mebibytes_seen_by(
+                settings.this_process,
+                settings.mebibytes_each_process_keeps_on_a_card,
+            ),
+        }
     }
 
-    /// 每张卡此刻已经记了多少（照 [`CardLedgerRecords::booked_mebibytes_seen_by`]）。
-    fn booked_on_every_card(&self) -> Result<Vec<u64>, LedgerFileError> {
-        self.card_file_stems
+    /// 这几张卡此刻各看这个进程的容量与已记的。
+    fn cards_seen_by_this_process(
+        &self,
+        card_positions: &[usize],
+    ) -> Result<Vec<CardSeenByThisProcess>, LedgerFileError> {
+        card_positions
             .iter()
-            .map(|file_stem| {
-                with_the_card_ledger_locked(&self.settings, file_stem, |records| {
-                    records.booked_mebibytes_seen_by(
-                        self.settings.this_process,
-                        self.settings.mebibytes_each_other_process_keeps_on_a_card,
-                    )
-                })
+            .map(|card_position| {
+                with_the_card_ledger_locked(
+                    &self.settings,
+                    &self.card_file_stems[*card_position],
+                    |records| {
+                        Self::card_seen_by_this_process(
+                            records,
+                            &self.settings,
+                            &self.capacity_for_this_process,
+                            *card_position,
+                        )
+                    },
+                )
             })
             .collect()
     }
 
-    /// 锁住这张卡的账本重读一遍，此刻还放得下就记上，交回借据号；读完到锁上之间别的进程记了一笔、放不下了，交 `None`。
+    /// 这个进程此刻看每张卡的容量（MiB），次序就是优先级；照账本里此刻的估计算。
+    ///
+    /// # Errors
+    /// 账本文件读写不了、认不出。
+    pub fn card_capacities_mebibytes(&self) -> Result<Vec<u64>, LedgerFileError> {
+        let every_card: Vec<usize> = (0..self.card_file_stems.len()).collect();
+        Ok(self
+            .cards_seen_by_this_process(&every_card)?
+            .iter()
+            .map(|card| card.capacity_mebibytes)
+            .collect())
+    }
+
+    /// 这个进程在这张卡上登记了没有（第一次在它上面借、或 [`Self::register_this_process_on_the_card_waiting_for_room`] 之后就登记了）。
+    ///
+    /// # Errors
+    /// 账本文件读写不了、认不出。
+    pub fn is_this_process_registered_on_the_card(
+        &self,
+        card_position: usize,
+    ) -> Result<bool, LedgerFileError> {
+        with_the_card_ledger_locked(
+            &self.settings,
+            &self.card_file_stems[card_position],
+            |records| {
+                records
+                    .registered_processes
+                    .contains(&self.settings.this_process)
+            },
+        )
+    }
+
+    /// 锁住这张卡的账本重读一遍，此刻还放得下就记上（这个进程在这张卡上还没登记的一并登记），交回记上的那一笔；
+    /// 读完到锁上之间别的进程记了一笔、放不下了，交 `None`。
     fn book_on_the_card_if_it_still_fits(
         &self,
         card_position: usize,
-        need_mebibytes: u64,
-    ) -> Result<Option<u64>, LedgerFileError> {
+        what: WhatIsBooked,
+    ) -> Result<Option<BookedOnACard>, LedgerFileError> {
+        let need_mebibytes = match what {
+            WhatIsBooked::LeaseOf { need_mebibytes } => need_mebibytes,
+            WhatIsBooked::DriverShareOnly => 0,
+        };
         with_the_card_ledger_locked(
             &self.settings,
             &self.card_file_stems[card_position],
             |records| {
-                let booked = records.booked_mebibytes_seen_by(
-                    self.settings.this_process,
-                    self.settings.mebibytes_each_other_process_keeps_on_a_card,
+                let seen = Self::card_seen_by_this_process(
+                    records,
+                    &self.settings,
+                    &self.capacity_for_this_process,
+                    card_position,
                 );
                 match decide_booking_by_priority(
-                    &[self.capacities_mebibytes[card_position]],
-                    &[booked],
+                    &[seen.capacity_mebibytes],
+                    &[seen.booked_mebibytes],
                     need_mebibytes,
                 ) {
                     BookingDecision::BookOnTheCardAt { .. } => {
-                        let lease_identifier =
-                            self.next_lease_identifier.fetch_add(1, Ordering::Relaxed);
-                        records.leases.push(RecordedLease {
-                            process: self.settings.this_process,
+                        let lease_identifier = match what {
+                            WhatIsBooked::LeaseOf { .. } => {
+                                let lease_identifier =
+                                    self.next_lease_identifier.fetch_add(1, Ordering::Relaxed);
+                                records.leases.push(RecordedLease {
+                                    process: self.settings.this_process,
+                                    lease_identifier,
+                                    mebibytes: need_mebibytes,
+                                });
+                                Some(lease_identifier)
+                            }
+                            WhatIsBooked::DriverShareOnly => None,
+                        };
+                        if !records
+                            .registered_processes
+                            .contains(&self.settings.this_process)
+                        {
+                            records
+                                .registered_processes
+                                .push(self.settings.this_process);
+                        }
+                        Some(BookedOnACard {
+                            card_position,
                             lease_identifier,
-                            mebibytes: need_mebibytes,
-                        });
-                        Some(lease_identifier)
+                            capacity_mebibytes: seen.capacity_mebibytes,
+                        })
                     }
                     BookingDecision::WaitForMemoryToBeReturned
                     | BookingDecision::NeverFits { .. } => None,
@@ -507,16 +713,22 @@ impl GpuMemoryLedger {
         )
     }
 
-    /// 借 `need_mebibytes`：记得上就交回借据（丢掉时销账）；此刻放不下就等别的借据还（这个进程里还的当场叫醒，
+    /// 照 `which` 挑卡记 `what`：记得上就交回记上的那一笔；此刻放不下就等别的借据还（这个进程里还的当场叫醒，
     /// 别的进程还的隔 [`LedgerSettings::poll_interval_for_memory_returned_by_other_processes`] 重读一次看到），
     /// 等满 [`LedgerSettings::longest_wait`] 还放不下就放弃；永远放不下交错误、什么都不记。
-    ///
-    /// # Errors
-    /// 见 [`BookingRefusal`]。
-    pub fn book_waiting_for_room(
+    fn book_waiting(
         &self,
-        need_mebibytes: u64,
-    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
+        which: WhichCards,
+        what: WhatIsBooked,
+    ) -> Result<BookedOnACard, BookingRefusal> {
+        let need_mebibytes = match what {
+            WhatIsBooked::LeaseOf { need_mebibytes } => need_mebibytes,
+            WhatIsBooked::DriverShareOnly => 0,
+        };
+        let card_positions: Vec<usize> = match which {
+            WhichCards::FirstThatFitsByPriority => (0..self.card_file_stems.len()).collect(),
+            WhichCards::OnlyTheCardAt { card_position } => vec![card_position],
+        };
         let started = Instant::now();
         loop {
             // 计数器上只做加一，持锁的那一段不会 panic 到一半：别的线程 panic 留下的毒不影响它，照用
@@ -524,21 +736,18 @@ impl GpuMemoryLedger {
                 .returns_in_this_process
                 .lock()
                 .unwrap_or_else(PoisonError::into_inner);
-            let booked = self
-                .booked_on_every_card()
+            let cards = self
+                .cards_seen_by_this_process(&card_positions)
                 .map_err(BookingRefusal::LedgerFileUnusable)?;
-            match decide_booking_by_priority(&self.capacities_mebibytes, &booked, need_mebibytes) {
+            let capacities: Vec<u64> = cards.iter().map(|card| card.capacity_mebibytes).collect();
+            let booked: Vec<u64> = cards.iter().map(|card| card.booked_mebibytes).collect();
+            match decide_booking_by_priority(&capacities, &booked, need_mebibytes) {
                 BookingDecision::BookOnTheCardAt { card_position } => {
-                    if let Some(lease_identifier) = self
-                        .book_on_the_card_if_it_still_fits(card_position, need_mebibytes)
+                    if let Some(booked_on_the_card) = self
+                        .book_on_the_card_if_it_still_fits(card_positions[card_position], what)
                         .map_err(BookingRefusal::LedgerFileUnusable)?
                     {
-                        return Ok(GpuMemoryBooking {
-                            ledger: self,
-                            card_position,
-                            mebibytes: need_mebibytes,
-                            lease_identifier,
-                        });
+                        return Ok(booked_on_the_card);
                     }
                 }
                 BookingDecision::WaitForMemoryToBeReturned => {
@@ -547,6 +756,7 @@ impl GpuMemoryLedger {
                         return Err(BookingRefusal::WaitedPastTheLongestWait {
                             need_mebibytes,
                             waited,
+                            card_capacities_mebibytes: capacities,
                         });
                     }
                     let returns = self
@@ -579,6 +789,52 @@ impl GpuMemoryLedger {
         }
     }
 
+    /// 借 `need_mebibytes`：照卡的优先级记在第一张放得下的卡上（这个进程在那张卡上还没登记的，它的驱动那一份也要放得下，一并登记），
+    /// 记得上就交回借据（丢掉时销账）；放不下就等，等满放弃；永远放不下交错误、什么都不记（同 [`Self::book_waiting`]）。
+    ///
+    /// # Errors
+    /// 见 [`BookingRefusal`]。
+    pub fn book_waiting_for_room(
+        &self,
+        need_mebibytes: u64,
+    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
+        let booked = self.book_waiting(
+            WhichCards::FirstThatFitsByPriority,
+            WhatIsBooked::LeaseOf { need_mebibytes },
+        )?;
+        Ok(GpuMemoryBooking {
+            ledger: self,
+            card_position: booked.card_position,
+            mebibytes: need_mebibytes,
+            lease_identifier: booked
+                .lease_identifier
+                .expect("记的是一笔借据（WhatIsBooked::LeaseOf），记上就有借据号"),
+            card_capacity_mebibytes_when_booked: booked.capacity_mebibytes,
+        })
+    }
+
+    /// 把这个进程登记到第 `card_position` 张卡上（记它的驱动那一份），不借：已经登记了的当场交回；没登记的要账上放得下这一份才登记，
+    /// 放不下就等、等满放弃（同 [`Self::book_waiting`]）。调用方拿到 `Ok` 之后才在这张卡上起设备。
+    ///
+    /// # Errors
+    /// 见 [`BookingRefusal`]。
+    pub fn register_this_process_on_the_card_waiting_for_room(
+        &self,
+        card_position: usize,
+    ) -> Result<(), BookingRefusal> {
+        if self
+            .is_this_process_registered_on_the_card(card_position)
+            .map_err(BookingRefusal::LedgerFileUnusable)?
+        {
+            return Ok(());
+        }
+        self.book_waiting(
+            WhichCards::OnlyTheCardAt { card_position },
+            WhatIsBooked::DriverShareOnly,
+        )
+        .map(|_registered| ())
+    }
+
     /// 从这张卡的账本上删掉这个进程的这一笔。
     fn remove_the_lease(
         &self,
@@ -598,12 +854,13 @@ impl GpuMemoryLedger {
     }
 }
 
-/// 账上的一笔：记在哪张卡、多少 MiB；丢掉时销账、叫醒这个进程里等着借的。
+/// 账上的一笔：记在哪张卡、多少 MiB、记的那一刻这张卡看这个进程的容量；丢掉时销账、叫醒这个进程里等着借的。
 pub struct GpuMemoryBooking<'ledger> {
     ledger: &'ledger GpuMemoryLedger,
     card_position: usize,
     mebibytes: u64,
     lease_identifier: u64,
+    card_capacity_mebibytes_when_booked: u64,
 }
 
 impl GpuMemoryBooking<'_> {
@@ -617,6 +874,12 @@ impl GpuMemoryBooking<'_> {
     pub fn mebibytes(&self) -> u64 {
         self.mebibytes
     }
+
+    /// 记的那一刻这张卡看这个进程的容量（MiB）。
+    #[must_use]
+    pub fn card_capacity_mebibytes_when_booked(&self) -> u64 {
+        self.card_capacity_mebibytes_when_booked
+    }
 }
 
 impl Drop for GpuMemoryBooking<'_> {
@@ -645,14 +908,19 @@ impl Drop for GpuMemoryBooking<'_> {
 mod tests {
     use super::{
         card_ledger_capacity_mebibytes, decide_booking_by_priority, gpu_memory_ledger_directory,
+        memory_all_registered_processes_may_hold_mebibytes,
         process_is_alive_by_its_entry_in_the_process_filesystem,
+        start_time_in_clock_ticks_from_the_process_status_line,
         states_in_one_dispatch_capped_by_the_state_count, BookingDecision, BookingRefusal,
-        CardToRegister, GpuMemoryLedger, LedgerSettings, NeedLargerThanEveryCard, RecordedProcess,
+        GpuMemoryLedger, LedgerSettings, NeedLargerThanEveryCard, RecordedProcess,
         GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME,
     };
     use std::ffi::OsStr;
     use std::path::{Path, PathBuf};
-    use std::time::Duration;
+    use std::time::{Duration, Instant};
+
+    /// 驱动的固定开销，照卡池用的那个数（`crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER`）。
+    const DRIVER: u64 = 2560;
 
     /// 这条用例自己的账本目录（临时目录下，带用例名与进程号）：先删掉上一趟留下的，用例收尾时删。
     struct LedgerDirectory {
@@ -684,37 +952,431 @@ mod tests {
         }
     }
 
-    /// 一个假进程在 `directory` 下记一张卡（文件名主干 `card-test`、容量 `capacity_mebibytes`）的账：判活只认 `alive` 里的进程号，
-    /// 别的进程各在卡上付 `mebibytes_each_other_process_keeps_on_a_card`。
+    /// 一个假进程的记账设置：判活只认 `alive` 里的进程号，别的进程各在卡上付 `mebibytes_each_process_keeps_on_a_card`。
+    fn settings_of_a_fake_process(
+        directory: &Path,
+        process_identifier: u32,
+        alive: &'static [u32],
+        mebibytes_each_process_keeps_on_a_card: u64,
+        poll_interval_for_memory_returned_by_other_processes: Duration,
+        longest_wait: Duration,
+    ) -> LedgerSettings {
+        LedgerSettings {
+            directory: directory.to_path_buf(),
+            this_process: fake_process(process_identifier),
+            is_alive: Box::new(move |process: RecordedProcess| {
+                alive.contains(&process.process_identifier)
+            }),
+            mebibytes_each_process_keeps_on_a_card,
+            poll_interval_for_memory_returned_by_other_processes,
+            longest_wait,
+        }
+    }
+
+    /// 一个假进程在 `directory` 下开一张卡（文件名主干 `card-test`）的账，容量不看估计、恒为 `capacity_mebibytes`
+    /// （只测借与等的用例用它；开账读到的空闲随便给一个数）。
     fn ledger_of_a_fake_process(
         directory: &Path,
         process_identifier: u32,
         alive: &'static [u32],
         capacity_mebibytes: u64,
-        mebibytes_each_other_process_keeps_on_a_card: u64,
+        mebibytes_each_process_keeps_on_a_card: u64,
         poll_interval_for_memory_returned_by_other_processes: Duration,
         longest_wait: Duration,
     ) -> GpuMemoryLedger {
-        GpuMemoryLedger::register_this_process_on_the_cards(
-            LedgerSettings {
-                directory: directory.to_path_buf(),
-                this_process: fake_process(process_identifier),
-                is_alive: Box::new(move |process: RecordedProcess| {
-                    alive.contains(&process.process_identifier)
-                }),
-                mebibytes_each_other_process_keeps_on_a_card,
+        GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
+            settings_of_a_fake_process(
+                directory,
+                process_identifier,
+                alive,
+                mebibytes_each_process_keeps_on_a_card,
                 poll_interval_for_memory_returned_by_other_processes,
                 longest_wait,
-            },
-            &[CardToRegister {
-                file_stem: "card-test".to_string(),
-                free_memory_read_by_this_process_mebibytes: 1000,
-            }],
-            |_position, _free_after_quiescing| capacity_mebibytes,
+            ),
+            vec!["card-test".to_string()],
+            &|_position| Ok(1000),
+            Box::new(move |_position, _estimate| capacity_mebibytes),
+        )
+        .expect("临时目录下的账本读写得了")
+    }
+
+    /// 一个假进程在 `directory` 下开一张卡（文件名主干 `card-test`）的账：开账时读到的空闲是 `free_memory_read_mebibytes`，
+    /// 容量照卡池没配额度、没设额度变量时的算法里不打 3/4 折扣的那一份：账本里的估计减驱动那一份 [`DRIVER`]。
+    fn ledger_reading_the_free_memory(
+        directory: &Path,
+        process_identifier: u32,
+        alive: &'static [u32],
+        free_memory_read_mebibytes: u64,
+        longest_wait: Duration,
+    ) -> GpuMemoryLedger {
+        GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
+            settings_of_a_fake_process(
+                directory,
+                process_identifier,
+                alive,
+                DRIVER,
+                Duration::from_millis(20),
+                longest_wait,
+            ),
+            vec!["card-test".to_string()],
+            &|_position| Ok(free_memory_read_mebibytes),
+            Box::new(|_position, estimate| {
+                card_ledger_capacity_mebibytes(None, None, estimate, estimate, DRIVER)
+            }),
         )
         .expect("临时目录下的账本读写得了")
     }
 
+    /// 借一笔，等不到就放弃（等的上限是开账时给的）：交回借据或借不到的结局。
+    fn booking_outcome(
+        ledger: &GpuMemoryLedger,
+        need_mebibytes: u64,
+    ) -> Result<u64, BookingRefusal> {
+        ledger.book_waiting_for_room(need_mebibytes).map(|booking| {
+            let mebibytes = booking.mebibytes();
+            std::mem::forget(booking);
+            mebibytes
+        })
+    }
+
+    /// 估计是旧估计夹进 [现读的空闲, 现读的空闲 + 活进程已记的]：没有旧估计取现读的；外人占了卡（现读的小）压到上界，
+    /// 外人走了（现读的大）抬到下界，订满了（现读的近 0、已记的近满）照旧。
+    #[test]
+    fn the_memory_estimate_is_the_previous_one_clamped_between_the_free_memory_read_now_and_that_plus_the_live_bookings(
+    ) {
+        assert_eq!(
+            memory_all_registered_processes_may_hold_mebibytes(None, 30_000, 0),
+            30_000,
+            "没有旧估计：取现读的空闲"
+        );
+        assert_eq!(
+            memory_all_registered_processes_may_hold_mebibytes(Some(30_000), 7_340, 2_660),
+            10_000,
+            "外人后来占了 20 000：压到现读的加已记的"
+        );
+        assert_eq!(
+            memory_all_registered_processes_may_hold_mebibytes(Some(10_000), 25_440, 4_560),
+            25_440,
+            "外人后来走了：抬到现读的"
+        );
+        assert_eq!(
+            memory_all_registered_processes_may_hold_mebibytes(Some(30_000), 0, 30_000),
+            30_000,
+            "卡被订满：旧估计在区间里，照旧"
+        );
+    }
+
+    /// 第 10t 批 K4-1：配置给这张卡 8 G、额度变量 20 000：容量不越过配置的额度（减驱动那一份），额度变量更小时取额度变量；
+    /// 没设额度变量照旧。数照一张清场后空 30 000 MiB 的 5090。
+    #[test]
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
+        );
+        assert!(
+            capacity <= CONFIGURED_CARD_QUOTA,
+            "额度变量 20 000 不越过配置给卡的 8 G：实际 {capacity}"
+        );
+        assert_eq!(
+            capacity,
+            CONFIGURED_CARD_QUOTA - DRIVER,
+            "取配置额度减驱动那一份"
+        );
+        assert_eq!(
+            card_ledger_capacity_mebibytes(
+                Some(4_000),
+                Some(CONFIGURED_CARD_QUOTA),
+                budget,
+                30_000,
+                DRIVER
+            ),
+            4_000,
+            "额度变量更小：取额度变量"
+        );
+        assert_eq!(
+            card_ledger_capacity_mebibytes(
+                None,
+                Some(CONFIGURED_CARD_QUOTA),
+                budget,
+                30_000,
+                DRIVER
+            ),
+            CONFIGURED_CARD_QUOTA - DRIVER,
+            "没设额度变量：照旧是给崩溃放量的额度减驱动那一份"
+        );
+    }
+
+    /// 第 10t 批 K4-2：子进程被杀、父进程（跑用例的这个进程）还没收尸时它是僵尸，`/proc/<进程号>/stat` 还在、启动时刻也对得上，
+    /// 判成不在（驱动已经还了它的显存）；收了尸照样不在。
+    #[test]
+    fn a_killed_process_that_its_parent_has_not_reaped_yet_is_judged_not_alive() {
+        let state_of = |process_identifier: u32| -> Option<String> {
+            let status_line =
+                std::fs::read_to_string(format!("/proc/{process_identifier}/stat")).ok()?;
+            let (_, after_the_name) = status_line.rsplit_once(')')?;
+            after_the_name.split_whitespace().next().map(str::to_string)
+        };
+        let mut child = std::process::Command::new("sleep")
+            .arg("600")
+            .spawn()
+            .expect("起得了 sleep");
+        let process_identifier = child.id();
+        let recorded = RecordedProcess {
+            process_identifier,
+            start_time_in_clock_ticks: start_time_in_clock_ticks_from_the_process_status_line(
+                &std::fs::read_to_string(format!("/proc/{process_identifier}/stat"))
+                    .expect("刚起的子进程在 /proc 里"),
+            )
+            .expect("认得出启动时刻"),
+        };
+        assert!(
+            process_is_alive_by_its_entry_in_the_process_filesystem(recorded),
+            "还在跑的子进程判成还在"
+        );
+        child.kill().expect("杀得了自己起的子进程");
+        let deadline = Instant::now() + Duration::from_secs(30);
+        while state_of(process_identifier).as_deref() != Some("Z") {
+            assert!(
+                Instant::now() < deadline,
+                "被杀的子进程 30 秒内没变成僵尸：状态 {:?}",
+                state_of(process_identifier)
+            );
+            std::thread::sleep(Duration::from_millis(10));
+        }
+        assert!(
+            !process_is_alive_by_its_entry_in_the_process_filesystem(recorded),
+            "被杀而没收尸的僵尸判成不在"
+        );
+        child.wait().expect("收得了尸");
+        assert!(
+            !process_is_alive_by_its_entry_in_the_process_filesystem(recorded),
+            "收了尸照样不在"
+        );
+    }
+
+    /// 第 10t 批 K4-2：先开账的进程 100 借了 1 000、起了设备，后开账的进程 200 读到的空闲（26 440）里已经少了进程 100 的
+    /// 驱动那一份与它借的：账照样只记一份。进程 200 能借的正好是真空闲减它自己的驱动那一份（23 880），多 1 MiB 就要等。
+    #[test]
+    fn a_process_whose_free_memory_reading_already_lacks_the_driver_share_of_a_registered_process_counts_that_share_once(
+    ) {
+        let directory = LedgerDirectory::for_the_test("driver-share-once");
+        let first = ledger_reading_the_free_memory(
+            &directory.path,
+            100,
+            &[100, 200],
+            30_000,
+            Duration::from_secs(60),
+        );
+        let first_booking = booking_outcome(&first, 1_000).expect("空着的卡放得下 1 000");
+        assert_eq!(first_booking, 1_000);
+        let true_free_when_the_second_opens = 30_000 - DRIVER - 1_000;
+        let second = ledger_reading_the_free_memory(
+            &directory.path,
+            200,
+            &[100, 200],
+            true_free_when_the_second_opens,
+            Duration::from_millis(200),
+        );
+        assert_eq!(
+            booking_outcome(&second, true_free_when_the_second_opens - DRIVER),
+            Ok(true_free_when_the_second_opens - DRIVER),
+            "进程 100 的驱动那一份只记一份：进程 200 借得到真空闲减它自己的驱动那一份"
+        );
+        assert!(
+            matches!(
+                booking_outcome(&second, 1),
+                Err(BookingRefusal::WaitedPastTheLongestWait { .. })
+            ),
+            "再多 1 MiB 就超过了真空闲：等满放弃"
+        );
+    }
+
+    /// 第 10t 批 K4-3 撞显存那一向：进程 100 开账时卡上空 30 000、借了 100；之后一个不经账的 GPU 用户占了 20 000；
+    /// 进程 200 开账时读到 7 340。进程 200 能再借的加它自己的驱动那一份不超过真空闲（4 780 + 2 560 = 7 340），
+    /// 进程 100 的下一笔也照改小了的估计借。
+    #[test]
+    fn an_outsider_taking_memory_after_the_first_process_opened_its_ledger_caps_a_later_process_by_the_free_memory_it_reads(
+    ) {
+        let directory = LedgerDirectory::for_the_test("outsider-took-memory");
+        let first = ledger_reading_the_free_memory(
+            &directory.path,
+            100,
+            &[100, 200],
+            30_000,
+            Duration::from_millis(200),
+        );
+        booking_outcome(&first, 100).expect("空着的卡放得下 100");
+        let true_free_when_the_second_opens = 30_000 - DRIVER - 100 - 20_000;
+        let second = ledger_reading_the_free_memory(
+            &directory.path,
+            200,
+            &[100, 200],
+            true_free_when_the_second_opens,
+            Duration::from_millis(200),
+        );
+        let room_of_the_second = true_free_when_the_second_opens - DRIVER;
+        assert_eq!(
+            booking_outcome(&second, room_of_the_second),
+            Ok(room_of_the_second),
+            "进程 200 借得到真空闲减它自己的驱动那一份"
+        );
+        assert!(
+            matches!(
+                booking_outcome(&second, 1),
+                Err(BookingRefusal::WaitedPastTheLongestWait { .. })
+            ),
+            "进程 200 再多借 1 MiB 就超过了真空闲：等满放弃"
+        );
+        assert!(
+            matches!(
+                booking_outcome(&first, 1),
+                Err(BookingRefusal::WaitedPastTheLongestWait { .. })
+            ),
+            "进程 100 的下一笔照改小了的估计借：卡上已经没有空的了"
+        );
+    }
+
+    /// 第 10t 批 K4-3 等满报本次未跑那一向：进程 100 开账时卡被外人占着、读到 10 000，借了 2 000；外人走了，进程 200 开账时
+    /// 读到 25 440。进程 100 还借着 2 000 时，进程 200 借 6 000、再借 3 000 都当场借到，不等。
+    #[test]
+    fn an_outsider_leaving_after_the_first_process_opened_its_ledger_lets_a_later_process_book_what_it_now_reads(
+    ) {
+        let directory = LedgerDirectory::for_the_test("outsider-left");
+        let first = ledger_reading_the_free_memory(
+            &directory.path,
+            100,
+            &[100, 200],
+            10_000,
+            Duration::from_secs(60),
+        );
+        booking_outcome(&first, 2_000).expect("外人占着时卡上也放得下 2 000");
+        let second = ledger_reading_the_free_memory(
+            &directory.path,
+            200,
+            &[100, 200],
+            30_000 - DRIVER - 2_000,
+            Duration::from_millis(200),
+        );
+        assert_eq!(
+            booking_outcome(&second, 6_000),
+            Ok(6_000),
+            "外人走了，卡上空着两万多：6 000 当场借到"
+        );
+        assert_eq!(
+            booking_outcome(&second, 3_000),
+            Ok(3_000),
+            "进程 100 还借着 2 000：3 000 也当场借到，不等"
+        );
+    }
+
+    /// 第 10t 批 K4-3 正推那一形：进程 100 把卡订满（借了它的整份容量），进程 200 开账时读到的空闲是 0。进程 200 借 1 000
+    /// 要等、不当成卡不够（不交「比每张卡都大」）；进程 100 还了，进程 200 重读账本时借到。
+    #[test]
+    fn a_card_the_first_process_booked_full_makes_a_later_process_wait_instead_of_refusing_the_card(
+    ) {
+        let directory = LedgerDirectory::for_the_test("booked-full");
+        let first = ledger_reading_the_free_memory(
+            &directory.path,
+            100,
+            &[100, 200],
+            30_000,
+            Duration::from_secs(60),
+        );
+        let whole_card = first
+            .book_waiting_for_room(30_000 - DRIVER)
+            .expect("空着的卡放得下它的整份容量");
+        // 等着借的那条线程拿的是账的借用；判红时它可能一直等着，账交给 'static、线程不回收，判红不卡住整个用例
+        let second: &'static GpuMemoryLedger = Box::leak(Box::new(ledger_reading_the_free_memory(
+            &directory.path,
+            200,
+            &[100, 200],
+            0,
+            Duration::from_secs(3600),
+        )));
+        let (sender, receiver) = std::sync::mpsc::channel();
+        std::thread::spawn(move || {
+            sender
+                .send(booking_outcome(second, 1_000))
+                .expect("用例还在等这一条");
+        });
+        assert!(
+            receiver.recv_timeout(Duration::from_millis(300)).is_err(),
+            "卡被进程 100 订满：进程 200 要等，不当场借到、也不当成卡不够"
+        );
+        drop(whole_card);
+        assert_eq!(
+            receiver.recv_timeout(Duration::from_secs(30)),
+            Ok(Ok(1_000)),
+            "进程 100 还了之后，进程 200 借到"
+        );
+    }
+
+    /// 第 10t 批 K4-3 先登记、后建设备：开账不登记；第一次借时连这个进程的驱动那一份一起登记；只登记驱动那一份的，放得下才登记，
+    /// 放不下等着、等满放弃，一直不登记。卡的估计 10 000：进程 100 登记并借 4 880（连驱动 7 440），进程 200 登记（连上 10 000），
+    /// 进程 300 的驱动那一份放不下。
+    #[test]
+    fn a_process_is_registered_on_a_card_only_once_its_first_booking_or_its_driver_share_fits() {
+        let directory = LedgerDirectory::for_the_test("register-before-the-device");
+        let first = ledger_reading_the_free_memory(
+            &directory.path,
+            100,
+            &[100, 200, 300],
+            10_000,
+            Duration::from_secs(60),
+        );
+        assert_eq!(
+            first.is_this_process_registered_on_the_card(0),
+            Ok(false),
+            "开账只读空闲，不登记"
+        );
+        const FIRST_LEASE: u64 = 10_000 - 2 * DRIVER;
+        booking_outcome(&first, FIRST_LEASE).expect("空着的卡放得下");
+        assert_eq!(
+            first.is_this_process_registered_on_the_card(0),
+            Ok(true),
+            "第一次借时登记"
+        );
+        let second = ledger_reading_the_free_memory(
+            &directory.path,
+            200,
+            &[100, 200, 300],
+            10_000 - DRIVER - FIRST_LEASE,
+            Duration::from_millis(200),
+        );
+        assert_eq!(
+            second.register_this_process_on_the_card_waiting_for_room(0),
+            Ok(()),
+            "进程 200 的驱动那一份正好放得下"
+        );
+        assert_eq!(second.is_this_process_registered_on_the_card(0), Ok(true));
+        let third = ledger_reading_the_free_memory(
+            &directory.path,
+            300,
+            &[100, 200, 300],
+            0,
+            Duration::from_millis(200),
+        );
+        assert!(
+            matches!(
+                third.register_this_process_on_the_card_waiting_for_room(0),
+                Err(BookingRefusal::WaitedPastTheLongestWait { .. })
+            ),
+            "进程 300 的驱动那一份放不下：等满放弃"
+        );
+        assert_eq!(
+            third.is_this_process_registered_on_the_card(0),
+            Ok(false),
+            "放不下就不登记（调用方不起设备）"
+        );
+    }
+
     /// 一张空着的 5090 照 3/4 额度装得下 17 867 个状态（2026-10-06 日志里那一行）：45 个状态的流只开 45 个，
     /// 131 195 个状态的大流照旧按显存开，调用方要得更少就照它的，一点显存都不给、流是空的也带一个。
     #[test]
@@ -761,17 +1423,17 @@ mod tests {
     ) {
         const DRIVER: u64 = 2560;
         assert_eq!(
-            card_ledger_capacity_mebibytes(Some(4096), 22_500, 30_000, DRIVER),
+            card_ledger_capacity_mebibytes(Some(4096), None, 22_500, 30_000, DRIVER),
             4096,
             "额度比空闲显存减驱动那一份小：取额度"
         );
         assert_eq!(
-            card_ledger_capacity_mebibytes(Some(40_000), 22_500, 30_000, DRIVER),
+            card_ledger_capacity_mebibytes(Some(40_000), None, 22_500, 30_000, DRIVER),
             30_000 - DRIVER,
             "额度比空闲显存减驱动那一份大：取空闲显存那一边"
         );
         assert_eq!(
-            card_ledger_capacity_mebibytes(None, 22_500, 30_000, DRIVER),
+            card_ledger_capacity_mebibytes(None, None, 22_500, 30_000, DRIVER),
             22_500 - DRIVER,
             "没设额度：照旧是给崩溃放量的额度减驱动那一份"
         );
@@ -976,6 +1638,7 @@ mod tests {
             Ok(Err(BookingRefusal::WaitedPastTheLongestWait {
                 need_mebibytes,
                 waited,
+                ..
             })) => {
                 assert_eq!(need_mebibytes, 30);
                 assert!(
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index e58fe8f..01b6d08 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -2,7 +2,8 @@
 //! 这一版 GPU 接单元级那一截：一批单元的头校验和与载荷 CRC 在 GPU 上算（CRC-32C，参数与 D18（块里携带什么信息） 已定项 17 相同），
 //! 同一批在 CPU 上再算一遍逐个对拍，对不上就判红（`GpuCpuDisagreement`）；走树、恢复、记录核对器照旧在 CPU 上。
 //! 着色器与多卡枚举取自 E163（GPU多卡算单元校验和） 的装置（那边两卡 2560/2560 与 CPU 逐个相同）。
-//! 分工按显卡空闲显存（`nvidia-smi` 的 memory.free，按 PCI 总线号对上 wgpu 的适配器），空闲显存低于门槛的卡不用。
+//! 分工按显卡空闲显存（`nvidia-smi` 的 memory.free，按 PCI 总线号对上 wgpu 的适配器），空闲显存低于门槛的卡不用；
+//! GPU 用例共用的卡池（[`gpu_test_start::GpuCardPool`]）不看这个门槛，能不能借由跨进程的显存账判（第 10t 批 K4-3）。
 
 use singlefs_checker::{
     crc32_castagnoli_table, unit_checksum_layout, UNIT_HEADER_CHECKSUM_BYTE_OFFSET,
@@ -10,6 +11,7 @@ use singlefs_checker::{
 use singlefs_harness::sha256::DIGEST_BYTES;
 
 /// 空闲显存低于这个数（MiB）的卡不用：本机有卡被别的服务占着时只剩几百 MiB，起上下文都难（E163 M2 那张卡 141 MiB 起不来）。
+/// 只管 [`usable_gpu_cards`] 与 [`gpu_cards_by_priority`]；卡池按账判，被先起的进程订满的卡照样进池、借的时候等。
 pub const MINIMUM_FREE_MEMORY_MEBIBYTES: u64 = 2048;
 /// 一个 GPU 槽装一段字节：最大 32 KiB（数据单元的长度），短的按长度截断。
 const SLOT_BYTES: usize = 32_768;
@@ -78,9 +80,11 @@ pub struct GpuCard {
     pub pci_bus_identifier: String,
     /// `nvidia-smi` 给这张卡的序号（配置按它点名）。
     pub index: u32,
-    /// 起上下文那一刻 `nvidia-smi` 报的空闲显存（MiB）。
+    /// 起上下文之前 `nvidia-smi` 报的空闲显存（MiB；卡池里的卡是开账时在账本锁里读的那一份）。
+    /// 卡池借出去的卡改成借的量加驱动那一份：账判过卡上放得下这么多，不让起池那一刻的读数把草稿区压得比借的小。
     pub free_memory_mebibytes: u64,
     /// 配置给这张卡的显存额度（MiB）：崩溃放量在这张卡上占的显存不超过它；没给额度的是 `None`。
+    /// 卡池借出去的卡改成借的量加驱动那一份。
     pub memory_quota_mebibytes: Option<u64>,
     device: wgpu::Device,
     queue: wgpu::Queue,
@@ -132,11 +136,98 @@ fn normalized_bus(text: &str) -> String {
     parts[parts.len().saturating_sub(2)..].join(":")
 }
 
-/// 配置点名的卡，按配置的次序（优先级）：`(nvidia-smi 序号, 显存额度 MiB)`。点名而用不了的卡（本机没有这个序号、
-/// 空闲显存低于 [`MINIMUM_FREE_MEMORY_MEBIBYTES`]、上下文起不来）打一行说明，不悄悄换成别的卡；没点名的卡一张不用。
+/// Vulkan 下的一张 NVIDIA 独显与 `nvidia-smi` 报的它（序号、那一刻的空闲显存）：还没起设备。
+/// 卡池先找到点名的卡、在账上登记了这个进程，才在卡上起设备（[`gpu_test_start::GpuCardPool`]）；别的取卡入口找到就起。
+#[derive(Clone)]
+struct DiscoveredCard {
+    adapter: wgpu::Adapter,
+    name: String,
+    pci_bus_identifier: String,
+    reported: ReportedCard,
+}
+
+/// 本机 Vulkan 下每张 NVIDIA 独显，带 `nvidia-smi` 按 PCI 总线号报的序号与空闲显存（`nvidia-smi` 里对不上的卡不算）；一个设备都不起。
+fn nvidia_discrete_cards_reported_by_nvidia_smi() -> Vec<DiscoveredCard> {
+    enable_the_driver_shader_disk_cache();
+    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
+        backends: wgpu::Backends::VULKAN,
+        ..wgpu::InstanceDescriptor::new_without_display_handle()
+    });
+    let reported_by_bus = reported_cards_by_pci_bus();
+    pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN))
+        .into_iter()
+        .filter_map(|adapter| {
+            let information = adapter.get_info();
+            if information.vendor != NVIDIA_VENDOR_IDENTIFIER
+                || information.device_type != wgpu::DeviceType::DiscreteGpu
+            {
+                return None;
+            }
+            let pci_bus_identifier = normalized_bus(&information.device_pci_bus_id);
+            let reported = *reported_by_bus.get(&pci_bus_identifier)?;
+            Some(DiscoveredCard {
+                adapter,
+                name: information.name,
+                pci_bus_identifier,
+                reported,
+            })
+        })
+        .collect()
+}
+
+/// 在找到的这张卡上起设备、编单元校验和的管线；起不来交 `None`。不看空闲显存（看不看由调用方定）。
+fn start_the_card(discovered: &DiscoveredCard) -> Option<GpuCard> {
+    let adapter = &discovered.adapter;
+    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
+        label: Some("singlefs-gpu-unit-checks"),
+        required_features: wgpu::Features::empty(),
+        // 崩溃放量的判器把整条流的版本字节与每个状态的草稿区各绑成一张存储缓冲，默认的 128 MiB 绑定上限装不下大流：
+        // 按这张卡实际支持的上限要（NVIDIA Vulkan 上是 2 GiB − 1）。
+        required_limits: adapter.limits(),
+        experimental_features: wgpu::ExperimentalFeatures::disabled(),
+        memory_hints: wgpu::MemoryHints::Performance,
+        trace: wgpu::Trace::Off,
+    }))
+    .ok()?;
+    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
+        label: Some("crc32c"),
+        source: wgpu::ShaderSource::Wgsl(CRC32C_SHADER_SOURCE.into()),
+    });
+    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
+        label: Some("crc32c"),
+        layout: None,
+        module: &module,
+        entry_point: Some("main"),
+        compilation_options: wgpu::PipelineCompilationOptions::default(),
+        cache: None,
+    });
+    Some(GpuCard {
+        name: discovered.name.clone(),
+        pci_bus_identifier: discovered.pci_bus_identifier.clone(),
+        index: discovered.reported.index,
+        free_memory_mebibytes: discovered.reported.free_memory_mebibytes,
+        memory_quota_mebibytes: None,
+        device,
+        queue,
+        pipeline,
+        compiled_pipelines_by_source_digest: std::sync::Arc::default(),
+    })
+}
+
+/// 配置点名的卡，按配置的次序（优先级）：`(nvidia-smi 序号, 显存额度 MiB)`。只在点名的卡上起设备（没点名的卡一个设备都不起）；
+/// 点名而用不了的卡（本机没有这个序号、空闲显存低于 [`MINIMUM_FREE_MEMORY_MEBIBYTES`]、上下文起不来）打一行说明，不悄悄换成别的卡。
+/// 起的设备不进卡池的账（崩溃放量没有卡池的进程走这里；这个进程起过卡池的，崩溃放量照卡池里的卡用）。
 #[must_use]
 pub fn gpu_cards_by_priority(configured: &[(u32, u64)]) -> Vec<GpuCard> {
-    let mut usable = usable_gpu_cards();
+    let named_indices: Vec<u32> = configured.iter().map(|(index, _)| *index).collect();
+    let mut usable: Vec<GpuCard> = nvidia_discrete_cards_reported_by_nvidia_smi()
+        .iter()
+        .filter(|discovered| named_indices.contains(&discovered.reported.index))
+        .filter(|discovered| {
+            discovered.reported.free_memory_mebibytes >= MINIMUM_FREE_MEMORY_MEBIBYTES
+        })
+        .filter_map(start_the_card)
+        .collect();
     let mut cards = Vec::with_capacity(configured.len());
     for (index, quota) in configured {
         match usable.iter().position(|card| card.index == *index) {
@@ -154,67 +245,16 @@ pub fn gpu_cards_by_priority(configured: &[(u32, u64)]) -> Vec<GpuCard> {
 }
 
 /// 本机能用的卡：Vulkan 下的 NVIDIA 独显，空闲显存不低于 [`MINIMUM_FREE_MEMORY_MEBIBYTES`]，上下文起得来；按空闲显存从大到小排。
+/// 起的设备不进卡池的账。
 #[must_use]
 pub fn usable_gpu_cards() -> Vec<GpuCard> {
-    enable_the_driver_shader_disk_cache();
-    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
-        backends: wgpu::Backends::VULKAN,
-        ..wgpu::InstanceDescriptor::new_without_display_handle()
-    });
-    let reported_by_bus = reported_cards_by_pci_bus();
-    let mut cards: Vec<GpuCard> =
-        pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN))
-            .into_iter()
-            .filter_map(|adapter| {
-                let information = adapter.get_info();
-                if information.vendor != NVIDIA_VENDOR_IDENTIFIER
-                    || information.device_type != wgpu::DeviceType::DiscreteGpu
-                {
-                    return None;
-                }
-                let reported =
-                    *reported_by_bus.get(&normalized_bus(&information.device_pci_bus_id))?;
-                let free = reported.free_memory_mebibytes;
-                if free < MINIMUM_FREE_MEMORY_MEBIBYTES {
-                    return None;
-                }
-                let (device, queue) =
-                    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
-                        label: Some("singlefs-gpu-unit-checks"),
-                        required_features: wgpu::Features::empty(),
-                        // 崩溃放量的判器把整条流的版本字节与每个状态的草稿区各绑成一张存储缓冲，默认的 128 MiB 绑定上限装不下大流：
-                        // 按这张卡实际支持的上限要（NVIDIA Vulkan 上是 2 GiB − 1）。
-                        required_limits: adapter.limits(),
-                        experimental_features: wgpu::ExperimentalFeatures::disabled(),
-                        memory_hints: wgpu::MemoryHints::Performance,
-                        trace: wgpu::Trace::Off,
-                    }))
-                    .ok()?;
-                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
-                    label: Some("crc32c"),
-                    source: wgpu::ShaderSource::Wgsl(CRC32C_SHADER_SOURCE.into()),
-                });
-                let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
-                    label: Some("crc32c"),
-                    layout: None,
-                    module: &module,
-                    entry_point: Some("main"),
-                    compilation_options: wgpu::PipelineCompilationOptions::default(),
-                    cache: None,
-                });
-                Some(GpuCard {
-                    name: information.name,
-                    pci_bus_identifier: normalized_bus(&information.device_pci_bus_id),
-                    index: reported.index,
-                    free_memory_mebibytes: free,
-                    memory_quota_mebibytes: None,
-                    device,
-                    queue,
-                    pipeline,
-                    compiled_pipelines_by_source_digest: std::sync::Arc::default(),
-                })
-            })
-            .collect();
+    let mut cards: Vec<GpuCard> = nvidia_discrete_cards_reported_by_nvidia_smi()
+        .iter()
+        .filter(|discovered| {
+            discovered.reported.free_memory_mebibytes >= MINIMUM_FREE_MEMORY_MEBIBYTES
+        })
+        .filter_map(start_the_card)
+        .collect();
     cards.sort_by_key(|card| std::cmp::Reverse(card.free_memory_mebibytes));
     cards
 }
@@ -663,7 +703,8 @@ pub mod gpu_test_start {
         PciBusVisibleButListingUnreadable { reason: String },
     }
 
-    /// `adapters` 是 Vulkan 枚举的结果，`pci` 是读 PCI 设备表的结果，`usable_card_count` 是起好上下文的卡数（[`usable_gpu_cards`] 交回几张）。
+    /// `adapters` 是 Vulkan 枚举的结果，`pci` 是读 PCI 设备表的结果，`usable_card_count` 是用得上的卡数：
+    /// 起好上下文的卡数（[`usable_gpu_cards`] 交回几张），卡池是找到的点名卡数（卡池起池时不起设备，借到哪张才起哪张）。
     #[must_use]
     pub fn decide_gpu_test_start(
         adapters: &[EnumeratedAdapter],
@@ -878,12 +919,14 @@ pub mod gpu_test_start {
 
     /// GPU 用例共用的卡池（[`card_pool::GpuCardPool`]）：只开 `gpu` 也编得进（显卡配置的读法在 `crate::gpu_card_configuration`）。
     pub use card_pool::{
-        gpu_card_pool_or_report_not_run, GpuCardLease, GpuCardPool, CARD_QUOTA_ENVIRONMENT_VARIABLE,
+        card_pool_started_in_this_process, gpu_card_pool_or_report_not_run, GpuCardLease,
+        GpuCardPool, CARD_QUOTA_ENVIRONMENT_VARIABLE,
     };
 
     mod card_pool {
         use super::super::{
-            gpu_cards_by_priority, memory_budget_mebibytes_of, usable_gpu_cards, GpuCard,
+            memory_budget_mebibytes_of, nvidia_discrete_cards_reported_by_nvidia_smi,
+            reported_cards_by_pci_bus, start_the_card, DiscoveredCard, GpuCard,
         };
         use super::{
             decide_gpu_test_start, enumerate_vulkan_adapters, not_run_line,
@@ -897,29 +940,43 @@ pub mod gpu_test_start {
         use singlefs_harness::gpu_memory_ledger::{
             card_ledger_capacity_mebibytes, gpu_memory_ledger_directory_of_this_process,
             process_is_alive_by_its_entry_in_the_process_filesystem, BookingRefusal,
-            CardToRegister, GpuMemoryBooking, GpuMemoryLedger, LedgerSettings, RecordedProcess,
-            LONGEST_WAIT_FOR_GPU_MEMORY, POLL_INTERVAL_FOR_MEMORY_RETURNED_BY_OTHER_PROCESSES,
+            GpuMemoryBooking, GpuMemoryLedger, LedgerOpeningRefusal, LedgerSettings,
+            RecordedProcess, LONGEST_WAIT_FOR_GPU_MEMORY,
+            POLL_INTERVAL_FOR_MEMORY_RETURNED_BY_OTHER_PROCESSES,
         };
         use singlefs_harness::numeric_environment_variable::{
             number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
         };
 
-        /// 每张卡账的上限（MiB）：设了，每张卡的账取它与「清场后的空闲显存减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]」里小的那个
+        /// 每张卡账的上限（MiB）：设了，每张卡的账取它、配置给这张卡的额度减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]、
+        /// 记账进程合起来能占的显存减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`] 三者里最小的
         /// （`singlefs_harness::gpu_memory_ledger::card_ledger_capacity_mebibytes`）；不是每条用例的上限，借得比它多的照常判「放不下」。
         pub const CARD_QUOTA_ENVIRONMENT_VARIABLE: &str =
             "SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES";
 
-        /// GPU 用例共用的卡：这台机器点名的每张卡起一次设备，各一本跨进程的显存账（[`GpuMemoryLedger`]，账本在工作根下）；
+        /// 池里的一张卡：照配置点了名、Vulkan 与 `nvidia-smi` 里都找得到；设备等这个进程在这张卡的账上登记了之后才起
+        /// （[`GpuCardPool::started_card`]，一个进程在一张卡上只起一次）。
+        struct CardOfThePool {
+            discovered: DiscoveredCard,
+            /// `GPU_CARDS_BY_PRIORITY` 配给这张卡的额度（MiB）；没配显卡优先级的是 `None`。
+            configured_card_quota_mebibytes: Option<u64>,
+            /// 开账时在这张卡的账本锁里读的空闲显存（MiB）。
+            free_memory_read_when_the_ledger_opened_mebibytes: u64,
+            started: std::sync::OnceLock<Result<GpuCard, String>>,
+        }
+
+        /// GPU 用例共用的卡：这台机器点名的每张卡各一本跨进程的显存账（[`GpuMemoryLedger`]，账本在工作根下），每张卡这个进程只起一个设备；
         /// 用例按自己实际要的量借（[`GpuCardPool::lease_card_or_report_not_run`]），借据丢掉时还。
-        /// 一张卡的账照 [`card_ledger_capacity_mebibytes`]：这张卡给崩溃放量的额度（[`GpuCard::memory_budget_mebibytes`] 的算法，
-        /// 空闲显存取账本里记的清场后那一份）或 [`CARD_QUOTA_ENVIRONMENT_VARIABLE`]，减一份 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]：
-        /// 驱动的上下文与编好的内核每个进程在每张卡上付一次，同一台机器上别的进程起的池各再记一份。
+        /// 一张卡的账照 [`card_ledger_capacity_mebibytes`]（账本里的估计怎么来见 `singlefs_harness::gpu_memory_ledger` 文件头）：
+        /// 驱动的上下文与编好的内核每个进程在每张卡上付一份 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]，同一台机器上别的进程各再记一份。
+        /// 起池不看卡上此刻空闲多少（被先起的进程订满的卡照样进池），能不能借由账判；这个进程第一次在一张卡上借（连它的驱动那一份）
+        /// 记上了，才在这张卡上起设备。
         pub struct GpuCardPool {
-            cards: Vec<GpuCard>,
+            cards: Vec<CardOfThePool>,
             ledger: GpuMemoryLedger,
         }
 
-        /// 借来的一张卡：[`GpuCardLease::card`] 与池里那张共用设备、编好的内核，显存额度改成借的量加 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]
+        /// 借来的一张卡：[`GpuCardLease::card`] 与池里那张共用设备、编好的内核，显存额度与空闲改成借的量加 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]
         /// （判器照「额度 − 表 − 那一份」开草稿区，开出来正好是借的那么多）。丢掉时先让设备收掉已丢的缓冲，再销账。
         pub struct GpuCardLease {
             card: GpuCard,
@@ -952,24 +1009,64 @@ pub mod gpu_test_start {
         }
 
         impl GpuCardPool {
-            /// 池里每张卡存储缓冲绑定上限里最小的那个（用例借之前按它估一条草稿缓冲装几个状态）。
+            /// 池里每张卡存储缓冲绑定上限里最小的那个（用例借之前按它估一条草稿缓冲装几个状态）。设备按适配器的上限要
+            /// （`start_the_card`），没起设备也照适配器读得到。
             #[must_use]
             pub fn smallest_max_storage_buffer_binding_size(&self) -> u64 {
                 self.cards
                     .iter()
-                    .map(|card| card.device().limits().max_storage_buffer_binding_size)
+                    .map(|card| {
+                        card.discovered
+                            .adapter
+                            .limits()
+                            .max_storage_buffer_binding_size
+                    })
                     .min()
                     .expect(
                         "池里至少一张卡：没有卡的机器上起不出池（gpu_card_pool_or_report_not_run）",
                     )
             }
 
+            /// 第 `card_position` 张卡起好的设备：第一次要时起（调用方先在这张卡的账上登记了这个进程），之后拿同一个；起不来交一句为什么。
+            ///
+            /// # Panics
+            /// 这个进程还没在这张卡的账上登记就来起设备（先登记、后建设备的次序被打乱了）；账本读写不了。
+            fn started_card(&self, card_position: usize) -> Result<&GpuCard, String> {
+                let card = &self.cards[card_position];
+                card.started
+                    .get_or_init(|| {
+                        assert!(
+                            self.ledger
+                                .is_this_process_registered_on_the_card(card_position)
+                                .unwrap_or_else(|error| panic!(
+                                    "显存账本 {} 用不了：{}",
+                                    error.path.display(),
+                                    error.reason
+                                )),
+                            "序号 {} 的卡：这个进程在它的账上登记之前就要起设备（先登记、后建设备）",
+                            card.discovered.reported.index
+                        );
+                        let mut started = start_the_card(&card.discovered).ok_or_else(|| {
+                            format!(
+                                "序号 {} 的卡（{}）上起不了设备：账上登记了这个进程，卡却起不来上下文（驱动坏了，或卡被不经账的程序占满）",
+                                card.discovered.reported.index, card.discovered.name
+                            )
+                        })?;
+                        started.memory_quota_mebibytes = card.configured_card_quota_mebibytes;
+                        started.free_memory_mebibytes =
+                            card.free_memory_read_when_the_ledger_opened_mebibytes;
+                        Ok(started)
+                    })
+                    .as_ref()
+                    .map_err(Clone::clone)
+            }
+
             /// 借 `need_mebibytes`：照卡的优先级记在第一张放得下的卡上（前一张放满了才用下一张）；此刻每张都放不下就等别的用例、
             /// 别的 GPU 测试进程还，等满 [`LONGEST_WAIT_FOR_GPU_MEMORY`] 还借不到就打 [`not_run_line`] 那一行、交回 `None`（调用的用例当场返回）。
             /// 每借到一次打一行 `GPU_CARD_LEASE test=… index=… mebibytes=… capacity_mebibytes=…`。
             ///
             /// # Panics
-            /// 要的比每张卡的账都大（配了 GPU 就一定用 GPU，放不下判红，不悄悄换小）；账本文件读写不了。
+            /// 要的比每张卡的账都大（配了 GPU 就一定用 GPU，放不下判红，不悄悄换小）；账本文件读写不了；借到的卡上起不了设备。
             #[must_use]
             pub fn lease_card_or_report_not_run(
                 &'static self,
@@ -1011,20 +1108,18 @@ pub mod gpu_test_start {
                 let booking = match self.ledger.book_waiting_for_room(need_mebibytes) {
                     Ok(booking) => booking,
                     Err(BookingRefusal::NeedLargerThanEveryCard(refusal)) => panic!(
-                        "{test_name}：要借 {} MiB，池里最大的一张卡的账只有 {} MiB（每张卡的账 {:?}）：这台机器的卡放不下这条用例，\
+                        "{test_name}：要借 {} MiB，池里最大的一张卡的账只有 {} MiB：这台机器的卡放不下这条用例，\
                          腾出显存、给卡更大的额度（GPU_CARDS_BY_PRIORITY），或缩小用例",
-                        refusal.need_mebibytes,
-                        refusal.largest_capacity_mebibytes,
-                        self.ledger.card_capacities_mebibytes()
+                        refusal.need_mebibytes, refusal.largest_capacity_mebibytes
                     ),
                     Err(BookingRefusal::WaitedPastTheLongestWait {
                         need_mebibytes: waited_for_mebibytes,
                         waited,
+                        card_capacities_mebibytes,
                     }) => {
                         return Err(format!(
-                            "等了 {} 秒，池里没有一张卡空得出 {waited_for_mebibytes} MiB（每张卡的账 {:?}；这台机器上别的 GPU 用例或测试进程一直占着）",
-                            waited.as_secs(),
-                            self.ledger.card_capacities_mebibytes()
+                            "等了 {} 秒，池里没有一张卡空得出 {waited_for_mebibytes} MiB（每张卡的账 {card_capacities_mebibytes:?}；这台机器上别的 GPU 用例或测试进程一直占着）",
+                            waited.as_secs()
                         ))
                     }
                     Err(BookingRefusal::LedgerFileUnusable(error)) => panic!(
@@ -1034,16 +1129,44 @@ pub mod gpu_test_start {
                     ),
                 };
                 let position = booking.card_position();
-                let mut card = self.cards[position].clone();
+                let mut card = self
+                    .started_card(position)
+                    .unwrap_or_else(|reason| panic!("{test_name}：{reason}"))
+                    .clone();
                 card.memory_quota_mebibytes = Some(need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER);
+                card.free_memory_mebibytes = need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER;
                 eprintln!(
                     "GPU_CARD_LEASE test={test_name} card={:?} index={} mebibytes={need_mebibytes} capacity_mebibytes={}",
                     card.name,
                     card.index,
-                    self.ledger.card_capacities_mebibytes()[position]
+                    booking.card_capacity_mebibytes_when_booked()
                 );
                 Ok(GpuCardLease { card, booking })
             }
+
+            /// 池里每张卡（按优先级）：这个进程在每张卡的账上登记（只记驱动那一份，不借；放不下就等，等满放弃），再取起好的设备。
+            /// 给崩溃放量用（`crate::crash_amplification`）：这个进程起过卡池时，放量照池里的卡判，不另起一套设备。
+            /// 交回的卡是池里那张的克隆（额度是配置给这张卡的、空闲是开账时读的）；放量开的草稿区不进账：该记多少、等不到时放量怎么退，条款没写，第 10t 批交主 agent 定。
+            ///
+            /// # Errors
+            /// 某张卡的账上放不下这个进程的驱动那一份（等满了、或永远放不下），账本读写不了，或卡上起不了设备。
+            pub fn cards_registered_and_started_in_priority_order(
+                &'static self,
+            ) -> Result<Vec<GpuCard>, String> {
+                (0..self.cards.len())
+                    .map(|card_position| {
+                        self.ledger
+                            .register_this_process_on_the_card_waiting_for_room(card_position)
+                            .map_err(|refusal| {
+                                format!(
+                                    "序号 {} 的卡：账上登记不了这个进程（{refusal:?}）",
+                                    self.cards[card_position].discovered.reported.index
+                                )
+                            })?;
+                        self.started_card(card_position).cloned()
+                    })
+                    .collect()
+            }
         }
 
         /// 起池那一次的结局：进程里只判一次，之后每条用例照它。
@@ -1053,22 +1176,23 @@ pub mod gpu_test_start {
             CardsPresentButNotUsable { reason: String },
         }
 
-        /// 这台机器给卡池点名的卡。
+        /// 这台机器给卡池点名的卡（还没起设备）。
         struct CardsNamedForThisMachine {
-            cards: Vec<GpuCard>,
-            /// 配置点了名、本机却起不来的卡（序号，按配置的次序）。
-            named_but_not_started: Vec<u32>,
-            /// 配置读不出、写法不对、没给这台机器点卡：一张卡都不起，原因在这里。
+            /// 点名的卡与配置给它的额度，按配置的次序（没配显卡优先级时是本机每张卡，按空闲显存从大到小、没有额度）。
+            cards: Vec<(DiscoveredCard, Option<u64>)>,
+            /// 配置点了名、本机 Vulkan 或 `nvidia-smi` 里却找不到的卡（序号，按配置的次序）。
+            named_but_not_found: Vec<u32>,
+            /// 配置读不出、写法不对、没给这台机器点卡：一张卡都不取，原因在这里。
             configuration_refusal: Option<String>,
         }
 
         /// 照显卡配置挑这台机器的卡：`GPU_CARDS_BY_PRIORITY`（环境变量，没设读 `SINGLEFS_GPU_CARDS_CONFIG` 指的显卡配置里的同名键）
-        /// 里这台机器（`SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`，没设是本机）的那几项，照配置的次序与额度起（与崩溃放量取卡同一个约定）；
-        /// 都没配就是本机能用的卡，按空闲显存从大到小、不设额度。
+        /// 里这台机器（`SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`，没设是本机）的那几项，照配置的次序与额度（与崩溃放量取卡同一个约定）；
+        /// 都没配就是本机每张 NVIDIA 独显，按空闲显存从大到小、不设额度。只找卡、不起设备，也不看此刻空闲多少（能不能借由账判）。
         fn cards_named_for_this_machine() -> CardsNamedForThisMachine {
             let refused = |reason: String| CardsNamedForThisMachine {
                 cards: Vec::new(),
-                named_but_not_started: Vec::new(),
+                named_but_not_found: Vec::new(),
                 configuration_refusal: Some(reason),
             };
             let host = match CardHost::of_this_process() {
@@ -1094,6 +1218,7 @@ pub mod gpu_test_start {
                 Ok(configured) => configured,
                 Err(reason) => return refused(reason),
             };
+            let mut discovered = nvidia_discrete_cards_reported_by_nvidia_smi();
             match configured {
                 Some(configured) => {
                     let mine: Vec<(u32, u64)> = configured
@@ -1107,31 +1232,48 @@ pub mod gpu_test_start {
                             host.name()
                         ));
                     }
-                    let cards = gpu_cards_by_priority(&mine);
-                    let started: Vec<u32> = cards.iter().map(|card| card.index).collect();
+                    let mut cards = Vec::with_capacity(mine.len());
+                    for (index, quota) in &mine {
+                        match discovered
+                            .iter()
+                            .position(|card| card.reported.index == *index)
+                        {
+                            Some(position) => cards.push((discovered.remove(position), Some(*quota))),
+                            None => eprintln!(
+                                "GPU_CARD_NOT_USABLE index={index}：配置点名了这张卡，本机 Vulkan 或 nvidia-smi 里却找不到它"
+                            ),
+                        }
+                    }
+                    let found: Vec<u32> =
+                        cards.iter().map(|(card, _)| card.reported.index).collect();
                     CardsNamedForThisMachine {
-                        named_but_not_started: configured_cards_not_usable(&mine, &started),
+                        named_but_not_found: configured_cards_not_usable(&mine, &found),
                         cards,
                         configuration_refusal: None,
                     }
                 }
-                None => CardsNamedForThisMachine {
-                    cards: usable_gpu_cards(),
-                    named_but_not_started: Vec::new(),
-                    configuration_refusal: None,
-                },
+                None => {
+                    discovered
+                        .sort_by_key(|card| std::cmp::Reverse(card.reported.free_memory_mebibytes));
+                    CardsNamedForThisMachine {
+                        cards: discovered.into_iter().map(|card| (card, None)).collect(),
+                        named_but_not_found: Vec::new(),
+                        configuration_refusal: None,
+                    }
+                }
             }
         }
 
         /// 一张卡的账本文件名的主干：按 PCI 总线号（换掉 `:`），同一台机器上几个进程认得出是同一张卡。
-        fn ledger_file_stem_of(card: &GpuCard) -> String {
+        fn ledger_file_stem_of(card: &DiscoveredCard) -> String {
             format!("card-{}", card.pci_bus_identifier.replace(':', "-"))
         }
 
-        /// 点名的卡都起好了之后，在每张卡的账本上记这个进程，开账。
-        fn open_the_ledger_of_the_started_cards(
-            cards: &[GpuCard],
-        ) -> Result<GpuMemoryLedger, String> {
+        /// 开点名的那几张卡的账（每张卡在锁里现读一次空闲显存，[`GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock`]），
+        /// 不起设备；交回池。
+        fn open_the_card_pool(
+            named: Vec<(DiscoveredCard, Option<u64>)>,
+        ) -> Result<GpuCardPool, String> {
             let card_quota = number_from_the_variable_value_panicking_when_unreadable::<u64>(
                 CARD_QUOTA_ENVIRONMENT_VARIABLE,
                 process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),
@@ -1146,34 +1288,66 @@ pub mod gpu_test_start {
                 directory,
                 this_process,
                 is_alive: Box::new(process_is_alive_by_its_entry_in_the_process_filesystem),
-                mebibytes_each_other_process_keeps_on_a_card: MEBIBYTES_LEFT_FOR_THE_DRIVER,
+                mebibytes_each_process_keeps_on_a_card: MEBIBYTES_LEFT_FOR_THE_DRIVER,
                 poll_interval_for_memory_returned_by_other_processes:
                     POLL_INTERVAL_FOR_MEMORY_RETURNED_BY_OTHER_PROCESSES,
                 longest_wait: LONGEST_WAIT_FOR_GPU_MEMORY,
             };
-            let to_register: Vec<CardToRegister> = cards
+            let file_stems: Vec<String> = named
                 .iter()
-                .map(|card| CardToRegister {
-                    file_stem: ledger_file_stem_of(card),
-                    free_memory_read_by_this_process_mebibytes: card.free_memory_mebibytes,
-                })
+                .map(|(card, _)| ledger_file_stem_of(card))
                 .collect();
-            GpuMemoryLedger::register_this_process_on_the_cards(
+            let configured_card_quotas: Vec<Option<u64>> =
+                named.iter().map(|(_, quota)| *quota).collect();
+            let free_memory_read_in_the_lock: Vec<std::cell::Cell<Option<u64>>> =
+                named.iter().map(|_| std::cell::Cell::new(None)).collect();
+            let read_free_memory_mebibytes = |position: usize| -> Result<u64, String> {
+                let bus = &named[position].0.pci_bus_identifier;
+                let free = reported_cards_by_pci_bus()
+                    .get(bus)
+                    .map(|reported| reported.free_memory_mebibytes)
+                    .ok_or_else(|| format!("nvidia-smi 里读不到总线 {bus} 那张卡的空闲显存"))?;
+                free_memory_read_in_the_lock[position].set(Some(free));
+                Ok(free)
+            };
+            let ledger = GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
                 settings,
-                &to_register,
-                |position, free_memory_after_quiescing_mebibytes| {
+                file_stems,
+                &read_free_memory_mebibytes,
+                Box::new(move |position, estimate| {
+                    let configured = configured_card_quotas[position];
                     card_ledger_capacity_mebibytes(
                         card_quota,
-                        memory_budget_mebibytes_of(
-                            cards[position].memory_quota_mebibytes,
-                            free_memory_after_quiescing_mebibytes,
-                        ),
-                        free_memory_after_quiescing_mebibytes,
+                        configured,
+                        memory_budget_mebibytes_of(configured, estimate),
+                        estimate,
                         MEBIBYTES_LEFT_FOR_THE_DRIVER,
                     )
-                },
+                }),
             )
-            .map_err(|error| format!("显存账本 {} 用不了：{}", error.path.display(), error.reason))
+            .map_err(|refusal| match refusal {
+                LedgerOpeningRefusal::LedgerFileUnusable(error) => {
+                    format!("显存账本 {} 用不了：{}", error.path.display(), error.reason)
+                }
+                LedgerOpeningRefusal::FreeMemoryUnreadable { file_stem, reason } => {
+                    format!("开 {file_stem} 那张卡的账时读不出空闲显存：{reason}")
+                }
+            })?;
+            let cards = named
+                .into_iter()
+                .zip(free_memory_read_in_the_lock)
+                .map(
+                    |((discovered, configured_card_quota_mebibytes), read)| CardOfThePool {
+                        discovered,
+                        configured_card_quota_mebibytes,
+                        free_memory_read_when_the_ledger_opened_mebibytes: read
+                            .get()
+                            .expect("开账成了，每张卡都在锁里读过一次空闲显存"),
+                        started: std::sync::OnceLock::new(),
+                    },
+                )
+                .collect();
+            Ok(GpuCardPool { cards, ledger })
         }
 
         fn start_the_gpu_card_pool() -> GpuCardPoolStart {
@@ -1191,12 +1365,9 @@ pub mod gpu_test_start {
                 .map_or(String::new(), |reason| format!("（显卡配置：{reason}）"));
             match decision {
                 GpuTestStart::RunOnTheUsableCards => {
-                    if named.named_but_not_started.is_empty() {
-                        match open_the_ledger_of_the_started_cards(&named.cards) {
-                            Ok(ledger) => GpuCardPoolStart::Started(GpuCardPool {
-                                cards: named.cards,
-                                ledger,
-                            }),
+                    if named.named_but_not_found.is_empty() {
+                        match open_the_card_pool(named.cards) {
+                            Ok(pool) => GpuCardPoolStart::Started(pool),
                             Err(reason) => GpuCardPoolStart::CardsPresentButNotUsable { reason },
                         }
                     } else {
@@ -1204,7 +1375,7 @@ pub mod gpu_test_start {
                         GpuCardPoolStart::CardsPresentButNotUsable {
                             reason: format!(
                                 "配置点名给这台机器的卡里，序号 {:?} 用不了（上面 GPU_CARD_NOT_USABLE 那几行）：点名的卡每一张都要用得上",
-                                named.named_but_not_started
+                                named.named_but_not_found
                             ),
                         }
                     }
@@ -1224,13 +1395,13 @@ pub mod gpu_test_start {
 
         static GPU_CARD_POOL: std::sync::OnceLock<GpuCardPoolStart> = std::sync::OnceLock::new();
 
-        /// 这个进程的卡池：第一条来要的用例起池（只在这时判一次有没有卡、读一次 `nvidia-smi`、每张卡起一次设备、在每张卡的账本上记这个进程，
-        /// 挑卡照 [`cards_named_for_this_machine`]），之后每条用例拿同一个。本机没有 NVIDIA 卡时每条来要的用例各打一行
+        /// 这个进程的卡池：第一条来要的用例起池（只在这时判一次有没有卡、找一次点名的卡、在每张卡的账本锁里读一次空闲显存，
+        /// 挑卡照 [`cards_named_for_this_machine`]；设备等借到那张卡时才起），之后每条用例拿同一个。本机没有 NVIDIA 卡时每条来要的用例各打一行
         /// [`not_run_line`]、交回 `None`（调用的用例当场返回）。先判有没有卡、建好表知道要多少显存之后再借（[`GpuCardPool::lease_card_or_report_not_run`]）。
         ///
         /// # Panics
-        /// 有卡而用不上（同 [`super::usable_cards_or_report_not_run`]）；显卡配置写错、没给这台机器点卡、点名的卡少一张起不来；
-        /// [`CARD_QUOTA_ENVIRONMENT_VARIABLE`] 设了而认不出；账本目录定不下、账本读写不了。
+        /// 有卡而用不上（同 [`super::usable_cards_or_report_not_run`]）；显卡配置写错、没给这台机器点卡、点名的卡少一张找不到；
+        /// [`CARD_QUOTA_ENVIRONMENT_VARIABLE`] 设了而认不出；账本目录定不下、账本读写不了、开账时读不出空闲显存。
         #[must_use]
         pub fn gpu_card_pool_or_report_not_run(test_name: &str) -> Option<&'static GpuCardPool> {
             match GPU_CARD_POOL.get_or_init(start_the_gpu_card_pool) {
@@ -1244,6 +1415,19 @@ pub mod gpu_test_start {
                 }
             }
         }
+
+        /// 这个进程已经起好的卡池；还没起过池、或起池判成没卡、有卡而用不上，交 `None`（不起池，什么都不打）。
+        #[must_use]
+        pub fn card_pool_started_in_this_process() -> Option<&'static GpuCardPool> {
+            match GPU_CARD_POOL.get() {
+                Some(GpuCardPoolStart::Started(pool)) => Some(pool),
+                Some(
+                    GpuCardPoolStart::NotRunNoGpuAdapter { .. }
+                    | GpuCardPoolStart::CardsPresentButNotUsable { .. },
+                )
+                | None => None,
+            }
+        }
     }
 }
 
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index 7e00f3e..42c4e9c 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -3515,6 +3515,20 @@ pub struct EnvironmentRun {
     /// 测试代码里预设 ignore 的点（按登记整批不判的一类，不算剪枝）与因此没展开的态数。
     pub ignored_points: usize,
     pub states_skipped_by_ignore: u64,
+    /// 这一趟用的卡从哪来（[`GpuCardsSource`]）。
+    pub gpu_cards_taken_from: GpuCardsSource,
+}
+
+/// 一趟用的卡从哪来。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum GpuCardsSource {
+    /// 这一趟没有一类计算走 GPU，不要卡。
+    NoCardNeeded,
+    /// 这个进程起过 GPU 用例共用的卡池（`gpu_unit_checks::gpu_test_start::GpuCardPool`）：照池里的卡用，每张卡这个进程只有一个设备，
+    /// 驱动那一份在跨进程的显存账上登记了；这一趟开的草稿区不进账（第 10t 批交主 agent 定该记多少）。
+    TheCardPoolOfThisProcess,
+    /// 这个进程没起过卡池：这一趟自己在点名的卡上起设备（`gpu_unit_checks::gpu_cards_by_priority` 或 `usable_gpu_cards`），不进显存账。
+    DevicesStartedForThisRun,
 }
 
 /// 每一行 `CRASH_AMPLIFICATION ` 末尾都接的几个字段（汇总行、被叫停的行、`flow=` 行、两台跑的分份行与导入行，打法都在
@@ -3922,10 +3936,11 @@ pub fn run_from_environment(
         }
     }
     let seconds_planning_and_recording = planning_started.elapsed().as_secs_f64();
-    let cards = if routes.need_a_card() {
-        Some(gpu_cards_in_use(cards_by_priority.as_deref(), host)?)
+    let (cards, gpu_cards_taken_from) = if routes.need_a_card() {
+        let (cards, taken_from) = gpu_cards_in_use(cards_by_priority.as_deref(), host)?;
+        (Some(cards), taken_from)
     } else {
-        None
+        (None, GpuCardsSource::NoCardNeeded)
     };
     if placement_only {
         let (ComputedOn::Gpu, Some(cards)) = (routes.crash_judge, &cards) else {
@@ -3976,6 +3991,7 @@ pub fn run_from_environment(
             states_reused: states_reused_at_recording,
             ignored_points,
             states_skipped_by_ignore,
+            gpu_cards_taken_from,
         });
     }
     // 第 ① 段：登记了 crash_judge_gpu 就由 GPU 判器判这台机器领的块（判不了就报错，不退回 CPU）；没登记由 CPU 判器判，
@@ -4052,6 +4068,7 @@ pub fn run_from_environment(
             states_reused: states_reused_at_recording,
             ignored_points,
             states_skipped_by_ignore,
+            gpu_cards_taken_from,
         });
     }
     let seconds_judging = judging_started.elapsed().as_secs_f64();
@@ -4165,6 +4182,7 @@ pub fn run_from_environment(
         states_reused: states_reused_at_recording,
         ignored_points,
         states_skipped_by_ignore,
+        gpu_cards_taken_from,
     })
 }
 
@@ -4212,11 +4230,25 @@ type CardsInUse = Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>>;
 type CardsInUse = ();
 
 /// 这个进程用哪几张卡：配置点了名的只用这台机器那几张、照配置的次序与额度；没配就是本机能用的卡按空闲显存从大到小。
+/// 这个进程起过 GPU 用例共用的卡池（[`crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process`]）就照池里的卡用
+/// （[`cards_from_the_card_pool_of_this_process`]），不另起一套设备；没起过才自己起。
 #[cfg(feature = "gpu")]
 fn gpu_cards_in_use(
     configured: Option<&[CardByPriority]>,
     host: CardHost,
-) -> Result<CardsInUse, String> {
+) -> Result<(CardsInUse, GpuCardsSource), String> {
+    if let Some(pool) = crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process()
+    {
+        let cards = cards_from_the_card_pool_of_this_process(
+            pool.cards_registered_and_started_in_priority_order()?,
+            configured,
+            host,
+        )?;
+        return Ok((
+            cards.into_iter().map(std::sync::Arc::new).collect(),
+            GpuCardsSource::TheCardPoolOfThisProcess,
+        ));
+    }
     let cards = match configured {
         Some(configured) => {
             let mine: Vec<(u32, u64)> = configured
@@ -4250,7 +4282,39 @@ fn gpu_cards_in_use(
                 .to_string(),
         );
     }
-    Ok(cards.into_iter().map(std::sync::Arc::new).collect())
+    Ok((
+        cards.into_iter().map(std::sync::Arc::new).collect(),
+        GpuCardsSource::DevicesStartedForThisRun,
+    ))
+}
+
+/// 照池里的卡（`pool_cards`，按池的优先级）给这一趟用：配置点了名的（`configured` 里这台机器那几项），池里的卡要与它逐张同序号、同额度，
+/// 对不上就报错（池与这一趟读到的显卡配置不同，不悄悄换卡）；没配就照池里的卡与它们的额度用。
+#[cfg(feature = "gpu")]
+fn cards_from_the_card_pool_of_this_process(
+    pool_cards: Vec<crate::gpu_unit_checks::GpuCard>,
+    configured: Option<&[CardByPriority]>,
+    host: CardHost,
+) -> Result<Vec<crate::gpu_unit_checks::GpuCard>, String> {
+    if let Some(configured) = configured {
+        let mine: Vec<(u32, Option<u64>)> = configured
+            .iter()
+            .filter(|card| card.host == host)
+            .map(|card| (card.index, Some(card.memory_quota_mebibytes)))
+            .collect();
+        let in_the_pool: Vec<(u32, Option<u64>)> = pool_cards
+            .iter()
+            .map(|card| (card.index, card.memory_quota_mebibytes))
+            .collect();
+        if mine != in_the_pool {
+            return Err(format!(
+                "这个进程的卡池里是 {in_the_pool:?}（序号，额度 MiB），这一趟的显卡配置给 {} 这台机器点的是 {mine:?}：两份显卡配置对不上，\
+                 不另起一套设备、也不悄悄换卡",
+                host.name()
+            ));
+        }
+    }
+    Ok(pool_cards)
 }
 
 /// 第 ① 段由 CPU 判时，单元两道校验和在哪对拍：登记了 `gpu_unit_checks` 交优先级最高的那张卡，没登记只在 CPU 上。
@@ -4299,7 +4363,7 @@ fn unit_checksums_on(
 fn gpu_cards_in_use(
     _configured: Option<&[CardByPriority]>,
     _host: CardHost,
-) -> Result<CardsInUse, String> {
+) -> Result<(CardsInUse, GpuCardsSource), String> {
     Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
 }
 
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
index 3365f95..1beb82a 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
@@ -954,11 +954,13 @@ fn a_run_importing_another_library_reports_the_states_judged_before_the_import_a
 
 /// L1：只排派活计划的那一趟照实报录入时库里已判的态数，不写 0。同一份库上第一趟（单机 CPU）把整条流判完；第二趟配置开 GPU、
 /// 登记 `crash_judge_gpu`、只排计划：一块都不判，复用数等于整条流的态数。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」
-/// （经这个进程的卡池判有没有卡，`gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run`），有适配器而一张卡都起不来判红。
-/// 只排计划的那一趟一个状态都不上卡判，这条用例不向卡池借显存。要 `verdict-store,gpu` 两个特性。
+/// （经这个进程的卡池判有没有卡，`gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run`），有适配器而点名的卡找不到判红。
+/// 只排计划的那一趟一个状态都不上卡判，这条用例不向卡池借显存；那一趟的卡照这个进程的卡池用（第 10t 批 K4-4：每张卡这个进程只起一个设备，
+/// 驱动那一份在显存账上登记了），不经产品路径另起一套不记账的设备。要 `verdict-store,gpu` 两个特性。
 #[cfg(feature = "gpu")]
 #[test]
 fn a_placement_only_run_reports_the_states_judged_at_recording_as_reused() {
+    use singlefs_checker_tier::crash_amplification::GpuCardsSource;
     if gpu_card_pool_or_report_not_run(
         "a_placement_only_run_reports_the_states_judged_at_recording_as_reused",
     )
@@ -992,6 +994,11 @@ fn a_placement_only_run_reports_the_states_judged_at_recording_as_reused() {
         placement_run.states_reused, state_count,
         "只排计划的那一趟录入时库里整条流都判过"
     );
+    assert_eq!(
+        placement_run.gpu_cards_taken_from,
+        GpuCardsSource::TheCardPoolOfThisProcess,
+        "这个进程起过卡池：只排计划的那一趟照池里的卡用，不另起不记账的设备"
+    );
     drop(directory);
 }
 
diff --git a/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs b/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
index 1b87089..536a2f2 100644
--- a/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
@@ -4,29 +4,32 @@
 //! 与进程里共用一池卡的 `gpu_card_pool_or_report_not_run`）：
 //! 没卡的机器上它报「本次未跑」，有卡而用不上它判红。绕开它直接调 `usable_gpu_cards` 或 `gpu_cards_by_priority` 起卡的用例，
 //! 在有卡的机器上与经入口走同一条路，跑用例分不出；这一条扫源码，不要显卡。
-//! 扫的是 `tests/` 下每份 `.rs`（这一份除外）整份，与 `src/` 下每份 `.rs` 里顶格 `#[cfg(test)]` 下的模块与函数；
-//! 产品路径（崩溃放量取卡、`gpu_unit_checks` 自己）不在测试代码里，不扫。行里 `//` 之后的注释不扫。
+//! 扫的是 `tests/` 下每份 `.rs`（这一份除外）整份，与 `src/` 下每份 `.rs` 里条件含 `test`（不在 `not(…)` 里）的 `#[cfg(…)]` 下的项，
+//! 顶格不顶格都算（`#[cfg(test)]`、`#[cfg(all(test, feature = "gpu"))]`、`impl` 块里缩进的 `#[cfg(test)]`）；
+//! 产品路径（崩溃放量取卡、`gpu_unit_checks` 自己）不在测试代码里，不扫。注释（`//` 到行尾、`/* */`）不扫，
+//! 字符串与字符字面量的内容也不扫（字符串里的 `//` 不当注释起头，同一行后面的代码照扫）。
+//! 判的是名字出现没出现，不只是带括号的调用：`use … as 别名` 那一行、拿函数当值、包一层改名都留得下这个名字（第 10t 批 K4-4）。
 //! 改走卡池的几份测试文件（[`FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL`]）另判一层：只许经 `gpu_card_pool_or_report_not_run` 取卡、
 //! 按量借，不许再各自拿第一张能用的卡、拿整份卡表或直接在卡上建判器（各按空闲显存的比例开，几个用例并行时互抢显存）。
 
 use std::path::{Path, PathBuf};
 
-/// 这一份自己的文件名：它的常量里写着要找的调用，不扫。
+/// 这一份自己的文件名：它的常量与样本里写着要找的名字，不扫。
 const THIS_FILE_NAME: &str =
     "gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs";
 
-/// 绕开入口直接起卡的调用。
-const CARD_TAKING_CALLS_THAT_BYPASS_THE_ENTRY: [&str; 2] =
-    ["usable_gpu_cards(", "gpu_cards_by_priority("];
+/// 绕开入口直接起卡的函数名：代码里出现就算（调用、`use` 引进来、改名、当值传）。
+const CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY: [&str; 2] =
+    ["usable_gpu_cards", "gpu_cards_by_priority"];
 
-/// 经入口取卡的调用。
-const CARD_TAKING_CALLS_THROUGH_THE_ENTRY: [&str; 3] = [
-    "usable_cards_or_report_not_run(",
-    "first_usable_card_or_report_not_run(",
-    "gpu_card_pool_or_report_not_run(",
+/// 经入口取卡的函数名（数调用）。
+const CARD_TAKING_FUNCTIONS_THROUGH_THE_ENTRY: [&str; 3] = [
+    "usable_cards_or_report_not_run",
+    "first_usable_card_or_report_not_run",
+    "gpu_card_pool_or_report_not_run",
 ];
 
-/// 改走卡池的测试文件（相对包根）：每份至少一处经 `gpu_card_pool_or_report_not_run(` 取卡，[`CALLS_THAT_BYPASS_THE_POOL`] 一处都不许有。
+/// 改走卡池的测试文件（相对包根）：每份至少一处调 [`CARD_POOL_FUNCTION`]，[`NAMES_THAT_BYPASS_THE_POOL`] 一处都不许有。
 const FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL: [&str; 7] = [
     "tests/crash_amplification_pipeline_records_checks_and_reuses.rs",
     "tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs",
@@ -37,15 +40,16 @@ const FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL: [&str; 7] = [
     "tests/record_checker_judges_absence_by_the_persisted_set.rs",
 ];
 
-/// 绕开卡池拿卡、建判器的调用：经入口拿第一张能用的卡、拿整份卡表，或拿一张卡直接建判器（判器照那张卡的空闲显存开草稿区，不记账）。
-const CALLS_THAT_BYPASS_THE_POOL: [&str; 3] = [
-    "first_usable_card_or_report_not_run(",
-    "usable_cards_or_report_not_run(",
-    "GpuCrashJudge::new(",
+/// 绕开卡池拿卡、建判器的名字：经入口拿第一张能用的卡、拿整份卡表，或拿一张卡直接建判器（`GpuCrashJudge` 的每个构造都照那张卡的
+/// 空闲显存或额度开草稿区，不记账）。代码里出现就算。
+const NAMES_THAT_BYPASS_THE_POOL: [&str; 3] = [
+    "first_usable_card_or_report_not_run",
+    "usable_cards_or_report_not_run",
+    "GpuCrashJudge",
 ];
 
-/// 改走卡池的文件里经卡池取卡的调用。
-const CARD_POOL_CALL: &str = "gpu_card_pool_or_report_not_run(";
+/// 改走卡池的文件里经卡池取卡的函数名。
+const CARD_POOL_FUNCTION: &str = "gpu_card_pool_or_report_not_run";
 
 /// 经入口判过「有卡、照跑」之后再起卡、准许绕开的几处：(相对包根的文件, 那一行去掉前后空白的代码, 为什么准许)。
 /// 每一项必须恰好命中一次，命中不到也判红：准许的那一行改了或删了，这里不许还留着。
@@ -83,84 +87,333 @@ fn rust_source_files_under(directory: &Path) -> Vec<PathBuf> {
     found
 }
 
-/// 一份源码每一行的代码部分（`//` 之后的注释去掉），带 1 起的行号。
-fn code_lines(source: &str) -> Vec<(usize, &str)> {
-    source
-        .lines()
+fn is_an_identifier_character(character: char) -> bool {
+    character.is_alphanumeric() || character == '_'
+}
+
+/// 从 `opening_position` 那个 `"` 起的一个字符串字面量：引号照抄，内容换成空格（换行照留，行号不变）；交回字面量之后的位置。
+/// `closing_hashes` 是原始字符串（`r#"…"#`）收尾的 `#` 个数，普通字符串是 0 且认 `\` 转义。
+fn blank_a_string_literal(
+    characters: &[char],
+    opening_position: usize,
+    closing_hashes: Option<usize>,
+    code: &mut String,
+) -> usize {
+    code.push('"');
+    let mut position = opening_position + 1;
+    while position < characters.len() {
+        let character = characters[position];
+        match closing_hashes {
+            None if character == '\\' => {
+                code.push(' ');
+                match characters.get(position + 1) {
+                    Some('\n') => code.push('\n'),
+                    Some(_) => code.push(' '),
+                    None => {}
+                }
+                position += 2;
+                continue;
+            }
+            None if character == '"' => {
+                code.push('"');
+                return position + 1;
+            }
+            Some(hashes)
+                if character == '"'
+                    && characters[position + 1..]
+                        .iter()
+                        .take(hashes)
+                        .filter(|following| **following == '#')
+                        .count()
+                        == hashes =>
+            {
+                code.push('"');
+                code.extend(std::iter::repeat_n('#', hashes));
+                return position + 1 + hashes;
+            }
+            None | Some(_) => {}
+        }
+        code.push(if character == '\n' { '\n' } else { ' ' });
+        position += 1;
+    }
+    position
+}
+
+/// 一份源码每一行的代码部分，带 1 起的行号：注释（`//` 到行尾、可嵌套的 `/* */`）去掉，字符串（普通、`b"…"`、`r#"…"#`）与
+/// 字符字面量的内容换成空格（引号留着）。字符串里的 `//` 不当注释，写在字符串里的名字也不当代码里的名字。
+fn code_lines(source: &str) -> Vec<(usize, String)> {
+    let characters: Vec<char> = source.chars().collect();
+    let mut code = String::with_capacity(source.len());
+    let mut position = 0;
+    while position < characters.len() {
+        let character = characters[position];
+        let next = characters.get(position + 1).copied();
+        let follows_an_identifier =
+            position > 0 && is_an_identifier_character(characters[position - 1]);
+        let raw_string_hashes = |after_the_prefix: usize| -> Option<usize> {
+            let hashes = characters[after_the_prefix..]
+                .iter()
+                .take_while(|following| **following == '#')
+                .count();
+            (characters.get(after_the_prefix + hashes) == Some(&'"')).then_some(hashes)
+        };
+        match (character, next) {
+            ('/', Some('/')) => {
+                while position < characters.len() && characters[position] != '\n' {
+                    position += 1;
+                }
+            }
+            ('/', Some('*')) => {
+                let mut depth = 1_usize;
+                position += 2;
+                while position < characters.len() && depth > 0 {
+                    match (characters[position], characters.get(position + 1)) {
+                        ('/', Some('*')) => {
+                            depth += 1;
+                            position += 2;
+                        }
+                        ('*', Some('/')) => {
+                            depth -= 1;
+                            position += 2;
+                        }
+                        ('\n', _) => {
+                            code.push('\n');
+                            position += 1;
+                        }
+                        (_, _) => position += 1,
+                    }
+                }
+            }
+            ('"', _) => {
+                position = blank_a_string_literal(&characters, position, None, &mut code);
+            }
+            ('r' | 'b', Some(_)) if !follows_an_identifier => {
+                let prefix_length = if character == 'b' && next == Some('r') {
+                    2
+                } else {
+                    1
+                };
+                let is_raw = character == 'r' || prefix_length == 2;
+                match (is_raw, raw_string_hashes(position + prefix_length)) {
+                    (true, Some(hashes)) => {
+                        code.extend(&characters[position..position + prefix_length + hashes]);
+                        position = blank_a_string_literal(
+                            &characters,
+                            position + prefix_length + hashes,
+                            Some(hashes),
+                            &mut code,
+                        );
+                    }
+                    (false, Some(0)) => {
+                        code.push('b');
+                        position =
+                            blank_a_string_literal(&characters, position + 1, None, &mut code);
+                    }
+                    (true | false, Some(_) | None) => {
+                        code.push(character);
+                        position += 1;
+                    }
+                }
+            }
+            ('\'', Some('\\')) => {
+                // 带转义的字符字面量（'\n'、'\''、'\u{…}'）：被转义的那一个之后找收尾的引号
+                let closing = characters
+                    .get(position + 3..)
+                    .unwrap_or_default()
+                    .iter()
+                    .position(|following| *following == '\'')
+                    .map_or(characters.len(), |offset| position + 3 + offset);
+                code.push_str("' '");
+                position = closing + 1;
+            }
+            ('\'', Some(_)) if characters.get(position + 2) == Some(&'\'') => {
+                code.push_str("' '");
+                position += 3;
+            }
+            (_, _) => {
+                // 生命周期、标签与别的字符照抄
+                code.push(character);
+                position += 1;
+            }
+        }
+    }
+    code.split('\n')
+        .take(source.split('\n').count())
         .enumerate()
-        .map(|(index, line)| {
-            (
-                index + 1,
-                line.split_once("//").map_or(line, |(code, _comment)| code),
-            )
-        })
+        .map(|(index, line)| (index + 1, line.to_string()))
         .collect()
 }
 
-/// `src/` 下一份源码里属于测试代码的行：顶格 `#[cfg(test)]`（与它下面紧跟的别的属性）之后是顶格的 `mod `、`fn `、`pub fn ` 起头的项，
-/// 从项头起到下一个顶格的 `}` 为止；`#[cfg(test)]` 下的别的项（常量、`use`）不跑代码，不收。交回 (测试代码的行, 收了几项)。
-fn test_code_lines_of_a_library_source<'source>(
-    lines: &[(usize, &'source str)],
-) -> (Vec<(usize, &'source str)>, usize) {
-    let mut collected = Vec::new();
+/// `code` 里名字 `identifier` 出现几次（前后都不是名字字符）。
+fn mentions_of(code: &str, identifier: &str) -> usize {
+    code.match_indices(identifier)
+        .filter(|(start, _)| {
+            let before = code[..*start].chars().next_back();
+            let after = code[start + identifier.len()..].chars().next();
+            !before.is_some_and(is_an_identifier_character)
+                && !after.is_some_and(is_an_identifier_character)
+        })
+        .count()
+}
+
+/// `code` 里调 `function` 几次：名字之后（隔着空白）紧跟 `(`。
+fn calls_of(code: &str, function: &str) -> usize {
+    code.match_indices(function)
+        .filter(|(start, _)| {
+            let before = code[..*start].chars().next_back();
+            let after = code[start + function.len()..].trim_start().chars().next();
+            !before.is_some_and(is_an_identifier_character) && after == Some('(')
+        })
+        .count()
+}
+
+/// 一条 `cfg(…)` 属性的条件（`attribute` 是去掉注释、字符串内容之后的属性原文）里有没有一处不在 `not(…)` 里面的 `test`：
+/// 有，这条属性下的项只在测试里编，是测试代码。
+fn cfg_predicate_includes_test_outside_a_not(attribute: &str) -> bool {
+    let mut enclosing_functions: Vec<String> = Vec::new();
+    let mut word = String::new();
+    let mut pending_word: Option<String> = None;
+    for character in attribute.chars().chain(std::iter::once(' ')) {
+        if is_an_identifier_character(character) {
+            word.push(character);
+            continue;
+        }
+        if !word.is_empty() {
+            pending_word = Some(std::mem::take(&mut word));
+        }
+        if character.is_whitespace() {
+            continue;
+        }
+        if character == '(' {
+            enclosing_functions.push(pending_word.take().unwrap_or_default());
+            continue;
+        }
+        if pending_word.take().as_deref() == Some("test")
+            && !enclosing_functions.iter().any(|function| function == "not")
+        {
+            return true;
+        }
+        if character == ')' {
+            enclosing_functions.pop();
+        }
+    }
+    false
+}
+
+/// `src/` 下一份源码里属于测试代码的行：条件含 `test`（不在 `not(…)` 里，[`cfg_predicate_includes_test_outside_a_not`]）的
+/// `#[cfg(…)]` 属性（可以缩进、可以跨几行）连同它下面的那一项（之间别的属性一并收），项到它的 `{…}` 收尾、或在最外层遇到
+/// `;`、`,`、多出来的右括号为止。交回 (测试代码的行, 收了几项)。
+fn test_code_lines_of_a_library_source(lines: &[(usize, String)]) -> (Vec<(usize, String)>, usize) {
+    let characters: Vec<(usize, char)> = lines
+        .iter()
+        .enumerate()
+        .flat_map(|(line_position, (_, code))| {
+            code.chars()
+                .chain(std::iter::once('\n'))
+                .map(move |character| (line_position, character))
+        })
+        .collect();
+    let mut collected_line_positions = std::collections::BTreeSet::new();
     let mut item_count = 0;
-    let mut position = 0;
-    while position < lines.len() {
-        if lines[position].1.trim_end() != "#[cfg(test)]" {
-            position += 1;
+    let mut line_position = 0;
+    while line_position < lines.len() {
+        if !lines[line_position].1.trim_start().starts_with("#[cfg(") {
+            line_position += 1;
             continue;
         }
-        let Some(header_offset) = lines[position + 1..]
+        let attribute_start = characters
             .iter()
-            .position(|(_, code)| !code.starts_with("#["))
-        else {
-            break;
-        };
-        let header_position = position + 1 + header_offset;
-        let header = lines[header_position].1;
-        if !(header.starts_with("mod ")
-            || header.starts_with("fn ")
-            || header.starts_with("pub fn "))
-        {
-            position = header_position;
+            .position(|(position, character)| *position == line_position && *character == '#')
+            .expect("这一行去掉前导空白以 # 起头");
+        let mut bracket_depth = 0_usize;
+        let mut attribute_end = attribute_start;
+        for (offset, (_, character)) in characters[attribute_start..].iter().enumerate() {
+            match character {
+                '[' => bracket_depth += 1,
+                ']' => {
+                    bracket_depth -= 1;
+                    if bracket_depth == 0 {
+                        attribute_end = attribute_start + offset;
+                        break;
+                    }
+                }
+                _ => {}
+            }
+        }
+        let attribute: String = characters[attribute_start..=attribute_end]
+            .iter()
+            .map(|(_, character)| *character)
+            .collect();
+        if !cfg_predicate_includes_test_outside_a_not(&attribute) {
+            line_position = characters[attribute_end].0 + 1;
             continue;
         }
+        let mut depth = 0_usize;
+        let mut body_started = false;
+        let mut item_end = characters.len() - 1;
+        for (offset, (_, character)) in characters[attribute_end + 1..].iter().enumerate() {
+            let here = attribute_end + 1 + offset;
+            match character {
+                '(' | '[' | '{' => {
+                    if *character == '{' && depth == 0 {
+                        body_started = true;
+                    }
+                    depth += 1;
+                }
+                ')' | ']' | '}' => {
+                    if depth == 0 {
+                        item_end = here;
+                        break;
+                    }
+                    depth -= 1;
+                    if depth == 0 && *character == '}' && body_started {
+                        item_end = here;
+                        break;
+                    }
+                }
+                ';' | ',' if depth == 0 => {
+                    item_end = here;
+                    break;
+                }
+                _ => {}
+            }
+        }
         item_count += 1;
-        let end_position = lines[header_position..]
-            .iter()
-            .position(|(_, code)| code.trim_end() == "}")
-            .map_or(lines.len(), |offset| header_position + offset + 1);
-        collected.extend_from_slice(&lines[header_position..end_position]);
-        position = end_position;
+        let last_line_position = characters[item_end].0;
+        collected_line_positions.extend(line_position..=last_line_position);
+        line_position = last_line_position + 1;
     }
-    (collected, item_count)
+    (
+        collected_line_positions
+            .into_iter()
+            .map(|position| lines[position].clone())
+            .collect(),
+        item_count,
+    )
 }
 
 /// 这几行里经入口取卡的调用有几处。
-fn entry_call_count(lines: &[(usize, &str)]) -> usize {
+fn entry_call_count(lines: &[(usize, String)]) -> usize {
     lines
         .iter()
         .map(|(_, code)| {
-            CARD_TAKING_CALLS_THROUGH_THE_ENTRY
+            CARD_TAKING_FUNCTIONS_THROUGH_THE_ENTRY
                 .iter()
-                .map(|call| code.matches(call).count())
+                .map(|function| calls_of(code, function))
                 .sum::<usize>()
         })
         .sum()
 }
 
-/// 这几行里绕开入口起卡的每一行。
-fn bypassing_lines(
+/// 这几行里出现了 `names` 里某个名字的每一行。
+fn lines_mentioning(
     file_relative_to_the_package: &str,
-    lines: &[(usize, &str)],
+    lines: &[(usize, String)],
+    names: &[&str],
 ) -> Vec<BypassingLine> {
     lines
         .iter()
-        .filter(|(_, code)| {
-            CARD_TAKING_CALLS_THAT_BYPASS_THE_ENTRY
-                .iter()
-                .any(|call| code.contains(call))
-        })
+        .filter(|(_, code)| names.iter().any(|name| mentions_of(code, name) > 0))
         .map(|(line_number, code)| BypassingLine {
             file_relative_to_the_package: file_relative_to_the_package.to_string(),
             line_number: *line_number,
@@ -193,7 +446,11 @@ fn every_gpu_test_takes_its_cards_through_gpu_test_start() {
         let lines = code_lines(&source);
         scanned_test_file_count += 1;
         entry_calls_under_tests += entry_call_count(&lines);
-        bypasses.extend(bypassing_lines(&relative(&path), &lines));
+        bypasses.extend(lines_mentioning(
+            &relative(&path),
+            &lines,
+            &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
+        ));
     }
     let mut library_test_item_count = 0_usize;
     let mut entry_calls_in_library_test_code = 0_usize;
@@ -202,7 +459,11 @@ fn every_gpu_test_takes_its_cards_through_gpu_test_start() {
         let (lines, item_count) = test_code_lines_of_a_library_source(&code_lines(&source));
         library_test_item_count += item_count;
         entry_calls_in_library_test_code += entry_call_count(&lines);
-        bypasses.extend(bypassing_lines(&relative(&path), &lines));
+        bypasses.extend(lines_mentioning(
+            &relative(&path),
+            &lines,
+            &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
+        ));
     }
     assert!(
         scanned_test_file_count >= 20 && library_test_item_count >= 10,
@@ -240,6 +501,10 @@ fn every_gpu_test_takes_its_cards_through_gpu_test_start() {
         "这几处绕开 gpu_unit_checks::gpu_test_start 直接起卡（没卡的机器上报不出「本次未跑」、有卡而用不上判不了红），\
          改成经 gpu_card_pool_or_report_not_run（共用一池卡、按量借）、usable_cards_or_report_not_run 或 first_usable_card_or_report_not_run 取卡：{not_allowed:#?}"
     );
+    println!(
+        "GPU_CARD_ENTRY_SCAN test_files={scanned_test_file_count} library_test_items={library_test_item_count} bypasses_allowed={}",
+        BYPASSES_ALLOWED_AFTER_THE_ENTRY_DECIDED.len()
+    );
 }
 
 #[test]
@@ -256,27 +521,17 @@ fn every_test_file_moved_onto_the_card_pool_takes_its_cards_only_from_the_pool()
             )
         });
         let lines = code_lines(&source);
-        if !lines.iter().any(|(_, code)| code.contains(CARD_POOL_CALL)) {
+        if !lines
+            .iter()
+            .any(|(_, code)| calls_of(code, CARD_POOL_FUNCTION) > 0)
+        {
             files_without_a_pool_call.push(file);
         }
-        bypasses_of_the_pool.extend(
-            lines
-                .iter()
-                .filter(|(_, code)| {
-                    CALLS_THAT_BYPASS_THE_POOL
-                        .iter()
-                        .any(|call| code.contains(call))
-                })
-                .map(|(line_number, code)| BypassingLine {
-                    file_relative_to_the_package: file.to_string(),
-                    line_number: *line_number,
-                    code: code.trim().to_string(),
-                }),
-        );
+        bypasses_of_the_pool.extend(lines_mentioning(file, &lines, &NAMES_THAT_BYPASS_THE_POOL));
     }
     assert!(
         files_without_a_pool_call.is_empty(),
-        "这几份改走卡池的文件里一处 {CARD_POOL_CALL} 都没有：{files_without_a_pool_call:?}"
+        "这几份改走卡池的文件里一处 {CARD_POOL_FUNCTION}(…) 都没有：{files_without_a_pool_call:?}"
     );
     assert!(
         bypasses_of_the_pool.is_empty(),
@@ -289,3 +544,90 @@ fn every_test_file_moved_onto_the_card_pool_takes_its_cards_only_from_the_pool()
         FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL.len()
     );
 }
+
+/// 第 10t 批 K4-4 红样本一：`use … as 别名` 引进起卡的函数、再调别名，调用那一行没有原名，`use` 那一行有：扫得出；
+/// 改走卡池的文件里给入口函数起别名同样扫得出。
+#[test]
+fn a_card_taking_function_brought_in_under_an_alias_is_caught() {
+    let sample = "use singlefs_checker_tier::gpu_unit_checks::usable_gpu_cards as cards_now;\n\
+                  use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run as first_card;\n\
+                  fn takes_cards() {\n    let _cards = cards_now();\n    let _card = first_card(\"sample\");\n}\n";
+    let lines = code_lines(sample);
+    let entry_bypasses = lines_mentioning(
+        "tests/sample.rs",
+        &lines,
+        &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
+    );
+    assert_eq!(
+        entry_bypasses
+            .iter()
+            .map(|bypass| bypass.line_number)
+            .collect::<Vec<_>>(),
+        vec![1],
+        "改名引进 usable_gpu_cards 的那一行要扫得出：{entry_bypasses:#?}"
+    );
+    let pool_bypasses = lines_mentioning("tests/sample.rs", &lines, &NAMES_THAT_BYPASS_THE_POOL);
+    assert_eq!(
+        pool_bypasses.iter().map(|bypass| bypass.line_number).collect::<Vec<_>>(),
+        vec![2],
+        "改走卡池的文件里改名引进 first_usable_card_or_report_not_run 的那一行要扫得出：{pool_bypasses:#?}"
+    );
+}
+
+/// 第 10t 批 K4-4 红样本二：同一行前面有个带 `//` 的字符串字面量，后面的起卡调用照扫；字符串里写的名字不当调用。
+#[test]
+fn a_card_taking_call_after_a_string_holding_two_slashes_on_the_same_line_is_caught() {
+    let sample = "fn takes_cards() {\n    let _address = \"http://example\"; let _cards = singlefs_checker_tier::gpu_unit_checks::usable_gpu_cards();\n    let _message = \"usable_gpu_cards() 写在字符串里不算\";\n    let _raw = r#\"// 原始字符串里的也不算\"#; let _card = singlefs_checker_tier::gpu_unit_checks::gpu_cards_by_priority(&[]);\n}\n";
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
+        vec![2, 4],
+        "字符串里的 // 之后的调用要扫得出，字符串里写的名字不算：{bypasses:#?}"
+    );
+}
+
+/// 第 10t 批 K4-4 红样本三：库源码里条件不只是 `test` 的 `#[cfg(all(test, feature = "gpu"))]` 与 `impl` 块里缩进的 `#[cfg(test)]`
+/// 下的项都是测试代码、照扫；`#[cfg(not(test))]` 下的是产品代码、不扫。
+#[test]
+fn test_code_under_a_compound_or_indented_cfg_test_in_a_library_source_is_caught() {
+    let sample = "pub fn product() {}\n\
+                  \n\
+                  #[cfg(all(test, feature = \"gpu\"))]\n\
+                  mod gpu_tests {\n\
+                  \x20   #[test]\n\
+                  \x20   fn takes_cards() {\n\
+                  \x20       let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
+                  \x20   }\n\
+                  }\n\
+                  \n\
+                  impl Something {\n\
+                  \x20   #[cfg(test)]\n\
+                  \x20   fn helper() {\n\
+                  \x20       let _cards = crate::gpu_unit_checks::gpu_cards_by_priority(&[]);\n\
+                  \x20   }\n\
+                  \n\
+                  \x20   #[cfg(not(test))]\n\
+                  \x20   fn product_only() {\n\
+                  \x20       let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
+                  \x20   }\n\
+                  }\n";
+    let (lines, item_count) = test_code_lines_of_a_library_source(&code_lines(sample));
+    assert_eq!(item_count, 2, "两项测试代码：{lines:#?}");
+    let bypasses = lines_mentioning(
+        "src/sample.rs",
+        &lines,
+        &CARD_TAKING_FUNCTIONS_THAT_BYPASS_THE_ENTRY,
+    );
+    assert_eq!(
+        bypasses.iter().map(|bypass| bypass.line_number).collect::<Vec<_>>(),
+        vec![7, 14],
+        "cfg(all(test, …)) 与缩进的 cfg(test) 下的起卡调用要扫得出，cfg(not(test)) 下的不扫：{bypasses:#?}"
+    );
+}
````

### 10t mutations-append.tsv

````text
第10t批 显存账 K4-1 额度变量越过配置给卡的额度	crates/singlefs-harness/src/gpu_memory_ledger.rs	                .map_or(u64::MAX, |configured| {\n                    configured.saturating_sub(mebibytes_left_for_the_driver)\n                })	                .map_or(u64::MAX, |_configured| u64::MAX)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_environment_quota_larger_than_the_configured_card_quota_stays_within_the_configured_card_quota	an_environment_quota_larger_than_the_configured_card_quota_stays_within_the_configured_card_quota
第10t批 显存账 K4-2 被杀而没收尸的僵尸判成活着	crates/singlefs-harness/src/gpu_memory_ledger.rs	        Some("Z" | "X") => false,	        Some("never-a-process-state") => false,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_killed_process_that_its_parent_has_not_reaped_yet_is_judged_not_alive	a_killed_process_that_its_parent_has_not_reaped_yet_is_judged_not_alive
第10t批 显存账 K4-2 开账的上界不算登记进程的驱动那一份（后读的进程把先登记的驱动那一份记两次）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                        records.booked_mebibytes_by_every_live_process(this_process, driver),	                        records.booked_mebibytes_by_every_live_process(this_process, 0),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_process_whose_free_memory_reading_already_lacks_the_driver_share_of_a_registered_process_counts_that_share_once	a_process_whose_free_memory_reading_already_lacks_the_driver_share_of_a_registered_process_counts_that_share_once
第10t批 显存账 K4-3 估计没有上界（外人后来占了卡，旧估计照用）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        Some(previous) => previous.clamp(lowest, highest),	        Some(previous) => previous.max(lowest),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_outsider_taking_memory_after_the_first_process_opened_its_ledger_caps_a_later_process_by_the_free_memory_it_reads	an_outsider_taking_memory_after_the_first_process_opened_its_ledger_caps_a_later_process_by_the_free_memory_it_reads
第10t批 显存账 K4-3 估计没有下界（外人后来走了，旧估计照用）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        Some(previous) => previous.clamp(lowest, highest),	        Some(previous) => previous.min(highest),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_outsider_leaving_after_the_first_process_opened_its_ledger_lets_a_later_process_book_what_it_now_reads	an_outsider_leaving_after_the_first_process_opened_its_ledger_lets_a_later_process_book_what_it_now_reads
第10t批 显存账 K4-3 估计只取现读的空闲（卡被订满时后起的进程判成放不下）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        Some(previous) => previous.clamp(lowest, highest),	        Some(_previous) => lowest,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_the_first_process_booked_full_makes_a_later_process_wait_instead_of_refusing_the_card	a_card_the_first_process_booked_full_makes_a_later_process_wait_instead_of_refusing_the_card
第10t批 显存账 K4-3 估计照抄旧的、不夹进现读的区间	crates/singlefs-harness/src/gpu_memory_ledger.rs	        Some(previous) => previous.clamp(lowest, highest),	        Some(previous) => previous,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_memory_estimate_is_the_previous_one_clamped_between_the_free_memory_read_now_and_that_plus_the_live_bookings	the_memory_estimate_is_the_previous_one_clamped_between_the_free_memory_read_now_and_that_plus_the_live_bookings
第10t批 显存账 K4-3 借到了也不登记这个进程（驱动那一份不进账）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                                .push(self.settings.this_process);	                                .retain(|_process| true);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_process_is_registered_on_a_card_only_once_its_first_booking_or_its_driver_share_fits	a_process_is_registered_on_a_card_only_once_its_first_booking_or_its_driver_share_fits
第10t批 源码扫描 K4-4 只认带括号的调用（改名引进的绕得过）	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	        .filter(|(_, code)| names.iter().any(|name| mentions_of(code, name) > 0))	        .filter(|(_, code)| names.iter().any(|name| calls_of(code, name) > 0))	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- a_card_taking_function_brought_in_under_an_alias_is_caught	a_card_taking_function_brought_in_under_an_alias_is_caught
第10t批 源码扫描 K4-4 不认字符串（字符串里的 // 当注释起头）	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	            ('"', _) => {	            ('"', _) if false => {	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- a_card_taking_call_after_a_string_holding_two_slashes_on_the_same_line_is_caught	a_card_taking_call_after_a_string_holding_two_slashes_on_the_same_line_is_caught
第10t批 源码扫描 K4-4 只认 cfg 里单独一个 test（cfg(all(test, …)) 下的不扫）	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	        if pending_word.take().as_deref() == Some("test")	        if pending_word.take().as_deref() == Some("test") && enclosing_functions.len() == 1	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- test_code_under_a_compound_or_indented_cfg_test_in_a_library_source_is_caught	test_code_under_a_compound_or_indented_cfg_test_in_a_library_source_is_caught
第10t批 崩溃放量 K4-4 起过卡池的进程照样另起不记账的设备	crates/singlefs-checker-tier/src/crash_amplification.rs	    if let Some(pool) = crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process()\n    {	    if let Some(pool) = crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process()\n        .filter(|_pool| false)\n    {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_pipeline_records_checks_and_reuses -- a_placement_only_run_reports_the_states_judged_at_recording_as_reused	a_placement_only_run_reports_the_states_judged_at_recording_as_reused
````

## 第 10u 批

````diff
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -722,10 +722,25 @@
 enum ChainRecordView {
     LastPage,
     NextPage(PointerView),
-    /// 记录宽不是 88、kind 不是 1、有无下一片不是 0 或 1、无下一片而位置指针不全零、有下一片而位置指针全零。
+    /// 记录宽不是 88、kind 不是 1、有无下一片不是 0 或 1、无下一片而位置指针不全零、有下一片而位置指针全零、
+    /// 有下一片而位置指针头部有一段不是第一版的取值（[`pointer_head_fields_take_the_first_version_values`]）。
     Malformed,
 }
 
+/// 一条指针头部 extent 偏移及之前那几段——MAC 16 与 nonce 12、算法类型、压缩算法码、压后长度、extent 偏移——都是第一版的取值
+/// （I-2.4 在走读跟随的指针上逐段判的那几格，`image::judge_pointer_mac_and_nonce_are_zero` 等三个判法读的同一组字段）。
+/// 实例表链指针记录说「有下一片」时只在这几段都过时算合法：有一段不过是 I-3.14 第五形，这一片解不开（用户 2026-10-06 定 K2-1 甲，
+/// `research/prompts/m3-batch10o-code-r1-forks.md`；实现 `InstanceTableChainRecord::parse` 经 `NodePointer::read_with_head_fields_judgement_from`
+/// 判同一组，D19（块指针的结构与宽度预算） 已定项 3、D9（加密） 已定项 14、C581）。
+fn pointer_head_fields_take_the_first_version_values(pointer: &PointerView) -> bool {
+    let fields = pointer.encryption_and_compression_fields;
+    pointer.is_mac_and_nonce_all_zero
+        && fields.algorithm_type == 0
+        && fields.compression_algorithm_code == 0
+        && fields.compressed_length == 0
+        && pointer.is_extent_offset_zero
+}
+
 /// 链指针记录里位置指针的偏移：kind 1 字节 + 有无下一片 1 字节。
 const CHAIN_RECORD_POINTER_OFFSET: usize = 2;
 /// 实例表记录宽（D18（块里携带什么信息） 已定项 11）：链指针记录与行记录同宽。
@@ -738,7 +753,9 @@
     let pointer = parse_node_pointer(&record[CHAIN_RECORD_POINTER_OFFSET..]);
     match record[1] {
         0 if pointer.all_zero => ChainRecordView::LastPage,
-        1 if !pointer.all_zero => ChainRecordView::NextPage(pointer),
+        1 if !pointer.all_zero && pointer_head_fields_take_the_first_version_values(&pointer) => {
+            ChainRecordView::NextPage(pointer)
+        }
         _flag_or_pointer_that_does_not_match => ChainRecordView::Malformed,
     }
 }
@@ -3179,8 +3196,9 @@
     /// 迭代上界：第 k 片要求容器号是 k，走过的单元不再走（`visited_units`，按读回的那一份的落点认），每一轮是盘上一个不同的单元。
     /// 提前出口，链都不再往下走：读不出、身份不对（被引用的单元坏了或换了人，记进走读失败；头用不了由 `judge_unit_header` 记）；
     /// 容器记录不合法、I-3.14 不成立（这一片自己写错了，只红 I-1.7 / I-3.14 那一格；这一片解不开，行不收）；走到已经走过的单元（前一条根走过这条链）。
-    /// I-3.14 不成立的四形（kind 不是 0 或 1、片中间有链指针记录、行的 flags 不是 0、最后一条不是合法的链指针记录）一律当这一片解不开，
-    /// 与实现 `InstanceTablePage::parse` 同一个读法（用户 2026-10-06 给 D18（块里携带什么信息） 已定项 11 补的那一句）。
+    /// I-3.14 不成立的五形（kind 不是 0 或 1、片中间有链指针记录、行的 flags 不是 0、最后一条不是合法的链指针记录、它说有下一片而位置指针头部
+    /// 有一段不是第一版的取值）一律当这一片解不开，与实现 `InstanceTablePage::parse` 同一个读法（用户 2026-10-06 给 D18（块里携带什么信息）
+    /// 已定项 11 补的那一句；第五形用户 2026-10-06 定 K2-1 甲）。第五形那条指针不跟随，走读不往下走，它头部那几段 I-2.4 不判（走读不跟随的指针 I-2.4 不判）。
     fn walk_instance_table_chain(
         &mut self,
         first_page_pointer: &PointerView,
@@ -3276,7 +3294,7 @@
                 chain != ChainRecordView::Malformed,
                 || {
                     format!(
-                        "{what}：最后一条不是合法的链指针记录（kind 1、有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零）"
+                        "{what}：最后一条不是合法的链指针记录（kind 1、有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零且头部 MAC / nonce、算法类型、压缩算法码、压后长度、extent 偏移都是第一版的取值）"
                     )
                 },
             );
@@ -4762,7 +4780,8 @@
 
 /// 沿实例表链（根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片，D18（块里携带什么信息） 已定项 11）把第 1 片起每一片的
 /// 指针记进这条根的引用集合。只读不判：一片读不出、解不开、身份不是 (0, 4, 片序号, 0)、I-3.14 不成立（链指针记录之前有不是行的记录、
-/// 行的 flags 不是 0、最后一条不是合法的链指针记录），都只如实记成数不全（判定归主走读：I-2.1、I-1.1、I-3.14）。
+/// 行的 flags 不是 0、最后一条不是合法的链指针记录、它说有下一片而位置指针头部有一段不是第一版的取值），都只如实记成数不全
+/// （判定归主走读：I-2.1、I-1.1、I-3.14）。第五形那条指针不记进引用集合，与主走读不跟随它同一个读法（[`chain_record_view`]）。
 /// 迭代上界：第 k 片要求容器号是 k，每一轮读的是盘上一个不同的单元；跨轮带的只有下一片的指针；提前出口是走到最后一片与上面那几样读不下去。
 fn note_instance_table_pages_after_the_first(
     reader: &dyn ImageReader,
@@ -6245,7 +6264,8 @@
 /// 一条根自己指着的那张实例表的全部行，沿链读（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片）。
 /// 只读不判：哪一片读不出（两份的整单元 CRC 都对不上位置条目）、解不开（头判不过、类不是码 3、记录类型不是 4、记录宽不是 88、
 /// 记录数 × 记录宽超过声明长度或声明长度超过容量、身份不是 (0, 4, 片序号, 0)、链指针记录之前有不是行的记录、行的 flags 不是 0、
-/// 最后一条不是合法的链指针记录）⇒ None，判定归主走读那一遍（I-2.1、I-1.7、I-1.1、I-3.14）。I-3.14 的四形与主走读、与实现同一个读法：解不开。
+/// 最后一条不是合法的链指针记录，含它说有下一片而位置指针头部有一段不是第一版的取值）⇒ None，判定归主走读那一遍（I-2.1、I-1.7、I-1.1、I-3.14）。
+/// I-3.14 的五形与主走读、与实现同一个读法：解不开。
 /// 迭代上界：第 k 轮要求读到的单元身份里容器号是 k，两轮读到同一个单元就要它的容器号同时等于两个数 ⇒ 每一轮读的是盘上不同的单元，
 /// 轮数不超过盘上装得下的 32 KiB 单元数。跨轮带的是下一片的指针、片序号与已经接起来的行；提前出口只有读不出、解不开。
 fn instance_table_rows_of_root_without_judging(
@@ -9936,7 +9956,7 @@
 /// 那张表读不全时滤不全，被抛弃时间线上的根可能进了候选集。I-9.29（同号只许一个出生代） 跨版本比，这时记判不了
 /// （`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」）：被抛弃的线上同号可以是另一个对象。
 /// 读没读全看的是候选集滤法用的那一遍（主走读，`Walk::walk_instance_table_chain`），不另读一遍：另一个读法（只读不判的
-/// [`instance_table_rows_of_root_without_judging`]）拒收的比主走读多（载荷 CRC 对不上；I-3.14 的四形两边一样当解不开，第 10k 批），
+/// [`instance_table_rows_of_root_without_judging`]）拒收的比主走读多（载荷 CRC 对不上；I-3.14 的五形两边一样当解不开，第 10k 批、第 10u 批），
 /// 用它当门会在候选集已经滤全时报判不了（判决 `research/prompts/m3-batch23-code-r1-main-verification.md` 第二节 K2 N5）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum AbandonedRootsFilteringOfTheCandidateSet {
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -602,7 +602,9 @@
     return 1u;
 }
 
-// 链指针记录：0 最后一片、1 有下一片、2 坏
+// 链指针记录：0 最后一片、1 有下一片、2 坏（walk.rs chain_record_view）。有下一片还要位置指针头部 extent 偏移及之前那几段都是第一版的取值
+// （MAC / nonce、算法类型、压缩算法码、压后长度经 crash_judge_common.wgsl 的 pointer_encryption_and_compression_fields_are_the_first_version，
+// extent 偏移经 pointer_extent_offset_is_zero）：有一段不过是 I-3.14 第五形，这一片解不开（用户 2026-10-06 定 K2-1 甲，第 10u 批）
 fn w_chain_record_kind(base: u32, o: u32, width: u32) -> u32 {
     if (width != 88u || rd8(base, o) != 1u) {
         return 2u;
@@ -612,7 +614,8 @@
     if (flag == 0u && pointer_zero) {
         return 0u;
     }
-    if (flag == 1u && !pointer_zero) {
+    if (flag == 1u && !pointer_zero && pointer_encryption_and_compression_fields_are_the_first_version(base, o + 2u)
+        && pointer_extent_offset_is_zero(base, o + 2u)) {
         return 1u;
     }
     return 2u;
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -516,6 +516,7 @@
             }
         }
         let last = packed_record_offset(packed, packed.record_count - 1u);
+        // 有下一片而位置指针头部不是第一版的取值（I-3.14 第五形，第 10u 批）同样交 2u：这条指针不记、数不全
         let chain = w_chain_record_kind(packed.base, last, packed.record_width);
         if (chain == 0u) {
             return;
@@ -1165,6 +1166,7 @@
             rows = rows + 1u;
         }
         let last = 136u + (count - 1u) * width;
+        // 有下一片而位置指针头部不是第一版的取值（I-3.14 第五形，第 10u 批）同样交 2u：读不出
         let chain = w_chain_record_kind(s.base, last, width);
         if (chain == 0u) {
             c_r_row_count = rows;
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1696,6 +1696,7 @@
         w_judge(INVARIANT_I_3_14, kinds_are_registered);
         w_judge(INVARIANT_I_3_14, chain_records_only_last);
         w_judge(INVARIANT_I_3_14, row_flags_are_zero);
+        // 最后一条是合法的链指针记录那一格：有下一片而位置指针头部不是第一版的取值（第五形，第 10u 批）同样 chain == 2u（w_chain_record_kind）
         w_judge(INVARIANT_I_3_14, chain != 2u);
         // I-3.14 不成立的一片解不开（用户 2026-10-06 给 D18 已定项 11 补的那一句）：行不收、链不往下走、不记走读失败、没走到末片
         if (!(kinds_are_registered && chain_records_only_last && row_flags_are_zero)) {
--- a/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
@@ -274,7 +274,9 @@
         if (flag == 0u && next.all_zero) {
             return true;
         }
-        if (flag == 1u && !next.all_zero) {
+        // 有下一片还要位置指针头部 extent 偏移及之前那几段都是第一版的取值（Ptr 的 head_ok 与 extent_offset_zero；walk.rs chain_record_view 同判）：
+        // 有一段不过是 I-3.14 第五形，这一片解不开、读不出整张表（用户 2026-10-06 定 K2-1 甲，第 10u 批）
+        if (flag == 1u && !next.all_zero && next.head_ok && next.extent_offset_zero) {
             pointer = next;
             page_index = add64(page_index, u64_of(1u));
             continue;
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -26,6 +26,7 @@
 //! 主走读走不到末片、I-9.29 不比），同一段历史再加片中间多一条链指针记录、第一行 flags 非 0 两条流，三形各一条。
 //! 第 10n 批：同一段历史再加两条，都整份比——第一行 flags 非 0 且 T 抬到最新根的 txg（只读不判的读法拒收这一片，I-3.12 两边都不判它）；
 //! 拆成两片、第 0 片链指针盘 0 那条位置条目指回第 0 片自己（攻方 H-b：「走过」按读回那一份的落点认，第 1 片照走、走到末片）。
+//! 第 10u 批：同一段历史、拆成两片（实例 2 那一行在第 1 片）、第 0 片链指针 MAC 头一个字节写 1（I-3.14 第五形，第 0 片解不开）一条流整份比。
 //! 第 26 批：inode 树根条目类型段 0、身份三段清零，叶容器盘 0 那一份换成码 2 节点的陈旧字节（攻方 H1：头里的类恰是类型段那一类的副本也要自证）
 //! 一条流钉 CPU 红 I-9.2、不红 I-7.2，GPU 逐位跟上。
 //! 第 27 批：inode 叶容器类标签改成不登记的码、不重封（孩子读不出），树根条目类型段 0 带身份、类型段 7 各一条流整份比；
@@ -6614,6 +6615,30 @@
                 point_the_first_location_of_the_chain_record_at_slot(bytes, first_page_slot);
             });
         }
+        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero => {
+            let (rows_of_the_abandoned_instance, other_rows) =
+                rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
+                    &flow,
+                    &newest_instance_table,
+                );
+            rebuild_the_newest_instance_table_as_a_chain(
+                &mut flow,
+                &newest_instance_table,
+                &[other_rows, rows_of_the_abandoned_instance],
+                PagesAfterTheFirstInTheFlow::Intact,
+            );
+            let first_page_copies = last_unit_copies(
+                &flow,
+                "改造成两片之后的第 0 片",
+                &is_an_instance_table_unit,
+                false,
+            );
+            corrupt_units_and_reseal_up_to_the_root(
+                &mut flow,
+                &first_page_copies,
+                &set_the_first_mac_byte_of_the_chain_record_pointer,
+            );
+        }
         ChangeOfTheNewestInstanceTable::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance => {
             let (rows_of_the_abandoned_instance, other_rows) =
                 rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
@@ -6687,6 +6712,12 @@
     /// 读第 1 片时盘 0 那一份对不上、读回的是盘 1 那一份：「走过」按读回那一份的落点认，第 1 片没走过，主走读走到末片、门开，
     /// 候选集里没有实例 2 那一行、滤不掉 C，CPU 红 I-9.29。GPU 按第 0 条位置条目认「走过」时第 1 片被当成走过、门关，GPU 不红而 CPU 红。
     SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage,
+    /// 拆成两片：第 0 片放实例 2 之外的行、链指针记录指第 1 片；第 1 片放实例 2 那一行与末片的链指针记录，两份完好。第 0 片链指针记录里
+    /// 那条指针 MAC 的头一个字节写 1、位置条目不动，重封到根（I-3.14 第五形：有下一片而位置指针头部有一段不是第一版的取值，用户 2026-10-06
+    /// 定 K2-1 甲，第 10u 批）。CPU 三个读法都当第 0 片解不开：主走读红 I-3.14、行不收、链不往下走（那条指针 I-2.4 不判）、门关；
+    /// 只读不判的读法与记录核对器读恢复那一版的实例表（`InstanceTableOfRootRecord::read`）读不出；引用集合那一遍记数不全。
+    /// GPU 照走第 1 片时：走读内核不红 I-3.14、红 I-2.4，恢复内核读出实例 2 那一行、把 C 那几次发布判成被抛弃，都与 CPU 不一致。
+    SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero,
     /// 唯一那一片里去掉被抛弃的实例 2 那一行，链指针记录的「有无下一片」写 1、指针照旧全零（不合法的链指针记录）、重封到根
     /// （判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第三节第 2 条，GPU 关门那一侧「链指针记录不合法」一形）：
     /// 主走读没走到末片，门关，CPU 报 I-9.29 部分没判、不红，GPU 不置位、不比。门要是开了，候选集里没有实例 2 那一行、滤不掉 C，
@@ -6722,6 +6753,9 @@
             Self::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage => {
                 "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-pointer-on-device-zero-naming-the-first-page"
             }
+            Self::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero => {
+                "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-named-through-a-pointer-whose-mac-is-not-zero"
+            }
             Self::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance => {
                 "abandoned-root-naming-another-object-birth-newest-instance-table-with-an-invalid-chain-record"
             }
@@ -7175,6 +7209,34 @@
     bytes[first_location_slot..first_location_slot + 6].copy_from_slice(&slot.to_le_bytes()[..6]);
 }
 
+/// 指针头部 MAC 16 字节的头一个字节在指针里的偏移（D19（块指针的结构与宽度预算） 已定项 11 的偏移表：MAC 从偏移 0 起）。
+const POINTER_MAC_FIRST_BYTE_OFFSET: usize = 0;
+
+/// 实例表一片末尾那条指下一片的链指针记录里，位置指针 MAC 的头一个字节写 1（I-3.14 第五形：有下一片而位置指针头部不是第一版的取值；
+/// 位置条目不动，指的仍是下一片；重封归调用方）。
+fn set_the_first_mac_byte_of_the_chain_record_pointer(bytes: &mut [u8]) {
+    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
+    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
+    let chain_record_start =
+        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
+    assert_eq!(
+        (
+            bytes[chain_record_start],
+            bytes[chain_record_start + INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET]
+        ),
+        (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 1),
+        "最后一条是指下一片的链指针记录"
+    );
+    let first_mac_byte = chain_record_start
+        + INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET
+        + POINTER_MAC_FIRST_BYTE_OFFSET;
+    assert_eq!(
+        bytes[first_mac_byte], 0,
+        "照第 0 片指针的头造的链指针 MAC 全零"
+    );
+    bytes[first_mac_byte] = 1;
+}
+
 /// 实例表一片的第一条（实例 3 写行那一片里是一行）flags 写成退役的 bit0（重封归调用方）。
 fn set_the_flags_of_the_first_row_to_the_retired_bit(bytes: &mut [u8]) {
     let first_row_flags = PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_ROW_FLAGS_OFFSET;
@@ -7934,6 +7996,20 @@
     );
 }
 
+/// I-3.14 第五形的流（[`ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero`]，
+/// 第 10u 批），判法同上，另钉 CPU 一个状态都不红 I-2.4：那条指针不跟随、它的头部不判。三个内核都要把这一形当解不开——
+/// 走读内核与扫描内核经 `crash_judge_checker_common.wgsl` 的 `w_chain_record_kind`，恢复内核在 `load_instance_table_of_the_effective_root` 里判。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_mac_is_not_zero_does_not_decode(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_mac_is_not_zero_does_not_decode",
+        "crash-judge-gpu-i314-newest-instance-table-chain-pointer-mac-not-zero",
+        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero,
+        &["I-2.4"],
+    );
+}
+
 /// 行的 flags 不是 0、那一行的 T 抬到最新根的 txg 那一形的流
 /// （[`ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBitAndItsPublishedTxgRaisedToTheNewestRoot`]，第 10n 批），判法同上，
 /// 另钉 CPU 一个状态都不红 I-3.12：只读不判的读法拒收这一片，指着它的根不判。GPU 扫描内核 `r_instance_rows` 不判 flags 时照收这一行、
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -4322,6 +4322,99 @@
     }
 }
 
+/// 指针头部 MAC 16 字节的头一个字节在指针里的偏移（D19（块指针的结构与宽度预算） 已定项 11 的偏移表：MAC 从偏移 0 起）。
+const POINTER_MAC_FIRST_BYTE_OFFSET: usize = 0;
+
+/// I-3.14 第五形的造法（用户 2026-10-06 定 K2-1 甲：「有下一片而位置指针头部有一段不是第一版的取值」算这一片解不开，
+/// `research/prompts/m3-batch10o-code-r1-forks.md` K2-1 那一行；攻方的形在 `research/prompts/m3-batch10o-code-r1-opus-output.md`「K2-1 打中」）：
+/// 唯一那一片末尾的链指针记录改成「有下一片」，位置指针只把偏移 `offset_in_the_pointer` 那一字节写成 1、其余照旧全零。
+/// 指针因此不全零（「有下一片时不全零」那一句过得去），头部那一段不是第一版的取值。重封归调用方。
+fn make_the_chain_record_name_a_next_page_through_a_pointer_whose_head_byte_is_not_zero(
+    bytes: &mut [u8],
+    offset_in_the_pointer: usize,
+) {
+    let record_count = usize::from(u16::from_le_bytes([
+        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
+        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
+    ]));
+    let pointer_start = INSTANCE_TABLE_FIRST_ROW_START
+        + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES
+        + CHAIN_RECORD_POINTER_OFFSET;
+    assert!(
+        bytes[pointer_start..pointer_start + NODE_POINTER_BYTES]
+            .iter()
+            .all(|byte| *byte == 0),
+        "末片的链指针记录位置指针清零占位"
+    );
+    bytes[pointer_start - CHAIN_RECORD_POINTER_OFFSET + CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET] =
+        CHAIN_RECORD_HAS_NEXT_PAGE;
+    bytes[pointer_start + offset_in_the_pointer] = 1;
+}
+
+/// I-3.14 第五形的坏镜像（MAC 那一段）：最新根的实例表唯一那一片末尾的链指针记录改成有下一片、位置指针 MAC 头一个字节写 1，重封、补到根槽。
+/// 实现读到它这一片解不开（`InstanceTableChainRecord::parse` 经 `NodePointer::read_with_head_fields_judgement_from` 判 MAC 非 0，交 None）；
+/// 池级 checker 同一个读法（第 10u 批）：I-3.14 红（最后一条是合法的链指针记录那一格），这一片的行不收、链不往下走、走不到末片，
+/// 判红的另两条与 I-3.8 的理由同另四形（[`assert_the_newest_instance_table_page_does_not_decode`]：I-3.8 一行都没收、报不适用），
+/// I-9.29 报部分没判。那条指针不跟随，它的头部 I-2.4 不判、不红。
+/// 判别力（`crates/mutations.tsv` 里点名这条的行）：「有下一片」不看头部（或不看 MAC 那一段）时主走读沿这条指针往下走——I-2.4 红、
+/// 指到的落点读不出记走读失败、I-3.14 不红，判红的集合对不上。
+#[test]
+fn a_chain_record_naming_a_next_page_through_a_pointer_whose_mac_is_not_zero_on_the_newest_instance_table_does_not_decode(
+) {
+    let (image, instance_table_slot) =
+        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
+            "known-bad-instance-table-record-form-chain-pointer-mac-not-zero",
+            |bytes| {
+                make_the_chain_record_name_a_next_page_through_a_pointer_whose_head_byte_is_not_zero(
+                    bytes,
+                    POINTER_MAC_FIRST_BYTE_OFFSET,
+                );
+            },
+            true,
+        );
+    assert_the_newest_instance_table_page_does_not_decode(
+        &image,
+        &format!(
+            "最新根的实例表（槽 {instance_table_slot}）链指针记录有下一片、位置指针 MAC 不全零"
+        ),
+    );
+    match verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
+        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
+            unjudged.starts_with("最新根的实例表读不出"),
+            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::NotApplicable(_)) => panic!(
+            "主走读没沿最新根的实例表链走到末片（槽 {instance_table_slot} 链指针 MAC 不全零）：I-9.29 要报部分没判，实际 {other_verdict:?}"
+        ),
+    }
+}
+
+/// I-3.14 第五形的坏镜像（extent 偏移那一段）：同上一条，位置指针写 1 的是头部 extent 偏移的低字节（偏移 32）。判红的与 I-3.8 的理由同另四形。
+/// 判别力：「有下一片」不看 extent 偏移那一段时主走读沿这条指针往下走，判红的集合对不上。
+#[test]
+fn a_chain_record_naming_a_next_page_through_a_pointer_whose_extent_offset_is_not_zero_on_the_newest_instance_table_does_not_decode(
+) {
+    let (image, instance_table_slot) =
+        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
+            "known-bad-instance-table-record-form-chain-pointer-extent-offset-not-zero",
+            |bytes| {
+                make_the_chain_record_name_a_next_page_through_a_pointer_whose_head_byte_is_not_zero(
+                    bytes,
+                    POINTER_EXTENT_OFFSET_FIELD_OFFSET,
+                );
+            },
+            true,
+        );
+    assert_the_newest_instance_table_page_does_not_decode(
+        &image,
+        &format!(
+            "最新根的实例表（槽 {instance_table_slot}）链指针记录有下一片、位置指针 extent 偏移非 0"
+        ),
+    );
+}
+
 /// 根环里 (checkpoint_txg, 实例) 最大的那条根（择根序）的根记录 512 字节。
 fn newest_root_record_bytes(image: &MemoryPool) -> Vec<u8> {
     root_slots()
````

### 10u mutations-append.tsv

````text
第 10u 批 I-3.14 第五形：链指针记录的「有下一片」不看位置指针头部（主走读、只读不判的读法、引用集合那一遍都照走那条指针）	crates/singlefs-checker/src/walk.rs	        1 if !pointer.all_zero && pointer_head_fields_take_the_first_version_values(&pointer) => {	        1 if !pointer.all_zero => {	-p singlefs-harness --release --test checker_known_bad_images -- a_chain_record_naming_a_next_page_through_a_pointer_whose	a_chain_record_naming_a_next_page_through_a_pointer_whose_mac_is_not_zero_on_the_newest_instance_table_does_not_decode
第 10u 批 I-3.14 第五形：链指针的头部判法不看 MAC / nonce 那一段	crates/singlefs-checker/src/walk.rs	    pointer.is_mac_and_nonce_all_zero\n        && fields.algorithm_type == 0	    true\n        && fields.algorithm_type == 0	-p singlefs-harness --release --test checker_known_bad_images -- a_chain_record_naming_a_next_page_through_a_pointer_whose_mac	a_chain_record_naming_a_next_page_through_a_pointer_whose_mac_is_not_zero_on_the_newest_instance_table_does_not_decode
第 10u 批 I-3.14 第五形：链指针的头部判法不看 extent 偏移那一段	crates/singlefs-checker/src/walk.rs	        && pointer.is_extent_offset_zero\n}	        && true\n}	-p singlefs-harness --release --test checker_known_bad_images -- a_chain_record_naming_a_next_page_through_a_pointer_whose_extent_offset	a_chain_record_naming_a_next_page_through_a_pointer_whose_extent_offset_is_not_zero_on_the_newest_instance_table_does_not_decode
第 10u 批 GPU 判器 I-3.14 第五形：w_chain_record_kind 有下一片不看位置指针头部（走读内核与扫描内核照走那条指针）	crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl	    if (flag == 1u && !pointer_zero && pointer_encryption_and_compression_fields_are_the_first_version(base, o + 2u)\n        && pointer_extent_offset_is_zero(base, o + 2u)) {	    if (flag == 1u && !pointer_zero) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_mac_is_not_zero_does_not_decode	the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_mac_is_not_zero_does_not_decode
第 10u 批 GPU 判器 I-3.14 第五形：恢复内核读实例表时有下一片不看位置指针头部（照读第 1 片的行）	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	        if (flag == 1u && !next.all_zero && next.head_ok && next.extent_offset_zero) {	        if (flag == 1u && !next.all_zero) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_mac_is_not_zero_does_not_decode	the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_mac_is_not_zero_does_not_decode
````

## 第 10v 批

````diff
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
index 14ca2c9..6ee1d3c 100644
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -5,6 +5,10 @@
 //! （用户 2026-09-29 定「一个显卡 任务满了才去分另外一个」「一边占一点绝对不合理」）；
 //! 账只拦放不下的，不限同时借几份、也不限一台机器上同时跑几个 GPU 测试进程（用户 2026-10-06 定「开到最大的并发和gpu」
 //! 「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」）。
+//! 崩溃放量产品路径在起过卡池的进程里也走这份账：第 ① 段每张装了状态的卡按装的状态数借草稿区（[`scratch_states_to_book_on_each_card`]，
+//! 只记在那张卡上，[`GpuMemoryLedger::book_on_the_card_waiting_for_room`]），等满了照 GPU 用例那一路报「本次未跑」
+//! （[`scratch_booking_refusal_outcome`]，同一个记号 [`GPU_NOT_RUN_MARKER`]）；没起过卡池的进程照旧不进账，是哪几种调用方写在
+//! `singlefs_checker_tier::crash_amplification` 的文件头。
 //!
 //! 账跨进程：每张卡一份账本文件，放在工作根下的 [`GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME`]，读写前对同名的锁文件 `flock`，
 //! 改完写到临时文件再改名换上（进程在写的半路被杀，账本还是改之前那一份）。账本里记三样：记账的进程合起来能在这张卡上占多少显存
@@ -55,9 +59,10 @@ pub fn states_in_one_dispatch_capped_by_the_state_count(
 
 /// 一张卡的账能借给这个进程多少（MiB，这个进程自己的驱动那一份已经扣掉）。
 /// 设了 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（`card_quota_from_the_environment_mebibytes`）就取三者里最小的：这个额度、
-/// `GPU_CARDS_BY_PRIORITY` 配给这张卡的额度减驱动的固定开销（`configured_card_quota_mebibytes`，没配给这张卡额度的没有这一项）、
-/// 记账进程合起来能占的显存减驱动的固定开销（第 10t 批 K4-1：额度变量不越过配置给卡的额度）；
-/// 没设照旧是这张卡给崩溃放量的额度（`budget_mebibytes`，配置的额度或那份显存的 3/4）减驱动的固定开销
+/// 配置给卡的额度减驱动的固定开销、记账进程合起来能占的显存减驱动的固定开销（第 10t 批 K4-1：额度变量不越过配置给卡的额度）。
+/// 配置给卡的额度：`GPU_CARDS_BY_PRIORITY` 配给这张卡的（`configured_card_quota_mebibytes`）；没配显卡优先级（`None`）时由
+/// 没配额度时默认的那一份代替，即 `budget_mebibytes`（那份显存的 3/4，第 10v 批：主 agent 定没配优先级时 3/4 那一份也并进最小值）。
+/// 没设额度变量照旧是这张卡给崩溃放量的额度（`budget_mebibytes`，配置的额度或那份显存的 3/4）减驱动的固定开销
 /// （主 agent 2026-10-06 定：它是每张卡账的上限，不是每条用例的上限，不判红）。
 #[must_use]
 pub fn card_ledger_capacity_mebibytes(
@@ -67,12 +72,18 @@ pub fn card_ledger_capacity_mebibytes(
     memory_all_registered_processes_may_hold_mebibytes: u64,
     mebibytes_left_for_the_driver: u64,
 ) -> u64 {
+    let default_share_without_a_configured_card_quota_mebibytes =
+        match configured_card_quota_mebibytes {
+            Some(_) => u64::MAX,
+            None => budget_mebibytes.saturating_sub(mebibytes_left_for_the_driver),
+        };
     match card_quota_from_the_environment_mebibytes {
         Some(quota) => quota.min(
             configured_card_quota_mebibytes
                 .map_or(u64::MAX, |configured| {
                     configured.saturating_sub(mebibytes_left_for_the_driver)
                 })
+                .min(default_share_without_a_configured_card_quota_mebibytes)
                 .min(
                     memory_all_registered_processes_may_hold_mebibytes
                         .saturating_sub(mebibytes_left_for_the_driver),
@@ -149,6 +160,157 @@ pub struct NeedLargerThanEveryCard {
     pub largest_capacity_mebibytes: u64,
 }
 
+/// 在 `0..=most_states` 里找最多的状态数，它要的显存（`mebibytes_for_states`，MiB）不超过 `capacity_mebibytes`：
+/// 0 个状态不借、恒放得下；`mebibytes_for_states` 要随状态数单调不减（判器按状态数借的那一族都是）。二分，最多问 65 次。
+#[must_use]
+pub fn most_states_whose_need_fits_the_capacity(
+    most_states: usize,
+    capacity_mebibytes: u64,
+    mebibytes_for_states: &dyn Fn(usize) -> u64,
+) -> usize {
+    if most_states == 0 || mebibytes_for_states(most_states) <= capacity_mebibytes {
+        return most_states;
+    }
+    // 不变量：`fitting` 放得下（0 恒放得下），`not_fitting` 放不下；两者之间每轮砍半
+    let mut fitting = 0usize;
+    let mut not_fitting = most_states;
+    while not_fitting - fitting > 1 {
+        let middle = fitting + (not_fitting - fitting) / 2;
+        if mebibytes_for_states(middle) <= capacity_mebibytes {
+            fitting = middle;
+        } else {
+            not_fitting = middle;
+        }
+    }
+    fitting
+}
+
+/// 崩溃放量产品路径（`singlefs_checker_tier::crash_amplification`）一趟用的卡从哪来；草稿区进不进显存账也照它
+/// （[`scratch_states_to_book_on_each_card`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum GpuCardsSource {
+    /// 这一趟没有一类计算走 GPU，不要卡。
+    NoCardNeeded,
+    /// 这个进程起过 GPU 用例共用的卡池（`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::GpuCardPool`）：照池里的卡用，
+    /// 每张卡这个进程只有一个设备，驱动那一份在跨进程的显存账上登记了；第 ① 段判器开的草稿区按装到这张卡上的状态数借
+    /// （第 10v 批：主 agent 定照 state_count 记）。
+    TheCardPoolOfThisProcess,
+    /// 这个进程没起过卡池：这一趟自己在点名的卡上起设备（`gpu_unit_checks::gpu_cards_by_priority` 或 `usable_gpu_cards`），不进显存账。
+    DevicesStartedForThisRun,
+}
+
+/// 崩溃放量产品路径第 ① 段每张卡的草稿区在显存账上记几个状态。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum ScratchStatesToBook {
+    /// 不进账：这一趟不要卡，或这个进程没起过卡池、自己起的设备（[`GpuCardsSource::DevicesStartedForThisRun`]）。
+    NotBooked,
+    /// 起过卡池：按卡的次序每张卡借几个状态的草稿区（0 是这张卡没装状态、不借）。
+    OnEachCard(Vec<usize>),
+}
+
+/// 产品路径第 ① 段的草稿区记几个状态的账：起过卡池的进程，每张装了状态的卡记它装的状态数，与这条流的状态数取小
+/// （[`states_in_one_dispatch_capped_by_the_state_count`]，与 GPU 对拍用例借卡那一路同一个封顶：判器的草稿区不为这条流没有的状态开）；
+/// 没起过卡池的照旧不进账。调用方照这里的状态数按 `crash_judge_dispatch::mebibytes_a_judge_needs` 算显存去借。
+#[must_use]
+pub fn scratch_states_to_book_on_each_card(
+    cards_taken_from: GpuCardsSource,
+    states_placed_on_each_card: &[u64],
+    state_count: u64,
+) -> ScratchStatesToBook {
+    match cards_taken_from {
+        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
+            ScratchStatesToBook::NotBooked
+        }
+        GpuCardsSource::TheCardPoolOfThisProcess => ScratchStatesToBook::OnEachCard(
+            states_placed_on_each_card
+                .iter()
+                .map(|states_placed| {
+                    if *states_placed == 0 {
+                        return 0;
+                    }
+                    let states_placed_as_usize =
+                        usize::try_from(*states_placed).unwrap_or(usize::MAX);
+                    states_in_one_dispatch_capped_by_the_state_count(
+                        states_placed_as_usize,
+                        states_placed_as_usize,
+                        state_count,
+                    )
+                })
+                .collect(),
+        ),
+    }
+}
+
+/// GPU 用例与产品路径报「本次未跑」那一行的记号（`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::not_run_line` 照它打；
+/// 门禁 `checker-tier-layer0-replay` 的 feature-gated-tests 格按它逐条报本次未跑）。
+pub const GPU_NOT_RUN_MARKER: &str = "NOT_RUN_NO_GPU_ADAPTER";
+
+/// 「本次未跑」那一行：`<记号> test=<用例或产品路径的名字> reason=<为什么>`。
+#[must_use]
+pub fn gpu_not_run_line(test_name: &str, reason: &str) -> String {
+    format!("{GPU_NOT_RUN_MARKER} test={test_name} reason={reason}")
+}
+
+/// 等满了还借不到的那一句（GPU 用例借卡与产品路径借草稿区同一句）。
+#[must_use]
+pub fn reason_for_a_booking_that_waited_past_the_longest_wait(
+    need_mebibytes: u64,
+    waited: Duration,
+    card_capacities_mebibytes: &[u64],
+) -> String {
+    format!(
+        "等了 {} 秒，池里没有一张卡空得出 {need_mebibytes} MiB（每张卡的账 {card_capacities_mebibytes:?}；这台机器上别的 GPU 用例或测试进程一直占着）",
+        waited.as_secs()
+    )
+}
+
+/// 产品路径借草稿区借不到的结局（[`scratch_booking_refusal_outcome`]）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum ScratchBookingRefusalOutcome {
+    /// 放得下而等满了：打这一行，这一趟照产品路径的「本次未跑」交回（判过的块在库里，没判的留给下一趟）。
+    NotRun { not_run_line: String },
+    /// 要的比那张卡的账大、账本读写不了：判红。
+    Red { reason: String },
+}
+
+/// 产品路径借草稿区被拒时怎么报：等满了照 GPU 对拍用例那一路报「本次未跑」（同一个记号、同一句），别的判红。
+#[must_use]
+pub fn scratch_booking_refusal_outcome(
+    run_name: &str,
+    card_position: usize,
+    refusal: &BookingRefusal,
+) -> ScratchBookingRefusalOutcome {
+    match refusal {
+        BookingRefusal::WaitedPastTheLongestWait {
+            need_mebibytes,
+            waited,
+            card_capacities_mebibytes,
+        } => ScratchBookingRefusalOutcome::NotRun {
+            not_run_line: gpu_not_run_line(
+                run_name,
+                &reason_for_a_booking_that_waited_past_the_longest_wait(
+                    *need_mebibytes,
+                    *waited,
+                    card_capacities_mebibytes,
+                ),
+            ),
+        },
+        BookingRefusal::NeedLargerThanEveryCard(refusal) => ScratchBookingRefusalOutcome::Red {
+            reason: format!(
+                "{run_name}：第 {card_position} 张卡的草稿区要借 {} MiB，那张卡的账只有 {} MiB",
+                refusal.need_mebibytes, refusal.largest_capacity_mebibytes
+            ),
+        },
+        BookingRefusal::LedgerFileUnusable(error) => ScratchBookingRefusalOutcome::Red {
+            reason: format!(
+                "{run_name}：显存账本 {} 用不了：{}",
+                error.path.display(),
+                error.reason
+            ),
+        },
+    }
+}
+
 /// 一笔借不到、等满 [`LONGEST_WAIT_FOR_GPU_MEMORY`] 才放弃：一条 GPU 用例在卡上判一条流以分钟计，等半小时还空不出来，
 /// 说明这台机器上别的 GPU 测试进程占得太久，这条用例报「本次未跑」，不干等到整趟被外面的限时掐掉（推的，没量过用例最长占多久）。
 pub const LONGEST_WAIT_FOR_GPU_MEMORY: Duration = Duration::from_secs(30 * 60);
@@ -798,10 +960,16 @@ impl GpuMemoryLedger {
         &self,
         need_mebibytes: u64,
     ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
-        let booked = self.book_waiting(
-            WhichCards::FirstThatFitsByPriority,
-            WhatIsBooked::LeaseOf { need_mebibytes },
-        )?;
+        self.book_a_lease_waiting(WhichCards::FirstThatFitsByPriority, need_mebibytes)
+    }
+
+    /// 照 `which` 挑卡借 `need_mebibytes`（[`Self::book_waiting`]），记上的那一笔做成借据。
+    fn book_a_lease_waiting(
+        &self,
+        which: WhichCards,
+        need_mebibytes: u64,
+    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
+        let booked = self.book_waiting(which, WhatIsBooked::LeaseOf { need_mebibytes })?;
         Ok(GpuMemoryBooking {
             ledger: self,
             card_position: booked.card_position,
@@ -813,6 +981,20 @@ impl GpuMemoryLedger {
         })
     }
 
+    /// 借 `need_mebibytes`，只记在第 `card_position` 张卡上（崩溃放量产品路径按每张卡装的状态借草稿区，卡由派活定、不由账挑）；
+    /// 别的同 [`Self::book_waiting_for_room`]：这个进程在这张卡上还没登记的，驱动那一份一并登记；放不下就等，等满放弃；
+    /// 永远放不下（要的比这张卡的容量大）交错误、什么都不记。
+    ///
+    /// # Errors
+    /// 见 [`BookingRefusal`]。
+    pub fn book_on_the_card_waiting_for_room(
+        &self,
+        card_position: usize,
+        need_mebibytes: u64,
+    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
+        self.book_a_lease_waiting(WhichCards::OnlyTheCardAt { card_position }, need_mebibytes)
+    }
+
     /// 把这个进程登记到第 `card_position` 张卡上（记它的驱动那一份），不借：已经登记了的当场交回；没登记的要账上放得下这一份才登记，
     /// 放不下就等、等满放弃（同 [`Self::book_waiting`]）。调用方拿到 `Ok` 之后才在这张卡上起设备。
     ///
@@ -909,11 +1091,13 @@ mod tests {
     use super::{
         card_ledger_capacity_mebibytes, decide_booking_by_priority, gpu_memory_ledger_directory,
         memory_all_registered_processes_may_hold_mebibytes,
-        process_is_alive_by_its_entry_in_the_process_filesystem,
+        most_states_whose_need_fits_the_capacity,
+        process_is_alive_by_its_entry_in_the_process_filesystem, scratch_booking_refusal_outcome,
+        scratch_states_to_book_on_each_card,
         start_time_in_clock_ticks_from_the_process_status_line,
         states_in_one_dispatch_capped_by_the_state_count, BookingDecision, BookingRefusal,
-        GpuMemoryLedger, LedgerSettings, NeedLargerThanEveryCard, RecordedProcess,
-        GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME,
+        GpuCardsSource, GpuMemoryLedger, LedgerSettings, NeedLargerThanEveryCard, RecordedProcess,
+        ScratchBookingRefusalOutcome, ScratchStatesToBook, GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME,
     };
     use std::ffi::OsStr;
     use std::path::{Path, PathBuf};
@@ -1429,8 +1613,8 @@ mod tests {
         );
         assert_eq!(
             card_ledger_capacity_mebibytes(Some(40_000), None, 22_500, 30_000, DRIVER),
-            30_000 - DRIVER,
-            "额度比空闲显存减驱动那一份大：取空闲显存那一边"
+            22_500 - DRIVER,
+            "额度比 3/4 那一份减驱动那一份大：没配显卡优先级时 3/4 那一份代替配置的额度，取它（第 10v 批）"
         );
         assert_eq!(
             card_ledger_capacity_mebibytes(None, None, 22_500, 30_000, DRIVER),
@@ -1439,6 +1623,177 @@ mod tests {
         );
     }
 
+    /// 第 10v 批：没配显卡优先级、设了额度变量 20 000、清场后空 30 000 MiB：没配额度时默认的 3/4 那一份（22 500）代替配置的额度，
+    /// 并进最小值，容量不超过它减驱动那一份。
+    #[test]
+    fn an_environment_quota_without_a_card_priority_configured_stays_within_the_default_three_quarter_share(
+    ) {
+        const FREE_MEMORY_AFTER_QUIESCING: u64 = 30_000;
+        let three_quarter_share = FREE_MEMORY_AFTER_QUIESCING / 4 * 3;
+        let capacity = card_ledger_capacity_mebibytes(
+            Some(20_000),
+            None,
+            three_quarter_share,
+            FREE_MEMORY_AFTER_QUIESCING,
+            DRIVER,
+        );
+        assert!(
+            capacity <= three_quarter_share - DRIVER,
+            "额度变量 20 000 不越过 3/4 那一份 {three_quarter_share} 减驱动那一份：实际 {capacity}"
+        );
+        assert_eq!(
+            capacity,
+            three_quarter_share - DRIVER,
+            "三者里最小的是 3/4 那一份减驱动那一份"
+        );
+    }
+
+    /// 第 10v 批：起过卡池的进程，产品路径第 ① 段每张装了状态的卡按装的状态数借草稿区（45 个状态的小流只借 45 个，
+    /// 一张 5090 装满 17 867 个、余下 3 000 个给下一张的大流各借各的），没装状态的卡不借；没起过卡池、不要卡的照旧不进账。
+    #[test]
+    fn a_process_that_started_the_card_pool_books_the_scratch_of_each_card_for_the_states_placed_on_it(
+    ) {
+        assert_eq!(
+            scratch_states_to_book_on_each_card(
+                GpuCardsSource::TheCardPoolOfThisProcess,
+                &[45, 0],
+                45
+            ),
+            ScratchStatesToBook::OnEachCard(vec![45, 0]),
+            "45 个状态的流只在第一张卡上借 45 个状态的草稿区，第二张不借"
+        );
+        assert_eq!(
+            scratch_states_to_book_on_each_card(
+                GpuCardsSource::TheCardPoolOfThisProcess,
+                &[17_867, 3_000],
+                131_195
+            ),
+            ScratchStatesToBook::OnEachCard(vec![17_867, 3_000]),
+            "大流按每张卡装的状态数借"
+        );
+        assert_eq!(
+            scratch_states_to_book_on_each_card(
+                GpuCardsSource::DevicesStartedForThisRun,
+                &[45, 0],
+                45
+            ),
+            ScratchStatesToBook::NotBooked,
+            "没起过卡池的进程自己起设备，照旧不进账"
+        );
+        assert_eq!(
+            scratch_states_to_book_on_each_card(GpuCardsSource::NoCardNeeded, &[], 45),
+            ScratchStatesToBook::NotBooked,
+            "不要卡的一趟不借"
+        );
+    }
+
+    /// 第 10v 批：卡被别的进程订满，产品路径在这张卡上借草稿区等满了：照 GPU 对拍用例那一路报「本次未跑」，同一个记号、同一句。
+    #[test]
+    fn a_scratch_booking_that_waits_past_the_longest_wait_is_reported_not_run_with_the_gpu_not_run_marker(
+    ) {
+        let directory = LedgerDirectory::for_the_test("scratch-waits-past-the-longest-wait");
+        let first = ledger_of_a_fake_process(
+            &directory.path,
+            1,
+            &[1, 2],
+            10_000,
+            0,
+            Duration::from_millis(20),
+            Duration::from_secs(5),
+        );
+        let held = first
+            .book_on_the_card_waiting_for_room(0, 10_000)
+            .expect("空着的卡上借得到整份");
+        let second = ledger_of_a_fake_process(
+            &directory.path,
+            2,
+            &[1, 2],
+            10_000,
+            0,
+            Duration::from_millis(20),
+            Duration::from_millis(100),
+        );
+        let refusal = second
+            .book_on_the_card_waiting_for_room(0, 4_000)
+            .map(|booking| booking.mebibytes())
+            .expect_err("卡被第一个进程订满，等满 100 毫秒借不到");
+        let outcome = scratch_booking_refusal_outcome("crash_amplification::run", 0, &refusal);
+        drop(held);
+        let ScratchBookingRefusalOutcome::NotRun { not_run_line } = &outcome else {
+            panic!("等满了借不到要报本次未跑，判成了 {outcome:?}");
+        };
+        assert!(
+            not_run_line
+                .starts_with("NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run reason=等了 "),
+            "本次未跑那一行用 GPU 用例的记号（门禁 feature-gated-tests 格认的就是这一串）与同一句：{not_run_line}"
+        );
+    }
+
+    /// 第 10v 批：点名一张卡借，只记在那张卡上：第一张卡空着也不挪过去；要的比点名的那张大、比第一张小的，判成放不下（判红），不记到第一张上。
+    #[test]
+    fn a_booking_on_a_named_card_stays_on_that_card_even_when_an_earlier_card_has_room() {
+        let directory = LedgerDirectory::for_the_test("booking-on-a-named-card");
+        let ledger = GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
+            settings_of_a_fake_process(
+                &directory.path,
+                1,
+                &[1],
+                0,
+                Duration::from_millis(20),
+                Duration::from_millis(100),
+            ),
+            vec!["card-first".to_string(), "card-second".to_string()],
+            &|_position| Ok(1000),
+            Box::new(|position, _estimate| match position {
+                0 => 29_000,
+                _ => 12_000,
+            }),
+        )
+        .expect("临时目录下的账本读写得了");
+        let booking = ledger
+            .book_on_the_card_waiting_for_room(1, 4_000)
+            .expect("第二张卡空着，借得到");
+        assert_eq!(booking.card_position(), 1, "点名第二张卡就记在第二张卡上");
+        drop(booking);
+        let refusal = ledger
+            .book_on_the_card_waiting_for_room(1, 20_000)
+            .map(|too_large_booking| too_large_booking.card_position())
+            .expect_err("要的比第二张卡的容量大");
+        assert!(
+            matches!(
+                scratch_booking_refusal_outcome("crash_amplification::run", 1, &refusal),
+                ScratchBookingRefusalOutcome::Red { .. }
+            ),
+            "放不下点名的那张卡要判红，不挪到第一张卡上：{refusal:?}"
+        );
+    }
+
+    /// 第 10v 批：按判器那一族的算法（表 1 MiB，每个状态 1 261 616 字节）在 56 MiB 里装得下 45 个状态（45 个要 56、46 个要 57）；
+    /// 上限内全放得下就取上限，一个都放不下取 0。
+    #[test]
+    fn the_most_states_whose_need_fits_is_the_largest_count_whose_need_is_within_the_capacity() {
+        let judge_needs = |states: usize| -> u64 {
+            1 + (u64::try_from(states).expect("装得进 u64") * 1_261_616).div_ceil(1024 * 1024)
+        };
+        assert_eq!(judge_needs(45), 56);
+        assert_eq!(judge_needs(46), 57);
+        assert_eq!(
+            most_states_whose_need_fits_the_capacity(100_000, 56, &judge_needs),
+            45,
+            "56 MiB 装 45 个状态"
+        );
+        assert_eq!(
+            most_states_whose_need_fits_the_capacity(30, 56, &judge_needs),
+            30,
+            "上限内全放得下取上限"
+        );
+        assert_eq!(
+            most_states_whose_need_fits_the_capacity(100_000, 0, &judge_needs),
+            0,
+            "一个都放不下取 0"
+        );
+    }
+
     /// 两张卡（容量照 5090 与 5060 Ti 的量级）：前一张放得下就一直记在前一张上，前一张放不下这一笔才记到下一张。
     #[test]
     fn bookings_fill_the_first_card_before_the_next_card_takes_any() {
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index 01b6d08..44f0863 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -764,10 +764,11 @@ pub mod gpu_test_start {
         }
     }
 
-    /// 这条用例报「本次未跑」时打的那一行。
+    /// 这条用例报「本次未跑」时打的那一行（记号与写法在 harness 档一处定义，崩溃放量产品路径借草稿区等满了也打它：
+    /// `singlefs_harness::gpu_memory_ledger::gpu_not_run_line`）。
     #[must_use]
     pub fn not_run_line(test_name: &str, reason: &str) -> String {
-        format!("NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}")
+        singlefs_harness::gpu_memory_ledger::gpu_not_run_line(test_name, reason)
     }
 
     fn adapter_device_kind(device_type: wgpu::DeviceType) -> AdapterDeviceKind {
@@ -939,7 +940,8 @@ pub mod gpu_test_start {
         };
         use singlefs_harness::gpu_memory_ledger::{
             card_ledger_capacity_mebibytes, gpu_memory_ledger_directory_of_this_process,
-            process_is_alive_by_its_entry_in_the_process_filesystem, BookingRefusal,
+            process_is_alive_by_its_entry_in_the_process_filesystem,
+            reason_for_a_booking_that_waited_past_the_longest_wait, BookingRefusal,
             GpuMemoryBooking, GpuMemoryLedger, LedgerOpeningRefusal, LedgerSettings,
             RecordedProcess, LONGEST_WAIT_FOR_GPU_MEMORY,
             POLL_INTERVAL_FOR_MEMORY_RETURNED_BY_OTHER_PROCESSES,
@@ -1117,9 +1119,10 @@ pub mod gpu_test_start {
                         waited,
                         card_capacities_mebibytes,
                     }) => {
-                        return Err(format!(
-                            "等了 {} 秒，池里没有一张卡空得出 {waited_for_mebibytes} MiB（每张卡的账 {card_capacities_mebibytes:?}；这台机器上别的 GPU 用例或测试进程一直占着）",
-                            waited.as_secs()
+                        return Err(reason_for_a_booking_that_waited_past_the_longest_wait(
+                            waited_for_mebibytes,
+                            waited,
+                            &card_capacities_mebibytes,
                         ))
                     }
                     Err(BookingRefusal::LedgerFileUnusable(error)) => panic!(
@@ -1128,9 +1131,42 @@ pub mod gpu_test_start {
                         error.reason
                     ),
                 };
-                let position = booking.card_position();
+                Ok(self.card_lent_for_the_booking(test_name, booking))
+            }
+
+            /// 借只记在第 `card_position` 张卡上（`singlefs_harness::gpu_memory_ledger::GpuMemoryLedger::book_on_the_card_waiting_for_room`）：
+            /// 崩溃放量产品路径在起过卡池的进程里按派活装到每张卡上的状态借草稿区，卡由派活定。借不到交账的结局，由调用方定怎么报
+            /// （`singlefs_harness::gpu_memory_ledger::scratch_booking_refusal_outcome`：等满了报本次未跑，别的判红）。
+            ///
+            /// # Errors
+            /// 见 `singlefs_harness::gpu_memory_ledger::BookingRefusal`。
+            ///
+            /// # Panics
+            /// 借到的卡上起不了设备。
+            pub fn lease_the_card_at_waiting_for_room(
+                &'static self,
+                run_name: &str,
+                card_position: usize,
+                need_mebibytes: u64,
+            ) -> Result<GpuCardLease, BookingRefusal> {
+                let booking = self
+                    .ledger
+                    .book_on_the_card_waiting_for_room(card_position, need_mebibytes)?;
+                Ok(self.card_lent_for_the_booking(run_name, booking))
+            }
+
+            /// 记上了的那一笔交成借来的卡：与池里那张共用设备，额度与空闲改成借的量加驱动那一份；打一行 `GPU_CARD_LEASE`。
+            ///
+            /// # Panics
+            /// 那张卡上起不了设备。
+            fn card_lent_for_the_booking(
+                &'static self,
+                test_name: &str,
+                booking: GpuMemoryBooking<'static>,
+            ) -> GpuCardLease {
+                let need_mebibytes = booking.mebibytes();
                 let mut card = self
-                    .started_card(position)
+                    .started_card(booking.card_position())
                     .unwrap_or_else(|reason| panic!("{test_name}：{reason}"))
                     .clone();
                 card.memory_quota_mebibytes = Some(need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER);
@@ -1141,12 +1177,23 @@ pub mod gpu_test_start {
                     card.index,
                     booking.card_capacity_mebibytes_when_booked()
                 );
-                Ok(GpuCardLease { card, booking })
+                GpuCardLease { card, booking }
+            }
+
+            /// 这个进程此刻看池里每张卡的账能借多少（MiB，驱动那一份已经扣掉；按池的次序）。
+            ///
+            /// # Errors
+            /// 账本读写不了。
+            pub fn card_capacities_mebibytes(&self) -> Result<Vec<u64>, String> {
+                self.ledger.card_capacities_mebibytes().map_err(|error| {
+                    format!("显存账本 {} 用不了：{}", error.path.display(), error.reason)
+                })
             }
 
             /// 池里每张卡（按优先级）：这个进程在每张卡的账上登记（只记驱动那一份，不借；放不下就等，等满放弃），再取起好的设备。
             /// 给崩溃放量用（`crate::crash_amplification`）：这个进程起过卡池时，放量照池里的卡判，不另起一套设备。
-            /// 交回的卡是池里那张的克隆（额度是配置给这张卡的、空闲是开账时读的）；放量开的草稿区不进账：该记多少、等不到时放量怎么退，条款没写，第 10t 批交主 agent 定。
+            /// 交回的卡是池里那张的克隆（额度是配置给这张卡的、空闲是开账时读的）；放量第 ① 段开的草稿区另按每张卡装的状态数借
+            /// （[`Self::lease_the_card_at_waiting_for_room`]，第 10v 批：主 agent 定照 state_count 记、等不到报本次未跑）。
             ///
             /// # Errors
             /// 某张卡的账上放不下这个进程的驱动那一份（等满了、或永远放不下），账本读写不了，或卡上起不了设备。
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index 42c4e9c..9d37e97 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -14,6 +14,18 @@
 //!    领块只在内存里，库里不记谁领了，不会有块被一张死卡占着。
 //! 4. **分工与跑法**：块按权重分给几方（一方一台机器的 CPU，或一张卡，权重取空闲显存），各写各的库，最后
 //!    [`crate::verdict_store::VerdictStore::import_from`] 并成一个；跑法由配置的两个开关定：`ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`。
+//! 5. **显存账**（`singlefs_harness::gpu_memory_ledger`，只防这台机器的显存 OOM、不压并发：用户 2026-10-06 定）：这个进程起过 GPU 用例共用的卡池
+//!    （[`GpuCardsSource::TheCardPoolOfThisProcess`]）时，取卡照池里的卡、驱动那一份在账上登记；第 ① 段判器的草稿区按派活装到每张卡上的状态数
+//!    借（照 state_count 封顶，`singlefs_harness::gpu_memory_ledger::scratch_states_to_book_on_each_card`，显存按与 GPU 对拍用例同一份的
+//!    `crate::crash_judge_dispatch::mebibytes_a_judge_needs` 算，只记在那张卡上），每张卡装几个状态先照账能借的量压（借得到才装）；
+//!    等满了借不到，照对拍用例那一路打「本次未跑」那一行（同一个记号），这一趟照叫停的样子交回（`stopped_early`，判过的块在库里，
+//!    分项驱动把这一项记成本次未跑；派出的项全没判完时整趟退 77）。只排派活计划那一趟试判前同样借。
+//!    没起过卡池的进程照旧不进账（[`GpuCardsSource::DevicesStartedForThisRun`]：这一趟自己在点名的卡上起设备），调用方是这几种：
+//!    崩溃放量分项驱动（`scripts/crash/crash-amplification-item.sh`）起的、只有经 [`run_from_environment`] 跑的用例而没有一条取卡池的测试二进制
+//!    （`tests/crash_amplification_fixed_script_through_rollback.rs`、`…_through_unmount.rs`、`crash_amplification_two_flows_share_their_prefix.rs`、
+//!    经 `tests/common_crash_points` 跑的 `crash_enumeration_*` 那几份）；同一个测试二进制里有取卡池的 GPU 用例、而经 [`run_from_environment`]
+//!    的用例跑在它们起池之前的（`crash_amplification_pipeline_records_checks_and_reuses.rs`、`record_checker_judges_absence_by_the_persisted_set.rs`）。
+//!    直接调 `judge_my_blocks_on_gpu_cards` 的对拍用例不走这一条，卡由用例自己给、不进账。第 ② 段核对与单元校验和对拍在第一张卡上建的缓冲不进账。
 
 use std::collections::{BTreeMap, BTreeSet};
 use std::ops::Range;
@@ -24,6 +36,11 @@ use std::time::Instant;
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
 use singlefs_core::recovery::{recover, JournalPolicy};
 use singlefs_core::root_record::ROOT_MAGIC;
+#[cfg(feature = "gpu")]
+use singlefs_harness::gpu_memory_ledger::{
+    most_states_whose_need_fits_the_capacity, scratch_booking_refusal_outcome,
+    scratch_states_to_book_on_each_card, ScratchBookingRefusalOutcome, ScratchStatesToBook,
+};
 use singlefs_harness::hexadecimal::hexadecimal_text;
 use singlefs_harness::memory_pool::{
     root_identity_of_write, root_identity_written_by, CrashImage, MemoryPool, PublishedVersion,
@@ -1987,6 +2004,192 @@ pub fn judge_my_blocks_on_gpu_cards(
     cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
     stop_is_requested: &(dyn Fn() -> bool + Sync),
 ) -> Result<(CheckingTally, GpuJudgeOutcome), String> {
+    match judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(
+        store,
+        flow,
+        plan,
+        is_mine,
+        version,
+        cards,
+        GpuCardsSource::DevicesStartedForThisRun,
+        stop_is_requested,
+    )? {
+        GpuJudgingOfMyBlocks::Judged { tally, outcome } => Ok((tally, outcome)),
+        GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime => unreachable!(
+            "卡判成调用方自己起的（DevicesStartedForThisRun）就不借草稿区（scratch_states_to_book_on_each_card 交 NotBooked），等不到账这一形走不到"
+        ),
+    }
+}
+
+/// 第 ① 段在卡上判的结局。
+#[cfg_attr(
+    not(feature = "gpu"),
+    allow(
+        dead_code,
+        reason = "没编进 gpu 特性时第 ① 段在卡上的那一份只报错，两种结局只在编进来时造"
+    )
+)]
+enum GpuJudgingOfMyBlocks {
+    Judged {
+        tally: CheckingTally,
+        outcome: GpuJudgeOutcome,
+    },
+    /// 起过卡池的进程在账上借草稿区等满了：「本次未跑」那一行已经打了，一块都没判（调用方照叫停的样子交回）。
+    NotRunTheScratchWasNotBookedInTime,
+}
+
+/// 「本次未跑」那一行里产品路径的名字（GPU 用例写用例名的那一格）。
+#[cfg(feature = "gpu")]
+const PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE: &str = "crash_amplification::run_from_environment";
+
+/// 这一趟的卡判成照这个进程的卡池用（[`gpu_cards_in_use`] 交 [`GpuCardsSource::TheCardPoolOfThisProcess`]）时的那个池。
+///
+/// # Panics
+/// 这个进程没起过卡池（调用方把卡的来路判错了）。
+#[cfg(feature = "gpu")]
+fn the_card_pool_this_run_takes_its_cards_from(
+) -> &'static crate::gpu_unit_checks::gpu_test_start::GpuCardPool {
+    crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process().expect(
+        "这一趟的卡判成照这个进程的卡池用时池已经起好（gpu_cards_in_use 看过），起好的池在进程里不会再变回没起",
+    )
+}
+
+/// 起过卡池的这一趟每张卡装得下几个状态：照卡的额度算的（[`states_in_flight_of_each_card`]）再按这张卡的账此刻能借给这个进程的量压
+/// （判器按装的状态数借 `crate::crash_judge_dispatch::mebibytes_a_judge_needs`，借的量不越过账的容量，装上的状态才借得到）。
+///
+/// # Errors
+/// 账本读写不了。
+///
+/// # Panics
+/// 这一趟的卡与池里的卡张数不同（[`cards_from_the_card_pool_of_this_process`] 按池的次序交回，逐张对上）。
+#[cfg(feature = "gpu")]
+fn states_each_card_of_the_pool_lends(
+    pool: &crate::gpu_unit_checks::gpu_test_start::GpuCardPool,
+    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
+    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
+) -> Result<Vec<u64>, String> {
+    let ledger_capacities = pool.card_capacities_mebibytes()?;
+    assert_eq!(
+        ledger_capacities.len(),
+        cards.len(),
+        "这一趟照池里的卡用：卡与池里的卡逐张对上"
+    );
+    Ok(states_in_flight_of_each_card(cards, tables)
+        .into_iter()
+        .zip(ledger_capacities)
+        .map(|(states_by_the_card_budget, ledger_capacity_mebibytes)| {
+            let most_states = usize::try_from(states_by_the_card_budget).unwrap_or(usize::MAX);
+            u64::try_from(most_states_whose_need_fits_the_capacity(
+                most_states,
+                ledger_capacity_mebibytes,
+                &|states| crate::crash_judge_dispatch::mebibytes_a_judge_needs(tables, states),
+            ))
+            .expect("装得进 u64")
+        })
+        .collect())
+}
+
+/// 第 ① 段每张卡上判器用的卡：借了草稿区的那几张换成借来的卡（额度是借的量加驱动那一份，判器照它开出来的草稿区正好是借的状态数）。
+#[cfg(feature = "gpu")]
+enum ScratchOfEachCard {
+    /// `cards` 按这一趟卡的次序；`leases` 是借据（不进账时是空的），判器都丢掉之后才还。
+    Ready {
+        cards: Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>>,
+        leases: Vec<crate::gpu_unit_checks::gpu_test_start::GpuCardLease>,
+    },
+    /// 等满了借不到：「本次未跑」那一行已经打了，已经借到的几张都还了。
+    NotRunTheScratchWasNotBookedInTime,
+}
+
+/// 照卡的来路给第 ① 段借草稿区（[`scratch_states_to_book_on_each_card`]）：起过卡池的，每张装了状态的卡按它装的状态数在那张卡上借
+/// （显存按 `crate::crash_judge_dispatch::mebibytes_a_judge_needs`，与 GPU 对拍用例借卡同一份），按卡的次序一张一张借，放不下就等；
+/// 等满了照对拍用例那一路打「本次未跑」那一行（[`scratch_booking_refusal_outcome`]）。没起过卡池的不借、卡照原样。
+///
+/// # Errors
+/// 要借的比那张卡的账大、账本读写不了（判红）。
+#[cfg(feature = "gpu")]
+fn scratch_of_each_card_booked_by_where_the_cards_came_from(
+    cards_taken_from: GpuCardsSource,
+    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
+    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
+    state_count: u64,
+    states_placed: &[u64],
+) -> Result<ScratchOfEachCard, String> {
+    match scratch_states_to_book_on_each_card(cards_taken_from, states_placed, state_count) {
+        ScratchStatesToBook::NotBooked => Ok(ScratchOfEachCard::Ready {
+            cards: cards.to_vec(),
+            leases: Vec::new(),
+        }),
+        ScratchStatesToBook::OnEachCard(states_to_book) => {
+            let pool = the_card_pool_this_run_takes_its_cards_from();
+            let mut cards_for_the_judges = cards.to_vec();
+            let mut leases = Vec::new();
+            for (card_position, states) in states_to_book.iter().enumerate() {
+                if *states == 0 {
+                    continue;
+                }
+                match pool.lease_the_card_at_waiting_for_room(
+                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
+                    card_position,
+                    crate::crash_judge_dispatch::mebibytes_a_judge_needs(tables, *states),
+                ) {
+                    Ok(lease) => {
+                        cards_for_the_judges[card_position] =
+                            std::sync::Arc::new(lease.card().clone());
+                        leases.push(lease);
+                    }
+                    Err(refusal) => {
+                        return match scratch_booking_refusal_outcome(
+                            PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
+                            card_position,
+                            &refusal,
+                        ) {
+                            ScratchBookingRefusalOutcome::NotRun { not_run_line } => {
+                                println!("{not_run_line}");
+                                Ok(ScratchOfEachCard::NotRunTheScratchWasNotBookedInTime)
+                            }
+                            ScratchBookingRefusalOutcome::Red { reason } => Err(reason),
+                        };
+                    }
+                }
+            }
+            Ok(ScratchOfEachCard::Ready {
+                cards: cards_for_the_judges,
+                leases,
+            })
+        }
+    }
+}
+
+/// 同 [`judge_my_blocks_on_gpu_cards`]，卡的来路另给：起过卡池的（[`GpuCardsSource::TheCardPoolOfThisProcess`]），每张卡装几个状态
+/// 先照账能借的量压（[`states_each_card_of_the_pool_lends`]），判之前按装的状态数在那张卡上借草稿区
+/// （[`scratch_of_each_card_booked_by_where_the_cards_came_from`]），等满了借不到交 [`GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime`]、
+/// 一块都不判；判完先丢判器、再还借据。
+///
+/// # Errors
+/// 同 [`judge_my_blocks_on_gpu_cards`]；借草稿区时要的比那张卡的账大、账本读写不了。
+///
+/// # Panics
+/// 同 [`judge_my_blocks_on_gpu_cards`]。
+#[cfg(feature = "gpu")]
+#[allow(
+    clippy::too_many_arguments,
+    reason = "库、流、计划、分工、判法版本、卡、卡的来路、叫停，各是各的东西，凑成结构体只为让签名变短"
+)]
+#[allow(
+    clippy::too_many_lines,
+    reason = "备表、装卡、借草稿区、几张卡一起判、按块序写库、收尾，是第 ① 段的全部步骤，拆开要把十几个量来回传"
+)]
+fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(
+    store: &mut VerdictStore,
+    flow: &CrashFlow<'_>,
+    plan: &CrashPlan,
+    is_mine: &dyn Fn(u64) -> bool,
+    version: &JudgeVersion,
+    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
+    cards_taken_from: GpuCardsSource,
+    stop_is_requested: &(dyn Fn() -> bool + Sync),
+) -> Result<GpuJudgingOfMyBlocks, String> {
     use crate::crash_judge_tables::{build_judge_tables, JudgeFlow};
     let pending = pending_blocks(store, plan, version, is_mine)
         .map_err(|error| format!("读判定块失败：{error:?}"))?;
@@ -2012,14 +2215,24 @@ pub fn judge_my_blocks_on_gpu_cards(
     let mut tally = CheckingTally::default();
     let mut outcome = GpuJudgeOutcome::default();
     if pending.is_empty() {
-        return Ok((tally, outcome));
+        return Ok(GpuJudgingOfMyBlocks::Judged { tally, outcome });
     }
+    let state_count = tables.state_count;
     let tables = crate::crash_judge_dispatch::PackedJudgeTables::of(&tables);
     let pending_states: u64 = pending
         .iter()
         .map(|(_, local)| local.end - local.start)
         .sum();
-    let capacities = states_in_flight_of_each_card(cards, &tables);
+    let capacities = match cards_taken_from {
+        GpuCardsSource::TheCardPoolOfThisProcess => states_each_card_of_the_pool_lends(
+            the_card_pool_this_run_takes_its_cards_from(),
+            cards,
+            &tables,
+        )?,
+        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
+            states_in_flight_of_each_card(cards, &tables)
+        }
+    };
     let states_placed = place_states_on_cards_by_priority(pending_states, &capacities);
     for ((card, capacity), placed) in cards.iter().zip(&capacities).zip(&states_placed) {
         eprintln!(
@@ -2036,11 +2249,25 @@ pub fn judge_my_blocks_on_gpu_cards(
             tables.mebibytes()
         ));
     };
+    // 借据活到这个函数收尾：几张卡上的判器在下面的线程里判完就丢，丢完才还
+    let (cards_for_the_judges, _scratch_leases) =
+        match scratch_of_each_card_booked_by_where_the_cards_came_from(
+            cards_taken_from,
+            cards,
+            &tables,
+            state_count,
+            &states_placed,
+        )? {
+            ScratchOfEachCard::Ready { cards, leases } => (cards, leases),
+            ScratchOfEachCard::NotRunTheScratchWasNotBookedInTime => {
+                return Ok(GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime);
+            }
+        };
     let work = CardsAtWork {
         plan,
         pending: &pending,
         tables: &tables,
-        cards,
+        cards: &cards_for_the_judges,
         states_placed: &states_placed,
         next_pending: std::sync::atomic::AtomicUsize::new(0),
         stop: std::sync::atomic::AtomicBool::new(false),
@@ -2118,7 +2345,7 @@ pub fn judge_my_blocks_on_gpu_cards(
     outcome.compile_seconds = outcome.cards.iter().map(|card| card.build_seconds).sum();
     outcome.judge_seconds = (started.elapsed().as_secs_f64() - outcome.compile_seconds).max(0.0);
     outcome.red_states = tally.red_states;
-    Ok((tally, outcome))
+    Ok(GpuJudgingOfMyBlocks::Judged { tally, outcome })
 }
 
 /// GPU 判完这一趟打的那一行：判了多少、多快，连这一趟的展开上限与全域与否（[`run_from_environment`] 打，展开上限只有它知道）。
@@ -3500,7 +3727,9 @@ pub struct EnvironmentRun {
     pub gpu_judge: Option<GpuJudgeOutcome>,
     /// 这一趟只排了派活计划（`SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1`）：录入与导入照做，一块都没判。
     pub placement_only: bool,
-    /// 这一趟被叫停了（[`stop_is_requested_by_the_environment`]）：判过的块在库里，第 ②③ 段没做，对账不为空是正常的。
+    /// 这一趟被叫停了（[`stop_is_requested_by_the_environment`]），或起过卡池的进程在显存账上借草稿区等满了、打了「本次未跑」那一行
+    /// （文件头第 5 条）：判过的块在库里，第 ②③ 段没做，对账不为空是正常的；分项驱动照它把这一项记成本次未跑。
+    /// 只排派活计划的那一趟借试判的草稿区等满了也置上（`placement_only` 同时为真，那一趟没打 `CRASH_AMPLIFICATION_PLACEMENT` 行）。
     pub stopped_early: bool,
     /// 这一趟判定行上的判法版本。
     pub judge_version: JudgeVersion,
@@ -3519,17 +3748,9 @@ pub struct EnvironmentRun {
     pub gpu_cards_taken_from: GpuCardsSource,
 }
 
-/// 一趟用的卡从哪来。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub enum GpuCardsSource {
-    /// 这一趟没有一类计算走 GPU，不要卡。
-    NoCardNeeded,
-    /// 这个进程起过 GPU 用例共用的卡池（`gpu_unit_checks::gpu_test_start::GpuCardPool`）：照池里的卡用，每张卡这个进程只有一个设备，
-    /// 驱动那一份在跨进程的显存账上登记了；这一趟开的草稿区不进账（第 10t 批交主 agent 定该记多少）。
-    TheCardPoolOfThisProcess,
-    /// 这个进程没起过卡池：这一趟自己在点名的卡上起设备（`gpu_unit_checks::gpu_cards_by_priority` 或 `usable_gpu_cards`），不进显存账。
-    DevicesStartedForThisRun,
-}
+/// 一趟用的卡从哪来（定义在 harness 档，草稿区记不记账的判法 `singlefs_harness::gpu_memory_ledger::scratch_states_to_book_on_each_card`
+/// 照它分，单测在 harness 档随时跑得到）。
+pub use singlefs_harness::gpu_memory_ledger::GpuCardsSource;
 
 /// 每一行 `CRASH_AMPLIFICATION ` 末尾都接的几个字段（汇总行、被叫停的行、`flow=` 行、两台跑的分份行与导入行，打法都在
 /// `crates/singlefs-checker-tier/tests/` 里调 [`run_from_environment`] 的用例）：
@@ -3949,13 +4170,14 @@ pub fn run_from_environment(
                 GpuComputationClass::CrashJudgeGpu.name()
             ));
         };
-        print_the_placement(
+        let placement = print_the_placement(
             &store,
             flow,
             &plan,
             &|block| selected.contains(&block),
             &version,
             cards,
+            gpu_cards_taken_from,
             cards_by_priority.as_deref(),
             host,
             states_reused_at_recording,
@@ -3982,7 +4204,8 @@ pub fn run_from_environment(
             comparison: None,
             gpu_judge: None,
             placement_only: true,
-            stopped_early: false,
+            // 起过卡池的进程借试判的草稿区等满了：「本次未跑」那一行已经打了，没打 PLACEMENT 行（第 10v 批）
+            stopped_early: placement == PlacementOfThisRun::NotRunTheScratchWasNotBookedInTime,
             judge_version: version,
             computation_routes: routes,
             expand_up_to,
@@ -3997,20 +4220,26 @@ pub fn run_from_environment(
     // 第 ① 段：登记了 crash_judge_gpu 就由 GPU 判器判这台机器领的块（判不了就报错，不退回 CPU）；没登记由 CPU 判器判，
     // 那时登记了 gpu_unit_checks 就把单元两道校验和另交一张卡对拍
     let judging_started = std::time::Instant::now();
-    let (checking, gpu_judge) = match (routes.crash_judge, &cards) {
-        (ComputedOn::Gpu, Some(cards)) => {
-            let (checking, outcome) = judge_my_blocks_on_gpu(
-                &mut store,
-                flow,
-                &plan,
-                &|block| selected.contains(&block),
-                &version,
-                cards,
-                &stop_is_requested_by_the_environment,
-            )?;
-            eprintln!("{}", gpu_total_line(&outcome, expand_up_to, exhaustive));
-            (checking, Some(outcome))
-        }
+    let (checking, gpu_judge, scratch_was_not_booked_in_time) = match (routes.crash_judge, &cards) {
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
+                (tally, Some(outcome), false)
+            }
+            // 起过卡池的进程借草稿区等满了：「本次未跑」那一行已经打了，下面照叫停的样子交回（第 10v 批）
+            GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime => {
+                (CheckingTally::default(), None, true)
+            }
+        },
         (ComputedOn::Gpu, None) => {
             return Err(format!(
                 "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回 CPU 判器",
@@ -4029,10 +4258,11 @@ pub fn run_from_environment(
             )
             .map_err(|error| format!("核对失败：{error:?}"))?,
             None,
+            false,
         ),
     };
-    if stop_is_requested_by_the_environment() {
-        // 被叫停：判过的块已经在库里，第 ②③ 段留给接着判的那一趟
+    if scratch_was_not_booked_in_time || stop_is_requested_by_the_environment() {
+        // 被叫停，或起过卡池的进程借草稿区等满了（本次未跑）：判过的块已经在库里，第 ②③ 段留给接着判的那一趟
         let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &version)
             .map_err(|error| format!("对账失败：{error:?}"))?
             .len();
@@ -4367,8 +4597,13 @@ fn gpu_cards_in_use(
     Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
 }
 
-/// 第 ① 段在卡上的入口（`gpu` 特性编进来时调 [`judge_my_blocks_on_gpu_cards`]）。
+/// 第 ① 段在卡上的入口（`gpu` 特性编进来时调 [`judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from`]：
+/// 起过卡池的这一趟草稿区进账，等满了借不到交 [`GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime`]）。
 #[cfg(feature = "gpu")]
+#[allow(
+    clippy::too_many_arguments,
+    reason = "库、流、计划、分工、判法版本、卡、卡的来路、叫停，各是各的东西，凑成结构体只为让签名变短"
+)]
 fn judge_my_blocks_on_gpu(
     store: &mut VerdictStore,
     flow: &CrashFlow<'_>,
@@ -4376,21 +4611,27 @@ fn judge_my_blocks_on_gpu(
     is_mine: &dyn Fn(u64) -> bool,
     version: &JudgeVersion,
     cards: &CardsInUse,
+    cards_taken_from: GpuCardsSource,
     stop_is_requested: &(dyn Fn() -> bool + Sync),
-) -> Result<(CheckingTally, GpuJudgeOutcome), String> {
-    judge_my_blocks_on_gpu_cards(
+) -> Result<GpuJudgingOfMyBlocks, String> {
+    judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(
         store,
         flow,
         plan,
         is_mine,
         version,
         cards,
+        cards_taken_from,
         stop_is_requested,
     )
 }
 
 /// 没编进 `gpu` 特性走不到这里（[`gpu_cards_in_use`] 先报错）。
 #[cfg(not(feature = "gpu"))]
+#[allow(
+    clippy::too_many_arguments,
+    reason = "与编进 gpu 特性的那一份同一个签名"
+)]
 fn judge_my_blocks_on_gpu(
     _store: &mut VerdictStore,
     _flow: &CrashFlow<'_>,
@@ -4398,8 +4639,9 @@ fn judge_my_blocks_on_gpu(
     _is_mine: &dyn Fn(u64) -> bool,
     _version: &JudgeVersion,
     _cards: &CardsInUse,
+    _cards_taken_from: GpuCardsSource,
     _stop_is_requested: &(dyn Fn() -> bool + Sync),
-) -> Result<(CheckingTally, GpuJudgeOutcome), String> {
+) -> Result<GpuJudgingOfMyBlocks, String> {
     Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
 }
 
@@ -4449,13 +4691,35 @@ fn verify_my_blocks_on_the_first_card(
     Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
 }
 
+/// 只排派活计划那一趟的结局。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+#[cfg_attr(
+    not(feature = "gpu"),
+    allow(
+        dead_code,
+        reason = "没编进 gpu 特性时只排计划的那一份只报错，打了那一行的结局只在编进来时造"
+    )
+)]
+enum PlacementOfThisRun {
+    /// 打了 `CRASH_AMPLIFICATION_PLACEMENT` 那一行。
+    Printed,
+    /// 起过卡池的进程借试判的草稿区等满了：「本次未跑」那一行已经打了，没打 PLACEMENT 行。
+    NotRunTheScratchWasNotBookedInTime,
+}
+
 /// 只排派活计划：这一趟要判多少、配置的每张卡按额度带得了多少、按优先级各装多少；这台机器装了状态的卡各试判几次派活量速度。
 /// 打一行 `CRASH_AMPLIFICATION_PLACEMENT`（带这一趟录入时库里已判、因此不排的态数 `states_reused`）。别的机器上的卡带得了多少按它的额度算
-/// （绑定上限按本机第一张卡估）。
+/// （绑定上限按本机第一张卡估）。起过卡池的这一趟（`cards_taken_from`），本机每张卡装几个状态照账能借的量压，试判之前按装的状态数
+/// 借草稿区（同第 ① 段，[`scratch_of_each_card_booked_by_where_the_cards_came_from`]），等满了借不到交
+/// [`PlacementOfThisRun::NotRunTheScratchWasNotBookedInTime`]。
 #[cfg(feature = "gpu")]
 #[allow(
     clippy::too_many_arguments,
-    reason = "库、流、计划、分工、判法版本、卡、配置、机器、复用数，各是各的东西，凑成结构体只为让签名变短"
+    reason = "库、流、计划、分工、判法版本、卡、卡的来路、配置、机器、复用数，各是各的东西，凑成结构体只为让签名变短"
+)]
+#[allow(
+    clippy::too_many_lines,
+    reason = "备表、按配置排每张卡、借草稿区、试判量速度、打行，是只排计划那一趟的全部步骤，拆开要把十几个量来回传"
 )]
 fn print_the_placement(
     store: &VerdictStore,
@@ -4464,10 +4728,11 @@ fn print_the_placement(
     is_mine: &dyn Fn(u64) -> bool,
     version: &JudgeVersion,
     cards: &CardsInUse,
+    cards_taken_from: GpuCardsSource,
     configured: Option<&[CardByPriority]>,
     host: CardHost,
     states_reused: u64,
-) -> Result<(), String> {
+) -> Result<PlacementOfThisRun, String> {
     use crate::crash_judge_dispatch::{states_in_flight_within, GpuCrashJudge, PackedJudgeTables};
     use crate::crash_judge_tables::{build_judge_tables, JudgeFlow};
     /// 量完宽度之后计时判几次派活那么多的状态。
@@ -4489,8 +4754,8 @@ fn print_the_placement(
         versions: flow.versions,
         expansion: &expansion,
     };
-    let tables = match build_judge_tables(&judge_flow) {
-        Ok(tables) => PackedJudgeTables::of(&tables),
+    let (states_of_the_flow, tables) = match build_judge_tables(&judge_flow) {
+        Ok(tables) => (tables.state_count, PackedJudgeTables::of(&tables)),
         Err(refusal) => {
             return Err(format!(
                 "GPU 判器建不出这条流的表（{refusal:?}；{} 块、{pending_states} 个状态要判）。配置开了 GPU，不退回 CPU 判器：让判器的表认得这种写再跑",
@@ -4498,7 +4763,16 @@ fn print_the_placement(
             ));
         }
     };
-    let local_capacities = states_in_flight_of_each_card(cards, &tables);
+    let local_capacities = match cards_taken_from {
+        GpuCardsSource::TheCardPoolOfThisProcess => states_each_card_of_the_pool_lends(
+            the_card_pool_this_run_takes_its_cards_from(),
+            cards,
+            &tables,
+        )?,
+        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
+            states_in_flight_of_each_card(cards, &tables)
+        }
+    };
     let limit = cards
         .first()
         .expect("用的卡至少一张（gpu_cards_in_use 判过）")
@@ -4543,11 +4817,34 @@ fn print_the_placement(
         .max_by_key(|(_, local)| local.end - local.start)
     {
         let point = &plan.points[*point_index];
+        // 本机每张卡装的状态数（按这一趟卡的次序）：试判的草稿区照它借，借据活到试判完
+        let states_placed_on_local_cards: Vec<u64> = cards
+            .iter()
+            .map(|card| {
+                rows.iter()
+                    .zip(&placed)
+                    .find(|((row_host, index, _), _)| *row_host == host && *index == card.index)
+                    .map_or(0, |(_, states_placed)| *states_placed)
+            })
+            .collect();
+        let (cards_for_the_judges, _scratch_leases) =
+            match scratch_of_each_card_booked_by_where_the_cards_came_from(
+                cards_taken_from,
+                cards,
+                &tables,
+                states_of_the_flow,
+                &states_placed_on_local_cards,
+            )? {
+                ScratchOfEachCard::Ready { cards, leases } => (cards, leases),
+                ScratchOfEachCard::NotRunTheScratchWasNotBookedInTime => {
+                    return Ok(PlacementOfThisRun::NotRunTheScratchWasNotBookedInTime);
+                }
+            };
         for ((row_host, index, _), states_placed) in rows.iter().zip(&placed) {
             if *row_host != host || *states_placed == 0 {
                 continue;
             }
-            let card = cards
+            let card = cards_for_the_judges
                 .iter()
                 .find(|card| card.index == *index)
                 .expect("装了状态的本机卡是起好的");
@@ -4604,7 +4901,7 @@ fn print_the_placement(
             0.0
         }
     );
-    Ok(())
+    Ok(PlacementOfThisRun::Printed)
 }
 
 /// 没编进 `gpu` 特性走不到这里（[`gpu_cards_in_use`] 先报错）。
@@ -4620,9 +4917,10 @@ fn print_the_placement(
     _is_mine: &dyn Fn(u64) -> bool,
     _version: &JudgeVersion,
     _cards: &CardsInUse,
+    _cards_taken_from: GpuCardsSource,
     _configured: Option<&[CardByPriority]>,
     _host: CardHost,
     _states_reused: u64,
-) -> Result<(), String> {
+) -> Result<PlacementOfThisRun, String> {
     Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
 }
````

### 10v mutations-append.tsv

````text
第10v批 显存账 没配显卡优先级时 3/4 那一份不并进最小值	crates/singlefs-harness/src/gpu_memory_ledger.rs	            None => budget_mebibytes.saturating_sub(mebibytes_left_for_the_driver),	            None => u64::MAX,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_environment_quota_without_a_card_priority_configured_stays_within_the_default_three_quarter_share	an_environment_quota_without_a_card_priority_configured_stays_within_the_default_three_quarter_share
第10v批 显存账 起过卡池的产品路径草稿区照样不进账	crates/singlefs-harness/src/gpu_memory_ledger.rs	        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {\n            ScratchStatesToBook::NotBooked\n        }\n        GpuCardsSource::TheCardPoolOfThisProcess => ScratchStatesToBook::OnEachCard(	        GpuCardsSource::NoCardNeeded | GpuCardsSource::TheCardPoolOfThisProcess => {\n            ScratchStatesToBook::NotBooked\n        }\n        GpuCardsSource::DevicesStartedForThisRun => ScratchStatesToBook::OnEachCard(	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_process_that_started_the_card_pool_books_the_scratch_of_each_card_for_the_states_placed_on_it	a_process_that_started_the_card_pool_books_the_scratch_of_each_card_for_the_states_placed_on_it
第10v批 显存账 没装状态的卡也借一个状态的草稿区	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    if *states_placed == 0 {	                    if *states_placed == u64::MAX {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_process_that_started_the_card_pool_books_the_scratch_of_each_card_for_the_states_placed_on_it	a_process_that_started_the_card_pool_books_the_scratch_of_each_card_for_the_states_placed_on_it
第10v批 显存账 产品路径借草稿区等满了判红（不报本次未跑）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        } => ScratchBookingRefusalOutcome::NotRun {\n            not_run_line: gpu_not_run_line(	        } => ScratchBookingRefusalOutcome::Red {\n            reason: gpu_not_run_line(	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_scratch_booking_that_waits_past_the_longest_wait_is_reported_not_run_with_the_gpu_not_run_marker	a_scratch_booking_that_waits_past_the_longest_wait_is_reported_not_run_with_the_gpu_not_run_marker
第10v批 显存账 本次未跑那一行的记号与 GPU 用例的不同	crates/singlefs-harness/src/gpu_memory_ledger.rs	pub const GPU_NOT_RUN_MARKER: &str = "NOT_RUN_NO_GPU_ADAPTER";	pub const GPU_NOT_RUN_MARKER: &str = "NOT_RUN_SCRATCH";	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_scratch_booking_that_waits_past_the_longest_wait_is_reported_not_run_with_the_gpu_not_run_marker	a_scratch_booking_that_waits_past_the_longest_wait_is_reported_not_run_with_the_gpu_not_run_marker
第10v批 显存账 点名一张卡借时照优先级挑卡	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.book_a_lease_waiting(WhichCards::OnlyTheCardAt { card_position }, need_mebibytes)	        self.book_a_lease_waiting(WhichCards::FirstThatFitsByPriority, need_mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_booking_on_a_named_card_stays_on_that_card_even_when_an_earlier_card_has_room	a_booking_on_a_named_card_stays_on_that_card_even_when_an_earlier_card_has_room
第10v批 显存账 要的正好等于容量的状态数判成放不下	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if mebibytes_for_states(middle) <= capacity_mebibytes {	        if mebibytes_for_states(middle) < capacity_mebibytes {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_most_states_whose_need_fits_is_the_largest_count_whose_need_is_within_the_capacity	the_most_states_whose_need_fits_is_the_largest_count_whose_need_is_within_the_capacity
````

## 第 10w 批

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index 9d37e97..facac84 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -20,12 +20,26 @@
 //!    `crate::crash_judge_dispatch::mebibytes_a_judge_needs` 算，只记在那张卡上），每张卡装几个状态先照账能借的量压（借得到才装）；
 //!    等满了借不到，照对拍用例那一路打「本次未跑」那一行（同一个记号），这一趟照叫停的样子交回（`stopped_early`，判过的块在库里，
 //!    分项驱动把这一项记成本次未跑；派出的项全没判完时整趟退 77）。只排派活计划那一趟试判前同样借。
+//!    几张卡一起借（第 10w 批，主 agent 定）：取卡时登记驱动那一份、第 ① 段借草稿区，都按卡的先后每张当场试一次、不等，借不到的卡
+//!    这一趟不用（不装状态，它那一份留给后面的卡），一张都借不到才等、等满报本次未跑；每张卡装几个状态在账本锁里照此刻的估计再压一次
+//!    （压完到借之间别的进程把估计改小了，照新的估计重压再借，不判红）。
+//!    第 ② 段核对（`crash_verify_gpu`）与第 ① 段由 CPU 判时的单元校验和对拍（`gpu_unit_checks`）在卡上建的缓冲同样进账：照卡的先后借第一张
+//!    放得下的（此刻每张都放不下才等），用借来的那张卡算，等满了报本次未跑（判过的块在库里）。量照
+//!    `singlefs_harness::gpu_memory_ledger::mebibytes_of_buffers_held_at_once`，下面的字节数是照两份代码建的缓冲推的，没上卡量过。
+//!    第 ② 段（`facts_verifier_buffer_footprint`）：上卡一次的六张事实表（`crate::crash_verify_gpu::GpuFactsVerifier::new`：
+//!    录制流每次写 4 个字、每对重放 2 个、每段 6 个、段里每次写 1 个、每个单元 6 个、每条根 38 个，空表也建一个字），
+//!    加一块一趟的头（12 个字）、结论与回读（每个状态各 1 个字），一块按 [`STATES_PER_BLOCK`] 个状态算；核对一块接一块，同时还活着的
+//!    照 `PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD` 算。几个字照 `crash_verify_gpu.rs` 里那几个常数抄的，那边改了布局这里要跟着改。
+//!    单元校验和（`unit_checksum_buffer_footprint`）：一个状态一趟，这条流里每次单元写各两段
+//!    （`crate::gpu_unit_checks::bytes_crc32c_of_ranges_needs`），核对线程数（[`checking_threads`]）条线程同时派活，
+//!    每条同时还活着的照 `PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD` 算。
+//!    驱动的上下文与编好的管线算在登记时的驱动那一份里，不另记。
 //!    没起过卡池的进程照旧不进账（[`GpuCardsSource::DevicesStartedForThisRun`]：这一趟自己在点名的卡上起设备），调用方是这几种：
 //!    崩溃放量分项驱动（`scripts/crash/crash-amplification-item.sh`）起的、只有经 [`run_from_environment`] 跑的用例而没有一条取卡池的测试二进制
 //!    （`tests/crash_amplification_fixed_script_through_rollback.rs`、`…_through_unmount.rs`、`crash_amplification_two_flows_share_their_prefix.rs`、
 //!    经 `tests/common_crash_points` 跑的 `crash_enumeration_*` 那几份）；同一个测试二进制里有取卡池的 GPU 用例、而经 [`run_from_environment`]
 //!    的用例跑在它们起池之前的（`crash_amplification_pipeline_records_checks_and_reuses.rs`、`record_checker_judges_absence_by_the_persisted_set.rs`）。
-//!    直接调 `judge_my_blocks_on_gpu_cards` 的对拍用例不走这一条，卡由用例自己给、不进账。第 ② 段核对与单元校验和对拍在第一张卡上建的缓冲不进账。
+//!    直接调 `judge_my_blocks_on_gpu_cards` 的对拍用例不走这一条，卡由用例自己给、不进账。
 
 use std::collections::{BTreeMap, BTreeSet};
 use std::ops::Range;
@@ -38,8 +52,10 @@ use singlefs_core::recovery::{recover, JournalPolicy};
 use singlefs_core::root_record::ROOT_MAGIC;
 #[cfg(feature = "gpu")]
 use singlefs_harness::gpu_memory_ledger::{
-    most_states_whose_need_fits_the_capacity, scratch_booking_refusal_outcome,
-    scratch_states_to_book_on_each_card, ScratchBookingRefusalOutcome, ScratchStatesToBook,
+    gpu_not_run_line, mebibytes_of_buffers_held_at_once, most_states_whose_need_fits_the_capacity,
+    scratch_booking_refusal_outcome, scratch_states_to_book_on_each_card, BookingRefusal,
+    NeedLargerThanEveryCard, ScratchBookingRefusalOutcome, ScratchStatesToBook,
+    PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
 };
 use singlefs_harness::hexadecimal::hexadecimal_text;
 use singlefs_harness::memory_pool::{
@@ -1443,6 +1459,99 @@ pub fn gpu_judge_wall_clock_line(
     )
 }
 
+/// 一段在卡上建的缓冲合起来的字节数（文件头第 5 条），折成 MiB 照 `singlefs_harness::gpu_memory_ledger::mebibytes_of_buffers_held_at_once`。
+#[cfg(feature = "gpu")]
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct GpuBufferFootprint {
+    bytes_uploaded_once: u64,
+    bytes_of_one_pass: u64,
+    passes_alive_at_once: u64,
+}
+
+#[cfg(feature = "gpu")]
+impl GpuBufferFootprint {
+    fn mebibytes(self) -> u64 {
+        mebibytes_of_buffers_held_at_once(
+            self.bytes_uploaded_once,
+            self.bytes_of_one_pass,
+            self.passes_alive_at_once,
+        )
+    }
+}
+
+/// `crate::crash_verify_gpu::GpuFactsVerifier` 建的几张缓冲里每一项占几个 u32：照 `crash_verify_gpu.rs` 的 `RECORDED_WORDS`、
+/// 每对重放两个字、`SEGMENT_WORDS`、`UNIT_WORDS`、`ROOT_WORDS`、`HEADER_WORDS` 抄（那几个常数那边是私有的，这一批碰不到那份文件），
+/// 那边改了布局这里跟着改。
+#[cfg(feature = "gpu")]
+const FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE: u64 = 4;
+#[cfg(feature = "gpu")]
+const FACTS_VERIFIER_WORDS_PER_REPLAY: u64 = 2;
+#[cfg(feature = "gpu")]
+const FACTS_VERIFIER_WORDS_PER_SEGMENT: u64 = 6;
+#[cfg(feature = "gpu")]
+const FACTS_VERIFIER_WORDS_PER_UNIT: u64 = 6;
+#[cfg(feature = "gpu")]
+const FACTS_VERIFIER_WORDS_PER_ROOT: u64 = 2 + 4 * (1 + 2 * 4);
+#[cfg(feature = "gpu")]
+const FACTS_VERIFIER_HEADER_WORDS: u64 = 12;
+#[cfg(feature = "gpu")]
+const BYTES_PER_GPU_WORD: u64 = 4;
+
+/// 第 ② 段在卡上核一条流（`crate::crash_verify_gpu::GpuFactsVerifier`）建的缓冲（文件头第 5 条）：六张事实表上卡一次（空表也建一个字），
+/// 核一块一趟建头、结论与回读（每个状态各一个字），一块按 `states_in_the_largest_block` 个状态算；一块接一块核，
+/// 同时还活着的照一条线程算。
+#[cfg(feature = "gpu")]
+fn facts_verifier_buffer_footprint(
+    facts: &FlowFacts,
+    states_in_the_largest_block: u64,
+) -> GpuBufferFootprint {
+    let count = |length: usize| u64::try_from(length).expect("表长装得进 u64");
+    let replays: u64 = facts
+        .torn_images
+        .iter()
+        .map(|torn| count(torn.replays.len()))
+        .sum();
+    let writes_of_the_segments: u64 = facts
+        .segments
+        .iter()
+        .map(|segment| count(segment.writes.len()))
+        .sum();
+    let words_of_each_table = [
+        count(facts.is_tearable.len()) * FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE,
+        replays * FACTS_VERIFIER_WORDS_PER_REPLAY,
+        count(facts.segments.len()) * FACTS_VERIFIER_WORDS_PER_SEGMENT,
+        writes_of_the_segments,
+        count(facts.units.len()) * FACTS_VERIFIER_WORDS_PER_UNIT,
+        count(facts.roots.len()) * FACTS_VERIFIER_WORDS_PER_ROOT,
+    ];
+    GpuBufferFootprint {
+        bytes_uploaded_once: words_of_each_table
+            .iter()
+            .map(|words| (*words).max(1) * BYTES_PER_GPU_WORD)
+            .sum(),
+        bytes_of_one_pass: (FACTS_VERIFIER_HEADER_WORDS + 2 * states_in_the_largest_block)
+            * BYTES_PER_GPU_WORD,
+        passes_alive_at_once: PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
+    }
+}
+
+/// 第 ① 段由 CPU 判、单元两道校验和交卡对拍（[`gpu_unit_checksum_items`]）建的缓冲（文件头第 5 条）：一个状态一趟，
+/// 这条流里每次单元写各两段（`crate::gpu_unit_checks::bytes_crc32c_of_ranges_needs`），`checking_threads` 条线程同时派活。
+#[cfg(feature = "gpu")]
+fn unit_checksum_buffer_footprint(
+    unit_writes_in_the_flow: usize,
+    checking_threads: usize,
+) -> GpuBufferFootprint {
+    GpuBufferFootprint {
+        bytes_uploaded_once: 0,
+        bytes_of_one_pass: crate::gpu_unit_checks::bytes_crc32c_of_ranges_needs(
+            2 * unit_writes_in_the_flow,
+        ),
+        passes_alive_at_once: u64::try_from(checking_threads).expect("线程数装得进 u64")
+            * PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
+    }
+}
+
 /// 一个崩溃镜像里被写表写过的每个单元（数据单元、节点、打包单元）两道校验和在 GPU 上算、与 CPU 对拍，
 /// 交回判红的项（`gpu_unit:<盘>:<偏移>:<结论>`）。对不上的记 `gpu_cpu_disagreement:…`，整趟不作数。
 #[cfg(feature = "gpu")]
@@ -1611,6 +1720,8 @@ pub struct GpuCardOutcome {
     /// 带得最多的一次派活几个状态、卡算得最久的一次派活多久（一次派活带几个按用时调，不一定到 `states_per_dispatch`）。
     pub states_in_the_widest_dispatch: u64,
     pub seconds_of_the_longest_dispatch: f64,
+    /// 起过卡池的进程在显存账上给这张卡的判器借的草稿区（MiB，文件头第 5 条）；不进账（卡是这一趟自己起的、或调用方给的）是 `None`。
+    pub scratch_booked_mebibytes: Option<u64>,
 }
 
 impl GpuCardOutcome {
@@ -1807,6 +1918,8 @@ struct CardsAtWork<'work> {
     cards: &'work [std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
     /// 每张卡按优先级装到的状态数（一次派活带几个）；0 的这一趟不用。
     states_placed: &'work [u64],
+    /// 每张卡的判器在显存账上借的草稿区（MiB）；不进账的是 `None`。
+    scratch_booked_mebibytes: &'work [Option<u64>],
     next_pending: std::sync::atomic::AtomicUsize,
     /// 写库失败之后置上：卡上的线程判完手里那一块就停。
     stop: std::sync::atomic::AtomicBool,
@@ -1867,6 +1980,7 @@ fn judge_on_one_card_and_start_the_next<'scope, 'work>(
         states_per_dispatch: u64::try_from(judge.states_per_dispatch()).expect("装得进 u64"),
         scratch_mebibytes: u64::try_from(judge.scratch_mebibytes()).expect("装得进 u64"),
         build_seconds,
+        scratch_booked_mebibytes: work.scratch_booked_mebibytes[position],
         ..GpuCardOutcome::default()
     };
     while !work.must_stop() {
@@ -2061,19 +2175,15 @@ fn the_card_pool_this_run_takes_its_cards_from(
 /// 账本读写不了。
 ///
 /// # Panics
-/// 这一趟的卡与池里的卡张数不同（[`cards_from_the_card_pool_of_this_process`] 按池的次序交回，逐张对上）。
+/// 这一趟有一张卡不是池里的卡（[`gpu_cards_in_use`] 照池交回登记上的那几张）。
 #[cfg(feature = "gpu")]
 fn states_each_card_of_the_pool_lends(
     pool: &crate::gpu_unit_checks::gpu_test_start::GpuCardPool,
     cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
     tables: &crate::crash_judge_dispatch::PackedJudgeTables,
 ) -> Result<Vec<u64>, String> {
-    let ledger_capacities = pool.card_capacities_mebibytes()?;
-    assert_eq!(
-        ledger_capacities.len(),
-        cards.len(),
-        "这一趟照池里的卡用：卡与池里的卡逐张对上"
-    );
+    let card_indices: Vec<u32> = cards.iter().map(|card| card.index).collect();
+    let ledger_capacities = pool.card_capacities_mebibytes_of(&card_indices)?;
     Ok(states_in_flight_of_each_card(cards, tables)
         .into_iter()
         .zip(ledger_capacities)
@@ -2089,85 +2199,138 @@ fn states_each_card_of_the_pool_lends(
         .collect())
 }
 
-/// 第 ① 段每张卡上判器用的卡：借了草稿区的那几张换成借来的卡（额度是借的量加驱动那一份，判器照它开出来的草稿区正好是借的状态数）。
+/// 起过卡池的产品路径在显存账上借不到时怎么报（`what_was_booked` 说借的是什么）：等满了照对拍用例那一路打「本次未跑」那一行
+/// （[`scratch_booking_refusal_outcome`]，同一个记号）、交 `Ok(())`，调用方照叫停的样子交回；要的比每张卡的账都大、账本读写不了判红。
+///
+/// # Errors
+/// 要的比每张卡的账都大（说法与没有一张卡装得下这条流的表那一句同一类，分项驱动照它认成设备用不上）、账本读写不了。
+#[cfg(feature = "gpu")]
+fn print_the_not_run_line_or_refuse(
+    what_was_booked: &str,
+    refusal: &BookingRefusal,
+) -> Result<(), String> {
+    if let BookingRefusal::NeedLargerThanEveryCard(NeedLargerThanEveryCard {
+        need_mebibytes,
+        largest_capacity_mebibytes,
+    }) = refusal
+    {
+        return Err(format!(
+            "没有一张卡装得下这条流的表与{what_was_booked}：要借 {need_mebibytes} MiB，池里最大的一张卡的显存账只有 {largest_capacity_mebibytes} MiB。\
+             看配置里每张卡的显存额度与卡上此刻的空闲显存"
+        ));
+    }
+    match scratch_booking_refusal_outcome(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, 0, refusal) {
+        ScratchBookingRefusalOutcome::NotRun { not_run_line } => {
+            println!("{not_run_line}");
+            Ok(())
+        }
+        ScratchBookingRefusalOutcome::Red { reason } => Err(format!("{what_was_booked}：{reason}")),
+    }
+}
+
+/// 第 ① 段每张卡上的判器用什么：借了草稿区的那几张换成借来的卡（额度是借的量加驱动那一份，判器照它开出来的草稿区正好是借的状态数），
+/// 每张卡装几个状态（一次派活带几个；0 的这一趟不用），借了多少；按这一趟卡的次序。
+#[cfg(feature = "gpu")]
+struct ScratchOfTheCards {
+    cards_for_the_judges: Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>>,
+    states_placed: Vec<u64>,
+    scratch_booked_mebibytes: Vec<Option<u64>>,
+    /// 借据（不进账时是空的），判器都丢掉之后才还。
+    leases: Vec<crate::gpu_unit_checks::gpu_test_start::GpuCardLease>,
+}
+
+/// 第 ① 段借草稿区的结局。
 #[cfg(feature = "gpu")]
 enum ScratchOfEachCard {
-    /// `cards` 按这一趟卡的次序；`leases` 是借据（不进账时是空的），判器都丢掉之后才还。
-    Ready {
-        cards: Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>>,
-        leases: Vec<crate::gpu_unit_checks::gpu_test_start::GpuCardLease>,
-    },
-    /// 等满了借不到：「本次未跑」那一行已经打了，已经借到的几张都还了。
+    Ready(ScratchOfTheCards),
+    /// 一张卡都借不到、等满了：「本次未跑」那一行已经打了，什么都没借着。
     NotRunTheScratchWasNotBookedInTime,
 }
 
-/// 照卡的来路给第 ① 段借草稿区（[`scratch_states_to_book_on_each_card`]）：起过卡池的，每张装了状态的卡按它装的状态数在那张卡上借
-/// （显存按 `crate::crash_judge_dispatch::mebibytes_a_judge_needs`，与 GPU 对拍用例借卡同一份），按卡的次序一张一张借，放不下就等；
-/// 等满了照对拍用例那一路打「本次未跑」那一行（[`scratch_booking_refusal_outcome`]）。没起过卡池的不借、卡照原样。
+/// 照卡的来路给第 ① 段装状态、借草稿区（[`scratch_states_to_book_on_each_card`]）。`states_each_card_holds` 是每张卡最多装几个
+/// （起过卡池的已照账能借的量压过，[`states_each_card_of_the_pool_lends`]）。起过卡池的：按卡的先后把 `pending_states` 个状态装到此刻借得到的卡上，
+/// 每张在账本锁里照此刻的估计再压一次再借（显存按 `crate::crash_judge_dispatch::mebibytes_a_judge_needs`，与 GPU 对拍用例借卡同一份），
+/// 借不到的卡不装、不等，一张都借不到才等；等满了打「本次未跑」那一行（[`print_the_not_run_line_or_refuse`]）。
+/// 没起过卡池的不借、卡照原样，照 [`place_states_on_cards_by_priority`] 装。
 ///
 /// # Errors
-/// 要借的比那张卡的账大、账本读写不了（判红）。
+/// 每张卡的账连一个状态都放不下、账本读写不了（判红）。
 #[cfg(feature = "gpu")]
 fn scratch_of_each_card_booked_by_where_the_cards_came_from(
     cards_taken_from: GpuCardsSource,
     cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
     tables: &crate::crash_judge_dispatch::PackedJudgeTables,
     state_count: u64,
-    states_placed: &[u64],
+    pending_states: u64,
+    states_each_card_holds: &[u64],
 ) -> Result<ScratchOfEachCard, String> {
-    match scratch_states_to_book_on_each_card(cards_taken_from, states_placed, state_count) {
-        ScratchStatesToBook::NotBooked => Ok(ScratchOfEachCard::Ready {
-            cards: cards.to_vec(),
+    match scratch_states_to_book_on_each_card(cards_taken_from, states_each_card_holds, state_count)
+    {
+        ScratchStatesToBook::NotBooked => Ok(ScratchOfEachCard::Ready(ScratchOfTheCards {
+            cards_for_the_judges: cards.to_vec(),
+            states_placed: place_states_on_cards_by_priority(
+                pending_states,
+                states_each_card_holds,
+            ),
+            scratch_booked_mebibytes: vec![None; cards.len()],
             leases: Vec::new(),
-        }),
-        ScratchStatesToBook::OnEachCard(states_to_book) => {
-            let pool = the_card_pool_this_run_takes_its_cards_from();
-            let mut cards_for_the_judges = cards.to_vec();
-            let mut leases = Vec::new();
-            for (card_position, states) in states_to_book.iter().enumerate() {
-                if *states == 0 {
-                    continue;
-                }
-                match pool.lease_the_card_at_waiting_for_room(
+        })),
+        ScratchStatesToBook::OnEachCard(states_each_card_lends) => {
+            let card_indices: Vec<u32> = cards.iter().map(|card| card.index).collect();
+            let states_each_card_lends: Vec<u64> = states_each_card_lends
+                .into_iter()
+                .map(|states| u64::try_from(states).expect("状态数装得进 u64"))
+                .collect();
+            let booked = match the_card_pool_this_run_takes_its_cards_from()
+                .book_the_scratch_of_the_cards_waiting_only_when_none_has_room(
                     PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
-                    card_position,
-                    crate::crash_judge_dispatch::mebibytes_a_judge_needs(tables, *states),
+                    &card_indices,
+                    pending_states,
+                    &states_each_card_lends,
+                    &|states| {
+                        crate::crash_judge_dispatch::mebibytes_a_judge_needs(
+                            tables,
+                            usize::try_from(states).unwrap_or(usize::MAX),
+                        )
+                    },
                 ) {
-                    Ok(lease) => {
-                        cards_for_the_judges[card_position] =
-                            std::sync::Arc::new(lease.card().clone());
-                        leases.push(lease);
-                    }
-                    Err(refusal) => {
-                        return match scratch_booking_refusal_outcome(
-                            PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
-                            card_position,
-                            &refusal,
-                        ) {
-                            ScratchBookingRefusalOutcome::NotRun { not_run_line } => {
-                                println!("{not_run_line}");
-                                Ok(ScratchOfEachCard::NotRunTheScratchWasNotBookedInTime)
-                            }
-                            ScratchBookingRefusalOutcome::Red { reason } => Err(reason),
-                        };
-                    }
+                Ok(booked) => booked,
+                Err(refusal) => {
+                    print_the_not_run_line_or_refuse("第 ① 段判器的草稿区", &refusal)?;
+                    return Ok(ScratchOfEachCard::NotRunTheScratchWasNotBookedInTime);
+                }
+            };
+            let mut scratch = ScratchOfTheCards {
+                cards_for_the_judges: cards.to_vec(),
+                states_placed: vec![0; cards.len()],
+                scratch_booked_mebibytes: vec![None; cards.len()],
+                leases: Vec::new(),
+            };
+            for (card_slot, booked_on_the_card) in booked.into_iter().enumerate() {
+                if let Some(crate::gpu_unit_checks::gpu_test_start::ScratchLeaseOnACard {
+                    lease,
+                    states,
+                }) = booked_on_the_card
+                {
+                    scratch.cards_for_the_judges[card_slot] =
+                        std::sync::Arc::new(lease.card().clone());
+                    scratch.states_placed[card_slot] = states;
+                    scratch.scratch_booked_mebibytes[card_slot] = Some(lease.mebibytes());
+                    scratch.leases.push(lease);
                 }
             }
-            Ok(ScratchOfEachCard::Ready {
-                cards: cards_for_the_judges,
-                leases,
-            })
+            Ok(ScratchOfEachCard::Ready(scratch))
         }
     }
 }
 
 /// 同 [`judge_my_blocks_on_gpu_cards`]，卡的来路另给：起过卡池的（[`GpuCardsSource::TheCardPoolOfThisProcess`]），每张卡装几个状态
-/// 先照账能借的量压（[`states_each_card_of_the_pool_lends`]），判之前按装的状态数在那张卡上借草稿区
-/// （[`scratch_of_each_card_booked_by_where_the_cards_came_from`]），等满了借不到交 [`GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime`]、
-/// 一块都不判；判完先丢判器、再还借据。
+/// 先照账能借的量压（[`states_each_card_of_the_pool_lends`]），判之前把要判的状态装到此刻借得到草稿区的卡上
+/// （[`scratch_of_each_card_booked_by_where_the_cards_came_from`]：借不到的卡这一趟不用、不等它，一张都借不到才等），等满了借不到交
+/// [`GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime`]、一块都不判；判完先丢判器、再还借据。
 ///
 /// # Errors
-/// 同 [`judge_my_blocks_on_gpu_cards`]；借草稿区时要的比那张卡的账大、账本读写不了。
+/// 同 [`judge_my_blocks_on_gpu_cards`]；借草稿区时每张卡的账连一个状态都放不下、账本读写不了。
 ///
 /// # Panics
 /// 同 [`judge_my_blocks_on_gpu_cards`]。
@@ -2233,7 +2396,28 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
             states_in_flight_of_each_card(cards, &tables)
         }
     };
-    let states_placed = place_states_on_cards_by_priority(pending_states, &capacities);
+    // 借据活到这个函数收尾：几张卡上的判器在下面的线程里判完就丢，丢完才还
+    let scratch = if capacities.iter().all(|capacity| *capacity == 0) {
+        None
+    } else {
+        match scratch_of_each_card_booked_by_where_the_cards_came_from(
+            cards_taken_from,
+            cards,
+            &tables,
+            state_count,
+            pending_states,
+            &capacities,
+        )? {
+            ScratchOfEachCard::Ready(scratch) => Some(scratch),
+            ScratchOfEachCard::NotRunTheScratchWasNotBookedInTime => {
+                return Ok(GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime);
+            }
+        }
+    };
+    let states_placed = scratch.as_ref().map_or_else(
+        || vec![0; cards.len()],
+        |scratch| scratch.states_placed.clone(),
+    );
     for ((card, capacity), placed) in cards.iter().zip(&capacities).zip(&states_placed) {
         eprintln!(
             "CRASH_AMPLIFICATION_GPU_PLACEMENT card={:?} index={} memory_budget_mebibytes={} states_in_flight_capacity={capacity} states_placed={placed} pending_states={pending_states} tables_mebibytes={}",
@@ -2243,32 +2427,21 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
             tables.mebibytes()
         );
     }
-    let Some(first_position) = states_placed.iter().position(|placed| *placed > 0) else {
+    let (Some(scratch), Some(first_position)) =
+        (scratch, states_placed.iter().position(|placed| *placed > 0))
+    else {
         return Err(format!(
             "没有一张卡装得下这条流的表（{} MiB）与一个状态的草稿区：看配置里每张卡的显存额度与卡上此刻的空闲显存",
             tables.mebibytes()
         ));
     };
-    // 借据活到这个函数收尾：几张卡上的判器在下面的线程里判完就丢，丢完才还
-    let (cards_for_the_judges, _scratch_leases) =
-        match scratch_of_each_card_booked_by_where_the_cards_came_from(
-            cards_taken_from,
-            cards,
-            &tables,
-            state_count,
-            &states_placed,
-        )? {
-            ScratchOfEachCard::Ready { cards, leases } => (cards, leases),
-            ScratchOfEachCard::NotRunTheScratchWasNotBookedInTime => {
-                return Ok(GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime);
-            }
-        };
     let work = CardsAtWork {
         plan,
         pending: &pending,
         tables: &tables,
-        cards: &cards_for_the_judges,
+        cards: &scratch.cards_for_the_judges,
         states_placed: &states_placed,
+        scratch_booked_mebibytes: &scratch.scratch_booked_mebibytes,
         next_pending: std::sync::atomic::AtomicUsize::new(0),
         stop: std::sync::atomic::AtomicBool::new(false),
         stop_is_requested,
@@ -2430,12 +2603,26 @@ pub fn verify_blocks_from_facts(
     on: &FactsVerifierOn,
 ) -> Result<VerificationTally, String> {
     let facts = extract_flow_facts(flow.base, flow.writes, flow.segments, flow.expansion);
+    verify_blocks_from_these_facts(store, plan, &facts, is_mine, on)
+}
+
+/// 同 [`verify_blocks_from_facts`]，事实表由调用方抽好（起过卡池的进程按事实表的大小借第 ② 段的缓冲，抽一次、借、再核）。
+///
+/// # Errors
+/// 同 [`verify_blocks_from_facts`]。
+fn verify_blocks_from_these_facts(
+    store: &mut VerdictStore,
+    plan: &CrashPlan,
+    facts: &FlowFacts,
+    is_mine: &dyn Fn(u64) -> bool,
+    on: &FactsVerifierOn,
+) -> Result<VerificationTally, String> {
     let facts_digest = facts.digest();
     store
         .store_flow_facts(&facts_digest, &facts.to_bytes())
         .map_err(|error| format!("事实表写不进库：{error:?}"))?;
     let verifier_digest = on.verifier_digest();
-    let verifier = BlockVerifier::new(on, &facts);
+    let verifier = BlockVerifier::new(on, facts);
     let mut tally = VerificationTally::default();
     let mut block_sequence = 0u64;
     for point in &plan.points {
@@ -2455,7 +2642,7 @@ pub fn verify_blocks_from_facts(
                 continue;
             }
             let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
-            let bits = verifier.verify(&facts, ordinals);
+            let bits = verifier.verify(facts, ordinals);
             tally.verifier_red_states += bits.iter().filter(|value| **value != 0).count() as u64;
             tally.states_verified += bits.len() as u64;
             tally.blocks_verified += 1;
@@ -2748,6 +2935,68 @@ mod tests {
         );
     }
 
+    /// 第 10w 批：第 ② 段核对在卡上建的缓冲进账的量（文件头第 5 条）。录制流 3 次写（12 个字）、一张撕裂镜像带 2 对重放（4 个字）、
+    /// 2 段（12 个字）、段里共 3 次写（3 个字），单元与根都没有（空表各建一个字）：上卡一次 33 个字 132 字节；一块 65 536 个状态一趟
+    /// 建头 12 个字、结论与回读各 65 536 个字，同时两趟活着：合起来 1 048 804 字节，向上取整 2 MiB。
+    #[cfg(feature = "gpu")]
+    #[test]
+    fn the_facts_verifier_books_its_tables_once_and_two_blocks_of_header_verdicts_and_readback() {
+        let facts = FlowFacts {
+            recorded_write_count: 3,
+            enumerated_write_count: 3,
+            is_tearable: vec![true, false, false],
+            torn_images: vec![crate::crash_facts::TornImageFact {
+                write_index: 0,
+                torn_image_index: 0,
+                replays: vec![(0, 1), (1, 2)],
+            }],
+            segments: vec![
+                crate::crash_facts::SegmentFact {
+                    writes: vec![0, 1],
+                    states: 0..4,
+                },
+                crate::crash_facts::SegmentFact {
+                    writes: vec![2],
+                    states: 4..6,
+                },
+            ],
+            state_count: 6,
+            units: Vec::new(),
+            roots: Vec::new(),
+            newest_base_root: None,
+        };
+        let footprint = facts_verifier_buffer_footprint(&facts, STATES_PER_BLOCK);
+        assert_eq!(
+            footprint,
+            GpuBufferFootprint {
+                bytes_uploaded_once: 132,
+                bytes_of_one_pass: (12 + 2 * 65_536) * 4,
+                passes_alive_at_once: 2,
+            },
+            "六张表上卡一次（空表各一个字），一块一趟建头、结论与回读"
+        );
+        assert_eq!(footprint.mebibytes(), 2);
+    }
+
+    /// 第 10w 批：单元校验和对拍在卡上建的缓冲进账的量（文件头第 5 条）。这条流 10 次单元写、32 条核对线程：一个状态一趟 20 段，
+    /// 每段一个 32 KiB 的槽、一个长度字、一条结果与回读，加 16 字节派活参数，共 655 776 字节；每条线程同时两趟活着：64 趟，
+    /// 合起来 41 969 664 字节，向上取整 41 MiB。
+    #[cfg(feature = "gpu")]
+    #[test]
+    fn unit_checksums_book_two_ranges_per_unit_write_for_two_passes_on_every_checking_thread() {
+        let footprint = unit_checksum_buffer_footprint(10, 32);
+        assert_eq!(
+            footprint,
+            GpuBufferFootprint {
+                bytes_uploaded_once: 0,
+                bytes_of_one_pass: 20 * (32_768 + 4 + 16) + 16,
+                passes_alive_at_once: 64,
+            },
+            "每次单元写两段、每条线程两趟"
+        );
+        assert_eq!(footprint.mebibytes(), 41);
+    }
+
     /// 显卡优先级照写的次序读：写在前面的先装；额度带单位（G、M，1024 进制）。写法不对的每一种都拒。
     #[test]
     fn cards_by_priority_are_read_in_the_order_written_with_their_quotas() {
@@ -3232,6 +3481,7 @@ mod tests {
                     seconds_decoding: 0.0,
                     states_in_the_widest_dispatch: 0,
                     seconds_of_the_longest_dispatch: 0.0,
+                    scratch_booked_mebibytes: None,
                 };
                 card_sender
                     .send(CardMessage::Finished(finished_card))
@@ -3746,6 +3996,10 @@ pub struct EnvironmentRun {
     pub states_skipped_by_ignore: u64,
     /// 这一趟用的卡从哪来（[`GpuCardsSource`]）。
     pub gpu_cards_taken_from: GpuCardsSource,
+    /// 起过卡池的进程第 ① 段由 CPU 判、单元校验和交卡对拍时，在显存账上为对拍的缓冲借的（MiB，文件头第 5 条）；没借是 `None`。
+    pub unit_checksum_buffers_booked_mebibytes: Option<u64>,
+    /// 起过卡池的进程第 ② 段在卡上核时，在显存账上为核对的缓冲借的（MiB，文件头第 5 条）；没借是 `None`。
+    pub facts_verifier_buffers_booked_mebibytes: Option<u64>,
 }
 
 /// 一趟用的卡从哪来（定义在 harness 档，草稿区记不记账的判法 `singlefs_harness::gpu_memory_ledger::scratch_states_to_book_on_each_card`
@@ -4157,31 +4411,44 @@ pub fn run_from_environment(
         }
     }
     let seconds_planning_and_recording = planning_started.elapsed().as_secs_f64();
-    let (cards, gpu_cards_taken_from) = if routes.need_a_card() {
-        let (cards, taken_from) = gpu_cards_in_use(cards_by_priority.as_deref(), host)?;
-        (Some(cards), taken_from)
-    } else {
-        (None, GpuCardsSource::NoCardNeeded)
-    };
+    // 起过卡池的进程在池里一张卡都登记不上、等满了：「本次未跑」那一行已经打了，下面照叫停的样子交回（文件头第 5 条）
+    let (cards, gpu_cards_taken_from, no_card_of_the_pool_was_registered_in_time) =
+        if routes.need_a_card() {
+            match gpu_cards_in_use(cards_by_priority.as_deref(), host)? {
+                GpuCardsOfThisRun::Taken { cards, taken_from } => (Some(cards), taken_from, false),
+                GpuCardsOfThisRun::NotRunNoCardOfThePoolHadRoomInTime => {
+                    (None, GpuCardsSource::TheCardPoolOfThisProcess, true)
+                }
+            }
+        } else {
+            (None, GpuCardsSource::NoCardNeeded, false)
+        };
     if placement_only {
-        let (ComputedOn::Gpu, Some(cards)) = (routes.crash_judge, &cards) else {
-            return Err(format!(
-                "只排派活计划排的是 GPU 判器的活：要配置开着 GPU（ENABLE_GPU=1）、{GPU_COMPUTATION_CLASSES_KEY} 里登记 {}",
-                GpuComputationClass::CrashJudgeGpu.name()
-            ));
+        let placement = match (
+            routes.crash_judge,
+            &cards,
+            no_card_of_the_pool_was_registered_in_time,
+        ) {
+            (ComputedOn::Gpu, _, true) => PlacementOfThisRun::NotRunTheScratchWasNotBookedInTime,
+            (ComputedOn::Gpu, Some(cards), false) => print_the_placement(
+                &store,
+                flow,
+                &plan,
+                &|block| selected.contains(&block),
+                &version,
+                cards,
+                gpu_cards_taken_from,
+                cards_by_priority.as_deref(),
+                host,
+                states_reused_at_recording,
+            )?,
+            (ComputedOn::Gpu, None, false) | (ComputedOn::Cpu, _, _) => {
+                return Err(format!(
+                    "只排派活计划排的是 GPU 判器的活：要配置开着 GPU（ENABLE_GPU=1）、{GPU_COMPUTATION_CLASSES_KEY} 里登记 {}",
+                    GpuComputationClass::CrashJudgeGpu.name()
+                ));
+            }
         };
-        let placement = print_the_placement(
-            &store,
-            flow,
-            &plan,
-            &|block| selected.contains(&block),
-            &version,
-            cards,
-            gpu_cards_taken_from,
-            cards_by_priority.as_deref(),
-            host,
-            states_reused_at_recording,
-        )?;
         let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &version)
             .map_err(|error| format!("对账失败：{error:?}"))?
             .len();
@@ -4215,54 +4482,143 @@ pub fn run_from_environment(
             ignored_points,
             states_skipped_by_ignore,
             gpu_cards_taken_from,
+            unit_checksum_buffers_booked_mebibytes: None,
+            facts_verifier_buffers_booked_mebibytes: None,
         });
     }
     // 第 ① 段：登记了 crash_judge_gpu 就由 GPU 判器判这台机器领的块（判不了就报错，不退回 CPU）；没登记由 CPU 判器判，
-    // 那时登记了 gpu_unit_checks 就把单元两道校验和另交一张卡对拍
+    // 那时登记了 gpu_unit_checks 就把单元两道校验和另交一张卡对拍。起过卡池的进程借显存等满了：「本次未跑」那一行已经打了，
+    // 一块都不判，下面照叫停的样子交回（文件头第 5 条）
     let judging_started = std::time::Instant::now();
-    let (checking, gpu_judge, scratch_was_not_booked_in_time) = match (routes.crash_judge, &cards) {
-        (ComputedOn::Gpu, Some(cards)) => match judge_my_blocks_on_gpu(
+    let (
+        checking,
+        gpu_judge,
+        unit_checksum_buffers_booked_mebibytes,
+        memory_was_not_booked_in_time,
+    ) = if no_card_of_the_pool_was_registered_in_time {
+        (CheckingTally::default(), None, None, true)
+    } else {
+        match (routes.crash_judge, &cards) {
+            (ComputedOn::Gpu, Some(cards)) => match judge_my_blocks_on_gpu(
+                &mut store,
+                flow,
+                &plan,
+                &|block| selected.contains(&block),
+                &version,
+                cards,
+                gpu_cards_taken_from,
+                &stop_is_requested_by_the_environment,
+            )? {
+                GpuJudgingOfMyBlocks::Judged { tally, outcome } => {
+                    eprintln!("{}", gpu_total_line(&outcome, expand_up_to, exhaustive));
+                    (tally, Some(outcome), None, false)
+                }
+                GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime => {
+                    (CheckingTally::default(), None, None, true)
+                }
+            },
+            (ComputedOn::Gpu, None) => {
+                return Err(format!(
+                    "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回 CPU 判器",
+                    GpuComputationClass::CrashJudgeGpu.name()
+                ));
+            }
+            (ComputedOn::Cpu, _) => match unit_checksums_on(
+                routes.unit_checksums,
+                cards.as_ref(),
+                gpu_cards_taken_from,
+                flow,
+            )? {
+                // 借来的缓冲活到这一段判完
+                UnitChecksumsOfThisRun::Ready {
+                    on,
+                    lease: _buffers_held_until_checked,
+                    buffers_booked_mebibytes,
+                } => (
+                    check_selected_blocks(
+                        &mut store,
+                        flow,
+                        &plan,
+                        &selected,
+                        &on,
+                        &version,
+                        &stop_is_requested_by_the_environment,
+                    )
+                    .map_err(|error| format!("核对失败：{error:?}"))?,
+                    None,
+                    buffers_booked_mebibytes,
+                    false,
+                ),
+                UnitChecksumsOfThisRun::NotRunTheBuffersWereNotBookedInTime => {
+                    (CheckingTally::default(), None, None, true)
+                }
+            },
+        }
+    };
+    let stop_was_requested =
+        memory_was_not_booked_in_time || stop_is_requested_by_the_environment();
+    let seconds_judging = judging_started.elapsed().as_secs_f64();
+    // 第 ② 段：配置开了 GPU 就核，登记了 crash_verify_gpu 在一张卡上核（起过卡池的照账借第一张放得下的，第 ① 段的草稿区已经放掉）、
+    // 没登记用 CPU 参照核；没开 GPU 而 SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu 也用 CPU 参照核；被叫停的不核
+    let verifying_started = std::time::Instant::now();
+    let facts_verifier_route = if stop_was_requested {
+        None
+    } else if mode.uses_gpu()
+        || std::env::var("SINGLEFS_CRASH_AMPLIFICATION_VERIFY").is_ok_and(|value| value == "cpu")
+    {
+        Some(routes.facts_verifier)
+    } else {
+        None
+    };
+    let (
+        verification,
+        verifying_cards_used,
+        facts_verifier_buffers_booked_mebibytes,
+        facts_were_not_verified_in_time,
+    ) = match (facts_verifier_route, &cards) {
+        (Some(ComputedOn::Gpu), Some(cards)) => match verify_my_blocks_on_the_first_card(
             &mut store,
             flow,
             &plan,
-            &|block| selected.contains(&block),
-            &version,
+            &my_blocks,
             cards,
             gpu_cards_taken_from,
-            &stop_is_requested_by_the_environment,
         )? {
-            GpuJudgingOfMyBlocks::Judged { tally, outcome } => {
-                eprintln!("{}", gpu_total_line(&outcome, expand_up_to, exhaustive));
-                (tally, Some(outcome), false)
-            }
-            // 起过卡池的进程借草稿区等满了：「本次未跑」那一行已经打了，下面照叫停的样子交回（第 10v 批）
-            GpuJudgingOfMyBlocks::NotRunTheScratchWasNotBookedInTime => {
-                (CheckingTally::default(), None, true)
-            }
+            FactsVerifiedOnACard::Verified {
+                outcome,
+                buffers_booked_mebibytes,
+            } => (Some(outcome), 1, buffers_booked_mebibytes, false),
+            FactsVerifiedOnACard::NotRunTheBuffersWereNotBookedInTime => (None, 0, None, true),
         },
-        (ComputedOn::Gpu, None) => {
+        (Some(ComputedOn::Gpu), None) => {
             return Err(format!(
-                "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回 CPU 判器",
-                GpuComputationClass::CrashJudgeGpu.name()
+                "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回 CPU 参照核",
+                GpuComputationClass::CrashVerifyGpu.name()
             ));
         }
-        (ComputedOn::Cpu, _) => (
-            check_selected_blocks(
+        (None, _) => (None, 0, None, false),
+        (Some(ComputedOn::Cpu), _) => {
+            let on = FactsVerifierOn::Cpu;
+            let tally = verify_blocks_from_facts(
                 &mut store,
                 flow,
                 &plan,
-                &selected,
-                &unit_checksums_on(routes.unit_checksums, cards.as_ref())?,
-                &version,
-                &stop_is_requested_by_the_environment,
+                &|block| selected.contains(&block),
+                &on,
+            )?;
+            (
+                Some(VerificationOutcome {
+                    verifier: on.name(),
+                    tally,
+                }),
+                0,
+                None,
+                false,
             )
-            .map_err(|error| format!("核对失败：{error:?}"))?,
-            None,
-            false,
-        ),
+        }
     };
-    if scratch_was_not_booked_in_time || stop_is_requested_by_the_environment() {
-        // 被叫停，或起过卡池的进程借草稿区等满了（本次未跑）：判过的块已经在库里，第 ②③ 段留给接着判的那一趟
+    if stop_was_requested || facts_were_not_verified_in_time {
+        // 被叫停，或起过卡池的进程借显存等满了（本次未跑）：判过的块已经在库里，没做的段留给接着判的那一趟
         let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &version)
             .map_err(|error| format!("对账失败：{error:?}"))?
             .len();
@@ -4299,48 +4655,10 @@ pub fn run_from_environment(
             ignored_points,
             states_skipped_by_ignore,
             gpu_cards_taken_from,
+            unit_checksum_buffers_booked_mebibytes,
+            facts_verifier_buffers_booked_mebibytes,
         });
     }
-    let seconds_judging = judging_started.elapsed().as_secs_f64();
-    // 第 ② 段：配置开了 GPU 就核，登记了 crash_verify_gpu 在优先级最高的那张卡上核（第 ① 段的草稿区已经放掉）、没登记用 CPU 参照核；
-    // 没开 GPU 而 SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu 也用 CPU 参照核
-    let verifying_started = std::time::Instant::now();
-    let facts_verifier_route = if mode.uses_gpu()
-        || std::env::var("SINGLEFS_CRASH_AMPLIFICATION_VERIFY").is_ok_and(|value| value == "cpu")
-    {
-        Some(routes.facts_verifier)
-    } else {
-        None
-    };
-    let (verification, verifying_cards_used) = match (facts_verifier_route, &cards) {
-        (Some(ComputedOn::Gpu), Some(cards)) => {
-            verify_my_blocks_on_the_first_card(&mut store, flow, &plan, &my_blocks, cards)?
-        }
-        (Some(ComputedOn::Gpu), None) => {
-            return Err(format!(
-                "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回 CPU 参照核",
-                GpuComputationClass::CrashVerifyGpu.name()
-            ));
-        }
-        (None, _) => (None, 0),
-        (Some(ComputedOn::Cpu), _) => {
-            let on = FactsVerifierOn::Cpu;
-            let tally = verify_blocks_from_facts(
-                &mut store,
-                flow,
-                &plan,
-                &|block| selected.contains(&block),
-                &on,
-            )?;
-            (
-                Some(VerificationOutcome {
-                    verifier: on.name(),
-                    tally,
-                }),
-                0,
-            )
-        }
-    };
     // 用了几张卡：第 ① 段在卡上判的按它用的，第 ② 段在卡上核、单元校验和在卡上对拍的各一张
     let cards_used = [
         gpu_judge.as_ref().map_or(0, |outcome| outcome.cards_used),
@@ -4413,6 +4731,8 @@ pub fn run_from_environment(
         ignored_points,
         states_skipped_by_ignore,
         gpu_cards_taken_from,
+        unit_checksum_buffers_booked_mebibytes,
+        facts_verifier_buffers_booked_mebibytes,
     })
 }
 
@@ -4459,25 +4779,60 @@ type CardsInUse = Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>>;
 #[cfg(not(feature = "gpu"))]
 type CardsInUse = ();
 
+/// 起过卡池的进程为第 ② 段或单元校验和对拍在显存账上借的那一笔：活到那一段算完；卡不是从池里来的不借（`None`）。
+/// 没编进 `gpu` 特性时没有这个东西。
+#[cfg(feature = "gpu")]
+type BufferLeaseOnTheCardPool = Option<crate::gpu_unit_checks::gpu_test_start::GpuCardLease>;
+#[cfg(not(feature = "gpu"))]
+type BufferLeaseOnTheCardPool = ();
+
+/// 这一趟的卡（[`gpu_cards_in_use`]）。
+#[cfg_attr(
+    not(feature = "gpu"),
+    allow(
+        dead_code,
+        reason = "没编进 gpu 特性时取卡那一份只报错，两种结局只在编进来时造"
+    )
+)]
+enum GpuCardsOfThisRun {
+    Taken {
+        cards: CardsInUse,
+        taken_from: GpuCardsSource,
+    },
+    /// 起过卡池的进程在池里一张卡都登记不上、等满了：「本次未跑」那一行已经打了（文件头第 5 条）。
+    NotRunNoCardOfThePoolHadRoomInTime,
+}
+
 /// 这个进程用哪几张卡：配置点了名的只用这台机器那几张、照配置的次序与额度；没配就是本机能用的卡按空闲显存从大到小。
 /// 这个进程起过 GPU 用例共用的卡池（[`crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process`]）就照池里的卡用
-/// （[`cards_from_the_card_pool_of_this_process`]），不另起一套设备；没起过才自己起。
+/// （池里的卡要与配置对得上，[`the_card_pool_matches_the_configured_cards`]），不另起一套设备；池里此刻登记不上这个进程的卡这一趟不用，
+/// 一张都登记不上才等，等满了打「本次未跑」那一行、交 [`GpuCardsOfThisRun::NotRunNoCardOfThePoolHadRoomInTime`]。没起过池才自己起。
 #[cfg(feature = "gpu")]
 fn gpu_cards_in_use(
     configured: Option<&[CardByPriority]>,
     host: CardHost,
-) -> Result<(CardsInUse, GpuCardsSource), String> {
+) -> Result<GpuCardsOfThisRun, String> {
+    use crate::gpu_unit_checks::gpu_test_start::CardsRegisteredForTheProductPath;
     if let Some(pool) = crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process()
     {
-        let cards = cards_from_the_card_pool_of_this_process(
-            pool.cards_registered_and_started_in_priority_order()?,
+        the_card_pool_matches_the_configured_cards(
+            &pool.cards_in_priority_order_as_configured(),
             configured,
             host,
         )?;
-        return Ok((
-            cards.into_iter().map(std::sync::Arc::new).collect(),
-            GpuCardsSource::TheCardPoolOfThisProcess,
-        ));
+        return match pool.cards_registered_waiting_only_when_none_has_room()? {
+            CardsRegisteredForTheProductPath::Started(cards) => Ok(GpuCardsOfThisRun::Taken {
+                cards: cards.into_iter().map(std::sync::Arc::new).collect(),
+                taken_from: GpuCardsSource::TheCardPoolOfThisProcess,
+            }),
+            CardsRegisteredForTheProductPath::NoCardHadRoomBeforeTheLongestWait { reason } => {
+                println!(
+                    "{}",
+                    gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
+                );
+                Ok(GpuCardsOfThisRun::NotRunNoCardOfThePoolHadRoomInTime)
+            }
+        };
     }
     let cards = match configured {
         Some(configured) => {
@@ -4512,61 +4867,116 @@ fn gpu_cards_in_use(
                 .to_string(),
         );
     }
-    Ok((
-        cards.into_iter().map(std::sync::Arc::new).collect(),
-        GpuCardsSource::DevicesStartedForThisRun,
-    ))
+    Ok(GpuCardsOfThisRun::Taken {
+        cards: cards.into_iter().map(std::sync::Arc::new).collect(),
+        taken_from: GpuCardsSource::DevicesStartedForThisRun,
+    })
 }
 
-/// 照池里的卡（`pool_cards`，按池的优先级）给这一趟用：配置点了名的（`configured` 里这台机器那几项），池里的卡要与它逐张同序号、同额度，
-/// 对不上就报错（池与这一趟读到的显卡配置不同，不悄悄换卡）；没配就照池里的卡与它们的额度用。
+/// 池里的卡（`pool_cards`：按池的优先级，每张的序号与配置给它的额度）对不对得上这一趟读到的显卡配置：配置点了名的（`configured` 里
+/// 这台机器那几项），池里的卡要与它逐张同序号、同额度，对不上就报错（池与这一趟读到的显卡配置不同，不悄悄换卡）；没配就照池里的卡用。
+///
+/// # Errors
+/// 两份显卡配置对不上。
 #[cfg(feature = "gpu")]
-fn cards_from_the_card_pool_of_this_process(
-    pool_cards: Vec<crate::gpu_unit_checks::GpuCard>,
+fn the_card_pool_matches_the_configured_cards(
+    pool_cards: &[(u32, Option<u64>)],
     configured: Option<&[CardByPriority]>,
     host: CardHost,
-) -> Result<Vec<crate::gpu_unit_checks::GpuCard>, String> {
+) -> Result<(), String> {
     if let Some(configured) = configured {
         let mine: Vec<(u32, Option<u64>)> = configured
             .iter()
             .filter(|card| card.host == host)
             .map(|card| (card.index, Some(card.memory_quota_mebibytes)))
             .collect();
-        let in_the_pool: Vec<(u32, Option<u64>)> = pool_cards
-            .iter()
-            .map(|card| (card.index, card.memory_quota_mebibytes))
-            .collect();
-        if mine != in_the_pool {
+        if mine != pool_cards {
             return Err(format!(
-                "这个进程的卡池里是 {in_the_pool:?}（序号，额度 MiB），这一趟的显卡配置给 {} 这台机器点的是 {mine:?}：两份显卡配置对不上，\
+                "这个进程的卡池里是 {pool_cards:?}（序号，额度 MiB），这一趟的显卡配置给 {} 这台机器点的是 {mine:?}：两份显卡配置对不上，\
                  不另起一套设备、也不悄悄换卡",
                 host.name()
             ));
         }
     }
-    Ok(pool_cards)
+    Ok(())
+}
+
+/// 第 ① 段由 CPU 判时，单元两道校验和在哪对拍（[`unit_checksums_on`]）。
+#[cfg_attr(
+    not(feature = "gpu"),
+    allow(
+        dead_code,
+        reason = "没编进 gpu 特性时单元校验和只在 CPU 上，借不到缓冲那一形只在编进来时造"
+    )
+)]
+enum UnitChecksumsOfThisRun {
+    /// `on` 是在哪对拍；`lease` 是起过卡池的进程为对拍的缓冲借的那一笔（活到这一段判完），`buffers_booked_mebibytes` 是借的量。
+    Ready {
+        on: UnitChecksumsOn,
+        lease: BufferLeaseOnTheCardPool,
+        buffers_booked_mebibytes: Option<u64>,
+    },
+    /// 起过卡池的进程借对拍的缓冲等满了：「本次未跑」那一行已经打了。
+    NotRunTheBuffersWereNotBookedInTime,
 }
 
-/// 第 ① 段由 CPU 判时，单元两道校验和在哪对拍：登记了 `gpu_unit_checks` 交优先级最高的那张卡，没登记只在 CPU 上。
+/// 第 ① 段由 CPU 判时，单元两道校验和在哪对拍：登记了 `gpu_unit_checks` 交一张卡，没登记只在 CPU 上。
+/// 卡是这一趟自己起的交优先级最高的那张；起过卡池的照对拍的缓冲（`unit_checksum_buffer_footprint`，这条流的单元写数、核对线程数）
+/// 在账上借第一张放得下的（此刻每张都放不下才等），用借来的那张算；等满了打「本次未跑」那一行（文件头第 5 条）。
 ///
 /// # Errors
-/// 登记了却没有起好的卡（不退回只在 CPU 上算）。
+/// 登记了却没有起好的卡（不退回只在 CPU 上算）；起过卡池的进程借缓冲时要的比每张卡的账都大、账本读写不了。
 #[cfg(feature = "gpu")]
 fn unit_checksums_on(
     route: ComputedOn,
     cards: Option<&CardsInUse>,
-) -> Result<UnitChecksumsOn, String> {
+    cards_taken_from: GpuCardsSource,
+    flow: &CrashFlow<'_>,
+) -> Result<UnitChecksumsOfThisRun, String> {
+    let ready_without_a_lease = |on: UnitChecksumsOn| UnitChecksumsOfThisRun::Ready {
+        on,
+        lease: None,
+        buffers_booked_mebibytes: None,
+    };
     match route {
-        ComputedOn::Cpu => Ok(UnitChecksumsOn::CpuOnly),
-        ComputedOn::Gpu => cards
-            .and_then(|cards| cards.first())
-            .map(|card| UnitChecksumsOn::Gpu(card.clone()))
-            .ok_or_else(|| {
+        ComputedOn::Cpu => Ok(ready_without_a_lease(UnitChecksumsOn::CpuOnly)),
+        ComputedOn::Gpu => {
+            let first_card = cards.and_then(|cards| cards.first()).ok_or_else(|| {
                 format!(
                     "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回只在 CPU 上算",
                     GpuComputationClass::GpuUnitChecks.name()
                 )
-            }),
+            })?;
+            match cards_taken_from {
+                GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => Ok(
+                    ready_without_a_lease(UnitChecksumsOn::Gpu(first_card.clone())),
+                ),
+                GpuCardsSource::TheCardPoolOfThisProcess => {
+                    let unit_writes = flow
+                        .writes
+                        .iter()
+                        .filter(|write| write.kind == StepKind::UnitWrite)
+                        .count();
+                    let need_mebibytes =
+                        unit_checksum_buffer_footprint(unit_writes, checking_threads()).mebibytes();
+                    match the_card_pool_this_run_takes_its_cards_from()
+                        .lease_the_first_card_with_room_waiting(
+                            PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
+                            need_mebibytes,
+                        ) {
+                        Ok(lease) => Ok(UnitChecksumsOfThisRun::Ready {
+                            on: UnitChecksumsOn::Gpu(std::sync::Arc::new(lease.card().clone())),
+                            buffers_booked_mebibytes: Some(lease.mebibytes()),
+                            lease: Some(lease),
+                        }),
+                        Err(refusal) => {
+                            print_the_not_run_line_or_refuse("单元校验和对拍的缓冲", &refusal)?;
+                            Ok(UnitChecksumsOfThisRun::NotRunTheBuffersWereNotBookedInTime)
+                        }
+                    }
+                }
+            }
+        }
     }
 }
 
@@ -4578,9 +4988,15 @@ fn unit_checksums_on(
 fn unit_checksums_on(
     route: ComputedOn,
     _cards: Option<&CardsInUse>,
-) -> Result<UnitChecksumsOn, String> {
+    _cards_taken_from: GpuCardsSource,
+    _flow: &CrashFlow<'_>,
+) -> Result<UnitChecksumsOfThisRun, String> {
     match route {
-        ComputedOn::Cpu => Ok(UnitChecksumsOn::CpuOnly),
+        ComputedOn::Cpu => Ok(UnitChecksumsOfThisRun::Ready {
+            on: UnitChecksumsOn::CpuOnly,
+            lease: (),
+            buffers_booked_mebibytes: None,
+        }),
         ComputedOn::Gpu => Err(format!(
             "登记了 {} 走 GPU，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）",
             GpuComputationClass::GpuUnitChecks.name()
@@ -4593,7 +5009,7 @@ fn unit_checksums_on(
 fn gpu_cards_in_use(
     _configured: Option<&[CardByPriority]>,
     _host: CardHost,
-) -> Result<(CardsInUse, GpuCardsSource), String> {
+) -> Result<GpuCardsOfThisRun, String> {
     Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
 }
 
@@ -4645,8 +5061,30 @@ fn judge_my_blocks_on_gpu(
     Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
 }
 
-/// 第 ② 段在卡上：这台机器领到的块都在优先级最高的那张卡上核（第 ① 段的草稿区那时已经放掉，这一段一块一次派活、不占草稿区）。
-/// 交回合计与用了几张卡。
+/// 第 ② 段在卡上核的结局（[`verify_my_blocks_on_the_first_card`]）。
+#[cfg_attr(
+    not(feature = "gpu"),
+    allow(
+        dead_code,
+        reason = "没编进 gpu 特性时第 ② 段在卡上的那一份只报错，两种结局只在编进来时造"
+    )
+)]
+enum FactsVerifiedOnACard {
+    /// 核完了：`buffers_booked_mebibytes` 是起过卡池的进程为核对的缓冲借的（不借是 `None`）。
+    Verified {
+        outcome: VerificationOutcome,
+        buffers_booked_mebibytes: Option<u64>,
+    },
+    /// 起过卡池的进程借核对的缓冲等满了：「本次未跑」那一行已经打了，一块都没核。
+    NotRunTheBuffersWereNotBookedInTime,
+}
+
+/// 第 ② 段在卡上：这台机器领到的块都在一张卡上核（第 ① 段的草稿区那时已经放掉，这一段一块一次派活、不占草稿区）。
+/// 卡是这一趟自己起的用优先级最高的那张；起过卡池的照这条流的事实表与一块的缓冲（`facts_verifier_buffer_footprint`）在账上借
+/// 第一张放得下的（此刻每张都放不下才等），用借来的那张核，等满了打「本次未跑」那一行（文件头第 5 条）。
+///
+/// # Errors
+/// 库读写失败；起过卡池的进程借缓冲时要的比每张卡的账都大、账本读写不了。
 #[cfg(feature = "gpu")]
 fn verify_my_blocks_on_the_first_card(
     store: &mut VerdictStore,
@@ -4654,11 +5092,34 @@ fn verify_my_blocks_on_the_first_card(
     plan: &CrashPlan,
     my_blocks: &[u64],
     cards: &CardsInUse,
-) -> Result<(Option<VerificationOutcome>, usize), String> {
-    let card = cards
-        .first()
-        .expect("用的卡至少一张（gpu_cards_in_use 判过）")
-        .clone();
+    cards_taken_from: GpuCardsSource,
+) -> Result<FactsVerifiedOnACard, String> {
+    let facts = extract_flow_facts(flow.base, flow.writes, flow.segments, flow.expansion);
+    // 借据活到这一段核完（核对的缓冲在下面那一次调用里建、丢）
+    let (card, lease): (_, BufferLeaseOnTheCardPool) = match cards_taken_from {
+        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => (
+            cards
+                .first()
+                .expect("用的卡至少一张（gpu_cards_in_use 判过）")
+                .clone(),
+            None,
+        ),
+        GpuCardsSource::TheCardPoolOfThisProcess => {
+            let need_mebibytes =
+                facts_verifier_buffer_footprint(&facts, STATES_PER_BLOCK).mebibytes();
+            match the_card_pool_this_run_takes_its_cards_from()
+                .lease_the_first_card_with_room_waiting(
+                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
+                    need_mebibytes,
+                ) {
+                Ok(lease) => (std::sync::Arc::new(lease.card().clone()), Some(lease)),
+                Err(refusal) => {
+                    print_the_not_run_line_or_refuse("第 ② 段核对的缓冲", &refusal)?;
+                    return Ok(FactsVerifiedOnACard::NotRunTheBuffersWereNotBookedInTime);
+                }
+            }
+        }
+    };
     let selected: std::collections::HashSet<u64> = my_blocks.iter().copied().collect();
     eprintln!(
         "CRASH_AMPLIFICATION_CARD name={:?} index={} memory_budget_mebibytes={} blocks={}",
@@ -4668,15 +5129,22 @@ fn verify_my_blocks_on_the_first_card(
         selected.len()
     );
     let on = FactsVerifierOn::Gpu(card);
-    let tally =
-        verify_blocks_from_facts(store, flow, plan, &|block| selected.contains(&block), &on)?;
-    Ok((
-        Some(VerificationOutcome {
+    let tally = verify_blocks_from_these_facts(
+        store,
+        plan,
+        &facts,
+        &|block| selected.contains(&block),
+        &on,
+    )?;
+    Ok(FactsVerifiedOnACard::Verified {
+        outcome: VerificationOutcome {
             verifier: "gpu",
             tally,
-        }),
-        1,
-    ))
+        },
+        buffers_booked_mebibytes: lease
+            .as_ref()
+            .map(crate::gpu_unit_checks::gpu_test_start::GpuCardLease::mebibytes),
+    })
 }
 
 /// 没编进 `gpu` 特性走不到这里（[`gpu_cards_in_use`] 先报错）。
@@ -4687,7 +5155,8 @@ fn verify_my_blocks_on_the_first_card(
     _plan: &CrashPlan,
     _my_blocks: &[u64],
     _cards: &CardsInUse,
-) -> Result<(Option<VerificationOutcome>, usize), String> {
+    _cards_taken_from: GpuCardsSource,
+) -> Result<FactsVerifiedOnACard, String> {
     Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
 }
 
@@ -4827,27 +5296,34 @@ fn print_the_placement(
                     .map_or(0, |(_, states_placed)| *states_placed)
             })
             .collect();
-        let (cards_for_the_judges, _scratch_leases) =
+        // 借据活到试判完（这一段收尾）；本机的卡照装的状态数借，此刻借不到的卡不试判、不等它；本机一张卡都没装状态就不借
+        let states_placed_on_this_host: u64 = states_placed_on_local_cards.iter().sum();
+        let scratch = if states_placed_on_this_host == 0 {
+            None
+        } else {
             match scratch_of_each_card_booked_by_where_the_cards_came_from(
                 cards_taken_from,
                 cards,
                 &tables,
                 states_of_the_flow,
+                states_placed_on_this_host,
                 &states_placed_on_local_cards,
             )? {
-                ScratchOfEachCard::Ready { cards, leases } => (cards, leases),
+                ScratchOfEachCard::Ready(scratch) => Some(scratch),
                 ScratchOfEachCard::NotRunTheScratchWasNotBookedInTime => {
                     return Ok(PlacementOfThisRun::NotRunTheScratchWasNotBookedInTime);
                 }
-            };
-        for ((row_host, index, _), states_placed) in rows.iter().zip(&placed) {
-            if *row_host != host || *states_placed == 0 {
-                continue;
             }
-            let card = cards_for_the_judges
+        };
+        for (card, states_placed) in scratch.iter().flat_map(|scratch| {
+            scratch
+                .cards_for_the_judges
                 .iter()
-                .find(|card| card.index == *index)
-                .expect("装了状态的本机卡是起好的");
+                .zip(&scratch.states_placed)
+        }) {
+            if *states_placed == 0 {
+                continue;
+            }
             let judge = GpuCrashJudge::on_card_with_packed_tables(
                 card,
                 &tables,
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index 44f0863..e12d371 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -281,11 +281,17 @@ pub fn memory_budget_mebibytes_of(
 /// 用例向卡池借卡时按它借（`gpu_test_start::GpuCardPool::lease_card_or_report_not_run`）。
 #[must_use]
 pub fn mebibytes_crc32c_of_ranges_needs(range_count: usize) -> u64 {
+    bytes_crc32c_of_ranges_needs(range_count).div_ceil(1024 * 1024)
+}
+
+/// 同 [`mebibytes_crc32c_of_ranges_needs`]，按字节、不取整：[`GpuCard::crc32c_of_ranges`] 一次建的五个缓冲
+/// （派活参数 16 字节、每段一个 32 KiB 的槽、一个长度字、一条结果与它的回读各 8 字节）。崩溃放量第 ① 段由 CPU 判、
+/// 单元校验和交卡对拍时按它算几条线程同时派活占多少（`crate::crash_amplification` 文件头第 5 条）。
+#[must_use]
+pub fn bytes_crc32c_of_ranges_needs(range_count: usize) -> u64 {
     const PARAMETER_BYTES: usize = 16;
     let bytes = range_count * (SLOT_BYTES + 4 + 2 * RESULT_ENTRY_BYTES) + PARAMETER_BYTES;
-    u64::try_from(bytes)
-        .expect("装得进 u64")
-        .div_ceil(1024 * 1024)
+    u64::try_from(bytes).expect("装得进 u64")
 }
 
 /// 驱动把编好的 Vulkan 管线存到盘上的目录：跨轮复用的缓存放 `GATE_CROSS_RUN_TMPDIR` 下；没设就放家目录 `~/.cache/singlefs`
@@ -920,8 +926,9 @@ pub mod gpu_test_start {
 
     /// GPU 用例共用的卡池（[`card_pool::GpuCardPool`]）：只开 `gpu` 也编得进（显卡配置的读法在 `crate::gpu_card_configuration`）。
     pub use card_pool::{
-        card_pool_started_in_this_process, gpu_card_pool_or_report_not_run, GpuCardLease,
-        GpuCardPool, CARD_QUOTA_ENVIRONMENT_VARIABLE,
+        card_pool_started_in_this_process, gpu_card_pool_or_report_not_run,
+        CardsRegisteredForTheProductPath, GpuCardLease, GpuCardPool, ScratchLeaseOnACard,
+        CARD_QUOTA_ENVIRONMENT_VARIABLE,
     };
 
     mod card_pool {
@@ -943,7 +950,7 @@ pub mod gpu_test_start {
             process_is_alive_by_its_entry_in_the_process_filesystem,
             reason_for_a_booking_that_waited_past_the_longest_wait, BookingRefusal,
             GpuMemoryBooking, GpuMemoryLedger, LedgerOpeningRefusal, LedgerSettings,
-            RecordedProcess, LONGEST_WAIT_FOR_GPU_MEMORY,
+            RecordedProcess, StatesBookedOnACard, LONGEST_WAIT_FOR_GPU_MEMORY,
             POLL_INTERVAL_FOR_MEMORY_RETURNED_BY_OTHER_PROCESSES,
         };
         use singlefs_harness::numeric_environment_variable::{
@@ -998,6 +1005,20 @@ pub mod gpu_test_start {
             }
         }
 
+        /// 崩溃放量产品路径在一张卡上借到的草稿区：借来的卡与借了几个状态（在账本锁里照此刻的估计压过）。
+        pub struct ScratchLeaseOnACard {
+            pub lease: GpuCardLease,
+            pub states: u64,
+        }
+
+        /// 崩溃放量产品路径这一趟用池里哪几张卡（[`GpuCardPool::cards_registered_waiting_only_when_none_has_room`]）。
+        pub enum CardsRegisteredForTheProductPath {
+            /// 此刻登记得上的那几张（按池的优先级，至少一张）：设备起好了。
+            Started(Vec<GpuCard>),
+            /// 一张都登记不上、等满了：调用方报「本次未跑」，`reason` 是等满那一句。
+            NoCardHadRoomBeforeTheLongestWait { reason: String },
+        }
+
         impl Drop for GpuCardLease {
             fn drop(&mut self) {
                 // wgpu 丢掉的缓冲要等设备下一次收拾才还给驱动：销账之前收一次，免得账上还了、卡上还占着，下一个借的建缓冲时撞满
@@ -1134,8 +1155,8 @@ pub mod gpu_test_start {
                 Ok(self.card_lent_for_the_booking(test_name, booking))
             }
 
-            /// 借只记在第 `card_position` 张卡上（`singlefs_harness::gpu_memory_ledger::GpuMemoryLedger::book_on_the_card_waiting_for_room`）：
-            /// 崩溃放量产品路径在起过卡池的进程里按派活装到每张卡上的状态借草稿区，卡由派活定。借不到交账的结局，由调用方定怎么报
+            /// 同 [`Self::lease_card_or_report_not_run`]（照卡的优先级记在第一张放得下的卡上，此刻每张都放不下才等），借不到交账的结局、
+            /// 由调用方定怎么报：崩溃放量产品路径在起过卡池的进程里，第 ② 段核对与单元校验和对拍在卡上建的缓冲照它借
             /// （`singlefs_harness::gpu_memory_ledger::scratch_booking_refusal_outcome`：等满了报本次未跑，别的判红）。
             ///
             /// # Errors
@@ -1143,18 +1164,71 @@ pub mod gpu_test_start {
             ///
             /// # Panics
             /// 借到的卡上起不了设备。
-            pub fn lease_the_card_at_waiting_for_room(
+            pub fn lease_the_first_card_with_room_waiting(
                 &'static self,
                 run_name: &str,
-                card_position: usize,
                 need_mebibytes: u64,
             ) -> Result<GpuCardLease, BookingRefusal> {
-                let booking = self
-                    .ledger
-                    .book_on_the_card_waiting_for_room(card_position, need_mebibytes)?;
+                let booking = self.ledger.book_waiting_for_room(need_mebibytes)?;
                 Ok(self.card_lent_for_the_booking(run_name, booking))
             }
 
+            /// 序号是 `index`（`nvidia-smi` 给的）的那张卡在池里排第几。
+            ///
+            /// # Panics
+            /// 池里没有这张卡（崩溃放量产品路径的卡都取自这个池，[`Self::cards_registered_waiting_only_when_none_has_room`]）。
+            fn position_of_the_card_with_index(&self, index: u32) -> usize {
+                self.cards
+                    .iter()
+                    .position(|card| card.discovered.reported.index == index)
+                    .unwrap_or_else(|| {
+                        panic!("序号 {index} 的卡不在这个进程的卡池里：产品路径的卡都取自这个池")
+                    })
+            }
+
+            /// 崩溃放量产品路径第 ① 段借草稿区：`card_indices` 是这一趟用的卡（按优先级，都是池里的卡），每张最多装
+            /// `states_each_card_holds` 个状态；把 `pending_states` 个状态装到此刻借得到的卡上，每张卡在账本锁里照此刻的估计压状态数再借
+            /// （几个状态要多少显存照 `mebibytes_for_states`），借不到的卡不装、不等，一张都借不到才等，等满放弃
+            /// （`singlefs_harness::gpu_memory_ledger::GpuMemoryLedger::book_states_on_the_cards_waiting_only_when_none_has_room`）。
+            /// 借到的每张交成借来的卡（额度是借的量加驱动那一份，判器照它开出来的草稿区正好是借的状态数）；交回与 `card_indices` 逐张对上。
+            ///
+            /// # Errors
+            /// 见 `singlefs_harness::gpu_memory_ledger::BookingRefusal`。
+            ///
+            /// # Panics
+            /// 有一张不是池里的卡；借到的卡上起不了设备。
+            pub fn book_the_scratch_of_the_cards_waiting_only_when_none_has_room(
+                &'static self,
+                run_name: &str,
+                card_indices: &[u32],
+                pending_states: u64,
+                states_each_card_holds: &[u64],
+                mebibytes_for_states: &dyn Fn(u64) -> u64,
+            ) -> Result<Vec<Option<ScratchLeaseOnACard>>, BookingRefusal> {
+                let card_positions: Vec<usize> = card_indices
+                    .iter()
+                    .map(|index| self.position_of_the_card_with_index(*index))
+                    .collect();
+                Ok(self
+                    .ledger
+                    .book_states_on_the_cards_waiting_only_when_none_has_room(
+                        &card_positions,
+                        pending_states,
+                        states_each_card_holds,
+                        mebibytes_for_states,
+                    )?
+                    .into_iter()
+                    .map(|booked| {
+                        booked.map(
+                            |StatesBookedOnACard { states, booking }| ScratchLeaseOnACard {
+                                lease: self.card_lent_for_the_booking(run_name, booking),
+                                states,
+                            },
+                        )
+                    })
+                    .collect())
+            }
+
             /// 记上了的那一笔交成借来的卡：与池里那张共用设备，额度与空闲改成借的量加驱动那一份；打一行 `GPU_CARD_LEASE`。
             ///
             /// # Panics
@@ -1180,39 +1254,97 @@ pub mod gpu_test_start {
                 GpuCardLease { card, booking }
             }
 
-            /// 这个进程此刻看池里每张卡的账能借多少（MiB，驱动那一份已经扣掉；按池的次序）。
+            /// 这个进程此刻看 `card_indices` 这几张卡（都是池里的卡）的账各能借多少（MiB，驱动那一份已经扣掉；与 `card_indices` 逐张对上）。
             ///
             /// # Errors
             /// 账本读写不了。
-            pub fn card_capacities_mebibytes(&self) -> Result<Vec<u64>, String> {
-                self.ledger.card_capacities_mebibytes().map_err(|error| {
-                    format!("显存账本 {} 用不了：{}", error.path.display(), error.reason)
-                })
+            ///
+            /// # Panics
+            /// 有一张不是池里的卡。
+            pub fn card_capacities_mebibytes_of(
+                &self,
+                card_indices: &[u32],
+            ) -> Result<Vec<u64>, String> {
+                let capacities_in_pool_order =
+                    self.ledger.card_capacities_mebibytes().map_err(|error| {
+                        format!("显存账本 {} 用不了：{}", error.path.display(), error.reason)
+                    })?;
+                Ok(card_indices
+                    .iter()
+                    .map(|index| {
+                        capacities_in_pool_order[self.position_of_the_card_with_index(*index)]
+                    })
+                    .collect())
+            }
+
+            /// 池里每张卡（按优先级）的序号与配置给它的额度（MiB，没配显卡优先级的是 `None`），不起设备。
+            #[must_use]
+            pub fn cards_in_priority_order_as_configured(&self) -> Vec<(u32, Option<u64>)> {
+                self.cards
+                    .iter()
+                    .map(|card| {
+                        (
+                            card.discovered.reported.index,
+                            card.configured_card_quota_mebibytes,
+                        )
+                    })
+                    .collect()
             }
 
-            /// 池里每张卡（按优先级）：这个进程在每张卡的账上登记（只记驱动那一份，不借；放不下就等，等满放弃），再取起好的设备。
-            /// 给崩溃放量用（`crate::crash_amplification`）：这个进程起过卡池时，放量照池里的卡判，不另起一套设备。
-            /// 交回的卡是池里那张的克隆（额度是配置给这张卡的、空闲是开账时读的）；放量第 ① 段开的草稿区另按每张卡装的状态数借
-            /// （[`Self::lease_the_card_at_waiting_for_room`]，第 10v 批：主 agent 定照 state_count 记、等不到报本次未跑）。
+            /// 崩溃放量产品路径这一趟用池里哪几张卡（`crate::crash_amplification`：这个进程起过卡池时，放量照池里的卡判，不另起一套设备）：
+            /// 这个进程在每张卡的账上登记（只记驱动那一份，不借），每张当场试一次、不等，此刻登记不上的那几张这一趟不用；
+            /// 一张都登记不上才等，等满交 [`CardsRegisteredForTheProductPath::NoCardHadRoomBeforeTheLongestWait`]
+            /// （第 10w 批：主 agent 定几张卡一张借不到时不拿着别的卡干等）。交回的卡是池里那张的克隆（额度是配置给这张卡的、
+            /// 空闲是开账时读的），按池的优先级；放量第 ① 段开的草稿区另按每张卡装的状态数借
+            /// （[`Self::book_the_scratch_of_the_cards_waiting_only_when_none_has_room`]）。
             ///
             /// # Errors
-            /// 某张卡的账上放不下这个进程的驱动那一份（等满了、或永远放不下），账本读写不了，或卡上起不了设备。
-            pub fn cards_registered_and_started_in_priority_order(
+            /// 账本读写不了，或登记上的卡上起不了设备。
+            pub fn cards_registered_waiting_only_when_none_has_room(
                 &'static self,
-            ) -> Result<Vec<GpuCard>, String> {
-                (0..self.cards.len())
-                    .map(|card_position| {
-                        self.ledger
-                            .register_this_process_on_the_card_waiting_for_room(card_position)
-                            .map_err(|refusal| {
-                                format!(
-                                    "序号 {} 的卡：账上登记不了这个进程（{refusal:?}）",
-                                    self.cards[card_position].discovered.reported.index
-                                )
-                            })?;
-                        self.started_card(card_position).cloned()
-                    })
-                    .collect()
+            ) -> Result<CardsRegisteredForTheProductPath, String> {
+                let every_card: Vec<usize> = (0..self.cards.len()).collect();
+                let registered = match self
+                    .ledger
+                    .register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(
+                        &every_card,
+                    ) {
+                    Ok(registered) => registered,
+                    Err(BookingRefusal::WaitedPastTheLongestWait {
+                        need_mebibytes,
+                        waited,
+                        card_capacities_mebibytes,
+                    }) => {
+                        return Ok(
+                            CardsRegisteredForTheProductPath::NoCardHadRoomBeforeTheLongestWait {
+                                reason: reason_for_a_booking_that_waited_past_the_longest_wait(
+                                    need_mebibytes,
+                                    waited,
+                                    &card_capacities_mebibytes,
+                                ),
+                            },
+                        );
+                    }
+                    Err(BookingRefusal::LedgerFileUnusable(error)) => {
+                        return Err(format!(
+                            "显存账本 {} 用不了：{}",
+                            error.path.display(),
+                            error.reason
+                        ));
+                    }
+                    Err(BookingRefusal::NeedLargerThanEveryCard(refusal)) => {
+                        return Err(format!(
+                            "登记驱动那一份不借，账却判成要的比每张卡都大（{refusal:?}）：登记那一笔要的是 0"
+                        ));
+                    }
+                };
+                every_card
+                    .into_iter()
+                    .zip(registered)
+                    .filter(|(_, is_registered)| *is_registered)
+                    .map(|(card_position, _)| self.started_card(card_position).cloned())
+                    .collect::<Result<Vec<GpuCard>, String>>()
+                    .map(CardsRegisteredForTheProductPath::Started)
             }
         }
 
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
index 6ee1d3c..2404ae1 100644
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -6,10 +6,20 @@
 //! 账只拦放不下的，不限同时借几份、也不限一台机器上同时跑几个 GPU 测试进程（用户 2026-10-06 定「开到最大的并发和gpu」
 //! 「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」）。
 //! 崩溃放量产品路径在起过卡池的进程里也走这份账：第 ① 段每张装了状态的卡按装的状态数借草稿区（[`scratch_states_to_book_on_each_card`]，
-//! 只记在那张卡上，[`GpuMemoryLedger::book_on_the_card_waiting_for_room`]），等满了照 GPU 用例那一路报「本次未跑」
+//! 只记在那张卡上），等满了照 GPU 用例那一路报「本次未跑」
 //! （[`scratch_booking_refusal_outcome`]，同一个记号 [`GPU_NOT_RUN_MARKER`]）；没起过卡池的进程照旧不进账，是哪几种调用方写在
 //! `singlefs_checker_tier::crash_amplification` 的文件头。
 //!
+//! 产品路径几张卡一起借（第 10w 批，主 agent 定）：
+//! - 一张借不到不拿着别的卡干等：登记驱动那一份（[`GpuMemoryLedger::register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room`]）
+//!   与借草稿区（[`GpuMemoryLedger::book_states_on_the_cards_waiting_only_when_none_has_room`]）都按卡的先后每张当场试一次、不等；
+//!   借不到的卡不装状态，它那一份留给后面的卡（[`place_states_on_the_cards_that_book_now`]）；一张都借不到才等，等满报本次未跑。
+//! - 估计在压与借之间变小：按状态数借的那一笔在这张卡的锁里照此刻账本里的估计压状态数（[`WhatIsBookedOnOneCard`]），
+//!   调用方压过之后别的进程开账把估计改小了，这里照新的重压一次再借，不判成放不下。
+//! - 第 ② 段核对与单元校验和在卡上建的缓冲照草稿区那一套进账，量按 [`mebibytes_of_buffers_held_at_once`] 算：
+//!   上卡一次、整段都在的，加每一趟派活建、派完就丢的乘同时还活着的趟数。两样各自的字节数在
+//!   `singlefs_checker_tier::crash_amplification` 的文件头第 5 条（哪几张表、每趟几个缓冲照缓冲的大小推的，没上卡量过）。
+//!
 //! 账跨进程：每张卡一份账本文件，放在工作根下的 [`GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME`]，读写前对同名的锁文件 `flock`，
 //! 改完写到临时文件再改名换上（进程在写的半路被杀，账本还是改之前那一份）。账本里记三样：记账的进程合起来能在这张卡上占多少显存
 //! （[`memory_all_registered_processes_may_hold_mebibytes`] 的估计；那一行的键沿用 `free_memory_after_quiescing_mebibytes`，
@@ -185,6 +195,115 @@ pub fn most_states_whose_need_fits_the_capacity(
     fitting
 }
 
+/// 产品路径在一张卡上当场（不等）借草稿区的结局（[`place_states_on_the_cards_that_book_now`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ScratchAttemptOnACard {
+    /// 前面的卡已经装下了全部要判的状态（或这张卡按它的额度一个状态都带不了）：没去借。
+    NothingToPlace,
+    /// 借到了 `states` 个状态的草稿区（在锁里照此刻的估计压过，不多于要装的）。
+    Booked { states: u64 },
+    /// 此刻放不下（别的借据占着）：这张卡这一趟不装状态，不等它；`need_mebibytes` 是压过之后要借的量。
+    NoRoomNow { need_mebibytes: u64 },
+    /// 这张卡的账连一个状态的草稿区都放不下，等多久都一样。
+    CannotHoldOneState { capacity_mebibytes: u64 },
+}
+
+/// 产品路径按卡的先后把 `pending_states` 个状态装到此刻借得到的卡上：每张卡要装的是剩下的与它带得了的（`states_each_card_holds`）
+/// 里小的那个，交 `attempt_now(卡在表里的次序, 要装的状态数)` 当场借一次、不等；借到几个就从剩下的里减几个，
+/// 借不到的卡不装，它那一份留给后面的卡。装满了的卡之后剩下的状态排队等前面的卡判完（不算在任何一张卡上），
+/// 后面的卡要装的是 0，不去借。交回每张卡的结局，与 `states_each_card_holds` 逐张对上。
+///
+/// # Panics
+/// `attempt_now` 交回的借到的状态数是 0、或比要装的多。
+pub fn place_states_on_the_cards_that_book_now(
+    pending_states: u64,
+    states_each_card_holds: &[u64],
+    attempt_now: &mut dyn FnMut(usize, u64) -> ScratchAttemptOnACard,
+) -> Vec<ScratchAttemptOnACard> {
+    let mut states_left_to_place = pending_states;
+    let mut attempts = Vec::with_capacity(states_each_card_holds.len());
+    for (card_slot, states_the_card_holds) in states_each_card_holds.iter().enumerate() {
+        let states_to_place = states_left_to_place.min(*states_the_card_holds);
+        if states_to_place == 0 {
+            attempts.push(ScratchAttemptOnACard::NothingToPlace);
+            continue;
+        }
+        let attempt = attempt_now(card_slot, states_to_place);
+        match attempt {
+            ScratchAttemptOnACard::Booked { states } => {
+                assert!(
+                    states > 0 && states <= states_to_place,
+                    "第 {card_slot} 张卡要装 {states_to_place} 个状态，借到的是 {states} 个：借到的要在 1 到要装的之间"
+                );
+                states_left_to_place -= states;
+            }
+            ScratchAttemptOnACard::NothingToPlace
+            | ScratchAttemptOnACard::NoRoomNow { .. }
+            | ScratchAttemptOnACard::CannotHoldOneState { .. } => {}
+        }
+        attempts.push(attempt);
+    }
+    attempts
+}
+
+/// 每张卡当场试过一次之后怎么办（[`decide_after_attempting_every_card`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ScratchPlacementDecision {
+    /// 至少一张卡借到了：就在借到的那几张上判，借不到的不等。
+    JudgeOnTheCardsBooked,
+    /// 一张都没借到，而至少一张卡空着时放得下：等别的借据还（`need_mebibytes` 是第一张放不下的卡要借的量）。
+    WaitForRoom { need_mebibytes: u64 },
+    /// 一张都没借到，每张卡的账连一个状态都放不下：等多久都一样，调用方判红。
+    NoCardHoldsOneState { largest_capacity_mebibytes: u64 },
+}
+
+/// 一张借到了就判；一张都没借到时，有放不下而空着时放得下的就等，没有就判成没有一张卡装得下一个状态。
+#[must_use]
+pub fn decide_after_attempting_every_card(
+    attempts: &[ScratchAttemptOnACard],
+) -> ScratchPlacementDecision {
+    let mut first_need_without_room = None;
+    let mut largest_capacity_mebibytes = 0u64;
+    for attempt in attempts {
+        match attempt {
+            ScratchAttemptOnACard::Booked { .. } => {
+                return ScratchPlacementDecision::JudgeOnTheCardsBooked;
+            }
+            ScratchAttemptOnACard::NoRoomNow { need_mebibytes } => {
+                first_need_without_room = first_need_without_room.or(Some(*need_mebibytes));
+            }
+            ScratchAttemptOnACard::CannotHoldOneState { capacity_mebibytes } => {
+                largest_capacity_mebibytes = largest_capacity_mebibytes.max(*capacity_mebibytes);
+            }
+            ScratchAttemptOnACard::NothingToPlace => {}
+        }
+    }
+    match first_need_without_room {
+        Some(need_mebibytes) => ScratchPlacementDecision::WaitForRoom { need_mebibytes },
+        None => ScratchPlacementDecision::NoCardHoldsOneState {
+            largest_capacity_mebibytes,
+        },
+    }
+}
+
+/// 一条线程上同时还活着几趟派活的缓冲：wgpu 丢掉的缓冲要等设备下一次收拾才还给驱动，这一趟的缓冲建起来时
+/// 上一趟丢掉的可能还占着（推的，没上卡量过）。
+pub const PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD: u64 = 2;
+
+/// 一段在卡上建的缓冲合起来占多少显存（MiB，向上取整）：上卡一次、整段都在的 `bytes_uploaded_once`，
+/// 加每一趟派活建、派完就丢的 `bytes_of_one_pass` 乘同时还活着的趟数 `passes_alive_at_once`
+/// （几条线程同时派活时是线程数乘 [`PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD`]）。
+#[must_use]
+pub fn mebibytes_of_buffers_held_at_once(
+    bytes_uploaded_once: u64,
+    bytes_of_one_pass: u64,
+    passes_alive_at_once: u64,
+) -> u64 {
+    bytes_uploaded_once
+        .saturating_add(bytes_of_one_pass.saturating_mul(passes_alive_at_once))
+        .div_ceil(1024 * 1024)
+}
+
 /// 崩溃放量产品路径（`singlefs_checker_tier::crash_amplification`）一趟用的卡从哪来；草稿区进不进显存账也照它
 /// （[`scratch_states_to_book_on_each_card`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -598,6 +717,59 @@ enum WhatIsBooked {
     DriverShareOnly,
 }
 
+/// 在一张卡的锁里记什么：照调用方给的量记，或按状态数借、在锁里照此刻的估计压过再记。
+#[derive(Clone, Copy)]
+enum WhatIsBookedOnOneCard<'need> {
+    AsGiven(WhatIsBooked),
+    /// 借 `want_states` 个状态的草稿区（第 ① 段的判器），`mebibytes_for_states` 是几个状态要多少显存（随状态数单调不减）：
+    /// 在锁里照此刻这张卡看这个进程的容量压（[`most_states_whose_need_fits_the_capacity`]），压出来的状态数要的放得下才记。
+    StatesCompressedToTheCapacity {
+        want_states: u64,
+        mebibytes_for_states: &'need dyn Fn(u64) -> u64,
+    },
+}
+
+impl WhatIsBookedOnOneCard<'_> {
+    /// 照这张卡此刻看这个进程的容量，这一笔要记多少（MiB）、按状态数借的借几个状态。一个状态都放不下时照一个状态算：
+    /// 要的比容量大，判成永远放不下。
+    fn need_within_the_capacity(self, capacity_mebibytes: u64) -> (u64, Option<u64>) {
+        match self {
+            Self::AsGiven(WhatIsBooked::LeaseOf { need_mebibytes }) => (need_mebibytes, None),
+            Self::AsGiven(WhatIsBooked::DriverShareOnly) => (0, None),
+            Self::StatesCompressedToTheCapacity {
+                want_states,
+                mebibytes_for_states,
+            } => {
+                let states_that_fit = most_states_whose_need_fits_the_capacity(
+                    usize::try_from(want_states).unwrap_or(usize::MAX),
+                    capacity_mebibytes,
+                    &|states| {
+                        mebibytes_for_states(u64::try_from(states).expect("状态数装得进 u64"))
+                    },
+                );
+                let states_to_book = u64::try_from(states_that_fit)
+                    .expect("压出来的不多于要借的，装得进 u64")
+                    .max(1);
+                (mebibytes_for_states(states_to_book), Some(states_to_book))
+            }
+        }
+    }
+}
+
+/// 一张卡此刻（在它的锁里）记得记不上（[`GpuMemoryLedger::book_on_the_card_if_it_still_fits`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum CardBookingNow {
+    Booked(BookedOnACard),
+    /// 别的借据占着，此刻放不下。
+    NoRoomNow {
+        need_mebibytes: u64,
+    },
+    /// 要的比这张卡看这个进程的容量大，等多久都放不下。
+    NeverFits {
+        capacity_mebibytes: u64,
+    },
+}
+
 /// 记在哪几张卡里挑。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum WhichCards {
@@ -607,11 +779,14 @@ enum WhichCards {
     OnlyTheCardAt { card_position: usize },
 }
 
-/// 记上的那一笔：在账里第几张卡、借据号（只登记驱动那一份的没有）、记的那一刻这张卡看这个进程的容量。
+/// 记上的那一笔：在账里第几张卡、借据号（只登记驱动那一份的没有）、记了多少（MiB）、按状态数借的借了几个状态、
+/// 记的那一刻这张卡看这个进程的容量。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct BookedOnACard {
     card_position: usize,
     lease_identifier: Option<u64>,
+    mebibytes_booked: u64,
+    states_booked: Option<u64>,
     capacity_mebibytes: u64,
 }
 
@@ -815,16 +990,13 @@ impl GpuMemoryLedger {
     }
 
     /// 锁住这张卡的账本重读一遍，此刻还放得下就记上（这个进程在这张卡上还没登记的一并登记），交回记上的那一笔；
-    /// 读完到锁上之间别的进程记了一笔、放不下了，交 `None`。
+    /// 读完到锁上之间别的进程记了一笔、放不下了，交 [`CardBookingNow::NoRoomNow`]；要的比此刻的容量大，交
+    /// [`CardBookingNow::NeverFits`]。按状态数借的照锁里此刻的估计压（[`WhatIsBookedOnOneCard::need_within_the_capacity`]）。
     fn book_on_the_card_if_it_still_fits(
         &self,
         card_position: usize,
-        what: WhatIsBooked,
-    ) -> Result<Option<BookedOnACard>, LedgerFileError> {
-        let need_mebibytes = match what {
-            WhatIsBooked::LeaseOf { need_mebibytes } => need_mebibytes,
-            WhatIsBooked::DriverShareOnly => 0,
-        };
+        what: WhatIsBookedOnOneCard<'_>,
+    ) -> Result<CardBookingNow, LedgerFileError> {
         with_the_card_ledger_locked(
             &self.settings,
             &self.card_file_stems[card_position],
@@ -835,6 +1007,8 @@ impl GpuMemoryLedger {
                     &self.capacity_for_this_process,
                     card_position,
                 );
+                let (need_mebibytes, states_booked) =
+                    what.need_within_the_capacity(seen.capacity_mebibytes);
                 match decide_booking_by_priority(
                     &[seen.capacity_mebibytes],
                     &[seen.booked_mebibytes],
@@ -842,7 +1016,8 @@ impl GpuMemoryLedger {
                 ) {
                     BookingDecision::BookOnTheCardAt { .. } => {
                         let lease_identifier = match what {
-                            WhatIsBooked::LeaseOf { .. } => {
+                            WhatIsBookedOnOneCard::AsGiven(WhatIsBooked::LeaseOf { .. })
+                            | WhatIsBookedOnOneCard::StatesCompressedToTheCapacity { .. } => {
                                 let lease_identifier =
                                     self.next_lease_identifier.fetch_add(1, Ordering::Relaxed);
                                 records.leases.push(RecordedLease {
@@ -852,7 +1027,7 @@ impl GpuMemoryLedger {
                                 });
                                 Some(lease_identifier)
                             }
-                            WhatIsBooked::DriverShareOnly => None,
+                            WhatIsBookedOnOneCard::AsGiven(WhatIsBooked::DriverShareOnly) => None,
                         };
                         if !records
                             .registered_processes
@@ -862,19 +1037,258 @@ impl GpuMemoryLedger {
                                 .registered_processes
                                 .push(self.settings.this_process);
                         }
-                        Some(BookedOnACard {
+                        CardBookingNow::Booked(BookedOnACard {
                             card_position,
                             lease_identifier,
+                            mebibytes_booked: need_mebibytes,
+                            states_booked,
                             capacity_mebibytes: seen.capacity_mebibytes,
                         })
                     }
-                    BookingDecision::WaitForMemoryToBeReturned
-                    | BookingDecision::NeverFits { .. } => None,
+                    BookingDecision::WaitForMemoryToBeReturned => {
+                        CardBookingNow::NoRoomNow { need_mebibytes }
+                    }
+                    BookingDecision::NeverFits {
+                        largest_capacity_mebibytes,
+                    } => CardBookingNow::NeverFits {
+                        capacity_mebibytes: largest_capacity_mebibytes,
+                    },
                 }
             },
         )
     }
 
+    /// 计数器上只做加一，持锁的那一段不会 panic 到一半：别的线程 panic 留下的毒不影响它，照用。
+    fn returns_seen_in_this_process(&self) -> u64 {
+        *self
+            .returns_in_this_process
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+    }
+
+    /// 睡到这个进程里有一笔还回来（还的当场叫醒）、或到重读账本的间隔（别的进程还的叫不醒这里），不越过最长的等待；
+    /// `returns_seen` 是判之前看到的还过几次：判完到睡下之间有人还了就不睡。
+    fn sleep_until_a_return_or_the_poll_interval(&self, returns_seen: u64, waited: Duration) {
+        let returns = self
+            .returns_in_this_process
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        if *returns == returns_seen {
+            let sleep = self
+                .settings
+                .poll_interval_for_memory_returned_by_other_processes
+                .min(self.settings.longest_wait.saturating_sub(waited));
+            drop(
+                self.memory_returned
+                    .wait_timeout(returns, sleep)
+                    .unwrap_or_else(PoisonError::into_inner),
+            );
+        }
+    }
+
+    /// 几张卡一起借的那几条路等一轮：从 `started` 起已经等满最长的等待交 `Err(等了多久)`，没等满就睡一下
+    /// （[`Self::sleep_until_a_return_or_the_poll_interval`]）。
+    fn wait_once_more_unless_the_longest_wait_is_over(
+        &self,
+        started: Instant,
+        returns_seen: u64,
+    ) -> Result<(), Duration> {
+        let waited_so_far = started.elapsed();
+        if self.settings.longest_wait <= waited_so_far {
+            return Err(waited_so_far);
+        }
+        self.sleep_until_a_return_or_the_poll_interval(returns_seen, waited_so_far);
+        Ok(())
+    }
+
+    /// 等满了还借不到：带上这几张卡此刻看这个进程的容量。
+    fn refusal_after_the_longest_wait(
+        &self,
+        card_positions: &[usize],
+        need_mebibytes: u64,
+        waited: Duration,
+    ) -> BookingRefusal {
+        match self.cards_seen_by_this_process(card_positions) {
+            Ok(cards) => BookingRefusal::WaitedPastTheLongestWait {
+                need_mebibytes,
+                waited,
+                card_capacities_mebibytes: cards
+                    .iter()
+                    .map(|card| card.capacity_mebibytes)
+                    .collect(),
+            },
+            Err(error) => BookingRefusal::LedgerFileUnusable(error),
+        }
+    }
+
+    /// 记上的一笔借据做成借据（丢掉时销账）。
+    ///
+    /// # Panics
+    /// 记的不是借据（只登记驱动那一份的没有借据号）。
+    fn booking_of(&self, booked: BookedOnACard) -> GpuMemoryBooking<'_> {
+        GpuMemoryBooking {
+            ledger: self,
+            card_position: booked.card_position,
+            mebibytes: booked.mebibytes_booked,
+            lease_identifier: booked
+                .lease_identifier
+                .expect("记的是一笔借据（借一个量或按状态数借），记上就有借据号"),
+            card_capacity_mebibytes_when_booked: booked.capacity_mebibytes,
+        }
+    }
+
+    /// 把这个进程登记到 `card_positions`（次序就是优先级）里此刻放得下它的驱动那一份的每张卡上（已经登记了的照算），每张当场试一次、不等；
+    /// 一张都登记不上才等别的借据还，等满放弃（第 10w 批：主 agent 定几张卡一张借不到时不拿着别的卡干等）。
+    /// 交回每张卡登记了没有（与 `card_positions` 逐张对上，至少一张登记了）；调用方只在登记了的卡上起设备。
+    ///
+    /// # Errors
+    /// 一张都登记不上而等满了、账本读写不了（见 [`BookingRefusal`]）。
+    ///
+    /// # Panics
+    /// `card_positions` 是空的。
+    pub fn register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(
+        &self,
+        card_positions: &[usize],
+    ) -> Result<Vec<bool>, BookingRefusal> {
+        assert!(
+            !card_positions.is_empty(),
+            "至少给一张卡：一张卡都没有就只能等满"
+        );
+        let started = Instant::now();
+        loop {
+            let returns_seen = self.returns_seen_in_this_process();
+            let registered = card_positions
+                .iter()
+                .map(|card_position| {
+                    if self.is_this_process_registered_on_the_card(*card_position)? {
+                        return Ok(true);
+                    }
+                    match self.book_on_the_card_if_it_still_fits(
+                        *card_position,
+                        WhatIsBookedOnOneCard::AsGiven(WhatIsBooked::DriverShareOnly),
+                    )? {
+                        CardBookingNow::Booked(_) => Ok(true),
+                        CardBookingNow::NoRoomNow { .. } | CardBookingNow::NeverFits { .. } => {
+                            Ok(false)
+                        }
+                    }
+                })
+                .collect::<Result<Vec<bool>, LedgerFileError>>()
+                .map_err(BookingRefusal::LedgerFileUnusable)?;
+            if registered.contains(&true) {
+                return Ok(registered);
+            }
+            if let Err(waited) =
+                self.wait_once_more_unless_the_longest_wait_is_over(started, returns_seen)
+            {
+                return Err(self.refusal_after_the_longest_wait(
+                    card_positions,
+                    self.settings.mebibytes_each_process_keeps_on_a_card,
+                    waited,
+                ));
+            }
+        }
+    }
+
+    /// 崩溃放量产品路径第 ① 段借草稿区：按卡的先后（`card_positions`）把 `pending_states` 个状态装到此刻借得到的卡上
+    /// （[`place_states_on_the_cards_that_book_now`]，每张卡最多装 `states_each_card_holds` 个，与 `card_positions` 逐张对上），
+    /// 每张卡在它的锁里照此刻的估计压状态数再借（[`WhatIsBookedOnOneCard::StatesCompressedToTheCapacity`]，几个状态要多少显存照
+    /// `mebibytes_for_states`）；借不到的卡不装、不等。一张都没借到才等别的借据还，等满放弃。
+    /// 交回每张卡借到的（与 `card_positions` 逐张对上，没借到的是 `None`，至少一张借到了）。
+    ///
+    /// # Errors
+    /// 一张都没借到而等满了；每张卡的账连一个状态都放不下（[`BookingRefusal::NeedLargerThanEveryCard`]，要的是一个状态的量）；
+    /// 账本读写不了。出错时已经借到的都还了。
+    ///
+    /// # Panics
+    /// 两张表的卡数不同。
+    pub fn book_states_on_the_cards_waiting_only_when_none_has_room(
+        &self,
+        card_positions: &[usize],
+        pending_states: u64,
+        states_each_card_holds: &[u64],
+        mebibytes_for_states: &dyn Fn(u64) -> u64,
+    ) -> Result<Vec<Option<StatesBookedOnACard<'_>>>, BookingRefusal> {
+        assert_eq!(
+            card_positions.len(),
+            states_each_card_holds.len(),
+            "卡表与每张卡带得了几个状态的表要逐张对上"
+        );
+        let started = Instant::now();
+        loop {
+            let returns_seen = self.returns_seen_in_this_process();
+            let mut bookings: Vec<Option<StatesBookedOnACard<'_>>> =
+                card_positions.iter().map(|_| None).collect();
+            let mut ledger_error = None;
+            let attempts = place_states_on_the_cards_that_book_now(
+                pending_states,
+                states_each_card_holds,
+                &mut |card_slot, states_to_place| {
+                    if ledger_error.is_some() {
+                        // 这一轮已经作废（下面整轮交错误、借到的都还），后面的卡不再去借
+                        return ScratchAttemptOnACard::NothingToPlace;
+                    }
+                    let booked_now = self.book_on_the_card_if_it_still_fits(
+                        card_positions[card_slot],
+                        WhatIsBookedOnOneCard::StatesCompressedToTheCapacity {
+                            want_states: states_to_place,
+                            mebibytes_for_states,
+                        },
+                    );
+                    match booked_now {
+                        Ok(CardBookingNow::Booked(booked)) => {
+                            let states = booked
+                                .states_booked
+                                .expect("按状态数借的那一笔记上就带着借了几个状态");
+                            bookings[card_slot] = Some(StatesBookedOnACard {
+                                states,
+                                booking: self.booking_of(booked),
+                            });
+                            ScratchAttemptOnACard::Booked { states }
+                        }
+                        Ok(CardBookingNow::NoRoomNow { need_mebibytes }) => {
+                            ScratchAttemptOnACard::NoRoomNow { need_mebibytes }
+                        }
+                        Ok(CardBookingNow::NeverFits { capacity_mebibytes }) => {
+                            ScratchAttemptOnACard::CannotHoldOneState { capacity_mebibytes }
+                        }
+                        Err(error) => {
+                            ledger_error = Some(error);
+                            ScratchAttemptOnACard::NothingToPlace
+                        }
+                    }
+                },
+            );
+            if let Some(error) = ledger_error {
+                return Err(BookingRefusal::LedgerFileUnusable(error));
+            }
+            match decide_after_attempting_every_card(&attempts) {
+                ScratchPlacementDecision::JudgeOnTheCardsBooked => return Ok(bookings),
+                ScratchPlacementDecision::NoCardHoldsOneState {
+                    largest_capacity_mebibytes,
+                } => {
+                    return Err(BookingRefusal::NeedLargerThanEveryCard(
+                        NeedLargerThanEveryCard {
+                            need_mebibytes: mebibytes_for_states(1),
+                            largest_capacity_mebibytes,
+                        },
+                    ));
+                }
+                ScratchPlacementDecision::WaitForRoom { need_mebibytes } => {
+                    if let Err(waited) =
+                        self.wait_once_more_unless_the_longest_wait_is_over(started, returns_seen)
+                    {
+                        return Err(self.refusal_after_the_longest_wait(
+                            card_positions,
+                            need_mebibytes,
+                            waited,
+                        ));
+                    }
+                }
+            }
+        }
+    }
+
     /// 照 `which` 挑卡记 `what`：记得上就交回记上的那一笔；此刻放不下就等别的借据还（这个进程里还的当场叫醒，
     /// 别的进程还的隔 [`LedgerSettings::poll_interval_for_memory_returned_by_other_processes`] 重读一次看到），
     /// 等满 [`LedgerSettings::longest_wait`] 还放不下就放弃；永远放不下交错误、什么都不记。
@@ -893,11 +1307,7 @@ impl GpuMemoryLedger {
         };
         let started = Instant::now();
         loop {
-            // 计数器上只做加一，持锁的那一段不会 panic 到一半：别的线程 panic 留下的毒不影响它，照用
-            let returns_seen = *self
-                .returns_in_this_process
-                .lock()
-                .unwrap_or_else(PoisonError::into_inner);
+            let returns_seen = self.returns_seen_in_this_process();
             let cards = self
                 .cards_seen_by_this_process(&card_positions)
                 .map_err(BookingRefusal::LedgerFileUnusable)?;
@@ -905,11 +1315,18 @@ impl GpuMemoryLedger {
             let booked: Vec<u64> = cards.iter().map(|card| card.booked_mebibytes).collect();
             match decide_booking_by_priority(&capacities, &booked, need_mebibytes) {
                 BookingDecision::BookOnTheCardAt { card_position } => {
-                    if let Some(booked_on_the_card) = self
-                        .book_on_the_card_if_it_still_fits(card_positions[card_position], what)
+                    match self
+                        .book_on_the_card_if_it_still_fits(
+                            card_positions[card_position],
+                            WhatIsBookedOnOneCard::AsGiven(what),
+                        )
                         .map_err(BookingRefusal::LedgerFileUnusable)?
                     {
-                        return Ok(booked_on_the_card);
+                        CardBookingNow::Booked(booked_on_the_card) => {
+                            return Ok(booked_on_the_card);
+                        }
+                        // 读完到锁上之间别的进程记了一笔、或开账把估计改小了：照此刻的账重判
+                        CardBookingNow::NoRoomNow { .. } | CardBookingNow::NeverFits { .. } => {}
                     }
                 }
                 BookingDecision::WaitForMemoryToBeReturned => {
@@ -921,21 +1338,7 @@ impl GpuMemoryLedger {
                             card_capacities_mebibytes: capacities,
                         });
                     }
-                    let returns = self
-                        .returns_in_this_process
-                        .lock()
-                        .unwrap_or_else(PoisonError::into_inner);
-                    if *returns == returns_seen {
-                        let sleep = self
-                            .settings
-                            .poll_interval_for_memory_returned_by_other_processes
-                            .min(self.settings.longest_wait - waited);
-                        drop(
-                            self.memory_returned
-                                .wait_timeout(returns, sleep)
-                                .unwrap_or_else(PoisonError::into_inner),
-                        );
-                    }
+                    self.sleep_until_a_return_or_the_poll_interval(returns_seen, waited);
                 }
                 BookingDecision::NeverFits {
                     largest_capacity_mebibytes,
@@ -970,15 +1373,7 @@ impl GpuMemoryLedger {
         need_mebibytes: u64,
     ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
         let booked = self.book_waiting(which, WhatIsBooked::LeaseOf { need_mebibytes })?;
-        Ok(GpuMemoryBooking {
-            ledger: self,
-            card_position: booked.card_position,
-            mebibytes: need_mebibytes,
-            lease_identifier: booked
-                .lease_identifier
-                .expect("记的是一笔借据（WhatIsBooked::LeaseOf），记上就有借据号"),
-            card_capacity_mebibytes_when_booked: booked.capacity_mebibytes,
-        })
+        Ok(self.booking_of(booked))
     }
 
     /// 借 `need_mebibytes`，只记在第 `card_position` 张卡上（崩溃放量产品路径按每张卡装的状态借草稿区，卡由派活定、不由账挑）；
@@ -1036,6 +1431,12 @@ impl GpuMemoryLedger {
     }
 }
 
+/// 按状态数在一张卡上借到的草稿区：借了几个状态（在锁里照此刻的估计压过）与那一笔借据（丢掉时还）。
+pub struct StatesBookedOnACard<'ledger> {
+    pub states: u64,
+    pub booking: GpuMemoryBooking<'ledger>,
+}
+
 /// 账上的一笔：记在哪张卡、多少 MiB、记的那一刻这张卡看这个进程的容量；丢掉时销账、叫醒这个进程里等着借的。
 pub struct GpuMemoryBooking<'ledger> {
     ledger: &'ledger GpuMemoryLedger,
@@ -1089,15 +1490,17 @@ impl Drop for GpuMemoryBooking<'_> {
 #[cfg(test)]
 mod tests {
     use super::{
-        card_ledger_capacity_mebibytes, decide_booking_by_priority, gpu_memory_ledger_directory,
+        card_ledger_capacity_mebibytes, decide_after_attempting_every_card,
+        decide_booking_by_priority, gpu_memory_ledger_directory, mebibytes_of_buffers_held_at_once,
         memory_all_registered_processes_may_hold_mebibytes,
-        most_states_whose_need_fits_the_capacity,
+        most_states_whose_need_fits_the_capacity, place_states_on_the_cards_that_book_now,
         process_is_alive_by_its_entry_in_the_process_filesystem, scratch_booking_refusal_outcome,
         scratch_states_to_book_on_each_card,
         start_time_in_clock_ticks_from_the_process_status_line,
         states_in_one_dispatch_capped_by_the_state_count, BookingDecision, BookingRefusal,
         GpuCardsSource, GpuMemoryLedger, LedgerSettings, NeedLargerThanEveryCard, RecordedProcess,
-        ScratchBookingRefusalOutcome, ScratchStatesToBook, GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME,
+        ScratchAttemptOnACard, ScratchBookingRefusalOutcome, ScratchPlacementDecision,
+        ScratchStatesToBook, GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME,
     };
     use std::ffi::OsStr;
     use std::path::{Path, PathBuf};
@@ -1794,6 +2197,270 @@ mod tests {
         );
     }
 
+    /// 两张卡的账（容量照 5090 与 5060 Ti 的量级，不看估计）：第一张 29 000、第二张 12 000 MiB，各登记驱动那一份
+    /// `driver_share_mebibytes`；进程号 1 是别的进程、2 是这个进程，都活着。
+    fn two_card_ledger_of_a_fake_process(
+        directory: &Path,
+        process_identifier: u32,
+        driver_share_mebibytes: u64,
+        longest_wait: Duration,
+    ) -> GpuMemoryLedger {
+        GpuMemoryLedger::open_reading_the_free_memory_inside_each_card_lock(
+            settings_of_a_fake_process(
+                directory,
+                process_identifier,
+                &[1, 2],
+                driver_share_mebibytes,
+                Duration::from_millis(20),
+                longest_wait,
+            ),
+            vec!["card-first".to_string(), "card-second".to_string()],
+            &|_position| Ok(1000),
+            Box::new(|position, _estimate| match position {
+                0 => 29_000,
+                _ => 12_000,
+            }),
+        )
+        .expect("临时目录下的账本读写得了")
+    }
+
+    /// 第 10w 批「几张卡一张借不到时不拿着别的卡干等」：100 个状态、两张卡各带得了 60 个；第一张当场借到 60 个，第二张被订满。
+    /// 第二张只试一次、要装的是剩下的 40 个，借不到就不装它，判成就在第一张上判（剩下的 40 个排队等第一张），不等第二张。
+    #[test]
+    fn a_second_card_booked_full_leaves_every_state_to_the_first_card_without_waiting() {
+        let mut attempts_made = Vec::new();
+        let attempts = place_states_on_the_cards_that_book_now(
+            100,
+            &[60, 60],
+            &mut |card_slot, states_to_place| {
+                attempts_made.push((card_slot, states_to_place));
+                match card_slot {
+                    0 => ScratchAttemptOnACard::Booked {
+                        states: states_to_place,
+                    },
+                    _ => ScratchAttemptOnACard::NoRoomNow {
+                        need_mebibytes: 2_000,
+                    },
+                }
+            },
+        );
+        assert_eq!(
+            attempts_made,
+            vec![(0, 60), (1, 40)],
+            "每张卡当场试一次：第一张装 60、第二张装剩下的 40"
+        );
+        assert_eq!(
+            attempts,
+            vec![
+                ScratchAttemptOnACard::Booked { states: 60 },
+                ScratchAttemptOnACard::NoRoomNow {
+                    need_mebibytes: 2_000
+                },
+            ],
+            "第二张借不到就不装状态"
+        );
+        assert_eq!(
+            decide_after_attempting_every_card(&attempts),
+            ScratchPlacementDecision::JudgeOnTheCardsBooked,
+            "第一张借到了：就在第一张上判，不等第二张"
+        );
+    }
+
+    /// 第 10w 批：借不到的卡那一份留给后面的卡。第一张此刻借不到，100 个状态里第二张装满它带得了的 60 个；
+    /// 第一张借到的比要装的少（在锁里照变小的估计压过，60 个里借到 30 个），第二张装的是剩下的 70 个与 60 里小的那个。
+    #[test]
+    fn the_states_a_card_did_not_book_go_to_the_next_card() {
+        let first_card_without_room = place_states_on_the_cards_that_book_now(
+            100,
+            &[60, 60],
+            &mut |card_slot, states_to_place| match card_slot {
+                0 => ScratchAttemptOnACard::NoRoomNow {
+                    need_mebibytes: 3_000,
+                },
+                _ => ScratchAttemptOnACard::Booked {
+                    states: states_to_place,
+                },
+            },
+        );
+        assert_eq!(
+            first_card_without_room[1],
+            ScratchAttemptOnACard::Booked { states: 60 },
+            "第一张借不到：第二张装满它带得了的 60 个"
+        );
+        let mut second_card_asked_for = 0;
+        place_states_on_the_cards_that_book_now(
+            100,
+            &[60, 60],
+            &mut |card_slot, states_to_place| match card_slot {
+                0 => ScratchAttemptOnACard::Booked { states: 30 },
+                _ => {
+                    second_card_asked_for = states_to_place;
+                    ScratchAttemptOnACard::Booked {
+                        states: states_to_place,
+                    }
+                }
+            },
+        );
+        assert_eq!(
+            second_card_asked_for, 60,
+            "第一张只借到 30 个：剩下的 70 个里第二张装它带得了的 60 个"
+        );
+    }
+
+    /// 第 10w 批：一张都没借到时，有空着时放得下的卡就等（要的是第一张放不下的卡那一笔），每张卡连一个状态都放不下就判红。
+    #[test]
+    fn with_no_card_booked_a_card_that_fits_when_empty_means_waiting_and_none_means_red() {
+        assert_eq!(
+            decide_after_attempting_every_card(&[
+                ScratchAttemptOnACard::CannotHoldOneState {
+                    capacity_mebibytes: 40
+                },
+                ScratchAttemptOnACard::NoRoomNow {
+                    need_mebibytes: 3_000
+                },
+                ScratchAttemptOnACard::NoRoomNow {
+                    need_mebibytes: 1_000
+                },
+            ]),
+            ScratchPlacementDecision::WaitForRoom {
+                need_mebibytes: 3_000
+            },
+            "第二、三张空着时放得下：等"
+        );
+        assert_eq!(
+            decide_after_attempting_every_card(&[
+                ScratchAttemptOnACard::CannotHoldOneState {
+                    capacity_mebibytes: 40
+                },
+                ScratchAttemptOnACard::CannotHoldOneState {
+                    capacity_mebibytes: 90
+                },
+            ]),
+            ScratchPlacementDecision::NoCardHoldsOneState {
+                largest_capacity_mebibytes: 90
+            },
+            "每张都连一个状态都放不下：判红，带最大的那张的容量"
+        );
+    }
+
+    /// 第 10w 批：真账本上两张卡，第二张被别的进程订满。这个进程借 400 个状态（每个 50 MiB、每张带得了 300 个）：
+    /// 第一张借到 300 个，第二张借不到就不装，当场交回、不等（最长的等待给 30 秒，判红时才等满）。
+    #[test]
+    fn a_scratch_booking_does_not_wait_for_a_card_booked_full_when_another_card_has_room() {
+        let directory = LedgerDirectory::for_the_test("scratch-skips-a-full-card");
+        let other =
+            two_card_ledger_of_a_fake_process(&directory.path, 1, 0, Duration::from_secs(1));
+        let held = other
+            .book_on_the_card_waiting_for_room(1, 12_000)
+            .expect("第二张卡空着，别的进程借得到整份");
+        let this =
+            two_card_ledger_of_a_fake_process(&directory.path, 2, 0, Duration::from_secs(30));
+        let started = Instant::now();
+        let bookings = this
+            .book_states_on_the_cards_waiting_only_when_none_has_room(
+                &[0, 1],
+                400,
+                &[300, 300],
+                &|states| states * 50,
+            )
+            .expect("第一张空着，借得到");
+        let elapsed = started.elapsed();
+        let states_booked: Vec<Option<u64>> = bookings
+            .iter()
+            .map(|booked| booked.as_ref().map(|booked| booked.states))
+            .collect();
+        drop(bookings);
+        drop(held);
+        assert_eq!(
+            states_booked,
+            vec![Some(300), None],
+            "第一张借到 300 个，被订满的第二张不装"
+        );
+        assert!(
+            elapsed < Duration::from_secs(10),
+            "第一张借到了就不等第二张：用了 {elapsed:?}"
+        );
+    }
+
+    /// 第 10w 批「估计在压与借之间变小 ⇒ 重压后借到」：进程 100 开账时空 30 000 MiB，照它的账（30 000 − 驱动那一份）压出 274 个
+    /// 每个 100 MiB 的状态；还没借，进程 200 开账读到只空 10 000（外人占了卡），把估计改成 10 000。进程 100 借那 274 个：
+    /// 在锁里照新的估计重压成 74 个（7 400 ≤ 10 000 − 驱动那一份），借到，不判成放不下。
+    #[test]
+    fn an_estimate_lowered_between_compressing_and_booking_is_compressed_again_and_the_booking_goes_through(
+    ) {
+        let directory = LedgerDirectory::for_the_test("estimate-lowered-before-booking");
+        let need_of = |states: u64| states * 100;
+        let first = ledger_reading_the_free_memory(
+            &directory.path,
+            100,
+            &[100, 200],
+            30_000,
+            Duration::from_millis(200),
+        );
+        let capacity_when_compressed = first.card_capacities_mebibytes().expect("账本读得了")[0];
+        let states_compressed =
+            most_states_whose_need_fits_the_capacity(1_000, capacity_when_compressed, &|states| {
+                need_of(u64::try_from(states).expect("装得进 u64"))
+            });
+        assert_eq!(states_compressed, 274, "照 30 000 − 驱动那一份压出 274 个");
+        let _second = ledger_reading_the_free_memory(
+            &directory.path,
+            200,
+            &[100, 200],
+            10_000,
+            Duration::from_millis(200),
+        );
+        let bookings = first
+            .book_states_on_the_cards_waiting_only_when_none_has_room(&[0], 274, &[274], &need_of)
+            .expect("估计变小之后照新的估计重压一次，借得到");
+        let booked = bookings[0].as_ref().expect("只有这一张卡，借到了");
+        assert_eq!(booked.states, 74, "照 10 000 − 驱动那一份重压成 74 个");
+        assert_eq!(booked.booking.mebibytes(), 7_400);
+    }
+
+    /// 第 10w 批：登记驱动那一份同样不拿着别的卡干等。第二张被别的进程订满（连这个进程的驱动那一份 1 000 都放不下），
+    /// 这个进程只登记在第一张上、当场交回，不等第二张。
+    #[test]
+    fn registering_on_the_cards_does_not_wait_for_a_card_booked_full_when_another_card_has_room() {
+        let directory = LedgerDirectory::for_the_test("register-skips-a-full-card");
+        let other =
+            two_card_ledger_of_a_fake_process(&directory.path, 1, 1_000, Duration::from_secs(1));
+        let held = other
+            .book_on_the_card_waiting_for_room(1, 12_000)
+            .expect("第二张卡空着，别的进程借得到整份");
+        let this =
+            two_card_ledger_of_a_fake_process(&directory.path, 2, 1_000, Duration::from_secs(30));
+        let started = Instant::now();
+        let registered = this
+            .register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room(&[0, 1])
+            .expect("第一张空着，登记得上");
+        let elapsed = started.elapsed();
+        drop(held);
+        assert_eq!(registered, vec![true, false], "只登记在第一张上");
+        assert!(
+            elapsed < Duration::from_secs(10),
+            "第一张登记上了就不等第二张：用了 {elapsed:?}"
+        );
+    }
+
+    /// 第 10w 批：第 ② 段核对与单元校验和的缓冲进账的量。事实表 3 MiB 多 1 字节上卡一次，一趟派活建 512 KiB 多 48 字节、
+    /// 同时两趟活着：合起来 4 MiB 多 97 字节，向上取整 5 MiB；什么都不建是 0。
+    #[test]
+    fn buffers_held_at_once_are_the_uploaded_tables_plus_every_pass_still_alive_rounded_up() {
+        const MEBIBYTE: u64 = 1024 * 1024;
+        assert_eq!(
+            mebibytes_of_buffers_held_at_once(3 * MEBIBYTE + 1, MEBIBYTE / 2 + 48, 2),
+            5,
+            "3 MiB + 1 + 2 × (512 KiB + 48) = 4 MiB + 97 字节，向上取整"
+        );
+        assert_eq!(
+            mebibytes_of_buffers_held_at_once(0, MEBIBYTE, 64),
+            64,
+            "32 条线程各两趟：64 趟各 1 MiB"
+        );
+        assert_eq!(mebibytes_of_buffers_held_at_once(0, 0, 0), 0);
+    }
+
     /// 两张卡（容量照 5090 与 5060 Ti 的量级）：前一张放得下就一直记在前一张上，前一张放不下这一笔才记到下一张。
     #[test]
     fn bookings_fill_the_first_card_before_the_next_card_takes_any() {
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs b/crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs
new file mode 100644
index 0000000..551c636
--- /dev/null
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs
@@ -0,0 +1,239 @@
+//! checker 档模块：crash_amplification、gpu_unit_checks、crash_judge_dispatch
+//! 起过 GPU 用例共用的卡池的进程里，崩溃放量产品路径（`crash_amplification::run_from_environment`）在卡上建的缓冲都在跨进程的显存账上借
+//! （`crash_amplification` 文件头第 5 条）：第 ① 段判器的草稿区、第 ① 段由 CPU 判时单元校验和对拍的缓冲、第 ② 段核对的缓冲。
+//! 两条用例各在新池新建文件那条流上按环境真跑一趟，看这一趟交回的借的量：哪一样改回不借，对应的那条就红。
+//! 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」（经这个进程的卡池判，`gpu_test_start::gpu_card_pool_or_report_not_run`），
+//! 有适配器而点名的卡找不到判红；账上被别的进程订满、等满了借不到，产品路径打它自己的「本次未跑」那一行、这一趟照叫停的样子交回，
+//! 这里同样报「本次未跑」。要 `verdict-store,gpu` 两个特性，少一个这份文件是空的。
+#![cfg(all(feature = "verdict-store", feature = "gpu"))]
+
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
+mod common;
+mod common_crash_points;
+
+use std::ffi::OsStr;
+use std::path::PathBuf;
+
+use common::{build_pool, file_content, geometry};
+use singlefs_checker_tier::crash_amplification::{CrashPointSpan, EnvironmentRun, GpuCardsSource};
+use singlefs_checker_tier::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
+use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
+    gpu_card_pool_or_report_not_run, not_run_line,
+};
+use singlefs_harness::memory_pool::{
+    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
+};
+use singlefs_harness::segments::StepKind;
+
+const THIS_FILE: &str =
+    "crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs";
+/// 只展开不超过 6 次写的段：新池新建文件那条流展开的十段共几十个状态，秒级。
+const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;
+
+/// 录好的一条流：崩溃枚举要的五样加两个崩溃点。
+struct RecordedFlow {
+    base: MemoryPool,
+    writes: Vec<RetainedWrite>,
+    segments: Vec<Vec<usize>>,
+    judged_root_index: usize,
+    versions: Vec<PublishedVersion>,
+    crash_points: Vec<CrashPointSpan>,
+}
+
+/// mkfs → 取号 → 两次暖机 → 新池新建文件，按 harness 档 `build_pool` 的做法录；崩溃点按录制流里暖机与发布的分界切。
+fn record_new_pool_file_creation(tag: &str) -> RecordedFlow {
+    let pool = build_pool(tag);
+    let operations = pool.retained_operations();
+    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[pool.mkfs_operation_count..],
+        &geometry(),
+    );
+    let publish_boundary = pool.warm_up_operation_count - pool.mkfs_operation_count;
+    let first_publish_write = stream_indexes
+        .iter()
+        .position(|stream_index| *stream_index >= publish_boundary)
+        .expect("发布至少一次写");
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("发布有一条根槽写");
+    let versions = vec![PublishedVersion {
+        instance: pool.output.root.instance,
+        checkpoint_txg: pool.output.root.checkpoint_txg,
+        content: file_content(),
+    }];
+    let crash_points = vec![
+        CrashPointSpan {
+            name: "acquire_instance_and_warm_up".to_string(),
+            code_file_in_repository: THIS_FILE.to_string(),
+            writes: 0..first_publish_write,
+            ignored: false,
+        },
+        CrashPointSpan {
+            name: "publish_first_file".to_string(),
+            code_file_in_repository: THIS_FILE.to_string(),
+            writes: first_publish_write..writes.len(),
+            ignored: false,
+        },
+    ];
+    RecordedFlow {
+        base: pool.memory_pool_after_mkfs(),
+        writes,
+        segments,
+        judged_root_index,
+        versions,
+        crash_points,
+    }
+}
+
+/// 临时目录：一份开了 GPU 的多机配置（单机）与一份库，用例结束（成功、失败、panic）都删掉。
+struct GpuRunDirectory {
+    directory: PathBuf,
+    configuration: PathBuf,
+    library: PathBuf,
+}
+
+impl GpuRunDirectory {
+    fn new(tag: &str) -> Self {
+        let directory = std::env::temp_dir().join(format!(
+            "singlefs-crash-amplification-booking-{tag}-{}",
+            std::process::id()
+        ));
+        let _ = std::fs::remove_dir_all(&directory);
+        std::fs::create_dir_all(&directory).expect("建得了临时目录");
+        let configuration = directory.join("multi-host-with-gpu.env");
+        std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
+        let library = directory.join("library");
+        Self {
+            directory,
+            configuration,
+            library,
+        }
+    }
+}
+
+impl Drop for GpuRunDirectory {
+    fn drop(&mut self) {
+        let _ = std::fs::remove_dir_all(&self.directory);
+    }
+}
+
+/// 在这个进程的卡池上按环境真跑一趟：配置开 GPU、单机，`GPU_COMPUTATION_CLASSES` 照 `gpu_classes`，库是新建的空库（每个块都要判）。
+/// 本机没卡（卡池报了「本次未跑」）、或产品路径借显存等满了（这一趟照叫停的样子交回），交 `None`，调用的用例报「本次未跑」当场返回。
+fn amplify_on_the_card_pool(test_name: &str, gpu_classes: &str) -> Option<EnvironmentRun> {
+    gpu_card_pool_or_report_not_run(test_name)?;
+    let recorded = record_new_pool_file_creation(test_name);
+    let directory = GpuRunDirectory::new(test_name);
+    let environment = common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[
+        (
+            "SINGLEFS_MULTI_HOST_CONFIG",
+            directory.configuration.as_os_str(),
+        ),
+        ("GPU_COMPUTATION_CLASSES", OsStr::new(gpu_classes)),
+        (
+            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
+            directory.library.as_os_str(),
+        ),
+        (
+            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY",
+            OsStr::new("1"),
+        ),
+    ]);
+    let run = common_crash_points::amplify_prepared_flow(
+        test_name,
+        &recorded.base,
+        &recorded.writes,
+        &recorded.segments,
+        recorded.judged_root_index,
+        &recorded.versions,
+        recorded.crash_points.clone(),
+        LARGEST_EXPANDED_SEGMENT_WRITES,
+    );
+    drop(environment);
+    drop(directory);
+    assert_eq!(
+        run.gpu_cards_taken_from,
+        GpuCardsSource::TheCardPoolOfThisProcess,
+        "这个进程起过卡池：产品路径照池里的卡用"
+    );
+    if run.stopped_early {
+        println!(
+            "{}",
+            not_run_line(
+                test_name,
+                "产品路径在显存账上借缓冲等满了也借不到（上面 test=crash_amplification::run_from_environment 那一行）"
+            )
+        );
+        return None;
+    }
+    Some(run)
+}
+
+/// 第 ① 段在卡上判（登记 `crash_judge_gpu`）：每张用到的卡上，判器的草稿区都在显存账上借了，判器用的就是借来的那张
+/// （额度是借的量加驱动那一份，开出来的草稿区不超过借的）。哪天这一支改回不借，借的量是 `None`，这条红。
+#[test]
+fn the_product_path_judges_on_scratch_it_booked_on_the_card_pool() {
+    let Some(run) = amplify_on_the_card_pool(
+        "the_product_path_judges_on_scratch_it_booked_on_the_card_pool",
+        "crash_judge_gpu",
+    ) else {
+        return;
+    };
+    let gpu_judge = run
+        .gpu_judge
+        .as_ref()
+        .expect("登记了 crash_judge_gpu、有卡：第 ① 段在卡上判");
+    assert!(
+        !gpu_judge.cards.is_empty(),
+        "新建的空库里每个块都要判：至少一张卡判了"
+    );
+    for card in &gpu_judge.cards {
+        let booked = card.scratch_booked_mebibytes.unwrap_or_else(|| {
+            panic!(
+                "起过卡池的进程，序号 {} 的卡上判器的草稿区要在显存账上借：没借",
+                card.index
+            )
+        });
+        assert_eq!(
+            card.memory_budget_mebibytes,
+            booked + MEBIBYTES_LEFT_FOR_THE_DRIVER,
+            "序号 {} 的卡：判器用的是借来的那张（额度是借的量加驱动那一份）",
+            card.index
+        );
+        assert!(
+            card.scratch_mebibytes <= booked,
+            "序号 {} 的卡：判器开的草稿区 {} MiB 不超过借的 {booked} MiB",
+            card.index,
+            card.scratch_mebibytes
+        );
+    }
+}
+
+/// 第 ① 段由 CPU 判、单元校验和交卡对拍（登记 `gpu_unit_checks`），第 ② 段在卡上核（登记 `crash_verify_gpu`）：
+/// 两样在卡上建的缓冲都在显存账上借了（借的量不为 0）、第 ② 段在卡上核了。哪一样改回不借，对应的借的量是 `None`，这条红。
+#[test]
+fn the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool() {
+    let Some(run) = amplify_on_the_card_pool(
+        "the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool",
+        "gpu_unit_checks crash_verify_gpu",
+    ) else {
+        return;
+    };
+    assert!(
+        run.unit_checksum_buffers_booked_mebibytes
+            .is_some_and(|booked| booked > 0),
+        "单元校验和对拍的缓冲要在显存账上借：{:?}",
+        run.unit_checksum_buffers_booked_mebibytes
+    );
+    assert!(
+        run.facts_verifier_buffers_booked_mebibytes
+            .is_some_and(|booked| booked > 0),
+        "第 ② 段核对的缓冲要在显存账上借：{:?}",
+        run.facts_verifier_buffers_booked_mebibytes
+    );
+    assert_eq!(
+        run.verification.map(|outcome| outcome.verifier),
+        Some("gpu"),
+        "登记了 crash_verify_gpu：第 ② 段在卡上核"
+    );
+}
````

### 10w mutations-append.tsv

````text
第10w批 显存账 一张卡借到了也等借不到的那张	crates/singlefs-harness/src/gpu_memory_ledger.rs	            ScratchAttemptOnACard::Booked { .. } => {\n                return ScratchPlacementDecision::JudgeOnTheCardsBooked;\n            }	            ScratchAttemptOnACard::Booked { .. } => {}	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_second_card_booked_full_leaves_every_state_to_the_first_card_without_waiting	a_second_card_booked_full_leaves_every_state_to_the_first_card_without_waiting
第10w批 显存账 真账本上借草稿区一张借到了也等被订满的那张	crates/singlefs-harness/src/gpu_memory_ledger.rs	            ScratchAttemptOnACard::Booked { .. } => {\n                return ScratchPlacementDecision::JudgeOnTheCardsBooked;\n            }	            ScratchAttemptOnACard::Booked { .. } => {}	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_scratch_booking_does_not_wait_for_a_card_booked_full_when_another_card_has_room	a_scratch_booking_does_not_wait_for_a_card_booked_full_when_another_card_has_room
第10w批 显存账 压过的卡照要装的扣（借到的少了后面的卡也少装）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                states_left_to_place -= states;	                states_left_to_place -= states_to_place;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_states_a_card_did_not_book_go_to_the_next_card	the_states_a_card_did_not_book_go_to_the_next_card
第10w批 显存账 一张都没借到而空着时放得下的也判红（不等）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        Some(need_mebibytes) => ScratchPlacementDecision::WaitForRoom { need_mebibytes },	        Some(_) => ScratchPlacementDecision::NoCardHoldsOneState {\n            largest_capacity_mebibytes,\n        },	-p singlefs-harness --lib -- gpu_memory_ledger::tests::with_no_card_booked_a_card_that_fits_when_empty_means_waiting_and_none_means_red	with_no_card_booked_a_card_that_fits_when_empty_means_waiting_and_none_means_red
第10w批 显存账 估计在压与借之间变小也不在锁里重压	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    capacity_mebibytes,\n                    &|states| {	                    u64::MAX,\n                    &|states| {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::an_estimate_lowered_between_compressing_and_booking_is_compressed_again_and_the_booking_goes_through	an_estimate_lowered_between_compressing_and_booking_is_compressed_again_and_the_booking_goes_through
第10w批 显存账 登记驱动那一份要每张都登记上才交回（被订满的那张也等）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            if registered.contains(&true) {	            if !registered.contains(&false) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::registering_on_the_cards_does_not_wait_for_a_card_booked_full_when_another_card_has_room	registering_on_the_cards_does_not_wait_for_a_card_booked_full_when_another_card_has_room
第10w批 显存账 缓冲的量不乘同时活着的趟数	crates/singlefs-harness/src/gpu_memory_ledger.rs	        .saturating_add(bytes_of_one_pass.saturating_mul(passes_alive_at_once))	        .saturating_add(bytes_of_one_pass)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::buffers_held_at_once_are_the_uploaded_tables_plus_every_pass_still_alive_rounded_up	buffers_held_at_once_are_the_uploaded_tables_plus_every_pass_still_alive_rounded_up
第10w批 崩溃放量 起过卡池的产品路径第 ① 段草稿区改回不借	crates/singlefs-checker-tier/src/crash_amplification.rs	            cards_taken_from,\n            cards,\n            &tables,\n            state_count,\n            pending_states,	            GpuCardsSource::DevicesStartedForThisRun,\n            cards,\n            &tables,\n            state_count,\n            pending_states,	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_judges_on_scratch_it_booked_on_the_card_pool	the_product_path_judges_on_scratch_it_booked_on_the_card_pool
第10w批 崩溃放量 起过卡池的产品路径单元校验和对拍的缓冲改回不借	crates/singlefs-checker-tier/src/crash_amplification.rs	                cards.as_ref(),\n                gpu_cards_taken_from,\n                flow,	                cards.as_ref(),\n                GpuCardsSource::DevicesStartedForThisRun,\n                flow,	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool	the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool
第10w批 崩溃放量 起过卡池的产品路径第 ② 段核对的缓冲改回不借	crates/singlefs-checker-tier/src/crash_amplification.rs	            &my_blocks,\n            cards,\n            gpu_cards_taken_from,	            &my_blocks,\n            cards,\n            GpuCardsSource::DevicesStartedForThisRun,	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool	the_product_path_books_the_unit_checksum_and_facts_verifier_buffers_on_the_card_pool
第10w批 崩溃放量 第 ② 段核对的空事实表不按一个字算	crates/singlefs-checker-tier/src/crash_amplification.rs	            .map(|words| (*words).max(1) * BYTES_PER_GPU_WORD)	            .map(|words| *words * BYTES_PER_GPU_WORD)	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_facts_verifier_books_its_tables_once_and_two_blocks_of_header_verdicts_and_readback	the_facts_verifier_books_its_tables_once_and_two_blocks_of_header_verdicts_and_readback
第10w批 崩溃放量 单元校验和对拍每次单元写只算一段	crates/singlefs-checker-tier/src/crash_amplification.rs	            2 * unit_writes_in_the_flow,	            unit_writes_in_the_flow,	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::unit_checksums_book_two_ranges_per_unit_write_for_two_passes_on_every_checking_thread	unit_checksums_book_two_ranges_per_unit_write_for_two_passes_on_every_checking_thread
````

## 主 agent 合入 10u 时改的钉值与文档

````diff
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -1344,18 +1344,18 @@
     #[test]
     fn each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget() {
         for (name, source, pinned_expressions, pinned_margin) in [
-            ("recovery", recovery_kernel_source(), 319_127, 80_873),
+            ("recovery", recovery_kernel_source(), 319_137, 80_863),
             (
                 "checker-scan",
                 crate::crash_judge_gpu::checker_scan_kernel_source(),
-                380_969,
-                19_031,
+                381_545,
+                18_455,
             ),
             (
                 "checker-walk",
                 crate::crash_judge_gpu::checker_walk_kernel_source(),
-                329_198,
-                70_802,
+                329_390,
+                70_610,
             ),
         ] {
             let size = super::kernel_inlined_size(&source);
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -279,7 +279,7 @@
 /// （判决第二节 T10 乙、第四节 T15「写」：D8（核心索引结构） 已定项 6「同一条时间线上 inode 号不复用」）。
 pub const ONE_OBJECT_BIRTH_PER_INODE_NUMBER: &str = "I-9.29";
 /// I-3.14（实例表每片记录形态合法，用户 2026-10-06 定另立）：走读到的实例表每一片里，每条记录的 `kind` 只有 0（行）或 1（链指针记录）；
-/// 链指针记录恰好是这一片的最后一条（合法：有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零），片中间不出现；
+/// 链指针记录恰好是这一片的最后一条（合法：有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零且位置指针头部 MAC / nonce、算法类型、压缩算法码与压后长度、extent 偏移都是第一版的取值），片中间不出现；
 /// `kind` 0 行的 flags 字节为 0（D18（块里携带什么信息） 已定项 11：flags「第一版恒 0、非 0 拒收」）。行唯一与低于挂载根归 I-3.8。
 /// 不成立的一片解不开（用户 2026-10-06 给 D18 已定项 11 补「kind 不是 0 或 1 ⇒ 这一片解不开（表不可读）」）：主走读与只读不判的读法都不收它的行。
 pub const INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED: &str = "I-3.14";
````
