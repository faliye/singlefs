//! E203（准入之前扫类的步代价）的计数模型：纯算术，不 `use` 任何 `crates/` 里的东西（跑前登记
//! `research/prompts/e203-preregistration.md` 第五节「装置写在哪」第 2 条）。
//!
//! 做的事：B8（逐项按差量）与 B4（改射程）两条臂的步数式子、B8 的常驻内存与每次发布的维护、增长阶 α 与三档判法、
//! 用户关心的规模上的外推（Q4）与外推核对（Q5）、阳性与阴性对照、计时包含自检，以及读量的那一路（插了计数的副本）
//! 打出的 `E203RESULT` 行，逐段逐行算 B1 的数并与站点读数对账（登记第七节 ③2）。
//!
//! 报告另做的事：Q6、Q8、Q9 在用户关心规模上的值（与 Q4 同一个外推法，`user_scale` 行）；第七节 ③2 的对账（26 行：16 行吃入口、出口读数的式子
//! 逐格相等，修订 4 那十行吃调用处读的表长，全表扫那一部分逐格相等、走到命中为止那一部分不超过上界；没接的行点名）；
//! 作废条款 8（补满五轮仍不满的覆盖写格整格拿掉并点名）；停机条款 3、4 与失败条款 F1–F6 的判读行。
//!
//! 用法：
//!   `e203-admission-scan-cost report <计数产物>… [--timing <计时产物>…]`  读产物、打 `name=verdict` 与各量的行
//!   `e203-admission-scan-cost anchors`                                  打第七节锚点（模型自己算的，与登记原样比）
//! 单测钉第七节锚点的绝对值：`cargo test --release --bin e203-admission-scan-cost`。

#![allow(dead_code, reason = "模型的库函数一部分只有单测与后续接线的报告用")]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::process::ExitCode;

mod arms_memory_and_scale_reports;
mod bands_and_scan_step_formulas;
mod by_delta_input_and_segment_totals;
mod controls_and_full_report;
mod memory_and_maintenance_models;
mod product_cells_and_axes;
mod reconciliation_of_site_readings;
#[cfg(test)]
mod test_support_for_the_reports;
mod timing_formulas_and_row_reading;
mod timing_rounds_and_report;
mod user_scale_and_extrapolation;

use self::arms_memory_and_scale_reports::*;
use self::bands_and_scan_step_formulas::*;
use self::by_delta_input_and_segment_totals::*;
use self::controls_and_full_report::*;
use self::memory_and_maintenance_models::*;
use self::product_cells_and_axes::*;
use self::reconciliation_of_site_readings::*;
#[cfg(test)]
use self::test_support_for_the_reports::*;
use self::timing_formulas_and_row_reading::*;
use self::timing_rounds_and_report::*;
use self::user_scale_and_extrapolation::*;

// ---- 本地常量（出处是冻结副本的 文件:行，产物头里实现报的值与这里逐个比；对不上走停机条款第 2 条）----
/// 一槽 16 KiB（`crates/singlefs-format/src/lib.rs:14`）。
const SLOT_BYTES: u64 = 16384;
/// 一段 64 槽（`allocator.rs` 的 `CLUSTER_SEGMENT_SLOTS`）。
const SEGMENT_SLOTS: u64 = 64;
/// 6 MiB 环下单元区起点槽（`UnitAreaStart::following_the_journal_ring(6 << 20)`）。
const UNIT_AREA_START_SLOT: u64 = 1408;
/// 第一版固定两块盘。
const DISKS: u64 = 2;
/// inode 叶每片至多 233 条、至多 135 片（`transaction.rs:4205` 那一族的封顶）。
const INODE_LEAF_RECORDS: u64 = 233;
const INODE_LEAVES_AT_MOST: u64 = 135;
/// 墓碑每片至多 815 条、退役 959 条（第四节第 5 条）。
const TOMBSTONE_CONTAINER_RECORDS: u64 = 815;
/// 分配记录树固定点至多 64 轮（`transaction.rs:13141`）。
const SETTLING_ROUNDS_AT_MOST: u64 = 64;
/// 被量的覆盖写把第一个文件换成这么多字节（`common_admission_pool/mod.rs:42`）。
const OVERWRITE_BYTES: u64 = 2999;
/// 每盘 16 TiB 的盘字节数 2⁴⁴ 对应的单元区槽数 2³⁰ − 1408（登记读法第 5 条）。
const USER_SCALE_DEVICE_SLOTS: u64 = 1 << 30;
/// 历史用例源文件（由 e203-admission-scan-cost-run.sh 拷进副本跑）。
const _HISTORIES_PAYLOAD: &str = include_str!("../e203_admission_scan_cost_histories.rs");

fn main() -> ExitCode {
    let command_line: Vec<String> = std::env::args().skip(1).collect();
    match command_line.first().map(String::as_str) {
        Some("report") if command_line.len() >= 2 => run_report(&command_line[1..]),
        Some("anchors") => {
            print!("{}", anchors_lines());
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("用法：e203-admission-scan-cost report <计数产物>… [--timing <计时产物>…] | anchors");
            ExitCode::from(2)
        }
    }
}

/// 线性找走过的步数（登记第七节 ③2 里「找到那一个」也算一步）：答案在第 `position` 个（从 0 起）走 position + 1 步，没找到走完整张表。
fn linear_find_steps(position_of_the_answer: Option<u64>, table_length: u64) -> u64 {
    position_of_the_answer.map_or(table_length, |position| position + 1)
}

#[cfg(test)]
mod tests;
