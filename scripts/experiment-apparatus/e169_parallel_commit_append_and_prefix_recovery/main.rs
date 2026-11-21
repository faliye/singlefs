//! E169（并行提交的追加纪律与前缀恢复）第一段（不带参数）：Q1 的「串落」「同发」两个条件下，
//! Q2 四条追加臂 × 三族历史 × 两种「已追加」读法 × Q3 四条恢复臂的崩溃枚举；
//! 第二段（参数 `segment-two`）：Q1 的「多飞·根序」「多飞·根乱」两个条件下同样的量，k = 2 直接枚举执行、k = 3 按配置去重。
//! 跑前登记 `research/prompts/e169-preregistration.md`（含第十二节修订）；岔路单是
//! `research/prompts/m3-concurrency-questions.md` 的 Q2、Q3 两行。
//!
//! 独立手写模型：不与 `crates/` 共用代码、不引 `crates/` 的常量（`.claude/rules/implementation-first.md` 第 4 条）。
//! 丙0 是 `crates/singlefs-replay-prefix/src/lib.rs` 第 422–534 行逐判的转写，每一判旁边写那一行的行号；
//! 登记第七节 A2 的十四条固定记录序列（停机条款 T1）在开跑之前逐条核，有一条对不上就停、其余不跑。
//!
//! 崩溃状态照登记第一节 1.2：一个执行的写按屏障与 FUA 切段（D13（验证路线） 已定项 4），
//! 崩溃点 = （已完成的最后一道闭段点，本段里已发出到哪一个操作），持久集合是已发出的本段写各取各的态；
//! 已返回 fsync 的事务集合按崩溃点现算（修订 ④ 补的 R1 读法让返回可以落在两个闭段点之间，所以按发出前缀切）。
//! 同一格里全部执行的崩溃状态按内容去重。

// admission: always 每次照此刻的仓与自己的常量现算，上一次跑出的产物不替这一次作保
// run-condition: command python3
use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

mod cell_plans_executions_and_crash_states;
mod grid_and_cell_emission;
mod histories_arms_and_record_streams;
mod multi_flight_templates_and_search;
mod prefix_recovery_and_metrics;
mod segment_runs_and_anchors;
mod wait_cost_emission_and_replay_inputs;
mod wait_cost_model;

use self::cell_plans_executions_and_crash_states::*;
use self::grid_and_cell_emission::*;
use self::histories_arms_and_record_streams::*;
use self::multi_flight_templates_and_search::*;
use self::prefix_recovery_and_metrics::*;
use self::segment_runs_and_anchors::*;
use self::wait_cost_emission_and_replay_inputs::*;
use self::wait_cost_model::*;

/// D23（journal 的角色与格式） 已定项 18 条文：默认环下 196608 ÷ 3 = 65536
/// （抄自 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 18，登记第二节第 266 行原样抄录）。
const IN_FLIGHT_RECORD_LIMIT: usize = 65_536;
/// 登记第七节 A6：k = 3、r = 2 时严格连续段至多 6 条。
const STRICT_SEGMENT_LENGTH_BOUND: usize = 6;
/// 历史开始之前的根：txg 0，恒持久，覆盖到 jsn 0（登记第一节 1.2「恢复」）。
const START_ROOT_TXG: u64 = 0;
/// 系统配置轮换写是原地覆写，取三态（D13（验证路线） 已定项 4）。
const SYSTEM_CONFIGURATION_DURABILITY: WriteDurabilityStates = WriteDurabilityStates::NewMissingOrTorn;
/// 根槽写带 FUA：关掉它所在的那一段（D13（验证路线） 已定项 4）。
const ROOT_SLOT_WRITE_CLOSES_ITS_SEGMENT: bool = true;

/// 带 `segment-three` 跑第三段（fsync 等待模型）；带 `replay-core-inputs` 打出第一、二段每个崩溃状态的恢复输入与模型丙0 的结果。
const SEGMENT_THREE_ARGUMENT: &str = "segment-three";
const REPLAY_CORE_INPUTS_ARGUMENT: &str = "replay-core-inputs";

/// 不带参数依次跑第一、二、三段（三段的产物依次接在一起，每段自己的收尾行照打；第一段停机条款打中就退 3、不往下）；
/// 带 `segment-one` 只跑第一段（`research/results/e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out` 那一份）；带 `segment-two` 只跑第二段（多飞）。
const SEGMENT_ONE_ARGUMENT: &str = "segment-one";
const SEGMENT_TWO_ARGUMENT: &str = "segment-two";

/// 命令行参数选哪几段。不带参数是第一、二、三段依次全跑；段名是指定范围；`replay-core-inputs` 是子集。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentChoice {
    One,
    Two,
    Three,
    ReplayCoreInputs,
}

/// 认不出的参数返回 `None`，由 `main` 退 2。
fn segments_for_arguments(arguments: &[String]) -> Option<Vec<SegmentChoice>> {
    match arguments {
        [] => Some(vec![SegmentChoice::One, SegmentChoice::Two, SegmentChoice::Three]),
        [segment] if segment == SEGMENT_ONE_ARGUMENT => Some(vec![SegmentChoice::One]),
        [segment] if segment == SEGMENT_TWO_ARGUMENT => Some(vec![SegmentChoice::Two]),
        [segment] if segment == SEGMENT_THREE_ARGUMENT => Some(vec![SegmentChoice::Three]),
        [mode] if mode == REPLAY_CORE_INPUTS_ARGUMENT => Some(vec![SegmentChoice::ReplayCoreInputs]),
        _ => None,
    }
}

fn main() {
    preflight();
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(choices) = segments_for_arguments(&arguments) else {
        eprintln!("用法：不带参数依次跑三段；带 {SEGMENT_ONE_ARGUMENT}、{SEGMENT_TWO_ARGUMENT}、{SEGMENT_THREE_ARGUMENT} 只跑那一段；带 {REPLAY_CORE_INPUTS_ARGUMENT} 打出恢复输入。收到的参数：{arguments:?}");
        std::process::exit(2);
    };
    for choice in choices {
        match choice {
            SegmentChoice::One => run_segment_one(),
            SegmentChoice::Two => run_segment_two(),
            SegmentChoice::Three => run_segment_three(),
            SegmentChoice::ReplayCoreInputs => run_replay_core_inputs(),
        }
    }
}

/// 准入用的仓根：环境变量 SINGLEFS_REPOSITORY_ROOT，没设取当前目录（replay.sh 与实验页的命令都在仓根起）；不取「清单目录/../..」（单元键把它当运行期路径拒算）。
fn repository_root() -> std::path::PathBuf {
    let root = std::env::var_os("SINGLEFS_REPOSITORY_ROOT")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    if !root.join(".claude/singlefs-ai-sop/scripts/preflight.py").is_file() {
        eprintln!("  ✗ {} 不是仓根（下面没有 .claude/singlefs-ai-sop/scripts/preflight.py），判不了准入", root.display());
        eprintln!("  → 怎么办：在仓根起这个装置，或设 SINGLEFS_REPOSITORY_ROOT=<仓根>");
        std::process::exit(78);
    }
    root
}

fn preflight() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let root = repository_root();
    let source = root.join("scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery/main.rs");
    let script = root.join(".claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

#[cfg(test)]
mod tests;
