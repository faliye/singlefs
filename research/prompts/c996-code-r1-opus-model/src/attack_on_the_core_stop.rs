
    // ── 攻方腿模型（c996-code-r1）：切换开卡期间核心层的停下与解除，接在 background_compute.rs 的测试模块末尾 ──
    // 替身只照 `BackgroundGpuCompute` 文档写的那一份约定：点到没开的卡就在会话锁外开、开着的这段时间照交来之前那一组跑，
    // 开完了在两批之间那一问里换上、那一问交回 `None`（产品 GPU 包那一份的同一行为由 background_gpu.rs 里攻方腿的几条用例对着真外壳量）。

    fn attack_card(address: &[u8]) -> GpuDeviceIdentifier {
        GpuDeviceIdentifier::of_opaque_bytes(address.to_vec())
    }

    /// 替身的状态：要开的那张卡（`card_needing_an_opening`）开没开成；此刻这组设置下一批分不分得到卡上（`failure_under_the_settings_in_use`）；
    /// 一组设置换上（当场或开完）之后它答什么（`failure_once_put_in_effect`）；用例把 `opening_ends` 摆成 `true` 就算开卡线程开完了。
    struct AttackDeferredOpeningState {
        card_needing_an_opening: GpuDeviceIdentifier,
        card_opened: bool,
        opening_under_way: bool,
        opening_ends: bool,
        failure_under_the_settings_in_use: Option<BackgroundGpuComputeFailed>,
        failure_once_put_in_effect: Option<BackgroundGpuComputeFailed>,
    }

    struct AttackDeferredOpeningForTests {
        state: Arc<Mutex<AttackDeferredOpeningState>>,
    }

    impl GpuInventory for AttackDeferredOpeningForTests {
        fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
            vec![
                (attack_card(b"0000:01:00.0"), GpuMemoryInBytes(8 << 30)),
                (attack_card(b"0000:41:00.0"), GpuMemoryInBytes(8 << 30)),
            ]
        }
    }

    impl BackgroundGpuCompute for AttackDeferredOpeningForTests {
        fn verify_unit_checksums(
            &mut self,
            _units: &[UnitChecksumToVerifyOnTheGpu<'_>],
        ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>
        {
            Ok(Vec::new())
        }

        fn classify_region_records(
            &mut self,
            _region: crate::logged_operations::CompactionRegion,
            _records: &[RegionRecordToClassifyOnTheGpu<'_>],
            _mapping_entries: &[crate::region_scan::CentralMappingEntry],
        ) -> Result<
            Vec<GpuEntryDecision<crate::region_scan::AllocationInRegion>>,
            BackgroundGpuComputeFailed,
        > {
            Ok(Vec::new())
        }

        fn count_live_slots_per_cluster_segment(
            &mut self,
            _allocation_records: &[crate::allocator::AllocationRecord],
            _unit_area: crate::compaction_plan::UnitAreaOfOneDevice,
            _segments: &[ClusterSegmentEntryKey],
        ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
            Ok(Vec::new())
        }

        fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
            let mut state = self.state.lock().expect("用例没在别处 panic 过");
            let names_the_card_not_opened = !state.card_opened
                && settings
                    .participating_devices()
                    .iter()
                    .any(|participating| participating.device == state.card_needing_an_opening);
            if names_the_card_not_opened {
                state.opening_under_way = true;
            } else {
                state.failure_under_the_settings_in_use = state.failure_once_put_in_effect.clone();
            }
        }

        fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
            &mut self,
        ) -> Option<BackgroundGpuComputeFailed> {
            let mut state = self.state.lock().expect("用例没在别处 panic 过");
            if state.opening_under_way && state.opening_ends {
                state.opening_under_way = false;
                state.card_opened = true;
                state.failure_under_the_settings_in_use = state.failure_once_put_in_effect.clone();
            }
            None
        }

        fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
            self.state
                .lock()
                .expect("用例没在别处 panic 过")
                .failure_under_the_settings_in_use
                .clone()
        }
    }

    fn attack_gpu_settings_naming(
        address: &[u8],
        memory_limit: Option<u64>,
    ) -> BackgroundComputeSettings {
        BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![attack_card(address)]),
            background_gpu_memory_limit: memory_limit.map(|limit| vec![GpuMemoryInBytes(limit)]),
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        }
    }

    /// 池子最满的那一块盘远没到补跑阈值：停着就照旧停着。
    fn attack_fullest_device_far_below_the_threshold() -> AllocatedBytesAndCapacityOfADevice {
        AllocatedBytesAndCapacityOfADevice {
            device: crate::address::DeviceIdentity(0),
            allocated_bytes: 0,
            capacity_bytes: 1 << 30,
        }
    }

    fn attack_every_gpu_left_out(address: &[u8]) -> BackgroundGpuComputeFailed {
        BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
            gpus_left_out: vec![GpuLeftOutUnderItsMemoryLimit {
                device: attack_card(address),
                memory_limit: GpuMemoryInBytes(64),
                smallest_piece_bytes: 972,
                bytes_already_held_on_the_device: 0,
            }],
        }
    }

    /// 挂载点「0000:01:00.0」的一次挂载，替身状态照 `state`。
    fn attack_mount_on_the_card_at_one(
        state: &Arc<Mutex<AttackDeferredOpeningState>>,
    ) -> BackgroundComputeOfThisMount {
        let handle =
            BackgroundGpuComputeHandle::of_an_implementation(AttackDeferredOpeningForTests {
                state: Arc::clone(state),
            });
        BackgroundComputeOfThisMount::of_a_mount(
            admit_background_compute_settings_with_the_gpu_compute(
                attack_gpu_settings_naming(b"0000:01:00.0", None),
                Some(handle),
            )
            .expect("卡清单列着「0000:01:00.0」"),
        )
    }

    fn attack_state(
        failure_under_the_settings_in_use: Option<BackgroundGpuComputeFailed>,
        failure_once_put_in_effect: Option<BackgroundGpuComputeFailed>,
    ) -> Arc<Mutex<AttackDeferredOpeningState>> {
        Arc::new(Mutex::new(AttackDeferredOpeningState {
            card_needing_an_opening: attack_card(b"0000:41:00.0"),
            card_opened: false,
            opening_under_way: false,
            opening_ends: false,
            failure_under_the_settings_in_use,
            failure_once_put_in_effect,
        }))
    }

    fn attack_failure_on_the_card_at_one() -> BackgroundComputeError {
        BackgroundComputeError::GpuSegmentFailed {
            segment: GpuComputedSegment::UnitChecksumVerification,
            failure: BackgroundGpuComputeFailed::GpuFailed {
                device: attack_card(b"0000:01:00.0"),
                cause: GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
                    reason: "攻方腿模型：切换之前那张卡上设备丢了".to_string(),
                },
            },
        }
    }

    /// 攻方腿模型（第 4 条「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下」）：挂载点「0000:01:00.0」，请求切到没开的「0000:41:00.0」；
    /// 下一批开头核心层把它交给实现（实现开着卡、照切换之前那一组在「0000:01:00.0」上跑），这一批在「0000:01:00.0」上出错；之后卡开成、换上。
    /// 报：出错那一刻停没停、卡换上之后那一批开不开、停着报的是哪张卡。
    #[test]
    fn attack_a_stop_on_the_card_before_the_switch_while_the_new_card_opens_outlives_the_switch_taking_effect(
    ) {
        let state = attack_state(None, None);
        let mut background_compute = attack_mount_on_the_card_at_one(&state);
        background_compute
            .request_switch_from_the_next_batch(
                attack_gpu_settings_naming(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        let batch_while_opening = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            attack_fullest_device_far_below_the_threshold(),
        );
        let opening_under_way = state
            .lock()
            .expect("用例没在别处 panic 过")
            .opening_under_way;
        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            attack_failure_on_the_card_at_one(),
        );
        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
        let batch_once_opened = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            attack_fullest_device_far_below_the_threshold(),
        );
        let card_opened = state.lock().expect("用例没在别处 panic 过").card_opened;
        let devices_named: Vec<Vec<GpuDeviceIdentifier>> = background_compute
            .stopped_after_a_compute_error()
            .iter()
            .map(|stopped| stopped.error.gpu_devices_named())
            .collect();
        println!(
            "name=attack_stop_outlives_the_switch arm=error_after_the_handing_in batch_while_opening_begun={} opening_under_way={opening_under_way} after_the_error={after_the_error:?} card_opened={card_opened} batch_once_opened_begun={} stopped={} devices_named={devices_named:?}",
            batch_while_opening.is_ok(),
            batch_once_opened.is_ok(),
            background_compute.is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation)
        );
        assert!(batch_while_opening.is_ok() && opening_under_way);
        assert_eq!(
            after_the_error,
            BackgroundTaskAfterTheComputeError::LeftStopped
        );
        assert!(card_opened, "新卡开成、换上了");
        assert!(batch_once_opened.is_err(), "换上之后照旧停着");
        assert_eq!(devices_named, vec![vec![attack_card(b"0000:01:00.0")]]);
    }

    /// 同一段历史的对照臂：同一个错落在核心层把切换交给实现之前（请求还挂着）——照那一次请求当场解除。两臂里实现都还在切换之前那张卡上。
    #[test]
    fn attack_the_same_error_before_the_handing_in_is_lifted_by_the_switch_request() {
        let state = attack_state(None, None);
        let mut background_compute = attack_mount_on_the_card_at_one(&state);
        background_compute
            .request_switch_from_the_next_batch(
                attack_gpu_settings_naming(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            attack_failure_on_the_card_at_one(),
        );
        println!(
            "name=attack_stop_outlives_the_switch arm=error_before_the_handing_in after_the_error={after_the_error:?} stopped={}",
            background_compute.is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation)
        );
        assert_eq!(
            after_the_error,
            BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError
        );
    }

    /// 攻方腿模型（第 4 条「停着的时候收到任何一次通过校验的切换请求……当场解除停下，从下一批起用新设置」）：挂载那一刻「0000:01:00.0」参与的卡一张都不剩，
    /// 核心层挂载就停着；请求切到没开的「0000:41:00.0」（装得下）。替身照真外壳（background_gpu.rs 攻方腿那一条）：交设置那一刻答的还是切换之前那一组的「一张都不剩」，
    /// 开完换上之后答用得上。报：请求之后停没停、下一批开头停没停、卡换上之后那一批开不开。
    #[test]
    fn attack_the_stop_of_the_settings_before_the_switch_is_put_back_at_the_handing_in_and_outlives_the_opening(
    ) {
        // 挂载那一刻换上的那一组答「一张都不剩」；之后换上的（切到「0000:41:00.0」开完）答用得上。
        let state = attack_state(None, Some(attack_every_gpu_left_out(b"0000:01:00.0")));
        let mut background_compute = attack_mount_on_the_card_at_one(&state);
        state
            .lock()
            .expect("用例没在别处 panic 过")
            .failure_once_put_in_effect = None;
        let stopped_at_the_mount = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        background_compute
            .request_switch_from_the_next_batch(
                attack_gpu_settings_naming(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        let stopped_after_the_request = background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation);
        let batch_at_the_handing_in = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            attack_fullest_device_far_below_the_threshold(),
        );
        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
        let batch_once_opened = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            attack_fullest_device_far_below_the_threshold(),
        );
        let failure_now = state
            .lock()
            .expect("用例没在别处 panic 过")
            .failure_under_the_settings_in_use
            .clone();
        println!(
            "name=attack_stale_stop stopped_at_the_mount={stopped_at_the_mount} stopped_after_the_request={stopped_after_the_request} batch_at_the_handing_in_begun={} batch_once_opened_begun={} stopped={} implementation_usable_now={}",
            batch_at_the_handing_in.is_ok(),
            batch_once_opened.is_ok(),
            background_compute.is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
            failure_now.is_none()
        );
        assert!(stopped_at_the_mount && !stopped_after_the_request);
        assert!(batch_at_the_handing_in.is_err(), "交设置那一刻又停下");
        assert!(failure_now.is_none(), "实现换上之后用得上");
        assert!(batch_once_opened.is_err(), "核心层照旧停着");
    }

    /// 攻方腿模型（第 4 条「挂着切换到参与的卡一张都不剩的设置……报的是挂载那一刻同一个成员（GpuUnusableUnderTheSettingsHandedIn）」）：
    /// 请求切到没开的「0000:41:00.0」、给它装不下最小的一片的上限。替身照真外壳：开完换上之后答「一张都不剩」，两批之间那一问交回 `None`。
    /// 报：卡换上之后那一批开不开、走哪条路、停没停。对照臂见下一条（同样的设置点的是开着的卡）。
    #[test]
    fn attack_the_switch_to_every_card_left_out_put_in_effect_by_the_opening_starts_a_gpu_batch_instead_of_stopping(
    ) {
        let state = attack_state(None, None);
        let mut background_compute = attack_mount_on_the_card_at_one(&state);
        // 挂载那一组用得上；之后换上的那一组（开完的「0000:41:00.0」、64 字节上限）答「一张都不剩」。
        state
            .lock()
            .expect("用例没在别处 panic 过")
            .failure_once_put_in_effect = Some(attack_every_gpu_left_out(b"0000:41:00.0"));
        background_compute
            .request_switch_from_the_next_batch(
                attack_gpu_settings_naming(b"0000:41:00.0", Some(64)),
                None,
            )
            .expect("上限不超过显存总量：照收");
        let batch_while_opening = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            attack_fullest_device_far_below_the_threshold(),
        );
        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
        let batch_once_opened = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            attack_fullest_device_far_below_the_threshold(),
        );
        let on_the_gpu = batch_once_opened
            .as_ref()
            .is_ok_and(|under_way| matches!(under_way.route, BackgroundComputeRoute::Gpu));
        println!(
            "name=attack_left_out_after_the_opening arm=card_not_open_at_the_switch batch_while_opening_begun={} batch_once_opened_begun={} batch_once_opened_on_the_gpu={on_the_gpu} stopped={}",
            batch_while_opening.is_ok(),
            batch_once_opened.is_ok(),
            background_compute.is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation)
        );
        assert!(on_the_gpu, "换上之后那一批照 GPU 开了");
        assert!(!background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
    }

    /// 对照臂：同样「一张都不剩」的设置，点的是挂载那一刻开着的「0000:01:00.0」——交设置那一刻就停着、报挂载那一刻同一个成员。
    #[test]
    fn attack_the_switch_to_every_card_left_out_on_a_card_already_open_stops_at_the_handing_in() {
        let state = attack_state(None, None);
        let mut background_compute = attack_mount_on_the_card_at_one(&state);
        // 挂载那一组用得上；之后当场换上的那一组（开着的「0000:01:00.0」、64 字节上限）答「一张都不剩」。
        state
            .lock()
            .expect("用例没在别处 panic 过")
            .failure_once_put_in_effect = Some(attack_every_gpu_left_out(b"0000:01:00.0"));
        background_compute
            .request_switch_from_the_next_batch(
                attack_gpu_settings_naming(b"0000:01:00.0", Some(64)),
                None,
            )
            .expect("上限不超过显存总量：照收");
        let batch_at_the_handing_in = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            attack_fullest_device_far_below_the_threshold(),
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
            "name=attack_left_out_after_the_opening arm=card_already_open batch_at_the_handing_in_begun={} stopped_by_gpu_unusable_under_the_settings_handed_in={stopped_by_the_member}",
            batch_at_the_handing_in.is_ok()
        );
        assert!(batch_at_the_handing_in.is_err() && stopped_by_the_member);
    }
