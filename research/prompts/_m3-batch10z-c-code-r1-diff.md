# 根环实现第 10z 批第三块：代码轮第一轮改动全文（2026-10-08）

实现员交的 `crates.patch` 原样，打在删除线第 38 批 Y、第 39 批 Z 合后的主工作区上。变异表追加、替换、删除三份附在后面。

````diff
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
index 12b9a26..4105de0 100644
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -22,9 +22,12 @@
 //! - F − 这个进程许出去而还没占上卡的那一份：许出去的是 B 连驱动那一份 D（还没登记时 B 是 0、D 照样要放得下），
 //!   减实际占着的里属于许出去的那部分。实际占着的先扣掉这个进程已经销账、卡上可能还没还的那一份
 //!   （[`BookedByThisProcessOnACard::returned_possibly_still_on_the_card_mebibytes`]，第 10x 批代码轮判决 K1-1：不当成别的借据已经建上的）；
-//!   那一份每读一次照「实际占着的减早先读数见到占在卡上、此刻还活着的那一份」往下压（[`returned_possibly_still_on_the_card_after_a_reading`]），
-//!   不照「驱动那一份整份在卡上」压（第 10y 批代码轮判决 K1-1 攻方丙形：驱动实占的 x 比 D 小时压过头，多给 D − x），也不只看这一次读数
-//!   （第 10y 批代码轮判决 K2-2 P1：有长借据建着时，已经还给驱动的短借据一笔一笔累积下来压不掉，每还一笔多扣一份）。
+//!   那一份每读一次只压到读数里这个进程此刻实际占着的为止（[`returned_possibly_still_on_the_card_after_a_reading`]），不再往下减驱动那一份、
+//!   也不减借据建上的（用户 2026-10-08 定第 10z 批代码轮岔路 G2「先回甲」：读数只给这个进程的合计，压得比合计更低就要猜每笔借据此刻在卡上
+//!   占多少，猜错一边多给——第 10z 批代码轮判决 K2-1 第 2 条，活着的借据缓冲缩回驱动、同时一笔销了账的卡上还没还；照驱动那一份整份在卡上压是
+//!   第 10y 批代码轮判决 K1-1 攻方丙形，驱动实占的 x 比 D 小时多给 D − x）。
+//!   代价是少借：有借据建着或设备起着时，已经还给驱动的那一份要到这个进程占着的比它还少才压得掉（第 10y 批代码轮判决 K2-2 P1 那一形，
+//!   长借据建着时每还一笔短借据多扣一份），借不到报本次未跑。
 //!   读不到实际占着的（`nvidia-smi` 没列这个进程），或这个进程在这张卡上另有不走这份账的设备（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]，
 //!   分不出哪一份是借据建的），借着的当一点都没建、驱动那一份也当还没占上卡（第 10y 批代码轮判决 K1-1 攻方乙附：当它整份在卡上，实占的 x 比 D 小时
 //!   多给 D − x），只会少借。
@@ -47,7 +50,8 @@
 //! 调用方拿到借据之后才在这张卡上起设备（[`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`]），
 //! 起好了记一笔（[`GpuMemoryLedger::record_the_device_started_on_the_card`]），从此驱动那一份算实际占在卡上。设备丢了要重起时，驱动那一份记回
 //! 许出去而还没占上卡、旧设备那一份当销了账而卡上可能还没还的，此刻放不下就不起（第 10y 批代码轮判决 K1-3：重起不另登记驱动那一份，
-//! 新设备的上下文没留地方）。
+//! 新设备的上下文没留地方）；被拒之后新设备起好之前，每一次要起都照此刻再判一次（第 10z 批代码轮判决 K1-3 正推 S6：被拒之前已借到的
+//! 并发用例再来起设备时不直接放行）。
 //!
 //! wgpu 在卡上报的 Out of Memory 记在设备上（[`DeviceConditionOfACard`]），按记的那条线程上正在做的那段活认（[`WorkOnTheCardOfThisThread`]）：
 //! 别的线程上一次 Out of Memory 不把这条线程上真正的 panic 说成显存不够（第 10y 批代码轮判决 K1-3 攻方）；判器内核在活另起的线程上编，
@@ -145,6 +149,9 @@ pub enum DriverShareOnTheCard {
     RegisteredDeviceNotStarted,
     /// 设备起好了：驱动那一份占在卡上。
     DeviceStarted,
+    /// 起好过的设备丢了、新设备还没起好（重起时此刻放不下被拒过，或还没起）：驱动那一份许出去、还没占在卡上；
+    /// 每一次要起新设备都照此刻再判一次放不放得下（[`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`]）。
+    DeviceLostNotStartedAgain,
 }
 
 /// 这个进程在一张卡上记着的：借着的合计（MiB）、驱动那一份走到哪一步，与已经销账而卡上可能还没还的那一份。
@@ -185,11 +192,11 @@ pub fn occupied_part_of_what_this_process_promised_mebibytes(
 ) -> u64 {
     let promised_mebibytes = match booked.driver_share {
         DriverShareOnTheCard::NotRegistered => booked.leases_mebibytes,
-        DriverShareOnTheCard::RegisteredDeviceNotStarted | DriverShareOnTheCard::DeviceStarted => {
-            booked
-                .leases_mebibytes
-                .saturating_add(mebibytes_left_for_the_driver)
-        }
+        DriverShareOnTheCard::RegisteredDeviceNotStarted
+        | DriverShareOnTheCard::DeviceStarted
+        | DriverShareOnTheCard::DeviceLostNotStartedAgain => booked
+            .leases_mebibytes
+            .saturating_add(mebibytes_left_for_the_driver),
     };
     match reading.occupied_by_this_process {
         OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => mebibytes
@@ -200,21 +207,19 @@ pub fn occupied_part_of_what_this_process_promised_mebibytes(
     }
 }
 
-/// 销了账、卡上可能还没还的那一份照这一次读数往下压（MiB）：这个进程实际占着的里，早先读数已经见到占在卡上、此刻还算数的那一份
-/// （`seen_on_the_card_at_least_mebibytes`：驱动那一份与那一刻活着、此刻还活着的借据建上的，[`GpuMemoryLedger`] 每读一次记）
-/// 之外才可能是它，压到不超过那么多；读不到实际占着的、或另有不走这份账的设备，照旧（分不出就不压）。
-/// 不照「驱动那一份 D 已经整份在卡上」压：驱动实占的 x 比 D 小时压过头，被压掉的那截在下一笔里当成借据建上的，多给 D − x
-/// （第 10y 批代码轮判决 K1-1 攻方丙形）。
+/// 销了账、卡上可能还没还的那一份照这一次读数往下压（MiB）：它在卡上的那一截是这个进程实际占着的一部分，压到不超过读数里这个进程
+/// 此刻占着的；读不到实际占着的、或另有不走这份账的设备，照旧（分不出就不压）。
+/// 不往下减驱动那一份、也不减借据建上的（用户 2026-10-08 定第 10z 批代码轮岔路 G2「先回甲」，文件头「口径」）：读数只给这个进程的合计，
+/// 驱动实占的 x 比 D 小时减 D 压过头、多给 D − x（第 10y 批代码轮判决 K1-1 攻方丙形）；照早先见到的借据建上的减，借据的缓冲缩回驱动时压过头、
+/// 多给（第 10z 批代码轮判决 K2-1 第 2 条）。压过头的那截在下一笔里被当成借据已经建上的。
 #[must_use]
 pub fn returned_possibly_still_on_the_card_after_a_reading(
     returned_possibly_still_on_the_card_mebibytes: u64,
     occupied_by_this_process: OccupiedByThisProcess,
-    seen_on_the_card_at_least_mebibytes: u64,
 ) -> u64 {
     match occupied_by_this_process {
         OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => {
-            returned_possibly_still_on_the_card_mebibytes
-                .min(mebibytes.saturating_sub(seen_on_the_card_at_least_mebibytes))
+            returned_possibly_still_on_the_card_mebibytes.min(mebibytes)
         }
         OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => {
             returned_possibly_still_on_the_card_mebibytes
@@ -830,11 +835,6 @@ struct BooksOfACard {
     driver_share: DriverShareOnTheCard,
     /// 销了账、卡上可能还没还的（[`BookedByThisProcessOnACard::returned_possibly_still_on_the_card_mebibytes`]）。
     returned_possibly_still_on_the_card_mebibytes: u64,
-    /// 早先一次读数见到占在卡上的、这个进程许出去的那一份（MiB，[`occupied_part_of_what_this_process_promised_mebibytes`]：驱动那一份与借据建上的）
-    /// 里此刻还算数的：那一刻活着的借据（`leases_seen_on_the_card`）销了账就减掉它借的量，那一刻之后借的销账不减。只拿来压销了账、
-    /// 卡上可能还没还的那一份（[`returned_possibly_still_on_the_card_after_a_reading`]，第 10y 批代码轮判决 K2-2 P1）。
-    seen_on_the_card_at_least_mebibytes: u64,
-    leases_seen_on_the_card: BTreeSet<u64>,
 }
 
 impl BooksOfACard {
@@ -844,44 +844,25 @@ impl BooksOfACard {
             leases: BTreeMap::new(),
             driver_share: DriverShareOnTheCard::NotRegistered,
             returned_possibly_still_on_the_card_mebibytes: 0,
-            seen_on_the_card_at_least_mebibytes: 0,
-            leases_seen_on_the_card: BTreeSet::new(),
         }
     }
 
-    /// 读到一次这张卡的显存：销了账、卡上可能还没还的那一份先照它往下压，再看这一次见到占在卡上的许出去的那一份，
-    /// 比记着的多就换成它、连同这一刻活着的借据。
-    fn take_in_a_reading(
-        &mut self,
-        reading: CardMemoryReading,
-        mebibytes_left_for_the_driver: u64,
-    ) {
+    /// 读到一次这张卡的显存：销了账、卡上可能还没还的那一份照它往下压（[`returned_possibly_still_on_the_card_after_a_reading`]）。
+    fn take_in_a_reading(&mut self, reading: CardMemoryReading) {
         self.returned_possibly_still_on_the_card_mebibytes =
             returned_possibly_still_on_the_card_after_a_reading(
                 self.returned_possibly_still_on_the_card_mebibytes,
                 reading.occupied_by_this_process,
-                self.seen_on_the_card_at_least_mebibytes,
             );
-        let seen_now_mebibytes = occupied_part_of_what_this_process_promised_mebibytes(
-            reading,
-            self.booked(),
-            mebibytes_left_for_the_driver,
-        );
-        if seen_now_mebibytes > self.seen_on_the_card_at_least_mebibytes {
-            self.seen_on_the_card_at_least_mebibytes = seen_now_mebibytes;
-            self.leases_seen_on_the_card = self.leases.keys().copied().collect();
-        }
     }
 
-    /// 这张卡上起过的设备丢了、要重起：驱动那一份记回许出去而还没占上卡，旧设备那一份（`mebibytes_left_for_the_driver`）当销了账而卡上
-    /// 可能还没还的，早先读数见到的作废（见到的是旧设备上的）。
+    /// 这张卡上起好过的设备丢了、要重起：驱动那一份记回许出去而还没占上卡，旧设备那一份（`mebibytes_left_for_the_driver`）当销了账而卡上
+    /// 可能还没还的。
     fn take_back_the_driver_share_of_a_lost_device(&mut self, mebibytes_left_for_the_driver: u64) {
-        self.driver_share = DriverShareOnTheCard::RegisteredDeviceNotStarted;
+        self.driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;
         self.returned_possibly_still_on_the_card_mebibytes = self
             .returned_possibly_still_on_the_card_mebibytes
             .saturating_add(mebibytes_left_for_the_driver);
-        self.seen_on_the_card_at_least_mebibytes = 0;
-        self.leases_seen_on_the_card.clear();
     }
 
     fn booked(&self) -> BookedByThisProcessOnACard {
@@ -893,18 +874,13 @@ impl BooksOfACard {
         }
     }
 
-    /// 销一笔借据（借据丢掉时）：它建的缓冲记进卡上可能还没还的那一份；早先读数见到的那一份里有它的话减掉它借的量。交回销没销到。
+    /// 销一笔借据（借据丢掉时）：它建的缓冲记进卡上可能还没还的那一份。交回销没销到。
     fn return_a_lease(&mut self, lease_identifier: u64) -> bool {
         match self.leases.remove(&lease_identifier) {
             Some(mebibytes) => {
                 self.returned_possibly_still_on_the_card_mebibytes = self
                     .returned_possibly_still_on_the_card_mebibytes
                     .saturating_add(mebibytes);
-                if self.leases_seen_on_the_card.remove(&lease_identifier) {
-                    self.seen_on_the_card_at_least_mebibytes = self
-                        .seen_on_the_card_at_least_mebibytes
-                        .saturating_sub(mebibytes);
-                }
                 true
             }
             None => false,
@@ -923,7 +899,8 @@ impl BooksOfACard {
                 self.driver_share = DriverShareOnTheCard::RegisteredDeviceNotStarted;
             }
             DriverShareOnTheCard::RegisteredDeviceNotStarted
-            | DriverShareOnTheCard::DeviceStarted => {}
+            | DriverShareOnTheCard::DeviceStarted
+            | DriverShareOnTheCard::DeviceLostNotStartedAgain => {}
         }
     }
 }
@@ -972,8 +949,7 @@ impl GpuMemoryLedger {
         self.books.lock().unwrap_or_else(PoisonError::into_inner)
     }
 
-    /// 在锁里现读每张卡（次序与池一样），每张卡的账照这一次读数记（`BooksOfACard::take_in_a_reading`：销了账、卡上可能还没还的那一份往下压，
-    /// 见到占在卡上的那一份往上换）。
+    /// 在锁里现读每张卡（次序与池一样），每张卡的账照这一次读数记（`BooksOfACard::take_in_a_reading`：销了账、卡上可能还没还的那一份往下压）。
     fn readings_taken_into_the_books(
         &self,
         books: &mut [BooksOfACard],
@@ -990,8 +966,7 @@ impl GpuMemoryLedger {
             });
         }
         for (books_of_the_card, reading) in books.iter_mut().zip(&readings) {
-            books_of_the_card
-                .take_in_a_reading(*reading, self.mebibytes_each_process_keeps_on_a_card);
+            books_of_the_card.take_in_a_reading(*reading);
         }
         Ok(readings)
     }
@@ -1257,7 +1232,8 @@ impl GpuMemoryLedger {
         match self.books_locked()[card_position].driver_share {
             DriverShareOnTheCard::NotRegistered => false,
             DriverShareOnTheCard::RegisteredDeviceNotStarted
-            | DriverShareOnTheCard::DeviceStarted => true,
+            | DriverShareOnTheCard::DeviceStarted
+            | DriverShareOnTheCard::DeviceLostNotStartedAgain => true,
         }
     }
 
@@ -1276,7 +1252,9 @@ impl GpuMemoryLedger {
     /// 驱动那一份在登记的那一笔里一起借过，照旧；起过的设备丢了要重起（驱动那一份记着已经占在卡上），把它记回许出去而还没占上卡、
     /// 旧设备那一份当销了账而卡上可能还没还的（`BooksOfACard::take_back_the_driver_share_of_a_lost_device`），在锁里现读一次：
     /// 借着的连新设备的驱动那一份此刻放不下，交借不到，调用方不起（第 10y 批代码轮判决 K1-3 攻方：重起不另登记驱动那一份，
-    /// 触发重起的那一笔是按驱动那一份已在卡上借的，新设备的上下文没留地方）。交借不到之后驱动那一份照旧记着许出去而没占上卡。
+    /// 触发重起的那一笔是按驱动那一份已在卡上借的，新设备的上下文没留地方）。交借不到之后驱动那一份照旧记着许出去而没占上卡
+    /// （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]），新设备起好之前每一次再来都照此刻再判一次（第 10z 批代码轮判决 K1-3 正推 S6：
+    /// 被拒之前已借到、借据还在手上的并发用例是按丢之前的账借的，再来起设备时不直接放行）。
     ///
     /// # Errors
     /// 重起而此刻放不下（[`BookingRefusal::NoCardHasRoomNow`]，要的是驱动那一份，带着这张卡此刻能借的）；卡上的显存读不出。
@@ -1297,27 +1275,42 @@ impl GpuMemoryLedger {
                 books[card_position].take_back_the_driver_share_of_a_lost_device(
                     self.mebibytes_each_process_keeps_on_a_card,
                 );
-                let readings = self.readings_taken_into_the_books(&mut books)?;
-                let booked = books[card_position].booked();
-                let capacity_mebibytes = (self.capacity_for_this_process)(
-                    card_position,
-                    memory_this_process_may_hold_mebibytes(
-                        readings[card_position],
-                        booked,
-                        self.mebibytes_each_process_keeps_on_a_card,
-                    ),
-                );
-                if capacity_mebibytes < booked.leases_mebibytes {
-                    return Err(BookingRefusal::NoCardHasRoomNow {
-                        need_mebibytes: self.mebibytes_each_process_keeps_on_a_card,
-                        card_rooms_mebibytes: vec![
-                            capacity_mebibytes.saturating_sub(booked.leases_mebibytes)
-                        ],
-                    });
-                }
-                Ok(())
+                self.driver_share_of_a_new_device_fits_now(&mut books, card_position)
             }
+            DriverShareOnTheCard::DeviceLostNotStartedAgain => {
+                self.driver_share_of_a_new_device_fits_now(&mut books, card_position)
+            }
+        }
+    }
+
+    /// 第 `card_position` 张卡上丢了的设备要重起（驱动那一份记着许出去而还没占上卡）：在锁里现读一次，借着的连新设备的驱动那一份此刻放不放得下。
+    ///
+    /// # Errors
+    /// 同 [`Self::prepare_the_driver_share_before_starting_the_device`]。
+    fn driver_share_of_a_new_device_fits_now(
+        &self,
+        books: &mut [BooksOfACard],
+        card_position: usize,
+    ) -> Result<(), BookingRefusal> {
+        let readings = self.readings_taken_into_the_books(books)?;
+        let booked = books[card_position].booked();
+        let capacity_mebibytes = (self.capacity_for_this_process)(
+            card_position,
+            memory_this_process_may_hold_mebibytes(
+                readings[card_position],
+                booked,
+                self.mebibytes_each_process_keeps_on_a_card,
+            ),
+        );
+        if capacity_mebibytes < booked.leases_mebibytes {
+            return Err(BookingRefusal::NoCardHasRoomNow {
+                need_mebibytes: self.mebibytes_each_process_keeps_on_a_card,
+                card_rooms_mebibytes: vec![
+                    capacity_mebibytes.saturating_sub(booked.leases_mebibytes)
+                ],
+            });
         }
+        Ok(())
     }
 
     /// 第 `card_position` 张卡上的设备起好了：从此这个进程的驱动那一份算实际占在卡上（读到实际占着的时候照它算）。
@@ -1661,6 +1654,33 @@ impl DeviceConditionOfACard {
             .clone()
     }
 
+    /// 设备丢了没有（卡池照它判留着的设备还能不能拿：丢了就重起）。
+    #[must_use]
+    pub fn is_lost(&self) -> bool {
+        self.lost
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .is_some()
+    }
+
+    /// 一段活收完：`caught_in_the_scope`（这条线程上的错误范围接到的 Out of Memory，`None` 是没接到）记进设备，交 `ending` 报到、等同一个设备上
+    /// 一起开的其余几段都报到，再认这段活从记到 `reports_before` 次那一刻起出的坏事能不能说成显存不够
+    /// （[`Self::out_of_memory_explaining_a_failure_of_the_work`]）。先等再认（第 10z 批代码轮判决 K1-3 攻方：一块的几段并发，一段因设备丢了先失败，
+    /// 同块另一段范围里接到的 Out of Memory 要等那一段收完才记进设备；先认就认不出、判红）。
+    pub fn end_the_work_and_explain_its_failure(
+        &self,
+        work: &WorkOnTheCardOfThisThread,
+        reports_before: usize,
+        caught_in_the_scope: Option<String>,
+        ending: WorkEndingTogether<'_>,
+    ) -> Option<String> {
+        if let Some(caught) = caught_in_the_scope {
+            self.record_an_out_of_memory(caught);
+        }
+        ending.end_and_wait_for_the_others();
+        self.out_of_memory_explaining_a_failure_of_the_work(work.identifier(), reports_before)
+    }
+
     /// 段活 `work` 从记到 `reports_before` 次那一刻起出了一样坏事、能不能说成是它的显存不够：这之后记了算在它上面的 Out of Memory，
     /// 交最后那一句；设备丢了、这之后记过 Out of Memory（算在哪段活上都行：wgpu 把建管线、提交这类调用上的 Out of Memory 当致命、丢掉设备，
     /// 丢了设备谁都做不成），交两样都说的一句；别的交 `None`（别的活上的 Out of Memory、这段活开始之前的，都说不成）。
@@ -1691,9 +1711,338 @@ impl DeviceConditionOfACard {
     }
 }
 
+/// 同一个设备上一起开的几段活（崩溃放量单元校验和对拍时一块切成的几段，`singlefs_checker_tier::crash_amplification`）：每段收完报到一次，
+/// 认它是不是显存不够之前等其余几段都报到（[`DeviceConditionOfACard::end_the_work_and_explain_its_failure`]）。先把每一段的报到都领出来
+/// （[`Self::join`]），再起那几段：领出来的没报到就丢掉（那一段 panic 了）也算报到，别的段不会等它一辈子。
+pub struct WorksEndingTogetherOnADevice {
+    works_not_yet_ended: Mutex<usize>,
+    every_work_ended: std::sync::Condvar,
+}
+
+/// 一段都没领（还没开活）就是起点，没有别的不变量。
+impl Default for WorksEndingTogetherOnADevice {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+impl WorksEndingTogetherOnADevice {
+    #[must_use]
+    pub const fn new() -> Self {
+        Self {
+            works_not_yet_ended: Mutex::new(0),
+            every_work_ended: std::sync::Condvar::new(),
+        }
+    }
+
+    /// 领一段活的报到（这一段起之前领）。
+    #[must_use]
+    pub fn join(&self) -> WorkEndingTogether<'_> {
+        // 持锁只为改一个计数，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
+        *self
+            .works_not_yet_ended
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner) += 1;
+        WorkEndingTogether {
+            together: self,
+            has_ended: false,
+        }
+    }
+
+    /// 领了报到、还没报到的段数。
+    #[must_use]
+    pub fn works_not_yet_ended(&self) -> usize {
+        *self
+            .works_not_yet_ended
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+    }
+
+    fn end_one_work(&self) {
+        let mut works_not_yet_ended = self
+            .works_not_yet_ended
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        *works_not_yet_ended = works_not_yet_ended
+            .checked_sub(1)
+            .expect("每一份报到只报一次：领一次加一，报一次减一");
+        if *works_not_yet_ended == 0 {
+            self.every_work_ended.notify_all();
+        }
+    }
+}
+
+/// 一段活的报到（[`WorksEndingTogetherOnADevice::join`]）：[`Self::end_and_wait_for_the_others`] 报到；没报到就丢掉，丢的时候补报（不等）。
+pub struct WorkEndingTogether<'together> {
+    together: &'together WorksEndingTogetherOnADevice,
+    has_ended: bool,
+}
+
+impl WorkEndingTogether<'_> {
+    /// 这一段收完了：报到，等一起开的其余几段都报到（或丢掉了报到）再交回。
+    pub fn end_and_wait_for_the_others(mut self) {
+        self.has_ended = true;
+        self.together.end_one_work();
+        let mut works_not_yet_ended = self
+            .together
+            .works_not_yet_ended
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        while *works_not_yet_ended > 0 {
+            works_not_yet_ended = self
+                .together
+                .every_work_ended
+                .wait(works_not_yet_ended)
+                .unwrap_or_else(PoisonError::into_inner);
+        }
+    }
+}
+
+impl Drop for WorkEndingTogether<'_> {
+    fn drop(&mut self) {
+        if !self.has_ended {
+            self.together.end_one_work();
+        }
+    }
+}
+
+/// 几张卡一起判的那一队块（崩溃放量产品路径第 ① 段，`singlefs_checker_tier::crash_amplification`；块号是要判的块在那一趟 `pending` 里的次序）：
+/// 每张卡领序号最小的、还没领走的那一块；一张卡报了 Out of Memory 把手里那一块交回，交回的序号比没领过的都小，下一张来领的卡先领到它
+/// （第 10z 批，第 10y 批代码轮判决 K2-2 P2：一张卡 Out of Memory 不停整趟）。不碰卡：单测在 harness 档随时跑（文件头）。
+pub struct BlocksForTheCards {
+    blocks_not_taken: Mutex<BTreeSet<usize>>,
+}
+
+impl BlocksForTheCards {
+    /// 一队块：`pending_indices` 里的每一块（一轮只判上一轮剩下的那几块，[`after_a_round_of_the_cards`]）。
+    #[must_use]
+    pub fn new(pending_indices: impl IntoIterator<Item = usize>) -> Self {
+        Self {
+            blocks_not_taken: Mutex::new(pending_indices.into_iter().collect()),
+        }
+    }
+
+    fn blocks_not_taken_locked(&self) -> MutexGuard<'_, BTreeSet<usize>> {
+        // 持锁只为取、放一个序号，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
+        self.blocks_not_taken
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+    }
+
+    /// 领一块：序号最小的、还没领走的（交回的也算）；都领走了是 `None`。
+    #[must_use]
+    pub fn take_a_block(&self) -> Option<usize> {
+        self.blocks_not_taken_locked().pop_first()
+    }
+
+    /// 领了没判完的一块交回队里（报了 Out of Memory 的卡交）。
+    ///
+    /// # Panics
+    /// 这一块此刻就在队里（交了两次，或没领过）。
+    pub fn give_back(&self, pending_index: usize) {
+        let is_newly_given_back = self.blocks_not_taken_locked().insert(pending_index);
+        assert!(
+            is_newly_given_back,
+            "块 {pending_index} 交回时已经在队里：一块领一次只交回一次"
+        );
+    }
+
+    /// 还有没领走的块没有（没领过的或交回的）。
+    #[must_use]
+    pub fn any_left(&self) -> bool {
+        !self.blocks_not_taken_locked().is_empty()
+    }
+
+    /// 此刻还没领走的块，按序号（几张卡都收工之后，就是这一轮没判的那几块）。
+    #[must_use]
+    pub fn blocks_not_taken(&self) -> Vec<usize> {
+        self.blocks_not_taken_locked().iter().copied().collect()
+    }
+}
+
+/// 一张卡判一块的结局（[`judge_blocks_on_one_card`] 的 `judge_a_block` 交）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum BlockJudgedOnACard<Judged> {
+    Judged(Judged),
+    /// 卡上 wgpu 报了 Out of Memory（`reason` 是本次未跑那一行的原因）：这一块交回，这张卡收工。
+    OutOfMemory {
+        reason: String,
+    },
+    /// 这张卡判不了这一块里的某个状态（表越过了内核的容量这类，`reason` 说哪一个）：这一块不交回，调用方叫所有卡停下。
+    Refused {
+        reason: String,
+    },
+}
+
+/// 判完的一块交没交到写库那一头（[`judge_blocks_on_one_card`] 的 `hand_to_the_store` 交）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum BlockHandedToTheStore {
+    Handed,
+    /// 写库那一头已经不收了（写库那条线程走了）。
+    TheStoreIsGone,
+}
+
+/// 一张卡怎么收的工（[`judge_blocks_on_one_card`]）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum CardLeftTheBlocks {
+    /// 队里没有要领的了，或被叫停（`must_stop`）。
+    NothingLeftOrStopped,
+    /// 判一块时报了 Out of Memory：那一块已经交回队里。
+    OutOfMemory { reason: String },
+    /// 判不了一块：那一块没交回。
+    Refused { reason: String },
+    /// 判完的一块交不到写库那一头。
+    TheStoreIsGone,
+}
+
+/// 一张卡的线程判块的那一圈（崩溃放量产品路径第 ① 段的卡线程，判器建好之后）：每领一块之前问 `must_stop`；领一块（[`BlocksForTheCards::take_a_block`]），
+/// 交 `judge_a_block` 判；判完交 `hand_to_the_store`（写库按块序，[`BlocksStoredInOrder`]）；报了 Out of Memory 就把这一块交回队里、这张卡收工，
+/// 不再领（只停这一张卡，第 10z 批）；判不了就收工、这一块不交回。循环的次数不超过队里的块数加交回的次数（每次领走一块，每张卡至多交回一次）。
+pub fn judge_blocks_on_one_card<Judged>(
+    blocks: &BlocksForTheCards,
+    must_stop: &dyn Fn() -> bool,
+    judge_a_block: &mut dyn FnMut(usize) -> BlockJudgedOnACard<Judged>,
+    hand_to_the_store: &mut dyn FnMut(usize, Judged) -> BlockHandedToTheStore,
+) -> CardLeftTheBlocks {
+    while !must_stop() {
+        let Some(pending_index) = blocks.take_a_block() else {
+            break;
+        };
+        match judge_a_block(pending_index) {
+            BlockJudgedOnACard::Judged(judged) => match hand_to_the_store(pending_index, judged) {
+                BlockHandedToTheStore::Handed => {}
+                BlockHandedToTheStore::TheStoreIsGone => return CardLeftTheBlocks::TheStoreIsGone,
+            },
+            BlockJudgedOnACard::OutOfMemory { reason } => {
+                blocks.give_back(pending_index);
+                return CardLeftTheBlocks::OutOfMemory { reason };
+            }
+            BlockJudgedOnACard::Refused { reason } => return CardLeftTheBlocks::Refused { reason },
+        }
+    }
+    CardLeftTheBlocks::NothingLeftOrStopped
+}
+
+/// 判完的块按块序写库（判定向量的编号按块序登记，与用了几张卡无关）：块号从 0 起连着，后面的块先到就等着，前面那一块到了一起写；
+/// 写库失败之后到的不再写。几轮卡（[`after_a_round_of_the_cards`]）共用一份：上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写，
+/// 不再判一遍。
+pub struct BlocksStoredInOrder<Block> {
+    blocks_stored: usize,
+    waiting: BTreeMap<usize, Block>,
+    storing_error: Option<String>,
+}
+
+/// 一块都没写、没有等着的就是起点，没有别的不变量。
+impl<Block> Default for BlocksStoredInOrder<Block> {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+impl<Block> BlocksStoredInOrder<Block> {
+    #[must_use]
+    pub const fn new() -> Self {
+        Self {
+            blocks_stored: 0,
+            waiting: BTreeMap::new(),
+            storing_error: None,
+        }
+    }
+
+    /// 块号 `pending_index` 的一块判完到了：照块序交 `store_block` 写（它和前面连着的、等着的一起）；写失败记下那一句。
+    pub fn arrive(
+        &mut self,
+        pending_index: usize,
+        block: Block,
+        store_block: &mut dyn FnMut(Block) -> Result<(), String>,
+    ) {
+        self.waiting.insert(pending_index, block);
+        while let Some(next_block) = self.waiting.remove(&self.blocks_stored) {
+            if self.storing_error.is_some() {
+                continue;
+            }
+            match store_block(next_block) {
+                Ok(()) => self.blocks_stored += 1,
+                Err(error) => self.storing_error = Some(error),
+            }
+        }
+    }
+
+    /// 照块序写进库了几块（块号 `0..` 这几块）。
+    #[must_use]
+    pub fn blocks_stored(&self) -> usize {
+        self.blocks_stored
+    }
+
+    /// 判完了、等着前面那一块的有几块。
+    #[must_use]
+    pub fn blocks_waiting(&self) -> usize {
+        self.waiting.len()
+    }
+
+    /// 写库失败的那一句（没失败是 `None`）。
+    #[must_use]
+    pub fn storing_error(&self) -> Option<&str> {
+        self.storing_error.as_deref()
+    }
+}
+
+/// 调用方叫没叫停（每领一块之前问的那一个）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum StopOfTheRun {
+    Requested,
+    NotRequested,
+}
+
+/// 一轮卡都收工之后这一趟怎么走（[`after_a_round_of_the_cards`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum AfterARoundOfTheCards {
+    /// 要判的块都写进了库。
+    EveryBlockStored,
+    /// 这一轮没有卡报 Out of Memory、还有块没写：被叫停，没判的留给下一趟。
+    StoppedOnRequest,
+    /// 这一轮有卡报了 Out of Memory、还有块没写、没被叫停，而还有没报过的卡：收尾这一步再起一轮，只用没报过的卡判剩下的块
+    /// （第 10z 批代码轮判决 K2-1 第 5 条：交回的块在别的卡都收工之后也要有人判，状态全装在第一张卡时也要起第二张）。
+    AnotherRoundWithoutTheCardsShortOfMemory,
+    /// 有卡报了 Out of Memory、还有块没写，而报过的卡之外没有卡了（或被叫停）：这一趟报本次未跑，写进库的块在库里。
+    NotRunTheGpuMemoryWasShort,
+}
+
+/// 一轮卡都收工之后：`blocks_left_unstored` 块还没写进库（共要判的减照块序写进的，[`BlocksStoredInOrder::blocks_stored`]），这一轮
+/// `cards_short_of_memory_this_round` 张卡报了 Out of Memory，到这一轮为止没报过的卡还有 `cards_not_short_of_memory` 张。
+/// 每再起一轮至少少一张卡，轮数不超过卡数。
+#[must_use]
+pub fn after_a_round_of_the_cards(
+    blocks_left_unstored: usize,
+    cards_short_of_memory_this_round: usize,
+    stop: StopOfTheRun,
+    cards_not_short_of_memory: usize,
+) -> AfterARoundOfTheCards {
+    if blocks_left_unstored == 0 {
+        return AfterARoundOfTheCards::EveryBlockStored;
+    }
+    if cards_short_of_memory_this_round == 0 {
+        return AfterARoundOfTheCards::StoppedOnRequest;
+    }
+    match stop {
+        StopOfTheRun::Requested => AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,
+        StopOfTheRun::NotRequested if cards_not_short_of_memory == 0 => {
+            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
+        }
+        StopOfTheRun::NotRequested => {
+            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
+        }
+    }
+}
+
 #[cfg(test)]
 mod tests {
     use super::DeviceStartedOnDemand;
+    use super::{
+        after_a_round_of_the_cards, judge_blocks_on_one_card, AfterARoundOfTheCards,
+        BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards, BlocksStoredInOrder,
+        CardLeftTheBlocks, StopOfTheRun, WorksEndingTogetherOnADevice,
+    };
     use super::{
         attempt_on_a_card_without_room_for_one_state, card_memory_reader_of_one_report_per_reading,
         card_memory_readings_of_one_nvidia_smi_report, decide_scratch_before_booking,
@@ -2939,8 +3288,9 @@ mod tests {
         );
         assert_eq!(
             ledger.card_rooms_mebibytes_now(&[0]),
-            Ok(vec![30_000 - DRIVER - 4_000]),
-            "卡上还了第一笔：那一份照读数压掉，只扣第二笔还要建的"
+            Ok(vec![30_000 - DRIVER - 4_000 - DRIVER]),
+            "卡上还了第一笔：那一份只压到这个进程占着的驱动那一份 2 560 为止，照旧多扣这 2 560（第 10z 批代码轮岔路 G2 甲认的少借），\
+             只扣第二笔还要建的是 24 880"
         );
         drop(second);
     }
@@ -3186,10 +3536,11 @@ mod tests {
     }
 
     /// 第 10z 批（第 10y 批代码轮判决 K1-1 攻方丙形，数照攻方模型那一行：5090 空卡 32 110，外人占 10 000，借 4 000、设备起好、建满之后丢掉，
-    /// 驱动实占 600，那 4 000 卡上还没还）：销了账的那一份不照「驱动那一份整份在卡上」压，下一笔能借的是卡上此刻真能再给的 15 550
-    /// （空 17 510 减驱动还要长的 1 960）；照 D 压是 17 510，多给 D − x。
+    /// 驱动实占 600，那 4 000 卡上还没还）：销了账的那一份只压到这个进程占着的 4 600 为止，不再减驱动那一份，下一笔能借的是卡上此刻真能再给的
+    /// 15 550（空 17 510 减驱动还要长的 1 960）；减驱动那一份压是 17 510，多给 D − x（第 10z 批代码轮岔路 G2 甲：压到低于这个进程占着的要红）。
     #[test]
-    fn a_returned_lease_is_pressed_by_what_was_seen_on_the_card_not_by_a_whole_driver_share() {
+    fn a_returned_lease_is_pressed_only_to_what_this_process_occupies_not_below_by_a_whole_driver_share(
+    ) {
         const EMPTY: u64 = 32_110;
         const OUTSIDER: u64 = 10_000;
         const DRIVER_HOLDS_NOW: u64 = 600;
@@ -3242,11 +3593,13 @@ mod tests {
         );
     }
 
-    /// 第 10z 批（第 10y 批代码轮判决 K2-2 P1，数照正推在真账上造出的那一组）：一笔 10 000 的长借据建满之后，十笔 1 000 的短借据各借、建、丢，
-    /// 丢了之后卡上马上还给驱动（这个进程占的回到驱动那一份加 10 000）。每还一笔，能借的照旧是真空闲 19 550（不再一笔一笔多扣）；
-    /// 要 18 550 放得下，借得到。
+    /// 第 10z 批（第 10y 批代码轮判决 K2-2 P1，数照正推在真账上造出的那一组 S10）：一笔 10 000 的长借据建满之后，十笔 1 000 的短借据各借、建、丢，
+    /// 丢了之后卡上马上还给驱动（这个进程占的回到驱动那一份加 10 000）。销了账的那一份只压到这个进程占着的 12 560 为止
+    /// （第 10z 批代码轮岔路 G2 甲），十笔 10 000 都压不掉：第 k 笔还了之后能借的是 19 550 − 1 000k，真空闲一直是 19 550，第 10 笔之后
+    /// 要 18 550 放得下却借不到（能借的 9 550），报本次未跑。这是甲认的代价（少借，不多给），钉住它：压得比这个进程占着的更低就会多出来。
     #[test]
-    fn returned_short_leases_the_card_gave_back_cost_nothing_while_a_long_lease_stays_built() {
+    fn returned_short_leases_while_a_long_lease_stays_built_each_stay_booked_as_possibly_on_the_card(
+    ) {
         const EMPTY: u64 = 32_110;
         const LONG_LEASE: u64 = 10_000;
         const SHORT_LEASE: u64 = 1_000;
@@ -3308,15 +3661,18 @@ mod tests {
             );
             assert_eq!(
                 ledger.card_rooms_mebibytes_now(&[0]),
-                Ok(vec![free_while_the_long_lease_is_built]),
-                "第 {short_lease_number} 笔短借据还了、卡上已经还给驱动：不多扣"
+                Ok(vec![free_while_the_long_lease_is_built - SHORT_LEASE * short_lease_number]),
+                "第 {short_lease_number} 笔短借据还了、卡上已经还给驱动：这个进程占着的 12 560 压不掉还了的那几笔，每笔多扣 1 000（甲的代价）"
             );
         }
         let need = free_while_the_long_lease_is_built - SHORT_LEASE;
         assert_eq!(
             booking_kept_on_the_books(&ledger, need),
-            Ok(need),
-            "要 18 550、真空闲 19 550：放得下，借得到"
+            Err(BookingRefusal::NoCardHasRoomNow {
+                need_mebibytes: need,
+                card_rooms_mebibytes: vec![free_while_the_long_lease_is_built - SHORT_LEASE * 10],
+            }),
+            "要 18 550、真空闲 19 550：放得下，账上只给 9 550，借不到（甲的代价，报本次未跑）"
         );
         drop(long_lease);
     }
@@ -3585,8 +3941,7 @@ mod tests {
         assert_eq!(
             returned_possibly_still_on_the_card_after_a_reading(
                 4_000,
-                OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 1_000 },
-                0
+                OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 1_000 }
             ),
             1_000,
             "都走这份账：压到占着的那么多"
@@ -3594,8 +3949,7 @@ mod tests {
         assert_eq!(
             returned_possibly_still_on_the_card_after_a_reading(
                 4_000,
-                OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 1_000 },
-                0
+                OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 1_000 }
             ),
             4_000,
             "另有不走账的设备：分不出，不压"
@@ -3603,8 +3957,7 @@ mod tests {
         assert_eq!(
             returned_possibly_still_on_the_card_after_a_reading(
                 4_000,
-                OccupiedByThisProcess::NotListed,
-                0
+                OccupiedByThisProcess::NotListed
             ),
             4_000,
             "没列这个进程：不压"
@@ -3632,4 +3985,583 @@ mod tests {
             }
         );
     }
+
+    /// 第 10z 批第三块（第 10z 批代码轮判决 K2-1 第 2 条，数照攻方模型「丁」：5090 空卡 32 110，驱动实占 600）：长借据 10 000、C 8 000、V 3 000 都建满、
+    /// 读过一次；C 销了账、卡上还没还，V 这一趟派完、缓冲缩回驱动（V 的借据还活着）。销了账的那一份只压到这个进程占着的 18 600 为止、照旧 8 000，
+    /// 能借的是卡上此刻真能再给的 8 550（空 13 510 减驱动还要长的 1 960、减 V 还要长回的 3 000）；照活着的借据都建满往下压是 10 950，
+    /// 照早先见到的那一份压是 11 550，照账借满卡上就超了（第 10z 批代码轮岔路 G2 甲：压到低于这个进程占着的要红）。
+    #[test]
+    fn a_returned_lease_still_on_the_card_is_not_pressed_away_when_a_live_lease_shrinks_back_to_the_driver(
+    ) {
+        const EMPTY: u64 = 32_110;
+        const DRIVER_HOLDS_NOW: u64 = 600;
+        const LONG_LEASE: u64 = 10_000;
+        const RETURNED_LEASE: u64 = 8_000;
+        const SHRINKING_LEASE: u64 = 3_000;
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
+                    Some(32_000),
+                    may_hold_mebibytes.min(32_000),
+                    may_hold_mebibytes,
+                    DRIVER,
+                )
+            }),
+            DRIVER,
+        );
+        let occupied = |mebibytes: u64| CardMemoryReading {
+            free_mebibytes: EMPTY - mebibytes,
+            total_less_reserved_mebibytes: EMPTY,
+            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
+        };
+        let long_lease = ledger.book_on_the_card(0, LONG_LEASE).expect("空卡借得到");
+        ledger.record_the_device_started_on_the_card(0);
+        let returned_lease = ledger
+            .book_on_the_card(0, RETURNED_LEASE)
+            .expect("卡上空着 19 440");
+        let shrinking_lease = ledger
+            .book_on_the_card(0, SHRINKING_LEASE)
+            .expect("卡上空着 11 440");
+        replace_the_reading(
+            &readings,
+            0,
+            occupied(DRIVER_HOLDS_NOW + LONG_LEASE + RETURNED_LEASE + SHRINKING_LEASE),
+        );
+        assert!(
+            ledger.card_rooms_mebibytes_now(&[0]).is_ok(),
+            "三笔都建满时读一次"
+        );
+        drop(returned_lease);
+        replace_the_reading(
+            &readings,
+            0,
+            occupied(DRIVER_HOLDS_NOW + LONG_LEASE + RETURNED_LEASE),
+        );
+        let free_now = EMPTY - DRIVER_HOLDS_NOW - LONG_LEASE - RETURNED_LEASE;
+        let truly_lendable = free_now - (DRIVER - DRIVER_HOLDS_NOW) - SHRINKING_LEASE;
+        assert_eq!(
+            ledger.card_rooms_mebibytes_now(&[0]),
+            Ok(vec![truly_lendable]),
+            "C 卡上还没还、V 缩回驱动：能借的是卡上此刻真能再给的 8 550，不多给"
+        );
+        drop(shrinking_lease);
+        drop(long_lease);
+    }
+
+    /// 第 10z 批第三块（第 10z 批代码轮判决 K1-3 正推 S6）：丢了的设备重起被拒过一次（卡上空 17 440，第二笔 17 440 还没建），被拒之前已借到的
+    /// 并发用例再来起设备，照此刻再判、照样拒，不直接放行；旧设备与它上面的缓冲都还给驱动之后（卡上空 30 000、这个进程不在报告里）再来，放得下，起。
+    #[test]
+    fn a_lost_device_refused_a_restart_is_judged_again_on_every_later_attempt() {
+        let (ledger, readings) =
+            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(
+                30_000 - DRIVER - 10_000,
+            );
+        let refused = Err(BookingRefusal::NoCardHasRoomNow {
+            need_mebibytes: DRIVER,
+            card_rooms_mebibytes: vec![0],
+        });
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            refused,
+            "头一次重起：放不下，不起"
+        );
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            refused,
+            "被拒之前已借到的用例再来：照此刻再判，照样放不下"
+        );
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            refused,
+            "再来一次照旧"
+        );
+        replace_the_reading(&readings, 0, reading_of_an_rtx_5090(30_000));
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(()),
+            "旧设备还给驱动之后：借着的 27 440 连新设备的驱动那一份放得下，起"
+        );
+        ledger.record_the_device_started_on_the_card(0);
+        assert!(ledger.is_this_process_registered_on_the_card(0));
+    }
+
+    /// 第 10z 批第三块（第 10z 批代码轮正推 N10、攻方第三节第 1310 行）：重起时借着的连新设备的驱动那一份恰好放得下（此刻这个进程能占的 24 880，
+    /// 借着的 10 000 + 14 880），起，不拒。
+    #[test]
+    fn a_lost_device_started_again_fits_when_its_driver_share_fits_exactly() {
+        let (ledger, _readings) =
+            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(14_880);
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(()),
+            "恰好放得下：起"
+        );
+    }
+
+    /// 第 10z 批第三块（第 10z 批代码轮正推 N1）：同一条线程上活里又起一段活，里面那段丢掉之后，这条线程上记的 Out of Memory 算回外面那段。
+    #[test]
+    fn a_work_begun_inside_another_hands_the_thread_back_to_the_outer_work_when_it_ends() {
+        let condition = DeviceConditionOfACard::default();
+        let outer_work = WorkOnTheCardOfThisThread::begin();
+        let reports_before = condition.out_of_memory_reports();
+        drop(WorkOnTheCardOfThisThread::begin());
+        condition.record_an_out_of_memory("里面那段收完之后外面那段撞满".to_string());
+        assert_eq!(
+            condition.out_of_memory_explaining_a_failure_of_the_work(
+                outer_work.identifier(),
+                reports_before
+            ),
+            Some("里面那段收完之后外面那段撞满".to_string())
+        );
+    }
+
+    /// 第 10z 批第三块（第 10z 批代码轮正推 N2、N3）：同一段活在这个设备上同时两处编内核，里面那一处编完、外面那一处还在编时，
+    /// 没有活的线程上记的 Out of Memory 照样算这段活的。
+    #[test]
+    fn a_work_compiling_in_two_places_at_once_stays_compiling_until_both_end() {
+        let condition = DeviceConditionOfACard::default();
+        let compiling_work = WorkOnTheCardOfThisThread::begin();
+        let reports_before = condition.out_of_memory_reports();
+        condition.compile_for_the_work_of_this_thread(|| {
+            condition.compile_for_the_work_of_this_thread(|| {});
+            std::thread::scope(|scope| {
+                scope.spawn(|| {
+                    condition.record_an_out_of_memory("外面那一处还在编时没接住的".to_string());
+                });
+            });
+        });
+        assert_eq!(
+            condition.out_of_memory_explaining_a_failure_of_the_work(
+                compiling_work.identifier(),
+                reports_before
+            ),
+            Some("外面那一处还在编时没接住的".to_string())
+        );
+    }
+
+    /// 第 10z 批第三块（第 10z 批代码轮正推 K3 第 3 条）：记了丢失的设备是丢了的，没记的不是（卡池照它判留着的设备要不要重起）。
+    #[test]
+    fn a_device_recorded_lost_is_lost_and_another_is_not() {
+        let condition = DeviceConditionOfACard::default();
+        assert!(!condition.is_lost(), "没记丢失");
+        condition.record_the_device_lost("Unknown：驱动重置".to_string());
+        assert!(condition.is_lost(), "记了丢失");
+    }
+
+    /// 第 10z 批第三块（第 10z 批代码轮判决 K1-3 攻方）：一块的两段在同一个设备上并发，一段因设备丢了先失败、先来认；另一段范围里接到的
+    /// Out of Memory 要等那一段先收完才记进设备。先失败的那一段等两段都报到再认：认得出是设备因显存不够丢了，不判红。
+    #[test]
+    fn a_piece_failing_on_a_lost_device_is_told_after_another_piece_records_its_out_of_memory() {
+        let condition = &DeviceConditionOfACard::default();
+        let together = &WorksEndingTogetherOnADevice::new();
+        let failed_piece_ending = together.join();
+        let caught_piece_ending = together.join();
+        let explained = std::thread::scope(|scope| {
+            let failed_piece = scope.spawn(move || {
+                let work = WorkOnTheCardOfThisThread::begin();
+                let reports_before = condition.out_of_memory_reports();
+                condition.record_the_device_lost("Unknown：提交时显存不够".to_string());
+                condition.end_the_work_and_explain_its_failure(
+                    &work,
+                    reports_before,
+                    None,
+                    failed_piece_ending,
+                )
+            });
+            scope.spawn(move || {
+                let work = WorkOnTheCardOfThisThread::begin();
+                let reports_before = condition.out_of_memory_reports();
+                let waiting_since = std::time::Instant::now();
+                while together.works_not_yet_ended() > 1 {
+                    assert!(
+                        waiting_since.elapsed() < std::time::Duration::from_secs(60),
+                        "先失败的那一段一分钟都没报到"
+                    );
+                    std::thread::yield_now();
+                }
+                condition.end_the_work_and_explain_its_failure(
+                    &work,
+                    reports_before,
+                    Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
+                    caught_piece_ending,
+                )
+            });
+            failed_piece.join().expect("先失败的那一段不 panic")
+        });
+        assert!(
+            explained
+                .as_deref()
+                .is_some_and(|reason| reason.starts_with("设备丢了")),
+            "等另一段记进设备之后再认：说成设备因显存不够丢了，{explained:?}"
+        );
+    }
+
+    /// 第 10z 批第三块：一段领了报到、没报到就丢掉（那一段 panic 了），算它报到了，别的段不等它。
+    #[test]
+    fn a_piece_dropping_its_ending_without_ending_does_not_keep_the_others_waiting() {
+        let together: &'static WorksEndingTogetherOnADevice =
+            Box::leak(Box::new(WorksEndingTogetherOnADevice::new()));
+        let dropped_ending = together.join();
+        let waiting_ending = together.join();
+        let (ended_sender, ended_receiver) = std::sync::mpsc::channel();
+        std::thread::spawn(move || {
+            waiting_ending.end_and_wait_for_the_others();
+            let _ = ended_sender.send(());
+        });
+        drop(dropped_ending);
+        assert_eq!(
+            ended_receiver.recv_timeout(std::time::Duration::from_secs(60)),
+            Ok(()),
+            "丢掉的那一份算报到：另一段一分钟之内等完"
+        );
+        assert_eq!(together.works_not_yet_ended(), 0);
+    }
+
+    /// 第 10z 批（第 10y 批代码轮判决 K2-2 P2）：几张卡一起领的那一队块，报了 Out of Memory 的卡交回的那一块先给下一张来领的卡；
+    /// 交回的都领走、队里也领完才算没有剩下的。
+    #[test]
+    fn a_block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one() {
+        let blocks = BlocksForTheCards::new(0..3);
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
+        assert_eq!(blocks.blocks_not_taken(), vec![1]);
+        assert_eq!(blocks.take_a_block(), Some(1));
+        assert_eq!(blocks.take_a_block(), None);
+    }
+
+    /// 第 10z 批第三块：判完的块按块序写库，后面的先到就等着；写库失败之后到的不再写。
+    #[test]
+    fn judged_blocks_are_stored_in_block_order_and_none_after_a_storing_error() {
+        let mut stored_in_order: BlocksStoredInOrder<usize> = BlocksStoredInOrder::new();
+        let mut stored = Vec::new();
+        for (pending_index, arrived_block) in [(2, 2), (0, 0), (1, 1), (3, 3)] {
+            stored_in_order.arrive(pending_index, arrived_block, &mut |block| {
+                stored.push(block);
+                Ok(())
+            });
+        }
+        assert_eq!(stored, vec![0, 1, 2, 3], "按块序写");
+        assert_eq!(stored_in_order.blocks_stored(), 4);
+        assert_eq!(stored_in_order.blocks_waiting(), 0);
+        let mut failing: BlocksStoredInOrder<usize> = BlocksStoredInOrder::new();
+        for (pending_index, arrived_block) in [(1, 1), (0, 0), (2, 2)] {
+            failing.arrive(pending_index, arrived_block, &mut |block| match block {
+                1 => Err("写判定块失败".to_string()),
+                _ => Ok(()),
+            });
+        }
+        assert_eq!(failing.blocks_stored(), 1, "块 1 写失败：只写进了块 0");
+        assert_eq!(failing.storing_error(), Some("写判定块失败"));
+    }
+
+    /// 第 10z 批第三块：一轮卡收工之后这一趟怎么走，逐种。
+    #[test]
+    fn after_a_round_another_round_starts_only_when_a_card_ran_out_and_another_card_is_left() {
+        assert_eq!(
+            after_a_round_of_the_cards(0, 1, StopOfTheRun::NotRequested, 1),
+            AfterARoundOfTheCards::EveryBlockStored,
+            "有卡报了而别的卡判完了：照常交回"
+        );
+        assert_eq!(
+            after_a_round_of_the_cards(3, 0, StopOfTheRun::Requested, 2),
+            AfterARoundOfTheCards::StoppedOnRequest
+        );
+        assert_eq!(
+            after_a_round_of_the_cards(3, 1, StopOfTheRun::NotRequested, 1),
+            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
+        );
+        assert_eq!(
+            after_a_round_of_the_cards(3, 1, StopOfTheRun::NotRequested, 0),
+            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,
+            "报过的卡之外没有卡了"
+        );
+        assert_eq!(
+            after_a_round_of_the_cards(3, 1, StopOfTheRun::Requested, 1),
+            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,
+            "叫停了不再起一轮"
+        );
+    }
+
+    /// P2 那几条用例的假卡按拍走的挂钟（次序照攻方模型 `research/prompts/m3-batch10z-a-code-r1-opus-model/p2_blocks_10z.rs`）：每张卡判一块占几拍；
+    /// 挂钟只在没有一张卡在跑（都停在判块上、或收了工）时往前走，最先判完的那一张先往下，同一拍卡号大的先。起卡的那一方也算在跑，
+    /// 卡一张一张起完它才让开，卡领第一块的次序因此照卡号。
+    struct CardsOnAClockOfTicks {
+        clock: Mutex<ClockOfTicks>,
+        changed: std::sync::Condvar,
+    }
+
+    struct ClockOfTicks {
+        now: u64,
+        running: usize,
+        judging_until: std::collections::BTreeSet<(u64, std::cmp::Reverse<usize>)>,
+    }
+
+    impl CardsOnAClockOfTicks {
+        fn started_by_the_one_starting_the_cards() -> Self {
+            Self {
+                clock: Mutex::new(ClockOfTicks {
+                    now: 0,
+                    running: 1,
+                    judging_until: std::collections::BTreeSet::new(),
+                }),
+                changed: std::sync::Condvar::new(),
+            }
+        }
+
+        fn locked(&self) -> std::sync::MutexGuard<'_, ClockOfTicks> {
+            self.clock
+                .lock()
+                .unwrap_or_else(std::sync::PoisonError::into_inner)
+        }
+
+        fn wait<'clock>(
+            &self,
+            clock: std::sync::MutexGuard<'clock, ClockOfTicks>,
+        ) -> std::sync::MutexGuard<'clock, ClockOfTicks> {
+            self.changed
+                .wait(clock)
+                .unwrap_or_else(std::sync::PoisonError::into_inner)
+        }
+
+        fn a_card_joins(&self) {
+            self.locked().running += 1;
+        }
+
+        /// 起卡的那一方等它刚起的那张卡停在判块上或收了工。
+        fn wait_until_only_the_one_starting_the_cards_runs(&self) {
+            let mut clock = self.locked();
+            while clock.running > 1 {
+                clock = self.wait(clock);
+            }
+        }
+
+        fn one_running_stops(&self) {
+            let mut clock = self.locked();
+            clock.running -= 1;
+            self.changed.notify_all();
+        }
+
+        fn judge_for(&self, card: usize, ticks: u64) {
+            let mut clock = self.locked();
+            let until = (clock.now + ticks, std::cmp::Reverse(card));
+            clock.judging_until.insert(until);
+            clock.running -= 1;
+            self.changed.notify_all();
+            while !(clock.running == 0 && clock.judging_until.first() == Some(&until)) {
+                clock = self.wait(clock);
+            }
+            clock.judging_until.pop_first();
+            clock.now = until.0;
+            clock.running += 1;
+            self.changed.notify_all();
+        }
+    }
+
+    /// 一张假卡的线程收了工（正常收或 panic 了）就不再算在跑。
+    struct CardRunningOnTheClock<'clock>(&'clock CardsOnAClockOfTicks);
+
+    impl Drop for CardRunningOnTheClock<'_> {
+        fn drop(&mut self) {
+            self.0.one_running_stops();
+        }
+    }
+
+    /// 假卡上几轮判完一队块的结局：照块序写进库的块、每块判了几遍、最后一轮之后怎么走。
+    struct BlocksJudgedOnFakeCards {
+        stored: Vec<usize>,
+        judged_times: Vec<usize>,
+        after_the_last_round: AfterARoundOfTheCards,
+    }
+
+    /// 几张假卡几轮判 `pending_blocks` 块（照崩溃放量产品路径第 ① 段的走法：一轮里卡一张一张起，从一队块里领、判、交去按块序写库；
+    /// 一轮收工之后照 [`after_a_round_of_the_cards`] 走）：第 `card` 张判一块占 `ticks_of_each_card[card]` 拍；头一轮只起前
+    /// `cards_holding_states_in_the_first_round` 张（装了状态的），再起的一轮起没报过 Out of Memory 的第一张（收尾那一步再起一张还有空的卡）；
+    /// 第 0 张判它领的第 `out_of_memory_at_block_of_card_0` 块（从 1 数）到末尾时报 Out of Memory，报了之后再给它块就 panic（报了就收工）。
+    fn blocks_judged_on_fake_cards(
+        pending_blocks: usize,
+        ticks_of_each_card: &[u64],
+        cards_holding_states_in_the_first_round: usize,
+        out_of_memory_at_block_of_card_0: usize,
+    ) -> BlocksJudgedOnFakeCards {
+        let stored_in_order: Mutex<BlocksStoredInOrder<usize>> =
+            Mutex::new(BlocksStoredInOrder::new());
+        let stored = Mutex::new(Vec::new());
+        let judged_times = Mutex::new(vec![0usize; pending_blocks]);
+        let blocks_taken_by_card_0 = AtomicUsize::new(0);
+        let mut cards_short_of_memory: Vec<usize> = Vec::new();
+        let mut blocks_of_the_round: Vec<usize> = (0..pending_blocks).collect();
+        let mut round = 0;
+        loop {
+            let cards_of_the_round: Vec<usize> = if round == 0 {
+                (0..cards_holding_states_in_the_first_round).collect()
+            } else {
+                (0..ticks_of_each_card.len())
+                    .filter(|card| !cards_short_of_memory.contains(card))
+                    .take(1)
+                    .collect()
+            };
+            let blocks = BlocksForTheCards::new(blocks_of_the_round.clone());
+            let clock = CardsOnAClockOfTicks::started_by_the_one_starting_the_cards();
+            let cards_left: Vec<(usize, CardLeftTheBlocks)> = std::thread::scope(|scope| {
+                let mut card_threads = Vec::new();
+                for card in cards_of_the_round.iter().copied() {
+                    clock.a_card_joins();
+                    let (blocks, clock, stored_in_order, stored, judged_times) =
+                        (&blocks, &clock, &stored_in_order, &stored, &judged_times);
+                    let blocks_taken_by_card_0 = &blocks_taken_by_card_0;
+                    card_threads.push((
+                        card,
+                        scope.spawn(move || {
+                            let _running = CardRunningOnTheClock(clock);
+                            judge_blocks_on_one_card(
+                                blocks,
+                                &|| false,
+                                &mut |pending_index| {
+                                    let taken = match card {
+                                        0 => blocks_taken_by_card_0.fetch_add(1, Ordering::SeqCst) + 1,
+                                        _ => 0,
+                                    };
+                                    assert!(
+                                        card != 0 || taken <= out_of_memory_at_block_of_card_0,
+                                        "第 0 张卡报了 Out of Memory 之后又领了块 {pending_index}：报了就收工"
+                                    );
+                                    clock.judge_for(card, ticks_of_each_card[card]);
+                                    if card == 0 && taken == out_of_memory_at_block_of_card_0 {
+                                        return BlockJudgedOnACard::OutOfMemory {
+                                            reason: format!("显存不够：第 0 张卡判块 {pending_index}"),
+                                        };
+                                    }
+                                    judged_times.lock().expect("判块的线程不 panic")[pending_index] += 1;
+                                    BlockJudgedOnACard::Judged(pending_index)
+                                },
+                                &mut |pending_index, judged| {
+                                    stored_in_order.lock().expect("写库不 panic").arrive(
+                                        pending_index,
+                                        judged,
+                                        &mut |block| {
+                                            stored.lock().expect("写库不 panic").push(block);
+                                            Ok(())
+                                        },
+                                    );
+                                    BlockHandedToTheStore::Handed
+                                },
+                            )
+                        }),
+                    ));
+                    clock.wait_until_only_the_one_starting_the_cards_runs();
+                }
+                clock.one_running_stops();
+                card_threads
+                    .into_iter()
+                    .map(|(card, thread)| (card, thread.join().expect("假卡的线程不 panic")))
+                    .collect()
+            });
+            let short_this_round: Vec<usize> = cards_left
+                .iter()
+                .filter(|(_, left)| matches!(left, CardLeftTheBlocks::OutOfMemory { .. }))
+                .map(|(card, _)| *card)
+                .collect();
+            cards_short_of_memory.extend(&short_this_round);
+            blocks_of_the_round = blocks.blocks_not_taken();
+            let blocks_stored = stored_in_order
+                .lock()
+                .expect("写库不 panic")
+                .blocks_stored();
+            match after_a_round_of_the_cards(
+                pending_blocks - blocks_stored,
+                short_this_round.len(),
+                StopOfTheRun::NotRequested,
+                ticks_of_each_card.len() - cards_short_of_memory.len(),
+            ) {
+                AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory => round += 1,
+                after_the_last_round @ (AfterARoundOfTheCards::EveryBlockStored
+                | AfterARoundOfTheCards::StoppedOnRequest
+                | AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort) => {
+                    return BlocksJudgedOnFakeCards {
+                        stored: stored.into_inner().expect("写库不 panic"),
+                        judged_times: judged_times.into_inner().expect("判块的线程不 panic"),
+                        after_the_last_round,
+                    };
+                }
+            }
+        }
+    }
+
+    /// 第 10z 批第三块（第 10z 批代码轮判决 K2-1 第 5 条，数照攻方模型那三组快慢：20 块，第 0 张卡判一块是第 1 张的 1、2、3 倍）：第 0 张卡在它领的
+    /// 第 1 到 20 块中任一块末尾报 Out of Memory、交回那一块，别的卡都已收工时也由收尾那一步再起的一轮判掉；每一格 20 块都照块序写进库、
+    /// 没有一块判两遍。改前快慢 3:1、第 5 块那一格交回的没人判、它后面判完的 2 块也不入库。
+    #[test]
+    fn a_block_a_card_gives_back_is_judged_once_and_every_block_is_stored_at_any_speed_of_two_cards(
+    ) {
+        const PENDING_BLOCKS: usize = 20;
+        for ticks_of_each_card in [[1u64, 1], [2, 1], [3, 1]] {
+            for out_of_memory_at_block_of_card_0 in 1..=PENDING_BLOCKS {
+                let judged = blocks_judged_on_fake_cards(
+                    PENDING_BLOCKS,
+                    &ticks_of_each_card,
+                    2,
+                    out_of_memory_at_block_of_card_0,
+                );
+                let cell = format!(
+                    "快慢 {ticks_of_each_card:?}、第 0 张卡第 {out_of_memory_at_block_of_card_0} 块报"
+                );
+                assert_eq!(
+                    judged.after_the_last_round,
+                    AfterARoundOfTheCards::EveryBlockStored,
+                    "{cell}"
+                );
+                assert_eq!(
+                    judged.stored,
+                    (0..PENDING_BLOCKS).collect::<Vec<usize>>(),
+                    "{cell}：每一块都照块序写进库"
+                );
+                assert_eq!(
+                    judged.judged_times,
+                    vec![1; PENDING_BLOCKS],
+                    "{cell}：每一块判一遍"
+                );
+            }
+        }
+    }
+
+    /// 第 10z 批第三块（第 10z 批代码轮判决 K2-1 第 5 条攻方另一形）：状态全装在第一张卡（第二张头一轮没起），第一张在它的第 5 块报了：
+    /// 收尾那一步再起一轮、第二张判剩下的，20 块都照块序写进库、没有一块判两遍。改前第二张闲着、这一趟报本次未跑。
+    #[test]
+    fn with_every_state_on_the_first_card_the_blocks_it_leaves_are_judged_on_the_second_card() {
+        let judged = blocks_judged_on_fake_cards(20, &[1, 1], 1, 5);
+        assert_eq!(
+            judged.after_the_last_round,
+            AfterARoundOfTheCards::EveryBlockStored
+        );
+        assert_eq!(judged.stored, (0..20).collect::<Vec<usize>>());
+        assert_eq!(judged.judged_times, vec![1; 20]);
+    }
+
+    /// 第 10z 批第三块：只有一张卡，它在第 5 块报了：没有卡能判了，这一趟报本次未跑，前 4 块在库里。
+    #[test]
+    fn with_one_card_only_a_block_it_gives_back_leaves_the_run_not_run() {
+        let judged = blocks_judged_on_fake_cards(20, &[1], 1, 5);
+        assert_eq!(
+            judged.after_the_last_round,
+            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
+        );
+        assert_eq!(judged.stored, vec![0, 1, 2, 3]);
+    }
 }
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index 542a947..7023980 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -46,7 +46,12 @@
 //!    第 ① 段由 CPU 判时单元校验和对拍几条线程一起派活，每条线程那一段各自包进接 Out of Memory 的那一层（第 10z 批）：有一段报了，
 //!    这一块不入库、不再领下一块，这一趟报本次未跑（`CheckingTally::stopped_because_the_gpu_memory_was_short`，之前判完的块在库里）。
 //!    第 ① 段在几张卡上判时，一张卡建判器或判一块时报了 Out of Memory，只停那一张：它手里那一块交回队里，别的卡照判
-//!    （第 10z 批，第 10y 批代码轮判决 K2-2 P2）；剩下的块别的卡判完了这一趟照常交回，判不完（卡都停了）才报本次未跑。
+//!    （第 10z 批，第 10y 批代码轮判决 K2-2 P2）；剩下的块别的卡判完了这一趟照常交回。别的卡都已收工、还有交回的块没判时，收尾那一步
+//!    再起一轮，只用没报过的卡（照这一刻能借的重新装状态、借草稿区）判剩下的块，判完的块照块序都入库（第 10z 批第三块，第 10z 批代码轮判决
+//!    K2-1 第 5 条：状态全装在第一张卡时也起第二张）；报过的卡之外没有卡能判才报本次未跑。卡线程那一圈与按块序写库、几轮怎么走在 harness 档
+//!    （`singlefs_harness::gpu_memory_ledger::judge_blocks_on_one_card`、`BlocksStoredInOrder`、`after_a_round_of_the_cards`），不要卡就测得到。
+//!    单元校验和对拍时一块的几段在同一个设备上并发，认一段是不是显存不够之前等同块的几段都收完（第 10z 批第三块，攻方：一段因设备丢了先失败，
+//!    同块另一段范围里接到的 Out of Memory 还没记进设备时不判红）。
 //!    没起过卡池的进程照旧不进账（[`GpuCardsSource::DevicesStartedForThisRun`]：这一趟自己在点名的卡上起设备），调用方是这几种：
 //!    崩溃放量分项驱动（`scripts/crash/crash-amplification-item.sh`）起的、只有经 [`run_from_environment`] 跑的用例而没有一条取卡池的测试二进制
 //!    （`tests/crash_amplification_fixed_script_through_rollback.rs`、`…_through_unmount.rs`、`crash_amplification_two_flows_share_their_prefix.rs`、
@@ -64,6 +69,12 @@ use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
 use singlefs_core::recovery::{recover, JournalPolicy};
 use singlefs_core::root_record::ROOT_MAGIC;
 #[cfg(feature = "gpu")]
+use singlefs_harness::gpu_memory_ledger::{
+    after_a_round_of_the_cards, judge_blocks_on_one_card, AfterARoundOfTheCards,
+    BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards, BlocksStoredInOrder,
+    CardLeftTheBlocks, StopOfTheRun,
+};
+#[cfg(feature = "gpu")]
 use singlefs_harness::gpu_memory_ledger::{
     decide_scratch_before_booking, gpu_not_run_line, mebibytes_of_buffers_held_at_once,
     most_states_whose_need_fits_the_capacity, reason_for_a_booking_without_room_now,
@@ -72,6 +83,7 @@ use singlefs_harness::gpu_memory_ledger::{
     ScratchBeforeBooking, ScratchBookingRefusalOutcome, ScratchStatesToBook,
     PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
 };
+use singlefs_harness::gpu_memory_ledger::{WorkEndingTogether, WorksEndingTogetherOnADevice};
 use singlefs_harness::hexadecimal::hexadecimal_text;
 use singlefs_harness::memory_pool::{
     root_identity_of_write, root_identity_written_by, CrashImage, MemoryPool, PublishedVersion,
@@ -1207,21 +1219,30 @@ struct JudgedBlock {
 }
 
 /// 判一块的一段（[`judge_one_block`]）：单元校验和交卡对拍的，这一段包进接 Out of Memory 的那一层（[`on_the_card_catching_out_of_memory`]，
-/// 每条核对线程各认各的），报了就交一段不作数、带着原因的（第 10z 批，第 10y 批代码轮判决 K1-3）。
+/// 每条核对线程各认各的），报了就交一段不作数、带着原因的（第 10z 批，第 10y 批代码轮判决 K1-3）；`ending` 是这一段在同一块几段里的报到，
+/// 认是不是显存不够之前等同一块的几段都收完（第 10z 批第三块，第 10z 批代码轮判决 K1-3 攻方：一段因设备丢了先失败，同块另一段范围里接到的
+/// Out of Memory 还没记进设备）。只在 CPU 上判的不用它（丢掉就是报到）。
 fn judge_one_piece_catching_out_of_memory_on_the_card(
     flow: &CrashFlow<'_>,
     plan: &CrashPlan,
     point_index: usize,
     local: Range<u64>,
     unit_checksums: &UnitChecksumsOn,
+    ending: WorkEndingTogether<'_>,
 ) -> JudgedBlock {
     match unit_checksums {
-        UnitChecksumsOn::CpuOnly => judge_one_block(flow, plan, point_index, local, unit_checksums),
+        UnitChecksumsOn::CpuOnly => {
+            drop(ending);
+            judge_one_block(flow, plan, point_index, local, unit_checksums)
+        }
         #[cfg(feature = "gpu")]
         UnitChecksumsOn::Gpu(card) => {
-            match on_the_card_catching_out_of_memory(card, "单元校验和对拍时", || {
-                judge_one_block(flow, plan, point_index, local.clone(), unit_checksums)
-            }) {
+            match on_the_card_catching_out_of_memory_ending_together(
+                card,
+                "单元校验和对拍时",
+                ending,
+                || judge_one_block(flow, plan, point_index, local.clone(), unit_checksums),
+            ) {
                 Ok(judged) => judged,
                 Err(reason) => JudgedBlock {
                     local,
@@ -1377,9 +1398,81 @@ pub fn check_blocks_where_until(
     stop_is_requested: &(dyn Fn() -> bool + Sync),
 ) -> Result<CheckingTally, VerdictStoreError> {
     let pending = pending_blocks(store, plan, version, is_mine)?;
+    let points: Vec<NamesOfACrashPoint> = plan
+        .points
+        .iter()
+        .map(|point| NamesOfACrashPoint {
+            name: point.name.clone(),
+            path_number: point.path_number.clone(),
+        })
+        .collect();
+    check_pending_blocks_in_pieces(
+        &pending,
+        &points,
+        checking_threads(),
+        stop_is_requested,
+        &|point_index, local, ending| {
+            judge_one_piece_catching_out_of_memory_on_the_card(
+                flow,
+                plan,
+                point_index,
+                local,
+                unit_checksums,
+                ending,
+            )
+        },
+        &mut |point_index, local, items_of_each_state, violating| {
+            let mut numbers = Vec::with_capacity(items_of_each_state.len());
+            for items in items_of_each_state {
+                numbers.push(store.register_verdict_vector(&encoded_verdict(items))?);
+            }
+            let verdict_block = VerdictBlock::new(numbers).expect("一块至少一个状态");
+            store.store_verdict_block(
+                &plan.points[point_index].block_key(local),
+                version,
+                &verdict_block,
+                violating,
+            )?;
+            Ok(())
+        },
+    )
+}
+
+/// [`check_pending_blocks_in_pieces`] 写一块的那一份：点在 `points` 里的次序、块内序号、逐状态红项、红状态的块内序号。
+type StoreACheckedBlock<'store> = dyn FnMut(
+        usize,
+        &Range<u64>,
+        &[Vec<String>],
+        &[StateOffsetInBlock],
+    ) -> Result<(), VerdictStoreError>
+    + 'store;
+
+/// 一个崩溃点的名字与路径编号（核对时打进度行、记红状态的正文用）。
+struct NamesOfACrashPoint {
+    name: String,
+    path_number: String,
+}
+
+/// [`check_blocks_where_until`] 的那一圈（不碰库与卡：判一段交 `judge_a_piece`，写一块交 `store_a_block`，单测用假的）：`pending` 里的块按次序，
+/// 每领一块之前先问 `stop_is_requested`；一块的序号切成 `threads` 那么多段、一段一个线程交 `judge_a_piece`（带这一段在同一块几段里的报到），
+/// 收齐按段序拼回一块、当场交 `store_a_block`（点在 `points` 里的次序、块内序号、逐状态红项、红状态的块内序号）。有一段报了显存不够，
+/// 这一块不写、不再领下一块，交回的账带着原因。
+///
+/// # Errors
+/// `store_a_block` 的错。
+///
+/// # Panics
+/// 判一段的线程 panic。
+fn check_pending_blocks_in_pieces(
+    pending: &[(usize, Range<u64>)],
+    points: &[NamesOfACrashPoint],
+    threads: usize,
+    stop_is_requested: &(dyn Fn() -> bool + Sync),
+    judge_a_piece: &(dyn Fn(usize, Range<u64>, WorkEndingTogether<'_>) -> JudgedBlock + Sync),
+    store_a_block: &mut StoreACheckedBlock<'_>,
+) -> Result<CheckingTally, VerdictStoreError> {
     // 块是库与 GPU 批的粒度，不改小；一块的序号切成线程数那么多段、一段一个线程（一块 65536 个状态、32 线程时一段 2048 个），
     // 判完按段序拼回一块、当场写库：中途被停，写进库的块下一次不再判。
-    let threads = checking_threads();
     let mut tally = CheckingTally::default();
     // 每一段判完打一行 `CRASH_AMPLIFICATION_BLOCK`；一段判得比进度间隔久时，等着收的这条线程隔一个间隔另打一行按挂钟的
     let started = std::time::Instant::now();
@@ -1389,7 +1482,7 @@ pub fn check_blocks_where_until(
         if stop_is_requested() {
             break;
         }
-        let point = &plan.points[*point_index];
+        let point = &points[*point_index];
         let piece_length = (local.end - local.start)
             .div_ceil(u64::try_from(threads).expect("段数装得进 u64"))
             .max(1);
@@ -1401,19 +1494,16 @@ pub fn check_blocks_where_until(
             start = end;
         }
         let piece_count = pieces.len();
+        // 每一段的报到在起线程之前都领出来：先收完的那一段等的是这一块全部的段
+        let ending_together = WorksEndingTogetherOnADevice::new();
+        let endings: Vec<WorkEndingTogether<'_>> =
+            pieces.iter().map(|_| ending_together.join()).collect();
         let mut judged: Vec<JudgedBlock> = std::thread::scope(|scope| {
             let (judged_sender, judged_receiver) = std::sync::mpsc::channel::<JudgedBlock>();
-            for range in &pieces {
-                let range = range.clone();
+            for (range, ending) in pieces.iter().cloned().zip(endings) {
                 let judged_sender = judged_sender.clone();
                 scope.spawn(move || {
-                    let piece = judge_one_piece_catching_out_of_memory_on_the_card(
-                        flow,
-                        plan,
-                        *point_index,
-                        range,
-                        unit_checksums,
-                    );
+                    let piece = judge_a_piece(*point_index, range, ending);
                     // 发不出去说明收的这条线程已经走了（它 panic 了），这一段的结果没人要
                     let _ = judged_sender.send(piece);
                 });
@@ -1466,7 +1556,6 @@ pub fn check_blocks_where_until(
             }
         }
         assert_eq!(next_expected, local.end, "块 {pending_index} 的段没有拼满");
-        let mut numbers = Vec::with_capacity(items_of_each_state.len());
         let mut violating = Vec::new();
         for (offset, items) in items_of_each_state.iter().enumerate() {
             if !items.is_empty() {
@@ -1475,10 +1564,8 @@ pub fn check_blocks_where_until(
                     u64::try_from(offset).expect("块内序号装得进 u64"),
                 ));
             }
-            numbers.push(store.register_verdict_vector(&encoded_verdict(items))?);
         }
-        let verdict_block = VerdictBlock::new(numbers).expect("一块至少一个状态");
-        store.store_verdict_block(&point.block_key(local), version, &verdict_block, &violating)?;
+        store_a_block(*point_index, local, &items_of_each_state, &violating)?;
         tally.blocks_checked += 1;
         tally.states_checked += local.end - local.start;
     }
@@ -1938,24 +2025,26 @@ enum CardMessage {
     Finished(GpuCardOutcome),
     /// 这张卡判不了领到的块里的某个状态（表越过了内核的容量）：整趟停下、报错，不退回 CPU 判器。
     Refused(String),
-    /// 这张卡建判器或判一块时 wgpu 报了 Out of Memory（文件头第 5 条）：整趟停下，这一块不入库，这一趟报本次未跑；带那一句为什么。
-    OutOfMemory(String),
+    /// 这张卡（`index` 是 `nvidia-smi` 给它的序号）建判器或判一块时 wgpu 报了 Out of Memory（文件头第 5 条）：只停这一张，判一块时报的那一块已经交回队里；
+    /// 带那一句为什么。
+    OutOfMemory {
+        index: u32,
+        reason: String,
+    },
 }
 
-/// 写库那条线程收完卡上的线程发来的话之后手里的：按块序写进库几块、收工的卡各自的账（按收到的次序）、写库的错、卡拒判的理由、
-/// 显存不够的那一句。
+/// 写库那条线程收完一轮卡上的线程发来的话之后手里的：收工的卡各自的账（按收到的次序）、卡拒判的理由、报了 Out of Memory 的卡（序号与那一句，
+/// 按收到的次序）。判完的块按块序写进库几块、写库的错在几轮共用的那一份里（[`BlocksStoredInOrder`]）。
 #[cfg(feature = "gpu")]
 struct CardsMessagesReceived {
-    blocks_stored: usize,
     finished_cards: Vec<GpuCardOutcome>,
-    storing_error: Option<String>,
     refusal: Option<String>,
-    out_of_memory: Option<String>,
+    cards_out_of_memory: Vec<(u32, String)>,
 }
 
-/// 写库那条线程收卡上的线程发来的话，到发送端全丢掉为止：判完的块按块序交 `store_block` 写库（后面的块先到就等前面那一块，
-/// 到了一起写；写库失败之后到的块不再写），收工的卡记下它的账，拒判的留第一条理由；上一次收到之后 `clock` 走过整个进度间隔还没有新的话，
-/// 就交 `write_progress_line` 一行 `CRASH_AMPLIFICATION_BLOCK by=wall_clock judge=gpu …`（C611：卡上判一块、编内核比进度间隔久时
+/// 写库那条线程收卡上的线程发来的话，到发送端全丢掉为止：判完的块交 `stored_in_order` 按块序写库（`store_block` 写一块；后面的块先到就等前面那一块，
+/// 到了一起写；写库失败之后到的块不再写），收工的卡记下它的账，拒判的留第一条理由，报了 Out of Memory 的卡都记下；上一次收到之后 `clock` 走过整个进度间隔
+/// 还没有新的话，就交 `write_progress_line` 一行 `CRASH_AMPLIFICATION_BLOCK by=wall_clock judge=gpu …`（C611：卡上判一块、编内核比进度间隔久时
 /// 照样隔一个间隔报一行，[`gpu_judge_wall_clock_line`]）。`pending_blocks` 是这一趟要判的块数，只进那一行。
 #[cfg(feature = "gpu")]
 fn receive_the_cards_messages_and_store_the_blocks_in_order(
@@ -1963,18 +2052,15 @@ fn receive_the_cards_messages_and_store_the_blocks_in_order(
     wall_clock_lines: &mut WallClockProgressLines,
     clock: &dyn Fn() -> std::time::Duration,
     pending_blocks: usize,
+    stored_in_order: &mut BlocksStoredInOrder<GpuJudgedBlock>,
     store_block: &mut dyn FnMut(GpuJudgedBlock) -> Result<(), String>,
     write_progress_line: &mut dyn FnMut(String),
 ) -> CardsMessagesReceived {
     let mut received = CardsMessagesReceived {
-        blocks_stored: 0,
         finished_cards: Vec::new(),
-        storing_error: None,
         refusal: None,
-        out_of_memory: None,
+        cards_out_of_memory: Vec::new(),
     };
-    // 写库：按块序。后面的块先判完就在这里等，前面那一块到了一起写
-    let mut waiting: BTreeMap<usize, GpuJudgedBlock> = BTreeMap::new();
     receive_until_the_senders_are_gone_with_wall_clock_lines(
         receiver,
         wall_clock_lines,
@@ -1982,9 +2068,9 @@ fn receive_the_cards_messages_and_store_the_blocks_in_order(
         &mut |event| match event {
             ReceivedOrWallClockLineDue::WallClockLineDue { elapsed } => {
                 write_progress_line(gpu_judge_wall_clock_line(
-                    received.blocks_stored,
+                    stored_in_order.blocks_stored(),
                     pending_blocks,
-                    waiting.len(),
+                    stored_in_order.blocks_waiting(),
                     received.finished_cards.len(),
                     elapsed,
                 ));
@@ -1995,20 +2081,11 @@ fn receive_the_cards_messages_and_store_the_blocks_in_order(
             ReceivedOrWallClockLineDue::Received(CardMessage::Refused(reason)) => {
                 received.refusal.get_or_insert(reason);
             }
-            ReceivedOrWallClockLineDue::Received(CardMessage::OutOfMemory(reason)) => {
-                received.out_of_memory.get_or_insert(reason);
+            ReceivedOrWallClockLineDue::Received(CardMessage::OutOfMemory { index, reason }) => {
+                received.cards_out_of_memory.push((index, reason));
             }
             ReceivedOrWallClockLineDue::Received(CardMessage::Block(arrived_block)) => {
-                waiting.insert(arrived_block.pending_index, arrived_block);
-                while let Some(block) = waiting.remove(&received.blocks_stored) {
-                    if received.storing_error.is_some() {
-                        continue;
-                    }
-                    match store_block(block) {
-                        Ok(()) => received.blocks_stored += 1,
-                        Err(error) => received.storing_error = Some(error),
-                    }
-                }
+                stored_in_order.arrive(arrived_block.pending_index, arrived_block, store_block);
             }
         },
     );
@@ -2042,69 +2119,6 @@ impl CardsAtWork<'_> {
     }
 }
 
-/// 几张卡一起判的那一队块（要判的块在 `pending` 里的次序）：从队头领；一张卡报了 Out of Memory 把手里那一块交回（第 10z 批，
-/// 第 10y 批代码轮判决 K2-2 P2：一张卡 Out of Memory 不停整趟），交回的先给下一张来领的卡。
-#[cfg(feature = "gpu")]
-struct BlocksForTheCards {
-    pending_blocks: usize,
-    next_pending: std::sync::atomic::AtomicUsize,
-    given_back: std::sync::Mutex<Vec<usize>>,
-}
-
-#[cfg(feature = "gpu")]
-impl BlocksForTheCards {
-    fn new(pending_blocks: usize) -> Self {
-        Self {
-            pending_blocks,
-            next_pending: std::sync::atomic::AtomicUsize::new(0),
-            given_back: std::sync::Mutex::new(Vec::new()),
-        }
-    }
-
-    /// 领一块：先领交回的（序号小的先），没有就从队头领；都领完了是 `None`。
-    fn take_a_block(&self) -> Option<usize> {
-        // 持锁只为取一个序号，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
-        let mut given_back = self
-            .given_back
-            .lock()
-            .unwrap_or_else(std::sync::PoisonError::into_inner);
-        if let Some(smallest) = given_back.iter().copied().min() {
-            given_back.retain(|pending_index| *pending_index != smallest);
-            return Some(smallest);
-        }
-        drop(given_back);
-        let pending_index = self
-            .next_pending
-            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
-        (pending_index < self.pending_blocks).then_some(pending_index)
-    }
-
-    /// 领了没判完的一块交回队里（报了 Out of Memory 的卡交）。
-    fn give_back(&self, pending_index: usize) {
-        self.given_back
-            .lock()
-            .unwrap_or_else(std::sync::PoisonError::into_inner)
-            .push(pending_index);
-    }
-
-    /// 还有没领走的块没有（队里的或交回的）。
-    fn any_left(&self) -> bool {
-        self.next_pending.load(std::sync::atomic::Ordering::SeqCst) < self.pending_blocks
-            || !self
-                .given_back
-                .lock()
-                .unwrap_or_else(std::sync::PoisonError::into_inner)
-                .is_empty()
-    }
-
-    /// 从队头领走了几块（不算交回的；序号连着、从 0 起）。
-    fn taken_from_the_head(&self) -> usize {
-        self.next_pending
-            .load(std::sync::atomic::Ordering::SeqCst)
-            .min(self.pending_blocks)
-    }
-}
-
 /// 卡上一段活做完之后的结局（[`decide_the_work_on_the_card`]）。
 #[cfg(feature = "gpu")]
 enum WorkOnTheCard<Outcome> {
@@ -2149,6 +2163,21 @@ fn on_the_card_catching_out_of_memory<Outcome>(
     card: &crate::gpu_unit_checks::GpuCard,
     what_ran_out: &str,
     work: impl FnOnce() -> Outcome,
+) -> Result<Outcome, String> {
+    let alone = WorksEndingTogetherOnADevice::new();
+    on_the_card_catching_out_of_memory_ending_together(card, what_ran_out, alone.join(), work)
+}
+
+/// 同 [`on_the_card_catching_out_of_memory`]，这段活与同一个设备上别的几段一起开（`ending` 是它的报到，单元校验和对拍时一块切成的几段）：
+/// 这条线程上的错误范围接到的 Out of Memory 记进设备之后报到，等那几段都报到了再认设备上记的
+/// （`singlefs_harness::gpu_memory_ledger::DeviceConditionOfACard::end_the_work_and_explain_its_failure`，第 10z 批第三块：一段因设备丢了先失败、
+/// 同块另一段范围里接到的还没记进设备时，先认就认不出、判红）。
+#[cfg(feature = "gpu")]
+fn on_the_card_catching_out_of_memory_ending_together<Outcome>(
+    card: &crate::gpu_unit_checks::GpuCard,
+    what_ran_out: &str,
+    ending: WorkEndingTogether<'_>,
+    work: impl FnOnce() -> Outcome,
 ) -> Result<Outcome, String> {
     let work_on_the_card = singlefs_harness::gpu_memory_ledger::WorkOnTheCardOfThisThread::begin();
     let out_of_memory_reports_before = card.device_condition().out_of_memory_reports();
@@ -2160,16 +2189,14 @@ fn on_the_card_catching_out_of_memory<Outcome>(
         pollster::block_on(out_of_memory_scope.pop()).map(|error| error.to_string());
     let out_of_memory_on_the_device = card
         .device_condition()
-        .out_of_memory_explaining_a_failure_of_the_work(
-            work_on_the_card.identifier(),
+        .end_the_work_and_explain_its_failure(
+            &work_on_the_card,
             out_of_memory_reports_before,
+            out_of_memory_in_the_scope
+                .as_ref()
+                .map(|caught| format!("序号 {} 的卡{what_ran_out}报了 {caught}", card.index)),
+            ending,
         );
-    if let Some(caught) = &out_of_memory_in_the_scope {
-        card.device_condition().record_an_out_of_memory(format!(
-            "序号 {} 的卡{what_ran_out}报了 {caught}",
-            card.index
-        ));
-    }
     match decide_the_work_on_the_card(
         outcome,
         out_of_memory_in_the_scope,
@@ -2213,7 +2240,10 @@ fn judge_on_one_card_and_start_the_next<'scope, 'work>(
             Ok(judge) => judge,
             Err(reason) => {
                 // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
-                let _ = sender.send(CardMessage::OutOfMemory(reason));
+                let _ = sender.send(CardMessage::OutOfMemory {
+                    index: card.index,
+                    reason,
+                });
                 start_the_next_card_with_states(scope, work, position, &sender);
                 return;
             }
@@ -2247,67 +2277,83 @@ fn judge_on_one_card_and_start_the_next<'scope, 'work>(
         scratch_lease_on_the_books_when_the_judge_was_built,
         ..GpuCardOutcome::default()
     };
-    while !work.must_stop() {
-        let Some(pending_index) = work.blocks.take_a_block() else {
-            break;
-        };
-        let (point_index, local) = &work.pending[pending_index];
-        let point = &work.plan.points[*point_index];
-        let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
-        let words = match on_the_card_catching_out_of_memory(card, "判一块时", || {
-            judge.judge_states(ordinals.clone())
-        }) {
-            Ok(words) => words,
-            Err(reason) => {
+    let card_left = judge_blocks_on_one_card(
+        &work.blocks,
+        &|| work.must_stop(),
+        &mut |pending_index| {
+            let (point_index, local) = &work.pending[pending_index];
+            let point = &work.plan.points[*point_index];
+            let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
+            let words = match on_the_card_catching_out_of_memory(card, "判一块时", || {
+                judge.judge_states(ordinals.clone())
+            }) {
+                Ok(words) => words,
                 // 只停这一张卡：这一块交回给别的卡（第 10z 批）
-                work.blocks.give_back(pending_index);
-                let _ = sender.send(CardMessage::OutOfMemory(reason));
-                return;
+                Err(reason) => return BlockJudgedOnACard::OutOfMemory { reason },
+            };
+            let decoding_started = Instant::now();
+            if let Some(offset) = first_state_the_gpu_could_not_judge(&words) {
+                let state_words = &words[offset];
+                return BlockJudgedOnACard::Refused {
+                    reason: format!(
+                        "卡 {} 判不了点 {} 块内第 {} 个状态（恢复内核第 1 字 {}、checker 内核第 {} 字 {}：表越过了内核的容量）。\
+                         配置开了 GPU，不退回 CPU 判器：照这两个字查是哪张表满了、改内核的容量再跑",
+                        card.index,
+                        point.name,
+                        local.start + u64::try_from(offset).expect("块内序号"),
+                        state_words[1],
+                        crate::crash_judge_gpu::POOL_CHECKER_UNDECIDABLE_WORD,
+                        state_words[crate::crash_judge_gpu::POOL_CHECKER_UNDECIDABLE_WORD]
+                    ),
+                };
             }
-        };
-        let decoding_started = Instant::now();
-        if let Some(offset) = first_state_the_gpu_could_not_judge(&words) {
-            let state_words = &words[offset];
-            work.stop.store(true, Ordering::SeqCst);
-            // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
-            let _ = sender.send(CardMessage::Refused(format!(
-                "卡 {} 判不了点 {} 块内第 {} 个状态（恢复内核第 1 字 {}、checker 内核第 {} 字 {}：表越过了内核的容量）。\
-                 配置开了 GPU，不退回 CPU 判器：照这两个字查是哪张表满了、改内核的容量再跑",
-                card.index,
-                point.name,
-                local.start + u64::try_from(offset).expect("块内序号"),
-                state_words[1],
-                crate::crash_judge_gpu::POOL_CHECKER_UNDECIDABLE_WORD,
-                state_words[crate::crash_judge_gpu::POOL_CHECKER_UNDECIDABLE_WORD]
-            )));
-            return;
-        }
-        let items_of_each_state: Vec<Vec<String>> = words
-            .iter()
-            .map(|state_words| gpu_verdict_items(state_words).expect("判不了的状态上面已经拒过"))
-            .collect();
-        outcome.seconds_decoding += decoding_started.elapsed().as_secs_f64();
-        eprintln!(
-            "CRASH_AMPLIFICATION_BLOCK point={} block_start={} states={} red={} judge=gpu card={}",
-            point.name,
-            local.start,
-            local.end - local.start,
-            items_of_each_state
+            let items_of_each_state: Vec<Vec<String>> = words
                 .iter()
-                .filter(|items| !items.is_empty())
-                .count(),
-            card.index
-        );
-        outcome.blocks_judged += 1;
-        outcome.states_judged += local.end - local.start;
-        if sender
-            .send(CardMessage::Block(GpuJudgedBlock {
+                .map(|state_words| {
+                    gpu_verdict_items(state_words).expect("判不了的状态上面已经拒过")
+                })
+                .collect();
+            outcome.seconds_decoding += decoding_started.elapsed().as_secs_f64();
+            eprintln!(
+                "CRASH_AMPLIFICATION_BLOCK point={} block_start={} states={} red={} judge=gpu card={}",
+                point.name,
+                local.start,
+                local.end - local.start,
+                items_of_each_state
+                    .iter()
+                    .filter(|items| !items.is_empty())
+                    .count(),
+                card.index
+            );
+            outcome.blocks_judged += 1;
+            outcome.states_judged += local.end - local.start;
+            BlockJudgedOnACard::Judged(items_of_each_state)
+        },
+        &mut |pending_index, items_of_each_state| match sender.send(CardMessage::Block(
+            GpuJudgedBlock {
                 pending_index,
                 items_of_each_state,
-            }))
-            .is_err()
-        {
-            break;
+            },
+        )) {
+            Ok(()) => BlockHandedToTheStore::Handed,
+            Err(_) => BlockHandedToTheStore::TheStoreIsGone,
+        },
+    );
+    match card_left {
+        CardLeftTheBlocks::NothingLeftOrStopped | CardLeftTheBlocks::TheStoreIsGone => {}
+        CardLeftTheBlocks::OutOfMemory { reason } => {
+            // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
+            let _ = sender.send(CardMessage::OutOfMemory {
+                index: card.index,
+                reason,
+            });
+            return;
+        }
+        CardLeftTheBlocks::Refused { reason } => {
+            work.stop.store(true, Ordering::SeqCst);
+            // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
+            let _ = sender.send(CardMessage::Refused(reason));
+            return;
         }
     }
     outcome.judge_seconds = judging_started.elapsed().as_secs_f64();
@@ -2413,30 +2459,6 @@ fn start_the_next_card_with_states<'scope, 'work>(
     }
 }
 
-/// 几张卡都收工之后，有卡报过 Out of Memory 时这一趟怎么交回（[`judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from`]）。
-#[cfg(feature = "gpu")]
-#[derive(Clone, Debug, PartialEq, Eq)]
-enum CardsOutcomeAfterAnOutOfMemory {
-    /// 别的卡把剩下的块都判完写进了库：这一趟照常交回（报 Out of Memory 的那几张只是没判）。
-    EveryBlockJudged,
-    /// 还有块没判（卡都停了，或被叫停）：这一趟报本次未跑，判过的块在库里。
-    NotRunTheGpuMemoryWasShort { reason: String },
-}
-
-/// 几张卡收工之后：写进库 `blocks_stored` 块（共 `pending_blocks` 块），`out_of_memory` 是第一张报 Out of Memory 的卡那一句（没有是 `None`）。
-#[cfg(feature = "gpu")]
-fn cards_outcome_after_an_out_of_memory(
-    blocks_stored: usize,
-    pending_blocks: usize,
-    out_of_memory: Option<String>,
-) -> Option<CardsOutcomeAfterAnOutOfMemory> {
-    let reason = out_of_memory?;
-    if blocks_stored >= pending_blocks {
-        return Some(CardsOutcomeAfterAnOutOfMemory::EveryBlockJudged);
-    }
-    Some(CardsOutcomeAfterAnOutOfMemory::NotRunTheGpuMemoryWasShort { reason })
-}
-
 /// 一张卡照额度 `memory_budget_mebibytes`（MiB）一次派活带得了几个状态（表上卡之后剩下的显存装草稿区）：表比存储缓冲绑定上限
 /// `max_storage_buffer_binding_size` 大、或额度装不下表与驱动那一份，是 0。
 #[cfg(feature = "gpu")]
@@ -2893,175 +2915,241 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
     }
     let state_count = tables.state_count;
     let tables = crate::crash_judge_dispatch::PackedJudgeTables::of(&tables);
-    let pending_states: u64 = pending
-        .iter()
-        .map(|(_, local)| local.end - local.start)
-        .sum();
-    let run_cards = card_capacities_of_this_run(cards_taken_from, cards, &tables)?;
-    let capacities: Vec<u64> = run_cards
-        .iter()
-        .map(|card| card.states_in_flight_capacity)
-        .collect();
-    let capacities_when_the_cards_are_empty: Vec<u64> = run_cards
-        .iter()
-        .map(|card| card.states_in_flight_capacity_when_the_card_is_empty)
-        .collect();
-    // 借据活到这个函数收尾：几张卡上的判器在下面的线程里判完就丢，丢完才还
-    let scratch = match scratch_step_before_booking(
-        decide_scratch_before_booking(&capacities, &capacities_when_the_cards_are_empty),
-        &run_cards,
-        crate::crash_judge_dispatch::mebibytes_a_judge_needs(&tables, 1),
-    ) {
-        ScratchStepBeforeBooking::BookOnTheCards => {
-            match scratch_of_each_card_booked_by_where_the_cards_came_from(
-                cards_taken_from,
-                cards,
-                &run_cards,
-                &tables,
-                state_count,
-                pending_states,
-                &capacities,
-            )? {
-                ScratchOfEachCard::Ready(scratch) => Some(scratch),
-                ScratchOfEachCard::NotRunTheGpuMemoryWasShort { not_run_line } => {
-                    return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
-                        tally,
-                        reason: not_run_line,
-                    });
-                }
-            }
-        }
-        ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow { not_run_line } => {
-            println!("{not_run_line}");
-            return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
-                tally,
-                reason: not_run_line,
-            });
-        }
-        ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty => None,
-    };
-    let states_placed_on_each_card_of_this_run = scratch.as_ref().map_or_else(
-        || vec![0; run_cards.len()],
-        |scratch| scratch.states_placed_on_each_card_of_this_run.clone(),
-    );
-    for ((card, capacity), placed) in run_cards
-        .iter()
-        .zip(&capacities)
-        .zip(&states_placed_on_each_card_of_this_run)
-    {
-        eprintln!(
-            "CRASH_AMPLIFICATION_GPU_PLACEMENT card={:?} index={} memory_budget_mebibytes={} states_in_flight_capacity={capacity} states_placed={placed} pending_states={pending_states} tables_mebibytes={}",
-            card.name,
-            card.index,
-            card.memory_budget_mebibytes,
-            tables.mebibytes()
-        );
-    }
-    let Some(scratch) = scratch.filter(|scratch| !scratch.cards_for_the_judges.is_empty()) else {
-        return Err(format!(
-            "没有一张卡装得下这条流的表（{} MiB）与一个状态的草稿区：看配置里每张卡的显存额度与卡上此刻的空闲显存",
-            tables.mebibytes()
-        ));
-    };
-    let work = CardsAtWork {
-        plan,
-        pending: &pending,
-        tables: &tables,
-        cards: &scratch.cards_for_the_judges,
-        states_placed: &scratch.states_placed,
-        scratch_booked_mebibytes: &scratch.scratch_booked_mebibytes,
-        scratch_lease_identifiers: &scratch.scratch_lease_identifiers,
-        blocks: BlocksForTheCards::new(pending.len()),
-        stop: std::sync::atomic::AtomicBool::new(false),
-        stop_is_requested,
-    };
+    // 几轮卡共用：上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写
+    let mut stored_in_order: BlocksStoredInOrder<GpuJudgedBlock> = BlocksStoredInOrder::new();
+    let mut blocks_of_the_round: Vec<usize> = (0..pending.len()).collect();
+    let mut cards_short_of_memory: BTreeSet<u32> = BTreeSet::new();
+    let mut first_out_of_memory: Option<String> = None;
+    // 这一趟本机每张卡的优先级（头一轮那一份，后面几轮只少不多）：收尾时每张卡的账照它排
+    let mut card_indices_by_priority: Vec<u32> = Vec::new();
     let started = Instant::now();
-    let received = std::thread::scope(|scope| {
-        let (sender, receiver) = std::sync::mpsc::channel::<CardMessage>();
-        let work = &work;
-        scope.spawn(move || {
-            judge_on_one_card_and_start_the_next(scope, work, 0, sender);
-        });
-        // 卡上判一块、编内核都可能比进度间隔久：等着收的这条线程隔一个间隔打一行按挂钟的
-        let clock = || started.elapsed();
-        let mut wall_clock_lines = WallClockProgressLines::every_progress_line_interval(clock());
-        let received = receive_the_cards_messages_and_store_the_blocks_in_order(
-            &receiver,
-            &mut wall_clock_lines,
-            &clock,
-            pending.len(),
-            &mut |block| {
-                let storing_started = Instant::now();
-                let (point_index, local) = &pending[block.pending_index];
-                let stored = store_judged_block(
-                    store,
-                    &plan.points[*point_index],
-                    local,
-                    version,
-                    &block.items_of_each_state,
-                    &mut tally,
-                );
-                outcome.seconds_storing += storing_started.elapsed().as_secs_f64();
-                match stored {
-                    Ok(()) => {
-                        outcome.blocks_judged += 1;
-                        outcome.states_judged += local.end - local.start;
-                        Ok(())
-                    }
-                    Err(error) => {
-                        work.stop.store(true, std::sync::atomic::Ordering::SeqCst);
-                        Err(format!("写判定块失败：{error:?}"))
+    // 每再起一轮至少少一张卡（报过 Out of Memory 的不再用），轮数不超过卡数
+    loop {
+        let pending_states: u64 = blocks_of_the_round
+            .iter()
+            .map(|pending_index| {
+                let (_, local) = &pending[*pending_index];
+                local.end - local.start
+            })
+            .sum();
+        let cards_of_the_round: Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>> = cards
+            .iter()
+            .filter(|card| !cards_short_of_memory.contains(&card.index))
+            .cloned()
+            .collect();
+        let run_cards: Vec<CardCapacityOfThisRun> =
+            card_capacities_of_this_run(cards_taken_from, &cards_of_the_round, &tables)?
+                .into_iter()
+                .filter(|card| !cards_short_of_memory.contains(&card.index))
+                .collect();
+        if card_indices_by_priority.is_empty() {
+            card_indices_by_priority = run_cards.iter().map(|card| card.index).collect();
+        }
+        let capacities: Vec<u64> = run_cards
+            .iter()
+            .map(|card| card.states_in_flight_capacity)
+            .collect();
+        let capacities_when_the_cards_are_empty: Vec<u64> = run_cards
+            .iter()
+            .map(|card| card.states_in_flight_capacity_when_the_card_is_empty)
+            .collect();
+        // 借据活到这一轮收尾：几张卡上的判器在下面的线程里判完就丢，丢完才还
+        let scratch = match scratch_step_before_booking(
+            decide_scratch_before_booking(&capacities, &capacities_when_the_cards_are_empty),
+            &run_cards,
+            crate::crash_judge_dispatch::mebibytes_a_judge_needs(&tables, 1),
+        ) {
+            ScratchStepBeforeBooking::BookOnTheCards => {
+                match scratch_of_each_card_booked_by_where_the_cards_came_from(
+                    cards_taken_from,
+                    &cards_of_the_round,
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
                     }
                 }
-            },
-            &mut |line| eprintln!("{line}"),
+            }
+            ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow { not_run_line } => {
+                println!("{not_run_line}");
+                return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
+                    tally,
+                    reason: not_run_line,
+                });
+            }
+            ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty => None,
+        };
+        let states_placed_on_each_card_of_this_run = scratch.as_ref().map_or_else(
+            || vec![0; run_cards.len()],
+            |scratch| scratch.states_placed_on_each_card_of_this_run.clone(),
         );
-        if received.storing_error.is_none()
-            && received.refusal.is_none()
-            && received.out_of_memory.is_none()
+        for ((card, capacity), placed) in run_cards
+            .iter()
+            .zip(&capacities)
+            .zip(&states_placed_on_each_card_of_this_run)
         {
-            // 领走的块（序号连着、从 0 起）每一块都写进了库；没被叫停时领走的就是全部
-            let taken = work.blocks.taken_from_the_head();
-            assert_eq!(received.blocks_stored, taken, "领走的块每一块都写进了库");
-            assert!(
-                received.blocks_stored == pending.len() || stop_is_requested(),
-                "没人叫停，却有 {} 块没判",
-                pending.len() - received.blocks_stored
-            );
-        }
-        received
-    });
-    outcome.cards = received.finished_cards;
-    if let Some(error) = received.storing_error {
-        return Err(error);
-    }
-    if let Some(reason) = received.refusal {
-        return Err(reason);
-    }
-    match cards_outcome_after_an_out_of_memory(
-        received.blocks_stored,
-        pending.len(),
-        received.out_of_memory,
-    ) {
-        None => {}
-        Some(CardsOutcomeAfterAnOutOfMemory::EveryBlockJudged) => {
             eprintln!(
-                "CRASH_AMPLIFICATION_GPU_OUT_OF_MEMORY_ON_A_CARD 有卡报了 Out of Memory、只停那一张，别的卡判完了全部 {} 块",
-                pending.len()
+                "CRASH_AMPLIFICATION_GPU_PLACEMENT card={:?} index={} memory_budget_mebibytes={} states_in_flight_capacity={capacity} states_placed={placed} pending_states={pending_states} tables_mebibytes={}",
+                card.name,
+                card.index,
+                card.memory_budget_mebibytes,
+                tables.mebibytes()
             );
         }
-        Some(CardsOutcomeAfterAnOutOfMemory::NotRunTheGpuMemoryWasShort { reason }) => {
-            println!(
-                "{}",
-                gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
+        let Some(scratch) = scratch.filter(|scratch| !scratch.cards_for_the_judges.is_empty())
+        else {
+            if let Some(reason) = first_out_of_memory {
+                // 报过 Out of Memory 的卡之外没有一张装得下：没有卡能判剩下的块了
+                println!(
+                    "{}",
+                    gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
+                );
+                return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason });
+            }
+            return Err(format!(
+                "没有一张卡装得下这条流的表（{} MiB）与一个状态的草稿区：看配置里每张卡的显存额度与卡上此刻的空闲显存",
+                tables.mebibytes()
+            ));
+        };
+        let work = CardsAtWork {
+            plan,
+            pending: &pending,
+            tables: &tables,
+            cards: &scratch.cards_for_the_judges,
+            states_placed: &scratch.states_placed,
+            scratch_booked_mebibytes: &scratch.scratch_booked_mebibytes,
+            scratch_lease_identifiers: &scratch.scratch_lease_identifiers,
+            blocks: BlocksForTheCards::new(blocks_of_the_round.iter().copied()),
+            stop: std::sync::atomic::AtomicBool::new(false),
+            stop_is_requested,
+        };
+        let received = std::thread::scope(|scope| {
+            let (sender, receiver) = std::sync::mpsc::channel::<CardMessage>();
+            let work = &work;
+            scope.spawn(move || {
+                judge_on_one_card_and_start_the_next(scope, work, 0, sender);
+            });
+            // 卡上判一块、编内核都可能比进度间隔久：等着收的这条线程隔一个间隔打一行按挂钟的
+            let clock = || started.elapsed();
+            let mut wall_clock_lines =
+                WallClockProgressLines::every_progress_line_interval(clock());
+            let received = receive_the_cards_messages_and_store_the_blocks_in_order(
+                &receiver,
+                &mut wall_clock_lines,
+                &clock,
+                pending.len(),
+                &mut stored_in_order,
+                &mut |block| {
+                    let storing_started = Instant::now();
+                    let (point_index, local) = &pending[block.pending_index];
+                    let stored = store_judged_block(
+                        store,
+                        &plan.points[*point_index],
+                        local,
+                        version,
+                        &block.items_of_each_state,
+                        &mut tally,
+                    );
+                    outcome.seconds_storing += storing_started.elapsed().as_secs_f64();
+                    match stored {
+                        Ok(()) => {
+                            outcome.blocks_judged += 1;
+                            outcome.states_judged += local.end - local.start;
+                            Ok(())
+                        }
+                        Err(error) => {
+                            work.stop.store(true, std::sync::atomic::Ordering::SeqCst);
+                            Err(format!("写判定块失败：{error:?}"))
+                        }
+                    }
+                },
+                &mut |line| eprintln!("{line}"),
             );
-            return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason });
+            if stored_in_order.storing_error().is_none()
+                && received.refusal.is_none()
+                && received.cards_out_of_memory.is_empty()
+            {
+                // 没有卡报 Out of Memory：领走的块（序号连着、从这一轮最小的那一块起）每一块都写进了库；没被叫停时领走的就是全部
+                assert_eq!(
+                    stored_in_order.blocks_waiting(),
+                    0,
+                    "领走的块每一块都写进了库"
+                );
+                assert!(
+                    !work.blocks.any_left() || stop_is_requested(),
+                    "没人叫停，却有 {} 块没判",
+                    work.blocks.blocks_not_taken().len()
+                );
+            }
+            received
+        });
+        outcome.cards.extend(received.finished_cards);
+        if let Some(error) = stored_in_order.storing_error() {
+            return Err(error.to_string());
+        }
+        if let Some(reason) = received.refusal {
+            return Err(reason);
+        }
+        for (index, reason) in &received.cards_out_of_memory {
+            cards_short_of_memory.insert(*index);
+            first_out_of_memory.get_or_insert_with(|| reason.clone());
+        }
+        blocks_of_the_round = work.blocks.blocks_not_taken();
+        let cards_not_short_of_memory = run_cards
+            .iter()
+            .filter(|card| !cards_short_of_memory.contains(&card.index))
+            .count();
+        let stop = if stop_is_requested() {
+            StopOfTheRun::Requested
+        } else {
+            StopOfTheRun::NotRequested
+        };
+        match after_a_round_of_the_cards(
+            pending.len() - stored_in_order.blocks_stored(),
+            received.cards_out_of_memory.len(),
+            stop,
+            cards_not_short_of_memory,
+        ) {
+            AfterARoundOfTheCards::EveryBlockStored => {
+                if first_out_of_memory.is_some() {
+                    eprintln!(
+                        "CRASH_AMPLIFICATION_GPU_OUT_OF_MEMORY_ON_A_CARD 有卡报了 Out of Memory、只停那一张，别的卡判完了全部 {} 块",
+                        pending.len()
+                    );
+                }
+                break;
+            }
+            AfterARoundOfTheCards::StoppedOnRequest => break,
+            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory => {
+                eprintln!(
+                    "CRASH_AMPLIFICATION_GPU_ANOTHER_ROUND 报了 Out of Memory 的卡 {cards_short_of_memory:?} 收工之后还有 {} 块没写进库，收尾再起一轮、用别的卡判剩下的 {} 块",
+                    pending.len() - stored_in_order.blocks_stored(),
+                    blocks_of_the_round.len()
+                );
+            }
+            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort => {
+                let reason = first_out_of_memory
+                    .expect("这一轮有卡报了 Out of Memory 才会走到本次未跑这一支");
+                println!(
+                    "{}",
+                    gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
+                );
+                return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason });
+            }
         }
     }
     outcome.cards.sort_by_key(|card| {
-        run_cards
+        card_indices_by_priority
             .iter()
-            .position(|candidate| candidate.index == card.index)
+            .position(|index| *index == card.index)
     });
     outcome.cards_used = outcome.cards.len();
     outcome.compile_seconds = outcome.cards.iter().map(|card| card.build_seconds).sum();
@@ -3488,43 +3576,62 @@ mod tests {
         );
     }
 
-    /// 第 10z 批（第 10y 批代码轮判决 K2-2 P2）：几张卡一起领的那一队块，报了 Out of Memory 的卡交回的那一块先给下一张来领的卡；
-    /// 交回的都领走、队里也领完才算没有剩下的。不要卡。
-    #[cfg(feature = "gpu")]
-    #[test]
-    fn a_block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one() {
-        let blocks = BlocksForTheCards::new(3);
-        assert_eq!(blocks.take_a_block(), Some(0));
-        assert_eq!(blocks.take_a_block(), Some(1));
-        blocks.give_back(0);
-        assert!(blocks.any_left());
-        assert_eq!(blocks.take_a_block(), Some(0), "交回的先领");
-        assert_eq!(blocks.take_a_block(), Some(2));
-        assert_eq!(blocks.take_a_block(), None);
-        assert!(!blocks.any_left());
-        blocks.give_back(1);
-        assert!(blocks.any_left(), "队里领完了、还有交回的");
-        assert_eq!(blocks.take_a_block(), Some(1));
-        assert_eq!(blocks.take_a_block(), None);
-        assert_eq!(blocks.taken_from_the_head(), 3);
-    }
-
-    /// 第 10z 批（第 10y 批代码轮判决 K2-2 P2）：有卡报了 Out of Memory，别的卡把剩下的块都判完写进了库，这一趟照常交回；
-    /// 没判完才报本次未跑（带第一张报的卡那一句）；没有卡报就不归这里管。
-    #[cfg(feature = "gpu")]
+    /// 第 10z 批第三块（第 10z 批代码轮判决 K3 攻方第三节第 1445–1447 行）：一块的几段里有一段报了显存不够，这一块不写、不再领下一块，
+    /// 之前判完的块写了，交回的账带着那一段的原因。不要卡、不碰库：判一段与写一块都用假的。
     #[test]
-    fn an_out_of_memory_on_one_card_is_not_run_only_when_the_other_cards_left_blocks_unjudged() {
+    fn a_block_with_a_piece_short_of_gpu_memory_is_not_stored_and_no_later_block_is_taken() {
+        let pending: Vec<(usize, Range<u64>)> = vec![(0, 0..2), (0, 2..4), (0, 4..6), (0, 6..8)];
+        let points = vec![NamesOfACrashPoint {
+            name: "一个点".to_string(),
+            path_number: "0".to_string(),
+        }];
+        let mut stored_block_starts = Vec::new();
+        let tally = check_pending_blocks_in_pieces(
+            &pending,
+            &points,
+            2,
+            &|| false,
+            &|_point_index, local, ending| {
+                drop(ending);
+                let is_short = local.start == 5;
+                JudgedBlock {
+                    items_of_each_state: if is_short {
+                        Vec::new()
+                    } else {
+                        vec![
+                            Vec::new();
+                            usize::try_from(local.end - local.start).expect("一段装得进")
+                        ]
+                    },
+                    local,
+                    red_texts: Vec::new(),
+                    gpu_cpu_disagreements: 0,
+                    gpu_memory_was_short: is_short
+                        .then(|| "显存不够：块 4..6 的第二段".to_string()),
+                }
+            },
+            &mut |_point_index, local, items_of_each_state, _violating| {
+                assert_eq!(
+                    items_of_each_state.len(),
+                    usize::try_from(local.end - local.start).expect("一块装得进"),
+                    "写进库的块每个状态都有判定"
+                );
+                stored_block_starts.push(local.start);
+                Ok(())
+            },
+        )
+        .expect("假的写库不失败");
         assert_eq!(
-            cards_outcome_after_an_out_of_memory(3, 3, Some("显存不够：卡 1".to_string())),
-            Some(CardsOutcomeAfterAnOutOfMemory::EveryBlockJudged)
+            stored_block_starts,
+            vec![0, 2],
+            "报了的那一块与之后的都不写"
         );
+        assert_eq!(tally.blocks_checked, 2);
+        assert_eq!(tally.states_checked, 4);
         assert_eq!(
-            cards_outcome_after_an_out_of_memory(2, 3, Some("显存不够：卡 1".to_string())),
-            Some(CardsOutcomeAfterAnOutOfMemory::NotRunTheGpuMemoryWasShort {
-                reason: "显存不够：卡 1".to_string()
-            })
+            tally.stopped_because_the_gpu_memory_was_short,
+            Some("显存不够：块 4..6 的第二段".to_string())
         );
-        assert_eq!(cards_outcome_after_an_out_of_memory(2, 3, None), None);
     }
 
     /// 第 10z 批（第 10y 批代码轮正推 K3 列的第 7 处）：池里一张卡此刻照这一刻能借的算带得了几个状态，空着时照空着时能借的算。不要卡。
@@ -4190,6 +4297,7 @@ mod tests {
         let mut wall_clock_lines = WallClockProgressLines::every_progress_line_interval(clock());
         let mut printed: Vec<String> = Vec::new();
         let mut blocks_handed_to_the_store = 0usize;
+        let mut stored_in_order: BlocksStoredInOrder<GpuJudgedBlock> = BlocksStoredInOrder::new();
         let received = std::thread::scope(|scope| {
             scope.spawn(move || {
                 release_receiver.recv().expect("时钟读到第 4 次就放行");
@@ -4223,6 +4331,7 @@ mod tests {
                 &mut wall_clock_lines,
                 &clock,
                 3,
+                &mut stored_in_order,
                 &mut |_block| {
                     blocks_handed_to_the_store += 1;
                     Ok(())
@@ -4247,11 +4356,11 @@ mod tests {
             "收工的那张卡照样记下"
         );
         assert_eq!(
-            (received.blocks_stored, blocks_handed_to_the_store),
+            (stored_in_order.blocks_stored(), blocks_handed_to_the_store),
             (0, 0),
             "块 1 在等块 0，一块都不写"
         );
-        assert_eq!(received.storing_error, None);
+        assert_eq!(stored_in_order.storing_error(), None);
         assert_eq!(received.refusal, None);
     }
 
@@ -4305,7 +4414,7 @@ mod tests {
         );
     }
 
-    /// 第 10x 批：卡上 wgpu 报了 Out of Memory（[`CardMessage::OutOfMemory`]，文件头第 5 条）记成显存不够、留第一句，不记成拒判
+    /// 第 10x 批：卡上 wgpu 报了 Out of Memory（[`CardMessage::OutOfMemory`]，文件头第 5 条）记成显存不够、按卡记下，不记成拒判
     /// （拒判整趟判红，显存不够这一趟报本次未跑）；之前判完的块照写进库。不要卡：卡上的线程换成直接发这几句话。
     #[cfg(feature = "gpu")]
     #[test]
@@ -4318,20 +4427,26 @@ mod tests {
                 items_of_each_state: vec![Vec::new()],
             }))
             .expect("收的那一头还在");
-        for reason in ["第一张卡报的那一句", "第二张卡报的那一句"] {
+        for (index, reason) in [(0, "第一张卡报的那一句"), (1, "第二张卡报的那一句")]
+        {
             card_sender
-                .send(CardMessage::OutOfMemory(reason.to_string()))
+                .send(CardMessage::OutOfMemory {
+                    index,
+                    reason: reason.to_string(),
+                })
                 .expect("收的那一头还在");
         }
         drop(card_sender);
         let mut wall_clock_lines =
             WallClockProgressLines::every_progress_line_interval(std::time::Duration::ZERO);
         let mut blocks_handed_to_the_store = 0usize;
+        let mut stored_in_order: BlocksStoredInOrder<GpuJudgedBlock> = BlocksStoredInOrder::new();
         let received = receive_the_cards_messages_and_store_the_blocks_in_order(
             &card_receiver,
             &mut wall_clock_lines,
             &|| std::time::Duration::ZERO,
             2,
+            &mut stored_in_order,
             &mut |_block| {
                 blocks_handed_to_the_store += 1;
                 Ok(())
@@ -4339,16 +4454,19 @@ mod tests {
             &mut |_line| {},
         );
         assert_eq!(
-            received.out_of_memory.as_deref(),
-            Some("第一张卡报的那一句"),
-            "显存不够留第一句"
+            received.cards_out_of_memory,
+            vec![
+                (0, "第一张卡报的那一句".to_string()),
+                (1, "第二张卡报的那一句".to_string())
+            ],
+            "报了 Out of Memory 的卡按收到的次序都记下"
         );
         assert_eq!(
             received.refusal, None,
             "Out of Memory 不记成拒判：拒判整趟判红，显存不够报本次未跑"
         );
         assert_eq!(
-            (received.blocks_stored, blocks_handed_to_the_store),
+            (stored_in_order.blocks_stored(), blocks_handed_to_the_store),
             (1, 1),
             "Out of Memory 之前判完的块 0 照写进库"
         );
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index c3c1290..593c165 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -118,6 +118,16 @@ fn handle_an_uncaptured_wgpu_error(condition: &DeviceConditionOfACard, error: wg
     panic!("wgpu error: {error}\n")
 }
 
+/// 设备丢失的回调（装在每个设备上，[`start_the_card`]）：记进这个设备的情况，连同 wgpu 给的丢失原因与那一句（卡池下一次借这张卡时照它重起，
+/// 丢了之后撞上的 Out of Memory 照它认成显存不够）。
+fn record_the_loss_of_the_device(
+    condition: &DeviceConditionOfACard,
+    reason: wgpu::DeviceLostReason,
+    message: &str,
+) {
+    condition.record_the_device_lost(format!("{reason:?}：{message}"));
+}
+
 /// 一张卡起不来的两种。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum CardStartFailure {
@@ -328,7 +338,7 @@ fn start_the_device_and_the_pipeline(
     }));
     let condition_for_the_loss = std::sync::Arc::clone(&device_condition);
     device.set_device_lost_callback(move |reason, message| {
-        condition_for_the_loss.record_the_device_lost(format!("{reason:?}：{message}"));
+        record_the_loss_of_the_device(&condition_for_the_loss, reason, &message);
     });
     // 编单元校验和的管线：这条线程上推一个 Out of Memory 范围，卡被别的进程占满时显存不够报成起不来，不让默认处理 panic
     let out_of_memory_scope = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
@@ -1250,7 +1260,7 @@ pub mod gpu_test_start {
                 let card_position = booking.card_position();
                 let card = &self.cards[card_position];
                 card.started.started_or_start(
-                    |started: &GpuCard| started.device_condition().lost_reason().is_some(),
+                    |started: &GpuCard| started.device_condition().is_lost(),
                     || {
                         let which_card = format!(
                             "序号 {} 的卡（{}）",
@@ -1309,25 +1319,6 @@ pub mod gpu_test_start {
                 }
             }
 
-            /// 同 [`Self::lease_card_or_report_not_run`]，此刻借不到判红而不报「本次未跑」：给还没改成报本次未跑的调用方留着
-            /// （`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`，在删除线那一批手里）。名字是过时的，已经不等了
-            /// （第 10x 批：只记这个进程借的，借不到不等）；第 10x 批代码轮判决要它改名、同进程别的用例借着时不 panic，
-            /// 要连那几处调用一起改成经 [`Self::lease_card_or_report_not_run`]（借不到报本次未跑），那份文件改完删掉这一个。
-            ///
-            /// # Panics
-            /// 同 [`Self::lease_card_or_report_not_run`]；此刻每张卡都借不到。
-            #[must_use]
-            pub fn lease_card_waiting_for_room(
-                &'static self,
-                test_name: &str,
-                need_mebibytes: u64,
-            ) -> GpuCardLease {
-                self.lease_card(test_name, need_mebibytes)
-                    .unwrap_or_else(|reason| {
-                        panic!("{test_name}：{reason}（这一处还没改成经 lease_card_or_report_not_run 报本次未跑）")
-                    })
-            }
-
             /// 借一次：借到交借来的卡，此刻借不到、借到的卡上起设备时显存不够交一句为什么。
             fn lease_card(
                 &'static self,
@@ -1789,8 +1780,8 @@ mod tests {
     use super::{
         card_memory_reader_of_the_card_pool, failure_of_a_device_request,
         handle_an_uncaptured_wgpu_error, has_devices_started_outside_the_card_pool_on_the_bus,
-        started_with_its_owner_recorded, CardStartFailure, DeviceConditionOfACard,
-        DeviceOwnerOnTheCard,
+        record_the_loss_of_the_device, started_with_its_owner_recorded, CardStartFailure,
+        DeviceConditionOfACard, DeviceOwnerOnTheCard,
     };
     use singlefs_harness::gpu_memory_ledger::OccupiedByThisProcess;
 
@@ -2035,4 +2026,51 @@ mod tests {
             ])
         );
     }
+
+    /// 第 10z 批第三块（第 10z 批代码轮正推 K3 第 3 条）：设备丢失的回调把这个设备记成丢了，连同 wgpu 给的原因与那一句（卡池照它重起）。不要卡。
+    #[test]
+    fn the_device_lost_callback_records_the_device_as_lost_with_the_reason_and_the_message() {
+        let condition = DeviceConditionOfACard::default();
+        record_the_loss_of_the_device(&condition, wgpu::DeviceLostReason::Unknown, "驱动重置");
+        assert!(condition.is_lost());
+        assert_eq!(
+            condition.lost_reason(),
+            Some("Unknown：驱动重置".to_string())
+        );
+    }
+
+    /// 第 10z 批第三块（第 10z 批代码轮正推 K3 第 1 条）：不走卡池的两个取卡入口起设备都照不归卡池起（起之前记下总线号），卡池起设备只有一处、
+    /// 照归卡池起。扫这份源码（测试那一段之外），不要卡。
+    #[test]
+    fn the_entries_outside_the_card_pool_start_their_devices_as_outside_the_card_pool() {
+        let source = include_str!("gpu_unit_checks.rs");
+        let code = source
+            .split(concat!("#[cfg(test)]", "\n", "mod tests {"))
+            .next()
+            .expect("split 至少交一段");
+        let body_of = |signature: &str| {
+            let start = code
+                .find(signature)
+                .unwrap_or_else(|| panic!("源码里有 {signature}"));
+            let after = &code[start + signature.len()..];
+            &after[..after.find("\n}\n").expect("函数体以顶格的 } 收尾")]
+        };
+        let outside_the_pool = concat!(
+            "start_the_card(discovered, DeviceOwnerOnTheCard::",
+            "OutsideTheCardPool)"
+        );
+        for entry in ["pub fn gpu_cards_by_priority(", "pub fn usable_gpu_cards("] {
+            assert_eq!(
+                body_of(entry).matches(outside_the_pool).count(),
+                1,
+                "{entry} 起设备照不归卡池起"
+            );
+        }
+        assert_eq!(
+            code.matches(concat!("DeviceOwnerOnTheCard::", "TheCardPool)"))
+                .count(),
+            1,
+            "照归卡池起设备的只有卡池那一处"
+        );
+    }
 }
diff --git a/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs b/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
index d033ba8..8f6fc32 100644
--- a/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
@@ -14,14 +14,14 @@ use std::sync::Arc;
 
 use common::{build_pool, file_content, geometry};
 use singlefs_checker::crc32_castagnoli_table;
-use singlefs_checker_tier::crash::Layer0SegmentExpansion;
+use singlefs_checker_tier::crash::{layer0_plan_state_count, Layer0SegmentExpansion};
 use singlefs_checker_tier::crash_amplification::{
     check_recorded_blocks, checking_threads, plan_crash_points, record_crash_points, CrashFlow,
     CrashPointSpan, PipelineJudgingCode, UnitChecksumsOn, WeightedShares,
 };
 use singlefs_checker_tier::crash_identity::CoverageReport;
 use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
-    gpu_card_pool_or_report_not_run, GpuCardLease, GpuCardPool,
+    gpu_card_pool_or_report_not_run, not_run_line, GpuCardLease, GpuCardPool,
 };
 use singlefs_checker_tier::gpu_unit_checks::{
     check_unit_checksums_on_gpu, mebibytes_crc32c_of_ranges_needs, UnitChecksumVerdict,
@@ -278,7 +278,21 @@ fn the_pipeline_with_gpu_unit_checks_reports_no_red_and_no_disagreement_on_the_h
     .expect("核得完");
     drop(store);
     std::fs::remove_dir_all(&directory).expect("删得掉库");
-    assert!(tally.states_checked > 0);
+    // 单元校验和对拍时卡上报了 Out of Memory（第 10z 批代码轮判决 K1-3 正推）：那一块与之后的没判，不判绿也不判红，报本次未跑
+    if let Some(reason) = &tally.stopped_because_the_gpu_memory_was_short {
+        println!("{}", not_run_line(TEST_NAME, reason));
+        return;
+    }
+    assert_eq!(
+        tally.states_checked,
+        layer0_plan_state_count(
+            &recorded.base,
+            &recorded.writes,
+            &recorded.segments,
+            &expansion
+        ),
+        "没报显存不够：这条流的每个状态都判了"
+    );
     assert_eq!(
         tally.gpu_cpu_disagreements, 0,
         "GPU 与 CPU 一处都不许对不上"
````

## 变异表追加（`mutations-append.tsv` 原样）

````tsv
第10z批第三块 显存账 G2 甲 销了账的那一份照活着的借据都建满往下压（压到低于这个进程占着的，攻方丁、正推 S3 多给）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                reading.occupied_by_this_process,\n            );\n    }	                match reading.occupied_by_this_process { OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: mebibytes.saturating_sub(self.leases.values().sum::<u64>()) }, other => other },\n            );\n    }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_returned_lease_still_on_the_card_is_not_pressed_away_when_a_live_lease_shrinks_back_to_the_driver	a_returned_lease_still_on_the_card_is_not_pressed_away_when_a_live_lease_shrinks_back_to_the_driver
第10z批第三块 显存账 G2 甲 销了账的那一份每读一次整份压掉（第 10z 批「不多扣」那一形，S10 钉的甲的数变）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            returned_possibly_still_on_the_card_mebibytes.min(mebibytes)	            returned_possibly_still_on_the_card_mebibytes.min(0 * mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::returned_short_leases_while_a_long_lease_stays_built_each_stay_booked_as_possibly_on_the_card	returned_short_leases_while_a_long_lease_stays_built_each_stay_booked_as_possibly_on_the_card
第10z批第三块 显存账 K1-3 重起被拒之后再来直接放行（正推 S6）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            DriverShareOnTheCard::DeviceLostNotStartedAgain => {\n                self.driver_share_of_a_new_device_fits_now(&mut books, card_position)\n            }	            DriverShareOnTheCard::DeviceLostNotStartedAgain => Ok(()),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_refused_a_restart_is_judged_again_on_every_later_attempt	a_lost_device_refused_a_restart_is_judged_again_on_every_later_attempt
第10z批第三块 显存账 K3 重起恰好放得下判成放不下（正推 N10、攻方第 1310 行）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if capacity_mebibytes < booked.leases_mebibytes {	        if capacity_mebibytes <= booked.leases_mebibytes {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_started_again_fits_when_its_driver_share_fits_exactly	a_lost_device_started_again_fits_when_its_driver_share_fits_exactly
第10z批第三块 GPU 取卡 K3 线程上的活丢掉时不换回原来那段（正推 N1）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        WORK_ON_THE_CARD_OF_THIS_THREAD.with(|work| work.set(self.work_of_this_thread_before));	        WORK_ON_THE_CARD_OF_THIS_THREAD.with(|work| work.set(None));	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_work_begun_inside_another_hands_the_thread_back_to_the_outer_work_when_it_ends	a_work_begun_inside_another_hands_the_thread_back_to_the_outer_work_when_it_ends
第10z批第三块 GPU 取卡 K3 编内核的计数头一处编完就抹（正推 N2）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if still_compiling <= 1 {	        if still_compiling <= 2 {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_work_compiling_in_two_places_at_once_stays_compiling_until_both_end	a_work_compiling_in_two_places_at_once_stays_compiling_until_both_end
第10z批第三块 GPU 取卡 K3 编内核的计数不累加（正推 N3）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .or_insert(0) += 1;	            .or_insert(0) = 1;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_work_compiling_in_two_places_at_once_stays_compiling_until_both_end	a_work_compiling_in_two_places_at_once_stays_compiling_until_both_end
第10z批第三块 GPU 取卡 K3 设备丢了没有判反（卡池拿丢了的设备、没丢的反倒重起）	crates/singlefs-harness/src/gpu_memory_ledger.rs	    pub fn is_lost(&self) -> bool {\n        self.lost\n            .lock()\n            .unwrap_or_else(PoisonError::into_inner)\n            .is_some()	    pub fn is_lost(&self) -> bool {\n        self.lost\n            .lock()\n            .unwrap_or_else(PoisonError::into_inner)\n            .is_none()	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_device_recorded_lost_is_lost_and_another_is_not	a_device_recorded_lost_is_lost_and_another_is_not
第10z批第三块 GPU 取卡 K1-3 一块的几段先认后等（设备丢了先失败的那一段认不出同块另一段范围里接到的 Out of Memory，攻方）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        ending.end_and_wait_for_the_others();\n        self.out_of_memory_explaining_a_failure_of_the_work(work.identifier(), reports_before)	        let explained =\n            self.out_of_memory_explaining_a_failure_of_the_work(work.identifier(), reports_before);\n        ending.end_and_wait_for_the_others();\n        explained	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_piece_failing_on_a_lost_device_is_told_after_another_piece_records_its_out_of_memory	a_piece_failing_on_a_lost_device_is_told_after_another_piece_records_its_out_of_memory
第10z批第三块 GPU 取卡 K1-3 一段的报到丢掉不补报（那一段 panic 了别的段等它一辈子）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if !self.has_ended {\n            self.together.end_one_work();\n        }	        if !self.has_ended {}	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_piece_dropping_its_ending_without_ending_does_not_keep_the_others_waiting	a_piece_dropping_its_ending_without_ending_does_not_keep_the_others_waiting
第10z批第三块 崩溃放量 K2-1 P2 报了 Out of Memory 的卡不交回手里那一块（攻方第 2263 行）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                blocks.give_back(pending_index);\n                return CardLeftTheBlocks::OutOfMemory { reason };	                return CardLeftTheBlocks::OutOfMemory { reason };	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_block_a_card_gives_back_is_judged_once_and_every_block_is_stored_at_any_speed_of_two_cards	a_block_a_card_gives_back_is_judged_once_and_every_block_is_stored_at_any_speed_of_two_cards
第10z批第三块 崩溃放量 K2-1 P2 报了 Out of Memory 的卡交回之后接着领（攻方第 2265 行）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                return CardLeftTheBlocks::OutOfMemory { reason };	                let _ = reason;\n                continue;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_block_a_card_gives_back_is_judged_once_and_every_block_is_stored_at_any_speed_of_two_cards	a_block_a_card_gives_back_is_judged_once_and_every_block_is_stored_at_any_speed_of_two_cards
第10z批第三块 崩溃放量 K2-1 P2 判完的块不按块序写库	crates/singlefs-harness/src/gpu_memory_ledger.rs	        while let Some(next_block) = self.waiting.remove(&self.blocks_stored) {	        while let Some((_, next_block)) = self.waiting.pop_first() {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::judged_blocks_are_stored_in_block_order_and_none_after_a_storing_error	judged_blocks_are_stored_in_block_order_and_none_after_a_storing_error
第10z批第三块 崩溃放量 K2-1 P2 收尾不再起一轮（状态全装在第一张卡时第二张闲着，攻方）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        StopOfTheRun::NotRequested if cards_not_short_of_memory == 0 => {	        StopOfTheRun::NotRequested if cards_not_short_of_memory < usize::MAX => {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::with_every_state_on_the_first_card_the_blocks_it_leaves_are_judged_on_the_second_card	with_every_state_on_the_first_card_the_blocks_it_leaves_are_judged_on_the_second_card
第10z批第三块 崩溃放量 K3 单元校验和对拍时报了显存不够的那一块照样写库、接着领（攻方第 1445–1447 行）	crates/singlefs-checker-tier/src/crash_amplification.rs	        if let Some(reason) = reason_the_gpu_memory_was_short_in_one_of_the_pieces(&judged) {\n            tally.stopped_because_the_gpu_memory_was_short = Some(reason);\n            break;\n        }		-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::a_block_with_a_piece_short_of_gpu_memory_is_not_stored_and_no_later_block_is_taken	a_block_with_a_piece_short_of_gpu_memory_is_not_stored_and_no_later_block_is_taken
第10z批第三块 GPU 取卡 K3 设备丢失的回调不记丢失（正推 K3 第 3 条）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	    condition.record_the_device_lost(format!("{reason:?}：{message}"));	    let _ = (condition, reason, message);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- gpu_unit_checks::tests::the_device_lost_callback_records_the_device_as_lost_with_the_reason_and_the_message	the_device_lost_callback_records_the_device_as_lost_with_the_reason_and_the_message
第10z批第三块 GPU 取卡 K3 按优先级取卡的入口照归卡池起设备（起之前不记总线号，正推 K3 第 1 条）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            start_the_card(discovered, DeviceOwnerOnTheCard::OutsideTheCardPool).ok()\n        })\n        .collect();\n    let mut cards = Vec::with_capacity(configured.len());	            start_the_card(discovered, DeviceOwnerOnTheCard::TheCardPool).ok()\n        })\n        .collect();\n    let mut cards = Vec::with_capacity(configured.len());	-p singlefs-checker-tier --features verdict-store,gpu --lib -- gpu_unit_checks::tests::the_entries_outside_the_card_pool_start_their_devices_as_outside_the_card_pool	the_entries_outside_the_card_pool_start_their_devices_as_outside_the_card_pool
````

## 变异表替换（`mutations-replacements.tsv` 原样）

````tsv
第10y批 显存账 K1-1 销了账、卡上还没还的那一份不照读数往下压（卡上还了照样少借）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            returned_possibly_still_on_the_card_mebibytes.min(mebibytes)	            returned_possibly_still_on_the_card_mebibytes.max(mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_returned_lease_still_on_the_card_is_not_taken_for_a_live_lease_already_built	a_returned_lease_still_on_the_card_is_not_taken_for_a_live_lease_already_built
第10z批 显存账 K1-1 销了账的那一份照驱动那一份整份在卡上往下压（实占 x < D 多给 D − x，攻方丙形）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            returned_possibly_still_on_the_card_mebibytes.min(mebibytes)	            returned_possibly_still_on_the_card_mebibytes.min(mebibytes.saturating_sub(2560))	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_returned_lease_is_pressed_only_to_what_this_process_occupies_not_below_by_a_whole_driver_share	a_returned_lease_is_pressed_only_to_what_this_process_occupies_not_below_by_a_whole_driver_share
崩溃放量 卡上不理会调用方的叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	        &|| work.must_stop(),	        &|| work.stop.load(Ordering::SeqCst),	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest	the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest
崩溃放量 CPU 判器不理会调用方的叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	        if stop_is_requested() {\n            break;\n        }\n        let point = &points[*point_index];\n        let piece_length	        let point = &points[*point_index];\n        let piece_length	-p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest	a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest
第10w批 崩溃放量 起过卡池的产品路径第 ① 段草稿区改回不借	crates/singlefs-checker-tier/src/crash_amplification.rs	                match scratch_of_each_card_booked_by_where_the_cards_came_from(\n                    cards_taken_from,\n                    &cards_of_the_round,\n                    &run_cards,	                match scratch_of_each_card_booked_by_where_the_cards_came_from(\n                    GpuCardsSource::DevicesStartedForThisRun,\n                    &cards_of_the_round,\n                    &run_cards,	-p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_judges_on_scratch_it_booked_on_the_card_pool	the_product_path_judges_on_scratch_it_booked_on_the_card_pool
第10x批 崩溃放量 卡上报 Out of Memory 当成拒判（整趟判红，不报本次未跑）	crates/singlefs-checker-tier/src/crash_amplification.rs	                received.cards_out_of_memory.push((index, reason));	                let _ = index;\n                received.refusal.get_or_insert(reason);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::an_out_of_memory_on_a_card_is_kept_apart_from_a_refusal_and_the_blocks_judged_before_it_are_stored	an_out_of_memory_on_a_card_is_kept_apart_from_a_refusal_and_the_blocks_judged_before_it_are_stored
第10z批 崩溃放量 K2-2 P2 一张卡交回的块不再给别的卡领	crates/singlefs-harness/src/gpu_memory_ledger.rs	        let is_newly_given_back = self.blocks_not_taken_locked().insert(pending_index);	        let is_newly_given_back = !self.blocks_not_taken_locked().contains(&pending_index);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one	a_block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one
第10z批 崩溃放量 K2-2 P2 一张卡 Out of Memory 别的卡判完了也报本次未跑	crates/singlefs-harness/src/gpu_memory_ledger.rs	    if blocks_left_unstored == 0 {	    if false && blocks_left_unstored == 0 {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_block_a_card_gives_back_is_judged_once_and_every_block_is_stored_at_any_speed_of_two_cards	a_block_a_card_gives_back_is_judged_once_and_every_block_is_stored_at_any_speed_of_two_cards
````

## 变异表删除（`mutations-delete.txt` 原样）

````text
第10z批 显存账 K2-2 P1 往下压时不看早先见到占在卡上的（还给驱动的短借据一笔一笔累积压不掉）
第10z批 显存账 K2-2 P1 那一刻之后借的销账也从早先见到的那一份里减
第10z批 显存账 K2-2 P1 读到的不记成见到占在卡上的
````
