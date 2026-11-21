//! 并进给定状态引擎池级 checker 结论的历史：池级 checker 结论从不判红时只剩 harness 自己判的，引擎判红的步镜像让并起来的历史停在那一步。

use super::*;

/// 合并入口那两条用例用的那一段历史与注入次数：broad 那组比重、种子基那一段 8 步，每段抽 6 次。
const MERGED_HISTORY_OPERATIONS: usize = 8;

const MERGED_FAULTS_PER_HISTORY: usize = 6;

/// 合并入口那一条用例里，用例替给定状态引擎判红的那一条不变量（真名，归类只看名字）与它的细节。
const INVARIANT_THE_TEST_JUDGES_RED: &str = "I-2.1";

/// 用例替给定状态引擎判的结论：起点之后的每一步（`StepPosition::Operation`）的镜像一律判红 [`INVARIANT_THE_TEST_JUDGES_RED`]，
/// 起点那一张不红；重开那张盘就是录制流整条，历史走到过起点之后写过盘的一步就红（同一个判法：那张盘是那一步镜像的延续）。
/// 被吞的写插回去那张盘不红。
fn verdicts_red_after_the_starting_point(
    images: &ImagesAfterAFault,
) -> PoolCheckerVerdictsAfterAFault {
    let red = || {
        vec![(
            INVARIANT_THE_TEST_JUDGES_RED,
            "用例替给定状态引擎判红".to_string(),
        )]
    };
    let violations_of_the_step_images: Vec<Vec<(&'static str, String)>> = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .map(|step_image| match step_image.position {
            StepPosition::StartingPoint => Vec::new(),
            StepPosition::Operation(_) => red(),
        })
        .collect();
    let reached_a_step_after_the_starting_point = violations_of_the_step_images
        .iter()
        .any(|violations| !violations.is_empty());
    PoolCheckerVerdictsAfterAFault {
        violations_of_the_step_images,
        violations_after_the_fault: if reached_a_step_after_the_starting_point {
            red()
        } else {
            Vec::new()
        },
        invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet::new(),
    }
}

/// 合并入口收到的结论一条都不红：交回的与 harness 不判池级 checker、把盘交出去的那一路（[`inject_faults_into_history`]）逐项相同，
/// 只差「交出去几次」那一格（合并入口交给判的不记在那一格）。每个注入点只交一次判。
/// 判别力：合并入口在不红的结论上也截短重跑、或把结论并错地方，交回的与那一路不同；交判的次数不是注入点数。
#[test]
fn merging_pool_checker_verdicts_that_never_turn_red_returns_what_the_harness_alone_judges() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        MERGED_HISTORY_OPERATIONS,
        &GenerationWeights::BROAD,
    );
    let mut hand_outs = 0usize;
    let mut merged =
        inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine(
            &history,
            HANDING_EVERY_STEP_IMAGE_OUT,
            MERGED_FAULTS_PER_HISTORY,
            &mut |images| {
                hand_outs += 1;
                PoolCheckerVerdictsAfterAFault {
                    violations_of_the_step_images: vec![
                        Vec::new();
                        images
                            .step_images_of_the_run_with_the_fault
                            .len()
                    ],
                    violations_after_the_fault: Vec::new(),
                    invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet::new(),
                }
            },
        );
    let mut handed_out = inject_faults_into_history(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        MERGED_FAULTS_PER_HISTORY,
    );
    assert!(!merged.drawn_faults.is_empty(), "前提：这一段摆出了注入点");
    assert_eq!(
        hand_outs,
        merged.drawn_faults.len(),
        "结论都不红时每个注入点只交一次判"
    );
    assert_eq!(
        (
            merged.tally.images_after_the_fault_handed_out,
            handed_out.tally.images_after_the_fault_handed_out
        ),
        (
            0,
            u64::try_from(handed_out.drawn_faults.len()).expect("装得进 u64")
        ),
        "合并入口交给判的不记在「交出去几次」那一格，交出去的那一路每个注入点记一次"
    );
    merged.tally.images_after_the_fault_handed_out = 0;
    handed_out.tally.images_after_the_fault_handed_out = 0;
    assert_eq!(
        merged, handed_out,
        "结论一条都不红时合并入口交回的与 harness 自己判的那一路逐项相同"
    );
}

/// 合并入口收到某个注入点上一张每步镜像判红：照判红就停那一档的口径，历史停在那一步——这个注入点截短重跑一遍，重跑那一遍交出去的每步镜像
/// 就是整段那一遍到判红那一张为止（含）的那几张、重开的正是那一张的盘，再交一次判；新发现里的那一条停在判红那一步、违例是判红的那一条。
/// 用例替给定状态引擎判（[`verdicts_red_after_the_starting_point`]：起点之后的每一步都红），逐次记它被交了什么。
/// 判别力：不截短（`crates/mutations.tsv` 同名那一行把找第一张判红的那一处改成永远找不到），第二次交判没了、新发现落在注入点那一段而不是判红那一步。
#[test]
fn a_step_image_the_given_states_engine_judges_red_stops_the_merged_history_at_that_step() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        MERGED_HISTORY_OPERATIONS,
        &GenerationWeights::BROAD,
    );
    let mut asked: Vec<ImagesAfterAFault> = Vec::new();
    let merged =
        inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine(
            &history,
            HANDING_EVERY_STEP_IMAGE_OUT,
            MERGED_FAULTS_PER_HISTORY,
            &mut |images| {
                asked.push(images.clone());
                verdicts_red_after_the_starting_point(images)
            },
        );
    let mut asked_in_order = asked.iter();
    let mut stopped_positions: Vec<StepPosition> = Vec::new();
    for drawn in &merged.drawn_faults {
        let run_through = asked_in_order
            .next()
            .unwrap_or_else(|| panic!("{} 没交判", drawn.render()));
        assert_eq!(run_through.drawn, *drawn, "交判的次序就是注入点的次序");
        let Some(first_red) = run_through
            .step_images_of_the_run_with_the_fault
            .iter()
            .position(|step_image| matches!(step_image.position, StepPosition::Operation(_)))
        else {
            continue;
        };
        let stopped = asked_in_order.next().unwrap_or_else(|| {
            panic!(
                "{} 的第 {first_red} 张每步镜像判红，要截短重跑、再交一次判，没交",
                drawn.render()
            )
        });
        assert_eq!(stopped.drawn, *drawn, "截短重跑的是同一个注入点");
        assert_eq!(
            stopped.step_images_of_the_run_with_the_fault.as_slice(),
            &run_through.step_images_of_the_run_with_the_fault[..=first_red],
            "{}：截短重跑那一遍停在判红那一张",
            drawn.render()
        );
        assert_eq!(
            stopped.image_after_the_fault.operations.len(),
            run_through.step_images_of_the_run_with_the_fault[first_red].operations_recorded,
            "{}：截短重跑那一遍重开的就是判红那一步的盘",
            drawn.render()
        );
        stopped_positions
            .push(run_through.step_images_of_the_run_with_the_fault[first_red].position);
    }
    assert!(
        asked_in_order.next().is_none(),
        "交判的只有每个注入点整段那一遍与判红时截短那一遍"
    );
    assert!(
        !stopped_positions.is_empty(),
        "前提：至少一个注入点上历史走到了起点之后写过盘的一步（有判红的每步镜像）"
    );
    let red_signature = FailureSignature::CheckerViolations {
        invariants: vec![INVARIANT_THE_TEST_JUDGES_RED],
    };
    let red_finding = merged
        .new_findings
        .iter()
        .find(|finding| finding.signature == red_signature)
        .unwrap_or_else(|| panic!("判红那一条要成新发现：{:?}", merged.new_findings));
    assert!(
        stopped_positions.contains(&red_finding.observation.position),
        "新发现停在判红的那一步（{:?} 之一），不在注入点那一段：{:?}",
        stopped_positions,
        red_finding.observation.position
    );
    assert_eq!(
        red_finding
            .observation
            .violations
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect::<Vec<_>>(),
        vec![INVARIANT_THE_TEST_JUDGES_RED],
        "停下那一步的违例就是判红的那一条"
    );
}
