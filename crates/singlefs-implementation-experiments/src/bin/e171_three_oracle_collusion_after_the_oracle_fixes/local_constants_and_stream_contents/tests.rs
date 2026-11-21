//! 本地常量（停机条款 S1 回比的那几个）、三条流的名字与各文件的内容的单测。

use super::*;

/// 三条流本地段长表的闭式 1 + Σ(2^n − 1) 与写数，与本地抄来的闭式、写数逐个相等（改段长表里任何一个数，这里都红）。
#[test]
fn local_segment_tables_reproduce_the_closed_forms_and_write_counts_copied_from_the_tests() {
    let closed_form = |lengths: &[usize]| -> u64 {
        1 + lengths
            .iter()
            .map(|length| (1u64 << length) - 1)
            .sum::<u64>()
    };
    assert_eq!(
        closed_form(&LOCAL_FIRST_STREAM_SEGMENT_LENGTHS),
        LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM
    );
    assert_eq!(
        LOCAL_FIRST_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
        LOCAL_FIRST_STREAM_WRITES
    );
    assert_eq!(
        closed_form(&LOCAL_SECOND_STREAM_SEGMENT_LENGTHS),
        LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM
    );
    assert_eq!(
        LOCAL_SECOND_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
        LOCAL_SECOND_STREAM_WRITES
    );
    assert_eq!(
        closed_form(LOCAL_THIRD_STREAM_SEGMENT_LENGTHS),
        LOCAL_THIRD_STREAM_TWO_STATE_CLOSED_FORM
    );
    assert_eq!(
        LOCAL_THIRD_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
        LOCAL_THIRD_STREAM_WRITES
    );
    assert_eq!(LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM, 1_678_377_095);
    assert_eq!(LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM, 16_777_243);
}

#[test]
fn stream_numbers_select_the_streams_in_order_and_reject_unknown_numbers() {
    assert_eq!(
        stream_names_of_the_numbers("3,1"),
        Some(vec![
            StreamName::NewPoolFileCreation,
            StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf
        ])
    );
    assert_eq!(stream_names_of_the_numbers("4"), None);
    assert_eq!(stream_names_of_the_numbers(""), None);
}
