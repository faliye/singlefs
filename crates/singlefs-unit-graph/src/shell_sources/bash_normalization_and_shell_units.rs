//! bash 规范化（`bash -n` 过了才包进一个函数、`declare -f` 打出来，`normal_forms`）与在规范文本上切出
//! 函数单元与脚本体（`ShellFile`），连同单元在原文里的起止行。
use super::*;

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
pub(super) fn normal_forms(
    contents: &[Vec<u8>],
    bash_program: &OsStr,
) -> Result<Vec<Option<String>>, String> {
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
pub(super) enum ShellUnitKind {
    Function,
    ScriptBody,
}

/// 一个 shell 单元：函数（套着的函数各一个，`container` 指外层函数在同一份文件的单元表里的位置）或脚本体。
/// 行号是原文里的（函数按头那一行与 `shell_block_end` 找到的收尾行，只当 `word_line` 的范围用；脚本体是整份）。
#[derive(Clone, Debug)]
pub(super) struct ShellUnit {
    pub(super) kind: ShellUnitKind,
    qualified_name: String,
    pub(super) first_line: u32,
    pub(super) last_line: u32,
    pub(super) body: String,
    pub(super) container: Option<usize>,
}

pub(super) struct ShellFile {
    pub(super) rows: Vec<String>,
    pub(super) refusals: Vec<ShellRefusal>,
    pub(super) units: Vec<ShellUnit>,
    pub(super) functions_by_name: HashMap<String, Vec<usize>>,
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

pub(super) fn line_number(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

/// 原文第 `line` 行（从 1 起）；0 与超出末行的是 `None`。
pub(super) fn row_of_line(rows: &[String], line: u32) -> Option<&String> {
    let position = usize::try_from(line).ok()?.checked_sub(1)?;
    rows.get(position)
}

impl ShellFile {
    pub(super) fn new(path: &str, content: &[u8], normal: Option<&str>) -> Self {
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
