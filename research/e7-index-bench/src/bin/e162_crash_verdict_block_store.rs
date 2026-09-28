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
//! 候选「加固的文件」的持久步骤（`crates/singlefs-checker-tier/src/layer0_progress.rs:1140-1160`）。
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
/// 可行性档（用户 2026-09-27 定，登记修订六）：S1 杀 20 次、S2 写 10⁴ 块，各缩十倍；S1-large 与几何取样点不跑。
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

    /// V15 等安静的上限（用户 2026-09-27 定，登记修订七）：可行性档不要求计时干净，上限设成 0——
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

// ============================== 随机源与校验和 ==============================

/// SplitMix64（登记 5.1）：全部 64 位回绕。
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(state: u64) -> Self {
        Self { state }
    }

    fn next_output(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX_INCREMENT);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(SPLITMIX_FIRST_MULTIPLIER);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(SPLITMIX_SECOND_MULTIPLIER);
        mixed ^ (mixed >> 31)
    }
}

/// CRC-32C（Castagnoli，反射多项式 0x82F63B78），查表法。
struct Crc32cTable {
    entries: [u32; 256],
}

impl Crc32cTable {
    fn new() -> Self {
        let mut entries = [0u32; 256];
        for (byte_value, entry) in entries.iter_mut().enumerate() {
            let mut remainder = u32::try_from(byte_value).expect("表下标小于 256");
            for _bit in 0..8 {
                remainder = if remainder & 1 == 1 {
                    (remainder >> 1) ^ 0x82F6_3B78
                } else {
                    remainder >> 1
                };
            }
            *entry = remainder;
        }
        Self { entries }
    }

    fn checksum(&self, parts: &[&[u8]]) -> u32 {
        let mut running = 0xFFFF_FFFFu32;
        for part in parts {
            for &byte in part.iter() {
                let table_index = usize::from(running.to_le_bytes()[0] ^ byte);
                running = (running >> 8) ^ self.entries[table_index];
            }
        }
        running ^ 0xFFFF_FFFF
    }
}

fn crc32c(parts: &[&[u8]]) -> u32 {
    thread_local! {
        static TABLE: Crc32cTable = Crc32cTable::new();
    }
    TABLE.with(|table| table.checksum(parts))
}

// ============================== 块值、键、次序、杀点 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValuePattern {
    /// P-rand：从起始状态连取 V/8 个输出，逐个小端拼成 V 字节。
    Random,
    /// P-sparse：每状态取一个输出 r，r >> 56 == 0 时是 1 + (r & 0xFF) mod 255，否则 0。
    Sparse,
}

impl ValuePattern {
    fn label(self) -> &'static str {
        match self {
            ValuePattern::Random => "random",
            ValuePattern::Sparse => "sparse",
        }
    }

    fn parse(text: &str) -> ValuePattern {
        match text {
            "random" => ValuePattern::Random,
            "sparse" => ValuePattern::Sparse,
            other => panic!("认不出的块值形态 {other}"),
        }
    }
}

fn block_start_state(seed: u64, block_index: u64) -> u64 {
    seed ^ block_index.wrapping_mul(BLOCK_START_MULTIPLIER)
}

/// 第 block_index 块的块值；每状态 1 字节，所以块值字节数 = 每块状态数。
fn block_value(pattern: ValuePattern, seed: u64, block_index: u64, states_per_block: u64) -> Vec<u8> {
    let value_bytes = usize::try_from(states_per_block).expect("每块状态数装得进 usize");
    let mut generator = SplitMix64::new(block_start_state(seed, block_index));
    match pattern {
        ValuePattern::Random => {
            let mut value = Vec::with_capacity(value_bytes);
            for _word in 0..value_bytes / 8 {
                value.extend_from_slice(&generator.next_output().to_le_bytes());
            }
            value
        }
        ValuePattern::Sparse => {
            let mut value = vec![0u8; value_bytes];
            for state_verdict in value.iter_mut() {
                let output = generator.next_output();
                if output >> 56 == 0 {
                    *state_verdict = 1 + u8::try_from((output & 0xFF) % 255).expect("小于 255");
                }
            }
            value
        }
    }
}

/// 输入指纹：SplitMix64(seed_FP) 连取两个输出、小端拼成 16 字节。
fn input_fingerprint() -> [u8; FINGERPRINT_BYTES] {
    let mut generator = SplitMix64::new(SEED_FINGERPRINT);
    let mut fingerprint = [0u8; FINGERPRINT_BYTES];
    fingerprint[..8].copy_from_slice(&generator.next_output().to_le_bytes());
    fingerprint[8..].copy_from_slice(&generator.next_output().to_le_bytes());
    fingerprint
}

type BlockKey = [u8; BLOCK_KEY_BYTES];

/// 块键：节点 u32 大端｜段号 u32 大端｜起点 u64 大端｜输入指纹 16 字节。
fn compose_block_key(node: u32, segment: u32, first_state: u64, fingerprint: &[u8; FINGERPRINT_BYTES]) -> BlockKey {
    let mut key = [0u8; BLOCK_KEY_BYTES];
    key[0..4].copy_from_slice(&node.to_be_bytes());
    key[4..8].copy_from_slice(&segment.to_be_bytes());
    key[8..16].copy_from_slice(&first_state.to_be_bytes());
    key[16..32].copy_from_slice(fingerprint);
    key
}

/// 交错次序下第 block_index 块的键：流号 s = j mod 64、流内块号 b = j div 64，起点 = b × 每块状态数。
fn interleaved_block_key(block_index: u64, states_per_block: u64, fingerprint: &[u8; FINGERPRINT_BYTES]) -> BlockKey {
    let stream = block_index % INTERLEAVED_STREAM_COUNT;
    let block_in_stream = block_index / INTERLEAVED_STREAM_COUNT;
    let node = u32::try_from(stream / SEGMENTS_PER_NODE).expect("节点号小于 8");
    let segment = u32::try_from(stream % SEGMENTS_PER_NODE).expect("段号小于 8");
    compose_block_key(node, segment, block_in_stream * states_per_block, fingerprint)
}

/// [0, N) 的 Fisher–Yates 置换：i 从 N−1 降到 1，k = 输出 mod (i+1)，交换 a[i] 与 a[k]。
fn random_permutation(element_count: u64, seed: u64) -> Vec<u64> {
    let mut permutation: Vec<u64> = (0..element_count).collect();
    let mut generator = SplitMix64::new(seed);
    let mut position = permutation.len();
    while position > 1 {
        position -= 1;
        let bound = u64::try_from(position).expect("位置装得进 u64") + 1;
        let swap_with = usize::try_from(generator.next_output() % bound).expect("小于元素数");
        permutation.swap(position, swap_with);
    }
    permutation
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum KillPhase {
    /// 等到本趟第 n 行 `C` 之后再睡 u × 4 × m 杀。
    DuringWrite { confirmations_before_kill: u64 },
    /// 起子进程之后睡 u × 2 × o 杀。
    DuringOpen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct KillPlanRow {
    cycle_index: usize,
    phase: KillPhase,
    delay_fraction: f64,
}

/// 杀点计划：第 i 次先后取 r1、r2；u = (r2 >> 11) ÷ 2⁵³（登记 5.5）。
fn kill_plan(seed: u64, cycle_count: usize) -> Vec<KillPlanRow> {
    let mut generator = SplitMix64::new(seed);
    (0..cycle_count)
        .map(|cycle_index| {
            let first_output = generator.next_output();
            let second_output = generator.next_output();
            let phase = if cycle_index % OPEN_PHASE_CYCLE_MODULUS == OPEN_PHASE_CYCLE_REMAINDER {
                KillPhase::DuringOpen
            } else {
                KillPhase::DuringWrite {
                    confirmations_before_kill: 1 + first_output % WRITE_PHASE_CONFIRMATION_MODULUS,
                }
            };
            let delay_fraction = (second_output >> 11) as f64 / (1u64 << 53) as f64;
            KillPlanRow { cycle_index, phase, delay_fraction }
        })
        .collect()
}

fn hexadecimal(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ============================== 准入、入口、产物行 ==============================

/// 子进程角色的第一个参数；带着它起的是装置自己起的子进程，不再判准入。
const CHILD_ROLE_ARGUMENTS: [&str; 3] = ["writer", "verify", "read"];

fn preflight() -> Option<String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments
        .first()
        .is_some_and(|role| CHILD_ROLE_ARGUMENTS.contains(&role.as_str()))
    {
        return None;
    }
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/src/bin/e162_crash_verdict_block_store.rs");
    let script = format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py");
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

/// 产物行：每行 `E7RESULT ` 起头，收尾 `name=done emitted=N`（N 连 done 行自己都算）。
struct ProductLines {
    emitter: Emitter,
}

impl ProductLines {
    fn new() -> Self {
        Self { emitter: Emitter::new() }
    }

    fn line(&mut self, body: String) {
        let text = self.emitter.emit_raw(&body);
        let mut standard_output = std::io::stdout().lock();
        writeln!(standard_output, "{text}").expect("写得进标准输出");
        standard_output.flush().expect("刷得出标准输出");
    }

    fn finish(&mut self) {
        let text = self.emitter.finish();
        println!("{text}");
    }
}

// ============================== 锚点（登记 7.2，复跑逐字节比） ==============================

fn run_anchors(output: &mut ProductLines) -> i32 {
    let mut mismatches = 0usize;
    let mut record = |output: &mut ProductLines, name: &str, got: String, expected: &str| {
        let matches = got == expected;
        if !matches {
            mismatches += 1;
        }
        output.line(format!("name=anchor id={name} value={got} matches_registration={matches}"));
    };
    let mut zero_generator = SplitMix64::new(0);
    let first_three: Vec<String> = (0..3).map(|_| format!("{:#018x}", zero_generator.next_output())).collect();
    record(output, "A1", first_three.join(","), "0xe220a8397b1dcdaf,0x6e789e6aa1b965f4,0x06c45d188009454f");
    record(output, "A2", format!("{:#010x}", crc32c(&[b"123456789"])), "0xe3069283");
    for (block_index, expected) in [
        (0u64, "76020f1b59444d587a6607195c0f8fed/8351809/0x08eefd5c"),
        (12_345, "b3c1f0df0c77c9e98c9df5b21d0395bf/8321879/0x565d4801"),
    ] {
        let value = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
        let byte_sum: u64 = value.iter().map(|&byte| u64::from(byte)).sum();
        let summary = format!("{}/{byte_sum}/{:#010x}", hexadecimal(&value[..16]), crc32c(&[&value]));
        record(output, &format!("A3_j{block_index}"), summary, expected);
    }
    let small = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, 0, SMALL_STATES_PER_BLOCK);
    let small_sum: u64 = small.iter().map(|&byte| u64::from(byte)).sum();
    record(output, "A3b", format!("{}/{small_sum}", hexadecimal(&small[..16])), "76020f1b59444d587a6607195c0f8fed/131533");
    for (block_index, expected) in [(0u64, "267/32629"), (12_345, "245/28761")] {
        let value = block_value(ValuePattern::Sparse, SEED_THROUGHPUT_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
        let nonzero = value.iter().filter(|&&byte| byte != 0).count();
        let byte_sum: u64 = value.iter().map(|&byte| u64::from(byte)).sum();
        record(output, &format!("A4_j{block_index}"), format!("{nonzero}/{byte_sum}"), expected);
    }
    let fingerprint = input_fingerprint();
    record(output, "A5", hexadecimal(&fingerprint), "7299025377b7594e92ceeb8a1c3cbeac");
    for (block_index, expected) in [
        (0u64, "000000000000000000000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (1, "000000000000000100000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (63, "000000070000000700000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (64, "000000000000000000000000000100007299025377b7594e92ceeb8a1c3cbeac"),
        (12_345, "00000007000000010000000000c000007299025377b7594e92ceeb8a1c3cbeac"),
        (99_999, "000000030000000700000000061a00007299025377b7594e92ceeb8a1c3cbeac"),
    ] {
        let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
        record(output, &format!("A6_j{block_index}"), hexadecimal(&key), expected);
    }
    let small_key = interleaved_block_key(12_345, SMALL_STATES_PER_BLOCK, &fingerprint);
    record(output, "A6b", hexadecimal(&small_key), "000000070000000100000000000300007299025377b7594e92ceeb8a1c3cbeac");
    let permutation = random_permutation(THROUGHPUT_BLOCK_COUNT, SEED_ORDER);
    let head: Vec<String> = permutation[..6].iter().map(u64::to_string).collect();
    record(output, "A7", format!("{}/{}", head.join(","), permutation[99_999]), "40238,12140,20790,5207,14245,28683/91647");
    let plan = kill_plan(SEED_KILL, CRASH_MAIN_KILL_CYCLES);
    for (row, expected) in plan.iter().take(5).zip([
        "0/write/12/0.3236817149915837",
        "1/write/51/0.5669778417801824",
        "2/write/50/0.6058528089881228",
        "3/write/59/0.14049280271075437",
        "4/write/7/0.8853027823689169",
    ]) {
        record(output, &format!("A8_row{}", row.cycle_index), kill_plan_row_text(row), expected);
    }
    let open_rows = plan.iter().filter(|row| row.phase == KillPhase::DuringOpen).count();
    let write_confirmations: u64 = plan.iter().map(|row| confirmations_of(row.phase)).sum();
    record(output, "A8b", format!("{open_rows}/{write_confirmations}"), "20/6178");
    let milestone_blocks = (MILESTONE_TWO_STATES / PRIMARY_STATES_PER_BLOCK as f64).ceil();
    record(output, "A9", format!("{milestone_blocks}"), "289917");
    record(
        output,
        "A10",
        format!("{}/{}", THROUGHPUT_BLOCK_COUNT * PRIMARY_STATES_PER_BLOCK, THROUGHPUT_BLOCK_COUNT * SMALL_STATES_PER_BLOCK),
        "6553600000/102400000",
    );
    let frame_bytes = 4 + BLOCK_KEY_BYTES + 4 + 65_536 + 4;
    record(output, "A11", format!("{frame_bytes}/{}", FILE_HEADER_BYTES + 65_536), "65580/65584");
    output.line(format!("name=verdict part=anchors anchor_mismatches={mismatches}"));
    if mismatches == 0 {
        0
    } else {
        1
    }
}

fn kill_plan_row_text(row: &KillPlanRow) -> String {
    let (phase_label, confirmations) = match row.phase {
        KillPhase::DuringWrite { confirmations_before_kill } => ("write", confirmations_before_kill),
        KillPhase::DuringOpen => ("open", 0),
    };
    format!("{}/{phase_label}/{confirmations}/{}", row.cycle_index, row.delay_fraction)
}

fn confirmations_of(phase: KillPhase) -> u64 {
    match phase {
        KillPhase::DuringWrite { confirmations_before_kill } => confirmations_before_kill,
        KillPhase::DuringOpen => 0,
    }
}

// ============================== 候选与写法 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Candidate {
    /// R1：redb，每个写事务 Durability::Immediate + quick-repair。
    RedbQuickRepair,
    /// R0：redb，Durability::Immediate，quick-repair 关（默认）。
    RedbDefault,
    /// K：RocksDB，LZ4，同步写、WAL 开。
    RocksDatabase,
    /// F：加固的文件，一块一个文件，临时文件 → fsync → 改名 → 目录 fsync。
    HardenedFile,
}

impl Candidate {
    fn label(self) -> &'static str {
        match self {
            Candidate::RedbQuickRepair => "R1",
            Candidate::RedbDefault => "R0",
            Candidate::RocksDatabase => "K",
            Candidate::HardenedFile => "F",
        }
    }

    fn parse(text: &str) -> Option<Candidate> {
        match text {
            "R1" => Some(Candidate::RedbQuickRepair),
            "R0" => Some(Candidate::RedbDefault),
            "K" => Some(Candidate::RocksDatabase),
            "F" => Some(Candidate::HardenedFile),
            _ => None,
        }
    }

    /// PC-L 用的坏形态：R-None、K-noWAL、F-lazyrename（登记 5.3）。
    fn lost_block_control_write_path(self) -> WritePath {
        match self {
            Candidate::RedbQuickRepair | Candidate::RedbDefault => WritePath::RedbDurabilityNone,
            Candidate::RocksDatabase => WritePath::RocksDatabaseWithoutWriteAheadLog,
            Candidate::HardenedFile => WritePath::FileLazyRename,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WritePath {
    /// 臂定义里的写法（登记 5.2）。
    Registered,
    /// R-None：每次提交 Durability::None。
    RedbDurabilityNone,
    /// K-noWAL：disable_wal(true)（RocksDB 不许同步写关 WAL，所以 sync 同时关掉）。
    RocksDatabaseWithoutWriteAheadLog,
    /// F-lazyrename：临时文件 sync_all 后就报已确认，改名与目录 fsync 攒到第 8、16… 次确认。
    FileLazyRename,
    /// F-direct：直接写终名，头一次、块值分 16 次写，每次之间睡 100 µs，不 fsync、不改名。
    FileDirectInPlace,
}

impl WritePath {
    fn label(self) -> &'static str {
        match self {
            WritePath::Registered => "registered",
            WritePath::RedbDurabilityNone => "redb-durability-none",
            WritePath::RocksDatabaseWithoutWriteAheadLog => "rocksdb-no-wal",
            WritePath::FileLazyRename => "file-lazy-rename",
            WritePath::FileDirectInPlace => "file-direct",
        }
    }

    fn parse(text: &str) -> WritePath {
        match text {
            "registered" => WritePath::Registered,
            "redb-durability-none" => WritePath::RedbDurabilityNone,
            "rocksdb-no-wal" => WritePath::RocksDatabaseWithoutWriteAheadLog,
            "file-lazy-rename" => WritePath::FileLazyRename,
            "file-direct" => WritePath::FileDirectInPlace,
            other => panic!("认不出的写法 {other}"),
        }
    }
}

#[derive(Debug)]
enum StoreError {
    Redb(String),
    RocksDatabase(String),
    File(String),
    /// 单测用的「做到第 k 步就停」：留下的盘面与第 k 步之后被 SIGKILL 相同（页缓存还在）。
    AbandonedAtStep,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Redb(message) => write!(formatter, "redb: {message}"),
            StoreError::RocksDatabase(message) => write!(formatter, "rocksdb: {message}"),
            StoreError::File(message) => write!(formatter, "file: {message}"),
            StoreError::AbandonedAtStep => write!(formatter, "abandoned at step"),
        }
    }
}

fn redb_error(error: impl std::fmt::Display) -> StoreError {
    StoreError::Redb(error.to_string())
}

fn rocksdb_error(error: impl std::fmt::Display) -> StoreError {
    StoreError::RocksDatabase(error.to_string())
}

fn file_error(context: &str, error: impl std::fmt::Display) -> StoreError {
    StoreError::File(format!("{context}: {error}"))
}

/// 读一块的三种结局（登记 5.1 的三列）。
#[derive(Debug, PartialEq, Eq)]
enum ReadOutcome {
    Found(Vec<u8>),
    NotFound,
    ReadError(String),
}

enum StoreBackend {
    Redb(redb::Database),
    RocksDatabase(rocksdb::DB),
    File(FileLibrary),
}

struct BlockStore {
    candidate: Candidate,
    write_path: WritePath,
    backend: StoreBackend,
}

fn redb_durability_for(write_path: WritePath) -> redb::Durability {
    match write_path {
        WritePath::Registered => redb::Durability::Immediate,
        WritePath::RedbDurabilityNone => redb::Durability::None,
        WritePath::RocksDatabaseWithoutWriteAheadLog | WritePath::FileLazyRename | WritePath::FileDirectInPlace => {
            panic!("redb 臂不用 {} 这种写法", write_path.label())
        }
    }
}

fn rocksdb_options() -> rocksdb::Options {
    let mut options = rocksdb::Options::default();
    options.create_if_missing(true);
    options.set_compression_type(rocksdb::DBCompressionType::Lz4);
    options
}

fn rocksdb_write_options(write_path: WritePath) -> rocksdb::WriteOptions {
    let mut write_options = rocksdb::WriteOptions::default();
    let disable_write_ahead_log = write_path == WritePath::RocksDatabaseWithoutWriteAheadLog;
    write_options.disable_wal(disable_write_ahead_log);
    write_options.set_sync(!disable_write_ahead_log);
    write_options
}

/// 写入用的打开：库不在就建（redb 的库文件、RocksDB 的库目录、F 的根目录连同 FORMAT）。
fn open_store_for_writing(candidate: Candidate, write_path: WritePath, library: &Path) -> Result<BlockStore, StoreError> {
    let backend = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            fs::create_dir_all(library).map_err(|error| file_error("建 redb 库目录", error))?;
            StoreBackend::Redb(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?)
        }
        Candidate::RocksDatabase => StoreBackend::RocksDatabase(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?),
        Candidate::HardenedFile => StoreBackend::File(FileLibrary::open(library, write_path, true)?),
    };
    Ok(BlockStore { candidate, write_path, backend })
}

/// 核对用的打开：同一个打开调用（redb 的 `Database::create`、RocksDB 的 `DB::open`），F 不建新库。
fn open_existing_store(candidate: Candidate, library: &Path) -> Result<BlockStore, StoreError> {
    let backend = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => {
            StoreBackend::Redb(redb::Database::create(library.join(REDB_FILE_NAME)).map_err(redb_error)?)
        }
        Candidate::RocksDatabase => StoreBackend::RocksDatabase(rocksdb::DB::open(&rocksdb_options(), library).map_err(rocksdb_error)?),
        Candidate::HardenedFile => StoreBackend::File(FileLibrary::open(library, WritePath::Registered, false)?),
    };
    Ok(BlockStore { candidate, write_path: WritePath::Registered, backend })
}

impl BlockStore {
    /// 一次持久提交：blocks 是 1 块（主读法）或 16 块（G-batch16）。返回 Ok 就算这几块已确认。
    fn put_blocks(&mut self, blocks: &[(BlockKey, Vec<u8>)]) -> Result<(), StoreError> {
        match &mut self.backend {
            StoreBackend::Redb(database) => {
                let quick_repair = self.candidate == Candidate::RedbQuickRepair;
                let mut transaction = database.begin_write().map_err(redb_error)?;
                transaction.set_durability(redb_durability_for(self.write_path)).map_err(redb_error)?;
                transaction.set_quick_repair(quick_repair);
                {
                    let mut table = transaction.open_table(REDB_TABLE).map_err(redb_error)?;
                    for (key, value) in blocks {
                        table.insert(&key[..], &value[..]).map_err(redb_error)?;
                    }
                }
                transaction.commit().map_err(redb_error)
            }
            StoreBackend::RocksDatabase(database) => {
                let write_options = rocksdb_write_options(self.write_path);
                if blocks.len() == 1 {
                    database.put_opt(blocks[0].0, &blocks[0].1, &write_options).map_err(rocksdb_error)
                } else {
                    let mut batch = rocksdb::WriteBatch::default();
                    for (key, value) in blocks {
                        batch.put(key, value);
                    }
                    database.write_opt(batch, &write_options).map_err(rocksdb_error)
                }
            }
            StoreBackend::File(library) => library.put_blocks(blocks),
        }
    }

    fn read_block(&self, key: &BlockKey) -> ReadOutcome {
        match &self.backend {
            StoreBackend::Redb(database) => {
                let transaction = match database.begin_read() {
                    Ok(transaction) => transaction,
                    Err(error) => return ReadOutcome::ReadError(error.to_string()),
                };
                let table = match transaction.open_table(REDB_TABLE) {
                    Ok(table) => table,
                    Err(redb::TableError::TableDoesNotExist(_)) => return ReadOutcome::NotFound,
                    Err(error) => return ReadOutcome::ReadError(error.to_string()),
                };
                let outcome = match redb::ReadableTable::get(&table, &key[..]) {
                    Ok(Some(guard)) => ReadOutcome::Found(guard.value().to_vec()),
                    Ok(None) => ReadOutcome::NotFound,
                    Err(error) => ReadOutcome::ReadError(error.to_string()),
                };
                outcome
            }
            StoreBackend::RocksDatabase(database) => match database.get(key) {
                Ok(Some(value)) => ReadOutcome::Found(value),
                Ok(None) => ReadOutcome::NotFound,
                Err(error) => ReadOutcome::ReadError(error.to_string()),
            },
            StoreBackend::File(library) => library.read_block(key),
        }
    }

    /// 枚举库里的全部键与各自块值的长度（F 按文件名与文件长度，不读内容）。
    fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
        match &self.backend {
            StoreBackend::Redb(database) => {
                let transaction = database.begin_read().map_err(redb_error)?;
                let table = match transaction.open_table(REDB_TABLE) {
                    Ok(table) => table,
                    Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
                    Err(error) => return Err(redb_error(error)),
                };
                let mut entries = Vec::new();
                for item in redb::ReadableTable::iter(&table).map_err(redb_error)? {
                    let (key, value) = item.map_err(redb_error)?;
                    entries.push((key.value().to_vec(), u64::try_from(value.value().len()).expect("块值长度装得进 u64")));
                }
                Ok(entries)
            }
            StoreBackend::RocksDatabase(database) => {
                let mut entries = Vec::new();
                for item in database.iterator(rocksdb::IteratorMode::Start) {
                    let (key, value) = item.map_err(rocksdb_error)?;
                    entries.push((key.to_vec(), u64::try_from(value.len()).expect("块值长度装得进 u64")));
                }
                Ok(entries)
            }
            StoreBackend::File(library) => library.enumerate_keys_with_value_lengths(),
        }
    }
}

// ============================== 候选 F：加固的文件 ==============================

/// 一块一个文件：`<根>/<节点 8 位十六进制>/<段号 8 位十六进制>/<起点 16 位十六进制>-<指纹 32 位十六进制>.blk`。
struct FileLibrary {
    root: PathBuf,
    write_path: WritePath,
    /// F-lazyrename 攒着还没改名的（临时文件，终名）。
    pending_renames: Vec<(PathBuf, PathBuf)>,
    confirmations_in_this_run: u64,
    /// 单测用：还剩几步文件系统改动；None 是不限（产物里恒为 None）。
    remaining_steps: Option<u64>,
    known_directories: HashSet<PathBuf>,
}

fn sync_directory(directory: &Path) -> Result<(), StoreError> {
    File::open(directory)
        .and_then(|handle| handle.sync_all())
        .map_err(|error| file_error(&format!("目录 fsync {}", directory.display()), error))
}

fn format_file_contents() -> [u8; FORMAT_FILE_BYTES] {
    let mut contents = [0u8; FORMAT_FILE_BYTES];
    contents[0..8].copy_from_slice(FORMAT_MAGIC);
    contents[8..12].copy_from_slice(&FILE_FORMAT_VERSION.to_le_bytes());
    let checksum = crc32c(&[&contents[0..12]]);
    contents[12..16].copy_from_slice(&checksum.to_le_bytes());
    contents
}

fn block_file_relative_path(key: &[u8]) -> Option<PathBuf> {
    if key.len() != BLOCK_KEY_BYTES {
        return None;
    }
    let node = u32::from_be_bytes(key[0..4].try_into().expect("4 字节"));
    let segment = u32::from_be_bytes(key[4..8].try_into().expect("4 字节"));
    let first_state = u64::from_be_bytes(key[8..16].try_into().expect("8 字节"));
    Some(
        PathBuf::from(format!("{node:08x}"))
            .join(format!("{segment:08x}"))
            .join(format!("{first_state:016x}-{}.blk", hexadecimal(&key[16..32]))),
    )
}

fn temporary_path_of(final_path: &Path) -> PathBuf {
    let mut name = final_path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

/// 文件头 48 字节：魔数｜版本 u32 小端｜块键｜块值长度 u32 小端｜CRC-32C（覆盖头的前 44 字节加整块值）。
fn block_file_header(key: &BlockKey, value: &[u8]) -> [u8; FILE_HEADER_BYTES] {
    let mut header = [0u8; FILE_HEADER_BYTES];
    header[0..4].copy_from_slice(FILE_MAGIC);
    header[4..8].copy_from_slice(&FILE_FORMAT_VERSION.to_le_bytes());
    header[8..40].copy_from_slice(key);
    let value_length = u32::try_from(value.len()).expect("块值装得进 u32");
    header[40..44].copy_from_slice(&value_length.to_le_bytes());
    let checksum = crc32c(&[&header[..FILE_HEADER_CHECKSUMMED_BYTES], value]);
    header[44..48].copy_from_slice(&checksum.to_le_bytes());
    header
}

/// 读路径的校验：长度、魔数、版本、头里的键、长度字段、CRC 任一不对就是读报错。
fn validate_block_file(bytes: &[u8], key: &BlockKey) -> Result<Vec<u8>, String> {
    if bytes.len() < FILE_HEADER_BYTES {
        return Err(format!("文件只有 {} 字节，短于文件头", bytes.len()));
    }
    if &bytes[0..4] != FILE_MAGIC {
        return Err("魔数不对".to_string());
    }
    if bytes[4..8] != FILE_FORMAT_VERSION.to_le_bytes() {
        return Err("版本不对".to_string());
    }
    if &bytes[8..40] != key {
        return Err("头里的键与文件名不符".to_string());
    }
    let value_length = u32::from_le_bytes(bytes[40..44].try_into().expect("4 字节"));
    let value = &bytes[FILE_HEADER_BYTES..];
    if u64::from(value_length) != u64::try_from(value.len()).expect("装得进 u64") {
        return Err(format!("长度字段 {value_length} 与文件里的 {} 字节块值不符", value.len()));
    }
    let stored_checksum = u32::from_le_bytes(bytes[44..48].try_into().expect("4 字节"));
    if crc32c(&[&bytes[..FILE_HEADER_CHECKSUMMED_BYTES], value]) != stored_checksum {
        return Err("CRC-32C 不对".to_string());
    }
    Ok(value.to_vec())
}

impl FileLibrary {
    /// 打开：根目录在、FORMAT 在且对，否则打开失败；打开时删掉残留的 `*.tmp`。
    /// allow_create：根目录不在时（写入用）先经 `<根>.creating` 建好 FORMAT、改名成根目录。
    fn open(root: &Path, write_path: WritePath, allow_create: bool) -> Result<FileLibrary, StoreError> {
        if !root.exists() {
            if !allow_create {
                return Err(StoreError::File(format!("根目录 {} 不在", root.display())));
            }
            Self::create_root_atomically(root)?;
        }
        let format_bytes = fs::read(root.join(FORMAT_FILE_NAME)).map_err(|error| file_error("读 FORMAT", error))?;
        if format_bytes[..] != format_file_contents()[..] {
            return Err(StoreError::File("FORMAT 不对".to_string()));
        }
        let mut known_directories = HashSet::new();
        for node_entry in fs::read_dir(root).map_err(|error| file_error("列根目录", error))? {
            let node_path = node_entry.map_err(|error| file_error("列根目录", error))?.path();
            if !node_path.is_dir() {
                continue;
            }
            known_directories.insert(node_path.clone());
            for segment_entry in fs::read_dir(&node_path).map_err(|error| file_error("列节点目录", error))? {
                let segment_path = segment_entry.map_err(|error| file_error("列节点目录", error))?.path();
                if !segment_path.is_dir() {
                    continue;
                }
                known_directories.insert(segment_path.clone());
                for block_entry in fs::read_dir(&segment_path).map_err(|error| file_error("列段目录", error))? {
                    let block_path = block_entry.map_err(|error| file_error("列段目录", error))?.path();
                    if block_path.extension().is_some_and(|extension| extension == "tmp") {
                        fs::remove_file(&block_path).map_err(|error| file_error("删残留的临时文件", error))?;
                    }
                }
            }
        }
        Ok(FileLibrary {
            root: root.to_path_buf(),
            write_path,
            pending_renames: Vec::new(),
            confirmations_in_this_run: 0,
            remaining_steps: None,
            known_directories,
        })
    }

    fn create_root_atomically(root: &Path) -> Result<(), StoreError> {
        let mut creating_name = root.as_os_str().to_owned();
        creating_name.push(".creating");
        let creating = PathBuf::from(creating_name);
        if creating.exists() {
            fs::remove_dir_all(&creating).map_err(|error| file_error("删上一次没建完的根目录", error))?;
        }
        fs::create_dir_all(&creating).map_err(|error| file_error("建根目录", error))?;
        let mut format_file = File::create(creating.join(FORMAT_FILE_NAME)).map_err(|error| file_error("建 FORMAT", error))?;
        format_file.write_all(&format_file_contents()).map_err(|error| file_error("写 FORMAT", error))?;
        format_file.sync_all().map_err(|error| file_error("fsync FORMAT", error))?;
        sync_directory(&creating)?;
        fs::rename(&creating, root).map_err(|error| file_error("根目录改名", error))?;
        let parent = root.parent().expect("根目录有父目录");
        sync_directory(parent)
    }

    /// 单测的「做到第 k 步就停」：每一步文件系统改动之前扣一步，扣光了就当被杀。
    fn take_step(&mut self) -> Result<(), StoreError> {
        match self.remaining_steps {
            None => Ok(()),
            Some(0) => Err(StoreError::AbandonedAtStep),
            Some(remaining) => {
                self.remaining_steps = Some(remaining - 1);
                Ok(())
            }
        }
    }

    /// 节点与段目录不在就建，建完对新目录与它的父目录各 fsync。
    fn ensure_directory(&mut self, directory: &Path) -> Result<(), StoreError> {
        if self.known_directories.contains(directory) {
            return Ok(());
        }
        let parent = directory.parent().expect("目录有父目录").to_path_buf();
        if parent != self.root {
            self.ensure_directory(&parent)?;
        }
        if !directory.exists() {
            self.take_step()?;
            fs::create_dir(directory).map_err(|error| file_error("建目录", error))?;
            self.take_step()?;
            sync_directory(directory)?;
            self.take_step()?;
            sync_directory(&parent)?;
        }
        self.known_directories.insert(directory.to_path_buf());
        Ok(())
    }

    fn final_path_of(&self, key: &BlockKey) -> PathBuf {
        self.root.join(block_file_relative_path(key).expect("块键 32 字节"))
    }

    fn put_blocks(&mut self, blocks: &[(BlockKey, Vec<u8>)]) -> Result<(), StoreError> {
        match self.write_path {
            WritePath::Registered => self.put_blocks_registered(blocks),
            WritePath::FileLazyRename => self.put_blocks_lazy_rename(blocks),
            WritePath::FileDirectInPlace => self.put_blocks_direct_in_place(blocks),
            WritePath::RedbDurabilityNone | WritePath::RocksDatabaseWithoutWriteAheadLog => {
                panic!("F 臂不用 {} 这种写法", self.write_path.label())
            }
        }
    }

    /// 写临时文件（一次 write_all）→ sync_all；全部写完再逐个改名；再对碰到的每个目录 fsync 一次。
    /// 一块时就是登记的「临时文件 → fsync → 改名 → 目录 fsync」。
    fn put_blocks_registered(&mut self, blocks: &[(BlockKey, Vec<u8>)]) -> Result<(), StoreError> {
        let mut renames = Vec::with_capacity(blocks.len());
        for (key, value) in blocks {
            let final_path = self.final_path_of(key);
            let directory = final_path.parent().expect("块文件有父目录").to_path_buf();
            self.ensure_directory(&directory)?;
            let temporary_path = temporary_path_of(&final_path);
            self.write_synced_temporary_file(&temporary_path, key, value)?;
            renames.push((temporary_path, final_path));
        }
        self.rename_and_sync_directories(&renames)
    }

    fn write_synced_temporary_file(&mut self, temporary_path: &Path, key: &BlockKey, value: &[u8]) -> Result<(), StoreError> {
        let mut contents = Vec::with_capacity(FILE_HEADER_BYTES + value.len());
        contents.extend_from_slice(&block_file_header(key, value));
        contents.extend_from_slice(value);
        self.take_step()?;
        let mut handle = File::create(temporary_path).map_err(|error| file_error("建临时文件", error))?;
        self.take_step()?;
        handle.write_all(&contents).map_err(|error| file_error("写临时文件", error))?;
        self.take_step()?;
        handle.sync_all().map_err(|error| file_error("fsync 临时文件", error))
    }

    fn rename_and_sync_directories(&mut self, renames: &[(PathBuf, PathBuf)]) -> Result<(), StoreError> {
        let mut touched = BTreeSet::new();
        for (temporary_path, final_path) in renames {
            self.take_step()?;
            fs::rename(temporary_path, final_path).map_err(|error| file_error("改名", error))?;
            touched.insert(final_path.parent().expect("块文件有父目录").to_path_buf());
        }
        for directory in touched {
            self.take_step()?;
            sync_directory(&directory)?;
        }
        Ok(())
    }

    fn put_blocks_lazy_rename(&mut self, blocks: &[(BlockKey, Vec<u8>)]) -> Result<(), StoreError> {
        for (key, value) in blocks {
            let final_path = self.final_path_of(key);
            let directory = final_path.parent().expect("块文件有父目录").to_path_buf();
            self.ensure_directory(&directory)?;
            let temporary_path = temporary_path_of(&final_path);
            self.write_synced_temporary_file(&temporary_path, key, value)?;
            self.pending_renames.push((temporary_path, final_path));
            self.confirmations_in_this_run += 1;
            if self.confirmations_in_this_run % LAZY_RENAME_EVERY_CONFIRMATIONS == 0 {
                let pending = std::mem::take(&mut self.pending_renames);
                self.rename_and_sync_directories(&pending)?;
            }
        }
        Ok(())
    }

    fn put_blocks_direct_in_place(&mut self, blocks: &[(BlockKey, Vec<u8>)]) -> Result<(), StoreError> {
        for (key, value) in blocks {
            let final_path = self.final_path_of(key);
            let directory = final_path.parent().expect("块文件有父目录").to_path_buf();
            self.ensure_directory(&directory)?;
            self.take_step()?;
            let mut handle = File::create(&final_path).map_err(|error| file_error("建终名", error))?;
            self.take_step()?;
            handle.write_all(&block_file_header(key, value)).map_err(|error| file_error("写文件头", error))?;
            for chunk in value.chunks(DIRECT_WRITE_CHUNK_BYTES) {
                std::thread::sleep(Duration::from_micros(DIRECT_WRITE_PAUSE_MICROSECONDS));
                self.take_step()?;
                handle.write_all(chunk).map_err(|error| file_error("写块值", error))?;
            }
        }
        Ok(())
    }

    fn read_block(&self, key: &BlockKey) -> ReadOutcome {
        match fs::read(self.final_path_of(key)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => ReadOutcome::NotFound,
            Err(error) => ReadOutcome::ReadError(error.to_string()),
            Ok(bytes) => match validate_block_file(&bytes, key) {
                Ok(value) => ReadOutcome::Found(value),
                Err(reason) => ReadOutcome::ReadError(reason),
            },
        }
    }

    /// 走目录收全部 `.blk` 文件名（`*.tmp` 与别的文件不算）；键从路径拆回，块值长度 = 文件长度 − 48。
    fn enumerate_keys_with_value_lengths(&self) -> Result<Vec<(Vec<u8>, u64)>, StoreError> {
        let mut entries = Vec::new();
        for node_entry in fs::read_dir(&self.root).map_err(|error| file_error("列根目录", error))? {
            let node_path = node_entry.map_err(|error| file_error("列根目录", error))?.path();
            let Some(node) = hexadecimal_name_as_u32(&node_path) else { continue };
            for segment_entry in fs::read_dir(&node_path).map_err(|error| file_error("列节点目录", error))? {
                let segment_path = segment_entry.map_err(|error| file_error("列节点目录", error))?.path();
                let Some(segment) = hexadecimal_name_as_u32(&segment_path) else { continue };
                for block_entry in fs::read_dir(&segment_path).map_err(|error| file_error("列段目录", error))? {
                    let block_entry = block_entry.map_err(|error| file_error("列段目录", error))?;
                    let name = block_entry.file_name().to_string_lossy().to_string();
                    let Some(stem) = name.strip_suffix(".blk") else { continue };
                    let Some((first_state_text, fingerprint_text)) = stem.split_once('-') else { continue };
                    let (Ok(first_state), Some(fingerprint)) =
                        (u64::from_str_radix(first_state_text, 16), parse_hexadecimal(fingerprint_text))
                    else {
                        continue;
                    };
                    let mut key = Vec::with_capacity(BLOCK_KEY_BYTES);
                    key.extend_from_slice(&node.to_be_bytes());
                    key.extend_from_slice(&segment.to_be_bytes());
                    key.extend_from_slice(&first_state.to_be_bytes());
                    key.extend_from_slice(&fingerprint);
                    let file_length = block_entry.metadata().map_err(|error| file_error("读文件长度", error))?.len();
                    entries.push((key, file_length.saturating_sub(u64::try_from(FILE_HEADER_BYTES).expect("48"))));
                }
            }
        }
        Ok(entries)
    }
}

fn hexadecimal_name_as_u32(path: &Path) -> Option<u32> {
    if !path.is_dir() {
        return None;
    }
    let name = path.file_name()?.to_str()?;
    if name.len() != 8 {
        return None;
    }
    u32::from_str_radix(name, 16).ok()
}

fn parse_hexadecimal(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).ok())
        .collect()
}

/// 库目录下全部普通文件的 st_blocks × 512 之和（登记 1.1）；走的时候消失的文件（改名中的临时文件）不算。
fn library_occupancy_bytes(directory: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt;
    let Ok(entries) = fs::read_dir(directory) else { return 0 };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let Ok(metadata) = fs::symlink_metadata(entry.path()) else { continue };
        if metadata.is_dir() {
            total += library_occupancy_bytes(&entry.path());
        } else if metadata.is_file() {
            total += metadata.blocks() * 512;
        }
    }
    total
}

// ============================== 逐块核（登记 5.1 的表） ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockClass {
    /// 收到过 `C`。
    Confirmed,
    /// 收到过 `B`、没收到过 `C`。
    InFlight,
    /// 没收到过 `B`。
    NotStarted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadKind {
    MatchingBytes,
    DifferentBytes,
    NotFound,
    ReadError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockVerdict {
    Correct,
    /// Q1a
    LostConfirmed,
    /// Q1b
    SilentlyCorrupted,
    /// Q1c
    ConfirmedReadError,
    /// Q1d
    UnconfirmedReadError,
    /// Q1e
    Phantom,
}

fn classify_block(class: BlockClass, read: ReadKind) -> BlockVerdict {
    match (class, read) {
        (BlockClass::Confirmed, ReadKind::MatchingBytes) => BlockVerdict::Correct,
        (BlockClass::Confirmed, ReadKind::DifferentBytes) => BlockVerdict::SilentlyCorrupted,
        (BlockClass::Confirmed, ReadKind::NotFound) => BlockVerdict::LostConfirmed,
        (BlockClass::Confirmed, ReadKind::ReadError) => BlockVerdict::ConfirmedReadError,
        (BlockClass::InFlight, ReadKind::MatchingBytes) => BlockVerdict::Correct,
        (BlockClass::InFlight, ReadKind::DifferentBytes) => BlockVerdict::SilentlyCorrupted,
        (BlockClass::InFlight, ReadKind::NotFound) => BlockVerdict::Correct,
        (BlockClass::InFlight, ReadKind::ReadError) => BlockVerdict::UnconfirmedReadError,
        (BlockClass::NotStarted, ReadKind::MatchingBytes | ReadKind::DifferentBytes) => BlockVerdict::Phantom,
        (BlockClass::NotStarted, ReadKind::NotFound) => BlockVerdict::Correct,
        (BlockClass::NotStarted, ReadKind::ReadError) => BlockVerdict::UnconfirmedReadError,
    }
}

fn values_match(found: &[u8], expected: &[u8]) -> bool {
    found.len() == expected.len() && found == expected
}

/// 核哪些块：一段连续的块号，或一张单子（S1-large 的抽核）。
#[derive(Debug, Clone, PartialEq, Eq)]
enum CheckedBlocks {
    Range { start: u64, end: u64 },
    Listed(Vec<u64>),
}

/// 一次核的计划。写入是顺序的，所以已确认的块是前缀 [0, confirmed_end)，在途的是 [confirmed_end, started_end)。
#[derive(Debug, Clone, PartialEq, Eq)]
struct VerificationPlan {
    seed: u64,
    pattern: ValuePattern,
    states_per_block: u64,
    confirmed_end: u64,
    started_end: u64,
    checked_blocks: CheckedBlocks,
}

impl VerificationPlan {
    fn class_of(&self, block_index: u64) -> BlockClass {
        if block_index < self.confirmed_end {
            BlockClass::Confirmed
        } else if block_index < self.started_end {
            BlockClass::InFlight
        } else {
            BlockClass::NotStarted
        }
    }

    fn checked_list(&self) -> Vec<u64> {
        match &self.checked_blocks {
            CheckedBlocks::Range { start, end } => (*start..*end).collect(),
            CheckedBlocks::Listed(blocks) => blocks.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct VerificationReport {
    open_failed: bool,
    open_error: String,
    /// 只有 K：第一次打开失败之后调 DB::repair 再开，能开就是 Some(true)。
    opened_after_repair: Option<bool>,
    open_microseconds: u64,
    confirmed_checked: u64,
    in_flight_checked: u64,
    not_started_checked: u64,
    lost_confirmed: Vec<u64>,
    silently_corrupted: Vec<u64>,
    confirmed_read_errors: Vec<u64>,
    unconfirmed_read_errors: Vec<u64>,
    /// 读到的未开始块与枚举出来而不属于任何已开始块的键（十六进制），去重。
    phantom_keys: BTreeSet<String>,
    enumerated_key_count: u64,
    enumerated_value_bytes: u64,
    enumeration_error: String,
    first_read_error: String,
}

/// 打开失败时记下来；K 调一次 DB::repair 再开（登记 5.2），别的臂不修。
fn record_open_failure_then_try_repair(
    candidate: Candidate,
    library: &Path,
    open_error: StoreError,
    report: &mut VerificationReport,
) -> Option<BlockStore> {
    report.open_failed = true;
    report.open_error = open_error.to_string();
    match candidate {
        Candidate::RocksDatabase => {
            let repaired = rocksdb::DB::repair(&rocksdb_options(), library)
                .map_err(rocksdb_error)
                .and_then(|()| open_existing_store(candidate, library));
            report.opened_after_repair = Some(repaired.is_ok());
            repaired.ok()
        }
        Candidate::RedbQuickRepair | Candidate::RedbDefault | Candidate::HardenedFile => None,
    }
}

fn verify_library(candidate: Candidate, library: &Path, plan: &VerificationPlan) -> VerificationReport {
    let mut report = VerificationReport::default();
    let open_started = Instant::now();
    let opened = open_existing_store(candidate, library);
    report.open_microseconds = elapsed_microseconds(open_started);
    let store = match opened {
        Ok(store) => store,
        Err(open_error) => match record_open_failure_then_try_repair(candidate, library, open_error, &mut report) {
            Some(repaired) => repaired,
            None => return report,
        },
    };
    let fingerprint = input_fingerprint();
    for block_index in plan.checked_list() {
        let class = plan.class_of(block_index);
        match class {
            BlockClass::Confirmed => report.confirmed_checked += 1,
            BlockClass::InFlight => report.in_flight_checked += 1,
            BlockClass::NotStarted => report.not_started_checked += 1,
        }
        let key = interleaved_block_key(block_index, plan.states_per_block, &fingerprint);
        let read = match store.read_block(&key) {
            ReadOutcome::Found(found) => {
                let expected = block_value(plan.pattern, plan.seed, block_index, plan.states_per_block);
                if values_match(&found, &expected) {
                    ReadKind::MatchingBytes
                } else {
                    ReadKind::DifferentBytes
                }
            }
            ReadOutcome::NotFound => ReadKind::NotFound,
            ReadOutcome::ReadError(reason) => {
                if report.first_read_error.is_empty() {
                    report.first_read_error = format!("block={block_index}:{reason}");
                }
                ReadKind::ReadError
            }
        };
        match classify_block(class, read) {
            BlockVerdict::Correct => {}
            BlockVerdict::LostConfirmed => report.lost_confirmed.push(block_index),
            BlockVerdict::SilentlyCorrupted => report.silently_corrupted.push(block_index),
            BlockVerdict::ConfirmedReadError => report.confirmed_read_errors.push(block_index),
            BlockVerdict::UnconfirmedReadError => report.unconfirmed_read_errors.push(block_index),
            BlockVerdict::Phantom => {
                report.phantom_keys.insert(hexadecimal(&key));
            }
        }
    }
    let started_keys: HashSet<BlockKey> = (0..plan.started_end)
        .map(|block_index| interleaved_block_key(block_index, plan.states_per_block, &fingerprint))
        .collect();
    let enumerated = store.enumerate_keys_with_value_lengths();
    match enumerated {
        Ok(entries) => {
            for (key, value_length) in entries {
                report.enumerated_key_count += 1;
                report.enumerated_value_bytes += value_length;
                let is_started = <[u8; BLOCK_KEY_BYTES]>::try_from(key.as_slice())
                    .is_ok_and(|fixed_key| started_keys.contains(&fixed_key));
                if !is_started {
                    report.phantom_keys.insert(hexadecimal(&key));
                }
            }
        }
        Err(error) => report.enumeration_error = error.to_string(),
    }
    report
}

fn elapsed_microseconds(since: Instant) -> u64 {
    u64::try_from(since.elapsed().as_micros()).expect("微秒数装得进 u64")
}

fn join_numbers(numbers: &[u64]) -> String {
    if numbers.is_empty() {
        "-".to_string()
    } else {
        numbers.iter().map(u64::to_string).collect::<Vec<_>>().join(",")
    }
}

fn split_numbers(text: &str) -> Vec<u64> {
    if text == "-" {
        Vec::new()
    } else {
        text.split(',').map(|number| number.parse().expect("逗号隔开的整数")).collect()
    }
}

/// 核对报告编成一行 `VERIFY k=v …`（核对子进程写，父进程读回）；文本字段里的空白换成下划线。
fn encode_verification_report(report: &VerificationReport) -> String {
    let phantom: Vec<String> = report.phantom_keys.iter().cloned().collect();
    let text = |value: &str| if value.is_empty() { "-".to_string() } else { value.split_whitespace().collect::<Vec<_>>().join("_") };
    let repaired = match report.opened_after_repair {
        None => "not_applicable",
        Some(true) => "yes",
        Some(false) => "no",
    };
    format!(
        "VERIFY open_failed={} opened_after_repair={repaired} open_microseconds={} confirmed_checked={} in_flight_checked={} not_started_checked={} lost={} corrupted={} confirmed_read_errors={} unconfirmed_read_errors={} phantom={} enumerated_keys={} enumerated_value_bytes={} enumeration_error={} first_read_error={} open_error={}",
        report.open_failed,
        report.open_microseconds,
        report.confirmed_checked,
        report.in_flight_checked,
        report.not_started_checked,
        join_numbers(&report.lost_confirmed),
        join_numbers(&report.silently_corrupted),
        join_numbers(&report.confirmed_read_errors),
        join_numbers(&report.unconfirmed_read_errors),
        if phantom.is_empty() { "-".to_string() } else { phantom.join(",") },
        report.enumerated_key_count,
        report.enumerated_value_bytes,
        text(&report.enumeration_error),
        text(&report.first_read_error),
        text(&report.open_error),
    )
}

fn decode_verification_report(line: &str) -> Option<VerificationReport> {
    let body = line.strip_prefix("VERIFY ")?;
    let fields: BTreeMap<&str, &str> = body.split(' ').filter_map(|field| field.split_once('=')).collect();
    let untext = |value: &str| if value == "-" { String::new() } else { value.to_string() };
    Some(VerificationReport {
        open_failed: fields.get("open_failed")? == &"true",
        open_error: untext(fields.get("open_error")?),
        opened_after_repair: match *fields.get("opened_after_repair")? {
            "yes" => Some(true),
            "no" => Some(false),
            _ => None,
        },
        open_microseconds: fields.get("open_microseconds")?.parse().ok()?,
        confirmed_checked: fields.get("confirmed_checked")?.parse().ok()?,
        in_flight_checked: fields.get("in_flight_checked")?.parse().ok()?,
        not_started_checked: fields.get("not_started_checked")?.parse().ok()?,
        lost_confirmed: split_numbers(fields.get("lost")?),
        silently_corrupted: split_numbers(fields.get("corrupted")?),
        confirmed_read_errors: split_numbers(fields.get("confirmed_read_errors")?),
        unconfirmed_read_errors: split_numbers(fields.get("unconfirmed_read_errors")?),
        phantom_keys: match *fields.get("phantom")? {
            "-" => BTreeSet::new(),
            listed => listed.split(',').map(str::to_string).collect(),
        },
        enumerated_key_count: fields.get("enumerated_keys")?.parse().ok()?,
        enumerated_value_bytes: fields.get("enumerated_value_bytes")?.parse().ok()?,
        enumeration_error: untext(fields.get("enumeration_error")?),
        first_read_error: untext(fields.get("first_read_error")?),
    })
}

// ============================== 子进程角色的参数 ==============================

/// 子进程参数一律写成 `键=值`。
struct RoleArguments {
    fields: BTreeMap<String, String>,
}

impl RoleArguments {
    fn parse(arguments: &[String]) -> RoleArguments {
        let fields = arguments
            .iter()
            .map(|argument| {
                let (key, value) = argument.split_once('=').unwrap_or_else(|| panic!("子进程参数 {argument} 不是 键=值"));
                (key.to_string(), value.to_string())
            })
            .collect();
        RoleArguments { fields }
    }

    fn text(&self, key: &str) -> &str {
        self.fields.get(key).unwrap_or_else(|| panic!("子进程少了参数 {key}"))
    }

    fn number(&self, key: &str) -> u64 {
        self.text(key).parse().unwrap_or_else(|_| panic!("子进程参数 {key} 不是整数"))
    }

    fn candidate(&self) -> Candidate {
        Candidate::parse(self.text("candidate")).expect("认得的臂")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteOrder {
    Interleaved,
    RandomPermutation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProgressLines {
    /// S1：每块写之前一行 `B <j>`、提交返回之后一行 `C <j>`。
    EveryBlock,
    /// S2：只在每写满 1 000 块时一行 `C <已写块数>`，给父进程排占用测量。
    EveryWindow,
}

/// 一次写一行、不经缓冲：锁住标准输出、整行 write_all、flush。
fn emit_progress_line(line: &str) {
    let mut standard_output = std::io::stdout().lock();
    standard_output.write_all(line.as_bytes()).expect("写得进标准输出");
    standard_output.flush().expect("刷得出标准输出");
}

fn sorted_percentile(sorted: &[u64], numerator: usize, denominator: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = (sorted.len() * numerator).div_ceil(denominator).max(1);
    sorted[rank - 1]
}

fn median_of(values: &[u64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted_percentile(&sorted, 1, 2)
}

/// A-raw：在 directory 里 1 000 次「追加 64 KiB（P-rand）+ fdatasync」，返回每次的微秒数。
fn raw_append_fdatasync_probe(directory: &Path) -> Vec<u64> {
    let probe_path = directory.join("e162-raw-probe.bin");
    let mut probe = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&probe_path)
        .expect("建得了 A-raw 探针文件");
    let mut latencies = Vec::with_capacity(RAW_PROBE_APPENDS);
    for append_index in 0..RAW_PROBE_APPENDS {
        let states = u64::try_from(RAW_PROBE_APPEND_BYTES).expect("64 KiB");
        let payload = block_value(ValuePattern::Random, SEED_READ, u64::try_from(append_index).expect("小于 1000"), states);
        let started = Instant::now();
        probe.write_all(&payload).expect("写得进 A-raw 探针文件");
        probe.sync_data().expect("fdatasync 得了 A-raw 探针文件");
        latencies.push(elapsed_microseconds(started));
    }
    drop(probe);
    fs::remove_file(&probe_path).expect("删得了 A-raw 探针文件");
    latencies
}

/// 写入子进程：`writer candidate= write_path= library= seed= pattern= states= order= batch= first= end= sleep= timed= probe= progress=`。
/// end=0 是一直写到被杀。
fn run_writer_role(arguments: &[String]) -> i32 {
    let role = RoleArguments::parse(arguments);
    let candidate = role.candidate();
    let write_path = WritePath::parse(role.text("write_path"));
    let library = PathBuf::from(role.text("library"));
    let seed = role.number("seed");
    let pattern = ValuePattern::parse(role.text("pattern"));
    let states_per_block = role.number("states");
    let order = match role.text("order") {
        "interleaved" => WriteOrder::Interleaved,
        "random" => WriteOrder::RandomPermutation,
        other => panic!("认不出的次序 {other}"),
    };
    let batch = usize::try_from(role.number("batch")).expect("批大小装得进 usize");
    let first_position = role.number("first");
    let end_position = match role.number("end") {
        0 => u64::MAX,
        end => end,
    };
    let sleep_microseconds = role.number("sleep");
    let timed = role.number("timed") == 1;
    let progress = match role.text("progress") {
        "every-block" => ProgressLines::EveryBlock,
        "every-window" => ProgressLines::EveryWindow,
        other => panic!("认不出的进度行形态 {other}"),
    };
    if role.number("probe") == 1 {
        let parent = library.parent().expect("库有父目录");
        let latencies = raw_append_fdatasync_probe(parent);
        emit_progress_line(&format!("P raw_median_microseconds={}\n", median_of(&latencies)));
    }
    let open_started = Instant::now();
    let mut store = match open_store_for_writing(candidate, write_path, &library) {
        Ok(store) => store,
        Err(error) => {
            emit_progress_line(&format!("E open {}\n", error.to_string().replace(char::is_whitespace, "_")));
            return 3;
        }
    };
    emit_progress_line(&format!("O {}\n", elapsed_microseconds(open_started)));
    let (sender, receiver) = mpsc::sync_channel::<(u64, BlockKey, Vec<u8>)>(GENERATOR_CHANNEL_DEPTH);
    let generator = std::thread::spawn(move || {
        let fingerprint = input_fingerprint();
        let permutation = match order {
            WriteOrder::Interleaved => Vec::new(),
            WriteOrder::RandomPermutation => random_permutation(end_position, SEED_ORDER),
        };
        let mut position = first_position;
        while position < end_position {
            let block_index = match order {
                WriteOrder::Interleaved => position,
                WriteOrder::RandomPermutation => permutation[usize::try_from(position).expect("位置装得进 usize")],
            };
            let key = interleaved_block_key(block_index, states_per_block, &fingerprint);
            let value = block_value(pattern, seed, block_index, states_per_block);
            if sender.send((block_index, key, value)).is_err() {
                return;
            }
            position += 1;
        }
    });
    let mut commit_latencies: Vec<u64> = Vec::new();
    let mut block_return_offsets: Vec<u64> = Vec::new();
    let mut first_commit_started: Option<Instant> = None;
    let mut written_blocks = 0u64;
    loop {
        let mut batch_blocks: Vec<(u64, BlockKey, Vec<u8>)> = Vec::with_capacity(batch);
        while batch_blocks.len() < batch {
            match receiver.recv() {
                Ok(item) => batch_blocks.push(item),
                Err(_) => break,
            }
        }
        if batch_blocks.is_empty() {
            break;
        }
        if progress == ProgressLines::EveryBlock {
            for (block_index, _, _) in &batch_blocks {
                emit_progress_line(&format!("B {block_index}\n"));
            }
        }
        let pairs: Vec<(BlockKey, Vec<u8>)> = batch_blocks.iter().map(|(_, key, value)| (*key, value.clone())).collect();
        let commit_started = Instant::now();
        let origin = *first_commit_started.get_or_insert(commit_started);
        if sleep_microseconds > 0 {
            std::thread::sleep(Duration::from_micros(sleep_microseconds));
        }
        let committed = store.put_blocks(&pairs);
        let commit_returned = Instant::now();
        if let Err(error) = committed {
            emit_progress_line(&format!("E commit {}\n", error.to_string().replace(char::is_whitespace, "_")));
            return 3;
        }
        commit_latencies.push(u64::try_from((commit_returned - commit_started).as_micros()).expect("装得进 u64"));
        let return_offset = u64::try_from((commit_returned - origin).as_micros()).expect("装得进 u64");
        for (block_index, _, _) in &batch_blocks {
            written_blocks += 1;
            block_return_offsets.push(return_offset);
            match progress {
                ProgressLines::EveryBlock => emit_progress_line(&format!("C {block_index}\n")),
                ProgressLines::EveryWindow => {
                    if written_blocks % WINDOW_BLOCKS == 0 {
                        emit_progress_line(&format!("C {written_blocks}\n"));
                    }
                }
            }
        }
    }
    generator.join().expect("生成线程没有 panic");
    if timed {
        for line in timing_summary_lines(&commit_latencies, &block_return_offsets, states_per_block) {
            emit_progress_line(&format!("{line}\n"));
        }
    }
    drop(store);
    emit_progress_line("D\n");
    0
}

/// 计时汇总：`T` 一行（Q2a、提交时延分布、Q2c 在几个产出速率下的最大积压），`W` 每窗一行（Q2b）。
fn timing_summary_lines(commit_latencies: &[u64], block_return_offsets: &[u64], states_per_block: u64) -> Vec<String> {
    let block_count = u64::try_from(block_return_offsets.len()).expect("装得进 u64");
    let elapsed = block_return_offsets.last().copied().unwrap_or(0).max(1);
    let blocks_per_second = block_count as f64 * 1e6 / elapsed as f64;
    let states_per_second = blocks_per_second * states_per_block as f64;
    let mut sorted = commit_latencies.to_vec();
    sorted.sort_unstable();
    let mut lines = vec![format!(
        "T blocks={block_count} commits={} elapsed_microseconds={elapsed} blocks_per_second={blocks_per_second:.3} states_per_second={states_per_second:.1} commit_median_microseconds={} commit_percentile_99_microseconds={} commit_maximum_microseconds={}",
        commit_latencies.len(),
        sorted_percentile(&sorted, 1, 2),
        sorted_percentile(&sorted, 99, 100),
        sorted.last().copied().unwrap_or(0),
    )];
    for fraction_percent in [50u64, 75, 90, 100] {
        let production_states_per_microsecond = states_per_second * fraction_percent as f64 / 100.0 / 1e6;
        let mut maximum_backlog_states = 0f64;
        let mut peak_block = 0u64;
        for (consumed_before, &return_offset) in block_return_offsets.iter().enumerate() {
            let produced = production_states_per_microsecond * return_offset as f64;
            let consumed = consumed_before as f64 * states_per_block as f64;
            if produced - consumed > maximum_backlog_states {
                maximum_backlog_states = produced - consumed;
                peak_block = u64::try_from(consumed_before).expect("装得进 u64") + 1;
            }
        }
        lines.push(format!(
            "Q backlog_rate_percent_of_measured={fraction_percent} maximum_backlog_states={maximum_backlog_states:.0} peak_at_block={peak_block}"
        ));
    }
    let mut window_start = 0u64;
    for window_index in 0..block_count / WINDOW_BLOCKS {
        let last_block = usize::try_from((window_index + 1) * WINDOW_BLOCKS - 1).expect("装得进 usize");
        let window_end = block_return_offsets[last_block];
        let window_elapsed = (window_end - window_start).max(1);
        let window_states_per_second = (WINDOW_BLOCKS * states_per_block) as f64 * 1e6 / window_elapsed as f64;
        lines.push(format!(
            "W window={window_index} end_microseconds={window_end} states_per_second={window_states_per_second:.1}"
        ));
        window_start = window_end;
    }
    lines
}

// ============================== 读子进程与核对子进程 ==============================

fn process_read_bytes() -> u64 {
    let text = fs::read_to_string("/proc/self/io").unwrap_or_default();
    text.lines()
        .find_map(|line| line.strip_prefix("read_bytes: "))
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0)
}

/// 读子进程：`read candidate= library= seed= pattern= states= blocks= count= sleep= passes=`。
/// 块键按 seed_READ 在 [0, blocks) 里均匀取 count 个；每次一次读调用连同整块值拷进自己的缓冲区，计时只包这一段
/// （sleep > 0 时计时区间里、读调用之前再睡这么久，给 PC-read）。passes=2 是冷一遍再在同一个打开的库上热一遍。
fn run_read_role(arguments: &[String]) -> i32 {
    let role = RoleArguments::parse(arguments);
    let candidate = role.candidate();
    let library = PathBuf::from(role.text("library"));
    let seed = role.number("seed");
    let pattern = ValuePattern::parse(role.text("pattern"));
    let states_per_block = role.number("states");
    let block_count = role.number("blocks");
    let read_count = usize::try_from(role.number("count")).expect("装得进 usize");
    let sleep_microseconds = role.number("sleep");
    let passes = role.number("passes");
    let open_started = Instant::now();
    let store = match open_existing_store(candidate, &library) {
        Ok(store) => store,
        Err(error) => {
            emit_progress_line(&format!("E open {}\n", error.to_string().replace(char::is_whitespace, "_")));
            return 3;
        }
    };
    let open_microseconds = elapsed_microseconds(open_started);
    let fingerprint = input_fingerprint();
    let mut generator = SplitMix64::new(SEED_READ);
    let chosen: Vec<u64> = (0..read_count).map(|_| generator.next_output() % block_count).collect();
    let distinct = chosen.iter().collect::<HashSet<_>>().len();
    for pass_index in 0..passes {
        let read_bytes_before = process_read_bytes();
        let mut latencies = Vec::with_capacity(read_count);
        let mut own_buffer: Vec<u8> = Vec::new();
        let mut mismatches = 0u64;
        for &block_index in &chosen {
            let key = interleaved_block_key(block_index, states_per_block, &fingerprint);
            let started = Instant::now();
            if sleep_microseconds > 0 {
                std::thread::sleep(Duration::from_micros(sleep_microseconds));
            }
            let outcome = store.read_block(&key);
            if let ReadOutcome::Found(value) = &outcome {
                own_buffer.clear();
                own_buffer.extend_from_slice(value);
            }
            latencies.push(elapsed_microseconds(started));
            let expected = block_value(pattern, seed, block_index, states_per_block);
            if !matches!(outcome, ReadOutcome::Found(_)) || own_buffer != expected {
                mismatches += 1;
            }
        }
        let read_bytes_delta = process_read_bytes().saturating_sub(read_bytes_before);
        let mut sorted = latencies.clone();
        sorted.sort_unstable();
        let pass_label = if pass_index == 0 { "first" } else { "second" };
        emit_progress_line(&format!(
            "R pass={pass_label} reads={read_count} distinct_blocks={distinct} open_microseconds={open_microseconds} median_microseconds={} percentile_99_microseconds={} maximum_microseconds={} mismatches={mismatches} read_bytes_delta={read_bytes_delta}\n",
            sorted_percentile(&sorted, 1, 2),
            sorted_percentile(&sorted, 99, 100),
            sorted.last().copied().unwrap_or(0),
        ));
    }
    0
}

/// 核对子进程：`verify candidate= library= seed= pattern= states= confirmed_end= started_end= checked=`，
/// checked 是 `range:<起>:<止>` 或 `list:<逗号隔开的块号>`。
fn run_verify_role(arguments: &[String]) -> i32 {
    let role = RoleArguments::parse(arguments);
    let checked_text = role.text("checked");
    let checked_blocks = if let Some(range) = checked_text.strip_prefix("range:") {
        let (start, end) = range.split_once(':').expect("range:<起>:<止>");
        CheckedBlocks::Range {
            start: start.parse().expect("起点是整数"),
            end: end.parse().expect("止点是整数"),
        }
    } else {
        CheckedBlocks::Listed(split_numbers(checked_text.strip_prefix("list:").expect("list:<块号>")))
    };
    let plan = VerificationPlan {
        seed: role.number("seed"),
        pattern: ValuePattern::parse(role.text("pattern")),
        states_per_block: role.number("states"),
        confirmed_end: role.number("confirmed_end"),
        started_end: role.number("started_end"),
        checked_blocks,
    };
    let report = verify_library(role.candidate(), Path::new(role.text("library")), &plan);
    emit_progress_line(&format!("{}\n", encode_verification_report(&report)));
    0
}

// ============================== 父进程：起子进程、读它的行 ==============================

/// 起这个 bin 的一个子进程角色。单测里 current_exe 是测试二进制，改走 libtest 自起一个测试函数（见 tests 模块）。
#[cfg(not(test))]
fn child_command(role_arguments: &[String]) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("找得到自己的可执行文件"));
    command.args(role_arguments);
    command
}

#[cfg(test)]
fn child_command(role_arguments: &[String]) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("找得到测试二进制"));
    command
        .args(["--exact", "tests::child_process_role_entry", "--nocapture", "--test-threads=1", "--quiet"])
        .env(tests::CHILD_ROLE_ENVIRONMENT, role_arguments.join("\u{1f}"));
    command
}

fn spawn_child(role_arguments: &[String]) -> (Child, mpsc::Receiver<String>) {
    let mut child = child_command(role_arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("起得了子进程");
    let standard_output = child.stdout.take().expect("子进程的标准输出接在管道上");
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(standard_output).lines() {
            let Ok(line) = line else { break };
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    (child, receiver)
}

/// 对子进程的 pid 发 SIGKILL（`Child::kill` 在 Unix 上发的就是 SIGKILL），再收掉它。
fn kill_and_reap(child: &mut Child) {
    let _already_exited = child.kill();
    child.wait().expect("收得了被杀的子进程");
}

/// 子进程跑完（或被杀）之后把剩下的行读到 EOF。
fn drain_lines(receiver: &mpsc::Receiver<String>) -> Vec<String> {
    receiver.iter().collect()
}

fn run_child_to_completion(role_arguments: &[String]) -> (Vec<String>, i32) {
    let (mut child, receiver) = spawn_child(role_arguments);
    let lines = drain_lines(&receiver);
    let status = child.wait().expect("收得了子进程");
    (lines, status.code().unwrap_or(-1))
}

fn field_value<'line>(line: &'line str, key: &str) -> Option<&'line str> {
    line.split(' ').find_map(|field| field.strip_prefix(key).and_then(|rest| rest.strip_prefix('=')))
}

// ============================== 盘上位置与环境核查 ==============================

/// `${TMPDIR:-/tmp}/e162-<pid>/`，Drop 时整目录删掉；E162_KEEP_SCRATCH=1 时留下并打印路径。
struct ScratchDirectory {
    path: PathBuf,
}

impl ScratchDirectory {
    fn create() -> ScratchDirectory {
        let base = std::env::var_os("TMPDIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
        let path = base.join(format!("e162-{}", std::process::id()));
        fs::create_dir_all(&path).expect("建得了盘上位置");
        ScratchDirectory { path }
    }
}

impl Drop for ScratchDirectory {
    fn drop(&mut self) {
        if std::env::var("E162_KEEP_SCRATCH").is_ok_and(|value| value == "1") {
            eprintln!("E162_KEEP_SCRATCH=1：现场留在 {}", self.path.display());
        } else {
            let _cleanup_outcome = fs::remove_dir_all(&self.path);
        }
    }
}

fn command_output_text(program: &str, arguments: &[&str]) -> String {
    Command::new(program)
        .args(arguments)
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default()
}

/// V9：`stat -f -c %T`；H4：`df -B1` 的可用字节。
fn filesystem_type_and_available_bytes(directory: &Path) -> (String, u64) {
    let directory_text = directory.to_string_lossy().to_string();
    let filesystem_type = command_output_text("stat", &["-f", "-c", "%T", &directory_text]);
    let available_text = command_output_text("df", &["-B1", "--output=avail", &directory_text]);
    let available = available_text.lines().last().and_then(|line| line.trim().parse().ok()).unwrap_or(0);
    (filesystem_type, available)
}

/// V15：本用户的进程里有没有会干扰计时的（按可执行文件名与参数逐个比，不用模式串比整条命令行）。
fn interfering_processes() -> Vec<String> {
    use std::os::unix::fs::MetadataExt;
    let own_user = fs::metadata("/proc/self").map(|metadata| metadata.uid()).unwrap_or(u32::MAX);
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else { return found };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        if fs::metadata(entry.path()).map(|metadata| metadata.uid()).ok() != Some(own_user) {
            continue;
        }
        let Ok(command_line) = fs::read(entry.path().join("cmdline")) else { continue };
        let words: Vec<String> = command_line
            .split(|&byte| byte == 0)
            .filter(|word| !word.is_empty())
            .map(|word| String::from_utf8_lossy(word).to_string())
            .collect();
        let Some(program) = words.first() else { continue };
        let program_name = Path::new(program).file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default();
        let interferes = program_name.starts_with("qemu-system")
            || program_name == "cargo"
            || program_name == "rustc"
            || program_name == "fio"
            || program_name == "e152-file-system-benchmark"
            || words.iter().any(|word| word.ends_with("vm-bench.sh") || word.contains("e161"));
        if interferes {
            found.push(format!("{name}:{program_name}"));
        }
    }
    found.sort();
    found
}

fn load_average_text() -> String {
    fs::read_to_string("/proc/loadavg").unwrap_or_default().split_whitespace().take(3).collect::<Vec<_>>().join("/")
}

/// /proc/diskstats 里 nvme0n1 的写扇区数（第 10 列）。
fn nvme_written_sectors() -> u64 {
    fs::read_to_string("/proc/diskstats")
        .unwrap_or_default()
        .lines()
        .find_map(|line| {
            let columns: Vec<&str> = line.split_whitespace().collect();
            (columns.get(2) == Some(&"nvme0n1")).then(|| columns.get(9).and_then(|value| value.parse().ok())).flatten()
        })
        .unwrap_or(0)
}

// ============================== 杀进程驱动（S1 主格、S1-large、PC-L、PC-T 共用） ==============================

/// 一串杀进程的趟：同一个库，写入进程每趟从「最后一个已确认块的下一块」写起。
struct KillCampaign {
    label: String,
    candidate: Candidate,
    write_path: WritePath,
    library: PathBuf,
    seed: u64,
    /// 每块提交时延中位 m（校准趟）。
    commit_median_microseconds: u64,
    /// 前缀：[0, confirmed_end) 已确认，[confirmed_end, started_end) 在途。
    confirmed_end: u64,
    started_end: u64,
    /// 这条臂此前各次写入子进程自报的打开用时（给打开中途的杀点）。
    open_times_microseconds: Vec<u64>,
    /// S1-large：每次杀后怎么挑要核的块（None 是核 [0, started_end + 64) 全部）。
    large_library_sampling: Option<u64>,
    cumulative_lost: BTreeSet<u64>,
    cumulative_corrupted: BTreeSet<u64>,
    cumulative_confirmed_read_errors: BTreeSet<u64>,
    cumulative_unconfirmed_read_errors: BTreeSet<u64>,
    cumulative_phantoms: BTreeSet<String>,
    open_failures: u64,
    verification_timeouts: u64,
    repaired_openings: u64,
    kills_inside_commit: u64,
    write_phase_kills: u64,
    confirmations_before_write_kills: u64,
    reopen_microseconds: Vec<u64>,
    /// 每次核里 Q1a 或 Q1c 或 Q1d 或 Q1e 有没有（按次数算的，给 PC-L、PC-T）。
    write_kills_with_loss: u64,
    write_kills_with_new_loss: u64,
    write_kills_with_read_error: u64,
    /// Q1a–Q1f 各自在第几次杀后第一次涨、涨过几次核（轨迹）。
    first_rise: BTreeMap<&'static str, usize>,
    rises: BTreeMap<&'static str, u64>,
    assertion_failures: Vec<String>,
    stopped_after_unopenable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LastProgressLine {
    None,
    Open,
    Begin,
    Commit,
}

impl KillCampaign {
    fn new(label: &str, candidate: Candidate, write_path: WritePath, library: PathBuf, seed: u64, commit_median_microseconds: u64) -> Self {
        KillCampaign {
            label: label.to_string(),
            candidate,
            write_path,
            library,
            seed,
            commit_median_microseconds,
            confirmed_end: 0,
            started_end: 0,
            open_times_microseconds: Vec::new(),
            large_library_sampling: None,
            cumulative_lost: BTreeSet::new(),
            cumulative_corrupted: BTreeSet::new(),
            cumulative_confirmed_read_errors: BTreeSet::new(),
            cumulative_unconfirmed_read_errors: BTreeSet::new(),
            cumulative_phantoms: BTreeSet::new(),
            open_failures: 0,
            verification_timeouts: 0,
            repaired_openings: 0,
            kills_inside_commit: 0,
            write_phase_kills: 0,
            confirmations_before_write_kills: 0,
            reopen_microseconds: Vec::new(),
            write_kills_with_loss: 0,
            write_kills_with_new_loss: 0,
            write_kills_with_read_error: 0,
            first_rise: BTreeMap::new(),
            rises: BTreeMap::new(),
            assertion_failures: Vec::new(),
            stopped_after_unopenable: false,
        }
    }

    fn writer_arguments(&self) -> Vec<String> {
        vec![
            "writer".to_string(),
            format!("candidate={}", self.candidate.label()),
            format!("write_path={}", self.write_path.label()),
            format!("library={}", self.library.display()),
            format!("seed={}", self.seed),
            "pattern=random".to_string(),
            format!("states={PRIMARY_STATES_PER_BLOCK}"),
            "order=interleaved".to_string(),
            "batch=1".to_string(),
            format!("first={}", self.confirmed_end),
            "end=0".to_string(),
            "sleep=0".to_string(),
            "timed=0".to_string(),
            "probe=0".to_string(),
            "progress=every-block".to_string(),
        ]
    }

    fn verification_plan(&self, cycle_index: usize, final_full_check: bool) -> VerificationPlan {
        let checked_end = self.started_end + NOT_STARTED_BLOCKS_READ_AFTER_STARTED;
        let checked_blocks = match self.large_library_sampling {
            Some(old_block_count) if !final_full_check => {
                let mut generator = SplitMix64::new(SEED_LARGE ^ u64::try_from(cycle_index).expect("装得进 u64"));
                let mut listed: BTreeSet<u64> = (old_block_count..checked_end).collect();
                while listed.len() < usize::try_from(checked_end - old_block_count).expect("装得进 usize") + CRASH_LARGE_SAMPLED_OLD_BLOCKS {
                    listed.insert(generator.next_output() % old_block_count);
                }
                CheckedBlocks::Listed(listed.into_iter().collect())
            }
            Some(_) | None => CheckedBlocks::Range { start: 0, end: checked_end },
        };
        VerificationPlan {
            seed: self.seed,
            pattern: ValuePattern::Random,
            states_per_block: PRIMARY_STATES_PER_BLOCK,
            confirmed_end: self.confirmed_end,
            started_end: self.started_end,
            checked_blocks,
        }
    }

    fn note_rise(&mut self, quantity: &'static str, cycle_index: usize) {
        self.first_rise.entry(quantity).or_insert(cycle_index);
        *self.rises.entry(quantity).or_insert(0) += 1;
    }

    /// 一趟：起写入子进程 → 按计划杀 → 读到 EOF → 起核对子进程逐块核。返回这一趟的产物行。
    fn run_cycle(&mut self, row: &KillPlanRow, final_full_check: bool) -> String {
        let (mut child, receiver) = spawn_child(&self.writer_arguments());
        let mut lines: Vec<String> = Vec::new();
        let mut confirmations_this_cycle = 0u64;
        let delay_microseconds = match row.phase {
            KillPhase::DuringWrite { confirmations_before_kill } => {
                while confirmations_this_cycle < confirmations_before_kill {
                    let Ok(line) = receiver.recv() else { break };
                    if line.starts_with("C ") {
                        confirmations_this_cycle += 1;
                    }
                    lines.push(line);
                }
                row.delay_fraction * WRITE_PHASE_DELAY_FACTOR * self.commit_median_microseconds as f64
            }
            KillPhase::DuringOpen => row.delay_fraction * OPEN_PHASE_DELAY_FACTOR * median_of(&self.open_times_microseconds) as f64,
        };
        std::thread::sleep(Duration::from_secs_f64(delay_microseconds / 1e6));
        kill_and_reap(&mut child);
        lines.extend(drain_lines(&receiver));
        let mut last_line = LastProgressLine::None;
        let mut writer_error = String::from("-");
        let mut confirmations_total = 0u64;
        for line in &lines {
            let mut words = line.split(' ');
            match (words.next(), words.next().and_then(|number| number.parse::<u64>().ok())) {
                (Some("O"), Some(open_microseconds)) => {
                    self.open_times_microseconds.push(open_microseconds);
                    last_line = LastProgressLine::Open;
                }
                (Some("B"), Some(block_index)) => {
                    self.started_end = self.started_end.max(block_index + 1);
                    last_line = LastProgressLine::Begin;
                }
                (Some("C"), Some(block_index)) => {
                    self.confirmed_end = self.confirmed_end.max(block_index + 1);
                    confirmations_total += 1;
                    last_line = LastProgressLine::Commit;
                }
                (Some("E"), _) => writer_error = line.replace(' ', "_"),
                _ => {}
            }
        }
        let is_write_phase = matches!(row.phase, KillPhase::DuringWrite { .. });
        if is_write_phase {
            self.write_phase_kills += 1;
            self.confirmations_before_write_kills += confirmations_this_cycle;
            if last_line == LastProgressLine::Begin {
                self.kills_inside_commit += 1;
            }
            if confirmations_this_cycle < confirmations_of(row.phase) {
                self.assertion_failures.push(format!("A-S1 cycle={} confirmations={confirmations_this_cycle}", row.cycle_index));
            }
        }
        let plan = self.verification_plan(row.cycle_index, final_full_check);
        let checked_count = u64::try_from(plan.checked_list().len()).expect("装得进 u64");
        let verification = self.verify_with_timeout(&plan);
        let mut line = format!(
            "name=kill_cycle campaign={} arm={} write_path={} cycle={} phase={} delay_microseconds={:.0} confirmations_this_cycle={confirmations_this_cycle} confirmations_seen={confirmations_total} last_line={last_line:?} writer_error={writer_error} confirmed_end={} started_end={}",
            self.label,
            self.candidate.label(),
            self.write_path.label(),
            row.cycle_index,
            if is_write_phase { "write" } else { "open" },
            delay_microseconds,
            self.confirmed_end,
            self.started_end,
        );
        match verification {
            None => {
                self.open_failures += 1;
                self.verification_timeouts += 1;
                self.stopped_after_unopenable = true;
                self.note_rise("q1f", row.cycle_index);
                line.push_str(&format!(" verification=timeout_{VERIFICATION_TIMEOUT_SECONDS}s"));
            }
            Some(report) => {
                self.reopen_microseconds.push(report.open_microseconds);
                if report.open_failed {
                    self.open_failures += 1;
                    self.note_rise("q1f", row.cycle_index);
                    if report.opened_after_repair == Some(true) {
                        self.repaired_openings += 1;
                    } else {
                        self.stopped_after_unopenable = true;
                    }
                }
                let checked_sum = report.confirmed_checked + report.in_flight_checked + report.not_started_checked;
                if !report.open_failed && checked_sum != checked_count {
                    self.assertion_failures.push(format!("A-S1 cycle={} checked_sum={checked_sum} checked={checked_count}", row.cycle_index));
                }
                let new_loss = report.lost_confirmed.iter().any(|&block| !self.cumulative_lost.contains(&block));
                let before = [
                    self.cumulative_lost.len(),
                    self.cumulative_corrupted.len(),
                    self.cumulative_confirmed_read_errors.len(),
                    self.cumulative_unconfirmed_read_errors.len(),
                    self.cumulative_phantoms.len(),
                ];
                self.cumulative_lost.extend(report.lost_confirmed.iter().copied());
                self.cumulative_corrupted.extend(report.silently_corrupted.iter().copied());
                self.cumulative_confirmed_read_errors.extend(report.confirmed_read_errors.iter().copied());
                self.cumulative_unconfirmed_read_errors.extend(report.unconfirmed_read_errors.iter().copied());
                self.cumulative_phantoms.extend(report.phantom_keys.iter().cloned());
                let after = [
                    self.cumulative_lost.len(),
                    self.cumulative_corrupted.len(),
                    self.cumulative_confirmed_read_errors.len(),
                    self.cumulative_unconfirmed_read_errors.len(),
                    self.cumulative_phantoms.len(),
                ];
                for (quantity, (old, new)) in ["q1a", "q1b", "q1c", "q1d", "q1e"].into_iter().zip(before.into_iter().zip(after)) {
                    if new > old {
                        self.note_rise(quantity, row.cycle_index);
                    }
                }
                if is_write_phase {
                    if !report.lost_confirmed.is_empty() {
                        self.write_kills_with_loss += 1;
                    }
                    if new_loss {
                        self.write_kills_with_new_loss += 1;
                    }
                    if !report.confirmed_read_errors.is_empty() || !report.unconfirmed_read_errors.is_empty() {
                        self.write_kills_with_read_error += 1;
                    }
                }
                line.push_str(&format!(
                    " open_failed={} opened_after_repair={:?} reopen_microseconds={} checked={checked_count} q1a={} q1b={} q1c={} q1d={} q1e={} enumerated_keys={} first_read_error={} open_error={}",
                    report.open_failed,
                    report.opened_after_repair,
                    report.open_microseconds,
                    report.lost_confirmed.len(),
                    report.silently_corrupted.len(),
                    report.confirmed_read_errors.len(),
                    report.unconfirmed_read_errors.len(),
                    report.phantom_keys.len(),
                    report.enumerated_key_count,
                    if report.first_read_error.is_empty() { "-".to_string() } else { report.first_read_error.replace(' ', "_") },
                    if report.open_error.is_empty() { "-".to_string() } else { report.open_error.replace(' ', "_") },
                ));
                if self.large_library_sampling.is_some() {
                    // [10⁵ + 已确认的新块数, 10⁵ + 已开始的新块数]：写入是顺序前缀，所以就是 [confirmed_end, started_end]。
                    let within = report.enumerated_key_count >= self.confirmed_end && report.enumerated_key_count <= self.started_end;
                    line.push_str(&format!(" key_count_within_bounds={within}"));
                    if !within && !report.open_failed {
                        self.assertion_failures.push(format!("S1-large key count cycle={}", row.cycle_index));
                    }
                }
            }
        }
        line
    }

    /// 起核对子进程，3 600 秒还没交回就对它的 pid 发 SIGKILL（记 Q1f 超时一次，返回 None）。
    fn verify_with_timeout(&self, plan: &VerificationPlan) -> Option<VerificationReport> {
        let checked = match &plan.checked_blocks {
            CheckedBlocks::Range { start, end } => format!("range:{start}:{end}"),
            CheckedBlocks::Listed(blocks) => format!("list:{}", join_numbers(blocks)),
        };
        let arguments = vec![
            "verify".to_string(),
            format!("candidate={}", self.candidate.label()),
            format!("library={}", self.library.display()),
            format!("seed={}", plan.seed),
            format!("pattern={}", plan.pattern.label()),
            format!("states={}", plan.states_per_block),
            format!("confirmed_end={}", plan.confirmed_end),
            format!("started_end={}", plan.started_end),
            format!("checked={checked}"),
        ];
        let (mut child, receiver) = spawn_child(&arguments);
        let deadline = Instant::now() + Duration::from_secs(VERIFICATION_TIMEOUT_SECONDS);
        loop {
            match receiver.recv_timeout(Duration::from_millis(200)) {
                Ok(line) => {
                    if let Some(report) = decode_verification_report(&line) {
                        child.wait().expect("收得了核对子进程");
                        return Some(report);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    child.wait().expect("收得了核对子进程");
                    panic!("核对子进程没交回 VERIFY 行就退出了（{}）", self.label);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if Instant::now() >= deadline {
                        kill_and_reap(&mut child);
                        return None;
                    }
                }
            }
        }
    }

    fn summary_line(&self) -> String {
        let mut sorted = self.reopen_microseconds.clone();
        sorted.sort_unstable();
        let over_sixty_seconds = sorted.iter().filter(|&&microseconds| microseconds > 60_000_000).count();
        let rises: Vec<String> = self.rises.iter().map(|(quantity, count)| format!("{quantity}:{count}")).collect();
        let first: Vec<String> = self.first_rise.iter().map(|(quantity, cycle)| format!("{quantity}:{cycle}")).collect();
        format!(
            "name=kill_campaign_summary campaign={} arm={} write_path={} cycles_verified={} q1a_lost={} q1b_corrupted={} q1c_confirmed_read_errors={} q1d_unconfirmed_read_errors={} q1e_phantoms={} q1f_open_failures={} q1f_timeouts={} repaired_openings={} q1g_kills_inside_commit={} write_phase_kills={} confirmations_before_write_kills={} write_kills_with_loss={} write_kills_with_new_loss={} write_kills_with_read_error={} reopen_peak_microseconds={} reopen_median_microseconds={} reopen_last_microseconds={} reopen_over_60s={over_sixty_seconds} rises={} first_rise_cycle={} stopped_after_unopenable={} assertion_failures={}",
            self.label,
            self.candidate.label(),
            self.write_path.label(),
            self.reopen_microseconds.len(),
            self.cumulative_lost.len(),
            self.cumulative_corrupted.len(),
            self.cumulative_confirmed_read_errors.len(),
            self.cumulative_unconfirmed_read_errors.len(),
            self.cumulative_phantoms.len(),
            self.open_failures,
            self.verification_timeouts,
            self.repaired_openings,
            self.kills_inside_commit,
            self.write_phase_kills,
            self.confirmations_before_write_kills,
            self.write_kills_with_loss,
            self.write_kills_with_new_loss,
            self.write_kills_with_read_error,
            sorted.last().copied().unwrap_or(0),
            sorted_percentile(&sorted, 1, 2),
            self.reopen_microseconds.last().copied().unwrap_or(0),
            if rises.is_empty() { "-".to_string() } else { rises.join(",") },
            if first.is_empty() { "-".to_string() } else { first.join(",") },
            self.stopped_after_unopenable,
            if self.assertion_failures.is_empty() { "-".to_string() } else { self.assertion_failures.join(",") },
        )
    }
}

fn run_kill_campaign(output: &mut ProductLines, campaign: &mut KillCampaign, plan: &[KillPlanRow]) {
    let last_cycle = plan.len().saturating_sub(1);
    for row in plan {
        let final_full_check = campaign.large_library_sampling.is_some() && row.cycle_index == last_cycle;
        let line = campaign.run_cycle(row, final_full_check);
        output.line(line);
        if campaign.stopped_after_unopenable {
            output.line(format!(
                "name=kill_campaign_stopped campaign={} arm={} after_cycle={} reason=library_unopenable remaining_cycles_not_run={}",
                campaign.label,
                campaign.candidate.label(),
                row.cycle_index,
                last_cycle - row.cycle_index,
            ));
            break;
        }
    }
    output.line(campaign.summary_line());
}

// ============================== 计时的写入趟（校准、PC-rate、S2、几何取样点） ==============================

#[derive(Debug, Clone)]
struct TimedWriteSettings {
    candidate: Candidate,
    write_path: WritePath,
    library: PathBuf,
    seed: u64,
    pattern: ValuePattern,
    states_per_block: u64,
    order: WriteOrder,
    batch: usize,
    block_count: u64,
    sleep_microseconds: u64,
    raw_probe: bool,
    measure_occupancy: bool,
    quiet_wait_maximum_seconds: u64,
    allow_interference_reruns: bool,
}

#[derive(Debug, Default)]
struct TimedWriteOutcome {
    exit_code: i32,
    raw_median_microseconds: Option<u64>,
    open_microseconds: u64,
    timing_line: String,
    backlog_lines: Vec<String>,
    window_lines: Vec<String>,
    /// （走之前已写块数，走完已写块数，占用字节）。
    occupancy_samples: Vec<(u64, u64, u64)>,
    stopped_for_occupancy: bool,
    error_line: String,
    interference_at_start: Vec<String>,
    interference_at_end: Vec<String>,
    load_before: String,
    load_after: String,
    nvme_sectors_before: u64,
    nvme_sectors_after: u64,
    quiet_wait_seconds: u64,
}

impl TimedWriteSettings {
    fn writer_arguments(&self) -> Vec<String> {
        vec![
            "writer".to_string(),
            format!("candidate={}", self.candidate.label()),
            format!("write_path={}", self.write_path.label()),
            format!("library={}", self.library.display()),
            format!("seed={}", self.seed),
            format!("pattern={}", self.pattern.label()),
            format!("states={}", self.states_per_block),
            format!("order={}", match self.order {
                WriteOrder::Interleaved => "interleaved",
                WriteOrder::RandomPermutation => "random",
            }),
            format!("batch={}", self.batch),
            "first=0".to_string(),
            format!("end={}", self.block_count),
            format!("sleep={}", self.sleep_microseconds),
            "timed=1".to_string(),
            format!("probe={}", u8::from(self.raw_probe)),
            "progress=every-window".to_string(),
        ]
    }
}

fn remove_library(library: &Path) {
    if library.exists() {
        fs::remove_dir_all(library).expect("删得了库目录");
    }
}

/// V15：开跑前等干扰进程散去（每 30 秒看一次，最多等 maximum_wait_seconds 秒），返回等了几秒。
/// 可行性档传 0（`RunTier::quiet_wait_maximum_seconds`）：不等，直接往下走。
fn wait_for_quiet_machine(maximum_wait_seconds: u64) -> u64 {
    if cfg!(test) {
        // 单测跑在 cargo test 底下，cargo 自己就算干扰进程；单测不判 V15。
        return 0;
    }
    let started = Instant::now();
    while !interfering_processes().is_empty() && started.elapsed().as_secs() < maximum_wait_seconds {
        std::thread::sleep(Duration::from_secs(INTERFERENCE_POLL_SECONDS));
    }
    started.elapsed().as_secs()
}

fn run_timed_write(settings: &TimedWriteSettings) -> TimedWriteOutcome {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;
    let mut outcome = TimedWriteOutcome {
        quiet_wait_seconds: wait_for_quiet_machine(settings.quiet_wait_maximum_seconds),
        interference_at_start: interfering_processes(),
        load_before: load_average_text(),
        nvme_sectors_before: nvme_written_sectors(),
        ..TimedWriteOutcome::default()
    };
    let written = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let (request_sender, request_receiver) = mpsc::channel::<()>();
    let measurer = {
        let written = Arc::clone(&written);
        let stop = Arc::clone(&stop);
        let library = settings.library.clone();
        std::thread::spawn(move || {
            let mut samples = Vec::new();
            for () in request_receiver.iter() {
                let before = written.load(Ordering::SeqCst);
                let occupancy = library_occupancy_bytes(&library);
                let after = written.load(Ordering::SeqCst);
                if occupancy > OCCUPANCY_STOP_BYTES {
                    stop.store(true, Ordering::SeqCst);
                }
                samples.push((before, after, occupancy));
            }
            samples
        })
    };
    let (mut child, receiver) = spawn_child(&settings.writer_arguments());
    loop {
        match receiver.recv_timeout(Duration::from_millis(500)) {
            Ok(line) => {
                if let Some(count) = line.strip_prefix("C ").and_then(|count| count.parse::<u64>().ok()) {
                    written.store(count, Ordering::SeqCst);
                    if settings.measure_occupancy {
                        request_sender.send(()).expect("占用测量线程还在");
                    }
                } else if let Some(open) = line.strip_prefix("O ") {
                    outcome.open_microseconds = open.parse().unwrap_or(0);
                } else if line.starts_with("P ") {
                    outcome.raw_median_microseconds = field_value(&line, "raw_median_microseconds").and_then(|value| value.parse().ok());
                } else if line.starts_with("T ") {
                    outcome.timing_line = line;
                } else if line.starts_with("Q ") {
                    outcome.backlog_lines.push(line);
                } else if line.starts_with("W ") {
                    outcome.window_lines.push(line);
                } else if line.starts_with("E ") {
                    outcome.error_line = line.replace(' ', "_");
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if stop.load(Ordering::SeqCst) && !outcome.stopped_for_occupancy {
            outcome.stopped_for_occupancy = true;
            kill_and_reap(&mut child);
        }
    }
    let status = child.wait().expect("收得了写入子进程");
    outcome.exit_code = status.code().unwrap_or(-1);
    drop(request_sender);
    outcome.occupancy_samples = measurer.join().expect("占用测量线程没有 panic");
    outcome.interference_at_end = interfering_processes();
    outcome.load_after = load_average_text();
    outcome.nvme_sectors_after = nvme_written_sectors();
    outcome
}

fn list_or_dash(items: &[String]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(",")
    }
}

/// V15：一趟开始或结束时有干扰进程就重跑（新库），最多重跑三次；还有就带 interfered=true 照报最后一次。
fn run_timed_write_with_interference_reruns(output: &mut ProductLines, cell: &str, settings: &TimedWriteSettings) -> (TimedWriteOutcome, bool) {
    let mut attempt = 0usize;
    loop {
        remove_library(&settings.library);
        let outcome = run_timed_write(settings);
        let interfered = !outcome.interference_at_start.is_empty() || !outcome.interference_at_end.is_empty();
        output.line(format!(
            "name=timed_attempt cell={cell} arm={} attempt={attempt} quiet_wait_seconds={} interference_at_start={} interference_at_end={} load_before={} load_after={} nvme0n1_written_sectors_before={} nvme0n1_written_sectors_after={} exit_code={}",
            settings.candidate.label(),
            outcome.quiet_wait_seconds,
            list_or_dash(&outcome.interference_at_start),
            list_or_dash(&outcome.interference_at_end),
            outcome.load_before,
            outcome.load_after,
            outcome.nvme_sectors_before,
            outcome.nvme_sectors_after,
            outcome.exit_code,
        ));
        if !interfered || attempt >= INTERFERENCE_MAXIMUM_RERUNS || !settings.allow_interference_reruns {
            return (outcome, interfered);
        }
        attempt += 1;
    }
}

fn timing_field(outcome: &TimedWriteOutcome, key: &str) -> f64 {
    field_value(&outcome.timing_line, key).and_then(|value| value.parse().ok()).unwrap_or(0.0)
}

// ============================== 阳性对照 PC-S、PC-P、PC-O（登记 5.3，钉绝对值） ==============================

fn small_library_plan() -> VerificationPlan {
    VerificationPlan {
        seed: SEED_CRASH_BLOCKS,
        pattern: ValuePattern::Random,
        states_per_block: PRIMARY_STATES_PER_BLOCK,
        confirmed_end: SMALL_LIBRARY_BLOCK_COUNT,
        started_end: SMALL_LIBRARY_BLOCK_COUNT,
        checked_blocks: CheckedBlocks::Range { start: 0, end: SMALL_LIBRARY_BLOCK_COUNT + NOT_STARTED_BLOCKS_READ_AFTER_STARTED },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SmallLibraryDamage {
    /// PC-S：第 7 块写进去之前第 40 000 字节异或 0x5A。
    SilentCorruption,
    /// PC-P：另写一个计划外的键。
    PhantomKey,
    /// PC-O：正常关库之后毁掉打开要用的那一处。
    OpenBreaking,
}

/// 新库写第 0–63 块（seed_S1），按 damage 动一处，正常关库，再照 5.1 逐块核。
fn positive_control_small_library(candidate: Candidate, library: &Path, damage: SmallLibraryDamage) -> VerificationReport {
    remove_library(library);
    let fingerprint = input_fingerprint();
    {
        let mut store = open_store_for_writing(candidate, WritePath::Registered, library).expect("阳性对照的新库打得开");
        for block_index in 0..SMALL_LIBRARY_BLOCK_COUNT {
            let mut value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
            if damage == SmallLibraryDamage::SilentCorruption && block_index == SILENT_CORRUPTION_BLOCK {
                value[SILENT_CORRUPTION_BYTE_OFFSET] ^= SILENT_CORRUPTION_MASK;
            }
            let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
            store.put_blocks(&[(key, value)]).expect("阳性对照写得进");
        }
        if damage == SmallLibraryDamage::PhantomKey {
            let phantom_key = compose_block_key(PHANTOM_KEY_NODE, 0, 0, &fingerprint);
            let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, 0, PRIMARY_STATES_PER_BLOCK);
            store.put_blocks(&[(phantom_key, value)]).expect("阳性对照写得进计划外的键");
        }
    }
    if damage == SmallLibraryDamage::OpenBreaking {
        break_library_opening(candidate, library);
    }
    let report = verify_library(candidate, library, &small_library_plan());
    remove_library(library);
    report
}

/// R1、R0 把库文件前 4096 字节写成 0xFF；K 把 CURRENT 换成 `MANIFEST-999999\n`；F 把 FORMAT 的 16 字节写成 0xFF。
fn break_library_opening(candidate: Candidate, library: &Path) {
    let (path, bytes): (PathBuf, Vec<u8>) = match candidate {
        Candidate::RedbQuickRepair | Candidate::RedbDefault => (library.join(REDB_FILE_NAME), vec![0xFF; 4096]),
        Candidate::RocksDatabase => (library.join("CURRENT"), b"MANIFEST-999999\n".to_vec()),
        Candidate::HardenedFile => (library.join(FORMAT_FILE_NAME), vec![0xFF; FORMAT_FILE_BYTES]),
    };
    let truncate = candidate == Candidate::RocksDatabase;
    let mut handle = OpenOptions::new().write(true).truncate(truncate).open(&path).expect("打得开要毁的文件");
    handle.write_all(&bytes).expect("写得进要毁的文件");
    handle.sync_all().expect("fsync 得了要毁的文件");
}

fn report_counts(report: &VerificationReport) -> String {
    format!(
        "open_failed={} opened_after_repair={} q1a={} q1b={} q1b_blocks={} q1c={} q1d={} q1e={} enumerated_keys={}",
        report.open_failed,
        match report.opened_after_repair {
            None => "not_applicable",
            Some(true) => "yes",
            Some(false) => "no",
        },
        report.lost_confirmed.len(),
        report.silently_corrupted.len(),
        join_numbers(&report.silently_corrupted),
        report.confirmed_read_errors.len(),
        report.unconfirmed_read_errors.len(),
        report.phantom_keys.len(),
        report.enumerated_key_count,
    )
}

fn all_zero_except(report: &VerificationReport, allowed: &str) -> bool {
    (allowed == "q1a" || report.lost_confirmed.is_empty())
        && (allowed == "q1b" || report.silently_corrupted.is_empty())
        && report.confirmed_read_errors.is_empty()
        && report.unconfirmed_read_errors.is_empty()
        && (allowed == "q1e" || report.phantom_keys.is_empty())
        && (allowed == "q1f" || !report.open_failed)
}

// ============================== 第一段：S1（每条臂一份产物） ==============================

fn parse_arm(remaining: &[String]) -> Option<Candidate> {
    let candidate = remaining.first().and_then(|label| Candidate::parse(label));
    if candidate.is_none() {
        eprintln!("  ✗ 少了臂或臂写错：{remaining:?}");
        eprintln!("  → 怎么办：臂取 R1 R0 K F 之一");
    }
    candidate
}

/// V9 与 H4：库所在的文件系统不许是 tmpfs、ramfs、overlay；可用 < 40 GiB 不开跑。
fn environment_ready(output: &mut ProductLines, scratch: &ScratchDirectory) -> (bool, u64) {
    let (filesystem_type, available) = filesystem_type_and_available_bytes(&scratch.path);
    let filesystem_voids_everything = matches!(filesystem_type.as_str(), "tmpfs" | "ramfs" | "overlayfs" | "overlay");
    let too_little_space_to_start = available < MINIMUM_AVAILABLE_BYTES;
    output.line(format!(
        "name=environment filesystem_type={filesystem_type} available_bytes={available} filesystem_voids_everything={filesystem_voids_everything} too_little_space_to_start={too_little_space_to_start} logical_processors={} redb=4.3.0 rocksdb=0.25.0",
        std::thread::available_parallelism().map(usize::from).unwrap_or(0),
    ));
    (!filesystem_voids_everything && !too_little_space_to_start, available)
}

fn calibration_settings(candidate: Candidate, library: PathBuf, seed: u64, tier: RunTier) -> TimedWriteSettings {
    TimedWriteSettings {
        candidate,
        write_path: WritePath::Registered,
        library,
        seed,
        pattern: ValuePattern::Random,
        states_per_block: PRIMARY_STATES_PER_BLOCK,
        order: WriteOrder::Interleaved,
        batch: 1,
        block_count: CALIBRATION_BLOCK_COUNT,
        sleep_microseconds: 0,
        raw_probe: true,
        measure_occupancy: false,
        quiet_wait_maximum_seconds: tier.quiet_wait_maximum_seconds(),
        allow_interference_reruns: tier.allow_interference_reruns(),
    }
}

fn parse_tier(remaining: &[String]) -> Option<RunTier> {
    let tier = RunTier::from_argument(remaining.get(1));
    if tier.is_none() {
        eprintln!("  ✗ 臂后面那一个参数认不出：{remaining:?}");
        eprintln!("  → 怎么办：不写就是登记的次数；写 feasibility 是可行性档（S1 杀 20 次、S2 写 10⁴ 块）");
    }
    tier
}

fn run_segment_one_crash_question(output: &mut ProductLines, remaining: &[String]) -> i32 {
    let Some(candidate) = parse_arm(remaining) else { return 2 };
    let Some(tier) = parse_tier(remaining) else { return 2 };
    let main_cycles = match tier {
        RunTier::Registered => CRASH_MAIN_KILL_CYCLES,
        RunTier::Feasibility => FEASIBILITY_KILL_CYCLES,
    };
    let scratch = ScratchDirectory::create();
    output.line(format!(
        "name=config part=s1 arm={} tier={} kill_cycles={main_cycles} positive_control_cycles={POSITIVE_CONTROL_KILL_CYCLES} states_per_block={PRIMARY_STATES_PER_BLOCK} seed_s1={SEED_CRASH_BLOCKS:#x} seed_kill={SEED_KILL:#x}",
        candidate.label(),
        tier.label(),
    ));
    let (ready, _available) = environment_ready(output, &scratch);
    if !ready {
        output.line(format!("name=verdict part=s1 arm={} run=not_run reason=v9_or_h4", candidate.label()));
        return 3;
    }
    // S1 校准：A-raw 与 1 000 块，给 m。
    let calibration_library = scratch.path.join("calibration");
    let (calibration, calibration_interfered) =
        run_timed_write_with_interference_reruns(output, "s1-calibration", &calibration_settings(candidate, calibration_library.clone(), SEED_CRASH_BLOCKS, tier));
    remove_library(&calibration_library);
    let commit_median = timing_field(&calibration, "commit_median_microseconds");
    let raw_median = calibration.raw_median_microseconds.unwrap_or(0);
    let commit_waits_for_device = commit_median * 2.0 >= raw_median as f64;
    output.line(format!(
        "name=s1_calibration arm={} raw_median_microseconds={raw_median} commit_median_microseconds={commit_median:.0} commit_percentile_99_microseconds={:.0} blocks_per_second={:.3} interfered={calibration_interfered} h2_commit_median_at_least_half_raw={commit_waits_for_device} exit_code={} writer_error={}",
        candidate.label(),
        timing_field(&calibration, "commit_percentile_99_microseconds"),
        timing_field(&calibration, "blocks_per_second"),
        calibration.exit_code,
        if calibration.error_line.is_empty() { "-" } else { &calibration.error_line },
    ));
    if calibration.exit_code != 0 || !commit_waits_for_device {
        output.line(format!(
            "name=verdict part=s1 arm={} run=stopped reason={} commit_waits_for_device={commit_waits_for_device}",
            candidate.label(),
            if calibration.exit_code != 0 { "calibration_writer_failed" } else { "h2_stop" }
        ));
        return 4;
    }
    let commit_median_microseconds = commit_median.round() as u64;
    // PC-S、PC-P、PC-O：钉绝对值。
    let silent = positive_control_small_library(candidate, &scratch.path.join("pc-s"), SmallLibraryDamage::SilentCorruption);
    let silent_ok = silent.silently_corrupted == vec![SILENT_CORRUPTION_BLOCK] && all_zero_except(&silent, "q1b");
    output.line(format!("name=positive_control control=PC-S arm={} {} ok={silent_ok}", candidate.label(), report_counts(&silent)));
    let phantom = positive_control_small_library(candidate, &scratch.path.join("pc-p"), SmallLibraryDamage::PhantomKey);
    let phantom_ok = phantom.phantom_keys.len() == 1 && all_zero_except(&phantom, "q1e");
    output.line(format!("name=positive_control control=PC-P arm={} {} ok={phantom_ok}", candidate.label(), report_counts(&phantom)));
    let open_breaking = positive_control_small_library(candidate, &scratch.path.join("pc-o"), SmallLibraryDamage::OpenBreaking);
    let open_ok = open_breaking.open_failed && all_zero_except(&open_breaking, "q1f");
    output.line(format!(
        "name=positive_control control=PC-O arm={} {} ok={open_ok} open_error={}",
        candidate.label(),
        report_counts(&open_breaking),
        open_breaking.open_error.replace(char::is_whitespace, "_"),
    ));
    let full_plan = kill_plan(SEED_KILL, CRASH_MAIN_KILL_CYCLES);
    // PC-L：坏形态，杀点计划前 20 行。
    let lost_write_path = candidate.lost_block_control_write_path();
    let lost_library = scratch.path.join("pc-l");
    remove_library(&lost_library);
    let mut lost_campaign = KillCampaign::new("PC-L", candidate, lost_write_path, lost_library.clone(), SEED_CRASH_BLOCKS, commit_median_microseconds);
    run_kill_campaign(output, &mut lost_campaign, &full_plan[..POSITIVE_CONTROL_KILL_CYCLES]);
    remove_library(&lost_library);
    let lost_ok = lost_campaign.write_kills_with_loss >= u64::try_from(POSITIVE_CONTROL_LOST_MINIMUM_HITS).expect("10");
    output.line(format!(
        "name=positive_control control=PC-L arm={} write_path={} write_kills={} write_kills_with_q1a={} write_kills_with_new_q1a={} ok={lost_ok}",
        candidate.label(),
        lost_write_path.label(),
        lost_campaign.write_phase_kills,
        lost_campaign.write_kills_with_loss,
        lost_campaign.write_kills_with_new_loss,
    ));
    // PC-T：只有 F（F-direct）；前 20 行一次都没看到读报错就把 200 行跑完。
    let torn_ok = match candidate {
        Candidate::HardenedFile => {
            let torn_library = scratch.path.join("pc-t");
            remove_library(&torn_library);
            let mut torn_campaign =
                KillCampaign::new("PC-T", candidate, WritePath::FileDirectInPlace, torn_library.clone(), SEED_CRASH_BLOCKS, commit_median_microseconds);
            run_kill_campaign(output, &mut torn_campaign, &full_plan[..POSITIVE_CONTROL_KILL_CYCLES]);
            let mut extended = false;
            if torn_campaign.write_kills_with_read_error == 0 && tier == RunTier::Registered {
                extended = true;
                run_kill_campaign(output, &mut torn_campaign, &full_plan[POSITIVE_CONTROL_KILL_CYCLES..]);
            }
            remove_library(&torn_library);
            let ok = torn_campaign.write_kills_with_read_error >= 1 && torn_campaign.cumulative_corrupted.is_empty();
            output.line(format!(
                "name=positive_control control=PC-T arm=F write_kills={} write_kills_with_read_error={} q1b={} extended_to_200={extended} ok={ok}",
                torn_campaign.write_phase_kills,
                torn_campaign.write_kills_with_read_error,
                torn_campaign.cumulative_corrupted.len(),
            ));
            if ok {
                "true"
            } else {
                "false"
            }
        }
        Candidate::RedbQuickRepair | Candidate::RedbDefault | Candidate::RocksDatabase => "not_applicable",
    };
    // S1 主格：200 次杀。
    let main_library = scratch.path.join("s1-main");
    remove_library(&main_library);
    let mut main_campaign = KillCampaign::new("S1-main", candidate, WritePath::Registered, main_library.clone(), SEED_CRASH_BLOCKS, commit_median_microseconds);
    let main_plan = &full_plan[..main_cycles];
    run_kill_campaign(output, &mut main_campaign, main_plan);
    remove_library(&main_library);
    // V6 的门槛 100 是对 180 次写中途的杀写的；可行性档只报数，不判。
    let kills_inside_commit_ok = match tier {
        RunTier::Registered => {
            if main_campaign.kills_inside_commit >= u64::try_from(KILL_INSIDE_COMMIT_MINIMUM).expect("100") {
                "true"
            } else {
                "false"
            }
        }
        RunTier::Feasibility => "not_applicable",
    };
    // A-S1：确认数合计 ≥ 用到的那几行 n_i 之和（200 行时是 6 178）。
    let planned_confirmations: u64 = main_plan.iter().map(|row| confirmations_of(row.phase)).sum();
    let confirmations_ok =
        main_campaign.confirmations_before_write_kills >= planned_confirmations && main_campaign.assertion_failures.is_empty();
    let flipped_quantities: Vec<&str> = [
        ("q1a", main_campaign.cumulative_lost.len()),
        ("q1b", main_campaign.cumulative_corrupted.len()),
        ("q1c", main_campaign.cumulative_confirmed_read_errors.len()),
        ("q1d", main_campaign.cumulative_unconfirmed_read_errors.len()),
        ("q1e", main_campaign.cumulative_phantoms.len()),
        ("q1f", usize::try_from(main_campaign.open_failures).expect("装得进 usize")),
    ]
    .into_iter()
    .filter(|(_, count)| *count > 0)
    .map(|(quantity, _)| quantity)
    .collect();
    output.line(format!(
        "name=verdict part=s1 arm={} tier={} judgement={} kill_cycles={main_cycles} kills_inside_commit={} pc_l_ok={lost_ok} pc_s_ok={silent_ok} pc_p_ok={phantom_ok} pc_o_ok={open_ok} pc_t_ok={torn_ok} commit_waits_for_device={commit_waits_for_device} calibration_interfered={calibration_interfered} v6_kills_inside_commit_ok={kills_inside_commit_ok} planned_confirmations={planned_confirmations} a_s1_ok={confirmations_ok} q1a={} q1b={} q1c={} q1d={} q1e={} q1f={} repaired_openings={} s1_positive_quantities={}",
        candidate.label(),
        tier.label(),
        tier.judgement(),
        main_campaign.kills_inside_commit,
        main_campaign.cumulative_lost.len(),
        main_campaign.cumulative_corrupted.len(),
        main_campaign.cumulative_confirmed_read_errors.len(),
        main_campaign.cumulative_unconfirmed_read_errors.len(),
        main_campaign.cumulative_phantoms.len(),
        main_campaign.open_failures,
        main_campaign.repaired_openings,
        if flipped_quantities.is_empty() { "none".to_string() } else { flipped_quantities.join(",") },
    ));
    0
}

// ============================== 第一段：S2 与几何敏感性 ==============================

/// P_B：`ssh -o BatchMode=yes <PEER_SSH_HOST> nproc`，主机名取仓根 `multi-host.env`（与 layer0-shard-run.sh 的 run_on_peer 函数同一个取法）。
fn peer_logical_processors() -> (String, String) {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let configuration = fs::read_to_string(repository_root.join("multi-host.env")).unwrap_or_default();
    let host = configuration
        .lines()
        .find_map(|line| line.trim().strip_prefix("PEER_SSH_HOST="))
        .map(|value| value.trim_matches('"').to_string())
        .unwrap_or_default();
    if host.is_empty() {
        return ("-".to_string(), "no_peer_host_configured".to_string());
    }
    let answer = Command::new("ssh")
        .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=10", &host, "nproc"])
        .stdin(Stdio::null())
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default();
    (host, if answer.is_empty() { "no_answer".to_string() } else { answer })
}

/// 冷读之前：对库目录每个普通文件 posix_fadvise(DONTNEED)（python3 做，不给装置加依赖）。
fn drop_library_page_cache(library: &Path) -> bool {
    let script = "import os, sys\nfor root, _directories, files in os.walk(sys.argv[1]):\n    for name in files:\n        handle = os.open(os.path.join(root, name), os.O_RDONLY)\n        os.posix_fadvise(handle, 0, 0, os.POSIX_FADV_DONTNEED)\n        os.close(handle)\n";
    Command::new("python3").arg("-c").arg(script).arg(library).status().is_ok_and(|status| status.success())
}

#[allow(clippy::too_many_arguments, reason = "读子进程的参数一个个对着登记列，不收成结构体")]
fn run_read_child(candidate: Candidate, library: &Path, seed: u64, pattern: ValuePattern, states_per_block: u64, block_count: u64, read_count: usize, sleep_microseconds: u64, passes: u64) -> Vec<String> {
    let arguments = vec![
        "read".to_string(),
        format!("candidate={}", candidate.label()),
        format!("library={}", library.display()),
        format!("seed={seed}"),
        format!("pattern={}", pattern.label()),
        format!("states={states_per_block}"),
        format!("blocks={block_count}"),
        format!("count={read_count}"),
        format!("sleep={sleep_microseconds}"),
        format!("passes={passes}"),
    ];
    let (lines, _exit_code) = run_child_to_completion(&arguments);
    lines.into_iter().filter(|line| line.starts_with("R ") || line.starts_with("E ")).collect()
}

fn read_line_number(line: &str, key: &str) -> u64 {
    field_value(line, key).and_then(|value| value.parse().ok()).unwrap_or(0)
}

/// Q2a 的判定：跟不上（翻）就是 true。判别力自证用它（登记 8.2）。
fn write_rate_flips(measured_states_per_second: f64, required_states_per_second: f64) -> bool {
    measured_states_per_second < required_states_per_second
}

struct ThroughputContext {
    available_bytes: u64,
    total_logical_processors: u64,
}

/// 一趟 10⁵ 块：计时写入（带 A-raw、占用测量、V15 重跑）→ A-count → 冷热随机读。库留给调用方删。
fn run_throughput_cell(output: &mut ProductLines, cell: &str, settings: &TimedWriteSettings, context: &ThroughputContext) -> f64 {
    let arm = settings.candidate.label();
    let (outcome, interfered) = run_timed_write_with_interference_reruns(output, cell, settings);
    let states_per_second = timing_field(&outcome, "states_per_second");
    let commit_median = timing_field(&outcome, "commit_median_microseconds");
    let raw_median = outcome.raw_median_microseconds.unwrap_or(0);
    let commit_waits_for_device = commit_median * 2.0 >= raw_median as f64;
    let break_even_thread_seconds_per_state = if states_per_second > 0.0 { context.total_logical_processors as f64 / states_per_second } else { 0.0 };
    output.line(format!(
        "name=throughput cell={cell} arm={arm} blocks={} elapsed_microseconds={:.0} blocks_per_second={:.3} write_states_per_second={states_per_second:.1} commit_median_microseconds={commit_median:.0} commit_percentile_99_microseconds={:.0} commit_maximum_microseconds={:.0} raw_median_microseconds={raw_median} commit_waits_for_device={commit_waits_for_device} interfered={interfered} writer_exit={} writer_error={} stopped_h3={} open_microseconds={} break_even_thread_seconds_per_state_thread_seconds={break_even_thread_seconds_per_state:.6e} q2a_verdict=pending_e161",
        timing_field(&outcome, "blocks"),
        timing_field(&outcome, "elapsed_microseconds"),
        timing_field(&outcome, "blocks_per_second"),
        timing_field(&outcome, "commit_percentile_99_microseconds"),
        timing_field(&outcome, "commit_maximum_microseconds"),
        outcome.exit_code,
        if outcome.error_line.is_empty() { "-" } else { &outcome.error_line },
        outcome.stopped_for_occupancy,
        outcome.open_microseconds,
    ));
    let mut minimum_window = f64::MAX;
    for line in &outcome.window_lines {
        let window_rate: f64 = field_value(line, "states_per_second").and_then(|value| value.parse().ok()).unwrap_or(0.0);
        minimum_window = minimum_window.min(window_rate);
        output.line(format!(
            "name=window cell={cell} arm={arm} window={} end_microseconds={} states_per_second={window_rate:.1}",
            field_value(line, "window").unwrap_or("-"),
            field_value(line, "end_microseconds").unwrap_or("-"),
        ));
    }
    let last_window = outcome.window_lines.last().and_then(|line| field_value(line, "states_per_second")).unwrap_or("-");
    output.line(format!(
        "name=window_summary cell={cell} arm={arm} windows={} minimum_states_per_second={:.1} last_states_per_second={last_window} break_even_thread_seconds_per_state_minimum_window_thread_seconds={:.6e} q2b_verdict=pending_e161",
        outcome.window_lines.len(),
        if outcome.window_lines.is_empty() { 0.0 } else { minimum_window },
        if outcome.window_lines.is_empty() || minimum_window <= 0.0 { 0.0 } else { context.total_logical_processors as f64 / minimum_window },
    ));
    for line in &outcome.backlog_lines {
        output.line(format!("name=backlog cell={cell} arm={arm} {}", line.trim_start_matches("Q ")));
    }
    let value_bytes_per_block = settings.states_per_block as f64;
    let (mut peak_high, mut peak_low) = (0f64, 0f64);
    for &(written_before, written_after, occupancy) in &outcome.occupancy_samples {
        let ratio_high = if written_before > 0 { occupancy as f64 / (written_before as f64 * value_bytes_per_block) } else { 0.0 };
        let ratio_low = if written_after > 0 { occupancy as f64 / (written_after as f64 * value_bytes_per_block) } else { 0.0 };
        peak_high = peak_high.max(ratio_high);
        peak_low = peak_low.max(ratio_low);
        output.line(format!(
            "name=occupancy cell={cell} arm={arm} written_before={written_before} written_after={written_after} occupancy_bytes={occupancy} ratio_high={ratio_high:.4} ratio_low={ratio_low:.4}"
        ));
    }
    let after_close = library_occupancy_bytes(&settings.library);
    let after_close_ratio = after_close as f64 / (settings.block_count as f64 * value_bytes_per_block);
    output.line(format!(
        "name=occupancy_summary cell={cell} arm={arm} samples={} peak_ratio_high={peak_high:.4} peak_ratio_low={peak_low:.4} after_close_bytes={after_close} after_close_ratio={after_close_ratio:.4} milestone_bytes_at_peak_high={:.0} clause_bytes_at_peak_high={:.0} available_bytes={} q2e_flips_at_1_9e10={} q2e_flips_at_1_5e10={} q2e_flips_at_1_9e10_using_low={}",
        outcome.occupancy_samples.len(),
        peak_high * MILESTONE_TWO_STATES,
        peak_high * CLAUSE_TEXT_STATES,
        context.available_bytes,
        peak_high * MILESTONE_TWO_STATES > context.available_bytes as f64,
        peak_high * CLAUSE_TEXT_STATES > context.available_bytes as f64,
        peak_low * MILESTONE_TWO_STATES > context.available_bytes as f64,
    ));
    // A-count：恰好 N 个键，块值长度之和恰好 N × V。
    let (key_count, value_bytes) = match open_existing_store(settings.candidate, &settings.library)
        .and_then(|store| store.enumerate_keys_with_value_lengths())
    {
        Ok(entries) => (
            u64::try_from(entries.len()).expect("装得进 u64"),
            entries.iter().map(|(_, length)| length).sum::<u64>(),
        ),
        Err(_) => (0, 0),
    };
    let expected_value_bytes = settings.block_count * settings.states_per_block;
    output.line(format!(
        "name=a_count cell={cell} arm={arm} keys={key_count} value_bytes={value_bytes} expected_keys={} expected_value_bytes={expected_value_bytes} a_count_ok={}",
        settings.block_count,
        key_count == settings.block_count && value_bytes == expected_value_bytes,
    ));
    // Q2d：冷（fadvise 之后重开）一遍、同一个打开的库上热一遍。
    let cache_dropped = drop_library_page_cache(&settings.library);
    let read_interference_at_start = list_or_dash(&interfering_processes());
    let read_lines = run_read_child(settings.candidate, &settings.library, settings.seed, settings.pattern, settings.states_per_block, settings.block_count, RANDOM_READ_COUNT, 0, 2);
    let read_interference_at_end = list_or_dash(&interfering_processes());
    for line in &read_lines {
        let pass = field_value(line, "pass").unwrap_or("-");
        let temperature = if pass == "first" { "cold" } else { "warm" };
        let distinct = read_line_number(line, "distinct_blocks");
        let read_bytes = read_line_number(line, "read_bytes_delta");
        let percentile_99 = read_line_number(line, "percentile_99_microseconds");
        let cold_cache_confirmed = temperature == "warm" || read_bytes * 2 >= distinct * settings.states_per_block;
        output.line(format!(
            "name=random_read cell={cell} arm={arm} pass={temperature} cache_dropped={cache_dropped} reads={} distinct_blocks={distinct} median_microseconds={} percentile_99_microseconds={percentile_99} maximum_microseconds={} mismatches={} read_bytes_delta={read_bytes} v8_cold_cache_ok={cold_cache_confirmed} open_microseconds={} t_state_flip_below_thread_seconds={:.6e} q2d_verdict={} interference_at_start={read_interference_at_start} interference_at_end={read_interference_at_end}",
            read_line_number(line, "reads"),
            read_line_number(line, "median_microseconds"),
            read_line_number(line, "maximum_microseconds"),
            read_line_number(line, "mismatches"),
            read_line_number(line, "open_microseconds"),
            percentile_99 as f64 / 1e6 / settings.states_per_block as f64,
            if temperature == "cold" { "pending_e161" } else { "report_only" },
        ));
    }
    states_per_second
}

fn throughput_settings(candidate: Candidate, library: PathBuf, point: &str, block_count: u64, tier: RunTier) -> TimedWriteSettings {
    let mut settings = TimedWriteSettings {
        candidate,
        write_path: WritePath::Registered,
        library,
        seed: SEED_THROUGHPUT_BLOCKS,
        pattern: ValuePattern::Random,
        states_per_block: PRIMARY_STATES_PER_BLOCK,
        order: WriteOrder::Interleaved,
        batch: 1,
        block_count,
        sleep_microseconds: 0,
        raw_probe: true,
        measure_occupancy: true,
        quiet_wait_maximum_seconds: tier.quiet_wait_maximum_seconds(),
        allow_interference_reruns: tier.allow_interference_reruns(),
    };
    match point {
        "S2-main" => {}
        "G-bs10" => settings.states_per_block = SMALL_STATES_PER_BLOCK,
        "G-batch16" => settings.batch = BATCH_BLOCKS,
        "G-sparse" => settings.pattern = ValuePattern::Sparse,
        "G-random" => settings.order = WriteOrder::RandomPermutation,
        other => panic!("认不出的取样点 {other}"),
    }
    settings
}

fn throughput_context(output: &mut ProductLines, scratch: &ScratchDirectory, part: &str, candidate: Candidate) -> Option<ThroughputContext> {
    let (ready, available_bytes) = environment_ready(output, scratch);
    let local = u64::try_from(std::thread::available_parallelism().map(usize::from).unwrap_or(0)).expect("装得进 u64");
    let (peer_host, peer_answer) = peer_logical_processors();
    let peer = peer_answer.parse::<u64>().ok();
    output.line(format!(
        "name=processors part={part} arm={} p_a={local} p_b_command=ssh_-o_BatchMode=yes_-o_ConnectTimeout=10_{peer_host}_nproc p_b_answer={peer_answer} p_a_plus_p_b={} t_state=pending_e161",
        candidate.label(),
        peer.map_or("pending_p_b".to_string(), |peer| (local + peer).to_string()),
    ));
    if !ready {
        output.line(format!("name=verdict part={part} arm={} run=not_run reason=v9_or_h4", candidate.label()));
        return None;
    }
    Some(ThroughputContext { available_bytes, total_logical_processors: local + peer.unwrap_or(0) })
}

fn run_segment_one_throughput_question(output: &mut ProductLines, remaining: &[String]) -> i32 {
    let Some(candidate) = parse_arm(remaining) else { return 2 };
    let Some(tier) = parse_tier(remaining) else { return 2 };
    let main_block_count = match tier {
        RunTier::Registered => THROUGHPUT_BLOCK_COUNT,
        RunTier::Feasibility => FEASIBILITY_THROUGHPUT_BLOCK_COUNT,
    };
    let arm = candidate.label();
    let scratch = ScratchDirectory::create();
    output.line(format!(
        "name=config part=s2 arm={arm} tier={} blocks={main_block_count} states_per_block={PRIMARY_STATES_PER_BLOCK} seed_s2={SEED_THROUGHPUT_BLOCKS:#x} seed_read={SEED_READ:#x} seed_large={SEED_LARGE:#x}",
        tier.label()
    ));
    let Some(context) = throughput_context(output, &scratch, "s2", candidate) else { return 3 };
    // PC-rate：1 000 块两遍，一遍在计时区间里、写入调用之前睡 5 ms。
    let plain_library = scratch.path.join("pc-rate-plain");
    let sleep_library = scratch.path.join("pc-rate-sleep");
    let mut plain_settings = calibration_settings(candidate, plain_library.clone(), SEED_THROUGHPUT_BLOCKS, tier);
    plain_settings.raw_probe = false;
    let mut sleep_settings = plain_settings.clone();
    sleep_settings.library = sleep_library.clone();
    sleep_settings.sleep_microseconds = POSITIVE_CONTROL_SLEEP_MICROSECONDS;
    let (plain, _plain_interfered) = run_timed_write_with_interference_reruns(output, "pc-rate-plain", &plain_settings);
    let (slept, _slept_interfered) = run_timed_write_with_interference_reruns(output, "pc-rate-sleep", &sleep_settings);
    remove_library(&sleep_library);
    let rate_difference = timing_field(&slept, "commit_median_microseconds") - timing_field(&plain, "commit_median_microseconds");
    let rate_ok = rate_difference >= POSITIVE_CONTROL_MINIMUM_DIFFERENCE_MICROSECONDS as f64;
    output.line(format!(
        "name=positive_control control=PC-rate arm={arm} plain_commit_median_microseconds={:.0} sleep_commit_median_microseconds={:.0} difference_microseconds={rate_difference:.0} ok={rate_ok}",
        timing_field(&plain, "commit_median_microseconds"),
        timing_field(&slept, "commit_median_microseconds"),
    ));
    // PC-read：在 PC-rate 那个库上各读 1 000 次，一遍在计时区间里读调用之前睡 5 ms（修订五：照 V15 等、照 V15 重跑）。
    let mut attempt = 0usize;
    let (read_plain, read_slept) = loop {
        let quiet_wait_seconds = wait_for_quiet_machine(tier.quiet_wait_maximum_seconds());
        let interference_at_start = interfering_processes();
        let plain_lines = run_read_child(candidate, &plain_library, SEED_THROUGHPUT_BLOCKS, ValuePattern::Random, PRIMARY_STATES_PER_BLOCK, CALIBRATION_BLOCK_COUNT, POSITIVE_CONTROL_READ_COUNT, 0, 1);
        let slept_lines = run_read_child(candidate, &plain_library, SEED_THROUGHPUT_BLOCKS, ValuePattern::Random, PRIMARY_STATES_PER_BLOCK, CALIBRATION_BLOCK_COUNT, POSITIVE_CONTROL_READ_COUNT, POSITIVE_CONTROL_SLEEP_MICROSECONDS, 1);
        let interference_at_end = interfering_processes();
        output.line(format!(
            "name=timed_attempt cell=pc-read arm={arm} attempt={attempt} quiet_wait_seconds={quiet_wait_seconds} interference_at_start={} interference_at_end={}",
            list_or_dash(&interference_at_start),
            list_or_dash(&interference_at_end),
        ));
        let interfered = !interference_at_start.is_empty() || !interference_at_end.is_empty();
        if !interfered || attempt >= INTERFERENCE_MAXIMUM_RERUNS || !tier.allow_interference_reruns() {
            break (plain_lines, slept_lines);
        }
        attempt += 1;
    };
    let read_median = |lines: &[String]| lines.first().map_or(0, |line| read_line_number(line, "median_microseconds"));
    let read_difference = read_median(&read_slept).saturating_sub(read_median(&read_plain));
    let read_ok = read_difference >= POSITIVE_CONTROL_MINIMUM_DIFFERENCE_MICROSECONDS;
    output.line(format!(
        "name=positive_control control=PC-read arm={arm} plain_median_microseconds={} sleep_median_microseconds={} difference_microseconds={read_difference} ok={read_ok}",
        read_median(&read_plain),
        read_median(&read_slept),
    ));
    // PC-space：库关好之后量一次；往库目录里写一个 2³⁰ 字节的随机文件（sync_all）再量。
    let before = library_occupancy_bytes(&plain_library);
    let space_file = plain_library.join("e162-positive-control-space.bin");
    {
        let mut handle = File::create(&space_file).expect("建得了 PC-space 文件");
        let mut generator = SplitMix64::new(SEED_READ);
        let chunk_words = 1 << 17;
        for _chunk in 0..POSITIVE_CONTROL_SPACE_FILE_BYTES / (chunk_words * 8) {
            let chunk: Vec<u8> = (0..chunk_words).flat_map(|_| generator.next_output().to_le_bytes()).collect();
            handle.write_all(&chunk).expect("写得进 PC-space 文件");
        }
        handle.sync_all().expect("fsync 得了 PC-space 文件");
    }
    let after = library_occupancy_bytes(&plain_library);
    let space_difference = after.saturating_sub(before);
    let space_ok = (POSITIVE_CONTROL_SPACE_FILE_BYTES..=POSITIVE_CONTROL_SPACE_FILE_BYTES + POSITIVE_CONTROL_SPACE_SLACK_BYTES).contains(&space_difference);
    output.line(format!("name=positive_control control=PC-space arm={arm} before_bytes={before} after_bytes={after} difference_bytes={space_difference} ok={space_ok}"));
    remove_library(&plain_library);
    // S2 主格，然后在同一个库上接着跑 S1-large。
    let main_library = scratch.path.join("s2-main");
    let main_settings = throughput_settings(candidate, main_library.clone(), "S2-main", main_block_count, tier);
    let main_rate = run_throughput_cell(output, "S2-main", &main_settings, &context);
    let large_summary = match tier {
        RunTier::Feasibility => "s1_large=not_run_feasibility_tier".to_string(),
        RunTier::Registered => {
            let large_commit_median = calibration_commit_median_in_this_process(&main_library, candidate);
            let mut large_campaign =
                KillCampaign::new("S1-large", candidate, WritePath::Registered, main_library.clone(), SEED_THROUGHPUT_BLOCKS, large_commit_median);
            large_campaign.confirmed_end = THROUGHPUT_BLOCK_COUNT;
            large_campaign.started_end = THROUGHPUT_BLOCK_COUNT;
            large_campaign.large_library_sampling = Some(THROUGHPUT_BLOCK_COUNT);
            run_kill_campaign(output, &mut large_campaign, &kill_plan(SEED_LARGE, CRASH_LARGE_KILL_CYCLES));
            format!(
                "s1_large_q1a={} s1_large_q1b={} s1_large_q1c={} s1_large_q1d={} s1_large_q1e={} s1_large_q1f={} s1_large_assertion_failures={}",
                large_campaign.cumulative_lost.len(),
                large_campaign.cumulative_corrupted.len(),
                large_campaign.cumulative_confirmed_read_errors.len(),
                large_campaign.cumulative_unconfirmed_read_errors.len(),
                large_campaign.cumulative_phantoms.len(),
                large_campaign.open_failures,
                large_campaign.assertion_failures.len(),
            )
        }
    };
    remove_library(&main_library);
    output.line(format!(
        "name=verdict part=s2 arm={arm} tier={} judgement={} blocks={main_block_count} pc_rate_ok={rate_ok} pc_read_ok={read_ok} pc_space_ok={space_ok} s2_main_write_states_per_second={main_rate:.1} q2a_verdict=pending_e161 {large_summary}",
        tier.label(),
        tier.judgement(),
    ));
    0
}

/// S1-large 的 m：照 5.5 的校准趟（新库、1 000 块、主取样点形态、不杀）在这个进程里再量一遍（S1 那一份在另一个进程里）。
fn calibration_commit_median_in_this_process(main_library: &Path, candidate: Candidate) -> u64 {
    let probe_library = main_library.with_file_name("s1-large-calibration");
    let settings = TimedWriteSettings { raw_probe: false, ..calibration_settings(candidate, probe_library.clone(), SEED_CRASH_BLOCKS, RunTier::Registered) };
    remove_library(&probe_library);
    let outcome = run_timed_write(&settings);
    remove_library(&probe_library);
    timing_field(&outcome, "commit_median_microseconds").round() as u64
}

fn run_segment_one_geometry(output: &mut ProductLines, remaining: &[String]) -> i32 {
    let Some(candidate) = parse_arm(remaining) else { return 2 };
    let Some(point) = remaining.get(1).map(String::as_str).filter(|point| ["G-bs10", "G-batch16", "G-sparse", "G-random"].contains(point)) else {
        eprintln!("  ✗ 少了取样点或取样点写错：{remaining:?}");
        eprintln!("  → 怎么办：取样点取 G-bs10 G-batch16 G-sparse G-random 之一");
        return 2;
    };
    let scratch = ScratchDirectory::create();
    output.line(format!("name=config part=geometry point={point} arm={} blocks={THROUGHPUT_BLOCK_COUNT} seed_s2={SEED_THROUGHPUT_BLOCKS:#x}", candidate.label()));
    let Some(context) = throughput_context(output, &scratch, "geometry", candidate) else { return 3 };
    let library = scratch.path.join(point);
    let settings = throughput_settings(candidate, library.clone(), point, THROUGHPUT_BLOCK_COUNT, RunTier::Registered);
    let rate = run_throughput_cell(output, point, &settings, &context);
    remove_library(&library);
    output.line(format!("name=verdict part=geometry point={point} arm={} write_states_per_second={rate:.1} q2a_verdict=pending_e161", candidate.label()));
    0
}

/// 登记 8.2 的判别力自证：门槛设在两个 Q2a 值的中点，两点必须一翻一不翻；设在两点之外，两点判得相同。
fn run_discrimination(output: &mut ProductLines, remaining: &[String]) -> i32 {
    let values: Vec<f64> = remaining.iter().filter_map(|value| value.parse().ok()).collect();
    let [first, second] = values[..] else {
        eprintln!("  ✗ 要两个 Q2a 值（状态/秒）：{remaining:?}");
        eprintln!("  → 怎么办：从产物里取同一条臂主取样点与 G-bs10 的 write_states_per_second 两个数传进来");
        return 2;
    };
    let midpoint = (first + second) / 2.0;
    let below = first.min(second) / 2.0;
    let above = first.max(second) * 2.0;
    let differs_at_midpoint = write_rate_flips(first, midpoint) != write_rate_flips(second, midpoint);
    let same_below = write_rate_flips(first, below) == write_rate_flips(second, below);
    let same_above = write_rate_flips(first, above) == write_rate_flips(second, above);
    output.line(format!(
        "name=discrimination first={first} second={second} midpoint={midpoint} differs_at_midpoint={differs_at_midpoint} same_below_both={same_below} same_above_both={same_above}"
    ));
    if differs_at_midpoint && same_below && same_above {
        0
    } else {
        1
    }
}

// ============================== 单测 ==============================

#[cfg(test)]
mod tests {
    use super::*;

    /// 单测里起子进程角色：测试二进制带着这个环境变量、只跑 `child_process_role_entry` 一个测试。
    pub(super) const CHILD_ROLE_ENVIRONMENT: &str = "E162_TEST_CHILD_ROLE";

    #[test]
    fn child_process_role_entry() {
        let Ok(joined) = std::env::var(CHILD_ROLE_ENVIRONMENT) else { return };
        let arguments: Vec<String> = joined.split('\u{1f}').map(str::to_string).collect();
        let remaining = &arguments[1..];
        let exit_code = match arguments[0].as_str() {
            "writer" => run_writer_role(remaining),
            "verify" => run_verify_role(remaining),
            "read" => run_read_role(remaining),
            other => panic!("认不出的子进程角色 {other}"),
        };
        std::process::exit(exit_code);
    }

    /// 每个单测一个自己的临时目录，测完删掉。
    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new(name: &str) -> TestDirectory {
            let path = std::env::temp_dir().join(format!("e162-test-{}-{name}", std::process::id()));
            if path.exists() {
                fs::remove_dir_all(&path).expect("删得了上一次的临时目录");
            }
            fs::create_dir_all(&path).expect("建得了临时目录");
            TestDirectory { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _cleanup_outcome = fs::remove_dir_all(&self.path);
        }
    }

    const ALL_CANDIDATES: [Candidate; 4] = [Candidate::RedbQuickRepair, Candidate::RedbDefault, Candidate::RocksDatabase, Candidate::HardenedFile];

    /// 碰盘的单测一个一个跑：并行跑时别的单测的 fsync 挤进计时区间（PC-rate、H2 两条会因此误红）。
    static DEVICE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn device_lock() -> std::sync::MutexGuard<'static, ()> {
        DEVICE_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn seeds_equal_the_registered_bitwise_or_with_the_prefix() {
        assert_eq!(SEED_CRASH_BLOCKS, 0xE162_0000_0000_0001);
        assert_eq!(SEED_THROUGHPUT_BLOCKS, 0xE162_0000_0000_0002);
        assert_eq!(SEED_KILL, 0xE162_0000_0000_0003);
        assert_eq!(SEED_READ, 0xE162_0000_0000_0005);
        assert_eq!(SEED_ORDER, 0xE162_0000_0000_0006);
        assert_eq!(SEED_FINGERPRINT, 0xE162_0000_0000_0007);
        assert_eq!(SEED_LARGE, 0xE162_0000_0000_0008);
    }

    #[test]
    fn splitmix_and_crc_match_registered_anchor_values() {
        let mut generator = SplitMix64::new(0);
        assert_eq!(generator.next_output(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(generator.next_output(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(generator.next_output(), 0x06c4_5d18_8009_454f);
        assert_eq!(crc32c(&[b"123456789"]), 0xe306_9283);
        assert_eq!(crc32c(&[b"1234", b"56789"]), 0xe306_9283, "分段喂与整段喂同值");
    }

    #[test]
    fn random_block_values_match_registered_anchor_values() {
        let first = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, 0, PRIMARY_STATES_PER_BLOCK);
        assert_eq!(first.len(), 65_536);
        assert_eq!(hexadecimal(&first[..16]), "76020f1b59444d587a6607195c0f8fed");
        assert_eq!(first.iter().map(|&byte| u64::from(byte)).sum::<u64>(), 8_351_809);
        assert_eq!(crc32c(&[&first]), 0x08ee_fd5c);
        let later = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, 12_345, PRIMARY_STATES_PER_BLOCK);
        assert_eq!(hexadecimal(&later[..16]), "b3c1f0df0c77c9e98c9df5b21d0395bf");
        assert_eq!(later.iter().map(|&byte| u64::from(byte)).sum::<u64>(), 8_321_879);
        assert_eq!(crc32c(&[&later]), 0x565d_4801);
        let small = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, 0, SMALL_STATES_PER_BLOCK);
        assert_eq!(small.len(), 1_024);
        assert_eq!(small.iter().map(|&byte| u64::from(byte)).sum::<u64>(), 131_533);
    }

    #[test]
    fn sparse_block_values_match_registered_anchor_values() {
        let first = block_value(ValuePattern::Sparse, SEED_THROUGHPUT_BLOCKS, 0, PRIMARY_STATES_PER_BLOCK);
        assert_eq!(first.iter().filter(|&&byte| byte != 0).count(), 267);
        assert_eq!(first.iter().map(|&byte| u64::from(byte)).sum::<u64>(), 32_629);
        let later = block_value(ValuePattern::Sparse, SEED_THROUGHPUT_BLOCKS, 12_345, PRIMARY_STATES_PER_BLOCK);
        assert_eq!(later.iter().filter(|&&byte| byte != 0).count(), 245);
        assert_eq!(later.iter().map(|&byte| u64::from(byte)).sum::<u64>(), 28_761);
    }

    #[test]
    fn fingerprint_and_interleaved_keys_match_registered_anchor_values() {
        let fingerprint = input_fingerprint();
        assert_eq!(hexadecimal(&fingerprint), "7299025377b7594e92ceeb8a1c3cbeac");
        for (block_index, expected) in [
            (0u64, "000000000000000000000000000000007299025377b7594e92ceeb8a1c3cbeac"),
            (1, "000000000000000100000000000000007299025377b7594e92ceeb8a1c3cbeac"),
            (63, "000000070000000700000000000000007299025377b7594e92ceeb8a1c3cbeac"),
            (64, "000000000000000000000000000100007299025377b7594e92ceeb8a1c3cbeac"),
            (12_345, "00000007000000010000000000c000007299025377b7594e92ceeb8a1c3cbeac"),
            (99_999, "000000030000000700000000061a00007299025377b7594e92ceeb8a1c3cbeac"),
        ] {
            assert_eq!(hexadecimal(&interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint)), expected);
        }
        assert_eq!(
            hexadecimal(&interleaved_block_key(12_345, SMALL_STATES_PER_BLOCK, &fingerprint)),
            "000000070000000100000000000300007299025377b7594e92ceeb8a1c3cbeac"
        );
    }

    #[test]
    fn permutation_and_kill_plan_match_registered_anchor_values() {
        let permutation = random_permutation(THROUGHPUT_BLOCK_COUNT, SEED_ORDER);
        assert_eq!(permutation[..6], [40_238, 12_140, 20_790, 5_207, 14_245, 28_683]);
        assert_eq!(permutation[99_999], 91_647);
        let plan = kill_plan(SEED_KILL, CRASH_MAIN_KILL_CYCLES);
        let rendered: Vec<String> = plan.iter().take(5).map(kill_plan_row_text).collect();
        assert_eq!(
            rendered,
            [
                "0/write/12/0.3236817149915837",
                "1/write/51/0.5669778417801824",
                "2/write/50/0.6058528089881228",
                "3/write/59/0.14049280271075437",
                "4/write/7/0.8853027823689169",
            ]
        );
        assert_eq!(plan.iter().filter(|row| row.phase == KillPhase::DuringOpen).count(), 20);
        assert_eq!(plan.iter().map(|row| confirmations_of(row.phase)).sum::<u64>(), 6_178);
        assert_eq!(plan[9].phase, KillPhase::DuringOpen);
    }

    #[test]
    fn block_classification_follows_the_registered_table() {
        use BlockClass::{Confirmed, InFlight, NotStarted};
        use BlockVerdict::{ConfirmedReadError, Correct, LostConfirmed, Phantom, SilentlyCorrupted, UnconfirmedReadError};
        use ReadKind::{DifferentBytes, MatchingBytes, NotFound, ReadError};
        let expected = [
            (Confirmed, MatchingBytes, Correct),
            (Confirmed, DifferentBytes, SilentlyCorrupted),
            (Confirmed, NotFound, LostConfirmed),
            (Confirmed, ReadError, ConfirmedReadError),
            (InFlight, MatchingBytes, Correct),
            (InFlight, DifferentBytes, SilentlyCorrupted),
            (InFlight, NotFound, Correct),
            (InFlight, ReadError, UnconfirmedReadError),
            (NotStarted, MatchingBytes, Phantom),
            (NotStarted, DifferentBytes, Phantom),
            (NotStarted, NotFound, Correct),
            (NotStarted, ReadError, UnconfirmedReadError),
        ];
        for (class, read, verdict) in expected {
            assert_eq!(classify_block(class, read), verdict, "{class:?} × {read:?}");
        }
    }

    fn write_blocks(store: &mut BlockStore, seed: u64, blocks: std::ops::Range<u64>) {
        let fingerprint = input_fingerprint();
        for block_index in blocks {
            let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
            let value = block_value(ValuePattern::Random, seed, block_index, PRIMARY_STATES_PER_BLOCK);
            store.put_blocks(&[(key, value)]).expect("写得进");
        }
    }

    fn plan_for(confirmed_end: u64, started_end: u64) -> VerificationPlan {
        VerificationPlan {
            seed: SEED_CRASH_BLOCKS,
            pattern: ValuePattern::Random,
            states_per_block: PRIMARY_STATES_PER_BLOCK,
            confirmed_end,
            started_end,
            checked_blocks: CheckedBlocks::Range { start: 0, end: started_end + NOT_STARTED_BLOCKS_READ_AFTER_STARTED },
        }
    }

    fn assert_clean(report: &VerificationReport, context: &str) {
        assert!(!report.open_failed, "{context}: 打不开 {}", report.open_error);
        assert!(report.lost_confirmed.is_empty(), "{context}: 丢块 {:?}", report.lost_confirmed);
        assert!(report.silently_corrupted.is_empty(), "{context}: 坏块 {:?}", report.silently_corrupted);
        assert!(report.confirmed_read_errors.is_empty(), "{context}: 已确认块读报错 {:?}", report.confirmed_read_errors);
        assert!(report.unconfirmed_read_errors.is_empty(), "{context}: 未确认块读报错 {:?} {}", report.unconfirmed_read_errors, report.first_read_error);
        assert!(report.phantom_keys.is_empty(), "{context}: 幽灵块 {:?}", report.phantom_keys);
    }

    /// M1：F 的登记写法在任何一步被打断（等同那一步之后被 SIGKILL，页缓存还在），终名下都不会留半截文件。
    #[test]
    fn file_registered_write_path_leaves_no_partial_file_at_any_abandoned_step() {
        let _device = device_lock();
        for steps_allowed in 0..24u64 {
            let directory = TestDirectory::new(&format!("abandon-{steps_allowed}"));
            let library = directory.path.join("library");
            let mut store = open_store_for_writing(Candidate::HardenedFile, WritePath::Registered, &library).expect("建得了 F 库");
            write_blocks(&mut store, SEED_CRASH_BLOCKS, 0..4);
            match &mut store.backend {
                StoreBackend::File(file_library) => file_library.remaining_steps = Some(steps_allowed),
                StoreBackend::Redb(_) | StoreBackend::RocksDatabase(_) => unreachable!("F 臂的后端是文件"),
            }
            let fingerprint = input_fingerprint();
            let key = interleaved_block_key(4, PRIMARY_STATES_PER_BLOCK, &fingerprint);
            let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, 4, PRIMARY_STATES_PER_BLOCK);
            let confirmed_end = match store.put_blocks(&[(key, value)]) {
                Ok(()) => 5,
                Err(StoreError::AbandonedAtStep) => 4,
                Err(other) => panic!("第 {steps_allowed} 步：{other}"),
            };
            drop(store);
            let report = verify_library(Candidate::HardenedFile, &library, &plan_for(confirmed_end, 5));
            assert_clean(&report, &format!("打断在第 {steps_allowed} 步"));
        }
    }

    /// M5：半截文件（头加前 4 096 字节块值）在读路径上是读报错，不是读到字节。
    #[test]
    fn file_read_path_reports_a_half_written_file_as_a_read_error() {
        let _device = device_lock();
        let directory = TestDirectory::new("half-file");
        let library = directory.path.join("library");
        let mut store = open_store_for_writing(Candidate::HardenedFile, WritePath::Registered, &library).expect("建得了 F 库");
        write_blocks(&mut store, SEED_CRASH_BLOCKS, 0..3);
        drop(store);
        let fingerprint = input_fingerprint();
        let key = interleaved_block_key(3, PRIMARY_STATES_PER_BLOCK, &fingerprint);
        let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, 3, PRIMARY_STATES_PER_BLOCK);
        let final_path = library.join(block_file_relative_path(&key).expect("32 字节"));
        fs::create_dir_all(final_path.parent().expect("有父目录")).expect("建得了段目录");
        let mut partial = block_file_header(&key, &value).to_vec();
        partial.extend_from_slice(&value[..DIRECT_WRITE_CHUNK_BYTES]);
        fs::write(&final_path, &partial).expect("写得了半截文件");
        let report = verify_library(Candidate::HardenedFile, &library, &plan_for(3, 4));
        assert_eq!(report.unconfirmed_read_errors, vec![3]);
        assert!(report.silently_corrupted.is_empty(), "半截文件被读成了字节：{:?}", report.silently_corrupted);
        assert!(report.lost_confirmed.is_empty());
    }

    /// M6：PC-S 在每条臂上恰好查出第 7 块。
    #[test]
    fn silent_corruption_control_finds_exactly_block_seven_on_every_arm() {
        let _device = device_lock();
        for candidate in ALL_CANDIDATES {
            let directory = TestDirectory::new(&format!("pc-s-{}", candidate.label()));
            let report = positive_control_small_library(candidate, &directory.path.join("library"), SmallLibraryDamage::SilentCorruption);
            assert_eq!(report.silently_corrupted, vec![SILENT_CORRUPTION_BLOCK], "{}", candidate.label());
            assert!(all_zero_except(&report, "q1b"), "{}: {report:?}", candidate.label());
            assert_eq!(report.enumerated_key_count, 64, "{}", candidate.label());
        }
    }

    /// M8：PC-P 在每条臂上恰好查出一个计划外的键。
    #[test]
    fn phantom_key_control_finds_exactly_one_key_on_every_arm() {
        let _device = device_lock();
        for candidate in ALL_CANDIDATES {
            let directory = TestDirectory::new(&format!("pc-p-{}", candidate.label()));
            let report = positive_control_small_library(candidate, &directory.path.join("library"), SmallLibraryDamage::PhantomKey);
            assert_eq!(report.phantom_keys.len(), 1, "{}", candidate.label());
            assert!(report.phantom_keys.iter().all(|key| key.starts_with("ffffffff00000000")), "{}", candidate.label());
            assert!(all_zero_except(&report, "q1e"), "{}: {report:?}", candidate.label());
        }
    }

    /// M9：PC-O 在每条臂上恰好记一次打不开。
    #[test]
    fn open_breaking_control_counts_exactly_one_open_failure_on_every_arm() {
        let _device = device_lock();
        for candidate in ALL_CANDIDATES {
            let directory = TestDirectory::new(&format!("pc-o-{}", candidate.label()));
            let report = positive_control_small_library(candidate, &directory.path.join("library"), SmallLibraryDamage::OpenBreaking);
            assert!(report.open_failed, "{} 被毁之后还打得开", candidate.label());
            assert!(all_zero_except(&report, "q1f"), "{}: {report:?}", candidate.label());
        }
    }

    /// M7：在途的块没落下来是对的，不算丢块。
    #[test]
    fn in_flight_block_that_never_landed_is_not_a_lost_block() {
        let _device = device_lock();
        let directory = TestDirectory::new("in-flight");
        let library = directory.path.join("library");
        let mut store = open_store_for_writing(Candidate::HardenedFile, WritePath::Registered, &library).expect("建得了 F 库");
        write_blocks(&mut store, SEED_CRASH_BLOCKS, 0..10);
        drop(store);
        let report = verify_library(Candidate::HardenedFile, &library, &plan_for(10, 11));
        assert_clean(&report, "第 10 块在途、没落下来");
        assert_eq!(report.confirmed_checked, 10);
        assert_eq!(report.in_flight_checked, 1);
        assert_eq!(report.not_started_checked, 64);
    }

    /// 一次杀：写 20 块确认之后立刻杀，重开逐块核。
    fn kill_after_twenty_confirmations(candidate: Candidate, write_path: WritePath, name: &str) -> KillCampaign {
        let directory = TestDirectory::new(name);
        let library = directory.path.join("library");
        let mut campaign = KillCampaign::new(name, candidate, write_path, library, SEED_CRASH_BLOCKS, 0);
        let row = KillPlanRow { cycle_index: 0, phase: KillPhase::DuringWrite { confirmations_before_kill: 20 }, delay_fraction: 0.0 };
        let line = campaign.run_cycle(&row, false);
        assert!(line.contains("confirmations_this_cycle=20"), "{name}: {line}");
        campaign
    }

    /// M2：redb 的登记写法（Durability::Immediate）被 SIGKILL 之后，已确认的块一块不少；R-None 丢块（PC-L 的形态）。
    #[test]
    fn redb_registered_write_path_keeps_every_confirmed_block_across_sigkill() {
        let _device = device_lock();
        for candidate in [Candidate::RedbQuickRepair, Candidate::RedbDefault] {
            let registered = kill_after_twenty_confirmations(candidate, WritePath::Registered, &format!("kill-{}", candidate.label()));
            assert!(registered.cumulative_lost.is_empty(), "{} 丢了已确认的块 {:?}", candidate.label(), registered.cumulative_lost);
            assert_eq!(registered.open_failures, 0);
            let none = kill_after_twenty_confirmations(candidate, WritePath::RedbDurabilityNone, &format!("kill-none-{}", candidate.label()));
            assert!(!none.cumulative_lost.is_empty(), "{} 的 Durability::None 被杀之后一块都没丢，PC-L 看不见丢块", candidate.label());
        }
    }

    /// M4：RocksDB 的登记写法（同步写、WAL 开）被 SIGKILL 之后，已确认的块一块不少；K-noWAL 丢块。
    #[test]
    fn rocksdb_registered_write_path_keeps_every_confirmed_block_across_sigkill() {
        let _device = device_lock();
        let registered = kill_after_twenty_confirmations(Candidate::RocksDatabase, WritePath::Registered, "kill-K");
        assert!(registered.cumulative_lost.is_empty(), "K 丢了已确认的块 {:?}", registered.cumulative_lost);
        assert_eq!(registered.open_failures, 0);
        let no_log = kill_after_twenty_confirmations(Candidate::RocksDatabase, WritePath::RocksDatabaseWithoutWriteAheadLog, "kill-K-noWAL");
        assert!(!no_log.cumulative_lost.is_empty(), "K-noWAL 被杀之后一块都没丢，PC-L 看不见丢块");
    }

    /// F 的登记写法被 SIGKILL 之后一块不少；F-lazyrename 丢块（PC-L 的形态）。
    #[test]
    fn file_registered_write_path_keeps_every_confirmed_block_across_sigkill() {
        let _device = device_lock();
        let registered = kill_after_twenty_confirmations(Candidate::HardenedFile, WritePath::Registered, "kill-F");
        assert!(registered.cumulative_lost.is_empty(), "F 丢了已确认的块 {:?}", registered.cumulative_lost);
        let lazy = kill_after_twenty_confirmations(Candidate::HardenedFile, WritePath::FileLazyRename, "kill-F-lazy");
        assert!(
            !lazy.cumulative_lost.is_empty(),
            "F-lazyrename 第 17 次起的确认还没改名就被杀，应当丢块；confirmed_end={}",
            lazy.confirmed_end
        );
    }

    #[test]
    fn verification_report_survives_the_line_encoding() {
        let report = VerificationReport {
            open_failed: true,
            open_error: "redb: bad magic number".to_string(),
            opened_after_repair: Some(false),
            open_microseconds: 1_234,
            confirmed_checked: 10,
            in_flight_checked: 1,
            not_started_checked: 64,
            lost_confirmed: vec![3, 4],
            silently_corrupted: vec![7],
            confirmed_read_errors: Vec::new(),
            unconfirmed_read_errors: vec![10],
            phantom_keys: ["ffff".to_string()].into_iter().collect(),
            enumerated_key_count: 12,
            enumerated_value_bytes: 786_432,
            enumeration_error: String::new(),
            first_read_error: "block=10:CRC-32C 不对".to_string(),
        };
        let decoded = decode_verification_report(&encode_verification_report(&report)).expect("读得回");
        assert_eq!(decoded.open_error, "redb:_bad_magic_number");
        assert_eq!(decoded.lost_confirmed, vec![3, 4]);
        assert_eq!(decoded.silently_corrupted, vec![7]);
        assert_eq!(decoded.unconfirmed_read_errors, vec![10]);
        assert_eq!(decoded.opened_after_repair, Some(false));
        assert_eq!(decoded.enumerated_value_bytes, 786_432);
        assert_eq!(decoded.phantom_keys.len(), 1);
    }

    #[test]
    fn timing_summary_pins_rate_windows_and_backlog() {
        // 2 000 块，第 k 块（从 1 数）在 k × 1 000 µs 返回：1 000 块/秒，每窗 1 000 块、1 秒。
        let offsets: Vec<u64> = (1..=2_000u64).map(|block| block * 1_000).collect();
        let latencies = vec![1_000u64; 2_000];
        let lines = timing_summary_lines(&latencies, &offsets, 1_024);
        assert!(lines[0].contains("blocks=2000 "), "{}", lines[0]);
        assert!(lines[0].contains("elapsed_microseconds=2000000 "), "{}", lines[0]);
        assert!(lines[0].contains("blocks_per_second=1000.000 "), "{}", lines[0]);
        assert!(lines[0].contains("states_per_second=1024000.0 "), "{}", lines[0]);
        assert!(lines[0].contains("commit_median_microseconds=1000 "), "{}", lines[0]);
        // 按实测速率产出：第 k 块返回时已产出 k × 1 024 个状态、已消化 (k − 1) × 1 024，积压恒为 1 024。
        assert!(
            lines.iter().any(|line| line.starts_with("Q backlog_rate_percent_of_measured=100 maximum_backlog_states=1024 ")),
            "{lines:?}"
        );
        // 按实测速率的一半产出：库总比产出快，积压最大就是第一块返回之前攒的 512 个状态。
        assert!(
            lines.iter().any(|line| line.starts_with("Q backlog_rate_percent_of_measured=50 maximum_backlog_states=512 peak_at_block=1")),
            "{lines:?}"
        );
        assert!(lines.iter().any(|line| line == "W window=1 end_microseconds=2000000 states_per_second=1024000.0"), "{lines:?}");
        assert_eq!(lines.iter().filter(|line| line.starts_with("W ")).count(), 2);
    }

    #[test]
    fn block_file_validation_rejects_each_kind_of_damage() {
        let fingerprint = input_fingerprint();
        let key = interleaved_block_key(5, PRIMARY_STATES_PER_BLOCK, &fingerprint);
        let value = block_value(ValuePattern::Random, SEED_CRASH_BLOCKS, 5, PRIMARY_STATES_PER_BLOCK);
        let mut good = block_file_header(&key, &value).to_vec();
        good.extend_from_slice(&value);
        assert_eq!(good.len(), 65_584);
        assert_eq!(validate_block_file(&good, &key).expect("好文件读得出"), value);
        let other_key = interleaved_block_key(6, PRIMARY_STATES_PER_BLOCK, &fingerprint);
        assert!(validate_block_file(&good, &other_key).is_err(), "头里的键不对");
        assert!(validate_block_file(&good[..47], &key).is_err(), "短于文件头");
        assert!(validate_block_file(&good[..good.len() - 1], &key).is_err(), "少一个字节");
        for damaged_offset in [0usize, 4, 40, 44, 48, 65_583] {
            let mut damaged = good.clone();
            damaged[damaged_offset] ^= 0x01;
            assert!(validate_block_file(&damaged, &key).is_err(), "第 {damaged_offset} 字节翻了一位");
        }
    }

    #[test]
    fn write_rate_judgement_flips_only_below_the_requirement() {
        assert!(write_rate_flips(99.0, 100.0));
        assert!(!write_rate_flips(100.0, 100.0));
        assert!(!write_rate_flips(101.0, 100.0));
    }


    /// M3：每条臂的登记写法每块提交中位 ≥ 0.5 × 同一目录里裸「追加 64 KiB + fdatasync」的中位（H2）。
    #[test]
    fn every_registered_arm_commit_waits_for_the_device() {
        let _device = device_lock();
        let directory = TestDirectory::new("h2");
        let raw_median = median_of(&raw_append_fdatasync_probe(&directory.path));
        for candidate in ALL_CANDIDATES {
            let library = directory.path.join(format!("library-{}", candidate.label()));
            let mut store = open_store_for_writing(candidate, WritePath::Registered, &library).expect("建得了库");
            let fingerprint = input_fingerprint();
            let mut latencies = Vec::new();
            for block_index in 0..50u64 {
                let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
                let value = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
                let started = Instant::now();
                store.put_blocks(&[(key, value)]).expect("写得进");
                latencies.push(elapsed_microseconds(started));
            }
            let commit_median = median_of(&latencies);
            assert!(commit_median * 2 >= raw_median, "{}：每块提交中位 {commit_median} µs < 0.5 × L_raw {raw_median} µs", candidate.label());
        }
    }

    /// M11：计时区间包住提交调用：计时区间里多睡 5 ms，每块提交中位多出 ≥ 4.5 ms。
    #[test]
    fn commit_timing_includes_the_injected_five_millisecond_sleep() {
        let _device = device_lock();
        let directory = TestDirectory::new("pc-rate");
        let mut plain = calibration_settings(Candidate::HardenedFile, directory.path.join("plain"), SEED_THROUGHPUT_BLOCKS, RunTier::Registered);
        plain.raw_probe = false;
        plain.block_count = 30;
        let mut slept = plain.clone();
        slept.library = directory.path.join("slept");
        slept.sleep_microseconds = POSITIVE_CONTROL_SLEEP_MICROSECONDS;
        let plain_outcome = run_timed_write(&plain);
        let slept_outcome = run_timed_write(&slept);
        assert_eq!(timing_field(&plain_outcome, "blocks"), 30.0, "{}", plain_outcome.timing_line);
        let difference = timing_field(&slept_outcome, "commit_median_microseconds") - timing_field(&plain_outcome, "commit_median_microseconds");
        assert!(
            difference >= 4_500.0,
            "两遍中位数之差 {difference} µs；照常：{}；睡 5 ms：{}",
            plain_outcome.timing_line,
            slept_outcome.timing_line
        );
    }
}
