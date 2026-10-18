//! `Cargo.toml` 读法：只读编译目标、依赖与特性要用的那几样，TOML 只认这里用到的子集
//! （表头、`[[数组表]]`、点分键、串、布尔、数组、内联表；别的值原样留着不解释）。

use std::collections::BTreeMap;

/// 读出来的一个值。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TomlValue {
    Text(String),
    Boolean(bool),
    Array(Vec<TomlValue>),
    InlineTable(Vec<(String, TomlValue)>),
    /// 数、日期这类本工具不用的值，原文。
    Unparsed(String),
}

/// 一条键值：落在哪张表（数组表的第几次出现另记）、键的各段与值。
#[derive(Clone, Debug)]
pub struct TomlEntry {
    pub table_path: Vec<String>,
    pub table_occurrence: usize,
    pub key_path: Vec<String>,
    pub value: TomlValue,
}

/// `Cargo.toml` 读不了。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestError {
    pub line: usize,
    pub reason: String,
}

struct TomlReader<'text> {
    bytes: &'text [u8],
    position: usize,
}

impl TomlReader<'_> {
    fn line(&self) -> usize {
        1 + self.bytes[..self.position.min(self.bytes.len())]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count()
    }

    fn failure(&self, reason: &str) -> ManifestError {
        ManifestError {
            line: self.line(),
            reason: reason.to_string(),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn skip_blanks_on_the_line(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t')) {
            self.position += 1;
        }
    }

    fn skip_blanks_newlines_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\r' | b'\n') => self.position += 1,
                Some(b'#') => {
                    while self.peek().is_some_and(|byte| byte != b'\n') {
                        self.position += 1;
                    }
                }
                _ => return,
            }
        }
    }

    fn read_key_segment(&mut self) -> Result<String, ManifestError> {
        self.skip_blanks_on_the_line();
        match self.peek() {
            Some(b'"') => self.read_basic_string(),
            Some(b'\'') => self.read_literal_string(),
            _ => {
                let start = self.position;
                while self.peek().is_some_and(|byte| {
                    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
                }) {
                    self.position += 1;
                }
                if start == self.position {
                    return Err(self.failure("这里要一个键"));
                }
                Ok(String::from_utf8_lossy(&self.bytes[start..self.position]).into_owned())
            }
        }
    }

    fn read_key_path(&mut self) -> Result<Vec<String>, ManifestError> {
        let mut segments = vec![self.read_key_segment()?];
        loop {
            self.skip_blanks_on_the_line();
            if self.peek() == Some(b'.') {
                self.position += 1;
                segments.push(self.read_key_segment()?);
            } else {
                return Ok(segments);
            }
        }
    }

    fn read_basic_string(&mut self) -> Result<String, ManifestError> {
        // 三引号写成常量：变异表的锚点里不许有反斜杠转义（`.claude/rules/mutation-sampling.md` 第八类），锚到这几行时引它的名字
        const MULTI_LINE_BASIC_STRING_DELIMITER: &[u8] = b"\"\"\"";
        if self.bytes[self.position..].starts_with(MULTI_LINE_BASIC_STRING_DELIMITER) {
            self.position += 3;
            let start = self.position;
            while self.position < self.bytes.len()
                && !self.bytes[self.position..].starts_with(MULTI_LINE_BASIC_STRING_DELIMITER)
            {
                self.position += 1;
            }
            if !self.bytes[self.position..].starts_with(MULTI_LINE_BASIC_STRING_DELIMITER) {
                return Err(self.failure("串没闭上"));
            }
            let content = String::from_utf8_lossy(&self.bytes[start..self.position]).into_owned();
            self.position = (self.position + 3).min(self.bytes.len());
            return Ok(content);
        }
        self.position += 1;
        let mut content = Vec::new();
        loop {
            match self.peek() {
                None | Some(b'\n') => return Err(self.failure("串没闭上")),
                Some(b'"') => {
                    self.position += 1;
                    return Ok(String::from_utf8_lossy(&content).into_owned());
                }
                Some(b'\\') => {
                    // TOML 基础串只认这几种转义；认不出的转义是语法错，不把转义字符原样收下（`\r` 收成字母 r 那种）
                    let escaped = self.bytes.get(self.position + 1).copied();
                    match escaped {
                        Some(b'n') => content.push(b'\n'),
                        Some(b't') => content.push(b'\t'),
                        Some(b'r') => content.push(b'\r'),
                        Some(b'b') => content.push(0x08),
                        Some(b'f') => content.push(0x0C),
                        Some(b'"') => content.push(b'"'),
                        Some(b'\\') => content.push(b'\\'),
                        Some(b'u' | b'U') => {
                            let digit_count = if escaped == Some(b'u') { 4 } else { 8 };
                            let digits_start = self.position + 2;
                            let digits_end = digits_start + digit_count;
                            let digits = self
                                .bytes
                                .get(digits_start..digits_end)
                                .ok_or_else(|| self.failure("串没闭上"))?;
                            let scalar = std::str::from_utf8(digits)
                                .ok()
                                .and_then(|text| u32::from_str_radix(text, 16).ok())
                                .and_then(char::from_u32)
                                .ok_or_else(|| self.failure("Unicode 转义不是一个标量值"))?;
                            let mut encoded = [0u8; 4];
                            content.extend_from_slice(scalar.encode_utf8(&mut encoded).as_bytes());
                            self.position = digits_end;
                            continue;
                        }
                        Some(_) => return Err(self.failure("认不出的转义")),
                        None => return Err(self.failure("串没闭上")),
                    }
                    self.position += 2;
                }
                Some(byte) => {
                    content.push(byte);
                    self.position += 1;
                }
            }
        }
    }

    fn read_literal_string(&mut self) -> Result<String, ManifestError> {
        self.position += 1;
        let start = self.position;
        while self
            .peek()
            .is_some_and(|byte| byte != b'\'' && byte != b'\n')
        {
            self.position += 1;
        }
        if self.peek() != Some(b'\'') {
            return Err(self.failure("串没闭上"));
        }
        let content = String::from_utf8_lossy(&self.bytes[start..self.position]).into_owned();
        self.position += 1;
        Ok(content)
    }

    fn read_value(&mut self) -> Result<TomlValue, ManifestError> {
        self.skip_blanks_on_the_line();
        match self.peek() {
            Some(b'"') => Ok(TomlValue::Text(self.read_basic_string()?)),
            Some(b'\'') => Ok(TomlValue::Text(self.read_literal_string()?)),
            Some(b'[') => {
                self.position += 1;
                let mut elements = Vec::new();
                loop {
                    self.skip_blanks_newlines_and_comments();
                    match self.peek() {
                        Some(b']') => {
                            self.position += 1;
                            return Ok(TomlValue::Array(elements));
                        }
                        Some(b',') => self.position += 1,
                        None => return Err(self.failure("数组没闭上")),
                        Some(_) => elements.push(self.read_value()?),
                    }
                }
            }
            Some(b'{') => {
                self.position += 1;
                let mut pairs = Vec::new();
                loop {
                    self.skip_blanks_newlines_and_comments();
                    match self.peek() {
                        Some(b'}') => {
                            self.position += 1;
                            return Ok(TomlValue::InlineTable(pairs));
                        }
                        Some(b',') => self.position += 1,
                        None => return Err(self.failure("内联表没闭上")),
                        Some(_) => {
                            let key = self.read_key_path()?.join(".");
                            self.skip_blanks_on_the_line();
                            if self.peek() != Some(b'=') {
                                return Err(self.failure("内联表的键后面要 ="));
                            }
                            self.position += 1;
                            let value = self.read_value()?;
                            pairs.push((key, value));
                        }
                    }
                }
            }
            _ => {
                let start = self.position;
                while self
                    .peek()
                    .is_some_and(|byte| !matches!(byte, b'\n' | b',' | b']' | b'}' | b'#'))
                {
                    self.position += 1;
                }
                let raw = String::from_utf8_lossy(&self.bytes[start..self.position])
                    .trim()
                    .to_string();
                match raw.as_str() {
                    "true" => Ok(TomlValue::Boolean(true)),
                    "false" => Ok(TomlValue::Boolean(false)),
                    "" => Err(self.failure("这里要一个值")),
                    _ => Ok(TomlValue::Unparsed(raw)),
                }
            }
        }
    }
}

/// 把整份 `Cargo.toml` 读成一串键值。
///
/// # Errors
/// 表头、键、串、数组或内联表写坏了，交出错的行号与原因。
pub fn parse_toml_entries(text: &str) -> Result<Vec<TomlEntry>, ManifestError> {
    let mut reader = TomlReader {
        bytes: text.as_bytes(),
        position: 0,
    };
    let mut entries = Vec::new();
    let mut table_path: Vec<String> = Vec::new();
    let mut table_occurrence = 0usize;
    loop {
        reader.skip_blanks_newlines_and_comments();
        let Some(byte) = reader.peek() else {
            return Ok(entries);
        };
        if byte == b'[' {
            let is_array_table = reader.bytes.get(reader.position + 1) == Some(&b'[');
            reader.position += if is_array_table { 2 } else { 1 };
            table_path = reader.read_key_path()?;
            reader.skip_blanks_on_the_line();
            let closing: &[u8] = if is_array_table { b"]]" } else { b"]" };
            if !reader.bytes[reader.position..].starts_with(closing) {
                return Err(reader.failure("表头没闭上"));
            }
            reader.position += closing.len();
            table_occurrence += 1;
            continue;
        }
        let key_path = reader.read_key_path()?;
        reader.skip_blanks_on_the_line();
        if reader.peek() != Some(b'=') {
            return Err(reader.failure("键后面要 ="));
        }
        reader.position += 1;
        let value = reader.read_value()?;
        entries.push(TomlEntry {
            table_path: table_path.clone(),
            table_occurrence,
            key_path,
            value,
        });
    }
}

/// 依赖写在哪一节。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DependencySection {
    Normal,
    Development,
    Build,
}

/// 一条依赖：只有带 `path` 的落到工作区里的 crate，其余是外部依赖（解析结果由调用方以 `Cargo.lock` 的行拼进键）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub section: DependencySection,
    pub key: String,
    /// 改名依赖（`foo = { package = "bar", … }`）真正的包名：代码里照键 `foo` 点它，`Cargo.lock` 里照包名 `bar` 找它；没改名是 `None`。
    pub package: Option<String>,
    pub path: Option<String>,
    pub features: Vec<String>,
    pub uses_default_features: bool,
    pub is_optional: bool,
}

/// 清单里显式声明的一个编译目标（`[lib]`、`[[bin]]`、`[[test]]`、`[[example]]`、`[[bench]]`）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeclaredTarget {
    pub name: Option<String>,
    pub path: Option<String>,
}

/// 构建脚本怎么定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildScriptDeclaration {
    /// 没写：有 `build.rs` 就是它。
    Discovered,
    Path(String),
    Disabled,
}

/// 一份 `Cargo.toml` 里本工具用到的那几样。
#[derive(Clone, Debug)]
pub struct CrateManifest {
    pub package_name: String,
    pub library: Option<DeclaredTarget>,
    pub is_procedure_macro: bool,
    pub binaries: Vec<DeclaredTarget>,
    pub integration_tests: Vec<DeclaredTarget>,
    pub examples: Vec<DeclaredTarget>,
    pub benchmarks: Vec<DeclaredTarget>,
    pub build_script: BuildScriptDeclaration,
    pub discovers_binaries: bool,
    pub discovers_integration_tests: bool,
    pub discovers_examples: bool,
    pub discovers_benchmarks: bool,
    pub dependencies: Vec<Dependency>,
    pub features: BTreeMap<String, Vec<String>>,
}

fn text_of(value: &TomlValue) -> Option<String> {
    match value {
        TomlValue::Text(text) => Some(text.clone()),
        TomlValue::Boolean(_)
        | TomlValue::Array(_)
        | TomlValue::InlineTable(_)
        | TomlValue::Unparsed(_) => None,
    }
}

fn boolean_of(value: &TomlValue) -> Option<bool> {
    match value {
        TomlValue::Boolean(flag) => Some(*flag),
        TomlValue::Text(_)
        | TomlValue::Array(_)
        | TomlValue::InlineTable(_)
        | TomlValue::Unparsed(_) => None,
    }
}

fn texts_of(value: &TomlValue) -> Vec<String> {
    match value {
        TomlValue::Array(elements) => elements.iter().filter_map(text_of).collect(),
        TomlValue::Text(_)
        | TomlValue::Boolean(_)
        | TomlValue::InlineTable(_)
        | TomlValue::Unparsed(_) => Vec::new(),
    }
}

fn dependency_section_of(table_path: &[String]) -> Option<DependencySection> {
    let section_name = match table_path {
        [name, ..] if name != "target" => name.as_str(),
        [_target, _platform, name, ..] => name.as_str(),
        _ => return None,
    };
    match section_name {
        "dependencies" => Some(DependencySection::Normal),
        "dev-dependencies" | "dev_dependencies" => Some(DependencySection::Development),
        "build-dependencies" | "build_dependencies" => Some(DependencySection::Build),
        _ => None,
    }
}

/// 依赖表头之后那一段（`[dependencies.名字]` 的 `名字`）：表头本身就是依赖节时为空。
fn dependency_table_suffix(table_path: &[String]) -> &[String] {
    match table_path {
        [name, rest @ ..] if name != "target" => rest,
        [_target, _platform, _name, rest @ ..] => rest,
        _ => &[],
    }
}

fn apply_dependency_field(dependency: &mut Dependency, field: &str, value: &TomlValue) {
    match field {
        "path" => dependency.path = text_of(value),
        "package" => dependency.package = text_of(value),
        "features" => dependency.features = texts_of(value),
        "default-features" | "default_features" => {
            dependency.uses_default_features = boolean_of(value).unwrap_or(true);
        }
        "optional" => dependency.is_optional = boolean_of(value).unwrap_or(false),
        _ => {}
    }
}

/// 读一份 `Cargo.toml`。
///
/// # Errors
/// TOML 读不了，或没有 `[package] name`。
pub fn parse_crate_manifest(text: &str) -> Result<CrateManifest, ManifestError> {
    let entries = parse_toml_entries(text)?;
    let mut package_name = None;
    let mut library: Option<DeclaredTarget> = None;
    let mut is_procedure_macro = false;
    let mut declared_by_occurrence: BTreeMap<(String, usize), DeclaredTarget> = BTreeMap::new();
    let mut build_script = BuildScriptDeclaration::Discovered;
    let mut discovers = BTreeMap::new();
    let mut dependencies: BTreeMap<(DependencySection, String), Dependency> = BTreeMap::new();
    let mut features = BTreeMap::new();
    for entry in &entries {
        let table: Vec<&str> = entry.table_path.iter().map(String::as_str).collect();
        let key: Vec<&str> = entry.key_path.iter().map(String::as_str).collect();
        match (table.as_slice(), key.as_slice()) {
            (["package"], ["name"]) => package_name = text_of(&entry.value),
            (["package"], ["build"]) => {
                build_script = match (&entry.value, text_of(&entry.value)) {
                    (TomlValue::Boolean(false), _) => BuildScriptDeclaration::Disabled,
                    (_, Some(path)) => BuildScriptDeclaration::Path(path),
                    (_, None) => BuildScriptDeclaration::Discovered,
                };
            }
            (["package"], [flag @ ("autobins" | "autotests" | "autoexamples" | "autobenches")]) => {
                discovers.insert(
                    (*flag).to_string(),
                    boolean_of(&entry.value).unwrap_or(true),
                );
            }
            (["lib"], [field]) => {
                let declared = library.get_or_insert_with(DeclaredTarget::default);
                match *field {
                    "name" => declared.name = text_of(&entry.value),
                    "path" => declared.path = text_of(&entry.value),
                    "proc-macro" | "proc_macro" => {
                        is_procedure_macro = boolean_of(&entry.value).unwrap_or(false);
                    }
                    // Cargo 的另一种写法：`crate-type = ["proc-macro"]`
                    "crate-type" | "crate_type"
                        if texts_of(&entry.value)
                            .iter()
                            .any(|kind| kind == "proc-macro") =>
                    {
                        is_procedure_macro = true;
                    }
                    _ => {}
                }
            }
            ([section @ ("bin" | "test" | "example" | "bench")], [field @ ("name" | "path")]) => {
                let declared = declared_by_occurrence
                    .entry(((*section).to_string(), entry.table_occurrence))
                    .or_default();
                if *field == "name" {
                    declared.name = text_of(&entry.value);
                } else {
                    declared.path = text_of(&entry.value);
                }
            }
            (["features"], [feature]) => {
                features.insert((*feature).to_string(), texts_of(&entry.value));
            }
            _ => {
                let Some(section) = dependency_section_of(&entry.table_path) else {
                    continue;
                };
                let suffix = dependency_table_suffix(&entry.table_path);
                let (dependency_key, field) = match (suffix, entry.key_path.as_slice()) {
                    ([name], [field]) => (name.clone(), Some(field.clone())),
                    ([], [name]) => (name.clone(), None),
                    ([], [name, field]) => (name.clone(), Some(field.clone())),
                    _ => continue,
                };
                let dependency = dependencies
                    .entry((section, dependency_key.clone()))
                    .or_insert_with(|| Dependency {
                        section,
                        key: dependency_key.clone(),
                        package: None,
                        path: None,
                        features: Vec::new(),
                        uses_default_features: true,
                        is_optional: false,
                    });
                match field {
                    Some(field) => apply_dependency_field(dependency, &field, &entry.value),
                    None => {
                        if let TomlValue::InlineTable(pairs) = &entry.value {
                            for (inner_field, inner_value) in pairs {
                                apply_dependency_field(dependency, inner_field, inner_value);
                            }
                        }
                    }
                }
            }
        }
    }
    let package_name = package_name.ok_or(ManifestError {
        line: 1,
        reason: "没有 [package] name".to_string(),
    })?;
    let declared_of = |section: &str| -> Vec<DeclaredTarget> {
        declared_by_occurrence
            .iter()
            .filter(|((declared_section, _), _)| declared_section == section)
            .map(|(_, declared)| declared.clone())
            .collect()
    };
    let discovers_of = |flag: &str| discovers.get(flag).copied().unwrap_or(true);
    Ok(CrateManifest {
        package_name,
        library,
        is_procedure_macro,
        binaries: declared_of("bin"),
        integration_tests: declared_of("test"),
        examples: declared_of("example"),
        benchmarks: declared_of("bench"),
        build_script,
        discovers_binaries: discovers_of("autobins"),
        discovers_integration_tests: discovers_of("autotests"),
        discovers_examples: discovers_of("autoexamples"),
        discovers_benchmarks: discovers_of("autobenches"),
        dependencies: dependencies.into_values().collect(),
        features,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `[lib] crate-type = ["proc-macro"]` 与 `proc-macro = true` 同义：都认成过程宏 crate。
    #[test]
    fn a_library_declaring_the_proc_macro_crate_type_is_a_procedure_macro_crate() {
        let manifest = parse_crate_manifest(
            "[package]\nname = \"demo-macros\"\n\n[lib]\ncrate-type = [\"proc-macro\"]\n",
        )
        .expect("清单读得了");
        assert!(manifest.is_procedure_macro);
    }

    /// 基础串的转义按 TOML 认：`\b`、`\r`、`\"`、`\\`、`é` 各还原成它指的字符，不把 `\r` 收成字母 r；认不出的转义（`\q`）拒。
    #[test]
    fn basic_string_escapes_are_decoded_and_an_unknown_escape_is_refused() {
        let manifest = parse_crate_manifest("[package]\nname = \"\\ba\\rb\\\"c\\\\d\\u00e9\"\n")
            .expect("清单读得了");
        assert_eq!(manifest.package_name, "\u{8}a\rb\"c\\dé");
        assert!(
            parse_crate_manifest("[package]\nname = \"a\\qb\"\n").is_err(),
            "认不出的转义要拒"
        );
    }

    /// 三引号串没闭上读到文件尾：报「串没闭上」，与单引号串那一支同一个判法，不静默收下。
    #[test]
    fn an_unterminated_multi_line_basic_string_is_refused() {
        assert!(
            parse_crate_manifest("[package]\nname = \"\"\"abc\n").is_err(),
            "没闭上的三引号串要拒"
        );
    }

    #[test]
    fn dependencies_features_and_declared_targets_are_read() {
        let manifest = parse_crate_manifest(
            r#"
# 注释
[package]
name = "demo-crate"
edition = "2021"

[lib]
proc-macro = false

[dependencies]
local = { path = "../local", features = ["wide"], default-features = false }
external = "1.0"
optional-local = { path = "../optional", optional = true }

[dev-dependencies.helper]
path = "../helper"

[features]
default = ["wide"]
wide = [
    "local/wide", # 跨行的数组
    "dep:optional-local",
]

[[bin]]
name = "first-tool"
path = "src/bin/first_tool.rs"

[[bin]]
name = "second-tool"
"#,
        )
        .expect("清单读得了");
        assert_eq!(manifest.package_name, "demo-crate");
        assert!(!manifest.is_procedure_macro);
        assert_eq!(manifest.binaries.len(), 2, "两个 [[bin]] 各一个");
        assert_eq!(
            manifest.binaries[0].path.as_deref(),
            Some("src/bin/first_tool.rs")
        );
        assert_eq!(
            manifest.features.get("wide").cloned(),
            Some(vec![
                "local/wide".to_string(),
                "dep:optional-local".to_string()
            ])
        );
        let local = manifest
            .dependencies
            .iter()
            .find(|dependency| dependency.key == "local")
            .expect("local 那条依赖在");
        assert_eq!(local.path.as_deref(), Some("../local"));
        assert!(!local.uses_default_features);
        assert_eq!(local.features, ["wide"]);
        let helper = manifest
            .dependencies
            .iter()
            .find(|dependency| dependency.key == "helper")
            .expect("点分表写法的开发依赖在");
        assert_eq!(helper.section, DependencySection::Development);
        assert_eq!(helper.path.as_deref(), Some("../helper"));
        let optional = manifest
            .dependencies
            .iter()
            .find(|dependency| dependency.key == "optional-local")
            .expect("可选依赖在");
        assert!(optional.is_optional);
    }
}
