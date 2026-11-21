use super::*;

/// 一个崩溃状态的 CPU 判定，看 journal 那一遍落到 `consulted_effective_root`（读到文件；`None` 是择不到根、失败），
/// 不看 journal 那一遍择不到根、失败；oracle、池级 checker、记录核对器都不报（恢复摘要的几条用例只看落到的根与两遍的结局）。
fn judgement_whose_consulted_pass_landed_on(
    consulted_effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
) -> crate::crash::CrashStateJudgement {
    let recovery_report = |outcome, effective_root| RecoveryReport {
        outcome,
        effective_root,
        journal: singlefs_core::recovery::JournalScanReport::default(),
        mapping_fallbacks: 0,
        devices_without_a_mountable_system_configuration_slot: Vec::new(),
    };
    let failed_with_no_root = || RecoveryOutcome::Failed {
        root: None,
        failure: singlefs_core::recovery::RecoveryFailure::NoValidRoot,
    };
    crate::crash::CrashStateJudgement {
        consulted: recovery_report(
            consulted_effective_root.map_or_else(failed_with_no_root, |root| {
                RecoveryOutcome::FileRead {
                    root,
                    content: Vec::new(),
                }
            }),
            consulted_effective_root,
        ),
        ignored: recovery_report(failed_with_no_root(), None),
        root_persisted: false,
        consulted_oracle: None,
        ignored_oracle: None,
        pool_checker: Vec::new(),
        records: crate::memory_pool::RecordCheck::default(),
    }
}

#[test]
fn two_roots_of_one_instance_whose_checkpoint_txgs_differ_only_in_the_high_thirty_two_bits_give_different_recovery_summaries(
) {
    let instance = InstanceGeneration(1);
    for low_thirty_two_bits in [0u64, 1, 7, u64::from(u32::MAX)] {
        for high_thirty_two_bits in [1u64, 2, u64::from(u32::MAX)] {
            let low_root = (instance, CheckpointTxg(low_thirty_two_bits));
            let high_root = (
                instance,
                CheckpointTxg((high_thirty_two_bits << 32) | low_thirty_two_bits),
            );
            let low_summary =
                recovery_words_of(&judgement_whose_consulted_pass_landed_on(Some(low_root)));
            let high_summary =
                recovery_words_of(&judgement_whose_consulted_pass_landed_on(Some(high_root)));
            assert_ne!(
                low_summary, high_summary,
                "{low_root:?} 与 {high_root:?} 的恢复摘要相同"
            );
            assert_ne!(
                low_summary.words(),
                high_summary.words(),
                "{low_root:?} 与 {high_root:?} 的恢复摘要摊出的字相同"
            );
            for (root, summary) in [(low_root, low_summary), (high_root, high_summary)] {
                assert_eq!(
                    summary.consulted_pass_landed_root.landed_root(),
                    Some(root),
                    "{root:?} 的恢复摘要换得回完整的 checkpoint_txg"
                );
                assert_eq!(
                    RecoveryRootAndOutcomesOfState::of_the_words(summary.words()),
                    summary,
                    "{root:?} 的恢复摘要摊成字再换回来不变"
                );
            }
        }
    }
}

/// 审计复核 T01：没落到根与落到实例 `u32::MAX`、checkpoint_txg `u64::MAX` 那条根（每个字都是 `u32::MAX`，盘上读来的坏镜像取得到）
/// 换出的恢复摘要不同：落没落到根看标记那一字，不看实例代号与 checkpoint_txg 取哪一组值。
#[test]
fn landing_on_no_root_and_landing_on_the_root_whose_instance_and_checkpoint_txg_are_at_their_maximum_give_different_recovery_summaries(
) {
    let root_at_the_maximum = (InstanceGeneration(u32::MAX), CheckpointTxg(u64::MAX));
    let no_root_summary = recovery_words_of(&judgement_whose_consulted_pass_landed_on(None));
    let maximum_root_summary = recovery_words_of(&judgement_whose_consulted_pass_landed_on(Some(
        root_at_the_maximum,
    )));
    assert_eq!(
        no_root_summary.consulted_pass_landed_root,
        LandedRootWords::NOT_LANDED,
        "择不到根的那一遍落到的根是没落到"
    );
    assert_ne!(
        no_root_summary.consulted_pass_landed_root, maximum_root_summary.consulted_pass_landed_root,
        "没落到根与落到 {root_at_the_maximum:?} 的那四个字相同"
    );
    assert_ne!(
        no_root_summary.words(),
        maximum_root_summary.words(),
        "没落到根与落到 {root_at_the_maximum:?} 的恢复摘要摊出的字相同"
    );
    assert_eq!(
        maximum_root_summary
            .consulted_pass_landed_root
            .landed_root(),
        Some(root_at_the_maximum),
        "落到 {root_at_the_maximum:?} 的恢复摘要换得回那条根"
    );
}

/// 这一趟自己起的卡此刻照起设备那一刻的额度算、空着时照空卡上能用的算的额度算：一张卡
/// 此刻额度 2254 MiB（一个状态都放不下）、空卡上能用的 15849 MiB，两样分开算，每张卡此刻都带不了时是借不到（报本次未跑），不是放不下（判红）。不要卡。
/// 判别力（`crates/mutations.tsv`「崩溃放量 自己起的卡空着时照此刻的额度算」）：空着时也照此刻的额度算，这条红。
#[cfg(feature = "gpu")]
#[test]
fn a_card_started_for_this_run_holds_states_now_by_its_budget_now_and_when_empty_by_the_empty_card()
{
    let states_in_flight_for_a_budget = |memory_budget_mebibytes: u64| {
        memory_budget_mebibytes
            .saturating_sub(crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER)
            / 1_000
    };
    let capacities = states_in_flight_now_and_when_empty_of_a_card_started_for_this_run(
        2_254,
        15_849,
        &states_in_flight_for_a_budget,
    );
    assert_eq!(
        capacities,
        (0, 13),
        "此刻照此刻的额度、空着时照空卡上能用的算"
    );
    assert_eq!(
        decide_scratch_before_booking(&[capacities.0], &[capacities.1]),
        ScratchBeforeBooking::NoCardHoldsAStateNow,
        "此刻带不了、空着时带得了：借不到，不是放不下"
    );
}

/// （列的第 6 处）：借草稿区之前每张卡此刻都带不了一个状态、至少一张空着时带得了，这一步是报本次未跑
/// （GPU 用例的记号、原因写显存不够、带每张卡此刻能借的）；空着时也带不了才判红；有一张此刻带得了就去借。不要卡。
#[cfg(feature = "gpu")]
#[test]
fn with_no_card_holding_a_state_now_the_step_before_booking_is_a_not_run_line_not_red() {
    let run_cards = [CardCapacityOfThisRun {
        index: 0,
        name: "造的一张卡".to_string(),
        memory_budget_mebibytes: crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER + 700,
        states_in_flight_capacity: 0,
        states_in_flight_capacity_when_the_card_is_empty: 3,
    }];
    match scratch_step_before_booking(ScratchBeforeBooking::NoCardHoldsAStateNow, &run_cards, 900)
        {
            ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow { not_run_line } => assert!(
                not_run_line.starts_with(
                    "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=显存不够："
                ) && not_run_line.contains("900 MiB")
                    && not_run_line.contains("[700]"),
                "{not_run_line}"
            ),
            other @ (ScratchStepBeforeBooking::BookOnTheCards
            | ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty) => {
                panic!("此刻借不到：要报本次未跑，判成了 {other:?}")
            }
        }
    assert_eq!(
        scratch_step_before_booking(
            ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty,
            &run_cards,
            900
        ),
        ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty
    );
    assert_eq!(
        scratch_step_before_booking(
            ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow,
            &run_cards,
            900
        ),
        ScratchStepBeforeBooking::BookOnTheCards
    );
}

/// （列的第 5 处）：产品路径向卡池借不到时，借到的卡上起设备显存不够与此刻借不到都打本次未跑那一行、
/// 交 `Ok`；要的比每张卡空着时都大、卡上的显存读不出判红。不要卡：结局照卡池交回的造；只调不打印的那一半
/// （打印那一半会把造出来的那一行打进测试段的标准输出，门禁收成本次未跑）。
#[cfg(feature = "gpu")]
#[test]
fn a_device_short_of_gpu_memory_on_the_card_lent_is_reported_not_run_like_no_room_now() {
    use crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal;
    assert!(
        not_run_line_or_refusal_of_a_lease(
            "单元校验和对拍的缓冲",
            &CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice {
                reason: "显存不够：起设备时".to_string()
            }
        )
        .is_ok_and(|line| line.starts_with("NOT_RUN_NO_GPU_ADAPTER ")
            && line.ends_with("reason=显存不够：起设备时")),
        "起设备时显存不够：本次未跑"
    );
    assert!(
        not_run_line_or_refusal_of_a_lease(
            "单元校验和对拍的缓冲",
            &CardPoolLeaseRefusal::Booking(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes: 900,
                card_rooms_mebibytes: vec![700],
            })
        )
        .is_ok_and(|line| line.starts_with("NOT_RUN_NO_GPU_ADAPTER ")),
        "此刻借不到：本次未跑"
    );
    assert!(
        not_run_line_or_refusal_of_a_lease(
            "单元校验和对拍的缓冲",
            &CardPoolLeaseRefusal::Booking(BookingRefusal::NeedLargerThanEveryCard(
                NeedLargerThanEveryCard {
                    need_mebibytes: 40_000,
                    largest_ceiling_mebibytes: 29_550,
                }
            ))
        )
        .is_err(),
        "比每张卡空着时都大：判红"
    );
    assert!(
        not_run_line_or_refusal_of_a_lease(
            "单元校验和对拍的缓冲",
            &CardPoolLeaseRefusal::Booking(BookingRefusal::CardMemoryUnreadable {
                reason: "nvidia-smi 起不来".to_string()
            })
        )
        .is_err(),
        "读不出：判红"
    );
}

/// 给定状态引擎补的第 ③ 样：给定状态表的判器借不到显存（此刻借不到、借到的卡上起设备时显存不够）照运行条件不满足报
/// （[`GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX`] 起头、点名那张表，入口照它退 78），行里不带「本次未跑」的记号（不报本次未跑、
/// 不换成 CPU 判器的结论往下走）；要的比每张卡空着时能借的都大照旧那一句判红。不要卡：只造那一句。
/// 判别力：那一句换成「本次未跑」的记号起头（`crates/mutations.tsv` 那一行），入口就把它记成本次未跑。
#[cfg(feature = "gpu")]
#[test]
fn a_given_states_judge_that_cannot_lease_gpu_memory_is_a_run_condition_not_met_and_not_a_not_run_line(
) {
    use crate::gpu_memory_ledger::GPU_NOT_RUN_MARKER;
    use crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal;
    for refusal in [
        CardPoolLeaseRefusal::Booking(BookingRefusal::NoCardHasRoomNow {
            need_mebibytes: 900,
            card_rooms_mebibytes: vec![100, 200],
        }),
        CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice {
            reason: "显存不够：第 0 张卡上起设备时 wgpu 报了 Out of Memory".to_string(),
        },
    ] {
        let reason =
            given_states_lease_refusal_as_a_run_condition_not_met("seed=1/first-crash", &refusal);
        assert!(
            reason.starts_with(GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX)
                && reason.contains("table=seed=1/first-crash")
                && !reason.contains(GPU_NOT_RUN_MARKER),
            "借不到显存：运行条件不满足那一行、点名那张表、不带本次未跑的记号：{reason}"
        );
    }
    let larger_than_every_card = given_states_lease_refusal_as_a_run_condition_not_met(
        "seed=1/first-crash",
        &CardPoolLeaseRefusal::Booking(BookingRefusal::NeedLargerThanEveryCard(
            NeedLargerThanEveryCard {
                need_mebibytes: 90_000,
                largest_ceiling_mebibytes: 30_000,
            },
        )),
    );
    assert!(
        !larger_than_every_card.starts_with(GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX)
            && larger_than_every_card.contains("没有一张卡装得下"),
        "要的比每张卡空着时都大：照旧那一句（判红）：{larger_than_every_card}"
    );
}

/// 一块里第一个判不了的状态：恢复内核的「判不了」位或 checker 内核的「判不了」字，哪一样都算；全判得了是 `None`。
#[cfg(feature = "gpu")]
#[test]
fn the_first_state_the_gpu_could_not_judge_is_found_by_either_kernel() {
    use crate::crash_judge_gpu::{verdict_bits, POOL_CHECKER_UNDECIDABLE_WORD, VERDICT_WORDS};
    let judged = [0u32; VERDICT_WORDS];
    let mut recovery_gave_up = judged;
    recovery_gave_up[0] = 1 << verdict_bits::UNDECIDABLE;
    let mut checker_gave_up = judged;
    checker_gave_up[POOL_CHECKER_UNDECIDABLE_WORD] = 3;
    assert_eq!(first_state_the_gpu_could_not_judge(&[judged, judged]), None);
    assert_eq!(
        first_state_the_gpu_could_not_judge(&[judged, recovery_gave_up, checker_gave_up]),
        Some(1)
    );
    assert_eq!(
        first_state_the_gpu_could_not_judge(&[judged, judged, checker_gave_up]),
        Some(2)
    );
}

/// C611：GPU 判器那一处按挂钟的行：写进库几块（共几块）、几块在等前面的块、几张卡收工、开跑多久。
#[test]
fn the_gpu_judge_wall_clock_line_names_the_blocks_stored_and_waiting_and_the_cards_finished() {
    assert_eq!(
            gpu_judge_wall_clock_line(3, 10, 2, 1, std::time::Duration::from_secs(301)),
            "CRASH_AMPLIFICATION_BLOCK by=wall_clock judge=gpu blocks_stored=3/10 blocks_waiting_for_an_earlier_block=2 cards_finished=1 elapsed_seconds=301.0"
        );
}

/// C611 GPU 判器那一处：写库那条线程收卡上的话，过了一个进度间隔还没有新的话（块 1 先到、在等块 0；假时钟每读一次走过一个间隔多一秒，
/// 那张卡收工的话等到时钟读过四次才发），打一行按挂钟的：写进库几块（共几块）、几块在等前面的块、几张卡收工、开跑多久；
/// 之后收工的那张卡照样记下，没写成的块一块都不写。不要卡：卡上的线程换成一条发同样几句话的线程。
#[cfg(feature = "gpu")]
#[test]
fn the_gpu_judge_reports_a_wall_clock_line_while_it_waits_for_the_cards() {
    use crate::crash::PROGRESS_LINE_INTERVAL_IN_SECONDS;
    use std::cell::Cell;
    let one_interval_and_a_second =
        std::time::Duration::from_secs(PROGRESS_LINE_INTERVAL_IN_SECONDS + 1);
    let (card_sender, card_receiver) = std::sync::mpsc::channel::<CardMessage>();
    let (release_sender, release_receiver) = std::sync::mpsc::channel::<()>();
    card_sender
        .send(CardMessage::Block(GpuVerdictWordsOfABlock {
            pending_index: 1,
            words_of_each_state: vec![[0; crate::crash_judge_gpu::VERDICT_WORDS]],
            card_index: 0,
            card_name: "测试造的卡".to_string(),
            card_driver_version: "测试造的驱动".to_string(),
        }))
        .expect("收的那一头还在");
    let clock_reads = Cell::new(0u32);
    // 第 0 次读是开跑那一刻、第 1 次读收到块 1、第 2 次读记下收到的时刻；第 4 次读（块 1 之后过了一个间隔、按挂钟那一行到期）时
    // 才放收工的那句话进来
    let clock = || {
        let reads = clock_reads.get();
        clock_reads.set(reads + 1);
        if reads == 4 {
            // 放行的那一头只收一次，收过之后发不出去也无妨
            let _ = release_sender.send(());
        }
        one_interval_and_a_second * reads
    };
    let mut wall_clock_lines = WallClockProgressLines::every_progress_line_interval(clock());
    let mut printed: Vec<String> = Vec::new();
    let mut blocks_handed_to_the_store = 0usize;
    let mut stored_in_order: BlocksStoredInOrder<GpuVerdictWordsOfABlock> =
        BlocksStoredInOrder::new();
    let received = std::thread::scope(|scope| {
        scope.spawn(move || {
            release_receiver.recv().expect("时钟读到第 4 次就放行");
            let finished_card = GpuCardOutcome {
                name: "stand-in card".to_string(),
                index: 3,
                memory_budget_mebibytes: 0,
                states_per_dispatch: 0,
                scratch_mebibytes: 0,
                blocks_judged: 0,
                states_judged: 0,
                build_seconds: 0.0,
                judge_seconds: 0.0,
                dispatches: 0,
                seconds_preparing: 0.0,
                seconds_waiting_for_the_card: 0.0,
                seconds_reading_back: 0.0,
                seconds_finding_states_the_gpu_could_not_judge: 0.0,
                states_in_the_widest_dispatch: 0,
                seconds_of_the_longest_dispatch: 0.0,
                submission_waits_of_the_passes:
                    crate::crash_judge_dispatch::SubmissionWaitsOfThePasses::default(),
            };
            card_sender
                .send(CardMessage::Finished(Box::new(finished_card)))
                .expect("收的那一头还在等");
        });
        receive_the_cards_messages_and_store_the_blocks_in_order(
            &card_receiver,
            &mut wall_clock_lines,
            &clock,
            3,
            &mut stored_in_order,
            &mut |_block| {
                blocks_handed_to_the_store += 1;
                Ok(())
            },
            &mut |line| printed.push(line),
        )
    });
    assert_eq!(
            printed.first().map(String::as_str),
            Some(
                "CRASH_AMPLIFICATION_BLOCK by=wall_clock judge=gpu blocks_stored=0/3 blocks_waiting_for_an_earlier_block=1 cards_finished=0 elapsed_seconds=1204.0"
            ),
            "过了一个间隔还没收到新的话，按挂钟打一行：{printed:?}"
        );
    assert_eq!(
        received
            .finished_cards
            .iter()
            .map(|card| card.index)
            .collect::<Vec<u32>>(),
        vec![3],
        "收工的那张卡照样记下"
    );
    assert_eq!(
        (stored_in_order.blocks_stored(), blocks_handed_to_the_store),
        (0, 0),
        "块 1 在等块 0，一块都不写"
    );
    assert_eq!(stored_in_order.storing_error(), None);
    assert_eq!(received.refusal, None);
}

/// （攻方 R2）：卡线程上的错误范围接到 Out of Memory 就是显存不够（活成没成都一样）；没接到而活 panic 了，
/// 设备上从活开始起记了 Out of Memory（另起的线程上编内核时没接住的）也是显存不够；都没有照样抛；活没 panic 交结果。
/// 不要卡：几样输入照 [`on_the_card_catching_out_of_memory`] 收到的造。
#[cfg(feature = "gpu")]
#[test]
fn a_panic_on_the_card_after_an_out_of_memory_on_another_thread_is_put_down_to_gpu_memory() {
    let panicked =
        || -> std::thread::Result<u32> { Err(Box::new("恢复内核的编译线程不 panic")) };
    assert!(
        matches!(
            decide_the_work_on_the_card(
                panicked(),
                None,
                Some("另起的线程上没接住的 Out of Memory".to_string())
            ),
            WorkOnTheCard::OutOfMemory { ref detail } if detail == "另起的线程上没接住的 Out of Memory"
        ),
        "范围没接到、设备上记了：显存不够，不往外抛"
    );
    assert!(
        matches!(
            decide_the_work_on_the_card(panicked(), Some("Out of Memory".to_string()), None),
            WorkOnTheCard::OutOfMemory { .. }
        ),
        "范围接到了：显存不够"
    );
    assert!(
        matches!(
            decide_the_work_on_the_card(Ok(7u32), Some("Out of Memory".to_string()), None),
            WorkOnTheCard::OutOfMemory { .. }
        ),
        "活成了而范围接到 Out of Memory：缓冲没建成，判的不作数"
    );
    assert!(
        matches!(
            decide_the_work_on_the_card(panicked(), None, None),
            WorkOnTheCard::Panicked(_)
        ),
        "都没有：照样往外抛"
    );
    assert!(
        matches!(
            decide_the_work_on_the_card(Ok(7u32), None, None),
            WorkOnTheCard::Done(7)
        ),
        "活成了：交结果"
    );
}

/// 卡上 wgpu 报了 Out of Memory（[`CardMessage::OutOfMemory`]，文件头第 5 条）记成显存不够、按卡记下，不记成拒判
/// （拒判整趟判红，显存不够这一趟报本次未跑）；之前判完的块照写进库。不要卡：卡上的线程换成直接发这几句话。
#[cfg(feature = "gpu")]
#[test]
fn an_out_of_memory_on_a_card_is_kept_apart_from_a_refusal_and_the_blocks_judged_before_it_are_stored(
) {
    let (card_sender, card_receiver) = std::sync::mpsc::channel::<CardMessage>();
    card_sender
        .send(CardMessage::Block(GpuVerdictWordsOfABlock {
            pending_index: 0,
            words_of_each_state: vec![[0; crate::crash_judge_gpu::VERDICT_WORDS]],
            card_index: 0,
            card_name: "测试造的卡".to_string(),
            card_driver_version: "测试造的驱动".to_string(),
        }))
        .expect("收的那一头还在");
    for (index, reason) in [(0, "第一张卡报的那一句"), (1, "第二张卡报的那一句")]
    {
        card_sender
            .send(CardMessage::OutOfMemory {
                index,
                reason: reason.to_string(),
            })
            .expect("收的那一头还在");
    }
    drop(card_sender);
    let mut wall_clock_lines =
        WallClockProgressLines::every_progress_line_interval(std::time::Duration::ZERO);
    let mut blocks_handed_to_the_store = 0usize;
    let mut stored_in_order: BlocksStoredInOrder<GpuVerdictWordsOfABlock> =
        BlocksStoredInOrder::new();
    let received = receive_the_cards_messages_and_store_the_blocks_in_order(
        &card_receiver,
        &mut wall_clock_lines,
        &|| std::time::Duration::ZERO,
        2,
        &mut stored_in_order,
        &mut |_block| {
            blocks_handed_to_the_store += 1;
            Ok(())
        },
        &mut |_line| {},
    );
    assert_eq!(
        received.cards_out_of_memory,
        vec![
            (0, "第一张卡报的那一句".to_string()),
            (1, "第二张卡报的那一句".to_string())
        ],
        "报了 Out of Memory 的卡按收到的次序都记下"
    );
    assert_eq!(
        received.refusal, None,
        "Out of Memory 不记成拒判：拒判整趟判红，显存不够报本次未跑"
    );
    assert_eq!(
        (stored_in_order.blocks_stored(), blocks_handed_to_the_store),
        (1, 1),
        "Out of Memory 之前判完的块 0 照写进库"
    );
}

/// 场景（结论字挪出卡线程）：三块假的结论字（`verdict_words_of_a_made_up_block`，种子定下），照块 2、块 0、块 1 的次序交给写库那条线程，
/// 写库那一头照块序先解（[`decoded_gpu_block`]）再交写库；池里三块盘。
/// 预期：写库那一头按块序收到三块，每块与在这条线程上逐状态现解（红项、恢复字、逐盘自证槽数、不变量计数四样，各调判器一侧的逐状态函数，
/// 照判器一侧逐状态函数的次序）逐字相同；每块头一个状态红项是空的、第二个恰好一项（绝对值另钉，四样一起错也红）。
/// 判别力：`crates/mutations.tsv` 里「崩溃放量 GPU 结论字在写库线程上解时违例位取成判到位」那一行让这一条红在计数上。
#[cfg(feature = "gpu")]
#[test]
fn verdict_words_decoded_on_the_storing_thread_equal_decoding_each_state_on_the_card_thread() {
    use crate::crash_judge_gpu::{
        gpu_verdict_items, pool_checker_judged_bits, pool_checker_violated_bits,
        recovery_words_of_the_gpu_verdict,
        self_verified_system_configuration_slots_of_each_device_of_the_gpu_verdict,
    };
    let devices_of_the_pool = [1u32, 2, 3];
    let mut seed = 0x5EED_0FDE_C0DE_u64;
    let words_of_each_block: Vec<Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]>> = [5, 3, 7]
        .iter()
        .map(|states| verdict_words_of_a_made_up_block(&mut seed, *states))
        .collect();
    let decoded_here: Vec<GpuJudgedBlock> = words_of_each_block
        .iter()
        .enumerate()
        .map(|(pending_index, words)| {
            let mut block = GpuJudgedBlock {
                pending_index,
                items_of_each_state: Vec::new(),
                recovery_words_of_each_state: Vec::new(),
                state_counts: StateCountsOfABlock::default(),
                self_verified_system_configuration_slots_of_the_states:
                    SelfVerifiedSystemConfigurationSlotsOfTheStates::default(),
                card_name: "测试造的卡".to_string(),
                card_driver_version: "测试造的驱动".to_string(),
            };
            for state_words in words {
                block
                    .items_of_each_state
                    .push(gpu_verdict_items(state_words).expect("判不了的位清过"));
                block
                    .recovery_words_of_each_state
                    .push(recovery_words_of_the_gpu_verdict(state_words));
                block
                    .self_verified_system_configuration_slots_of_the_states
                    .push_a_state(
                        self_verified_system_configuration_slots_of_each_device_of_the_gpu_verdict(
                            state_words,
                            &devices_of_the_pool,
                        ),
                    );
                block.state_counts.count_the_gpu_bits(
                    pool_checker_judged_bits(state_words),
                    pool_checker_violated_bits(state_words),
                );
            }
            block
        })
        .collect();
    let (card_sender, card_receiver) = std::sync::mpsc::channel::<CardMessage>();
    for pending_index in [2usize, 0, 1] {
        card_sender
            .send(CardMessage::Block(GpuVerdictWordsOfABlock {
                pending_index,
                words_of_each_state: words_of_each_block[pending_index].clone(),
                card_index: 0,
                card_name: "测试造的卡".to_string(),
                card_driver_version: "测试造的驱动".to_string(),
            }))
            .expect("收的那一头还在");
    }
    drop(card_sender);
    let mut wall_clock_lines =
        WallClockProgressLines::every_progress_line_interval(std::time::Duration::ZERO);
    let mut stored_in_order: BlocksStoredInOrder<GpuVerdictWordsOfABlock> =
        BlocksStoredInOrder::new();
    let mut decoded_on_the_storing_thread: Vec<GpuJudgedBlock> = Vec::new();
    let received = receive_the_cards_messages_and_store_the_blocks_in_order(
        &card_receiver,
        &mut wall_clock_lines,
        &|| std::time::Duration::ZERO,
        3,
        &mut stored_in_order,
        &mut |words_of_the_block| {
            decoded_on_the_storing_thread
                .push(decoded_gpu_block(words_of_the_block, &devices_of_the_pool));
            Ok(())
        },
        &mut |_line| {},
    );
    assert_eq!(received.refusal, None);
    assert_eq!(stored_in_order.blocks_stored(), 3, "三块都照块序写了");
    assert_eq!(
        decoded_on_the_storing_thread, decoded_here,
        "写库那条线程按块序解出的三块与逐状态现解的逐字相同"
    );
    for block in &decoded_on_the_storing_thread {
        assert_eq!(
            (
                block.items_of_each_state[0].len(),
                block.items_of_each_state[1].len()
            ),
            (0, 1),
            "块 {} 头一个状态全零是绿的，第二个只置一类 oracle 恰好一项红",
            block.pending_index
        );
    }
}

/// 判完这一趟打的每趟用时汇总行：用到的每张卡每一趟一行，卡按收工的先后、同一张卡收工两次只打一份；一张卡都没用到打一行提交次数 0 的。
#[cfg(feature = "gpu")]
#[test]
fn gpu_pass_lines_print_every_pass_of_every_card_once_and_one_line_when_no_card_dispatched() {
    use crate::crash_judge_dispatch::{
        GpuJudgePass, SubmissionWaitsOfOneDispatch, SubmissionWaitsOfThePasses, GPU_JUDGE_PASSES,
        GPU_PASS_LINE_PREFIX,
    };
    let mut readback_only = SubmissionWaitsOfOneDispatch::default();
    readback_only.note_a_submission(GpuJudgePass::Readback, 2_000_000_000);
    let mut waits_of_a_round = SubmissionWaitsOfThePasses::default();
    waits_of_a_round.add_a_dispatch(3, &readback_only);
    let card_finishing = |name: &str, index: u32| GpuCardOutcome {
        name: name.to_string(),
        index,
        memory_budget_mebibytes: 0,
        states_per_dispatch: 3,
        scratch_mebibytes: 0,
        blocks_judged: 1,
        states_judged: 3,
        build_seconds: 0.0,
        judge_seconds: 2.0,
        dispatches: 1,
        seconds_preparing: 0.0,
        seconds_waiting_for_the_card: 2.0,
        seconds_reading_back: 0.0,
        seconds_finding_states_the_gpu_could_not_judge: 0.0,
        states_in_the_widest_dispatch: 3,
        seconds_of_the_longest_dispatch: 2.0,
        submission_waits_of_the_passes: waits_of_a_round.clone(),
    };
    let lines = gpu_pass_lines_of_the_cards(
        "fixed-script-stream",
        &[
            card_finishing("card at index two", 2),
            card_finishing("card at index five", 5),
            card_finishing("card at index two", 2),
        ],
    );
    assert_eq!(
        lines.len(),
        2 * GPU_JUDGE_PASSES,
        "两张卡各十二行：{lines:#?}"
    );
    assert!(
        lines
            .iter()
            .all(|line| line
                .starts_with(&format!("{GPU_PASS_LINE_PREFIX} flow=fixed-script-stream "))),
        "{lines:#?}"
    );
    assert_eq!(
            lines[GPU_JUDGE_PASSES - 1],
            "CRASH_AMPLIFICATION_GPU_PASS flow=fixed-script-stream card=\"card at index two\" index=2 pass=readback submissions=2 \
             seconds_waiting_for_the_card=4.000 seconds_of_the_longest_submission=2.000 median_states_per_submission=3 share_of_the_waits_of_the_flow=0.6667",
            "收工两次的那张卡拷回那一趟并成两次、4 秒，占这条流 6 秒里的三分之二"
        );
    assert_eq!(
            gpu_pass_lines_of_the_cards("fixed-script-stream", &[]),
            vec![
                "CRASH_AMPLIFICATION_GPU_PASS flow=fixed-script-stream card=none index=none pass=none submissions=0 seconds_waiting_for_the_card=0.000 \
                 seconds_of_the_longest_submission=0.000 median_states_per_submission=0 share_of_the_waits_of_the_flow=0.0000"
                    .to_string()
            ],
            "一张卡都没派活的流打一行"
        );
}
