# 根环实现第 10z 批第十、十一块：代码轮第一轮改动全文（2026-10-08）

两块实现员各交的 `crates.patch` 原样，按合入次序：第十一块、第十块（第十一块打在第九块合入、删除线修 rollback_by_a_forward_publish 之后，第十块打在第十一块合入之后）。各块的变异表追加、整行替换、删除附在各自后面。

## 第十一块

````diff
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -23,9 +23,11 @@
 //!   减实际占着的里属于许出去的那部分。账上只有活着的借据：借据丢掉时卡池先等这张卡的设备把提交了的活都做完、丢掉的缓冲交回驱动，
 //!   再销账（[`GpuMemoryBooking::take_in_the_wait_before_returning`]；用户 2026-10-08 定第 10z 批代码轮岔路 G2「改用乙」，E174 在两张卡上量过：
 //!   等完之后下一次读数里那笔缓冲都已不在），销了账的那一份不再记、也不从空闲里扣（第 10z 批「甲」那一份「销了账、卡上可能还没还的」整个拿掉）。
-//!   等失败（超时或报错）的那一笔照旧记在账上，只会少借；它等的那台设备上提交的活后来做完了就销（卡池借之前对那张卡 poll 一次交队列空了、
-//!   或同一台设备上另一笔借据等成了，[`DeviceStartedOnTheBooks::take_in_that_the_submitted_work_finished`]），那台设备记成丢了、或放掉了也销
-//!   （第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：两腿都量到等超时的那几笔累加到整张卡借不出，到设备丢了才回来）。
+//!   等失败（超时或报错）的那一笔照旧记在账上，只会少借；它等的那台设备上提交的活后来做完了就销（卡池借之前列出每张卡上那一刻照旧记着的那几笔、
+//!   对那张卡 poll 一次交队列空了只销列出来的那几笔，[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]；
+//!   或同一台设备上另一笔借据等成了），那台设备记成丢了、或放掉了也销
+//!   （第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：两腿都量到等超时的那几笔累加到整张卡借不出，到设备丢了才回来；
+//!   第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K1-1：列卡之后才记成没等成的那一笔不一起销，它的活可以是 poll 之后才提交的）。
 //!   读不到实际占着的（`nvidia-smi` 没列这个进程），或这个进程在这张卡上另有不走这份账的设备（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]，
 //!   分不出哪一份是借据建的），借着的当一点都没建、驱动那一份也当还没占上卡（第 10y 批代码轮判决 K1-1 攻方乙附：当它整份在卡上，实占的 x 比 D 小时
 //!   多给 D − x），只会少借。这张卡上起好过的设备丢了、账照重起记了之后，到丢了的那几台都放掉之前也这样算
@@ -838,7 +840,8 @@
     /// 它们连上面的缓冲可能还占着卡，不空时这个进程占着的照分不出算（[`DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains`]）。
     lost_devices_not_released: BTreeSet<DeviceNumberOnTheCard>,
     /// 丢掉时等设备没等成（超时或报错）、照旧记在 `leases` 里的借据（借据号 → 它等的那一台，[`GpuMemoryBooking::take_in_the_wait_before_returning`]）：
-    /// 那一台上提交的活后来做完了（[`Self::return_the_leases_kept_after_a_failed_wait_on`]）、那一台记成丢了、或放掉了，销账。
+    /// 那一台上提交的活后来做完了（[`Self::return_the_leases_kept_after_a_failed_wait_on`]、
+    /// [`Self::return_the_listed_leases_kept_after_a_failed_wait_on`]）、那一台记成丢了、或放掉了，销账。
     /// 只记在用的那一台上的：等的是丢了还没放掉的那一台而没等成的，照常销账（那时这个进程占着的照分不出算，放掉之前它的缓冲交回驱动）。
     leases_kept_after_a_failed_wait: BTreeMap<u64, DeviceNumberOnTheCard>,
 }
@@ -895,6 +898,27 @@
             });
     }
 
+    /// 第 `device` 台上到列卡之后的某一刻为止提交的活都做完了（卡池在列卡之后 poll 交回队列空了）：`listed`（列卡那一刻这张卡上照旧记着的，
+    /// 借据号 → 它等的那一台）里等的是这一台、此刻还照旧记着的那几笔销账。列卡之后才记成没等成的不销：它的活可以是 poll 之后才提交的，
+    /// 缓冲可能还在卡上（第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K1-1 攻方：poll 之后、销账之前另一笔刚记成没等成，一起销掉，账多给）。
+    fn return_the_listed_leases_kept_after_a_failed_wait_on(
+        &mut self,
+        device: DeviceNumberOnTheCard,
+        listed: &BTreeMap<u64, DeviceNumberOnTheCard>,
+    ) {
+        // 一笔借据照旧记着时等的是哪一台，记上之后不变：列出来时等的是这一台、此刻还照旧记着的，就是还没销的那几笔
+        for (listed_lease, listed_on_the_device) in listed {
+            if *listed_on_the_device == device
+                && self
+                    .leases_kept_after_a_failed_wait
+                    .remove(listed_lease)
+                    .is_some()
+            {
+                self.leases.remove(listed_lease);
+            }
+        }
+    }
+
     /// 第 `device` 台放掉了（这个进程里它的句柄都丢了，wgpu 收掉设备、它上面的缓冲交回驱动）：在它上面没等成的那几笔销账；
     /// 它记成丢了的，不再让这张卡照分不出算，丢了的都放掉了、新设备已经起好，回到 [`DriverShareOnTheCard::DeviceStarted`]。
     fn release_a_device(&mut self, device: DeviceNumberOnTheCard) {
@@ -1376,17 +1400,27 @@
         }
     }
 
-    /// 哪几张卡上有等设备没等成而照旧记着的借据（卡池借之前对它们留着的设备 poll 一次，
-    /// [`DeviceStartedOnTheBooks::take_in_that_the_submitted_work_finished`]）。
+    /// 列卡：此刻每张有等设备没等成而照旧记着的借据的卡，连同那一刻照旧记着的那几笔（卡池借之前对这几张卡留着的设备 poll 一次，交回队列空了
+    /// 只销列出来的那几笔，[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]；第 10z 批第十一块，
+    /// 第 10z 批第七八九块代码轮判决 K1-1）。按卡在池里的先后。
     #[must_use]
-    pub fn cards_with_leases_kept_after_a_failed_wait(&self) -> Vec<usize> {
+    pub fn leases_kept_after_a_failed_wait_on_each_card(
+        &self,
+    ) -> Vec<LeasesKeptAfterAFailedWaitWhenListed> {
         self.books_locked()
             .iter()
             .enumerate()
             .filter(|(_card_position, books_of_the_card)| {
                 !books_of_the_card.leases_kept_after_a_failed_wait.is_empty()
             })
-            .map(|(card_position, _books_of_the_card)| card_position)
+            .map(
+                |(card_position, books_of_the_card)| LeasesKeptAfterAFailedWaitWhenListed {
+                    card_position,
+                    leases_kept_on_each_device: books_of_the_card
+                        .leases_kept_after_a_failed_wait
+                        .clone(),
+                },
+            )
             .collect()
     }
 
@@ -1399,6 +1433,30 @@
     }
 }
 
+/// 列卡那一刻一张卡上等设备没等成而照旧记着的那几笔（[`GpuMemoryLedger::leases_kept_after_a_failed_wait_on_each_card`]）：
+/// 卡池列卡之后对这张卡留着的设备 poll，交回队列空了只销这几笔（[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]）。
+/// 列卡之后才记成没等成的那一笔不在里面：它的活可以是 poll 之后才提交的。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct LeasesKeptAfterAFailedWaitWhenListed {
+    card_position: usize,
+    /// 借据号 → 它等的那一台。
+    leases_kept_on_each_device: BTreeMap<u64, DeviceNumberOnTheCard>,
+}
+
+impl LeasesKeptAfterAFailedWaitWhenListed {
+    /// 在池里第几张卡上。
+    #[must_use]
+    pub fn card_position(&self) -> usize {
+        self.card_position
+    }
+
+    /// 列出来的借据号（从小到大）。
+    #[must_use]
+    pub fn lease_identifiers(&self) -> Vec<u64> {
+        self.leases_kept_on_each_device.keys().copied().collect()
+    }
+}
+
 /// 按状态数在一张卡上借到的草稿区：借了几个状态（在锁里照此刻能借的压过）与那一笔借据（丢掉时还）。
 pub struct StatesBookedOnACard<'ledger> {
     pub states: u64,
@@ -1539,11 +1597,27 @@
 }
 
 impl DeviceStartedOnTheBooks<'_> {
-    /// 这台设备上到此刻为止提交的活都做完了（卡池借之前对它 poll 一次交回队列空了：wgpu 收完那几次提交、丢掉的缓冲交回了驱动）：
-    /// 在它上面等设备没等成而照旧记着的那几笔销账（第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：两次超时之间没有别的等成时也出得来）。
-    pub fn take_in_that_the_submitted_work_finished(&self) {
+    /// 列卡（`listed`，[`GpuMemoryLedger::leases_kept_after_a_failed_wait_on_each_card`]）之后对这台设备 poll 一次交回队列空了：到 poll 那一刻为止提交的活
+    /// 都做完了，wgpu 收完那几次提交、丢掉的缓冲交回了驱动。列出来的、等的是这一台、此刻还照旧记着的那几笔销账（它们的缓冲在列卡之前就丢了、
+    /// 活在那之前提交）；列卡之后才记成没等成的照旧记着（第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：两次超时之间没有别的等成时也出得来；
+    /// 第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K1-1）。
+    ///
+    /// # Panics
+    /// `listed` 列的不是这台设备那张卡。
+    pub fn take_in_that_the_work_of_the_listed_leases_finished(
+        &self,
+        listed: &LeasesKeptAfterAFailedWaitWhenListed,
+    ) {
+        assert_eq!(
+            listed.card_position, self.card_position,
+            "列的是第 {} 张卡，poll 的设备在第 {} 张卡上：只销这张卡上列出来的",
+            listed.card_position, self.card_position
+        );
         self.ledger.books_locked()[self.card_position]
-            .return_the_leases_kept_after_a_failed_wait_on(self.device);
+            .return_the_listed_leases_kept_after_a_failed_wait_on(
+                self.device,
+                &listed.leases_kept_on_each_device,
+            );
     }
 }
 
@@ -1582,8 +1656,14 @@
 /// 一张卡上这个进程的设备（卡池每张卡一格）：要时有留着的、没丢的就拿它；没有（第一次要、上一次起不来）或丢了就起一个，
 /// 起好了留着，起不来交为什么、不留着（第 10x 批代码轮判决 K1-3：起设备失败不缓存成这个进程永久的错；设备丢了之后借这张卡的
 /// 不拿丢了的那个）。不碰 wgpu：设备、怎么起、怎么算丢了由卡池给，单测换成假的。
+/// 两把锁：起设备时只拿 `starting`（同一张卡上同时要起的等这一次起完），留着的那一份只在取、换的那一下拿 `started`，
+/// 取留着的设备（[`Self::started_now`]，卡池借之前 poll 用）不等正在起的设备（第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K4-2：
+/// 正推量到一次 300 毫秒的重起里借之前那一下等了 250 毫秒，哪怕借的是别的卡）。
 pub struct DeviceStartedOnDemand<Device> {
+    /// 留着的设备：持锁的那几段只取克隆、换一份，不起设备。
     started: Mutex<Option<Device>>,
+    /// 起设备的那一段拿着。
+    starting: Mutex<()>,
 }
 
 /// 空着（还没起设备）就是起点，没有别的不变量。
@@ -1598,11 +1678,21 @@
     pub const fn new() -> Self {
         Self {
             started: Mutex::new(None),
+            starting: Mutex::new(()),
         }
     }
 
+    /// 留着的那一份的克隆（`started` 只在这一下拿）。持锁的这一段只克隆，不 panic；别的线程 panic 留下的毒照用。
+    fn clone_of_the_device_kept(&self) -> Option<Device> {
+        self.started
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .clone()
+    }
+
     /// 留着的设备 `is_lost` 不为真就交它的克隆；否则调 `start` 起一个，起好了留着、交克隆，起不来交 `start` 的错（不留着）。
-    /// 起的时候拿着这张卡的锁：同一张卡上同时要的等这一次起完（别的卡不等）。
+    /// 起的时候拿着这张卡起设备的那把锁：同一张卡上同时要起的等这一次起完、再看一次留着的（别的卡不等）；
+    /// 取留着的设备（[`Self::started_now`]、[`Self::is_started`]）不等。
     ///
     /// # Errors
     /// `start` 的错。
@@ -1611,19 +1701,31 @@
         is_lost: impl Fn(&Device) -> bool,
         start: impl FnOnce() -> Result<Device, Failure>,
     ) -> Result<Device, Failure> {
-        // 持锁时 panic 只会在 `start` 里（起设备的断言）：那时这一格还是旧的（没有或丢了的），别的线程照用、照起
-        let mut started = self.started.lock().unwrap_or_else(PoisonError::into_inner);
-        if let Some(device) = started.as_ref() {
-            if !is_lost(device) {
-                return Ok(device.clone());
+        if let Some(device) = self.clone_of_the_device_kept() {
+            if !is_lost(&device) {
+                return Ok(device);
+            }
+        }
+        // 持这把锁时 panic 只会在 `start` 里（起设备的断言）：那时留着的还是旧的（没有或丢了的），别的线程照用、照起
+        let _starting = self.starting.lock().unwrap_or_else(PoisonError::into_inner);
+        // 等这把锁的那一段里别的线程可能已经起好了：再看一次，起好了就用它，不再起
+        if let Some(device) = self.clone_of_the_device_kept() {
+            if !is_lost(&device) {
+                return Ok(device);
             }
         }
         let device = start()?;
-        *started = Some(device.clone());
+        // 换下来的旧设备在放开 `started` 之后才丢：它可能是这个进程里那台设备的最后一个克隆，丢的时候 wgpu 要等设备空了再收拾
+        let replaced = self
+            .started
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner)
+            .replace(device.clone());
+        drop(replaced);
         Ok(device)
     }
 
-    /// 起过、留着一个设备没有（丢了的也算留着，下一次要时重起）。
+    /// 起过、留着一个设备没有（丢了的也算留着，下一次要时重起）；不等正在起的设备。
     #[must_use]
     pub fn is_started(&self) -> bool {
         self.started
@@ -1632,13 +1734,11 @@
             .is_some()
     }
 
-    /// 留着的设备的克隆（丢了的也交；没起过、上一次没起来是 `None`）；不起设备（卡池借之前 poll 留着的设备用）。
+    /// 留着的设备的克隆（丢了的也交；没起过、上一次没起来是 `None`）；不起设备，也不等正在起的设备：起着的时候交的是起之前留着的那一份
+    /// （没有或丢了的；卡池借之前 poll 留着的设备用，丢了的不 poll）。
     #[must_use]
     pub fn started_now(&self) -> Option<Device> {
-        self.started
-            .lock()
-            .unwrap_or_else(PoisonError::into_inner)
-            .clone()
+        self.clone_of_the_device_kept()
     }
 }
 
@@ -2637,7 +2737,8 @@
     use super::{
         next_card_to_start, wait_for_the_device_and_take_in_the_outcome,
         work_caught_noting_the_device_when_it_returned, DeviceStartedOnTheBooks,
-        DeviceWaitBeforeReturningTheLease, StopRequestRememberedInThisRun,
+        DeviceWaitBeforeReturningTheLease, LeasesKeptAfterAFailedWaitWhenListed,
+        StopRequestRememberedInThisRun,
     };
     use std::sync::atomic::{AtomicUsize, Ordering};
     use std::sync::{Arc, Mutex};
@@ -4070,6 +4171,113 @@
         assert!(slot.is_started());
     }
 
+    /// 第 10z 批第十一块（第 10z 批第七八九块代码轮判决 K4-2，正推 `probe_started_now_blocks.rs`：取留着的设备与起设备共用一把锁，
+    /// 300 毫秒的重起里借之前那一下等了 250 毫秒）：留着的设备 1 丢了，一条线程在重起、起到一半停住；这时取留着的设备不等，
+    /// 交起之前留着的那一台（丢了的 1，卡池借之前不 poll 丢了的）；重起完了交新起的 2。
+    #[test]
+    fn taking_the_device_kept_does_not_wait_for_a_restart_in_progress() {
+        let slot: DeviceStartedOnDemand<u32> = DeviceStartedOnDemand::new();
+        assert_eq!(
+            slot.started_or_start(|_device: &u32| false, || Ok::<u32, String>(1)),
+            Ok(1)
+        );
+        let slot_shared = &slot;
+        let (restart_began_sender, restart_began) = std::sync::mpsc::channel::<()>();
+        let (finish_the_restart, restart_may_finish) = std::sync::mpsc::channel::<()>();
+        let (taken_sender, taken) = std::sync::mpsc::channel::<Option<u32>>();
+        std::thread::scope(|scope| {
+            let restarting = scope.spawn(move || {
+                slot_shared.started_or_start(
+                    |device: &u32| *device == 1,
+                    move || {
+                        restart_began_sender.send(()).expect("用例在等重起起头");
+                        // 用例断言失败、放开之前就丢了发的那一头时也收尾
+                        let _finished_or_the_test_gave_up = restart_may_finish.recv();
+                        Ok::<u32, String>(2)
+                    },
+                )
+            });
+            restart_began.recv().expect("重起起头了");
+            scope.spawn(move || {
+                taken_sender
+                    .send(slot_shared.started_now())
+                    .expect("用例在等取到的那一台");
+            });
+            let taken_during_the_restart = taken.recv_timeout(std::time::Duration::from_secs(10));
+            finish_the_restart.send(()).expect("重起在等放开");
+            assert_eq!(
+                taken_during_the_restart,
+                Ok(Some(1)),
+                "重起着的时候取留着的设备不等：交起之前留着的那一台（丢了的 1）"
+            );
+            assert_eq!(restarting.join().expect("重起不 panic"), Ok(2));
+        });
+        assert_eq!(
+            slot.started_now(),
+            Some(2),
+            "重起完了：留着的是新起的那一台"
+        );
+    }
+
+    /// 第 10z 批第十一块（K4-2 拆了锁之后）：同一张卡上一条线程在重起，另一条也要这张卡的设备：等这一次起完、拿它起好的那一台，不另起一台。
+    #[test]
+    fn a_second_start_on_the_same_card_waits_for_the_restart_in_progress_and_takes_its_device() {
+        let slot: DeviceStartedOnDemand<u32> = DeviceStartedOnDemand::new();
+        assert_eq!(
+            slot.started_or_start(|_device: &u32| false, || Ok::<u32, String>(1)),
+            Ok(1)
+        );
+        let slot_shared = &slot;
+        let starts_by_the_second = AtomicUsize::new(0);
+        let starts_by_the_second_shared = &starts_by_the_second;
+        let (restart_began_sender, restart_began) = std::sync::mpsc::channel::<()>();
+        let (finish_the_restart, restart_may_finish) = std::sync::mpsc::channel::<()>();
+        let (second_sender, second) = std::sync::mpsc::channel::<Result<u32, String>>();
+        std::thread::scope(|scope| {
+            let restarting = scope.spawn(move || {
+                slot_shared.started_or_start(
+                    |device: &u32| *device == 1,
+                    move || {
+                        restart_began_sender.send(()).expect("用例在等重起起头");
+                        let _finished_or_the_test_gave_up = restart_may_finish.recv();
+                        Ok::<u32, String>(2)
+                    },
+                )
+            });
+            restart_began.recv().expect("重起起头了");
+            scope.spawn(move || {
+                second_sender
+                    .send(slot_shared.started_or_start(
+                        |device: &u32| *device == 1,
+                        || {
+                            starts_by_the_second_shared.fetch_add(1, Ordering::SeqCst);
+                            Ok::<u32, String>(3)
+                        },
+                    ))
+                    .expect("用例在等第二条拿到的那一台");
+            });
+            let second_before_the_restart_finished =
+                second.recv_timeout(std::time::Duration::from_millis(500));
+            finish_the_restart.send(()).expect("重起在等放开");
+            assert_eq!(
+                second_before_the_restart_finished,
+                Err(std::sync::mpsc::RecvTimeoutError::Timeout),
+                "第一条还在起：第二条等它起完，不先交回"
+            );
+            assert_eq!(restarting.join().expect("重起不 panic"), Ok(2));
+            assert_eq!(
+                second.recv_timeout(std::time::Duration::from_secs(10)),
+                Ok(Ok(2)),
+                "第二条拿的是第一条起好的那一台"
+            );
+        });
+        assert_eq!(
+            starts_by_the_second.load(Ordering::SeqCst),
+            0,
+            "第二条没另起一台"
+        );
+    }
+
     /// 第 10z 批（第 10y 批代码轮判决 K1-1 攻方乙形）：不走账的设备起之前就记下它在哪张卡上：起的那一段里卡池读这张卡已经认得出
     /// 这个进程另有不走账的设备；没起过的卡不记。
     #[test]
@@ -4880,13 +5088,20 @@
                 room_now(&ledger) <= room_truly_left(&card, 0),
                 "第 {timed_out_lease_number} 笔的缓冲还在卡上时不多给"
             );
+            let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
             assert_eq!(
-                ledger.cards_with_leases_kept_after_a_failed_wait(),
-                vec![0],
+                listed
+                    .iter()
+                    .map(|listed_on_a_card| (
+                        listed_on_a_card.card_position(),
+                        listed_on_a_card.lease_identifiers()
+                    ))
+                    .collect::<Vec<(usize, Vec<u64>)>>(),
+                vec![(0, vec![timed_out_lease])],
                 "第 {timed_out_lease_number} 笔：这张卡上有没等成而照旧记着的，卡池借之前要 poll 它"
             );
             card.lock().expect("假卡不 panic").built_mebibytes = 0;
-            device.take_in_that_the_submitted_work_finished();
+            device.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
             assert!(
                 !ledger.is_the_lease_on_the_books(0, timed_out_lease),
                 "第 {timed_out_lease_number} 笔：poll 交回队列空了，那一笔销账"
@@ -4898,10 +5113,118 @@
             );
         }
         assert!(ledger
-            .cards_with_leases_kept_after_a_failed_wait()
+            .leases_kept_after_a_failed_wait_on_each_card()
             .is_empty());
     }
 
+    /// 第 10z 批第十一块（第 10z 批第七八九块代码轮判决 K1-1，攻方 `opus_k11_poll_race.rs` 的 take-in-between：卡池列卡、poll 交回队列空了，
+    /// 销账之前另一条线程借了一笔、建满、提交了活、等设备没等成；销账时它被一起销了，它的缓冲还在卡上，账多给）：X 10 000 建满、没等成、照旧记着；
+    /// 卡池列卡（列出 X）、poll 交回队列空了（X 的缓冲还给驱动）；A 借 8 000、建满、没等成；卡池照列出来的销：X 销账，A 照旧记着，不多给。
+    /// 下一次列卡列出 A，A 的缓冲还给驱动之后再 poll 交回队列空了，A 才销账。
+    #[test]
+    fn a_lease_kept_after_the_listing_stays_on_the_books_when_the_poll_after_the_listing_finds_the_device_idle(
+    ) {
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
+        let lease_x =
+            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 10_000);
+        let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
+        assert_eq!(
+            listed
+                .iter()
+                .map(LeasesKeptAfterAFailedWaitWhenListed::lease_identifiers)
+                .collect::<Vec<Vec<u64>>>(),
+            vec![vec![lease_x]],
+            "列卡那一刻照旧记着的只有 X"
+        );
+        // poll 交回队列空了：X 的活做完、缓冲还给驱动
+        card.lock().expect("假卡不 panic").built_mebibytes = 0;
+        let lease_a =
+            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 8_000);
+        device.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, lease_x),
+            "X 列出来了、poll 那一刻之前提交的活都做完了：销账"
+        );
+        assert!(
+            ledger.is_the_lease_on_the_books(0, lease_a),
+            "A 是列卡之后才记成没等成的：它的活可以是 poll 之后才提交的，照旧记着"
+        );
+        assert!(
+            room_now(&ledger) <= room_truly_left(&card, 0),
+            "A 的缓冲还在卡上：不多给（{} 对 {}）",
+            room_now(&ledger),
+            room_truly_left(&card, 0)
+        );
+        let listed_again = ledger.leases_kept_after_a_failed_wait_on_each_card();
+        card.lock().expect("假卡不 panic").built_mebibytes = 0;
+        device.take_in_that_the_work_of_the_listed_leases_finished(&listed_again[0]);
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, lease_a),
+            "下一次列卡列出 A、poll 交回队列空了：A 销账"
+        );
+        assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
+    }
+
+    /// 第 10z 批第十一块（第 10z 批第七八九块代码轮攻方 `scripts/mut.py` 的 M1、M2 照绿：放掉的是在用的那一台没有用例）：在用的那一台上 X 没等成、
+    /// 照旧记着，这台设备的克隆都丢了（卡池整个丢掉那一类；今天卡池活到进程退出，产品路径走不到，账的接口上走得到）：X 销账、这张卡上不再有在用的那一台，
+    /// 下一台起得来，能借的是卡上真能再给的。
+    #[test]
+    fn releasing_the_device_in_use_returns_the_leases_kept_on_it_and_a_new_device_starts_on_the_card(
+    ) {
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
+        let kept_lease =
+            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 4_000);
+        {
+            // 放掉：wgpu 收掉设备，它上面的缓冲与驱动那一份交回驱动
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = false;
+            card.built_mebibytes = 0;
+        }
+        drop(device);
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, kept_lease),
+            "在用的那一台放掉了：它上面没等成的那一笔销账"
+        );
+        let next_lease = ledger.book_on_the_card(0, 1_000).expect("借得到 1 000");
+        let next_device = ledger.record_the_device_started_on_the_card(0);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = 600;
+        }
+        assert_eq!(
+            room_now(&ledger),
+            room_truly_left(&card, 1_000),
+            "下一台起好了：能借的是卡上真能再给的"
+        );
+        drop(next_lease);
+        drop(next_device);
+    }
+
     /// 第 10z 批第八块（第 10z 批第六块代码轮攻方改法 F-K1）：没等成的那一笔记着，同一台设备上后来另一笔借据等成了（等的是到那一刻为止的全部提交）：
     /// 没等成的那一笔一起销账。
     #[test]
@@ -4985,7 +5308,9 @@
             room_truly_left(&card, 0)
         );
         card.lock().expect("假卡不 panic").built_mebibytes = 0;
-        replacement.take_in_that_the_submitted_work_finished();
+        let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
+        assert_eq!(listed.len(), 1, "B 照旧记在这张卡上：列得出");
+        replacement.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
         assert!(!ledger.is_the_lease_on_the_books(0, starting_the_replacement));
         assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
     }
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -1326,13 +1326,17 @@
                     )
             }
 
-            /// 借之前：账上有等设备没等成而照旧记着的借据的卡，对池里那张卡留着的设备 poll 一次（不等）；交回队列空了，那台设备上提交的活
-            /// 都做完了、丢掉的缓冲交回了驱动，那几笔销账（第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：等超时的那一笔不永久少借，
-            /// 两次超时之间没有别的等成时也出得来）。留着的设备丢了的不 poll（wgpu 在丢了的设备上 poll 会 panic）：那几笔到重起时照重起记销。
-            /// 不在账的锁里 poll：队列空了是在 poll 那一刻，那几笔丢掉的缓冲早在那之前提交、丢掉，之后什么时候销都不多给。
+            /// 借之前：列出账上有等设备没等成而照旧记着的借据的卡、连同那一刻照旧记着的那几笔，对池里那张卡留着的设备 poll 一次（不等）；
+            /// 交回队列空了，那台设备上到 poll 那一刻为止提交的活都做完了、丢掉的缓冲交回了驱动，列出来的那几笔销账（第 10z 批第八块，
+            /// 第 10z 批第六块代码轮判决 K1-2：等超时的那一笔不永久少借，两次超时之间没有别的等成时也出得来）。留着的设备丢了的不 poll
+            /// （wgpu 在丢了的设备上 poll 会 panic）：那几笔到重起时照重起记销。取留着的设备不等别的线程起设备（`DeviceStartedOnDemand::started_now`，
+            /// 第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K4-2）。
+            /// 不在账的锁里 poll，只销列卡那一刻记着的：它们丢掉的缓冲在列卡之前提交、丢掉，poll 交回队列空了之后什么时候销都不多给；
+            /// 列卡之后才记成没等成的那一笔不销，它的活可以是 poll 之后才提交的（第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K1-1）。
             fn take_in_the_devices_found_idle(&self) {
-                for card_position in self.ledger.cards_with_leases_kept_after_a_failed_wait() {
-                    let Some(started) = self.cards[card_position].started.started_now() else {
+                for listed in self.ledger.leases_kept_after_a_failed_wait_on_each_card() {
+                    let Some(started) = self.cards[listed.card_position()].started.started_now()
+                    else {
                         continue;
                     };
                     if started.device_condition().is_lost() {
@@ -1346,7 +1350,7 @@
                             .device_on_the_books
                             .as_ref()
                             .expect("卡池起的设备起好时放进了账上的那一台（started_card）")
-                            .take_in_that_the_submitted_work_finished();
+                            .take_in_that_the_work_of_the_listed_leases_finished(&listed);
                     }
                 }
             }
@@ -2371,14 +2375,27 @@
             "pub fn book_the_scratch_of_the_cards_with_room_now(",
             "pub fn card_rooms_now_and_when_empty_of(",
         ] {
+            // 第 10z 批第十一块（第 10z 批第七八九块代码轮判决 K5，正推六·二：那一句注释掉、包进 `if false` 照绿）：注释行不算，
+            // 那一句要整行、缩进十六格（方法体顶层的语句），包进别的块里的缩进更深、同一行里另有别的不是整行
             let body = body_until(entry, "\n            \u{7d}\n");
-            let polled = body
-                .find("self.take_in_the_devices_found_idle();")
-                .unwrap_or_else(|| panic!("{entry} 借之前先 poll 没等成的那几张卡：{body}"));
-            let booked = body
-                .find("ledger")
-                .unwrap_or_else(|| panic!("{entry} 照账借：{body}"));
-            assert!(polled < booked, "{entry} 先 poll、再照账借：{body}");
+            let code_lines: Vec<&str> = body
+                .lines()
+                .filter(|line| !line.trim_start().starts_with("//"))
+                .collect();
+            let polled = code_lines
+                .iter()
+                .position(|line| *line == "                self.take_in_the_devices_found_idle();")
+                .unwrap_or_else(|| {
+                    panic!("{entry} 借之前先 poll 没等成的那几张卡（方法体顶层整行一句）：{code_lines:#?}")
+                });
+            let booked = code_lines
+                .iter()
+                .position(|line| line.contains("ledger"))
+                .unwrap_or_else(|| panic!("{entry} 照账借：{code_lines:#?}"));
+            assert!(
+                polled < booked,
+                "{entry} 先 poll、再照账借：{code_lines:#?}"
+            );
         }
     }
 
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -4783,6 +4783,48 @@
         );
     }
 
+    /// 第 10z 批第十一块（第 10z 批第七八九块代码轮判决 K5，正推六·二：交给第 ① 段在卡上判与 CPU 判器的叫停那一行换成永不叫停，
+    /// 上一条照绿）：这一趟记住的那一个叫停（`stop_is_requested`）就是交给 `judge_my_blocks_on_gpu` 与 `check_selected_blocks` 的那一个
+    /// （两处调用的最后一个参数整行比）。下游各层把收到的叫停原样往下交，换掉就是那一层的参数没用上、编译器报没用的变量。扫这份源码，不要卡。
+    #[test]
+    fn the_stop_request_remembered_in_this_run_is_the_one_handed_to_the_gpu_judge_and_the_cpu_checker(
+    ) {
+        let lines: Vec<String> = code_lines_outside_the_tests()
+            .iter()
+            .map(|line| line.trim().to_string())
+            .collect();
+        for (call_opening, what) in [
+            (
+                concat!(
+                    "(ComputedOn::Gpu, Some(cards)) => match judge_my_blocks_on_gpu",
+                    "("
+                ),
+                "第 ① 段在卡上判",
+            ),
+            (
+                concat!("let checking = check_selected_blocks", "("),
+                "第 ① 段 CPU 判器",
+            ),
+        ] {
+            let openings: Vec<usize> = lines
+                .iter()
+                .enumerate()
+                .filter(|(_line_index, line)| line.as_str() == call_opening)
+                .map(|(line_index, _line)| line_index)
+                .collect();
+            assert_eq!(openings.len(), 1, "{what}：这份源码里有一处 {call_opening}");
+            let arguments: Vec<&String> = lines[openings[0] + 1..]
+                .iter()
+                .take_while(|line| !line.starts_with(')'))
+                .collect();
+            assert_eq!(
+                arguments.last().map(|argument| argument.as_str()),
+                Some(concat!("&stop_is_", "requested,")),
+                "{what}收到的叫停是这一趟记住的那一个：{arguments:?}"
+            );
+        }
+    }
+
     fn registered(classes: &[GpuComputationClass]) -> BTreeSet<GpuComputationClass> {
         classes.iter().copied().collect()
     }
````

### 第十一块 mutations-append.tsv

````
第10z批第十一块 显存账 K1-1 poll 交回队列空了连列卡之后才没等成的也销（攻方 take-in-between）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .return_the_listed_leases_kept_after_a_failed_wait_on(\n                self.device,\n                &listed.leases_kept_on_each_device,\n            );	            .return_the_leases_kept_after_a_failed_wait_on(self.device);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_kept_after_the_listing_stays_on_the_books_when_the_poll_after_the_listing_finds_the_device_idle	a_lease_kept_after_the_listing_stays_on_the_books_when_the_poll_after_the_listing_finds_the_device_idle
第10z批第十一块 显存账 K4-2 借之前取留着的设备照旧等正在起的设备（正推 probe_started_now_blocks）	crates/singlefs-harness/src/gpu_memory_ledger.rs	    pub fn started_now(&self) -> Option<Device> {\n        self.clone_of_the_device_kept()	    pub fn started_now(&self) -> Option<Device> {\n        let _starting = self.starting.lock().unwrap_or_else(PoisonError::into_inner);\n        self.clone_of_the_device_kept()	-p singlefs-harness --lib -- gpu_memory_ledger::tests::taking_the_device_kept_does_not_wait_for_a_restart_in_progress	taking_the_device_kept_does_not_wait_for_a_restart_in_progress
第10z批第十一块 显存账 K4-2 拆锁之后同一张卡上同时要起的不等这一次起完（各起一台）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        let _starting = self.starting.lock().unwrap_or_else(PoisonError::into_inner);\n        // 等这把锁的那一段里	        // 等这把锁的那一段里	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_second_start_on_the_same_card_waits_for_the_restart_in_progress_and_takes_its_device	a_second_start_on_the_same_card_waits_for_the_restart_in_progress_and_takes_its_device
第10z批第十一块 显存账 K5 放掉在用的那一台时不销它上面没等成的（攻方 mut.py M1）	crates/singlefs-harness/src/gpu_memory_ledger.rs	    fn release_a_device(&mut self, device: DeviceNumberOnTheCard) {\n        self.return_the_leases_kept_after_a_failed_wait_on(device);	    fn release_a_device(&mut self, device: DeviceNumberOnTheCard) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::releasing_the_device_in_use_returns_the_leases_kept_on_it_and_a_new_device_starts_on_the_card	releasing_the_device_in_use_returns_the_leases_kept_on_it_and_a_new_device_starts_on_the_card
第10z批第十一块 显存账 K5 放掉在用的那一台时不把在用的清掉（攻方 mut.py M2）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if self.device_in_use == Some(device) {\n            self.device_in_use = None;\n        }	        // 变异：放掉在用的那一台不清	-p singlefs-harness --lib -- gpu_memory_ledger::tests::releasing_the_device_in_use_returns_the_leases_kept_on_it_and_a_new_device_starts_on_the_card	releasing_the_device_in_use_returns_the_leases_kept_on_it_and_a_new_device_starts_on_the_card
第10z批第十一块 GPU 取卡 K5 借之前 poll 那一句注释掉（正推六·二 X4）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                self.take_in_the_devices_found_idle();\n                let booking = match self.ledger.book_on_the_first_card_with_room(need_mebibytes) {	                // self.take_in_the_devices_found_idle();\n                let booking = match self.ledger.book_on_the_first_card_with_room(need_mebibytes) {	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first	the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first
第10z批第十一块 GPU 取卡 K5 借之前 poll 那一句包进 if false（正推六·二 X4b）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                self.take_in_the_devices_found_idle();\n                self.ledger\n                    .card_rooms_now_and_when_empty(&card_positions)	                if false {\n                    self.take_in_the_devices_found_idle();\n                }\n                self.ledger\n                    .card_rooms_now_and_when_empty(&card_positions)	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first	the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first
第10z批第十一块 崩溃放量 K5 交给第 ① 段在卡上判的叫停换成永不叫停（正推六·二 X1）	crates/singlefs-checker-tier/src/crash_amplification.rs	            gpu_cards_taken_from,\n            &stop_is_requested,\n        )? {	            gpu_cards_taken_from,\n            &|| false,\n        )? {	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_remembered_in_this_run_is_the_one_handed_to_the_gpu_judge_and_the_cpu_checker	the_stop_request_remembered_in_this_run_is_the_one_handed_to_the_gpu_judge_and_the_cpu_checker
第10z批第十一块 崩溃放量 K5 交给 CPU 判器的叫停换成永不叫停（正推六·二 X2）	crates/singlefs-checker-tier/src/crash_amplification.rs	                    &version,\n                    &stop_is_requested,\n                )\n                .map_err(|error| format!("核对失败	                    &version,\n                    &|| false,\n                )\n                .map_err(|error| format!("核对失败	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_remembered_in_this_run_is_the_one_handed_to_the_gpu_judge_and_the_cpu_checker	the_stop_request_remembered_in_this_run_is_the_one_handed_to_the_gpu_judge_and_the_cpu_checker
第10z批第八块 显存账 K1-3 重起好了就当丢了的设备都已放掉（丢了的那一台还有克隆没丢）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        books_of_the_card.driver_share = if books_of_the_card.lost_devices_not_released.is_empty() {	        books_of_the_card.driver_share = if books_of_the_card.lost_devices_not_released.is_empty() || true {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement	a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
第10z批第八块 显存账 K1-3 丢了的设备都放掉了也不回到照读数分	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    self.driver_share = DriverShareOnTheCard::DeviceStarted;	                    self.driver_share =\n                        DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement	a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
第10z批第八块 显存账 K1-3 设备丢了、账照重起记时不把在用的那一台记成丢了还没放掉	crates/singlefs-harness/src/gpu_memory_ledger.rs	            self.lost_devices_not_released.insert(lost_device);	            let _ = lost_device;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement	a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement
第10z批第十一块 显存账 K4-2 拿到起设备的锁之后再看那一下丢了的也拿	crates/singlefs-harness/src/gpu_memory_ledger.rs	            if !is_lost(&device) {\n                return Ok(device);\n            }\n        }\n        let device = start()?;	            if is_lost(&device) || !is_lost(&device) {\n                return Ok(device);\n            }\n        }\n        let device = start()?;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced	a_device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced
第10z批第十一块 显存账 K4-2 拆锁之后拿到起设备的锁不再看一次留着的（等完照样另起一台）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        // 等这把锁的那一段里别的线程可能已经起好了：再看一次，起好了就用它，不再起\n        if let Some(device) = self.clone_of_the_device_kept() {\n            if !is_lost(&device) {\n                return Ok(device);\n            }\n        }\n        let device = start()?;	        let device = start()?;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_second_start_on_the_same_card_waits_for_the_restart_in_progress_and_takes_its_device	a_second_start_on_the_same_card_waits_for_the_restart_in_progress_and_takes_its_device
````

### 第十一块 mutations-replacements.tsv

````
第10z批第八块 显存账 K1-2 poll 交回队列空了也不销没等成的那几笔	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .return_the_listed_leases_kept_after_a_failed_wait_on(\n                self.device,\n                &listed.leases_kept_on_each_device,\n            );	            .leases_kept_after_a_failed_wait\n            .clear();	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_whose_wait_timed_out_leaves_the_books_when_a_poll_finds_its_device_idle	a_lease_whose_wait_timed_out_leaves_the_books_when_a_poll_finds_its_device_idle
第10y批 显存账 K1-3 卡上留着的设备丢了照旧拿它	crates/singlefs-harness/src/gpu_memory_ledger.rs	            if !is_lost(&device) {\n                return Ok(device);\n            }\n        }\n        // 持这把锁时	            if is_lost(&device) || !is_lost(&device) {\n                return Ok(device);\n            }\n        }\n        // 持这把锁时	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced	a_device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced
````

### 第十一块 mutations-delete.txt

````
第10z批第六块 显存账 G2 乙 重起好了就当旧设备已经不在（设备丢了那一刻活着的借据还在）
第10z批第六块 显存账 G2 乙 设备丢了那一刻活着的借据都还了也不回到照读数分
第10z批第六块 显存账 G2 乙 设备丢了、账照重起记时不记那一刻活着的借据
````

## 第十块

````diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1939,11 +1939,12 @@
     current_records: &[AllocationRecord],
     journal_instance_in_the_system_configuration: InstanceGeneration,
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
-) -> Result<BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>, MountError> {
+) -> Result<PlacementsInTheAccountsOfNewerReadableRoots, MountError> {
     let mut in_the_accounts_of_newer_roots_by_ring_slot: BTreeMap<
         RootRingSlot,
         Vec<PlacementOnDevice>,
     > = BTreeMap::new();
+    let mut txgs_of_the_roots_taking_the_current_account = Vec::new();
     for root in roots.iter().filter(|root| {
         !is_abandoned(root)
             && (root.checkpoint_txg, root.instance)
@@ -2005,7 +2006,27 @@
                         // 也就是现行账里这一截。它们这时分配着、影子账照豁免（第九项不数两遍）；这次挂载里释放之后，抬 F 借 C 那一槽
                         // （写行之后这条根被抛弃，H2）与进程内回收（`allocator::PoolAllocator::record_root_written_by_this_process`）照这一份补隔离。
                         // 不并它时，抬 F 只借到记录点名的那几个，它从所选那一版继承、被会话释放的单元回收出去（攻方 K2-2，I-7.4 红）。
-                        placements_still_allocated_in_the_account(current_records)
+                        // 这一句的前提有两处不成立（判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 两行，
+                        // 用户 2026-10-08 定「丙继续补洞」；两处补法的形是第 10z 批第十块实现员定的，被攻过零轮）：
+                        // 一、这条根与所选那一版之间还有别的发布（攻方一形）：它从那几次继承的单元只记在 C 里那几次的槽上，
+                        //    调用方照交回的 txg 把那几槽并进这条根那一槽（[`with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`]）；
+                        // 二、这条根是一次管理员回退（正推一形）：它复活的单元在现行账里是已释放，这里另并上
+                        //    [`placements_a_rollback_publish_revived_released_in_the_current_account`]。
+                        txgs_of_the_roots_taking_the_current_account.push(root.checkpoint_txg);
+                        let mut placements =
+                            placements_still_allocated_in_the_account(current_records);
+                        placements.extend(
+                            placements_a_rollback_publish_revived_released_in_the_current_account(
+                                devices,
+                                unit_area_start,
+                                root,
+                                &readable_records_of_its_publish,
+                                roots,
+                                is_abandoned,
+                                current_records,
+                            ),
+                        );
+                        placements
                     }
                 }
             };
@@ -2018,7 +2039,135 @@
                     .map(|(device, slot, span)| PlacementOnDevice { device, slot, span }),
             );
     }
-    Ok(in_the_accounts_of_newer_roots_by_ring_slot)
+    Ok(PlacementsInTheAccountsOfNewerReadableRoots {
+        by_ring_slot: in_the_accounts_of_newer_roots_by_ring_slot,
+        txgs_of_the_roots_taking_the_current_account,
+    })
+}
+
+/// [`placements_in_the_accounts_of_newer_readable_roots`] 交回的两样。
+struct PlacementsInTheAccountsOfNewerReadableRoots {
+    /// 较新可读根的账里的落点，按那条根的 txg 在根环里对应的槽（[`target_for_publish`]）归组。
+    by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>,
+    /// 账读不出而记录齐、那一槽改交现行账里仍分配着的落点（丙）的那几条根的 txg，按根环里读出的次序：调用方把所选那一版与它之间
+    /// 那几次发布在 C 里记着的落点并进它那一槽（[`with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`]）。
+    txgs_of_the_roots_taking_the_current_account: Vec<CheckpointTxg>,
+}
+
+/// 丙并中间（攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-output.md` 第一节改法表第一行 `chain`，判决
+/// `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 攻方一形；第 10z 批第十块实现员定形，被攻过零轮）：
+/// 走了丙那一支的根 R（`txgs_of_the_roots_taking_the_current_account`）与所选那一版之间还有别的发布 Q 时，R 从 Q 继承的单元既不是 R 写的、
+/// 也不在所选那一版的账里，只记在 C 里 Q 那一槽；这个进程写掉 Q 那一槽时那一份解开，R 那一槽要晚几个 txg 才写掉，其间它们发得出去。
+/// 这里把 (所选那一版, R) 之间每个 txg 在根环里对应的槽在 C 里记着的落点（`named_by_ring_slot`，C 的三份并好之后那一份）并进 R 那一槽，
+/// 已在 R 那一槽里的（盘、起点槽、跨度都同）不再并：它们就随 R 那一槽一起解开。中间的 txg 至多取一整圈（环长 3S 个），
+/// 两条根相差超过一圈时环里每一槽都已取到。只算内存，不读盘。
+fn with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
+    named_by_ring_slot: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>,
+    selected_version_txg: CheckpointTxg,
+    txgs_of_the_roots_taking_the_current_account: &[CheckpointTxg],
+    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
+) -> BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> {
+    let ring_length_in_txgs =
+        usize::try_from(ROOT_RING_REGIONS * slots_per_region.count()).expect("环长 3S ≤ 48");
+    let mut chained = named_by_ring_slot.clone();
+    for newer_root_txg in txgs_of_the_roots_taking_the_current_account {
+        let ring_slot_of_the_newer_root = target_for_publish(*newer_root_txg, slots_per_region);
+        let ring_slots_in_between: BTreeSet<RootRingSlot> = (selected_version_txg.0 + 1
+            ..newer_root_txg.0)
+            .rev()
+            .take(ring_length_in_txgs)
+            .map(|txg_in_between| {
+                target_for_publish(CheckpointTxg(txg_in_between), slots_per_region)
+            })
+            .filter(|ring_slot| *ring_slot != ring_slot_of_the_newer_root)
+            .collect();
+        let on_the_ring_slot_of_the_newer_root =
+            chained.entry(ring_slot_of_the_newer_root).or_default();
+        let mut already_there: BTreeSet<(DeviceIdentity, SlotNumber, u64)> =
+            on_the_ring_slot_of_the_newer_root
+                .iter()
+                .map(|placement| (placement.device, placement.slot, placement.span))
+                .collect();
+        for placement in ring_slots_in_between
+            .iter()
+            .filter_map(|ring_slot| named_by_ring_slot.get(ring_slot))
+            .flatten()
+        {
+            if already_there.insert((placement.device, placement.slot, placement.span)) {
+                on_the_ring_slot_of_the_newer_root.push(*placement);
+            }
+        }
+    }
+    chained
+}
+
+/// 回退复活（判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 正推一形，用户 2026-10-08 定「丙继续补洞」；
+/// 第 10z 批第十块实现员定形，被攻过零轮）：走了丙那一支的根 `newer_root` 是一次管理员回退（向前发布）时，它从回退目标 R_old 取用户可见树，
+/// R_old 引用、现行账记成已释放的单元被它复活；回退发布不写它们，记录不点名，现行账里又是已释放，丙那一截滤掉了它们。
+/// 认回退：那次发布读得出的记录点名的打包单元（类标签 3）里，读得出、校验和对得上、打包记录类型是谱系（5）的那一片里有一条
+/// rollback_txg 等于这条根 txg 的谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条：回退那次发布追加一条 (T_rb, T_old)），
+/// T_old 就是回退目标的 txg。根环里读出、按最新那条根的实例表有效、txg 等于 T_old 的根就是 R_old，读它的账（影子账同一份读法
+/// [`placements_referenced_by_root`]）。交回 R_old 账里仍分配着、现行账里记成已释放的落点（按盘与起点槽对上）：只护复活的那一截，
+/// 不并现行账里别的已释放落点（那是攻方乙那一类，第四块判决第二节 K2-2 改法行：384 槽盘上隔离约 370 个槽、600 段里 588 段被拒）。
+/// 认不出回退（记录没点名谱系片、那一片读不出、里面没有这条根的谱系记录）、根环里没读出 R_old、R_old 的账读不出，都交空：
+/// 那几形照丙原样，少护的那一截交主 agent（第十块报告「停下交主 agent」）。只读盘，不改分配器。
+#[allow(
+    clippy::ptr_arg,
+    reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
+)]
+fn placements_a_rollback_publish_revived_released_in_the_current_account<Device: BlockDevice>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    unit_area_start: UnitAreaStart,
+    newer_root: &RootRecord,
+    readable_records_of_its_publish: &[&crate::journal::JournalRecord],
+    roots: &[RootRecord],
+    is_abandoned: &dyn Fn(&RootRecord) -> bool,
+    current_records: &[AllocationRecord],
+) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
+    let Some(rollback_target_txg) = rollback_target_txg_named_by_the_records_of_a_publish(
+        devices,
+        newer_root.checkpoint_txg,
+        readable_records_of_its_publish,
+    ) else {
+        return Vec::new();
+    };
+    let released_in_the_current_account: BTreeSet<(DeviceIdentity, SlotNumber)> = current_records
+        .iter()
+        .filter(|record| record.is_released)
+        .map(|record| (record.device, record.slot))
+        .collect();
+    roots
+        .iter()
+        .filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)
+        .filter_map(|rollback_target| {
+            placements_referenced_by_root(devices, unit_area_start, rollback_target)
+        })
+        .flatten()
+        .filter(|(device, slot, _)| released_in_the_current_account.contains(&(*device, *slot)))
+        .collect()
+}
+
+/// 一次发布读得出的记录点名的打包单元（类标签 3）里，读得出、校验和对得上、打包记录类型是谱系（5）的每一片里，rollback_txg 等于
+/// `checkpoint_txg_of_the_publish` 的那条谱系记录的 T_old：那次发布是一次管理员回退，回到 T_old 那一版。认不出交 `None`。
+fn rollback_target_txg_named_by_the_records_of_a_publish<Device: BlockDevice>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    checkpoint_txg_of_the_publish: CheckpointTxg,
+    readable_records_of_the_publish: &[&crate::journal::JournalRecord],
+) -> Option<CheckpointTxg> {
+    let packed_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
+    readable_records_of_the_publish
+        .iter()
+        .flat_map(|record| record.named.iter())
+        .filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)
+        .filter_map(|named| {
+            read_unit_via_locations(devices, &named.locations, packed_unit_bytes).ok()
+        })
+        .filter_map(|bytes| crate::unit::parse_packed_unit(&bytes).ok())
+        .filter(|packed| packed.identity.record_type == crate::unit::PACKED_TYPE_LINEAGE)
+        .flat_map(|packed| packed.records)
+        .filter_map(|record_bytes| LineageRecord::parse(&record_bytes))
+        .find(|lineage| lineage.rollback_txg == checkpoint_txg_of_the_publish)
+        .map(|lineage| lineage.rollback_target_txg)
 }
 
 /// 一份账里仍分配着的每个落点（盘、起点槽、跨度），按记录的次序。只读内存。
@@ -2754,26 +2903,36 @@
             }
             // 再并上环里比所选那一版新、按实例表有效、读得出的根自己的账（`C_NEWER_ROOTS`，[`placements_in_the_accounts_of_newer_readable_roots`]）：
             // F2 退回之后被退掉的那条根，影子账不管它，它的记录又读不出时前两份都是空的。同一个时机解开。
+            let PlacementsInTheAccountsOfNewerReadableRoots {
+                by_ring_slot: in_the_accounts_of_newer_roots_by_ring_slot,
+                txgs_of_the_roots_taking_the_current_account,
+            } = placements_in_the_accounts_of_newer_readable_roots(
+                devices,
+                unit_area_start,
+                &roots,
+                &is_abandoned,
+                records,
+                selected_version,
+                slots_per_region,
+                &current_records,
+                system_configuration.quantities.journal_instance,
+                before_the_one_reread,
+            )?;
             for (ring_slot, placements_in_the_newer_roots_account) in
-                placements_in_the_accounts_of_newer_readable_roots(
-                    devices,
-                    unit_area_start,
-                    &roots,
-                    &is_abandoned,
-                    records,
-                    selected_version,
-                    slots_per_region,
-                    &current_records,
-                    system_configuration.quantities.journal_instance,
-                    before_the_one_reread,
-                )?
+                in_the_accounts_of_newer_roots_by_ring_slot
             {
                 named_by_ring_slot
                     .entry(ring_slot)
                     .or_default()
                     .extend(placements_in_the_newer_roots_account);
             }
-            named_by_ring_slot
+            // 丙并中间：三份都并好之后，走了丙那一支的根那一槽再并上它与所选那一版之间那几槽（函数的注释）。
+            with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
+                named_by_ring_slot,
+                selected_version.checkpoint_txg,
+                &txgs_of_the_roots_taking_the_current_account,
+                slots_per_region,
+            )
         }
         ShadowLedger::Off => BTreeMap::new(),
     };
@@ -8370,3 +8529,80 @@
         );
     }
 }
+
+#[cfg(test)]
+mod placements_of_the_publishes_in_between_chained_onto_the_newer_roots_tests {
+    use super::with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots;
+    use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+    use crate::allocator::PlacementOnDevice;
+    use crate::root_ring::{target_for_publish, RootRingSlotsPerRegion};
+    use std::collections::BTreeMap;
+
+    fn placement(slot: u64) -> PlacementOnDevice {
+        PlacementOnDevice {
+            device: DeviceIdentity(0),
+            slot: SlotNumber(slot),
+            span: 2,
+        }
+    }
+
+    /// 所选那一版 txg 6，走了丙那一支的根 R txg 9，其间 Q（txg 7）、txg 8 两槽在 C 里各记着落点，R 之后的 txg 10 那一槽也记着：
+    /// R 那一槽并上 txg 7、8 两槽记着的、R 那一槽里还没有的落点（R 那一槽已有的不再并一次），txg 10 那一槽的不并；别的槽原样。
+    #[test]
+    fn the_ring_slot_of_a_root_taking_the_current_account_takes_what_the_publishes_in_between_named(
+    ) {
+        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
+        let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
+        let named_by_ring_slot = BTreeMap::from([
+            (ring_slot_of(7), vec![placement(100), placement(102)]),
+            (ring_slot_of(8), vec![placement(104), placement(200)]),
+            (ring_slot_of(9), vec![placement(200)]),
+            (ring_slot_of(10), vec![placement(300)]),
+        ]);
+        let chained = with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
+            named_by_ring_slot.clone(),
+            CheckpointTxg(6),
+            &[CheckpointTxg(9)],
+            slots_per_region,
+        );
+        assert_eq!(
+            chained[&ring_slot_of(9)],
+            vec![
+                placement(200),
+                placement(100),
+                placement(102),
+                placement(104)
+            ],
+            "R 那一槽：原有的在前，再按槽的次序并上 txg 7、8 两槽的，已有的 200 不再并"
+        );
+        for txg in [7, 8, 10] {
+            assert_eq!(
+                chained[&ring_slot_of(txg)],
+                named_by_ring_slot[&ring_slot_of(txg)],
+                "txg {txg} 那一槽原样（中间那几槽照旧在自己的 txg 上解开）"
+            );
+        }
+        assert_eq!(chained.len(), named_by_ring_slot.len(), "不新开别的槽");
+    }
+
+    /// 走了丙那一支的根紧挨着所选那一版（其间没有别的发布）：什么都不并，原样交回。
+    #[test]
+    fn a_root_taking_the_current_account_right_after_the_selected_version_takes_nothing_more() {
+        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
+        let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
+        let named_by_ring_slot = BTreeMap::from([
+            (ring_slot_of(6), vec![placement(100)]),
+            (ring_slot_of(7), vec![placement(200)]),
+        ]);
+        assert_eq!(
+            with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
+                named_by_ring_slot.clone(),
+                CheckpointTxg(6),
+                &[CheckpointTxg(7)],
+                slots_per_region,
+            ),
+            named_by_ring_slot,
+            "(6, 7) 之间没有 txg：原样"
+        );
+    }
+}
diff --git a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
--- a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -21,6 +21,11 @@
 //!   （与所选那一版 P 共用文件的数据单元），挂载那一刻 R 的账读回全零而记录齐，C 那一槽另并上那一刻现行账里仍分配着的落点；
 //!   会话覆盖写释放了那几个共用的数据单元、推抬 F、再顺序写，池级 checker 不红 I-7.4。只有 H2、没有丙时这一形红（攻方 K2-2）。
 //! - 公开的抬 F 上限（`rollback_floor_ceiling`）照抬 F 那一串的口径取拒集（正推 K1-1）：C 的根槽读坏不拒、不进上限。
+//! - 丙补洞两形（用户 2026-10-08 定「丙继续补洞」，判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 两行、第四节；
+//!   两处补法的形由第 10z 批第十块实现员定，被攻过零轮）：
+//!   丙并中间——R 与所选那一版 P 之间还有一次发布 Q（攻方一形），C 里 R 那一槽并上 Q 那一槽记着的落点，会话写到写掉 R 那一槽，
+//!   每一步根落之前那一态与每一步之后池级 checker 不红 I-7.4；
+//!   回退复活——R 是一次管理员回退（正推一形），C 里 R 那一槽并上回退目标账里仍分配、P 的账里已释放的落点，推过抬 F 复活的槽不空闲、不红 I-7.4。
 // 池（384 槽那一档小盘）与 R 那块盘都经 `tests/common_unwitnessed_publish/mod.rs` 起（它转出 `tests/common_admission_pool/mod.rs` 的全部）：
 // 再声明 `common_admission` 就把 `common_admission_pool` 当模块装两遍（clippy `duplicate_mod`）。
 mod common_unwitnessed_publish;
@@ -37,7 +42,9 @@
     DeviceOverASparseImage, DeviceReadingBackAllZeroOnRanges, PoolUnderTest, PublishRecordedAsR,
     RangesReadingBackAllZero, ReadBackAllZeroLift, RecordedPublish, OVERWRITE_BYTES,
 };
-use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
+};
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::allocator::{AllocationRecord, PlacementOnDevice, PoolAllocator};
 use singlefs_core::block_device::{
@@ -47,8 +54,8 @@
 use singlefs_core::mount::{
     mount_writable, mount_writable_with_space_admission, mount_writable_with_test_only_switches,
     ring_slots_known_to_hold_a_root_by, ring_slots_whose_reread_still_bad_refuses_the_ceiling_by,
-    rollback_floor_ceiling, unmount, MountError, RollbackTarget, ShadowLedger,
-    StillUnreadableAfterOneReread, Unmounted,
+    roll_back_by_a_forward_publish, rollback_floor_ceiling, unmount, MountError, RollbackTarget,
+    ShadowLedger, StillUnreadableAfterOneReread, Unmounted,
 };
 use singlefs_core::mounted_session::MountedSession;
 use singlefs_core::pointer::{LocationEntry, NodePointer};
@@ -1390,3 +1397,521 @@
         AccountOfRAfterTheMount::StillReadingBackAllZero,
     );
 }
+
+/// 丙并中间那一形之前覆盖写几次（攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-output.md` 第一节原样那一格 k = 3）。
+const OVERWRITES_BEFORE_THE_PUBLISHES_RECORDED_AS_Q_AND_R: u64 = 3;
+
+/// 丙并中间那一形挂上之后，会话至多覆盖写几次去等写掉 R 那一槽（384 槽小盘一圈根环 24 次发布；攻方原样那一格第 20 步写掉）。
+const OVERWRITES_UNTIL_THE_RING_SLOT_OF_R_IS_WRITTEN_AT_MOST: usize = 40;
+
+/// 录下来的 Q、R 两次发布：Q 之前那一刻（所选那一版 P）的盘面、Q 与 R 两次发布的全部写（按落盘次序）、两条根槽写在写表里的下标、
+/// P、Q、R 的 (实例, txg)，加起池那一份（会话停在 R 那一版）。
+struct RecordedQAndR {
+    pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
+    base: MemoryPool,
+    writes: Vec<RetainedWrite>,
+    root_index_of_q: usize,
+    root_index_of_r: usize,
+    key_of_p: (InstanceGeneration, CheckpointTxg),
+    key_of_q: (InstanceGeneration, CheckpointTxg),
+    key_of_r: (InstanceGeneration, CheckpointTxg),
+}
+
+/// 攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-model/src/opus10zghi-probe/tests/opus_k21_chain.rs` 的 `record_chain`（q = `s`、r = `n`）：
+/// 起池（384 槽那一档、第一个文件），覆盖写 `overwrites_before` 次得 P，再录 Q（顺序写三个数据单元，三条记录）与 R（建一个 inode，
+/// 文件的数据单元与 Q 共用）。
+fn record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites(
+    overwrites_before: u64,
+) -> RecordedQAndR {
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
+    let key_of = |pool_being_recorded: &PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>| {
+        let root = pool_being_recorded.session().current.root();
+        (root.instance, root.checkpoint_txg)
+    };
+    let key_of_p = key_of(&pool);
+    let base = pool.image();
+    let operations_before_q = stream.operation_count();
+    pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity() + 1000)
+        .expect("录的那次顺序写（Q）落得下");
+    let key_of_q = key_of(&pool);
+    pool.create_one_inode()
+        .expect("录的那次建 inode（R）落得下");
+    let key_of_r = key_of(&pool);
+    let operations = stream.retained_operations();
+    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[operations_before_q..],
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
+        2,
+        "录的 Q、R 两次发布各写一条根（准入够，没推抬 F）"
+    );
+    RecordedQAndR {
+        pool,
+        base,
+        writes,
+        root_index_of_q: root_indices[0],
+        root_index_of_r: root_indices[1],
+        key_of_p,
+        key_of_q,
+        key_of_r,
+    }
+}
+
+impl RecordedQAndR {
+    /// 崩在 R 的根槽落盘之后、R 那次系统配置轮换之前（Q 那次轮换已持久）。
+    fn crash_image_with_the_root_of_r_persisted_and_its_rotation_lost(&self) -> MemoryPool {
+        let mut image = self.base.clone();
+        image.apply_writes(&self.writes[..=self.root_index_of_r]);
+        image
+    }
+
+    fn root_of_r(&self) -> RootRecord {
+        RootRecord::parse_slot(
+            self.writes[self.root_index_of_r]
+                .bytes()
+                .expect("根槽写带字节"),
+            &self.pool.parameters.filesystem_identifier,
+        )
+        .expect("R 的根自证")
+    }
+
+    /// Q 写出的单元写（每块盘一份）。
+    fn unit_writes_of_q(&self) -> Vec<&RetainedWrite> {
+        self.writes[..self.root_index_of_q]
+            .iter()
+            .filter(|write| write.kind == StepKind::UnitWrite)
+            .collect()
+    }
+
+    /// 可写挂载那一刻读回全零的几段（攻方故障组 `cfgq,qroot,qlast,racct`）：每块盘上 Q 那次轮换写的系统配置槽、Q 的根槽、
+    /// Q 带末条标志的那条记录、R 写的树表 / 中央映射树根 / 分配记录树根那几份。
+    fn ranges_reading_back_all_zero_at_the_mount(
+        &self,
+        crash_image: &MemoryPool,
+    ) -> RangesReadingBackAllZero {
+        let root_of_r = self.root_of_r();
+        let account_pointers_of_r = [
+            root_of_r.tree_table,
+            root_of_r.mapping_root,
+            root_of_r.allocation_record_tree_root,
+        ];
+        let writes_in_between_the_two_roots =
+            &self.writes[self.root_index_of_q..self.root_index_of_r];
+        let rotation_of_q: Vec<&RetainedWrite> = writes_in_between_the_two_roots
+            .iter()
+            .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
+            .collect();
+        assert!(!rotation_of_q.is_empty(), "Q 那次轮换写了系统配置槽");
+        let account_of_r: Vec<&RetainedWrite> = writes_in_between_the_two_roots
+            .iter()
+            .filter(|write| write.kind == StepKind::UnitWrite)
+            .filter(|write| {
+                account_pointers_of_r.iter().any(|pointer| {
+                    *pointer != NodePointer::empty_root()
+                        && pointer.locations.iter().any(|location| {
+                            location.device == write.device
+                                && location.slot.to_device_offset() == write.offset
+                        })
+                })
+            })
+            .collect();
+        assert!(!account_of_r.is_empty(), "R 自己写了它的账那几个单元");
+        RangesReadingBackAllZero::default()
+            .with_the_writes(&rotation_of_q)
+            .with_the_writes(&[&self.writes[self.root_index_of_q]])
+            .with_the_last_record_of_the_publish(crash_image, self.key_of_q)
+            .with_the_writes(&account_of_r)
+    }
+}
+
+/// `placements` 里有没有一个落点罩住这次单元写的起点（同一块盘、起点槽到跨度之内）。
+fn some_placement_covers(placements: &[PlacementOnDevice], write: &RetainedWrite) -> bool {
+    placements.iter().any(|placement| {
+        let start_in_bytes = placement.slot.to_device_offset().0;
+        placement.device == write.device
+            && start_in_bytes <= write.offset.0
+            && write.offset.0 < start_in_bytes + placement.span * singlefs_format::SLOT_BYTES
+    })
+}
+
+/// 丙并中间（攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-output.md` 第一节，原样那一格 k = 3、Q 顺序写、R 建 inode、
+/// 故障 `cfgq,qroot,qlast,racct`、动作全是覆盖写；判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 攻方一形）：
+/// 第一个文件之后覆盖写 3 次得 P，录 Q（顺序写）与 R（建 inode，数据单元与 Q 共用），崩在 R 的根槽落盘之后、R 那次轮换之前；
+/// 可写挂载那一刻 Q 轮换写的系统配置槽、Q 的根槽、Q 的末条记录、R 的账读回全零：择根退到 P，R 的记录齐、账读不出，走丙。
+/// C 里 R 那一槽并上 (P, R) 之间 Q 那一槽记着的落点，Q 写的单元全在里面。挂上之后撤故障，会话覆盖写到写掉 R 那一槽：
+/// 每一步之后、每一步最后一条根槽写之前那个段边界状态（单元与记录都落了、这一步的根没落：攻方红的那一态）池级 checker 都不红
+/// I-7.4（近 K 代块未被复用）与 I-4.8；其间有一步 Q 已离开根环而 R 还在（这一形的窗口确实走到了）。
+/// 判别力：撤掉丙并中间（变异「第10z批第十块 丙并中间 走丙那一支的根那一槽不并所选那一版与它之间那几槽」）时，写掉 Q 那一槽之后、
+/// 写掉 R 那一槽那一步根落之前那一态 I-7.4 红，违例的根是 R；挂载那一刻 C 那一槽缺 Q 的单元那一句先红。
+// harness-test-granularity:one-scenario 一个池、一次挂载，循环是同一个会话里接着覆盖写，不是每轮新建一个池
+#[test]
+fn units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written(
+) {
+    let recorded = record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites(
+        OVERWRITES_BEFORE_THE_PUBLISHES_RECORDED_AS_Q_AND_R,
+    );
+    let crash_image = recorded.crash_image_with_the_root_of_r_persisted_and_its_rotation_lost();
+    let ranges = recorded.ranges_reading_back_all_zero_at_the_mount(&crash_image);
+    let mut faulted_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
+    let mounted = mount_writable_with_space_admission(
+        &recorded.pool.parameters,
+        &mut faulted_devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("Q 的根与末条记录、R 的账读回全零：可写挂载退到 P、做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.key_of_p,
+        "R 走读不成、Q 的根读不出：择根退到 P（阳性对照：故障打到了）"
+    );
+    let borrowed_for_r = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+            recorded.key_of_r.0,
+            recorded.key_of_r.1,
+        )
+        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
+    let units_of_q_not_in_the_ring_slot_of_r: Vec<&RetainedWrite> = recorded
+        .unit_writes_of_q()
+        .into_iter()
+        .filter(|write| !some_placement_covers(&borrowed_for_r, write))
+        .collect();
+    assert!(
+        units_of_q_not_in_the_ring_slot_of_r.is_empty(),
+        "挂载那一刻 C 里 R 那一槽并上了 Q 那一槽记着的落点（丙并中间）：Q 写的单元都在里面，缺 {} 份：{units_of_q_not_in_the_ring_slot_of_r:?}",
+        units_of_q_not_in_the_ring_slot_of_r.len()
+    );
+    let image_after_the_mount =
+        image_of_devices_reading_back_all_zero(&faulted_devices, recorded.pool.device_bytes);
+    let (output, session) = MountedSession::of_the_mount(mounted);
+    let content_of_the_selected_version = recorded
+        .pool
+        .content_of_each_root
+        .get(&recorded.key_of_p)
+        .cloned()
+        .expect("P 是录的池写过的一版");
+    let mut content_of_each_root = recorded.pool.content_of_each_root.clone();
+    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
+        let root = version.root();
+        content_of_each_root.insert(
+            (root.instance, root.checkpoint_txg),
+            content_of_the_selected_version.clone(),
+        );
+    }
+    let session_stream = SharedStream::retaining_contents();
+    let mut pool = PoolUnderTest {
+        parameters: recorded.pool.parameters.clone(),
+        device_bytes: recorded.pool.device_bytes,
+        devices: plain_devices_on(&image_after_the_mount)
+            .into_iter()
+            .map(|(identity, device)| {
+                (
+                    identity,
+                    RecordingBlockDevice::with_shared_stream(
+                        identity,
+                        device,
+                        session_stream.clone(),
+                    ),
+                )
+            })
+            .collect(),
+        session: Some(session),
+        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
+        content_of_the_current_version: content_of_the_selected_version,
+        content_of_each_root,
+    };
+    let mut q_left_the_ring_while_r_was_in_it = false;
+    let mut overwrites = 0;
+    // 迭代上界 OVERWRITES_UNTIL_THE_RING_SLOT_OF_R_IS_WRITTEN_AT_MOST；跨轮带着「Q 先离开根环、R 还在」那一个判断；
+    // 出口：R 离开根环的那一步判完就 break，被拒就当场判红（panic）。
+    loop {
+        assert!(
+            overwrites < OVERWRITES_UNTIL_THE_RING_SLOT_OF_R_IS_WRITTEN_AT_MOST,
+            "{OVERWRITES_UNTIL_THE_RING_SLOT_OF_R_IS_WRITTEN_AT_MOST} 次覆盖写之内写掉 R 那一槽（384 槽小盘一圈根环 24 次发布）"
+        );
+        let image_before_the_step = pool.image();
+        let operations_before_the_step = session_stream.operation_count();
+        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!("挂上之后第 {} 次覆盖写被拒：{refusal:?}", overwrites + 1)
+        });
+        overwrites += 1;
+        let (writes_of_the_step, _segments, _stream_indexes) =
+            writes_and_segments_with_stream_indexes(
+                &session_stream.retained_operations()[operations_before_the_step..],
+                &fixed_geometry_of_the_384_slot_small_devices(),
+            );
+        let last_root_of_the_step = writes_of_the_step
+            .iter()
+            .rposition(|write| write.kind == StepKind::RootRecordFua)
+            .expect("一次覆盖写至少写一条根");
+        let mut state_before_the_last_root_of_the_step = image_before_the_step;
+        state_before_the_last_root_of_the_step
+            .apply_writes(&writes_of_the_step[..last_root_of_the_step]);
+        let violations_before_the_last_root =
+            violations_of_unit_reuse_on(&state_before_the_last_root_of_the_step);
+        assert!(
+            violations_before_the_last_root.is_empty(),
+            "第 {overwrites} 次覆盖写最后一条根槽写之前那一态（R 在根环里：{}）：被抛弃根引用的单元一个都没复用：{violations_before_the_last_root:?}",
+            root_is_in_the_ring(&state_before_the_last_root_of_the_step, recorded.key_of_r)
+        );
+        let image_after_the_step = pool.image();
+        let violations_after_the_step = violations_of_unit_reuse_on(&image_after_the_step);
+        assert!(
+            violations_after_the_step.is_empty(),
+            "第 {overwrites} 次覆盖写之后：被抛弃根引用的单元一个都没复用：{violations_after_the_step:?}"
+        );
+        let r_is_in_the_ring = root_is_in_the_ring(&image_after_the_step, recorded.key_of_r);
+        if r_is_in_the_ring && !root_is_in_the_ring(&image_after_the_step, recorded.key_of_q) {
+            q_left_the_ring_while_r_was_in_it = true;
+        }
+        if !r_is_in_the_ring {
+            break;
+        }
+    }
+    assert!(
+        q_left_the_ring_while_r_was_in_it,
+        "有一步之后 Q 已离开根环而 R 还在（这一形的窗口走到了：Q 那一槽先写掉、解开，R 那一槽晚一个 txg）"
+    );
+}
+
+/// 正推 `research/prompts/m3-batch10z-ghi-code-r1-sonnet-model/probe_rollback_revived.tail.rs` 那一形之前覆盖写几次（探针原样 4 次）。
+const OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R: u64 = 4;
+
+/// 回退复活那一形推过第一串抬 F 之后再顺序写几次（每次两个数据单元；探针原样 3 次）。
+const SEQUENTIAL_WRITES_AFTER_THE_FLOOR_RAISE_OVER_THE_ROLLBACK: usize = 3;
+
+/// 正推探针的 `record_one_rollback_after_overwrites`：起池（384 槽那一档、第一个文件，那一版是 R_old），覆盖写 `overwrites_before` 次得 P，
+/// 再经会话做一次管理员回退到 R_old、录这一次发布得 R。交回录下来的 R 与 R 复活的单元的落点：R_old 的文件数据指针里 P 不引用的那几条位置条目
+/// （从录下来的会话算，不经被测的方法；跨度按数据单元宽折成槽数）。
+fn record_one_rollback_to_the_first_file_after_overwrites(
+    overwrites_before: u64,
+) -> (RecordedPublish, Vec<PlacementOnDevice>) {
+    let stream = SharedStream::retaining_contents();
+    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
+        parameters_of_the_384_slot_small_devices(),
+        device_bytes_of_the_384_slot_small_devices(),
+        |identity, device| {
+            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
+        },
+    );
+    let root_of_the_first_file = *pool.session().current.root();
+    let data_pointers_of_the_first_file = pool.current_file_version().data_pointers.clone();
+    for _overwrite in 0..overwrites_before {
+        pool.overwrite(OVERWRITE_BYTES)
+            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
+    }
+    let previous_root = *pool.session().current.root();
+    let previous_content = pool.content_of_the_current_version.clone();
+    let locations_referenced_by_p: Vec<(DeviceIdentity, SlotNumber)> = pool
+        .current_file_version()
+        .data_pointers
+        .iter()
+        .flat_map(|pointer| pointer.locations)
+        .map(|location| (location.device, location.slot))
+        .collect();
+    let base = pool.image();
+    let operations_before_r = stream.operation_count();
+    let mut session = pool.session.take().expect("会话");
+    {
+        let PoolVersion::WithFile(current) = &mut session.current else {
+            panic!("第一个文件之后现行那一版带文件");
+        };
+        roll_back_by_a_forward_publish(
+            &pool.parameters,
+            &mut pool.devices,
+            &mut session.allocator,
+            current,
+            RollbackTarget {
+                instance: root_of_the_first_file.instance,
+                checkpoint_txg: root_of_the_first_file.checkpoint_txg,
+            },
+        )
+        .expect("回退到第一个文件那一版做得成（它在根环里、按实例表有效）");
+    }
+    pool.session = Some(session);
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
+    assert_eq!(root_indices.len(), 1, "回退那次发布只写一条根");
+    let revived: Vec<PlacementOnDevice> = data_pointers_of_the_first_file
+        .iter()
+        .flat_map(|pointer| pointer.locations)
+        .filter(|location| !locations_referenced_by_p.contains(&(location.device, location.slot)))
+        .map(|location| PlacementOnDevice {
+            device: location.device,
+            slot: location.slot,
+            span: singlefs_format::DATA_UNIT_BYTES.div_ceil(singlefs_format::SLOT_BYTES),
+        })
+        .collect();
+    (
+        RecordedPublish {
+            content: previous_content.clone(),
+            pool,
+            base,
+            writes,
+            root_index: root_indices[0],
+            key: (root.instance, root.checkpoint_txg),
+            previous_key: (previous_root.instance, previous_root.checkpoint_txg),
+            previous_content,
+        },
+        revived,
+    )
+}
+
+/// 回退复活（正推 `research/prompts/m3-batch10z-ghi-code-r1-sonnet-output.md` K2-1、探针 `probe_rollback_revived.tail.rs`；判决
+/// `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 正推一形）：第一个文件之后覆盖写 4 次得 P，经会话管理员回退到
+/// 第一个文件那一版（R_old）、录这一次发布得 R，崩在 R 的根槽落盘之后、轮换之前；可写挂载那一刻 R 的账（树表、中央映射树根、分配记录树根）
+/// 读回全零而记录齐：择根退到 P，走丙。R 复活的数据单元在 P 的账里是已释放，丙那一截滤掉它们；补法认出 R 是回退（记录点名的谱系片里有
+/// rollback_txg 等于 R 的那条谱系记录），把 R_old 账里仍分配、P 的账里已释放的落点并进 C 里 R 那一槽：复活的落点都在里面。
+/// 挂上之后撤故障，会话覆盖写到准入推出第一串抬 F（推的那一刻 R 还在根环里），推过那一刻复活的落点一槽都不空闲；再顺序写 3 次，
+/// 每一步之后池级 checker 不红 I-7.4 与 I-4.8。判别力：撤掉回退复活那一截（变异「第10z批第十块 回退复活 丙那一支不并回退目标账里仍分配、
+/// 现行账里已释放的落点」）时 C 那一槽缺复活的落点那一句先红；去掉那一句，推过抬 F 复活的槽空闲、顺序写之后 I-7.4 红（正推探针原样）。
+#[test]
+fn units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount(
+) {
+    let (recorded, revived) = record_one_rollback_to_the_first_file_after_overwrites(
+        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
+    );
+    assert!(
+        !revived.is_empty(),
+        "第一个文件那一版的数据单元 P 不引用：回退复活了它们（阳性对照：这一形走到了复活）"
+    );
+    let root_of_r = recorded.root();
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
+        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
+    );
+    let borrowed_for_r = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+            root_of_r.instance,
+            root_of_r.checkpoint_txg,
+        )
+        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
+    let revived_not_borrowed: Vec<&PlacementOnDevice> = revived
+        .iter()
+        .filter(|revived_placement| {
+            !borrowed_for_r.iter().any(|borrowed| {
+                borrowed.device == revived_placement.device
+                    && borrowed.slot == revived_placement.slot
+            })
+        })
+        .collect();
+    assert!(
+        revived_not_borrowed.is_empty(),
+        "挂载那一刻 C 里 R 那一槽并上了回退目标账里仍分配、P 的账里已释放的落点（回退复活）：复活的都在里面，缺 {revived_not_borrowed:?}"
+    );
+    lift.lift();
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
+        let violations = violations_of_unit_reuse_on(&pool_after_the_step.image());
+        assert!(
+            violations.is_empty(),
+            "{after_what}：被抛弃根引用的单元一个都没复用：{violations:?}"
+        );
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
+                "第 {} 次覆盖写被拒：{refusal:?}",
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
+    let free_among_the_revived_after_the_raise =
+        free_slots_covered_by(&pool.session().allocator, &revived);
+    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_THE_FLOOR_RAISE_OVER_THE_ROLLBACK {
+        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
+            .unwrap_or_else(|refusal| {
+                panic!("推过抬 F 之后第 {sequential_write_number} 次顺序写被拒：{refusal:?}")
+            });
+        assert_nothing_of_r_reused(&pool, "推过抬 F 之后顺序写");
+    }
+    assert!(
+        free_among_the_revived_after_the_raise.is_empty(),
+        "推过抬 F 那一刻 R 复活的数据单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_revived_after_the_raise:?}"
+    );
+}
````

### 第十块 mutations-append.tsv

````
第10z批第十块 丙并中间 走丙那一支的根那一槽不并所选那一版与它之间那几槽（攻方 K2-1：写掉 Q 那一槽之后 Q 的单元发得出去）	crates/singlefs-core/src/mount.rs	    for newer_root_txg in txgs_of_the_roots_taking_the_current_account {	    for newer_root_txg in txgs_of_the_roots_taking_the_current_account.iter().take(0) {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written
第10z批第十块 丙并中间 走丙那一支的根那一槽不并所选那一版与它之间那几槽（库单测：R 那一槽缺 txg 7、8 两槽的落点）	crates/singlefs-core/src/mount.rs	    for newer_root_txg in txgs_of_the_roots_taking_the_current_account {	    for newer_root_txg in txgs_of_the_roots_taking_the_current_account.iter().take(0) {	-p singlefs-core --lib -- placements_of_the_publishes_in_between_chained_onto_the_newer_roots_tests	the_ring_slot_of_a_root_taking_the_current_account_takes_what_the_publishes_in_between_named
第10z批第十块 丙并中间 中间那几槽只取紧挨着所选那一版之后的第二个 txg 起（Q 紧挨着 P 时 Q 那一槽并不进来）	crates/singlefs-core/src/mount.rs	(selected_version_txg.0 + 1\n            ..newer_root_txg.0)	(selected_version_txg.0 + 2\n            ..newer_root_txg.0)	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written
第10z批第十块 丙并中间 中间那几槽只取紧挨着 R 的那一个（库单测：txg 7 那一槽并不进来）	crates/singlefs-core/src/mount.rs	            .take(ring_length_in_txgs)	            .take(1)	-p singlefs-core --lib -- placements_of_the_publishes_in_between_chained_onto_the_newer_roots_tests	the_ring_slot_of_a_root_taking_the_current_account_takes_what_the_publishes_in_between_named
第10z批第十块 丙并中间 R 那一槽已有的落点也再并一次（库单测：200 重复）	crates/singlefs-core/src/mount.rs	            if already_there.insert((placement.device, placement.slot, placement.span)) {	            if already_there.insert((placement.device, placement.slot, placement.span)) || true {	-p singlefs-core --lib -- placements_of_the_publishes_in_between_chained_onto_the_newer_roots_tests	the_ring_slot_of_a_root_taking_the_current_account_takes_what_the_publishes_in_between_named
第10z批第十块 回退复活 认回退时谱系记录按 txg 0 找（认不出回退：丙那一支不并回退目标账里仍分配、现行账里已释放的落点）	crates/singlefs-core/src/mount.rs	        .find(|lineage| lineage.rollback_txg == checkpoint_txg_of_the_publish)	        .find(|lineage| lineage.rollback_txg == CheckpointTxg(0))	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount
第10z批第十块 回退复活 回退目标账里取现行账里仍分配的、不取已释放的（复活的那几个滤掉）	crates/singlefs-core/src/mount.rs	        .filter(|(device, slot, _)| released_in_the_current_account.contains(&(*device, *slot)))	        .filter(|(device, slot, _)| !released_in_the_current_account.contains(&(*device, *slot)))	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount
第10z批第十块 回退复活 打包单元只看非谱系的那几片（谱系片里那条记录看不到）	crates/singlefs-core/src/mount.rs	        .filter(|packed| packed.identity.record_type == crate::unit::PACKED_TYPE_LINEAGE)	        .filter(|packed| packed.identity.record_type != crate::unit::PACKED_TYPE_LINEAGE)	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount
第10z批第十块 回退复活 回退目标按被抛弃的根找（R_old 按实例表有效，找不到）	crates/singlefs-core/src/mount.rs	        .filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)	        .filter(|root| is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount
第10z批第十块 丙并中间 中间那几槽从所选那一版自己的 txg 起（库单测：紧挨着的那一形把所选那一版那一槽也并进来）	crates/singlefs-core/src/mount.rs	(selected_version_txg.0 + 1\n            ..newer_root_txg.0)	(selected_version_txg.0\n            ..newer_root_txg.0)	-p singlefs-core --lib -- placements_of_the_publishes_in_between_chained_onto_the_newer_roots_tests	a_root_taking_the_current_account_right_after_the_selected_version_takes_nothing_more
````

