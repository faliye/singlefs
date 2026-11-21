//! 条目级宏调用与 extern 块两类条目：宏调用的形状与它声明的名字，extern 块声明的名字与只声明登记过的系统库调用。
use super::*;

/// 属性里没有 `link` 的 `extern "C"` 块里只声明这几个符号时，切分不拒算（`extern_block_declares_only_listed_system_library_calls`）：C 库的系统调用封装，
/// 标准库的文件读写走的是同一层，换 C 库影响它们的程度与 `std::fs` 相同。名单只收仓里真在用的。
/// 闭包里再看链进同一份产物的库都看不看得到，看不全的照拒算（[`SourceRead::ExternBlockOfListedSystemLibraryCalls`]）。
/// 与 `scripts/gate/lib_function_units.py` 同一份。
const SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS: [&str; 2] = ["pwrite", "fdatasync"];

impl<'scan> RustScanner<'scan> {
    pub(super) fn macro_invocation_at(&self, start: usize) -> Option<MacroInvocationShape> {
        let mut index = start;
        if self.text_of(index) == ":" && self.text_of(index + 1) == ":" {
            index += 2;
        }
        let mut macro_name;
        loop {
            if !self.is_identifier(index) {
                return None;
            }
            macro_name = self.name_at(index);
            index += 1;
            if self.text_of(index) == ":" && self.text_of(index + 1) == ":" {
                if !self.switches.is_intact(BrokenRule::PathMacroInvocations) {
                    return None;
                }
                index += 2;
                continue;
            }
            break;
        }
        if self.text_of(index) != "!" || !self.is_open_bracket(index + 1) {
            return None;
        }
        Some(MacroInvocationShape {
            macro_name,
            open_bracket: index + 1,
        })
    }

    /// R-宏：条目级宏调用里的标识符都算它可能定义的名字（属性里的、`.` 后面的与基本类型名也算，宁多不漏），只去掉 Rust 关键字；
    /// 与 `scripts/gate/lib_function_units.py` 的条目级宏调用 `defines` 同一口径。
    pub(super) fn item_macro_declared_names(&self, open: usize, close: usize) -> Vec<String> {
        let mut names = BTreeSet::new();
        for index in open + 1..close {
            if self.is_identifier(index) {
                let name = self.name_at(index);
                if !is_keyword(&name) {
                    names.insert(name);
                }
            }
        }
        names.into_iter().collect()
    }

    pub(super) fn extern_block_declared_names(&self, open: usize, close: usize) -> Vec<String> {
        let mut names = BTreeSet::new();
        for index in open + 1..close {
            if matches!(self.text_of(index), "fn" | "static") {
                let mut name_index = index + 1;
                if self.text_of(name_index) == "mut" {
                    name_index += 1;
                }
                if self.is_identifier(name_index) {
                    names.insert(self.name_at(name_index));
                }
            }
        }
        names.into_iter().collect()
    }

    /// `extern` 块（`keyword_index` 是 `extern`，块在 `open..close`）切分时不用拒算：ABI 是 `"C"`，条目前面的属性里没有 `link` 这个词
    /// （[`extern_block_attributes_name_link`]），块里只有 `fn` 声明，
    /// 每条的前面最多一个 `#[link_name = "…"]`、符号（`link_name` 或函数名）都在 [`SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS`] 里，至少声明一条。
    /// 有 `static`、别的属性、`safe fn` 这类写法的一律不算。
    pub(super) fn extern_block_declares_only_listed_system_library_calls(
        &self,
        item_start: usize,
        keyword_index: usize,
        open: usize,
        close: usize,
    ) -> bool {
        if plain_string_content(self.text_of(keyword_index + 1)) != Some("C")
            || keyword_index + 2 != open
        {
            return false;
        }
        let has_link_attribute =
            extern_block_attributes_name_link(self.source, &self.tokens[item_start..keyword_index]);
        if has_link_attribute {
            return false;
        }
        let mut position = open + 1;
        let mut declared_any = false;
        while position < close {
            let mut link_name = None;
            if self.text_of(position) == "#" {
                let is_link_name = self.text_of(position + 1) == "["
                    && self.text_of(position + 2) == "link_name"
                    && self.text_of(position + 3) == "="
                    && self.text_of(position + 5) == "]";
                if !is_link_name {
                    return false;
                }
                link_name = plain_string_content(self.text_of(position + 4));
                if link_name.is_none() {
                    return false;
                }
                position += 6;
            }
            if self.text_of(position) == "pub" {
                position += 1;
            }
            if self.text_of(position) != "fn"
                || !self.is_identifier(position + 1)
                || self.text_of(position + 2) != "("
            {
                return false;
            }
            let declared_name = self.name_at(position + 1);
            let symbol = link_name.unwrap_or(&declared_name);
            if !SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS.contains(&symbol) {
                return false;
            }
            declared_any = true;
            let Some(semicolon) =
                (self.close_of(position + 2)..close).find(|index| self.text_of(*index) == ";")
            else {
                return false;
            };
            position = semicolon + 1;
        }
        declared_any
    }

    pub(super) fn scan_item_macro_invocation(
        &mut self,
        item_start: usize,
        header: usize,
        invocation: &MacroInvocationShape,
        conditions: Vec<Predicate>,
        scan_context: &ScanContext,
    ) -> usize {
        let open = invocation.open_bracket;
        let close = self.close_of(open);
        let mut end = close + 1;
        if self.text_of(end) == ";" {
            end += 1;
        }
        let is_braced = self.text_of(open) == "{";
        let inner_context = ScanContext {
            module_path: scan_context.module_path.clone(),
            container: scan_context.container,
            container_label: scan_context.container_label.clone(),
            module_head: scan_context.module_head,
            conditions: conditions.clone(),
        };
        match invocation.macro_name.as_str() {
            "verus" if is_braced => {
                if self.switches.is_intact(BrokenRule::VerusClauseBraces) {
                    self.scan_verus_items(open + 1, close, &inner_context);
                } else {
                    self.scan_items(open + 1, close, &inner_context);
                }
                return end;
            }
            "thread_local" if is_braced => {
                self.scan_items(open + 1, close, &inner_context);
                return end;
            }
            _ => {}
        }
        let mut new_unit = NewUnit::plain(
            UnitKind::ItemMacroInvocation,
            format!("{}!@{}", invocation.macro_name, self.line_of(header)),
            (item_start, end),
            header,
            conditions,
            true,
        );
        new_unit.declared_names = Some(self.item_macro_declared_names(open, close));
        match invocation.macro_name.as_str() {
            "include" => self.file_level_refusals.push(self.refusal(
                RefusalForm::ItemLevelInclude,
                header,
                self.fragment(header, end),
            )),
            "paste" | "concat_idents" => new_unit
                .extra_refusals
                .push(RefusalForm::IdentifierPastingMacro),
            _ => {}
        }
        self.push_unit(new_unit, scan_context);
        end
    }
}

#[cfg(test)]
mod tests;
