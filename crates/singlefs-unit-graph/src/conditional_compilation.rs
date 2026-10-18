//! `#[cfg(…)]` 与 `#[cfg_attr(…, cfg(…))]` 的谓词：从属性的词法单元解析，按一个编译单元的生效集合求值。
//!
//! 生效集合里有的名字与键值为真；目标平台的那几个键（`target_arch` 这类）、`feature` 与认得的名字（`test`、`debug_assertions` 这类）
//! 不在集合里为假；认不出的名字与写不成谓词的属性判「不知道」——不知道的单元照算进闭包，同时记一条拒算，不静默当假。
//! `cfg(any())` 是空的「任一」，恒假。
//!
//! 目标平台不写死：从工具链目标三元组查 [`TargetPlatform`] 那张表（`target_platform_of_triple`）；三元组由调用方经工具链行给
//! （`target_triple_of_toolchain_lines`：cargo `--target` 的那一行优先，没有就取 `rustc -vV` 的 `host:` 行），表里查不出的三元组由闭包拒算。

use std::collections::BTreeSet;

use crate::lexer::{token_text, Token, TokenKind};

/// 一个条件编译谓词。
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Predicate {
    Name(String),
    KeyValue {
        key: String,
        value: String,
    },
    All(Vec<Predicate>),
    Any(Vec<Predicate>),
    Not(Box<Predicate>),
    /// 写不成谓词的原文（词法单元以空格连起）。
    Unrecognized(String),
}

/// 一个谓词在某个生效集合下的真假。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evaluation {
    Holds,
    DoesNotHold,
    Unknown,
}

/// rustc 自己认得、本工具也认得的无值名字。
const RECOGNIZED_NAMES: [&str; 9] = [
    "test",
    "debug_assertions",
    "unix",
    "windows",
    "doc",
    "doctest",
    "miri",
    "proc_macro",
    "overflow_checks",
];

/// 认得的键：值不在集合里就为假。
const RECOGNIZED_KEYS: [&str; 11] = [
    "feature",
    "target_arch",
    "target_os",
    "target_family",
    "target_env",
    "target_endian",
    "target_pointer_width",
    "target_vendor",
    "target_has_atomic",
    "target_abi",
    "panic",
];

/// 构建默认就有的无值名字（与目标平台无关）：dev、test 与本工作区的 release 都开着 `debug-assertions`（工作区 `Cargo.toml` 的 `[profile.release]`）。
const PROFILE_NAMES: [&str; 1] = ["debug_assertions"];

/// 工具链行里 cargo `--target` 给的目标三元组那一行的前缀（调用方照这个写法给）。
pub const CARGO_TARGET_TOOLCHAIN_LINE_PREFIX: &str = "target: ";
/// `rustc -vV` 的宿主三元组那一行的前缀。
pub const RUSTC_HOST_TOOLCHAIN_LINE_PREFIX: &str = "host: ";

/// 一个目标三元组的平台条件：无值名字（`unix` / `windows`）与认得的键的值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetPlatform {
    pub triple: &'static str,
    pub names: &'static [&'static str],
    pub key_values: &'static [(&'static str, &'static str)],
}

/// 认得的目标三元组：每一行逐项取自 `rustc --print cfg --target <三元组>`（rustc 1.98.0，2026-10-15 在本机跑的），
/// 只留 `RECOGNIZED_KEYS` 里的键与 `unix` / `windows`（十个三元组的 `target_abi` 都是空串、`panic` 都是 `unwind`）；
/// `target_has_atomic` 的几个值列在 `atomics` 一段。
macro_rules! target_platform_table {
    ($(
        $triple:literal: names [$($name:literal),*],
        arch $arch:literal, os $os:literal, family $family:literal, env $environment:literal,
        endian $endian:literal, pointer_width $pointer_width:literal, vendor $vendor:literal,
        atomics [$($atomic:literal),*];
    )*) => {
        /// 目标三元组表（查不出的三元组闭包拒算）。
        pub const TARGET_PLATFORMS: &[TargetPlatform] = &[$(TargetPlatform {
            triple: $triple,
            names: &[$($name),*],
            key_values: &[
                ("target_arch", $arch),
                ("target_os", $os), // naming-lint:external rustc 定的 cfg 名
                ("target_family", $family),
                ("target_env", $environment), // naming-lint:external rustc 定的 cfg 名
                ("target_endian", $endian),
                ("target_pointer_width", $pointer_width),
                ("target_vendor", $vendor),
                ("target_abi", ""),
                ("panic", "unwind"),
                $(("target_has_atomic", $atomic),)*
            ],
        },)*];
    };
}

target_platform_table! {
    "x86_64-unknown-linux-gnu": names ["unix"],
        arch "x86_64", os "linux", family "unix", env "gnu", endian "little", pointer_width "64", vendor "unknown",
        atomics ["8", "16", "32", "64", "ptr"];
    "x86_64-unknown-linux-musl": names ["unix"],
        arch "x86_64", os "linux", family "unix", env "musl", endian "little", pointer_width "64", vendor "unknown",
        atomics ["8", "16", "32", "64", "ptr"];
    "aarch64-unknown-linux-gnu": names ["unix"],
        arch "aarch64", os "linux", family "unix", env "gnu", endian "little", pointer_width "64", vendor "unknown",
        atomics ["8", "16", "32", "64", "128", "ptr"];
    "aarch64-unknown-linux-musl": names ["unix"],
        arch "aarch64", os "linux", family "unix", env "musl", endian "little", pointer_width "64", vendor "unknown",
        atomics ["8", "16", "32", "64", "128", "ptr"];
    "aarch64-apple-darwin": names ["unix"],
        arch "aarch64", os "macos", family "unix", env "", endian "little", pointer_width "64", vendor "apple",
        atomics ["8", "16", "32", "64", "128", "ptr"];
    "x86_64-apple-darwin": names ["unix"],
        arch "x86_64", os "macos", family "unix", env "", endian "little", pointer_width "64", vendor "apple",
        atomics ["8", "16", "32", "64", "128", "ptr"];
    "x86_64-pc-windows-msvc": names ["windows"],
        arch "x86_64", os "windows", family "windows", env "msvc", endian "little", pointer_width "64", vendor "pc",
        atomics ["8", "16", "32", "64", "128", "ptr"];
    "x86_64-pc-windows-gnu": names ["windows"],
        arch "x86_64", os "windows", family "windows", env "gnu", endian "little", pointer_width "64", vendor "pc",
        atomics ["8", "16", "32", "64", "128", "ptr"];
    "i686-unknown-linux-gnu": names ["unix"],
        arch "x86", os "linux", family "unix", env "gnu", endian "little", pointer_width "32", vendor "unknown",
        atomics ["8", "16", "32", "64", "ptr"];
    "riscv64gc-unknown-linux-gnu": names ["unix"],
        arch "riscv64", os "linux", family "unix", env "gnu", endian "little", pointer_width "64", vendor "unknown",
        atomics ["8", "16", "32", "64", "ptr"];
}

/// 按三元组查平台条件；表里没有交 `None`（闭包拒算）。
#[must_use]
pub fn target_platform_of_triple(triple: &str) -> Option<&'static TargetPlatform> {
    TARGET_PLATFORMS
        .iter()
        .find(|platform| platform.triple == triple)
}

/// 工具链行里的目标三元组：cargo `--target` 那一行（[`CARGO_TARGET_TOOLCHAIN_LINE_PREFIX`]）优先，没有就取 `rustc -vV` 的
/// `host:` 行（[`RUSTC_HOST_TOOLCHAIN_LINE_PREFIX`]）；两种都没有交 `Ok(None)`。
///
/// # Errors
/// 同一种行给了不止一行（取哪一个说不清），交那一种行的前缀。
pub fn target_triple_of_toolchain_lines(lines: &[String]) -> Result<Option<String>, String> {
    let triple_after = |prefix: &str| -> Result<Option<String>, String> {
        let found: Vec<&str> = lines
            .iter()
            .filter_map(|line| line.strip_prefix(prefix))
            .map(str::trim)
            .collect();
        match found.as_slice() {
            [] => Ok(None),
            [triple] => Ok(Some((*triple).to_string())),
            [_, _, ..] => Err(prefix.trim_end().to_string()),
        }
    };
    match triple_after(CARGO_TARGET_TOOLCHAIN_LINE_PREFIX)? {
        Some(triple) => Ok(Some(triple)),
        None => triple_after(RUSTC_HOST_TOOLCHAIN_LINE_PREFIX),
    }
}

/// 一个编译单元的生效集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionalCompilationSet {
    names: BTreeSet<String>,
    key_values: BTreeSet<(String, String)>,
    extra_recognized_names: BTreeSet<String>,
    holds_for_every_predicate: bool,
}

/// 一个编译单元除了目标平台之外还带的条件。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompilationConditions {
    pub features: BTreeSet<String>,
    pub is_test: bool,
    pub extra_names: BTreeSet<String>,
    pub extra_key_values: BTreeSet<(String, String)>,
}

impl ConditionalCompilationSet {
    /// 目标平台（`None`：三元组查不出，平台的键一律为假，闭包另记拒算）加上这一个编译单元的特性、`test` 与命令行给的 `--cfg`。
    #[must_use]
    pub fn for_target(
        platform: Option<&TargetPlatform>,
        conditions: &CompilationConditions,
    ) -> Self {
        let mut names: BTreeSet<String> = PROFILE_NAMES
            .iter()
            .chain(platform.map_or(&[][..], |known| known.names))
            .map(|name| (*name).to_string())
            .collect();
        if conditions.is_test {
            names.insert("test".to_string());
        }
        names.extend(conditions.extra_names.iter().cloned());
        let mut key_values: BTreeSet<(String, String)> = platform
            .map_or(&[][..], |known| known.key_values)
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();
        for feature in &conditions.features {
            key_values.insert(("feature".to_string(), feature.clone()));
        }
        key_values.extend(conditions.extra_key_values.iter().cloned());
        Self {
            names,
            key_values,
            extra_recognized_names: conditions.extra_names.clone(),
            holds_for_every_predicate: false,
        }
    }

    /// 只认 rustc 自带名字、一个特性都不开、不带目标平台的集合：`dump` 用它判哪些谓词认不出（拒算条数），只看认不认得、不看真假。
    #[must_use]
    pub fn for_recognition_only() -> Self {
        Self::for_target(None, &CompilationConditions::default())
    }

    /// 弄坏开关 `conditional-compilation` 用：每个谓词都当真。
    #[must_use]
    pub fn holding_every_predicate(mut self) -> Self {
        self.holds_for_every_predicate = true;
        self
    }

    /// 集合里的每一项，排好序：无值名字写成名字，键值写成 `键="值"`。进复用键。
    #[must_use]
    pub fn atoms(&self) -> Vec<String> {
        let mut atoms: Vec<String> = self.names.iter().cloned().collect();
        atoms.extend(
            self.key_values
                .iter()
                .map(|(key, value)| format!("{key}=\"{value}\"")),
        );
        atoms.sort();
        atoms
    }

    fn is_recognized_name(&self, name: &str) -> bool {
        RECOGNIZED_NAMES.contains(&name)
            || self.extra_recognized_names.contains(name)
            || self.names.contains(name)
    }

    /// 谓词在这个集合下的真假。
    #[must_use]
    pub fn evaluate(&self, predicate: &Predicate) -> Evaluation {
        if self.holds_for_every_predicate {
            return Evaluation::Holds;
        }
        match predicate {
            Predicate::Name(name) => {
                if self.names.contains(name) {
                    Evaluation::Holds
                } else if self.is_recognized_name(name) {
                    Evaluation::DoesNotHold
                } else {
                    Evaluation::Unknown
                }
            }
            Predicate::KeyValue { key, value } => {
                if self.key_values.contains(&(key.clone(), value.clone())) {
                    Evaluation::Holds
                } else if RECOGNIZED_KEYS.contains(&key.as_str()) {
                    Evaluation::DoesNotHold
                } else {
                    Evaluation::Unknown
                }
            }
            Predicate::All(parts) => {
                let mut has_unknown = false;
                for part in parts {
                    match self.evaluate(part) {
                        Evaluation::DoesNotHold => return Evaluation::DoesNotHold,
                        Evaluation::Unknown => has_unknown = true,
                        Evaluation::Holds => {}
                    }
                }
                if has_unknown {
                    Evaluation::Unknown
                } else {
                    Evaluation::Holds
                }
            }
            Predicate::Any(parts) => {
                let mut has_unknown = false;
                for part in parts {
                    match self.evaluate(part) {
                        Evaluation::Holds => return Evaluation::Holds,
                        Evaluation::Unknown => has_unknown = true,
                        Evaluation::DoesNotHold => {}
                    }
                }
                if has_unknown {
                    Evaluation::Unknown
                } else {
                    Evaluation::DoesNotHold
                }
            }
            Predicate::Not(inner) => match self.evaluate(inner) {
                Evaluation::Holds => Evaluation::DoesNotHold,
                Evaluation::DoesNotHold => Evaluation::Holds,
                Evaluation::Unknown => Evaluation::Unknown,
            },
            Predicate::Unrecognized(_) => Evaluation::Unknown,
        }
    }

    /// 一串谓词（与起来）的真假。
    #[must_use]
    pub fn evaluate_conjunction(&self, predicates: &[Predicate]) -> Evaluation {
        let mut has_unknown = false;
        for predicate in predicates {
            match self.evaluate(predicate) {
                Evaluation::DoesNotHold => return Evaluation::DoesNotHold,
                Evaluation::Unknown => has_unknown = true,
                Evaluation::Holds => {}
            }
        }
        if has_unknown {
            Evaluation::Unknown
        } else {
            Evaluation::Holds
        }
    }
}

/// 谓词写回文字（拒算行的原样片段用）。
#[must_use]
pub fn rendered_predicate(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Name(name) => name.clone(),
        Predicate::KeyValue { key, value } => format!("{key} = \"{value}\""),
        Predicate::All(parts) => format!("all({})", rendered_list(parts)),
        Predicate::Any(parts) => format!("any({})", rendered_list(parts)),
        Predicate::Not(inner) => format!("not({})", rendered_predicate(inner)),
        Predicate::Unrecognized(fragment) => fragment.clone(),
    }
}

fn rendered_list(parts: &[Predicate]) -> String {
    parts
        .iter()
        .map(rendered_predicate)
        .collect::<Vec<String>>()
        .join(", ")
}

fn is_identifier_text(text: &str) -> bool {
    text.bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// 普通串字面量（`"…"`）的内容；不是普通串返回 `None`。谓词里的值不带转义，只去引号。
#[must_use]
pub fn plain_string_content(literal: &str) -> Option<&str> {
    literal
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
}

/// 按最外一层的逗号切开（括号里的逗号不切；末尾多一个逗号不算一段）。
#[must_use]
pub fn split_at_top_level_commas<'tokens>(tokens: &[&'tokens str]) -> Vec<Vec<&'tokens str>> {
    let mut parts = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0usize;
    for token in tokens {
        match *token {
            "(" | "[" | "{" => {
                depth += 1;
                current.push(*token);
            }
            ")" | "]" | "}" => {
                depth = depth.saturating_sub(1);
                current.push(*token);
            }
            "," if depth == 0 => parts.push(std::mem::take(&mut current)),
            _ => current.push(*token),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

/// 从谓词的词法单元解析一个谓词。
#[must_use]
pub fn parse_predicate(tokens: &[&str]) -> Predicate {
    match tokens {
        [name] if is_identifier_text(name) => Predicate::Name((*name).to_string()),
        [key, "=", value] if is_identifier_text(key) => match plain_string_content(value) {
            Some(content) => Predicate::KeyValue {
                key: (*key).to_string(),
                value: content.to_string(),
            },
            None => Predicate::Unrecognized(tokens.join(" ")),
        },
        [combinator, "(", inside @ .., ")"] => {
            let parts: Vec<Predicate> = split_at_top_level_commas(inside)
                .iter()
                .map(|part| parse_predicate(part))
                .collect();
            match *combinator {
                "all" => Predicate::All(parts),
                "any" => Predicate::Any(parts),
                "not" if parts.len() == 1 => {
                    Predicate::Not(Box::new(parts.into_iter().next().expect("判过恰好一段")))
                }
                _ => Predicate::Unrecognized(tokens.join(" ")),
            }
        }
        _ => Predicate::Unrecognized(tokens.join(" ")),
    }
}

/// 一个属性（`#[` 与 `]` 之间的词法单元）带来的条件：`cfg(谓词)` 交那个谓词；
/// `cfg_attr(条件, 属性…)` 里每个 `cfg(谓词)` 交「条件不成立，或谓词成立」；别的属性不带条件。
#[must_use]
pub fn conditions_of_attribute(attribute_tokens: &[&str]) -> Vec<Predicate> {
    match attribute_tokens {
        ["cfg", "(", inside @ .., ")"] => vec![parse_predicate(inside)],
        ["cfg_attr", "(", inside @ .., ")"] => {
            let parts = split_at_top_level_commas(inside);
            let Some((condition_tokens, attributes)) = parts.split_first() else {
                return vec![Predicate::Unrecognized(attribute_tokens.join(" "))];
            };
            let condition = parse_predicate(condition_tokens);
            let mut conditions = Vec::new();
            for attribute in attributes {
                for inner in conditions_of_attribute(attribute) {
                    conditions.push(Predicate::Any(vec![
                        Predicate::Not(Box::new(condition.clone())),
                        inner,
                    ]));
                }
            }
            conditions
        }
        _ => Vec::new(),
    }
}

/// 一个开括号（`(`、`[`、`{`）配的那个闭括号的下标；没配上交 `None`。三种括号一起数深度（词法单元来自编得过的源码，括号本来就配得上）。
fn matching_close_bracket(texts: &[&str], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (position, text) in texts.iter().enumerate().skip(open) {
        match *text {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(position);
                }
            }
            _ => {}
        }
    }
    None
}

/// 一个属性（`#[` 与 `]` 之间的词法单元）里写出来的谓词：`cfg(谓词)` 交那个谓词，`cfg_attr(条件, 属性…)` 交条件，再逐个看后面那几个属性
/// （里面套的 `cfg(…)`、`cfg_attr(…)` 照样交）。别的属性不交。
fn predicates_written_in_an_attribute(attribute: &[&str], predicates: &mut Vec<Predicate>) {
    let [name, "(", ..] = attribute else {
        return;
    };
    let Some(close) = matching_close_bracket(attribute, 1) else {
        predicates.push(Predicate::Unrecognized(attribute.join(" ")));
        return;
    };
    let inside = &attribute[2..close];
    match *name {
        "cfg" => predicates.push(parse_predicate(inside)),
        "cfg_attr" => {
            let parts = split_at_top_level_commas(inside);
            let Some((condition, attributes)) = parts.split_first() else {
                predicates.push(Predicate::Unrecognized(attribute.join(" ")));
                return;
            };
            predicates.push(parse_predicate(condition));
            for inner_attribute in attributes {
                predicates_written_in_an_attribute(inner_attribute, predicates);
            }
        }
        _ => {}
    }
}

/// 一串词法单元（单元自己的那一截，或文档测试的代码）里写出来的条件编译谓词，按出现次序：每个属性 `#[…]` / `#![…]` 里的
/// `cfg(谓词)`、`cfg_attr(条件, 属性…)` 的条件与它属性里套的 `cfg(…)` / `cfg_attr(…)`，以及 `cfg!(谓词)`（括号写成 `(`、`[`、`{` 都认）。
/// 写不成谓词的交 [`Predicate::Unrecognized`]。与 `scripts/gate/lib_function_units.py` 的 `cfg_predicates_in_tokens` 同一口径。
#[must_use]
pub fn condition_predicates_written_in_tokens(source: &str, tokens: &[Token]) -> Vec<Predicate> {
    let texts: Vec<&str> = tokens
        .iter()
        .map(|token| token_text(source, token))
        .collect();
    let text_at = |position: usize| texts.get(position).copied().unwrap_or("");
    let mut predicates = Vec::new();
    let mut position = 0usize;
    while position < texts.len() {
        let attribute_open = if text_at(position) == "#" && text_at(position + 1) == "[" {
            Some(position + 1)
        } else if text_at(position) == "#"
            && text_at(position + 1) == "!"
            && text_at(position + 2) == "["
        {
            Some(position + 2)
        } else {
            None
        };
        if let Some(open) = attribute_open {
            if let Some(close) = matching_close_bracket(&texts, open) {
                predicates_written_in_an_attribute(&texts[open + 1..close], &mut predicates);
            }
        } else if tokens[position].kind == TokenKind::Identifier
            && text_at(position) == "cfg"
            && text_at(position + 1) == "!"
            && matches!(text_at(position + 2), "(" | "[" | "{")
        {
            match matching_close_bracket(&texts, position + 2) {
                Some(close) => predicates.push(parse_predicate(&texts[position + 3..close])),
                None => predicates.push(Predicate::Unrecognized(texts[position..].join(" "))),
            }
        }
        position += 1;
    }
    predicates
}

/// 一个谓词的叶子（无值名字与键值；写不成谓词的原文也算一片叶子），由左到右。
#[must_use]
pub fn leaves_of_the_predicate(predicate: &Predicate) -> Vec<&Predicate> {
    match predicate {
        Predicate::Name(_) | Predicate::KeyValue { .. } | Predicate::Unrecognized(_) => {
            vec![predicate]
        }
        Predicate::All(parts) | Predicate::Any(parts) => {
            parts.iter().flat_map(leaves_of_the_predicate).collect()
        }
        Predicate::Not(inner) => leaves_of_the_predicate(inner),
    }
}

/// 一片叶子写成原子：无值名字写名字，键值写 `键="值"`（与 [`ConditionalCompilationSet::atoms`] 同一种写法）；写不成谓词的交 `None`。
#[must_use]
pub fn atom_of_the_leaf(leaf: &Predicate) -> Option<String> {
    match leaf {
        Predicate::Name(name) => Some(name.clone()),
        Predicate::KeyValue { key, value } => Some(format!("{key}=\"{value}\"")),
        Predicate::All(_) | Predicate::Any(_) | Predicate::Not(_) | Predicate::Unrecognized(_) => {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conditions(text: &str) -> Vec<Predicate> {
        let tokens: Vec<&str> = text.split(' ').collect();
        conditions_of_attribute(&tokens)
    }

    fn host_with_feature(feature: &str) -> ConditionalCompilationSet {
        ConditionalCompilationSet::for_target(
            target_platform_of_triple("x86_64-unknown-linux-gnu"),
            &CompilationConditions {
                features: [feature.to_string()].into_iter().collect(),
                is_test: false,
                extra_names: BTreeSet::new(),
                extra_key_values: BTreeSet::new(),
            },
        )
    }

    fn platform_of(triple: &str) -> ConditionalCompilationSet {
        ConditionalCompilationSet::for_target(
            Some(target_platform_of_triple(triple).expect("测试点名的三元组在表里")),
            &CompilationConditions::default(),
        )
    }

    #[test]
    fn the_target_triple_decides_the_platform_predicates() {
        let linux = platform_of("x86_64-unknown-linux-gnu");
        let macos = platform_of("aarch64-apple-darwin");
        let windows = platform_of("x86_64-pc-windows-msvc");
        let thirty_two_bit = platform_of("i686-unknown-linux-gnu");
        let os_is_linux = &conditions("cfg ( target_os = \"linux\" )")[0];
        let os_is_macos = &conditions("cfg ( target_os = \"macos\" )")[0];
        assert_eq!(linux.evaluate(os_is_linux), Evaluation::Holds);
        assert_eq!(linux.evaluate(os_is_macos), Evaluation::DoesNotHold);
        assert_eq!(macos.evaluate(os_is_macos), Evaluation::Holds);
        assert_eq!(macos.evaluate(os_is_linux), Evaluation::DoesNotHold);
        let unix = &conditions("cfg ( unix )")[0];
        assert_eq!(linux.evaluate(unix), Evaluation::Holds);
        assert_eq!(windows.evaluate(unix), Evaluation::DoesNotHold);
        assert_eq!(
            windows.evaluate(&conditions("cfg ( windows )")[0]),
            Evaluation::Holds
        );
        let pointer_width_64 = &conditions("cfg ( target_pointer_width = \"64\" )")[0];
        assert_eq!(linux.evaluate(pointer_width_64), Evaluation::Holds);
        assert_eq!(
            thirty_two_bit.evaluate(pointer_width_64),
            Evaluation::DoesNotHold
        );
        assert_eq!(
            macos.evaluate(&conditions("cfg ( target_arch = \"aarch64\" )")[0]),
            Evaluation::Holds
        );
    }

    #[test]
    fn an_unknown_triple_is_not_in_the_table_and_the_cargo_target_line_wins_over_the_host_line() {
        assert!(target_platform_of_triple("sparc64-sun-solaris").is_none());
        let host_only = vec![
            "rustc 1.98.0".to_string(),
            "host: aarch64-apple-darwin".to_string(),
        ];
        assert_eq!(
            target_triple_of_toolchain_lines(&host_only),
            Ok(Some("aarch64-apple-darwin".to_string()))
        );
        let with_cargo_target = vec![
            "host: aarch64-apple-darwin".to_string(),
            "target: x86_64-unknown-linux-gnu".to_string(),
        ];
        assert_eq!(
            target_triple_of_toolchain_lines(&with_cargo_target),
            Ok(Some("x86_64-unknown-linux-gnu".to_string()))
        );
        assert_eq!(
            target_triple_of_toolchain_lines(&["rustc 1.98.0".to_string()]),
            Ok(None)
        );
        let two_hosts = vec![
            "host: aarch64-apple-darwin".to_string(),
            "host: x86_64-unknown-linux-gnu".to_string(),
        ];
        assert_eq!(
            target_triple_of_toolchain_lines(&two_hosts),
            Err("host:".to_string())
        );
    }

    #[test]
    fn empty_any_never_holds_and_empty_all_always_holds() {
        let set = ConditionalCompilationSet::for_recognition_only();
        assert_eq!(
            set.evaluate(&conditions("cfg ( any ( ) )")[0]),
            Evaluation::DoesNotHold
        );
        assert_eq!(
            set.evaluate(&conditions("cfg ( all ( ) )")[0]),
            Evaluation::Holds
        );
    }

    #[test]
    fn features_test_and_the_host_platform_decide_the_predicates() {
        let set = host_with_feature("gpu");
        assert_eq!(
            set.evaluate(&conditions("cfg ( feature = \"gpu\" )")[0]),
            Evaluation::Holds
        );
        assert_eq!(
            set.evaluate(&conditions("cfg ( not ( feature = \"verdict-store\" ) )")[0]),
            Evaluation::Holds
        );
        assert_eq!(
            set.evaluate(&conditions("cfg ( test )")[0]),
            Evaluation::DoesNotHold
        );
        assert_eq!(
            set.evaluate(&conditions("cfg ( target_arch = \"aarch64\" )")[0]),
            Evaluation::DoesNotHold
        );
        assert_eq!(
            set.evaluate(&conditions("cfg ( any ( feature = \"gpu\" , test ) )")[0]),
            Evaluation::Holds
        );
    }

    #[test]
    fn an_unknown_name_or_an_unparsable_predicate_is_unknown_not_false() {
        let set = ConditionalCompilationSet::for_recognition_only();
        assert_eq!(
            set.evaluate(&conditions("cfg ( verus_keep_ghost )")[0]),
            Evaluation::Unknown
        );
        assert_eq!(
            set.evaluate(&conditions("cfg ( foo ( bar ) )")[0]),
            Evaluation::Unknown
        );
        assert_eq!(
            set.evaluate(&conditions("cfg ( all ( verus_keep_ghost , test ) )")[0]),
            Evaluation::DoesNotHold,
            "「全部」里有一项确定为假，整条就是假，不知道的那一项不影响"
        );
    }

    #[test]
    fn a_cfg_inside_cfg_attr_only_applies_when_the_condition_holds() {
        let attribute = conditions("cfg_attr ( feature = \"gpu\" , cfg ( test ) )");
        assert_eq!(attribute.len(), 1);
        assert_eq!(
            host_with_feature("gpu").evaluate(&attribute[0]),
            Evaluation::DoesNotHold,
            "开着 gpu 时这一条等于 cfg(test)，非测试构建为假"
        );
        assert_eq!(
            host_with_feature("verdict-store").evaluate(&attribute[0]),
            Evaluation::Holds,
            "没开 gpu 时这一条不起作用"
        );
        assert!(conditions("cfg_attr ( test , allow ( dead_code ) )").is_empty());
    }
}
