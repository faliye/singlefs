//! 样本：验证包库根 forbid，管不到 src/bin/ 下的 bin 根。
#![forbid(unsafe_code)]

pub fn crash_state_count() -> usize {
    0
}
