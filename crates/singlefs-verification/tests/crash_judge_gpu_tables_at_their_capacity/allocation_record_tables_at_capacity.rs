//! 分配记录那两张表（恢复内核读一版分配记录树、扫描内核收一条根的分配记录，与同宽的已释放记录表）：恰好装满、多一条与抬上限之前的边界。

use super::*;

/// 被判那一版的分配记录恰好 2048 条、其中已释放 2010 条（多于 1024 条）：每个状态 GPU 都判得了，对拍字与 CPU 逐状态相同。
/// 已释放那张表改回 1024 时，那一版的根槽写落了的状态扫描内核在站点 29 记判不了，这条红。
#[test]
fn the_gpu_judge_judges_a_version_of_exactly_two_thousand_and_forty_eight_allocation_records_like_the_cpu_judge(
) {
    assert_the_gpu_judge_judges_a_version_of_these_many_allocation_records_like_the_cpu_judge(
        "the_gpu_judge_judges_a_version_of_exactly_two_thousand_and_forty_eight_allocation_records_like_the_cpu_judge",
        "allocation-records-at-the-old-capacity-of-the-gpu-tables",
        GROW_AND_SHRINK_ROUNDS_OF_THE_FLOWS_AROUND_TWO_THOUSAND_AND_FORTY_EIGHT_RECORDS,
        DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_AT_TWO_THOUSAND_AND_FORTY_EIGHT_RECORDS,
        RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE_TO_FOUR_THOUSAND_AND_NINETY_SIX,
        RELEASED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_FIRST_RAISE,
        ExaminedRecordsOfTheFlow::WithinTheOldCapacityOfTheExaminedTable,
    );
}

/// 被判那一版的分配记录 2050 条（多于两张表抬到 4096 之前的 2048 条）：每个状态 GPU 都判得了，对拍字与 CPU 逐状态相同。
/// 恢复内核那张表改回 2048 时，有一遍恢复落到那一版的状态恢复内核记条目过多；扫描内核那张改回 2048 时，那一版的根槽写落了的状态在站点 20 记判不了：
/// 两样都让这条红（容器树旋钮压小的随机历史第 48 步起分配记录多于 2048 条，就是这两处判不了）。
#[test]
fn the_gpu_judge_judges_a_version_of_two_thousand_and_fifty_allocation_records_past_the_old_capacity_like_the_cpu_judge(
) {
    assert_the_gpu_judge_judges_a_version_of_these_many_allocation_records_like_the_cpu_judge(
        "the_gpu_judge_judges_a_version_of_two_thousand_and_fifty_allocation_records_past_the_old_capacity_like_the_cpu_judge",
        "allocation-records-just-past-the-old-capacity-of-the-gpu-tables",
        GROW_AND_SHRINK_ROUNDS_OF_THE_FLOWS_AROUND_TWO_THOUSAND_AND_FORTY_EIGHT_RECORDS,
        DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_JUST_PAST_TWO_THOUSAND_AND_FORTY_EIGHT_RECORDS,
        RECORDS_JUST_PAST_THE_GPU_TABLES_BEFORE_THE_RAISE_TO_FOUR_THOUSAND_AND_NINETY_SIX,
        RELEASED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_FIRST_RAISE,
        ExaminedRecordsOfTheFlow::WithinTheOldCapacityOfTheExaminedTable,
    );
}

/// 被判那一版的分配记录 2088 条、其中已释放 2050 条（这段历史回退下界不抬、F = 0，扫描内核把已释放的全数收进释放代高于 F 的那张表，
/// 多于那张表抬之前的 2048 条）：每个状态 GPU 都判得了，对拍字与 CPU 逐状态相同。那张表改回 2048 时，那一版的根槽写落了的状态
/// 扫描内核在站点 34 记判不了，这条红。各版未释放的分配记录去重累计 2086 条，多于 I-3.10 去重表抬之前的 2048 条：那张表满了就记判不了的那时候，
/// 判满的界退回 2048 时 13 个状态都在站点 31 记判不了；此刻表满之后照判，界退回 2048 只多读几次单元头，这条照绿（不再钉那张表的界）。
#[test]
fn the_gpu_judge_judges_a_version_of_more_than_two_thousand_and_forty_eight_records_released_above_the_floor_like_the_cpu_judge(
) {
    assert_the_gpu_judge_judges_a_version_of_these_many_allocation_records_like_the_cpu_judge(
        "the_gpu_judge_judges_a_version_of_more_than_two_thousand_and_forty_eight_records_released_above_the_floor_like_the_cpu_judge",
        "allocation-records-released-above-the-floor-past-the-old-capacity-of-the-gpu-table",
        GROW_AND_SHRINK_ROUNDS_OF_THE_FLOWS_AROUND_TWO_THOUSAND_AND_FORTY_EIGHT_RECORDS,
        DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_PAST_THE_OLD_RELEASED_ABOVE_THE_FLOOR_CAPACITY,
        RECORDS_OF_THE_VERSION_PAST_THE_OLD_RELEASED_ABOVE_THE_FLOOR_CAPACITY,
        RELEASED_RECORDS_ABOVE_THE_FLOOR_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE,
        ExaminedRecordsOfTheFlow::PastTheOldCapacityOfTheExaminedTable,
    );
}

/// 对拍字里恢复内核写的那几样：红位（两遍 oracle 与记录核对器）、两遍各自落到的根、走读结局与失败点。池级 checker 内核判不了的状态，
/// 恢复内核那一半照样判完、照样与 CPU 比。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecoveryCrossCheckWords {
    red_bits: u32,
    consulted_pass_landed_root: LandedRootWords,
    ignored_pass_landed_root: LandedRootWords,
    consulted_pass_outcome: u32,
    ignored_pass_outcome: u32,
    consulted_pass_failure_point: u32,
    ignored_pass_failure_point: u32,
}

fn recovery_cross_check_words_of(words: &CrossCheckWords) -> RecoveryCrossCheckWords {
    RecoveryCrossCheckWords {
        red_bits: words.red_bits,
        consulted_pass_landed_root: words.consulted_pass_landed_root,
        ignored_pass_landed_root: words.ignored_pass_landed_root,
        consulted_pass_outcome: words.consulted_pass_outcome,
        ignored_pass_outcome: words.ignored_pass_outcome,
        consulted_pass_failure_point: words.consulted_pass_failure_point,
        ignored_pass_failure_point: words.ignored_pass_failure_point,
    }
}

/// 被判那一版的分配记录 4098 条（抬到 8192 之前的 4096 之上最近造得出来的条数）：两遍恢复有一遍落到那一版的状态，GPU 恢复内核判得了
/// （分配记录那张表 8192 条装得下），恢复那几个对拍字（[`RecoveryCrossCheckWords`]）与 CPU 相同；恢复内核那张表改回 4096 时这些状态记判不了，这条红。
/// 那一版的根槽写落了的状态，池级 checker 记判不了、原因是条目过多那一类（站点不钉：这条不回收的流上走读内核按单元记的几张表还是 4096 条
/// （欠账 C899），CPU 一侧在全落的镜像上数到引用集合 4098 条、I-3.5 落点 4098 个，哪一张先满看走读的次序）；两个内核都判得了的状态对拍字与 CPU 相同。
/// CPU 判器每个状态照旧判、都不红。
#[test]
fn the_gpu_recovery_kernel_judges_a_version_of_four_thousand_and_ninety_eight_allocation_records_and_the_pool_checker_kernels_leave_it_to_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_recovery_kernel_judges_a_version_of_four_thousand_and_ninety_eight_allocation_records_and_the_pool_checker_kernels_leave_it_to_the_cpu_judge";
    const FLOW_NAME: &str =
        "allocation-records-just-past-four-thousand-and-ninety-six-within-the-gpu-recovery-table";
    let flow = record_the_flow(
        FLOW_NAME,
        GROW_AND_SHRINK_ROUNDS_OF_THE_FLOW_JUST_PAST_FOUR_THOUSAND_AND_NINETY_SIX_RECORDS,
        DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_JUST_PAST_FOUR_THOUSAND_AND_NINETY_SIX_RECORDS,
    );
    assert_the_allocation_records_of_the_versions(
        FLOW_NAME,
        &flow,
        RECORDS_JUST_PAST_THE_GPU_TABLES_BEFORE_THE_RAISE_TO_EIGHT_THOUSAND_ONE_HUNDRED_AND_NINETY_TWO,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_the_cpu_judge_finds_every_state_green(FLOW_NAME, &cpu);
    let mut states_whose_recovery_lands_on_the_judged_version = 0usize;
    let mut states_left_to_the_cpu = 0usize;
    let mut problems: Vec<String> = Vec::new();
    for (ordinal, (cpu_judgement, gpu_words)) in cpu.iter().zip(&gpu).enumerate() {
        let recovery_could_not_judge = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0;
        let checker_reason = gpu_words[POOL_CHECKER_UNDECIDABLE_WORD];
        let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
        if cpu_judgement.a_recovery_lands_on_the_judged_version {
            states_whose_recovery_lands_on_the_judged_version += 1;
            if recovery_could_not_judge {
                problems.push(format!(
                    "状态 {ordinal}：恢复落到 4098 条那一版，GPU 恢复内核该判得了（分配记录那张表 {RECORDS_THE_GPU_TABLES_HOLD} 条），实际记判不了、第 1 字 {:#x}",
                    gpu_words[1]
                ));
            } else if recovery_cross_check_words_of(&gpu_cross_check_words)
                != recovery_cross_check_words_of(&cpu_judgement.cross_check_words)
            {
                problems.push(format!(
                    "状态 {ordinal}：恢复落到 4098 条那一版，GPU 恢复内核判得了而与 CPU 不一致：GPU {:?}；CPU {:?}",
                    recovery_cross_check_words_of(&gpu_cross_check_words),
                    recovery_cross_check_words_of(&cpu_judgement.cross_check_words)
                ));
            }
        }
        if cpu_judgement.judged_root_persisted
            && checker_reason & POOL_CHECKER_UNDECIDABLE_REASON_MASK
                != RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES
        {
            problems.push(format!(
                "状态 {ordinal}：4098 条那一版的根槽写落了，池级 checker 该记判不了、原因是条目过多（{RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES}），实际原因字 {checker_reason:#x}"
            ));
        }
        if gpu_could_not_judge(gpu_words) {
            states_left_to_the_cpu += 1;
            continue;
        }
        if !cross_check_words_agree_in_the_cross_check(
            &gpu_cross_check_words,
            &cpu_judgement.cross_check_words,
        ) {
            problems.push(format!(
                "状态 {ordinal}：GPU 判得了而与 CPU 不一致：GPU {gpu_cross_check_words:?}；CPU {:?}",
                cpu_judgement.cross_check_words
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "流 {FLOW_NAME}：{} 处不对（前几处）：\n{}",
        problems.len(),
        problems
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        states_whose_recovery_lands_on_the_judged_version > 0,
        "流 {FLOW_NAME}：要有两遍恢复有一遍落到 4098 条那一版的状态（GPU 恢复内核读那一版的分配记录）"
    );
    assert!(
        states_left_to_the_cpu > 0,
        "流 {FLOW_NAME}：要有 GPU 判不了、交 CPU 判的状态（4098 条那一版的根槽写落了，池级 checker 走读内核那几张表到顶）"
    );
}

/// 8194 条那条流被判那次之前的顺序写：17 次各写 200 个单元、再写 201 个、再写 144 个（每次整个文件重写）。
/// CPU 一侧现数的：前一版（144 个单元）8166 条分配记录，被判那次缩回 1 个之后 8194 条；盘 0 上 3749 条 journal 记录写。
/// 换 201 那一次是凑偶数之外的那两条：17 次 200 个、再 145 个时前一版 8170、被判那一版 8198 条（长到 145 个时元数据多一档）。
fn sequential_writes_before_the_judged_publish_just_past_eight_thousand_one_hundred_and_ninety_two_records(
) -> Vec<usize> {
    std::iter::repeat_n(200, 17).chain([201, 144]).collect()
}

/// 被判那一版的分配记录 8194 条（此刻 8192 之上最近造得出来的条数）：两遍恢复有一遍落到那一版的状态，GPU 恢复内核记判不了（条目过多），交 CPU 判；
/// 两遍都落到前一版（8166 条）的状态，GPU 恢复内核判得了，恢复那几个对拍字（[`RecoveryCrossCheckWords`]）与 CPU 相同。
/// 那一版的根槽写落了的状态，池级 checker 记判不了、原因是条目过多那一类（站点不钉：走读内核按单元记的几张表还是 4096 条，欠账 C899）；
/// 两个内核都判得了的状态对拍字与 CPU 相同。CPU 判器每个状态照旧判、都不红。
/// 取卡之前在 CPU 上核：被判那一版恰好 8194 条、前一版多于 4096 且不多于 8192 条，journal 记录多于 2048、不多于 4096 条（两个内核扫环收得下）。
/// 判别力（`crates/mutations.tsv`）：恢复内核那张表按 4096 判满时，恢复落到前一版的状态记判不了，这条红。
#[test]
fn the_gpu_recovery_kernel_leaves_a_version_of_eight_thousand_one_hundred_and_ninety_four_allocation_records_to_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_recovery_kernel_leaves_a_version_of_eight_thousand_one_hundred_and_ninety_four_allocation_records_to_the_cpu_judge";
    const FLOW_NAME: &str = "allocation-records-just-past-the-gpu-recovery-table";
    let flow = record_the_flow_of_these_sequential_writes(
        FLOW_NAME,
        &sequential_writes_before_the_judged_publish_just_past_eight_thousand_one_hundred_and_ninety_two_records(),
    );
    assert_the_allocation_records_of_the_versions(
        FLOW_NAME,
        &flow,
        RECORDS_JUST_PAST_THE_GPU_TABLES,
    );
    let records_of_the_version_before_the_judged_one = flow
        .allocation_record_counts_of_the_version_before_the_judged_one()
        .every_record;
    assert!(
        records_of_the_version_before_the_judged_one
            > RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE_TO_EIGHT_THOUSAND_ONE_HUNDRED_AND_NINETY_TWO,
        "流 {FLOW_NAME}：被判那一版之前那一版 {records_of_the_version_before_the_judged_one} 条，要多于 {RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE_TO_EIGHT_THOUSAND_ONE_HUNDRED_AND_NINETY_TWO} 条（恢复内核那张表按 4096 判满时恢复落到它的状态判不了）"
    );
    let journal_records = flow.journal_record_writes_on_device_zero_after_mkfs;
    assert!(
        journal_records > JOURNAL_RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE
            && journal_records <= JOURNAL_RECORDS_THE_GPU_TABLES_HOLD,
        "流 {FLOW_NAME}：盘 0 上 {journal_records} 条 journal 记录写，要多于 {JOURNAL_RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE}、不多于 {JOURNAL_RECORDS_THE_GPU_TABLES_HOLD}（两个内核扫环收得下）"
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_the_cpu_judge_finds_every_state_green(FLOW_NAME, &cpu);
    let mut states_whose_recovery_lands_on_the_judged_version = 0usize;
    let mut states_whose_recoveries_land_before_the_judged_version = 0usize;
    let mut states_left_to_the_cpu = 0usize;
    let mut problems: Vec<String> = Vec::new();
    for (ordinal, (cpu_judgement, gpu_words)) in cpu.iter().zip(&gpu).enumerate() {
        let recovery_could_not_judge = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0;
        let checker_reason = gpu_words[POOL_CHECKER_UNDECIDABLE_WORD];
        let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
        if cpu_judgement.a_recovery_lands_on_the_judged_version {
            states_whose_recovery_lands_on_the_judged_version += 1;
            if !(recovery_could_not_judge && gpu_words[1] == RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES)
            {
                problems.push(format!(
                    "状态 {ordinal}：恢复落到 8194 条那一版，GPU 恢复内核该记判不了（条目过多 {RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES}），实际判不了 {recovery_could_not_judge}、第 1 字 {:#x}",
                    gpu_words[1]
                ));
            }
        } else {
            states_whose_recoveries_land_before_the_judged_version += 1;
            if recovery_could_not_judge {
                problems.push(format!(
                    "状态 {ordinal}：两遍恢复都没落到 8194 条那一版，GPU 恢复内核该判得了（前一版 {records_of_the_version_before_the_judged_one} 条，分配记录那张表 {RECORDS_THE_GPU_TABLES_HOLD} 条），实际记判不了、第 1 字 {:#x}",
                    gpu_words[1]
                ));
            } else if recovery_cross_check_words_of(&gpu_cross_check_words)
                != recovery_cross_check_words_of(&cpu_judgement.cross_check_words)
            {
                problems.push(format!(
                    "状态 {ordinal}：两遍恢复都没落到 8194 条那一版，GPU 恢复内核判得了而与 CPU 不一致：GPU {:?}；CPU {:?}",
                    recovery_cross_check_words_of(&gpu_cross_check_words),
                    recovery_cross_check_words_of(&cpu_judgement.cross_check_words)
                ));
            }
        }
        if cpu_judgement.judged_root_persisted
            && checker_reason & POOL_CHECKER_UNDECIDABLE_REASON_MASK
                != RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES
        {
            problems.push(format!(
                "状态 {ordinal}：8194 条那一版的根槽写落了，池级 checker 该记判不了、原因是条目过多（{RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES}），实际原因字 {checker_reason:#x}"
            ));
        }
        if gpu_could_not_judge(gpu_words) {
            states_left_to_the_cpu += 1;
            continue;
        }
        if !cross_check_words_agree_in_the_cross_check(
            &gpu_cross_check_words,
            &cpu_judgement.cross_check_words,
        ) {
            problems.push(format!(
                "状态 {ordinal}：GPU 判得了而与 CPU 不一致：GPU {gpu_cross_check_words:?}；CPU {:?}",
                cpu_judgement.cross_check_words
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "流 {FLOW_NAME}：{} 处不对（前几处）：\n{}",
        problems.len(),
        problems
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        states_whose_recovery_lands_on_the_judged_version > 0,
        "流 {FLOW_NAME}：要有两遍恢复有一遍落到 8194 条那一版的状态（GPU 恢复内核读那一版的分配记录、记判不了）"
    );
    assert!(
        states_whose_recoveries_land_before_the_judged_version > 0,
        "流 {FLOW_NAME}：要有两遍恢复都没落到 8194 条那一版的状态（GPU 恢复内核读前一版 {records_of_the_version_before_the_judged_one} 条分配记录）"
    );
    assert!(
        states_left_to_the_cpu > 0,
        "流 {FLOW_NAME}：要有 GPU 判不了、交 CPU 判的状态（恢复落到 8194 条那一版，或它的根槽写落了）"
    );
}
