
    /// 攻方并发压测（m3-batch10z-no-code-r1，只在草稿副本里）：两张假卡、几条线程同时走「借之前那一步（看池里每张卡）→ 借第一张放得下的 →
    /// 取设备（丢了就重起）→ 随机让设备丢 → 还（随机没等成）」。核：没丢的设备从没被账记成丢了；借之前那一步里那条「留着的设备不在取它的那张卡上」
    /// 的断言一次都不响（线程 panic 数）；收尾时丢了而账还记着在用的设备，再走一遍借之前那一步就记上。
    /// 环境变量 OPUS_NO_THREADS、OPUS_NO_ROUNDS。
    #[test]
    fn opus_no_two_card_early_lost_mark_racing_the_restart() {
        type Kept<'l> = std::sync::Arc<(DeviceStartedOnTheBooks<'l>, std::sync::atomic::AtomicBool, u64, usize)>;
        let threads: usize = std::env::var("OPUS_NO_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
        let rounds: usize = std::env::var("OPUS_NO_ROUNDS").ok().and_then(|v| v.parse().ok()).unwrap_or(2000);
        let cards = [Arc::new(Mutex::new(CardUnderTheLedger::default())), Arc::new(Mutex::new(CardUnderTheLedger::default()))];
        let read = [Arc::clone(&cards[0]), Arc::clone(&cards[1])];
        let ledger = GpuMemoryLedger::new(
            2,
            Box::new(move || Ok(vec![read[0].lock().expect("假卡").reading(), read[1].lock().expect("假卡").reading()])),
            Box::new(|_card_position, may_hold| card_ledger_capacity_mebibytes(None, None, may_hold, may_hold, DRIVER)),
            DRIVER,
        );
        let slots: [DeviceStartedOnDemand<Kept<'_>>; 2] = [DeviceStartedOnDemand::new(), DeviceStartedOnDemand::new()];
        let registry: Mutex<Vec<Kept<'_>>> = Mutex::new(Vec::new());
        let wrongly_lost = AtomicUsize::new(0);
        let restarts = AtomicUsize::new(0);
        let restart_refused = AtomicUsize::new(0);
        let bookings = AtomicUsize::new(0);
        let panics = AtomicUsize::new(0);
        std::thread::scope(|scope| {
            for t in 0..threads {
                let (ledger, slots, registry, wrongly_lost, restarts, restart_refused, bookings, panics) =
                    (&ledger, &slots, &registry, &wrongly_lost, &restarts, &restart_refused, &bookings, &panics);
                scope.spawn(move || {
                    let mut state = (t as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
                    let mut next = move |n: u64| { state ^= state << 13; state ^= state >> 7; state ^= state << 17; state % n };
                    for _round in 0..rounds {
                        let idle = next(2) == 0;
                        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            ledger.take_in_the_devices_kept_before_booking(
                                |p| slots[p].started_now(),
                                |k: &Kept<'_>| k.1.load(Ordering::SeqCst),
                                |_k: &Kept<'_>| idle,
                                |k: &Kept<'_>| &k.0,
                            );
                        }));
                        if outcome.is_err() { panics.fetch_add(1, Ordering::Relaxed); }
                        let Ok(mut lease) = ledger.book_on_the_first_card_with_room(100 + next(400)) else { continue; };
                        bookings.fetch_add(1, Ordering::Relaxed);
                        let card = lease.card_position();
                        let started = slots[card].started_or_start(
                            |k: &Kept<'_>| k.1.load(Ordering::SeqCst),
                            || {
                                ledger.prepare_the_driver_share_before_starting_the_device(card).map_err(|_| ())?;
                                let on_the_books = ledger.record_the_device_started_on_the_card(card);
                                let number = on_the_books.device.0;
                                let kept: Kept<'_> = std::sync::Arc::new((on_the_books, std::sync::atomic::AtomicBool::new(false), number, card));
                                registry.lock().expect("登记").push(kept.clone());
                                restarts.fetch_add(1, Ordering::Relaxed);
                                Ok::<_, ()>(kept)
                            },
                        );
                        let Ok(device) = started else { restart_refused.fetch_add(1, Ordering::Relaxed); continue; };
                        if next(20) == 0 { device.1.store(true, Ordering::SeqCst); }
                        lease.take_in_the_wait_before_returning(&device.0, if next(3) == 0 { DeviceWaitBeforeReturningTheLease::TheWaitFailed } else { DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished });
                        drop(lease);
                        let books = ledger.books_locked();
                        for kept in registry.lock().expect("登记").iter() {
                            if !kept.1.load(Ordering::SeqCst) && books[kept.3].lost_devices_not_released.contains(&super::DeviceNumberOnTheCard(kept.2)) {
                                wrongly_lost.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        drop(books);
                        drop(device);
                    }
                });
            }
        });
        let left: Vec<Option<Kept<'_>>> = (0..2).map(|p| slots[p].started_now()).collect();
        ledger.take_in_the_devices_kept_before_booking(|p| left[p].clone(), |k: &Kept<'_>| k.1.load(Ordering::SeqCst), |_k| false, |k: &Kept<'_>| &k.0);
        let books = ledger.books_locked();
        let lost_still_in_use = (0..2).filter(|p| left[*p].as_ref().is_some_and(|k| k.1.load(Ordering::SeqCst) && books[*p].device_in_use == Some(super::DeviceNumberOnTheCard(k.2)))).count();
        println!("OPUS_NO_STRESS threads={threads} rounds={rounds} bookings={} restarts={} restart_refused={} wrongly_lost={} panics={} lost_and_still_in_use_after_one_more_pass={lost_still_in_use}",
            bookings.load(Ordering::Relaxed), restarts.load(Ordering::Relaxed), restart_refused.load(Ordering::Relaxed), wrongly_lost.load(Ordering::Relaxed), panics.load(Ordering::Relaxed));
        drop(books);
        drop(left);
        registry.lock().expect("登记").clear();
    }
