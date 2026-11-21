//! 几种看不出运行时调到谁的写法：闭包要收宽（照连同名函数、退回全连），或者拒算，不许静默少收（D13（验证路线） 已定项 17
//! 「收不全的那一份拒算键、照跑，不静默少收」「键过窄…同样算违反」）。
//! - 宏调用括号里的名字（`src/closure.rs` 的 `is_bare_name_only_a_local_binding_of_the_function`）：工作区的宏可能把它接在路径后面、放进嵌套条目，
//!   照连同名函数；只有标准库那 16 个就地求值实参的宏（`STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE`）的括号里照算局部，
//!   名单每一项各一条用例；名单外的标准库宏（`dbg!`）照连。
//! - use 路径点准候选（`candidates_seen_from_the_module_of_the_unit`）：点到的候选带条件编译、那个模块另有 use 按名字引进它、
//!   相对路径首段是本作用域里哪条 use 引进的名字（同一条 use 里先改名的、`use 甲::{self, …}` 引进的也算）、或本作用域里有任一次
//!   条目级宏调用（不看宏里写没写首段），退回全连。
//! - 程序起自己（`src/rust_items.rs` 的 `own_program_path_is_handed_to_command_unchanged`、`expression_ending_at_diverges`）：用法出了 `let` 所在的块、
//!   进了比 `let` 更深的块、失败时的闭包以 `if … else` 收尾的，不算起自己，拒算 `current-exe`。
//! - `extern` 块名单（`extern_block_declares_only_listed_system_library_calls` 与闭包的
//!   `final_product_may_link_libraries_beyond_the_system_library`）：属性里任何位置有 `link`；名单块所在的包与入口所在的包、连同它们顺依赖
//!   （含构建依赖）够到的包里，任一目标另有块带 `link`、构建脚本或构建时跑的包的库里有以 `cargo:`（或 `cargo::`）接
//!   `rustc-link-lib`、`rustc-link-arg`、`rustc-flags` 起头的串、依赖里有工作区外的包的，拒算 `extern-function-interface`；
//!   没有依赖关系的包里的这些、只写指令名不带 `cargo:` 的串、构建依赖的测试目标里的串不看。
//!
//! 夹具只切词、不编译；判的是闭包里有没有点名的单元、拒算行点没点名那一形。

mod common_fixture_workspace;

use common_fixture_workspace::{
    closure_with_triple, key_of, workspace_of, EntrySpecification, FixtureTree, HOST_TARGET_TRIPLE,
};
use singlefs_unit_graph::rule_switches::RuleSwitches;

const NO_DEPENDENCY_RESOLUTION: (&str, &str) = ("Cargo.lock", "# 夹具不解析依赖\n");

/// 入口闭包里各 Rust 单元的限定名；闭包收不起来就 panic。
fn closure_unit_names(tree: &FixtureTree, entry: &EntrySpecification) -> Vec<String> {
    let switches = RuleSwitches::every_rule_intact();
    let workspace = workspace_of(tree.root(), &switches);
    let outcome = closure_with_triple(&workspace, entry, &switches, HOST_TARGET_TRIPLE)
        .unwrap_or_else(|error| panic!("{} 的闭包收得起来：{error}", entry.entry_path));
    outcome
        .units
        .iter()
        .map(|unit_index| workspace.units[unit_index.0].qualified_name.clone())
        .collect()
}

/// 入口的键：拒算交 `Err`（带拒算行）。
fn key_verdict(tree: &FixtureTree, entry: &EntrySpecification) -> Result<String, String> {
    let switches = RuleSwitches::every_rule_intact();
    key_of(&workspace_of(tree.root(), &switches), entry, &switches).map(|key| key.hexadecimal())
}

fn library_entry(entry_path: &'static str) -> EntrySpecification {
    EntrySpecification {
        entry_path,
        target: "lib",
        conditions: &[],
    }
}

const MACRO_FILE: &str = "crates/fshadow/src/lib.rs";

/// 宏把实参接在路径后面、把实参块放进嵌套函数的两个入口与一个对照入口，加标准库宏、宏里块中的 `let`、改名原名与区间几个入口。
const MACRO_LIBRARY: &str = r#"use std::fmt::Write;

macro_rules! call_through_the_module_path {
    ($function:ident) => {
        self::$function()
    };
}

macro_rules! call_inside_a_nested_function {
    ($body:block) => {{
        fn nested() -> u32 $body
        nested()
    }};
}

fn helper_total() -> u32 {
    7
}

pub fn entry_through_the_module_path() -> u32 {
    let helper_total = 1u32;
    call_through_the_module_path!(helper_total) + helper_total
}

pub fn entry_inside_a_nested_function() -> u32 {
    let helper_total = 1u32;
    helper_total + call_inside_a_nested_function!({ helper_total() })
}

pub fn entry_without_a_binding() -> u32 {
    helper_total()
}

pub fn local_name_in_format() -> String {
    let helper_total = 1u32;
    format!("{}", helper_total)
}

pub fn local_name_in_format_args() -> String {
    let helper_total = 1u32;
    let text = format_args!("{}", helper_total).to_string();
    text
}

pub fn local_name_in_print() -> u32 {
    let helper_total = 1u32;
    print!("{}", helper_total);
    0
}

pub fn local_name_in_println() -> u32 {
    let helper_total = 1u32;
    println!("{}", helper_total);
    0
}

pub fn local_name_in_eprint() -> u32 {
    let helper_total = 1u32;
    eprint!("{}", helper_total);
    0
}

pub fn local_name_in_eprintln() -> u32 {
    let helper_total = 1u32;
    eprintln!("{}", helper_total);
    0
}

pub fn local_name_in_write() -> String {
    let helper_total = 1u32;
    let mut text = String::new();
    write!(text, "{}", helper_total).expect("写得进串");
    text
}

pub fn local_name_in_writeln() -> String {
    let helper_total = 1u32;
    let mut text = String::new();
    writeln!(text, "{}", helper_total).expect("写得进串");
    text
}

pub fn local_name_in_panic() -> u32 {
    let helper_total = 1u32;
    panic!("{}", helper_total)
}

pub fn local_name_in_assert() -> u32 {
    let helper_total = 1u32;
    assert!(helper_total > 0);
    0
}

pub fn local_name_in_assert_eq() -> u32 {
    let helper_total = 1u32;
    assert_eq!(helper_total, 1);
    0
}

pub fn local_name_in_assert_ne() -> u32 {
    let helper_total = 1u32;
    assert_ne!(helper_total, 2);
    0
}

pub fn local_name_in_debug_assert() -> u32 {
    let helper_total = 1u32;
    debug_assert!(helper_total > 0);
    0
}

pub fn local_name_in_debug_assert_eq() -> u32 {
    let helper_total = 1u32;
    debug_assert_eq!(helper_total, 1);
    0
}

pub fn local_name_in_debug_assert_ne() -> u32 {
    let helper_total = 1u32;
    debug_assert_ne!(helper_total, 2);
    0
}

pub fn local_name_in_vec() -> Vec<u32> {
    let helper_total = 1u32;
    vec![helper_total]
}

pub fn local_name_in_dbg() -> u32 {
    let helper_total = 1u32;
    dbg!(helper_total)
}

pub fn let_inside_a_block_of_the_vector_macro() -> Vec<u32> {
    vec![{
        let helper_total = 7;
        helper_total
    }]
}

pub mod aliased {
    pub fn helper_total() -> u32 {
        9
    }
}

use crate::aliased::helper_total as aliased_total;

pub fn calls_through_an_alias_and_binds_the_original_name() -> u32 {
    let helper_total = 1;
    aliased_total() + helper_total
}

pub fn calls_the_function_in_a_range_before_the_let() -> u32 {
    let mut sum = 0;
    for step in sum..helper_total() {
        sum += step;
    }
    let helper_total = sum;
    helper_total
}
"#;

fn macro_tree(label: &str) -> FixtureTree {
    FixtureTree::materialize(
        label,
        &[
            (
                "crates/fshadow/Cargo.toml",
                "[package]\nname = \"fshadow\"\n",
            ),
            (MACRO_FILE, MACRO_LIBRARY),
            NO_DEPENDENCY_RESOLUTION,
        ],
    )
}

/// 场景：宏把实参接在 `self::` 后面（`self::$function()`），实参是外层的同名局部变量。预期：闭包有 `fn helper_total`。
#[test]
fn name_spliced_after_a_path_by_a_workspace_macro_links_the_function() {
    let tree = macro_tree("macro-spliced-after-a-path");
    let names = closure_unit_names(
        &tree,
        &library_entry("fshadow::entry_through_the_module_path"),
    );
    assert!(
        names.iter().any(|name| name == "fshadow::helper_total"),
        "宏展开成 self::helper_total()，运行时调的是函数：{names:?}"
    );
}

/// 场景：宏把实参块放进嵌套的 `fn nested()`，块里调同名函数。预期：闭包有 `fn helper_total`。
#[test]
fn name_in_a_block_moved_into_a_nested_function_by_a_workspace_macro_links_the_function() {
    let tree = macro_tree("macro-moved-into-a-nested-function");
    let names = closure_unit_names(
        &tree,
        &library_entry("fshadow::entry_inside_a_nested_function"),
    );
    assert!(
        names.iter().any(|name| name == "fshadow::helper_total"),
        "嵌套条目看不见外层局部变量，块里的 helper_total() 是函数：{names:?}"
    );
}

/// 场景（对照）：没有局部绑定，直接调同名函数。预期：闭包有 `fn helper_total`。
#[test]
fn call_without_a_binding_links_the_function_in_the_macro_fixture() {
    let tree = macro_tree("macro-control-without-a-binding");
    let names = closure_unit_names(&tree, &library_entry("fshadow::entry_without_a_binding"));
    assert!(
        names.iter().any(|name| name == "fshadow::helper_total"),
        "对照入口连不上就说明夹具本身连不上：{names:?}"
    );
}

/// 局部变量 `helper_total` 只写进名单里那一个标准库宏的括号里：闭包没有 `fn helper_total`（宏就地求值实参，括号里的是局部变量）。
fn assert_local_name_in_the_listed_standard_macro_shadows_the_function(
    macro_name: &str,
    entry_path: &'static str,
) {
    let tree = macro_tree(&format!("macro-listed-standard-macro-{macro_name}"));
    let names = closure_unit_names(&tree, &library_entry(entry_path));
    assert!(
        !names.iter().any(|name| name == "fshadow::helper_total"),
        "{macro_name}! 就地求值实参，括号里的 helper_total 是局部变量：{names:?}"
    );
}

/// 场景：局部变量只写进 `format!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_format_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "format",
        "fshadow::local_name_in_format",
    );
}

/// 场景：局部变量只写进 `format_args!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_the_format_arguments_macro_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "format_args",
        "fshadow::local_name_in_format_args",
    );
}

/// 场景：局部变量只写进 `print!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_print_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "print",
        "fshadow::local_name_in_print",
    );
}

/// 场景：局部变量只写进 `println!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_println_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "println",
        "fshadow::local_name_in_println",
    );
}

/// 场景：局部变量只写进 `eprint!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_eprint_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "eprint",
        "fshadow::local_name_in_eprint",
    );
}

/// 场景：局部变量只写进 `eprintln!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_eprintln_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "eprintln",
        "fshadow::local_name_in_eprintln",
    );
}

/// 场景：局部变量只写进 `write!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_write_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "write",
        "fshadow::local_name_in_write",
    );
}

/// 场景：局部变量只写进 `writeln!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_writeln_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "writeln",
        "fshadow::local_name_in_writeln",
    );
}

/// 场景：局部变量只写进 `panic!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_panic_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "panic",
        "fshadow::local_name_in_panic",
    );
}

/// 场景：局部变量只写进 `assert!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_assert_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "assert",
        "fshadow::local_name_in_assert",
    );
}

/// 场景：局部变量只写进 `assert_eq!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_the_assert_equal_macro_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "assert_eq",
        "fshadow::local_name_in_assert_eq",
    );
}

/// 场景：局部变量只写进 `assert_ne!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_assert_ne_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "assert_ne",
        "fshadow::local_name_in_assert_ne",
    );
}

/// 场景：局部变量只写进 `debug_assert!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_debug_assert_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "debug_assert",
        "fshadow::local_name_in_debug_assert",
    );
}

/// 场景：局部变量只写进 `debug_assert_eq!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_the_debug_assert_equal_macro_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "debug_assert_eq",
        "fshadow::local_name_in_debug_assert_eq",
    );
}

/// 场景：局部变量只写进 `debug_assert_ne!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_debug_assert_ne_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "debug_assert_ne",
        "fshadow::local_name_in_debug_assert_ne",
    );
}

/// 场景：局部变量只写进 `vec!` 的括号里。预期：闭包没有 `fn helper_total`。
#[test]
fn local_name_in_vec_shadows_the_function() {
    assert_local_name_in_the_listed_standard_macro_shadows_the_function(
        "vec",
        "fshadow::local_name_in_vec",
    );
}

/// 场景：局部变量只写进 `dbg!` 的括号里（标准库宏，不在名单上）。预期：闭包有 `fn helper_total`（名单外的宏照连）。
#[test]
fn local_name_in_a_standard_macro_outside_the_list_links_the_function() {
    let tree = macro_tree("macro-standard-macro-outside-the-list");
    let names = closure_unit_names(&tree, &library_entry("fshadow::local_name_in_dbg"));
    assert!(
        names.iter().any(|name| name == "fshadow::helper_total"),
        "dbg! 不在名单上，括号里的 helper_total 照连同名函数：{names:?}"
    );
}

/// 场景：`vec![{ let helper_total = 7; helper_total }]`，`let` 写在标准库宏括号里的块中。预期：闭包有 `fn helper_total`（宏调用括号里的 `let` 不当绑定）。
#[test]
fn let_inside_a_block_of_a_listed_standard_macro_does_not_shadow() {
    let tree = macro_tree("macro-let-inside-a-block-of-vec");
    let names = closure_unit_names(
        &tree,
        &library_entry("fshadow::let_inside_a_block_of_the_vector_macro"),
    );
    assert!(
        names.iter().any(|name| name == "fshadow::helper_total"),
        "宏调用括号里（套在块里也算）的 let 不当绑定：{names:?}"
    );
}

/// 场景：`use crate::aliased::helper_total as aliased_total;`，函数调 `aliased_total()`、又有局部变量 `helper_total`。预期：闭包有 `fn aliased::helper_total`。
#[test]
fn alias_original_bound_as_a_local_name_still_links_the_aliased_function() {
    let tree = macro_tree("macro-alias-original-bound-locally");
    let names = closure_unit_names(
        &tree,
        &library_entry("fshadow::calls_through_an_alias_and_binds_the_original_name"),
    );
    assert!(
        names
            .iter()
            .any(|name| name == "fshadow::aliased::helper_total"),
        "改名的原名不是这个单元写的名字，局部绑定遮不住它：{names:?}"
    );
}

/// 场景：`let` 之前在区间 `sum..helper_total()` 里调同名函数。预期：闭包有 `fn helper_total`（`..` 后面的不是字段）。
#[test]
fn call_in_a_range_before_the_let_links_the_function() {
    let tree = macro_tree("macro-range-before-the-let");
    let names = closure_unit_names(
        &tree,
        &library_entry("fshadow::calls_the_function_in_a_range_before_the_let"),
    );
    assert!(
        names.iter().any(|name| name == "fshadow::helper_total"),
        "区间的 `..` 不是单个点，后面的 helper_total() 指函数：{names:?}"
    );
}

/// `backend` 里的 `checksum` 只在 `fast` 下编，不开时经 `pub use` glob 转 `portable` 的。
/// 转的那一条写成 glob 而不按名字引进 `checksum`：按名字引进的会让「候选所在模块另有 use 引进同名」那一支也成立，
/// 盖住「候选自己带条件编译」这一支，只靠条件编译退回全连的那一形就测不出来。
const CONDITIONAL_CANDIDATE_LIBRARY: &str = r#"pub mod backend {
    #[cfg(feature = "fast")]
    pub fn checksum(data: &[u8]) -> u32 {
        data.len() as u32
    }

    #[cfg(not(feature = "fast"))]
    pub use crate::portable::*;
}

pub mod portable {
    pub fn checksum(data: &[u8]) -> u32 {
        data.iter().map(|byte| u32::from(*byte)).sum()
    }
}

use crate::backend::checksum;

pub fn entry_point(data: &[u8]) -> u32 {
    checksum(data)
}
"#;

/// `backend` 里的候选是条目级宏调用 `fmacros::make_checksum!()`：它展开成不带条件的花括号结构体 `checksum`（只占类型命名空间），
/// 按宏体顶层按字面写出的条目名登在 `checksum` 名下；同一个模块另有 `pub use crate::portable::checksum`（值命名空间的函数）。
/// 候选不带条件、所在文件也不经带条件的 `mod` 连进来，退回全连只能靠「候选所在模块另有 use 引进同名」那一支。
/// 候选的词法单元里不写 `checksum`：自己写着这个名字的候选（结构体、函数、括号里点名它的宏调用）连进闭包之后，在 `backend` 里按名字解它，
/// 经那条 `pub use` 照样连到 `portable::checksum`，盖住这一支。宏定义在另一个包 `fmacros` 里：宏定义单元的宏体裸名在调用处解、一律全连，
/// 定义在 `fswap` 里会把 `portable::checksum` 从那一边连进闭包；定义在 `fmacros` 里只在 `fmacros` 看得见的目标里找，碰不到 `fswap`。
const RE_IMPORTING_MODULE_LIBRARY: &str = r#"pub mod backend {
    fmacros::make_checksum!();

    pub use crate::portable::checksum;
}

pub mod portable {
    pub fn checksum(data: &[u8]) -> u32 {
        data.iter().map(|byte| u32::from(*byte)).sum()
    }
}

use crate::backend::checksum;

pub fn entry_point(data: &[u8]) -> u32 {
    checksum(data)
}
"#;

/// `fmacros` 的 `make_checksum!` 展开成花括号结构体 `checksum`（宏体顶层按字面写出的条目名）。
const CHECKSUM_STRUCT_MACRO_LIBRARY: &str = r#"#[macro_export]
macro_rules! make_checksum {
    () => {
        pub struct checksum {
            pub total: u32,
        }
    };
}
"#;

/// `backend` 有两份：开 `fast` 时是 `backend.rs` 那个文件，不开时是行内模块、经 `pub use` 转 `portable` 的。
const CONDITIONAL_FILE_MODULE_LIBRARY: &str = r#"#[cfg(feature = "fast")]
mod backend;

#[cfg(not(feature = "fast"))]
pub mod backend {
    pub use crate::portable::checksum;
}

pub mod portable {
    pub fn checksum(data: &[u8]) -> u32 {
        data.iter().map(|byte| u32::from(*byte)).sum()
    }
}

use crate::backend::checksum;

pub fn entry_point(data: &[u8]) -> u32 {
    checksum(data)
}
"#;

const CONDITIONAL_FILE_MODULE_BACKEND: &str =
    "pub fn checksum(data: &[u8]) -> u32 {\n    data.len() as u32\n}\n";

fn swap_tree(label: &str, library: &str, extra_files: &[(&str, &str)]) -> FixtureTree {
    let mut files = vec![
        (
            "crates/fswap/Cargo.toml",
            "[package]\nname = \"fswap\"\n\n[features]\nfast = []\n",
        ),
        ("crates/fswap/src/lib.rs", library),
        NO_DEPENDENCY_RESOLUTION,
    ];
    files.extend_from_slice(extra_files);
    FixtureTree::materialize(label, &files)
}

/// 场景：use 点到的 `backend::checksum` 带 `#[cfg(feature = "fast")]`，同一个模块不开 `fast` 时经 glob 转 `portable`，不开 `fast`。预期：闭包有 `fn portable::checksum`。
#[test]
fn use_pointing_at_a_conditionally_compiled_candidate_links_the_re_exported_one_too() {
    let tree = swap_tree(
        "use-conditional-candidate",
        CONDITIONAL_CANDIDATE_LIBRARY,
        &[],
    );
    let names = closure_unit_names(&tree, &library_entry("fswap::entry_point"));
    assert!(
        names.iter().any(|name| name == "fswap::portable::checksum"),
        "候选带 cfg，名字可能是 cfg 互斥的 pub use 转来的：{names:?}"
    );
}

/// 场景：use 点到的 `backend` 里候选是 `fmacros::make_checksum!()` 展开的花括号结构体 `checksum`，同一个模块另有 `pub use crate::portable::checksum`。预期：闭包有 `fn portable::checksum`。
#[test]
fn use_pointing_at_a_module_that_also_imports_the_name_links_the_imported_one_too() {
    let tree = FixtureTree::materialize(
        "use-module-re-importing",
        &[
            (
                "crates/fswap/Cargo.toml",
                "[package]\nname = \"fswap\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nfmacros = { path = \"../fmacros\" }\n",
            ),
            ("crates/fswap/src/lib.rs", RE_IMPORTING_MODULE_LIBRARY),
            (
                "crates/fmacros/Cargo.toml",
                "[package]\nname = \"fmacros\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            ("crates/fmacros/src/lib.rs", CHECKSUM_STRUCT_MACRO_LIBRARY),
            NO_DEPENDENCY_RESOLUTION,
        ],
    );
    let names = closure_unit_names(&tree, &library_entry("fswap::entry_point"));
    assert!(
        names.iter().any(|name| name == "fswap::portable::checksum"),
        "候选所在模块另有 use 引进同名的，落在哪一份看不出：{names:?}"
    );
}

/// 场景：use 点到的 `backend::checksum` 在 `#[cfg(feature = "fast")] mod backend;` 连进来的文件里，不开 `fast`。预期：闭包有 `fn portable::checksum`。
#[test]
fn use_pointing_at_a_candidate_in_a_conditionally_declared_file_links_the_re_exported_one_too() {
    let tree = swap_tree(
        "use-conditional-file-module",
        CONDITIONAL_FILE_MODULE_LIBRARY,
        &[(
            "crates/fswap/src/backend.rs",
            CONDITIONAL_FILE_MODULE_BACKEND,
        )],
    );
    let names = closure_unit_names(&tree, &library_entry("fswap::entry_point"));
    assert!(
        names.iter().any(|name| name == "fswap::portable::checksum"),
        "候选所在文件经带 cfg 的 mod 连进来，同样算带条件：{names:?}"
    );
}

/// `use crate::deep::real as short; use short::helper_total;`，别处另有 `other::short`。
const MODULE_ALIAS_LIBRARY: &str = r#"pub mod deep {
    pub mod real {
        pub fn helper_total() -> u32 {
            7
        }
    }
}

pub mod other {
    pub mod short {
        pub fn helper_total() -> u32 {
            8
        }
    }
}

use crate::deep::real as short;
use short::helper_total;

pub fn entry_point() -> u32 {
    helper_total()
}
"#;

/// 场景：相对路径 `short::helper_total` 的首段是 `use … as short` 改出来的名字。预期：闭包有 `fn deep::real::helper_total`。
#[test]
fn relative_use_path_starting_with_a_module_alias_links_the_aliased_module() {
    let tree = FixtureTree::materialize(
        "use-relative-path-through-module-alias",
        &[
            ("crates/falias/Cargo.toml", "[package]\nname = \"falias\"\n"),
            ("crates/falias/src/lib.rs", MODULE_ALIAS_LIBRARY),
            NO_DEPENDENCY_RESOLUTION,
        ],
    );
    let names = closure_unit_names(&tree, &library_entry("falias::entry_point"));
    assert!(
        names
            .iter()
            .any(|name| name == "falias::deep::real::helper_total"),
        "首段 short 是改名的 deep::real，真去处看不出就退回全连：{names:?}"
    );
}

/// `use crate::first::old_total as helper_total;`，`first`、`second` 各有一个 `helper_total`。
const ALIAS_LEAF_LIBRARY: &str = r#"pub mod first {
    pub fn old_total() -> u32 {
        1
    }

    pub fn helper_total() -> u32 {
        2
    }
}

pub mod second {
    pub fn helper_total() -> u32 {
        3
    }
}

use crate::first::old_total as helper_total;

pub fn entry_point() -> u32 {
    helper_total()
}
"#;

/// 场景：`原名 as 名` 那一支的叶子是别名。预期：两份 `helper_total` 都在闭包里（退回全连），改名的原名 `old_total` 也在。
#[test]
fn alias_leaf_of_a_use_falls_back_to_every_same_named_function() {
    let tree = FixtureTree::materialize(
        "use-alias-leaf",
        &[
            (
                "crates/fdiverge/Cargo.toml",
                "[package]\nname = \"fdiverge\"\n",
            ),
            ("crates/fdiverge/src/lib.rs", ALIAS_LEAF_LIBRARY),
            NO_DEPENDENCY_RESOLUTION,
        ],
    );
    let names = closure_unit_names(&tree, &library_entry("fdiverge::entry_point"));
    for expected in [
        "fdiverge::first::helper_total",
        "fdiverge::second::helper_total",
        "fdiverge::first::old_total",
    ] {
        assert!(
            names.iter().any(|name| name == expected),
            "{expected} 要在闭包里：{names:?}"
        );
    }
}

/// 一条 use 里两支引进同一个名字（glob 经 `prelude` 转出来的一支、指名 `layout` 的一支），再加 `r#` 写的名字引进的一条 use。
const MIXED_BRANCHES_LIBRARY: &str = r#"pub mod items {
    pub const MARK: u32 = 1;
}

pub mod prelude {
    pub use crate::items::*;
}

pub mod layout {
    pub const MARK: u32 = 2;
}

pub mod consumer_of_two_branches {
    use crate::{layout::MARK, prelude::*};
    pub fn read_mark() -> u32 {
        MARK
    }
}

pub mod counting {
    pub fn r#total() -> u32 {
        1
    }
}

pub mod spare_counting {
    pub fn r#total() -> u32 {
        2
    }
}

pub mod consumer_of_a_raw_name {
    use crate::counting::r#total;
    pub fn read_total() -> u32 {
        r#total()
    }
}
"#;

fn mixed_branches_tree(label: &str) -> FixtureTree {
    FixtureTree::materialize(
        label,
        &[
            ("crates/fmixed/Cargo.toml", "[package]\nname = \"fmixed\"\n"),
            ("crates/fmixed/src/lib.rs", MIXED_BRANCHES_LIBRARY),
            NO_DEPENDENCY_RESOLUTION,
        ],
    )
}

/// 场景：`use crate::{layout::MARK, prelude::*};`，glob 那一支点到的 `prelude` 里没有 `MARK` 的候选。预期：整条 use 退回全连，闭包有 `items::MARK`。
#[test]
fn use_whose_glob_branch_points_nowhere_falls_back_though_another_branch_points_at_a_candidate() {
    let tree = mixed_branches_tree("use-two-branches");
    let names = closure_unit_names(
        &tree,
        &library_entry("fmixed::consumer_of_two_branches::read_mark"),
    );
    assert!(
        names.iter().any(|name| name == "fmixed::items::MARK"),
        "每一支都点到才算点准，glob 那一支点不到就整条退回全连：{names:?}"
    );
}

/// 场景：`use crate::counting::r#total;`，use 里的叶子写成 `r#total`、与名字 `total` 字面对不上。预期：退回全连，闭包有 `counting::total`
/// 与 `spare_counting::total` 两份（只有前一份是点准，一份都没有是少收）。
#[test]
fn use_whose_leaf_is_written_as_a_raw_identifier_falls_back_to_every_candidate() {
    let tree = mixed_branches_tree("use-raw-identifier-leaf");
    let names = closure_unit_names(
        &tree,
        &library_entry("fmixed::consumer_of_a_raw_name::read_total"),
    );
    for expected in ["fmixed::counting::total", "fmixed::spare_counting::total"] {
        assert!(
            names.iter().any(|name| name == expected),
            "引进名单里有、use 里找不到那一支，退回全连，{expected} 要在闭包里：{names:?}"
        );
    }
}

const RESTART_MANIFEST: (&str, &str) = (
    "crates/frestart/Cargo.toml",
    "[package]\nname = \"frestart\"\n",
);
const RESTART_MAIN_FILE: &str = "crates/frestart/src/main.rs";

const RESTART_MAIN_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "frestart::main",
    target: "bin:frestart",
    conditions: &[],
};

/// 块里那个 `exe` 是 `current_exe()`，块外交给 `Command::new` 的是 `worker_path()`。
const SHADOWED_BINDING_MAIN: &str = r#"fn worker_path() -> std::path::PathBuf {
    std::env::current_exe().unwrap().with_file_name("worker")
}

fn main() {
    let exe = worker_path();
    {
        let exe = std::env::current_exe().unwrap();
        eprintln!("launcher at {}", exe.display());
    }
    std::process::Command::new(&exe).status().unwrap();
}
"#;

const WORKER_MAIN: &str = "fn worker_role() -> u32 {\n    3\n}\n\nfn main() {\n    std::process::exit(worker_role() as i32);\n}\n";

/// `unwrap_or_else` 的闭包以 `if … { 路径 } else { panic!() }` 收尾。
const ARGUMENT_FALLBACK_MAIN: &str = r#"fn main() {
    let this_program = std::env::current_exe().unwrap_or_else(|_| {
        if let Some(first_argument) = std::env::args_os().next() {
            std::path::PathBuf::from(first_argument)
        } else {
            panic!("拿不到程序路径")
        }
    });
    if std::env::args().count() > 1 {
        return;
    }
    std::process::Command::new(&this_program)
        .arg("role")
        .status()
        .unwrap();
}
"#;

/// `match` 的 `Err` 分支是 `=>` 后面以 `process::exit` 收尾的块。
const MATCH_ARM_BLOCK_MAIN: &str = r#"fn role_body() -> u32 {
    1
}

fn main() {
    if std::env::args().count() > 1 {
        role_body();
        return;
    }
    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) => {
            eprintln!("取不到路径：{error}");
            std::process::exit(78)
        }
    };
    std::process::Command::new(&executable)
        .arg("role")
        .status()
        .expect("起得来");
}
"#;

/// `let` 写在 `main` 的函数体里，`Command::new(&exe)` 写在更深一层的 `if` 块里。
const USE_IN_A_DEEPER_BLOCK_MAIN: &str = r#"fn main() {
    let exe = std::env::current_exe().expect("路径");
    if std::env::args().count() == 1 {
        std::process::Command::new(&exe).arg("role").status().expect("起得来");
    }
}
"#;

/// 路径只拿来显示，起的是别的程序。
const PATH_ONLY_DISPLAYED_MAIN: &str = r#"fn main() {
    let exe = std::env::current_exe().expect("路径");
    eprintln!("from {}", exe.display());
    std::process::Command::new("true").status().expect("起得来");
}
"#;

fn restart_tree(label: &str, main_source: &str, extra_files: &[(&str, &str)]) -> FixtureTree {
    let mut files = vec![
        RESTART_MANIFEST,
        (RESTART_MAIN_FILE, main_source),
        NO_DEPENDENCY_RESOLUTION,
    ];
    files.extend_from_slice(extra_files);
    FixtureTree::materialize(label, &files)
}

fn assert_refused_as_current_exe_in_the_main_file(verdict: Result<String, String>) {
    let reason = verdict.expect_err("起的可能是别的程序，拒算");
    assert!(
        reason.contains("refused\tcurrent-exe\tcrates/frestart/src/main.rs:"),
        "拒算行点名 current-exe 与 main.rs：{reason}"
    );
}

/// 场景：`current_exe()` 绑在内层块的 `exe` 上，块外交给 `Command::new(&exe)` 的是外层另一个 `exe`。预期：拒算 `current-exe`。
#[test]
fn binding_inside_a_block_with_the_use_after_the_block_is_refused_as_current_exe() {
    let tree = restart_tree(
        "restart-shadowed-binding",
        SHADOWED_BINDING_MAIN,
        &[("crates/frestart/src/bin/worker.rs", WORKER_MAIN)],
    );
    assert_refused_as_current_exe_in_the_main_file(key_verdict(&tree, &RESTART_MAIN_ENTRY));
}

/// 场景：`let exe = current_exe()…;` 在函数体那一层，`Command::new(&exe)` 在更深一层的 `if` 块里（用法与 `let` 不同层）。预期：拒算 `current-exe`。
#[test]
fn use_in_a_deeper_block_than_the_let_is_refused_as_current_exe() {
    let tree = restart_tree(
        "restart-use-in-a-deeper-block",
        USE_IN_A_DEEPER_BLOCK_MAIN,
        &[],
    );
    assert_refused_as_current_exe_in_the_main_file(key_verdict(&tree, &RESTART_MAIN_ENTRY));
}

/// 场景：失败时的闭包以 `if … else { panic!() }` 收尾，`if` 那一支交回 `argv[0]`。预期：拒算 `current-exe`。
#[test]
fn fallback_closure_ending_in_an_if_else_is_refused_as_current_exe() {
    let tree = restart_tree("restart-argument-fallback", ARGUMENT_FALLBACK_MAIN, &[]);
    assert_refused_as_current_exe_in_the_main_file(key_verdict(&tree, &RESTART_MAIN_ENTRY));
}

/// 场景：`match current_exe()` 的 `Err` 分支是 `=> { …; process::exit(78) }`。预期：从 bin 的 `main` 算得出键，改角色函数键变。
#[test]
fn match_arm_block_ending_in_process_exit_is_keyed_from_the_main_of_the_bin() {
    let tree = restart_tree("restart-match-arm-block", MATCH_ARM_BLOCK_MAIN, &[]);
    let kept = key_verdict(&tree, &RESTART_MAIN_ENTRY)
        .unwrap_or_else(|reason| panic!("`=>` 后面一定不交值的块，起的就是这个程序：{reason}"));
    tree.replace_exactly_once(
        RESTART_MAIN_FILE,
        "fn role_body() -> u32 {\n    1",
        "fn role_body() -> u32 {\n    2",
    );
    let after_the_edit =
        key_verdict(&tree, &RESTART_MAIN_ENTRY).expect("改了角色的函数体照样算得出键");
    assert_ne!(kept, after_the_edit, "子进程那个角色跑的函数改了，键要变");
}

/// 场景：`current_exe()` 的路径只拿来显示，同一个单元里 `Command::new("true")` 起别的程序。预期：拒算 `current-exe`。
#[test]
fn path_only_displayed_while_another_program_starts_is_refused_as_current_exe() {
    let tree = restart_tree("restart-path-only-displayed", PATH_ONLY_DISPLAYED_MAIN, &[]);
    assert_refused_as_current_exe_in_the_main_file(key_verdict(&tree, &RESTART_MAIN_ENTRY));
}

const EXTERN_LIBRARY_FILE: &str = "crates/fextern/src/lib.rs";

fn extern_tree(label: &str, library: &str, extra_files: &[(&str, &str)]) -> FixtureTree {
    let mut files = vec![
        (
            "crates/fextern/Cargo.toml",
            "[package]\nname = \"fextern\"\n",
        ),
        (EXTERN_LIBRARY_FILE, library),
        NO_DEPENDENCY_RESOLUTION,
    ];
    files.extend_from_slice(extra_files);
    FixtureTree::materialize(label, &files)
}

fn assert_refused_as_extern_function_interface(verdict: Result<String, String>) {
    let reason = verdict.expect_err("链接时可能多出别的库，拒算");
    assert!(
        reason.contains("refused\textern-function-interface\tcrates/fextern/src/lib.rs:"),
        "拒算行点名 extern-function-interface 与夹具文件：{reason}"
    );
}

const EXTERN_ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fextern::entry_point",
    target: "lib",
    conditions: &[],
};

/// 场景：块前面写 `#[cfg_attr(target_os = "linux", link(name = "fakesync"))]`。预期：拒算。
#[test]
fn link_inside_a_conditional_configuration_attribute_on_the_block_is_refused() {
    let tree = extern_tree(
        "extern-link-through-cfg-attr",
        "#[cfg_attr(target_os = \"linux\", link(name = \"fakesync\"))]\nextern \"C\" {\n    fn fdatasync(fd: i32) -> i32;\n}\n\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n",
        &[],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// 场景：同一个目标里另一个空块带 `#[link(name = "fakesync")]`，没人点名它。预期：名单里的块照样拒算。
#[test]
fn link_on_another_extern_block_of_the_target_refuses_the_listed_block() {
    let tree = extern_tree(
        "extern-link-on-another-block",
        "#[link(name = \"fakesync\")]\nextern \"C\" {}\n\nextern \"C\" {\n    fn fdatasync(fd: i32) -> i32;\n}\n\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n",
        &[],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// 场景：包的构建脚本打 `cargo:rustc-link-lib=fakesync`，库里的块只声明名单里的符号。预期：拒算。
#[test]
fn build_script_linking_a_library_refuses_the_listed_block() {
    let tree = extern_tree(
        "extern-build-script-link",
        "extern \"C\" {\n    fn fdatasync(fd: i32) -> i32;\n}\n\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n",
        &[(
            "crates/fextern/build.rs",
            "fn main() {\n    println!(\"cargo:rustc-link-lib=fakesync\");\n}\n",
        )],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// 场景（对照）：只声明 `pwrite`、`fdatasync`，哪里都没有 `link`，构建脚本不加库。预期：算得出键。
#[test]
fn listed_calls_without_any_link_are_still_keyed() {
    let tree = extern_tree(
        "extern-listed-without-link",
        "extern \"C\" {\n    fn pwrite(fd: i32, buf: *const u8, count: usize, offset: i64) -> isize;\n    fn fdatasync(fd: i32) -> i32;\n}\n\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n",
        &[(
            "crates/fextern/build.rs",
            "fn main() {\n    println!(\"cargo:rerun-if-changed=build.rs\");\n}\n",
        )],
    );
    key_verdict(&tree, &EXTERN_ENTRY)
        .unwrap_or_else(|reason| panic!("名单里的 C 库调用、没有 link，不拒算：{reason}"));
}

/// 场景：声明前面的属性是 `#[doc = "fdatasync"]`（`link_name` 之外的属性）。预期：拒算。
#[test]
fn another_attribute_on_a_declaration_is_refused() {
    let tree = extern_tree(
        "extern-another-attribute",
        "extern \"C\" {\n    #[doc = \"fdatasync\"]\n    fn fdatasync_raw(fd: i32) -> i32;\n}\n\npub fn entry_point() -> i32 {\n    unsafe { fdatasync_raw(3) }\n}\n",
        &[],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// `deep::real` 与 `other::short` 各有一个 `helper_total`；四个模块各用一种改名再走相对路径 `short::helper_total`：同一条 use 里先改名、
/// 条目级宏的宏体展开出改名、条目级宏把改出的名字当实参交给宏体、条目级宏再调别的宏展开出改名（调用与宏体里都不写 `short`）；
/// 另一个模块经 `use crate::deep::real::{self}` 引进 `real` 再走相对路径 `real::helper_total`。
const SAME_USE_ALIAS_LIBRARY: &str = r#"pub mod deep {
    pub mod real {
        pub fn helper_total() -> u32 {
            7
        }
    }
}

pub mod other {
    pub mod short {
        pub fn helper_total() -> u32 {
            8
        }
    }
}

macro_rules! bring_in_the_real_module_as_short {
    () => {
        use crate::deep::real as short;
    };
}

macro_rules! bring_in_the_real_module_as {
    ($alias:ident) => {
        use crate::deep::real as $alias;
    };
}

pub mod through_one_use {
    use {crate::deep::real as short, short::helper_total};

    pub fn entry_point() -> u32 {
        helper_total()
    }
}

pub mod through_a_macro_body {
    bring_in_the_real_module_as_short!();
    use short::helper_total;

    pub fn entry_point() -> u32 {
        helper_total()
    }
}

pub mod through_a_macro_argument {
    bring_in_the_real_module_as!(short);
    use short::helper_total;

    pub fn entry_point() -> u32 {
        helper_total()
    }
}

macro_rules! bring_in_the_real_module_through_another_macro {
    () => {
        bring_in_the_real_module_as_short!();
    };
}

pub mod through_a_nested_macro {
    bring_in_the_real_module_through_another_macro!();
    use short::helper_total;

    pub fn entry_point() -> u32 {
        helper_total()
    }
}

pub mod through_a_self_import {
    use crate::deep::real::{self};
    use real::helper_total;

    pub fn entry_point() -> u32 {
        helper_total()
    }
}
"#;

fn same_use_alias_tree(label: &str) -> FixtureTree {
    FixtureTree::materialize(
        label,
        &[
            (
                "crates/fsameuse/Cargo.toml",
                "[package]\nname = \"fsameuse\"\n",
            ),
            ("crates/fsameuse/src/lib.rs", SAME_USE_ALIAS_LIBRARY),
            NO_DEPENDENCY_RESOLUTION,
        ],
    )
}

/// 入口闭包里有 `deep::real::helper_total`（首段 `short` 指谁看不出，退回全连）。
fn assert_the_aliased_module_is_linked(entry_path: &'static str, label: &str) {
    let tree = same_use_alias_tree(label);
    let names = closure_unit_names(&tree, &library_entry(entry_path));
    assert!(
        names
            .iter()
            .any(|name| name == "fsameuse::deep::real::helper_total"),
        "{entry_path}：首段 short 是改名的 deep::real，真去处看不出就退回全连：{names:?}"
    );
}

/// 场景：`use {crate::deep::real as short, short::helper_total};`，改名与相对路径写在同一条 use 里。预期：闭包有 `fn deep::real::helper_total`。
#[test]
fn relative_use_path_through_an_alias_made_in_the_same_use_links_the_aliased_module() {
    assert_the_aliased_module_is_linked(
        "fsameuse::through_one_use::entry_point",
        "use-alias-in-the-same-use",
    );
}

/// 场景：条目级宏调用 `bring_in_the_real_module_as_short!();` 的宏体是 `use crate::deep::real as short;`，下一条 `use short::helper_total;`。
/// 预期：闭包有 `fn deep::real::helper_total`。
#[test]
fn relative_use_path_through_an_alias_written_in_an_item_macro_body_links_the_aliased_module() {
    assert_the_aliased_module_is_linked(
        "fsameuse::through_a_macro_body::entry_point",
        "use-alias-in-a-macro-body",
    );
}

/// 场景：条目级宏调用 `bring_in_the_real_module_as!(short);` 把 `short` 当实参交给宏体 `use crate::deep::real as $alias;`。
/// 预期：闭包有 `fn deep::real::helper_total`。
#[test]
fn relative_use_path_through_an_alias_passed_to_an_item_macro_links_the_aliased_module() {
    assert_the_aliased_module_is_linked(
        "fsameuse::through_a_macro_argument::entry_point",
        "use-alias-as-a-macro-argument",
    );
}

/// 场景：条目级宏调用 `bring_in_the_real_module_through_another_macro!();` 的宏体再调 `bring_in_the_real_module_as_short!();`，
/// 调用与它的宏体里都没有 `short` 这个标识符，下一条 `use short::helper_total;`。预期：本模块有条目级宏调用就退回全连，
/// 闭包有 `fn deep::real::helper_total`。
#[test]
fn relative_use_path_in_a_module_with_an_item_macro_invocation_falls_back_though_the_macro_never_names_the_first_segment(
) {
    assert_the_aliased_module_is_linked(
        "fsameuse::through_a_nested_macro::entry_point",
        "use-alias-from-a-nested-macro",
    );
}

/// 场景：`use crate::deep::real::{self};` 引进 `real`，下一条 `use real::helper_total;`。预期：首段是 `{self}` 引进的名字，退回全连，
/// 闭包里 `deep::real::helper_total` 与后缀对不上的 `other::short::helper_total` 都在。
#[test]
fn relative_use_path_whose_first_segment_comes_from_a_self_import_falls_back_to_every_candidate() {
    let tree = same_use_alias_tree("use-first-segment-from-a-self-import");
    let names = closure_unit_names(
        &tree,
        &library_entry("fsameuse::through_a_self_import::entry_point"),
    );
    for expected in [
        "fsameuse::deep::real::helper_total",
        "fsameuse::other::short::helper_total",
    ] {
        assert!(
            names.iter().any(|name| name == expected),
            "首段 real 是 use … ::{{self}} 引进的，退回全连，{expected} 要在闭包里：{names:?}"
        );
    }
}

/// 只声明 `fdatasync` 的名单块与调它的入口。
const LISTED_BLOCK_LIBRARY: &str = "extern \"C\" {\n    fn fdatasync(fd: i32) -> i32;\n}\n\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n";

/// 只有一个带 `#[link(name = "fakesync")]` 的空块的库。
const LINKING_LIBRARY: &str =
    "#[link(name = \"fakesync\")]\nextern \"C\" {}\n\npub fn linked() {}\n";

/// 构建脚本打 `cargo:rustc-link-lib=fakesync` 的函数。
const LINK_PRINTING_HELPER_LIBRARY: &str =
    "pub fn link_the_fake_library() {\n    println!(\"cargo:rustc-link-lib=fakesync\");\n}\n";

/// 场景：库里是名单块，同一个包的集成测试 `tests/linked.rs` 里有带 `#[link]` 的空块（测试二进制把两者链在一起）。预期：拒算。
#[test]
fn link_on_an_extern_block_of_another_target_of_the_package_refuses_the_listed_block() {
    let tree = extern_tree(
        "extern-link-in-another-target",
        LISTED_BLOCK_LIBRARY,
        &[(
            "crates/fextern/tests/linked.rs",
            "#[link(name = \"fakesync\")]\nextern \"C\" {}\n\n#[test]\nfn fdatasync_through_the_library() {\n    assert_eq!(fextern::entry_point(), 4242);\n}\n",
        )],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// `fextern` 的清单，`[dependencies]` 与别的节照 `sections` 接在 `[package]` 后面。
fn extern_manifest_with(sections: &'static str) -> (&'static str, String) {
    (
        "crates/fextern/Cargo.toml",
        format!("[package]\nname = \"fextern\"\n\n{sections}"),
    )
}

/// 名单块所在的 `fextern` 清单另写几节、再加几份文件的夹具树。
fn extern_tree_with_manifest_sections(
    label: &str,
    sections: &'static str,
    extra_files: &[(&str, &str)],
) -> FixtureTree {
    let (manifest_path, manifest_text) = extern_manifest_with(sections);
    let mut files = vec![
        (manifest_path, manifest_text.as_str()),
        (EXTERN_LIBRARY_FILE, LISTED_BLOCK_LIBRARY),
        NO_DEPENDENCY_RESOLUTION,
    ];
    files.extend_from_slice(extra_files);
    FixtureTree::materialize(label, &files)
}

/// 场景：`fextern` 依赖 path 包 `fdeplink`，`fdeplink` 里有带 `#[link]` 的空块。预期：拒算。
#[test]
fn link_in_a_dependency_package_refuses_the_listed_block() {
    let tree = extern_tree_with_manifest_sections(
        "extern-link-in-a-dependency",
        "[dependencies]\nfdeplink = { path = \"../fdeplink\" }\n",
        &[
            (
                "crates/fdeplink/Cargo.toml",
                "[package]\nname = \"fdeplink\"\n",
            ),
            ("crates/fdeplink/src/lib.rs", LINKING_LIBRARY),
        ],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// 场景：`fextern` 的构建脚本只调构建依赖 `fbuildhelper` 的函数，打 `cargo:rustc-link-lib=fakesync` 的是那个函数。预期：拒算。
#[test]
fn build_dependency_printing_a_link_instruction_refuses_the_listed_block() {
    let tree = extern_tree_with_manifest_sections(
        "extern-link-through-a-build-dependency",
        "[build-dependencies]\nfbuildhelper = { path = \"../fbuildhelper\" }\n",
        &[
            (
                "crates/fextern/build.rs",
                "fn main() {\n    fbuildhelper::link_the_fake_library();\n}\n",
            ),
            (
                "crates/fbuildhelper/Cargo.toml",
                "[package]\nname = \"fbuildhelper\"\n",
            ),
            (
                "crates/fbuildhelper/src/lib.rs",
                LINK_PRINTING_HELPER_LIBRARY,
            ),
        ],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// 场景：`fextern` 的构建脚本用工作区外的构建依赖 `cc` 编一份 C 代码（加库的指令写在 `cc` 里，看不到）。预期：拒算。
#[test]
fn build_dependency_outside_the_workspace_refuses_the_listed_block() {
    let tree = extern_tree_with_manifest_sections(
        "extern-link-through-an-outside-build-dependency",
        "[build-dependencies]\ncc = \"1\"\n",
        &[(
            "crates/fextern/build.rs",
            "fn main() {\n    cc::Build::new().file(\"src/shim.c\").compile(\"shim\");\n}\n",
        )],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// 场景：包的构建脚本打 `cargo:rustc-link-arg=-lfakesync`。预期：拒算。
#[test]
fn build_script_passing_a_link_argument_refuses_the_listed_block() {
    let tree = extern_tree(
        "extern-build-script-link-argument",
        LISTED_BLOCK_LIBRARY,
        &[(
            "crates/fextern/build.rs",
            "fn main() {\n    println!(\"cargo:rustc-link-arg=-lfakesync\");\n}\n",
        )],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// 场景：包的构建脚本打 `cargo:rustc-flags=-l fakesync`。预期：拒算。
#[test]
fn build_script_passing_rustc_flags_refuses_the_listed_block() {
    let tree = extern_tree(
        "extern-build-script-rustc-flags",
        LISTED_BLOCK_LIBRARY,
        &[(
            "crates/fextern/build.rs",
            "fn main() {\n    println!(\"cargo:rustc-flags=-l fakesync\");\n}\n",
        )],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// 场景：入口在 `fentry`，它依赖名单块所在的 `fextern` 与带 `#[link]` 的 `fdeplink`（`fextern` 自己不依赖 `fdeplink`，两者链进同一份产物）。
/// 预期：拒算。
#[test]
fn link_in_another_dependency_of_the_entry_package_refuses_the_listed_block() {
    let tree = FixtureTree::materialize(
        "extern-link-beside-the-listed-package",
        &[
            (
                "crates/fentry/Cargo.toml",
                "[package]\nname = \"fentry\"\n\n[dependencies]\nfextern = { path = \"../fextern\" }\nfdeplink = { path = \"../fdeplink\" }\n",
            ),
            (
                "crates/fentry/src/lib.rs",
                "pub fn entry_point() -> i32 {\n    fdeplink::linked();\n    fextern::entry_point()\n}\n",
            ),
            ("crates/fextern/Cargo.toml", "[package]\nname = \"fextern\"\n"),
            (EXTERN_LIBRARY_FILE, LISTED_BLOCK_LIBRARY),
            ("crates/fdeplink/Cargo.toml", "[package]\nname = \"fdeplink\"\n"),
            ("crates/fdeplink/src/lib.rs", LINKING_LIBRARY),
            NO_DEPENDENCY_RESOLUTION,
        ],
    );
    assert_refused_as_extern_function_interface(key_verdict(
        &tree,
        &library_entry("fentry::entry_point"),
    ));
}

/// 场景（对照）：工作区里另一个包 `funrelated` 与 `fextern` 没有依赖关系，它有带 `#[link]` 的空块，它的构建脚本打 `rustc-link-lib`。
/// 预期：名单块算得出键（不链进这一份产物的包不看）。
#[test]
fn links_in_a_package_outside_the_dependencies_leave_the_listed_block_keyed() {
    let tree = extern_tree(
        "extern-link-in-an-unrelated-package",
        LISTED_BLOCK_LIBRARY,
        &[
            (
                "crates/funrelated/Cargo.toml",
                "[package]\nname = \"funrelated\"\n",
            ),
            ("crates/funrelated/src/lib.rs", LINKING_LIBRARY),
            (
                "crates/funrelated/build.rs",
                "fn main() {\n    println!(\"cargo:rustc-link-lib=fakesync\");\n}\n",
            ),
        ],
    );
    let verdict = key_verdict(&tree, &EXTERN_ENTRY);
    assert!(
        verdict.is_ok(),
        "funrelated 不链进 fextern 的产物，它的 link 与构建脚本不拒算名单块：{verdict:?}"
    );
}

/// 场景：包的构建脚本打两冒号写法 `cargo::rustc-link-lib=fakesync`。预期：拒算。
#[test]
fn build_script_printing_the_two_colon_link_instruction_refuses_the_listed_block() {
    let tree = extern_tree(
        "extern-build-script-two-colon-instruction",
        LISTED_BLOCK_LIBRARY,
        &[(
            "crates/fextern/build.rs",
            "fn main() {\n    println!(\"cargo::rustc-link-lib=fakesync\");\n}\n",
        )],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// 场景：包的构建脚本把指令写在原始串里 `println!(r#"cargo:rustc-link-lib=fakesync"#)`。预期：拒算。
#[test]
fn build_script_printing_a_link_instruction_in_a_raw_string_refuses_the_listed_block() {
    let tree = extern_tree(
        "extern-build-script-raw-string-instruction",
        LISTED_BLOCK_LIBRARY,
        &[(
            "crates/fextern/build.rs",
            "fn main() {\n    println!(r#\"cargo:rustc-link-lib=fakesync\"#);\n}\n",
        )],
    );
    assert_refused_as_extern_function_interface(key_verdict(&tree, &EXTERN_ENTRY));
}

/// 构建依赖的库：只写指令名、不带 `cargo:` 的串（单元图自己的指令名表就是这样），整句指令只出现在注释里。
const BARE_INSTRUCTION_NAMES_LIBRARY: &str = "/// 构建脚本打 `cargo:rustc-link-lib=…` 时多链接一个库。\npub const LINK_INSTRUCTION_NAMES: [&str; 3] = [\"rustc-link-lib\", \"rustc-link-arg\", \"rustc-flags\"];\n\n// cargo::rustc-flags=-l fakesync\npub fn instruction_count() -> usize {\n    LINK_INSTRUCTION_NAMES.len()\n}\n";

/// 场景：`fextern` 的构建依赖 `fbuildhelper` 的库里只有裸的指令名串 `"rustc-link-lib"` 与注释里的整句指令，没有以 `cargo:` 起头的串。
/// 预期：名单块算得出键。
#[test]
fn bare_link_instruction_names_in_a_build_dependency_library_leave_the_listed_block_keyed() {
    let tree = extern_tree_with_manifest_sections(
        "extern-bare-instruction-names-in-a-build-dependency",
        "[build-dependencies]\nfbuildhelper = { path = \"../fbuildhelper\" }\n",
        &[
            (
                "crates/fextern/build.rs",
                "fn main() {\n    assert_eq!(fbuildhelper::instruction_count(), 3);\n}\n",
            ),
            (
                "crates/fbuildhelper/Cargo.toml",
                "[package]\nname = \"fbuildhelper\"\n",
            ),
            (
                "crates/fbuildhelper/src/lib.rs",
                BARE_INSTRUCTION_NAMES_LIBRARY,
            ),
        ],
    );
    let verdict = key_verdict(&tree, &EXTERN_ENTRY);
    assert!(
        verdict.is_ok(),
        "不带 cargo: 前缀的指令名串与注释不是加库的指令，不拒算名单块：{verdict:?}"
    );
}

/// 场景：构建依赖 `fbuildhelper` 的库干净，打 `cargo:rustc-link-lib=fakesync` 的是它自己的集成测试 `tests/prints.rs`
/// （构建脚本只链它的库）。预期：名单块算得出键。
#[test]
fn link_instruction_in_a_test_of_a_build_dependency_leaves_the_listed_block_keyed() {
    let tree = extern_tree_with_manifest_sections(
        "extern-link-instruction-in-a-test-of-a-build-dependency",
        "[build-dependencies]\nfbuildhelper = { path = \"../fbuildhelper\" }\n",
        &[
            (
                "crates/fextern/build.rs",
                "fn main() {\n    fbuildhelper::nothing_to_link();\n}\n",
            ),
            (
                "crates/fbuildhelper/Cargo.toml",
                "[package]\nname = \"fbuildhelper\"\n",
            ),
            (
                "crates/fbuildhelper/src/lib.rs",
                "pub fn nothing_to_link() {}\n",
            ),
            (
                "crates/fbuildhelper/tests/prints.rs",
                "#[test]\nfn prints_the_instruction() {\n    println!(\"cargo:rustc-link-lib=fakesync\");\n}\n",
            ),
        ],
    );
    let verdict = key_verdict(&tree, &EXTERN_ENTRY);
    assert!(
        verdict.is_ok(),
        "构建依赖的测试目标不在构建时跑，它里面的指令不拒算名单块：{verdict:?}"
    );
}
