//! 判的几块（judging piece）：各块的结局、块内 panic 的记账与注入的单测。

use super::*;
// 经产品取样路径（`sample_crash_states_on_the_cpu`、给定状态引擎）开全仓那一份的用例开头拿它：同一个二进制里
// `pool_checker_holds_and_violations_through_the_given_states_engine` 设一次性库，不拿的在它设着时开到它那一份、它收尾停服之后连不上
use singlefs_verification::crash_library::TheOneLibraryKeptForThisProcess;

/// 登记「主 agent 认定：27-2 装置交回」第 2 条（变异「unattributed_panic_states 恒 0」在这里红）：对拍流（k = 2）末段第一态上注入一条
/// 不点名件的规则，只让正常那一路的回调 panic；逐件重做十件一件都不 panic。`unattributed_panic_states=` 数到 1、
/// `name=q5_piece_panics` 行带它，Q5 走 `q5_not_run=unattributed_panic` 那个出口：判决行 `s13=false`、退 3。
#[test]
fn judging_path_panic_that_no_piece_repeats_stops_the_confirmed_loss_form_as_unattributed() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let stream = equivalence_pairing_stream();
    let observation_tables = ObservationTables::of(&stream);
    let injection = PiecePanicInjection {
        rules: vec![PiecePanicInjectionRule {
            piece: None,
            conditions: Vec::new(),
        }],
    };
    let mut context = observation_tables.context(&stream);
    context.piece_panic_injection = Some(&injection);
    let digits = LastSegmentDigits::of(&stream, &observation_tables.tables);
    let observations = observe_sampled_states(
        &context,
        &SampledOrdinals {
            batches: [
                BTreeSet::from([digits.first_ordinal]),
                BTreeSet::new(),
                BTreeSet::new(),
            ],
            torn_scans: Vec::new(),
        },
        RunSwitches::default(),
        ConsultedReports::Dropped,
    );
    assert_eq!(observations.len(), 1, "只判末段第一态");
    assert!(
        observations[0].judging_path_panic.is_some(),
        "正常那一路照注入 panic 了"
    );
    assert_eq!(
        piece_panic_states_of(&observations),
        [0; JUDGING_PIECE_COUNT],
        "逐件重做十件一件都没 panic"
    );
    assert_eq!(unattributed_panic_states_of(&observations), 1);
    let piece_panics_line = confirmed_loss_form_piece_panics_line("pairing", &observations);
    assert!(
        piece_panics_line.ends_with(" unattributed_panic_states=1"),
        "{piece_panics_line}"
    );
    let mut lines = ResultLines {
        emitted: 0,
        collected: Some(Vec::new()),
    };
    let exit_code = stop_the_confirmed_loss_form_and_write_the_verdict_when_a_panic_is_unattributed(
        &mut lines,
        0,
        &observations,
    );
    let collected = lines.collected.expect("收集模式");
    assert_eq!(exit_code, Some(3), "退 3：{collected:?}");
    assert_eq!(
        collected
            .iter()
            .filter(|line| line.starts_with("name=verdict ")
                && line.contains(" s13=false")
                && line.contains(" q5_not_run=unattributed_panic ")
                && line.ends_with(" unattributed_panic_states=1"))
            .count(),
        1,
        "一行判决带 s13=false、q5_not_run=unattributed_panic 与 unattributed_panic_states=1：{collected:?}"
    );
}
