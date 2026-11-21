//! R-裸名局部绑定遮住同名函数：函数单元里一个名字是不是只落在局部绑定与它的作用域里。
use super::*;

/// 函数体里这几个词起头的可能是嵌套条目。嵌套条目看不见外层函数的局部绑定，里面的同名引用要往外解；哪一处在嵌套条目里不细分，
/// 函数体里有这几个词的函数一律不算遮住（`fn` 指针类型、`*const` 这类不是条目的写法同样算，只多连）。
const WORDS_STARTING_A_NESTED_ITEM: [&str; 13] = [
    "fn",
    "impl",
    "trait",
    "mod",
    "const",
    "static",
    "struct",
    "enum",
    "union",
    "type",
    "use",
    "macro_rules",
    "extern",
];

/// 函数单元的签名在它词法单元里的位置（`function_signature_positions`）。
struct FunctionSignaturePositions {
    own_name: usize,
    parameters_open: usize,
    parameters_close: usize,
    body_open: usize,
}

/// 找函数单元的签名：深度 0 上头一个 `fn`、后面的名字、跳过尖括号里的泛型参数之后的 `(` 与配对的 `)`、再往后深度 0 上头一个 `{`。
/// 哪一步对不上（`verus!` 里的怪写法、没有函数体的声明）交 None。
fn function_signature_positions(
    source: &str,
    tokens: &[Token],
) -> Option<FunctionSignaturePositions> {
    let text_at = |position: usize| {
        tokens
            .get(position)
            .map_or("", |token| token_text(source, token))
    };
    let mut depth_before_the_keyword = 0usize;
    let mut function_keyword = None;
    for position in 0..tokens.len() {
        match text_at(position) {
            "(" | "[" | "{" => depth_before_the_keyword += 1,
            ")" | "]" | "}" => {
                depth_before_the_keyword = depth_before_the_keyword.saturating_sub(1)
            }
            "fn" if depth_before_the_keyword == 0 => {
                function_keyword = Some(position);
                break;
            }
            _ => {}
        }
    }
    let own_name = function_keyword? + 1;
    let mut parameters_open = own_name + 1;
    if text_at(parameters_open) == "<" {
        let mut angle_depth = 0usize;
        loop {
            match text_at(parameters_open) {
                "" => return None,
                "<" => angle_depth += 1,
                ">" if text_at(parameters_open - 1) != "-" => {
                    angle_depth = angle_depth.saturating_sub(1);
                }
                _ => {}
            }
            parameters_open += 1;
            if angle_depth == 0 {
                break;
            }
        }
    }
    if text_at(parameters_open) != "(" {
        return None;
    }
    let mut depth_from_the_parameters = 0usize;
    let mut parameters_close = None;
    let mut body_open = None;
    for position in parameters_open..tokens.len() {
        match text_at(position) {
            "{" if parameters_close.is_some() && depth_from_the_parameters == 0 => {
                body_open = Some(position);
                break;
            }
            "(" | "[" | "{" => depth_from_the_parameters += 1,
            ")" | "]" | "}" => {
                depth_from_the_parameters = depth_from_the_parameters.saturating_sub(1);
                if depth_from_the_parameters == 0 && parameters_close.is_none() {
                    parameters_close = Some(position);
                }
            }
            _ => {}
        }
    }
    Some(FunctionSignaturePositions {
        own_name,
        parameters_open,
        parameters_close: parameters_close?,
        body_open: body_open?,
    })
}

/// 参数表里绑定 `name` 的那一处：括号深度 1 上紧跟 `(` 或 `,`、可带 `mut`、后面接单个 `:`。解构的参数（`(a, b): …`、`&a: …`）不认。
fn parameter_binding_position(
    source: &str,
    tokens: &[Token],
    signature: &FunctionSignaturePositions,
    name: &str,
) -> Option<usize> {
    let text_at = |position: usize| {
        tokens
            .get(position)
            .map_or("", |token| token_text(source, token))
    };
    let mut delimiter_depth = 0usize;
    let mut is_at_parameter_start = true;
    for position in signature.parameters_open + 1..signature.parameters_close {
        let word = text_at(position);
        match word {
            "(" | "[" | "{" => delimiter_depth += 1,
            ")" | "]" | "}" => delimiter_depth = delimiter_depth.saturating_sub(1),
            "," if delimiter_depth == 0 => {
                is_at_parameter_start = true;
                continue;
            }
            _ => {}
        }
        if !is_at_parameter_start {
            continue;
        }
        if word == "mut" {
            continue;
        }
        is_at_parameter_start = false;
        if tokens
            .get(position)
            .is_some_and(|token| token.kind == TokenKind::Identifier)
            && identifier_name(word) == name
            && text_at(position + 1) == ":"
            && text_at(position + 2) != ":"
        {
            return Some(position);
        }
    }
    None
}

/// `name_position` 上的名字是一条语句开头的 `let [mut] 名` 绑定：`let` 紧跟 `{`、`}` 或 `;`（`if let`、`while let`、`&& let` 不算），名字后面接 `=`、`;`
/// 或单个 `:`。解构模式不认。
fn is_statement_let_binding_at(source: &str, tokens: &[Token], name_position: usize) -> bool {
    let text_at = |position: usize| {
        tokens
            .get(position)
            .map_or("", |token| token_text(source, token))
    };
    let let_distance = if name_position >= 1 && text_at(name_position - 1) == "mut" {
        2
    } else {
        1
    };
    let Some(let_position) = name_position.checked_sub(let_distance) else {
        return false;
    };
    let Some(before_the_let) = let_position.checked_sub(1) else {
        return false;
    };
    text_at(let_position) == "let"
        && matches!(text_at(before_the_let), "{" | "}" | ";")
        && match text_at(name_position + 1) {
            "=" | ";" => true,
            ":" => text_at(name_position + 2) != ":",
            _ => false,
        }
}

/// R-裸名局部绑定：名字落在这几个标准库宏的调用括号里，照算局部。它们把实参当表达式就地求值，不把实参接在路径后面、
/// 不放进嵌套条目里；别的宏（工作区的、外部 crate 的，标准库里不在这张表上的）把实参展开到哪看不出，落在它们调用括号里的名字不算遮住。
const STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE: [&str; 16] = [
    "format",
    "format_args",
    "print",
    "println",
    "eprint",
    "eprintln",
    "write",
    "writeln",
    "panic",
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "vec",
];

/// 还开着的一层括号，只记判局部绑定要的那几样。
#[derive(Clone, Copy)]
struct OpenDelimiterOfTheBindingScan {
    is_brace: bool,
    is_inside_a_macro_call: bool,
    /// 这一层落在某个不在 [`STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE`] 里的宏的调用括号里（外层落在里面的算）。
    is_inside_a_macro_call_that_may_move_its_arguments: bool,
    /// 这一层此刻看得见 `name` 的局部绑定（外层绑定了的算）。
    is_name_bound: bool,
    /// 这一层有一条 `let 名` 还没走到它的 `;`：初始值里的 `名` 指外面的，走到 `;` 才算绑定了。
    is_let_waiting_for_its_semicolon: bool,
}

/// 函数单元 `tokens` 里 `name` 只是局部绑定：至少有一处绑定（语句开头的 `let [mut] 名`，或参数），别的每一处都落在绑定的作用域里。块级作用域按花括号认：
/// `let` 从它的 `;` 起到外层花括号收尾，参数从参数表的 `)` 起到单元收尾。这几样不算遮住（宁可多连）：绑定之前或作用域之外出现的、`let` 初始值里的；
/// 带路径的（前后挨 `::`）、宏名（后接 `!` 与开括号）；格式串捕获（`{名}`）在作用域外的；宏调用括号里的 `let`（展开成什么看不出，不当绑定）；
/// 落在宏调用括号里的名字（宏可能把它接在路径后面或放进嵌套条目，运行时解到同名函数），只有
/// [`STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE`] 那几个宏的括号里、在作用域里的照算局部；解构、`if let`、`match`、`for`、闭包参数绑定的不认（这几处的名字按「作用域外」算）；函数体里有嵌套条目的词
/// （`WORDS_STARTING_A_NESTED_ITEM`）的。单个 `.` 后面的是字段或方法，不指函数，不看；函数自己的名字不看。
pub(super) fn is_bare_name_only_a_local_binding_of_the_function(
    source: &str,
    tokens: &[Token],
    name: &str,
) -> bool {
    let Some(signature) = function_signature_positions(source, tokens) else {
        return false;
    };
    let text_at = |position: usize| {
        tokens
            .get(position)
            .map_or("", |token| token_text(source, token))
    };
    let parameter_position = parameter_binding_position(source, tokens, &signature, name);
    let mut has_binding = parameter_position.is_some();
    let mut open_delimiters = vec![OpenDelimiterOfTheBindingScan {
        is_brace: false,
        is_inside_a_macro_call: false,
        is_inside_a_macro_call_that_may_move_its_arguments: false,
        is_name_bound: false,
        is_let_waiting_for_its_semicolon: false,
    }];
    for (position, token) in tokens.iter().enumerate() {
        let text = token_text(source, token);
        let innermost = *open_delimiters
            .last()
            .expect("最外面那一层不出栈（下面的出栈判了长度），栈不空");
        match token.kind {
            TokenKind::Lifetime => {}
            TokenKind::Literal => {
                if !innermost.is_name_bound
                    && text.contains(name)
                    && references_of_tokens(source, std::slice::from_ref(token), None)
                        .plain_names
                        .iter()
                        .any(|captured| captured == name)
                {
                    return false;
                }
            }
            TokenKind::Punctuation => match text {
                "(" | "[" | "{" => {
                    let follows_a_macro_name = position >= 2
                        && text_at(position - 1) == "!"
                        && tokens[position - 2].kind == TokenKind::Identifier;
                    let opens_a_macro_call_that_may_move_its_arguments = follows_a_macro_name
                        && !STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE
                            .contains(&identifier_name(text_at(position - 2)));
                    open_delimiters.push(OpenDelimiterOfTheBindingScan {
                        is_brace: text == "{",
                        is_inside_a_macro_call: innermost.is_inside_a_macro_call
                            || follows_a_macro_name,
                        is_inside_a_macro_call_that_may_move_its_arguments: innermost
                            .is_inside_a_macro_call_that_may_move_its_arguments
                            || opens_a_macro_call_that_may_move_its_arguments,
                        is_name_bound: innermost.is_name_bound,
                        is_let_waiting_for_its_semicolon: false,
                    });
                }
                ")" | "]" | "}" => {
                    if open_delimiters.len() > 1 {
                        open_delimiters.pop();
                    }
                    if position == signature.parameters_close && parameter_position.is_some() {
                        open_delimiters
                            .last_mut()
                            .expect("上面只在长度大于 1 时出栈，栈不空")
                            .is_name_bound = true;
                    }
                }
                ";" => {
                    let scope = open_delimiters
                        .last_mut()
                        .expect("最外面那一层不出栈（上面的出栈判了长度），栈不空");
                    if scope.is_let_waiting_for_its_semicolon {
                        scope.is_let_waiting_for_its_semicolon = false;
                        scope.is_name_bound = true;
                    }
                }
                _ => {}
            },
            TokenKind::Identifier => {
                if identifier_name(text) != name
                    || position == signature.own_name
                    || Some(position) == parameter_position
                {
                    continue;
                }
                if innermost.is_brace
                    && !innermost.is_inside_a_macro_call
                    && is_statement_let_binding_at(source, tokens, position)
                {
                    let scope = open_delimiters
                        .last_mut()
                        .expect("最外面那一层不出栈（上面的出栈判了长度），栈不空");
                    scope.is_let_waiting_for_its_semicolon = true;
                    has_binding = true;
                    continue;
                }
                let is_after_a_single_dot = position >= 1
                    && text_at(position - 1) == "."
                    && !(position >= 2 && text_at(position - 2) == ".");
                if is_after_a_single_dot {
                    continue;
                }
                let is_after_a_path_separator =
                    position >= 2 && text_at(position - 1) == ":" && text_at(position - 2) == ":";
                let is_before_a_path_separator =
                    text_at(position + 1) == ":" && text_at(position + 2) == ":";
                let is_a_macro_name = text_at(position + 1) == "!"
                    && matches!(text_at(position + 2), "(" | "[" | "{");
                if is_after_a_path_separator || is_before_a_path_separator || is_a_macro_name {
                    return false;
                }
                if innermost.is_inside_a_macro_call_that_may_move_its_arguments {
                    return false;
                }
                if !innermost.is_name_bound {
                    return false;
                }
            }
        }
    }
    has_binding
        && !tokens[signature.body_open..].iter().any(|token| {
            token.kind == TokenKind::Identifier
                && WORDS_STARTING_A_NESTED_ITEM.contains(&token_text(source, token))
        })
}
