//! 单元里的名字：裸名、路径、方法名、字段名与格式串里捕获的名字。
use super::*;

/// 一个串字面量里 `{名字}`、`{名字:…}` 形式的格式捕获：`format!("{DATA_POINTER_BYTES}")` 这样用到的常量也算点了它的名。
fn add_format_captures<'source>(literal: &'source str, names: &mut HashSet<&'source str>) {
    if !(literal.starts_with('"') || literal.starts_with("r\"") || literal.starts_with("r#"))
        || !literal.contains('{')
    {
        return;
    }
    let bytes = literal.as_bytes();
    let mut position = 0usize;
    while position < bytes.len() {
        if bytes[position] != b'{' {
            position += 1;
            continue;
        }
        if bytes.get(position + 1) == Some(&b'{') {
            position += 2;
            continue;
        }
        let start = position + 1;
        let mut end = start;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            end += 1;
        }
        let is_name = end > start && !bytes[start].is_ascii_digit();
        if is_name && matches!(bytes.get(end), Some(b'}' | b':')) {
            names.insert(&literal[start..end]);
        }
        position = end.max(position + 1);
    }
}

fn sorted_strings(names: HashSet<&str>) -> Vec<String> {
    let mut sorted: Vec<String> = names.into_iter().map(str::to_string).collect();
    sorted.sort();
    sorted
}

/// 一段词法单元里的名字，按 3.3 的规则分好类。`signature_end` 是函数体那个 `{` 在这一段里的位置（函数单元才有）。
#[must_use]
pub fn references_of_tokens(
    source: &str,
    tokens: &[Token],
    signature_end: Option<usize>,
) -> UnitReferences {
    let text_at = |position: usize| {
        tokens
            .get(position)
            .map_or("", |token| token_text(source, token))
    };
    let mut plain_names: HashSet<&str> = HashSet::new();
    let mut method_names: HashSet<&str> = HashSet::new();
    let mut qualified_paths: BTreeSet<(String, String)> = BTreeSet::new();
    let mut dyn_trait_names: HashSet<&str> = HashSet::new();
    for (position, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::Literal => {
                add_format_captures(token_text(source, token), &mut plain_names);
                continue;
            }
            TokenKind::Lifetime | TokenKind::Punctuation => continue,
            TokenKind::Identifier => {}
        }
        let name = identifier_name(token_text(source, token));
        let previous = if position > 0 {
            text_at(position - 1)
        } else {
            ""
        };
        let is_after_single_dot =
            previous == "." && !(position > 1 && text_at(position - 2) == ".");
        let next = text_at(position + 1);
        if is_after_single_dot {
            if next == "(" || (next == ":" && text_at(position + 2) == ":") {
                method_names.insert(name);
            }
            continue;
        }
        if previous == ":"
            && position >= 3
            && text_at(position - 2) == ":"
            && tokens[position - 3].kind == TokenKind::Identifier
        {
            qualified_paths.insert((
                identifier_name(text_at(position - 3)).to_string(),
                name.to_string(),
            ));
        }
        if name == "dyn" {
            let mut path_position = position + 1;
            let mut last_segment = None;
            while tokens
                .get(path_position)
                .is_some_and(|candidate| candidate.kind == TokenKind::Identifier)
            {
                last_segment = Some(identifier_name(text_at(path_position)));
                if text_at(path_position + 1) == ":" && text_at(path_position + 2) == ":" {
                    path_position += 3;
                } else {
                    break;
                }
            }
            if let Some(trait_name) = last_segment {
                dyn_trait_names.insert(trait_name);
            }
        }
        if !is_keyword(name) {
            plain_names.insert(name);
        }
    }
    let mut signature_trait_names: HashSet<&str> = HashSet::new();
    if let Some(end) = signature_end {
        for position in 0..end.min(tokens.len()) {
            if matches!(text_at(position), "impl" | "dyn" | ":" | "+")
                && tokens
                    .get(position + 1)
                    .is_some_and(|candidate| candidate.kind == TokenKind::Identifier)
            {
                let candidate = identifier_name(text_at(position + 1));
                if !is_keyword(candidate) {
                    signature_trait_names.insert(candidate);
                }
            }
        }
    }
    UnitReferences {
        plain_names: sorted_strings(plain_names),
        method_names: sorted_strings(method_names),
        qualified_paths: qualified_paths.into_iter().collect(),
        dyn_trait_names: sorted_strings(dyn_trait_names),
        signature_trait_names: sorted_strings(signature_trait_names),
    }
}

#[cfg(test)]
mod tests;
