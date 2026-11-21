//! E173（上限卡在读坏槽的三种处置）：准入抬 F 的上限遇到挂载那一刻读坏或读回全零的根槽时，四条 Q1 臂（照今天、推空B、推空W、不并）与
//! 两条 Q2 臂（全零不并、全零并）各付出什么代价：被拒次数、F 越过的非空根数、推空用掉的空间、回退候选数。
//!
//! 跑前登记 `research/prompts/e173-preregistration.md` 写死了这个装置：**独立手写的计数模型，不与 `crates/` 共用代码**（不是入库装置）。
//! 模型的口径只来自登记第二节抄下的 D16（发布语义）已定项 1、8、9 与 D22「每区槽数 S」那一行，以及登记第三节与第五节写明的「照今天」读法
//! （冻结树 `crates/singlefs-core/src/mount.rs`、`allocator.rs`、`mounted_session.rs` 的行为，只读了出处行，没有共用一行代码）：
//!
//! - 根环：R = 3 个区域、每区 S 个槽，txg t 落区域 t mod 3、区内槽 (t ÷ 3) mod S；区域归属 0 / 1 / 0。
//! - 空间：计数单位是块；覆盖写新占 d + c 块、放掉 d + c 块，空发布新占 c 块、放掉 c 块，放掉的块带释放代 = 这次发布的 txg，进待回收；
//!   每记一条根之后待回收里释放代不高于门槛 max(F_生效, 环里最旧有效根) 的转成空闲；抬 F 那一串回收出来的扣到这一串做完才放开。
//! - 准入：发布要求 空闲 − P保留 ≥ d + c；空发布（抬 F 那一串、推空）只要求 空闲 ≥ c；一次准入最多 B = 4 + 2 k_tol = 8 次发布。
//!
//! 子命令：`checks` 跑登记第十一节的对拍与第七节的锚点、第五节的阳性对照；`first` 是第一段（补 Q1）；`second` 是第二段（补 Q2）；
//! `checkpoint-cost-reserve` 是第一次重跑的第二段（重跑登记 `research/prompts/e173-r1-prereg.md`：保留池按第一段在真盘面上量出的 κ_G 现算，
//! 代码读法 P = κ_G、条款读法 P = 10 + 7 κ_G，c 取同一几何的 c实_G；另跑保留池改回 10 + 7 c实 的判别力对照）。前三个模式的输出不随它变（重跑登记停机条款 S1）。

// admission: always 独立手写的计数模型，每次现算；不读 crates/，只读自己的常量
// run-condition: command python3
use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

mod ceiling_readings_and_mounted_pool;
mod checkpoint_cost_reserve_mode;
mod group_runs_rendering_and_judgments;
mod mount_runs_metrics_and_groups;
mod ring_geometry_histories_and_faults;
mod session_requests_and_raises;
mod zero_slot_judgments_and_anchors;

use self::ceiling_readings_and_mounted_pool::*;
use self::checkpoint_cost_reserve_mode::*;
use self::group_runs_rendering_and_judgments::*;
use self::mount_runs_metrics_and_groups::*;
use self::ring_geometry_histories_and_faults::*;
use self::session_requests_and_raises::*;
use self::zero_slot_judgments_and_anchors::*;

// ── 条款与结构常量（出处：登记第二节抄的 D16 已定项 1、8、D22「每区槽数 S」那一行；`crates/singlefs-core/src/root_ring.rs:109` 落点公式） ──

/// 根环区域数 R（D16 已定项 8「至多 R = 3 次」，登记 A5）。
const REGIONS_IN_THE_ROOT_RING: u64 = 3;
/// 三个区域的设备归属 0 / 1 / 0（D16 已定项 8）。
const DEVICE_OF_EACH_REGION: [usize; 3] = [0, 1, 0];
const DEVICES_IN_THE_POOL: usize = 2;
/// k_tol = 2（D16 已定项 1 射程「按 k_tol」，「准入」那一行 B = 4 + 2 k_tol ⇒ 8，登记 A1）。
const TOLERATED_ROOT_SLOT_WRITE_FAILURES: usize = 2;
const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 4 + 2 * TOLERATED_ROOT_SLOT_WRITE_FAILURES;
/// 上限取「第 4 新的非空有效根」：最近 4 个不同状态（D16 已定项 1 首句）。
const NEWEST_DISTINCT_STATES_THE_CEILING_KEEPS: usize = 4;
/// 一个文件 64 块，覆盖写每次换 8 块（登记第五节「历史族」）。
const DATA_BLOCKS_PER_OVERWRITE: i64 = 8;
/// S 的取样点（登记第八节）：格式范围 4..16 的两端与 mkfs 取值 8（D22「每区槽数 S」那一行，`crates/singlefs-format/src/lib.rs:270`、`275`、`279`）。
const SLOTS_PER_REGION_CHOICES: [u64; 3] = [4, 8, 16];
/// c_max 的取样点（登记第八节：条款举例的两档）。
const METADATA_BLOCKS_CHOICES: [i64; 2] = [5, 10];
/// 没有文件的那一版（mkfs、写行、暖机）的状态号。
const STATE_WITHOUT_FILE: u32 = 0;
/// 池级读坏的「持久到被测挂载结束」的取样点上限：被测挂载的请求数 J = 2 × 3S + 8（登记第一节）。
const REQUESTS_PER_MOUNT_BASE: u64 = 8;

include!("../heavy_threads_default/threads.rs");

/// 线程数只影响挂钟、不影响任何一行输出（每组各算各的，结果按组的次序收）。`SINGLEFS_THREAD_CAP` 没设取登记的重活默认并行度（`.claude/heavy-thread-default`）。
fn worker_thread_count() -> usize {
    default_heavy_threads(include_str!("../../../.claude/heavy-thread-default"))
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
    let source = root.join("scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions/main.rs");
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

fn main() {
    preflight();
    let mut emitter = Emitter::new();
    let mut tally = RunTally::default();
    match std::env::args().nth(1).as_deref() {
        Some("checks") => run_the_checks(&mut emitter, &mut tally),
        Some("first") => run_the_first_segment(&mut emitter, &mut tally),
        Some("second") => run_the_second_segment(&mut emitter, &mut tally),
        Some("checkpoint-cost-reserve") => run_the_checkpoint_cost_reserve_mode(&mut emitter, &mut tally),
        Some(other) => {
            eprintln!("不认识的子命令：{other}（可用：checks、first、second、checkpoint-cost-reserve）");
            std::process::exit(2);
        }
        None => {
            eprintln!("要一个子命令：checks、first、second、checkpoint-cost-reserve");
            std::process::exit(2);
        }
    }
    println!("{}", emitter.finish());
    if !tally.halt_conditions.is_empty() {
        std::process::exit(3);
    }
    if !tally.void_conditions.is_empty() {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod pinned_readings;

#[cfg(test)]
mod measurement_readings;

#[cfg(test)]
mod boundary_readings;

#[cfg(test)]
mod checkpoint_cost_reserve_readings;
