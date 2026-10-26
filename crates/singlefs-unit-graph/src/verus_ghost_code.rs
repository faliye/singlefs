//! 产品行为判定与形式化判定分开取键（m4-exit5-relocation-proof-r3 判决第二节 Z4 与第三节「判器键」，D13（验证路线）已定项 17）：
//! 产品行为那几类判定（崩溃判定库的判器组、checker 档快档按条复用、crates 变异判法单元）的单元指纹，去注释之后再剥掉 `verus!` 里
//! 只在 Verus 下有意义的部分——它们不进产品编译出来的代码，只改它们的改动不改产品行为；形式化判定（门禁 checker-tier-formal-verification
//! 的每条）判的正是规格与证明，键保留它们，剥了会拿旧判定顶新规格（假绿）。
//!
//! 剥法用 Verus 自己的语法树，不用词法：每一块 `verus! { … }` 的内文照 `verus_builtin_macros` 的做法（`verus_syn::rejoin_tokens` 之后
//! 逐条解成 `verus_syn::Item`；产品编译擦 ghost 代码用的就是同一个 `verus_syn`）解开，按语法树挑出要剥的节点，把它们在源码里占的字节
//! 换成空格（换行留着，行号不变），再照常切词、切单元、算指纹与收闭包。剥掉的节点里点到的名字随之不再是边。
//!
//! 剥的：spec、proof、axiom 函数与常量（整条，连属性与文档注释）、`broadcast group`、`broadcast use`、`assume_specification`；
//! exec 函数签名里的 `by(…)`、`atomically`、requires、recommends、ensures、default_ensures、returns、decreases、opens_invariants、no_unwind、
//! `with` 一截，具名返回值 `-> (名: 类型)` 的 `(名:` 与 `)`；函数体（与常量、静态量的初值）里的 `proof { … }`、`assert(…)`（连 `by …`）、
//! `assert forall`、`assume(…)`、`reveal(…)` 一族、`let ghost` 与 `let tracked`，循环头的 invariant 一族、ensures、decreases 与 for 循环给迭代器起的名，
//! 闭包的 requires 与 ensures，exec 常量与静态量的 ensures。
//! 再剥（主 agent 2026-10-26 定：产品编译里它们只剩不带数据的壳，值在擦 ghost 代码时擦掉）：函数签名里类型是 `Ghost<…>` / `Tracked<…>`
//! 的参数（连它的模式，例 `Ghost(名): Ghost<类型>`）；结构体与元组结构体里 ghost / tracked 模式的字段与类型是 `Ghost<…>` / `Tracked<…>`
//! 的字段；调用与方法调用里整个实参是 `Ghost(…)` / `Tracked(…)` 的那几个、结构体字面量里初值是 `Ghost(…)` / `Tracked(…)` 的字段、
//! 结构体模式里子模式是 `Ghost(名)` / `Tracked(名)` 的字段。剥一列里的几样时连它们的逗号一起剥，剩下的与压根没写那几样时逐个词法单元相同。
//! `Ghost(…)` / `Tracked(…)` 与解包模式照 `verus_builtin_macros` 认（`handle_mode_blocks`、`ExecGhostPatVisitor`：不带限定、只有一段路径、
//! 恰好一个实参或元素）。
//! 不剥（偏保守，改了照样换键）：别处的 `Ghost(…)` / `Tracked(…)`（例 exec 的 `let` 初值）、闭包的参数、结构体字面量里简写的字段（`S { 名 }`，
//! 看不出是不是 ghost 字段）、`tracked` 记号的参数与返回值、`exec` 记号、`#[verifier::…]` 这类属性。执行代码一个字节不剥。
//!
//! 一份文件里有一块 `verus!` 解不开，整份文件原样留着，交 [`UnparsedVerusBlock`]：工作区装产品行为视图时记一条
//! `verus-ghost-code-unparsed` 拒算，链接到它的产品行为判定照跑、不复用（不静默少剥，也不静默当成剥过）。

use std::ops::Range;

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use verus_syn::parse::{Parse, ParseStream};
use verus_syn::punctuated::Punctuated;
use verus_syn::token::Comma;
use verus_syn::visit::{self, Visit};
use verus_syn::{
    DataMode, Expr, Field, FieldPat, FieldValue, FieldsNamed, FieldsUnnamed, FnArg, FnArgKind,
    FnMode, ImplItem, ImplItemConst, ImplItemFn, Item, ItemConst, ItemFn, ItemStatic, Pat,
    PatStruct, PathArguments, ReturnType, Signature, Stmt, TraitItemConst, TraitItemFn, Type, UnOp,
};

use crate::lexer::{lex_source, token_text, DocumentationCommentTreatment, TokenKind};

/// 换行符：剥的时候留着它，行号不变。
const LINE_FEED: u8 = 0x0a;

/// 一份判定判的是什么，决定它的键取哪一个视图（`Workspace::view_for_judgment`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgmentKind {
    /// 判产品行为：崩溃判定库的判器组、checker 档快档按条复用、crates 变异判法单元。键在剥掉 Verus 规格与证明块的视图上算。
    ProductBehavior,
    /// 判 Verus 的证明：门禁 checker-tier-formal-verification 的每条。键在原样的源码上算，规格与证明都进指纹。
    FormalVerification,
}

impl JudgmentKind {
    /// 命令行 `--judgment-kind` 的写法。
    #[must_use]
    pub fn option_value(self) -> &'static str {
        match self {
            JudgmentKind::ProductBehavior => "product-behavior",
            JudgmentKind::FormalVerification => "formal-verification",
        }
    }

    /// 读 `--judgment-kind` 的值。
    ///
    /// # Errors
    /// 不是两种写法之一，交原文。
    pub fn parse(text: &str) -> Result<Self, String> {
        [
            JudgmentKind::ProductBehavior,
            JudgmentKind::FormalVerification,
        ]
        .into_iter()
        .find(|kind| kind.option_value() == text)
        .ok_or_else(|| text.to_string())
    }
}

/// 一份工作区里的源码，Verus 的规格与证明块在不在：装出来的那一份原样（形式化判定用），产品行为视图里剥掉了。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerusGhostCode {
    Kept,
    Blanked,
}

/// 一块 `verus!` 解不开：它的 `verus` 那个词法单元在第几行、`verus_syn` 报的是什么（带它在源码里的行号）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnparsedVerusBlock {
    pub line: u32,
    pub reason: String,
}

/// `verus!` 块的内文：照 `verus_builtin_macros` 的 `Items`，一条接一条解到底。
struct VerusItems {
    items: Vec<Item>,
}

impl Parse for VerusItems {
    fn parse(input: ParseStream) -> verus_syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items })
    }
}

/// 一块 `verus!`：`verus` 那个词法单元的行号，`{` 所在的行号，内文（两个花括号之间）在源码里的字节区间。
struct VerusBlock {
    line_of_the_macro_name: u32,
    line_of_the_opening_brace: u32,
    interior: Range<usize>,
}

fn byte_index(byte_offset: u32) -> usize {
    usize::try_from(byte_offset).expect("字节偏移放得进 usize")
}

/// 源码里每一块最外层的 `verus! { … }`（`vstd::prelude::verus! { … }` 也算）：词法单元 `verus`、`!`、`{` 挨着，配到它的 `}`。
/// 配不上的不算（括号配不上的文件由切分记拒算）。
fn verus_blocks_of(source: &str) -> Vec<VerusBlock> {
    let lexed = lex_source(source, DocumentationCommentTreatment::KeptOutOfTheTokens);
    let tokens = &lexed.tokens;
    let text = |index: usize| token_text(source, &tokens[index]);
    let is_bracket_punctuation = |index: usize, brackets: &[&str]| {
        tokens[index].kind == TokenKind::Punctuation && brackets.contains(&text(index))
    };
    let mut blocks = Vec::new();
    let mut index = 0usize;
    while index + 2 < tokens.len() {
        let opens_a_verus_block = tokens[index].kind == TokenKind::Identifier
            && text(index) == "verus"
            && tokens[index + 1].kind == TokenKind::Punctuation
            && text(index + 1) == "!"
            && is_bracket_punctuation(index + 2, &["{"]);
        if !opens_a_verus_block {
            index += 1;
            continue;
        }
        let open = index + 2;
        let mut depth = 0usize;
        let mut close = None;
        // 每一步往后走一个词法单元：至多走到文件末尾
        for position in open..tokens.len() {
            if is_bracket_punctuation(position, &["{", "(", "["]) {
                depth += 1;
            } else if is_bracket_punctuation(position, &["}", ")", "]"]) {
                depth -= 1;
                if depth == 0 {
                    close = Some(position);
                    break;
                }
            }
        }
        let Some(close) = close else {
            break;
        };
        blocks.push(VerusBlock {
            line_of_the_macro_name: tokens[index].line,
            line_of_the_opening_brace: tokens[open].line,
            interior: byte_index(tokens[open].end_byte)..byte_index(tokens[close].start_byte),
        });
        index = close + 1;
    }
    blocks
}

/// 一个节点交回词法单元串之后各词法单元（括号组取整组）的字节区间的并：从最早的起点到最晚的终点。
/// 交回时新造、不落在源码里的词法单元（区间是空的）不算。
fn byte_range_of_the_tokens(node: &impl ToTokens) -> Option<Range<usize>> {
    span_union(
        node.to_token_stream()
            .into_iter()
            .map(|tree| tree.span().byte_range()),
    )
}

fn span_union(ranges: impl Iterator<Item = Range<usize>>) -> Option<Range<usize>> {
    let mut union: Option<Range<usize>> = None;
    for range in ranges.filter(|range| !range.is_empty()) {
        union = Some(match union {
            None => range,
            Some(so_far) => so_far.start.min(range.start)..so_far.end.max(range.end),
        });
    }
    union
}

/// 只在 Verus 下有意义的函数与常量：spec、带检查的 spec、proof、axiom。
fn is_ghost_mode(mode: &FnMode) -> bool {
    match mode {
        FnMode::Spec(_) | FnMode::SpecChecked(_) | FnMode::Proof(_) | FnMode::ProofAxiom(_) => true,
        FnMode::Exec(_) | FnMode::Default => false,
    }
}

/// 整条只在 Verus 下有意义的表达式（当语句时连分号一起剥）：`proof { … }`、`assert`、`assert forall`、`assume`、`reveal` 一族。
fn is_ghost_expression(expression: &Expr) -> bool {
    if let Expr::Unary(unary) = expression {
        return matches!(unary.op, UnOp::Proof(_));
    }
    matches!(
        expression,
        Expr::Assert(_) | Expr::AssertForall(_) | Expr::Assume(_) | Expr::RevealHide(_)
    )
}

/// Verus 擦 ghost 代码时按名字认的两个包装：`Ghost` 与 `Tracked`。
const GHOST_WRAPPER_NAME: &str = "Ghost";
const TRACKED_WRAPPER_NAME: &str = "Tracked";

fn is_ghost_or_tracked_wrapper_name(identifier: &Ident) -> bool {
    identifier == GHOST_WRAPPER_NAME || identifier == TRACKED_WRAPPER_NAME
}

/// `Ghost<…>` / `Tracked<…>`：路径末段是包装名、带尖括号参数（`vstd::prelude::Ghost<…>` 也算）。
fn is_ghost_or_tracked_wrapper_type(written_type: &Type) -> bool {
    let Type::Path(type_path) = written_type else {
        return false;
    };
    type_path.qself.is_none()
        && type_path.path.segments.last().is_some_and(|segment| {
            is_ghost_or_tracked_wrapper_name(&segment.ident)
                && matches!(segment.arguments, PathArguments::AngleBracketed(_))
        })
}

/// `Ghost(…)` / `Tracked(…)`（带 turbofish 的也算）：照 `handle_mode_blocks` 认，不带限定、只有一段路径、恰好一个实参。
/// 产品编译把整个调用换成不带数据的壳，里面的表达式擦掉。
fn is_ghost_or_tracked_wrapper_call(expression: &Expr) -> bool {
    let Expr::Call(call) = expression else {
        return false;
    };
    let Expr::Path(path) = &*call.func else {
        return false;
    };
    path.qself.is_none()
        && call.args.len() == 1
        && path.path.segments.len() == 1
        && path
            .path
            .segments
            .first()
            .is_some_and(|segment| is_ghost_or_tracked_wrapper_name(&segment.ident))
}

/// `Ghost(名)` / `Tracked(名)` 这样解包的模式：照 `ExecGhostPatVisitor` 认，只有一段路径、恰好一个元素。
fn is_ghost_or_tracked_wrapper_pattern(pattern: &Pat) -> bool {
    let Pat::TupleStruct(tuple_struct) = pattern else {
        return false;
    };
    tuple_struct.qself.is_none()
        && tuple_struct.elems.len() == 1
        && tuple_struct.path.segments.len() == 1
        && tuple_struct
            .path
            .segments
            .first()
            .is_some_and(|segment| is_ghost_or_tracked_wrapper_name(&segment.ident))
}

/// 类型是 `Ghost<…>` / `Tracked<…>` 的参数，连它的模式；`self` 不算。
fn is_ghost_or_tracked_parameter(parameter: &FnArg) -> bool {
    match &parameter.kind {
        FnArgKind::Typed(typed) => is_ghost_or_tracked_wrapper_type(&typed.ty),
        FnArgKind::Receiver(_) => false,
    }
}

/// ghost / tracked 模式的字段，或类型是 `Ghost<…>` / `Tracked<…>` 的字段。
fn is_ghost_or_tracked_field(field: &Field) -> bool {
    let is_ghost_or_tracked_mode = match field.mode {
        DataMode::Ghost(_) | DataMode::Tracked(_) => true,
        DataMode::Exec(_) | DataMode::Default => false,
    };
    is_ghost_or_tracked_mode || is_ghost_or_tracked_wrapper_type(&field.ty)
}

/// 结构体字面量里初值是 `Ghost(…)` / `Tracked(…)` 的字段。
fn is_ghost_or_tracked_field_initialization(field_value: &FieldValue) -> bool {
    is_ghost_or_tracked_wrapper_call(&field_value.expr)
}

/// 结构体模式里子模式是 `Ghost(名)` / `Tracked(名)` 的字段。
fn is_ghost_or_tracked_field_pattern(field_pattern: &FieldPat) -> bool {
    is_ghost_or_tracked_wrapper_pattern(&field_pattern.pat)
}

/// 沿语法树收要剥的字节区间（内文里的偏移）。
struct GhostCodeCollector {
    ranges: Vec<Range<usize>>,
}

impl GhostCodeCollector {
    fn blank(&mut self, node: &impl ToTokens) {
        if let Some(range) = byte_range_of_the_tokens(node) {
            self.ranges.push(range);
        }
    }

    fn blank_span(&mut self, span: Span) {
        let range = span.byte_range();
        if !range.is_empty() {
            self.ranges.push(range);
        }
    }

    /// 一列用逗号隔开的东西里剥掉挑中的那几样，连它们后面的逗号；挑中的哪一样后面没有逗号（列尾不带逗号），留下的那几样里离它最近的那一样后面的逗号也剥。
    /// 剥完的样子与源码里压根没写那几样时逐个词法单元相同（`f(值, Ghost(名))` 剥成 `f(值)`，不是 `f(值,)`）。
    fn blank_the_chosen_elements_of_a_list<Element: ToTokens>(
        &mut self,
        list: &Punctuated<Element, Comma>,
        is_chosen: impl Fn(&Element) -> bool,
    ) {
        let mut comma_after_the_last_kept_element: Option<&Comma> = None;
        let mut is_a_chosen_element_last_without_a_comma = false;
        for pair in list.pairs() {
            let (element, comma) = pair.into_tuple();
            if !is_chosen(element) {
                comma_after_the_last_kept_element = comma;
                continue;
            }
            self.blank(element);
            match comma {
                Some(comma) => self.blank(comma),
                None => is_a_chosen_element_last_without_a_comma = true,
            }
        }
        if is_a_chosen_element_last_without_a_comma {
            if let Some(comma) = comma_after_the_last_kept_element {
                self.blank(comma);
            }
        }
    }

    /// exec 函数签名里只给 Verus 看的那几截：`Ghost<…>` / `Tracked<…>` 参数，规格子句，具名返回值的 `(名:` 与 `)`（类型留着）。
    fn blank_the_specification_of_the_signature(&mut self, signature: &Signature) {
        self.blank_the_chosen_elements_of_a_list(&signature.inputs, is_ghost_or_tracked_parameter);
        let specification = &signature.spec;
        if let Some(prover) = &specification.prover {
            self.blank(prover);
        }
        if let Some(atomic_specification) = &specification.atomic_spec {
            self.blank(atomic_specification);
        }
        if let Some(requires) = &specification.requires {
            self.blank(requires);
        }
        if let Some(recommends) = &specification.recommends {
            self.blank(recommends);
        }
        if let Some(ensures) = &specification.ensures {
            self.blank(ensures);
        }
        if let Some(default_ensures) = &specification.default_ensures {
            self.blank(default_ensures);
        }
        if let Some(returns) = &specification.returns {
            self.blank(returns);
        }
        if let Some(decreases) = &specification.decreases {
            self.blank(decreases);
        }
        if let Some(invariants) = &specification.invariants {
            self.blank(invariants);
        }
        if let Some(unwind) = &specification.unwind {
            self.blank(unwind);
        }
        if let Some(with) = &specification.with {
            // `with` 那一截没有交回词法单元串的写法（verus_syn 不给它 ToTokens）：逐段剥
            self.blank(&with.with);
            self.blank(&with.inputs);
            if let Some((arrow, outputs)) = &with.outputs {
                self.blank(arrow);
                self.blank(outputs);
            }
        }
        if let ReturnType::Type(_, _, Some(named_return), _) = &signature.output {
            let (parenthesis, pattern, colon) = &**named_return;
            self.blank_span(parenthesis.span.open());
            self.blank(pattern);
            self.blank(colon);
            self.blank_span(parenthesis.span.close());
        }
    }
}

impl<'syntax> Visit<'syntax> for GhostCodeCollector {
    fn visit_item(&mut self, item: &'syntax Item) {
        if matches!(
            item,
            Item::BroadcastGroup(_) | Item::BroadcastUse(_) | Item::AssumeSpecification(_)
        ) {
            self.blank(item);
            return;
        }
        visit::visit_item(self, item);
    }

    fn visit_impl_item(&mut self, item: &'syntax ImplItem) {
        if matches!(item, ImplItem::BroadcastGroup(_)) {
            self.blank(item);
            return;
        }
        visit::visit_impl_item(self, item);
    }

    fn visit_item_fn(&mut self, function: &'syntax ItemFn) {
        if is_ghost_mode(&function.sig.mode) {
            self.blank(function);
            return;
        }
        self.blank_the_specification_of_the_signature(&function.sig);
        self.visit_block(&function.block);
    }

    fn visit_impl_item_fn(&mut self, function: &'syntax ImplItemFn) {
        if is_ghost_mode(&function.sig.mode) {
            self.blank(function);
            return;
        }
        self.blank_the_specification_of_the_signature(&function.sig);
        self.visit_block(&function.block);
    }

    fn visit_trait_item_fn(&mut self, function: &'syntax TraitItemFn) {
        if is_ghost_mode(&function.sig.mode) {
            self.blank(function);
            return;
        }
        self.blank_the_specification_of_the_signature(&function.sig);
        if let Some(body) = &function.default {
            self.visit_block(body);
        }
    }

    fn visit_item_const(&mut self, constant: &'syntax ItemConst) {
        if is_ghost_mode(&constant.mode) {
            self.blank(constant);
            return;
        }
        if let Some(ensures) = &constant.ensures {
            self.blank(ensures);
        }
        visit::visit_item_const(self, constant);
    }

    fn visit_impl_item_const(&mut self, constant: &'syntax ImplItemConst) {
        if is_ghost_mode(&constant.mode) {
            self.blank(constant);
            return;
        }
        if let Some(ensures) = &constant.ensures {
            self.blank(ensures);
        }
        visit::visit_impl_item_const(self, constant);
    }

    fn visit_trait_item_const(&mut self, constant: &'syntax TraitItemConst) {
        if is_ghost_mode(&constant.mode) {
            self.blank(constant);
            return;
        }
        visit::visit_trait_item_const(self, constant);
    }

    fn visit_item_static(&mut self, value: &'syntax ItemStatic) {
        if is_ghost_mode(&value.mode) {
            self.blank(value);
            return;
        }
        if let Some(ensures) = &value.ensures {
            self.blank(ensures);
        }
        visit::visit_item_static(self, value);
    }

    fn visit_stmt(&mut self, statement: &'syntax Stmt) {
        match statement {
            Stmt::Local(local) if local.ghost.is_some() || local.tracked.is_some() => {
                self.blank(statement);
            }
            Stmt::Expr(expression, _) if is_ghost_expression(expression) => {
                self.blank(statement);
            }
            Stmt::Local(_) | Stmt::Item(_) | Stmt::Expr(..) | Stmt::Macro(_) => {
                visit::visit_stmt(self, statement);
            }
        }
    }

    fn visit_expr(&mut self, expression: &'syntax Expr) {
        if is_ghost_expression(expression) {
            self.blank(expression);
            return;
        }
        // Expr 是外部的大枚举、六十来种：只挑带 Verus 子句的四种与带 Ghost / Tracked 实参或初值的三种，用 if let 不写通配臂
        if let Expr::While(loop_expression) = expression {
            self.blank_loop_specification(
                loop_expression.invariant_except_break.as_ref(),
                loop_expression.invariant.as_ref(),
                loop_expression.invariant_ensures.as_ref(),
                loop_expression.ensures.as_ref(),
                loop_expression.decreases.as_ref(),
            );
        } else if let Expr::Loop(loop_expression) = expression {
            self.blank_loop_specification(
                loop_expression.invariant_except_break.as_ref(),
                loop_expression.invariant.as_ref(),
                loop_expression.invariant_ensures.as_ref(),
                loop_expression.ensures.as_ref(),
                loop_expression.decreases.as_ref(),
            );
        } else if let Expr::ForLoop(loop_expression) = expression {
            if let Some(iterator_name) = &loop_expression.expr_name {
                let (name, colon) = &**iterator_name;
                self.blank(name);
                self.blank(colon);
            }
            self.blank_loop_specification(
                loop_expression.invariant_except_break.as_ref(),
                loop_expression.invariant.as_ref(),
                None,
                loop_expression.ensures.as_ref(),
                loop_expression.decreases.as_ref(),
            );
        } else if let Expr::Closure(closure) = expression {
            if let Some(requires) = &closure.requires {
                self.blank(requires);
            }
            if let Some(ensures) = &closure.ensures {
                self.blank(ensures);
            }
        } else if let Expr::Call(call) = expression {
            self.blank_the_chosen_elements_of_a_list(&call.args, is_ghost_or_tracked_wrapper_call);
        } else if let Expr::MethodCall(method_call) = expression {
            self.blank_the_chosen_elements_of_a_list(
                &method_call.args,
                is_ghost_or_tracked_wrapper_call,
            );
        } else if let Expr::Struct(structure) = expression {
            self.blank_the_chosen_elements_of_a_list(
                &structure.fields,
                is_ghost_or_tracked_field_initialization,
            );
        }
        visit::visit_expr(self, expression);
    }

    fn visit_fields_named(&mut self, fields: &'syntax FieldsNamed) {
        self.blank_the_chosen_elements_of_a_list(&fields.named, is_ghost_or_tracked_field);
        visit::visit_fields_named(self, fields);
    }

    fn visit_fields_unnamed(&mut self, fields: &'syntax FieldsUnnamed) {
        self.blank_the_chosen_elements_of_a_list(&fields.unnamed, is_ghost_or_tracked_field);
        visit::visit_fields_unnamed(self, fields);
    }

    fn visit_pat_struct(&mut self, pattern: &'syntax PatStruct) {
        self.blank_the_chosen_elements_of_a_list(
            &pattern.fields,
            is_ghost_or_tracked_field_pattern,
        );
        visit::visit_pat_struct(self, pattern);
    }
}

impl GhostCodeCollector {
    fn blank_loop_specification(
        &mut self,
        invariant_except_break: Option<&verus_syn::InvariantExceptBreak>,
        invariant: Option<&verus_syn::Invariant>,
        invariant_ensures: Option<&verus_syn::InvariantEnsures>,
        ensures: Option<&verus_syn::Ensures>,
        decreases: Option<&verus_syn::Decreases>,
    ) {
        if let Some(clause) = invariant_except_break {
            self.blank(clause);
        }
        if let Some(clause) = invariant {
            self.blank(clause);
        }
        if let Some(clause) = invariant_ensures {
            self.blank(clause);
        }
        if let Some(clause) = ensures {
            self.blank(clause);
        }
        if let Some(clause) = decreases {
            self.blank(clause);
        }
    }
}

/// 一块 `verus!` 内文里要剥的字节区间（内文里的偏移）；`verus_syn` 解不开交它报的话与它在内文里的行号（从 1 数）。
fn ghost_code_ranges_of_the_interior(interior: &str) -> Result<Vec<Range<usize>>, (usize, String)> {
    let stream: TokenStream = interior
        .parse()
        .map_err(|error: proc_macro2::LexError| (error.span().start().line, error.to_string()))?;
    let items: VerusItems = verus_syn::parse2(verus_syn::rejoin_tokens(stream))
        .map_err(|error| (error.span().start().line, error.to_string()))?;
    let mut collector = GhostCodeCollector { ranges: Vec::new() };
    for item in &items.items {
        collector.visit_item(item);
    }
    Ok(collector.ranges)
}

/// 解不开的那一块：按 `verus` 所在的行报，话里带 `verus_syn` 报错落在源码里的哪一行。
fn unparsed_block(
    block: &VerusBlock,
    line_within_the_interior: usize,
    reason: &str,
) -> UnparsedVerusBlock {
    let line_within_the_interior =
        u32::try_from(line_within_the_interior.max(1)).expect("一块 verus! 的行数放得进 u32");
    UnparsedVerusBlock {
        line: block.line_of_the_macro_name,
        reason: format!(
            "第 {} 行：{reason}",
            block.line_of_the_opening_brace + line_within_the_interior - 1
        ),
    }
}

/// 源码里每一块 `verus!` 的规格与证明块换成空格（换行留着，字节数与行号都不变）；没有 `verus!` 的原样交回。
///
/// # Errors
/// 有一块交给 `verus_syn` 解不开：交头一块解不开的那一块（行号是源码里的行号）。
pub fn source_with_verus_ghost_code_blanked(source: &str) -> Result<String, UnparsedVerusBlock> {
    let mut bytes = source.as_bytes().to_vec();
    for block in verus_blocks_of(source) {
        let interior = &source[block.interior.clone()];
        let ranges = match ghost_code_ranges_of_the_interior(interior) {
            Ok(ranges) => ranges,
            Err((line_within_the_interior, reason)) => {
                return Err(unparsed_block(&block, line_within_the_interior, &reason))
            }
        };
        for range in ranges {
            let absolute = block.interior.start + range.start..block.interior.start + range.end;
            for byte in &mut bytes[absolute] {
                if *byte != LINE_FEED {
                    *byte = b' ';
                }
            }
        }
    }
    Ok(String::from_utf8(bytes).expect(
        "换成空格的区间是语法树节点的字节区间，起止都落在字符边界上，整段多字节字符一起换掉，交回的仍是 UTF-8",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 没有 `verus!` 的原样交回：别的宏名带花括号的调用（里面写得像 Verus）也不剥。
    #[test]
    fn a_source_without_verus_blocks_comes_back_byte_for_byte() {
        let source = "fn plain(value: u64) -> u64 {\n    value + 1\n}\nlookalike! {\n    fn inside(width: usize) -> (named: usize) ensures named == width, { width }\n}\n";
        assert_eq!(
            source_with_verus_ghost_code_blanked(source).as_deref(),
            Ok(source)
        );
    }

    #[test]
    fn blanking_keeps_the_byte_length_and_every_newline() {
        let source = "verus! {\nspec fn width_in_the_specification() -> int {\n    10\n}\nfn width() -> (width: usize)\n    ensures width == 10,\n{\n    proof { assert(true); }\n    10\n}\n}\n";
        let blanked = source_with_verus_ghost_code_blanked(source).expect("这块解得开");
        assert_eq!(blanked.len(), source.len(), "字节数不变");
        assert_eq!(
            blanked.matches('\n').count(),
            source.matches('\n').count(),
            "换行都留着"
        );
        let words: Vec<&str> = blanked.split_whitespace().collect();
        assert_eq!(
            words,
            ["verus!", "{", "fn", "width()", "->", "usize", "{", "10", "}", "}"],
            "剥掉 spec 函数、具名返回值的名字、ensures 与 proof 块，剩下 exec 代码：{blanked}"
        );
    }

    #[test]
    fn an_unparsable_verus_block_names_its_line() {
        let source = "\n\nverus! {\nfn broken() -> {}\n}\n";
        let verdict = source_with_verus_ghost_code_blanked(source);
        assert!(
            matches!(&verdict, Err(UnparsedVerusBlock { line: 3, .. })),
            "解不开的那一块按 verus 所在的第 3 行报：{verdict:?}"
        );
    }

    #[test]
    fn the_judgment_kind_reads_back_both_option_values() {
        for kind in [
            JudgmentKind::ProductBehavior,
            JudgmentKind::FormalVerification,
        ] {
            assert_eq!(JudgmentKind::parse(kind.option_value()), Ok(kind));
        }
        assert!(JudgmentKind::parse("both").is_err());
    }
}
