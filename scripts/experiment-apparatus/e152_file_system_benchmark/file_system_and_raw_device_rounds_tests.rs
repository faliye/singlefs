use super::*;

#[test]
fn inner_singlefs_line_is_reemitted_without_a_name_equals_done_field() {
    let reemitted = reemit_inner_line(5, "name=done emitted=9");
    assert_eq!(reemitted, "at_nanoseconds=5 inner=done emitted=9");
    assert!(!reemitted.contains("name=done"), "vm-bench.sh 按 name=done emitted= 找收尾行");
}

#[test]
// wallclock-assert-load-safe: 比的是同一条单调时钟上先后盖的戳：每行时刻都在读完那一刻之前盖下，次序由读取先后定，耗时多长都不翻面
fn arrival_reader_returns_every_child_line_in_order_with_nondecreasing_timestamps() {
    let child_output = "E7RESULT name=transaction root_txg=3\nnot a result line\nE7RESULT name=file_overwrite nanoseconds=6608505\nE7RESULT name=done emitted=3";
    let started = Instant::now();
    let arrived_lines = read_lines_with_arrival_times(child_output.as_bytes(), started).expect("内存里的输入读不出错");
    let read_finished_nanoseconds = started.elapsed().as_nanos();
    let texts: Vec<&str> = arrived_lines.iter().map(|arrived_line| arrived_line.text.as_str()).collect();
    assert_eq!(
        texts,
        [
            "E7RESULT name=transaction root_txg=3",
            "not a result line",
            "E7RESULT name=file_overwrite nanoseconds=6608505",
            "E7RESULT name=done emitted=3"
        ],
        "四行全要、次序不变，不是结果行的也要，末行没有换行也要"
    );
    for adjacent_pair in arrived_lines.windows(2) {
        assert!(
            adjacent_pair[0].at_nanoseconds <= adjacent_pair[1].at_nanoseconds,
            "读出次序上的时刻不许倒退：{arrived_lines:?}"
        );
    }
    assert!(
        arrived_lines.iter().all(|arrived_line| arrived_line.at_nanoseconds <= read_finished_nanoseconds),
        "每行的时刻都早于读完那一刻 {read_finished_nanoseconds}：{arrived_lines:?}"
    );
}

#[test]
fn outer_phase_contains_inner_phase_when_equal_or_larger_and_not_when_smaller_or_missing() {
    assert_eq!(outer_phase_contains_inner_phase(Some(6_608_505), Some(6_608_505)), Some(true), "相等算包住");
    assert_eq!(
        outer_phase_contains_inner_phase(Some(16_327_481), Some(6_368_614)),
        Some(true),
        "外层 16.33 ms 包住里层 6.37 ms"
    );
    assert_eq!(
        outer_phase_contains_inner_phase(Some(1_991_507), Some(6_608_505)),
        Some(false),
        "外层 1.99 ms 包不住里层 6.61 ms（2026-09-17 第二次正式跑第 2 轮）"
    );
    assert_eq!(outer_phase_contains_inner_phase(None, Some(6_608_505)), None, "外层缺值判不了");
    assert_eq!(outer_phase_contains_inner_phase(Some(1_991_507), None), None, "里层缺值判不了");
}

#[test]
fn file_overwrite_containment_fields_compare_the_arrival_phase_with_the_child_nanoseconds_field() {
    assert_eq!(phase_nanoseconds(Some(43_231_321), Some(41_239_814)), Some(1_991_507), "两行到达时刻之差");
    assert_eq!(phase_nanoseconds(Some(43_231_321), None), None);
    let child_fields: BTreeMap<String, String> =
        parse_key_values("name=file_overwrite root_txg=4 transaction=2 released=8 nanoseconds=6608505 operations=23")
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();
    assert_eq!(
        file_overwrite_containment_fields(Some(1_991_507), Some(&child_fields)),
        "file_overwrite_inner_nanoseconds=6608505 file_overwrite_outer_contains_inner=false"
    );
    assert_eq!(
        file_overwrite_containment_fields(Some(16_327_481), Some(&child_fields)),
        "file_overwrite_inner_nanoseconds=6608505 file_overwrite_outer_contains_inner=true"
    );
    assert_eq!(
        file_overwrite_containment_fields(None, Some(&child_fields)),
        "file_overwrite_inner_nanoseconds=6608505 file_overwrite_outer_contains_inner=NA"
    );
    assert_eq!(
        file_overwrite_containment_fields(Some(1_991_507), None),
        "file_overwrite_inner_nanoseconds=NA file_overwrite_outer_contains_inner=NA",
        "子进程的 file_overwrite 行没到"
    );
}
