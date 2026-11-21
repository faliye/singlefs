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

mod child_processes_and_own_program;
mod documentation_tests;
mod file_reads_and_path_folds;
mod implementations_and_uses;
mod item_macro_invocations_and_extern_blocks;
mod references_of_a_unit;
mod verus_items;

pub use child_processes_and_own_program::*;
use file_reads_and_path_folds::*;
pub use implementations_and_uses::*;
pub use references_of_a_unit::*;

/// 词法单元之间的分隔字节：单元串按它连起来取 SHA-256。
pub const UNIT_SEPARATOR: u8 = 0x1f;

/// 拒算行的原样片段最多取几个词法单元。
const FRAGMENT_TOKEN_LIMIT: usize = 12;
/// 拒算行的原样片段最多几个字符。
const FRAGMENT_CHARACTER_LIMIT: usize = 160;

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
    /// 工作区的仓根（规范化的绝对路径）：目录数据单元只收 `<仓根>/crates/` 之下的目录，清单目录不在 `crates/` 下的成员
    /// （仓根下一层的包、仓根自己那个包）拼出自己的目录照样拒算。
    pub repository_root: &'input Path,
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
    /// `extern` 块只声明登记的 C 库调用：`push_unit` 记一条 [`SourceRead::ExternBlockOfListedSystemLibraryCalls`]。
    is_extern_block_of_listed_system_library_calls: bool,
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
            is_extern_block_of_listed_system_library_calls: false,
        }
    }
}

/// `extern` 块单元的词法单元（从它的外属性起）里，`extern` 之前的属性有没有 `link` 这个词：`#[link(…)]`、
/// `#[cfg_attr(条件, link(…))]` 里的都算（`link_name` 是别的词，不算）。带它的块改链接的库。
pub(crate) fn extern_block_attributes_name_link(source: &str, block_tokens: &[Token]) -> bool {
    block_tokens
        .iter()
        .take_while(|token| {
            !(token.kind == TokenKind::Identifier && token_text(source, token) == "extern")
        })
        .any(|token| token.kind == TokenKind::Identifier && token_text(source, token) == "link")
}

/// 条目级宏调用的位置：最后一段名字、`!` 与开括号。
struct MacroInvocationShape {
    macro_name: String,
    open_bracket: usize,
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

struct RustScanner<'scan> {
    source: &'scan str,
    file: FileIndex,
    file_directory: &'scan Path,
    manifest_directory: Option<&'scan Path>,
    repository_root: &'scan Path,
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

    fn arrow_close(&self, index: usize) -> bool {
        index > 0
            && self.text_of(index - 1) == "-"
            && self.tokens[index - 1].end_byte == self.tokens[index].start_byte
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
        let (mut reads, mut refusals) = self.reads_and_refusals_of(first, end);
        for form in &new_unit.extra_refusals {
            refusals.push(self.refusal(*form, first, self.fragment(first, end)));
        }
        if new_unit.is_extern_block_of_listed_system_library_calls {
            reads.push(SourceRead::ExternBlockOfListedSystemLibraryCalls {
                line: self.line_of(first),
                fragment: self.fragment(first, end),
            });
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
                    if self.extern_block_declares_only_listed_system_library_calls(
                        item_start,
                        keyword_index,
                        open,
                        close,
                    ) {
                        new_unit.is_extern_block_of_listed_system_library_calls = true;
                    } else {
                        new_unit
                            .extra_refusals
                            .push(RefusalForm::ExternalFunctionInterface);
                    }
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
        repository_root: input.repository_root,
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
mod tests;
