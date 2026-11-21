//! 一组入口的键不收工具链行（rustc、cargo、LLVM、目标、RUSTFLAGS）与 `Cargo.lock` 的依赖行（D13（验证路线） 已定项 17）：
//! 验证包、实验包与真卡对拍键包的构建脚本都经 `entry_groups` 算崩溃判定、对拍、观察者那几把键，这一层不带这两样，几把键都跟着不带。
//! 抬闭包真点名的外部包在锁文件里的版本，键不变；闭包里的产品单元改一处，键照变；键就是复用键不带工具链行、不带依赖行的那一把。

mod common_fixture_workspace;

use common_fixture_workspace::{
    closure_with_triple, key_of_outcome, workspace_of, EntrySpecification, FixtureTree,
    HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::{
    entry_group_key, EntryGroupKey, EntryGroupRequest, EntryGroupVerdict, ExternalKeyInputs,
    JudgmentKind, TargetSelector,
};

/// 一个工作区 crate `fconsumer` 依赖外部 crate `alpha`（入口 `run` 经 `use` 点名它），入口另调自己的 `helper`；锁文件里 `alpha` 是 1.0.0。
const LOCKED_DEPENDENCY_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/fconsumer/Cargo.toml",
        "[package]\nname = \"fconsumer\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
         [dependencies]\nalpha = \"1\"\n",
    ),
    (
        "crates/fconsumer/src/lib.rs",
        "use alpha::Engine;\n\
         pub fn run() -> u32 {\n\
         \x20   Engine::start() + helper()\n\
         }\n\
         fn helper() -> u32 {\n\
         \x20   1\n\
         }\n",
    ),
    (
        "Cargo.lock",
        "version = 4\n\n\
         [[package]]\nname = \"alpha\"\nversion = \"1.0.0\"\nsource = \"registry+https://example.invalid/index\"\n\
         checksum = \"a1\"\n\n\
         [[package]]\nname = \"fconsumer\"\nversion = \"0.1.0\"\ndependencies = [\n \"alpha\",\n]\n",
    ),
];

/// 锁文件里 `alpha` 的版本与校验和抬一版。
const LOCK_FILE_VERSION_EDIT: (&str, &str, &str) = (
    "Cargo.lock",
    "version = \"1.0.0\"\nsource = \"registry+https://example.invalid/index\"\nchecksum = \"a1\"",
    "version = \"1.0.1\"\nsource = \"registry+https://example.invalid/index\"\nchecksum = \"a2\"",
);

/// 入口调到的 `helper` 改一个数。
const PRODUCT_UNIT_EDIT: (&str, &str, &str) = ("crates/fconsumer/src/lib.rs", "    1\n", "    2\n");

const RUN_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fconsumer::run",
    target: "lib",
    conditions: &[],
};

/// 照 `edit` 改夹具一处（`None` 不改），交入口 `run` 那一组的键与同一棵树上不带工具链行、不带依赖行的复用键（入口照样、产品行为视图）。
fn entry_group_key_and_the_reuse_key_without_external_lines(
    edit: Option<(&str, &str, &str)>,
) -> (EntryGroupKey, String) {
    let tree = FixtureTree::materialize(
        "entry-group-without-external-lines",
        &LOCKED_DEPENDENCY_FIXTURE,
    );
    if let Some((relative, original, replacement)) = edit {
        tree.replace_exactly_once(relative, original, replacement);
    }
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let group = entry_group_key(
        &workspace,
        &EntryGroupRequest {
            group_name: "fixture_group".to_string(),
            target: TargetSelector::Library,
            entry_paths: vec![RUN_ENTRY.entry_path.to_string()],
            features: Vec::new(),
        },
        HOST_TARGET_TRIPLE,
        &switches,
    )
    .unwrap_or_else(|error| panic!("夹具的入口写对了：{error}"));
    let view = workspace.view_for_judgment(JudgmentKind::ProductBehavior);
    let outcome = closure_with_triple(view, &RUN_ENTRY, &switches, HOST_TARGET_TRIPLE)
        .unwrap_or_else(|error| panic!("夹具的入口找得到：{error}"));
    let reuse_key = key_of_outcome(view, &outcome, &ExternalKeyInputs::default())
        .unwrap_or_else(|reason| panic!("夹具的入口交得出键：{reason}"))
        .hexadecimal();
    (group, reuse_key)
}

fn keyed(group_key: &EntryGroupKey) -> String {
    match &group_key.verdict {
        EntryGroupVerdict::Keyed { key } => key.hexadecimal(),
        EntryGroupVerdict::Refused { refusal_lines } => panic!("不该拒算：{refusal_lines:?}"),
        EntryGroupVerdict::NotCompiled { reason } => panic!("不该不编进来：{reason}"),
    }
}

/// 场景：入口 `run` 经 `use` 点名外部包 `alpha`；锁文件里抬 `alpha` 一版前后各算一次，再改入口调到的产品单元 `helper` 算一次。
/// 预期：抬版本前后那一组的键相同；产品单元改了键变。
/// 判别力（`crates/mutations.tsv`「一组入口的键带上收窄的锁文件依赖行」）：一组的键照锁文件收窄出依赖行带进去，这条红在「不变」那一句。
#[test]
fn the_entry_group_key_stays_when_a_lock_file_version_changes_and_changes_with_a_product_unit() {
    let (before, _) = entry_group_key_and_the_reuse_key_without_external_lines(None);
    let (lock_raised, _) =
        entry_group_key_and_the_reuse_key_without_external_lines(Some(LOCK_FILE_VERSION_EDIT));
    let (unit_edited, _) =
        entry_group_key_and_the_reuse_key_without_external_lines(Some(PRODUCT_UNIT_EDIT));
    assert_eq!(
        keyed(&before),
        keyed(&lock_raised),
        "锁文件里闭包点名的外部包抬一版：一组的键不变（依赖版本不进键）"
    );
    assert_ne!(
        keyed(&before),
        keyed(&unit_edited),
        "闭包里的产品单元改了：一组的键变"
    );
}

/// 场景：同一棵树上，入口 `run` 那一组的键，与入口照样收的闭包不带工具链行、不带依赖行算的复用键；另把一行工具链文字或一行依赖行带进复用键各算一次。
/// 预期：一组的键就是不带两样的那一把；带上一行工具链文字或一行依赖行的复用键与它不同（复用键对这两样敏感，一组的键不带它们）。
/// 判别力（`crates/mutations.tsv`「一组入口的键带上目标那一行工具链」）：一组的键把目标那一行当工具链行带进去，这条红在「就是」那一句。
#[test]
fn the_entry_group_key_is_the_reuse_key_without_a_toolchain_line_or_a_lock_line() {
    let (group, reuse_key_without_external_lines) =
        entry_group_key_and_the_reuse_key_without_external_lines(None);
    assert_eq!(
        keyed(&group),
        reuse_key_without_external_lines,
        "一组的键就是不带工具链行、不带依赖行的复用键"
    );
    let tree = FixtureTree::materialize(
        "entry-group-with-external-lines",
        &LOCKED_DEPENDENCY_FIXTURE,
    );
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let view = workspace.view_for_judgment(JudgmentKind::ProductBehavior);
    let outcome = closure_with_triple(view, &RUN_ENTRY, &switches, HOST_TARGET_TRIPLE)
        .unwrap_or_else(|error| panic!("夹具的入口找得到：{error}"));
    for (external, what) in [
        (
            ExternalKeyInputs {
                toolchain_lines: vec!["rustc 1.98.0 (LLVM version 20.1.0)".to_string()],
                lock_lines: Vec::new(),
            },
            "一行工具链文字",
        ),
        (
            ExternalKeyInputs {
                toolchain_lines: Vec::new(),
                lock_lines: vec![
                    "alpha\t1.0.0\tregistry+https://example.invalid/index\ta1".to_string()
                ],
            },
            "一行依赖行",
        ),
    ] {
        let with_the_line = key_of_outcome(view, &outcome, &external)
            .unwrap_or_else(|reason| panic!("夹具的入口交得出键：{reason}"))
            .hexadecimal();
        assert_ne!(
            keyed(&group),
            with_the_line,
            "带上{what}的复用键与一组的键不同：一组的键不带它"
        );
    }
}
