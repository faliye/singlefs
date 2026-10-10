

    /// `code`（注释与字面量已换成空格）读到结尾时还没收尾的圆括号与方括号有几层。
    ///
    /// # Panics
    /// 右圆括号、右方括号多于左的。
    fn parenthesis_and_bracket_depth_at_the_end_of(code: &str) -> usize {
        code.chars().fold(0_usize, |depth, character| {
            if matches!(character, '(' | '[') {
                depth + 1
            } else if matches!(character, ')' | ']') {
                depth.checked_sub(1).expect("右圆括号、右方括号不多于左的")
            } else {
                depth
            }
        })
    }

    /// 数花括号时，读到的这个字在源码的哪一种地方（跨行时带到下一行）。
    #[derive(Clone, Copy)]
    enum PlaceInTheSource {
        Code,
        InsideAString,
        /// 原始字符串（`r"…"`、`r#"…"#`、`br"…"`、`cr"…"`）：收尾是一个 `"` 后面跟这么多个 `#`。
        InsideARawString {
            closing_hashes: usize,
        },
        /// 块注释，可以嵌套：`nesting` 是还没收尾的 `/*` 有几层。
        InsideABlockComment {
            nesting: usize,
        },
    }

    /// 读过的一个字算源码里的哪一类（`lines_read_past_comments_and_literals`）。
    #[derive(Clone, Copy)]
    enum KindOfACharacterInTheSource {
        Code,
        /// 行注释、块注释里的字，连同 `//`、`/*`、`*/`。
        Comment,
        /// 字符串（普通的、原始的）与字符字面量里的字，连同引号与原始字符串的 `r`、`br`、`cr` 和 `#`。
        Literal,
    }

    /// 一行读过注释与字面量之后的样子（`lines_read_past_comments_and_literals`）。
    struct LineReadPastCommentsAndLiterals {
        /// 这一行起头处在第几层花括号里。
        brace_depth_at_the_start: usize,
        /// 注释里的字换成空格（一个字换成与它的 UTF-8 字节数一样多的空格，字节位置不变），字面量照原样。
        without_comments: String,
        /// 注释与字面量里的字都换成空格，字节位置不变：扫源码的用例照它找名字、数 `let`、数花括号，注释与字符串里写的不算。
        without_comments_and_literals: String,
    }

    /// `lines` 每一行起头处在第几层花括号里（第一行起头算第 0 层）：字符串（普通的、原始的）、字符字面量、行注释与块注释（可以嵌套）里的
    /// 花括号不算，字符串、块注释跨行时照跨着数。
    ///
    /// # Panics
    /// 右花括号多于左花括号。
    fn brace_depth_at_the_start_of_each_line(lines: &[&str]) -> Vec<usize> {
        lines_read_past_comments_and_literals(lines)
            .iter()
            .map(|read_line| read_line.brace_depth_at_the_start)
            .collect()
    }

    /// `lines` 逐行读过注释与字面量：每一行起头处的花括号层数，连同去掉注释、去掉注释与字面量的两份（字节位置与原行相同）。
    /// 字符串（普通的、原始的 `r"…"`、`r#"…"#`、`br`、`cr`）、字符字面量、行注释与块注释（可以嵌套）里的花括号不算，
    /// 字符串、块注释跨行时照跨着读。与卡池扫源码那一份分词（`gpu_unit_checks.rs` 的 `rust_tokens_without_comments_and_literals`）
    /// 认原始字符串的写法相同：`r`、`br`、`cr` 后面紧跟若干个 `#` 与一个 `"`。
    ///
    /// # Panics
    /// 右花括号多于左花括号。
    fn lines_read_past_comments_and_literals(
        lines: &[&str],
    ) -> Vec<LineReadPastCommentsAndLiterals> {
        let is_identifier_character =
            |character: char| character.is_alphanumeric() || character == '_';
        let mut depth = 0_usize;
        let mut place = PlaceInTheSource::Code;
        let mut read_lines = Vec::with_capacity(lines.len());
        for line in lines {
            let brace_depth_at_the_start = depth;
            let characters: Vec<char> = line.chars().collect();
            let mut kinds = vec![KindOfACharacterInTheSource::Code; characters.len()];
            let mut index = 0;
            while index < characters.len() {
                let character = characters[index];
                let next_character = characters.get(index + 1).copied();
                let read_from = index;
                let place_before = place;
                match place {
                    PlaceInTheSource::InsideAString => {
                        if character == '\\' {
                            index += 1;
                        } else if character == '"' {
                            place = PlaceInTheSource::Code;
                        }
                    }
                    PlaceInTheSource::InsideARawString { closing_hashes } => {
                        let closing_hashes_follow = characters
                            .get(index + 1..index + 1 + closing_hashes)
                            .is_some_and(|after_the_quote| {
                                after_the_quote.iter().all(|after| *after == '#')
                            });
                        if character == '"' && closing_hashes_follow {
                            index += closing_hashes;
                            place = PlaceInTheSource::Code;
                        }
                    }
                    PlaceInTheSource::InsideABlockComment { nesting } => {
                        if character == '/' && next_character == Some('*') {
                            index += 1;
                            place = PlaceInTheSource::InsideABlockComment {
                                nesting: nesting + 1,
                            };
                        } else if character == '*' && next_character == Some('/') {
                            index += 1;
                            place = match nesting.checked_sub(1) {
                                Some(0) | None => PlaceInTheSource::Code,
                                Some(nesting_left) => PlaceInTheSource::InsideABlockComment {
                                    nesting: nesting_left,
                                },
                            };
                        }
                    }
                    PlaceInTheSource::Code => {
                        if character == '"' {
                            place = PlaceInTheSource::InsideAString;
                        } else if character == '\'' {
                            // 字符字面量（`'x'`、`'\''`、`'\u{7b}'`）整个跳过；生命周期（`'static`）只跳这个引号
                            if next_character == Some('\\') {
                                index += 2;
                                while characters.get(index + 1).is_some_and(|next| *next != '\'') {
                                    index += 1;
                                }
                                index += 1;
                            } else if characters.get(index + 2) == Some(&'\'') {
                                index += 2;
                            }
                        } else if character == '/' && next_character == Some('/') {
                            kinds[index..].fill(KindOfACharacterInTheSource::Comment);
                            break;
                        } else if character == '/' && next_character == Some('*') {
                            index += 1;
                            place = PlaceInTheSource::InsideABlockComment { nesting: 1 };
                        } else if is_identifier_character(character) {
                            // 一整个词跳过；词是 r、br、cr 而后面紧跟若干个 # 与一个 " 的，是原始字符串的开头
                            let word_end = characters[index..]
                                .iter()
                                .position(|after| !is_identifier_character(*after))
                                .map_or(characters.len(), |word_length| index + word_length);
                            let word: String = characters[index..word_end].iter().collect();
                            let opening_hashes = characters[word_end..]
                                .iter()
                                .take_while(|after| **after == '#')
                                .count();
                            if matches!(word.as_str(), "r" | "br" | "cr")
                                && characters.get(word_end + opening_hashes) == Some(&'"')
                            {
                                index = word_end + opening_hashes;
                                place = PlaceInTheSource::InsideARawString {
                                    closing_hashes: opening_hashes,
                                };
                            } else {
                                index = word_end - 1;
                            }
                        } else if character == '\u{7b}' {
                            depth += 1;
                        } else if character == '\u{7d}' {
                            depth = depth.checked_sub(1).expect("右花括号不多于左花括号");
                        }
                    }
                }
                index += 1;
                let kind_read = match place_before {
                    PlaceInTheSource::InsideAString | PlaceInTheSource::InsideARawString { .. } => {
                        KindOfACharacterInTheSource::Literal
                    }
                    PlaceInTheSource::InsideABlockComment { .. } => {
                        KindOfACharacterInTheSource::Comment
                    }
                    PlaceInTheSource::Code => match place {
                        PlaceInTheSource::InsideAString
                        | PlaceInTheSource::InsideARawString { .. } => {
                            KindOfACharacterInTheSource::Literal
                        }
                        PlaceInTheSource::InsideABlockComment { .. } => {
                            KindOfACharacterInTheSource::Comment
                        }
                        // 一次读过不止一个字的引号是整个字符字面量；生命周期只读过那一个引号
                        PlaceInTheSource::Code if character == '\'' && index > read_from + 1 => {
                            KindOfACharacterInTheSource::Literal
                        }
                        PlaceInTheSource::Code => KindOfACharacterInTheSource::Code,
                    },
                };
                kinds[read_from..index.min(characters.len())].fill(kind_read);
            }
            let mut without_comments = String::with_capacity(line.len());
            let mut without_comments_and_literals = String::with_capacity(line.len());
            for (character, kind) in characters.iter().zip(&kinds) {
                let spaces = " ".repeat(character.len_utf8());
                match kind {
                    KindOfACharacterInTheSource::Code => {
                        without_comments.push(*character);
                        without_comments_and_literals.push(*character);
                    }
                    KindOfACharacterInTheSource::Comment => {
                        without_comments.push_str(&spaces);
                        without_comments_and_literals.push_str(&spaces);
                    }
                    KindOfACharacterInTheSource::Literal => {
                        without_comments.push(*character);
                        without_comments_and_literals.push_str(&spaces);
                    }
                }
            }
            read_lines.push(LineReadPastCommentsAndLiterals {
                brace_depth_at_the_start,
                without_comments,
                without_comments_and_literals,
            });
        }
        read_lines
    }

