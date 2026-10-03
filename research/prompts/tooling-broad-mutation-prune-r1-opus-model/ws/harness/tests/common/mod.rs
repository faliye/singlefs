/// 经 `mod common;` 与 `#[path = "../../harness/tests/common/mod.rs"]` 两种写法共享的模块（照 harness 的 tests/common/mod.rs）。
pub fn sample_lengths(limit: u64) -> Vec<u64> {
    (0..limit).collect()
}
