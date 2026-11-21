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

mod bash_normalization_and_shell_units;
mod characters_and_python_string_operations;
mod normalized_text_lexing;
mod unit_facts_and_folding;

use bash_normalization_and_shell_units::*;
use characters_and_python_string_operations::*;
use normalized_text_lexing::*;
use unit_facts_and_folding::*;

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
