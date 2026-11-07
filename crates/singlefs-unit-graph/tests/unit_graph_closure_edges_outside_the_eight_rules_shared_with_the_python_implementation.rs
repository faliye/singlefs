//! 闭包外的四类边（`scripts/gate/lib_function_units.py` 文件头 Rust 闭包「闭包外的四类边」；批次 `records/2026-11-02-一百七十六节欠账.md#6`）
//! 的 Rust 侧：`tests/fixtures/closure_edges_shared_with_python/` 下的五份夹具与那边的 `MACRO_EDGE_FIXTURE`、`ALLOCATOR_EDGE_FIXTURE`、
//! `FOREIGN_IMPL_EDGE_FIXTURE`、`BUILD_ENVIRONMENT_EDGE_FIXTURE`、`EXCLUDED_PATH_DEPENDENCY_EDGE_FIXTURE` 逐字相同，改的那一处与
//! 那边 `closure_edge_cells` 的六格相同、结论相同（规则全开时键变）；每类的弄坏开关（`src/rule_switches.rs` 文件头「闭包外的四类边」）
//! 打开时同一处改动键不变，与那边 `BREAK_CELLS` 里六个开关同一个结论。另有几条只在这一份里的边界：外部类型的两种认法各一条、
//! 宏生成的类型算本仓类型、`Tr::x` 也放行外部类型上的 impl、`concat!` 里的 `env!` 不连构建脚本、没有构建脚本时不拒算、仓根外的 path 依赖不收。
//! 工作区外的 crate 按调用关系连（那边文件头「工作区外的 path 依赖」后半）：夹具 `outside_crate_names/` 下两份与那边的
//! `OUTSIDE_CRATE_NAME_FIXTURE` 逐字相同（清单三份与 `EXCLUDED_PATH_DEPENDENCY_EDGE_FIXTURE` 共用），逐处改、跟前一把比的结论与那边
//! `outside_import_cell`、`outside_module_cell` 两格相同；开关 `outside-crate-names-imported-only`、`outside-crate-names-by-module` 打开时，
//! 那两格里键不变的几处转成键变，与那边 `BREAK_CELLS` 里两个开关同一个结论。

mod common_fixture_workspace;

use common_fixture_workspace::{key_of, workspace_of, EntrySpecification, FixtureTree};
use singlefs_unit_graph::output::dump_lines;
use singlefs_unit_graph::rule_switches::{BrokenRule, RuleSwitches};

const MACRO_EDGE_FIXTURE: [(&str, &str); 2] = [
    (
        "crates/fmacro/Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/fmacro/Cargo.toml.fixture"),
    ),
    (
        "crates/fmacro/src/lib.rs",
        include_str!("fixtures/closure_edges_shared_with_python/fmacro/src/lib.rs.fixture"),
    ),
];
const MACRO_EDGE_FILE: &str = "crates/fmacro/src/lib.rs";

const ALLOCATOR_EDGE_FIXTURE: [(&str, &str); 2] = [
    (
        "crates/falloc/Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/falloc/Cargo.toml.fixture"),
    ),
    (
        "crates/falloc/src/main.rs",
        include_str!("fixtures/closure_edges_shared_with_python/falloc/src/main.rs.fixture"),
    ),
];
const ALLOCATOR_EDGE_FILE: &str = "crates/falloc/src/main.rs";

const FOREIGN_IMPLEMENTATION_EDGE_FIXTURE: [(&str, &str); 2] = [
    (
        "crates/fforeign/Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/fforeign/Cargo.toml.fixture"),
    ),
    (
        "crates/fforeign/src/lib.rs",
        include_str!("fixtures/closure_edges_shared_with_python/fforeign/src/lib.rs.fixture"),
    ),
];
const FOREIGN_IMPLEMENTATION_EDGE_FILE: &str = "crates/fforeign/src/lib.rs";

const BUILD_ENVIRONMENT_EDGE_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/fbuild/Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/fbuild/Cargo.toml.fixture"),
    ),
    (
        "crates/fbuild/build.rs",
        include_str!("fixtures/closure_edges_shared_with_python/fbuild/build.rs.fixture"),
    ),
    (
        "crates/fbuild/src/lib.rs",
        include_str!("fixtures/closure_edges_shared_with_python/fbuild/src/lib.rs.fixture"),
    ),
];
const BUILD_SCRIPT_FILE: &str = "crates/fbuild/build.rs";

const EXCLUDED_PATH_DEPENDENCY_EDGE_FIXTURE: [(&str, &str); 5] = [
    (
        "Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/Cargo.toml.fixture"),
    ),
    (
        "crates/fuser/Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/fuser/Cargo.toml.fixture"),
    ),
    (
        "crates/fuser/src/lib.rs",
        include_str!("fixtures/closure_edges_shared_with_python/fuser/src/lib.rs.fixture"),
    ),
    (
        "fservice/Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/fservice/Cargo.toml.fixture"),
    ),
    (
        "fservice/src/lib.rs",
        include_str!("fixtures/closure_edges_shared_with_python/fservice/src/lib.rs.fixture"),
    ),
];
const EXCLUDED_PATH_DEPENDENCY_FILE: &str = "fservice/src/lib.rs";

/// 工作区外的 crate 按调用关系连：`fuser` 只在字段名里写着 `halted`；`fservice` 里 `log` 与 `bench` 两个模块各有一个 `Log`，
/// `bench` 模块里的 `fn launch` 与 `Worker::launch` 同名。清单与 [`EXCLUDED_PATH_DEPENDENCY_EDGE_FIXTURE`] 的三份相同。
const OUTSIDE_CRATE_NAME_FIXTURE: [(&str, &str); 5] = [
    (
        "Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/Cargo.toml.fixture"),
    ),
    (
        "crates/fuser/Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/fuser/Cargo.toml.fixture"),
    ),
    (
        "crates/fuser/src/lib.rs",
        include_str!(
            "fixtures/closure_edges_shared_with_python/outside_crate_names/fuser/src/lib.rs.fixture"
        ),
    ),
    (
        "fservice/Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/fservice/Cargo.toml.fixture"),
    ),
    (
        "fservice/src/lib.rs",
        include_str!(
            "fixtures/closure_edges_shared_with_python/outside_crate_names/fservice/src/lib.rs.fixture"
        ),
    ),
];

/// 外部类型写成路径、它的末段名字在工作区里另有一个本仓类型（没链接的 `flocalduration` 里的 `Duration`）：单按名字认会当成本仓类型，
/// 按路径头一段 `std` 认才是外部类型（python 那一份文件头「单按名字认不够」那一条）。
const FOREIGN_PATH_TYPE_SHARING_A_LOCAL_NAME_FIXTURE: [(&str, &str); 4] = [
    (
        "crates/fforeigncollision/Cargo.toml",
        "[package]\nname = \"fforeigncollision\"\n",
    ),
    (
        "crates/fforeigncollision/src/lib.rs",
        "pub trait Measured {\n    fn measure(&self) -> u64;\n}\n\nimpl Measured for std::time::Duration {\n    fn measure(&self) -> u64 {\n        self.as_secs() + 1\n    }\n}\n\npub fn measure_of<Value>(value: &Value) -> u64\nwhere\n    Value: Measured,\n{\n    value.measure()\n}\n\npub fn measured_second() -> u64 {\n    measure_of(&std::time::Duration::from_secs(1))\n}\n",
    ),
    (
        "crates/flocalduration/Cargo.toml",
        "[package]\nname = \"flocalduration\"\n",
    ),
    (
        "crates/flocalduration/src/lib.rs",
        "pub struct Duration;\n",
    ),
];

/// 外部类型经 `use` 引进、impl 里不写路径：自身类型里的名字没有一个是本仓类型，照样是外部类型。
const IMPORTED_FOREIGN_TYPE_FIXTURE: [(&str, &str); 2] = [
    (
        "crates/fforeignimported/Cargo.toml",
        "[package]\nname = \"fforeignimported\"\n",
    ),
    (
        "crates/fforeignimported/src/lib.rs",
        "use std::time::Duration;\n\npub trait Measured {\n    fn measure(&self) -> u64;\n}\n\nimpl Measured for Duration {\n    fn measure(&self) -> u64 {\n        self.as_secs() + 1\n    }\n}\n\npub fn measure_of<Value>(value: &Value) -> u64\nwhere\n    Value: Measured,\n{\n    value.measure()\n}\n\npub fn measured_second() -> u64 {\n    measure_of(&Duration::from_secs(1))\n}\n",
    ),
];

/// 本仓 trait 写在条目级宏调用生成的类型 `Weight` 上：`Weight` 算本仓类型，这个 impl 不是外部类型上的 impl，等 `Weight` 被点名才连。
const MACRO_GENERATED_LOCAL_TYPE_FIXTURE: [(&str, &str); 2] = [
    (
        "crates/fforeignmacro/Cargo.toml",
        "[package]\nname = \"fforeignmacro\"\n",
    ),
    (
        "crates/fforeignmacro/src/lib.rs",
        "macro_rules! listed_enumeration {\n    ($visibility:vis enum $name:ident { $($variant:ident,)+ }) => {\n        $visibility enum $name { $($variant,)+ }\n    };\n}\nlisted_enumeration! {\n    pub enum Weight {\n        Light,\n        Heavy,\n    }\n}\n\npub trait Measured {\n    fn measure(&self) -> u64;\n}\n\nimpl Measured for Weight {\n    fn measure(&self) -> u64 {\n        5\n    }\n}\n\nimpl Measured for std::time::Duration {\n    fn measure(&self) -> u64 {\n        self.as_secs() + 1\n    }\n}\n\npub fn measure_of<Value>(value: &Value) -> u64\nwhere\n    Value: Measured,\n{\n    value.measure()\n}\n\npub fn measured_second() -> u64 {\n    measure_of(&std::time::Duration::from_secs(1))\n}\n",
    ),
];

/// R-路径 `Tr::x`：经 trait 路径调到外部类型上的本仓 trait impl。
const FOREIGN_IMPLEMENTATION_THROUGH_A_TRAIT_PATH_FIXTURE: [(&str, &str); 2] = [
    (
        "crates/fforeignpath/Cargo.toml",
        "[package]\nname = \"fforeignpath\"\n",
    ),
    (
        "crates/fforeignpath/src/lib.rs",
        "pub trait Measured {\n    fn measure(&self) -> u64;\n}\n\nimpl Measured for std::time::Duration {\n    fn measure(&self) -> u64 {\n        self.as_secs() + 1\n    }\n}\n\npub fn measured_through_the_trait_path() -> u64 {\n    Measured::measure(&std::time::Duration::from_secs(1))\n}\n",
    ),
];

/// `concat!(…)` 里的 `env!`：python 那一份整段跳过 `concat!`，不连构建脚本；这一份照同样的边界。
const ENVIRONMENT_VALUE_INSIDE_A_CONCATENATION_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/fbuildconcat/Cargo.toml",
        "[package]\nname = \"fbuildconcat\"\n",
    ),
    (
        "crates/fbuildconcat/build.rs",
        "fn reported_features() -> String {\n    String::from(\"plain\")\n}\n\nfn main() {\n    println!(\"cargo::rustc-env=FBUILD_REPORTED_FEATURES={}\", reported_features());\n}\n",
    ),
    (
        "crates/fbuildconcat/src/lib.rs",
        "pub fn reported_inside_a_concatenation() -> &'static str {\n    concat!(env!(\"FBUILD_REPORTED_FEATURES\"), \"\")\n}\n",
    ),
];

/// 读一个非 cargo 变量、crate 却没有构建脚本：值来自编译时的进程环境，不连、不拒算。
const ENVIRONMENT_VALUE_WITHOUT_A_BUILD_SCRIPT_FIXTURE: [(&str, &str); 2] = [
    (
        "crates/fnobuild/Cargo.toml",
        "[package]\nname = \"fnobuild\"\n",
    ),
    (
        "crates/fnobuild/src/lib.rs",
        "pub fn reported() -> &'static str {\n    env!(\"FNOBUILD_REPORTED_FEATURES\")\n}\n",
    ),
];

/// `fuser` 用 `use fservice::*;` 整个引进 `fservice`：哪个名字都可能引进来了，字段名 `halted` 照同名连到 `fservice` 的 `fn halted`
/// （python 那边 `names_reaching_outside_crate` 的「根名后面有 * 的全收」；那边的夹具里没有 glob，这一条只在这一份里）。
const OUTSIDE_CRATE_GLOB_IMPORT_FIXTURE: [(&str, &str); 5] = [
    (
        "Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/Cargo.toml.fixture"),
    ),
    (
        "crates/fuser/Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/fuser/Cargo.toml.fixture"),
    ),
    (
        "crates/fuser/src/lib.rs",
        "use fservice::*;\n\npub struct Counters {\n    pub halted: u32,\n}\n\npub fn served() -> u32 {\n    let counters = Counters { halted: 1 };\n    restarts() + counters.halted\n}\n",
    ),
    (
        "fservice/Cargo.toml",
        include_str!("fixtures/closure_edges_shared_with_python/fservice/Cargo.toml.fixture"),
    ),
    (
        "fservice/src/lib.rs",
        "pub fn restarts() -> u32 {\n    4\n}\n\npub fn halted() -> u32 {\n    6\n}\n",
    ),
];

/// 仓根是 `repository/`：path 依赖 `../../../outside` 落在仓根外面，不收。
const PATH_DEPENDENCY_OUTSIDE_THE_ROOT_FIXTURE: [(&str, &str); 4] = [
    (
        "repository/crates/fuser/Cargo.toml",
        "[package]\nname = \"fuser\"\n\n[dependencies]\noutside = { path = \"../../../outside\" }\n",
    ),
    (
        "repository/crates/fuser/src/lib.rs",
        "pub fn stop_count() -> u32 {\n    outside::stopped_connections()\n}\n",
    ),
    ("outside/Cargo.toml", "[package]\nname = \"outside\"\n"),
    (
        "outside/src/lib.rs",
        "pub fn stopped_connections() -> u32 {\n    3\n}\n",
    ),
];

/// 改了之后键怎样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyExpectation {
    Changes,
    StaysTheSame,
}

/// 在夹具里改的那一处：哪份文件、恰好出现一次的原文、换成什么。
#[derive(Clone, Copy, Debug)]
struct FixtureEdit {
    file: &'static str,
    original: &'static str,
    replacement: &'static str,
}

const fn library_entry(entry_path: &'static str) -> EntrySpecification {
    EntrySpecification {
        entry_path,
        target: "lib",
        conditions: &[],
    }
}

/// 同一组开关下，夹具改那一处之前与之后入口的键变不变；两趟都要算得出键，算不出就 panic（带入口错或拒算行）。
fn key_after_editing(
    fixture: &[(&str, &str)],
    entry: &EntrySpecification,
    edit: &FixtureEdit,
    switches: &RuleSwitches,
) -> KeyExpectation {
    let base = FixtureTree::materialize("closure-edges-base", fixture);
    let base_workspace = workspace_of(base.root(), switches);
    let edited = FixtureTree::materialize("closure-edges-edited", fixture);
    edited.replace_exactly_once(edit.file, edit.original, edit.replacement);
    let edited_workspace = workspace_of(edited.root(), switches);
    let key_before = key_of(&base_workspace, entry, switches)
        .unwrap_or_else(|reason| panic!("{} 改前要算得出键：{reason}", entry.entry_path));
    let key_after = key_of(&edited_workspace, entry, switches)
        .unwrap_or_else(|reason| panic!("{} 改后要算得出键：{reason}", entry.entry_path));
    if key_before == key_after {
        KeyExpectation::StaysTheSame
    } else {
        KeyExpectation::Changes
    }
}

/// 依次改夹具里的几处（改了不还原），每改一处算一次入口的键、跟前一把比（python 那边 `outside_crate_edit_verdicts` 同一个比法）；
/// 每一趟都要算得出键，算不出就 panic（带入口错或拒算行）。
fn key_expectations_after_successive_edits(
    fixture: &[(&str, &str)],
    entry: &EntrySpecification,
    edits: &[FixtureEdit],
    switches: &RuleSwitches,
) -> Vec<KeyExpectation> {
    let tree = FixtureTree::materialize("closure-edges-successive", fixture);
    let mut previous_key = key_of(&workspace_of(tree.root(), switches), entry, switches)
        .unwrap_or_else(|reason| panic!("{} 改前要算得出键：{reason}", entry.entry_path));
    edits
        .iter()
        .map(|edit| {
            tree.replace_exactly_once(edit.file, edit.original, edit.replacement);
            let key = key_of(&workspace_of(tree.root(), switches), entry, switches).unwrap_or_else(
                |reason| {
                    panic!(
                        "{} 改了「{}」之后要算得出键：{reason}",
                        entry.entry_path, edit.original
                    )
                },
            );
            let expectation = if key == previous_key {
                KeyExpectation::StaysTheSame
            } else {
                KeyExpectation::Changes
            };
            previous_key = key;
            expectation
        })
        .collect()
}

const MACRO_GENERATED_TYPE_ENTRY: EntrySpecification = library_entry("fmacro::heavy_amount");
const MACRO_GENERATED_TYPE_EDIT: FixtureEdit = FixtureEdit {
    file: MACRO_EDGE_FILE,
    original: "Weight::Heavy => 9,",
    replacement: "Weight::Heavy => 8,",
};

const MACRO_RULES_LITERAL_NAME_ENTRY: EntrySpecification = library_entry("fmacro::code_count");
const MACRO_RULES_LITERAL_NAME_EDIT: FixtureEdit = FixtureEdit {
    file: MACRO_EDGE_FILE,
    original: "CODE_FULL = 7,",
    replacement: "CODE_FULL = 8,",
};

const GLOBAL_ALLOCATOR_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "falloc::growths",
    target: "bin:falloc",
    conditions: &[],
};
const GLOBAL_ALLOCATOR_EDIT: FixtureEdit = FixtureEdit {
    file: ALLOCATOR_EDGE_FILE,
    original: "GROWTHS.fetch_add(1, Ordering::Relaxed);",
    replacement: "GROWTHS.fetch_add(2, Ordering::Relaxed);",
};

const FOREIGN_IMPLEMENTATION_ENTRY: EntrySpecification = library_entry("fforeign::measured_second");
const FOREIGN_IMPLEMENTATION_EDIT: FixtureEdit = FixtureEdit {
    file: FOREIGN_IMPLEMENTATION_EDGE_FILE,
    original: "self.as_secs() + 1",
    replacement: "self.as_secs() + 2",
};

const BUILD_ENVIRONMENT_ENTRY: EntrySpecification = library_entry("fbuild::reported");
const BUILD_ENVIRONMENT_EDIT: FixtureEdit = FixtureEdit {
    file: BUILD_SCRIPT_FILE,
    original: "String::from(\"plain\")",
    replacement: "String::from(\"gpu\")",
};

const OUTSIDE_CRATE_NAME_ENTRY: EntrySpecification = library_entry("fuser::served");
const OUTSIDE_CRATE_NAME_FILE: &str = "fservice/src/lib.rs";
/// python 那边 `outside_import_cell` 依次改的四处：`fn halted`、`fn restarts`、`control::paused`、`fn stopped_connections`。
const OUTSIDE_CRATE_IMPORT_EDITS: [FixtureEdit; 4] = [
    FixtureEdit {
        file: OUTSIDE_CRATE_NAME_FILE,
        original: "    6\n",
        replacement: "    60\n",
    },
    FixtureEdit {
        file: OUTSIDE_CRATE_NAME_FILE,
        original: "    4\n",
        replacement: "    40\n",
    },
    FixtureEdit {
        file: OUTSIDE_CRATE_NAME_FILE,
        original: "        5\n",
        replacement: "        50\n",
    },
    FixtureEdit {
        file: OUTSIDE_CRATE_NAME_FILE,
        original: "Worker.launch() + Log",
        replacement: "Worker.launch() + 1 + Log",
    },
];
/// python 那边 `outside_module_cell` 依次改的四处：`bench::Log`、`bench::launch`、`log::Log`、`Worker::launch`。
const OUTSIDE_CRATE_MODULE_EDITS: [FixtureEdit; 4] = [
    FixtureEdit {
        file: OUTSIDE_CRATE_NAME_FILE,
        original: "pub entries: u64,",
        replacement: "pub entries: u16,",
    },
    FixtureEdit {
        file: OUTSIDE_CRATE_NAME_FILE,
        original: "        7\n",
        replacement: "        70\n",
    },
    FixtureEdit {
        file: OUTSIDE_CRATE_NAME_FILE,
        original: "pub entries: u32,",
        replacement: "pub entries: u8,",
    },
    FixtureEdit {
        file: OUTSIDE_CRATE_NAME_FILE,
        original: "        2\n",
        replacement: "        20\n",
    },
];

const EXCLUDED_PATH_DEPENDENCY_ENTRY: EntrySpecification = library_entry("fuser::stop_count");
const EXCLUDED_PATH_DEPENDENCY_EDIT: FixtureEdit = FixtureEdit {
    file: EXCLUDED_PATH_DEPENDENCY_FILE,
    original: "    3\n",
    replacement: "    4\n",
};

/// python 那边「R-宏：条目级宏调用里 enum 定义的类型算被点名，.方法() 连到写给它的 impl，改方法作废」。
#[test]
fn type_generated_by_an_item_macro_is_named_and_editing_its_inherent_method_changes_the_key() {
    assert_eq!(
        key_after_editing(
            &MACRO_EDGE_FIXTURE,
            &MACRO_GENERATED_TYPE_ENTRY,
            &MACRO_GENERATED_TYPE_EDIT,
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::Changes,
        "heavy_amount 点名的 Weight 是 listed_enumeration! 生成的：调用进闭包时 Weight 算被点名，impl Weight 的 amount 连上"
    );
}

/// 弄坏开关 macro-generated-types（python 的 rust-macro-generated-types-unnamed）：同一处改动键不变。
#[test]
fn with_macro_generated_types_broken_editing_the_inherent_method_keeps_the_key() {
    assert_eq!(
        key_after_editing(
            &MACRO_EDGE_FIXTURE,
            &MACRO_GENERATED_TYPE_ENTRY,
            &MACRO_GENERATED_TYPE_EDIT,
            &RuleSwitches::with_broken(&[BrokenRule::MacroGeneratedTypes]),
        ),
        KeyExpectation::StaysTheSame,
        "宏生成的类型不算被点名时 impl Weight 不进 heavy_amount 的闭包"
    );
}

/// python 那边「R-宏：macro_rules! 体里按字面写出的常量名登到这个宏的每次调用名下，点名它的入口连上调用，改调用体作废」。
#[test]
fn literal_name_in_a_macro_rules_expansion_links_its_invocation_and_editing_the_invocation_changes_the_key(
) {
    assert_eq!(
        key_after_editing(
            &MACRO_EDGE_FIXTURE,
            &MACRO_RULES_LITERAL_NAME_ENTRY,
            &MACRO_RULES_LITERAL_NAME_EDIT,
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::Changes,
        "code_count 点名的 EVERY_CODE 是 code_table! 宏体顶层按字面写出的：它登到 code_table! 的调用名下，调用进闭包"
    );
}

/// 弄坏开关 macro-rules-literal-names（python 的 rust-macro-rules-literal-names-unlinked）：同一处改动键不变。
#[test]
fn with_macro_rules_literal_names_broken_editing_the_invocation_keeps_the_key() {
    assert_eq!(
        key_after_editing(
            &MACRO_EDGE_FIXTURE,
            &MACRO_RULES_LITERAL_NAME_ENTRY,
            &MACRO_RULES_LITERAL_NAME_EDIT,
            &RuleSwitches::with_broken(&[BrokenRule::MacroRulesLiteralNames]),
        ),
        KeyExpectation::StaysTheSame,
        "宏体里的字面名字不登时 code_table! 的调用不进 code_count 的闭包"
    );
}

/// python 那边「#[global_allocator]：链接集里的分配器 static 进闭包，连上它的 GlobalAlloc impl，改 realloc 作废」。
#[test]
fn global_allocator_of_the_linked_binary_is_in_the_closure_and_editing_its_realloc_changes_the_key()
{
    assert_eq!(
        key_after_editing(
            &ALLOCATOR_EDGE_FIXTURE,
            &GLOBAL_ALLOCATOR_ENTRY,
            &GLOBAL_ALLOCATOR_EDIT,
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::Changes,
        "growths 所在的 bin 里有 #[global_allocator] static ALLOCATOR：它进闭包，经 CountingAllocator 带上 GlobalAlloc 的 impl"
    );
}

/// 弄坏开关 global-allocator-edges（python 的 rust-global-allocators-unlinked）：同一处改动键不变。
#[test]
fn with_global_allocator_edges_broken_editing_realloc_keeps_the_key() {
    assert_eq!(
        key_after_editing(
            &ALLOCATOR_EDGE_FIXTURE,
            &GLOBAL_ALLOCATOR_ENTRY,
            &GLOBAL_ALLOCATOR_EDIT,
            &RuleSwitches::with_broken(&[BrokenRule::GlobalAllocatorEdges]),
        ),
        KeyExpectation::StaysTheSame,
        "分配器 static 不进闭包时 growths 够不到 realloc"
    );
}

/// python 那边「R-方法：本仓 trait 写在外部类型上的 impl（impl Tr for std::time::Duration）经泛型约束调到，方法连上，改它作废」。
#[test]
fn local_trait_implemented_on_a_foreign_path_type_is_reached_through_a_generic_bound_and_editing_it_changes_the_key(
) {
    assert_eq!(
        key_after_editing(
            &FOREIGN_IMPLEMENTATION_EDGE_FIXTURE,
            &FOREIGN_IMPLEMENTATION_ENTRY,
            &FOREIGN_IMPLEMENTATION_EDIT,
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::Changes,
        "measure_of 里的 .measure() 落到 impl Measured for std::time::Duration 的 measure：自身类型是外部类型，不等它被点名就放行"
    );
}

/// 弄坏开关 foreign-type-trait-implementations（python 的 rust-foreign-self-trait-impls-unlinked）：同一处改动键不变。
#[test]
fn with_foreign_type_trait_implementations_broken_editing_that_implementation_keeps_the_key() {
    assert_eq!(
        key_after_editing(
            &FOREIGN_IMPLEMENTATION_EDGE_FIXTURE,
            &FOREIGN_IMPLEMENTATION_ENTRY,
            &FOREIGN_IMPLEMENTATION_EDIT,
            &RuleSwitches::with_broken(&[BrokenRule::ForeignTypeTraitImplementations]),
        ),
        KeyExpectation::StaysTheSame,
        "外部类型上的 impl 不放行时 Duration 永远点不到，measure 不进闭包"
    );
}

/// 自身类型写成路径 `std::time::Duration`、工作区里另有一个本仓 `struct Duration`：按路径头一段认，照样是外部类型。
#[test]
fn foreign_path_type_whose_last_segment_names_a_local_type_still_counts_as_foreign() {
    assert_eq!(
        key_after_editing(
            &FOREIGN_PATH_TYPE_SHARING_A_LOCAL_NAME_FIXTURE,
            &library_entry("fforeigncollision::measured_second"),
            &FixtureEdit {
                file: "crates/fforeigncollision/src/lib.rs",
                original: "self.as_secs() + 1",
                replacement: "self.as_secs() + 2",
            },
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::Changes,
        "impl Measured for std::time::Duration 的路径头一段是 std：外部类型，measure 连上（单按名字认会被 flocalduration 的 Duration 挡住）"
    );
}

/// 外部类型经 `use` 引进、impl 里只写 `Duration`：自身类型里的名字没有一个是本仓类型，照样是外部类型。
#[test]
fn imported_foreign_type_written_without_a_path_counts_as_foreign() {
    assert_eq!(
        key_after_editing(
            &IMPORTED_FOREIGN_TYPE_FIXTURE,
            &library_entry("fforeignimported::measured_second"),
            &FixtureEdit {
                file: "crates/fforeignimported/src/lib.rs",
                original: "self.as_secs() + 1",
                replacement: "self.as_secs() + 2",
            },
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::Changes,
        "impl Measured for Duration 的 Duration 不是本仓定义的类型：外部类型，measure 连上"
    );
}

/// 本仓 trait 写在条目级宏调用生成的 `Weight` 上：宏生成的类型算本仓类型，这个 impl 不是外部类型上的 impl，入口没点名 `Weight` 就不连。
#[test]
fn local_trait_implemented_on_a_macro_generated_type_is_not_foreign_and_editing_it_keeps_the_key() {
    assert_eq!(
        key_after_editing(
            &MACRO_GENERATED_LOCAL_TYPE_FIXTURE,
            &library_entry("fforeignmacro::measured_second"),
            &FixtureEdit {
                file: "crates/fforeignmacro/src/lib.rs",
                original: "        5\n",
                replacement: "        6\n",
            },
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::StaysTheSame,
        "Weight 是 listed_enumeration! 生成的本仓类型：impl Measured for Weight 等 Weight 被点名才连，measured_second 没点它"
    );
}

/// R-路径：`Measured::measure(…)` 经 trait 路径调到外部类型上的本仓 trait impl，同样不等自身类型被点名就放行。
#[test]
fn local_trait_implemented_on_a_foreign_type_is_reached_through_a_trait_path_call() {
    assert_eq!(
        key_after_editing(
            &FOREIGN_IMPLEMENTATION_THROUGH_A_TRAIT_PATH_FIXTURE,
            &library_entry("fforeignpath::measured_through_the_trait_path"),
            &FixtureEdit {
                file: "crates/fforeignpath/src/lib.rs",
                original: "self.as_secs() + 1",
                replacement: "self.as_secs() + 2",
            },
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::Changes,
        "Measured::measure 落到 impl Measured for std::time::Duration 的 measure"
    );
}

/// python 那边「构建脚本经 cargo::rustc-env 交的值：用 env! 读它的单元连到本 crate 构建脚本的 main，改 main 里算的值作废」。
#[test]
fn build_script_value_read_with_the_environment_macro_links_the_build_script_main_and_editing_it_changes_the_key(
) {
    assert_eq!(
        key_after_editing(
            &BUILD_ENVIRONMENT_EDGE_FIXTURE,
            &BUILD_ENVIRONMENT_ENTRY,
            &BUILD_ENVIRONMENT_EDIT,
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::Changes,
        "reported 读 env!(\"FBUILD_REPORTED_FEATURES\")：不是 cargo 给的变量，连到 build.rs 的 main，main 调 reported_features"
    );
}

/// 弄坏开关 build-script-environment-edges（python 的 rust-build-environment-unlinked）：同一处改动键不变。
#[test]
fn with_build_script_environment_edges_broken_editing_the_build_script_keeps_the_key() {
    assert_eq!(
        key_after_editing(
            &BUILD_ENVIRONMENT_EDGE_FIXTURE,
            &BUILD_ENVIRONMENT_ENTRY,
            &BUILD_ENVIRONMENT_EDIT,
            &RuleSwitches::with_broken(&[BrokenRule::BuildScriptEnvironmentEdges]),
        ),
        KeyExpectation::StaysTheSame,
        "不连构建脚本时 reported 的闭包里没有 build.rs 的单元"
    );
}

/// `concat!(env!("X"), …)`：python 那一份整段跳过 `concat!`，里面的 `env!` 不连构建脚本；这一份照同样的边界，改 build.rs 键不变。
#[test]
fn environment_macro_inside_a_concatenation_does_not_link_the_build_script() {
    assert_eq!(
        key_after_editing(
            &ENVIRONMENT_VALUE_INSIDE_A_CONCATENATION_FIXTURE,
            &library_entry("fbuildconcat::reported_inside_a_concatenation"),
            &FixtureEdit {
                file: "crates/fbuildconcat/build.rs",
                original: "String::from(\"plain\")",
                replacement: "String::from(\"gpu\")",
            },
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::StaysTheSame,
        "concat! 里的 env! 两份实现都不当构建脚本交的值"
    );
}

/// 读非 cargo 变量的 crate 没有构建脚本：不连、也不拒算（`env!("OUT_DIR")` 那一条找不到构建脚本才拒算 build-script-missing）。
#[test]
fn build_script_value_read_without_a_build_script_is_not_refused() {
    let tree = FixtureTree::materialize(
        "closure-edges-no-build-script",
        &ENVIRONMENT_VALUE_WITHOUT_A_BUILD_SCRIPT_FIXTURE,
    );
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let key = key_of(&workspace, &library_entry("fnobuild::reported"), &switches);
    assert!(
        key.is_ok(),
        "没有构建脚本时 env!(\"FNOBUILD_REPORTED_FEATURES\") 不连、不拒算，键算得出：{key:?}"
    );
}

/// python 那边「工作区外的 path 依赖（根清单 exclude 里）：顺 path 依赖收进来，调到它的函数连上，改它作废」。
#[test]
fn path_dependency_excluded_from_the_workspace_is_collected_and_editing_it_changes_the_key() {
    assert_eq!(
        key_after_editing(
            &EXCLUDED_PATH_DEPENDENCY_EDGE_FIXTURE,
            &EXCLUDED_PATH_DEPENDENCY_ENTRY,
            &EXCLUDED_PATH_DEPENDENCY_EDIT,
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::Changes,
        "fuser 经 path 依赖 ../../fservice 调 stopped_connections：fservice 在 crates/ 外、根清单 exclude 里，照样收成一个 crate"
    );
}

/// 弄坏开关 path-dependencies-outside-the-workspace（python 的 rust-excluded-path-dependencies-dropped）：同一处改动键不变。
#[test]
fn with_path_dependencies_outside_the_workspace_broken_editing_the_dependency_keeps_the_key() {
    assert_eq!(
        key_after_editing(
            &EXCLUDED_PATH_DEPENDENCY_EDGE_FIXTURE,
            &EXCLUDED_PATH_DEPENDENCY_ENTRY,
            &EXCLUDED_PATH_DEPENDENCY_EDIT,
            &RuleSwitches::with_broken(&[BrokenRule::PathDependenciesOutsideTheWorkspace]),
        ),
        KeyExpectation::StaysTheSame,
        "不顺 path 依赖收时 fservice 不在工作区里，stopped_connections 解不到"
    );
}

/// 顺 path 依赖收进来的 crate 的单元出现在 dump 里（两份实现对拍的 dump 那一半：真仓上是 `prune-server/` 下的单元）。
#[test]
fn the_dump_lists_the_units_of_a_path_dependency_outside_the_crates_directory() {
    let tree =
        FixtureTree::materialize("closure-edges-dump", &EXCLUDED_PATH_DEPENDENCY_EDGE_FIXTURE);
    let workspace = workspace_of(tree.root(), &RuleSwitches::every_rule_intact());
    let lines = dump_lines(&workspace);
    assert!(
        lines.iter().any(|line| line
            .split('\t')
            .nth(2)
            .is_some_and(|place| place.starts_with("fservice/src/lib.rs:"))),
        "dump 里要有 fservice/src/lib.rs 的单元：{lines:?}"
    );
}

/// path 依赖落在仓根外面（仓根是 `repository/`，依赖写 `../../../outside`）：不收，工作区里没有 `outside` 这个 crate。
#[test]
fn path_dependency_outside_the_repository_root_is_not_collected() {
    let tree = FixtureTree::materialize(
        "closure-edges-outside-the-root",
        &PATH_DEPENDENCY_OUTSIDE_THE_ROOT_FIXTURE,
    );
    let workspace = workspace_of(
        &tree.root().join("repository"),
        &RuleSwitches::every_rule_intact(),
    );
    assert!(
        workspace.crate_named("outside").is_none(),
        "仓根外的 path 依赖不收：{:?}",
        workspace
            .crates
            .iter()
            .map(|workspace_crate| workspace_crate.package_name.clone())
            .collect::<Vec<String>>()
    );
}

/// python 那边「工作区外的 crate 按引进的名字连：别的包里只是同名的字段 halted 不连到它的 fn halted，改它键不变；use 引进的 restarts、
/// use 引进的模块下的 control::paused、写成路径的 stopped_connections 照连，改它们键变」。
#[test]
fn outside_crate_names_link_from_another_crate_only_when_imported_through_a_use_or_a_path() {
    assert_eq!(
        key_expectations_after_successive_edits(
            &OUTSIDE_CRATE_NAME_FIXTURE,
            &OUTSIDE_CRATE_NAME_ENTRY,
            &OUTSIDE_CRATE_IMPORT_EDITS,
            &RuleSwitches::every_rule_intact(),
        ),
        [
            KeyExpectation::StaysTheSame,
            KeyExpectation::Changes,
            KeyExpectation::Changes,
            KeyExpectation::Changes,
        ],
        "fuser 里 halted 只是 Counters 的字段名，没经 use 或路径从 fservice 引进：不连 fn halted；restarts、control::paused、\
         fservice::stopped_connections 都引进了，照连"
    );
}

/// 弄坏开关 outside-crate-names-imported-only（python 的 rust-outside-crate-names-linked-unimported）：字段名 halted 照旧按同名连到
/// fservice 的 fn halted，改它键也变。
#[test]
fn with_outside_crate_names_imported_only_broken_a_field_named_like_an_outside_function_links_it() {
    assert_eq!(
        key_expectations_after_successive_edits(
            &OUTSIDE_CRATE_NAME_FIXTURE,
            &OUTSIDE_CRATE_NAME_ENTRY,
            &OUTSIDE_CRATE_IMPORT_EDITS,
            &RuleSwitches::with_broken(&[BrokenRule::OutsideCrateNamesImportedOnly]),
        ),
        [
            KeyExpectation::Changes,
            KeyExpectation::Changes,
            KeyExpectation::Changes,
            KeyExpectation::Changes,
        ],
        "不要求引进时 served 的字段名 halted 连到 fservice 的 fn halted"
    );
}

/// python 那边「工作区外的 crate 里按模块连：use 引进 log::Log 的只连它、不连 bench 模块里同名的 Log，Worker::launch 自己的名字不连
/// bench::launch；改 bench 模块里这两样键不变，改 log::Log、改 Worker::launch 键变」。
#[test]
fn outside_crate_names_inside_the_crate_link_only_the_items_seen_from_the_module_of_the_unit() {
    assert_eq!(
        key_expectations_after_successive_edits(
            &OUTSIDE_CRATE_NAME_FIXTURE,
            &OUTSIDE_CRATE_NAME_ENTRY,
            &OUTSIDE_CRATE_MODULE_EDITS,
            &RuleSwitches::every_rule_intact(),
        ),
        [
            KeyExpectation::StaysTheSame,
            KeyExpectation::StaysTheSame,
            KeyExpectation::Changes,
            KeyExpectation::Changes,
        ],
        "stopped_connections 经 use log::Log 用 Log：只连 log::Log；Worker::launch 自己的名字 launch 不连 bench::launch"
    );
}

/// 弄坏开关 outside-crate-names-by-module（python 的 rust-outside-crate-names-linked-across-modules）：bench 模块里同名的 Log 与
/// launch 照旧按同名连，改它们键也变。
#[test]
fn with_outside_crate_names_by_module_broken_the_same_names_in_another_module_link_too() {
    assert_eq!(
        key_expectations_after_successive_edits(
            &OUTSIDE_CRATE_NAME_FIXTURE,
            &OUTSIDE_CRATE_NAME_ENTRY,
            &OUTSIDE_CRATE_MODULE_EDITS,
            &RuleSwitches::with_broken(&[BrokenRule::OutsideCrateNamesByModule]),
        ),
        [
            KeyExpectation::Changes,
            KeyExpectation::Changes,
            KeyExpectation::Changes,
            KeyExpectation::Changes,
        ],
        "不按模块筛时 Log 连到两个模块里的 Log，launch 连到 bench::launch"
    );
}

/// 别的包 `use 库名::*;` 整个引进工作区外的 crate：字段名 halted 照同名连到它的 fn halted，改它键变。
#[test]
fn outside_crate_names_all_link_from_another_crate_after_a_glob_import() {
    assert_eq!(
        key_after_editing(
            &OUTSIDE_CRATE_GLOB_IMPORT_FIXTURE,
            &OUTSIDE_CRATE_NAME_ENTRY,
            &FixtureEdit {
                file: OUTSIDE_CRATE_NAME_FILE,
                original: "    6\n",
                replacement: "    60\n",
            },
            &RuleSwitches::every_rule_intact(),
        ),
        KeyExpectation::Changes,
        "use fservice::* 之后 fservice 的每个名字都算引进来了：served 的字段名 halted 连到 fn halted"
    );
}
