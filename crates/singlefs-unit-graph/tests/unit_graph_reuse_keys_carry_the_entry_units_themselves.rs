//! 复用键带入口单元自己（键里的 entry 行，与 `scripts/gate/lib_function_units.py` 的 `entry_key_lines` 同一个口径）：
//! 两个入口互相收进对方的闭包时，闭包的单元多重集一模一样，键仍各是各的，判定不互相顶替；同一个入口算两趟，键逐字相同。
//! 夹具与 `lib_function_units.py` 的 `ENTRY_UNIT_KEY_FIXTURE` 同一个样子：互相调用的两个函数，一个 trait impl 里的两个方法。

mod common_fixture_workspace;

use common_fixture_workspace::{
    closure_with_triple, key_of_outcome, workspace_of, EntrySpecification, FixtureTree,
    HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::sha256::Sha256Digest;
use singlefs_unit_graph::{ClosureOutcome, ExternalKeyInputs, Workspace};

/// 一个 crate：`ping` 与 `pong` 互相调用；`SampleDevice` 的 `SampleStore` impl 里有 `fill` 与 `width` 两个方法
/// （方法连 impl 头，impl 头点名 `SampleDevice`，类型照 R-隐式 收它的 trait impl 全部成员）。
const SIBLING_ENTRIES_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/fsiblings/Cargo.toml",
        "[package]\nname = \"fsiblings\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        "crates/fsiblings/src/lib.rs",
        "pub fn ping(count: u64) -> u64 {\n\
         \x20   if count > 0 {\n\
         \x20       pong(count - 1)\n\
         \x20   } else {\n\
         \x20       0\n\
         \x20   }\n\
         }\n\
         \n\
         pub fn pong(count: u64) -> u64 {\n\
         \x20   if count > 0 {\n\
         \x20       ping(count - 1)\n\
         \x20   } else {\n\
         \x20       1\n\
         \x20   }\n\
         }\n\
         \n\
         pub trait SampleStore {\n\
         \x20   fn fill(&self, count: u64) -> u64;\n\
         \x20   fn width(&self) -> u64;\n\
         }\n\
         \n\
         pub struct SampleDevice {\n\
         \x20   pub width: u64,\n\
         }\n\
         \n\
         impl SampleStore for SampleDevice {\n\
         \x20   fn fill(&self, count: u64) -> u64 {\n\
         \x20       count.min(self.width)\n\
         \x20   }\n\
         \n\
         \x20   fn width(&self) -> u64 {\n\
         \x20       self.width\n\
         \x20   }\n\
         }\n",
    ),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

const PING_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fsiblings::ping",
    target: "lib",
    conditions: &[],
};

const PONG_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fsiblings::pong",
    target: "lib",
    conditions: &[],
};

const FILL_METHOD_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fsiblings::SampleDevice::fill",
    target: "lib",
    conditions: &[],
};

const WIDTH_METHOD_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fsiblings::SampleDevice::width",
    target: "lib",
    conditions: &[],
};

fn closure_of(workspace: &Workspace, entry: &EntrySpecification) -> ClosureOutcome {
    closure_with_triple(
        workspace,
        entry,
        &RuleSwitches::every_rule_intact(),
        HOST_TARGET_TRIPLE,
    )
    .unwrap_or_else(|error| panic!("夹具里找得到入口 {}：{error}", entry.entry_path))
}

fn key_of_closure(workspace: &Workspace, outcome: &ClosureOutcome) -> Sha256Digest {
    key_of_outcome(workspace, outcome, &ExternalKeyInputs::default())
        .unwrap_or_else(|refusal| panic!("夹具闭包里没有拒算：{refusal}"))
}

/// 闭包的单元多重集：每个单元（种类，指纹），排好序；不带名字、路径与行号，与键里的 unit 行同一个口径。
fn kinds_and_fingerprints_of_the_closure(
    workspace: &Workspace,
    outcome: &ClosureOutcome,
) -> Vec<String> {
    let mut lines: Vec<String> = outcome
        .units
        .iter()
        .map(|unit_index| {
            let unit = &workspace.units[unit_index.0];
            format!(
                "{}\t{}",
                unit.kind.output_name(),
                unit.fingerprint.hexadecimal()
            )
        })
        .collect();
    lines.sort();
    lines
}

/// 两个入口先核夹具造对了（两个闭包的单元多重集相同、没有 WGSL 与数据单元），再核两把键不同。
fn assert_entries_close_over_each_other_and_keys_differ(
    first_entry: &EntrySpecification,
    second_entry: &EntrySpecification,
) {
    let tree = FixtureTree::materialize("reuse-key-entry-units", &SIBLING_ENTRIES_FIXTURE);
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    let first_closure = closure_of(&workspace, first_entry);
    let second_closure = closure_of(&workspace, second_entry);
    let first_units = kinds_and_fingerprints_of_the_closure(&workspace, &first_closure);
    let second_units = kinds_and_fingerprints_of_the_closure(&workspace, &second_closure);
    assert_eq!(
        first_units, second_units,
        "夹具要造出互相收进对方闭包的样子：{} 与 {} 的闭包单元多重集相同",
        first_entry.entry_path, second_entry.entry_path
    );
    assert!(
        first_closure.wgsl_units.is_empty()
            && second_closure.wgsl_units.is_empty()
            && first_closure.data_units.is_empty()
            && second_closure.data_units.is_empty(),
        "夹具的闭包里没有 WGSL 单元与数据单元：两个闭包只差入口是谁"
    );
    assert_ne!(
        key_of_closure(&workspace, &first_closure),
        key_of_closure(&workspace, &second_closure),
        "{} 与 {} 闭包相同，键里有入口单元自己（entry 行），两把键不同",
        first_entry.entry_path,
        second_entry.entry_path
    );
}

#[test]
fn two_functions_calling_each_other_have_the_same_closure_and_different_keys() {
    assert_entries_close_over_each_other_and_keys_differ(&PING_ENTRY, &PONG_ENTRY);
}

#[test]
fn two_methods_of_one_trait_implementation_have_the_same_closure_and_different_keys() {
    assert_entries_close_over_each_other_and_keys_differ(&FILL_METHOD_ENTRY, &WIDTH_METHOD_ENTRY);
}

/// 同一棵树装两趟工作区（两份同时在），同一个入口各算一次键：逐字相同（entry 行只取种类与指纹，不取下标与地址）。
#[test]
fn the_same_entry_keyed_on_two_loads_of_one_tree_gets_the_same_key() {
    let tree = FixtureTree::materialize("reuse-key-entry-units-twice", &SIBLING_ENTRIES_FIXTURE);
    let switches = RuleSwitches::every_rule_intact();
    let first_load = workspace_of(tree.root(), &switches);
    let second_load = workspace_of(tree.root(), &switches);
    let key_on_the_first_load = key_of_closure(&first_load, &closure_of(&first_load, &PING_ENTRY));
    let key_on_the_second_load =
        key_of_closure(&second_load, &closure_of(&second_load, &PING_ENTRY));
    assert_eq!(
        key_on_the_first_load, key_on_the_second_load,
        "同一个入口在同一棵树上算两趟，键逐字相同"
    );
}
