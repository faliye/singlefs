
    // ── 攻方腿模型（c996-code-r3）：接在 background_compute.rs 的测试模块末尾，只在冻结副本的拷贝里，不进仓 ──
    // c996-code-r2 攻方腿核心层那九条在今天的代码上重跑：替身与历史照原样（替身不按自己的规则答，两批之间那一问答什么由用例逐批摆），
    // name= 行的键照原样，断言换成条款现行文本要的结局；改了断言的那几处在注释里写原读数（任务 14649）。名字带 attack_r3_ 的是这一轮新加的。


    /// 攻方腿模型的替身状态：两批之间那一问答 `answer`；交设置、告知一张卡都不参与各记一次。
    struct AttackScriptOfTheImplementation {
        answer: LatestGpuSettingsHandedIn,
        settings_handed_in: u32,
        times_told_no_gpu_takes_part: u32,
    }

    /// 攻方腿模型的替身：卡清单同 [`GpuComputeOpeningCardsWhenTheTestSays`]（「0000:01:00.0」「0000:41:00.0」），三段交回空。
    struct AttackImplementationAnsweringByTheScript {
        script: Arc<Mutex<AttackScriptOfTheImplementation>>,
    }

    impl GpuInventory for AttackImplementationAnsweringByTheScript {
        fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
            vec![
                (
                    card_for_the_switch_tests(b"0000:01:00.0"),
                    GpuMemoryInBytes(8 << 30),
                ),
                (
                    card_for_the_switch_tests(b"0000:41:00.0"),
                    GpuMemoryInBytes(8 << 30),
                ),
            ]
        }
    }

    impl BackgroundGpuCompute for AttackImplementationAnsweringByTheScript {
        fn verify_unit_checksums(
            &mut self,
            _units: &[UnitChecksumToVerifyOnTheGpu<'_>],
        ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>
        {
            Ok(Vec::new())
        }

        fn classify_region_records(
            &mut self,
            _region: CompactionRegion,
            _records: &[RegionRecordToClassifyOnTheGpu<'_>],
            _mapping_entries: &[CentralMappingEntry],
        ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
            Ok(Vec::new())
        }

        fn count_live_slots_per_cluster_segment(
            &mut self,
            _allocation_records: &[AllocationRecord],
            _unit_area: UnitAreaOfOneDevice,
            _segments: &[ClusterSegmentEntryKey],
        ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
            Ok(Vec::new())
        }

        fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {
            self.script
                .lock()
                .expect("用例没在别处 panic 过")
                .settings_handed_in += 1;
        }

        fn use_no_gpu_from_the_next_batch(&mut self) {
            self.script
                .lock()
                .expect("用例没在别处 panic 过")
                .times_told_no_gpu_takes_part += 1;
        }

        fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
            &mut self,
        ) -> LatestGpuSettingsHandedIn {
            self.script
                .lock()
                .expect("用例没在别处 panic 过")
                .answer
                .clone()
        }
    }

    fn attack_script_answering(
        answer: LatestGpuSettingsHandedIn,
    ) -> Arc<Mutex<AttackScriptOfTheImplementation>> {
        Arc::new(Mutex::new(AttackScriptOfTheImplementation {
            answer,
            settings_handed_in: 0,
            times_told_no_gpu_takes_part: 0,
        }))
    }

    fn attack_answer_from_now_on(
        script: &Arc<Mutex<AttackScriptOfTheImplementation>>,
        answer: LatestGpuSettingsHandedIn,
    ) {
        script.lock().expect("用例没在别处 panic 过").answer = answer;
    }

    fn attack_put_in_effect(
        failure_under_the_settings: Option<BackgroundGpuComputeFailed>,
    ) -> LatestGpuSettingsHandedIn {
        LatestGpuSettingsHandedIn::PutInEffect {
            failure_under_the_settings,
        }
    }

    /// 选 `gpu`、点「0000:01:00.0」的一次挂载，替身照 `script` 答。
    fn attack_mount_on_the_card_at_one(
        script: &Arc<Mutex<AttackScriptOfTheImplementation>>,
    ) -> BackgroundComputeOfThisMount {
        let handle = BackgroundGpuComputeHandle::of_an_implementation(
            AttackImplementationAnsweringByTheScript {
                script: Arc::clone(script),
            },
        );
        BackgroundComputeOfThisMount::of_a_mount(
            admit_background_compute_settings_with_the_gpu_compute(
                gpu_settings_naming_the_card(b"0000:01:00.0", None),
                Some(handle),
            )
            .expect("卡清单列着「0000:01:00.0」"),
        )
    }

    fn attack_not_opened_on_the_card_at_forty_one() -> BackgroundGpuComputeFailed {
        BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
            device: card_for_the_switch_tests(b"0000:41:00.0"),
            failures_of_each_opening: Vec::new(),
        }
    }

    /// 池子最满的那一块盘 95%，过了默认 85% 的补跑阈值：停着就在 CPU 上补跑。
    fn attack_fullest_device_above_the_threshold() -> AllocatedBytesAndCapacityOfADevice {
        AllocatedBytesAndCapacityOfADevice {
            device: DeviceIdentity(0),
            allocated_bytes: 95,
            capacity_bytes: 100,
        }
    }

    fn attack_begin_a_batch(
        background_compute: &mut BackgroundComputeOfThisMount,
        fullest_device: AllocatedBytesAndCapacityOfADevice,
    ) -> Option<BackgroundComputeRoute> {
        route_of_the_batch(
            &background_compute.begin_a_batch(BackgroundTask::CompactionRelocation, fullest_device),
        )
    }

    /// 新一、新二（核心层这一半）：挂载点「0000:01:00.0」；`switch_to_the_cpu_first` 时先切到 `cpu`；请求切到没开的「0000:41:00.0」，实现答还在开卡、之后答开不成；
    /// 用户再请求同一组，实现照 `answer_at_the_retry` 答（c996-code-r2 那一份摆的是还在开卡；今天的外壳对试满的卡当场答开不成，两种都摆）。
    fn attack_routes_of_the_retry_after_not_opened(
        switch_to_the_cpu_first: bool,
        answer_at_the_retry: LatestGpuSettingsHandedIn,
    ) -> (
        Option<BackgroundComputeRoute>,
        Option<Vec<GpuDeviceIdentifier>>,
        Option<BackgroundComputeRoute>,
        bool,
        bool,
    ) {
        let script = attack_script_answering(attack_put_in_effect(None));
        let mut background_compute = attack_mount_on_the_card_at_one(&script);
        if switch_to_the_cpu_first {
            background_compute
                .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
                .expect("选 cpu 照收");
            let _route_on_the_cpu = attack_begin_a_batch(
                &mut background_compute,
                fullest_device_far_below_the_threshold(),
            );
        }
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        attack_answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let route_while_opening = attack_begin_a_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        attack_answer_from_now_on(
            &script,
            LatestGpuSettingsHandedIn::NotOpened {
                failure: attack_not_opened_on_the_card_at_forty_one(),
            },
        );
        let route_once_not_opened = attack_begin_a_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let devices_in_effect_once_not_opened = background_compute
            .in_effect()
            .settings()
            .background_gpu_devices
            .clone();
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        let retry_answered_still_opening =
            answer_at_the_retry == LatestGpuSettingsHandedIn::StillOpeningTheirCards;
        attack_answer_from_now_on(&script, answer_at_the_retry);
        let route_at_the_retry = attack_begin_a_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let stopped_at_the_retry = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        let handed_in_at_the_retry = background_compute
            .handed_in_while_the_implementation_opens_its_cards()
            .is_some();
        let times_told_no_gpu_takes_part = script
            .lock()
            .expect("用例没在别处 panic 过")
            .times_told_no_gpu_takes_part;
        println!(
            "name=attack_core_retry_after_not_opened switched_to_the_cpu_first={switch_to_the_cpu_first} retry_answered_still_opening={retry_answered_still_opening} times_told_no_gpu_takes_part={times_told_no_gpu_takes_part} route_while_opening={route_while_opening:?} route_once_not_opened={route_once_not_opened:?} devices_in_effect_once_not_opened={devices_in_effect_once_not_opened:?} route_at_the_retry={route_at_the_retry:?} stopped_at_the_retry={stopped_at_the_retry} handed_in_at_the_retry={handed_in_at_the_retry}"
        );
        assert_eq!(route_once_not_opened, None, "开不成：停着、不开批");
        (
            route_while_opening,
            devices_in_effect_once_not_opened,
            route_at_the_retry,
            stopped_at_the_retry,
            handed_in_at_the_retry,
        )
    }

    fn attack_not_opened_with_the_openings_used_up_on_the_card_at_forty_one() -> LatestGpuSettingsHandedIn {
        LatestGpuSettingsHandedIn::NotOpened {
            failure: BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
                device: card_for_the_switch_tests(b"0000:41:00.0"),
                failures_of_each_opening: vec![BackgroundGpuComputeFailed::GpuFailed {
                    device: card_for_the_switch_tests(b"0000:41:00.0"),
                    cause: GpuFailureOnTheDevice::OpeningsInThisMountUsedUp { failed_openings: 4 },
                }],
            },
        }
    }

    /// 先切到 `cpu` 的那一臂。原读数（任务 14649）：devices_in_effect_once_not_opened=Some([「0000:41:00.0」]) route_at_the_retry=Some(Gpu)。
    #[test]
    fn attack_core_retry_after_not_opened_following_the_cpu_routes_the_batch_to_the_gpu() {
        let cpu = Some(BackgroundComputeRoute::Cpu(
            ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings,
        ));
        let (while_opening, devices, at_the_retry, stopped, handed_in) =
            attack_routes_of_the_retry_after_not_opened(
                true,
                LatestGpuSettingsHandedIn::StillOpeningTheirCards,
            );
        assert_eq!(while_opening, cpu);
        assert_eq!(devices, None, "开不成的那一组不生效：生效的仍是 cpu");
        assert_eq!(at_the_retry, cpu, "再请求之后那一批照生效的 cpu 算，不交给一张卡都没开的实现");
        assert!(!stopped && handed_in);
        let (_while_opening, _devices, at_the_retry, stopped, handed_in) =
            attack_routes_of_the_retry_after_not_opened(
                true,
                attack_not_opened_with_the_openings_used_up_on_the_card_at_forty_one(),
            );
        assert_eq!(at_the_retry, None, "试满：当场开不成、停着");
        assert!(stopped && !handed_in);
    }

    /// 不先切到 `cpu` 的那一臂。原读数（任务 14649）：devices_in_effect_once_not_opened=Some([「0000:41:00.0」])（实现参与的是「0000:01:00.0」）。
    #[test]
    fn attack_core_retry_after_not_opened_on_the_gpu_routes_the_batch_to_the_gpu() {
        let (while_opening, devices, at_the_retry, stopped, handed_in) =
            attack_routes_of_the_retry_after_not_opened(
                false,
                LatestGpuSettingsHandedIn::StillOpeningTheirCards,
            );
        assert_eq!(while_opening, Some(BackgroundComputeRoute::Gpu));
        assert_eq!(
            devices,
            Some(vec![card_for_the_switch_tests(b"0000:01:00.0")]),
            "核心层认的生效那一组与实现照旧参与的是同一张"
        );
        assert_eq!(at_the_retry, Some(BackgroundComputeRoute::Gpu));
        assert!(!stopped && handed_in);
        let (_while_opening, _devices, at_the_retry, stopped, handed_in) =
            attack_routes_of_the_retry_after_not_opened(
                false,
                attack_not_opened_with_the_openings_used_up_on_the_card_at_forty_one(),
            );
        assert_eq!(at_the_retry, None);
        assert!(stopped && !handed_in);
    }

    /// 新四。原读数（任务 14649）：routes=[Some(Gpu), Some(Gpu), Some(Gpu)] after_each_error=[StopLifted…×3] stopped=false。
    /// 条款（第 4 条现行文本）：停下、每次叫起报警、照阈值在 CPU 上补跑；实现答换上了那一刻解除。
    #[test]
    fn attack_batches_failing_while_the_switched_card_opens_never_stop_nor_compensate() {
        let script = attack_script_answering(attack_put_in_effect(None));
        let mut background_compute = attack_mount_on_the_card_at_one(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        attack_answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let mut routes = Vec::new();
        let mut after_each_error = Vec::new();
        let mut alarms = Vec::new();
        for _batch_while_opening in 0..3 {
            background_compute.begin_a_wake();
            let route = attack_begin_a_batch(
                &mut background_compute,
                attack_fullest_device_above_the_threshold(),
            );
            if route == Some(BackgroundComputeRoute::Gpu) {
                after_each_error.push(background_compute.stop_the_task_after_a_compute_error(
                    BackgroundTask::CompactionRelocation,
                    failure_on_the_card_at_one(),
                ));
            }
            routes.push(route);
            alarms.push(
                background_compute
                    .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
                    .map(|alarm| alarm.wakes_while_stopped),
            );
        }
        let stopped = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        attack_answer_from_now_on(&script, attack_put_in_effect(None));
        let route_once_put_in_effect = attack_begin_a_batch(
            &mut background_compute,
            attack_fullest_device_above_the_threshold(),
        );
        let stopped_once_put_in_effect = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        println!(
            "name=attack_errors_while_opening arm=switch_opening routes={routes:?} after_each_error={after_each_error:?} alarms={alarms:?} stopped={stopped} route_once_put_in_effect={route_once_put_in_effect:?} stopped_once_put_in_effect={stopped_once_put_in_effect}"
        );
        assert_eq!(routes[0], Some(BackgroundComputeRoute::Gpu));
        assert!(routes[1..].iter().all(|route| matches!(
            route,
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
            ))
        )));
        assert_eq!(
            after_each_error,
            vec![BackgroundTaskAfterTheComputeError::LeftStoppedWhileTheSwitchOpensItsCards]
        );
        assert_eq!(alarms, vec![Some(0), Some(1), Some(2)], "停下之后每次叫起报警");
        assert!(stopped);
        assert_eq!(route_once_put_in_effect, Some(BackgroundComputeRoute::Gpu));
        assert!(!stopped_once_put_in_effect);
    }

    /// 新四的对照臂：同一个错、同样过了补跑阈值，没有切换。原读数不变：after_the_error=LeftStopped compensating=true。
    #[test]
    fn attack_the_same_error_without_a_switch_stops_and_compensates_on_the_cpu() {
        let script = attack_script_answering(attack_put_in_effect(None));
        let mut background_compute = attack_mount_on_the_card_at_one(&script);
        let route_of_the_failing_batch = attack_begin_a_batch(
            &mut background_compute,
            attack_fullest_device_above_the_threshold(),
        );
        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            failure_on_the_card_at_one(),
        );
        let route_of_the_next_batch = attack_begin_a_batch(
            &mut background_compute,
            attack_fullest_device_above_the_threshold(),
        );
        let compensating = matches!(
            route_of_the_next_batch,
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
            ))
        );
        println!(
            "name=attack_errors_while_opening arm=no_switch route_of_the_failing_batch={route_of_the_failing_batch:?} after_the_error={after_the_error:?} compensating={compensating}"
        );
        assert_eq!(after_the_error, BackgroundTaskAfterTheComputeError::LeftStopped);
        assert!(compensating);
    }

    /// 甲（c996-code-r1 那一条）：交给实现那一批（实现还在开卡）在「0000:01:00.0」上出错。原读数（任务 14649）：after_the_error=StopLiftedByTheSwitchRequestedBeforeTheError。
    /// 条款现行文本：交给了实现、还在开卡的那一组不算「请求了、还没交给实现」的那一种，停着，换上了那一刻解除。
    #[test]
    fn attack_repaired_stop_on_the_card_before_the_switch() {
        let script = attack_script_answering(attack_put_in_effect(None));
        let mut background_compute = attack_mount_on_the_card_at_one(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        attack_answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let route_while_opening = attack_begin_a_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            failure_on_the_card_at_one(),
        );
        attack_answer_from_now_on(&script, attack_put_in_effect(None));
        let route_once_opened = attack_begin_a_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let stopped = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        let devices_in_effect = background_compute
            .in_effect()
            .settings()
            .background_gpu_devices
            .clone();
        println!(
            "name=attack_repaired_stop_outlives_the_switch arm=error_after_the_handing_in route_while_opening={route_while_opening:?} after_the_error={after_the_error:?} route_once_opened={route_once_opened:?} stopped={stopped} devices_in_effect={devices_in_effect:?}"
        );
        assert_eq!(
            after_the_error,
            BackgroundTaskAfterTheComputeError::LeftStoppedWhileTheSwitchOpensItsCards
        );
        assert_eq!(route_once_opened, Some(BackgroundComputeRoute::Gpu));
        assert!(!stopped);
        assert_eq!(
            devices_in_effect,
            Some(vec![card_for_the_switch_tests(b"0000:41:00.0")])
        );
    }
    /// 甲的对照臂：同一个错落在交给实现之前（请求还挂着）——照那一次请求当场解除。
    #[test]
    fn attack_repaired_stop_before_the_handing_in() {
        let script = attack_script_answering(attack_put_in_effect(None));
        let mut background_compute = attack_mount_on_the_card_at_one(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            failure_on_the_card_at_one(),
        );
        let stopped = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        println!(
            "name=attack_repaired_stop_outlives_the_switch arm=error_before_the_handing_in after_the_error={after_the_error:?} stopped={stopped}"
        );
        assert_eq!(
            after_the_error,
            BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError
        );
        assert!(!stopped);
    }

    /// 乙：挂载那一组一张卡都用不上（挂载就停着）；请求切到没开的「0000:41:00.0」。条款：请求那一刻解除；交给实现那一批（实现还在开卡）不再停；
    /// 卡开成、换上之后用得上，照常开批、不停。
    #[test]
    fn attack_repaired_stale_stop() {
        let script = attack_script_answering(attack_put_in_effect(Some(
            every_gpu_left_out_on_the_card(b"0000:01:00.0"),
        )));
        let mut background_compute = attack_mount_on_the_card_at_one(&script);
        let stopped_at_the_mount = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        let stopped_after_the_request = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        attack_answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let route_at_the_handing_in = attack_begin_a_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        attack_answer_from_now_on(&script, attack_put_in_effect(None));
        let route_once_opened = attack_begin_a_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let stopped = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        println!(
            "name=attack_repaired_stale_stop stopped_at_the_mount={stopped_at_the_mount} stopped_after_the_request={stopped_after_the_request} route_at_the_handing_in={route_at_the_handing_in:?} route_once_opened={route_once_opened:?} stopped={stopped}"
        );
        assert!(stopped_at_the_mount);
        assert!(!stopped_after_the_request);
        assert_eq!(route_once_opened, Some(BackgroundComputeRoute::Gpu));
        assert!(!stopped);
    }

    /// 丙：请求切到没开的「0000:41:00.0」、给它装不下最小的一片的上限。条款：卡开成、实现答换上了而一张卡都用不上的那一批开头，
    /// 按挂载那一刻同一个成员停着、不开批。
    #[test]
    fn attack_repaired_left_out_after_the_opening_card_not_open_at_the_switch() {
        let script = attack_script_answering(attack_put_in_effect(None));
        let mut background_compute = attack_mount_on_the_card_at_one(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", Some(64)),
                None,
            )
            .expect("上限不超过显存总量：照收");
        attack_answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let route_while_opening = attack_begin_a_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        attack_answer_from_now_on(
            &script,
            attack_put_in_effect(Some(every_gpu_left_out_on_the_card(b"0000:41:00.0"))),
        );
        let route_once_opened = attack_begin_a_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let stopped_by_the_member = background_compute
            .stopped_after_a_compute_error()
            .iter()
            .any(|stopped| {
                matches!(
                    stopped.error,
                    BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { .. }
                )
            });
        println!(
            "name=attack_repaired_left_out_after_the_opening arm=card_not_open_at_the_switch route_while_opening={route_while_opening:?} route_once_opened={route_once_opened:?} stopped_by_gpu_unusable_under_the_settings_handed_in={stopped_by_the_member}"
        );
        assert_eq!(
            route_once_opened, None,
            "换上的那一组一张卡都用不上：不开批"
        );
        assert!(stopped_by_the_member);
    }

    /// 丙的对照臂：同样的上限点开着的「0000:01:00.0」——交给实现那一刻就换上、当场按同一个成员停着、不开批。
    #[test]
    fn attack_repaired_left_out_after_the_opening_card_already_open() {
        let script = attack_script_answering(attack_put_in_effect(None));
        let mut background_compute = attack_mount_on_the_card_at_one(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:01:00.0", Some(64)),
                None,
            )
            .expect("上限不超过显存总量：照收");
        attack_answer_from_now_on(
            &script,
            attack_put_in_effect(Some(every_gpu_left_out_on_the_card(b"0000:01:00.0"))),
        );
        let route_at_the_handing_in = attack_begin_a_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let stopped_by_the_member = background_compute
            .stopped_after_a_compute_error()
            .iter()
            .any(|stopped| {
                matches!(
                    stopped.error,
                    BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { .. }
                )
            });
        println!(
            "name=attack_repaired_left_out_after_the_opening arm=card_already_open route_at_the_handing_in={route_at_the_handing_in:?} stopped_by_gpu_unusable_under_the_settings_handed_in={stopped_by_the_member}"
        );
        assert_eq!(route_at_the_handing_in, None);
        assert!(stopped_by_the_member);
    }

    // ── c996-code-r3 新加：核心层这一半修补带出的交错 ──

    /// 一次叫起里一批出错（`error`），池子最满的那一块盘过了补跑阈值；交回这一次叫起之后下一批走哪条路、这一次叫起结束时的报警（停下以来叫起几次）。
    fn attack_r3_wake_with_a_failing_batch(
        background_compute: &mut BackgroundComputeOfThisMount,
        error: BackgroundComputeError,
    ) -> (Option<BackgroundComputeRoute>, Option<BackgroundTaskAfterTheComputeError>, Option<u64>) {
        background_compute.begin_a_wake();
        let route = attack_begin_a_batch(background_compute, attack_fullest_device_above_the_threshold());
        let after_the_error = (route == Some(BackgroundComputeRoute::Gpu)).then(|| {
            background_compute
                .stop_the_task_after_a_compute_error(BackgroundTask::CompactionRelocation, error)
        });
        let alarm = background_compute
            .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
            .map(|alarm| u64::from(alarm.wakes_while_stopped));
        (route, after_the_error, alarm)
    }

    /// 攻方腿模型（c996-code-r3）：选段打分那一段每一批都交回「同一个条目键出现两次」（调用方组批的 bug 在同一份盘上每一批都一样），池子过了补跑阈值；
    /// 对照臂是同一处每一批都 GPU 出错。报：每一批走哪条路、每次叫起报不报警。条款字面：重复键只放弃这一批、不停、不报警，所以 GC 一批都做不成而既不报警也不补跑。
    #[test]
    fn attack_r3_a_repeated_entry_key_in_every_batch_neither_stops_nor_compensates_nor_alarms() {
        for arm in ["repeated_entry_key", "gpu_failed"] {
            let script = attack_script_answering(attack_put_in_effect(None));
            let mut background_compute = attack_mount_on_the_card_at_one(&script);
            let mut readings = Vec::new();
            for _wake in 0..4 {
                let error = if arm == "repeated_entry_key" {
                    BackgroundComputeError::GpuSegmentFailed {
                        segment: GpuComputedSegment::LiveSlotsPerClusterSegment,
                        failure: BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
                            position_of_the_first_occurrence: 0,
                            position_of_the_second_occurrence: 3,
                        },
                    }
                } else {
                    failure_on_the_card_at_one()
                };
                readings.push(attack_r3_wake_with_a_failing_batch(&mut background_compute, error));
            }
            println!("name=attack_r3_repeated_entry_key_every_batch arm={arm} wakes={readings:?}");
            if arm == "repeated_entry_key" {
                assert!(readings.iter().all(|(route, after, alarm)| *route
                    == Some(BackgroundComputeRoute::Gpu)
                    && *after
                        == Some(BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned)
                    && alarm.is_none()));
            }
        }
    }

    /// 攻方腿模型（c996-code-r3）：交给了实现、实现还在开卡的期间一批出错停下（新四的改法），之后几次叫起都在 CPU 上补跑、报警；再之后实现答开不成。
    /// 报：答开不成之前与之后各一次叫起的报警（停下以来叫起几次）、补跑了几批、报的是哪个错。任务从头到尾没解除过。
    #[test]
    fn attack_r3_stopped_while_the_switch_opens_then_not_opened_restarts_the_count_of_the_alarm() {
        let script = attack_script_answering(attack_put_in_effect(None));
        let mut background_compute = attack_mount_on_the_card_at_one(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        attack_answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let mut wakes = Vec::new();
        for _wake in 0..3 {
            wakes.push(attack_r3_wake_with_a_failing_batch(
                &mut background_compute,
                failure_on_the_card_at_one(),
            ));
        }
        let compensated_before = background_compute.stopped_after_a_compute_error()[0]
            .batches_compensated_on_the_cpu;
        attack_answer_from_now_on(
            &script,
            LatestGpuSettingsHandedIn::NotOpened {
                failure: attack_not_opened_on_the_card_at_forty_one(),
            },
        );
        let wake_once_not_opened =
            attack_r3_wake_with_a_failing_batch(&mut background_compute, failure_on_the_card_at_one());
        let stopped = background_compute.stopped_after_a_compute_error()[0].clone();
        println!(
            "name=attack_r3_stopped_while_opening_then_not_opened wakes_before={wakes:?} compensated_before={compensated_before} wake_once_not_opened={wake_once_not_opened:?} compensated_after={} error_after={:?}",
            stopped.batches_compensated_on_the_cpu,
            format!("{:?}", stopped.error).chars().take(60).collect::<String>()
        );
        assert!(background_compute.is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
    }

    /// 攻方腿模型（c996-code-r3）：挂载点「0000:01:00.0」；请求切到「0000:01:00.0」与没开的「0000:41:00.0」两张（新的那一组仍点着出错的那张）；
    /// 交给实现、还在开卡的期间那一批在「0000:01:00.0」上出错停下；实现答换上了。报：换上之后停没停、下一批走哪条路、生效的那一组点哪几张。
    #[test]
    fn attack_r3_the_switch_still_naming_the_failing_card_lifts_the_stop_once_put_in_effect() {
        let script = attack_script_answering(attack_put_in_effect(None));
        let mut background_compute = attack_mount_on_the_card_at_one(&script);
        background_compute
            .request_switch_from_the_next_batch(
                BackgroundComputeSettings {
                    background_compute: BackgroundComputeBackend::Gpu,
                    background_gpu_devices: Some(vec![
                        card_for_the_switch_tests(b"0000:01:00.0"),
                        card_for_the_switch_tests(b"0000:41:00.0"),
                    ]),
                    background_gpu_memory_limit: None,
                    background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                },
                None,
            )
            .expect("卡清单列着这两张");
        attack_answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let failing_wake =
            attack_r3_wake_with_a_failing_batch(&mut background_compute, failure_on_the_card_at_one());
        attack_answer_from_now_on(&script, attack_put_in_effect(None));
        let route_once_put_in_effect = attack_begin_a_batch(
            &mut background_compute,
            attack_fullest_device_above_the_threshold(),
        );
        let stopped = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        let devices_in_effect = background_compute
            .in_effect()
            .settings()
            .background_gpu_devices
            .clone();
        println!(
            "name=attack_r3_switch_still_naming_the_failing_card failing_wake={failing_wake:?} route_once_put_in_effect={route_once_put_in_effect:?} stopped={stopped} devices_in_effect={devices_in_effect:?}"
        );
        assert!(!stopped, "条款字面：实现答换上了那一刻解除（不问新的那一组点没点出错的那张）");
    }
