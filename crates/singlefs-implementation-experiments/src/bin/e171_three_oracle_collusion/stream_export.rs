//! 一条流的导出与它的状态数回比 crates。

use super::*;

/// 一条流导出之后的样子。`not_exported` 有值：这条流没有导出状态（驱动里实现 panic、没有根槽写可判、某个状态上实现 panic、
/// 或流水线里别处 panic），只有 `call-outcomes.tsv` 与 `stream-status.tsv`（能写出来的话）。
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
            elapsed_nanoseconds: started.elapsed().as_nanos(),
            drive_nanoseconds: 0,
            observe_nanoseconds: 0,
            write_nanoseconds: 0,
        }
    }

    /// 判段里引擎判这条流某一态的盘时失败（`what` 见 [`JudgedTablesOfAStream::EngineFailedInAState`]）：照两段之前取样回调里接住它时那样，
    /// 这条流记成某个状态上实现自崩（`implementation_crashed_in_a_state:<what>`），盖过录入段记的那一句；`directory`（这条流的导出目录）里只留
    /// 照写的两个文件（[`FILES_WRITTEN_EVEN_WHEN_THE_STREAM_IS_NOT_EXPORTED`]，录入段写的别的导出文件删掉），取样之后才有的几样清零。
    ///
    /// # Panics
    /// 录入段写的导出文件删不掉。
    pub(super) fn record_an_engine_failure_in_a_state(&mut self, directory: &Path, what: &str) {
        for file in &self.files {
            if !FILES_WRITTEN_EVEN_WHEN_THE_STREAM_IS_NOT_EXPORTED.contains(&file.name.as_str()) {
                let path = directory.join(&file.name);
                std::fs::remove_file(&path)
                    .unwrap_or_else(|error| panic!("删 {}：{error}", path.display()));
            }
        }
        self.files.retain(|file| {
            FILES_WRITTEN_EVEN_WHEN_THE_STREAM_IS_NOT_EXPORTED.contains(&file.name.as_str())
        });
        self.not_exported = Some(format!("implementation_crashed_in_a_state:{what}"));
        self.tearable_write_count = 0;
        self.full_states = 0;
        self.in_place_subsets_states = 0;
        self.torn_scans = Vec::new();
        self.observations = Vec::new();
        self.rebuilt_digest_mismatches = 0;
        self.rebuilt_states = 0;
        self.observe_nanoseconds = 0;
        self.write_nanoseconds = 0;
    }
}

/// 基线与坏实现那一趟的差别：基线回比本地常量、取三批；坏实现那一趟不回比（写流与段表本来就会变）、只取批 1 与批 2。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RunKind {
    Baseline,
    BadImplementationRow,
}

/// 录入段导出一条流：驱动、取样、写录入段写得了的导出文件（[`write_stream_files_before_the_judge_stage`]）、S4 回读；O2 要判的盘录进 `pool_checker_tables`，不判。
/// 某个状态上实现自崩的流只写两个照写的文件，崩之前判完的那几态留在 `observations` 里交判段（[`judge_the_dump`] 判完清掉）。
pub(super) fn export_stream(
    name: StreamName,
    output_root: &Path,
    run_kind: RunKind,
    switches: RunSwitches,
    pool_checker_tables: &dyn PoolCheckerTablesOfTheTwoStages,
) -> StreamExport {
    let started = Instant::now();
    let is_baseline = run_kind == RunKind::Baseline;
    let stream = prepare_stream(name, switches, is_baseline);
    let drive_nanoseconds = started.elapsed().as_nanos();
    let label = name.directory_name();
    let directory = output_root.join(label);
    std::fs::create_dir_all(&directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
    // 回比只在基线那一趟：坏实现那一趟本来就会让取号、txg、落点、写数与段表变，那些不是停机条款。
    let mut checks = if is_baseline {
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
    let unexported_reason = match &stream.ending {
        StreamEnding::ImplementationCrashed { what } => Some(format!(
            "implementation_crashed_in_a_call:{}",
            what.replace(' ', "_")
        )),
        StreamEnding::Completed | StreamEnding::StoppedByAnError { .. }
            if stream.judged_root_index == usize::MAX =>
        {
            Some("no_root_write_to_judge".to_string())
        }
        StreamEnding::Completed | StreamEnding::StoppedByAnError { .. } => None,
    };
    if let Some(reason) = unexported_reason {
        export.files =
            write_the_files_that_exist_even_when_the_stream_is_not_exported(&directory, &stream);
        export.checks = checks;
        export.not_exported = Some(reason);
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
    let facts = extract_flow_facts(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    if is_baseline {
        checks.extend(state_count_checks_against_the_crates(
            name,
            &stream,
            &tables,
            &facts,
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
    if !is_baseline {
        sampled.batches[2].clear();
    }
    let observe_started = Instant::now();
    let observations = match observe_sampled_states(
        &stream,
        &tables,
        &facts,
        &torn_image_index_by_write,
        &sampled,
        switches,
        sampling_threads_of_this_run(switches),
        &|stream_name, ordinal, image_digest, image, pool_checker_control| {
            pool_checker_table_of_a_sampled_state(
                pool_checker_tables,
                stream_name,
                ordinal,
                image_digest,
                image,
                pool_checker_control,
            )
        },
    ) {
        Ok(observations) => observations,
        Err(ObservationsUpToACrashInAState {
            crash,
            observations,
        }) => {
            export.files = write_the_files_that_exist_even_when_the_stream_is_not_exported(
                &directory, &stream,
            );
            export.checks = checks;
            export.not_exported = Some(format!("implementation_crashed_in_a_state:{crash}"));
            // 崩之前判完的那几态录下的盘留给判段判（引擎在其中一态上失败时那一句盖过这一句），判完清掉
            export.observations = observations;
            export.elapsed_nanoseconds = started.elapsed().as_nanos();
            return export;
        }
    };
    let observe_nanoseconds = observe_started.elapsed().as_nanos();
    let write_started = Instant::now();
    let files = write_stream_files_before_the_judge_stage(
        &directory,
        &stream,
        &tables,
        &observations,
        full_states,
        in_place_subsets_states,
    );
    // S4：只读刚写出的文件，按 states.tsv 的落地向量重建，摘要与 states.tsv 里（直接从 CrashImage 算的）那一列比。
    let neutral = read_neutral_stream(&directory);
    let mut rebuilt_states = 0usize;
    let mut rebuilt_digest_mismatches = 0usize;
    for line in read_text_lines(&directory, STATES_FILE) {
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
fn state_count_checks_against_the_crates(
    name: StreamName,
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
    facts: &FlowFacts,
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
            singlefs_verification::memory_pool::closed_form_state_count_of_write_index_segments(
                &stream.segments,
            ),
        ),
        StopCheck::new(
            &format!("{label}_enumeration_tables_state_count"),
            full_states,
            tables.state_count,
        ),
        StopCheck::new(
            &format!("{label}_flow_facts_state_count"),
            full_states,
            facts.state_count,
        ),
    ]
}
