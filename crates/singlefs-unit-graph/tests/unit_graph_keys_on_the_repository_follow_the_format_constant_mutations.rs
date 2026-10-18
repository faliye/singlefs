//! 真仓上的阳性与阴性对照（设计报告 2.2 那张表）：测试自己把仓里的 `crates/`（连 `Cargo.toml`、`Cargo.lock`、
//! `.claude/heavy-thread-default`、`litmus/`）拷进临时目录，施加 `crates/mutations.tsv` 里按函数切会漏的两条变异
//! （格式库的 `DATA_POINTER_BYTES` 改一个数、池级 checker 的 `JOURNAL_PAYLOAD_CHECKSUM_OFFSET` 改一个数），
//! 看三个登记入口（格式库那条单测、新池发布那条集成测试、CPU 判器 `judge_crash_image`）的键照表变或不变；
//! 打开去掉 const 的弄坏开关，两条变异在三个入口上都转成「不变」。另核整仓切分认得每一种条目起头、多线程与单线程逐字相同；
//! 失败点码表的条目级宏调用改一个码，拼恢复内核的函数的键要变（WGSL 外名的着落做成边）。

mod common_fixture_workspace;

use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use common_fixture_workspace::{key_of, EntrySpecification, FixtureTree};
use singlefs_unit_graph::heavy_threads::default_heavy_threads;
use singlefs_unit_graph::output::dump_lines;
use singlefs_unit_graph::rule_switches::{BrokenRule, RuleSwitches};
use singlefs_unit_graph::{load_workspace, Workspace};

static NEXT_COPY_NUMBER: AtomicUsize = AtomicUsize::new(0);

/// 整仓切分用登记的每台默认重活线程数（`SINGLEFS_THREAD_CAP` 设了照它）。
fn repository_workspace(root: &Path, switches: &RuleSwitches) -> Workspace {
    load_workspace(root, default_heavy_threads(), switches)
        .unwrap_or_else(|error| panic!("装 {} 失败：{error}", root.display()))
}

/// 格式库那条单测（V11 点名的测试）。
const FORMAT_LIBRARY_WIDTH_TEST: EntrySpecification = EntrySpecification {
    entry_path: "singlefs_format::tests::pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for",
    target: "lib",
    conditions: &["test"],
};

/// 新池发布那条集成测试（V5 点名的测试）。
const NEW_POOL_PUBLISH_TEST: EntrySpecification = EntrySpecification {
    entry_path: "singlefs_harness::root_slots_system_configurations_and_journal_ring_hold_the_published_state",
    target: "test:new_pool_file_creation_publish",
    conditions: &[],
};

/// CPU 判器入口。
const CENTRAL_PROCESSOR_JUDGE: EntrySpecification = EntrySpecification {
    entry_path: "singlefs_checker_tier::crash::judge_crash_image",
    target: "lib",
    conditions: &[],
};

/// `crates/mutations.tsv` 里格式库常量那一行（设计报告叫 V11）。
const POINTER_WIDTH_MUTATION: &str = "格式常量字面量改掉一个数：DATA_POINTER_BYTES";
/// `crates/mutations.tsv` 里池级 checker 常量那一行（设计报告叫 V5）。
const CHECKSUM_OFFSET_MUTATION: &str = "记录头 311：池级 checker 的载荷校验和偏移没跟着后挪 4 字节";

fn repository_root() -> PathBuf {
    let crate_directory = Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_directory
        .parent()
        .and_then(Path::parent)
        .expect("本 crate 在仓根的 crates/ 下")
        .to_path_buf()
}

fn copy_directory(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap_or_else(|error| panic!("建 {}：{error}", to.display()));
    let entries =
        std::fs::read_dir(from).unwrap_or_else(|error| panic!("读 {}：{error}", from.display()));
    for entry in entries {
        let entry = entry.expect("目录项读得出");
        let name = entry.file_name();
        if name == "target" || name == ".git" {
            continue;
        }
        let source = entry.path();
        let destination = to.join(&name);
        if source.is_dir() {
            copy_directory(&source, &destination);
        } else {
            std::fs::copy(&source, destination)
                .unwrap_or_else(|error| panic!("拷 {}：{error}", source.display()));
        }
    }
}

/// 把仓里这几样拷进一个新的临时目录。
fn repository_copy(label: &str) -> FixtureTree {
    let repository = repository_root();
    let copy_number = NEXT_COPY_NUMBER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!(
        "singlefs-unit-graph-{label}-{}-{copy_number}",
        std::process::id()
    ));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("上一趟同名的临时目录删得掉");
    }
    copy_directory(&repository.join("crates"), &root.join("crates"));
    copy_directory(&repository.join("litmus"), &root.join("litmus"));
    for file in ["Cargo.toml", "Cargo.lock", ".claude/heavy-thread-default"] {
        let destination = root.join(file);
        std::fs::create_dir_all(destination.parent().expect("文件在某个目录里")).expect("建目录");
        std::fs::copy(repository.join(file), destination)
            .unwrap_or_else(|error| panic!("拷 {file}：{error}"));
    }
    FixtureTree::adopt(root)
}

/// 变异表里按名字找那一行：（文件，原文，替换文），`\n` 还原成换行。
fn mutation_row(tree: &FixtureTree, name: &str) -> (String, String, String) {
    let table =
        std::fs::read_to_string(tree.root().join("crates/mutations.tsv")).expect("变异表读得出");
    let row = table
        .lines()
        .find(|line| line.split('\t').next() == Some(name))
        .unwrap_or_else(|| panic!("变异表里没有「{name}」那一行"));
    let fields: Vec<&str> = row.split('\t').collect();
    assert_eq!(fields.len(), 6, "变异行六段：{row}");
    let unescaped = |field: &str| field.replace("\\n", "\n");
    (
        fields[1].to_string(),
        unescaped(fields[2]),
        unescaped(fields[3]),
    )
}

/// 三个入口在施加这条变异前后键变没变（变为 true），按 格式库单测、新池发布测试、CPU 判器 的次序。
fn key_changes_under_mutation(mutation_name: &str, switches: &RuleSwitches) -> [bool; 3] {
    let tree = repository_copy("repository-mutation");
    let entries = [
        FORMAT_LIBRARY_WIDTH_TEST,
        NEW_POOL_PUBLISH_TEST,
        CENTRAL_PROCESSOR_JUDGE,
    ];
    let base_workspace = repository_workspace(tree.root(), switches);
    let before: Vec<String> = entries
        .iter()
        .map(|entry| {
            key_of(&base_workspace, entry, switches)
                .unwrap_or_else(|reason| panic!("{reason}"))
                .hexadecimal()
        })
        .collect();
    drop(base_workspace);
    let (file, original, replacement) = mutation_row(&tree, mutation_name);
    tree.replace_exactly_once(&file, &original, &replacement);
    let variant_workspace = repository_workspace(tree.root(), switches);
    let mut changed = [false; 3];
    for (position, entry) in entries.iter().enumerate() {
        let after = key_of(&variant_workspace, entry, switches)
            .unwrap_or_else(|reason| panic!("{reason}"))
            .hexadecimal();
        changed[position] = after != before[position];
    }
    changed
}

#[test]
fn the_checksum_offset_mutation_changes_the_publish_test_and_judge_keys_but_not_the_format_test_key(
) {
    assert_eq!(
        key_changes_under_mutation(CHECKSUM_OFFSET_MUTATION, &RuleSwitches::every_rule_intact()),
        [false, true, true],
        "池级 checker 的私有常量格式库够不到；新池发布测试与 CPU 判器都经它判"
    );
}

#[test]
fn the_pointer_width_mutation_changes_all_three_entry_keys() {
    assert_eq!(
        key_changes_under_mutation(POINTER_WIDTH_MUTATION, &RuleSwitches::every_rule_intact()),
        [true, true, true],
        "格式常量三个入口都用到"
    );
}

#[test]
fn without_constant_units_the_checksum_offset_mutation_changes_no_entry_key() {
    assert_eq!(
        key_changes_under_mutation(
            CHECKSUM_OFFSET_MUTATION,
            &RuleSwitches::with_broken(&[BrokenRule::ConstantAndStaticUnits])
        ),
        [false, false, false],
        "按函数切（不切 const）漏掉这条变异"
    );
}

#[test]
fn without_constant_units_the_pointer_width_mutation_changes_no_entry_key() {
    assert_eq!(
        key_changes_under_mutation(
            POINTER_WIDTH_MUTATION,
            &RuleSwitches::with_broken(&[BrokenRule::ConstantAndStaticUnits])
        ),
        [false, false, false],
        "按函数切（不切 const）漏掉这条变异"
    );
}

#[test]
fn dumping_the_repository_recognizes_every_item_start_and_balances_every_bracket() {
    let switches = RuleSwitches::every_rule_intact();
    let workspace = repository_workspace(&repository_root(), &switches);
    let structural: Vec<String> = dump_lines(&workspace)
        .into_iter()
        .filter(|line| {
            line.starts_with("refused\tunrecognized-item-start\t")
                || line.starts_with("refused\tunbalanced-brackets\t")
                || line.starts_with("refused\twgsl-unrecognized-item-start\t")
        })
        .collect();
    assert!(
        structural.is_empty(),
        "仓里每一种条目起头都认得：{structural:?}"
    );
}

#[test]
fn dumping_the_repository_with_one_thread_and_with_several_threads_gives_the_same_lines() {
    let switches = RuleSwitches::every_rule_intact();
    let root = repository_root();
    let single = singlefs_unit_graph::load_workspace(
        &root,
        NonZeroUsize::new(1).expect("1 是正整数"),
        &switches,
    )
    .expect("仓装得起来");
    let several = singlefs_unit_graph::load_workspace(
        &root,
        NonZeroUsize::new(8).expect("8 是正整数"),
        &switches,
    )
    .expect("仓装得起来");
    assert_eq!(dump_lines(&single), dump_lines(&several));
}

/// GPU 判器拼内核源码的三个函数（`crash_judge_gpu.rs` 整份在 `feature = "gpu"` 下）：内核里 Rust 现拼的外名
/// （`format!` 拼的 `T_*`、宏 `gpu_failure_points!` 的名字去掉 `GPU_` 拼的 `FAILURE_POINT_*`）都有着落、`_ = …` 不当外名，三把键都交得出。
const GPU_KERNEL_SOURCE_ENTRIES: [&str; 3] = [
    "singlefs_checker_tier::crash_judge_gpu::recovery_kernel_source",
    "singlefs_checker_tier::crash_judge_gpu::checker_walk_kernel_source",
    "singlefs_checker_tier::crash_judge_gpu::checker_scan_kernel_source",
];

#[test]
fn the_three_gpu_kernel_source_functions_close_with_no_refusal() {
    let switches = RuleSwitches::every_rule_intact();
    let workspace = repository_workspace(&repository_root(), &switches);
    for entry_path in GPU_KERNEL_SOURCE_ENTRIES {
        let verdict = key_of(
            &workspace,
            &EntrySpecification {
                entry_path,
                target: "lib",
                conditions: &["feature=\"gpu\""],
            },
            &switches,
        );
        assert!(verdict.is_ok(), "{entry_path} 不该拒算：{verdict:?}");
    }
}

/// 失败点码表（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs` 的条目级宏调用 `gpu_failure_points! { … }`）里改一个码
/// 前后，拼恢复内核的函数的键：都交得出时交两把是否不同，有一把拒算交拒算说明。
fn failure_point_code_change_changes_the_recovery_kernel_key(
    switches: &RuleSwitches,
) -> Result<bool, String> {
    let entry = EntrySpecification {
        entry_path: GPU_KERNEL_SOURCE_ENTRIES[0],
        target: "lib",
        conditions: &["feature=\"gpu\""],
    };
    let tree = repository_copy("repository-failure-point-code");
    let before = key_of(
        &repository_workspace(tree.root(), switches),
        &entry,
        switches,
    )?;
    tree.replace_exactly_once(
        "crates/singlefs-checker-tier/src/crash_judge_conclusions.rs",
        "    GPU_FAILURE_POINT_UNIT_UNREADABLE = 9,\n",
        "    GPU_FAILURE_POINT_UNIT_UNREADABLE = 99,\n",
    );
    let after = key_of(
        &repository_workspace(tree.root(), switches),
        &entry,
        switches,
    )?;
    Ok(before != after)
}

#[test]
fn changing_a_failure_point_code_in_the_item_macro_changes_the_recovery_kernel_key() {
    assert_eq!(
        failure_point_code_change_changes_the_recovery_kernel_key(
            &RuleSwitches::every_rule_intact()
        ),
        Ok(true),
        "恢复内核用的 FAILURE_POINT_* 由宏调用里的 GPU_FAILURE_POINT_* 供：调用经着落的边进闭包，改码键变"
    );
}

#[test]
fn without_supplier_edges_the_failure_point_code_change_is_missed() {
    let verdict =
        failure_point_code_change_changes_the_recovery_kernel_key(&RuleSwitches::with_broken(&[
            BrokenRule::WgslSupplierEdges,
        ]));
    assert!(
        verdict != Ok(true),
        "着落不做成边：宏调用不进恢复内核的闭包，改码不该照常换键：{verdict:?}"
    );
}
