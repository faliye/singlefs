//! 压测器：在 `$TMPDIR` 下建一个这一趟自己的压测库，起一份真服务挂在它上面，按参数给的目标速率与持续秒数压，
//! 报每秒帧数、每秒记录条数、单帧延迟 p50 / p99、服务端 CPU 秒数与内存峰值，末行 `PRUNE_SERVER_BENCH …` 判达标没达标
//! （达到的每秒态数不少于目标的 95% 算达标）。压测库成功、失败、被打断（SIGTERM / SIGINT）都删。
//!
//! 负载（`--mode`）：
//! - `full-block`：一块 65536 态（`--states-per-block` 可改），每态结论 52 字节（4+48），一块一帧写入，写的回应到了再一帧成批读回；
//! - `small-block-per-frame`：一块 6 态，一块一帧写、一帧读回；
//! - `small-block-batched`：一块 6 态，按 `--batch-blocks` 块或 `--batch-milliseconds` 毫秒攒够一帧写、一帧读回；
//! - `recording`：每块一行 81 字节的录入块（键 73 字节加值 8 字节）加一条路径索引，按同样的两个旋钮攒批写，不读回；
//!   每块按 `--states-per-block` 态计（缺省 6，与小块同一个块速率）；
//! - `two-routes`：一路走 unix socket、一路走 TCP 同时压，各取目标的一半；每批一帧写录入块、路径索引与结论，再一帧读回结论；
//! - `fault-injection`：同小块攒批，客户端经本进程里的故障注入代理连服务，持续秒数的 25% 处起注入一次断在帧中间、50% 处起
//!   `kill -9` 服务再在同一个库上重起、75% 处起注入一段迟到的帧重复（每 2 帧重复一帧，直到注入够 3 次）；每一次都等上一次恢复
//!   才往下走，机器忙时整个调度往后推，调度走完之前这一路照目标速率接着发帧（用时随之拉长）。每一帧写另外改写一个
//!   「帧序」键（值是这一帧的帧号），跑完读回它，要等于最后一帧的帧号——旧帧的重复或重放若再生效一次，它会被改回旧值。
//!   每次故障打一行 `PRUNE_SERVER_BENCH_FAULT kind=… injected_at_seconds=… recovery_ms=…`（恢复用时：注入到之后第一个写确认）。
//!
//! 结论的存法（`--layout`）：`per-state` 每态一条（键 77 字节 = 表标签 1 + 块键 72 + 态号 4，值 52 字节），条数最多的那一种；
//! `per-block` 每块一条（键 73 字节，值 = 态数 × 52 字节），与崩溃判定库现在按块存的形态相近。
//!
//! 每一路用一个容错客户端（`reliable_client`，追加日志放在压测库目录里）。
//! 计量：一块写完（要读回的读回并逐字节核对）才算完成；态数按完成的块数乘每块态数算。开放节奏发帧（按目标速率到点就发，不等前一帧），
//! 在途帧数到 `--maximum-in-flight-frames` 时停发新写帧（读回照发），服务跟不上就表现为达不到目标。
//! 块号 k 在 k × 每块态数 ÷ 目标速率 秒时到点（块号 0 在开压那一刻）；持续秒数到了停发新块，再等在途的块做完（至多 `--drain-seconds`），
//! 用时算到最后一块完成、不短于持续秒数（服务跟得上时达到的速率就是目标速率，跟不上时用时拉长、速率降下来）。
//! 完整性闸：每一路发出的帧数与收到的回应数对不上，判失败。
//!
//! 诊断：服务端每次 WAL 同步的用时分布（`wal_sync_p50_ms`、`wal_sync_p99_ms`、`wal_sync_max_ms`、`wal_syncs`，服务自己量的；
//! fault-injection 那一趟只含重起之后那一份服务）、压测期间压测库所在块设备的忙度（`disk_busy_percent`，`/proc/diskstats` 的 io_ticks 差值
//! 除以墙钟，读不到写 unknown）、起止 `/proc/loadavg` 的 1 分钟值。起跑前或跑完时 1 分钟值超过 `--maximum-load-average`
//! （缺省本机核数的一半）的，末行 `result=environment-dirty`、退 4，不判达标没达标（同机别的重活压着时量到的吞吐不作数）。
//!
//! 进度与卡住：跑的过程中打进度行、计数长时间一点不动判卡住并按线程打各自等在哪（`src/bench_progress.rs`，看进度的线程单独跑）。
//! fault-injection 的 `kill -9` 那一次，恢复用时从注入算起，但只认新服务就绪之后到的写确认（注入记下之后、主线程真杀掉服务之前
//! 旧服务回的确认不算恢复）。每一路客户端追加日志的整理门槛由 `--journal-compaction-bytes` 给（缺省与容错客户端相同 64 MiB），
//! 末行与每路一行报追加、整理次数、整理重写的字节。

use std::collections::HashMap;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::bench_progress::{BenchPhase, PhaseCell, ProgressWatch, RouteCounters, RouteProgress};
use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::client::{RouteAddress, SimpleClient};
use crate::configuration::socket_path_problem;
use crate::environment::{
    block_device_for_path, busy_percent, default_load_threshold, judge_environment,
    read_input_output_ticks, read_load_averages, EnvironmentJudgment,
};
use crate::fault_proxy::{FaultProxy, ProxyListen};
use crate::interruption::{install_termination_handlers, is_interrupted};
use crate::journal::JournalStatistics;
use crate::process_watch::{peak_resident_memory_kibibytes, processor_ticks, ticks_per_second};
use crate::protocol::{statistics_value, PointReadFrameBuilder, Table, WriteFrameBuilder};
use crate::reliable_client::{
    ClientFailure, Completion, OperationIdentifier, ReconnectPolicy, ReliableClient,
    ReliableClientSettings,
};
use crate::server::{
    LISTENING_LINE_PREFIX, WRITE_AHEAD_LOG_SYNC_COUNT_COUNTER,
    WRITE_AHEAD_LOG_SYNC_MAXIMUM_COUNTER, WRITE_AHEAD_LOG_SYNC_MEDIAN_COUNTER,
    WRITE_AHEAD_LOG_SYNC_NINETY_NINTH_PERCENTILE_COUNTER,
};
use crate::spawned::{SpawnedProcess, StandardError};

/// 末行的开头。
pub const BENCH_LINE_PREFIX: &str = "PRUNE_SERVER_BENCH";
/// 建好压测库时打的一行（自证的「被打断也删」那一格认它拿路径）。
pub const LIBRARY_LINE_PREFIX: &str = "PRUNE_SERVER_BENCH_LIBRARY";
/// 服务就绪、开压之前打的一行。
pub const STARTED_LINE_PREFIX: &str = "PRUNE_SERVER_BENCH_STARTED";
/// 每态结论的字节数：判定向量编号 4 加恢复字 48（`crates/singlefs-verification/src/crash_judge_conclusions.rs` 的 `RECOVERY_WORDS_PER_STATE` 那一段推的宽度）。
pub const CONCLUSION_BYTES_PER_STATE: usize = 52;
/// 块键字节数（摘要 32、计划哈希 32、块起点 8）。
pub const BLOCK_KEY_BYTES: usize = 72;
/// 满块的态数（`STATES_PER_BLOCK = 1 << 16`）。
pub const FULL_BLOCK_STATES: u32 = 1 << 16;
/// 崩溃点重放一趟量到的平均每块态数（225/35 那一趟推的约 6）。
pub const SMALL_BLOCK_STATES: u32 = 6;
/// 达标线：达到的每秒态数不少于目标的这个百分比。
pub const REQUIRED_PERCENT_OF_TARGET: u64 = 95;
/// 弄坏开关 `bench-judgment-lenient` 打开时的达标线。
const LENIENT_PERCENT_OF_TARGET: u64 = 50;
/// 退出码：起跑前或跑完时 loadavg 超过阈值，这一趟不判。
pub const ENVIRONMENT_DIRTY_EXIT_CODE: i32 = 4;

const CONCLUSION_TABLE_TAG: u8 = 0x0A;
const RECORDING_BLOCK_TAG: u8 = 0x05;
const PATH_INDEX_TAG: u8 = 0x06;
const FRAME_ORDER_TAG: u8 = 0x0F;

/// 压测客户端的重连策略：服务在 fault-injection 里被 `kill -9` 再重起，重起要等 RocksDB 开库，给足两分钟。
const BENCH_RECONNECT_POLICY: ReconnectPolicy = ReconnectPolicy {
    initial_delay: Duration::from_millis(20),
    maximum_delay: Duration::from_secs(1),
    maximum_attempts: 600,
    maximum_total: Duration::from_secs(120),
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BenchMode {
    FullBlock,
    SmallBlockPerFrame,
    SmallBlockBatched,
    Recording,
    TwoRoutes,
    FaultInjection,
}

impl BenchMode {
    pub const ALL: [BenchMode; 6] = [
        BenchMode::FullBlock,
        BenchMode::SmallBlockPerFrame,
        BenchMode::SmallBlockBatched,
        BenchMode::Recording,
        BenchMode::TwoRoutes,
        BenchMode::FaultInjection,
    ];

    pub fn name(self) -> &'static str {
        match self {
            BenchMode::FullBlock => "full-block",
            BenchMode::SmallBlockPerFrame => "small-block-per-frame",
            BenchMode::SmallBlockBatched => "small-block-batched",
            BenchMode::Recording => "recording",
            BenchMode::TwoRoutes => "two-routes",
            BenchMode::FaultInjection => "fault-injection",
        }
    }

    fn default_states_per_block(self) -> u32 {
        match self {
            BenchMode::FullBlock => FULL_BLOCK_STATES,
            BenchMode::SmallBlockPerFrame
            | BenchMode::SmallBlockBatched
            | BenchMode::Recording
            | BenchMode::TwoRoutes
            | BenchMode::FaultInjection => SMALL_BLOCK_STATES,
        }
    }

    /// 一帧至多装几块：一块一帧的两种固定 1，其余照 `--batch-blocks`。
    fn blocks_per_frame_limit(self, batch_blocks: u32) -> u32 {
        match self {
            BenchMode::FullBlock | BenchMode::SmallBlockPerFrame => 1,
            BenchMode::SmallBlockBatched
            | BenchMode::Recording
            | BenchMode::TwoRoutes
            | BenchMode::FaultInjection => batch_blocks,
        }
    }

    fn writes_recording(self) -> bool {
        match self {
            BenchMode::Recording | BenchMode::TwoRoutes => true,
            BenchMode::FullBlock
            | BenchMode::SmallBlockPerFrame
            | BenchMode::SmallBlockBatched
            | BenchMode::FaultInjection => false,
        }
    }

    fn writes_conclusions(self) -> bool {
        match self {
            BenchMode::FullBlock
            | BenchMode::SmallBlockPerFrame
            | BenchMode::SmallBlockBatched
            | BenchMode::TwoRoutes
            | BenchMode::FaultInjection => true,
            BenchMode::Recording => false,
        }
    }

    fn injects_faults(self) -> bool {
        match self {
            BenchMode::FaultInjection => true,
            BenchMode::FullBlock
            | BenchMode::SmallBlockPerFrame
            | BenchMode::SmallBlockBatched
            | BenchMode::Recording
            | BenchMode::TwoRoutes => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConclusionLayout {
    PerState,
    PerBlock,
}

impl ConclusionLayout {
    pub fn name(self) -> &'static str {
        match self {
            ConclusionLayout::PerState => "per-state",
            ConclusionLayout::PerBlock => "per-block",
        }
    }
}

#[derive(Clone, Debug)]
pub struct BenchSettings {
    pub mode: BenchMode,
    pub target_states_per_second: u64,
    pub duration: Duration,
    pub states_per_block: u32,
    pub batch_blocks: u32,
    pub batch_window: Duration,
    pub layout: ConclusionLayout,
    pub network_address: String,
    pub worker_threads: usize,
    pub maximum_in_flight_frames: usize,
    pub drain_timeout: Duration,
    pub maximum_load_average: f64,
    pub group_commit_milliseconds: Option<u64>,
    pub group_commit_frames: Option<u64>,
    pub is_selftest_instance: bool,
    pub progress_interval: Duration,
    /// 每一路客户端追加日志的整理门槛（字节）。
    pub journal_compaction_bytes: u64,
    pub stall_timeout: Duration,
}

pub const BENCH_USAGE: &str = "用法：prune-server-bench --mode <full-block|small-block-per-frame|small-block-batched|recording|two-routes|fault-injection> \
--target-states-per-second <数> --duration-seconds <数> [--states-per-block <数>] [--batch-blocks <数，缺省 512>] \
[--batch-milliseconds <数，缺省 20>] [--layout <per-state|per-block，缺省 per-state>] [--tcp <服务的 TCP 监听地址，缺省 127.0.0.1:0>] \
[--worker-threads <服务的工作线程数，缺省本机核数>] [--maximum-in-flight-frames <数，缺省 256>] [--drain-seconds <数，缺省 30>] \
[--maximum-load-average <数，缺省本机核数的一半>] [--group-commit-milliseconds <数，缺省服务的缺省>] [--group-commit-frames <数，缺省服务的缺省>] \
[--progress-seconds <数，缺省 5>] [--stall-seconds <数，缺省 300>] [--journal-compaction-bytes <数，缺省 67108864>]";

/// 压测客户端追加日志的整理门槛缺省值，与容错客户端的缺省（`ReliableClientSettings::new`）相同。
const DEFAULT_JOURNAL_COMPACTION_BYTES: u64 = 64 * 1024 * 1024;

fn parse_positive(name: &str, text: &str) -> Result<u64, String> {
    match text.parse::<u64>() {
        Ok(value) if value > 0 => Ok(value),
        Ok(_) | Err(_) => Err(format!("{name} 要正整数，收到「{text}」")),
    }
}

fn parse_positive_u32(name: &str, text: &str) -> Result<u32, String> {
    u32::try_from(parse_positive(name, text)?).map_err(|_| format!("{name} 超出 u32：{text}"))
}

fn parse_positive_usize(name: &str, text: &str) -> Result<usize, String> {
    usize::try_from(parse_positive(name, text)?).map_err(|_| format!("{name} 超出 usize：{text}"))
}

pub fn default_worker_threads() -> usize {
    thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
}

pub fn parse_bench_arguments(arguments: &[String]) -> Result<BenchSettings, String> {
    let mut mode = None;
    let mut target_states_per_second = None;
    let mut duration_seconds = None;
    let mut states_per_block = None;
    let mut batch_blocks = 512;
    let mut batch_milliseconds = 20;
    let mut layout = ConclusionLayout::PerState;
    let mut network_address = "127.0.0.1:0".to_string();
    let mut worker_threads = default_worker_threads();
    let mut maximum_in_flight_frames = 256;
    let mut drain_seconds = 30;
    let mut maximum_load_average = default_load_threshold();
    let mut group_commit_milliseconds = None;
    let mut group_commit_frames = None;
    let mut is_selftest_instance = false;
    let mut progress_seconds = 5;
    let mut stall_seconds = 300;
    let mut journal_compaction_bytes = DEFAULT_JOURNAL_COMPACTION_BYTES;
    let mut position = 0;
    while position < arguments.len() {
        let flag = arguments[position].as_str();
        if flag == "--selftest-instance" {
            is_selftest_instance = true;
            position += 1;
            continue;
        }
        let value = arguments
            .get(position + 1)
            .ok_or_else(|| format!("{flag} 后面缺值"))?
            .as_str();
        match flag {
            "--mode" => {
                mode = Some(
                    BenchMode::ALL
                        .into_iter()
                        .find(|candidate| candidate.name() == value)
                        .ok_or_else(|| format!("认不得的 --mode「{value}」"))?,
                );
            }
            "--target-states-per-second" => {
                target_states_per_second = Some(parse_positive(flag, value)?)
            }
            "--duration-seconds" => duration_seconds = Some(parse_positive(flag, value)?),
            "--states-per-block" => states_per_block = Some(parse_positive_u32(flag, value)?),
            "--batch-blocks" => batch_blocks = parse_positive_u32(flag, value)?,
            "--batch-milliseconds" => batch_milliseconds = parse_positive(flag, value)?,
            "--layout" => {
                layout = match value {
                    "per-state" => ConclusionLayout::PerState,
                    "per-block" => ConclusionLayout::PerBlock,
                    other => return Err(format!("认不得的 --layout「{other}」")),
                };
            }
            "--tcp" => network_address = value.to_string(),
            "--worker-threads" => worker_threads = parse_positive_usize(flag, value)?,
            "--maximum-in-flight-frames" => {
                maximum_in_flight_frames = parse_positive_usize(flag, value)?
            }
            "--drain-seconds" => drain_seconds = parse_positive(flag, value)?,
            "--maximum-load-average" => {
                maximum_load_average = value
                    .parse::<f64>()
                    .ok()
                    .filter(|threshold| threshold.is_finite() && *threshold > 0.0)
                    .ok_or_else(|| format!("{flag} 要正数，收到「{value}」"))?;
            }
            "--group-commit-milliseconds" => {
                group_commit_milliseconds = Some(parse_positive(flag, value)?);
            }
            "--group-commit-frames" => group_commit_frames = Some(parse_positive(flag, value)?),
            "--progress-seconds" => progress_seconds = parse_positive(flag, value)?,
            "--stall-seconds" => stall_seconds = parse_positive(flag, value)?,
            "--journal-compaction-bytes" => {
                journal_compaction_bytes = parse_positive(flag, value)?;
            }
            other => return Err(format!("认不得的参数「{other}」")),
        }
        position += 2;
    }
    let mode = mode.ok_or("缺 --mode")?;
    let target_states_per_second =
        target_states_per_second.ok_or("缺 --target-states-per-second")?;
    let duration_seconds = duration_seconds.ok_or("缺 --duration-seconds")?;
    Ok(BenchSettings {
        mode,
        target_states_per_second,
        duration: Duration::from_secs(duration_seconds),
        states_per_block: states_per_block.unwrap_or_else(|| mode.default_states_per_block()),
        batch_blocks,
        batch_window: Duration::from_millis(batch_milliseconds),
        layout,
        network_address,
        worker_threads,
        maximum_in_flight_frames,
        drain_timeout: Duration::from_secs(drain_seconds),
        maximum_load_average,
        group_commit_milliseconds,
        group_commit_frames,
        is_selftest_instance,
        progress_interval: Duration::from_secs(progress_seconds),
        stall_timeout: Duration::from_secs(stall_seconds),
        journal_compaction_bytes,
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RateJudgment {
    Met,
    Missed,
}

/// 达标判法：达到的每秒态数不少于目标的 95%。
pub fn judge_achieved_rate(
    achieved_states_per_second: f64,
    target_states_per_second: u64,
    break_switches: &BreakSwitches,
) -> RateJudgment {
    let required_percent = if break_switches.is_enabled(BreakSwitch::BenchJudgmentLenient) {
        LENIENT_PERCENT_OF_TARGET
    } else {
        REQUIRED_PERCENT_OF_TARGET
    };
    let required = target_states_per_second as f64 * required_percent as f64 / 100.0;
    if achieved_states_per_second >= required {
        RateJudgment::Met
    } else {
        RateJudgment::Missed
    }
}

/// 最近秩法取百分位，单位微秒；没有样本回 0。
pub fn percentile_microseconds(sorted_latencies: &[u32], percent: usize) -> u32 {
    if sorted_latencies.is_empty() {
        return 0;
    }
    let rank = (sorted_latencies.len() * percent).div_ceil(100).max(1);
    sorted_latencies[rank - 1]
}

fn split_mix(state: u64) -> u64 {
    let mut mixed = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

/// 一块的身份：哪一路、第几块。块键由它散列出来，键在库里打散分布，与按内容摘要定的真块键一样没有局部性。
#[derive(Clone, Copy, Debug)]
struct BlockIdentity {
    route_tag: u8,
    block_number: u64,
}

impl BlockIdentity {
    fn seed(self) -> u64 {
        split_mix((u64::from(self.route_tag) << 56) ^ self.block_number)
    }

    fn block_key(self) -> [u8; BLOCK_KEY_BYTES] {
        let mut key = [0_u8; BLOCK_KEY_BYTES];
        let mut state = self.seed();
        for chunk in key.as_chunks_mut::<8>().0 {
            state = split_mix(state);
            chunk.copy_from_slice(&state.to_le_bytes());
        }
        key
    }
}

fn conclusion_value_for_state(block_seed: u64, state_index: u32, destination: &mut [u8]) {
    debug_assert_eq!(destination.len(), CONCLUSION_BYTES_PER_STATE);
    let mut state = block_seed ^ (u64::from(state_index).wrapping_mul(0xD6E8_FEB8_6659_FD93));
    for chunk in destination.chunks_mut(8) {
        state = split_mix(state);
        chunk.copy_from_slice(&state.to_le_bytes()[..chunk.len()]);
    }
}

fn per_state_conclusion_key(
    block_key: &[u8; BLOCK_KEY_BYTES],
    state_index: u32,
) -> [u8; 1 + BLOCK_KEY_BYTES + 4] {
    let mut key = [0_u8; 1 + BLOCK_KEY_BYTES + 4];
    key[0] = CONCLUSION_TABLE_TAG;
    key[1..=BLOCK_KEY_BYTES].copy_from_slice(block_key);
    key[1 + BLOCK_KEY_BYTES..].copy_from_slice(&state_index.to_be_bytes());
    key
}

fn tagged_block_key(tag: u8, block_key: &[u8; BLOCK_KEY_BYTES]) -> [u8; 1 + BLOCK_KEY_BYTES] {
    let mut key = [0_u8; 1 + BLOCK_KEY_BYTES];
    key[0] = tag;
    key[1..].copy_from_slice(block_key);
    key
}

/// 一批连续的块。
#[derive(Clone, Copy, Debug)]
struct Batch {
    first_block: u64,
    block_count: u32,
}

impl Batch {
    fn blocks(self, route_tag: u8) -> impl Iterator<Item = BlockIdentity> {
        (self.first_block..self.first_block + u64::from(self.block_count)).map(
            move |block_number| BlockIdentity {
                route_tag,
                block_number,
            },
        )
    }
}

/// 一路要压的东西。
#[derive(Clone, Debug)]
struct RoutePlan {
    route: RouteAddress,
    route_tag: u8,
    target_states_per_second: u64,
    settings: BenchSettings,
}

impl RoutePlan {
    fn conclusion_records_per_block(&self) -> u32 {
        if !self.settings.mode.writes_conclusions() {
            return 0;
        }
        match self.settings.layout {
            ConclusionLayout::PerState => self.settings.states_per_block,
            ConclusionLayout::PerBlock => 1,
        }
    }

    fn write_entries_per_block(&self) -> u32 {
        let recording_records = if self.settings.mode.writes_recording() {
            2
        } else {
            0
        };
        recording_records + self.conclusion_records_per_block()
    }

    /// 这一帧写几条：每块的条数乘块数，fault-injection 另加一条帧序。
    fn write_entry_count(&self, batch: Batch) -> u32 {
        let frame_order_records = u32::from(self.settings.mode.injects_faults());
        self.write_entries_per_block() * batch.block_count + frame_order_records
    }

    fn write_capacity(&self, batch: Batch) -> usize {
        let per_block_bytes = usize::try_from(self.settings.states_per_block)
            .expect("u32 放得进 usize")
            * (CONCLUSION_BYTES_PER_STATE + 90)
            + 200;
        per_block_bytes * usize::try_from(batch.block_count).expect("u32 放得进 usize") + 64
    }

    fn reads_back(&self) -> bool {
        self.settings.mode.writes_conclusions()
    }

    /// fault-injection 每帧改写的那一个键。
    fn frame_order_key(&self) -> Vec<u8> {
        let mut key = vec![FRAME_ORDER_TAG];
        key.extend_from_slice(format!("frame-order/{}", self.route_tag).as_bytes());
        key
    }

    fn fill_write_frame(&self, builder: &mut WriteFrameBuilder, batch: Batch, frame_number: u64) {
        let states_per_block = self.settings.states_per_block;
        if self.settings.mode.injects_faults() {
            builder.entry(
                Table::Conclusions,
                &self.frame_order_key(),
                &frame_number.to_be_bytes(),
            );
        }
        let mut value = vec![0_u8; CONCLUSION_BYTES_PER_STATE];
        for block in batch.blocks(self.route_tag) {
            let block_key = block.block_key();
            if self.settings.mode.writes_recording() {
                builder.entry(
                    Table::Recording,
                    &tagged_block_key(RECORDING_BLOCK_TAG, &block_key),
                    &u64::from(states_per_block).to_be_bytes(),
                );
                let mut path_key = vec![PATH_INDEX_TAG];
                path_key.extend_from_slice(
                    format!("path/{}/{:016x}", block.route_tag, block.block_number).as_bytes(),
                );
                builder.entry(
                    Table::Recording,
                    &path_key,
                    format!("identity-{:016x}", block.block_number).as_bytes(),
                );
            }
            if !self.settings.mode.writes_conclusions() {
                continue;
            }
            let block_seed = block.seed();
            match self.settings.layout {
                ConclusionLayout::PerState => {
                    for state_index in 0..states_per_block {
                        conclusion_value_for_state(block_seed, state_index, &mut value);
                        builder.entry(
                            Table::Conclusions,
                            &per_state_conclusion_key(&block_key, state_index),
                            &value,
                        );
                    }
                }
                ConclusionLayout::PerBlock => {
                    builder.entry(
                        Table::Conclusions,
                        &tagged_block_key(CONCLUSION_TABLE_TAG, &block_key),
                        &self.per_block_value(block_seed),
                    );
                }
            }
        }
    }

    fn per_block_value(&self, block_seed: u64) -> Vec<u8> {
        let states = usize::try_from(self.settings.states_per_block).expect("u32 放得进 usize");
        let mut value = vec![0_u8; states * CONCLUSION_BYTES_PER_STATE];
        for (state_index, chunk) in
            (0_u32..).zip(value.as_chunks_mut::<CONCLUSION_BYTES_PER_STATE>().0)
        {
            conclusion_value_for_state(block_seed, state_index, chunk);
        }
        value
    }

    fn read_key_count(&self, batch: Batch) -> u32 {
        self.conclusion_records_per_block() * batch.block_count
    }

    fn fill_read_frame(&self, builder: &mut PointReadFrameBuilder, batch: Batch) {
        for block in batch.blocks(self.route_tag) {
            let block_key = block.block_key();
            match self.settings.layout {
                ConclusionLayout::PerState => {
                    for state_index in 0..self.settings.states_per_block {
                        builder.key(
                            Table::Conclusions,
                            &per_state_conclusion_key(&block_key, state_index),
                        );
                    }
                }
                ConclusionLayout::PerBlock => builder.key(
                    Table::Conclusions,
                    &tagged_block_key(CONCLUSION_TABLE_TAG, &block_key),
                ),
            }
        }
    }

    /// 逐字节核对读回的值；对不上回第一处的说明。
    fn verify_read_back(&self, batch: Batch, values: &[Option<Vec<u8>>]) -> Result<(), String> {
        let expected_count =
            usize::try_from(self.conclusion_records_per_block() * batch.block_count)
                .expect("u32 放得进 usize");
        if values.len() != expected_count {
            return Err(format!(
                "读回 {} 个值，该是 {expected_count} 个",
                values.len()
            ));
        }
        let mut expected_state_value = vec![0_u8; CONCLUSION_BYTES_PER_STATE];
        let mut values_in_order = values.iter();
        for block in batch.blocks(self.route_tag) {
            let block_seed = block.seed();
            match self.settings.layout {
                ConclusionLayout::PerState => {
                    for state_index in 0..self.settings.states_per_block {
                        conclusion_value_for_state(
                            block_seed,
                            state_index,
                            &mut expected_state_value,
                        );
                        let received = values_in_order.next().expect("个数上面核过");
                        if received.as_deref() != Some(expected_state_value.as_slice()) {
                            return Err(format!(
                                "块号 {} 的态 {state_index} 读回的值与写进去的不同",
                                block.block_number
                            ));
                        }
                    }
                }
                ConclusionLayout::PerBlock => {
                    let received = values_in_order.next().expect("个数上面核过");
                    if received.as_deref() != Some(self.per_block_value(block_seed).as_slice()) {
                        return Err(format!(
                            "块号 {} 读回的值与写进去的不同",
                            block.block_number
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FrameKind {
    Write,
    ReadBack,
}

#[derive(Clone, Copy, Debug)]
struct InFlightFrame {
    kind: FrameKind,
    batch: Batch,
    sent_at: Instant,
}

#[derive(Default)]
struct RouteTally {
    completed_states: u64,
    completed_blocks: u64,
    answered_frames: u64,
    records: u64,
    latencies_microseconds: Vec<u32>,
    last_completion: Option<Instant>,
}

pub struct RouteOutcome {
    pub label: &'static str,
    pub target_states_per_second: u64,
    pub completed_states: u64,
    pub completed_blocks: u64,
    pub sent_frames: u64,
    pub answered_frames: u64,
    pub records: u64,
    pub latencies_microseconds: Vec<u32>,
    pub last_completion: Option<Instant>,
    pub reconnections: u64,
    pub duplicate_responses: u64,
    pub busy_responses: u64,
    pub journal: JournalStatistics,
    pub failure: Option<String>,
}

/// fault-injection 的一次故障：什么、什么时候注入、之后第一个写确认什么时候到。
struct FaultRecord {
    kind: &'static str,
    injected_at: Instant,
    /// 从这一刻起到的写确认才算恢复；`None` 是还不算（`kill -9` 那一次要等新服务就绪：
    /// 注入记下之后、主线程真的杀掉服务之前，旧服务回的确认不是恢复）。
    counts_from: Option<Instant>,
    recovered_at: Option<Instant>,
}

/// 故障调度线程与各路的驱动线程共用的故障记录。
#[derive(Default)]
struct FaultLog {
    records: Mutex<Vec<FaultRecord>>,
    failure: Mutex<Option<String>>,
    schedule_done: AtomicBool,
}

impl FaultLog {
    /// 记一次注入；`counts_now` 为假的，之后的确认先不算恢复，等 `start_counting_recovery`。
    fn record_injection(&self, kind: &'static str, counts_now: bool) {
        let now = Instant::now();
        self.records
            .lock()
            .expect("故障记录的锁只在记一笔时短暂持有")
            .push(FaultRecord {
                kind,
                injected_at: now,
                counts_from: counts_now.then_some(now),
                recovered_at: None,
            });
    }

    /// 最近那一次注入从这一刻起到的确认算恢复（`kill -9` 之后新服务就绪了）。
    fn start_counting_recovery(&self) {
        if let Some(record) = self
            .records
            .lock()
            .expect("故障记录的锁只在记一笔时短暂持有")
            .last_mut()
        {
            record.counts_from = Some(Instant::now());
        }
    }

    /// 一个写确认到了：已经开始算、还没恢复的故障都算在这一刻恢复。
    fn note_write_acknowledged(&self, at: Instant) {
        let mut records = self
            .records
            .lock()
            .expect("故障记录的锁只在记一笔时短暂持有");
        for record in records.iter_mut() {
            if record.recovered_at.is_none() && record.counts_from.is_some_and(|from| from <= at) {
                record.recovered_at = Some(at);
            }
        }
    }

    /// 最近注入的那一次故障之后已经收到写确认（没注入过也算）。
    fn latest_is_recovered(&self) -> bool {
        self.records
            .lock()
            .expect("故障记录的锁只在记一笔时短暂持有")
            .last()
            .is_none_or(|record| record.recovered_at.is_some())
    }

    /// 故障调度走完了（注入完、或中途失败放弃）。
    fn is_schedule_done(&self) -> bool {
        self.schedule_done.load(Ordering::SeqCst)
    }

    fn record_failure(&self, message: String) {
        let mut failure = self
            .failure
            .lock()
            .expect("故障调度失败说明的锁只在记一次时短暂持有");
        if failure.is_none() {
            *failure = Some(message);
        }
    }
}

/// 一路的驱动：一个线程、一个容错客户端，按节奏发写帧，写确认到了发读回帧，读回逐字节核对。
struct RouteDriver<'plan> {
    plan: &'plan RoutePlan,
    client: ReliableClient,
    in_flight: HashMap<OperationIdentifier, InFlightFrame>,
    tally: RouteTally,
    outstanding_blocks: u64,
    sent_frames: u64,
    frame_number: u64,
    faults: Option<&'plan FaultLog>,
    progress: &'plan RouteProgress,
    stalled: Arc<AtomicBool>,
}

impl RouteDriver<'_> {
    /// 把这一刻的计数发布给看进度的线程。
    fn publish_progress(&self) {
        self.progress.publish(RouteCounters {
            completed_blocks: self.tally.completed_blocks,
            sent_frames: self.sent_frames,
            answered_frames: self.tally.answered_frames,
            in_flight: u64::try_from(self.client.outstanding_operations()).unwrap_or(u64::MAX),
            reconnections: self.client.diagnostics().reconnections,
            journal_compactions: self.client.journal_statistics().compactions,
        });
    }

    fn submit_write(&mut self, batch: Batch) -> Result<(), ClientFailure> {
        self.frame_number += 1;
        let frame_number = self.frame_number;
        let plan = self.plan;
        let (operation, _) = self.client.submit_write(
            plan.write_entry_count(batch),
            plan.write_capacity(batch),
            |builder| plan.fill_write_frame(builder, batch, frame_number),
        )?;
        self.track(operation, FrameKind::Write, batch);
        self.outstanding_blocks += u64::from(batch.block_count);
        Ok(())
    }

    fn submit_read_back(&mut self, batch: Batch) -> Result<(), ClientFailure> {
        let plan = self.plan;
        let key_count = plan.read_key_count(batch);
        let capacity = usize::try_from(key_count).expect("u32 放得进 usize") * 85;
        let operation = self.client.submit_read(key_count, capacity, |builder| {
            plan.fill_read_frame(builder, batch)
        })?;
        self.track(operation, FrameKind::ReadBack, batch);
        Ok(())
    }

    fn track(&mut self, operation: OperationIdentifier, kind: FrameKind, batch: Batch) {
        self.sent_frames += 1;
        self.in_flight.insert(
            operation,
            InFlightFrame {
                kind,
                batch,
                sent_at: Instant::now(),
            },
        );
    }

    /// 处理一个做完的帧；回 `Err` 是这一路失败。
    fn handle_completion(&mut self, completion: Completion) -> Result<(), String> {
        let received_at = Instant::now();
        let operation = completion.operation();
        let frame = self
            .in_flight
            .remove(&operation)
            .ok_or_else(|| format!("做完的操作 {operation:?} 不在在途表里"))?;
        let latency = u32::try_from(received_at.duration_since(frame.sent_at).as_micros())
            .unwrap_or(u32::MAX);
        let mut completed_batch = None;
        let records = match (frame.kind, completion) {
            (
                FrameKind::Write,
                Completion::WriteAcknowledged {
                    entries_written, ..
                },
            ) => {
                let expected = self.plan.write_entry_count(frame.batch);
                if entries_written != expected {
                    return Err(format!(
                        "写确认说写了 {entries_written} 条，该是 {expected} 条"
                    ));
                }
                if let Some(faults) = self.faults {
                    faults.note_write_acknowledged(received_at);
                }
                if self.plan.reads_back() {
                    self.submit_read_back(frame.batch)
                        .map_err(|failure| format!("发读回帧失败：{failure}"))?;
                } else {
                    completed_batch = Some(frame.batch);
                }
                u64::from(entries_written)
            }
            (FrameKind::ReadBack, Completion::Values { values, .. }) => {
                self.plan.verify_read_back(frame.batch, &values)?;
                completed_batch = Some(frame.batch);
                u64::try_from(values.len()).expect("usize 放得进 u64")
            }
            (kind, Completion::Failed { failure, .. }) => {
                return Err(format!("{kind:?} 帧没做成：{failure}"));
            }
            (kind, other) => return Err(format!("{kind:?} 帧收到的回应不对：{other:?}")),
        };
        let tally = &mut self.tally;
        tally.answered_frames += 1;
        tally.records += records;
        tally.latencies_microseconds.push(latency);
        if let Some(batch) = completed_batch {
            tally.completed_blocks += u64::from(batch.block_count);
            tally.completed_states +=
                u64::from(batch.block_count) * u64::from(self.plan.settings.states_per_block);
            tally.last_completion = Some(received_at);
            self.outstanding_blocks -= u64::from(batch.block_count);
        }
        Ok(())
    }

    /// fault-injection 跑完之后核帧序键：要等于最后一帧的帧号。
    fn verify_frame_order(&mut self) -> Result<(), String> {
        let key = self.plan.frame_order_key();
        let values = self
            .client
            .read(&[(Table::Conclusions, key)])
            .map_err(|failure| format!("读帧序键失败：{failure}"))?;
        let expected = self.frame_number.to_be_bytes();
        match values.first() {
            Some(Some(value)) if value.as_slice() == expected.as_slice() => Ok(()),
            other => Err(format!(
                "帧序键读回 {other:?}，该是最后一帧的帧号 {}：有旧帧的重复或重放又生效了一次，或最后一帧丢了",
                self.frame_number
            )),
        }
    }
}

fn drive_route(
    plan: &RoutePlan,
    start: Instant,
    journal_directory: &Path,
    faults: Option<&FaultLog>,
    progress: &RouteProgress,
    stalled: &Arc<AtomicBool>,
    client_switches: &BreakSwitches,
) -> RouteOutcome {
    let label = plan.route.label();
    let mut client_settings = ReliableClientSettings::new(
        plan.route.clone(),
        journal_directory.join(format!("route-{}.journal", plan.route_tag)),
    );
    client_settings.maximum_in_flight_frames = plan.settings.maximum_in_flight_frames;
    client_settings.journal_compaction_bytes = plan.settings.journal_compaction_bytes;
    client_settings.break_switches = client_switches.clone();
    client_settings.reconnect = BENCH_RECONNECT_POLICY;
    let stop_flag = Arc::clone(stalled);
    client_settings.stop_check =
        Arc::new(move || is_interrupted() || stop_flag.load(Ordering::SeqCst));
    let failed_outcome = |failure: String| RouteOutcome {
        label,
        target_states_per_second: plan.target_states_per_second,
        completed_states: 0,
        completed_blocks: 0,
        sent_frames: 0,
        answered_frames: 0,
        records: 0,
        latencies_microseconds: Vec::new(),
        last_completion: None,
        reconnections: 0,
        duplicate_responses: 0,
        busy_responses: 0,
        journal: JournalStatistics::default(),
        failure: Some(failure),
    };
    let client = match ReliableClient::open(client_settings) {
        Ok(client) => client,
        Err(failure) => return failed_outcome(format!("连不上 {:?}：{failure}", plan.route)),
    };
    let mut driver = RouteDriver {
        plan,
        client,
        in_flight: HashMap::new(),
        tally: RouteTally::default(),
        outstanding_blocks: 0,
        sent_frames: 0,
        frame_number: 0,
        faults,
        progress,
        stalled: Arc::clone(stalled),
    };
    progress.attach(driver.client.activity_probe());
    let mut failure = issue_frames(&mut driver, start).err();
    driver.publish_progress();
    if failure.is_none() && plan.settings.mode.injects_faults() {
        failure = driver.verify_frame_order().err();
    }
    if failure.is_none() && driver.sent_frames != driver.tally.answered_frames {
        failure = Some(format!(
            "完整性闸：发出 {} 帧，收到 {} 个回应",
            driver.sent_frames, driver.tally.answered_frames
        ));
    }
    let diagnostics = driver.client.diagnostics().clone();
    RouteOutcome {
        label,
        target_states_per_second: plan.target_states_per_second,
        completed_states: driver.tally.completed_states,
        completed_blocks: driver.tally.completed_blocks,
        sent_frames: driver.sent_frames,
        answered_frames: driver.tally.answered_frames,
        records: driver.tally.records,
        latencies_microseconds: std::mem::take(&mut driver.tally.latencies_microseconds),
        last_completion: driver.tally.last_completion,
        reconnections: diagnostics.reconnections,
        duplicate_responses: diagnostics.duplicate_responses,
        busy_responses: diagnostics.busy_responses,
        journal: driver.client.journal_statistics(),
        failure,
    }
}

/// 到这一刻为止按目标速率该发出的块数。
/// 块号 k 在 k × 每块态数 ÷ 目标速率 秒时到点（块号 0 在开压那一刻），持续 D 秒共发 ⌈D × 速率 ÷ 每块态数⌉ 块。
fn blocks_due(elapsed: Duration, target_states_per_second: u64, states_per_block: u32) -> u64 {
    let numerator = elapsed.as_micros() * u128::from(target_states_per_second);
    let denominator = u128::from(states_per_block) * 1_000_000;
    u64::try_from(numerator / denominator + 1).unwrap_or(u64::MAX)
}

/// 按节奏发写帧、收回应、发读回帧，直到持续秒数到了而且在途的块都做完；回 `Err` 是这一路失败。
fn issue_frames(driver: &mut RouteDriver<'_>, start: Instant) -> Result<(), String> {
    let plan = driver.plan;
    let settings = &plan.settings;
    let frame_limit = settings.mode.blocks_per_frame_limit(settings.batch_blocks);
    let mut blocks_accounted = 0_u64;
    let mut next_unsent_block = 0_u64;
    let mut oldest_pending_since: Option<Instant> = None;
    let mut drain_deadline: Option<Instant> = None;
    loop {
        if is_interrupted() {
            return Err("被打断（SIGTERM / SIGINT）".to_string());
        }
        if driver.stalled.load(Ordering::SeqCst) {
            return Err("看进度的线程判了卡住，这一路停下".to_string());
        }
        let now = Instant::now();
        let elapsed = now.duration_since(start);
        // fault-injection 的故障调度没走完之前照目标速率接着发：故障要有帧经过才注入得上、恢复得了。
        let faults_pending = driver
            .faults
            .is_some_and(|faults| !faults.is_schedule_done());
        let within_duration = elapsed < settings.duration || faults_pending;
        if within_duration {
            let due = blocks_due(
                elapsed,
                plan.target_states_per_second,
                settings.states_per_block,
            );
            if due > blocks_accounted {
                if blocks_accounted == next_unsent_block {
                    oldest_pending_since = Some(now);
                }
                blocks_accounted = due;
            }
        }
        let mut pending_blocks = blocks_accounted - next_unsent_block;
        let window_expired = oldest_pending_since
            .is_some_and(|since| now.duration_since(since) >= settings.batch_window);
        while pending_blocks > 0
            && (pending_blocks >= u64::from(frame_limit) || window_expired || !within_duration)
            && driver.client.outstanding_operations() < settings.maximum_in_flight_frames
        {
            let block_count = u32::try_from(pending_blocks.min(u64::from(frame_limit)))
                .expect("不超过 frame_limit，放得进 u32");
            let batch = Batch {
                first_block: next_unsent_block,
                block_count,
            };
            driver
                .submit_write(batch)
                .map_err(|failure| format!("发写帧失败：{failure}"))?;
            next_unsent_block += u64::from(block_count);
            pending_blocks -= u64::from(block_count);
            oldest_pending_since = if pending_blocks > 0 { Some(now) } else { None };
        }
        if !within_duration && pending_blocks == 0 {
            if driver.outstanding_blocks == 0 {
                return Ok(());
            }
            let deadline = *drain_deadline.get_or_insert(now + settings.drain_timeout);
            if now >= deadline {
                return Err(format!(
                    "持续秒数到了之后等了 {} 秒，还有 {} 块没做完",
                    settings.drain_timeout.as_secs(),
                    driver.outstanding_blocks
                ));
            }
        }
        let wait = if within_duration {
            let next_block_at_micros =
                u128::from(blocks_accounted) * u128::from(settings.states_per_block) * 1_000_000
                    / u128::from(plan.target_states_per_second);
            let until_next =
                Duration::from_micros(u64::try_from(next_block_at_micros).unwrap_or(u64::MAX))
                    .saturating_sub(elapsed);
            until_next.clamp(Duration::from_micros(20), Duration::from_millis(2))
        } else {
            Duration::from_millis(20)
        };
        let completions = driver
            .client
            .poll(wait)
            .map_err(|failure| format!("收回应失败：{failure}"))?;
        for completion in completions {
            driver.handle_completion(completion)?;
        }
        driver.publish_progress();
    }
}

/// 压测库目录：放手时删（成功、失败、panic 的返回路上都走到这里）。
struct TemporaryLibraryDirectory {
    path: PathBuf,
    keep_when_interrupted: bool,
    is_removed: bool,
}

impl TemporaryLibraryDirectory {
    fn create(break_switches: &BreakSwitches) -> Result<TemporaryLibraryDirectory, String> {
        let base = std::env::var_os("TMPDIR")
            .filter(|value| !value.is_empty())
            .map_or_else(|| PathBuf::from("/tmp"), PathBuf::from);
        let nanoseconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        let path = base.join(format!(
            "prune-server-bench-{}-{nanoseconds}",
            std::process::id()
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .map_err(|error| format!("建不了压测库目录 {}：{error}", path.display()))?;
        Ok(TemporaryLibraryDirectory {
            path,
            keep_when_interrupted: break_switches
                .is_enabled(BreakSwitch::BenchLibraryKeptOnInterrupt),
            is_removed: false,
        })
    }

    fn remove_now(&mut self) -> Result<(), String> {
        if self.is_removed {
            return Ok(());
        }
        if self.keep_when_interrupted && is_interrupted() {
            return Ok(());
        }
        std::fs::remove_dir_all(&self.path)
            .map_err(|error| format!("删不了压测库目录 {}：{error}", self.path.display()))?;
        self.is_removed = true;
        Ok(())
    }
}

impl Drop for TemporaryLibraryDirectory {
    fn drop(&mut self) {
        if let Err(message) = self.remove_now() {
            eprintln!("  ✗ {message}");
            eprintln!("  → 怎么办：手动删掉这个目录");
        }
    }
}

fn field_of_line(line: &str, name: &str) -> Option<String> {
    line.split_whitespace()
        .find_map(|field| {
            field
                .strip_prefix(name)
                .and_then(|rest| rest.strip_prefix('='))
        })
        .map(str::to_string)
}

/// 压测器起的服务；fault-injection 里会被 `kill -9` 再在同一个库、同一个 socket 上重起。
struct StartedServer {
    process: SpawnedProcess,
    unix_socket: PathBuf,
    network_address: String,
    arguments: Vec<String>,
    /// 已经被杀掉的那几份服务用掉的 CPU 滴答。
    earlier_processor_ticks: u64,
    /// 这一份服务开压时的 CPU 滴答（重起的那一份从 0 算）。
    ticks_at_start: u64,
}

/// 服务就绪的等待上限：同机重活压着时开库也慢，给足。
const SERVER_READY_TIMEOUT: Duration = Duration::from_secs(120);

fn spawn_server(arguments: &[String]) -> Result<(SpawnedProcess, PathBuf, String), String> {
    let executable =
        std::env::current_exe().map_err(|error| format!("找不到自己的可执行文件：{error}"))?;
    let mut command = Command::new(executable);
    command.args(arguments);
    let mut process = SpawnedProcess::spawn(command, StandardError::Inherit)?;
    let line = process.wait_for_line_starting_with(LISTENING_LINE_PREFIX, SERVER_READY_TIMEOUT)?;
    let unix_socket =
        field_of_line(&line, "unix").ok_or_else(|| format!("就绪行里没有 unix=：{line}"))?;
    let network_address =
        field_of_line(&line, "tcp").ok_or_else(|| format!("就绪行里没有 tcp=：{line}"))?;
    Ok((process, PathBuf::from(unix_socket), network_address))
}

fn start_server(settings: &BenchSettings, directory: &Path) -> Result<StartedServer, String> {
    let socket_path = directory.join("server.sock");
    if let Some(problem) = socket_path_problem(&socket_path) {
        return Err(format!("{problem}；把 TMPDIR 换成短一些的目录"));
    }
    let mut arguments = vec![
        "serve".to_string(),
        "--spawned-by-parent".to_string(),
        "--socket".to_string(),
        socket_path.display().to_string(),
        "--state-directory".to_string(),
        directory.display().to_string(),
        "--library".to_string(),
        directory.join("library").display().to_string(),
        "--tcp".to_string(),
        settings.network_address.clone(),
        "--worker-threads".to_string(),
        settings.worker_threads.to_string(),
    ];
    if let Some(milliseconds) = settings.group_commit_milliseconds {
        arguments.extend([
            "--group-commit-milliseconds".to_string(),
            milliseconds.to_string(),
        ]);
    }
    if let Some(frames) = settings.group_commit_frames {
        arguments.extend(["--group-commit-frames".to_string(), frames.to_string()]);
    }
    let (process, unix_socket, network_address) = spawn_server(&arguments)?;
    Ok(StartedServer {
        process,
        unix_socket,
        network_address,
        arguments,
        earlier_processor_ticks: 0,
        ticks_at_start: 0,
    })
}

impl StartedServer {
    fn process_identifier(&self) -> u32 {
        self.process.process_identifier()
    }

    fn processor_ticks_so_far(&self) -> Option<u64> {
        processor_ticks(self.process_identifier())
            .map(|ticks| self.earlier_processor_ticks + ticks.saturating_sub(self.ticks_at_start))
    }

    /// `kill -9` 这一份服务，在同一个库与 socket 上起一份新的。
    fn kill_and_restart(&mut self) -> Result<(), String> {
        if let Some(ticks) = processor_ticks(self.process_identifier()) {
            self.earlier_processor_ticks += ticks.saturating_sub(self.ticks_at_start);
        }
        self.process.send_signal(libc::SIGKILL)?;
        self.process
            .wait_for_exit(SERVER_READY_TIMEOUT)
            .ok_or("服务收了 SIGKILL 还没退出")?;
        let (process, unix_socket, network_address) = spawn_server(&self.arguments)?;
        if unix_socket != self.unix_socket {
            return Err(format!(
                "重起的服务挂在 {}，原来是 {}",
                unix_socket.display(),
                self.unix_socket.display()
            ));
        }
        self.process = process;
        self.network_address = network_address;
        self.ticks_at_start = 0;
        Ok(())
    }
}

fn route_plans(
    settings: &BenchSettings,
    server: &StartedServer,
    proxy_route: Option<RouteAddress>,
) -> Vec<RoutePlan> {
    let unix_route =
        proxy_route.unwrap_or_else(|| RouteAddress::UnixSocket(server.unix_socket.clone()));
    match settings.mode {
        BenchMode::TwoRoutes => {
            let unix_share = settings.target_states_per_second / 2;
            let network_share = settings.target_states_per_second - unix_share;
            vec![
                RoutePlan {
                    route: unix_route,
                    route_tag: 1,
                    target_states_per_second: unix_share.max(1),
                    settings: settings.clone(),
                },
                RoutePlan {
                    route: RouteAddress::Network(server.network_address.clone()),
                    route_tag: 2,
                    target_states_per_second: network_share.max(1),
                    settings: settings.clone(),
                },
            ]
        }
        BenchMode::FullBlock
        | BenchMode::SmallBlockPerFrame
        | BenchMode::SmallBlockBatched
        | BenchMode::Recording
        | BenchMode::FaultInjection => {
            vec![RoutePlan {
                route: unix_route,
                route_tag: 1,
                target_states_per_second: settings.target_states_per_second,
                settings: settings.clone(),
            }]
        }
    }
}

fn per_second(count: u64, elapsed: Duration) -> f64 {
    let seconds = elapsed.as_secs_f64();
    if seconds <= 0.0 {
        return 0.0;
    }
    count as f64 / seconds
}

fn milliseconds(microseconds: u32) -> f64 {
    f64::from(microseconds) / 1000.0
}

/// 判了卡住之后等各路线程出来的宽限：客户端的写超时与每帧超时（缺省 120 秒）再加 30 秒。
const STALL_EXIT_GRACE: Duration = Duration::from_secs(150);

/// 起一个带名字的线程（卡住诊断按名字认线程）。
fn spawn_named<Output: Send + 'static>(
    name: &str,
    work: impl FnOnce() -> Output + Send + 'static,
) -> Result<thread::JoinHandle<Output>, String> {
    thread::Builder::new()
        .name(name.to_string())
        .spawn(work)
        .map_err(|error| format!("起不了线程 {name}：{error}"))
}

/// 故障调度线程请主线程重起服务时带的回信通道。
type RestartReply = mpsc::Sender<Result<(), String>>;

/// fault-injection 要注入的故障次数（断在帧中间、服务重起、帧重复各一次）。
const FAULTS_PER_INJECTION_RUN: usize = 3;

/// fault-injection 跑完：核三次故障都注入上、之后都等到了写确认；回每次故障的那一行。别的负载回空。
fn settle_faults(
    settings: &BenchSettings,
    proxy: Option<&FaultProxy>,
    log: &FaultLog,
    start: Instant,
    failures: &mut Vec<String>,
) -> Vec<String> {
    let Some(proxy) = proxy else {
        return Vec::new();
    };
    debug_assert!(settings.mode.injects_faults());
    if let Some(message) = log
        .failure
        .lock()
        .expect("故障调度失败说明的锁只在记一次时短暂持有")
        .clone()
    {
        failures.push(message);
    }
    let records = log
        .records
        .lock()
        .expect("故障记录的锁只在记一笔时短暂持有");
    let mut lines = Vec::new();
    for record in records.iter() {
        let recovery = record
            .recovered_at
            .map(|at| at.duration_since(record.injected_at).as_secs_f64() * 1000.0);
        lines.push(format!(
            "PRUNE_SERVER_BENCH_FAULT kind={} injected_at_seconds={:.3} recovery_ms={}",
            record.kind,
            record.injected_at.duration_since(start).as_secs_f64(),
            recovery.map_or_else(
                || "none".to_string(),
                |milliseconds| format!("{milliseconds:.1}")
            )
        ));
        if recovery.is_none() {
            failures.push(format!("故障 {} 注入之后再没收到写确认", record.kind));
        }
    }
    if records.len() != FAULTS_PER_INJECTION_RUN {
        failures.push(format!(
            "该注入 {FAULTS_PER_INJECTION_RUN} 次故障，注入了 {} 次",
            records.len()
        ));
    }
    let control = proxy.control();
    if control.cuts_done.load(Ordering::SeqCst) == 0 {
        failures.push("「断在帧中间」没断上：注入之后代理没再收到一帧".to_string());
    }
    if control.duplicates_injected.load(Ordering::SeqCst) == 0 {
        failures.push("「帧重复」没注入上：那一段里代理没转到够两帧".to_string());
    }
    lines
}

fn fault_recovery_maximum_milliseconds(log: &FaultLog) -> Option<f64> {
    log.records
        .lock()
        .expect("故障记录的锁只在记一笔时短暂持有")
        .iter()
        .filter_map(|record| {
            record
                .recovered_at
                .map(|at| at.duration_since(record.injected_at).as_secs_f64() * 1000.0)
        })
        .reduce(f64::max)
}

/// fault-injection 那一段里要注入够的重复帧数。
const DUPLICATES_PER_INJECTION_RUN: u64 = 3;

/// fault-injection 的故障调度：持续秒数的 25% 处起断在帧中间、50% 处起 `kill -9` 服务再重起、75% 处起注入迟到的帧重复。
/// 每一次都等上一次恢复（之后收到写确认）才往下走，重复那一段等代理真注入够 `DUPLICATES_PER_INJECTION_RUN` 次才收；
/// 机器忙、重起慢时整个调度往后推，压测的那一路在调度走完之前接着照目标速率发帧（`FaultLog::is_schedule_done`）。
fn run_fault_schedule(
    start: Instant,
    duration: Duration,
    control: &crate::fault_proxy::ProxyControl,
    restart_requests: &mpsc::Sender<RestartReply>,
    log: &FaultLog,
    phase: &PhaseCell,
    routes_finished: &AtomicBool,
) {
    inject_scheduled_faults(
        start,
        duration,
        control,
        restart_requests,
        log,
        phase,
        routes_finished,
    );
    control.duplicate_every_frames.store(0, Ordering::SeqCst);
    phase.set(BenchPhase::FaultsDone);
    log.schedule_done.store(true, Ordering::SeqCst);
}

fn inject_scheduled_faults(
    start: Instant,
    duration: Duration,
    control: &crate::fault_proxy::ProxyControl,
    restart_requests: &mpsc::Sender<RestartReply>,
    log: &FaultLog,
    phase: &PhaseCell,
    routes_finished: &AtomicBool,
) {
    let wait_for = |condition: &dyn Fn() -> bool| -> bool {
        loop {
            if routes_finished.load(Ordering::SeqCst) || is_interrupted() {
                return false;
            }
            if condition() {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
    };
    let reached = |fraction: f64| {
        let target = start + duration.mul_f64(fraction);
        move || Instant::now() >= target
    };
    let latest_recovered = || log.latest_is_recovered();
    if !wait_for(&reached(0.25)) {
        return;
    }
    control.cut_mid_frame_requests.store(1, Ordering::SeqCst);
    log.record_injection("disconnect-mid-frame", true);
    phase.set(BenchPhase::DisconnectMidFrame);
    if !wait_for(&latest_recovered) {
        return;
    }
    phase.set(BenchPhase::BeforeServerKill);
    if !wait_for(&reached(0.5)) {
        return;
    }
    log.record_injection("server-kill-restart", false);
    phase.set(BenchPhase::ServerRestarting);
    let (reply_sender, reply_receiver) = mpsc::channel();
    let restarted = restart_requests
        .send(reply_sender)
        .map_err(|_| "主线程已经不收重起请求".to_string())
        .and_then(|()| {
            reply_receiver
                .recv()
                .map_err(|_| "主线程没回重起的结果".to_string())?
        });
    if let Err(message) = restarted {
        log.record_failure(format!("kill -9 之后重起服务失败：{message}"));
        return;
    }
    log.start_counting_recovery();
    phase.set(BenchPhase::ServerKillRecovering);
    if !wait_for(&latest_recovered) {
        return;
    }
    phase.set(BenchPhase::BeforeDuplicates);
    if !wait_for(&reached(0.75)) {
        return;
    }
    let duplicates_before = control.duplicates_injected.load(Ordering::SeqCst);
    control.duplicate_every_frames.store(2, Ordering::SeqCst);
    log.record_injection("duplicate-frames", true);
    phase.set(BenchPhase::DuplicateFrames);
    wait_for(&|| {
        control.duplicates_injected.load(Ordering::SeqCst) - duplicates_before
            >= DUPLICATES_PER_INJECTION_RUN
    });
}

/// 压测器入口；回退出码：0 达标，1 没达标，2 用法错，3 没跑成（服务起不来、回应不对、读回对不上、完整性闸、被打断），
/// 4 环境不干净（loadavg 超阈值，不判），78 容错客户端连不上服务到了上限。
pub fn run_bench(arguments: &[String]) -> i32 {
    let settings = match parse_bench_arguments(arguments) {
        Ok(settings) => settings,
        Err(message) => {
            eprintln!("  ✗ {message}");
            eprintln!("  → 怎么办：{BENCH_USAGE}");
            return 2;
        }
    };
    let break_switches = if settings.is_selftest_instance {
        match BreakSwitches::from_environment() {
            Ok(switches) => switches,
            Err(message) => {
                eprintln!("  ✗ {message}");
                eprintln!("  → 怎么办：改对 PRUNE_SERVER_BREAK 的值，或不设它");
                return 2;
            }
        }
    } else {
        BreakSwitches::none()
    };
    if let Err(message) = install_termination_handlers() {
        eprintln!("  ✗ {message}");
        eprintln!("  → 怎么办：在能装信号处理的环境里跑");
        return 3;
    }
    let mut directory = match TemporaryLibraryDirectory::create(&break_switches) {
        Ok(directory) => directory,
        Err(message) => {
            eprintln!("  ✗ {message}");
            eprintln!("  → 怎么办：TMPDIR 指到一个可写的目录");
            return 3;
        }
    };
    println!("{LIBRARY_LINE_PREFIX} {}", directory.path.display());
    let outcome = run_with_server(&settings, &break_switches, &directory.path);
    let cleanup_failure = directory.remove_now().err();
    match (outcome, cleanup_failure) {
        (Ok(report), None) => {
            println!("{}", report.final_line);
            report.exit_code
        }
        (Ok(report), Some(message)) => {
            println!(
                "{}",
                report
                    .final_line
                    .replace(" result=met", " result=failed")
                    .replace(" result=missed", " result=failed")
                    .replace(" result=environment-dirty", " result=failed")
            );
            eprintln!("  ✗ {message}");
            eprintln!("  → 怎么办：手动删掉这个目录，看是谁还开着它");
            3
        }
        (Err(message), cleanup) => {
            println!(
                "{BENCH_LINE_PREFIX} mode={} target_states_per_second={} result=failed",
                settings.mode.name(),
                settings.target_states_per_second
            );
            eprintln!("  ✗ {message}");
            if let Some(cleanup_message) = cleanup {
                eprintln!("  ✗ {cleanup_message}");
            }
            eprintln!("  → 怎么办：看上面那一句卡在哪；服务起不来先看它自己报的错，读回对不上是服务或协议的 bug");
            3
        }
    }
}

struct BenchReport {
    final_line: String,
    exit_code: i32,
}

fn run_with_server(
    settings: &BenchSettings,
    break_switches: &BreakSwitches,
    directory: &Path,
) -> Result<BenchReport, String> {
    let load_at_start = read_load_averages().map(|averages| averages[0]);
    let device = block_device_for_path(directory);
    let mut started = start_server(settings, directory)?;
    println!(
        "{STARTED_LINE_PREFIX} server_process={} unix={} tcp={} worker_threads={}",
        started.process_identifier(),
        started.unix_socket.display(),
        started.network_address,
        settings.worker_threads
    );
    started.ticks_at_start = processor_ticks(started.process_identifier())
        .ok_or("读不到服务进程的 CPU 时间（/proc/<进程号>/stat）")?;
    let mut proxy = if settings.mode.injects_faults() {
        Some(FaultProxy::start(
            ProxyListen::UnixSocket(directory.join("proxy.sock")),
            RouteAddress::UnixSocket(started.unix_socket.clone()),
            // 代理的弄坏开关只给自证 server-kill-while-client-writes 那一格：压测里撞不撞得上看时机，不拿它证红。
            &BreakSwitches::none(),
        )?)
    } else {
        None
    };
    let plans = route_plans(settings, &started, proxy.as_ref().map(FaultProxy::route));
    let unix_socket = started.unix_socket.clone();
    let server_process = Arc::new(AtomicU32::new(started.process_identifier()));
    let mut server = started;
    let input_output_ticks_at_start = device.as_ref().and_then(read_input_output_ticks);
    let fault_log = Arc::new(FaultLog::default());
    let routes_finished = Arc::new(AtomicBool::new(false));
    let phase = Arc::new(PhaseCell::new(if settings.mode.injects_faults() {
        BenchPhase::BeforeDisconnect
    } else {
        BenchPhase::Steady
    }));
    let stalled = Arc::new(AtomicBool::new(false));
    let stall_description = Arc::new(Mutex::new(None));
    let route_progress: Vec<Arc<RouteProgress>> = plans
        .iter()
        .map(|plan| Arc::new(RouteProgress::new(plan.route.label())))
        .collect();
    // 服务子进程设了「起它的线程退出时收 SIGKILL」，所以 kill -9 之后的重起由起服务的这个主线程做，故障调度线程只发请求。
    let (restart_sender, restart_receiver) = mpsc::channel::<RestartReply>();
    let start = Instant::now();
    let watch = ProgressWatch {
        start,
        duration: settings.duration,
        interval: settings.progress_interval,
        stall_timeout: settings.stall_timeout,
        routes: route_progress.clone(),
        phase: Arc::clone(&phase),
        proxy: proxy.as_ref().map(FaultProxy::shared_control),
        server_process: Arc::clone(&server_process),
        stalled: Arc::clone(&stalled),
        stall_description: Arc::clone(&stall_description),
    };
    let watch_finished = Arc::new(AtomicBool::new(false));
    let watcher = {
        let watch_finished = Arc::clone(&watch_finished);
        spawn_named("bench-progress", move || watch.run(&watch_finished))?
    };
    let scheduler = match proxy.as_ref() {
        Some(proxy) => {
            let control = proxy.shared_control();
            let restart_sender = restart_sender.clone();
            let fault_log = Arc::clone(&fault_log);
            let routes_finished = Arc::clone(&routes_finished);
            let phase = Arc::clone(&phase);
            let duration = settings.duration;
            Some(spawn_named("fault-schedule", move || {
                run_fault_schedule(
                    start,
                    duration,
                    &control,
                    &restart_sender,
                    &fault_log,
                    &phase,
                    &routes_finished,
                );
            })?)
        }
        None => None,
    };
    let mut drivers: Vec<thread::JoinHandle<RouteOutcome>> = Vec::new();
    for (plan, progress) in plans.into_iter().zip(route_progress) {
        let journal_directory = directory.to_path_buf();
        let fault_log = settings
            .mode
            .injects_faults()
            .then(|| Arc::clone(&fault_log));
        let stalled = Arc::clone(&stalled);
        let client_switches =
            break_switches.only(BreakSwitch::JournalCompactsOnEveryAcknowledgement);
        drivers.push(spawn_named(
            &format!("route-{}", plan.route_tag),
            move || {
                drive_route(
                    &plan,
                    start,
                    &journal_directory,
                    fault_log.as_deref(),
                    &progress,
                    &stalled,
                    &client_switches,
                )
            },
        )?);
    }
    drop(restart_sender);
    let mut stalled_since: Option<Instant> = None;
    while !drivers.iter().all(thread::JoinHandle::is_finished) {
        if let Ok(reply) = restart_receiver.recv_timeout(Duration::from_millis(20)) {
            // 调度线程在等这个结果；它已经走了就没人收，丢掉。
            let restarted = server.kill_and_restart();
            server_process.store(server.process_identifier(), Ordering::SeqCst);
            let _ = reply.send(restarted);
        }
        if stalled.load(Ordering::SeqCst) {
            let since = *stalled_since.get_or_insert_with(Instant::now);
            if since.elapsed() >= STALL_EXIT_GRACE {
                break;
            }
        }
    }
    // 不再收重起请求：调度线程这之后发的请求、已经排着没回的请求都回错，不会一直等。
    drop(restart_receiver);
    let mut outcomes = Vec::new();
    let mut failures = Vec::new();
    if let Some(description) = stall_description
        .lock()
        .expect("卡住说明的锁只在判卡住与收尾时短暂持有")
        .clone()
    {
        failures.push(description);
    }
    for driver in drivers {
        if !driver.is_finished() {
            // 判了卡住、关了代理的连接、置了停标记，宽限期过了还没出来：不等它，这一趟照失败收尾（进程退出时它随之结束）。
            failures.push(format!(
                "判卡住之后 {} 秒，压测的一路线程还没退出",
                STALL_EXIT_GRACE.as_secs()
            ));
            continue;
        }
        match driver.join() {
            Ok(outcome) => outcomes.push(outcome),
            Err(_) => failures.push("压测的一路线程 panic 了".to_string()),
        }
    }
    routes_finished.store(true, Ordering::SeqCst);
    if let Some(handle) = scheduler {
        if handle.join().is_err() {
            failures.push("故障调度线程 panic 了".to_string());
        }
    }
    watch_finished.store(true, Ordering::SeqCst);
    if watcher.join().is_err() {
        failures.push("看进度的线程 panic 了".to_string());
    }
    let wall_milliseconds = start.elapsed().as_secs_f64() * 1000.0;
    let input_output_ticks_at_end = device.as_ref().and_then(read_input_output_ticks);
    let fault_lines = settle_faults(settings, proxy.as_ref(), &fault_log, start, &mut failures);
    if let Some(proxy) = proxy.as_mut() {
        proxy.stop();
    }
    let processor_ticks_used = server
        .processor_ticks_so_far()
        .ok_or("读不到服务进程的 CPU 时间（/proc/<进程号>/stat）")?;
    let peak_kibibytes = peak_resident_memory_kibibytes(server.process_identifier())
        .ok_or("读不到服务进程的内存峰值（/proc/<进程号>/status 的 VmHWM）")?;
    let server_counters = SimpleClient::connect(
        &RouteAddress::UnixSocket(unix_socket),
        BreakSwitches::none(),
    )
    .and_then(|mut client| client.statistics());
    let counters_line = match &server_counters {
        Ok(reported) => reported
            .iter()
            .map(|counter| format!("{}={}", counter.name, counter.value))
            .collect::<Vec<_>>()
            .join(" "),
        Err(message) => format!("读不到：{message}"),
    };
    let write_ahead_log_value = |name: &str| -> String {
        match &server_counters {
            Ok(reported) => statistics_value(reported, name)
                .map_or_else(|| "unknown".to_string(), |value| value.to_string()),
            Err(_) => "unknown".to_string(),
        }
    };
    let write_ahead_log_milliseconds = |name: &str| -> String {
        match &server_counters {
            Ok(reported) => statistics_value(reported, name).map_or_else(
                || "unknown".to_string(),
                |microseconds| format!("{:.3}", microseconds as f64 / 1000.0),
            ),
            Err(_) => "unknown".to_string(),
        }
    };
    let write_ahead_log_fields = format!(
        "wal_syncs={} wal_sync_p50_ms={} wal_sync_p99_ms={} wal_sync_max_ms={}",
        write_ahead_log_value(WRITE_AHEAD_LOG_SYNC_COUNT_COUNTER),
        write_ahead_log_milliseconds(WRITE_AHEAD_LOG_SYNC_MEDIAN_COUNTER),
        write_ahead_log_milliseconds(WRITE_AHEAD_LOG_SYNC_NINETY_NINTH_PERCENTILE_COUNTER),
        write_ahead_log_milliseconds(WRITE_AHEAD_LOG_SYNC_MAXIMUM_COUNTER)
    );
    drop(server);
    let load_at_end = read_load_averages().map(|averages| averages[0]);
    let disk_fields = match (
        &device,
        input_output_ticks_at_start,
        input_output_ticks_at_end,
    ) {
        (Some(device), Some(ticks_at_start), Some(ticks_at_end)) => format!(
            "disk_device={} disk_busy_percent={:.1}",
            device.name,
            busy_percent(ticks_at_start, ticks_at_end, wall_milliseconds)
        ),
        (Some(device), _, _) => format!("disk_device={} disk_busy_percent=unknown", device.name),
        (None, _, _) => "disk_device=unknown disk_busy_percent=unknown".to_string(),
    };
    let format_load = |load: Option<f64>| {
        load.map_or_else(|| "unknown".to_string(), |value| format!("{value:.2}"))
    };
    let environment = judge_environment(
        load_at_start,
        load_at_end,
        settings.maximum_load_average,
        break_switches,
    );
    let environment_fields = format!(
        "load_average_start={} load_average_end={} load_average_threshold={:.2} {disk_fields} {write_ahead_log_fields}",
        format_load(load_at_start),
        format_load(load_at_end),
        settings.maximum_load_average
    );
    for line in &fault_lines {
        println!("{line}");
    }

    let elapsed = outcomes
        .iter()
        .filter_map(|outcome| outcome.last_completion)
        .max()
        .map_or(settings.duration, |last| last.duration_since(start))
        .max(settings.duration);
    let mut all_latencies = Vec::new();
    let mut completed_states = 0_u64;
    let mut answered_frames = 0_u64;
    let mut records = 0_u64;
    let mut reconnections = 0_u64;
    let mut duplicate_responses = 0_u64;
    let mut busy_responses = 0_u64;
    let mut journal_totals = JournalStatistics::default();
    for outcome in &mut outcomes {
        outcome.latencies_microseconds.sort_unstable();
        println!(
            "PRUNE_SERVER_BENCH_ROUTE route={} target_states_per_second={} achieved_states_per_second={:.1} blocks={} frames_sent={} frames_answered={} records={} p50_ms={:.3} p99_ms={:.3} reconnections={} duplicate_responses={} busy_responses={} journal_appended_bytes={} journal_compactions={} journal_rewritten_bytes={} failure={}",
            outcome.label,
            outcome.target_states_per_second,
            per_second(outcome.completed_states, elapsed),
            outcome.completed_blocks,
            outcome.sent_frames,
            outcome.answered_frames,
            outcome.records,
            milliseconds(percentile_microseconds(&outcome.latencies_microseconds, 50)),
            milliseconds(percentile_microseconds(&outcome.latencies_microseconds, 99)),
            outcome.reconnections,
            outcome.duplicate_responses,
            outcome.busy_responses,
            outcome.journal.appended_bytes,
            outcome.journal.compactions,
            outcome.journal.rewritten_bytes,
            outcome.failure.as_deref().unwrap_or("none").replace(' ', "_")
        );
        all_latencies.extend_from_slice(&outcome.latencies_microseconds);
        completed_states += outcome.completed_states;
        answered_frames += outcome.answered_frames;
        records += outcome.records;
        reconnections += outcome.reconnections;
        duplicate_responses += outcome.duplicate_responses;
        busy_responses += outcome.busy_responses;
        journal_totals.appended_bytes += outcome.journal.appended_bytes;
        journal_totals.compactions += outcome.journal.compactions;
        journal_totals.rewritten_bytes += outcome.journal.rewritten_bytes;
        if let Some(failure) = &outcome.failure {
            failures.push(format!("{}：{failure}", outcome.label));
        }
    }
    println!("PRUNE_SERVER_BENCH_SERVER_COUNTERS {counters_line}");
    all_latencies.sort_unstable();
    let achieved = per_second(completed_states, elapsed);
    let judgment = judge_achieved_rate(achieved, settings.target_states_per_second, break_switches);
    let (result, exit_code) = match (failures.is_empty(), environment, judgment) {
        (false, _, _) => ("failed", 3),
        (true, EnvironmentJudgment::Dirty, _) => ("environment-dirty", ENVIRONMENT_DIRTY_EXIT_CODE),
        (true, EnvironmentJudgment::Clean, RateJudgment::Met) => ("met", 0),
        (true, EnvironmentJudgment::Clean, RateJudgment::Missed) => ("missed", 1),
    };
    for failure in &failures {
        eprintln!("  ✗ {failure}");
    }
    if !failures.is_empty() {
        eprintln!("  → 怎么办：看上面每一路的失败说明；完整性闸与读回对不上是服务或协议的 bug，被打断的重跑同一条命令");
    }
    if failures.is_empty() && environment == EnvironmentJudgment::Dirty {
        eprintln!(
            "  ✗ 环境不干净：起跑前 loadavg {}、跑完 {}，阈值 {:.2}；这一趟的吞吐不作数",
            format_load(load_at_start),
            format_load(load_at_end),
            settings.maximum_load_average
        );
        eprintln!("  → 怎么办：等机器闲下来（loadavg 1 分钟值不超过阈值）再跑同一条命令");
    }
    let mebibytes = peak_kibibytes as f64 / 1024.0;
    let processor_seconds = processor_ticks_used as f64 / ticks_per_second() as f64;
    let routes: Vec<&str> = outcomes.iter().map(|outcome| outcome.label).collect();
    let recovery_maximum = fault_recovery_maximum_milliseconds(&fault_log);
    let fault_fields = format!(
        "faults={} fault_recovery_max_ms={} client_reconnections={reconnections} duplicate_responses={duplicate_responses} busy_responses={busy_responses} \
journal_appended_bytes={} journal_compactions={} journal_rewritten_bytes={}",
        fault_log
            .records
            .lock()
            .expect("故障记录的锁只在记一笔时短暂持有")
            .len(),
        recovery_maximum.map_or_else(|| "none".to_string(), |maximum| format!("{maximum:.1}")),
        journal_totals.appended_bytes,
        journal_totals.compactions,
        journal_totals.rewritten_bytes,
    );
    let final_line = format!(
        "{BENCH_LINE_PREFIX} mode={} target_states_per_second={} achieved_states_per_second={achieved:.1} frames_per_second={:.1} p50_ms={:.3} p99_ms={:.3} server_peak_rss_mib={mebibytes:.1} server_cpu_seconds={processor_seconds:.2} records_per_second={:.1} layout={} states_per_block={} batch_blocks={} batch_milliseconds={} duration_seconds={} elapsed_seconds={:.3} routes={} required_percent={} {environment_fields} {fault_fields} result={result}",
        settings.mode.name(),
        settings.target_states_per_second,
        per_second(answered_frames, elapsed),
        milliseconds(percentile_microseconds(&all_latencies, 50)),
        milliseconds(percentile_microseconds(&all_latencies, 99)),
        per_second(records, elapsed),
        settings.layout.name(),
        settings.states_per_block,
        settings.mode.blocks_per_frame_limit(settings.batch_blocks),
        settings.batch_window.as_millis(),
        settings.duration.as_secs(),
        elapsed.as_secs_f64(),
        routes.join("+"),
        if break_switches.is_enabled(BreakSwitch::BenchJudgmentLenient) { LENIENT_PERCENT_OF_TARGET } else { REQUIRED_PERCENT_OF_TARGET },
    );
    Ok(BenchReport {
        final_line,
        exit_code,
    })
}
