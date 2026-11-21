//! E167（多在飞发布的崩溃枚举切法）：两个事务各两个单元的小流上，「照旧按根槽写切」与「按事务切」两条臂各自枚举出多少崩溃状态、
//! 覆盖多少不合法前缀、漏不漏跨两个事务的落盘子集。
//!
//! 跑前登记 `research/prompts/e167-preregistration.md` 写死了这个装置：**独立手写计数模型，不与 `crates/` 共用代码**（不是入库装置）。
//! 模型的口径只来自登记第二节抄下的 D13（验证路线）已定项 4（枚举域）、D16（发布语义）已定项 7（单事务写链的次序）与已定项 2 射程那一句：
//!
//! - 崩溃状态按设备切段：一次写只被它自己那块盘上之后的屏障（或那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或
//!   FUA 放行，这一段才关；FUA 只放行它自己那块盘，与那块盘上前面没被屏障隔开的写同段；一个状态 = 前若干段全持久、当前段每次写各取它的几态
//!   任意组合、之后各段全不持久；原地覆写（系统配置槽写）多取第三态「撕裂」。
//! - 真值 T(P) = P 允许的全部发出流上这套枚举域的并；另有一份按设备偏序算的精确可达集 T_exact 只核参照。
//! - 臂 K1（按根槽写切）= 录到的那一条发出流上的枚举域；臂 K2（按事务切）= 两条单事务链各自枚举域的积。
//!
//! 子命令：不带参数跑第一、二、三段全部（一份产物、一个收尾行；第一段停机条款打中就不往下）；段名 `first`、`second`、`third` 各跑一段（指定范围）；
//! `halt` 只跑登记第十一节的停机条款 S1、S2；`s3` 只估规模、不判。认不出的参数退 2。

// admission: always 独立手写的计数模型，每次现算；不读 crates/，只读自己的常量
// run-condition: command python3
use e7_index_bench::Emitter;
use std::collections::{HashSet, VecDeque};

mod covers_and_stream_rendering;
mod geometries_and_write_lists;
mod in_memory_cell_runs;
mod streaming_and_true_cross_runs;
mod streaming_cell_emission;
mod streaming_walks_and_searches;
mod worlds_and_state_truth;

use self::covers_and_stream_rendering::*;
use self::geometries_and_write_lists::*;
use self::in_memory_cell_runs::*;
use self::streaming_and_true_cross_runs::*;
use self::streaming_cell_emission::*;
use self::streaming_walks_and_searches::*;
use self::worlds_and_state_truth::*;

// ── 状态编码：每次写两位，0 没持久、1 持久、2 撕裂 ────────────────────────────────────────

type StateCode = u128;
/// 排好序、去了重的状态向量表：成员判断用二分。
type StateList = Vec<StateCode>;

const NOT_PERSISTED: u8 = 0;
const PERSISTED: u8 = 1;
const TORN: u8 = 2;
const BITS_PER_WRITE: usize = 2;
const MAXIMUM_WRITES_IN_STATE_CODE: usize = 64;

/// 崩溃放量装置 GPU 持久掩码的位数：`crates/singlefs-verification/src/crash_facts.rs:129` `MAXIMUM_ENUMERATED_WRITES`（登记第三节，结构常量）。
const MAXIMUM_ENUMERATED_WRITES: usize = 2048;
/// 一段展开出的状态数要装得进 u32：`crates/singlefs-verification/src/crash_verify_gpu.rs:305` 的断言（登记第三节，结构常量）。
const MAXIMUM_SEGMENT_STATES_IN_U32: u128 = u32::MAX as u128;
/// 一段的状态数要装得进 u64：`crates/singlefs-verification/src/crash.rs:1681`（登记第三节，结构常量）。
const MAXIMUM_SEGMENT_STATES_IN_U64: u128 = u64::MAX as u128;

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
    let source = root.join("scripts/experiment-apparatus/e167_multi_inflight_crash_cut/main.rs");
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

/// 命令行的第一个参数选哪几段。不带参数是第一、二、三段全跑；段名是指定范围；`halt` 与 `s3` 是子集。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentChoice {
    First,
    Second,
    Third,
    HaltChecksOnly,
    ScaleEstimateOnly,
}

/// 认不出的参数返回 `None`，由 `main` 退 2。
fn segments_for_argument(argument: Option<&str>) -> Option<Vec<SegmentChoice>> {
    match argument {
        None => Some(vec![SegmentChoice::First, SegmentChoice::Second, SegmentChoice::Third]),
        Some("first") => Some(vec![SegmentChoice::First]),
        Some("second") => Some(vec![SegmentChoice::Second]),
        Some("third") => Some(vec![SegmentChoice::Third]),
        Some("halt") => Some(vec![SegmentChoice::HaltChecksOnly]),
        Some("s3") => Some(vec![SegmentChoice::ScaleEstimateOnly]),
        Some(_) => None,
    }
}

fn main() {
    preflight();
    let arguments: Vec<String> = std::env::args().skip(1).filter(|argument| argument != "--force").collect();
    let Some(choices) = segments_for_argument(arguments.first().map(String::as_str)) else {
        eprintln!("  ✗ 认不出的参数 {:?}", arguments.first());
        eprintln!("  → 怎么办：不带参数跑全部三段；写段名 first、second、third 之一只跑那一段，或写 halt（只跑停机条款）、s3（只估规模）");
        std::process::exit(2);
    };
    let mut emitter = Emitter::new();
    let mut tally = RunTally::default();
    for choice in choices {
        if !tally.halt_conditions.is_empty() {
            break;
        }
        match choice {
            SegmentChoice::First => tally.absorb(run_everything(&mut emitter)),
            SegmentChoice::Second => tally.absorb(run_second_segment(&mut emitter)),
            SegmentChoice::Third => tally.absorb(run_third_segment(&mut emitter)),
            SegmentChoice::HaltChecksOnly => run_halt_checks(&mut emitter, &mut tally),
            SegmentChoice::ScaleEstimateOnly => run_scale_estimate(&mut emitter),
        }
    }
    emit(
        &mut emitter,
        format!(
            "name=verdict scope=run self_checks_passed={} void_conditions={} halt_conditions={} void_list={} halt_list={}",
            tally.self_checks_passed,
            tally.void_conditions.len(),
            tally.halt_conditions.len(),
            if tally.void_conditions.is_empty() { "none".to_string() } else { tally.void_conditions.join(",") },
            if tally.halt_conditions.is_empty() { "none".to_string() } else { tally.halt_conditions.join(",") },
        ),
    );
    println!("{}", emitter.finish());
    if !tally.halt_conditions.is_empty() {
        std::process::exit(3);
    }
    if !tally.void_conditions.is_empty() {
        std::process::exit(1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// 第二段：两盘几何（G4a、G4b）上的量
// ═══════════════════════════════════════════════════════════════════════════════════════════════
//
// 第一段把 Σ_free 的每一条发出流存在内存里逐条算（G1 共 12870 条）。两盘几何的 Σ_free 有 C(30, 15) = 155117520 条，存不下，
// 所以第二段不存流：深搜一边走一边记「每次写落在第几段」（段签名；同一个签名的流枚举域相同，只算一次、带上条数）与
// 「每次写发出时已被屏障或 FUA 钉住的写」（偏序的直接前驱，T_exact 用）。与第一段的对照：
// 单测 `streaming_evaluation_equals_the_in_memory_evaluation_on_the_one_disk_cells` 在 G1、G3、G5 的六个自由与带约束格上逐项对拍。

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

include!("../heavy_threads_default/threads.rs");

/// 线程数只影响挂钟、不影响任何一行输出（并行单位之间的合并是求和、取最小、取并）。`SINGLEFS_THREAD_CAP` 没设取登记的重活默认并行度（`.claude/heavy-thread-default`）。
fn worker_thread_count() -> usize {
    default_heavy_threads(include_str!("../../../.claude/heavy-thread-default"))
}

#[cfg(test)]
mod tests;
