//! 命令行 `unit-graph`：dump / closure / key 三个子命令一行一项、次序确定，行格式是单元行、键行、拒算行那三种；
//! 拒算退 3、用法错退 2（缺目标三元组、两种依赖行都给、入口在这组 cfg 下不编进来也是用法错）。

mod common_fixture_workspace;

use std::process::{Command, Output};

use common_fixture_workspace::{FixtureTree, RULE_CELL_FIXTURE};

fn unit_graph(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_unit-graph"))
        .args(arguments)
        .output()
        .expect("测试二进制旁边的 unit-graph 起得来")
}

fn stdout_lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

/// `rustc -vV` 的宿主三元组那一行（closure 与 key 都要有一行目标三元组）。
const HOST_LINE: &str = "host: x86_64-unknown-linux-gnu";

fn is_hexadecimal_digest(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[test]
fn dump_prints_four_tab_separated_fields_per_unit_and_the_same_lines_for_any_thread_count() {
    let tree = FixtureTree::materialize("command-line-dump", &RULE_CELL_FIXTURE);
    let root = tree.root().to_string_lossy().into_owned();
    let single = unit_graph(&["dump", &root, "--threads", "1"]);
    assert_eq!(
        single.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&single.stderr)
    );
    let lines = stdout_lines(&single);
    assert!(!lines.is_empty());
    for line in &lines {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 4, "每行四段：{line}");
        if fields[0] != "refused" {
            assert!(
                is_hexadecimal_digest(fields[3]),
                "单元行最后一段是 64 位十六进制指纹：{line}"
            );
            assert!(
                fields[2].starts_with("crates/") && fields[2].contains(':'),
                "第三段是从仓根起的文件:行：{line}"
            );
        }
    }
    assert!(lines
        .iter()
        .any(|line| line.starts_with("fn\tftier::judge_operator\tcrates/ftier/src/lib.rs:")));
    assert!(lines
        .iter()
        .any(|line| line.starts_with("doctest\tfcore::with_example::<doctest line ")));
    let several = unit_graph(&["dump", &root, "--threads", "4"]);
    assert_eq!(stdout_lines(&several), lines, "多线程与单线程逐字相同");
}

#[test]
fn closure_lists_the_composition_and_ends_with_the_key_that_key_prints() {
    let tree = FixtureTree::materialize("command-line-closure", &RULE_CELL_FIXTURE);
    let root = tree.root().to_string_lossy().into_owned();
    let closure = unit_graph(&[
        "closure",
        &root,
        "ftier::judge_operator",
        "--toolchain-line",
        HOST_LINE,
    ]);
    assert_eq!(
        closure.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&closure.stderr)
    );
    let lines = stdout_lines(&closure);
    let last = lines.last().expect("closure 至少打键行");
    let key_digest = last.strip_prefix("key\t").expect("最后一行是键行");
    assert!(is_hexadecimal_digest(key_digest));
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("fn\tfcore::<impl std::ops::Add for Money>::add\t")),
        "{lines:?}"
    );
    let key = unit_graph(&[
        "key",
        &root,
        "ftier::judge_operator",
        "--toolchain-line",
        HOST_LINE,
        "--toolchain-line",
        "rustc 1.98.0",
    ]);
    assert_eq!(key.status.code(), Some(0));
    let key_lines = stdout_lines(&key);
    assert_eq!(key_lines.len(), 1);
    assert_ne!(key_lines[0], *last, "工具链行进键");
    let key_without_toolchain = unit_graph(&[
        "key",
        &root,
        "ftier::judge_operator",
        "--toolchain-line",
        HOST_LINE,
    ]);
    assert_eq!(
        stdout_lines(&key_without_toolchain),
        std::slice::from_ref(last),
        "key 与 closure 的键行相同"
    );
}

#[test]
fn a_refused_closure_exits_three_and_prints_no_key_line() {
    let tree = FixtureTree::materialize(
        "command-line-refused",
        &[
            (
                "crates/refusing/Cargo.toml",
                "[package]\nname = \"refusing\"\nversion = \"0.1.0\"\n",
            ),
            (
                "crates/refusing/src/lib.rs",
                "pub fn entry_point() -> bool {\n    std::env::current_exe().is_ok()\n}\n",
            ),
        ],
    );
    let root = tree.root().to_string_lossy().into_owned();
    let key = unit_graph(&[
        "key",
        &root,
        "refusing::entry_point",
        "--toolchain-line",
        HOST_LINE,
    ]);
    assert_eq!(key.status.code(), Some(3));
    let lines = stdout_lines(&key);
    assert_eq!(lines.len(), 1);
    assert!(
        lines[0].starts_with("refused\tcurrent-exe\tcrates/refusing/src/lib.rs:2\t"),
        "{lines:?}"
    );
}

#[test]
fn an_unknown_entry_or_option_exits_two() {
    let tree = FixtureTree::materialize("command-line-usage", &RULE_CELL_FIXTURE);
    let root = tree.root().to_string_lossy().into_owned();
    assert_eq!(
        unit_graph(&[
            "closure",
            &root,
            "ftier::no_such_function",
            "--toolchain-line",
            HOST_LINE
        ])
        .status
        .code(),
        Some(2)
    );
    assert_eq!(
        unit_graph(&["dump", &root, "--no-such-option"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(unit_graph(&["closure", &root]).status.code(), Some(2));
}

#[test]
fn a_closure_without_a_target_triple_line_exits_two() {
    let tree = FixtureTree::materialize("command-line-no-triple", &RULE_CELL_FIXTURE);
    let root = tree.root().to_string_lossy().into_owned();
    let closure = unit_graph(&[
        "closure",
        &root,
        "ftier::judge_operator",
        "--toolchain-line",
        "rustc 1.98.0",
    ]);
    assert_eq!(closure.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&closure.stderr).contains("缺目标三元组"),
        "{}",
        String::from_utf8_lossy(&closure.stderr)
    );
    assert!(stdout_lines(&closure).is_empty(), "用法错不交任何行");
}

#[test]
fn an_entry_whose_file_is_only_compiled_under_a_feature_exits_two_without_that_feature() {
    let tree = FixtureTree::materialize(
        "command-line-gated-entry",
        &[
            (
                "crates/fgated/Cargo.toml",
                "[package]\nname = \"fgated\"\nversion = \"0.1.0\"\n\n[features]\ngpu = []\n",
            ),
            (
                "crates/fgated/src/lib.rs",
                "#[cfg(feature = \"gpu\")]\npub mod kernels;\n",
            ),
            (
                "crates/fgated/src/kernels.rs",
                "pub fn kernel_source() -> &'static str {\n    \"fn main() {}\"\n}\n",
            ),
        ],
    );
    let root = tree.root().to_string_lossy().into_owned();
    let without_feature = unit_graph(&[
        "key",
        &root,
        "fgated::kernels::kernel_source",
        "--toolchain-line",
        HOST_LINE,
    ]);
    assert_eq!(
        without_feature.status.code(),
        Some(2),
        "整份文件在 feature = \"gpu\" 下、没给这个特性：入口不编进来，用法错，不交空闭包的键：{}",
        String::from_utf8_lossy(&without_feature.stderr)
    );
    assert!(stdout_lines(&without_feature).is_empty());
    let with_feature = unit_graph(&[
        "key",
        &root,
        "fgated::kernels::kernel_source",
        "--cfg",
        "feature=\"gpu\"",
        "--toolchain-line",
        HOST_LINE,
    ]);
    assert_eq!(
        with_feature.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&with_feature.stderr)
    );
}

#[test]
fn a_lock_file_whose_packages_miss_a_named_external_crate_exits_three_and_lock_lines_with_a_lock_file_exit_two(
) {
    let tree = FixtureTree::materialize(
        "command-line-lock-file",
        &[
            (
                "crates/fconsumer/Cargo.toml",
                "[package]\nname = \"fconsumer\"\nversion = \"0.1.0\"\n\n[dependencies]\ndelta = \"1\"\n",
            ),
            (
                "crates/fconsumer/src/lib.rs",
                "pub fn uses_delta() -> u32 {\n    delta::value()\n}\n",
            ),
            (
                "Cargo.lock",
                "version = 4\n\n[[package]]\nname = \"fconsumer\"\nversion = \"0.1.0\"\n",
            ),
        ],
    );
    let root = tree.root().to_string_lossy().into_owned();
    let lock_file = tree
        .root()
        .join("Cargo.lock")
        .to_string_lossy()
        .into_owned();
    let key = unit_graph(&[
        "key",
        &root,
        "fconsumer::uses_delta",
        "--toolchain-line",
        HOST_LINE,
        "--lock-file",
        &lock_file,
    ]);
    assert_eq!(key.status.code(), Some(3));
    assert_eq!(
        stdout_lines(&key),
        ["refused\tlock-dependency-unresolved\tcrates/fconsumer/src/lib.rs:2\tfconsumer → delta"]
    );
    let both = unit_graph(&[
        "key",
        &root,
        "fconsumer::uses_delta",
        "--toolchain-line",
        HOST_LINE,
        "--lock-file",
        &lock_file,
        "--lock-line",
        "delta\t1.0.0\t-\t-",
    ]);
    assert_eq!(both.status.code(), Some(2), "两种依赖行只能给一种");
}
