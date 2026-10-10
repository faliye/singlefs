
    /// 攻方随机模型（m3-batch10z-no-code-r1，只在草稿副本里）：两张假卡、照卡池的走法（借之前那一步看池里每张卡 → 借第一张放得下的 →
    /// 留着的设备没有或丢了就照 `DeviceStartedOnDemand::started_or_start` 重起 → 借据拿着设备的克隆 → 建 → 等成或没等成再还），
    /// 另有设备随机丢、别的进程随机占卡、卡池 poll 随机交回空了。每借到一笔核：借的不超过借那一刻卡上真能再给的（多给）；
    /// 每次两张都借不到核：有没有一张卡真放得下（少给，按那张卡的账走到哪一步分）；借到之后重起被拒数一次；卡上实占超过总量数一次（OOM）。
    /// 环境变量 OPUS_NO_GPU_SEEDS、OPUS_NO_GPU_STEPS；OPUS_NO_GPU_ARM=listed_only 时借之前那一步只看列出来的卡（第十五块之前的走法）。
    #[test]
    fn opus_no_random_two_card_pool_model() {
        use std::cell::Cell;
        use std::rc::Rc;
        struct TDev { card: usize, driver: u64, built: u64, leftover: u64, released: bool, lost: bool }
        struct Truth { others: [u64; 2], listed: [bool; 2], devs: Vec<TDev> }
        impl Truth {
            fn occupied(&self, c: usize) -> u64 {
                self.devs.iter().filter(|d| d.card == c && !d.released).map(|d| d.driver + d.built + d.leftover).sum()
            }
            fn reading(&self, c: usize) -> CardMemoryReading {
                let occ = self.occupied(c);
                CardMemoryReading {
                    free_mebibytes: AN_EMPTY_RTX_5090_HAS.saturating_sub(self.others[c] + occ),
                    total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
                    occupied_by_this_process: if occ > 0 && self.listed[c] { OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: occ } } else { OccupiedByThisProcess::NotListed },
                }
            }
        }
        struct FakeDev<'l> { on_books: DeviceStartedOnTheBooks<'l>, id: usize, truth: Arc<Mutex<Truth>> }
        impl Drop for FakeDev<'_> {
            fn drop(&mut self) { self.truth.lock().expect("真值").devs[self.id].released = true; }
        }
        struct Live<'l> { dev: Rc<FakeDev<'l>>, lease: GpuMemoryBooking<'l>, need: u64, built: u64 }
        const STARTED_DRIVER: u64 = 600;
        let seeds: u64 = std::env::var("OPUS_NO_GPU_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(300);
        let steps: usize = std::env::var("OPUS_NO_GPU_STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(300);
        let arm = std::env::var("OPUS_NO_GPU_ARM").unwrap_or_default();
        let mut c: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
        let mut worst_over = 0u64;
        for seed in 0..seeds {
            let rng = Cell::new(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let next = |n: u64| { let mut s = rng.get(); s ^= s << 13; s ^= s >> 7; s ^= s << 17; rng.set(s); s % n };
            let truth = Arc::new(Mutex::new(Truth { others: [next(6000), next(6000)], listed: [next(4) != 0, next(4) != 0], devs: Vec::new() }));
            let read_truth = Arc::clone(&truth);
            let ledger = GpuMemoryLedger::new(
                2,
                Box::new(move || { let t = read_truth.lock().expect("真值"); Ok(vec![t.reading(0), t.reading(1)]) }),
                Box::new(|_card_position, may_hold| card_ledger_capacity_mebibytes(None, None, may_hold, may_hold, DRIVER)),
                DRIVER,
            );
            let slots: [DeviceStartedOnDemand<Rc<FakeDev<'_>>>; 2] = [DeviceStartedOnDemand::new(), DeviceStartedOnDemand::new()];
            let mut live: Vec<Live<'_>> = Vec::new();
            for _step in 0..steps {
                match next(10) {
                    0..=3 => {
                        // 借之前那一步（卡池 take_in_the_devices_found_idle 的四样）
                        let poll = |d: &Rc<FakeDev<'_>>| {
                            let idle = next(2) == 0;
                            let mut t = truth.lock().expect("真值");
                            if idle && !t.devs[d.id].lost { t.devs[d.id].leftover = 0; }
                            idle
                        };
                        if arm == "listed_only" {
                            let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
                            for l in &listed {
                                let Some(kept) = slots[l.card_position()].started_now() else { continue };
                                if truth.lock().expect("真值").devs[kept.id].lost { kept.on_books.take_in_that_the_device_was_found_lost(); continue; }
                                if poll(&kept) { kept.on_books.take_in_that_the_work_of_the_listed_leases_finished(l); }
                            }
                        } else {
                            ledger.take_in_the_devices_kept_before_booking(
                                |p| slots[p].started_now(),
                                |d: &Rc<FakeDev<'_>>| truth.lock().expect("真值").devs[d.id].lost,
                                poll,
                                |d: &Rc<FakeDev<'_>>| &d.on_books,
                            );
                        }
                        let need = 200 + next(14_000);
                        let truly: Vec<u64> = (0..2).map(|card| {
                            let t = truth.lock().expect("真值");
                            let kept = slots[card].started_now();
                            let growth = match kept.as_ref().filter(|k| !t.devs[k.id].lost && !t.devs[k.id].released) { Some(k) => DRIVER - t.devs[k.id].driver, None => DRIVER };
                            let unbuilt: u64 = live.iter().filter(|l| l.lease.card_position() == card && !t.devs[l.dev.id].lost).map(|l| l.need - l.built).sum();
                            t.reading(card).free_mebibytes.saturating_sub(growth).saturating_sub(unbuilt)
                        }).collect();
                        let states: Vec<String> = (0..2).map(|card| { let b = ledger.books_locked(); let listed = truth.lock().expect("真值").listed[card]; format!("{:?}/lostkept={}/failedwait={}/listed={listed}", b[card].driver_share, b[card].lost_devices_not_released.len().min(2), b[card].leases_kept_after_a_failed_wait.len().min(1)) }).collect();
                        match ledger.book_on_the_first_card_with_room(need) {
                            Ok(lease) => {
                                *c.entry("bookings".into()).or_default() += 1;
                                let card = lease.card_position();
                                if need > truly[card] {
                                    *c.entry(format!("over@{}", states[card])).or_default() += 1;
                                    worst_over = worst_over.max(need - truly[card]);
                                }
                                let started = slots[card].started_or_start(
                                    |d: &Rc<FakeDev<'_>>| truth.lock().expect("真值").devs[d.id].lost,
                                    || {
                                        ledger.prepare_the_driver_share_before_starting_the_device(card).map_err(|_| ())?;
                                        let on_books = ledger.record_the_device_started_on_the_card(card);
                                        let mut t = truth.lock().expect("真值");
                                        t.devs.push(TDev { card, driver: STARTED_DRIVER, built: 0, leftover: 0, released: false, lost: false });
                                        let id = t.devs.len() - 1;
                                        Ok::<_, ()>(Rc::new(FakeDev { on_books, id, truth: Arc::clone(&truth) }))
                                    },
                                );
                                match started {
                                    Ok(dev) => live.push(Live { dev, lease, need, built: 0 }),
                                    Err(()) => { *c.entry(format!("restart_refused_after_booking@{}", states[card])).or_default() += 1; drop(lease); }
                                }
                            }
                            Err(_) => {
                                *c.entry("refusals".into()).or_default() += 1;
                                for card in 0..2 {
                                    if truly[card] >= need { *c.entry(format!("under@card{card}:{}", states[card])).or_default() += 1; break; }
                                }
                            }
                        }
                    }
                    4 | 5 => {
                        // 建满一笔（设备没丢）
                        let n = live.len() as u64;
                        if n > 0 {
                            let i = next(n) as usize;
                            let l = &mut live[i];
                            let mut t = truth.lock().expect("真值");
                            if !t.devs[l.dev.id].lost && l.built < l.need { t.devs[l.dev.id].built += l.need - l.built; l.built = l.need; }
                        }
                    }
                    6 | 7 => {
                        // 还一笔：随机等成或没等成（设备丢了照没等成）
                        let n = live.len() as u64;
                        if n > 0 {
                            let mut l = live.swap_remove(next(n) as usize);
                            let failed = { let t = truth.lock().expect("真值"); t.devs[l.dev.id].lost } || next(3) == 0;
                            {
                                let mut t = truth.lock().expect("真值");
                                let d = &mut t.devs[l.dev.id];
                                if !d.lost {
                                    d.built -= l.built;
                                    if failed { d.leftover += l.built; } else { d.leftover = 0; }
                                }
                            }
                            l.lease.take_in_the_wait_before_returning(&l.dev.on_books, if failed { DeviceWaitBeforeReturningTheLease::TheWaitFailed } else { DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished });
                            let Live { dev, lease, .. } = l;
                            drop(dev);
                            drop(lease);
                        }
                    }
                    8 => {
                        // 一张卡上在用的设备丢了
                        let card = next(2) as usize;
                        if let Some(k) = slots[card].started_now() { if next(3) == 0 { truth.lock().expect("真值").devs[k.id].lost = true; } }
                    }
                    _ => {
                        // 别的进程只拿卡上此刻空着的（或放掉一些）：不由它把卡撑爆
                        // 只拿此刻没许给这个进程的（空闲减驱动还要长的、减活着而没建的）：两进程同读一份空闲那一类（用户已认）不进这个模型
                        let card = next(2) as usize;
                        let unbuilt: u64 = { let t = truth.lock().expect("真值"); live.iter().filter(|l| l.lease.card_position() == card && !t.devs[l.dev.id].lost).map(|l| l.need - l.built).sum() };
                        let mut t = truth.lock().expect("真值");
                        let free = t.reading(card).free_mebibytes.saturating_sub(DRIVER).saturating_sub(unbuilt);
                        t.others[card] = next(t.others[card] + free.min(9000) + 1);
                    }
                }
                let t = truth.lock().expect("真值");
                for card in 0..2 { if t.others[card] + t.occupied(card) > AN_EMPTY_RTX_5090_HAS { *c.entry(format!("physically_overfull_card{card}")).or_default() += 1; } }
            }
            drop(live);
        }
        println!("OPUS_NO_GPU arm={} seeds={seeds} steps={steps} worst_over_mebibytes={worst_over} {:?}", if arm.is_empty() { "today" } else { &arm }, c);
    }
