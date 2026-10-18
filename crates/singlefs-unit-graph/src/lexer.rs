//! 词法：Rust 与 WGSL 共用这一个切词器（WGSL 的字面量是 Rust 的子集）。
//!
//! 去掉空白、`//` 行注释、可嵌套的 `/* */` 块注释与四种文档注释（`///`、`//!`、`/** */`、`/*! */`）；
//! 串、字节串、C 串、原始串（带 `#` 的也算）、字符、字节字符、生命周期、数字（带指数、后缀与进制前缀）各算一个词法单元，
//! 其余标点一个字符一个单元。文档注释的正文另收一份（文档测试单元从这里取代码块），默认不进单元串，所以只改文档注释不改指纹。

/// 一个词法单元是哪一类。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Literal,
    Lifetime,
    Punctuation,
}

/// 一个词法单元：在源码里的字节区间与它起头那一行（从 1 数）。
#[derive(Clone, Copy, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub start_byte: u32,
    pub end_byte: u32,
    pub line: u32,
}

/// 文档注释挂在谁身上：`///`、`/** */` 挂后面那个条目；`//!`、`/*! */` 挂包住它的模块。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentationPlacement {
    FollowingItem,
    EnclosingModule,
}

/// 一段文档注释的正文（去掉注释记号），按行切开。
#[derive(Clone, Debug)]
pub struct DocumentationComment {
    pub placement: DocumentationPlacement,
    pub body_lines: Vec<String>,
    pub first_line: u32,
    /// 这段注释之后的第一个词法单元的下标（注释在文件末尾时等于词法单元总数）。
    pub next_token_index: usize,
}

/// 文档注释进不进词法单元串。只有弄坏开关 `documentation-comments-in-fingerprint` 让它进。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentationCommentTreatment {
    KeptOutOfTheTokens,
    KeptAsTokens,
}

/// 一份源码切出来的词法单元与文档注释。
#[derive(Clone, Debug)]
pub struct LexedSource {
    pub tokens: Vec<Token>,
    pub documentation_comments: Vec<DocumentationComment>,
}

/// 一个词法单元的原文。
#[must_use]
pub fn token_text<'source>(source: &'source str, token: &Token) -> &'source str {
    let start = usize::try_from(token.start_byte).expect("字节偏移放得进 usize");
    let end = usize::try_from(token.end_byte).expect("字节偏移放得进 usize");
    &source[start..end]
}

/// 标识符的名字：原始标识符 `r#type` 的名字是 `type`。
#[must_use]
pub fn identifier_name(identifier_text: &str) -> &str {
    identifier_text
        .strip_prefix("r#")
        .unwrap_or(identifier_text)
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte >= 0x80
}

fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 0x80
}

fn byte_offset(position: usize) -> u32 {
    u32::try_from(position).expect("一份源码小于 4 GiB，字节偏移放得进 u32")
}

fn newline_count(bytes: &[u8]) -> u32 {
    u32::try_from(bytes.iter().filter(|byte| **byte == b'\n').count())
        .expect("一份源码的行数放得进 u32")
}

/// 从开引号起到配对的闭引号之后；反斜杠转义下一个字节。没闭上就到文件末尾。
fn end_of_quoted(bytes: &[u8], opening_quote_index: usize, quote: u8) -> usize {
    let mut position = opening_quote_index + 1;
    while position < bytes.len() {
        let byte = bytes[position];
        if byte == b'\\' {
            position += 2;
            continue;
        }
        if byte == quote {
            return position + 1;
        }
        position += 1;
    }
    bytes.len()
}

/// `raw_marker_index` 指的那个 `r` 之后若是若干 `#` 再接 `"`，返回整个原始串之后的位置；不是原始串返回 `None`。
fn end_of_raw_string(bytes: &[u8], raw_marker_index: usize) -> Option<usize> {
    let mut position = raw_marker_index + 1;
    let mut hash_count = 0usize;
    while bytes.get(position) == Some(&b'#') {
        hash_count += 1;
        position += 1;
    }
    if bytes.get(position) != Some(&b'"') {
        return None;
    }
    position += 1;
    while position < bytes.len() {
        if bytes[position] == b'"'
            && (0..hash_count).all(|offset| bytes.get(position + 1 + offset) == Some(&b'#'))
        {
            return Some(position + 1 + hash_count);
        }
        position += 1;
    }
    Some(bytes.len())
}

/// 可嵌套块注释之后的位置；没闭上就到文件末尾。
fn end_of_block_comment(bytes: &[u8], start: usize) -> usize {
    let mut depth = 0usize;
    let mut position = start;
    while position + 1 < bytes.len() {
        if bytes[position] == b'/' && bytes[position + 1] == b'*' {
            depth += 1;
            position += 2;
        } else if bytes[position] == b'*' && bytes[position + 1] == b'/' {
            depth -= 1;
            position += 2;
            if depth == 0 {
                return position;
            }
        } else {
            position += 1;
        }
    }
    bytes.len()
}

fn end_of_identifier(bytes: &[u8], start: usize) -> usize {
    let mut position = start;
    while position < bytes.len() && is_identifier_continue(bytes[position]) {
        position += 1;
    }
    position
}

fn end_of_number(bytes: &[u8], start: usize) -> usize {
    let has_radix_prefix =
        bytes[start] == b'0' && matches!(bytes.get(start + 1), Some(b'x' | b'X' | b'b' | b'o'));
    let mut position = start + 1;
    let mut has_fraction_point = false;
    while position < bytes.len() {
        let byte = bytes[position];
        if is_identifier_continue(byte) {
            position += 1;
            continue;
        }
        if byte == b'.'
            && !has_radix_prefix
            && !has_fraction_point
            && bytes.get(position + 1).is_some_and(u8::is_ascii_digit)
        {
            has_fraction_point = true;
            position += 1;
            continue;
        }
        if (byte == b'+' || byte == b'-')
            && !has_radix_prefix
            && matches!(bytes[position - 1], b'e' | b'E')
        {
            position += 1;
            continue;
        }
        break;
    }
    position
}

/// 一个 UTF-8 字符占几个字节（按起头那个字节判）。
fn utf8_character_width(first_byte: u8) -> usize {
    if first_byte < 0x80 {
        1
    } else if first_byte >= 0xF0 {
        4
    } else if first_byte >= 0xE0 {
        3
    } else {
        2
    }
}

/// 一段注释是不是文档注释、是哪一种、正文是哪几行。`comment` 是整段注释的原文（含记号）。
fn documentation_comment_of(comment: &str) -> Option<(DocumentationPlacement, Vec<String>)> {
    if let Some(body) = comment.strip_prefix("///") {
        if body.starts_with('/') {
            return None;
        }
        return Some((
            DocumentationPlacement::FollowingItem,
            vec![body.to_string()],
        ));
    }
    if let Some(body) = comment.strip_prefix("//!") {
        return Some((
            DocumentationPlacement::EnclosingModule,
            vec![body.to_string()],
        ));
    }
    if comment.starts_with("/**") && !comment.starts_with("/***") && comment != "/**/" {
        let body = comment
            .strip_prefix("/**")
            .and_then(|rest| rest.strip_suffix("*/"))
            .unwrap_or("");
        return Some((
            DocumentationPlacement::FollowingItem,
            body.split('\n').map(str::to_string).collect(),
        ));
    }
    if let Some(rest) = comment.strip_prefix("/*!") {
        let body = rest.strip_suffix("*/").unwrap_or(rest);
        return Some((
            DocumentationPlacement::EnclosingModule,
            body.split('\n').map(str::to_string).collect(),
        ));
    }
    None
}

/// 把一份源码切成词法单元，另收文档注释。
#[must_use]
pub fn lex_source(source: &str, treatment: DocumentationCommentTreatment) -> LexedSource {
    let bytes = source.as_bytes();
    let mut tokens = Vec::with_capacity(bytes.len() / 4);
    let mut documentation_comments = Vec::new();
    let mut position = 0usize;
    let mut line = 1u32;
    while position < bytes.len() {
        let current = bytes[position];
        let next = bytes.get(position + 1).copied();
        if current == b'\n' {
            line += 1;
            position += 1;
            continue;
        }
        if current.is_ascii_whitespace() {
            position += 1;
            continue;
        }
        if current == b'/' && (next == Some(b'/') || next == Some(b'*')) {
            let end = if next == Some(b'/') {
                bytes[position..]
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(bytes.len(), |offset| position + offset)
            } else {
                end_of_block_comment(bytes, position)
            };
            let comment = &source[position..end];
            if let Some((placement, body_lines)) = documentation_comment_of(comment) {
                documentation_comments.push(DocumentationComment {
                    placement,
                    body_lines,
                    first_line: line,
                    next_token_index: tokens.len(),
                });
                if treatment == DocumentationCommentTreatment::KeptAsTokens {
                    tokens.push(Token {
                        kind: TokenKind::Literal,
                        start_byte: byte_offset(position),
                        end_byte: byte_offset(end),
                        line,
                    });
                }
            }
            line += newline_count(&bytes[position..end]);
            position = end;
            continue;
        }
        let start = position;
        let kind;
        if let Some(end) = prefixed_or_raw_string_end(bytes, position) {
            kind = TokenKind::Literal;
            position = end;
        } else if current == b'r'
            && next == Some(b'#')
            && bytes
                .get(position + 2)
                .copied()
                .is_some_and(is_identifier_start)
        {
            kind = TokenKind::Identifier;
            position = end_of_identifier(bytes, position + 2);
        } else if current == b'"' {
            kind = TokenKind::Literal;
            position = end_of_quoted(bytes, position, b'"');
        } else if current == b'\'' {
            if next == Some(b'\\') {
                kind = TokenKind::Literal;
                position = end_of_quoted(bytes, position, b'\'');
            } else {
                let character_width = next.map_or(1, utf8_character_width);
                if bytes.get(position + 1 + character_width) == Some(&b'\'') {
                    kind = TokenKind::Literal;
                    position += 2 + character_width;
                } else {
                    kind = TokenKind::Lifetime;
                    position = end_of_identifier(bytes, position + 1);
                }
            }
        } else if is_identifier_start(current) {
            // 非 ASCII 起头的名字也是标识符（与 `r#中文`、`_中文` 一致；Python 那一份 `lib_function_units.py` 同一条判法）
            kind = TokenKind::Identifier;
            position = end_of_identifier(bytes, position);
        } else if current.is_ascii_digit() {
            kind = TokenKind::Literal;
            position = end_of_number(bytes, position);
        } else {
            kind = TokenKind::Punctuation;
            position += utf8_character_width(current);
        }
        let end = position.min(bytes.len());
        position = end;
        tokens.push(Token {
            kind,
            start_byte: byte_offset(start),
            end_byte: byte_offset(end),
            line,
        });
        line += newline_count(&bytes[start..end]);
    }
    LexedSource {
        tokens,
        documentation_comments,
    }
}

/// 带前缀的串（`b"…"`、`b'…'`、`c"…"`）与原始串（`r"…"`、`r#"…"#`、`br…`、`cr…`）之后的位置；不是这几种返回 `None`。
fn prefixed_or_raw_string_end(bytes: &[u8], position: usize) -> Option<usize> {
    let current = bytes[position];
    let next = bytes.get(position + 1).copied();
    match current {
        b'r' => end_of_raw_string(bytes, position),
        b'b' | b'c' => match next {
            Some(b'"') => Some(end_of_quoted(bytes, position + 1, b'"')),
            Some(b'\'') if current == b'b' => Some(end_of_quoted(bytes, position + 1, b'\'')),
            Some(b'r') => end_of_raw_string(bytes, position + 1),
            _ => None,
        },
        _ => None,
    }
}

/// 每个开括号配对的闭括号下标；括号配不上时返回第一个出错的词法单元下标。
///
/// # Errors
/// 闭括号没有开括号、闭括号与开括号不是一对、或到文件末尾还有没闭上的开括号，交出错处的词法单元下标。
pub fn matching_brackets(source: &str, tokens: &[Token]) -> Result<Vec<Option<usize>>, usize> {
    let mut closes = vec![None; tokens.len()];
    let mut open_stack: Vec<(usize, u8)> = Vec::new();
    for (token_index, token) in tokens.iter().enumerate() {
        if token.kind != TokenKind::Punctuation {
            continue;
        }
        let text = token_text(source, token).as_bytes();
        let Some(&bracket) = text.first() else {
            continue;
        };
        match bracket {
            b'(' | b'[' | b'{' => open_stack.push((token_index, bracket)),
            b')' | b']' | b'}' => {
                let expected_open = match bracket {
                    b')' => b'(',
                    b']' => b'[',
                    _ => b'{',
                };
                match open_stack.pop() {
                    Some((open_index, open_bracket)) if open_bracket == expected_open => {
                        closes[open_index] = Some(token_index);
                    }
                    Some(_) | None => return Err(token_index),
                }
            }
            _ => {}
        }
    }
    match open_stack.first() {
        Some((open_index, _)) => Err(*open_index),
        None => Ok(closes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(source: &str) -> Vec<String> {
        lex_source(source, DocumentationCommentTreatment::KeptOutOfTheTokens)
            .tokens
            .iter()
            .map(|token| token_text(source, token).to_string())
            .collect()
    }

    #[test]
    fn comments_and_documentation_comments_leave_no_tokens() {
        let source = "/// 文档\n//! 模块文档\n// 普通\n/* 外 /* 嵌套 */ 还在外 */ /** 块文档 */ /*! 块模块文档 */ fn a() {}";
        assert_eq!(texts(source), ["fn", "a", "(", ")", "{", "}"]);
        let lexed = lex_source(source, DocumentationCommentTreatment::KeptOutOfTheTokens);
        assert_eq!(
            lexed.documentation_comments.len(),
            4,
            "四种文档注释各一段：///、//!、/** */、/*! */"
        );
    }

    /// 非 ASCII 起头的名字切成一个标识符 token，与 `_中文`、`r#中文` 一致（早先多了一个「起头字节小于 0x80」的条件，「中文名」被按字节切成标点）。
    #[test]
    fn identifiers_starting_with_a_non_ascii_character_are_one_token_like_raw_and_underscore_ones()
    {
        assert_eq!(
            texts("fn 中文名() {} fn _中文() {} fn r#中文() {}"),
            [
                "fn",
                "中文名",
                "(",
                ")",
                "{",
                "}",
                "fn",
                "_中文",
                "(",
                ")",
                "{",
                "}",
                "fn",
                "r#中文",
                "(",
                ")",
                "{",
                "}"
            ]
        );
    }

    #[test]
    fn strings_characters_lifetimes_and_numbers_are_one_token_each() {
        let source = r####"r#"原始 "串""# b"字节" c"C 串" br##"x"## 'a' '\n' '中' b'q' 'life 1.5e-3_f64 0x1f r#type"####;
        assert_eq!(
            texts(source),
            [
                r##"r#"原始 "串""#"##,
                "b\"字节\"",
                "c\"C 串\"",
                r###"br##"x"##"###,
                "'a'",
                "'\\n'",
                "'中'",
                "b'q'",
                "'life",
                "1.5e-3_f64",
                "0x1f",
                "r#type"
            ]
        );
    }

    #[test]
    fn a_range_after_a_number_is_not_part_of_the_number() {
        assert_eq!(texts("0..10"), ["0", ".", ".", "10"]);
        assert_eq!(texts("tuple.0.1"), ["tuple", ".", "0.1"]);
    }

    #[test]
    fn lines_count_from_one_and_follow_multi_line_tokens() {
        let source = "a\n\"多\n行\"\nb";
        let lexed = lex_source(source, DocumentationCommentTreatment::KeptOutOfTheTokens);
        let lines: Vec<u32> = lexed.tokens.iter().map(|token| token.line).collect();
        assert_eq!(lines, [1, 2, 4]);
    }

    #[test]
    fn unmatched_brackets_are_reported_at_the_offending_token() {
        let source = "fn a() { ( }";
        let lexed = lex_source(source, DocumentationCommentTreatment::KeptOutOfTheTokens);
        assert_eq!(matching_brackets(source, &lexed.tokens), Err(6));
        let balanced = "fn a() { [()] }";
        let lexed_balanced =
            lex_source(balanced, DocumentationCommentTreatment::KeptOutOfTheTokens);
        let closes = matching_brackets(balanced, &lexed_balanced.tokens).expect("括号配得上");
        assert_eq!(closes[4], Some(9));
    }
}
