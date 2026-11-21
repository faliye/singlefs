//! E162（崩溃放量判定块存储选型）第三段 S3：两机共用一个库时，经网络送块跟不跟得上、断线重连之后库里的块与送出的块一致不一致。
//!
// admission: always 每跑一次都是新的中断与计时观测，同一份代码两次跑的计数与时延本来就不同
// run-condition: command python3 stat df
//!
//! 跑前登记：`research/prompts/e162-preregistration.md`：6.3（Q3a–Q3j、帧与中断）、5.3 的 PC3-drop、PC3-torn、PC3-rate、
//! 5.5 的 S3-T、S3-C、S3-W1、S3-bs10，补 5.1（S3-W256、S3-raw、跨机格的接法）、补 6（Q3k、Q3r）、补 8、补 9 的 M27–M29，
//! 第十二节「修订」里第三段那几行。
//!
//! 独立手写模型，不与 `crates/` 共用代码（`.claude/rules/implementation-first.md` 第 4 条）。候选的读写路径照登记 5.2
//! 逐字实现，从 `e162_crash_verdict_block_store.rs` 拷来（出处行号写在修订里）；帧、块值、块键与锚点用送块那个 bin 的同一份
//! （`#[path]` 引进来的 `sender_role` 模块）。
//!
//! 角色：父进程（`anchors`、`s3 <臂> [格…]`、`discrimination <Q3a> <Q3a>`）按格驱动；库进程（`library`）、回环转发进程
//! （`forward`）、核对进程（`verify`）是这个 bin 按角色参数起的子进程；送块进程是同一个目录下的 `e162-verdict-store-sender`。
//! 跨机格的数据通道（库进程自己起 `<对端命令> <中继命令>` 当数据通道、回出第 a_k 个确认之后杀它）写在库进程里；
//! 跨机格在另一台上放文件、起送块进程与收尾（补 5.1「B 上放什么」）这一版没写，见修订。
//!
//! 计时都在被测进程里用 `Instant` 计（送块进程计 Q3a、库进程的直写线程计 Q3b），父进程读子进程的行只用来排中断，
//! 不打时间戳（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）。

use e7_index_bench::Emitter;
use redb::ReadableDatabase;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

mod block_verification;
mod candidates_and_thread_stores;
mod cell_reports_and_questions;
mod child_processes_and_machine_probes;
mod direct_path_and_library_role;
mod file_library;
mod interruptions_and_frame_receiving;
mod loopback_cells;
mod product_lines_and_anchors;
#[cfg(test)]
mod test_support_for_the_network_runs;

use self::block_verification::*;
use self::candidates_and_thread_stores::*;
use self::cell_reports_and_questions::*;
use self::child_processes_and_machine_probes::*;
use self::direct_path_and_library_role::*;
use self::file_library::*;
use self::interruptions_and_frame_receiving::*;
use self::loopback_cells::*;
use self::product_lines_and_anchors::*;
#[cfg(test)]
use self::test_support_for_the_network_runs::*;

#[path = "../e162_verdict_store_sender.rs"]
#[allow(dead_code, reason = "送块二进制的入口与只有它自己用的几样，在这个 bin 里只有单测自起送块角色时用")]
mod sender_role;

use sender_role::{
    anchor_checks, block_frame_checksum, block_value, compress_true_positions, encode_acknowledgement_frame, hexadecimal,
    input_fingerprint, interleaved_block_key, network_path_block_index, read_until_full_or_end, BlockKey, SenderEndpoint,
    SenderSettings, SplitMix64, ValuePattern, ACKNOWLEDGEMENTS_PER_WINDOW, BLOCK_FRAME_HEADER_BYTES, BLOCK_FRAME_MAGIC,
    BLOCK_KEY_BYTES, DEFAULT_ACKNOWLEDGEMENT_STALL_SECONDS, FRAME_CHECKSUM_BYTES, PATH_BLOCK_COUNT, PRIMARY_STATES_PER_BLOCK,
    SEED_NETWORK_BLOCKS, SMALL_STATES_PER_BLOCK,
};

// ============================== 登记写死的常量 ==============================

/// seed_NET（登记 5.1）：中断点 a_k、u_k 与 PC3-drop 挑帧各从它起一个 SplitMix64。
const SEED_NETWORK_INTERRUPTIONS: u64 = 0xE162_0000_0000_0009;
/// S3-C、PC3-torn 的中断次数（登记 5.3、6.3 末段）；V12、V14 打中时加到 200。
const REGISTERED_INTERRUPTIONS: usize = 50;
const ESCALATED_INTERRUPTIONS: usize = 200;
/// 中断之后停 u × 200 ms 再起（登记 6.3 末段）。
const INTERRUPTION_PAUSE_UNIT_MILLISECONDS: f64 = 200.0;
/// PC3-drop：每 1 000 帧挑一帧只回确认、不写（登记 5.3）。
const DROP_CONTROL_FRAMES_PER_PICK: u64 = 1_000;
/// PC3-rate：送 2 000 块两遍，慢的那遍每帧提交之前睡 5 ms；慢的 ≤ 200 块/秒、快的 > 200 块/秒（登记 5.3）。
const RATE_CONTROL_BLOCKS: u64 = 2_000;
const RATE_CONTROL_SLEEP: Duration = Duration::from_millis(5);
const RATE_CONTROL_THRESHOLD_BLOCKS_PER_SECOND: f64 = 200.0;
/// 窗口：主 64，敏感性 1 与 256（登记 6.3 末段、补 5.1）。
const PRIMARY_WINDOW_FRAMES: usize = 64;
const STOP_AND_WAIT_WINDOW_FRAMES: usize = 1;
const WIDE_WINDOW_FRAMES: usize = 256;
/// 核对子进程 3 600 秒没交回就杀（登记 5.5，同 S1）。
const VERIFICATION_TIMEOUT_SECONDS: u64 = 3_600;
/// V15 与修订五：每 30 秒看一次、最多等 20 分钟，最多重跑三次。
const INTERFERENCE_POLL_SECONDS: u64 = 30;
const INTERFERENCE_MAXIMUM_WAIT_SECONDS: u64 = 1_200;
const INTERFERENCE_MAXIMUM_RERUNS: usize = 3;
/// H4：开跑前库所在文件系统可用 < 40 GiB 不开跑。
const MINIMUM_AVAILABLE_BYTES: u64 = 40 << 30;
const FORWARDER_BIND_DEADLINE: Duration = Duration::from_secs(5);
const CHILD_START_DEADLINE: Duration = Duration::from_secs(600);
const SENDER_BINARY_NAME: &str = "e162-verdict-store-sender";

/// 候选 F（登记 5.2）：文件头 48 字节；FORMAT 16 字节。
const FILE_MAGIC: &[u8; 4] = b"SFCB";
const FILE_FORMAT_VERSION: u32 = 1;
const FILE_HEADER_BYTES: usize = 48;
const FILE_HEADER_CHECKSUMMED_BYTES: usize = 44;
const FORMAT_MAGIC: &[u8; 8] = b"SFE162FB";
const FORMAT_FILE_BYTES: usize = 16;
const FORMAT_FILE_NAME: &str = "FORMAT";

const REDB_TABLE: redb::TableDefinition<&[u8], &[u8]> = redb::TableDefinition::new("blocks");
const REDB_FILE_NAME: &str = "blocks.redb";

// ============================== 准入、入口、产物行 ==============================

/// 子进程角色的第一个参数；带着它起的是装置自己起的子进程，不再判准入。
const CHILD_ROLE_ARGUMENTS: [&str; 3] = ["library", "forward", "verify"];

fn preflight() -> Option<String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments
        .first()
        .is_some_and(|role| CHILD_ROLE_ARGUMENTS.contains(&role.as_str()))
    {
        return None;
    }
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/e162_verdict_store_network/main.rs");
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
    let first_line = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").to_string();
    first_line.strip_prefix("forced").map(|summary| summary.trim().replace(' ', "_"))
}

fn main() {
    let forced_summary = preflight();
    let arguments: Vec<String> = std::env::args().skip(1).filter(|argument| argument != "--force").collect();
    let mode = arguments.first().map(String::as_str).unwrap_or("");
    let remaining: Vec<String> = arguments.iter().skip(1).cloned().collect();
    let exit_code = match mode {
        "library" => run_library_role(&remaining),
        "forward" => run_forward_role(&remaining),
        "verify" => run_verify_role(&remaining),
        "anchors" | "s3" | "discrimination" => {
            let mut output = ProductLines::new();
            if let Some(summary) = forced_summary {
                output.line(format!("name=preflight forced={summary}"));
            }
            let code = match mode {
                "anchors" => run_anchors(&mut output),
                "s3" => run_loopback_question(&mut output, &remaining),
                _ => run_discrimination(&mut output, &remaining),
            };
            output.finish();
            code
        }
        _ => {
            eprintln!("  ✗ 用法：e162-verdict-store-network anchors | s3 <臂> [格…] | discrimination <Q3a 状态每秒> <Q3a 状态每秒>");
            eprintln!("  → 怎么办：臂取 R1 R0 K F；格取 PC3-drop PC3-rate PC3-torn S3-T S3-C S3-W1 S3-W256 S3-bs10 S3-raw（不写就按这个次序全跑）；");
            eprintln!("     library/forward/verify 是装置自己起的子进程角色；送块进程 e162-verdict-store-sender 要先编好、放在同一个目录");
            2
        }
    };
    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests;
