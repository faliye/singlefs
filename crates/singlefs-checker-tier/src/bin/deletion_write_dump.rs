//! 里程碑三·6（`.claude/kb/milestone/03-third-txn.md` 第三节「三·6」）干跑的 `crates/` 侧只读导出：删除写出的字节。
//!
//! 在两块内存盘上依次跑 `singlefs_harness::scenario::run_deletion_history` 的几段历史（E142 那套参数、
//! 三个数据单元的第一个文件、只供测试的批大小 1；不带参数跑 H1、H2，`--history <名>` 只跑点名的那一段；
//! H3 在顺序写之前多建一个 size 0 的 2 号 inode、意图的调用方时间晚顺序写 60 秒，
//! 它的 `name=impl_config` 多打一格 `intent_write_time_seconds=`，E172 跑前登记第五节 5.3），把 unlink 之后的窗口（意图那次发布之前录制流的长度起，到这段历史结束为止）
//! 里**每一次写**按（步号、设备、字节偏移、字节长度、种类、sha256、整段十六进制）原样打出来，屏障各打一行，
//! 每一段（一次发布，或不属于任何一次发布的几次写）前后各打一行边界：发布的边界带 txg、哪一种发布、这次重写的角色。
//! **不带任何位置寻址的布局知识**：不按偏移归类；段的边界与种类来自场景函数（直接调的发布按调用前后录制流的长度，
//! 卸载与挂载里的几次按 core 交回的每次发布写账切开），角色名来自 core 交回的 `TransactionOutput::rewritten`。
//! 步号从窗口起点数，窗口起点在录制流里的下标打在 `name=before_window_summary` 的 `operations=`。
//!
//! 只驱动、只观测：不改 `singlefs-core`、`singlefs-checker`、`singlefs-format`。
//!
//!   deletion_write_dump                    导 H1、H2（E172 第一段那一份，`research/scripts/replay.sh` 里 E172 那一行逐字节比它）
//!   deletion_write_dump --history <名>     只导点名的那一段，名字是 h1、h2、h3 之一（E172 第三段用 `--history h3`）
//!
//! H3 不进不带参数的导出：进了，E172 第一段那一份与它的复跑就对不上了。
//! 不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行不对（不认识的参数、`--history` 后面缺名字、
//! 名字不是 h1 / h2 / h3、名字后面还有参数），stderr 打原因与用法，一行都不导；101 = 某段历史在内存盘上没走完（panic 带原因）。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_format::TEST_IMAGE_DEFAULT_BYTES;
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::{
    deletion_history_batch_size, e142_parameters, run_deletion_history,
    three_data_unit_file_content, DeletionHistory, DeletionHistoryIntentWriteTime,
    DeletionHistorySegment, E142_FILESYSTEM_IDENTIFIER, FIXED_WRITE_TIME_SECONDS,
    SECONDS_FROM_THE_FIRST_FILE_TO_THE_SEQUENTIAL_WRITE_IN_THE_DELETION_HISTORIES,
};
use singlefs_harness::sha256::sha256_hexadecimal;
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

/// E142 那套几何：两块同构盘，`physical_block_size` = io_min = 512（与 `e142_new_pool_file_creation_write_dump` 同一组值）。
const DEVICE_COUNT: u32 = 2;
const PHYSICAL_BLOCK_SIZE_IN_BYTES: u32 = 512;
const MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;

/// 点名一段历史的参数，后面跟 `DeletionHistory::name()` 那个名字。
const HISTORY_OPTION: &str = "--history";

/// 不带参数时导的历史：E172 第一段那一份导出就是这两段。
const HISTORIES_DUMPED_WITHOUT_ARGUMENTS: [DeletionHistory; 2] = [
    DeletionHistory::IntentAndEveryBatchInProcess,
    DeletionHistory::IntentThenUnmountThenMountContinuation,
];

/// 命令行不对时的退出码（文件头「退出码」那一句）。
const EXIT_CODE_OF_A_REFUSED_COMMAND_LINE: i32 = 1;

/// 这一次导哪几段历史。
#[derive(Debug, PartialEq, Eq)]
enum DumpedHistories {
    /// 不带参数：[`HISTORIES_DUMPED_WITHOUT_ARGUMENTS`]。
    HistoriesOfTheE172FirstSegment,
    /// `--history <名>`：只导这一段。
    OneNamedHistory(DeletionHistory),
}

impl DumpedHistories {
    fn histories_in_dump_order(&self) -> Vec<DeletionHistory> {
        match self {
            DumpedHistories::HistoriesOfTheE172FirstSegment => {
                HISTORIES_DUMPED_WITHOUT_ARGUMENTS.to_vec()
            }
            DumpedHistories::OneNamedHistory(history) => vec![*history],
        }
    }
}

/// 命令行哪里不对。
#[derive(Debug, PartialEq, Eq)]
enum CommandLineRefusal {
    /// 第一个参数不是 [`HISTORY_OPTION`]。
    UnknownArgument(String),
    /// [`HISTORY_OPTION`] 后面没有名字。
    HistoryOptionWithoutAName,
    /// 名字不是任何一段历史的 `DeletionHistory::name()`。
    UnknownHistoryName(String),
    /// 名字后面还有参数。
    ArgumentsAfterTheHistoryName(Vec<String>),
}

/// 把命令行（不含程序名）解成这一次导哪几段历史。
fn parse_dumped_histories(arguments: &[String]) -> Result<DumpedHistories, CommandLineRefusal> {
    match arguments {
        [] => Ok(DumpedHistories::HistoriesOfTheE172FirstSegment),
        [option] if option == HISTORY_OPTION => Err(CommandLineRefusal::HistoryOptionWithoutAName),
        [option, history_name] if option == HISTORY_OPTION => DeletionHistory::EVERY_HISTORY
            .into_iter()
            .find(|history| history.name() == history_name)
            .map(DumpedHistories::OneNamedHistory)
            .ok_or_else(|| CommandLineRefusal::UnknownHistoryName(history_name.clone())),
        [option, _history_name, arguments_after_the_name @ ..] if option == HISTORY_OPTION => Err(
            CommandLineRefusal::ArgumentsAfterTheHistoryName(arguments_after_the_name.to_vec()),
        ),
        [first_argument, ..] => Err(CommandLineRefusal::UnknownArgument(first_argument.clone())),
    }
}

struct Emitter {
    emitted: u64,
}

impl Emitter {
    fn emit(&mut self, body: &str) {
        self.emitted += 1;
        println!("E7RESULT {body}");
    }
    fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
    }
}

/// 一段前后那两行边界的正文（不带 `edge=` 与 `step=`）。
fn segment_boundary_fields(segment: &DeletionHistorySegment) -> (String, String) {
    match segment {
        DeletionHistorySegment::Publish { kind, version, .. } => (
            "publish_boundary".to_string(),
            format!(
                "publish={} txg={} rewritten_units={} rewritten_roles={} data_units_after={} deleted_inodes_after={} inode_record_after={}",
                kind.name(),
                version.root.checkpoint_txg.0,
                version.rewritten.len(),
                version
                    .rewritten
                    .iter()
                    .map(|role| role.tag())
                    .collect::<Vec<_>>()
                    .join(","),
                version.data_pointers.len(),
                version.deleted_inodes.len(),
                version.inode_record.is_some(),
            ),
        ),
        DeletionHistorySegment::WritesOutsideAnyPublish { kind, .. } => (
            "writes_outside_any_publish_boundary".to_string(),
            format!("segment={}", kind.name()),
        ),
    }
}

/// 一段历史：跑、打窗口里的每一步与每一段的边界。
fn dump_one_history(history: DeletionHistory, emitter: &mut Emitter) {
    let parameters = e142_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0
        ..DEVICE_COUNT)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(
                        TEST_IMAGE_DEFAULT_BYTES,
                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE_IN_BYTES),
                    ),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let run = run_deletion_history(history, &parameters, &mut devices, &stream)
        .unwrap_or_else(|failure| panic!("{} 在内存盘上没走完：{failure:?}", history.name()));
    let history_name = history.name();
    let content_bytes = three_data_unit_file_content().len();
    // 打的是场景交给意图那次发布的调用方时间（`DeletionHistoryRun::intent_write_time_seconds`），不从 core 交回的记录里读：
    // 比对侧拿它判记录里写的是不是调用方给的那一个。H1、H2 不打这一格，它们的导出与 E172 第一段那一份逐字节相同。
    let intent_write_time_field = match history.intent_write_time() {
        DeletionHistoryIntentWriteTime::TheFileRecordsCurrentWriteTime => String::new(),
        DeletionHistoryIntentWriteTime::SecondsAfterTheSequentialWrite => {
            format!(
                " intent_write_time_seconds={}",
                run.intent_write_time_seconds
            )
        }
    };
    emitter.emit(&format!(
        "name=impl_config history={history_name} devices={DEVICE_COUNT} image_bytes={TEST_IMAGE_DEFAULT_BYTES} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} region_device_0={} region_device_1={} region_device_2={} slots_per_region={} journal_ring_bytes={} first_file_bytes_before_the_sequential_write={} file_bytes={content_bytes} batch_branch={} batch_data_units={} write_time_seconds={FIXED_WRITE_TIME_SECONDS} sequential_write_time_seconds={} filesystem_identifier={} mkfs_operations={}{intent_write_time_field}",
        parameters.region_devices[0].0,
        parameters.region_devices[1].0,
        parameters.region_devices[2].0,
        parameters.geometry.root_ring_slots_per_region.count(),
        parameters.geometry.journal_ring_bytes,
        singlefs_harness::scenario::FIRST_FILE_BYTES,
        deletion_history_batch_size().name(),
        deletion_history_batch_size().data_units_per_batch(),
        FIXED_WRITE_TIME_SECONDS
            + SECONDS_FROM_THE_FIRST_FILE_TO_THE_SEQUENTIAL_WRITE_IN_THE_DELETION_HISTORIES,
        hexadecimal_text(&E142_FILESYSTEM_IDENTIFIER),
        run.mkfs_operation_count,
    ));

    let all_operations = stream.retained_operations();
    let window_start = run.operations_before_the_unlink;
    let before_window = &all_operations[..window_start];
    emitter.emit(&format!(
        "name=before_window_summary history={history_name} operations={} writes={}",
        before_window.len(),
        before_window
            .iter()
            .filter(|retained| retained.operation.kind != RecordedOperationKind::Barrier)
            .count(),
    ));

    let mut write_count = 0u64;
    let mut barrier_count = 0u64;
    let mut publish_count = 0u64;
    let mut next_step_expected = window_start;
    for segment in &run.segments {
        let operations = segment.operations();
        assert_eq!(
            operations.start, next_step_expected,
            "场景函数交回的各段首尾相接，从窗口起点铺起"
        );
        next_step_expected = operations.end;
        let (line_name, fields) = segment_boundary_fields(segment);
        if matches!(segment, DeletionHistorySegment::Publish { .. }) {
            publish_count += 1;
        }
        emitter.emit(&format!(
            "name={line_name} history={history_name} edge=before step={} {fields}",
            operations.start - window_start
        ));
        for absolute_step in operations.clone() {
            let retained = &all_operations[absolute_step];
            let operation = &retained.operation;
            let step = absolute_step - window_start;
            if operation.kind == RecordedOperationKind::Barrier {
                barrier_count += 1;
                emitter.emit(&format!(
                    "name=window_barrier history={history_name} step={step} device={}",
                    operation.device.0
                ));
                continue;
            }
            write_count += 1;
            let bytes = retained
                .written_bytes_rebuilding_zero_fills()
                .expect("write / write_fua / write_zeroes 都能重建出完整字节");
            emitter.emit(&format!(
                "name=device_region_bytes history={history_name} step={step} device={} offset={} length={} kind={} sha256={} hexadecimal={}",
                operation.device.0,
                operation.offset.0,
                operation.length,
                operation.kind.name(),
                sha256_hexadecimal(&bytes),
                hexadecimal_text(&bytes),
            ));
        }
        emitter.emit(&format!(
            "name={line_name} history={history_name} edge=after step={} {fields}",
            operations.end - window_start
        ));
    }
    assert_eq!(
        next_step_expected,
        all_operations.len(),
        "场景函数交回的各段铺到录制流的尾上"
    );
    emitter.emit(&format!(
        "name=device_region_bytes_summary history={history_name} window_operations={} writes={write_count} barriers={barrier_count} segments={} publishes={publish_count}",
        all_operations.len() - window_start,
        run.segments.len(),
    ));
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let dumped_histories = parse_dumped_histories(&arguments).unwrap_or_else(|refusal| {
        let history_names = DeletionHistory::EVERY_HISTORY
            .map(DeletionHistory::name)
            .join(" | ");
        eprintln!(
            "deletion_write_dump 命令行不对：{refusal:?}（收到 {arguments:?}）；用法：不带参数（导 h1、h2）或 {HISTORY_OPTION} <{history_names}>"
        );
        std::process::exit(EXIT_CODE_OF_A_REFUSED_COMMAND_LINE)
    });
    let mut emitter = Emitter { emitted: 0 };
    for history in dumped_histories.histories_in_dump_order() {
        dump_one_history(history, &mut emitter);
    }
    emitter.finish();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command_line(arguments: &[&str]) -> Vec<String> {
        arguments
            .iter()
            .map(|argument| (*argument).to_string())
            .collect()
    }

    #[test]
    fn no_arguments_dump_h1_then_h2_and_leave_h3_out() {
        let dumped_histories = parse_dumped_histories(&[]).expect("不带参数是认得的命令行");
        assert_eq!(
            dumped_histories.histories_in_dump_order(),
            vec![
                DeletionHistory::IntentAndEveryBatchInProcess,
                DeletionHistory::IntentThenUnmountThenMountContinuation,
            ],
            "不带参数导 H1、H2（这个次序），不导 H3：E172 第一段那一份导出与它的逐字节复跑就是这两段"
        );
    }

    #[test]
    fn the_history_option_with_h3_dumps_only_h3() {
        let dumped_histories = parse_dumped_histories(&command_line(&["--history", "h3"]))
            .expect("`--history h3` 是认得的命令行");
        assert_eq!(
            dumped_histories.histories_in_dump_order(),
            vec![DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess],
            "`--history h3` 只导 H3 一段"
        );
    }

    #[test]
    fn the_history_option_dumps_exactly_the_history_whose_name_it_gives() {
        for history in DeletionHistory::EVERY_HISTORY {
            assert_eq!(
                parse_dumped_histories(&command_line(&["--history", history.name()])),
                Ok(DumpedHistories::OneNamedHistory(history)),
                "`--history {}` 只点名那一段",
                history.name()
            );
        }
    }

    #[test]
    fn an_unknown_first_argument_is_refused() {
        assert_eq!(
            parse_dumped_histories(&command_line(&["h3"])),
            Err(CommandLineRefusal::UnknownArgument("h3".to_string())),
            "名字前面没有 `--history` 不认"
        );
        assert_eq!(
            parse_dumped_histories(&command_line(&["--history=h3"])),
            Err(CommandLineRefusal::UnknownArgument(
                "--history=h3".to_string()
            )),
            "`--history=<名>` 这种写法不认"
        );
    }

    #[test]
    fn the_history_option_without_a_name_is_refused() {
        assert_eq!(
            parse_dumped_histories(&command_line(&["--history"])),
            Err(CommandLineRefusal::HistoryOptionWithoutAName),
            "`--history` 后面缺名字不认"
        );
    }

    #[test]
    fn a_history_name_that_is_not_h1_h2_or_h3_is_refused() {
        assert_eq!(
            parse_dumped_histories(&command_line(&["--history", "h4"])),
            Err(CommandLineRefusal::UnknownHistoryName("h4".to_string())),
            "h4 不是任何一段历史的名字"
        );
        assert_eq!(
            parse_dumped_histories(&command_line(&["--history", "H3"])),
            Err(CommandLineRefusal::UnknownHistoryName("H3".to_string())),
            "名字按 `DeletionHistory::name()` 原样比，大写不认"
        );
    }

    #[test]
    fn arguments_after_the_history_name_are_refused() {
        assert_eq!(
            parse_dumped_histories(&command_line(&["--history", "h3", "h1"])),
            Err(CommandLineRefusal::ArgumentsAfterTheHistoryName(vec![
                "h1".to_string()
            ])),
            "一次只点名一段，名字后面多出的参数不认"
        );
    }
}
