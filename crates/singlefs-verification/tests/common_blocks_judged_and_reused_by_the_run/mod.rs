//! 判完的那一趟崩溃放量「判齐、核过、再跑一趟全部复用」的断言，固定脚本两条放量（到回退、到卸载）共用这一份：
//! 没设块区间（`singlefs_verification::crash_amplification::BLOCK_RANGE_ENVIRONMENT_VARIABLE`）照整条流判；
//! 设了块区间的那一趟只判这一片：这一片的块数与状态数照测试自己排的计划数（[`blocks_and_states_of_the_block_range`]），
//! 「这一趟把这一片录入时没判的状态都判了」「再跑一趟这一片全部复用、一块不再核」照这一片判；
//! 「判过的块第 ② 段都核过」数的是整条流（第 ③ 段比对走整条流的块），这一片判不了，打一行说明，留给几片都判完之后不设块区间的那一趟。
#![allow(
    dead_code,
    reason = "共用的断言模块：每个测试文件只用其中几样，没用的在那个测试二进制里就是没用到"
)]

use singlefs_verification::crash_amplification::{BlockRangeOfTheFlow, CrashPlan, EnvironmentRun};

/// 设了块区间、「判过的块第 ② 段都核过」这一片判不了时打的那一行的行首。
pub const BLOCKS_WITHOUT_VERIFIER_LEFT_TO_THE_RUN_WITHOUT_A_BLOCK_RANGE_LINE_PREFIX: &str =
    "CRASH_AMPLIFICATION_BLOCKS_WITHOUT_VERIFIER_LEFT_TO_THE_RUN_WITHOUT_A_BLOCK_RANGE";

/// 一片在计划里罩的块数与状态数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlocksAndStatesOfTheBlockRange {
    pub blocks: u64,
    pub states: u64,
}

/// `block_range` 在计划 `plan` 里罩几块、几个状态：块序号照整条流按点的次序编号（与引擎选块同一种编号），
/// 整条流（[`BlockRangeOfTheFlow::EveryBlockOfTheFlow`]）罩全部的块。
#[must_use]
pub fn blocks_and_states_of_the_block_range(
    plan: &CrashPlan,
    block_range: &BlockRangeOfTheFlow,
) -> BlocksAndStatesOfTheBlockRange {
    let mut covered = BlocksAndStatesOfTheBlockRange {
        blocks: 0,
        states: 0,
    };
    let mut block_sequence = 0u64;
    for point in &plan.points {
        for states in point.blocks() {
            if block_range.holds(block_sequence) {
                covered.blocks += 1;
                covered.states += states.end - states.start;
            }
            block_sequence += 1;
        }
    }
    covered
}

/// 判完的那一趟（不是只录、只排派活计划、被叫停的那一趟）：整份自己跑的（`share.count == 1`）对账为空、库里没有红，
/// 这一趟把录入时没判的状态都判了，再跑一趟（`run_again`）全部复用、一块不再核；第 ②③ 段跑了的，核对红而判器绿一条都不许，
/// 整份自己跑的判过的块第 ② 段都核过。设了块区间的那一趟照这一片判（模块文件头）；`plan` 是测试自己照同一条流排的计划。
///
/// # Panics
/// 上面哪一条不成立；设了块区间而 `plan` 的块数与引擎读回的计划不同（块序号对不上，这一片数不出来）。
#[track_caller]
pub fn assert_the_run_judged_verified_and_reused_its_blocks(
    run: &EnvironmentRun,
    plan: &CrashPlan,
    run_again: impl FnOnce() -> EnvironmentRun,
) {
    if run.share.count == 1 {
        assert_eq!(
            run.unjudged_blocks, 0,
            "整份自己跑（两台各判一份时，两份都判进服务那一份之后不分片的那一趟就是整份），对账为空"
        );
        assert!(run.violations.is_empty());
    }
    if run.share.count == 1 {
        match &run.block_range {
            BlockRangeOfTheFlow::EveryBlockOfTheFlow => {
                // 库里这一判法单元指纹已经判过的块复用、不重核（续跑）：这一趟核的是它录下来还没判的那些
                assert_eq!(
                    run.checking.states_checked, run.recording.states_recorded,
                    "这一趟录下来要判的状态都核了"
                );
                let rerun = run_again();
                assert_eq!(
                    rerun.recording.blocks_already_judged,
                    run.recording.blocks_recorded + run.recording.blocks_already_judged,
                    "第二趟全部复用"
                );
                assert_eq!(rerun.checking.blocks_checked, 0, "复用的块不再核");
            }
            BlockRangeOfTheFlow::BlocksInTheRange(range) => {
                let covered = blocks_and_states_of_the_block_range(plan, &run.block_range);
                let whole_flow = blocks_and_states_of_the_block_range(
                    plan,
                    &BlockRangeOfTheFlow::EveryBlockOfTheFlow,
                );
                assert_eq!(
                    whole_flow.blocks, run.blocks_of_the_flow,
                    "测试排的计划与引擎读回的计划块数相同（块区间按块序号点块，两边的编号要对得上）"
                );
                // 录入段的账（`run.recording`）数的是整条流；这一片录入时已判的是 `states_reused`，没判的这一趟都要判
                assert_eq!(
                    run.checking.states_checked + run.states_reused,
                    covered.states,
                    "块区间 {}-{}：这一片 {} 块 {} 个状态，录入时没判的这一趟都核了",
                    range.start,
                    range.end,
                    covered.blocks,
                    covered.states
                );
                let rerun = run_again();
                assert_eq!(
                    rerun.block_range, run.block_range,
                    "再跑的那一趟判的是同一片"
                );
                assert_eq!(
                    rerun.states_reused, covered.states,
                    "块区间 {}-{}：再跑一趟，这一片 {} 块全部复用",
                    range.start, range.end, covered.blocks
                );
                assert_eq!(
                    rerun.checking.blocks_checked, 0,
                    "块区间 {}-{}：复用的块不再核",
                    range.start, range.end
                );
            }
        }
    }
    // 第 ②③ 段跑了就要：核对红 ⇒ 判器红，一条不一致都不许
    if let Some(comparison) = &run.comparison {
        assert_eq!(
            comparison.disagreements, 0,
            "核对红而判器绿：{:?}",
            comparison.disagreement_samples
        );
        // 两台各判一份的那一趟：服务那一份里有对方判过、第 ② 段归对方核而对方还没核到的块，这一份不判；
        // 两份都判进服务那一份之后不分片再跑的那一趟（share.count == 1）罩整条流，判过的块都要核过
        if run.share.count == 1 {
            match &run.block_range {
                BlockRangeOfTheFlow::EveryBlockOfTheFlow => {
                    assert_eq!(
                        comparison.blocks_without_verifier, 0,
                        "判过的块第 ② 段都核过"
                    );
                }
                // 第 ③ 段比对数整条流的块：别的片判过、第 ② 段还没核到的块（那一片还在跑，或判完之前被停了）也算在里面，
                // 这一片判不了；这一片自己的块第 ② 段都核（核对照这台机器领的块逐块核，核过的不再核）
                BlockRangeOfTheFlow::BlocksInTheRange(range) => {
                    println!(
                        "{BLOCKS_WITHOUT_VERIFIER_LEFT_TO_THE_RUN_WITHOUT_A_BLOCK_RANGE_LINE_PREFIX} block_range={}-{} blocks_without_verifier={}：整条流判过而第 ② 段没核的块，这一片判不了，留给几片都判完之后不设块区间的那一趟（第 ② 段补核、照整条流判 0）",
                        range.start, range.end, comparison.blocks_without_verifier
                    );
                }
            }
        }
    }
}
