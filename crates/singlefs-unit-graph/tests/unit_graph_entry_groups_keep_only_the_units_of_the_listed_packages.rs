//! 一组入口只键几个包的单元（`singlefs-verification` 的构建脚本算判器与观察者引到的被检测产品代码用的那一层）：闭包从入口照收全，
//! 键与组成清单只取点名的包里的单元，入口单元与入口目标的文件都不进；判器与别的包改一处键不变，判器调到的产品改一处键变；
//! 只说别的包里那一处自己读的输入定不下来的拒算不挡键，说闭包收没收全的照挡；依赖图上够不到点名的包的 crate 不沿它收，它的文件不看。

mod common_fixture_workspace;

use std::collections::BTreeSet;

use common_fixture_workspace::{workspace_of, FixtureTree, HOST_TARGET_TRIPLE};
use singlefs_unit_graph::closure::PackagesKeptInTheKey;
use singlefs_unit_graph::entry_groups::{
    entry_group_key_keeping_only_the_listed_packages,
    entry_group_key_not_following_the_listed_units,
};
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::{
    EntryGroupKey, EntryGroupRequest, EntryGroupVerdict, FeaturesEnabledByTheBuild, TargetSelector,
};

/// 两个 crate：产品 `fproduct`（写一块、再挂一次）与判器那一侧的 `fpipeline`（录、判、读配置文件、拿自己的可执行文件起子进程），
/// `fpipeline` 的库另有一个判器 `judge_the_mount`（调产品的 `mount_again`）；`fpipeline` 的测试目标 `case` 里四条测试：`scenario` 录三步、读一份配置、判；
/// `another_scenario` 录五步、判；`scenario_restarting_the_program` 录两步、判、起自己；`scenario_with_an_observer` 录七步、经测试目标里的观察者
/// `observe_the_mount`（它调产品的 `mount_again`）再判。
const PACKAGE_FILTER_FIXTURE: [(&str, &str); 6] = [
    (
        "crates/fproduct/Cargo.toml",
        "[package]\nname = \"fproduct\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        "crates/fproduct/src/lib.rs",
        "pub fn write_block(steps: u32) -> u32 {\n\
         \x20   steps + 7\n\
         }\n\
         pub fn mount_again() -> u32 {\n\
         \x20   11\n\
         }\n",
    ),
    (
        "crates/fpipeline/Cargo.toml",
        "[package]\nname = \"fpipeline\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nfproduct = { path = \"../fproduct\" }\n",
    ),
    (
        "crates/fpipeline/src/lib.rs",
        "pub fn record(steps: u32) -> u32 {\n\
         \x20   fproduct::write_block(steps)\n\
         }\n\
         pub fn judge(written: u32) -> bool {\n\
         \x20   written > 1\n\
         }\n\
         pub fn read_the_configuration(file_name: &str) -> String {\n\
         \x20   std::fs::read_to_string(std::path::Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(file_name)).unwrap_or_default()\n\
         }\n\
         pub fn restart_the_program() -> bool {\n\
         \x20   std::process::Command::new(std::env::current_exe().expect(\"路径\")).status().is_ok()\n\
         }\n\
         pub fn judge_the_mount(written: u32) -> bool {\n\
         \x20   fproduct::mount_again() > written\n\
         }\n",
    ),
    (
        "crates/fpipeline/tests/case.rs",
        "fn steps_of_the_scenario() -> u32 {\n\
         \x20   3\n\
         }\n#[test]\n\
         fn scenario() {\n\
         \x20   let written = fpipeline::record(steps_of_the_scenario());\n\
         \x20   let _configuration = fpipeline::read_the_configuration(\"settings.txt\");\n\
         \x20   assert!(fpipeline::judge(written));\n\
         }\n#[test]\n\
         fn another_scenario() {\n\
         \x20   assert!(fpipeline::judge(fpipeline::record(5)));\n\
         }\n#[test]\n\
         fn scenario_restarting_the_program() {\n\
         \x20   assert!(fpipeline::judge(fpipeline::record(2)) && fpipeline::restart_the_program());\n\
         }\n\
         fn observe_the_mount(written: u32) -> u32 {\n\
         \x20   fproduct::mount_again() + written\n\
         }\n#[test]\n\
         fn scenario_with_an_observer() {\n\
         \x20   let written = fpipeline::record(7);\n\
         \x20   assert!(fpipeline::judge(observe_the_mount(written)));\n\
         }\n",
    ),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

/// `fpipeline` 的库里判的那一处改一个数。
const PIPELINE_EDIT: (&str, &str, &str) = (
    "crates/fpipeline/src/lib.rs",
    "    written > 1\n",
    "    written > 2\n",
);

/// 产品 `fproduct` 写一块的那一处改一个数。
const PRODUCT_EDIT: (&str, &str, &str) = (
    "crates/fproduct/src/lib.rs",
    "    steps + 7\n",
    "    steps + 8\n",
);

/// 判器 `judge_the_mount` 的函数体改一处写法。
const JUDGE_EDIT: (&str, &str, &str) = (
    "crates/fpipeline/src/lib.rs",
    "    fproduct::mount_again() > written\n",
    "    fproduct::mount_again() > (written)\n",
);

/// 产品 `fproduct` 再挂一次的那一处改一个数（只经判器 `judge_the_mount` 够到）。
const PRODUCT_MOUNT_EDIT: (&str, &str, &str) =
    ("crates/fproduct/src/lib.rs", "    11\n", "    12\n");

fn test_group(test_function: &str) -> EntryGroupRequest {
    EntryGroupRequest {
        group_name: format!("fixture_test:{test_function}"),
        target: TargetSelector::parse("test:case").expect("测试里写的目标认得"),
        entry_paths: vec![format!("fpipeline::{test_function}")],
        features: Vec::new(),
    }
}

fn judge_group() -> EntryGroupRequest {
    EntryGroupRequest {
        group_name: "judged_product_code".to_string(),
        target: TargetSelector::parse("lib").expect("测试里写的目标认得"),
        entry_paths: vec!["fpipeline::judge_the_mount".to_string()],
        features: Vec::new(),
    }
}

/// 照 `edit` 改夹具一处（`None` 不改），交 `request` 那一组只键产品 `fproduct` 的那一把与收全闭包的那一把。
fn kept_and_whole_keys_with(
    label: &str,
    request: &EntryGroupRequest,
    edit: Option<(&str, &str, &str)>,
) -> (EntryGroupKey, EntryGroupKey) {
    let tree = FixtureTree::materialize(label, &PACKAGE_FILTER_FIXTURE);
    if let Some((relative, original, replacement)) = edit {
        tree.replace_exactly_once(relative, original, replacement);
    }
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let kept = PackagesKeptInTheKey {
        package_names: BTreeSet::from(["fproduct".to_string()]),
    };
    let kept_key = entry_group_key_keeping_only_the_listed_packages(
        &workspace,
        request,
        &kept,
        &FeaturesEnabledByTheBuild::default(),
        HOST_TARGET_TRIPLE,
        &switches,
    )
    .expect("入口找得到");
    let whole_key = entry_group_key_not_following_the_listed_units(
        &workspace,
        request,
        &BTreeSet::new(),
        &FeaturesEnabledByTheBuild::default(),
        HOST_TARGET_TRIPLE,
        &switches,
    )
    .expect("入口找得到")
    .key;
    (kept_key, whole_key)
}

fn keyed(group_key: &EntryGroupKey) -> String {
    match &group_key.verdict {
        EntryGroupVerdict::Keyed { key } => key.hexadecimal(),
        EntryGroupVerdict::Refused { refusal_lines } => panic!("不该拒算：{refusal_lines:?}"),
        EntryGroupVerdict::NotCompiled { reason } => panic!("不该不编进来：{reason}"),
    }
}

/// 组成清单里每个单元落在哪份文件（`<种类>\t<限定名>\t<文件>:<起>-<止>\t<指纹>` 的第三段去掉行号）。
fn files_of_the_composition(group_key: &EntryGroupKey) -> BTreeSet<String> {
    group_key
        .composition_lines
        .iter()
        .filter_map(|line| line.split('\t').nth(2))
        .map(|location| {
            location
                .rsplit_once(':')
                .map_or(location, |(file, _lines)| file)
                .to_string()
        })
        .collect()
}

/// 组成清单里有没有一个单元的限定名以 `::<name>` 收尾。
fn composition_holds_a_unit_named(group_key: &EntryGroupKey, name: &str) -> bool {
    let suffix = format!("::{name}");
    group_key.composition_lines.iter().any(|line| {
        line.split('\t')
            .nth(1)
            .is_some_and(|qualified_name| qualified_name.ends_with(&suffix))
    })
}

/// 场景：从测试目标里的 `another_scenario` 收闭包，只键产品 `fproduct`。
/// 预期：组成清单里只有产品的文件，`fpipeline` 的库与测试目标一个单元都没有；收全闭包的那一把里 `fpipeline` 的库在（经它才够到产品）。
/// 判别力（`crates/mutations.tsv`）：只留的那一步把闭包里的单元全留下，这条红。
#[test]
fn the_kept_composition_holds_only_the_units_of_the_listed_package() {
    let (kept, whole) =
        kept_and_whole_keys_with("package-filter", &test_group("another_scenario"), None);
    assert_eq!(
        files_of_the_composition(&kept),
        BTreeSet::from(["crates/fproduct/src/lib.rs".to_string()]),
        "只键产品"
    );
    assert!(
        files_of_the_composition(&whole).contains("crates/fpipeline/src/lib.rs"),
        "收全的闭包经 fpipeline 的库够到产品：{:?}",
        whole.composition_lines
    );
}

/// 场景：`fpipeline` 的库里判的那一处改一个数，`another_scenario` 前后各算一次。
/// 预期：只键产品的那一把不变；收全闭包的那一把变（改动真在闭包里）。
/// 判别力（`crates/mutations.tsv`）：只留的那一步把闭包里的单元全留下，这条红在「不变」那一句。
#[test]
fn an_edit_of_a_unit_outside_the_listed_packages_keeps_the_kept_key() {
    let (kept_before, whole_before) =
        kept_and_whole_keys_with("package-filter", &test_group("another_scenario"), None);
    let (kept_after, whole_after) = kept_and_whole_keys_with(
        "package-filter",
        &test_group("another_scenario"),
        Some(PIPELINE_EDIT),
    );
    assert_ne!(
        keyed(&whole_before),
        keyed(&whole_after),
        "收全闭包的那一把变：改的那一处在闭包里"
    );
    assert_eq!(
        keyed(&kept_before),
        keyed(&kept_after),
        "fpipeline 不在点名的包里，只键产品的那一把不变"
    );
}

/// 场景：产品里写一块的那一处改一个数（产品只经 `fpipeline` 的库才够到），`another_scenario` 前后各算一次。
/// 预期：只键产品的那一把变。
/// 判别力（`crates/mutations.tsv`）：点名的包一个单元都不留，这条红。
#[test]
fn an_edit_of_a_unit_of_a_listed_package_reached_only_through_another_package_changes_the_kept_key()
{
    let (kept_before, _whole_before) =
        kept_and_whole_keys_with("package-filter", &test_group("another_scenario"), None);
    let (kept_after, _whole_after) = kept_and_whole_keys_with(
        "package-filter",
        &test_group("another_scenario"),
        Some(PRODUCT_EDIT),
    );
    assert_ne!(
        keyed(&kept_before),
        keyed(&kept_after),
        "产品经 fpipeline 才够到，照收、照键"
    );
}

/// 场景：`scenario` 经 `fpipeline` 读一份路径拼出来的配置文件（`fpipeline` 那一处拒算 `runtime-read-path-not-literal`）。
/// 预期：收全闭包的那一把拒算；只键产品的那一把算得出（那一处只说它自己读的输入定不下来，它不进键）。
/// 判别力（`crates/mutations.tsv`）：只留的那一步把拒算一律当闭包收没收全的那一种留下，这条红。
#[test]
fn a_refusal_of_an_input_read_outside_the_listed_packages_does_not_refuse_the_kept_key() {
    let (kept, whole) = kept_and_whole_keys_with("package-filter", &test_group("scenario"), None);
    match &whole.verdict {
        EntryGroupVerdict::Refused { refusal_lines } => assert!(
            refusal_lines.iter().any(|line| line.starts_with(
                "refused\truntime-read-path-not-literal\tcrates/fpipeline/src/lib.rs:"
            )),
            "{refusal_lines:?}"
        ),
        EntryGroupVerdict::Keyed { .. } | EntryGroupVerdict::NotCompiled { .. } => {
            panic!(
                "收全的闭包读了路径拼出来的文件，应当拒算：{:?}",
                whole.verdict
            )
        }
    }
    keyed(&kept);
}

/// 场景：`scenario_restarting_the_program` 经 `fpipeline` 拿自己的可执行文件起子进程（那一处拒算 `current-exe`：子进程跑的代码收不进来）。
/// 预期：只键产品的那一把照样拒算，拒算行点名 `fpipeline` 那一处。
/// 判别力（`crates/mutations.tsv`）：只留的那一步把拒算一律当只说那一处输入的那一种（落在别的包里就丢），这条红。
#[test]
fn a_refusal_that_leaves_the_closure_uncertain_outside_the_listed_packages_refuses_the_kept_key() {
    let (kept, _whole) = kept_and_whole_keys_with(
        "package-filter",
        &test_group("scenario_restarting_the_program"),
        None,
    );
    match &kept.verdict {
        EntryGroupVerdict::Refused { refusal_lines } => assert!(
            refusal_lines
                .iter()
                .any(|line| line.starts_with("refused\tcurrent-exe\tcrates/fpipeline/src/lib.rs:")),
            "{refusal_lines:?}"
        ),
        EntryGroupVerdict::Keyed { .. } | EntryGroupVerdict::NotCompiled { .. } => panic!(
            "起自己的子进程收不全闭包，只键几个包也应当拒算：{:?}",
            kept.verdict
        ),
    }
}

/// 场景：从库里的判器 `judge_the_mount`（它调产品的 `mount_again`）收闭包，只键产品 `fproduct`。
/// 预期：组成清单里只有产品的文件，判器自己（入口单元）不在；对照：收全闭包的那一把里判器在。
/// 判别力（`crates/mutations.tsv`）：只留的那一步把闭包里的单元全留下，这条红。
#[test]
fn the_composition_keeping_only_the_listed_packages_holds_neither_the_entry_units_nor_the_entry_target(
) {
    let (only_the_listed_packages, whole) =
        kept_and_whole_keys_with("package-filter-judge", &judge_group(), None);
    assert_eq!(
        files_of_the_composition(&only_the_listed_packages),
        BTreeSet::from(["crates/fproduct/src/lib.rs".to_string()]),
        "只键产品：{:?}",
        only_the_listed_packages.composition_lines
    );
    assert!(
        composition_holds_a_unit_named(&whole, "judge_the_mount"),
        "对照：收全闭包的那一把里入口单元在：{:?}",
        whole.composition_lines
    );
}

/// 场景：判器 `judge_the_mount` 的函数体改一处写法，前后各算一次。
/// 预期：只键产品的那一把不变；收全闭包的那一把变（改动真在闭包里）。
/// 判别力（`crates/mutations.tsv`）：只留的那一步把闭包里的单元全留下，这条红在「不变」那一句。
#[test]
fn an_edit_of_the_judge_at_the_entry_keeps_the_key_of_only_the_listed_packages() {
    let (only_before, whole_before) =
        kept_and_whole_keys_with("package-filter-judge", &judge_group(), None);
    let (only_after, whole_after) =
        kept_and_whole_keys_with("package-filter-judge", &judge_group(), Some(JUDGE_EDIT));
    assert_ne!(
        keyed(&whole_before),
        keyed(&whole_after),
        "收全闭包的那一把变：改的那一处在闭包里"
    );
    assert_eq!(
        keyed(&only_before),
        keyed(&only_after),
        "判器不在点名的包里，只键产品的那一把不变"
    );
}

/// 场景：判器调到的产品 `mount_again` 改一个数，前后各算一次。
/// 预期：只键产品的那一把变。
/// 判别力（`crates/mutations.tsv`）：点名的包一个单元也不留，这条红。
#[test]
fn an_edit_of_the_product_the_judge_reaches_changes_the_key_of_only_the_listed_packages() {
    let (only_before, _whole_before) =
        kept_and_whole_keys_with("package-filter-judge", &judge_group(), None);
    let (only_after, _whole_after) = kept_and_whole_keys_with(
        "package-filter-judge",
        &judge_group(),
        Some(PRODUCT_MOUNT_EDIT),
    );
    assert_ne!(
        keyed(&only_before),
        keyed(&only_after),
        "判器调到的产品改了，只键产品的那一把变"
    );
}

/// 三个 crate：产品 `fproduct`（再挂一次）、判器那一侧的 `fpipeline`（判器 `judge_the_mount_and_store_the_verdict` 调产品的 `mount_again`，
/// 再把判出的结论交给存结论的服务），与经 path 依赖收进来的工作区外的服务 `store-service`（存结论：不依赖任何 crate，判定库服务 prune-server 那一类）。
const FIXTURE_WITH_A_SERVICE_REACHING_NO_LISTED_PACKAGE: [(&str, &str); 7] = [
    (
        "crates/fproduct/Cargo.toml",
        "[package]\nname = \"fproduct\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        "crates/fproduct/src/lib.rs",
        "pub fn mount_again() -> u32 {\n\
         \x20   11\n\
         }\n",
    ),
    (
        "crates/fpipeline/Cargo.toml",
        "[package]\nname = \"fpipeline\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nfproduct = { path = \"../fproduct\" }\nstore-service = { path = \"../../store-service\" }\n",
    ),
    (
        "crates/fpipeline/src/lib.rs",
        "pub fn judge_the_mount_and_store_the_verdict(written: u32) -> bool {\n\
         \x20   let is_judged_red = fproduct::mount_again() > written;\n\
         \x20   store_service::store_the_verdict(is_judged_red)\n\
         }\n",
    ),
    (
        "store-service/Cargo.toml",
        "[package]\nname = \"store-service\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    ),
    (
        "store-service/src/lib.rs",
        "pub fn store_the_verdict(is_judged_red: bool) -> bool {\n\
         \x20   !is_judged_red\n\
         }\n",
    ),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

/// 场景：从判器 `judge_the_mount_and_store_the_verdict` 收闭包，只键产品 `fproduct`；存结论的服务 `store-service` 依赖图上够不到产品。
/// 预期：只键产品的那一把要看的文件里有产品与判器所在的 `fpipeline`（判器改一处会改够到产品的哪些单元），没有 `store-service` 的文件
/// （它不沿它收：那里的代码调不到产品）；对照：收全闭包的那一把要看的文件里有 `store-service` 的库根（判器真调到它）。
/// 判别力（`crates/mutations.tsv`）：只键点名的包的那一把交空的「不沿它收」登记（照旧沿够不到产品的 crate 收），这条红在「没有 store-service」那一句。
#[test]
fn the_kept_key_watches_no_file_of_a_crate_reaching_none_of_the_listed_packages() {
    let tree = FixtureTree::materialize(
        "package-filter-service",
        &FIXTURE_WITH_A_SERVICE_REACHING_NO_LISTED_PACKAGE,
    );
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let request = EntryGroupRequest {
        group_name: "judged_product_code".to_string(),
        target: TargetSelector::parse("lib").expect("测试里写的目标认得"),
        entry_paths: vec!["fpipeline::judge_the_mount_and_store_the_verdict".to_string()],
        features: Vec::new(),
    };
    let kept = entry_group_key_keeping_only_the_listed_packages(
        &workspace,
        &request,
        &PackagesKeptInTheKey {
            package_names: BTreeSet::from(["fproduct".to_string()]),
        },
        &FeaturesEnabledByTheBuild::default(),
        HOST_TARGET_TRIPLE,
        &switches,
    )
    .expect("入口找得到");
    let whole = entry_group_key_not_following_the_listed_units(
        &workspace,
        &request,
        &BTreeSet::new(),
        &FeaturesEnabledByTheBuild::default(),
        HOST_TARGET_TRIPLE,
        &switches,
    )
    .expect("入口找得到")
    .key;
    assert!(
        whole.watched_files.contains("store-service/src/lib.rs"),
        "对照：收全的闭包够到存结论的服务：{:?}",
        whole.watched_files
    );
    assert!(
        kept.watched_files.contains("crates/fproduct/src/lib.rs")
            && kept.watched_files.contains("crates/fpipeline/src/lib.rs"),
        "只键产品的那一把看着产品与判器所在的 crate：{:?}",
        kept.watched_files
    );
    assert!(
        !kept
            .watched_files
            .iter()
            .any(|path| path.starts_with("store-service/")),
        "只键产品的那一把不看够不到产品的服务的文件：{:?}",
        kept.watched_files
    );
    assert_eq!(
        files_of_the_composition(&kept),
        BTreeSet::from(["crates/fproduct/src/lib.rs".to_string()]),
        "只键产品：{:?}",
        kept.composition_lines
    );
}
