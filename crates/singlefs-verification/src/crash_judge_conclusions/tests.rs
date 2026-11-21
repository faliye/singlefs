use super::pool_checker_source::{
    is_in_the_image_family, is_in_the_walk_family, pool_checker_source_before_its_tests,
    pool_checker_sources_before_their_tests, source_with_line_comments_blanked, WalkFamilySource,
};
use super::*;

/// 池级 checker 被抛弃根那一半的两种说明（引用的单元被重新分配或抹头、树表指针全零走不完）都读出点名的那一条根；别的说明一条都不读出。
#[test]
fn the_abandoned_root_is_read_from_both_details_of_the_abandoned_root_half_and_from_no_other_detail(
) {
    let reused = "被抛弃的根（实例 1、txg 5）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：盘 0 槽 1234 校验和对不上";
    let all_zero = "被抛弃的根（实例 2、txg 8）走不完：根记录里树表指针全零，core 的根记录读者照它读盘 0 槽 0、读不出（这条根本身损坏，不只是它引用的单元被重新分配或抹头）：";
    assert_eq!(
        abandoned_roots_named_in_a_pool_checker_detail(reused),
        [AbandonedRootNamedByThePoolChecker {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(5),
        }]
    );
    assert_eq!(
        abandoned_roots_named_in_a_pool_checker_detail(all_zero),
        [AbandonedRootNamedByThePoolChecker {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(8),
        }]
    );
    assert_eq!(
        abandoned_roots_named_in_a_pool_checker_detail("最新根引用的单元校验和对不上：盘 1 槽 9"),
        []
    );
}

/// 恢复自称的那一版那一栏：没写的照看 journal 那一遍落到的判、恒不判红；写了的照写的判，与落到的根实例代号或 checkpoint_txg 有一样不同、
/// 一边落到根一边没有都判红，相同不判红。红了出一项记录核对器的红项，从库里读回的红项还原出同一位。
#[test]
fn claimed_version_other_than_the_one_landed_on_takes_the_record_checker_and_reddens_its_own_bit() {
    let newest = Some((InstanceGeneration(1), CheckpointTxg(7)));
    let older = Some((InstanceGeneration(1), CheckpointTxg(6)));
    let other_instance = Some((InstanceGeneration(2), CheckpointTxg(7)));
    let not_written = ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery;
    let written = ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(newest);
    assert_eq!(not_written.version_the_record_checker_takes(older), older);
    assert_eq!(written.version_the_record_checker_takes(older), newest);
    assert!(!not_written.is_not_the_version_landed_on(older));
    assert!(!not_written.is_not_the_version_landed_on(None));
    assert!(!written.is_not_the_version_landed_on(newest));
    for landed_on in [older, other_instance, None] {
        assert!(
            written.is_not_the_version_landed_on(landed_on),
            "写的是 {newest:?}、落到 {landed_on:?}：判红"
        );
    }
    assert!(
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None)
            .is_not_the_version_landed_on(older),
        "写的那一版是没有根、落到了一条根：判红"
    );
    let words = CrossCheckWords {
        red_bits: 1 << verdict_bits::CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY,
        pool_checker_violated_bits: 0,
        consulted_pass_landed_root: LandedRootWords::NOT_LANDED,
        ignored_pass_landed_root: LandedRootWords::NOT_LANDED,
        consulted_pass_outcome: RECOVERY_OUTCOME_CODE_FAILED,
        ignored_pass_outcome: RECOVERY_OUTCOME_CODE_FAILED,
        consulted_pass_failure_point: GPU_FAILURE_POINT_NO_VALID_ROOT,
        ignored_pass_failure_point: GPU_FAILURE_POINT_NO_VALID_ROOT,
        pool_checker_judged_bits: 0,
        scan_rebuild_outcome: SCAN_REBUILD_OUTCOME_NOT_REACHED,
        gpu_could_not_judge: false,
    };
    let items = red_items_of_a_given_state(&words);
    assert_eq!(
        items,
        [format!(
            "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY}"
        )]
    );
    let read_back = cross_check_words_of_a_stored_given_state(
        &items,
        recovery_words_of_the_cross_check_words(&words),
    )
    .expect("这一项认得");
    assert!(claimed_version_is_not_landed_on_in_the_cross_check_words(
        &read_back
    ));
    assert_eq!(read_back.red_bits, words.red_bits);
}

/// 给定状态那一路的红项从对拍字换出、存进库，读回来还原出同样的红位、违例位与恢复字（判到位不存，读回是 0）；
/// oracle 那十四位在对拍之前去掉（两侧都不判不比）。
#[test]
fn given_state_read_back_from_its_red_items_and_recovery_words_keeps_its_red_bits() {
    let words = CrossCheckWords {
        red_bits: (1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT)
            | (1 << verdict_bits::CONSULTED_ORACLE_FIRST)
            | (1 << (verdict_bits::IGNORED_ORACLE_FIRST + 6)),
        pool_checker_violated_bits: (1u128 << 0) | (1u128 << 2),
        consulted_pass_landed_root: LandedRootWords::of_the_landed_root(
            InstanceGeneration(3),
            CheckpointTxg((5 << 32) | 9),
        ),
        ignored_pass_landed_root: LandedRootWords::NOT_LANDED,
        consulted_pass_outcome: RECOVERY_OUTCOME_CODE_FILE_READ,
        ignored_pass_outcome: RECOVERY_OUTCOME_CODE_FAILED,
        consulted_pass_failure_point: GPU_FAILURE_POINT_NONE,
        ignored_pass_failure_point: GPU_FAILURE_POINT_NO_VALID_ROOT,
        pool_checker_judged_bits: (1u128 << 0) | (1u128 << 1) | (1u128 << 2),
        scan_rebuild_outcome: SCAN_REBUILD_OUTCOMES[0].code,
        gpu_could_not_judge: false,
    };
    let without_the_oracle = cross_check_words_without_the_oracle(words);
    assert_eq!(
        without_the_oracle.red_bits,
        1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT,
        "两遍 oracle 的位都去掉，记录核对器那一位留着"
    );
    let items = red_items_of_a_given_state(&without_the_oracle);
    assert_eq!(
        items,
        vec![
            format!(
                "{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{}",
                IMPLEMENTED_INVARIANTS[0]
            ),
            format!(
                "{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{}",
                IMPLEMENTED_INVARIANTS[2]
            ),
            format!(
                "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
            ),
        ]
    );
    let read_back = cross_check_words_of_a_stored_given_state(
        &items,
        recovery_words_of_the_cross_check_words(&without_the_oracle),
    )
    .expect("这一路写的红项读得回");
    let mut without_the_judged_bits = without_the_oracle;
    without_the_judged_bits.pool_checker_judged_bits = 0;
    without_the_judged_bits.scan_rebuild_outcome = SCAN_REBUILD_OUTCOME_NOT_REACHED;
    assert_eq!(
        read_back, without_the_judged_bits,
        "判到位与 I-1.13 的结局码不存，别的逐字段相同"
    );
    assert!(
        cross_check_words_of_a_stored_given_state(
            &["journal_consulted_oracle:no_root_chosen".to_string()],
            recovery_words_of_the_cross_check_words(&without_the_oracle),
        )
        .is_err(),
        "oracle 的红项不是这一路写得出的"
    );
}

/// J7：一块的计数向量清单里每条不变量一行，没判到的状态（不适用、部分没判，连同池级 checker 没报的）都记进不适用：
/// 不适用 = 状态数 − 判到。几段并起来按条相加。
#[test]
fn the_count_vector_of_a_block_puts_every_state_not_judged_on_an_invariant_under_not_applicable() {
    let first_invariant = IMPLEMENTED_INVARIANTS[0];
    let second_invariant = IMPLEMENTED_INVARIANTS[1];
    let mut first_piece = StateCountsOfABlock::default();
    first_piece.judged_states_of_each_invariant.insert(
        first_invariant,
        InvariantStateCounts {
            judged: 2,
            violated: 1,
            not_applicable: 0,
        },
    );
    let mut second_piece = StateCountsOfABlock::default();
    second_piece.judged_states_of_each_invariant.insert(
        first_invariant,
        InvariantStateCounts {
            judged: 1,
            violated: 0,
            not_applicable: 0,
        },
    );
    first_piece.absorb(second_piece);
    let count_vector = first_piece.into_count_vector(5);
    assert_eq!(
        count_vector.per_invariant.len(),
        IMPLEMENTED_INVARIANTS.len(),
        "清单里每条不变量一行"
    );
    assert_eq!(
        count_vector.per_invariant.get(first_invariant),
        Some(&InvariantStateCounts {
            judged: 3,
            violated: 1,
            not_applicable: 2,
        }),
        "两段相加：判到 3、违例 1，五个状态里另两个不适用"
    );
    assert_eq!(
        count_vector.per_invariant.get(second_invariant),
        Some(&InvariantStateCounts {
            judged: 0,
            violated: 0,
            not_applicable: 5,
        }),
        "一个状态都没判到的那一条五个都不适用"
    );
}

/// J7：按发布分的态数逐状态记、几段并起来相加，进计数向量；没记过的发布不出现（不写 0）。
#[test]
fn the_count_vector_of_a_block_carries_the_states_of_each_publish_counted_state_by_state() {
    let mut first_piece = StateCountsOfABlock::default();
    first_piece.count_the_publish_of_a_state(CrashPointReplayPublishOfState::AfterTheLastRoot);
    first_piece.count_the_publish_of_a_state(CrashPointReplayPublishOfState::AfterTheLastRoot);
    let mut second_piece = StateCountsOfABlock::default();
    second_piece.count_the_publish_of_a_state(CrashPointReplayPublishOfState::EveryWritePersisted);
    second_piece.count_states_of_a_publish(CrashPointReplayPublishOfState::AfterTheLastRoot, 3);
    second_piece.count_states_of_a_publish(CrashPointReplayPublishOfState::EveryWritePersisted, 0);
    first_piece.absorb(second_piece);
    let count_vector = first_piece.into_count_vector(6);
    assert_eq!(
        count_vector.states_by_publish,
        BTreeMap::from([
            ("after_the_last_root".to_string(), 5),
            ("every_write_persisted".to_string(), 1),
        ]),
        "两段相加：最后一次根槽写之后 2 + 3 个，全部持久那一个；记 0 个的不出现"
    );
    assert!(
        count_vector.per_observer.is_empty(),
        "判器那一份不带观察者计数"
    );
}

/// J8、M4 第 8 条：失败点码表不挂 `gpu` 特性；恢复字的次序是看 journal 那一遍落到的根四个字（标记、实例代号、checkpoint_txg 低 32 位与
/// 高 32 位）、结局、失败点，再是不看 journal 那一遍的同六样（这一份用例在不编 `gpu` 特性的库单测里也跑）。
#[test]
fn the_recovery_words_of_a_state_are_the_two_passes_roots_outcomes_and_failure_points_without_the_gpu_feature(
) {
    let words = CrossCheckWords {
        red_bits: 0,
        pool_checker_violated_bits: 0,
        consulted_pass_landed_root: LandedRootWords::of_the_landed_root(
            InstanceGeneration(2),
            CheckpointTxg((3 << 32) | 7),
        ),
        ignored_pass_landed_root: LandedRootWords::NOT_LANDED,
        consulted_pass_outcome: RECOVERY_OUTCOME_CODE_FILE_READ,
        ignored_pass_outcome: RECOVERY_OUTCOME_CODE_FAILED,
        consulted_pass_failure_point: GPU_FAILURE_POINT_NONE,
        ignored_pass_failure_point: gpu_failure_point_code(&RecoveryFailure::NoValidRoot),
        pool_checker_judged_bits: 0,
        scan_rebuild_outcome: SCAN_REBUILD_OUTCOME_NOT_REACHED,
        gpu_could_not_judge: false,
    };
    assert_eq!(
        recovery_words_of_the_cross_check_words(&words),
        RecoveryWordsOfState([
            LandedRootWords::LANDED_ON_A_ROOT,
            2,
            7,
            3,
            RECOVERY_OUTCOME_CODE_FILE_READ,
            GPU_FAILURE_POINT_NONE,
            LandedRootWords::LANDED_ON_NO_ROOT,
            u32::MAX,
            u32::MAX,
            u32::MAX,
            RECOVERY_OUTCOME_CODE_FAILED,
            GPU_FAILURE_POINT_NO_VALID_ROOT,
        ])
    );
    assert_ne!(
        gpu_failure_point_code(&RecoveryFailure::NoValidRoot),
        gpu_failure_point_code(&RecoveryFailure::NoDeviceInThePool),
        "两个成员两个码"
    );
}

/// 一遍择不到根的恢复（搭对拍字的构造函数用：这几条用例只看池级 checker 那两种位）。
fn recovery_report_that_chose_no_root() -> singlefs_core::recovery::RecoveryReport {
    singlefs_core::recovery::RecoveryReport {
        outcome: RecoveryOutcome::Failed {
            root: None,
            failure: RecoveryFailure::NoValidRoot,
        },
        effective_root: None,
        journal: singlefs_core::recovery::JournalScanReport::default(),
        mapping_fallbacks: 0,
        devices_without_a_mountable_system_configuration_slot: Vec::new(),
    }
}

/// 一遍落到 `effective_root` 的恢复（结局与失败点同 [`recovery_report_that_chose_no_root`]：下面几条用例只看落到的根）。
fn recovery_report_that_landed_on(
    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
) -> singlefs_core::recovery::RecoveryReport {
    let mut report = recovery_report_that_chose_no_root();
    report.effective_root = effective_root;
    report
}

/// 两遍各落到给定的根时，CPU 两个构造函数换出的对拍字（崩溃点重放那一份、给定状态那一份），池级 checker 一条都不报。
fn cross_check_words_of_both_cpu_constructors_landed_on(
    consulted_root: Option<(InstanceGeneration, CheckpointTxg)>,
    ignored_root: Option<(InstanceGeneration, CheckpointTxg)>,
) -> [CrossCheckWords; 2] {
    let judgement = CrashStateJudgement {
        consulted: recovery_report_that_landed_on(consulted_root),
        ignored: recovery_report_that_landed_on(ignored_root),
        root_persisted: false,
        consulted_oracle: None,
        ignored_oracle: None,
        pool_checker: Vec::new(),
        records: crate::memory_pool::RecordCheck::default(),
    };
    [
        cross_check_words_of_cpu_judgement(&judgement),
        cross_check_words_of_a_given_state(
            &recovery_report_that_landed_on(consulted_root),
            &recovery_report_that_landed_on(ignored_root),
            &[],
            crate::memory_pool::RecordCheck::default(),
            ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery,
        ),
    ]
}

/// 两条根（或没落到根）在对拍与恢复字里分不分得开：两遍都落到 `left` 的那一份与两遍都落到 `right` 的那一份，CPU 两个构造函数各比一次——
/// 对拍字不同、对拍比较判对不上、恢复字不同；一个状态看 journal 那一遍落到 `left`、不看那一遍落到 `right`，判两遍不同。
/// `scenario` 是这一对的说法，写进断言消息。
fn assert_the_two_landings_are_told_apart(
    left: Option<(InstanceGeneration, CheckpointTxg)>,
    right: Option<(InstanceGeneration, CheckpointTxg)>,
    scenario: &str,
) {
    let left_words = cross_check_words_of_both_cpu_constructors_landed_on(left, left);
    let right_words = cross_check_words_of_both_cpu_constructors_landed_on(right, right);
    for (left_of_the_constructor, right_of_the_constructor) in left_words.iter().zip(&right_words) {
        assert_ne!(
            left_of_the_constructor, right_of_the_constructor,
            "{scenario}：{left:?} 与 {right:?} 的对拍字相同"
        );
        assert!(
            !cross_check_words_agree_in_the_cross_check(
                left_of_the_constructor,
                right_of_the_constructor
            ),
            "{scenario}：{left:?} 与 {right:?} 在对拍比较里判对得上"
        );
        assert_ne!(
            recovery_words_of_the_cross_check_words(left_of_the_constructor),
            recovery_words_of_the_cross_check_words(right_of_the_constructor),
            "{scenario}：{left:?} 与 {right:?} 的恢复字相同"
        );
    }
    assert!(
        recovery_passes_differ(
            &recovery_report_that_landed_on(left),
            &recovery_report_that_landed_on(right)
        ),
        "{scenario}：看 journal 那一遍落到 {left:?}、不看那一遍落到 {right:?}，判成两遍相同"
    );
}

/// 审计复核 T01（`records/2026-10-24-singlefs-checker-tier-审计复核.md`「存完整 u64」）：同一个实例、checkpoint_txg
/// 低 32 位相同（0、1、7、`u32::MAX`）而高 32 位不同（0 对 1、2、`0xFFFF_FFFF`）的两条根，十二对都分得开——对拍字、对拍比较、恢复字、
/// 两遍恢复相不相同那一判。
#[test]
fn two_roots_of_one_instance_whose_checkpoint_txgs_differ_only_in_the_high_thirty_two_bits_are_told_apart(
) {
    for low in [0u64, 1, 7, u64::from(u32::MAX)] {
        for high in [1u64, 2, 0xFFFF_FFFF] {
            assert_the_two_landings_are_told_apart(
                Some((InstanceGeneration(1), CheckpointTxg(low))),
                Some((InstanceGeneration(1), CheckpointTxg((high << 32) | low))),
                &format!("低 32 位 {low}、高 32 位 0 对 {high}"),
            );
        }
    }
}

/// T01：checkpoint_txg 2^32 − 1 与 2^32（低 32 位一个全 1、一个全 0，跨过低 32 位的进位）分得开。
#[test]
fn the_checkpoint_txg_just_below_two_to_the_thirty_second_and_two_to_the_thirty_second_are_told_apart(
) {
    assert_the_two_landings_are_told_apart(
        Some((InstanceGeneration(1), CheckpointTxg((1 << 32) - 1))),
        Some((InstanceGeneration(1), CheckpointTxg(1 << 32))),
        "2^32 − 1 对 2^32",
    );
}

/// T01：没落到根与落到一条实例代号 `u32::MAX`、checkpoint_txg `u64::MAX` 的根（坏镜像、外来镜像才有，每个字都与没落到根的那一组相同，
/// 只差标记）分得开；落到的那条根读得回完整的实例代号与 checkpoint_txg，没落到根读回 `None`。
#[test]
fn landing_on_no_root_and_landing_on_the_root_whose_every_word_is_at_its_maximum_are_told_apart() {
    let root_at_the_maximum = (InstanceGeneration(u32::MAX), CheckpointTxg(u64::MAX));
    assert_the_two_landings_are_told_apart(
        None,
        Some(root_at_the_maximum),
        "没落到根对最大的那条根",
    );
    assert_eq!(
        landed_root_words_of(Some(root_at_the_maximum)).landed_root(),
        Some(root_at_the_maximum),
        "落到的那条根读得回"
    );
    assert_eq!(
        landed_root_words_of(None),
        LandedRootWords::NOT_LANDED,
        "没落到根换出的是没落到那一组"
    );
    assert_eq!(
        LandedRootWords::NOT_LANDED.landed_root(),
        None,
        "没落到根读回 None"
    );
}

/// T01 的对照：低 32 位不同的 txg 7 与 txg 8 照旧分得开；两遍落到同一条高 32 位非 0 的根时对拍判对得上、两遍判相同。
#[test]
fn the_checkpoint_txgs_seven_and_eight_are_told_apart_and_one_root_past_thirty_two_bits_agrees_with_itself(
) {
    assert_the_two_landings_are_told_apart(
        Some((InstanceGeneration(1), CheckpointTxg(7))),
        Some((InstanceGeneration(1), CheckpointTxg(8))),
        "txg 7 对 txg 8",
    );
    let root_past_thirty_two_bits = Some((InstanceGeneration(1), CheckpointTxg((2 << 32) | 7)));
    for words in cross_check_words_of_both_cpu_constructors_landed_on(
        root_past_thirty_two_bits,
        root_past_thirty_two_bits,
    ) {
        assert!(
            cross_check_words_agree_in_the_cross_check(&words, &words),
            "同一份对拍字判对得上"
        );
    }
    assert!(
        !recovery_passes_differ(
            &recovery_report_that_landed_on(root_past_thirty_two_bits),
            &recovery_report_that_landed_on(root_past_thirty_two_bits)
        ),
        "两遍落到同一条根、结局与失败点相同：判两遍相同"
    );
}

/// CPU 一侧两个构造函数换出的对拍字：崩溃点重放那一份（[`cross_check_words_of_cpu_judgement`]）与给定状态那一份
/// （[`cross_check_words_of_a_given_state`]），池级 checker 取同一份判定。
fn cross_check_words_of_both_cpu_constructors(
    pool_checker: Vec<(&'static str, InvariantVerdict)>,
) -> [CrossCheckWords; 2] {
    let judgement = CrashStateJudgement {
        consulted: recovery_report_that_chose_no_root(),
        ignored: recovery_report_that_chose_no_root(),
        root_persisted: false,
        consulted_oracle: None,
        ignored_oracle: None,
        pool_checker: pool_checker.clone(),
        records: crate::memory_pool::RecordCheck::default(),
    };
    [
        cross_check_words_of_cpu_judgement(&judgement),
        cross_check_words_of_a_given_state(
            &recovery_report_that_chose_no_root(),
            &recovery_report_that_chose_no_root(),
            &pool_checker,
            crate::memory_pool::RecordCheck::default(),
            ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery,
        ),
    ]
}

/// 扫描重建那一条（I-1.13）在池级 checker 清单里的下标。
fn scan_rebuild_invariant_position() -> usize {
    IMPLEMENTED_INVARIANTS
        .iter()
        .position(|candidate| {
            *candidate == singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX
        })
        .expect("I-1.13 在池级 checker 的清单里")
}

/// C821 的口径：I-1.13 判成立时 CPU 两个构造函数换出的判到位第 92 位是 1、违例位第 92 位是 0（GPU 扫描重建那一截照同一口径置位，
/// 对拍逐位比）。
#[test]
fn the_cpu_cross_check_words_set_the_scan_rebuild_judged_bit_when_the_scan_rebuild_invariant_holds()
{
    let scan_rebuild_bit = 1u128 << scan_rebuild_invariant_position();
    for words in cross_check_words_of_both_cpu_constructors(vec![(
        singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
        InvariantVerdict::Holds,
    )]) {
        assert_eq!(
            words.pool_checker_judged_bits, scan_rebuild_bit,
            "I-1.13 成立：判到位只有第 92 位（{words:?}）"
        );
        assert_eq!(
            words.pool_checker_violated_bits, 0,
            "I-1.13 成立：违例位一位都不置（{words:?}）"
        );
    }
}

/// C821 的口径：I-1.13 判违例时 CPU 两个构造函数换出的违例位与判到位第 92 位都是 1。
#[test]
fn the_cpu_cross_check_words_set_the_scan_rebuild_violated_and_judged_bits_when_the_scan_rebuild_invariant_is_violated(
) {
    let scan_rebuild_bit = 1u128 << scan_rebuild_invariant_position();
    for words in cross_check_words_of_both_cpu_constructors(vec![(
        singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
        InvariantVerdict::Violated("扫描重建与遍历方向对不上".to_string()),
    )]) {
        assert_eq!(
            (
                words.pool_checker_violated_bits,
                words.pool_checker_judged_bits
            ),
            (scan_rebuild_bit, scan_rebuild_bit),
            "I-1.13 违例：违例位与判到位都只有第 92 位（{words:?}）"
        );
    }
}

/// 补上扫描重建内核之后对拍比较处不再去掉第 92 位（C821）：两组位只差第 92 位判对不上（GPU 没置、CPU 置了与反过来各一次），
/// 两边都置第 92 位而别的位相同判对得上。
#[test]
fn the_cross_check_comparison_of_pool_checker_bits_compares_the_scan_rebuild_bit() {
    let scan_rebuild_bit = 1u128 << scan_rebuild_invariant_position();
    let shared_bits = (1u128 << 0) | (1u128 << 40);
    assert!(
        !pool_checker_bits_agree_in_the_cross_check(shared_bits, shared_bits | scan_rebuild_bit),
        "只差第 92 位（GPU 没置、CPU 置了）判对不上"
    );
    assert!(
        !pool_checker_bits_agree_in_the_cross_check(shared_bits | scan_rebuild_bit, shared_bits),
        "只差第 92 位（GPU 置了、CPU 没置）判对不上"
    );
    assert!(
        pool_checker_bits_agree_in_the_cross_check(
            shared_bits | scan_rebuild_bit,
            shared_bits | scan_rebuild_bit
        ),
        "两边都置第 92 位、别的位相同判对得上"
    );
}

/// 内核不置位的表是空的，对拍比较处去掉的位也是空的：任何一组位经 [`pool_checker_bits_compared_by_the_cross_check`] 都原样交回。
/// 这是一条等价变异的留档（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」等价那一行）：C821 那一条「比较处不屏蔽表里那几位」
/// （把 `bits & !POOL_CHECKER_BITS_LEFT_OUT_OF_THE_CROSS_CHECK` 换成 `bits`）在今天的空表上对每一组位都与原式同值，换哪个取样点都抓不到；
/// `crates/mutations.tsv` 里 C821 那一行换成「比较处又去掉第 92 位」，由 `the_cross_check_comparison_of_pool_checker_bits_compares_the_scan_rebuild_bit` 判出。
/// 表里又登记回一条时这里红，那时 C821 原来那一条变异重新会改行为。
#[test]
fn the_cross_check_comparison_passes_every_pool_checker_bit_through_while_no_invariant_is_left_unset(
) {
    for position in 0..u128::BITS {
        let bit = 1u128 << position;
        assert_eq!(
            pool_checker_bits_compared_by_the_cross_check(bit),
            bit,
            "第 {position} 位原样交回"
        );
    }
    assert_eq!(
        pool_checker_bits_compared_by_the_cross_check(u128::MAX),
        u128::MAX,
        "全部位一起原样交回"
    );
}

/// 对拍字的比较不再去掉违例位与判到位的第 92 位：CPU 判 I-1.13 违例、GPU 那一份只在第 92 位上不同时判对不上。
#[test]
fn the_cross_check_words_differing_only_in_the_scan_rebuild_bits_disagree() {
    let [cpu_words, _] = cross_check_words_of_both_cpu_constructors(vec![
        (IMPLEMENTED_INVARIANTS[0], InvariantVerdict::Holds),
        (
            singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
            InvariantVerdict::Violated("扫描重建与遍历方向对不上".to_string()),
        ),
    ]);
    assert!(
        cross_check_words_agree_in_the_cross_check(&cpu_words, &cpu_words),
        "同一份对拍字判对得上"
    );
    let mut gpu_words_without_the_scan_rebuild_violation = cpu_words;
    gpu_words_without_the_scan_rebuild_violation.pool_checker_violated_bits = 0;
    assert!(
        !cross_check_words_agree_in_the_cross_check(
            &gpu_words_without_the_scan_rebuild_violation,
            &cpu_words
        ),
        "违例位只差第 92 位判对不上"
    );
    let mut gpu_words_without_the_scan_rebuild_judgement = cpu_words;
    gpu_words_without_the_scan_rebuild_judgement.pool_checker_judged_bits = 1;
    assert!(
        !cross_check_words_agree_in_the_cross_check(
            &gpu_words_without_the_scan_rebuild_judgement,
            &cpu_words
        ),
        "判到位只差第 92 位判对不上"
    );
}

/// 扫描重建的结局码从 0 起连着排到表里最后一种，中间不留空号（表里的次序是扫描重建依次判到的次序，不是码的次序）。
/// 判别力（`crates/mutations.tsv`）：I-9.32 判损坏那一种的码写成 26（23 空出来、26 撞上整池交回那一种），这条红。
#[test]
fn scan_rebuild_outcome_codes_run_from_zero_without_a_gap() {
    let mut codes: Vec<u32> = SCAN_REBUILD_OUTCOMES
        .iter()
        .map(|outcome| outcome.code)
        .collect();
    codes.sort_unstable();
    let outcome_count =
        u32::try_from(SCAN_REBUILD_OUTCOMES.len()).expect("结局表是几十行的常量表，行数装得进 u32");
    let expected_codes: Vec<u32> = (0..outcome_count).collect();
    assert_eq!(
        codes, expected_codes,
        "结局码是 0 到行数减一各一个，中间不空"
    );
}

/// 内核不置位那张表是空的（C821 还清：I-1.13 由扫描重建那一截判，I-1.13 是下标 92）：对拍不去掉任何一位；
/// 只读快照那一批的 I-3.6（下标 93）、I-3.7（下标 94）由引用与快照那一截判；多文件删除那一批新立的 I-9.40（下标 102）由扫描内核的
/// 删除态扫描判（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2，不留成「进覆盖表、内核不置位」的欠账）。
/// 表里又登记回一条时这里红。
#[test]
fn no_invariant_is_left_unset_by_the_gpu_kernels() {
    let position_of = |invariant: &str| {
        IMPLEMENTED_INVARIANTS
            .iter()
            .position(|candidate| *candidate == invariant)
            .expect("在池级 checker 的清单里")
    };
    assert!(
        POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS.is_empty(),
        "内核不置位的表是空的，实际 {POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS:?}"
    );
    assert_eq!(
        scan_rebuild_invariant_position(),
        92,
        "I-1.13 是清单第 93 条（下标 92），内核里的 INVARIANT_I_1_13 照这个下标置位"
    );
    assert_eq!(
        position_of(
            singlefs_checker::image::EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY
        ),
        102,
        "I-9.40 是清单第 103 条（下标 102），扫描内核删除态扫描里的 INVARIANT_I_9_40 照这个下标置位、不进内核不置位的表"
    );
    assert_eq!(
        (
            position_of(
                singlefs_checker::image::ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD
            ),
            position_of(singlefs_checker::image::PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE)
        ),
        (104, 105),
        "I-7.18、I-7.19 是清单第 105、106 条（下标 104、105），走读内核里的 INVARIANT_I_7_18、INVARIANT_I_7_19 照这两个下标置位"
    );
    assert_eq!(
        (
            position_of(
                singlefs_checker::image::DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE
            ),
            position_of(singlefs_checker::image::LIVE_SNAPSHOTS_ARE_ENUMERABLE)
        ),
        (93, 94),
        "I-3.6、I-3.7 是清单第 94、95 条（下标 93、94），内核里的 INVARIANT_I_3_6、INVARIANT_I_3_7 照这两个下标置位"
    );
    assert_eq!(
        POOL_CHECKER_BITS_LEFT_OUT_OF_THE_CROSS_CHECK, 0,
        "对拍不去掉任何一位"
    );
}

/// 里程碑四自建 GC 判的那五条（I-4.4 下标 73，I-3.15、I-5.6、I-3.16、I-1.14 下标 95–98）对拍逐位比：GPU 与 CPU 只在其中一位上不同，
/// 判器级的比较判对不上（那一位列进内核不置位的表、比较处把它从两边去掉时这里红）。
#[test]
fn the_compaction_and_relocation_invariants_are_compared_bit_by_bit_in_the_cross_check() {
    let position_of = |invariant: &str| {
        IMPLEMENTED_INVARIANTS
            .iter()
            .position(|candidate| *candidate == invariant)
            .expect("在池级 checker 的清单里")
    };
    for invariant in [
        singlefs_checker::image::INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
        singlefs_checker::image::UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
        singlefs_checker::image::COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
        singlefs_checker::image::FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
        singlefs_checker::image::UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
    ] {
        let bit = 1u128 << position_of(invariant);
        assert!(
            !pool_checker_bits_agree_in_the_cross_check(bit, 0),
            "{invariant} 的违例位只在 GPU 一侧置了，对拍判对不上"
        );
        assert!(
            !pool_checker_bits_agree_in_the_cross_check(0, bit),
            "{invariant} 的违例位只在 CPU 一侧置了，对拍判对不上"
        );
    }
}

/// 红项编码：逐项换行接起来存，读回来逐项相同、次序不变；绿的状态是空向量，读回一项都没有。
#[test]
fn the_red_items_read_back_from_their_encoding_item_by_item_in_order() {
    let items = vec![
        "pool_checker:I-3.1".to_string(),
        "record_checker:root_without_record".to_string(),
    ];
    let encoded = encoded_verdict_of_the_red_items(&items);
    assert_eq!(red_items_of_the_encoded_verdict(&encoded), items);
    assert_eq!(encoded_verdict_of_the_red_items(&[]).0, Vec::<u8>::new());
    assert!(red_items_of_the_encoded_verdict(&encoded_verdict_of_the_red_items(&[])).is_empty());
}

// ---------------- 扫描重建结局表与池级 checker 源码里 I-1.13 能报的理由逐句相等 ----------------

/// 判定累加器（`singlefs_checker::image::Judgements`）上第一个实参点名一条不变量的四种调用。
#[derive(Clone, Copy, Debug)]
enum JudgementsMethodNamingAnInvariant {
    Judge,
    NotApplicable,
    PartOfTheRangeNotJudged,
    PartiallyJudged,
}

impl JudgementsMethodNamingAnInvariant {
    const EVERY_METHOD: [Self; 4] = [
        Self::Judge,
        Self::NotApplicable,
        Self::PartOfTheRangeNotJudged,
        Self::PartiallyJudged,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Judge => "judge",
            Self::NotApplicable => "not_applicable",
            Self::PartOfTheRangeNotJudged => "part_of_the_range_not_judged",
            Self::PartiallyJudged => "partially_judged",
        }
    }
}

/// 源码里一处调用的一个实参：去掉首尾空白之后的原文与它在源码里的字节起点。
struct CallArgument {
    start: usize,
    text: String,
}

/// 从 `open` 那个开括号（`(`、`[`、`{` 之一）起，配对的闭括号的字节位置；串字面量里的括号不算。
fn position_of_the_matching_bracket(source: &str, open: usize) -> usize {
    let mut depth = 0usize;
    let mut inside_a_string_literal = false;
    let mut the_previous_character_escapes = false;
    for (offset, character) in source[open..].char_indices() {
        if inside_a_string_literal {
            if the_previous_character_escapes {
                the_previous_character_escapes = false;
            } else if character == '\\' {
                the_previous_character_escapes = true;
            } else if character == '"' {
                inside_a_string_literal = false;
            }
            continue;
        }
        match character {
            '"' => inside_a_string_literal = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + offset;
                }
            }
            _ => {}
        }
    }
    panic!("第 {open} 字节起的括号没有配对的闭括号");
}

/// 源码里 `.<方法名>(` 的每一处调用：调用的字节起点与按顶层逗号切开的实参（括号、串字面量里的逗号不切）。
fn method_calls_in(source: &str, method_name: &str) -> Vec<(usize, Vec<CallArgument>)> {
    let call_opening = format!(".{method_name}(");
    source
        .match_indices(call_opening.as_str())
        .map(|(call_start, _)| {
            let open = call_start + call_opening.len() - 1;
            let close = position_of_the_matching_bracket(source, open);
            let mut arguments = Vec::new();
            let mut argument_start = open + 1;
            let mut depth = 0usize;
            let mut inside_a_string_literal = false;
            let mut the_previous_character_escapes = false;
            let mut push_the_argument_ending_at =
                |end: usize, arguments_so_far: &mut Vec<CallArgument>| {
                    let raw = &source[argument_start..end];
                    let text = raw.trim();
                    if !text.is_empty() {
                        arguments_so_far.push(CallArgument {
                            start: argument_start + (raw.len() - raw.trim_start().len()),
                            text: text.to_string(),
                        });
                    }
                    argument_start = end + 1;
                };
            for (offset, character) in source[open + 1..close].char_indices() {
                if inside_a_string_literal {
                    if the_previous_character_escapes {
                        the_previous_character_escapes = false;
                    } else if character == '\\' {
                        the_previous_character_escapes = true;
                    } else if character == '"' {
                        inside_a_string_literal = false;
                    }
                    continue;
                }
                match character {
                    '"' => inside_a_string_literal = true,
                    '(' | '[' | '{' => depth += 1,
                    ')' | ']' | '}' => depth -= 1,
                    ',' if depth == 0 => {
                        push_the_argument_ending_at(open + 1 + offset, &mut arguments);
                    }
                    _ => {}
                }
            }
            push_the_argument_ending_at(close, &mut arguments);
            (call_start, arguments)
        })
        .collect()
}

/// 一个串字面量实参（`"…"`）里的字；不是串字面量是 `None`。
fn string_literal_contents(argument: &str) -> Option<String> {
    let contents = argument.strip_prefix('"')?.strip_suffix('"')?;
    assert!(
        !contents.contains('\\') && !contents.contains('"'),
        "串字面量 {argument} 里有转义或引号：这份扫描不展开转义"
    );
    Some(contents.to_string())
}

/// 一段源码里每个串字面量的字，按出现次序。
fn string_literals_in(source: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut rest = source;
    while let Some(open) = rest.find('"') {
        let after_open = &rest[open + 1..];
        let close = after_open.find('"').expect("串字面量有收尾的引号");
        literals.push(after_open[..close].to_string());
        rest = &after_open[close + 1..];
    }
    literals
}

/// 源码里 `const <名字>: &str =` 那个串常量的值。
fn string_constant_in(source: &str, constant_name: &str) -> Option<String> {
    let declaration = format!("const {constant_name}: &str =");
    let after = &source[source.find(declaration.as_str())? + declaration.len()..];
    let literal_end = after.find(';')?;
    string_literal_contents(after[..literal_end].trim())
}

/// 调用所在的那个函数从哪个字节起（往前找最近的一个顶格函数，或 `impl` 块里缩进四格的方法开头）。
/// 认缩进的方法是因为池级 checker 的判定累加器上有转交 `invariant` 的方法（`Judgements::judge_naming_the_slots` 调
/// `self.judge(invariant, …)`）：它在 `impl` 块里，往前不一定有顶格函数；往前有的话那一个也不是包住它的函数，
/// 那个函数里遍历 `IMPLEMENTED_INVARIANTS` 的一圈不该算成这一处的绑定。可见性认 `pub`、`pub(crate)` 与 `pub(super)`
/// （`walk` 的子模块之间互相用的函数写 `pub(super)`）：走读那一族接成一段（[`WalkFamilySource`]）之后，认不出的函数开头让往前找
/// 越过它、落进前面别的函数，甚至前一份文件里的函数。
fn start_of_the_function_enclosing(source: &str, position: usize) -> usize {
    [
        "\nfn ",
        "\npub fn ",
        "\npub(crate) fn ",
        "\npub(super) fn ",
        "\n    fn ",
        "\n    pub fn ",
        "\n    pub(crate) fn ",
        "\n    pub(super) fn ",
    ]
    .iter()
    .filter_map(|opening| source[..position].rfind(opening))
    .max()
    .expect("调用在某个顶格的函数或 impl 块里缩进四格的方法里")
}

/// 判定累加器 `impl` 块里转交 `invariant` 的方法（形如 `Judgements::judge_naming_the_slots`）不算点名 I-1.13：
/// 包住调用的是那个方法，不是它前面那个遍历 `IMPLEMENTED_INVARIANTS` 的顶格函数；往前没有顶格函数时也不 panic。
#[test]
fn method_in_an_impl_block_forwarding_invariant_does_not_name_the_scan_rebuild_invariant() {
    let forwarding_method = concat!(
        "impl Judgements {\n",
        "    pub fn judge_naming_the_slots(&mut self, invariant: &'static str, holds: bool) {\n",
        "        self.judge(invariant, holds, detail);\n",
        "    }\n",
        "}\n",
    );
    let loop_over_every_invariant_then_the_forwarding_method = format!(
        "\npub fn walk_every_invariant(judgements: &mut Judgements) {{\n    for invariant in IMPLEMENTED_INVARIANTS {{\n        judgements.not_applicable(invariant, \"走不到\");\n    }}\n}}\n{forwarding_method}"
    );
    for source in [
        forwarding_method.to_string(),
        loop_over_every_invariant_then_the_forwarding_method,
    ] {
        let calls = method_calls_in(&source, "judge");
        assert_eq!(calls.len(), 1, "造的源码里恰好一处 .judge(：{source}");
        let (call_start, arguments) = &calls[0];
        assert!(
            !first_argument_names_the_scan_rebuild_invariant(
                &source,
                *call_start,
                &arguments[0].text
            ),
            "impl 块里转交 invariant 的方法不点名 I-1.13（包住它的是那个方法，不是前面遍历全部不变量的顶格函数）：{source}"
        );
    }
}

/// 写 `pub(super)` 的函数（顶格的，与 `impl` 块里缩进四格的）转交 `invariant`，前面紧挨着一个遍历 `IMPLEMENTED_INVARIANTS` 的顶格函数：
/// 包住调用的是那个 `pub(super)` 函数，不点名 I-1.13。认不出 `pub(super)` 的函数开头，往前找就落进那个遍历全部不变量的函数，这条红。
#[test]
fn function_visible_to_its_parent_module_forwarding_invariant_does_not_name_the_scan_rebuild_invariant(
) {
    let loop_over_every_invariant = concat!(
        "\nfn judge_every_invariant(judgements: &mut Judgements) {\n",
        "    for invariant in IMPLEMENTED_INVARIANTS {\n",
        "        judgements.not_applicable(invariant, \"走不到\");\n",
        "    }\n",
        "}\n",
    );
    let forwarding_function = concat!(
        "pub(super) fn judge_the_invariant_it_is_given(judgements: &mut Judgements, invariant: &'static str) {\n",
        "    judgements.judge(invariant, true, String::new);\n",
        "}\n",
    );
    let forwarding_method = concat!(
        "impl Walk {\n",
        "    pub(super) fn judge_the_invariant_it_is_given(&mut self, invariant: &'static str) {\n",
        "        self.judgements.judge(invariant, true, String::new);\n",
        "    }\n",
        "}\n",
    );
    for forwarding in [forwarding_function, forwarding_method] {
        let source = format!("{loop_over_every_invariant}{forwarding}");
        let calls = method_calls_in(&source, "judge");
        assert_eq!(calls.len(), 1, "造的源码里恰好一处 .judge(：{source}");
        let (call_start, arguments) = &calls[0];
        assert!(
            !first_argument_names_the_scan_rebuild_invariant(
                &source,
                *call_start,
                &arguments[0].text
            ),
            "pub(super) 函数里转交 invariant 不点名 I-1.13（包住它的是那个 pub(super) 函数，不是前面遍历全部不变量的顶格函数）：{source}"
        );
    }
}

/// 第一个实参点不点名 I-1.13：直接写那个常量或 `"I-1.13"`，或是遍历 `IMPLEMENTED_INVARIANTS` 那一圈的 `invariant`
/// （往前最近的一处 `invariant` 绑定就是那一圈的头）。别的写法不点名它——别处另写到 I-1.13 的，
/// [`every_reason_the_pool_checker_can_give_for_the_scan_rebuild_is_a_row_of_the_outcome_table`] 另查、判红。
fn first_argument_names_the_scan_rebuild_invariant(
    source: &str,
    call_start: usize,
    first_argument: &str,
) -> bool {
    if first_argument == "SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX"
        || first_argument == "\"I-1.13\""
    {
        return true;
    }
    if first_argument != "invariant" {
        return false;
    }
    // 只在调用所在的函数里找绑定：函数参数叫 invariant 的，调用方传进来的那个常量另由「别处另写到 I-1.13」那一查管
    let within_the_function =
        &source[start_of_the_function_enclosing(source, call_start)..call_start];
    let latest_loop_start = match (
        within_the_function.rfind("for invariant in "),
        within_the_function.rfind("let invariant"),
    ) {
        (Some(loop_start), Some(binding_start)) => {
            (loop_start > binding_start).then_some(loop_start)
        }
        (Some(loop_start), None) => Some(loop_start),
        (None, Some(_) | None) => None,
    };
    latest_loop_start.is_some_and(|loop_start| {
        let header = &within_the_function[loop_start..];
        header.starts_with("for invariant in crate::image::IMPLEMENTED_INVARIANTS {")
            || header.starts_with("for invariant in IMPLEMENTED_INVARIANTS {")
    })
}

/// 理由枚举的 `fn reason` 在池级 checker 的哪一份源码里。
#[derive(Clone, Copy, Debug)]
enum PoolCheckerSourceFile {
    Walk,
    Rebuild,
}

/// I-1.13 那几处 `not_applicable` / `part_of_the_range_not_judged` 的理由实参认成的一句或几句原文（`walk` 是走读那一族接成的一段，
/// [`WalkFamilySource`]）。认法：串字面量；走读那一族里的串常量；绑在遍历方向已判损坏那张表上的 `reason`（那张表的第二栏）；`IndexTableTheWalkDoesNotReadInFull` 与
/// `rebuild::ScanRebuildNotDecided` 的 `reason()`（各自 `fn reason` 里的每一句，绑定在同一个函数的 `Err(…)` 里）。
///
/// # Panics
/// 认不得的实参写法：池级 checker 多了一种理由的来源，在这里补认法。
fn reasons_of_the_argument(
    walk: &str,
    rebuild: &str,
    call_start: usize,
    reason_argument: &str,
) -> Vec<String> {
    const REASON_ENUMS_OF_THE_POOL_CHECKER: [(&str, &str, PoolCheckerSourceFile); 2] = [
        (
            "table_not_read_in_full",
            "IndexTableTheWalkDoesNotReadInFull",
            PoolCheckerSourceFile::Walk,
        ),
        (
            "not_decided",
            "ScanRebuildNotDecided",
            PoolCheckerSourceFile::Rebuild,
        ),
    ];
    if let Some(literal) = string_literal_contents(reason_argument) {
        return vec![literal];
    }
    if let Some(value) = string_constant_in(walk, reason_argument) {
        return vec![value];
    }
    let enclosing_function = &walk[start_of_the_function_enclosing(walk, call_start)..call_start];
    if reason_argument == "reason" {
        assert!(
            enclosing_function.contains(
                "Some((_, reason)) = INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED"
            ),
            "实参 reason 绑在遍历方向已判损坏那张表上"
        );
        return singlefs_checker::walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED
            .iter()
            .map(|(_, reason)| (*reason).to_string())
            .collect();
    }
    for (binding, enum_name, source_file) in REASON_ENUMS_OF_THE_POOL_CHECKER {
        if reason_argument != format!("{binding}.reason()") {
            continue;
        }
        assert!(
            enclosing_function.contains(&format!("Err({binding}) =>")),
            "{binding} 绑在同一个函数的 Err({binding}) 里"
        );
        let enum_source = match source_file {
            PoolCheckerSourceFile::Walk => walk,
            PoolCheckerSourceFile::Rebuild => rebuild,
        };
        let implementation_start = enum_source
            .find(&format!("impl {enum_name} {{"))
            .unwrap_or_else(|| panic!("源码里有 impl {enum_name}"));
        let reason_function = implementation_start
            + enum_source[implementation_start..]
                .find("fn reason(self) -> &'static str {")
                .unwrap_or_else(|| panic!("impl {enum_name} 里有 fn reason"));
        let body_open = reason_function + enum_source[reason_function..].find('{').expect("函数体");
        let body =
            &enum_source[body_open..=position_of_the_matching_bracket(enum_source, body_open)];
        let literals = string_literals_in(body);
        assert_eq!(
            literals.len(),
            body.matches("=>").count(),
            "{enum_name}::reason 的每一臂都是一句串字面量"
        );
        return literals;
    }
    panic!("认不得 I-1.13 的这一种理由实参：{reason_argument}（池级 checker 多了一种理由的来源，在这里补认法）");
}

/// 结局表（[`SCAN_REBUILD_OUTCOMES`]）里不适用的每一句，与扫描池级 checker 源码认出的 I-1.13 能报的每一句不适用理由逐句相等：
/// 走读那一族（`walk.rs` 与 `walk/` 下的产品代码接成一段，[`WalkFamilySource`]）里判定累加器四种调用的第一个实参点名 I-1.13 的那几处
/// （直接写那个常量，或遍历 `IMPLEMENTED_INVARIANTS` 的那一圈——走到扫描重建之前整池交回的那几处），理由实参照 [`reasons_of_the_argument`] 认；
/// `partially_judged` 不报 I-1.13，成立与违例那一处在。走读那一族里别处写到 I-1.13 或 `IMPLEMENTED_INVARIANTS`（不是那几种调用的第一个实参、
/// 不是 import、不是那一圈的头）、别的源码文件（`src/` 下递归，测试代码不算）写到 I-1.13、判定累加器那一族（`image.rs` 与 `image/` 下）
/// 里有调用报 I-1.13，都判红：这份扫描认不得它报什么；`image/` 下的子模块里一处判定累加器调用都没照 `image.rs` 的规矩判过（那一圈没下子目录），
/// 也判红。`Judgements::into_report` 的缺省理由不在里面
/// （每条路径都给 I-1.13 记了判定或理由：四处整池交回各记一句，其余都走到 `judge_scan_rebuild_agrees_with_the_walked_index`，那里每条出口都记）。
/// 表少一句、多一句、或一句的字与池级 checker 的不同，这条红。
#[test]
fn every_reason_the_pool_checker_can_give_for_the_scan_rebuild_is_a_row_of_the_outcome_table() {
    const SCAN_REBUILD_INVARIANT_TOKEN: &str = "SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX";
    const SCAN_REBUILD_INVARIANT_LITERAL: &str = "\"I-1.13\"";
    const LOOP_OVER_EVERY_INVARIANT: &str =
        "for invariant in crate::image::IMPLEMENTED_INVARIANTS {";
    let walk_family = WalkFamilySource::read_before_the_tests();
    let walk = walk_family.text.as_str();
    let rebuild =
        source_with_line_comments_blanked(&pool_checker_source_before_its_tests("rebuild.rs"));

    let mut reasons_in_the_source = std::collections::BTreeSet::new();
    let mut first_argument_starts = std::collections::BTreeSet::new();
    let mut is_judged_in_the_source = false;
    for method in JudgementsMethodNamingAnInvariant::EVERY_METHOD {
        for (call_start, arguments) in method_calls_in(walk, method.name()) {
            let first = arguments.first().unwrap_or_else(|| {
                panic!(
                    "{} 的 {} 调用有实参",
                    walk_family.place_of(call_start),
                    method.name()
                )
            });
            first_argument_starts.insert(first.start);
            if !first_argument_names_the_scan_rebuild_invariant(walk, call_start, &first.text) {
                continue;
            }
            match method {
                JudgementsMethodNamingAnInvariant::Judge => is_judged_in_the_source = true,
                JudgementsMethodNamingAnInvariant::PartiallyJudged => panic!(
                    "{} 把 I-1.13 报成部分没判：结局表里没有这一种",
                    walk_family.place_of(call_start)
                ),
                JudgementsMethodNamingAnInvariant::NotApplicable
                | JudgementsMethodNamingAnInvariant::PartOfTheRangeNotJudged => {
                    let reason = arguments.get(1).unwrap_or_else(|| {
                        panic!(
                            "{} 的 {} 调用有理由实参",
                            walk_family.place_of(call_start),
                            method.name()
                        )
                    });
                    reasons_in_the_source.extend(reasons_of_the_argument(
                        walk,
                        &rebuild,
                        call_start,
                        &reason.text,
                    ));
                }
            }
        }
    }

    let import_start = walk
        .find("use crate::image::{")
        .expect("走读那一族从 image 引名字");
    let import_end = import_start + walk[import_start..].find("};").expect("import 收尾");
    for (token_start, _) in walk.match_indices(SCAN_REBUILD_INVARIANT_TOKEN) {
        assert!(
            (import_start..import_end).contains(&token_start)
                || first_argument_starts.contains(&token_start),
            "{} 另写到 I-1.13（不是判定累加器调用的第一个实参，也不在 import 里）：这份扫描认不得它报什么，补认法",
            walk_family.place_of(token_start)
        );
    }
    for (token_start, _) in walk.match_indices(SCAN_REBUILD_INVARIANT_LITERAL) {
        assert!(
            first_argument_starts.contains(&token_start),
            "{} 另写到 \"I-1.13\"：这份扫描认不得它报什么，补认法",
            walk_family.place_of(token_start)
        );
    }
    for (token_start, _) in walk.match_indices("IMPLEMENTED_INVARIANTS") {
        let loop_header_start = token_start + "IMPLEMENTED_INVARIANTS".len();
        assert!(
            walk[..loop_header_start].ends_with(&LOOP_OVER_EVERY_INVARIANT[..LOOP_OVER_EVERY_INVARIANT.len() - 2]),
            "{} 另写到 IMPLEMENTED_INVARIANTS（不是遍历每条不变量那一圈的头）：这份扫描认不得它报什么，补认法",
            walk_family.place_of(token_start)
        );
    }
    let pool_checker_sources = pool_checker_sources_before_their_tests();
    let relative_paths: Vec<&str> = pool_checker_sources
        .iter()
        .map(|(relative_path, _)| relative_path.as_str())
        .collect();
    assert!(
        relative_paths.contains(&"walk.rs") && relative_paths.contains(&"image.rs"),
        "池级 checker 的源码里有 walk.rs 与 image.rs：{relative_paths:?}"
    );
    let mut image_family_files_with_a_call_checked = std::collections::BTreeSet::new();
    for (relative_path, source_before_its_tests) in &pool_checker_sources {
        if is_in_the_walk_family(relative_path) {
            continue;
        }
        let source = source_with_line_comments_blanked(source_before_its_tests);
        let line_of = |position: usize| source[..position].matches('\n').count() + 1;
        if is_in_the_image_family(relative_path) {
            for method in JudgementsMethodNamingAnInvariant::EVERY_METHOD {
                for (call_start, arguments) in method_calls_in(&source, method.name()) {
                    image_family_files_with_a_call_checked.insert(relative_path.as_str());
                    let first = arguments
                        .first()
                        .map_or("", |argument| argument.text.as_str());
                    assert!(
                        !first_argument_names_the_scan_rebuild_invariant(&source, call_start, first),
                        "{relative_path} 第 {} 行的 {} 调用报 I-1.13：这份扫描只认走读那一族里那几处，补认法",
                        line_of(call_start),
                        method.name()
                    );
                }
            }
        } else {
            assert!(
                !source.contains(SCAN_REBUILD_INVARIANT_TOKEN)
                    && !source.contains(SCAN_REBUILD_INVARIANT_LITERAL),
                "{relative_path} 写到了 I-1.13：这份扫描只认走读那一族里那几处，补认法"
            );
        }
    }
    assert!(
        image_family_files_with_a_call_checked
            .iter()
            .any(|relative_path| relative_path.starts_with("image/")),
        "image/ 下的子模块里的判定累加器调用照 image.rs 的规矩逐处判过（这一圈下子目录）：{image_family_files_with_a_call_checked:?}"
    );

    let reasons_in_the_table: std::collections::BTreeSet<String> = SCAN_REBUILD_OUTCOMES
        .iter()
        .filter_map(|outcome| match outcome.verdict_of_the_pool_checker {
            ScanRebuildVerdictOfThePoolChecker::NotApplicable(reason) => Some(reason.to_string()),
            ScanRebuildVerdictOfThePoolChecker::Holds
            | ScanRebuildVerdictOfThePoolChecker::Violated => None,
        })
        .collect();
    let rows_that_are_not_applicable = SCAN_REBUILD_OUTCOMES
        .iter()
        .filter(|outcome| {
            matches!(
                outcome.verdict_of_the_pool_checker,
                ScanRebuildVerdictOfThePoolChecker::NotApplicable(_)
            )
        })
        .count();
    assert_eq!(
        reasons_in_the_table.len(),
        rows_that_are_not_applicable,
        "结局表里不适用的理由一句一行，不重"
    );
    let missing_from_the_table: Vec<&String> = reasons_in_the_source
        .difference(&reasons_in_the_table)
        .collect();
    let not_given_by_the_pool_checker: Vec<&String> = reasons_in_the_table
        .difference(&reasons_in_the_source)
        .collect();
    assert!(
        missing_from_the_table.is_empty() && not_given_by_the_pool_checker.is_empty(),
        "结局表与池级 checker 源码里 I-1.13 能报的不适用理由逐句相等（源码里认出 {} 句、表里 {} 句）。\n表里缺的：{missing_from_the_table:#?}\n池级 checker 报不出的：{not_given_by_the_pool_checker:#?}",
        reasons_in_the_source.len(),
        reasons_in_the_table.len()
    );
    assert!(
        is_judged_in_the_source,
        "走读那一族里有判 I-1.13 成立或违例的那一处（结局表的成立、违例两行对着它）"
    );
    for verdict in [
        ScanRebuildVerdictOfThePoolChecker::Holds,
        ScanRebuildVerdictOfThePoolChecker::Violated,
    ] {
        assert_eq!(
            SCAN_REBUILD_OUTCOMES
                .iter()
                .filter(|outcome| outcome.verdict_of_the_pool_checker == verdict)
                .count(),
            1,
            "结局表里 {verdict:?} 恰好一行"
        );
    }
}

/// 场景：两块盘的池，崩溃镜像上盘 0 一个自证过的系统配置槽都没有、盘 1 两槽都在；基镜像上两块盘都有。
/// 预期：判段那一判记盘 0 是新出现的死盘。
#[test]
fn device_dead_on_the_crash_image_and_alive_on_the_base_image_is_reported_new() {
    assert_eq!(
        devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts(
            &[(0, 0), (1, 2)],
            &[(0, 2), (1, 2)],
        ),
        DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage::New(vec![0]),
        "盘 0 在基镜像上还有自证过的槽、崩溃镜像上一个都没有"
    );
}

/// 场景：基镜像上盘 1 已经两槽全废（补写流的起点），崩溃镜像上它照旧全废、盘 0 只剩一槽。
/// 预期：盘 1 不是新出现的，盘 0 还有一槽，判段那一判不记。
#[test]
fn device_already_dead_on_the_base_image_is_not_reported_again() {
    assert_eq!(
        devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts(
            &[(0, 1), (1, 0)],
            &[(0, 2), (1, 0)],
        ),
        DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage::NoneNew,
        "盘 1 在基镜像上就两槽全废，盘 0 还有一槽自证过"
    );
}

/// 场景：崩溃镜像上两块盘都一个自证过的槽都没有（全池皆废），基镜像上都有。
/// 预期：交 I-8.5，判段那一判不重记。
#[test]
fn every_device_dead_on_the_crash_image_is_left_to_the_tail_slot_invariant() {
    assert_eq!(
        devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts(
            &[(0, 0), (1, 0)],
            &[(0, 2), (1, 2)],
        ),
        DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage::EveryDeviceLeftToTheTailSlotInvariant,
        "全池皆废留给 I-8.5"
    );
}
