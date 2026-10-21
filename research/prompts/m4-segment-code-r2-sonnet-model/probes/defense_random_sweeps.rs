//! 辩方腿的随机历史扫描：按环境变量选比重、种子区间、步数、池级 checker 判哪些步。只读测试装置，不改被判代码。
//! 环境变量：DEFENSE_WEIGHTS = compaction | compaction_rollback | snapshots；DEFENSE_SEED_START（相对种子基的偏移）；DEFENSE_SEED_COUNT；
//! DEFENSE_OPS（每段步数）；DEFENSE_CHECK = all | rollback | none。
//! 每段历史打一行 `DEFENSE_SEED seed_offset=<偏移> ending=<结局> reds=<池级 checker 判红的条数>`，末尾一行 `DEFENSE_SUMMARY ...`。

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::admission::SpaceAdmission;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, PerStepChecker,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::SharedStream;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

#[test]
#[ignore = "辩方腿的扫描，按环境变量跑"]
fn defense_random_sweep() {
    let weights_name = std::env::var("DEFENSE_WEIGHTS").unwrap_or_else(|_| "compaction".to_string());
    let weights = match weights_name.as_str() {
        "compaction" => GenerationWeights::COMPACTION_WHILE_MOUNTED,
        "compaction_rollback" => GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK,
        "snapshots" => GenerationWeights::SNAPSHOT_OPERATIONS,
        other => panic!("认不出的比重 {other}"),
    };
    let check = std::env::var("DEFENSE_CHECK").unwrap_or_else(|_| "none".to_string());
    let start = env_u64("DEFENSE_SEED_START", 0);
    let count = env_u64("DEFENSE_SEED_COUNT", 4);
    let operations = usize::try_from(env_u64("DEFENSE_OPS", 40)).expect("步数");
    let width_name = std::env::var("DEFENSE_WIDTH").unwrap_or_else(|_| "4g".to_string());
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: match width_name.as_str() {
            "4g" => HistoryDeviceWidth::FourGibibytes,
            "384" => HistoryDeviceWidth::UnitAreaOf384Slots,
            "256" => HistoryDeviceWidth::UnitAreaOf256Slots,
            "240" => HistoryDeviceWidth::UnitAreaOf240Slots,
            other => panic!("认不出的盘宽 {other}"),
        },
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let mut not_completed = 0_u64;
    let mut total_reds = 0_u64;
    for offset in start..start + count {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(offset);
        let history = generate_history_with_weights(HistorySeed(seed), operations, &weights);
        let mut reds: Vec<String> = Vec::new();
        let run = execute_history_with(
            &history,
            execution,
            &SharedStream::new(),
            &mut |observation| {
                let judge = match check.as_str() {
                    "all" => true,
                    "rollback" => matches!(
                        observation.operation,
                        Some(HistoryOperation::RollBackWhileMounted(_))
                    ),
                    _ => false,
                };
                if !judge {
                    return;
                }
                for (name, verdict) in check_pool_image(observation.image) {
                    if let InvariantVerdict::Violated(why) = verdict {
                        reds.push(format!("{:?} {name}：{why}", observation.position));
                    }
                }
            },
        );
        let ending = match &run.ending {
            HistoryEnding::Completed => "Completed".to_string(),
            other => {
                not_completed += 1;
                format!("{other:?}").chars().take(900).collect()
            }
        };
        if !matches!(run.ending, HistoryEnding::Completed) && std::env::var("DEFENSE_PRINT_HISTORY").is_ok() {
            println!("DEFENSE_HISTORY seed_offset={offset} operations={:?}", history.operations);
            for (index, outcome) in run.outcomes.iter().enumerate() {
                println!(
                    "DEFENSE_OUTCOME seed_offset={offset} step={index} {}",
                    format!("{outcome:?}").chars().take(400).collect::<String>()
                );
            }
        }
        {
            let mut refusals: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
            for outcome in &run.outcomes {
                if let singlefs_harness::history::StepOutcome::Refused { member, .. } = outcome {
                    *refusals.entry(format!("{member}")).or_insert(0) += 1;
                }
            }
            if !refusals.is_empty() {
                println!("DEFENSE_REFUSALS seed_offset={offset} {refusals:?}");
            }
        }
        total_reds += u64::try_from(reds.len()).expect("条数");
        println!(
            "DEFENSE_SEED weights={weights_name} width={width_name} seed_offset={offset} ending={ending} reds={} {}",
            reds.len(),
            reds.iter()
                .take(3)
                .map(|red| red.chars().take(300).collect::<String>())
                .collect::<Vec<_>>()
                .join(" | ")
        );
    }
    println!(
        "DEFENSE_SUMMARY weights={weights_name} start={start} count={count} operations={operations} check={check} not_completed={not_completed} reds={total_reds}"
    );
}

/// 小历史枚举：先写 `units` 个数据单元的文件（0 表示只有起点的第一个文件）、可写挂载、整理一趟，批大小 1 到 3、强制选段下标 0 到 5。
/// 只看 `DEFENSE_TRACE` 行（被测代码的拷贝里加了只打印的行，补丁在模型目录的 `patches/`）。
#[test]
#[ignore = "辩方腿的枚举，按环境变量跑"]
fn defense_small_compaction_enumeration() {
    use singlefs_harness::history::{
        CompactionChoice, CompactionRunEnding, ContentChoice, ContentLength, GeneratedHistory,
        HistoryStartingPoint,
    };
    let units = env_u64("DEFENSE_UNITS", 0);
    let mut completed = 0_u64;
    let mut other = 0_u64;
    for batch_units in 1..=3_u64 {
        for selector in 0..6_u64 {
            let mut operations = Vec::new();
            if units >= 2 {
                operations.push(HistoryOperation::PublishOverwrite(ContentChoice {
                    length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                        data_units: units,
                        selector: 5,
                    },
                    fill_seed: 3,
                }));
            }
            operations.push(HistoryOperation::CloseAndMountWritable);
            operations.push(HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units,
                region_selector: selector,
                ending: CompactionRunEnding::RunsUntilItStops,
            }));
            let history = GeneratedHistory {
                seed: HistorySeed(0),
                starting_point: HistoryStartingPoint::AfterFirstFile,
                operations,
            };
            println!("DEFENSE_CASE units={units} batch_units={batch_units} selector={selector}");
            let run = execute_history_with(
                &history,
                HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
                &SharedStream::new(),
                &mut |_| {},
            );
            match run.ending {
                HistoryEnding::Completed => completed += 1,
                other_ending => {
                    other += 1;
                    println!(
                        "DEFENSE_CASE_ENDING units={units} batch_units={batch_units} selector={selector} ending={}",
                        format!("{other_ending:?}").chars().take(600).collect::<String>()
                    );
                }
            }
        }
    }
    println!("DEFENSE_ENUM_SUMMARY units={units} completed={completed} not_completed={other}");
}
