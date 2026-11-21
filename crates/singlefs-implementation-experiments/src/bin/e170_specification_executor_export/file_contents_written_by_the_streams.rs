//! 流二的内容生成式（照抄测试文件）。

use super::*;

// ───────────────────────────── 流二的内容生成式（照抄测试文件） ─────────────────────────────

/// `tests/crash_enumeration_fixed_script_stream.rs` 的 `SECOND_FILE_BYTES` 与 `second_content`。
const SECOND_FILE_BYTES: usize = 4100;

pub(super) fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 同一份文件第 131–143 行。
const THIRD_FILE_BYTES: usize = 2500;

pub(super) fn later_content(seed: usize) -> Vec<u8> {
    (0..3000 + seed)
        .map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256"))
        .collect()
}

pub(super) fn third_content() -> Vec<u8> {
    (0..THIRD_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

/// 第一个文件的内容按本地常量现算，与验证包库的 `scenario::first_file_content()` 回比（S1）。
pub(super) fn local_first_file_content() -> Vec<u8> {
    (0..LOCAL_FIRST_FILE_BYTES)
        .map(|index| {
            u8::try_from(index % LOCAL_FIRST_FILE_CONTENT_MODULUS).expect("模 251 小于 256")
        })
        .collect()
}
