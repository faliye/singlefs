// task-server-gpu-arguments: baseline row confirmed-loss-form
// 这三个模式经 `given_states_engine_of_this_experiment`（`crash_amplification::GivenStatesEngine`）判取样态的 O2；stream-constants 只驱动流、
// 回比本地常量，不判态、不要卡。带 gpu 编好的二进制直接交统一任务服务时，由服务按这一行清场、分卡。
//! E171（三oracle合谋测试） 第 1 次重跑：入库装置（登记 `research/prompts/e171-r1-prereg.md`，5.3「装置写在哪」）。
//!
//! 与 `e171_three_oracle_collusion/main.rs` 是两份装置、不 `#[path]` 引它，两份的本地常量（流一段长 `LOCAL_FIRST_STREAM_SEGMENT_LENGTHS` 这类）一样跟着 crates 改；比它改五处、加两样（登记 5.3）：
//! ① 卸载调理想模型的卸载答（`IdealModel::answer_unmount`，对拍用 `model_comparison::observed_unmount`）；
//! ② 每次成功调用的交回值与调用之后装置手里的实现对象，`{:?}` 文本的 sha256 记进 `call-outcomes.tsv` 新加的两列与 `call-return-values.tsv`（读法 4）；
//! ③ 调用 panic 与交回错误同样截断照判，态上的恢复 panic 不截断（读法 5）；
//! ④ O1 崩溃一列并列 RefFS 子列（读法 1，`oracle-verdicts.tsv` 新加的几列）；
//! ⑤ 流三 `first_file_unlinked_then_inodes_across_one_leaf`（读法 2）；
//! 加：`--writable-mount-probe`（Q4，读法 8，`writable-mount-probe.tsv`）与 `confirmed-loss-form` 模式（Q5，读法 7）。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness、format、replay-prefix 与 reference-filesystem。
//!
//!   e171_three_oracle_collusion_after_the_oracle_fixes baseline --output-directory <目录> [--streams 1,2,3] [--writable-mount-probe]
//!   e171_three_oracle_collusion_after_the_oracle_fixes row --output-directory <目录> [--streams 1,2,3] [--writable-mount-probe]
//!   e171_three_oracle_collusion_after_the_oracle_fixes confirmed-loss-form --output-directory <目录> [--truncate-at-the-return-point]
//!   e171_three_oracle_collusion_after_the_oracle_fixes stream-constants [--streams 1,2,3]
//!   baseline 与 row 之后可以加 `--positive-control <O1-对拍|O1-对拍-卸载|O1-崩溃判官|RefFS|O2|交回值|内存对象|panic|见证>`
//!   或 `--disable-oracle <O1|O2>`。
//!
//! `baseline`：回比本地常量（S1）、三批取样、写导出文件；`row`：不回比常量、只取批 1 与批 2。`stream-constants`：只驱动选中的流、
//! 打 S1 回比与 Q5 造法在基线上的几样（填本地常量用，不写导出、不判态）。`confirmed-loss-form`：流二驱到第 3 次调用（从 0 数）、
//! 照读法 7 造「已确认的丢了」那一形；方向相反的取样点末段全枚举，主取样点先过「等价类对拍」、再按等价类判（登记修订 2 第 2.1 节
//! 候选二、修订 3），每个判过的态算 O1 三个子列与附带的 O2。目录已存在就拒绝。
//! 标准输出：`E7RESULT` 行（与跑几次无关）与 `E7TIMING` 行（耗时）；收尾 `name=done emitted=N`。
//! 退出码：0 正常；2 命令行不对或输出目录已存在；3 停机条款 S1、S3、S5、S9、S13 没过，或 `confirmed-loss-form` 那一趟 Q5 没跑
//! （判决行带 `q5_not_run=`）（产物照打，交主 agent）。

/// 崩溃点重放取样过崩溃判定库的项（`crash_state_enumeration::sample_crash_states_on_the_cpu`）：项名与设点文件（这份装置的根文件，
/// 就是这一份；单测 `setting_file_tests` 拿编译期的 `file!()` 核它）。
const CRASH_POINT_REPLAY_SAMPLING_ITEM: CrashPointReplayItem<'static> = CrashPointReplayItem {
    name: "e171_three_oracle_collusion_after_the_oracle_fixes",
    setting_file:
        "crates/singlefs-implementation-experiments/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes/main.rs",
};

/// 这一次取样的项名：装置一趟按不同的段、不同的序号区间取样几十次，每次是一条不同的流，项名各给一个（库里同一个项名下只登一份清单，
/// 几次共用会撞；块键按内容算，下一趟照样复用判过的块）。
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

// RefFS（`crates/singlefs-reference-filesystem`，D13（验证路线） 已定项 3）：验证包直接依赖它（登记修订 2 第 2.3 节），
// 类型与 harness 翻译层用的是同一份；翻译层的 `events_of` 交不出本装置三条流的内容，所以照那份映射在本装置里交事件，
// 要直接用 RefFS 的类型。它自己的单测归它自己的包，不再编进本装置。
use singlefs_reference_filesystem as reference_filesystem;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator, RootRingOccupancy};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::JournalRecordPlaceInPublish;
use singlefs_core::make_filesystem::{
    root_ring_occupancy_after_make_filesystem, MakeFilesystemOutput, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
    ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::recovery::{
    choose_system_configuration, scan_journal, verified_system_configuration_slots, PoolReader,
    RecoveryOutcome, RecoveryReport,
};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite,
    publish_unlink_intent, publish_unlink_step, warm_up, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput, WarmUpOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_verification::crash::{
    check_records, classified_oracle_violation_for_versions, crash_point_replay_plan_state_count,
    crash_state_enumeration_tables, full_expansion,
    in_place_subsets_with_copy_on_write_none_or_all_expansion, CrashPointReplayOracleViolation,
    CrashStateEnumerationTables, CrashStateJudgement,
    CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
    CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE, CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER,
    CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER, RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT,
    RECORD_CHECKER_ROOT_WITHOUT_RECORD,
};
use singlefs_verification::crash_facts::{
    extract_flow_facts, persisted_writes_of_state, SegmentFact, TornImageFact,
    MAXIMUM_ENUMERATED_WRITES,
};
use singlefs_verification::crash_injection::{
    materialized_crash_image, writable_mount_alone_on, WritableMountAlone,
};
use singlefs_verification::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};
use singlefs_verification::history::StepPosition;
use singlefs_verification::known_bad_images::known_bad_images;
use singlefs_verification::memory_pool::{
    newest_persisted_root, writes_and_segments_with_stream_indexes_and_entries, CrashImage,
    MemoryPool, PublishedVersion, RecordCheck, RetainedWrite, WrittenContents,
};
use singlefs_verification::model::{
    crash_recovery_disagreement, IdealModel, ModelAnswer, ModelDeviceIdentity, ModelDisagreement,
    ModelPoolGeometry, ModelRootKey, ObservedEffect, ObservedFile, ObservedOutcome,
    ObservedReadBack,
};
use singlefs_verification::model_comparison::{
    model_root_key, observed_mount_carrying_to_zero_unit_publishes,
    observed_read_back_after_a_crash, observed_root_of_file_version, observed_unmount,
    refusal_reason_of_mount_error, refusal_reason_of_publish_error,
    refusal_reason_of_rollback_error, reported_ceiling_of_mount_error,
    root_ring_slot_still_bad_after_one_reread_of_mount_error,
    root_ring_slot_still_bad_after_one_reread_of_rollback_error,
};
use singlefs_verification::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state_of_the_pool,
    refusal_reason_on_the_legal_refusal_list, DiskAfterTheRefusal, ReadFaultsInTheCrashState,
    ReadFaultsUnderTheRefusedMount, ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet,
    RefusalReasonOnTheLegalRefusalList, WritableMountRefusalVerdict,
};
use singlefs_verification::reference_filesystem_translation::{
    crash_moment_of_a_segment_closed_at, segment_closing_stream_indexes,
    ReferenceFileSystemCrashMoment,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::sha256::{sha256_digest, sha256_hexadecimal, DIGEST_BYTES};
use singlefs_verification::{
    RecordedEntrySpan, RecordedOperationKind, RecordedPublishEntry, RetainedOperation, SharedStream,
};

mod call_outcomes_and_run_switches;
mod command_line_parsing;
mod confirmed_loss_class_evaluation;
mod confirmed_loss_form;
mod confirmed_loss_partition_and_recorded_reads;
mod crash_state_sampling_and_image_digests;
mod equivalence_pairing_and_piece_panic_control;
mod export_files;
#[cfg(test)]
mod helpers_shared_by_the_tests;
mod implementation_and_crash_judge_verdicts;
mod judging_pieces_and_panic_injection;
mod live_stream;
mod local_constants_and_stream_contents;
mod neutral_format_read_back;
mod pool_checker_through_the_given_states_engine;
mod prepared_stream_and_crates_checks;
mod reference_filesystem_vote;
mod result_lines;
mod state_observation;
mod stream_driver;
mod stream_export;
mod stream_report;
mod witness_criterion_and_writable_mount_probe;

use call_outcomes_and_run_switches::*;
use command_line_parsing::*;
use confirmed_loss_class_evaluation::*;
use confirmed_loss_form::*;
use confirmed_loss_partition_and_recorded_reads::*;
use crash_state_sampling_and_image_digests::*;
use equivalence_pairing_and_piece_panic_control::*;
use export_files::*;
#[cfg(test)]
use helpers_shared_by_the_tests::*;
use implementation_and_crash_judge_verdicts::*;
use judging_pieces_and_panic_injection::*;
use live_stream::*;
use local_constants_and_stream_contents::*;
use neutral_format_read_back::*;
use pool_checker_through_the_given_states_engine::*;
use prepared_stream_and_crates_checks::*;
use reference_filesystem_vote::*;
use result_lines::*;
use state_observation::*;
use stream_driver::*;
use stream_export::*;
use stream_report::*;
use witness_criterion_and_writable_mount_probe::*;

/// S1 里第 1 次重跑新加的、与流无关的几条：一片 inode 叶容器的记录数、每盘系统配置槽数与槽长，对 `singlefs_format` 的同名公开常量。
fn stream_independent_checks_of_the_first_rerun() -> Vec<StopCheck> {
    vec![
        StopCheck::new(
            "inode_leaf_records",
            LOCAL_INODE_LEAF_RECORDS,
            singlefs_format::INODE_LEAF_RECORDS,
        ),
        StopCheck::new(
            "system_configuration_slots_per_device",
            u64::try_from(LOCAL_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).expect("2"),
            singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
        ),
        StopCheck::new(
            "system_configuration_slot_bytes",
            LOCAL_SYSTEM_CONFIGURATION_SLOT_BYTES,
            singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES,
        ),
    ]
}

fn run_dump(command_line: &CommandLine) -> i32 {
    let run_kind = command_line.run_kind;
    let switches = command_line.switches;
    let mut lines = ResultLines {
        emitted: 0,
        collected: None,
    };
    lines.emit_result_line(&format!(
        "name=config segment=s1 mode={} streams={} sector_bytes={LOCAL_SECTOR_BYTES} first_batch_divisor={FIRST_BATCH_DIVISOR} second_batch_divisor={SECOND_BATCH_DIVISOR} third_batch_divisor={THIRD_BATCH_DIVISOR} torn_states_per_segment={TORN_STATES_PER_SEGMENT} torn_scan_ordinals_per_segment={TORN_SCAN_ORDINALS_PER_SEGMENT} third_batch_sampled={} positive_control={} disabled_oracle={} writable_mount_probe={}",
        if run_kind == RunKind::Baseline { "baseline" } else { "row" },
        command_line
            .streams
            .iter()
            .map(|stream| stream.number().to_string())
            .collect::<Vec<String>>()
            .join(","),
        run_kind == RunKind::Baseline,
        switches
            .positive_control
            .map_or_else(|| "none".to_string(), |control| format!("{control:?}")),
        switches
            .disabled_oracle
            .map_or_else(|| "none".to_string(), |oracle| format!("{oracle:?}")),
        switches.writable_mount_probe,
    ));
    let mut failed_checks = 0usize;
    if run_kind == RunKind::Baseline {
        let mut independent = stream_independent_checks();
        independent.extend(stream_independent_checks_of_the_first_rerun());
        independent.extend(confirmed_loss_form_checks_against_the_crates());
        for check in independent {
            failed_checks += usize::from(!check.holds());
            lines.emit_result_line(&format!(
                "name=s1 stream=independent check={} local={} crates={} holds={}",
                check.name,
                field_text(&check.local),
                field_text(&check.crates),
                check.holds()
            ));
        }
    }
    let mut totals = StreamTotals::default();
    let mut streams_not_exported = 0usize;
    let mut not_applicable_source_checks_failed = 0usize;
    let mut reference_filesystem_checks_failed = 0usize;
    for name in &command_line.streams {
        let started = Instant::now();
        let output_root = command_line.output_directory.clone();
        let exported = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            export_stream(*name, &output_root, run_kind, switches)
        }));
        let mut export = exported.unwrap_or_else(|payload| {
            StreamExport::not_exported(
                *name,
                format!("pipeline_panicked:{}", panic_text(&*payload)),
                started,
            )
        });
        streams_not_exported += usize::from(export.not_exported.is_some());
        let not_applicable_source_checks =
            not_applicable_source_checks_to_judge(&export, run_kind, switches);
        not_applicable_source_checks_failed += not_applicable_source_checks
            .iter()
            .filter(|check| !check.holds())
            .count();
        let reference_filesystem_checks = reference_filesystem_checks_of_a_stream(&export);
        reference_filesystem_checks_failed += reference_filesystem_checks
            .iter()
            .filter(|check| !check.holds())
            .count();
        export.checks.extend(not_applicable_source_checks);
        export.checks.extend(reference_filesystem_checks);
        let stream_totals = report_stream(&mut lines, &export);
        add_stream_totals(&mut totals, &stream_totals);
    }
    failed_checks += totals.failed_checks;
    lines.emit_result_line(&format!(
        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} crash_facts_mask_mismatches={} s3_o2_judgement_mismatches={} s9_failed_checks={reference_filesystem_checks_failed} streams_not_exported={streams_not_exported} o1_comparison_red_calls={} o1_crash_judge_red_states={} o1_reference_filesystem_red_states={} o2_red_states={} recovery_panic_states={} probed_states={} counter_branch_states={} counter_branch_states_judged_legal={} undecidable_branch_states={} witness_refusals_without_either_branch={} counter_branch_states_that_mounted={} piece_panic_states={} unattributed_panic_states={}",
        totals.rebuilt_digest_mismatches,
        totals.mask_mismatches,
        totals.pool_checker_direct_mismatches,
        totals.comparison_red_calls,
        totals.crash_judge_red_states,
        totals.reference_filesystem_red_states,
        totals.pool_checker_red_states,
        totals.recovery_panic_states,
        totals.probed_states,
        totals.counter_branch_states,
        totals.counter_branch_states_judged_legal,
        totals.undecidable_branch_states,
        totals.witness_refusals_without_either_branch,
        totals.counter_branch_states_that_mounted,
        piece_panic_states_text(&totals.piece_panic_states),
        totals.unattributed_panic_states,
    ));
    lines.finish();
    let writable_mount_probe_reconciliation_hit = totals.witness_refusals_without_either_branch > 0
        || totals.counter_branch_states_that_mounted > 0;
    let stop_clause_hit = failed_checks > 0
        || totals.rebuilt_digest_mismatches > 0
        || totals.pool_checker_direct_mismatches > 0
        || reference_filesystem_checks_failed > 0
        || writable_mount_probe_reconciliation_hit
        || (run_kind == RunKind::Baseline
            && (not_applicable_source_checks_failed > 0 || streams_not_exported > 0));
    if stop_clause_hit {
        3
    } else {
        0
    }
}

/// `stream-constants` 模式：只驱动、只回比（S1 的流相关几条与状态数几条、与流无关的几条、Q5 造法那几样），不写导出、不判态。
fn run_stream_constants(streams: &[StreamName]) -> i32 {
    let mut lines = ResultLines {
        emitted: 0,
        collected: None,
    };
    lines.emit_result_line("name=config segment=s1 mode=stream-constants");
    let mut failed_checks = 0usize;
    let mut checks = stream_independent_checks();
    checks.extend(stream_independent_checks_of_the_first_rerun());
    checks.extend(confirmed_loss_form_checks_against_the_crates());
    for name in streams {
        let stream = prepare_stream(*name, RunSwitches::default(), true);
        let tables = crash_state_enumeration_tables(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &full_expansion,
        );
        let full_states = tables.state_count;
        let in_place_subsets_states = crash_point_replay_plan_state_count(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &in_place_subsets_with_copy_on_write_none_or_all_expansion,
        );
        checks.extend(stream.checks.iter().cloned());
        checks.extend(state_count_checks_against_the_crates(
            *name,
            &stream,
            &tables,
            flow_facts_state_count(&stream, &tables),
            full_states,
            in_place_subsets_states,
        ));
        lines.emit_result_line(&format!(
            "name=stream_constants stream={} ending={} calls={} writes={} segments={} reference_filesystem_segments={}",
            name.directory_name(),
            ending_text(&stream.ending),
            stream.calls.len(),
            stream.writes.len(),
            stream.segments.len(),
            segment_closing_stream_indexes(&stream.operations).len()
        ));
    }
    for check in checks {
        failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    lines.emit_result_line(&format!("name=verdict s1_failed_checks={failed_checks}"));
    lines.finish();
    if failed_checks > 0 {
        3
    } else {
        0
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(parsed) = parse_command_line(&arguments) else {
        eprintln!(
            "用法：e171_three_oracle_collusion_after_the_oracle_fixes baseline|row --output-directory <目录> [--streams 1,2,3] [--writable-mount-probe] [--positive-control <名> | --disable-oracle O1|O2]；confirmed-loss-form --output-directory <目录> [--truncate-at-the-return-point]；stream-constants [--streams 1,2,3]；收到 {arguments:?}"
        );
        std::process::exit(2);
    };
    let output_directory = match &parsed {
        ParsedCommand::Dump(command_line) => Some(&command_line.output_directory),
        ParsedCommand::ConfirmedLossForm(command_line) => Some(&command_line.output_directory),
        ParsedCommand::StreamConstants(_) => None,
    };
    if let Some(directory) = output_directory.filter(|directory| directory.exists()) {
        eprintln!(
            "输出目录 {} 已存在：装置不覆盖已有的文件，换一个目录",
            directory.display()
        );
        std::process::exit(2);
    }
    let exit_code = match &parsed {
        ParsedCommand::Dump(command_line) => run_dump(command_line),
        ParsedCommand::StreamConstants(streams) => run_stream_constants(streams),
        ParsedCommand::ConfirmedLossForm(command_line) => run_confirmed_loss_form(command_line),
    };
    if exit_code != 0 {
        std::process::exit(exit_code);
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
