//! 单元的事实与折叠：每个单元的命令、赋值、`source` 写法与 `cd` 过去的目录（`ShellFacts`），路径词的折法，
//! 装进来的文件、source 边与每处 `source` 连到的文件或拒算（`ShellIndex`）。
use super::*;

// ── 单元的事实与折叠 ───────────────────────────────────────────────────────

/// 一份文件里的一个单元（文件从仓根起，`position` 是它在那份文件单元表里的位置）。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct UnitPlace {
    pub(super) file: String,
    pub(super) position: usize,
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
pub(super) struct SourceLink {
    pub(super) files: Vec<String>,
    pub(super) refusal: Option<ShellRefusal>,
}

/// 那边的 `ShellIndex`：装进来的 shell 文件、各单元的事实、source 边与命令替换里 `cat <<词` 的输出。
pub(super) struct ShellIndex {
    root: String,
    /// 规范化起的 bash（`DEFAULT_BASH_PROGRAM`，测试换成起不来的程序）。
    bash_program: OsString,
    file_order: Vec<String>,
    pub(super) files: HashMap<String, ShellFile>,
    facts: HashMap<UnitPlace, ShellFacts>,
    sources_of: HashMap<String, BTreeSet<String>>,
    sourcers_of: HashMap<String, BTreeSet<String>>,
    substitution_outputs: RefCell<HashMap<String, BTreeSet<HeredocBody>>>,
}

impl ShellIndex {
    pub(super) fn new(root: String, bash_program: OsString) -> Self {
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
    pub(super) fn add(&mut self, paths: &[String]) -> Result<(), String> {
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
    pub(super) fn source_links(&self, place: &UnitPlace) -> Vec<SourceLink> {
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
