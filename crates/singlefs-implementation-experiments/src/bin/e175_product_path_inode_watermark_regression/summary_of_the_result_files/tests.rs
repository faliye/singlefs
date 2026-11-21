//! `summary_of_the_result_files` 的单测。
use super::super::tests::ScratchFile;
use super::*;

/// 汇总的判法：同一（族、臂、序号）两行内容相同记重复、不同记冲突，没写完的行另数，`done` 行的条数把自己算进去。
#[test]
fn summary_counts_duplicates_conflicts_and_unfinished_lines() {
    let text = "\
E7RESULT name=history family=F2 arm=P index=0 label=a q1=0 q3_peak=2 elapsed_ns=5 line_end=ok
E7RESULT name=history family=F2 arm=P index=0 label=a q1=0 q3_peak=2 elapsed_ns=9 line_end=ok
E7RESULT name=history family=F2 arm=P index=1 label=b q1=3 q3_peak=1 elapsed_ns=5 line_end=ok
E7RESULT name=history family=F2 arm=P index=1 label=b q1=4 q3_peak=1 elapsed_ns=5 line_end=ok
E7RESULT name=history family=F2 arm=P index=2 label=c q1=0
E7RESULT name=event family=F2 arm=P index=1 kind=q1_same_timeline
E7RESULT name=family_complete family=F2 arm=P histories=2
";
    let output = summarize(text);
    assert_eq!(
    output[0],
    "E7RESULT name=summary family=F2 arm=P histories=2 complete=yes duplicates=1 conflicts=1 q10_segments_issuing_at_least_two=0 ending_=2 sum_q1=3 segments_q1=1 sum_q3_peak=3 segments_q3_peak=2 max_q3_peak=2"
);
    assert_eq!(
        output[1],
        "E7RESULT name=event_count family=F2 arm=P kind=q1_same_timeline events=1"
    );
    assert_eq!(output[2], "E7RESULT name=unfinished_lines count=1");
    assert_eq!(output[3], "E7RESULT name=done emitted=4");
    assert_eq!(output.len(), 4);
}

#[test]
fn summary_of_result_files_carries_the_code_baseline_on_every_line() {
    let input_file = ScratchFile::named_for_the_test("summary-input");
    let summary_file = ScratchFile::named_for_the_test("summary-output");
    std::fs::write(
    &input_file.0,
    "# code_baseline=baseline-text\nE7RESULT name=history family=F2 arm=P index=0 label=a q1=2 line_end=ok\n",
)
.expect("写输入");
    write_summary_of_the_result_files(&[input_file.path_text()], "baseline-text", &summary_file.0);
    assert_eq!(
    std::fs::read_to_string(&summary_file.0).expect("汇总写出来了"),
    "# code_baseline=baseline-text\n\
E7RESULT name=summary family=F2 arm=P histories=1 complete=no duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=0 ending_=1 sum_q1=2 segments_q1=1 code_baseline=baseline-text\n\
E7RESULT name=unfinished_lines count=0 code_baseline=baseline-text\n\
E7RESULT name=done emitted=3 code_baseline=baseline-text\n"
);
}

#[test]
#[should_panic(expected = "汇总写新文件，不覆盖")]
fn summary_onto_an_existing_file_is_refused() {
    let input_file = ScratchFile::named_for_the_test("summary-refused-input");
    let summary_file = ScratchFile::named_for_the_test("summary-refused-output");
    std::fs::write(&input_file.0, "# code_baseline=baseline-text\n").expect("写输入");
    std::fs::write(&summary_file.0, "已有的汇总\n").expect("写已有的汇总");
    write_summary_of_the_result_files(&[input_file.path_text()], "baseline-text", &summary_file.0);
}
