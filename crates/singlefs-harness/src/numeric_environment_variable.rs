//! 数值环境变量的取值怎么换成数：测试、装置与 checker 档流水线的规模、线程数、显存额度这一类变量共用这一份解析。
//! 单放一个模块、不放在 `history`：checker 档的 `crash_amplification` 也调它，而登记的崩溃枚举用例的判法代码按模块引用收文件
//! （`crash_identity` 的 `every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries`），
//! 随机历史脚手架只许由测试文件自己直接引进来。
//! 变量的原文从进程环境怎么读（[`process_environment_variable`]）也只有这里一份：`history` 的复现入口与 `crash_amplification`
//! 原来各写一份，都经这里。

/// 进程环境里的一个变量的原文：没设是 `None`；不是 UTF-8 就 panic、写明变量名与原值（复现入口或规模变量给错了，不当成没设）。
/// 数值怎么解析交 [`number_from_the_variable_value_panicking_when_unreadable`]。
///
/// # Panics
/// 变量的取值不是 UTF-8。
#[must_use]
pub fn process_environment_variable(variable: &str) -> Option<String> {
    variable_text_panicking_when_not_unicode(variable, std::env::var(variable))
}

/// [`process_environment_variable`] 的判定本身：`std::env::var` 读到什么由调用方给（用例不改进程的环境变量）。
///
/// # Panics
/// `read` 是 [`std::env::VarError::NotUnicode`]。
fn variable_text_panicking_when_not_unicode(
    variable: &str,
    read: Result<String, std::env::VarError>,
) -> Option<String> {
    match read {
        Ok(text) => Some(text),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(raw)) => {
            panic!("{variable} 的值不是 UTF-8：{raw:?}；不当成没设")
        }
    }
}

/// 一个数值变量的值（`None` 是没设）换成数：没设是 `None`，由调用方取它没设时的那一档；设了而认不出就 panic，消息带变量名与原值。
/// 写错的值不悄悄当成没设、不退回默认（照样跑完、照样退 0，跑的就不是要的那一档；代码三方 test-script-align-code-r1 判决 C606）。
/// 线程数这类不许为 0 的，`Number` 取 `NonZeroUsize`，0 同样认不出。
///
/// # Panics
/// 设了而 `Number` 的 `from_str` 认不出（前后空白先去掉）。
#[must_use]
pub fn number_from_the_variable_value_panicking_when_unreadable<Number>(
    variable: &str,
    value: Option<&str>,
) -> Option<Number>
where
    Number: std::str::FromStr,
    Number::Err: std::fmt::Display,
{
    let text = value?;
    match text.trim().parse::<Number>() {
        Ok(number) => Some(number),
        Err(error) => panic!(
            "{variable}={text:?} 认不出（{error}）：要写一个 {} 的取值，不当成没设、不退回没设时的那一档",
            std::any::type_name::<Number>()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 数值变量设成认不出的字样：panic，消息带变量名与原值，不当成没设（C606 那一类：写错的规模悄悄退回默认）。
    #[test]
    #[should_panic(expected = "SINGLEFS_EXAMPLE_LIMIT=\"eight\" 认不出")]
    fn a_number_variable_set_to_text_that_is_not_a_number_panics_naming_the_variable_and_the_value()
    {
        let _limit = number_from_the_variable_value_panicking_when_unreadable::<usize>(
            "SINGLEFS_EXAMPLE_LIMIT",
            Some("eight"),
        );
    }

    /// 不许为 0 的数（线程数）设成 0：同样 panic、带变量名与原值，不悄悄换成至少 1 或本机并行度。
    #[test]
    #[should_panic(expected = "SINGLEFS_EXAMPLE_THREADS=\"0\" 认不出")]
    fn a_nonzero_number_variable_set_to_zero_panics_naming_the_variable_and_the_value() {
        let _threads = number_from_the_variable_value_panicking_when_unreadable::<
            std::num::NonZeroUsize,
        >("SINGLEFS_EXAMPLE_THREADS", Some("0"));
    }

    /// 没设是 `None`：调用方取它没设时的那一档。
    #[test]
    fn an_unset_number_variable_is_none() {
        assert_eq!(
            number_from_the_variable_value_panicking_when_unreadable::<u64>(
                "SINGLEFS_EXAMPLE_LIMIT",
                None
            ),
            None,
            "没设是 None"
        );
    }

    /// 变量的值不是 UTF-8：panic，消息带变量名与原值，不当成没设（没设时调用方会悄悄取它那一档）。
    #[test]
    #[should_panic(expected = "SINGLEFS_EXAMPLE_LIMIT 的值不是 UTF-8")]
    fn a_variable_whose_value_is_not_unicode_panics_naming_the_variable() {
        use std::os::unix::ffi::OsStringExt as _;
        let _text = variable_text_panicking_when_not_unicode(
            "SINGLEFS_EXAMPLE_LIMIT",
            Err(std::env::VarError::NotUnicode(
                std::ffi::OsString::from_vec(vec![0x38, 0xff]),
            )),
        );
    }

    /// 没设的变量读成 `None`。
    #[test]
    fn an_unset_variable_reads_as_none() {
        assert_eq!(
            variable_text_panicking_when_not_unicode(
                "SINGLEFS_EXAMPLE_LIMIT",
                Err(std::env::VarError::NotPresent)
            ),
            None,
            "没设是 None"
        );
    }

    /// 设了的变量原文照读（空白不去：数值解析那一步才去）。
    #[test]
    fn a_set_variable_reads_back_verbatim() {
        assert_eq!(
            variable_text_panicking_when_not_unicode(
                "SINGLEFS_EXAMPLE_LIMIT",
                Ok(" 8\n".to_string())
            ),
            Some(" 8\n".to_string()),
            "设了的原文照读"
        );
    }

    /// 设了的数值变量去掉前后空白读回。
    #[test]
    fn a_set_number_variable_reads_back_without_surrounding_blanks() {
        assert_eq!(
            number_from_the_variable_value_panicking_when_unreadable::<u64>(
                "SINGLEFS_EXAMPLE_LIMIT",
                Some(" 8\n")
            ),
            Some(8),
            "设了的去掉前后空白读回"
        );
    }
}
