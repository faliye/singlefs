
    // ── 攻方腿 m4-gpugc-code-r3 的探针（只接进腿或主 agent 复跑时的副本，不进仓）：只打 ATTACK 行，不断言 ──

    /// 一段文字的前 `count` 个字（ATTACK 行里放原因用，不让一行太长）。
    fn attack_probe_r3_first_chars(text: &str, count: usize) -> String {
        text.chars()
            .take(count)
            .collect::<String>()
            .replace('\n', " ")
    }

    /// wgpu 30.0.1 的 noop 后端起一个设备（`wgpu::Device::noop`：不起 GPU、不碰 Vulkan，建缓冲在主机内存里 `try_reserve_exact`，
    /// 建不成报 `DeviceError::OutOfMemory`），装上与产品开卡相同的两个回调，把没接住的错与设备丢了记进交回的那一份。
    fn attack_probe_r3_noop_device_recording_what_wgpu_reports(
    ) -> (wgpu::Device, wgpu::Queue, Arc<Mutex<Vec<String>>>) {
        let (device, queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor {
            label: Some("attack-probe-r3-noop"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        });
        let reports: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let reports_of_uncaptured_errors = Arc::clone(&reports);
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            reports_of_uncaptured_errors
                .lock()
                .expect("探针没在别处 panic 过")
                .push(format!(
                    "uncaptured:{}",
                    attack_probe_r3_first_chars(&format!("{error}"), 80)
                ));
        }));
        let reports_of_the_loss = Arc::clone(&reports);
        device.set_device_lost_callback(move |reason, message| {
            reports_of_the_loss
                .lock()
                .expect("探针没在别处 panic 过")
                .push(format!(
                    "lost:{reason:?}:{}",
                    attack_probe_r3_first_chars(&message, 80)
                ));
        });
        (device, queue, reports)
    }

    /// 本进程的地址空间上限（RLIMIT_AS）此刻压到「此刻的 VmSize 加 `bytes_beyond_the_present_size`」跑 `run`，跑完放回原来的上限：
    /// noop 后端在主机内存里建缓冲，压住之后建一份比这一截大的缓冲 `try_reserve_exact` 就失败、wgpu 报显存不足（不起 GPU 造出 wgpu 自己报的显存不足）。
    /// 交回压之前的 VmSize 与 `run` 的结局。
    #[allow(
        unsafe_code,
        reason = "探针：读写本进程的地址空间上限，std 没有 getrlimit/setrlimit"
    )]
    fn attack_probe_r3_with_the_address_space_capped<Outcome>(
        bytes_beyond_the_present_size: u64,
        run: impl FnOnce() -> Outcome,
    ) -> (u64, Outcome) {
        let status = std::fs::read_to_string("/proc/self/status").expect("读得到 /proc/self/status");
        let present_kibibytes: u64 = status
            .lines()
            .find_map(|line| line.strip_prefix("VmSize:"))
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|kibibytes| kibibytes.parse().ok())
            .expect("/proc/self/status 有 VmSize 一行");
        let present_bytes = present_kibibytes * 1024;
        let mut before = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: `before` 是这个栈帧里初始化过的 `rlimit`，getrlimit 只往它里面写。
        let read = unsafe { libc::getrlimit(libc::RLIMIT_AS, &raw mut before) };
        assert_eq!(read, 0, "getrlimit(RLIMIT_AS)");
        let capped = libc::rlimit {
            rlim_cur: present_bytes + bytes_beyond_the_present_size,
            rlim_max: before.rlim_max,
        };
        // SAFETY: `capped` 是这个栈帧里初始化过的 `rlimit`，setrlimit 只读它。
        let set = unsafe { libc::setrlimit(libc::RLIMIT_AS, &raw const capped) };
        assert_eq!(set, 0, "setrlimit(RLIMIT_AS) 压上限");
        let outcome = run();
        // SAFETY: `before` 是上面 getrlimit 读回的那一份，setrlimit 只读它。
        let restored = unsafe { libc::setrlimit(libc::RLIMIT_AS, &raw const before) };
        assert_eq!(restored, 0, "setrlimit(RLIMIT_AS) 放回");
        (present_bytes, outcome)
    }

    /// 建一段那条泛型函数的结局，写成 ATTACK 行里的一个词。
    fn attack_probe_r3_outcome_of_the_chunk<Buffers, Readback>(
        created: &Result<BuffersOfAChunk<Buffers, Readback>, GpuFailureCause>,
    ) -> String {
        match created {
            Ok(_buffers_and_readbacks) => "Ok".to_string(),
            Err(cause) => format!("Err:{}", attack_probe_r3_first_chars(&format!("{cause:?}"), 60)),
        }
    }

    /// 攻方 T3（第 60 批第 1 条的替身照什么规矩记错）：替身 `DeviceOfAChunkInMemoryForTests` 把这一段自己那几样缓冲与回读暂存一样记——
    /// 建不成就记进最里层范围、照常往下走，所以「这一段自己的缓冲建不成」在替身上也报成 `OutOfMemoryCreatingTheBuffers`。
    /// 产品建这一段自己的缓冲用的是 `create_buffer_init`（`GpuInUse::buffer_with_contents`），wgpu 30.0.1 里它是 `mapped_at_creation: true`
    /// 的 `create_buffer` 接 `get_mapped_range_mut(..).expect(..)`（`wgpu-30.0.1/src/util/device.rs`）：缓冲建不成时那一句 panic，范围没机会关。
    /// 这里在 noop 后端上走产品那一份 `impl DeviceCreatingTheBuffersOfAChunk for wgpu::Device`，五格：
    /// 不压上限的阳性对照；压住之后只让回读暂存建不成；回读暂存建在范围之外（第 60 批那一行变异的形状）；这一段自己的缓冲用 `create_buffer`；
    /// 这一段自己的缓冲用产品的 `create_buffer_init`。
    #[test]
    fn attack_probe_r3_wgpu_noop_out_of_memory_inside_the_scope_of_a_chunk() {
        const CHUNK_BYTES: u64 = 128 << 20;
        const ROOM_BEYOND_THE_PRESENT_SIZE: u64 = 32 << 20;
        let (device, _queue, reports) = attack_probe_r3_noop_device_recording_what_wgpu_reports();
        let reports_so_far = |reports: &Arc<Mutex<Vec<String>>>| {
            reports.lock().expect("探针没在别处 panic 过").clone()
        };

        let small_chunk = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
            &device,
            Duration::from_secs(10),
            0,
            64,
            || {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &[7_u8; 16],
                    usage: wgpu::BufferUsages::STORAGE,
                })
            },
        );
        println!(
            "ATTACK name=noop_chunk_within_the_room outcome={} reports={:?}",
            attack_probe_r3_outcome_of_the_chunk(&small_chunk),
            reports_so_far(&reports)
        );
        drop(small_chunk);

        let (size_before_the_readbacks, readbacks_short) =
            attack_probe_r3_with_the_address_space_capped(ROOM_BEYOND_THE_PRESENT_SIZE, || {
                attack_probe_r3_outcome_of_the_chunk(
                    &buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
                        &device,
                        Duration::from_secs(10),
                        2 * CHUNK_BYTES,
                        CHUNK_BYTES,
                        || (),
                    ),
                )
            });
        println!(
            "ATTACK name=noop_readbacks_short_inside_the_scope vm_size_bytes={size_before_the_readbacks} room_beyond_bytes={ROOM_BEYOND_THE_PRESENT_SIZE} readback_bytes={CHUNK_BYTES} outcome={readbacks_short} reports={:?}",
            reports_so_far(&reports)
        );

        let (size_before_the_readback_outside, captured_by_the_closed_scope) =
            attack_probe_r3_with_the_address_space_capped(ROOM_BEYOND_THE_PRESENT_SIZE, || {
                let scope = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
                let captured = wait_for_the_wgpu_future(
                    scope.pop(),
                    Duration::from_secs(10),
                    &SteadyClockOfThisProcess::starting_now(),
                );
                let readback_outside = device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: CHUNK_BYTES,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                drop(readback_outside);
                match captured {
                    Ok(None) => "None".to_string(),
                    Ok(Some(error)) => format!("Some:{}", attack_probe_r3_first_chars(&format!("{error}"), 60)),
                    Err(cause) => format!("Err:{cause:?}"),
                }
            });
        println!(
            "ATTACK name=noop_readback_short_outside_the_scope vm_size_bytes={size_before_the_readback_outside} captured_by_the_closed_scope={captured_by_the_closed_scope} reports={:?}",
            reports_so_far(&reports)
        );

        let (size_before_the_plain_buffer, plain_buffer_short) =
            attack_probe_r3_with_the_address_space_capped(ROOM_BEYOND_THE_PRESENT_SIZE, || {
                attack_probe_r3_outcome_of_the_chunk(
                    &buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
                        &device,
                        Duration::from_secs(10),
                        CHUNK_BYTES + 128,
                        64,
                        || {
                            device.create_buffer(&wgpu::BufferDescriptor {
                                label: None,
                                size: CHUNK_BYTES,
                                usage: wgpu::BufferUsages::STORAGE,
                                mapped_at_creation: false,
                            })
                        },
                    ),
                )
            });
        println!(
            "ATTACK name=noop_chunk_buffer_created_unmapped_short vm_size_bytes={size_before_the_plain_buffer} buffer_bytes={CHUNK_BYTES} outcome={plain_buffer_short} reports={:?}",
            reports_so_far(&reports)
        );

        // 产品 `GpuInUse::buffer_with_contents` 的写法：内容先在主机上备好（压上限之前），压住之后建。
        let contents = vec![7_u8; usize::try_from(CHUNK_BYTES).expect("装得进 usize")];
        let (size_before_the_buffer_with_contents, buffer_with_contents_short) =
            attack_probe_r3_with_the_address_space_capped(ROOM_BEYOND_THE_PRESENT_SIZE, || {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    attack_probe_r3_outcome_of_the_chunk(
                        &buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
                            &device,
                            Duration::from_secs(10),
                            CHUNK_BYTES + 128,
                            64,
                            || {
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: None,
                                    contents: &contents,
                                    usage: wgpu::BufferUsages::STORAGE,
                                })
                            },
                        ),
                    )
                }))
            });
        let buffer_with_contents_short = match buffer_with_contents_short {
            Ok(outcome) => format!("returned:{outcome}"),
            Err(payload) => format!(
                "panicked:{}",
                attack_probe_r3_first_chars(
                    &payload
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))
                        .unwrap_or_else(|| "（不是文字）".to_string()),
                    110
                )
            ),
        };
        println!(
            "ATTACK name=noop_chunk_buffer_with_contents_short vm_size_bytes={size_before_the_buffer_with_contents} buffer_bytes={CHUNK_BYTES} outcome={buffer_with_contents_short} reports={:?}",
            reports_so_far(&reports)
        );
        drop(contents);
    }

    /// 攻方 T1（第 59 批第 2 条「重交只数真交出去的」）在纯函数上：一次交等超过上限；重交之前那一等等到了，重交那一次在建缓冲时显存不足
    /// （`OutOfMemoryCreatingTheBuffers`：放掉这一段的账、一次都不交）。记下的重交次数与真交出去的次数。
    #[test]
    fn attack_probe_r3_resubmission_counted_when_the_resubmission_runs_out_of_memory_before_submitting(
    ) {
        let mut script: VecDeque<Result<(), GpuFailureCause>> = VecDeque::from([
            Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
            }),
            Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                bytes_held_by_the_chunk: 2000,
                reason: "攻方脚本：重交那一次卡上空闲的不够".to_string(),
            }),
        ]);
        let mut attempts: Vec<&'static str> = Vec::new();
        let mut timeouts = crate::waiting_for_the_gpu::TimeoutsOfTheBatchOnOneGpu::none_yet();
        let outcome = crate::waiting_for_the_gpu::outcome_resubmitted_after_timeouts(
            &mut attempts,
            &mut timeouts,
            |_attempts| Ok(()),
            |attempts| {
                let next = script.pop_front().expect("脚本只走两次");
                attempts.push(match &next {
                    Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }) => "not_submitted",
                    _ => "submitted",
                });
                next
            },
        );
        let submitted_after_the_timeout = attempts
            .iter()
            .skip(1)
            .filter(|attempt| **attempt == "submitted")
            .count();
        println!(
            "ATTACK name=resubmission_out_of_memory_before_submitting outcome={} attempts={attempts:?} timeouts={} resubmissions_recorded={} resubmissions_actually_submitted={submitted_after_the_timeout}",
            match &outcome {
                Ok(()) => "Ok".to_string(),
                Err(cause) => attack_probe_r3_first_chars(&format!("{cause:?}"), 40),
            },
            timeouts.timeouts(),
            timeouts.resubmissions()
        );
    }

    /// 替身卡外面套一层：头一次交等超过上限之后，卡上此刻空闲的掉到那一段的一半（别的进程在这张卡上建了缓冲）。每次交记一笔。
    struct CardWhoseFreeMemoryHalvesAfterItsFirstTimeout {
        inner: CardComputedOnTheCpuForTests,
        has_halved: bool,
        chunks: Vec<String>,
    }

    impl CardOfTheBackgroundGpu for CardWhoseFreeMemoryHalvesAfterItsFirstTimeout {
        type Submission = u32;

        fn discrete_gpu(&self) -> &DiscreteGpuOnThisHost {
            self.inner.discrete_gpu()
        }

        fn budget(&self) -> &GpuBudgetUnderTheSettings<u32> {
            self.inner.budget()
        }

        fn budget_mut(&mut self) -> &mut GpuBudgetUnderTheSettings<u32> {
            self.inner.budget_mut()
        }

        fn measured_cost(&self) -> Option<HostCpuNanosecondsPerSubmission> {
            self.inner.measured_cost()
        }

        fn measure_the_cost_of_one_submission(&mut self) -> Result<(), GpuFailureCause> {
            self.inner.measure_the_cost_of_one_submission()
        }

        fn room_once_the_submission_running_past_its_wait_completed(
            &mut self,
        ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause> {
            self.inner
                .room_once_the_submission_running_past_its_wait_completed()
        }

        fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause> {
            self.inner.wait_for_the_timed_out_submission()
        }

        fn both_kernel_outcomes_of_a_checksum_chunk(
            &mut self,
            units: &[UnitChecksumToVerify<'_, ()>],
        ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause> {
            let input_bytes: usize = units.iter().map(|unit| unit.unit_bytes.len()).sum();
            let bytes_held_by_the_chunk = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
                u64::try_from(input_bytes).expect("装得进 u64"),
                u64::try_from(units.len()).expect("装得进 u64"),
            );
            let submissions_before = self.inner.submissions_made;
            let outcome = self.inner.both_kernel_outcomes_of_a_checksum_chunk(units);
            let went_out = self.inner.submissions_made > submissions_before;
            self.chunks.push(format!(
                "{bytes_held_by_the_chunk}B/{}/{}",
                if went_out { "submitted" } else { "not_submitted" },
                match &outcome {
                    Ok(_outcomes) => "ok".to_string(),
                    Err(cause) => attack_probe_r3_first_chars(&format!("{cause:?}"), 30),
                }
            ));
            if !self.has_halved
                && matches!(
                    outcome,
                    Err(GpuFailureCause::WaitForTheGpuExceededTheLimit { .. })
                )
            {
                self.inner.free_bytes = Some(bytes_held_by_the_chunk / 2);
                self.has_halved = true;
            }
            outcome
        }

        fn both_kernel_outcomes_of_a_region_record_chunk(
            &mut self,
            pieces: &[RegionRecordPieceForTheKernels<'_>],
            mapping_tables: Option<&MappingTablesForTheKernels>,
        ) -> Result<BothKernelOutcomesOfARegionRecordChunk, GpuFailureCause> {
            self.inner
                .both_kernel_outcomes_of_a_region_record_chunk(pieces, mapping_tables)
        }

        fn both_kernel_live_slots_of_a_chunk(
            &mut self,
            segments: &[ClusterSegmentEntryKey],
            record_tables: &AllocationRecordTablesForTheKernels,
            unit_area: UnitAreaOfOneDevice,
        ) -> Result<Vec<(u64, u64)>, GpuFailureCause> {
            self.inner
                .both_kernel_live_slots_of_a_chunk(segments, record_tables, unit_area)
        }
    }

    /// 攻方 T1（第 59 批第 2 条）在外壳上：一张卡、校验和那一段四条各 1000 字节的单元整段一次交；头一次交等超过上限，之后卡上空闲的掉到那一段的一半，
    /// 重交那一次建缓冲显存不足（一次都没交出去）、按一半的地方重切、从头交完。这一段记下的重交次数与这张卡上真交出去的次数。
    #[test]
    fn attack_probe_r3_resubmission_counted_on_the_card_when_free_memory_halves_after_a_timeout() {
        let mut inner = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        inner.submissions_timing_out = VecDeque::from([true]);
        let discrete_gpu = inner.discrete_gpu.clone();
        let identifier = discrete_gpu.identifier.clone();
        let mut background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
            vec![CardWhoseFreeMemoryHalvesAfterItsFirstTimeout {
                inner,
                has_halved: false,
                chunks: Vec::new(),
            }],
            DiscreteGpusAfterTheOpening {
                opened: vec![discrete_gpu],
                not_opened: Vec::new(),
            },
            Vec::new(),
            ParticipatingGpusOnThisHost::of_the_positions(vec![0]),
            Vec::new(),
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(vec![
                identifier,
            ])))),
        );
        let unit_bytes = unit_bytes_of(4, 1000);
        let units = units_to_verify(&unit_bytes);
        let outcome = background_gpu.both_kernel_outcomes_of_the_batch(&units);
        let returned_as_on_the_cpu = outcome == Ok(cpu_reference_outcomes_of(&units));
        let submissions = background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification;
        let card = &background_gpu.cards_on_this_host[0];
        println!(
            "ATTACK name=resubmission_counted_when_free_memory_halves_after_a_timeout returned_as_on_the_cpu={returned_as_on_the_cpu} resubmissions_after_timeouts={} recuts_after_running_out_of_memory={} submissions_completed={} submissions_made_on_the_card={} chunks={:?}",
            submissions.resubmissions_after_timeouts,
            submissions.recuts_after_running_out_of_memory,
            submissions.submissions_completed,
            card.inner.submissions_made,
            card.chunks
        );
    }

    /// 攻方 T1（第 59 批第 1 条「一次切换只现枚举一次」之后挂着时卡掉了）：两张开成、都参与的卡，挂载之后 0000:01:00.0 掉了（此刻本机只在 0000:41:00.0）。
    /// 四种切换：不点名不给上限、点名掉了的那张、不点名照挂载时给两个上限、不点名给一个上限。每种的校验结局、照它换上之后参与的卡与现枚举了几次。
    #[test]
    fn attack_probe_r3_switches_after_a_participating_card_dropped_while_mounted() {
        let dropped = card_identifier("0000:01:00.0");
        let remaining = card_identifier("0000:41:00.0");
        let one_gibibyte = GpuMemoryInBytes(1 << 30);
        let switches: [(&str, Option<Vec<GpuDeviceIdentifier>>, Option<Vec<GpuMemoryInBytes>>); 4] = [
            ("unnamed_no_limit", None, None),
            ("naming_the_dropped_card", Some(vec![dropped.clone()]), None),
            ("unnamed_two_limits", None, Some(vec![one_gibibyte, one_gibibyte])),
            ("unnamed_one_limit", None, Some(vec![one_gibibyte])),
        ];
        for (shape, background_gpu_devices, background_gpu_memory_limit) in switches {
            let (mut background_gpu, present) = background_gpu_over_cards_on_the_cpu(
                vec![
                    CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0),
                    CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
                ],
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
            present
                .lock()
                .expect("探针没在别处 panic 过")
                .retain(|identifier| *identifier != dropped);
            let admitted = admit_gpu_settings_against_the_inventory(
                &BackgroundComputeSettings {
                    background_compute: BackgroundComputeBackend::Gpu,
                    background_gpu_devices,
                    background_gpu_memory_limit,
                    background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                },
                &background_gpu,
            );
            let (outcome, participating) = match &admitted {
                Ok(settings) => {
                    background_gpu.use_the_settings_from_the_next_batch(settings);
                    let participating: Vec<String> = background_gpu
                        .participating
                        .positions_on_this_host()
                        .iter()
                        .map(|position| {
                            background_gpu.cards_on_this_host[*position]
                                .discrete_gpu
                                .pci_bus_address
                                .clone()
                        })
                        .collect();
                    ("admitted".to_string(), participating)
                }
                Err(refused) => (
                    format!("refused:{}", attack_probe_r3_first_chars(&format!("{refused:?}"), 70)),
                    Vec::new(),
                ),
            };
            println!(
                "ATTACK name=switch_after_a_participating_card_dropped shape={shape} outcome={outcome} participating={participating:?} remaining_present={}",
                present
                    .lock()
                    .expect("探针没在别处 panic 过")
                    .contains(&remaining)
            );
        }
    }
}
