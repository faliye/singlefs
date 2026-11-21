//! E206（发布固定点轮数的代价）的计数模型：纯算术，不 `use` 任何 `crates/` 里的东西（跑前登记
//! `research/prompts/e206-preregistration.md` 第五节「装置写在哪」第 2 条）。
//!
//! 做的事：抽象固定点（第七节 ③）、节点上界与增长阶、B8（逐项按差量）的本地式子（E203（准入之前扫类的步代价）登记第五节 B8 一表与修订 7 的 s36）、
//! P3 的式子与常驻内存（第五节）、读量的那一路打出的 `E206RESULT` 行，逐项报 Q1–Q10、失败条款、作废条款与停机条款的判读。
//!
//! 用法：
//!   `e206-fixed-point-cost report <产物>…`                      读产物，每个量各占一行，打 `name=verdict`
//!   `e206-fixed-point-cost anchors`                             打第七节 ③ 与几何锚点（模型自己算的）
//!   `e206-fixed-point-cost e203-anchors <base 产物> <files_10000 产物>`  第七节 ② 第 3 条：读 E203 那两份计数产物的同一次挂载覆盖写第 1、2 轮，比 2102、2098、152344、4226
//! 退出码：0 读完；1 第七节 ② 第 3 条对不上；2 用法错或产物读不了。读不到的量报 `unread`，不当 0。
//! 单测钉第七节锚点的绝对值：`cargo test --release --bin e206-fixed-point-cost`。

#![allow(dead_code, reason = "模型的库函数一部分只有单测与报告的某一支用")]

use std::collections::{BTreeMap, BTreeSet};

mod alpha_bands_and_settle_model;
mod by_delta_rows_and_pricing_inputs;
mod judgement_formulas;
mod reading_the_products;
mod report_lines;
#[cfg(test)]
mod tests;

use self::alpha_bands_and_settle_model::*;
use self::by_delta_rows_and_pricing_inputs::*;
use self::judgement_formulas::*;
use self::reading_the_products::*;
use self::report_lines::*;

/// 历史用例不是这个 bin 的目标（它拷进副本、当验证包的测试编，见入口 `e206-fixed-point-cost-run.sh`）；当文本读进来，让门禁 crate-sources-reachable 认它有着落。
const _HISTORIES_PAYLOAD: &str = include_str!("../e206_fixed_point_cost_histories.rs");

/// 分配记录树一片叶罩多少槽（`crates/singlefs-format/src/lib.rs:163`）。
const LEAF_SLOTS: u64 = 812;
/// 内部节点扇出（`crates/singlefs-format/src/lib.rs:169`）。
const INTERNAL_FANOUT: u64 = 169;
/// 固定点上界（`crates/singlefs-core/src/transaction.rs:13466`）。
const SETTLING_ROUNDS_LIMIT: usize = 64;
/// 两块盘（第五节参数 `e142_parameters(512, 512)`）。
const DISKS: u64 = 2;
/// 单元区之前的槽数（6 MiB 环，`common_admission_pool` 同 E203）。
const UNIT_AREA_START_SLOT: u64 = 1408;
/// inode 叶容器每片记录数（E203 模型同一个常量）。
const INODE_LEAF_RECORDS: u64 = 233;
/// 每段槽数（E203 模型同一个常量）。
const SEGMENT_SLOTS: u64 = 64;
/// Q1、Q4 的门槛（第六节表后：8 = 2 × 源码注释「三四轮」的 4）。
const MANY_ROUNDS_ABOVE: u64 = 8;
/// Q4 有效轮数门槛。
const EFFECTIVE_ROUNDS_AT_MOST: f64 = 4.0;
/// Q6 写放大门槛（第六节表后，推的）。
const WRITE_AMPLIFICATION_AT_MOST: f64 = 1.25;
/// Q7 前提比门槛。
const PREMISE_RATIO_AT_MOST: f64 = 0.5;
/// Q10 判「释放链解释得了」的比例与容差。
const CHAIN_EXPLAINS_AT_LEAST: f64 = 0.9;
const CHAIN_TOLERANCE: i64 = 2;
/// 增长阶的轴端点（读法第 6 条）。
const AXIS_LOW: f64 = 10.0;
const AXIS_HIGH: f64 = 10_000.0;

/// 第七节 ② 第 3 条的四个数：同一次挂载覆盖写第 1、2 轮，`base`、`files_10000`。
const E203_ANCHORS: [(u64, u64, u64); 2] = [(0, 2102, 2098), (1, 152_344, 4226)];

fn e203_anchors(paths: &[String]) -> Result<bool, String> {
    let mut all_match = true;
    for (index, first, second) in E203_ANCHORS {
        let path = &paths[usize::try_from(index).expect("下标")];
        for (round, expected) in [(1, first), (2, second)] {
            let (judgement, unit_area_slots) = e203_judgement(path, round)?;
            let got = by_delta_of_the_judgement(&judgement, unit_area_slots)
                .map_err(|gaps| format!("{path} 第 {round} 轮读不到：{gaps:?}"))?;
            println!("name=e203_anchor product={path} round={round} total={got} expected={expected} matches={}", got == expected);
            all_match &= got == expected;
        }
    }
    Ok(all_match)
}

/// `anchors` 子命令：第七节 ③ 模型自己算的。
fn print_anchors() {
    for leaves in [1, 5, 20, 70] {
        for (arm, name) in [(ModelArm::Today, "p1"), (ModelArm::Closure, "p2a")] {
            let outcome = chain_outcome(leaves, 3, arm);
            println!("name=chain leaves={leaves} ordinary_roles=3 arm={name} rounds={:?} W={} per_round={:?} sum_of_increments={}", outcome.rounds, outcome.round_work(), outcome.per_round, outcome.sum_of_increments());
        }
    }
    for (arm, name) in [(ModelArm::Today, "p1"), (ModelArm::Closure, "p2a")] {
        let outcome = crossing_outcome(arm);
        println!(
            "name=crossing arm={name} rounds={:?} W={} per_round={:?}",
            outcome.rounds,
            outcome.round_work(),
            outcome.per_round
        );
    }
    for exponent in [14, 17, 18, 19, 20, 24] {
        let (leaves, level, nodes) = tree_geometry(1 << exponent);
        println!("name=geometry unit_area_exponent={exponent} leaves_per_disk={leaves} root_level={level} nodes_at_most={nodes}");
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let code = match arguments.first().map(String::as_str) {
        Some("anchors") => {
            print_anchors();
            0
        }
        Some("report") if arguments.len() > 1 => match report(&arguments[1..]) {
            Ok(()) => 0,
            Err(problem) => {
                eprintln!("  ✗ {problem}\n     → 怎么办：把产物路径改对，或先跑入口把产物落下来");
                2
            }
        },
        Some("e203-anchors") if arguments.len() == 3 => match e203_anchors(&arguments[1..]) {
            Ok(true) => 0,
            Ok(false) => 1,
            Err(problem) => {
                eprintln!("  ✗ {problem}\n     → 怎么办：两份产物从 git 取（git show 84b4ae23f:research/results/<名>）");
                2
            }
        },
        _ => {
            eprintln!("用法：e206-fixed-point-cost report <产物>… | anchors | e203-anchors <base 产物> <files_10000 产物>");
            2
        }
    };
    std::process::exit(code);
}
