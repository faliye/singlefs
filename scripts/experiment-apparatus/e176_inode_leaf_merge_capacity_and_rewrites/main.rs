//! E176（inode 树叶合并三候选的容量与写代价）：跑前登记 `research/prompts/e176-preregistration.md`。
//!
//! 独立手写模型，不与 `crates/` 共用代码（`implementation-first.md` 第 4 条）。三条臂：
//! M1 只删空即摘（`crates/` 今天）、M2 只做左吸收右（相邻两片合起来装得下就并）、M3 左吸收右加左从右借。
//! M1 那一臂另由 `e176_inode_leaf_merge_crates_oracle.rs`（特性 `e176-crates-oracle`）逐步对拍 `crates/` 的两个纯函数；
//! 那个 bin 以 `#[path]` 把这份文件当模块引进去，所以对拍 bin 用到的项都写成 `pub`。
//!
//! 用法：`e176-inode-leaf-merge-capacity-and-rewrites segment-one`（第一段）或 `segment-two`（第二段，登记第五节与修订 3）；线程数读 `SINGLEFS_THREAD_CAP`（`scripts/hosts/capped.sh` 设），没设取 1。
//! 产物是 `E7RESULT` 行，收尾 `E7RESULT name=done emitted=N`（N 把 config 行与 done 行都算进去）。
//! 确定性模型：跑两遍逐字节相同只说明没有隐藏状态，不是统计上稳定。

// admission: always 每次照此刻的仓与自己的常量现算，上一次跑出的产物不替这一次作保
// run-condition: command python3
use std::collections::BTreeSet;
use std::fmt::Write as _;

mod arms_trajectories_and_windows;
mod leaf_containers_and_the_tree;
mod operations_and_workloads;
mod output_lines_and_sensitivity;
mod segment_outputs;
mod sequence_runs_and_anchor_readings;
mod streaming_sha256;
mod thresholds_and_verdicts;

pub use self::arms_trajectories_and_windows::*;
pub use self::leaf_containers_and_the_tree::*;
pub use self::operations_and_workloads::*;
pub use self::output_lines_and_sensitivity::*;
pub use self::segment_outputs::*;
pub use self::sequence_runs_and_anchor_readings::*;
pub use self::streaming_sha256::*;
pub use self::thresholds_and_verdicts::*;

/// 一片叶至多几条记录：D8（核心索引结构） 已定项 6「一叶 233 条 140 字节记录」。
/// 值抄自 `crates/singlefs-format/src/lib.rs:110` 的 `INODE_LEAF_RECORDS`，对拍 bin 拿它回比 `crates/`。
pub const LEAF_RECORD_CAPACITY: usize = 233;
/// 一个码 2 根至多几条叶条目：D8（核心索引结构） 已定项 6「扇出 135」「⌊(16384 − 131) / 120⌋ = 135」。
/// 值抄自 `crates/singlefs-core/src/inode_tree.rs:109` 的 `leaf_containers_one_root_node_holds()`，对拍 bin 拿它回比 `crates/`。
pub const ROOT_NODE_LEAF_CONTAINER_CAPACITY: usize = 135;
/// 每条序列每条臂最多吃几个操作（跑前登记第六节）：到了还没被拒就报「未撞到」。
pub const OPERATIONS_PER_ARM_AT_MOST: u64 = 20_000_000;
/// 公共窗口里的最后一批少于这个次数，Q2、Q3 两行只报数、标「样本不足」（跑前登记第六节）。
pub const UNLINKS_IN_THE_WINDOW_FOR_A_JUDGEMENT_AT_LEAST: u64 = 100;
/// M3 主取样点的下限 f = ⌈233 ÷ 2⌉ = 117；方向相反的取样点 59 = ⌈233 ÷ 4⌉（跑前登记第五、八节）。
pub const BORROW_FLOOR_MAIN: usize = 117;
pub const BORROW_FLOOR_SMALL: usize = 59;

/// 线程数：`SINGLEFS_THREAD_CAP`（`scripts/hosts/capped.sh` 设），没设取登记的重活默认并行度（`.claude/heavy-thread-default`）；写错就停。
fn thread_count_from_environment() -> usize {
    match std::env::var("SINGLEFS_THREAD_CAP") {
        Ok(text) => text.parse::<usize>().ok().filter(|threads| *threads >= 1).expect("SINGLEFS_THREAD_CAP 要是正整数"),
        Err(_) => default_heavy_threads(include_str!("../../../.claude/heavy-thread-default")),
    }
}

fn main() {
    preflight();
    let mode = std::env::args().nth(1).unwrap_or_default();
    let threads = thread_count_from_environment();
    let (lines, positive_controls_ok) = match mode.as_str() {
        "segment-one" => {
            eprintln!("E176 第一段：{threads} 个线程（线程数不进产物）");
            segment_one_output(threads)
        }
        "segment-two" => {
            eprintln!("E176 第二段：{threads} 个线程（线程数不进产物）");
            segment_two_output(threads)
        }
        _ => {
            eprintln!("用法：e176-inode-leaf-merge-capacity-and-rewrites segment-one | segment-two（线程数读 SINGLEFS_THREAD_CAP）");
            std::process::exit(2);
        }
    };
    let emitted = lines.len() + 1;
    for line in lines {
        println!("E7RESULT {line}");
    }
    println!("E7RESULT name=done emitted={emitted}");
    if !positive_controls_ok {
        eprintln!("阳性对照或锚点有一格对不上（第十一节作废）：见 name=anchor 那几行的 matches=false");
        std::process::exit(1);
    }
}

/// 准入用的仓根：环境变量 SINGLEFS_REPOSITORY_ROOT，没设取当前目录（replay.sh 与实验页的命令都在仓根起）；不取「清单目录/../..」（单元键把它当运行期路径拒算）。
fn repository_root() -> std::path::PathBuf {
    let root = std::env::var_os("SINGLEFS_REPOSITORY_ROOT")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    if !root.join(".claude/singlefs-ai-sop/scripts/preflight.py").is_file() {
        eprintln!("  ✗ {} 不是仓根（下面没有 .claude/singlefs-ai-sop/scripts/preflight.py），判不了准入", root.display());
        eprintln!("  → 怎么办：在仓根起这个装置，或设 SINGLEFS_REPOSITORY_ROOT=<仓根>");
        std::process::exit(78);
    }
    root
}

fn preflight() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let root = repository_root();
    let source = root.join("scripts/experiment-apparatus/e176_inode_leaf_merge_capacity_and_rewrites/main.rs");
    let script = root.join(".claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

include!("../heavy_threads_default/threads.rs");
