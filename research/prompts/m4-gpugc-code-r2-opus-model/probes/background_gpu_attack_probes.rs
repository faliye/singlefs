
    // ── m4-gpugc-code-r2 攻方腿的探针（接在 background_gpu.rs 测试模块末尾，替身卡与帮手照用；只打 ATTACK 行，不断言） ──

    /// T1 第 8 条：重交之前先等前一次超时的那次提交，那一等又超时时，记账里的「重交了几次」照样加一，而这张卡上一次都没重交。
    #[test]
    fn attack_probe_resubmissions_counted_while_the_wait_before_resubmitting_times_out() {
        for earlier_waits_timing_out in [2_usize, 3] {
            let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
            card.submissions_timing_out = VecDeque::from([true]);
            card.earlier_submission_waits_timing_out =
                VecDeque::from(vec![true; earlier_waits_timing_out]);
            let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
                vec![card],
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
            let unit_bytes = unit_bytes_of(1, 1000);
            let units = units_to_verify(&unit_bytes);
            let returned = background_gpu.both_kernel_outcomes_of_the_batch(&units);
            let returned_as_on_the_cpu = returned == Ok(cpu_reference_outcomes_of(&units));
            let submissions = background_gpu
                .take_the_submissions_of_the_batch()
                .unit_checksum_verification;
            let submissions_made = background_gpu.cards_on_this_host[0].submissions_made;
            println!(
                "ATTACK name=resubmissions_counted_while_the_wait_times_out earlier_waits_timing_out={earlier_waits_timing_out} returned_as_on_the_cpu={returned_as_on_the_cpu} returned_error={:?} submissions_made_on_the_card={submissions_made} resubmissions_actually_made={} resubmissions_after_timeouts={} submissions_completed={}",
                returned.as_ref().err(),
                submissions_made.saturating_sub(1),
                submissions.resubmissions_after_timeouts,
                submissions.submissions_completed
            );
        }
    }

    /// T1 第 4、6 条：切换那一刻判「显存上限扣掉卡上已占的装不装得下最小的一片」时，已占的含一次等超过上限、还在跑的提交：
    /// 这张卡这一组设置里一直不用（下一次切换之前），而同一状态下不切换、照常交一批，交活之前先等它做完、放掉账，照常交回。
    #[test]
    fn attack_probe_a_switch_leaves_out_a_card_for_a_submission_still_running_past_its_wait() {
        let smallest_piece = bytes_of_the_smallest_piece_every_participating_gpu_holds();
        let memory_limit = 4 * smallest_piece;
        let running_bytes = memory_limit - smallest_piece + 1;
        let settings = BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![card_identifier("0000:01:00.0")]),
            background_gpu_memory_limit: Some(vec![GpuMemoryInBytes(memory_limit)]),
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        };
        let unit_bytes = unit_bytes_of(1, 100);
        let units = units_to_verify(&unit_bytes);
        for (with_the_running_submission, with_the_switch) in
            [(false, true), (true, true), (true, false)]
        {
            let mut card = CardComputedOnTheCpuForTests::of_the_address(
                "0000:01:00.0",
                Some(memory_limit),
                0,
            );
            if with_the_running_submission {
                card = card.with_a_submission_running_past_its_wait(running_bytes);
            }
            let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
                vec![card],
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
            let mut admitted_ok = true;
            if with_the_switch {
                match admit_gpu_settings_against_the_inventory(&settings, &background_gpu) {
                    Ok(admitted) => background_gpu.use_the_settings_from_the_next_batch(&admitted),
                    Err(_refused) => admitted_ok = false,
                }
            }
            let left_out_after_the_switch = background_gpu.gpus_left_out_under_their_memory_limit().len();
            let batch_not_split = background_gpu.batch_not_split_over_the_gpus().is_some();
            let returned = background_gpu.both_kernel_outcomes_of_the_batch(&units);
            println!(
                "ATTACK name=switch_left_out_for_a_running_submission with_the_running_submission={with_the_running_submission} with_the_switch={with_the_switch} admitted_ok={admitted_ok} smallest_piece={smallest_piece} memory_limit={memory_limit} running_bytes={running_bytes} left_out={left_out_after_the_switch} batch_not_split={batch_not_split} batch_ok={} batch_error={:?}",
                returned.is_ok(),
                returned.as_ref().err()
            );
        }
    }

    /// 切换那一刻现查的卡清单：第一次枚举交回全部、之后的枚举交回一张都没有（等 wgpu 超过上限当一张都不在，或两次之间掉了卡）。
    struct DiscreteGpusPresentOnTheFirstEnumerationOnly {
        enumerations: std::sync::atomic::AtomicUsize,
        present_on_the_first_enumeration: Vec<GpuDeviceIdentifier>,
    }

    impl DiscreteGpusPresentNow for DiscreteGpusPresentOnTheFirstEnumerationOnly {
        fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
            if self
                .enumerations
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                == 0
            {
                self.present_on_the_first_enumeration.clone()
            } else {
                Vec::new()
            }
        }
    }

    /// T4：核心层的切换校验读卡清单的两样（显存总量、挂载那一刻没开成的）各自现查一次枚举；两次答得不一样时，点名一张挂载那一刻没开成的卡的切换照收，
    /// 交给实现那一刻在 `ParticipatingGpusOnThisHost::of_the_settings` 的 expect 上 panic。对照：两次答得一样时照第 4 条拒。
    #[test]
    fn attack_probe_a_switch_naming_a_card_not_opened_at_the_mount_passes_when_the_second_enumeration_comes_back_empty() {
        let opened_card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        let opened = opened_card.discrete_gpu.clone();
        let not_opened = DiscreteGpuNotOpenedAtTheMount {
            discrete_gpu: DiscreteGpuOnThisHost {
                position_in_the_enumeration: 1,
                identifier: card_identifier("0000:41:00.0"),
                pci_bus_address: "0000:41:00.0".to_string(),
                name: "cpu card that did not open at the mount".to_string(),
                total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
            },
            cause: GpuFailureCause::DeviceOrPipelineDidNotStart {
                reason: "替身：挂载那一刻没开成".to_string(),
            },
        };
        let settings = BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![card_identifier("0000:41:00.0")]),
            background_gpu_memory_limit: None,
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        };
        let both_present = vec![card_identifier("0000:01:00.0"), card_identifier("0000:41:00.0")];
        drop(opened_card);
        for second_enumeration_empty in [false, true] {
            let present: Box<dyn DiscreteGpusPresentNow + Send> = if second_enumeration_empty {
                Box::new(DiscreteGpusPresentOnTheFirstEnumerationOnly {
                    enumerations: std::sync::atomic::AtomicUsize::new(0),
                    present_on_the_first_enumeration: both_present.clone(),
                })
            } else {
                Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(
                    both_present.clone(),
                ))))
            };
            let mut background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
                vec![CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)],
                DiscreteGpusAfterTheOpening {
                    opened: vec![opened.clone()],
                    not_opened: vec![not_opened.clone()],
                },
                Vec::new(),
                ParticipatingGpusOnThisHost::of_the_positions(vec![0]),
                Vec::new(),
                GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
                present,
            );
            let admitted = admit_gpu_settings_against_the_inventory(&settings, &background_gpu);
            let outcome = match &admitted {
                Err(refused) => format!("refused:{refused:?}"),
                Ok(admitted_settings) => {
                    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        background_gpu.use_the_settings_from_the_next_batch(admitted_settings);
                    }));
                    match caught {
                        Ok(()) => "handed_to_the_implementation_without_a_panic".to_string(),
                        Err(payload) => format!(
                            "panicked:{}",
                            payload
                                .downcast_ref::<String>()
                                .cloned()
                                .or_else(|| payload.downcast_ref::<&str>().map(|message| (*message).to_string()))
                                .unwrap_or_default()
                        ),
                    }
                }
            };
            println!(
                "ATTACK name=switch_naming_a_card_not_opened second_enumeration_empty={second_enumeration_empty} admitted={} outcome={outcome}",
                admitted.is_ok()
            );
        }
    }

    /// T1 第 6 条：同一进程里同时开着两次挂载、都在同一张卡上泄漏（各有一次等超过上限还在跑的提交），进程里那一份按「后丢的那一次那一刻占着的」盖掉前一次的，
    /// 第三次挂载只扣后丢的那一次。
    #[test]
    fn attack_probe_two_mounts_alive_at_once_overwrite_the_leak_of_the_card() {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let card = card_identifier("0000:01:00.0");
        let (first_mount, _first_present) = background_gpu_over_cards_on_the_cpu(
            vec![CardComputedOnTheCpuForTests::of_the_address(
                "0000:01:00.0",
                None,
                memory_leaked.bytes_leaked_on(&card),
            )
            .with_a_submission_running_past_its_wait(600)],
            &memory_leaked,
        );
        let (second_mount, _second_present) = background_gpu_over_cards_on_the_cpu(
            vec![CardComputedOnTheCpuForTests::of_the_address(
                "0000:01:00.0",
                None,
                memory_leaked.bytes_leaked_on(&card),
            )
            .with_a_submission_running_past_its_wait(300)],
            &memory_leaked,
        );
        let leaked_by_the_first = first_mount.gpus_leaked_when_dropped();
        let leaked_by_the_second = second_mount.gpus_leaked_when_dropped();
        drop(first_mount);
        drop(second_mount);
        let bytes_leaked_by_the_two_mounts: u64 = [&leaked_by_the_first, &leaked_by_the_second]
            .iter()
            .flat_map(|leaked| leaked.gpus_leaked().unwrap_or_default())
            .map(|leaked| leaked.bytes_held_on_the_device)
            .sum();
        println!(
            "ATTACK name=two_mounts_alive_at_once bytes_leaked_by_the_two_mounts={bytes_leaked_by_the_two_mounts} bytes_leaked_on_the_card_in_the_process={} account_of_a_third_mount_starts_at={}",
            memory_leaked.bytes_leaked_on(&card),
            CardComputedOnTheCpuForTests::of_the_address(
                "0000:01:00.0",
                Some(1 << 20),
                memory_leaked.bytes_leaked_on(&card),
            )
            .budget
            .memory_account()
            .bytes_held()
        );
    }

    /// T1 第 5 条：一张卡上一段建缓冲显存不足、压一半重切之后，从头再交它分到的那几条——这一次调用里已经做完的那一段（100 字节的单元）每压一次就再交一次。
    /// 上限 1100、空闲 500，单元 100 与 1000 字节；手算：头一轮 [100] 做完、[1000]（1060 字节）建不成，压到 530；第二轮 [100] 再做一次、468 那一片（528 字节）建不成，
    /// 压到 264；第三轮 [100] 第三次做、204 字节五片做完。这一段最后那一切是 6 段，交成 8 次。
    #[test]
    fn attack_probe_free_memory_recut_redoes_the_chunks_already_completed_on_the_card() {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(1100), 0);
        card.free_bytes = Some(500);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = vec![vec![1_u8; 100], vec![2_u8; 1000]];
        let units = units_to_verify(&unit_bytes);
        let returned = background_gpu.both_kernel_outcomes_of_the_batch(&units);
        let returned_as_on_the_cpu = returned == Ok(cpu_reference_outcomes_of(&units));
        let submissions = background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification;
        let chunks_of_the_last_cut = pieces_cut_to_fit_on_the_gpu(
            &[100, 1000],
            CHECKSUM_CHUNK_FOOTPRINT,
            ChunkBoundsOfAGpu {
                memory_limit: Some(GpuMemoryInBytes(264)),
                largest_storage_binding_in_bytes: 1 << 30,
                largest_entry_count_per_dispatch: 1 << 20,
            },
        )
        .map(|chunks| chunks.len());
        println!(
            "ATTACK name=free_memory_recut_redoes_completed_chunks returned_as_on_the_cpu={returned_as_on_the_cpu} submissions_completed={} pieces_submitted={} recuts_after_running_out_of_memory={} submissions_made_on_the_card={} chunks_of_the_cut_at_264_bytes={chunks_of_the_last_cut:?}",
            submissions.submissions_completed,
            submissions.pieces_submitted,
            submissions.recuts_after_running_out_of_memory,
            background_gpu.cards_on_this_host[0].submissions_made
        );
    }

    /// T1 第 5 条：同一次调用里一张卡（0000:01:00.0，空闲 10 字节）不用了、它那份转给另一张（0000:41:00.0，空闲 300 字节）时，接手的卡从设置给的界重新切，
    /// 把自己头一轮已经撞过的同一种建不成再撞一次。四个 100 字节的单元、不给上限、两张卡代价相同。手算：头一张压 3 次后不用，接手的卡头一轮压 1 次、
    /// 接过来的那两条再压 1 次，合计 5 次；接手的卡记着这一次调用里压到的界就是 4 次。
    #[test]
    fn attack_probe_a_card_taking_over_entries_hits_the_same_shortage_of_free_memory_again() {
        let mut card_short_of_free_memory =
            CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        card_short_of_free_memory.free_bytes = Some(10);
        let mut card_taking_over = CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0);
        card_taking_over.free_bytes = Some(300);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card_short_of_free_memory, card_taking_over],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(4, 100);
        let units = units_to_verify(&unit_bytes);
        let returned = background_gpu.both_kernel_outcomes_of_the_batch(&units);
        let returned_as_on_the_cpu = returned == Ok(cpu_reference_outcomes_of(&units));
        let submissions = background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification;
        println!(
            "ATTACK name=card_taking_over_hits_the_same_shortage_again returned_as_on_the_cpu={returned_as_on_the_cpu} recuts_after_running_out_of_memory={} submissions_completed={} submissions_made_on_the_card_short={} submissions_made_on_the_card_taking_over={} devices_given_work={}",
            submissions.recuts_after_running_out_of_memory,
            submissions.submissions_completed,
            background_gpu.cards_on_this_host[0].submissions_made,
            background_gpu.cards_on_this_host[1].submissions_made,
            submissions.devices_given_work.len()
        );
    }
}
