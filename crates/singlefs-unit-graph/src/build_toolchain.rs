//! 一次构建的工具链行（设计 3.1「工具链」，底稿第二节第 4 条「构建环境按 cargo 解析后的生效值进键，不取配置文件原文」）：
//! `rustc -vV` 的每一行、`cargo -V` 那一行、目标三元组、cargo 交给编译器的 RUSTFLAGS，加上这一趟 profile 的生效值。
//!
//! profile 的生效值不是哪个环境变量能直接给的（cargo 给构建脚本的 `OPT_LEVEL`、`DEBUG` 之外，`overflow-checks`、`debug-assertions`
//! 这类一个都不给），这里照 cargo 的口径现算：内置的 `dev` / `release` 默认值，`test` 继承 `dev`、`bench` 继承 `release`、自定义 profile 照
//! `inherits` 往上找；工作区根 `Cargo.toml` 的 `[profile.<名>]` 逐层盖上去，最后盖 `CARGO_PROFILE_<名>_<键>` 环境变量。
//! 不认的：`.cargo/config.toml` 与 `--config` 里写的 profile、`[profile.<名>.package.<包>]` 与 `build-override`（今天工作区里一处都没有）。
//!
//! `singlefs-checker-tier` 的构建脚本与命令行 `unit-graph --toolchain-of-the-build <profile>` 都经 [`toolchain_lines_of_the_build`] 拼，
//! 两边逐字相同（同一份输入算出同一把键）。

use std::collections::BTreeMap;

use crate::manifest::{parse_toml_entries, TomlValue};

/// 进键的 profile 设置：会改编出来的代码怎么跑的那几样（按 cargo 文档 Profiles 一节的键名）。
pub const PROFILE_SETTING_KEYS: [&str; 8] = [
    "opt-level",
    "debug",
    "debug-assertions",
    "overflow-checks",
    "lto",
    "panic",
    "incremental",
    "codegen-units",
];

/// 内置 profile 的默认值与它继承谁（cargo 文档 Profiles 一节「Default profiles」）。
fn builtin_profile(profile_name: &str) -> Option<(Option<&'static str>, [&'static str; 8])> {
    // 次序照 [`PROFILE_SETTING_KEYS`]
    match profile_name {
        "dev" => Some((
            None,
            [
                "0", "true", "true", "true", "false", "unwind", "true", "256",
            ],
        )),
        "release" => Some((
            None,
            [
                "3", "false", "false", "false", "false", "unwind", "false", "16",
            ],
        )),
        "test" => Some((Some("dev"), [""; 8])),
        "bench" => Some((Some("release"), [""; 8])),
        _ => None,
    }
}

/// 一个 TOML 值写成键行里的文字：串照原样，布尔写 `true` / `false`，数照原文。
fn setting_text(value: &TomlValue) -> Option<String> {
    match value {
        TomlValue::Text(text) | TomlValue::Unparsed(text) => Some(text.clone()),
        TomlValue::Boolean(flag) => Some(flag.to_string()),
        TomlValue::Array(_) | TomlValue::InlineTable(_) => None,
    }
}

/// `CARGO_PROFILE_<名>_<键>` 的变量名（名与键大写、`-` 换成 `_`）。
#[must_use]
pub fn profile_environment_variable(profile_name: &str, setting_key: &str) -> String {
    format!(
        "CARGO_PROFILE_{}_{}",
        profile_name.to_ascii_uppercase().replace('-', "_"),
        setting_key.to_ascii_uppercase().replace('-', "_")
    )
}

/// 构建脚本看到的 `PROFILE`（`debug` 或 `release`）对应的 profile 名：`debug` 是 `dev` 一系（`test` 继承它），`release` 是
/// `release` 一系（`bench` 继承它）。别的值原样交回。
#[must_use]
pub fn profile_name_of_the_build_script_profile_variable(profile_variable: &str) -> &str {
    match profile_variable {
        "debug" => "dev", // naming-lint:external cargo 定的构建档名：调试档叫 dev
        other => other,
    }
}

/// 这个 profile 的生效值：`profile: <名>` 一行，再按 [`PROFILE_SETTING_KEYS`] 的次序每个键一行 `profile.<键>: <值>`。
/// `environment` 读环境变量（构建脚本交 `std::env::var`，测试交假的）。
///
/// # Errors
/// 根清单读不了；profile 不是内置的四种、根清单里也没写；`inherits` 绕成了圈。
pub fn effective_profile_lines(
    root_manifest_text: &str,
    profile_name: &str,
    environment: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<String>, String> {
    let entries = parse_toml_entries(root_manifest_text)
        .map_err(|error| format!("根 Cargo.toml 第 {} 行读不了：{}", error.line, error.reason))?;
    let mut written: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for entry in entries {
        if let ([profile_table, name], [key]) =
            (entry.table_path.as_slice(), entry.key_path.as_slice())
        {
            if profile_table == "profile" {
                if let Some(text) = setting_text(&entry.value) {
                    written
                        .entry(name.clone())
                        .or_default()
                        .insert(key.clone(), text);
                }
            }
        }
    }
    // 从这一个 profile 往上找到内置的根：链从叶到根
    let mut chain: Vec<String> = Vec::new();
    let mut current = profile_name.to_string();
    let root_defaults = loop {
        if chain.contains(&current) {
            return Err(format!(
                "profile 的 inherits 绕成了圈：{}",
                chain.join(" → ")
            ));
        }
        chain.push(current.clone());
        let inherited_from_the_manifest = written
            .get(&current)
            .and_then(|settings| settings.get("inherits"))
            .cloned();
        match (builtin_profile(&current), inherited_from_the_manifest) {
            (Some((None, defaults)), _) => break defaults,
            (Some((Some(parent), _)), None) => current = parent.to_string(),
            (_, Some(parent)) => current = parent,
            (None, None) => {
                return Err(format!(
                    "profile「{current}」不是内置的 dev / release / test / bench，根 Cargo.toml 里也没写它的 inherits"
                ))
            }
        }
    };
    let mut effective: BTreeMap<&str, String> = PROFILE_SETTING_KEYS
        .iter()
        .zip(root_defaults)
        .map(|(key, value)| (*key, value.to_string()))
        .collect();
    for profile in chain.iter().rev() {
        if let Some(settings) = written.get(profile) {
            for key in PROFILE_SETTING_KEYS {
                if let Some(value) = settings.get(key) {
                    effective.insert(key, value.clone());
                }
            }
        }
    }
    for key in PROFILE_SETTING_KEYS {
        if let Some(value) = environment(&profile_environment_variable(profile_name, key)) {
            effective.insert(key, value);
        }
    }
    let mut lines = vec![format!("profile: {profile_name}")];
    lines.extend(
        PROFILE_SETTING_KEYS
            .iter()
            .map(|key| format!("profile.{key}: {}", effective[key])),
    );
    Ok(lines)
}

/// 拼工具链行要的输入：各样的原文，由调用方取（构建脚本跑 `$RUSTC -vV`、`$CARGO -V`，读 `TARGET`、`CARGO_ENCODED_RUSTFLAGS`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildToolchainInputs<'inputs> {
    pub rustc_version_text: &'inputs str,
    pub cargo_version_text: &'inputs str,
    pub target_triple: &'inputs str,
    /// cargo 交给编译器的 RUSTFLAGS，照 `CARGO_ENCODED_RUSTFLAGS` 的写法：几个参数以 0x1f 隔开，没有就是空串。
    pub encoded_rustflags: &'inputs str,
    pub profile_lines: &'inputs [String],
}

/// 工具链行：`rustc -vV` 的每个非空行原样、`cargo-version: <cargo -V>`、`target: <三元组>`（闭包按它取平台条件）、
/// `rustflags: <参数以空格连起>`、profile 的生效值（[`effective_profile_lines`]）。
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
    lines.extend(inputs.profile_lines.iter().cloned());
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT_MANIFEST: &str = "[workspace]\nmembers = [\"crates/alpha\"]\n\n[profile.release]\nopt-level = 3\ndebug = true\n\
        overflow-checks = true\ndebug-assertions = true\n\n[profile.checked]\ninherits = \"release\"\nlto = \"thin\"\n";

    fn no_environment(_name: &str) -> Option<String> {
        None
    }

    #[test]
    fn the_release_profile_takes_the_root_manifest_over_the_builtin_defaults() {
        let lines =
            effective_profile_lines(ROOT_MANIFEST, "release", &no_environment).expect("算得出");
        assert!(
            lines.contains(&"profile.overflow-checks: true".to_string()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"profile.debug-assertions: true".to_string()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"profile.codegen-units: 16".to_string()),
            "内置默认：{lines:?}"
        );
    }

    #[test]
    fn bench_and_a_custom_profile_inherit_release_and_the_environment_overrides_last() {
        let bench =
            effective_profile_lines(ROOT_MANIFEST, "bench", &no_environment).expect("算得出");
        assert!(
            bench.contains(&"profile.overflow-checks: true".to_string()),
            "{bench:?}"
        );
        let custom =
            effective_profile_lines(ROOT_MANIFEST, "checked", &no_environment).expect("算得出");
        assert!(
            custom.contains(&"profile.lto: thin".to_string()),
            "{custom:?}"
        );
        assert!(
            custom.contains(&"profile.opt-level: 3".to_string()),
            "{custom:?}"
        );
        let overridden = effective_profile_lines(ROOT_MANIFEST, "release", &|name| {
            (name == "CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS").then(|| "false".to_string())
        })
        .expect("算得出");
        assert!(
            overridden.contains(&"profile.overflow-checks: false".to_string()),
            "{overridden:?}"
        );
    }

    #[test]
    fn the_dev_profile_without_a_table_keeps_the_builtin_overflow_checks() {
        let lines = effective_profile_lines(ROOT_MANIFEST, "dev", &no_environment).expect("算得出");
        assert!(
            lines.contains(&"profile.overflow-checks: true".to_string()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"profile.opt-level: 0".to_string()),
            "{lines:?}"
        );
    }

    #[test]
    fn an_unknown_profile_without_inherits_is_an_error() {
        assert!(effective_profile_lines(ROOT_MANIFEST, "nowhere", &no_environment).is_err());
    }

    #[test]
    fn the_toolchain_lines_carry_rustc_cargo_target_rustflags_and_the_profile() {
        let profile_lines = vec!["profile: release".to_string()];
        let lines = toolchain_lines_of_the_build(&BuildToolchainInputs {
            rustc_version_text: "rustc 1.98.0\nhost: x86_64-unknown-linux-gnu\n",
            cargo_version_text: "cargo 1.98.0\n",
            target_triple: "x86_64-unknown-linux-gnu",
            encoded_rustflags: "-C\u{1f}target-feature=+crt-static",
            profile_lines: &profile_lines,
        });
        assert_eq!(
            lines,
            [
                "rustc 1.98.0",
                "host: x86_64-unknown-linux-gnu",
                "cargo-version: cargo 1.98.0",
                "target: x86_64-unknown-linux-gnu",
                "rustflags: -C target-feature=+crt-static",
                "profile: release",
            ]
        );
    }
}
