//! Rust 源码切成单元（设计 3.2）：每份文件一份词法，按条目起头切；每个单元取去注释指纹、收它里面的名字、读文件的写法与拒算。
//!
//! 单元：自由 fn（函数体里的嵌套条目算它的）；impl 头（`impl … {` 之前，连属性）与它的每个方法、关联 const、关联 type；
//! trait 头与它的每个成员；const；static；struct / enum / union；type 别名；`macro_rules!` 整个定义；每条 use；
//! 条目级宏调用（带路径的 `a::b::m! { … }` 与 `m! { … }` 同一种）；`mod x;`（连 `#[path]`）；行内模块头与文件开头的内属性；
//! extern 块；extern crate。`thread_local!` 展开成里面的 static 再切；`verus!` 里的条目按「从这个条目起头到下一个同层条目起头之前」
//! 整段一个单元（Verus 的 `requires` / `ensures` 可以带花括号，「签名之后第一个 `{` 是函数体」不成立）；其中
//! `assume_specification[ 路径 ] …;`（带不带可见性、属性、泛型参数表、参数表、`returns` 都算）是一个 `assume_specification` 单元，
//! 名字写 `assume_specification@<关键字那一行>`，按方括号里路径末段的名字找得到它。
//! 文档注释不进条目指纹；文档注释里的 ```` ``` ```` Rust 代码块另成文档测试单元，挂在所属条目（或模块）下。

use std::collections::{BTreeSet, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

use crate::conditional_compilation::{
    condition_predicates_written_in_tokens, conditions_of_attribute, plain_string_content,
    split_at_top_level_commas, Predicate,
};
use crate::lexer::{
    identifier_name, lex_source, matching_brackets, token_text, DocumentationComment,
    DocumentationCommentTreatment, DocumentationPlacement, Token, TokenKind,
};
use crate::rule_switches::{BrokenRule, RuleSwitches};
use crate::sha256::{sha256_of, Sha256Digest, Sha256Hasher};
use crate::units::{
    directory_digest, ChildTestName, DocumentationTestAttachment, FileIndex, ImplementationShape,
    ModuleDeclarationShape, ReadOrigin, Refusal, RefusalForm, SourceRead, TestNamePiece, Unit,
    UnitIndex, UnitKind, UnitReferences, UseImports,
};

/// 词法单元之间的分隔字节：单元串按它连起来取 SHA-256。
pub const UNIT_SEPARATOR: u8 = 0x1f;

/// 拒算行的原样片段最多取几个词法单元。
const FRAGMENT_TOKEN_LIMIT: usize = 12;
/// 拒算行的原样片段最多几个字符。
const FRAGMENT_CHARACTER_LIMIT: usize = 160;

/// 找 impl 自身类型路径的头一段时跳过的词（与 `scripts/gate/lib_function_units.py` 的 `SELF_TYPE_SKIPPED` 同一份）。
const WORDS_BEFORE_THE_SELF_TYPE_PATH_ROOT: [&str; 8] = [
    "dyn", "mut", "const", "impl", "for", "where", "as", "unsafe",
];

/// `env!` / `option_env!` 读的变量里 cargo 自己给的那一类的前缀（`CARGO_MANIFEST_DIR`、`CARGO_BIN_EXE_<名>`、`CARGO_PKG_*` 这类）。
const CARGO_SUPPLIED_ENVIRONMENT_VARIABLE_PREFIX: &str = "CARGO";
/// cargo 给构建脚本输出目录的变量。
const BUILD_SCRIPT_OUTPUT_DIRECTORY_VARIABLE: &str = "OUT_DIR";

/// 清单目录拼出的目录绑到名字、名字的用法再绑到名字，最多跟这么多层（再深的拒算）。
const DEEPEST_DIRECTORY_BINDING_CHAIN: usize = 4;

/// 单元里出现这几个词算起了子进程：此时 `current_exe()` 折不出要点名的那一条测试就拒算 `current-exe`；一个都没有的只是在问路径，
/// 不拒算（与 `scripts/gate/lib_function_units.py` 的 `SPAWNING_WORDS` 同一份）。
const SPAWNING_WORDS: [&str; 7] = [
    "Command",
    "spawn",
    "posix_spawn",
    "fork",
    "execv",
    "execve",
    "execvp",
];

const RUST_KEYWORDS: [&str; 39] = [
    "as",
    "async",
    "await",
    "break",
    "const",
    "continue",
    "crate",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "macro_rules",
];

fn keyword_set() -> &'static HashSet<&'static str> {
    static KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    KEYWORDS.get_or_init(|| RUST_KEYWORDS.iter().copied().collect())
}

/// 是不是关键字（含 Verus 的上下文关键字）：关键字不当名字解。
#[must_use]
pub fn is_keyword(name: &str) -> bool {
    keyword_set().contains(name)
}

/// 一份 Rust 文件切分要的输入。
pub struct RustFileInput<'input> {
    pub file: FileIndex,
    pub source: &'input str,
    pub file_directory: &'input Path,
    pub manifest_directory: Option<&'input Path>,
    pub switches: &'input RuleSwitches,
}

/// 一份 Rust 文件切出来的东西。单元的 `UnitIndex` 是文件内的下标，工作区合并时加上偏移。
pub struct ParsedRustFile {
    pub tokens: Vec<Token>,
    pub units: Vec<Unit>,
    /// 切分本身做不下去的拒算（括号配不上、认不出的条目起头）：这份文件里定义了哪些名字说不全，链接到它的闭包一律拒算。
    pub file_level_refusals: Vec<Refusal>,
}

/// 词法单元串的 SHA-256：单元之间用 0x1f 连起来。
#[must_use]
pub fn fingerprint_of_tokens(source: &str, tokens: &[Token]) -> Sha256Digest {
    let mut hasher = Sha256Hasher::with_no_input();
    for (position, token) in tokens.iter().enumerate() {
        if position > 0 {
            hasher.absorb(&[UNIT_SEPARATOR]);
        }
        hasher.absorb(token_text(source, token).as_bytes());
    }
    hasher.finish()
}

/// 路径按字面规整：去掉 `.`，`..` 退一级（不碰文件系统、不跟符号链接）。
#[must_use]
pub fn normalized_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

/// 一个串字面量里 `{名字}`、`{名字:…}` 形式的格式捕获：`format!("{DATA_POINTER_BYTES}")` 这样用到的常量也算点了它的名。
fn add_format_captures<'source>(literal: &'source str, names: &mut HashSet<&'source str>) {
    if !(literal.starts_with('"') || literal.starts_with("r\"") || literal.starts_with("r#"))
        || !literal.contains('{')
    {
        return;
    }
    let bytes = literal.as_bytes();
    let mut position = 0usize;
    while position < bytes.len() {
        if bytes[position] != b'{' {
            position += 1;
            continue;
        }
        if bytes.get(position + 1) == Some(&b'{') {
            position += 2;
            continue;
        }
        let start = position + 1;
        let mut end = start;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            end += 1;
        }
        let is_name = end > start && !bytes[start].is_ascii_digit();
        if is_name && matches!(bytes.get(end), Some(b'}' | b':')) {
            names.insert(&literal[start..end]);
        }
        position = end.max(position + 1);
    }
}

fn sorted_strings(names: HashSet<&str>) -> Vec<String> {
    let mut sorted: Vec<String> = names.into_iter().map(str::to_string).collect();
    sorted.sort();
    sorted
}

/// 一段词法单元里的名字，按 3.3 的规则分好类。`signature_end` 是函数体那个 `{` 在这一段里的位置（函数单元才有）。
#[must_use]
pub fn references_of_tokens(
    source: &str,
    tokens: &[Token],
    signature_end: Option<usize>,
) -> UnitReferences {
    let text_at = |position: usize| {
        tokens
            .get(position)
            .map_or("", |token| token_text(source, token))
    };
    let mut plain_names: HashSet<&str> = HashSet::new();
    let mut method_names: HashSet<&str> = HashSet::new();
    let mut qualified_paths: BTreeSet<(String, String)> = BTreeSet::new();
    let mut dyn_trait_names: HashSet<&str> = HashSet::new();
    for (position, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::Literal => {
                add_format_captures(token_text(source, token), &mut plain_names);
                continue;
            }
            TokenKind::Lifetime | TokenKind::Punctuation => continue,
            TokenKind::Identifier => {}
        }
        let name = identifier_name(token_text(source, token));
        let previous = if position > 0 {
            text_at(position - 1)
        } else {
            ""
        };
        let is_after_single_dot =
            previous == "." && !(position > 1 && text_at(position - 2) == ".");
        let next = text_at(position + 1);
        if is_after_single_dot {
            if next == "(" || (next == ":" && text_at(position + 2) == ":") {
                method_names.insert(name);
            }
            continue;
        }
        if previous == ":"
            && position >= 3
            && text_at(position - 2) == ":"
            && tokens[position - 3].kind == TokenKind::Identifier
        {
            qualified_paths.insert((
                identifier_name(text_at(position - 3)).to_string(),
                name.to_string(),
            ));
        }
        if name == "dyn" {
            let mut path_position = position + 1;
            let mut last_segment = None;
            while tokens
                .get(path_position)
                .is_some_and(|candidate| candidate.kind == TokenKind::Identifier)
            {
                last_segment = Some(identifier_name(text_at(path_position)));
                if text_at(path_position + 1) == ":" && text_at(path_position + 2) == ":" {
                    path_position += 3;
                } else {
                    break;
                }
            }
            if let Some(trait_name) = last_segment {
                dyn_trait_names.insert(trait_name);
            }
        }
        if !is_keyword(name) {
            plain_names.insert(name);
        }
    }
    let mut signature_trait_names: HashSet<&str> = HashSet::new();
    if let Some(end) = signature_end {
        for position in 0..end.min(tokens.len()) {
            if matches!(text_at(position), "impl" | "dyn" | ":" | "+")
                && tokens
                    .get(position + 1)
                    .is_some_and(|candidate| candidate.kind == TokenKind::Identifier)
            {
                let candidate = identifier_name(text_at(position + 1));
                if !is_keyword(candidate) {
                    signature_trait_names.insert(candidate);
                }
            }
        }
    }
    UnitReferences {
        plain_names: sorted_strings(plain_names),
        method_names: sorted_strings(method_names),
        qualified_paths: qualified_paths.into_iter().collect(),
        dyn_trait_names: sorted_strings(dyn_trait_names),
        signature_trait_names: sorted_strings(signature_trait_names),
    }
}

/// 文档注释里的一个 Rust 代码块。
struct DocumentationCodeBlock {
    code: String,
    opening_line: u32,
    closing_line: u32,
}

const RUST_CODE_BLOCK_WORDS: [&str; 9] = [
    "rust",
    "ignore",
    "should_panic",
    "no_run",
    "compile_fail",
    "test_harness",
    "allow_fail",
    "standalone_crate",
    "custom",
];

/// 代码块的信息串是不是 rustdoc 当 Rust 文档测试编的那一种（空的也是）。
fn is_rust_code_block_information(information: &str) -> bool {
    information
        .split([',', ' ', '\t'])
        .filter(|word| !word.is_empty())
        .all(|word| {
            RUST_CODE_BLOCK_WORDS.contains(&word)
                || word.starts_with("edition")
                || word.starts_with("ignore-")
                || (word.len() > 1
                    && word
                        .strip_prefix('E')
                        .is_some_and(|digits| digits.bytes().all(|byte| byte.is_ascii_digit())))
        })
}

/// 行首围栏：三个以上的 `` ` `` 或 `~`，交（围栏字符，个数）。
fn fence_of(trimmed_line: &str) -> Option<(u8, usize)> {
    let first = *trimmed_line.as_bytes().first()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let count = trimmed_line
        .bytes()
        .take_while(|byte| *byte == first)
        .count();
    (count >= 3).then_some((first, count))
}

/// rustdoc 的隐藏行：`# 代码` 照编、不显示；`##` 开头的是一个字面的 `#`。
fn without_hidden_line_marker(line: &str) -> &str {
    let trimmed = line.trim_start();
    if trimmed == "#" {
        return "";
    }
    if let Some(rest) = trimmed.strip_prefix("# ") {
        return rest;
    }
    if trimmed.starts_with("##") {
        return &trimmed[1..];
    }
    line
}

struct OpenCodeBlock {
    fence_character: u8,
    fence_length: usize,
    opening_line: u32,
    lines: Vec<String>,
    is_rust: bool,
}

fn rust_code_blocks(comments: &[&DocumentationComment]) -> Vec<DocumentationCodeBlock> {
    let mut lines: Vec<(u32, &str)> = Vec::new();
    for comment in comments {
        for (offset, body_line) in comment.body_lines.iter().enumerate() {
            let line_number =
                comment.first_line + u32::try_from(offset).expect("一段文档注释的行数放得进 u32");
            lines.push((line_number, body_line.as_str()));
        }
    }
    let mut blocks = Vec::new();
    let mut open_block: Option<OpenCodeBlock> = None;
    for (line_number, raw_line) in lines {
        let trimmed = raw_line.trim_start();
        match open_block.as_mut() {
            None => {
                if let Some((fence_character, fence_length)) = fence_of(trimmed) {
                    let information = trimmed[fence_length..].trim();
                    open_block = Some(OpenCodeBlock {
                        fence_character,
                        fence_length,
                        opening_line: line_number,
                        lines: Vec::new(),
                        is_rust: is_rust_code_block_information(information),
                    });
                }
            }
            Some(block) => {
                let is_closing = fence_of(trimmed).is_some_and(|(character, length)| {
                    character == block.fence_character
                        && length >= block.fence_length
                        && trimmed[length..].trim().is_empty()
                });
                if is_closing {
                    let finished = open_block.take().expect("这一臂里 open_block 是 Some");
                    if finished.is_rust {
                        blocks.push(DocumentationCodeBlock {
                            code: finished.lines.join("\n"),
                            opening_line: finished.opening_line,
                            closing_line: line_number,
                        });
                    }
                } else {
                    block
                        .lines
                        .push(without_hidden_line_marker(raw_line).to_string());
                }
            }
        }
    }
    if let Some(unfinished) = open_block {
        if unfinished.is_rust {
            let closing_line = unfinished.opening_line
                + u32::try_from(unfinished.lines.len()).expect("代码块行数放得进 u32");
            blocks.push(DocumentationCodeBlock {
                code: unfinished.lines.join("\n"),
                opening_line: unfinished.opening_line,
                closing_line,
            });
        }
    }
    blocks
}

/// 切分时一层一层往里带的上下文。
#[derive(Clone)]
struct ScanContext {
    module_path: Vec<String>,
    container: Option<UnitIndex>,
    container_label: Option<String>,
    module_head: Option<UnitIndex>,
    conditions: Vec<Predicate>,
}

/// 属性里本工具要的几样。
#[derive(Default)]
struct AttributeFacts {
    conditions: Vec<Predicate>,
    path_attribute: Option<String>,
    is_macro_export: bool,
}

/// 新单元的形状（切分时现填，指纹、名字与读文件由 `push_unit` 现算）。
struct NewUnit {
    kind: UnitKind,
    name: String,
    first_token: usize,
    end_token: usize,
    header_token: usize,
    conditions: Vec<Predicate>,
    is_public: bool,
    declared_names: Option<Vec<String>>,
    imports: Option<UseImports>,
    implementation: Option<ImplementationShape>,
    module_declaration: Option<ModuleDeclarationShape>,
    extra_refusals: Vec<RefusalForm>,
}

impl NewUnit {
    fn plain(
        kind: UnitKind,
        name: String,
        range: (usize, usize),
        header_token: usize,
        conditions: Vec<Predicate>,
        is_public: bool,
    ) -> Self {
        Self {
            kind,
            name,
            first_token: range.0,
            end_token: range.1,
            header_token,
            conditions,
            is_public,
            declared_names: None,
            imports: None,
            implementation: None,
            module_declaration: None,
            extra_refusals: Vec::new(),
        }
    }
}

/// 条目级宏调用的位置：最后一段名字、`!` 与开括号。
struct MacroInvocationShape {
    macro_name: String,
    open_bracket: usize,
}

/// `include_*!` 的参数是哪一种允许的形态。
enum IncludeArgument {
    RelativeToFile(String),
    RelativeToManifest(String),
    BuildScriptOutput,
    NotAllowed,
}

/// 运行期按清单目录拼路径读，折出来的一样（拼出的目录绑到一个名字上时，那个名字的每一处用法各折一样）。
enum RuntimeFold {
    File(PathBuf),
    /// `crates/` 之下（不是 `crates/` 本身）的一个目录：整个目录当一个数据单元。
    Directory(PathBuf),
    Missing(PathBuf),
    NotFoldable,
}

/// `env!` / `option_env!` 读的这个变量的值只能由本 crate 构建脚本经 `cargo::rustc-env` 交进来：不空、不以 `CARGO` 起头、也不是 `OUT_DIR`
/// （`SourceRead::BuildScriptEnvironmentValue`；与 `scripts/gate/lib_function_units.py` 的 `build-environment` 那一支同一个判法）。
fn is_handed_over_by_a_build_script(variable: &str) -> bool {
    !variable.is_empty()
        && !variable.starts_with(CARGO_SUPPLIED_ENVIRONMENT_VARIABLE_PREFIX)
        && variable != BUILD_SCRIPT_OUTPUT_DIRECTORY_VARIABLE
}

/// 名字的写法是不是一个标识符（`r#` 起头的也算）。
fn is_identifier_text(text: &str) -> bool {
    let name = identifier_name(text);
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_')
        && characters.all(|character| character.is_alphanumeric() || character == '_')
}

/// `format!` 的格式串与实参折成测试名的几段：`{}` 依次取实参、`{数字}` 按位置取、`{名字}` 是常量；带格式说明（`{:?}`）、
/// 具名实参或实参不是字面串与常量名的，交 `None`。
fn format_pieces(inside: &[&str]) -> Option<Vec<TestNamePiece>> {
    let parts = split_at_top_level_commas(inside);
    let (format_part, argument_parts) = parts.split_first()?;
    let [format_literal] = format_part.as_slice() else {
        return None;
    };
    let template = plain_string_content(format_literal)?;
    let mut arguments = Vec::new();
    for argument in argument_parts {
        match argument.as_slice() {
            [literal] if plain_string_content(literal).is_some() => arguments.push(
                TestNamePiece::Literal(plain_string_content(literal)?.to_string()),
            ),
            [identifier] | ["&", identifier] if is_identifier_text(identifier) => arguments.push(
                TestNamePiece::Constant(identifier_name(identifier).to_string()),
            ),
            _ => return None,
        }
    }
    let characters: Vec<char> = template.chars().collect();
    let mut pieces = Vec::new();
    let mut literal_run = String::new();
    let mut next_positional = 0usize;
    let mut position = 0usize;
    while position < characters.len() {
        let current = characters[position];
        let next = characters.get(position + 1).copied();
        match (current, next) {
            ('{', Some('{')) | ('}', Some('}')) => {
                literal_run.push(current);
                position += 2;
            }
            ('{', _) => {
                let close =
                    (position + 1..characters.len()).find(|index| characters[*index] == '}')?;
                let placeholder: String = characters[position + 1..close].iter().collect();
                if !literal_run.is_empty() {
                    pieces.push(TestNamePiece::Literal(std::mem::take(&mut literal_run)));
                }
                if placeholder.is_empty() {
                    pieces.push(arguments.get(next_positional)?.clone());
                    next_positional += 1;
                } else if placeholder
                    .chars()
                    .all(|character| character.is_ascii_digit())
                {
                    let argument_position: usize = placeholder.parse().ok()?;
                    pieces.push(arguments.get(argument_position)?.clone());
                } else if is_identifier_text(&placeholder) {
                    pieces.push(TestNamePiece::Constant(placeholder));
                } else {
                    return None;
                }
                position = close + 1;
            }
            ('}', _) => return None,
            _ => {
                literal_run.push(current);
                position += 1;
            }
        }
    }
    if !literal_run.is_empty() {
        pieces.push(TestNamePiece::Literal(literal_run));
    }
    Some(pieces)
}

/// 一个表达式（词法单元原文）折成子进程测试名的几段：字面串、常量名（`&常量名` 同）、全由字面串与常量组成的 `format!`、
/// 全是字面串的 `concat!`。别的写法交 `None`。闭包里折调用处实参时同用这一份（`closure`）。
#[must_use]
pub fn test_name_pieces_of_texts(texts: &[&str]) -> Option<Vec<TestNamePiece>> {
    match texts {
        [literal] if plain_string_content(literal).is_some() => Some(vec![TestNamePiece::Literal(
            plain_string_content(literal)?.to_string(),
        )]),
        [identifier] | ["&", identifier] if is_identifier_text(identifier) => {
            Some(vec![TestNamePiece::Constant(
                identifier_name(identifier).to_string(),
            )])
        }
        ["format", "!", "(", inside @ .., ")"] => format_pieces(inside),
        ["concat", "!", "(", inside @ .., ")"] => split_at_top_level_commas(inside)
            .iter()
            .map(|part| match part.as_slice() {
                [literal] => plain_string_content(literal)
                    .map(|content| TestNamePiece::Literal(content.to_string())),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

/// 一段条件链里有没有 `test` 这一项（与起来的：`test` 本身，或 `all(…)` 里的一项）：有它，这段代码只在测试里编进来。
#[must_use]
pub fn predicate_requires_the_test_configuration(predicate: &Predicate) -> bool {
    match predicate {
        Predicate::Name(name) => name == "test",
        Predicate::All(conjuncts) => conjuncts
            .iter()
            .any(predicate_requires_the_test_configuration),
        Predicate::KeyValue { .. }
        | Predicate::Any(_)
        | Predicate::Not(_)
        | Predicate::Unrecognized(_) => false,
    }
}

/// 一份文件里 `const 名字: &str = env!("CARGO_MANIFEST_DIR");` 这一形的私有常量（`pub` 的不收：别的文件里的用法这里看不见）。
/// 文件里哪个串字面量带 `{名字` 的格式捕获也不收：捕获里的用法不是词法单元，跟不到。
fn private_manifest_directory_constants(source: &str, tokens: &[Token]) -> BTreeSet<String> {
    let text_at = |index: usize| {
        tokens
            .get(index)
            .map_or("", |token| token_text(source, token))
    };
    let mut constants = BTreeSet::new();
    for index in 0..tokens.len() {
        if text_at(index) != "const"
            || tokens[index + 1..].first().map(|token| token.kind) != Some(TokenKind::Identifier)
        {
            continue;
        }
        let is_public = index > 0 && matches!(text_at(index - 1), "pub" | ")");
        let Some(equals) = (index + 2..tokens.len().min(index + 12))
            .find(|position| matches!(text_at(*position), "=" | ";"))
        else {
            continue;
        };
        let is_manifest_directory = text_at(equals) == "="
            && text_at(equals + 1) == "env"
            && text_at(equals + 2) == "!"
            && text_at(equals + 3) == "("
            && plain_string_content(text_at(equals + 4)) == Some("CARGO_MANIFEST_DIR")
            && text_at(equals + 5) == ")"
            && text_at(equals + 6) == ";";
        if is_manifest_directory && !is_public {
            constants.insert(identifier_name(text_at(index + 1)).to_string());
        }
    }
    constants.retain(|name| {
        let capture = format!("{{{name}");
        !tokens.iter().any(|token| {
            token.kind == TokenKind::Literal && token_text(source, token).contains(&capture)
        })
    });
    constants
}

struct RustScanner<'scan> {
    source: &'scan str,
    file: FileIndex,
    file_directory: &'scan Path,
    manifest_directory: Option<&'scan Path>,
    switches: &'scan RuleSwitches,
    tokens: &'scan [Token],
    closes: &'scan [Option<usize>],
    documentation_comments: &'scan [DocumentationComment],
    /// 这份文件里的私有清单目录常量（[`private_manifest_directory_constants`]）：它们的用法按 `env!("CARGO_MANIFEST_DIR")` 折。
    manifest_directory_constants: BTreeSet<String>,
    units: Vec<Unit>,
    file_level_refusals: Vec<Refusal>,
}

impl<'scan> RustScanner<'scan> {
    fn text_of(&self, index: usize) -> &'scan str {
        self.tokens
            .get(index)
            .map_or("", |token| token_text(self.source, token))
    }

    fn kind_of(&self, index: usize) -> Option<TokenKind> {
        self.tokens.get(index).map(|token| token.kind)
    }

    fn is_identifier(&self, index: usize) -> bool {
        self.kind_of(index) == Some(TokenKind::Identifier)
    }

    fn name_at(&self, index: usize) -> String {
        identifier_name(self.text_of(index)).to_string()
    }

    fn line_of(&self, index: usize) -> u32 {
        self.tokens
            .get(index)
            .or_else(|| self.tokens.last())
            .map_or(1, |token| token.line)
    }

    fn is_open_bracket(&self, index: usize) -> bool {
        self.kind_of(index) == Some(TokenKind::Punctuation)
            && matches!(self.text_of(index), "(" | "[" | "{")
    }

    fn close_of(&self, open_index: usize) -> usize {
        self.closes[open_index].expect("括号在切条目之前配过，开括号一定有配对的闭括号")
    }

    fn is_documentation_token(&self, index: usize) -> bool {
        self.kind_of(index) == Some(TokenKind::Literal)
            && (self.text_of(index).starts_with("//") || self.text_of(index).starts_with("/*"))
    }

    fn fragment(&self, first: usize, end: usize) -> String {
        let last = end.min(first + FRAGMENT_TOKEN_LIMIT).min(self.tokens.len());
        let joined = (first..last)
            .map(|index| self.text_of(index))
            .collect::<Vec<&str>>()
            .join(" ");
        cleaned_fragment(&joined)
    }

    fn refusal(&self, form: RefusalForm, at_token: usize, fragment: String) -> Refusal {
        Refusal {
            file: crate::units::RefusalFile::WorkspaceSourceFile(self.file),
            line: self.line_of(at_token),
            form,
            fragment,
        }
    }

    fn attribute_facts(&self, bracket_ranges: &[(usize, usize)]) -> AttributeFacts {
        let mut facts = AttributeFacts::default();
        for (open, close) in bracket_ranges {
            let texts: Vec<&str> = (open + 1..*close)
                .map(|index| self.text_of(index))
                .collect();
            facts.conditions.extend(conditions_of_attribute(&texts));
            if let ["path", "=", literal] = texts.as_slice() {
                facts.path_attribute = plain_string_content(literal).map(str::to_string);
            }
            if texts.first() == Some(&"macro_export") {
                facts.is_macro_export = true;
            }
        }
        facts
    }

    /// 从 `low` 起的内属性 `#![…]`（与夹在中间的内文档注释词法单元）之后的位置，连同每个内属性的方括号区间。
    fn inner_attributes_from(&self, low: usize, high: usize) -> (usize, Vec<(usize, usize)>) {
        let mut index = low;
        let mut ranges = Vec::new();
        while index < high {
            if self.text_of(index) == "#"
                && self.text_of(index + 1) == "!"
                && self.text_of(index + 2) == "["
            {
                let close = self.close_of(index + 2);
                ranges.push((index + 2, close));
                index = close + 1;
            } else if self.is_documentation_token(index)
                && (self.text_of(index).starts_with("//!")
                    || self.text_of(index).starts_with("/*!"))
            {
                index += 1;
            } else {
                break;
            }
        }
        (index, ranges)
    }

    fn skip_visibility(&self, index: usize) -> (usize, bool) {
        match self.text_of(index) {
            "pub" => {
                if self.text_of(index + 1) == "(" {
                    (self.close_of(index + 1) + 1, false)
                } else {
                    (index + 1, true)
                }
            }
            "crate" if self.text_of(index + 1) != ":" => (index + 1, false),
            _ => (index, false),
        }
    }

    /// 跳过 `const`、`async`、`unsafe`、`extern "ABI"`、`default`、`safe`、`auto` 这类修饰词，交条目关键字的位置。
    /// `verus` 为真时另跳 Verus 的 `open`、`closed`、`spec`、`proof`、`exec`、`tracked`、`ghost`、`uninterp`、`axiom` 与 `broadcast proof`。
    fn skip_qualifiers(&self, start: usize, verus: bool) -> usize {
        let mut index = start;
        loop {
            let next = self.text_of(index + 1);
            let is_qualifier = match self.text_of(index) {
                "const" => matches!(next, "fn" | "unsafe" | "async" | "extern"),
                "async" => matches!(next, "fn" | "unsafe"),
                "unsafe" => matches!(next, "fn" | "impl" | "trait" | "extern" | "auto"),
                "extern" => {
                    if self.kind_of(index + 1) == Some(TokenKind::Literal)
                        && self.text_of(index + 2) == "fn"
                    {
                        index += 1;
                        true
                    } else {
                        next == "fn"
                    }
                }
                "default" => matches!(next, "fn" | "type" | "const" | "unsafe" | "async" | "impl"),
                "safe" => matches!(next, "fn" | "static"),
                "auto" => next == "trait",
                "open" | "closed" | "spec" | "proof" | "exec" | "tracked" | "ghost"
                | "uninterp" | "axiom" => verus,
                "broadcast" => {
                    verus
                        && matches!(
                            next,
                            "proof" | "fn" | "spec" | "axiom" | "uninterp" | "open" | "closed"
                        )
                }
                _ => false,
            };
            if !is_qualifier {
                return index;
            }
            index += 1;
        }
    }

    /// 条目从 `from` 起到哪里止：遇到 `;` 止；`braced_body_ends` 为真时第一个顶层 `{…}` 闭上就止。
    fn end_of_item(&self, from: usize, high: usize, braced_body_ends: bool) -> usize {
        let mut index = from;
        while index < high {
            if self.is_open_bracket(index) {
                let close = self.close_of(index);
                if braced_body_ends && self.text_of(index) == "{" {
                    return close + 1;
                }
                index = close + 1;
                continue;
            }
            if self.text_of(index) == ";" && self.kind_of(index) == Some(TokenKind::Punctuation) {
                return index + 1;
            }
            index += 1;
        }
        high
    }

    /// 第一个顶层 `{` 或 `;`（跳过圆括号与方括号）。
    fn first_brace_or_semicolon(&self, from: usize, high: usize) -> usize {
        let mut index = from;
        while index < high {
            match self.text_of(index) {
                "{" | ";" => return index,
                "(" | "[" => index = self.close_of(index) + 1,
                _ => index += 1,
            }
        }
        high
    }

    fn macro_invocation_at(&self, start: usize) -> Option<MacroInvocationShape> {
        let mut index = start;
        if self.text_of(index) == ":" && self.text_of(index + 1) == ":" {
            index += 2;
        }
        let mut macro_name;
        loop {
            if !self.is_identifier(index) {
                return None;
            }
            macro_name = self.name_at(index);
            index += 1;
            if self.text_of(index) == ":" && self.text_of(index + 1) == ":" {
                if !self.switches.is_intact(BrokenRule::PathMacroInvocations) {
                    return None;
                }
                index += 2;
                continue;
            }
            break;
        }
        if self.text_of(index) != "!" || !self.is_open_bracket(index + 1) {
            return None;
        }
        Some(MacroInvocationShape {
            macro_name,
            open_bracket: index + 1,
        })
    }

    fn arrow_close(&self, index: usize) -> bool {
        index > 0
            && self.text_of(index - 1) == "-"
            && self.tokens[index - 1].end_byte == self.tokens[index].start_byte
    }

    fn implementation_shape(&self, low: usize, high: usize) -> ImplementationShape {
        let mut index = low;
        // `impl<…>` 自己的泛型参数：深度 1 上紧跟 `<`、`,` 或 `const` 的名字（自身类型里出现其中任一个的 impl 算 blanket）
        let mut generic_parameters: BTreeSet<String> = BTreeSet::new();
        if self.text_of(index) == "<" {
            let mut depth = 0usize;
            while index < high {
                match self.text_of(index) {
                    "<" => depth += 1,
                    ">" if !self.arrow_close(index) => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            index += 1;
                            break;
                        }
                    }
                    text if depth == 1
                        && self.is_identifier(index)
                        && text != "const"
                        && index > low
                        && matches!(self.text_of(index - 1), "<" | "," | "const") =>
                    {
                        generic_parameters.insert(self.name_at(index));
                    }
                    _ => {}
                }
                index += 1;
            }
        }
        let mut depth = 0usize;
        let mut for_index = None;
        let mut where_index = high;
        for position in index..high {
            match self.text_of(position) {
                "<" => depth += 1,
                ">" if !self.arrow_close(position) => depth = depth.saturating_sub(1),
                "for" if depth == 0 && for_index.is_none() && self.text_of(position + 1) != "<" => {
                    for_index = Some(position);
                }
                "where" if depth == 0 => {
                    where_index = position;
                    break;
                }
                _ => {}
            }
        }
        let is_skipped_word =
            |name: &str| matches!(name, "dyn" | "mut" | "const" | "impl" | "for" | "where");
        let names_in = |range_start: usize, range_end: usize| -> Vec<String> {
            let mut names = BTreeSet::new();
            for position in range_start..range_end {
                if self.is_identifier(position) {
                    let name = self.name_at(position);
                    if !is_skipped_word(&name) {
                        names.insert(name);
                    }
                }
            }
            names.into_iter().collect()
        };
        match for_index {
            Some(for_position) => {
                // trait 自己的名字：`for` 之前那一截里尖括号外的最后一个名字（`dyn` 不算）
                let mut trait_name = None;
                let mut trait_depth = 0usize;
                for position in index..for_position {
                    match self.text_of(position) {
                        "<" => trait_depth += 1,
                        ">" if !self.arrow_close(position) => {
                            trait_depth = trait_depth.saturating_sub(1);
                        }
                        _ if trait_depth == 0 && self.is_identifier(position) => {
                            let name = self.name_at(position);
                            if name != "dyn" {
                                trait_name = Some(name);
                            }
                        }
                        _ => {}
                    }
                }
                let self_type_names = names_in(for_position + 1, where_index);
                // 自身类型里任何深度出现这个 impl 自己的一个泛型参数（`for T`、`&T`、`Box<T>`、`(u32, T)`、`[T; N]`、
                // `Option<Box<T>>`）就算 blanket：这类 impl 的自身类型里可能一个能被点名的具体名字都没有（`[T]`），
                // 不放行它的成员就进不了任何闭包、键过窄（「多连不少连」）。全具体的自身类型照旧等被点名。
                let is_blanket_implementation = self_type_names
                    .iter()
                    .any(|name| generic_parameters.contains(name));
                ImplementationShape {
                    self_type_names,
                    trait_name,
                    is_trait_implementation: true,
                    is_blanket_implementation,
                    self_type_path_root: self.self_type_path_root(for_position + 1, where_index),
                }
            }
            None => ImplementationShape {
                self_type_names: names_in(index, where_index),
                trait_name: None,
                is_trait_implementation: false,
                is_blanket_implementation: false,
                self_type_path_root: self.self_type_path_root(index, where_index),
            },
        }
    }

    /// impl 自身类型（`range_start..range_end`）写成路径 `a::B` 时的头一段：跳过 [`WORDS_BEFORE_THE_SELF_TYPE_PATH_ROOT`] 之后的头一个名字，
    /// 后面紧接 `::` 才算；不是路径的交 `None`（`ImplementationShape::self_type_path_root`）。
    fn self_type_path_root(&self, range_start: usize, range_end: usize) -> Option<String> {
        let first_name = (range_start..range_end).find(|position| {
            self.is_identifier(*position)
                && !WORDS_BEFORE_THE_SELF_TYPE_PATH_ROOT.contains(&self.text_of(*position))
        })?;
        (self.text_of(first_name + 1) == ":" && self.text_of(first_name + 2) == ":")
            .then(|| self.name_at(first_name))
    }

    fn use_imports(&self, low: usize, high: usize) -> UseImports {
        let mut imports = UseImports::default();
        let mut names = BTreeSet::new();
        // 套着的每一层 `{` 前面那一段（`a::{` 的 `a`；不是名字、或是 self / super / crate 的记 None）：花括号里的 `self` 引进的就是它
        let mut segments_before_the_braces: Vec<Option<String>> = Vec::new();
        let original_name_at = |index: usize, segments: &[Option<String>]| -> Option<String> {
            let name = self.name_at(index);
            if name == "self" && self.text_of(index + 1) != ":" {
                if let Some(segment) = segments.last() {
                    return segment.clone();
                }
            }
            Some(name)
        };
        for index in low..high {
            let text = self.text_of(index);
            if text == "*" {
                imports.is_glob = true;
                continue;
            }
            if text == "{" {
                let segment = if index >= low + 3
                    && self.text_of(index - 1) == ":"
                    && self.text_of(index - 2) == ":"
                    && self.is_identifier(index - 3)
                {
                    Some(self.name_at(index - 3))
                        .filter(|name| !matches!(name.as_str(), "self" | "super" | "crate"))
                } else {
                    None
                };
                segments_before_the_braces.push(segment);
                continue;
            }
            if text == "}" {
                segments_before_the_braces.pop();
                continue;
            }
            if !self.is_identifier(index) {
                continue;
            }
            let name = self.name_at(index);
            if name == "as" {
                if self.is_identifier(index + 1) && index > low && self.is_identifier(index - 1) {
                    if let Some(original) = original_name_at(index - 1, &segments_before_the_braces)
                    {
                        imports.aliases.push((self.name_at(index + 1), original));
                    }
                }
                continue;
            }
            let is_path_segment = self.text_of(index + 1) == ":" && self.text_of(index + 2) == ":";
            if is_path_segment {
                continue;
            }
            // `a::b as c`：原名 `b` 不进当前作用域，进的只有别名 `c`（在 `as` 那一格记进 aliases 与 names）
            if self.text_of(index + 1) == "as" {
                continue;
            }
            if !matches!(name.as_str(), "self" | "super" | "crate") {
                names.insert(name);
            } else if name == "self" {
                if let Some(original) = original_name_at(index, &segments_before_the_braces)
                    .filter(|original| original != "self")
                {
                    names.insert(original);
                }
            }
        }
        imports.names = names.into_iter().collect();
        imports
    }

    /// R-宏：条目级宏调用里的标识符都算它可能定义的名字（属性里的、`.` 后面的与基本类型名也算，宁多不漏），只去掉 Rust 关键字；
    /// 与 `scripts/gate/lib_function_units.py` 的条目级宏调用 `defines` 同一口径。
    fn item_macro_declared_names(&self, open: usize, close: usize) -> Vec<String> {
        let mut names = BTreeSet::new();
        for index in open + 1..close {
            if self.is_identifier(index) {
                let name = self.name_at(index);
                if !is_keyword(&name) {
                    names.insert(name);
                }
            }
        }
        names.into_iter().collect()
    }

    fn extern_block_declared_names(&self, open: usize, close: usize) -> Vec<String> {
        let mut names = BTreeSet::new();
        for index in open + 1..close {
            if matches!(self.text_of(index), "fn" | "static") {
                let mut name_index = index + 1;
                if self.text_of(name_index) == "mut" {
                    name_index += 1;
                }
                if self.is_identifier(name_index) {
                    names.insert(self.name_at(name_index));
                }
            }
        }
        names.into_iter().collect()
    }

    fn include_argument(&self, low: usize, high: usize) -> IncludeArgument {
        let texts: Vec<&str> = (low..high).map(|index| self.text_of(index)).collect();
        if let [literal] = texts.as_slice() {
            return plain_string_content(literal).map_or(IncludeArgument::NotAllowed, |content| {
                IncludeArgument::RelativeToFile(content.to_string())
            });
        }
        let ["concat", "!", "(", inside @ .., ")"] = texts.as_slice() else {
            return IncludeArgument::NotAllowed;
        };
        if self.close_of(low + 2) + 1 != high {
            return IncludeArgument::NotAllowed;
        }
        let parts = split_at_top_level_commas(inside);
        let mut concatenated = String::new();
        let mut environment_start = None;
        for (part_position, part) in parts.iter().enumerate() {
            match part.as_slice() {
                [literal] => match plain_string_content(literal) {
                    Some(content) => concatenated.push_str(content),
                    None => return IncludeArgument::NotAllowed,
                },
                ["env", "!", "(", literal, ")"] if part_position == 0 => {
                    environment_start = plain_string_content(literal);
                }
                _ => return IncludeArgument::NotAllowed,
            }
        }
        match environment_start {
            None => IncludeArgument::RelativeToFile(concatenated),
            Some("CARGO_MANIFEST_DIR") => IncludeArgument::RelativeToManifest(concatenated),
            Some("OUT_DIR") => IncludeArgument::BuildScriptOutput,
            Some(_) => IncludeArgument::NotAllowed,
        }
    }

    fn file_read(&self, absolute_path: PathBuf, line: u32, origin: ReadOrigin) -> SourceRead {
        let digest = std::fs::read(&absolute_path)
            .ok()
            .map(|bytes| sha256_of(&bytes));
        SourceRead::File {
            absolute_path,
            line,
            origin,
            digest,
        }
    }

    fn manifest_relative_path(&self, suffix: &str) -> Option<PathBuf> {
        let manifest = self.manifest_directory?;
        let joined = format!("{}{suffix}", manifest.display());
        Some(normalized_path(Path::new(&joined)))
    }

    fn text_before(&self, index: usize) -> &'scan str {
        index
            .checked_sub(1)
            .map_or("", |previous| self.text_of(previous))
    }

    /// 折出的目录落在清单目录的上一级（`crates/`）之下、又不是它本身：目录数据单元；仓根、`crates/` 之外与 `crates/` 整个照旧拒算。
    fn directory_fold(&self, folded: PathBuf) -> RuntimeFold {
        match self.manifest_directory.and_then(Path::parent) {
            Some(crates_directory)
                if folded.starts_with(crates_directory) && folded != crates_directory =>
            {
                RuntimeFold::Directory(folded)
            }
            Some(_) | None => RuntimeFold::NotFoldable,
        }
    }

    /// `std::path::Path::new` 这类写法起头的那个词法单元：从 `new` / `from` 往回跨过 `名字 ::`。
    fn path_expression_start(&self, constructor_index: usize) -> usize {
        let mut start = constructor_index;
        while start >= 3
            && self.text_of(start - 1) == ":"
            && self.text_of(start - 2) == ":"
            && self.is_identifier(start - 3)
        {
            start -= 3;
        }
        start
    }

    /// 从 `expression_start` 起、到 `after_expression` 之前的表达式整个是 `let [mut] 名字 [: 类型] = …;` 的右边时，交那个名字。
    fn let_binding_name(&self, expression_start: usize, after_expression: usize) -> Option<String> {
        if self.text_of(after_expression) != ";" || self.text_before(expression_start) != "=" {
            return None;
        }
        let equals = expression_start - 1;
        let let_index = (0..equals)
            .rev()
            .find(|position| matches!(self.text_of(*position), "let" | ";" | "{" | "}" | "="))?;
        if self.text_of(let_index) != "let" {
            return None;
        }
        let mut name_index = let_index + 1;
        if self.text_of(name_index) == "mut" {
            name_index += 1;
        }
        if !self.is_identifier(name_index) {
            return None;
        }
        let is_plain = name_index + 1 == equals;
        let is_typed = self.text_of(name_index + 1) == ":" && self.text_of(name_index + 2) != ":";
        (is_plain || is_typed).then(|| self.name_at(name_index))
    }

    /// 名字在 `from..end` 里的每一处用法（`.名字` 的字段或方法、`名字::` 的路径段不算）；碰到再 `let` 这个名字就停（之后是另一个值）。
    fn uses_of_the_binding(&self, name: &str, from: usize, end: usize) -> Vec<usize> {
        let mut uses = Vec::new();
        for index in from..end {
            if !self.is_identifier(index) || self.name_at(index) != name {
                continue;
            }
            let previous = self.text_before(index);
            if previous == "let" || (previous == "mut" && self.text_before(index - 1) == "let") {
                break;
            }
            if previous == "." || (self.text_of(index + 1) == ":" && self.text_of(index + 2) == ":")
            {
                continue;
            }
            uses.push(index);
        }
        uses
    }

    /// 从 `chain_start` 起接字面 `.join("…")` 链，拼在 `base` 后面。折出的是目录、表达式又整个是一条 `let 名字 = …;` 的右边时，
    /// 交那个名字每一处用法折出的（[`Self::fold_use_of_a_path`]；名字后面再没用到的交目录本身）：名字之后接了 `.parent()`、
    /// `.join(变量)` 这类的，那一处拒算，读整棵仓的写法照旧拒。
    fn fold_path_chain(
        &self,
        base: &Path,
        chain_start: usize,
        expression_start: usize,
        unit_end: usize,
        binding_depth: usize,
    ) -> Vec<RuntimeFold> {
        let mut path = base.to_path_buf();
        let mut position = chain_start;
        while self.text_of(position) == "."
            && self.text_of(position + 1) == "join"
            && self.text_of(position + 2) == "("
            && self.text_of(position + 4) == ")"
        {
            let Some(content) = plain_string_content(self.text_of(position + 3)) else {
                break;
            };
            path.push(content);
            position += 5;
        }
        // 字面 `.join(…)` 链之后还接任何方法（`.canonicalize()`、`.parent()`、`.join(变量)`、`.read_dir()` 这类）：
        // 拼出的不再是交给读文件的那条字面路径，拒算（与 `scripts/gate/lib_function_units.py` 的 `runtime_manifest_path` 同一条）
        if self.text_of(position) == "." {
            return vec![RuntimeFold::NotFoldable];
        }
        let folded = normalized_path(&path);
        if folded.is_file() {
            return vec![RuntimeFold::File(folded)];
        }
        if !folded.is_dir() {
            return vec![RuntimeFold::Missing(folded)];
        }
        let Some(binding) = self.let_binding_name(expression_start, position) else {
            return vec![self.directory_fold(folded)];
        };
        if binding_depth >= DEEPEST_DIRECTORY_BINDING_CHAIN {
            return vec![RuntimeFold::NotFoldable];
        }
        let uses = self.uses_of_the_binding(&binding, position + 1, unit_end);
        if uses.is_empty() {
            return vec![self.directory_fold(folded)];
        }
        uses.into_iter()
            .flat_map(|use_index| {
                self.fold_use_of_a_path(&folded, use_index, unit_end, binding_depth + 1)
            })
            .collect()
    }

    fn is_wrapped_in_a_path_constructor(&self, use_index: usize) -> bool {
        use_index >= 2
            && self.text_of(use_index - 1) == "("
            && matches!(self.text_of(use_index - 2), "new" | "from")
            && self.text_of(use_index + 1) == ")"
    }

    /// 绑着一个路径的名字的一处用法：`Path::new(名字)` / `PathBuf::from(名字)` 之后、或名字本身之后接的字面 `.join` 链；
    /// 名字单独交给别的函数（`strip_prefix(名字)`、`扫目录(名字)`）的，折出的就是它绑着的目录。
    fn fold_use_of_a_path(
        &self,
        value: &Path,
        use_index: usize,
        unit_end: usize,
        binding_depth: usize,
    ) -> Vec<RuntimeFold> {
        if self.is_wrapped_in_a_path_constructor(use_index) {
            return self.fold_path_chain(
                value,
                use_index + 2,
                self.path_expression_start(use_index - 2),
                unit_end,
                binding_depth,
            );
        }
        self.fold_path_chain(value, use_index + 1, use_index, unit_end, binding_depth)
    }

    /// 私有清单目录常量的一处用法：只认 `Path::new(常量)` / `PathBuf::from(常量)`（串拼进 `format!` 这类的拒算）。
    fn fold_manifest_directory_constant_use(
        &self,
        use_index: usize,
        unit_end: usize,
    ) -> Vec<RuntimeFold> {
        match self.manifest_directory {
            Some(manifest) if self.is_wrapped_in_a_path_constructor(use_index) => {
                self.fold_use_of_a_path(manifest, use_index, unit_end, 0)
            }
            Some(_) | None => vec![RuntimeFold::NotFoldable],
        }
    }

    /// `const 名字: &str = env!("CARGO_MANIFEST_DIR");` 里的那个 `env!`，而且名字是这份文件收下的私有清单目录常量。
    fn is_manifest_directory_constant_definition(
        &self,
        environment_index: usize,
        environment_close: usize,
    ) -> bool {
        if self.text_of(environment_close + 1) != ";" || self.text_before(environment_index) != "="
        {
            return false;
        }
        (environment_index.saturating_sub(12)..environment_index)
            .rev()
            .find(|position| self.text_of(*position) == "const")
            .is_some_and(|const_index| {
                self.manifest_directory_constants
                    .contains(&self.name_at(const_index + 1))
            })
    }

    fn fold_manifest_directory_read(
        &self,
        environment_index: usize,
        environment_close: usize,
        unit_end: usize,
    ) -> Vec<RuntimeFold> {
        let Some(manifest) = self.manifest_directory else {
            return vec![RuntimeFold::NotFoldable];
        };
        if environment_index >= 3
            && self.text_of(environment_index - 1) == "("
            && self.text_of(environment_index - 2) == "!"
            && self.text_of(environment_index - 3) == "concat"
        {
            let concat_close = self.close_of(environment_index - 1);
            if self.text_of(environment_close + 1) != "," && environment_close + 1 != concat_close {
                return vec![RuntimeFold::NotFoldable];
            }
            let texts: Vec<&str> = (environment_close + 2..concat_close)
                .map(|index| self.text_of(index))
                .collect();
            let mut suffix = String::new();
            for part in split_at_top_level_commas(&texts) {
                match part.as_slice() {
                    [literal] => match plain_string_content(literal) {
                        Some(content) => suffix.push_str(content),
                        None => return vec![RuntimeFold::NotFoldable],
                    },
                    _ => return vec![RuntimeFold::NotFoldable],
                }
            }
            let Some(folded) = self.manifest_relative_path(&suffix) else {
                return vec![RuntimeFold::NotFoldable];
            };
            // `concat!` 拼出的是串：绑到名字上的不跟（用法里的 `Path::new(名字).parent()` 看不见），只认就地交出去的那一处
            return if folded.is_file() {
                vec![RuntimeFold::File(folded)]
            } else if !folded.is_dir() {
                vec![RuntimeFold::Missing(folded)]
            } else if self
                .let_binding_name(environment_index - 3, concat_close + 1)
                .is_some()
            {
                vec![RuntimeFold::NotFoldable]
            } else {
                vec![self.directory_fold(folded)]
            };
        }
        if environment_index >= 2
            && self.text_of(environment_index - 1) == "("
            && matches!(self.text_of(environment_index - 2), "new" | "from")
            && self.close_of(environment_index - 1) == environment_close + 1
        {
            return self.fold_path_chain(
                manifest,
                environment_close + 2,
                self.path_expression_start(environment_index - 2),
                unit_end,
                0,
            );
        }
        if self.is_manifest_directory_constant_definition(environment_index, environment_close) {
            // 读在常量的用法那几处折（`fold_manifest_directory_constant_use`）
            return Vec::new();
        }
        vec![RuntimeFold::NotFoldable]
    }

    /// 一样折出的东西记成读或拒算；拒算落在起折的那个词法单元（`env!` 或常量名）上。
    fn record_runtime_fold(
        &self,
        fold: RuntimeFold,
        origin_index: usize,
        reads: &mut Vec<SourceRead>,
        refusals: &mut Vec<Refusal>,
    ) {
        let line = self.line_of(origin_index);
        let read = match fold {
            RuntimeFold::File(path) => self.file_read(path, line, ReadOrigin::Runtime),
            RuntimeFold::Directory(path) => match directory_digest(&path) {
                Ok(digest) => SourceRead::Directory {
                    absolute_path: path,
                    line,
                    digest,
                },
                Err(unreadable) => {
                    refusals.push(self.refusal(
                        RefusalForm::DataFileUnreadable,
                        origin_index,
                        cleaned_fragment(&unreadable.display().to_string()),
                    ));
                    return;
                }
            },
            RuntimeFold::Missing(path) => {
                refusals.push(self.refusal(
                    RefusalForm::DataFileUnreadable,
                    origin_index,
                    cleaned_fragment(&path.display().to_string()),
                ));
                return;
            }
            RuntimeFold::NotFoldable => {
                refusals.push(self.refusal(
                    RefusalForm::RuntimeReadWithoutLiteralPath,
                    origin_index,
                    self.fragment(origin_index.saturating_sub(4), origin_index + 9),
                ));
                return;
            }
        };
        if !reads.contains(&read) {
            reads.push(read);
        }
    }

    /// 这个单元是函数时，`name` 是它第几个参数（从 0 起）；带 `self` 的方法、不是参数的交 `None`。同时交函数名。
    fn parameter_position(
        &self,
        unit_first: usize,
        unit_end: usize,
        name: &str,
    ) -> Option<(String, usize)> {
        let fn_index = (unit_first..unit_end)
            .find(|index| self.text_of(*index) == "fn" && self.is_identifier(index + 1))?;
        let open = (fn_index + 2..unit_end).find(|index| self.text_of(*index) == "(")?;
        let close = self.close_of(open);
        let mut parameters: Vec<Vec<&str>> = Vec::new();
        let mut current: Vec<&str> = Vec::new();
        let mut bracket_depth = 0usize;
        let mut angle_depth = 0usize;
        for index in open + 1..close {
            let text = self.text_of(index);
            match text {
                "(" | "[" | "{" => bracket_depth += 1,
                ")" | "]" | "}" => bracket_depth = bracket_depth.saturating_sub(1),
                "<" => angle_depth += 1,
                ">" if self.text_before(index) != "-" => {
                    angle_depth = angle_depth.saturating_sub(1);
                }
                "," if bracket_depth == 0 && angle_depth == 0 => {
                    parameters.push(std::mem::take(&mut current));
                    continue;
                }
                _ => {}
            }
            current.push(text);
        }
        if !current.is_empty() {
            parameters.push(current);
        }
        let has_receiver = parameters.first().is_some_and(|first| {
            first
                .iter()
                .take_while(|text| **text != ":")
                .any(|text| *text == "self")
        });
        if has_receiver {
            return None;
        }
        parameters
            .iter()
            .position(|parameter| {
                let pattern = match parameter.as_slice() {
                    ["mut", rest @ ..] => rest,
                    whole => whole,
                };
                matches!(pattern, [identifier, ":", ..] if identifier_name(identifier) == name)
            })
            .map(|position| (self.name_at(fn_index + 1), position))
    }

    /// `current_exe` 那个词交给 `Command::new(…)` 起子进程、方法链里 `.arg(…)` / `.args([…])` 的参数有 `"--exact"`、它后面那个参数
    /// 折得出测试名的写法（[`test_name_pieces_of_texts`]，或起子进程的函数自己的一个参数）时交那个写法；不是这一形交 `None`
    /// （单元里有 [`SPAWNING_WORDS`] 的词才拒算，没有的只是在问路径）。
    /// 名字在单元里另有 `let` 绑定的不认（认成常量或参数会认错）。
    fn child_test_name_at(
        &self,
        current_exe_index: usize,
        unit_first: usize,
        unit_end: usize,
    ) -> Option<ChildTestName> {
        let enclosing_open = (unit_first..current_exe_index).rev().find(|position| {
            self.text_of(*position) == "("
                && self.closes[*position].is_some_and(|close| close > current_exe_index)
        })?;
        let is_command_new = enclosing_open >= 4
            && self.text_of(enclosing_open - 1) == "new"
            && self.text_of(enclosing_open - 2) == ":"
            && self.text_of(enclosing_open - 3) == ":"
            && self.text_of(enclosing_open - 4) == "Command";
        if !is_command_new {
            return None;
        }
        let mut arguments: Vec<Vec<&str>> = Vec::new();
        let mut position = self.close_of(enclosing_open) + 1;
        while position + 2 < unit_end
            && self.text_of(position) == "."
            && self.is_identifier(position + 1)
            && self.text_of(position + 2) == "("
        {
            let call_open = position + 2;
            let call_close = self.close_of(call_open);
            let inside: Vec<&str> = (call_open + 1..call_close)
                .map(|index| self.text_of(index))
                .collect();
            match self.text_of(position + 1) {
                "arg" => arguments.push(inside),
                "args" => {
                    let listed = match inside.as_slice() {
                        ["[", listed @ .., "]"]
                        | ["&", "[", listed @ .., "]"]
                        | ["vec", "!", "[", listed @ .., "]"] => listed,
                        _ => return None,
                    };
                    arguments.extend(split_at_top_level_commas(listed));
                }
                _ => {}
            }
            position = call_close + 1;
        }
        let exact_position = arguments
            .iter()
            .position(|argument| argument.as_slice() == ["\"--exact\""])?;
        let name_texts = arguments.get(exact_position + 1)?;
        let bound_by_let = |name: &str| {
            (unit_first..unit_end).any(|index| {
                self.text_of(index) == "let"
                    && (self.name_at(index + 1) == name
                        || (self.text_of(index + 1) == "mut" && self.name_at(index + 2) == name))
            })
        };
        match name_texts.as_slice() {
            [identifier] | ["&", identifier] if is_identifier_text(identifier) => {
                let name = identifier_name(identifier);
                if bound_by_let(name) {
                    return None;
                }
                if let Some((function_name, parameter_position)) =
                    self.parameter_position(unit_first, unit_end, name)
                {
                    return Some(ChildTestName::ParameterOfTheSpawningFunction {
                        function_name,
                        parameter_position,
                    });
                }
                Some(ChildTestName::Pieces(vec![TestNamePiece::Constant(
                    name.to_string(),
                )]))
            }
            _ => test_name_pieces_of_texts(name_texts).map(ChildTestName::Pieces),
        }
    }

    /// 单元的词法单元 `[first, end)` 里有没有起子进程的词（[`SPAWNING_WORDS`]，标识符逐字比）。
    fn starts_a_process(&self, first: usize, end: usize) -> bool {
        (first..end)
            .any(|index| self.is_identifier(index) && SPAWNING_WORDS.contains(&self.text_of(index)))
    }

    fn reads_and_refusals_of(&self, first: usize, end: usize) -> (Vec<SourceRead>, Vec<Refusal>) {
        let mut reads = Vec::new();
        let mut refusals = Vec::new();
        if !self.switches.is_intact(BrokenRule::DataUnits) {
            return (reads, refusals);
        }
        let mut consumed_until = first;
        // 最近一处 `concat!(…)` 的闭括号：在它之前的 `env!` 不当构建脚本交的值（与 python 那一份的跳法相同）
        let mut concatenation_close: Option<usize> = None;
        for index in first..end {
            if index < consumed_until || !self.is_identifier(index) {
                continue;
            }
            match self.text_of(index) {
                "include_str" | "include_bytes" | "include"
                    if self.text_of(index + 1) == "!" && self.text_of(index + 2) == "(" =>
                {
                    let close = self.close_of(index + 2);
                    consumed_until = close + 1;
                    let line = self.line_of(index);
                    match self.include_argument(index + 3, close) {
                        IncludeArgument::RelativeToFile(relative) => reads.push(self.file_read(
                            normalized_path(&self.file_directory.join(relative)),
                            line,
                            ReadOrigin::Included,
                        )),
                        IncludeArgument::RelativeToManifest(suffix) => {
                            match self.manifest_relative_path(&suffix) {
                                Some(path) => {
                                    reads.push(self.file_read(path, line, ReadOrigin::Included))
                                }
                                None => refusals.push(self.refusal(
                                    RefusalForm::IncludeArgumentNotInAllowedForms,
                                    index,
                                    self.fragment(index, close + 1),
                                )),
                            }
                        }
                        IncludeArgument::BuildScriptOutput => {
                            reads.push(SourceRead::BuildScriptOutput { line })
                        }
                        IncludeArgument::NotAllowed => refusals.push(self.refusal(
                            RefusalForm::IncludeArgumentNotInAllowedForms,
                            index,
                            self.fragment(index, close + 1),
                        )),
                    }
                }
                "concat" if self.text_of(index + 1) == "!" && self.is_open_bracket(index + 2) => {
                    concatenation_close = Some(self.close_of(index + 2));
                }
                "env" | "option_env"
                    if self.text_of(index + 1) == "!" && self.is_open_bracket(index + 2) =>
                {
                    let close = self.close_of(index + 2);
                    let Some(variable) = plain_string_content(self.text_of(index + 3)) else {
                        continue;
                    };
                    // python 那一份读到 `concat!(…)` 整段跳过（runtime_concat），里面的 `env!` 不当构建脚本交的值
                    let is_inside_a_concatenation = concatenation_close
                        .is_some_and(|concatenation_end| index < concatenation_end);
                    if !is_inside_a_concatenation && is_handed_over_by_a_build_script(variable) {
                        reads.push(SourceRead::BuildScriptEnvironmentValue {
                            variable: variable.to_string(),
                            line: self.line_of(index),
                        });
                        continue;
                    }
                    if self.text_of(index) != "env"
                        || self.text_of(index + 2) != "("
                        || close != index + 4
                    {
                        continue;
                    }
                    if variable == "CARGO_MANIFEST_DIR" {
                        for fold in self.fold_manifest_directory_read(index, close, end) {
                            self.record_runtime_fold(fold, index, &mut reads, &mut refusals);
                        }
                    } else if let Some(binary_name) = variable.strip_prefix("CARGO_BIN_EXE_") {
                        reads.push(SourceRead::BinaryExecutable {
                            binary_name: binary_name.to_string(),
                            line: self.line_of(index),
                        });
                    }
                }
                "var" | "var_os"
                    if self.text_of(index + 1) == "("
                        && plain_string_content(self.text_of(index + 2))
                            == Some("CARGO_MANIFEST_DIR") =>
                {
                    refusals.push(self.refusal(
                        RefusalForm::RuntimeReadWithoutLiteralPath,
                        index,
                        self.fragment(index.saturating_sub(4), index + 8),
                    ));
                }
                "current_exe" if self.text_of(index + 1) == "(" => {
                    let fragment = self.fragment(index.saturating_sub(4), index + 4);
                    match self.child_test_name_at(index, first, end) {
                        Some(test_name) => reads.push(SourceRead::ChildTestProcess {
                            line: self.line_of(index),
                            test_name,
                            fragment,
                        }),
                        // 同一个单元里起了子进程、又折不出点名的那一条测试：子进程跑什么说不清，拒算
                        None if self.starts_a_process(first, end) => refusals.push(self.refusal(
                            RefusalForm::CurrentExecutablePath,
                            index,
                            fragment,
                        )),
                        // 不起子进程、只问路径（读程序文件的大小、`current_exe().is_ok()`）：路径拿去做什么与要点名的测试无关，
                        // 不拒算、不连边（与 `scripts/gate/lib_function_units.py` 文件头「测试二进制起自己」同一判法）
                        None => {}
                    }
                }
                constant
                    if self
                        .manifest_directory_constants
                        .contains(identifier_name(constant))
                        && self.text_before(index) != "const" =>
                {
                    for fold in self.fold_manifest_directory_constant_use(index, end) {
                        self.record_runtime_fold(fold, index, &mut reads, &mut refusals);
                    }
                }
                "concat_idents" | "paste" if self.text_of(index + 1) == "!" => {
                    refusals.push(self.refusal(
                        RefusalForm::IdentifierPastingMacro,
                        index,
                        self.fragment(index, index + 8),
                    ))
                }
                _ => {}
            }
        }
        (reads, refusals)
    }

    fn documentation_comments_between(
        &self,
        low_token: usize,
        high_token: usize,
        placement: DocumentationPlacement,
    ) -> Vec<&'scan DocumentationComment> {
        let start = self
            .documentation_comments
            .partition_point(|comment| comment.next_token_index < low_token);
        let end = self
            .documentation_comments
            .partition_point(|comment| comment.next_token_index <= high_token);
        self.documentation_comments[start..end.max(start)]
            .iter()
            .filter(|comment| comment.placement == placement)
            .collect()
    }

    fn push_documentation_tests(
        &mut self,
        comments: &[&DocumentationComment],
        attachment: &DocumentationTestAttachment,
        module_path: &[String],
    ) {
        if !self.switches.is_intact(BrokenRule::DocumentationTestUnits) {
            return;
        }
        for block in rust_code_blocks(comments) {
            let lexed = lex_source(
                &block.code,
                DocumentationCommentTreatment::KeptOutOfTheTokens,
            );
            let fingerprint = fingerprint_of_tokens(&block.code, &lexed.tokens);
            let references = references_of_tokens(&block.code, &lexed.tokens, None);
            let condition_predicates_in_the_tokens =
                condition_predicates_written_in_tokens(&block.code, &lexed.tokens);
            // 文档测试的条件链取它所挂条目的那一串（挂模块的没有）：键里的条件原子照它取，与 lib_function_units.py 的 add_doctest 同
            let conditions = match attachment {
                DocumentationTestAttachment::Item(owner) => self.units[owner.0].conditions.clone(),
                DocumentationTestAttachment::Module(_) => Vec::new(),
            };
            self.units.push(Unit {
                kind: UnitKind::DocumentationTest,
                name: format!("<doctest line {}>", block.opening_line),
                qualified_name: String::new(),
                file: self.file,
                local_module_path: module_path.to_vec(),
                container_label: None,
                start_line: block.opening_line,
                end_line: block.closing_line,
                token_range: None,
                container: None,
                module_head: None,
                conditions,
                condition_predicates_in_the_tokens,
                is_public: false,
                fingerprint,
                references,
                declared_names: Vec::new(),
                imports: None,
                implementation: None,
                module_declaration: None,
                reads: Vec::new(),
                refusals: Vec::new(),
                documentation_test_of: Some(attachment.clone()),
                has_test_attribute: false,
                is_inside_test_only_code: false,
            });
        }
    }

    fn push_unit(&mut self, new_unit: NewUnit, scan_context: &ScanContext) -> UnitIndex {
        let first = new_unit.first_token;
        let end = new_unit.end_token.max(first + 1).min(self.tokens.len());
        let unit_tokens = &self.tokens[first..end];
        let fingerprint = fingerprint_of_tokens(self.source, unit_tokens);
        let signature_end = match new_unit.kind {
            UnitKind::Function => unit_tokens.iter().position(|token| {
                token.kind == TokenKind::Punctuation && token_text(self.source, token) == "{"
            }),
            UnitKind::ImplementationHead
            | UnitKind::TraitHead
            | UnitKind::Constant
            | UnitKind::Static
            | UnitKind::Struct
            | UnitKind::Enumeration
            | UnitKind::Union
            | UnitKind::TypeAlias
            | UnitKind::MacroRulesDefinition
            | UnitKind::Use
            | UnitKind::ItemMacroInvocation
            | UnitKind::ModuleDeclaration
            | UnitKind::ModuleHead
            | UnitKind::ExternBlock
            | UnitKind::ExternCrate
            | UnitKind::VerusBroadcastGroup
            | UnitKind::VerusAssumeSpecification
            | UnitKind::DocumentationTest => None,
        };
        let references = references_of_tokens(self.source, unit_tokens, signature_end);
        let condition_predicates_in_the_tokens =
            condition_predicates_written_in_tokens(self.source, unit_tokens);
        let (reads, mut refusals) = self.reads_and_refusals_of(first, end);
        for form in &new_unit.extra_refusals {
            refusals.push(self.refusal(*form, first, self.fragment(first, end)));
        }
        let declared_names = new_unit
            .declared_names
            .unwrap_or_else(|| vec![new_unit.name.clone()]);
        let has_test_attribute = new_unit.kind == UnitKind::Function
            && (first..new_unit.header_token.min(end)).any(|index| {
                self.text_of(index) == "#"
                    && self.text_of(index + 1) == "["
                    && self.text_of(index + 2) == "test"
                    && self.text_of(index + 3) == "]"
            });
        let is_inside_test_only_code = scan_context
            .conditions
            .iter()
            .any(predicate_requires_the_test_configuration);
        let unit_index = UnitIndex(self.units.len());
        self.units.push(Unit {
            kind: new_unit.kind,
            name: new_unit.name,
            qualified_name: String::new(),
            file: self.file,
            local_module_path: scan_context.module_path.clone(),
            container_label: scan_context.container_label.clone(),
            start_line: self.line_of(first),
            end_line: self.line_of(end - 1),
            token_range: Some((first, end)),
            container: scan_context.container,
            module_head: scan_context.module_head,
            conditions: new_unit.conditions,
            condition_predicates_in_the_tokens,
            is_public: new_unit.is_public,
            fingerprint,
            references,
            declared_names,
            imports: new_unit.imports,
            implementation: new_unit.implementation,
            module_declaration: new_unit.module_declaration,
            reads,
            refusals,
            documentation_test_of: None,
            has_test_attribute,
            is_inside_test_only_code,
        });
        if new_unit.kind != UnitKind::ModuleHead {
            let comments = self.documentation_comments_between(
                first,
                new_unit.header_token,
                DocumentationPlacement::FollowingItem,
            );
            self.push_documentation_tests(
                &comments,
                &DocumentationTestAttachment::Item(unit_index),
                &scan_context.module_path,
            );
        }
        unit_index
    }

    /// 一个模块体从 `low` 起：先收开头的内属性当模块头，再切条目。
    fn scan_module_body(
        &mut self,
        low: usize,
        high: usize,
        outer_context: &ScanContext,
        head_unit: Option<UnitIndex>,
    ) {
        let module_comments = self.documentation_comments_between(
            low,
            low.max(self.inner_attributes_from(low, high).0),
            DocumentationPlacement::EnclosingModule,
        );
        self.push_documentation_tests(
            &module_comments,
            &DocumentationTestAttachment::Module(outer_context.module_path.clone()),
            &outer_context.module_path,
        );
        let mut body_context = outer_context.clone();
        if head_unit.is_some() {
            body_context.module_head = head_unit;
        }
        let (after_inner, _) = self.inner_attributes_from(low, high);
        self.scan_items(after_inner, high, &body_context);
    }

    fn scan_items(&mut self, low: usize, high: usize, scan_context: &ScanContext) {
        let mut index = low;
        while index < high {
            let item_start = index;
            let mut attribute_ranges: Vec<(usize, usize)> = Vec::new();
            while index < high {
                if self.text_of(index) == "#" && self.text_of(index + 1) == "[" {
                    let close = self.close_of(index + 1);
                    attribute_ranges.push((index + 1, close));
                    index = close + 1;
                } else if self.text_of(index) == "#"
                    && self.text_of(index + 1) == "!"
                    && self.text_of(index + 2) == "["
                {
                    index = self.close_of(index + 2) + 1;
                } else if self.is_documentation_token(index) {
                    index += 1;
                } else {
                    break;
                }
            }
            if index >= high {
                break;
            }
            if self.text_of(index) == ";" {
                index += 1;
                continue;
            }
            let end = self.scan_one_item(item_start, index, high, &attribute_ranges, scan_context);
            index = end.max(index + 1);
        }
    }

    fn scan_one_item(
        &mut self,
        item_start: usize,
        header: usize,
        high: usize,
        attribute_ranges: &[(usize, usize)],
        scan_context: &ScanContext,
    ) -> usize {
        let attributes = self.attribute_facts(attribute_ranges);
        let mut conditions = scan_context.conditions.clone();
        conditions.extend(attributes.conditions.iter().cloned());
        let (after_visibility, is_public) = self.skip_visibility(header);
        let keyword_index = self.skip_qualifiers(after_visibility, false);
        let keyword = self.text_of(keyword_index);
        let name_after_keyword = self.name_at(keyword_index + 1);
        match keyword {
            "fn" => {
                let end = self.end_of_item(keyword_index, high, true);
                self.push_unit(
                    NewUnit::plain(
                        UnitKind::Function,
                        name_after_keyword,
                        (item_start, end),
                        header,
                        conditions,
                        is_public,
                    ),
                    scan_context,
                );
                return end;
            }
            "impl" | "trait" => {
                let head_end = self.first_brace_or_semicolon(keyword_index + 1, high);
                if self.text_of(head_end) != "{" {
                    let end = (head_end + 1).min(high);
                    let kind = if keyword == "impl" {
                        UnitKind::ImplementationHead
                    } else {
                        UnitKind::TraitHead
                    };
                    self.push_unit(
                        NewUnit::plain(
                            kind,
                            name_after_keyword,
                            (item_start, end),
                            header,
                            conditions,
                            is_public,
                        ),
                        scan_context,
                    );
                    return end;
                }
                let body_close = self.close_of(head_end);
                let (kind, name, label, implementation) = if keyword == "impl" {
                    let label = implementation_label(
                        &(keyword_index + 1..head_end)
                            .map(|index| self.text_of(index))
                            .collect::<Vec<&str>>(),
                    );
                    (
                        UnitKind::ImplementationHead,
                        label.clone(),
                        label,
                        Some(self.implementation_shape(keyword_index + 1, head_end)),
                    )
                } else {
                    (
                        UnitKind::TraitHead,
                        name_after_keyword.clone(),
                        format!("<trait {name_after_keyword}>"),
                        None,
                    )
                };
                let mut new_unit = NewUnit::plain(
                    kind,
                    name,
                    (item_start, head_end),
                    header,
                    conditions.clone(),
                    is_public,
                );
                new_unit.implementation = implementation;
                let head = self.push_unit(new_unit, scan_context);
                let member_context = ScanContext {
                    module_path: scan_context.module_path.clone(),
                    container: Some(head),
                    container_label: Some(label),
                    module_head: scan_context.module_head,
                    conditions,
                };
                self.scan_items(head_end + 1, body_close, &member_context);
                return body_close + 1;
            }
            "mod" if self.is_identifier(keyword_index + 1) => {
                if self.text_of(keyword_index + 2) == ";" {
                    let end = keyword_index + 3;
                    let mut new_unit = NewUnit::plain(
                        UnitKind::ModuleDeclaration,
                        name_after_keyword.clone(),
                        (item_start, end),
                        header,
                        conditions,
                        is_public,
                    );
                    new_unit.module_declaration = Some(ModuleDeclarationShape {
                        module_name: name_after_keyword,
                        path_attribute: attributes.path_attribute,
                    });
                    self.push_unit(new_unit, scan_context);
                    return end;
                }
                if self.text_of(keyword_index + 2) == "{" {
                    let open = keyword_index + 2;
                    let close = self.close_of(open);
                    let (inner_end, inner_ranges) = self.inner_attributes_from(open + 1, close);
                    let inner_facts = self.attribute_facts(&inner_ranges);
                    conditions.extend(inner_facts.conditions);
                    let head = self.push_unit(
                        NewUnit::plain(
                            UnitKind::ModuleHead,
                            name_after_keyword.clone(),
                            (item_start, inner_end),
                            header,
                            conditions.clone(),
                            is_public,
                        ),
                        scan_context,
                    );
                    let mut module_path = scan_context.module_path.clone();
                    module_path.push(name_after_keyword);
                    let body_context = ScanContext {
                        module_path,
                        container: None,
                        container_label: None,
                        module_head: Some(head),
                        conditions,
                    };
                    self.scan_module_body(open + 1, close, &body_context, Some(head));
                    return close + 1;
                }
            }
            "macro_rules"
                if self.text_of(keyword_index + 1) == "!"
                    && self.is_open_bracket(keyword_index + 3) =>
            {
                let close = self.close_of(keyword_index + 3);
                let mut end = close + 1;
                if self.text_of(end) == ";" {
                    end += 1;
                }
                self.push_unit(
                    NewUnit::plain(
                        UnitKind::MacroRulesDefinition,
                        self.name_at(keyword_index + 2),
                        (item_start, end),
                        header,
                        conditions,
                        attributes.is_macro_export,
                    ),
                    scan_context,
                );
                return end;
            }
            "use" => {
                let end = self.end_of_item(keyword_index, high, false);
                let mut new_unit = NewUnit::plain(
                    UnitKind::Use,
                    format!("use@{}", self.line_of(keyword_index)),
                    (item_start, end),
                    header,
                    conditions,
                    is_public,
                );
                new_unit.imports = Some(self.use_imports(keyword_index + 1, end));
                self.push_unit(new_unit, scan_context);
                return end;
            }
            "struct" | "enum" | "type" | "const" | "static" => {
                let kind = match keyword {
                    "struct" => UnitKind::Struct,
                    "enum" => UnitKind::Enumeration,
                    "type" => UnitKind::TypeAlias,
                    "const" => UnitKind::Constant,
                    _ => UnitKind::Static,
                };
                let braced = matches!(kind, UnitKind::Struct | UnitKind::Enumeration);
                let name = if keyword == "static" && self.text_of(keyword_index + 1) == "mut" {
                    self.name_at(keyword_index + 2)
                } else {
                    name_after_keyword
                };
                let end = self.end_of_item(keyword_index, high, braced);
                self.push_unit(
                    NewUnit::plain(kind, name, (item_start, end), header, conditions, is_public),
                    scan_context,
                );
                return end;
            }
            "union"
                if self.is_identifier(keyword_index + 1)
                    && matches!(self.text_of(keyword_index + 2), "{" | "<") =>
            {
                let end = self.end_of_item(keyword_index, high, true);
                self.push_unit(
                    NewUnit::plain(
                        UnitKind::Union,
                        name_after_keyword,
                        (item_start, end),
                        header,
                        conditions,
                        is_public,
                    ),
                    scan_context,
                );
                return end;
            }
            "extern" => {
                if self.text_of(keyword_index + 1) == "crate" {
                    let end = self.end_of_item(keyword_index, high, false);
                    self.push_unit(
                        NewUnit::plain(
                            UnitKind::ExternCrate,
                            self.name_at(keyword_index + 2),
                            (item_start, end),
                            header,
                            conditions,
                            is_public,
                        ),
                        scan_context,
                    );
                    return end;
                }
                let open = self.first_brace_or_semicolon(keyword_index + 1, high);
                if self.text_of(open) == "{" {
                    let close = self.close_of(open);
                    let mut new_unit = NewUnit::plain(
                        UnitKind::ExternBlock,
                        format!("extern@{}", self.line_of(keyword_index)),
                        (item_start, close + 1),
                        header,
                        conditions,
                        true,
                    );
                    new_unit.declared_names = Some(self.extern_block_declared_names(open, close));
                    new_unit
                        .extra_refusals
                        .push(RefusalForm::ExternalFunctionInterface);
                    self.push_unit(new_unit, scan_context);
                    return close + 1;
                }
            }
            _ => {}
        }
        if let Some(invocation) = self.macro_invocation_at(keyword_index) {
            return self.scan_item_macro_invocation(
                item_start,
                header,
                &invocation,
                conditions,
                scan_context,
            );
        }
        let end = self.end_of_item(header, high, true);
        self.file_level_refusals.push(self.refusal(
            RefusalForm::UnrecognizedItemStart,
            header,
            self.fragment(header, end),
        ));
        end
    }

    fn scan_item_macro_invocation(
        &mut self,
        item_start: usize,
        header: usize,
        invocation: &MacroInvocationShape,
        conditions: Vec<Predicate>,
        scan_context: &ScanContext,
    ) -> usize {
        let open = invocation.open_bracket;
        let close = self.close_of(open);
        let mut end = close + 1;
        if self.text_of(end) == ";" {
            end += 1;
        }
        let is_braced = self.text_of(open) == "{";
        let inner_context = ScanContext {
            module_path: scan_context.module_path.clone(),
            container: scan_context.container,
            container_label: scan_context.container_label.clone(),
            module_head: scan_context.module_head,
            conditions: conditions.clone(),
        };
        match invocation.macro_name.as_str() {
            "verus" if is_braced => {
                if self.switches.is_intact(BrokenRule::VerusClauseBraces) {
                    self.scan_verus_items(open + 1, close, &inner_context);
                } else {
                    self.scan_items(open + 1, close, &inner_context);
                }
                return end;
            }
            "thread_local" if is_braced => {
                self.scan_items(open + 1, close, &inner_context);
                return end;
            }
            _ => {}
        }
        let mut new_unit = NewUnit::plain(
            UnitKind::ItemMacroInvocation,
            format!("{}!@{}", invocation.macro_name, self.line_of(header)),
            (item_start, end),
            header,
            conditions,
            true,
        );
        new_unit.declared_names = Some(self.item_macro_declared_names(open, close));
        match invocation.macro_name.as_str() {
            "include" => self.file_level_refusals.push(self.refusal(
                RefusalForm::ItemLevelInclude,
                header,
                self.fragment(header, end),
            )),
            "paste" | "concat_idents" => new_unit
                .extra_refusals
                .push(RefusalForm::IdentifierPastingMacro),
            _ => {}
        }
        self.push_unit(new_unit, scan_context);
        end
    }

    fn is_verus_item_start(&self, index: usize) -> bool {
        if self.is_documentation_token(index) {
            return true;
        }
        if !self.is_identifier(index) && self.text_of(index) != "#" {
            return false;
        }
        match self.text_of(index) {
            "#" => self.text_of(index + 1) == "[",
            "pub" | "fn" | "spec" | "proof" | "exec" | "open" | "closed" | "broadcast"
            | "struct" | "enum" | "union" | "impl" | "trait" | "const" | "static" | "type"
            | "use" | "mod" | "unsafe" | "extern" | "macro_rules" | "global" | "tracked"
            | "ghost" | "uninterp" | "axiom" | "async" | "default" => true,
            "assume_specification" => self.is_assume_specification_keyword(index),
            _ => self.text_of(index + 1) == "!" && self.is_open_bracket(index + 2),
        }
    }

    /// `assume_specification` 这个词后面接 `[`（点名的路径）或 `<`（泛型参数表）：Verus 的 `assume_specification` 条目的关键字。
    fn is_assume_specification_keyword(&self, index: usize) -> bool {
        self.text_of(index) == "assume_specification"
            && self.kind_of(index + 1) == Some(TokenKind::Punctuation)
            && matches!(self.text_of(index + 1), "[" | "<")
    }

    /// 从 `open_angle` 那个 `<` 起配到它的 `>`，交 `>` 后面那个词法单元的位置；`->` 的 `>` 不算。配不上（`end` 之前没回到零层）交 `None`。
    fn after_the_angle_brackets(&self, open_angle: usize, end: usize) -> Option<usize> {
        let mut depth = 0usize;
        // 每一步往后走一个词法单元：至多走到条目末尾
        for position in open_angle..end {
            match self.text_of(position) {
                "<" => depth += 1,
                ">" if position == 0 || self.text_of(position - 1) != "-" => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(position + 1);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// `assume_specification` 之后（可带一段 `<…>` 泛型参数表）那对方括号里点名的路径，末段的名字：
    /// `[ singlefs_format::WIDE_CHECKSUM_BYTES ]` 是 `WIDE_CHECKSUM_BYTES`，`[ Vec::<Element>::len ]` 是 `len`（尖括号里的名字不算）。
    /// 认不出那对方括号、方括号里尖括号外一个名字都没有的，交 `None`。
    fn assume_specification_target_name(
        &self,
        after_the_keyword: usize,
        end: usize,
    ) -> Option<String> {
        let open_bracket = if self.text_of(after_the_keyword) == "<" {
            self.after_the_angle_brackets(after_the_keyword, end)?
        } else {
            after_the_keyword
        };
        if self.text_of(open_bracket) != "[" || !self.is_open_bracket(open_bracket) {
            return None;
        }
        let close_bracket = self.close_of(open_bracket);
        let mut angle_depth = 0usize;
        let mut last_name_outside_the_angle_brackets = None;
        // 每一步往后走一个词法单元：至多走到配对的 `]`
        for position in open_bracket + 1..close_bracket {
            match self.text_of(position) {
                "<" => angle_depth += 1,
                ">" if self.text_of(position - 1) != "-" => {
                    angle_depth = angle_depth.saturating_sub(1);
                }
                _ => {
                    if angle_depth == 0 && self.is_identifier(position) {
                        last_name_outside_the_angle_brackets = Some(self.name_at(position));
                    }
                }
            }
        }
        last_name_outside_the_angle_brackets
    }

    /// `verus!` 块里同层条目的起头：块首，或顶层的 `;` / `}` 之后紧接一个条目关键字（或属性）的位置。
    fn verus_item_starts(&self, low: usize, high: usize) -> Vec<usize> {
        let mut starts = Vec::new();
        let mut is_at_boundary = true;
        let mut index = low;
        while index < high {
            if is_at_boundary && self.is_verus_item_start(index) {
                starts.push(index);
            }
            if self.is_open_bracket(index) {
                let close = self.close_of(index);
                is_at_boundary = self.text_of(close) == "}";
                index = close + 1;
                continue;
            }
            is_at_boundary = self.text_of(index) == ";";
            index += 1;
        }
        starts
    }

    fn scan_verus_items(&mut self, low: usize, high: usize, scan_context: &ScanContext) {
        let starts = self.verus_item_starts(low, high);
        if starts.first().is_some_and(|first| *first > low) || (starts.is_empty() && low < high) {
            let stray_end = starts.first().copied().unwrap_or(high);
            self.file_level_refusals.push(self.refusal(
                RefusalForm::UnrecognizedItemStart,
                low,
                self.fragment(low, stray_end),
            ));
        }
        for (position, start) in starts.iter().enumerate() {
            let end = starts.get(position + 1).copied().unwrap_or(high);
            self.scan_one_verus_item(*start, end, scan_context);
        }
    }

    fn scan_one_verus_item(&mut self, start: usize, end: usize, scan_context: &ScanContext) {
        let mut index = start;
        let mut attribute_ranges = Vec::new();
        while index < end {
            if self.text_of(index) == "#" && self.text_of(index + 1) == "[" {
                let close = self.close_of(index + 1);
                attribute_ranges.push((index + 1, close));
                index = close + 1;
            } else if self.is_documentation_token(index) {
                index += 1;
            } else {
                break;
            }
        }
        let header = index;
        let attributes = self.attribute_facts(&attribute_ranges);
        let mut conditions = scan_context.conditions.clone();
        conditions.extend(attributes.conditions.iter().cloned());
        let (after_visibility, is_public) = self.skip_visibility(header);
        let keyword_index = self.skip_qualifiers(after_visibility, true);
        let keyword = self.text_of(keyword_index);
        let name_after_keyword = self.name_at(keyword_index + 1);
        let whole = (start, end);
        let simple_kind = match keyword {
            "fn" => Some(UnitKind::Function),
            "struct" => Some(UnitKind::Struct),
            "enum" => Some(UnitKind::Enumeration),
            "union" => Some(UnitKind::Union),
            "type" => Some(UnitKind::TypeAlias),
            "const" => Some(UnitKind::Constant),
            "static" => Some(UnitKind::Static),
            _ => None,
        };
        if let Some(kind) = simple_kind {
            let name = if keyword == "static" && self.text_of(keyword_index + 1) == "mut" {
                self.name_at(keyword_index + 2)
            } else {
                name_after_keyword
            };
            self.push_unit(
                NewUnit::plain(kind, name, whole, header, conditions, is_public),
                scan_context,
            );
            return;
        }
        match keyword {
            "impl" | "trait" => {
                let head_end = self.first_brace_or_semicolon(keyword_index + 1, end);
                let (kind, name, label, implementation) = if keyword == "impl" {
                    let label = implementation_label(
                        &(keyword_index + 1..head_end)
                            .map(|position| self.text_of(position))
                            .collect::<Vec<&str>>(),
                    );
                    (
                        UnitKind::ImplementationHead,
                        label.clone(),
                        label,
                        Some(self.implementation_shape(keyword_index + 1, head_end)),
                    )
                } else {
                    (
                        UnitKind::TraitHead,
                        name_after_keyword.clone(),
                        format!("<trait {name_after_keyword}>"),
                        None,
                    )
                };
                let mut new_unit = NewUnit::plain(
                    kind,
                    name,
                    (start, head_end),
                    header,
                    conditions.clone(),
                    is_public,
                );
                new_unit.implementation = implementation;
                let head = self.push_unit(new_unit, scan_context);
                if self.text_of(head_end) == "{" {
                    let body_close = self.close_of(head_end);
                    let member_context = ScanContext {
                        module_path: scan_context.module_path.clone(),
                        container: Some(head),
                        container_label: Some(label),
                        module_head: scan_context.module_head,
                        conditions,
                    };
                    self.scan_verus_items(head_end + 1, body_close, &member_context);
                }
            }
            "use" => {
                let mut new_unit = NewUnit::plain(
                    UnitKind::Use,
                    format!("use@{}", self.line_of(keyword_index)),
                    whole,
                    header,
                    conditions,
                    is_public,
                );
                new_unit.imports = Some(self.use_imports(keyword_index + 1, end));
                self.push_unit(new_unit, scan_context);
            }
            "broadcast" if self.text_of(keyword_index + 1) == "use" => {
                let mut new_unit = NewUnit::plain(
                    UnitKind::Use,
                    format!("broadcast-use@{}", self.line_of(keyword_index)),
                    whole,
                    header,
                    conditions,
                    is_public,
                );
                new_unit.imports = Some(self.use_imports(keyword_index + 2, end));
                self.push_unit(new_unit, scan_context);
            }
            "broadcast" if self.text_of(keyword_index + 1) == "group" => {
                self.push_unit(
                    NewUnit::plain(
                        UnitKind::VerusBroadcastGroup,
                        self.name_at(keyword_index + 2),
                        whole,
                        header,
                        conditions,
                        is_public,
                    ),
                    scan_context,
                );
            }
            "mod" if self.text_of(keyword_index + 2) == ";" => {
                let mut new_unit = NewUnit::plain(
                    UnitKind::ModuleDeclaration,
                    name_after_keyword.clone(),
                    whole,
                    header,
                    conditions,
                    is_public,
                );
                new_unit.module_declaration = Some(ModuleDeclarationShape {
                    module_name: name_after_keyword,
                    path_attribute: attributes.path_attribute,
                });
                self.push_unit(new_unit, scan_context);
            }
            "mod" if self.text_of(keyword_index + 2) == "{" => {
                let open = keyword_index + 2;
                let close = self.close_of(open);
                let head = self.push_unit(
                    NewUnit::plain(
                        UnitKind::ModuleHead,
                        name_after_keyword.clone(),
                        (start, open + 1),
                        header,
                        conditions.clone(),
                        is_public,
                    ),
                    scan_context,
                );
                let mut module_path = scan_context.module_path.clone();
                module_path.push(name_after_keyword);
                let body_context = ScanContext {
                    module_path,
                    container: None,
                    container_label: None,
                    module_head: Some(head),
                    conditions,
                };
                self.scan_verus_items(open + 1, close, &body_context);
            }
            "assume_specification" if self.is_assume_specification_keyword(keyword_index) => {
                // 按点名的那一项的名字找得到它；名字认不出时这份文件里谁连得到它说不全，照认不出的条目起头记文件级拒算
                let Some(target_name) =
                    self.assume_specification_target_name(keyword_index + 1, end)
                else {
                    self.file_level_refusals.push(self.refusal(
                        RefusalForm::UnrecognizedItemStart,
                        header,
                        self.fragment(header, end),
                    ));
                    return;
                };
                let mut new_unit = NewUnit::plain(
                    UnitKind::VerusAssumeSpecification,
                    format!("assume_specification@{}", self.line_of(keyword_index)),
                    whole,
                    header,
                    conditions,
                    is_public,
                );
                new_unit.declared_names = Some(vec![target_name]);
                self.push_unit(new_unit, scan_context);
            }
            "macro_rules" => {
                self.push_unit(
                    NewUnit::plain(
                        UnitKind::MacroRulesDefinition,
                        self.name_at(keyword_index + 2),
                        whole,
                        header,
                        conditions,
                        attributes.is_macro_export,
                    ),
                    scan_context,
                );
            }
            _ => {
                if let Some(invocation) = self.macro_invocation_at(keyword_index) {
                    let mut new_unit = NewUnit::plain(
                        UnitKind::ItemMacroInvocation,
                        format!("{}!@{}", invocation.macro_name, self.line_of(header)),
                        whole,
                        header,
                        conditions,
                        true,
                    );
                    let close = self.close_of(invocation.open_bracket);
                    new_unit.declared_names =
                        Some(self.item_macro_declared_names(invocation.open_bracket, close));
                    self.push_unit(new_unit, scan_context);
                } else {
                    self.file_level_refusals.push(self.refusal(
                        RefusalForm::UnrecognizedItemStart,
                        header,
                        self.fragment(header, end),
                    ));
                }
            }
        }
    }
}

/// 拒算行的原样片段：制表符、换行换成空格，截到上限。
#[must_use]
pub fn cleaned_fragment(text: &str) -> String {
    let flattened: String = text
        .chars()
        .map(|character| {
            if character == '\t' || character == '\n' || character == '\r' {
                ' '
            } else {
                character
            }
        })
        .collect();
    flattened.chars().take(FRAGMENT_CHARACTER_LIMIT).collect()
}

/// impl 头的标签（限定名里用）：`impl` 之后的词法单元，去掉紧跟 `impl` 的泛型参数表与 `where` 子句，逐个用一个空格连起来，
/// 相邻两个 `:` 连成 `::`、两边不留空格；例 `<impl std::ops::Add for Money>`、`<impl Wrapper < Inner >>`。
/// 与 `scripts/gate/lib_function_units.py` 的 `impl_head` 同一种写法（两份实现逐单元对拍限定名）。
#[must_use]
pub fn implementation_label(texts_after_the_keyword: &[&str]) -> String {
    let mut texts = texts_after_the_keyword;
    if texts.first() == Some(&"<") {
        let mut depth = 0usize;
        for (position, text) in texts.iter().enumerate() {
            let is_arrow_head = position > 0 && texts[position - 1] == "-";
            match *text {
                "<" => depth += 1,
                ">" if !is_arrow_head => depth = depth.saturating_sub(1),
                _ => {}
            }
            if depth == 0 {
                texts = &texts[position + 1..];
                break;
            }
        }
    }
    if let Some(where_position) = texts.iter().position(|text| *text == "where") {
        texts = &texts[..where_position];
    }
    let mut rendered = String::from("<impl ");
    for (position, text) in texts.iter().enumerate() {
        let is_path_separator_colon = *text == ":"
            && ((position > 0 && texts[position - 1] == ":")
                || texts.get(position + 1) == Some(&":"));
        let follows_path_separator =
            position >= 2 && texts[position - 1] == ":" && texts[position - 2] == ":";
        if position > 0 && !is_path_separator_colon && !follows_path_separator {
            rendered.push(' ');
        }
        rendered.push_str(text);
    }
    rendered.push('>');
    rendered
}

/// 切一份 Rust 文件。
#[must_use]
pub fn parse_rust_file(input: &RustFileInput) -> ParsedRustFile {
    let treatment = if input
        .switches
        .is_intact(BrokenRule::DocumentationCommentsInFingerprint)
    {
        DocumentationCommentTreatment::KeptOutOfTheTokens
    } else {
        DocumentationCommentTreatment::KeptAsTokens
    };
    let lexed = lex_source(input.source, treatment);
    let closes = match matching_brackets(input.source, &lexed.tokens) {
        Ok(closes) => closes,
        Err(offending_index) => {
            let line = lexed
                .tokens
                .get(offending_index)
                .map_or(1, |token| token.line);
            let fragment = lexed
                .tokens
                .get(offending_index)
                .map_or(String::new(), |token| {
                    token_text(input.source, token).to_string()
                });
            return ParsedRustFile {
                tokens: lexed.tokens,
                units: Vec::new(),
                file_level_refusals: vec![Refusal {
                    file: crate::units::RefusalFile::WorkspaceSourceFile(input.file),
                    line,
                    form: RefusalForm::UnbalancedBrackets,
                    fragment,
                }],
            };
        }
    };
    let mut scanner = RustScanner {
        source: input.source,
        file: input.file,
        file_directory: input.file_directory,
        manifest_directory: input.manifest_directory,
        switches: input.switches,
        tokens: &lexed.tokens,
        closes: &closes,
        documentation_comments: &lexed.documentation_comments,
        manifest_directory_constants: private_manifest_directory_constants(
            input.source,
            &lexed.tokens,
        ),
        units: Vec::new(),
        file_level_refusals: Vec::new(),
    };
    let token_count = lexed.tokens.len();
    let (after_inner, inner_ranges) = scanner.inner_attributes_from(0, token_count);
    let file_facts = scanner.attribute_facts(&inner_ranges);
    let file_context = ScanContext {
        module_path: Vec::new(),
        container: None,
        container_label: None,
        module_head: None,
        conditions: file_facts.conditions.clone(),
    };
    let head = if inner_ranges.is_empty() {
        None
    } else {
        Some(scanner.push_unit(
            NewUnit::plain(
                UnitKind::ModuleHead,
                "<module attributes>".to_string(),
                (0, after_inner),
                0,
                file_facts.conditions,
                false,
            ),
            &file_context,
        ))
    };
    scanner.scan_module_body(0, token_count, &file_context, head);
    let units = scanner.units;
    let file_level_refusals = scanner.file_level_refusals;
    ParsedRustFile {
        tokens: lexed.tokens,
        units,
        file_level_refusals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> ParsedRustFile {
        let switches = RuleSwitches::every_rule_intact();
        parse_rust_file(&RustFileInput {
            file: FileIndex(0),
            source,
            file_directory: Path::new("/nonexistent"),
            manifest_directory: None,
            switches: &switches,
        })
    }

    fn kinds_and_names(parsed: &ParsedRustFile) -> Vec<(UnitKind, String)> {
        parsed
            .units
            .iter()
            .map(|unit| (unit.kind, unit.name.clone()))
            .collect()
    }

    /// `use a::b as c;`：进当前模块可用名字集的只有别名 `c`，原名 `b` 不进（早先 `b` 也被记进去，别处同名的条目会被误认成命中了这条 use）。
    #[test]
    fn a_use_with_an_alias_brings_only_the_alias_into_the_module_names() {
        let parsed = parse("use a::b as c;\n");
        let imports = parsed.units[0]
            .imports
            .as_ref()
            .expect("use 条目带导入清单");
        assert_eq!(imports.names, ["c"]);
        assert_eq!(imports.aliases, [("c".to_string(), "b".to_string())]);
    }

    #[test]
    fn items_members_and_heads_become_separate_units() {
        let parsed = parse(
            "use std::fmt;\npub const WIDTH: usize = 4;\npub struct Money(pub u32);\n\
             impl std::ops::Add for Money { type Output = Money; fn add(self, other: Money) -> Money { Money(self.0 + other.0) } }\n\
             pub trait Limit { const LIMIT: u32; fn doubled(&self) -> u32 { 2 } }\n\
             macro_rules! make { () => {} }\nmake!();\nmod nested { pub fn inside() {} }\nmod declared;\n",
        );
        assert!(
            parsed.file_level_refusals.is_empty(),
            "全都认得：{:?}",
            parsed.file_level_refusals
        );
        assert_eq!(
            kinds_and_names(&parsed),
            [
                (UnitKind::Use, "use@1".to_string()),
                (UnitKind::Constant, "WIDTH".to_string()),
                (UnitKind::Struct, "Money".to_string()),
                (
                    UnitKind::ImplementationHead,
                    "<impl std::ops::Add for Money>".to_string()
                ),
                (UnitKind::TypeAlias, "Output".to_string()),
                (UnitKind::Function, "add".to_string()),
                (UnitKind::TraitHead, "Limit".to_string()),
                (UnitKind::Constant, "LIMIT".to_string()),
                (UnitKind::Function, "doubled".to_string()),
                (UnitKind::MacroRulesDefinition, "make".to_string()),
                (UnitKind::ItemMacroInvocation, "make!@7".to_string()),
                (UnitKind::ModuleHead, "nested".to_string()),
                (UnitKind::Function, "inside".to_string()),
                (UnitKind::ModuleDeclaration, "declared".to_string()),
            ]
        );
        let implementation = parsed.units[3]
            .implementation
            .as_ref()
            .expect("impl 头有形状");
        assert_eq!(implementation.self_type_names, ["Money"]);
        assert!(implementation.is_trait_implementation);
        assert_eq!(
            parsed.units[5].container,
            Some(UnitIndex(3)),
            "方法挂在它的 impl 头下"
        );
    }

    #[test]
    fn a_path_qualified_item_macro_invocation_is_one_unit_that_declares_its_names() {
        let parsed = parse("singlefs_verification::enum_listing_every_member! {\n    #[derive(Debug)]\n    pub enum Color { Red, Green }\n}\n");
        assert!(
            parsed.file_level_refusals.is_empty(),
            "带路径的条目级宏调用认得：{:?}",
            parsed.file_level_refusals
        );
        assert_eq!(parsed.units.len(), 1);
        assert_eq!(parsed.units[0].kind, UnitKind::ItemMacroInvocation);
        assert_eq!(
            parsed.units[0].declared_names,
            ["Color", "Debug", "Green", "Red", "derive"],
            "宏调用里的标识符都算可能定义的名字（属性里的也算），只去掉关键字 pub、enum"
        );
    }

    #[test]
    fn an_item_macro_invocation_declares_identifiers_after_a_dot_and_primitive_type_names_but_no_keywords(
    ) {
        let parsed = parse(
            "make_items! {
    fn widen(value: u8) -> u64 { value.into() }
}
",
        );
        assert_eq!(parsed.units.len(), 1);
        assert_eq!(
            parsed.units[0].declared_names,
            ["into", "u64", "u8", "value", "widen"],
            "fn 是关键字、不算；`.` 后面的 into 与基本类型名 u8、u64 都算"
        );
    }

    #[test]
    fn an_implementation_label_drops_the_generic_parameters_and_where_clause_and_spaces_every_token(
    ) {
        let parsed = parse(
            "impl<'flow, Device: core::fmt::Debug> std::ops::Deref for Flow<'flow, Device> where Device: Clone { type Target = u8; }\n\
             impl<Value> From<Vec<Value>> for Wrapper<Value> {}\n",
        );
        let labels: Vec<&str> = parsed
            .units
            .iter()
            .filter(|unit| unit.kind == UnitKind::ImplementationHead)
            .map(|unit| unit.name.as_str())
            .collect();
        assert_eq!(
            labels,
            [
                "<impl std::ops::Deref for Flow < 'flow , Device >>",
                "<impl From < Vec < Value > > for Wrapper < Value >>",
            ],
            "与 lib_function_units.py 的 impl 标签同一写法：去掉 impl 后的泛型参数表与 where，词法单元之间一个空格，:: 两边不留空格"
        );
    }

    #[test]
    fn a_verus_ensures_clause_with_braces_stays_inside_its_function_unit() {
        let parsed = parse(
            "verus! {\npub fn chosen(value: u64) -> (result: u64)\n    ensures if value > 0 { result == value } else { result == 0 },\n{\n    value\n}\n\
             pub open spec fn never_called() -> bool { true }\n}\n",
        );
        assert!(
            parsed.file_level_refusals.is_empty(),
            "verus! 里 ensures 带花括号也认得：{:?}",
            parsed.file_level_refusals
        );
        assert_eq!(
            kinds_and_names(&parsed),
            [
                (UnitKind::Function, "chosen".to_string()),
                (UnitKind::Function, "never_called".to_string()),
            ]
        );
        assert_eq!(
            parsed.units[0].end_line, 6,
            "chosen 这个单元到它的函数体闭上为止"
        );
    }

    #[test]
    fn documentation_comments_stay_out_of_the_fingerprint_and_code_blocks_become_documentation_tests(
    ) {
        let with_documentation = parse("/// 说明\n/// ```\n/// assert_eq!(crate_name::value(), 1);\n/// ```\npub fn value() -> u32 { 1 }\n");
        let without_documentation = parse("pub fn value() -> u32 { 1 }\n");
        assert_eq!(
            with_documentation.units[0].fingerprint, without_documentation.units[0].fingerprint,
            "文档注释不进条目指纹"
        );
        assert_eq!(
            with_documentation.units.len(),
            2,
            "函数一个单元，文档里的代码块一个文档测试单元"
        );
        assert_eq!(
            with_documentation.units[1].kind,
            UnitKind::DocumentationTest
        );
        assert_eq!(
            with_documentation.units[1].documentation_test_of,
            Some(DocumentationTestAttachment::Item(UnitIndex(0)))
        );
    }

    #[test]
    fn an_unknown_item_start_and_unbalanced_brackets_are_refused_for_the_whole_file() {
        let unknown = parse("pub fn fine() {}\nwhat is this;\n");
        assert_eq!(unknown.file_level_refusals.len(), 1);
        assert_eq!(
            unknown.file_level_refusals[0].form,
            RefusalForm::UnrecognizedItemStart
        );
        let unbalanced = parse("fn broken() { (\n");
        assert_eq!(
            unbalanced.file_level_refusals[0].form,
            RefusalForm::UnbalancedBrackets
        );
    }

    #[test]
    fn format_string_captures_count_as_bare_names() {
        let parsed = parse(
            "fn show() -> String { format!(\"{DATA_POINTER_BYTES} {{literal}} {width:>4}\") }\n",
        );
        let names = &parsed.units[0].references.plain_names;
        assert!(names.contains(&"DATA_POINTER_BYTES".to_string()));
        assert!(names.contains(&"width".to_string()));
        assert!(!names.contains(&"literal".to_string()), "{{ 转义的不是捕获");
    }

    #[test]
    fn a_capture_right_after_an_escaped_brace_still_counts_as_a_bare_name() {
        let parsed = parse("fn show() -> String { format!(\"{{{WRAPPED_WIDTH}}}\") }\n");
        assert!(
            parsed.units[0]
                .references
                .plain_names
                .contains(&"WRAPPED_WIDTH".to_string()),
            "{{{{{{名}}}}}} 是字面的花括号夹着一个捕获，捕获的名字要收"
        );
    }
}
