# 根环实现第 10y 批：代码轮第一轮改动全文（2026-10-07）

实现员交的 `crates.patch` 原样，打在删除线第 33 批 T 合后的主工作区上（底含删除线 S、U、T）。变异表追加、替换两份附在后面。

````diff
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -7,8 +7,11 @@
 //! 「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」）。
 //!
 //! 只记这个进程借的（用户 2026-10-07 定第 10t 批代码轮判决岔路 G1「二：去掉跨进程账」）：没有账本文件、不锁文件、
-//! 不判别的进程活没活，也不等。每一次借都在这个进程的锁里现读一次每张卡此刻的显存（[`CardMemoryReading`]：空闲、总量、
-//! 这个进程实际占着的；卡池用 `nvidia-smi` 读），照这个进程在这张卡上已经借着的算这一笔放不放得下（[`card_room_for_this_process`]）。
+//! 不判别的进程活没活，也不等。每一次借都在这个进程的锁里现读一次每张卡此刻的显存（[`CardMemoryReading`]：空闲、空卡上能用的、
+//! 这个进程实际占着的），照这个进程在这张卡上已经借着的算这一笔放不放得下（[`card_room_for_this_process`]）。
+//! 卡池每读一次跑一次 `nvidia-smi -q -d MEMORY,PIDS`（[`NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS`]）：空闲与这个进程占着的
+//! 出自同一份报告（[`card_memory_reader_of_one_report_per_reading`]，第 10x 批代码轮判决 K1-1：分两次读时，两次之间这个进程别的线程
+//! 建上卡的那一笔既在空闲里又在占着的里，算两遍）。
 //!
 //! 口径（第 10x 批，主 agent 定「三者取最小」照第 10t、10v 批）：设 F 是现读的空闲，B 是这个进程在这张卡上借着的，
 //! D 是驱动的固定开销（每个进程在每张卡上一份，起设备时付），M 是这个进程此刻能在这张卡上占的
@@ -16,15 +19,21 @@
 //! `card_ledger_capacity_mebibytes(…, M, D) − B`，展开就是下面几项里最小的：
 //! - 额度变量 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` − B（设了才有这一项）；
 //! - 配置给卡的额度 − D − B；没配显卡优先级时由 M 的 3/4 − D − B 代替（这几项都是这个进程在这张卡上一共借的连驱动那一份不超过额度）；
-//! - F − 这个进程借了还没实际占的 − D（这个进程还没在这张卡上起设备时）。「还没实际占的」是借着的连驱动那一份减实际占着的；
-//!   实际占着的读不到（`nvidia-smi` 没列这个进程）时当借着的一点都没建、起了设备的驱动那一份已经在卡上，只会少借。
+//! - F − 这个进程许出去而还没占上卡的那一份：许出去的是 B 连驱动那一份 D（还没登记时 B 是 0、D 照样要放得下），
+//!   减实际占着的里属于许出去的那部分。实际占着的先扣掉这个进程已经销账、卡上可能还没还的那一份
+//!   （[`BookedByThisProcessOnACard::returned_possibly_still_on_the_card_mebibytes`]，第 10x 批代码轮判决 K1-1：不当成别的借据已经建上的）；
+//!   读不到实际占着的（`nvidia-smi` 没列这个进程），或这个进程在这张卡上另有不走这份账的设备（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]，
+//!   分不出哪一份是借据建的），借着的当一点都没建、起了设备的驱动那一份当已经在卡上，只会少借。
 //!
-//! 借的要比这张卡空着时能借的（同一个算法拿卡的总量当 M、B 当 0）还大，等多久都放不下，调用方判红（配了 GPU 就一定用 GPU）；
+//! 借的要比这张卡空着时能借的（同一个算法拿空卡上能用的当 M、B 当 0；空卡上能用的是 `nvidia-smi` 报的总量减驱动保留的那一份，
+//! 2026-10-07 本机两张空卡上逐 MiB 等于报的空闲：5090 是 32607 − 497 = 32110，5060 Ti 是 16311 − 462 = 15849；第 10x 批代码轮判决 K1-1
+//! 「上界不用总量」）还大，等多久都放不下，调用方判红（配了 GPU 就一定用 GPU）；
 //! 比此刻能借的大而空着时放得下，是借不到：不等，调用方按借不到走下一条（几张卡一起借的换后面的卡，一张都借不到报本次未跑，
 //! [`scratch_booking_refusal_outcome`]、[`GPU_NOT_RUN_MARKER`]）。
 //! 账不防的：两个进程同一刻读到同一份空闲、一起借超了（各自只记自己借的）。这一形由运行时兜底：内存包装
 //! `scripts/hosts/run-with-memory-cap.sh` 认出内核日志里的 `NVRM … NV_ERR_NO_MEMORY` 退 245，wgpu 报 Out of Memory 时
-//! 崩溃放量产品路径把这一块报本次未跑（原因写显存不够，[`reason_for_running_out_of_gpu_memory`]）。
+//! 崩溃放量产品路径与卡池起设备把这一趟报本次未跑（原因写显存不够，[`GPU_MEMORY_SHORT_REASON_PREFIX`]、
+//! [`reason_for_running_out_of_gpu_memory`]）。
 //!
 //! 先登记、后建设备（这个进程里）：一个进程在一张卡上第一次借时，这一笔连同它的驱动那一份放得下才记、才把这张卡记成登记了；
 //! 调用方拿到借据之后才在这张卡上起设备（[`GpuMemoryLedger::check_registered_before_starting_the_device`]），
@@ -90,13 +99,25 @@
     }
 }
 
-/// 这一刻读到的一张卡的显存（MiB）。
+/// 这一刻读到的一张卡的显存（MiB）：几个量出自同一份 `nvidia-smi` 报告（[`card_memory_readings_of_one_nvidia_smi_report`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct CardMemoryReading {
     pub free_mebibytes: u64,
-    pub total_mebibytes: u64,
-    /// 这个进程此刻实际占在这张卡上的（卡池照 `nvidia-smi` 按进程号报的）；读不到（没列这个进程、读不了）是 `None`。
-    pub occupied_by_this_process_mebibytes: Option<u64>,
+    /// 空卡上能用的：报的总量减驱动保留的那一份（谁都借不走；空卡上读到的空闲就是它，文件头）。
+    pub total_less_reserved_mebibytes: u64,
+    pub occupied_by_this_process: OccupiedByThisProcess,
+}
+
+/// 这个进程此刻实际占在一张卡上的（`nvidia-smi` 按进程号报的），连同这一份能不能按这份账拆。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum OccupiedByThisProcess {
+    /// 报告里没列这个进程（没在这张卡上起设备，或驱动不按进程报），或列了而占的量读不出。
+    NotListed,
+    /// 列了：这个进程在这张卡上的设备都走这份账（卡池起的那一个）。
+    OnlyThroughTheBooks { mebibytes: u64 },
+    /// 列了，而这个进程在这张卡上另有不走这份账的设备（崩溃放量没起卡池时自己起的、经入口直接起的）：
+    /// 占的量里分不出哪一份是借据建的（第 10x 批代码轮判决 K1-1，攻方乙形）。
+    AlsoOutsideTheBooks { mebibytes: u64 },
 }
 
 /// 这个进程的驱动那一份在一张卡上走到哪一步。
@@ -110,16 +131,19 @@
     DeviceStarted,
 }
 
-/// 这个进程在一张卡上记着的：借着的合计（MiB）与驱动那一份走到哪一步。
+/// 这个进程在一张卡上记着的：借着的合计（MiB）、驱动那一份走到哪一步，与已经销账而卡上可能还没还的那一份。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct BookedByThisProcessOnACard {
     pub leases_mebibytes: u64,
     pub driver_share: DriverShareOnTheCard,
+    /// 丢掉的借据建的缓冲：wgpu 要等设备收拾才还给驱动，销了账卡上可能还占着（MiB，上界；每读一次照实际占着的往下压，
+    /// [`GpuMemoryLedger`] 的 `rooms_of_the_cards`）。实际占着的先扣掉它，才算借着的已经建上的（第 10x 批代码轮判决 K1-1，正推冲突 B）。
+    pub returned_possibly_still_on_the_card_mebibytes: u64,
 }
 
 /// 这个进程此刻能在一张卡上占多少（MiB，文件头「口径」里的 M）：现读的空闲，加上这个进程实际占着的里属于它许出去的那一部分
-/// （借着的，加登记了时驱动那一份）。实际占着的比许出去的多（分配器按块拿、别的入口起的设备）只算到许出去的那么多；
-/// 读不到实际占着的，借着的当一点都没建，起好了设备的驱动那一份当已经在卡上。
+/// （借着的，加登记了时驱动那一份）。实际占着的先扣掉已经销账、卡上可能还没还的那一份；比许出去的多（分配器按块拿）只算到许出去的那么多；
+/// 读不到实际占着的、或这个进程在这张卡上另有不走这份账的设备，借着的当一点都没建，起好了设备的驱动那一份当已经在卡上。
 #[must_use]
 pub fn memory_this_process_may_hold_mebibytes(
     reading: CardMemoryReading,
@@ -134,26 +158,56 @@
                 .saturating_add(mebibytes_left_for_the_driver)
         }
     };
-    let occupied_and_promised_mebibytes = match reading.occupied_by_this_process_mebibytes {
-        Some(occupied) => occupied.min(promised_mebibytes),
-        None => match booked.driver_share {
-            DriverShareOnTheCard::DeviceStarted => {
-                mebibytes_left_for_the_driver.min(promised_mebibytes)
+    let occupied_and_promised_mebibytes = match reading.occupied_by_this_process {
+        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => mebibytes
+            .saturating_sub(booked.returned_possibly_still_on_the_card_mebibytes)
+            .min(promised_mebibytes),
+        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {
+            match booked.driver_share {
+                DriverShareOnTheCard::DeviceStarted => {
+                    mebibytes_left_for_the_driver.min(promised_mebibytes)
+                }
+                DriverShareOnTheCard::NotRegistered
+                | DriverShareOnTheCard::RegisteredDeviceNotStarted => 0,
             }
-            DriverShareOnTheCard::NotRegistered
-            | DriverShareOnTheCard::RegisteredDeviceNotStarted => 0,
-        },
+        }
     };
     reading
         .free_mebibytes
         .saturating_add(occupied_and_promised_mebibytes)
 }
 
+/// 销了账、卡上可能还没还的那一份照这一次读数往下压（MiB）：这个进程实际占着的里，起好了设备的驱动那一份之外才可能是它，
+/// 压到不超过那么多；读不到实际占着的、或另有不走这份账的设备，照旧（分不出就不压）。
+#[must_use]
+pub fn returned_possibly_still_on_the_card_after_a_reading(
+    returned_possibly_still_on_the_card_mebibytes: u64,
+    occupied_by_this_process: OccupiedByThisProcess,
+    driver_share: DriverShareOnTheCard,
+    mebibytes_left_for_the_driver: u64,
+) -> u64 {
+    match occupied_by_this_process {
+        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => {
+            let driver_on_the_card_mebibytes = match driver_share {
+                DriverShareOnTheCard::DeviceStarted => mebibytes_left_for_the_driver,
+                DriverShareOnTheCard::NotRegistered
+                | DriverShareOnTheCard::RegisteredDeviceNotStarted => 0,
+            };
+            returned_possibly_still_on_the_card_mebibytes
+                .min(mebibytes.saturating_sub(driver_on_the_card_mebibytes))
+        }
+        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {
+            returned_possibly_still_on_the_card_mebibytes
+        }
+    }
+}
+
 /// 一张卡此刻对这个进程：这一笔最多能借多少，与这张卡空着时最多能借多少（MiB）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct CardRoomForThisProcess {
     pub room_now_mebibytes: u64,
-    /// 同一个算法拿卡的总量当这个进程能占的、借着的当 0：要的比它大，等多久都放不下。
+    /// 同一个算法拿空卡上能用的（[`CardMemoryReading::total_less_reserved_mebibytes`]）当这个进程能占的、借着的当 0：
+    /// 要的比它大，等多久都放不下。
     pub ceiling_mebibytes: u64,
 }
 
@@ -171,7 +225,9 @@
     CardRoomForThisProcess {
         room_now_mebibytes: capacity_of_memory_this_process_may_hold(may_hold_mebibytes)
             .saturating_sub(booked.leases_mebibytes),
-        ceiling_mebibytes: capacity_of_memory_this_process_may_hold(reading.total_mebibytes),
+        ceiling_mebibytes: capacity_of_memory_this_process_may_hold(
+            reading.total_less_reserved_mebibytes,
+        ),
     }
 }
 
@@ -246,8 +302,9 @@
 /// 产品路径在一张卡上当场借草稿区的结局（[`place_states_on_the_cards_that_book_now`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum ScratchAttemptOnACard {
-    /// 前面的卡已经装下了全部要判的状态（或这张卡一个状态都带不了）：没去借。
-    NothingToPlace,
+    /// 前面的卡已经装下了全部要判的状态：没去借。调用方算出这张卡一个状态都带不了的不是这一种，是下面两种之一
+    /// （第 10x 批代码轮判决 K2-2：「此刻带不了」不并进「没有要装的」，否则一张都借不到时落进判红那一支）。
+    NothingLeftToPlace,
     /// 借到了 `states` 个状态的草稿区（在锁里照此刻能借的压过，不多于要装的）。
     Booked { states: u64 },
     /// 此刻连一个状态都放不下（这个进程别的借据或卡上别的用户占着），空着时放得下：这张卡这一趟不装状态；
@@ -260,23 +317,41 @@
 /// 产品路径按卡的先后把 `pending_states` 个状态装到此刻借得到的卡上：每张卡要装的是剩下的与它带得了的（`states_each_card_holds`）
 /// 里小的那个，交 `attempt_now(卡在表里的次序, 要装的状态数)` 当场借一次；借到几个就从剩下的里减几个，
 /// 借不到的卡不装，它那一份留给后面的卡。装满了的卡之后剩下的状态排队等前面的卡判完（不算在任何一张卡上），
-/// 后面的卡要装的是 0，不去借。交回每张卡的结局，与 `states_each_card_holds` 逐张对上。
+/// 后面的卡没有要装的，不去借（[`ScratchAttemptOnACard::NothingLeftToPlace`]）。还有要装的而调用方算出这张卡一个状态都带不了，
+/// 不去借，交 `classify_a_card_holding_no_state(卡在表里的次序)` 判是此刻带不了还是空着时也带不了。
+/// 交回每张卡的结局，与 `states_each_card_holds` 逐张对上。
 ///
 /// # Panics
-/// `attempt_now` 交回的借到的状态数是 0、或比要装的多。
+/// `attempt_now` 交回的借到的状态数是 0、或比要装的多；`classify_a_card_holding_no_state` 交回的不是此刻借不到、空着时也放不下两种之一。
 pub fn place_states_on_the_cards_that_book_now(
     pending_states: u64,
     states_each_card_holds: &[u64],
     attempt_now: &mut dyn FnMut(usize, u64) -> ScratchAttemptOnACard,
+    classify_a_card_holding_no_state: &mut dyn FnMut(usize) -> ScratchAttemptOnACard,
 ) -> Vec<ScratchAttemptOnACard> {
     let mut states_left_to_place = pending_states;
     let mut attempts = Vec::with_capacity(states_each_card_holds.len());
     for (card_slot, states_the_card_holds) in states_each_card_holds.iter().enumerate() {
-        let states_to_place = states_left_to_place.min(*states_the_card_holds);
-        if states_to_place == 0 {
-            attempts.push(ScratchAttemptOnACard::NothingToPlace);
+        if states_left_to_place == 0 {
+            attempts.push(ScratchAttemptOnACard::NothingLeftToPlace);
             continue;
         }
+        if *states_the_card_holds == 0 {
+            let classified = classify_a_card_holding_no_state(card_slot);
+            match classified {
+                ScratchAttemptOnACard::NoRoomNow { .. }
+                | ScratchAttemptOnACard::CannotHoldOneState { .. } => {}
+                ScratchAttemptOnACard::NothingLeftToPlace
+                | ScratchAttemptOnACard::Booked { .. } => {
+                    panic!(
+                        "第 {card_slot} 张卡一个状态都带不了，判出来的是 {classified:?}：只能是此刻借不到或空着时也放不下"
+                    )
+                }
+            }
+            attempts.push(classified);
+            continue;
+        }
+        let states_to_place = states_left_to_place.min(*states_the_card_holds);
         let attempt = attempt_now(card_slot, states_to_place);
         match attempt {
             ScratchAttemptOnACard::Booked { states } => {
@@ -286,7 +361,7 @@
                 );
                 states_left_to_place -= states;
             }
-            ScratchAttemptOnACard::NothingToPlace
+            ScratchAttemptOnACard::NothingLeftToPlace
             | ScratchAttemptOnACard::NoRoomNow { .. }
             | ScratchAttemptOnACard::CannotHoldOneState { .. } => {}
         }
@@ -295,6 +370,23 @@
     attempts
 }
 
+/// 一张此刻连一个状态（`need_of_one_state_mebibytes`）都借不出的卡算哪一种：空着时也放不下是 [`ScratchAttemptOnACard::CannotHoldOneState`]，
+/// 空着时放得下是此刻借不到（[`ScratchAttemptOnACard::NoRoomNow`]）。
+#[must_use]
+pub fn attempt_on_a_card_without_room_for_one_state(
+    need_of_one_state_mebibytes: u64,
+    room: CardRoomForThisProcess,
+) -> ScratchAttemptOnACard {
+    if need_of_one_state_mebibytes > room.ceiling_mebibytes {
+        return ScratchAttemptOnACard::CannotHoldOneState {
+            ceiling_mebibytes: room.ceiling_mebibytes,
+        };
+    }
+    ScratchAttemptOnACard::NoRoomNow {
+        need_mebibytes: need_of_one_state_mebibytes,
+    }
+}
+
 /// 每张卡当场试过一次之后怎么办（[`decide_after_attempting_every_card`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum ScratchPlacementDecision {
@@ -324,7 +416,7 @@
             ScratchAttemptOnACard::CannotHoldOneState { ceiling_mebibytes } => {
                 largest_ceiling_mebibytes = largest_ceiling_mebibytes.max(*ceiling_mebibytes);
             }
-            ScratchAttemptOnACard::NothingToPlace => {}
+            ScratchAttemptOnACard::NothingLeftToPlace => {}
         }
     }
     match first_need_without_room {
@@ -335,6 +427,44 @@
     }
 }
 
+/// 产品路径第 ① 段借草稿区之前怎么走（[`decide_scratch_before_booking`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ScratchBeforeBooking {
+    /// 至少一张卡此刻带得了一个状态：去借。
+    BookOnTheCardsThatHoldAStateNow,
+    /// 每张卡此刻都带不了一个状态，至少一张空着时带得了：此刻借不到，调用方报本次未跑（不等、不判红）。
+    NoCardHoldsAStateNow,
+    /// 每张卡空着时也带不了一个状态（一张卡都没有也算）：等多久都一样，调用方判红。
+    NoCardHoldsAStateEvenWhenEmpty,
+}
+
+/// 照每张卡此刻带得了几个状态（`states_each_card_holds_now`）与空着时带得了几个（`states_each_card_holds_when_empty`，逐张对上）
+/// 定借草稿区之前怎么走（第 10x 批代码轮判决 K2-2：每张卡此刻都放不下一个状态而空着时放得下，是借不到，不是放不下）。
+///
+/// # Panics
+/// 两张表的卡数不同。
+#[must_use]
+pub fn decide_scratch_before_booking(
+    states_each_card_holds_now: &[u64],
+    states_each_card_holds_when_empty: &[u64],
+) -> ScratchBeforeBooking {
+    assert_eq!(
+        states_each_card_holds_now.len(),
+        states_each_card_holds_when_empty.len(),
+        "此刻与空着时两张表要逐张对上"
+    );
+    if states_each_card_holds_now.iter().any(|states| *states > 0) {
+        return ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow;
+    }
+    if states_each_card_holds_when_empty
+        .iter()
+        .any(|states| *states > 0)
+    {
+        return ScratchBeforeBooking::NoCardHoldsAStateNow;
+    }
+    ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty
+}
+
 /// 一条线程上同时还活着几趟派活的缓冲：wgpu 丢掉的缓冲要等设备下一次收拾才还给驱动，这一趟的缓冲建起来时
 /// 上一趟丢掉的可能还占着（推的，没上卡量过）。
 pub const PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD: u64 = 2;
@@ -419,6 +549,10 @@
     format!("{GPU_NOT_RUN_MARKER} test={test_name} reason={reason}")
 }
 
+/// 显存不够而报「本次未跑」时原因的开头（借不到、卡上报 Out of Memory 都是它）：分项驱动与门禁按 [`GPU_NOT_RUN_MARKER`] 认这一行，
+/// 人照这几个字认出是显存不够（第 10x 批代码轮判决 K1-3：记号照 `NOT_RUN_NO_GPU_ADAPTER` 一族，原因写显存不够）。
+pub const GPU_MEMORY_SHORT_REASON_PREFIX: &str = "显存不够：";
+
 /// 此刻借不到的那一句（GPU 用例借卡与产品路径借缓冲同一句）。
 #[must_use]
 pub fn reason_for_a_booking_without_room_now(
@@ -426,7 +560,7 @@
     card_rooms_mebibytes: &[u64],
 ) -> String {
     format!(
-        "显存不够：此刻池里没有一张卡借得出 {need_mebibytes} MiB（这个进程看每张卡此刻能借的 {card_rooms_mebibytes:?}：\
+        "{GPU_MEMORY_SHORT_REASON_PREFIX}此刻池里没有一张卡借得出 {need_mebibytes} MiB（这个进程看每张卡此刻能借的 {card_rooms_mebibytes:?}：\
          现读的空闲减这个进程借了还没占的；卡上别的 GPU 用户或这个进程里别的用例占着）"
     )
 }
@@ -436,7 +570,7 @@
 #[must_use]
 pub fn reason_for_running_out_of_gpu_memory(what_ran_out: &str) -> String {
     format!(
-        "显存不够：{what_ran_out}上 wgpu 报了 Out of Memory（账只记这个进程借的，卡上别的进程同一刻借走了；判过的块在库里）"
+        "{GPU_MEMORY_SHORT_REASON_PREFIX}{what_ran_out}上 wgpu 报了 Out of Memory（账只记这个进程借的，卡上别的进程同一刻借走了；判过的块在库里）"
     )
 }
 
@@ -492,9 +626,179 @@
     CardMemoryUnreadable { reason: String },
 }
 
-/// 读每张卡此刻的显存（次序与卡池一样）：卡池用 `nvidia-smi` 读，单测换成假的。
+/// 读每张卡此刻的显存（次序与卡池一样）：卡池每读一次跑一次 `nvidia-smi`（[`card_memory_reader_of_one_report_per_reading`]），单测换成假的。
 pub type CardMemoryReader = Box<dyn Fn() -> Result<Vec<CardMemoryReading>, String> + Send + Sync>;
 
+/// 一次读出每张卡的显存与每个进程占的那一份报告：`nvidia-smi` 带这几个参数（文本版，每张卡一段：`FB Memory Usage` 下的
+/// `Total`、`Reserved`、`Free`，`Processes` 下每个进程的 `Process ID` 与 `Used GPU Memory`）。一次调用里两样都有，空闲与这个进程占着的
+/// 是同一次读的（`--query-gpu` 与 `--query-compute-apps` 不能在一次调用里一起用，`nvidia-smi` 报「Only one --query-* switch」）。
+/// 格式照本机驱动 595.84 的 `nvidia-smi` 文本模板（`strings` 读出的 `Process ID : %u`、`Used GPU Memory : %llu MiB`）。
+pub const NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS: [&str; 3] = ["-q", "-d", "MEMORY,PIDS"];
+
+/// PCI 总线号只留 `总线:设备.功能` 那一段、小写：`00000000:01:00.0` 与 `0000:01:00.0` 认成同一张卡（卡池按它对上 wgpu 的适配器与
+/// `nvidia-smi` 报的卡）。
+#[must_use]
+pub fn pci_bus_identifier_without_the_domain(text: &str) -> String {
+    let lowered = text.trim().to_ascii_lowercase();
+    let parts: Vec<&str> = lowered.split(':').collect();
+    parts[parts.len().saturating_sub(2)..].join(":")
+}
+
+/// 一份报告里一张卡那一段读到的。
+#[derive(Default)]
+struct OneCardOfTheReport {
+    total_mebibytes: Option<u64>,
+    reserved_mebibytes: Option<u64>,
+    free_mebibytes: Option<u64>,
+    /// 这个进程那几行占的合计；列了这个进程而有一行读不出是 `Err`。
+    used_by_this_process_mebibytes: Option<Result<u64, ()>>,
+}
+
+/// 报告里一张卡那一段眼下读到哪一小节。
+#[derive(Clone, Copy, PartialEq, Eq)]
+enum SectionOfTheReport {
+    FrameBufferMemory,
+    Processes,
+    Other,
+}
+
+/// `32607 MiB` 那样的量（MiB）；读不出交 `None`。
+fn mebibytes_of_a_report_value(value: &str) -> Option<u64> {
+    value.strip_suffix("MiB")?.trim().parse().ok()
+}
+
+/// 一份 `nvidia-smi -q -d MEMORY,PIDS` 报告（[`NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS`]）里 `buses` 这几张卡此刻的显存
+/// （与 `buses` 逐张对上，`buses` 照 [`pci_bus_identifier_without_the_domain`] 写）：空闲、空卡上能用的（总量减保留），
+/// 与进程号是 `this_process_identifier` 的那几行占的合计。这个进程在一张卡上另有不走这份账的设备
+/// （`has_devices_outside_the_books_on_the_bus(总线号)` 为真）时，占的那一份记成 [`OccupiedByThisProcess::AlsoOutsideTheBooks`]。
+///
+/// # Errors
+/// 报告里没有某张卡、那张卡的总量、保留、空闲读不出，或一行进程号认不出。
+pub fn card_memory_readings_of_one_nvidia_smi_report(
+    report: &str,
+    this_process_identifier: u32,
+    buses: &[String],
+    has_devices_outside_the_books_on_the_bus: &dyn Fn(&str) -> bool,
+) -> Result<Vec<CardMemoryReading>, String> {
+    let mut cards: BTreeMap<String, OneCardOfTheReport> = BTreeMap::new();
+    let mut current_bus: Option<String> = None;
+    let mut section = SectionOfTheReport::Other;
+    let mut the_listed_process_is_this_one = false;
+    for line in report.lines() {
+        if let Some(bus) = line.strip_prefix("GPU ") {
+            let bus = pci_bus_identifier_without_the_domain(bus);
+            cards.insert(bus.clone(), OneCardOfTheReport::default());
+            current_bus = Some(bus);
+            section = SectionOfTheReport::Other;
+            the_listed_process_is_this_one = false;
+            continue;
+        }
+        let Some(card) = current_bus.as_ref().and_then(|bus| cards.get_mut(bus)) else {
+            continue;
+        };
+        let indentation = line.len() - line.trim_start().len();
+        let (key, value) = match line.split_once(':') {
+            Some((key, value)) => (key.trim(), value.trim()),
+            None => (line.trim(), ""),
+        };
+        if indentation <= 4 {
+            section = match key {
+                "FB Memory Usage" => SectionOfTheReport::FrameBufferMemory,
+                "Processes" => SectionOfTheReport::Processes,
+                _ => SectionOfTheReport::Other,
+            };
+            continue;
+        }
+        match (section, key) {
+            (SectionOfTheReport::FrameBufferMemory, "Total") => {
+                card.total_mebibytes = mebibytes_of_a_report_value(value);
+            }
+            (SectionOfTheReport::FrameBufferMemory, "Reserved") => {
+                card.reserved_mebibytes = mebibytes_of_a_report_value(value);
+            }
+            (SectionOfTheReport::FrameBufferMemory, "Free") => {
+                card.free_mebibytes = mebibytes_of_a_report_value(value);
+            }
+            (SectionOfTheReport::Processes, "Process ID") => {
+                let process_identifier: u32 = value
+                    .parse()
+                    .map_err(|_| format!("nvidia-smi 的报告里一行进程号认不出：{line:?}"))?;
+                the_listed_process_is_this_one = process_identifier == this_process_identifier;
+            }
+            (SectionOfTheReport::Processes, "Used GPU Memory")
+                if the_listed_process_is_this_one =>
+            {
+                card.used_by_this_process_mebibytes = Some(
+                    match (
+                        card.used_by_this_process_mebibytes,
+                        mebibytes_of_a_report_value(value),
+                    ) {
+                        (None, Some(used)) => Ok(used),
+                        (Some(Ok(earlier)), Some(used)) => Ok(earlier.saturating_add(used)),
+                        (Some(Err(())), Some(_)) | (_, None) => Err(()),
+                    },
+                );
+            }
+            (
+                SectionOfTheReport::FrameBufferMemory
+                | SectionOfTheReport::Processes
+                | SectionOfTheReport::Other,
+                _,
+            ) => {}
+        }
+    }
+    buses
+        .iter()
+        .map(|bus| {
+            let card = cards
+                .get(bus)
+                .ok_or_else(|| format!("nvidia-smi 的报告里没有总线 {bus} 那张卡"))?;
+            let unreadable =
+                |what: &str| format!("nvidia-smi 的报告里总线 {bus} 那张卡的 {what} 读不出");
+            let total_mebibytes = card.total_mebibytes.ok_or_else(|| unreadable("Total"))?;
+            let reserved_mebibytes = card
+                .reserved_mebibytes
+                .ok_or_else(|| unreadable("Reserved"))?;
+            let free_mebibytes = card.free_mebibytes.ok_or_else(|| unreadable("Free"))?;
+            let occupied_by_this_process = match card.used_by_this_process_mebibytes {
+                None | Some(Err(())) => OccupiedByThisProcess::NotListed,
+                Some(Ok(mebibytes)) => {
+                    if has_devices_outside_the_books_on_the_bus(bus) {
+                        OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes }
+                    } else {
+                        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes }
+                    }
+                }
+            };
+            Ok(CardMemoryReading {
+                free_mebibytes,
+                total_less_reserved_mebibytes: total_mebibytes.saturating_sub(reserved_mebibytes),
+                occupied_by_this_process,
+            })
+        })
+        .collect()
+}
+
+/// 卡池读显存的那一份：每读一次照 `run_the_report` 取一份报告（卡池是跑一次 `nvidia-smi`），整张表照这一份读
+/// （[`card_memory_readings_of_one_nvidia_smi_report`]）：空闲与这个进程占着的不分两次读（文件头）。
+#[must_use]
+pub fn card_memory_reader_of_one_report_per_reading(
+    run_the_report: Box<dyn Fn() -> Result<String, String> + Send + Sync>,
+    this_process_identifier: u32,
+    buses: Vec<String>,
+    has_devices_outside_the_books_on_the_bus: Box<dyn Fn(&str) -> bool + Send + Sync>,
+) -> CardMemoryReader {
+    Box::new(move || {
+        let report = run_the_report()?;
+        card_memory_readings_of_one_nvidia_smi_report(
+            &report,
+            this_process_identifier,
+            &buses,
+            &*has_devices_outside_the_books_on_the_bus,
+        )
+    })
+}
+
 /// 一张卡上这个进程一共能占多少（MiB）：`(卡在池里的次序, 这个进程能占的)` → 容量（这个进程自己的驱动那一份已经扣掉，
 /// 卡池照 [`card_ledger_capacity_mebibytes`] 算）。
 pub type CardCapacityForThisProcess = Box<dyn Fn(usize, u64) -> u64 + Send + Sync>;
@@ -504,6 +808,8 @@
 struct BooksOfACard {
     leases: BTreeMap<u64, u64>,
     driver_share: DriverShareOnTheCard,
+    /// 销了账、卡上可能还没还的（[`BookedByThisProcessOnACard::returned_possibly_still_on_the_card_mebibytes`]）。
+    returned_possibly_still_on_the_card_mebibytes: u64,
 }
 
 impl BooksOfACard {
@@ -511,6 +817,21 @@
         BookedByThisProcessOnACard {
             leases_mebibytes: self.leases.values().sum(),
             driver_share: self.driver_share,
+            returned_possibly_still_on_the_card_mebibytes: self
+                .returned_possibly_still_on_the_card_mebibytes,
+        }
+    }
+
+    /// 销一笔借据（借据丢掉时）：它建的缓冲记进卡上可能还没还的那一份，交回销没销到。
+    fn return_a_lease(&mut self, lease_identifier: u64) -> bool {
+        match self.leases.remove(&lease_identifier) {
+            Some(mebibytes) => {
+                self.returned_possibly_still_on_the_card_mebibytes = self
+                    .returned_possibly_still_on_the_card_mebibytes
+                    .saturating_add(mebibytes);
+                true
+            }
+            None => false,
         }
     }
 
@@ -570,6 +891,7 @@
                     .map(|_| BooksOfACard {
                         leases: BTreeMap::new(),
                         driver_share: DriverShareOnTheCard::NotRegistered,
+                        returned_possibly_still_on_the_card_mebibytes: 0,
                     })
                     .collect(),
             ),
@@ -582,10 +904,11 @@
         self.books.lock().unwrap_or_else(PoisonError::into_inner)
     }
 
-    /// 在锁里现读每张卡，算这几张卡此刻各能借给这个进程的（与 `card_positions` 逐张对上）。
+    /// 在锁里现读每张卡，算这几张卡此刻各能借给这个进程的（与 `card_positions` 逐张对上）；每张卡销了账、卡上可能还没还的那一份
+    /// 先照这一次读数往下压（[`returned_possibly_still_on_the_card_after_a_reading`]）。
     fn rooms_of_the_cards(
         &self,
-        books: &[BooksOfACard],
+        books: &mut [BooksOfACard],
         card_positions: &[usize],
     ) -> Result<Vec<CardRoomForThisProcess>, BookingRefusal> {
         let readings = (self.read_card_memory)()
@@ -599,6 +922,15 @@
                 ),
             });
         }
+        for (books_of_the_card, reading) in books.iter_mut().zip(&readings) {
+            books_of_the_card.returned_possibly_still_on_the_card_mebibytes =
+                returned_possibly_still_on_the_card_after_a_reading(
+                    books_of_the_card.returned_possibly_still_on_the_card_mebibytes,
+                    reading.occupied_by_this_process,
+                    books_of_the_card.driver_share,
+                    self.mebibytes_each_process_keeps_on_a_card,
+                );
+        }
         Ok(card_positions
             .iter()
             .map(|card_position| {
@@ -642,14 +974,25 @@
         &self,
         card_positions: &[usize],
     ) -> Result<Vec<u64>, BookingRefusal> {
-        let books = self.books_locked();
         Ok(self
-            .rooms_of_the_cards(&books, card_positions)?
+            .card_rooms_now_and_when_empty(card_positions)?
             .iter()
             .map(|room| room.room_now_mebibytes)
             .collect())
     }
 
+    /// 这几张卡此刻与空着时各能借给这个进程多少（现读一次，与 `card_positions` 逐张对上）。
+    ///
+    /// # Errors
+    /// 卡上的显存读不出（[`BookingRefusal::CardMemoryUnreadable`]）。
+    pub fn card_rooms_now_and_when_empty(
+        &self,
+        card_positions: &[usize],
+    ) -> Result<Vec<CardRoomForThisProcess>, BookingRefusal> {
+        let mut books = self.books_locked();
+        self.rooms_of_the_cards(&mut books, card_positions)
+    }
+
     /// 借 `need_mebibytes`：照卡的优先级记在第一张此刻放得下的卡上（这个进程在那张卡上还没登记的，它的驱动那一份也要放得下，
     /// 一并登记），交回借据（丢掉时销账）；此刻每张都放不下就交借不到、什么都不记，不等。
     ///
@@ -692,7 +1035,7 @@
             "借 0 MiB 照样要登记驱动那一份：调用方按实际要建的缓冲借，至少 1 MiB"
         );
         let mut books = self.books_locked();
-        let rooms = self.rooms_of_the_cards(&books, card_positions)?;
+        let rooms = self.rooms_of_the_cards(&mut books, card_positions)?;
         match decide_booking_by_priority(&rooms, need_mebibytes) {
             BookingDecision::BookOnTheCardAt { card_position } => {
                 let booked = self.book_in_the_lock(
@@ -756,7 +1099,7 @@
             "一个状态要 0 MiB：判器按状态数借的那一族至少有表与一个状态的草稿区"
         );
         let mut books = self.books_locked();
-        let rooms = self.rooms_of_the_cards(&books, card_positions)?;
+        let rooms = self.rooms_of_the_cards(&mut books, card_positions)?;
         let mut booked_on_each_card: Vec<Option<BookedOnACard>> =
             card_positions.iter().map(|_| None).collect();
         let attempts = place_states_on_the_cards_that_book_now(
@@ -783,14 +1126,13 @@
                     ));
                     return ScratchAttemptOnACard::Booked { states };
                 }
-                if need_of_one_state_mebibytes > room.ceiling_mebibytes {
-                    return ScratchAttemptOnACard::CannotHoldOneState {
-                        ceiling_mebibytes: room.ceiling_mebibytes,
-                    };
-                }
-                ScratchAttemptOnACard::NoRoomNow {
-                    need_mebibytes: need_of_one_state_mebibytes,
-                }
+                attempt_on_a_card_without_room_for_one_state(need_of_one_state_mebibytes, room)
+            },
+            &mut |card_slot| {
+                attempt_on_a_card_without_room_for_one_state(
+                    need_of_one_state_mebibytes,
+                    rooms[card_slot],
+                )
             },
         );
         drop(books);
@@ -925,12 +1267,11 @@
 
 impl Drop for GpuMemoryBooking<'_> {
     fn drop(&mut self) {
-        let removed = self.ledger.books_locked()[self.card_position]
-            .leases
-            .remove(&self.lease_identifier);
+        let removed =
+            self.ledger.books_locked()[self.card_position].return_a_lease(self.lease_identifier);
         // 别处已经在 panic（丢借据是收尾的一部分）时不再断言：两次 panic 会让整个进程中止
         assert!(
-            removed.is_some() || std::thread::panicking(),
+            removed || std::thread::panicking(),
             "第 {} 张卡上借据号 {} 不在账上：一笔借据只在它丢掉时销一次账",
             self.card_position,
             self.lease_identifier
@@ -938,8 +1279,63 @@
     }
 }
 
+/// 一张卡上这个进程的设备（卡池每张卡一格）：要时有留着的、没丢的就拿它；没有（第一次要、上一次起不来）或丢了就起一个，
+/// 起好了留着，起不来交为什么、不留着（第 10x 批代码轮判决 K1-3：起设备失败不缓存成这个进程永久的错；设备丢了之后借这张卡的
+/// 不拿丢了的那个）。不碰 wgpu：设备、怎么起、怎么算丢了由卡池给，单测换成假的。
+pub struct DeviceStartedOnDemand<Device> {
+    started: Mutex<Option<Device>>,
+}
+
+/// 空着（还没起设备）就是起点，没有别的不变量。
+impl<Device: Clone> Default for DeviceStartedOnDemand<Device> {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+impl<Device: Clone> DeviceStartedOnDemand<Device> {
+    #[must_use]
+    pub const fn new() -> Self {
+        Self {
+            started: Mutex::new(None),
+        }
+    }
+
+    /// 留着的设备 `is_lost` 不为真就交它的克隆；否则调 `start` 起一个，起好了留着、交克隆，起不来交 `start` 的错（不留着）。
+    /// 起的时候拿着这张卡的锁：同一张卡上同时要的等这一次起完（别的卡不等）。
+    ///
+    /// # Errors
+    /// `start` 的错。
+    pub fn started_or_start<Failure>(
+        &self,
+        is_lost: impl Fn(&Device) -> bool,
+        start: impl FnOnce() -> Result<Device, Failure>,
+    ) -> Result<Device, Failure> {
+        // 持锁时 panic 只会在 `start` 里（起设备的断言）：那时这一格还是旧的（没有或丢了的），别的线程照用、照起
+        let mut started = self.started.lock().unwrap_or_else(PoisonError::into_inner);
+        if let Some(device) = started.as_ref() {
+            if !is_lost(device) {
+                return Ok(device.clone());
+            }
+        }
+        let device = start()?;
+        *started = Some(device.clone());
+        Ok(device)
+    }
+
+    /// 起过、留着一个设备没有（丢了的也算留着，下一次要时重起）。
+    #[must_use]
+    pub fn is_started(&self) -> bool {
+        self.started
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .is_some()
+    }
+}
+
 #[cfg(test)]
 mod tests {
+    use super::DeviceStartedOnDemand;
     use super::{
         card_ledger_capacity_mebibytes, card_room_for_this_process,
         decide_after_attempting_every_card, decide_booking_by_priority,
@@ -949,22 +1345,32 @@
         states_in_one_dispatch_capped_by_the_state_count, BookedByThisProcessOnACard,
         BookingDecision, BookingRefusal, CardMemoryReading, CardRoomForThisProcess,
         DriverShareOnTheCard, GpuCardsSource, GpuMemoryLedger, NeedLargerThanEveryCard,
-        ScratchAttemptOnACard, ScratchBookingRefusalOutcome, ScratchPlacementDecision,
-        ScratchStatesToBook,
+        OccupiedByThisProcess, ScratchAttemptOnACard, ScratchBeforeBooking,
+        ScratchBookingRefusalOutcome, ScratchPlacementDecision, ScratchStatesToBook,
     };
+    use super::{
+        card_memory_reader_of_one_report_per_reading,
+        card_memory_readings_of_one_nvidia_smi_report, decide_scratch_before_booking,
+        gpu_not_run_line, pci_bus_identifier_without_the_domain,
+        reason_for_running_out_of_gpu_memory, GPU_MEMORY_SHORT_REASON_PREFIX,
+    };
+    use std::sync::atomic::{AtomicUsize, Ordering};
     use std::sync::{Arc, Mutex};
 
     /// 驱动的固定开销，照卡池用的那个数（`crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER`）。
     const DRIVER: u64 = 2560;
-    /// 一张 5090 的显存总量（`nvidia-smi` 报的 memory.total）。
+    /// 一张 5090 的显存总量与驱动保留的那一份（2026-10-07 本机 `nvidia-smi -q -d MEMORY` 报的 Total、Reserved）。
     const TOTAL_OF_AN_RTX_5090: u64 = 32_607;
+    const RESERVED_ON_AN_RTX_5090: u64 = 497;
+    /// 一张空着的 5090 上能用的：总量减保留（同一刻报的空闲就是这个数）。
+    const AN_EMPTY_RTX_5090_HAS: u64 = TOTAL_OF_AN_RTX_5090 - RESERVED_ON_AN_RTX_5090;
 
     /// 一张 5090 此刻空着 `free_mebibytes`，这个进程实际占着的读不到。
     fn reading_of_an_rtx_5090(free_mebibytes: u64) -> CardMemoryReading {
         CardMemoryReading {
             free_mebibytes,
-            total_mebibytes: TOTAL_OF_AN_RTX_5090,
-            occupied_by_this_process_mebibytes: None,
+            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
+            occupied_by_this_process: OccupiedByThisProcess::NotListed,
         }
     }
 
@@ -1046,7 +1452,7 @@
         );
     }
 
-    /// 第 10x 批（G1 二）：外人占着卡（空 5 000）时借 10 000 借不到，不判红（卡空着时借得出 30 047）；
+    /// 第 10x 批（G1 二）：外人占着卡（空 5 000）时借 10 000 借不到，不判红（卡空着时借得出 29 550）；
     /// 外人走了（空 30 000），下一笔照现读的借到。
     #[test]
     fn a_booking_after_an_outsider_left_gets_the_memory_it_returned() {
@@ -1099,6 +1505,7 @@
             BookedByThisProcessOnACard {
                 leases_mebibytes,
                 driver_share,
+                returned_possibly_still_on_the_card_mebibytes: 0,
             }
         };
         assert_eq!(
@@ -1129,7 +1536,7 @@
             "起了设备、实际占着的读不到：驱动那一份当已经在卡上"
         );
         let occupied = |mebibytes: u64| CardMemoryReading {
-            occupied_by_this_process_mebibytes: Some(mebibytes),
+            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
             ..free
         };
         assert_eq!(
@@ -1282,13 +1689,13 @@
         assert_eq!(states_booked, vec![Some(5), Some(13)]);
     }
 
-    /// 第 10x 批：要的比卡空着时能借的（总量减驱动那一份，30 047）还大，判成比每张卡都大（调用方判红）；不超过它、只是此刻
+    /// 第 10x 批：要的比卡空着时能借的（空卡上能用的减驱动那一份，29 550）还大，判成比每张卡都大（调用方判红）；不超过它、只是此刻
     /// 放不下的是借不到（调用方报本次未跑）。按状态数借时一个状态都放不下同理。
     #[test]
     fn a_need_larger_than_the_card_when_empty_never_fits_and_a_smaller_one_has_no_room_now() {
         let readings = replaceable_readings(vec![reading_of_an_rtx_5090(5_000)]);
         let ledger = ledger_reading(&readings);
-        let ceiling = TOTAL_OF_AN_RTX_5090 - DRIVER;
+        let ceiling = AN_EMPTY_RTX_5090_HAS - DRIVER;
         assert_eq!(
             booking_kept_on_the_books(&ledger, ceiling + 1),
             Err(BookingRefusal::NeedLargerThanEveryCard(
@@ -1642,6 +2049,11 @@
         );
     }
 
+    /// 这几条用例里每张卡都带得了状态：调用方算出一个都带不了的那一支走不到。
+    fn every_card_here_holds_a_state(card_slot: usize) -> ScratchAttemptOnACard {
+        panic!("第 {card_slot} 张卡带得了状态，不该问它一个都带不了时算哪一种")
+    }
+
     /// 第 10w 批「几张卡一张借不到时不拿着别的卡干等」：100 个状态、两张卡各带得了 60 个；第一张当场借到 60 个，第二张此刻借不到。
     /// 第二张只试一次、要装的是剩下的 40 个，借不到就不装它，判成就在第一张上判（剩下的 40 个排队等第一张）。
     #[test]
@@ -1661,6 +2073,7 @@
                     },
                 }
             },
+            &mut every_card_here_holds_a_state,
         );
         assert_eq!(
             attempts_made,
@@ -1699,6 +2112,7 @@
                     states: states_to_place,
                 },
             },
+            &mut every_card_here_holds_a_state,
         );
         assert_eq!(
             first_card_without_room[1],
@@ -1718,6 +2132,7 @@
                     }
                 }
             },
+            &mut every_card_here_holds_a_state,
         );
         assert_eq!(
             second_card_asked_for, 60,
@@ -1824,19 +2239,20 @@
         );
     }
 
-    /// 一张卡此刻能借给这个进程的照 [`card_room_for_this_process`]：容量拿这个进程能占的算、减借着的；空着时能借的拿总量算、
-    /// 不减借着的。
+    /// 一张卡此刻能借给这个进程的照 [`card_room_for_this_process`]：容量拿这个进程能占的算、减借着的；空着时能借的拿空卡上能用的
+    /// （总量减保留）算、不减借着的。
     #[test]
-    fn the_room_now_subtracts_the_leases_and_the_ceiling_takes_the_total_memory() {
+    fn the_room_now_subtracts_the_leases_and_the_ceiling_takes_what_an_empty_card_has() {
         let room = card_room_for_this_process(
             CardMemoryReading {
                 free_mebibytes: 20_000,
-                total_mebibytes: TOTAL_OF_AN_RTX_5090,
-                occupied_by_this_process_mebibytes: None,
+                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
+                occupied_by_this_process: OccupiedByThisProcess::NotListed,
             },
             BookedByThisProcessOnACard {
                 leases_mebibytes: 3_000,
                 driver_share: DriverShareOnTheCard::DeviceStarted,
+                returned_possibly_still_on_the_card_mebibytes: 0,
             },
             DRIVER,
             &|may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER),
@@ -1845,8 +2261,493 @@
             room,
             CardRoomForThisProcess {
                 room_now_mebibytes: 20_000 - 3_000,
-                ceiling_mebibytes: TOTAL_OF_AN_RTX_5090 - DRIVER,
+                ceiling_mebibytes: AN_EMPTY_RTX_5090_HAS - DRIVER,
+            }
+        );
+    }
+
+    /// 测试里进程号写死的「这个进程」与「别的进程」。
+    const THIS_PROCESS: u32 = 4_242;
+    const ANOTHER_PROCESS: u32 = 7_777;
+    /// 测试里那张 5090 的总线号（`nvidia-smi` 写法，去掉域之前）。
+    const BUS_OF_THE_RTX_5090: &str = "00000000:01:00.0";
+    const BUS_OF_THE_RTX_5060_TI: &str = "00000000:10:00.0";
+
+    /// 一张卡在假报告里的那一段：空闲与每个进程（进程号、占的 MiB）。总量、保留照 5090 的两个数写。
+    struct CardInTheReport<'processes> {
+        bus: &'processes str,
+        total_mebibytes: u64,
+        reserved_mebibytes: u64,
+        free_mebibytes: u64,
+        processes: &'processes [(u32, u64)],
+    }
+
+    /// 照本机驱动 595.84 的 `nvidia-smi -q -d MEMORY,PIDS` 文本写一份假报告（BAR1、Conf Compute 两小节也带着，
+    /// 它们的 Total、Free 不该被当成显存读）。
+    fn nvidia_smi_report(cards: &[CardInTheReport<'_>]) -> String {
+        let mut report = format!(
+            "\n==============NVSMI LOG==============\n\nTimestamp                                              : Wed Oct  7 2026\n\
+             Driver Version                                         : 595.84\nCUDA Version                                           : 13.2\n\n\
+             Attached GPUs                                          : {}\n",
+            cards.len()
+        );
+        for card in cards {
+            report.push_str(&format!(
+                "GPU {}\n    FB Memory Usage\n        Total                                          : {} MiB\n\
+                 \x20       Reserved                                       : {} MiB\n        Used                                           : 2 MiB\n\
+                 \x20       Free                                           : {} MiB\n    BAR1 Memory Usage\n\
+                 \x20       Total                                          : 32768 MiB\n        Used                                           : 1 MiB\n\
+                 \x20       Free                                           : 32767 MiB\n    Conf Compute Protected Memory Usage\n\
+                 \x20       Total                                          : 0 MiB\n        Used                                           : 0 MiB\n\
+                 \x20       Free                                           : 0 MiB\n",
+                card.bus, card.total_mebibytes, card.reserved_mebibytes, card.free_mebibytes
+            ));
+            if card.processes.is_empty() {
+                report
+                    .push_str("    Processes                                          : None\n\n");
+                continue;
+            }
+            report.push_str("    Processes\n");
+            for (process_identifier, used_mebibytes) in card.processes {
+                report.push_str(&format!(
+                    "        GPU instance ID                                : N/A\n        Compute instance ID                            : N/A\n\
+                     \x20       Process ID                                     : {process_identifier}\n\
+                     \x20           Type                                       : C+G\n\
+                     \x20           Name                                       : /tmp/target/debug/deps/a_gpu_test\n\
+                     \x20           Used GPU Memory                            : {used_mebibytes} MiB\n"
+                ));
             }
+            report.push('\n');
+        }
+        report
+    }
+
+    /// 一张 5090 的那一段：空 `free_mebibytes`，进程照给的。
+    fn an_rtx_5090_in_the_report(
+        free_mebibytes: u64,
+        processes: &[(u32, u64)],
+    ) -> CardInTheReport<'_> {
+        CardInTheReport {
+            bus: BUS_OF_THE_RTX_5090,
+            total_mebibytes: TOTAL_OF_AN_RTX_5090,
+            reserved_mebibytes: RESERVED_ON_AN_RTX_5090,
+            free_mebibytes,
+            processes,
+        }
+    }
+
+    fn no_device_outside_the_books(_bus: &str) -> bool {
+        false
+    }
+
+    /// 第 10x 批代码轮判决 K1-1：一份报告里读出每张卡 FB 那一小节的空闲与空卡上能用的（BAR1 的不算），与这个进程那几行占的合计
+    /// （别的进程的不算）；这个进程在一张卡上另有不走账的设备时那一份记成分不出；没列这个进程是没列；报告里没有点名的卡是读不出。
+    #[test]
+    fn one_nvidia_smi_report_gives_the_frame_buffer_memory_and_this_process_occupancy_of_each_card()
+    {
+        let report = nvidia_smi_report(&[
+            an_rtx_5090_in_the_report(
+                20_000,
+                &[
+                    (ANOTHER_PROCESS, 9_000),
+                    (THIS_PROCESS, 3_000),
+                    (THIS_PROCESS, 211),
+                ],
+            ),
+            CardInTheReport {
+                bus: BUS_OF_THE_RTX_5060_TI,
+                total_mebibytes: 16_311,
+                reserved_mebibytes: 462,
+                free_mebibytes: 15_849,
+                processes: &[(ANOTHER_PROCESS, 500)],
+            },
+        ]);
+        let buses = vec![
+            pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090),
+            pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5060_TI),
+        ];
+        assert_eq!(buses, vec!["01:00.0".to_string(), "10:00.0".to_string()]);
+        assert_eq!(
+            card_memory_readings_of_one_nvidia_smi_report(
+                &report,
+                THIS_PROCESS,
+                &buses,
+                &no_device_outside_the_books
+            ),
+            Ok(vec![
+                CardMemoryReading {
+                    free_mebibytes: 20_000,
+                    total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
+                    occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
+                        mebibytes: 3_211
+                    },
+                },
+                CardMemoryReading {
+                    free_mebibytes: 15_849,
+                    total_less_reserved_mebibytes: 16_311 - 462,
+                    occupied_by_this_process: OccupiedByThisProcess::NotListed,
+                },
+            ]),
+            "FB 那一小节的空闲与总量减保留；这个进程两行相加，别的进程不算；第二张卡没列这个进程"
+        );
+        assert_eq!(
+            card_memory_readings_of_one_nvidia_smi_report(&report, THIS_PROCESS, &buses, &|bus| {
+                bus == "01:00.0"
+            })
+            .map(|readings| readings[0].occupied_by_this_process),
+            Ok(OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 3_211 }),
+            "这个进程在这张卡上另有不走账的设备：占的那一份分不出"
+        );
+        assert!(
+            card_memory_readings_of_one_nvidia_smi_report(
+                &report,
+                THIS_PROCESS,
+                &["02:00.0".to_string()],
+                &no_device_outside_the_books
+            )
+            .is_err(),
+            "报告里没有点名的卡：读不出"
+        );
+    }
+
+    /// 第 10x 批代码轮判决 K1-1（两腿甲形）：每借一次只取一份报告，空闲与这个进程占着的出自那一份。假的报告按次序给：
+    /// 第一份卡空着（这一笔借 3 000）；第二份是第一笔建上卡之后同一刻的空闲与占着的；第三份是外人又占了 10 000。
+    /// 两次借各取一份、借的那一刻能借的照各自那一份算；取两份拼一份（空闲取前一份、占着的取后一份，或整份取后一份）都对不上。
+    #[test]
+    fn each_booking_reads_the_free_memory_and_this_process_occupancy_from_one_report() {
+        let reports = [
+            nvidia_smi_report(&[an_rtx_5090_in_the_report(30_000, &[])]),
+            nvidia_smi_report(&[an_rtx_5090_in_the_report(
+                30_000 - DRIVER - 3_000,
+                &[(THIS_PROCESS, DRIVER + 3_000)],
+            )]),
+            nvidia_smi_report(&[an_rtx_5090_in_the_report(
+                30_000 - DRIVER - 3_000 - 10_000,
+                &[(THIS_PROCESS, DRIVER + 3_000), (ANOTHER_PROCESS, 10_000)],
+            )]),
+        ];
+        let reports_taken = Arc::new(AtomicUsize::new(0));
+        let taken_by_the_reader = Arc::clone(&reports_taken);
+        let ledger = GpuMemoryLedger::new(
+            1,
+            card_memory_reader_of_one_report_per_reading(
+                Box::new(move || {
+                    let taken = taken_by_the_reader.fetch_add(1, Ordering::SeqCst);
+                    reports
+                        .get(taken)
+                        .cloned()
+                        .ok_or_else(|| format!("第 {} 份报告用例里没有", taken + 1))
+                }),
+                THIS_PROCESS,
+                vec![pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
+                Box::new(no_device_outside_the_books),
+            ),
+            Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER)),
+            DRIVER,
+        );
+        let first = ledger.book_on_the_card(0, 3_000).expect("空着的卡借得到");
+        assert_eq!(
+            first.card_room_mebibytes_when_booked(),
+            30_000 - DRIVER,
+            "第一笔照第一份报告：空 30 000，减还没起设备的驱动那一份"
+        );
+        ledger.record_the_device_started_on_the_card(0);
+        let second = ledger
+            .book_on_the_card(0, 1_000)
+            .expect("第二份报告里还空着 24 440");
+        assert_eq!(
+            second.card_room_mebibytes_when_booked(),
+            30_000 - DRIVER - 3_000,
+            "第二笔照第二份报告：第一笔建上卡的 3 000 只从空闲里扣一次，不既在空闲里又在占着的里"
+        );
+        assert_eq!(
+            reports_taken.load(Ordering::SeqCst),
+            2,
+            "借两次各取一份报告：空闲与占着的不分两次读"
+        );
+        drop(second);
+        drop(first);
+    }
+
+    /// 第 10x 批代码轮判决 K1-1（攻方乙形）：这个进程在卡上占着 9 000，其中有不走这份账的设备（读数记成分不出）。
+    /// 起了设备、借着 4 000 还没建：能借的照读不到实际占着的算，不把那 9 000 当成借据已经建上的那一份。
+    #[test]
+    fn memory_of_this_process_outside_the_books_is_not_taken_for_leases_already_built() {
+        let booked = BookedByThisProcessOnACard {
+            leases_mebibytes: 4_000,
+            driver_share: DriverShareOnTheCard::DeviceStarted,
+            returned_possibly_still_on_the_card_mebibytes: 0,
+        };
+        let reading = |occupied_by_this_process| CardMemoryReading {
+            free_mebibytes: 15_000,
+            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
+            occupied_by_this_process,
+        };
+        assert_eq!(
+            memory_this_process_may_hold_mebibytes(
+                reading(OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 9_000 }),
+                booked,
+                DRIVER
+            ),
+            15_000 + DRIVER,
+            "分不出：借着的 4 000 当一点都没建，只把起好了设备的驱动那一份当在卡上"
+        );
+        assert_eq!(
+            memory_this_process_may_hold_mebibytes(
+                reading(OccupiedByThisProcess::NotListed),
+                booked,
+                DRIVER
+            ),
+            15_000 + DRIVER,
+            "与没列这个进程同样算"
+        );
+        assert_eq!(
+            memory_this_process_may_hold_mebibytes(
+                reading(OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 9_000 }),
+                booked,
+                DRIVER
+            ),
+            15_000 + DRIVER + 4_000,
+            "都走这份账时那 9 000 封顶到许出去的 6 560，才算借据建上了"
+        );
+    }
+
+    /// 第 10x 批代码轮判决 K1-1（正推冲突 B）：第一笔 4 000 建上卡之后丢掉（销了账），卡上还占着；第二笔 4 000 借着还没建。
+    /// 第三笔能借的照「卡上真空着的减第二笔还要建的」算（23 440 − 4 000），不把还没还的 4 000 当成第二笔已经建上的。
+    /// 卡上把第一笔还了之后，那一份照读数压掉，能借的照旧不少算。
+    #[test]
+    fn a_returned_lease_still_on_the_card_is_not_taken_for_a_live_lease_already_built() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+        let ledger = ledger_reading(&readings);
+        let first = ledger.book_on_the_card(0, 4_000).expect("空着的卡借得到");
+        ledger.record_the_device_started_on_the_card(0);
+        let first_built = CardMemoryReading {
+            free_mebibytes: 30_000 - DRIVER - 4_000,
+            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
+            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
+                mebibytes: DRIVER + 4_000,
+            },
+        };
+        replace_the_reading(&readings, 0, first_built);
+        drop(first);
+        let second = ledger
+            .book_on_the_card(0, 4_000)
+            .expect("卡上空着 23 440，借得到");
+        assert_eq!(
+            ledger.card_rooms_mebibytes_now(&[0]),
+            Ok(vec![30_000 - DRIVER - 4_000 - 4_000]),
+            "还没还的 4 000 不算第二笔已经建上的：第二笔还要从空闲里拿 4 000"
+        );
+        assert!(
+            matches!(
+                ledger.book_on_the_card(0, 30_000 - DRIVER - 4_000 - 4_000 + 1),
+                Err(BookingRefusal::NoCardHasRoomNow { .. })
+            ),
+            "多 1 MiB 就借不到"
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
+            Ok(vec![30_000 - DRIVER - 4_000]),
+            "卡上还了第一笔：那一份照读数压掉，只扣第二笔还要建的"
+        );
+        drop(second);
+    }
+
+    /// 第 10x 批代码轮判决 K1-1（攻方丙形）：报告里一张空着的 5090 空 32 110（总量 32 607 减保留 497）。要的比空卡上能用的减驱动那一份
+    /// （29 550）大 1 MiB：判成比每张卡都大（判红），不是永远借不到的本次未跑。
+    #[test]
+    fn a_need_beyond_what_an_empty_card_has_never_fits_even_below_the_total_memory() {
+        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(5_000, &[])]);
+        let ledger = GpuMemoryLedger::new(
+            1,
+            card_memory_reader_of_one_report_per_reading(
+                Box::new(move || Ok(report.clone())),
+                THIS_PROCESS,
+                vec![pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
+                Box::new(no_device_outside_the_books),
+            ),
+            Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER)),
+            DRIVER,
+        );
+        let ceiling = AN_EMPTY_RTX_5090_HAS - DRIVER;
+        assert!(ceiling + 1 < TOTAL_OF_AN_RTX_5090 - DRIVER);
+        assert_eq!(
+            booking_kept_on_the_books(&ledger, ceiling + 1),
+            Err(BookingRefusal::NeedLargerThanEveryCard(
+                NeedLargerThanEveryCard {
+                    need_mebibytes: ceiling + 1,
+                    largest_ceiling_mebibytes: ceiling,
+                }
+            )),
+            "空卡上能用的装不下：判红，不照总量判成此刻借不到"
+        );
+    }
+
+    /// 第 10x 批代码轮判决 K2-2：调用方照此刻能借的算出这张卡一个状态都带不了（带得了 0 个），而空着时放得下：
+    /// 判成此刻借不到（调用方报本次未跑），不当成前面的卡装完了、一张都没借到就判红。
+    #[test]
+    fn a_card_the_caller_found_holding_no_state_now_is_no_room_now_not_nothing_left_to_place() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(DRIVER + 1_000)]);
+        let ledger = ledger_reading(&readings);
+        assert_eq!(
+            ledger
+                .book_states_on_the_cards_with_room(&[0], 10, &[0], &|states| states * 2_000)
+                .map(|bookings| bookings.len()),
+            Err(BookingRefusal::NoCardHasRoomNow {
+                need_mebibytes: 2_000,
+                card_rooms_mebibytes: vec![1_000],
+            }),
+            "此刻带不了、空着时放得下：此刻借不到"
+        );
+        assert_eq!(
+            place_states_on_the_cards_that_book_now(
+                10,
+                &[4, 0],
+                &mut |_card_slot, states_to_place| ScratchAttemptOnACard::Booked {
+                    states: states_to_place
+                },
+                &mut |_card_slot| ScratchAttemptOnACard::NoRoomNow {
+                    need_mebibytes: 2_000
+                },
+            ),
+            vec![
+                ScratchAttemptOnACard::Booked { states: 4 },
+                ScratchAttemptOnACard::NoRoomNow {
+                    need_mebibytes: 2_000
+                },
+            ],
+            "还剩 6 个要装、第二张此刻带不了：是此刻借不到，不是没有要装的"
+        );
+    }
+
+    /// 第 10x 批代码轮判决 K2-2（白红那一处）：借草稿区之前，每张卡此刻都带不了一个状态而至少一张空着时带得了，是此刻借不到
+    /// （报本次未跑）；空着时也带不了（一张卡都没有也算）才判红；有一张此刻带得了就去借。
+    #[test]
+    fn with_no_card_holding_a_state_now_a_card_holding_one_when_empty_means_not_run_not_red() {
+        assert_eq!(
+            decide_scratch_before_booking(&[0, 0], &[0, 3]),
+            ScratchBeforeBooking::NoCardHoldsAStateNow,
+            "第二张空着时带得了 3 个：此刻借不到"
+        );
+        assert_eq!(
+            decide_scratch_before_booking(&[0, 0], &[0, 0]),
+            ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty,
+            "空着时也带不了：判红"
+        );
+        assert_eq!(
+            decide_scratch_before_booking(&[], &[]),
+            ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty,
+            "一张卡都没有：判红"
+        );
+        assert_eq!(
+            decide_scratch_before_booking(&[0, 2], &[5, 5]),
+            ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow,
+            "第二张此刻带得了 2 个：去借"
+        );
+    }
+
+    /// 第 10x 批代码轮判决 K1-3、K3：卡上 wgpu 报 Out of Memory 那一句以「显存不够：」起头，打成本次未跑那一行时跟着 GPU 用例的记号
+    /// （门禁 feature-gated-tests 格认记号，人认这几个字）。
+    #[test]
+    fn the_out_of_memory_reason_says_the_gpu_memory_was_short_in_the_not_run_line() {
+        let reason = reason_for_running_out_of_gpu_memory("序号 0 的卡建判器时");
+        assert!(
+            reason.starts_with(GPU_MEMORY_SHORT_REASON_PREFIX),
+            "原因写显存不够：{reason}"
+        );
+        assert!(
+            gpu_not_run_line("crash_amplification::run", &reason).starts_with(
+                "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run reason=显存不够：序号 0 的卡建判器时"
+            ),
+            "本次未跑那一行：GPU 用例的记号、原因写显存不够"
+        );
+    }
+
+    /// 第 10x 批代码轮判决 K3：按状态数借时同一张卡给了两次要断言（两笔记在一张卡上、各按那张卡此刻能借的压，会借超）。
+    #[test]
+    fn the_same_card_given_twice_to_a_scratch_booking_is_refused_by_an_assertion() {
+        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
+        let ledger = ledger_reading(&readings);
+        assert!(
+            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+                ledger
+                    .book_states_on_the_cards_with_room(&[0, 0], 4, &[2, 2], &|states| states * 100)
+                    .map(|bookings| bookings.len())
+            }))
+            .is_err(),
+            "同一张卡给了两次：断言"
+        );
+        assert!(
+            !ledger.is_this_process_registered_on_the_card(0),
+            "断言在借之前：什么都没记"
+        );
+    }
+
+    /// 第 10x 批代码轮判决 K3：读到的卡比池里多（一张卡的池读到两张）也判读不出，什么都不记。
+    #[test]
+    fn a_reading_of_more_cards_than_the_pool_is_refused_and_nothing_is_booked() {
+        let too_many = GpuMemoryLedger::new(
+            1,
+            Box::new(|| {
+                Ok(vec![
+                    reading_of_an_rtx_5090(30_000),
+                    reading_of_an_rtx_5090(30_000),
+                ])
+            }),
+            Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes),
+            DRIVER,
+        );
+        assert!(matches!(
+            booking_kept_on_the_books(&too_many, 1),
+            Err(BookingRefusal::CardMemoryUnreadable { .. })
+        ));
+        assert!(!too_many.is_this_process_registered_on_the_card(0));
+    }
+
+    /// 第 10x 批代码轮判决 K1-3：卡池每张卡的设备第一次起不来不留着，下一次要时再起；起好了留着给后面要的；留着的那个丢了就重起一个。
+    #[test]
+    fn a_device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced() {
+        let slot: DeviceStartedOnDemand<u32> = DeviceStartedOnDemand::new();
+        let starts = AtomicUsize::new(0);
+        let start_as = |device: u32| {
+            starts.fetch_add(1, Ordering::SeqCst);
+            Ok::<u32, String>(device)
+        };
+        let never_lost = |_device: &u32| false;
+        assert_eq!(
+            slot.started_or_start(never_lost, || Err::<u32, String>("显存不够".to_string())),
+            Err("显存不够".to_string()),
+            "第一次起不来：交起不来的那一句"
+        );
+        assert!(!slot.is_started(), "起不来不留着");
+        assert_eq!(
+            slot.started_or_start(never_lost, || start_as(1)),
+            Ok(1),
+            "下一次要时再起，起得来"
+        );
+        assert_eq!(
+            slot.started_or_start(never_lost, || start_as(2)),
+            Ok(1),
+            "起好了留着：后面要的拿同一个，不再起"
+        );
+        assert_eq!(starts.load(Ordering::SeqCst), 1);
+        assert_eq!(
+            slot.started_or_start(|device: &u32| *device == 1, || start_as(3)),
+            Ok(3),
+            "留着的那个丢了：重起一个"
         );
+        assert_eq!(starts.load(Ordering::SeqCst), 2);
+        assert!(slot.is_started());
     }
 }
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -3,15 +3,21 @@
 //! 同一批在 CPU 上再算一遍逐个对拍，对不上就判红（`GpuCpuDisagreement`）；走树、恢复、记录核对器照旧在 CPU 上。
 //! 着色器与多卡枚举取自 E163（GPU多卡算单元校验和） 的装置（那边两卡 2560/2560 与 CPU 逐个相同）。
 //! 分工按显卡空闲显存（`nvidia-smi` 的 memory.free，按 PCI 总线号对上 wgpu 的适配器），空闲显存低于门槛的卡不用；
-//! GPU 用例共用的卡池（[`gpu_test_start::GpuCardPool`]）不看这个门槛，能不能借由跨进程的显存账判（第 10t 批 K4-3）。
+//! GPU 用例共用的卡池（[`gpu_test_start::GpuCardPool`]）不看这个门槛，能不能借由这个进程的显存账在借的那一刻判
+//! （`singlefs_harness::gpu_memory_ledger` 文件头：只记这个进程借的，借不到不等）。
+//! 每个设备起的时候装上 wgpu 没接住的错误与设备丢失的回调（[`DeviceConditionOfACard`]）：别的线程上报的 Out of Memory
+//! （判器内核在另起的线程上编）记在设备上，接 Out of Memory 的那一层照它认出是显存不够（第 10x 批代码轮判决 K1-3）。
 
 use singlefs_checker::{
     crc32_castagnoli_table, unit_checksum_layout, UNIT_HEADER_CHECKSUM_BYTE_OFFSET,
 };
+use singlefs_harness::gpu_memory_ledger::{
+    pci_bus_identifier_without_the_domain, NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
+};
 use singlefs_harness::sha256::DIGEST_BYTES;
 
 /// 空闲显存低于这个数（MiB）的卡不用：本机有卡被别的服务占着时只剩几百 MiB，起上下文都难（E163 M2 那张卡 141 MiB 起不来）。
-/// 只管 [`usable_gpu_cards`] 与 [`gpu_cards_by_priority`]；卡池按账判，被先起的进程订满的卡照样进池、借的时候等。
+/// 只管 [`usable_gpu_cards`] 与 [`gpu_cards_by_priority`]；卡池按账判，此刻被别的进程占着的卡照样进池，借的那一刻借不到就报本次未跑（不等）。
 pub const MINIMUM_FREE_MEMORY_MEBIBYTES: u64 = 2048;
 /// 一个 GPU 槽装一段字节：最大 32 KiB（数据单元的长度），短的按长度截断。
 const SLOT_BYTES: usize = 32_768;
@@ -91,21 +97,151 @@
     pipeline: wgpu::ComputePipeline,
     /// 这个设备上编好的管线，几个克隆共用：同一套源码在同一个设备上只编一次。
     compiled_pipelines_by_source_digest: std::sync::Arc<CompiledPipelinesBySourceDigest>,
+    /// 这个设备上 wgpu 报过的 Out of Memory 与设备丢没丢，几个克隆共用（[`start_the_card`] 装的回调往里记）。
+    device_condition: std::sync::Arc<DeviceConditionOfACard>,
+}
+
+/// 一个设备上 wgpu 报过的 Out of Memory（错误范围接住的与没接住的都记）与设备丢没丢。接 Out of Memory 的那一层
+/// （崩溃放量产品路径 `crate::crash_amplification`）照它认出：活在别的线程上 panic、或设备丢了，是不是显存不够（第 10x 批代码轮判决 K1-3：
+/// wgpu 的错误范围按线程分，判器内核在另起的线程上编，那里的 Out of Memory 范围接不住）。
+#[derive(Default)]
+pub struct DeviceConditionOfACard {
+    out_of_memory_reports: std::sync::atomic::AtomicU64,
+    last_out_of_memory: std::sync::Mutex<Option<String>>,
+    lost: std::sync::Mutex<Option<String>>,
+}
+
+impl DeviceConditionOfACard {
+    /// 记一次 Out of Memory（`what` 说是在哪、wgpu 报的那一句）。
+    pub fn record_an_out_of_memory(&self, what: String) {
+        // 持锁只为换一个字符串，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
+        *self
+            .last_out_of_memory
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(what);
+        self.out_of_memory_reports
+            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
+    }
+
+    /// 到此刻一共记了几次 Out of Memory。
+    #[must_use]
+    pub fn out_of_memory_reports(&self) -> u64 {
+        self.out_of_memory_reports
+            .load(std::sync::atomic::Ordering::SeqCst)
+    }
+
+    /// 记设备丢了（wgpu 的设备丢失回调调它）。
+    pub fn record_the_device_lost(&self, why: String) {
+        *self
+            .lost
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(why);
+    }
+
+    /// 设备丢了的话为什么丢（没丢是 `None`）。
+    #[must_use]
+    pub fn lost_reason(&self) -> Option<String> {
+        self.lost
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+            .clone()
+    }
+
+    /// 从记到 `reports_before` 次那一刻起出了一样坏事、能不能说成是显存不够：这之后又记了 Out of Memory，交最后那一句；
+    /// 设备丢了、丢之前报过 Out of Memory（wgpu 把建管线、提交这类调用上的 Out of Memory 当致命、丢掉设备），交两样都说的一句；别的交 `None`。
+    #[must_use]
+    pub fn out_of_memory_explaining_a_failure_since(&self, reports_before: u64) -> Option<String> {
+        let last_out_of_memory = self
+            .last_out_of_memory
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+            .clone();
+        if self.out_of_memory_reports() > reports_before {
+            return Some(
+                last_out_of_memory
+                    .unwrap_or_else(|| "wgpu 报了 Out of Memory（那一句没记下）".to_string()),
+            );
+        }
+        match (self.lost_reason(), last_out_of_memory) {
+            (Some(lost), Some(last_out_of_memory)) => Some(format!(
+                "设备丢了（{lost}），丢之前报过 {last_out_of_memory}"
+            )),
+            (None, Some(_) | None) | (Some(_), None) => None,
+        }
+    }
+}
+
+/// wgpu 没接住的错误（装在每个设备上，[`start_the_card`]）：Out of Memory 先记进这个设备的情况，再与别的错一样照 wgpu 默认的做法 panic
+/// （缓冲、管线没建成，往下跑判出来的也不作数）。
+fn handle_an_uncaptured_wgpu_error(condition: &DeviceConditionOfACard, error: wgpu::Error) {
+    match &error {
+        wgpu::Error::OutOfMemory { .. } => {
+            condition.record_an_out_of_memory(format!("没接住的 {error}"));
+        }
+        wgpu::Error::Validation { .. } | wgpu::Error::Internal { .. } => {}
+    }
+    panic!("wgpu error: {error}\n")
+}
+
+/// 一张卡起不来的两种。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum CardStartFailure {
+    /// 起设备、编单元校验和的管线时显存不够（`request_device` 交回 wgpu 的 Out of Memory，或编管线时错误范围接到 Out of Memory）：
+    /// 卡池的调用方报本次未跑（第 10x 批代码轮判决 K1-3）。
+    OutOfGpuMemory { reason: String },
+    /// 别的原因起不来（驱动坏了、适配器不给要的上限）。
+    DeviceNotStarted { reason: String },
+}
+
+/// `request_device` 交回的错是哪一种：与 wgpu 的「设备显存不够」那一句（`wgpu_core::device::DeviceError::OutOfMemory`）相同的是显存不够。
+/// wgpu 不公开这个错的成员，只能比它说的那一句；那一句取自 wgpu 自己，不在这里另抄一份。
+fn failure_of_a_device_request(
+    error: &wgpu::RequestDeviceError,
+    which_card: &str,
+) -> CardStartFailure {
+    let reason = format!("{which_card}上起不了设备（{error}）");
+    if error.to_string() == wgpu::wgc::device::DeviceError::OutOfMemory.to_string() {
+        CardStartFailure::OutOfGpuMemory { reason }
+    } else {
+        CardStartFailure::DeviceNotStarted { reason }
+    }
 }
 
-/// `nvidia-smi` 报的一张卡：序号、空闲显存与显存总量（MiB）。
+/// 这一个设备归谁：卡池的设备走这个进程的显存账；别的（[`gpu_cards_by_priority`]、[`usable_gpu_cards`] 起的）不走，
+/// 起了就记下它在哪张卡上（[`has_devices_started_outside_the_card_pool_on_the_bus`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum DeviceOwnerOnTheCard {
+    TheCardPool,
+    OutsideTheCardPool,
+}
+
+/// 这个进程在哪几张卡（PCI 总线号）上起过不走卡池账的设备。只增不减：那几个设备什么时候丢掉、卡上什么时候还，这里不知道，
+/// 记着就照「分不出」算（只会少借）。是这个进程的事实，所以是进程里一份（第 10x 批代码轮判决 K1-1，攻方乙形）。
+static BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL: std::sync::Mutex<
+    std::collections::BTreeSet<String>,
+> = std::sync::Mutex::new(std::collections::BTreeSet::new());
+
+/// 这个进程在这张卡（PCI 总线号，[`pci_bus_identifier_without_the_domain`] 写法）上起过不走卡池账的设备没有：起过，卡池读这张卡时
+/// 这个进程占着的那一份分不出哪些是借据建的（`singlefs_harness::gpu_memory_ledger::OccupiedByThisProcess::AlsoOutsideTheBooks`）。
+fn has_devices_started_outside_the_card_pool_on_the_bus(bus: &str) -> bool {
+    BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL
+        .lock()
+        .unwrap_or_else(std::sync::PoisonError::into_inner)
+        .contains(bus)
+}
+
+/// `nvidia-smi` 报的一张卡：序号与空闲显存（MiB）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct ReportedCard {
     index: u32,
     free_memory_mebibytes: u64,
-    total_memory_mebibytes: u64,
 }
 
-/// `nvidia-smi` 报的每张卡：PCI 总线号（小写、去掉前导的 0 域）→ 序号、空闲显存与显存总量。起不来 `nvidia-smi` 就是空表。
+/// `nvidia-smi` 报的每张卡：PCI 总线号（小写、去掉前导的 0 域）→ 序号与空闲显存。起不来 `nvidia-smi` 就是空表。
 fn reported_cards_by_pci_bus() -> std::collections::BTreeMap<String, ReportedCard> {
     let Ok(output) = std::process::Command::new("nvidia-smi")
         .args([
-            "--query-gpu=pci.bus_id,index,memory.free,memory.total",
+            "--query-gpu=pci.bus_id,index,memory.free",
             "--format=csv,noheader,nounits",
         ])
         .output()
@@ -119,59 +255,35 @@
             let bus = fields.next()?;
             let index = fields.next()?.trim().parse().ok()?;
             let free_memory_mebibytes = fields.next()?.trim().parse().ok()?;
-            let total_memory_mebibytes = fields.next()?.trim().parse().ok()?;
             Some((
-                normalized_bus(bus),
+                pci_bus_identifier_without_the_domain(bus),
                 ReportedCard {
                     index,
                     free_memory_mebibytes,
-                    total_memory_mebibytes,
                 },
             ))
         })
         .collect()
 }
 
-/// `nvidia-smi` 按进程报的显存里这个进程占着的：PCI 总线号 → MiB（同一张卡上几行的加起来）。起不来 `nvidia-smi` 交 `None`；
-/// 这个进程在某张卡上没被列出（没起设备，或驱动不按进程报 Vulkan 的上下文），那张卡不在表里。
-fn memory_used_by_this_process_on_each_bus() -> Option<std::collections::BTreeMap<String, u64>> {
+/// 卡池读显存用的那一份报告：跑一次 `nvidia-smi`（`singlefs_harness::gpu_memory_ledger::NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS`），
+/// 每张卡的显存与每个进程占的出自这一次。
+///
+/// # Errors
+/// `nvidia-smi` 起不来或退了非 0。
+fn nvidia_smi_memory_and_processes_report() -> Result<String, String> {
     let output = std::process::Command::new("nvidia-smi")
-        .args([
-            "--query-compute-apps=pid,gpu_bus_id,used_memory",
-            "--format=csv,noheader,nounits",
-        ])
+        .args(NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS)
         .output()
-        .ok()?;
+        .map_err(|error| format!("nvidia-smi 起不来：{error}"))?;
     if !output.status.success() {
-        return None;
-    }
-    let this_process = std::process::id();
-    let mut used_by_bus = std::collections::BTreeMap::new();
-    for line in String::from_utf8_lossy(&output.stdout).lines() {
-        let mut fields = line.split(',');
-        let (Some(process_identifier), Some(bus), Some(used)) =
-            (fields.next(), fields.next(), fields.next())
-        else {
-            continue;
-        };
-        let (Ok(process_identifier), Ok(used_mebibytes)) = (
-            process_identifier.trim().parse::<u32>(),
-            used.trim().parse::<u64>(),
-        ) else {
-            continue;
-        };
-        if process_identifier == this_process {
-            *used_by_bus.entry(normalized_bus(bus)).or_insert(0u64) += used_mebibytes;
-        }
+        return Err(format!(
+            "nvidia-smi {NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS:?} 退了 {}：{}",
+            output.status,
+            String::from_utf8_lossy(&output.stderr).trim()
+        ));
     }
-    Some(used_by_bus)
-}
-
-/// 总线号只留 `总线:设备.功能` 那一段、小写：`00000000:01:00.0` 与 `0000:01:00.0` 认成同一张卡。
-fn normalized_bus(text: &str) -> String {
-    let lowered = text.trim().to_ascii_lowercase();
-    let parts: Vec<&str> = lowered.split(':').collect();
-    parts[parts.len().saturating_sub(2)..].join(":")
+    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
 }
 
 /// Vulkan 下的一张 NVIDIA 独显与 `nvidia-smi` 报的它（序号、那一刻的空闲显存）：还没起设备。
@@ -201,7 +313,8 @@
             {
                 return None;
             }
-            let pci_bus_identifier = normalized_bus(&information.device_pci_bus_id);
+            let pci_bus_identifier =
+                pci_bus_identifier_without_the_domain(&information.device_pci_bus_id);
             let reported = *reported_by_bus.get(&pci_bus_identifier)?;
             Some(DiscoveredCard {
                 adapter,
@@ -213,8 +326,17 @@
         .collect()
 }
 
-/// 在找到的这张卡上起设备、编单元校验和的管线；起不来交 `None`。不看空闲显存（看不看由调用方定）。
-fn start_the_card(discovered: &DiscoveredCard) -> Option<GpuCard> {
+/// 在找到的这张卡上起设备、编单元校验和的管线；起不来交为什么（显存不够与别的分开，[`CardStartFailure`]）。不看空闲显存（看不看由调用方定）。
+/// 设备上装好接 wgpu 没接住的错误与设备丢失的回调（[`DeviceConditionOfACard`]）；不归卡池的设备起好了记下它在哪张卡上
+/// （[`has_devices_started_outside_the_card_pool_on_the_bus`]）。
+fn start_the_card(
+    discovered: &DiscoveredCard,
+    owner: DeviceOwnerOnTheCard,
+) -> Result<GpuCard, CardStartFailure> {
+    let which_card = format!(
+        "序号 {} 的卡（{}）",
+        discovered.reported.index, discovered.name
+    );
     let adapter = &discovered.adapter;
     let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
         label: Some("singlefs-gpu-unit-checks"),
@@ -226,7 +348,18 @@
         memory_hints: wgpu::MemoryHints::Performance,
         trace: wgpu::Trace::Off,
     }))
-    .ok()?;
+    .map_err(|error| failure_of_a_device_request(&error, &which_card))?;
+    let device_condition = std::sync::Arc::new(DeviceConditionOfACard::default());
+    let condition_for_uncaptured_errors = std::sync::Arc::clone(&device_condition);
+    device.on_uncaptured_error(std::sync::Arc::new(move |error: wgpu::Error| {
+        handle_an_uncaptured_wgpu_error(&condition_for_uncaptured_errors, error);
+    }));
+    let condition_for_the_loss = std::sync::Arc::clone(&device_condition);
+    device.set_device_lost_callback(move |reason, message| {
+        condition_for_the_loss.record_the_device_lost(format!("{reason:?}：{message}"));
+    });
+    // 编单元校验和的管线：这条线程上推一个 Out of Memory 范围，卡被别的进程占满时显存不够报成起不来，不让默认处理 panic
+    let out_of_memory_scope = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
     let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
         label: Some("crc32c"),
         source: wgpu::ShaderSource::Wgsl(CRC32C_SHADER_SOURCE.into()),
@@ -239,7 +372,21 @@
         compilation_options: wgpu::PipelineCompilationOptions::default(),
         cache: None,
     });
-    Some(GpuCard {
+    if let Some(error) = pollster::block_on(out_of_memory_scope.pop()) {
+        let reason = format!("{which_card}上编单元校验和的管线时 wgpu 报了 {error}");
+        device_condition.record_an_out_of_memory(reason.clone());
+        return Err(CardStartFailure::OutOfGpuMemory { reason });
+    }
+    match owner {
+        DeviceOwnerOnTheCard::TheCardPool => {}
+        DeviceOwnerOnTheCard::OutsideTheCardPool => {
+            BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL
+                .lock()
+                .unwrap_or_else(std::sync::PoisonError::into_inner)
+                .insert(discovered.pci_bus_identifier.clone());
+        }
+    }
+    Ok(GpuCard {
         name: discovered.name.clone(),
         pci_bus_identifier: discovered.pci_bus_identifier.clone(),
         index: discovered.reported.index,
@@ -249,6 +396,7 @@
         queue,
         pipeline,
         compiled_pipelines_by_source_digest: std::sync::Arc::default(),
+        device_condition,
     })
 }
 
@@ -264,7 +412,9 @@
         .filter(|discovered| {
             discovered.reported.free_memory_mebibytes >= MINIMUM_FREE_MEMORY_MEBIBYTES
         })
-        .filter_map(start_the_card)
+        .filter_map(|discovered| {
+            start_the_card(discovered, DeviceOwnerOnTheCard::OutsideTheCardPool).ok()
+        })
         .collect();
     let mut cards = Vec::with_capacity(configured.len());
     for (index, quota) in configured {
@@ -291,7 +441,9 @@
         .filter(|discovered| {
             discovered.reported.free_memory_mebibytes >= MINIMUM_FREE_MEMORY_MEBIBYTES
         })
-        .filter_map(start_the_card)
+        .filter_map(|discovered| {
+            start_the_card(discovered, DeviceOwnerOnTheCard::OutsideTheCardPool).ok()
+        })
         .collect();
     cards.sort_by_key(|card| std::cmp::Reverse(card.free_memory_mebibytes));
     cards
@@ -390,6 +542,12 @@
         &self.queue
     }
 
+    /// 这个设备上 wgpu 报过的 Out of Memory 与设备丢没丢（几个克隆共用一份）。
+    #[must_use]
+    pub fn device_condition(&self) -> &DeviceConditionOfACard {
+        &self.device_condition
+    }
+
     /// 这个设备上按 `source_digest` 编好的那组管线：这个设备（这张卡与它的每个克隆）上第一次要这个摘要时调 `compile` 编，
     /// 别的线程同时要同一个摘要就等这一次编完、拿同一组；以后再要直接拿。管线绑在设备上，另起的设备（另一次 [`usable_gpu_cards`]）另有一份。
     /// 冷编一套判器内核要几分钟、驱动编的时候吃几 GiB 主机内存：几十条用例共用一张卡时，每条各编一遍既慢又撞主机内存上限。
@@ -808,7 +966,7 @@
         }
     }
 
-    /// 这条用例报「本次未跑」时打的那一行（记号与写法在 harness 档一处定义，崩溃放量产品路径借草稿区等满了也打它：
+    /// 这条用例报「本次未跑」时打的那一行（记号与写法在 harness 档一处定义，崩溃放量产品路径此刻借不到、卡上报 Out of Memory 也打它：
     /// `singlefs_harness::gpu_memory_ledger::gpu_not_run_line`）。
     #[must_use]
     pub fn not_run_line(test_name: &str, reason: &str) -> String {
@@ -965,15 +1123,15 @@
     /// GPU 用例共用的卡池（[`card_pool::GpuCardPool`]）：只开 `gpu` 也编得进（显卡配置的读法在 `crate::gpu_card_configuration`）。
     pub use card_pool::{
         card_pool_started_in_this_process, gpu_card_pool_or_report_not_run,
-        CardOfThePoolForTheProductPath, GpuCardLease, GpuCardPool, ScratchLeaseOnACard,
-        CARD_QUOTA_ENVIRONMENT_VARIABLE,
+        CardOfThePoolForTheProductPath, CardPoolLeaseRefusal, GpuCardLease, GpuCardPool,
+        ScratchLeaseOnACard, CARD_QUOTA_ENVIRONMENT_VARIABLE,
     };
 
     mod card_pool {
         use super::super::{
-            memory_budget_mebibytes_of, memory_used_by_this_process_on_each_bus,
-            nvidia_discrete_cards_reported_by_nvidia_smi, reported_cards_by_pci_bus,
-            start_the_card, DiscoveredCard, GpuCard,
+            has_devices_started_outside_the_card_pool_on_the_bus, memory_budget_mebibytes_of,
+            nvidia_discrete_cards_reported_by_nvidia_smi, nvidia_smi_memory_and_processes_report,
+            start_the_card, CardStartFailure, DeviceOwnerOnTheCard, DiscoveredCard, GpuCard,
         };
         use super::{
             decide_gpu_test_start, enumerate_vulkan_adapters, not_run_line,
@@ -985,8 +1143,10 @@
             CardHost,
         };
         use singlefs_harness::gpu_memory_ledger::{
-            card_ledger_capacity_mebibytes, reason_for_a_booking_without_room_now, BookingRefusal,
-            CardMemoryReading, GpuMemoryBooking, GpuMemoryLedger, StatesBookedOnACard,
+            card_ledger_capacity_mebibytes, card_memory_reader_of_one_report_per_reading,
+            reason_for_a_booking_without_room_now, reason_for_running_out_of_gpu_memory,
+            BookingRefusal, CardRoomForThisProcess, DeviceStartedOnDemand, GpuMemoryBooking,
+            GpuMemoryLedger, StatesBookedOnACard,
         };
         use singlefs_harness::numeric_environment_variable::{
             number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
@@ -999,12 +1159,23 @@
             "SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES";
 
         /// 池里的一张卡：照配置点了名、Vulkan 与 `nvidia-smi` 里都找得到；设备等这个进程在这张卡上借到一笔之后才起
-        /// （[`GpuCardPool::started_card`]，一个进程在一张卡上只起一次）。
+        /// （[`GpuCardPool::started_card`]）：起好了留着给后面借的用，起不来不留、下一次借到再起，设备丢了也重起
+        /// （第 10x 批代码轮判决 K1-3：起设备失败不缓存成这个进程永久的错）。
         struct CardOfThePool {
             discovered: DiscoveredCard,
             /// `GPU_CARDS_BY_PRIORITY` 配给这张卡的额度（MiB）；没配显卡优先级的是 `None`。
             configured_card_quota_mebibytes: Option<u64>,
-            started: std::sync::OnceLock<Result<GpuCard, String>>,
+            started: DeviceStartedOnDemand<GpuCard>,
+        }
+
+        /// 向卡池借、借不到的两种（崩溃放量产品路径照它报）。
+        #[derive(Clone, Debug, PartialEq, Eq)]
+        pub enum CardPoolLeaseRefusal {
+            /// 账上借不到（`singlefs_harness::gpu_memory_ledger::BookingRefusal`）。
+            Booking(BookingRefusal),
+            /// 账上借到了，那张卡上起设备时显存不够（[`CardStartFailure::OutOfGpuMemory`]）：那一笔已经还了，调用方报本次未跑。
+            /// `reason` 是「本次未跑」那一行的原因（以「显存不够：」起头）。
+            OutOfGpuMemoryStartingTheDevice { reason: String },
         }
 
         /// GPU 用例共用的卡：这台机器点名的每张卡在这个进程里一本显存账（[`GpuMemoryLedger`]，只记这个进程借的，
@@ -1096,43 +1267,41 @@
                     )
             }
 
-            /// 借到 `booking` 的那张卡起好的设备：第一次要时起（借到就登记了这个进程，先登记、后建设备），之后拿同一个；
-            /// 起不来交一句为什么。要一笔借据才起得了：没借过的卡起不了设备。
+            /// 借到 `booking` 的那张卡起好的设备：留着的设备没丢就拿它，没有（第一次要、上一次起不来）或丢了就起一个
+            /// （借到就登记了这个进程，先登记、后建设备）；起不来交为什么、不留着。要一笔借据才起得了：没借过的卡起不了设备。
             ///
             /// # Panics
             /// 这个进程还没在这张卡的账上登记（`GpuMemoryLedger::check_registered_before_starting_the_device`）。
             fn started_card(
                 &self,
                 booking: &GpuMemoryBooking<'static>,
-            ) -> Result<&GpuCard, String> {
+            ) -> Result<GpuCard, CardStartFailure> {
                 let card_position = booking.card_position();
                 let card = &self.cards[card_position];
-                card.started
-                    .get_or_init(|| {
+                card.started.started_or_start(
+                    |started: &GpuCard| started.device_condition().lost_reason().is_some(),
+                    || {
                         self.ledger
                             .check_registered_before_starting_the_device(card_position);
-                        let mut started = start_the_card(&card.discovered).ok_or_else(|| {
-                            format!(
-                                "序号 {} 的卡（{}）上起不了设备：账上登记了这个进程，卡却起不来上下文（驱动坏了，或卡被别的程序占满）",
-                                card.discovered.reported.index, card.discovered.name
-                            )
-                        })?;
+                        let mut started =
+                            start_the_card(&card.discovered, DeviceOwnerOnTheCard::TheCardPool)?;
                         self.ledger
                             .record_the_device_started_on_the_card(card_position);
                         started.memory_quota_mebibytes = card.configured_card_quota_mebibytes;
                         Ok(started)
-                    })
-                    .as_ref()
-                    .map_err(Clone::clone)
+                    },
+                )
             }
 
             /// 借 `need_mebibytes`：照卡的优先级记在第一张此刻放得下的卡上（前一张放满了才用下一张）；此刻每张都放不下
             /// （这个进程别的用例或卡上别的 GPU 用户占着）不等，打 [`not_run_line`] 那一行（原因写显存不够）、交回 `None`
             /// （调用的用例当场返回）。每借到一次打一行 `GPU_CARD_LEASE test=… index=… mebibytes=… room_mebibytes=…`。
             ///
+            /// 借到的卡上起设备时显存不够同样报「本次未跑」（原因写显存不够）。
+            ///
             /// # Panics
-            /// 要的比每张卡空着时能借的都大（配了 GPU 就一定用 GPU，放不下判红，不悄悄换小）；卡上的显存读不出；借到的卡上起不了设备；
-            /// 要 0 MiB。
+            /// 要的比每张卡空着时能借的都大（配了 GPU 就一定用 GPU，放不下判红，不悄悄换小）；卡上的显存读不出；
+            /// 借到的卡上起不了设备（显存不够之外的原因）；要 0 MiB。
             #[must_use]
             pub fn lease_card_or_report_not_run(
                 &'static self,
@@ -1149,8 +1318,9 @@
             }
 
             /// 同 [`Self::lease_card_or_report_not_run`]，此刻借不到判红而不报「本次未跑」：给还没改成报本次未跑的调用方留着
-            /// （`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`）。名字沿用那几处调用方的写法：已经不等了
-            /// （第 10x 批：只记这个进程借的，借不到不等）。
+            /// （`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`，在删除线那一批手里）。名字是过时的，已经不等了
+            /// （第 10x 批：只记这个进程借的，借不到不等）；第 10x 批代码轮判决要它改名、同进程别的用例借着时不 panic，
+            /// 要连那几处调用一起改成经 [`Self::lease_card_or_report_not_run`]（借不到报本次未跑），那份文件改完删掉这一个。
             ///
             /// # Panics
             /// 同 [`Self::lease_card_or_report_not_run`]；此刻每张卡都借不到。
@@ -1166,7 +1336,7 @@
                     })
             }
 
-            /// 借一次：借到交借来的卡，此刻借不到交一句为什么。
+            /// 借一次：借到交借来的卡，此刻借不到、借到的卡上起设备时显存不够交一句为什么。
             fn lease_card(
                 &'static self,
                 test_name: &str,
@@ -1192,7 +1362,7 @@
                         panic!("{test_name}：卡上的显存读不出：{reason}")
                     }
                 };
-                Ok(self.card_lent_for_the_booking(test_name, booking))
+                self.card_lent_for_the_booking(test_name, booking)
             }
 
             /// 同 [`Self::lease_card_or_report_not_run`]（照卡的优先级记在第一张此刻放得下的卡上），借不到交账的结局、
@@ -1200,19 +1370,23 @@
             /// （`singlefs_harness::gpu_memory_ledger::scratch_booking_refusal_outcome`：此刻借不到报本次未跑，别的判红）。
             ///
             /// # Errors
-            /// 见 `singlefs_harness::gpu_memory_ledger::BookingRefusal`。
+            /// 见 [`CardPoolLeaseRefusal`]。
             ///
             /// # Panics
-            /// 借到的卡上起不了设备；要 0 MiB。
+            /// 借到的卡上起不了设备（显存不够之外的原因）；要 0 MiB。
             pub fn lease_the_first_card_with_room_now(
                 &'static self,
                 run_name: &str,
                 need_mebibytes: u64,
-            ) -> Result<GpuCardLease, BookingRefusal> {
+            ) -> Result<GpuCardLease, CardPoolLeaseRefusal> {
                 let booking = self
                     .ledger
-                    .book_on_the_first_card_with_room(need_mebibytes)?;
-                Ok(self.card_lent_for_the_booking(run_name, booking))
+                    .book_on_the_first_card_with_room(need_mebibytes)
+                    .map_err(CardPoolLeaseRefusal::Booking)?;
+                self.card_lent_for_the_booking(run_name, booking)
+                    .map_err(
+                        |reason| CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice { reason },
+                    )
             }
 
             /// 序号是 `index`（`nvidia-smi` 给的）的那张卡在池里排第几。
@@ -1233,13 +1407,13 @@
             /// 压状态数再借（几个状态要多少显存照 `mebibytes_for_states`），借不到的卡不装
             /// （`singlefs_harness::gpu_memory_ledger::GpuMemoryLedger::book_states_on_the_cards_with_room`）。
             /// 借到的每张交成借来的卡（额度是借的量加驱动那一份，判器照它开出来的草稿区正好是借的状态数；借到了才在那张卡上起设备）；
-            /// 交回与 `card_indices` 逐张对上。
+            /// 起设备时显存不够的卡还掉那一笔、这一趟不用（同借不到的卡）。交回与 `card_indices` 逐张对上。
             ///
             /// # Errors
-            /// 见 `singlefs_harness::gpu_memory_ledger::BookingRefusal`。
+            /// 见 [`CardPoolLeaseRefusal`]：借到的每张卡起设备时都显存不够是 [`CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice`]。
             ///
             /// # Panics
-            /// 有一张不是池里的卡；借到的卡上起不了设备。
+            /// 有一张不是池里的卡；借到的卡上起不了设备（显存不够之外的原因）。
             pub fn book_the_scratch_of_the_cards_with_room_now(
                 &'static self,
                 run_name: &str,
@@ -1247,46 +1421,66 @@
                 pending_states: u64,
                 states_each_card_holds: &[u64],
                 mebibytes_for_states: &dyn Fn(u64) -> u64,
-            ) -> Result<Vec<Option<ScratchLeaseOnACard>>, BookingRefusal> {
+            ) -> Result<Vec<Option<ScratchLeaseOnACard>>, CardPoolLeaseRefusal> {
                 let card_positions: Vec<usize> = card_indices
                     .iter()
                     .map(|index| self.position_of_the_card_with_index(*index))
                     .collect();
-                Ok(self
+                let booked = self
                     .ledger
                     .book_states_on_the_cards_with_room(
                         &card_positions,
                         pending_states,
                         states_each_card_holds,
                         mebibytes_for_states,
-                    )?
+                    )
+                    .map_err(CardPoolLeaseRefusal::Booking)?;
+                let mut first_card_short_of_memory = None;
+                let leases: Vec<Option<ScratchLeaseOnACard>> = booked
                     .into_iter()
                     .map(|booked| {
-                        booked.map(
-                            |StatesBookedOnACard { states, booking }| ScratchLeaseOnACard {
-                                lease: self.card_lent_for_the_booking(run_name, booking),
-                                states,
-                            },
-                        )
+                        let StatesBookedOnACard { states, booking } = booked?;
+                        match self.card_lent_for_the_booking(run_name, booking) {
+                            Ok(lease) => Some(ScratchLeaseOnACard { lease, states }),
+                            Err(reason) => {
+                                first_card_short_of_memory.get_or_insert(reason);
+                                None
+                            }
+                        }
                     })
-                    .collect())
+                    .collect();
+                if leases.iter().all(Option::is_none) {
+                    return Err(CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice {
+                        reason: first_card_short_of_memory.expect(
+                            "账交回借到了至少一张：一张都没交成借来的卡，只能是借到的每张起设备时都显存不够",
+                        ),
+                    });
+                }
+                Ok(leases)
             }
 
-            /// 记上了的那一笔交成借来的卡：在那张卡上起设备（第一次借到时，[`Self::started_card`]），与池里那张共用设备，
-            /// 额度与空闲改成借的量加驱动那一份；打一行 `GPU_CARD_LEASE`。
+            /// 记上了的那一笔交成借来的卡：在那张卡上起设备（还没有起好的设备时，[`Self::started_card`]），与池里那张共用设备，
+            /// 额度与空闲改成借的量加驱动那一份；打一行 `GPU_CARD_LEASE`。起设备时显存不够：还掉这一笔，交「本次未跑」那一行的原因。
             ///
             /// # Panics
-            /// 那张卡上起不了设备。
+            /// 那张卡上起不了设备（显存不够之外的原因：驱动坏了、适配器不给要的上限）。
             fn card_lent_for_the_booking(
                 &'static self,
                 test_name: &str,
                 booking: GpuMemoryBooking<'static>,
-            ) -> GpuCardLease {
+            ) -> Result<GpuCardLease, String> {
                 let need_mebibytes = booking.mebibytes();
-                let mut card = self
-                    .started_card(&booking)
-                    .unwrap_or_else(|reason| panic!("{test_name}：{reason}"))
-                    .clone();
+                let mut card = match self.started_card(&booking) {
+                    Ok(card) => card,
+                    Err(CardStartFailure::OutOfGpuMemory { reason }) => {
+                        return Err(reason_for_running_out_of_gpu_memory(&format!(
+                            "起设备时（{reason}）"
+                        )));
+                    }
+                    Err(CardStartFailure::DeviceNotStarted { reason }) => {
+                        panic!("{test_name}：{reason}：账上登记了这个进程，卡却起不来上下文（驱动坏了）")
+                    }
+                };
                 card.memory_quota_mebibytes = Some(need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER);
                 card.free_memory_mebibytes = need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER;
                 eprintln!(
@@ -1295,26 +1489,26 @@
                     card.index,
                     booking.card_room_mebibytes_when_booked()
                 );
-                GpuCardLease { card, booking }
+                Ok(GpuCardLease { card, booking })
             }
 
-            /// 这个进程此刻在 `card_indices` 这几张卡（都是池里的卡）上各能借多少（MiB，现读一次；与 `card_indices` 逐张对上）。
+            /// 这个进程此刻与这几张卡空着时，在 `card_indices` 这几张卡（都是池里的卡）上各能借多少（MiB，现读一次；与 `card_indices` 逐张对上）。
             ///
             /// # Errors
             /// 卡上的显存读不出。
             ///
             /// # Panics
             /// 有一张不是池里的卡。
-            pub fn card_rooms_mebibytes_now_of(
+            pub fn card_rooms_now_and_when_empty_of(
                 &self,
                 card_indices: &[u32],
-            ) -> Result<Vec<u64>, String> {
+            ) -> Result<Vec<CardRoomForThisProcess>, String> {
                 let card_positions: Vec<usize> = card_indices
                     .iter()
                     .map(|index| self.position_of_the_card_with_index(*index))
                     .collect();
                 self.ledger
-                    .card_rooms_mebibytes_now(&card_positions)
+                    .card_rooms_now_and_when_empty(&card_positions)
                     .map_err(|refusal| format!("读不出卡上此刻的显存：{refusal:?}"))
             }
 
@@ -1338,8 +1532,7 @@
             pub fn is_the_device_started_on_the_card(&self, index: u32) -> bool {
                 self.cards[self.position_of_the_card_with_index(index)]
                     .started
-                    .get()
-                    .is_some_and(Result::is_ok)
+                    .is_started()
             }
 
             /// 池里每张卡（按优先级）的序号与配置给它的额度（MiB，没配显卡优先级的是 `None`），不起设备。
@@ -1471,33 +1664,6 @@
             }
         }
 
-        /// 池里每张卡此刻的显存（与 `buses` 逐张对上，`buses` 是池里每张卡的 PCI 总线号）：一次 `nvidia-smi` 读空闲与总量，
-        /// 一次按进程读这个进程占着的（读不出、这个进程在那张卡上没被列出的是 `None`）。
-        ///
-        /// # Errors
-        /// `nvidia-smi` 里读不到某张卡。
-        fn memory_of_the_cards_on_these_buses(
-            buses: &[String],
-        ) -> Result<Vec<CardMemoryReading>, String> {
-            let reported_by_bus = reported_cards_by_pci_bus();
-            let used_by_this_process = memory_used_by_this_process_on_each_bus();
-            buses
-                .iter()
-                .map(|bus| {
-                    let reported = reported_by_bus
-                        .get(bus)
-                        .ok_or_else(|| format!("nvidia-smi 里读不到总线 {bus} 那张卡的显存"))?;
-                    Ok(CardMemoryReading {
-                        free_mebibytes: reported.free_memory_mebibytes,
-                        total_mebibytes: reported.total_memory_mebibytes,
-                        occupied_by_this_process_mebibytes: used_by_this_process
-                            .as_ref()
-                            .and_then(|used_by_bus| used_by_bus.get(bus).copied()),
-                    })
-                })
-                .collect()
-        }
-
         /// 开点名的那几张卡的账（这个进程里，不读卡、不起设备）；交回池。
         fn open_the_card_pool(named: Vec<(DiscoveredCard, Option<u64>)>) -> GpuCardPool {
             let card_quota = number_from_the_variable_value_panicking_when_unreadable::<u64>(
@@ -1510,9 +1676,15 @@
                 .collect();
             let configured_card_quotas: Vec<Option<u64>> =
                 named.iter().map(|(_, quota)| *quota).collect();
+            // 每次借读一份 nvidia-smi 报告，空闲与这个进程占着的出自同一份（第 10x 批代码轮判决 K1-1）
             let ledger = GpuMemoryLedger::new(
                 named.len(),
-                Box::new(move || memory_of_the_cards_on_these_buses(&buses)),
+                card_memory_reader_of_one_report_per_reading(
+                    Box::new(nvidia_smi_memory_and_processes_report),
+                    std::process::id(),
+                    buses,
+                    Box::new(has_devices_started_outside_the_card_pool_on_the_bus),
+                ),
                 Box::new(move |position, may_hold_mebibytes| {
                     let configured = configured_card_quotas[position];
                     card_ledger_capacity_mebibytes(
@@ -1531,7 +1703,7 @@
                     |(discovered, configured_card_quota_mebibytes)| CardOfThePool {
                         discovered,
                         configured_card_quota_mebibytes,
-                        started: std::sync::OnceLock::new(),
+                        started: DeviceStartedOnDemand::new(),
                     },
                 )
                 .collect();
@@ -1624,6 +1796,10 @@
         decide_gpu_test_start, read_pci_nvidia_display_controllers_on_the_bus, GpuTestStart,
         PciNvidiaDisplayControllers, PCI_DEVICES_SUBDIRECTORY_NAME,
     };
+    use super::{
+        failure_of_a_device_request, handle_an_uncaptured_wgpu_error, CardStartFailure,
+        DeviceConditionOfACard,
+    };
 
     /// 假的 sysfs 里 PCI 总线那一截的样子。
     enum FakePciBus<'devices> {
@@ -1730,4 +1906,101 @@
             "总线看得见而某个设备的 vendor 读不出，分不清没卡与驱动坏，照旧判红、不报「本次未跑」；读到 {read:?}，判成了 {start:?}"
         );
     }
+
+    /// 第 10x 批代码轮判决 K1-3（攻方 R1）：`request_device` 交回 wgpu 的设备显存不够是显存不够（卡池报本次未跑），
+    /// 设备丢了这类别的错是起不来（判红）。错是照 wgpu 的错类型造的，不要显卡。
+    #[test]
+    fn a_device_request_refused_for_lack_of_memory_is_told_apart_from_other_refusals() {
+        let refused = |device_error| {
+            failure_of_a_device_request(
+                &wgpu::RequestDeviceError::from(wgpu::wgc::instance::RequestDeviceError::Device(
+                    device_error,
+                )),
+                "序号 0 的卡",
+            )
+        };
+        assert!(
+            matches!(
+                refused(wgpu::wgc::device::DeviceError::OutOfMemory),
+                CardStartFailure::OutOfGpuMemory { .. }
+            ),
+            "起设备时显存不够：报本次未跑那一种"
+        );
+        assert!(
+            matches!(
+                refused(wgpu::wgc::device::DeviceError::Lost),
+                CardStartFailure::DeviceNotStarted { .. }
+            ),
+            "别的错：判红那一种"
+        );
+    }
+
+    /// 第 10x 批代码轮判决 K1-3（攻方 R2）：wgpu 没接住的 Out of Memory（判器内核在另起的线程上编，那里没有错误范围）先记在设备上、
+    /// 再照默认的做法 panic；校验错照样 panic、不记成显存不够。错是照 wgpu 的错类型造的，不要显卡。
+    #[test]
+    fn an_uncaptured_out_of_memory_is_recorded_on_the_device_before_the_panic() {
+        let condition = DeviceConditionOfACard::default();
+        assert!(
+            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+                handle_an_uncaptured_wgpu_error(
+                    &condition,
+                    wgpu::Error::OutOfMemory {
+                        source: Box::new(std::fmt::Error),
+                    },
+                );
+            }))
+            .is_err(),
+            "没接住的 Out of Memory 照默认的做法 panic"
+        );
+        assert_eq!(condition.out_of_memory_reports(), 1, "panic 之前记在设备上");
+        assert!(
+            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+                handle_an_uncaptured_wgpu_error(
+                    &condition,
+                    wgpu::Error::Validation {
+                        source: Box::new(std::fmt::Error),
+                        description: "绑定组布局对不上".to_string(),
+                    },
+                );
+            }))
+            .is_err(),
+            "校验错照样 panic"
+        );
+        assert_eq!(condition.out_of_memory_reports(), 1, "校验错不记成显存不够");
+    }
+
+    /// 第 10x 批代码轮判决 K1-3（攻方 R2、R4）：从某一刻起又记了 Out of Memory，或设备丢了而丢之前报过 Out of Memory，说得成显存不够；
+    /// 什么都没记、或设备丢了而从没报过 Out of Memory，说不成。
+    #[test]
+    fn a_failure_is_put_down_to_gpu_memory_only_after_an_out_of_memory_on_the_device() {
+        let condition = DeviceConditionOfACard::default();
+        assert_eq!(condition.out_of_memory_explaining_a_failure_since(0), None);
+        condition.record_the_device_lost("Unknown：驱动重置".to_string());
+        assert_eq!(
+            condition.out_of_memory_explaining_a_failure_since(0),
+            None,
+            "设备丢了而从没报过 Out of Memory：不是显存不够"
+        );
+        let fresh = DeviceConditionOfACard::default();
+        let reports_before = fresh.out_of_memory_reports();
+        fresh.record_an_out_of_memory("另起的线程上编恢复内核时没接住的 Out of Memory".to_string());
+        assert_eq!(
+            fresh.out_of_memory_explaining_a_failure_since(reports_before),
+            Some("另起的线程上编恢复内核时没接住的 Out of Memory".to_string()),
+            "这之后记了 Out of Memory：说成显存不够"
+        );
+        let reports_after = fresh.out_of_memory_reports();
+        assert_eq!(
+            fresh.out_of_memory_explaining_a_failure_since(reports_after),
+            None,
+            "这之后没再记、设备也没丢：说不成"
+        );
+        fresh.record_the_device_lost("Unknown：Out of memory".to_string());
+        assert!(
+            fresh
+                .out_of_memory_explaining_a_failure_since(reports_after)
+                .is_some(),
+            "设备丢了、丢之前报过 Out of Memory：说成显存不够"
+        );
+    }
 }
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -22,6 +22,8 @@
 //!    这张卡能借给这个进程的量压（不照起池那一刻的空闲，第 10t 批代码轮攻方 S5）。几张卡按卡的先后每张当场试一次，借不到的卡这一趟
 //!    不用（不装状态，它那一份留给后面的卡），一张都借不到就照对拍用例那一路打「本次未跑」那一行（同一个记号，原因写显存不够），
 //!    这一趟照叫停的样子交回（`stopped_early`，判过的块在库里，分项驱动把这一项记成本次未跑；派出的项全没判完时整趟退 77）。不等。
+//!    每张卡此刻都带不了一个状态而至少一张空着时带得了，同样是借不到、报本次未跑，不去借也不判红；空着时也带不了才判红
+//!    （`singlefs_harness::gpu_memory_ledger::decide_scratch_before_booking`，第 10x 批代码轮判决 K2-2）。
 //!    只排派活计划那一趟试判前同样借。
 //!    第 ② 段核对（`crash_verify_gpu`）与第 ① 段由 CPU 判时的单元校验和对拍（`gpu_unit_checks`）在卡上建的缓冲同样进账：照卡的先后借第一张
 //!    此刻放得下的，用借来的那张卡算，借不到报本次未跑（判过的块在库里）。量照
@@ -36,8 +38,11 @@
 //!    驱动的上下文与编好的管线算在每张卡第一次借时的驱动那一份里，不另记。
 //!    账只记这个进程借的：两个进程同一刻读到同一份空闲、一起借超了，由运行时兜底：第 ① 段卡上建判器、判一块时 wgpu 报 Out of Memory，
 //!    这张卡停下、这一块不入库、这一趟报本次未跑（原因写显存不够，`singlefs_harness::gpu_memory_ledger::reason_for_running_out_of_gpu_memory`）；
-//!    第 ② 段在卡上核时同样。单元校验和对拍几条线程一起派活，wgpu 的错误范围按线程分，一个范围接不住几条线程，这一段没接：
-//!    那里的 Out of Memory 照旧让这一趟红。
+//!    第 ② 段在卡上核时、只排派活计划那一趟试判时、卡池在借到的卡上起设备与编单元校验和管线时同样（第 10x 批代码轮判决 K1-3）。
+//!    判器内核在另起的线程上编，那里的 Out of Memory 不进卡线程的错误范围：每个设备上装了接 wgpu 没接住的错误的回调，Out of Memory
+//!    记在设备上（`crate::gpu_unit_checks::DeviceConditionOfACard`），卡线程上的活 panic 了就照它认是不是显存不够
+//!    （`decide_the_work_on_the_card`）；设备因为 Out of Memory 丢了，卡池下一次借这张卡时重起设备。
+//!    单元校验和对拍几条线程一起派活，那一段没包进接 Out of Memory 的那一层：那里的 Out of Memory 记在设备上、照旧让这一趟红。
 //!    没起过卡池的进程照旧不进账（[`GpuCardsSource::DevicesStartedForThisRun`]：这一趟自己在点名的卡上起设备），调用方是这几种：
 //!    崩溃放量分项驱动（`scripts/crash/crash-amplification-item.sh`）起的、只有经 [`run_from_environment`] 跑的用例而没有一条取卡池的测试二进制
 //!    （`tests/crash_amplification_fixed_script_through_rollback.rs`、`…_through_unmount.rs`、`crash_amplification_two_flows_share_their_prefix.rs`、
@@ -56,10 +61,12 @@
 use singlefs_core::root_record::ROOT_MAGIC;
 #[cfg(feature = "gpu")]
 use singlefs_harness::gpu_memory_ledger::{
-    gpu_not_run_line, mebibytes_of_buffers_held_at_once, most_states_whose_need_fits_the_capacity,
+    decide_scratch_before_booking, gpu_not_run_line, mebibytes_of_buffers_held_at_once,
+    most_states_whose_need_fits_the_capacity, reason_for_a_booking_without_room_now,
     reason_for_running_out_of_gpu_memory, scratch_booking_refusal_outcome,
     scratch_states_to_book_on_each_card, BookingRefusal, NeedLargerThanEveryCard,
-    ScratchBookingRefusalOutcome, ScratchStatesToBook, PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
+    ScratchBeforeBooking, ScratchBookingRefusalOutcome, ScratchStatesToBook,
+    PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
 };
 use singlefs_harness::hexadecimal::hexadecimal_text;
 use singlefs_harness::memory_pool::{
@@ -1753,6 +1760,8 @@
     /// 起过卡池的进程，这张卡的判器建好那一刻核账：草稿区那一笔借据还在账上没有（第 10t 批代码轮判决 K4-1：借了又在判之前还掉要红）；
     /// 不进账是 `None`。
     pub scratch_lease_on_the_books_when_the_judge_was_built: Option<bool>,
+    /// 同上，这张卡领的最后一块判完那一刻再核一次（第 10x 批代码轮判决 K3-1：建好之后、判完之前还掉也要红）；不进账是 `None`。
+    pub scratch_lease_on_the_books_when_the_judging_finished: Option<bool>,
 }
 
 impl GpuCardOutcome {
@@ -1975,26 +1984,76 @@
     }
 }
 
+/// 卡上一段活做完之后的结局（[`decide_the_work_on_the_card`]）。
+#[cfg(feature = "gpu")]
+enum WorkOnTheCard<Outcome> {
+    Done(Outcome),
+    /// 显存不够：`detail` 说 wgpu 报的那一句（这条线程上的错误范围接住的，或别的线程上没接住、记在设备上的）。
+    OutOfMemory {
+        detail: String,
+    },
+    /// 活 panic 了，又说不成显存不够：照样往外抛。
+    Panicked(Box<dyn std::any::Any + Send>),
+}
+
+/// 卡上一段活（`outcome`，panic 了是 `Err`）怎么收：这条线程上的 Out of Memory 范围接到了（`out_of_memory_in_the_scope`）就是显存不够，
+/// 活成没成都一样；没接到而活 panic 了，看设备上从活开始起记的（`out_of_memory_on_the_device_since_the_work_began`，别的线程上没接住的
+/// Out of Memory、或因它丢了设备）：有就是显存不够，没有照样抛（第 10x 批代码轮判决 K1-3：判器内核在另起的线程上编，那条线程上的
+/// Out of Memory 让卡线程在 `join` 那里 panic，范围接不到）。活没 panic 就交它的结果。
+#[cfg(feature = "gpu")]
+fn decide_the_work_on_the_card<Outcome>(
+    outcome: std::thread::Result<Outcome>,
+    out_of_memory_in_the_scope: Option<String>,
+    out_of_memory_on_the_device_since_the_work_began: Option<String>,
+) -> WorkOnTheCard<Outcome> {
+    match (
+        outcome,
+        out_of_memory_in_the_scope,
+        out_of_memory_on_the_device_since_the_work_began,
+    ) {
+        (Ok(_) | Err(_), Some(detail), _) => WorkOnTheCard::OutOfMemory { detail },
+        (Ok(outcome), None, _) => WorkOnTheCard::Done(outcome),
+        (Err(_), None, Some(detail)) => WorkOnTheCard::OutOfMemory { detail },
+        (Err(panic), None, None) => WorkOnTheCard::Panicked(panic),
+    }
+}
+
 /// 在 `card` 上做 `work`，同时接住 wgpu 报的 Out of Memory（文件头第 5 条）：没报就交 `work` 的结果；报了交 `Err(那一句)`
 /// （`what_ran_out` 说是哪一段，`work` 里因为缓冲没建成而 panic 的一并接住、不再往外抛）；没报而 `work` panic 了照样往外抛。
-/// wgpu 的错误范围按线程分：`work` 要在调它的这条线程上建缓冲、派活。
+/// wgpu 的错误范围按线程分：这条线程上的接在范围里；`work` 另起的线程上没接住的记在设备上（`crate::gpu_unit_checks::DeviceConditionOfACard`），
+/// 照 [`decide_the_work_on_the_card`] 一并认。范围接到的也记在设备上（同一个设备上别的线程照它认）。
 #[cfg(feature = "gpu")]
 fn on_the_card_catching_out_of_memory<Outcome>(
     card: &crate::gpu_unit_checks::GpuCard,
     what_ran_out: &str,
     work: impl FnOnce() -> Outcome,
 ) -> Result<Outcome, String> {
+    let out_of_memory_reports_before = card.device_condition().out_of_memory_reports();
     let out_of_memory_scope = card
         .device()
         .push_error_scope(wgpu::ErrorFilter::OutOfMemory);
     let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
-    match (outcome, pollster::block_on(out_of_memory_scope.pop())) {
-        (Ok(outcome), None) => Ok(outcome),
-        (Ok(_) | Err(_), Some(error)) => Err(reason_for_running_out_of_gpu_memory(&format!(
-            "序号 {} 的卡{what_ran_out}（{error}）",
+    let out_of_memory_in_the_scope =
+        pollster::block_on(out_of_memory_scope.pop()).map(|error| error.to_string());
+    let out_of_memory_on_the_device = card
+        .device_condition()
+        .out_of_memory_explaining_a_failure_since(out_of_memory_reports_before);
+    if let Some(caught) = &out_of_memory_in_the_scope {
+        card.device_condition().record_an_out_of_memory(format!(
+            "序号 {} 的卡{what_ran_out}报了 {caught}",
             card.index
-        ))),
-        (Err(panic), None) => std::panic::resume_unwind(panic),
+        ));
+    }
+    match decide_the_work_on_the_card(
+        outcome,
+        out_of_memory_in_the_scope,
+        out_of_memory_on_the_device,
+    ) {
+        WorkOnTheCard::Done(outcome) => Ok(outcome),
+        WorkOnTheCard::OutOfMemory { detail } => Err(reason_for_running_out_of_gpu_memory(
+            &format!("序号 {} 的卡{what_ran_out}（{detail}）", card.index),
+        )),
+        WorkOnTheCard::Panicked(panic) => std::panic::resume_unwind(panic),
     }
 }
 
@@ -2002,7 +2061,8 @@
 /// 然后从队头一块一块领、判、把结论字拆成红项、交给写库的线程。有一个状态判不了（结论字带「判不了」），
 /// 就叫所有卡停下、发一条 [`CardMessage::Refused`]，不退回 CPU 判器（用户 2026-09-29 定：配了 GPU 就一定要 GPU 判，做不到就报错）。
 /// 建判器或判一块时 wgpu 报 Out of Memory（[`on_the_card_catching_out_of_memory`]），叫所有卡停下、发一条 [`CardMessage::OutOfMemory`]，
-/// 那一块不交去写库。起过卡池的进程，判器建好那一刻核账：草稿区那一笔借据还在不在（第 10t 批代码轮判决 K4-1）。
+/// 那一块不交去写库。起过卡池的进程，判器建好那一刻与这张卡判完那一刻各核一次账：草稿区那一笔借据还在不在（第 10t 批代码轮判决 K4-1、
+/// 第 10x 批代码轮判决 K3-1）。
 #[cfg(feature = "gpu")]
 fn judge_on_one_card_and_start_the_next<'scope, 'work>(
     scope: &'scope std::thread::Scope<'scope, 'work>,
@@ -2031,11 +2091,13 @@
                 return;
             }
         };
-    let scratch_lease_on_the_books_when_the_judge_was_built =
+    let scratch_lease_is_on_the_books_now = || {
         work.scratch_lease_identifiers[position].map(|lease_identifier| {
             the_card_pool_this_run_takes_its_cards_from()
                 .is_the_lease_on_the_books(card.index, lease_identifier)
-        });
+        })
+    };
+    let scratch_lease_on_the_books_when_the_judge_was_built = scratch_lease_is_on_the_books_now();
     let build_seconds = build_started.elapsed().as_secs_f64();
     eprintln!(
         "CRASH_AMPLIFICATION_GPU_JUDGE card={:?} index={} memory_budget_mebibytes={} states_per_dispatch={} scratch_mebibytes={} build_seconds={build_seconds:.1}",
@@ -2131,6 +2193,8 @@
         }
     }
     outcome.judge_seconds = judging_started.elapsed().as_secs_f64();
+    outcome.scratch_lease_on_the_books_when_the_judging_finished =
+        scratch_lease_is_on_the_books_now();
     let spent = judge.time_spent();
     outcome.dispatches = spent.dispatches;
     outcome.seconds_preparing = spent.seconds_preparing;
@@ -2212,6 +2276,9 @@
     max_storage_buffer_binding_size: u64,
     memory_budget_mebibytes: u64,
     states_in_flight_capacity: u64,
+    /// 这张卡空着时一次派活带得了几个状态（起过卡池的照这张卡空着时能借给这个进程的算；这一趟自己起的卡没有「此刻」与「空着时」之分，
+    /// 同 `states_in_flight_capacity`）：每张卡此刻都带不了一个状态时照它分借不到与放不下（第 10x 批代码轮判决 K2-2）。
+    states_in_flight_capacity_when_the_card_is_empty: u64,
 }
 
 /// 起过卡池的这一趟本机每张卡（池里每张，按池的优先级，不起设备）：额度是这一刻这张卡能借给这个进程的加驱动那一份，照它算一次派活
@@ -2227,32 +2294,39 @@
 ) -> Result<Vec<CardCapacityOfThisRun>, String> {
     let cards = pool.cards_for_the_product_path();
     let card_indices: Vec<u32> = cards.iter().map(|card| card.index).collect();
-    let rooms_mebibytes = pool.card_rooms_mebibytes_now_of(&card_indices)?;
+    let rooms = pool.card_rooms_now_and_when_empty_of(&card_indices)?;
+    // 照能借的（MiB）算一次派活带得了几个状态：额度是能借的加驱动那一份，再按判器按状态数借的量压到能借的之内
+    let states_in_flight_for_a_room = |max_storage_buffer_binding_size: u64,
+                                       room_mebibytes: u64| {
+        let states_by_the_budget = states_in_flight_within_the_budget(
+            max_storage_buffer_binding_size,
+            room_mebibytes + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER,
+            tables,
+        );
+        u64::try_from(most_states_whose_need_fits_the_capacity(
+            usize::try_from(states_by_the_budget).unwrap_or(usize::MAX),
+            room_mebibytes,
+            &|states| crate::crash_judge_dispatch::mebibytes_a_judge_needs(tables, states),
+        ))
+        .expect("装得进 u64")
+    };
     Ok(cards
         .into_iter()
-        .zip(rooms_mebibytes)
-        .map(|(card, room_mebibytes)| {
-            let memory_budget_mebibytes =
-                room_mebibytes + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
-            let states_by_the_budget = states_in_flight_within_the_budget(
+        .zip(rooms)
+        .map(|(card, room)| CardCapacityOfThisRun {
+            index: card.index,
+            memory_budget_mebibytes: room.room_now_mebibytes
+                + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER,
+            states_in_flight_capacity: states_in_flight_for_a_room(
                 card.max_storage_buffer_binding_size,
-                memory_budget_mebibytes,
-                tables,
-            );
-            let states_in_flight_capacity =
-                u64::try_from(most_states_whose_need_fits_the_capacity(
-                    usize::try_from(states_by_the_budget).unwrap_or(usize::MAX),
-                    room_mebibytes,
-                    &|states| crate::crash_judge_dispatch::mebibytes_a_judge_needs(tables, states),
-                ))
-                .expect("装得进 u64");
-            CardCapacityOfThisRun {
-                index: card.index,
-                name: card.name,
-                max_storage_buffer_binding_size: card.max_storage_buffer_binding_size,
-                memory_budget_mebibytes,
-                states_in_flight_capacity,
-            }
+                room.room_now_mebibytes,
+            ),
+            states_in_flight_capacity_when_the_card_is_empty: states_in_flight_for_a_room(
+                card.max_storage_buffer_binding_size,
+                room.ceiling_mebibytes,
+            ),
+            name: card.name,
+            max_storage_buffer_binding_size: card.max_storage_buffer_binding_size,
         })
         .collect())
 }
@@ -2285,6 +2359,7 @@
                     .max_storage_buffer_binding_size,
                 memory_budget_mebibytes: card.memory_budget_mebibytes(),
                 states_in_flight_capacity,
+                states_in_flight_capacity_when_the_card_is_empty: states_in_flight_capacity,
             })
             .collect()),
     }
@@ -2366,17 +2441,26 @@
     )
 }
 
-/// 起过卡池的产品路径在显存账上借不到时怎么报（`what_was_booked` 说借的是什么）：此刻借不到照对拍用例那一路打「本次未跑」那一行
-/// （[`scratch_booking_refusal_outcome`]，同一个记号）、交 `Ok(那一行)`，调用方照叫停的样子交回；要的比每张卡空着时能借的都大、
-/// 卡上的显存读不出判红。
+/// 起过卡池的产品路径向卡池借不到时怎么报（`what_was_booked` 说借的是什么）：此刻借不到、借到的卡上起设备时显存不够，
+/// 照对拍用例那一路打「本次未跑」那一行（同一个记号、原因写显存不够）、交 `Ok(那一行)`，调用方照叫停的样子交回；
+/// 要的比每张卡空着时能借的都大、卡上的显存读不出判红。
 ///
 /// # Errors
 /// 要的比每张卡空着时能借的都大（说法与没有一张卡装得下这条流的表那一句同一类，分项驱动照它认成设备用不上）、卡上的显存读不出。
 #[cfg(feature = "gpu")]
 fn print_the_not_run_line_or_refuse(
     what_was_booked: &str,
-    refusal: &BookingRefusal,
+    refusal: &crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal,
 ) -> Result<String, String> {
+    use crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal;
+    let refusal = match refusal {
+        CardPoolLeaseRefusal::Booking(refusal) => refusal,
+        CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice { reason } => {
+            let not_run_line = gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, reason);
+            println!("{not_run_line}");
+            return Ok(not_run_line);
+        }
+    };
     if let BookingRefusal::NeedLargerThanEveryCard(NeedLargerThanEveryCard {
         need_mebibytes,
         largest_ceiling_mebibytes,
@@ -2592,28 +2676,56 @@
         .iter()
         .map(|card| card.states_in_flight_capacity)
         .collect();
+    let capacities_when_the_cards_are_empty: Vec<u64> = run_cards
+        .iter()
+        .map(|card| card.states_in_flight_capacity_when_the_card_is_empty)
+        .collect();
     // 借据活到这个函数收尾：几张卡上的判器在下面的线程里判完就丢，丢完才还
-    let scratch = if capacities.iter().all(|capacity| *capacity == 0) {
-        None
-    } else {
-        match scratch_of_each_card_booked_by_where_the_cards_came_from(
-            cards_taken_from,
-            cards,
-            &run_cards,
-            &tables,
-            state_count,
-            pending_states,
-            &capacities,
-        )? {
-            ScratchOfEachCard::Ready(scratch) => Some(scratch),
-            ScratchOfEachCard::NotRunTheGpuMemoryWasShort { not_run_line } => {
+    let scratch =
+        match decide_scratch_before_booking(&capacities, &capacities_when_the_cards_are_empty) {
+            ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow => {
+                match scratch_of_each_card_booked_by_where_the_cards_came_from(
+                    cards_taken_from,
+                    cards,
+                    &run_cards,
+                    &tables,
+                    state_count,
+                    pending_states,
+                    &capacities,
+                )? {
+                    ScratchOfEachCard::Ready(scratch) => Some(scratch),
+                    ScratchOfEachCard::NotRunTheGpuMemoryWasShort { not_run_line } => {
+                        return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
+                            tally,
+                            reason: not_run_line,
+                        });
+                    }
+                }
+            }
+            ScratchBeforeBooking::NoCardHoldsAStateNow => {
+                let card_rooms_mebibytes: Vec<u64> = run_cards
+                    .iter()
+                    .map(|card| {
+                        card.memory_budget_mebibytes.saturating_sub(
+                            crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER,
+                        )
+                    })
+                    .collect();
+                let not_run_line = gpu_not_run_line(
+                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
+                    &reason_for_a_booking_without_room_now(
+                        crate::crash_judge_dispatch::mebibytes_a_judge_needs(&tables, 1),
+                        &card_rooms_mebibytes,
+                    ),
+                );
+                println!("{not_run_line}");
                 return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
                     tally,
                     reason: not_run_line,
                 });
             }
-        }
-    };
+            ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty => None,
+        };
     let states_placed_on_each_card_of_this_run = scratch.as_ref().map_or_else(
         || vec![0; run_cards.len()],
         |scratch| scratch.states_placed_on_each_card_of_this_run.clone(),
@@ -3696,6 +3808,7 @@
                     seconds_of_the_longest_dispatch: 0.0,
                     scratch_booked_mebibytes: None,
                     scratch_lease_on_the_books_when_the_judge_was_built: None,
+                    scratch_lease_on_the_books_when_the_judging_finished: None,
                 };
                 card_sender
                     .send(CardMessage::Finished(finished_card))
@@ -3738,6 +3851,56 @@
         assert_eq!(received.refusal, None);
     }
 
+    /// 第 10x 批代码轮判决 K1-3（攻方 R2）：卡线程上的错误范围接到 Out of Memory 就是显存不够（活成没成都一样）；没接到而活 panic 了，
+    /// 设备上从活开始起记了 Out of Memory（另起的线程上编内核时没接住的）也是显存不够；都没有照样抛；活没 panic 交结果。
+    /// 不要卡：几样输入照 [`on_the_card_catching_out_of_memory`] 收到的造。
+    #[cfg(feature = "gpu")]
+    #[test]
+    fn a_panic_on_the_card_after_an_out_of_memory_on_another_thread_is_put_down_to_gpu_memory() {
+        let panicked = || -> std::thread::Result<u32> {
+            Err(Box::new("恢复内核的编译线程不 panic"))
+        };
+        assert!(
+            matches!(
+                decide_the_work_on_the_card(
+                    panicked(),
+                    None,
+                    Some("另起的线程上没接住的 Out of Memory".to_string())
+                ),
+                WorkOnTheCard::OutOfMemory { ref detail } if detail == "另起的线程上没接住的 Out of Memory"
+            ),
+            "范围没接到、设备上记了：显存不够，不往外抛"
+        );
+        assert!(
+            matches!(
+                decide_the_work_on_the_card(panicked(), Some("Out of Memory".to_string()), None),
+                WorkOnTheCard::OutOfMemory { .. }
+            ),
+            "范围接到了：显存不够"
+        );
+        assert!(
+            matches!(
+                decide_the_work_on_the_card(Ok(7u32), Some("Out of Memory".to_string()), None),
+                WorkOnTheCard::OutOfMemory { .. }
+            ),
+            "活成了而范围接到 Out of Memory：缓冲没建成，判的不作数"
+        );
+        assert!(
+            matches!(
+                decide_the_work_on_the_card(panicked(), None, None),
+                WorkOnTheCard::Panicked(_)
+            ),
+            "都没有：照样往外抛"
+        );
+        assert!(
+            matches!(
+                decide_the_work_on_the_card(Ok(7u32), None, None),
+                WorkOnTheCard::Done(7)
+            ),
+            "活成了：交结果"
+        );
+    }
+
     /// 第 10x 批：卡上 wgpu 报了 Out of Memory（[`CardMessage::OutOfMemory`]，文件头第 5 条）记成显存不够、留第一句，不记成拒判
     /// （拒判整趟判红，显存不够这一趟报本次未跑）；之前判完的块照写进库。不要卡：卡上的线程换成直接发这几句话。
     #[cfg(feature = "gpu")]
@@ -5460,7 +5623,7 @@
 enum PlacementOfThisRun {
     /// 打了 `CRASH_AMPLIFICATION_PLACEMENT` 那一行。
     Printed,
-    /// 起过卡池的进程此刻借不到试判的草稿区：「本次未跑」那一行已经打了，没打 PLACEMENT 行。
+    /// 起过卡池的进程此刻借不到试判的草稿区、或试判时卡上报了 Out of Memory：「本次未跑」那一行已经打了，没打 PLACEMENT 行。
     NotRunTheGpuMemoryWasShort,
 }
 
@@ -5609,28 +5772,45 @@
             if *states_placed == 0 {
                 continue;
             }
-            let judge = GpuCrashJudge::on_card_with_packed_tables(
+            // 建判器、试判都在这一层里：卡上报 Out of Memory（判器内核另起的线程上的也算）报本次未跑，不判红（第 10x 批代码轮判决 K1-3）
+            let trial = on_the_card_catching_out_of_memory(
                 card,
-                &tables,
-                usize::try_from(*states_placed).expect("装得进 usize"),
+                "只排派活计划那一趟试判时",
+                || {
+                    let judge = GpuCrashJudge::on_card_with_packed_tables(
+                        card,
+                        &tables,
+                        usize::try_from(*states_placed).expect("装得进 usize"),
+                    );
+                    // 先判一小段让卡量出派活宽度（量宽度那几次派活不算速度），再计时判几次派活那么多
+                    let first = point.ordinals.start + local.start;
+                    let state_count = judge.state_count();
+                    // 量宽度最宽试到这张卡一次带得了的上限，先判的这一段要比它长
+                    let warm_end = (first
+                        + 2 * u64::try_from(judge.states_per_dispatch()).expect("装得进 u64"))
+                    .min(state_count);
+                    let _warm = judge.judge_states(first..warm_end);
+                    let timed_end = (warm_end
+                        + u64::try_from(judge.dispatch_width()).expect("装得进 u64")
+                            * CALIBRATION_DISPATCHES)
+                        .min(state_count);
+                    let started = Instant::now();
+                    let words = judge.judge_states(warm_end..timed_end);
+                    (words.len(), started.elapsed().as_secs_f64())
+                },
             );
-            // 先判一小段让卡量出派活宽度（量宽度那几次派活不算速度），再计时判几次派活那么多
-            let first = point.ordinals.start + local.start;
-            let state_count = judge.state_count();
-            // 量宽度最宽试到这张卡一次带得了的上限，先判的这一段要比它长
-            let warm_end = (first
-                + 2 * u64::try_from(judge.states_per_dispatch()).expect("装得进 u64"))
-            .min(state_count);
-            let _warm = judge.judge_states(first..warm_end);
-            let timed_end = (warm_end
-                + u64::try_from(judge.dispatch_width()).expect("装得进 u64")
-                    * CALIBRATION_DISPATCHES)
-                .min(state_count);
-            let started = Instant::now();
-            let words = judge.judge_states(warm_end..timed_end);
-            let seconds = started.elapsed().as_secs_f64();
+            let (states_timed, seconds) = match trial {
+                Ok(timed) => timed,
+                Err(reason) => {
+                    println!(
+                        "{}",
+                        gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
+                    );
+                    return Ok(PlacementOfThisRun::NotRunTheGpuMemoryWasShort);
+                }
+            };
             if seconds > 0.0 {
-                states_per_second_on_this_host += words.len() as f64 / seconds;
+                states_per_second_on_this_host += states_timed as f64 / seconds;
             }
         }
     }
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs b/crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs
--- a/crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs
@@ -1,8 +1,8 @@
 //! checker 档模块：crash_amplification、gpu_unit_checks、crash_judge_dispatch
 //! 起过 GPU 用例共用的卡池的进程里，崩溃放量产品路径（`crash_amplification::run_from_environment`）在卡上建的缓冲都在这个进程的显存账上借
 //! （`crash_amplification` 文件头第 5 条）：第 ① 段判器的草稿区、第 ① 段由 CPU 判时单元校验和对拍的缓冲、第 ② 段核对的缓冲。
-//! 两条用例各在新池新建文件那条流上按环境真跑一趟，看这一趟交回的借的量，再看用的那一刻核账（判器建好、对拍与核对做完时那一笔借据还在账上，
-//! 第 10t 批代码轮判决 K4-1）：哪一样改回不借、或借了又在用之前还掉，对应的那条就红。
+//! 两条用例各在新池新建文件那条流上按环境真跑一趟，看这一趟交回的借的量，再看用的那一刻核账（判器建好与判完、对拍与核对做完时那一笔借据
+//! 还在账上，第 10t 批代码轮判决 K4-1、第 10x 批代码轮判决 K3-1）：哪一样改回不借、或借了又在用完之前还掉，对应的那条就红。
 //! 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」（经这个进程的卡池判，`gpu_test_start::gpu_card_pool_or_report_not_run`），
 //! 有适配器而点名的卡找不到判红；卡上此刻借不到、或卡上 wgpu 报了 Out of Memory，产品路径打它自己的「本次未跑」那一行、这一趟照叫停的样子交回，
 //! 这里同样报「本次未跑」。要 `verdict-store,gpu` 两个特性，少一个这份文件是空的。
@@ -171,8 +171,8 @@
 }
 
 /// 第 ① 段在卡上判（登记 `crash_judge_gpu`）：每张用到的卡上，判器的草稿区都在显存账上借了，判器用的就是借来的那张
-/// （额度是借的量加驱动那一份，开出来的草稿区不超过借的），判器建好那一刻那一笔借据还在账上。哪天这一支改回不借，借的量是 `None`，
-/// 借了又在判之前还掉，核账是 `Some(false)`，这条都红。
+/// （额度是借的量加驱动那一份，开出来的草稿区不超过借的），判器建好那一刻与这张卡判完那一刻那一笔借据都还在账上。哪天这一支改回不借，
+/// 借的量是 `None`，借了又在判之前或判完之前还掉，核账是 `Some(false)`，判完那一刻不核是 `None`，这条都红。
 #[test]
 fn the_product_path_judges_on_scratch_it_booked_on_the_card_pool() {
     let Some(run) = amplify_on_the_card_pool(
@@ -214,6 +214,12 @@
             "序号 {} 的卡：判器建好那一刻核账，草稿区那一笔借据要还在账上（借了又在判之前还掉要红）",
             card.index
         );
+        assert_eq!(
+            card.scratch_lease_on_the_books_when_the_judging_finished,
+            Some(true),
+            "序号 {} 的卡：判完那一刻再核一次账，草稿区那一笔借据要还在账上（建好之后、判完之前还掉要红）",
+            card.index
+        );
     }
 }
 
diff --git a/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs b/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
--- a/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
@@ -6,7 +6,8 @@
 //! 在有卡的机器上与经入口走同一条路，跑用例分不出；这一条扫源码，不要显卡。
 //! 扫的是 `tests/` 下每份 `.rs`（这一份除外）整份，与 `src/` 下每份 `.rs` 里的测试代码：`#[test]`、条件含 `test`（不在 `not(…)` 里）的
 //! `#[cfg(…)]` 下的项，顶格不顶格、与别的属性写在同一行、`cfg` 与括号之间带空白都算（`#[cfg(test)]`、`#[cfg(all(test, feature = "gpu"))]`、
-//! `impl` 块里缩进的 `#[cfg(test)]`、`#[allow(…)] #[cfg (test)]`），文件级的 `#![cfg(test)]` 整份算，测试代码里 `mod 名字;` 引进来的
+//! `impl` 块里缩进的 `#[cfg(test)]`、`#[allow(…)] #[cfg (test)]`），`#[cfg_attr(条件, 属性…)]` 里带着这两种属性的同样算
+//! （`#[cfg_attr(test, test)]`，第 10x 批代码轮判决 K3-2），文件级的 `#![cfg(test)]` 整份算，测试代码里 `mod 名字;` 引进来的
 //! 那份文件也整份算（第 10t 批代码轮判决 K4-2）；产品路径（崩溃放量取卡、`gpu_unit_checks` 自己）不在测试代码里，不扫。
 //! 注释（`//` 到行尾、`/* */`）不扫，字符串（普通、`b"…"`、`c"…"`、原始的 `r#"…"#`、`br#"…"#`、`cr#"…"#`）与字符字面量的内容也不扫
 //! （字符串里的 `//` 不当注释起头，同一行后面的代码照扫）。
@@ -14,6 +15,8 @@
 //! 不扫的（交 review）：`include!` 与 `#[path = "…"]` 引进来的文件、`benches/` 与 `examples/`、产品代码里包一层再从测试调的。
 //! 改走卡池的几份测试文件（[`FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL`]）另判一层：只许经 `gpu_card_pool_or_report_not_run` 取卡、
 //! 按量借，不许再各自拿第一张能用的卡、拿整份卡表或直接在卡上建判器（各按空闲显存的比例开，几个用例并行时互抢显存）。
+//! 名单之外的 `tests/` 文件同样扫这一层：还绕开卡池的要登记在 [`TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL`]、写明为什么还没改，
+//! 登记了的要真还在绕（第 10x 批代码轮判决 K3-2：名单之外的文件不扫，新写的 GPU 用例绕开卡池没人拦）。
 
 use std::path::{Path, PathBuf};
 
@@ -39,8 +42,9 @@
 ];
 
 /// 改走卡池的测试文件（相对包根）：每份至少一处调 [`CARD_POOL_FUNCTION`]，[`NAMES_THAT_BYPASS_THE_POOL`] 一处都不许有。
-const FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL: [&str; 7] = [
+const FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL: [&str; 8] = [
     "tests/crash_amplification_pipeline_records_checks_and_reuses.rs",
+    "tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs",
     "tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs",
     "tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs",
     "tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs",
@@ -60,6 +64,14 @@
 /// 改走卡池的文件里经卡池取卡的函数名。
 const CARD_POOL_FUNCTION: &str = "gpu_card_pool_or_report_not_run";
 
+/// [`FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL`] 之外、还绕开卡池拿卡或建判器（[`NAMES_THAT_BYPASS_THE_POOL`]）的测试文件（相对包根）
+/// 与为什么还没改。每一项要真还在绕：改完了就挪进名单、删这一项（只缩不涨）。
+const TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL: [(&str, &str); 1] = [(
+    "tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs",
+    "GPU 对拍那一份：量显存的用例与几条按整张卡起的对拍还经入口拿整份卡表、第一张卡或直接建判器；这份文件在删除线第 34 批 U 手里改\
+     （第 10x 批代码轮判决第三节末段：GPU 用例自己的 Out of Memory 与对拍流归它）",
+)];
+
 /// 经入口判过「有卡、照跑」之后再起卡、准许绕开的几处：(相对包根的文件, 那一行去掉前后空白的代码, 为什么准许)。
 /// 每一项必须恰好命中一次，命中不到也判红：准许的那一行改了或删了，这里不许还留着。
 const BYPASSES_ALLOWED_AFTER_THE_ENTRY_DECIDED: [(&str, &str, &str); 1] = [(
@@ -372,13 +384,46 @@
     None
 }
 
+/// `cfg_attr(…)` 括号里那一段（去掉注释、字符串内容之后的原文）按最外层的逗号切开：第一段是条件，后面每段是一条属性。
+fn arguments_of_a_cfg_attr(inside_the_parentheses: &str) -> Vec<&str> {
+    let mut arguments = Vec::new();
+    let mut depth = 0_usize;
+    let mut start = 0;
+    for (offset, character) in inside_the_parentheses.char_indices() {
+        match character {
+            '(' | '[' => depth += 1,
+            ')' | ']' => depth = depth.saturating_sub(1),
+            ',' if depth == 0 => {
+                arguments.push(&inside_the_parentheses[start..offset]);
+                start = offset + 1;
+            }
+            _ => {}
+        }
+    }
+    arguments.push(&inside_the_parentheses[start..]);
+    arguments
+}
+
 /// 属性（`[` 与 `]` 之间的原文）让它下面的东西只在测试里编：裸的 `test`（`#[test]`），或条件含 `test`（不在 `not(…)` 里，
-/// [`cfg_predicate_includes_test_outside_a_not`]）的 `cfg(…)`（`cfg` 与括号之间可以有空白）。
+/// [`cfg_predicate_includes_test_outside_a_not`]）的 `cfg(…)`（`cfg` 与括号之间可以有空白），或 `cfg_attr(条件, 属性…)` 里
+/// 带着这两种之一的（`#[cfg_attr(test, test)]`：测试里编成测试函数，第 10x 批代码轮判决 K3-2）。
 fn attribute_marks_test_code(attribute_inside: &str) -> bool {
     let trimmed = attribute_inside.trim();
     if trimmed == "test" {
         return true;
     }
+    if let Some(after_the_name) = trimmed.strip_prefix("cfg_attr") {
+        return after_the_name
+            .trim_start()
+            .strip_prefix('(')
+            .and_then(|rest| rest.trim_end().strip_suffix(')'))
+            .is_some_and(|inside_the_parentheses| {
+                arguments_of_a_cfg_attr(inside_the_parentheses)
+                    .iter()
+                    .skip(1)
+                    .any(|attribute| attribute_marks_test_code(attribute))
+            });
+    }
     let Some(after_the_name) = trimmed.strip_prefix("cfg") else {
         return false;
     };
@@ -773,6 +818,77 @@
     );
 }
 
+/// 第 10x 批代码轮判决 K3-2：名单（[`FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL`]）之外的 `tests/` 文件（含子目录，这一份除外）也扫
+/// 绕开卡池的名字：扫出来的文件要与登记的（[`TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL`]）逐个相同：新写的绕开卡池要红，
+/// 登记了而已经不绕的也红（挪进名单）。
+#[test]
+fn every_test_file_outside_the_pool_list_that_bypasses_the_pool_is_registered_with_a_reason() {
+    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
+    let mut scanned_file_count = 0_usize;
+    let mut bypassing_files = Vec::new();
+    for path in rust_source_files_under(&package_root.join("tests")) {
+        if path.file_name().is_some_and(|name| name == THIS_FILE_NAME) {
+            continue;
+        }
+        let relative = path
+            .strip_prefix(package_root)
+            .expect("扫的文件都在包根下")
+            .to_string_lossy()
+            .into_owned();
+        if FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL.contains(&relative.as_str()) {
+            continue;
+        }
+        let source = std::fs::read_to_string(&path)
+            .unwrap_or_else(|error| panic!("{} 读不出：{error}", path.display()));
+        scanned_file_count += 1;
+        if !lines_mentioning(&relative, &code_lines(&source), &NAMES_THAT_BYPASS_THE_POOL)
+            .is_empty()
+        {
+            bypassing_files.push(relative);
+        }
+    }
+    assert!(
+        scanned_file_count >= 20,
+        "名单之外扫到的 tests/ 文件只有 {scanned_file_count} 份，少得不对，扫法坏了"
+    );
+    let registered: Vec<String> = TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL
+        .iter()
+        .map(|(file, _why_not_yet)| (*file).to_string())
+        .collect();
+    assert_eq!(
+        bypassing_files, registered,
+        "名单之外还绕开卡池拿卡或建判器的测试文件要与登记的逐个相同：新扫出来的改成经 gpu_card_pool_or_report_not_run 取池、按量借\
+         （改不了的登记进 TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL、写明为什么）；登记了而已经不绕的挪进 FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL"
+    );
+    println!(
+        "CARD_POOL_BYPASSES_OUTSIDE_THE_LIST scanned={scanned_file_count} registered={}",
+        registered.len()
+    );
+}
+
+/// 第 10x 批代码轮判决 K3-2 红样本：库源码里 `#[cfg_attr(test, test)]` 下的函数在测试里编成测试函数，里面的起卡调用照扫；
+/// `cfg_attr` 只带别的属性（`allow`）的不算测试代码。
+#[test]
+fn test_code_under_a_cfg_attr_carrying_test_in_a_library_source_is_caught() {
+    let sample = "#[cfg_attr(test, test)]\n\
+                  fn takes_cards() {\n\
+                  \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
+                  }\n\
+                  #[cfg_attr(feature = \"gpu\", allow(dead_code))]\n\
+                  fn product() {\n\
+                  \x20   let _cards = crate::gpu_unit_checks::usable_gpu_cards();\n\
+                  }\n\
+                  #[cfg_attr(feature = \"gpu\", cfg(test))]\n\
+                  fn helper() {\n\
+                  \x20   let _cards = crate::gpu_unit_checks::gpu_cards_by_priority(&[]);\n\
+                  }\n";
+    assert_eq!(
+        bypass_line_numbers_in_a_library_sample(sample),
+        vec![3, 11],
+        "cfg_attr 带着 test 或 cfg(test) 的扫；只带 allow 的是产品代码、不扫"
+    );
+}
+
 /// 第 10t 批 K4-4 红样本一：`use … as 别名` 引进起卡的函数、再调别名，调用那一行没有原名，`use` 那一行有：扫得出；
 /// 改走卡池的文件里给入口函数起别名同样扫得出。
 #[test]
````

## mutations-append.tsv

````text
第10y批 显存账 K1-1 空闲与占着的分两次读（每借一次取两份报告）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        let report = run_the_report()?;	        let report = {\n            run_the_report()?;\n            run_the_report()?\n        };	-p singlefs-harness --lib -- gpu_memory_ledger::tests::each_booking_reads_the_free_memory_and_this_process_occupancy_from_one_report	each_booking_reads_the_free_memory_and_this_process_occupancy_from_one_report
第10y批 显存账 K1-1 报告里 FB 之外小节的 Free 也当空闲（BAR1 的盖掉 FB 的）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            (SectionOfTheReport::FrameBufferMemory, "Free") => {	            (SectionOfTheReport::FrameBufferMemory | SectionOfTheReport::Other, "Free") => {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::one_nvidia_smi_report_gives_the_frame_buffer_memory_and_this_process_occupancy_of_each_card	one_nvidia_smi_report_gives_the_frame_buffer_memory_and_this_process_occupancy_of_each_card
第10y批 显存账 K1-1 读报告时不认这个进程另有不走账的设备（攻方乙形）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    if has_devices_outside_the_books_on_the_bus(bus) {	                    if false && has_devices_outside_the_books_on_the_bus(bus) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::one_nvidia_smi_report_gives_the_frame_buffer_memory_and_this_process_occupancy_of_each_card	one_nvidia_smi_report_gives_the_frame_buffer_memory_and_this_process_occupancy_of_each_card
第10y批 显存账 K1-1 另有不走账的设备也当都走这份账（占着的当成借据建上的，攻方乙形）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => mebibytes	        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes }\n        | OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes } => mebibytes	-p singlefs-harness --lib -- gpu_memory_ledger::tests::memory_of_this_process_outside_the_books_is_not_taken_for_leases_already_built	memory_of_this_process_outside_the_books_is_not_taken_for_leases_already_built
第10y批 显存账 K1-1 估计只取现读的空闲（实际占着的里借据建上的一份都不算，第10t批 K4-3 那一形）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => mebibytes	        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => 0 * mebibytes	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised	the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised
第10y批 显存账 K1-1 销了账、卡上还没还的不记（当成别的借据已经建上的，正推冲突 B）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    .saturating_add(mebibytes);	                    .saturating_add(mebibytes.min(0));	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_returned_lease_still_on_the_card_is_not_taken_for_a_live_lease_already_built	a_returned_lease_still_on_the_card_is_not_taken_for_a_live_lease_already_built
第10y批 显存账 K1-1 销了账、卡上还没还的那一份不照读数往下压（卡上还了照样少借）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            returned_possibly_still_on_the_card_mebibytes\n                .min(mebibytes.saturating_sub(driver_on_the_card_mebibytes))	            returned_possibly_still_on_the_card_mebibytes\n                .max(mebibytes.saturating_sub(driver_on_the_card_mebibytes))	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_returned_lease_still_on_the_card_is_not_taken_for_a_live_lease_already_built	a_returned_lease_still_on_the_card_is_not_taken_for_a_live_lease_already_built
第10y批 显存账 K1-1 空卡上能用的照总量算（不减驱动保留的，攻方丙形）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                total_less_reserved_mebibytes: total_mebibytes.saturating_sub(reserved_mebibytes),	                total_less_reserved_mebibytes: total_mebibytes.max(reserved_mebibytes),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_need_beyond_what_an_empty_card_has_never_fits_even_below_the_total_memory	a_need_beyond_what_an_empty_card_has_never_fits_even_below_the_total_memory
第10y批 显存账 K2-2 此刻带不了的卡记成没有要装的（一张都借不到时判红）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if states_left_to_place == 0 {	        if states_left_to_place == 0 || *states_the_card_holds == 0 {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_the_caller_found_holding_no_state_now_is_no_room_now_not_nothing_left_to_place	a_card_the_caller_found_holding_no_state_now_is_no_room_now_not_nothing_left_to_place
第10y批 显存账 K2-2 每张卡此刻都带不了一个状态而空着时带得了也判红（白红）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        return ScratchBeforeBooking::NoCardHoldsAStateNow;	        return ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::with_no_card_holding_a_state_now_a_card_holding_one_when_empty_means_not_run_not_red	with_no_card_holding_a_state_now_a_card_holding_one_when_empty_means_not_run_not_red
第10y批 显存账 K3 卡上报 Out of Memory 那一句不写显存不够	crates/singlefs-harness/src/gpu_memory_ledger.rs	        "{GPU_MEMORY_SHORT_REASON_PREFIX}{what_ran_out}上 wgpu 报了 Out of Memory	        "内存不够：{what_ran_out}上 wgpu 报了 Out of Memory	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_out_of_memory_reason_says_the_gpu_memory_was_short_in_the_not_run_line	the_out_of_memory_reason_says_the_gpu_memory_was_short_in_the_not_run_line
第10y批 显存账 K3 按状态数借时同一张卡给了两次不断言	crates/singlefs-harness/src/gpu_memory_ledger.rs	                .all(|(slot, position)| !card_positions[..slot].contains(position)),	                .all(|(_slot, _position)| true),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_same_card_given_twice_to_a_scratch_booking_is_refused_by_an_assertion	the_same_card_given_twice_to_a_scratch_booking_is_refused_by_an_assertion
第10y批 显存账 K3 读到的卡比池里多不拒	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if readings.len() != books.len() {	        if readings.len() < books.len() {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_reading_of_more_cards_than_the_pool_is_refused_and_nothing_is_booked	a_reading_of_more_cards_than_the_pool_is_refused_and_nothing_is_booked
第10y批 显存账 K1-3 卡上留着的设备丢了照旧拿它	crates/singlefs-harness/src/gpu_memory_ledger.rs	            if !is_lost(device) {	            if is_lost(device) || !is_lost(device) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced	a_device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced
第10y批 显存账 K1-3 起设备失败当场判红（不交错、下一次不再起）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        let device = start()?;	        let device = start().unwrap_or_else(|_failure| panic!("起不来"));	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced	a_device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced
第10y批 GPU 取卡 K1-3 起设备时显存不够也当起不来（判红，不报本次未跑）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	    if error.to_string() == wgpu::wgc::device::DeviceError::OutOfMemory.to_string() {	    if false && error.to_string() == wgpu::wgc::device::DeviceError::OutOfMemory.to_string() {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::a_device_request_refused_for_lack_of_memory_is_told_apart_from_other_refusals	a_device_request_refused_for_lack_of_memory_is_told_apart_from_other_refusals
第10y批 GPU 取卡 K1-3 没接住的 Out of Memory 不记在设备上（另起的线程上的认不出）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            condition.record_an_out_of_memory(format!("没接住的 {error}"));	            drop(format!("没接住的 {error}"));	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::an_uncaptured_out_of_memory_is_recorded_on_the_device_before_the_panic	an_uncaptured_out_of_memory_is_recorded_on_the_device_before_the_panic
第10y批 GPU 取卡 K1-3 活开始之后记的 Out of Memory 不算（另起的线程上的说不成显存不够）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        if self.out_of_memory_reports() > reports_before {	        if self.out_of_memory_reports() > reports_before + 1 {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::a_failure_is_put_down_to_gpu_memory_only_after_an_out_of_memory_on_the_device	a_failure_is_put_down_to_gpu_memory_only_after_an_out_of_memory_on_the_device
第10y批 GPU 取卡 K1-3 设备因 Out of Memory 丢了也说不成显存不够	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	        match (self.lost_reason(), last_out_of_memory) {	        match (None::<String>, last_out_of_memory) {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::a_failure_is_put_down_to_gpu_memory_only_after_an_out_of_memory_on_the_device	a_failure_is_put_down_to_gpu_memory_only_after_an_out_of_memory_on_the_device
第10y批 崩溃放量 K1-3 卡线程上的活 panic 了不看设备上记的 Out of Memory（判器内核另起的线程上的照样判红）	crates/singlefs-checker-tier/src/crash_amplification.rs	        (Err(_), None, Some(detail)) => WorkOnTheCard::OutOfMemory { detail },	        (Err(panic), None, Some(_detail)) => WorkOnTheCard::Panicked(panic),	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::a_panic_on_the_card_after_an_out_of_memory_on_another_thread_is_put_down_to_gpu_memory	a_panic_on_the_card_after_an_out_of_memory_on_another_thread_is_put_down_to_gpu_memory
第10y批 崩溃放量 K3-1 第 ① 段判完那一刻不再核账	crates/singlefs-checker-tier/src/crash_amplification.rs	    outcome.scratch_lease_on_the_books_when_the_judging_finished =\n        scratch_lease_is_on_the_books_now();	    outcome.scratch_lease_on_the_books_when_the_judging_finished = None;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_judges_on_scratch_it_booked_on_the_card_pool	the_product_path_judges_on_scratch_it_booked_on_the_card_pool
第10y批 源码扫描 K3-2 #[cfg_attr(test, test)] 下的项不当测试代码	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	    if let Some(after_the_name) = trimmed.strip_prefix("cfg_attr") {	    if let Some(after_the_name) = trimmed.strip_prefix("cfg_attr_that_no_code_writes") {	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- test_code_under_a_cfg_attr_carrying_test_in_a_library_source_is_caught	test_code_under_a_cfg_attr_carrying_test_in_a_library_source_is_caught
第10y批 源码扫描 K3-2 名单之外的 tests/ 文件不扫绕开卡池的名字	crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs	        if !lines_mentioning(&relative, &code_lines(&source), &NAMES_THAT_BYPASS_THE_POOL)	        if false && !lines_mentioning(&relative, &code_lines(&source), &NAMES_THAT_BYPASS_THE_POOL)	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- every_test_file_outside_the_pool_list_that_bypasses_the_pool_is_registered_with_a_reason	every_test_file_outside_the_pool_list_that_bypasses_the_pool_is_registered_with_a_reason
````

## mutations-replacements.tsv

````text
第10w批 崩溃放量 起过卡池的产品路径第 ① 段草稿区改回不借	crates/singlefs-checker-tier/src/crash_amplification.rs	                match scratch_of_each_card_booked_by_where_the_cards_came_from(\n                    cards_taken_from,\n                    cards,\n                    &run_cards,	                match scratch_of_each_card_booked_by_where_the_cards_came_from(\n                    GpuCardsSource::DevicesStartedForThisRun,\n                    cards,\n                    &run_cards,	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_judges_on_scratch_it_booked_on_the_card_pool	the_product_path_judges_on_scratch_it_booked_on_the_card_pool
第10x批 显存账 借的时候不照现读的空闲（照卡的总量算）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        room_now_mebibytes: capacity_of_memory_this_process_may_hold(may_hold_mebibytes)	        room_now_mebibytes: capacity_of_memory_this_process_may_hold(reading.total_less_reserved_mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_booking_after_an_outsider_took_memory_follows_the_free_memory_read_at_that_booking	a_booking_after_an_outsider_took_memory_follows_the_free_memory_read_at_that_booking
第10x批 显存账 实际占着的不封顶到许出去的（分配器多拿的块也算成能借的）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .saturating_sub(booked.returned_possibly_still_on_the_card_mebibytes)\n            .min(promised_mebibytes),	            .saturating_sub(booked.returned_possibly_still_on_the_card_mebibytes),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised	the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised
第10x批 显存账 起好了设备驱动那一份照样从空闲里扣	crates/singlefs-harness/src/gpu_memory_ledger.rs	                DriverShareOnTheCard::DeviceStarted => {\n                    mebibytes_left_for_the_driver.min(promised_mebibytes)\n                }	                DriverShareOnTheCard::DeviceStarted => 0,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised	the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised
第10x批 显存账 还没起设备就把驱动那一份当已经在卡上（空闲不到一份驱动开销也借给）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                DriverShareOnTheCard::NotRegistered\n                | DriverShareOnTheCard::RegisteredDeviceNotStarted => 0,\n            }\n	                DriverShareOnTheCard::NotRegistered\n                | DriverShareOnTheCard::RegisteredDeviceNotStarted => mebibytes_left_for_the_driver,\n            }\n	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_with_less_free_memory_than_one_driver_share_lends_nothing_and_registers_nothing	a_card_with_less_free_memory_than_one_driver_share_lends_nothing_and_registers_nothing
第10x批 显存账 借据丢掉不销账	crates/singlefs-harness/src/gpu_memory_ledger.rs	        match self.leases.remove(&lease_identifier) {	        match self.leases.get(&lease_identifier).copied() {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_is_on_the_books_until_it_is_dropped	a_lease_is_on_the_books_until_it_is_dropped
第10x批 显存账 此刻连一个状态都放不下的卡判成空着时也放不下（判红）	crates/singlefs-harness/src/gpu_memory_ledger.rs	    if need_of_one_state_mebibytes > room.ceiling_mebibytes {	    if need_of_one_state_mebibytes > room.room_now_mebibytes {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_need_larger_than_the_card_when_empty_never_fits_and_a_smaller_one_has_no_room_now	a_need_larger_than_the_card_when_empty_never_fits_and_a_smaller_one_has_no_room_now
第10x批 显存账 空着时能借的照此刻能占的算（外人占着时判成比卡大）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            reading.total_less_reserved_mebibytes,\n        ),	            may_hold_mebibytes,\n        ),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_room_now_subtracts_the_leases_and_the_ceiling_takes_what_an_empty_card_has	the_room_now_subtracts_the_leases_and_the_ceiling_takes_what_an_empty_card_has
````
