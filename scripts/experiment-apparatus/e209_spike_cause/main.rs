//! E209（尖峰成因）的推的那一路：纯算术，不 `use` 任何 `crates/` 里的东西（跑前登记 `research/prompts/e209-preregistration.md` 第五节「装置写在哪」第 3 条）。
//!
//! 做的事：读量的那一路打出的 `E209RESULT` 产物（每个项一份：一个点上放同一段历史、再做一个脚本），算旗上操作、节点的新鲜度、
//! 与实现不共用代码的释放链闭包、问题单五行各自的判器、旗线与几何的敏感性、行表差与它落在 `crates/` 的哪个函数，逐量各占一行报；
//! 合成臂（答案已知的手造日志）与钉绝对值的断言写在单测里，不进产物。
//!
//! 用法：
//!   `e209-spike-cause report [--crates-root <仓根>] [--out <新文件>] <产物文件>…`   每个量各占一行，末尾打 `name=verdict` 行；给了 `--crates-root` 才查行表各行在 `crates/` 里的函数在第几行
//!   `e209-spike-cause list-items`                                   登记的项名与所属段（入口自证核它与历史用例、入口的名单同一份）
//! 退出码：0 读完；2 用法错或产物读不了。读不到的量报 `unread`，不当 0。
//! 单测钉合成臂与第七节 ③ 的绝对值：`cargo test --release --bin e209-spike-cause`。
//! 弄坏开关：无（变异表 `scripts/mutation/tables/e209_spike_cause.tsv` 逐条改源码，单测要红）。

#![allow(dead_code, reason = "模型的库函数一部分只有单测与报告的某一支用")]

#[path = "../e209_spike_cause_scripts.rs"]
mod scripts;

use std::collections::{BTreeMap, BTreeSet};

mod by_delta_rows_and_product_fields;
mod nodes_release_chains_and_spike_lines;
mod operation_views;
#[cfg(test)]
mod pins;
mod reading_the_items;
mod report_lines;
mod rule_judgements;
#[cfg(test)]
mod tests;
mod world_checks_and_set_comparisons;

use self::by_delta_rows_and_product_fields::*;
use self::nodes_release_chains_and_spike_lines::*;
use self::operation_views::*;
use self::reading_the_items::*;
use self::report_lines::*;
use self::rule_judgements::*;
use self::world_checks_and_set_comparisons::*;

/// 历史用例不是这个 bin 的目标（它拷进副本、当验证包的测试编，见入口 `e209-spike-cause-run.sh`）；当文本读进来，让门禁 crate-sources-reachable 认它有着落。
const _HISTORIES_PAYLOAD: &str = include_str!("../e209_spike_cause_histories.rs");

/// 分配记录树一片叶罩多少槽（`crates/singlefs-format/src/lib.rs` 的 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`）。
const LEAF_SLOTS: u64 = 812;
/// 内部节点扇出（同文件的 `ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`）。
const INTERNAL_FANOUT: u64 = 169;
/// 固定点上界（`crates/singlefs-core/src/transaction.rs` 的 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`）。
const SETTLING_ROUNDS_LIMIT: u64 = 64;
/// 旗线下限与倍数（E207 装置 `e207_strict_scan_steps.rs` 的 `SPIKE_FLOOR`、`SPIKE_MULTIPLE`；读法 1）。
const SPIKE_FLOOR: u64 = 28;
const SPIKE_MULTIPLE: u64 = 4;
/// 旗线灵敏度：下限 L 取这三个，主读第二个（读法 1）。
const FLOOR_LINES: [u64; 3] = [14, 28, 56];
const MAIN_FLOOR_LINE_POSITION: usize = 1;
const LOW_FLOOR_LINE_POSITION: usize = 0;
/// 行一的检验力：旗上操作与不旗操作各至少这么多个（失败条款：检验力不足）。
const FLAGGED_OPERATIONS_AT_LEAST: usize = 3;
/// 行四：挂载带出占比线（第六节 Q8、Q9）与单一一步占比线。
const MOUNT_SHARE_CARRIED_AT_LEAST: f64 = 0.9;
const MOUNT_SHARE_NOT_CARRIED_BELOW: f64 = 0.5;
const ONE_STEP_SHARE_AT_LEAST: f64 = 0.9;
/// 行五：前三大行占比线、行数、噪声倍数。
const TOP_ROWS_SHARE_AT_LEAST: f64 = 0.9;
const TOP_ROW_COUNT: usize = 3;
const NOISE_MULTIPLE: u64 = 10;
/// 几何敏感性里每个训练脚本（第六节「训练集」）。
const TRAINING_SCRIPTS: [&str; 4] = ["walk_forward", "walk_backward", "first_repeated", "create_then_first"];
const MANIPULATION_SCRIPTS: [&str; 3] = ["walk_forward", "walk_backward", "walk_from_middle"];

fn list_items() -> Vec<String> {
    scripts::registered_items().into_iter().map(|(name, segment)| format!("{name} {segment}")).collect()
}

/// `report` 的参数：`--crates-root <仓根>`、`--out <新文件>`（报告行同时写进这个文件）、其余是产物文件。
fn split_report_arguments(arguments: &[String]) -> (Option<String>, Option<String>, Vec<String>) {
    let mut crates_root: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut paths = Vec::new();
    let mut rest = arguments.iter();
    while let Some(argument) = rest.next() {
        if argument == "--crates-root" {
            crates_root = rest.next().cloned();
        } else if argument == "--out" {
            output_path = rest.next().cloned();
        } else {
            paths.push(argument.clone());
        }
    }
    (crates_root, output_path, paths)
}

/// 报告行写进一个排他新建的文件：同名文件已在就拒（产物不覆盖）。
fn write_new_report_file(path: &str, lines: &[String]) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("报告文件 {path} 不能新建：{error}"))?;
    for line in lines {
        writeln!(file, "{line}").map_err(|error| format!("写 {path} 失败：{error}"))?;
    }
    Ok(())
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("list-items") => {
            for line in list_items() {
                println!("{line}");
            }
        }
        Some("report") => {
            let (crates_root, output_path, paths) = split_report_arguments(&arguments[1..]);
            let mut items = Vec::new();
            for path in &paths {
                match read_item(path) {
                    Ok(item) => items.push(item),
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                }
            }
            let lines: Vec<String> = report(&items, crates_root.as_deref()).into_iter().map(|line| format!("E209RESULT {line}")).collect();
            for line in &lines {
                println!("{line}");
            }
            if let Some(path) = output_path {
                if let Err(error) = write_new_report_file(&path, &lines) {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            }
        }
        _ => {
            eprintln!("用法：e209-spike-cause report [--crates-root <仓根>] [--out <新文件>] <产物文件>… | list-items");
            std::process::exit(2);
        }
    }
}
