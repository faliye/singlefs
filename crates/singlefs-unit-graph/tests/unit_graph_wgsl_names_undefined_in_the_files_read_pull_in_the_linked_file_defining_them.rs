//! WGSL 跨文件着落边（`records/2026-10-13-复用键与全仓改名.md`「crates 第三轮 A 交回」之后主 agent 定的第 2 条，定案 3「多连不少连」）：
//! 闭包读到的 `.wgsl` 里用到、却没定义的名字，链接集里别的单元 `include_str!` 读进来的 `.wgsl` 定义了它的，把那个单元连同那份 `.wgsl`
//! 并进闭包，名字在合起来的名字空间里解；哪份都没定义的才拒算（`wgsl-undefined-name`）。真仓的形状：`crash_judge_checker_common.wgsl`
//! 的常量引 `crash_judge_common.wgsl` 里定义的 `MAX_ROWS` 这类常量，两份由 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 的
//! `checker_walk_kernel_source` 拼成一份着色器，只把前一份当文本读的测试闭包里没有后一份。
//! 夹具：`fsplitkernel` 用两个 `include_str!` 读两份 `.wgsl`，`checker.wgsl` 的 `const_assert` 引 `common.wgsl` 的 `SCRATCH_BASE`；
//! 谁都不依赖的 `funlinkedcommon` 里另有一份定义同名常量的 `.wgsl`。

mod common_fixture_workspace;

use common_fixture_workspace::{
    closure_with_triple, key_of, workspace_of, EntrySpecification, FixtureTree, HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::rule_switches::RuleSwitches;

const SPLIT_KERNEL_FIXTURE: [(&str, &str); 8] = [
    (
        "crates/fsplitkernel/Cargo.toml",
        include_str!("fixtures/wgsl_split_kernel/fsplitkernel/Cargo.toml.fixture"),
    ),
    (
        "crates/fsplitkernel/src/lib.rs",
        include_str!("fixtures/wgsl_split_kernel/fsplitkernel/src/lib.rs.fixture"),
    ),
    (
        "crates/fsplitkernel/src/common.wgsl",
        include_str!("fixtures/wgsl_split_kernel/fsplitkernel/src/common.wgsl.fixture"),
    ),
    (
        "crates/fsplitkernel/src/checker.wgsl",
        include_str!("fixtures/wgsl_split_kernel/fsplitkernel/src/checker.wgsl.fixture"),
    ),
    (
        "crates/funlinkedcommon/Cargo.toml",
        include_str!("fixtures/wgsl_split_kernel/funlinkedcommon/Cargo.toml.fixture"),
    ),
    (
        "crates/funlinkedcommon/src/lib.rs",
        include_str!("fixtures/wgsl_split_kernel/funlinkedcommon/src/lib.rs.fixture"),
    ),
    (
        "crates/funlinkedcommon/src/common.wgsl",
        include_str!("fixtures/wgsl_split_kernel/funlinkedcommon/src/common.wgsl.fixture"),
    ),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

const COMMON_WGSL_FILE: &str = "crates/fsplitkernel/src/common.wgsl";

/// 读两份 `.wgsl`、拼成一份内核的入口。
const BOTH_FILES_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fsplitkernel::kernel_text",
    target: "lib",
    conditions: &[],
};

/// 只读断言那一份 `.wgsl` 的入口。
const CHECKER_FILE_ONLY_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fsplitkernel::checker_text_alone",
    target: "lib",
    conditions: &[],
};

/// 入口闭包里的（Rust 单元的限定名，WGSL 单元的「名字@文件」）。
fn closure_contents(tree: &FixtureTree, entry: &EntrySpecification) -> (Vec<String>, Vec<String>) {
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_with_triple(&workspace, entry, &switches, HOST_TARGET_TRIPLE)
        .unwrap_or_else(|error| panic!("{} 的闭包收得起来：{error}", entry.entry_path));
    let rust_units = outcome
        .units
        .iter()
        .map(|unit_index| workspace.units[unit_index.0].qualified_name.clone())
        .collect();
    let wgsl_units = outcome
        .wgsl_units
        .iter()
        .map(|wgsl_index| {
            let wgsl_unit = &workspace.wgsl_units[wgsl_index.0];
            format!(
                "{}@{}",
                wgsl_unit.name, workspace.files[wgsl_unit.file.0].relative_path
            )
        })
        .collect();
    (rust_units, wgsl_units)
}

#[test]
fn a_constant_assertion_naming_a_constant_of_another_included_wgsl_file_is_not_refused() {
    let switches = RuleSwitches::every_rule_intact();
    let tree = FixtureTree::materialize("wgsl-split-kernel", &SPLIT_KERNEL_FIXTURE);
    let verdict = key_of(
        &workspace_of(tree.root(), &switches),
        &BOTH_FILES_ENTRY,
        &switches,
    );
    assert!(
        verdict.is_ok(),
        "checker.wgsl 的 const_assert 引的 SCRATCH_BASE 定义在同一个闭包读到的 common.wgsl 里，不是外名：{verdict:?}"
    );
}

#[test]
fn changing_the_constant_another_wgsl_file_defines_for_the_assertion_changes_the_key() {
    let switches = RuleSwitches::every_rule_intact();
    let base = FixtureTree::materialize("wgsl-split-kernel-base", &SPLIT_KERNEL_FIXTURE);
    let variant = FixtureTree::materialize("wgsl-split-kernel-variant", &SPLIT_KERNEL_FIXTURE);
    variant.replace_exactly_once(
        COMMON_WGSL_FILE,
        "const SCRATCH_BASE: u32 = 16u;",
        "const SCRATCH_BASE: u32 = 32u;",
    );
    let before = key_of(
        &workspace_of(base.root(), &switches),
        &BOTH_FILES_ENTRY,
        &switches,
    )
    .expect("改前不拒算");
    let after = key_of(
        &workspace_of(variant.root(), &switches),
        &BOTH_FILES_ENTRY,
        &switches,
    )
    .expect("改后不拒算");
    assert_ne!(
        before, after,
        "断言经名字收到 common.wgsl 的 SCRATCH_BASE：改它的值键变"
    );
}

#[test]
fn reading_only_the_asserting_file_pulls_in_the_defining_file_and_the_rust_unit_including_it() {
    let switches = RuleSwitches::every_rule_intact();
    let tree = FixtureTree::materialize("wgsl-split-kernel-checker-only", &SPLIT_KERNEL_FIXTURE);
    let verdict = key_of(
        &workspace_of(tree.root(), &switches),
        &CHECKER_FILE_ONLY_ENTRY,
        &switches,
    );
    assert!(
        verdict.is_ok(),
        "闭包只读 checker.wgsl，SCRATCH_BASE 由链接集里 include_str! 读进的 common.wgsl 定义：拉进来、不拒算：{verdict:?}"
    );
    let (rust_units, wgsl_units) = closure_contents(&tree, &CHECKER_FILE_ONLY_ENTRY);
    assert!(
        rust_units
            .iter()
            .any(|name| name == "fsplitkernel::COMMON_SOURCE"),
        "读 common.wgsl 的 COMMON_SOURCE 经着落边进闭包：{rust_units:?}"
    );
    assert!(
        wgsl_units
            .iter()
            .any(|name| name == "SCRATCH_BASE@crates/fsplitkernel/src/common.wgsl"),
        "common.wgsl 的 SCRATCH_BASE 进闭包：{wgsl_units:?}"
    );
    assert!(
        !rust_units
            .iter()
            .chain(&wgsl_units)
            .any(|name| name.contains("funlinkedcommon")),
        "链接集外的 funlinkedcommon 同名常量不进闭包：{rust_units:?} {wgsl_units:?}"
    );
}

#[test]
fn changing_the_constant_in_the_pulled_in_file_changes_the_key_of_the_entry_reading_only_the_asserting_file(
) {
    let switches = RuleSwitches::every_rule_intact();
    let base = FixtureTree::materialize("wgsl-split-kernel-pulled-base", &SPLIT_KERNEL_FIXTURE);
    let variant =
        FixtureTree::materialize("wgsl-split-kernel-pulled-variant", &SPLIT_KERNEL_FIXTURE);
    variant.replace_exactly_once(
        COMMON_WGSL_FILE,
        "const SCRATCH_BASE: u32 = 16u;",
        "const SCRATCH_BASE: u32 = 32u;",
    );
    let before = key_of(
        &workspace_of(base.root(), &switches),
        &CHECKER_FILE_ONLY_ENTRY,
        &switches,
    )
    .expect("改前不拒算");
    let after = key_of(
        &workspace_of(variant.root(), &switches),
        &CHECKER_FILE_ONLY_ENTRY,
        &switches,
    )
    .expect("改后不拒算");
    assert_ne!(
        before, after,
        "只读 checker.wgsl 的入口经着落边收到 common.wgsl 的 SCRATCH_BASE：改它的值键变"
    );
}

#[test]
fn a_file_defining_the_constant_in_a_crate_outside_the_link_does_not_supply_it_and_the_name_is_refused(
) {
    let switches = RuleSwitches::every_rule_intact();
    let tree = FixtureTree::materialize("wgsl-split-kernel-unlinked", &SPLIT_KERNEL_FIXTURE);
    tree.replace_exactly_once(
        COMMON_WGSL_FILE,
        "const SCRATCH_BASE: u32 = 16u;",
        "const SCRATCH_START: u32 = 16u;",
    );
    let reason = key_of(
        &workspace_of(tree.root(), &switches),
        &CHECKER_FILE_ONLY_ENTRY,
        &switches,
    )
    .expect_err(
        "链接集里没有哪份 .wgsl 定义 SCRATCH_BASE（只有链接集外的 funlinkedcommon 定义），应当拒算",
    );
    assert!(
        reason.contains(
            "refused\twgsl-undefined-name\tcrates/fsplitkernel/src/checker.wgsl:2\tSCRATCH_BASE"
        ),
        "拒算行点名断言那个单元与名字：{reason}"
    );
}
