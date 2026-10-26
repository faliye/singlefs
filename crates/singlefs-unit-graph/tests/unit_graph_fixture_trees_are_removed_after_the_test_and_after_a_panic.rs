//! 夹具树（`tests/common_fixture_workspace/mod.rs` 的 `FixtureTree`）跑完就删：用例照常走完时删，写夹具写到一半 panic 时
//! 写了的那一半也删（`.claude/singlefs-ai-sop/rules/command-safety.md`「测试镜像一律放临时目录」）。

mod common_fixture_workspace;

use common_fixture_workspace::FixtureTree;

/// 临时目录里名字以 `prefix` 起头的条目：这条用例的标签只在这里用，并行跑的别的用例建不出这个前缀。
fn temporary_entries_named_after(prefix: &str) -> Vec<String> {
    std::fs::read_dir(std::env::temp_dir())
        .expect("临时目录读得了")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(prefix))
        .collect()
}

/// 场景：写一棵两份文件的夹具树，用例照常走完。预期：走完之前根在、两份文件在，走完之后根不在。
/// 判别力（`crates/mutations.tsv`）：夹具树的 Drop 不删，根还在，这条红。
#[test]
fn a_fixture_tree_is_gone_once_the_test_scope_ends_normally() {
    let root = {
        let tree = FixtureTree::materialize(
            "fixture-tree-normal-end",
            &[
                ("crates/one/Cargo.toml", "[package]\n"),
                ("crates/one/src/lib.rs", ""),
            ],
        );
        assert!(
            tree.root().join("crates/one/src/lib.rs").exists(),
            "走完之前夹具文件在"
        );
        tree.root().to_path_buf()
    };
    assert!(!root.exists(), "走完之后夹具树 {} 不在了", root.display());
}

/// 场景：夹具里先写一份文件 `crates/one/lib.rs`，再写一份要把它当目录用的 `crates/one/lib.rs/inner`：建目录那一步失败、
/// 写夹具途中 panic。预期：panic 接住之后，这条用例标签下的临时目录一个都不在（写了的那一半跟着删了）。
/// 判别力（`crates/mutations.tsv`）：夹具树的 Drop 不删，半棵树还在，这条红。
#[test]
fn a_fixture_tree_half_written_when_a_panic_hits_is_gone() {
    let label = "fixture-tree-panic-while-writing";
    let prefix = format!("singlefs-unit-graph-{label}-{}-", std::process::id());
    let panicked = std::panic::catch_unwind(|| {
        FixtureTree::materialize(
            label,
            &[
                ("crates/one/lib.rs", "pub fn one() {}\n"),
                ("crates/one/lib.rs/inner", "写不进：上一级是文件\n"),
            ],
        )
    });
    assert!(panicked.is_err(), "写夹具途中真 panic 了");
    assert_eq!(
        temporary_entries_named_after(&prefix),
        Vec::<String>::new(),
        "panic 之后 {prefix}* 一个都不在"
    );
}
