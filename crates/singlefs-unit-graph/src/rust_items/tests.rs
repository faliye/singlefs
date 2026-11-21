use super::*;

pub(super) fn parse(source: &str) -> ParsedRustFile {
    let switches = RuleSwitches::every_rule_intact();
    parse_rust_file(&RustFileInput {
        file: FileIndex(0),
        source,
        file_directory: Path::new("/nonexistent"),
        manifest_directory: None,
        repository_root: Path::new("/nonexistent"),
        switches: &switches,
    })
}

pub(super) fn kinds_and_names(parsed: &ParsedRustFile) -> Vec<(UnitKind, String)> {
    parsed
        .units
        .iter()
        .map(|unit| (unit.kind, unit.name.clone()))
        .collect()
}

#[test]
fn items_members_and_heads_become_separate_units() {
    let parsed = parse(
        "use std::fmt;\npub const WIDTH: usize = 4;\npub struct Money(pub u32);\n\
             impl std::ops::Add for Money { type Output = Money; fn add(self, other: Money) -> Money { Money(self.0 + other.0) } }\n\
             pub trait Limit { const LIMIT: u32; fn doubled(&self) -> u32 { 2 } }\n\
             macro_rules! make { () => {} }\nmake!();\nmod nested { pub fn inside() {} }\nmod declared;\n",
    );
    assert!(
        parsed.file_level_refusals.is_empty(),
        "全都认得：{:?}",
        parsed.file_level_refusals
    );
    assert_eq!(
        kinds_and_names(&parsed),
        [
            (UnitKind::Use, "use@1".to_string()),
            (UnitKind::Constant, "WIDTH".to_string()),
            (UnitKind::Struct, "Money".to_string()),
            (
                UnitKind::ImplementationHead,
                "<impl std::ops::Add for Money>".to_string()
            ),
            (UnitKind::TypeAlias, "Output".to_string()),
            (UnitKind::Function, "add".to_string()),
            (UnitKind::TraitHead, "Limit".to_string()),
            (UnitKind::Constant, "LIMIT".to_string()),
            (UnitKind::Function, "doubled".to_string()),
            (UnitKind::MacroRulesDefinition, "make".to_string()),
            (UnitKind::ItemMacroInvocation, "make!@7".to_string()),
            (UnitKind::ModuleHead, "nested".to_string()),
            (UnitKind::Function, "inside".to_string()),
            (UnitKind::ModuleDeclaration, "declared".to_string()),
        ]
    );
    let implementation = parsed.units[3]
        .implementation
        .as_ref()
        .expect("impl 头有形状");
    assert_eq!(implementation.self_type_names, ["Money"]);
    assert!(implementation.is_trait_implementation);
    assert_eq!(
        parsed.units[5].container,
        Some(UnitIndex(3)),
        "方法挂在它的 impl 头下"
    );
}

#[test]
fn an_unknown_item_start_and_unbalanced_brackets_are_refused_for_the_whole_file() {
    let unknown = parse("pub fn fine() {}\nwhat is this;\n");
    assert_eq!(unknown.file_level_refusals.len(), 1);
    assert_eq!(
        unknown.file_level_refusals[0].form,
        RefusalForm::UnrecognizedItemStart
    );
    let unbalanced = parse("fn broken() { (\n");
    assert_eq!(
        unbalanced.file_level_refusals[0].form,
        RefusalForm::UnbalancedBrackets
    );
}
