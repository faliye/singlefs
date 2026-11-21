//! 备好的流：流的收尾、从跑着的流备好它与流的形状回比 crates。

use super::*;

/// 一条流驱动完之后的样子：mkfs 之后的基镜像、mkfs 之后的录制流、写表与段、写表每项在录制流里的下标、调用表、被判的根、版本表、
/// 模型提交过的版本目录、S1 回比，以及驱动怎么结束的。
pub(super) struct PreparedStream {
    pub(super) name: StreamName,
    pub(super) base: MemoryPool,
    pub(super) operations: Vec<RetainedOperation>,
    pub(super) writes: Vec<RetainedWrite>,
    pub(super) segments: Vec<Vec<usize>>,
    pub(super) stream_indexes: Vec<usize>,
    pub(super) calls: Vec<RecordedCall>,
    pub(super) judged_root_index: usize,
    pub(super) versions: Vec<PublishedVersion>,
    pub(super) committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    /// 模型不再往前走的那次调用（`None`：整条流的每次调用都比过了）。
    pub(super) model_stopped_at_call: Option<usize>,
    pub(super) checks: Vec<StopCheck>,
    pub(super) ending: StreamEnding,
}

/// 驱动怎么结束的：每一步都成功；某一步交回错误、或 panic（这条流都在那里截断，照常导出、照判，读法 5）；
/// 驱动没出错却接不下去（可写挂载交回的现行版本不带文件，同样截断照判）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum StreamEnding {
    Completed,
    StoppedByAnError { call_index: usize, member: String },
    StoppedByAPanic { call_index: usize, message: String },
    StoppedWithoutAnError { what: String },
}

/// 驱动怎么结束的：看最后一次调用的结局与驱动记下的「接不下去」。
pub(super) fn ending_of_the_driver(driver: &StreamDriver) -> StreamEnding {
    if let Some(what) = &driver.stopped_without_an_error {
        return StreamEnding::StoppedWithoutAnError { what: what.clone() };
    }
    match driver.calls.last().map(|call| &call.outcome) {
        Some(CallOutcome::ReturnedError(member)) => StreamEnding::StoppedByAnError {
            call_index: driver.calls.len() - 1,
            member: format!("err:{member}"),
        },
        Some(CallOutcome::Panicked(message)) => StreamEnding::StoppedByAPanic {
            call_index: driver.calls.len() - 1,
            message: message.clone(),
        },
        Some(CallOutcome::Succeeded | CallOutcome::InFlightAtTheTruncation) | None => {
            StreamEnding::Completed
        }
    }
}

/// 驱动一条流并取出它的基镜像、写表与段。`check_against_the_crates` 只在基线那一趟开：流一整条流逐条与验证包库的新池新建文件那条路比（S1）。
pub(super) fn prepare_stream(
    name: StreamName,
    switches: RunSwitches,
    check_against_the_crates: bool,
) -> PreparedStream {
    let tag = format!("e171r1-{}", name.directory_name());
    let mut live = LiveStream::start(name, &tag, switches);
    live.run_the_script();
    prepared_stream_of_the_live_stream(name, live, check_against_the_crates)
}

/// 驱动完的一条流取出基镜像、写表与段（[`prepare_stream`] 与 Q5 那一形共用）。
fn prepared_stream_of_the_live_stream(
    name: StreamName,
    mut live: LiveStream,
    check_against_the_crates: bool,
) -> PreparedStream {
    let ending = ending_of_the_driver(&live.driver);
    let all_operations = live.formatted.retained_operations();
    let mkfs_operation_count = live.formatted.mkfs_operation_count;
    // 照抄 `tests/crash_enumeration_fixed_script_stream.rs` 的 `writes_and_segments_after_mkfs`：入口段（卸载入口发的那几段）一起交给切段（C557）。
    let entry_spans: Vec<RecordedEntrySpan> = live
        .formatted
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - mkfs_operation_count
                ..span.operations.end - mkfs_operation_count,
        })
        .collect();
    let operations = all_operations[mkfs_operation_count..].to_vec();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations,
        &entry_spans,
        &experiment_pool::geometry(),
    );
    let base = live.formatted.memory_pool_after_mkfs();
    let mut checks = std::mem::take(&mut live.checks);
    if check_against_the_crates {
        checks.extend(stream_checks_against_the_crates(
            name,
            &live,
            &all_operations,
            &operations,
            &writes,
            &segments,
            &stream_indexes,
            &base,
        ));
    }
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .unwrap_or(usize::MAX);
    let versions = if name == StreamName::NewPoolFileCreation && judged_root_index != usize::MAX {
        // 流一的版本表照 `tests/crash_enumeration_new_pool_file_creation_stream.rs` 的 `single_version`。
        let (instance, checkpoint_txg) =
            singlefs_verification::memory_pool::root_identity_of_write(&writes[judged_root_index]);
        vec![PublishedVersion {
            instance,
            checkpoint_txg,
            content: local_first_file_content(),
        }]
    } else {
        std::mem::take(&mut live.versions)
    };
    PreparedStream {
        name,
        base,
        operations,
        writes,
        segments,
        stream_indexes,
        calls: live.driver.calls.clone(),
        judged_root_index,
        versions,
        committed_versions: std::mem::take(&mut live.driver.committed_versions),
        model_stopped_at_call: live.driver.model_stopped_at_call,
        checks,
        ending,
    }
}

/// S1 里跟流有关的几条（只在基线那一趟）：段长表、写数、根槽写数、盘数与盘字节数对本地常量；流一整条流逐条与验证包库的新池新建文件那条路比。
/// 与 `e170_specification_executor_export.rs` 的 `prepare_stream` 里那一批 S1 回比同一批。
#[allow(
    clippy::too_many_arguments,
    reason = "这几样是 prepare_stream 驱动完之后手里同时有的，拆成结构体只会多一层搬运"
)]
fn stream_checks_against_the_crates(
    name: StreamName,
    live: &LiveStream,
    all_operations: &[RetainedOperation],
    operations: &[RetainedOperation],
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    stream_indexes: &[usize],
    base: &MemoryPool,
) -> Vec<StopCheck> {
    let mkfs_operation_count = live.formatted.mkfs_operation_count;
    let label = name.directory_name();
    let non_barrier_operations: Vec<usize> = operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| retained.operation.kind != RecordedOperationKind::Barrier)
        .map(|(stream_index, _)| stream_index)
        .collect();
    let (local_segment_lengths, local_writes, local_root_writes): (&[usize], usize, usize) =
        match name {
            StreamName::NewPoolFileCreation => (
                &LOCAL_FIRST_STREAM_SEGMENT_LENGTHS,
                LOCAL_FIRST_STREAM_WRITES,
                LOCAL_FIRST_STREAM_ROOT_WRITES,
            ),
            StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount => (
                &LOCAL_SECOND_STREAM_SEGMENT_LENGTHS,
                LOCAL_SECOND_STREAM_WRITES,
                LOCAL_SECOND_STREAM_ROOT_WRITES,
            ),
            StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf => (
                LOCAL_THIRD_STREAM_SEGMENT_LENGTHS,
                LOCAL_THIRD_STREAM_WRITES,
                LOCAL_THIRD_STREAM_ROOT_WRITES,
            ),
        };
    let root_write_count = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .count();
    let mut checks = vec![
        StopCheck::new(
            &format!("{label}_write_order_is_the_non_barrier_operation_order"),
            true,
            non_barrier_operations == stream_indexes,
        ),
        StopCheck::new(
            &format!("{label}_segment_lengths"),
            local_segment_lengths.to_vec(),
            segments.iter().map(Vec::len).collect::<Vec<usize>>(),
        ),
        StopCheck::new(&format!("{label}_writes"), local_writes, writes.len()),
        StopCheck::new(
            &format!("{label}_root_writes"),
            local_root_writes,
            root_write_count,
        ),
        StopCheck::new(
            &format!("{label}_device_count"),
            LOCAL_DEVICE_COUNT,
            base.devices.len(),
        ),
        StopCheck::new(
            &format!("{label}_device_bytes"),
            LOCAL_DEVICE_BYTES,
            base.device_size_in_bytes,
        ),
    ];
    if name == StreamName::NewPoolFileCreation {
        // 流一的驱动走的是新池新建文件那三步：整条流（含 mkfs）逐条与验证包库的新池新建文件那条路比（不拿实验包自己的造池比自己）。
        let reference = experiment_pool::new_pool_file_creation_of_the_verification_library();
        checks.push(StopCheck::new(
            "new_pool_file_creation_operations_equal_the_verification_scenario",
            true,
            reference.operations == all_operations,
        ));
        checks.push(StopCheck::new(
            "new_pool_file_creation_mkfs_operations_equal_the_verification_scenario",
            mkfs_operation_count,
            reference.mkfs_operation_count,
        ));
        checks.push(StopCheck::new(
            "new_pool_file_creation_warm_up_return_point_equal_the_verification_scenario",
            live.warm_up_operation_count,
            reference.warm_up_operation_count,
        ));
    }
    checks
}
