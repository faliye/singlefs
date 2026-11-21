//! 一条流导出：取样的状态过一遍实现的判定，写出中性格式的各个文件的单测。

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
