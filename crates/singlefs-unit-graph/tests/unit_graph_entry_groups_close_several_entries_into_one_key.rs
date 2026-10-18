//! 一组入口（`singlefs-checker-tier` 的构建脚本按判器入口逐组算键用的那一层）：几个入口一起收一个闭包，组里任一入口够得着的单元改了键就变；
//! 只在关着的特性下编进来的入口交「不编进来」，不是拒算；闭包里有拒算的交拒算行；写成表再读回逐项相同。

mod common_fixture_workspace;

use common_fixture_workspace::{workspace_of, FixtureTree, HOST_TARGET_TRIPLE};
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::{
    compute_closure_of_entries, entry_group_key, entry_group_table_text, parse_entry_group_table,
    ClosureRequest, EntryError, EntryGroupKey, EntryGroupRequest, EntryGroupVerdict,
    ReadBackEntryGroupVerdict, TargetSelector,
};

/// 一个 crate：`alpha` 调 `shared`，`beta` 调 `other`；`gpu_only` 只在特性 `gpu` 下编进来；`starts_itself` 读自己的可执行文件路径（拒算）。
/// 另一个 crate `fneighbour` 给「不在同一个 crate」那一格用。
const GROUP_FIXTURE: [(&str, &str); 5] = [
    (
        "crates/fgroup/Cargo.toml",
        "[package]\nname = \"fgroup\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\ngpu = []\n",
    ),
    (
        "crates/fgroup/src/lib.rs",
        "fn shared() -> u32 {\n\
         \x20   1\n\
         }\n\
         fn other() -> u32 {\n\
         \x20   2\n\
         }\n\
         pub fn alpha() -> u32 {\n\
         \x20   shared()\n\
         }\n\
         pub fn beta() -> u32 {\n\
         \x20   other()\n\
         }\n\
         #[cfg(feature = \"gpu\")]\n\
         pub fn gpu_only() -> u32 {\n\
         \x20   3\n\
         }\n\
         pub fn starts_itself() -> bool {\n\
         \x20   std::env::current_exe().is_ok()\n\
         }\n",
    ),
    (
        "crates/fneighbour/Cargo.toml",
        "[package]\nname = \"fneighbour\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    ("crates/fneighbour/src/lib.rs", "pub fn far() -> u32 {\n    4\n}\n"),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

const TOOLCHAIN_LINE: &str = "host: x86_64-unknown-linux-gnu";

fn group(entry_paths: &[&str], features: &[&str]) -> EntryGroupRequest {
    EntryGroupRequest {
        group_name: "fixture_group".to_string(),
        target: TargetSelector::Library,
        entry_paths: entry_paths.iter().map(|path| (*path).to_string()).collect(),
        features: features
            .iter()
            .map(|feature| (*feature).to_string())
            .collect(),
    }
}

/// 在夹具上（先把 `other` 的函数体改一处，`None` 不改）算一组。
fn group_key_with(
    entry_paths: &[&str],
    features: &[&str],
    other_body_change: Option<(&str, &str)>,
) -> Result<EntryGroupKey, EntryError> {
    let tree = FixtureTree::materialize("entry-group", &GROUP_FIXTURE);
    if let Some((original, replacement)) = other_body_change {
        tree.replace_exactly_once("crates/fgroup/src/lib.rs", original, replacement);
    }
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    entry_group_key(
        &workspace,
        &group(entry_paths, features),
        &[TOOLCHAIN_LINE.to_string()],
        &[],
        &switches,
    )
}

fn keyed(group_key: &EntryGroupKey) -> String {
    match &group_key.verdict {
        EntryGroupVerdict::Keyed { key } => key.hexadecimal(),
        EntryGroupVerdict::Refused { refusal_lines } => panic!("不该拒算：{refusal_lines:?}"),
        EntryGroupVerdict::NotCompiled { reason } => panic!("不该不编进来：{reason}"),
    }
}

const OTHER_BODY_CHANGE: (&str, &str) = (
    "fn other() -> u32 {\n    2\n",
    "fn other() -> u32 {\n    5\n",
);

#[test]
fn a_unit_only_the_second_entry_reaches_changes_the_group_key_but_not_the_key_of_the_first_entry_alone(
) {
    let group_before =
        keyed(&group_key_with(&["fgroup::alpha", "fgroup::beta"], &[], None).expect("入口找得到"));
    let group_after = keyed(
        &group_key_with(
            &["fgroup::alpha", "fgroup::beta"],
            &[],
            Some(OTHER_BODY_CHANGE),
        )
        .expect("入口找得到"),
    );
    assert_ne!(
        group_before, group_after,
        "other 只有 beta 够得着；组里有 beta，键变"
    );
    let alone_before = keyed(&group_key_with(&["fgroup::alpha"], &[], None).expect("入口找得到"));
    let alone_after = keyed(
        &group_key_with(&["fgroup::alpha"], &[], Some(OTHER_BODY_CHANGE)).expect("入口找得到"),
    );
    assert_eq!(
        alone_before, alone_after,
        "alpha 一个入口够不着 other，键不变"
    );
}

#[test]
fn the_composition_lists_every_unit_the_group_reaches() {
    let group_key =
        group_key_with(&["fgroup::alpha", "fgroup::beta"], &[], None).expect("入口找得到");
    let names: Vec<&str> = group_key
        .composition_lines
        .iter()
        .filter_map(|line| line.split('\t').nth(1))
        .collect();
    for wanted in [
        "fgroup::alpha",
        "fgroup::beta",
        "fgroup::shared",
        "fgroup::other",
    ] {
        assert!(
            names.contains(&wanted),
            "组成清单里要有 {wanted}：{names:?}"
        );
    }
    assert!(
        !names.contains(&"fgroup::starts_itself"),
        "够不着的不在组成清单里：{names:?}"
    );
}

#[test]
fn an_entry_under_a_feature_that_is_off_is_not_compiled_and_with_the_feature_it_is_keyed() {
    let off = group_key_with(&["fgroup::gpu_only"], &[], None).expect("入口找得到");
    assert!(
        matches!(off.verdict, EntryGroupVerdict::NotCompiled { .. }),
        "特性关着：{:?}",
        off.verdict
    );
    let on = group_key_with(&["fgroup::gpu_only"], &["gpu"], None).expect("入口找得到");
    assert!(
        matches!(on.verdict, EntryGroupVerdict::Keyed { .. }),
        "特性开着：{:?}",
        on.verdict
    );
}

#[test]
fn an_entry_not_compiled_under_the_features_is_left_out_and_the_rest_are_keyed() {
    let group_key =
        group_key_with(&["fgroup::alpha", "fgroup::gpu_only"], &[], None).expect("入口找得到");
    assert!(
        matches!(group_key.verdict, EntryGroupVerdict::Keyed { .. }),
        "alpha 编得进来：{:?}",
        group_key.verdict
    );
    assert_eq!(group_key.entries_not_compiled, ["fgroup::gpu_only"]);
    assert_eq!(
        keyed(&group_key),
        keyed(&group_key_with(&["fgroup::alpha"], &[], None).expect("入口找得到")),
        "没收的入口不进键：与只有 alpha 一个入口的那一组同一把"
    );
}

#[test]
fn a_group_whose_closure_has_a_refusal_hands_back_the_refusal_lines() {
    let group_key =
        group_key_with(&["fgroup::alpha", "fgroup::starts_itself"], &[], None).expect("入口找得到");
    match &group_key.verdict {
        EntryGroupVerdict::Refused { refusal_lines } => assert!(
            refusal_lines
                .iter()
                .any(|line| line.starts_with("refused\tcurrent-exe\tcrates/fgroup/src/lib.rs:")),
            "{refusal_lines:?}"
        ),
        EntryGroupVerdict::Keyed { .. } | EntryGroupVerdict::NotCompiled { .. } => {
            panic!("闭包里有 current_exe，应当拒算：{:?}", group_key.verdict)
        }
    }
}

#[test]
fn an_entry_in_another_crate_is_a_usage_error() {
    let tree = FixtureTree::materialize("entry-group-crates", &GROUP_FIXTURE);
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let request = ClosureRequest {
        entry_path: "fgroup::alpha".to_string(),
        target: TargetSelector::Library,
        requested_conditions: Vec::new(),
        uses_default_features: true,
        target_triple: HOST_TARGET_TRIPLE.to_string(),
    };
    let verdict = compute_closure_of_entries(
        &workspace,
        &request,
        &["fneighbour::far".to_string()],
        &switches,
    );
    assert!(
        matches!(verdict, Err(EntryError::EntryInAnotherCrate { .. })),
        "{verdict:?}"
    );
}

#[test]
fn the_table_of_the_groups_reads_back_the_same_verdicts_and_compositions() {
    let keyed_group =
        group_key_with(&["fgroup::alpha", "fgroup::beta"], &[], None).expect("入口找得到");
    let refused_alone = group_key_with(&["fgroup::starts_itself"], &[], None).expect("入口找得到");
    let refused_group = EntryGroupKey {
        group_name: "refused_group".to_string(),
        verdict: refused_alone.verdict,
        entries_not_compiled: refused_alone.entries_not_compiled,
        composition_lines: refused_alone.composition_lines,
        watched_files: refused_alone.watched_files,
    };
    let text = entry_group_table_text(
        &[TOOLCHAIN_LINE.to_string()],
        &[keyed_group.clone(), refused_group.clone()],
    );
    let table = parse_entry_group_table(&text).expect("写的表读得回");
    assert_eq!(table.groups.len(), 2);
    let EntryGroupVerdict::Keyed { key } = &keyed_group.verdict else {
        panic!("头一组算得出键");
    };
    assert_eq!(
        table.groups[0].verdict,
        ReadBackEntryGroupVerdict::Keyed { key: key.0 }
    );
    assert_eq!(
        table.groups[0].composition.len(),
        keyed_group.composition_lines.len(),
        "组成清单逐行读回"
    );
    let EntryGroupVerdict::Refused { refusal_lines } = &refused_group.verdict else {
        panic!("另一组拒算");
    };
    assert_eq!(
        table.groups[1].verdict,
        ReadBackEntryGroupVerdict::Refused {
            refusal_lines: refusal_lines.clone()
        }
    );
    assert!(
        table
            .watched_files
            .contains(&"crates/fgroup/src/lib.rs".to_string()),
        "{:?}",
        table.watched_files
    );
}
