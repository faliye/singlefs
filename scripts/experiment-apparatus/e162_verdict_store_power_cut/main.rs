//! E162（崩溃放量判定块存储选型）第四段：S4 掉电装置——三个候选各按自己声明的持久化路径，断电之后已确认的块丢不丢、坏不坏、库打不打得开。
//!
// admission: always 每跑一次都是新的虚机开机与新的设备侧日志，同一份代码两次跑的断电点落在的条目不同
// run-condition: command python3 bash ldd sha256sum strip
//!
//! 跑前登记：`research/prompts/e162-preregistration.md` 补 1–补 14（S4 的定义在补 5.2、补 6、补 7、补 8、补 9、补 10、补 11；
//! 这个文件是补 12 分段表「第四段」那一行）。独立手写模型，不与 `crates/` 共用代码（`.claude/rules/implementation-first.md` 第 4 条）：
//! 设备侧日志的解析自己写一份，读法照 dm-log-writes 的格式定义（与 `crates/singlefs-verification/src/device_log.rs` 文件头读同一份定义，
//! 不引它的代码；两处读法的差别写在补 5.2「设备侧日志的读法」）。候选的读写路径照登记 5.2 逐字实现，拷自 S1 那份源文件
//! `scripts/experiment-apparatus/e162_crash_verdict_block_store.rs`（出处行号在登记第十二节修订）。
//!
//! 角色（`--role <名>`；来宾里的角色前面还有 `vm-bench.sh` 塞进来的设备路径 `/dev/vda`，不用它）：
//!   selftest          锚点自检（登记 7.2 的 A1–A6 与补 7 的 B 系列），对不上退 1（V10）
//!   s4-stage          搭写盘附加根：`--destination <空目录>`；搭完用搭好的加载器实跑一次搭好的二进制 `--role selftest`
//!   s4-drive          宿主上的总驱动：`[--arms <R1,R0,K,F 的子集>] --kernel <内核> --out <产物文件> [--dry-run]`（不给 `--arms` 就是 R1、R0、K、F 全部四臂；给了是缩域，`s4_drive_config` 行写 `reduced=yes`）
//!   s4-write-guest    来宾：写盘开机（分区、mkfs、挂载、建库、标记、逐块写）
//!   s4-verify-guest   来宾：验盘开机（按计划重放盘面、loop、挂载、起核对子进程）
//!   s4-check-point    来宾里验盘角色起的核对子进程（开库、PC4-S/P/O 的改动、逐块核）
//!
//! **s4-drive 起 QEMU 虚机**，经统一任务服务交。带 `--dry-run` 时只搭附加根、打印将要起的每条 vm-bench 命令与变量，不起虚机。单测不调 s4-drive。
//! s4-drive 的退出码：0 全部开机做完、判定行都打了（判「翻」也是 0）；1 锚点对不上（V10）；2 用法不对；3 停机条款打中（H7–H10、H14）
//! 或开跑前提不满足（可用盘 < 40 GiB、可用内存 < 16 GiB、机器上有别的虚机或性能测量）。
//!
//! 计时：只有 Q4h（重开用时）一个量，由核对子进程自己用 `Instant` 计、写进它的结果行；父进程读子进程输出不打时间戳
//! （`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）。

use e7_index_bench::Emitter;
use redb::ReadableDatabase;
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::os::unix::fs::{FileExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

mod block_stores_and_verification;
mod block_values_candidates_and_power_cut_arms;
mod guest_roles_and_check_points;
mod marker_audit_drawing_and_anchors;
mod markers_device_logs_and_replay;
mod point_summaries_and_verdicts;
mod staging_and_virtual_machine_runs;

use self::block_stores_and_verification::*;
use self::block_values_candidates_and_power_cut_arms::*;
use self::guest_roles_and_check_points::*;
use self::marker_audit_drawing_and_anchors::*;
use self::markers_device_logs_and_replay::*;
use self::point_summaries_and_verdicts::*;
use self::staging_and_virtual_machine_runs::*;

// ============================== 登记写死的常量 ==============================

/// seed_S1（登记 5.1）：S4 的块值（补 5.2「每条主臂写 N_W = 6 000 块（2¹⁶ 状态、P-rand、seed_S1……）」）。
const SEED_CRASH_BLOCKS: u64 = 0xE162_0000_0000_0001;
/// seed_S2：只给 7.2 的 A3、A4 锚点用。
const SEED_THROUGHPUT_BLOCKS: u64 = 0xE162_0000_0000_0002;
/// seed_FP：输入指纹。
const SEED_FINGERPRINT: u64 = 0xE162_0000_0000_0007;
/// seed_CUT：分桶取断电点（补 7 B2）。
const SEED_CUT: u64 = 0xE162_0000_0000_000A;
/// seed_SUB：U-rand / U-tear 的子集与撕法（补 7 B3）。
const SEED_SUBSET: u64 = 0xE162_0000_0000_000B;
/// seed_PICK：C-commit 挑块（补 7 B5）。
const SEED_PICK: u64 = 0xE162_0000_0000_000C;

const SPLITMIX_INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;
const SPLITMIX_FIRST_MULTIPLIER: u64 = 0xBF58_476D_1CE4_E5B9;
const SPLITMIX_SECOND_MULTIPLIER: u64 = 0x94D0_49BB_1331_11EB;
/// 奇数混淆乘子：块值起点（登记 5.1）与各生成器的状态（补 7 B2、B3、B5）都乘它。
const MIXING_MULTIPLIER: u64 = 0xD1B5_4A32_D192_ED03;

const PRIMARY_STATES_PER_BLOCK: u64 = 65_536;
/// S4-bs10：每块 2¹⁰ 个状态。
const SMALL_STATES_PER_BLOCK: u64 = 1_024;
const BLOCK_KEY_BYTES: usize = 32;
const FINGERPRINT_BYTES: usize = 16;
const INTERLEAVED_STREAM_COUNT: u64 = 64;
const SEGMENTS_PER_NODE: u64 = 8;

/// N_W：每条主臂（与 S4-bs10）写盘开机写的块数（补 5.2）。
const MAIN_BLOCK_COUNT: u64 = 6_000;
/// PC4-L、PC4-NB、PC4-FD 各写 1 000 块（补 5.2 阳性对照表）。
const CONTROL_BLOCK_COUNT: u64 = 1_000;
/// V25：K 的写盘开机按 N_W 翻倍重做，至多到这么多块。
const ROCKSDB_REDO_BLOCK_LIMIT: u64 = 24_000;
/// V25：K 主格写盘开机报的 L0 落盘不到 4 次、或合并 0 次，就翻倍重做。
const ROCKSDB_FLUSH_MINIMUM: u64 = 4;
const ROCKSDB_COMPACTION_MINIMUM: u64 = 1;

const SECTOR_BYTES: usize = 512;
const MEBIBYTE: u64 = 1 << 20;
/// 分区（补 7 B9c）：vda1 自第 2 048 扇区起 3 072 MiB，vda2 紧随 64 MiB。
const FIRST_PARTITION_START_SECTOR: u64 = 2_048;
const FIRST_PARTITION_MEBIBYTES: u64 = 3_072;
const MARKER_PARTITION_MEBIBYTES: u64 = 64;
/// vm-bench 的变量（补 5.2 两种开机那张表）。
const DISK_MEBIBYTES: u64 = 3_200;
const LOG_DISK_MEBIBYTES: u64 = 4_096;
const WRITE_BOOT_MEMORY_MEBIBYTES: u64 = 4_096;
const WRITE_BOOT_PROCESSORS: u64 = 4;
const WRITE_BOOT_TIMEOUT_SECONDS: u64 = 3_600;
const VERIFY_BOOT_MEMORY_MEBIBYTES: u64 = 12_288;
const VERIFY_BOOT_TIMEOUT_SECONDS: u64 = 14_400;
/// 验盘每批不超过 200 个「断电点 × 取法」。
const VERIFY_BATCH_ITEM_LIMIT: usize = 200;

/// PC4-C：在 /dev/vda 第 8 192 扇区起逐个写 100 个扇区，fdatasync，再写 100 个（补 5.2）。
const WRITE_CACHE_MODEL_FIRST_SECTOR: u64 = 8_192;
const WRITE_CACHE_MODEL_BATCH_WRITES: u64 = 100;
/// PC4-C 的 U-rand 取 a = 9、i = 0，登记的丢块数是 47（补 7 B4）。
const WRITE_CACHE_MODEL_RANDOM_SUBSET_LOST: u64 = 47;

/// C-strat：200 点，桶 B 60、桶 C 40、其余归桶 A；S4-bs10：100 点，桶 B 30、桶 C 20。
const STRATIFIED_POINT_TOTAL: usize = 200;
const LONG_COMMIT_BUCKET_QUOTA: usize = 60;
const BETWEEN_COMMITS_BUCKET_QUOTA: usize = 40;
const SMALL_BLOCK_POINT_TOTAL: usize = 100;
const SMALL_BLOCK_LONG_COMMIT_QUOTA: usize = 30;
const SMALL_BLOCK_BETWEEN_COMMITS_QUOTA: usize = 20;
/// L_j 大于全部 L 的中位数 4 倍的算「长窗」。
const LONG_WINDOW_MEDIAN_FACTOR: u64 = 4;
/// C-commit：普通窗挑 6 块、长窗挑 2 块（长窗不足 2 个就全从普通窗补），一块多于 64 个边界时取 64 个等距点。
const PICKED_COMMIT_TOTAL: usize = 8;
const PICKED_LONG_COMMIT_TARGET: usize = 2;
const COMMIT_BOUNDARY_LIMIT: usize = 64;
/// C-commit 各点的 U-rand 生成器下标 = 10 000 + 100 × m + b。
const COMMIT_SUBSET_INDEX_BASE: u64 = 10_000;
const COMMIT_SUBSET_INDEX_STRIDE: u64 = 100;
/// PC4-L、PC4-NB、PC4-FD：20 点里 ≥ 10 点看到丢块信号。
const CONTROL_POINT_TOTAL: usize = 20;
const CONTROL_SIGNAL_POINT_MINIMUM: usize = 10;
/// V24：一条臂 C-strat 200 点里 U(k) 非空的不到 100 点，U-drop 的「不翻」不算不翻。
const UNFLUSHED_POINT_MINIMUM: usize = 100;
/// 分桶生成器的桶号（补 7 B2、B5）。
const BUCKET_NORMAL_COMMITS: u64 = 0;
const BUCKET_LONG_COMMITS: u64 = 1;
const BUCKET_BETWEEN_COMMITS: u64 = 2;
const BUCKET_PICK_NORMAL: u64 = 3;
const BUCKET_PICK_LONG: u64 = 4;

/// 核对子进程 3 600 秒没交回就杀、记 Q4f 超时；Q4h 另报 > 60 秒的次数。
const CHECK_TIMEOUT_SECONDS: u64 = 3_600;
const SLOW_OPEN_MICROSECONDS: u64 = 60_000_000;
const CHECK_POLL_MILLISECONDS: u64 = 100;

/// PC4-S、PC4-P、PC4-O（照登记 5.3 PC-S、PC-P、PC-O）。
const SILENT_CORRUPTION_BLOCK: u64 = 7;
const SILENT_CORRUPTION_BYTE_OFFSET: usize = 40_000;
const SILENT_CORRUPTION_MASK: u8 = 0x5A;
const PHANTOM_KEY_NODE: u32 = 0xFFFF_FFFF;
const REDB_BROKEN_PREFIX_BYTES: usize = 4_096;
const ROCKSDB_BROKEN_CURRENT: &[u8] = b"MANIFEST-999999\n";

/// 候选 F 的文件格式（登记 5.2）。
const FILE_MAGIC: &[u8; 4] = b"SFCB";
const FILE_FORMAT_VERSION: u32 = 1;
const FILE_HEADER_BYTES: usize = 48;
const FILE_HEADER_CHECKSUMMED_BYTES: usize = 44;
const FORMAT_MAGIC: &[u8; 8] = b"SFE162FB";
const FORMAT_FILE_BYTES: usize = 16;
const FORMAT_FILE_NAME: &str = "FORMAT";
const REDB_TABLE: redb::TableDefinition<&[u8], &[u8]> = redb::TableDefinition::new("blocks");
const REDB_FILE_NAME: &str = "blocks.redb";

/// 标记扇区（补 5.2「标记」）：`SFMK`｜种类｜三个 0｜c｜j｜臂号｜前 28 字节的 CRC-32C｜480 个 0。
const MARKER_MAGIC: &[u8; 4] = b"SFMK";
const MARKER_CHECKSUMMED_BYTES: usize = 28;
const MARKER_WITHOUT_BLOCK: u64 = u64::MAX;

/// dm-log-writes 的系统配置与条目头（小端）。
const WRITE_LOG_MAGIC: u64 = 0x006a_7366_7773_6872;
const WRITE_LOG_VERSION: u64 = 1;
const LOG_FLUSH_FLAG: u64 = 1;
const LOG_FUA_FLAG: u64 = 1 << 1;
const LOG_DISCARD_FLAG: u64 = 1 << 2;
const LOG_MARK_FLAG: u64 = 1 << 3;
const LOG_METADATA_FLAG: u64 = 1 << 4;
const LOG_ENTRY_HEADER_BYTES: usize = 32;
const LOG_SUPER_BYTES: usize = 28;

/// Linux x86-64 的 O_DIRECT（`<asm-generic/fcntl.h>` 的 00040000）。标记的 pwrite 要它：返回时请求已经到盘。
const OPEN_DIRECT_FLAG: i32 = 0o40_000;
const DIRECT_WRITE_ALIGNMENT_BYTES: usize = 4_096;

/// 来宾里的路径。
const GUEST_WHOLE_DISK: &str = "/dev/vda";
const GUEST_FIRST_PARTITION: &str = "/dev/vda1";
const GUEST_MARKER_PARTITION: &str = "/dev/vda2";
const GUEST_MOUNT_POINT: &str = "/mnt/s4";
const GUEST_LIBRARY_NAME: &str = "library";
const GUEST_BUSYBOX: &str = "/bin/busybox";
const GUEST_PARTITIONER: &str = "/usr/sbin/sfdisk";
const GUEST_FILESYSTEM_FORMATTER: &str = "/usr/sbin/mkfs.ext4";
const GUEST_IMAGE_DIRECTORY: &str = "/s4-images";
const GUEST_IMAGE_FILESYSTEM_OPTIONS: &str = "size=8g";
const GUEST_LOOP_DEVICE: &str = "/dev/loop0";
const GUEST_LOG_PATH: &str = "/s4/device.log";
const GUEST_PLAN_PATH: &str = "/s4/plan.txt";
const STAGED_BINARY_GUEST_PATH: &str = "/usr/bin/e162-verdict-store-power-cut";
const STAGED_LOADER_GUEST_PATH: &str = "/lib64/ld-linux-x86-64.so.2";

const HALT_EXIT_CODE: i32 = 3;
const USAGE_EXIT_CODE: i32 = 2;
/// 开跑前提（补 5.3「S4：宿主资源」）：可用内存 ≥ 16 GiB，库所在文件系统可用 ≥ 40 GiB。
const MINIMUM_AVAILABLE_MEMORY_BYTES: u64 = 16 << 30;
const MINIMUM_AVAILABLE_DISK_BYTES: u64 = 40 << 30;

fn sector_bytes() -> u64 {
    u64::try_from(SECTOR_BYTES).expect("512 装得进 u64")
}

fn first_partition_sectors() -> u64 {
    FIRST_PARTITION_MEBIBYTES * MEBIBYTE / sector_bytes()
}

fn marker_partition_start_sector() -> u64 {
    FIRST_PARTITION_START_SECTOR + first_partition_sectors()
}

fn marker_partition_sectors() -> u64 {
    MARKER_PARTITION_MEBIBYTES * MEBIBYTE / sector_bytes()
}

// ============================== 入口 ==============================

/// 来宾里跑的角色与核对子进程：来宾里没有 python3，不判准入（宿主上的 s4-drive 已经判过）。
const GUEST_ROLES: [&str; 3] = ["s4-write-guest", "s4-verify-guest", "s4-check-point"];

fn role_of(arguments: &[String]) -> Option<String> {
    arguments.iter().position(|argument| argument == "--role").and_then(|position| arguments.get(position + 1)).cloned()
}

fn preflight() -> Option<String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if role_of(&arguments).is_some_and(|role| GUEST_ROLES.contains(&role.as_str())) {
        return None;
    }
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/e162_verdict_store_power_cut/main.rs");
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = Command::new("python3");
    command.arg(script).arg("check").arg(source);
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
    let first_line = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").to_string();
    first_line.strip_prefix("forced").map(|summary| summary.trim().replace(' ', "_"))
}

fn main() {
    let forced_summary = preflight();
    let arguments: Vec<String> = std::env::args().skip(1).filter(|argument| argument != "--force").collect();
    let parsed = RoleArguments::parse(&arguments);
    let exit_code = match parsed.text("role").unwrap_or("") {
        "selftest" => {
            let mut output = ProductLines::to_standard_output();
            if let Some(summary) = &forced_summary {
                output.line(format!("name=preflight forced={summary}"));
            }
            let mismatches = run_anchors(&mut output);
            output.line(format!("name=verdict part=anchors anchor_mismatches={mismatches}"));
            output.finish();
            i32::from(mismatches != 0)
        }
        "s4-stage" => run_stage_role(&parsed),
        "s4-drive" => run_drive_role(&parsed, forced_summary.as_deref()),
        "s4-write-guest" => run_write_guest_role(&parsed),
        "s4-verify-guest" => run_verify_guest_role(&parsed),
        "s4-check-point" => run_check_point_role(&parsed),
        _ => {
            eprintln!("  ✗ 用法：e162-verdict-store-power-cut --role selftest | s4-stage --destination <空目录> | s4-drive [--arms <R1,R0,K,F>] --kernel <内核> --out <产物> [--dry-run]");
            eprintln!("  → 怎么办：s4-write-guest、s4-verify-guest、s4-check-point 是来宾里的角色，由 s4-drive 经 vm-bench.sh 起，不手敲");
            USAGE_EXIT_CODE
        }
    };
    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests;
