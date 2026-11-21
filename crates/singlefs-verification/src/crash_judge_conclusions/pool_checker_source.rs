//! 读池级 checker（`crates/singlefs-checker/src/`）的源码，给 `tests` 里扫 I-1.13 能报的理由那条测试用：列 `src/` 下（递归）不是测试代码的每一份、
//! 截掉行内的测试模块、把 `//` 注释换成空格，把走读那一族（`walk.rs` 与 `walk/` 下）接成一段并能把位置折回「文件 第几行」。

/// 池级 checker 的源码目录 `crates/singlefs-checker/src/`：清单目录接一段字面路径，判法单元的键把这个目录记成数据目录（目录下每份文件的字节都进键）。
fn pool_checker_source_directory() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../singlefs-checker/src")
}

/// 池级 checker 的一份源码（`crates/singlefs-checker/src/` 下，`relative_path` 相对这个目录），截到它行内的测试模块之前
/// （测试里造的判定不是池级 checker 能报的）。
pub(super) fn pool_checker_source_before_its_tests(relative_path: &str) -> String {
    let path = pool_checker_source_directory().join(relative_path);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("读得到池级 checker 的源码 {}：{error}", path.display()));
    match source.find("\n#[cfg(test)]\nmod tests {") {
        Some(end) => source[..end].to_string(),
        None => source,
    }
}

/// 一份源码里 `#[cfg(test)]` 声明、放在别的文件里的子模块（`#[cfg(test)]` 的下一行是 `mod <名>;`），写成相对 `src/` 的模块路径：
/// `lib.rs` 里的 `mod <名>;` 是 `<名>`，`walk.rs` 里的是 `walk/<名>`，`walk/<子模块>.rs` 里的是 `walk/<子模块>/<名>`。`<模块路径>.rs` 与 `<模块路径>/` 下整份是测试代码。
fn test_module_paths_declared_in(relative_path: &str, source: &str) -> Vec<String> {
    let directory_of_the_submodules = match relative_path {
        "lib.rs" => String::new(),
        _ => relative_path
            .strip_suffix("/mod.rs")
            .or_else(|| relative_path.strip_suffix(".rs"))
            .unwrap_or_else(|| panic!("池级 checker 的源码文件 {relative_path} 以 .rs 收尾"))
            .to_string(),
    };
    source
        .match_indices("#[cfg(test)]\n")
        .filter_map(|(attribute_start, attribute)| {
            let declaration = source[attribute_start + attribute.len()..]
                .lines()
                .next()?
                .trim();
            let module_name = declaration.strip_prefix("mod ")?.strip_suffix(';')?;
            Some(if directory_of_the_submodules.is_empty() {
                module_name.to_string()
            } else {
                format!("{directory_of_the_submodules}/{module_name}")
            })
        })
        .collect()
}

/// 池级 checker `src/` 下（递归）每份不是测试代码的 `.rs`：相对 `src/` 的路径（以 `/` 分段）与截到行内测试模块之前的源码，按路径排序。
/// `#[cfg(test)]` 声明的子模块（[`test_module_paths_declared_in`]）的文件与它底下的目录整份是测试代码，不在里面。
pub(super) fn pool_checker_sources_before_their_tests() -> Vec<(String, String)> {
    let source_directory = pool_checker_source_directory();
    let mut relative_paths = Vec::new();
    let mut relative_directories_to_list = vec![String::new()];
    while let Some(relative_directory) = relative_directories_to_list.pop() {
        let directory = source_directory.join(&relative_directory);
        let entries = std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("读得到 {}：{error}", directory.display()));
        for entry in entries {
            let entry = entry.expect("读得到目录项");
            let entry_name = entry.file_name().into_string().expect("文件名是 UTF-8");
            let relative_path = if relative_directory.is_empty() {
                entry_name
            } else {
                format!("{relative_directory}/{entry_name}")
            };
            if entry.file_type().expect("读得到目录项的种类").is_dir() {
                relative_directories_to_list.push(relative_path);
            } else if relative_path.ends_with(".rs") {
                relative_paths.push(relative_path);
            }
        }
    }
    relative_paths.sort();
    let sources: Vec<(String, String)> = relative_paths
        .into_iter()
        .map(|relative_path| {
            let source = pool_checker_source_before_its_tests(&relative_path);
            (relative_path, source)
        })
        .collect();
    let test_module_paths: Vec<String> = sources
        .iter()
        .flat_map(|(relative_path, source)| test_module_paths_declared_in(relative_path, source))
        .collect();
    sources
        .into_iter()
        .filter(|(relative_path, _)| {
            !test_module_paths.iter().any(|test_module_path| {
                relative_path
                    .strip_prefix(test_module_path.as_str())
                    .is_some_and(|rest| rest == ".rs" || rest.starts_with('/'))
            })
        })
        .collect()
}

/// 走读那一族：`walk.rs` 与它的子模块（`walk/` 下）。
pub(super) fn is_in_the_walk_family(relative_path: &str) -> bool {
    relative_path == "walk.rs" || relative_path.starts_with("walk/")
}

/// 判定累加器那一族：`image.rs` 与它的子模块（`image/` 下）。
pub(super) fn is_in_the_image_family(relative_path: &str) -> bool {
    relative_path == "image.rs" || relative_path.starts_with("image/")
}

/// 走读那一族的产品代码（[`pool_checker_sources_before_their_tests`] 里 [`is_in_the_walk_family`] 的那几份）按路径次序接成的一段，
/// 每份先把 `//` 注释换成空格、不以换行收尾的补一个换行；`file_starts` 是每份在这一段里的字节起点，报位置时折回「文件 第几行」。
pub(super) struct WalkFamilySource {
    pub(super) text: String,
    file_starts: Vec<(String, usize)>,
}

impl WalkFamilySource {
    pub(super) fn read_before_the_tests() -> Self {
        let mut text = String::new();
        let mut file_starts = Vec::new();
        for (relative_path, source) in pool_checker_sources_before_their_tests() {
            if !is_in_the_walk_family(&relative_path) {
                continue;
            }
            file_starts.push((relative_path, text.len()));
            text.push_str(&source_with_line_comments_blanked(&source));
            if !text.ends_with('\n') {
                text.push('\n');
            }
        }
        Self { text, file_starts }
    }

    /// `position`（这一段里的字节位置）在哪份文件的第几行，写成「walk/scan_rebuild_comparison.rs 第 561 行」。
    pub(super) fn place_of(&self, position: usize) -> String {
        let (relative_path, file_start) = self
            .file_starts
            .iter()
            .rev()
            .find(|(_, file_start)| *file_start <= position)
            .expect("位置落在接起来的某一份文件里");
        let line = self.text[*file_start..position].matches('\n').count() + 1;
        format!("{relative_path} 第 {line} 行")
    }
}

/// `//` 注释换成同样字节数的空格（字节位置不变，注释里提到的名字不算源码里的一处）；串字面量里的字不动。
///
/// # Panics
/// 源码里有这个简单读法认不全的写法（块注释、原始串、`'"'` 字符字面量）。
pub(super) fn source_with_line_comments_blanked(source: &str) -> String {
    for unsupported in ["/*", "r#\"", "'\"'"] {
        assert!(
            !source.contains(unsupported),
            "池级 checker 源码里出现了 {unsupported:?}：这份扫描只认行注释与普通串字面量，要先补读法"
        );
    }
    let mut blanked = String::with_capacity(source.len());
    let mut inside_a_string_literal = false;
    let mut inside_a_line_comment = false;
    let mut the_previous_character_escapes = false;
    let mut characters = source.chars().peekable();
    while let Some(character) = characters.next() {
        if inside_a_line_comment {
            if character == '\n' {
                inside_a_line_comment = false;
                blanked.push('\n');
            } else {
                blanked.push_str(&" ".repeat(character.len_utf8()));
            }
            continue;
        }
        if inside_a_string_literal {
            blanked.push(character);
            if the_previous_character_escapes {
                the_previous_character_escapes = false;
            } else if character == '\\' {
                the_previous_character_escapes = true;
            } else if character == '"' {
                inside_a_string_literal = false;
            }
            continue;
        }
        if character == '/' && characters.peek() == Some(&'/') {
            inside_a_line_comment = true;
            blanked.push(' ');
            continue;
        }
        if character == '"' {
            inside_a_string_literal = true;
        }
        blanked.push(character);
    }
    blanked
}
