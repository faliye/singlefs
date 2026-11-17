
    // ── 攻方腿模型（c996-code-r2）：核心层照实现的三种答法走，接在 background_compute.rs 的测试模块末尾 ──
    // 每条用例打一行 `name=… 键=值`；断言写的是攻方腿报告里说的那种行为：用例过 = 那一格照报告说的走，用例红 = 那一格撤回。
    // 替身不按自己的规则答：两批之间那一问答什么由用例逐批摆，摆的是产品外壳在同一段历史上的答法（GPU 包那一半的攻方腿用例量的）。

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

    /// 攻方腿模型（核心层这一半）：挂载点「0000:01:00.0」；`switch_to_the_cpu_first` 时先切到 `cpu`；请求切到没开的「0000:41:00.0」，
    /// 实现答还在开卡、之后答开不成（核心层照设计问题 ③(a) 把它记成生效、停着）；用户再请求同一组（停着时的切换：当场解除），实现答还在开卡。
    /// 报：开卡期间那一批走哪条路、开不成那一批开没开、开不成之后生效的那一组点哪张卡、再请求之后那一批走哪条路、停没停、交没交给了实现还在开卡。
    fn attack_routes_of_the_retry_after_not_opened(switch_to_the_cpu_first: bool) {
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
        attack_answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
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
            "name=attack_core_retry_after_not_opened switched_to_the_cpu_first={switch_to_the_cpu_first} times_told_no_gpu_takes_part={times_told_no_gpu_takes_part} route_while_opening={route_while_opening:?} route_once_not_opened={route_once_not_opened:?} devices_in_effect_once_not_opened={devices_in_effect_once_not_opened:?} route_at_the_retry={route_at_the_retry:?} stopped_at_the_retry={stopped_at_the_retry} handed_in_at_the_retry={handed_in_at_the_retry}"
        );
        let expected_route_while_opening = if switch_to_the_cpu_first {
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings,
            ))
        } else {
            Some(BackgroundComputeRoute::Gpu)
        };
        assert_eq!(route_while_opening, expected_route_while_opening);
        assert_eq!(route_once_not_opened, None, "开不成：停着、不开批");
        assert_eq!(
            devices_in_effect_once_not_opened,
            Some(vec![card_for_the_switch_tests(b"0000:41:00.0")]),
            "设计问题 ③(a)：开不成的那一组记成生效"
        );
        assert_eq!(
            route_at_the_retry,
            Some(BackgroundComputeRoute::Gpu),
            "再请求之后实现还在开卡，这一批照生效的那一组（开不成的那一组，选 gpu）走 GPU"
        );
        assert!(!stopped_at_the_retry);
        assert!(handed_in_at_the_retry);
    }

    /// 先切到 `cpu` 的那一臂：再请求之后那一批走 GPU，而实现那一刻一张卡都没开（GPU 包那一半：这一批 panic）。
    #[test]
    fn attack_core_retry_after_not_opened_following_the_cpu_routes_the_batch_to_the_gpu() {
        attack_routes_of_the_retry_after_not_opened(true);
    }

    /// 不先切到 `cpu` 的那一臂：再请求之后那一批走 GPU，实现那一刻参与的是切换之前那张「0000:01:00.0」（GPU 包那一半：这一批分给它）。
    #[test]
    fn attack_core_retry_after_not_opened_on_the_gpu_routes_the_batch_to_the_gpu() {
        attack_routes_of_the_retry_after_not_opened(false);
    }

    /// 攻方腿模型（新冒出来的一格）：挂载点「0000:01:00.0」，请求切到没开的「0000:41:00.0」；实现开卡的这段时间每一批都在「0000:01:00.0」上出错
    /// （用户切走它正是因为它坏了），池子最满的那一块盘已过补跑阈值。报：每一批走哪条路、出错之后怎样了、之后停没停。
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
        for _batch_while_opening in 0..3 {
            routes.push(attack_begin_a_batch(
                &mut background_compute,
                attack_fullest_device_above_the_threshold(),
            ));
            after_each_error.push(background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                failure_on_the_card_at_one(),
            ));
        }
        let stopped = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        println!(
            "name=attack_errors_while_opening arm=switch_opening routes={routes:?} after_each_error={after_each_error:?} stopped={stopped}"
        );
        assert!(routes
            .iter()
            .all(|route| *route == Some(BackgroundComputeRoute::Gpu)));
        assert!(after_each_error.iter().all(|after| *after
            == BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError));
        assert!(!stopped, "开卡的这段时间一次都不停、不报警、不补跑");
    }

    /// 对照臂：同一个错、同样过了补跑阈值，没有切换——停下，下一批在 CPU 上补跑。
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
        assert_eq!(
            after_the_error,
            BackgroundTaskAfterTheComputeError::LeftStopped
        );
        assert!(compensating);
    }

    // ── 第一轮攻方腿五条核心层用例的历史，在修补之后的代码上该读出什么（断言写条款要的结局；实现的答法照产品外壳在同一段历史上答的摆）──

    /// 甲：挂载点「0000:01:00.0」，请求切到没开的「0000:41:00.0」；交给实现那一批（实现还在开卡）在「0000:01:00.0」上出错。
    /// 条款：出错之前已经收到、还没生效的切换请求照它解除停下；卡开成换上之后那一批照常开、生效的是点「0000:41:00.0」的那一组。
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
            BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError
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
