# 根环实现第 10z 批第十二、十三块：代码轮第一轮改动全文（2026-10-08）

两块实现员各交的 `crates.patch` 原样，按合入次序：第十三块、第十二块（第十三块打在第十、十一块合入之后，第十二块打在第十三块合入之后）。各块的变异表追加、整行替换附在各自后面。

## 第十三块

````diff
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
index 2f558e2..22cfc19 100644
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -28,6 +28,8 @@
 //!   或同一台设备上另一笔借据等成了），那台设备记成丢了、或放掉了也销
 //!   （第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：两腿都量到等超时的那几笔累加到整张卡借不出，到设备丢了才回来；
 //!   第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K1-1：列卡之后才记成没等成的那一笔不一起销，它的活可以是 poll 之后才提交的）。
+//!   卡池借之前列卡时看到那张卡留着的设备丢了，当场告诉账、照重起那一套记成丢了（[`GpuMemoryLedger::take_in_the_devices_kept_before_booking`]），
+//!   不等借到之后走到重起（第 10z 批第十三块，第 10z 批第十、十一块代码轮判决 K4-2：那几笔让后面的借都放不下时走不到重起，卡空着却再也借不出）。
 //!   读不到实际占着的（`nvidia-smi` 没列这个进程），或这个进程在这张卡上另有不走这份账的设备（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]，
 //!   分不出哪一份是借据建的），借着的当一点都没建、驱动那一份也当还没占上卡（第 10y 批代码轮判决 K1-1 攻方乙附：当它整份在卡上，实占的 x 比 D 小时
 //!   多给 D − x），只会少借。这张卡上起好过的设备丢了、账照重起记了之后，到丢了的那几台都放掉之前也这样算
@@ -1402,7 +1404,7 @@ impl GpuMemoryLedger {
 
     /// 列卡：此刻每张有等设备没等成而照旧记着的借据的卡，连同那一刻照旧记着的那几笔（卡池借之前对这几张卡留着的设备 poll 一次，交回队列空了
     /// 只销列出来的那几笔，[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]；第 10z 批第十一块，
-    /// 第 10z 批第七八九块代码轮判决 K1-1）。按卡在池里的先后。
+    /// 第 10z 批第七八九块代码轮判决 K1-1；留着的设备丢了的告诉账，[`Self::take_in_the_devices_kept_before_booking`]）。按卡在池里的先后。
     #[must_use]
     pub fn leases_kept_after_a_failed_wait_on_each_card(
         &self,
@@ -1424,6 +1426,49 @@ impl GpuMemoryLedger {
             .collect()
     }
 
+    /// 卡池借之前那一步（`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::GpuCardPool` 借之前只调它）：列卡
+    /// （[`Self::leases_kept_after_a_failed_wait_on_each_card`]），对每张列出来的卡取卡池那一格留着的设备（`device_kept_on_the_card`，
+    /// 不等正在起的设备；没有的跳过）：
+    /// - 丢了（`is_lost`）：不 poll（wgpu 在丢了的设备上 poll 会 panic），告诉账这台设备丢了，照重起那一套记
+    ///   （[`DeviceStartedOnTheBooks::take_in_that_the_device_was_found_lost`]：它上面没等成的那几笔销账、这张卡照分不出算）。
+    ///   不等借到之后走到重起才记：那几笔记着的量让后面的借都放不下时，借不到就走不到重起，这张卡在这个进程里再也借不出
+    ///   （第 10z 批第十三块，第 10z 批第十、十一块代码轮判决 K4-2，正推账层探针：丢了之后能借的 1 000、借 4 000 被拒）；
+    /// - 没丢：poll 一次（`found_idle_by_a_poll`，不等），交回队列空了只销列出来的那几笔
+    ///   （[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]）。
+    ///
+    /// 不在账的锁里 poll；`on_the_books` 交那台设备在账上的那一台。
+    ///
+    /// # Panics
+    /// `on_the_books` 交的那一台不在这本账、或不在列出来的那张卡上（卡池那一格接错了卡）。
+    pub fn take_in_the_devices_kept_before_booking<Device>(
+        &self,
+        device_kept_on_the_card: impl Fn(usize) -> Option<Device>,
+        is_lost: impl Fn(&Device) -> bool,
+        found_idle_by_a_poll: impl Fn(&Device) -> bool,
+        on_the_books: impl Fn(&Device) -> &DeviceStartedOnTheBooks<'_>,
+    ) {
+        for listed in self.leases_kept_after_a_failed_wait_on_each_card() {
+            let Some(kept) = device_kept_on_the_card(listed.card_position()) else {
+                continue;
+            };
+            let kept_on_the_books = on_the_books(&kept);
+            assert!(
+                std::ptr::eq(kept_on_the_books.ledger, self)
+                    && kept_on_the_books.card_position == listed.card_position(),
+                "列的是第 {} 张卡，卡池那一格留着的设备在账上记在第 {} 张卡上（或别的账上）",
+                listed.card_position(),
+                kept_on_the_books.card_position
+            );
+            if is_lost(&kept) {
+                kept_on_the_books.take_in_that_the_device_was_found_lost();
+                continue;
+            }
+            if found_idle_by_a_poll(&kept) {
+                kept_on_the_books.take_in_that_the_work_of_the_listed_leases_finished(&listed);
+            }
+        }
+    }
+
     /// 借据号是 `lease_identifier` 的那一笔此刻还记在第 `card_position` 张卡上没有（还了就不在了）。
     #[must_use]
     pub fn is_the_lease_on_the_books(&self, card_position: usize, lease_identifier: u64) -> bool {
@@ -1619,6 +1664,19 @@ impl DeviceStartedOnTheBooks<'_> {
                 &listed.leases_kept_on_each_device,
             );
     }
+
+    /// 卡池借之前看到这台设备丢了（[`GpuMemoryLedger::take_in_the_devices_kept_before_booking`]，还没走到重起）：它还是这张卡上在用的那一台时，
+    /// 照重起那一套记（`BooksOfACard::take_back_the_driver_share_of_a_lost_device`：驱动那一份记回许出去而还没占上卡、这一台记成丢了、
+    /// 放掉之前这个进程占着的照分不出算，它上面没等成而照旧记着的那几笔销账）；之后重起时驱动那一份照此刻再判一次
+    /// （[`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`]）。已经记成丢了、或这张卡已经起了别的一台（卡池看到的
+    /// 是重起之前留着的那一份）时什么都不改：起好的那一台没丢，它上面没等成的照旧记着。
+    pub fn take_in_that_the_device_was_found_lost(&self) {
+        let mut books = self.ledger.books_locked();
+        let books_of_the_card = &mut books[self.card_position];
+        if books_of_the_card.device_in_use == Some(self.device) {
+            books_of_the_card.take_back_the_driver_share_of_a_lost_device();
+        }
+    }
 }
 
 impl Drop for DeviceStartedOnTheBooks<'_> {
@@ -5225,6 +5283,445 @@ mod tests {
         drop(next_device);
     }
 
+    /// 正推账层探针那一形的起头：设备起好（驱动实占 600）、头一笔等完设备还了；再借此刻能借的减 1 000、建满、等设备超时、丢掉（照旧记着）。
+    /// 交回账上的那一台、照旧记着的那一笔的借据号与它借的量（MiB）。
+    fn a_started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait<'ledger>(
+        card: &Mutex<CardUnderTheLedger>,
+        ledger: &'ledger GpuMemoryLedger,
+    ) -> (DeviceStartedOnTheBooks<'ledger>, u64, u64) {
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
+        let kept_mebibytes = room_now(ledger) - 1_000;
+        let kept_lease =
+            a_lease_built_and_dropped_after_a_timed_out_wait(card, ledger, &device, kept_mebibytes);
+        (device, kept_lease, kept_mebibytes)
+    }
+
+    /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K4-2，正推账层探针 `probe_kept_leases_on_a_lost_device.rs`：丢了的设备上没等成的那一笔
+    /// 记在账上，账到重起才知道设备丢了，卡池只在借到之后才走到重起；能借的 1 000、借 4 000 被拒，这张卡在这个进程里再也借不出）：设备丢了，
+    /// 它上面的缓冲与驱动那一份交回了驱动（卡上空着）；卡池借之前列卡、看到留着的设备丢了：不 poll 它、告诉账，那一笔销账，借 4 000 借得到，
+    /// 能借的不超过卡上真能再给的；之后的重起照常（驱动那一份照此刻判、新的一台记得上）。
+    #[test]
+    fn a_lease_kept_on_a_device_found_lost_before_booking_leaves_the_books_and_four_thousand_fit_again(
+    ) {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let (device, kept_lease, _kept_mebibytes) =
+            a_started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = false;
+            card.driver_occupied_mebibytes = 0;
+            card.built_mebibytes = 0;
+        }
+        assert_eq!(
+            room_now(&ledger),
+            1_000,
+            "账还不知道设备丢了：照旧记着的那一笔占着，能借的 1 000（正推探针 room_now_after_lost）"
+        );
+        let polls_of_the_lost_device = AtomicUsize::new(0);
+        ledger.take_in_the_devices_kept_before_booking(
+            |card_position| (card_position == 0).then_some(&device),
+            |_kept| true,
+            |_kept| {
+                polls_of_the_lost_device.fetch_add(1, Ordering::SeqCst);
+                true
+            },
+            |kept| *kept,
+        );
+        assert_eq!(
+            polls_of_the_lost_device.load(Ordering::SeqCst),
+            0,
+            "丢了的设备不 poll（wgpu 在丢了的设备上 poll 会 panic）"
+        );
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, kept_lease),
+            "借之前账就知道设备丢了：它上面没等成的那一笔照重起记、销账"
+        );
+        assert!(
+            room_now(&ledger) <= room_truly_left(&card, 0),
+            "不多给（{} 对 {}）",
+            room_now(&ledger),
+            room_truly_left(&card, 0)
+        );
+        let booking = ledger
+            .book_on_the_card(0, 4_000)
+            .expect("卡上空着：借 4 000 借得到（改之前 NoCardHasRoomNow，能借的 1 000）");
+        ledger
+            .prepare_the_driver_share_before_starting_the_device(0)
+            .expect("重起：驱动那一份照此刻放得下");
+        let replacement = ledger.record_the_device_started_on_the_card(0);
+        drop(booking);
+        drop(replacement);
+        drop(device);
+    }
+
+    /// 第 10z 批第十三块（K4-2 那一改不多给）：同上，设备丢了而它上面那一笔的缓冲与驱动那一份还占着卡（旧设备还没收拾）；卡池借之前告诉账之后，
+    /// 那一笔出了账、这张卡照分不出算：能借的不超过卡上真能再给的，借 4 000 照旧借不到。
+    #[test]
+    fn a_device_found_lost_with_its_buffers_still_on_the_card_lends_no_more_than_the_card_has_left()
+    {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let (device, kept_lease, kept_mebibytes) =
+            a_started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = false;
+            card.driver_occupied_mebibytes = 0;
+            card.built_mebibytes = 0;
+            card.left_by_a_lost_device_mebibytes = 600 + kept_mebibytes;
+        }
+        ledger.take_in_the_devices_kept_before_booking(
+            |card_position| (card_position == 0).then_some(&device),
+            |_kept| true,
+            |_kept| panic!("丢了的设备不 poll"),
+            |kept| *kept,
+        );
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, kept_lease),
+            "告诉账之后照重起记：那一笔出了账（缓冲在丢了的那一台上，放掉之前这张卡照分不出算）"
+        );
+        let room = room_now(&ledger);
+        let room_truly_left_on_the_card = room_truly_left(&card, 0);
+        assert!(
+            room <= room_truly_left_on_the_card,
+            "旧设备连那一笔的缓冲还占着卡：能借的 {room} 不超过卡上真能再给的 {room_truly_left_on_the_card}（不多给）"
+        );
+        assert!(
+            ledger.book_on_the_card(0, 4_000).is_err(),
+            "卡上真没有 4 000：照旧借不到"
+        );
+        drop(device);
+    }
+
+    /// 第 10z 批第十三块（K4-2 那一改，卡池看到的是重起之前留着的那一份）：丢了的那一台重起过了（重起那一步已经把它记成丢了），
+    /// 新的一台上有一笔没等成而照旧记着；卡池借之前取到的还是丢了的那一台、告诉账它丢了：账不动，新的一台没丢，它上面那一笔照旧记着。
+    #[test]
+    fn a_device_found_lost_after_its_restart_leaves_the_replacement_and_its_kept_lease_alone() {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
+        let lost_device = ledger.record_the_device_started_on_the_card(0);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = 600;
+        }
+        first.take_in_the_wait_before_returning(
+            &lost_device,
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+        drop(first);
+        ledger
+            .prepare_the_driver_share_before_starting_the_device(0)
+            .expect("重起：驱动那一份放得下");
+        let replacement = ledger.record_the_device_started_on_the_card(0);
+        let kept_on_the_replacement =
+            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &replacement, 4_000);
+        let room_before = room_now(&ledger);
+        ledger.take_in_the_devices_kept_before_booking(
+            |card_position| (card_position == 0).then_some(&lost_device),
+            |_kept| true,
+            |_kept| panic!("丢了的设备不 poll"),
+            |kept| *kept,
+        );
+        assert!(
+            ledger.is_the_lease_on_the_books(0, kept_on_the_replacement),
+            "新的一台没丢：它上面没等成的那一笔照旧记着"
+        );
+        assert_eq!(room_now(&ledger), room_before, "账不动");
+        assert_eq!(
+            ledger
+                .leases_kept_after_a_failed_wait_on_each_card()
+                .iter()
+                .map(LeasesKeptAfterAFailedWaitWhenListed::lease_identifiers)
+                .collect::<Vec<Vec<u64>>>(),
+            vec![vec![kept_on_the_replacement]],
+            "下一次列卡照旧列出新的一台上那一笔"
+        );
+        drop(replacement);
+        drop(lost_device);
+    }
+
+    /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K5，正推 Y1：卡池借之前 poll 的结局改成恒真照绿）：留着的设备没丢，poll 交回还有在飞的提交：
+    /// poll 了一次，那一笔照旧记着（它的缓冲可能还在卡上），账不动。
+    #[test]
+    fn a_device_still_busy_at_the_poll_before_booking_keeps_its_kept_lease_on_the_books() {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let (device, kept_lease, _kept_mebibytes) =
+            a_started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
+        let room_before = room_now(&ledger);
+        let polls = AtomicUsize::new(0);
+        ledger.take_in_the_devices_kept_before_booking(
+            |card_position| (card_position == 0).then_some(&device),
+            |_kept| false,
+            |_kept| {
+                polls.fetch_add(1, Ordering::SeqCst);
+                false
+            },
+            |kept| *kept,
+        );
+        assert_eq!(polls.load(Ordering::SeqCst), 1, "没丢：poll 一次");
+        assert!(
+            ledger.is_the_lease_on_the_books(0, kept_lease),
+            "还有在飞的提交：那一笔照旧记着"
+        );
+        assert_eq!(room_now(&ledger), room_before, "账不动");
+        assert!(room_now(&ledger) <= room_truly_left(&card, 0), "不多给");
+        drop(device);
+    }
+
+    /// 第 10z 批第十三块（K5，正推 Y2、Y3：poll 的结局改成恒假、列卡的循环只看第一张卡照绿）：两张卡上各有一笔没等成而照旧记着，
+    /// 两台设备都没丢、poll 都交回队列空了：每张卡 poll 一次，两笔都销账。
+    #[test]
+    fn every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease() {
+        let readings = replaceable_readings(vec![
+            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
+            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
+        ]);
+        let ledger = ledger_reading(&readings);
+        let mut devices = Vec::new();
+        let mut kept_leases = Vec::new();
+        for card_position in 0..2 {
+            let mut lease = ledger
+                .book_on_the_card(card_position, 1_000)
+                .expect("空卡借得到");
+            let device = ledger.record_the_device_started_on_the_card(card_position);
+            lease.take_in_the_wait_before_returning(
+                &device,
+                DeviceWaitBeforeReturningTheLease::TheWaitFailed,
+            );
+            kept_leases.push(lease.lease_identifier());
+            drop(lease);
+            devices.push(device);
+        }
+        let polls_on_each_card = [AtomicUsize::new(0), AtomicUsize::new(0)];
+        ledger.take_in_the_devices_kept_before_booking(
+            |card_position| devices.get(card_position),
+            |_kept| false,
+            |kept| {
+                polls_on_each_card[kept.card_position].fetch_add(1, Ordering::SeqCst);
+                true
+            },
+            |kept| *kept,
+        );
+        assert_eq!(
+            polls_on_each_card
+                .iter()
+                .map(|polls| polls.load(Ordering::SeqCst))
+                .collect::<Vec<usize>>(),
+            vec![1, 1],
+            "列出来的每张卡 poll 一次"
+        );
+        for (card_position, kept_lease) in kept_leases.iter().enumerate() {
+            assert!(
+                !ledger.is_the_lease_on_the_books(card_position, *kept_lease),
+                "第 {card_position} 张卡的设备队列空了：那一笔销账"
+            );
+        }
+    }
+
+    /// 第 10z 批第十三块（K5，正推 G1：去掉销列出来的借据时「等的是 poll 的那一台」那一条照绿）：丢了的那一台重起过、还没放掉，新的一台上
+    /// 有一笔没等成而照旧记着；拿丢了的那一台交「列出来的那几笔的活做完了」：列出来的那一笔等的是新的一台，不销。
+    #[test]
+    fn a_finished_poll_of_a_lost_device_left_alive_does_not_return_the_lease_kept_on_its_replacement(
+    ) {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
+        let lost_device = ledger.record_the_device_started_on_the_card(0);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = 600;
+        }
+        first.take_in_the_wait_before_returning(
+            &lost_device,
+            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+        );
+        drop(first);
+        ledger
+            .prepare_the_driver_share_before_starting_the_device(0)
+            .expect("重起：驱动那一份放得下");
+        let replacement = ledger.record_the_device_started_on_the_card(0);
+        let kept_on_the_replacement =
+            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &replacement, 4_000);
+        let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
+        lost_device.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
+        assert!(
+            ledger.is_the_lease_on_the_books(0, kept_on_the_replacement),
+            "poll 的是丢了的那一台：等新的一台的那一笔不销"
+        );
+        drop(replacement);
+        drop(lost_device);
+    }
+
+    /// 第 10z 批第十三块（K5，正推 G4：去掉「列的是这台设备那张卡」那一条断言照绿）：列的是第 1 张卡，交给第 0 张卡上的设备销：断言拦下，
+    /// 第 1 张卡上那一笔照旧记着。
+    #[test]
+    fn the_listed_leases_of_another_card_given_to_a_device_are_refused_by_an_assertion() {
+        let readings = replaceable_readings(vec![
+            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
+            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
+        ]);
+        let ledger = ledger_reading(&readings);
+        let booking_on_the_first_card = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
+        let device_on_the_first_card = ledger.record_the_device_started_on_the_card(0);
+        let mut lease_on_the_second_card = ledger.book_on_the_card(1, 1_000).expect("空卡借得到");
+        let device_on_the_second_card = ledger.record_the_device_started_on_the_card(1);
+        lease_on_the_second_card.take_in_the_wait_before_returning(
+            &device_on_the_second_card,
+            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
+        );
+        let kept_on_the_second_card = lease_on_the_second_card.lease_identifier();
+        drop(lease_on_the_second_card);
+        let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
+        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+            device_on_the_first_card
+                .take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
+        }));
+        assert!(
+            refused.is_err(),
+            "列的是第 1 张卡、交给第 0 张卡上的设备：断言拦下"
+        );
+        assert!(ledger.is_the_lease_on_the_books(1, kept_on_the_second_card));
+        drop(booking_on_the_first_card);
+        drop(device_on_the_first_card);
+        drop(device_on_the_second_card);
+    }
+
+    /// 第 10z 批第十三块（K4-2 那一改的接线）：卡池那一格交的设备在账上记在别的卡上（列的是第 1 张卡，取到的是第 0 张卡上的设备）：
+    /// 断言拦下。
+    #[test]
+    fn a_kept_device_on_the_books_of_another_card_is_refused_by_an_assertion() {
+        let readings = replaceable_readings(vec![
+            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
+            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
+        ]);
+        let ledger = ledger_reading(&readings);
+        let booking_on_the_first_card = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
+        let device_on_the_first_card = ledger.record_the_device_started_on_the_card(0);
+        let mut lease_on_the_second_card = ledger.book_on_the_card(1, 1_000).expect("空卡借得到");
+        let device_on_the_second_card = ledger.record_the_device_started_on_the_card(1);
+        lease_on_the_second_card.take_in_the_wait_before_returning(
+            &device_on_the_second_card,
+            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
+        );
+        let kept_on_the_second_card = lease_on_the_second_card.lease_identifier();
+        drop(lease_on_the_second_card);
+        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+            ledger.take_in_the_devices_kept_before_booking(
+                |_card_position| Some(&device_on_the_first_card),
+                |_kept| true,
+                |_kept| panic!("丢了的设备不 poll"),
+                |kept| *kept,
+            );
+        }));
+        assert!(refused.is_err(), "取到的设备不在列出来的那张卡上：断言拦下");
+        assert!(
+            ledger.is_the_lease_on_the_books(1, kept_on_the_second_card),
+            "第 1 张卡上那一笔照旧记着"
+        );
+        drop(booking_on_the_first_card);
+        drop(device_on_the_first_card);
+        drop(device_on_the_second_card);
+    }
+
+    /// 一台假设备：最后一个克隆丢掉时（wgpu 收设备那一下要等设备空了）先报一声、等用例放开才收完；`number` 认是哪一台。
+    struct DeviceWhoseLastDropWaits {
+        number: u32,
+        dropping_began: Mutex<Option<std::sync::mpsc::Sender<()>>>,
+        drop_may_finish: Mutex<Option<std::sync::mpsc::Receiver<()>>>,
+    }
+
+    impl Drop for DeviceWhoseLastDropWaits {
+        fn drop(&mut self) {
+            if let Some(dropping_began) =
+                self.dropping_began.lock().expect("只在丢的这一下拿").take()
+            {
+                // 用例已经不等了（断言失败、先丢了收的那一头）时照样往下收
+                let _sent_or_the_test_gave_up = dropping_began.send(());
+            }
+            if let Some(drop_may_finish) = self
+                .drop_may_finish
+                .lock()
+                .expect("只在丢的这一下拿")
+                .take()
+            {
+                let _finished_or_the_test_gave_up = drop_may_finish.recv();
+            }
+        }
+    }
+
+    /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K5，正推 G2：换下来的旧设备改回在 `started` 的锁里丢照绿）：留着的设备 1 丢了，
+    /// 一条线程重起、换上 2，丢 1 时卡住（等设备空了）；这时取留着的设备不等丢旧设备的那一下，交新起的 2。
+    #[test]
+    fn taking_the_device_kept_does_not_wait_for_the_replaced_device_to_be_dropped() {
+        let slot: DeviceStartedOnDemand<Arc<DeviceWhoseLastDropWaits>> =
+            DeviceStartedOnDemand::new();
+        let (dropping_began_sender, dropping_began) = std::sync::mpsc::channel::<()>();
+        let (finish_the_drop, drop_may_finish) = std::sync::mpsc::channel::<()>();
+        let first_started = slot.started_or_start(
+            |_device: &Arc<DeviceWhoseLastDropWaits>| false,
+            move || {
+                Ok::<Arc<DeviceWhoseLastDropWaits>, String>(Arc::new(DeviceWhoseLastDropWaits {
+                    number: 1,
+                    dropping_began: Mutex::new(Some(dropping_began_sender)),
+                    drop_may_finish: Mutex::new(Some(drop_may_finish)),
+                }))
+            },
+        );
+        assert_eq!(first_started.map(|device| device.number), Ok(1));
+        let slot_shared = &slot;
+        let (taken_sender, taken) = std::sync::mpsc::channel::<Option<u32>>();
+        std::thread::scope(|scope| {
+            let restarting = scope.spawn(move || {
+                slot_shared
+                    .started_or_start(
+                        |device: &Arc<DeviceWhoseLastDropWaits>| device.number == 1,
+                        || {
+                            Ok::<Arc<DeviceWhoseLastDropWaits>, String>(Arc::new(
+                                DeviceWhoseLastDropWaits {
+                                    number: 2,
+                                    dropping_began: Mutex::new(None),
+                                    drop_may_finish: Mutex::new(None),
+                                },
+                            ))
+                        },
+                    )
+                    .map(|device| device.number)
+            });
+            let dropping_the_replaced_device =
+                dropping_began.recv_timeout(std::time::Duration::from_secs(10));
+            scope.spawn(move || {
+                // 用例已经不等了时照样收尾
+                let _sent_or_the_test_gave_up =
+                    taken_sender.send(slot_shared.started_now().map(|device| device.number));
+            });
+            let taken_while_the_replaced_device_drops =
+                taken.recv_timeout(std::time::Duration::from_secs(10));
+            finish_the_drop.send(()).expect("丢旧设备那一下在等放开");
+            assert_eq!(
+                dropping_the_replaced_device,
+                Ok(()),
+                "重起换上 2 之后丢旧设备 1"
+            );
+            assert_eq!(
+                taken_while_the_replaced_device_drops,
+                Ok(Some(2)),
+                "丢旧设备的那一下不拿着留着的那一份：取留着的设备不等，交新起的 2"
+            );
+            assert_eq!(restarting.join().expect("重起不 panic"), Ok(2));
+        });
+    }
+
     /// 第 10z 批第八块（第 10z 批第六块代码轮攻方改法 F-K1）：没等成的那一笔记着，同一台设备上后来另一笔借据等成了（等的是到那一刻为止的全部提交）：
     /// 没等成的那一笔一起销账。
     #[test]
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index 0cfd69b..7a18528 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -1329,30 +1329,31 @@ pub mod gpu_test_start {
             /// 借之前：列出账上有等设备没等成而照旧记着的借据的卡、连同那一刻照旧记着的那几笔，对池里那张卡留着的设备 poll 一次（不等）；
             /// 交回队列空了，那台设备上到 poll 那一刻为止提交的活都做完了、丢掉的缓冲交回了驱动，列出来的那几笔销账（第 10z 批第八块，
             /// 第 10z 批第六块代码轮判决 K1-2：等超时的那一笔不永久少借，两次超时之间没有别的等成时也出得来）。留着的设备丢了的不 poll
-            /// （wgpu 在丢了的设备上 poll 会 panic）：那几笔到重起时照重起记销。取留着的设备不等别的线程起设备（`DeviceStartedOnDemand::started_now`，
+            /// （wgpu 在丢了的设备上 poll 会 panic），当场告诉账、照重起记（那几笔销账、这张卡照分不出算），不等借到之后走到重起
+            /// （第 10z 批第十三块，第 10z 批第十、十一块代码轮判决 K4-2）。取留着的设备不等别的线程起设备（`DeviceStartedOnDemand::started_now`，
             /// 第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K4-2）。
             /// 不在账的锁里 poll，只销列卡那一刻记着的：它们丢掉的缓冲在列卡之前提交、丢掉，poll 交回队列空了之后什么时候销都不多给；
             /// 列卡之后才记成没等成的那一笔不销，它的活可以是 poll 之后才提交的（第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K1-1）。
+            /// 列卡、丢没丢、poll 的结局怎么记都在 harness 档（`GpuMemoryLedger::take_in_the_devices_kept_before_booking`，不要卡就测得到），
+            /// 这里只接卡池这一侧的四样：哪一格留着的设备、丢没丢、poll 一次、账上的那一台。
             fn take_in_the_devices_found_idle(&self) {
-                for listed in self.ledger.leases_kept_after_a_failed_wait_on_each_card() {
-                    let Some(started) = self.cards[listed.card_position()].started.started_now()
-                    else {
-                        continue;
-                    };
-                    if started.device_condition().is_lost() {
-                        continue;
-                    }
-                    let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
-                        started.device().poll(wgpu::PollType::Poll)
-                    }));
-                    if device_found_idle_by_a_poll(&polled) {
+                self.ledger.take_in_the_devices_kept_before_booking(
+                    |card_position| self.cards[card_position].started.started_now(),
+                    |started: &GpuCard| started.device_condition().is_lost(),
+                    |started: &GpuCard| {
+                        device_found_idle_by_a_poll(&std::panic::catch_unwind(
+                            std::panic::AssertUnwindSafe(|| {
+                                started.device().poll(wgpu::PollType::Poll)
+                            }),
+                        ))
+                    },
+                    |started: &GpuCard| {
                         started
                             .device_on_the_books
-                            .as_ref()
+                            .as_deref()
                             .expect("卡池起的设备起好时放进了账上的那一台（started_card）")
-                            .take_in_that_the_work_of_the_listed_leases_finished(&listed);
-                    }
-                }
+                    },
+                );
             }
 
             /// 借到 `booking` 的那张卡起好的设备：留着的设备没丢就拿它，没有（第一次要、上一次起不来）或丢了就起一个
@@ -2399,6 +2400,68 @@ mod tests {
         }
     }
 
+    /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K4-2、K5：借之前那一步里面没有用例，丢了的设备到借到之后的重起才告诉账）：
+    /// 借之前那一步整个交给 harness 档的 `GpuMemoryLedger::take_in_the_devices_kept_before_booking`（列卡、丢了的告诉账不 poll、没丢的 poll、
+    /// 只销列出来的，都在那边不要卡地测），卡池这一侧只接四样：列出来那一格留着的设备（不等正在起的）、丢没丢照设备的情况、poll 一次不等
+    /// 且照 [`device_found_idle_by_a_poll`] 判、账上的那一台。扫这份源码（测试那一段之外、去掉注释行与空白），不要卡。
+    #[test]
+    fn the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking() {
+        let source = include_str!("gpu_unit_checks.rs");
+        let code = source
+            .split(TESTS_MODULE_OPENING)
+            .next()
+            .expect("split 至少交一段");
+        let signature = "fn take_in_the_devices_found_idle(&self) \u{7b}";
+        let start = code
+            .find(signature)
+            .unwrap_or_else(|| panic!("源码里有 {signature}"));
+        let after = &code[start + signature.len()..];
+        let body = &after[..after
+            .find("\n            \u{7d}\n")
+            .expect("借之前那一步以缩进十二格的右括号收尾")];
+        let body_without_comments_and_spaces: String = body
+            .lines()
+            .filter(|line| !line.trim_start().starts_with("//"))
+            .flat_map(str::chars)
+            .filter(|character| !character.is_whitespace())
+            .collect();
+        for (expected, what) in [
+            (
+                "self.ledger.take_in_the_devices_kept_before_booking(",
+                "整个交给账的那一步",
+            ),
+            (
+                "|card_position|self.cards[card_position].started.started_now(),",
+                "列出来那一格留着的设备，不等正在起的",
+            ),
+            (
+                "|started:&GpuCard|started.device_condition().is_lost(),",
+                "丢没丢照设备的情况",
+            ),
+            (
+                "|started:&GpuCard|{device_found_idle_by_a_poll(&std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{started.device().poll(wgpu::PollType::Poll)}),))},",
+                "poll 一次不等、接住 panic、照队列空了判",
+            ),
+            (
+                "|started:&GpuCard|{started.device_on_the_books.as_deref().expect(",
+                "账上的那一台是这台设备自己的",
+            ),
+        ] {
+            assert_eq!(
+                body_without_comments_and_spaces.matches(expected).count(),
+                1,
+                "借之前那一步要有一处 {expected}（{what}）：{body_without_comments_and_spaces}"
+            );
+        }
+        assert_eq!(
+            body_without_comments_and_spaces
+                .matches("|started:&GpuCard|")
+                .count(),
+            3,
+            "取到设备之后的三样各一个闭包、不多不少：{body_without_comments_and_spaces}"
+        );
+    }
+
     /// Cargo.lock 里 wgpu-core 的版本（`name = "wgpu-core"` 下一行的 `version = "…"`）。
     fn wgpu_core_version_in_the_lock(lock: &str) -> Option<String> {
         let mut lines = lock.lines();
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index cad0cf2..726c5b7 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -4825,6 +4825,75 @@ mod tests {
         }
     }
 
+    /// `line` 里 `stop_is_requested` 作为整个名字出现了几次（前后不连着字母、数字、下划线：`stop_is_requested_by_the_environment`、
+    /// `_stop_is_requested` 不算）。
+    fn whole_name_uses_of_the_stop_request(line: &str) -> usize {
+        let name = concat!("stop_is_", "requested");
+        let is_part_of_a_name = |neighbour: Option<char>| {
+            neighbour.is_some_and(|neighbour_character| {
+                neighbour_character.is_alphanumeric() || neighbour_character == '_'
+            })
+        };
+        line.match_indices(name)
+            .filter(|(position, _name)| {
+                !is_part_of_a_name(line[..*position].chars().next_back())
+                    && !is_part_of_a_name(line[position + name.len()..].chars().next())
+            })
+            .count()
+    }
+
+    /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K5，正推：下游各层把收到的叫停换成永不叫停，`cargo check` 只报没用的变量、
+    /// 变异跑道不带 `-D warnings`，照绿）：这份源码里每个收叫停参数（`stop_is_requested: &(dyn Fn() -> bool + Sync),`，测试那一段之外）的函数，
+    /// 参数行之后到函数收尾的顶格右括号都用到它（往下交，或当场问）；收它的是这六层，次序照源码。扫这份源码，不要卡。
+    #[test]
+    fn every_layer_handed_the_stop_request_hands_it_on_or_asks_it() {
+        let stop_parameter = concat!("stop_is_", "requested: &(dyn Fn() -> bool + Sync),");
+        let lines = code_lines_outside_the_tests();
+        let mut layers_handed_the_stop_request = Vec::new();
+        for (parameter_line_index, _parameter_line) in lines
+            .iter()
+            .enumerate()
+            .filter(|(_line_index, line)| line.trim() == stop_parameter)
+        {
+            let signature = lines[..parameter_line_index]
+                .iter()
+                .rev()
+                .find(|line| line.contains("fn "))
+                .expect("参数行之前有函数签名");
+            let layer = signature
+                .split("fn ")
+                .nth(1)
+                .and_then(|after_fn| after_fn.split(['(', '<']).next())
+                .expect("签名里 fn 之后是函数名")
+                .to_string();
+            let rest_of_the_layer: Vec<&String> = lines[parameter_line_index + 1..]
+                .iter()
+                .take_while(|line| line.as_str() != "\u{7d}")
+                .collect();
+            let uses: usize = rest_of_the_layer
+                .iter()
+                .map(|line| whole_name_uses_of_the_stop_request(line))
+                .sum();
+            assert!(
+                uses >= 1,
+                "{layer} 收了叫停却没用上（往下交、或当场问；换成永不叫停只会报没用的变量）：{rest_of_the_layer:#?}"
+            );
+            layers_handed_the_stop_request.push(layer);
+        }
+        assert_eq!(
+            layers_handed_the_stop_request,
+            vec![
+                "check_blocks_where_until",
+                "check_pending_blocks_in_pieces",
+                "judge_my_blocks_on_gpu_cards",
+                "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from",
+                "check_selected_blocks",
+                "judge_my_blocks_on_gpu",
+            ],
+            "收叫停的是这六层（多一层少一层都要回来看这条用例罩没罩到）"
+        );
+    }
+
     fn registered(classes: &[GpuComputationClass]) -> BTreeSet<GpuComputationClass> {
         classes.iter().copied().collect()
     }
````

### 第十三块 mutations-append.tsv

````
第10z批第十三块 显存账 K4-2 借之前看到留着的设备丢了不告诉账（照旧等借到之后的重起，正推账层探针）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                kept_on_the_books.take_in_that_the_device_was_found_lost();\n                continue;	                continue;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_kept_on_a_device_found_lost_before_booking_leaves_the_books_and_four_thousand_fit_again	a_lease_kept_on_a_device_found_lost_before_booking_leaves_the_books_and_four_thousand_fit_again
第10z批第十三块 显存账 K4-2 告诉账设备丢了只销那几笔、不照重起记（这张卡照旧按读数分，多给）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if books_of_the_card.device_in_use == Some(self.device) {\n            books_of_the_card.take_back_the_driver_share_of_a_lost_device();\n        }	        if books_of_the_card.device_in_use == Some(self.device) {\n            for kept_lease in std::mem::take(&mut books_of_the_card.leases_kept_after_a_failed_wait).into_keys() {\n                books_of_the_card.leases.remove(&kept_lease);\n            }\n        }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_device_found_lost_with_its_buffers_still_on_the_card_lends_no_more_than_the_card_has_left	a_device_found_lost_with_its_buffers_still_on_the_card_lends_no_more_than_the_card_has_left
第10z批第十三块 显存账 K4-2 告诉账设备丢了不看它还是不是在用的那一台（重起之后把新的一台也记成丢了）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if books_of_the_card.device_in_use == Some(self.device) {\n            books_of_the_card.take_back_the_driver_share_of_a_lost_device();	        if true {\n            books_of_the_card.take_back_the_driver_share_of_a_lost_device();	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_device_found_lost_after_its_restart_leaves_the_replacement_and_its_kept_lease_alone	a_device_found_lost_after_its_restart_leaves_the_replacement_and_its_kept_lease_alone
第10z批第十三块 显存账 K5 借之前 poll 的结局当成恒为队列空了（正推 Y1）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            if found_idle_by_a_poll(&kept) {	            if true || found_idle_by_a_poll(&kept) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_device_still_busy_at_the_poll_before_booking_keeps_its_kept_lease_on_the_books	a_device_still_busy_at_the_poll_before_booking_keeps_its_kept_lease_on_the_books
第10z批第十三块 显存账 K5 借之前 poll 的结局当成恒为还有在飞的（正推 Y2）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            if found_idle_by_a_poll(&kept) {	            if false && found_idle_by_a_poll(&kept) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease	every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease
第10z批第十三块 显存账 K5 借之前列卡的循环只看第一张卡（正推 Y3）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        for listed in self.leases_kept_after_a_failed_wait_on_each_card() {	        for listed in self.leases_kept_after_a_failed_wait_on_each_card().into_iter().take(1) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease	every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease
第10z批第十三块 显存账 K5 借之前留着的设备丢了也照 poll、不告诉账（正推 Y4）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            if is_lost(&kept) {	            if false && is_lost(&kept) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_kept_on_a_device_found_lost_before_booking_leaves_the_books_and_four_thousand_fit_again	a_lease_kept_on_a_device_found_lost_before_booking_leaves_the_books_and_four_thousand_fit_again
第10z批第十三块 显存账 K4-2 借之前那一步不核卡池那一格交的设备在不在列出来的那张卡上	crates/singlefs-harness/src/gpu_memory_ledger.rs	                std::ptr::eq(kept_on_the_books.ledger, self)\n                    && kept_on_the_books.card_position == listed.card_position(),	                std::ptr::eq(kept_on_the_books.ledger, self)\n                    || kept_on_the_books.card_position == listed.card_position(),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_kept_device_on_the_books_of_another_card_is_refused_by_an_assertion	a_kept_device_on_the_books_of_another_card_is_refused_by_an_assertion
第10z批第十三块 显存账 K5 销列出来的借据不看它等的是不是 poll 的那一台（正推 G1）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            if *listed_on_the_device == device\n                && self\n                    .leases_kept_after_a_failed_wait\n                    .remove(listed_lease)\n                    .is_some()	            if { let _ = (listed_on_the_device, device); true }\n                && self\n                    .leases_kept_after_a_failed_wait\n                    .remove(listed_lease)\n                    .is_some()	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_finished_poll_of_a_lost_device_left_alive_does_not_return_the_lease_kept_on_its_replacement	a_finished_poll_of_a_lost_device_left_alive_does_not_return_the_lease_kept_on_its_replacement
第10z批第十三块 显存账 K5 换下来的旧设备改回在 started 的锁里丢（正推 G2）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        let replaced = self\n            .started\n            .lock()\n            .unwrap_or_else(PoisonError::into_inner)\n            .replace(device.clone());\n        drop(replaced);	        *self.started.lock().unwrap_or_else(PoisonError::into_inner) = Some(device.clone());	-p singlefs-harness --lib -- gpu_memory_ledger::tests::taking_the_device_kept_does_not_wait_for_the_replaced_device_to_be_dropped	taking_the_device_kept_does_not_wait_for_the_replaced_device_to_be_dropped
第10z批第十三块 显存账 K5 销列出来的借据不核列的是不是这台设备那张卡（正推 G4）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        assert_eq!(\n            listed.card_position, self.card_position,	        assert_eq!(\n            self.card_position, self.card_position,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_listed_leases_of_another_card_given_to_a_device_are_refused_by_an_assertion	the_listed_leases_of_another_card_given_to_a_device_are_refused_by_an_assertion
第10z批第十三块 GPU 取卡 K4-2 借之前那一步丢没丢恒为没丢（丢了的照 poll、不告诉账）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                    |started: &GpuCard| started.device_condition().is_lost(),\n                    |started: &GpuCard| {\n                        device_found_idle_by_a_poll(	                    |_started: &GpuCard| false,\n                    |started: &GpuCard| {\n                        device_found_idle_by_a_poll(	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking	the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking
第10z批第十三块 GPU 取卡 K5 借之前那一步 poll 的结局恒为队列空了	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                        device_found_idle_by_a_poll(&std::panic::catch_unwind(	                        true || device_found_idle_by_a_poll(&std::panic::catch_unwind(	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking	the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking
第10z批第十三块 GPU 取卡 K4-2 借之前那一步取的总是第一张卡留着的设备	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                    |card_position| self.cards[card_position].started.started_now(),	                    |_card_position| self.cards[0].started.started_now(),	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking	the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking
第10z批第十三块 崩溃放量 K5 check_blocks_where_until 往下交的叫停换成永不叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	        checking_threads(),\n        stop_is_requested,	        checking_threads(),\n        &|| false,	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
第10z批第十三块 崩溃放量 K5 check_pending_blocks_in_pieces 领块之前不问叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	        if stop_is_requested() {\n            break;\n        }	        if false {\n            break;\n        }	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
第10z批第十三块 崩溃放量 K5 judge_my_blocks_on_gpu_cards 往下交的叫停换成永不叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	        GpuCardsSource::DevicesStartedForThisRun,\n        stop_is_requested,	        GpuCardsSource::DevicesStartedForThisRun,\n        &|| false,	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
第10z批第十三块 崩溃放量 K5 第 ① 段在卡上判记住的叫停换成永不叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	    let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);	    let stop_request = StopRequestRememberedInThisRun::new(&|| false);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
第10z批第十三块 崩溃放量 K5 check_selected_blocks 往下交的叫停换成永不叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	        unit_checksums,\n        version,\n        stop_is_requested,\n    )	        unit_checksums,\n        version,\n        &|| false,\n    )	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
第10z批第十三块 崩溃放量 K5 judge_my_blocks_on_gpu 往下交的叫停换成永不叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	        cards_taken_from,\n        stop_is_requested,\n    )\n}	        cards_taken_from,\n        &|| false,\n    )\n}	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
````

## 第十二块

````diff
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -2011,7 +2011,8 @@
                         // 一、这条根与所选那一版之间还有别的发布（攻方一形）：它从那几次继承的单元只记在 C 里那几次的槽上，
                         //    调用方照交回的 txg 把那几槽并进这条根那一槽（[`with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`]）；
                         // 二、这条根是一次管理员回退（正推一形）：它复活的单元在现行账里是已释放，这里另并上
-                        //    [`placements_a_rollback_publish_revived_released_in_the_current_account`]。
+                        //    [`placements_a_rollback_publish_revived_released_in_the_current_account`]；认不出是不是回退、
+                        //    或认出回退而复活集算不出时，那里退到现行账里全部已释放的落点（tri，用户 2026-10-08 定）。
                         txgs_of_the_roots_taking_the_current_account.push(root.checkpoint_txg);
                         let mut placements =
                             placements_still_allocated_in_the_account(current_records);
@@ -2104,13 +2105,19 @@
 /// 回退复活（判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 正推一形，用户 2026-10-08 定「丙继续补洞」；
 /// 第 10z 批第十块实现员定形，被攻过零轮）：走了丙那一支的根 `newer_root` 是一次管理员回退（向前发布）时，它从回退目标 R_old 取用户可见树，
 /// R_old 引用、现行账记成已释放的单元被它复活；回退发布不写它们，记录不点名，现行账里又是已释放，丙那一截滤掉了它们。
-/// 认回退：那次发布读得出的记录点名的打包单元（类标签 3）里，读得出、校验和对得上、打包记录类型是谱系（5）的那一片里有一条
-/// rollback_txg 等于这条根 txg 的谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条：回退那次发布追加一条 (T_rb, T_old)），
-/// T_old 就是回退目标的 txg。根环里读出、按最新那条根的实例表有效、txg 等于 T_old 的根就是 R_old，读它的账（影子账同一份读法
-/// [`placements_referenced_by_root`]）。交回 R_old 账里仍分配着、现行账里记成已释放的落点（按盘与起点槽对上）：只护复活的那一截，
-/// 不并现行账里别的已释放落点（那是攻方乙那一类，第四块判决第二节 K2-2 改法行：384 槽盘上隔离约 370 个槽、600 段里 588 段被拒）。
-/// 认不出回退（记录没点名谱系片、那一片读不出、里面没有这条根的谱系记录）、根环里没读出 R_old、R_old 的账读不出，都交空：
-/// 那几形照丙原样，少护的那一截交主 agent（第十块报告「停下交主 agent」）。只读盘，不改分配器。
+/// 认回退照那次发布读得出的记录点名的打包单元（类标签 3）分三种结局（[`rollback_recognized_among_the_packed_units_named_by_a_publish`]）：
+/// - 是回退：解开的谱系片（打包记录类型 5）里有一条 rollback_txg 等于这条根 txg 的谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条：
+///   回退那次发布追加一条 (T_rb, T_old)），T_old 就是回退目标的 txg。根环里读出、按最新那条根的实例表有效、txg 等于 T_old 的根就是 R_old，
+///   读它的账（影子账同一份读法 [`placements_referenced_by_root`]）。交回 R_old 账里仍分配着、现行账里记成已释放的落点（按盘与起点槽对上）：
+///   只护复活的那一截，不并现行账里别的已释放落点（那是攻方乙那一类，第四块判决第二节 K2-2 改法行：384 槽盘上隔离约 370 个槽、600 段里 588 段被拒）。
+/// - 肯定不是回退：点名的打包单元全读得出、解得开，里面没有这条谱系记录：交空，一槽不多。
+/// - 认不出时宽护（tri，用户 2026-10-08 弹窗定「认不出时宽护 tri」，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第五节；
+///   判法照正推 tri 臂 `research/prompts/m3-batch10z-jk-code-r1-sonnet-model/src/probe-arms-on-mount.diff`，细节第 10z 批第十二块实现员定，被攻过零轮）：
+///   没认出回退而点名的打包单元有读不出、解不开的（不知道是不是回退；删除发布的墓碑片读不出也走到这里），或认出回退而根环里没读出 R_old
+///   （它的根槽读不出，或它已不在环里）、R_old 的账读不出（是回退、复活集算不出），都交回现行账里全部已释放的落点（正推的对照臂那一截）。
+///   宽护那一截与别的 C 落点同一个时机解开（这个进程写掉这条根那一槽时）。
+///
+/// 只读盘，不改分配器。
 #[allow(
     clippy::ptr_arg,
     reason = "placements_referenced_by_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
@@ -2124,50 +2131,149 @@
     is_abandoned: &dyn Fn(&RootRecord) -> bool,
     current_records: &[AllocationRecord],
 ) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
-    let Some(rollback_target_txg) = rollback_target_txg_named_by_the_records_of_a_publish(
-        devices,
+    let all_released_in_the_current_account = || -> Vec<(DeviceIdentity, SlotNumber, u64)> {
+        current_records
+            .iter()
+            .filter(|record| record.is_released)
+            .map(|record| (record.device, record.slot, u64::from(record.span_slots)))
+            .collect()
+    };
+    let rollback_target_txg = match rollback_recognized_among_the_packed_units_named_by_a_publish(
         newer_root.checkpoint_txg,
-        readable_records_of_its_publish,
-    ) else {
-        return Vec::new();
+        &packed_units_named_by_the_records_of_a_publish_as_read(
+            devices,
+            readable_records_of_its_publish,
+        ),
+    ) {
+        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(rollback_target_txg) => {
+            rollback_target_txg
+        }
+        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback => return Vec::new(),
+        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback => {
+            return all_released_in_the_current_account();
+        }
     };
-    let released_in_the_current_account: BTreeSet<(DeviceIdentity, SlotNumber)> = current_records
-        .iter()
-        .filter(|record| record.is_released)
-        .map(|record| (record.device, record.slot))
-        .collect();
-    roots
-        .iter()
-        .filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)
-        .filter_map(|rollback_target| {
-            placements_referenced_by_root(devices, unit_area_start, rollback_target)
-        })
-        .flatten()
-        .filter(|(device, slot, _)| released_in_the_current_account.contains(&(*device, *slot)))
-        .collect()
+    let accounts_of_the_rollback_target: Option<Vec<Vec<(DeviceIdentity, SlotNumber, u64)>>> =
+        roots
+            .iter()
+            .filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)
+            .map(|rollback_target| {
+                placements_referenced_by_root(devices, unit_area_start, rollback_target)
+            })
+            .collect();
+    match accounts_of_the_rollback_target {
+        Some(accounts) if !accounts.is_empty() => {
+            let released_in_the_current_account: BTreeSet<(DeviceIdentity, SlotNumber)> =
+                current_records
+                    .iter()
+                    .filter(|record| record.is_released)
+                    .map(|record| (record.device, record.slot))
+                    .collect();
+            accounts
+                .into_iter()
+                .flatten()
+                .filter(|(device, slot, _)| {
+                    released_in_the_current_account.contains(&(*device, *slot))
+                })
+                .collect()
+        }
+        // 根环里没读出 R_old（它的根槽读不出，或它已不在环里），或 R_old 的账读不出：是回退、复活集算不出（tri 的第二种退法）。
+        Some(_) | None => all_released_in_the_current_account(),
+    }
 }
 
-/// 一次发布读得出的记录点名的打包单元（类标签 3）里，读得出、校验和对得上、打包记录类型是谱系（5）的每一片里，rollback_txg 等于
-/// `checkpoint_txg_of_the_publish` 的那条谱系记录的 T_old：那次发布是一次管理员回退，回到 T_old 那一版。认不出交 `None`。
-fn rollback_target_txg_named_by_the_records_of_a_publish<Device: BlockDevice>(
+/// 一次发布读得出的记录点名的一个打包单元（类标签 3）读成了什么。
+enum PackedUnitNamedByAPublishAsRead {
+    /// 两条位置条目都读不出，或读出来整单元校验和对不上条目里的。
+    UnreadableOrChecksumMismatch,
+    /// 读出来校验和对得上，`parse_packed_unit` 解不开。
+    ReadButUnparsable,
+    /// 解开了：打包记录类型与逐条记录的字节。
+    Parsed {
+        record_type: u16,
+        records: Vec<Vec<u8>>,
+    },
+}
+
+/// 一次发布读得出的记录点名的每个打包单元（类标签 3），按记录与点名的次序，经位置条目里的校验和读、解。只读盘。
+#[allow(
+    clippy::ptr_arg,
+    reason = "read_unit_via_locations 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
+)]
+fn packed_units_named_by_the_records_of_a_publish_as_read<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
-    checkpoint_txg_of_the_publish: CheckpointTxg,
     readable_records_of_the_publish: &[&crate::journal::JournalRecord],
-) -> Option<CheckpointTxg> {
+) -> Vec<PackedUnitNamedByAPublishAsRead> {
     let packed_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
     readable_records_of_the_publish
         .iter()
         .flat_map(|record| record.named.iter())
         .filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)
-        .filter_map(|named| {
-            read_unit_via_locations(devices, &named.locations, packed_unit_bytes).ok()
-        })
-        .filter_map(|bytes| crate::unit::parse_packed_unit(&bytes).ok())
-        .filter(|packed| packed.identity.record_type == crate::unit::PACKED_TYPE_LINEAGE)
-        .flat_map(|packed| packed.records)
-        .filter_map(|record_bytes| LineageRecord::parse(&record_bytes))
-        .find(|lineage| lineage.rollback_txg == checkpoint_txg_of_the_publish)
-        .map(|lineage| lineage.rollback_target_txg)
+        .map(
+            |named| match read_unit_via_locations(devices, &named.locations, packed_unit_bytes) {
+                Err(_unreadable) => PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch,
+                Ok(bytes) => match crate::unit::parse_packed_unit(&bytes) {
+                    Err(_unparsable) => PackedUnitNamedByAPublishAsRead::ReadButUnparsable,
+                    Ok(packed) => PackedUnitNamedByAPublishAsRead::Parsed {
+                        record_type: packed.identity.record_type,
+                        records: packed.records,
+                    },
+                },
+            },
+        )
+        .collect()
+}
+
+/// 认回退的三种结局（tri，用户 2026-10-08 弹窗定「认不出时宽护 tri」，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第五节）。
+#[derive(Debug, PartialEq, Eq)]
+enum RollbackRecognizedAmongThePackedUnitsNamedByAPublish {
+    /// 解开的谱系片（打包记录类型 5）里有一条 rollback_txg 等于这次发布 txg 的谱系记录：这次发布是一次管理员回退，回到 T_old 那一版。
+    RollbackTo(CheckpointTxg),
+    /// 点名的打包单元全读得出、解得开，谱系片里的记录也全解得开，没有那一条：肯定不是回退。
+    NotARollback,
+    /// 没认出回退，而点名的打包单元有读不出、校验和对不上、解不开的，或谱系片里有一条记录解不开：不知道是不是回退。
+    UnknownWhetherARollback,
+}
+
+/// 照一次发布的记录点名的打包单元读成的样子认回退（D18（块里携带什么信息） 已定项 24 第 5 条：回退那次发布追加一条 (T_rb, T_old)，
+/// 装在它写的谱系片里、记录点名）。只算内存。读出来的里面认出了就是回退，不管别的几片读不读得出；没认出时，有一片读不出、解不开就是不知道。
+fn rollback_recognized_among_the_packed_units_named_by_a_publish(
+    checkpoint_txg_of_the_publish: CheckpointTxg,
+    packed_units_as_read: &[PackedUnitNamedByAPublishAsRead],
+) -> RollbackRecognizedAmongThePackedUnitsNamedByAPublish {
+    let mut some_named_packed_unit_unknown = false;
+    for packed_unit in packed_units_as_read {
+        match packed_unit {
+            PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch
+            | PackedUnitNamedByAPublishAsRead::ReadButUnparsable => {
+                some_named_packed_unit_unknown = true;
+            }
+            PackedUnitNamedByAPublishAsRead::Parsed {
+                record_type,
+                records,
+            } => {
+                if *record_type != crate::unit::PACKED_TYPE_LINEAGE {
+                    continue;
+                }
+                for record_bytes in records {
+                    match LineageRecord::parse(record_bytes) {
+                        Some(lineage) if lineage.rollback_txg == checkpoint_txg_of_the_publish => {
+                            return RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(
+                                lineage.rollback_target_txg,
+                            );
+                        }
+                        Some(_lineage_of_another_publish) => {}
+                        None => some_named_packed_unit_unknown = true,
+                    }
+                }
+            }
+        }
+    }
+    if some_named_packed_unit_unknown {
+        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback
+    } else {
+        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback
+    }
 }
 
 /// 一份账里仍分配着的每个落点（盘、起点槽、跨度），按记录的次序。只读内存。
@@ -8605,4 +8711,194 @@
             "(6, 7) 之间没有 txg：原样"
         );
     }
+
+    /// 两条走了丙那一支的根各并各自与所选那一版之间那几槽（正推 K5 的 E7：只并最后一条照绿）。交进来的 txg 按根环里读出的次序，
+    /// 那不是 txg 的次序：S = 8 时 txg 24 在区域 0 槽 0、txg 23 在区域 2 槽 7，较新的 24 排在前面。所选那一版 txg 21，其间 Q（txg 22）。
+    /// 24 那一槽并 22、23 两槽的，23 那一槽并 22 那一槽的；只并其中一条时另一条那一槽缺它的中间那几槽
+    /// （只并 23 时 24 那一槽缺 23 写的 200：23 那一槽先于 24 那一槽写掉，其间 24 还在环里、引用着 200）。
+    #[test]
+    fn every_root_taking_the_current_account_takes_what_the_publishes_between_it_and_the_selected_version_named(
+    ) {
+        let slots_per_region = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM;
+        let ring_slot_of = |txg: u64| target_for_publish(CheckpointTxg(txg), slots_per_region);
+        assert!(
+            ring_slot_of(24) < ring_slot_of(23),
+            "根环里读出的次序是区域、槽：txg 24 排在 txg 23 前面（这条用例要的那一形）"
+        );
+        let named_by_ring_slot = BTreeMap::from([
+            (ring_slot_of(22), vec![placement(100)]),
+            (ring_slot_of(23), vec![placement(200)]),
+            (ring_slot_of(24), vec![placement(300)]),
+        ]);
+        let chained = with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(
+            named_by_ring_slot.clone(),
+            CheckpointTxg(21),
+            &[CheckpointTxg(24), CheckpointTxg(23)],
+            slots_per_region,
+        );
+        assert_eq!(
+            chained[&ring_slot_of(24)],
+            vec![placement(300), placement(100), placement(200)],
+            "txg 24 那一槽：原有的在前，再并上 22、23 两槽的"
+        );
+        assert_eq!(
+            chained[&ring_slot_of(23)],
+            vec![placement(200), placement(100)],
+            "txg 23 那一槽：原有的在前，再并上 22 那一槽的"
+        );
+        assert_eq!(
+            chained[&ring_slot_of(22)],
+            named_by_ring_slot[&ring_slot_of(22)],
+            "txg 22 那一槽原样"
+        );
+    }
+}
+
+/// 认回退的三种结局（tri，用户 2026-10-08 弹窗定「认不出时宽护 tri」）：只算内存的那一半
+/// （[`super::rollback_recognized_among_the_packed_units_named_by_a_publish`]）。读盘那一半与落到 C 里的那一截由 harness 档
+/// `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` 里回退复活那几条用例判。
+#[cfg(test)]
+mod rollback_recognized_among_the_packed_units_named_by_a_publish_tests {
+    use super::{
+        rollback_recognized_among_the_packed_units_named_by_a_publish,
+        PackedUnitNamedByAPublishAsRead, RollbackRecognizedAmongThePackedUnitsNamedByAPublish,
+    };
+    use crate::address::CheckpointTxg;
+    use crate::records::LineageRecord;
+    use crate::unit::{PACKED_TYPE_INODE, PACKED_TYPE_LINEAGE};
+
+    /// 这几条用例里那次发布的 txg。
+    const CHECKPOINT_TXG_OF_THE_PUBLISH: CheckpointTxg = CheckpointTxg(9);
+
+    fn lineage_record_bytes(rollback_txg: u64, rollback_target_txg: u64) -> Vec<u8> {
+        LineageRecord {
+            rollback_txg: CheckpointTxg(rollback_txg),
+            rollback_target_txg: CheckpointTxg(rollback_target_txg),
+        }
+        .to_bytes()
+    }
+
+    fn parsed(record_type: u16, records: Vec<Vec<u8>>) -> PackedUnitNamedByAPublishAsRead {
+        PackedUnitNamedByAPublishAsRead::Parsed {
+            record_type,
+            records,
+        }
+    }
+
+    /// 谱系片里有 rollback_txg 等于这次发布 txg 的那一条：是回退，回到它的 T_old；同一片里别的发布的谱系记录不碍事。
+    #[test]
+    fn a_lineage_record_of_this_publish_recognizes_a_rollback_to_its_target() {
+        assert_eq!(
+            rollback_recognized_among_the_packed_units_named_by_a_publish(
+                CHECKPOINT_TXG_OF_THE_PUBLISH,
+                &[
+                    parsed(PACKED_TYPE_INODE, vec![vec![0; 16]]),
+                    parsed(
+                        PACKED_TYPE_LINEAGE,
+                        vec![lineage_record_bytes(7, 3), lineage_record_bytes(9, 5)]
+                    ),
+                ],
+            ),
+            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(CheckpointTxg(5)),
+        );
+    }
+
+    /// 点名的打包单元全解得开，谱系片里只有别的发布（txg 7）的谱系记录：肯定不是回退（正推 K5 的 E2：不比 rollback_txg 时认成回到 3）。
+    #[test]
+    fn lineage_records_of_other_publishes_only_are_not_a_rollback() {
+        assert_eq!(
+            rollback_recognized_among_the_packed_units_named_by_a_publish(
+                CHECKPOINT_TXG_OF_THE_PUBLISH,
+                &[
+                    parsed(PACKED_TYPE_INODE, vec![vec![0; 16]]),
+                    parsed(PACKED_TYPE_LINEAGE, vec![lineage_record_bytes(7, 3)]),
+                ],
+            ),
+            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback,
+        );
+    }
+
+    /// 不是谱系片的打包单元里的记录不当谱系记录读：一片 inode 叶里的 16 字节记录恰好拼得出 (9, 5)，也不认成回退。
+    #[test]
+    fn records_of_a_packed_unit_that_is_not_a_lineage_unit_are_not_read_as_lineage_records() {
+        assert_eq!(
+            rollback_recognized_among_the_packed_units_named_by_a_publish(
+                CHECKPOINT_TXG_OF_THE_PUBLISH,
+                &[parsed(PACKED_TYPE_INODE, vec![lineage_record_bytes(9, 5)])],
+            ),
+            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback,
+        );
+    }
+
+    /// 点名的打包单元有一片读不出（或校验和对不上），读得出的那几片里没认出：不知道是不是回退。
+    #[test]
+    fn an_unreadable_named_packed_unit_without_a_recognized_rollback_is_unknown() {
+        assert_eq!(
+            rollback_recognized_among_the_packed_units_named_by_a_publish(
+                CHECKPOINT_TXG_OF_THE_PUBLISH,
+                &[
+                    parsed(PACKED_TYPE_INODE, vec![vec![0; 16]]),
+                    PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch,
+                ],
+            ),
+            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback,
+        );
+    }
+
+    /// 点名的打包单元有一片读得出、解不开：同样不知道。
+    #[test]
+    fn an_unparsable_named_packed_unit_without_a_recognized_rollback_is_unknown() {
+        assert_eq!(
+            rollback_recognized_among_the_packed_units_named_by_a_publish(
+                CHECKPOINT_TXG_OF_THE_PUBLISH,
+                &[
+                    PackedUnitNamedByAPublishAsRead::ReadButUnparsable,
+                    parsed(PACKED_TYPE_LINEAGE, vec![lineage_record_bytes(7, 3)]),
+                ],
+            ),
+            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback,
+        );
+    }
+
+    /// 谱系片解开了、里面有一条记录解不开（长度不是 16），别的记录都不是这次发布的：同样不知道。
+    #[test]
+    fn an_unparsable_lineage_record_without_a_recognized_rollback_is_unknown() {
+        assert_eq!(
+            rollback_recognized_among_the_packed_units_named_by_a_publish(
+                CHECKPOINT_TXG_OF_THE_PUBLISH,
+                &[parsed(
+                    PACKED_TYPE_LINEAGE,
+                    vec![lineage_record_bytes(7, 3), vec![0; 15]]
+                )],
+            ),
+            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback,
+        );
+    }
+
+    /// 读得出的那几片里认出了回退，就是回退：另一片读不出不改结局（回退那条谱系记录只这一次发布写）。
+    #[test]
+    fn a_recognized_rollback_stands_though_another_named_packed_unit_is_unreadable() {
+        assert_eq!(
+            rollback_recognized_among_the_packed_units_named_by_a_publish(
+                CHECKPOINT_TXG_OF_THE_PUBLISH,
+                &[
+                    PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch,
+                    parsed(PACKED_TYPE_LINEAGE, vec![lineage_record_bytes(9, 5)]),
+                ],
+            ),
+            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(CheckpointTxg(5)),
+        );
+    }
+
+    /// 一片打包单元都没点名（读得出的记录里没有类标签 3 的）：肯定不是回退（回退那次发布必定写谱系片）。
+    #[test]
+    fn no_named_packed_unit_is_not_a_rollback() {
+        assert_eq!(
+            rollback_recognized_among_the_packed_units_named_by_a_publish(
+                CHECKPOINT_TXG_OF_THE_PUBLISH,
+                &[],
+            ),
+            RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback,
+        );
+    }
 }
--- a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -26,6 +26,9 @@
 //!   丙并中间——R 与所选那一版 P 之间还有一次发布 Q（攻方一形），C 里 R 那一槽并上 Q 那一槽记着的落点，会话写到写掉 R 那一槽，
 //!   每一步根落之前那一态与每一步之后池级 checker 不红 I-7.4；
 //!   回退复活——R 是一次管理员回退（正推一形），C 里 R 那一槽并上回退目标账里仍分配、P 的账里已释放的落点，推过抬 F 复活的槽不空闲、不红 I-7.4。
+//! - 认不出时宽护（tri，用户 2026-10-08 弹窗定，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第五节；细节第 10z 批第十二块实现员定，
+//!   被攻过零轮）：回退复活那一形再让 R 写的谱系片、R 写的全部单元、R_old 的根槽、R_old 的账之一读回全零，C 里 R 那一槽退到现行账里全部已释放的落点，
+//!   写到推过抬 F 不红 I-7.4；认得出回退时只多 R_old 账里仍分配、现行账里已释放的那一截；R 不是回退时一槽不多、挂载那一刻隔离数与之前相同。
 // 池（384 槽那一档小盘）与 R 那块盘都经 `tests/common_unwitnessed_publish/mod.rs` 起（它转出 `tests/common_admission_pool/mod.rs` 的全部）：
 // 再声明 `common_admission` 就把 `common_admission_pool` 当模块装两遍（clippy `duplicate_mod`）。
 mod common_unwitnessed_publish;
@@ -1704,12 +1707,23 @@
 /// 回退复活那一形推过第一串抬 F 之后再顺序写几次（每次两个数据单元；探针原样 3 次）。
 const SEQUENTIAL_WRITES_AFTER_THE_FLOOR_RAISE_OVER_THE_ROLLBACK: usize = 3;
 
+/// 录下来的一次管理员回退 R（[`record_one_rollback_to_the_first_file_after_overwrites`]）。
+struct RecordedRollback {
+    recorded: RecordedPublish,
+    /// R 复活的数据单元的落点：R_old 的文件数据指针里 P 不引用的那几条位置条目（跨度按数据单元宽折成槽数）。
+    revived: Vec<PlacementOnDevice>,
+    /// 回退目标 R_old（第一个文件那一版）的根。
+    rollback_target: RootRecord,
+    /// R_old 那一版的账里仍分配着的（盘、起点槽）：起池之后、覆盖写之前会话分配器里的记录（R_old 那次发布写下的账）。
+    allocated_in_the_account_of_the_rollback_target: Vec<(DeviceIdentity, SlotNumber)>,
+}
+
 /// 正推探针的 `record_one_rollback_after_overwrites`：起池（384 槽那一档、第一个文件，那一版是 R_old），覆盖写 `overwrites_before` 次得 P，
-/// 再经会话做一次管理员回退到 R_old、录这一次发布得 R。交回录下来的 R 与 R 复活的单元的落点：R_old 的文件数据指针里 P 不引用的那几条位置条目
-/// （从录下来的会话算，不经被测的方法；跨度按数据单元宽折成槽数）。
+/// 再经会话做一次管理员回退到 R_old、录这一次发布得 R。交回录下来的 R、R 复活的单元的落点、R_old 的根与它账里仍分配着的落点
+/// （都从录下来的会话算，不经被测的方法）。
 fn record_one_rollback_to_the_first_file_after_overwrites(
     overwrites_before: u64,
-) -> (RecordedPublish, Vec<PlacementOnDevice>) {
+) -> RecordedRollback {
     let stream = SharedStream::retaining_contents();
     let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
         parameters_of_the_384_slot_small_devices(),
@@ -1720,6 +1734,14 @@
     );
     let root_of_the_first_file = *pool.session().current.root();
     let data_pointers_of_the_first_file = pool.current_file_version().data_pointers.clone();
+    let allocated_in_the_account_of_the_rollback_target: Vec<(DeviceIdentity, SlotNumber)> = pool
+        .session()
+        .allocator
+        .records()
+        .iter()
+        .filter(|record| !record.is_released)
+        .map(|record| (record.device, record.slot))
+        .collect();
     for _overwrite in 0..overwrites_before {
         pool.overwrite(OVERWRITE_BYTES)
             .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
@@ -1776,8 +1798,8 @@
             span: singlefs_format::DATA_UNIT_BYTES.div_ceil(singlefs_format::SLOT_BYTES),
         })
         .collect();
-    (
-        RecordedPublish {
+    RecordedRollback {
+        recorded: RecordedPublish {
             content: previous_content.clone(),
             pool,
             base,
@@ -1788,7 +1810,9 @@
             previous_content,
         },
         revived,
-    )
+        rollback_target: root_of_the_first_file,
+        allocated_in_the_account_of_the_rollback_target,
+    }
 }
 
 /// 回退复活（正推 `research/prompts/m3-batch10z-ghi-code-r1-sonnet-output.md` K2-1、探针 `probe_rollback_revived.tail.rs`；判决
@@ -1802,7 +1826,9 @@
 #[test]
 fn units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount(
 ) {
-    let (recorded, revived) = record_one_rollback_to_the_first_file_after_overwrites(
+    let RecordedRollback {
+        recorded, revived, ..
+    } = record_one_rollback_to_the_first_file_after_overwrites(
         OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
     );
     assert!(
@@ -1915,3 +1941,435 @@
         "推过抬 F 那一刻 R 复活的数据单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_revived_after_the_raise:?}"
     );
 }
+
+/// 认不出时宽护（tri，用户 2026-10-08 弹窗定「认不出时宽护 tri」，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第五节；
+/// 判法照正推 tri 臂 `research/prompts/m3-batch10z-jk-code-r1-sonnet-model/src/probe-arms-on-mount.diff`，第 10z 批第十二块实现员定细节，被攻过零轮）
+/// 那四形共用的走法：回退复活那一形（覆盖写 4 次得 P、回退到第一个文件那一版录 R），可写挂载那一刻 `ranges` 读回全零（R 的账另加一处，
+/// 让认回退或算复活集那一步走不成）：择根退到 P，走丙。挂上之后撤故障，会话覆盖写到准入推出第一串抬 F（推的那一刻 R 还在根环里），
+/// 再顺序写 3 次，每一步之后池级 checker 不红 I-7.4 与 I-4.8；全程过了之后再核 C 里 R 那一槽有复活的全部落点、推过抬 F 那一刻复活的槽不空闲。
+/// 次序是故意的：撤掉宽护（这几形照丙原样交空）时先红的是写到半途的 I-7.4（复活的单元被回收、写掉），不是借到那一句。
+fn units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
+    recorded_rollback: &RecordedRollback,
+    ranges: &RangesReadingBackAllZero,
+) {
+    let recorded = &recorded_rollback.recorded;
+    let revived = &recorded_rollback.revived;
+    assert!(
+        !revived.is_empty(),
+        "第一个文件那一版的数据单元 P 不引用：回退复活了它们（阳性对照：这一形走到了复活）"
+    );
+    let root_of_r = recorded.root();
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let lift = ReadBackAllZeroLift::default();
+    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, ranges, &lift);
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
+        free_slots_covered_by(&pool.session().allocator, revived);
+    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_THE_FLOOR_RAISE_OVER_THE_ROLLBACK {
+        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
+            .unwrap_or_else(|refusal| {
+                panic!("推过抬 F 之后第 {sequential_write_number} 次顺序写被拒：{refusal:?}")
+            });
+        assert_nothing_of_r_reused(&pool, "推过抬 F 之后顺序写");
+    }
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
+        "挂载那一刻 C 里 R 那一槽退到现行账里全部已释放的落点（宽护）：复活的都在里面，缺 {revived_not_borrowed:?}"
+    );
+    assert!(
+        free_among_the_revived_after_the_raise.is_empty(),
+        "推过抬 F 那一刻 R 复活的数据单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_revived_after_the_raise:?}"
+    );
+}
+
+/// R 写出的谱系片（打包记录类型 5 的打包单元）的写：从录下来的写的字节解，不经被测的方法。
+fn lineage_unit_writes_of(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
+    recorded
+        .unit_writes()
+        .into_iter()
+        .filter(|write| {
+            write
+                .bytes()
+                .and_then(|bytes| singlefs_core::unit::parse_packed_unit(bytes).ok())
+                .is_some_and(|packed| {
+                    packed.identity.record_type == singlefs_core::unit::PACKED_TYPE_LINEAGE
+                })
+        })
+        .collect()
+}
+
+/// `image` 的根环里 (实例, txg) 为 `key`、自证过的那条根（盘上那一份：带着它的树表、分配记录树根等指针）。
+///
+/// # Panics
+/// `image` 上择不到系统配置，或根环里没有自证过的这条根。
+fn root_read_from_the_ring_of(
+    image: &MemoryPool,
+    key: (InstanceGeneration, CheckpointTxg),
+) -> RootRecord {
+    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
+    let sizes = &configuration.immutable.sizes;
+    singlefs_core::recovery::every_root_ring_slot(sizes)
+        .into_iter()
+        .find_map(|ring_slot| {
+            if let singlefs_core::recovery::RootRingSlotReading::SelfVerified(root) =
+                singlefs_core::recovery::read_root_ring_slot(
+                    image,
+                    &configuration.immutable.region_devices,
+                    sizes,
+                    &configuration.immutable.filesystem_identifier,
+                    ring_slot,
+                )
+            {
+                ((root.instance, root.checkpoint_txg) == key).then_some(root)
+            } else {
+                None
+            }
+        })
+        .expect("根环里有自证过的这条根")
+}
+
+/// 挂载那一刻现行账（所选那一版 P 的账）里记成已释放、挂上之后还记着的（盘、起点槽）：可写挂载交回的分配器里释放代不高于 P 的 txg 的那几条
+/// （释放代高于它的是这次挂载自己的写行、暖机释放的，挂载那一刻还分配着）。
+fn released_in_the_account_of_the_selected_version(
+    mounted: &singlefs_core::mount::Mounted,
+) -> std::collections::BTreeSet<(DeviceIdentity, SlotNumber)> {
+    let selected_version_txg = mounted.output.effective_root.checkpoint_txg;
+    mounted
+        .allocator
+        .records()
+        .iter()
+        .filter(|record| record.is_released && record.generation <= selected_version_txg)
+        .map(|record| (record.device, record.slot))
+        .collect()
+}
+
+/// tri 第一种「不知道是不是回退」：R 的账与 R 写的谱系片都读回全零（正推 `rlin`）——R 的记录点名的打包单元有一片读不出，认不出是不是回退。
+#[test]
+fn units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount(
+) {
+    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
+        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
+    );
+    let recorded = &recorded_rollback.recorded;
+    let lineage_unit_writes = lineage_unit_writes_of(recorded);
+    assert!(
+        !lineage_unit_writes.is_empty(),
+        "回退那次发布写了谱系片（D18（块里携带什么信息） 已定项 24 第 5 条；阳性对照：故障有处可打）"
+    );
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&account_writes_of(recorded))
+        .with_the_writes(&lineage_unit_writes);
+    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
+        &recorded_rollback,
+        &ranges,
+    );
+}
+
+/// tri 第一种的另一形：R 这次发布写的全部单元都读回全零（攻方 s2 `rall`），记录齐。
+#[test]
+fn units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount(
+) {
+    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
+        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
+    );
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&recorded_rollback.recorded.unit_writes());
+    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
+        &recorded_rollback,
+        &ranges,
+    );
+}
+
+/// tri 第二种「是回退、复活集算不出」：认得出回退（谱系片读得出），而 R_old 的根槽读回全零（正推 `rold`）——根环里没读出 R_old。
+#[test]
+fn units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount(
+) {
+    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
+        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
+    );
+    let recorded = &recorded_rollback.recorded;
+    let rollback_target = recorded_rollback.rollback_target;
+    let ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&account_writes_of(recorded))
+        .with_the_root_slot_of(
+            &recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
+            (rollback_target.instance, rollback_target.checkpoint_txg),
+        );
+    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
+        &recorded_rollback,
+        &ranges,
+    );
+}
+
+/// tri 第二种的另一形：认得出回退、根环里读得出 R_old，而 R_old 的树表与分配记录树根读回全零（正推 `roldacct`）——R_old 的账读不出。
+#[test]
+fn units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount(
+) {
+    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
+        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
+    );
+    let rollback_target = root_read_from_the_ring_of(
+        &recorded_rollback
+            .recorded
+            .crash_image_with_the_root_persisted_and_the_rotation_lost(),
+        (
+            recorded_rollback.rollback_target.instance,
+            recorded_rollback.rollback_target.checkpoint_txg,
+        ),
+    );
+    let mut ranges = RangesReadingBackAllZero::default()
+        .with_the_writes(&account_writes_of(&recorded_rollback.recorded));
+    for (pointer, unit) in [
+        (
+            rollback_target.tree_table,
+            singlefs_core::transaction::TransactionUnit::TreeTable,
+        ),
+        (
+            rollback_target.allocation_record_tree_root,
+            singlefs_core::transaction::TransactionUnit::AllocationTree,
+        ),
+    ]
+    .into_iter()
+    .filter(|(pointer, _)| *pointer != NodePointer::empty_root())
+    {
+        for location in pointer.locations {
+            ranges.0.push((
+                location.device,
+                location.slot.to_device_offset().0,
+                unit.span_slots() * singlefs_format::SLOT_BYTES,
+            ));
+        }
+    }
+    assert!(
+        ranges.0.len() > account_writes_of(&recorded_rollback.recorded).len(),
+        "R_old（第一个文件那一版）的树表或分配记录树根有落点（阳性对照：故障有处可打）"
+    );
+    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
+        &recorded_rollback,
+        &ranges,
+    );
+}
+
+/// 认得出回退时只多那一截（正推 K5 的 E3、E1，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第二节 K5 那一行）：
+/// 回退复活那一形、只有 R 的账读回全零（谱系片与 R_old 都读得出）。挂载那一刻 C 里 R 那一槽里、在挂载那一刻现行账里记成已释放的落点，
+/// 逐个就是 R_old 账里仍分配着、现行账里已释放的那一截（按盘与起点槽），一个不多、一个不少。认出回退就并现行账里全部已释放的（E3，
+/// 正推的对照臂）、回退目标按所有有效根找（E1）都会多出别的已释放落点。期望从录下来的会话算（R_old 那一刻的会话分配器），不经被测的方法。
+#[test]
+fn a_rollback_recognized_at_the_mount_takes_among_the_released_only_what_its_target_still_allocated(
+) {
+    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
+        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
+    );
+    let recorded = &recorded_rollback.recorded;
+    let root_of_r = recorded.root();
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let mut devices = devices_reading_back_all_zero_on(
+        &crash_image,
+        &RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(recorded)),
+    );
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
+    let released_in_the_current_account = released_in_the_account_of_the_selected_version(&mounted);
+    let borrowed_for_r = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+            root_of_r.instance,
+            root_of_r.checkpoint_txg,
+        )
+        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
+    let released_borrowed_for_r: std::collections::BTreeSet<(DeviceIdentity, SlotNumber)> =
+        borrowed_for_r
+            .iter()
+            .map(|placement| (placement.device, placement.slot))
+            .filter(|placement| released_in_the_current_account.contains(placement))
+            .collect();
+    let still_allocated_by_the_target_and_released: std::collections::BTreeSet<(
+        DeviceIdentity,
+        SlotNumber,
+    )> = recorded_rollback
+        .allocated_in_the_account_of_the_rollback_target
+        .iter()
+        .copied()
+        .filter(|placement| released_in_the_current_account.contains(placement))
+        .collect();
+    assert!(
+        recorded_rollback.revived.iter().all(|revived| {
+            still_allocated_by_the_target_and_released.contains(&(revived.device, revived.slot))
+        }),
+        "复活的落点都在 R_old 账里仍分配、现行账里已释放的那一截里（阳性对照：期望那一截罩着复活集）"
+    );
+    assert_eq!(
+        released_borrowed_for_r, still_allocated_by_the_target_and_released,
+        "C 里 R 那一槽里现行账已释放的落点，就是 R_old 账里仍分配、现行账里已释放的那一截（窄取法：不多并现行账里别的已释放落点）"
+    );
+}
+
+/// tri 不碰的那一支（正推 K2-3 第 1 条「读得出、确实没有这条根的谱系记录（肯定不是回退）」）：R 建一个 inode、挂载那一刻只有 R 的账读回全零，
+/// R 的记录点名的打包单元（inode 叶容器）都读得出、里面没有谱系记录。C 里 R 那一槽里一个现行账已释放的落点都没有（丙只并仍分配着的），
+/// 挂载那一刻两块盘隔离的槽数与 tri 之前逐格相同：18（R 写的单元，两块盘各 9 槽；正推对照臂这一形 116，
+/// 量法副本 `SINGLEFS_PROBE_ARM=narrow`（tri 之前）与 tri 两臂挂载那一刻同为 18，第 10z 批第十二块实现员报告第三节）。
+#[test]
+fn a_root_taking_the_current_account_that_is_not_a_rollback_takes_nothing_released_and_isolates_as_before_at_the_mount(
+) {
+    let recorded = record_one_inode_creation_after_overwrites(
+        OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_R,
+    );
+    let root_of_r = recorded.root();
+    assert!(
+        lineage_unit_writes_of(&recorded).is_empty(),
+        "建 inode 不写谱系片（阳性对照：R 不是回退）"
+    );
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let mut devices = devices_reading_back_all_zero_on(
+        &crash_image,
+        &RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded)),
+    );
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
+    let released_in_the_current_account = released_in_the_account_of_the_selected_version(&mounted);
+    assert!(
+        !released_in_the_current_account.is_empty(),
+        "挂载那一刻现行账里有已释放的落点（阳性对照：并多了看得出来）"
+    );
+    let released_borrowed_for_r: Vec<PlacementOnDevice> = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+            root_of_r.instance,
+            root_of_r.checkpoint_txg,
+        )
+        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）")
+        .into_iter()
+        .filter(|placement| {
+            released_in_the_current_account.contains(&(placement.device, placement.slot))
+        })
+        .collect();
+    assert!(
+        released_borrowed_for_r.is_empty(),
+        "R 不是回退：C 里 R 那一槽一个现行账里已释放的落点都不并，多并的 {released_borrowed_for_r:?}"
+    );
+    let isolated_slots_at_the_mount: u64 = mounted
+        .allocator
+        .devices
+        .iter()
+        .map(singlefs_core::allocator::DeviceFreeMap::isolated_slots)
+        .sum();
+    assert_eq!(
+        isolated_slots_at_the_mount,
+        ISOLATED_SLOTS_AT_THE_MOUNT_WHEN_A_ROOT_TAKING_THE_CURRENT_ACCOUNT_IS_NOT_A_ROLLBACK,
+        "挂载那一刻两块盘隔离的槽数与 tri 之前相同（D28（挂载期承诺量） 已定项 1 第九项）"
+    );
+}
+
+/// [`a_root_taking_the_current_account_that_is_not_a_rollback_takes_nothing_released_and_isolates_as_before_at_the_mount`] 那一形挂载那一刻两块盘隔离的槽数（tri 之前量的）。
+const ISOLATED_SLOTS_AT_THE_MOUNT_WHEN_A_ROOT_TAKING_THE_CURRENT_ACCOUNT_IS_NOT_A_ROLLBACK: u64 =
+    18;
````

### 第十二块 mutations-append.tsv

````
第10z批第十二块 tri 不知道是不是回退时交空（撤掉宽护：谱系片读不出那一形复活的单元回收出去）	crates/singlefs-core/src/mount.rs	        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback => {\n            return all_released_in_the_current_account();\n        }	        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback => {\n            return Vec::new();\n        }	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount
第10z批第十二块 tri 不知道是不是回退时交空（撤掉宽护：R 写的全部单元读不出那一形复活的单元回收出去）	crates/singlefs-core/src/mount.rs	        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback => {\n            return all_released_in_the_current_account();\n        }	        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback => {\n            return Vec::new();\n        }	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount
第10z批第十二块 tri 读不出的打包单元当读得出的非谱系片（认成肯定不是回退）	crates/singlefs-core/src/mount.rs	                Err(_unreadable) => PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch,	                Err(_unreadable) => PackedUnitNamedByAPublishAsRead::Parsed {\n                    record_type: 0,\n                    records: Vec::new(),\n                },	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount
第10z批第十二块 tri 是回退而 R_old 不在环里或账读不出时交空（撤掉宽护：R_old 根槽读不出那一形）	crates/singlefs-core/src/mount.rs	        Some(_) | None => all_released_in_the_current_account(),	        Some(_) | None => Vec::new(),	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount
第10z批第十二块 tri 根环里没读出 R_old 时照窄取法交空	crates/singlefs-core/src/mount.rs	        Some(accounts) if !accounts.is_empty() => {	        Some(accounts) => {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount
第10z批第十二块 tri R_old 的账读不出时当空账（照窄取法交空）	crates/singlefs-core/src/mount.rs	                placements_referenced_by_root(devices, unit_area_start, rollback_target)\n            })\n            .collect();	                Some(placements_referenced_by_root(devices, unit_area_start, rollback_target).unwrap_or_default())\n            })\n            .collect();	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount
第10z批第十二块 tri 肯定不是回退时也宽护（R 不是回退那一形多并现行账里已释放的）	crates/singlefs-core/src/mount.rs	        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback => return Vec::new(),	        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::NotARollback => {\n            return all_released_in_the_current_account()\n        }	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	a_root_taking_the_current_account_that_is_not_a_rollback_takes_nothing_released_and_isolates_as_before_at_the_mount
第10z批第十二块 K5 E3 认出回退就并现行账里全部已释放的（正推对照臂）	crates/singlefs-core/src/mount.rs	        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(rollback_target_txg) => {\n            rollback_target_txg\n        }	        RollbackRecognizedAmongThePackedUnitsNamedByAPublish::RollbackTo(_rollback_target_txg) => {\n            return all_released_in_the_current_account();\n        }	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	a_rollback_recognized_at_the_mount_takes_among_the_released_only_what_its_target_still_allocated
第10z批第十二块 K5 E1 回退目标按全部有效根找（不限 txg = T_old）	crates/singlefs-core/src/mount.rs	            .filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)	            .filter(|root| !is_abandoned(root))	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	a_rollback_recognized_at_the_mount_takes_among_the_released_only_what_its_target_still_allocated
第10z批第十二块 K5 E2 认回退时不比 rollback_txg（别的发布的谱系记录也认）	crates/singlefs-core/src/mount.rs	                        Some(lineage) if lineage.rollback_txg == checkpoint_txg_of_the_publish => {	                        Some(lineage) if lineage.rollback_txg == lineage.rollback_txg => {	-p singlefs-core --lib -- rollback_recognized_among_the_packed_units_named_by_a_publish_tests	lineage_records_of_other_publishes_only_are_not_a_rollback
第10z批第十二块 tri 认回退交回的是 T_rb 不是 T_old	crates/singlefs-core/src/mount.rs	                                lineage.rollback_target_txg,	                                lineage.rollback_txg,	-p singlefs-core --lib -- rollback_recognized_among_the_packed_units_named_by_a_publish_tests	a_lineage_record_of_this_publish_recognizes_a_rollback_to_its_target
第10z批第十二块 tri 不是谱系片的打包单元里的记录也当谱系记录读	crates/singlefs-core/src/mount.rs	                if *record_type != crate::unit::PACKED_TYPE_LINEAGE {\n                    continue;\n                }\n		-p singlefs-core --lib -- rollback_recognized_among_the_packed_units_named_by_a_publish_tests	records_of_a_packed_unit_that_is_not_a_lineage_unit_are_not_read_as_lineage_records
第10z批第十二块 tri 读不出的那一片不算不知道	crates/singlefs-core/src/mount.rs	                some_named_packed_unit_unknown = true;	                some_named_packed_unit_unknown = false;	-p singlefs-core --lib -- rollback_recognized_among_the_packed_units_named_by_a_publish_tests	an_unreadable_named_packed_unit_without_a_recognized_rollback_is_unknown
第10z批第十二块 tri 解不开的那一片不算不知道	crates/singlefs-core/src/mount.rs	                some_named_packed_unit_unknown = true;	                some_named_packed_unit_unknown = false;	-p singlefs-core --lib -- rollback_recognized_among_the_packed_units_named_by_a_publish_tests	an_unparsable_named_packed_unit_without_a_recognized_rollback_is_unknown
第10z批第十二块 tri 谱系片里解不开的记录不算不知道	crates/singlefs-core/src/mount.rs	                        None => some_named_packed_unit_unknown = true,	                        None => {}	-p singlefs-core --lib -- rollback_recognized_among_the_packed_units_named_by_a_publish_tests	an_unparsable_lineage_record_without_a_recognized_rollback_is_unknown
第10z批第十二块 tri 有一片读不出就当不知道、不再看别的几片认没认出回退	crates/singlefs-core/src/mount.rs	                some_named_packed_unit_unknown = true;	                return RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback;	-p singlefs-core --lib -- rollback_recognized_among_the_packed_units_named_by_a_publish_tests	a_recognized_rollback_stands_though_another_named_packed_unit_is_unreadable
第10z批第十二块 tri 一片打包单元都没点名时当不知道	crates/singlefs-core/src/mount.rs	    if some_named_packed_unit_unknown {	    if some_named_packed_unit_unknown || packed_units_as_read.is_empty() {	-p singlefs-core --lib -- rollback_recognized_among_the_packed_units_named_by_a_publish_tests	no_named_packed_unit_is_not_a_rollback
第10z批第十二块 K5 E7 丙并中间每个走丙的根都当成同一条根（只并交进来的最后一条）	crates/singlefs-core/src/mount.rs	    for newer_root_txg in txgs_of_the_roots_taking_the_current_account {	    for newer_root_txg in txgs_of_the_roots_taking_the_current_account.iter().rev().take(1) {	-p singlefs-core --lib -- placements_of_the_publishes_in_between_chained_onto_the_newer_roots_tests	every_root_taking_the_current_account_takes_what_the_publishes_between_it_and_the_selected_version_named
````

### 第十二块 mutations-replacements.tsv

````
第10z批第十块 回退复活 认回退时谱系记录按 txg 0 找（认不出回退：丙那一支不并回退目标账里仍分配、现行账里已释放的落点）	crates/singlefs-core/src/mount.rs	                        Some(lineage) if lineage.rollback_txg == checkpoint_txg_of_the_publish => {	                        Some(lineage) if lineage.rollback_txg == CheckpointTxg(0) => {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount
第10z批第十块 回退复活 回退目标账里取现行账里仍分配的、不取已释放的（复活的那几个滤掉）	crates/singlefs-core/src/mount.rs	                    released_in_the_current_account.contains(&(*device, *slot))	                    !released_in_the_current_account.contains(&(*device, *slot))	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount
第10z批第十块 回退复活 打包单元只看非谱系的那几片（谱系片里那条记录看不到）	crates/singlefs-core/src/mount.rs	                if *record_type != crate::unit::PACKED_TYPE_LINEAGE {	                if *record_type == crate::unit::PACKED_TYPE_LINEAGE {	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	a_rollback_recognized_at_the_mount_takes_among_the_released_only_what_its_target_still_allocated
第10z批第十块 回退复活 回退目标按被抛弃的根找（R_old 按实例表有效，找不到）	crates/singlefs-core/src/mount.rs	            .filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)	            .filter(|root| is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table	a_rollback_recognized_at_the_mount_takes_among_the_released_only_what_its_target_still_allocated
````

