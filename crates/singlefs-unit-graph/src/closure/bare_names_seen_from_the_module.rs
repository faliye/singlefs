//! 收集：R-裸名按名字连单元，同一个目标里只连单元所在模块看得见的那几份候选。
use super::*;

/// R-裸名按模块筛候选时，路径根名是这几个的指标准库，不点本目标里的哪一份（另几个是看得见的别的 crate 的库名）。
const PATH_ROOTS_OF_THE_STANDARD_CRATES: [&str; 3] = ["std", "core", "alloc"];

/// use 语句里 `path_start` 那一段前面的路径，花括号展开：`use a::{b::{c, d}, e}` 里 `d` 前面是 `a`、`b`。顺着 `段::` 往前收，碰到不是 `段::` 的就跳到包着它的 `{` 接着收，
/// 收到 `use` 为止；`use_keyword` 是 `use` 的位置。
fn use_path_leading_to<'source>(
    source: &'source str,
    tokens: &[Token],
    use_keyword: usize,
    path_start: usize,
) -> Vec<&'source str> {
    let text_at = |position: usize| {
        tokens
            .get(position)
            .map_or("", |token| token_text(source, token))
    };
    let mut path = Vec::new();
    let mut position = path_start;
    loop {
        if position >= use_keyword + 4
            && text_at(position - 1) == ":"
            && text_at(position - 2) == ":"
            && tokens[position - 3].kind == TokenKind::Identifier
        {
            path.push(text_at(position - 3));
            position -= 3;
            continue;
        }
        let mut nested_braces = 0usize;
        let mut enclosing_brace = None;
        for cursor in (use_keyword + 1..position).rev() {
            match text_at(cursor) {
                "}" => nested_braces += 1,
                "{" if nested_braces == 0 => {
                    enclosing_brace = Some(cursor);
                    break;
                }
                "{" => nested_braces -= 1,
                _ => {}
            }
        }
        let Some(brace) = enclosing_brace else {
            path.reverse();
            return path;
        };
        position = brace;
    }
}

impl<'workspace> ClosureRun<'workspace> {
    pub(super) fn follow_bare_names(
        &mut self,
        unit_index: UnitIndex,
        target: TargetIndex,
        alias_originals: &[String],
    ) {
        if !self.switches.is_intact(BrokenRule::BareNames) {
            return;
        }
        self.rule_being_followed = "R-裸名";
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let visible = self.link.visible_targets(target);
        let ignores_visibility = !self.switches.is_intact(BrokenRule::TargetScope);
        let links_outside_crate_names_only_when_imported = self
            .switches
            .is_intact(BrokenRule::OutsideCrateNamesImportedOnly);
        let links_outside_crate_names_by_module = self
            .switches
            .is_intact(BrokenRule::OutsideCrateNamesByModule);
        let links_crates_directory_names_by_module = self
            .switches
            .is_intact(BrokenRule::CratesDirectoryNamesByModule);
        // macro_rules! 体里的裸名在调用处解，定义所在模块看不出它落在哪一份：一律全连
        let is_resolved_at_the_call_site = unit.kind == UnitKind::MacroRulesDefinition;
        let local_bindings_shadow_same_named_functions = self
            .switches
            .is_intact(BrokenRule::LocalBindingsShadowSameNamedFunctions);
        let unit_crate = workspace.targets[target.0].crate_index;
        let roots_naming_another_crate: BTreeSet<&str> = PATH_ROOTS_OF_THE_STANDARD_CRATES
            .iter()
            .copied()
            .chain(
                visible
                    .iter()
                    .map(|visible_target| workspace.targets[visible_target.0].crate_index)
                    .filter(|visible_crate| *visible_crate != unit_crate)
                    .map(|visible_crate| {
                        workspace.crates[visible_crate.0].crate_identifier.as_str()
                    }),
            )
            .collect();
        for name in unit.references.plain_names.iter().chain(alias_originals) {
            // 只在这个名字真有同名函数要连时算一次（文件头「R-裸名局部绑定遮住同名函数」）；use 改名的原名不是这个单元写的名字，不算遮住
            let mut is_only_a_local_binding: Option<bool> = None;
            for visible_target in &visible {
                let Some(found) = workspace
                    .indexes
                    .units_by_target_and_name
                    .get(visible_target)
                    .and_then(|by_name| by_name.get(name.as_str()))
                else {
                    continue;
                };
                let visible_crate = workspace.targets[visible_target.0].crate_index;
                let seen_from_the_module: Vec<UnitIndex>;
                let mut candidates: &[UnitIndex] = found;
                // 同一个目标里只连这个单元所在模块看得见的那几份（文件头「R-裸名」）：工作区成员与工作区外的 crate 一样
                // （验证包里 CPU 判器的单元经 use 从格式库引进 SLOT_BYTES，不连到 gpu_unit_checks 里私有的同名常量）
                let links_by_module = match workspace.crates[visible_crate.0].origin {
                    CrateOrigin::WorkspaceMember => links_crates_directory_names_by_module,
                    CrateOrigin::PathDependencyOutsideTheWorkspace => {
                        // 工作区外的 crate 按调用关系连（文件头「工作区外的 path 依赖」）：从别的包连进来，这个名字要在工作区里经 use 或路径
                        // 从它引进来过（验证包的 run 字段不连到 prune-server 的 command_line::run）
                        if links_outside_crate_names_only_when_imported
                            && visible_crate != unit_crate
                            && !self.is_name_imported_from_the_outside_crate(visible_crate, name)
                        {
                            continue;
                        }
                        links_outside_crate_names_by_module
                    }
                };
                if links_by_module
                    && *visible_target == target
                    && !alias_originals.contains(name)
                    && !is_resolved_at_the_call_site
                {
                    seen_from_the_module = self.candidates_seen_from_the_module_of_the_unit(
                        unit_index,
                        target,
                        name,
                        candidates,
                        &roots_naming_another_crate,
                    );
                    candidates = &seen_from_the_module;
                }
                for found_unit in candidates {
                    if local_bindings_shadow_same_named_functions
                        && workspace.units[found_unit.0].kind == UnitKind::Function
                        && unit.kind == UnitKind::Function
                        && !alias_originals.contains(name)
                        && *is_only_a_local_binding.get_or_insert_with(|| {
                            unit.token_range.is_some_and(|(first_token, end_token)| {
                                let file = &workspace.files[unit.file.0];
                                is_bare_name_only_a_local_binding_of_the_function(
                                    &file.source,
                                    &file.tokens[first_token..end_token],
                                    name,
                                )
                            })
                        })
                    {
                        continue;
                    }
                    if *visible_target == target
                        || workspace.units[found_unit.0].is_public
                        || ignores_visibility
                    {
                        self.record_the_first_edge(*found_unit, name);
                        self.push(*found_unit, *visible_target);
                    }
                }
            }
        }
    }

    /// 别的包里的裸名 `name` 在工作区里经 use 或路径从工作区外的 `outside_crate` 引进来过（`workspace` 的
    /// `names_reaching_an_outside_crate`）。
    fn is_name_imported_from_the_outside_crate(
        &self,
        outside_crate: CrateIndex,
        name: &str,
    ) -> bool {
        match self
            .workspace
            .indexes
            .names_reaching_crates_outside_the_workspace
            .get(&outside_crate)
            .expect("每个工作区外的 crate 装工作区时都算了引进名字；这里没有说明 CrateOrigin 与索引对不上")
        {
            NamesReachingAnOutsideCrate::EveryName => true,
            NamesReachingAnOutsideCrate::Named(names) => names.contains(name),
        }
    }

    /// 单元在目标里所在的模块路径：文件在目标模块树里的路径接文件里的行内模块路径。
    fn module_path_in_the_target(&self, unit_index: UnitIndex, target: TargetIndex) -> Vec<String> {
        let unit = &self.workspace.units[unit_index.0];
        let mut module_path = self
            .workspace
            .membership(unit.file, target)
            .expect("候选与单元都是按这个目标的模块树找到的，文件在这个目标里")
            .module_path
            .clone();
        module_path.extend(unit.local_module_path.iter().cloned());
        module_path
    }

    /// 同一个目标里，单元按裸名 `name` 连得到 `candidates` 里的哪几份（与 `scripts/gate/lib_function_units.py` 的
    /// `RustWorkspace.candidates_seen_from_module` 同一个判法）：与单元同一模块的；所在模块的 use 引进了 `name`（或是 glob）、引进它的那一支路径
    /// （花括号展开之后）恰好点到候选所在模块的（`crate`、`self`、`super` 起头的从根模块、本模块、往上那一层接着走，要整条相等；别的起头的相对路径要是候选模块路径的后缀）；
    /// 单元里写成 `前缀::name`、前缀是候选所在模块末段的（前缀是 `super` 时按整条路径里 `super` 的个数往上数层）。
    /// use 路径或 `…::name` 整条路径的根名在 `roots_naming_another_crate` 里的（别的 crate 的库名、`std`、`core`、`alloc`）指的是别的 crate，
    /// 不点这个目标里的哪一份，跳过这一处。
    /// 认不出落在哪一份的退回全连：use 的某一支或前缀一份都没点到（经 `pub use` 转出来的）、这条 use 里找不到引进 `name` 的那一支、单元里另有 `use`（函数体里的）、`name` 后面接 `!` 与开括号；
    /// use 的一支点到的候选带条件编译、或候选所在模块另有 use 按名字引进 `name`（cfg 互斥的另一支可能是 `pub use` 转来的）；use 的相对路径首段在本作用域被哪条 use 按名字引进（含 `as` 改名与 `{self}`，路径所在的这条 use 也算）、或本作用域里有条目级宏调用。
    /// 前缀是本目标的类型或 trait、或 `Self` 的是关联项（R-路径另管），跳过这一处。哪一样都不沾的（字段名、局部变量、impl 成员自己的名字）一份都不连。
    fn candidates_seen_from_the_module_of_the_unit(
        &self,
        unit_index: UnitIndex,
        target: TargetIndex,
        name: &str,
        candidates: &[UnitIndex],
        roots_naming_another_crate: &BTreeSet<&str>,
    ) -> Vec<UnitIndex> {
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let file = &workspace.files[unit.file.0];
        let text_at = |position: usize| {
            file.tokens
                .get(position)
                .map_or("", |token| token_text(&file.source, token))
        };
        let is_identifier_at = |position: usize| {
            file.tokens
                .get(position)
                .is_some_and(|token| token.kind == TokenKind::Identifier)
        };
        let own_module = self.module_path_in_the_target(unit_index, target);
        let candidate_modules: Vec<Vec<String>> = candidates
            .iter()
            .map(|candidate| self.module_path_in_the_target(*candidate, target))
            .collect();
        let mut seen: BTreeSet<usize> = candidate_modules
            .iter()
            .enumerate()
            .filter(|(_, module)| **module == own_module)
            .map(|(candidate_position, _)| candidate_position)
            .collect();
        // 路径（不含 name 本身）点到的候选（下标）；一份都没点到交 None
        let placed = |segments: &[&str]| -> Option<BTreeSet<usize>> {
            // 路径里写了几个 `super` 就往上几层；超出根模块的写法编不过，按根模块算
            let levels_up = segments
                .iter()
                .filter(|segment| **segment == "super")
                .count();
            let found: BTreeSet<usize> = candidate_modules
                .iter()
                .enumerate()
                .filter(|(_, module)| {
                    module
                        .last()
                        .is_some_and(|last_segment| segments.contains(&last_segment.as_str()))
                        || (module.is_empty() && segments.first() == Some(&"crate"))
                        || (levels_up > 0
                            && module[..]
                                == own_module[..own_module.len().saturating_sub(levels_up)])
                        || (segments.first() == Some(&"self") && **module == own_module)
                })
                .map(|(candidate_position, _)| candidate_position)
                .collect();
            (!found.is_empty()).then_some(found)
        };
        let uses_in_the_scope_of_the_unit: &[UnitIndex] = workspace
            .indexes
            .uses_by_scope
            .get(&(unit.file, unit.local_module_path.clone()))
            .map_or(&[][..], Vec::as_slice);
        // 作用域 `uses` 里除 `except_use` 之外有一条 use 按名字引进 `imported_name`（`as` 改出的名字也算；glob 不算）
        let is_imported_by_another_use =
            |uses: &[UnitIndex], except_use: Option<UnitIndex>, imported_name: &str| {
                uses.iter().any(|other_use| {
                    Some(*other_use) != except_use
                        && workspace.units[other_use.0]
                            .imports
                            .as_ref()
                            .is_some_and(|imports| {
                                imports
                                    .names
                                    .iter()
                                    .any(|imported| imported == imported_name)
                            })
                })
            };
        // 本作用域（与单元同一文件、同一行内模块）里有条目级宏调用：它展开出什么名字（宏体里写的、实参交进去的、再调别的宏生成的）
        // 看不全，相对路径的首段可能是它展开出的 `use … as 首段`，一律当首段指谁看不出
        let scope_has_an_item_macro_invocation = workspace
            .indexes
            .scopes_with_an_item_macro_invocation
            .contains(&(unit.file, unit.local_module_path.clone()));
        // 候选带条件编译：它自己（连外层行内模块、impl 头，`cfg_attr` 里套的 `cfg` 也算）的条件，或它所在文件经哪条 `mod` 连进来带条件
        let is_conditionally_compiled = |candidate: UnitIndex| {
            let candidate_unit = &workspace.units[candidate.0];
            !candidate_unit.conditions.is_empty()
                || workspace
                    .membership(candidate_unit.file, target)
                    .is_some_and(|membership| {
                        membership
                            .condition_alternatives
                            .iter()
                            .any(|alternative| !alternative.is_empty())
                    })
        };
        // use 的一支路径（不含名字本身）恰好点到的候选（下标）；一份都没点到交 None。
        // 恰好点到才算：点到的模块里自己定义的名字压过它 glob 转出来的；模块里没有这个名字的候选，名字可能是它经 `pub use` 转出来的。
        // 点到的候选带条件编译、或候选所在模块另有 use 按名字引进它，名字可能是 cfg 互斥的另一支 `pub use` 转来的，交 None 退回全连；
        // 相对路径的首段在本作用域被哪条 use 按名字引进（含 `as` 改名；路径所在的这条 use 自己的另一支也算：`use {甲 as 乙, 乙::名}`、
        // `use 甲::{self, 名}`），或本作用域里有条目级宏调用，首段真指的模块看不出，交 None 退回全连。`path_use` 是这一支所在的 use
        let placed_by_the_use_path = |path: &[&str],
                                      path_use: UnitIndex|
         -> Option<BTreeSet<usize>> {
            let leading_supers = path
                .iter()
                .take_while(|segment| **segment == "super")
                .count();
            let own_module_words = own_module.iter().map(String::as_str);
            let exact_module: Option<Vec<&str>> = match path.first().copied() {
                Some("crate") => Some(path[1..].to_vec()),
                Some("self") => Some(own_module_words.chain(path[1..].iter().copied()).collect()),
                Some("super") => Some(
                    own_module_words
                        .take(own_module.len().saturating_sub(leading_supers))
                        .chain(path[leading_supers..].iter().copied())
                        .collect(),
                ),
                Some(_) | None => None,
            };
            if exact_module.is_none()
                && path.first().copied().is_some_and(|first_segment| {
                    scope_has_an_item_macro_invocation
                        || is_imported_by_another_use(
                            uses_in_the_scope_of_the_unit,
                            None,
                            first_segment,
                        )
                })
            {
                return None;
            }
            let found: BTreeSet<usize> = candidate_modules
                .iter()
                .enumerate()
                .filter(|(_, module)| {
                    let module_words: Vec<&str> = module.iter().map(String::as_str).collect();
                    match &exact_module {
                        Some(exact) => module_words == *exact,
                        None => !path.is_empty() && module_words.ends_with(path),
                    }
                })
                .map(|(candidate_position, _)| candidate_position)
                .collect();
            let may_be_re_exported_in_a_conditional_branch =
                found.iter().any(|candidate_position| {
                    let candidate = candidates[*candidate_position];
                    let candidate_unit = &workspace.units[candidate.0];
                    is_conditionally_compiled(candidate)
                        || workspace
                            .indexes
                            .uses_by_scope
                            .get(&(
                                candidate_unit.file,
                                candidate_unit.local_module_path.clone(),
                            ))
                            .is_some_and(|uses_of_the_candidate_module| {
                                is_imported_by_another_use(
                                    uses_of_the_candidate_module,
                                    Some(path_use),
                                    name,
                                )
                            })
                });
            if may_be_re_exported_in_a_conditional_branch {
                return None;
            }
            (!found.is_empty()).then_some(found)
        };
        let mut is_placement_unknown = false;
        if let Some(uses) = workspace
            .indexes
            .uses_by_scope
            .get(&(unit.file, unit.local_module_path.clone()))
        {
            for use_index in uses {
                let use_unit = &workspace.units[use_index.0];
                let (Some((first_token, end_token)), Some(imports)) =
                    (use_unit.token_range, use_unit.imports.as_ref())
                else {
                    continue;
                };
                if !imports.is_glob
                    && !imports
                        .names
                        .iter()
                        .any(|imported| imported.as_str() == name)
                {
                    continue;
                }
                // use 之后头一个标识符（跳过开头的 `::`）是路径的根名；`use {…}` 没有根名
                let path_root = (first_token..end_token)
                    .skip_while(|position| {
                        !(is_identifier_at(*position) && text_at(*position) == "use")
                    })
                    .skip(1)
                    .find(|position| text_at(*position) != ":")
                    .filter(|position| is_identifier_at(*position))
                    .map(text_at);
                if path_root.is_some_and(|root| roots_naming_another_crate.contains(root)) {
                    continue;
                }
                // 这条 use 里引进 name 的每一支（`*`、名字本身）的路径，花括号展开之后；`原名 as name` 那一支 `as` 前面不是 `段::`，路径只到外层花括号，多半点不到，退回全连
                let use_keyword = (first_token..end_token)
                    .find(|position| is_identifier_at(*position) && text_at(*position) == "use")
                    .unwrap_or(first_token);
                let paths_bringing_in_the_name: Vec<Vec<&str>> = (use_keyword + 1..end_token)
                    .filter(|position| {
                        text_at(*position) == "*"
                            || (is_identifier_at(*position)
                                && text_at(*position) == name
                                && text_at(*position + 1) != ":")
                    })
                    .map(|leaf| use_path_leading_to(&file.source, &file.tokens, use_keyword, leaf))
                    .collect();
                let placed_by_every_path = (!paths_bringing_in_the_name.is_empty())
                    .then(|| {
                        paths_bringing_in_the_name
                            .iter()
                            .map(|path| placed_by_the_use_path(path, *use_index))
                            .collect::<Option<Vec<BTreeSet<usize>>>>()
                    })
                    .flatten()
                    .map(|hits| hits.into_iter().flatten().collect::<BTreeSet<usize>>());
                match placed_by_every_path {
                    Some(hit) => seen.extend(hit),
                    None => is_placement_unknown = true,
                }
            }
        }
        if let (Some((first_token, end_token)), false) = (unit.token_range, is_placement_unknown) {
            let names_in_the_target = workspace.indexes.units_by_target_and_name.get(&target);
            let is_type_or_trait_of_the_target = |prefix: &str| {
                names_in_the_target
                    .and_then(|by_name| by_name.get(prefix))
                    .is_some_and(|units| {
                        units.iter().any(|found_unit| {
                            let kind = workspace.units[found_unit.0].kind;
                            kind.is_type() || kind == UnitKind::TraitHead
                        })
                    })
            };
            for position in first_token..end_token {
                let word = text_at(position);
                if word == "use" && is_identifier_at(position) && position != first_token {
                    is_placement_unknown = true;
                    break;
                }
                if word != name || !is_identifier_at(position) {
                    continue;
                }
                if text_at(position + 1) == "!" && matches!(text_at(position + 2), "(" | "[" | "{")
                {
                    is_placement_unknown = true;
                    break;
                }
                let Some(prefix_position) = position.checked_sub(3) else {
                    continue;
                };
                if text_at(position - 1) != ":"
                    || text_at(position - 2) != ":"
                    || !is_identifier_at(prefix_position)
                {
                    continue;
                }
                let prefix = text_at(prefix_position);
                // 整条路径 `根::…::前缀::name`，从前缀往前顺着 `标识符 ::` 收到根
                let mut path_before_the_name = vec![prefix];
                let mut segment_position = prefix_position;
                while segment_position >= 3
                    && text_at(segment_position - 1) == ":"
                    && text_at(segment_position - 2) == ":"
                    && is_identifier_at(segment_position - 3)
                {
                    segment_position -= 3;
                    path_before_the_name.push(text_at(segment_position));
                }
                path_before_the_name.reverse();
                if path_before_the_name
                    .first()
                    .is_some_and(|root| roots_naming_another_crate.contains(root))
                {
                    continue;
                }
                if prefix == "Self"
                    || (is_type_or_trait_of_the_target(prefix)
                        && !matches!(prefix, "crate" | "self" | "super"))
                {
                    continue;
                }
                let only_the_prefix = [prefix];
                let placement_segments: &[&str] = if prefix == "super" {
                    &path_before_the_name
                } else {
                    &only_the_prefix
                };
                match placed(placement_segments) {
                    Some(hit) => seen.extend(hit),
                    None => {
                        is_placement_unknown = true;
                        break;
                    }
                }
            }
        }
        if is_placement_unknown {
            return candidates.to_vec();
        }
        candidates
            .iter()
            .enumerate()
            .filter(|(candidate_position, _)| seen.contains(candidate_position))
            .map(|(_, candidate)| *candidate)
            .collect()
    }
}
