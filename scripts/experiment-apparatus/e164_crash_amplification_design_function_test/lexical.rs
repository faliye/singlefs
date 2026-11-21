//! E164 的词法摘要与四种节点代码取法（跑前登记 5.4「词法摘要」「节点代码：按函数 / 按走到的文件」「读自己原文的目标」）。
//! 不链接 rustc 的词法器：照 5.4 的字面手写一份，锚点在跑前登记 7.2 前五行。

use crate::common::sha256_hexadecimal;

/// 一个词法单元：文本取源码原文。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexicalToken {
    pub text: String,
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character.is_alphabetic()
}

fn is_identifier_continue(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}

/// 从 `start` 起（指向开引号）读一个带转义的串，交回结束之后的下标。
fn end_of_escaped_quoted(characters: &[char], start: usize, quote: char) -> usize {
    let mut position = start + 1;
    while position < characters.len() {
        if characters[position] == '\\' {
            position += 2;
            continue;
        }
        if characters[position] == quote {
            return position + 1;
        }
        position += 1;
    }
    characters.len()
}

/// 从 `start` 起（指向 `r`）读原始串 `r#…#"…"#…#`，交回结束之后的下标；不是原始串交回 None。
fn end_of_raw_string(characters: &[char], start: usize) -> Option<usize> {
    let mut position = start + 1;
    let mut hashes = 0usize;
    while position < characters.len() && characters[position] == '#' {
        hashes += 1;
        position += 1;
    }
    if position >= characters.len() || characters[position] != '"' {
        return None;
    }
    position += 1;
    while position < characters.len() {
        if characters[position] == '"'
            && (0..hashes).all(|offset| characters.get(position + 1 + offset) == Some(&'#'))
        {
            return Some(position + 1 + hashes);
        }
        position += 1;
    }
    Some(characters.len())
}

/// 从 `start` 起跳过可嵌套的块注释，交回结束之后的下标。
fn end_of_block_comment(characters: &[char], start: usize) -> usize {
    let mut depth = 0usize;
    let mut position = start;
    while position + 1 < characters.len() {
        if characters[position] == '/' && characters[position + 1] == '*' {
            depth += 1;
            position += 2;
            continue;
        }
        if characters[position] == '*' && characters[position + 1] == '/' {
            depth -= 1;
            position += 2;
            if depth == 0 {
                return position;
            }
            continue;
        }
        position += 1;
    }
    characters.len()
}

/// 5.4「词法摘要」：去空白、行注释、块注释（可嵌套）与全部文档注释；串、字节串、原始串、字符、生命周期各算一个单元。
pub fn lexical_tokens(source: &str) -> Vec<LexicalToken> {
    let characters: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut position = 0usize;
    while position < characters.len() {
        let current = characters[position];
        let next = characters.get(position + 1).copied();
        if current.is_whitespace() {
            position += 1;
            continue;
        }
        if current == '/' && next == Some('/') {
            while position < characters.len() && characters[position] != '\n' {
                position += 1;
            }
            continue;
        }
        if current == '/' && next == Some('*') {
            position = end_of_block_comment(&characters, position);
            continue;
        }
        let token_start = position;
        if (current == 'r' || (current == 'b' && next == Some('r')))
            && end_of_raw_string(&characters, if current == 'b' { position + 1 } else { position }).is_some()
        {
            position = end_of_raw_string(&characters, if current == 'b' { position + 1 } else { position })
                .expect("上一行判过是原始串");
        } else if current == 'b' && (next == Some('"') || next == Some('\'')) {
            position = end_of_escaped_quoted(&characters, position + 1, next.expect("判过是引号"));
        } else if current == '"' {
            position = end_of_escaped_quoted(&characters, position, '"');
        } else if current == '\'' {
            position = end_of_character_or_lifetime(&characters, position);
        } else if current == 'r' && next == Some('#') && characters.get(position + 2).is_some_and(|character| is_identifier_start(*character)) {
            position += 2;
            while position < characters.len() && is_identifier_continue(characters[position]) {
                position += 1;
            }
        } else if is_identifier_start(current) {
            while position < characters.len() && is_identifier_continue(characters[position]) {
                position += 1;
            }
        } else if current.is_ascii_digit() {
            position = end_of_number(&characters, position);
        } else {
            position += 1;
        }
        tokens.push(LexicalToken {
            text: characters[token_start..position].iter().collect(),
        });
    }
    tokens
}

/// `'` 起头：`'\…'` 与 `'x'` 是字符，其余是生命周期。
fn end_of_character_or_lifetime(characters: &[char], start: usize) -> usize {
    if characters.get(start + 1) == Some(&'\\') {
        return end_of_escaped_quoted(characters, start, '\'');
    }
    if characters.get(start + 2) == Some(&'\'') {
        return start + 3;
    }
    let mut position = start + 1;
    while position < characters.len() && is_identifier_continue(characters[position]) {
        position += 1;
    }
    position
}

/// 数字字面量：数字起头，接字母数字下划线，`.` 后面紧跟数字时连上。
fn end_of_number(characters: &[char], start: usize) -> usize {
    let mut position = start;
    while position < characters.len() {
        let character = characters[position];
        let continues_with_fraction = character == '.'
            && characters.get(position + 1).is_some_and(char::is_ascii_digit);
        if is_identifier_continue(character) || continues_with_fraction {
            position += 1;
        } else {
            break;
        }
    }
    position
}

pub fn token_text(tokens: &[LexicalToken]) -> String {
    tokens.iter().map(|token| token.text.as_str()).collect::<Vec<&str>>().join(" ")
}

/// 一份源码的词法摘要：单元串用一个空格连起来取 SHA-256。
pub fn lexical_digest(source: &str) -> String {
    sha256_hexadecimal(token_text(&lexical_tokens(source)).as_bytes())
}

/// 多份文件的拼法：按仓内相对路径字典序，每份拼「路径 + 换行 + 内容串 + 换行」，整串取 SHA-256。
/// `per_file` 给每份文件交回它那一段内容串（词法单元串、原文摘要，或别的取法）。
pub fn combined_digest(files: &[(String, String)]) -> String {
    let mut sorted: Vec<&(String, String)> = files.iter().collect();
    sorted.sort_by(|left, right| left.0.cmp(&right.0));
    let mut joined = String::new();
    for (path, content_text) in sorted {
        joined.push_str(path);
        joined.push('\n');
        joined.push_str(content_text);
        joined.push('\n');
    }
    sha256_hexadecimal(joined.as_bytes())
}

/// 从 `start`（一个开括号 `{`）找到配对的 `}`，交回它的下标。
fn matching_close_brace(tokens: &[LexicalToken], start: usize) -> usize {
    let mut depth = 0usize;
    for (position, token) in tokens.iter().enumerate().skip(start) {
        match token.text.as_str() {
            "{" => depth += 1,
            "}" => {
                depth -= 1;
                if depth == 0 {
                    return position;
                }
            }
            _ => {}
        }
    }
    tokens.len() - 1
}

/// 从 `start` 起找一个项的末尾：先遇到 `;` 就到 `;`，先遇到 `{` 就到配对的 `}`。
fn end_of_item(tokens: &[LexicalToken], start: usize) -> usize {
    for position in start..tokens.len() {
        match tokens[position].text.as_str() {
            ";" => return position,
            "{" => return matching_close_brace(tokens, position),
            _ => {}
        }
    }
    tokens.len() - 1
}

/// `#[cfg(test)]` 或 `#[test]` 这一个属性从 `position` 起，交回属性末尾（`]`）的下标。
fn test_attribute_end(tokens: &[LexicalToken], position: usize) -> Option<usize> {
    let texts: Vec<&str> = tokens[position..tokens.len().min(position + 7)].iter().map(|token| token.text.as_str()).collect();
    if texts.starts_with(&["#", "[", "cfg", "(", "test", ")", "]"]) {
        return Some(position + 6);
    }
    if texts.starts_with(&["#", "[", "test", "]"]) {
        return Some(position + 3);
    }
    None
}

/// 5.4「节点代码：按函数」：非测试 fn 项（从 `fn` 到体的配对 `}`）的单元，按次序；
/// 带 `#[cfg(test)]` 或 `#[test]` 的项（测试模块、测试函数）整个跳过；`fn(` 这类函数指针类型不算项。
pub fn non_test_function_items(tokens: &[LexicalToken]) -> Vec<Vec<LexicalToken>> {
    let mut items = Vec::new();
    let mut position = 0usize;
    let mut next_item_is_test = false;
    while position < tokens.len() {
        if let Some(attribute_end) = test_attribute_end(tokens, position) {
            next_item_is_test = true;
            position = attribute_end + 1;
            continue;
        }
        let text = tokens[position].text.as_str();
        if text == "#" {
            position += 1;
            continue;
        }
        if next_item_is_test && (text == "mod" || text == "fn") {
            position = end_of_item(tokens, position) + 1;
            next_item_is_test = false;
            continue;
        }
        let names_a_function = text == "fn"
            && tokens.get(position + 1).is_some_and(|name| name.text != "(");
        if names_a_function {
            let end = end_of_item(tokens, position);
            items.push(tokens[position..=end].to_vec());
            position = end + 1;
            continue;
        }
        position += 1;
    }
    items
}

/// 按函数取时一份文件的内容串：各非测试 fn 项的单元串按次序用空格连起来。
pub fn function_items_text(source: &str) -> String {
    non_test_function_items(&lexical_tokens(source))
        .iter()
        .map(|item| token_text(item))
        .collect::<Vec<String>>()
        .join(" ")
}

/// 按走到的文件取时这份文件进不进：至少含一个非测试 fn 项。
pub fn has_non_test_function(source: &str) -> bool {
    !non_test_function_items(&lexical_tokens(source)).is_empty()
}

/// 5.4「读自己原文的目标」：源码里出现 `include_str!`、`include_bytes!`、`file!()`，或以字面路径读仓内 `.rs`，
/// 交回它读的字面路径（`file!()` 交回 `<file!>`，由调用方换成这份文件自己）。
pub fn own_source_reads(source: &str) -> Vec<String> {
    let tokens = lexical_tokens(source);
    let mut reads = Vec::new();
    for (position, token) in tokens.iter().enumerate() {
        let literal_argument = tokens.get(position + 3).map(|argument| argument.text.clone());
        match token.text.as_str() {
            "include_str" | "include_bytes" if tokens.get(position + 1).is_some_and(|bang| bang.text == "!") => {
                if let Some(argument) = literal_argument.filter(|argument| argument.starts_with('"')) {
                    reads.push(argument.trim_matches('"').to_string());
                }
            }
            "file" if tokens.get(position + 1).is_some_and(|bang| bang.text == "!") => reads.push("<file!>".to_string()),
            "read_to_string" | "read" => {
                let argument = tokens.get(position + 2).map(|argument| argument.text.clone());
                if let Some(literal) = argument.filter(|literal| literal.starts_with('"') && literal.ends_with(".rs\"")) {
                    reads.push(literal.trim_matches('"').to_string());
                }
            }
            _ => {}
        }
    }
    reads
}
