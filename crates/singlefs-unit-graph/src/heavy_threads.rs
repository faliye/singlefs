//! 没给 `--threads` 时用几个线程：取法与 `scripts/experiments/heavy_threads_default/threads.rs` 相同——
//! `SINGLEFS_THREAD_CAP` 是正整数照它（`scripts/hosts/capped.sh` 设），否则取登记文件 `.claude/heavy-thread-default`
//! 第一条非注释行的头一个词。登记文件在构建时编进来，运行时不找仓根；本 crate 不依赖 `singlefs-harness`，所以不用它那一份。

use std::num::NonZeroUsize;

/// `scripts/hosts/capped.sh <N>` 设成 N 的通用线程上限。
pub const THREAD_CAP_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_THREAD_CAP";

/// 构建时编进来的登记文件原文：路径从这个包的清单目录往上两级到仓根。相对的那一段单写成以 `../` 起头的字面量，
/// 拷贝范围漏了这份登记时门禁 code-source-discipline 的 mutation-tables 格点名（同 `singlefs-harness` 的 `heavy_threads`）。
pub const REGISTRATION_FILE_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/",
    "../../.claude/heavy-thread-default"
));

/// 登记文件原文里第一条非注释行的头一个词，要是正整数；没有这样一行返回 `None`。
#[must_use]
pub fn first_registered_threads(registration_text: &str) -> Option<NonZeroUsize> {
    registration_text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .and_then(|line| line.split_whitespace().next())
        .and_then(|word| word.parse::<usize>().ok())
        .and_then(NonZeroUsize::new)
}

/// 没给线程数时用几个线程。
///
/// # Panics
/// `SINGLEFS_THREAD_CAP` 没设或不是正整数、登记文件又读不出数：编进来的登记写坏了是 bug，不悄悄退回 1 或按核数算。
#[must_use]
pub fn default_heavy_threads() -> NonZeroUsize {
    std::env::var(THREAD_CAP_ENVIRONMENT_VARIABLE)
        .ok()
        .and_then(|text| text.trim().parse::<usize>().ok())
        .and_then(NonZeroUsize::new)
        .or_else(|| first_registered_threads(REGISTRATION_FILE_TEXT))
        .expect(".claude/heavy-thread-default 要有一条非注释行，头一个词是正整数")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_non_comment_line_gives_the_default() {
        assert_eq!(
            first_registered_threads("# 注释\n\n31  # 理由\n"),
            NonZeroUsize::new(31)
        );
        assert_eq!(first_registered_threads("# 只有注释\n"), None);
        assert_eq!(first_registered_threads("0  # 不是正整数\n"), None);
    }
}
