//! E159：fsync 等待时间随并发数——组提交（甲）与 WAL 两臂（wal_full-K10、乙-M-K10）。
//!
//! 判据、失败条款、变异表、停机条款的权威登记在 `research/prompts/e159-preregistration.md`；
//! 这一份只实现登记写死的东西，不重新论证。**这一次只做到正式计时之前**：装置、单测、钉绝对值
//! 的断言、变异表、停机条款 S1–S3、冒烟跑（跑前登记第五节 5.7 第一段的主网格、PC1–PC3、AA、
//! G8、G1、G6 五轮正式计时不在这一次范围内，机器上并发编译会污染计时，见交回报告）。
//!
//! ## 从 E155 拷来的部分（到「二、」小节末尾为止，一个字不改）
//!
//! 从 `scripts/experiment-apparatus/e155_fourth_run_group_commit_concurrency.rs` 第 36–2058
//! 行原样拷来：常量、K1–K4 容量与树高、5.3 组公式、`solve_fixed_point_core`、甲 / WAL 两臂的单发
//! 与批量函数（含 `solve_jia_batch_publish`、`solve_write_ahead_log_batch_fsync`、
//! `solve_write_ahead_log_batch_checkpoint`），以及它们依赖的全部类型与函数。这些函数算得对不对
//! 归 E155 的锚点与变异表管；跑前登记停机条款 S2 要求这一段与源文件逐个 diff 为空。
//!
//! ## 这一次新写的部分
//!
//! 客体设备区域布局与 O_DIRECT I/O（`RealBlockIo`）、录制假设备（`RecordingBlockIo`，供
//! 不碰真 I/O 的结构性单测用）、睡眠假设备（`SleepingBlockIo`，供不需要真磁盘的时序结构单测
//! 用）、随机取整（`round_stochastic`）、三条臂的批计划构造（`plan_jia_batch` /
//! `plan_write_ahead_log_fsync_batch` / `plan_write_ahead_log_checkpoint`）、批的执行顺序
//! （`execute_batch_persist` / `execute_batch_finalize_system_configuration`）、组提交队列与
//! 发布者（`publisher_loop`）、WAL checkpoint 后台线程（`checkpoint_loop`）、闭环调用方
//! （`caller_loop`）、统计（分位数、规则 R、J1 / J2a / J4、Mann–Whitney A、PC1 的 Δ）、来宾块层
//! 读数、CLI 与 `main`。
//!
//! ## 已知的范围缺口（交回报告里也会写，供续派时看）
//!
//! - 开环到达（第五节 5.5 G8）只实现了 `t_issue` 取排定时刻这一条纯语义（`open_loop_t_issue`），
//!   没有接上完整的开环调用方池；G1–G8 的其余反向取样点、Q1–Q12 的完整统计与结果行、五轮合并
//!   都还没有实现——这些都在「正式计时」的范围内，这一次不做。
//! - journal 环记录数到上限时批次要等 checkpoint 做完（第五节 5.3、第八节表格「环里在飞记录数」）
//!   没有接上背压；这一次的取样量级碰不到 65536 条的上限，接上背压是续做时的欠账。
//! - 在飞 checkpoint 深度实现成单个顺序线程（峰值恒为 1），满足 A2「≤ 2」的上界但不是「最多
//!   两个」的字面实现；续做正式计时前要不要做成真并发由主 agent 定。

#![allow(dead_code, reason = "装置只实现了登记里的一部分，没接上的项是续做正式计时的入口")]

const NODE_BYTES: u64 = 16384;

mod aligned_buffers_and_block_devices;
mod batch_plans_and_execution;
#[cfg(test)]
mod capacity_tests;
#[cfg(test)]
mod convergence_and_saturation_status_tests;
// ============================================================
// 单测（跑前登记第七节钉绝对值的断言、第九节变异表 M1–M14 的目标）
// ============================================================
#[cfg(test)]
mod e159_anchor_and_rule_tests;
#[cfg(test)]
mod group_formula_tests;
mod group_touches_and_tree_layers;
#[cfg(test)]
mod jia_anchor_tests;
// ============================================================
// 五之二、legacy：第一次装置的原样实现（3.2 四处未改），只为 Q6.11 对比用，不进任何判据
// ============================================================
mod legacy;
mod pool_shape_and_fixed_point_solvers;
mod queue_loops_and_cell_runs;
mod verdicts_and_block_statistics;
mod write_ahead_log_solvers;

use self::aligned_buffers_and_block_devices::*;
use self::batch_plans_and_execution::*;
use self::group_touches_and_tree_layers::*;
use self::pool_shape_and_fixed_point_solvers::*;
use self::queue_loops_and_cell_runs::*;
use self::verdicts_and_block_statistics::*;
use self::write_ahead_log_solvers::*;

/// 数据单元与码 3 容器占盘宽度（含头）。
const UNIT_BYTES: u64 = 32768;
/// journal 记录固定宽度。
const RECORD_BYTES: u64 = 4096;
/// 系统配置槽宽度。
const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
/// 记录头宽度（`journal_named_items_per_record` 用它反推容量）。
const RECORD_HEADER_BYTES: u64 = 311;
/// 一条点名项的宽度。
const NAMED_ITEM_BYTES: u64 = 56;
/// 两块盘（`D`）。
const DEVICE_COUNT: u64 = 2;
/// 主几何下 journal 环安全系数 `F`（D22 已定项 2）。
const RING_SAFETY_FACTOR_MAIN: u64 = 3;
/// journal 环默认容量。
const RING_DEFAULT_BYTES: u64 = 768 * 1024 * 1024;
/// 主几何 `physical_block_size`。
const PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES: u64 = 512;
/// 反向取样点 `physical_block_size`（8.2）。
const REVERSE_PHYSICAL_BLOCK_SIZE_BYTES: u64 = 4096;

/// 码 2 节点头宽公式的常量部分：`86 + 2×key宽 + 29`（`anchors_r2*.py` 的 `hdr2`）。
const NODE_HEADER_BASE_BYTES: u64 = 86 + 29;
/// 节点指针宽（D19 已定项 8）。
const NODE_POINTER_BYTES: u64 = 86;
/// K11 反向取样点：内部条目再加身份引用 26 字节。
const IDENTITY_REFERENCE_EXTRA_BYTES: u64 = 26;

const EXTENT_KEY_BYTES: u64 = 24;
const EXTENT_LEAF_ENTRY_BYTES: u64 = 112;
const INODE_KEY_BYTES: u64 = 8;
const INODE_INTERNAL_ENTRY_BYTES: u64 = 120;
const ALLOCATION_KEY_BYTES: u64 = 10;
const ALLOCATION_LEAF_ENTRY_BYTES: u64 = 20;
const ACCOUNTING_KEY_BYTES: u64 = 22;
const ACCOUNTING_LEAF_ENTRY_BYTES: u64 = 34;
const MAPPING_KEY_BYTES: u64 = 27;
const MAPPING_LEAF_ENTRY_BYTES: u64 = 55;
const TREE_TABLE_KEY_BYTES: u64 = 8;
const TREE_TABLE_ENTRY_BYTES: u64 = 200;
/// 码 3 容器头宽（`UNIT_BYTES - 136`，独立于 `hdr2` 公式）。
const INODE_CONTAINER_HEADER_BYTES: u64 = 136;
const INODE_CONTAINER_ENTRY_BYTES: u64 = 140;

/// 记账池级（不带设备维）行数：inode 号水位、待删占用、已承诺预留。
const ACCOUNTING_POOL_WIDE_ENTRIES: u64 = 3;
/// 记账每块盘各一行的行数。
const ACCOUNTING_ENTRIES_PER_DEVICE: u64 = 6;
/// 树表条目数（主几何，7 棵）。
const TREE_TABLE_ENTRY_COUNT: u64 = 7;
/// 树表改根时相邻改动的条目数（extent、inode、分配记录、记账四棵树的根）。
const TREE_TABLE_TOUCHED_ROOTS: u64 = 4;
/// 一次发布点名项超过 67 时怎么切（K7 主口径）。
const NAMED_ITEMS_PER_RECORD_MAIN: u64 = (RECORD_BYTES - RECORD_HEADER_BYTES) / NAMED_ITEM_BYTES;

/// 码 2 节点头宽：`86 + 2×key宽 + 29`。
const fn node_header_bytes(key_bytes: u64) -> u64 {
    NODE_HEADER_BASE_BYTES + 2 * key_bytes
}

/// 一个节点能装的条目数：`(节点宽 − 头宽) / 条目宽`。
const fn node_capacity(key_bytes: u64, entry_bytes: u64) -> u64 {
    (NODE_BYTES - node_header_bytes(key_bytes)) / entry_bytes
}

/// 内部节点条目宽：分隔 key + 子指针（K3 主几何）；K11 反向取样点再加身份引用 26 字节。
const fn internal_entry_bytes(key_bytes: u64, identity_reference: bool) -> u64 {
    key_bytes + NODE_POINTER_BYTES + if identity_reference { IDENTITY_REFERENCE_EXTRA_BYTES } else { 0 }
}

fn inode_container_capacity() -> u64 {
    (UNIT_BYTES - INODE_CONTAINER_HEADER_BYTES) / INODE_CONTAINER_ENTRY_BYTES
}

// ============================================================
// E159 新增（跑前登记 research/prompts/e159-preregistration.md 第五节）：
// 客体设备布局、随机取整、批发布引擎、组提交与 WAL checkpoint、计时统计、CLI。
// 上面拷自 E155 的部分到此为止，一个字不改（登记第十一节停机条款 S2）。
// ============================================================

use e7_index_bench::Emitter;
use std::alloc::{alloc, dealloc, Layout};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const O_DIRECT: i32 = 0o40000; // naming-lint:external Linux open(2) 标志名
const ALIGNMENT_BYTES: usize = 4096;

// ---- 5.3 区域布局 ----
const CONFIGURATION_REGION_OFFSET_BYTES: u64 = 0;
const SYSTEM_CONFIGURATION_SLOT_ZERO_OFFSET_BYTES: u64 = 0;
const SYSTEM_CONFIGURATION_SLOT_ONE_OFFSET_BYTES: u64 = SYSTEM_CONFIGURATION_SLOT_BYTES;
const ROOT_SLOT_BASE_OFFSET_BYTES: u64 = 65536; // 64 KiB，留在 [0, 1 MiB) 区内、避开两个系统配置槽
const ROOT_SLOT_STRIDE_BYTES: u64 = 16384;
const ROOT_SLOT_COUNT: u64 = 24;
const JOURNAL_REGION_OFFSET_BYTES: u64 = 16 * 1024 * 1024;
const JOURNAL_REGION_BYTES: u64 = RING_DEFAULT_BYTES;
const UNIT_REGION_OFFSET_BYTES: u64 = 1024 * 1024 * 1024;
const UNIT_REGION_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// 只回放一块盘（第五节 5.3「回放哪块盘」）：模型算出的是两盘合计，除以这个数拿单盘量。
const REPLAY_DEVICE_COUNT: u64 = 1;

/// journal 环里在飞记录数上限：768 MiB ÷ 4096 ÷ F（F=3，A1 锚点）。
const JOURNAL_RING_RECORD_LIMIT: u64 = RING_DEFAULT_BYTES / RECORD_BYTES / RING_SAFETY_FACTOR_MAIN;

/// 诊断用：把 B1–B4 用到的几格期望值打出来，供人眼核对；判据本身钉在单测里
/// （`anchors_match_prereg_section_seven` 等，第七节 7.2），不靠这条命令的输出。
/// 纯算术、不碰真设备、不含任何计时字段——`scripts/experiment-apparatus/replay.sh` 拿它逐字节复跑
/// （E159 装置还没有正式计时产物时，这是唯一能稳定逐字节复现的一份）。
fn print_anchors() {
    let mut emitter = Emitter::new();
    let shape_file_count_one_family_one_data_unit = main_shape(1, Family::OneDataUnitPerFile);
    let jia_outcome_at_file_count_one = solve_jia_batch_publish(shape_file_count_one_family_one_data_unit, 1, true);
    let write_ahead_log_full_outcome_at_file_count_one = solve_write_ahead_log_batch_fsync(shape_file_count_one_family_one_data_unit, WriteAheadLogArm::WriteAheadLogFull, 1, true);
    let write_ahead_log_leaf_outcome_at_file_count_one = solve_write_ahead_log_batch_fsync(shape_file_count_one_family_one_data_unit, WriteAheadLogArm::WriteAheadLogLeaf, 1, true);
    println!("{}", emitter.emit_raw(&format!("name=anchor label=jia_p1_f1_k1_single_device_bytes value={}", jia_outcome_single_device_bytes(&jia_outcome_at_file_count_one))));
    println!("{}", emitter.emit_raw(&format!("name=anchor label=write_ahead_log_full_k10_p1_f1_k1_single_device_bytes value={}", write_ahead_log_full_outcome_at_file_count_one.total_bytes() / DEVICE_COUNT)));
    println!("{}", emitter.emit_raw(&format!("name=anchor label=yi_m_k10_p1_f1_k1_single_device_bytes value={}", write_ahead_log_leaf_outcome_at_file_count_one.total_bytes() / DEVICE_COUNT)));
    let main_geometry_shape = main_shape(10_000, Family::EightAdjacentDataUnitsPerFsync);
    let write_ahead_log_full_outcome_at_main_geometry_batch_size_one = solve_write_ahead_log_batch_fsync(main_geometry_shape, WriteAheadLogArm::WriteAheadLogFull, 1, true);
    let write_ahead_log_full_outcome_at_main_geometry_batch_size_sixteen = solve_write_ahead_log_batch_fsync(main_geometry_shape, WriteAheadLogArm::WriteAheadLogFull, 16, true);
    println!("{}", emitter.emit_raw(&format!("name=anchor label=write_ahead_log_full_k10_f8a_p10000_k1_single_device_expected_bytes value={:.3}", write_ahead_log_full_outcome_at_main_geometry_batch_size_one.total_bytes() as f64 / DEVICE_COUNT as f64)));
    println!("{}", emitter.emit_raw(&format!("name=anchor label=write_ahead_log_full_k10_f8a_p10000_k16_single_device_expected_bytes value={:.3}", write_ahead_log_full_outcome_at_main_geometry_batch_size_sixteen.total_bytes() as f64 / DEVICE_COUNT as f64)));
    println!("{}", emitter.finish());
}

struct CliArguments {
    subcommand: String,
    device_path: Option<String>,
    arm: Arm,
    concurrent_fsync_count: u64,
    group_commit_enabled: bool,
    warmup_seconds: f64,
    duration_seconds: f64,
    pc1_inject_milliseconds: Option<f64>,
    seed: u64,
    file_count: u64,
    family: Family,
}
const KNOWN_SUBCOMMANDS: [&str; 3] = ["anchors", "cell", "probe-device"];

/// `cell` 子命令登记的暖机秒数、时长秒数与文件数；不给对应选项就是它们。
const REGISTERED_WARMUP_SECONDS: f64 = 1.0;
const REGISTERED_DURATION_SECONDS: f64 = 2.0;
const REGISTERED_FILE_COUNT: u64 = 10_000;

/// 与登记规模不同的暖机、时长或文件数就是缩过规模的一趟：结果行接这一段，产物不当正式产物；三项都是登记值时不接。
fn reduced_scale_note(arguments: &CliArguments) -> String {
    let mut registered_values: Vec<String> = Vec::new();
    if arguments.warmup_seconds != REGISTERED_WARMUP_SECONDS {
        registered_values.push(format!("registered_warmup_seconds={REGISTERED_WARMUP_SECONDS}"));
    }
    if arguments.duration_seconds != REGISTERED_DURATION_SECONDS {
        registered_values.push(format!("registered_duration_seconds={REGISTERED_DURATION_SECONDS}"));
    }
    if arguments.file_count != REGISTERED_FILE_COUNT {
        registered_values.push(format!("registered_file_count={REGISTERED_FILE_COUNT}"));
    }
    if registered_values.is_empty() {
        String::new()
    } else {
        format!(" reduced=yes {} not_official_product=yes", registered_values.join(" "))
    }
}

/// `scripts/devices/vm-bench.sh` 按 `.claude/kb/vm-harness.md`「挂几块盘」的约定，
/// 把设备路径按顺序当成前缀参数塞在最前面、我们自己的参数跟在后面——
/// 所以在找到第一个认得出的子命令之前，把前面的位置参数都当成设备路径，
/// 第一个当默认 `--device`（后面出现的 `--device` 可以覆盖它）。
fn parse_arguments(raw_arguments: &[String]) -> CliArguments {
    let subcommand_index = raw_arguments.iter().position(|argument| KNOWN_SUBCOMMANDS.contains(&argument.as_str()));
    let leading_device_path = match subcommand_index {
        Some(0) | None => None,
        Some(_) => raw_arguments.first().cloned(),
    };
    let subcommand = subcommand_index.map(|found_index| raw_arguments[found_index].clone()).unwrap_or_else(|| "anchors".to_string());
    let remaining_arguments: &[String] = match subcommand_index {
        Some(found_index) => &raw_arguments[found_index + 1..],
        None => &[],
    };

    let mut arguments = CliArguments {
        subcommand,
        device_path: leading_device_path,
        arm: Arm::Jia,
        concurrent_fsync_count: 1,
        group_commit_enabled: true,
        warmup_seconds: REGISTERED_WARMUP_SECONDS,
        duration_seconds: REGISTERED_DURATION_SECONDS,
        pc1_inject_milliseconds: None,
        seed: 1,
        file_count: REGISTERED_FILE_COUNT,
        family: Family::EightAdjacentDataUnitsPerFsync,
    };
    let remaining_raw_arguments = remaining_arguments;
    let mut index = 0usize;
    while index < remaining_raw_arguments.len() {
        let flag = remaining_raw_arguments[index].as_str();
        let mut take_value = || {
            index += 1;
            remaining_raw_arguments.get(index).cloned().unwrap_or_default()
        };
        match flag {
            "--device" => arguments.device_path = Some(take_value()),
            "--arm" => {
                arguments.arm = match take_value().as_str() {
                    "jia" => Arm::Jia,
                    "wal-full" => Arm::WriteAheadLogFull,
                    "yi-m" => Arm::WriteAheadLogLeaf,
                    other => panic!("未知 --arm：{other}"),
                }
            }
            "--k" => arguments.concurrent_fsync_count = take_value().parse().expect("--k 要是整数"),
            "--group-commit" => arguments.group_commit_enabled = take_value() == "on",
            "--warmup-seconds" => arguments.warmup_seconds = take_value().parse().expect("--warmup-seconds 要是数字"),
            "--duration-seconds" => arguments.duration_seconds = take_value().parse().expect("--duration-seconds 要是数字"),
            "--pc1-inject-ms" => arguments.pc1_inject_milliseconds = Some(take_value().parse().expect("--pc1-inject-ms 要是数字")),
            "--seed" => arguments.seed = take_value().parse().expect("--seed 要是整数"),
            "--file-count" => arguments.file_count = take_value().parse().expect("--file-count 要是整数"),
            "--family" => {
                arguments.family = match take_value().as_str() {
                    "f1" => Family::OneDataUnitPerFile,
                    "f8a" => Family::EightAdjacentDataUnitsPerFsync,
                    other => panic!("未知 --family：{other}"),
                }
            }
            other => panic!("未知参数：{other}"),
        }
        index += 1;
    }
    arguments
}

fn run_cell_subcommand(arguments: &CliArguments) {
    let device_path = arguments.device_path.as_deref().expect("--device 必须给");
    let device_name = device_name_from_path(device_path);
    let device: Arc<dyn BlockDevice> = Arc::new(RealBlockDevice::open(device_path).expect("打开设备失败"));
    let before = read_block_layer_statistics(&device_name).ok();
    let shape = main_shape(arguments.file_count, arguments.family);
    let configuration = CellConfiguration {
        arm: arguments.arm,
        concurrent_fsync_count: arguments.concurrent_fsync_count,
        shared: true,
        group_commit_enabled: arguments.group_commit_enabled,
        warmup: Duration::from_secs_f64(arguments.warmup_seconds),
        duration: Duration::from_secs_f64(arguments.duration_seconds),
        worker_count: 32,
        pc1_inject_duration: arguments.pc1_inject_milliseconds.map(Duration::from_secs_f64),
        checkpoint_time_threshold: CHECKPOINT_TIME_THRESHOLD,
        effective_dirty_bytes_threshold: 256 * 1024 * 1024,
        seed: arguments.seed,
        shape,
    };
    let report = run_closed_loop_cell(device, configuration);
    let after = read_block_layer_statistics(&device_name).ok();
    let mut waits: Vec<u64> = report.samples_in_window.iter().map(|sample| sample.wait_nanos()).collect();
    waits.sort_unstable();
    let median = if waits.is_empty() { 0 } else { percentile_nanos(&waits, 0.5) };
    let ninety_ninth_percentile_nanoseconds = if waits.is_empty() { 0 } else { percentile_nanos(&waits, 0.99) };
    // 结果抓取要有完整性闸（`command-safety.md`）：借用 `e7_index_bench::Emitter`，
    // 收尾打一行 `E7RESULT name=done emitted=N`，`vm-bench.sh` 拿它对账（`.claude/kb/vm-harness.md`「结果抓取有一道完整性闸」）。
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "E159RESULT arm={} k={} group_commit={} samples={} median_ns={} p99_ns={} containment_violations={} checkpoint_count={} checkpoint_bytes={} bytes_written_estimate={}{}",
            arguments.arm.tag(),
            arguments.concurrent_fsync_count,
            arguments.group_commit_enabled,
            report.total_samples,
            median,
            ninety_ninth_percentile_nanoseconds,
            report.containment_violations,
            report.checkpoint_count,
            report.checkpoint_bytes_total,
            report.bytes_written_estimate,
            reduced_scale_note(arguments),
        ))
    );
    if let (Some(before), Some(after)) = (before, after) {
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "E159BLOCKSTAT device={} sectors_written_diff={} sectors_read_diff={} writes_completed_diff={} reads_completed_diff={}",
                device_name,
                after.sectors_written.saturating_sub(before.sectors_written),
                after.sectors_read.saturating_sub(before.sectors_read),
                after.writes_completed.saturating_sub(before.writes_completed),
                after.reads_completed.saturating_sub(before.reads_completed),
            ))
        );
    }
    println!("{}", emitter.finish());
}

/// 停机条款 S3：来宾回放盘的 `logical_block_size` 必须是 512（根槽宽度依赖这个值，第七节 B9）。
fn run_probe_device_subcommand(arguments: &CliArguments) {
    let device_path = arguments.device_path.as_deref().expect("--device 必须给");
    let device_name = device_name_from_path(device_path);
    let logical_block_size_path = format!("/sys/block/{device_name}/queue/logical_block_size");
    let logical_block_size_text = std::fs::read_to_string(&logical_block_size_path).unwrap_or_else(|error| panic!("读不到 {logical_block_size_path}：{error}"));
    let logical_block_size: u64 = logical_block_size_text.trim().parse().unwrap_or_else(|error| panic!("{logical_block_size_path} 里的内容 {logical_block_size_text:?} 解析不出整数：{error}"));
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!("E159PROBE device={device_name} logical_block_size={logical_block_size}")));
    println!("{}", emitter.finish());
}

fn main() {
    let raw_arguments: Vec<String> = std::env::args().collect();
    let arguments = parse_arguments(&raw_arguments[1..]);
    match arguments.subcommand.as_str() {
        "anchors" => print_anchors(),
        "cell" => run_cell_subcommand(&arguments),
        "probe-device" => run_probe_device_subcommand(&arguments),
        other => panic!("未知子命令：{other}（用 anchors、cell 或 probe-device）"),
    }
}
