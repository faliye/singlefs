//! 收集：起本测试二进制、`--exact` 点名的子进程测试连到入口目标里那个 `#[test] fn`。
use super::*;

impl<'workspace> ClosureRun<'workspace> {
    /// 只在测试里编进来的代码：带 `#[test]` 的函数，外层条件链里有 `test` 的单元（`#[cfg(test)]` 模块里的），
    /// 或它所在的文件在链接集里的那个目标下每一种挂法都带 `test`（`#[cfg(test)] mod 名字;` 引进来的文件）。
    pub(super) fn is_test_only_code(&self, unit_index: UnitIndex) -> bool {
        let unit = &self.workspace.units[unit_index.0];
        if unit.has_test_attribute || unit.is_inside_test_only_code {
            return true;
        }
        self.linked_target_of(unit_index)
            .and_then(|linked_target| self.workspace.membership(unit.file, linked_target))
            .is_some_and(|membership| {
                !membership.condition_alternatives.is_empty()
                    && membership.condition_alternatives.iter().all(|alternative| {
                        alternative
                            .iter()
                            .any(crate::rust_items::predicate_requires_the_test_configuration)
                    })
            })
    }

    /// 单元自己的词法单元原文。
    fn token_texts_of(&self, unit_index: UnitIndex) -> Vec<&'workspace str> {
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let Some((first, end)) = unit.token_range else {
            return Vec::new();
        };
        let file = &workspace.files[unit.file.0];
        file.tokens[first..end]
            .iter()
            .map(|token| token_text(&file.source, token))
            .collect()
    }

    /// 常量 `name` 的串值：先找写在 `context` 这个单元里的 `const name: … = "…";`，再找同一个包、在这一次链接集里（同一包的别的 bin 不算）
    /// 叫这个名字的常量单元（几个都得写成 `= "…";`、值相同）。找不到、写法不是串字面量或几个值不同交 `None`。
    fn constant_string_value(&self, context: UnitIndex, name: &str) -> Option<String> {
        let definition_value = |texts: &[&str]| -> Option<String> {
            let const_index = (0..texts.len()).find(|index| {
                texts[*index] == "const"
                    && texts.get(index + 1).copied() == Some(name)
                    && texts.get(index + 2).copied() == Some(":")
            })?;
            let equals = (const_index + 3..texts.len()).find(|index| texts[*index] == "=")?;
            match texts.get(equals + 1..equals + 3)? {
                [literal, ";"] => plain_string_content(literal).map(str::to_string),
                _ => None,
            }
        };
        if let Some(value) = definition_value(&self.token_texts_of(context)) {
            return Some(value);
        }
        let workspace = self.workspace;
        let crate_index = workspace.files[workspace.units[context.0].file.0].crate_index;
        let mut values = BTreeSet::new();
        for (position, unit) in workspace.units.iter().enumerate() {
            if unit.kind == UnitKind::Constant
                && unit.name == name
                && workspace.files[unit.file.0].crate_index == crate_index
                && self.linked_target_of(UnitIndex(position)).is_some()
            {
                values.insert(definition_value(&self.token_texts_of(UnitIndex(position)))?);
            }
        }
        match values.len() {
            1 => values.into_iter().next(),
            _ => None,
        }
    }

    /// 几段连成测试名：常量在 `context` 这个单元（与它的 crate）里解。
    fn folded_test_name(&self, context: UnitIndex, pieces: &[TestNamePiece]) -> Option<String> {
        let mut name = String::new();
        for piece in pieces {
            match piece {
                TestNamePiece::Literal(text) => name.push_str(text),
                TestNamePiece::Constant(constant) => {
                    name.push_str(&self.constant_string_value(context, constant)?);
                }
            }
        }
        Some(name)
    }

    /// 入口目标里 libtest 名字（模块路径 `::` 函数名，不带 crate 名）恰好是 `test_name` 的那一个 `#[test] fn`。
    fn test_function_named(&self, test_name: &str) -> Option<UnitIndex> {
        let workspace = self.workspace;
        let segments: Vec<&str> = test_name.split("::").collect();
        let (function_name, module_path) = segments.split_last()?;
        let mut found = Vec::new();
        for file in &workspace.files_of_target[self.entry_target.0] {
            for unit_index in units_of_file(workspace, *file) {
                let unit = &workspace.units[unit_index.0];
                if unit.kind != UnitKind::Function
                    || !unit.has_test_attribute
                    || unit.container.is_some()
                    || unit.name != *function_name
                {
                    continue;
                }
                let matches_the_module = unit_module_path(workspace, unit_index, self.entry_target)
                    .is_some_and(|path| {
                        path.iter()
                            .map(String::as_str)
                            .eq(module_path.iter().copied())
                    });
                if matches_the_module {
                    found.push(unit_index);
                }
            }
        }
        match found.as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    }

    /// 起子进程的单元 `spawning` 里那条 `--exact` 点名的测试：解出来的每一个 `#[test] fn`（参数写法时闭包里每一处调用各解一个）。
    /// 有一处折不出、解不到恰好一个，或参数写法在闭包里一处调用都没有，交 `None`（拒算 `current-exe`）。
    pub(super) fn child_tests_of(
        &self,
        spawning: UnitIndex,
        test_name: &ChildTestName,
    ) -> Option<Vec<UnitIndex>> {
        match test_name {
            ChildTestName::Pieces(pieces) => {
                let name = self.folded_test_name(spawning, pieces)?;
                Some(vec![self.test_function_named(&name)?])
            }
            ChildTestName::ParameterOfTheSpawningFunction {
                function_name,
                parameter_position,
            } => {
                let mut tests = Vec::new();
                for caller in &self.in_closure {
                    let texts = self.token_texts_of(*caller);
                    for (index, text) in texts.iter().enumerate() {
                        let is_call = *text == function_name
                            && texts.get(index + 1).copied() == Some("(")
                            && !matches!(
                                index.checked_sub(1).map(|previous| texts[previous]),
                                Some("fn" | ".")
                            );
                        if !is_call {
                            continue;
                        }
                        let mut depth = 0usize;
                        let close = (index + 1..texts.len()).find(|position| {
                            match texts[*position] {
                                "(" | "[" | "{" => depth += 1,
                                ")" | "]" | "}" => depth = depth.saturating_sub(1),
                                _ => {}
                            }
                            depth == 0
                        })?;
                        let arguments = split_at_top_level_commas(&texts[index + 2..close]);
                        let pieces =
                            test_name_pieces_of_texts(arguments.get(*parameter_position)?)?;
                        let name = self.folded_test_name(*caller, &pieces)?;
                        tests.push(self.test_function_named(&name)?);
                    }
                }
                (!tests.is_empty()).then_some(tests)
            }
        }
    }

    /// 子进程测试边（`current-exe-child`）：闭包里起子进程的单元点名的测试并进闭包（在入口目标里收）。交回这一趟有没有新推进去的单元。
    pub(super) fn pull_in_child_process_tests(&mut self) -> bool {
        let workspace = self.workspace;
        let mut children = Vec::new();
        for unit_index in &self.in_closure {
            for read in &workspace.units[unit_index.0].reads {
                if let SourceRead::ChildTestProcess { test_name, .. } = read {
                    if let Some(tests) = self.child_tests_of(*unit_index, test_name) {
                        children.extend(tests);
                    }
                }
            }
        }
        self.rule_being_followed = "子进程测试";
        let mut pushed = false;
        for child in children {
            if !self.seen_pairs.contains(&(child, self.entry_target)) {
                self.push(child, self.entry_target);
                pushed = true;
            }
        }
        pushed
    }
}
