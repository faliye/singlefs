//! 依赖行收窄：整份 `Cargo.lock` 交进来，只取闭包成员真点名的外部 crate 解析到的包与它的传递依赖进键；
//! 抬闭包没引到的外部包的版本，键不变；抬引到的（或它传递依赖的）包的版本，键变；点名的外部 crate 在锁文件里落不到包，拒算。

mod common_fixture_workspace;

use common_fixture_workspace::{
    closure_with_triple, key_of_outcome, workspace_of, EntrySpecification, FixtureTree,
    HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::output::refusal_lines;
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::sha256::Sha256Digest;
use singlefs_unit_graph::{narrowed_lock_lines, parse_lock_file, ExternalKeyInputs};

/// 一个工作区 crate 依赖三个外部 crate：`alpha`（入口经 `use` 点名，它又依赖 `gamma`）、`beta-tools`（只有别的函数点名）、
/// `delta`（锁文件里没有它）。
const LOCKED_DEPENDENCY_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/fconsumer/Cargo.toml",
        "[package]\nname = \"fconsumer\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
         [dependencies]\nalpha = \"1\"\nbeta-tools = \"1\"\ndelta = \"1\"\n",
    ),
    (
        "crates/fconsumer/src/lib.rs",
        "use alpha::Engine;\n\
         pub fn run() -> u32 {\n\
         \x20   Engine::start() + helper()\n\
         }\n\
         fn helper() -> u32 {\n\
         \x20   1\n\
         }\n\
         pub fn uses_beta() -> u32 {\n\
         \x20   beta_tools::value()\n\
         }\n\
         pub fn uses_delta() -> u32 {\n\
         \x20   delta::value()\n\
         }\n",
    ),
    (
        "Cargo.lock",
        "version = 4\n\n\
         [[package]]\nname = \"alpha\"\nversion = \"1.0.0\"\nsource = \"registry+https://example.invalid/index\"\n\
         checksum = \"a1\"\ndependencies = [\n \"gamma\",\n]\n\n\
         [[package]]\nname = \"beta-tools\"\nversion = \"1.0.0\"\nsource = \"registry+https://example.invalid/index\"\n\
         checksum = \"b1\"\n\n\
         [[package]]\nname = \"fconsumer\"\nversion = \"0.1.0\"\ndependencies = [\n \"alpha\",\n \"beta-tools\",\n]\n\n\
         [[package]]\nname = \"gamma\"\nversion = \"1.0.0\"\nsource = \"registry+https://example.invalid/index\"\n\
         checksum = \"c1\"\n",
    ),
];

fn entry(entry_path: &'static str) -> EntrySpecification {
    EntrySpecification {
        entry_path,
        target: "lib",
        conditions: &[],
    }
}

/// 在一份夹具上（先改一处锁文件）算入口的依赖行与键。
fn narrowed_lines_and_key(
    entry_path: &'static str,
    lock_change: Option<(&str, &str)>,
) -> Result<(Vec<String>, Sha256Digest), String> {
    let tree = FixtureTree::materialize("locked-dependencies", &LOCKED_DEPENDENCY_FIXTURE);
    if let Some((original, replacement)) = lock_change {
        tree.replace_exactly_once("Cargo.lock", original, replacement);
    }
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_with_triple(
        &workspace,
        &entry(entry_path),
        &switches,
        HOST_TARGET_TRIPLE,
    )
    .expect("入口找得到");
    let lock_text =
        std::fs::read_to_string(tree.root().join("Cargo.lock")).expect("夹具锁文件读得了");
    let packages = parse_lock_file(&lock_text).expect("夹具锁文件解析得了");
    let lines = narrowed_lock_lines(&workspace, &outcome, &packages).map_err(|refusals| {
        format!("拒算：{}", refusal_lines(&workspace, &refusals).join(" | "))
    })?;
    let key = key_of_outcome(
        &workspace,
        &outcome,
        &ExternalKeyInputs {
            toolchain_lines: Vec::new(),
            lock_lines: lines.clone(),
        },
    )?;
    Ok((lines, key))
}

#[test]
fn only_the_named_external_crate_and_its_transitive_dependency_become_lock_lines() {
    let (lines, _) = narrowed_lines_and_key("fconsumer::run", None).expect("不拒算");
    assert_eq!(
        lines,
        [
            "alpha\t1.0.0\tregistry+https://example.invalid/index\ta1",
            "gamma\t1.0.0\tregistry+https://example.invalid/index\tc1",
        ],
        "入口经 use 点名 alpha，alpha 依赖 gamma；beta-tools 闭包没点名、不进"
    );
}

#[test]
fn raising_the_version_of_an_external_crate_the_closure_does_not_name_keeps_the_key() {
    let (_, base) = narrowed_lines_and_key("fconsumer::run", None).expect("不拒算");
    let (_, raised) = narrowed_lines_and_key(
        "fconsumer::run",
        Some((
            "name = \"beta-tools\"\nversion = \"1.0.0\"",
            "name = \"beta-tools\"\nversion = \"1.0.1\"",
        )),
    )
    .expect("不拒算");
    assert_eq!(base, raised, "beta-tools 不在入口的闭包里，键不变");
}

#[test]
fn raising_the_version_of_the_named_external_crate_changes_the_key() {
    let (_, base) = narrowed_lines_and_key("fconsumer::run", None).expect("不拒算");
    let (_, raised) = narrowed_lines_and_key(
        "fconsumer::run",
        Some((
            "name = \"alpha\"\nversion = \"1.0.0\"",
            "name = \"alpha\"\nversion = \"1.0.1\"",
        )),
    )
    .expect("不拒算");
    assert_ne!(base, raised, "alpha 是入口点名的外部包，抬它的版本键变");
}

#[test]
fn raising_the_version_of_a_transitive_dependency_of_the_named_crate_changes_the_key() {
    let (_, base) = narrowed_lines_and_key("fconsumer::run", None).expect("不拒算");
    let (_, raised) = narrowed_lines_and_key(
        "fconsumer::run",
        Some((
            "name = \"gamma\"\nversion = \"1.0.0\"",
            "name = \"gamma\"\nversion = \"1.0.1\"",
        )),
    )
    .expect("不拒算");
    assert_ne!(base, raised, "gamma 是 alpha 的传递依赖，抬它的版本键变");
}

#[test]
fn a_path_qualified_call_names_its_external_crate_too() {
    let (lines, _) = narrowed_lines_and_key("fconsumer::uses_beta", None).expect("不拒算");
    assert_eq!(
        lines,
        ["beta-tools\t1.0.0\tregistry+https://example.invalid/index\tb1"],
        "路径 beta_tools::value 点名 beta-tools（代码里 - 换成 _）"
    );
}

#[test]
fn a_named_external_crate_missing_from_the_lock_file_is_refused() {
    let reason = narrowed_lines_and_key("fconsumer::uses_delta", None)
        .expect_err("delta 在清单里、不在锁文件里，落不到包");
    assert!(
        reason.contains(
            "refused\tlock-dependency-unresolved\tcrates/fconsumer/src/lib.rs:12\tfconsumer → delta"
        ),
        "拒算行落在点名 delta 的那一行：{reason}"
    );
}

/// 改名依赖：清单里 `foo = { package = "bar", … }`，代码里 `use foo::…`；`Cargo.lock` 里这个包叫 `bar`。
const RENAMED_DEPENDENCY_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/frenamed/Cargo.toml",
        "[package]\nname = \"frenamed\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
         [dependencies]\nfoo = { package = \"bar\", version = \"1\" }\n",
    ),
    (
        "crates/frenamed/src/lib.rs",
        "use foo::Thing;\n\
         pub fn run() -> u32 {\n\
         \x20   Thing::make()\n\
         }\n",
    ),
    (
        "Cargo.lock",
        "version = 4\n\n\
         [[package]]\nname = \"bar\"\nversion = \"1.0.0\"\nsource = \"registry+https://example.invalid/index\"\n\
         checksum = \"b2\"\n\n\
         [[package]]\nname = \"frenamed\"\nversion = \"0.1.0\"\ndependencies = [\n \"bar\",\n]\n",
    ),
];

#[test]
fn a_renamed_dependency_is_found_in_the_lock_file_under_its_package_name() {
    let tree = FixtureTree::materialize("renamed-dependency", &RENAMED_DEPENDENCY_FIXTURE);
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_with_triple(
        &workspace,
        &entry("frenamed::run"),
        &switches,
        HOST_TARGET_TRIPLE,
    )
    .expect("入口找得到");
    let lock_text =
        std::fs::read_to_string(tree.root().join("Cargo.lock")).expect("夹具锁文件读得了");
    let packages = parse_lock_file(&lock_text).expect("夹具锁文件解析得了");
    let lines = narrowed_lock_lines(&workspace, &outcome, &packages).unwrap_or_else(|refusals| {
        panic!(
            "改名依赖应当按 package 写的包名找到：{}",
            refusal_lines(&workspace, &refusals).join(" | ")
        )
    });
    assert_eq!(
        lines,
        ["bar\t1.0.0\tregistry+https://example.invalid/index\tb2"],
        "代码里 use foo::…，清单把 foo 改名自包 bar，依赖行收 bar"
    );
}
