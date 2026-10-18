//! WGSL（设计 3.5）：Rust 闭包里 `include_str!` 读到的 `.wgsl` 组成模块，从 `@compute` 入口按名字精确收；
//! 用到而 `.wgsl` 里没定义的名字要在 Rust 闭包里有着落（串字面量里的词、定义的名字以 `_` 加它收尾、首段与其余一段都落得下），
//! 闭包里没有着落的把链接集里供它的单元连同闭包并进来（着落做成边），最后还没有着落的拒算。

mod common_fixture_workspace;

use common_fixture_workspace::{
    key_of, workspace_of, EntrySpecification, FixtureTree, WGSL_KERNEL_FIXTURE,
};
use singlefs_unit_graph::rule_switches::{BrokenRule, RuleSwitches};

const KERNEL_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fkernel::kernel_text",
    target: "lib",
    conditions: &[],
};

fn kernel_key_changes_after(original: &str, replacement: &str) -> bool {
    let switches = RuleSwitches::every_rule_intact();
    let base = FixtureTree::materialize("kernel-base", &WGSL_KERNEL_FIXTURE);
    let variant = FixtureTree::materialize("kernel-variant", &WGSL_KERNEL_FIXTURE);
    variant.replace_exactly_once("crates/fkernel/src/kernel.wgsl", original, replacement);
    let before = key_of(
        &workspace_of(base.root(), &switches),
        &KERNEL_ENTRY,
        &switches,
    )
    .expect("基线不拒算");
    let after = key_of(
        &workspace_of(variant.root(), &switches),
        &KERNEL_ENTRY,
        &switches,
    )
    .expect("改后不拒算");
    before != after
}

#[test]
fn changing_a_function_the_compute_entry_reaches_changes_the_key() {
    assert!(kernel_key_changes_after(
        "value * 2u + WIDTH",
        "value * 3u + WIDTH"
    ));
}

#[test]
fn changing_a_function_no_compute_entry_reaches_keeps_the_key() {
    assert!(!kernel_key_changes_after("value + 7u", "value + 8u"));
}

#[test]
fn changing_only_a_wgsl_comment_keeps_the_key() {
    assert!(!kernel_key_changes_after(
        "// 夹具内核：PRELUDE_OFFSET 由 Rust 侧拼在前面。",
        "// 只改注释。"
    ));
}

#[test]
fn the_compute_entry_and_what_it_reaches_are_listed_as_wgsl_units() {
    let switches = RuleSwitches::every_rule_intact();
    let tree = FixtureTree::materialize("kernel-lines", &WGSL_KERNEL_FIXTURE);
    let workspace = workspace_of(tree.root(), &switches);
    let request = singlefs_unit_graph::ClosureRequest {
        entry_path: KERNEL_ENTRY.entry_path.to_string(),
        target: singlefs_unit_graph::TargetSelector::Library,
        requested_conditions: Vec::new(),
        uses_default_features: true,
        target_triple: common_fixture_workspace::HOST_TARGET_TRIPLE.to_string(),
    };
    let outcome =
        singlefs_unit_graph::compute_closure(&workspace, &request, &switches).expect("入口找得到");
    let names: Vec<&str> = outcome
        .wgsl_units
        .iter()
        .map(|wgsl_index| workspace.wgsl_units[wgsl_index.0].name.as_str())
        .collect();
    assert_eq!(
        names,
        ["WIDTH", "helper", "words", "main"],
        "入口 main 收到 helper、words，helper 收到 WIDTH；够不到的 unreachable_helper 不在"
    );
}

fn kernel_refusals_after_renaming_the_prelude_constant(
    switches: &RuleSwitches,
) -> Result<(), String> {
    let tree = FixtureTree::materialize("kernel-external", &WGSL_KERNEL_FIXTURE);
    tree.replace_exactly_once(
        "crates/fkernel/src/lib.rs",
        "const PRELUDE_OFFSET: u32 = 4u;",
        "const OTHER_OFFSET: u32 = 4u;",
    );
    key_of(
        &workspace_of(tree.root(), switches),
        &KERNEL_ENTRY,
        switches,
    )
    .map(|_| ())
}

#[test]
fn a_wgsl_name_that_no_rust_string_supplies_is_refused() {
    let verdict =
        kernel_refusals_after_renaming_the_prelude_constant(&RuleSwitches::every_rule_intact());
    let reason = verdict.expect_err("Rust 串里没有 PRELUDE_OFFSET 了，应当拒算");
    assert!(
        reason.contains(
            "refused\twgsl-undefined-name\tcrates/fkernel/src/kernel.wgsl:3\tPRELUDE_OFFSET"
        ),
        "拒算行点名 helper 那个单元与名字：{reason}"
    );
}

#[test]
fn breaking_the_wgsl_external_name_check_lets_the_missing_name_through() {
    let verdict =
        kernel_refusals_after_renaming_the_prelude_constant(&RuleSwitches::with_broken(&[
            BrokenRule::WgslExternalNames,
        ]));
    assert!(verdict.is_ok(), "弄坏外名核对之后不再拒算：{verdict:?}");
}

/// 外名由 Rust 现拼的夹具（照 `crash_judge_gpu.rs` 拼内核的两种写法）：表号常量由 `format!("const T_{upper}: …")` 拼、
/// 失败点常量由宏 `stringify!` 出的名字去掉 `GPU_` 拼（宏调用经 `GPU_FAILURE_POINT_NONE` 进闭包、宏定义经宏名跟进来，与真仓同一条路）；第三份内核用一个以 `_` 起头的外名（首段加 `_` 只有一个字符）。
const PREFIXED_EXTERNAL_NAME_FIXTURE: [(&str, &str); 5] = [
    (
        "crates/fprefixed/Cargo.toml",
        "[package]\nname = \"fprefixed\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        "crates/fprefixed/src/lib.rs",
        "pub const TABLE_KERNEL_SOURCE: &str = include_str!(\"tables.wgsl\");\n\
         pub const CODE_TABLE_KERNEL_SOURCE: &str = include_str!(\"failure_points.wgsl\");\n\
         pub const UNDERSCORE_KERNEL_SOURCE: &str = include_str!(\"underscore.wgsl\");\n\
         pub const TABLE_NAMES: [&str; 2] = [\"header\", \"versions\"];\n\
         pub fn table_constants() -> String {\n\
         \x20   let mut constants = String::new();\n\
         \x20   for (index, name) in TABLE_NAMES.iter().enumerate() {\n\
         \x20       let upper = name.to_ascii_uppercase();\n\
         \x20       constants.push_str(&format!(\"const T_{upper}: u32 = {index}u;\\n\"));\n\
         \x20   }\n\
         \x20   constants\n\
         }\n\
         macro_rules! failure_points {\n\
         \x20   ($($constant:ident = $code:literal,)*) => {\n\
         \x20       $(pub const $constant: u32 = $code;)*\n\
         \x20       pub const GPU_FAILURE_POINTS: &[(&str, u32)] = &[$((stringify!($constant), $code),)*];\n\
         \x20   };\n\
         }\n\
         failure_points! {\n\
         \x20   GPU_FAILURE_POINT_NONE = 0,\n\
         \x20   GPU_FAILURE_POINT_UNREADABLE = 1,\n\
         }\n\
         pub fn failure_point_constants() -> String {\n\
         \x20   GPU_FAILURE_POINTS\n\
         \x20       .iter()\n\
         \x20       .map(|(name, code)| format!(\"const {}: u32 = {code}u;\\n\", name.strip_prefix(\"GPU_\").expect(\"都以 GPU_ 起头\")))\n\
         \x20       .collect()\n\
         }\n\
         pub fn underscore_constants() -> String {\n\
         \x20   let upper = \"HEADER\";\n\
         \x20   format!(\"const _{upper}: u32 = 1u;\\n\")\n\
         }\n\
         pub fn table_kernel_text() -> String {\n\
         \x20   format!(\"{}{TABLE_KERNEL_SOURCE}\", table_constants())\n\
         }\n\
         pub fn failure_point_kernel_text() -> String {\n\
         \x20   assert_eq!(GPU_FAILURE_POINT_NONE, 0, \"无失败点的码是 0\");\n\
         \x20   format!(\"{}{CODE_TABLE_KERNEL_SOURCE}\", failure_point_constants())\n\
         }\n\
         pub fn underscore_kernel_text() -> String {\n\
         \x20   format!(\"{}{UNDERSCORE_KERNEL_SOURCE}\", underscore_constants())\n\
         }\n",
    ),
    (
        "crates/fprefixed/src/tables.wgsl",
        "@group(0) @binding(0) var<storage, read_write> words: array<u32>;\n\
         @compute @workgroup_size(64)\n\
         fn main(@builtin(global_invocation_id) id: vec3<u32>) {\n\
         \x20   words[id.x] = T_HEADER + T_VERSIONS;\n\
         }\n",
    ),
    (
        "crates/fprefixed/src/failure_points.wgsl",
        "@group(0) @binding(0) var<storage, read_write> words: array<u32>;\n\
         @compute @workgroup_size(64)\n\
         fn main(@builtin(global_invocation_id) id: vec3<u32>) {\n\
         \x20   words[id.x] = FAILURE_POINT_NONE + FAILURE_POINT_UNREADABLE;\n\
         }\n",
    ),
    (
        "crates/fprefixed/src/underscore.wgsl",
        "@group(0) @binding(0) var<storage, read_write> words: array<u32>;\n\
         @compute @workgroup_size(64)\n\
         fn main(@builtin(global_invocation_id) id: vec3<u32>) {\n\
         \x20   words[id.x] = _HEADER;\n\
         }\n",
    ),
];

fn prefixed_kernel_key(entry_path: &'static str, switches: &RuleSwitches) -> Result<(), String> {
    let tree = FixtureTree::materialize("kernel-prefixed", &PREFIXED_EXTERNAL_NAME_FIXTURE);
    key_of(
        &workspace_of(tree.root(), switches),
        &EntrySpecification {
            entry_path,
            target: "lib",
            conditions: &[],
        },
        switches,
    )
    .map(|_| ())
}

#[test]
fn an_external_name_built_by_format_from_a_prefix_literal_has_a_source() {
    let verdict = prefixed_kernel_key(
        "fprefixed::table_kernel_text",
        &RuleSwitches::every_rule_intact(),
    );
    assert!(
        verdict.is_ok(),
        "T_HEADER、T_VERSIONS 的首段加 _（T_）落在 format! 的串字面量里，不拒算：{verdict:?}"
    );
}

#[test]
fn an_external_name_built_from_identifiers_in_a_macro_body_has_a_source() {
    let verdict = prefixed_kernel_key(
        "fprefixed::failure_point_kernel_text",
        &RuleSwitches::every_rule_intact(),
    );
    assert!(
        verdict.is_ok(),
        "FAILURE_POINT_* 的首段加 _（FAILURE_）落在 macro_rules! failure_points 的宏体里，不拒算：{verdict:?}"
    );
}

#[test]
fn an_external_name_whose_prefix_is_a_single_character_is_still_refused() {
    let reason = prefixed_kernel_key(
        "fprefixed::underscore_kernel_text",
        &RuleSwitches::every_rule_intact(),
    )
    .expect_err("_HEADER 的首段加 _ 只有一个字符，照旧要整名落在串字面量里，它不在");
    assert!(
        reason.contains(
            "refused\twgsl-undefined-name\tcrates/fprefixed/src/underscore.wgsl:2\t_HEADER"
        ),
        "拒算行点名 @compute 入口那个单元（从第 2 行的属性起）与名字：{reason}"
    );
}

#[test]
fn breaking_the_prefix_rule_brings_back_the_whole_name_in_a_string_literal_criterion() {
    let switches = RuleSwitches::with_broken(&[BrokenRule::WgslExternalNamePrefixes]);
    let tables = prefixed_kernel_key("fprefixed::table_kernel_text", &switches)
        .expect_err("只认整名的口径下 T_HEADER 没有着落");
    assert!(
        tables.contains("\tT_HEADER") && tables.contains("\tT_VERSIONS"),
        "{tables}"
    );
    let failure_points = prefixed_kernel_key("fprefixed::failure_point_kernel_text", &switches);
    assert!(
        failure_points.is_ok(),
        "宏调用里定义的 GPU_FAILURE_POINT_* 以 _ 加外名收尾，不靠首段规则也有着落：{failure_points:?}"
    );
}

/// 外名的首段落在一个无关的串里（`"FOO_ 起头的说明串"`），其余那一段（`BAR`）哪儿都没有：不算着落（剪枝复查 U5 发现的碰巧命中）。
const UNRELATED_PREFIX_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/funrelated/Cargo.toml",
        "[package]\nname = \"funrelated\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        "crates/funrelated/src/lib.rs",
        "pub const KERNEL_SOURCE: &str = include_str!(\"kernel.wgsl\");\n\
         pub fn kernel_text() -> String {\n\
         \x20   let note = \"FOO_ 起头的说明串，与内核里的名字无关\";\n\
         \x20   format!(\"// {note}\\n{KERNEL_SOURCE}\")\n\
         }\n",
    ),
    (
        "crates/funrelated/src/kernel.wgsl",
        "@group(0) @binding(0) var<storage, read_write> words: array<u32>;\n\
         @compute @workgroup_size(64)\n\
         fn main(@builtin(global_invocation_id) id: vec3<u32>) {\n\
         \x20   words[id.x] = FOO_BAR;\n\
         }\n",
    ),
];

#[test]
fn an_external_name_whose_prefix_lands_only_in_an_unrelated_string_is_refused() {
    let switches = RuleSwitches::every_rule_intact();
    let tree = FixtureTree::materialize("kernel-unrelated-prefix", &UNRELATED_PREFIX_FIXTURE);
    let reason = key_of(
        &workspace_of(tree.root(), &switches),
        &EntrySpecification {
            entry_path: "funrelated::kernel_text",
            target: "lib",
            conditions: &[],
        },
        &switches,
    )
    .expect_err("FOO_ 落在一个说明串里、BAR 哪儿都没有：没有着落");
    assert!(
        reason
            .contains("refused\twgsl-undefined-name\tcrates/funrelated/src/kernel.wgsl:2\tFOO_BAR"),
        "{reason}"
    );
}

/// 供外名的条目级宏调用不经任何名字点到（拼内核的函数只用宏体里定义的 `GPU_FAILURE_POINTS`，宏体里的名字按 R-宏 解不到调用，
/// 与真仓 `crash_judge_conclusions.rs` 的 `gpu_failure_points!` 同一条路）：靠外名 `FAILURE_POINT_*` 的着落做成边，调用才进闭包。
const SUPPLIER_EDGE_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/fsupplied/Cargo.toml",
        "[package]\nname = \"fsupplied\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        "crates/fsupplied/src/lib.rs",
        "pub const KERNEL_SOURCE: &str = include_str!(\"kernel.wgsl\");\n\
         macro_rules! failure_points {\n\
         \x20   ($($constant:ident = $code:literal,)*) => {\n\
         \x20       $(pub const $constant: u32 = $code;)*\n\
         \x20       pub const GPU_FAILURE_POINTS: &[(&str, u32)] = &[$((stringify!($constant), $code),)*];\n\
         \x20   };\n\
         }\n\
         failure_points! {\n\
         \x20   GPU_FAILURE_POINT_NONE = 0,\n\
         \x20   GPU_FAILURE_POINT_UNREADABLE = 9,\n\
         }\n\
         pub fn failure_point_constants() -> String {\n\
         \x20   GPU_FAILURE_POINTS\n\
         \x20       .iter()\n\
         \x20       .map(|(name, code)| format!(\"const {}: u32 = {code}u;\\n\", name.strip_prefix(\"GPU_\").expect(\"都以 GPU_ 起头\")))\n\
         \x20       .collect()\n\
         }\n\
         pub fn kernel_text() -> String {\n\
         \x20   format!(\"{}{KERNEL_SOURCE}\", failure_point_constants())\n\
         }\n",
    ),
    (
        "crates/fsupplied/src/kernel.wgsl",
        "@group(0) @binding(0) var<storage, read_write> words: array<u32>;\n\
         @compute @workgroup_size(64)\n\
         fn main(@builtin(global_invocation_id) id: vec3<u32>) {\n\
         \x20   words[id.x] = FAILURE_POINT_NONE + FAILURE_POINT_UNREADABLE;\n\
         }\n",
    ),
];

/// 改宏调用里的一个码（9 → 99）前后 `fsupplied::kernel_text` 的键：都交得出时交两把是否不同，有一把拒算交拒算说明。
fn supplied_kernel_key_changes_with_the_code(switches: &RuleSwitches) -> Result<bool, String> {
    let entry = EntrySpecification {
        entry_path: "fsupplied::kernel_text",
        target: "lib",
        conditions: &[],
    };
    let base = FixtureTree::materialize("kernel-supplier-base", &SUPPLIER_EDGE_FIXTURE);
    let variant = FixtureTree::materialize("kernel-supplier-variant", &SUPPLIER_EDGE_FIXTURE);
    variant.replace_exactly_once(
        "crates/fsupplied/src/lib.rs",
        "GPU_FAILURE_POINT_UNREADABLE = 9,",
        "GPU_FAILURE_POINT_UNREADABLE = 99,",
    );
    let before = key_of(&workspace_of(base.root(), switches), &entry, switches)?;
    let after = key_of(&workspace_of(variant.root(), switches), &entry, switches)?;
    Ok(before != after)
}

#[test]
fn the_item_macro_supplying_an_external_name_is_pulled_into_the_closure_and_its_codes_change_the_key(
) {
    assert_eq!(
        supplied_kernel_key_changes_with_the_code(&RuleSwitches::every_rule_intact()),
        Ok(true),
        "宏调用定义的 GPU_FAILURE_POINT_UNREADABLE 以 _ 加外名收尾：着落做成边，调用进闭包，改码键变"
    );
}

#[test]
fn breaking_the_supplier_edges_misses_the_code_change() {
    let verdict = supplied_kernel_key_changes_with_the_code(&RuleSwitches::with_broken(&[
        BrokenRule::WgslSupplierEdges,
    ]));
    assert!(
        verdict != Ok(true),
        "着落不做成边：调用不进闭包，改码要么键不变、要么外名没着落拒算，就是不能照常变：{verdict:?}"
    );
}
