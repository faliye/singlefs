//! E207（严格判据下的扫类步数）的推的那一路：纯算术，不 `use` 任何 `crates/` 里的东西（跑前登记
//! `research/prompts/e207-preregistration.md` 第五节「装置写在哪」第 3 条）。
//!
//! 做的事：读量的那一路打出的 `E207RESULT` 产物与 E206（发布固定点轮数的代价）的 p2a 产物（只读，回扫），按第一节读法 1–9
//! 算每次操作的 B8 步数（E203（准入之前扫类的步代价）登记第五节 B8 一表的本地式子）、外生的 `user_units`、量到的总量 x，
//! 逐格报 Q1–Q10、控制臂 `C1`、`C2`、`C3`、尖峰旗、节点年龄与首次合并、失败条款、作废条款与停机条款的判读。每个量各占一行，不合取。
//!
//! 用法：
//!   `e207-strict-scan-steps report --segment first|second|third|fourth|hundred_units_per_file_repeat|reconciliation_split <E207 产物>… [--retro <E206 p2a 产物>…] [--previous <第一段产物>…] [--pin-against <第一段产物>…]`
//!       读产物，每个量各占一行，末尾打 `name=verdict` 行；回扫只在给了 `--retro` 时做
//!   `e207-strict-scan-steps old-criterion-pins <E206 p2a 产物>…`  第七节 ② 第 1 条：照旧判据重算 `q8_first_publish` 的 files 轴与 `q8_peak` 的 data_units 轴两行
//! 退出码：0 读完；1 第七节 ② 第 1 条的两行与登记抄下的不逐字符相同；2 用法错或产物读不了。读不到的量报 `unread`，不当 0。
//! 单测钉第七节 ③ 的绝对值：`cargo test --release --bin e207-strict-scan-steps`。

#![allow(dead_code, reason = "模型的库函数一部分只有单测与报告的某一支用")]

use std::collections::{BTreeMap, BTreeSet};

mod cell_reports_and_probes;
mod model_arithmetic_and_by_delta_rows;
mod readings_judgement_and_operations;
mod series_and_plain_comparison_reports;

use self::cell_reports_and_probes::*;
use self::model_arithmetic_and_by_delta_rows::*;
use self::readings_judgement_and_operations::*;
use self::series_and_plain_comparison_reports::*;

/// 历史用例与紧凑盘不是这个 bin 的目标（它们拷进副本、当验证包的测试编，见入口 `e207-strict-scan-steps-run.sh`）；当文本读进来，让门禁 crate-sources-reachable 认它们有着落。
const _HISTORIES_PAYLOAD: &str = include_str!("../e207_strict_scan_steps_histories.rs");
const _COMPACT_DEVICE_PAYLOAD: &str = include_str!("../e207_strict_scan_steps_compact_device.rs");

/// 分配记录树一片叶罩多少槽（`crates/singlefs-format/src/lib.rs:163`）。
const LEAF_SLOTS: u64 = 812;
/// 内部节点扇出（`crates/singlefs-format/src/lib.rs:169`）。
const INTERNAL_FANOUT: u64 = 169;
/// 固定点上界（`crates/singlefs-core/src/transaction.rs:14111` 的 `ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`）。
const SETTLING_ROUNDS_LIMIT: u64 = 64;
/// 两块盘（第五节参数 `e142_parameters(512, 512)`）。
const DISKS: u64 = 2;
/// 单元区之前的槽数（6 MiB 环，`common_admission_pool` 同 E203）。
const UNIT_AREA_START_SLOT: u64 = 1408;
/// inode 叶容器每片记录数（E203 模型同一个常量）。
const INODE_LEAF_RECORDS: u64 = 233;
/// 每段槽数（E203 模型同一个常量）。
const SEGMENT_SLOTS: u64 = 64;
/// 读法第 8 条的尖峰旗：final_nodes ≥ max(28, 4 × 同点同状态量窗全部操作 final_nodes 的中位数)。
const SPIKE_FLOOR: u64 = 28;
const SPIKE_MULTIPLE: u64 = 4;
/// Q7 的 H1：新并进（顺着释放链拉进来的）占 final_nodes 至少 80%，年龄中位数至少 100 个 settle_call。
const RELEASE_CHAIN_PULLED_SHARE_AT_LEAST: f64 = 0.8;
const RELEASE_CHAIN_AGE_MEDIAN_AT_LEAST: u64 = 100;
/// 控制臂 `C1`、`C2` 每条记录几步（第五节阳性对照表）。
const SCAN_EVERY_PUBLISH_STEPS_PER_RECORD: u64 = 1;
const SCAN_EVERY_PUBLISH_FOUR_TIMES_STEPS_PER_RECORD: u64 = 4;
/// Q9 的倍数线（正推一路的取法，第四节第 6 条）。
const THEN_BIG_RATIO_AT_MOST: f64 = 2.0;
/// 第七节 ② 第 4 条：声明的单元差与 `mapped_units` 差之比的区间。
const DECLARED_TO_MAPPED_AT_LEAST: f64 = 0.5;
const DECLARED_TO_MAPPED_AT_MOST: f64 = 2.0;
/// 第八节：改动占总量超过 5% 的点在「远小于总量」上不计。
const FAR_BELOW_THE_TOTAL_AT_MOST: f64 = 0.05;
/// 阴性对照：没有普通分配的发布轮数上界（第五节）。
const NEGATIVE_CONTROL_ROUNDS_AT_MOST: u64 = 4;
/// `user_units_equal_declared` 档的 K 下限（读法第 7 条）。
const LARGE_CHANGE_UNITS_AT_LEAST: u64 = 100;
/// F8：每个新点每个状态里「第一次碰、仍是 K 个单元」的 W2 操作至少几个。
const FIRST_TOUCHES_AT_LEAST: usize = 5;
/// 第七节 ② 第 3 条的尖峰复现钉（`files_1000_units_100` 同一次挂载 W1 第 1、2 次）。
const SPIKE_PIN_FIRST: [(&str, u64); 5] =
    [("rounds", 3), ("final_roles", 452), ("final_nodes", 433), ("round_work", 917), ("closure_nodes_added", 410)];
const SPIKE_PIN_STATE: [(&str, u64); 2] = [("mapped_units", 101_581), ("allocation_records", 238_324)];
const SPIKE_PIN_SECOND: [(&str, u64); 4] = [("rounds", 2), ("final_roles", 20), ("final_nodes", 7), ("round_work", 32)];
/// 第七节 ② 第 6 条：第四段历史末 `mapped_units` 的区间。
const BIG_POINT_MAPPED_UNITS: [(&str, u64, u64); 3] = [
    ("files_1000_units_1000", 1_000_000, 1_050_000),
    ("files_100_units_1000_unit_area_2_pow_21", 100_000, 110_000),
    ("files_10_units_1000_unit_area_2_pow_21", 10_000, 12_000),
];
/// 第七节 ② 第 1 条：登记抄下的两行（`research/results/e206-fixed-point-cost-report-2026-11-09.out:695`、`:706`）。
const OLD_CRITERION_FIRST_PUBLISH_FILES: &str =
    "name=q8_first_publish arm=p2a axis=files low_end=2102 high_end=15440 alpha=0.289 band=between local_slopes=0.045,0.295,0.526";
const OLD_CRITERION_PEAK_DATA_UNITS: &str =
    "name=q8_peak arm=p2a axis=data_units low_end=3763 high_end=3231038 alpha=0.978 band=grows local_slopes=0.660,1.019,1.255";

// ---- 增长阶与判定带（E206 模型同名函数，抄来）----
const AXIS_LOW: f64 = 10.0;
const AXIS_HIGH: f64 = 10_000.0;

fn report(segment: &str, products: &[String], retro_products: &[String], previous_products: &[String], pin_products: &[String]) -> Result<bool, String> {
    let items = read_items(products)?;
    let retro_items = read_items(retro_products)?;
    let mut out = Lines::new();
    let pins_match = if matches!(segment, "hundred_units_per_file_repeat" | "reconciliation_split") {
        // 这两段的回扫（E206 的三个 p2a 产物）只给对账分辨用，不进通用的轴上回扫与旧判据钉（那要十六份）
        report_items(segment, &items, &[], &mut out);
        report_new_segment(segment, &items, &read_items(previous_products)?, &read_items(pin_products)?, &retro_items, &mut out)
    } else {
        report_items(segment, &items, &retro_items, &mut out)
    };
    for line in &out {
        println!("{line}");
    }
    Ok(pins_match)
}

fn usage() -> ! {
    eprintln!("用法：e207-strict-scan-steps report --segment first|second|third|fourth <E207 产物>… [--retro <E206 p2a 产物>…]");
    eprintln!("      e207-strict-scan-steps report --segment hundred_units_per_file_repeat <复量段五项产物> <第一段 files_1000_units_100 产物> --pin-against <第一段 files_100_units_100 产物>");
    eprintln!("      e207-strict-scan-steps report --segment reconciliation_split <四项新产物> --previous <第一段 base、files_1000、files_10000 的 head 产物> --retro <E206 的 base、files_1000、files_10000 p2a 产物>");
    eprintln!("      e207-strict-scan-steps old-criterion-pins <E206 p2a 产物>…");
    std::process::exit(2);
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match arguments.first().map(String::as_str) {
        Some("report") => {
            let (Some("--segment"), Some(segment)) = (arguments.get(1).map(String::as_str), arguments.get(2)) else { usage() };
            if !["first", "second", "third", "fourth", "hundred_units_per_file_repeat", "reconciliation_split"].contains(&segment.as_str()) {
                usage();
            }
            // 位置参数是产物；`--retro`、`--previous`、`--pin-against` 各自收后面跟着的路径
            let mut lists: BTreeMap<&str, Vec<String>> = BTreeMap::new();
            let mut current = "products";
            for argument in &arguments[3..] {
                match argument.as_str() {
                    "--retro" => current = "retro",
                    "--previous" => current = "previous",
                    "--pin-against" => current = "pin_against",
                    path => lists.entry(current).or_default().push(path.to_string()),
                }
            }
            let list = |name: &str| lists.get(name).cloned().unwrap_or_default();
            report(segment, &list("products"), &list("retro"), &list("previous"), &list("pin_against"))
        }
        Some("old-criterion-pins") => arguments[1..]
            .iter()
            .map(|path| read_item(path))
            .collect::<Result<Vec<_>, _>>()
            .map(|items| {
                let mut out = Lines::new();
                let same = old_criterion_pins(&items, &mut out);
                for line in &out {
                    println!("{line}");
                }
                same
            }),
        _ => usage(),
    };
    match outcome {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(problem) => {
            eprintln!("{problem}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests;
