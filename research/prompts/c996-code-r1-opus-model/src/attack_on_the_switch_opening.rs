
    // ── 攻方腿模型（c996-code-r1）：切换开卡的几种交错，接在 background_gpu.rs 的测试模块末尾 ──
    // 每条用例打一行 `name=… 键=值`，断言写的是攻方腿报的那种行为：用例过 = 那一格照报告里说的走。

    /// 攻方腿模型：本机三张独显——「0000:01:00.0」挂载那一刻开着（给它的显存上限 `memory_limit_of_the_card_at_one`；
    /// `left_out_at_the_mount` 时它因显存上限不参与、参与的卡一张都不剩），「0000:41:00.0」「0000:81:00.0」没开。
    /// 切换点到没开的卡时开它那一步交 `opening_at_the_switch`，重开照用例的规矩（间隔 0、至多重开 3 次）。
    fn attack_background_gpu_with_three_cards(
        opening_at_the_switch: impl FnMut(
                &DiscreteGpuOnThisHost,
                Option<GpuMemoryInBytes>,
                Duration,
                CostAtTheOpening,
            ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
            + Send
            + 'static,
        memory_limit_of_the_card_at_one: Option<u64>,
        left_out_at_the_mount: bool,
    ) -> (
        BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        Arc<Mutex<Vec<String>>>,
    ) {
        let card_at_one = CardComputedOnTheCpuForTests::of_the_address(
            "0000:01:00.0",
            memory_limit_of_the_card_at_one,
            0,
        );
        let opened = card_at_one.discrete_gpu.clone();
        let not_opened_card =
            |pci_bus_address: &str, position_in_the_enumeration: usize| DiscreteGpuOnThisHost {
                position_in_the_enumeration,
                identifier: card_identifier(pci_bus_address),
                pci_bus_address: pci_bus_address.to_string(),
                name: format!("cpu card at {pci_bus_address} not opened at the mount"),
                total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
            };
        let (participating, left_out) = if left_out_at_the_mount {
            (
                ParticipatingGpusOnThisHost::of_the_positions(Vec::new()),
                vec![GpuLeftOutUnderItsMemoryLimit {
                    device: card_identifier("0000:01:00.0"),
                    memory_limit: GpuMemoryInBytes(memory_limit_of_the_card_at_one.unwrap_or(0)),
                    smallest_piece_bytes: bytes_of_the_smallest_piece_every_participating_gpu_holds(
                    ),
                    bytes_already_held_on_the_device: 0,
                }],
            )
        } else {
            (
                ParticipatingGpusOnThisHost::of_the_positions(vec![0]),
                Vec::new(),
            )
        };
        let (opening_noting_the_cards, cards_handed_to_the_opening) =
            opening_noting_the_cards_handed_to_it(opening_at_the_switch);
        let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
            vec![card_at_one],
            DiscreteGpusAfterTheOpening {
                opened: vec![opened],
                not_opened_for_not_taking_part: vec![
                    not_opened_card("0000:41:00.0", 1),
                    not_opened_card("0000:81:00.0", 2),
                ],
            },
            Vec::new(),
            participating,
            left_out,
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0"),
                card_identifier("0000:81:00.0"),
            ])))),
            opening_noting_the_cards,
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
        );
        (background_gpu, cards_handed_to_the_opening)
    }

    /// 攻方腿模型：核心层照外壳答的卡清单校验点 `addresses` 那几张（按这个次序）、显存上限 `memory_limits` 的切换。
    fn attack_switch_admitted(
        background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        addresses: &[&str],
        memory_limits: Option<Vec<u64>>,
    ) -> AdmittedGpuSettings {
        admit_gpu_settings_against_the_inventory(
            &BackgroundComputeSettings {
                background_compute: BackgroundComputeBackend::Gpu,
                background_gpu_devices: Some(
                    addresses
                        .iter()
                        .map(|address| card_identifier(address))
                        .collect(),
                ),
                background_gpu_memory_limit: memory_limits
                    .map(|limits| limits.into_iter().map(GpuMemoryInBytes).collect()),
                background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            },
            background_gpu,
        )
        .expect("卡清单列着这几张：切换照收")
    }

    fn attack_openings_so_far(cards_handed_to_the_opening: &Arc<Mutex<Vec<String>>>) -> usize {
        cards_handed_to_the_opening
            .lock()
            .expect("用例没在别处 panic 过")
            .len()
    }

    /// 攻方腿模型（设计问题 ④⑨，陈旧结局臂）：切换只点「0000:41:00.0」，头一次与 3 次重开都开不成；开卡线程开完 4 次之后、
    /// 两批之间那一问还没问到时，同一组设置又交了一次（用户修好卡再切一次；或中间切到 `cpu`，那一段核心层一次都不问）。
    /// 报：又交那一刻开卡那一步被调了几次、两批之间那一问交回的原因个数、问完之后一共开了几次。
    #[test]
    fn attack_the_same_settings_handed_in_again_before_the_poll_get_the_old_failure_without_any_reopening(
    ) {
        let (opened_four_times, every_opening_made) = std::sync::mpsc::channel::<()>();
        let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let openings_counted = Arc::clone(&openings_made);
        let (mut background_gpu, cards_handed_to_the_opening) =
            attack_background_gpu_with_three_cards(
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
            attack_switch_admitted(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        every_opening_made
            .recv_timeout(Duration::from_secs(20))
            .expect("开卡线程 20 秒内开完 4 次");
        let openings_when_handed_in_again = attack_openings_so_far(&cards_handed_to_the_opening);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let reported = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let openings_after_the_poll = attack_openings_so_far(&cards_handed_to_the_opening);
        println!(
            "name=attack_stale_failure arm=handed_in_again_before_the_poll openings_when_handed_in_again={openings_when_handed_in_again} causes_reported={:?} openings_after_the_poll={openings_after_the_poll}",
            reported.as_ref().map(|not_opened| not_opened.causes_of_each_opening.len())
        );
        assert_eq!(openings_when_handed_in_again, 4);
        assert_eq!(
            reported
                .as_ref()
                .map(|not_opened| not_opened.causes_of_each_opening.len()),
            Some(4),
            "又交的那一组拿到的是上一次开卡留下的 4 个原因"
        );
        assert_eq!(openings_after_the_poll, 4, "又交的那一组一次都没重开");
    }

    /// 同一段历史的对照臂：两批之间那一问在又交之前问到了（核心层两次交设置之间有一批或一次入口）——又交的那一组照新开 4 次。
    /// 两臂只差核心层那一问落在又交之前还是之后。
    #[test]
    fn attack_the_same_settings_handed_in_again_after_the_poll_reopen_the_card_four_more_times() {
        let (mut background_gpu, cards_handed_to_the_opening) =
            attack_background_gpu_with_three_cards(
                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            attack_switch_admitted(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let reported_before = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let reported_after = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let openings = attack_openings_so_far(&cards_handed_to_the_opening);
        println!(
            "name=attack_stale_failure arm=handed_in_again_after_the_poll reported_before={} reported_after={} openings={openings}",
            reported_before.is_some(),
            reported_after.is_some()
        );
        assert!(reported_before.is_some() && reported_after.is_some());
        assert_eq!(openings, 8, "问过之后又交：新开 4 次");
    }

    /// 攻方腿模型（设计问题 ④）：切换点「0000:41:00.0」，它在开（等用例放开）；又来一次切换只点「0000:81:00.0」。放开之后「0000:41:00.0」开成，
    /// 接着为最近那一组开「0000:81:00.0」，它头一次与 3 次重开都开不成。报：之后开着的卡、参与的卡、「0000:41:00.0」那张替身卡析构了几次。
    #[test]
    fn attack_the_card_opened_for_a_superseded_switch_stays_open_without_taking_part_when_the_latest_switch_fails(
    ) {
        let (release_the_card_at_forty_one, card_at_forty_one_released) =
            std::sync::mpsc::channel::<()>();
        let drops_of_the_card_at_forty_one = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let drops_noted_by_the_opening = Arc::clone(&drops_of_the_card_at_forty_one);
        let (mut background_gpu, cards_handed_to_the_opening) =
            attack_background_gpu_with_three_cards(
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
            attack_switch_admitted(&background_gpu, &["0000:41:00.0"], None);
        let to_the_card_at_eighty_one =
            attack_switch_admitted(&background_gpu, &["0000:81:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
        release_the_card_at_forty_one
            .send(())
            .expect("开「0000:41:00.0」那一步还在等放开");
        let reported_when_forty_one_opened = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds,
            );
        let reported_when_eighty_one_failed = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds,
            );
        let gpus_opened = background_gpu.gpus_opened();
        let participating = participating_devices_of(&background_gpu);
        let drops = drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst);
        println!(
            "name=attack_superseded_card arm=latest_switch_fails reported_when_forty_one_opened={reported_when_forty_one_opened:?} reported_device_when_eighty_one_failed={:?} gpus_opened={gpus_opened:?} participating={participating:?} drops_of_forty_one={drops} openings={:?}",
            reported_when_eighty_one_failed.as_ref().map(|not_opened| not_opened.device.clone()),
            cards_handed_to_the_opening.lock().expect("用例没在别处 panic 过")
        );
        assert_eq!(reported_when_forty_one_opened, None);
        assert_eq!(
            reported_when_eighty_one_failed.map(|not_opened| not_opened.device),
            Some(card_identifier("0000:81:00.0"))
        );
        assert_eq!(
            gpus_opened,
            vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0")
            ],
            "为被盖掉的那一组开成的卡照旧开着"
        );
        assert_eq!(
            participating,
            vec![card_identifier("0000:01:00.0")],
            "它不参与"
        );
        assert_eq!(drops, 0, "它没关");
    }

    /// 攻方腿模型（第 4 条「挂着切换到参与的卡一张都不剩的设置……报的是挂载那一刻同一个成员」）：切换只点没开的「0000:41:00.0」、
    /// 给它 64 字节的上限（连最小的一片都装不下）。交设置那一刻（核心层这时读 `failure_under_the_settings_handed_in`）、开完之后两批之间那一问、
    /// 那一问之后各读一次「这组设置下一批分不分得到卡上」，再交一批校验和。
    #[test]
    fn attack_the_switch_put_in_effect_by_the_opening_with_every_card_left_out_is_not_handed_back_by_the_poll(
    ) {
        let (mut background_gpu, _cards_handed_to_the_opening) =
            attack_background_gpu_with_three_cards(opening_that_always_opens, None, false);
        let to_the_card_at_forty_one_under_a_tiny_limit =
            attack_switch_admitted(&background_gpu, &["0000:41:00.0"], Some(vec![64]));
        background_gpu
            .use_the_settings_from_the_next_batch(&to_the_card_at_forty_one_under_a_tiny_limit);
        let not_split_when_handed_in = background_gpu.batch_not_split_over_the_gpus();
        let reported_by_the_poll = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let not_split_after_the_poll = background_gpu.batch_not_split_over_the_gpus();
        let (outcome, devices_given_work) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        println!(
            "name=attack_left_out_after_the_opening not_split_when_handed_in={not_split_when_handed_in:?} reported_by_the_poll={reported_by_the_poll:?} not_split_after_the_poll={} batch_failed={} devices_given_work={devices_given_work:?}",
            matches!(
                not_split_after_the_poll,
                Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(_))
            ),
            outcome.is_err()
        );
        assert_eq!(
            not_split_when_handed_in, None,
            "交设置那一刻读的是交来之前那一组"
        );
        assert_eq!(
            reported_by_the_poll, None,
            "开完换上的那一问不交回参与的卡一张都不剩"
        );
        assert!(matches!(
            not_split_after_the_poll,
            Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(_))
        ));
        assert!(outcome.is_err(), "下一批交下去才出错");
    }

    /// 攻方腿模型（设计问题 ⑤⑨ 与核心层停着那一判）：挂载那一刻「0000:01:00.0」因显存上限不参与、参与的卡一张都不剩（核心层挂载就停着）；
    /// 切换只点没开的「0000:41:00.0」（不给上限）。交设置那一刻、开完之后各读一次「这组设置下一批分不分得到卡上」。
    #[test]
    fn attack_the_failure_read_when_the_settings_are_handed_in_is_the_one_of_the_settings_before_the_switch(
    ) {
        let (mut background_gpu, _cards_handed_to_the_opening) =
            attack_background_gpu_with_three_cards(opening_that_always_opens, Some(64), true);
        let to_the_card_at_forty_one =
            attack_switch_admitted(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let not_split_when_handed_in = background_gpu.batch_not_split_over_the_gpus();
        let reported_by_the_poll = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let not_split_after_the_poll = background_gpu.batch_not_split_over_the_gpus();
        println!(
            "name=attack_failure_of_the_settings_before_the_switch not_split_when_handed_in={not_split_when_handed_in:?} reported_by_the_poll={reported_by_the_poll:?} not_split_after_the_poll={not_split_after_the_poll:?} participating={:?}",
            participating_devices_of(&background_gpu)
        );
        assert!(
            matches!(
                &not_split_when_handed_in,
                Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(left_out))
                    if left_out.iter().map(|gpu| gpu.device.clone()).collect::<Vec<_>>()
                        == vec![card_identifier("0000:01:00.0")]
            ),
            "交设置那一刻交回的是切换之前那一组（「0000:01:00.0」）的「一张都不剩」"
        );
        assert_eq!(reported_by_the_poll, None);
        assert_eq!(not_split_after_the_poll, None, "开完换上之后这组设置用得上");
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![card_identifier("0000:41:00.0")]
        );
    }
