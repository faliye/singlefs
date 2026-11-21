//! E170（O3规约执行器第一版） 第一段：入库导出装置。
//!
//! 登记：`research/prompts/e170-preregistration.md` 5.3（导出格式）、5.4（取样三批）、5.7（装置写在哪）、5.8「第一段」、
//! 第十一节停机 S1、S4 与作废 V2、第九节 XE1–XE3。它回答问题单 Q2 的实现那一侧：两条崩溃点重放流的录制写流按 5.3 的中性格式写出，
//! 按 5.4 取三批状态，每个状态调 `crates/` 的崩溃点重放判定（`singlefs_verification::crash::judge_crash_image`）记实现的判定与单线程耗时。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness 与 format。
//! 流一（新池新建文件）在实验包库造的池上走（`singlefs_implementation_experiments::experiment_pool` 的 `format_pool` 只做 mkfs），
//! 取号、暖机、新池新建文件三步由这里自己调、每次调用返回时记流长（`calls.tsv` 的返回点），
//! 再拿验证包库的新池新建文件那条路（`singlefs_verification::scenario::run_new_pool_file_creation`，经
//! `experiment_pool::new_pool_file_creation_of_the_verification_library` 在内存盘上走一遍）整条流逐条回比（S1）；
//! 流二照抄 `crates/singlefs-verification/tests/crash_enumeration_fixed_script_stream.rs`
//! 的 `prepare` 里 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支（第 218 行起）。
//! 决定几何与工作量的数写成本地常量（值抄自那两份测试文件与 `tests/common/mod.rs`，逐个注明），`main` 开头与验证包库的同名
//! 公开常量或现算值回比（`singlefs_verification::scenario`、`known_bad_images`、`history`、`memory_pool`），不直接引（登记 5.7、S1）。
//!
//!   e170_specification_executor_export dump --output-directory <目录> [--compare-with <上一次导出的目录>]
//!   e170_specification_executor_export bad-states --dump-directory <dump 写出的目录> --seed-hex <第二段冻结的种子，16 个十六进制字符>
//!
//! 第三段（登记 5.5、5.8「第三段」）的 `bad-states` 模式见文件末尾「第三段：坏状态」一节：P 行、N 行、盲测族造在第一段导出的同一批
//! 流上，每行过一遍实现的判定，写进 `<目录>/<流名>/bad-*.tsv` 与 `bad-patches.bin`；回比没过同样退 3。
//!
//! 每条流写进 `<目录>/<流名>/`（流名 `new_pool_file_creation`、`fixed_script_through_e_then_unmount`），目录已存在就拒绝。
//! 给了 `--compare-with` 时逐个文件比 sha256（V2），另按登记第十二节修订比去掉耗时那一列之后的两个文件（V2b）。
//! 标准输出：`E7RESULT` 行（与跑几次无关，入库产物只留它们）与 `E7TIMING` 行（耗时与带耗时的文件摘要）；收尾 `name=done emitted=N`。
//! 退出码：0 正常；2 命令行不对或输出目录已存在；3 停机条款 S1 或 S4 没过（产物照打，交主 agent）。
//! 单线程：每个状态的 `judge_crash_image` 耗时在这个进程里用 `Instant` 计（登记 5.3 `impl-verdicts.tsv`）。

/// 崩溃点重放取样过崩溃判定库的项（`crash_state_enumeration::sample_crash_states_on_the_cpu`）：项名与设点文件（这份装置自己）。
const CRASH_POINT_REPLAY_SAMPLING_ITEM: CrashPointReplayItem<'static> = CrashPointReplayItem {
    name: "e170_o3_spec_executor_export",
    setting_file:
        "crates/singlefs-implementation-experiments/src/bin/e170_specification_executor_export.rs",
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

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::journal::{
    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
    JournalRecordPlaceInPublish, JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET,
};
use singlefs_core::make_filesystem::{
    root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
    ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::recovery::{PoolReader, RecoveryOutcome, RecoveryReport};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::system_configuration::{
    SystemConfiguration, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_verification::crash::{
    crash_point_replay_plan_state_count, crash_state_enumeration_tables, full_expansion,
    in_place_subsets_with_copy_on_write_none_or_all_expansion, judge_crash_image,
    CrashStateEnumerationTables, CrashStateJudgement,
};
use singlefs_verification::crash_facts::{extract_flow_facts, FlowFacts};
use singlefs_verification::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};
use singlefs_verification::memory_pool::{
    publishes_of_one_recording, writes_and_segments_with_stream_indexes_and_entries, CrashImage,
    MemoryPool, PublishWrites, PublishedVersion, RetainedWrite, WrittenContents,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::sha256::{sha256_digest, sha256_hexadecimal, DIGEST_BYTES};
use singlefs_verification::{
    RecordedEntrySpan, RecordedOperationKind, RecordedPublishEntry, RetainedOperation, SharedStream,
};

mod bad_rows;
mod bad_state_families_and_tables;
mod bad_state_observations_and_checks;
mod bad_states_mode;
mod bad_stream_facts;
mod driver_recorded_calls;
mod driving_the_two_streams;
mod file_contents_written_by_the_streams;
mod implementation_verdicts;
mod landing_vectors_and_image_digests;
mod local_constants;
mod neutral_format_write_and_rebuild;
mod sampling;
mod stream_export;

use bad_rows::*;
use bad_state_families_and_tables::*;
use bad_state_observations_and_checks::*;
use bad_states_mode::*;
use bad_stream_facts::*;
use driver_recorded_calls::*;
use driving_the_two_streams::*;
use file_contents_written_by_the_streams::*;
use implementation_verdicts::*;
use landing_vectors_and_image_digests::*;
use local_constants::*;
use neutral_format_write_and_rebuild::*;
use sampling::*;
use stream_export::*;

// ───────────────────────────── 结果行、V2 与 main ─────────────────────────────

/// `E7RESULT` 行计数（收尾 `name=done emitted=N`，N 把 done 自己也算进去）；`E7TIMING` 行不计数、不进入库产物。
struct ResultLines {
    emitted: u64,
}

impl ResultLines {
    fn emit_result_line(&mut self, body: &str) {
        self.emitted += 1;
        println!("E7RESULT {body}");
    }
    fn timing(body: &str) {
        println!("E7TIMING {body}");
    }
    fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
    }
}

/// 结果行里的值不许带空白（按空格切字段）。
fn field_text(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

/// S1 里与流无关的几条：本地常量对验证包库的公开常量或现算值（实验包的造池取的也是这几样，回比不经它）。盘宽是库给新池新建文件那一类池定的
/// 那一档（`history::HistoryDeviceWidth::FourGibibytes`，库里写明与 `tests/common/mod.rs` 的文件镜像同宽）；mkfs 参数是
/// `known_bad_images::parameters_of_the_common_test_pool`；固定写时刻、第一个文件的字节数与内容是 `scenario` 那一份。
fn stream_independent_checks() -> Vec<StopCheck> {
    let parameters = singlefs_verification::known_bad_images::parameters_of_the_common_test_pool();
    vec![
        StopCheck::new(
            "device_bytes",
            LOCAL_DEVICE_BYTES,
            singlefs_verification::history::HistoryDeviceWidth::FourGibibytes.device_bytes(),
        ),
        StopCheck::new(
            "physical_block_bytes",
            LOCAL_PHYSICAL_BLOCK_BYTES,
            parameters.geometry.physical_block_size,
        ),
        StopCheck::new(
            "minimum_input_output_bytes",
            LOCAL_MINIMUM_INPUT_OUTPUT_BYTES,
            parameters.geometry.minimum_input_output_bytes,
        ),
        StopCheck::new(
            "sector_bytes",
            LOCAL_SECTOR_BYTES,
            singlefs_verification::memory_pool::SECTOR_BYTES,
        ),
        StopCheck::new(
            "filesystem_identifier",
            LOCAL_FILESYSTEM_IDENTIFIER,
            singlefs_verification::known_bad_images::E142_FILESYSTEM_IDENTIFIER,
        ),
        StopCheck::new(
            "fixed_write_time_seconds",
            LOCAL_FIXED_WRITE_TIME_SECONDS,
            singlefs_verification::scenario::FIXED_WRITE_TIME_SECONDS,
        ),
        StopCheck::new(
            "first_file_bytes",
            LOCAL_FIRST_FILE_BYTES,
            singlefs_verification::scenario::FIRST_FILE_BYTES,
        ),
        StopCheck::new(
            "first_file_content",
            true,
            local_first_file_content() == singlefs_verification::scenario::first_file_content(),
        ),
    ]
}

/// 中位取排好序之后下标 ⌊(n − 1)/2⌋ 那一个，p95 取下标 ⌈0.95·n⌉ − 1 那一个。
fn timing_summary(mut nanoseconds: Vec<u128>) -> (u128, u128, u128, u128) {
    if nanoseconds.is_empty() {
        return (0, 0, 0, 0);
    }
    nanoseconds.sort_unstable();
    let count = nanoseconds.len();
    let total: u128 = nanoseconds.iter().sum();
    let ninety_fifth_percentile_index = (count * 95).div_ceil(100) - 1;
    (
        total,
        nanoseconds[(count - 1) / 2],
        nanoseconds[ninety_fifth_percentile_index],
        nanoseconds[count - 1],
    )
}

/// V2：同一个文件两次导出逐字节比；V2b：带耗时的两个文件去掉耗时之后再比。交回（文件名, 逐字节相同, 去掉耗时之后相同）。
fn compare_with_previous_export(
    directory: &Path,
    previous_directory: &Path,
) -> Vec<(&'static str, bool, Option<bool>)> {
    FILES_LISTED_IN_THE_MANIFEST
        .iter()
        .chain(std::iter::once(&MANIFEST_FILE))
        .map(|file_name| {
            let this_run = read_file(directory, file_name);
            let previous_run = read_file(previous_directory, file_name);
            let without_timing = contents_without_timing(file_name, &this_run)
                .map(|kept| Some(kept) == contents_without_timing(file_name, &previous_run));
            (*file_name, this_run == previous_run, without_timing)
        })
        .collect()
}

struct CommandLine {
    output_directory: PathBuf,
    compare_with: Option<PathBuf>,
}

fn parse_command_line(arguments: &[String]) -> Option<CommandLine> {
    let mut words = arguments.iter();
    if words.next().map(String::as_str) != Some("dump") {
        return None;
    }
    let mut output_directory = None;
    let mut compare_with = None;
    while let Some(word) = words.next() {
        match word.as_str() {
            "--output-directory" => output_directory = Some(PathBuf::from(words.next()?)),
            "--compare-with" => compare_with = Some(PathBuf::from(words.next()?)),
            _ => return None,
        }
    }
    Some(CommandLine {
        output_directory: output_directory?,
        compare_with,
    })
}

/// 一条流的结果行（`E7RESULT` 与 `E7TIMING`）；交回 (S1 没过的条数, S4 对不上的态数, 掩码对不上的态数, 两次判定不同的态数, 实现没两判都放行的态数)。
fn report_stream(lines: &mut ResultLines, export: &StreamExport) -> [usize; 5] {
    let label = export.name.directory_name();
    let mut failed_checks = 0usize;
    for check in &export.checks {
        failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream={label} check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    lines.emit_result_line(&format!(
        "name=stream stream={label} operations={} writes={} segments={} tearable_writes={} calls={} full_states={} in_place_subsets_states={} sampled_states={}",
        export.operation_count,
        export.write_count,
        export.segment_count,
        export.tearable_write_count,
        export.calls.len(),
        export.full_states,
        export.in_place_subsets_states,
        export.observations.len()
    ));
    for (call_index, call) in export.calls.iter().enumerate() {
        lines.emit_result_line(&format!(
            "name=call stream={label} call={call_index} label={} first_operation={} return_point={} status=ok",
            call.label, call.first_operation, call.return_point
        ));
    }
    for scan in &export.torn_scans {
        lines.emit_result_line(&format!(
            "name=torn_scan stream={label} segment={} scanned={} found={} wanted={TORN_STATES_PER_SEGMENT}",
            scan.segment_index, scan.scanned, scan.found
        ));
    }
    let mut not_both_pass = 0usize;
    for batch in 1..=3u8 {
        let in_batch: Vec<&StateObservation> = export
            .observations
            .iter()
            .filter(|observation| observation.batch == batch)
            .collect();
        let count_where = |predicate: &dyn Fn(&StateObservation) -> bool| {
            in_batch
                .iter()
                .filter(|observation| predicate(observation))
                .count()
        };
        let red_item_states = |prefix: &str| {
            count_where(&|observation| {
                observation
                    .verdict
                    .red_items
                    .iter()
                    .any(|item| item.starts_with(prefix))
            })
        };
        let both_pass = count_where(&|observation| {
            observation
                .verdict
                .pool_checker_and_record_checker_both_pass()
        });
        not_both_pass += in_batch.len() - both_pass;
        let segments: BTreeSet<usize> = in_batch
            .iter()
            .map(|observation| observation.segment_index)
            .collect();
        lines.emit_result_line(&format!(
            "name=batch stream={label} batch={batch} states={} segments_touched={} states_with_a_torn_write={} pool_checker_and_record_checker_both_pass={both_pass} pool_checker_red_states={} record_checker_red_states={} consulted_oracle_red_states={} ignored_oracle_red_states={} consulted_effective_root_none={} consulted_failed_outcomes={}",
            in_batch.len(),
            segments.len(),
            count_where(&|observation| observation.landing_vector.contains('1')),
            red_item_states("pool_checker:"),
            red_item_states("record_checker:"),
            red_item_states("journal_consulted_oracle:"),
            red_item_states("journal_ignored_oracle:"),
            count_where(&|observation| observation.verdict.consulted_root == "none"),
            count_where(&|observation| observation.verdict.consulted_outcome.starts_with("failed:")),
        ));
        let (total, median, ninety_fifth_percentile, maximum) = timing_summary(
            in_batch
                .iter()
                .map(|observation| observation.judge_nanoseconds)
                .collect(),
        );
        ResultLines::timing(&format!(
            "name=judge_time stream={label} batch={batch} states={} total_ns={total} median_ns={median} p95_ns={ninety_fifth_percentile} max_ns={maximum}",
            in_batch.len()
        ));
    }
    let mask_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.mask_matches_the_facts)
        .count();
    let rejudge_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.rejudge_matches)
        .count();
    lines.emit_result_line(&format!(
        "name=s4 stream={label} rebuilt_states={} digest_mismatches={}",
        export.rebuilt_states, export.rebuilt_digest_mismatches
    ));
    lines.emit_result_line(&format!(
        "name=crash_facts_mask stream={label} states={} mismatches={mask_mismatches}",
        export.observations.len()
    ));
    lines.emit_result_line(&format!(
        "name=rejudge stream={label} states={} mismatches={rejudge_mismatches}",
        export.observations.len()
    ));
    for file in &export.files {
        match &file.sha256_without_timing {
            None => lines.emit_result_line(&format!(
                "name=file stream={label} file={} bytes={} sha256={}",
                file.name, file.bytes, file.sha256
            )),
            Some(without_timing) => {
                lines.emit_result_line(&format!(
                    "name=file stream={label} file={} sha256_without_timing={without_timing}",
                    file.name
                ));
                ResultLines::timing(&format!(
                    "name=file_with_timing stream={label} file={} bytes={} sha256={}",
                    file.name, file.bytes, file.sha256
                ));
            }
        }
    }
    ResultLines::timing(&format!(
        "name=stream_time stream={label} elapsed_ns={}",
        export.elapsed_nanoseconds
    ));
    [
        failed_checks,
        export.rebuilt_digest_mismatches,
        mask_mismatches,
        rejudge_mismatches,
        not_both_pass,
    ]
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let first_word = arguments.first().map(String::as_str);
    if first_word == Some(BadTableVersion::FirstRun.mode_word())
        || first_word == Some(BadTableVersion::SecondRun.mode_word())
    {
        let Some(bad_states_command_line) = parse_bad_states_command_line(&arguments) else {
            eprintln!(
                "用法：e170_specification_executor_export bad-states（或 bad-states-second-run）--dump-directory <第一段的导出目录> --seed-hex <16 个十六进制字符>；收到 {arguments:?}"
            );
            std::process::exit(2);
        };
        std::process::exit(run_bad_states(&bad_states_command_line));
    }
    let Some(command_line) = parse_command_line(&arguments) else {
        eprintln!(
            "用法：e170_specification_executor_export dump --output-directory <目录> [--compare-with <上一次导出的目录>]，或 bad-states --dump-directory <目录> --seed-hex <种子>；收到 {arguments:?}"
        );
        std::process::exit(2);
    };
    if command_line.output_directory.exists() {
        eprintln!(
            "输出目录 {} 已存在：导出不覆盖已有的文件，换一个目录",
            command_line.output_directory.display()
        );
        std::process::exit(2);
    }
    let mut lines = ResultLines { emitted: 0 };
    lines.emit_result_line(&format!(
        "name=config segment=s1 streams={} sector_bytes={LOCAL_SECTOR_BYTES} first_batch_divisor={FIRST_BATCH_DIVISOR} second_batch_divisor={SECOND_BATCH_DIVISOR} third_batch_divisor={THIRD_BATCH_DIVISOR} torn_states_per_segment={TORN_STATES_PER_SEGMENT} torn_scan_ordinals_per_segment={TORN_SCAN_ORDINALS_PER_SEGMENT} judge_threads=1 compared_with_a_previous_export={}",
        STREAM_NAMES.len(),
        command_line.compare_with.is_some()
    ));
    let mut failed_checks = 0usize;
    for check in stream_independent_checks() {
        failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream=both check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let mut totals = [0usize; 5];
    let mut differing_files_against_the_compared_export = 0usize;
    let mut differing_files_against_the_compared_export_without_timing = 0usize;
    for name in STREAM_NAMES {
        let export = export_stream(name, &command_line.output_directory);
        let stream_totals = report_stream(&mut lines, &export);
        for (total, value) in totals.iter_mut().zip(stream_totals) {
            *total += value;
        }
        if let Some(previous) = &command_line.compare_with {
            let label = name.directory_name();
            let comparisons = compare_with_previous_export(
                &command_line.output_directory.join(label),
                &previous.join(label),
            );
            for (file_name, identical, without_timing) in comparisons {
                differing_files_against_the_compared_export += usize::from(!identical);
                lines.emit_result_line(&format!(
                    "name=v2 stream={label} file={file_name} identical={identical}"
                ));
                if let Some(identical_without_timing) = without_timing {
                    differing_files_against_the_compared_export_without_timing +=
                        usize::from(!identical_without_timing);
                    lines.emit_result_line(&format!(
                        "name=v2_without_timing stream={label} file={file_name} identical={identical_without_timing}"
                    ));
                }
            }
        }
    }
    failed_checks += totals[0];
    let [_, rebuilt_digest_mismatches, mask_mismatches, rejudge_mismatches, not_both_pass] = totals;
    lines.emit_result_line(&format!(
        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={rebuilt_digest_mismatches} crash_facts_mask_mismatches={mask_mismatches} rejudge_mismatches={rejudge_mismatches} states_where_pool_checker_or_record_checker_is_red={not_both_pass} v2_compared={} v2_differing_files={differing_files_against_the_compared_export} v2_differing_files_without_timing={differing_files_against_the_compared_export_without_timing}",
        command_line.compare_with.is_some()
    ));
    lines.finish();
    if failed_checks > 0 || rebuilt_digest_mismatches > 0 {
        std::process::exit(3);
    }
}

#[cfg(test)]
mod tests;
