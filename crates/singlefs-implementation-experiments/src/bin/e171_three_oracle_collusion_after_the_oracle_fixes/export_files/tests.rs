//! 导出文件：中性格式的十个文件、调用结局与返回值、流的状态、oracle 判与可写挂载探针的文件的单测。

use super::*;

#[test]
fn contents_without_timing_drops_only_the_timing_column_and_the_verdicts_digest_line() {
    assert_eq!(
        contents_without_timing(
            IMPLEMENTATION_VERDICTS_FILE,
            b"0\ta\tb\t123\n1\tc\td\t4567\n"
        ),
        Some(b"0\ta\tb\n1\tc\td\n".to_vec())
    );
    assert_eq!(
        contents_without_timing(
            MANIFEST_FILE,
            b"stream\tx\nsha256:states.tsv\taa\nsha256:impl-verdicts.tsv\tbb\n"
        ),
        Some(b"stream\tx\nsha256:states.tsv\taa\n".to_vec())
    );
    assert_eq!(contents_without_timing(STATES_FILE, b"0\t1\n"), None);
}

#[test]
fn call_outcome_writes_ok_or_error_into_the_calls_file_and_a_word_into_the_outcomes_file() {
    assert_eq!(CallOutcome::Succeeded.calls_file_word(), "ok");
    assert_eq!(
        CallOutcome::ReturnedError("NoSpace".to_string()).calls_file_word(),
        "err"
    );
    assert_eq!(
        CallOutcome::ReturnedError("NoSpace".to_string()).outcome_word(),
        "err:NoSpace"
    );
    assert_eq!(
        CallOutcome::Panicked("炸".to_string()).outcome_word(),
        "panic"
    );
    assert_eq!(
        CallOutcome::Panicked("炸".to_string()).calls_file_word(),
        "err"
    );
    assert_eq!(
        CallOutcome::InFlightAtTheTruncation.outcome_word(),
        "in_flight"
    );
}
