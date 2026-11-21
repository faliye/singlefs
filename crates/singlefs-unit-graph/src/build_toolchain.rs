//! 一次构建的工具链行（设计 3.1「工具链」）：`rustc -vV` 的每一行、`cargo -V` 那一行、目标三元组、cargo 交给编译器的 RUSTFLAGS。
//! 编译档（`[profile.<名>]` 的各项设置、增量编译开关）不拼进来，也不读：哪个档编的不进任何键。
//!
//! 命令行 `unit-graph --toolchain-of-the-build` 经 [`toolchain_lines_of_the_build`] 拼。构建脚本不拼工具链行：工具链不进崩溃判定、对拍、
//! 观察者那几把键（[`crate::entry_groups`]，D13（验证路线） 已定项 17）。

/// 拼工具链行要的输入：各样的原文，由调用方取（构建脚本跑 `$RUSTC -vV`、`$CARGO -V`，读 `TARGET`、`CARGO_ENCODED_RUSTFLAGS`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildToolchainInputs<'inputs> {
    pub rustc_version_text: &'inputs str,
    pub cargo_version_text: &'inputs str,
    pub target_triple: &'inputs str,
    /// cargo 交给编译器的 RUSTFLAGS，照 `CARGO_ENCODED_RUSTFLAGS` 的写法：几个参数以 0x1f 隔开，没有就是空串。
    pub encoded_rustflags: &'inputs str,
}

/// 工具链行：`rustc -vV` 的每个非空行原样、`cargo-version: <cargo -V>`、`target: <三元组>`（闭包按它取平台条件）、
/// `rustflags: <参数以空格连起>`。
#[must_use]
pub fn toolchain_lines_of_the_build(inputs: &BuildToolchainInputs<'_>) -> Vec<String> {
    let mut lines: Vec<String> = inputs
        .rustc_version_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    lines.push(format!(
        "cargo-version: {}",
        inputs.cargo_version_text.trim()
    ));
    lines.push(format!(
        "{}{}",
        crate::conditional_compilation::CARGO_TARGET_TOOLCHAIN_LINE_PREFIX,
        inputs.target_triple
    ));
    lines.push(format!(
        "rustflags: {}",
        inputs
            .encoded_rustflags
            .split('\u{1f}')
            .filter(|flag| !flag.is_empty())
            .collect::<Vec<&str>>()
            .join(" ")
    ));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 场景：给定 `rustc -vV`、`cargo -V`、目标三元组与两段 RUSTFLAGS 拼一次工具链行。
    /// 预期：恰好是 rustc 的两行、cargo、目标、RUSTFLAGS 五行，没有编译档那几行。
    /// 判别力（`crates/mutations.tsv`）：拼完又接上一行编译档，这条红。
    #[test]
    fn the_toolchain_lines_carry_rustc_cargo_target_and_rustflags_and_no_profile() {
        let lines = toolchain_lines_of_the_build(&BuildToolchainInputs {
            rustc_version_text: "rustc 1.98.0\nhost: x86_64-unknown-linux-gnu\n",
            cargo_version_text: "cargo 1.98.0\n",
            target_triple: "x86_64-unknown-linux-gnu",
            encoded_rustflags: "-C\u{1f}target-feature=+crt-static",
        });
        assert_eq!(
            lines,
            [
                "rustc 1.98.0",
                "host: x86_64-unknown-linux-gnu",
                "cargo-version: cargo 1.98.0",
                "target: x86_64-unknown-linux-gnu",
                "rustflags: -C target-feature=+crt-static",
            ]
        );
    }
}
