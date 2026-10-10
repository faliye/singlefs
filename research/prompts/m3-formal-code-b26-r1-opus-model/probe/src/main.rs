//! m3-formal-code-b26-r1 攻方腿的探针。子命令：
//! - `l0 <比重> <盘宽> <步数> <起始种子> <段数> [状态数上限] [第二截每几个状态取一个] [二次崩溃每段上限]`
//! - `show <比重> <盘宽> <步数> <种子>`：打出历史的步序与起点之后的段长。
//! 线程数取 `OPUS_THREADS`（缺省 7）；层 0 自己的线程由 `capped.sh` 设的变量管。
mod common;
mod custom;
mod f3;
mod unmount;
mod l0;
mod stage2;

use std::time::Instant;

use crate::common::{record, weights_named, width_named};
use crate::stage2::Tally;

fn argument(arguments: &[String], index: usize, default: &str) -> String {
    arguments.get(index).cloned().unwrap_or_else(|| default.to_string())
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let mode = argument(&arguments, 1, "");
    let weights = if mode == "custom" {
        weights_named("broad")
    } else {
        weights_named(&argument(&arguments, 2, "broad"))
    };
    let width = width_named(&argument(&arguments, 3, "unit-area-384"));
    // custom 子命令的第 4、5 个参数是前缀步数与步序，不在这里解。
    let steps: usize = argument(&arguments, 4, "8").parse().unwrap_or(0);
    let seed_from: u64 = argument(&arguments, 5, "0").parse().unwrap_or(0);
    match mode.as_str() {
        "show" => {
            let recorded = record(&weights, width, seed_from, steps, 0);
            println!("SHOW seed={seed_from} start={:?} ending={}", recorded.history.starting_point, recorded.ending);
            for (index, operation) in recorded.history.operations.iter().enumerate() {
                println!("  step {index}: {operation:?}");
            }
            let lengths: Vec<usize> = recorded.segments.iter().map(Vec::len).collect();
            println!("  segments after the starting point: {lengths:?}");
            println!("  after_each_step: {:?}", recorded.after_each_step);
            println!("  faulted_steps: {:?}", recorded.faulted_steps);
        }
        "l0" => {
            let seed_count: u64 = argument(&arguments, 6, "1").parse().expect("段数");
            let options = l0::Options {
                max_states: argument(&arguments, 7, "1000000").parse().expect("状态数上限"),
                stage2_every: argument(&arguments, 8, "0").parse().expect("第二截取样"),
                second_crash_cap: argument(&arguments, 9, "16").parse().expect("二次崩溃上限"),
                threads: std::env::var("OPUS_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(7),
            };
            let label = format!("{}/{}/{steps}", weights.environment_value(), width.environment_value());
            let started = Instant::now();
            let mut total_states = 0u64;
            let mut total = Tally::default();
            for seed in seed_from..seed_from + seed_count {
                // 固定前缀取最短的那一个：起点之后从第 k 步起全量枚举，状态数不超过上限的最小 k（前缀只施加、不枚举）。
                let chosen = (0..=steps).find_map(|prefix_steps| {
                    let recorded = record(&weights, width, seed, steps, prefix_steps);
                    l0::state_count(&recorded)
                        .filter(|states| *states <= options.max_states)
                        .map(|_| (prefix_steps, recorded))
                });
                let Some((prefix_steps, recorded)) = chosen else {
                    println!("L0SKIP {label} seed={seed} 哪一个前缀下状态数都超过上限或没有根槽写");
                    continue;
                };
                println!("L0PREFIX {label} seed={seed} prefix_steps={prefix_steps}");
                if let Some((states, stage2)) = l0::run_one(&recorded, seed, &options, &label) {
                    total_states += states;
                    total.absorb(stage2);
                }
            }
            println!(
                "L0TOTAL {label} seeds={seed_from}..{} layer0_states={total_states} elapsed_secs={:.1}",
                seed_from + seed_count,
                started.elapsed().as_secs_f64()
            );
            for (key, count) in &total.counts {
                println!("S2 {key}={count}");
            }
            for (key, list) in &total.examples {
                for text in list {
                    println!("S2EX {key} :: {text}");
                }
            }
        }
        "frac" => {
            // frac <比重> <盘宽> <步数> <起始种子> <段数>：起点之后的段里，崩溃窗口末尾落在第一个「读故障下发过东西」的步之后的占几成
            // （这些段上摆的第一次崩溃态在崩溃注入里都认成注入过读故障：第一截挂载透明不判、新拒因复算为假不算不合法）。
            let seed_count: u64 = argument(&arguments, 6, "1").parse().expect("段数");
            let (mut segments_total, mut segments_injected, mut histories_with_fault) = (0usize, 0usize, 0usize);
            for seed in seed_from..seed_from + seed_count {
                let recorded = record(&weights, width, seed, steps, 0);
                let first_fault = recorded.faulted_steps.iter().map(|(_, before)| *before).min();
                histories_with_fault += usize::from(first_fault.is_some());
                for segment in &recorded.segments {
                    segments_total += 1;
                    let window_end = recorded.stream_indexes[*segment.last().expect("段非空")];
                    if first_fault.is_some_and(|before| before <= window_end) {
                        segments_injected += 1;
                    }
                }
            }
            println!(
                "FRAC {} {} steps={steps} seeds={seed_from}..{} histories_with_a_faulted_step_that_sent_writes={histories_with_fault} segments={segments_total} segments_marked_injected={segments_injected}",
                weights.environment_value(),
                width.environment_value(),
                seed_from + seed_count
            );
        }
        "f3" => f3::run(),
        "unmount" => {
            // unmount broad <盘宽> <覆盖写次数> [状态数上限] [第二截每几个取一个] [二次崩溃每段上限]
            let overwrites: usize = argument(&arguments, 4, "1").parse().expect("覆盖写次数");
            let options = l0::Options {
                max_states: argument(&arguments, 5, "1000000").parse().expect("状态数上限"),
                stage2_every: argument(&arguments, 6, "1").parse().expect("第二截取样"),
                second_crash_cap: argument(&arguments, 7, "16").parse().expect("二次崩溃上限"),
                threads: std::env::var("OPUS_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(7),
            };
            let recorded = unmount::record(width, overwrites);
            let lengths: Vec<usize> = recorded.segments.iter().map(Vec::len).collect();
            let label = format!("unmount/{}/ow{overwrites}", width.environment_value());
            println!("UNMOUNT {label} segments={lengths:?} states={:?}", l0::state_count(&recorded));
            if let Some((states, stage2)) = l0::run_one(&recorded, 0, &options, &label) {
                println!("UNMOUNTTOTAL {label} layer0_states={states}");
                for (key, count) in &stage2.counts {
                    println!("S2 {key}={count}");
                }
                for (key, list) in &stage2.examples {
                    for text in list {
                        println!("S2EX {key} :: {text}");
                    }
                }
            }
        }
        "quick" => {
            // quick <盘宽>：起点取「mkfs 同一个进程里取号、暖机、发第一个文件」（AfterFirstFile），前缀只有 mkfs，
            // 写表按甲二展开（单元写全不落或全落），打出 checker 每条的 评估/违例/不适用——对 26-2 #8 那 69 条钉值用。
            use singlefs_checker_tier::crash::enumerate_layer0_quick_tier_versions;
            use singlefs_harness::segments::StepKind;
            let history = singlefs_harness::history::GeneratedHistory {
                seed: singlefs_harness::history::HistorySeed(0),
                starting_point: singlefs_harness::history::HistoryStartingPoint::AfterFirstFile,
                operations: Vec::new(),
            };
            // 盘宽取第 3 个参数（quick broad <盘宽>）。
            let recorded = common::record_history(history, width, usize::MAX);
            let lengths: Vec<usize> = recorded.segments.iter().map(Vec::len).collect();
            let judged_root_index = recorded
                .writes
                .iter()
                .rposition(|write| write.kind == StepKind::RootRecordFua)
                .expect("有根槽写");
            let tally = enumerate_layer0_quick_tier_versions(
                &recorded.base,
                &recorded.writes,
                &recorded.segments,
                judged_root_index,
                &recorded.versions,
            );
            println!("QUICK {} segments={lengths:?} states={} root_persisted={} verification_ran={}", width.environment_value(), tally.states, tally.root_persisted_states, tally.verification_ran_states);
            for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
                println!(
                    "QUICKINV {invariant}={}/{}/{}",
                    tally.checker_evaluated_states.get(invariant).copied().unwrap_or(0),
                    tally.checker_violated_states.get(invariant).copied().unwrap_or(0),
                    tally.checker_not_applicable_states.get(invariant).copied().unwrap_or(0)
                );
            }
        }
        "seglen" => {
            // seglen <比重> <盘宽> <步数> <起始种子> <段数>：整条录制流（mkfs 之后起）的段长分布：最长、超过 32 个写的段数。
            let seed_count: u64 = argument(&arguments, 6, "1").parse().expect("段数");
            let (mut longest, mut over_32, mut segments_total, mut longest_seed) = (0usize, 0usize, 0usize, 0u64);
            for seed in seed_from..seed_from + seed_count {
                let history = singlefs_harness::history::generate_history_with_weights(
                    singlefs_harness::history::HistorySeed(seed),
                    steps,
                    &weights,
                );
                let starting_point = history.starting_point;
                let recorded = common::record_history(history, width, 0);
                // 起点那几步也算进来（崩溃注入从 mkfs 之后起摆）：再录一份只含起点的。
                let _ = starting_point;
                for segment in &recorded.segments {
                    segments_total += 1;
                    if segment.len() > 32 {
                        over_32 += 1;
                    }
                    if segment.len() > longest {
                        longest = segment.len();
                        longest_seed = seed;
                    }
                }
            }
            println!(
                "SEGLEN {} {} steps={steps} seeds={seed_from}..{} segments_after_the_starting_point={segments_total} longest={longest} (seed {longest_seed}) over_32={over_32}",
                weights.environment_value(),
                width.environment_value(),
                seed_from + seed_count
            );
        }
        "custom" => {
            // custom <起点 mkfs|file> <盘宽> <固定前缀步数> <步序> [状态数上限] [第二截每几个取一个] [二次崩溃每段上限]
            let start = argument(&arguments, 2, "file");
            let prefix_steps: usize = argument(&arguments, 4, "0").parse().expect("前缀步数");
            let ops = argument(&arguments, 5, "");
            let options = l0::Options {
                max_states: argument(&arguments, 6, "1000000").parse().expect("状态数上限"),
                stage2_every: argument(&arguments, 7, "0").parse().expect("第二截取样"),
                second_crash_cap: argument(&arguments, 8, "16").parse().expect("二次崩溃上限"),
                threads: std::env::var("OPUS_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(7),
            };
            let seed: u64 = std::env::var("OPUS_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
            let label = format!("custom/{start}/{}/p{prefix_steps}/{ops}", width.environment_value());
            let recorded = common::record_history(custom::history(&start, &ops, seed), width, prefix_steps);
            let lengths: Vec<usize> = recorded.segments.iter().map(Vec::len).collect();
            println!("CUSTOM {label} ending={} segments={lengths:?} faulted_steps={:?} after_each_step={:?} states={:?}",
                recorded.ending, recorded.faulted_steps, recorded.after_each_step, l0::state_count(&recorded));
            if options.max_states > 0 {
                let started = Instant::now();
                if let Some((states, stage2)) = l0::run_one(&recorded, seed, &options, &label) {
                    println!("CUSTOMTOTAL {label} layer0_states={states} elapsed_secs={:.1}", started.elapsed().as_secs_f64());
                    for (key, count) in &stage2.counts {
                        println!("S2 {key}={count}");
                    }
                    for (key, list) in &stage2.examples {
                        for text in list {
                            println!("S2EX {key} :: {text}");
                        }
                    }
                }
            }
        }
        other => panic!("不认得的子命令 {other:?}"),
    }
}
