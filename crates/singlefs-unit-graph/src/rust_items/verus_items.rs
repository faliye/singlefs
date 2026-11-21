//! `verus!` 里的条目：按条目起头切单元，`assume_specification` 单元。
use super::*;

impl<'scan> RustScanner<'scan> {
    fn is_verus_item_start(&self, index: usize) -> bool {
        if self.is_documentation_token(index) {
            return true;
        }
        if !self.is_identifier(index) && self.text_of(index) != "#" {
            return false;
        }
        match self.text_of(index) {
            "#" => self.text_of(index + 1) == "[",
            "pub" | "fn" | "spec" | "proof" | "exec" | "open" | "closed" | "broadcast"
            | "struct" | "enum" | "union" | "impl" | "trait" | "const" | "static" | "type"
            | "use" | "mod" | "unsafe" | "extern" | "macro_rules" | "global" | "tracked"
            | "ghost" | "uninterp" | "axiom" | "async" | "default" => true,
            "assume_specification" => self.is_assume_specification_keyword(index),
            _ => self.text_of(index + 1) == "!" && self.is_open_bracket(index + 2),
        }
    }

    /// `assume_specification` 这个词后面接 `[`（点名的路径）或 `<`（泛型参数表）：Verus 的 `assume_specification` 条目的关键字。
    fn is_assume_specification_keyword(&self, index: usize) -> bool {
        self.text_of(index) == "assume_specification"
            && self.kind_of(index + 1) == Some(TokenKind::Punctuation)
            && matches!(self.text_of(index + 1), "[" | "<")
    }

    /// 从 `open_angle` 那个 `<` 起配到它的 `>`，交 `>` 后面那个词法单元的位置；`->` 的 `>` 不算。配不上（`end` 之前没回到零层）交 `None`。
    fn after_the_angle_brackets(&self, open_angle: usize, end: usize) -> Option<usize> {
        let mut depth = 0usize;
        // 每一步往后走一个词法单元：至多走到条目末尾
        for position in open_angle..end {
            match self.text_of(position) {
                "<" => depth += 1,
                ">" if position == 0 || self.text_of(position - 1) != "-" => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(position + 1);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// `assume_specification` 之后（可带一段 `<…>` 泛型参数表）那对方括号里点名的路径，末段的名字：
    /// `[ singlefs_format::WIDE_CHECKSUM_BYTES ]` 是 `WIDE_CHECKSUM_BYTES`，`[ Vec::<Element>::len ]` 是 `len`（尖括号里的名字不算）。
    /// 认不出那对方括号、方括号里尖括号外一个名字都没有的，交 `None`。
    fn assume_specification_target_name(
        &self,
        after_the_keyword: usize,
        end: usize,
    ) -> Option<String> {
        let open_bracket = if self.text_of(after_the_keyword) == "<" {
            self.after_the_angle_brackets(after_the_keyword, end)?
        } else {
            after_the_keyword
        };
        if self.text_of(open_bracket) != "[" || !self.is_open_bracket(open_bracket) {
            return None;
        }
        let close_bracket = self.close_of(open_bracket);
        let mut angle_depth = 0usize;
        let mut last_name_outside_the_angle_brackets = None;
        // 每一步往后走一个词法单元：至多走到配对的 `]`
        for position in open_bracket + 1..close_bracket {
            match self.text_of(position) {
                "<" => angle_depth += 1,
                ">" if self.text_of(position - 1) != "-" => {
                    angle_depth = angle_depth.saturating_sub(1);
                }
                _ => {
                    if angle_depth == 0 && self.is_identifier(position) {
                        last_name_outside_the_angle_brackets = Some(self.name_at(position));
                    }
                }
            }
        }
        last_name_outside_the_angle_brackets
    }

    /// `verus!` 块里同层条目的起头：块首，或顶层的 `;` / `}` 之后紧接一个条目关键字（或属性）的位置。
    fn verus_item_starts(&self, low: usize, high: usize) -> Vec<usize> {
        let mut starts = Vec::new();
        let mut is_at_boundary = true;
        let mut index = low;
        while index < high {
            if is_at_boundary && self.is_verus_item_start(index) {
                starts.push(index);
            }
            if self.is_open_bracket(index) {
                let close = self.close_of(index);
                is_at_boundary = self.text_of(close) == "}";
                index = close + 1;
                continue;
            }
            is_at_boundary = self.text_of(index) == ";";
            index += 1;
        }
        starts
    }

    pub(super) fn scan_verus_items(&mut self, low: usize, high: usize, scan_context: &ScanContext) {
        let starts = self.verus_item_starts(low, high);
        if starts.first().is_some_and(|first| *first > low) || (starts.is_empty() && low < high) {
            let stray_end = starts.first().copied().unwrap_or(high);
            self.file_level_refusals.push(self.refusal(
                RefusalForm::UnrecognizedItemStart,
                low,
                self.fragment(low, stray_end),
            ));
        }
        for (position, start) in starts.iter().enumerate() {
            let end = starts.get(position + 1).copied().unwrap_or(high);
            self.scan_one_verus_item(*start, end, scan_context);
        }
    }

    fn scan_one_verus_item(&mut self, start: usize, end: usize, scan_context: &ScanContext) {
        let mut index = start;
        let mut attribute_ranges = Vec::new();
        while index < end {
            if self.text_of(index) == "#" && self.text_of(index + 1) == "[" {
                let close = self.close_of(index + 1);
                attribute_ranges.push((index + 1, close));
                index = close + 1;
            } else if self.is_documentation_token(index) {
                index += 1;
            } else {
                break;
            }
        }
        let header = index;
        let attributes = self.attribute_facts(&attribute_ranges);
        let mut conditions = scan_context.conditions.clone();
        conditions.extend(attributes.conditions.iter().cloned());
        let (after_visibility, is_public) = self.skip_visibility(header);
        let keyword_index = self.skip_qualifiers(after_visibility, true);
        let keyword = self.text_of(keyword_index);
        let name_after_keyword = self.name_at(keyword_index + 1);
        let whole = (start, end);
        let simple_kind = match keyword {
            "fn" => Some(UnitKind::Function),
            "struct" => Some(UnitKind::Struct),
            "enum" => Some(UnitKind::Enumeration),
            "union" => Some(UnitKind::Union),
            "type" => Some(UnitKind::TypeAlias),
            "const" => Some(UnitKind::Constant),
            "static" => Some(UnitKind::Static),
            _ => None,
        };
        if let Some(kind) = simple_kind {
            let name = if keyword == "static" && self.text_of(keyword_index + 1) == "mut" {
                self.name_at(keyword_index + 2)
            } else {
                name_after_keyword
            };
            self.push_unit(
                NewUnit::plain(kind, name, whole, header, conditions, is_public),
                scan_context,
            );
            return;
        }
        match keyword {
            "impl" | "trait" => {
                let head_end = self.first_brace_or_semicolon(keyword_index + 1, end);
                let (kind, name, label, implementation) = if keyword == "impl" {
                    let label = implementation_label(
                        &(keyword_index + 1..head_end)
                            .map(|position| self.text_of(position))
                            .collect::<Vec<&str>>(),
                    );
                    (
                        UnitKind::ImplementationHead,
                        label.clone(),
                        label,
                        Some(self.implementation_shape(keyword_index + 1, head_end)),
                    )
                } else {
                    (
                        UnitKind::TraitHead,
                        name_after_keyword.clone(),
                        format!("<trait {name_after_keyword}>"),
                        None,
                    )
                };
                let mut new_unit = NewUnit::plain(
                    kind,
                    name,
                    (start, head_end),
                    header,
                    conditions.clone(),
                    is_public,
                );
                new_unit.implementation = implementation;
                let head = self.push_unit(new_unit, scan_context);
                if self.text_of(head_end) == "{" {
                    let body_close = self.close_of(head_end);
                    let member_context = ScanContext {
                        module_path: scan_context.module_path.clone(),
                        container: Some(head),
                        container_label: Some(label),
                        module_head: scan_context.module_head,
                        conditions,
                    };
                    self.scan_verus_items(head_end + 1, body_close, &member_context);
                }
            }
            "use" => {
                let mut new_unit = NewUnit::plain(
                    UnitKind::Use,
                    format!("use@{}", self.line_of(keyword_index)),
                    whole,
                    header,
                    conditions,
                    is_public,
                );
                new_unit.imports = Some(self.use_imports(keyword_index + 1, end));
                self.push_unit(new_unit, scan_context);
            }
            "broadcast" if self.text_of(keyword_index + 1) == "use" => {
                let mut new_unit = NewUnit::plain(
                    UnitKind::Use,
                    format!("broadcast-use@{}", self.line_of(keyword_index)),
                    whole,
                    header,
                    conditions,
                    is_public,
                );
                new_unit.imports = Some(self.use_imports(keyword_index + 2, end));
                self.push_unit(new_unit, scan_context);
            }
            "broadcast" if self.text_of(keyword_index + 1) == "group" => {
                self.push_unit(
                    NewUnit::plain(
                        UnitKind::VerusBroadcastGroup,
                        self.name_at(keyword_index + 2),
                        whole,
                        header,
                        conditions,
                        is_public,
                    ),
                    scan_context,
                );
            }
            "mod" if self.text_of(keyword_index + 2) == ";" => {
                let mut new_unit = NewUnit::plain(
                    UnitKind::ModuleDeclaration,
                    name_after_keyword.clone(),
                    whole,
                    header,
                    conditions,
                    is_public,
                );
                new_unit.module_declaration = Some(ModuleDeclarationShape {
                    module_name: name_after_keyword,
                    path_attribute: attributes.path_attribute,
                });
                self.push_unit(new_unit, scan_context);
            }
            "mod" if self.text_of(keyword_index + 2) == "{" => {
                let open = keyword_index + 2;
                let close = self.close_of(open);
                let head = self.push_unit(
                    NewUnit::plain(
                        UnitKind::ModuleHead,
                        name_after_keyword.clone(),
                        (start, open + 1),
                        header,
                        conditions.clone(),
                        is_public,
                    ),
                    scan_context,
                );
                let mut module_path = scan_context.module_path.clone();
                module_path.push(name_after_keyword);
                let body_context = ScanContext {
                    module_path,
                    container: None,
                    container_label: None,
                    module_head: Some(head),
                    conditions,
                };
                self.scan_verus_items(open + 1, close, &body_context);
            }
            "assume_specification" if self.is_assume_specification_keyword(keyword_index) => {
                // 按点名的那一项的名字找得到它；名字认不出时这份文件里谁连得到它说不全，照认不出的条目起头记文件级拒算
                let Some(target_name) =
                    self.assume_specification_target_name(keyword_index + 1, end)
                else {
                    self.file_level_refusals.push(self.refusal(
                        RefusalForm::UnrecognizedItemStart,
                        header,
                        self.fragment(header, end),
                    ));
                    return;
                };
                let mut new_unit = NewUnit::plain(
                    UnitKind::VerusAssumeSpecification,
                    format!("assume_specification@{}", self.line_of(keyword_index)),
                    whole,
                    header,
                    conditions,
                    is_public,
                );
                new_unit.declared_names = Some(vec![target_name]);
                self.push_unit(new_unit, scan_context);
            }
            "macro_rules" => {
                self.push_unit(
                    NewUnit::plain(
                        UnitKind::MacroRulesDefinition,
                        self.name_at(keyword_index + 2),
                        whole,
                        header,
                        conditions,
                        attributes.is_macro_export,
                    ),
                    scan_context,
                );
            }
            _ => {
                if let Some(invocation) = self.macro_invocation_at(keyword_index) {
                    let mut new_unit = NewUnit::plain(
                        UnitKind::ItemMacroInvocation,
                        format!("{}!@{}", invocation.macro_name, self.line_of(header)),
                        whole,
                        header,
                        conditions,
                        true,
                    );
                    let close = self.close_of(invocation.open_bracket);
                    new_unit.declared_names =
                        Some(self.item_macro_declared_names(invocation.open_bracket, close));
                    self.push_unit(new_unit, scan_context);
                } else {
                    self.file_level_refusals.push(self.refusal(
                        RefusalForm::UnrecognizedItemStart,
                        header,
                        self.fragment(header, end),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
