//! E177（逐树不写墓碑的复活数）装置：跑前登记 `research/prompts/e177-preregistration.md`（第一至十一节、修订 1、执行员修订 2）。
//!
//! 独立手写模型，不与 `crates/` 共用代码、不依赖 `crates/`：写路径照登记第三节 S2–S16、修订 1 的 W1–W8 与执行员修订 2 的
//! 核对 2–4（M1–M11）逐条手写；读侧三种（R-U、R-S、R-G）、三条臂（N、B、P）、三个几何（G-环、G-宽、G-紧）。
//! 格式常量写成本地常量，旁边写来源行；不引 `crates/` 的任何东西。登记没说到、非取一个读法不可的几格照执行员修订 2 的 C1–C16。
//!
//! 用法（在 research/ 下）：
//!   cargo run --release --bin e177-per-tree-resurrection-without-tombstones -- segment-one
//! 线程数取环境变量 E177_THREADS（没设取 SINGLEFS_THREAD_CAP，都没设取登记的重活默认并行度 .claude/heavy-thread-default，按段切片）；产物按段的次序输出，与线程数无关。
//! 产物是 `E7RESULT` 行，收尾 `E7RESULT name=done emitted=N`（N 把 config 行与 done 行都算进去）。

// admission: always 每次照此刻的仓与自己的常量现算，上一次跑出的产物不替这一次作保
// run-condition: command python3
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

mod allocation_nodes_and_unit_contents;
mod code_two_tree_planning;
mod histories_and_segment_runs;
mod logical_planning;
mod placement_and_allocator;
mod planting_for_positive_controls;
mod publish_assembly;
mod readers_arms_and_geometries;
mod scan_reading;
mod simulation;
#[cfg(test)]
mod test_support_for_the_runs;
mod tree_kinds_namespaces_and_reported_trees;
mod verdicts_and_anchor_lines;

use self::allocation_nodes_and_unit_contents::*;
use self::code_two_tree_planning::*;
use self::histories_and_segment_runs::*;
use self::logical_planning::*;
use self::placement_and_allocator::*;
use self::planting_for_positive_controls::*;
use self::publish_assembly::*;
use self::readers_arms_and_geometries::*;
use self::scan_reading::*;
use self::simulation::*;
#[cfg(test)]
use self::test_support_for_the_runs::*;
use self::tree_kinds_namespaces_and_reported_trees::*;
use self::verdicts_and_anchor_lines::*;

// ---------------------------------------------------------------------------------------------
// 格式常量（本地常量，值抄自 `crates/` 的登记位，行号取自工作区）
// ---------------------------------------------------------------------------------------------

/// 槽 16 KiB（`crates/singlefs-format/src/lib.rs:14`）。
const SLOT_BYTES: u64 = 16_384;
/// 每块盘 4 GiB（`crates/singlefs-format/src/lib.rs:308` 的测试镜像默认字节数）。
const DEVICE_BYTES: u64 = 4_294_967_296;
/// 一块盘的绝对槽数：262 144。
const DEVICE_SLOTS: u64 = DEVICE_BYTES / SLOT_BYTES;
/// 单元区起点（`crates/singlefs-format/src/lib.rs:291`）。
const UNIT_AREA_START_SLOT: u64 = 50_176;
/// G-环 的单元区槽数（`allocator.rs:342-348`）：211 968。
const RING_GEOMETRY_UNIT_AREA_SLOTS: u64 = DEVICE_SLOTS - UNIT_AREA_START_SLOT;
/// 聚簇段 64 槽（`crates/singlefs-format/src/lib.rs:294`）。
const CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// 根环区域数（`crates/singlefs-format/src/lib.rs:261`）。
const ROOT_RING_REGIONS: u64 = 3;
/// mkfs 时每区槽数（`crates/singlefs-format/src/lib.rs:279`）。
const ROOT_RING_SLOTS_PER_REGION: u64 = 8;
/// A15 的环长：独立写成 24，不从上面两个常量算（修订 1 第 12.8 节 V12 那一行）。
const RING_LENGTH_IN_ROOTS_FOR_THE_REUSE_ASSERTION: u64 = 24;
/// Q14 的环窗：t − s ≤ 23 算环窗内（修订 1 第 12.5 节）。
const RING_WINDOW_IN_PUBLISHES: u64 = 23;
/// G-环 的占用上限：单元区的 10%，21 196 槽（修订 1 第 12.3 节）。
const OCCUPANCY_CAP_SLOTS: u64 = 21_196;
/// 池里两块盘（登记 5.1）。
const DEVICE_COUNT: u8 = 2;
/// 码 3 容器 32 KiB（`crates/singlefs-format/src/lib.rs:20`）与记录区起点 136（`:44`）。
const PACKED_CONTAINER_BYTES: u64 = 32_768;
const PACKED_RECORDS_OFFSET: u64 = 136;
/// inode 记录 140 字节（`crates/singlefs-format/src/lib.rs:104`）、墓碑记录 40 字节（`:113`）。
const INODE_RECORD_BYTES: u64 = 140;
const TOMBSTONE_RECORD_BYTES: u64 = 40;
/// 一片 inode 叶容器装 233 条（`crates/singlefs-format/src/lib.rs:110`）。
const INODE_LEAF_RECORDS: u64 = 233;
/// 一片墓碑容器装 815 条（`crates/singlefs-format/src/lib.rs:116`；登记 K1）。
const TOMBSTONE_CONTAINER_RECORDS: u64 = 815;

// 整数字面量给门禁 format-constants 读；推导式在编译期核，两处对不上编不过
const _: () = assert!(INODE_LEAF_RECORDS == (PACKED_CONTAINER_BYTES - PACKED_RECORDS_OFFSET) / INODE_RECORD_BYTES);
const _: () = assert!(TOMBSTONE_CONTAINER_RECORDS == (PACKED_CONTAINER_BYTES - PACKED_RECORDS_OFFSET) / TOMBSTONE_RECORD_BYTES);
/// 码 2 节点 16 KiB；头 = 86 + 2 × key 宽 + 预留 29（`crates/singlefs-core/src/unit.rs:129-152`）。
const NODE_BYTES: u64 = 16_384;
const INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE: u64 = 86;
const INDEX_NODE_RESERVED_BYTES: u64 = 29;
/// 子指针 86 字节（`crates/singlefs-format/src/lib.rs:89`）。
const NODE_POINTER_BYTES: u64 = 86;
/// 记账 key 22、条目 34（`crates/singlefs-format/src/lib.rs:143`、`:145`）；映射 key 27、条目 55（`:92`、`:95`）。
const ACCOUNTING_KEY_BYTES: u64 = 22;
const ACCOUNTING_ENTRY_BYTES: u64 = 34;
const MAPPING_KEY_BYTES: u64 = 27;
const MAPPING_ENTRY_BYTES: u64 = 55;
/// inode 树根条目 120 字节、key 8 字节（`crates/singlefs-format/src/lib.rs:107`）：一个根装 135 片叶容器。
const INODE_INTERNAL_ENTRY_BYTES: u64 = 120;
const INODE_KEY_BYTES: u64 = 8;
/// extent 树下段一片叶 144 个单元（`crates/singlefs-format/src/lib.rs:171`），内部扇出 147（`:168`）。
const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
/// 分配记录树叶罩 812 槽（`crates/singlefs-format/src/lib.rs:155`），内部扇出 169（`:161`）。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// 产品批大小 = 下段一片叶的单元数（`crates/singlefs-core/src/unlink.rs` 的 `DeletionSwitches::PRODUCT`）。
const PRODUCT_UNLINK_BATCH_DATA_UNITS: u64 = EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
/// 记账行：池级 3 行（`transaction.rs:4816`）、每盘 6 行（`transaction.rs:4856`）。
const ACCOUNTING_POOL_WIDE_ROWS: u64 = 3;
const ACCOUNTING_PER_DEVICE_ROWS: u64 = 6;
/// 不带设备维的统计量的设备段（模型自取，执行员修订 2 C6）。
const STATISTIC_WITHOUT_DEVICE: u64 = 0xFFFF_FFFF;
/// 新池新建文件写出的树表条目数（`crates/singlefs-format/src/lib.rs:188`；登记 K5）。
const TREE_TABLE_ENTRIES: u64 = 11;
/// 十二棵树的树 ID（`crates/singlefs-format/src/lib.rs:191-204`）。
const TREE_IDENTIFIER_EXTENT: u64 = 65_536;
const TREE_IDENTIFIER_INODE: u64 = 65_537;
const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = 65_538;
const TREE_IDENTIFIER_ACCOUNTING: u64 = 65_539;
const TREE_IDENTIFIER_DELETED_INODES: u64 = 65_544;
const TREE_IDENTIFIER_TOMBSTONE: u64 = 65_545;
/// 单元类标签（D18 已定项 11 的登记表）：码 1 数据、码 2 索引节点、码 3 打包记录单元。
const UNIT_CLASS_DATA: u64 = 1;
const UNIT_CLASS_INDEX_NODE: u64 = 2;
const UNIT_CLASS_PACKED: u64 = 3;
/// 打包记录类型：1 墓碑、2 inode、4 实例表（`crates/singlefs-core/src/unit.rs:21-25`）。
const PACKED_TYPE_TOMBSTONE: u64 = 1;
const PACKED_TYPE_INODE: u64 = 2;
/// 第一个文件恒是 inode 1；新池新建文件的 txg 3（`crates/singlefs-format/src/lib.rs:301`）。
const FIRST_FILE_INODE: u64 = 1;
const NEW_POOL_FILE_CREATION_TXG: u64 = 3;
/// mkfs 那个进程取号 1，可写挂载取号 2（执行员修订 2 C5）。
const MAKE_FILESYSTEM_PROCESS_INSTANCE: u64 = 1;
const MOUNT_INSTANCE: u64 = 2;
/// mkfs 写在单元区里的两个单元（`crates/singlefs-core/src/make_filesystem.rs:54-55`）。
const INSTANCE_TABLE_GENESIS_SLOT: u64 = UNIT_AREA_START_SLOT;
const TREE_TABLE_GENESIS_SLOT: u64 = UNIT_AREA_START_SLOT + 2;
/// 整对象墓碑的区间止（D18 已定项 24 第 2 条）。
const WHOLE_OBJECT_RANGE_END: u64 = u64::MAX;
/// 阳性对照植入的单元用的出生树：哪棵真树的 ID 都不是，保证「身份全新」（登记 5.5 PC1）。
const PLANTED_BIRTH_TREE: u64 = 0xE177;
/// 阳性对照植入的单元在单元表之外，用这个号指它。
const PLANTED_UNIT: usize = usize::MAX;

/// 一个码 2 节点装得下几条定宽条目：(16384 − 头) ÷ 条目宽。
const fn index_node_entry_capacity(key_width: u64, entry_width: u64) -> u64 {
    (NODE_BYTES - (INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE + 2 * key_width + INDEX_NODE_RESERVED_BYTES))
        / entry_width
}

/// 一个 inode 树根（与墓碑树根同形）装得下几片叶容器：135。
const ROOT_NODE_CONTAINER_CAPACITY: u64 =
    index_node_entry_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES);

fn length_as_u64(length: usize) -> u64 {
    u64::try_from(length).expect("长度装得进 u64")
}

fn u64_as_index(value: u64) -> usize {
    usize::try_from(value).expect("下标装得进 usize")
}

/// 线程数：`E177_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，都没设取登记的重活默认并行度（`.claude/heavy-thread-default`）。
fn threads_from_the_environment() -> usize {
    std::env::var("E177_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|threads| *threads >= 1)
        .unwrap_or_else(|| default_heavy_threads(include_str!("../../../.claude/heavy-thread-default")))
}

fn main() {
    preflight();
    let mode = std::env::args().nth(1).unwrap_or_default();
    let threads = threads_from_the_environment();
    let mut emitted_lines: Vec<String> = Vec::new();
    match mode.as_str() {
        "segment-one" => {
            let histories = segment_one_histories();
            emitted_lines.push(format!(
                "name=config experiment=E177 segment=one histories={} geometries=ring,wide,tight arms=N,B,P readers=R-U,R-S,R-G reported_trees={} unit_area_start={UNIT_AREA_START_SLOT} ring_unit_area_slots={RING_GEOMETRY_UNIT_AREA_SLOTS} occupancy_cap={OCCUPANCY_CAP_SLOTS} ring_window={RING_WINDOW_IN_PUBLISHES} inode_leaf_records={INODE_LEAF_RECORDS} tombstone_container_records={TOMBSTONE_CONTAINER_RECORDS} allocation_root_level={}",
                histories.len(),
                REPORTED_TREES.len(),
                allocation_tree_root_level(),
            ));
            let anchor = run_ring_anchor();
            emitted_lines.extend(anchor.lines.iter().cloned());
            let results = run_segments_in_parallel(&histories, threads);
            for ((result, identical), specification) in results.iter().zip(&histories) {
                emitted_lines.extend(result.lines.iter().cloned());
                emitted_lines.push(format!("name=determinism {} two_runs_identical={identical}", specification.label()));
            }
            emitted_lines.extend(verdict_lines(&results));
            emitted_lines.extend(summary_lines(&results, &anchor));
        }
        "anchor" => {
            let anchor = run_ring_anchor();
            emitted_lines.extend(anchor.lines);
        }
        _ => {
            eprintln!("用法：e177-per-tree-resurrection-without-tombstones segment-one | anchor");
            std::process::exit(2);
        }
    }
    let emitted = emitted_lines.len() + 1;
    for line in &emitted_lines {
        println!("E7RESULT {line}");
    }
    println!("E7RESULT name=done emitted={emitted}");
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
    let source = root.join("scripts/experiment-apparatus/e177_per_tree_resurrection_without_tombstones/main.rs");
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
