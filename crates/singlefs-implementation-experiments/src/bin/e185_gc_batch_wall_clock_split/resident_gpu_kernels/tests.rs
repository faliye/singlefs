use super::*;

/// 先录后判：只录那一段造 64 个槽、算 CPU 参照、写进库，不取卡（打 `RECORDED_ONLY` 收工）；判段读回之后才取卡，两核逐条比读回的参照，核一与 crates 的 GPU CRC 比。
#[cfg(feature = "gpu")]
#[test]
fn resident_kernels_agree_with_the_cpu_and_kernel_one_agrees_with_the_crates_on_seeded_units() {
    use singlefs_verification::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run;
    let (seeded_slot_bytes, seeded_lengths) = seeded_slots(64);
    let Some((read_back, card_or_not_run)) =
        cards_and_pairing_of_the_gpu_segments::slots_read_back_before_the_card(
            &format!("e185_gc_batch_wall_clock_split/{SEEDED_UNITS_TEST_NAME}"),
            &seeded_slot_bytes,
            &seeded_lengths,
            cards_and_pairing_of_the_gpu_segments::input_and_reference_through_the_pairing_library,
            || first_usable_card_or_report_not_run(SEEDED_UNITS_TEST_NAME),
        )
    else {
        println!("RECORDED_ONLY test={SEEDED_UNITS_TEST_NAME}");
        return;
    };
    let Some(card) = card_or_not_run else {
        return;
    };
    let kernels = resident_gpu_kernels::ResidentKernels::start(&card, 64);
    let cards_and_pairing_of_the_gpu_segments::SlotsReadBack {
        slots,
        lengths,
        cpu_crcs: crcs,
    } = read_back;
    for form in InputForm::ALL {
        let (results, are_the_sequence_numbers_holding) = resident_gpu_kernels::untimed_results(
            &kernels,
            form,
            &slots,
            &lengths,
            &[],
            KernelsToDispatch::Both,
        );
        assert!(are_the_sequence_numbers_holding, "{form:?}");
        let expected: Vec<KernelResultsOfAUnit> =
            crcs.iter().map(|crc| (Some(*crc), Some(*crc))).collect();
        assert_eq!(results, expected, "{form:?}");
    }
    let (kernel_one_only, _sequence) = resident_gpu_kernels::untimed_results(
        &kernels,
        InputForm::Shared,
        &slots,
        &lengths,
        &[],
        KernelsToDispatch::KernelOneOnly,
    );
    let kernel_one: Vec<u32> = kernel_one_only.iter().filter_map(|(crc, _)| *crc).collect();
    assert!(kernel_one_only
        .iter()
        .all(|(_, kernel_two)| kernel_two.is_none()));
    let slot_bytes = usize::try_from(LOCAL_GPU_SLOT_BYTES).expect("32768");
    let ranges: Vec<&[u8]> = slots
        .chunks(slot_bytes)
        .zip(&lengths)
        .map(|(slot, length)| &slot[..usize::try_from(*length).expect("长度")])
        .collect();
    assert_eq!(
        kernel_one_finding_against_the_crates(64, &kernel_one, &card.crc32c_of_ranges(&ranges)),
        None
    );
}

/// 第九节 V99：同一个设备上先交这一次（n = 8），再交一次长的活（核一 64 遍 × 16384 个单元）；阻塞等只等这一次，比长活单独跑的一半还快。
#[cfg(feature = "gpu")]
#[test]
fn blocking_wait_waits_only_for_its_own_submission() {
    use singlefs_verification::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run;
    const LONG_JOB_PASSES: u32 = 64;
    let Some(card) =
        first_usable_card_or_report_not_run("blocking_wait_waits_only_for_its_own_submission")
    else {
        return;
    };
    let kernels = resident_gpu_kernels::ResidentKernels::start(&card, UNITS_PER_DISAGREEMENT_BATCH);
    let slot_bytes = usize::try_from(LOCAL_GPU_SLOT_BYTES).expect("32768");
    let units = usize::try_from(UNITS_PER_DISAGREEMENT_BATCH).expect("16384");
    let long_bytes = vec![0x5A_u8; units * slot_bytes];
    let lengths = vec![u32::try_from(slot_bytes).expect("32768"); units];
    let _long_job_bytes_written =
        kernels.write_the_inputs(InputForm::Shared, &long_bytes, &lengths, &[]);
    let long_job_started = Instant::now();
    let alone = kernels.submit_a_long_job(UNITS_PER_DISAGREEMENT_BATCH, LONG_JOB_PASSES);
    kernels.block_until_done(alone);
    let long_job_seconds = long_job_started.elapsed().as_secs_f64();
    let short_units = 8;
    let expected = vec![crc32_castagnoli(&long_bytes[..slot_bytes]); short_units];
    let _short_bytes_written = kernels.write_the_inputs(
        InputForm::Shared,
        &long_bytes[..short_units * slot_bytes],
        &lengths[..short_units],
        &[],
    );
    let short = kernels.encode_and_submit(InputForm::Shared, 8, KernelsToDispatch::Both);
    let pending = kernels.register_the_readback(short, 8);
    let _second_long_job_bytes_written =
        kernels.write_the_inputs(InputForm::Shared, &long_bytes, &lengths, &[]);
    let long_again = kernels.submit_a_long_job(UNITS_PER_DISAGREEMENT_BATCH, LONG_JOB_PASSES);
    let wait_started = Instant::now();
    let _polls = kernels.wait(&pending, WaitMethod::Blocking);
    let short_wait_seconds = wait_started.elapsed().as_secs_f64();
    let comparison = kernels.read_then_unmap(&pending, |readback| {
        compare_the_readback(readback, &expected, None)
    });
    kernels.block_until_done(long_again);
    assert_eq!(comparison.confirmed, 8);
    assert_eq!(comparison.kernel_one_differs_from_the_expected, 0);
    assert!(
        short_wait_seconds < long_job_seconds / 2.0,
        "阻塞等等了 {short_wait_seconds} 秒，长活单独 {long_job_seconds} 秒：等的不只是这一次"
    );
}
