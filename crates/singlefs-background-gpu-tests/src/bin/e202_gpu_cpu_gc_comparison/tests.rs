use super::*;

/// 同上一条，GPU 点名一张卡臂换成真卡上产品那一份：五遍每段的活槽数都等于装置自己数的。先录后判：只录那一段在 CPU 上造小池、
/// 取分配记录与单元区、装置自数参照、写进判定库，不取卡；判段读回之后才取卡、只跑 GPU、逐段比，对拍结论按键入库复用（库里有两边一致的不取卡）。
/// 持卡判入口 [`judged_on_the_cards_that_the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool`]。
/// 对拍经全仓那一份库，包在 [`with_the_one_library_kept_for_this_process`] 里：同一个二进制里设一次性库的用例等它对拍完。
#[cfg(feature = "tests-on-a-real-gpu")]
#[test]
fn the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool(
) {
    with_the_one_library_kept_for_this_process(|| {
        crate::common_pairing_input_through_the_library::pairing_concluded_on_the_cards_through_the_pairing_library(
            "the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool",
            REFERENCE_UNITS_GROUP_OF_THE_APPARATUS_COUNT,
            || {
                let (allocation_records, unit_area) =
                    allocation_records_and_unit_area_of_the_image(&small_pool_image())
                        .expect("量挂载成");
                pairing_input_bytes_of_the_allocation_records_and_the_unit_area(
                    &allocation_records,
                    unit_area,
                )
            },
            reference_bytes_of_the_apparatus_count_of_the_live_slots_per_segment,
            judged_on_the_cards_that_the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool,
        );
    });
}

/// `lockstep` 子命令的必填参数，后面接 `extra_words`。
fn lockstep_command_of_the_words(extra_words: &[&str]) -> Result<CommandOfTheApparatus, String> {
    let mut words = vec![
        "lockstep",
        "--reading",
        "occupancy",
        "--unit-area",
        "4GiB",
        "--product",
        "lockstep.out",
    ];
    words.extend_from_slice(extra_words);
    command_of_the_arguments(&arguments_of(&words))
}

/// 场景：`lockstep --cards 2`。
/// 预期：认成只开交来的卡里排在前头的两张。
#[test]
fn lockstep_with_the_cards_option_two_names_the_first_two_handed_over_cards() {
    match lockstep_command_of_the_words(&["--cards", "2"]) {
        Ok(CommandOfTheApparatus::Lockstep(command)) => assert_eq!(
            command.cards_of_the_every_card_arm,
            CardsOfTheEveryCardArm::FirstHandedOverCards(NonZeroUsize::new(2).expect("2 不是 0"))
        ),
        Ok(_) | Err(_) => panic!("lockstep --cards 2 认得出"),
    }
}

/// 场景：`lockstep` 不给 `--cards`。
/// 预期：认成开服务交来的全部。
#[test]
fn lockstep_without_the_cards_option_opens_every_handed_over_card() {
    match lockstep_command_of_the_words(&[]) {
        Ok(CommandOfTheApparatus::Lockstep(command)) => assert_eq!(
            command.cards_of_the_every_card_arm,
            CardsOfTheEveryCardArm::EveryHandedOverCard
        ),
        Ok(_) | Err(_) => panic!("lockstep 不给 --cards 认得出"),
    }
}

/// 场景：`lockstep --cards 0`。
/// 预期：命令行不对，与 `--cards 1` 同一句（全部卡臂至少开两张），不当成没给。
#[test]
fn lockstep_with_the_cards_option_zero_is_a_usage_error() {
    assert_eq!(
        lockstep_command_of_the_words(&["--cards", "0"]).err(),
        Some("--cards 0：全部卡臂至少开 2 张（只开一张时它与点名一张卡臂是同一条臂）".to_string())
    );
}

/// 场景：`lockstep --cards 1`。
/// 预期：命令行不对（只开一张时全部卡臂与点名一张卡臂是同一条臂），不认成只开一张、报本次未跑。
#[test]
fn lockstep_with_the_cards_option_one_is_a_usage_error() {
    assert_eq!(
        lockstep_command_of_the_words(&["--cards", "1"]).err(),
        Some("--cards 1：全部卡臂至少开 2 张（只开一张时它与点名一张卡臂是同一条臂）".to_string())
    );
}

/// 场景：`lockstep --cards 两张`。
/// 预期：命令行不对（不是数）。
#[test]
fn lockstep_with_the_cards_option_not_a_number_is_a_usage_error() {
    assert!(lockstep_command_of_the_words(&["--cards", "两张"])
        .err()
        .is_some_and(|reason| reason.starts_with("--cards 两张 不是数")));
}

/// 场景：`lockstep --cards 2 --arms cpu,gpu_one_card`（没点全部卡臂）；另一趟 `--arms gpu_every_card_rejudged`。
/// 预期：没点全部卡臂的命令行不对；点了全部卡多判臂的认得出。
#[test]
fn lockstep_with_the_cards_option_but_without_an_every_card_arm_is_a_usage_error() {
    assert_eq!(
        lockstep_command_of_the_words(&["--cards", "2", "--arms", "cpu,gpu_one_card"]).err(),
        Some("--cards 只管全部卡臂开几张，--arms 里没点全部卡臂".to_string())
    );
    assert!(
        lockstep_command_of_the_words(&["--cards", "2", "--arms", "gpu_every_card_rejudged"])
            .is_ok()
    );
}

/// 场景：没有全部卡臂的子命令 `concurrent` 给 `--cards 2`。
/// 预期：命令行不对（`--cards` 认不得）。
#[test]
fn a_subcommand_without_the_every_card_arm_refuses_the_cards_option() {
    assert_eq!(
        command_of_the_arguments(&arguments_of(&[
            "concurrent",
            "--reading",
            "occupancy",
            "--unit-area",
            "4GiB",
            "--cards",
            "2",
            "--product",
            "concurrent.out",
        ]))
        .err(),
        Some("--cards 认不得".to_string())
    );
}

/// 一条分配记录（代照写 5）。
fn allocation_record_of_the_pairing(
    device: u32,
    slot: u64,
    span_slots: u16,
    is_released: bool,
    quarantine_mark: AllocationRecordQuarantineMark,
) -> AllocationRecord {
    AllocationRecord {
        device: DeviceIdentity(device),
        slot: SlotNumber(slot),
        span_slots,
        generation: CheckpointTxg(5),
        is_released,
        quarantine_mark,
    }
}

/// 场景：默认环长下单元区（三个整段加末尾 5 个槽）上的几条记录（段里的、跨两段的、释放了的、别的盘上的、隔离了的），不造池，
/// 在一次性的库上（验证包 `DisposableLibrary`，它自己起的临时 prune-server）先只录、再只判；持卡判换成只数自己被调了几次、解读回的参照。
/// 预期：只录那一趟交回只录、持卡判一次都没调（不取卡）；只判那一趟持卡判被调一次，读回的参照就是装置在只录那一段自数的每段活槽数
/// （隔离了的仍分配、照数，释放了的与别的盘上的不数）。
#[test]
fn the_record_stage_of_the_live_slot_pairing_takes_no_card_and_the_judge_stage_reads_back_the_apparatus_count(
) {
    use crate::common_pairing_input_through_the_library::{
        pairing_concluded_on_the_cards_in_the_library, KeysOfAPairingOnTheCards,
        PairingOnTheCardsOutcome,
    };
    use singlefs_verification::crash_identity::JudgingUnitsKey;
    use singlefs_verification::crash_library::{
        recording_key_of_the_pairing_input_and_the_reference_units, CrashLibrary,
        CrashLibraryClass, DisposableLibrary, RunStage,
    };
    use singlefs_verification::sha256::DIGEST_BYTES;
    use singlefs_verification::verdict_store::PairingAgreement;

    let test_name = "the_record_stage_of_the_live_slot_pairing_takes_no_card_and_the_judge_stage_reads_back_the_apparatus_count";
    let unit_area = UnitAreaOfOneDevice {
        device: DeviceIdentity(0),
        start: UnitAreaStart::of_the_default_journal_ring(),
        slots: 3 * LOCAL_CLUSTER_SEGMENT_SLOTS + 5,
    };
    let start = unit_area.start.slot().0;
    let allocation_records = vec![
        allocation_record_of_the_pairing(
            0,
            start + 2,
            4,
            false,
            AllocationRecordQuarantineMark::NotQuarantined,
        ),
        allocation_record_of_the_pairing(
            0,
            start + 62,
            4,
            false,
            AllocationRecordQuarantineMark::NotQuarantined,
        ),
        allocation_record_of_the_pairing(
            0,
            start + 70,
            2,
            true,
            AllocationRecordQuarantineMark::NotQuarantined,
        ),
        allocation_record_of_the_pairing(
            1,
            start + 10,
            8,
            false,
            AllocationRecordQuarantineMark::NotQuarantined,
        ),
        allocation_record_of_the_pairing(
            0,
            start + 130,
            2,
            false,
            AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
        ),
    ];
    let input_bytes = pairing_input_bytes_of_the_allocation_records_and_the_unit_area(
        &allocation_records,
        unit_area,
    );
    assert_eq!(
        allocation_records_and_the_unit_area_of_the_pairing_input_bytes(&input_bytes),
        (allocation_records.clone(), unit_area),
        "对拍输入的字节读回原样"
    );
    let by_the_apparatus =
        live_slots_of_each_segment_counted_by_the_apparatus(&allocation_records, unit_area);
    assert_eq!(by_the_apparatus, vec![6, 2, 2]);
    // 一次性库留着：要从空库数持卡判被调了几次
    let disposable = DisposableLibrary::new("e202-live-slot-pairing-record-stage")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::InputPairing).expect("开得了一次性库");
    let key_filled_with = |fingerprint_byte: u8| -> Result<JudgingUnitsKey, String> {
        Ok(JudgingUnitsKey::of_the_parts(
            [fingerprint_byte; DIGEST_BYTES],
            Vec::new(),
        ))
    };
    let reference_units = key_filled_with(0x31);
    let keys = KeysOfAPairingOnTheCards {
        recording: recording_key_of_the_pairing_input_and_the_reference_units(
            "e202-gpu-cpu-gc-comparison",
            &input_bytes,
            &reference_units,
        ),
        reference_units,
        gpu_side_units: key_filled_with(0x32),
    };
    let cards_taken = Cell::new(0_u32);
    let reference_read_back: RefCell<Option<Vec<u64>>> = RefCell::new(None);
    let judge_counting_the_cards_taken = |_input_read_back: &[u8], reference_bytes: &[u8]| {
        cards_taken.set(cards_taken.get() + 1);
        *reference_read_back.borrow_mut() = Some(live_slots_per_segment_of_the_reference_bytes(
            reference_bytes,
        ));
        PairingJudged {
            conclusion: format!("{test_name}：假的持卡判").into_bytes(),
            agreement: PairingAgreement::BothSidesAgree,
        }
    };
    let recorded = pairing_concluded_on_the_cards_in_the_library(
        &library,
        RunStage::RecordOnly,
        test_name,
        &keys,
        &input_bytes,
        reference_bytes_of_the_apparatus_count_of_the_live_slots_per_segment,
        judge_counting_the_cards_taken,
    );
    let cards_taken_in_the_record_stage = cards_taken.get();
    let judged = pairing_concluded_on_the_cards_in_the_library(
        &library,
        RunStage::JudgeOnly,
        test_name,
        &keys,
        &input_bytes,
        |_input_bytes: &[u8]| -> Vec<u8> { panic!("只判那一段不算参照") },
        judge_counting_the_cards_taken,
    );
    assert_eq!(
        (recorded, cards_taken_in_the_record_stage),
        (PairingOnTheCardsOutcome::RecordedOnly, 0),
        "只录那一趟不调持卡判（不取卡）"
    );
    assert_eq!(
        (judged, cards_taken.get()),
        (PairingOnTheCardsOutcome::JudgedNowAndBothSidesAgree, 1),
        "只判那一趟读回之后调一次持卡判"
    );
    assert_eq!(reference_read_back.into_inner(), Some(by_the_apparatus));
}

/// 场景：对拍输入里「释放没有」那一格读回 2（录的与读的布局对不上）。
/// 预期：当场停下，那一句点名是哪一格、读回的是几。
#[test]
#[should_panic(expected = "释放没有只写 0 或 1，读回 2")]
fn a_number_written_other_than_zero_or_one_is_refused_as_a_layout_mismatch() {
    whether_of_the_number_written(2, "释放没有");
}

/// [`a_disposable_library_is_set_only_after_the_pairing_through_the_one_library_returns`] 里那一段在对拍里停多久：
/// 设一次性库那一条线程在这么久里早就设上了（没有守卫的话），守卫在时它等满这么久。
const TIME_THE_TEST_STAYS_INSIDE_THE_PAIRING: Duration = Duration::from_millis(200);

/// 真卡对拍用例经全仓那一份库对拍时包在外面（`the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool`）：
/// 先拿 `TheOneLibraryKeptForThisProcess`，到 `pairing` 返回为止，同一个二进制里设一次性库的用例
/// （`the_record_stage_of_the_live_slot_pairing_takes_no_card_and_the_judge_stage_reads_back_the_apparatus_count`）等它放手。
/// 不拿的，那条用例设着一次性库的那一刻开库就开到那一份：它收尾停掉临时服务，这一边接着读写连不上，只录那一段录进去的输入跟着丢。
fn with_the_one_library_kept_for_this_process<Outcome>(
    pairing: impl FnOnce() -> Outcome,
) -> Outcome {
    let _the_one_library = singlefs_verification::crash_library::TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    pairing()
}

/// 场景：一条线程进了 [`with_the_one_library_kept_for_this_process`]（真卡对拍用例经全仓那一份库对拍的那一段），在里面停一会儿；
/// 它进去之后，另一条线程去设一次性库（同一个二进制里先录后判那条用例做的事）。
/// 预期：一次性库等那一段返回之后才设得上（设上的那一刻那一段已经返回）：同时跑的两条用例，真卡那一条开库开不到一次性库。
/// 两条线程是这个场景本身（并发下的守卫），不拆。
#[test]
fn a_disposable_library_is_set_only_after_the_pairing_through_the_one_library_returns() {
    let disposable = singlefs_verification::crash_library::DisposableLibrary::new(
        "e202-one-library-kept-during-the-pairing",
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let (entered_sender, entered_receiver) = mpsc::channel::<()>();
    let has_the_pairing_returned = AtomicBool::new(false);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            with_the_one_library_kept_for_this_process(|| {
                entered_sender
                    .send(())
                    .expect("设一次性库的那一条线程在等这一声");
                std::thread::sleep(TIME_THE_TEST_STAYS_INSIDE_THE_PAIRING);
                has_the_pairing_returned.store(true, Ordering::SeqCst);
            });
        });
        entered_receiver
            .recv()
            .expect("对拍那一段的线程进去之后才发这一声");
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        assert!(
            has_the_pairing_returned.load(Ordering::SeqCst),
            "一次性库在对拍那一段还没返回时就设上了：那一段里开的库会开到这一份"
        );
    });
}
