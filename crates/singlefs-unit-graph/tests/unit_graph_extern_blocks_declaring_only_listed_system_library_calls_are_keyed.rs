//! `extern` 块（`src/rust_items.rs` 的 `extern_block_declares_only_listed_system_library_calls`，`scripts/gate/lib_function_units.py` 同一判法）：ABI 是 `"C"`、
//! 没有 `#[link…]`、块里只有 `fn` 声明、每条的符号（`#[link_name]` 或函数名）都在名单里的不拒算；别的写法拒算 `extern-function-interface`。
//! 夹具只切词、不编译。

mod common_fixture_workspace;

use common_fixture_workspace::{key_of, workspace_of, EntrySpecification, FixtureTree};
use singlefs_unit_graph::rule_switches::RuleSwitches;

const LIBRARY_FILE: &str = "crates/fextern/src/lib.rs";

const ENTRY: EntrySpecification = EntrySpecification {
    entry_path: "fextern::entry_point",
    target: "lib",
    conditions: &[],
};

const LISTED_CALLS_UNDER_LINK_NAMES: &str = r#"extern "C" {
    #[link_name = "pwrite"]
    fn pwrite_raw(
        fd: i32,
        buf: *const core::ffi::c_void,
        count: usize,
        offset: i64,
    ) -> isize;
    #[link_name = "fdatasync"]
    fn fdatasync_raw(fd: i32) -> i32;
}
pub fn entry_point() -> i32 {
    unsafe { fdatasync_raw(3) }
}
"#;

fn tree_of(label: &str, library: &str) -> FixtureTree {
    FixtureTree::materialize(
        label,
        &[
            (
                "crates/fextern/Cargo.toml",
                "[package]\nname = \"fextern\"\n",
            ),
            (LIBRARY_FILE, library),
            ("Cargo.lock", "# 夹具不解析依赖\n"),
        ],
    )
}

fn key_text(tree: &FixtureTree) -> Result<String, String> {
    let switches = RuleSwitches::every_rule_intact();
    key_of(&workspace_of(tree.root(), &switches), &ENTRY, &switches).map(|key| key.hexadecimal())
}

fn assert_refused_as_extern_function_interface(label: &str, library: &str) {
    let tree = tree_of(label, library);
    let reason = key_text(&tree).expect_err("这种 extern 块拒算");
    assert!(
        reason.contains("refused\textern-function-interface\tcrates/fextern/src/lib.rs:"),
        "拒算行点名 extern-function-interface 与夹具文件：{reason}"
    );
}

/// 场景：`#[link_name]` 指到名单里的两个符号（E107 的形状）。预期：算得出键；改声明的符号以外的调用方函数体键变。
#[test]
fn listed_calls_under_link_names_are_keyed_and_the_key_follows_the_caller() {
    let tree = tree_of("extern-listed-link-names", LISTED_CALLS_UNDER_LINK_NAMES);
    let kept = key_text(&tree).unwrap_or_else(|reason| panic!("名单里的 C 库调用不拒算：{reason}"));
    tree.replace_exactly_once(LIBRARY_FILE, "fdatasync_raw(3)", "fdatasync_raw(4)");
    let after_the_edit = key_text(&tree).expect("改调用方之后照样算得出键");
    assert_ne!(kept, after_the_edit, "调用方改了，键要变");
}

/// 场景：不写 `#[link_name]`，函数名就是名单里的符号。预期：算得出键。
#[test]
fn listed_call_declared_under_its_own_name_is_keyed() {
    let tree = tree_of(
        "extern-listed-own-name",
        "extern \"C\" {\n    pub fn fdatasync(fd: i32) -> i32;\n}\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n",
    );
    key_text(&tree).unwrap_or_else(|reason| panic!("名单里的 C 库调用不拒算：{reason}"));
}

/// 场景：符号不在名单里。预期：拒算。
#[test]
fn symbol_outside_the_list_is_refused() {
    assert_refused_as_extern_function_interface(
        "extern-unlisted",
        "extern \"C\" {\n    fn external_symbol() -> u32;\n}\npub fn entry_point() -> u32 {\n    unsafe { external_symbol() }\n}\n",
    );
}

/// 场景：函数名在名单里，`#[link_name]` 指到名单外的符号。预期：拒算：符号看 `link_name`。
#[test]
fn listed_name_linked_to_a_symbol_outside_the_list_is_refused() {
    assert_refused_as_extern_function_interface(
        "extern-link-name-outside",
        "extern \"C\" {\n    #[link_name = \"my_native_write\"]\n    fn fdatasync(fd: i32) -> i32;\n}\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n",
    );
}

/// 场景：`#[link_name]` 在名单里，函数名不在，后面又多声明一条名单外的。预期：拒算：每一条都要在名单里。
#[test]
fn one_symbol_outside_the_list_refuses_the_whole_block() {
    assert_refused_as_extern_function_interface(
        "extern-one-outside",
        "extern \"C\" {\n    #[link_name = \"fdatasync\"]\n    fn sync_raw(fd: i32) -> i32;\n    fn external_symbol() -> u32;\n}\npub fn entry_point() -> i32 {\n    unsafe { sync_raw(3) }\n}\n",
    );
}

/// 场景：块前面写 `#[link(name = "…")]`，符号在名单里。预期：拒算：符号可能从点名的本地库来。
#[test]
fn block_with_a_link_attribute_is_refused() {
    assert_refused_as_extern_function_interface(
        "extern-link-attribute",
        "#[link(name = \"mine\")]\nextern \"C\" {\n    fn fdatasync(fd: i32) -> i32;\n}\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n",
    );
}

/// 场景：ABI 不是 `"C"`。预期：拒算。
#[test]
fn block_with_another_abi_is_refused() {
    assert_refused_as_extern_function_interface(
        "extern-another-abi",
        "extern \"system\" {\n    fn fdatasync(fd: i32) -> i32;\n}\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n",
    );
}

/// 场景：块里有 `static`。预期：拒算：只认 `fn` 声明。
#[test]
fn block_declaring_a_static_is_refused() {
    assert_refused_as_extern_function_interface(
        "extern-static",
        "extern \"C\" {\n    fn fdatasync(fd: i32) -> i32;\n    static environ: *const u8;\n}\npub fn entry_point() -> i32 {\n    unsafe { fdatasync(3) }\n}\n",
    );
}
