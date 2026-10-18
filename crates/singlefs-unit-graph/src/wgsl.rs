//! WGSL 切成单元（设计 3.5）：`fn`、`@compute` 入口、`const`、`override`、`struct`、`alias`、`var<…>` 绑定声明，
//! 外加模块级的 `const_assert` 与 `enable` / `requires` / `diagnostic` 指令（它们不定义名字，但改了就改了着色器能不能编，
//! 所以模块一进闭包它们就进）。去 `//` 与可嵌套的 `/* */`，切词器与 Rust 那一份同一个。
//!
//! 模块从 Rust 闭包里 `include_str!` 读到的那几份 `.wgsl` 组成；WGSL 没有重载、动态分派与宏，从 `@compute` 入口按名字精确收。
//! 用到而这几份 `.wgsl` 里没定义的名字交给调用方：链接集里别的 `.wgsl` 定义了它的，调用方把读那一份的单元连同那一份并进模块再收；
//! 哪份都没定义的（Rust 侧拼进来的常量前缀）要在 Rust 闭包里有着落，否则拒算。

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::OnceLock;

use crate::lexer::{
    identifier_name, lex_source, matching_brackets, token_text, DocumentationCommentTreatment,
    Token, TokenKind,
};
use crate::rust_items::{cleaned_fragment, fingerprint_of_tokens};
use crate::sha256::Sha256Digest;
use crate::units::{FileIndex, Refusal, RefusalForm, WgslUnitIndex};

/// WGSL 单元的种类。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WgslUnitKind {
    Function,
    ComputeEntry,
    Constant,
    Override,
    Struct,
    Alias,
    Variable,
    ConstantAssertion,
    Directive,
}

impl WgslUnitKind {
    /// 输出行与复用键里的种类名。
    #[must_use]
    pub fn output_name(self) -> &'static str {
        match self {
            WgslUnitKind::Function => "wgsl_fn",
            WgslUnitKind::ComputeEntry => "wgsl_compute",
            WgslUnitKind::Constant => "wgsl_const",
            WgslUnitKind::Override => "wgsl_override",
            WgslUnitKind::Struct => "wgsl_struct",
            WgslUnitKind::Alias => "wgsl_alias",
            WgslUnitKind::Variable => "wgsl_var", // naming-lint:external 照 WGSL 的声明关键字 var 起，与 lib_function_units.py 那一份逐字相同
            WgslUnitKind::ConstantAssertion => "wgsl_const_assert",
            WgslUnitKind::Directive => "wgsl_directive",
        }
    }

    /// 模块一进闭包就进的那几种：入口、断言与指令。
    #[must_use]
    pub fn is_module_root(self) -> bool {
        match self {
            WgslUnitKind::ComputeEntry
            | WgslUnitKind::ConstantAssertion
            | WgslUnitKind::Directive => true,
            WgslUnitKind::Function
            | WgslUnitKind::Constant
            | WgslUnitKind::Override
            | WgslUnitKind::Struct
            | WgslUnitKind::Alias
            | WgslUnitKind::Variable => false,
        }
    }
}

/// 一个 WGSL 单元。
#[derive(Clone, Debug)]
pub struct WgslUnit {
    pub kind: WgslUnitKind,
    pub name: String,
    pub file: FileIndex,
    pub start_line: u32,
    pub end_line: u32,
    pub fingerprint: Sha256Digest,
    /// 它用到的模块级名字（去掉了字段访问、属性名、局部变量、参数、结构体成员与自己的名字）。
    pub referenced_names: Vec<String>,
}

/// 一份 `.wgsl` 切出来的东西。
pub struct ParsedWgslFile {
    pub units: Vec<WgslUnit>,
    pub refusals: Vec<Refusal>,
}

/// 关键字、内建类型、内建函数与属性名；`_` 是占位赋值（`_ = 表达式;`）的左边，不是名字。
const WGSL_BUILTIN_NAMES: [&str; 159] = [
    "_",
    "fn",
    "let",
    "var",
    "const",
    "override",
    "struct",
    "alias",
    "return",
    "if",
    "else",
    "loop",
    "for",
    "while",
    "break",
    "continue",
    "continuing",
    "switch",
    "case",
    "default",
    "discard",
    "true",
    "false",
    "enable",
    "requires",
    "diagnostic",
    "const_assert",
    "function",
    "private",
    "workgroup",
    "uniform",
    "storage",
    "read",
    "write",
    "read_write",
    "bool",
    "i32",
    "u32",
    "f32",
    "f16",
    "vec2",
    "vec3",
    "vec4",
    "vec2i",
    "vec3i",
    "vec4i",
    "vec2u",
    "vec3u",
    "vec4u",
    "vec2f",
    "vec3f",
    "vec4f",
    "vec2h",
    "vec3h",
    "vec4h",
    "mat2x2",
    "mat2x3",
    "mat2x4",
    "mat3x2",
    "mat3x3",
    "mat3x4",
    "mat4x2",
    "mat4x3",
    "mat4x4",
    "mat2x2f",
    "mat3x3f",
    "mat4x4f",
    "array",
    "atomic",
    "ptr",
    "sampler",
    "sampler_comparison",
    "compute",
    "vertex",
    "fragment",
    "workgroup_size",
    "group",
    "binding",
    "builtin",
    "location",
    "align",
    "size",
    "id",
    "interpolate",
    "invariant",
    "must_use",
    "global_invocation_id",
    "local_invocation_id",
    "local_invocation_index",
    "workgroup_id",
    "num_workgroups",
    "subgroup_size",
    "subgroup_invocation_id",
    "atomicLoad",
    "atomicStore",
    "atomicAdd",
    "atomicSub",
    "atomicMax",
    "atomicMin",
    "atomicAnd",
    "atomicOr",
    "atomicXor",
    "atomicExchange",
    "atomicCompareExchangeWeak",
    "arrayLength",
    "select",
    "min",
    "max",
    "clamp",
    "abs",
    "sign",
    "countOneBits",
    "countLeadingZeros",
    "countTrailingZeros",
    "firstLeadingBit",
    "firstTrailingBit",
    "extractBits",
    "insertBits",
    "reverseBits",
    "bitcast",
    "workgroupBarrier",
    "storageBarrier",
    "textureBarrier",
    "workgroupUniformLoad",
    "dot",
    "all",
    "any",
    "pack4x8unorm",
    "unpack4x8unorm",
    "floor",
    "ceil",
    "round",
    "sqrt",
    "pow",
    "exp",
    "log",
    "fract",
    "trunc",
    "mix",
    "step",
    "smoothstep",
    "length",
    "distance",
    "normalize",
    "cross",
    "transpose",
    "determinant",
    "dpdx",
    "dpdy",
    "fwidth",
    "saturate",
    "fma",
    "ldexp",
    "frexp",
    "modf",
    "degrees",
    "radians",
    "sin",
    "cos",
];

fn builtin_set() -> &'static HashSet<&'static str> {
    static BUILTINS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    BUILTINS.get_or_init(|| WGSL_BUILTIN_NAMES.iter().copied().collect())
}

/// 是不是 WGSL 的关键字、内建类型、内建函数或属性名。
#[must_use]
pub fn is_wgsl_builtin_name(name: &str) -> bool {
    builtin_set().contains(name)
}

struct WgslScanner<'scan> {
    source: &'scan str,
    tokens: &'scan [Token],
    closes: &'scan [Option<usize>],
    file: FileIndex,
}

impl<'scan> WgslScanner<'scan> {
    fn text_of(&self, index: usize) -> &'scan str {
        self.tokens
            .get(index)
            .map_or("", |token| token_text(self.source, token))
    }

    fn is_identifier(&self, index: usize) -> bool {
        self.tokens
            .get(index)
            .is_some_and(|token| token.kind == TokenKind::Identifier)
    }

    /// `switch` 的 `case A, B:`：冒号前的名字是被比的常量，不是参数或成员的声明。从冒号前那个名字往回跳过名字、数与逗号，碰到 `case` 就是。
    fn is_a_case_selector(&self, index: usize, first: usize) -> bool {
        let mut cursor = index;
        while cursor > first {
            cursor -= 1;
            match self.text_of(cursor) {
                "case" => return true,
                "," => {}
                _ if self.is_identifier(cursor)
                    || self
                        .tokens
                        .get(cursor)
                        .is_some_and(|token| token.kind == TokenKind::Literal) => {}
                _ => return false,
            }
        }
        false
    }

    fn is_open_bracket(&self, index: usize) -> bool {
        self.tokens
            .get(index)
            .is_some_and(|token| token.kind == TokenKind::Punctuation)
            && matches!(self.text_of(index), "(" | "[" | "{")
    }

    fn close_of(&self, open_index: usize) -> usize {
        self.closes[open_index].expect("括号在切条目之前配过，开括号一定有配对的闭括号")
    }

    fn line_of(&self, index: usize) -> u32 {
        self.tokens
            .get(index)
            .or_else(|| self.tokens.last())
            .map_or(1, |token| token.line)
    }

    fn end_at_semicolon(&self, from: usize) -> usize {
        let mut index = from;
        while index < self.tokens.len() {
            if self.is_open_bracket(index) {
                index = self.close_of(index) + 1;
                continue;
            }
            if self.text_of(index) == ";" {
                return index + 1;
            }
            index += 1;
        }
        self.tokens.len()
    }

    /// 单元里用到的模块级名字：去掉 `.` 之后的字段、`@` 之后的属性名、`let` / `var` / `const` 声明的局部名、
    /// 参数名与结构体成员名（后面紧跟 `:` 的，在一层括号里）、以及单元自己的名字。
    fn referenced_names(&self, first: usize, end: usize, own_name: &str) -> Vec<String> {
        let mut declared_here: HashSet<&str> = HashSet::new();
        declared_here.insert(own_name);
        let mut depth = 0usize;
        for index in first..end {
            match self.text_of(index) {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => depth = depth.saturating_sub(1),
                "let" | "var" | "const" if index > first => {
                    let mut name_index = index + 1;
                    if self.text_of(name_index) == "<" {
                        while name_index < end && self.text_of(name_index) != ">" {
                            name_index += 1;
                        }
                        name_index += 1;
                    }
                    if self.is_identifier(name_index) {
                        declared_here.insert(self.text_of(name_index));
                    }
                }
                _ => {
                    if depth >= 1
                        && self.is_identifier(index)
                        && self.text_of(index + 1) == ":"
                        && !self.is_a_case_selector(index, first)
                    {
                        declared_here.insert(self.text_of(index));
                    }
                }
            }
        }
        let mut names = BTreeSet::new();
        for index in first..end {
            if !self.is_identifier(index) {
                continue;
            }
            let previous = if index > first {
                self.text_of(index - 1)
            } else {
                ""
            };
            if previous == "." || previous == "@" {
                continue;
            }
            let name = identifier_name(self.text_of(index));
            if declared_here.contains(name) || is_wgsl_builtin_name(name) {
                continue;
            }
            names.insert(name.to_string());
        }
        names.into_iter().collect()
    }

    fn unit(&self, kind: WgslUnitKind, name: String, first: usize, end: usize) -> WgslUnit {
        let referenced_names = self.referenced_names(first, end, &name);
        WgslUnit {
            kind,
            name,
            file: self.file,
            start_line: self.line_of(first),
            end_line: self.line_of(end.saturating_sub(1)),
            fingerprint: fingerprint_of_tokens(self.source, &self.tokens[first..end]),
            referenced_names,
        }
    }
}

/// 切一份 `.wgsl`。
#[must_use]
pub fn parse_wgsl_file(source: &str, file: FileIndex) -> ParsedWgslFile {
    let lexed = lex_source(source, DocumentationCommentTreatment::KeptOutOfTheTokens);
    let closes = match matching_brackets(source, &lexed.tokens) {
        Ok(closes) => closes,
        Err(offending_index) => {
            return ParsedWgslFile {
                units: Vec::new(),
                refusals: vec![Refusal {
                    file,
                    line: lexed
                        .tokens
                        .get(offending_index)
                        .map_or(1, |token| token.line),
                    form: RefusalForm::UnbalancedBrackets,
                    fragment: lexed
                        .tokens
                        .get(offending_index)
                        .map_or(String::new(), |token| token_text(source, token).to_string()),
                }],
            };
        }
    };
    let scanner = WgslScanner {
        source,
        tokens: &lexed.tokens,
        closes: &closes,
        file,
    };
    let mut units = Vec::new();
    let mut refusals = Vec::new();
    let mut index = 0usize;
    while index < lexed.tokens.len() {
        let start = index;
        let mut is_compute = false;
        while scanner.text_of(index) == "@" && scanner.is_identifier(index + 1) {
            if scanner.text_of(index + 1) == "compute" {
                is_compute = true;
            }
            index += 2;
            if scanner.text_of(index) == "(" {
                index = scanner.close_of(index) + 1;
            }
        }
        let keyword = scanner.text_of(index);
        let name_after_keyword = identifier_name(scanner.text_of(index + 1)).to_string();
        let end = match keyword {
            ";" => {
                index += 1;
                continue;
            }
            "fn" => {
                let mut body = index;
                while body < lexed.tokens.len() && scanner.text_of(body) != "{" {
                    if scanner.is_open_bracket(body) {
                        body = scanner.close_of(body) + 1;
                    } else {
                        body += 1;
                    }
                }
                let end = if body < lexed.tokens.len() {
                    scanner.close_of(body) + 1
                } else {
                    body
                };
                let kind = if is_compute {
                    WgslUnitKind::ComputeEntry
                } else {
                    WgslUnitKind::Function
                };
                units.push(scanner.unit(kind, name_after_keyword, start, end));
                end
            }
            "struct" => {
                let open = (index..lexed.tokens.len())
                    .find(|position| scanner.text_of(*position) == "{")
                    .unwrap_or(lexed.tokens.len());
                let mut end = if open < lexed.tokens.len() {
                    scanner.close_of(open) + 1
                } else {
                    open
                };
                if scanner.text_of(end) == ";" {
                    end += 1;
                }
                units.push(scanner.unit(WgslUnitKind::Struct, name_after_keyword, start, end));
                end
            }
            "const" | "override" | "alias" => {
                let end = scanner.end_at_semicolon(index);
                let kind = match keyword {
                    "const" => WgslUnitKind::Constant,
                    "override" => WgslUnitKind::Override,
                    _ => WgslUnitKind::Alias,
                };
                units.push(scanner.unit(kind, name_after_keyword, start, end));
                end
            }
            "var" => {
                let mut name_index = index + 1;
                if scanner.text_of(name_index) == "<" {
                    while name_index < lexed.tokens.len() && scanner.text_of(name_index) != ">" {
                        name_index += 1;
                    }
                    name_index += 1;
                }
                let end = scanner.end_at_semicolon(index);
                let name = identifier_name(scanner.text_of(name_index)).to_string();
                units.push(scanner.unit(WgslUnitKind::Variable, name, start, end));
                end
            }
            "const_assert" | "enable" | "requires" | "diagnostic" => {
                let end = scanner.end_at_semicolon(index);
                let kind = if keyword == "const_assert" {
                    WgslUnitKind::ConstantAssertion
                } else {
                    WgslUnitKind::Directive
                };
                let name = format!("{keyword}@{}", scanner.line_of(index));
                units.push(scanner.unit(kind, name, start, end));
                end
            }
            _ => {
                let end = scanner.end_at_semicolon(index);
                let fragment = (index..end.min(index + 10))
                    .map(|position| scanner.text_of(position))
                    .collect::<Vec<&str>>()
                    .join(" ");
                refusals.push(Refusal {
                    file,
                    line: scanner.line_of(index),
                    form: RefusalForm::WgslUnrecognizedItemStart,
                    fragment: cleaned_fragment(&fragment),
                });
                end
            }
        };
        index = end.max(index + 1);
    }
    ParsedWgslFile { units, refusals }
}

/// 一个 WGSL 模块的闭包：模块一进就进的那几种单元起，按名字精确收。交收到的单元与用到而模块里没定义的名字（单元，名字）。
#[must_use]
pub fn wgsl_module_closure(
    units: &[WgslUnit],
    module_units: &[WgslUnitIndex],
) -> (BTreeSet<WgslUnitIndex>, Vec<(WgslUnitIndex, String)>) {
    let mut by_name: BTreeMap<&str, Vec<WgslUnitIndex>> = BTreeMap::new();
    for unit_index in module_units {
        by_name
            .entry(units[unit_index.0].name.as_str())
            .or_default()
            .push(*unit_index);
    }
    let mut reached = BTreeSet::new();
    let mut external_names = Vec::new();
    let mut pending: Vec<WgslUnitIndex> = module_units
        .iter()
        .copied()
        .filter(|unit_index| units[unit_index.0].kind.is_module_root())
        .collect();
    while let Some(unit_index) = pending.pop() {
        if !reached.insert(unit_index) {
            continue;
        }
        for name in &units[unit_index.0].referenced_names {
            match by_name.get(name.as_str()) {
                Some(definitions) => pending.extend(definitions.iter().copied()),
                None => external_names.push((unit_index, name.clone())),
            }
        }
    }
    external_names.sort();
    external_names.dedup();
    (reached, external_names)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `switch x { case LIMIT: … }`：冒号前的 `LIMIT` 是被比的常量、是引用，不是局部声明（早先被当成参数名吞掉，常量改了内核的闭包不变）。
    #[test]
    fn a_constant_named_in_a_switch_case_selector_is_a_reference_not_a_local_declaration() {
        let parsed = parse_wgsl_file(
            "const LIMIT: u32 = 3u;\nconst OTHER: u32 = 5u;\n\
             fn pick(x: u32) -> u32 { switch x { case LIMIT, OTHER: { return 1u; } case 7u: { return 2u; } default: { return 0u; } } }\n",
            FileIndex(0),
        );
        assert!(
            parsed.refusals.is_empty(),
            "全都认得：{:?}",
            parsed.refusals
        );
        assert_eq!(parsed.units[2].referenced_names, ["LIMIT", "OTHER"]);
    }

    #[test]
    fn module_level_declarations_become_units_and_locals_are_not_references() {
        let parsed = parse_wgsl_file(
            "const WIDTH: u32 = 4u;\nstruct Pair { left: u32, right: u32 }\n\
             @group(0) @binding(1) var<storage, read_write> words: array<u32>;\n\
             fn helper(value: u32) -> u32 { let doubled = value * 2u; return doubled + WIDTH + PRELUDE_OFFSET; }\n\
             @compute @workgroup_size(64) fn main(@builtin(global_invocation_id) id: vec3<u32>) { words[id.x] = helper(id.x); }\n\
             const_assert WIDTH > 0u;\n",
            FileIndex(0),
        );
        assert!(
            parsed.refusals.is_empty(),
            "全都认得：{:?}",
            parsed.refusals
        );
        let kinds: Vec<WgslUnitKind> = parsed.units.iter().map(|unit| unit.kind).collect();
        assert_eq!(
            kinds,
            [
                WgslUnitKind::Constant,
                WgslUnitKind::Struct,
                WgslUnitKind::Variable,
                WgslUnitKind::Function,
                WgslUnitKind::ComputeEntry,
                WgslUnitKind::ConstantAssertion,
            ]
        );
        assert_eq!(
            parsed.units[3].referenced_names,
            ["PRELUDE_OFFSET", "WIDTH"]
        );
        assert_eq!(parsed.units[4].referenced_names, ["helper", "words"]);
        let every_unit: Vec<WgslUnitIndex> = (0..parsed.units.len()).map(WgslUnitIndex).collect();
        let (reached, external_names) = wgsl_module_closure(&parsed.units, &every_unit);
        assert!(
            !reached.contains(&WgslUnitIndex(1)),
            "没人用到的结构体不进闭包"
        );
        assert_eq!(
            external_names,
            [(WgslUnitIndex(3), "PRELUDE_OFFSET".to_string())]
        );
    }

    #[test]
    fn the_placeholder_assignment_target_is_not_an_external_name() {
        let parsed = parse_wgsl_file(
            "fn touch(value: u32) -> u32 { _ = value; return value; }\n\
             @compute @workgroup_size(1) fn main() { _ = touch(1u); }\n",
            FileIndex(0),
        );
        assert!(
            parsed.refusals.is_empty(),
            "全都认得：{:?}",
            parsed.refusals
        );
        assert_eq!(
            parsed.units[1].referenced_names,
            ["touch"],
            "`_ = …` 的 `_` 不当名字收"
        );
        let every_unit: Vec<WgslUnitIndex> = (0..parsed.units.len()).map(WgslUnitIndex).collect();
        let (_, external_names) = wgsl_module_closure(&parsed.units, &every_unit);
        assert!(
            external_names.is_empty(),
            "`_` 不是外名：{external_names:?}"
        );
    }
}
