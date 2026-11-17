
    // ── 攻方腿模型（c996-code-r2）：修补之后的切换开卡，接在 background_gpu.rs 的测试模块末尾 ──
    // 每条用例打一行 `name=… 键=值`；断言写的是攻方腿报告里说的那种行为：用例过 = 那一格照报告说的走，用例红 = 那一格撤回。

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
        batch_panicked_with: Option<String>,
        devices_given_work: Vec<GpuDeviceIdentifier>,
    }

    /// 攻方腿模型（GPU 包这一半）：挂载开着「0000:01:00.0」，核心层挂载那一刻交来点它的那一组；`switch_to_the_cpu_first` 时先告知一张卡都不参与
    /// （核心层切到 `cpu`）；切换只点没开的「0000:41:00.0」，头一次与 3 次重开都开不成（那一问答开不成，核心层照实现员的读法 ③(a) 把这一组记成生效、停着）；
    /// 用户再交同一组（停着时的切换请求：核心层当场解除停下，下一批开头交给实现、实现答还在开卡），核心层照生效的那一组（选 `gpu`）开一批、
    /// 交实现算校验和那一段。报：开不成之后开着的卡、再交那一刻的答、这一批 panic 没有与 panic 的原话、分到活的卡。
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
        let batch = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            checksum_batch_and_the_devices_given_work(&mut background_gpu).1
        }));
        release_the_opening
            .send(())
            .expect("再交那一次开卡还在等放开：收的那一头还在");
        drop(release_the_opening);
        // 开卡线程从头一次开卡之前到开完都锁着这一步：拿到锁就是它开完了（没起来的那一刻拿到也无妨：发的那一头已丢，它当场开完）。
        drop(
            background_gpu
                .opening_of_a_gpu_at_a_switch
                .lock()
                .expect("开一次卡里的 panic 在锁里就接住了：锁没被毒化"),
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
            batch_panicked_with,
            devices_given_work: batch.unwrap_or_default(),
        }
    }

    /// 攻方腿模型（检查单「走不到的理由」第一处）：先切到 `cpu`，切回 `gpu` 点的卡开不成，再交同一组——`batch_not_split_over_the_gpus` 的 `assert!` 在下一批里走到。
    #[test]
    fn attack_the_retry_after_a_switch_not_opened_following_the_cpu_panics_in_the_next_gpu_batch() {
        let readings = attack_history_of_the_retry_after_the_switch_not_opened(true);
        println!(
            "name=attack_retry_after_not_opened arm=switched_to_the_cpu_first not_opened={} gpus_opened_after_not_opened={:?} still_opening_at_the_retry={} batch_panicked_with={:?} devices_given_work={:?}",
            readings.not_opened_after_the_reopenings,
            readings.gpus_opened_after_not_opened,
            readings.still_opening_at_the_retry,
            readings.batch_panicked_with,
            readings.devices_given_work
        );
        assert!(readings.not_opened_after_the_reopenings);
        assert!(readings.gpus_opened_after_not_opened.is_empty());
        assert!(readings.still_opening_at_the_retry);
        assert!(
            readings
                .batch_panicked_with
                .as_deref()
                .is_some_and(|said| said.contains("参与的卡一张都不剩")),
            "那一批走到 batch_not_split_over_the_gpus 的 assert!"
        );
    }

    /// 同一段历史的对照臂：不先切到 `cpu`——开不成之后切换之前那张「0000:01:00.0」照旧开着、参与，再交同一组之后那一批分给它。
    #[test]
    fn attack_the_retry_after_a_switch_not_opened_runs_the_batch_on_the_card_switched_away_from() {
        let readings = attack_history_of_the_retry_after_the_switch_not_opened(false);
        println!(
            "name=attack_retry_after_not_opened arm=stayed_on_the_gpu not_opened={} gpus_opened_after_not_opened={:?} still_opening_at_the_retry={} batch_panicked_with={:?} devices_given_work={:?}",
            readings.not_opened_after_the_reopenings,
            readings.gpus_opened_after_not_opened,
            readings.still_opening_at_the_retry,
            readings.batch_panicked_with,
            readings.devices_given_work
        );
        assert!(readings.not_opened_after_the_reopenings);
        assert_eq!(
            readings.gpus_opened_after_not_opened,
            vec![card_identifier("0000:01:00.0")],
            "开不成之后切换之前那张卡照旧开着"
        );
        assert!(readings.still_opening_at_the_retry);
        assert_eq!(readings.batch_panicked_with, None);
        assert_eq!(
            readings.devices_given_work,
            vec![card_identifier("0000:01:00.0")],
            "核心层认的生效那一组点的是「0000:41:00.0」，这一批分给的是「0000:01:00.0」"
        );
    }

    /// 攻方腿模型：一张替身独显（没开）。
    fn attack_discrete_gpu_at(pci_bus_address: &str) -> DiscreteGpuOnThisHost {
        DiscreteGpuOnThisHost {
            position_in_the_enumeration: 0,
            identifier: card_identifier(pci_bus_address),
            pci_bus_address: pci_bus_address.to_string(),
            name: format!("attack card at {pci_bus_address}"),
            total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
        }
    }

    /// 攻方腿模型：产品那一份开一张卡那一步（`opening_of_a_card_within_the_limit`，一次至多等 20 秒），起卡那一件头一次 panic、之后开成替身卡；
    /// 起卡那一件每被起一次往 `starts_made` 加一。
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
                    bytes_leaked_by_earlier_mounts,
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
        )
    }

    /// 攻方腿模型（第 4 条「起不了开卡线程、开卡线程 panic 都算这张卡这一次没开成，照重开」用于挂载那一刻）：同一种开卡那一步（起卡那一件头一次 panic），
    /// 交挂载那一刻的开法（`gpus_opened_at_the_mount`）——panic 原样往上传、一次都不重开；交切换开卡线程的开法（每次包 `opening_with_its_panic_caught`、照重开）——
    /// 第二次开成。报：两种开法各 panic 没有、起卡那一件各被起了几次。
    #[test]
    fn attack_an_opening_panicking_at_the_mount_is_not_reopened_while_the_same_at_a_switch_is() {
        let discrete_gpu = attack_discrete_gpu_at("0000:41:00.0");
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
        println!(
            "name=attack_opening_panic arm=mount panicked={} starts={mount_starts}",
            mount.is_err()
        );
        println!(
            "name=attack_opening_panic arm=switch panicked=false opened={} starts={switch_starts}",
            switch.is_ok()
        );
        assert!(mount.is_err(), "挂载那一刻：开卡线程的 panic 原样往上传");
        assert_eq!(mount_starts, 1, "挂载那一刻：一次都没重开");
        assert!(switch.is_ok(), "切换开卡线程：接住、重开、开成");
        assert_eq!(switch_starts, 2);
    }

    /// 攻方腿模型：产品那一份开一张卡那一步（`opening_of_a_card_within_the_limit`，一次至多等 50 毫秒），起卡那一件挂着直到用例放开；
    /// 每被起一次往 `starts_made` 加一、往 `start_begun` 送一声。产品每一次挂载各起一份（`open_background_gpu_with_the_memory_leaked_in_this_process`）。
    fn attack_opening_held_until_released(
        start_released: &Arc<Mutex<std::sync::mpsc::Receiver<()>>>,
        starts_made: &Arc<std::sync::atomic::AtomicU32>,
        start_begun: &std::sync::mpsc::Sender<()>,
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
                    bytes_leaked_by_earlier_mounts,
                );
                let start_released_seen_by_the_start = Arc::clone(&start_released);
                let starts_counted_by_the_start = Arc::clone(&starts_counted);
                let start_begun_by_the_start = start_begun.clone();
                move || {
                    starts_counted_by_the_start.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    // 收的那一头在用例里一直拿着。
                    let _signal_sent = start_begun_by_the_start.send(());
                    let _released_or_not = start_released_seen_by_the_start
                        .lock()
                        .expect("用例没在别处 panic 过")
                        .recv_timeout(Duration::from_secs(20));
                    Ok(card)
                }
            },
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        )
    }

    /// 攻方腿模型（壬还中没有：第 4 条「同一张卡前一次开卡没回来就不再起下一次」）：前一次挂载在「0000:41:00.0」上的一次开卡到点之后还挂着；
    /// 同一次挂载里再开它不起（闸住），下一次挂载（产品另起一份开卡那一步）开它——照起，同一张卡上同时挂着两条。报：各次的结局与起卡那一件被起了几次。
    #[test]
    fn attack_an_opening_hung_in_an_earlier_mount_does_not_keep_a_later_mount_from_starting_another(
    ) {
        let discrete_gpu = attack_discrete_gpu_at("0000:41:00.0");
        let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
        let start_released = Arc::new(Mutex::new(start_released));
        let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
        let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let mut opening_of_the_earlier_mount =
            attack_opening_held_until_released(&start_released, &starts_made, &start_begun);
        let mut opening_of_the_later_mount =
            attack_opening_held_until_released(&start_released, &starts_made, &start_begun);
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
        let starts_within_the_earlier_mount = starts_made.load(std::sync::atomic::Ordering::SeqCst);
        let later_mount_first = opening_of_the_later_mount(
            &discrete_gpu,
            None,
            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
        )
        .err();
        every_start_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("下一次挂载的那一次开卡 20 秒内起了");
        let starts_after_the_later_mount = starts_made.load(std::sync::atomic::Ordering::SeqCst);
        for _hung_start in 0..2 {
            release_the_start
                .send(())
                .expect("起卡那一件还在等放开：收的那一头还在");
        }
        println!(
            "name=attack_hung_opening_across_mounts earlier_mount_first={earlier_mount_first:?} earlier_mount_again={earlier_mount_again:?} starts_within_the_earlier_mount={starts_within_the_earlier_mount} later_mount_first={later_mount_first:?} starts_after_the_later_mount={starts_after_the_later_mount}"
        );
        assert!(matches!(
            earlier_mount_first,
            Some(GpuFailureCause::WaitForTheGpuExceededTheLimit { .. })
        ));
        assert_eq!(
            earlier_mount_again,
            Some(opening_not_started_while_an_earlier_one_has_not_returned()),
            "同一次挂载里闸住"
        );
        assert_eq!(starts_within_the_earlier_mount, 1);
        assert!(matches!(
            later_mount_first,
            Some(GpuFailureCause::WaitForTheGpuExceededTheLimit { .. })
        ));
        assert_eq!(
            starts_after_the_later_mount, 2,
            "下一次挂载在同一张卡上又起了一条：两条同时挂着"
        );
    }

    // ── 第一轮攻方腿五条 GPU 包用例的历史，在修补之后的代码上该读出什么（断言写条款要的结局）──

    /// 丁（同一组设置在那一问之前又交一次）：切换只点「0000:41:00.0」，头一次与 3 次重开都开不成；开卡线程开完 4 次之后、那一问之前又交同一组。
    /// 条款：又交的那一次照它重开，报的是它自己的 4 个原因。
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
                        // 收的那一头还在：用例在等这一声。
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
            .expect("开卡线程 20 秒内开完 4 次");
        let openings_when_handed_in_again = openings_made_so_far(&cards_handed_to_the_opening);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let openings_after_the_poll = openings_made_so_far(&cards_handed_to_the_opening);
        let causes_reported = if let LatestSettingsHandedIn::NotOpened(not_opened) = &answered {
            Some(not_opened.causes_of_each_opening.len())
        } else {
            None
        };
        println!(
            "name=attack_repaired_stale_failure arm=handed_in_again_before_the_poll openings_when_handed_in_again={openings_when_handed_in_again} causes_reported={causes_reported:?} openings_after_the_poll={openings_after_the_poll}"
        );
        assert_eq!(openings_when_handed_in_again, 4);
        assert_eq!(causes_reported, Some(4));
        assert_eq!(
            openings_after_the_poll, 8,
            "又交的那一次照它重开：一共开 8 次"
        );
    }

    /// 丁的对照臂：那一问在又交之前问到了——又交的那一组同样新开 4 次。
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
        assert_eq!(openings, 8);
    }

    /// 戊 ②（为被盖掉的切换开成的卡）：切换点「0000:41:00.0」，它在开（等用例放开）；又来一次切换只点「0000:81:00.0」。放开之后「0000:41:00.0」开成，
    /// 接着为最近那一组开「0000:81:00.0」，头一次与 3 次重开都开不成。条款：「0000:41:00.0」不参与，关掉。
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
            cards_handed_to_the_opening.lock().expect("用例没在别处 panic 过")
        );
        assert_eq!(device_not_opened, Some(card_identifier("0000:81:00.0")));
        assert_eq!(gpus_opened, vec![card_identifier("0000:01:00.0")]);
        assert_eq!(participating, vec![card_identifier("0000:01:00.0")]);
        assert_eq!(drops, 1, "为被盖掉的那一组开成的卡关掉了");
    }

    /// 丙（开完换上的那一组一张卡都用不上）：切换只点没开的「0000:41:00.0」、给它 64 字节的上限。条款：换上的那一刻就交回「一张都不剩」，
    /// 核心层照挂载那一刻的成员停（核心层那一半见 background_compute.rs 的攻方腿用例）。
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
        assert!(answered_every_card_left_out, "那一问换上时就交回一张都不剩");
        assert!(outcome.is_err());
        assert!(devices_given_work.is_empty());
    }

    /// 乙（交设置那一刻读到切换之前那一组的「用不上」）：挂载那一刻「0000:01:00.0」因显存上限不参与、一张都不剩；切换只点没开的「0000:41:00.0」。
    /// 条款：交设置那一刻（实现还在开卡）不答切换之前那一组的「用不上」；开完换上之后答用得上、参与的是「0000:41:00.0」。
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
