# 根环实现第 10z 批第十四、十五块：代码轮第一轮改动全文（2026-10-08）

两块实现员各交的 `crates.patch` 原样，按合入次序：第十五块、第十四块（第十五块打在第十二、十三块合入之后，第十四块打在第十五块合入之后）。各块的变异表追加、整行替换附在各自后面。

## 第十五块

````diff
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
index 22cfc19..229ecd5 100644
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -28,8 +28,10 @@
 //!   或同一台设备上另一笔借据等成了），那台设备记成丢了、或放掉了也销
 //!   （第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：两腿都量到等超时的那几笔累加到整张卡借不出，到设备丢了才回来；
 //!   第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K1-1：列卡之后才记成没等成的那一笔不一起销，它的活可以是 poll 之后才提交的）。
-//!   卡池借之前列卡时看到那张卡留着的设备丢了，当场告诉账、照重起那一套记成丢了（[`GpuMemoryLedger::take_in_the_devices_kept_before_booking`]），
-//!   不等借到之后走到重起（第 10z 批第十三块，第 10z 批第十、十一块代码轮判决 K4-2：那几笔让后面的借都放不下时走不到重起，卡空着却再也借不出）。
+//!   卡池借之前看到池里哪张卡留着的设备丢了，当场告诉账、照重起那一套记成丢了（[`GpuMemoryLedger::take_in_the_devices_kept_before_booking`]），
+//!   不等借到之后走到重起（第 10z 批第十三块，第 10z 批第十、十一块代码轮判决 K4-2：那几笔让后面的借都放不下时走不到重起，卡空着却再也借不出）；
+//!   那张卡上没有没等成的借据也一样（第 10z 批第十五块，第 10z 批第十二、十三块代码轮判决 K3-3：账不知道丢了时把丢了的设备还占着的
+//!   驱动那一份当成这个进程许出去的，多给，借走之后重起时被拒、报本次未跑）。
 //!   读不到实际占着的（`nvidia-smi` 没列这个进程），或这个进程在这张卡上另有不走这份账的设备（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]，
 //!   分不出哪一份是借据建的），借着的当一点都没建、驱动那一份也当还没占上卡（第 10y 批代码轮判决 K1-1 攻方乙附：当它整份在卡上，实占的 x 比 D 小时
 //!   多给 D − x），只会少借。这张卡上起好过的设备丢了、账照重起记了之后，到丢了的那几台都放掉之前也这样算
@@ -1426,20 +1428,23 @@ impl GpuMemoryLedger {
             .collect()
     }
 
-    /// 卡池借之前那一步（`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::GpuCardPool` 借之前只调它）：列卡
-    /// （[`Self::leases_kept_after_a_failed_wait_on_each_card`]），对每张列出来的卡取卡池那一格留着的设备（`device_kept_on_the_card`，
+    /// 卡池借之前那一步（`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::GpuCardPool` 借之前只调它）：先列卡
+    /// （[`Self::leases_kept_after_a_failed_wait_on_each_card`]），再对池里每张卡取卡池那一格留着的设备（`device_kept_on_the_card`，
     /// 不等正在起的设备；没有的跳过）：
     /// - 丢了（`is_lost`）：不 poll（wgpu 在丢了的设备上 poll 会 panic），告诉账这台设备丢了，照重起那一套记
     ///   （[`DeviceStartedOnTheBooks::take_in_that_the_device_was_found_lost`]：它上面没等成的那几笔销账、这张卡照分不出算）。
     ///   不等借到之后走到重起才记：那几笔记着的量让后面的借都放不下时，借不到就走不到重起，这张卡在这个进程里再也借不出
-    ///   （第 10z 批第十三块，第 10z 批第十、十一块代码轮判决 K4-2，正推账层探针：丢了之后能借的 1 000、借 4 000 被拒）；
-    /// - 没丢：poll 一次（`found_idle_by_a_poll`，不等），交回队列空了只销列出来的那几笔
-    ///   （[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]）。
+    ///   （第 10z 批第十三块，第 10z 批第十、十一块代码轮判决 K4-2，正推账层探针：丢了之后能借的 1 000、借 4 000 被拒）。
+    ///   没有没等成借据的卡（列不出来）也看：丢了的设备的驱动那一份与缓冲还占着卡，账不知道丢了时照读数把它们算成这个进程许出去的、
+    ///   多给，借走之后重起时照此刻再判被拒、报本次未跑（第 10z 批第十五块，第 10z 批第十二、十三块代码轮判决 K3-3，正推 T4：
+    ///   能借的 29 550、卡上真能再给的 28 950）；
+    /// - 没丢、这张卡列出来了：poll 一次（`found_idle_by_a_poll`，不等），交回队列空了只销列出来的那几笔
+    ///   （[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]）；没列出来的不 poll。
     ///
     /// 不在账的锁里 poll；`on_the_books` 交那台设备在账上的那一台。
     ///
     /// # Panics
-    /// `on_the_books` 交的那一台不在这本账、或不在列出来的那张卡上（卡池那一格接错了卡）。
+    /// `on_the_books` 交的那一台不在这本账、或不在取它的那张卡上（卡池那一格接错了卡）。
     pub fn take_in_the_devices_kept_before_booking<Device>(
         &self,
         device_kept_on_the_card: impl Fn(usize) -> Option<Device>,
@@ -1447,24 +1452,31 @@ impl GpuMemoryLedger {
         found_idle_by_a_poll: impl Fn(&Device) -> bool,
         on_the_books: impl Fn(&Device) -> &DeviceStartedOnTheBooks<'_>,
     ) {
-        for listed in self.leases_kept_after_a_failed_wait_on_each_card() {
-            let Some(kept) = device_kept_on_the_card(listed.card_position()) else {
+        let listed_cards = self.leases_kept_after_a_failed_wait_on_each_card();
+        let card_count = self.books_locked().len();
+        for card_position in 0..card_count {
+            let Some(kept) = device_kept_on_the_card(card_position) else {
                 continue;
             };
             let kept_on_the_books = on_the_books(&kept);
             assert!(
                 std::ptr::eq(kept_on_the_books.ledger, self)
-                    && kept_on_the_books.card_position == listed.card_position(),
-                "列的是第 {} 张卡，卡池那一格留着的设备在账上记在第 {} 张卡上（或别的账上）",
-                listed.card_position(),
+                    && kept_on_the_books.card_position == card_position,
+                "取的是第 {card_position} 张卡那一格留着的设备，它在账上记在第 {} 张卡上（或别的账上）",
                 kept_on_the_books.card_position
             );
             if is_lost(&kept) {
                 kept_on_the_books.take_in_that_the_device_was_found_lost();
                 continue;
             }
+            let Some(listed) = listed_cards
+                .iter()
+                .find(|listed| listed.card_position == card_position)
+            else {
+                continue;
+            };
             if found_idle_by_a_poll(&kept) {
-                kept_on_the_books.take_in_that_the_work_of_the_listed_leases_finished(&listed);
+                kept_on_the_books.take_in_that_the_work_of_the_listed_leases_finished(listed);
             }
         }
     }
@@ -5634,6 +5646,524 @@ mod tests {
         drop(device_on_the_second_card);
     }
 
+    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4，正推 T1 拦 Z8：借之前没丢的那一支 poll 之后销这张卡此刻全部没等成的、
+    /// 不只列出来的，照绿）：poll 的当口（不在账的锁里）又有一笔借据记成没等成；poll 交回队列空了：列出来的那一笔销账，
+    /// 列卡之后才记成没等成的那一笔照旧记着（它的活可以是 poll 之后才提交的）。
+    #[test]
+    fn a_lease_kept_after_the_listing_survives_the_poll_before_booking() {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let (device, listed_lease, _kept_mebibytes) =
+            a_started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
+        let lease_kept_during_the_poll = std::cell::Cell::new(None);
+        ledger.take_in_the_devices_kept_before_booking(
+            |card_position| (card_position == 0).then_some(&device),
+            |_kept| false,
+            |kept| {
+                let mut lease = ledger
+                    .book_on_the_card(0, 500)
+                    .expect("还剩 1 000：借得到 500");
+                lease.take_in_the_wait_before_returning(
+                    kept,
+                    DeviceWaitBeforeReturningTheLease::TheWaitFailed,
+                );
+                lease_kept_during_the_poll.set(Some(lease.lease_identifier()));
+                drop(lease);
+                true
+            },
+            |kept| *kept,
+        );
+        assert!(
+            !ledger.is_the_lease_on_the_books(0, listed_lease),
+            "列出来的那一笔：poll 交回队列空了，销账"
+        );
+        assert!(
+            ledger.is_the_lease_on_the_books(
+                0,
+                lease_kept_during_the_poll
+                    .get()
+                    .expect("poll 的当口记了一笔没等成的")
+            ),
+            "列卡之后才记成没等成的那一笔照旧记着"
+        );
+        drop(device);
+    }
+
+    /// 第 10z 批第十五块（K4，正推 T2 拦 Z6：告诉账丢了时在用的那一台不进丢了还没放掉的那一堆，照绿）：借之前告诉账设备丢了、
+    /// 之后重起好了，丢了的那一台连它的缓冲与驱动那一份还占着卡（还没放掉）：能借的不超过卡上真能再给的。
+    #[test]
+    fn the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again() {
+        let (card, ledger) = ledger_on_a_card_under_the_ledger();
+        let (device, _kept_lease, kept_mebibytes) =
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
+        ledger
+            .prepare_the_driver_share_before_starting_the_device(0)
+            .expect("重起：驱动那一份放得下");
+        let replacement = ledger.record_the_device_started_on_the_card(0);
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = true;
+            card.driver_occupied_mebibytes = 600;
+        }
+        let room = room_now(&ledger);
+        let room_truly_left_on_the_card = room_truly_left(&card, 0);
+        assert!(
+            room <= room_truly_left_on_the_card,
+            "丢了的那一台还在卡上：能借的 {room} 不超过卡上真能再给的 {room_truly_left_on_the_card}"
+        );
+        drop(replacement);
+        drop(device);
+    }
+
+    /// 第 10z 批第十五块（K4，正推 T3 拦 Z1：取留着的设备缺了的那张卡就停、不看后面的卡，照绿）：第 0 张卡那一格没有留着的设备
+    /// （正在起），第 1 张卡的设备队列空了：第 1 张卡上那一笔照样销账，第 0 张卡上那一笔照旧记着。
+    #[test]
+    fn a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease(
+    ) {
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
+        ledger.take_in_the_devices_kept_before_booking(
+            |card_position| (card_position == 1).then(|| &devices[1]),
+            |_kept| false,
+            |_kept| true,
+            |kept| *kept,
+        );
+        assert!(
+            ledger.is_the_lease_on_the_books(0, kept_leases[0]),
+            "第 0 张卡那一格没有留着的设备可 poll：那一笔照旧记着"
+        );
+        assert!(
+            !ledger.is_the_lease_on_the_books(1, kept_leases[1]),
+            "第 1 张卡的设备队列空了：那一笔销账"
+        );
+    }
+
+    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K3-3，正推 T4）：设备起好（驱动实占 600）、头一笔等完设备还了，之后设备丢了，
+    /// 它的驱动那一份还占着卡，账上没有没等成的借据（列卡列不出这张卡）。账还不知道丢了时照读数把那 600 当成这个进程许出去的：
+    /// 能借的 29 550、卡上真能再给的 28 950，借走之后重起被拒。借之前那一步看池里每张卡：丢了的照重起记，能借的就是卡上真能再给的，
+    /// 借走它之后重起放得下。
+    #[test]
+    fn a_device_lost_without_a_kept_lease_is_taken_in_before_booking_so_the_room_lent_leaves_space_for_the_restart(
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
+        {
+            let mut card = card.lock().expect("假卡不 panic");
+            card.device_started = false;
+            card.driver_occupied_mebibytes = 0;
+            card.left_by_a_lost_device_mebibytes = 600;
+        }
+        assert_eq!(
+            room_now(&ledger),
+            29_550,
+            "账还不知道设备丢了：把丢了的设备还占着的驱动那一份 600 当成许出去的（正推 T4 room_now）"
+        );
+        assert_eq!(
+            room_truly_left(&card, 0),
+            28_950,
+            "卡上真能再给的（正推 T4 truly_left）"
+        );
+        let losses_asked = AtomicUsize::new(0);
+        ledger.take_in_the_devices_kept_before_booking(
+            |card_position| (card_position == 0).then_some(&device),
+            |_kept| {
+                losses_asked.fetch_add(1, Ordering::SeqCst);
+                true
+            },
+            |_kept| panic!("丢了的设备不 poll"),
+            |kept| *kept,
+        );
+        assert_eq!(
+            losses_asked.load(Ordering::SeqCst),
+            1,
+            "没有没等成借据的卡也问一次丢没丢"
+        );
+        let room = room_now(&ledger);
+        assert_eq!(
+            room,
+            room_truly_left(&card, 0),
+            "借之前账就知道丢了：能借的就是卡上真能再给的（不多给，也不少给）"
+        );
+        let booking = ledger
+            .book_on_the_card(0, room)
+            .expect("能借的那么多借得到");
+        assert_eq!(
+            ledger.prepare_the_driver_share_before_starting_the_device(0),
+            Ok(()),
+            "借走能借的全部之后重起：驱动那一份照此刻放得下"
+        );
+        let replacement = ledger.record_the_device_started_on_the_card(0);
+        drop(booking);
+        drop(replacement);
+        drop(device);
+    }
+
+    /// 第 10z 批第十五块（K3-3 那一改：借之前看池里每张卡，只 poll 列出来的）：第 0 张卡的设备起着、没丢、没有没等成的借据，
+    /// 第 1 张卡上一笔没等成：只 poll 第 1 张卡，那一笔销账，第 0 张卡的账不动。
+    #[test]
+    fn a_card_without_a_kept_lease_is_not_polled_before_booking() {
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
+        let devices = [&device_on_the_first_card, &device_on_the_second_card];
+        let rooms_before = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出");
+        let polls_on_each_card = [AtomicUsize::new(0), AtomicUsize::new(0)];
+        ledger.take_in_the_devices_kept_before_booking(
+            |card_position| devices.get(card_position).copied(),
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
+            vec![0, 1],
+            "只 poll 列出来的那张卡"
+        );
+        assert!(
+            !ledger.is_the_lease_on_the_books(1, kept_on_the_second_card),
+            "第 1 张卡的设备队列空了：那一笔销账"
+        );
+        assert_eq!(
+            ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出"),
+            rooms_before,
+            "第 0 张卡的账不动"
+        );
+        drop(booking_on_the_first_card);
+        drop(device_on_the_first_card);
+        drop(device_on_the_second_card);
+    }
+
+    /// 随机操作串里的一步（[`random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss`]）。
+    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
+    enum RandomStepOnTheCard {
+        /// 借 1 000、4 000、9 000 或 15 000（借不到就不借）。
+        Book,
+        /// 设备没丢时，把头一笔没建的借据建满。
+        BuildTheFirstUnbuiltLease,
+        /// 随机丢一笔借据：设备没丢就等成（它与之前没等成的缓冲都交回驱动），丢了就没等成。
+        DropALeaseAfterTheWait,
+        /// 随机丢一笔借据，没等成（建过的缓冲留在卡上）。
+        DropALeaseAfterAFailedWait,
+        /// 设备丢了：它的驱动那一份与卡上的缓冲都成了丢了的设备留在卡上的。
+        LoseTheDevice,
+        /// 卡池借之前那一步（设备丢了就交丢了，没丢的 poll 交还有在飞的）。
+        StepBeforeBooking,
+        /// 设备丢了就重起（驱动那一份照此刻判，放得下才起）。
+        RestartTheLostDevice,
+        /// 放掉最早的那一台丢了的设备（它留在卡上的都交回驱动要等最后一台放掉）。
+        ReleaseTheEarliestLostDevice,
+    }
+
+    /// 随机操作串每一步从这里按下标取（与正推腿 T5 的操作号同序：0 借、1 建、2 等成丢、3 没等成丢、4 丢设备、5 借之前那一步、6 重起、7 放掉）。
+    const RANDOM_STEPS_ON_THE_CARD: [RandomStepOnTheCard; 8] = [
+        RandomStepOnTheCard::Book,
+        RandomStepOnTheCard::BuildTheFirstUnbuiltLease,
+        RandomStepOnTheCard::DropALeaseAfterTheWait,
+        RandomStepOnTheCard::DropALeaseAfterAFailedWait,
+        RandomStepOnTheCard::LoseTheDevice,
+        RandomStepOnTheCard::StepBeforeBooking,
+        RandomStepOnTheCard::RestartTheLostDevice,
+        RandomStepOnTheCard::ReleaseTheEarliestLostDevice,
+    ];
+
+    /// 随机操作串借的那几种量（MiB）。
+    const RANDOM_BOOKING_SIZES_MEBIBYTES: [u64; 4] = [1_000, 4_000, 9_000, 15_000];
+
+    /// 线性同余的随机数（正推腿 T5 那一个，同一个种子出同一串）：种子先乘加混一次，之后每取一次乘加一次、取高 31 位。
+    struct OperationStringGenerator {
+        state: u64,
+    }
+
+    impl OperationStringGenerator {
+        const SEED_MULTIPLIER: u64 = 2_862_933_555_777_941_757;
+        const SEED_INCREMENT: u64 = 3_037_000_493;
+        const STEP_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
+        const STEP_INCREMENT: u64 = 1_442_695_040_888_963_407;
+        const DISCARDED_LOW_BITS: u32 = 33;
+
+        fn from_seed(seed: u64) -> Self {
+            Self {
+                state: seed
+                    .wrapping_mul(Self::SEED_MULTIPLIER)
+                    .wrapping_add(Self::SEED_INCREMENT),
+            }
+        }
+
+        /// `0..bound` 里的一个（`bound` 不为 0）。
+        fn next_below(&mut self, bound: usize) -> usize {
+            self.state = self
+                .state
+                .wrapping_mul(Self::STEP_MULTIPLIER)
+                .wrapping_add(Self::STEP_INCREMENT);
+            let bound_as_u64 = u64::try_from(bound).expect("下标的上界装得进 u64");
+            usize::try_from((self.state >> Self::DISCARDED_LOW_BITS) % bound_as_u64)
+                .expect("比 bound 小，装得进 usize")
+        }
+    }
+
+    /// 随机操作串里一笔活着的借据建到哪一步。
+    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
+    enum LeaseBuiltOnTheCard {
+        NotBuilt,
+        BuiltOnTheDeviceInUse,
+        /// 建在后来丢了的那一台上（缓冲已经算进丢了的设备留在卡上的）。
+        BuiltOnALostDevice,
+    }
+
+    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4，正推 T5：随机操作串拦「告诉账丢了只销那几笔、不照重起记」与 Z6；
+    /// K3-3 改了之后借之前那一步看见丢了就记）：一张假卡、一台设备，4 000 个种子各 14 步随机的借、建、丢借据（等成或没等成）、设备丢了、
+    /// 借之前那一步、重起、放掉旧设备。设备没丢、或丢了而账已经知道（丢了之后走过借之前那一步，或重起那一步判过）的每一步之后：
+    /// 能借的不超过卡上真能再给的（建过没还的缓冲、丢了的设备留在卡上的都不在空闲里，活着而没建的借据还要建）。
+    #[test]
+    fn random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss(
+    ) {
+        const SEEDS: u64 = 4_000;
+        const STEPS_IN_A_STRING: usize = 14;
+        const DRIVER_HOLDS_ONCE_STARTED: u64 = 600;
+        let mut states_checked = 0usize;
+        let mut first_overgiving_strings: Vec<String> = Vec::new();
+        let mut overgiving_states = 0usize;
+        for seed in 0..SEEDS {
+            let mut generator = OperationStringGenerator::from_seed(seed);
+            let (card, ledger) = ledger_on_a_card_under_the_ledger();
+            let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
+            let first_device = ledger.record_the_device_started_on_the_card(0);
+            {
+                let mut card = card.lock().expect("假卡不 panic");
+                card.device_started = true;
+                card.driver_occupied_mebibytes = DRIVER_HOLDS_ONCE_STARTED;
+            }
+            first.take_in_the_wait_before_returning(
+                &first_device,
+                DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
+            );
+            drop(first);
+            let mut device_in_use: std::rc::Rc<DeviceStartedOnTheBooks<'_>> =
+                std::rc::Rc::new(first_device);
+            let mut lost_devices_kept: Vec<std::rc::Rc<DeviceStartedOnTheBooks<'_>>> = Vec::new();
+            let mut alive: Vec<(GpuMemoryBooking<'_>, LeaseBuiltOnTheCard, u64)> = Vec::new();
+            let mut is_the_device_lost = false;
+            let mut does_the_ledger_know_of_the_loss = false;
+            let mut built_by_leases_dropped_after_a_failed_wait_mebibytes: u64 = 0;
+            let mut steps_taken = String::new();
+            for _step_in_the_string in 0..STEPS_IN_A_STRING {
+                let step =
+                    RANDOM_STEPS_ON_THE_CARD[generator.next_below(RANDOM_STEPS_ON_THE_CARD.len())];
+                match step {
+                    RandomStepOnTheCard::Book => {
+                        let mebibytes = RANDOM_BOOKING_SIZES_MEBIBYTES
+                            [generator.next_below(RANDOM_BOOKING_SIZES_MEBIBYTES.len())];
+                        if let Ok(booking) = ledger.book_on_the_card(0, mebibytes) {
+                            alive.push((booking, LeaseBuiltOnTheCard::NotBuilt, mebibytes));
+                            steps_taken.push_str(&format!("借{mebibytes} "));
+                        }
+                    }
+                    RandomStepOnTheCard::BuildTheFirstUnbuiltLease => {
+                        if !is_the_device_lost {
+                            if let Some(unbuilt) =
+                                alive.iter_mut().find(|(_booking, built, _mebibytes)| {
+                                    *built == LeaseBuiltOnTheCard::NotBuilt
+                                })
+                            {
+                                card.lock().expect("假卡不 panic").built_mebibytes += unbuilt.2;
+                                unbuilt.1 = LeaseBuiltOnTheCard::BuiltOnTheDeviceInUse;
+                                steps_taken.push_str("建 ");
+                            }
+                        }
+                    }
+                    RandomStepOnTheCard::DropALeaseAfterTheWait
+                    | RandomStepOnTheCard::DropALeaseAfterAFailedWait => {
+                        if !alive.is_empty() {
+                            let (mut booking, built, mebibytes) =
+                                alive.remove(generator.next_below(alive.len()));
+                            let is_the_wait_finished = step
+                                == RandomStepOnTheCard::DropALeaseAfterTheWait
+                                && !is_the_device_lost;
+                            booking.take_in_the_wait_before_returning(
+                                &device_in_use,
+                                if is_the_wait_finished {
+                                    DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished
+                                } else {
+                                    DeviceWaitBeforeReturningTheLease::TheWaitFailed
+                                },
+                            );
+                            let built_on_the_device_in_use =
+                                if built == LeaseBuiltOnTheCard::BuiltOnTheDeviceInUse {
+                                    mebibytes
+                                } else {
+                                    0
+                                };
+                            if is_the_wait_finished {
+                                card.lock().expect("假卡不 panic").built_mebibytes -=
+                                    built_by_leases_dropped_after_a_failed_wait_mebibytes
+                                        + built_on_the_device_in_use;
+                                built_by_leases_dropped_after_a_failed_wait_mebibytes = 0;
+                            } else {
+                                built_by_leases_dropped_after_a_failed_wait_mebibytes +=
+                                    built_on_the_device_in_use;
+                            }
+                            drop(booking);
+                            steps_taken.push_str(if is_the_wait_finished {
+                                "等成丢 "
+                            } else {
+                                "没等成丢 "
+                            });
+                        }
+                    }
+                    RandomStepOnTheCard::LoseTheDevice => {
+                        if !is_the_device_lost {
+                            is_the_device_lost = true;
+                            does_the_ledger_know_of_the_loss = false;
+                            let mut card = card.lock().expect("假卡不 panic");
+                            card.left_by_a_lost_device_mebibytes +=
+                                card.driver_occupied_mebibytes + card.built_mebibytes;
+                            card.built_mebibytes = 0;
+                            card.driver_occupied_mebibytes = 0;
+                            card.device_started = false;
+                            for (_booking, built, _mebibytes) in &mut alive {
+                                if *built == LeaseBuiltOnTheCard::BuiltOnTheDeviceInUse {
+                                    *built = LeaseBuiltOnTheCard::BuiltOnALostDevice;
+                                }
+                            }
+                            built_by_leases_dropped_after_a_failed_wait_mebibytes = 0;
+                            steps_taken.push_str("丢设备 ");
+                        }
+                    }
+                    RandomStepOnTheCard::StepBeforeBooking => {
+                        ledger.take_in_the_devices_kept_before_booking(
+                            |_card_position| Some(std::rc::Rc::clone(&device_in_use)),
+                            |_kept| is_the_device_lost,
+                            |_kept| false,
+                            |kept| &**kept,
+                        );
+                        if is_the_device_lost {
+                            does_the_ledger_know_of_the_loss = true;
+                        }
+                        steps_taken.push_str("借之前 ");
+                    }
+                    RandomStepOnTheCard::RestartTheLostDevice => {
+                        if is_the_device_lost {
+                            does_the_ledger_know_of_the_loss = true;
+                            if ledger
+                                .prepare_the_driver_share_before_starting_the_device(0)
+                                .is_ok()
+                            {
+                                let replacement = ledger.record_the_device_started_on_the_card(0);
+                                lost_devices_kept.push(std::mem::replace(
+                                    &mut device_in_use,
+                                    std::rc::Rc::new(replacement),
+                                ));
+                                is_the_device_lost = false;
+                                let mut card = card.lock().expect("假卡不 panic");
+                                card.device_started = true;
+                                card.driver_occupied_mebibytes = DRIVER_HOLDS_ONCE_STARTED;
+                                steps_taken.push_str("重起 ");
+                            } else {
+                                steps_taken.push_str("重起被拒 ");
+                            }
+                        }
+                    }
+                    RandomStepOnTheCard::ReleaseTheEarliestLostDevice => {
+                        if !lost_devices_kept.is_empty() {
+                            drop(lost_devices_kept.remove(0));
+                            if lost_devices_kept.is_empty() {
+                                card.lock()
+                                    .expect("假卡不 panic")
+                                    .left_by_a_lost_device_mebibytes = 0;
+                            }
+                            steps_taken.push_str("放掉 ");
+                        }
+                    }
+                }
+                let unbuilt_mebibytes: u64 = alive
+                    .iter()
+                    .filter(|(_booking, built, _mebibytes)| *built == LeaseBuiltOnTheCard::NotBuilt)
+                    .map(|(_booking, _built, mebibytes)| *mebibytes)
+                    .sum();
+                if !is_the_device_lost || does_the_ledger_know_of_the_loss {
+                    states_checked += 1;
+                    let room = room_now(&ledger);
+                    let room_truly_left_on_the_card = room_truly_left(&card, unbuilt_mebibytes);
+                    if room > room_truly_left_on_the_card {
+                        overgiving_states += 1;
+                        if first_overgiving_strings.len() < 5 {
+                            first_overgiving_strings.push(format!(
+                                "种子 {seed}：{steps_taken}能借 {room}，卡上真能再给 {room_truly_left_on_the_card}"
+                            ));
+                        }
+                    }
+                }
+            }
+            drop(alive);
+            drop(lost_devices_kept);
+            drop(device_in_use);
+        }
+        assert!(states_checked > 0, "随机串里至少有一步落在账知道的状态上");
+        assert_eq!(
+            overgiving_states, 0,
+            "核了 {states_checked} 步，能借的都不超过卡上真能再给的；头几条多给的：{first_overgiving_strings:#?}"
+        );
+    }
+
     /// 一台假设备：最后一个克隆丢掉时（wgpu 收设备那一下要等设备空了）先报一声、等用例放开才收完；`number` 认是哪一台。
     struct DeviceWhoseLastDropWaits {
         number: u32,
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index 7a18528..9b2dbd7 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -2401,9 +2401,18 @@ mod tests {
     }
 
     /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K4-2、K5：借之前那一步里面没有用例，丢了的设备到借到之后的重起才告诉账）：
-    /// 借之前那一步整个交给 harness 档的 `GpuMemoryLedger::take_in_the_devices_kept_before_booking`（列卡、丢了的告诉账不 poll、没丢的 poll、
-    /// 只销列出来的，都在那边不要卡地测），卡池这一侧只接四样：列出来那一格留着的设备（不等正在起的）、丢没丢照设备的情况、poll 一次不等
-    /// 且照 [`device_found_idle_by_a_poll`] 判、账上的那一台。扫这份源码（测试那一段之外、去掉注释行与空白），不要卡。
+    /// 借之前那一步整个交给 harness 档的 `GpuMemoryLedger::take_in_the_devices_kept_before_booking`（看池里每张卡、丢了的告诉账不 poll、
+    /// 列出来的 poll、只销列出来的，都在那边不要卡地测，`singlefs_harness::gpu_memory_ledger` 的库单测），卡池这一侧只接四样：那一格留着的设备
+    /// （不等正在起的）、丢没丢照设备的情况、poll 一次不等且照 [`device_found_idle_by_a_poll`] 判、账上的那一台。这四样要真卡上起好的设备才走得到，
+    /// 换不成不要卡的行为用例，这里扫这份源码（测试那一段之外、去掉注释行与空白）：方法体整个等于钉住的那一句。不要卡。
+    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4）：原先只数几段文本各出现一处、闭包三个，被这几种改了行为而编得过的写法骗过：
+    /// 方法体最前面 `if true { return; }`（正推 A2）、丢没丢与 poll 两个同型闭包对调（正推 A3）、整个包进 `if false { … }`（攻方 gpu-2）、
+    /// 方法体里另起一个同名的 `device_found_idle_by_a_poll` 恒交真遮住外面那个（攻方 gpu-3）；整句比之后这四种都红。
+    /// 整句比也拦不住的（文本都在这个方法体之外）：改 [`device_found_idle_by_a_poll`] 本身（由
+    /// `a_poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty` 管；`card_pool` 模块按名字从上一层引它，模块里再起一个同名的编译不过）；
+    /// 改 `GpuCard::device_condition`、`device_on_the_books` 放进去的是哪一台（`started_card`）、harness 档的 `DeviceStartedOnDemand::started_now`
+    /// 与 `DeviceConditionOfACard::is_lost`；在方法上面加 `#[cfg]` 让另一份同名方法编进来；四个入口不调这一步
+    /// （`the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first` 管，它也只看文本）。
     #[test]
     fn the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking() {
         let source = include_str!("gpu_unit_checks.rs");
@@ -2412,6 +2421,11 @@ mod tests {
             .next()
             .expect("split 至少交一段");
         let signature = "fn take_in_the_devices_found_idle(&self) \u{7b}";
+        assert_eq!(
+            code.matches(signature).count(),
+            1,
+            "源码里恰好一处 {signature}"
+        );
         let start = code
             .find(signature)
             .unwrap_or_else(|| panic!("源码里有 {signature}"));
@@ -2425,40 +2439,19 @@ mod tests {
             .flat_map(str::chars)
             .filter(|character| !character.is_whitespace())
             .collect();
-        for (expected, what) in [
-            (
-                "self.ledger.take_in_the_devices_kept_before_booking(",
-                "整个交给账的那一步",
-            ),
-            (
-                "|card_position|self.cards[card_position].started.started_now(),",
-                "列出来那一格留着的设备，不等正在起的",
-            ),
-            (
-                "|started:&GpuCard|started.device_condition().is_lost(),",
-                "丢没丢照设备的情况",
-            ),
-            (
-                "|started:&GpuCard|{device_found_idle_by_a_poll(&std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{started.device().poll(wgpu::PollType::Poll)}),))},",
-                "poll 一次不等、接住 panic、照队列空了判",
-            ),
-            (
-                "|started:&GpuCard|{started.device_on_the_books.as_deref().expect(",
-                "账上的那一台是这台设备自己的",
-            ),
-        ] {
-            assert_eq!(
-                body_without_comments_and_spaces.matches(expected).count(),
-                1,
-                "借之前那一步要有一处 {expected}（{what}）：{body_without_comments_and_spaces}"
-            );
-        }
+        let expected_body = concat!(
+            "self.ledger.take_in_the_devices_kept_before_booking(",
+            "|card_position|self.cards[card_position].started.started_now(),",
+            "|started:&GpuCard|started.device_condition().is_lost(),",
+            "|started:&GpuCard|\u{7b}device_found_idle_by_a_poll(&std::panic::catch_unwind(",
+            "std::panic::AssertUnwindSafe(||\u{7b}started.device().poll(wgpu::PollType::Poll)\u{7d}),))\u{7d},",
+            "|started:&GpuCard|\u{7b}started.device_on_the_books.as_deref().expect(",
+            "\"卡池起的设备起好时放进了账上的那一台（started_card）\")\u{7d},",
+            ");"
+        );
         assert_eq!(
-            body_without_comments_and_spaces
-                .matches("|started:&GpuCard|")
-                .count(),
-            3,
-            "取到设备之后的三样各一个闭包、不多不少：{body_without_comments_and_spaces}"
+            body_without_comments_and_spaces, expected_body,
+            "借之前那一步整个交给账、四样各接一处，次序是那一格留着的设备、丢没丢、poll、账上的那一台，此外什么都没有"
         );
     }
 
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index 726c5b7..edb1393 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -4842,12 +4842,95 @@ mod tests {
             .count()
     }
 
+    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4：扫源码的用例被领块之前那一问写成 `if false && stop_is_requested()` 骗过，
+    /// 攻方 crash-3）：四块要判，叫停在第三次被问时交真：领每一块之前各问一次、问到真就不再领，叫停之前领的两块判完写了，之后的不领，
+    /// 叫停一共被问三次。不要卡、不碰库：判一段与写一块都用假的。
+    #[test]
+    fn the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken()
+    {
+        const ASK_THAT_FINDS_THE_STOP: usize = 3;
+        let pending: Vec<(usize, Range<u64>)> = vec![(0, 0..2), (0, 2..4), (0, 4..6), (0, 6..8)];
+        let points = vec![NamesOfACrashPoint {
+            name: "一个点".to_string(),
+            path_number: "0".to_string(),
+        }];
+        let asks = std::sync::atomic::AtomicUsize::new(0);
+        let mut stored_block_starts = Vec::new();
+        let tally = check_pending_blocks_in_pieces(
+            &pending,
+            &points,
+            2,
+            &|| {
+                asks.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
+                    >= ASK_THAT_FINDS_THE_STOP
+            },
+            &|_point_index, local, ending| {
+                drop(ending);
+                JudgedBlock {
+                    items_of_each_state: vec![
+                        Vec::new();
+                        usize::try_from(local.end - local.start)
+                            .expect("一段装得进")
+                    ],
+                    local,
+                    red_texts: Vec::new(),
+                    gpu_cpu_disagreements: 0,
+                    gpu_memory_was_short: None,
+                }
+            },
+            &mut |_point_index, local, _items_of_each_state, _violating| {
+                stored_block_starts.push(local.start);
+                Ok(())
+            },
+        )
+        .expect("假的写库不失败");
+        assert_eq!(
+            asks.load(std::sync::atomic::Ordering::SeqCst),
+            ASK_THAT_FINDS_THE_STOP,
+            "领第一、二块之前各问一次（不停），领第三块之前问到停、不再问"
+        );
+        assert_eq!(
+            stored_block_starts,
+            vec![0, 2],
+            "叫停之前领的两块判完写了，之后的不领"
+        );
+        assert_eq!(tally.blocks_checked, 2);
+    }
+
     /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K5，正推：下游各层把收到的叫停换成永不叫停，`cargo check` 只报没用的变量、
-    /// 变异跑道不带 `-D warnings`，照绿）：这份源码里每个收叫停参数（`stop_is_requested: &(dyn Fn() -> bool + Sync),`，测试那一段之外）的函数，
-    /// 参数行之后到函数收尾的顶格右括号都用到它（往下交，或当场问）；收它的是这六层，次序照源码。扫这份源码，不要卡。
+    /// 变异跑道不带 `-D warnings`，照绿）：这份源码里收叫停参数（`stop_is_requested: &(dyn Fn() -> bool + Sync),`，测试那一段之外）的函数
+    /// 照源码次序是这六层（多一层少一层都要回来看罩没罩到）。
+    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4：「参数行之后整名用上至少一次」被只提一次名字而永不叫停的写法骗过，
+    /// 正推 A7 `let _ = stop_is_requested`、攻方 crash-2 `&|| { let _ = stop_is_requested; false }`、crash-3 `if false && stop_is_requested()`）：
+    /// 不要卡的两层改由行为用例数叫停被问了几次——`check_pending_blocks_in_pieces` 由本模块
+    /// `the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken`，`check_blocks_where_until` 由
+    /// `tests/crash_amplification_pipeline_records_checks_and_reuses.rs` 的 `a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest`
+    /// （叫停在第二次被问时交真，只判了一块）。另四层换不成不要卡的行为用例：第 ① 段在卡上判那三层要卡，`check_selected_blocks` 要录好的流与库
+    /// （只有 `run_from_environment` 调它，经环境跑的那几条用例里叫停文件放上时块都判过了，问不到这一层），这里扫源码：参数行之后到函数收尾的
+    /// 顶格右括号，用到叫停整名的行（去掉首尾空白）逐行等于钉住的那几行，往下交的那一行之后紧跟右括号（是调下一层的最后一个实参）。
+    /// 整行比拦得住上面三种写法（多一行、或那一行不是原样）。拦不住的：同一行原样交下去而下一层的函数体里不用（下一层由它自己那一条管，
+    /// 第 ① 段在卡上判那一层里 `stop_request` 往下怎么用由 `the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run`
+    /// 管，也只看文本）；把整次调用换成另一个同签名的函数；在函数上面加 `#[cfg]` 让另一份同名函数编进来；宏展开出来的调用。
     #[test]
     fn every_layer_handed_the_stop_request_hands_it_on_or_asks_it() {
         let stop_parameter = concat!("stop_is_", "requested: &(dyn Fn() -> bool + Sync),");
+        let handed_on_as_the_last_argument = concat!("stop_is_", "requested,");
+        let lines_using_the_stop_request_in_the_layers_without_a_behaviour_test: [(&str, &[&str]);
+            4] = [
+            (
+                "judge_my_blocks_on_gpu_cards",
+                &[handed_on_as_the_last_argument],
+            ),
+            (
+                "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from",
+                &[concat!(
+                    "let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
+                    "requested);"
+                )],
+            ),
+            ("check_selected_blocks", &[handed_on_as_the_last_argument]),
+            ("judge_my_blocks_on_gpu", &[handed_on_as_the_last_argument]),
+        ];
         let lines = code_lines_outside_the_tests();
         let mut layers_handed_the_stop_request = Vec::new();
         for (parameter_line_index, _parameter_line) in lines
@@ -4870,14 +4953,38 @@ mod tests {
                 .iter()
                 .take_while(|line| line.as_str() != "\u{7d}")
                 .collect();
-            let uses: usize = rest_of_the_layer
-                .iter()
-                .map(|line| whole_name_uses_of_the_stop_request(line))
-                .sum();
-            assert!(
-                uses >= 1,
-                "{layer} 收了叫停却没用上（往下交、或当场问；换成永不叫停只会报没用的变量）：{rest_of_the_layer:#?}"
-            );
+            if let Some((_layer, expected_lines)) =
+                lines_using_the_stop_request_in_the_layers_without_a_behaviour_test
+                    .iter()
+                    .find(|(layer_without_a_behaviour_test, _expected_lines)| {
+                        *layer_without_a_behaviour_test == layer.as_str()
+                    })
+            {
+                let lines_using_it: Vec<(usize, &str)> = rest_of_the_layer
+                    .iter()
+                    .enumerate()
+                    .filter(|(_offset, line)| whole_name_uses_of_the_stop_request(line) > 0)
+                    .map(|(offset, line)| (offset, line.trim()))
+                    .collect();
+                assert_eq!(
+                    lines_using_it
+                        .iter()
+                        .map(|(_offset, line)| *line)
+                        .collect::<Vec<&str>>(),
+                    expected_lines.to_vec(),
+                    "{layer} 用到叫停的只有钉住的那几行、原样（只提一次名字而永不叫停的写法多一行或那一行变了）：{rest_of_the_layer:#?}"
+                );
+                for (offset, line) in &lines_using_it {
+                    if *line == handed_on_as_the_last_argument {
+                        assert!(
+                            rest_of_the_layer
+                                .get(offset + 1)
+                                .is_some_and(|next_line| next_line.trim_start().starts_with(')')),
+                            "{layer} 把叫停当调下一层的最后一个实参原样交下去：{rest_of_the_layer:#?}"
+                        );
+                    }
+                }
+            }
             layers_handed_the_stop_request.push(layer);
         }
         assert_eq!(
````

### 第十五块 mutations-append.tsv

````
第10z批第十五块 显存账 K3-3 借之前那一步只看列出来的卡（没有没等成借据的卡上设备丢了不告诉账，正推 T4）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        for card_position in 0..card_count {	        for card_position in listed_cards.iter().map(LeasesKeptAfterAFailedWaitWhenListed::card_position) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_device_lost_without_a_kept_lease_is_taken_in_before_booking_so_the_room_lent_leaves_space_for_the_restart	a_device_lost_without_a_kept_lease_is_taken_in_before_booking_so_the_room_lent_leaves_space_for_the_restart
第10z批第十五块 显存账 K3-3 随机操作串 借之前那一步只看列出来的卡	crates/singlefs-harness/src/gpu_memory_ledger.rs	        for card_position in 0..card_count {	        for card_position in listed_cards.iter().map(LeasesKeptAfterAFailedWaitWhenListed::card_position) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss	random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss
第10z批第十五块 显存账 K4 借之前取留着的设备缺了的那张卡就停（正推 Z1）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            let Some(kept) = device_kept_on_the_card(card_position) else {\n                continue;\n            };	            let Some(kept) = device_kept_on_the_card(card_position) else {\n                break;\n            };	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease	a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease
第10z批第十五块 显存账 K4 告诉账丢了时在用的那一台不进丢了还没放掉的那一堆（正推 Z6）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if books_of_the_card.device_in_use == Some(self.device) {\n            books_of_the_card.take_back_the_driver_share_of_a_lost_device();\n        }	        if books_of_the_card.device_in_use == Some(self.device) {\n            books_of_the_card.driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;\n            books_of_the_card.device_in_use = None;\n            for kept_lease in std::mem::take(&mut books_of_the_card.leases_kept_after_a_failed_wait).into_keys() {\n                books_of_the_card.leases.remove(&kept_lease);\n            }\n        }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again	the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again
第10z批第十五块 显存账 K4 借之前没丢的那一支 poll 之后销这张卡此刻全部没等成的、不只列出来的（正推 Z8）	crates/singlefs-harness/src/gpu_memory_ledger.rs	                kept_on_the_books.take_in_that_the_work_of_the_listed_leases_finished(listed);	                kept_on_the_books.take_in_that_the_work_of_the_listed_leases_finished(&self.leases_kept_after_a_failed_wait_on_each_card().into_iter().find(|listed_now| listed_now.card_position == card_position).unwrap_or_else(|| listed.clone()));	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_lease_kept_after_the_listing_survives_the_poll_before_booking	a_lease_kept_after_the_listing_survives_the_poll_before_booking
第10z批第十五块 显存账 K4 随机操作串 告诉账设备丢了只销那几笔、不照重起记	crates/singlefs-harness/src/gpu_memory_ledger.rs	        if books_of_the_card.device_in_use == Some(self.device) {\n            books_of_the_card.take_back_the_driver_share_of_a_lost_device();\n        }	        if books_of_the_card.device_in_use == Some(self.device) {\n            for kept_lease in std::mem::take(&mut books_of_the_card.leases_kept_after_a_failed_wait).into_keys() {\n                books_of_the_card.leases.remove(&kept_lease);\n            }\n        }	-p singlefs-harness --lib -- gpu_memory_ledger::tests::random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss	random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss
第10z批第十五块 显存账 K3-3 借之前那一步没列出来的卡也拿别的卡列出来的那几笔去 poll	crates/singlefs-harness/src/gpu_memory_ledger.rs	                .find(|listed| listed.card_position == card_position)	                .find(|_listed| true)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_without_a_kept_lease_is_not_polled_before_booking	a_card_without_a_kept_lease_is_not_polled_before_booking
第10z批第十五块 GPU 取卡 K4 借之前那一步方法体最前面整句返回（正推 A2）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            fn take_in_the_devices_found_idle(&self) {\n                self.ledger.take_in_the_devices_kept_before_booking(	            fn take_in_the_devices_found_idle(&self) {\n                if true { return; }\n                self.ledger.take_in_the_devices_kept_before_booking(	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking	the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking
第10z批第十五块 GPU 取卡 K4 借之前那一步丢没丢与 poll 两个闭包对调（正推 A3）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	                    |started: &GpuCard| started.device_condition().is_lost(),\n                    |started: &GpuCard| {\n                        device_found_idle_by_a_poll(&std::panic::catch_unwind(\n                            std::panic::AssertUnwindSafe(|| {\n                                started.device().poll(wgpu::PollType::Poll)\n                            }),\n                        ))\n                    },\n	                    |started: &GpuCard| {\n                        device_found_idle_by_a_poll(&std::panic::catch_unwind(\n                            std::panic::AssertUnwindSafe(|| {\n                                started.device().poll(wgpu::PollType::Poll)\n                            }),\n                        ))\n                    },\n                    |started: &GpuCard| started.device_condition().is_lost(),\n	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking	the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking
第10z批第十五块 GPU 取卡 K4 借之前那一步整个包进 if false（攻方 gpu-2）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            fn take_in_the_devices_found_idle(&self) {\n                self.ledger.take_in_the_devices_kept_before_booking(\n                    |card_position| self.cards[card_position].started.started_now(),\n                    |started: &GpuCard| started.device_condition().is_lost(),\n                    |started: &GpuCard| {\n                        device_found_idle_by_a_poll(&std::panic::catch_unwind(\n                            std::panic::AssertUnwindSafe(|| {\n                                started.device().poll(wgpu::PollType::Poll)\n                            }),\n                        ))\n                    },\n                    |started: &GpuCard| {\n                        started\n                            .device_on_the_books\n                            .as_deref()\n                            .expect("卡池起的设备起好时放进了账上的那一台（started_card）")\n                    },\n                );\n            }	            fn take_in_the_devices_found_idle(&self) {\n                if false {\n                self.ledger.take_in_the_devices_kept_before_booking(\n                    |card_position| self.cards[card_position].started.started_now(),\n                    |started: &GpuCard| started.device_condition().is_lost(),\n                    |started: &GpuCard| {\n                        device_found_idle_by_a_poll(&std::panic::catch_unwind(\n                            std::panic::AssertUnwindSafe(|| {\n                                started.device().poll(wgpu::PollType::Poll)\n                            }),\n                        ))\n                    },\n                    |started: &GpuCard| {\n                        started\n                            .device_on_the_books\n                            .as_deref()\n                            .expect("卡池起的设备起好时放进了账上的那一台（started_card）")\n                    },\n                );\n                }\n            }	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking	the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking
第10z批第十五块 GPU 取卡 K4 借之前那一步方法体里另起同名 device_found_idle_by_a_poll 恒交真（攻方 gpu-3）	crates/singlefs-checker-tier/src/gpu_unit_checks.rs	            fn take_in_the_devices_found_idle(&self) {\n                self.ledger.take_in_the_devices_kept_before_booking(	            fn take_in_the_devices_found_idle(&self) {\n                fn device_found_idle_by_a_poll<Polled>(_polled: &Polled) -> bool { true }\n                self.ledger.take_in_the_devices_kept_before_booking(	-p singlefs-checker-tier --features gpu --lib -- gpu_unit_checks::tests::the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking	the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking
第10z批第十五块 崩溃放量 K4 check_blocks_where_until 往下交永不叫停、另提一句叫停的名字（正推 A7）	crates/singlefs-checker-tier/src/crash_amplification.rs	        checking_threads(),\n        stop_is_requested,	        checking_threads(),\n        { let _ = stop_is_requested; &|| false },	-p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest	a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest
第10z批第十五块 崩溃放量 K4 check_blocks_where_until 往下交的闭包里提一次叫停的名字而永不叫停（攻方 crash-2）	crates/singlefs-checker-tier/src/crash_amplification.rs	        checking_threads(),\n        stop_is_requested,	        checking_threads(),\n        &|| { let _ = stop_is_requested; false },	-p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest	a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest
第10z批第十五块 崩溃放量 K4 check_pending_blocks_in_pieces 领块之前那一问短路成永不问（攻方 crash-3）	crates/singlefs-checker-tier/src/crash_amplification.rs	        if stop_is_requested() {\n            break;\n        }	        if false && stop_is_requested() {\n            break;\n        }	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken	the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken
第10z批第十五块 崩溃放量 K4 judge_my_blocks_on_gpu_cards 往下交的闭包里提一次叫停的名字而永不叫停（攻方 crash-2 那一形挪到要卡的一层）	crates/singlefs-checker-tier/src/crash_amplification.rs	        GpuCardsSource::DevicesStartedForThisRun,\n        stop_is_requested,	        GpuCardsSource::DevicesStartedForThisRun,\n        &|| { let _ = stop_is_requested; false },	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
第10z批第十五块 崩溃放量 K4 check_selected_blocks 往下交永不叫停、另提一句叫停的名字（正推 A7 那一形挪到要库的一层）	crates/singlefs-checker-tier/src/crash_amplification.rs	        unit_checksums,\n        version,\n        stop_is_requested,\n    )	        unit_checksums,\n        version,\n        { let _ = stop_is_requested; &|| false },\n    )	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::every_layer_handed_the_stop_request_hands_it_on_or_asks_it	every_layer_handed_the_stop_request_hands_it_on_or_asks_it
````

### 第十五块 mutations-replacements.tsv

````
第10z批第十三块 显存账 K5 借之前列卡的循环只看第一张卡（正推 Y3）	crates/singlefs-harness/src/gpu_memory_ledger.rs	        for card_position in 0..card_count {	        for card_position in 0..card_count.min(1) {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease	every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease
第10z批第十三块 显存账 K4-2 借之前那一步不核卡池那一格交的设备在不在列出来的那张卡上	crates/singlefs-harness/src/gpu_memory_ledger.rs	                std::ptr::eq(kept_on_the_books.ledger, self)\n                    && kept_on_the_books.card_position == card_position,	                std::ptr::eq(kept_on_the_books.ledger, self)\n                    || kept_on_the_books.card_position == card_position,	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_kept_device_on_the_books_of_another_card_is_refused_by_an_assertion	a_kept_device_on_the_books_of_another_card_is_refused_by_an_assertion
第10z批第十三块 崩溃放量 K5 check_blocks_where_until 往下交的叫停换成永不叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	        checking_threads(),\n        stop_is_requested,	        checking_threads(),\n        &|| false,	-p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest	a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest
第10z批第十三块 崩溃放量 K5 check_pending_blocks_in_pieces 领块之前不问叫停	crates/singlefs-checker-tier/src/crash_amplification.rs	        if stop_is_requested() {\n            break;\n        }	        if false {\n            break;\n        }	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken	the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken
````

## 第十四块

````diff
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -264,7 +264,8 @@
     free_slots: u64,
     /// 影子账（D23（journal 的角色与格式） 已定项 14 回退段与「可写挂载时的预隔离」；D28（挂载期承诺量） 已定项 1 第九项）隔离的槽，两部分置同一位：
     /// 只被被抛弃根引用的槽，隔离到那条被抛弃的根离开根环；可写挂载那一刻水位之上没施加的 journal 记录点名的落点与那几次发布的账里的落点
-    /// （C694 的 C，现行账里仍分配着的除外），隔离到这个进程写掉那条记录的 txg 对应的根环槽（`RootRingOccupancy` 里记着的那一份）。
+    /// （C694 的 C，分配位置着的槽除外：现行账里仍分配的与已释放、还在 defer 窗口里的，它们在「已分配」里；回收时补隔离），
+    /// 隔离到这个进程写掉那条记录的 txg 对应的根环槽（`RootRingOccupancy` 里记着的那一份）。
     /// 隔离位独立于分配位（[`Self::isolate`]）。只住内存、不进记账行。
     isolated: Vec<bool>,
     isolated_per_segment: Vec<u64>,
@@ -586,15 +587,52 @@
             "跨度越过单元区末尾：隔离的落点来自被抛弃根，它的槽与跨度在 recovery::placement_lies_in_the_unit_area_of_its_device / allocation_records_fit_the_pool_geometry 判过"
         );
         for index in start..end {
-            if !self.isolated[index] {
-                self.set_blocking_bit(index, SlotBlockingBit::IsolatedByTheShadowLedger, true);
-                let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
-                self.isolated_per_segment[segment] += 1;
-                self.isolated_slots += 1;
+            self.isolate_slot_at_index(index);
+        }
+    }
+
+    /// 同 [`Self::isolate`]，只隔离落点里分配位没置着的槽：分配位置着的（现行账里仍分配的，或已释放、还在 defer 窗口里的）算在
+    /// 「已分配」里，D28（挂载期承诺量） 已定项 1 第九项「不在这里再算一次」；它们回收的那一刻由分配器照还没解开的那一份补隔离
+    /// （`PoolAllocator::record_root_written_by_this_process`、`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`）。
+    /// 给水位之上没施加的记录点名的落点（C694 的 C 那一圈）用。
+    pub fn isolate_the_slots_not_allocated(&mut self, slot: SlotNumber, span: u64) {
+        let start = self.index(slot);
+        let end = start + usize::try_from(span).expect("跨度");
+        assert!(
+            end <= self.allocated.len(),
+            "跨度越过单元区末尾：C 那一圈的落点是盘上读来的记录点名的与账里的（mount::placements_named_by_unapplied_journal_records、placements_referenced_by_root 逐条经 recovery::placement_lies_in_the_unit_area_of_its_device 判过落在这块盘的单元区里），或现行账里的记录"
+        );
+        for index in start..end {
+            if !self.allocated[index] {
+                self.isolate_slot_at_index(index);
             }
         }
     }
 
+    /// 置一槽的隔离位并维护两个隔离计数；已置着就什么都不动（同一个槽隔离两次不重复计数）。
+    fn isolate_slot_at_index(&mut self, index: usize) {
+        if !self.isolated[index] {
+            self.set_blocking_bit(index, SlotBlockingBit::IsolatedByTheShadowLedger, true);
+            let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
+            self.isolated_per_segment[segment] += 1;
+            self.isolated_slots += 1;
+        }
+    }
+
+    /// 分配位与隔离位同时置着的槽数：D28（挂载期承诺量） 已定项 1 第九项与「已分配」各扣一次的那几个。逐槽扫一遍单元区，
+    /// 只给核对与用例读，运行时的判定不读它（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」）。
+    #[must_use]
+    pub fn slots_counted_both_as_allocated_and_as_isolated(&self) -> u64 {
+        u64::try_from(
+            self.allocated
+                .iter()
+                .zip(&self.isolated)
+                .filter(|(is_allocated, is_isolated)| **is_allocated && **is_isolated)
+                .count(),
+        )
+        .expect("单元区槽数装得进 u64")
+    }
+
     /// 这块盘上隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项的这块盘那一项）：只被被抛弃根引用的，加水位之上没施加的 journal 记录点名、
     /// 还没解开的（C694 的 C）；两部分置的是同一个隔离位，同一个槽只数一次。
     #[must_use]
@@ -2042,6 +2080,36 @@
             .collect()
     }
 
+    /// 同 [`Self::reclaim_released_up_to`]，回收出来的槽里隔离还护着的（环里还在的被抛弃根引用的、还没解开的水位之上没施加的记录点名的，
+    /// [`RootRingOccupancy::placements_the_isolation_still_protects`]）当场补隔离，与 [`Self::record_root_written_by_this_process`] 里回收之后那一步
+    /// 同一件事：C 那一圈的落点挂载那一刻分配位置着的不隔离（[`DeviceFreeMap::isolate_the_slots_not_allocated`]，D28（挂载期承诺量） 已定项 1
+    /// 第九项），抬 F 那一串回收它们时在这里补上。没装根环表时没有要护的，只回收。
+    pub fn reclaim_released_up_to_isolating_what_the_isolation_still_protects(
+        &mut self,
+        floor: CheckpointTxg,
+        reuse: ReclaimedReuse,
+    ) -> Vec<Placement> {
+        let first_device = self.devices[0].device;
+        let reclaimed = self.reclaim_released_records_up_to(floor, reuse);
+        let still_protected_by_the_isolation = self
+            .root_ring
+            .as_ref()
+            .map(RootRingOccupancy::placements_the_isolation_still_protects)
+            .unwrap_or_default();
+        self.isolate_reclaimed_slots_the_isolation_still_protects(
+            &reclaimed,
+            &still_protected_by_the_isolation,
+        );
+        reclaimed
+            .into_iter()
+            .filter(|record| record.device == first_device)
+            .map(|record| Placement {
+                slot: record.slot,
+                span: u64::from(record.span_slots),
+            })
+            .collect()
+    }
+
     /// `reclaim_released_up_to` 的本体，交回这次回收的每一条记录（逐盘各一条）：挂载内回收要按盘判哪几个槽还被被抛弃的根引用着。
     fn reclaim_released_records_up_to(
         &mut self,
@@ -2085,6 +2153,22 @@
         }
     }
 
+    /// 水位之上没施加的 journal 记录点名、不被现行账引用的落点（C694 的 C 那一圈）在它那块盘上隔离，只隔离分配位没置着的槽
+    /// （[`DeviceFreeMap::isolate_the_slots_not_allocated`]，D28（挂载期承诺量） 已定项 1 第九项「只除去现行账里仍分配着的槽」）。
+    pub fn isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(
+        &mut self,
+        device: DeviceIdentity,
+        slot: SlotNumber,
+        span: u64,
+    ) {
+        let device_map = self
+            .devices
+            .iter_mut()
+            .find(|device_map| device_map.device == device)
+            .expect("C 那一圈的落点的盘在池里：盘上读来的那几份逐条经 recovery::placement_lies_in_the_unit_area_of_its_device 判过这块盘在池里，现行账里的记录在 recovery::allocation_records_fit_the_pool_geometry 比过");
+        device_map.isolate_the_slots_not_allocated(slot, span);
+    }
+
     /// 回退的影子账：把一个只被被抛弃根引用的落点，或水位之上没施加的 journal 记录点名、不被候选根与现行账引用的落点（C694 的 C），
     /// 在它那块盘上隔离。
     pub fn isolate_abandoned(&mut self, device: DeviceIdentity, slot: SlotNumber, span: u64) {
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1538,7 +1538,8 @@
 /// （[`StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot`]）。候选根读不出就当它什么都不豁免（隔离只会多不会少）。
 /// 同一次也隔离水位之上没施加的 journal 记录点名的落点、那几次发布的账里的落点与比所选那一版新的可读根的账里的落点
 /// （`placements_named_by_unapplied_journal_records`，C694 的 C 连同 `C_ACCOUNT`、`C_NEWER_ROOTS`、`C_MAPPING`）：
-/// 只豁免现行账里仍分配着的槽，不豁免候选集里的根引用的槽（根环线 K4-1，函数体里那一圈的注释），D28（挂载期承诺量） 已定项 1 第九项
+/// 只豁免现行账里仍分配着的槽，不豁免候选集里的根引用的槽（根环线 K4-1，函数体里那一圈的注释）；落点里分配位置着的槽
+/// （已释放、还在 defer 窗口里的也算）不隔离、回收时补（同一圈的注释），D28（挂载期承诺量） 已定项 1 第九项
 /// 按分配器实际隔离的槽数取，两部分都在里面。
 /// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销；
 /// 被抛弃根引用的只在它离开根环的那一次发布里清，水位之上的记录点名的只在这个进程写掉那条记录的 txg 对应的根环槽那一次发布里清
@@ -1641,12 +1642,22 @@
     // 它比所选那一版新，它新分配的单元不在现行账里、回收门槛也罩不到，挂载之后当空闲槽发出去。只豁免现行账里仍分配着的槽：它们这时发不出去，
     // 这次挂载里被释放、回收出来的时候，分配器按还没解开的这一份补隔离（`PoolAllocator::record_root_written_by_this_process`）；
     // 不豁免它们，第九项与「已分配」就把同一个槽数两遍。
+    // 已释放、还在 defer 窗口里的槽同样在「已分配」里（分配位置着，`DeviceFreeMap::allocated_slots` 的读法甲），D28（挂载期承诺量） 已定项 1
+    // 第九项「不在这里再算一次」：落点里分配位置着的槽一律不隔离（判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md`
+    // 第二节 K1-3 H2：宽护并进来的已释放落点被第九项与「已分配」各扣一次，1536 槽盘挂载那一刻隔离 2242 槽、写全被拒），它们回收的那一刻
+    // 分配器照还没解开的这一份补隔离：挂载之内记根时（`PoolAllocator::record_root_written_by_this_process`），抬 F 那一串回收时
+    // （`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`，[`raise_rollback_floor`]）。
     for placement in placements_named_by_unapplied_journal_records {
         let key = (placement.device.0, placement.slot.0);
         if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {
             continue;
         }
-        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);
+        allocator
+            .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(
+                placement.device,
+                placement.slot,
+                placement.span,
+            );
     }
     Ok(ShadowLedgerComputed {
         abandoned_roots_unreadable: unreadable,
@@ -4484,11 +4495,14 @@
     // 其余那一截与准入那一格一样扣到生效。分配器记着回收过的槽不重复回收，第二次只回收第一次没回收的那几个。
     let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_ring);
     let reclaimed = match entry.reuse_of_the_reclaimed_slots() {
-        ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => allocator
-            .reclaim_released_up_to(
+        ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => {
+            reclaim_released_in_the_sequence_raising_the_floor_up_to(
+                allocator,
+                shadow_ledger,
                 floor_of_the_reclaim,
                 ReclaimedReuse::HeldUntilFloorTakesEffect,
-            ),
+            )
+        }
         ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice => {
             let roots_on_the_devices_of_their_txgs: Vec<(DeviceIdentity, RootRecord)> = roots
                 .iter()
@@ -4499,13 +4513,19 @@
                 &table,
                 &all_devices,
             ) {
-                Some(newest_valid_root_on_every_device) => allocator.reclaim_released_up_to(
-                    floor_of_the_reclaim.min(newest_valid_root_on_every_device),
-                    ReclaimedReuse::Immediately,
-                ),
+                Some(newest_valid_root_on_every_device) => {
+                    reclaim_released_in_the_sequence_raising_the_floor_up_to(
+                        allocator,
+                        shadow_ledger,
+                        floor_of_the_reclaim.min(newest_valid_root_on_every_device),
+                        ReclaimedReuse::Immediately,
+                    )
+                }
                 None => Vec::new(),
             };
-            reclaimed.extend(allocator.reclaim_released_up_to(
+            reclaimed.extend(reclaim_released_in_the_sequence_raising_the_floor_up_to(
+                allocator,
+                shadow_ledger,
                 floor_of_the_reclaim,
                 ReclaimedReuse::HeldUntilFloorTakesEffect,
             ));
@@ -4695,6 +4715,23 @@
     }
 }
 
+/// 抬 F 那一串的回收（[`raise_rollback_floor`]）：影子账开着时回收出来的槽里隔离还护着的当场补隔离
+/// （`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`）——C 那一圈的落点在抬 F 重算影子账那一刻
+/// 分配位置着的不隔离（D28（挂载期承诺量） 已定项 1 第九项，[`isolate_slots_referenced_only_by_abandoned_roots`] 那一圈的注释），
+/// 不在这里补，它们回收之后就发得出去。影子账关着（只供测试的对照，什么都不护）时只回收。
+fn reclaim_released_in_the_sequence_raising_the_floor_up_to(
+    allocator: &mut PoolAllocator,
+    shadow_ledger: ShadowLedger,
+    floor: CheckpointTxg,
+    reuse: ReclaimedReuse,
+) -> Vec<Placement> {
+    match shadow_ledger {
+        ShadowLedger::On => allocator
+            .reclaim_released_up_to_isolating_what_the_isolation_still_protects(floor, reuse),
+        ShadowLedger::Off => allocator.reclaim_released_up_to(floor, reuse),
+    }
+}
+
 /// 抬 F 那一串在预演里第几次（从 1 数）报了什么错。
 struct RaiseFloorRehearsalRefusal {
     refused_publish_in_the_sequence: usize,
@@ -8902,3 +8939,276 @@
         );
     }
 }
+
+#[cfg(test)]
+mod packed_units_named_by_the_records_of_a_publish_as_read_tests {
+    use super::{
+        packed_units_named_by_the_records_of_a_publish_as_read, PackedUnitNamedByAPublishAsRead,
+    };
+    use crate::address::{
+        CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
+        TreeIdentifier,
+    };
+    use crate::block_device::{
+        BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+    };
+    use crate::checksum::crc32_castagnoli;
+    use crate::journal::{
+        JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish, NamedUnit,
+    };
+    use crate::pointer::{BirthSequence, LocationEntry, NodePointer};
+    use crate::records::LineageRecord;
+    use crate::root_record::UnmountMarker;
+    use crate::unit::{
+        build_packed_unit, PackedIdentity, WriteOrder, PACKED_TYPE_INODE, PACKED_TYPE_LINEAGE,
+        UNIT_CLASS_DATA, UNIT_CLASS_PACKED,
+    };
+    use singlefs_format::{DATA_UNIT_BYTES, LINEAGE_RECORD_BYTES, SLOT_BYTES};
+
+    /// 这几条用例里那块盘的槽数：四个打包单元各占两槽，再留几槽。
+    const SLOTS_OF_THE_DEVICE: u64 = 16;
+
+    /// 这几条用例里点名的单元出生的那一代、那次发布的 txg。
+    const CHECKPOINT_TXG_OF_THE_PUBLISH: CheckpointTxg = CheckpointTxg(9);
+
+    /// 一块只在内存里的盘：读按偏移切字节，越过末尾报越界。
+    struct DeviceOfBytesInMemory {
+        bytes: Vec<u8>,
+    }
+
+    impl DeviceOfBytesInMemory {
+        fn range_of(
+            &self,
+            offset: DeviceOffsetInBytes,
+            length: usize,
+        ) -> Result<std::ops::Range<usize>, BlockDeviceError> {
+            let start = usize::try_from(offset.0).expect("用例里的偏移装得进 usize");
+            if start + length > self.bytes.len() {
+                return Err(BlockDeviceError::OutOfRange {
+                    offset,
+                    length: u64::try_from(length).expect("长度装得进 u64"),
+                    device_size: self.size_in_bytes(),
+                });
+            }
+            Ok(start..start + length)
+        }
+    }
+
+    impl BlockDevice for DeviceOfBytesInMemory {
+        fn read_at(
+            &self,
+            offset: DeviceOffsetInBytes,
+            buffer: &mut [u8],
+        ) -> Result<(), BlockDeviceError> {
+            let range = self.range_of(offset, buffer.len())?;
+            buffer.copy_from_slice(&self.bytes[range]);
+            Ok(())
+        }
+        fn write_at(
+            &mut self,
+            offset: DeviceOffsetInBytes,
+            bytes: &[u8],
+            _durability: WriteDurability,
+        ) -> Result<(), BlockDeviceError> {
+            let range = self.range_of(offset, bytes.len())?;
+            self.bytes[range].copy_from_slice(bytes);
+            Ok(())
+        }
+        fn write_zeroes_at(
+            &mut self,
+            offset: DeviceOffsetInBytes,
+            length: u64,
+        ) -> Result<(), BlockDeviceError> {
+            let range =
+                self.range_of(offset, usize::try_from(length).expect("长度装得进 usize"))?;
+            self.bytes[range].fill(0);
+            Ok(())
+        }
+        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+            Ok(())
+        }
+        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+            PhysicalBlockSizeInBytes(512)
+        }
+        fn size_in_bytes(&self) -> u64 {
+            u64::try_from(self.bytes.len()).expect("盘宽装得进 u64")
+        }
+    }
+
+    fn packed_unit_of(record_type: u16, record_width: u16, records: &[Vec<u8>]) -> Vec<u8> {
+        build_packed_unit(
+            PackedIdentity {
+                birth_tree: TreeIdentifier(65_536),
+                record_type,
+                container: 0,
+                container_birth: CHECKPOINT_TXG_OF_THE_PUBLISH,
+            },
+            record_width,
+            records,
+            CHECKPOINT_TXG_OF_THE_PUBLISH,
+            &[1; 16],
+            WriteOrder {
+                instance: InstanceGeneration(1),
+                transaction: 1,
+            },
+            BirthSequence(0),
+        )
+    }
+
+    /// 点名 `slot` 上那个单元的点名项，两条位置条目都在盘 0 的这一槽、校验和是 `unit_checksum`。
+    fn named_unit_at(slot: u64, unit_class: u8, unit_checksum: u32) -> NamedUnit {
+        let location = LocationEntry {
+            device: DeviceIdentity(0),
+            slot: SlotNumber(slot),
+            unit_checksum,
+        };
+        NamedUnit {
+            locations: [location, location],
+            unit_class,
+            birth_tree: TreeIdentifier(65_536),
+            birth_txg: CHECKPOINT_TXG_OF_THE_PUBLISH,
+            key_tail: [0; 10],
+        }
+    }
+
+    fn record_naming(ordinal_from_one: u32, named: Vec<NamedUnit>) -> JournalRecord {
+        JournalRecord {
+            instance: InstanceGeneration(1),
+            counter: u64::from(ordinal_from_one),
+            checkpoint_txg: CHECKPOINT_TXG_OF_THE_PUBLISH,
+            transaction: u64::from(ordinal_from_one),
+            is_commit: true,
+            ordinal_within_publish: JournalRecordOrdinalWithinPublish(ordinal_from_one),
+            place_in_publish: JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
+            back_chain: 0,
+            filesystem_identifier: 0,
+            new_tree_table: NodePointer::empty_root(),
+            new_mapping_root: NodePointer::empty_root(),
+            new_tree_identifier_watermark: 0,
+            new_rollback_floor: CheckpointTxg(0),
+            new_instance_table: NodePointer::empty_root(),
+            new_allocation_record_tree_root: NodePointer::empty_root(),
+            new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+            named,
+        }
+    }
+
+    /// 读成的样子写成一行字（用例只比这一行，被测的类型不为用例另加比较）。
+    fn description_of(packed_unit: &PackedUnitNamedByAPublishAsRead) -> String {
+        match packed_unit {
+            PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch => {
+                "读不出或校验和对不上".to_string()
+            }
+            PackedUnitNamedByAPublishAsRead::ReadButUnparsable => "读得出、解不开".to_string(),
+            PackedUnitNamedByAPublishAsRead::Parsed {
+                record_type,
+                records,
+            } => format!("解开了：类型 {record_type}，记录 {records:?}"),
+        }
+    }
+
+    /// 读盘一层（正推 `research/prompts/m3-batch10z-lm-code-r1-sonnet-output.md` 第 2 节 X4、X5、X18）：一次发布两条记录，第一条点名一片 inode 叶容器、
+    /// 一个数据单元与一片读得出而解不开的打包单元（整单元校验和对得上条目），第二条点名一片校验和对不上条目的 inode 叶容器与一片谱系片。
+    /// 交回逐条记录、逐个点名的每一片打包单元（数据单元不算），次序照记录与点名的次序，四种结局各认得出：只看第一条记录、只看第一片、
+    /// 解不开的当解开了（旧写法 `.ok()` 吞掉）、不筛类标签、记录类型或记录读丢了，这一句都红。
+    #[test]
+    fn every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart(
+    ) {
+        let unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
+        let slots_per_unit = DATA_UNIT_BYTES / SLOT_BYTES;
+        let inode_record = vec![7u8; 16];
+        let inode_unit = packed_unit_of(PACKED_TYPE_INODE, 16, std::slice::from_ref(&inode_record));
+        let lineage_record = LineageRecord {
+            rollback_txg: CHECKPOINT_TXG_OF_THE_PUBLISH,
+            rollback_target_txg: CheckpointTxg(5),
+        }
+        .to_bytes();
+        let lineage_unit = packed_unit_of(
+            PACKED_TYPE_LINEAGE,
+            u16::try_from(LINEAGE_RECORD_BYTES).expect("16"),
+            std::slice::from_ref(&lineage_record),
+        );
+        let unparsable_unit = vec![0xA5u8; unit_bytes];
+        assert!(
+            crate::unit::parse_packed_unit(&unparsable_unit).is_err(),
+            "那一片解不开（阳性对照）"
+        );
+        let slot_of_the_inode_unit = 0;
+        let slot_of_the_unparsable_unit = slots_per_unit;
+        let slot_of_the_mismatching_unit = 2 * slots_per_unit;
+        let slot_of_the_lineage_unit = 3 * slots_per_unit;
+        let slot_of_the_data_unit = 4 * slots_per_unit;
+        let mut device = DeviceOfBytesInMemory {
+            bytes: vec![0; usize::try_from(SLOTS_OF_THE_DEVICE * SLOT_BYTES).expect("盘宽")],
+        };
+        for (slot, bytes) in [
+            (slot_of_the_inode_unit, &inode_unit),
+            (slot_of_the_unparsable_unit, &unparsable_unit),
+            (slot_of_the_mismatching_unit, &inode_unit),
+            (slot_of_the_lineage_unit, &lineage_unit),
+        ] {
+            device
+                .write_at(
+                    SlotNumber(slot).to_device_offset(),
+                    bytes,
+                    WriteDurability::Plain,
+                )
+                .expect("写在盘宽之内");
+        }
+        let devices = vec![(DeviceIdentity(0), device)];
+        let first_record = record_naming(
+            1,
+            vec![
+                named_unit_at(
+                    slot_of_the_inode_unit,
+                    UNIT_CLASS_PACKED,
+                    crc32_castagnoli(&inode_unit),
+                ),
+                named_unit_at(slot_of_the_data_unit, UNIT_CLASS_DATA, 0),
+                named_unit_at(
+                    slot_of_the_unparsable_unit,
+                    UNIT_CLASS_PACKED,
+                    crc32_castagnoli(&unparsable_unit),
+                ),
+            ],
+        );
+        let second_record = record_naming(
+            2,
+            vec![
+                named_unit_at(
+                    slot_of_the_mismatching_unit,
+                    UNIT_CLASS_PACKED,
+                    crc32_castagnoli(&inode_unit) ^ 1,
+                ),
+                named_unit_at(
+                    slot_of_the_lineage_unit,
+                    UNIT_CLASS_PACKED,
+                    crc32_castagnoli(&lineage_unit),
+                ),
+            ],
+        );
+        let descriptions: Vec<String> = packed_units_named_by_the_records_of_a_publish_as_read(
+            &devices,
+            &[&first_record, &second_record],
+        )
+        .iter()
+        .map(description_of)
+        .collect();
+        assert_eq!(
+            descriptions,
+            vec![
+                description_of(&PackedUnitNamedByAPublishAsRead::Parsed {
+                    record_type: PACKED_TYPE_INODE,
+                    records: vec![inode_record],
+                }),
+                description_of(&PackedUnitNamedByAPublishAsRead::ReadButUnparsable),
+                description_of(&PackedUnitNamedByAPublishAsRead::UnreadableOrChecksumMismatch),
+                description_of(&PackedUnitNamedByAPublishAsRead::Parsed {
+                    record_type: PACKED_TYPE_LINEAGE,
+                    records: vec![lineage_record],
+                }),
+            ],
+            "两条记录点名的四片打包单元逐个读成各自的样子、次序照记录与点名"
+        );
+    }
+}
--- a/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
+++ b/crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs
@@ -56,8 +56,9 @@
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
 use singlefs_core::mount::{
     mount_writable, mount_writable_with_space_admission, mount_writable_with_test_only_switches,
-    ring_slots_known_to_hold_a_root_by, ring_slots_whose_reread_still_bad_refuses_the_ceiling_by,
-    roll_back_by_a_forward_publish, rollback_floor_ceiling, unmount, MountError, RollbackTarget,
+    raise_rollback_floor_to_the_admission_ceiling, ring_slots_known_to_hold_a_root_by,
+    ring_slots_whose_reread_still_bad_refuses_the_ceiling_by, roll_back_by_a_forward_publish,
+    rollback_floor_ceiling, unmount, MountError, RaiseToTheAdmissionCeiling, RollbackTarget,
     ShadowLedger, StillUnreadableAfterOneReread, Unmounted,
 };
 use singlefs_core::mounted_session::MountedSession;
@@ -2373,3 +2374,875 @@
 /// [`a_root_taking_the_current_account_that_is_not_a_rollback_takes_nothing_released_and_isolates_as_before_at_the_mount`] 那一形挂载那一刻两块盘隔离的槽数（tri 之前量的）。
 const ISOLATED_SLOTS_AT_THE_MOUNT_WHEN_A_ROOT_TAKING_THE_CURRENT_ACCOUNT_IS_NOT_A_ROLLBACK: u64 =
     18;
+
+/// 两块盘上分配位与隔离位同时置着的槽数之和：D28（挂载期承诺量） 已定项 1 第九项与「已分配」各扣一次的那几个。
+fn slots_counted_both_as_allocated_and_as_isolated(allocator: &PoolAllocator) -> u64 {
+    allocator
+        .devices
+        .iter()
+        .map(singlefs_core::allocator::DeviceFreeMap::slots_counted_both_as_allocated_and_as_isolated)
+        .sum()
+}
+
+/// 两块盘隔离的槽数之和（D28（挂载期承诺量） 已定项 1 第九项那一项）。
+fn isolated_slots_of(allocator: &PoolAllocator) -> u64 {
+    allocator
+        .devices
+        .iter()
+        .map(singlefs_core::allocator::DeviceFreeMap::isolated_slots)
+        .sum()
+}
+
+/// 正推 `research/prompts/m3-batch10z-lm-code-r1-sonnet-output.md` 第 1.3 节那一形之前覆盖写几次（探针 `S2_PREFIXES` 原样 40 次）。
+const OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK: u64 = 40;
+
+/// 那一形回退到倒数第几版（探针 `S2_RS=b6`：录的版本表里倒数第 1 + 6 版）。
+const VERSIONS_BACK_OF_THE_ROLLBACK_TARGET: usize = 6;
+
+/// 起池（384 槽那一档、第一个文件），覆盖写 `overwrites_before` 次得 P，经会话管理员回退到此前写过的倒数第 1 + `versions_back` 版（R_old），
+/// 录这一次发布得 R（正推探针的 `record` 那一种 `b<数>`）。交回录下来的 R、R 复活的数据单元的落点（R_old 的文件数据指针里 P 不引用的那几条位置条目）、
+/// R_old 的根与它账里仍分配着的落点（都从录下来的会话算，不经被测的方法）。
+fn record_one_rollback_after_overwrites(
+    overwrites_before: u64,
+    versions_back: usize,
+) -> RecordedRollback {
+    let stream = SharedStream::retaining_contents();
+    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
+        parameters_of_the_384_slot_small_devices(),
+        device_bytes_of_the_384_slot_small_devices(),
+        |identity, device| {
+            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
+        },
+    );
+    // 每一版：根、文件数据指针的位置条目（盘、槽）、账里仍分配着的（盘、起点槽）。
+    let version_of = |pool_at_this_version: &PoolUnderTest<
+        RecordingBlockDevice<SparseBlockDevice>,
+    >|
+     -> RolledBackVersion {
+        RolledBackVersion {
+            root: *pool_at_this_version.session().current.root(),
+            data_locations: pool_at_this_version
+                .current_file_version()
+                .data_pointers
+                .iter()
+                .flat_map(|pointer| pointer.locations)
+                .map(|location| (location.device, location.slot))
+                .collect(),
+            allocated_in_its_account: pool_at_this_version
+                .session()
+                .allocator
+                .records()
+                .iter()
+                .filter(|record| !record.is_released)
+                .map(|record| (record.device, record.slot))
+                .collect(),
+        }
+    };
+    let mut versions_written = vec![version_of(&pool)];
+    for _overwrite in 0..overwrites_before {
+        pool.overwrite(OVERWRITE_BYTES)
+            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下（准入推抬 F 也做得成）");
+        versions_written.push(version_of(&pool));
+    }
+    let target_version = &versions_written[versions_written.len() - 1 - versions_back];
+    let rollback_target = target_version.root;
+    let locations_referenced_by_p = &versions_written[versions_written.len() - 1].data_locations;
+    let revived: Vec<PlacementOnDevice> = target_version
+        .data_locations
+        .iter()
+        .filter(|location| !locations_referenced_by_p.contains(location))
+        .map(|(device, slot)| PlacementOnDevice {
+            device: *device,
+            slot: *slot,
+            span: singlefs_format::DATA_UNIT_BYTES.div_ceil(singlefs_format::SLOT_BYTES),
+        })
+        .collect();
+    let allocated_in_the_account_of_the_rollback_target =
+        target_version.allocated_in_its_account.clone();
+    let previous_root = *pool.session().current.root();
+    let previous_content = pool.content_of_the_current_version.clone();
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
+                instance: rollback_target.instance,
+                checkpoint_txg: rollback_target.checkpoint_txg,
+            },
+        )
+        .expect("回退到倒数第 7 版做得成（它在根环里、按实例表有效）");
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
+    RecordedRollback {
+        recorded: RecordedPublish {
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
+        rollback_target,
+        allocated_in_the_account_of_the_rollback_target,
+    }
+}
+
+/// [`record_one_rollback_after_overwrites`] 录的每一版。
+struct RolledBackVersion {
+    root: RootRecord,
+    data_locations: Vec<(DeviceIdentity, SlotNumber)>,
+    allocated_in_its_account: Vec<(DeviceIdentity, SlotNumber)>,
+}
+
+/// H2 双扣（判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第二节 K1-3、第三节第十四块第 1 条；正推第 1.3 节 384 槽盘 720 槽那一形）：
+/// 覆盖写 40 次、回退到倒数第 7 版录 R，可写挂载那一刻 R 的账与谱系片读回全零：认不出是不是回退，C 里 R 那一槽宽护、并进现行账里全部已释放的落点。
+/// 其中还在 defer 窗口里的（分配位置着）算在「已分配」里，D28（挂载期承诺量） 已定项 1 第九项「不在这里再算一次」：挂载那一刻一个槽都不同时
+/// 置着分配位与隔离位，C 里 R 那一槽的每个槽也都不空闲（分配着或隔离着，恰好一样）。改之前两块盘 104 个槽两项各扣一次（隔离 720 槽）。
+/// 这一形去掉双扣之后写照旧全被拒（隔离 616 槽，第 10z 批第十四块实现员报告「H1 重量」），不在这里钉，交主 agent。
+#[test]
+fn wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices(
+) {
+    let recorded = record_one_rollback_after_overwrites(
+        OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
+        VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
+    )
+    .recorded;
+    let lineage_unit_writes = lineage_unit_writes_of(&recorded);
+    assert!(
+        !lineage_unit_writes.is_empty(),
+        "回退那次发布写了谱系片（阳性对照：故障有处可打）"
+    );
+    let root_of_r = recorded.root();
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let mut devices = devices_reading_back_all_zero_on(
+        &crash_image,
+        &RangesReadingBackAllZero::default()
+            .with_the_writes(&account_writes_of(&recorded))
+            .with_the_writes(&lineage_unit_writes),
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
+    let released_not_borrowed: Vec<&(DeviceIdentity, SlotNumber)> = released_in_the_current_account
+        .iter()
+        .filter(|(device, slot)| {
+            !borrowed_for_r
+                .iter()
+                .any(|borrowed| borrowed.device == *device && borrowed.slot == *slot)
+        })
+        .collect();
+    assert!(
+        released_not_borrowed.is_empty(),
+        "认不出是不是回退：C 里 R 那一槽并进现行账里全部已释放的落点（阳性对照：宽护走到了），缺 {released_not_borrowed:?}"
+    );
+    assert!(
+        isolated_slots_of(&mounted.allocator) > 0,
+        "挂载那一刻隔离着 C 里分配位没置着的槽（阳性对照）"
+    );
+    assert_eq!(
+        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
+        0,
+        "挂载那一刻没有一个槽同时算进「已分配」与第九项（D28（挂载期承诺量） 已定项 1 第九项：现行账里仍分配着的、已释放而还在 defer 窗口里的不在这里再算一次）"
+    );
+    let free_among_the_borrowed = free_slots_covered_by(&mounted.allocator, &borrowed_for_r);
+    assert!(
+        free_among_the_borrowed.is_empty(),
+        "C 里 R 那一槽的每个槽挂载那一刻都不空闲（分配着或隔离着）：空闲的（盘、槽）{free_among_the_borrowed:?}"
+    );
+}
+
+/// 单元区 1536 槽那一档每块盘的字节数：384 槽那一档的盘宽再加 1536 − 384 槽（攻方 `device_bytes_with_unit_area_slots(1536)` 同一个数）。
+fn device_bytes_of_the_1536_slot_devices() -> u64 {
+    const UNIT_AREA_SLOTS_OF_THE_1536_SLOT_DEVICES: u64 = 1536;
+    const UNIT_AREA_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 384;
+    device_bytes_of_the_384_slot_small_devices()
+        + (UNIT_AREA_SLOTS_OF_THE_1536_SLOT_DEVICES - UNIT_AREA_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES)
+            * singlefs_format::SLOT_BYTES
+}
+
+/// 攻方 `research/prompts/m3-batch10z-lm-code-r1-opus-model/runs-summary/mount-counts-b11.txt` 那一形的前缀：顺序写几次（`S` 22 次）。
+const SEQUENTIAL_WRITES_OF_TWENTY_DATA_UNITS_BEFORE_R: usize = 22;
+
+/// 每次顺序写几个数据单元（攻方 `S`）。
+const DATA_UNITS_OF_EACH_SEQUENTIAL_WRITE_BEFORE_R: usize = 20;
+
+/// 那一形前缀末尾回退到倒数第几版（攻方 `b8`）。
+const VERSIONS_BACK_OF_THE_ROLLBACK_BEFORE_R: usize = 8;
+
+/// 挂上之后覆盖写几次、再顺序写几次（每次两个数据单元）：攻方 b9 那一串 30 步全被拒，这里 40 步。
+const OVERWRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES: usize = 30;
+const SEQUENTIAL_WRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES: usize = 10;
+
+/// 挂上之后会话做的一步写。
+#[derive(Clone, Copy, Debug)]
+enum UserWriteAfterTheMount {
+    Overwrite,
+    SequentialWriteOfTwoDataUnits,
+}
+
+/// 攻方那一形（1536 槽两块盘）：起池、第一个文件之后顺序写 22 次（每次 20 个数据单元，先释放一大批），回退到倒数第 9 版，录一次建 inode 得 R。
+fn record_one_inode_creation_after_releasing_a_large_batch_on_the_1536_slot_devices(
+) -> RecordedPublish {
+    let stream = SharedStream::retaining_contents();
+    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
+        parameters_of_the_384_slot_small_devices(),
+        device_bytes_of_the_1536_slot_devices(),
+        |identity, device| {
+            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
+        },
+    );
+    let mut roots_written = vec![*pool.session().current.root()];
+    for _sequential_write in 0..SEQUENTIAL_WRITES_OF_TWENTY_DATA_UNITS_BEFORE_R {
+        pool.sequential_write(
+            DATA_UNITS_OF_EACH_SEQUENTIAL_WRITE_BEFORE_R
+                * singlefs_core::unit::data_unit_payload_capacity(),
+        )
+        .expect("1536 槽两块盘上这几次顺序写都落得下");
+        roots_written.push(*pool.session().current.root());
+    }
+    let rollback_target =
+        roots_written[roots_written.len() - 1 - VERSIONS_BACK_OF_THE_ROLLBACK_BEFORE_R];
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
+                instance: rollback_target.instance,
+                checkpoint_txg: rollback_target.checkpoint_txg,
+            },
+        )
+        .expect("回退到倒数第 9 版做得成（它在根环里、按实例表有效）");
+    }
+    pool.session = Some(session);
+    pool.content_of_the_current_version = pool
+        .content_of_each_root
+        .get(&(rollback_target.instance, rollback_target.checkpoint_txg))
+        .cloned()
+        .expect("回退目标是录的池写过的一版");
+    let root_after_the_rollback = *pool.session().current.root();
+    pool.note_root_of_the_current_content(&root_after_the_rollback);
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
+        previous_key: (
+            root_after_the_rollback.instance,
+            root_after_the_rollback.checkpoint_txg,
+        ),
+        previous_content,
+    }
+}
+
+/// H2 双扣是 H1 卡死的主来源那一形（攻方 `research/prompts/m3-batch10z-lm-code-r1-opus-output.md`「K1-3 打中」两节，`mount-counts-b11.txt`、
+/// `wedge-b6-b10.txt` 的 b9）：1536 槽两块盘、先顺序写释放一大批再回退，R 建一个 inode、R 写的全部单元挂载那一刻读回全零（不是回退而认不出，
+/// 宽护并进现行账里全部已释放的落点）。改之前挂载那一刻隔离 2242 槽、其中 338 槽两项各扣一次，之后每次写都被准入拒、R 一直在根环里。
+/// 改之后：挂载那一刻、每一步之后都没有一个槽同时算进「已分配」与第九项；覆盖写 30 次、顺序写 10 次一次都不拒；R 那一槽写掉（R 离开根环）；
+/// R 还在根环里时 C 里 R 那一槽的槽一个都不空闲，每一步之后池级 checker 不红 I-7.4 与 I-4.8（回收之后补的隔离照罩着）。
+#[test]
+#[ignore = "harness 慢档用例：1536 槽两块盘、40 步每步跑池级 checker，debug 下单条过 60 秒"]
+fn wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices(
+) {
+    let recorded =
+        record_one_inode_creation_after_releasing_a_large_batch_on_the_1536_slot_devices();
+    assert!(
+        lineage_unit_writes_of(&recorded).is_empty(),
+        "建 inode 不写谱系片（R 不是回退：宽护只因为它写的单元读不出）"
+    );
+    let root_of_r = recorded.root();
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let lift = ReadBackAllZeroLift::default();
+    let mut devices = devices_reading_back_all_zero_on_until_lifted(
+        &crash_image,
+        &RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes()),
+        &lift,
+    );
+    let mounted = mount_writable_with_space_admission(
+        &recorded.pool.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("R 写的单元读回全零而记录齐：可写挂载落到 P、做得成");
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
+    assert!(
+        released_in_the_current_account
+            .iter()
+            .all(|(device, slot)| {
+                borrowed_for_r
+                    .iter()
+                    .any(|borrowed| borrowed.device == *device && borrowed.slot == *slot)
+            }),
+        "认不出是不是回退：C 里 R 那一槽并进现行账里全部已释放的落点（阳性对照：宽护走到了）"
+    );
+    assert_eq!(
+        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
+        0,
+        "挂载那一刻没有一个槽同时算进「已分配」与第九项（改之前 338 个）"
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
+    let steps = std::iter::repeat_n(
+        UserWriteAfterTheMount::Overwrite,
+        OVERWRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES,
+    )
+    .chain(std::iter::repeat_n(
+        UserWriteAfterTheMount::SequentialWriteOfTwoDataUnits,
+        SEQUENTIAL_WRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES,
+    ));
+    let mut root_of_r_left_the_ring = false;
+    // 迭代上界是步数（40）；跨轮带的是「R 离没离开根环」；出口：被拒、双扣、C 里 R 那一槽有空闲的槽、checker 红都当场判红（panic）。
+    for (step_number, user_write) in (1..).zip(steps) {
+        let root_of_r_was_in_the_ring = root_is_in_the_ring(&pool.image(), recorded.key);
+        let published = match user_write {
+            UserWriteAfterTheMount::Overwrite => pool.overwrite(OVERWRITE_BYTES),
+            UserWriteAfterTheMount::SequentialWriteOfTwoDataUnits => {
+                pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
+            }
+        };
+        if let Err(refusal) = published {
+            panic!("第 {step_number} 步被拒（改之前每一步都被准入拒）：{refusal:?}");
+        }
+        assert_eq!(
+            slots_counted_both_as_allocated_and_as_isolated(&pool.session().allocator),
+            0,
+            "第 {step_number} 步之后没有一个槽同时算进「已分配」与第九项"
+        );
+        if root_of_r_was_in_the_ring && root_is_in_the_ring(&pool.image(), recorded.key) {
+            let free_among_the_borrowed =
+                free_slots_covered_by(&pool.session().allocator, &borrowed_for_r);
+            assert!(
+                free_among_the_borrowed.is_empty(),
+                "第 {step_number} 步之后 R 还在根环里：C 里 R 那一槽的槽一个都不空闲（回收时补了隔离），空闲的 {free_among_the_borrowed:?}"
+            );
+        } else {
+            root_of_r_left_the_ring = true;
+        }
+        let violations = violations_of_unit_reuse_on(&pool.image());
+        assert!(
+            violations.is_empty(),
+            "第 {step_number} 步之后被抛弃根引用的单元一个都没复用：{violations:?}"
+        );
+    }
+    assert!(
+        root_of_r_left_the_ring,
+        "40 步之内 R 那一槽写掉、R 离开根环（改之前 R 一直在环里、隔离到进程结束）"
+    );
+}
+
+/// R 每条记录（按本次发布内序号）点名的打包单元（类标签 3）各是哪一种打包记录类型，按点名的次序：从录下来的记录写与单元写的字节解，
+/// 不经被测的方法。点名的单元 R 没写（照抄上一版的）记 `None`。
+fn packed_record_types_named_by_each_record_of(
+    recorded: &RecordedPublish,
+) -> Vec<Vec<Option<u16>>> {
+    let filesystem_identifier = singlefs_core::unit::unit_filesystem_identifier(
+        &recorded.pool.parameters.filesystem_identifier,
+    );
+    (1..=recorded.record_count())
+        .map(|ordinal_from_one| {
+            let record_bytes = recorded.record_writes_of_the_record(ordinal_from_one)[0]
+                .bytes()
+                .expect("记录写带字节");
+            let record =
+                singlefs_core::journal::JournalRecord::parse(record_bytes, filesystem_identifier)
+                    .expect("R 录下来的记录解得开");
+            record
+                .named
+                .iter()
+                .filter(|named| named.unit_class == singlefs_core::unit::UNIT_CLASS_PACKED)
+                .map(|named| {
+                    recorded
+                        .unit_writes()
+                        .into_iter()
+                        .find(|write| {
+                            write.device == named.locations[0].device
+                                && write.offset == named.locations[0].slot.to_device_offset()
+                        })
+                        .and_then(|write| write.bytes())
+                        .and_then(|bytes| singlefs_core::unit::parse_packed_unit(bytes).ok())
+                        .map(|packed| packed.identity.record_type)
+                })
+                .collect()
+        })
+        .collect()
+}
+
+/// R 写出的打包单元里打包记录类型是 `record_type` 的那几份写：从录下来的写的字节解，不经被测的方法。
+fn packed_unit_writes_of_type(recorded: &RecordedPublish, record_type: u16) -> Vec<&RetainedWrite> {
+    recorded
+        .unit_writes()
+        .into_iter()
+        .filter(|write| {
+            write
+                .bytes()
+                .and_then(|bytes| singlefs_core::unit::parse_packed_unit(bytes).ok())
+                .is_some_and(|packed| packed.identity.record_type == record_type)
+        })
+        .collect()
+}
+
+/// R 走丙、认不出是不是回退那一形的共同判法（tri 第一种）：可写挂载那一刻 `ranges` 读回全零，择根退到 P；C 里 R 那一槽里在现行账里记成已释放的落点
+/// 就是现行账里全部已释放的（宽护：一个不缺，按盘与起点槽）。
+fn assert_the_root_taking_the_current_account_takes_every_released_placement_at_the_mount(
+    recorded: &RecordedPublish,
+    ranges: &RangesReadingBackAllZero,
+) {
+    let root_of_r = recorded.root();
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let mut devices = devices_reading_back_all_zero_on(&crash_image, ranges);
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
+        "挂载那一刻现行账里有已释放的落点（阳性对照：并少了看得出来）"
+    );
+    let released_borrowed_for_r: std::collections::BTreeSet<(DeviceIdentity, SlotNumber)> = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
+            root_of_r.instance,
+            root_of_r.checkpoint_txg,
+        )
+        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）")
+        .into_iter()
+        .map(|placement| (placement.device, placement.slot))
+        .filter(|placement| released_in_the_current_account.contains(placement))
+        .collect();
+    assert_eq!(
+        released_borrowed_for_r, released_in_the_current_account,
+        "R 的记录点名的打包单元有一片读不出、认不出是不是回退：C 里 R 那一槽并进现行账里全部已释放的落点（tri 宽护）"
+    );
+}
+
+/// 删除那一形之前覆盖写几次。
+const OVERWRITES_BEFORE_THE_DELETION_RECORDED_AS_R: u64 = 3;
+
+/// 起池（384 槽那一档、第一个文件），覆盖写 `overwrites_before` 次、建一个 inode，再经会话删掉它，录删除的第一次发布得 R
+/// （崩在 R 的根槽落盘之后、轮换之前那一份写表只截到第一条根：删除那一次写两条根）。R 的记录点名它写的 inode 叶容器与墓碑片。
+fn record_the_first_publish_of_one_deletion_after_overwrites(
+    overwrites_before: u64,
+) -> RecordedPublish {
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
+    pool.create_one_inode().expect("建要删的那个 inode 落得下");
+    let previous_root = *pool.session().current.root();
+    let previous_content = pool.content_of_the_current_version.clone();
+    let base = pool.image();
+    let operations_before_r = stream.operation_count();
+    let inode = singlefs_core::address::InodeNumber(
+        pool.current_file_version().inode_number_watermark() - 1,
+    );
+    pool.write_time_seconds += 1;
+    let write_time_seconds = pool.write_time_seconds;
+    pool.publish(
+        singlefs_core::mounted_session::UserChange::Unlink {
+            inode,
+            write_time_seconds,
+        },
+        None,
+    )
+    .expect("删掉刚建的那个 inode 做得成");
+    let operations = stream.retained_operations();
+    let (mut writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[operations_before_r..],
+        &fixed_geometry_of_the_384_slot_small_devices(),
+    );
+    let first_root_index = writes
+        .iter()
+        .position(|write| write.kind == StepKind::RootRecordFua)
+        .expect("删除写了根");
+    writes.truncate(first_root_index + 1);
+    let root = RootRecord::parse_slot(
+        writes[first_root_index].bytes().expect("根槽写带字节"),
+        &pool.parameters.filesystem_identifier,
+    )
+    .expect("删除的第一条根自证");
+    RecordedPublish {
+        content: previous_content.clone(),
+        pool,
+        base,
+        writes,
+        root_index: first_root_index,
+        key: (root.instance, root.checkpoint_txg),
+        previous_key: (previous_root.instance, previous_root.checkpoint_txg),
+        previous_content,
+    }
+}
+
+/// tri 第一种的删除那一形（正推 `research/prompts/m3-batch10z-lm-code-r1-sonnet-output.md` 第 2 节「删除发布墓碑片读不出 → 宽护没有入库用例」，
+/// 探针 `faults=racct,rtomb`）：R 是一次删除的第一次发布，记录点名它写的 inode 叶容器与墓碑片（叶容器排在前面）；挂载那一刻 R 的账与墓碑片
+/// 读回全零：认不出是不是回退，C 里 R 那一槽并进现行账里全部已释放的落点。读盘一层只看第一个打包单元（正推 X5）时只看到读得出的叶容器、
+/// 认成肯定不是回退，这一句红。
+#[test]
+fn a_deletion_whose_tombstone_unit_and_account_read_back_zero_at_the_mount_takes_every_released_placement(
+) {
+    let recorded = record_the_first_publish_of_one_deletion_after_overwrites(
+        OVERWRITES_BEFORE_THE_DELETION_RECORDED_AS_R,
+    );
+    let tombstone_unit_writes =
+        packed_unit_writes_of_type(&recorded, singlefs_core::unit::PACKED_TYPE_TOMBSTONE);
+    assert!(
+        !tombstone_unit_writes.is_empty(),
+        "删除的第一次发布写了墓碑片（阳性对照：故障有处可打）"
+    );
+    let named_in_order: Vec<Option<u16>> = packed_record_types_named_by_each_record_of(&recorded)
+        .into_iter()
+        .flatten()
+        .collect();
+    let tombstone_position = named_in_order
+        .iter()
+        .position(|record_type| *record_type == Some(singlefs_core::unit::PACKED_TYPE_TOMBSTONE))
+        .expect("R 的记录点名墓碑片");
+    assert!(
+        tombstone_position > 0,
+        "R 的记录在墓碑片之前还点名了别的打包单元（这一形分得出只看第一个打包单元）：{named_in_order:?}"
+    );
+    assert_the_root_taking_the_current_account_takes_every_released_placement_at_the_mount(
+        &recorded,
+        &RangesReadingBackAllZero::default()
+            .with_the_writes(&account_writes_of(&recorded))
+            .with_the_writes(&tombstone_unit_writes),
+    );
+}
+
+/// 顺序写那一形之前覆盖写几次。
+const OVERWRITES_BEFORE_THE_SEQUENTIAL_WRITE_RECORDED_AS_R: u64 = 3;
+
+/// tri 第一种的多条记录那一形：R 顺序写三个数据单元（三条记录，前两条只点名数据单元，inode 叶容器只在末条点名）；挂载那一刻 R 的账与它写的
+/// inode 叶容器读回全零：认不出是不是回退，C 里 R 那一槽并进现行账里全部已释放的落点。读盘一层只看第一条记录点名的打包单元（正推 X4）时
+/// 一个打包单元都看不到、认成肯定不是回退，这一句红。
+#[test]
+fn a_sequential_write_whose_inode_unit_named_by_its_last_record_and_account_read_back_zero_at_the_mount_takes_every_released_placement(
+) {
+    let recorded = record_one_publish_after_overwrites(
+        OVERWRITES_BEFORE_THE_SEQUENTIAL_WRITE_RECORDED_AS_R,
+        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
+    );
+    let named_by_each_record = packed_record_types_named_by_each_record_of(&recorded);
+    assert!(
+        named_by_each_record.len() >= 2 && named_by_each_record[0].is_empty(),
+        "R 有两条以上记录、第一条不点名打包单元（这一形分得出只看第一条记录）：{named_by_each_record:?}"
+    );
+    let inode_unit_writes =
+        packed_unit_writes_of_type(&recorded, singlefs_core::unit::PACKED_TYPE_INODE);
+    assert!(
+        !inode_unit_writes.is_empty(),
+        "R 写了 inode 叶容器（阳性对照：故障有处可打）"
+    );
+    assert_the_root_taking_the_current_account_takes_every_released_placement_at_the_mount(
+        &recorded,
+        &RangesReadingBackAllZero::default()
+            .with_the_writes(&account_writes_of(&recorded))
+            .with_the_writes(&inode_unit_writes),
+    );
+}
+
+/// 回退之前覆盖写几次（隔离量离整盘还远、写不被拒）。
+const OVERWRITES_BEFORE_THE_ROLLBACK_WHOSE_PLACEMENTS_A_FLOOR_RAISE_RECLAIMS: u64 = 10;
+
+/// 挂上之后至多覆盖写几次去等 R 那一槽写掉（R 离开根环）。
+const OVERWRITES_UNTIL_R_LEAVES_THE_RING_AT_MOST: usize = 40;
+
+/// H2 改法要回收时补隔离那一半（判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第三节第十四块第 1 条「I-7.4 照旧不红」）：
+/// 覆盖写 10 次、回退到倒数第 7 版录 R，崩在 R 的根槽落盘之后、轮换之前，可写挂载那一刻 R 的根槽与 R 写的谱系片读回全零（C694 的 C：
+/// 系统配置没见证 R、记录点名的谱系片验不过、记录不施加，择根落到 P）。R 的记录齐、用记录的新根段读得出 R 的账，C 那一圈把 R 的账里的落点
+/// 隔离到这个进程写掉 R 那一槽；其中 R 复活的数据单元在 P 的账里已释放、
+/// 还在 defer 窗口里（分配位置着），挂载那一刻不隔离（第九项不再算一次）。挂上之后先推一串抬 F 到准入上限：根环还没转，回收这几个复活单元的
+/// 只有这一串；R 的根槽挂上之后照旧读回全零（到写掉那一槽为止），这一串读不出 R、不把它当被抛弃根算影子账，护着它们的只有 C 那一圈——分配器回收时照它补隔离
+/// （`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`）。推过之后复活的槽一个都不空闲；之后覆盖写到 R 离开根环，
+/// 每一步之后没有一个槽同时算进「已分配」与第九项、R 还在根环里时复活的槽不空闲、池级 checker 不红 I-7.4 与 I-4.8、写一次都不拒。
+/// 抬 F 那一串回收之后不补隔离时，推过抬 F 那一句先红。
+#[test]
+fn revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero(
+) {
+    let RecordedRollback {
+        recorded, revived, ..
+    } = record_one_rollback_after_overwrites(
+        OVERWRITES_BEFORE_THE_ROLLBACK_WHOSE_PLACEMENTS_A_FLOOR_RAISE_RECLAIMS,
+        VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
+    );
+    assert!(
+        !revived.is_empty(),
+        "回退目标的数据单元 P 不引用：回退复活了它们（阳性对照：这一形走到了复活）"
+    );
+    let lineage_unit_writes = lineage_unit_writes_of(&recorded);
+    assert!(
+        !lineage_unit_writes.is_empty(),
+        "回退那次发布写了谱系片（阳性对照：故障有处可打）"
+    );
+    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
+    let mut devices = devices_reading_back_all_zero_on(
+        &crash_image,
+        &RangesReadingBackAllZero::default()
+            .with_the_root_slot_of(&crash_image, recorded.key)
+            .with_the_writes(&lineage_unit_writes),
+    );
+    let mounted = mount_writable_with_space_admission(
+        &recorded.pool.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("R 的根槽与谱系片读回全零而记录齐：可写挂载落到 P、做得成");
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        recorded.previous_key,
+        "R 的根槽读不出、系统配置没见证 R、记录不施加：择根落到 P（阳性对照：故障打到了）"
+    );
+    let isolated_by_the_unapplied_records = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("挂载建的分配器装着根环表")
+        .placements_named_by_unapplied_journal_records();
+    let revived_not_in_c: Vec<&PlacementOnDevice> = revived
+        .iter()
+        .filter(|revived_placement| {
+            !isolated_by_the_unapplied_records.iter().any(|placement| {
+                placement.device == revived_placement.device
+                    && placement.slot == revived_placement.slot
+            })
+        })
+        .collect();
+    assert!(
+        revived_not_in_c.is_empty(),
+        "C 那一圈并进了 R 的账（记录的新根段读得出），R 复活的数据单元都在里面（阳性对照），缺 {revived_not_in_c:?}"
+    );
+    assert_eq!(
+        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
+        0,
+        "挂载那一刻没有一个槽同时算进「已分配」与第九项"
+    );
+    // R 的根槽与谱系片挂上之后照旧读回全零，到这个进程写掉那几段为止（写过之后照读）：抬 F 那一串读不出 R，不把它当被抛弃根算影子账。
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
+    // 挂上之后先推一串抬 F 到准入上限（攻方动作 `F`）。
+    let mut session_raising_the_floor = pool.session.take().expect("挂着的会话");
+    let raised = {
+        let PoolVersion::WithFile(current) = &mut session_raising_the_floor.current else {
+            panic!("挂载落到的 P 带文件");
+        };
+        raise_rollback_floor_to_the_admission_ceiling(
+            &pool.parameters,
+            &mut pool.devices,
+            &mut session_raising_the_floor.allocator,
+            current,
+            ShadowLedger::On,
+        )
+    };
+    pool.session = Some(session_raising_the_floor);
+    let reclaimed_by_the_raise = match raised {
+        Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => {
+            pool.note_floor_raises(std::slice::from_ref(&raised));
+            raised.reclaimed
+        }
+        Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) => {
+            panic!(
+                "挂上之后准入上限高于 F、推得出一串抬 F（阳性对照）：F {floor:?}、上限 {ceiling:?}"
+            )
+        }
+        Err(refusal) => panic!("挂上之后抬 F 到准入上限做得成：{refusal:?}"),
+    };
+    assert!(
+        revived.iter().any(|revived_placement| {
+            reclaimed_by_the_raise
+                .iter()
+                .any(|reclaimed| reclaimed.slot == revived_placement.slot)
+        }),
+        "这一串抬 F 回收了复活的数据单元（阳性对照：回收时补隔离那一处走到了），回收的 {reclaimed_by_the_raise:?}"
+    );
+    assert!(
+        root_is_in_the_ring(&pool.image(), recorded.key),
+        "推过那一串抬 F 时 R 还在根环里"
+    );
+    let free_among_the_revived_after_the_raise =
+        free_slots_covered_by(&pool.session().allocator, &revived);
+    assert!(
+        free_among_the_revived_after_the_raise.is_empty(),
+        "推过抬 F 那一刻 R 还在根环里：复活的数据单元覆盖的槽一个都不空闲（那一串回收时照 C 那一圈补了隔离），空闲的 {free_among_the_revived_after_the_raise:?}"
+    );
+    assert_eq!(
+        slots_counted_both_as_allocated_and_as_isolated(&pool.session().allocator),
+        0,
+        "推过抬 F 之后没有一个槽同时算进「已分配」与第九项"
+    );
+    let mut overwrites = 0;
+    // 迭代上界 OVERWRITES_UNTIL_R_LEAVES_THE_RING_AT_MOST；跨轮只带计数；出口：R 离开根环，
+    // 被拒、双扣、复活的槽空闲、checker 红都当场判红（panic）。
+    while root_is_in_the_ring(&pool.image(), recorded.key) {
+        assert!(
+            overwrites < OVERWRITES_UNTIL_R_LEAVES_THE_RING_AT_MOST,
+            "{OVERWRITES_UNTIL_R_LEAVES_THE_RING_AT_MOST} 次覆盖写之内 R 那一槽写掉、R 离开根环"
+        );
+        pool.overwrite(OVERWRITE_BYTES)
+            .unwrap_or_else(|refusal| panic!("第 {} 次覆盖写被拒：{refusal:?}", overwrites + 1));
+        overwrites += 1;
+        assert_eq!(
+            slots_counted_both_as_allocated_and_as_isolated(&pool.session().allocator),
+            0,
+            "第 {overwrites} 次覆盖写之后没有一个槽同时算进「已分配」与第九项"
+        );
+        if root_is_in_the_ring(&pool.image(), recorded.key) {
+            let free_among_the_revived = free_slots_covered_by(&pool.session().allocator, &revived);
+            assert!(
+                free_among_the_revived.is_empty(),
+                "第 {overwrites} 次覆盖写之后 R 还在根环里：复活的数据单元覆盖的槽一个都不空闲，空闲的 {free_among_the_revived:?}"
+            );
+        }
+        let violations = violations_of_unit_reuse_on(&pool.image());
+        assert!(
+            violations.is_empty(),
+            "第 {overwrites} 次覆盖写之后被抛弃根引用的单元一个都没复用：{violations:?}"
+        );
+    }
+}
````

### 第十四块 mutations-append.tsv

````
第10z批第十四块 H2 挂载那一刻 C 那一圈照旧隔离分配位置着的槽（1536 槽盘先释放一大批再认不出那一形）	crates/singlefs-core/src/mount.rs	        allocator\n            .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(\n                placement.device,\n                placement.slot,\n                placement.span,\n            );	        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- --include-ignored wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices	wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
第10z批第十四块 H2 挂载那一刻 C 那一圈照旧隔离分配位置着的槽（第九项与已分配双扣）	crates/singlefs-core/src/mount.rs	        allocator\n            .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(\n                placement.device,\n                placement.slot,\n                placement.span,\n            );	        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices	wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices
第10z批第十四块 H2 C 那一圈隔离时不看分配位（分配器那一判拿掉）	crates/singlefs-core/src/allocator.rs	            if !self.allocated[index] {\n                self.isolate_slot_at_index(index);\n            }	            self.isolate_slot_at_index(index);	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices	wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices
第10z批第十四块 H2 抬 F 那一串回收之后不补隔离（挂载那一刻跳过的 defer 槽回收出去）	crates/singlefs-core/src/mount.rs	        ShadowLedger::On => allocator\n            .reclaim_released_up_to_isolating_what_the_isolation_still_protects(floor, reuse),	        ShadowLedger::On => allocator.reclaim_released_up_to(floor, reuse),	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero	revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero
第10z批第十四块 H2 分配器回收之后补隔离那一步拿掉（抬 F 那一串）	crates/singlefs-core/src/allocator.rs	        self.isolate_reclaimed_slots_the_isolation_still_protects(\n            &reclaimed,\n            &still_protected_by_the_isolation,\n        );\n        reclaimed\n            .into_iter()	        reclaimed\n            .into_iter()	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero	revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero
第10z批第十四块 K2 读盘一层只看第一个打包单元（删除发布墓碑片读不出那一形）	crates/singlefs-core/src/mount.rs	.filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)\n        .map(\n	.filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)\n        .take(1)\n        .map(\n	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_deletion_whose_tombstone_unit_and_account_read_back_zero_at_the_mount_takes_every_released_placement	a_deletion_whose_tombstone_unit_and_account_read_back_zero_at_the_mount_takes_every_released_placement
第10z批第十四块 K2 读盘一层只看第一条记录点名的打包单元（顺序写末条点名叶容器那一形）	crates/singlefs-core/src/mount.rs	        .iter()\n        .flat_map(|record| record.named.iter())\n        .filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)	        .iter()\n        .take(1)\n        .flat_map(|record| record.named.iter())\n        .filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)	-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table -- a_sequential_write_whose_inode_unit_named_by_its_last_record_and_account_read_back_zero_at_the_mount_takes_every_released_placement	a_sequential_write_whose_inode_unit_named_by_its_last_record_and_account_read_back_zero_at_the_mount_takes_every_released_placement
第10z批第十四块 K2 读得出而解不开的打包单元当解开了（旧写法 ok() 吞掉）	crates/singlefs-core/src/mount.rs	                    Err(_unparsable) => PackedUnitNamedByAPublishAsRead::ReadButUnparsable,	                    Err(_unparsable) => PackedUnitNamedByAPublishAsRead::Parsed {\n                        record_type: 0,\n                        records: Vec::new(),\n                    },	-p singlefs-core --lib -- every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart	every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart
第10z批第十四块 K2 读盘一层只看第一个打包单元（读盘一层的库用例）	crates/singlefs-core/src/mount.rs	.filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)\n        .map(\n	.filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)\n        .take(1)\n        .map(\n	-p singlefs-core --lib -- every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart	every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart
第10z批第十四块 K2 读盘一层只看第一条记录点名的打包单元（读盘一层的库用例）	crates/singlefs-core/src/mount.rs	        .iter()\n        .flat_map(|record| record.named.iter())\n        .filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)	        .iter()\n        .take(1)\n        .flat_map(|record| record.named.iter())\n        .filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)	-p singlefs-core --lib -- every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart	every_packed_unit_named_by_every_record_is_read_in_order_and_an_unparsable_one_is_told_apart
````

### 第十四块 mutations-replacements.tsv

````
步 5：抬 F 回收的槽不扣住、生效之前就能发出去	crates/singlefs-core/src/mount.rs	        ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => {\n            reclaim_released_in_the_sequence_raising_the_floor_up_to(\n                allocator,\n                shadow_ledger,\n                floor_of_the_reclaim,\n                ReclaimedReuse::HeldUntilFloorTakesEffect,\n            )\n        }	        ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => {\n            reclaim_released_in_the_sequence_raising_the_floor_up_to(\n                allocator,\n                shadow_ledger,\n                floor_of_the_reclaim,\n                ReclaimedReuse::Immediately,\n            )\n        }	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored slots_reclaimed_by_raising	slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect
C518（一次挂载之内环转过一圈之后不回收）：挂载内回收出来、被抛弃根还引用着的槽不补隔离（D23 已定项 14 主句）	crates/singlefs-core/src/allocator.rs	            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);\n        self.isolate_reclaimed_slots_the_isolation_still_protects(\n            &reclaimed,\n            &still_protected_by_the_isolation,\n        );	            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);\n        let _ = (&reclaimed, &still_protected_by_the_isolation);	-p singlefs-harness --test unreadable_abandoned_root_slot -- slot_reclaimed_while_an_abandoned_root_still_references	slot_reclaimed_while_an_abandoned_root_still_references_it_stays_isolated_until_that_root_leaves_the_ring
````

