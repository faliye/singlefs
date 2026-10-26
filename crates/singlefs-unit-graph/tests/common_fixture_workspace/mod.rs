//! 测试共用的搭建：把夹具（`tests/fixtures/` 下以 `.fixture` 收尾的文件，免得仓里的切分、命名与编译把它们当真代码）
//! 写进一个临时目录，按原样改一处、装工作区、算键。每条用例自己建、自己删目录。

#![allow(
    dead_code,
    reason = "几份测试文件各用这里的一部分，没用到的那部分在那一份里算死代码"
)]

use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use singlefs_unit_graph::output::refusal_lines;
use singlefs_unit_graph::rule_switches::{RuleSwitches, BREAK_ENVIRONMENT_VARIABLE};
use singlefs_unit_graph::sha256::Sha256Digest;
use singlefs_unit_graph::{
    compute_closure, load_workspace, reuse_key, ClosureOutcome, ClosureRequest, EntryError,
    ExternalKeyInputs, RequestedCondition, TargetSelector, Workspace,
};

/// 夹具与真仓的用例默认用的目标三元组（本机 `rustc -vV` 的 `host:` 行）。
pub const HOST_TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";

/// 规则格夹具：设计报告 2.3 的三个 crate（`fcore` ← `fharness` ← `ftier`），加上带路径宏调用的 `fpainter`、
/// `verus!` 的 `fverified`、谁都不依赖的 `funlinked`。
pub const RULE_CELL_FIXTURE: [(&str, &str); 14] = [
    (
        "crates/fcore/Cargo.toml",
        include_str!("../fixtures/rule_cells/fcore/Cargo.toml.fixture"),
    ),
    (
        "crates/fcore/src/lib.rs",
        include_str!("../fixtures/rule_cells/fcore/src/lib.rs.fixture"),
    ),
    (
        "crates/fcore/data/table.txt",
        include_str!("../fixtures/rule_cells/fcore/data/table.txt.fixture"),
    ),
    (
        "crates/fharness/Cargo.toml",
        include_str!("../fixtures/rule_cells/fharness/Cargo.toml.fixture"),
    ),
    (
        "crates/fharness/src/lib.rs",
        include_str!("../fixtures/rule_cells/fharness/src/lib.rs.fixture"),
    ),
    (
        "crates/ftier/Cargo.toml",
        include_str!("../fixtures/rule_cells/ftier/Cargo.toml.fixture"),
    ),
    (
        "crates/ftier/src/lib.rs",
        include_str!("../fixtures/rule_cells/ftier/src/lib.rs.fixture"),
    ),
    (
        "crates/fpainter/Cargo.toml",
        include_str!("../fixtures/rule_cells/fpainter/Cargo.toml.fixture"),
    ),
    (
        "crates/fpainter/src/lib.rs",
        include_str!("../fixtures/rule_cells/fpainter/src/lib.rs.fixture"),
    ),
    (
        "crates/fverified/Cargo.toml",
        include_str!("../fixtures/rule_cells/fverified/Cargo.toml.fixture"),
    ),
    (
        "crates/fverified/src/lib.rs",
        include_str!("../fixtures/rule_cells/fverified/src/lib.rs.fixture"),
    ),
    (
        "crates/funlinked/Cargo.toml",
        include_str!("../fixtures/rule_cells/funlinked/Cargo.toml.fixture"),
    ),
    (
        "crates/funlinked/src/lib.rs",
        include_str!("../fixtures/rule_cells/funlinked/src/lib.rs.fixture"),
    ),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

/// WGSL 夹具：Rust 侧 `include_str!` 读一份 `.wgsl`，串里拼进一个 `.wgsl` 里没定义的常量。
pub const WGSL_KERNEL_FIXTURE: [(&str, &str); 3] = [
    (
        "crates/fkernel/Cargo.toml",
        include_str!("../fixtures/wgsl_kernel/fkernel/Cargo.toml.fixture"),
    ),
    (
        "crates/fkernel/src/lib.rs",
        include_str!("../fixtures/wgsl_kernel/fkernel/src/lib.rs.fixture"),
    ),
    (
        "crates/fkernel/src/kernel.wgsl",
        include_str!("../fixtures/wgsl_kernel/fkernel/src/kernel.wgsl.fixture"),
    ),
];

/// 与 `scripts/gate/lib_function_units.py` 同形的四份夹具（定案 2、3 的 Rust 侧，4-3d 报告第三节 R1–R4 的验收）：键的条件成分只取闭包真引到的原子
/// （`fcfg`）、`use a::{self}` 引进 `a`（`fuse`）、R-开放只按 trait 自己的名字找 impl（`fopen`）、blanket impl 够得到的方法算边（`fblanket`）；
/// 再加 `const _` 不是 R-裸名的着落（`funderscore`，与那边的 `UNDERSCORE_FIXTURE` 同形）。
pub const CLOSURE_RULES_SHARED_WITH_PYTHON_FIXTURE: [(&str, &str); 11] = [
    (
        "crates/fcfg/Cargo.toml",
        include_str!("../fixtures/closure_rules_shared_with_python/fcfg/Cargo.toml.fixture"),
    ),
    (
        "crates/fcfg/src/lib.rs",
        include_str!("../fixtures/closure_rules_shared_with_python/fcfg/src/lib.rs.fixture"),
    ),
    (
        "crates/fuse/Cargo.toml",
        include_str!("../fixtures/closure_rules_shared_with_python/fuse/Cargo.toml.fixture"),
    ),
    (
        "crates/fuse/src/lib.rs",
        include_str!("../fixtures/closure_rules_shared_with_python/fuse/src/lib.rs.fixture"),
    ),
    (
        "crates/fopen/Cargo.toml",
        include_str!("../fixtures/closure_rules_shared_with_python/fopen/Cargo.toml.fixture"),
    ),
    (
        "crates/fopen/src/lib.rs",
        include_str!("../fixtures/closure_rules_shared_with_python/fopen/src/lib.rs.fixture"),
    ),
    (
        "crates/fblanket/Cargo.toml",
        include_str!("../fixtures/closure_rules_shared_with_python/fblanket/Cargo.toml.fixture"),
    ),
    (
        "crates/fblanket/src/lib.rs",
        include_str!("../fixtures/closure_rules_shared_with_python/fblanket/src/lib.rs.fixture"),
    ),
    (
        "crates/funderscore/Cargo.toml",
        include_str!("../fixtures/closure_rules_shared_with_python/funderscore/Cargo.toml.fixture"),
    ),
    (
        "crates/funderscore/src/lib.rs",
        include_str!("../fixtures/closure_rules_shared_with_python/funderscore/src/lib.rs.fixture"),
    ),
    ("Cargo.lock", "# 夹具不解析依赖\n"),
];

static NEXT_TREE_NUMBER: AtomicUsize = AtomicUsize::new(0);

/// 一棵写在临时目录里的夹具树，离开作用域时删掉：用例做完、断言失败、panic 都删，写到一半 panic 留下的半棵也删。
pub struct FixtureTree {
    root: PathBuf,
}

impl FixtureTree {
    /// 这个进程里下一棵夹具树的根（临时目录下，名字带标签、进程号与进程内序号）。
    pub fn fresh_root(label: &str) -> PathBuf {
        let tree_number = NEXT_TREE_NUMBER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!(
            "singlefs-unit-graph-{label}-{}-{tree_number}",
            std::process::id()
        ))
    }

    /// 把（相对路径，内容）逐份写进一个新的临时目录。先认领根、再写：写到一半 panic 时守卫删掉写了的那一半。
    pub fn materialize(label: &str, files: &[(&str, &str)]) -> Self {
        let tree = Self::adopt(Self::fresh_root(label));
        for (relative, content) in files {
            let path = tree.root.join(relative);
            std::fs::create_dir_all(path.parent().expect("夹具文件都在某个目录里"))
                .expect("临时目录建得起来");
            std::fs::write(path, content).expect("夹具文件写得进临时目录");
        }
        tree
    }

    /// 认领一个临时目录（测试自己往里拷仓的那一份）：先认领、再往里写，离开作用域时同样删掉。
    /// 同名的目录已经在（同进程号的旧进程被杀时留下的）就先删掉。
    pub fn adopt(root: PathBuf) -> Self {
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("上一趟同名的临时目录删得掉");
        }
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 把一份文件里恰好出现一次的原文换成替换文；不是恰好一次就 panic（锚点腐化不许悄悄放过）。
    pub fn replace_exactly_once(&self, relative: &str, original: &str, replacement: &str) {
        let path = self.root.join(relative);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("读 {}：{error}", path.display()));
        let occurrences = text.matches(original).count();
        assert_eq!(
            occurrences, 1,
            "{relative} 里原文「{original}」要恰好出现一次，实际 {occurrences} 次"
        );
        std::fs::write(&path, text.replacen(original, replacement, 1))
            .unwrap_or_else(|error| panic!("写 {}：{error}", path.display()));
    }
}

impl Drop for FixtureTree {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.root) {
            if error.kind() != std::io::ErrorKind::NotFound {
                eprintln!("删不掉夹具树 {}：{error}", self.root.display());
            }
        }
    }
}

/// 测试读弄坏开关的环境变量（平常没设，就是全开）。
pub fn switches_from_the_environment() -> RuleSwitches {
    let text = std::env::var(BREAK_ENVIRONMENT_VARIABLE).unwrap_or_default();
    RuleSwitches::from_break_variable_text(&text)
        .unwrap_or_else(|name| panic!("{BREAK_ENVIRONMENT_VARIABLE} 里的开关名「{name}」认不得"))
}

/// 测试切分用的线程数：两个线程，并行那一支照样走到。
pub fn test_threads() -> NonZeroUsize {
    NonZeroUsize::new(2).expect("2 是正整数")
}

/// 装一个工作区；装不起来就 panic。
pub fn workspace_of(root: &Path, switches: &RuleSwitches) -> Workspace {
    load_workspace(root, test_threads(), switches)
        .unwrap_or_else(|error| panic!("装 {} 失败：{error}", root.display()))
}

/// 一个入口：入口 fn 路径、目标写法（`lib`、`test:<名>`、`doctest`……）、`--cfg` 写法。
#[derive(Clone, Copy, Debug)]
pub struct EntrySpecification {
    pub entry_path: &'static str,
    pub target: &'static str,
    pub conditions: &'static [&'static str],
}

/// 一个入口在点名的目标三元组下的闭包。
pub fn closure_with_triple(
    workspace: &Workspace,
    entry: &EntrySpecification,
    switches: &RuleSwitches,
    target_triple: &str,
) -> Result<ClosureOutcome, EntryError> {
    let request = ClosureRequest {
        entry_path: entry.entry_path.to_string(),
        target: TargetSelector::parse(entry.target).expect("测试里写的目标认得"),
        requested_conditions: entry
            .conditions
            .iter()
            .map(|condition| RequestedCondition::parse(condition).expect("测试里写的 --cfg 认得"))
            .collect(),
        uses_default_features: true,
        target_triple: target_triple.to_string(),
    };
    compute_closure(workspace, &request, switches)
}

/// 闭包算出的键；拒算交 `Err("拒算：…")`（带拒算行）。
pub fn key_of_outcome(
    workspace: &Workspace,
    outcome: &ClosureOutcome,
    external: &ExternalKeyInputs,
) -> Result<Sha256Digest, String> {
    reuse_key(workspace, outcome, external)
        .map_err(|refusals| format!("拒算：{}", refusal_lines(workspace, &refusals).join(" | ")))
}

/// 入口的键（本机三元组、不带工具链与依赖行）；入口找不到交 `Err("入口：…")`，拒算交 `Err("拒算：…")`（带拒算行）。
pub fn key_of(
    workspace: &Workspace,
    entry: &EntrySpecification,
    switches: &RuleSwitches,
) -> Result<Sha256Digest, String> {
    let outcome = closure_with_triple(workspace, entry, switches, HOST_TARGET_TRIPLE)
        .map_err(|error| format!("入口：{error}"))?;
    key_of_outcome(workspace, &outcome, &ExternalKeyInputs::default())
}
