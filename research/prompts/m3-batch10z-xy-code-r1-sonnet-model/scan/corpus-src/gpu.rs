

    /// `text` 以 `"` 起头时，那一个字符串字面量连两头引号的长度（反斜杠连同它转义的那个字跳过）。
    fn length_of_the_quoted_literal_at_the_start(text: &str) -> usize {
        let bytes = text.as_bytes();
        let mut position = 1;
        loop {
            let byte = *bytes.get(position).expect("字符串字面量有收尾的引号");
            if byte == b'"' {
                return position + 1;
            }
            position += if byte == b'\\' { 2 } else { 1 };
        }
    }

    /// `text` 以 `'` 起头时，那一个字符字面量连两头引号的长度；是生命周期（`'static`、`'_`）交 `None`。
    fn length_of_the_character_literal_at_the_start(text: &str) -> Option<usize> {
        let after_the_quote = &text[1..];
        if after_the_quote.starts_with('\\') {
            // 转义的（`'\''`、`'\n'`、`'\u{7b}'`）：收尾的引号在反斜杠与它后面那个字之后
            let closing_quote = 3 + text[3..].find('\'').expect("转义的字符字面量有收尾的引号");
            return Some(closing_quote + 1);
        }
        let first_character = after_the_quote.chars().next().expect("引号后面还有字");
        after_the_quote[first_character.len_utf8()..]
            .starts_with('\'')
            .then(|| first_character.len_utf8() + 2)
    }

    /// `text` 是原始字符串去掉 `r`、`br`、`cr` 之后那一截（以若干个 `#` 与一个 `"` 起头）时，到收尾的 `"` 与同样多个 `#` 为止的长度。
    fn length_of_the_raw_string_at_the_start(text: &str) -> usize {
        let hashes = text.len() - text.trim_start_matches('#').len();
        let closing = format!("\"{}", "#".repeat(hashes));
        let body_start = hashes + 1;
        body_start + text[body_start..].find(&closing).expect("原始字符串有收尾") + closing.len()
    }

    /// `text` 以 `/*` 起头时，那一段块注释（可以嵌套）的长度。
    fn length_of_the_block_comment_at_the_start(text: &str) -> usize {
        let mut depth = 0_usize;
        let mut position = 0;
        loop {
            let rest = &text[position..];
            if rest.starts_with("/*") {
                depth += 1;
                position += 2;
            } else if rest.starts_with("*/") {
                depth -= 1;
                position += 2;
                if depth == 0 {
                    return position;
                }
            } else {
                position += rest.chars().next().expect("块注释有收尾").len_utf8();
            }
        }
    }

    /// 拼成关键字的原始标识符（`r#mod`、`r#use`）是名字，不是关键字：分词时这几个连着 `r#` 出词，免得被当成关键字（第 10z 批第二十三块，
    /// 正推 K4-2 L14）；别的原始标识符（`r#std`）与不带 `r#` 的同名，取它后面的名字。
    const KEYWORDS_A_RAW_IDENTIFIER_CAN_SPELL: [&str; 48] = [
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do",
        "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in",
        "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
        "return", "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe",
        "unsized", "use", "virtual", "where", "while", "yield",
    ];

    /// `code` 拆成的词：标识符与关键字（`r#` 原始标识符取它后面的名字，拼成关键字的连着 `r#`）、生命周期（连着它前面的 `'`）、
    /// 数字（照标识符那样收一段）、`::`、`!=`，其余标点一个字一个词；注释（行注释、可以嵌套的块注释）、字符串与字符字面量
    /// （普通的、原始的 `r#"…"#`、`br`、`cr`，字节的、C 的）不出词。认原始字符串的写法与崩溃放量那一份数花括号
    /// （`crash_amplification.rs` 的 `lines_read_past_comments_and_literals`）相同：`r`、`br`、`cr` 后面紧跟若干个 `#` 与一个 `"`。
    /// 扫源码的用例照它找条目、`use` 引进来的名字与花括号的层数：字面量与注释里的 `mod`、`use`、花括号都不算。
    /// 非 ASCII 的字只落在注释、字面量与标识符里，切出来的词因此总在字符边界上。
    fn rust_tokens_without_comments_and_literals(code: &str) -> Vec<&str> {
        rust_tokens_with_their_offsets(code)
            .into_iter()
            .map(|(_offset, token)| token)
            .collect()
    }

    /// [`rust_tokens_without_comments_and_literals`] 的词，各连同它在 `code` 里起头的字节位置。
    fn rust_tokens_with_their_offsets(code: &str) -> Vec<(usize, &str)> {
        let is_identifier_character =
            |character: char| character.is_alphanumeric() || character == '_';
        let mut tokens = Vec::new();
        let mut position = 0;
        while position < code.len() {
            let rest = &code[position..];
            let first_character = rest.chars().next().expect("没到结尾就还有字");
            if first_character.is_whitespace() {
                position += first_character.len_utf8();
            } else if rest.starts_with("//") {
                position += rest.find('\n').unwrap_or(rest.len());
            } else if rest.starts_with("/*") {
                position += length_of_the_block_comment_at_the_start(rest);
            } else if first_character == '"' {
                position += length_of_the_quoted_literal_at_the_start(rest);
            } else if first_character == '\'' {
                match length_of_the_character_literal_at_the_start(rest) {
                    Some(literal_length) => position += literal_length,
                    None => {
                        let lifetime_length = 1 + rest[1..]
                            .find(|character: char| !is_identifier_character(character))
                            .unwrap_or(rest.len() - 1);
                        tokens.push((position, &rest[..lifetime_length]));
                        position += lifetime_length;
                    }
                }
            } else if is_identifier_character(first_character) {
                let word_length = rest
                    .find(|character: char| !is_identifier_character(character))
                    .unwrap_or(rest.len());
                let word = &rest[..word_length];
                let after_the_word = &rest[word_length..];
                if matches!(word, "r" | "br" | "cr")
                    && after_the_word.trim_start_matches('#').starts_with('"')
                {
                    position += word_length + length_of_the_raw_string_at_the_start(after_the_word);
                } else if word == "r" && after_the_word.starts_with('#') {
                    // 原始标识符 `r#名字`：拼成关键字的连着 `r#` 出词（`r#mod` 是名字，不是关键字），别的名字下一圈照标识符收
                    let name = &after_the_word[1..];
                    let name_length = name
                        .find(|character: char| !is_identifier_character(character))
                        .unwrap_or(name.len());
                    if KEYWORDS_A_RAW_IDENTIFIER_CAN_SPELL.contains(&&name[..name_length]) {
                        tokens.push((position, &rest[..word_length + 1 + name_length]));
                        position += word_length + 1 + name_length;
                    } else {
                        position += word_length + 1;
                    }
                } else if matches!(word, "b" | "c") && after_the_word.starts_with('"') {
                    position +=
                        word_length + length_of_the_quoted_literal_at_the_start(after_the_word);
                } else if word == "b" && after_the_word.starts_with('\'') {
                    position += word_length
                        + length_of_the_character_literal_at_the_start(after_the_word)
                            .expect("b 后面紧跟引号的是字节字面量");
                } else {
                    tokens.push((position, word));
                    position += word_length;
                }
            } else if rest.starts_with("::") {
                tokens.push((position, "::"));
                position += 2;
            } else if rest.starts_with("!=") {
                // 不等号整个一个词：拆成 `!`、`=` 时，条目层 `const 名: bool = 甲 != 乙;` 的 `甲 !` 会被认成调宏
                tokens.push((position, "!="));
                position += 2;
            } else {
                tokens.push((position, &rest[..first_character.len_utf8()]));
                position += first_character.len_utf8();
            }
        }
        tokens
    }

