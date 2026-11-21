//! `evaluation_of_a_history` 的单测。
use super::super::tests::{
    arm_of_this_tree, breakage_of_this_process, FIRST_FILE_INODE_NUMBER_IN_THE_CLAUSE,
    FIRST_FILE_OBJECT_BIRTH_IN_THE_CLAUSE, WATERMARK_AFTER_THE_FIRST_FILE_IN_THE_CLAUSE,
};
use super::*;

fn issued_numbers(evaluation: &HistoryEvaluation) -> Vec<Vec<u64>> {
    evaluation
        .account
        .creation_issues
        .iter()
        .map(|issue| issue.issued_numbers.clone())
        .collect()
}

/// A1、A2：第一个文件之后（D1 的起点），S1 读到的现行那一版 inode 树恰好 {(1, 3)}、水位恰好 2（四条臂都成立：
/// V1 只改回退、V2 只在有新建 inode 的发布上少加）。
#[test]
fn first_file_alone_reads_inode_one_born_at_three_and_watermark_two() {
    let evaluation = evaluate_history(
        &history_item(Family::Directed, 0),
        breakage_of_this_process(),
    );
    assert_eq!(evaluation.ending, "completed");
    let (position, watermark, births) = &evaluation.account.current_trajectory[0];
    assert_eq!(position, "start");
    assert_eq!(
        births,
        &vec![(
            FIRST_FILE_INODE_NUMBER_IN_THE_CLAUSE,
            FIRST_FILE_OBJECT_BIRTH_IN_THE_CLAUSE
        )],
        "A1"
    );
    assert_eq!(
        *watermark,
        Some(WATERMARK_AFTER_THE_FIRST_FILE_IN_THE_CLAUSE),
        "A2"
    );
    assert_eq!(evaluation.account.counts.issuances, 1, "起点只发了 inode 1");
}

/// D2(k) 在这一臂上的钉值（A3–A6）：一条带参数的函数，k = 1..=6 各一条用例。
fn rollback_then_create_matches_the_arm(rollback_index: u64) {
    let arm = arm_of_this_tree();
    let evaluation = evaluate_history(
        &history_item(Family::Directed, rollback_index),
        breakage_of_this_process(),
    );
    let account = &evaluation.account;
    let first = WATERMARK_AFTER_THE_FIRST_FILE_IN_THE_CLAUSE;
    let issued = issued_numbers(&evaluation);
    println!(
    "D2({rollback_index}) arm={} ending={} issued={issued:?} rollbacks_applied={} rollback_targets={:?} q1={} q3b={}",
    arm.label(),
    evaluation.ending,
    account.counts.rollbacks_applied,
    account.rollback_targets,
    evaluation.same_timeline_reissues(),
    evaluation.reissues_across_abandoned_timelines()
);
    assert!(
        issued.len() >= 2,
        "D2({rollback_index}) 前两次建 inode 都要做成：{issued:?}"
    );
    match arm {
        Arm::Product
        | Arm::RollbackTakesOnlyTheRingsWatermark
        | Arm::RollbackTakesTheTargetsWatermark => {
            assert_eq!(
                issued[..2],
                [vec![first], vec![first + 1]],
                "A4：前两次发 2、3"
            );
        }
        Arm::PublishDoesNotAddTheNewInodes => {
            assert_eq!(
                issued[..2],
                [vec![first], vec![first]],
                "A6：水位不加新建个数，两次都发 2"
            );
        }
    }
    let rollback_applied = account.counts.rollbacks_applied == 1;
    if rollback_index == 1 {
        assert!(
            rollback_applied,
            "D2(1) 回退到第一次建 inode 那一版，要做成"
        );
    }
    if rollback_index == 1 {
        assert_eq!(
            issued.len(),
            3,
            "D2(1) 回退之后那次建 inode 要做成（目标那一版第一个文件已删完）：{issued:?}"
        );
    }
    // 回退到第一个文件还活着的那一版时，之后那次建 inode 前提不满足、执行器不调（只在第一个文件已删完的一版上建），
    // A3 / A5 这一格没有样本；只在它做成时钉。
    if rollback_applied && issued.len() == 3 {
        match arm {
            Arm::Product | Arm::RollbackTakesOnlyTheRingsWatermark => {
                assert_eq!(issued[2], vec![first + 2], "A3：回退取 max，不退回去，发 4");
            }
            Arm::RollbackTakesTheTargetsWatermark => {
                let target_watermark = account.rollback_targets[0]
                    .1
                    .expect("回退目标那一版 S1 读得到水位");
                assert_eq!(
                    issued[2],
                    vec![target_watermark],
                    "A5：发的号 = 目标那一版的水位"
                );
                if rollback_index == 1 {
                    assert_eq!(
                        target_watermark,
                        first + 1,
                        "A5：目标是第一次建 inode 那一版，水位 3"
                    );
                }
            }
            Arm::PublishDoesNotAddTheNewInodes => {
                assert_eq!(issued[2], vec![first], "水位一直是 2");
            }
        }
    }
    match arm {
        Arm::Product | Arm::RollbackTakesOnlyTheRingsWatermark => {
            assert_eq!(
                evaluation.same_timeline_reissues(),
                0,
                "P 臂上 D2({rollback_index}) 不该重发号"
            );
        }
        Arm::RollbackTakesTheTargetsWatermark => {
            if rollback_index == 1 {
                assert_eq!(evaluation.same_timeline_reissues(), 1, "A5：Q1 恰好 1");
                assert_eq!(
                    evaluation.reissues_across_abandoned_timelines(),
                    0,
                    "A5：Q3b 恰好 0"
                );
                let event = account
                    .events
                    .iter()
                    .find(|event| event.same_timeline)
                    .expect("Q1 那个事件");
                assert_eq!(
                    (event.inode, event.position.as_str()),
                    (first + 1, "op4"),
                    "A5：第三次建发的 3"
                );
                assert_eq!(
                    event.older_birth_candidacy,
                    OlderBirthStillInTheCandidateSet::Still,
                    "A5：Q9 判「还在」"
                );
                assert_eq!(
                    event.one_birth_per_number_at_this_step,
                    VerdictClass::Violated,
                    "A5：那一步 I-9.29 红"
                );
            }
        }
        Arm::PublishDoesNotAddTheNewInodes => {
            if rollback_index == 1 {
                let at_second_creation = account
                    .events
                    .iter()
                    .filter(|event| event.same_timeline && event.position == "op2")
                    .count();
                assert_eq!(at_second_creation, 1, "A6：第二次建那一步 Q1 恰好 1");
                let after_first_creation = account
                    .step_verdicts
                    .iter()
                    .find(|verdicts| verdicts.position == "op1")
                    .expect("第一次建之后那一步有判定");
                assert_eq!(
                    after_first_creation.watermark_above_both_highest_numbers,
                    VerdictClass::Violated,
                    "A6：第一次建之后那一步 I-9.6 红"
                );
            }
        }
    }
}

#[test]
fn rollback_one_step_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(1);
}
#[test]
fn rollback_two_steps_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(2);
}
#[test]
fn rollback_three_steps_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(3);
}
#[test]
fn rollback_four_steps_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(4);
}
#[test]
fn rollback_five_steps_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(5);
}
#[test]
fn rollback_six_steps_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(6);
}

/// A4（D3）：建、挂载、建、冷启动、挂载、建：P、V1、V717 依次发 2、3、4；V2 三次都发 2。
#[test]
fn creations_across_mounts_and_a_cold_start_match_the_arm() {
    let arm = arm_of_this_tree();
    let evaluation = evaluate_history(
        &history_item(Family::Directed, 7),
        breakage_of_this_process(),
    );
    let first = WATERMARK_AFTER_THE_FIRST_FILE_IN_THE_CLAUSE;
    let issued = issued_numbers(&evaluation);
    println!(
        "D3 arm={} ending={} issued={issued:?}",
        arm.label(),
        evaluation.ending
    );
    let expected = match arm {
        Arm::Product
        | Arm::RollbackTakesTheTargetsWatermark
        | Arm::RollbackTakesOnlyTheRingsWatermark => {
            vec![vec![first], vec![first + 1], vec![first + 2]]
        }
        Arm::PublishDoesNotAddTheNewInodes => vec![vec![first], vec![first], vec![first]],
    };
    assert_eq!(issued, expected, "A4（D3）");
    assert!(
        evaluation.account.counts.mounts_or_recoveries_applied >= 2,
        "两次挂载要做成"
    );
}

/// 多线程与单线程对拍（登记第十一节「合并不确定」）：F3a 前 12 段，1 个线程与 3 个线程的结果行（去掉挂钟）逐字相同。
#[test]
fn one_thread_and_three_threads_give_the_same_lines() {
    let items: Vec<HistoryItem> = (0..12)
        .map(|index| history_item(Family::DirectedExhaustive, index))
        .collect();
    let lines_with = |worker_threads: usize| {
        let mut lines = Vec::new();
        evaluate_items(&items, worker_threads, None, Breakage::NONE, |evaluation| {
            let line = evaluation.history_line(Arm::Product);
            lines.push(
                line.split(' ')
                    .filter(|field| !field.starts_with("elapsed_ns="))
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            lines.extend(evaluation.event_lines(Arm::Product));
        });
        lines
    };
    assert_eq!(lines_with(1), lines_with(3));
}

/// F1 四组与几何四点每一族在序号 0 上走一遍（变异取样第四类：取样点要覆盖大规模会跑到的每一族），结果行写得完整、两条被数的不变量每一步都有判定。
/// 一条带参数的函数，每族一条用例。
fn first_history_of_the_family_yields_a_complete_line(family: Family) {
    let arm = arm_of_this_tree();
    let item = history_item(family, 0);
    let evaluation = evaluate_history(&item, Breakage::NONE);
    let counts = &evaluation.account.counts;
    println!(
    "{} arm={} ending={} steps_observed={} issuances={} creations_applied={} rollbacks_applied={} q1={} q11={} s3_disagreements={}",
    family.label(),
    arm.label(),
    evaluation.ending,
    counts.steps_observed,
    counts.issuances,
    counts.creations_applied,
    counts.rollbacks_applied,
    evaluation.same_timeline_reissues(),
    counts.creations_not_issuing_the_previous_watermark,
    counts.abandonment_path_disagreements
);
    assert!(evaluation.history_line(arm).ends_with(" line_end=ok"));
    assert!(counts.steps_observed >= 1, "起点那一步要观察到");
    assert_eq!(
        counts.one_birth_per_number.missing, 0,
        "I-9.29 每一步都要有判定"
    );
    assert_eq!(
        counts.watermark_above_both_highest_numbers.missing, 0,
        "I-9.6 每一步都要有判定"
    );
}

#[test]
fn first_history_of_broad_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::ExistingRandom(
        ExistingWeightGroup::Broad,
    ));
}
#[test]
fn first_history_of_reuse_after_raising_the_floor_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::ExistingRandom(
        ExistingWeightGroup::ReuseAfterRaisingTheFloor,
    ));
}
#[test]
fn first_history_of_deleting_files_of_several_lower_leaves_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::ExistingRandom(
        ExistingWeightGroup::DeletingFilesOfSeveralLowerLeaves,
    ));
}
#[test]
fn first_history_of_newest_system_configuration_slot_reading_zeros_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::ExistingRandom(
        ExistingWeightGroup::NewestSystemConfigurationSlotOfEachDeviceReadingZeros,
    ));
}
#[test]
fn first_history_of_longer_random_with_rollbacks_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::Geometry(
        GeometryPoint::LongerRandomWithRollbacks,
    ));
}
#[test]
fn first_history_of_random_with_rollbacks_on_small_devices_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::Geometry(
        GeometryPoint::RandomWithRollbacksOnSmallDevices,
    ));
}
#[test]
fn first_history_of_random_with_rollbacks_drawing_recent_uniform_or_beyond_yields_a_complete_line()
{
    first_history_of_the_family_yields_a_complete_line(Family::Geometry(
        GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond,
    ));
}
#[test]
fn first_history_of_longer_broad_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::Geometry(
        GeometryPoint::LongerBroad,
    ));
}
