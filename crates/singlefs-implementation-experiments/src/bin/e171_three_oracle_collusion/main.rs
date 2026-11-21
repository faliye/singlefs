// task-server-gpu-arguments: baseline row known-bad-images
// 三个模式的判段都经 `given_states_engine_of_this_experiment`（`crash_amplification::GivenStatesEngine`）判盘：baseline 与 row 判取样态的 O2，
// known-bad-images 判已知坏镜像与 E170（O3规约执行器第一版）的坏状态；带 gpu 编好的二进制直接交统一任务服务时，由服务按这一行清场、分卡。
// 录入段（`--stage record`）不起引擎、不碰卡，判段（`--stage judge`）才在卡上判；这一行按模式词认，分不出段。
//! E171（三oracle合谋测试） 第一段：入库装置。
//!
//! 登记：`research/prompts/e171-preregistration.md` 5.3（装置写在哪）、5.5（坏镜像表）、5.6（阳性与阴性对照）、5.8「第一段」、
//! 第十一节停机 S1–S6 与 S8。它回答问题单 Q1–Q3 里 O1、O2 两列的那一半：两条崩溃点重放流（新池新建文件、固定脚本到 E 再正常卸载）上，
//! 每次 API 调用之后拿理想模型的答案与实现的结局比（O1-对拍），每个取样的崩溃状态上拿实现只读恢复读回的那一版问理想模型
//! （O1-崩溃判官），再拿池级 checker 判这一态的崩溃镜像（O2）；O3（Python 执行器）读本装置照 E170（O3规约执行器第一版） 登记 5.3 写出的
//! 中性格式，不在这个文件里。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness、format 与 replay-prefix。
//! 两条流的驱动、取样、中性格式的写出与读回照抄 `e170_specification_executor_export.rs`（不 `#[path]` 引它），改三处（登记 5.3）：
//! ① 驱动里的 API 调用不 `.expect`：结局（成功 / 错误成员 / panic）记进 `calls.tsv` 的 `ok` / `err` 列与 `call-outcomes.tsv`，
//! 出错之后这条流不再往下驱动；② 跟着每次调用推理想模型，记 O1-对拍；③ 每个取样的状态算 O1-崩溃判官与 O2，写 `oracle-verdicts.tsv`。
//!
//!   e171_three_oracle_collusion baseline --output-directory <目录>
//!   e171_three_oracle_collusion row --output-directory <目录>
//!   e171_three_oracle_collusion known-bad-images --dump-directory <基线导出目录> --output-directory <目录> [--e170-bad-rows]
//!   以上三种之后都可以加 `--positive-control <O1-对拍|O1-崩溃判官|O2>` 或 `--disable-oracle <O1|O2>`（只对 baseline 与 row）
//!   以及 `--sampling-threads <正整数>`：baseline 与 row 判取样态、known-bad-images 判已知坏镜像与 E170 坏状态的线程数，不给取每台默认重活线程数，都再按内存上限收（`sampling_threads_of_this_run`）；
//!   写 1 是一条线程按序号走完的那一趟，产物（`E7RESULT` 行与去掉耗时之后的文件）与几条线程的逐字相同
//!   以及 `--stage record|judge`（三种模式都认，取值照 `crash_library::RunStage` 的写法）：`record` 只跑录入段，`judge` 只跑判段，不给就两段先后跑。
//!
//! 两段（D13（验证路线） 已定项 15「判器上 GPU」；`.claude/rules/verification.md`「一个门禁：两种跑法」按需领卡那一句）：
//! 录入段只在 CPU 上跑：驱动、取样、O1、导出文件都在这一段，O2 要判的盘一张一张作给定状态表录进判定库（prune-server 里那一份，
//! 给定状态表那一类、按表的摘要），不起给定状态引擎、不碰卡；这一段的观测与结果行要的东西写成一份交接，放在输出目录里
//! （[`HAND_OFF_TO_THE_JUDGE_STAGE_FILE`]）。判段不再驱动、不再取样：读回交接，凭摘要从库里读回每张盘、交引擎判，写判完才有的产物
//! （`oracle-verdicts.tsv`、`known-bad-o2.tsv`、`e170-bad-rows-o2.tsv`，坏镜像模式的另两个产物也在这一段写），打全部 `E7RESULT` 行，删掉交接。
//! 不给 `--stage` 时两段先后跑，交接在内存里交，产物与结果行与分两趟跑的逐字节相同（去掉耗时之后）。
//! 判器的结论只进产物与结果行，不回头定取样与驱动：三种模式都分得开。
//! 段不用 `SINGLEFS_CRASH_AMPLIFICATION_STAGE` 点：取样那一步（`sample_crash_states_on_the_cpu`）照那个变量选段，设了它取样就只录不判。
//!
//! `baseline`：回比本地常量（S1）、三批取样、写十个导出文件加 `oracle-verdicts.tsv` 与 `call-outcomes.tsv`；`row`：同上，只是
//! 不回比常量（坏实现那一趟本来就会让写流与段表变）、只取批 1 与批 2；S5 在这一趟也判：返回了错误的调用（卸载除外）都记成 O1-对拍的红
//! （取号、暖机出错也是），不适用只来自卸载与对拍判出不同之后（`not_applicable_source_checks_of_a_bad_implementation_row`）。目录已存在就拒绝。
//! 标准输出：`E7RESULT` 行（与跑几次无关）与 `E7TIMING` 行（耗时）；收尾 `name=done emitted=N`。
//! 退出码：0 正常（只跑录入段的那一趟录完就是 0，停机条款归判段）；2 命令行不对、要录的输出目录已存在、只跑判段而录入段没跑过
//! （或交接与这一趟的参数对不上）；3 停机条款 S1、S3、S4、S5 没过（产物照打，交主 agent）。

/// 崩溃点重放取样过崩溃判定库的项（`crash_state_enumeration::sample_crash_states_on_the_cpu`）：项名与设点文件（这份装置的根文件，
/// 就是这一份；单测 `setting_file_tests` 拿编译期的 `file!()` 核它）。
const CRASH_POINT_REPLAY_SAMPLING_ITEM: CrashPointReplayItem<'static> = CrashPointReplayItem {
    name: "e171_three_oracle_collusion",
    setting_file:
        "crates/singlefs-implementation-experiments/src/bin/e171_three_oracle_collusion/main.rs",
};

/// 这一次取样的项名：装置一趟按不同的段、不同的序号区间取样几十次，每次是一条不同的流，项名各给一个（库里同一个项名下只登一份清单，
/// 几次共用会撞；块键按内容算，下一趟照样复用判过的块）。号按调用的次序发：`observe_sampled_states` 在起线程之前按连续段的次序取好，
/// 几条线程与一条线程的那一趟每个连续段拿到同一个项名、录入键相同。
fn sampling_item_name_of_this_call() -> String {
    static NEXT_SAMPLING_NUMBER: std::sync::atomic::AtomicU64 =
        std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        CRASH_POINT_REPLAY_SAMPLING_ITEM.name,
        NEXT_SAMPLING_NUMBER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    )
}

use singlefs_implementation_experiments::experiment_pool;

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::make_filesystem::{
    root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
    ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::recovery::{PoolReader, RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput, WarmUpOutput,
};
use singlefs_verification::crash::{
    crash_point_replay_plan_state_count, crash_state_enumeration_tables, full_expansion,
    in_place_subsets_with_copy_on_write_none_or_all_expansion, CrashStateEnumerationTables,
    CrashStateJudgement,
};
use singlefs_verification::crash_amplification::RecordedGivenStatesTableWithItsColumns;
use singlefs_verification::crash_facts::{extract_flow_facts, FlowFacts};
use singlefs_verification::crash_library::{LibraryOpenInThisProcess, RunStage};
use singlefs_verification::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};
use singlefs_verification::known_bad_images::known_bad_images;
use singlefs_verification::memory_pool::{
    newest_persisted_root, writes_and_segments_with_stream_indexes_and_entries, CrashImage,
    MemoryPool, PublishedVersion, RetainedWrite, WrittenContents,
};
use singlefs_verification::model::{
    crash_recovery_disagreement, IdealModel, ModelAnswer, ModelDeviceIdentity, ModelDisagreement,
    ModelPoolGeometry, ModelRootKey, ObservedEffect, ObservedFile, ObservedOutcome,
    ObservedReadBack,
};
use singlefs_verification::model_comparison::{
    model_root_key, observed_mount_carrying_to_zero_unit_publishes,
    observed_read_back_after_a_crash, observed_root_of_file_version, refusal_reason_of_mount_error,
    refusal_reason_of_publish_error, refusal_reason_of_rollback_error,
    reported_ceiling_of_mount_error, root_ring_slot_still_bad_after_one_reread_of_mount_error,
    root_ring_slot_still_bad_after_one_reread_of_rollback_error,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::sha256::{sha256_digest, sha256_hexadecimal, DIGEST_BYTES};
use singlefs_verification::{
    RecordedEntrySpan, RecordedOperationKind, RecordedPublishEntry, RetainedOperation, SharedStream,
};

mod call_outcomes_and_run_switches;
mod command_line_parsing;
mod crash_state_sampling_and_image_digests;
mod export_files;
mod hand_off_to_the_judge_stage;
#[cfg(test)]
mod helpers_shared_by_the_tests;
mod known_bad_images_mode;
mod live_stream_preparation;
mod local_constants_and_stream_contents;
mod neutral_format_read_back;
mod oracle_verdicts_of_a_state;
mod result_lines_and_stream_report;
mod sampled_state_observation;
mod sampling_thread_count_and_ordered_outcomes;
mod stream_driver;
mod stream_export;

use call_outcomes_and_run_switches::*;
use command_line_parsing::*;
use crash_state_sampling_and_image_digests::*;
use export_files::*;
use hand_off_to_the_judge_stage::*;
#[cfg(test)]
use helpers_shared_by_the_tests::*;
use known_bad_images_mode::*;
use live_stream_preparation::*;
use local_constants_and_stream_contents::*;
use neutral_format_read_back::*;
use oracle_verdicts_of_a_state::*;
use result_lines_and_stream_report::*;
use sampled_state_observation::*;
use sampling_thread_count_and_ordered_outcomes::*;
use stream_driver::*;
use stream_export::*;

/// 流一的段长（12 段）：取号两轮各一段（2、2；D22（单元原子性怎么合成） 已定项 8「取号写两轮」），暖机两次各是记录段 2、根槽段 1、轮换段 2，
/// A 是 24 个单元写一段、记录 2、根槽 1、轮换 2。`tests/crash_enumeration_new_pool_file_creation_stream.rs` 的段长断言是同一张表。
const LOCAL_FIRST_STREAM_SEGMENT_LENGTHS: [usize; 12] = [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];

/// 照 `stage` 跑两段，交回退出码：
/// - 只跑录入段：`record` 交出交接，写进输出目录里的 [`HAND_OFF_TO_THE_JUDGE_STAGE_FILE`]，不判，交回 [`EXIT_CODE_OF_A_RECORD_ONLY_RUN`]；
/// - 只跑判段：不跑 `record`，从那个文件读回交接交 `judge`，判完删掉那个文件（`judge` 拒了或 panic 了就留着，可以只重跑判段）；
/// - 两段先后跑：`record` 交出的交接在内存里写成字节、读回，交 `judge`（与分两趟同一种交法，不落文件）。
///
/// # Errors
/// 只跑判段而读不到交接（录入段没跑过：报错句点名先跑录入段）、交接读回不成（[`hand_off_of_the_bytes`]）、`judge` 拒了
/// （交接里录的盘与库里的对不上）；交接写不进、删不掉。
fn run_in_stages<HandOff: HandOffField>(
    stage: RunStage,
    run_arguments: &[String],
    output_directory: &Path,
    record: impl FnOnce() -> HandOff,
    judge: impl FnOnce(HandOff) -> Result<i32, String>,
) -> Result<i32, String> {
    let hand_off_path = output_directory.join(HAND_OFF_TO_THE_JUDGE_STAGE_FILE);
    match stage {
        RunStage::RecordOnly => {
            let bytes = hand_off_bytes(run_arguments, &record());
            std::fs::create_dir_all(output_directory)
                .map_err(|error| format!("建 {}：{error}", output_directory.display()))?;
            std::fs::write(&hand_off_path, bytes.as_slice())
                .map_err(|error| format!("交接写不进 {}：{error}", hand_off_path.display()))?;
            Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN)
        }
        RunStage::JudgeOnly => {
            let bytes = std::fs::read(&hand_off_path).map_err(|error| {
                format!(
                    "读不到录入段留下的交接 {}（{error}）：先跑录入段（{STAGE_ARGUMENT} record，或不给 {STAGE_ARGUMENT}、两段先后跑）",
                    hand_off_path.display()
                )
            })?;
            let hand_off = hand_off_of_the_bytes(&bytes, run_arguments)?;
            let exit_code = judge(hand_off)?;
            std::fs::remove_file(&hand_off_path)
                .map_err(|error| format!("交接删不掉 {}：{error}", hand_off_path.display()))?;
            Ok(exit_code)
        }
        RunStage::RecordThenJudge => {
            let bytes_handed_over_in_memory = hand_off_bytes(run_arguments, &record());
            judge(hand_off_of_the_bytes(
                &bytes_handed_over_in_memory,
                run_arguments,
            )?)
        }
    }
}

/// 从命令行里取出 `--stage <段>`（至多一次，取值照 `RunStage::of_the_variable_value`：`record`、`judge`），交回段与其余的参数
/// （进交接、两段核是不是同一条命令的那一份）。写错、给了两次交回 `None`。
fn stage_and_the_run_arguments(arguments: &[String]) -> Option<(RunStage, Vec<String>)> {
    let mut stage_word: Option<&str> = None;
    let mut run_arguments = Vec::new();
    let mut words = arguments.iter();
    while let Some(word) = words.next() {
        if word == STAGE_ARGUMENT {
            if stage_word.is_some() {
                return None;
            }
            stage_word = Some(words.next()?.as_str());
        } else {
            run_arguments.push(word.clone());
        }
    }
    let stage = RunStage::of_the_variable_value(stage_word).ok()?;
    Some((stage, run_arguments))
}

/// `known-bad-images` 模式照 `stage` 跑（[`run_in_stages`]：录入段 [`record_the_known_bad_images`]，判段 [`judge_the_known_bad_images`]）。
///
/// # Errors
/// 同 [`run_in_stages`]。
fn run_known_bad_images(
    command_line: &KnownBadImagesCommandLine,
    stage: RunStage,
    run_arguments: &[String],
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    result_lines_output: &mut dyn std::io::Write,
) -> Result<i32, String> {
    run_in_stages(
        stage,
        run_arguments,
        &command_line.output_directory,
        || record_the_known_bad_images(command_line, tables),
        |recorded| judge_the_known_bad_images(command_line, recorded, tables, result_lines_output),
    )
}

/// 录入段交给判段的 `baseline`、`row` 那一趟：与流无关的 S1 回比（只在基线那一趟有）与两条流各自导出之后的样子（O2 录下了、没判）。
struct RecordedDump {
    stream_independent_checks: Vec<StopCheck>,
    exports: Vec<StreamExport>,
}

/// `baseline`、`row` 的录入段：与流无关的 S1 回比（只在基线那一趟），两条流各导出一遍（[`export_stream`]：驱动、取样、写录入段写得了的导出文件，
/// O2 要判的盘录进 `tables`）；流水线里 panic 的那条流记成没导出。
fn record_the_dump(
    command_line: &CommandLine,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
) -> RecordedDump {
    let stream_independent_checks = if command_line.run_kind == RunKind::Baseline {
        stream_independent_checks()
    } else {
        Vec::new()
    };
    let exports = STREAM_NAMES
        .iter()
        .map(|name| {
            let started = Instant::now();
            let exported = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                export_stream(
                    *name,
                    &command_line.output_directory,
                    command_line.run_kind,
                    command_line.switches,
                    tables,
                )
            }));
            exported.unwrap_or_else(|payload| {
                StreamExport::not_exported(
                    *name,
                    format!("pipeline_panicked:{}", panic_text(&*payload)),
                    started,
                )
            })
        })
        .collect();
    RecordedDump {
        stream_independent_checks,
        exports,
    }
}

/// `baseline`、`row` 的判段：每条流录下的盘交 `tables` 判（[`judge_the_recorded_pool_checker_tables`]）；引擎在某一态上失败的流照两段之前记成
/// 某个状态上实现自崩（[`StreamExport::record_an_engine_failure_in_a_state`]），导出了的流写 `oracle-verdicts.tsv`，没导出的流清掉留给判段判的观测；
/// 再照一趟写完时的次序打全部结果行（S5 在这里判），交回退出码。结果行写进 `result_lines_output`。
///
/// # Errors
/// 交接与录下的对不上（[`PoolCheckerTablesOfTheTwoStages::judge_a_recorded_table`] 的 `Err`）：判段一个文件都不写、一行结果行都不打就交回那一句。
fn judge_the_dump(
    command_line: &CommandLine,
    recorded: RecordedDump,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    result_lines_output: &mut dyn std::io::Write,
) -> Result<i32, String> {
    let run_kind = command_line.run_kind;
    let switches = command_line.switches;
    let RecordedDump {
        stream_independent_checks,
        mut exports,
    } = recorded;
    let threads = sampling_threads_of_this_run(switches);
    let judged_streams = exports
        .iter_mut()
        .map(|export| {
            judge_the_recorded_pool_checker_tables(
                &mut export.observations,
                tables,
                threads,
                switches,
            )
        })
        .collect::<Result<Vec<JudgedTablesOfAStream>, String>>()?;
    for (export, judged) in exports.iter_mut().zip(judged_streams) {
        let directory = command_line
            .output_directory
            .join(export.name.directory_name());
        match judged {
            JudgedTablesOfAStream::AllJudged => {}
            JudgedTablesOfAStream::EngineFailedInAState { what } => {
                export.record_an_engine_failure_in_a_state(&directory, &what);
            }
        }
        if export.not_exported.is_none() {
            write_the_oracle_verdicts_file(&directory, &export.observations, &mut export.files);
        } else {
            export.observations = Vec::new();
        }
    }
    let mut lines = ResultLines {
        emitted: 0,
        output: result_lines_output,
    };
    lines.emit_result_line(&format!(
        "name=config segment=s1 mode={} streams={} sector_bytes={LOCAL_SECTOR_BYTES} first_batch_divisor={FIRST_BATCH_DIVISOR} second_batch_divisor={SECOND_BATCH_DIVISOR} third_batch_divisor={THIRD_BATCH_DIVISOR} torn_states_per_segment={TORN_STATES_PER_SEGMENT} torn_scan_ordinals_per_segment={TORN_SCAN_ORDINALS_PER_SEGMENT} third_batch_sampled={} positive_control={} disabled_oracle={}",
        if run_kind == RunKind::Baseline { "baseline" } else { "row" },
        STREAM_NAMES.len(),
        run_kind == RunKind::Baseline,
        switches
            .positive_control
            .map_or_else(|| "none".to_string(), |control| format!("{control:?}")),
        switches
            .disabled_oracle
            .map_or_else(|| "none".to_string(), |oracle| format!("{oracle:?}")),
    ));
    let mut failed_checks = 0usize;
    for check in &stream_independent_checks {
        failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream=both check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let mut totals = [0usize; 7];
    let mut streams_not_exported = 0usize;
    let mut not_applicable_source_checks_failed = 0usize;
    for mut export in exports {
        streams_not_exported += usize::from(export.not_exported.is_some());
        let not_applicable_source_checks =
            not_applicable_source_checks_to_judge(&export, run_kind, switches);
        not_applicable_source_checks_failed += not_applicable_source_checks
            .iter()
            .filter(|check| !check.holds())
            .count();
        export.checks.extend(not_applicable_source_checks);
        let stream_totals = report_stream(&mut lines, &export);
        for (total, value) in totals.iter_mut().zip(stream_totals) {
            *total += value;
        }
    }
    failed_checks += totals[0];
    let [_, rebuilt_digest_mismatches, mask_mismatches, pool_checker_direct_mismatches, red_calls, crash_judge_red, pool_checker_red] =
        totals;
    lines.emit_result_line(&format!(
        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={rebuilt_digest_mismatches} crash_facts_mask_mismatches={mask_mismatches} s3_o2_judgement_mismatches={pool_checker_direct_mismatches} streams_not_exported={streams_not_exported} o1_comparison_red_calls={red_calls} o1_crash_judge_red_states={crash_judge_red} o2_red_states={pool_checker_red}"
    ));
    lines.finish();
    let stop_clause_hit = failed_checks > 0
        || rebuilt_digest_mismatches > 0
        || pool_checker_direct_mismatches > 0
        || (run_kind == RunKind::Baseline
            && (not_applicable_source_checks_failed > 0 || streams_not_exported > 0));
    if stop_clause_hit {
        Ok(3)
    } else {
        Ok(0)
    }
}

/// `baseline`、`row` 照 `stage` 跑（[`run_in_stages`]：录入段 [`record_the_dump`]，判段 [`judge_the_dump`]）。
///
/// # Errors
/// 同 [`run_in_stages`]。
fn run_dump(
    command_line: &CommandLine,
    stage: RunStage,
    run_arguments: &[String],
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    result_lines_output: &mut dyn std::io::Write,
) -> Result<i32, String> {
    run_in_stages(
        stage,
        run_arguments,
        &command_line.output_directory,
        || record_the_dump(command_line, tables),
        |recorded| judge_the_dump(command_line, recorded, tables, result_lines_output),
    )
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some((stage, parsed, run_arguments)) =
        stage_and_the_run_arguments(&arguments).and_then(|(stage, run_arguments)| {
            parse_command_line(&run_arguments).map(|parsed| (stage, parsed, run_arguments))
        })
    else {
        eprintln!(
            "用法：e171_three_oracle_collusion baseline|row --output-directory <目录> [--positive-control O1-对拍|O1-崩溃判官|O2 | --disable-oracle O1|O2] [--sampling-threads <正整数>]，或 known-bad-images --dump-directory <基线导出目录> --output-directory <目录> [--e170-bad-rows] [--sampling-threads <正整数>]；都可以加 {STAGE_ARGUMENT} record|judge；收到 {arguments:?}"
        );
        std::process::exit(2);
    };
    let output_directory = match &parsed {
        ParsedCommand::Dump(command_line) => &command_line.output_directory,
        ParsedCommand::KnownBadImages(command_line) => &command_line.output_directory,
    };
    if stage.records() && output_directory.exists() {
        eprintln!(
            "输出目录 {} 已存在：装置不覆盖已有的文件，换一个目录（只跑判段的那一趟才接着录入段那一趟的目录：{STAGE_ARGUMENT} judge）",
            output_directory.display()
        );
        std::process::exit(2);
    }
    let tables = GivenStatesTablesInTheLibrary;
    let ran = match &parsed {
        ParsedCommand::Dump(command_line) => run_dump(
            command_line,
            stage,
            &run_arguments,
            &tables,
            &mut std::io::stdout(),
        ),
        ParsedCommand::KnownBadImages(command_line) => run_known_bad_images(
            command_line,
            stage,
            &run_arguments,
            &tables,
            &mut std::io::stdout(),
        ),
    };
    match ran {
        Ok(0) => {}
        Ok(exit_code) => std::process::exit(exit_code),
        Err(refusal) => {
            eprintln!("{refusal}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod setting_file_tests {
    use super::CRASH_POINT_REPLAY_SAMPLING_ITEM;
    use singlefs_verification::crash_amplification::repository_relative_file;

    /// 取样项的设点文件是编进这个二进制的根文件（仓内路径）：节点身份与录下的流按这条路径记，写成一份不在的文件，库里的点就挂在一个不存在的名字上。
    #[test]
    fn the_sampling_item_names_the_root_file_compiled_into_this_binary_as_its_setting_file() {
        assert_eq!(
            CRASH_POINT_REPLAY_SAMPLING_ITEM.setting_file,
            repository_relative_file(file!()),
            "设点文件要是这一份根文件"
        );
    }
}
