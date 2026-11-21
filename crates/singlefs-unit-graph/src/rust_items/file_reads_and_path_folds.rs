//! 单元读的文件与拒算：`include_str!` 一族、`env!`、按 `CARGO_MANIFEST_DIR` 拼的路径的折法，与一个单元的全部读与拒算。
use super::*;

/// `env!` / `option_env!` 读的变量里 cargo 自己给的那一类的前缀（`CARGO_MANIFEST_DIR`、`CARGO_BIN_EXE_<名>`、`CARGO_PKG_*` 这类）。
const CARGO_SUPPLIED_ENVIRONMENT_VARIABLE_PREFIX: &str = "CARGO";
/// cargo 给构建脚本输出目录的变量。
const BUILD_SCRIPT_OUTPUT_DIRECTORY_VARIABLE: &str = "OUT_DIR";

/// 清单目录拼出的目录绑到名字、名字的用法再绑到名字，最多跟这么多层（再深的拒算）。
const DEEPEST_DIRECTORY_BINDING_CHAIN: usize = 4;

/// 仓根下收得成目录数据单元的那一层目录的名字：折出的目录落在 `<仓根>/crates/` 之下（不是它本身）才收
/// （与 `scripts/gate/lib_function_units.py` 的 `is_data_directory` 认的 `crates/` 是同一层）。
const DATA_DIRECTORIES_PARENT_UNDER_THE_REPOSITORY_ROOT: &str = "crates";

/// `include_*!` 的参数是哪一种允许的形态。
enum IncludeArgument {
    RelativeToFile(String),
    RelativeToManifest(String),
    BuildScriptOutput,
    NotAllowed,
}

/// 运行期按清单目录拼路径读，折出来的一样（拼出的目录绑到一个名字上时，那个名字的每一处用法各折一样）。
enum RuntimeFold {
    File(PathBuf),
    /// `<仓根>/crates/` 之下（不是 `crates/` 本身）的一个目录：整个目录当一个数据单元。
    Directory(PathBuf),
    Missing(PathBuf),
    NotFoldable,
}

/// `env!` / `option_env!` 读的这个变量的值只能由本 crate 构建脚本经 `cargo::rustc-env` 交进来：不空、不以 `CARGO` 起头、也不是 `OUT_DIR`
/// （`SourceRead::BuildScriptEnvironmentValue`；与 `scripts/gate/lib_function_units.py` 的 `build-environment` 那一支同一个判法）。
fn is_handed_over_by_a_build_script(variable: &str) -> bool {
    !variable.is_empty()
        && !variable.starts_with(CARGO_SUPPLIED_ENVIRONMENT_VARIABLE_PREFIX)
        && variable != BUILD_SCRIPT_OUTPUT_DIRECTORY_VARIABLE
}

/// 一份文件里 `const 名字: &str = env!("CARGO_MANIFEST_DIR");` 这一形的私有常量（`pub` 的不收：别的文件里的用法这里看不见）。
/// 文件里哪个串字面量带 `{名字` 的格式捕获也不收：捕获里的用法不是词法单元，跟不到。
pub(super) fn private_manifest_directory_constants(
    source: &str,
    tokens: &[Token],
) -> BTreeSet<String> {
    let text_at = |index: usize| {
        tokens
            .get(index)
            .map_or("", |token| token_text(source, token))
    };
    let mut constants = BTreeSet::new();
    for index in 0..tokens.len() {
        if text_at(index) != "const"
            || tokens[index + 1..].first().map(|token| token.kind) != Some(TokenKind::Identifier)
        {
            continue;
        }
        let is_public = index > 0 && matches!(text_at(index - 1), "pub" | ")");
        let Some(equals) = (index + 2..tokens.len().min(index + 12))
            .find(|position| matches!(text_at(*position), "=" | ";"))
        else {
            continue;
        };
        let is_manifest_directory = text_at(equals) == "="
            && text_at(equals + 1) == "env"
            && text_at(equals + 2) == "!"
            && text_at(equals + 3) == "("
            && plain_string_content(text_at(equals + 4)) == Some("CARGO_MANIFEST_DIR")
            && text_at(equals + 5) == ")"
            && text_at(equals + 6) == ";";
        if is_manifest_directory && !is_public {
            constants.insert(identifier_name(text_at(index + 1)).to_string());
        }
    }
    constants.retain(|name| {
        let capture = format!("{{{name}");
        !tokens.iter().any(|token| {
            token.kind == TokenKind::Literal && token_text(source, token).contains(&capture)
        })
    });
    constants
}

impl<'scan> RustScanner<'scan> {
    fn include_argument(&self, low: usize, high: usize) -> IncludeArgument {
        let texts: Vec<&str> = (low..high).map(|index| self.text_of(index)).collect();
        if let [literal] = texts.as_slice() {
            return plain_string_content(literal).map_or(IncludeArgument::NotAllowed, |content| {
                IncludeArgument::RelativeToFile(content.to_string())
            });
        }
        let ["concat", "!", "(", inside @ .., ")"] = texts.as_slice() else {
            return IncludeArgument::NotAllowed;
        };
        if self.close_of(low + 2) + 1 != high {
            return IncludeArgument::NotAllowed;
        }
        let parts = split_at_top_level_commas(inside);
        let mut concatenated = String::new();
        let mut environment_start = None;
        for (part_position, part) in parts.iter().enumerate() {
            match part.as_slice() {
                [literal] => match plain_string_content(literal) {
                    Some(content) => concatenated.push_str(content),
                    None => return IncludeArgument::NotAllowed,
                },
                ["env", "!", "(", literal, ")"] if part_position == 0 => {
                    environment_start = plain_string_content(literal);
                }
                _ => return IncludeArgument::NotAllowed,
            }
        }
        match environment_start {
            None => IncludeArgument::RelativeToFile(concatenated),
            Some("CARGO_MANIFEST_DIR") => IncludeArgument::RelativeToManifest(concatenated),
            Some("OUT_DIR") => IncludeArgument::BuildScriptOutput,
            Some(_) => IncludeArgument::NotAllowed,
        }
    }

    fn file_read(&self, absolute_path: PathBuf, line: u32, origin: ReadOrigin) -> SourceRead {
        let digest = std::fs::read(&absolute_path)
            .ok()
            .map(|bytes| sha256_of(&bytes));
        SourceRead::File {
            absolute_path,
            line,
            origin,
            digest,
        }
    }

    fn manifest_relative_path(&self, suffix: &str) -> Option<PathBuf> {
        let manifest = self.manifest_directory?;
        let joined = format!("{}{suffix}", manifest.display());
        Some(normalized_path(Path::new(&joined)))
    }

    pub(super) fn text_before(&self, index: usize) -> &'scan str {
        index
            .checked_sub(1)
            .map_or("", |previous| self.text_of(previous))
    }

    /// 折出的目录落在 `<仓根>/crates/` 之下、又不是它本身：目录数据单元（与 `scripts/gate/lib_function_units.py` 的 `is_data_directory`
    /// 同一条）。仓根、根外、`crates/` 整个与 `crates/` 之外的目录拒算；清单目录的上一级不一定是 `crates/`（仓根下一层的成员、
    /// 仓根自己那个包），所以按仓根认，不按清单目录认。
    fn directory_fold(&self, folded: PathBuf) -> RuntimeFold {
        let crates_directory = self
            .repository_root
            .join(DATA_DIRECTORIES_PARENT_UNDER_THE_REPOSITORY_ROOT);
        if folded.starts_with(&crates_directory) && folded != crates_directory {
            RuntimeFold::Directory(folded)
        } else {
            RuntimeFold::NotFoldable
        }
    }

    /// `std::path::Path::new` 这类写法起头的那个词法单元：从 `new` / `from` 往回跨过 `名字 ::`。
    pub(super) fn path_expression_start(&self, constructor_index: usize) -> usize {
        let mut start = constructor_index;
        while start >= 3
            && self.text_of(start - 1) == ":"
            && self.text_of(start - 2) == ":"
            && self.is_identifier(start - 3)
        {
            start -= 3;
        }
        start
    }

    /// 从 `expression_start` 起、到 `after_expression` 之前的表达式整个是 `let [mut] 名字 [: 类型] = …;` 的右边时，交那个名字。
    pub(super) fn let_binding_name(
        &self,
        expression_start: usize,
        after_expression: usize,
    ) -> Option<String> {
        if self.text_of(after_expression) != ";" || self.text_before(expression_start) != "=" {
            return None;
        }
        let equals = expression_start - 1;
        let let_index = (0..equals)
            .rev()
            .find(|position| matches!(self.text_of(*position), "let" | ";" | "{" | "}" | "="))?;
        if self.text_of(let_index) != "let" {
            return None;
        }
        let mut name_index = let_index + 1;
        if self.text_of(name_index) == "mut" {
            name_index += 1;
        }
        if !self.is_identifier(name_index) {
            return None;
        }
        let is_plain = name_index + 1 == equals;
        let is_typed = self.text_of(name_index + 1) == ":" && self.text_of(name_index + 2) != ":";
        (is_plain || is_typed).then(|| self.name_at(name_index))
    }

    /// 名字在 `from..end` 里的每一处用法（`.名字` 的字段或方法、`名字::` 的路径段不算）；碰到再 `let` 这个名字就停（之后是另一个值）。
    pub(super) fn uses_of_the_binding(&self, name: &str, from: usize, end: usize) -> Vec<usize> {
        let mut uses = Vec::new();
        for index in from..end {
            if !self.is_identifier(index) || self.name_at(index) != name {
                continue;
            }
            let previous = self.text_before(index);
            if previous == "let" || (previous == "mut" && self.text_before(index - 1) == "let") {
                break;
            }
            if previous == "." || (self.text_of(index + 1) == ":" && self.text_of(index + 2) == ":")
            {
                continue;
            }
            uses.push(index);
        }
        uses
    }

    /// 从 `chain_start` 起接字面 `.join("…")` 链，拼在 `base` 后面。折出的是目录、表达式又整个是一条 `let 名字 = …;` 的右边时，
    /// 交那个名字每一处用法折出的（[`Self::fold_use_of_a_path`]；名字后面再没用到的交目录本身）：名字之后接了 `.parent()`、
    /// `.join(变量)` 这类的，那一处拒算，读整棵仓的写法照旧拒。
    fn fold_path_chain(
        &self,
        base: &Path,
        chain_start: usize,
        expression_start: usize,
        unit_end: usize,
        binding_depth: usize,
    ) -> Vec<RuntimeFold> {
        let mut path = base.to_path_buf();
        let mut position = chain_start;
        while self.text_of(position) == "."
            && self.text_of(position + 1) == "join"
            && self.text_of(position + 2) == "("
            && self.text_of(position + 4) == ")"
        {
            let Some(content) = plain_string_content(self.text_of(position + 3)) else {
                break;
            };
            path.push(content);
            position += 5;
        }
        // 字面 `.join(…)` 链之后还接任何方法（`.canonicalize()`、`.parent()`、`.join(变量)`、`.read_dir()` 这类）：
        // 拼出的不再是交给读文件的那条字面路径，拒算（与 `scripts/gate/lib_function_units.py` 的 `runtime_manifest_path` 同一条）
        if self.text_of(position) == "." {
            return vec![RuntimeFold::NotFoldable];
        }
        let folded = normalized_path(&path);
        if folded.is_file() {
            return vec![RuntimeFold::File(folded)];
        }
        if !folded.is_dir() {
            return vec![RuntimeFold::Missing(folded)];
        }
        let Some(binding) = self.let_binding_name(expression_start, position) else {
            return vec![self.directory_fold(folded)];
        };
        if binding_depth >= DEEPEST_DIRECTORY_BINDING_CHAIN {
            return vec![RuntimeFold::NotFoldable];
        }
        let uses = self.uses_of_the_binding(&binding, position + 1, unit_end);
        if uses.is_empty() {
            return vec![self.directory_fold(folded)];
        }
        uses.into_iter()
            .flat_map(|use_index| {
                self.fold_use_of_a_path(&folded, use_index, unit_end, binding_depth + 1)
            })
            .collect()
    }

    fn is_wrapped_in_a_path_constructor(&self, use_index: usize) -> bool {
        use_index >= 2
            && self.text_of(use_index - 1) == "("
            && matches!(self.text_of(use_index - 2), "new" | "from")
            && self.text_of(use_index + 1) == ")"
    }

    /// 绑着一个路径的名字的一处用法：`Path::new(名字)` / `PathBuf::from(名字)` 之后、或名字本身之后接的字面 `.join` 链；
    /// 名字单独交给别的函数（`strip_prefix(名字)`、`扫目录(名字)`）的，折出的就是它绑着的目录。
    fn fold_use_of_a_path(
        &self,
        value: &Path,
        use_index: usize,
        unit_end: usize,
        binding_depth: usize,
    ) -> Vec<RuntimeFold> {
        if self.is_wrapped_in_a_path_constructor(use_index) {
            return self.fold_path_chain(
                value,
                use_index + 2,
                self.path_expression_start(use_index - 2),
                unit_end,
                binding_depth,
            );
        }
        self.fold_path_chain(value, use_index + 1, use_index, unit_end, binding_depth)
    }

    /// 私有清单目录常量的一处用法：只认 `Path::new(常量)` / `PathBuf::from(常量)`（串拼进 `format!` 这类的拒算）。
    fn fold_manifest_directory_constant_use(
        &self,
        use_index: usize,
        unit_end: usize,
    ) -> Vec<RuntimeFold> {
        match self.manifest_directory {
            Some(manifest) if self.is_wrapped_in_a_path_constructor(use_index) => {
                self.fold_use_of_a_path(manifest, use_index, unit_end, 0)
            }
            Some(_) | None => vec![RuntimeFold::NotFoldable],
        }
    }

    /// `const 名字: &str = env!("CARGO_MANIFEST_DIR");` 里的那个 `env!`，而且名字是这份文件收下的私有清单目录常量。
    fn is_manifest_directory_constant_definition(
        &self,
        environment_index: usize,
        environment_close: usize,
    ) -> bool {
        if self.text_of(environment_close + 1) != ";" || self.text_before(environment_index) != "="
        {
            return false;
        }
        (environment_index.saturating_sub(12)..environment_index)
            .rev()
            .find(|position| self.text_of(*position) == "const")
            .is_some_and(|const_index| {
                self.manifest_directory_constants
                    .contains(&self.name_at(const_index + 1))
            })
    }

    fn fold_manifest_directory_read(
        &self,
        environment_index: usize,
        environment_close: usize,
        unit_end: usize,
    ) -> Vec<RuntimeFold> {
        let Some(manifest) = self.manifest_directory else {
            return vec![RuntimeFold::NotFoldable];
        };
        if environment_index >= 3
            && self.text_of(environment_index - 1) == "("
            && self.text_of(environment_index - 2) == "!"
            && self.text_of(environment_index - 3) == "concat"
        {
            let concat_close = self.close_of(environment_index - 1);
            if self.text_of(environment_close + 1) != "," && environment_close + 1 != concat_close {
                return vec![RuntimeFold::NotFoldable];
            }
            let texts: Vec<&str> = (environment_close + 2..concat_close)
                .map(|index| self.text_of(index))
                .collect();
            let mut suffix = String::new();
            for part in split_at_top_level_commas(&texts) {
                match part.as_slice() {
                    [literal] => match plain_string_content(literal) {
                        Some(content) => suffix.push_str(content),
                        None => return vec![RuntimeFold::NotFoldable],
                    },
                    _ => return vec![RuntimeFold::NotFoldable],
                }
            }
            let Some(folded) = self.manifest_relative_path(&suffix) else {
                return vec![RuntimeFold::NotFoldable];
            };
            // `concat!` 拼出的是串：绑到名字上的不跟（用法里的 `Path::new(名字).parent()` 看不见），只认就地交出去的那一处
            return if folded.is_file() {
                vec![RuntimeFold::File(folded)]
            } else if !folded.is_dir() {
                vec![RuntimeFold::Missing(folded)]
            } else if self
                .let_binding_name(environment_index - 3, concat_close + 1)
                .is_some()
            {
                vec![RuntimeFold::NotFoldable]
            } else {
                vec![self.directory_fold(folded)]
            };
        }
        if environment_index >= 2
            && self.text_of(environment_index - 1) == "("
            && matches!(self.text_of(environment_index - 2), "new" | "from")
            && self.close_of(environment_index - 1) == environment_close + 1
        {
            return self.fold_path_chain(
                manifest,
                environment_close + 2,
                self.path_expression_start(environment_index - 2),
                unit_end,
                0,
            );
        }
        if self.is_manifest_directory_constant_definition(environment_index, environment_close) {
            // 读在常量的用法那几处折（`fold_manifest_directory_constant_use`）
            return Vec::new();
        }
        vec![RuntimeFold::NotFoldable]
    }

    /// 一样折出的东西记成读或拒算；拒算落在起折的那个词法单元（`env!` 或常量名）上。
    fn record_runtime_fold(
        &self,
        fold: RuntimeFold,
        origin_index: usize,
        reads: &mut Vec<SourceRead>,
        refusals: &mut Vec<Refusal>,
    ) {
        let line = self.line_of(origin_index);
        let read = match fold {
            RuntimeFold::File(path) => self.file_read(path, line, ReadOrigin::Runtime),
            RuntimeFold::Directory(path) => match directory_digest(&path) {
                Ok(digest) => SourceRead::Directory {
                    absolute_path: path,
                    line,
                    digest,
                },
                Err(unreadable) => {
                    refusals.push(self.refusal(
                        RefusalForm::DataFileUnreadable,
                        origin_index,
                        cleaned_fragment(&unreadable.display().to_string()),
                    ));
                    return;
                }
            },
            RuntimeFold::Missing(path) => {
                refusals.push(self.refusal(
                    RefusalForm::DataFileUnreadable,
                    origin_index,
                    cleaned_fragment(&path.display().to_string()),
                ));
                return;
            }
            RuntimeFold::NotFoldable => {
                refusals.push(self.refusal(
                    RefusalForm::RuntimeReadWithoutLiteralPath,
                    origin_index,
                    self.fragment(origin_index.saturating_sub(4), origin_index + 9),
                ));
                return;
            }
        };
        if !reads.contains(&read) {
            reads.push(read);
        }
    }

    pub(super) fn reads_and_refusals_of(
        &self,
        first: usize,
        end: usize,
    ) -> (Vec<SourceRead>, Vec<Refusal>) {
        let mut reads = Vec::new();
        let mut refusals = Vec::new();
        if !self.switches.is_intact(BrokenRule::DataUnits) {
            return (reads, refusals);
        }
        let mut consumed_until = first;
        // 最近一处 `concat!(…)` 的闭括号：在它之前的 `env!` 不当构建脚本交的值（与 python 那一份的跳法相同）
        let mut concatenation_close: Option<usize> = None;
        for index in first..end {
            if index < consumed_until || !self.is_identifier(index) {
                continue;
            }
            match self.text_of(index) {
                "include_str" | "include_bytes" | "include"
                    if self.text_of(index + 1) == "!" && self.text_of(index + 2) == "(" =>
                {
                    let close = self.close_of(index + 2);
                    consumed_until = close + 1;
                    let line = self.line_of(index);
                    match self.include_argument(index + 3, close) {
                        IncludeArgument::RelativeToFile(relative) => reads.push(self.file_read(
                            normalized_path(&self.file_directory.join(relative)),
                            line,
                            ReadOrigin::Included,
                        )),
                        IncludeArgument::RelativeToManifest(suffix) => {
                            match self.manifest_relative_path(&suffix) {
                                Some(path) => {
                                    reads.push(self.file_read(path, line, ReadOrigin::Included))
                                }
                                None => refusals.push(self.refusal(
                                    RefusalForm::IncludeArgumentNotInAllowedForms,
                                    index,
                                    self.fragment(index, close + 1),
                                )),
                            }
                        }
                        IncludeArgument::BuildScriptOutput => {
                            reads.push(SourceRead::BuildScriptOutput { line })
                        }
                        IncludeArgument::NotAllowed => refusals.push(self.refusal(
                            RefusalForm::IncludeArgumentNotInAllowedForms,
                            index,
                            self.fragment(index, close + 1),
                        )),
                    }
                }
                "concat" if self.text_of(index + 1) == "!" && self.is_open_bracket(index + 2) => {
                    concatenation_close = Some(self.close_of(index + 2));
                }
                "env" | "option_env"
                    if self.text_of(index + 1) == "!" && self.is_open_bracket(index + 2) =>
                {
                    let close = self.close_of(index + 2);
                    let Some(variable) = plain_string_content(self.text_of(index + 3)) else {
                        continue;
                    };
                    // python 那一份读到 `concat!(…)` 整段跳过（runtime_concat），里面的 `env!` 不当构建脚本交的值
                    let is_inside_a_concatenation = concatenation_close
                        .is_some_and(|concatenation_end| index < concatenation_end);
                    if !is_inside_a_concatenation && is_handed_over_by_a_build_script(variable) {
                        reads.push(SourceRead::BuildScriptEnvironmentValue {
                            variable: variable.to_string(),
                            line: self.line_of(index),
                        });
                        continue;
                    }
                    if self.text_of(index) != "env"
                        || self.text_of(index + 2) != "("
                        || close != index + 4
                    {
                        continue;
                    }
                    if variable == "CARGO_MANIFEST_DIR" {
                        for fold in self.fold_manifest_directory_read(index, close, end) {
                            self.record_runtime_fold(fold, index, &mut reads, &mut refusals);
                        }
                    } else if let Some(binary_name) = variable.strip_prefix("CARGO_BIN_EXE_") {
                        reads.push(SourceRead::BinaryExecutable {
                            binary_name: binary_name.to_string(),
                            line: self.line_of(index),
                        });
                    }
                }
                "var" | "var_os"
                    if self.text_of(index + 1) == "("
                        && plain_string_content(self.text_of(index + 2))
                            == Some("CARGO_MANIFEST_DIR") =>
                {
                    refusals.push(self.refusal(
                        RefusalForm::RuntimeReadWithoutLiteralPath,
                        index,
                        self.fragment(index.saturating_sub(4), index + 8),
                    ));
                }
                "current_exe" if self.text_of(index + 1) == "(" => {
                    let fragment = self.fragment(index.saturating_sub(4), index + 4);
                    match self.child_test_name_at(index, first, end) {
                        Some(test_name) => reads.push(SourceRead::ChildTestProcess {
                            line: self.line_of(index),
                            test_name,
                            fragment,
                        }),
                        // 路径一个字不改地交给 `Command::new`：起的是这个程序自己，拒不拒算看入口（闭包的 `finish`）
                        None if self
                            .own_program_path_is_handed_to_command_unchanged(index, end) =>
                        {
                            reads.push(SourceRead::OwnProgramRestart {
                                line: self.line_of(index),
                                fragment,
                            })
                        }
                        // 同一个单元里起了子进程、又折不出点名的那一条测试：子进程跑什么说不清，拒算
                        None if self.starts_a_process(first, end) => refusals.push(self.refusal(
                            RefusalForm::CurrentExecutablePath,
                            index,
                            fragment,
                        )),
                        // 不起子进程、只问路径（读程序文件的大小、`current_exe().is_ok()`）：路径拿去做什么与要点名的测试无关，
                        // 不拒算、不连边（与 `scripts/gate/lib_function_units.py` 文件头「测试二进制起自己」同一判法）
                        None => {}
                    }
                }
                constant
                    if self
                        .manifest_directory_constants
                        .contains(identifier_name(constant))
                        && self.text_before(index) != "const" =>
                {
                    for fold in self.fold_manifest_directory_constant_use(index, end) {
                        self.record_runtime_fold(fold, index, &mut reads, &mut refusals);
                    }
                }
                "concat_idents" | "paste" if self.text_of(index + 1) == "!" => {
                    refusals.push(self.refusal(
                        RefusalForm::IdentifierPastingMacro,
                        index,
                        self.fragment(index, index + 8),
                    ))
                }
                _ => {}
            }
        }
        (reads, refusals)
    }
}
