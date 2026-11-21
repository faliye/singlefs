//! 单元起子进程：`--exact` 后面折得出的测试名、`current_exe()` 原样交给 `Command::new` 的程序起自己。
use super::*;

/// 单元里出现这几个词算起了子进程：此时 `current_exe()` 折不出要点名的那一条测试、路径又不是一个字不改地交给 `Command::new`（那一形看入口，
/// [`SourceRead::OwnProgramRestart`]）就拒算 `current-exe`；一个都没有的只是在问路径，不拒算（与 `scripts/gate/lib_function_units.py` 的 `SPAWNING_WORDS` 同一份）。
const SPAWNING_WORDS: [&str; 7] = [
    "Command",
    "spawn",
    "posix_spawn",
    "fork",
    "execv",
    "execve",
    "execvp",
];

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

impl<'scan> RustScanner<'scan> {
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
    pub(super) fn child_test_name_at(
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

    /// `last_token` 是一个表达式的最后一个词法单元时，这个表达式是否一定不交值：写成 `panic!(…)`、`…process::exit(…)`，或以它们收尾的块。
    /// 块只认紧跟闭包参数（开括号前面是 `|`）与紧跟 `=>` 的那两种：块前面是 `else`、`)`、`if` 的条件这类的，是分支里的一支，别的分支可能交值。
    fn expression_ending_at_diverges(&self, last_token: usize, first_allowed: usize) -> bool {
        match self.text_of(last_token) {
            "}" => {
                let Some(open) = (first_allowed..last_token).rev().find(|position| {
                    self.text_of(*position) == "{" && self.closes[*position] == Some(last_token)
                }) else {
                    return false;
                };
                let follows_closure_parameters = self.text_before(open) == "|";
                let follows_a_match_arrow =
                    open >= 2 && self.text_of(open - 1) == ">" && self.text_of(open - 2) == "=";
                if !(follows_closure_parameters || follows_a_match_arrow) {
                    return false;
                }
                let before_the_brace = last_token.saturating_sub(1);
                let last_inside = if self.text_of(before_the_brace) == ";" {
                    before_the_brace.saturating_sub(1)
                } else {
                    before_the_brace
                };
                self.expression_ending_at_diverges(last_inside, first_allowed)
            }
            ")" => {
                let Some(open) = (first_allowed..last_token).rev().find(|position| {
                    self.text_of(*position) == "(" && self.closes[*position] == Some(last_token)
                }) else {
                    return false;
                };
                let is_panic =
                    open >= 2 && self.text_of(open - 1) == "!" && self.text_of(open - 2) == "panic";
                let is_process_exit = open >= 4
                    && self.text_of(open - 1) == "exit"
                    && self.text_of(open - 2) == ":"
                    && self.text_of(open - 3) == ":"
                    && self.text_of(open - 4) == "process";
                is_panic || is_process_exit
            }
            _ => false,
        }
    }

    /// `current_exe()` 调用（闭括号之后是 `after_call`）连同把路径取出来的写法整个是什么表达式：交 `[起, 止)`。认的写法：
    /// 后接 `?`、`.unwrap()`、`.expect("字面串")`、闭包一定不交值的 `.unwrap_or_else(…)`；或
    /// `match current_exe() { Ok(名字) => 名字, Err(…) => <return … 或一定不交值的表达式> }`。别的写法（失败时换一个路径、取到路径又接方法）交 `None`。
    fn own_path_expression_range(
        &self,
        call_start: usize,
        after_call: usize,
        unit_end: usize,
    ) -> Option<(usize, usize)> {
        if self.text_of(after_call) == "?" {
            return Some((call_start, after_call + 1));
        }
        if self.text_of(after_call) == "." && self.text_of(after_call + 2) == "(" {
            let open = after_call + 2;
            let close = self.close_of(open);
            let is_taken = match self.text_of(after_call + 1) {
                "unwrap" => close == open + 1,
                "expect" => {
                    close == open + 2 && plain_string_content(self.text_of(open + 1)).is_some()
                }
                "unwrap_or_else" => {
                    self.text_of(open + 1) == "|"
                        && close > open + 1
                        && self.expression_ending_at_diverges(close - 1, open + 1)
                }
                _ => false,
            };
            return is_taken.then_some((call_start, close + 1));
        }
        if self.text_of(after_call) != "{" || self.text_before(call_start) != "match" {
            return None;
        }
        let close = self.close_of(after_call);
        if close > unit_end
            || self.text_of(after_call + 1) != "Ok"
            || self.text_of(after_call + 2) != "("
            || !self.is_identifier(after_call + 3)
            || self.text_of(after_call + 4) != ")"
            || self.text_of(after_call + 5) != "="
            || self.text_of(after_call + 6) != ">"
            || self.text_of(after_call + 7) != self.text_of(after_call + 3)
            || self.text_of(after_call + 8) != ","
            || self.text_of(after_call + 9) != "Err"
            || self.text_of(after_call + 10) != "("
        {
            return None;
        }
        let error_pattern_close = self.close_of(after_call + 10);
        if self.text_of(error_pattern_close + 1) != "="
            || self.text_of(error_pattern_close + 2) != ">"
        {
            return None;
        }
        let arm_start = error_pattern_close + 3;
        let arm_end = if self.text_of(close - 1) == "," {
            close - 1
        } else {
            close
        };
        if arm_start >= arm_end {
            return None;
        }
        let arm_texts: Vec<&str> = (arm_start..arm_end)
            .map(|index| self.text_of(index))
            .collect();
        let is_one_arm = split_at_top_level_commas(&arm_texts).len() == 1;
        let arm_does_not_hand_a_value = self.text_of(arm_start) == "return"
            || self.expression_ending_at_diverges(arm_end - 1, arm_start);
        (is_one_arm && arm_does_not_hand_a_value).then_some((call_start - 1, close + 1))
    }

    /// `open_paren` 是 `Command::new(` 的开括号。
    fn is_command_new_open(&self, open_paren: usize) -> bool {
        open_paren >= 4
            && self.text_of(open_paren) == "("
            && self.text_of(open_paren - 1) == "new"
            && self.text_of(open_paren - 2) == ":"
            && self.text_of(open_paren - 3) == ":"
            && self.text_of(open_paren - 4) == "Command"
    }

    /// `current_exe()` 的路径是不是一个字不改地交给了 `Command::new`：起的就是这个程序文件本身。两种写法：就地写在 `Command::new(…)` 的括号里；
    /// 或绑给 `let` 名字（同一个单元里这个名字不再 `let`），名字的每一处用法最内的那一层花括号就是这条 `let` 所在的那一层（不进更深的块、不出这个块）、只是
    /// `Command::new(名字)`、`Command::new(&名字)` 与 `名字.display()`，至少起一次。
    /// 拿路径拼别的路径（`.parent()`、`.with_file_name(…)`、`.join(…)`）、交给别的函数、当参数交给别的程序（`Command::new("sh").arg(名字)`）的不算：
    /// 那样起的可能是别的程序。
    pub(super) fn own_program_path_is_handed_to_command_unchanged(
        &self,
        current_exe_index: usize,
        unit_end: usize,
    ) -> bool {
        let call_start = self.path_expression_start(current_exe_index);
        if self.text_of(current_exe_index + 2) != ")" {
            return false;
        }
        let Some((expression_start, expression_end)) =
            self.own_path_expression_range(call_start, current_exe_index + 3, unit_end)
        else {
            return false;
        };
        if expression_start > 0
            && self.is_command_new_open(expression_start - 1)
            && self.close_of(expression_start - 1) == expression_end
        {
            return true;
        }
        let Some(name) = self.let_binding_name(expression_start, expression_end) else {
            return false;
        };
        let is_bound_again = (expression_end..unit_end).any(|index| {
            self.text_of(index) == "let"
                && (self.name_at(index + 1) == name
                    || (self.text_of(index + 1) == "mut" && self.name_at(index + 2) == name))
        });
        if is_bound_again {
            return false;
        }
        // 每一处用法最内的那一层花括号都要是这条 `let` 所在的那一层：出了这个块的同名用法指的是别的绑定；
        // 进了更深的块（`if`、循环、闭包体）的用法走不走得到、块里起的是不是它，看不出
        let innermost_brace_around = |index: usize| {
            (0..index).rev().find(|position| {
                self.text_of(*position) == "{"
                    && self.closes[*position].is_some_and(|close| close > index)
            })
        };
        let Some(block_of_the_let) = innermost_brace_around(expression_start) else {
            return false;
        };
        let uses = self.uses_of_the_binding(&name, expression_end, unit_end);
        if uses
            .iter()
            .any(|use_index| innermost_brace_around(*use_index) != Some(block_of_the_let))
        {
            return false;
        }
        let is_handed_to_command = |use_index: usize| {
            let open = if self.text_before(use_index) == "&" {
                use_index - 2
            } else {
                use_index - 1
            };
            self.is_command_new_open(open) && self.close_of(open) == use_index + 1
        };
        let is_only_displayed = |use_index: usize| {
            self.text_of(use_index + 1) == "."
                && self.text_of(use_index + 2) == "display"
                && self.text_of(use_index + 3) == "("
                && self.text_of(use_index + 4) == ")"
        };
        uses.iter()
            .any(|use_index| is_handed_to_command(*use_index))
            && uses
                .iter()
                .all(|use_index| is_handed_to_command(*use_index) || is_only_displayed(*use_index))
    }

    /// 单元的词法单元 `[first, end)` 里有没有起子进程的词（[`SPAWNING_WORDS`]，标识符逐字比）。
    pub(super) fn starts_a_process(&self, first: usize, end: usize) -> bool {
        (first..end)
            .any(|index| self.is_identifier(index) && SPAWNING_WORDS.contains(&self.text_of(index)))
    }
}
