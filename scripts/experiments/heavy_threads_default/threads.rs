// 实验装置没给线程数时的缺省（用户 2026-10-12 定：所有的默认都跑 31 线程，可以输入参数调整，不输入就是登记的默认）。
// 每个用到它的装置在文件里 `include!("heavy_threads_default/threads.rs");`，再把登记文件的原文传进来：
// `default_heavy_threads(include_str!("../../.claude/heavy-thread-default"))`（include_str 的路径从调用它的那个文件算，所以由装置自己写）。
// 规矩与 scripts/hosts/heavy_threads.py、scripts/hosts/heavy-threads.sh 相同：SINGLEFS_THREAD_CAP 是正整数取它（scripts/hosts/capped.sh 设），
// 否则取登记文件第一条非注释行的头一个词。

/// 登记文件原文里第一条非注释行的头一个词，要是正整数；没有这样一行返回 `None`。
fn first_registered_threads(registry_text: &str) -> Option<usize> {
    registry_text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .and_then(|line| line.split_whitespace().next())
        .and_then(|word| word.parse::<usize>().ok())
        .filter(|count| *count >= 1)
}

/// 没给线程数时用几个线程：`SINGLEFS_THREAD_CAP` 是正整数取它，否则登记文件的头一条数。登记文件读不出数就停（不悄悄退回 1）。
fn default_heavy_threads(registry_text: &str) -> usize {
    std::env::var("SINGLEFS_THREAD_CAP")
        .ok()
        .and_then(|text| text.parse::<usize>().ok())
        .filter(|count| *count >= 1)
        .or_else(|| first_registered_threads(registry_text))
        .expect(".claude/heavy-thread-default 要有一条非注释行，头一个词是正整数")
}

#[cfg(test)]
mod heavy_threads_default_tests {
    use super::first_registered_threads;

    #[test]
    fn the_first_non_comment_line_gives_the_default() {
        assert_eq!(first_registered_threads("# 注释\n\n31  # 理由\n"), Some(31));
        assert_eq!(first_registered_threads("  8\n31\n"), Some(8), "只看第一条非注释行");
    }

    #[test]
    fn a_registry_without_a_positive_number_gives_none() {
        assert_eq!(first_registered_threads("# 只有注释\n"), None);
        assert_eq!(first_registered_threads(""), None);
        assert_eq!(first_registered_threads("0  # 不是正整数\n"), None);
        assert_eq!(first_registered_threads("abc\n"), None);
    }
}
