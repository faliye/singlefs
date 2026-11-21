//! 规范文本的词法：命令切分（`split_commands`）、引号与替换的配对、字面词，here-doc 的起头、收尾与正文，
//! 一条命令读进来的东西（`command_inputs`），与单元自己写出、拷出的脚本（`generated_scripts`）。
use super::*;

// ── 规范文本的词法：命令切分、引号与替换的配对 ─────────────────────────────

/// `text[start]` 是 `opening`；交配对的 `closing` 的下标（跳过引号），配不上交最后一个下标。
pub(super) fn matching_close(text: &[char], start: usize, opening: char, closing: char) -> usize {
    let mut depth: i64 = 0;
    let mut index = start;
    while index < text.len() {
        let character = text[index];
        if character == '\\' {
            index += 2;
            continue;
        }
        if character == '\'' && opening != '{' {
            index = find_character(text, '\'', index + 1).map_or(text.len(), |end| end + 1);
            continue;
        }
        if character == '"' {
            index = double_quote_end(text, index);
            continue;
        }
        if character == opening {
            depth += 1;
        } else if character == closing {
            depth -= 1;
            if depth == 0 {
                return index;
            }
        }
        index += 1;
    }
    text.len().saturating_sub(1)
}

fn double_quote_end(text: &[char], start: usize) -> usize {
    let mut index = start + 1;
    while index < text.len() {
        let character = text[index];
        if character == '\\' {
            index += 2;
            continue;
        }
        if character == '"' {
            return index + 1;
        }
        if starts_with_at(text, index, "$(") {
            index = matching_close(text, index + 1, '(', ')') + 1;
            continue;
        }
        if starts_with_at(text, index, "${") {
            index = matching_close(text, index + 1, '{', '}') + 1;
            continue;
        }
        index += 1;
    }
    text.len()
}

/// 最外一层的命令替换 `$( … )` 与反引号 → （里面的文本，它在 `text` 里起头的下标）：双引号里的也算，单引号里的、`$(( ))` 与注释不算。
pub(super) fn substitutions(text: &[char]) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    let total = text.len();
    let mut index = 0;
    while index < total {
        let character = text[index];
        if character == '\\' {
            index += 2;
            continue;
        }
        if character == '\'' {
            index = find_character(text, '\'', index + 1).map_or(total, |end| end + 1);
            continue;
        }
        if character == '#' && (index == 0 || " \t\n;&|(".contains(text[index - 1])) {
            index = find_character(text, '\n', index).unwrap_or(total);
            continue;
        }
        if character == '"' {
            let mut inner = index + 1;
            while inner < total && text[inner] != '"' {
                if text[inner] == '\\' {
                    inner += 2;
                    continue;
                }
                if starts_with_at(text, inner, "$((") {
                    inner = matching_close(text, inner + 1, '(', ')') + 1;
                    continue;
                }
                if starts_with_at(text, inner, "$(") {
                    let end = matching_close(text, inner + 1, '(', ')');
                    found.push((text_between(text, inner + 2, end), inner + 2));
                    inner = end + 1;
                    continue;
                }
                if text[inner] == '`' {
                    let Some(end) = find_character(text, '`', inner + 1) else {
                        break;
                    };
                    found.push((text_between(text, inner + 1, end), inner + 1));
                    inner = end + 1;
                    continue;
                }
                inner += 1;
            }
            index = inner + 1;
            continue;
        }
        if starts_with_at(text, index, "$((") {
            index = matching_close(text, index + 1, '(', ')') + 1;
            continue;
        }
        if starts_with_at(text, index, "$(") {
            let end = matching_close(text, index + 1, '(', ')');
            found.push((text_between(text, index + 2, end), index + 2));
            index = end + 1;
            continue;
        }
        if character == '`' {
            let Some(end) = find_character(text, '`', index + 1) else {
                break;
            };
            found.push((text_between(text, index + 1, end), index + 1));
            index = end + 1;
            continue;
        }
        index += 1;
    }
    found
}

/// 没有展开的词 → 去掉引号后的字面值；有 `$`、反引号展开的 → `None`。双引号里的单引号是字面的，双引号里的反斜杠只在 `$`、反引号、`"`、`\`、换行前面才转义。
pub(super) fn literal_word(word: &str) -> Option<String> {
    let text = characters_of(word);
    let mut literal = String::new();
    let mut index = 0;
    let mut quoted = false;
    while index < text.len() {
        let character = text[index];
        if character == '\'' && !quoted {
            let end = find_character(&text, '\'', index + 1)?;
            literal.push_str(&text_between(&text, index + 1, end));
            index = end + 1;
            continue;
        }
        if character == '"' {
            quoted = !quoted;
            index += 1;
            continue;
        }
        if character == '$' || character == '`' {
            return None;
        }
        if character == '\\' && index + 1 < text.len() {
            if quoted && !"$`\"\\\n".contains(text[index + 1]) {
                literal.push(character);
                index += 1;
                continue;
            }
            literal.push(text[index + 1]);
            index += 2;
            continue;
        }
        literal.push(character);
        index += 1;
    }
    Some(literal)
}

/// 一个重定向：属于第几条命令、写法（`>`、`>>`、`<<` 这类）、目标词、写法在文本里起头的下标、前面写的文件描述符号。
#[derive(Clone, Debug)]
pub(super) struct Redirect {
    command_index: usize,
    operator: String,
    target: String,
    offset: usize,
    descriptor: Option<u64>,
}

/// `split_commands` 切出的简单命令（每条是词的序列，引号与 `$( … )`、`${ … }` 原样留在词里）与重定向。
/// `redirects` 只有每处重定向本身；`details` 另把写在 `{ … }` 收尾的 `}` 后面的重定向抄给组里的每一条命令（那边只在给了 `details` 时抄）。
pub(super) struct CommandSplit {
    pub(super) commands: Vec<Vec<String>>,
    pub(super) redirects: Vec<Redirect>,
    pub(super) details: Vec<Redirect>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CaseClausePart {
    Pattern,
    Body,
}

#[derive(Default)]
struct CommandSplitter {
    commands: Vec<Vec<String>>,
    words: Vec<String>,
    word: String,
    is_redirect_target: bool,
    redirect_operator: String,
    redirect_start: usize,
    redirect_descriptor: Option<u64>,
    case_clauses: Vec<CaseClausePart>,
    redirects: Vec<Redirect>,
}

impl CommandSplitter {
    fn end_word(&mut self) {
        if self.word.is_empty() {
            return;
        }
        if self.is_redirect_target {
            self.is_redirect_target = false;
            self.redirects.push(Redirect {
                command_index: self.commands.len(),
                operator: self.redirect_operator.clone(),
                target: self.word.clone(),
                offset: self.redirect_start,
                descriptor: self.redirect_descriptor,
            });
        } else {
            self.words.push(self.word.clone());
        }
        self.word.clear();
    }

    fn end_command(&mut self) {
        self.end_word();
        if self.words.is_empty() {
            return;
        }
        if self.words[0] == "case" && self.words.last().map(String::as_str) == Some("in") {
            self.case_clauses.push(CaseClausePart::Pattern);
        } else if self.words.len() == 1 && self.words[0] == "esac" && !self.case_clauses.is_empty()
        {
            self.case_clauses.pop();
        }
        self.commands.push(std::mem::take(&mut self.words));
    }
}

/// `[A-Za-z_][A-Za-z0-9_]*(?:\[[^\]]*\])?\+?=` 整个匹配：数组字面量 `x=( … )` 的左半。
fn is_array_assignment_head(word: &str) -> bool {
    let text = characters_of(word);
    let Some(mut index) = identifier_end(&text, 0) else {
        return false;
    };
    if text.get(index) == Some(&'[') {
        match find_character(&text, ']', index + 1) {
            Some(close) => index = close + 1,
            None => return false,
        }
    }
    if text.get(index) == Some(&'+') {
        index += 1;
    }
    text.get(index) == Some(&'=') && index + 1 == text.len()
}

/// 规范文本 → 简单命令：按 `;`、`&&`、`||`、`|`、`&`、`(`、`)` 与换行切，重定向与它的目标不进命令（那边的 `split_commands`）。
/// `[[ … ]]` 与 `(( … ))` 整段只留 `[[ ]]` / `((`，case 分支的模式跳过，数组字面量整个留在赋值词里，`<( … )` 里照命令切；
/// `名字[下标]=值`、`名字[下标]+=值` 下标里带空格与展开也是一个赋值词（名字紧跟 `[`，读到配对的 `]` 后接 `=` 或 `+=`）。
pub(super) fn split_commands(text: &[char]) -> CommandSplit {
    let mut splitter = CommandSplitter::default();
    let total = text.len();
    let mut index = 0;
    while index < total {
        let character = text[index];
        if splitter.case_clauses.last() == Some(&CaseClausePart::Pattern)
            && splitter.words.is_empty()
            && splitter.word.is_empty()
        {
            if " \t\n;".contains(character) {
                index += 1;
                continue;
            }
            if starts_with_at(text, index, "esac")
                && (index + 4 >= total || " \t\n;&|)".contains(text[index + 4]))
            {
                splitter.case_clauses.pop();
                index += 4;
                continue;
            }
            index = case_pattern_end(text, index) + 1;
            if let Some(last) = splitter.case_clauses.last_mut() {
                *last = CaseClausePart::Body;
            }
            continue;
        }
        if splitter.word.is_empty()
            && splitter
                .words
                .iter()
                .all(|item| SHELL_KEYWORDS.contains(&item.as_str()) || item == "for")
        {
            if starts_with_at(text, index, "[[")
                && (index + 2 >= total || " \t\n".contains(text[index + 2]))
            {
                splitter.words.push("[[".to_string());
                splitter.words.push("]]".to_string());
                index = test_command_end(text, index + 2);
                continue;
            }
            if starts_with_at(text, index, "((") {
                splitter.words.push("((".to_string());
                index = matching_close(text, index, '(', ')') + 1;
                continue;
            }
        }
        if character == ';'
            && !splitter.case_clauses.is_empty()
            && (starts_with_at(text, index, ";;") || starts_with_at(text, index, ";&"))
        {
            splitter.end_command();
            index += if starts_with_at(text, index, ";;&") {
                3
            } else {
                2
            };
            if let Some(last) = splitter.case_clauses.last_mut() {
                *last = CaseClausePart::Pattern;
            }
            continue;
        }
        if character == '(' && !splitter.word.is_empty() && is_array_assignment_head(&splitter.word)
        {
            let end = matching_close(text, index, '(', ')');
            splitter.word.push_str(&text_between(text, index, end + 1));
            index = end + 1;
            continue;
        }
        if character == '[' && is_plain_identifier(&splitter.word) {
            // 名字[下标]=值、名字[下标]+=值：下标里有空格、展开也是一个赋值词（bash 照赋值认）
            let end = matching_close(text, index, '[', ']');
            if starts_with_at(text, end + 1, "=") || starts_with_at(text, end + 1, "+=") {
                splitter.word.push_str(&text_between(text, index, end + 1));
                index = end + 1;
                continue;
            }
        }
        if character == '\\' && index + 1 < total {
            if text[index + 1] == '\n' {
                index += 2;
                continue;
            }
            splitter
                .word
                .push_str(&text_between(text, index, index + 2));
            index += 2;
            continue;
        }
        if character == '\'' {
            let end = find_character(text, '\'', index + 1).unwrap_or(total - 1);
            splitter.word.push_str(&text_between(text, index, end + 1));
            index = end + 1;
            continue;
        }
        if character == '"' {
            let end = double_quote_end(text, index);
            splitter.word.push_str(&text_between(text, index, end));
            index = end;
            continue;
        }
        if character == '$' && starts_with_at(text, index, "$(") {
            let end = matching_close(text, index + 1, '(', ')');
            splitter.word.push_str(&text_between(text, index, end + 1));
            index = end + 1;
            continue;
        }
        if character == '$' && starts_with_at(text, index, "${") {
            let end = matching_close(text, index + 1, '{', '}');
            splitter.word.push_str(&text_between(text, index, end + 1));
            index = end + 1;
            continue;
        }
        if character == '`' {
            let end = find_character(text, '`', index + 1).unwrap_or(total - 1);
            splitter.word.push_str(&text_between(text, index, end + 1));
            index = end + 1;
            continue;
        }
        if character == ' ' || character == '\t' {
            splitter.end_word();
            index += 1;
            continue;
        }
        if "\n;&|()".contains(character) {
            splitter.end_command();
            index += 1;
            continue;
        }
        if character == '<' || character == '>' {
            if text.get(index + 1) == Some(&'(') {
                // 进程替换 <( … ) / >( … )：括号里是要跑的命令，左括号在下一圈当分隔
                splitter.end_word();
                splitter.is_redirect_target = false;
                index += 1;
                continue;
            }
            splitter.redirect_descriptor = None;
            if is_ascii_digit_string(&splitter.word) {
                splitter.redirect_descriptor =
                    Some(splitter.word.parse::<u64>().unwrap_or(u64::MAX));
                splitter.word.clear();
            } else {
                splitter.end_word();
            }
            let start = index;
            while index < total && "<>&|-".contains(text[index]) {
                index += 1;
            }
            splitter.redirect_operator = text_between(text, start, index);
            splitter.redirect_start = start;
            while index < total && (text[index] == ' ' || text[index] == '\t') {
                index += 1;
            }
            splitter.is_redirect_target = true;
            continue;
        }
        if character == '#' && splitter.word.is_empty() {
            index = find_character(text, '\n', index).unwrap_or(total);
            continue;
        }
        splitter.word.push(character);
        index += 1;
    }
    splitter.end_command();
    let commands = splitter.commands;
    let redirects = splitter.redirects;
    // { … } 3<<'词'：写在收尾那个 } 后面的重定向管着组里的每一条命令
    let mut details = redirects.clone();
    let mut open_groups: Vec<usize> = Vec::new();
    let mut copies: Vec<Redirect> = Vec::new();
    for (command_index, command) in commands.iter().enumerate() {
        if command.first().map(String::as_str) == Some("{") {
            open_groups.push(command_index);
        } else if command.first().map(String::as_str) == Some("}") {
            if let Some(group_start) = open_groups.pop() {
                for detail in details
                    .iter()
                    .filter(|redirect| redirect.command_index == command_index)
                {
                    for inner in group_start..command_index {
                        copies.push(Redirect {
                            command_index: inner,
                            operator: detail.operator.clone(),
                            target: detail.target.clone(),
                            offset: detail.offset,
                            descriptor: detail.descriptor,
                        });
                    }
                }
            }
        }
    }
    details.extend(copies);
    CommandSplit {
        commands,
        redirects,
        details,
    }
}

/// case 分支模式（`text[start]` 起，可带一个开头的左括号）收尾的那个 `)` 的下标；找不到交最后一个下标。
fn case_pattern_end(text: &[char], start: usize) -> usize {
    let mut index = if starts_with_at(text, start, "(") {
        start + 1
    } else {
        start
    };
    let mut depth: usize = 0;
    while index < text.len() {
        let character = text[index];
        if character == '\\' {
            index += 2;
            continue;
        }
        if character == '\'' {
            index = find_character(text, '\'', index + 1).map_or(text.len(), |end| end + 1);
            continue;
        }
        if character == '"' {
            index = double_quote_end(text, index);
            continue;
        }
        if character == '(' {
            depth += 1;
        } else if character == ')' {
            if depth == 0 {
                return index;
            }
            depth -= 1;
        }
        index += 1;
    }
    text.len().saturating_sub(1)
}

/// `[[ … ]]`：从 `start` 起找收尾的 `]]`（前面是空白、后面是分隔或末尾），交它之后的下标；引号、`$( … )`、`${ … }` 跳过；到换行就停。
fn test_command_end(text: &[char], start: usize) -> usize {
    let mut index = start;
    while index < text.len() {
        let character = text[index];
        if character == '\\' {
            index += 2;
            continue;
        }
        if character == '\'' {
            index = find_character(text, '\'', index + 1).map_or(text.len(), |end| end + 1);
            continue;
        }
        if character == '"' {
            index = double_quote_end(text, index);
            continue;
        }
        if starts_with_at(text, index, "$(") {
            index = matching_close(text, index + 1, '(', ')') + 1;
            continue;
        }
        if starts_with_at(text, index, "${") {
            index = matching_close(text, index + 1, '{', '}') + 1;
            continue;
        }
        if starts_with_at(text, index, "]]")
            && index > 0
            && (text[index - 1] == ' ' || text[index - 1] == '\t')
            && (index + 2 >= text.len() || " \t\n;&|)".contains(text[index + 2]))
        {
            return index + 2;
        }
        if character == '\n' {
            return index;
        }
        index += 1;
    }
    text.len()
}

/// 一处 here-doc 起头 `<<词`（那边的 `HEREDOC_START`：前后不是第三个 `<`，可带 `-`，词可带一对相同的引号）。
pub(super) struct HeredocStart {
    start: usize,
    pub(super) end: usize,
    pub(super) strips_tabs: bool,
    is_quoted: bool,
    pub(super) delimiter: String,
}

pub(super) fn heredoc_start_at(row: &[char], position: usize) -> Option<HeredocStart> {
    if !starts_with_at(row, position, "<<") {
        return None;
    }
    if position > 0 && row[position - 1] == '<' {
        return None;
    }
    if row.get(position + 2) == Some(&'<') {
        return None;
    }
    let mut index = position + 2;
    let strips_tabs = row.get(index) == Some(&'-');
    if strips_tabs {
        index += 1;
    }
    while index < row.len() && is_python_whitespace(row[index]) {
        index += 1;
    }
    let quote = match row.get(index) {
        Some(&character) if character == '\'' || character == '"' => {
            index += 1;
            Some(character)
        }
        Some(_) | None => None,
    };
    let delimiter_end = identifier_end(row, index)?;
    let delimiter = text_between(row, index, delimiter_end);
    let mut end = delimiter_end;
    if let Some(quote) = quote {
        if row.get(end) != Some(&quote) {
            return None;
        }
        end += 1;
    }
    Some(HeredocStart {
        start: position,
        end,
        strips_tabs,
        is_quoted: quote.is_some(),
        delimiter,
    })
}

fn heredoc_starts(row: &[char]) -> Vec<HeredocStart> {
    let mut found = Vec::new();
    let mut position = 0;
    while position < row.len() {
        match heredoc_start_at(row, position) {
            Some(start) => {
                position = start.end;
                found.push(start);
            }
            None => position += 1,
        }
    }
    found
}

pub(super) fn row_closes_heredoc(row: &str, delimiter: &str, strips_tabs: bool) -> bool {
    if strips_tabs {
        row.trim_start_matches('\t') == delimiter
    } else {
        row == delimiter
    }
}

/// here-doc 正文那几行的下标（起止按 `<<词` 与单独一行的词认；后面找不到收尾那一行的不算）。
pub(super) fn heredoc_mask(rows: &[String]) -> HashSet<usize> {
    let mut mask = HashSet::new();
    let mut pending: VecDeque<(String, bool)> = VecDeque::new();
    for (position, row) in rows.iter().enumerate() {
        if let Some((delimiter, strips_tabs)) = pending.front() {
            mask.insert(position);
            if row_closes_heredoc(row, delimiter, *strips_tabs) {
                pending.pop_front();
            }
            continue;
        }
        for start in heredoc_starts(&characters_of(row)) {
            if rows[position + 1..]
                .iter()
                .any(|later| row_closes_heredoc(later, &start.delimiter, start.strips_tabs))
            {
                pending.push_back((start.delimiter, start.strips_tabs));
            }
        }
    }
    mask
}

/// here-doc 正文：（`<<词` 所在行，所在列）→（词带不带引号，正文）；起止的认法与 `heredoc_mask` 同一份。
pub(super) type HeredocBody = (bool, String);

pub(super) fn heredoc_bodies(rows: &[String]) -> HashMap<(usize, usize), HeredocBody> {
    struct Pending {
        place: (usize, usize),
        delimiter: String,
        strips_tabs: bool,
        is_quoted: bool,
        lines: Vec<String>,
    }
    let mut bodies = HashMap::new();
    let mut pending: VecDeque<Pending> = VecDeque::new();
    for (position, row) in rows.iter().enumerate() {
        if let Some(front) = pending.front_mut() {
            if row_closes_heredoc(row, &front.delimiter, front.strips_tabs) {
                bodies.insert(front.place, (front.is_quoted, front.lines.join("\n")));
                pending.pop_front();
            } else {
                front.lines.push(row.clone());
            }
            continue;
        }
        for start in heredoc_starts(&characters_of(row)) {
            if rows[position + 1..]
                .iter()
                .any(|later| row_closes_heredoc(later, &start.delimiter, start.strips_tabs))
            {
                pending.push_back(Pending {
                    place: (position, start.start),
                    delimiter: start.delimiter,
                    strips_tabs: start.strips_tabs,
                    is_quoted: start.is_quoted,
                    lines: Vec::new(),
                });
            }
        }
    }
    bodies
}

/// 一条命令读进来的一样东西（那边 `command_inputs` 的三种写法）。
pub(super) enum CommandInput {
    File,
    HereString,
    HereDocument(Option<HeredocBody>),
}

/// 重定向细目 → 命令下标 →（文件描述符号，读进来的东西）；`base` 是这一段在 `visible`（整个单元的规范文本，here-doc 正文已抹空）里的起点。
pub(super) fn command_inputs(
    details: &[Redirect],
    base: usize,
    visible: &[char],
    bodies: &HashMap<(usize, usize), HeredocBody>,
) -> HashMap<usize, Vec<(u64, CommandInput)>> {
    let mut inputs: HashMap<usize, Vec<(u64, CommandInput)>> = HashMap::new();
    for detail in details {
        let number = detail.descriptor.unwrap_or(0);
        let input = match detail.operator.as_str() {
            "<<" | "<<-" => {
                let absolute = base + detail.offset;
                let before = &visible[..absolute.min(visible.len())];
                let row = before
                    .iter()
                    .filter(|character| **character == '\n')
                    .count();
                let column = match before.iter().rposition(|character| *character == '\n') {
                    Some(newline) => absolute - (newline + 1),
                    None => absolute,
                };
                CommandInput::HereDocument(bodies.get(&(row, column)).cloned())
            }
            "<<<" => CommandInput::HereString,
            "<" => CommandInput::File,
            _ => continue,
        };
        inputs
            .entry(detail.command_index)
            .or_default()
            .push((number, input));
    }
    inputs
}

/// 单元自己写出来、拷出来的脚本（那边的 `generated_scripts`）。
#[derive(Clone, Debug)]
pub(super) enum GeneratedScript {
    /// 用 `printf` / `echo` / 只接 here-doc 的 `cat` 写出来的：`cat <<词` 写进去的正文（`printf` / `echo` 写的不当代码看）。
    Written(Vec<String>),
    /// 只由 `cp <源> <它>` 拷出来的：源词。
    Copied(Vec<String>),
}

pub(super) fn generated_scripts(
    commands: &[Vec<String>],
    redirects: &[Redirect],
    rows: &[String],
    mask: &HashSet<usize>,
) -> HashMap<String, GeneratedScript> {
    let mut writers: HashMap<String, Vec<Option<String>>> = HashMap::new();
    for redirect in redirects {
        let operator = redirect.operator.as_str();
        if !operator.starts_with('>')
            || (operator.starts_with(">&") && is_ascii_digit_string(&redirect.target))
        {
            continue;
        }
        let command: &[String] = commands
            .get(redirect.command_index)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let head = command.first().and_then(|word| literal_word(word));
        let operators: HashSet<&str> = redirects
            .iter()
            .filter(|other| other.command_index == redirect.command_index)
            .map(|other| other.operator.as_str())
            .collect();
        let is_generator = head
            .as_deref()
            .is_some_and(|head_word| GENERATOR_HEADS.contains(&head_word))
            || (head.as_deref() == Some("cat")
                && command.len() == 1
                && operators
                    .iter()
                    .any(|other| *other == "<<" || *other == "<<-"));
        writers
            .entry(redirect.target.replace('"', ""))
            .or_default()
            .push(if is_generator { head } else { None });
    }
    let mut copies: HashMap<String, Vec<Option<String>>> = HashMap::new();
    for command in commands {
        let head = command.first().and_then(|word| literal_word(word));
        let Some(head) = head.filter(|head_word| COPYING_HEADS.contains(&head_word.as_str()))
        else {
            continue;
        };
        let operands: Vec<&String> = command[1..]
            .iter()
            .filter(|argument| !literal_word(argument).unwrap_or_default().starts_with('-'))
            .collect();
        if head == "cp" && operands.len() == 2 {
            copies
                .entry(operands[1].replace('"', ""))
                .or_default()
                .push(Some(operands[0].clone()));
            continue;
        }
        for argument in &command[1..] {
            writers
                .entry(argument.replace('"', ""))
                .or_default()
                .push(None);
        }
    }
    for target in writers.keys() {
        if let Some(sources) = copies.get_mut(target) {
            sources.push(None);
        }
    }
    let mut generated: HashMap<String, GeneratedScript> = HashMap::new();
    for (target, heads) in &writers {
        if !heads.is_empty() && heads.iter().all(Option::is_some) && !copies.contains_key(target) {
            generated.insert(target.clone(), GeneratedScript::Written(Vec::new()));
        }
    }
    for (target, sources) in copies {
        if sources.iter().all(Option::is_some) {
            generated.insert(
                target,
                GeneratedScript::Copied(sources.into_iter().flatten().collect()),
            );
        }
    }
    for (position, row) in rows.iter().enumerate() {
        if mask.contains(&position) {
            continue;
        }
        let row_characters = characters_of(row);
        let starts = heredoc_starts(&row_characters);
        if starts.len() != 1 {
            continue;
        }
        let delimiter = &starts[0].delimiter;
        let strips_tabs = starts[0].strips_tabs;
        let mut body: Vec<&str> = Vec::new();
        let mut later = position + 1;
        while later < rows.len() && !row_closes_heredoc(&rows[later], delimiter, strips_tabs) {
            body.push(&rows[later]);
            later += 1;
        }
        if later >= rows.len() {
            continue;
        }
        let row_split = split_commands(&row_characters);
        for redirect in &row_split.redirects {
            let key = redirect.target.replace('"', "");
            let is_cat = row_split
                .commands
                .get(redirect.command_index)
                .and_then(|command| command.first())
                .and_then(|word| literal_word(word))
                .as_deref()
                == Some("cat");
            if redirect.operator.starts_with('>') && is_cat {
                if let Some(GeneratedScript::Written(bodies)) = generated.get_mut(&key) {
                    bodies.push(body.join("\n"));
                }
            }
        }
    }
    generated
}

#[cfg(test)]
mod tests {
    use super::{characters_of, literal_word, split_commands};

    #[test]
    fn literal_words_drop_quotes_and_refuse_expansions() {
        assert_eq!(literal_word("'a b'\"c\"").as_deref(), Some("a bc"));
        assert_eq!(literal_word("\"a\\qb\"").as_deref(), Some("a\\qb"));
        assert_eq!(literal_word("a\\ b").as_deref(), Some("a b"));
        assert_eq!(literal_word("\"$x\""), None);
        assert_eq!(literal_word("'unclosed"), None);
    }

    #[test]
    fn commands_split_on_separators_and_keep_substitutions_inside_words() {
        let split = split_commands(&characters_of(
            "x=\"$(cd \"$(dirname a)\" && pwd)\"; source \"$x/l.sh\" > /dev/null # 注释 source y\n[[ -f a ]] && . b",
        ));
        assert_eq!(
            split.commands,
            vec![
                vec!["x=\"$(cd \"$(dirname a)\" && pwd)\"".to_string()],
                vec!["source".to_string(), "\"$x/l.sh\"".to_string()],
                vec!["[[".to_string(), "]]".to_string()],
                vec![".".to_string(), "b".to_string()],
            ]
        );
        assert_eq!(split.redirects.len(), 1);
        assert_eq!(split.redirects[0].target, "/dev/null");
    }

    /// 样本照 `scripts/gate/lib_function_units.py` 自证里的 `s/subscript.sh`，另加下标带空格的 `+=` 与后面跟着别的赋值的一行；
    /// 下标后面不接 `=` 的照旧按空白切。
    #[test]
    fn subscript_assignments_with_spaces_and_expansions_in_the_subscript_stay_one_assignment_word()
    {
        let split = split_commands(&characters_of(
            "item[$key $address]=\"$value\"\nitem[$key]+=\" more\"\nitem[$key $address]+=\" more\" library=l.sh\necho item[$key $address]\n",
        ));
        assert_eq!(
            split.commands,
            vec![
                vec!["item[$key $address]=\"$value\"".to_string()],
                vec!["item[$key]+=\" more\"".to_string()],
                vec![
                    "item[$key $address]+=\" more\"".to_string(),
                    "library=l.sh".to_string(),
                ],
                vec![
                    "echo".to_string(),
                    "item[$key".to_string(),
                    "$address]".to_string(),
                ],
            ]
        );
    }
}
