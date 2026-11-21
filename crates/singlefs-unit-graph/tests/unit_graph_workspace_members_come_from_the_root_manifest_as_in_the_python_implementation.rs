//! 工作区成员照根清单收（`src/workspace.rs` 的 `directories_of_the_workspace_members`），与 `scripts/gate/lib_function_units.py` 的
//! `discover_rust_crates` 同一条规则：`[workspace] members` 逐条照通配展开（`*`、`?`、`[…]`，`*` 不配 `.` 起头的名字），去掉
//! `exclude` 点名的与底下没有 `Cargo.toml` 的；只写子表头的 `[workspace.metadata]` 也让 `workspace` 非空；根清单带 `[package]`（只写
//! 表头也算）时仓根自己也是一个成员，它底下别的 crate 目录里的文件归那个 crate；没写 `[package] name` 的包取目录名、仓根取 `root`；
//! 一个成员都收不到就是空的工作区、不报错；传进来的目录有根清单而没有 `crates/` 的照样当仓根。
//! 两份读法对不上的几形照多收的那一边：内联空表 `workspace = {}` 与没有 `workspace` 同样算；多行串开头三引号后面那个换行不算内容；
//! 写成绝对路径的成员从文件系统的根展开、落在仓根外照收（文件路径照 `os.path.relpath` 写成 `..` 起头）；成员目录底下套着别的成员时，
//! 套在里面的那个目录底下的文件只归里面那个 crate。
//! 每条用例里写的期望集合是照那段 python 规则在同一棵夹具树上逐条推出来的。

mod common_fixture_workspace;

use common_fixture_workspace::{test_threads, workspace_of, FixtureTree};
use singlefs_unit_graph::rule_switches::RuleSwitches;
use singlefs_unit_graph::workspace::CrateOrigin;
use singlefs_unit_graph::{load_workspace, Workspace};

/// 根清单的 `members` 里有 `crates/` 外的 `service`、一条 `crates/*`、一条 `?`、一条 `[…]` 与一条不存在的 `missing`；`exclude` 点名
/// `crates/excluded`。`crates/.hidden` 有清单而 `*` 不配它，`crates/notes` 没有清单，`tools/guild` 不在 `[bc]` 里；`crates/beta` 经
/// path 依赖接到工作区外的 `vendored`。
const MEMBER_PATTERN_FIXTURE: [(&str, &str); 20] = [
    (
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/*\", \"service\", \"tools/rend?r\", \"tools/[bc]uild\", \"missing\"]\nexclude = [\"crates/excluded\"]\n",
    ),
    ("crates/alpha/Cargo.toml", "[package]\nname = \"alpha\"\n"),
    ("crates/alpha/src/lib.rs", "pub fn alpha_value() -> u32 {\n    1\n}\n"),
    (
        "crates/beta/Cargo.toml",
        "[package]\nname = \"beta\"\n\n[dependencies]\nvendored = { path = \"../../vendored\" }\n",
    ),
    ("crates/beta/src/lib.rs", "pub fn beta_value() -> u32 {\n    vendored::vendored_value()\n}\n"),
    ("crates/excluded/Cargo.toml", "[package]\nname = \"excluded\"\n"),
    ("crates/excluded/src/lib.rs", "pub fn excluded_value() -> u32 {\n    3\n}\n"),
    ("crates/.hidden/Cargo.toml", "[package]\nname = \"hidden\"\n"),
    ("crates/.hidden/src/lib.rs", "pub fn hidden_value() -> u32 {\n    4\n}\n"),
    ("crates/notes/readme.txt", "这个目录没有清单\n"),
    ("service/Cargo.toml", "[package]\nname = \"service\"\n"),
    ("service/src/lib.rs", "pub fn service_value() -> u32 {\n    5\n}\n"),
    ("tools/render/Cargo.toml", "[package]\nname = \"render\"\n"),
    ("tools/render/src/lib.rs", "pub fn render_value() -> u32 {\n    6\n}\n"),
    ("tools/build/Cargo.toml", "[package]\nname = \"build\"\n"),
    ("tools/build/src/lib.rs", "pub fn build_value() -> u32 {\n    7\n}\n"),
    ("tools/guild/Cargo.toml", "[package]\nname = \"guild\"\n"),
    ("tools/guild/src/lib.rs", "pub fn guild_value() -> u32 {\n    8\n}\n"),
    ("vendored/Cargo.toml", "[package]\nname = \"vendored\"\n"),
    ("vendored/src/lib.rs", "pub fn vendored_value() -> u32 {\n    9\n}\n"),
];

/// 照 python 规则推出的 crate 集合（包名，怎么收进来的），按包名排好。
fn expected_crates_of_the_member_pattern_fixture() -> Vec<(String, CrateOrigin)> {
    vec![
        ("alpha".to_string(), CrateOrigin::WorkspaceMember),
        ("beta".to_string(), CrateOrigin::WorkspaceMember),
        ("build".to_string(), CrateOrigin::WorkspaceMember),
        ("render".to_string(), CrateOrigin::WorkspaceMember),
        ("service".to_string(), CrateOrigin::WorkspaceMember),
        (
            "vendored".to_string(),
            CrateOrigin::PathDependencyOutsideTheWorkspace,
        ),
    ]
}

/// 装出来的工作区里每个 crate 的（包名，怎么收进来的），按包名排好。
fn crates_by_package_name(workspace: &Workspace) -> Vec<(String, CrateOrigin)> {
    let mut crates: Vec<(String, CrateOrigin)> = workspace
        .crates
        .iter()
        .map(|workspace_crate| (workspace_crate.package_name.clone(), workspace_crate.origin))
        .collect();
    crates.sort_by_key(|(package_name, _)| package_name.clone());
    crates
}

#[test]
fn members_outside_the_crates_directory_globs_and_exclusions_follow_the_root_manifest() {
    let tree = FixtureTree::materialize("workspace-members-from-the-root", &MEMBER_PATTERN_FIXTURE);
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    assert_eq!(
        crates_by_package_name(&workspace),
        expected_crates_of_the_member_pattern_fixture(),
        "service 在 crates/ 外照样是成员；crates/* 收 alpha 与 beta、不收 exclude 点名的 excluded、不收 . 起头的 .hidden；\
         rend?r 收 render，[bc]uild 收 build、不收 guild；missing 不存在；vendored 是 beta 的 path 依赖、在工作区外"
    );
    assert!(
        workspace
            .files
            .iter()
            .any(|file| file.relative_path == "service/src/lib.rs"),
        "crates/ 外的成员 service 的源码要切进工作区"
    );
}

/// 根传的是 `crates/` 目录本身：仓根取它的上一层，成员照上一层的根清单收，与传仓根收出来的相同。
#[test]
fn passing_the_crates_directory_collects_the_same_members_as_passing_the_repository_root() {
    let tree = FixtureTree::materialize("workspace-members-from-crates", &MEMBER_PATTERN_FIXTURE);
    let workspace = workspace_of(
        &tree.root().join("crates"),
        &RuleSwitches::every_rule_intact(),
    );
    assert_eq!(
        crates_by_package_name(&workspace),
        expected_crates_of_the_member_pattern_fixture(),
        "传 crates/ 时同样照仓根的根清单收成员"
    );
}

/// 根清单带 `[package]`、`workspace` 写成内联表：仓根自己是成员 `rooted`，`crates/alpha` 照 `members` 收；仓根那个 crate 不把
/// `crates/alpha/` 底下的文件再收一遍。
const ROOT_PACKAGE_FIXTURE: [(&str, &str); 4] = [
    (
        "Cargo.toml",
        "workspace = { members = [\"crates/*\"] }\n\n[package]\nname = \"rooted\"\n",
    ),
    ("src/lib.rs", "pub fn rooted_value() -> u32 {\n    10\n}\n"),
    ("crates/alpha/Cargo.toml", "[package]\nname = \"alpha\"\n"),
    (
        "crates/alpha/src/lib.rs",
        "pub fn alpha_value() -> u32 {\n    1\n}\n",
    ),
];

#[test]
fn root_package_is_a_member_and_leaves_the_files_of_the_crates_below_it_to_them() {
    let tree = FixtureTree::materialize("workspace-members-root-package", &ROOT_PACKAGE_FIXTURE);
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    assert_eq!(
        crates_by_package_name(&workspace),
        vec![
            ("alpha".to_string(), CrateOrigin::WorkspaceMember),
            ("rooted".to_string(), CrateOrigin::WorkspaceMember),
        ],
        "根清单带 [package] 时仓根是成员；内联表写的 workspace.members 照样收 crates/alpha"
    );
    let mut files_with_their_package: Vec<(String, String)> = workspace
        .files
        .iter()
        .map(|file| {
            (
                file.relative_path.clone(),
                workspace.crates[file.crate_index.0].package_name.clone(),
            )
        })
        .collect();
    files_with_their_package.sort();
    assert_eq!(
        files_with_their_package,
        vec![
            ("crates/alpha/src/lib.rs".to_string(), "alpha".to_string()),
            ("src/lib.rs".to_string(), "rooted".to_string()),
        ],
        "每份文件只归一个 crate：crates/alpha/ 底下的归 alpha，不再算进仓根那个 crate"
    );
}

/// 装出来的工作区里每份文件（相对仓根的路径），排好序。
fn relative_paths_of_the_files(workspace: &Workspace) -> Vec<String> {
    let mut relative_paths: Vec<String> = workspace
        .files
        .iter()
        .map(|file| file.relative_path.clone())
        .collect();
    relative_paths.sort();
    relative_paths
}

/// 根上有带 `[workspace]` 的根清单、没有 `crates/` 目录。python 那一份把传进来的目录当仓根：`workspace` 非空，`exclude` 是
/// `{"tools/retired"}`；`service` 展开成 `service`（有清单，收）；`tools/*` 排好序展开成 `tools/render`（收）与 `tools/retired`（exclude
/// 点名，不收）；`loose` 不在 `members` 里；没有 `[package]`。推出的成员是 render 与 service，文件路径从这棵树的根算。
const ROOT_MANIFEST_WITHOUT_A_CRATES_DIRECTORY_FIXTURE: [(&str, &str); 9] = [
    (
        "Cargo.toml",
        "[workspace]\nmembers = [\"service\", \"tools/*\"]\nexclude = [\"tools/retired\"]\n",
    ),
    ("service/Cargo.toml", "[package]\nname = \"service\"\n"),
    (
        "service/src/lib.rs",
        "pub fn service_value() -> u32 {\n    5\n}\n",
    ),
    ("tools/render/Cargo.toml", "[package]\nname = \"render\"\n"),
    (
        "tools/render/src/lib.rs",
        "pub fn render_value() -> u32 {\n    6\n}\n",
    ),
    (
        "tools/retired/Cargo.toml",
        "[package]\nname = \"retired\"\n",
    ),
    (
        "tools/retired/src/lib.rs",
        "pub fn retired_value() -> u32 {\n    11\n}\n",
    ),
    ("loose/Cargo.toml", "[package]\nname = \"loose\"\n"),
    (
        "loose/src/lib.rs",
        "pub fn loose_value() -> u32 {\n    12\n}\n",
    ),
];

#[test]
fn root_manifest_without_a_crates_directory_is_read_as_the_repository_root() {
    let tree = FixtureTree::materialize(
        "workspace-members-without-crates",
        &ROOT_MANIFEST_WITHOUT_A_CRATES_DIRECTORY_FIXTURE,
    );
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    assert_eq!(
        crates_by_package_name(&workspace),
        vec![
            ("render".to_string(), CrateOrigin::WorkspaceMember),
            ("service".to_string(), CrateOrigin::WorkspaceMember),
        ],
        "没有 crates/ 而有根清单的目录是仓根：成员照它的 members 收 service 与 render，不收 exclude 点名的 retired、不在 members 里的 loose"
    );
    assert_eq!(
        relative_paths_of_the_files(&workspace),
        vec![
            "service/src/lib.rs".to_string(),
            "tools/render/src/lib.rs".to_string(),
        ],
        "文件路径从传进来的这一层算起"
    );
}

/// 根清单 `[workspace]` 有键，`members` 只有 `plugins/*`，而 `plugins/` 底下唯一的目录没有清单；`crates/alpha` 有清单。python 那一份：
/// `workspace` 非空，`plugins/*` 展开成 `plugins/notes`、没有 `Cargo.toml`，不收；有 `workspace` 就不走读 `crates/` 那一支，alpha 不收；
/// 没有 `[package]`。推出的成员是空列表，顺 path 依赖也收不到别的：一个 crate、一份文件都没有，不报错。
const WORKSPACE_WITHOUT_ANY_MEMBER_FIXTURE: [(&str, &str); 4] = [
    (
        "Cargo.toml",
        "[workspace]\nresolver = \"2\"\nmembers = [\"plugins/*\"]\n",
    ),
    ("plugins/notes/readme.txt", "这个目录没有清单\n"),
    ("crates/alpha/Cargo.toml", "[package]\nname = \"alpha\"\n"),
    (
        "crates/alpha/src/lib.rs",
        "pub fn alpha_value() -> u32 {\n    1\n}\n",
    ),
];

#[test]
fn workspace_whose_members_expand_to_nothing_is_an_empty_workspace() {
    let tree = FixtureTree::materialize(
        "workspace-members-expand-to-nothing",
        &WORKSPACE_WITHOUT_ANY_MEMBER_FIXTURE,
    );
    let loaded = load_workspace(
        tree.root(),
        test_threads(),
        &RuleSwitches::every_rule_intact(),
    );
    assert!(
        loaded.is_ok(),
        "members 一个都展开不出：空的工作区，不报读不了：{:?}",
        loaded.as_ref().err()
    );
    let workspace = loaded.expect("上一条断言核过装得起来");
    assert_eq!(
        crates_by_package_name(&workspace),
        Vec::<(String, CrateOrigin)>::new(),
        "members 一个都展开不出：空的工作区，不退回读 crates/"
    );
    assert_eq!(
        relative_paths_of_the_files(&workspace),
        Vec::<String>::new(),
        "空的工作区里没有文件"
    );
}

/// 根清单只有 `[package]`、没有 `crates/` 目录。python 那一份：没有 `workspace`，`crates/` 不在，读 `crates/` 那一支收不到；
/// `"package" in manifest` 成立，仓根是成员。推出的成员只有仓根这一个 standalone。
const ROOT_PACKAGE_WITHOUT_A_CRATES_DIRECTORY_FIXTURE: [(&str, &str); 2] = [
    ("Cargo.toml", "[package]\nname = \"standalone\"\n"),
    (
        "src/lib.rs",
        "pub fn standalone_value() -> u32 {\n    13\n}\n",
    ),
];

#[test]
fn root_package_without_a_crates_directory_is_the_only_member() {
    let tree = FixtureTree::materialize(
        "workspace-members-root-package-only",
        &ROOT_PACKAGE_WITHOUT_A_CRATES_DIRECTORY_FIXTURE,
    );
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    assert_eq!(
        crates_by_package_name(&workspace),
        vec![("standalone".to_string(), CrateOrigin::WorkspaceMember)],
        "没有 workspace、没有 crates/：仓根那个包是唯一的成员，crates/ 不在不算读不了"
    );
    assert_eq!(
        relative_paths_of_the_files(&workspace),
        vec!["src/lib.rs".to_string()],
        "仓根那个包的文件从仓根算起"
    );
}

/// 根清单只写一行表头 `[package]`、底下没有键；`crates/unnamed-member` 的清单有 `[package]` 而没写 `name`。python 那一份
/// （`discover_rust_crates` 与 `RustCrate.__init__`）：根清单读出 `{"package": {}}`，没有 `workspace`；`crates/` 在，`unnamed-member` 底下有
/// 清单，收进 `crates/unnamed-member`；`"package" in manifest` 成立，在最前插进仓根 `""`。包名取 `package.get("name",
/// posixpath.basename(directory) or "root")`：仓根的 `basename("")` 是空串，取 `root`；`crates/unnamed-member` 取 `unnamed-member`。
/// 库名取包名把 `-` 换成 `_`：`root` 与 `unnamed_member`。仓根那个 crate 不收 `crates/unnamed-member/` 底下的文件。
const BARE_PACKAGE_HEADER_AND_UNNAMED_PACKAGES_FIXTURE: [(&str, &str); 4] = [
    ("Cargo.toml", "[package]\n"),
    ("src/lib.rs", "pub fn root_value() -> u32 {\n    14\n}\n"),
    (
        "crates/unnamed-member/Cargo.toml",
        "[package]\nversion = \"0.1.0\"\n",
    ),
    (
        "crates/unnamed-member/src/lib.rs",
        "pub fn unnamed_member_value() -> u32 {\n    15\n}\n",
    ),
];

#[test]
fn bare_package_header_makes_the_root_a_member_and_unnamed_packages_take_the_directory_name() {
    let tree = FixtureTree::materialize(
        "workspace-members-bare-package-header",
        &BARE_PACKAGE_HEADER_AND_UNNAMED_PACKAGES_FIXTURE,
    );
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    assert_eq!(
        crates_by_package_name(&workspace),
        vec![
            ("root".to_string(), CrateOrigin::WorkspaceMember),
            ("unnamed-member".to_string(), CrateOrigin::WorkspaceMember),
        ],
        "只写表头的 [package] 也让仓根成为成员；没写 name 的包取目录名，仓根取 root"
    );
    let mut crate_identifiers: Vec<String> = workspace
        .crates
        .iter()
        .map(|workspace_crate| workspace_crate.crate_identifier.clone())
        .collect();
    crate_identifiers.sort();
    assert_eq!(
        crate_identifiers,
        vec!["root".to_string(), "unnamed_member".to_string()],
        "库名照取出来的包名把 - 换成 _"
    );
    let mut files_with_their_package: Vec<(String, String)> = workspace
        .files
        .iter()
        .map(|file| {
            (
                file.relative_path.clone(),
                workspace.crates[file.crate_index.0].package_name.clone(),
            )
        })
        .collect();
    files_with_their_package.sort();
    assert_eq!(
        files_with_their_package,
        vec![
            (
                "crates/unnamed-member/src/lib.rs".to_string(),
                "unnamed-member".to_string()
            ),
            ("src/lib.rs".to_string(), "root".to_string()),
        ],
        "仓根那个包只收自己的 src/lib.rs，crates/unnamed-member/ 底下的归 unnamed-member"
    );
}

/// 根清单只写一行子表头 `[workspace.metadata]`；`crates/alpha` 有清单。python 那一份：根清单读出 `{"workspace": {"metadata": {}}}`，
/// `workspace` 是一张带一张空子表的字典、为真；`members` 没写，展开不出成员；有 `workspace` 就不走读 `crates/` 那一支，alpha 不收；
/// 没有 `package`。推出的成员是空列表：一个 crate、一份文件都没有，不报错。
const WORKSPACE_SUB_TABLE_HEADER_ONLY_FIXTURE: [(&str, &str); 3] = [
    ("Cargo.toml", "[workspace.metadata]\n"),
    ("crates/alpha/Cargo.toml", "[package]\nname = \"alpha\"\n"),
    (
        "crates/alpha/src/lib.rs",
        "pub fn alpha_value() -> u32 {\n    1\n}\n",
    ),
];

#[test]
fn workspace_sub_table_header_alone_is_a_workspace_without_members() {
    let tree = FixtureTree::materialize(
        "workspace-members-sub-table-header-only",
        &WORKSPACE_SUB_TABLE_HEADER_ONLY_FIXTURE,
    );
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    assert_eq!(
        crates_by_package_name(&workspace),
        Vec::<(String, CrateOrigin)>::new(),
        "只写 [workspace.metadata] 的根清单有 workspace：不退回读 crates/，members 没写就一个成员都没有"
    );
    assert_eq!(
        relative_paths_of_the_files(&workspace),
        Vec::<String>::new(),
        "空的工作区里没有文件"
    );
}

/// 没有根清单；`crates/` 底下唯一的目录没有清单；`loose/` 有清单而不在 `crates/` 下。python 那一份：根清单不在，读成 `{}`，没有
/// `workspace`；`crates/` 在，`crates/notes` 底下没有 `Cargo.toml`，不收；没有 `package`；`loose` 不在任何一支里。推出的成员是空列表，
/// 顺 path 依赖也收不到别的：一个 crate、一份文件都没有，不报错。
const NO_WORKSPACE_AND_NO_MEMBER_FIXTURE: [(&str, &str); 3] = [
    ("crates/notes/readme.txt", "这个目录没有清单\n"),
    ("loose/Cargo.toml", "[package]\nname = \"loose\"\n"),
    (
        "loose/src/lib.rs",
        "pub fn loose_value() -> u32 {\n    12\n}\n",
    ),
];

#[test]
fn without_a_workspace_collecting_no_member_is_an_empty_workspace() {
    let tree = FixtureTree::materialize(
        "workspace-members-none-without-a-workspace",
        &NO_WORKSPACE_AND_NO_MEMBER_FIXTURE,
    );
    let loaded = load_workspace(
        tree.root(),
        test_threads(),
        &RuleSwitches::every_rule_intact(),
    );
    assert!(
        loaded.is_ok(),
        "没有 workspace、一个成员都收不到：空的工作区，不报读不了：{:?}",
        loaded.as_ref().err()
    );
    let workspace = loaded.expect("上一条断言核过装得起来");
    assert_eq!(
        crates_by_package_name(&workspace),
        Vec::<(String, CrateOrigin)>::new(),
        "没有 workspace、crates/ 底下没有带清单的目录、没有 package：空的工作区，不报错，也不收 crates/ 外的 loose"
    );
    assert_eq!(
        relative_paths_of_the_files(&workspace),
        Vec::<String>::new(),
        "空的工作区里没有文件"
    );
}

/// 根清单写成内联空表 `workspace = {}`；`crates/alpha` 有清单。python 那一份：`workspace` 是空字典、判假，与没有同样算，退回读 `crates/`，
/// 收 alpha（多收的那一边）。
const INLINE_EMPTY_WORKSPACE_TABLE_FIXTURE: [(&str, &str); 3] = [
    ("Cargo.toml", "workspace = {}\n"),
    ("crates/alpha/Cargo.toml", "[package]\nname = \"alpha\"\n"),
    (
        "crates/alpha/src/lib.rs",
        "pub fn alpha_value() -> u32 {\n    1\n}\n",
    ),
];

#[test]
fn inline_empty_workspace_table_reads_the_crates_directory_as_without_a_workspace() {
    let tree = FixtureTree::materialize(
        "workspace-members-inline-empty-table",
        &INLINE_EMPTY_WORKSPACE_TABLE_FIXTURE,
    );
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    assert_eq!(
        crates_by_package_name(&workspace),
        vec![("alpha".to_string(), CrateOrigin::WorkspaceMember)],
        "workspace = {{}} 与没有 workspace 同样算：退回读 crates/，收 alpha"
    );
}

/// 根清单的 `members` 写成多行基础串，开头三引号后面紧跟一个换行。python `tomllib` 去掉这个换行，成员是 `crates/alpha`。
const MULTI_LINE_STRING_MEMBER_FIXTURE: [(&str, &str); 3] = [
    (
        "Cargo.toml",
        "[workspace]\nmembers = [\"\"\"\ncrates/alpha\"\"\"]\n",
    ),
    ("crates/alpha/Cargo.toml", "[package]\nname = \"alpha\"\n"),
    (
        "crates/alpha/src/lib.rs",
        "pub fn alpha_value() -> u32 {\n    1\n}\n",
    ),
];

#[test]
fn member_written_in_a_multi_line_string_drops_the_newline_after_the_opening_quotes() {
    let tree = FixtureTree::materialize(
        "workspace-members-multi-line-string",
        &MULTI_LINE_STRING_MEMBER_FIXTURE,
    );
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    assert_eq!(
        crates_by_package_name(&workspace),
        vec![("alpha".to_string(), CrateOrigin::WorkspaceMember)],
        "多行串开头三引号后面那个换行不算内容，成员是 crates/alpha"
    );
}

/// 仓根是夹具树底下的 `workspace/`，`members` 写成夹具树底下 `outside/beta` 的绝对路径（落在仓根外）。python 那一份：`os.path.join`
/// 遇到绝对路径取它本身，收 beta，文件路径照 `os.path.relpath` 写成 `../outside/beta/src/lib.rs`。
#[test]
fn absolute_member_path_outside_the_repository_root_is_collected() {
    let tree = FixtureTree::materialize(
        "workspace-members-absolute-path",
        &[
            ("outside/beta/Cargo.toml", "[package]\nname = \"beta\"\n"),
            (
                "outside/beta/src/lib.rs",
                "pub fn beta_value() -> u32 {\n    1\n}\n",
            ),
        ],
    );
    let repository_root = tree.root().join("workspace");
    std::fs::create_dir_all(&repository_root).expect("临时目录建得起来");
    let canonical_tree_root = std::fs::canonicalize(tree.root()).expect("夹具树的根在");
    std::fs::write(
        repository_root.join("Cargo.toml"),
        format!(
            "[workspace]\nmembers = [\"{}\"]\n",
            canonical_tree_root.join("outside/beta").display()
        ),
    )
    .expect("根清单写得进临时目录");
    let workspace = workspace_of(&repository_root, &RuleSwitches::every_rule_intact());
    assert_eq!(
        crates_by_package_name(&workspace),
        vec![("beta".to_string(), CrateOrigin::WorkspaceMember)],
        "绝对路径的成员从文件系统的根展开，落在仓根外照收"
    );
    assert_eq!(
        relative_paths_of_the_files(&workspace),
        vec!["../outside/beta/src/lib.rs".to_string()],
        "仓根外的文件路径照 os.path.relpath 写成 .. 起头"
    );
}

/// `crates/outer` 与套在它底下的 `crates/outer/inner` 都是成员。python 那一份按最长前缀把 `crates/outer/inner/` 底下的文件归给 inner，
/// 每份文件只收一遍。
const NESTED_MEMBER_FIXTURE: [(&str, &str); 5] = [
    (
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/outer\", \"crates/outer/inner\"]\n",
    ),
    ("crates/outer/Cargo.toml", "[package]\nname = \"outer\"\n"),
    (
        "crates/outer/src/lib.rs",
        "pub fn outer_value() -> u32 {\n    1\n}\n",
    ),
    (
        "crates/outer/inner/Cargo.toml",
        "[package]\nname = \"inner\"\n",
    ),
    (
        "crates/outer/inner/src/lib.rs",
        "pub fn inner_value() -> u32 {\n    2\n}\n",
    ),
];

#[test]
fn files_of_a_member_nested_inside_another_member_belong_only_to_the_nested_one() {
    let tree = FixtureTree::materialize("workspace-members-nested", &NESTED_MEMBER_FIXTURE);
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    let mut files_with_their_package: Vec<(String, String)> = workspace
        .files
        .iter()
        .map(|file| {
            (
                file.relative_path.clone(),
                workspace.crates[file.crate_index.0].package_name.clone(),
            )
        })
        .collect();
    files_with_their_package.sort();
    assert_eq!(
        files_with_their_package,
        vec![
            (
                "crates/outer/inner/src/lib.rs".to_string(),
                "inner".to_string()
            ),
            ("crates/outer/src/lib.rs".to_string(), "outer".to_string()),
        ],
        "crates/outer/inner/ 底下的文件只归 inner，outer 不再收一遍"
    );
}
