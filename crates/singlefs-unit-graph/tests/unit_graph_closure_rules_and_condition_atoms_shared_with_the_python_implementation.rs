//! 两份实现归并的 Rust 侧（`records/2026-10-13-复用键与全仓改名.md`「2026-10-15 验证段第一步判红、两条定案、4-3 四批派出」定案 2、3；
//! 4-3d 报告第三节 R1–R4）：夹具与 `scripts/gate/lib_function_units.py` 的 `CFG_ATOMS_FIXTURE`、`USE_SELF_FIXTURE`、`OPEN_TRAIT_ARGUMENT_FIXTURE`、
//! `BLANKET_FIXTURE` 同形，每条断言与那边同名的格同一个结论；`const _` 那一条与那边的 `UNDERSCORE_FIXTURE`、`underscore_const_cell` 同形
//! （crates 红批报告「OUT_DIR 里 judging_units_keys.tsv 默认构建对 gpu 构建」：gpu 构建下 `const _` 被 R-裸名拉进 CPU 组闭包）。

mod common_fixture_workspace;

use common_fixture_workspace::{
    closure_with_triple, key_of, workspace_of, EntrySpecification, FixtureTree,
    CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE, HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::units::UnitKind;
use singlefs_unit_graph::Workspace;

const CONDITION_ATOMS_FILE: &str = "crates/fcfg/src/lib.rs";
const OPEN_TRAIT_FILE: &str = "crates/fopen/src/lib.rs";
const BLANKET_FILE: &str = "crates/fblanket/src/lib.rs";
const USE_SELF_FILE: &str = "crates/fuse/src/lib.rs";
const UNDERSCORE_FILE: &str = "crates/funderscore/src/lib.rs";

const fn library_entry(
    entry_path: &'static str,
    conditions: &'static [&'static str],
) -> EntrySpecification {
    EntrySpecification {
        entry_path,
        target: "lib",
        conditions,
    }
}

const WITHOUT_THE_GPU_FEATURE: &[&str] = &[];
const WITH_THE_GPU_FEATURE: &[&str] = &["feature=\"gpu\""];

fn key_or_panic(
    workspace: &Workspace,
    entry: &EntrySpecification,
) -> singlefs_unit_graph::sha256::Sha256Digest {
    key_of(workspace, entry, &RuleSwitches::every_rule_intact())
        .unwrap_or_else(|reason| panic!("{} 要算得出键：{reason}", entry.entry_path))
}

/// 入口的闭包里每个单元的（种类，限定名）。
fn closure_units(workspace: &Workspace, entry: &EntrySpecification) -> Vec<(UnitKind, String)> {
    let outcome = closure_with_triple(
        workspace,
        entry,
        &RuleSwitches::every_rule_intact(),
        HOST_TARGET_TRIPLE,
    )
    .unwrap_or_else(|error| panic!("{} 的闭包收得起来：{error}", entry.entry_path));
    outcome
        .units
        .iter()
        .map(|unit_index| {
            let unit = &workspace.units[unit_index.0];
            (unit.kind, unit.qualified_name.clone())
        })
        .collect()
}

/// R1：闭包里没有一个单元引到 `gpu`（`plain_entry` 与它调的 `helper`），带不带 `gpu` 特性两种目标下键相同。
#[test]
fn a_closure_that_never_mentions_gpu_keeps_its_key_with_and_without_the_gpu_feature() {
    let tree =
        FixtureTree::materialize("condition-atoms", &CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE);
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    assert_eq!(
        key_or_panic(
            &workspace,
            &library_entry("fcfg::plain_entry", WITHOUT_THE_GPU_FEATURE)
        ),
        key_or_panic(
            &workspace,
            &library_entry("fcfg::plain_entry", WITH_THE_GPU_FEATURE)
        ),
        "{CONDITION_ATOMS_FILE} 的 plain_entry 闭包不提 gpu：两种目标同一把键"
    );
}

/// R1：闭包里的单元经函数体里的 `cfg!`、语句上的 `#[cfg]`、入口自己的 `#[cfg_attr]` 引到 `gpu`，带不带 `gpu` 特性两把键不同。
#[test]
fn a_closure_mentioning_gpu_through_a_cfg_macro_a_statement_cfg_or_a_cfg_attr_changes_its_key_with_the_gpu_feature(
) {
    let tree =
        FixtureTree::materialize("condition-atoms", &CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE);
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    for entry_path in [
        "fcfg::macro_entry",
        "fcfg::statement_entry",
        "fcfg::attribute_only_entry",
    ] {
        assert_ne!(
            key_or_panic(
                &workspace,
                &library_entry(entry_path, WITHOUT_THE_GPU_FEATURE)
            ),
            key_or_panic(&workspace, &library_entry(entry_path, WITH_THE_GPU_FEATURE)),
            "{entry_path} 的闭包引到 gpu：带不带 gpu 两把键不同"
        );
    }
}

/// R2：`use crate::store::{self}` 引进名字 `store`：只写 `store::open()` 的 `reader` 连上这条 use，改这条 use 键变。
#[test]
fn a_use_with_self_in_braces_imports_the_module_name_and_editing_it_changes_the_key() {
    let entry = library_entry("fuse::user::reader", WITHOUT_THE_GPU_FEATURE);
    let base = FixtureTree::materialize("use-self", &CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE);
    let base_workspace = workspace_of(base.root(), &RuleSwitches::every_rule_intact());
    let uses: Vec<String> = closure_units(&base_workspace, &entry)
        .into_iter()
        .filter(|(kind, _name)| *kind == UnitKind::Use)
        .map(|(_kind, name)| name)
        .collect();
    assert_eq!(uses.len(), 1, "reader 的闭包里恰好一条 use：{uses:?}");
    let edited = FixtureTree::materialize("use-self", &CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE);
    edited.replace_exactly_once(
        USE_SELF_FILE,
        "use crate::store::{self};",
        "use crate::store::{self, open};",
    );
    let edited_workspace = workspace_of(edited.root(), &RuleSwitches::every_rule_intact());
    assert_ne!(
        key_or_panic(&base_workspace, &entry),
        key_or_panic(&edited_workspace, &entry),
        "改那条 use 键变"
    );
}

/// R3：blanket impl（`impl<T: Copy> Shout for T`）够得到的方法算边：`yell` 调 `value.shout()`，闭包里有 blanket impl 的 `shout`，改它键变。
#[test]
fn a_blanket_implementation_method_is_in_the_closure_and_editing_it_changes_the_key() {
    let entry = library_entry("fblanket::yell", WITHOUT_THE_GPU_FEATURE);
    let base = FixtureTree::materialize("blanket", &CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE);
    let base_workspace = workspace_of(base.root(), &RuleSwitches::every_rule_intact());
    let implementation_heads = closure_units(&base_workspace, &entry)
        .into_iter()
        .filter(|(kind, _name)| *kind == UnitKind::ImplementationHead)
        .count();
    assert_eq!(implementation_heads, 1, "yell 的闭包里有 blanket impl 的头");
    let edited = FixtureTree::materialize("blanket", &CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE);
    edited.replace_exactly_once(BLANKET_FILE, "        6\n", "        7\n");
    let edited_workspace = workspace_of(edited.root(), &RuleSwitches::every_rule_intact());
    assert_ne!(
        key_or_panic(&base_workspace, &entry),
        key_or_panic(&edited_workspace, &entry),
        "改 blanket impl 的 shout 键变"
    );
}

/// R4：`dyn Sink` 只带 `Sink` 自己的 impl（`impl Sink for Meter`，改它键变），不带只在 trait 参数里写了 `Sink` 的
/// `impl Wrap<dyn Sink> for Holder`（改它键不变）。
#[test]
fn a_dyn_trait_reaches_only_implementations_of_that_trait_and_not_ones_naming_it_in_their_trait_arguments(
) {
    let entry = library_entry("fopen::feed", WITHOUT_THE_GPU_FEATURE);
    let base = FixtureTree::materialize("open-trait", &CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE);
    let base_workspace = workspace_of(base.root(), &RuleSwitches::every_rule_intact());
    let holder_units: Vec<String> = closure_units(&base_workspace, &entry)
        .into_iter()
        .map(|(_kind, name)| name)
        .filter(|name| name.contains("Holder"))
        .collect();
    assert!(
        holder_units.is_empty(),
        "feed 的闭包里没有 Holder：{holder_units:?}"
    );
    let base_key = key_or_panic(&base_workspace, &entry);
    let holder_edited =
        FixtureTree::materialize("open-trait", &CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE);
    holder_edited.replace_exactly_once(OPEN_TRAIT_FILE, "        2\n", "        3\n");
    assert_eq!(
        key_or_panic(
            &workspace_of(holder_edited.root(), &RuleSwitches::every_rule_intact()),
            &entry
        ),
        base_key,
        "改 Holder 的 wrap 键不变"
    );
    let meter_edited =
        FixtureTree::materialize("open-trait", &CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE);
    meter_edited.replace_exactly_once(OPEN_TRAIT_FILE, "        1\n", "        4\n");
    assert_ne!(
        key_or_panic(
            &workspace_of(meter_edited.root(), &RuleSwitches::every_rule_intact()),
            &entry
        ),
        base_key,
        "改 Meter 的 level 键变"
    );
}

/// `const _` 没有名字、不是 R-裸名的着落：`pick` 里 `let _ = value;` 与 `_ =>` 的 `_` 不把只在 `gpu` 特性下编的 `const _` 拉进闭包——
/// 带 `gpu` 特性时闭包里没有名字是 `_` 的单元，带不带 `gpu` 两种目标下 `pick` 同一把键。
/// 判别力（`crates/mutations.tsv`）：建按裸名找的索引时照收名字 `_`，这条红。
#[test]
fn an_unnamed_const_is_not_reached_through_an_underscore_and_the_key_stays_with_and_without_the_gpu_feature(
) {
    let tree = FixtureTree::materialize("underscore", &CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE);
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    let entry_with_gpu = library_entry("funderscore::pick", WITH_THE_GPU_FEATURE);
    let outcome_with_gpu = closure_with_triple(
        &workspace,
        &entry_with_gpu,
        &RuleSwitches::every_rule_intact(),
        HOST_TARGET_TRIPLE,
    )
    .unwrap_or_else(|error| panic!("funderscore::pick 的闭包收得起来：{error}"));
    let unnamed_units_in_the_closure: Vec<String> = outcome_with_gpu
        .units
        .iter()
        .map(|unit_index| &workspace.units[unit_index.0])
        .filter(|unit| unit.name == "_")
        .map(|unit| format!("{:?} {}", unit.kind, unit.qualified_name))
        .collect();
    assert!(
        unnamed_units_in_the_closure.is_empty(),
        "{UNDERSCORE_FILE} 的 pick 带 gpu 特性时闭包里不该有 const _：{unnamed_units_in_the_closure:?}"
    );
    assert_eq!(
        key_or_panic(
            &workspace,
            &library_entry("funderscore::pick", WITHOUT_THE_GPU_FEATURE)
        ),
        key_or_panic(&workspace, &entry_with_gpu),
        "{UNDERSCORE_FILE} 的 pick 闭包不提 gpu：两种目标同一把键"
    );
}
