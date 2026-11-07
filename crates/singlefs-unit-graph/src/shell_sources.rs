//! Rust 闭包读到的 shell 脚本：它在自己那个 shell 里（传递地）`source` / `.` 进来的仓内文件。
//!
//! 与 `scripts/gate/lib_function_units.py` 的 `shell_sourced_files`（那份文件头「Rust 闭包读到的 shell 脚本」）是两条不共用代码的路子，
//! 照同一份规则各写各的；函数名与那边一一对应（`ShellIndex.links` 只取 `source` 那一种写法、`fold_word`、`fold_parameter`、`fold_substitution`、
//! `assignments_of`、`target_of`、`word_line`……），两边收出的文件与拒算由 `tests/` 里的两份对拍表钉住。
//!
//! 切法：每份文件先交 bash 规范化（`bash -n` 过了、标准错误为空，才包进一个函数、`declare -f` 打出来；过不了的整份拒算 `bash-syntax`），
//! 在规范文本上切出函数单元（`function 名 () ` 一行接 `{ `，到同一缩进的 `}`；套着的函数各是一个单元、连着外层）与脚本体（去掉顶层函数的其余各行）。
//! 每个单元都看（读到这份脚本的是整份字节，不按函数调没调到挑），被 source 进来的那几份接着看，到不再多出文件为止；拒算取最后一趟的。
//!
//! 认的写法（与那边同一个集合）：
//! - 命令位置上（去掉 `if`、`then`、`do`、`{`、`!` 这类关键字与前缀赋值之后）的 `source <词>` / `. <词>`；注释、here-doc 正文、`[[ … ]]`、
//!   `(( … ))` 里的不算；命令替换 `$( … )` 与反引号里的照命令认。
//! - 路径词的折法：引号与反斜杠照 bash 去掉；`$名字`、`${名字}` 取它的赋值（这个函数自己赋的、套着它的外层函数赋的、这份文件里各单元赋的
//!   （别的函数里 `local` 的不算）、它（传递地）source 进来的那几份赋的、source 了它的那几份赋的，依次取头一处有的；`for 名字 in 词…` 的词也算赋值），
//!   几处赋值各折一遍取并；`${BASH_SOURCE[0]}`、`${BASH_SOURCE}`、`$0` 是这份文件的绝对路径，`$CLAUDE_PROJECT_DIR` 是根；
//!   `${名字:-词}`、`:=`、`-`、`=` 并上缺省；`${名字:?…}`、`${名字?…}` 照名字；`${名字%/*}` 取目录；`$(dirname|basename|realpath|readlink 词)`；
//!   `$(git … rev-parse … --show-toplevel)` 是根、`--git-common-dir` 是根下的 `.git`；`$(cd 词 && pwd)`；`$(cat <<'词' … 词)` 是正文
//!   （不带引号的词照 bash 展开正文）；`$(函数名)`：看得见的同名函数整个函数体只有一条 `printf '%s\n' <字面的行…>` 或 `cat <<词 … 词` 的，取它打的行；
//!   一个词折出超过 64 个取值、或用到别的写法（位置参数、数组下标、`~`、`$(( ))`……）就折不出。
//! - 折出的路径：绝对的要在根里；相对的先在前面 `cd <词>` 过去的目录（折得出的每一个）底下找，再按根、再按这份文件所在的目录找。
//! - 折不出时：这个单元（连套着它的外层函数）自己用 `printf` / `echo` / `cat <<词` 写出来的脚本（正文里 `source` 的字面路径接着认，落在写它的那一行），
//!   `cp <源> <它>` 拷出来的（认源）；`$目录/<至少两段的字面尾巴>`、只取这种写法的变量按根里同一相对路径那一份认。
//! - 拒算：折不出 `shell-path-unfolded`，落在根外 `shell-path-outside-root`，指到不在的文件 `shell-path-missing`，bash 规范化不出来 `bash-syntax`；
//!   落在哪一行照 `word_line`（先找这个词与 `source` / `.` 同在的一行，再退回头一处提到它的行）。
//! - 不走的：`bash -c '… source …'` 这类子进程里的 source，与脚本起的 bash / python 子进程。
//!
//! 与那边已知的出入（只动「哪几份文件的赋值看得见」，不动认的写法）：那边算 source 边（谁 source 了谁，决定变量从哪几份文件里找）时，
//! 连经 `"${数组[@]}"` 摊开的命令、python 内联脚本按 `sys.argv` 起的命令、`bash <写出来的脚本>` 里的 `source` 一起算，这一份只算命令位置上的 `source` / `.`
//! 与 `source <写出来的脚本>` 里的；文件内容不是 UTF-8 时片段里的坏字节这一份换成 U+FFFD。
//!
//! bash 规范化跑不下去（bash 不在、退非 0、超时、临时目录建不了、脚本读不了）是运行条件不满足，不是拒算：两边都整趟报错
//! （那边抛 `FunctionUnitsError`，这一份交 [`ShellNormalizationFailure`]，闭包交 `EntryError::ShellNormalizationFailed`）。

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::units::RefusalForm;

/// 一个词折出的取值超过这么多个就当折不出（那边的 `FOLD_LIMIT`）。
const FOLD_VALUE_LIMIT: usize = 64;
/// 拒算片段至多这么多个字符（那边的 `SNIPPET_CHARACTERS`）。
const SNIPPET_CHARACTER_LIMIT: usize = 160;
/// 命令替换里再套命令替换，至多看这么多层（那边 `scan_substitutions` 的 `depth > 8`）。
const SUBSTITUTION_DEPTH_LIMIT: usize = 8;
/// `tree_copy_targets` 顺变量往下找，至多这么多层（那边的 `depth > 4`）。
const TREE_COPY_DEPTH_LIMIT: usize = 4;
/// 算 source 边至多来回算这么多趟（那边 `compute_groups` 的 `range(3)`）。
const SOURCE_EDGE_ROUNDS: usize = 3;
/// bash 规范化一批脚本的限时（那边的 `SHELL_TIMEOUT_SECONDS`）。
const NORMALIZATION_TIME_LIMIT: Duration = Duration::from_secs(600);
/// 等 bash 退出时两次查看之间睡多久。
const NORMALIZATION_POLL_INTERVAL: Duration = Duration::from_millis(2);
/// 规范化起的 bash（按子进程的 `PATH` 找，与那边 `subprocess.run(["bash", …])` 同）。
const DEFAULT_BASH_PROGRAM: &str = "bash";
/// 整份脚本包进去的那个函数名（那边的 `SHELL_WRAPPER`）。
const WRAPPER_FUNCTION_NAME: &str = "__singlefs_function_units_body";
/// 规范化脚本，与那边的 `SHELL_NORMALIZE_SCRIPT` 逐字相同：`bash -n` 过了而且标准错误为空，才包进函数、`declare -f` 打出来。
const NORMALIZE_SCRIPT: &str = r#"
for source_path in "$@"; do
  if bash -n "$source_path" 2>"$source_path.check" && [[ ! -s "$source_path.check" ]]; then
    source_text="$(<"$source_path")"
    if eval "__singlefs_function_units_body() {
$source_text
}" 2>/dev/null; then
      declare -f __singlefs_function_units_body >"$source_path.normal"
    fi
    unset -f __singlefs_function_units_body
  fi
done
"#;
const SHELL_KEYWORDS: [&str; 15] = [
    "if", "then", "else", "elif", "fi", "do", "done", "while", "until", "!", "{", "}", "time",
    "esac", "coproc",
];
const SHELL_DECLARATIONS: [&str; 5] = ["local", "declare", "export", "readonly", "typeset"];
const GENERATOR_HEADS: [&str; 2] = ["printf", "echo"];
const COPYING_HEADS: [&str; 8] = ["cp", "mv", "install", "ln", "rsync", "tee", "dd", "touch"];

/// 一条落在 shell 文件里的拒算：文件是从仓根起的路径，行从 1 起，片段是那一行原文（空白折成一个空格，至多 160 个字符）。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShellRefusal {
    pub file: String,
    pub line: u32,
    pub form: RefusalForm,
    pub snippet: String,
}

/// 一份 shell 脚本（传递地）source 进来的仓内文件（从仓根起的路径，按路径排好，不含脚本自己）与拒算（按文件、行、形态名、片段排好）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShellSourcedFiles {
    pub sourced_files: Vec<String>,
    pub refusals: Vec<ShellRefusal>,
}

/// bash 规范化跑不下去（bash 不在、退非 0、超时、临时目录建不了、脚本读不了）：运行条件不满足，整趟报错，不是拒算
/// （那边的 `FunctionUnitsError`）。`script` 是折的那份脚本（从仓根起的路径），`reason` 是原因。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellNormalizationFailure {
    pub script: String,
    pub reason: String,
}

impl fmt::Display for ShellNormalizationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "折 shell 脚本「{}」source 进来的文件时 bash 规范化跑不下去：{}（运行条件不满足，整趟报错，不是拒算）",
            self.script, self.reason
        )
    }
}

/// 一个工作区里每份脚本只折一趟（几十个入口读同一份脚本）；克隆出来的几份共用同一张表。折不下去的不进表。
#[derive(Clone, Debug)]
pub struct ShellSourcedFilesCache {
    by_script: Arc<Mutex<HashMap<String, Arc<ShellSourcedFiles>>>>,
    bash_program: OsString,
}

impl Default for ShellSourcedFilesCache {
    fn default() -> Self {
        Self::with_the_bash_program(OsString::from(DEFAULT_BASH_PROGRAM))
    }
}

impl ShellSourcedFilesCache {
    /// 规范化起 `bash_program`（程序名按子进程的 `PATH` 找，或给路径）的一张空表；`Default` 起的是 `bash`。
    /// 测试给一个起不来的程序，钉住 bash 起不来时闭包整趟报错。
    #[must_use]
    pub fn with_the_bash_program(bash_program: OsString) -> Self {
        Self {
            by_script: Arc::new(Mutex::new(HashMap::new())),
            bash_program,
        }
    }

    /// `shell_sourced_files(根, 脚本)`，同一份脚本第二次起取头一次的结果。
    ///
    /// # Errors
    /// bash 规范化跑不下去（同 [`shell_sourced_files`]）。
    pub fn sourced_files_of(
        &self,
        repository_root: &Path,
        script_path_from_the_repository_root: &str,
    ) -> Result<Arc<ShellSourcedFiles>, ShellNormalizationFailure> {
        let mut by_script = self
            .by_script
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(sourced) = by_script.get(script_path_from_the_repository_root) {
            return Ok(sourced.clone());
        }
        let sourced = Arc::new(shell_sourced_files_with_the_bash_program(
            repository_root,
            script_path_from_the_repository_root,
            &self.bash_program,
        )?);
        by_script.insert(
            script_path_from_the_repository_root.to_string(),
            sourced.clone(),
        );
        Ok(sourced)
    }
}

/// 这份文件按 shell 脚本认（那边的 `language_of_file` 判成 shell）：`.sh` / `.bash` 结尾，或不是 `.py` 而头一行是 `#!` 且带 `bash`
/// 或去掉尾部空白之后以 `/sh` 结尾。读不了的不算。
#[must_use]
pub fn is_shell_script(repository_root: &Path, path_from_the_repository_root: &str) -> bool {
    if path_from_the_repository_root.ends_with(".py") {
        return false;
    }
    if path_from_the_repository_root.ends_with(".sh")
        || path_from_the_repository_root.ends_with(".bash")
    {
        return true;
    }
    let Ok(bytes) = std::fs::read(repository_root.join(path_from_the_repository_root)) else {
        return false;
    };
    let first_line = match bytes.iter().position(|byte| *byte == b'\n') {
        Some(newline) => &bytes[..=newline],
        None => &bytes[..],
    };
    if !first_line.starts_with(b"#!") {
        return false;
    }
    let mut trimmed = first_line;
    while let Some((last, rest)) = trimmed.split_last() {
        if last.is_ascii_whitespace() || *last == 0x0b {
            trimmed = rest;
        } else {
            break;
        }
    }
    first_line.windows(4).any(|window| window == b"bash") || trimmed.ends_with(b"/sh")
}

/// shell 脚本（从仓根起的路径）在它自己那个 shell 里（传递地）source / . 进来的仓内文件与拒算（文件头）。规范化起的是 `bash`。
///
/// # Errors
/// bash 规范化跑不下去（bash 不在、退非 0、超时、临时目录建不了、脚本读不了）：整趟报错，不是拒算（那边同样抛 `FunctionUnitsError`）。
pub fn shell_sourced_files(
    repository_root: &Path,
    script_path_from_the_repository_root: &str,
) -> Result<ShellSourcedFiles, ShellNormalizationFailure> {
    shell_sourced_files_with_the_bash_program(
        repository_root,
        script_path_from_the_repository_root,
        OsStr::new(DEFAULT_BASH_PROGRAM),
    )
}

fn shell_sourced_files_with_the_bash_program(
    repository_root: &Path,
    script_path_from_the_repository_root: &str,
    bash_program: &OsStr,
) -> Result<ShellSourcedFiles, ShellNormalizationFailure> {
    let script = script_path_from_the_repository_root.to_string();
    let failure_of = |reason: String| ShellNormalizationFailure {
        script: script.clone(),
        reason,
    };
    let mut index = ShellIndex::new(
        repository_root.to_string_lossy().into_owned(),
        bash_program.to_os_string(),
    );
    index
        .add(std::slice::from_ref(&script))
        .map_err(failure_of)?;
    let mut files: BTreeSet<String> = BTreeSet::from([script.clone()]);
    let final_refusals = loop {
        let mut found: BTreeSet<String> = BTreeSet::new();
        let mut refusals: BTreeSet<ShellRefusal> = BTreeSet::new();
        for current in &files {
            let Some(shell_file) = index.files.get(current) else {
                continue;
            };
            refusals.extend(shell_file.refusals.iter().cloned());
            for position in 0..shell_file.units.len() {
                let place = UnitPlace {
                    file: current.clone(),
                    position,
                };
                for link in index.source_links(&place) {
                    if let Some(refusal) = link.refusal {
                        refusals.insert(refusal);
                    }
                    found.extend(link.files);
                }
            }
        }
        let grown: BTreeSet<String> = files.union(&found).cloned().collect();
        if grown == files {
            break refusals;
        }
        let added: Vec<String> = grown.difference(&files).cloned().collect();
        index.add(&added).map_err(failure_of)?;
        files = grown;
    };
    let mut ordered: Vec<ShellRefusal> = final_refusals.into_iter().collect();
    ordered.sort_by(|left, right| {
        (
            &left.file,
            left.line,
            left.form.output_name(),
            &left.snippet,
        )
            .cmp(&(
                &right.file,
                right.line,
                right.form.output_name(),
                &right.snippet,
            ))
    });
    Ok(ShellSourcedFiles {
        sourced_files: files.into_iter().filter(|file| *file != script).collect(),
        refusals: ordered,
    })
}

// ── 字符与 Python 字符串操作的对应 ─────────────────────────────────────────

fn characters_of(text: &str) -> Vec<char> {
    text.chars().collect()
}

/// `text[start:end]`（越界照 Python 截到两端，起点不在终点之前时是空串）。
fn text_between(text: &[char], start: usize, end: usize) -> String {
    let end = end.min(text.len());
    if start >= end {
        String::new()
    } else {
        text[start..end].iter().collect()
    }
}

fn starts_with_at(text: &[char], index: usize, pattern: &str) -> bool {
    pattern.chars().enumerate().all(|(offset, expected)| {
        index
            .checked_add(offset)
            .and_then(|position| text.get(position))
            == Some(&expected)
    })
}

fn find_character(text: &[char], wanted: char, start: usize) -> Option<usize> {
    (start..text.len()).find(|position| text[*position] == wanted)
}

fn find_text(haystack: &[char], needle: &[char], start: usize) -> Option<usize> {
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
fn is_python_whitespace(character: char) -> bool {
    character.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&character)
}

fn python_strip(text: &str) -> &str {
    text.trim_matches(is_python_whitespace)
}

fn python_left_strip(text: &str) -> &str {
    text.trim_start_matches(is_python_whitespace)
}

fn is_shell_name_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_shell_name_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

/// `[A-Za-z_][A-Za-z0-9_]*` 从 `start` 起的那一段的终点；不以这种字符起头是 `None`。
fn identifier_end(text: &[char], start: usize) -> Option<usize> {
    if !text.get(start).copied().is_some_and(is_shell_name_start) {
        return None;
    }
    let mut end = start + 1;
    while text.get(end).copied().is_some_and(is_shell_name_continue) {
        end += 1;
    }
    Some(end)
}

fn is_plain_identifier(word: &str) -> bool {
    let text = characters_of(word);
    identifier_end(&text, 0) == Some(text.len())
}

fn is_ascii_digit_string(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|character| character.is_ascii_digit())
}

/// 那边的 `snippet_of`：空白段折成一个空格、去掉两头，至多 160 个字符。
fn snippet_of(text: &str) -> String {
    let joined = text
        .split(is_python_whitespace)
        .filter(|piece| !piece.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    joined.chars().take(SNIPPET_CHARACTER_LIMIT).collect()
}

/// `posixpath.join(base, tail)`。
fn python_join(base: &str, tail: &str) -> String {
    if tail.starts_with('/') {
        tail.to_string()
    } else if base.is_empty() || base.ends_with('/') {
        format!("{base}{tail}")
    } else {
        format!("{base}/{tail}")
    }
}

/// `posixpath.normpath`：去掉 `.` 与空段，`..` 退一级（开头两个斜杠照留两个），不碰文件系统。
fn python_normpath(path: &str) -> String {
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
fn python_dirname(path: &str) -> String {
    let split = path.rfind('/').map_or(0, |slash| slash + 1);
    let head = &path[..split];
    if !head.is_empty() && head.chars().any(|character| character != '/') {
        head.trim_end_matches('/').to_string()
    } else {
        head.to_string()
    }
}

/// `posixpath.basename`。
fn python_basename(path: &str) -> String {
    let split = path.rfind('/').map_or(0, |slash| slash + 1);
    path[split..].to_string()
}

fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

// ── 规范文本的词法：命令切分、引号与替换的配对 ─────────────────────────────

/// `text[start]` 是 `opening`；交配对的 `closing` 的下标（跳过引号），配不上交最后一个下标。
fn matching_close(text: &[char], start: usize, opening: char, closing: char) -> usize {
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
fn substitutions(text: &[char]) -> Vec<(String, usize)> {
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
fn literal_word(word: &str) -> Option<String> {
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
struct Redirect {
    command_index: usize,
    operator: String,
    target: String,
    offset: usize,
    descriptor: Option<u64>,
}

/// `split_commands` 切出的简单命令（每条是词的序列，引号与 `$( … )`、`${ … }` 原样留在词里）与重定向。
/// `redirects` 只有每处重定向本身；`details` 另把写在 `{ … }` 收尾的 `}` 后面的重定向抄给组里的每一条命令（那边只在给了 `details` 时抄）。
struct CommandSplit {
    commands: Vec<Vec<String>>,
    redirects: Vec<Redirect>,
    details: Vec<Redirect>,
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
fn split_commands(text: &[char]) -> CommandSplit {
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
struct HeredocStart {
    start: usize,
    end: usize,
    strips_tabs: bool,
    is_quoted: bool,
    delimiter: String,
}

fn heredoc_start_at(row: &[char], position: usize) -> Option<HeredocStart> {
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

fn row_closes_heredoc(row: &str, delimiter: &str, strips_tabs: bool) -> bool {
    if strips_tabs {
        row.trim_start_matches('\t') == delimiter
    } else {
        row == delimiter
    }
}

/// here-doc 正文那几行的下标（起止按 `<<词` 与单独一行的词认；后面找不到收尾那一行的不算）。
fn heredoc_mask(rows: &[String]) -> HashSet<usize> {
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
type HeredocBody = (bool, String);

fn heredoc_bodies(rows: &[String]) -> HashMap<(usize, usize), HeredocBody> {
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
enum CommandInput {
    File,
    HereString,
    HereDocument(Option<HeredocBody>),
}

/// 重定向细目 → 命令下标 →（文件描述符号，读进来的东西）；`base` 是这一段在 `visible`（整个单元的规范文本，here-doc 正文已抹空）里的起点。
fn command_inputs(
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
enum GeneratedScript {
    /// 用 `printf` / `echo` / 只接 here-doc 的 `cat` 写出来的：`cat <<词` 写进去的正文（`printf` / `echo` 写的不当代码看）。
    Written(Vec<String>),
    /// 只由 `cp <源> <它>` 拷出来的：源词。
    Copied(Vec<String>),
}

fn generated_scripts(
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

// ── bash 规范化与切单元 ─────────────────────────────────────────────────────

static NEXT_NORMALIZATION_DIRECTORY_NUMBER: AtomicUsize = AtomicUsize::new(0);

/// 这一趟规范化的临时目录，用完（成功、失败、panic）删掉。
struct NormalizationDirectory {
    path: PathBuf,
}

impl NormalizationDirectory {
    fn create() -> Result<Self, String> {
        let nanoseconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.subsec_nanos());
        for _attempt in 0..16 {
            let number = NEXT_NORMALIZATION_DIRECTORY_NUMBER.fetch_add(1, Ordering::SeqCst);
            let path = std::env::temp_dir().join(format!(
                "singlefs-unit-graph-shell-{}-{number}-{nanoseconds}",
                std::process::id()
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(format!(
                        "建不了规范化用的临时目录 {}：{error}",
                        path.display()
                    ))
                }
            }
        }
        Err("规范化用的临时目录名连撞 16 次".to_string())
    }
}

impl Drop for NormalizationDirectory {
    fn drop(&mut self) {
        // 删不掉（被别的进程占着）只留下一个临时目录，不改结果
        let _ignored = std::fs::remove_dir_all(&self.path);
    }
}

/// 每份内容 → `declare -f` 打出来的那一份；`bash -n` 不过、有告警、包不进函数的是 `None`（那边的 `shell_normal_forms`）。
///
/// # Errors
/// `bash_program` 起不来、退非 0、超时、临时目录建不了或写不进：交原因。
fn normal_forms(contents: &[Vec<u8>], bash_program: &OsStr) -> Result<Vec<Option<String>>, String> {
    let directory = NormalizationDirectory::create()?;
    let mut paths = Vec::new();
    for (position, content) in contents.iter().enumerate() {
        let path = directory.path.join(format!("{position}.sh"));
        std::fs::write(&path, content)
            .map_err(|error| format!("写不进 {}：{error}", path.display()))?;
        paths.push(path);
    }
    let search_path = std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into());
    let mut child = Command::new(bash_program)
        .arg("--norc")
        .arg("--noprofile")
        .arg("-c")
        .arg(NORMALIZE_SCRIPT)
        .arg("function-units")
        .args(&paths)
        .current_dir(&directory.path)
        .env_clear()
        .env("PATH", search_path)
        .env("LC_ALL", "C")
        .env("HOME", &directory.path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("bash（{}）起不来：{error}", bash_program.to_string_lossy()))?;
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if started.elapsed() > NORMALIZATION_TIME_LIMIT {
                    let _killed = child.kill();
                    let _reaped = child.wait();
                    return Err(format!(
                        "bash 规范化 .sh 超过 {} 秒没跑完",
                        NORMALIZATION_TIME_LIMIT.as_secs()
                    ));
                }
                std::thread::sleep(NORMALIZATION_POLL_INTERVAL);
            }
            Err(error) => return Err(format!("等 bash 规范化退出时出错：{error}")),
        }
    };
    if !status.success() {
        return Err(format!("bash 规范化 .sh 退 {status}"));
    }
    let mut forms = Vec::new();
    for path in &paths {
        let mut normal_path = path.clone().into_os_string();
        normal_path.push(".normal");
        let normal_path = PathBuf::from(normal_path);
        if normal_path.is_file() {
            let bytes = std::fs::read(&normal_path)
                .map_err(|error| format!("读不了 {}：{error}", normal_path.display()))?;
            forms.push(Some(String::from_utf8_lossy(&bytes).into_owned()));
        } else {
            forms.push(None);
        }
    }
    Ok(forms)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShellUnitKind {
    Function,
    ScriptBody,
}

/// 一个 shell 单元：函数（套着的函数各一个，`container` 指外层函数在同一份文件的单元表里的位置）或脚本体。
/// 行号是原文里的（函数按头那一行与 `shell_block_end` 找到的收尾行，只当 `word_line` 的范围用；脚本体是整份）。
#[derive(Clone, Debug)]
struct ShellUnit {
    kind: ShellUnitKind,
    qualified_name: String,
    first_line: u32,
    last_line: u32,
    body: String,
    container: Option<usize>,
}

struct ShellFile {
    rows: Vec<String>,
    refusals: Vec<ShellRefusal>,
    units: Vec<ShellUnit>,
    functions_by_name: HashMap<String, Vec<usize>>,
}

/// `^( *)function (\S+) \(\) $` → （缩进的空格数，函数名）。
fn function_header(row: &str) -> Option<(usize, String)> {
    let indent = row
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    let after_keyword = row[indent..].strip_prefix("function ")?;
    let name = after_keyword.strip_suffix(" () ")?;
    if name.is_empty() || name.chars().any(is_python_whitespace) {
        return None;
    }
    Some((indent, name.to_string()))
}

fn line_number(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

/// 原文第 `line` 行（从 1 起）；0 与超出末行的是 `None`。
fn row_of_line(rows: &[String], line: u32) -> Option<&String> {
    let position = usize::try_from(line).ok()?.checked_sub(1)?;
    rows.get(position)
}

impl ShellFile {
    fn new(path: &str, content: &[u8], normal: Option<&str>) -> Self {
        let text = String::from_utf8_lossy(content).into_owned();
        let rows: Vec<String> = text.split('\n').map(str::to_string).collect();
        let mut shell_file = ShellFile {
            rows,
            refusals: Vec::new(),
            units: Vec::new(),
            functions_by_name: HashMap::new(),
        };
        match normal {
            Some(normal) => shell_file.parse(path, normal),
            None => shell_file.refuse_bash_syntax(path),
        }
        shell_file
    }

    fn refuse_bash_syntax(&mut self, path: &str) {
        self.refusals.push(ShellRefusal {
            file: path.to_string(),
            line: 1,
            form: RefusalForm::ShellBashSyntax,
            snippet: snippet_of(self.rows.first().map_or("", String::as_str)),
        });
    }

    fn parse(&mut self, path: &str, normal: &str) {
        let mut lines: Vec<&str> = normal.split('\n').collect();
        while lines.last() == Some(&"") {
            lines.pop();
        }
        if lines.len() < 3
            || !lines[0].starts_with(WRAPPER_FUNCTION_NAME)
            || python_strip(lines[1]) != "{"
            || lines.last() != Some(&"}")
        {
            self.refuse_bash_syntax(path);
            return;
        }
        let body: Vec<String> = lines[2..lines.len() - 1]
            .iter()
            .map(|line| (*line).to_string())
            .collect();
        let mask = heredoc_mask(&body);
        // （起行，止行，缩进，名字），按这四样排
        let mut functions: Vec<(usize, usize, usize, String)> = Vec::new();
        for (position, row) in body.iter().enumerate() {
            if mask.contains(&position) {
                continue;
            }
            let Some((indent, name)) = function_header(row) else {
                continue;
            };
            let indent_text = " ".repeat(indent);
            if position + 1 >= body.len() || body[position + 1] != format!("{indent_text}{{ ") {
                continue;
            }
            let closing = format!("{indent_text}}}");
            let end = (position + 2..body.len()).find(|later| {
                if mask.contains(later) {
                    return false;
                }
                let candidate = &body[*later];
                candidate.starts_with(&closing)
                    && candidate[closing.len()..]
                        .chars()
                        .next()
                        .is_none_or(|next| " ;|&<>)".contains(next))
            });
            if let Some(end) = end {
                functions.push((position, end, indent, name));
            }
        }
        functions.sort();
        let mut parents: Vec<Option<usize>> = Vec::new();
        let mut stack: Vec<usize> = Vec::new();
        for (function_position, function) in functions.iter().enumerate() {
            while let Some(&top) = stack.last() {
                if functions[top].1 < function.0 {
                    stack.pop();
                } else {
                    break;
                }
            }
            parents.push(stack.last().copied());
            stack.push(function_position);
        }
        let mut unit_of_function: Vec<usize> = Vec::new();
        let mut cursor = 0;
        for (function_position, (start, end, _indent, name)) in functions.iter().enumerate() {
            let parent_unit = parents[function_position].map(|parent| unit_of_function[parent]);
            let qualified_name = match parent_unit {
                Some(parent) => format!("{}::{name}", self.units[parent].qualified_name),
                None => name.clone(),
            };
            let (first_line, last_line, next_cursor) = self.original_lines(name, cursor);
            cursor = next_cursor;
            let unit_position = self.units.len();
            self.units.push(ShellUnit {
                kind: ShellUnitKind::Function,
                qualified_name,
                first_line,
                last_line,
                body: body[*start..=*end].join("\n"),
                container: parent_unit,
            });
            self.functions_by_name
                .entry(name.clone())
                .or_default()
                .push(unit_position);
            unit_of_function.push(unit_position);
        }
        let mut covered: HashSet<usize> = HashSet::new();
        for (start, end, indent, _name) in &functions {
            if *indent == 4 {
                covered.extend(*start..=*end);
            }
        }
        let script_body: Vec<&str> = body
            .iter()
            .enumerate()
            .filter(|(position, _row)| !covered.contains(position))
            .map(|(_position, row)| row.as_str())
            .collect();
        self.units.push(ShellUnit {
            kind: ShellUnitKind::ScriptBody,
            qualified_name: "<script-body>".to_string(),
            first_line: 1,
            last_line: line_number(self.rows.len().max(1)),
            body: script_body.join("\n"),
            container: None,
        });
    }

    /// 函数 `name` 在原文里的起止行（从 `cursor` 那一行起往后找头，找不到再从头找；都找不到是 1-1）与下一次找的起点（那边的 `original_lines`）。
    fn original_lines(&self, name: &str, cursor: usize) -> (u32, u32, usize) {
        let found = (cursor..self.rows.len())
            .find(|position| header_column(&self.rows[*position], name).is_some())
            .or_else(|| {
                (0..self.rows.len())
                    .find(|position| header_column(&self.rows[*position], name).is_some())
            });
        let Some(found) = found else {
            return (1, 1, cursor);
        };
        let column = header_column(&self.rows[found], name)
            .expect("上面刚在这一行找到过函数头，同一份判法再找一次");
        (
            line_number(found + 1),
            line_number(shell_block_end(&self.rows, found, column) + 1),
            found + 1,
        )
    }
}

/// 原文一行里函数 `name` 的定义头从哪一列起（那边 `original_lines` 的那条正则：前面是行首或 `;&|{(` 再接空白，
/// 头是 `function 名`（可接 `()`）或 `名()`，后面接 `{`、`(`、`#` 或行尾）；没有是 `None`。
fn header_column(row: &str, name: &str) -> Option<usize> {
    let text = characters_of(row);
    let name_characters = characters_of(name);
    (0..=text.len()).find(|group_start| {
        let before = &text[..*group_start];
        let trimmed_end = before
            .iter()
            .rposition(|character| !is_python_whitespace(*character));
        let prefix_allows = match trimmed_end {
            None => true,
            Some(last) => ";&|{(".contains(before[last]),
        };
        prefix_allows && header_group_matches_at(&text, *group_start, &name_characters)
    })
}

fn skip_python_whitespace(text: &[char], mut index: usize) -> usize {
    while index < text.len() && is_python_whitespace(text[index]) {
        index += 1;
    }
    index
}

fn header_terminator_at(text: &[char], index: usize) -> bool {
    let index = skip_python_whitespace(text, index);
    index == text.len() || matches!(text[index], '{' | '(' | '#')
}

/// `\s*\(\s*\)` 从 `index` 起配得上时交它之后的下标。
fn empty_parentheses_end(text: &[char], index: usize) -> Option<usize> {
    let open = skip_python_whitespace(text, index);
    if text.get(open) != Some(&'(') {
        return None;
    }
    let close = skip_python_whitespace(text, open + 1);
    (text.get(close) == Some(&')')).then_some(close + 1)
}

fn name_at(text: &[char], index: usize, name: &[char]) -> bool {
    index + name.len() <= text.len() && text[index..index + name.len()] == *name
}

fn header_group_matches_at(text: &[char], group_start: usize, name: &[char]) -> bool {
    if starts_with_at(text, group_start, "function")
        && text
            .get(group_start + 8)
            .copied()
            .is_some_and(is_python_whitespace)
    {
        let name_start = skip_python_whitespace(text, group_start + 8);
        if name_at(text, name_start, name) {
            let after_name = name_start + name.len();
            let with_parentheses = empty_parentheses_end(text, after_name)
                .is_some_and(|end| header_terminator_at(text, end));
            if with_parentheses || header_terminator_at(text, after_name) {
                return true;
            }
        }
    }
    if name_at(text, group_start, name) {
        if let Some(end) = empty_parentheses_end(text, group_start + name.len()) {
            return header_terminator_at(text, end);
        }
    }
    false
}

/// `\s*(?:function\s+)?[^\s(]+\s*(?:\(\s*\))?\s*` 从 `column` 起配得上时交它之后的下标。
fn header_prefix_end(row: &[char], column: usize) -> Option<usize> {
    let is_name_character = |character: char| !is_python_whitespace(character) && character != '(';
    let start = skip_python_whitespace(row, column);
    let after_keyword = (starts_with_at(row, start, "function")
        && row
            .get(start + 8)
            .copied()
            .is_some_and(is_python_whitespace))
    .then(|| skip_python_whitespace(row, start + 8));
    let name_start = match after_keyword {
        Some(after) if row.get(after).copied().is_some_and(is_name_character) => after,
        Some(_) | None => start,
    };
    if !row.get(name_start).copied().is_some_and(is_name_character) {
        return None;
    }
    let mut end = name_start;
    while row.get(end).copied().is_some_and(is_name_character) {
        end += 1;
    }
    end = skip_python_whitespace(row, end);
    if let Some(after_parentheses) = empty_parentheses_end(row, end) {
        if row.get(end) == Some(&'(') {
            end = after_parentheses;
        }
    }
    Some(skip_python_whitespace(row, end))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BlockContext {
    SingleQuote,
    DoubleQuote,
    CommandSubstitution,
}

/// 原文里 `rows[start]` 第 `column` 列起的函数定义到哪一行收尾（只当索引；认不出交起头那一行；那边的 `shell_block_end`）。
fn shell_block_end(rows: &[String], start: usize, column: usize) -> usize {
    let start_row = characters_of(&rows[start]);
    let mut column = header_prefix_end(&start_row, column).unwrap_or(column);
    let mut row_index = start;
    while row_index < rows.len() {
        let row = characters_of(&rows[row_index]);
        let rest = &row[column.min(row.len())..];
        if let Some(first) = rest
            .iter()
            .position(|character| !is_python_whitespace(*character))
        {
            column += first;
            break;
        }
        row_index += 1;
        column = 0;
    }
    if row_index >= rows.len() {
        return start;
    }
    let opener = match characters_of(&rows[row_index]).get(column) {
        Some(&character) if character == '{' || character == '(' => character,
        Some(_) | None => return start,
    };
    let mut depth: i64 = 0;
    let mut stack: Vec<BlockContext> = Vec::new();
    let mut parentheses: Vec<i64> = vec![0];
    while row_index < rows.len() {
        let row = characters_of(&rows[row_index]);
        let mut index = column;
        let mut heredocs: Vec<(String, bool)> = Vec::new();
        while index < row.len() {
            let character = row[index];
            let top = stack.last().copied();
            if top == Some(BlockContext::SingleQuote) {
                if character == '\'' {
                    stack.pop();
                }
                index += 1;
                continue;
            }
            if top == Some(BlockContext::DoubleQuote) {
                if character == '\\' {
                    index += 2;
                    continue;
                }
                if character == '"' {
                    stack.pop();
                } else if starts_with_at(&row, index, "$(") {
                    stack.push(BlockContext::CommandSubstitution);
                    parentheses.push(0);
                    index += 2;
                    continue;
                }
                index += 1;
                continue;
            }
            if character == '\\' {
                index += 2;
                continue;
            }
            if character == '\'' {
                stack.push(BlockContext::SingleQuote);
                index += 1;
                continue;
            }
            if character == '"' {
                stack.push(BlockContext::DoubleQuote);
                index += 1;
                continue;
            }
            if character == '#' && (index == 0 || " \t;".contains(row[index - 1])) {
                break;
            }
            if starts_with_at(&row, index, "<<") && !starts_with_at(&row, index, "<<<") {
                if let Some(heredoc) = heredoc_start_at(&row, index) {
                    heredocs.push((heredoc.delimiter, heredoc.strips_tabs));
                    index = heredoc.end;
                    continue;
                }
            }
            if starts_with_at(&row, index, "${") {
                index = matching_close(&row, index + 1, '{', '}') + 1;
                continue;
            }
            if starts_with_at(&row, index, "$(") {
                stack.push(BlockContext::CommandSubstitution);
                parentheses.push(0);
                index += 2;
                continue;
            }
            if top == Some(BlockContext::CommandSubstitution) {
                if character == '(' {
                    if let Some(open) = parentheses.last_mut() {
                        *open += 1;
                    }
                } else if character == ')' {
                    if parentheses.last() == Some(&0) {
                        stack.pop();
                        parentheses.pop();
                    } else if let Some(open) = parentheses.last_mut() {
                        *open -= 1;
                    }
                }
                index += 1;
                continue;
            }
            if opener == '{' {
                let is_word_start = index == 0 || " \t;&|()".contains(row[index - 1]);
                let is_word_end = index + 1 >= row.len() || " \t;)|&<>".contains(row[index + 1]);
                if character == '{' && is_word_start && is_word_end {
                    depth += 1;
                } else if character == '}' && is_word_start && is_word_end {
                    depth -= 1;
                    if depth == 0 {
                        return row_index;
                    }
                }
            } else if character == '(' {
                depth += 1;
            } else if character == ')' {
                depth -= 1;
                if depth == 0 {
                    return row_index;
                }
            }
            index += 1;
        }
        row_index += 1;
        column = 0;
        for (delimiter, strips_tabs) in heredocs {
            while row_index < rows.len()
                && !row_closes_heredoc(&rows[row_index], &delimiter, strips_tabs)
            {
                row_index += 1;
            }
            row_index += 1;
        }
    }
    start
}

// ── 单元的事实与折叠 ───────────────────────────────────────────────────────

/// 一份文件里的一个单元（文件从仓根起，`position` 是它在那份文件单元表里的位置）。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UnitPlace {
    file: String,
    position: usize,
}

/// 一处 `source <词>` / `. <词>`：路径词与前面 `cd` 过去的目录词。
#[derive(Clone, Debug)]
struct SourceInvocation {
    word: String,
    directory: Option<String>,
}

/// 一个单元（或一段写出来的脚本正文）里的事实（那边的 `ShellFacts`，只留 source 用得到的那几样）。
#[derive(Clone, Debug, Default)]
struct ShellFacts {
    commands: Vec<Vec<String>>,
    assignments: HashMap<String, Vec<String>>,
    local_names: HashSet<String>,
    source_invocations: Vec<SourceInvocation>,
    directory: Option<String>,
    generated: HashMap<String, GeneratedScript>,
}

/// `^([A-Za-z_][A-Za-z0-9_]*)(?:\[(?:[^\[\]]|\[[^\]]*\])*\])?\+?=(.*)$` → （名字，值）。
fn assignment_word(word: &str) -> Option<(String, String)> {
    let text = characters_of(word);
    let name_end = identifier_end(&text, 0)?;
    let mut index = name_end;
    if text.get(index) == Some(&'[') {
        let mut scan = index + 1;
        let mut closing = None;
        while scan < text.len() {
            match text[scan] {
                '[' => match find_character(&text, ']', scan + 1) {
                    Some(close) => scan = close + 1,
                    None => break,
                },
                ']' => {
                    closing = Some(scan);
                    break;
                }
                _ => scan += 1,
            }
        }
        index = closing? + 1;
    }
    if text.get(index) == Some(&'+') {
        index += 1;
    }
    if text.get(index) != Some(&'=') {
        return None;
    }
    Some((
        text_between(&text, 0, name_end),
        text_between(&text, index + 1, text.len()),
    ))
}

/// 一条简单命令记进事实（那边 `command_facts` 里 source 用得到的那几支：关键字、`for … in`、声明与赋值、`cd`、`source` / `.`）。
fn command_facts(command: &[String], facts: &mut ShellFacts) {
    let mut words = command;
    while let Some(first) = words.first() {
        if SHELL_KEYWORDS.contains(&first.as_str()) {
            words = &words[1..];
        } else {
            break;
        }
    }
    if words.is_empty() {
        return;
    }
    if words[0] == "for" && words.len() >= 3 && words[2] == "in" {
        for value in &words[3..] {
            facts
                .assignments
                .entry(words[1].clone())
                .or_default()
                .push(value.clone());
        }
        return;
    }
    let is_declaration = SHELL_DECLARATIONS.contains(&words[0].as_str());
    let is_scoped = words[0] == "local"
        || ((words[0] == "declare" || words[0] == "typeset")
            && !words[1..].iter().any(|word| word == "-g"));
    let mut position = usize::from(is_declaration);
    while position < words.len() {
        let word = &words[position];
        let assignment = assignment_word(word);
        if assignment.is_some() || (is_scoped && is_plain_identifier(word)) {
            let name = match assignment {
                Some((name, value)) => {
                    facts
                        .assignments
                        .entry(name.clone())
                        .or_default()
                        .push(value);
                    name
                }
                None => word.clone(),
            };
            if is_scoped {
                facts.local_names.insert(name);
            }
            position += 1;
            continue;
        }
        if is_declaration && word.starts_with('-') {
            position += 1;
            continue;
        }
        break;
    }
    if is_declaration {
        return;
    }
    let rest = &words[position..];
    if rest.is_empty() {
        return;
    }
    let first = literal_word(&rest[0]);
    if first.as_deref() == Some("cd") {
        let arguments: Vec<&String> = rest[1..]
            .iter()
            .filter(|word| !literal_word(word).unwrap_or_default().starts_with('-'))
            .collect();
        facts.directory = if arguments.len() == 1 {
            Some(arguments[0].clone())
        } else {
            None
        };
        return;
    }
    if matches!(first.as_deref(), Some("source" | ".")) && rest.len() > 1 {
        facts.source_invocations.push(SourceInvocation {
            word: rest[1].clone(),
            directory: facts.directory.clone(),
        });
    }
}

/// `$名字`、`$1` 这类简单写法在 `start`（`$` 之后）起的名字的终点（`[A-Za-z_][A-Za-z0-9_]*|[0-9@*#?$!-]`）。
fn simple_parameter_end(text: &[char], start: usize) -> Option<usize> {
    if let Some(end) = identifier_end(text, start) {
        return Some(end);
    }
    let character = *text.get(start)?;
    (character.is_ascii_digit() || "@*#?$!-".contains(character)).then_some(start + 1)
}

/// `\$\{?([A-Za-z_][A-Za-z0-9_]*)` 在词里的每一处的名字（不重叠，从左往右）。
fn variable_references(word: &str) -> Vec<String> {
    let text = characters_of(word);
    let mut names = Vec::new();
    let mut index = 0;
    while index < text.len() {
        if text[index] != '$' {
            index += 1;
            continue;
        }
        let braced_end = if text.get(index + 1) == Some(&'{') {
            identifier_end(&text, index + 2).map(|end| (index + 2, end))
        } else {
            None
        };
        match braced_end.or_else(|| identifier_end(&text, index + 1).map(|end| (index + 1, end))) {
            Some((name_start, name_end)) => {
                names.push(text_between(&text, name_start, name_end));
                index = name_end;
            }
            None => index += 1,
        }
    }
    names
}

/// `^(?:\$\{(?:IDENT|[1-9])(?:[:]?[-=?][^{}]*)?\}|\$(?:IDENT|[1-9]))(/.+)$` → 斜杠起的字面尾巴（那边的 `SHELL_TREE_PREFIX`）。
fn tree_prefix_tail(unquoted: &str) -> Option<String> {
    let text = characters_of(unquoted);
    if text.first() != Some(&'$') {
        return None;
    }
    let name_end = |start: usize| -> Option<usize> {
        identifier_end(&text, start).or_else(|| {
            text.get(start)
                .is_some_and(|character| ('1'..='9').contains(character))
                .then_some(start + 1)
        })
    };
    let after_prefix = if text.get(1) == Some(&'{') {
        let after_name = name_end(2)?;
        let with_operator = {
            let operator = if text.get(after_name) == Some(&':') {
                after_name + 1
            } else {
                after_name
            };
            if text
                .get(operator)
                .is_some_and(|character| "-=?".contains(*character))
            {
                let mut scan = operator + 1;
                while scan < text.len() && text[scan] != '{' && text[scan] != '}' {
                    scan += 1;
                }
                (text.get(scan) == Some(&'}')).then_some(scan + 1)
            } else {
                None
            }
        };
        match with_operator {
            Some(end) => end,
            None if text.get(after_name) == Some(&'}') => after_name + 1,
            None => return None,
        }
    } else {
        name_end(1)?
    };
    if text.get(after_prefix) != Some(&'/') {
        return None;
    }
    let mut content = &text[after_prefix + 1..];
    if content.last() == Some(&'\n') {
        content = &content[..content.len() - 1];
    }
    if content.is_empty() || content.contains(&'\n') {
        return None;
    }
    Some(format!("/{}", content.iter().collect::<String>()))
}

/// `^\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?$` → 名字。
fn bare_variable_name(unquoted: &str) -> Option<String> {
    let text = characters_of(unquoted);
    if text.first() != Some(&'$') {
        return None;
    }
    let name_start = if text.get(1) == Some(&'{') { 2 } else { 1 };
    let name_end = identifier_end(&text, name_start)?;
    let mut end = name_end;
    if text.get(end) == Some(&'}') {
        end += 1;
    }
    (end == text.len()).then(|| text_between(&text, name_start, name_end))
}

/// 一段尾巴 `[A-Za-z0-9_.+@,=-]+(?:/[A-Za-z0-9_.+@,=-]+)+` 整个匹配（那边的 `TREE_COPY_TAIL`）。
fn is_tree_copy_tail(tail: &str) -> bool {
    let segments: Vec<&str> = tail.split('/').collect();
    segments.len() >= 2
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && segment.chars().all(|character| {
                    character.is_ascii_alphanumeric() || "_.+@,=-".contains(character)
                })
        })
}

/// 折出来的路径落在哪：仓里的一份文件（从仓根起）、根外、仓里不在。
enum TargetStatus {
    Inside(String),
    Outside,
    Missing,
}

/// 一个单元里一处 source 认出来的结果：连到的文件，或一条拒算。
struct SourceLink {
    files: Vec<String>,
    refusal: Option<ShellRefusal>,
}

/// 那边的 `ShellIndex`：装进来的 shell 文件、各单元的事实、source 边与命令替换里 `cat <<词` 的输出。
struct ShellIndex {
    root: String,
    /// 规范化起的 bash（`DEFAULT_BASH_PROGRAM`，测试换成起不来的程序）。
    bash_program: OsString,
    file_order: Vec<String>,
    files: HashMap<String, ShellFile>,
    facts: HashMap<UnitPlace, ShellFacts>,
    sources_of: HashMap<String, BTreeSet<String>>,
    sourcers_of: HashMap<String, BTreeSet<String>>,
    substitution_outputs: RefCell<HashMap<String, BTreeSet<HeredocBody>>>,
}

impl ShellIndex {
    fn new(root: String, bash_program: OsString) -> Self {
        ShellIndex {
            root,
            bash_program,
            file_order: Vec::new(),
            files: HashMap::new(),
            facts: HashMap::new(),
            sources_of: HashMap::new(),
            sourcers_of: HashMap::new(),
            substitution_outputs: RefCell::new(HashMap::new()),
        }
    }

    /// 装进几份文件（已装的、不在的跳过），切单元、记事实，再算一遍 source 边。
    ///
    /// # Errors
    /// 读不了或 bash 规范化起不来：交原因。
    fn add(&mut self, paths: &[String]) -> Result<(), String> {
        let mut new_paths: Vec<String> = Vec::new();
        for path in paths {
            if !new_paths.contains(path)
                && !self.files.contains_key(path)
                && is_file(&python_join(&self.root, path))
            {
                new_paths.push(path.clone());
            }
        }
        if new_paths.is_empty() {
            return Ok(());
        }
        let mut contents = Vec::new();
        for path in &new_paths {
            let absolute = python_join(&self.root, path);
            contents.push(
                std::fs::read(&absolute).map_err(|error| format!("读不了 {absolute}：{error}"))?,
            );
        }
        let forms = normal_forms(&contents, &self.bash_program)?;
        for ((path, content), form) in new_paths.iter().zip(&contents).zip(&forms) {
            let shell_file = ShellFile::new(path, content, form.as_deref());
            self.file_order.push(path.clone());
            self.files.insert(path.clone(), shell_file);
        }
        for path in &new_paths {
            let bodies: Vec<String> = self.files[path]
                .units
                .iter()
                .map(|unit| unit.body.clone())
                .collect();
            for (position, body) in bodies.iter().enumerate() {
                let facts = self.unit_facts(body);
                self.facts.insert(
                    UnitPlace {
                        file: path.clone(),
                        position,
                    },
                    facts,
                );
            }
        }
        self.compute_source_edges();
        Ok(())
    }

    /// source 边（有向：谁 source 了谁）：折路径要用变量，变量又可能赋在被 source 的那份里，所以从空的起来回算到不再变（至多三趟）。
    /// 反向的边在算完之后才换（那边的 `compute_groups` 同样：算的时候用的是上一次的）。
    fn compute_source_edges(&mut self) {
        self.sources_of = self
            .file_order
            .iter()
            .map(|path| (path.clone(), BTreeSet::new()))
            .collect();
        for _round in 0..SOURCE_EDGE_ROUNDS {
            let mut edges: HashMap<String, BTreeSet<String>> = HashMap::new();
            for path in &self.file_order {
                for position in 0..self.files[path].units.len() {
                    let place = UnitPlace {
                        file: path.clone(),
                        position,
                    };
                    for link in self.source_links(&place) {
                        for target in link.files {
                            if self.files.contains_key(&target) && target != *path {
                                edges.entry(path.clone()).or_default().insert(target);
                            }
                        }
                    }
                }
            }
            let sources: HashMap<String, BTreeSet<String>> = self
                .file_order
                .iter()
                .map(|path| (path.clone(), edges.remove(path).unwrap_or_default()))
                .collect();
            if sources == self.sources_of {
                break;
            }
            self.sources_of = sources;
        }
        let mut sourcers: HashMap<String, BTreeSet<String>> = HashMap::new();
        for (path, targets) in &self.sources_of {
            for target in targets {
                sourcers
                    .entry(target.clone())
                    .or_default()
                    .insert(path.clone());
            }
        }
        self.sourcers_of = sourcers;
    }

    /// 这几份与它们（传递地）source 进来的全部文件。
    fn downward(&self, paths: &[String]) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut pending: Vec<String> = paths.to_vec();
        while let Some(path) = pending.pop() {
            if !seen.insert(path.clone()) {
                continue;
            }
            if let Some(targets) = self.sources_of.get(&path) {
                pending.extend(targets.iter().cloned());
            }
        }
        seen
    }

    /// （传递地）source 了这一份的全部文件，不含它自己。
    fn upward(&self, path: &str) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut pending: Vec<String> = self
            .sourcers_of
            .get(path)
            .map(|sourcers| sourcers.iter().cloned().collect())
            .unwrap_or_default();
        while let Some(item) = pending.pop() {
            if item == path || !seen.insert(item.clone()) {
                continue;
            }
            if let Some(sourcers) = self.sourcers_of.get(&item) {
                pending.extend(sourcers.iter().cloned());
            }
        }
        seen
    }

    fn facts_of(&self, place: &UnitPlace) -> &ShellFacts {
        self.facts
            .get(place)
            .expect("单元的事实在装进这份文件时就记下了（add）；这里没有说明单元的位置不是从这份文件的单元表里取的")
    }

    /// 一个单元的规范文本 → 事实（那边的 `facts`）：切命令（here-doc 正文抹空）、写出来的脚本、逐条命令与命令替换里的命令。
    fn unit_facts(&self, body: &str) -> ShellFacts {
        let rows: Vec<String> = body.split('\n').map(str::to_string).collect();
        let mask = heredoc_mask(&rows);
        let visible: Vec<char> = characters_of(
            &rows
                .iter()
                .enumerate()
                .map(|(position, row)| {
                    if mask.contains(&position) {
                        ""
                    } else {
                        row.as_str()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let split = split_commands(&visible);
        let bodies = if mask.is_empty() {
            HashMap::new()
        } else {
            heredoc_bodies(&rows)
        };
        let mut facts = ShellFacts {
            generated: generated_scripts(&split.commands, &split.redirects, &rows, &mask),
            commands: split.commands.clone(),
            assignments: HashMap::new(),
            local_names: HashSet::new(),
            source_invocations: Vec::new(),
            directory: None,
        };
        for command in &split.commands {
            command_facts(command, &mut facts);
        }
        self.scan_substitutions(&visible, 0, &visible, &bodies, &mut facts, 0);
        facts
    }

    /// 一段独立的正文（写出来的脚本）：切命令、认 here-doc 与命令替换（那边的 `scan_text`）。
    fn text_facts(&self, text: &str) -> ShellFacts {
        let rows: Vec<String> = text.split('\n').map(str::to_string).collect();
        let mask = heredoc_mask(&rows);
        let visible: Vec<char> = characters_of(
            &rows
                .iter()
                .enumerate()
                .map(|(position, row)| {
                    if mask.contains(&position) {
                        ""
                    } else {
                        row.as_str()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let split = split_commands(&visible);
        let bodies = if mask.is_empty() {
            HashMap::new()
        } else {
            heredoc_bodies(&rows)
        };
        let mut facts = ShellFacts::default();
        for command in &split.commands {
            command_facts(command, &mut facts);
        }
        self.scan_substitutions(&visible, 0, &visible, &bodies, &mut facts, 0);
        facts
    }

    /// 命令替换 `$( … )` 与反引号里的命令照样是要跑的命令：逐层切开再认；只有一个不带参数的 `cat` 接一份 here-doc 的，记下它的输出。
    fn scan_substitutions(
        &self,
        fragment: &[char],
        base: usize,
        visible: &[char],
        bodies: &HashMap<(usize, usize), HeredocBody>,
        facts: &mut ShellFacts,
        depth: usize,
    ) {
        if depth > SUBSTITUTION_DEPTH_LIMIT {
            return;
        }
        for (inner, offset) in substitutions(fragment) {
            let inner_characters = characters_of(&inner);
            let split = split_commands(&inner_characters);
            for command in &split.commands {
                command_facts(command, facts);
            }
            if split.commands.len() == 1
                && split.commands[0].len() == 1
                && literal_word(&split.commands[0][0]).as_deref() == Some("cat")
            {
                let inputs = command_inputs(&split.details, base + offset, visible, bodies);
                if let Some([(0, CommandInput::HereDocument(Some(body)))]) =
                    inputs.get(&0).map(Vec::as_slice)
                {
                    self.substitution_outputs
                        .borrow_mut()
                        .entry(inner.clone())
                        .or_default()
                        .insert(body.clone());
                }
            }
            self.scan_substitutions(
                &inner_characters,
                base + offset,
                visible,
                bodies,
                facts,
                depth + 1,
            );
        }
    }

    /// 一个变量的赋值词（那边的 `assignments_of`）：函数自己赋过的只看自己的；没有看套着它的外层函数；再没有取它那份文件里全部单元的
    /// （别的函数里 `local` 的不算），还没有取它（传递地）source 进来的那几份的，再没有取 source 了它的那几份的。
    fn assignments_of(&self, name: &str, place: &UnitPlace) -> Vec<String> {
        let shell_file = &self.files[&place.file];
        let unit = &shell_file.units[place.position];
        if unit.kind == ShellUnitKind::Function {
            if let Some(own) = self.facts_of(place).assignments.get(name) {
                if !own.is_empty() {
                    return own.clone();
                }
            }
        }
        let mut container = unit.container;
        while let Some(container_position) = container {
            let container_place = UnitPlace {
                file: place.file.clone(),
                position: container_position,
            };
            if let Some(enclosing) = self.facts_of(&container_place).assignments.get(name) {
                if !enclosing.is_empty() {
                    return enclosing.clone();
                }
            }
            container = shell_file.units[container_position].container;
        }
        let gathered = |paths: BTreeSet<String>| -> Vec<String> {
            let mut found = Vec::new();
            for path in paths {
                let Some(other_file) = self.files.get(&path) else {
                    continue;
                };
                for (position, other) in other_file.units.iter().enumerate() {
                    let other_facts = self.facts_of(&UnitPlace {
                        file: path.clone(),
                        position,
                    });
                    if other.kind == ShellUnitKind::Function
                        && other_facts.local_names.contains(name)
                    {
                        continue;
                    }
                    if let Some(values) = other_facts.assignments.get(name) {
                        found.extend(values.iter().cloned());
                    }
                }
            }
            found
        };
        let own_file = BTreeSet::from([place.file.clone()]);
        let mut sourced = self.downward(std::slice::from_ref(&place.file));
        sourced.remove(&place.file);
        for paths in [own_file, sourced, self.upward(&place.file)] {
            let found = gathered(paths);
            if !found.is_empty() {
                return found;
            }
        }
        Vec::new()
    }

    /// 一个词的全部可能取值；有一处折不出是 `None`（那边的 `fold_word`）。
    fn fold_word(
        &self,
        word: &str,
        place: &UnitPlace,
        progress: &BTreeSet<String>,
    ) -> Option<BTreeSet<String>> {
        let text = characters_of(word);
        let total = text.len();
        let mut results: BTreeSet<String> = BTreeSet::from([String::new()]);
        let extend =
            |prefixes: &BTreeSet<String>, suffixes: &BTreeSet<String>| -> BTreeSet<String> {
                prefixes
                    .iter()
                    .flat_map(|prefix| {
                        suffixes
                            .iter()
                            .map(move |suffix| format!("{prefix}{suffix}"))
                    })
                    .collect()
            };
        let mut index = 0;
        let mut quoted = false;
        while index < total {
            let character = text[index];
            if character == '"' {
                quoted = !quoted;
                index += 1;
                continue;
            }
            if character == '\'' && !quoted {
                let end = find_character(&text, '\'', index + 1)?;
                results = extend(
                    &results,
                    &BTreeSet::from([text_between(&text, index + 1, end)]),
                );
                index = end + 1;
                continue;
            }
            if character == '\\' && index + 1 < total {
                if quoted && !"$`\"\\\n".contains(text[index + 1]) {
                    results = extend(&results, &BTreeSet::from([character.to_string()]));
                    index += 1;
                    continue;
                }
                results = extend(&results, &BTreeSet::from([text[index + 1].to_string()]));
                index += 2;
                continue;
            }
            if character == '$' {
                if starts_with_at(&text, index, "$((") {
                    return None;
                }
                let values = if starts_with_at(&text, index, "$(") {
                    let end = matching_close(&text, index + 1, '(', ')');
                    let values = self.fold_substitution(
                        &text_between(&text, index + 2, end),
                        place,
                        progress,
                    );
                    index = end + 1;
                    values
                } else if starts_with_at(&text, index, "${") {
                    let end = matching_close(&text, index + 1, '{', '}');
                    let values =
                        self.fold_parameter(&text_between(&text, index + 2, end), place, progress);
                    index = end + 1;
                    values
                } else {
                    let Some(name_end) = simple_parameter_end(&text, index + 1) else {
                        results = extend(&results, &BTreeSet::from(["$".to_string()]));
                        index += 1;
                        continue;
                    };
                    let values = self.fold_parameter(
                        &text_between(&text, index + 1, name_end),
                        place,
                        progress,
                    );
                    index = name_end;
                    values
                };
                results = extend(&results, &values?);
                if results.len() > FOLD_VALUE_LIMIT {
                    return None;
                }
                continue;
            }
            if character == '`' {
                let end = find_character(&text, '`', index + 1)?;
                let values =
                    self.fold_substitution(&text_between(&text, index + 1, end), place, progress)?;
                results = extend(&results, &values);
                index = end + 1;
                continue;
            }
            if character == '~' && index == 0 && !quoted {
                return None;
            }
            results = extend(&results, &BTreeSet::from([character.to_string()]));
            index += 1;
        }
        Some(results)
    }

    /// `${ … }` 里面那一段（或 `$名字` 的名字）的取值（那边的 `fold_parameter`）。
    fn fold_parameter(
        &self,
        inner: &str,
        place: &UnitPlace,
        progress: &BTreeSet<String>,
    ) -> Option<BTreeSet<String>> {
        let text = characters_of(inner);
        let name_end = if let Some(end) = identifier_end(&text, 0) {
            end
        } else if text.first().is_some_and(char::is_ascii_digit) {
            text.iter()
                .position(|character| !character.is_ascii_digit())
                .unwrap_or(text.len())
        } else if text
            .first()
            .is_some_and(|character| "@*#?$!-".contains(*character))
        {
            1
        } else {
            return None;
        };
        let name = text_between(&text, 0, name_end);
        let (subscript, rest_start) = if text.get(name_end) == Some(&'[') {
            match find_character(&text, ']', name_end + 1) {
                Some(close) => (Some(text_between(&text, name_end, close + 1)), close + 1),
                None => (None, name_end),
            }
        } else {
            (None, name_end)
        };
        let rest = text_between(&text, rest_start, text.len());
        let absolute = python_join(&self.root, &place.file);
        let base: Option<BTreeSet<String>> = if (name == "BASH_SOURCE"
            && matches!(subscript.as_deref(), None | Some("[0]")))
            || name == "0"
        {
            Some(BTreeSet::from([absolute]))
        } else if name == "CLAUDE_PROJECT_DIR" {
            Some(BTreeSet::from([self.root.clone()]))
        } else if is_ascii_digit_string(&name)
            || (name.chars().count() == 1 && "@*#?$!-".contains(name.as_str()))
            || subscript.is_some()
        {
            None
        } else if progress.contains(&name) {
            Some(BTreeSet::new())
        } else {
            let words = self.assignments_of(&name, place);
            if words.is_empty() {
                None
            } else {
                let mut next_progress = progress.clone();
                next_progress.insert(name.clone());
                let mut values: Option<BTreeSet<String>> = Some(BTreeSet::new());
                for value in &words {
                    match self.fold_word(value, place, &next_progress) {
                        Some(folded) => {
                            if let Some(values) = values.as_mut() {
                                values.extend(folded);
                            }
                        }
                        None => {
                            values = None;
                            break;
                        }
                    }
                }
                if let Some(values) = values.as_mut() {
                    if values.len() > 1 {
                        values.remove("");
                    }
                }
                values
            }
        };
        if rest.is_empty() {
            return base;
        }
        for operator in [":-", ":=", "-", "="] {
            if let Some(default_word) = rest.strip_prefix(operator) {
                let default = self.fold_word(default_word, place, progress)?;
                return Some(match base {
                    Some(base) if !base.is_empty() => base.union(&default).cloned().collect(),
                    Some(_) | None => default,
                });
            }
        }
        if rest.starts_with(":?") || rest.starts_with('?') {
            return base;
        }
        match base {
            Some(base) if rest == "%/*" && !base.is_empty() => {
                Some(base.iter().map(|value| python_dirname(value)).collect())
            }
            Some(_) | None => None,
        }
    }

    /// `$( … )` 里面那一段的输出（那边的 `fold_substitution`）。
    fn fold_substitution(
        &self,
        inner: &str,
        place: &UnitPlace,
        progress: &BTreeSet<String>,
    ) -> Option<BTreeSet<String>> {
        let captured = self
            .substitution_outputs
            .borrow()
            .get(inner)
            .filter(|outputs| !outputs.is_empty())
            .cloned();
        let captured = captured.or_else(|| self.function_output(inner, place));
        if let Some(captured) = captured.filter(|outputs| !outputs.is_empty()) {
            // $(cat <<'词' … 词)：输出就是 here-doc 正文（不带引号的词照 bash 展开正文）
            let mut values = BTreeSet::new();
            for (is_quoted, body) in &captured {
                if *is_quoted {
                    values.insert(body.clone());
                } else {
                    values.extend(self.fold_heredoc(body, place, progress)?);
                }
            }
            return Some(values);
        }
        let split = split_commands(&characters_of(inner));
        let commands = &split.commands;
        let non_option_arguments = |words: &[String]| -> Vec<String> {
            words
                .iter()
                .filter(|word| !literal_word(word).unwrap_or_default().starts_with('-'))
                .cloned()
                .collect()
        };
        if commands.len() == 1 {
            let words = &commands[0];
            let head = literal_word(&words[0]);
            let arguments = non_option_arguments(&words[1..]);
            if matches!(
                head.as_deref(),
                Some("dirname" | "realpath" | "readlink" | "basename")
            ) && arguments.len() == 1
            {
                let folded = self.fold_word(&arguments[0], place, progress)?;
                return Some(match head.as_deref() {
                    Some("dirname") => folded.iter().map(|value| python_dirname(value)).collect(),
                    Some("basename") => folded.iter().map(|value| python_basename(value)).collect(),
                    Some(_) | None => folded.iter().map(|value| python_normpath(value)).collect(),
                });
            }
            if head.as_deref() == Some("git") {
                let values: Vec<Option<String>> =
                    words.iter().map(|word| literal_word(word)).collect();
                return self.git_repository_location(&values);
            }
            return None;
        }
        if commands.len() == 2
            && literal_word(&commands[0][0]).as_deref() == Some("cd")
            && literal_word(&commands[1][0]).as_deref() == Some("pwd")
        {
            let arguments = non_option_arguments(&commands[0][1..]);
            if arguments.len() == 1 {
                let folded = self.fold_word(&arguments[0], place, progress)?;
                return Some(folded.iter().map(|value| python_normpath(value)).collect());
            }
        }
        None
    }

    /// `git … rev-parse … --show-toplevel` → 根，`--git-common-dir` → 根下的 `.git`；别的 git 用法折不出（那边的 `git_repository_location`）。
    fn git_repository_location(&self, words: &[Option<String>]) -> Option<BTreeSet<String>> {
        let is_word = |wanted: &str| words.iter().any(|word| word.as_deref() == Some(wanted));
        if words.first().and_then(Option::as_deref) != Some("git") || !is_word("rev-parse") {
            return None;
        }
        if is_word("--show-toplevel") {
            return Some(BTreeSet::from([self.root.clone()]));
        }
        if is_word("--git-common-dir") {
            return Some(BTreeSet::from([python_join(&self.root, ".git")]));
        }
        None
    }

    /// `$(函数名)`：看得见的同名函数整个函数体只有一条 `printf '%s\n' <字面的行…>` 或 `cat <<词 … 词` 的 → 它打的那几行；
    /// 有一个不是这种写法 → `None`（那边的 `function_output`）。
    fn function_output(&self, inner: &str, place: &UnitPlace) -> Option<BTreeSet<HeredocBody>> {
        let split = split_commands(&characters_of(inner));
        if split.commands.len() != 1 || split.commands[0].len() != 1 {
            return None;
        }
        let name = literal_word(&split.commands[0][0])?;
        if name.is_empty() || !is_plain_identifier(&name) {
            return None;
        }
        let mut paths = vec![place.file.clone()];
        let mut sourced = self.downward(std::slice::from_ref(&place.file));
        sourced.remove(&place.file);
        paths.extend(sourced);
        paths.extend(self.upward(&place.file));
        let mut functions: Vec<UnitPlace> = Vec::new();
        for path in &paths {
            if let Some(shell_file) = self.files.get(path) {
                for position in shell_file
                    .functions_by_name
                    .get(&name)
                    .into_iter()
                    .flatten()
                {
                    functions.push(UnitPlace {
                        file: path.clone(),
                        position: *position,
                    });
                }
            }
        }
        if functions.is_empty() {
            return None;
        }
        let mut outputs = BTreeSet::new();
        for function in &functions {
            let commands: Vec<&Vec<String>> = self
                .facts_of(function)
                .commands
                .iter()
                .filter(|command| {
                    let is_the_function_header = matches!(
                        command.as_slice(),
                        [keyword, header_name] if keyword == "function" && *header_name == name
                    );
                    let is_a_brace = matches!(
                        command.as_slice(),
                        [brace] if brace == "{" || brace == "}"
                    );
                    !is_the_function_header && !is_a_brace
                })
                .collect();
            if commands.len() != 1 {
                return None;
            }
            let command = commands[0];
            let values: Vec<Option<String>> =
                command.iter().map(|word| literal_word(word)).collect();
            if command.len() >= 2
                && values[0].as_deref() == Some("printf")
                && values[1].as_deref() == Some("%s\\n")
                && values.iter().all(Option::is_some)
            {
                let lines: Vec<String> = values[2..].iter().flatten().cloned().collect();
                outputs.insert((true, lines.join("\n")));
                continue;
            }
            let body = self.function_heredoc(function);
            if values.len() == 1 && values[0].as_deref() == Some("cat") {
                if let Some(body) = body {
                    outputs.insert(body);
                    continue;
                }
            }
            return None;
        }
        Some(outputs)
    }

    /// 函数体是一条 `cat <<词 … 词` → （带不带引号，正文）。
    fn function_heredoc(&self, function: &UnitPlace) -> Option<HeredocBody> {
        let body = &self.files[&function.file].units[function.position].body;
        let rows: Vec<String> = body.split('\n').map(str::to_string).collect();
        let bodies = heredoc_bodies(&rows);
        if bodies.len() == 1 {
            bodies.into_values().next()
        } else {
            None
        }
    }

    /// 不带引号的 here-doc 正文：`$名字`、`${…}`、`$( … )`、反引号逐处折（`\$`、`` \` ``、`\\` 与续行照 bash 的规矩），别的字原样（那边的 `fold_heredoc`）。
    fn fold_heredoc(
        &self,
        body: &str,
        place: &UnitPlace,
        progress: &BTreeSet<String>,
    ) -> Option<BTreeSet<String>> {
        let text = characters_of(body);
        let total = text.len();
        let mut results: BTreeSet<String> = BTreeSet::from([String::new()]);
        let mut literal = String::new();
        let mut index = 0;
        while index < total {
            let character = text[index];
            if character == '\\' && index + 1 < total && "$`\\\n".contains(text[index + 1]) {
                if text[index + 1] != '\n' {
                    literal.push(text[index + 1]);
                }
                index += 2;
                continue;
            }
            if character == '$' || character == '`' {
                if starts_with_at(&text, index, "$((") {
                    return None;
                }
                let end = if starts_with_at(&text, index, "$(") {
                    matching_close(&text, index + 1, '(', ')') + 1
                } else if starts_with_at(&text, index, "${") {
                    matching_close(&text, index + 1, '{', '}') + 1
                } else if character == '`' {
                    find_character(&text, '`', index + 1)? + 1
                } else if let Some(name_end) = simple_parameter_end(&text, index + 1) {
                    name_end
                } else {
                    literal.push('$');
                    index += 1;
                    continue;
                };
                let values = self.fold_word(&text_between(&text, index, end), place, progress)?;
                let prefix = std::mem::take(&mut literal);
                results = results
                    .iter()
                    .flat_map(|result| {
                        let prefix = &prefix;
                        values
                            .iter()
                            .map(move |value| format!("{result}{prefix}{value}"))
                    })
                    .collect();
                if results.len() > FOLD_VALUE_LIMIT {
                    return None;
                }
                index = end;
                continue;
            }
            literal.push(character);
            index += 1;
        }
        Some(
            results
                .iter()
                .map(|result| format!("{result}{literal}"))
                .collect(),
        )
    }

    /// 折出来的路径值 → 落在哪（那边的 `target_of`）：相对路径先按前面 cd 过去的目录试，再按根、再按这份文件所在的目录。
    fn target_of(
        &self,
        value: &str,
        place: &UnitPlace,
        directories: Option<&BTreeSet<String>>,
    ) -> TargetStatus {
        if value.starts_with('/') {
            return match self.inside_root(value) {
                None => TargetStatus::Outside,
                Some(relative) if is_file(&python_join(&self.root, &relative)) => {
                    TargetStatus::Inside(relative)
                }
                Some(_) => TargetStatus::Missing,
            };
        }
        let mut bases: Vec<String> = directories
            .map(|directory_set| directory_set.iter().cloned().collect())
            .unwrap_or_default();
        bases.push(self.root.clone());
        bases.push(python_dirname(&python_join(&self.root, &place.file)));
        for base in &bases {
            let candidate = python_normpath(&python_join(base, value));
            if let Some(relative) = self.inside_root(&candidate) {
                if is_file(&candidate) {
                    return TargetStatus::Inside(relative);
                }
            }
        }
        TargetStatus::Missing
    }

    /// 绝对路径 → 从根起的路径（在根里面时）。
    fn inside_root(&self, absolute: &str) -> Option<String> {
        let root = python_normpath(&self.root);
        let candidate = python_normpath(absolute);
        if candidate == root {
            return Some(".".to_string());
        }
        candidate
            .strip_prefix(&format!("{root}/"))
            .map(str::to_string)
    }

    /// cp 的源词 → 仓里的文件；有一个源折不出或不在根里 → `None`。
    fn copied_sources(&self, sources: &[String], place: &UnitPlace) -> Option<Vec<String>> {
        let mut files = BTreeSet::new();
        for source in sources {
            for value in self.fold_word(source, place, &BTreeSet::new())? {
                match self.target_of(&value, place, None) {
                    TargetStatus::Inside(file) => {
                        files.insert(file);
                    }
                    TargetStatus::Outside | TargetStatus::Missing => return None,
                }
            }
        }
        Some(files.into_iter().collect())
    }

    /// 「$目录/<字面尾巴>」→ 根里同一相对路径那一份（`tree_copy_file`）；「$变量」→ 它每一处赋值各自折得出或各自是这种写法时的并。
    fn tree_copy_targets(
        &self,
        unquoted: &str,
        place: &UnitPlace,
        depth: usize,
    ) -> Option<BTreeSet<String>> {
        if depth > TREE_COPY_DEPTH_LIMIT {
            return None;
        }
        if let Some(tail) = tree_prefix_tail(unquoted) {
            return self
                .tree_copy_file(&tail)
                .map(|file| BTreeSet::from([file]));
        }
        let name = bare_variable_name(unquoted)?;
        let words = self.assignments_of(&name, place);
        if words.is_empty() {
            return None;
        }
        let mut files = BTreeSet::new();
        for word in &words {
            match self.fold_word(word, place, &BTreeSet::new()) {
                Some(values) => {
                    for value in values {
                        match self.target_of(&value, place, None) {
                            TargetStatus::Inside(file) => {
                                files.insert(file);
                            }
                            TargetStatus::Outside | TargetStatus::Missing => return None,
                        }
                    }
                }
                None => {
                    files.extend(self.tree_copy_targets(
                        &word.replace('"', ""),
                        place,
                        depth + 1,
                    )?);
                }
            }
        }
        Some(files)
    }

    /// 「<折不出的目录>/<字面尾巴>」→ 尾巴里最长的、至少两段、在根里存在的那一截（那边的 `tree_copy_file`）。
    fn tree_copy_file(&self, tail: &str) -> Option<String> {
        let tail = tail.trim_start_matches('/');
        if !is_tree_copy_tail(tail) {
            return None;
        }
        let parts: Vec<&str> = tail.split('/').collect();
        if parts.iter().any(|part| *part == "." || *part == "..") {
            return None;
        }
        (0..parts.len() - 1)
            .map(|start| parts[start..].join("/"))
            .find(|candidate| is_file(&python_join(&self.root, candidate)))
    }

    /// 拒算落在哪一行（那边的 `word_line`）：先找这个词与 `source` / `.` 同在的一行，找不到再退回头一处提到这个词的行；
    /// 同一个单元里同一个词 source 了几次，`after` 是上一次落的行，先往后找。
    fn word_line(shell_file: &ShellFile, unit: &ShellUnit, word: &str, after: u32) -> u32 {
        let row_count = line_number(shell_file.rows.len());
        let (first, last) = match unit.kind {
            ShellUnitKind::Function => (unit.first_line, unit.last_line),
            ShellUnitKind::ScriptBody => (1, row_count),
        };
        let mut ranges = vec![(first, last), (first, row_count)];
        if after != 0 && first <= after && after < last {
            ranges.insert(0, (after + 1, last));
        }
        let stripped = word.trim_matches('"');
        let names = variable_references(word);
        let mut needles = vec![stripped.to_string()];
        needles.extend(names.iter().map(|name| format!("${name}")));
        needles.extend(names.iter().map(|name| format!("${{{name}")));
        let tail = stripped.rsplit('/').next().unwrap_or("");
        if !tail.is_empty() && !tail.contains('$') {
            needles.push(tail.to_string());
        }
        for requires_the_loader in [true, false] {
            for (low, high) in &ranges {
                for needle in &needles {
                    for number in *low..=*high {
                        let Some(row) = row_of_line(&shell_file.rows, number) else {
                            continue;
                        };
                        if !row.contains(needle.as_str()) || python_left_strip(row).starts_with('#')
                        {
                            continue;
                        }
                        if !requires_the_loader || source_loader_precedes(row, needle) {
                            return number;
                        }
                    }
                }
            }
        }
        unit.first_line
    }

    /// 一个单元里每一处 source 认出来的文件与拒算（那边 `links(unit, kinds=("source",))`）。
    fn source_links(&self, place: &UnitPlace) -> Vec<SourceLink> {
        let shell_file = &self.files[&place.file];
        let unit = &shell_file.units[place.position];
        let facts = self.facts_of(place);
        // 写出来、拷出来的脚本：这个单元与套着它的外层函数里写的都算（嵌套函数用的是外层的局部变量）
        let mut generated: HashMap<String, GeneratedScript> = HashMap::new();
        let mut container = Some(place.position);
        while let Some(position) = container {
            let container_facts = self.facts_of(&UnitPlace {
                file: place.file.clone(),
                position,
            });
            for (target, written) in &container_facts.generated {
                generated
                    .entry(target.clone())
                    .or_insert_with(|| written.clone());
            }
            container = shell_file.units[position].container;
        }
        let mut work: VecDeque<(String, Option<String>, Option<u32>)> = facts
            .source_invocations
            .iter()
            .map(|invocation| (invocation.word.clone(), invocation.directory.clone(), None))
            .collect();
        let mut placed: HashMap<String, u32> = HashMap::new();
        let mut expanded: HashSet<String> = HashSet::new();
        let mut links = Vec::new();
        let refusal = |form: RefusalForm, line: u32, word: &str| -> SourceLink {
            let snippet = match row_of_line(&shell_file.rows, line) {
                Some(row) => snippet_of(row),
                None => snippet_of(word),
            };
            SourceLink {
                files: Vec::new(),
                refusal: Some(ShellRefusal {
                    file: place.file.clone(),
                    line,
                    form,
                    snippet,
                }),
            }
        };
        while let Some((word, directory, generator_line)) = work.pop_front() {
            if let Some(line) = generator_line {
                if literal_word(&word).is_none() {
                    // 写出来的脚本正文里带展开的路径：那是写出来的脚本自己运行时的变量，这个单元的赋值替它作不了保
                    links.push(refusal(RefusalForm::ShellPathUnfolded, line, &word));
                    continue;
                }
            }
            let no_progress = BTreeSet::new();
            let mut values = self.fold_word(&word, place, &no_progress);
            let directories: Option<BTreeSet<String>> = directory
                .as_ref()
                .and_then(|directory_word| self.fold_word(directory_word, place, &no_progress));
            if values.is_some()
                && directory.is_some()
                && directories.is_none()
                && values.iter().flatten().any(|value| !value.starts_with('/'))
            {
                values = None;
            }
            let line = match generator_line {
                Some(line) => line,
                None => {
                    let after = placed.get(&word).copied().unwrap_or(0);
                    let line = Self::word_line(shell_file, unit, &word, after);
                    placed.insert(word.clone(), line);
                    line
                }
            };
            let Some(values) = values else {
                let unquoted = word.replace(['"', '\''], "");
                match generated.get(&unquoted) {
                    Some(GeneratedScript::Copied(sources)) => {
                        // 这个单元自己 cp 出来的脚本：代码就是源那一份，源折得出就连过去
                        if let Some(files) = self.copied_sources(sources, place) {
                            if !files.is_empty() {
                                links.push(SourceLink {
                                    files,
                                    refusal: None,
                                });
                            }
                            continue;
                        }
                    }
                    Some(GeneratedScript::Written(bodies)) => {
                        // 这个单元自己写出来的脚本：正文里的 source 接着认，落在写它的这一行
                        if expanded.insert(unquoted.clone()) {
                            for body in bodies {
                                let inner = self.text_facts(body);
                                work.extend(inner.source_invocations.into_iter().map(
                                    |invocation| {
                                        (invocation.word, invocation.directory, Some(line))
                                    },
                                ));
                            }
                        }
                        continue;
                    }
                    None => {}
                }
                if let Some(copied) = self
                    .tree_copy_targets(&unquoted, place, 0)
                    .filter(|copied| !copied.is_empty())
                {
                    links.push(SourceLink {
                        files: copied.into_iter().collect(),
                        refusal: None,
                    });
                    continue;
                }
                links.push(refusal(RefusalForm::ShellPathUnfolded, line, &word));
                continue;
            };
            let mut files = Vec::new();
            for value in &values {
                // 前面 cd 过去的目录折出几个，相对路径就在每一个底下各找一次（运行期是其中哪一个都可能）
                let tries: Vec<Option<BTreeSet<String>>> = match &directories {
                    Some(directories) if !directories.is_empty() && !value.starts_with('/') => {
                        directories
                            .iter()
                            .map(|directory_value| Some(BTreeSet::from([directory_value.clone()])))
                            .collect()
                    }
                    Some(_) | None => vec![directories.clone()],
                };
                for base in &tries {
                    match self.target_of(value, place, base.as_ref()) {
                        TargetStatus::Inside(file) => files.push(file),
                        TargetStatus::Outside => {
                            links.push(refusal(RefusalForm::ShellPathOutsideRoot, line, &word));
                        }
                        TargetStatus::Missing => {
                            links.push(refusal(RefusalForm::ShellPathMissing, line, &word));
                        }
                    }
                }
            }
            if !files.is_empty() {
                links.push(SourceLink {
                    files,
                    refusal: None,
                });
            }
        }
        links
    }
}

/// `(?:^|[\s;&|({])(?:source|\.)\s+.*<needle>` 在这一行里找得到。
fn source_loader_precedes(row: &str, needle: &str) -> bool {
    let text = characters_of(row);
    let needle_characters = characters_of(needle);
    (0..text.len()).any(|keyword_start| {
        let boundary = keyword_start == 0 || {
            let before = text[keyword_start - 1];
            is_python_whitespace(before) || ";&|({".contains(before)
        };
        boundary
            && ["source", "."].iter().any(|keyword| {
                let after = keyword_start + keyword.chars().count();
                starts_with_at(&text, keyword_start, keyword)
                    && text.get(after).copied().is_some_and(is_python_whitespace)
                    && find_text(&text, &needle_characters, after + 1).is_some()
            })
    })
}

#[cfg(test)]
mod tests {
    use super::{
        characters_of, literal_word, python_dirname, python_normpath, snippet_of, split_commands,
    };

    #[test]
    fn python_path_helpers_agree_with_posixpath_on_the_edge_cases() {
        assert_eq!(python_normpath(""), ".");
        assert_eq!(python_normpath("/a/./b/../c/"), "/a/c");
        assert_eq!(python_normpath("//a//b"), "//a/b");
        assert_eq!(python_normpath("///a"), "/a");
        assert_eq!(python_normpath("../a/.."), "..");
        // 拼出来，不写成整段字面量：以两级上跳起头的字面量会被 crates 变异表的静态核当成逃出 crates/ 的路径
        let two_levels_up_then_a = concat!("..", "/../a");
        assert_eq!(python_normpath(two_levels_up_then_a), two_levels_up_then_a);
        assert_eq!(python_normpath("/.."), "/");
        assert_eq!(python_dirname("a"), "");
        assert_eq!(python_dirname("/a"), "/");
        assert_eq!(python_dirname("a/b//c"), "a/b");
        assert_eq!(python_dirname("//a"), "//");
    }

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

    #[test]
    fn snippets_fold_whitespace_runs_and_cut_at_one_hundred_sixty_characters() {
        assert_eq!(snippet_of("  a\t\tb  c "), "a b c");
        assert_eq!(snippet_of(&"字".repeat(200)).chars().count(), 160);
    }
}
