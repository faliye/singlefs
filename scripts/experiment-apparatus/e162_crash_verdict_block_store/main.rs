//! E162（崩溃放量判定块存储选型）：判定块存在 redb、RocksDB、加固的文件三者里哪一个。
//!
// admission: always 每跑一次都是新的杀进程与计时观测，同一份代码两次跑的计数与时延本来就不同
// run-condition: command python3 stat
//!
//! 跑前登记：`research/prompts/e162-preregistration.md`（第一段：锚点、S1、S2 与几何敏感性、M1–M13）。
//! 第二段（S3 经回环送块）不在这个文件里。
//!
//! 独立手写模型，不与 `crates/` 共用代码（`.claude/rules/implementation-first.md` 第 4 条）。
//! 与 `crates/` 的接点只有两处，都是照抄形态、不引代码：块键的形态（登记第三节「状态怎么编号」），
//! 候选「加固的文件」的持久步骤（`crates/singlefs-verification/src/layer0_progress.rs:1140-1160`）。
//!
//! 角色：父进程按模式驱动；被杀的写入进程（`writer`）、核对进程（`verify`）、读进程（`read`）
//! 都是这个 bin 按角色参数起的子进程。子进程不判准入（父进程已经判过），免得 python3 的启动
//! 时间落进「打开中途」那一截杀点里。
//!
//! 计时都在被测子进程里用 `Instant` 计、写进它自己的结果行；父进程读子进程的输出只用来排杀点
//! 与排占用测量，不打时间戳（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）。

use e7_index_bench::Emitter;
use redb::ReadableDatabase;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

mod block_stores_and_verification;
mod block_values_kill_plans_and_anchors;
mod child_roles_and_processes;
mod kill_campaigns;
mod timed_writes_and_segment_runs;

use self::block_stores_and_verification::*;
use self::block_values_kill_plans_and_anchors::*;
use self::child_roles_and_processes::*;
use self::kill_campaigns::*;
use self::timed_writes_and_segment_runs::*;

// ============================== 登记写死的常量 ==============================

/// 种子前缀（登记 5.1）。各种子 = 前缀加 1…9；前缀低 16 位是 0，加与按位或同值，单测钉死两者相等。
const SEED_PREFIX: u64 = 0xE162_0000_0000_0000;
const SEED_CRASH_BLOCKS: u64 = SEED_PREFIX + 1;
const SEED_THROUGHPUT_BLOCKS: u64 = SEED_PREFIX + 2;
const SEED_KILL: u64 = SEED_PREFIX + 3;
const SEED_READ: u64 = SEED_PREFIX + 5;
const SEED_ORDER: u64 = SEED_PREFIX + 6;
const SEED_FINGERPRINT: u64 = SEED_PREFIX + 7;
const SEED_LARGE: u64 = SEED_PREFIX + 8;

const SPLITMIX_INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;
const SPLITMIX_FIRST_MULTIPLIER: u64 = 0xBF58_476D_1CE4_E5B9;
const SPLITMIX_SECOND_MULTIPLIER: u64 = 0x94D0_49BB_1331_11EB;
/// 第 j 块的起始状态 = seed XOR (j × 这个数 mod 2⁶⁴)（登记 5.1）。
const BLOCK_START_MULTIPLIER: u64 = 0xD1B5_4A32_D192_ED03;

/// 主取样点每块状态数 2¹⁶；每状态 1 字节判定编号，块值 65 536 字节（登记 1.1、A-C1）。
const PRIMARY_STATES_PER_BLOCK: u64 = 65_536;
/// G-bs10 取样点每块状态数 2¹⁰。
const SMALL_STATES_PER_BLOCK: u64 = 1_024;
const BLOCK_KEY_BYTES: usize = 32;
const FINGERPRINT_BYTES: usize = 16;
/// 交错次序：64 条流，节点 = 流号 ÷ 8，段号 = 流号 mod 8（登记 5.1）。
const INTERLEAVED_STREAM_COUNT: u64 = 64;
const SEGMENTS_PER_NODE: u64 = 8;

/// S1 杀点计划 200 行，其中 i mod 10 = 9 的是「打开中途」（登记 5.5）。
const CRASH_MAIN_KILL_CYCLES: usize = 200;
const CRASH_LARGE_KILL_CYCLES: usize = 20;
const OPEN_PHASE_CYCLE_MODULUS: usize = 10;
const OPEN_PHASE_CYCLE_REMAINDER: usize = 9;
/// 写中途：n_i = 1 + (r1 mod 64)。
const WRITE_PHASE_CONFIRMATION_MODULUS: u64 = 64;
/// 写中途等 u × 4 × m、打开中途等 u × 2 × o。
const WRITE_PHASE_DELAY_FACTOR: f64 = 4.0;
const OPEN_PHASE_DELAY_FACTOR: f64 = 2.0;
/// 阳性对照 PC-L、PC-T 用杀点计划的前 20 行。
const POSITIVE_CONTROL_KILL_CYCLES: usize = 20;
/// PC-L：18 次写中途的杀里 ≥ 10 次 Q1a ≥ 1（V1）。
const POSITIVE_CONTROL_LOST_MINIMUM_HITS: usize = 10;
/// V6：180 次写中途的杀里杀在提交中间（最后一行是 B）的次数 ≥ 100。
const KILL_INSIDE_COMMIT_MINIMUM: usize = 100;
/// 核对子进程的悬挂保护（登记 5.5）。
const VERIFICATION_TIMEOUT_SECONDS: u64 = 3_600;
/// 杀后逐块核时，已开始的块之后再读这么多块未开始的块（找幽灵块，另有枚举兜底）。
const NOT_STARTED_BLOCKS_READ_AFTER_STARTED: u64 = 64;
/// S1-large 每次杀后从前 10⁵ 块里抽核的块数。
const CRASH_LARGE_SAMPLED_OLD_BLOCKS: usize = 1_000;

/// S1 校准趟与各阳性对照的块数；S2 每趟 10⁵ 块（登记 5.5）。
const CALIBRATION_BLOCK_COUNT: u64 = 1_000;
const THROUGHPUT_BLOCK_COUNT: u64 = 100_000;
/// 可行性档：S1 杀 20 次、S2 写 10⁴ 块，各缩十倍；S1-large 与几何取样点不跑。
const FEASIBILITY_KILL_CYCLES: usize = 20;
const FEASIBILITY_THROUGHPUT_BLOCK_COUNT: u64 = 10_000;

/// 跑哪一档：登记的次数，还是可行性档（只验能跑、能用，不判）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunTier {
    Registered,
    Feasibility,
}

impl RunTier {
    fn from_argument(argument: Option<&String>) -> Option<RunTier> {
        match argument.map(String::as_str) {
            None => Some(RunTier::Registered),
            Some("feasibility") => Some(RunTier::Feasibility),
            Some(_) => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            RunTier::Registered => "registered",
            RunTier::Feasibility => "feasibility",
        }
    }

    fn judgement(self) -> &'static str {
        match self {
            RunTier::Registered => "judged_by_registration",
            RunTier::Feasibility => "not_judged_feasibility_tier_only",
        }
    }

    /// V15 等安静的上限：可行性档不要求计时干净，上限设成 0——
    /// 机器上一直有别的实验与编译在跑，等满 20 分钟对这一档没有意义，干扰进程照实记进产物就够。
    fn quiet_wait_maximum_seconds(self) -> u64 {
        match self {
            RunTier::Registered => INTERFERENCE_MAXIMUM_WAIT_SECONDS,
            RunTier::Feasibility => 0,
        }
    }

    /// 同一条修订：可行性档不因为有干扰就重跑（重跑在保证还会撞见同样的干扰时没有意义）。
    fn allow_interference_reruns(self) -> bool {
        match self {
            RunTier::Registered => true,
            RunTier::Feasibility => false,
        }
    }
}
/// 阳性对照 PC-S、PC-P、PC-O 的库：64 块。
const SMALL_LIBRARY_BLOCK_COUNT: u64 = 64;
/// PC-S：第 7 块值的第 40 000 字节异或 0x5A。
const SILENT_CORRUPTION_BLOCK: u64 = 7;
const SILENT_CORRUPTION_BYTE_OFFSET: usize = 40_000;
const SILENT_CORRUPTION_MASK: u8 = 0x5A;
/// PC-P：计划外的键，节点 0xFFFFFFFF、段号 0、起点 0。
const PHANTOM_KEY_NODE: u32 = 0xFFFF_FFFF;

/// S2：每 1 000 块一窗、量一次占用；写入与生成线程之间的有界通道深度 64（登记 1.1）。
const WINDOW_BLOCKS: u64 = 1_000;
const GENERATOR_CHANNEL_DEPTH: usize = 64;
/// G-batch16：一次提交 16 块。
const BATCH_BLOCKS: usize = 16;
/// 随机读 10⁴ 次（登记 1.1），PC-read 1 000 次。
const RANDOM_READ_COUNT: usize = 10_000;
const POSITIVE_CONTROL_READ_COUNT: usize = 1_000;
/// PC-rate、PC-read：计时区间里多睡 5 ms，两遍中位数之差 ≥ 4.5 ms。
const POSITIVE_CONTROL_SLEEP_MICROSECONDS: u64 = 5_000;
const POSITIVE_CONTROL_MINIMUM_DIFFERENCE_MICROSECONDS: u64 = 4_500;
/// PC-space：写一个 2³⁰ 字节的文件，两次占用之差落在 [2³⁰, 2³⁰ + 2²⁰]。
const POSITIVE_CONTROL_SPACE_FILE_BYTES: u64 = 1 << 30;
const POSITIVE_CONTROL_SPACE_SLACK_BYTES: u64 = 1 << 20;
/// A-raw：1 000 次「追加 64 KiB + fdatasync」；正确形态每块提交中位 ≥ 0.5 × L_raw（H2）。
const RAW_PROBE_APPENDS: usize = 1_000;
const RAW_PROBE_APPEND_BYTES: usize = 65_536;
/// H3：库目录占用超过 20 GiB 就停这一趟；H4：可用 < 40 GiB 不开跑。
const OCCUPANCY_STOP_BYTES: u64 = 20 << 30;
const MINIMUM_AVAILABLE_BYTES: u64 = 40 << 30;
/// 空间判定（Q2e）按的两个量级（登记 4.2）。
const MILESTONE_TWO_STATES: f64 = 1.9e10;
const CLAUSE_TEXT_STATES: f64 = 1.5e10;
/// V15：计时的趟开跑前等干扰进程散去，每 30 秒看一次，最多等 20 分钟；之后照跑、结束时再判。
const INTERFERENCE_POLL_SECONDS: u64 = 30;
const INTERFERENCE_MAXIMUM_WAIT_SECONDS: u64 = 1_200;
/// V15：一趟受干扰就重跑，最多重跑三次。
const INTERFERENCE_MAXIMUM_RERUNS: usize = 3;

/// 加固的文件：块文件头 48 字节（魔数 4｜版本 4｜块键 32｜块值长度 4｜CRC-32C 4），FORMAT 16 字节。
const FILE_MAGIC: &[u8; 4] = b"SFCB";
const FILE_FORMAT_VERSION: u32 = 1;
const FILE_HEADER_BYTES: usize = 48;
const FILE_HEADER_CHECKSUMMED_BYTES: usize = 44;
const FORMAT_MAGIC: &[u8; 8] = b"SFE162FB";
const FORMAT_FILE_BYTES: usize = 16;
const FORMAT_FILE_NAME: &str = "FORMAT";
/// F-lazyrename：改名与目录 fsync 攒到本趟第 8、16、24… 次确认时一起做。
const LAZY_RENAME_EVERY_CONFIRMATIONS: u64 = 8;
/// F-direct：块值分 16 次各 4096 字节写，每次之间睡 100 µs。
const DIRECT_WRITE_CHUNK_BYTES: usize = 4_096;
const DIRECT_WRITE_PAUSE_MICROSECONDS: u64 = 100;

/// redb 的表名与库文件名。
const REDB_TABLE: redb::TableDefinition<&[u8], &[u8]> = redb::TableDefinition::new("blocks");
const REDB_FILE_NAME: &str = "blocks.redb";

// ============================== 准入、入口、产物行 ==============================

/// 子进程角色的第一个参数；带着它起的是装置自己起的子进程，不再判准入。
const CHILD_ROLE_ARGUMENTS: [&str; 3] = ["writer", "verify", "read"];

/// 准入与多机配置用的仓根：环境变量 SINGLEFS_REPOSITORY_ROOT，没设取当前目录（replay.sh 与实验页的命令都在仓根起）；不取「清单目录/../..」（单元键把它当运行期路径拒算）。
fn repository_root() -> PathBuf {
    let root = std::env::var_os("SINGLEFS_REPOSITORY_ROOT")
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    if !root.join(".claude/singlefs-ai-sop/scripts/preflight.py").is_file() {
        eprintln!("  ✗ {} 不是仓根（下面没有 .claude/singlefs-ai-sop/scripts/preflight.py），判不了准入", root.display());
        eprintln!("  → 怎么办：在仓根起这个装置，或设 SINGLEFS_REPOSITORY_ROOT=<仓根>");
        std::process::exit(78);
    }
    root
}

fn preflight() -> Option<String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments
        .first()
        .is_some_and(|role| CHILD_ROLE_ARGUMENTS.contains(&role.as_str()))
    {
        return None;
    }
    let root = repository_root();
    let source = root.join("scripts/experiment-apparatus/e162_crash_verdict_block_store/main.rs");
    let script = root.join(".claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = Command::new("python3");
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
    let first_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    first_line
        .strip_prefix("forced")
        .map(|summary| summary.trim().replace(' ', "_"))
}

fn main() {
    let forced_summary = preflight();
    let arguments: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    let mode = arguments.first().map(String::as_str).unwrap_or("");
    let remaining: Vec<String> = arguments.iter().skip(1).cloned().collect();
    let exit_code = match mode {
        "writer" => run_writer_role(&remaining),
        "verify" => run_verify_role(&remaining),
        "read" => run_read_role(&remaining),
        "anchors" | "s1" | "s2" | "geometry" | "discrimination" => {
            let mut output = ProductLines::new();
            if let Some(summary) = forced_summary {
                output.line(format!("name=preflight forced={summary}"));
            }
            let code = match mode {
                "anchors" => run_anchors(&mut output),
                "s1" => run_segment_one_crash_question(&mut output, &remaining),
                "s2" => run_segment_one_throughput_question(&mut output, &remaining),
                "geometry" => run_segment_one_geometry(&mut output, &remaining),
                _ => run_discrimination(&mut output, &remaining),
            };
            output.finish();
            code
        }
        _ => {
            eprintln!("  ✗ 用法：e162-crash-verdict-block-store anchors | s1 <臂> [feasibility] | s2 <臂> [feasibility] | geometry <臂> <取样点> | discrimination <Q2a> <Q2a>");
            eprintln!("  → 怎么办：臂取 R1 R0 K F；取样点取 G-bs10 G-batch16 G-sparse G-random；writer/verify/read 是装置自己起的子进程角色");
            2
        }
    };
    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests;
