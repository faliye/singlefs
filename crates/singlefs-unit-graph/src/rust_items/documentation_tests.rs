//! 文档测试单元：文档注释里的 Rust 代码块另成单元，挂在所属条目或模块下。
use super::*;

/// 文档注释里的一个 Rust 代码块。
struct DocumentationCodeBlock {
    code: String,
    opening_line: u32,
    closing_line: u32,
}

const RUST_CODE_BLOCK_WORDS: [&str; 9] = [
    "rust",
    "ignore",
    "should_panic",
    "no_run",
    "compile_fail",
    "test_harness",
    "allow_fail",
    "standalone_crate",
    "custom",
];

/// 代码块的信息串是不是 rustdoc 当 Rust 文档测试编的那一种（空的也是）。
fn is_rust_code_block_information(information: &str) -> bool {
    information
        .split([',', ' ', '\t'])
        .filter(|word| !word.is_empty())
        .all(|word| {
            RUST_CODE_BLOCK_WORDS.contains(&word)
                || word.starts_with("edition")
                || word.starts_with("ignore-")
                || (word.len() > 1
                    && word
                        .strip_prefix('E')
                        .is_some_and(|digits| digits.bytes().all(|byte| byte.is_ascii_digit())))
        })
}

/// 行首围栏：三个以上的 `` ` `` 或 `~`，交（围栏字符，个数）。
fn fence_of(trimmed_line: &str) -> Option<(u8, usize)> {
    let first = *trimmed_line.as_bytes().first()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let count = trimmed_line
        .bytes()
        .take_while(|byte| *byte == first)
        .count();
    (count >= 3).then_some((first, count))
}

/// rustdoc 的隐藏行：`# 代码` 照编、不显示；`##` 开头的是一个字面的 `#`。
fn without_hidden_line_marker(line: &str) -> &str {
    let trimmed = line.trim_start();
    if trimmed == "#" {
        return "";
    }
    if let Some(rest) = trimmed.strip_prefix("# ") {
        return rest;
    }
    if trimmed.starts_with("##") {
        return &trimmed[1..];
    }
    line
}

struct OpenCodeBlock {
    fence_character: u8,
    fence_length: usize,
    opening_line: u32,
    lines: Vec<String>,
    is_rust: bool,
}

fn rust_code_blocks(comments: &[&DocumentationComment]) -> Vec<DocumentationCodeBlock> {
    let mut lines: Vec<(u32, &str)> = Vec::new();
    for comment in comments {
        for (offset, body_line) in comment.body_lines.iter().enumerate() {
            let line_number =
                comment.first_line + u32::try_from(offset).expect("一段文档注释的行数放得进 u32");
            lines.push((line_number, body_line.as_str()));
        }
    }
    let mut blocks = Vec::new();
    let mut open_block: Option<OpenCodeBlock> = None;
    for (line_number, raw_line) in lines {
        let trimmed = raw_line.trim_start();
        match open_block.as_mut() {
            None => {
                if let Some((fence_character, fence_length)) = fence_of(trimmed) {
                    let information = trimmed[fence_length..].trim();
                    open_block = Some(OpenCodeBlock {
                        fence_character,
                        fence_length,
                        opening_line: line_number,
                        lines: Vec::new(),
                        is_rust: is_rust_code_block_information(information),
                    });
                }
            }
            Some(block) => {
                let is_closing = fence_of(trimmed).is_some_and(|(character, length)| {
                    character == block.fence_character
                        && length >= block.fence_length
                        && trimmed[length..].trim().is_empty()
                });
                if is_closing {
                    let finished = open_block.take().expect("这一臂里 open_block 是 Some");
                    if finished.is_rust {
                        blocks.push(DocumentationCodeBlock {
                            code: finished.lines.join("\n"),
                            opening_line: finished.opening_line,
                            closing_line: line_number,
                        });
                    }
                } else {
                    block
                        .lines
                        .push(without_hidden_line_marker(raw_line).to_string());
                }
            }
        }
    }
    if let Some(unfinished) = open_block {
        if unfinished.is_rust {
            let closing_line = unfinished.opening_line
                + u32::try_from(unfinished.lines.len()).expect("代码块行数放得进 u32");
            blocks.push(DocumentationCodeBlock {
                code: unfinished.lines.join("\n"),
                opening_line: unfinished.opening_line,
                closing_line,
            });
        }
    }
    blocks
}

impl<'scan> RustScanner<'scan> {
    pub(super) fn documentation_comments_between(
        &self,
        low_token: usize,
        high_token: usize,
        placement: DocumentationPlacement,
    ) -> Vec<&'scan DocumentationComment> {
        let start = self
            .documentation_comments
            .partition_point(|comment| comment.next_token_index < low_token);
        let end = self
            .documentation_comments
            .partition_point(|comment| comment.next_token_index <= high_token);
        self.documentation_comments[start..end.max(start)]
            .iter()
            .filter(|comment| comment.placement == placement)
            .collect()
    }

    pub(super) fn push_documentation_tests(
        &mut self,
        comments: &[&DocumentationComment],
        attachment: &DocumentationTestAttachment,
        module_path: &[String],
    ) {
        if !self.switches.is_intact(BrokenRule::DocumentationTestUnits) {
            return;
        }
        for block in rust_code_blocks(comments) {
            let lexed = lex_source(
                &block.code,
                DocumentationCommentTreatment::KeptOutOfTheTokens,
            );
            let fingerprint = fingerprint_of_tokens(&block.code, &lexed.tokens);
            let references = references_of_tokens(&block.code, &lexed.tokens, None);
            let condition_predicates_in_the_tokens =
                condition_predicates_written_in_tokens(&block.code, &lexed.tokens);
            // 文档测试的条件链取它所挂条目的那一串（挂模块的没有）：键里的条件原子照它取，与 lib_function_units.py 的 add_doctest 同
            let conditions = match attachment {
                DocumentationTestAttachment::Item(owner) => self.units[owner.0].conditions.clone(),
                DocumentationTestAttachment::Module(_) => Vec::new(),
            };
            self.units.push(Unit {
                kind: UnitKind::DocumentationTest,
                name: format!("<doctest line {}>", block.opening_line),
                qualified_name: String::new(),
                file: self.file,
                local_module_path: module_path.to_vec(),
                container_label: None,
                start_line: block.opening_line,
                end_line: block.closing_line,
                token_range: None,
                container: None,
                module_head: None,
                conditions,
                condition_predicates_in_the_tokens,
                is_public: false,
                fingerprint,
                references,
                declared_names: Vec::new(),
                imports: None,
                implementation: None,
                module_declaration: None,
                reads: Vec::new(),
                refusals: Vec::new(),
                documentation_test_of: Some(attachment.clone()),
                has_test_attribute: false,
                is_inside_test_only_code: false,
            });
        }
    }
}

#[cfg(test)]
mod tests;
