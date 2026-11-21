//! 每个状态上做的四件事（登记 5.1）：基线、各臂的键与判定、checker 读的计数与重放的单测。

use super::*;

#[test]
fn recorded_recovery_answers_exactly_like_the_baseline_on_the_first_stream() {
    let (_stream, totals, _small) =
        evaluate_the_first_stream_small_domain(2, TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT);
    assert_eq!(totals.states(), 53, "D1-small 新池新建文件流 53 个状态");
    assert_eq!(
        totals.consult_report_differs, 0,
        "S5：记录跑看 journal 那一遍与照跑逐项相同"
    );
    assert_eq!(totals.ignore_report_differs, 0, "S5：不看 journal 那一遍");
    assert_eq!(totals.verdicts_differ, 0, "S5：checker 正常那一遍");
    assert_eq!(
        totals.tally.file_read_states, 12,
        "4.1：读出文件的 12 个状态都在 D1-small 里"
    );
}
