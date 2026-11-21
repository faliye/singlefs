//! 确认丢失那一形（confirmed loss form）：形状、操作、备好的流、回比 crates、跑与判各形状、合计的报告的单测。

use super::*;
// 经产品取样路径（`sample_crash_states_on_the_cpu`、给定状态引擎）开全仓那一份的用例开头拿它：同一个二进制里
// `pool_checker_holds_and_violations_through_the_given_states_engine` 设一次性库，不拿的在它设着时开到它那一份、它收尾停服之后连不上
use singlefs_verification::crash_library::TheOneLibraryKeptForThisProcess;

/// Q5 那一形的写流（读法 7；V12：根槽 FUA 不改成普通写，这里红）：删掉的屏障不在、根槽那次写改成普通写、截在截断处、写的字节一个不改。
#[test]
fn the_confirmed_loss_form_drops_the_barriers_makes_the_root_write_plain_and_truncates() {
    let operations: Vec<RetainedOperation> = (0..10usize)
        .map(|index| RetainedOperation {
            operation: singlefs_verification::RecordedOperation {
                device: DeviceIdentity(0),
                kind: match index {
                    3 | 5 | 8 => RecordedOperationKind::Barrier,
                    6 => RecordedOperationKind::WriteForceUnitAccess,
                    _ => RecordedOperationKind::Write,
                },
                offset: DeviceOffsetInBytes(u64::try_from(index).expect("下标") * 4096),
                length: 4096,
                content_hash: u64::try_from(index).expect("下标"),
            },
            contents: Some(vec![u8::try_from(index).expect("下标"); 4]),
        })
        .collect();
    let shape = ConfirmedLossShape {
        removed_barriers: vec![3, 5],
        root_writes_made_plain: vec![6],
        truncation: 8,
        reverse_truncation: 7,
    };
    let (kept, new_index_of) = confirmed_loss_operations(&operations, &shape, false);
    assert_eq!(kept.len(), 6);
    assert_eq!(
        new_index_of,
        vec![
            Some(0),
            Some(1),
            Some(2),
            None,
            Some(3),
            None,
            Some(4),
            Some(5)
        ]
    );
    assert_eq!(kept[4].operation.kind, RecordedOperationKind::Write);
    assert_eq!(kept[4].contents, operations[6].contents);
    assert!(kept
        .iter()
        .all(|retained| retained.operation.kind != RecordedOperationKind::Barrier));
    let (reverse, _) = confirmed_loss_operations(&operations, &shape, true);
    assert_eq!(reverse.len(), 5);
}

/// U8 用的两形：主取样点那一形与对拍那一形（照 `run_confirmed_loss_form` 造）。
fn confirmed_loss_shapes_for_the_test() -> ConfirmedLossShapes {
    let (main, shape) = confirmed_loss_prepared_stream(
        second_stream_through_the_first_overwrite(RunSwitches::default()),
        ConfirmedLossTruncation::BeforeTheInFlightBarrier,
    );
    ConfirmedLossShapes {
        sampling_point: "main_before_the_first_barrier_of_call_3",
        main,
        shape,
        drive_nanoseconds: 0,
        pairing: Some(equivalence_pairing_stream_keeping(
            EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
        )),
        piece_panic_injections: ConfirmedLossPiecePanicInjections {
            pairing: None,
            piece_panic_control: None,
            sampling_point_export: None,
        },
    }
}

/// U8（修订 5 第 5.7 节，第 5.3 节；V30：根写的前置判删掉、退回先对拍，这里红）：主取样点那一形、对拍那一形各模拟一次没有根写
/// （只把 `judged_root_index` 设成 `usize::MAX`，写流不变），都在对拍之前退 3、记 `q5_not_run=no_root_write`。
#[test]
fn shape_without_a_root_write_stops_before_the_pairing() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    // 守卫认领输出目录：走岔了真写进去时，用例做完、断言失败、panic 都删掉它
    let output_directory =
        singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
            std::env::temp_dir().join("e171r1-unit-no-root-write-not-exported"),
        );
    let command_line = ConfirmedLossCommandLine {
        output_directory: output_directory.to_path_buf(),
        truncate_at_the_return_point: false,
    };
    enum ShapeWithoutARootWrite {
        Main,
        Pairing,
    }
    for shape_without_a_root in [
        ShapeWithoutARootWrite::Main,
        ShapeWithoutARootWrite::Pairing,
    ] {
        let mut shapes = confirmed_loss_shapes_for_the_test();
        let shape_name = match shape_without_a_root {
            ShapeWithoutARootWrite::Main => {
                shapes.main.judged_root_index = usize::MAX;
                "main"
            }
            ShapeWithoutARootWrite::Pairing => {
                shapes
                    .pairing
                    .as_mut()
                    .expect("主取样点那一跑有对拍那一形")
                    .judged_root_index = usize::MAX;
                "pairing"
            }
        };
        let mut lines = ResultLines {
            emitted: 0,
            collected: Some(Vec::new()),
        };
        let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
        let collected = lines.collected.expect("收集模式");
        assert_eq!(exit_code, 3, "{shape_name}：退 3");
        assert_eq!(
            collected
                .iter()
                .filter(|line| line.starts_with("name=q5 ")
                    && line.contains(" not_run=no_root_write")
                    && line.contains(&format!(" shape={shape_name}")))
                .count(),
            1,
            "{shape_name}：一行 name=q5 … not_run=no_root_write shape={shape_name}：{collected:?}"
        );
        assert!(
            collected
                .iter()
                .any(|line| line.starts_with("name=verdict ")
                    && line.contains(" s13=false")
                    && line.contains(" q5_not_run=no_root_write")),
            "{shape_name}：判决行带 s13=false 与 q5_not_run=no_root_write：{collected:?}"
        );
        assert!(
            !collected.iter().any(|line| {
                line.starts_with("name=q5_equivalence_pairing")
                    || line.starts_with("name=q5_piece_panic_control")
                    || line.starts_with("name=q5_equivalence")
            }),
            "{shape_name}：对拍、阳性对照与类核对一行都没打：{collected:?}"
        );
    }
}

/// U13–U16 用的注入：一条不点名件的规则（`piece: None`），条件是写表下标 `write_index` 那个写落了（「落」= 落地串那一位不是 `0`）。
fn an_unattributed_panic_where_the_write_landed(write_index: usize) -> PiecePanicInjection {
    PiecePanicInjection {
        rules: vec![PiecePanicInjectionRule {
            piece: None,
            conditions: vec![InjectedLandingCondition::Landed(write_index)],
        }],
    }
}

/// U15、U16 用的注入：一条不带条件、不点名件的规则（选中全部判的态）。
fn an_unattributed_panic_in_every_state() -> PiecePanicInjection {
    PiecePanicInjection {
        rules: vec![PiecePanicInjectionRule {
            piece: None,
            conditions: Vec::new(),
        }],
    }
}

/// 对拍流（k = 2）的 c1（末段里第 3 次调用发出的第一个写）与末段里 c1 落了的态数：用例自己按末段的混合进制逐态数，不读装置的计数。
fn first_in_flight_write_and_the_states_where_it_landed(stream: &PreparedStream) -> (usize, usize) {
    let tables = crash_state_enumeration_tables(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let first_in_flight_write = *equivalence_pairing_shape_facts(stream, &tables)
        .in_flight_writes
        .first()
        .expect("对拍流末段里有第 3 次调用发出的写");
    let digits = LastSegmentDigits::of(stream, &tables);
    let position = digits
        .write_indexes
        .iter()
        .position(|write_index| *write_index == first_in_flight_write)
        .expect("c1 在末段里");
    let states_where_it_landed = (digits.first_ordinal..tables.state_count)
        .filter(|ordinal| digits.digit_of(*ordinal, position) != 0)
        .count();
    (first_in_flight_write, states_where_it_landed)
}

/// U13–U16 用的两形：主那一形取 `truncation` 截出的流二前四次调用，对拍那一形照 `pairing` 交，注入照 `piece_panic_injections` 交。
fn confirmed_loss_shapes_with_injections(
    sampling_point: &'static str,
    truncation: ConfirmedLossTruncation,
    pairing: Option<PreparedStream>,
    piece_panic_injections: ConfirmedLossPiecePanicInjections,
) -> ConfirmedLossShapes {
    let (main, shape) = confirmed_loss_prepared_stream(
        second_stream_through_the_first_overwrite(RunSwitches::default()),
        truncation,
    );
    ConfirmedLossShapes {
        sampling_point,
        main,
        shape,
        drive_nanoseconds: 0,
        pairing,
        piece_panic_injections,
    }
}

/// U13–U16 的共同断言：退 3；倒数第二行（末行是 `name=done`）是判决行，带 `s13=false`、`q5_not_run=unattributed_panic` 与
/// `unattributed_panic_states=<用例现算的数>`，判决行只有这一行。
fn assert_the_confirmed_loss_form_stopped_as_unattributed(
    exit_code: i32,
    collected: &[String],
    expected_unattributed_panic_states: usize,
) {
    assert_eq!(exit_code, 3, "退 3：{collected:?}");
    assert!(
        collected
            .last()
            .is_some_and(|line| line.starts_with("name=done ")),
        "末行是 name=done：{collected:?}"
    );
    let verdict_lines: Vec<&String> = collected
        .iter()
        .filter(|line| line.starts_with("name=verdict "))
        .collect();
    assert_eq!(verdict_lines.len(), 1, "判决行恰一行：{collected:?}");
    assert_eq!(
        collected
            .iter()
            .rposition(|line| line.starts_with("name=verdict ")),
        Some(collected.len() - 2),
        "判决行就在 name=done 之前：{collected:?}"
    );
    let verdict = verdict_lines[0];
    assert!(
        verdict.contains(" s13=false ")
            && verdict.contains(" q5_not_run=unattributed_panic ")
            && verdict.ends_with(&format!(
                " unattributed_panic_states={expected_unattributed_panic_states}"
            )),
        "判决行带 s13=false、q5_not_run=unattributed_panic 与 unattributed_panic_states={expected_unattributed_panic_states}：{verdict}"
    );
}

/// U13（登记修订 6 第 6.5 节；V33：对拍之后收 unattributed 那一处整块删掉，往下走到阳性对照、k = 2 上内置规则造不出、判决行换成
/// `piece_panic_control_failed`，这里红）：主那一形与对拍都是 k = 2 的流，对拍那一遍注入一条不点名件的规则（条件「c1 落」），
/// 在对拍那一处就停：退 3、判决行 `q5_not_run=unattributed_panic`、数等于 c1 落了的态数；阳性对照一行都没打。
#[test]
fn an_unattributed_panic_in_the_pairing_stops_the_confirmed_loss_form_at_the_pairing() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let pairing = equivalence_pairing_stream();
    let (first_in_flight_write, states_where_it_landed) =
        first_in_flight_write_and_the_states_where_it_landed(&pairing);
    let shapes = confirmed_loss_shapes_with_injections(
        "main_before_the_first_barrier_of_call_3",
        ConfirmedLossTruncation::AfterTheFirstInFlightWrites(IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS),
        Some(pairing),
        ConfirmedLossPiecePanicInjections {
            pairing: Some(an_unattributed_panic_where_the_write_landed(
                first_in_flight_write,
            )),
            piece_panic_control: None,
            sampling_point_export: None,
        },
    );
    // 守卫认领输出目录：用例做完、断言失败、panic 都删掉它
    let output_directory =
        singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
            std::env::temp_dir().join(format!(
                "e171r1-unit-unattributed-in-the-pairing-{}",
                std::process::id()
            )),
        );
    let command_line = ConfirmedLossCommandLine {
        output_directory: output_directory.to_path_buf(),
        truncate_at_the_return_point: false,
    };
    let mut lines = ResultLines {
        emitted: 0,
        collected: Some(Vec::new()),
    };
    let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
    let collected = lines.collected.expect("收集模式");
    assert_the_confirmed_loss_form_stopped_as_unattributed(
        exit_code,
        &collected,
        states_where_it_landed,
    );
    assert!(
        !collected
            .iter()
            .any(|line| line.starts_with("name=q5_piece_panic_control")),
        "停在对拍那一处，阳性对照一行都没打：{collected:?}"
    );
}

/// U14（登记修订 6 第 6.5 节；V34：阳性对照之后收 unattributed 那一处整块删掉，走到 F9 那一支、判决行换成
/// `piece_panic_control_failed`，这里红）：同 U13 的两形，对拍不注入；阳性对照那一格交同一条规则（替换内置规则），
/// 在阳性对照那一处就停：退 3、数等于 c1 落了的态数；阳性对照那一行打了，主取样点那一形一行都没打。
#[test]
fn an_unattributed_panic_in_the_piece_panic_control_stops_the_confirmed_loss_form_at_the_control() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let pairing = equivalence_pairing_stream();
    let (first_in_flight_write, states_where_it_landed) =
        first_in_flight_write_and_the_states_where_it_landed(&pairing);
    let shapes = confirmed_loss_shapes_with_injections(
        "main_before_the_first_barrier_of_call_3",
        ConfirmedLossTruncation::AfterTheFirstInFlightWrites(IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS),
        Some(pairing),
        ConfirmedLossPiecePanicInjections {
            pairing: None,
            piece_panic_control: Some(an_unattributed_panic_where_the_write_landed(
                first_in_flight_write,
            )),
            sampling_point_export: None,
        },
    );
    // 守卫认领输出目录：用例做完、断言失败、panic 都删掉它
    let output_directory =
        singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
            std::env::temp_dir().join(format!(
                "e171r1-unit-unattributed-in-the-control-{}",
                std::process::id()
            )),
        );
    let command_line = ConfirmedLossCommandLine {
        output_directory: output_directory.to_path_buf(),
        truncate_at_the_return_point: false,
    };
    let mut lines = ResultLines {
        emitted: 0,
        collected: Some(Vec::new()),
    };
    let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
    let collected = lines.collected.expect("收集模式");
    assert_the_confirmed_loss_form_stopped_as_unattributed(
        exit_code,
        &collected,
        states_where_it_landed,
    );
    assert_eq!(
        collected
            .iter()
            .filter(|line| line.starts_with("name=q5_piece_panic_control "))
            .count(),
        1,
        "阳性对照那一行打了：{collected:?}"
    );
    assert!(
        !collected
            .iter()
            .any(|line| line.starts_with("name=q5_shape")),
        "停在阳性对照那一处，主取样点那一形一行都没打：{collected:?}"
    );
}

/// U15（登记修订 6 第 6.5 节；V35：反向取样点导出之后收 unattributed 那一处整块删掉，往下打出 `report_the_confirmed_loss` 那几行
/// 与不带 `q5_not_run` 的判决行，这里红）：反向取样点那一形（对拍为空），导出那一格交一条不带条件的不点名件规则，导出之后就停：
/// 退 3、数等于末段各写取法数之积罩的态加全持久态（用例现算）；`report_the_confirmed_loss` 的计数行一行都没打。
#[test]
fn an_unattributed_panic_at_the_reverse_sampling_point_stops_before_the_tally() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let shapes = confirmed_loss_shapes_with_injections(
        "reverse_at_the_return_point_of_call_2",
        ConfirmedLossTruncation::AtTheConfirmedReturnPoint,
        None,
        ConfirmedLossPiecePanicInjections {
            pairing: None,
            piece_panic_control: None,
            sampling_point_export: Some(an_unattributed_panic_in_every_state()),
        },
    );
    let judged_states = {
        let tables = crash_state_enumeration_tables(
            &shapes.main.base,
            &shapes.main.writes,
            &shapes.main.segments,
            &full_expansion,
        );
        let last_segment = shapes.main.segments.last().cloned().unwrap_or_default();
        let product_of_the_choices: u64 = last_segment
            .iter()
            .map(|write_index| {
                if tables.is_tearable_by_write[*write_index] {
                    3u64
                } else {
                    2u64
                }
            })
            .product();
        let first_ordinal_of_the_last_segment = tables
            .state_ranges_by_segment
            .last()
            .map_or(0, |state_range| state_range.start);
        let mut states: BTreeSet<u64> = (first_ordinal_of_the_last_segment
            ..first_ordinal_of_the_last_segment + product_of_the_choices)
            .collect();
        states.insert(tables.state_count - 1);
        states.len()
    };
    // 守卫认领输出目录：用例做完、断言失败、panic 都删掉它
    let output_directory =
        singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
            std::env::temp_dir().join(format!(
                "e171r1-unit-unattributed-at-the-reverse-sampling-point-{}",
                std::process::id()
            )),
        );
    let command_line = ConfirmedLossCommandLine {
        output_directory: output_directory.to_path_buf(),
        truncate_at_the_return_point: true,
    };
    let mut lines = ResultLines {
        emitted: 0,
        collected: Some(Vec::new()),
    };
    let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
    let collected = lines.collected.expect("收集模式");
    assert_the_confirmed_loss_form_stopped_as_unattributed(exit_code, &collected, judged_states);
    assert!(
        !collected
            .iter()
            .any(|line| line.starts_with("name=q5 sampling_point=")),
        "停在计数之前，report_the_confirmed_loss 的计数行一行都没打：{collected:?}"
    );
}

/// U16（登记修订 6 第 6.5 节；V36：主取样点导出之后收 unattributed 那一处整块删掉，往下走到类核对与判决，这里红）：主那一形取
/// k = 2 的流、对拍为空（只为走到主取样点导出之后那一处，跑法里不会这么交），导出那一格交同 U15 的规则，导出之后就停：
/// 退 3、数等于分法交的判态数（用例从 `classify_the_last_segment` 的 `judged_ordinals` 现数）；类核对那一行没打。
#[test]
fn an_unattributed_panic_at_the_main_sampling_point_stops_before_the_classes_are_judged() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let shapes = confirmed_loss_shapes_with_injections(
        "main_before_the_first_barrier_of_call_3",
        ConfirmedLossTruncation::AfterTheFirstInFlightWrites(IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS),
        None,
        ConfirmedLossPiecePanicInjections {
            pairing: None,
            piece_panic_control: None,
            sampling_point_export: Some(an_unattributed_panic_in_every_state()),
        },
    );
    let judged_states = {
        let observation_tables = ObservationTables::of(&shapes.main);
        let context = observation_tables.context(&shapes.main);
        let digits = LastSegmentDigits::of(&shapes.main, &observation_tables.tables);
        let classification = classify_the_last_segment(
            &context,
            &digits,
            CONFIRMED_LOSS_IN_FLIGHT_CALL,
            CONFIRMED_LOSS_STATE_LIMIT,
        );
        assert!(
            !classification.stopped_at_the_state_limit,
            "k = 2 的流分类没撞态数上限"
        );
        classification.judged_ordinals(&digits).len()
    };
    // 守卫认领输出目录：用例做完、断言失败、panic 都删掉它
    let output_directory =
        singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
            std::env::temp_dir().join(format!(
                "e171r1-unit-unattributed-at-the-main-sampling-point-{}",
                std::process::id()
            )),
        );
    let command_line = ConfirmedLossCommandLine {
        output_directory: output_directory.to_path_buf(),
        truncate_at_the_return_point: false,
    };
    let mut lines = ResultLines {
        emitted: 0,
        collected: Some(Vec::new()),
    };
    let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
    let collected = lines.collected.expect("收集模式");
    assert_the_confirmed_loss_form_stopped_as_unattributed(exit_code, &collected, judged_states);
    assert!(
        !collected
            .iter()
            .any(|line| line.starts_with("name=q5_equivalence sampling_point=")),
        "停在类核对之前，name=q5_equivalence sampling_point= 一行都没打：{collected:?}"
    );
}
