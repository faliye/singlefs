//! E161（崩溃放量的去重与分段耗时）跑前登记第一段：入库装置。
//!
// admission: always 每跑一次都在今天这份 crates 的恢复与 checker 上重量一遍逐状态的性质与分段耗时，计时每次都是新观测
// run-condition: command python3
// task-server-gpu-arguments: stop-clauses feasibility probe-first-stream-stop-clauses
// 只有这几个模式经 `report_crates_comparison` 走 `enumerate_crash_states_with_expansion` 的判段；segment-one 与 probe 只在本文件里
// 自己算、不要卡。带 gpu 编好的二进制直接交统一任务服务时，由服务按 task-server-gpu-arguments 的声明清场、分卡。
//!
//! 登记：`research/prompts/e161-preregistration.md`（5.5「第一段」：D1-small、D1-stride、D1-quick，停机条款 S1–S6、
//! 锚点 A1–A7、5.3 全部阳性对照、第八节几何敏感性里 D1-quick 与线程数 1 那两格）。第二、三段（D2、D3 全量）是同一个装置换取样域，
//! 本文件给了模式名但这一段不跑。
//! 第 2 次重跑登记：`research/prompts/e161-r2-prereg.md`（C577 那道屏障之后的段形：新池新建文件流 11 段、覆盖写固定脚本流 78 段取段 11–26，
//! 取样域照它 5.5 的读法甲重定，装置的改动照它 5.6 的 I-1 到 I-12）。
//! 第 3 次重跑登记：`research/prompts/e161-r3-prereg.md`（取号写两轮之后的段形：新池新建文件流 12 段、覆盖写固定脚本流 80 段取段 12–28，
//! 段长、锚点与 PC3 的段照它第七节重钉，装置的改动照它 5.6 的 J-1 到 J-7）。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness 已有的文件。
//! 新池新建文件流用实验包库造的池（`singlefs_implementation_experiments::experiment_pool::build_pool`：录出的流与验证包库的新池新建文件那条路逐条相同，
//! 由库的单测核）；覆盖写固定脚本流的脚本照抄
//! `crates/singlefs-verification/tests/crash_enumeration_fixed_script_stream.rs` 的 `prepare`（到 E 再正常卸载那一支）。
//! 决定几何与门槛的数都写成本地常量（值抄自登记第十一节 S1 与第七节），`main` 开头逐条与 crates 的同名常量回比（S1）。
//! 枚举计划与撕裂镜像表是本文件自己的一份，与 crates 私有的那一份逐状态对拍（S3）。
//!
//!   e161_crash_state_dedup_and_time_split segment-one
//!
//! 线程数取环境变量 `E161_THREADS`，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）。产物是 `E7RESULT` 行，收尾 `name=done emitted=N`。
//! 退出码：0 正常；2 命令行不对；3 停机条款没过（S1–S5，产物照打、交主 agent）；preflight 拒绝时照它的退出码退。

use singlefs_implementation_experiments::experiment_pool;

use singlefs_verification::crash_judge_conclusions::recovery_passes_differ;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use singlefs_checker::image::{
    chosen_system_configurations, valid_roots, ImageReader, InvariantVerdict,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_verification::crash::{
    check_records, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
};
use singlefs_verification::memory_pool::{
    newest_persisted_root, CrashImage, MemoryPool, PublishedVersion, RetainedWrite, WrittenContents,
};
use singlefs_verification::segments::StepKind;

mod anchor_values;
mod arm_record_spill_store;
mod cell_result_lines;
mod cell_tallies_and_multithreaded_run;
mod deterministic_128_bit_digest;
mod device_memory_and_extrapolation;
mod enumeration_plan_and_torn_write_table;
mod feasibility_mode;
mod geometry_sensitivity_and_discrimination;
#[cfg(test)]
mod helpers_shared_by_the_tests;
mod keys_without_one_position;
mod local_constants_checked_against_the_crates;
mod per_state_evaluation;
mod positive_controls_pc5_and_pc3;
mod read_categories_and_content_tables;
mod recording_reader;
mod reuse_arm_and_trajectory_counts;
mod sampling_domains;
mod segment_one_mode;
mod segment_state_counts_and_comparison_with_the_crates_enumeration;
mod stop_clauses_mode;
mod two_stream_setup;
mod unit_check_replay_and_state_clocks;
mod verdict_lines;
mod weighted_unit_share_estimate;

use anchor_values::*;
use arm_record_spill_store::*;
use cell_result_lines::*;
use cell_tallies_and_multithreaded_run::*;
use deterministic_128_bit_digest::*;
use device_memory_and_extrapolation::*;
use enumeration_plan_and_torn_write_table::*;
use feasibility_mode::*;
use geometry_sensitivity_and_discrimination::*;
#[cfg(test)]
use helpers_shared_by_the_tests::*;
use keys_without_one_position::*;
use local_constants_checked_against_the_crates::*;
use per_state_evaluation::*;
use positive_controls_pc5_and_pc3::*;
use read_categories_and_content_tables::*;
use recording_reader::*;
use reuse_arm_and_trajectory_counts::*;
use sampling_domains::*;
use segment_one_mode::*;
use segment_state_counts_and_comparison_with_the_crates_enumeration::*;
use stop_clauses_mode::*;
use two_stream_setup::*;
use unit_check_replay_and_state_clocks::*;
use verdict_lines::*;
use weighted_unit_share_estimate::*;

// ───────────────────────────── preflight 与入口 ─────────────────────────────

/// `preflight` 交给 `preflight.py check` 的源文件：这个二进制的根文件，准入与运行条件的声明写在它的文件头
/// （`preflight.py` 只读给它的那一份文件头，不读子模块）。
const SOURCE_PATH_CARRYING_THE_PREFLIGHT_DECLARATIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/bin/e161_crash_state_dedup_and_time_split/main.rs"
);

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `preflight.py check`，
/// 退出码不是 0 就原样退出；交回 `forced` 时的摘要（强制跑的要写进产物）。
fn preflight() -> Option<String> {
    let source = SOURCE_PATH_CARRYING_THE_PREFLIGHT_DECLARATIONS;
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../.claude/singlefs-ai-sop/scripts/preflight.py"
    );
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut command = std::process::Command::new("python3");
    command.arg(script).arg("check").arg(source);
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
    let first_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    first_line
        .strip_prefix("forced")
        .map(|summary| summary.trim().replace(' ', "_"))
}

fn main() {
    let forced_summary = preflight();
    let arguments: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    let mode = arguments.first().map(String::as_str);
    let mut output = ResultLines::new();
    if let Some(summary) = forced_summary {
        output.line(format!("name=preflight forced={summary}"));
    }
    let exit_code = match mode {
        Some("segment-one") => run_segment_one(&mut output),
        Some("probe") => run_probe(&mut output, arguments.get(1).map(String::as_str)),
        Some("stop-clauses") => run_stop_clauses(&mut output),
        Some("feasibility") => run_feasibility(&mut output),
        Some("probe-first-stream-stop-clauses") => run_first_stream_stop_clause_probe(&mut output),
        Some(_) | None => {
            eprintln!("  ✗ 用法：e161_crash_state_dedup_and_time_split stop-clauses | segment-one | feasibility | probe <状态数> | probe-first-stream-stop-clauses");
            eprintln!(
                "  → 怎么办：第一段照登记 5.5 跑 segment-one；feasibility 是缩域（只评小域，判定行 G 一律写 not_judged_feasibility_only，不当正式产物）；probe 只量每状态耗时，不出产物"
            );
            2
        }
    };
    output.finish();
    std::process::exit(exit_code);
}

/// 产物行：每行 `E7RESULT ` 起头，收尾 `name=done emitted=N`（N 连 config 行与 done 行自己都算）。
struct ResultLines {
    emitted: u64,
}

impl ResultLines {
    fn new() -> Self {
        Self { emitted: 0 }
    }

    fn line(&mut self, text: String) {
        self.emitted += 1;
        println!("E7RESULT {text}");
    }

    /// 收尾行：`name=done emitted=<行数，含这一行> loadavg1=<1 分钟负载> cpus=<在线 CPU 数>`
    /// （复跑脚本 `scripts/experiment-apparatus/replay.sh` 文件头「计时行的负载关」读这两个字段；它们不进 emitted）。
    fn finish(&mut self) {
        self.emitted += 1;
        let loadavg_text = std::fs::read_to_string("/proc/loadavg")
            .expect("Linux 上读得到 /proc/loadavg：复跑的负载关要这一项");
        let loadavg_one_minute = loadavg_text
            .split_whitespace()
            .next()
            .expect("/proc/loadavg 第一列是 1 分钟负载");
        let cpuinfo_text = std::fs::read_to_string("/proc/cpuinfo")
            .expect("Linux 上读得到 /proc/cpuinfo：复跑的负载关要在线 CPU 数");
        let cpus = cpuinfo_text
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count();
        println!(
            "E7RESULT name=done emitted={} loadavg1={loadavg_one_minute} cpus={cpus}",
            self.emitted
        );
    }
}

fn run_probe(output: &mut ResultLines, _states: Option<&str>) -> i32 {
    let first = prepare_first_stream();
    let second = prepare_second_stream();
    let first_check = check_stream_shape(&first);
    let second_check = check_stream_shape(&second);
    let (bases_identical, prefix_identical) = shared_prefix_is_identical(&first, &second);
    output.line(format!(
        "name=probe first_segments_ok={} first_writes_ok={} first_roots_ok={} second_segments_ok={} second_writes_ok={} second_roots_ok={} bases_identical={bases_identical} prefix_identical={prefix_identical}",
        first_check.segment_lengths_match,
        first_check.writes_match,
        first_check.roots_match,
        second_check.segment_lengths_match,
        second_check.writes_match,
        second_check.roots_match,
    ));
    let requested: usize = _states.and_then(|text| text.parse().ok()).unwrap_or(1000);
    for stream in [&first, &second] {
        let table = TornWriteTable::of(stream);
        let plan = LocalPlan::new(
            stream,
            &table,
            domain_expansions(stream, LocalExpansion::EveryCombination),
        );
        let context = EvaluationContext::new(stream, &table);
        let small = small_domain_ordinals(stream, &plan);
        let stride = stride_domain_ordinals(stream, &plan);
        let mut all: Vec<u64> = small.iter().chain(&stride).copied().collect();
        all.sort_unstable();
        let control = control_ordinals(stream, &plan, &all);
        for (label, ordinals) in [("small", &small), ("stride", &stride)] {
            let taken: Vec<u64> = ordinals.iter().copied().take(requested).collect();
            let started = Instant::now();
            let totals = evaluate_stream_domain(&StreamDomainRun {
                label: format!("probe-{}-{label}", stream.name.label()),
                context: &context,
                plan: &plan,
                ordinals: &taken,
                control_ordinals: &control,
                timing: TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT,
                threads: threads_from_environment(),
                depth: EvaluationDepth::FourSteps,
            });
            let control_calls: usize = totals
                .control
                .iter()
                .map(|data| {
                    data.consult_calls.len() + data.ignore_calls.len() + data.full_calls.len()
                })
                .sum();
            output.line(format!(
                "name=probe_cell stream={} domain={label} domain_states={} probed={} wall_elapsed_ns={} consult_differs={} ignore_differs={} verdicts_differ={} unit_events={} unit_positions={} control_states={} control_calls={control_calls} replays={} w={}",
                stream.name.label(),
                ordinals.len(),
                totals.states(),
                started.elapsed().as_nanos(),
                totals.consult_report_differs,
                totals.ignore_report_differs,
                totals.verdicts_differ,
                totals.whole_unit_events,
                totals.whole_unit_positions,
                totals.control.len(),
                table.replay_count,
                table.mask_width(),
            ));
        }
    }
    0
}

fn run_segment_one(output: &mut ResultLines) -> i32 {
    let threads = threads_from_environment();
    output.line(format!(
        "name=config experiment=E161 mode=segment_one threads={threads} registration=research/prompts/e161-preregistration.md stop_clauses_product=research/results/e161-crash-state-dedup-and-time-split-stop-clauses-2026-09-27.out"
    ));
    let constants_hold = local_constants_against_the_crates()
        .iter()
        .all(|(_name, local, crates)| local == crates);
    let first = prepare_first_stream();
    let second = prepare_second_stream();
    let first_shape = check_stream_shape(&first);
    let second_shape = check_stream_shape(&second);
    let shapes_hold = first_shape.segment_lengths_match
        && first_shape.writes_match
        && second_shape.segment_lengths_match
        && second_shape.writes_match;
    output.line(format!(
        "name=stop_clause_recheck s1_constants_hold={} s2_shapes_hold={}",
        bool_text(constants_hold),
        bool_text(shapes_hold)
    ));
    if !(constants_hold && shapes_hold) {
        return 3;
    }
    let (first_outcome, first_discriminating) = process_stream(output, &first, threads);
    let (second_outcome, second_discriminating) = process_stream(output, &second, threads);
    // 8.2 线程数那一格：新池新建文件流的 D1-stride 用线程数 1 再跑一遍，全部计数与结果行（去掉计时字段）逐字比（V6）。
    let first_table = TornWriteTable::of(&first);
    let first_context = EvaluationContext::new(&first, &first_table);
    let first_full = LocalPlan::new(
        &first,
        &first_table,
        domain_expansions(&first, LocalExpansion::EveryCombination),
    );
    let first_stride = stride_domain_ordinals(&first, &first_full);
    let strided: BTreeSet<usize> = first_stride
        .iter()
        .map(|ordinal| first_full.segment_of(*ordinal))
        .collect();
    let mut first_small_and_stride: Vec<u64> = small_domain_ordinals(&first, &first_full)
        .into_iter()
        .chain(first_stride.iter().copied())
        .collect();
    first_small_and_stride.sort_unstable();
    let first_control = control_ordinals(&first, &first_full, &first_small_and_stride);
    let mut single_thread = evaluate_stream_domain(&StreamDomainRun {
        label: "first_stride_threads_one".to_string(),
        context: &first_context,
        plan: &first_full,
        ordinals: &first_stride,
        control_ordinals: &first_control,
        timing: TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT,
        threads: 1,
        depth: EvaluationDepth::FourSteps,
    });
    let single_thread_stores = std::mem::take(&mut single_thread.arm_stores);
    let single_thread_summary = report_cell(
        output,
        "first_stride",
        &single_thread,
        single_thread_stores,
        &TrajectoryIntervals::of(&first_full, &first_stride, &strided),
        None,
    );
    let lines_identical =
        single_thread_summary.lines_without_timing == first_outcome.stride.lines_without_timing;
    output.line(format!(
        "name=geometry_sensitivity knob=threads stream=first cell=first_stride threads_default={threads} threads_other=1 result_lines_identical_without_timing={} {}",
        bool_text(lines_identical),
        share_sensitivity_fields(
            "unit_share",
            unweighted_unit_share(&first_outcome.stride),
            unweighted_unit_share(&single_thread_summary)
        ),
    ));
    // 8.2 掩码宽那一格：新池新建文件流的 D1 用覆盖写固定脚本流整条流的 W 重判 Q5b、Q5c。
    for arm in [Arm::WalkUnits, Arm::AllUnits, Arm::FullReadSet] {
        let mut relabelled = first_outcome.union.clone();
        relabelled.label = "first_d1_with_second_stream_w".to_string();
        let _verdicts = report_device_memory(
            output,
            &relabelled,
            second_outcome.write_table_length,
            second_outcome.write_table_length,
            arm,
        );
    }
    report_geometry_sensitivity(output, &first_outcome);
    report_geometry_sensitivity(output, &second_outcome);
    let timing_control_holds = report_timing_control(output, &second, threads);
    report_verdicts(
        output,
        &[&first_outcome, &second_outcome],
        &[first_discriminating, second_discriminating],
        timing_control_holds,
        lines_identical,
    )
}

/// S3 那个观察者（名字 `e161_crates_comparison`，登记在实验包的 `singlefs_implementation_experiments::judging_units_entries::OBSERVER_ENTRIES`，判法单元按这个入口 fn 收，
/// 键由实验包的构建脚本算）：
/// 经崩溃判定库那一路按块看（并行、不按序号先后），要记的数全经计数记：crates 枚举里序号 k 的状态就是 crates 次序里第 k 个，
/// 对装置计划的第 k 个序号；写表只在第一个取样的状态上比一次（整条流的枚举写表是同一张）。
fn observe_the_crates_comparison(
    table: &TornWriteTable,
    plan: &LocalPlan<'_>,
    ordinals: &[u64],
    ordinal: u64,
    image: &CrashImage<'_>,
    counts: &mut singlefs_verification::crash::CrashPointReplayObserverCounts,
) {
    counts.add("observed_states", 1);
    let Some(device_ordinal) = usize::try_from(ordinal)
        .ok()
        .and_then(|position| ordinals.get(position))
    else {
        return;
    };
    counts.add("compared_states", 1);
    if ordinal == 0 && image.writes != table.writes.as_slice() {
        counts.add("writes_table_differs", 1);
    }
    if image.persisted != plan.persisted_of(*device_ordinal) {
        counts.add("persisted_mismatches", 1);
    }
}

// e161-append-point

#[cfg(test)]
mod tests;
