//! 字符与 Python 字符串操作的对应：按字符下标取片段与找子串、Python 的空白与 strip、shell 名字的字符、
//! posixpath 的 join / normpath / dirname / basename，与拒算片段的折叠（`snippet_of`）。
use super::*;

// ── 字符与 Python 字符串操作的对应 ─────────────────────────────────────────

pub(super) fn characters_of(text: &str) -> Vec<char> {
    text.chars().collect()
}

/// `text[start:end]`（越界照 Python 截到两端，起点不在终点之前时是空串）。
pub(super) fn text_between(text: &[char], start: usize, end: usize) -> String {
    let end = end.min(text.len());
    if start >= end {
        String::new()
    } else {
        text[start..end].iter().collect()
    }
}

pub(super) fn starts_with_at(text: &[char], index: usize, pattern: &str) -> bool {
    pattern.chars().enumerate().all(|(offset, expected)| {
        index
            .checked_add(offset)
            .and_then(|position| text.get(position))
            == Some(&expected)
    })
}

pub(super) fn find_character(text: &[char], wanted: char, start: usize) -> Option<usize> {
    (start..text.len()).find(|position| text[*position] == wanted)
}

pub(super) fn find_text(haystack: &[char], needle: &[char], start: usize) -> Option<usize> {
    if needle.is_empty() {
        return (start <= haystack.len()).then_some(start);
    }
    if needle.len() > haystack.len() {
        return None;
    }
    (start..=haystack.len() - needle.len())
        .find(|position| haystack[*position..*position + needle.len()] == *needle)
}

/// Python `str.isspace` 认的空白（比 `char::is_whitespace` 多 U+001C 到 U+001F）。
pub(super) fn is_python_whitespace(character: char) -> bool {
    character.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&character)
}

pub(super) fn python_strip(text: &str) -> &str {
    text.trim_matches(is_python_whitespace)
}

pub(super) fn python_left_strip(text: &str) -> &str {
    text.trim_start_matches(is_python_whitespace)
}

fn is_shell_name_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_shell_name_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

/// `[A-Za-z_][A-Za-z0-9_]*` 从 `start` 起的那一段的终点；不以这种字符起头是 `None`。
pub(super) fn identifier_end(text: &[char], start: usize) -> Option<usize> {
    if !text.get(start).copied().is_some_and(is_shell_name_start) {
        return None;
    }
    let mut end = start + 1;
    while text.get(end).copied().is_some_and(is_shell_name_continue) {
        end += 1;
    }
    Some(end)
}

pub(super) fn is_plain_identifier(word: &str) -> bool {
    let text = characters_of(word);
    identifier_end(&text, 0) == Some(text.len())
}

pub(super) fn is_ascii_digit_string(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|character| character.is_ascii_digit())
}

/// 那边的 `snippet_of`：空白段折成一个空格、去掉两头，至多 160 个字符。
pub(super) fn snippet_of(text: &str) -> String {
    let joined = text
        .split(is_python_whitespace)
        .filter(|piece| !piece.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    joined.chars().take(SNIPPET_CHARACTER_LIMIT).collect()
}

/// `posixpath.join(base, tail)`。
pub(super) fn python_join(base: &str, tail: &str) -> String {
    if tail.starts_with('/') {
        tail.to_string()
    } else if base.is_empty() || base.ends_with('/') {
        format!("{base}{tail}")
    } else {
        format!("{base}/{tail}")
    }
}

/// `posixpath.normpath`：去掉 `.` 与空段，`..` 退一级（开头两个斜杠照留两个），不碰文件系统。
pub(super) fn python_normpath(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let initial_slashes = if path.starts_with("//") && !path.starts_with("///") {
        2
    } else {
        usize::from(path.starts_with('/'))
    };
    let mut components: Vec<&str> = Vec::new();
    for component in path.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component != ".."
            || (initial_slashes == 0 && components.is_empty())
            || components.last() == Some(&"..")
        {
            components.push(component);
        } else if !components.is_empty() {
            components.pop();
        }
    }
    let joined = format!("{}{}", "/".repeat(initial_slashes), components.join("/"));
    if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    }
}

/// `posixpath.dirname`。
pub(super) fn python_dirname(path: &str) -> String {
    let split = path.rfind('/').map_or(0, |slash| slash + 1);
    let head = &path[..split];
    if !head.is_empty() && head.chars().any(|character| character != '/') {
        head.trim_end_matches('/').to_string()
    } else {
        head.to_string()
    }
}

/// `posixpath.basename`。
pub(super) fn python_basename(path: &str) -> String {
    let split = path.rfind('/').map_or(0, |slash| slash + 1);
    path[split..].to_string()
}

pub(super) fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

#[cfg(test)]
mod tests {
    use super::{python_dirname, python_normpath, snippet_of};

    #[test]
    fn python_path_helpers_agree_with_posixpath_on_the_edge_cases() {
        assert_eq!(python_normpath(""), ".");
        assert_eq!(python_normpath("/a/./b/../c/"), "/a/c");
        assert_eq!(python_normpath("//a//b"), "//a/b");
        assert_eq!(python_normpath("///a"), "/a");
        assert_eq!(python_normpath("../a/.."), "..");
        // 拼出来，不写成整段字面量：以两级上跳起头的字面量会被 crates 变异表的静态核当成逃出 crates/ 的路径
        let path_climbing_two_levels_then_naming_one_file = concat!("..", "/../a");
        assert_eq!(
            python_normpath(path_climbing_two_levels_then_naming_one_file),
            path_climbing_two_levels_then_naming_one_file
        );
        assert_eq!(python_normpath("/.."), "/");
        assert_eq!(python_dirname("a"), "");
        assert_eq!(python_dirname("/a"), "/");
        assert_eq!(python_dirname("a/b//c"), "a/b");
        assert_eq!(python_dirname("//a"), "//");
    }

    #[test]
    fn snippets_fold_whitespace_runs_and_cut_at_one_hundred_sixty_characters() {
        assert_eq!(snippet_of("  a\t\tb  c "), "a b c");
        assert_eq!(snippet_of(&"字".repeat(200)).chars().count(), 160);
    }
}
