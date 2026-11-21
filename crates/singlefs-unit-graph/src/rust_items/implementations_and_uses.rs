//! impl 头与 use 两类条目：impl 头的形状与标签、自身类型路径的首段，use 引进的名字。
use super::*;

/// 找 impl 自身类型路径的头一段时跳过的词（与 `scripts/gate/lib_function_units.py` 的 `SELF_TYPE_SKIPPED` 同一份）。
const WORDS_BEFORE_THE_SELF_TYPE_PATH_ROOT: [&str; 8] = [
    "dyn", "mut", "const", "impl", "for", "where", "as", "unsafe",
];

impl<'scan> RustScanner<'scan> {
    pub(super) fn implementation_shape(&self, low: usize, high: usize) -> ImplementationShape {
        let mut index = low;
        // `impl<…>` 自己的泛型参数：深度 1 上紧跟 `<`、`,` 或 `const` 的名字（自身类型里出现其中任一个的 impl 算 blanket）
        let mut generic_parameters: BTreeSet<String> = BTreeSet::new();
        if self.text_of(index) == "<" {
            let mut depth = 0usize;
            while index < high {
                match self.text_of(index) {
                    "<" => depth += 1,
                    ">" if !self.arrow_close(index) => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            index += 1;
                            break;
                        }
                    }
                    text if depth == 1
                        && self.is_identifier(index)
                        && text != "const"
                        && index > low
                        && matches!(self.text_of(index - 1), "<" | "," | "const") =>
                    {
                        generic_parameters.insert(self.name_at(index));
                    }
                    _ => {}
                }
                index += 1;
            }
        }
        let mut depth = 0usize;
        let mut for_index = None;
        let mut where_index = high;
        for position in index..high {
            match self.text_of(position) {
                "<" => depth += 1,
                ">" if !self.arrow_close(position) => depth = depth.saturating_sub(1),
                "for" if depth == 0 && for_index.is_none() && self.text_of(position + 1) != "<" => {
                    for_index = Some(position);
                }
                "where" if depth == 0 => {
                    where_index = position;
                    break;
                }
                _ => {}
            }
        }
        let is_skipped_word =
            |name: &str| matches!(name, "dyn" | "mut" | "const" | "impl" | "for" | "where");
        let names_in = |range_start: usize, range_end: usize| -> Vec<String> {
            let mut names = BTreeSet::new();
            for position in range_start..range_end {
                if self.is_identifier(position) {
                    let name = self.name_at(position);
                    if !is_skipped_word(&name) {
                        names.insert(name);
                    }
                }
            }
            names.into_iter().collect()
        };
        match for_index {
            Some(for_position) => {
                // trait 自己的名字：`for` 之前那一截里尖括号外的最后一个名字（`dyn` 不算）
                let mut trait_name = None;
                let mut trait_depth = 0usize;
                for position in index..for_position {
                    match self.text_of(position) {
                        "<" => trait_depth += 1,
                        ">" if !self.arrow_close(position) => {
                            trait_depth = trait_depth.saturating_sub(1);
                        }
                        _ if trait_depth == 0 && self.is_identifier(position) => {
                            let name = self.name_at(position);
                            if name != "dyn" {
                                trait_name = Some(name);
                            }
                        }
                        _ => {}
                    }
                }
                let self_type_names = names_in(for_position + 1, where_index);
                // 自身类型里任何深度出现这个 impl 自己的一个泛型参数（`for T`、`&T`、`Box<T>`、`(u32, T)`、`[T; N]`、
                // `Option<Box<T>>`）就算 blanket：这类 impl 的自身类型里可能一个能被点名的具体名字都没有（`[T]`），
                // 不放行它的成员就进不了任何闭包、键过窄（「多连不少连」）。全具体的自身类型照旧等被点名。
                let is_blanket_implementation = self_type_names
                    .iter()
                    .any(|name| generic_parameters.contains(name));
                ImplementationShape {
                    self_type_names,
                    trait_name,
                    is_trait_implementation: true,
                    is_blanket_implementation,
                    self_type_path_root: self.self_type_path_root(for_position + 1, where_index),
                }
            }
            None => ImplementationShape {
                self_type_names: names_in(index, where_index),
                trait_name: None,
                is_trait_implementation: false,
                is_blanket_implementation: false,
                self_type_path_root: self.self_type_path_root(index, where_index),
            },
        }
    }

    /// impl 自身类型（`range_start..range_end`）写成路径 `a::B` 时的头一段：跳过 [`WORDS_BEFORE_THE_SELF_TYPE_PATH_ROOT`] 之后的头一个名字，
    /// 后面紧接 `::` 才算；不是路径的交 `None`（`ImplementationShape::self_type_path_root`）。
    fn self_type_path_root(&self, range_start: usize, range_end: usize) -> Option<String> {
        let first_name = (range_start..range_end).find(|position| {
            self.is_identifier(*position)
                && !WORDS_BEFORE_THE_SELF_TYPE_PATH_ROOT.contains(&self.text_of(*position))
        })?;
        (self.text_of(first_name + 1) == ":" && self.text_of(first_name + 2) == ":")
            .then(|| self.name_at(first_name))
    }

    pub(super) fn use_imports(&self, low: usize, high: usize) -> UseImports {
        let mut imports = UseImports::default();
        let mut names = BTreeSet::new();
        // 套着的每一层 `{` 前面那一段（`a::{` 的 `a`；不是名字、或是 self / super / crate 的记 None）：花括号里的 `self` 引进的就是它
        let mut segments_before_the_braces: Vec<Option<String>> = Vec::new();
        let original_name_at = |index: usize, segments: &[Option<String>]| -> Option<String> {
            let name = self.name_at(index);
            if name == "self" && self.text_of(index + 1) != ":" {
                if let Some(segment) = segments.last() {
                    return segment.clone();
                }
            }
            Some(name)
        };
        for index in low..high {
            let text = self.text_of(index);
            if text == "*" {
                imports.is_glob = true;
                continue;
            }
            if text == "{" {
                let segment = if index >= low + 3
                    && self.text_of(index - 1) == ":"
                    && self.text_of(index - 2) == ":"
                    && self.is_identifier(index - 3)
                {
                    Some(self.name_at(index - 3))
                        .filter(|name| !matches!(name.as_str(), "self" | "super" | "crate"))
                } else {
                    None
                };
                segments_before_the_braces.push(segment);
                continue;
            }
            if text == "}" {
                segments_before_the_braces.pop();
                continue;
            }
            if !self.is_identifier(index) {
                continue;
            }
            let name = self.name_at(index);
            if name == "as" {
                if self.is_identifier(index + 1) && index > low && self.is_identifier(index - 1) {
                    if let Some(original) = original_name_at(index - 1, &segments_before_the_braces)
                    {
                        imports.aliases.push((self.name_at(index + 1), original));
                    }
                }
                continue;
            }
            let is_path_segment = self.text_of(index + 1) == ":" && self.text_of(index + 2) == ":";
            if is_path_segment {
                continue;
            }
            // `a::b as c`：原名 `b` 不进当前作用域，进的只有别名 `c`（在 `as` 那一格记进 aliases 与 names）
            if self.text_of(index + 1) == "as" {
                continue;
            }
            if !matches!(name.as_str(), "self" | "super" | "crate") {
                names.insert(name);
            } else if name == "self" {
                if let Some(original) = original_name_at(index, &segments_before_the_braces)
                    .filter(|original| original != "self")
                {
                    names.insert(original);
                }
            }
        }
        imports.names = names.into_iter().collect();
        imports
    }
}

/// impl 头的标签（限定名里用）：`impl` 之后的词法单元，去掉紧跟 `impl` 的泛型参数表与 `where` 子句，逐个用一个空格连起来，
/// 相邻两个 `:` 连成 `::`、两边不留空格；例 `<impl std::ops::Add for Money>`、`<impl Wrapper < Inner >>`。
/// 与 `scripts/gate/lib_function_units.py` 的 `impl_head` 同一种写法（两份实现逐单元对拍限定名）。
#[must_use]
pub fn implementation_label(texts_after_the_keyword: &[&str]) -> String {
    let mut texts = texts_after_the_keyword;
    if texts.first() == Some(&"<") {
        let mut depth = 0usize;
        for (position, text) in texts.iter().enumerate() {
            let is_arrow_head = position > 0 && texts[position - 1] == "-";
            match *text {
                "<" => depth += 1,
                ">" if !is_arrow_head => depth = depth.saturating_sub(1),
                _ => {}
            }
            if depth == 0 {
                texts = &texts[position + 1..];
                break;
            }
        }
    }
    if let Some(where_position) = texts.iter().position(|text| *text == "where") {
        texts = &texts[..where_position];
    }
    let mut rendered = String::from("<impl ");
    for (position, text) in texts.iter().enumerate() {
        let is_path_separator_colon = *text == ":"
            && ((position > 0 && texts[position - 1] == ":")
                || texts.get(position + 1) == Some(&":"));
        let follows_path_separator =
            position >= 2 && texts[position - 1] == ":" && texts[position - 2] == ":";
        if position > 0 && !is_path_separator_colon && !follows_path_separator {
            rendered.push(' ');
        }
        rendered.push_str(text);
    }
    rendered.push('>');
    rendered
}

#[cfg(test)]
mod tests;
