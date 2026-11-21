//! 收集：WGSL 外名的着落，与把供名字的单元、读进定义它的 `.wgsl` 的单元并进闭包。
use super::*;

fn add_string_words(literal: &str, words: &mut HashSet<String>) {
    let mut current = String::new();
    for character in literal.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            current.push(character);
        } else if !current.is_empty() {
            words.insert(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        words.insert(current);
    }
}

/// 外名的首段加 `_` 至少这么多个字符才认（`T_` 认；`_` 起头的名字只剩一个 `_`，照旧要整名落在串字面量里）。
const SHORTEST_WGSL_EXTERNAL_NAME_PREFIX_IN_CHARACTERS: usize = 2;

/// WGSL 外名的首段加 `_`：按第一个 `_` 切，连那个 `_`（`T_HEADER` 取 `T_`，`FAILURE_POINT_NONE` 取 `FAILURE_`）；名字里没有 `_` 交 `None`。
fn wgsl_external_name_prefix(name: &str) -> Option<&str> {
    name.find('_').map(|position| &name[..=position])
}

/// 一个名字与它每个 `_` 之后的尾段（`GPU_FAILURE_POINT_NONE` 交它自己、`FAILURE_POINT_NONE`、`POINT_NONE`、`NONE`）：
/// 外名以 `_` 加它收尾的名字都算供它的名字。
fn name_and_its_underscore_suffixes(name: &str) -> impl Iterator<Item = &str> {
    std::iter::once(name).chain(
        name.match_indices('_')
            .map(move |(position, _)| &name[position + 1..])
            .filter(|suffix| !suffix.is_empty()),
    )
}

/// 按名字定义的名字能供 WGSL 外名的单元种类：常量、静态量与条目级宏调用（宏调用里的标识符都算它可能定义的名字，R-宏）。
fn defines_names_a_kernel_can_use(kind: UnitKind) -> bool {
    match kind {
        UnitKind::Constant | UnitKind::Static | UnitKind::ItemMacroInvocation => true,
        UnitKind::Function
        | UnitKind::ImplementationHead
        | UnitKind::TraitHead
        | UnitKind::Struct
        | UnitKind::Enumeration
        | UnitKind::Union
        | UnitKind::TypeAlias
        | UnitKind::MacroRulesDefinition
        | UnitKind::Use
        | UnitKind::ModuleDeclaration
        | UnitKind::ModuleHead
        | UnitKind::ExternBlock
        | UnitKind::ExternCrate
        | UnitKind::VerusBroadcastGroup
        | UnitKind::VerusAssumeSpecification
        | UnitKind::DocumentationTest => false,
    }
}

/// 闭包里 Rust 单元给 WGSL 外名的着落。
pub(super) struct WgslExternalNameSources {
    string_words: HashSet<String>,
    string_words_in_upper_case: HashSet<String>,
    texts_holding_prefixes: Vec<String>,
    /// 常量、静态量、条目级宏调用按名字定义的名字，连同它们每个 `_` 之后的尾段。
    defined_names_and_their_suffixes: HashSet<String>,
    /// 闭包里出现的名字（定义的与点名的），大写。
    names_in_upper_case: HashSet<String>,
}

impl WgslExternalNameSources {
    /// 这个外名有没有着落：整名是串字面量里的一个词；整名是闭包里定义的名字或它以 `_` 加整名收尾；或者（`accepts_prefixes` 时）
    /// 首段加 `_` 够长、落在串字面量或宏体原文里，且其余那一段不分大小写是串字面量里的一个词或闭包里的一个名字。
    pub(super) fn supply(&self, name: &str, accepts_prefixes: bool) -> bool {
        if self.string_words.contains(name) || self.defined_names_and_their_suffixes.contains(name)
        {
            return true;
        }
        if !accepts_prefixes {
            return false;
        }
        let Some(prefix) = wgsl_external_name_prefix(name) else {
            return false;
        };
        let remainder = name[prefix.len()..].to_ascii_uppercase();
        prefix.chars().count() >= SHORTEST_WGSL_EXTERNAL_NAME_PREFIX_IN_CHARACTERS
            && !remainder.is_empty()
            && self
                .texts_holding_prefixes
                .iter()
                .any(|text| text.contains(prefix))
            && (self.string_words_in_upper_case.contains(&remainder)
                || self.names_in_upper_case.contains(&remainder))
    }
}

impl<'workspace> ClosureRun<'workspace> {
    /// 闭包里 Rust 单元给 WGSL 外名的着落：串字面量里的词（整名要是其中一个），以及串字面量与 `macro_rules!` 体的原文
    /// （外名的首段加 `_` 要落在其中之一里）。
    pub(super) fn wgsl_external_name_sources_of_rust_closure(&self) -> WgslExternalNameSources {
        let mut sources = WgslExternalNameSources {
            string_words: HashSet::new(),
            string_words_in_upper_case: HashSet::new(),
            texts_holding_prefixes: Vec::new(),
            defined_names_and_their_suffixes: HashSet::new(),
            names_in_upper_case: HashSet::new(),
        };
        for unit_index in &self.in_closure {
            let unit = &self.workspace.units[unit_index.0];
            if defines_names_a_kernel_can_use(unit.kind) {
                for declared in &unit.declared_names {
                    sources
                        .defined_names_and_their_suffixes
                        .extend(name_and_its_underscore_suffixes(declared).map(str::to_string));
                }
            }
            sources.names_in_upper_case.extend(
                unit.declared_names
                    .iter()
                    .chain(&unit.references.plain_names)
                    .map(|name| name.to_ascii_uppercase()),
            );
            let Some((first, end)) = unit.token_range else {
                continue;
            };
            let file = &self.workspace.files[unit.file.0];
            let is_macro_body = unit.kind == UnitKind::MacroRulesDefinition;
            for token in &file.tokens[first..end] {
                let text = token_text(&file.source, token);
                let is_string_literal = token.kind == TokenKind::Literal
                    && (text.starts_with('"') || text.starts_with('r'));
                if is_string_literal {
                    add_string_words(text, &mut sources.string_words);
                }
                if is_string_literal || is_macro_body {
                    sources.texts_holding_prefixes.push(text.to_string());
                }
            }
        }
        sources.string_words_in_upper_case = sources
            .string_words
            .iter()
            .map(|word| word.to_ascii_uppercase())
            .collect();
        sources
    }

    /// 闭包读到的 `.wgsl` 组成的模块：从 `@compute` 入口收到的 WGSL 单元与用到而模块里没定义的外名。
    pub(super) fn wgsl_module_and_external_names(
        &self,
    ) -> (BTreeSet<WgslUnitIndex>, Vec<(WgslUnitIndex, String)>) {
        let module_units: Vec<WgslUnitIndex> = self
            .workspace
            .wgsl_units
            .iter()
            .enumerate()
            .filter(|(_, wgsl_unit)| self.wgsl_files.contains(&wgsl_unit.file))
            .map(|(position, _)| WgslUnitIndex(position))
            .collect();
        wgsl_module_closure(&self.workspace.wgsl_units, &module_units)
    }

    /// 链接集里、闭包外，供这几个外名的单元：按名字定义了外名或以 `_` 加外名收尾的常量、静态量、条目级宏调用，串字面量里有外名这个词的单元。
    fn wgsl_name_suppliers_outside_the_closure(
        &self,
        unsupplied_names: &BTreeSet<String>,
    ) -> Vec<UnitIndex> {
        let workspace = self.workspace;
        let mut suppliers = Vec::new();
        for (position, unit) in workspace.units.iter().enumerate() {
            let unit_index = UnitIndex(position);
            if self.in_closure.contains(&unit_index)
                || self.wgsl_name_suppliers_tried.contains(&unit_index)
                || self.linked_target_of(unit_index).is_none()
                || self.is_test_only_code(unit_index)
            {
                continue;
            }
            let defines_a_name = defines_names_a_kernel_can_use(unit.kind)
                && unit.declared_names.iter().any(|declared| {
                    name_and_its_underscore_suffixes(declared)
                        .any(|suffix| unsupplied_names.contains(suffix))
                });
            let names_one_in_a_string = !defines_a_name
                && unit.token_range.is_some_and(|(first, end)| {
                    let file = &workspace.files[unit.file.0];
                    file.tokens[first..end].iter().any(|token| {
                        let text = token_text(&file.source, token);
                        let is_string_literal = token.kind == TokenKind::Literal
                            && (text.starts_with('"') || text.starts_with('r'));
                        if !is_string_literal {
                            return false;
                        }
                        let mut words = HashSet::new();
                        add_string_words(text, &mut words);
                        words.iter().any(|word| unsupplied_names.contains(word))
                    })
                });
            if defines_a_name || names_one_in_a_string {
                suppliers.push(unit_index);
            }
        }
        suppliers
    }

    /// 链接集里、闭包外，`include_str!` 读进一份闭包还没读到、又定义了这几个名字之一的 `.wgsl` 的 Rust 单元（WGSL 跨文件着落边）。
    /// Rust 侧把几份 `.wgsl` 拼成一份着色器时，一份里用到的名字定义在另一份里；只读到前一份的闭包把读后一份的单元连同它读的那份
    /// `.wgsl` 并进来，名字在合起来的名字空间里解。只看那份 `.wgsl` 有没有定义这个名字，不核读它的单元真被拿去拼了内核（超集）。
    /// 交（单元，它在链接集里的那个目标）；链接集外的单元不交，它读的 `.wgsl` 不给这个入口着落。
    fn wgsl_includers_outside_the_closure(
        &self,
        names_the_wgsl_module_leaves_undefined: &BTreeSet<String>,
    ) -> Vec<(UnitIndex, TargetIndex)> {
        let workspace = self.workspace;
        let defining_files: BTreeSet<FileIndex> = workspace
            .wgsl_units
            .iter()
            .filter(|wgsl_unit| {
                !self.wgsl_files.contains(&wgsl_unit.file)
                    && names_the_wgsl_module_leaves_undefined.contains(&wgsl_unit.name)
            })
            .map(|wgsl_unit| wgsl_unit.file)
            .collect();
        if defining_files.is_empty() {
            return Vec::new();
        }
        let mut includers = Vec::new();
        for (position, unit) in workspace.units.iter().enumerate() {
            let unit_index = UnitIndex(position);
            if self.in_closure.contains(&unit_index)
                || self.wgsl_name_suppliers_tried.contains(&unit_index)
            {
                continue;
            }
            let Some(linked_target) = self.linked_target_of(unit_index) else {
                continue;
            };
            let includes_a_defining_file = unit.reads.iter().any(|read| match read {
                SourceRead::File {
                    absolute_path,
                    origin: ReadOrigin::Included,
                    ..
                } => workspace
                    .indexes
                    .files_by_path
                    .get(absolute_path)
                    .is_some_and(|file| defining_files.contains(file)),
                SourceRead::File {
                    origin: ReadOrigin::Runtime,
                    ..
                }
                | SourceRead::BuildScriptOutput { .. }
                | SourceRead::BuildScriptEnvironmentValue { .. }
                | SourceRead::BinaryExecutable { .. }
                | SourceRead::Directory { .. }
                | SourceRead::OwnProgramRestart { .. }
                | SourceRead::ExternBlockOfListedSystemLibraryCalls { .. }
                | SourceRead::ChildTestProcess { .. } => false,
            });
            if includes_a_defining_file {
                includers.push((unit_index, linked_target));
            }
        }
        includers
    }

    /// 着落做成边：闭包读到的 `.wgsl` 里没定义的名字，先把链接集里 `include_str!` 读进定义它的另一份 `.wgsl` 的单元并进闭包
    /// （`wgsl_includers_outside_the_closure`）；哪份 `.wgsl` 都没定义、Rust 闭包里也还没有着落的，再把链接集里供它的单元并进闭包
    /// （连同它们的闭包，由调用方再 `drain`）。交回这一趟有没有新并进来的单元。
    pub(super) fn pull_in_wgsl_name_suppliers(&mut self) -> bool {
        if self.wgsl_files.is_empty()
            || !self.switches.is_intact(BrokenRule::WgslExternalNames)
            || !self.switches.is_intact(BrokenRule::WgslSupplierEdges)
        {
            return false;
        }
        let (_reached, external_names) = self.wgsl_module_and_external_names();
        let names_the_wgsl_module_leaves_undefined: BTreeSet<String> = external_names
            .iter()
            .map(|(_, name)| name.clone())
            .collect();
        let includers =
            self.wgsl_includers_outside_the_closure(&names_the_wgsl_module_leaves_undefined);
        self.rule_being_followed = "WGSL 外名的着落";
        if !includers.is_empty() {
            // 先只并读 `.wgsl` 的单元：下一趟在合起来的模块上重算外名，定义在那几份里的名字不再去找 Rust 侧的着落
            for (includer, linked_target) in &includers {
                self.wgsl_name_suppliers_tried.insert(*includer);
                self.push(*includer, *linked_target);
            }
            return true;
        }
        let sources = self.wgsl_external_name_sources_of_rust_closure();
        let accepts_prefixes = self
            .switches
            .is_intact(BrokenRule::WgslExternalNamePrefixes);
        let unsupplied_names: BTreeSet<String> = external_names
            .into_iter()
            .map(|(_, name)| name)
            .filter(|name| !sources.supply(name, accepts_prefixes))
            .collect();
        if unsupplied_names.is_empty() {
            return false;
        }
        let suppliers = self.wgsl_name_suppliers_outside_the_closure(&unsupplied_names);
        for supplier in &suppliers {
            self.wgsl_name_suppliers_tried.insert(*supplier);
            self.push_with_a_linked_target(*supplier);
        }
        !suppliers.is_empty()
    }
}
