//! 录入段交给判段的交接：字段的写法与读法、交接文件的写与读的单测。

use super::*;

/// 只跑录入段：每张要判的盘都录下、一张都不判，不打结果行，输出目录里只留交接。
#[test]
fn the_record_stage_of_the_known_bad_images_mode_judges_no_table_and_leaves_only_the_hand_off() {
    let dump = DumpDirectoryForKnownBadImages::new("record-stage-only");
    let tables = PoolCheckerTablesKeptInMemory::judging_with(
        &pool_checker_red_names_of_a_given_image_on_the_cpu,
    );
    let output_directory = dump.directory.join("output-of-the-record-stage");
    let recorded = dump.run_stage_on(
        RunStage::RecordOnly,
        SEVERAL_THREADS,
        &output_directory,
        &tables,
    );
    let files = DumpDirectoryForKnownBadImages::read_and_remove(&output_directory);
    assert_eq!(recorded.ending, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
    assert_eq!(tables.tables_judged(), 0, "录入段一张盘都不判");
    let e170_bad_rows = (PATCHED_E170_BAD_ROWS_OF_EACH_STREAM
        + UNPATCHED_E170_BAD_ROWS_OF_EACH_STREAM)
        * STREAM_NAMES.len();
    assert_eq!(
        tables.tables_recorded(),
        1 + 2 * dump.known_bad_entry_count + 2 * e170_bad_rows,
        "干净镜像一张、已知坏镜像每条两张、E170 坏状态每行两张都录下"
    );
    assert!(
        recorded.result_lines.is_empty(),
        "录入段不打结果行：{}",
        recorded.result_lines
    );
    assert_eq!(
        files.keys().collect::<Vec<&String>>(),
        vec![&HAND_OFF_TO_THE_JUDGE_STAGE_FILE.to_string()],
        "录入段在输出目录里只留交接"
    );
}

/// 分两趟跑（录入段一趟、判段一趟）与两段先后跑一趟：退出码、结果行、产物文件逐字节相同（判完删掉交接）；
/// 判段一张盘都不再录（不再重建、不再驱动），录下的每张盘判段判一次。
#[test]
fn the_known_bad_images_mode_in_two_runs_prints_and_writes_the_same_as_in_one_run() {
    let dump = DumpDirectoryForKnownBadImages::new("two-runs");
    let in_one_run = dump.run_on(
        SEVERAL_THREADS,
        &pool_checker_red_names_of_a_given_image_on_the_cpu,
    );
    let tables = PoolCheckerTablesKeptInMemory::judging_with(
        &pool_checker_red_names_of_a_given_image_on_the_cpu,
    );
    let output_directory = dump.directory.join("output-in-two-runs");
    let recorded = dump.run_stage_on(
        RunStage::RecordOnly,
        SEVERAL_THREADS,
        &output_directory,
        &tables,
    );
    assert_eq!(recorded.ending, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
    let tables_recorded_by_the_record_stage = tables.tables_recorded();
    let judged = dump.run_stage_on(
        RunStage::JudgeOnly,
        SEVERAL_THREADS,
        &output_directory,
        &tables,
    );
    let in_two_runs = KnownBadImagesModeOutcome {
        ending: judged.ending,
        result_lines: judged.result_lines,
        files: DumpDirectoryForKnownBadImages::read_and_remove(&output_directory),
    };
    assert_eq!(
        tables.tables_recorded(),
        tables_recorded_by_the_record_stage,
        "判段一张盘都不再录：不重跑流程"
    );
    assert_eq!(
        tables.tables_judged(),
        tables_recorded_by_the_record_stage,
        "录下的每张盘判段判一次"
    );
    assert_eq!(
        in_one_run.ending,
        Ok(0),
        "停机条款都过：{}",
        in_one_run.result_lines
    );
    assert_eq!(
        in_two_runs, in_one_run,
        "分两趟跑与两段先后跑一趟的退出码、结果行、产物文件逐字节相同"
    );
}

/// 只跑判段而录入段没跑过：拒，报错句点名先跑录入段，一张盘都不录、不判，不打结果行。
#[test]
fn the_judge_stage_without_a_record_stage_refuses_and_names_the_record_stage() {
    let dump = DumpDirectoryForKnownBadImages::new("judge-without-record");
    let tables = PoolCheckerTablesKeptInMemory::judging_with(
        &pool_checker_red_names_of_a_given_image_on_the_cpu,
    );
    let output_directory = dump.directory.join("output-never-recorded");
    let judged = dump.run_stage_on(
        RunStage::JudgeOnly,
        SEVERAL_THREADS,
        &output_directory,
        &tables,
    );
    let refusal = judged.ending.expect_err("录入段没跑过就只跑判段，要拒");
    assert!(
        refusal.contains("先跑录入段") && refusal.contains(&format!("{STAGE_ARGUMENT} record")),
        "报错句点名先跑录入段：{refusal}"
    );
    assert_eq!(tables.tables_recorded(), 0, "判段不补录");
    assert_eq!(tables.tables_judged(), 0, "没有交接就一张都不判");
    assert!(
        judged.result_lines.is_empty(),
        "拒了就不打结果行：{}",
        judged.result_lines
    );
}

/// 判段的参数与录入段那一趟的不同（两趟只许换 `--stage`）：拒，点名两趟的参数，不判。
#[test]
fn the_judge_stage_refuses_a_hand_off_recorded_under_other_arguments() {
    let output_directory = std::env::temp_dir().join(format!(
        "e171-hand-off-under-other-arguments-{}",
        std::process::id()
    ));
    let recorded = run_in_stages(
        RunStage::RecordOnly,
        &["row".to_string()],
        &output_directory,
        || "录入段交出的".to_string(),
        |_hand_off: String| -> Result<i32, String> { panic!("只跑录入段不判") },
    );
    let hand_offs_judged = AtomicUsize::new(0);
    let judged = run_in_stages(
        RunStage::JudgeOnly,
        &["baseline".to_string()],
        &output_directory,
        || -> String { panic!("只跑判段不录") },
        |_hand_off: String| {
            hand_offs_judged.fetch_add(1, Ordering::SeqCst);
            Ok(0)
        },
    );
    std::fs::remove_dir_all(&output_directory)
        .unwrap_or_else(|error| panic!("删 {}：{error}", output_directory.display()));
    assert_eq!(recorded, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
    let refusal = judged.expect_err("参数不同要拒");
    assert!(
        refusal.contains("[\"row\"]") && refusal.contains("[\"baseline\"]"),
        "点名两趟的参数：{refusal}"
    );
    assert_eq!(hand_offs_judged.load(Ordering::SeqCst), 0, "拒了就不判");
}

/// 新池新建文件流的录入段（取样照 [`StreamToSampleInThePairing`] 收小，与 `export_stream` 同一种填法）：导出文件写进 `output_root`
/// 下这条流的目录，O2 要判的盘录进 `tables`，交回 `baseline` 那一趟的交接。
fn recorded_dump_of_the_pairing_stream(
    stream_to_sample: &StreamToSampleInThePairing,
    output_root: &Path,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
) -> RecordedDump {
    let started = Instant::now();
    let name = StreamName::NewPoolFileCreation;
    let directory = output_root.join(name.directory_name());
    std::fs::create_dir_all(&directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
    let observations = stream_to_sample
        .observations_recorded_in(SEVERAL_THREADS, tables)
        .expect("这条流上实现不 panic");
    let stream = &stream_to_sample.stream;
    let mut export = StreamExport::not_exported(name, String::new(), started);
    export.not_exported = None;
    export.checks = stream.checks.clone();
    export.calls = stream.calls.clone();
    export.ending = Some(stream.ending.clone());
    export.model_stopped_at_call = stream.model_stopped_at_call;
    export.operation_count = stream.operations.len();
    export.write_count = stream.writes.len();
    export.segment_count = stream.segments.len();
    export.tearable_write_count = stream_to_sample.torn_image_index_by_write.len();
    export.full_states = stream_to_sample.tables.state_count;
    export.torn_scans = stream_to_sample.sampled.torn_scans.clone();
    export.files = write_stream_files_before_the_judge_stage(
        &directory,
        stream,
        &stream_to_sample.tables,
        &observations,
        stream_to_sample.tables.state_count,
        0,
    );
    export.observations = observations;
    RecordedDump {
        stream_independent_checks: stream_independent_checks(),
        exports: vec![export],
    }
}

/// 一个目录里每个文件（名字 → 去掉耗时之后的字节，[`contents_without_timing`]；不带耗时的原样）。
fn files_without_timing_under(directory: &Path) -> BTreeMap<String, Vec<u8>> {
    std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("读 {}：{error}", directory.display()))
        .map(|entry| {
            let path = entry.expect("读得了目录的一项").path();
            let name = path
                .file_name()
                .expect("目录里的文件有名字")
                .to_string_lossy()
                .into_owned();
            let contents = std::fs::read(&path)
                .unwrap_or_else(|error| panic!("读 {}：{error}", path.display()));
            let kept = contents_without_timing(&name, &contents).unwrap_or(contents);
            (name, kept)
        })
        .collect()
}

/// `baseline` 的交接：新池新建文件流（取样收小）录入段一趟、判段一趟，与两段先后跑一趟，退出码与结果行逐字节相同、判完删掉交接；
/// 两趟写出的文件（去掉耗时之后）都与不经交接、一趟写完（`write_stream_files`）的逐个相同。
#[test]
fn a_dump_handed_off_to_the_judge_stage_prints_the_same_as_in_one_run_and_writes_the_same_as_without_a_hand_off(
) {
    let stream_to_sample = StreamToSampleInThePairing::new_pool_file_creation();
    let tables = PoolCheckerTablesKeptInMemory::judging_with(
        &pool_checker_red_names_of_a_given_image_on_the_cpu,
    );
    let label = StreamName::NewPoolFileCreation.directory_name();
    let root = std::env::temp_dir().join(format!("e171-dump-hand-off-{}", std::process::id()));
    let run_arguments = ["baseline".to_string()];
    let command_line_on = |output_directory: PathBuf| CommandLine {
        run_kind: RunKind::Baseline,
        output_directory,
        switches: RunSwitches::default(),
    };
    let in_one_run_directory = root.join("in-one-run");
    let mut in_one_run_lines = Vec::new();
    let in_one_run = run_in_stages(
        RunStage::RecordThenJudge,
        &run_arguments,
        &in_one_run_directory,
        || recorded_dump_of_the_pairing_stream(&stream_to_sample, &in_one_run_directory, &tables),
        |recorded_dump| {
            judge_the_dump(
                &command_line_on(in_one_run_directory.clone()),
                recorded_dump,
                &tables,
                &mut in_one_run_lines,
            )
        },
    );
    let in_two_runs_directory = root.join("in-two-runs");
    let recorded = run_in_stages(
        RunStage::RecordOnly,
        &run_arguments,
        &in_two_runs_directory,
        || recorded_dump_of_the_pairing_stream(&stream_to_sample, &in_two_runs_directory, &tables),
        |_recorded: RecordedDump| -> Result<i32, String> { panic!("只跑录入段不判") },
    );
    let mut in_two_runs_lines = Vec::new();
    let in_two_runs = run_in_stages(
        RunStage::JudgeOnly,
        &run_arguments,
        &in_two_runs_directory,
        || -> RecordedDump { panic!("只跑判段不再取样") },
        |recorded_dump| {
            judge_the_dump(
                &command_line_on(in_two_runs_directory.clone()),
                recorded_dump,
                &tables,
                &mut in_two_runs_lines,
            )
        },
    );
    let hand_off_is_left = in_two_runs_directory
        .join(HAND_OFF_TO_THE_JUDGE_STAGE_FILE)
        .exists();
    let without_a_hand_off_directory = root.join("without-a-hand-off");
    std::fs::create_dir_all(&without_a_hand_off_directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", without_a_hand_off_directory.display()));
    let observations =
        stream_to_sample.observations_recorded_and_judged_in(NonZeroUsize::MIN, &tables);
    write_stream_files(
        &without_a_hand_off_directory,
        &stream_to_sample.stream,
        &stream_to_sample.tables,
        &observations,
        stream_to_sample.tables.state_count,
        0,
    );
    let files_in_one_run = files_without_timing_under(&in_one_run_directory.join(label));
    let files_in_two_runs = files_without_timing_under(&in_two_runs_directory.join(label));
    let files_without_a_hand_off = files_without_timing_under(&without_a_hand_off_directory);
    std::fs::remove_dir_all(&root).unwrap_or_else(|error| panic!("删 {}：{error}", root.display()));
    assert_eq!(recorded, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
    assert!(in_one_run.is_ok(), "两段先后跑那一趟没拒：{in_one_run:?}");
    assert_eq!(in_two_runs, in_one_run, "两趟的退出码相同");
    assert_eq!(
        String::from_utf8(in_two_runs_lines).expect("结果行是 UTF-8"),
        String::from_utf8(in_one_run_lines).expect("结果行是 UTF-8"),
        "分两趟跑与两段先后跑一趟的结果行逐字节相同"
    );
    assert!(!hand_off_is_left, "判段判完删掉交接");
    assert_eq!(
        files_in_two_runs, files_without_a_hand_off,
        "经交接分两趟写出的文件与不经交接一趟写完的逐个相同"
    );
    assert_eq!(
        files_in_one_run, files_without_a_hand_off,
        "两段先后跑写出的文件与不经交接一趟写完的逐个相同"
    );
}

/// 判段里引擎判一态的盘时失败的那句消息（不带空白：结果行里的 `field_text` 去掉空白）。
const ENGINE_FAILURE_MESSAGE: &str = "两段对拍里造的引擎失败";

/// 判段里引擎判新池新建文件流（取样照 [`StreamToSampleInThePairing`] 收小）某一态的盘时失败：分两趟跑（录入段一趟、判段一趟）、两段先后跑一趟，
/// 与两段之前的退出码、结果行逐字节相同，这条流的目录里都只留照写的两个文件（去掉耗时之后逐字节相同）。两段之前的那一份照不分两段的 `export_stream`
/// 在取样回调里接住实现自崩那一支的填法造：`not_exported` 是 `implementation_crashed_in_a_state:<消息>:after_ordinal_<按序号它前面那一态>`、
/// 只写照写的两个文件、取样之后才有的几样是零；打结果行的是两段之前与之后同一份 `report_stream`。
#[test]
fn an_engine_failure_in_a_state_prints_the_same_in_two_runs_in_one_run_and_as_before_the_two_stages(
) {
    let stream_to_sample = StreamToSampleInThePairing::new_pool_file_creation();
    let sampled_in_order: Vec<u64> = stream_to_sample
        .sampled
        .batches
        .iter()
        .flatten()
        .copied()
        .collect::<BTreeSet<u64>>()
        .into_iter()
        .collect();
    let position_of_the_failure = sampled_in_order.len() / 2;
    let table_name_part_of_the_failure =
        format!("/ordinal={}/", sampled_in_order[position_of_the_failure]);
    let judge_failing_at_that_state = |table_name: &str, pool: &MemoryPool| -> Vec<String> {
        if table_name.contains(&table_name_part_of_the_failure) {
            panic!("{ENGINE_FAILURE_MESSAGE}");
        }
        pool_checker_red_names_of_a_given_image_on_the_cpu(table_name, pool)
    };
    let tables = PoolCheckerTablesKeptInMemory::judging_with(&judge_failing_at_that_state);
    let name = StreamName::NewPoolFileCreation;
    let label = name.directory_name();
    let root = std::env::temp_dir().join(format!("e171-engine-failure-{}", std::process::id()));
    let run_arguments = ["baseline".to_string()];
    let command_line_on = |output_directory: PathBuf| CommandLine {
        run_kind: RunKind::Baseline,
        output_directory,
        switches: RunSwitches::default(),
    };
    let in_one_run_directory = root.join("in-one-run");
    let mut in_one_run_lines = Vec::new();
    let in_one_run = run_in_stages(
        RunStage::RecordThenJudge,
        &run_arguments,
        &in_one_run_directory,
        || recorded_dump_of_the_pairing_stream(&stream_to_sample, &in_one_run_directory, &tables),
        |recorded_dump| {
            judge_the_dump(
                &command_line_on(in_one_run_directory.clone()),
                recorded_dump,
                &tables,
                &mut in_one_run_lines,
            )
        },
    );
    let in_two_runs_directory = root.join("in-two-runs");
    let recorded = run_in_stages(
        RunStage::RecordOnly,
        &run_arguments,
        &in_two_runs_directory,
        || recorded_dump_of_the_pairing_stream(&stream_to_sample, &in_two_runs_directory, &tables),
        |_recorded: RecordedDump| -> Result<i32, String> { panic!("只跑录入段不判") },
    );
    let mut in_two_runs_lines = Vec::new();
    let in_two_runs = run_in_stages(
        RunStage::JudgeOnly,
        &run_arguments,
        &in_two_runs_directory,
        || -> RecordedDump { panic!("只跑判段不再取样") },
        |recorded_dump| {
            judge_the_dump(
                &command_line_on(in_two_runs_directory.clone()),
                recorded_dump,
                &tables,
                &mut in_two_runs_lines,
            )
        },
    );
    let before_directory = root.join("before-the-two-stages");
    let before_stream_directory = before_directory.join(label);
    std::fs::create_dir_all(&before_stream_directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", before_stream_directory.display()));
    let stream = &stream_to_sample.stream;
    let mut crashed_before_the_two_stages = StreamExport::not_exported(
        name,
        format!(
            "implementation_crashed_in_a_state:{ENGINE_FAILURE_MESSAGE}:after_ordinal_{}",
            sampled_in_order[position_of_the_failure - 1]
        ),
        Instant::now(),
    );
    crashed_before_the_two_stages.checks = stream.checks.clone();
    crashed_before_the_two_stages.calls = stream.calls.clone();
    crashed_before_the_two_stages.ending = Some(stream.ending.clone());
    crashed_before_the_two_stages.model_stopped_at_call = stream.model_stopped_at_call;
    crashed_before_the_two_stages.operation_count = stream.operations.len();
    crashed_before_the_two_stages.write_count = stream.writes.len();
    crashed_before_the_two_stages.segment_count = stream.segments.len();
    crashed_before_the_two_stages.files =
        write_the_files_that_exist_even_when_the_stream_is_not_exported(
            &before_stream_directory,
            stream,
        );
    let mut before_lines = Vec::new();
    let before = judge_the_dump(
        &command_line_on(before_directory.clone()),
        RecordedDump {
            stream_independent_checks: stream_independent_checks(),
            exports: vec![crashed_before_the_two_stages],
        },
        &PoolCheckerTablesKeptInMemory::judging_with(
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        ),
        &mut before_lines,
    );
    let hand_off_is_left = in_two_runs_directory
        .join(HAND_OFF_TO_THE_JUDGE_STAGE_FILE)
        .exists();
    let files_in_one_run = files_without_timing_under(&in_one_run_directory.join(label));
    let files_in_two_runs = files_without_timing_under(&in_two_runs_directory.join(label));
    let files_before = files_without_timing_under(&before_stream_directory);
    std::fs::remove_dir_all(&root).unwrap_or_else(|error| panic!("删 {}：{error}", root.display()));
    let before_lines = String::from_utf8(before_lines).expect("结果行是 UTF-8");
    assert_eq!(recorded, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
    assert_eq!(before, Ok(3), "基线那一趟有流没导出，停机：{before_lines}");
    assert_eq!(in_one_run, before, "两段先后跑与两段之前的退出码相同");
    assert_eq!(in_two_runs, before, "分两趟跑与两段之前的退出码相同");
    assert_eq!(
        String::from_utf8(in_one_run_lines).expect("结果行是 UTF-8"),
        before_lines,
        "两段先后跑与两段之前的结果行逐字节相同"
    );
    assert_eq!(
        String::from_utf8(in_two_runs_lines).expect("结果行是 UTF-8"),
        before_lines,
        "分两趟跑与两段之前的结果行逐字节相同"
    );
    assert!(!hand_off_is_left, "判段判完删掉交接");
    assert_eq!(
        files_before.keys().collect::<Vec<&String>>(),
        FILES_WRITTEN_EVEN_WHEN_THE_STREAM_IS_NOT_EXPORTED
            .map(str::to_string)
            .iter()
            .collect::<Vec<&String>>(),
        "两段之前这条流的目录里只有照写的两个文件"
    );
    assert_eq!(
        files_in_one_run, files_before,
        "两段先后跑这条流的目录与两段之前的逐个相同"
    );
    assert_eq!(
        files_in_two_runs, files_before,
        "分两趟跑这条流的目录与两段之前的逐个相同"
    );
}

/// 坏镜像模式里录入段构造某一行 E170 坏状态时 panic（`bad-states.tsv` 那一行少一列）：两段先后跑一趟、分两趟跑，与两段之前的 panic 消息、
/// 结果行、产物文件逐字节相同（分两趟跑的输出目录另留着交接）。两段之前（`e170_bad_rows_on_both_streams` 逐行构造、判，panic 的那一行原样抛出）
/// 结果行打到 panic 的前一行为止，已知坏镜像的三个产物照写，E170 那一份不写：期望照同一份导出没改坏时那一趟的结果行截到 panic 那一行之前、
/// 产物去掉 E170 那一份。
#[test]
fn a_panic_building_an_e170_bad_row_prints_the_same_in_two_runs_in_one_run_and_as_before_the_two_stages(
) {
    const ROW_OF_THE_PANIC: usize = 2;
    let dump = DumpDirectoryForKnownBadImages::new("e170-bad-row-building-panic");
    let unbroken = dump.run_on(
        SEVERAL_THREADS,
        &pool_checker_red_names_of_a_given_image_on_the_cpu,
    );
    assert_eq!(
        unbroken.ending,
        Ok(0),
        "没改坏的导出停机条款都过：{}",
        unbroken.result_lines
    );
    let label = StreamName::NewPoolFileCreation.directory_name();
    let stream_directory = dump.directory.join(label);
    let mut state_rows = read_text_lines(&stream_directory, "bad-states.tsv");
    let (without_the_patches, _patches) = state_rows[ROW_OF_THE_PANIC]
        .rsplit_once('\t')
        .expect("bad-states.tsv 一行五列");
    state_rows[ROW_OF_THE_PANIC] = without_the_patches.to_string();
    std::fs::write(
        stream_directory.join("bad-states.tsv"),
        state_rows
            .iter()
            .map(|row| format!("{row}\n"))
            .collect::<String>(),
    )
    .unwrap_or_else(|error| panic!("写 bad-states.tsv：{error}"));
    let first_line_not_printed_before = format!(
        "E7RESULT name=e170_bad_row stream={label} row={} ",
        e170_bad_row_identifier(ROW_OF_THE_PANIC)
    );
    let lines_before = unbroken.result_lines[..unbroken
        .result_lines
        .find(&first_line_not_printed_before)
        .expect("没改坏的那一趟打了 panic 那一行")]
        .to_string();
    let mut files_before = unbroken.files;
    assert!(
        files_before.remove("e170-bad-rows-o2.tsv").is_some(),
        "没改坏的那一趟写了 E170 那一份产物"
    );
    let in_one_run = dump.run_on(
        SEVERAL_THREADS,
        &pool_checker_red_names_of_a_given_image_on_the_cpu,
    );
    let tables = PoolCheckerTablesKeptInMemory::judging_with(
        &pool_checker_red_names_of_a_given_image_on_the_cpu,
    );
    let output_directory = dump.directory.join("output-in-two-runs");
    let recorded = dump.run_stage_on(
        RunStage::RecordOnly,
        SEVERAL_THREADS,
        &output_directory,
        &tables,
    );
    let judged = dump.run_stage_on(
        RunStage::JudgeOnly,
        SEVERAL_THREADS,
        &output_directory,
        &tables,
    );
    let mut files_in_two_runs = DumpDirectoryForKnownBadImages::read_and_remove(&output_directory);
    let hand_off_is_left = files_in_two_runs
        .remove(HAND_OFF_TO_THE_JUDGE_STAGE_FILE)
        .is_some();
    assert!(
        in_one_run
            .ending
            .as_ref()
            .is_err_and(|text| text.contains("bad-states.tsv 一行五列")),
        "构造那一行时的 panic 原样抛出来：{:?}",
        in_one_run.ending
    );
    assert_eq!(
        in_one_run.result_lines, lines_before,
        "两段先后跑与两段之前的结果行逐字节相同（打到 panic 的前一行为止）"
    );
    assert_eq!(
        in_one_run.files, files_before,
        "两段先后跑与两段之前的产物逐字节相同（E170 那一份不写）"
    );
    assert_eq!(recorded.ending, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
    assert_eq!(
        judged.ending, in_one_run.ending,
        "分两趟跑与两段先后跑 panic 的消息相同"
    );
    assert_eq!(
        judged.result_lines, lines_before,
        "分两趟跑与两段之前的结果行逐字节相同"
    );
    assert_eq!(
        files_in_two_runs, files_before,
        "分两趟跑与两段之前的产物逐字节相同"
    );
    assert!(hand_off_is_left, "判段 panic 了，交接留着");
}

/// 判段读到的交接里录的盘在判段这一边读不回（判段换了一份没录过它们的库）：拒（退非 0），报错句点名那张表与出路，
/// 一行结果行都不打、一个产物都不写，交接留着。
#[test]
fn the_judge_stage_refuses_a_hand_off_whose_tables_cannot_be_read_back() {
    let dump = DumpDirectoryForKnownBadImages::new("tables-not-read-back");
    let output_directory = dump.directory.join("output-judged-on-another-library");
    let recorded = dump.run_stage_on(
        RunStage::RecordOnly,
        SEVERAL_THREADS,
        &output_directory,
        &PoolCheckerTablesKeptInMemory::judging_with(
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        ),
    );
    let judged = dump.run_stage_on(
        RunStage::JudgeOnly,
        SEVERAL_THREADS,
        &output_directory,
        &PoolCheckerTablesKeptInMemory::judging_with(
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        ),
    );
    let files = DumpDirectoryForKnownBadImages::read_and_remove(&output_directory);
    assert_eq!(recorded.ending, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
    let refusal = judged.ending.expect_err("交接里录的盘读不回，要拒");
    assert!(
        refusal.contains("known-bad/clean") && refusal.contains("先跑录入段"),
        "报错句点名录入次序上第一张读不回的表与出路：{refusal}"
    );
    assert!(
        judged.result_lines.is_empty(),
        "拒了就不打结果行：{}",
        judged.result_lines
    );
    assert_eq!(
        files.keys().collect::<Vec<&String>>(),
        vec![&HAND_OFF_TO_THE_JUDGE_STAGE_FILE.to_string()],
        "一个产物都不写，交接留着"
    );
}
