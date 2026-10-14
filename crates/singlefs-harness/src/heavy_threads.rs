//! 每台默认重活线程数（编译、测试、崩溃枚举、注入战役、装置的工作线程）：crates 里的取法只有这一份。
//! 登记位是仓根的 `.claude/heavy-thread-default`（写法在它的文件头），构建时经 `include_str!` 编进来，运行时不找仓根；
//! 判法与 `scripts/hosts/heavy_threads.py` 的 `default_heavy_threads`、`scripts/hosts/heavy-threads.sh` 的 `heavy_thread_default` 相同：
//! `SINGLEFS_THREAD_CAP` 设了（非空）照它，没设取登记文件第一条非注释行的数；固定取登记值，不看本机核数与此刻忙闲（用户 2026-10-12 定）。
//! 各处自己的线程变量（`SINGLEFS_CRASH_AMPLIFICATION_THREADS`、`E165_THREADS` 这类）设了照它，没设经 [`own_variable_or_default_heavy_threads`] 落到这里。

use std::num::NonZeroUsize;

use crate::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};

/// `scripts/hosts/capped.sh <N>` 设成 N 的通用线程上限；设了（非空）就是每台默认重活线程数。
pub const THREAD_CAP_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_THREAD_CAP";

/// 登记文件在仓里的位置（只给消息用；内容在构建时编进 [`REGISTRATION_FILE_TEXT`]）。
pub const REGISTRATION_FILE_IN_THE_REPOSITORY: &str = ".claude/heavy-thread-default";

/// 构建时编进来的登记文件原文：路径从这个包的清单目录往上两级到仓根。只拷 `crates/` 的副本里没有这份文件就编不过，
/// 不退回按核数算。相对的那一段单写成以 `../` 起头的字面量：`scripts/mutation/crates-mutation-rows.py` 的 `escaping_literals`
/// 按清单目录解析它，拷贝范围漏了这份登记时门禁 code-source-discipline 的 mutation-tables 格点名。
pub const REGISTRATION_FILE_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/",
    "../../.claude/heavy-thread-default"
));

/// 登记文件原文里的每台默认重活线程数：第一条非注释行（空行与 `#` 开头的行不算，行首的空格与制表符不算）要是
/// 「`<正整数><空格或制表符>#<理由>`」，理由不空，交那个数。
///
/// # Panics
/// 没有非注释行，或第一条非注释行不是这个形状（数为 0、以 0 开头、不是数、数与 `#` 之间没有空白、没写理由），或数大过 `usize`：
/// 编进来的登记写坏了是 bug，不退回按核数算。
#[must_use]
pub fn heavy_threads_registered_in(registration_text: &str) -> NonZeroUsize {
    let mut lines: Vec<&str> = registration_text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    for (line_index, line) in lines.iter().enumerate() {
        let without_leading_blanks = line.trim_start_matches([' ', '\t']);
        if without_leading_blanks.is_empty() || without_leading_blanks.starts_with('#') {
            continue;
        }
        return count_of_the_registration_line(without_leading_blanks).unwrap_or_else(|| {
            panic!(
                "{REGISTRATION_FILE_IN_THE_REPOSITORY} 第 {} 行「{line}」不是「<正整数>  # <理由>」：编进来的登记写坏了，\
                 照它的文件头改成「31  # <理由>」再编",
                line_index + 1
            )
        });
    }
    panic!(
        "{REGISTRATION_FILE_IN_THE_REPOSITORY} 里没有非注释行，读不出每台默认重活线程数：照它的文件头写一行「31  # <理由>」再编"
    )
}

/// 第一条非注释行（已去掉行首空白）是「数、至少一个空格或制表符、`#`、可有空白、至少一个非空白的理由」时交那个数，否则 `None`。
fn count_of_the_registration_line(line_without_leading_blanks: &str) -> Option<NonZeroUsize> {
    let digit_count = line_without_leading_blanks
        .bytes()
        .take_while(u8::is_ascii_digit)
        .count();
    let (digits, after_the_digits) = line_without_leading_blanks.split_at(digit_count);
    if digits.is_empty() || digits.starts_with('0') {
        return None;
    }
    let after_the_blanks = after_the_digits.trim_start_matches([' ', '\t']);
    if after_the_blanks.len() == after_the_digits.len() {
        return None;
    }
    let reason = after_the_blanks.strip_prefix('#')?;
    if reason.trim_start_matches([' ', '\t']).is_empty() {
        return None;
    }
    digits.parse::<NonZeroUsize>().ok()
}

/// 编进来的那一份登记文件里的数（[`heavy_threads_registered_in`] 判 [`REGISTRATION_FILE_TEXT`]）。
///
/// # Panics
/// 编进来的登记写坏了（[`heavy_threads_registered_in`]）。
#[must_use]
pub fn registered_heavy_threads() -> NonZeroUsize {
    heavy_threads_registered_in(REGISTRATION_FILE_TEXT)
}

/// 每台默认重活线程数的判法本身，`SINGLEFS_THREAD_CAP` 的值与登记文件原文都由调用方给（用例不改进程环境）：
/// 值设了而非空，要是正整数（只认十进制数字、不以 0 开头，与 `scripts/hosts/heavy_threads.py` 同一个判法）就交它；
/// 没设或空串，交登记文件里的数。
///
/// # Panics
/// `SINGLEFS_THREAD_CAP` 设了而不是正整数（消息带变量名与原值，不当成没设、不退回登记值）；或登记写坏了（[`heavy_threads_registered_in`]）。
#[must_use]
pub fn default_heavy_threads_given(
    thread_cap_value: Option<&str>,
    registration_text: &str,
) -> NonZeroUsize {
    match thread_cap_value {
        Some(text) if !text.is_empty() => {
            let is_positive_integer =
                text.bytes().all(|byte| byte.is_ascii_digit()) && !text.starts_with('0');
            is_positive_integer
                .then(|| text.parse::<NonZeroUsize>().ok())
                .flatten()
                .unwrap_or_else(|| {
                    panic!(
                        "{THREAD_CAP_ENVIRONMENT_VARIABLE}={text:?} 认不出：要写正整数（经 scripts/hosts/capped.sh <N> 起就是 N），\
                         或 unset 它取 {REGISTRATION_FILE_IN_THE_REPOSITORY} 的登记值；不退回登记值"
                    )
                })
        }
        Some(_) | None => heavy_threads_registered_in(registration_text),
    }
}

/// 同 [`default_heavy_threads_given`]，登记取编进来的那一份；`SINGLEFS_THREAD_CAP` 的值由调用方给。
///
/// # Panics
/// 同 [`default_heavy_threads_given`]。
#[must_use]
pub fn default_heavy_threads_of_the_cap_value(thread_cap_value: Option<&str>) -> NonZeroUsize {
    default_heavy_threads_given(thread_cap_value, REGISTRATION_FILE_TEXT)
}

/// 每台默认重活线程数：进程环境里的 `SINGLEFS_THREAD_CAP` 设了照它，没设取编进来的登记值。
///
/// # Panics
/// 同 [`default_heavy_threads_given`]；`SINGLEFS_THREAD_CAP` 的值不是 UTF-8 也 panic。
#[must_use]
pub fn default_heavy_threads() -> NonZeroUsize {
    default_heavy_threads_of_the_cap_value(
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

/// 一处自己的线程变量：设了照它（经数值环境变量共用的那一份解析，取 `NonZeroUsize`，写错、写 0 都 panic、消息带变量名与原值），
/// 没设取每台默认重活线程数（[`default_heavy_threads_of_the_cap_value`]）。两个值都由调用方给（用例不改进程环境）。
///
/// # Panics
/// 自己的变量设了而不是正整数；或落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
#[must_use]
pub fn own_variable_or_default_heavy_threads(
    own_variable: &str,
    own_value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> NonZeroUsize {
    number_from_the_variable_value_panicking_when_unreadable::<NonZeroUsize>(
        own_variable,
        own_value,
    )
    .unwrap_or_else(|| default_heavy_threads_of_the_cap_value(thread_cap_value))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_REGISTRATION_OF_SEVEN: &str = "7  # 样本理由\n";

    /// 没设 `SINGLEFS_THREAD_CAP`：等于编进来的登记文件里的数（不是本机核数）。
    #[test]
    fn the_default_is_the_registered_count_when_the_thread_cap_is_unset() {
        assert_eq!(
            default_heavy_threads_of_the_cap_value(None),
            registered_heavy_threads(),
            "没设 SINGLEFS_THREAD_CAP 取 {REGISTRATION_FILE_IN_THE_REPOSITORY} 的登记值"
        );
        assert_eq!(
            default_heavy_threads_given(None, SAMPLE_REGISTRATION_OF_SEVEN).get(),
            7,
            "没设 SINGLEFS_THREAD_CAP 取给的那份登记里的数"
        );
    }

    /// 设成空串当没设（与 `scripts/hosts/heavy_threads.py` 相同）。
    #[test]
    fn an_empty_thread_cap_counts_as_unset() {
        assert_eq!(
            default_heavy_threads_given(Some(""), SAMPLE_REGISTRATION_OF_SEVEN).get(),
            7,
            "空串当没设"
        );
    }

    /// 设了 `SINGLEFS_THREAD_CAP`：等于设的数，盖过登记值。
    #[test]
    fn the_default_is_the_thread_cap_when_it_is_set() {
        assert_eq!(
            default_heavy_threads_given(Some("5"), SAMPLE_REGISTRATION_OF_SEVEN).get(),
            5,
            "设了照它"
        );
        assert_eq!(
            default_heavy_threads_of_the_cap_value(Some("5")).get(),
            5,
            "编进来的登记也被它盖过"
        );
    }

    /// `SINGLEFS_THREAD_CAP` 写成 0、不是数、带空白或正号：panic，带变量名与原值，不退回登记值。
    #[test]
    fn an_unreadable_thread_cap_panics_naming_the_variable_and_the_value() {
        for value in ["0", "abc", " 5", "+5", "05", "-1"] {
            let caught = std::panic::catch_unwind(|| {
                default_heavy_threads_given(Some(value), SAMPLE_REGISTRATION_OF_SEVEN)
            });
            let payload = caught
                .err()
                .unwrap_or_else(|| panic!("SINGLEFS_THREAD_CAP={value:?} 要停下，不退回登记值"));
            let message = payload
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_default();
            assert!(
                message.contains(&format!("SINGLEFS_THREAD_CAP={value:?} 认不出")),
                "消息带变量名与原值：{message}"
            );
        }
    }

    /// 编进来的那一份登记读得出一个正整数。
    #[test]
    fn the_compiled_registration_reads_as_a_positive_count() {
        assert!(registered_heavy_threads().get() >= 1);
    }

    /// 只认第一条非注释行：前面的注释行、空行、行首空白不算，后面的行不看。
    #[test]
    fn only_the_first_non_comment_line_of_the_registration_counts() {
        assert_eq!(
            heavy_threads_registered_in("# 注释\n\n  \t\n7  # 样本理由\n8  # 第二条不认\n").get(),
            7
        );
        assert_eq!(heavy_threads_registered_in("\t7\t# 理由").get(), 7);
    }

    /// 登记写坏的几种形状都 panic（编进来的登记写坏是 bug），与 `scripts/hosts/heavy_threads.py` 拒的那几格相同。
    #[test]
    fn a_malformed_registration_panics() {
        for registration in [
            "",
            "# 只有注释\n",
            "abc  # 样本理由\n",
            "0  # 样本理由\n",
            "07  # 样本理由\n",
            "7\n",
            "7# 样本理由\n",
            "7  #\n",
            "7  #  \t\n",
            "7  理由\n",
        ] {
            let caught = std::panic::catch_unwind(|| heavy_threads_registered_in(registration));
            assert!(caught.is_err(), "{registration:?} 应当 panic");
        }
    }

    /// 自己的变量设了照它，没设落到每台默认重活线程数；写 0 照共用解析 panic。
    #[test]
    fn the_own_variable_wins_and_the_default_is_the_registered_count_when_it_is_unset() {
        assert_eq!(
            own_variable_or_default_heavy_threads("SINGLEFS_EXAMPLE_THREADS", Some("3"), Some("5"))
                .get(),
            3,
            "自己的变量先读"
        );
        assert_eq!(
            own_variable_or_default_heavy_threads("SINGLEFS_EXAMPLE_THREADS", None, Some("5"))
                .get(),
            5,
            "没设自己的变量读 SINGLEFS_THREAD_CAP"
        );
        assert_eq!(
            own_variable_or_default_heavy_threads("SINGLEFS_EXAMPLE_THREADS", None, None),
            registered_heavy_threads(),
            "都没设取登记值"
        );
    }

    #[test]
    #[should_panic(expected = "SINGLEFS_EXAMPLE_THREADS=\"0\" 认不出")]
    fn an_own_variable_of_zero_panics_naming_the_variable_and_the_value() {
        let _threads =
            own_variable_or_default_heavy_threads("SINGLEFS_EXAMPLE_THREADS", Some("0"), None);
    }
}
