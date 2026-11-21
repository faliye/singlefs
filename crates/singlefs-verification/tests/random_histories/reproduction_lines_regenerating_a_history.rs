//! 判红时打出的复现行喂回读环境的函数，重新生成同一段历史与同样的执行。

use super::*;

/// 照「收缩一个种子」那条入口（example `shrink_one_failing_random_history_seed`）的读法读环境、生成要复现的那段历史：
/// 种子（`SINGLEFS_RANDOM_HISTORY_SHRINK_SEED`）必给，取样参数经 `random_history_to_reproduce_from_environment` 读
/// （变量与判红时打的复现行同一张表），没设的取默认批的（`RANDOM_HISTORY_DEFAULT_SAMPLING`）。入口与「复现行喂回入口」那几条自证
/// 经同一个库函数、同一份默认值、同一个 `generate`：自证红了，就是照抄复现行起入口跑不出原来那一段。
fn history_to_reproduce_from(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> (GeneratedHistory, HistorySampling) {
    let (seed, sampling) =
        random_history_to_reproduce_from_environment(lookup, &DEFAULT_BATCH_SAMPLING)
            .unwrap_or_else(|error| panic!("{}", error.render()));
    (sampling.generate(seed), sampling)
}

/// 判红时对一段取样会打的那一行复现命令（`NewFindingReport::render` 里「复现：」那一行；造一个假发现，不必真红），
/// 原样喂回复现入口的读法（`history_to_reproduce_from`，与「收缩一个种子」那个 example 同一个库函数）：生成的历史逐步与原趟相同，跑法也相同。
/// 种子取种子基之后第 5 个（取样区间里任一个都行，只要不是 0）。
fn the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
    sampling: HistorySampling,
) {
    let seed = HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 5);
    let original = sampling.generate(seed);
    let rendered = NewFindingReport {
        signature: FailureSignature::HarnessJudgement {
            judgement: "自证用的假发现",
        },
        first_seed: seed,
        observation: FailureObservation {
            position: StepPosition::Operation(3),
            operation_kind: Some(original.operations[3].kind()),
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        },
        seeds: vec![seed],
        shrunk: None,
        sampling,
    }
    .render();
    let reproduction_lines: Vec<&str> = rendered
        .lines()
        .filter(|line| line.contains("SINGLEFS_RANDOM_HISTORY_SHRINK_SEED="))
        .collect();
    assert_eq!(
        reproduction_lines.len(),
        1,
        "判红的报告里恰好一行复现命令：\n{rendered}"
    );
    let assignments = environment_assignments_in_line(reproduction_lines[0]);
    let (reproduced, reproduced_sampling) =
        history_to_reproduce_from(&|variable: &str| assignments.get(variable).cloned());
    assert_eq!(
        reproduced.starting_point, original.starting_point,
        "复现出来的起点不同；复现行：{}",
        reproduction_lines[0]
    );
    let first_differing_step = original
        .operations
        .iter()
        .zip(&reproduced.operations)
        .position(|(original_step, reproduced_step)| original_step != reproduced_step);
    assert_eq!(
        (first_differing_step, reproduced.operations.len()),
        (None, original.operations.len()),
        "复现出来的历史与原趟逐步比：第一个不同的步、步数；复现行：{}",
        reproduction_lines[0]
    );
    assert_eq!(reproduced, original);
    assert_eq!(
        reproduced_sampling.execution, sampling.execution,
        "复现出来的跑法（checker、盘宽、空间准入）不同；复现行：{}",
        reproduction_lines[0]
    );
    assert_eq!(reproduced_sampling, sampling);
}

#[test]
fn reproduction_line_of_the_default_batch_regenerates_the_same_history_and_execution() {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        DEFAULT_BATCH_SAMPLING,
    );
}

#[test]
fn reproduction_line_of_the_reuse_sampling_regenerates_the_same_history_and_execution() {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        REUSE_SAMPLING,
    );
}

#[test]
fn reproduction_line_of_the_rollback_sampling_regenerates_the_same_history_and_execution() {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        ROLLBACK_SAMPLING,
    );
}

#[test]
fn reproduction_line_of_the_allocation_record_wall_sampling_regenerates_the_same_history_and_execution(
) {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        WALL_SAMPLING,
    );
}

#[test]
fn reproduction_line_of_the_unit_area_wall_sampling_with_the_space_admission_skipped_regenerates_the_same_history_and_execution(
) {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_SKIPPED,
    );
}

#[test]
fn reproduction_line_of_the_unit_area_wall_sampling_with_the_space_admission_judged_regenerates_the_same_history_and_execution(
) {
    the_reproduction_line_printed_for_a_finding_regenerates_the_same_history_and_execution(
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED,
    );
}
