
    // ── 攻方腿模型（c996-code-r3）：接在 background_gpu.rs 的测试模块末尾，只在冻结副本的拷贝里，不进仓 ──
    // 前两轮攻方腿用例在今天的代码上重跑：历史与 name= 行的键照原样（c996-code-r2 那一份的九条、bggpu-audit-fix-code-r1 那一份的三条），
    // 断言换成条款要的结局（用例过 = 修掉了，红 = 还中）；改了断言的那几处在注释里写原读数。名字带 attack_r3_ 的是这一轮新加的。

    /// 攻方腿模型：切换点到的卡开卡那一步——头 4 次当场开不成，之后每一次先等用例放开（至多 20 秒）再开不成。
    fn attack_opening_not_opened_four_times_then_held(
        opening_released: std::sync::mpsc::Receiver<()>,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        let mut openings_so_far: u32 = 0;
        move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
            openings_so_far += 1;
            if openings_so_far > 4 {
                let _released_or_not = opening_released.recv_timeout(Duration::from_secs(20));
            }
            Err(reason_the_card_does_not_open_at_the_switch())
        }
    }

    /// 攻方腿模型：panic 带的原话（`&str` 或 `String`；别的交回空串）。
    fn attack_said_by_the_panic(payload: &(dyn std::any::Any + Send)) -> String {
        payload
            .downcast_ref::<&str>()
            .map(|said| (*said).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_default()
    }

    /// 攻方腿模型：「再交一次开不成的那一组」那段历史的读数。
    struct AttackReadingsOfTheRetry {
        not_opened_after_the_reopenings: bool,
        gpus_opened_after_not_opened: Vec<GpuDeviceIdentifier>,
        still_opening_at_the_retry: bool,
        retry_answered_used_up: bool,
        batch_panicked_with: Option<String>,
        devices_given_work: Vec<GpuDeviceIdentifier>,
    }

    /// 同 c996-code-r2 攻方腿那一段历史：挂载开着「0000:01:00.0」，核心层挂载那一刻交来点它的那一组；`switch_to_the_cpu_first` 时先告知一张卡都不参与；
    /// 切换只点没开的「0000:41:00.0」，头一次与 3 次重开都开不成；用户再交同一组；之后照外壳直接交一批校验和那一段。
    fn attack_history_of_the_retry_after_the_switch_not_opened(
        switch_to_the_cpu_first: bool,
    ) -> AttackReadingsOfTheRetry {
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                attack_opening_not_opened_four_times_then_held(opening_released),
                None,
                false,
            );
        let at_the_mount =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&at_the_mount);
        if switch_to_the_cpu_first {
            background_gpu.use_no_gpu_from_the_next_batch();
        }
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered_after_the_reopenings = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds,
            );
        let gpus_opened_after_not_opened = background_gpu.gpus_opened();
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered_at_the_retry = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll);
        let retry_answered_used_up = matches!(
            &answered_at_the_retry,
            LatestSettingsHandedIn::NotOpened(not_opened)
                if not_opened.causes_of_each_opening
                    == vec![GpuFailureCause::OpeningsInThisMountUsedUp { failed_openings: 4 }]
        );
        let batch = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            checksum_batch_and_the_devices_given_work(&mut background_gpu).1
        }));
        release_the_opening
            .send(())
            .expect("开卡那一步还在外壳里：收的那一头还在");
        drop(release_the_opening);
        drop(
            background_gpu
                .opening_of_a_gpu_at_a_switch
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        let batch_panicked_with = batch
            .as_ref()
            .err()
            .map(|payload| attack_said_by_the_panic(&**payload));
        AttackReadingsOfTheRetry {
            not_opened_after_the_reopenings: matches!(
                answered_after_the_reopenings,
                LatestSettingsHandedIn::NotOpened(_)
            ),
            gpus_opened_after_not_opened,
            still_opening_at_the_retry: answered_at_the_retry
                == LatestSettingsHandedIn::StillOpeningItsCards,
            retry_answered_used_up,
            batch_panicked_with,
            devices_given_work: batch.unwrap_or_default(),
        }
    }

    /// 新一（GPU 包这一半）。原读数（任务 14649）：still_opening_at_the_retry=true batch_panicked_with=Some("参与的卡一张都不剩只有两种：…")。
    #[test]
    fn attack_the_retry_after_a_switch_not_opened_following_the_cpu_panics_in_the_next_gpu_batch() {
        let readings = attack_history_of_the_retry_after_the_switch_not_opened(true);
        println!(
            "name=attack_retry_after_not_opened arm=switched_to_the_cpu_first not_opened={} gpus_opened_after_not_opened={:?} still_opening_at_the_retry={} retry_answered_used_up={} batch_panicked_with={:?} devices_given_work={:?}",
            readings.not_opened_after_the_reopenings,
            readings.gpus_opened_after_not_opened,
            readings.still_opening_at_the_retry,
            readings.retry_answered_used_up,
            readings.batch_panicked_with,
            readings.devices_given_work
        );
        assert!(readings.not_opened_after_the_reopenings);
        assert!(readings.gpus_opened_after_not_opened.is_empty());
        assert!(!readings.still_opening_at_the_retry, "试满之后再交：当场答开不成");
        assert!(readings.retry_answered_used_up);
        assert_eq!(readings.batch_panicked_with, None, "不再 panic");
        assert!(readings.devices_given_work.is_empty());
    }

    /// 新二（GPU 包这一半）。原读数：still_opening_at_the_retry=true devices_given_work=[「0000:01:00.0」]。
    #[test]
    fn attack_the_retry_after_a_switch_not_opened_runs_the_batch_on_the_card_switched_away_from() {
        let readings = attack_history_of_the_retry_after_the_switch_not_opened(false);
        println!(
            "name=attack_retry_after_not_opened arm=stayed_on_the_gpu not_opened={} gpus_opened_after_not_opened={:?} still_opening_at_the_retry={} retry_answered_used_up={} batch_panicked_with={:?} devices_given_work={:?}",
            readings.not_opened_after_the_reopenings,
            readings.gpus_opened_after_not_opened,
            readings.still_opening_at_the_retry,
            readings.retry_answered_used_up,
            readings.batch_panicked_with,
            readings.devices_given_work
        );
        assert!(readings.not_opened_after_the_reopenings);
        assert_eq!(
            readings.gpus_opened_after_not_opened,
            vec![card_identifier("0000:01:00.0")]
        );
        assert!(!readings.still_opening_at_the_retry);
        assert!(readings.retry_answered_used_up);
        assert_eq!(readings.batch_panicked_with, None);
        assert!(readings.devices_given_work.is_empty(), "开不成照报错，一批都不分");
    }

    /// 攻方腿模型：产品那一份开一张卡那一步（一次至多等 20 秒），起卡那一件头一次 panic、之后开成替身卡；每被起一次往 `starts_made` 加一。
    fn attack_opening_panicking_at_its_first_start(
        starts_made: &Arc<std::sync::atomic::AtomicU32>,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        let starts_counted = Arc::clone(starts_made);
        opening_of_a_card_within_the_limit(
            Duration::from_secs(20),
            move |discrete_gpu: &DiscreteGpuOnThisHost,
                  memory_limit,
                  _wait_limit,
                  bytes_leaked_by_earlier_mounts| {
                let card = CardComputedOnTheCpuForTests::of_the_address(
                    &discrete_gpu.pci_bus_address,
                    memory_limit.map(|memory_limit| memory_limit.0),
                    bytes_leaked_by_earlier_mounts.0,
                );
                let starts_counted_by_the_start = Arc::clone(&starts_counted);
                move || {
                    let starts_before = starts_counted_by_the_start
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if starts_before == 0 {
                        panic!("攻方腿替身：起卡那一件头一次 panic");
                    }
                    Ok(card)
                }
            },
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            OpeningsNotReturnedInThisProcess::with_none_not_returned(),
        )
    }

    /// 新三。原读数（任务 14649）：arm=mount panicked=true starts=1；arm=switch panicked=false opened=true starts=2。
    #[test]
    fn attack_an_opening_panicking_at_the_mount_is_not_reopened_while_the_same_at_a_switch_is() {
        let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
        let starts_at_the_mount = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let mut opening_at_the_mount =
            attack_opening_panicking_at_its_first_start(&starts_at_the_mount);
        let mount = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            gpus_opened_at_the_mount(
                vec![discrete_gpu.clone()],
                &[ParticipatingGpu {
                    discrete_gpu: discrete_gpu.clone(),
                    memory_limit: None,
                }],
                bytes_of_the_smallest_piece_every_participating_gpu_holds(),
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                |_interval: Duration| {},
                |gpu_to_open, memory_limit_at_the_opening, cost_at_the_opening| {
                    opening_at_the_mount(
                        gpu_to_open,
                        memory_limit_at_the_opening,
                        BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                        cost_at_the_opening,
                    )
                },
            )
            .is_ok()
        }));
        let starts_at_the_switch = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let mut opening_at_the_switch =
            attack_opening_panicking_at_its_first_start(&starts_at_the_switch);
        let switch = gpu_opened_with_the_reopenings(
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            |_interval: Duration| {},
            || {
                opening_with_its_panic_caught(|| {
                    opening_at_the_switch(
                        &discrete_gpu,
                        None,
                        BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                        CostAtTheOpening::MeasuredUnderTheMemoryLimit,
                    )
                })
            },
        );
        let mount_starts = starts_at_the_mount.load(std::sync::atomic::Ordering::SeqCst);
        let switch_starts = starts_at_the_switch.load(std::sync::atomic::Ordering::SeqCst);
        let mount_opened = matches!(mount, Ok(true));
        println!(
            "name=attack_opening_panic arm=mount panicked={} opened={mount_opened} starts={mount_starts}",
            mount.is_err()
        );
        println!(
            "name=attack_opening_panic arm=switch panicked=false opened={} starts={switch_starts}",
            switch.is_ok()
        );
        assert!(mount.is_ok(), "挂载那一刻：开卡 panic 接住");
        assert!(mount_opened, "挂载那一刻：照重开、开成");
        assert_eq!(mount_starts, 2);
        assert!(switch.is_ok());
        assert_eq!(switch_starts, 2);
    }

    /// 攻方腿模型：产品那一份开一张卡那一步（一次至多等 50 毫秒），还没回来的开卡那一张表是 `openings_not_returned`；起卡那一件挂着直到用例放开，
    /// 每被起一次往 `starts_made` 加一、往 `start_begun` 送一声。
    fn attack_opening_held_until_released(
        start_released: &Arc<Mutex<std::sync::mpsc::Receiver<()>>>,
        starts_made: &Arc<std::sync::atomic::AtomicU32>,
        start_begun: &std::sync::mpsc::Sender<()>,
        openings_not_returned: &OpeningsNotReturnedInThisProcess,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        let start_released = Arc::clone(start_released);
        let starts_counted = Arc::clone(starts_made);
        let start_begun = start_begun.clone();
        opening_of_a_card_within_the_limit(
            Duration::from_millis(50),
            move |discrete_gpu: &DiscreteGpuOnThisHost,
                  memory_limit,
                  _wait_limit,
                  bytes_leaked_by_earlier_mounts| {
                let card = CardComputedOnTheCpuForTests::of_the_address(
                    &discrete_gpu.pci_bus_address,
                    memory_limit.map(|memory_limit| memory_limit.0),
                    bytes_leaked_by_earlier_mounts.0,
                );
                let start_released_seen_by_the_start = Arc::clone(&start_released);
                let starts_counted_by_the_start = Arc::clone(&starts_counted);
                let start_begun_by_the_start = start_begun.clone();
                move || {
                    starts_counted_by_the_start.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let _signal_sent = start_begun_by_the_start.send(());
                    let _released_or_not = start_released_seen_by_the_start
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .recv_timeout(Duration::from_secs(20));
                    Ok(card)
                }
            },
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            openings_not_returned.clone(),
        )
    }

    /// 壬：前一次挂载在「0000:41:00.0」上的一次开卡到点之后还挂着；同一次挂载里再开它、下一次挂载（另起一份开卡那一步）开它。
    /// `arm=shared_table` 两次挂载拿进程开卡器那一张表（产品照这样给），`arm=table_of_each_mount` 各拿一张（c996-code-r2 时的样子）。
    /// 原读数（任务 14649，各拿一张）：later_mount_first=Some(WaitForTheGpuExceededTheLimit…) starts_after_the_later_mount=2。
    #[test]
    fn attack_an_opening_hung_in_an_earlier_mount_does_not_keep_a_later_mount_from_starting_another(
    ) {
        for shared_table in [true, false] {
            let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
            let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
            let start_released = Arc::new(Mutex::new(start_released));
            let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
            let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
            let table_of_the_earlier_mount = OpeningsNotReturnedInThisProcess::with_none_not_returned();
            let table_of_the_later_mount = if shared_table {
                table_of_the_earlier_mount.clone()
            } else {
                OpeningsNotReturnedInThisProcess::with_none_not_returned()
            };
            let mut opening_of_the_earlier_mount = attack_opening_held_until_released(
                &start_released,
                &starts_made,
                &start_begun,
                &table_of_the_earlier_mount,
            );
            let mut opening_of_the_later_mount = attack_opening_held_until_released(
                &start_released,
                &starts_made,
                &start_begun,
                &table_of_the_later_mount,
            );
            let earlier_mount_first = opening_of_the_earlier_mount(
                &discrete_gpu,
                None,
                BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                CostAtTheOpening::MeasuredUnderTheMemoryLimit,
            )
            .err();
            every_start_begun
                .recv_timeout(Duration::from_secs(20))
                .expect("前一次挂载的那一次开卡 20 秒内起了");
            let earlier_mount_again = opening_of_the_earlier_mount(
                &discrete_gpu,
                None,
                BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                CostAtTheOpening::MeasuredUnderTheMemoryLimit,
            )
            .err();
            let starts_within_the_earlier_mount =
                starts_made.load(std::sync::atomic::Ordering::SeqCst);
            let later_mount_first = opening_of_the_later_mount(
                &discrete_gpu,
                None,
                BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                CostAtTheOpening::MeasuredUnderTheMemoryLimit,
            )
            .err();
            let later_mount_started = every_start_begun
                .recv_timeout(Duration::from_millis(500))
                .is_ok();
            let starts_after_the_later_mount = starts_made.load(std::sync::atomic::Ordering::SeqCst);
            for _hung_start in 0..2 {
                let _sent_or_nobody_waits = release_the_start.send(());
            }
            println!(
                "name=attack_hung_opening_across_mounts arm={} earlier_mount_first={earlier_mount_first:?} earlier_mount_again={earlier_mount_again:?} starts_within_the_earlier_mount={starts_within_the_earlier_mount} later_mount_first={later_mount_first:?} later_mount_started={later_mount_started} starts_after_the_later_mount={starts_after_the_later_mount}",
                if shared_table { "shared_table" } else { "table_of_each_mount" }
            );
            assert!(matches!(
                earlier_mount_first,
                Some(GpuFailureCause::WaitForTheGpuExceededTheLimit { .. })
            ));
            assert_eq!(
                earlier_mount_again,
                Some(opening_not_started_while_an_earlier_one_has_not_returned())
            );
            assert_eq!(starts_within_the_earlier_mount, 1);
            if shared_table {
                assert_eq!(
                    later_mount_first,
                    Some(GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned),
                    "进程级那一张表：下一次挂载不再起"
                );
                assert_eq!(starts_after_the_later_mount, 1);
            } else {
                assert_eq!(starts_after_the_later_mount, 2, "各拿一张表：照起");
            }
        }
    }

    /// 丁（同一组设置在那一问之前又交一次）。原读数（任务 14649）：openings_when_handed_in_again=4 causes_reported=Some(4) openings_after_the_poll=8。
    /// 条款（第 4 条现行文本）：连着开不成满 4 次之后再有切换点到它不再开，当场答开不成，原因是单独的一个成员。
    #[test]
    fn attack_repaired_stale_failure_handed_in_again_before_the_poll() {
        let (opened_four_times, every_opening_made) = std::sync::mpsc::channel::<()>();
        let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let openings_counted = Arc::clone(&openings_made);
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    let opening =
                        openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    if opening == 4 {
                        let _signal_sent = opened_four_times.send(());
                    }
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        every_opening_made
            .recv_timeout(Duration::from_secs(20))
            .expect("开卡线程 20 秒内开到第 4 次");
        let openings_when_handed_in_again = openings_made_so_far(&cards_handed_to_the_opening);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let openings_after_the_poll = openings_made_so_far(&cards_handed_to_the_opening);
        let causes_reported = if let LatestSettingsHandedIn::NotOpened(not_opened) = &answered {
            Some(not_opened.causes_of_each_opening.clone())
        } else {
            None
        };
        println!(
            "name=attack_repaired_stale_failure arm=handed_in_again_before_the_poll openings_when_handed_in_again={openings_when_handed_in_again} causes_reported={:?} causes={causes_reported:?} openings_after_the_poll={openings_after_the_poll}",
            causes_reported.as_ref().map(Vec::len)
        );
        assert_eq!(openings_when_handed_in_again, 4);
        assert_eq!(
            causes_reported,
            Some(vec![GpuFailureCause::OpeningsInThisMountUsedUp { failed_openings: 4 }])
        );
        assert_eq!(openings_after_the_poll, 4, "试满之后不再开");
    }

    /// 丁的对照臂：那一问在又交之前问到了。原读数（任务 14649）：not_opened_before=true not_opened_after=true openings=8。
    #[test]
    fn attack_repaired_stale_failure_handed_in_again_after_the_poll() {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered_before = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered_after = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let openings = openings_made_so_far(&cards_handed_to_the_opening);
        let not_opened_before = matches!(answered_before, LatestSettingsHandedIn::NotOpened(_));
        let not_opened_after = matches!(answered_after, LatestSettingsHandedIn::NotOpened(_));
        println!(
            "name=attack_repaired_stale_failure arm=handed_in_again_after_the_poll not_opened_before={not_opened_before} not_opened_after={not_opened_after} openings={openings}"
        );
        assert!(not_opened_before && not_opened_after);
        assert_eq!(openings, 4, "试满之后再交不再开");
    }

    /// 戊 ②（为被盖掉的切换开成的卡）。原读数（任务 14649）：drops_of_forty_one=1，不变。
    #[test]
    fn attack_repaired_superseded_card() {
        let (release_the_card_at_forty_one, card_at_forty_one_released) =
            std::sync::mpsc::channel::<()>();
        let drops_of_the_card_at_forty_one = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let drops_noted_by_the_opening = Arc::clone(&drops_of_the_card_at_forty_one);
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                        card_at_forty_one_released
                            .recv_timeout(Duration::from_secs(20))
                            .map_err(|_not_released| {
                                reason_the_card_does_not_open_at_the_switch()
                            })?;
                        let mut card = opening_that_always_opens(
                            discrete_gpu,
                            memory_limit,
                            wait_limit,
                            cost_at_the_opening,
                        )?;
                        card.drops_counted = Arc::clone(&drops_noted_by_the_opening);
                        return Ok(card);
                    }
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        let to_the_card_at_eighty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
        release_the_card_at_forty_one
            .send(())
            .expect("开「0000:41:00.0」那一步还在等放开");
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let device_not_opened = if let LatestSettingsHandedIn::NotOpened(not_opened) = &answered {
            Some(not_opened.device.clone())
        } else {
            None
        };
        let gpus_opened = background_gpu.gpus_opened();
        let participating = participating_devices_of(&background_gpu);
        let drops = drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst);
        println!(
            "name=attack_repaired_superseded_card device_not_opened={device_not_opened:?} gpus_opened={gpus_opened:?} participating={participating:?} drops_of_forty_one={drops} openings={:?}",
            cards_handed_to_the_opening
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        );
        assert_eq!(device_not_opened, Some(card_identifier("0000:81:00.0")));
        assert_eq!(gpus_opened, vec![card_identifier("0000:01:00.0")]);
        assert_eq!(participating, vec![card_identifier("0000:01:00.0")]);
        assert_eq!(drops, 1, "为被盖掉的那一组开成的卡关掉了");
    }

    /// 丙（开完换上的那一组一张卡都用不上）。原读数不变：answered_every_card_left_out=true batch_failed=true devices_given_work=[]。
    #[test]
    fn attack_repaired_left_out_after_the_opening() {
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_always_opens,
                None,
                false,
            );
        let to_the_card_at_forty_one_under_a_tiny_limit = switch_naming_the_cards_admitted_against(
            &background_gpu,
            &["0000:41:00.0"],
            Some(vec![64]),
        );
        background_gpu
            .use_the_settings_from_the_next_batch(&to_the_card_at_forty_one_under_a_tiny_limit);
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let (outcome, devices_given_work) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        let answered_every_card_left_out = matches!(
            answered,
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: Some(
                    BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(_)
                )
            }
        );
        println!(
            "name=attack_repaired_left_out_after_the_opening answered_every_card_left_out={answered_every_card_left_out} batch_failed={} devices_given_work={devices_given_work:?}",
            outcome.is_err()
        );
        assert!(answered_every_card_left_out);
        assert!(outcome.is_err());
        assert!(devices_given_work.is_empty());
    }

    /// 乙（交设置那一刻读到切换之前那一组的「用不上」）。原读数不变：StillOpeningItsCards、换上之后 PutInEffect{None}、参与的是「0000:41:00.0」。
    #[test]
    fn attack_repaired_failure_of_the_settings_before_the_switch() {
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_opens_once_released(opening_released),
                Some(64),
                true,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered_at_the_handing_in = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll);
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        let answered_once_opened = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let participating = participating_devices_of(&background_gpu);
        println!(
            "name=attack_repaired_failure_of_the_settings_before_the_switch answered_at_the_handing_in={answered_at_the_handing_in:?} answered_once_opened={answered_once_opened:?} participating={participating:?}"
        );
        assert_eq!(
            answered_at_the_handing_in,
            LatestSettingsHandedIn::StillOpeningItsCards
        );
        assert_eq!(
            answered_once_opened,
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            }
        );
        assert_eq!(participating, vec![card_identifier("0000:41:00.0")]);
    }

    // ── c996-code-r3 新加：修补带出的新状态与交错 ──

    /// 攻方腿模型（c996-code-r3）：切换开卡那一步——「0000:41:00.0」第 `held_opening` 次开卡先送一声、挂着等用例放开（至多 20 秒），
    /// 它的每一次都开不成；别的卡当场开成。
    fn attack_r3_opening_failing_and_holding_the_nth(
        held_opening: usize,
        held_opening_begun: std::sync::mpsc::Sender<()>,
        held_opening_released: std::sync::mpsc::Receiver<()>,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        let mut openings_of_forty_one: usize = 0;
        move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
            if discrete_gpu.pci_bus_address != "0000:41:00.0" {
                return opening_that_always_opens(
                    discrete_gpu,
                    memory_limit,
                    wait_limit,
                    cost_at_the_opening,
                );
            }
            openings_of_forty_one += 1;
            if openings_of_forty_one == held_opening {
                let _signal_sent = held_opening_begun.send(());
                let _released_or_not = held_opening_released.recv_timeout(Duration::from_secs(20));
            }
            Err(reason_the_card_does_not_open_at_the_switch())
        }
    }

    /// 「切到 cpu 再切回」那一段历史的读数（每一样都写成一行字，panic 的原话照收）。
    struct AttackR3ReadingsOfTheSwitchBack {
        answered_right_after_the_switch_back: String,
        poll_panicked_with: Option<String>,
        answered_once_ended: Option<String>,
        openings_of_forty_one: usize,
        later_switch_panicked_with: Option<String>,
    }

    /// 攻方腿模型（c996-code-r3，检查单「走不到的理由」的 `debug_assert!`）：挂载开着「0000:01:00.0」；切换只点没开的「0000:41:00.0」，
    /// 头几次当场开不成、第 `held_opening` 次挂着；挂着的时候用户切到 `cpu`（叫停那条线程，正在开的那一次照旧开完）；用户再切回点 `switch_back_to`——
    /// `switch_back_while_held` 时在挂着的那一次开完之前切回，不然等它开完（那条线程放开开卡那一步的锁）再切回。之后用户再切一次只点「0000:81:00.0」。
    fn attack_r3_switch_back_while_an_opening_is_held(
        held_opening: usize,
        switch_back_to: &[&str],
        switch_back_while_held: bool,
    ) -> AttackR3ReadingsOfTheSwitchBack {
        let (held_opening_begun, every_held_opening_begun) = std::sync::mpsc::channel::<()>();
        let (release_the_held_opening, held_opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                attack_r3_opening_failing_and_holding_the_nth(
                    held_opening,
                    held_opening_begun,
                    held_opening_released,
                ),
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        every_held_opening_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("挂着的那一次开卡 20 秒内起了");
        background_gpu.use_no_gpu_from_the_next_batch();
        let switching_back =
            switch_naming_the_cards_admitted_against(&background_gpu, switch_back_to, None);
        if !switch_back_while_held {
            release_the_held_opening
                .send(())
                .expect("挂着的那一次还在等放开");
            // 切换开卡线程从头一次开卡之前到开完都锁着开卡那一步：拿到锁就是它开完了。
            drop(
                background_gpu
                    .opening_of_a_gpu_at_a_switch
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
        }
        background_gpu.use_the_settings_from_the_next_batch(&switching_back);
        let answered_right_after_the_switch_back = format!(
            "{:?}",
            background_gpu
                .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll)
        );
        if switch_back_while_held {
            release_the_held_opening
                .send(())
                .expect("挂着的那一次还在等放开");
        }
        let poll = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            background_gpu
                .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds)
        }));
        let poll_panicked_with = poll
            .as_ref()
            .err()
            .map(|payload| attack_said_by_the_panic(&**payload));
        let answered_once_ended = poll.ok().map(|answered| format!("{answered:?}"));
        let openings_of_forty_one = cards_handed_to_the_opening
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|address| address.as_str() == "0000:41:00.0")
            .count();
        let to_the_card_at_eighty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
        let later_switch = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
            background_gpu
                .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds)
        }));
        let later_switch_panicked_with = later_switch
            .as_ref()
            .err()
            .map(|payload| attack_said_by_the_panic(&**payload));
        AttackR3ReadingsOfTheSwitchBack {
            answered_right_after_the_switch_back,
            poll_panicked_with,
            answered_once_ended,
            openings_of_forty_one,
            later_switch_panicked_with,
        }
    }

    /// 攻方腿模型（c996-code-r3）：上面那段历史放开用户的两步扫一遍——挂着的是第 3 次还是第 4 次开卡、切回点哪几张卡、在挂着的那一次开完之前还是之后切回。
    /// 每一行一段历史；断言写条款要的结局（不 panic；「0000:41:00.0」一次挂载里连着开不成至多 4 次）。
    #[test]
    fn attack_r3_switching_back_while_the_fourth_opening_is_held_reaches_the_debug_assertion() {
        let mut histories_breaking_the_clause = Vec::new();
        for held_opening in [3_usize, 4] {
            for switch_back_to in [
                vec!["0000:41:00.0"],
                vec!["0000:81:00.0"],
                vec!["0000:81:00.0", "0000:41:00.0"],
            ] {
                for switch_back_while_held in [true, false] {
                    let readings = attack_r3_switch_back_while_an_opening_is_held(
                        held_opening,
                        &switch_back_to,
                        switch_back_while_held,
                    );
                    println!(
                        "name=attack_r3_switch_back held_opening={held_opening} switch_back_to={switch_back_to:?} switch_back_while_held={switch_back_while_held} answered_right_after_the_switch_back={} poll_panicked_with={:?} answered_once_ended={:?} openings_of_forty_one={} later_switch_panicked_with={:?}",
                        readings.answered_right_after_the_switch_back.replace(char::is_whitespace, "_"),
                        readings.poll_panicked_with,
                        readings
                            .answered_once_ended
                            .as_ref()
                            .map(|answered| answered.replace(char::is_whitespace, "_")),
                        readings.openings_of_forty_one,
                        readings.later_switch_panicked_with
                    );
                    if readings.poll_panicked_with.is_some()
                        || readings.later_switch_panicked_with.is_some()
                        || readings.openings_of_forty_one > 4
                    {
                        histories_breaking_the_clause.push((
                            held_opening,
                            switch_back_to.clone(),
                            switch_back_while_held,
                        ));
                    }
                }
            }
        }
        println!(
            "name=attack_r3_switch_back_summary histories=12 breaking_the_clause={} which={histories_breaking_the_clause:?}",
            histories_breaking_the_clause.len()
        );
        assert!(
            histories_breaking_the_clause.is_empty(),
            "这几段历史 panic 或开过 4 次以上：{histories_breaking_the_clause:?}"
        );
    }

    /// 攻方腿模型（c996-code-r3，进程级那一张表）：同一进程里另一次挂载正在开「0000:41:00.0」（没到点，起卡那一件等用例放开），
    /// 这一次挂载（替身外壳，切换开卡那一步用产品那一份、带同一张表）切换只点它，开完之后用户再切一次同一组。
    /// `arm=shared_table` 两次挂载拿进程开卡器那一张表（产品照这样给），`arm=table_of_each_mount` 各拿一张。
    #[test]
    fn attack_r3_an_opening_under_way_in_another_mount_uses_up_the_card_in_this_mount() {
        let mut arms_breaking_the_clause = Vec::new();
        for shared_table in [true, false] {
            let table_of_this_mount = OpeningsNotReturnedInThisProcess::with_none_not_returned();
            let table_of_the_other_mount = if shared_table {
                table_of_this_mount.clone()
            } else {
                OpeningsNotReturnedInThisProcess::with_none_not_returned()
            };
            let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
            let start_released = Arc::new(Mutex::new(start_released));
            let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
            let starts_of_the_other_mount = Arc::new(std::sync::atomic::AtomicU32::new(0));
            let mut opening_of_the_other_mount = attack_r3_opening_held_within_twenty_seconds(
                &start_released,
                &starts_of_the_other_mount,
                &start_begun,
                &table_of_the_other_mount,
            );
            let the_other_mount = std::thread::spawn(move || {
                opening_of_the_other_mount(
                    &discrete_gpu_not_opened_at("0000:41:00.0"),
                    None,
                    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                    CostAtTheOpening::MeasuredUnderTheMemoryLimit,
                )
                .err()
            });
            every_start_begun
                .recv_timeout(Duration::from_secs(20))
                .expect("另一次挂载的开卡 20 秒内起了");
            let (mut background_gpu, cards_handed_to_the_opening) =
                background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                    attack_r3_opening_that_opens_at_once_within_twenty_seconds(&table_of_this_mount),
                    None,
                    false,
                );
            let to_the_card_at_forty_one =
                switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
            background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
            let answered_while_the_other_mount_opens = background_gpu
                .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
            release_the_start
                .send(())
                .expect("另一次挂载的起卡那一件还在等放开");
            let the_other_mount_failed_with = the_other_mount
                .join()
                .expect("另一次挂载那条线程没 panic");
            background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
            let answered_once_the_other_mount_returned = background_gpu
                .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::UntilItEnds);
            let openings_in_this_mount = openings_made_so_far(&cards_handed_to_the_opening);
            println!(
                "name=attack_r3_opening_under_way_in_another_mount arm={} the_other_mount_failed_with={the_other_mount_failed_with:?} answered_while_the_other_mount_opens={} answered_once_the_other_mount_returned={} openings_in_this_mount={openings_in_this_mount}",
                if shared_table { "shared_table" } else { "table_of_each_mount" },
                format!("{answered_while_the_other_mount_opens:?}").replace(char::is_whitespace, "_"),
                format!("{answered_once_the_other_mount_returned:?}").replace(char::is_whitespace, "_")
            );
            if the_other_mount_failed_with.is_some()
                || !matches!(
                    answered_while_the_other_mount_opens,
                    LatestSettingsHandedIn::PutInEffect { .. }
                )
            {
                // 条款要的：另一次挂载的开卡没到点、没挂住，这一次挂载的切换不该算它「前一次开卡没回来」、更不该把这张卡在这一次挂载里试满。
                arms_breaking_the_clause.push(shared_table);
            }
        }
        println!(
            "name=attack_r3_opening_under_way_in_another_mount_summary arms=2 breaking_the_clause={arms_breaking_the_clause:?}"
        );
        assert!(
            arms_breaking_the_clause.is_empty(),
            "shared_table 这几臂里另一次挂载正在开的卡算成了这一次挂载开不成：{arms_breaking_the_clause:?}"
        );
    }

    /// 攻方腿模型（c996-code-r3）：同 [`attack_opening_held_until_released`]，开一次至多等 20 秒（等用例放开的这段时间没到点）。
    fn attack_r3_opening_held_within_twenty_seconds(
        start_released: &Arc<Mutex<std::sync::mpsc::Receiver<()>>>,
        starts_made: &Arc<std::sync::atomic::AtomicU32>,
        start_begun: &std::sync::mpsc::Sender<()>,
        openings_not_returned: &OpeningsNotReturnedInThisProcess,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        let start_released = Arc::clone(start_released);
        let starts_counted = Arc::clone(starts_made);
        let start_begun = start_begun.clone();
        opening_of_a_card_within_the_limit(
            Duration::from_secs(20),
            move |discrete_gpu: &DiscreteGpuOnThisHost,
                  memory_limit,
                  _wait_limit,
                  bytes_leaked_by_earlier_mounts| {
                let card = CardComputedOnTheCpuForTests::of_the_address(
                    &discrete_gpu.pci_bus_address,
                    memory_limit.map(|memory_limit| memory_limit.0),
                    bytes_leaked_by_earlier_mounts.0,
                );
                let start_released_seen_by_the_start = Arc::clone(&start_released);
                let starts_counted_by_the_start = Arc::clone(&starts_counted);
                let start_begun_by_the_start = start_begun.clone();
                move || {
                    starts_counted_by_the_start.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let _signal_sent = start_begun_by_the_start.send(());
                    let _released_or_not = start_released_seen_by_the_start
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .recv_timeout(Duration::from_secs(20));
                    Ok(card)
                }
            },
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            openings_not_returned.clone(),
        )
    }

    /// 攻方腿模型（c996-code-r3）：产品那一份开一张卡那一步（一次至多等 20 秒、还没回来的开卡那一张表是 `openings_not_returned`），起卡那一件当场开成替身卡。
    fn attack_r3_opening_that_opens_at_once_within_twenty_seconds(
        openings_not_returned: &OpeningsNotReturnedInThisProcess,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        opening_of_a_card_within_the_limit(
            Duration::from_secs(20),
            move |discrete_gpu: &DiscreteGpuOnThisHost,
                  memory_limit,
                  _wait_limit,
                  bytes_leaked_by_earlier_mounts| {
                let card = CardComputedOnTheCpuForTests::of_the_address(
                    &discrete_gpu.pci_bus_address,
                    memory_limit.map(|memory_limit| memory_limit.0),
                    bytes_leaked_by_earlier_mounts.0,
                );
                move || Ok(card)
            },
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            openings_not_returned.clone(),
        )
    }

    // ── bggpu-audit-fix-code-r1 攻方腿的甲与暂存口径（替身卡今天已带 `cost_measured_through_the_checksum_chunk`，不再改替身卡）──

    fn attack_model_payload_text(payload: &(dyn std::any::Any + Send)) -> String {
        attack_said_by_the_panic(payload).replace(char::is_whitespace, "_")
    }

    /// 交一次切换（`switch` 改外壳），接住 panic；交回一行结局。
    fn attack_model_switch_outcome(
        background_gpu: &mut BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        switch: impl FnOnce(&mut BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>),
    ) -> String {
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            switch(&mut *background_gpu);
        }));
        match caught {
            Ok(()) => {
                let card_at_one = background_gpu.cards_on_this_host.iter().find(|card| {
                    card.discrete_gpu.identifier == card_identifier("0000:01:00.0")
                });
                format!(
                    "outcome=returned card_at_one_open={} card_at_one_running={} card_at_one_measurements={}",
                    card_at_one.is_some(),
                    card_at_one.is_some_and(|card| card
                        .budget
                        .memory_account()
                        .has_a_submission_running_past_its_wait()),
                    card_at_one.map_or(0, |card| card.cost_measurements),
                )
            }
            Err(payload) => format!(
                "outcome=panicked message={}",
                attack_model_payload_text(payload.as_ref())
            ),
        }
    }

    fn attack_model_two_cards(
        measured_at_the_mount: bool,
    ) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
        let mut card_at_one = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        card_at_one.cost_measured_through_the_checksum_chunk = true;
        if !measured_at_the_mount {
            card_at_one.measured_cost = None;
        }
        let card_at_forty_one =
            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0);
        let (background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card_at_one, card_at_forty_one],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        background_gpu
    }

    /// 历史甲（前缀）：「0000:01:00.0」没量过代价、被晾下；切换请回来，量代价那一次提交等超过上限（记成还在跑）。
    fn attack_model_prefix_measurement_timed_out_at_a_switch(
    ) -> (BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>, String) {
        let mut background_gpu = attack_model_two_cards(false);
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
        let leaving_it_out = settings_admitted_on_the_cards(
            &background_gpu,
            None,
            vec![smallest_piece_bytes - 1, 1 << 30],
        );
        let left_out = attack_model_switch_outcome(&mut background_gpu, |gpu| {
            gpu.use_the_settings_from_the_next_batch(&leaving_it_out);
        });
        background_gpu.cards_on_this_host[0]
            .submissions_timing_out
            .push_back(true);
        let bringing_it_back =
            settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
        let brought_back = attack_model_switch_outcome(&mut background_gpu, |gpu| {
            gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
        });
        let not_split = format!("{:?}", background_gpu.batch_not_split_over_the_gpus())
            .chars()
            .take(40)
            .collect::<String>()
            .replace(char::is_whitespace, "_");
        (
            background_gpu,
            format!(
                "prefix_left_out[{left_out}] prefix_brought_back[{brought_back}] batch_not_split={not_split}"
            ),
        )
    }

    /// 历史乙（前缀）：「0000:01:00.0」量过代价、参与；一批校验和头一次交等超过上限、之后三回等前一次都没等到；切换给它装不下最小一片的上限（晾下）。
    fn attack_model_prefix_batch_timed_out_then_left_out(
    ) -> (BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>, String) {
        let mut background_gpu = attack_model_two_cards(true);
        {
            let card_at_one = &mut background_gpu.cards_on_this_host[0];
            card_at_one.submissions_timing_out = VecDeque::from([true]);
            card_at_one.earlier_submission_waits_timing_out = VecDeque::from([true, true, true]);
        }
        let unit_bytes = unit_bytes_of(1, 1000);
        let batch = background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes));
        let batch_said = match &batch {
            Ok(_outcomes) => "ok".to_string(),
            Err(failed) => format!("{failed:?}")
                .chars()
                .take(60)
                .collect::<String>()
                .replace(char::is_whitespace, "_"),
        };
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
        let leaving_it_out = settings_admitted_on_the_cards(
            &background_gpu,
            None,
            vec![smallest_piece_bytes - 1, 1 << 30],
        );
        let left_out = attack_model_switch_outcome(&mut background_gpu, |gpu| {
            gpu.use_the_settings_from_the_next_batch(&leaving_it_out);
        });
        (
            background_gpu,
            format!("prefix_batch={batch_said} prefix_left_out[{left_out}]"),
        )
    }

    /// 前缀之后由用户定的那一次切换，放开扫的五种。
    fn attack_model_user_switches(
        background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    ) -> Vec<(&'static str, Option<AdmittedGpuSettings>)> {
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
        let held_on_the_card_at_one = background_gpu.cards_on_this_host[0]
            .budget
            .memory_account()
            .bytes_held()
            .0;
        vec![
            (
                "both_named_large_limit",
                Some(settings_admitted_on_the_cards(background_gpu, None, vec![1 << 30, 1 << 30])),
            ),
            (
                "both_named_limit_just_holding_the_smallest_piece",
                Some(settings_admitted_on_the_cards(
                    background_gpu,
                    None,
                    vec![held_on_the_card_at_one + smallest_piece_bytes, 1 << 30],
                )),
            ),
            (
                "both_named_limit_one_byte_short",
                Some(settings_admitted_on_the_cards(
                    background_gpu,
                    None,
                    vec![held_on_the_card_at_one + smallest_piece_bytes - 1, 1 << 30],
                )),
            ),
            (
                "only_the_other_card_named",
                Some(settings_admitted_on_the_cards(
                    background_gpu,
                    Some(vec![card_identifier("0000:41:00.0")]),
                    vec![1 << 30],
                )),
            ),
            ("switched_to_the_cpu", None),
        ]
    }

    /// 一段前缀之后扫五种用户切换，交回 panic 了的行数。
    fn attack_model_sweep(
        history: &str,
        prefix: fn() -> (BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>, String),
    ) -> usize {
        let (probe_gpu, _prefix_said) = prefix();
        let switch_names: Vec<&'static str> = attack_model_user_switches(&probe_gpu)
            .into_iter()
            .map(|(name, _settings)| name)
            .collect();
        drop(probe_gpu);
        let mut panicked = 0;
        for switch_name in switch_names {
            let (mut background_gpu, prefix_said) = prefix();
            let settings = attack_model_user_switches(&background_gpu)
                .into_iter()
                .find(|(name, _settings)| *name == switch_name)
                .expect("同一份名单")
                .1;
            let held_before = background_gpu.cards_on_this_host[0]
                .budget
                .memory_account()
                .bytes_held()
                .0;
            let outcome = attack_model_switch_outcome(&mut background_gpu, |gpu| match &settings {
                Some(settings) => gpu.use_the_settings_from_the_next_batch(settings),
                None => gpu.use_no_gpu_from_the_next_batch(),
            });
            if outcome.starts_with("outcome=panicked") {
                panicked += 1;
            }
            println!(
                "name=attack_model history={history} user_switch={switch_name} bytes_held_on_the_card_at_one_before={held_before} {prefix_said} {outcome}"
            );
        }
        panicked
    }

    /// 甲。原读数（任务 14625）：两段历史各 5 种用户切换里 4 行 `outcome=panicked`（给那张卡装得下的上限的那几种）。条款：都不 panic。
    #[test]
    fn attack_model_measurement_at_a_switch_on_a_card_with_a_submission_still_running() {
        let panicked = attack_model_sweep(
            "measurement_timed_out_at_a_switch",
            attack_model_prefix_measurement_timed_out_at_a_switch,
        ) + attack_model_sweep(
            "batch_timed_out_then_left_out",
            attack_model_prefix_batch_timed_out_then_left_out,
        );
        println!("name=attack_model_sweep_summary rows=10 panicked={panicked}");
        assert_eq!(panicked, 0);
    }

    /// 甲的对照：没有还在跑的提交时，照产品走账的量代价在请回来那一刻照常量成。原读数不变：outcome=returned card_at_one_running=false card_at_one_measurements=1。
    #[test]
    fn attack_model_control_bring_back_without_a_running_submission() {
        let mut background_gpu = attack_model_two_cards(true);
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
        let leaving_it_out = settings_admitted_on_the_cards(
            &background_gpu,
            None,
            vec![smallest_piece_bytes - 1, 1 << 30],
        );
        background_gpu.use_the_settings_from_the_next_batch(&leaving_it_out);
        let bringing_it_back =
            settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
        let outcome = attack_model_switch_outcome(&mut background_gpu, |gpu| {
            gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
        });
        println!(
            "name=attack_model_control history=no_running_submission user_switch=both_named_large_limit {outcome}"
        );
        assert_eq!(
            outcome,
            "outcome=returned card_at_one_open=true card_at_one_running=false card_at_one_measurements=1"
        );
    }

    /// 暂存口径的独立核（G04 的算术，换类型之后照同一个种子、同一批取样重算）：照 `ChunkFootprint::bytes_held_by_a_chunk`（连同暂存的那个数）
    /// 逐个比切批的判与切片，不经 `chunk_fits_with_the_staging_of_its_input` 与 `largest_piece_input_bytes` 的折算。
    /// 原读数（任务 14625）：fit_cases=200000 fit_disagreements=0 cut_cases=200000 chunks_over_the_limit=0 pieces_not_largest=0 cut_refusals_wrong=0。
    #[test]
    fn attack_model_staging_arithmetic_sweep() {
        use crate::memory_limit_and_chunks::{
            chunks_that_fit_on_the_gpu, pieces_cut_to_fit_on_the_gpu, ChunkBoundsOfAGpu,
            ChunkFootprint, EntryCountOfAChunk, InputLengthOnTheHostInBytes, PieceOfAnEntry,
        };
        let mut state: u64 = 0x5EED_0B66_A0D1_7001;
        let mut next = |bound: u64| -> u64 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (state >> 33) % bound
        };
        let (mut fit_cases, mut fit_disagreements) = (0_u64, 0_u64);
        let (mut cut_cases, mut chunks_over_the_limit, mut pieces_not_largest, mut cut_refusals_wrong) =
            (0_u64, 0_u64, 0_u64, 0_u64);
        for _case in 0..200_000 {
            let footprint = ChunkFootprint {
                bytes_per_chunk_besides_the_entries: GpuMemoryInBytes(next(300)),
                bytes_per_entry_besides_its_input: GpuMemoryInBytes(next(120)),
                bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes(1 + next(16)),
            };
            let limit = GpuMemoryInBytes(next(6000));
            let bounds = ChunkBoundsOfAGpu {
                memory_limit: Some(limit),
                largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 40),
                largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 40),
            };
            let input = InputLengthOnTheHostInBytes(next(4000));
            fit_cases += 1;
            let judged = chunks_that_fit_on_the_gpu(&[input], footprint, bounds).is_ok();
            let by_the_staging_arithmetic =
                footprint.bytes_held_by_a_chunk(input, EntryCountOfAChunk(1)) <= limit;
            if judged != by_the_staging_arithmetic {
                fit_disagreements += 1;
            }
            let entry_count = 1 + next(5);
            let lengths: Vec<InputLengthOnTheHostInBytes> = (0..entry_count)
                .map(|_entry| InputLengthOnTheHostInBytes(next(4000)))
                .collect();
            cut_cases += 1;
            match pieces_cut_to_fit_on_the_gpu(&lengths, footprint, bounds) {
                Ok(chunks) => {
                    for chunk in &chunks {
                        let input_of_the_chunk: InputLengthOnTheHostInBytes =
                            chunk.iter().map(PieceOfAnEntry::input_byte_count).sum();
                        let held = footprint.bytes_held_by_a_chunk(
                            input_of_the_chunk,
                            EntryCountOfAChunk::of_this_many_entries(chunk.len()),
                        );
                        if held > limit {
                            chunks_over_the_limit += 1;
                        }
                        for piece in chunk {
                            let whole = lengths[piece.entry_position];
                            let piece_bytes = piece.input_byte_count();
                            let is_cut = piece_bytes < whole;
                            let is_last_piece = piece.input_bytes.end == whole;
                            if is_cut
                                && !is_last_piece
                                && footprint.bytes_held_by_a_chunk(
                                    piece_bytes.plus(InputLengthOnTheHostInBytes(4)),
                                    EntryCountOfAChunk(1),
                                ) <= limit
                            {
                                pieces_not_largest += 1;
                            }
                        }
                    }
                }
                Err(_smallest_piece) => {
                    if footprint.bytes_held_by_a_chunk(
                        InputLengthOnTheHostInBytes(1),
                        EntryCountOfAChunk(1),
                    ) <= limit
                    {
                        cut_refusals_wrong += 1;
                    }
                }
            }
        }
        println!(
            "name=attack_model_staging_arithmetic fit_cases={fit_cases} fit_disagreements={fit_disagreements} cut_cases={cut_cases} chunks_over_the_limit={chunks_over_the_limit} pieces_not_largest={pieces_not_largest} cut_refusals_wrong={cut_refusals_wrong}"
        );
        assert_eq!(
            (fit_disagreements, chunks_over_the_limit, pieces_not_largest, cut_refusals_wrong),
            (0, 0, 0, 0)
        );
    }
