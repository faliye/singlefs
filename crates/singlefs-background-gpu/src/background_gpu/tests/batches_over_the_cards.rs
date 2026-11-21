//! 一批在替身卡上的分活与交活：超时重交、交活之前先等、按卡的顺序填满、显存不足重切、条目键重复当场交回。

use super::*;

/// 校验和那一段一批切成 4 段（上限 2200 字节只装得下一个 1000 字节的单元：连同暂存 2104 字节），每段头一次交都等超过上限、
/// 重交一次就做完——这一批在这张卡上第 4 次超时报这张卡用不了。这一段记下交成 3 次、3 片、重交 3 次、分到活的是这张卡。
#[test]
fn the_checksum_batch_cut_into_four_chunks_each_timing_out_once_reports_the_card_unusable() {
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(2200), 0);
    card.submissions_timing_out = VecDeque::from([true, false, true, false, true, false, true]);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = unit_bytes_of(4, 1000);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
        Err(ChecksumBatchFailed::GpuFailed {
            device: card_identifier("0000:01:00.0"),
            cause: GpuFailureCause::UnusableAfterResubmissions {
                resubmissions: 3,
                last_cause: Box::new(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                    wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                }),
            },
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:01:00.0"
            ),
        }),
        "四段各超时一次：这一批第 4 次超时报警"
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 3,
            pieces_submitted: 3,
            resubmissions_after_timeouts: 3,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: vec![card_identifier("0000:01:00.0")],
        }
    );
}

/// 同一条：一批 1 片超时 3 次、第 4 次交做完——照常交回，结果与 CPU 参照逐条相同；这一段记下交成 1 次、1 片、重交 3 次。
#[test]
fn the_checksum_batch_of_one_piece_timing_out_three_times_then_completing_is_returned() {
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    card.submissions_timing_out = VecDeque::from([true, true, true]);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = unit_bytes_of(1, 1000);
    let units = units_to_verify(&unit_bytes);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units),
        Ok(cpu_reference_outcomes_of(&units))
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 1,
            pieces_submitted: 1,
            resubmissions_after_timeouts: 3,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: vec![card_identifier("0000:01:00.0")],
        }
    );
}

/// 两条区域扫描会去读的记录（R 是第一块盘上槽 57600 起那一段）与空映射。
fn two_region_records_with_an_empty_mapping() -> (CompactionRegion, Vec<AllocationRecord>) {
    let region = CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(64 * 900))
        .expect("64 的倍数");
    let records = (0..2_u64)
        .map(|offset| AllocationRecord {
            device: DeviceIdentity(0),
            slot: SlotNumber(region.start_slot().0 + offset),
            span_slots: 1,
            generation: CheckpointTxg(7),
            is_released: false,
            quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
        })
        .collect();
    (region, records)
}

/// 同一条在区域扫描那一段的接线：卡上记着前一次超时、还在跑的提交，交活之前等了两回没等到、第三回等到，再交一段超时一次、重交做完——
/// 这一批在这张卡上超时 3 次，照常交回；这一段记下交成 1 次、2 片、重交 1 次。交活之前那一等不算进超时、或这一段不套重交，都交回出错。
#[test]
fn the_region_record_batch_waiting_twice_before_handing_over_and_timing_out_once_is_returned() {
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
        .with_a_submission_running_past_its_wait(100);
    card.earlier_submission_waits_timing_out = VecDeque::from([true, true]);
    card.submissions_timing_out = VecDeque::from([true]);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let (region, records) = two_region_records_with_an_empty_mapping();
    let unit_bytes = [7_u8; 64];
    let to_classify: Vec<RegionRecordToClassify<'_, usize>> = records
        .iter()
        .enumerate()
        .map(|(entry_key, record)| RegionRecordToClassify {
            entry_key,
            record: *record,
            unit_bytes: Some(&unit_bytes),
        })
        .collect();
    assert_eq!(
        background_gpu
            .both_kernel_classifications_of_the_region_records(region, &to_classify, &[])
            .map(|classifications| classifications.len()),
        Ok(2),
        "交活之前两回加这一段一回，第 3 次超时之内：照常交回"
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .region_record_classification,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 1,
            pieces_submitted: 2,
            resubmissions_after_timeouts: 1,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: vec![card_identifier("0000:01:00.0")],
        }
    );
}

/// 同一条在选段打分那一段的接线：同上一条的时序，两个段。
#[test]
fn the_live_slot_batch_waiting_twice_before_handing_over_and_timing_out_once_is_returned() {
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
        .with_a_submission_running_past_its_wait(100);
    card.earlier_submission_waits_timing_out = VecDeque::from([true, true]);
    card.submissions_timing_out = VecDeque::from([true]);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_area = UnitAreaOfOneDevice {
        device: DeviceIdentity(0),
        start: UnitAreaStart::of_the_default_journal_ring(),
        slots: 128,
    };
    let segments: Vec<ClusterSegmentEntryKey> = (0..2_u64)
        .map(|segment| ClusterSegmentEntryKey {
            device: DeviceIdentity(0),
            start_slot: SlotNumber(unit_area.start.slot().0 + 64 * segment),
        })
        .collect();
    assert_eq!(
        background_gpu.both_kernel_live_slots_of_the_cluster_segments(&[], unit_area, &segments),
        Ok(vec![(0, 0), (0, 0)]),
        "交活之前两回加这一段一回，第 3 次超时之内：照常交回"
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .live_slots_per_cluster_segment,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 1,
            pieces_submitted: 2,
            resubmissions_after_timeouts: 1,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: vec![card_identifier("0000:01:00.0")],
        }
    );
}

/// 外壳上：两张卡，「0000:01:00.0」上限 1000 字节而这个进程里之前泄漏了 950 字节，余量 50 字节装不下校验和那一段最小的一片（连同暂存 112 字节）——
/// 这一批不分给它，两条全交另一张卡，结果与 CPU 参照逐条相同；这一段记下分到活的只有「0000:41:00.0」。
#[test]
fn the_checksum_batch_goes_wholly_to_the_other_card_when_one_card_is_short_of_the_smallest_piece() {
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![
            CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(1000), 950),
            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
        ],
        &memory_leaked,
    );
    let unit_bytes = unit_bytes_of(2, 100);
    let units = units_to_verify(&unit_bytes);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units),
        Ok(cpu_reference_outcomes_of(&units))
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 1,
            pieces_submitted: 2,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: vec![card_identifier("0000:41:00.0")],
        }
    );
}

/// 第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」：上限 2 MiB、卡上实际只空闲 512 KiB，八个 64 KiB 的单元
/// 照上限一段装得下（连同暂存 1 MiB 多）而建缓冲显存不足——压到那一段的一半重切成三、三、二个一段，照常交回，判定与 CPU 参照逐条相同、全都确认；
/// 这一段记下重切 1 次、交成 3 次、8 片。
#[test]
fn the_checksum_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu(
) {
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(2 << 20), 0);
    card.free_bytes = Some(512 << 10);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = unit_bytes_of(8, 64 << 10);
    let units = units_to_verify(&unit_bytes);
    let reconciled = background_gpu
        .verify_unit_checksums(
            &units,
            &mut ConsecutiveUncertaintyCounts::with_no_entry_uncertain(),
        )
        .expect("显存不足按更小的片重切重交：不报出错");
    assert_eq!(
        reconciled.decisions,
        cpu_reference_outcomes_of(&units)
            .into_iter()
            .map(|(outcome, _same)| EntryDecision::ConfirmedByBothKernels(outcome))
            .collect::<Vec<_>>(),
        "判定与 CPU 参照逐条相同"
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 3,
            pieces_submitted: 8,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 1,
            devices_given_work: vec![card_identifier("0000:01:00.0")],
        }
    );
}

/// 只空闲 10 字节的那张替身卡上一个 1000 字节的单元一路压小（连同暂存 2104、1048、520、256、128 字节的一段各建不成一次）到连最小的一片（112 字节）都建不成时，
/// 它报的那一条：最后建不成的一段 128 字节、最小的一片 112 字节。
fn short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
    pci_bus_address: &str,
) -> GpuShortOfFreeMemoryForTheSmallestPiece {
    GpuShortOfFreeMemoryForTheSmallestPiece {
        device: card_identifier(pci_bus_address),
        bytes_of_the_last_chunk_not_created: GpuMemoryInBytes(128),
        smallest_piece_bytes: GpuMemoryInBytes(112),
    }
}

/// 「连一片都装不下就不用这张卡」：唯一一张卡上只空闲 10 字节，一路压小到连最小的一片（112 字节）都建不成——这一批不用这张卡，参与的卡一张都不剩，
/// 报空闲显存那一种（它是因空闲显存不用的那一张，没有因显存上限不用的），重切了 5 次。
#[test]
fn the_single_card_short_of_free_memory_for_the_smallest_piece_leaves_every_participating_gpu_short(
) {
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    card.free_bytes = Some(10);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = unit_bytes_of(1, 1000);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
        Err(
            ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                gpus_short_of_free_memory: vec![
                    short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
                        "0000:01:00.0"
                    )
                ],
                gpus_left_out_under_their_memory_limit: Vec::new(),
            }
        )
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification
            .recuts_after_running_out_of_memory,
        5,
        "2104、1048、520、256、128 字节的一段各建不成一次"
    );
}

/// 一张卡的上限一次只交得下一个 1000 字节的单元（连同暂存 2104 字节）：按卡的顺序它填一个，剩下的才给后面的卡。
const MEMORY_LIMIT_HOLDING_ONE_UNIT_OF_ONE_THOUSAND_BYTES: u64 = 2104;

/// 「连一片都装不下就不用这张卡」用于空闲显存：两张卡、两个 1000 字节的单元，排在前面的「0000:01:00.0」的上限一次只交得下一个，
/// 按卡的顺序它填一个、另一个给「0000:41:00.0」；「0000:01:00.0」只空闲 10 字节、压到最小的一片都建不成——这一批不用它，它分到的那一条转给
/// 「0000:41:00.0」重交（接在后面交、按条目的次序接回），照常交回、与 CPU 参照逐条相同；这一段记下交成 2 次、2 片、重切 5 次、分到活的是两张卡（先「0000:01:00.0」）。
#[test]
fn the_checksum_batch_goes_to_the_other_card_when_one_card_is_short_of_free_memory_for_the_smallest_piece(
) {
    let mut card_short_of_free_memory = CardComputedOnTheCpuForTests::of_the_address(
        "0000:01:00.0",
        Some(MEMORY_LIMIT_HOLDING_ONE_UNIT_OF_ONE_THOUSAND_BYTES),
        0,
    );
    card_short_of_free_memory.free_bytes = Some(10);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![
            card_short_of_free_memory,
            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
        ],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = unit_bytes_of(2, 1000);
    let units = units_to_verify(&unit_bytes);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units),
        Ok(cpu_reference_outcomes_of(&units)),
        "一张卡空闲的装不下最小的一片：活转给另一张，照常交回"
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 2,
            pieces_submitted: 2,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 5,
            devices_given_work: vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0")
            ],
        }
    );
}

/// 同上，两张卡都只空闲 10 字节：两条按卡的顺序先都给排在前面的「0000:01:00.0」、它不用之后都转给「0000:41:00.0」，这一批两张都不用，报空闲显存那一种，按不用的先后列出两张；
/// 重切了 12 次（一张 6 次）。
#[test]
fn the_checksum_batch_whose_cards_are_all_short_of_free_memory_reports_every_participating_gpu_short(
) {
    let mut cards = vec![
        CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0),
        CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
    ];
    for card in &mut cards {
        card.free_bytes = Some(10);
    }
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        cards,
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = unit_bytes_of(2, 1000);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
        Err(
            ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                gpus_short_of_free_memory: vec![
                    short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
                        "0000:01:00.0"
                    ),
                    short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
                        "0000:41:00.0"
                    ),
                ],
                gpus_left_out_under_their_memory_limit: Vec::new(),
            }
        )
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification
            .recuts_after_running_out_of_memory,
        12
    );
}

/// 「下一批照设置重新起」：两张卡、「0000:01:00.0」的上限一次只交得下一个单元、只空闲 10 字节，两个 1000 字节的单元，这一批不用它；
/// 下一批（外壳的一批是一段的一次调用）它又照设置排在前面、分到一个（又压小 5 次、又不用它），两条都照常交回。
#[test]
fn the_card_short_of_free_memory_is_given_work_again_in_the_next_batch() {
    let mut card_short_of_free_memory = CardComputedOnTheCpuForTests::of_the_address(
        "0000:01:00.0",
        Some(MEMORY_LIMIT_HOLDING_ONE_UNIT_OF_ONE_THOUSAND_BYTES),
        0,
    );
    card_short_of_free_memory.free_bytes = Some(10);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![
            card_short_of_free_memory,
            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
        ],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = unit_bytes_of(2, 1000);
    let units = units_to_verify(&unit_bytes);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units),
        Ok(cpu_reference_outcomes_of(&units))
    );
    drop(background_gpu.take_the_submissions_of_the_batch());
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units),
        Ok(cpu_reference_outcomes_of(&units))
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 2,
            pieces_submitted: 2,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 5,
            devices_given_work: vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0")
            ],
        },
        "下一批那张卡又分到活、又压小 5 次"
    );
}

/// 外壳上：校验和那一段一条 1000 字节的单元，头一次交等超过上限，重交之前等前一次那一等又超时 2 回、第 3 回等到再交、做完——
/// 这一段照常交回、与 CPU 参照相同，这张卡上交了 2 次，这一段记下重交 1 次（只数真交出去的）。
#[test]
fn the_checksum_batch_whose_wait_before_the_resubmission_times_out_twice_reports_one_resubmission()
{
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    card.submissions_timing_out = VecDeque::from([true]);
    card.earlier_submission_waits_timing_out = VecDeque::from([true, true]);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = unit_bytes_of(1, 1000);
    let units = units_to_verify(&unit_bytes);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units),
        Ok(cpu_reference_outcomes_of(&units)),
        "超时 3 次之后做完：照常交回"
    );
    let submissions = background_gpu
        .take_the_submissions_of_the_batch()
        .unit_checksum_verification;
    assert_eq!(
        (
            background_gpu.cards_on_this_host[0].submissions_made,
            submissions.resubmissions_after_timeouts
        ),
        (2, 1),
        "这张卡上交了 2 次，重交只数真交出去的那 1 次（{submissions:?}）"
    );
}

/// 按卡的顺序填满那几条用例里的单元：每个 100 字节。
const BYTES_OF_A_UNIT_IN_THE_FILL: usize = 100;

/// 校验和那一段一次交得下 `unit_count` 个 [`BYTES_OF_A_UNIT_IN_THE_FILL`] 字节的单元、多一个就交不下的显存上限（`CHECKSUM_CHUNK_FOOTPRINT` 算的）。
fn memory_limit_holding_units_in_one_submission(unit_count: u64) -> u64 {
    let bytes_of_a_unit = u64::try_from(BYTES_OF_A_UNIT_IN_THE_FILL).expect("小数");
    CHECKSUM_CHUNK_FOOTPRINT
        .bytes_held_by_a_chunk(
            EntryCountOfAChunk(unit_count)
                .input_bytes_of_the_entries(InputLengthOnTheHostInBytes(bytes_of_a_unit)),
            EntryCountOfAChunk(unit_count),
        )
        .0
}

/// 本机 `card_count` 张替身卡的总线地址，按本机的次序：「0000:01:00.0」「0000:41:00.0」「0000:81:00.0」……
fn addresses_of_the_cards(card_count: usize) -> Vec<String> {
    (0..card_count)
        .map(|card| format!("0000:{:02x}:00.0", 0x01 + 0x40 * card))
        .collect()
}

/// `card_count` 张替身卡全都参与、按本机的次序，每张给 `memory_limit_in_bytes` 的显存上限，交一批 `unit_count` 个 [`BYTES_OF_A_UNIT_IN_THE_FILL`] 字节的单元：
/// 结果与 CPU 参照逐条相同（断言在里面）；交回这一段记下的与每张卡交了几次（按本机的次序）。
fn checksum_batch_filling_cards_on_the_cpu(
    card_count: usize,
    memory_limit_in_bytes: u64,
    unit_count: usize,
) -> (GpuSubmissionsOfOneSegment, Vec<u32>) {
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        addresses_of_the_cards(card_count)
            .iter()
            .map(|address| {
                CardComputedOnTheCpuForTests::of_the_address(
                    address,
                    Some(memory_limit_in_bytes),
                    0,
                )
            })
            .collect(),
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = unit_bytes_of(unit_count, BYTES_OF_A_UNIT_IN_THE_FILL);
    let units = units_to_verify(&unit_bytes);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units),
        Ok(cpu_reference_outcomes_of(&units)),
        "按卡的顺序填满几张卡，结果照样与 CPU 参照逐条相同"
    );
    let submissions = background_gpu
        .take_the_submissions_of_the_batch()
        .unit_checksum_verification;
    let submissions_made_by_each_card = background_gpu
        .cards_on_this_host
        .iter()
        .map(|card| card.submissions_made)
        .collect();
    (submissions, submissions_made_by_each_card)
}

/// 按卡的顺序填满：三张卡各 1 GiB、一批 12 个单元一次交得下——只有排在前面的
/// 「0000:01:00.0」分到活、交一次，后两张一次都不交（不要求每张卡都分到活）。
#[test]
fn the_batch_the_first_card_in_the_order_holds_goes_only_to_that_card() {
    let (submissions, submissions_made_by_each_card) =
        checksum_batch_filling_cards_on_the_cpu(3, 1 << 30, 12);
    assert_eq!(
        submissions,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 1,
            pieces_submitted: 12,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: vec![card_identifier("0000:01:00.0")],
        }
    );
    assert_eq!(submissions_made_by_each_card, vec![1, 0, 0]);
}

/// 同一条：三张卡的上限各一次只交得下 8 个单元，一批 9 个——「0000:01:00.0」填满 8 个，剩下的 1 个才给「0000:41:00.0」，「0000:81:00.0」不分。
#[test]
fn the_batch_overflowing_the_first_card_in_the_order_hands_only_the_rest_to_the_next_card() {
    let (submissions, submissions_made_by_each_card) = checksum_batch_filling_cards_on_the_cpu(
        3,
        memory_limit_holding_units_in_one_submission(8),
        9,
    );
    assert_eq!(
        submissions,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 2,
            pieces_submitted: 9,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0")
            ],
        }
    );
    assert_eq!(submissions_made_by_each_card, vec![1, 1, 0]);
}

/// 同一条的饱和：三张卡的上限各一次只交得下 8 个单元，一批 30 个——前两张各填满 8 个、交一次，
/// 排在最后的那张拿剩下的 14 个、交两次（8 个与 6 个）；每张都分到活、各自都交过，结果与 CPU 参照逐条相同。
#[test]
fn the_batch_saturating_every_card_fills_each_card_and_each_card_submits() {
    let (submissions, submissions_made_by_each_card) = checksum_batch_filling_cards_on_the_cpu(
        3,
        memory_limit_holding_units_in_one_submission(8),
        30,
    );
    assert_eq!(
        submissions,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 4,
            pieces_submitted: 30,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: addresses_of_the_cards(3)
                .iter()
                .map(|address| card_identifier(address.as_str()))
                .collect(),
        }
    );
    assert_eq!(submissions_made_by_each_card, vec![1, 1, 2]);
}

/// 一批 4 个单元交给外壳，交回这一段记下的分到活的卡（结果与 CPU 参照逐条相同）。
fn devices_given_work_by_a_batch_of_four_units(
    background_gpu: &mut BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
) -> Vec<GpuDeviceIdentifier> {
    let unit_bytes = unit_bytes_of(4, BYTES_OF_A_UNIT_IN_THE_FILL);
    let units = units_to_verify(&unit_bytes);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units),
        Ok(cpu_reference_outcomes_of(&units))
    );
    background_gpu
        .take_the_submissions_of_the_batch()
        .unit_checksum_verification
        .devices_given_work
}

/// 同一条「点名了的按 `background_gpu_devices` 的次序」：切换点名「0000:41:00.0」在前、「0000:01:00.0」在后——活少时只有点名在前的那张分到活。
#[test]
fn the_batch_fills_the_cards_in_the_order_they_are_named() {
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        addresses_of_the_cards(2)
            .iter()
            .map(|address| CardComputedOnTheCpuForTests::of_the_address(address, None, 0))
            .collect(),
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let named_in_reverse = settings_admitted_on_the_cards(
        &background_gpu,
        Some(vec![
            card_identifier("0000:41:00.0"),
            card_identifier("0000:01:00.0"),
        ]),
        vec![1 << 30, 1 << 30],
    );
    background_gpu.use_the_settings_from_the_next_batch(&named_in_reverse);
    assert_eq!(
        devices_given_work_by_a_batch_of_four_units(&mut background_gpu),
        vec![card_identifier("0000:41:00.0")],
        "点名在前的那张全拿"
    );
}

/// 一组设置给「0000:01:00.0」的上限连最小的一片都装不下，它被晾下、
/// 活全给「0000:41:00.0」；下一组给它装得下的上限，它请回来——照新上限再量一次代价（另一张不重量），从下一批起照参与的次序排在前面、重新分到活。
#[test]
fn the_card_left_out_under_its_memory_limit_is_measured_again_and_takes_part_in_order_once_brought_back(
) {
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        addresses_of_the_cards(2)
            .iter()
            .map(|address| CardComputedOnTheCpuForTests::of_the_address(address, None, 0))
            .collect(),
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
    let leaving_out_the_card_in_front = settings_admitted_on_the_cards(
        &background_gpu,
        None,
        vec![smallest_piece_bytes - 1, 1 << 30],
    );
    background_gpu.use_the_settings_from_the_next_batch(&leaving_out_the_card_in_front);
    assert_eq!(
        devices_given_work_by_a_batch_of_four_units(&mut background_gpu),
        vec![card_identifier("0000:41:00.0")],
        "晾下的那张不分活"
    );
    let bringing_it_back =
        settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
    background_gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
    assert_eq!(
        background_gpu
            .cards_on_this_host
            .iter()
            .map(|card| card.cost_measurements)
            .collect::<Vec<_>>(),
        vec![1, 0],
        "请回来的那张照新上限再量一次，另一张不重量"
    );
    assert_eq!(
        devices_given_work_by_a_batch_of_four_units(&mut background_gpu),
        vec![card_identifier("0000:01:00.0")],
        "请回来的那张照参与的次序排在前面，活少时它全拿"
    );
}

/// 第 4 条「建缓冲显存不足、切小重切时，这一段在这张卡上已做完的片不留，照新的切法整段从头重交」：
/// 一张卡上限 3000 字节、卡上只空闲 700 字节，三个单元 100、600、600 字节一次交得下（连同暂存 2848 字节）而建不成——压到 1424 字节重切成三段
/// （连同暂存 304、1304、1304），头一段交成（这一次的结果是坏的：替身把它的判定翻了），第二段又建不成——压到 652 字节，照新切法整段从头重交七段
/// （600 字节的单元切成 272、272、56），
/// 头一段交成的那一次的结果不用：判定与 CPU 参照逐条相同；这张卡上交成 8 次（1 + 7），这一段记下交成 8 次、8 片、重切 2 次。
#[test]
fn the_recut_after_running_out_of_memory_resubmits_the_whole_share_without_keeping_the_pieces_already_done(
) {
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(3000), 0);
    card.free_bytes = Some(700);
    card.checksum_submission_with_flipped_verdicts = Some(1);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = vec![vec![3_u8; 100], vec![5_u8; 600], vec![7_u8; 600]];
    let units = units_to_verify(&unit_bytes);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units),
        Ok(cpu_reference_outcomes_of(&units)),
        "已交成的那一片的坏结果不用：整段照新切法重交，判定与 CPU 参照逐条相同"
    );
    assert_eq!(
        background_gpu.cards_on_this_host[0].submissions_made, 8,
        "交成 8 次：头一段那一次，加整段重交的 7 段"
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification,
        GpuSubmissionsOfOneSegment {
            submissions_completed: 8,
            pieces_submitted: 8,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 2,
            devices_given_work: vec![card_identifier("0000:01:00.0")],
        }
    );
}

/// 外壳上：一个单元头一次交等超过上限，之后重交之前那一等连着 3 回等不到、一次都没重交——
/// 第 4 次超时报这张卡用不了，报警带的重交次数是真交出去的 0 次（不是常数 3），原因是最后那一等没等到。
#[test]
fn the_card_unusable_without_any_resubmission_reaching_it_reports_no_resubmission() {
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    card.submissions_timing_out = VecDeque::from([true]);
    card.earlier_submission_waits_timing_out = VecDeque::from([true, true, true]);
    let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    let unit_bytes = unit_bytes_of(1, 1000);
    assert_eq!(
        background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
        Err(ChecksumBatchFailed::GpuFailed {
            device: card_identifier("0000:01:00.0"),
            cause: GpuFailureCause::UnusableAfterResubmissions {
                resubmissions: 0,
                last_cause: Box::new(
                    GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                        wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                    }
                ),
            },
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:01:00.0"
            ),
        })
    );
    assert_eq!(
        background_gpu.cards_on_this_host[0].submissions_made, 1,
        "只交了头一次"
    );
}

// ── 三段都在交卡之前查重复键 ──

/// 一张替身卡，每一次提交都等超过上限（交到它上面这一批就报这张卡出错）。
fn background_gpu_over_a_card_timing_out_every_submission(
) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
    let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
    card.submissions_timing_out = VecDeque::from([true; 16]);
    let (background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![card],
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    );
    background_gpu
}

/// 一段一片都没交过、一张卡都没分到活时记下的样子。
fn submissions_of_a_segment_never_handed_to_a_card() -> GpuSubmissionsOfOneSegment {
    GpuSubmissionsOfOneSegment {
        submissions_completed: 0,
        pieces_submitted: 0,
        resubmissions_after_timeouts: 0,
        recuts_after_running_out_of_memory: 0,
        devices_given_work: Vec::new(),
    }
}

fn repeated_at_the_first_two_positions() -> SameEntryKeyTwiceInOneBatch {
    SameEntryKeyTwiceInOneBatch {
        position_of_the_first_occurrence: 0,
        position_of_the_second_occurrence: 1,
    }
}

/// 校验和那一段一批里同一个条目键两次、交到卡上就会报这张卡出错：交卡之前就查出，交回重复的键，两份核一片都没交、这张卡没分到活。
#[test]
fn a_checksum_batch_with_a_repeated_entry_key_is_returned_as_repeated_before_any_piece_is_handed_to_a_card(
) {
    let mut background_gpu = background_gpu_over_a_card_timing_out_every_submission();
    let unit_bytes = unit_bytes_of(2, 100);
    let units: Vec<UnitChecksumToVerify<'_, usize>> = unit_bytes
        .iter()
        .map(|bytes| UnitChecksumToVerify {
            entry_key: 7,
            unit_bytes: bytes,
            expected_crc32c: singlefs_core::checksum::crc32_castagnoli(bytes),
        })
        .collect();
    assert_eq!(
        background_gpu.verify_unit_checksums(
            &units,
            &mut ConsecutiveUncertaintyCounts::with_no_entry_uncertain()
        ),
        Err(ChecksumBatchFailed::SameEntryKeyTwiceInOneBatch(
            repeated_at_the_first_two_positions()
        ))
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification,
        submissions_of_a_segment_never_handed_to_a_card()
    );
}

/// 同一条在区域扫描那一段：两条记录同一个条目键。
#[test]
fn a_region_record_batch_with_a_repeated_entry_key_is_returned_as_repeated_before_any_piece_is_handed_to_a_card(
) {
    let mut background_gpu = background_gpu_over_a_card_timing_out_every_submission();
    let (region, records) = two_region_records_with_an_empty_mapping();
    let unit_bytes = [7_u8; 64];
    let to_classify: Vec<RegionRecordToClassify<'_, usize>> = records
        .iter()
        .map(|record| RegionRecordToClassify {
            entry_key: 7,
            record: *record,
            unit_bytes: Some(&unit_bytes),
        })
        .collect();
    assert_eq!(
        background_gpu.classify_region_records(
            region,
            &to_classify,
            &[],
            &mut ConsecutiveUncertaintyCounts::with_no_entry_uncertain()
        ),
        Err(RegionRecordBatchFailed::SameEntryKeyTwiceInOneBatch(
            repeated_at_the_first_two_positions()
        ))
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .region_record_classification,
        submissions_of_a_segment_never_handed_to_a_card()
    );
}

/// 同一条在选段打分那一段：同一个段两次。
#[test]
fn a_live_slot_batch_with_a_repeated_segment_is_returned_as_repeated_before_any_piece_is_handed_to_a_card(
) {
    let mut background_gpu = background_gpu_over_a_card_timing_out_every_submission();
    let unit_area = UnitAreaOfOneDevice {
        device: DeviceIdentity(0),
        start: UnitAreaStart::of_the_default_journal_ring(),
        slots: 128,
    };
    let segment = ClusterSegmentEntryKey {
        device: DeviceIdentity(0),
        start_slot: unit_area.start.slot(),
    };
    assert_eq!(
        background_gpu.count_live_slots_per_cluster_segment(
            &[],
            unit_area,
            &[segment, segment],
            &mut ConsecutiveUncertaintyCounts::with_no_entry_uncertain()
        ),
        Err(LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch(
            repeated_at_the_first_two_positions()
        ))
    );
    assert_eq!(
        background_gpu
            .take_the_submissions_of_the_batch()
            .live_slots_per_cluster_segment,
        submissions_of_a_segment_never_handed_to_a_card()
    );
}
