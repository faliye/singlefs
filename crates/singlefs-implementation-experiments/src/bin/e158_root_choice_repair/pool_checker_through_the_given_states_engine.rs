//! 经 `GivenStatesEngine` 判盘：池级 checker 在故障之后的镜像与前缀上的判定。

use super::*;

/// 崩溃注入的给定状态表交崩溃点重放的引擎判（表与结论进家目录下给定状态表那一类的判定库；配了 GPU 且编进了 `gpu` 特性上卡，否则 CPU）。
/// 这份装置的产物只收它自己打的行：引擎不调 `finish`，不打收尾行。
pub(super) fn given_states_engine_of_this_experiment(
    seeds_and_scale: &str,
) -> singlefs_verification::crash_amplification::GivenStatesEngine {
    singlefs_verification::crash_amplification::GivenStatesEngine::start_from_environment(
        singlefs_verification::crash_amplification::GivenStatesItem {
            item_name: format!("e158_root_choice_repair::{}", seeds_and_scale),
        },
        &singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
}

/// 故障注入（`singlefs_verification::fault_injection::inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine`，
/// 就是它）一个注入点交出去的盘交 `engine` 两段判，
/// 交回池级 checker 的结论：重开那张盘的录制流的前缀（每步镜像各一个、整条一个）建成给定状态表，被吞的写插回去那张盘另一张表；
/// 录入段写进家目录下给定状态表那一类的判定库、判段从库里读回判（配了 GPU 且编进 `gpu` 特性在卡上判，否则 CPU 判器判），判过的复用。
/// 表名是 `table_name` 接上前缀那一截；一个注入点每交一次给一个新名字（同一趟里同名而持久集合不同的两张表，录入段报错）。
///
/// # Panics
/// 引擎判不了某张表（不退回别处判）；某张表交回的是占位结论（这一趟只录：配置开了 GPU 而没编进 `gpu` 特性，或设了只录那一段；
/// 或那张表归别的机器判）：装置要的是判过的结论，交不出就停，不拿占位往下走。
pub(super) fn pool_checker_verdicts_of_the_images_after_a_fault_through_the_engine(
    engine: &singlefs_verification::crash_amplification::GivenStatesEngine,
    table_name: &str,
    execution: singlefs_verification::history::HistoryExecution,
    images: &singlefs_verification::fault_injection::ImagesAfterAFault,
) -> singlefs_verification::fault_injection::PoolCheckerVerdictsAfterAFault {
    let mut prefixes: Vec<usize> = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .map(|step_image| step_image.operations_recorded)
        .collect();
    prefixes.push(images.image_after_the_fault.operations.len());
    let mut violations_of_the_prefixes = pool_checker_violations_of_the_prefixes_through_the_engine(
        engine,
        table_name,
        &images.image_after_the_fault,
        execution,
        &prefixes,
    );
    let violations_after_the_fault = violations_of_the_prefixes
        .pop()
        .expect("整条录制流那一个前缀在最后");
    let invariants_still_violated_if_the_swallowed_writes_had_landed = images
        .image_with_the_swallowed_writes_landed
        .as_ref()
        .map_or_else(BTreeSet::new, |landed| {
            pool_checker_violations_of_the_prefixes_through_the_engine(
                engine,
                &format!("{table_name}/swallowed-writes-landed"),
                landed,
                execution,
                &[landed.operations.len()],
            )
            .into_iter()
            .flatten()
            .map(|(invariant, _)| invariant)
            .collect()
        });
    singlefs_verification::fault_injection::PoolCheckerVerdictsAfterAFault {
        violations_of_the_step_images: violations_of_the_prefixes,
        violations_after_the_fault,
        invariants_still_violated_if_the_swallowed_writes_had_landed,
    }
}

/// 一张盘的录制流的若干个前缀（`prefixes`：每个是录制流里的步数）建成给定状态表交 `engine` 判，交回与 `prefixes` 同序的池级 checker 违例：
/// 录制流里 mkfs 写的那几步进基线，每个前缀一个持久集合（前缀里的写全落、之后的一个都没落），掉电接缝相同的几个前缀一张表。
/// 表的建法照抄 `crates/singlefs-verification/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`
/// 的 `judge_prefixes_of_a_recorded_stream`（那份用例自己的计数去掉）。
///
/// # Panics
/// 同 [`pool_checker_verdicts_of_the_images_after_a_fault_through_the_engine`]。
fn pool_checker_violations_of_the_prefixes_through_the_engine(
    engine: &singlefs_verification::crash_amplification::GivenStatesEngine,
    table_name: &str,
    recorded: &singlefs_verification::fault_injection::RecordedStreamOfAnImage,
    execution: singlefs_verification::history::HistoryExecution,
    prefixes: &[usize],
) -> Vec<Vec<(&'static str, String)>> {
    use singlefs_verification::crash_given_states::GivenStatesOfATable;
    use singlefs_verification::crash_injection::HistoryAfterTheMakeFilesystem;
    use singlefs_verification::memory_pool::{
        writes_and_segments_with_stream_indexes, RecordStreamSeam,
    };
    let geometry = execution.device_width.fixed_geometry();
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&recorded.operations, &geometry);
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        recorded.operations_written_by_make_filesystem,
        execution,
    );
    // 每次掉电之后第一个写在写表里的下标：取录制流下标不小于掉电之后第一步的第一个写（写表不算屏障），掉电之后一个写都没录下的取写表长度
    let first_write_after_each_power_loss: Vec<usize> = recorded
        .earliest_stream_index_after_each_power_loss
        .iter()
        .map(|earliest_stream_index_after_the_power_loss| {
            stream_indexes
                .iter()
                .position(|stream_index| {
                    *stream_index >= *earliest_stream_index_after_the_power_loss
                })
                .unwrap_or(stream_indexes.len())
        })
        .collect();
    let mut prefixes_by_power_losses: BTreeMap<Vec<usize>, Vec<usize>> = BTreeMap::new();
    let mut persisted_writes_of_each_prefix: Vec<usize> = Vec::with_capacity(prefixes.len());
    for (prefix_index, operations_recorded) in prefixes.iter().enumerate() {
        let persisted_writes = stream_indexes
            .iter()
            .filter(|stream_index| **stream_index < *operations_recorded)
            .count();
        persisted_writes_of_each_prefix.push(persisted_writes);
        let power_losses_before_the_prefix_ends: Vec<usize> = first_write_after_each_power_loss
            .iter()
            .copied()
            .filter(|first_write_after_the_power_loss| {
                *first_write_after_the_power_loss <= persisted_writes
            })
            .collect();
        prefixes_by_power_losses
            .entry(power_losses_before_the_prefix_ends)
            .or_default()
            .push(prefix_index);
    }
    let mut violations: Vec<Option<Vec<(&'static str, String)>>> = vec![None; prefixes.len()];
    for (power_losses, prefix_indexes) in prefixes_by_power_losses {
        let seams: Vec<RecordStreamSeam> = power_losses
            .iter()
            .map(
                |first_write_after_the_power_loss| RecordStreamSeam::AfterAPowerLoss {
                    first_write_after_the_power_loss: history_after_the_make_filesystem
                        .index_in_the_table(*first_write_after_the_power_loss),
                },
            )
            .collect();
        let persisted_sets: Vec<Vec<bool>> = prefix_indexes
            .iter()
            .map(|prefix_index| {
                let persisted_writes = persisted_writes_of_each_prefix[*prefix_index];
                let persisted_over_the_whole_write_table: Vec<bool> = (0..writes.len())
                    .map(|write_index| write_index < persisted_writes)
                    .collect();
                history_after_the_make_filesystem
                    .persisted_set_of_the_table(&persisted_over_the_whole_write_table)
            })
            .collect();
        let table = GivenStatesOfATable {
            table_name: format!("{table_name}/power-losses={power_losses:?}"),
            base: history_after_the_make_filesystem.base_pool(),
            writes: history_after_the_make_filesystem.writes(),
            segments: history_after_the_make_filesystem.segments(),
            seams: &seams,
            persisted_sets,
        };
        let states = given_states_judged_on_this_host_through_the_engine(engine, &table);
        assert_eq!(
            states.len(),
            prefix_indexes.len(),
            "判器逐状态交回结论，与表里的状态同序"
        );
        for (prefix_index, conclusion) in prefix_indexes.into_iter().zip(states) {
            violations[prefix_index] = Some(conclusion.pool_checker_violations());
        }
    }
    violations
        .into_iter()
        .map(|violations| violations.expect("每个前缀恰好落在一组掉电接缝里"))
        .collect()
}

/// 一张给定状态表交 `engine` 判，交回逐状态的结论（与表里的持久集合同序）：这一台判的或从判定库读回的才收。
///
/// # Panics
/// 引擎判不了这张表（不退回别处判）；交回的是占位结论（这一趟只录：配置开了 GPU 而没编进 `gpu` 特性，或设了只录那一段；
/// 或那张表归别的机器判）：装置要的是判过的结论，交不出就停，不拿占位往下走。
pub(super) fn given_states_judged_on_this_host_through_the_engine(
    engine: &singlefs_verification::crash_amplification::GivenStatesEngine,
    table: &singlefs_verification::crash_given_states::GivenStatesOfATable<'_>,
) -> Vec<singlefs_verification::crash_given_states::GivenStateConclusion> {
    use singlefs_verification::crash_given_states::{
        GivenStatesConclusionsSource, GivenStatesJudge,
    };
    let judged_table = engine
        .judge_the_given_states(table)
        .unwrap_or_else(|reason| {
            panic!(
                "给定状态表 {} 判不了（不退回别处判）：{reason}",
                table.table_name
            )
        });
    match judged_table.source {
        GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
        | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => panic!(
            "给定状态表 {} 交回的是占位结论：实验装置不分只录、只判两趟跑，也不分片（起它时不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE 与 \
             SINGLEFS_CRASH_AMPLIFICATION_SHARE）；多机配置开了 GPU 时要编进 gpu 特性起（不编进时判段不跑、只录）",
            table.table_name
        ),
    }
    judged_table.states
}
