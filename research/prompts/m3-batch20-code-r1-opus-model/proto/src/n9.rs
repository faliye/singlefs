//! N9：删除导出在冻结树上的输出（不起 checker 档的 bin：把 `crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs` 第一个
//! `use` 起到 `fn main` 之前那一段原样拷进来，`main` 换成按参数选历史），与入库的导出逐字节比在原型外做。
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

pub fn run() {
    let which = std::env::var("M3B20_N9_HISTORY").unwrap_or_default();
    let dumped = if which.is_empty() {
        DumpedHistories::HistoriesOfTheE172FirstSegment
    } else {
        parse_dumped_histories(&["--history".to_string(), which]).expect("认得的历史名")
    };
    let mut emitter = Emitter { emitted: 0 };
    for history in dumped.histories_in_dump_order() {
        dump_one_history(history, &mut emitter);
    }
    emitter.finish();
}
