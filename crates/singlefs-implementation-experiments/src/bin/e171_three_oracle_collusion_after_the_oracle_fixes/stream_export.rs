//! 一条流的导出与它的状态数回比 crates。

use super::*;

/// 一条流导出之后的样子。`not_exported` 有值：这条流没有导出状态（没有根槽写可判、或流水线里别处 panic），
/// 只有 `call-outcomes.tsv`、`call-return-values.tsv` 与 `stream-status.tsv`（能写出来的话）。
pub(super) struct StreamExport {
    pub(super) name: StreamName,
    pub(super) checks: Vec<StopCheck>,
    pub(super) calls: Vec<RecordedCall>,
    pub(super) ending: Option<StreamEnding>,
    pub(super) model_stopped_at_call: Option<usize>,
    pub(super) not_exported: Option<String>,
    pub(super) operation_count: usize,
    pub(super) write_count: usize,
    pub(super) segment_count: usize,
    pub(super) tearable_write_count: usize,
    pub(super) full_states: u64,
    pub(super) in_place_subsets_states: u64,
    pub(super) torn_scans: Vec<TornScan>,
    pub(super) observations: Vec<StateObservation>,
    pub(super) files: Vec<WrittenFile>,
    pub(super) rebuilt_digest_mismatches: usize,
    pub(super) rebuilt_states: usize,
    /// RefFS 一侧切的段数（`segment_closing_stream_indexes`，A9 与装置切的段数比）与翻译停在哪一步（没停写 `-`）。
    pub(super) reference_filesystem_segments: usize,
    pub(super) reference_filesystem_stopped: String,
    pub(super) elapsed_nanoseconds: u128,
    /// 驱动这条流（含每次调用与推模型、O1-对拍）、判一批状态、写出导出文件各花多久（装置进程里 `Instant` 计）。
    pub(super) drive_nanoseconds: u128,
    pub(super) observe_nanoseconds: u128,
    pub(super) write_nanoseconds: u128,
}

impl StreamExport {
    /// 什么都没导出的一条流：流水线在驱动之后、导出之前 panic 时用它占位。
    pub(super) fn not_exported(name: StreamName, why: String, started: Instant) -> Self {
        Self {
            name,
            checks: Vec::new(),
            calls: Vec::new(),
            ending: None,
            model_stopped_at_call: None,
            not_exported: Some(why),
            operation_count: 0,
            write_count: 0,
            segment_count: 0,
            tearable_write_count: 0,
            full_states: 0,
            in_place_subsets_states: 0,
            torn_scans: Vec::new(),
            observations: Vec::new(),
            files: Vec::new(),
            rebuilt_digest_mismatches: 0,
            rebuilt_states: 0,
            reference_filesystem_segments: 0,
            reference_filesystem_stopped: "-".to_string(),
            elapsed_nanoseconds: started.elapsed().as_nanos(),
            drive_nanoseconds: 0,
            observe_nanoseconds: 0,
            write_nanoseconds: 0,
        }
    }
}

/// 基线与坏实现那一趟的差别：基线回比本地常量、取三批；坏实现那一趟不回比（写流与段表本来就会变）、只取批 1 与批 2。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RunKind {
    Baseline,
    BadImplementationRow,
}

/// 判哪些态：基线与逐行那一趟照登记 5.4 取批（逐行那一趟不取批 3）；Q5 那一形取末段整段全枚举再加最后那个全持久态（读法 7），都记在批 1；
/// Q5 主取样点按等价类判（登记修订 2 第 2.1 节候选二、修订 3）只判点名的那几态（各类的代表与「C 全落」那一态），记在批 1、留整份恢复报告。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum StateSelection {
    RegisteredBatches { third_batch: bool },
    WholeLastSegmentAndTheFullyPersistedState,
    ListedStatesKeepingTheConsultedReports(BTreeSet<u64>),
}

pub(super) fn export_stream(
    name: StreamName,
    output_root: &Path,
    run_kind: RunKind,
    switches: RunSwitches,
) -> StreamExport {
    let started = Instant::now();
    let is_baseline = run_kind == RunKind::Baseline;
    let stream = prepare_stream(name, switches, is_baseline);
    let drive_nanoseconds = started.elapsed().as_nanos();
    let directory = output_root.join(name.directory_name());
    let selection = StateSelection::RegisteredBatches {
        third_batch: is_baseline,
    };
    export_prepared_stream(
        &stream,
        &directory,
        is_baseline,
        switches,
        selection,
        started,
        drive_nanoseconds,
        None,
    )
}

/// 一条驱动完的流导出：枚举、取样、判、写导出文件、S4 重建核对。`check_against_the_crates` 只在基线那一趟开（S1 的状态数那几条）。
/// `piece_panic_injection` 交给判态的 [`ObservationContext`]：只有 Q5 主取样点、反向取样点那一次导出可能不空（登记修订 6 第 6.2 节第 3 条，
/// 跑法里也是空，只有内联用例 U15、U16 交），别的调用处一律交空。
#[allow(
    clippy::too_many_arguments,
    reason = "八个参数各有名字、各有类型，凑不成一个带不变量的概念（code-discipline.md「参数不按个数限」）"
)]
pub(super) fn export_prepared_stream(
    stream: &PreparedStream,
    directory: &Path,
    check_against_the_crates: bool,
    switches: RunSwitches,
    selection: StateSelection,
    started: Instant,
    drive_nanoseconds: u128,
    piece_panic_injection: Option<&PiecePanicInjection>,
) -> StreamExport {
    std::fs::create_dir_all(directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
    let name = stream.name;
    // 回比只在基线那一趟：坏实现那一趟本来就会让取号、txg、落点、写数与段表变，那些不是停机条款。
    let mut checks = if check_against_the_crates {
        stream.checks.clone()
    } else {
        Vec::new()
    };
    let mut export = StreamExport::not_exported(name, String::new(), started);
    export.not_exported = None;
    export.calls = stream.calls.clone();
    export.ending = Some(stream.ending.clone());
    export.model_stopped_at_call = stream.model_stopped_at_call;
    export.operation_count = stream.operations.len();
    export.write_count = stream.writes.len();
    export.segment_count = stream.segments.len();
    export.drive_nanoseconds = drive_nanoseconds;
    if stream.judged_root_index == usize::MAX {
        export.files =
            write_the_files_that_exist_even_when_the_stream_is_not_exported(directory, stream);
        export.checks = checks;
        export.not_exported = Some("no_root_write_to_judge".to_string());
        export.elapsed_nanoseconds = started.elapsed().as_nanos();
        return export;
    }
    let tables = crash_state_enumeration_tables(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let full_states = crash_point_replay_plan_state_count(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let in_place_subsets_states = crash_point_replay_plan_state_count(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &in_place_subsets_with_copy_on_write_none_or_all_expansion,
    );
    let facts = StateMasks::of(&tables, &stream.segments);
    if check_against_the_crates {
        checks.extend(state_count_checks_against_the_crates(
            name,
            stream,
            &tables,
            flow_facts_state_count(stream, &tables),
            full_states,
            in_place_subsets_states,
        ));
    }
    let torn_image_index_by_write: BTreeMap<usize, usize> = tables
        .torn_images
        .iter()
        .map(|row| (row.write_index, row.torn_image_index))
        .collect();
    let segment_has_a_torn_write: Vec<bool> = stream
        .segments
        .iter()
        .map(|segment| {
            segment
                .iter()
                .any(|write_index| tables.is_tearable_by_write[*write_index])
        })
        .collect();
    let recorded_write_count = stream.writes.len();
    let sampled = match &selection {
        StateSelection::RegisteredBatches { third_batch } => {
            let mut sampled = sample_ordinals(
                &tables.state_ranges_by_segment,
                tables.state_count,
                &segment_has_a_torn_write,
                &|ordinal| {
                    landing_vector_of(
                        &facts.persisted_writes_of_state(ordinal),
                        recorded_write_count,
                        &torn_image_index_by_write,
                    )
                },
            );
            if !*third_batch {
                sampled.batches[2].clear();
            }
            sampled
        }
        StateSelection::WholeLastSegmentAndTheFullyPersistedState => {
            let last_segment = tables
                .state_ranges_by_segment
                .last()
                .cloned()
                .unwrap_or(0..0);
            let mut whole: BTreeSet<u64> = last_segment.collect();
            whole.insert(tables.state_count - 1);
            SampledOrdinals {
                batches: [whole, BTreeSet::new(), BTreeSet::new()],
                torn_scans: Vec::new(),
            }
        }
        StateSelection::ListedStatesKeepingTheConsultedReports(listed) => SampledOrdinals {
            batches: [listed.clone(), BTreeSet::new(), BTreeSet::new()],
            torn_scans: Vec::new(),
        },
    };
    let consulted_reports = match &selection {
        StateSelection::RegisteredBatches { .. }
        | StateSelection::WholeLastSegmentAndTheFullyPersistedState => ConsultedReports::Dropped,
        StateSelection::ListedStatesKeepingTheConsultedReports(_) => ConsultedReports::Kept,
    };
    let reference_filesystem = reference_filesystem_stream_of(
        &stream.calls,
        &stream.operations,
        &stream.writes,
        &stream.stream_indexes,
    );
    export.reference_filesystem_segments =
        reference_filesystem.segment_closing_stream_indexes.len();
    export.reference_filesystem_stopped = reference_filesystem.stopped.as_ref().map_or_else(
        || "-".to_string(),
        |(position, stop)| {
            format!(
                "{}:{}",
                crash_moment_text(ReferenceFileSystemCrashMoment::InsideStep(*position)),
                field_text(stop.name())
            )
        },
    );
    let context = ObservationContext {
        stream,
        tables: &tables,
        facts: &facts,
        torn_image_index_by_write: &torn_image_index_by_write,
        reference_filesystem: &reference_filesystem,
        call_of_write: call_index_of_each_write(stream),
        piece_panic_injection,
    };
    let observe_started = Instant::now();
    let observations = observe_sampled_states(&context, &sampled, switches, consulted_reports);
    let observe_nanoseconds = observe_started.elapsed().as_nanos();
    let write_started = Instant::now();
    let files = write_stream_files(
        directory,
        stream,
        &tables,
        &observations,
        full_states,
        in_place_subsets_states,
    );
    // S4：只读刚写出的文件，按 states.tsv 的落地向量重建，摘要与 states.tsv 里（直接从 CrashImage 算的）那一列比。
    let neutral = read_neutral_stream(directory);
    let mut rebuilt_states = 0usize;
    let mut rebuilt_digest_mismatches = 0usize;
    for line in read_text_lines(directory, STATES_FILE) {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 6, "states.tsv 一行六列：{line:?}");
        rebuilt_states += 1;
        if sha256_text(&rebuilt_image_digest(&neutral, fields[4])) != fields[5] {
            rebuilt_digest_mismatches += 1;
        }
    }
    export.write_nanoseconds = write_started.elapsed().as_nanos();
    export.observe_nanoseconds = observe_nanoseconds;
    export.checks = checks;
    export.tearable_write_count = torn_image_index_by_write.len();
    export.full_states = full_states;
    export.in_place_subsets_states = in_place_subsets_states;
    export.torn_scans = sampled.torn_scans;
    export.observations = observations;
    export.files = files;
    export.rebuilt_digest_mismatches = rebuilt_digest_mismatches;
    export.rebuilt_states = rebuilt_states;
    export.elapsed_nanoseconds = started.elapsed().as_nanos();
    export
}

/// S1 里跟状态数有关的几条（只在基线那一趟）：全量与甲二状态数、每次写只取两态时的闭式、枚举表与 `crash_facts` 数的总数，
/// 对本地常量。与 `e170_specification_executor_export.rs` 的 `export_stream` 里那一批 S1 回比同一批。
pub(super) fn state_count_checks_against_the_crates(
    name: StreamName,
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
    flow_facts_state_count: Option<u64>,
    full_states: u64,
    in_place_subsets_states: u64,
) -> Vec<StopCheck> {
    let label = name.directory_name();
    let (local_full, local_in_place_subsets, local_closed_form) = match name {
        StreamName::NewPoolFileCreation => (
            LOCAL_FIRST_STREAM_FULL_STATES,
            LOCAL_FIRST_STREAM_IN_PLACE_SUBSETS_STATES,
            LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM,
        ),
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount => (
            LOCAL_SECOND_STREAM_FULL_STATES,
            LOCAL_SECOND_STREAM_IN_PLACE_SUBSETS_STATES,
            LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM,
        ),
        StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf => (
            LOCAL_THIRD_STREAM_FULL_STATES,
            LOCAL_THIRD_STREAM_IN_PLACE_SUBSETS_STATES,
            LOCAL_THIRD_STREAM_TWO_STATE_CLOSED_FORM,
        ),
    };
    vec![
        StopCheck::new(&format!("{label}_full_states"), local_full, full_states),
        StopCheck::new(
            &format!("{label}_in_place_subsets_states"),
            local_in_place_subsets,
            in_place_subsets_states,
        ),
        StopCheck::new(
            &format!("{label}_two_state_closed_form"),
            local_closed_form,
            singlefs_verification::memory_pool::closed_form_state_count_of_write_index_segments(&stream.segments),
        ),
        StopCheck::new(
            &format!("{label}_enumeration_tables_state_count"),
            full_states,
            tables.state_count,
        ),
        match flow_facts_state_count {
            Some(state_count) => StopCheck::new(
                &format!("{label}_flow_facts_state_count"),
                full_states,
                state_count,
            ),
            // 枚举写表多于 GPU 掩码宽：`extract_flow_facts` 抽不了（它断言 ≤ 2048），这一条换成「确实多于」的回比。
            None => StopCheck::new(
                &format!("{label}_flow_facts_not_extracted_because_the_enumerated_writes_exceed_the_mask_width"),
                true,
                tables.enumerated_writes.len() > MAXIMUM_ENUMERATED_WRITES,
            ),
        },
    ]
}

#[cfg(test)]
mod tests;
