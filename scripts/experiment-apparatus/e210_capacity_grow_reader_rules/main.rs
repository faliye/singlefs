//! E210（盘变大改容量表的读者规则四臂）：独立手写的计数模型，不与 `crates/` 共用代码（不是入库装置）。
//!
//! 跑前登记 `research/prompts/e210-preregistration.md`；岔路单 `research/prompts/m4-capacity-grow-r1-forks.md` 三行。
//! 命令行参数选一段产物：
//! - `write-counts-and-root-level`：岔路单行 2（每条读者规则一次变大的可写挂载写几次系统配置槽、几道屏障）与行 3（变大跨过分配记录树
//!   根层级边界时，一点不记与记到边界为止两种做法下用得上的整槽数）。
//! - `reader-rules-on-histories`：岔路单行 1（四条读者规则加基线在同一组历史上的漏放行数与误拒数，各带分母）。
//!
//! 臂的名字（登记里的甲、乙、丙、丁与基线臂）：
//! - 甲 `ChainThenLowest`：全部有效槽的表成链就取逐项最小，不成链整池拒。
//! - 乙 `HighestTable`：生效表取各有效槽逐项最大。
//! - 丙 `OldTableWhileIntentMarkerIsOnTheDisk`：先写一轮带意图标记的槽、再写两轮新表；盘面上有带标记的槽取其中逐项最小，没有取逐项最大。
//! - 丁 `LowestToReadHighestToProbeWithAbandon`：生效表逐项最小，比探测值用逐项最大，未写齐的变大可以写回旧表放弃。
//! - 基线臂 `OnlyIfEveryTableIsEqual`：表全相等才认，否则整池拒。
//!
//! 本地常量抄自 `crates/singlefs-format/src/lib.rs`（行号写在各常量旁，登记第十一节停机条款 1 逐个核）；写序、世代号与屏障口径照
//! `crates/singlefs-core/src/transaction.rs` 的取号写两轮。模型里没有随机源、时钟与并发，同一份装置跑两次标准输出逐字节相同。

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use e7_index_bench::Emitter;

mod description_and_report_lines;
mod history_sets_and_canonical_classes;
mod reader_decisions_and_readings;
mod registered_shapes_and_user_actions;
mod slot_boards_and_write_plans;
#[cfg(test)]
mod tests;
mod write_counts_and_root_levels;

use self::description_and_report_lines::*;
use self::history_sets_and_canonical_classes::*;
use self::reader_decisions_and_readings::*;
use self::registered_shapes_and_user_actions::*;
use self::slot_boards_and_write_plans::*;
use self::write_counts_and_root_levels::*;

/// 线程数只影响挂钟、不影响任何一行输出（并行单位的结果按单元次序合并）。`SINGLEFS_THREAD_CAP` 没设取登记的重活默认并行度。
fn worker_thread_count() -> usize {
    default_heavy_threads(include_str!("../../../.claude/heavy-thread-default"))
}

/// `crates/singlefs-format/src/lib.rs:14` 的 `SLOT_BYTES`。
const SLOT_BYTES: u64 = 16384;
/// 同文件 `:163` 的 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
/// 同文件 `:169` 的 `ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`。
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// 同文件 `:279` 的 `SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE`（那边是 u64，这里当数组下标用，取 usize）。
const SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: usize = 2;
/// 同文件 `:286` 的 `SYSTEM_CONFIGURATION_SLOT_BYTES`。
const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
/// 同文件 `:321` 的 `UNIT_AREA_START_SLOT`。
const UNIT_AREA_START_SLOT: u64 = 50176;
/// 崩溃点重放的扇区取 512 字节（D13（验证路线） 已定项 4）：长于一个扇区的原地覆写多取第三态「新旧都读不出」。
const CRASH_REPLAY_SECTOR_BYTES: u64 = 512;
/// `crates/singlefs-core/src/transaction.rs:1313` 的 `ACQUISITION_WRITE_ROUNDS` 有两个元素：取号写两轮。
const ACQUISITION_WRITE_ROUNDS: usize = 2;

/// 池里两块盘（设备身份 0、1）。
const DISK_COUNT: usize = 2;
/// 「只有一块盘缩小或变大」的用户动作里动的那块盘的设备身份。
const DEVICE_THAT_CHANGES_ALONE: usize = 0;
/// 用户动作「再变大」每盘加的整槽数。
const GROW_STEP_SLOTS: u64 = 100;

/// 逐盘容量表：按设备身份排的两项，每盘一项，单位是整槽（`SLOT_BYTES`）。
type Table = [u64; DISK_COUNT];
/// 一个盘的集合，下标是设备身份。
type DiskSet = [bool; DISK_COUNT];
const NOBODY_NAMED: DiskSet = [false; DISK_COUNT];

fn main() {
    let threads = worker_thread_count();
    let lines = match std::env::args().nth(1).as_deref() {
        Some("write-counts-and-root-level") => write_counts_and_root_level_lines(threads),
        Some("reader-rules-on-histories") => reader_rules_on_histories_lines(threads),
        _ => {
            eprintln!("用法：e210-capacity-grow-reader-rules write-counts-and-root-level | reader-rules-on-histories");
            std::process::exit(2);
        }
    };
    let mut emitter = Emitter::new();
    let mut output_text = String::new();
    for line in &lines {
        output_text.push_str(&emitter.emit_raw(line));
        output_text.push('\n');
    }
    output_text.push_str(&emitter.finish());
    println!("{output_text}");
}

include!("../heavy_threads_default/threads.rs");
