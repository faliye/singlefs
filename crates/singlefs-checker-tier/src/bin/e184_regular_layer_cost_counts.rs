//! E184（常规层与大负载层的代价计数）全部子命令（跑前登记 `research/prompts/e184-preregistration.md` 5.3 第二份 bin）。
//!
// admission: always 每跑一次都在今天这份 crates 上重排计划、重取块键、重录固定流，产物里的挂钟每次都是新观测；已有写完的转储与块键文件由各子命令自己整份跳过
// run-condition: command python3
//!
//! 入库装置（登记「装置写在哪」）：读主工作区现状的 `crates/`，块键一律调 crates 的函数算（`crash_points_of_the_steps` 切节点、
//! `plan_crash_points` 排计划、`PlannedCrashPoint::block_key` 取块键），不另写一份。
//! 转储的写法与读法只有 `e184_regular_layer_cost_counts_write_dump.rs` 那一份，这里经 `#[path]` 当模块引进来。
//!
//! 子命令：
//!   selfcheck                                     停机 T1–T4 与第七节锚点逐条核（`name=stop_check`、`name=anchor`）
//!   record-fixed-chains --out <目录>              覆盖写链三档与冒烟层四条流各写一份转储
//!   keys --dump <目录>… --out <目录> [--controls] 逐份转储排计划、取块键，写块键文件，打 `name=flow` 与阳性对照
//!   share --candidates <目录>… --against <目录>… --out <文件> [--pinned-smoke <文件>]   共用块键与态数区间重叠
//!   library --keys <目录>… --library <绝对路径>    只读开长期库，数候选流的块键在录入表里的个数（附带）
//!   compare --before <目录> --after <目录> --kind <commit-pair|patch> --pair <名> --out <文件>   作废块数
//!   compose --j1 <文件> --keys <目录>… --j3 <文件> --j2-small <日志> --j2-medium <日志> --library-bytes <文件> --recorded-blocks <文件>   提交那一趟的挂钟
//!
//! 线程数取 `E184_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值（转储模块里那一份读法）。
//! 块数上界（总态数除每块态数向上取整，加每点一块的零头）超过 2^26 的展开不排计划、不取块键，`name=flow` 的 `blocks_full` / `blocks_quick` 写 `not_planned`，
//! 态数照报（登记第十二节修订 10）；转储往返对照比块键的上限再小一档，2^24。
//! 产物是 `E7RESULT` 行。退出码：0 正常；2 命令行不对；3 停机条款没过（产物照打、交主 agent）；4 阳性对照没过（作废）；
//! 78 准入与运行条件没满足（`preflight` 拒绝，带 `--force` 才照跑；强制跑的那一次，标准输出第一行多一行 `name=preflight forced=…`，`emitted=` 不数它）。
//! `preflight` 的实现只有转储模块里那一份（`write_dump::preflight`），这里传自己的文件名。
//!
//! 照抄的段与停机 T2（登记第十一节）：冒烟层三条流的建法照抄各测试文件里私有的 `prepare` 和它调到的私有函数，装置里用 `COPIED-BEGIN` / `COPIED-END`
//! 两行标记框出每一段；`selfcheck` 的 `copied_text_verbatim` 把每一段去掉缩进之后在对应的测试文件原文里逐字查一次（原文经 `include_str!` 编进来），
//! 查不到就停（退 3）。出处（文件在 `crates/singlefs-checker-tier/tests/` 下，起止行是登记写这一份时现读的）：
//!   fixed_script_head      crash_enumeration_fixed_script_stream.rs:53-90      第二份内容常量、`Prepared`、`Script`
//!   fixed_script_body      crash_enumeration_fixed_script_stream.rs:131-561    谱系常量到 `prepare` 末尾（`overwrite`、`unmount_recorded`、`writes_and_segments_after_mkfs` 在内）
//!   multi_record_publish   crash_enumeration_multi_record_publish_stream.rs:74-218   `MultiRecordPublish` 到 `prepare` 末尾
//!   new_pool_prepare_body  crash_enumeration_new_pool_file_creation_stream.rs:795-813   `prepare` 的前半（到被判的根槽写那一步）
//! 允许的替换只有一种：包进 `mod` 的缩进（每行前多 4 个空格）。`crash_points_of_the_steps` 不再照抄：主 agent 已把它改成 `pub`，装置直接调
//! `singlefs_checker_tier::crash_state_enumeration::crash_points_of_the_steps`。建快照流直接调公共模块的 `prepare_the_snapshot_crash_flow`，不抄。
//!
//! 决定工作量、几何或判据门槛的常量都是本地常量（值抄自登记位，注释写出处），`selfcheck` 与 crates 的同名常量回比（停机 T1）：
//! 每块状态数 65536、种子基、闭式能算的最长段 63。步数档、种子数、盘宽档、比重组、覆盖写链的单元数、补丁位、提交清单不从 `crates/` 算，在登记第五节写死。

#[path = "../../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../tests/common_crash_points/mod.rs"]
mod common_crash_points;
#[path = "../../../singlefs-harness/tests/common_snapshot_prefix/mod.rs"]
mod common_snapshot_prefix;
#[path = "e184_regular_layer_cost_counts_write_dump.rs"]
#[allow(
    dead_code,
    reason = "转储模块是另一份 bin 的整份源文件：它的命令行入口与随机历史的并行录制这个装置不调，只用转储格式、读回、录一条历史和线程数取法"
)]
mod write_dump;

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion, quick_tier_expansion,
    CrashPointReplaySegmentExpansion, StateCount1280Bits, TearableInPlaceOverwrites,
};
use singlefs_checker_tier::crash_amplification::{plan_crash_points, CrashFlow, STATES_PER_BLOCK};
use singlefs_checker_tier::crash_library::{
    absolute_library_directory_of_the_variable_value, DisposableLibraryDirectory,
};
use singlefs_checker_tier::crash_state_enumeration::{
    crash_points_of_the_steps, CrashPointReplayItem,
};
use singlefs_checker_tier::verdict_store::{
    BlockStartStateIndex, EnumerationPlanHash, NodeInputDigest, VerdictBlockKey, VerdictStore,
};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::{
    closed_form_state_count, root_identity_written_by, writes_and_segments, MemoryPool,
    PublishedVersion, PublishedVersionWithoutAFile, RetainedWrite, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::sha256_hexadecimal;
use singlefs_harness::RecordedOperationKind;

use write_dump::{Emitter, FlowDump, SEED_BASE_OF_THE_RANDOM_HISTORIES};

// ───────────────────────────── 本地常量（停机 T1 回比） ─────────────────────────────

/// 每块状态数：值抄自 `crates/singlefs-checker-tier/src/crash_amplification.rs` 的 `STATES_PER_BLOCK`（D13 已定项 16 没写块长，块长只在实现里）；
/// `selfcheck` 与 crates 那一份回比。
const LOCAL_STATES_PER_BLOCK: u64 = 65_536;
/// 闭式能算的最长段写数：值抄自 D13 已定项 16「整域装不进 u64」与 `memory_pool::closed_form_state_count` 的注释（64 写的段 2^64 装不进 u64）；
/// `selfcheck` 核 crates 的闭式函数在 64 写的段上 panic。
const LONGEST_SEGMENT_WITH_A_CLOSED_FORM: usize = 63;
/// 登记 1.1 节「上千」读作写数不少于它。
const WRITES_OF_A_THOUSAND_WRITE_STREAM: usize = 1000;
/// 一条流排计划时块数上界的上限（登记第十二节修订 10）：2^26 块。`plan_blocks` 把每个点的块一次摆进内存（`PlannedCrashPoint::blocks` 交回一个
/// 按块数长的 `Vec<Range<u64>>`，每块 16 字节，再加每块一条 `BlockOfAPlan`），块数没上限时一个 56 写的段（覆盖写链 u = 16 每次覆盖写的单元段）
/// 就是 2^40 块、`Vec` 要 2^44 字节。这是工程上限，不是判据：超过它的流照报态数，只是不排计划、不取块键（和「装不进 u64」同一种待遇）。
const MOST_BLOCKS_PLANNED_PER_FLOW: u64 = 1 << 26;
/// 转储往返对照（`direct_base_control`）比块键时块数上界的上限：2^24 块。对照要同时摆两份计划，比 [`MOST_BLOCKS_PLANNED_PER_FLOW`] 再小一档；
/// 超过它的流只比基镜像。
const MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP: u64 = 1 << 24;

// ───────────────────────────── 判据的门槛（登记第六节，跑前写死） ─────────────────────────────

/// J1-乙：落在冒烟层态数区间里的占比不小于它记「重叠」。
const INSIDE_FRACTION_AT_LEAST_FOR_OVERLAP: f64 = 0.5;
/// J1-乙：不大于它记「分开」。
const INSIDE_FRACTION_AT_MOST_FOR_SEPARATE: f64 = 0.1;
/// J1-丙：族中位不小于它记「换名」。
const SHARE_MEDIAN_AT_LEAST_FOR_RENAMED: f64 = 0.5;
/// J1-丙：族最大不大于它记「独立」。
const SHARE_MAXIMUM_AT_MOST_FOR_INDEPENDENT: f64 = 0.05;
/// J3-乙：中位不小于它记「写路径提交大多作废过半」。
const INVALIDATED_MEDIAN_AT_LEAST_FOR_MORE_THAN_HALF: f64 = 0.5;
/// J3-乙：中位不大于它记「块级复用省下大半」。
const INVALIDATED_MEDIAN_AT_MOST_FOR_MOST_REUSED: f64 = 0.1;
/// J3-乙：算得出的提交对数不到它记「还差」（G2 的够判条件）。
const COMMIT_PAIRS_NEEDED_FOR_ENOUGH: usize = 10;
/// 挂钟档的界：2 小时取自 `scripts/hosts/detached-run.sh` 的 `--wall-clock within-2h / over-2h` 分界，24 小时是一天；两个界都不是从哪条臂的定义推出来的。
const SECONDS_OF_TWO_HOURS: f64 = 7200.0;
const SECONDS_OF_A_DAY: f64 = 86400.0;

/// 退出码（登记 5.3）。
const EXIT_CODE_OK: i32 = 0;
const EXIT_CODE_USAGE: i32 = 2;
const EXIT_CODE_STOP_CLAUSE_FAILED: i32 = 3;
const EXIT_CODE_POSITIVE_CONTROL_FAILED: i32 = 4;

/// 节点的设点文件：这个装置自己的仓内路径（名字与路径都不进块键，登记 B3）。
const SETTING_FILE_OF_THE_FLOWS: &str =
    "crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs";
/// 装置里排计划用的项名；换一个项名再排一遍是阳性对照「名字不进键」。
const ITEM_NAME_OF_THE_FLOWS: &str = "e184_flow";
const OTHER_ITEM_NAME_FOR_THE_NAME_CONTROL: &str = "e184_flow_under_another_name";
const OTHER_SETTING_FILE_FOR_THE_NAME_CONTROL: &str =
    "crates/singlefs-checker-tier/tests/e184_the_same_flow_under_another_file.rs";

/// 字段值：装不进 u64 的数写它，不报数（登记 1.1）。
const BEYOND_U64: &str = "beyond_u64";

// ───────────────────────────── 排计划、取块键 ─────────────────────────────

/// 展开方式：全量（每段任意真子集，原地覆写取第三态），或平时快档（甲二）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expansion {
    Full,
    QuickTier,
}

impl Expansion {
    fn name(self) -> &'static str {
        match self {
            Expansion::Full => "full",
            Expansion::QuickTier => "quick",
        }
    }

    fn function(
        self,
    ) -> &'static (dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync) {
        match self {
            Expansion::Full => &full_expansion,
            Expansion::QuickTier => &quick_tier_expansion,
        }
    }
}

/// 计划里的一块：块键（crates 的 `PlannedCrashPoint::block_key` 算的）、这一块的态数、它属于计划里第几个崩溃点。
#[derive(Clone, Debug, PartialEq, Eq)]
struct BlockOfAPlan {
    key: VerdictBlockKey,
    states: u64,
    point_index: usize,
}

/// 一条流在一种展开下排出来的计划，只留这个装置要数的东西。
#[derive(Debug)]
struct PlannedBlocks {
    /// 计划里最后一个点（流的末节点）的状态序号尾：整条流的状态数。
    state_count: u64,
    /// `crash_state_count_with_torn_in_place_overwrites` 数的整条流状态数（登记停机 T4：与上一个相等）。
    state_count_by_the_counting_function: u64,
    point_count: usize,
    blocks: Vec<BlockOfAPlan>,
    /// 不管任何段的点（末节点在内）的块数：两种展开的计划哈希在这些点上相同（登记 5.2「同流两种展开」）。
    blocks_of_points_without_segments: usize,
}

/// 整条流在 `expansion` 下的状态数，按 1280 位算；装不进 u64 交 `None`（不 panic：登记 1.1「装不进 u64 记 beyond_u64」）。
fn state_count_of_the_flow(
    base: &MemoryPool,
    dump: &FlowDump,
    expansion: Expansion,
) -> Option<u64> {
    let tearable = TearableInPlaceOverwrites::of(base, &dump.writes);
    let mut total = StateCount1280Bits::from_u64(1);
    for (segment_index, segment) in dump.segments.iter().enumerate() {
        let segment_count = expansion.function()(segment_index, segment).checked_state_count(
            &dump.writes,
            segment,
            &tearable,
        )?;
        total = total.checked_add(&segment_count)?;
    }
    total.to_u64()
}

/// 一条流（`segment_count` 段、`states` 个状态）切出的块数的上界：总态数除每块态数向上取整，再加每个点最多一块的零头；
/// 点数不超过段数加末节点（`crash_points_of_the_steps` 一步至少含一段），这里多留 1 的余量。不排计划就能算，
/// 也就是在 `PlannedCrashPoint::blocks` 把块一次摆进内存之前拿它挡。
fn block_count_bound_of(states: u64, segment_count: usize) -> u64 {
    blocks_needed_for(states).saturating_add(
        u64::try_from(segment_count)
            .unwrap_or(u64::MAX)
            .saturating_add(2),
    )
}

/// 整条流在 `expansion` 下要排计划的态数：装得进 u64、且块数上界不超过 `most_blocks` 才有值；
/// 没有值的流照报态数（[`state_count_of_the_flow`]），只是不排计划、不取块键。
fn state_count_to_plan(
    base: &MemoryPool,
    dump: &FlowDump,
    expansion: Expansion,
    most_blocks: u64,
) -> Option<u64> {
    state_count_of_the_flow(base, dump, expansion)
        .filter(|states| block_count_bound_of(*states, dump.segments.len()) <= most_blocks)
}

/// 两态闭式 1 + Σ(2^|段| − 1) 的本地复算；有一段不少于 64 写，或总和装不进 u64，交 `None`。
fn two_state_closed_form(segments: &[Vec<usize>]) -> Option<u64> {
    let mut total: u64 = 1;
    for segment in segments {
        if segment.len() > LONGEST_SEGMENT_WITH_A_CLOSED_FORM {
            return None;
        }
        let shift = u32::try_from(segment.len()).expect("段写数不超过 63，装得进 u32");
        total = total.checked_add((1u64 << shift) - 1)?;
    }
    Some(total)
}

/// 最长段：段写数的最大值（登记 1.1）。
fn longest_segment_of(segments: &[Vec<usize>]) -> usize {
    segments.iter().map(Vec::len).max().unwrap_or(0)
}

/// 按本地的块长，`states` 个状态切几块。
fn blocks_needed_for(states: u64) -> u64 {
    states.div_ceil(LOCAL_STATES_PER_BLOCK)
}

/// 按 crates 的算法排计划、取每块的块键：节点照崩溃枚举用例那一路切（`crash_points_of_the_steps`），计划由 `plan_crash_points` 排。
///
/// # Errors
/// 写表里没有根槽 FUA 写（被判的根没有）；整条流的态数装不进 u64；块数上界超过 [`MOST_BLOCKS_PLANNED_PER_FLOW`]。
///
/// # Panics
/// `plan_crash_points` 自己的断言（调用方包一层 `catch_unwind`）。
fn plan_blocks(
    base: &MemoryPool,
    dump: &FlowDump,
    expansion: Expansion,
    item: CrashPointReplayItem<'_>,
) -> Result<PlannedBlocks, String> {
    plan_blocks_under(base, dump, expansion, item, MOST_BLOCKS_PLANNED_PER_FLOW)
}

/// [`plan_blocks`]，块数上界的上限由调用方给（单测拿小上限在真流上走拒绝那一支）。
fn plan_blocks_under(
    base: &MemoryPool,
    dump: &FlowDump,
    expansion: Expansion,
    item: CrashPointReplayItem<'_>,
    most_blocks: u64,
) -> Result<PlannedBlocks, String> {
    let judged_root_index = dump
        .judged_root_index
        .ok_or_else(|| format!("{}：写表里没有根槽 FUA 写，排不了计划", dump.label))?;
    match state_count_of_the_flow(base, dump, expansion) {
        None => return Err(format!("{}：整条流的态数装不进 u64，不排计划", dump.label)),
        Some(states) => {
            let bound = block_count_bound_of(states, dump.segments.len());
            if bound > most_blocks {
                return Err(format!(
                    "{}：整条流 {states} 个状态，块数上界 {bound} 超过上限 {most_blocks}，不排计划",
                    dump.label
                ));
            }
        }
    }
    let flow = CrashFlow {
        base,
        writes: &dump.writes,
        segments: &dump.segments,
        judged_root_index,
        versions: &dump.versions_with_a_file,
        versions_without_a_file: &dump.versions_without_a_file,
        expansion: expansion.function(),
        crash_points: crash_points_of_the_steps(item, &dump.writes, &dump.segments),
    };
    let plan = plan_crash_points(&flow);
    let mut blocks = Vec::new();
    let mut blocks_of_points_without_segments = 0usize;
    for (point_index, point) in plan.points.iter().enumerate() {
        for range in point.blocks() {
            blocks.push(BlockOfAPlan {
                key: point.block_key(&range),
                states: range.end - range.start,
                point_index,
            });
            if point.segments.is_empty() {
                blocks_of_points_without_segments += 1;
            }
        }
    }
    Ok(PlannedBlocks {
        state_count: plan
            .points
            .last()
            .map_or(0, |end_of_flow| end_of_flow.ordinals.end),
        state_count_by_the_counting_function: crash_state_count_with_torn_in_place_overwrites(
            base,
            &dump.writes,
            &dump.segments,
            expansion.function(),
        ),
        point_count: plan.points.len(),
        blocks,
        blocks_of_points_without_segments,
    })
}

/// 默认的项：名字与设点文件都是这个装置自己的（不进块键）。
fn default_item() -> CrashPointReplayItem<'static> {
    CrashPointReplayItem {
        name: ITEM_NAME_OF_THE_FLOWS,
        setting_file: SETTING_FILE_OF_THE_FLOWS,
    }
}

// ───────────────────────────── 一条流的一行（`name=flow`）与块键文件 ─────────────────────────────

fn count_field(count: Option<u64>) -> String {
    count.map_or_else(|| BEYOND_U64.to_string(), |value| value.to_string())
}

fn log2_field(count: Option<u64>) -> String {
    count.map_or_else(
        || BEYOND_U64.to_string(),
        |value| format!("{:.4}", (value as f64).log2()),
    )
}

/// 一行里 `名=值` 的字段（空白隔开；值里可以再有 `=`，只在第一个 `=` 处切）。
fn fields_of(body: &str) -> BTreeMap<String, String> {
    body.split_whitespace()
        .filter_map(|word| word.split_once('='))
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect()
}

fn required_field<'fields>(
    fields: &'fields BTreeMap<String, String>,
    name: &str,
) -> Result<&'fields str, String> {
    fields
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("缺字段 {name}"))
}

fn parse_count_field(text: &str) -> Result<Option<u64>, String> {
    if text == BEYOND_U64 {
        return Ok(None);
    }
    text.parse::<u64>()
        .map(Some)
        .map_err(|error| format!("{text:?} 不是整数也不是 {BEYOND_U64}：{error}"))
}

/// `name=flow` 那一行的内容（登记第六节 J1-甲）。
#[derive(Clone, Debug, PartialEq)]
struct FlowRow {
    family: String,
    tier: String,
    label: String,
    seed: Option<u64>,
    ending: String,
    writes: usize,
    segments: usize,
    longest_segment: usize,
    two_state: Option<u64>,
    full: Option<u64>,
    quick_tier: Option<u64>,
    /// 没排计划（整条流态数装不进 u64、块数上界超过 [`MOST_BLOCKS_PLANNED_PER_FLOW`]，或写表里没有根槽写）是 `None`；
    /// 态数（`full`、`quick_tier`）照报，块数是态数除每块态数。
    blocks_full: Option<usize>,
    blocks_quick: Option<usize>,
    points: Option<usize>,
    /// 这条流在进程里排计划、取块键用的秒数（两种展开合计）。
    keys_seconds: f64,
    /// 本地复算的两态闭式与 crates 的 `closed_form_state_count` 是否相同；闭式算不出（有段不少于 64 写）时 `None`。
    two_state_agrees_with_crates: Option<bool>,
}

impl FlowRow {
    fn body(&self) -> String {
        let planned = |count: Option<usize>| {
            count.map_or_else(|| "not_planned".to_string(), |value| value.to_string())
        };
        format!(
            "name=flow family={} tier={} label={} seed={} ending={} writes={} segments={} longest_segment={} two_state={} log2_two_state={} full={} quick_tier={} blocks_full={} blocks_quick={} points={} keys_seconds={:.1} two_state_agrees_with_crates={}",
            self.family,
            self.tier,
            self.label,
            self.seed.map_or_else(|| "none".to_string(), |seed| seed.to_string()),
            self.ending,
            self.writes,
            self.segments,
            self.longest_segment,
            count_field(self.two_state),
            log2_field(self.two_state),
            count_field(self.full),
            count_field(self.quick_tier),
            planned(self.blocks_full),
            planned(self.blocks_quick),
            planned(self.points),
            self.keys_seconds,
            self.two_state_agrees_with_crates.map_or_else(|| "not_applicable".to_string(), |agrees| agrees.to_string()),
        )
    }

    /// [`Self::body`] 的逆。
    fn from_body(body: &str) -> Result<Self, String> {
        let fields = fields_of(body);
        let usize_field = |name: &str| -> Result<usize, String> {
            required_field(&fields, name)?
                .parse::<usize>()
                .map_err(|error| format!("{name}：{error}"))
        };
        let planned = |name: &str| -> Result<Option<usize>, String> {
            match required_field(&fields, name)? {
                "not_planned" => Ok(None),
                text => text
                    .parse::<usize>()
                    .map(Some)
                    .map_err(|error| format!("{name}：{error}")),
            }
        };
        let seed = match required_field(&fields, "seed")? {
            "none" => None,
            text => Some(
                text.parse::<u64>()
                    .map_err(|error| format!("seed：{error}"))?,
            ),
        };
        Ok(Self {
            family: required_field(&fields, "family")?.to_string(),
            tier: required_field(&fields, "tier")?.to_string(),
            label: required_field(&fields, "label")?.to_string(),
            seed,
            ending: required_field(&fields, "ending")?.to_string(),
            writes: usize_field("writes")?,
            segments: usize_field("segments")?,
            longest_segment: usize_field("longest_segment")?,
            two_state: parse_count_field(required_field(&fields, "two_state")?)?,
            full: parse_count_field(required_field(&fields, "full")?)?,
            quick_tier: parse_count_field(required_field(&fields, "quick_tier")?)?,
            blocks_full: planned("blocks_full")?,
            blocks_quick: planned("blocks_quick")?,
            points: planned("points")?,
            keys_seconds: required_field(&fields, "keys_seconds")?
                .parse::<f64>()
                .map_err(|error| format!("keys_seconds：{error}"))?,
            two_state_agrees_with_crates: match required_field(
                &fields,
                "two_state_agrees_with_crates",
            )? {
                "not_applicable" => None,
                text => Some(
                    text.parse::<bool>()
                        .map_err(|error| format!("two_state_agrees_with_crates：{error}"))?,
                ),
            },
        })
    }

    /// 这条流在 `groups` 里的组名：流名去掉末尾的 `-seedNN`（同一族同一取样点的几个种子是一组）。
    fn group(&self) -> String {
        group_of_the_label(&self.label)
    }
}

fn group_of_the_label(label: &str) -> String {
    match label.rsplit_once("-seed") {
        Some((group, index))
            if !index.is_empty() && index.chars().all(|digit| digit.is_ascii_digit()) =>
        {
            group.to_string()
        }
        Some(_) | None => label.to_string(),
    }
}

// ───────────────────────────── 块键文件 ─────────────────────────────

const KEYS_FILE_HEADER_PREFIX: &str = "# E184KEYS version=1 expansion=";
const KEYS_FILE_ROW_PREFIX: &str = "# row ";
const KEYS_FILE_TRAILER_PREFIX: &str = "# keys_done blocks=";
const FULL_KEYS_FILE_SUFFIX: &str = ".keys";
const QUICK_KEYS_FILE_SUFFIX: &str = ".quick.keys";
const DUMP_FILE_SUFFIX: &str = ".dump";

/// 块键文件的内容写进 `writer`：头一行写展开方式，第二行抄这条流的 `name=flow` 行，之后每块一行（复用键、计划哈希、块起点、这一块的态数全是十六进制，
/// 最后是点序号），末行写块数——没有末行的文件当没写完。
fn write_keys_file_contents(
    writer: &mut impl std::io::Write,
    row: &FlowRow,
    expansion: Expansion,
    blocks: &[BlockOfAPlan],
) -> std::io::Result<()> {
    writeln!(
        writer,
        "{KEYS_FILE_HEADER_PREFIX}{}\n{KEYS_FILE_ROW_PREFIX}{}",
        expansion.name(),
        row.body()
    )?;
    for block in blocks {
        writeln!(
            writer,
            "{} {} {:x} {:x} {}",
            hexadecimal_text(&block.key.node_input_digest.0),
            hexadecimal_text(&block.key.enumeration_plan_hash.0),
            block.key.block_start.0,
            block.states,
            block.point_index
        )?;
    }
    writeln!(writer, "{KEYS_FILE_TRAILER_PREFIX}{}", blocks.len())
}

#[cfg(test)]
fn keys_file_text(row: &FlowRow, expansion: Expansion, blocks: &[BlockOfAPlan]) -> String {
    let mut bytes = Vec::new();
    write_keys_file_contents(&mut bytes, row, expansion, blocks).expect("写进内存不会失败");
    String::from_utf8(bytes).expect("块键文件里只有 ASCII 与流名")
}

/// 块键文件写到 `path`：先写同目录的临时名（边算边写，不先拼成一整个字符串：最大的流有上千万块）、同步、再改名。
fn write_keys_file_atomically(
    path: &Path,
    row: &FlowRow,
    expansion: Expansion,
    blocks: &[BlockOfAPlan],
) -> Result<(), String> {
    let temporary = path.with_extension("partial");
    let created = std::fs::File::create(&temporary)
        .map_err(|error| format!("建 {} 失败：{error}", temporary.display()))?;
    let mut writer = std::io::BufWriter::new(created);
    write_keys_file_contents(&mut writer, row, expansion, blocks)
        .map_err(|error| format!("写 {} 失败：{error}", temporary.display()))?;
    let flushed = writer
        .into_inner()
        .map_err(|error| format!("写 {} 失败：{error}", temporary.display()))?;
    flushed
        .sync_all()
        .map_err(|error| format!("同步 {} 失败：{error}", temporary.display()))?;
    std::fs::rename(&temporary, path).map_err(|error| {
        format!(
            "{} 改名为 {} 失败：{error}",
            temporary.display(),
            path.display()
        )
    })
}

fn digest_from_hexadecimal(text: &str) -> Result<[u8; 32], String> {
    if text.len() != 64 || !text.is_ascii() {
        return Err(format!("摘要要 64 个十六进制字符，读到 {text:?}"));
    }
    let mut digest = [0u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[2 * index..2 * index + 2], 16)
            .map_err(|error| format!("摘要 {text:?}：{error}"))?;
    }
    Ok(digest)
}

/// 读回的块键文件。
#[derive(Clone, Debug, PartialEq)]
struct KeysFile {
    expansion: Expansion,
    row: FlowRow,
    blocks: Vec<BlockOfAPlan>,
}

/// 块键文件读到的东西：头、`name=flow` 行、块（`keep_blocks` 是假时只数不存）、末行说的块数与实际数到的块数。
struct KeysFileContents {
    expansion: Expansion,
    row: FlowRow,
    blocks: Vec<BlockOfAPlan>,
    counted_blocks: usize,
    declared_blocks: Option<usize>,
}

fn block_of_the_line(line: &str) -> Result<BlockOfAPlan, String> {
    let words: Vec<&str> = line.split(' ').collect();
    let [reuse_key, plan_hash, block_start, states, point_index] = words.as_slice() else {
        return Err(format!("块键行要 5 段，读到 {line:?}"));
    };
    Ok(BlockOfAPlan {
        key: VerdictBlockKey {
            node_input_digest: NodeInputDigest(digest_from_hexadecimal(reuse_key)?),
            enumeration_plan_hash: EnumerationPlanHash(digest_from_hexadecimal(plan_hash)?),
            block_start: BlockStartStateIndex(
                u64::from_str_radix(block_start, 16).map_err(|error| format!("块起点：{error}"))?,
            ),
        },
        states: u64::from_str_radix(states, 16).map_err(|error| format!("态数：{error}"))?,
        point_index: point_index
            .parse::<usize>()
            .map_err(|error| format!("点序号：{error}"))?,
    })
}

fn read_keys_file_contents(
    reader: impl std::io::BufRead,
    keep_blocks: bool,
) -> Result<KeysFileContents, String> {
    let mut lines = reader
        .lines()
        .map(|line| line.map_err(|error| format!("读块键文件失败：{error}")));
    let header = lines.next().ok_or("块键文件是空的")??;
    let expansion = match header.strip_prefix(KEYS_FILE_HEADER_PREFIX) {
        Some("full") => Expansion::Full,
        Some("quick") => Expansion::QuickTier,
        Some(other) => return Err(format!("展开方式认不出：{other:?}")),
        None => return Err("头一行不是块键文件的头".to_string()),
    };
    let row_line = lines.next().ok_or("缺 name=flow 行")??;
    let row = FlowRow::from_body(
        row_line
            .strip_prefix(KEYS_FILE_ROW_PREFIX)
            .ok_or("第二行不是 # row")?,
    )?;
    let (mut blocks, mut counted_blocks, mut declared_blocks) = (Vec::new(), 0usize, None);
    for line in lines {
        let line = line?;
        if let Some(count) = line.strip_prefix(KEYS_FILE_TRAILER_PREFIX) {
            declared_blocks = Some(
                count
                    .parse::<usize>()
                    .map_err(|error| format!("末行块数：{error}"))?,
            );
            continue;
        }
        if declared_blocks.is_some() {
            return Err("末行之后还有内容".to_string());
        }
        counted_blocks += 1;
        if keep_blocks {
            blocks.push(block_of_the_line(&line)?);
        }
    }
    Ok(KeysFileContents {
        expansion,
        row,
        blocks,
        counted_blocks,
        declared_blocks,
    })
}

impl KeysFile {
    /// [`write_keys_file_contents`] 的逆。头、`name=flow` 行、末行块数与实际块数，任何一样对不上都交回说明。
    fn from_reader(reader: impl std::io::BufRead) -> Result<Self, String> {
        let contents = read_keys_file_contents(reader, true)?;
        match contents.declared_blocks {
            Some(count) if count == contents.counted_blocks => Ok(Self {
                expansion: contents.expansion,
                row: contents.row,
                blocks: contents.blocks,
            }),
            Some(count) => Err(format!(
                "末行说 {count} 块，实际读到 {}",
                contents.counted_blocks
            )),
            None => Err("没有 keys_done 末行：没写完".to_string()),
        }
    }

    #[cfg(test)]
    fn from_text(text: &str) -> Result<Self, String> {
        Self::from_reader(text.as_bytes())
    }

    fn read_from(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path)
            .map_err(|error| format!("读 {} 失败：{error}", path.display()))?;
        Self::from_reader(std::io::BufReader::new(file))
            .map_err(|error| format!("{}：{error}", path.display()))
    }

    /// 只看头、`name=flow` 行与末行（不存块）：文件写完了没有，说的是哪条流。
    fn read_row_of_a_finished_file(path: &Path) -> Result<(Expansion, FlowRow), String> {
        let file = std::fs::File::open(path)
            .map_err(|error| format!("读 {} 失败：{error}", path.display()))?;
        let contents = read_keys_file_contents(std::io::BufReader::new(file), false)
            .map_err(|error| format!("{}：{error}", path.display()))?;
        match contents.declared_blocks {
            Some(count) if count == contents.counted_blocks => {
                Ok((contents.expansion, contents.row))
            }
            Some(_) | None => Err(format!("{}：没写完", path.display())),
        }
    }

    fn state_total(&self) -> u64 {
        self.blocks.iter().map(|block| block.states).sum()
    }
}

/// `directories` 里某种后缀的文件，按文件名排；目录读不了交回说明。
fn files_with_suffix(
    directories: &[PathBuf],
    suffix: &str,
    excluded_suffix: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for directory in directories {
        let entries = std::fs::read_dir(directory)
            .map_err(|error| format!("读目录 {} 失败：{error}", directory.display()))?;
        for entry in entries {
            let path = entry
                .map_err(|error| format!("读目录 {} 失败：{error}", directory.display()))?
                .path();
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            if name.ends_with(suffix)
                && !excluded_suffix.is_some_and(|excluded| name.ends_with(excluded))
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// 目录里的块键文件路径（`expansion` 那一种）。
fn keys_file_paths(directories: &[PathBuf], expansion: Expansion) -> Result<Vec<PathBuf>, String> {
    match expansion {
        Expansion::Full => files_with_suffix(
            directories,
            FULL_KEYS_FILE_SUFFIX,
            Some(QUICK_KEYS_FILE_SUFFIX),
        ),
        Expansion::QuickTier => files_with_suffix(directories, QUICK_KEYS_FILE_SUFFIX, None),
    }
}

/// 读一份块键文件，展开方式要与文件名后缀对得上。
fn read_keys_file(path: &Path, expansion: Expansion) -> Result<KeysFile, String> {
    let keys = KeysFile::read_from(path)?;
    if keys.expansion == expansion {
        Ok(keys)
    } else {
        Err(format!("{}：展开方式与文件名后缀对不上", path.display()))
    }
}

/// 目录里的全部块键文件（`expansion` 那一种）；任何一份读不回都交回说明（不当没有）。
fn read_keys_files(directories: &[PathBuf], expansion: Expansion) -> Result<Vec<KeysFile>, String> {
    keys_file_paths(directories, expansion)?
        .iter()
        .map(|path| read_keys_file(path, expansion))
        .collect()
}

// ───────────────────────────── 读一条流：规模、计划、块键 ─────────────────────────────

/// 一条流读完的结果：`name=flow` 那一行，两种展开各自的计划，和读的过程中发现的、crates 两条路给的数对不上的地方。
#[derive(Debug)]
struct FlowReading {
    row: FlowRow,
    full: Option<PlannedBlocks>,
    quick: Option<PlannedBlocks>,
    /// 登记停机 T4：同一条流上 crates 的两条路给的全量态数不等（计划的末节点序号尾对计数函数），或各块态数之和与整条流的态数不等。
    count_disagreements: Vec<String>,
}

fn count_disagreements_of(
    label: &str,
    expansion: Expansion,
    counted: Option<u64>,
    plan: &PlannedBlocks,
) -> Vec<String> {
    let mut disagreements = Vec::new();
    if counted != Some(plan.state_count) {
        disagreements.push(format!(
            "{label} {}：本地按段数的态数 {counted:?}，计划的末节点序号尾 {}",
            expansion.name(),
            plan.state_count
        ));
    }
    if plan.state_count != plan.state_count_by_the_counting_function {
        disagreements.push(format!(
            "{label} {}：计划的末节点序号尾 {}，计数函数 {}",
            expansion.name(),
            plan.state_count,
            plan.state_count_by_the_counting_function
        ));
    }
    let block_total: u64 = plan.blocks.iter().map(|block| block.states).sum();
    if block_total != plan.state_count {
        disagreements.push(format!(
            "{label} {}：各块态数之和 {block_total}，整条流的态数 {}",
            expansion.name(),
            plan.state_count
        ));
    }
    disagreements
}

/// 读一条流：规模（写数、段数、最长段、三种态数）和两种展开的计划。`base` 是这条流的基镜像（直接录出来的池，或由转储重建的）。
///
/// # Errors
/// 计划排不了（写表里没有根槽写）。
///
/// # Panics
/// `plan_crash_points` 自己的断言（调用方包一层 `catch_unwind`）。
fn read_the_flow(dump: &FlowDump, base: &MemoryPool) -> Result<FlowReading, String> {
    read_the_flow_under(dump, base, MOST_BLOCKS_PLANNED_PER_FLOW)
}

/// [`read_the_flow`]，块数上界的上限由调用方给。态数照报；块数上界超过上限的展开不排计划，`blocks_*` 写 `not_planned`，
/// 与「装不进 u64」的流同一种待遇（对照里要计划的几条也跟着不跑）。
fn read_the_flow_under(
    dump: &FlowDump,
    base: &MemoryPool,
    most_blocks: u64,
) -> Result<FlowReading, String> {
    let full = state_count_of_the_flow(base, dump, Expansion::Full);
    let quick_tier = state_count_of_the_flow(base, dump, Expansion::QuickTier);
    let two_state = two_state_closed_form(&dump.segments);
    let started = Instant::now();
    let full_plan = state_count_to_plan(base, dump, Expansion::Full, most_blocks)
        .map(|_| plan_blocks_under(base, dump, Expansion::Full, default_item(), most_blocks))
        .transpose()?;
    let quick_plan = state_count_to_plan(base, dump, Expansion::QuickTier, most_blocks)
        .map(|_| {
            plan_blocks_under(
                base,
                dump,
                Expansion::QuickTier,
                default_item(),
                most_blocks,
            )
        })
        .transpose()?;
    let keys_seconds = started.elapsed().as_secs_f64();
    let mut count_disagreements = Vec::new();
    if let Some(plan) = &full_plan {
        count_disagreements.extend(count_disagreements_of(
            &dump.label,
            Expansion::Full,
            full,
            plan,
        ));
    }
    if let Some(plan) = &quick_plan {
        count_disagreements.extend(count_disagreements_of(
            &dump.label,
            Expansion::QuickTier,
            quick_tier,
            plan,
        ));
    }
    let row = FlowRow {
        family: dump.family.clone(),
        tier: dump.tier.clone(),
        label: dump.label.clone(),
        seed: dump.seed,
        ending: dump.ending.clone(),
        writes: dump.writes.len(),
        segments: dump.segments.len(),
        longest_segment: longest_segment_of(&dump.segments),
        two_state,
        full,
        quick_tier,
        blocks_full: full_plan.as_ref().map(|plan| plan.blocks.len()),
        blocks_quick: quick_plan.as_ref().map(|plan| plan.blocks.len()),
        points: full_plan
            .as_ref()
            .or(quick_plan.as_ref())
            .map(|plan| plan.point_count),
        keys_seconds,
        two_state_agrees_with_crates: two_state
            .map(|local| local == closed_form_state_count(&dump.segments)),
    };
    Ok(FlowReading {
        row,
        full: full_plan,
        quick: quick_plan,
        count_disagreements,
    })
}

// ───────────────────────────── 5.2 阳性对照（keys 子命令里对种子下标 0 的随机历史、覆盖写链、冒烟层各条流跑） ─────────────────────────────

/// 一条对照的结果。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ControlReading {
    kind: &'static str,
    flow: String,
    passed: bool,
    detail: String,
}

impl ControlReading {
    fn body(&self) -> String {
        format!(
            "name=control kind={} flow={} passed={} {}",
            self.kind, self.flow, self.passed, self.detail
        )
    }
}

fn block_keys_of(blocks: &[BlockOfAPlan]) -> BTreeSet<VerdictBlockKey> {
    blocks.iter().map(|block| block.key.clone()).collect()
}

fn reuse_keys_of(blocks: &[BlockOfAPlan]) -> BTreeSet<[u8; 32]> {
    blocks
        .iter()
        .map(|block| block.key.node_input_digest.0)
        .collect()
}

/// 候选的块里块键在 `against` 里的有几块、几个状态。
fn shared_blocks_and_states(
    candidate: &[BlockOfAPlan],
    against: &BTreeSet<VerdictBlockKey>,
) -> (usize, u64) {
    candidate
        .iter()
        .filter(|block| against.contains(&block.key))
        .fold((0usize, 0u64), |(blocks, states), block| {
            (blocks + 1, states + block.states)
        })
}

/// 写表里有没有一次写碰到 `device` 上的这个扇区。
fn some_write_touches_the_sector(writes: &[RetainedWrite], device: u32, sector: u64) -> bool {
    writes.iter().any(|write| {
        write.device.0 == device
            && write.offset.0 / SECTOR_BYTES <= sector
            && sector < (write.offset.0 + write.contents.length_in_bytes()).div_ceil(SECTOR_BYTES)
    })
}

/// 扇区宽，崩溃点重放取 512 字节（D13 已定项 4）。
const SECTOR_BYTES: u64 = write_dump::SECTOR_BYTES_OF_THE_BASE_IMAGE;

/// 5.2 的对照只在种子下标 0 的随机历史、覆盖写链的每一档、冒烟层的每一条上跑（对照那段循环遍历每一族每一档，不拿第一条跑一下就算）。
fn controls_apply_to(dump: &FlowDump) -> bool {
    dump.seed
        .is_none_or(|seed| seed == SEED_BASE_OF_THE_RANDOM_HISTORIES)
}

/// 基镜像里盘 0 上号最大的那个扇区被写成 512 个 0xFF 之后，与原流共用的块数必须恰好 0（计划的块数不变）。
fn base_sector_changed_control(
    dump: &FlowDump,
    base: &MemoryPool,
    full: &PlannedBlocks,
) -> Result<ControlReading, String> {
    let last_sector = base.device_size_in_bytes / SECTOR_BYTES - 1;
    let untouched = !base
        .devices
        .get(&DeviceIdentity(0))
        .is_some_and(|image| image.written_sectors().contains_key(&last_sector))
        && !some_write_touches_the_sector(&dump.writes, 0, last_sector);
    if !untouched {
        return Ok(ControlReading {
            kind: "base_sector_changed",
            flow: dump.label.clone(),
            passed: false,
            detail: format!(
                "盘 0 的扇区 {last_sector} 在基镜像或写表里被碰过，造不出改一扇区的对照"
            ),
        });
    }
    let mut changed = base.clone();
    changed.apply_writes(&[RetainedWrite {
        device: DeviceIdentity(0),
        kind: StepKind::UnitWrite,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(last_sector * SECTOR_BYTES),
        contents: WrittenContents::Bytes(vec![0xFF; 512]),
    }]);
    let changed_plan = plan_blocks(&changed, dump, Expansion::Full, default_item())?;
    let (shared_blocks, _shared_states) =
        shared_blocks_and_states(&changed_plan.blocks, &block_keys_of(&full.blocks));
    Ok(ControlReading {
        kind: "base_sector_changed",
        flow: dump.label.clone(),
        passed: shared_blocks == 0 && changed_plan.blocks.len() == full.blocks.len(),
        detail: format!(
            "changed_sector={last_sector} blocks={} blocks_after_the_change={} shared_blocks={shared_blocks} expected_shared_blocks=0",
            full.blocks.len(),
            changed_plan.blocks.len()
        ),
    })
}

/// 同一条流各按全量与甲二排计划：复用键两边同一批，块键共用的块恰好是不管任何段的点（末节点在内）的块。
fn two_expansions_control(
    label: &str,
    full: &PlannedBlocks,
    quick: &PlannedBlocks,
) -> ControlReading {
    let (shared_blocks, _shared_states) =
        shared_blocks_and_states(&quick.blocks, &block_keys_of(&full.blocks));
    let reuse_keys_equal = reuse_keys_of(&full.blocks) == reuse_keys_of(&quick.blocks);
    let expected = full.blocks_of_points_without_segments;
    ControlReading {
        kind: "two_expansions",
        flow: label.to_string(),
        passed: reuse_keys_equal
            && shared_blocks == expected
            && quick.blocks_of_points_without_segments == expected,
        detail: format!(
            "reuse_keys_equal={reuse_keys_equal} shared_blocks={shared_blocks} expected_shared_blocks={expected} blocks_full={} blocks_quick={}",
            full.blocks.len(),
            quick.blocks.len()
        ),
    }
}

/// 换项名与设点文件再排计划，块键逐个相同（D13 已定项 17「名字、路径不进键」）。
fn name_not_in_key_control(
    dump: &FlowDump,
    base: &MemoryPool,
    full: &PlannedBlocks,
) -> Result<ControlReading, String> {
    let renamed = plan_blocks(
        base,
        dump,
        Expansion::Full,
        CrashPointReplayItem {
            name: OTHER_ITEM_NAME_FOR_THE_NAME_CONTROL,
            setting_file: OTHER_SETTING_FILE_FOR_THE_NAME_CONTROL,
        },
    )?;
    Ok(ControlReading {
        kind: "name_not_in_key",
        flow: dump.label.clone(),
        passed: renamed.blocks == full.blocks,
        detail: format!(
            "blocks={} blocks_under_another_name={}",
            full.blocks.len(),
            renamed.blocks.len()
        ),
    })
}

/// 转储写成字节再读回，内容逐项相同（块键是转储内容的函数，读回相同则块键相同）。
fn dump_round_trip_control(dump: &FlowDump) -> ControlReading {
    let read_back = FlowDump::from_bytes(&dump.to_bytes());
    ControlReading {
        kind: "dump_round_trip",
        flow: dump.label.clone(),
        passed: read_back.as_ref().is_ok_and(|again| again == dump),
        detail: match read_back {
            Ok(_) => "read_back_equal_to_the_original".to_string(),
            Err(error) => format!("read_back_refused={error}"),
        },
    }
}

/// 一条流的全部对照。
fn controls_of_the_flow(
    dump: &FlowDump,
    base: &MemoryPool,
    reading: &FlowReading,
) -> Result<Vec<ControlReading>, String> {
    let mut controls = vec![dump_round_trip_control(dump)];
    if let Some(full) = &reading.full {
        controls.push(base_sector_changed_control(dump, base, full)?);
        controls.push(name_not_in_key_control(dump, base, full)?);
        if let Some(quick) = &reading.quick {
            controls.push(two_expansions_control(&dump.label, full, quick));
        }
    }
    Ok(controls)
}

// ───────────────────────────── 命令行与统计 ─────────────────────────────

/// 命令行上读出来的选项：带值的（可重复）和不带值的开关。
#[derive(Debug, Default)]
struct ParsedOptions {
    values: BTreeMap<String, Vec<String>>,
    flags: BTreeSet<String>,
}

impl ParsedOptions {
    /// 只许 `value_options` 与 `flag_options` 里的名字；带值的选项后面缺值、写了认不出的选项都交回说明。
    fn parse(
        arguments: &[String],
        value_options: &[&str],
        flag_options: &[&str],
    ) -> Result<Self, String> {
        let mut parsed = Self::default();
        let mut remaining = arguments.iter();
        while let Some(option) = remaining.next() {
            if flag_options.contains(&option.as_str()) {
                parsed.flags.insert(option.clone());
            } else if value_options.contains(&option.as_str()) {
                let value = remaining
                    .next()
                    .ok_or_else(|| format!("{option} 后面缺值"))?;
                parsed
                    .values
                    .entry(option.clone())
                    .or_default()
                    .push(value.clone());
            } else {
                return Err(format!("认不出的选项 {option}"));
            }
        }
        Ok(parsed)
    }

    fn all(&self, option: &str) -> &[String] {
        self.values.get(option).map_or(&[], Vec::as_slice)
    }

    /// 至少给一次。
    fn paths(&self, option: &str) -> Result<Vec<PathBuf>, String> {
        let given = self.all(option);
        if given.is_empty() {
            return Err(format!("缺 {option}"));
        }
        Ok(given.iter().map(PathBuf::from).collect())
    }

    /// 恰好给一次。
    fn single(&self, option: &str) -> Result<&str, String> {
        match self.all(option) {
            [only] => Ok(only.as_str()),
            [] => Err(format!("缺 {option}")),
            _ => Err(format!("{option} 只许给一次")),
        }
    }

    fn flag(&self, option: &str) -> bool {
        self.flags.contains(option)
    }
}

/// 一组数的最小、中位、最大；空组都是 `None`。偶数个时中位取中间两个的平均。
#[derive(Clone, Debug, PartialEq)]
struct Spread {
    minimum: f64,
    median: f64,
    maximum: f64,
}

impl Spread {
    fn of(values: &[f64]) -> Option<Self> {
        let mut sorted = values.to_vec();
        sorted.sort_by(f64::total_cmp);
        let middle = sorted.len() / 2;
        let median = match sorted.len() {
            0 => return None,
            count if count % 2 == 1 => sorted[middle],
            _ => f64::midpoint(sorted[middle - 1], sorted[middle]),
        };
        Some(Self {
            minimum: sorted[0],
            median,
            maximum: sorted[sorted.len() - 1],
        })
    }

    fn text(spread: Option<&Self>) -> String {
        spread.map_or_else(
            || "minimum=none median=none maximum=none".to_string(),
            |reading| {
                format!(
                    "minimum={} median={} maximum={}",
                    reading.minimum, reading.median, reading.maximum
                )
            },
        )
    }
}

/// 一行的写出口：行首 `E7RESULT `，数着打了几行（收尾行报它），打的同时记下来，`--out` 给了就写一份文件。
struct Output {
    emitter: Emitter,
    lines: Vec<String>,
}

impl Output {
    fn new() -> Self {
        Self {
            emitter: Emitter::new(),
            lines: Vec::new(),
        }
    }

    fn emit(&mut self, body: &str) {
        self.emitter.emit(body);
        self.lines.push(format!("E7RESULT {body}"));
    }

    fn emit_closing_line(&mut self, name: &str, fields: &str) {
        self.emitter.emit_closing_line(name, fields);
        let emitted = self.lines.len() + 1;
        self.lines
            .push(format!("E7RESULT name={name} {fields} emitted={emitted}"));
    }

    /// 打过的全部行写进 `path`（先写临时名再改名）。
    fn write_to(&self, path: &Path) -> Result<(), String> {
        let mut text = self.lines.join("\n");
        text.push('\n');
        write_dump::write_bytes_atomically(path, text.as_bytes())
    }
}

// ───────────────────────────── keys：逐份转储排计划、取块键 ─────────────────────────────

const CONTROLS_FILE_SUFFIX: &str = ".controls";
const CONTROLS_FILE_TRAILER_PREFIX: &str = "# controls_done control_lines=";

/// 一份转储这一趟做完的结局。
enum KeysOutcome {
    Done {
        row: Box<FlowRow>,
        reused: bool,
        /// （`name=control` 行的内容，过没过）。
        controls: Vec<(String, bool)>,
        count_disagreements: Vec<String>,
    },
    Failed {
        label: String,
        message: String,
    },
}

fn label_of_the_file(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
        .trim_end_matches(DUMP_FILE_SUFFIX)
        .to_string()
}

fn label_of_the_keys_file(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
        .trim_end_matches(FULL_KEYS_FILE_SUFFIX)
        .to_string()
}

fn controls_file_text(controls: &[ControlReading]) -> String {
    let mut text: String = controls
        .iter()
        .map(|control| format!("{}\n", control.body()))
        .collect();
    text.push_str(&format!(
        "{CONTROLS_FILE_TRAILER_PREFIX}{}\n",
        controls.len()
    ));
    text
}

/// 对照文件读回来的 `name=control` 行（每行一条）；缺末行（没写完）交 `None`。
fn control_lines_of(text: &str) -> Option<Vec<String>> {
    let (lines, trailer) = text
        .trim_end()
        .rsplit_once('\n')
        .map_or((None, text.trim_end()), |(head, last)| (Some(head), last));
    let declared = trailer
        .strip_prefix(CONTROLS_FILE_TRAILER_PREFIX)?
        .parse::<usize>()
        .ok()?;
    let kept: Vec<String> =
        lines.map_or_else(Vec::new, |head| head.lines().map(str::to_string).collect());
    (kept.len() == declared).then_some(kept)
}

/// 做一份转储：已经有写完的块键文件（与需要的对照）就整份跳过；否则读转储、排计划、写块键文件、跑对照。
fn keys_of_one_dump(path: &Path, out_directory: &Path, with_controls: bool) -> KeysOutcome {
    let label = label_of_the_file(path);
    let full_path = out_directory.join(format!("{label}{FULL_KEYS_FILE_SUFFIX}"));
    let quick_path = out_directory.join(format!("{label}{QUICK_KEYS_FILE_SUFFIX}"));
    let controls_path = out_directory.join(format!("{label}{CONTROLS_FILE_SUFFIX}"));
    let failed = |message: String| KeysOutcome::Failed {
        label: label.clone(),
        message,
    };
    let dump = match FlowDump::read_from(path) {
        Ok(dump) => dump,
        Err(message) => return failed(message),
    };
    let needs_controls = with_controls && controls_apply_to(&dump);
    if let Ok((_, existing_row)) = KeysFile::read_row_of_a_finished_file(&full_path) {
        let quick_done = existing_row.blocks_quick.is_none()
            || KeysFile::read_row_of_a_finished_file(&quick_path).is_ok();
        let control_lines = std::fs::read_to_string(&controls_path)
            .ok()
            .and_then(|text| control_lines_of(&text));
        if quick_done && (!needs_controls || control_lines.is_some()) {
            return KeysOutcome::Done {
                row: Box::new(existing_row),
                reused: true,
                controls: control_lines
                    .unwrap_or_default()
                    .into_iter()
                    .map(|line| {
                        let passed = line.contains(" passed=true ");
                        (line, passed)
                    })
                    .collect(),
                count_disagreements: Vec::new(),
            };
        }
    }
    let base = dump.base_pool();
    let outcome = catch_unwind(AssertUnwindSafe(
        || -> Result<(FlowReading, Vec<ControlReading>), String> {
            let reading = read_the_flow(&dump, &base)?;
            let controls = if needs_controls {
                controls_of_the_flow(&dump, &base, &reading)?
            } else {
                Vec::new()
            };
            Ok((reading, controls))
        },
    ));
    let (reading, controls) = match outcome {
        Err(payload) => return failed(format!("panic：{}", panic_text_of(payload.as_ref()))),
        Ok(Err(message)) => return failed(message),
        Ok(Ok(pair)) => pair,
    };
    let no_blocks: &[BlockOfAPlan] = &[];
    let full_blocks = reading.full.as_ref().map_or(no_blocks, |plan| &plan.blocks);
    if let Err(message) =
        write_keys_file_atomically(&full_path, &reading.row, Expansion::Full, full_blocks)
    {
        return failed(message);
    }
    if let Some(quick) = &reading.quick {
        if let Err(message) = write_keys_file_atomically(
            &quick_path,
            &reading.row,
            Expansion::QuickTier,
            &quick.blocks,
        ) {
            return failed(message);
        }
    }
    if needs_controls {
        if let Err(message) = write_dump::write_bytes_atomically(
            &controls_path,
            controls_file_text(&controls).as_bytes(),
        ) {
            return failed(message);
        }
    }
    KeysOutcome::Done {
        row: Box::new(reading.row),
        reused: false,
        controls: controls
            .iter()
            .map(|control| (control.body(), control.passed))
            .collect(),
        count_disagreements: reading.count_disagreements,
    }
}

fn panic_text_of(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "panic 的内容不是文本".to_string())
        .replace('\n', "\\n")
}

/// 每族每组一行 `name=family_summary`：各量的最小 / 中位 / 最大，与写数不少于 1000 的流数（登记第六节 J1-甲）。
fn family_summary_bodies(rows: &[FlowRow]) -> Vec<String> {
    let mut groups: BTreeMap<(String, String), Vec<&FlowRow>> = BTreeMap::new();
    for row in rows {
        groups
            .entry((row.family.clone(), row.group()))
            .or_default()
            .push(row);
    }
    groups
        .into_iter()
        .map(|((family, group), members)| {
            let spread_of = |value_of: &dyn Fn(&FlowRow) -> Option<f64>| {
                Spread::text(Spread::of(&members.iter().filter_map(|row| value_of(row)).collect::<Vec<f64>>()).as_ref())
            };
            let log2_of = |count: Option<u64>| count.map(|value| (value as f64).log2());
            format!(
                "name=family_summary family={family} group={group} flows={} flows_with_writes_at_least_1000={} flows_beyond_u64={} writes=[{}] segments=[{}] longest_segment=[{}] log2_two_state=[{}] log2_full=[{}] log2_quick_tier=[{}] blocks_full=[{}]",
                members.len(),
                members.iter().filter(|row| row.writes >= WRITES_OF_A_THOUSAND_WRITE_STREAM).count(),
                members.iter().filter(|row| row.full.is_none()).count(),
                spread_of(&|row| Some(row.writes as f64)),
                spread_of(&|row| Some(row.segments as f64)),
                spread_of(&|row| Some(row.longest_segment as f64)),
                spread_of(&|row| log2_of(row.two_state)),
                spread_of(&|row| log2_of(row.full)),
                spread_of(&|row| log2_of(row.quick_tier)),
                spread_of(&|row| row.blocks_full.map(|blocks| blocks as f64)),
            )
        })
        .collect()
}

/// `keys`：按流并行（线程数取转储模块里那一份读法），每份转储一个片，片与片之间不共享状态。
fn run_keys_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(arguments, &["--dump", "--out"], &["--controls"]) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let (dump_directories, out_directory) = match (parsed.paths("--dump"), parsed.single("--out")) {
        (Ok(dumps), Ok(out)) => (dumps, PathBuf::from(out)),
        (Err(message), _) | (_, Err(message)) => return usage_error(&message),
    };
    if let Err(error) = std::fs::create_dir_all(&out_directory) {
        return usage_error(&format!("建 {} 失败：{error}", out_directory.display()));
    }
    let dump_files = match files_with_suffix(&dump_directories, DUMP_FILE_SUFFIX, None) {
        Ok(files) => files,
        Err(message) => return usage_error(&message),
    };
    let with_controls = parsed.flag("--controls");
    let output = Mutex::new(Output::new());
    let outcomes: Mutex<BTreeMap<usize, KeysOutcome>> = Mutex::new(BTreeMap::new());
    let next_file = AtomicUsize::new(0);
    let started = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..write_dump::worker_threads_from_the_environment().min(dump_files.len().max(1)) {
            scope.spawn(|| loop {
                let file_index = next_file.fetch_add(1, Ordering::SeqCst);
                let Some(path) = dump_files.get(file_index) else {
                    break;
                };
                let outcome = keys_of_one_dump(path, &out_directory, with_controls);
                let mut finished = outcomes.lock().expect("结局表的锁没被毒化");
                finished.insert(file_index, outcome);
                let done = finished.len();
                drop(finished);
                output.lock().expect("打印器的锁没被毒化").emit(&format!(
                    "name=progress flow={} done={done}/{} seconds={:.1}",
                    label_of_the_file(path),
                    dump_files.len(),
                    started.elapsed().as_secs_f64()
                ));
            });
        }
    });
    let mut output = output.into_inner().expect("打印器的锁没被毒化");
    let (mut rows, mut failed_flows, mut failed_controls, mut disagreements_total) =
        (Vec::new(), 0usize, 0usize, 0usize);
    for outcome in outcomes
        .into_inner()
        .expect("结局表的锁没被毒化")
        .into_values()
    {
        match outcome {
            KeysOutcome::Failed { label, message } => {
                failed_flows += 1;
                output.emit(&format!("name=flow_failed flow={label} reason={message}"));
            }
            KeysOutcome::Done {
                row,
                reused,
                controls,
                count_disagreements,
            } => {
                output.emit(&format!("{} reused={reused}", row.body()));
                for (body, passed) in controls {
                    failed_controls += usize::from(!passed);
                    output.emit(&body);
                }
                for disagreement in &count_disagreements {
                    output.emit(&format!(
                        "name=stop_check item=two_counting_routes_agree result=fail detail={}",
                        disagreement.replace(' ', "_")
                    ));
                }
                disagreements_total += count_disagreements.len();
                rows.push(*row);
            }
        }
    }
    for body in family_summary_bodies(&rows) {
        output.emit(&body);
    }
    output.emit_closing_line(
        "keys_done",
        &format!("flows={} failed={failed_flows} controls_failed={failed_controls} count_disagreements={disagreements_total}", rows.len()),
    );
    if disagreements_total > 0 || failed_flows > 0 {
        EXIT_CODE_STOP_CLAUSE_FAILED
    } else if failed_controls > 0 {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

fn usage_error(message: &str) -> i32 {
    eprintln!("命令行不对：{message}");
    EXIT_CODE_USAGE
}

// ───────────────────────────── share：J1-乙（态数区间重叠）与 J1-丙（共用块键） ─────────────────────────────

/// J1-丙的判定（登记第六节）：换名 = renamed、独立 = independent、部分 = partial。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShareVerdict {
    Renamed,
    Independent,
    Partial,
}

impl ShareVerdict {
    fn name(self) -> &'static str {
        match self {
            ShareVerdict::Renamed => "renamed",
            ShareVerdict::Independent => "independent",
            ShareVerdict::Partial => "partial",
        }
    }
}

/// 族中位不小于门槛记换名；族最大不大于门槛记独立；其余记部分。
fn share_verdict(median: f64, maximum: f64) -> ShareVerdict {
    if median >= SHARE_MEDIAN_AT_LEAST_FOR_RENAMED {
        ShareVerdict::Renamed
    } else if maximum <= SHARE_MAXIMUM_AT_MOST_FOR_INDEPENDENT {
        ShareVerdict::Independent
    } else {
        ShareVerdict::Partial
    }
}

/// J1-乙的判定：重叠 = overlap、分开 = separate、部分 = partial。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverlapVerdict {
    Overlap,
    Separate,
    Partial,
}

impl OverlapVerdict {
    fn name(self) -> &'static str {
        match self {
            OverlapVerdict::Overlap => "overlap",
            OverlapVerdict::Separate => "separate",
            OverlapVerdict::Partial => "partial",
        }
    }
}

fn overlap_verdict(inside_fraction: f64) -> OverlapVerdict {
    if inside_fraction >= INSIDE_FRACTION_AT_LEAST_FOR_OVERLAP {
        OverlapVerdict::Overlap
    } else if inside_fraction <= INSIDE_FRACTION_AT_MOST_FOR_SEPARATE {
        OverlapVerdict::Separate
    } else {
        OverlapVerdict::Partial
    }
}

/// 候选流共用占比（按块）：候选的块里块键在比较集里的块数 ÷ 候选自己的块数。
fn block_share_fraction(candidate: &[BlockOfAPlan], against: &BTreeSet<VerdictBlockKey>) -> f64 {
    let (shared_blocks, _shared_states) = shared_blocks_and_states(candidate, against);
    shared_blocks as f64 / candidate.len() as f64
}

/// 候选流共用占比（按态数加权）：共用块里的态数 ÷ 候选的态数。
fn weighted_share_fraction(candidate: &[BlockOfAPlan], against: &BTreeSet<VerdictBlockKey>) -> f64 {
    let (_shared_blocks, shared_states) = shared_blocks_and_states(candidate, against);
    let states: u64 = candidate.iter().map(|block| block.states).sum();
    shared_states as f64 / states as f64
}

/// 复用键共用占比（块键共用的上界，不含计划哈希）：复用键在比较集里的块数 ÷ 候选的块数。
fn reuse_key_share_fraction(
    candidate: &[BlockOfAPlan],
    against_reuse_keys: &BTreeSet<[u8; 32]>,
) -> f64 {
    let shared = candidate
        .iter()
        .filter(|block| against_reuse_keys.contains(&block.key.node_input_digest.0))
        .count();
    shared as f64 / candidate.len() as f64
}

/// 冒烟层态数区间 S：最小与最大的两态闭式，和最长段的最小与最大（登记 1.1 节「冒烟层态数区间 S」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SmokeInterval {
    two_state_minimum: u64,
    two_state_maximum: u64,
    longest_segment_minimum: usize,
    longest_segment_maximum: usize,
}

/// 冒烟层里钉了常量的一条流：流名、两态闭式、最长段（执行员 grep 测试文件取常量、照闭式现算核过再写进 `--pinned-smoke` 文件）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct PinnedSmokeFlow {
    label: String,
    two_state: u64,
    longest_segment: usize,
}

/// `--pinned-smoke` 文件：一行一条流，`<流名> <两态闭式> <最长段>`，空行与 `#` 开头的行不算。
fn parse_pinned_smoke(text: &str) -> Result<Vec<PinnedSmokeFlow>, String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let words: Vec<&str> = line.split_whitespace().collect();
            let [label, two_state, longest_segment] = words.as_slice() else {
                return Err(format!(
                    "钉住的冒烟流要三段（流名 两态闭式 最长段），读到 {line:?}"
                ));
            };
            Ok(PinnedSmokeFlow {
                label: (*label).to_string(),
                two_state: two_state
                    .parse::<u64>()
                    .map_err(|error| format!("{line:?} 的两态闭式：{error}"))?,
                longest_segment: longest_segment
                    .parse::<usize>()
                    .map_err(|error| format!("{line:?} 的最长段：{error}"))?,
            })
        })
        .collect()
}

/// S：现录的冒烟层流（`baseline` 层）加钉住的那几条。
///
/// # Errors
/// 一条流都没有；现录的冒烟层流两态闭式算不出（有段不少于 64 写）。
fn smoke_interval(
    baseline_rows: &[FlowRow],
    pinned: &[PinnedSmokeFlow],
) -> Result<SmokeInterval, String> {
    let mut two_states = Vec::new();
    let mut longest_segments = Vec::new();
    for row in baseline_rows {
        two_states.push(
            row.two_state
                .ok_or_else(|| format!("冒烟层流 {} 的两态闭式算不出", row.label))?,
        );
        longest_segments.push(row.longest_segment);
    }
    for flow in pinned {
        two_states.push(flow.two_state);
        longest_segments.push(flow.longest_segment);
    }
    Ok(SmokeInterval {
        two_state_minimum: *two_states
            .iter()
            .min()
            .ok_or("一条冒烟层的流都没有，区间 S 定不出")?,
        two_state_maximum: *two_states
            .iter()
            .max()
            .ok_or("一条冒烟层的流都没有，区间 S 定不出")?,
        longest_segment_minimum: *longest_segments
            .iter()
            .min()
            .ok_or("一条冒烟层的流都没有，区间 S 定不出")?,
        longest_segment_maximum: *longest_segments
            .iter()
            .max()
            .ok_or("一条冒烟层的流都没有，区间 S 定不出")?,
    })
}

/// 一个值对区间 S 的位置。区间两端都算在里面（闭区间）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlaceAgainstInterval {
    Below,
    Inside,
    Above,
}

fn place_against(value: Option<u64>, minimum: u64, maximum: u64) -> PlaceAgainstInterval {
    match value {
        None => PlaceAgainstInterval::Above,
        Some(known) if known < minimum => PlaceAgainstInterval::Below,
        Some(known) if known > maximum => PlaceAgainstInterval::Above,
        Some(_) => PlaceAgainstInterval::Inside,
    }
}

/// 候选流的分组：每个（族，组）一个范围，另加两个主取样的合并范围（随机历史：比重 BROAD、盘宽 FourGibibytes 的各步数档；
/// 覆盖写链：每次覆盖写 4 个数据单元那一档）。敏感性取样点（盘宽、比重、步数、每次覆盖写的数据单元数）各自是单独的组，
/// 同一个门槛在两个取样点上各判一次（登记 8.2）。冒烟层的流（`baseline` 层）不在候选里。
fn scopes_of_the_candidates(rows: &[FlowRow]) -> Vec<(String, Vec<usize>)> {
    let mut scopes: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        if row.tier != "candidate" {
            continue;
        }
        let group = row.group();
        if group.starts_with("random_history-broad-FourGibibytes-steps") {
            scopes
                .entry("family:random_history_main".to_string())
                .or_default()
                .push(index);
        }
        if group == "overwrite_chain-04_units_per_overwrite" {
            scopes
                .entry("family:overwrite_chain_main".to_string())
                .or_default()
                .push(index);
        }
        scopes
            .entry(format!("group:{group}"))
            .or_default()
            .push(index);
    }
    scopes.into_iter().collect()
}

/// 一个范围里写数不少于 1000 的流对区间 S 的位置。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
struct OverlapCounts {
    flows: usize,
    flows_with_writes_at_least_1000: usize,
    inside: usize,
    above: usize,
    below: usize,
    longest_segment_inside: usize,
}

impl OverlapCounts {
    fn of(members: &[&FlowRow], interval: &SmokeInterval) -> Self {
        let mut counts = Self {
            flows: members.len(),
            ..Self::default()
        };
        for row in members
            .iter()
            .filter(|row| row.writes >= WRITES_OF_A_THOUSAND_WRITE_STREAM)
        {
            counts.flows_with_writes_at_least_1000 += 1;
            match place_against(
                row.two_state,
                interval.two_state_minimum,
                interval.two_state_maximum,
            ) {
                PlaceAgainstInterval::Inside => counts.inside += 1,
                PlaceAgainstInterval::Above => counts.above += 1,
                PlaceAgainstInterval::Below => counts.below += 1,
            }
            if (interval.longest_segment_minimum..=interval.longest_segment_maximum)
                .contains(&row.longest_segment)
            {
                counts.longest_segment_inside += 1;
            }
        }
        counts
    }

    /// 占写数不少于 1000 的流的比例；一条都没有交 `None`。
    fn fraction_of(self, count: usize) -> Option<f64> {
        (self.flows_with_writes_at_least_1000 > 0)
            .then(|| count as f64 / self.flows_with_writes_at_least_1000 as f64)
    }

    fn body(self, scope: &str, interval: &SmokeInterval) -> String {
        let fraction_text = |fraction: Option<f64>| {
            fraction.map_or_else(|| "none".to_string(), |value| format!("{value:.6}"))
        };
        let verdict = self
            .fraction_of(self.inside)
            .map_or("no_flow_with_writes_at_least_1000", |fraction| {
                overlap_verdict(fraction).name()
            });
        format!(
            "name=overlap scope={scope} smoke_interval_two_state_minimum={} smoke_interval_two_state_maximum={} smoke_interval_longest_segment_minimum={} smoke_interval_longest_segment_maximum={} flows={} flows_with_writes_at_least_1000={} inside={} above={} below={} inside_fraction={} above_fraction={} at_most_smoke_interval_maximum_fraction={} longest_segment_inside_fraction={} verdict={verdict}",
            interval.two_state_minimum,
            interval.two_state_maximum,
            interval.longest_segment_minimum,
            interval.longest_segment_maximum,
            self.flows,
            self.flows_with_writes_at_least_1000,
            self.inside,
            self.above,
            self.below,
            fraction_text(self.fraction_of(self.inside)),
            fraction_text(self.fraction_of(self.above)),
            fraction_text(self.fraction_of(self.inside + self.below)),
            fraction_text(self.fraction_of(self.longest_segment_inside)),
        )
    }
}

/// `share` 命令行读出来的输入：候选目录、比较集目录、输出文件、钉住的冒烟流。
type ShareInputs = (Vec<PathBuf>, Vec<PathBuf>, PathBuf, Vec<PinnedSmokeFlow>);

/// `share`：候选各流对比较集（冒烟层四条流的块键并集）的共用，加 J1-乙的区间重叠与自比对照。
fn run_share_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(
        arguments,
        &["--candidates", "--against", "--out", "--pinned-smoke"],
        &[],
    ) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let inputs = (|| -> Result<ShareInputs, String> {
        let pinned = match parsed.all("--pinned-smoke") {
            [] => Vec::new(),
            [file] => parse_pinned_smoke(
                &std::fs::read_to_string(file)
                    .map_err(|error| format!("读 {file} 失败：{error}"))?,
            )?,
            _ => return Err("--pinned-smoke 只许给一次".to_string()),
        };
        Ok((
            parsed.paths("--candidates")?,
            parsed.paths("--against")?,
            PathBuf::from(parsed.single("--out")?),
            pinned,
        ))
    })();
    let (candidate_directories, against_directories, out_file, pinned) = match inputs {
        Ok(inputs) => inputs,
        Err(message) => return usage_error(&message),
    };
    let (candidate_paths, against) = match (
        keys_file_paths(&candidate_directories, Expansion::Full),
        read_keys_files(&against_directories, Expansion::Full),
    ) {
        (Ok(candidate_paths), Ok(against)) => (candidate_paths, against),
        (Err(message), _) | (_, Err(message)) => {
            eprintln!("读块键文件失败：{message}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    let mut output = Output::new();
    let candidates = candidate_paths
        .iter()
        .map(|path| read_keys_file(path, Expansion::Full));
    let exit_code = match share_lines(candidates, &against, &pinned, &mut output) {
        Ok(exit_code) => exit_code,
        Err(message) => {
            eprintln!("读块键文件失败：{message}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    if let Err(message) = output.write_to(&out_file) {
        eprintln!("写 {} 失败：{message}", out_file.display());
        return EXIT_CODE_USAGE;
    }
    exit_code
}

fn fraction_text(fraction: f64) -> String {
    format!("{fraction:.6}")
}

/// 自比对照：一条流对「只含它自己」的比较集，共用占比恰好 1（共用块数等于它的块数）。
fn self_share_control_body(file: &KeysFile) -> (String, bool) {
    let own_keys = block_keys_of(&file.blocks);
    let (shared_blocks, _shared_states) = shared_blocks_and_states(&file.blocks, &own_keys);
    let passed = shared_blocks == file.blocks.len()
        && (block_share_fraction(&file.blocks, &own_keys) - 1.0).abs() < f64::EPSILON;
    (
        format!(
            "name=control kind=self_share flow={} passed={passed} blocks={} shared_blocks={shared_blocks} expected_shared_blocks={}",
            file.row.label,
            file.blocks.len(),
            file.blocks.len()
        ),
        passed,
    )
}

/// `share` 的全部输出行。候选一条一条读（最大的流有上千万块，不一起放进内存），只留每条流的行和三个占比。
/// 交回退出码：0；区间 S 定不出 3；自比对照没过 4。
///
/// # Errors
/// 某一份候选的块键文件读不回。
fn share_lines(
    candidates: impl Iterator<Item = Result<KeysFile, String>>,
    against: &[KeysFile],
    pinned: &[PinnedSmokeFlow],
    output: &mut Output,
) -> Result<i32, String> {
    // 比较集只收冒烟层（baseline 层）的流，候选只收 candidate 层的流：两边的目录里混了别层的流时，不静默当成别的层
    let against: Vec<KeysFile> = against
        .iter()
        .filter(|file| file.row.tier == "baseline")
        .cloned()
        .collect();
    let against = against.as_slice();
    let against_blocks: Vec<BlockOfAPlan> = against
        .iter()
        .flat_map(|file| file.blocks.iter().cloned())
        .collect();
    let against_keys = block_keys_of(&against_blocks);
    let against_reuse_keys = reuse_keys_of(&against_blocks);
    let mut controls_failed = 0usize;
    for file in against.iter().filter(|file| !file.blocks.is_empty()) {
        let (body, passed) = self_share_control_body(file);
        controls_failed += usize::from(!passed);
        output.emit(&body);
    }
    let mut rows: Vec<FlowRow> = Vec::new();
    let mut share_fractions: Vec<Option<(f64, f64, f64)>> = Vec::new();
    for candidate in candidates {
        let file = candidate?;
        if file.row.tier != "candidate" {
            output.emit(&format!(
                "name=share_ignored flow={} reason=tier_is_{}",
                file.row.label, file.row.tier
            ));
            continue;
        }
        rows.push(file.row.clone());
        if file.blocks.is_empty() {
            output.emit(&format!(
                "name=share_not_planned flow={} reason=no_blocks_planned",
                file.row.label
            ));
            share_fractions.push(None);
            continue;
        }
        let (body, passed) = self_share_control_body(&file);
        controls_failed += usize::from(!passed);
        output.emit(&body);
        let (shared_blocks, shared_states) = shared_blocks_and_states(&file.blocks, &against_keys);
        let block_fraction = block_share_fraction(&file.blocks, &against_keys);
        let weighted_fraction = weighted_share_fraction(&file.blocks, &against_keys);
        let reuse_fraction = reuse_key_share_fraction(&file.blocks, &against_reuse_keys);
        output.emit(&format!(
            "name=share flow={} family={} tier={} group={} blocks={} shared_blocks={shared_blocks} share_fraction={} states={} shared_states={shared_states} weighted_fraction={} reuse_fraction={}",
            file.row.label,
            file.row.family,
            file.row.tier,
            file.row.group(),
            file.blocks.len(),
            fraction_text(block_fraction),
            file.state_total(),
            fraction_text(weighted_fraction),
            fraction_text(reuse_fraction),
        ));
        share_fractions.push(Some((block_fraction, weighted_fraction, reuse_fraction)));
    }
    let baseline_rows: Vec<FlowRow> = against.iter().map(|file| file.row.clone()).collect();
    let interval = match smoke_interval(&baseline_rows, pinned) {
        Ok(interval) => interval,
        Err(message) => {
            output.emit(&format!(
                "name=stop_check item=S_interval result=fail detail={}",
                message.replace(' ', "_")
            ));
            output.emit_closing_line(
                "share_done",
                &format!(
                    "candidates={} against={} controls_failed={controls_failed}",
                    rows.len(),
                    against.len()
                ),
            );
            return Ok(EXIT_CODE_STOP_CLAUSE_FAILED);
        }
    };
    for (scope, members) in scopes_of_the_candidates(&rows) {
        let planned: Vec<(f64, f64, f64)> = members
            .iter()
            .filter_map(|index| share_fractions[*index])
            .collect();
        let spread_of = |pick: &dyn Fn(&(f64, f64, f64)) -> f64| {
            Spread::of(&planned.iter().map(pick).collect::<Vec<f64>>())
        };
        if let (Some(blocks), Some(weighted), Some(reuse)) = (
            spread_of(&|reading| reading.0),
            spread_of(&|reading| reading.1),
            spread_of(&|reading| reading.2),
        ) {
            output.emit(&format!(
                "name=share_group scope={scope} flows={} planned_flows={} median_share_fraction={} maximum_share_fraction={} median_weighted_fraction={} median_reuse_fraction={} maximum_reuse_fraction={} verdict={}",
                members.len(),
                planned.len(),
                fraction_text(blocks.median),
                fraction_text(blocks.maximum),
                fraction_text(weighted.median),
                fraction_text(reuse.median),
                fraction_text(reuse.maximum),
                share_verdict(blocks.median, blocks.maximum).name(),
            ));
        } else {
            output.emit(&format!(
                "name=share_group scope={scope} flows={} planned_flows=0 verdict=not_planned",
                members.len()
            ));
        }
        let member_rows: Vec<&FlowRow> = members.iter().map(|index| &rows[*index]).collect();
        output.emit(&OverlapCounts::of(&member_rows, &interval).body(&scope, &interval));
    }
    output.emit_closing_line(
        "share_done",
        &format!(
            "candidates={} against={} controls_failed={controls_failed}",
            rows.len(),
            against.len()
        ),
    );
    Ok(if controls_failed > 0 {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    })
}

// ───────────────────────────── library：长期库里已录的同键块（附带） ─────────────────────────────

/// 每条候选流的块键在长期库录入表里的个数（只读开库，不写）。
fn run_library_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(arguments, &["--keys", "--library"], &[]) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let (key_directories, library_argument) =
        match (parsed.paths("--keys"), parsed.single("--library")) {
            (Ok(directories), Ok(library)) => (directories, library.to_string()),
            (Err(message), _) | (_, Err(message)) => return usage_error(&message),
        };
    let library =
        match absolute_library_directory_of_the_variable_value("--library", &library_argument) {
            Ok(library) => library,
            Err(message) => return usage_error(&message),
        };
    let key_paths = match keys_file_paths(&key_directories, Expansion::Full) {
        Ok(key_paths) => key_paths,
        Err(message) => {
            eprintln!("读块键文件失败：{message}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    let log_directory = match DisposableLibraryDirectory::new("e184-library-share") {
        Ok(log_directory) => log_directory,
        Err(refusal) => {
            eprintln!("建不了放 RocksDB 运行日志的一次性目录：{refusal}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    let recorded =
        match VerdictStore::open_existing_for_reading_only(&library, log_directory.directory())
            .and_then(|store| store.recorded_blocks())
        {
            Ok(recorded) => recorded,
            Err(error) => {
                eprintln!("只读开库或读录入表失败：{error:?}");
                return EXIT_CODE_STOP_CLAUSE_FAILED;
            }
        };
    let recorded_keys: BTreeSet<VerdictBlockKey> =
        recorded.iter().map(|(key, _states)| key.clone()).collect();
    let mut output = Output::new();
    for path in &key_paths {
        let file = match read_keys_file(path, Expansion::Full) {
            Ok(file) => file,
            Err(message) => {
                eprintln!("读块键文件失败：{message}");
                return EXIT_CODE_STOP_CLAUSE_FAILED;
            }
        };
        let in_library = file
            .blocks
            .iter()
            .filter(|block| recorded_keys.contains(&block.key))
            .count();
        output.emit(&format!(
            "name=library_share flow={} family={} blocks={} in_library={in_library} fraction={}",
            file.row.label,
            file.row.family,
            file.blocks.len(),
            if file.blocks.is_empty() {
                "none".to_string()
            } else {
                fraction_text(in_library as f64 / file.blocks.len() as f64)
            },
        ));
    }
    output.emit_closing_line(
        "library_done",
        &format!(
            "flows={} recorded_blocks={}",
            key_paths.len(),
            recorded_keys.len()
        ),
    );
    EXIT_CODE_OK
}

// ───────────────────────────── compare：J3，前后两份写流的第一处不同与作废块数 ─────────────────────────────

/// 两份写流的第一处不同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FirstDifference {
    Identical,
    /// 基镜像就不同（mkfs 写的东西变了）。
    Base,
    /// 写表里第一条不同的写的下标（一边比另一边长时，是较短那边的写数）。
    Write(usize),
}

impl FirstDifference {
    fn text(self) -> String {
        match self {
            FirstDifference::Identical => "none".to_string(),
            FirstDifference::Base => "base".to_string(),
            FirstDifference::Write(index) => format!("write:{index}"),
        }
    }
}

fn first_difference(before: &FlowDump, after: &FlowDump) -> FirstDifference {
    if before.base_sectors_digest() != after.base_sectors_digest() {
        return FirstDifference::Base;
    }
    let shared = before.writes.len().min(after.writes.len());
    match (0..shared).find(|index| before.writes[*index] != after.writes[*index]) {
        Some(index) => FirstDifference::Write(index),
        None if before.writes.len() == after.writes.len() => FirstDifference::Identical,
        None => FirstDifference::Write(shared),
    }
}

/// 写下标落在第几段（前一份流的段）；不在任何段里交 `None`。
fn segment_of_write(segments: &[Vec<usize>], write_index: usize) -> Option<usize> {
    segments
        .iter()
        .position(|segment| segment.contains(&write_index))
}

/// 写下标落在崩溃枚举用例那一路切出来的第几个点里；不在任何点里交 `None`。
fn point_of_write(dump: &FlowDump, write_index: usize) -> Option<usize> {
    crash_points_of_the_steps(default_item(), &dump.writes, &dump.segments)
        .iter()
        .position(|span| span.writes.contains(&write_index))
}

/// 作废的块：后一份流里块键不在前一份流块键集合里的块（下一趟要现判的）。
fn invalidated_blocks<'blocks>(
    before: &[BlockOfAPlan],
    after: &'blocks [BlockOfAPlan],
) -> Vec<&'blocks BlockOfAPlan> {
    let before_keys = block_keys_of(before);
    after
        .iter()
        .filter(|block| !before_keys.contains(&block.key))
        .collect()
}

/// 一条流前后对比的结果。
#[derive(Clone, Debug)]
struct ComparedFlow {
    label: String,
    group: String,
    first_difference: FirstDifference,
    segment: Option<usize>,
    point: Option<usize>,
    first_journal_record_write: Option<usize>,
    blocks_before: usize,
    blocks_after: usize,
    invalidated_blocks: usize,
    states_after: u64,
    invalidated_states: u64,
}

impl ComparedFlow {
    fn block_fraction(&self) -> f64 {
        self.invalidated_blocks as f64 / self.blocks_after as f64
    }

    fn state_fraction(&self) -> f64 {
        self.invalidated_states as f64 / self.states_after as f64
    }

    fn body(&self) -> String {
        let optional = |value: Option<usize>| {
            value.map_or_else(|| "none".to_string(), |number| number.to_string())
        };
        format!(
            "name=compare flow={} group={} first_difference={} segment={} point={} first_journal_record_write={} blocks_before={} blocks_after={} invalidated_blocks={} states_after={} invalidated_states={} invalidated_block_fraction={} invalidated_state_fraction={}",
            self.label,
            self.group,
            self.first_difference.text(),
            if self.first_difference == FirstDifference::Base { "base".to_string() } else { optional(self.segment) },
            if self.first_difference == FirstDifference::Base { "base".to_string() } else { optional(self.point) },
            optional(self.first_journal_record_write),
            self.blocks_before,
            self.blocks_after,
            self.invalidated_blocks,
            self.states_after,
            self.invalidated_states,
            fraction_text(self.block_fraction()),
            fraction_text(self.state_fraction()),
        )
    }
}

fn compare_one_flow(
    before_dump: &FlowDump,
    after_dump: &FlowDump,
    before_keys: &KeysFile,
    after_keys: &KeysFile,
) -> ComparedFlow {
    let first = first_difference(before_dump, after_dump);
    let invalidated = invalidated_blocks(&before_keys.blocks, &after_keys.blocks);
    ComparedFlow {
        label: after_keys.row.label.clone(),
        group: after_keys.row.group(),
        first_difference: first,
        segment: match first {
            FirstDifference::Write(index) => segment_of_write(&before_dump.segments, index),
            FirstDifference::Identical | FirstDifference::Base => None,
        },
        point: match first {
            FirstDifference::Write(index) => point_of_write(before_dump, index),
            FirstDifference::Identical | FirstDifference::Base => None,
        },
        first_journal_record_write: before_dump
            .writes
            .iter()
            .position(|write| write.kind == StepKind::JournalRecord),
        blocks_before: before_keys.blocks.len(),
        blocks_after: after_keys.blocks.len(),
        invalidated_blocks: invalidated.len(),
        states_after: after_keys.state_total(),
        invalidated_states: invalidated.iter().map(|block| block.states).sum(),
    }
}

/// 同一组的几条流汇总成一行：作废块占后一份流块数的比例（几条流的块合起来算），按态数也算一遍，另报逐条流占比的中位与最大。
fn compare_summary_body(
    kind: &str,
    pair: &str,
    group: &str,
    flows: &[&ComparedFlow],
    unavailable: usize,
    key_unit_changed: &str,
) -> String {
    let blocks_after: usize = flows.iter().map(|flow| flow.blocks_after).sum();
    let invalidated: usize = flows.iter().map(|flow| flow.invalidated_blocks).sum();
    let states_after: u64 = flows.iter().map(|flow| flow.states_after).sum();
    let invalidated_states: u64 = flows.iter().map(|flow| flow.invalidated_states).sum();
    let per_flow = Spread::of(
        &flows
            .iter()
            .map(|flow| flow.block_fraction())
            .collect::<Vec<f64>>(),
    );
    format!(
        "name=compare_summary kind={kind} pair={pair} group={group} flows_compared={} flows_unavailable={unavailable} flows_with_a_difference={} blocks_after={blocks_after} invalidated_blocks={invalidated} invalidated_block_fraction={} states_after={states_after} invalidated_states={invalidated_states} invalidated_state_fraction={} per_flow_median_block_fraction={} per_flow_maximum_block_fraction={} judging_unit_key_changed={key_unit_changed}",
        flows.len(),
        flows.iter().filter(|flow| flow.first_difference != FirstDifference::Identical).count(),
        fraction_text(invalidated as f64 / blocks_after as f64),
        fraction_text(invalidated_states as f64 / states_after as f64),
        per_flow.as_ref().map_or_else(|| "none".to_string(), |spread| fraction_text(spread.median)),
        per_flow.as_ref().map_or_else(|| "none".to_string(), |spread| fraction_text(spread.maximum)),
    )
}

/// 补丁位该造出什么样的第一处不同（阳性对照 A9、A10、A11）：`journal-record` = 基镜像就不同，或第一处不同的写就是写表里第一条 journal 记录写；
/// `none` = 两份写流逐字相同、一块都不作废。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedFirstDifference {
    JournalRecord,
    Identical,
}

impl ExpectedFirstDifference {
    fn parse(text: &str) -> Result<Self, String> {
        match text {
            "journal-record" => Ok(Self::JournalRecord),
            "none" => Ok(Self::Identical),
            other => Err(format!(
                "--expect-first-difference 要写 journal-record 或 none，读到 {other:?}"
            )),
        }
    }

    fn holds_for(self, flow: &ComparedFlow) -> bool {
        match self {
            ExpectedFirstDifference::Identical => {
                flow.first_difference == FirstDifference::Identical && flow.invalidated_blocks == 0
            }
            ExpectedFirstDifference::JournalRecord => match flow.first_difference {
                FirstDifference::Base => true,
                FirstDifference::Write(index) => Some(index) == flow.first_journal_record_write,
                FirstDifference::Identical => false,
            },
        }
    }
}

/// 目录里这条流的块键文件与转储；两样有一样不在交 `None`（这条流这一侧取不出），在而读不回交回说明（不当没有）。
fn flow_files_of(directory: &Path, label: &str) -> Result<Option<(KeysFile, FlowDump)>, String> {
    let keys_path = directory.join(format!("{label}{FULL_KEYS_FILE_SUFFIX}"));
    let dump_path = directory.join(format!("{label}{DUMP_FILE_SUFFIX}"));
    if !keys_path.exists() || !dump_path.exists() {
        return Ok(None);
    }
    Ok(Some((
        read_keys_file(&keys_path, Expansion::Full)?,
        FlowDump::read_from(&dump_path)?,
    )))
}

fn run_compare_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(
        arguments,
        &[
            "--before",
            "--after",
            "--kind",
            "--pair",
            "--out",
            "--judging-unit-key-changed",
            "--expect-first-difference",
        ],
        &[],
    ) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let optional_single = |option: &str| parsed.all(option).first().cloned();
    let inputs = (|| -> Result<_, String> {
        let kind = parsed.single("--kind")?;
        if kind != "commit-pair" && kind != "patch" {
            return Err(format!("--kind 要写 commit-pair 或 patch，读到 {kind:?}"));
        }
        let expectation = optional_single("--expect-first-difference")
            .map(|text| ExpectedFirstDifference::parse(&text))
            .transpose()?;
        Ok((
            PathBuf::from(parsed.single("--before")?),
            PathBuf::from(parsed.single("--after")?),
            kind.to_string(),
            parsed.single("--pair")?.to_string(),
            PathBuf::from(parsed.single("--out")?),
            optional_single("--judging-unit-key-changed")
                .unwrap_or_else(|| "unavailable".to_string()),
            expectation,
        ))
    })();
    let (before_directory, after_directory, kind, pair, out_file, key_unit_changed, expectation) =
        match inputs {
            Ok(inputs) => inputs,
            Err(message) => return usage_error(&message),
        };
    let after_paths = match keys_file_paths(std::slice::from_ref(&after_directory), Expansion::Full)
    {
        Ok(after_paths) => after_paths,
        Err(message) => {
            eprintln!("读前后两份流失败：{message}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    let mut output = Output::new();
    let (mut compared, mut unavailable, mut controls_failed) = (Vec::new(), 0usize, 0usize);
    for after_path in &after_paths {
        let label = label_of_the_keys_file(after_path);
        let pair_of_sides = flow_files_of(&before_directory, &label)
            .and_then(|before| Ok((before, flow_files_of(&after_directory, &label)?)));
        let (Some((before, before_dump)), Some((after, after_dump))) = (match pair_of_sides {
            Ok(sides) => sides,
            Err(message) => {
                eprintln!("读前后两份流失败：{message}");
                return EXIT_CODE_STOP_CLAUSE_FAILED;
            }
        }) else {
            unavailable += 1;
            output.emit(&format!(
                "name=compare_unavailable flow={label} reason=missing_on_one_side"
            ));
            continue;
        };
        if before.blocks.is_empty() || after.blocks.is_empty() {
            unavailable += 1;
            output.emit(&format!(
                "name=compare_unavailable flow={label} reason=no_blocks_planned"
            ));
            continue;
        }
        let flow = compare_one_flow(&before_dump, &after_dump, &before, &after);
        output.emit(&flow.body());
        if let Some(expected) = expectation {
            let passed = expected.holds_for(&flow);
            controls_failed += usize::from(!passed);
            output.emit(&format!("name=control kind=expected_first_difference flow={label} passed={passed} expected={expected:?} first_difference={}", flow.first_difference.text()));
        }
        compared.push(flow);
    }
    let mut groups: BTreeMap<String, Vec<&ComparedFlow>> = BTreeMap::new();
    for flow in &compared {
        groups.entry(flow.group.clone()).or_default().push(flow);
    }
    for (group, flows) in &groups {
        output.emit(&compare_summary_body(
            &kind,
            &pair,
            group,
            flows,
            unavailable,
            &key_unit_changed,
        ));
    }
    output.emit_closing_line("compare_done", &format!("kind={kind} pair={pair} flows_compared={} flows_unavailable={unavailable} controls_failed={controls_failed}", compared.len()));
    if let Err(message) = output.write_to(&out_file) {
        eprintln!("写 {} 失败：{message}", out_file.display());
        return EXIT_CODE_USAGE;
    }
    if controls_failed > 0 {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

// ───────────────────────────── compose：J2-乙，提交那一趟的挂钟落在哪一档（纯算术） ─────────────────────────────

/// 挂钟档：≤ 2 小时 / 2–24 小时 / > 24 小时（界的出处在常量上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WallClockBand {
    WithinTwoHours,
    BetweenTwoHoursAndADay,
    OverADay,
}

impl WallClockBand {
    fn name(self) -> &'static str {
        match self {
            WallClockBand::WithinTwoHours => "within_2h",
            WallClockBand::BetweenTwoHoursAndADay => "between_2h_and_24h",
            WallClockBand::OverADay => "over_24h",
        }
    }
}

fn band_of_seconds(seconds: f64) -> WallClockBand {
    if seconds <= SECONDS_OF_TWO_HOURS {
        WallClockBand::WithinTwoHours
    } else if seconds <= SECONDS_OF_A_DAY {
        WallClockBand::BetweenTwoHoursAndADay
    } else {
        WallClockBand::OverADay
    }
}

/// 一行的字段（行首可以有 `E7RESULT `）。
fn fields_of_the_line(line: &str) -> BTreeMap<String, String> {
    fields_of(line.strip_prefix("E7RESULT ").unwrap_or(line))
}

/// 文本里 `name=<name>` 的每一行的字段。
fn lines_named(text: &str, name: &str) -> Vec<BTreeMap<String, String>> {
    text.lines()
        .map(fields_of_the_line)
        .filter(|fields| fields.get("name").is_some_and(|found| found == name))
        .collect()
}

fn float_field(fields: &BTreeMap<String, String>, name: &str) -> Result<f64, String> {
    required_field(fields, name)?
        .parse::<f64>()
        .map_err(|error| format!("{name}：{error}"))
}

fn u64_field(fields: &BTreeMap<String, String>, name: &str) -> Result<u64, String> {
    required_field(fields, name)?
        .parse::<u64>()
        .map_err(|error| format!("{name}：{error}"))
}

/// 崩溃放量一项冷跑的日志读出来的数（5.4：`CRASH_AMPLIFICATION_ITEM` 行与 `CRASH_AMPLIFICATION_GPU_TOTAL` 行）。
#[derive(Clone, Debug, PartialEq)]
struct ColdRun {
    item: String,
    exit_code: u64,
    /// 现判的状态数（`states=`）。
    states: u64,
    reused: u64,
    /// 这一项从起到收的秒数（入口自己计的）。
    seconds: f64,
    /// 判段：几台并行时各取最大（它们同时在判）。
    compile_seconds: f64,
    judge_seconds: f64,
    seconds_storing: f64,
    gpu_total_lines: usize,
}

impl ColdRun {
    /// 读一项的日志。ITEM 行恰好一行；GPU_TOTAL 行至少一行。
    ///
    /// # Errors
    /// ITEM 行不是恰好一行，或 GPU_TOTAL 一行都没有，或字段读不出：读不到不等于读到 0。
    fn from_log(text: &str) -> Result<Self, String> {
        let item_lines: Vec<BTreeMap<String, String>> = text
            .lines()
            .filter(|line| line.starts_with("CRASH_AMPLIFICATION_ITEM "))
            .map(fields_of)
            .collect();
        let [item_line] = item_lines.as_slice() else {
            return Err(format!(
                "CRASH_AMPLIFICATION_ITEM 行要恰好一行，读到 {} 行",
                item_lines.len()
            ));
        };
        let gpu_lines: Vec<BTreeMap<String, String>> = text
            .lines()
            .filter(|line| line.starts_with("CRASH_AMPLIFICATION_GPU_TOTAL "))
            .map(fields_of)
            .collect();
        if gpu_lines.is_empty() {
            return Err("一行 CRASH_AMPLIFICATION_GPU_TOTAL 都没有：判段用时读不到".to_string());
        }
        let largest = |name: &str| -> Result<f64, String> {
            gpu_lines
                .iter()
                .map(|fields| float_field(fields, name))
                .try_fold(0.0f64, |largest, value| {
                    value.map(|seconds| largest.max(seconds))
                })
        };
        Ok(Self {
            item: required_field(item_line, "item")?.to_string(),
            exit_code: u64_field(item_line, "exit")?,
            states: u64_field(item_line, "states")?,
            reused: u64_field(item_line, "reused")?,
            seconds: float_field(item_line, "seconds")?,
            compile_seconds: largest("compile_seconds")?,
            judge_seconds: largest("judge_seconds")?,
            seconds_storing: largest("seconds_storing")?,
            gpu_total_lines: gpu_lines.len(),
        })
    }

    /// 录入段用时（推的）：项的 `seconds` 减去编译、判段、存结论三段。
    fn recording_seconds(&self) -> f64 {
        self.seconds - self.compile_seconds - self.judge_seconds - self.seconds_storing
    }

    /// 包含自检：外层 `seconds` 包得住里层三段，包不住的这一项不进统计。
    fn outer_seconds_cover_the_inner_ones(&self) -> bool {
        self.recording_seconds() >= 0.0
    }

    /// 总吞吐：现判态数 ÷ 这一项的挂钟（录入加判）。
    fn rate_total(&self) -> f64 {
        self.states as f64 / self.seconds
    }

    /// 判段吞吐：现判态数 ÷ 判段用时（被测进程里计的 `judge_seconds`）。
    fn rate_judge(&self) -> f64 {
        self.states as f64 / self.judge_seconds
    }
}

/// 两点拟合：秒 = 固定 + 态数 ÷ 斜率。两点的秒数不同才拟合得出。
fn fit_through_two_points(
    small_states: u64,
    small_seconds: f64,
    medium_states: u64,
    medium_seconds: f64,
) -> Option<(f64, f64)> {
    let seconds_apart = medium_seconds - small_seconds;
    if seconds_apart <= 0.0 || medium_states <= small_states {
        return None;
    }
    let states_per_second = (medium_states - small_states) as f64 / seconds_apart;
    Some((
        small_seconds - small_states as f64 / states_per_second,
        states_per_second,
    ))
}

/// `du -system_configuration` 的输出里路径以 `suffix` 结尾的那一行的字节数。
fn library_bytes_of(du_text: &str, suffix: &str) -> Result<u64, String> {
    du_text
        .lines()
        .filter_map(|line| line.split_once(char::is_whitespace))
        .find(|(_bytes, path)| path.trim().ends_with(suffix))
        .ok_or_else(|| format!("du 的输出里没有以 {suffix} 结尾的路径"))
        .and_then(|(bytes, _path)| {
            bytes
                .parse::<u64>()
                .map_err(|error| format!("{suffix} 的字节数：{error}"))
        })
}

/// `crash_library_reader keys` 的输出里库目录以 `suffix` 结尾的那一行的 `recorded_blocks=`。
fn recorded_blocks_of(reader_text: &str, suffix: &str) -> Result<u64, String> {
    reader_text
        .lines()
        .filter(|line| line.starts_with("CRASH_LIBRARY_KEYS\t"))
        .map(fields_of)
        .find(|fields| {
            fields
                .get("library")
                .is_some_and(|library| library.ends_with(suffix))
        })
        .ok_or_else(|| {
            format!("读库程序的输出里没有库目录以 {suffix} 结尾的 CRASH_LIBRARY_KEYS 行")
        })
        .and_then(|fields| u64_field(&fields, "recorded_blocks"))
}

/// J3-乙的判定（登记第六节）：还差 = still_missing，写路径提交大多作废过半 = more_than_half_invalidated，块级复用省下大半 = mostly_reused，
/// 其间 = partial。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommitPairsVerdict {
    StillMissing,
    MoreThanHalfInvalidated,
    MostlyReused,
    Partial,
}

impl CommitPairsVerdict {
    fn name(self) -> &'static str {
        match self {
            CommitPairsVerdict::StillMissing => "still_missing",
            CommitPairsVerdict::MoreThanHalfInvalidated => "more_than_half_invalidated",
            CommitPairsVerdict::MostlyReused => "mostly_reused",
            CommitPairsVerdict::Partial => "partial",
        }
    }
}

fn commit_pairs_verdict(fractions: &[f64]) -> CommitPairsVerdict {
    let Some(spread) =
        Spread::of(fractions).filter(|_| fractions.len() >= COMMIT_PAIRS_NEEDED_FOR_ENOUGH)
    else {
        return CommitPairsVerdict::StillMissing;
    };
    if spread.median >= INVALIDATED_MEDIAN_AT_LEAST_FOR_MORE_THAN_HALF {
        CommitPairsVerdict::MoreThanHalfInvalidated
    } else if spread.median <= INVALIDATED_MEDIAN_AT_MOST_FOR_MOST_REUSED {
        CommitPairsVerdict::MostlyReused
    } else {
        CommitPairsVerdict::Partial
    }
}

/// 提交对的作废占比轨迹（8.1：被条款当谓词输入的量报轨迹，不只报中位）：算得出的对数、为正的对数、峰值、中位、期末值。
#[derive(Clone, Debug, PartialEq)]
struct CommitPairTrajectory {
    computable: usize,
    positive: usize,
    peak: f64,
    median: f64,
    final_value: Option<f64>,
}

/// 作废占比大于 0 的提交对数（占比为 0 的那一对一块都没作废，不算为正）。
fn pairs_with_a_positive_fraction(fractions: &[f64]) -> usize {
    fractions.iter().filter(|fraction| **fraction > 0.0).count()
}

/// `fractions_in_order` 按提交的时间次序；`final_index` 是期末那一对在其中的下标（主工作区对最近一次提交那一对）。
fn commit_pair_trajectory(
    fractions_in_order: &[f64],
    final_index: Option<usize>,
) -> Option<CommitPairTrajectory> {
    let spread = Spread::of(fractions_in_order)?;
    Some(CommitPairTrajectory {
        computable: fractions_in_order.len(),
        positive: pairs_with_a_positive_fraction(fractions_in_order),
        peak: spread.maximum,
        median: spread.median,
        final_value: final_index.and_then(|index| fractions_in_order.get(index).copied()),
    })
}

/// 一对提交在 `compare` 的汇总行里读出来的作废占比（同一对的几组合起来按块数、态数合算）。
#[derive(Clone, Debug, PartialEq)]
struct CommitPairReading {
    pair: String,
    block_fraction: f64,
    state_fraction: f64,
    judging_unit_key_changed: String,
}

/// `compare` 输出里 `kind=commit-pair` 的汇总行，按 `pair` 合算，按首次出现的次序。
///
/// # Errors
/// 汇总行的字段读不出。
fn commit_pairs_of(compare_text: &str) -> Result<Vec<CommitPairReading>, String> {
    let mut order: Vec<String> = Vec::new();
    let mut pooled: BTreeMap<String, (u64, u64, u64, u64, String)> = BTreeMap::new();
    for fields in lines_named(compare_text, "compare_summary") {
        if required_field(&fields, "kind")? != "commit-pair" {
            continue;
        }
        let pair = required_field(&fields, "pair")?.to_string();
        let entry = pooled.entry(pair.clone()).or_insert_with(|| {
            order.push(pair.clone());
            (0, 0, 0, 0, String::new())
        });
        entry.0 += u64_field(&fields, "blocks_after")?;
        entry.1 += u64_field(&fields, "invalidated_blocks")?;
        entry.2 += u64_field(&fields, "states_after")?;
        entry.3 += u64_field(&fields, "invalidated_states")?;
        entry.4 = required_field(&fields, "judging_unit_key_changed")?.to_string();
    }
    Ok(order
        .into_iter()
        .map(|pair| {
            let (blocks, invalidated_blocks, states, invalidated_states, key_changed) =
                &pooled[&pair];
            CommitPairReading {
                pair,
                block_fraction: *invalidated_blocks as f64 / *blocks as f64,
                state_fraction: *invalidated_states as f64 / *states as f64,
                judging_unit_key_changed: key_changed.clone(),
            }
        })
        .collect())
}

/// 提交那一趟要现判的秒数：查库的 `keys_seconds` 之和，加上作废的态数 ÷ 判段吞吐。
fn commit_cell_seconds(keys_seconds: f64, states_to_judge: f64, states_per_second: f64) -> f64 {
    keys_seconds + states_to_judge / states_per_second
}

/// 预算 `budget_seconds` 下按点的态数升序开：开得下几个点、几个状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OpenUnderBudget {
    points: usize,
    states: u64,
}

fn open_under_budget(
    point_states: &[u64],
    budget_seconds: f64,
    states_per_second: f64,
) -> OpenUnderBudget {
    let mut sorted = point_states.to_vec();
    sorted.sort_unstable();
    let mut open = OpenUnderBudget {
        points: 0,
        states: 0,
    };
    for states in sorted {
        let after = open.states.saturating_add(states);
        if after as f64 / states_per_second > budget_seconds {
            break;
        }
        open = OpenUnderBudget {
            points: open.points + 1,
            states: after,
        };
    }
    open
}

/// 按点的态数升序，每 10% 点数一个刻度的累计秒数（8.1 的累计曲线）。
fn cumulative_curve(point_states: &[u64], states_per_second: f64) -> Vec<(f64, f64)> {
    let mut sorted = point_states.to_vec();
    sorted.sort_unstable();
    (1..=10u64)
        .map(|tenth| {
            let points = usize::try_from((sorted.len() as u64 * tenth).div_ceil(10))
                .expect("点数装得进 usize");
            let states: u64 = sorted.iter().take(points).sum();
            (tenth as f64 / 10.0, states as f64 / states_per_second)
        })
        .collect()
}

/// 常规层名义负载 N（登记 5.4 前的 `compose` 定义）：随机历史 BROAD、FourGibibytes、100 步档的 20 段，加覆盖写链三档。
fn is_in_the_nominal_regular_layer_load(row: &FlowRow) -> bool {
    label_is_in_the_nominal_regular_layer_load(&row.label)
}

fn label_is_in_the_nominal_regular_layer_load(label: &str) -> bool {
    label.starts_with("random_history-broad-FourGibibytes-steps0100-")
        || label.starts_with("overwrite_chain-")
}

/// 合算要的 N 一侧的量。
#[derive(Clone, Debug, PartialEq)]
struct ComposeInputs {
    keys_seconds_total: f64,
    full_total: u64,
    quick_total: u64,
    /// N 里每一条流的每个崩溃点的全量态数（全部流的点放在一起）。
    point_states: Vec<u64>,
    write_path_median_fraction: Option<f64>,
    write_path_upper_bound_fraction: Option<f64>,
}

/// 一次冷跑量出来的两种吞吐（态每秒）。
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rates {
    total: f64,
    judge: f64,
}

/// 每个 G2 候选 × 提交类别的格子。丙只查库，另列一格「主 agent 起的全量」；丁在预算下开得下多少，另行列出。
fn cell_bodies(inputs: &ComposeInputs, rate_source: &str, rates: &Rates) -> Vec<String> {
    let classes = [
        ("write_path_median", inputs.write_path_median_fraction),
        (
            "write_path_upper_bound",
            inputs.write_path_upper_bound_fraction,
        ),
        ("judge_only", Some(1.0)),
    ];
    let full_states = inputs.full_total as f64;
    let quick_states = inputs.quick_total as f64;
    let mut bodies = Vec::new();
    for (class, fraction) in classes {
        let Some(fraction) = fraction else {
            bodies.push(format!("name=compose_cell commit_class={class} rate_source={rate_source} unavailable=no_invalidated_fraction"));
            continue;
        };
        let keys_seconds = inputs.keys_seconds_total;
        let seconds_when_every_commit_judges_the_full_domain =
            commit_cell_seconds(keys_seconds, fraction * full_states, rates.judge);
        let seconds_when_commits_judge_the_block_reuse_domain_only =
            commit_cell_seconds(keys_seconds, fraction * quick_states, rates.judge);
        let seconds_when_commits_query_the_library_only = keys_seconds;
        for (candidate, seconds) in [
            (
                "every_commit_judges_the_full_domain_by_block_keys",
                seconds_when_every_commit_judges_the_full_domain,
            ),
            (
                "commit_judges_the_block_reuse_domain_only",
                seconds_when_commits_judge_the_block_reuse_domain_only,
            ),
            (
                "commit_queries_the_library_only",
                seconds_when_commits_query_the_library_only,
            ),
        ] {
            bodies.push(format!(
                "name=compose_cell candidate={candidate} commit_class={class} rate_source={rate_source} invalidated_state_fraction={} seconds={seconds:.1} hours={:.3} band={}",
                fraction_text(fraction),
                seconds / 3600.0,
                band_of_seconds(seconds).name()
            ));
        }
        if candidate_is_over_a_day(seconds_when_every_commit_judges_the_full_domain) {
            bodies.push(format!("name=compose_trigger item=every_commit_judging_the_full_domain_takes_over_a_day commit_class={class} rate_source={rate_source} seconds={seconds_when_every_commit_judges_the_full_domain:.1}"));
        }
    }
    let full_by_the_main_agent = full_states / rates.total;
    bodies.push(format!(
        "name=compose_cell candidate=commit_queries_the_library_only_full_by_the_main_agent commit_class=any rate_source={rate_source} seconds={full_by_the_main_agent:.1} hours={:.3} band={}",
        full_by_the_main_agent / 3600.0,
        band_of_seconds(full_by_the_main_agent).name()
    ));
    for (hours, budget_seconds) in [(2u32, SECONDS_OF_TWO_HOURS), (24, SECONDS_OF_A_DAY)] {
        let open = open_under_budget(&inputs.point_states, budget_seconds, rates.judge);
        let total_states: u64 = inputs.point_states.iter().sum();
        bodies.push(format!(
            "name=compose_budget candidate=static_ignore_under_a_wall_clock_budget budget_hours={hours} rate_source={rate_source} points_total={} open_points={} open_point_fraction={} states_total={total_states} open_states={} open_state_fraction={}",
            inputs.point_states.len(),
            open.points,
            fraction_text(open.points as f64 / inputs.point_states.len() as f64),
            open.states,
            fraction_text(open.states as f64 / total_states as f64),
        ));
    }
    for (point_fraction, seconds) in cumulative_curve(&inputs.point_states, rates.judge) {
        bodies.push(format!(
            "name=compose_curve candidate=static_ignore_under_a_wall_clock_budget rate_source={rate_source} point_fraction={point_fraction:.1} cumulative_hours={:.3}",
            seconds / 3600.0
        ));
    }
    bodies
}

/// F4 的触发观测：甲的格子越过一天。
fn candidate_is_over_a_day(seconds: f64) -> bool {
    band_of_seconds(seconds) == WallClockBand::OverADay
}

/// J2 小项（建快照流）的全量态数：值抄自 `crash_enumeration_snapshot_creation_stream.rs` 文件头「崩溃点重放的枚举域 16842807 个状态」
/// （锚 A7，停机 T5 拿它与装置重录的建快照流的全量态数回比）。
/// 取号写两轮之后那份文件头已是 16842815；这个数同时是跑前登记作废条款 X3 写死的数，第一段的 J2 冷跑按它判过，改不改由登记定。
const SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST: u64 = 16_842_807;

/// J2 中项（覆盖写与 unlink 在快照之下的流）的全量态数：值抄自 `crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream.rs`
/// 文件头与全量用例的 `ignore` 说明（「崩溃点重放的枚举域 167837792 个状态」，锚 A8）；对不上不作废，只在 `name=anchor` 行报不符。
/// 取号写两轮之后测试文件改成 167837792（取号写一轮时 167837784），这里照新值重抄。
const OVERWRITE_AND_UNLINK_FULL_STATES_PINNED_BY_THE_TEST: u64 = 167_837_792;
#[cfg(test)]
const OVERWRITE_AND_UNLINK_TEST_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream.rs"
));

/// `keys` 的输出文本里的每一行 `name=flow`。
///
/// # Errors
/// 某一行的字段读不出。
fn flow_rows_of(text: &str) -> Result<Vec<FlowRow>, String> {
    text.lines()
        .map(|line| line.strip_prefix("E7RESULT ").unwrap_or(line))
        .filter(|line| line.starts_with("name=flow "))
        .map(FlowRow::from_body)
        .collect()
}

/// N 一侧的量：N 里每条流的 `keys_seconds`、全量与甲二态数、每个点的全量态数。
///
/// # Errors
/// N 里有流的全量态数装不进 u64（合算不出来）；N 里一条流都没有；N 里某条流没有块键文件。
fn compose_inputs_of(
    rows: &[FlowRow],
    keys_files: &[KeysFile],
    write_path_median_fraction: Option<f64>,
    write_path_upper_bound_fraction: Option<f64>,
) -> Result<ComposeInputs, String> {
    let nominal: Vec<&FlowRow> = rows
        .iter()
        .filter(|row| is_in_the_nominal_regular_layer_load(row))
        .collect();
    if nominal.is_empty() {
        return Err("常规层名义负载 N 里一条流都没有".to_string());
    }
    let mut inputs = ComposeInputs {
        keys_seconds_total: 0.0,
        full_total: 0,
        quick_total: 0,
        point_states: Vec::new(),
        write_path_median_fraction,
        write_path_upper_bound_fraction,
    };
    for row in nominal {
        inputs.keys_seconds_total += row.keys_seconds;
        inputs.full_total += row
            .full
            .ok_or_else(|| format!("{} 的全量态数装不进 u64，合算不出", row.label))?;
        inputs.quick_total += row
            .quick_tier
            .ok_or_else(|| format!("{} 的甲二态数装不进 u64，合算不出", row.label))?;
        let keys = keys_files
            .iter()
            .find(|file| file.row.label == row.label)
            .ok_or_else(|| format!("N 里的 {} 没有块键文件", row.label))?;
        let mut states_by_point: BTreeMap<usize, u64> = BTreeMap::new();
        for block in &keys.blocks {
            *states_by_point.entry(block.point_index).or_default() += block.states;
        }
        inputs.point_states.extend(states_by_point.into_values());
    }
    Ok(inputs)
}

/// `compare` 输出里 `kind=patch`、补丁位是 `pair` 的汇总行，只取属于 N 的组，按块数与态数合算出作废态数占比。
fn patch_state_fraction_in_the_nominal_load(
    compare_text: &str,
    pair: &str,
) -> Result<Option<f64>, String> {
    let (mut states, mut invalidated) = (0u64, 0u64);
    for fields in lines_named(compare_text, "compare_summary") {
        let group = required_field(&fields, "group")?;
        let in_the_load = group == "random_history-broad-FourGibibytes-steps0100"
            || group.starts_with("overwrite_chain-");
        if required_field(&fields, "kind")? == "patch"
            && required_field(&fields, "pair")? == pair
            && in_the_load
        {
            states += u64_field(&fields, "states_after")?;
            invalidated += u64_field(&fields, "invalidated_states")?;
        }
    }
    Ok((states > 0).then(|| invalidated as f64 / states as f64))
}

/// `compose` 的文本输入。
struct ComposeTexts {
    keys_output: String,
    compare_output: String,
    small_log: String,
    medium_log: String,
    library_bytes: String,
    recorded_blocks: String,
    final_pair: Option<String>,
}

/// 登记 X3：库不冷（`reused` 不为 0）、包含自检没过、中项的 `seconds` 不大于小项、小项 `states` 不是 16842807，任何一样都让那一项（自检、单调是整个 J2）作废。
fn voided_cold_runs(small: &ColdRun, medium: &ColdRun) -> Vec<String> {
    let mut reasons = Vec::new();
    for run in [small, medium] {
        if run.reused != 0 {
            reasons.push(format!("{}：reused={}，库不冷", run.item, run.reused));
        }
        if !run.outer_seconds_cover_the_inner_ones() {
            reasons.push(format!(
                "{}：seconds 包不住 compile_seconds、judge_seconds、seconds_storing 之和",
                run.item
            ));
        }
        if run.exit_code != 0 {
            reasons.push(format!("{}：驱动退出码 {}", run.item, run.exit_code));
        }
    }
    if medium.seconds <= small.seconds {
        reasons.push("中项的 seconds 不大于小项：两点分不出规模".to_string());
    }
    if small.states != SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST {
        reasons.push(format!(
            "小项 states={}，不是 {SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST}",
            small.states
        ));
    }
    reasons
}

/// `compose` 的全部输出行。交回退出码：0；停机条款没过 3（T5）；作废 4（X3）。
fn compose_lines(
    texts: &ComposeTexts,
    keys_files: &[KeysFile],
    output: &mut Output,
) -> Result<i32, String> {
    let rows = flow_rows_of(&texts.keys_output)?;
    let small =
        ColdRun::from_log(&texts.small_log).map_err(|error| format!("小项日志：{error}"))?;
    let medium =
        ColdRun::from_log(&texts.medium_log).map_err(|error| format!("中项日志：{error}"))?;
    let snapshot_recorded = rows
        .iter()
        .find(|row| row.label == "smoke-snapshot_creation_stream")
        .ok_or("J1 的输出里没有 smoke-snapshot_creation_stream 那一行")?;
    if snapshot_recorded.full != Some(small.states) {
        output.emit(&format!(
            "name=stop_check item=small_item_states_match_the_recorded_snapshot_full result=fail small_item_states={} recorded_full={:?}",
            small.states, snapshot_recorded.full
        ));
        output.emit_closing_line(
            "compose_done",
            "stopped=small_item_states_match_the_recorded_snapshot_full",
        );
        return Ok(EXIT_CODE_STOP_CLAUSE_FAILED);
    }
    output.emit(&format!(
        "name=stop_check item=small_item_states_match_the_recorded_snapshot_full result=pass small_item_states={}",
        small.states
    ));
    let voided = voided_cold_runs(&small, &medium);
    for reason in &voided {
        output.emit(&format!(
            "name=void item=cold_library_runs reason={}",
            reason.replace(' ', "_")
        ));
    }
    if !voided.is_empty() {
        output.emit_closing_line("compose_done", "voided=cold_library_runs");
        return Ok(EXIT_CODE_POSITIVE_CONTROL_FAILED);
    }
    for (identifier, expected, actual) in [
        (
            "small_item_states",
            snapshot_recorded
                .full
                .map_or_else(|| BEYOND_U64.to_string(), |full| full.to_string()),
            small.states.to_string(),
        ),
        (
            "medium_item_states",
            OVERWRITE_AND_UNLINK_FULL_STATES_PINNED_BY_THE_TEST.to_string(),
            medium.states.to_string(),
        ),
    ] {
        output.emit(&anchor_body(identifier, &expected, &actual).0);
    }
    let library_bytes = [
        library_bytes_of(&texts.library_bytes, "library-small")?,
        library_bytes_of(&texts.library_bytes, "library-medium")?,
    ];
    let recorded_blocks = [
        recorded_blocks_of(&texts.recorded_blocks, "library-small")?,
        recorded_blocks_of(&texts.recorded_blocks, "library-medium")?,
    ];
    for (source, run, bytes, blocks) in [
        ("small", &small, library_bytes[0], recorded_blocks[0]),
        ("medium", &medium, library_bytes[1], recorded_blocks[1]),
    ] {
        output.emit(&format!(
            "name=compose_rate rate_source={source} item={} states={} seconds={:.1} compile_seconds={:.1} judge_seconds={:.1} seconds_storing={:.1} recording_seconds={:.1} gpu_total_lines={} states_per_second_of_recording_and_judging={:.1} states_per_second_of_judging={:.1} library_bytes={bytes} library_bytes_per_state={:.4} recorded_blocks={blocks} library_bytes_per_recorded_block={:.1}",
            run.item,
            run.states,
            run.seconds,
            run.compile_seconds,
            run.judge_seconds,
            run.seconds_storing,
            run.recording_seconds(),
            run.gpu_total_lines,
            run.rate_total(),
            run.rate_judge(),
            bytes as f64 / run.states as f64,
            bytes as f64 / blocks as f64,
        ));
    }
    for (metric, small_seconds, medium_seconds) in [
        ("seconds", small.seconds, medium.seconds),
        ("judge_seconds", small.judge_seconds, medium.judge_seconds),
    ] {
        match fit_through_two_points(small.states, small_seconds, medium.states, medium_seconds) {
            Some((fixed, slope)) => output.emit(&format!("name=compose_fit metric={metric} fixed_seconds={fixed:.1} states_per_second={slope:.1}")),
            None => output.emit(&format!("name=compose_fit metric={metric} unavailable=the_two_points_do_not_rise")),
        }
    }
    let pairs = commit_pairs_of(&texts.compare_output)?;
    for pair in &pairs {
        output.emit(&format!(
            "name=commit_pair pair={} invalidated_block_fraction={} invalidated_state_fraction={} judging_unit_key_changed={}",
            pair.pair,
            fraction_text(pair.block_fraction),
            fraction_text(pair.state_fraction),
            pair.judging_unit_key_changed
        ));
    }
    let final_index = texts
        .final_pair
        .as_ref()
        .and_then(|name| pairs.iter().position(|pair| &pair.pair == name));
    let mut median_state_fraction = None;
    for (basis, fractions) in [
        (
            "blocks",
            pairs
                .iter()
                .map(|pair| pair.block_fraction)
                .collect::<Vec<f64>>(),
        ),
        (
            "states",
            pairs
                .iter()
                .map(|pair| pair.state_fraction)
                .collect::<Vec<f64>>(),
        ),
    ] {
        let Some(trajectory) = commit_pair_trajectory(&fractions, final_index) else {
            output.emit(&format!(
                "name=commit_pairs_trajectory basis={basis} computable=0 verdict={}",
                CommitPairsVerdict::StillMissing.name()
            ));
            continue;
        };
        if basis == "states" {
            median_state_fraction = Some(trajectory.median);
        }
        output.emit(&format!(
            "name=commit_pairs_trajectory basis={basis} computable={} positive={} peak={} median={} final={} verdict={}",
            trajectory.computable,
            trajectory.positive,
            fraction_text(trajectory.peak),
            fraction_text(trajectory.median),
            trajectory.final_value.map_or_else(|| "none".to_string(), fraction_text),
            commit_pairs_verdict(&fractions).name()
        ));
    }
    let upper_bound =
        patch_state_fraction_in_the_nominal_load(&texts.compare_output, "journal-mac")?;
    let inputs = compose_inputs_of(&rows, keys_files, median_state_fraction, upper_bound)?;
    output.emit(&format!(
        "name=compose_input flows_in_the_nominal_load={} keys_seconds_total={:.1} full_total={} quick_total={} points_total={}",
        rows.iter().filter(|row| is_in_the_nominal_regular_layer_load(row)).count(),
        inputs.keys_seconds_total,
        inputs.full_total,
        inputs.quick_total,
        inputs.point_states.len()
    ));
    for (source, run) in [("small", &small), ("medium", &medium)] {
        for body in cell_bodies(
            &inputs,
            source,
            &Rates {
                total: run.rate_total(),
                judge: run.rate_judge(),
            },
        ) {
            output.emit(&body);
        }
    }
    output.emit_closing_line(
        "compose_done",
        &format!("commit_pairs_computable={}", pairs.len()),
    );
    Ok(EXIT_CODE_OK)
}

fn read_texts(files: &[String]) -> Result<String, String> {
    let mut text = String::new();
    for file in files {
        text.push_str(
            &std::fs::read_to_string(file).map_err(|error| format!("读 {file} 失败：{error}"))?,
        );
        text.push('\n');
    }
    Ok(text)
}

fn run_compose_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(
        arguments,
        &[
            "--j1",
            "--keys",
            "--j3",
            "--j2-small",
            "--j2-medium",
            "--library-bytes",
            "--recorded-blocks",
            "--final-pair",
            "--out",
        ],
        &[],
    ) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let loaded = (|| -> Result<(ComposeTexts, Vec<KeysFile>, PathBuf), String> {
        parsed.paths("--j1")?;
        parsed.paths("--j3")?;
        let texts = ComposeTexts {
            keys_output: read_texts(parsed.all("--j1"))?,
            compare_output: read_texts(parsed.all("--j3"))?,
            small_log: read_texts(&[parsed.single("--j2-small")?.to_string()])?,
            medium_log: read_texts(&[parsed.single("--j2-medium")?.to_string()])?,
            library_bytes: read_texts(&[parsed.single("--library-bytes")?.to_string()])?,
            recorded_blocks: read_texts(&[parsed.single("--recorded-blocks")?.to_string()])?,
            final_pair: parsed.all("--final-pair").first().cloned(),
        };
        Ok((
            texts,
            keys_file_paths(&parsed.paths("--keys")?, Expansion::Full)?
                .iter()
                .filter(|path| {
                    label_is_in_the_nominal_regular_layer_load(&label_of_the_keys_file(path))
                })
                .map(|path| read_keys_file(path, Expansion::Full))
                .collect::<Result<Vec<KeysFile>, String>>()?,
            PathBuf::from(parsed.single("--out")?),
        ))
    })();
    let (texts, keys_files, out_file) = match loaded {
        Ok(loaded) => loaded,
        Err(message) => return usage_error(&message),
    };
    let mut output = Output::new();
    let exit_code = match compose_lines(&texts, &keys_files, &mut output) {
        Ok(exit_code) => exit_code,
        Err(message) => {
            eprintln!("合算失败：{message}");
            return EXIT_CODE_STOP_CLAUSE_FAILED;
        }
    };
    if let Err(message) = output.write_to(&out_file) {
        eprintln!("写 {} 失败：{message}", out_file.display());
        return EXIT_CODE_USAGE;
    }
    exit_code
}

// ───────────────────────────── 照抄：冒烟层三条流的建法（测试文件里私有的 `prepare`） ─────────────────────────────
//
// 登记 1.1「冒烟层比较集 R4」：测试文件里私有的 `prepare` 逐字照抄，抄的每一段由停机 T2 钉住（`selfcheck` 的 `copied_text_verbatim`）。
// 每一段上下有 `COPIED-BEGIN` / `COPIED-END` 两行标记，标记里写出处（文件与起止行）；段里每一行去掉包进模块的 4 格缩进之后，
// 要在原文件里整段逐字找得到。允许的替换只有这一种：包进模块的缩进（每行前多 4 个空格）。
// 每个模块自己带 `#[rustfmt::skip]`，rustfmt 不会重排照抄的行。

/// 覆盖写固定脚本流（到 E 再正常卸载那一支，`Script::ReuseAfterRaisingFloorThenNormalUnmount`）：`crash-case:overwrite-fixed-script-stream`。
#[rustfmt::skip]
mod fixed_script_stream {
    #![allow(
        dead_code,
        reason = "照抄的测试代码里只有 prepare 的这一支被调到，别的脚本变体、常量与函数在这个装置里没人用"
    )]

    use super::common::{
        build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    };
    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    use singlefs_core::mount::{
        mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount,
        RollbackTarget, ShadowLedger, UnmountRaisedTheFloor, Unmounted,
    };
    use singlefs_core::transaction::{
        publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
    };
    use singlefs_core::unit::{UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
    use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, SLOT_BYTES};
    use singlefs_harness::memory_pool::{
        writes_and_segments_with_stream_indexes_and_entries, MemoryPool, PublishedVersion,
        RetainedWrite,
    };
    use singlefs_harness::segments::StepKind;
    use singlefs_harness::{RecordedEntrySpan, RecordedPublishEntry};

    // COPIED-BEGIN fixed_script_head crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:53-90
    const SECOND_FILE_BYTES: usize = 4100;

    fn second_content() -> Vec<u8> {
        (0..SECOND_FILE_BYTES)
            .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
            .collect()
    }

    struct Prepared {
        base: MemoryPool,
        writes: Vec<RetainedWrite>,
        segments: Vec<Vec<usize>>,
        /// 被判的那次根槽 FUA 写在写表里的下标：脚本最后一次发布的根（到 B 为止就是 B 的，到 C 为止是 C 的）。
        judged_root_index: usize,
        /// A 的根槽 FUA 写在写表里的下标：它之后的写都是 B 的。
        first_root_index: usize,
        versions: Vec<PublishedVersion>,
    }

    /// 固定脚本跑到哪一步（里程碑「覆盖写、释放、回退与复用」步 0）：到 B 为止是靶向对照用的两次发布流；到 E 再正常卸载是崩溃点重放全量跑的那条流。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Script {
        SecondVersionOnly,
        /// 到 C 为止：B 之后进程重开、可写挂载（取号、写行、暖机两次），再发布 C。
        ThirdVersion,
        /// 到 D 为止：C 之后在同一个会话里挂着的时候回退到 A 的根 (1, 3)——一次向前发布 D（txg 9，实例仍是 2，
        /// D23（journal 的角色与格式） 已定项 14），不取号、不写行、不暖机。
        RollbackToFirstVersion,
        /// 到 E 为止再正常卸载：回退之后再覆盖写五次（txg 10–14，第一次释放 D 复活的 A 的数据单元、释放代 10），抬 F 到 11
        /// （两次空发布 txg 15、16），再发布 E（txg 17）——数据单元落回 mkfs 实例表那两槽 50176；最后正常卸载（B1，D16（发布语义）
        /// 已定项 1「正常卸载」）：先把 F = 17 写进每块盘的系统配置、过一道屏障，再推带卸载记号的空发布直到每块盘上都有一条（txg 18、19）。
        /// 这是崩溃点重放全量跑的那条流。卸载放在最后：它把 F 抬到现行 txg，排在回退之前的话回退候选集里就没有 A 了（D16（发布语义）
        /// 已定项 1 的候选集要 txg ≥ F_生效，与「抬 F 排在回退之后」同一个理由）。
        ReuseAfterRaisingFloorThenNormalUnmount,
        /// 到 E 之后（不卸载）再覆盖写两次：txg 18 的数据单元落回 50178–50179（mkfs 那片第 0 版树表那一槽起，没有哪条记录点名它），
        /// txg 19 的数据单元落回 A 的数据单元那一对槽 50180——环里第一条点名块被合法复用的记录是 A 那一条。
        ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish,
    }
    // COPIED-END fixed_script_head

    // COPIED-BEGIN fixed_script_body crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:137-572
    /// 回退 D（txg 9）那次发布在零根的谱系树上第一次追加谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条），多写的两个单元的落点：
    /// 谱系容器（码 3 打包记录单元，32768 字节）与谱系树根（码 2 索引节点，16384 字节），两块盘同槽各一份
    /// （根环线第 9 批 9-8 的探针在这条流上量到的槽号）。
    const LINEAGE_CONTAINER_SLOT_OF_THE_ROLLBACK_PUBLISH: u64 = 50352;
    const LINEAGE_TREE_ROOT_SLOT_OF_THE_ROLLBACK_PUBLISH: u64 = 50354;

    /// 回退 D 是这条脚本的第几条根（从 0 数：实例 1 的 txg 1–4，实例 2 的 txg 5–8，D 是 txg 9）。
    const ROOT_INDEX_OF_THE_ROLLBACK_PUBLISH: usize = 8;

    /// 回退 D 那次发布的单元写里落在 `slot` 起、单元类 `unit_class`、整单元 `unit_bytes` 字节的有几个（每块盘一份就是 2）。
    fn units_of_the_rollback_publish_at(
        writes: &[RetainedWrite],
        root_indexes: &[usize],
        slot: u64,
        unit_class: u8,
        unit_bytes: u64,
    ) -> usize {
        let after_the_root_before_it = root_indexes[ROOT_INDEX_OF_THE_ROLLBACK_PUBLISH - 1] + 1;
        let its_root = root_indexes[ROOT_INDEX_OF_THE_ROLLBACK_PUBLISH];
        writes[after_the_root_before_it..its_root]
            .iter()
            .filter(|write| write.kind == StepKind::UnitWrite)
            .filter(|write| write.offset.0 == slot * SLOT_BYTES)
            .filter(|write| {
                write.bytes().is_some_and(|bytes| {
                    u64::try_from(bytes.len()).expect("一个单元的字节数装得进 u64") == unit_bytes
                        && bytes[6] == unit_class
                })
            })
            .count()
    }

    /// 到 E 之后再覆盖写一次（txg 18）那一版的数据单元落点。
    const TXG_18_DATA_UNIT_SLOT: u64 = 50178;
    /// A（txg 3）的数据单元那一对槽：到 E 之后第二次覆盖写（txg 19）复用它。
    const FIRST_DATA_UNIT_SLOT: u64 = 50180;

    const THIRD_FILE_BYTES: usize = 2500;

    fn later_content(seed: usize) -> Vec<u8> {
        (0..3000 + seed)
            .map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256"))
            .collect()
    }

    fn third_content() -> Vec<u8> {
        (0..THIRD_FILE_BYTES)
            .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
            .collect()
    }

    fn overwrite(
        pool: &mut BuiltPool,
        content: &[u8],
        instance: InstanceGeneration,
    ) -> TransactionOutput {
        let parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.output,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .expect("覆盖写")
    }

    /// 正常卸载（影子账开着），录制流上把这一段记成卸载入口发的（C557（卸载记号只由卸载入口打没有检查））：带文件的一版上卸载必抬 F，
    /// 交回那一串；现行版本换成那一串最后落盘的那一次。
    fn unmount_recorded(pool: &mut BuiltPool) -> UnmountRaisedTheFloor {
        let mut current = PoolVersion::WithFile(pool.output.clone());
        let stream = pool.stream.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let allocator = &mut pool.allocator;
        let unmounted = stream
            .record_entry(RecordedPublishEntry::Unmount, || {
                unmount(
                    &parameters(),
                    devices,
                    allocator,
                    &mut current,
                    ShadowLedger::On,
                )
            })
            .expect("正常卸载");
        pool.output = current
            .into_file_version()
            .expect("带文件的一版卸载之后仍带文件");
        match unmounted {
            Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
            Unmounted::NothingWrittenOnAVersionWithoutFile {
                current: reported_version,
            } => {
                panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」：{reported_version:?}")
            }
        }
    }

    /// mkfs 之后那一段录制流切成写表与段：入口段（卸载入口发的那几段）一起交给切段，带卸载记号的根槽写只许落在卸载入口里（C557）。
    fn writes_and_segments_after_mkfs(pool: &BuiltPool) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
        let operations = pool.retained_operations();
        let entry_spans: Vec<RecordedEntrySpan> = pool
            .stream
            .entry_spans()
            .into_iter()
            .map(|span| RecordedEntrySpan {
                entry: span.entry,
                operations: span.operations.start - pool.mkfs_operation_count
                    ..span.operations.end - pool.mkfs_operation_count,
            })
            .collect();
        let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
            &operations[pool.mkfs_operation_count..],
            &entry_spans,
            &geometry(),
        );
        (writes, segments)
    }

    fn prepare(tag: &str, script: Script) -> Prepared {
        let mut pool = build_pool(tag);
        let second = overwrite(&mut pool, &second_content(), InstanceGeneration(1));
        let mut versions = vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: second_content(),
            },
        ];
        if script != Script::SecondVersionOnly {
            pool.output = second;
            // 进程重开：取号两轮各 2、写行、暖机两次（txg 5 / 6 落盘 0、txg 7 落盘 1），文件还是第二次的内容。
            let mut devices = pool.reopen_recorded();
            let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
            pool.devices = Some(devices);
            pool.allocator = mounted.allocator;
            pool.output = mounted
                .current
                .into_file_version()
                .expect("B 之后重开，现行那一版带文件");
            for txg in 5..=7 {
                versions.push(PublishedVersion {
                    instance: InstanceGeneration(2),
                    checkpoint_txg: CheckpointTxg(txg),
                    content: second_content(),
                });
            }
            let third = overwrite(&mut pool, &third_content(), InstanceGeneration(2));
            versions.push(PublishedVersion {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(8),
                content: third_content(),
            });
            if script != Script::ThirdVersion {
                pool.output = third;
                // 同一个会话里挂着的时候回退到 A 的根：一次向前发布 D（txg 9 落盘 0），文件回到第一次的内容。
                {
                    let rollback_parameters = parameters();
                    let open_devices = pool.devices.as_mut().expect("镜像还开着");
                    roll_back_by_a_forward_publish(
                        &rollback_parameters,
                        open_devices,
                        &mut pool.allocator,
                        &mut pool.output,
                        RollbackTarget {
                            instance: InstanceGeneration(1),
                            checkpoint_txg: CheckpointTxg(3),
                        },
                    )
                    .expect("挂着的时候回退到 A");
                }
                versions.push(PublishedVersion {
                    instance: InstanceGeneration(2),
                    checkpoint_txg: CheckpointTxg(9),
                    content: file_content(),
                });
                if matches!(
                    script,
                    Script::ReuseAfterRaisingFloorThenNormalUnmount
                        | Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish
                ) {
                    let mut latest = Vec::new();
                    for (txg, seed) in [(10u64, 13usize), (11, 17), (12, 19), (13, 23), (14, 29)] {
                        latest = later_content(seed);
                        pool.output = overwrite(&mut pool, &latest, InstanceGeneration(2));
                        versions.push(PublishedVersion {
                            instance: InstanceGeneration(2),
                            checkpoint_txg: CheckpointTxg(txg),
                            content: latest.clone(),
                        });
                    }
                    let mut current = pool.output.clone();
                    let raise_devices = pool.devices.as_mut().expect("镜像还开着");
                    let raised = raise_rollback_floor(
                        &parameters(),
                        raise_devices,
                        &mut pool.allocator,
                        &mut current,
                        CheckpointTxg(11),
                        ShadowLedger::On,
                    )
                    .expect("抬 F");
                    assert_eq!(raised.publishes.len(), 2, "txg 15 落盘 0、txg 16 落盘 1");
                    pool.output = current;
                    for txg in 15..=16 {
                        versions.push(PublishedVersion {
                            instance: InstanceGeneration(2),
                            checkpoint_txg: CheckpointTxg(txg),
                            content: latest.clone(),
                        });
                    }
                    let reuse = overwrite(&mut pool, &later_content(31), InstanceGeneration(2));
                    assert_eq!(
                        reuse.data_pointers[0].locations[0].slot.0,
                        50176,
                        "E 的数据单元落回最低的可再分配偶数槽对：mkfs 实例表那两槽（释放代 5，抬 F 到 11 回收了）"
                    );
                    versions.push(PublishedVersion {
                        instance: InstanceGeneration(2),
                        checkpoint_txg: CheckpointTxg(17),
                        content: later_content(31),
                    });
                    pool.output = reuse;
                    match script {
                        Script::ReuseAfterRaisingFloorThenNormalUnmount => {
                            let unmounted = unmount_recorded(&mut pool);
                            assert_eq!(
                                unmounted
                                    .publishes
                                    .iter()
                                    .map(|publish| publish.root.checkpoint_txg)
                                    .collect::<Vec<_>>(),
                                vec![CheckpointTxg(18), CheckpointTxg(19)],
                                "卸载推两次空发布：txg 18 落盘 0、txg 19 落盘 1（17 除以 3 余 2）"
                            );
                            for txg in 18..=19 {
                                versions.push(PublishedVersion {
                                    instance: InstanceGeneration(2),
                                    checkpoint_txg: CheckpointTxg(txg),
                                    content: later_content(31),
                                });
                            }
                        }
                        Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => {
                            for (txg, seed, data_unit_slot, why) in [
                                (
                                    18u64,
                                    37usize,
                                    TXG_18_DATA_UNIT_SLOT,
                                    "txg 18 的数据单元落回下一对可再分配的槽（抬 F 回收了、E 用掉的是更低的 50176）",
                                ),
                                (
                                    19,
                                    41,
                                    FIRST_DATA_UNIT_SLOT,
                                    "txg 19 的数据单元落回 A 的数据单元那一对槽（释放代 10，抬 F 到 11 回收了）",
                                ),
                            ] {
                                let later = overwrite(&mut pool, &later_content(seed), InstanceGeneration(2));
                                assert_eq!(
                                    later.data_pointers[0].locations[0].slot.0,
                                    data_unit_slot,
                                    "{why}"
                                );
                                versions.push(PublishedVersion {
                                    instance: InstanceGeneration(2),
                                    checkpoint_txg: CheckpointTxg(txg),
                                    content: later_content(seed),
                                });
                                pool.output = later;
                            }
                        }
                        Script::SecondVersionOnly
                        | Script::ThirdVersion
                        | Script::RollbackToFirstVersion => {
                            unreachable!("外面那一层只放行到 E 的两个脚本")
                        }
                    }
                }
            }
        }
        let base = pool.memory_pool_after_mkfs();
        let (writes, segments) = writes_and_segments_after_mkfs(&pool);
        let root_indexes: Vec<usize> = writes
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
            .map(|(index, _)| index)
            .collect();
        // 谱系那两个单元的槽号先于段序列钉：回退不追加谱系记录时 D 那一段从 24 退回 20，段序列那一条也红，这一条先说清缺的是谱系那两个单元。
        match script {
            Script::SecondVersionOnly | Script::ThirdVersion => {}
            Script::RollbackToFirstVersion
            | Script::ReuseAfterRaisingFloorThenNormalUnmount
            | Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => {
                assert_eq!(
                    (
                        units_of_the_rollback_publish_at(
                            &writes,
                            &root_indexes,
                            LINEAGE_CONTAINER_SLOT_OF_THE_ROLLBACK_PUBLISH,
                            UNIT_CLASS_PACKED,
                            DATA_UNIT_BYTES
                        ),
                        units_of_the_rollback_publish_at(
                            &writes,
                            &root_indexes,
                            LINEAGE_TREE_ROOT_SLOT_OF_THE_ROLLBACK_PUBLISH,
                            UNIT_CLASS_INDEX_NODE,
                            NODE_BYTES
                        )
                    ),
                    (2, 2),
                    "回退 D 那次发布写出谱系容器（码 3，槽 {LINEAGE_CONTAINER_SLOT_OF_THE_ROLLBACK_PUBLISH}）与谱系树根（码 2，槽 {LINEAGE_TREE_ROOT_SLOT_OF_THE_ROLLBACK_PUBLISH}），两块盘各一份"
                );
            }
        }
        let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
        // 每次发布：它的单元写、两份 journal 记录、一条根槽写、之后两块盘各一次系统配置槽轮换，共「单元写数 + 5」次写；
        // C577 之后轮换之后、发布返回之前有一道屏障，上一次发布的系统配置槽轮换自成一段，不再与这一次的单元写并成一段
        // （登记表八末尾那条 ⚠️ 说的并段不再有）。单元写数：暖机 0、A 与 B 与 C 各 24、
        // 写行发布 18、写行之后的暖机各 16、回退 D 24（分配记录树 7 个节点 + 记账 + 映射 + 树表 = 10 个固定点单元，加谱系容器与谱系树根
        // 2 个——D18（块里携带什么信息） 已定项 24 第 5 条，回退那次追加一条谱系记录；加谱系记录之前 20）、覆盖写与 E 各 28、
        // 抬 F 与卸载推的空发布各 16。取号、抬 F 与卸载那一串开头各多
        // 两块盘一次系统配置写（取号写实例代号，抬 F 与卸载先写新 F，D16（发布语义） 已定项 1「抬 F 那一串」），它们之前各一道屏障
        // （代码审阅第 19 条：覆写较旧那一槽之前，上一次轮换写的那一槽先持久），与上一次的轮换各自成段（2、2；改之前并成 4 写一段）；
        // 取号写两轮（D22（单元原子性怎么合成） 已定项 8 落地），每轮两盘各一写、轮后池屏障，两轮各成一段（2、2），这条流开头与 B 之后重开各一处。
        match script {
            Script::SecondVersionOnly => {
                assert_eq!(
                    sizes,
                    vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2],
                    "取号两轮各成一段；A 的两个系统配置槽写自成一段、B 的 24 个单元写自成一段：C577 之后轮换之后有屏障（改之前并成 26）"
                );
                assert_eq!(
                    writes.len(),
                    72,
                    "取号两轮各 2 + 暖机 5 × 2 + A 29 + B 29 次写"
                );
            }
            Script::ThirdVersion => {
                // B 的两个系统配置槽写自成一段、取号两轮各两个、各自成一段（2、2、2：取号写之前与每轮之后有屏障）；写行发布 18 个单元写自成一段（取号第二轮之后有屏障）；
                // 暖机两次各 16 个单元写、C 的 24 个单元写各自成一段，上一次发布的两个系统配置槽写在它们前面自成一段（C577 之前并成 18、18、26）。
                assert_eq!(
                    sizes,
                    vec![
                        2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1,
                        2, 16, 2, 1, 2, 24, 2, 1, 2
                    ],
                    "固定脚本到 C 为止的段序列"
                );
                assert_eq!(
                    writes.len(),
                    72 + 4 + 23 + 21 + 21 + 29,
                    "到 B 72 + 取号两轮 4 + 写行 23 + 暖机 21 × 2 + C 29"
                );
            }
            Script::RollbackToFirstVersion => {
                // 挂着时的向前回退 D：24 个单元写自成一段（10 个固定点单元加谱系容器与谱系树根，各两块盘；加谱系记录之前 20）、
                // C 的两个系统配置槽写在它前面自成一段（C577 之前并成 22），不取号、不写行、不暖机。
                assert_eq!(
                    sizes,
                    vec![
                        2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1,
                        2, 16, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2
                    ],
                    "固定脚本到 D 为止的段序列"
                );
                assert_eq!(writes.len(), 170 + 29, "到 C 170 + D 29");
            }
            Script::ReuseAfterRaisingFloorThenNormalUnmount => {
                // 五次覆盖写各 28 个单元写自成一段、上一次的两个系统配置槽写在它前面自成一段（C577 之前并成 30）；txg 14 的轮换自成一段、
                // 抬 F 先写系统配置自成一段（2、2），两次空发布各 16 个单元写各自成一段（第二次前面是第一次的轮换 2，C577 之前并成 18）；
                // E 同覆盖写（2、28）；卸载同抬 F（2、2、16、2、16），末尾是 txg 19 的轮换。
                assert_eq!(
                    sizes,
                    vec![
                        2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1,
                        2, 16, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1,
                        2, 28, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16,
                        2, 1, 2, 16, 2, 1, 2
                    ],
                    "固定脚本到 E 再正常卸载的段序列"
                );
                assert_eq!(
                    writes.len(),
                    199 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21),
                    "到 D 199 + 五次覆盖写 165 + 抬 F 44 + E 33 + 卸载 44"
                );
            }
            Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => {
                // 到 E 的段序列末尾那段（E 的两个系统配置槽写）自成一段，txg 18 的 28 个单元写自成一段（C577 之前并成 30），txg 19 同形。
                assert_eq!(
                    sizes,
                    vec![
                        2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1,
                        2, 16, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1,
                        2, 28, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 28, 2,
                        1, 2, 28, 2, 1, 2
                    ],
                    "固定脚本到 E 之后再覆盖写两次的段序列"
                );
                assert_eq!(
                    writes.len(),
                    199 + 5 * 33 + 44 + 33 + 2 * 33,
                    "到 E 441 + txg 18、19 的覆盖写各 33"
                );
            }
        }
        let expected_roots = match script {
            Script::SecondVersionOnly => 4,
            Script::ThirdVersion => 8,
            Script::RollbackToFirstVersion => 9,
            Script::ReuseAfterRaisingFloorThenNormalUnmount
            | Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => 19,
        };
        assert_eq!(root_indexes.len(), expected_roots, "每次发布一条根槽写");
        Prepared {
            base,
            writes,
            segments,
            judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
            first_root_index: root_indexes[2],
            versions,
        }
    }
    // COPIED-END fixed_script_body

    /// 装置要的几样：基镜像、写表、段、被判的根槽写下标、版本表。
    pub struct PreparedFixedScript {
        pub base: MemoryPool,
        pub writes: Vec<RetainedWrite>,
        pub segments: Vec<Vec<usize>>,
        pub judged_root_index: usize,
        pub versions: Vec<PublishedVersion>,
    }

    pub fn prepare_through_the_unmount(tag: &str) -> PreparedFixedScript {
        let prepared = prepare(tag, Script::ReuseAfterRaisingFloorThenNormalUnmount);
        PreparedFixedScript {
            base: prepared.base,
            writes: prepared.writes,
            segments: prepared.segments,
            judged_root_index: prepared.judged_root_index,
            versions: prepared.versions,
        }
    }
}

/// 多记录发布流（取号、暖机、A、B、C）：`crash-case:multi-record-publish-stream`。
#[rustfmt::skip]
mod multi_record_publish_stream {
    #![allow(
        dead_code,
        reason = "照抄的测试代码里发布边界那一半（MultiRecordPublish 的记录写下标）在这个装置里没人读"
    )]

    use super::common::{build_pool, file_content, geometry};
    use super::common_crash_points::{content_needing, sequential_write};
    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    use singlefs_core::journal::record_offset;
    use singlefs_core::transaction::TransactionOutput;
    use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
    use singlefs_harness::memory_pool::{
        writes_and_segments, MemoryPool, PublishedVersion, RetainedWrite,
    };
    use singlefs_harness::segments::StepKind;

    // COPIED-BEGIN multi_record_publish crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:74-218
    /// 一次多条记录的发布在写表里的样子：它每一条记录的两份写各在写表的哪一格，它的根在哪一版、上一版是哪一版。
    struct MultiRecordPublish {
        /// 这次发布的记录按 jsn 升序，每条两盘各一份：写表下标。
        record_writes_in_order: Vec<Vec<usize>>,
        version: (InstanceGeneration, CheckpointTxg),
        version_before: (InstanceGeneration, CheckpointTxg),
        content_before: Vec<u8>,
    }

    impl MultiRecordPublish {
        fn is_any_copy_persisted(persisted: &[bool], copies: &[usize]) -> bool {
            copies.iter().any(|copy| persisted[*copy])
        }

        /// 这一状态里这次发布的第一条记录至少落了一份，而末条一份都没落：前缀停在这次发布中间。
        fn is_cut_before_its_last_record(&self, persisted: &[bool]) -> bool {
            let first = self.record_writes_in_order.first().expect("至少两条记录");
            let last = self.record_writes_in_order.last().expect("至少两条记录");
            Self::is_any_copy_persisted(persisted, first)
                && !Self::is_any_copy_persisted(persisted, last)
        }
    }

    struct Prepared {
        base: MemoryPool,
        writes: Vec<RetainedWrite>,
        segments: Vec<Vec<usize>>,
        judged_root_index: usize,
        versions: Vec<PublishedVersion>,
        multi_record_publishes: Vec<MultiRecordPublish>,
    }

    /// 写表里写到某个 journal 记录槽（两盘同偏移）的那几格。
    fn writes_to_record(writes: &[RetainedWrite], counter: u64) -> Vec<usize> {
        let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES);
        writes
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::JournalRecord && write.offset == offset)
            .map(|(index, _)| index)
            .collect()
    }

    fn multi_record_publish(
        writes: &[RetainedWrite],
        output: &TransactionOutput,
        before: &TransactionOutput,
        content_before: Vec<u8>,
    ) -> MultiRecordPublish {
        let mut record_writes_in_order: Vec<Vec<usize>> = output
            .earlier_records_of_this_publish
            .iter()
            .map(|earlier| writes_to_record(writes, earlier.record.counter))
            .collect();
        record_writes_in_order.push(writes_to_record(writes, output.record.counter));
        for copies in &record_writes_in_order {
            assert_eq!(copies.len(), 2, "每条记录两盘各一份");
        }
        MultiRecordPublish {
            record_writes_in_order,
            version: (output.root.instance, output.root.checkpoint_txg),
            version_before: (before.root.instance, before.root.checkpoint_txg),
            content_before,
        }
    }

    fn prepare(tag: &str) -> Prepared {
        let mut pool = build_pool(tag);
        let first = pool.output.clone();
        let second_content = content_needing(2, 1);
        let second = sequential_write(&mut pool, &second_content);
        assert_eq!(
            second.earlier_records_of_this_publish.len() + 1,
            2,
            "B：两个数据单元两条记录"
        );
        assert_eq!(
            second.rewritten.len(),
            2 + 2 + 2 + 5 + 3,
            "B：两个数据单元 + extent 下段根兼叶与上段根 + inode 叶容器与根 + 分配记录树五个节点 + 记账树、中央映射树、树表 = 14 个单元\
             （验收第 3 条的「≤ 10」今天不成立，见文件头）：{:?}",
            second.rewritten
        );
        let third_content = content_needing(3, 2);
        let third = sequential_write(&mut pool, &third_content);
        assert_eq!(
            third.earlier_records_of_this_publish.len() + 1,
            3,
            "C：三个数据单元三条记录"
        );
        assert_eq!(
            third.rewritten.len(),
            3 + 2 + 2 + 5 + 3,
            "C：三个数据单元 + 与 B 同样的 12 个单元 = 15 个单元（「≤ 10」今天不成立，见文件头）：{:?}",
            third.rewritten
        );

        let base = pool.memory_pool_after_mkfs();
        let operations = pool.retained_operations();
        let (writes, segments) =
            writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
        let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
        assert_eq!(
            sizes,
            vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2],
            "取号两轮各 2、暖机两次、上一次的系统配置槽轮换 2 与 A 的 24 个单元写各自成一段；A 的轮换 2、B 的 28 个单元写、两条记录 4 写；\
             B 的轮换 2、C 的 30 个单元写、三条记录 6 写（C577 之后轮换之后有屏障，改之前轮换并进下一次的单元段：26、30、32）"
        );
        let root_indexes: Vec<usize> = writes
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
            .map(|(index, _)| index)
            .collect();
        assert_eq!(root_indexes.len(), 5, "暖机两次、A、B、C 各一条根槽写");
        let versions = vec![
            PublishedVersion {
                instance: first.root.instance,
                checkpoint_txg: first.root.checkpoint_txg,
                content: file_content(),
            },
            PublishedVersion {
                instance: second.root.instance,
                checkpoint_txg: second.root.checkpoint_txg,
                content: second_content.clone(),
            },
            PublishedVersion {
                instance: third.root.instance,
                checkpoint_txg: third.root.checkpoint_txg,
                content: third_content,
            },
        ];
        let multi_record_publishes = vec![
            multi_record_publish(&writes, &second, &first, file_content()),
            multi_record_publish(&writes, &third, &second, second_content),
        ];
        Prepared {
            base,
            writes,
            segments,
            judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
            versions,
            multi_record_publishes,
        }
    }
    // COPIED-END multi_record_publish

    pub fn prepare_through_the_third_publish(tag: &str) -> (MemoryPool, Vec<RetainedWrite>, Vec<Vec<usize>>, usize, Vec<PublishedVersion>) {
        let prepared = prepare(tag);
        (prepared.base, prepared.writes, prepared.segments, prepared.judged_root_index, prepared.versions)
    }
}

/// 新池新建文件流（取号、暖机、A）：`crash-case:new-pool-file-creation-stream`。`prepare` 的前半（到被判的根槽写那一步）逐字照抄，
/// 后半（`TransactionPublishInRecordedStream` 的自检）是那个测试文件观察者的东西，这个装置不用，不抄。
#[rustfmt::skip]
mod new_pool_file_creation_stream {
    #![allow(
        dead_code,
        reason = "照抄的片段把 pool 之后用不到的变量也建了出来"
    )]

    use super::common::{build_pool, geometry};
    use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, RetainedWrite};
    use singlefs_harness::segments::StepKind;

    pub struct PreparedNewPool {
        pub base: MemoryPool,
        pub writes: Vec<RetainedWrite>,
        pub segments: Vec<Vec<usize>>,
        pub root_index: usize,
    }

    pub fn prepare_through_the_first_file(tag: &str) -> PreparedNewPool {
    // COPIED-BEGIN new_pool_prepare_body crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:792-810
        let pool = build_pool(tag);
        let base = pool.memory_pool_after_mkfs();
        let operations = pool.retained_operations();
        let (writes, segments) =
            writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
        assert_eq!(
            segments.iter().map(Vec::len).collect::<Vec<_>>(),
            vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2],
            "取号写两轮、轮后各一道池屏障，各成一段；C577 之后暖机第二次的系统配置槽轮换过了发布末尾那道屏障，不再与 A 的 24 个单元写同段"
        );
        assert_eq!(
            writes.len(),
            43,
            "取号两轮各 2 + 暖机两次各 5 + A 29（24 个单元写、两份记录、根槽、两块盘的系统配置槽轮换）"
        );
        let root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("写流里有根槽那一条");
    // COPIED-END new_pool_prepare_body
        PreparedNewPool {
            base,
            writes,
            segments,
            root_index,
        }
    }
}

// ───────────────────────────── 固定流：覆盖写链三档与冒烟层四条 ─────────────────────────────

/// 覆盖写链每次覆盖写的数据单元数（登记 5.1：1、4、16 三档；4 是主取样，1 与 16 是方向相反的取样点）。
const OVERWRITE_CHAIN_UNITS_PER_OVERWRITE: [usize; 3] = [1, 4, 16];

/// 一条手写流在内存里摆好的样子（基镜像是建好的池，不是重放 mkfs 的写）。
struct PreparedFlow {
    family: &'static str,
    tier: &'static str,
    label: String,
    ending: String,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions_with_a_file: Vec<PublishedVersion>,
    versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
}

impl PreparedFlow {
    /// 转储：基镜像存成「池里写过的每个扇区各一次整扇区写」（`base_writes_of_the_pool`），叠回同样大小的空盘得到同一个池。
    /// 手写流不记 mkfs 占了录制流开头的几步（记 0）。
    fn dump(&self) -> FlowDump {
        FlowDump {
            family: self.family.to_string(),
            label: self.label.clone(),
            tier: self.tier.to_string(),
            seed: None,
            ending: self.ending.clone(),
            device_bytes: self.base.device_size_in_bytes,
            device_identities: self
                .base
                .devices
                .keys()
                .map(|identity| identity.0)
                .collect(),
            operations_of_the_make_filesystem: 0,
            base_writes: write_dump::base_writes_of_the_pool(&self.base),
            writes: self.writes.clone(),
            segments: self.segments.clone(),
            judged_root_index: Some(self.judged_root_index),
            versions_with_a_file: self.versions_with_a_file.clone(),
            versions_without_a_file: self.versions_without_a_file.clone(),
            compaction_operations: None,
        }
    }
}

fn smoke_flow(
    label: &str,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
) -> PreparedFlow {
    PreparedFlow {
        family: "smoke",
        tier: "baseline",
        label: format!("smoke-{label}"),
        ending: "NotAHistory".to_string(),
        base,
        writes,
        segments,
        judged_root_index,
        versions_with_a_file: versions,
        versions_without_a_file: Vec::new(),
    }
}

/// `crash-case:new-pool-file-creation-stream`：被判的版本就是被判的那条根槽写出的那一代，内容是第一个文件（测试文件 `single_version` 的做法）。
fn prepared_new_pool_stream() -> PreparedFlow {
    let prepared = new_pool_file_creation_stream::prepare_through_the_first_file("e184-new-pool");
    let (instance, checkpoint_txg) = root_identity_written_by(
        prepared.writes[prepared.root_index]
            .bytes()
            .expect("根槽写是普通写，带着字节"),
    );
    let versions = vec![PublishedVersion {
        instance,
        checkpoint_txg,
        content: common::file_content(),
    }];
    smoke_flow(
        "new_pool_file_creation_stream",
        prepared.base,
        prepared.writes,
        prepared.segments,
        prepared.root_index,
        versions,
    )
}

/// `crash-case:overwrite-fixed-script-stream`：到 E 再正常卸载那一支。
fn prepared_fixed_script_stream() -> PreparedFlow {
    let prepared = fixed_script_stream::prepare_through_the_unmount("e184-fixed-script");
    smoke_flow(
        "overwrite_fixed_script_stream",
        prepared.base,
        prepared.writes,
        prepared.segments,
        prepared.judged_root_index,
        prepared.versions,
    )
}

/// `crash-case:multi-record-publish-stream`。
fn prepared_multi_record_stream() -> PreparedFlow {
    let (base, writes, segments, judged_root_index, versions) =
        multi_record_publish_stream::prepare_through_the_third_publish("e184-multi-record");
    smoke_flow(
        "multi_record_publish_stream",
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    )
}

/// `crash-case:snapshot-creation-stream`：测试文件调的就是公共模块这一个函数。
fn prepared_snapshot_creation_stream() -> PreparedFlow {
    let prepared = common_snapshot_prefix::prepare_the_snapshot_crash_flow(
        "e184-snapshot-creation",
        common_snapshot_prefix::SnapshotCrashFlow::SnapshotCreation,
    );
    PreparedFlow {
        versions_without_a_file: prepared.versions_without_a_file,
        ..smoke_flow(
            "snapshot_creation_stream",
            prepared.base,
            prepared.writes,
            prepared.segments,
            prepared.judged_root_index,
            prepared.versions_with_a_file,
        )
    }
}

/// 覆盖写：把现行那一版的内容换成 `content`，`units` 是这份内容占几个数据单元。
/// 一个数据单元走 `publish_overwrite`（与 `common_crash_points::overwrite_step` 同一个发布调用，只是不设崩溃点）；
/// 多个数据单元走 `common_crash_points::sequential_write`（`publish_sequential_write`，录崩溃点那一路的 `sequential_write_step` 调的也是它）：
/// `publish_overwrite` 的契约是一个数据单元（`crates/singlefs-core/src/transaction.rs` 的 `publish_overwrite` 文档注释「契约是一个数据单元」，
/// 装不进报 `ContentExceedsDataUnit`），多个数据单元的整份内容替换在 crates 里只有顺序写这一个入口（登记第十二节修订 6）。
fn overwrite_the_first_file(
    pool: &mut common::BuiltPool,
    content: &[u8],
    units: usize,
) -> TransactionOutput {
    if units == 1 {
        let publish_parameters = common::parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.output,
            FirstFile {
                content,
                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect("覆盖写")
    } else {
        common_crash_points::sequential_write(pool, content)
    }
}

/// mkfs 之后录制流里的写数（屏障不算）。
fn writes_after_the_make_filesystem(pool: &common::BuiltPool) -> usize {
    pool.retained_operations()[pool.mkfs_operation_count..]
        .iter()
        .filter(|operation| operation.operation.kind != RecordedOperationKind::Barrier)
        .count()
}

/// 臂 F：标准前缀（取号、暖机、第一个文件 A）之后对第一个文件逐次覆盖写，第 k 次的内容是 `content_needing(units, k)`，写数到 1000 为止。
/// 一个数据单元的一档走 `publish_overwrite`，多个数据单元的两档走顺序写（原因见 [`overwrite_the_first_file`]）。交回流与覆盖写的次数。
fn prepared_overwrite_chain(units: usize) -> (PreparedFlow, usize) {
    let mut pool = common::build_pool(&format!(
        "e184-overwrite_chain-{units:02}_units_per_overwrite"
    ));
    let first = pool.output.clone();
    let mut versions = vec![PublishedVersion {
        instance: first.root.instance,
        checkpoint_txg: first.root.checkpoint_txg,
        content: common::file_content(),
    }];
    let mut overwrites = 0usize;
    while writes_after_the_make_filesystem(&pool) < WRITES_OF_A_THOUSAND_WRITE_STREAM {
        overwrites += 1;
        let content = common_crash_points::content_needing(units, overwrites);
        let output = overwrite_the_first_file(&mut pool, &content, units);
        versions.push(PublishedVersion {
            instance: output.root.instance,
            checkpoint_txg: output.root.checkpoint_txg,
            content,
        });
        pool.output = output;
    }
    let base = pool.memory_pool_after_mkfs();
    let (writes, segments) = writes_and_segments(
        &pool.retained_operations()[pool.mkfs_operation_count..],
        &common::geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("至少一条根槽写");
    let flow = PreparedFlow {
        family: "overwrite_chain",
        tier: "candidate",
        label: format!("overwrite_chain-{units:02}_units_per_overwrite"),
        ending: format!("NotAHistory:overwrites={overwrites}"),
        base,
        writes,
        segments,
        judged_root_index,
        versions_with_a_file: versions,
        versions_without_a_file: Vec::new(),
    };
    (flow, overwrites)
}

/// 固定流的种类：覆盖写链按每次覆盖写的数据单元数，冒烟层四条各一个。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FixedFlowKind {
    OverwriteChain { units: usize },
    NewPoolFileCreation,
    FixedScript,
    MultiRecordPublish,
    SnapshotCreation,
}

impl FixedFlowKind {
    /// 覆盖写链三档在前，冒烟层四条在后。
    fn all() -> Vec<Self> {
        OVERWRITE_CHAIN_UNITS_PER_OVERWRITE
            .iter()
            .map(|units| FixedFlowKind::OverwriteChain { units: *units })
            .chain([
                FixedFlowKind::NewPoolFileCreation,
                FixedFlowKind::FixedScript,
                FixedFlowKind::MultiRecordPublish,
                FixedFlowKind::SnapshotCreation,
            ])
            .collect()
    }

    /// 流名，也是转储文件的主干名。
    fn label(self) -> String {
        match self {
            FixedFlowKind::OverwriteChain { units } => {
                format!("overwrite_chain-{units:02}_units_per_overwrite")
            }
            FixedFlowKind::NewPoolFileCreation => "smoke-new_pool_file_creation_stream".to_string(),
            FixedFlowKind::FixedScript => "smoke-overwrite_fixed_script_stream".to_string(),
            FixedFlowKind::MultiRecordPublish => "smoke-multi_record_publish_stream".to_string(),
            FixedFlowKind::SnapshotCreation => "smoke-snapshot_creation_stream".to_string(),
        }
    }

    /// 建出这条流；覆盖写链另交回覆盖写的次数。
    fn build(self) -> (PreparedFlow, Option<usize>) {
        match self {
            FixedFlowKind::OverwriteChain { units } => {
                let (flow, overwrites) = prepared_overwrite_chain(units);
                (flow, Some(overwrites))
            }
            FixedFlowKind::NewPoolFileCreation => (prepared_new_pool_stream(), None),
            FixedFlowKind::FixedScript => (prepared_fixed_script_stream(), None),
            FixedFlowKind::MultiRecordPublish => (prepared_multi_record_stream(), None),
            FixedFlowKind::SnapshotCreation => (prepared_snapshot_creation_stream(), None),
        }
    }
}

/// 登记 5.2「转储往返」在手写流上的形态：直接用建好的池当基镜像排的块键，与写转储、读回、由转储重建基镜像再排的块键逐个相同，
/// 基镜像本身逐扇区相同。整条流态数装不进 u64、或块数上界超过 [`MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP`] 排不了（两份计划要同时摆在内存里）时，只比基镜像。
fn direct_base_control(prepared: &PreparedFlow, dump: &FlowDump) -> Result<ControlReading, String> {
    direct_base_control_under(prepared, dump, MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP)
}

/// [`direct_base_control`]，比块键的块数上界上限由调用方给。
fn direct_base_control_under(
    prepared: &PreparedFlow,
    dump: &FlowDump,
    most_blocks: u64,
) -> Result<ControlReading, String> {
    let read_back = FlowDump::from_bytes(&dump.to_bytes())?;
    let rebuilt_base = read_back.base_pool();
    let bases_equal = rebuilt_base == prepared.base;
    let keys_equal =
        if state_count_to_plan(&prepared.base, dump, Expansion::Full, most_blocks).is_some() {
            let direct = plan_blocks_under(
                &prepared.base,
                dump,
                Expansion::Full,
                default_item(),
                most_blocks,
            )?;
            let through_the_dump = plan_blocks_under(
                &rebuilt_base,
                &read_back,
                Expansion::Full,
                default_item(),
                most_blocks,
            )?;
            Some(direct.blocks == through_the_dump.blocks)
        } else {
            None
        };
    Ok(ControlReading {
        kind: "dump_round_trip_direct_base",
        flow: dump.label.clone(),
        passed: bases_equal && keys_equal != Some(false),
        detail: format!(
            "bases_equal={bases_equal} keys_equal={}",
            keys_equal.map_or_else(|| "not_planned".to_string(), |equal| equal.to_string())
        ),
    })
}

/// `record-fixed-chains`：覆盖写链三档与冒烟层四条各写一份转储；已有写完的转储整份跳过。
fn run_record_fixed_chains_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(arguments, &["--out"], &[]) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let out_directory = match parsed.single("--out") {
        Ok(out) => PathBuf::from(out),
        Err(message) => return usage_error(&message),
    };
    if let Err(error) = std::fs::create_dir_all(&out_directory) {
        return usage_error(&format!("建 {} 失败：{error}", out_directory.display()));
    }
    let mut output = Output::new();
    let (mut recorded, mut skipped, mut failed, mut controls_failed) =
        (0usize, 0usize, 0usize, 0usize);
    for kind in FixedFlowKind::all() {
        let label = kind.label();
        let path = out_directory.join(format!("{label}{DUMP_FILE_SUFFIX}"));
        if let Ok(bytes) = std::fs::read(&path) {
            if FlowDump::from_bytes(&bytes).is_ok() {
                skipped += 1;
                output.emit(&format!(
                    "name=chain_skipped label={label} sha256={}",
                    sha256_hexadecimal(&bytes)
                ));
                continue;
            }
        }
        let started = Instant::now();
        let built = catch_unwind(AssertUnwindSafe(
            || -> Result<(PreparedFlow, Option<usize>, FlowDump, ControlReading), String> {
                let (prepared, overwrites) = kind.build();
                let dump = prepared.dump();
                let control = direct_base_control(&prepared, &dump)?;
                Ok((prepared, overwrites, dump, control))
            },
        ));
        let (prepared, overwrites, dump, control) = match built {
            Err(payload) => {
                failed += 1;
                output.emit(&format!(
                    "name=chain_failed label={label} panic={}",
                    panic_text_of(payload.as_ref())
                ));
                continue;
            }
            Ok(Err(message)) => {
                failed += 1;
                output.emit(&format!("name=chain_failed label={label} reason={message}"));
                continue;
            }
            Ok(Ok(built)) => built,
        };
        let bytes = dump.to_bytes();
        if let Err(message) = write_dump::write_bytes_atomically(&path, &bytes) {
            failed += 1;
            output.emit(&format!("name=chain_failed label={label} reason={message}"));
            continue;
        }
        recorded += 1;
        controls_failed += usize::from(!control.passed);
        output.emit(&format!(
            "name=chain_recorded label={label} family={} overwrites={} writes={} segments={} base_sectors={} sha256={} seconds={:.1}",
            prepared.family,
            overwrites.map_or_else(|| "none".to_string(), |count| count.to_string()),
            dump.writes.len(),
            dump.segments.len(),
            dump.base_writes.len(),
            sha256_hexadecimal(&bytes),
            started.elapsed().as_secs_f64()
        ));
        output.emit(&control.body());
    }
    output.emit_closing_line("chains_done", &format!("recorded={recorded} skipped={skipped} failed={failed} controls_failed={controls_failed}"));
    if failed > 0 {
        EXIT_CODE_STOP_CLAUSE_FAILED
    } else if controls_failed > 0 {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

// ───────────────────────────── selfcheck：停机 T1–T4 与第七节锚点 ─────────────────────────────

/// 这份源文件自己的文本：`COPIED-BEGIN` / `COPIED-END` 之间的行就是照抄的段。
const OWN_SOURCE_TEXT: &str = include_str!("e184_regular_layer_cost_counts.rs");

/// 这个 bin 在 `src/bin/` 下的文件名（`write_dump::preflight` 判它头上的准入与运行条件）。
const OWN_SOURCE_FILE_NAME: &str = "e184_regular_layer_cost_counts.rs";

const FIXED_SCRIPT_TEST_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/crash_enumeration_fixed_script_stream.rs"
));
const MULTI_RECORD_TEST_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/crash_enumeration_multi_record_publish_stream.rs"
));
const NEW_POOL_TEST_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/crash_enumeration_new_pool_file_creation_stream.rs"
));
const SNAPSHOT_TEST_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/crash_enumeration_snapshot_creation_stream.rs"
));

/// 建快照流的段形（测试文件 `SEGMENT_SIZES`，也是登记锚 A1 的输入）。取号写两轮之后开头多一个 2 写的系统配置槽段（取号写一轮时 15 段、62 写）。
const SNAPSHOT_CREATION_SEGMENT_SIZES: [usize; 16] =
    [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2];
/// 建快照流前 12 段（新池新建文件流的段形，锚 A2 的输入；取号写一轮时前 11 段、41 写）。
const NEW_POOL_SEGMENT_SIZES: [usize; 12] = [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];
/// 多记录流的段形（测试文件 `prepare` 里钉的；取号写一轮时 19 段、115 写）。
const MULTI_RECORD_SEGMENT_SIZES: [usize; 20] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2,
];

/// 测试文件钉的常量：两态闭式、全量态数，和有的话甲二快档态数、段形、段数、写数（`None` 是测试文件没钉）。
#[derive(Clone, Copy, Debug)]
struct PinnedByTheTest {
    label: &'static str,
    test_text: &'static str,
    two_state: u64,
    full: u64,
    quick_tier: Option<u64>,
    segment_sizes: Option<&'static [usize]>,
    segment_count: Option<usize>,
    writes: Option<usize>,
}

/// 取号写两轮之后照四份测试文件现钉的数重抄（取号写一轮时依次是 16777240、16777260、46、41 写；1678377089、1678377204、278、78 段；
/// 1358954604、1358954634、115 写；16842782、16842807），`selfcheck` 现录的四条流与它们逐个相等。
const PINNED_BY_THE_TESTS: [PinnedByTheTest; 4] = [
    PinnedByTheTest {
        label: "smoke-new_pool_file_creation_stream",
        test_text: NEW_POOL_TEST_TEXT,
        two_state: 16_777_243,
        full: 16_777_268,
        quick_tier: Some(54),
        segment_sizes: Some(&NEW_POOL_SEGMENT_SIZES),
        segment_count: None,
        writes: Some(43),
    },
    PinnedByTheTest {
        label: "smoke-overwrite_fixed_script_stream",
        test_text: FIXED_SCRIPT_TEST_TEXT,
        two_state: 1_678_377_095,
        full: 1_678_377_220,
        quick_tier: Some(294),
        segment_sizes: None,
        segment_count: Some(80),
        writes: None,
    },
    PinnedByTheTest {
        label: "smoke-multi_record_publish_stream",
        test_text: MULTI_RECORD_TEST_TEXT,
        two_state: 1_358_954_607,
        full: 1_358_954_642,
        quick_tier: None,
        segment_sizes: Some(&MULTI_RECORD_SEGMENT_SIZES),
        segment_count: None,
        writes: Some(117),
    },
    PinnedByTheTest {
        label: "smoke-snapshot_creation_stream",
        test_text: SNAPSHOT_TEST_TEXT,
        two_state: 16_842_785,
        full: 16_842_815,
        quick_tier: None,
        segment_sizes: Some(&SNAPSHOT_CREATION_SEGMENT_SIZES),
        segment_count: None,
        writes: None,
    },
];

/// D13 已定项 9 射程写的「并行线一那条流 12230590578 个状态」（锚 B1）。测试文件头说它是 C577 之前的数，预计与现录的多记录流不符。
const MULTI_RECORD_FULL_STATES_NAMED_BY_THE_CLAUSE: u64 = 12_230_590_578;

/// 一个整数在文本里出现：原样，或按千分位加下划线（Rust 常量的写法）。
fn number_appears_in(text: &str, number: u64) -> bool {
    let digits = number.to_string();
    let mut grouped = String::new();
    for (position, digit) in digits.chars().enumerate() {
        if position > 0 && (digits.len() - position).is_multiple_of(3) {
            grouped.push('_');
        }
        grouped.push(digit);
    }
    text.contains(&digits) || text.contains(&grouped)
}

/// 一条检查的结果：停机条款（`stop_check`）或锚点（`anchor`）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckReading {
    item: String,
    passed: bool,
    detail: String,
}

impl CheckReading {
    fn new(item: &str, passed: bool, detail: String) -> Self {
        Self {
            item: item.to_string(),
            passed,
            detail,
        }
    }

    fn stop_check_body(&self) -> String {
        format!(
            "name=stop_check item={} result={} detail={}",
            self.item,
            if self.passed { "pass" } else { "fail" },
            self.detail.replace(' ', "_")
        )
    }
}

/// 停机 T1：文件头的本地常量与 crates 的同名常量回比。
fn local_constants_checks() -> Vec<CheckReading> {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let closed_form_of = |writes_in_the_segment: usize| {
        catch_unwind(|| closed_form_state_count(&[(0..writes_in_the_segment).collect()]))
    };
    let at_the_limit = closed_form_of(LONGEST_SEGMENT_WITH_A_CLOSED_FORM);
    let one_beyond = closed_form_of(LONGEST_SEGMENT_WITH_A_CLOSED_FORM + 1);
    std::panic::set_hook(previous_hook);
    vec![
        CheckReading::new(
            "local_states_per_block_matches_crates",
            LOCAL_STATES_PER_BLOCK == STATES_PER_BLOCK,
            format!("local={LOCAL_STATES_PER_BLOCK} crates={STATES_PER_BLOCK}"),
        ),
        CheckReading::new(
            "local_seed_base_matches_crates",
            SEED_BASE_OF_THE_RANDOM_HISTORIES == singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            format!("local={SEED_BASE_OF_THE_RANDOM_HISTORIES} crates={}", singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        ),
        CheckReading::new(
            "local_longest_closed_form_segment_matches_crates",
            at_the_limit.is_ok() && one_beyond.is_err() && two_state_closed_form(&[(0..=LONGEST_SEGMENT_WITH_A_CLOSED_FORM).collect()]).is_none(),
            format!(
                "limit={LONGEST_SEGMENT_WITH_A_CLOSED_FORM} crates_at_the_limit_panics={} crates_one_beyond_panics={}",
                at_the_limit.is_err(),
                one_beyond.is_err()
            ),
        ),
    ]
}

/// 照抄段的文本：`COPIED-BEGIN <名> <出处>` 与 `COPIED-END <名>` 之间的行，去掉包进模块的 4 格缩进，用换行接起来。
///
/// # Errors
/// 开始与结束对不上；段里有一行缩进不够 4 格。
fn copied_segments_of(source: &str) -> Result<Vec<(String, String, String)>, String> {
    let mut segments = Vec::new();
    let mut open: Option<(String, String, Vec<String>)> = None;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("// COPIED-BEGIN ") {
            let (name, origin) = rest
                .split_once(' ')
                .ok_or_else(|| format!("COPIED-BEGIN 行缺出处：{line}"))?;
            if open.is_some() {
                return Err(format!("{name} 开始之前上一段还没结束"));
            }
            open = Some((name.to_string(), origin.to_string(), Vec::new()));
        } else if let Some(name) = trimmed.strip_prefix("// COPIED-END ") {
            let (opened, origin, lines) = open
                .take()
                .ok_or_else(|| format!("{name} 结束了，可是没有开始"))?;
            if opened != name {
                return Err(format!("{opened} 开始、{name} 结束，对不上"));
            }
            segments.push((opened, origin, lines.join("\n")));
        } else if let Some((_, _, lines)) = open.as_mut() {
            lines.push(if line.is_empty() {
                String::new()
            } else {
                line.strip_prefix("    ")
                    .ok_or_else(|| format!("照抄的段里这一行缩进不够 4 格：{line}"))?
                    .to_string()
            });
        }
    }
    match open {
        Some((name, _, _)) => Err(format!("{name} 开始了，没有结束")),
        None => Ok(segments),
    }
}

/// 停机 T2：照抄的每一段在原文件里整段逐字找得到。出处里的文件名决定去哪份原文里找。
fn copied_text_checks(source: &str) -> Vec<CheckReading> {
    let originals = [
        (
            "crash_enumeration_fixed_script_stream.rs",
            FIXED_SCRIPT_TEST_TEXT,
        ),
        (
            "crash_enumeration_multi_record_publish_stream.rs",
            MULTI_RECORD_TEST_TEXT,
        ),
        (
            "crash_enumeration_new_pool_file_creation_stream.rs",
            NEW_POOL_TEST_TEXT,
        ),
    ];
    let segments = match copied_segments_of(source) {
        Ok(segments) => segments,
        Err(message) => return vec![CheckReading::new("copied_text_verbatim", false, message)],
    };
    let mut readings: Vec<CheckReading> = segments
        .iter()
        .map(|(name, origin, text)| {
            let original = originals
                .iter()
                .find(|(file, _)| origin.contains(file))
                .map(|(_, original)| *original);
            let found = original.is_some_and(|original| original.contains(text.as_str()));
            CheckReading::new(
                &format!("copied_text_verbatim_{name}"),
                found && !text.is_empty(),
                format!("origin={origin} lines={}", text.lines().count()),
            )
        })
        .collect();
    readings.push(CheckReading::new(
        "copied_segments_found",
        segments.len() == EXPECTED_COPIED_SEGMENTS,
        format!(
            "segments={} expected={EXPECTED_COPIED_SEGMENTS}",
            segments.len()
        ),
    ));
    readings
}

/// 源文件里应有的照抄段数：固定脚本两段、多记录一段、新池新建文件一段。
const EXPECTED_COPIED_SEGMENTS: usize = 4;

/// 每段写数的列表换成段（段里放 0..写数 的下标，闭式只看长度）。
fn segments_of_sizes(sizes: &[usize]) -> Vec<Vec<usize>> {
    sizes.iter().map(|size| (0..*size).collect()).collect()
}

/// 每个点的块数等于按本地块长算的块数（锚 A3 在真实计划上的形态）：一点 65537 态切 2 块、65536 态切 1 块靠 [`blocks_needed_for`]。
fn every_point_is_cut_into_the_blocks_the_local_block_length_gives(
    blocks: &[BlockOfAPlan],
) -> bool {
    let mut by_point: BTreeMap<usize, (u64, u64)> = BTreeMap::new();
    for block in blocks {
        let entry = by_point.entry(block.point_index).or_default();
        entry.0 += 1;
        entry.1 += block.states;
    }
    by_point
        .values()
        .all(|(block_count, states)| *block_count == blocks_needed_for(*states))
}

/// 一条冒烟层流重录的值与测试文件钉的常量回比（停机 T3、T4，锚 A5、A6、B2），另核钉的数还在测试文件里。
fn recorded_flow_readings(prepared: &PreparedFlow, pinned: &PinnedByTheTest) -> Vec<CheckReading> {
    let dump = prepared.dump();
    let reading = match read_the_flow(&dump, &prepared.base) {
        Ok(reading) => reading,
        Err(message) => {
            return vec![CheckReading::new(
                &format!("recorded_values_match_the_test_pins_{}", prepared.label),
                false,
                message,
            )]
        }
    };
    let row = &reading.row;
    let mut mismatches = Vec::new();
    if pinned.label != prepared.label {
        mismatches.push(format!(
            "pinned_for_{}_but_recorded_{}",
            pinned.label, prepared.label
        ));
    }
    for (name, recorded, pin) in [
        ("two_state", row.two_state, Some(pinned.two_state)),
        ("full", row.full, Some(pinned.full)),
        ("quick_tier", row.quick_tier, pinned.quick_tier),
    ] {
        if let Some(pin) = pin.filter(|pin| Some(*pin) != recorded) {
            mismatches.push(format!("{name}_recorded={recorded:?}_pinned={pin}"));
        }
    }
    let sizes: Vec<usize> = dump.segments.iter().map(Vec::len).collect();
    if pinned
        .segment_sizes
        .is_some_and(|pin| pin != sizes.as_slice())
    {
        mismatches.push(format!("segment_sizes_recorded={sizes:?}"));
    }
    if pinned
        .segment_count
        .is_some_and(|count| count != sizes.len())
    {
        mismatches.push(format!("segment_count_recorded={}", sizes.len()));
    }
    if pinned
        .writes
        .is_some_and(|writes| writes != dump.writes.len())
    {
        mismatches.push(format!("writes_recorded={}", dump.writes.len()));
    }
    let pins_in_the_test_file = [Some(pinned.two_state), Some(pinned.full), pinned.quick_tier]
        .into_iter()
        .flatten()
        .all(|number| number_appears_in(pinned.test_text, number));
    let full_plan = reading.full.as_ref();
    vec![
        CheckReading::new(
            &format!("recorded_values_match_the_test_pins_{}", prepared.label),
            mismatches.is_empty(),
            format!("mismatches=[{}]", mismatches.join(",")),
        ),
        CheckReading::new(
            &format!("pins_still_in_the_test_file_{}", prepared.label),
            pins_in_the_test_file,
            "two_state_full_quick_tier".to_string(),
        ),
        CheckReading::new(
            &format!("two_counting_routes_agree_{}", prepared.label),
            reading.count_disagreements.is_empty(),
            format!("disagreements=[{}]", reading.count_disagreements.join(";")),
        ),
        CheckReading::new(
            &format!("two_state_not_above_full_{}", prepared.label),
            matches!((row.two_state, row.full), (Some(two_state), Some(full)) if two_state <= full),
            format!("two_state={:?} full={:?}", row.two_state, row.full),
        ),
        CheckReading::new(
            &format!(
                "blocks_per_point_follow_the_local_block_length_{}",
                prepared.label
            ),
            full_plan.is_some_and(|plan| {
                every_point_is_cut_into_the_blocks_the_local_block_length_gives(&plan.blocks)
            }),
            format!("blocks_full={:?}", row.blocks_full),
        ),
    ]
}

/// 锚点（登记 7.2 的 A1、A2、A4 与 7.1 的 B1）：一行一条，写预期、实测与对不对得上。
fn anchor_body(identifier: &str, expected: &str, actual: &str) -> (String, bool) {
    let matched = expected == actual;
    (
        format!("name=anchor identifier={identifier} expected={expected} actual={actual} match={matched}"),
        matched,
    )
}

/// `selfcheck`：停机 T1–T4 与第七节锚点逐条核。有一条停机检查没过或 A 开头的锚点对不上退 3；B1 对不上是登记预计的（C577 改了段形），不在其内。
fn run_selfcheck_command(arguments: &[String]) -> i32 {
    if !arguments.is_empty() {
        return usage_error("selfcheck 不带参数");
    }
    let mut output = Output::new();
    let mut readings = local_constants_checks();
    readings.extend(copied_text_checks(OWN_SOURCE_TEXT));
    let mut anchors = vec![
        anchor_body(
            "snapshot_creation_two_state",
            "16842785",
            &count_field(two_state_closed_form(&segments_of_sizes(
                &SNAPSHOT_CREATION_SEGMENT_SIZES,
            ))),
        ),
        anchor_body(
            "new_pool_two_state",
            "16777243",
            &count_field(two_state_closed_form(&segments_of_sizes(
                &NEW_POOL_SEGMENT_SIZES,
            ))),
        ),
        anchor_body(
            "new_pool_writes",
            "43",
            &NEW_POOL_SEGMENT_SIZES.iter().sum::<usize>().to_string(),
        ),
        anchor_body(
            "blocks_needed_at_and_one_state_beyond_the_states_per_block",
            "1,2",
            &format!(
                "{},{}",
                blocks_needed_for(LOCAL_STATES_PER_BLOCK),
                blocks_needed_for(LOCAL_STATES_PER_BLOCK + 1)
            ),
        ),
        anchor_body(
            "first_history_seed",
            &SEED_BASE_OF_THE_RANDOM_HISTORIES.to_string(),
            &write_dump::RandomHistoryRequest {
                steps: 1,
                weights: write_dump::WeightsChoice::Broad,
                width: singlefs_harness::history::HistoryDeviceWidth::FourGibibytes,
                seed_index: 0,
            }
            .seed()
            .to_string(),
        ),
    ];
    let mut multi_record_full = None;
    for (kind, pinned) in [
        (FixedFlowKind::NewPoolFileCreation, &PINNED_BY_THE_TESTS[0]),
        (FixedFlowKind::FixedScript, &PINNED_BY_THE_TESTS[1]),
        (FixedFlowKind::MultiRecordPublish, &PINNED_BY_THE_TESTS[2]),
        (FixedFlowKind::SnapshotCreation, &PINNED_BY_THE_TESTS[3]),
    ] {
        let label = kind.label();
        match catch_unwind(AssertUnwindSafe(|| kind.build().0)) {
            Err(payload) => readings.push(CheckReading::new(
                &format!("recorded_values_match_the_test_pins_{label}"),
                false,
                format!("建流时 panic：{}", panic_text_of(payload.as_ref())),
            )),
            Ok(prepared) => {
                let recorded = catch_unwind(AssertUnwindSafe(|| {
                    recorded_flow_readings(&prepared, pinned)
                }));
                match recorded {
                    Err(payload) => readings.push(CheckReading::new(
                        &format!("two_counting_routes_agree_{label}"),
                        false,
                        format!("排计划时 panic：{}", panic_text_of(payload.as_ref())),
                    )),
                    Ok(flow_readings) => readings.extend(flow_readings),
                }
                let dump = prepared.dump();
                let full = state_count_of_the_flow(&prepared.base, &dump, Expansion::Full);
                anchors.push(anchor_body(
                    &format!("full_states_{label}"),
                    &pinned.full.to_string(),
                    &count_field(full),
                ));
                if kind == FixedFlowKind::MultiRecordPublish {
                    multi_record_full = full;
                }
            }
        }
    }
    let (clause_anchor_body, clause_anchor_matched) = anchor_body(
        "multi_record_full_named_by_the_clause",
        &MULTI_RECORD_FULL_STATES_NAMED_BY_THE_CLAUSE.to_string(),
        &count_field(multi_record_full),
    );
    for reading in &readings {
        output.emit(&reading.stop_check_body());
    }
    let anchors_mismatching = anchors.iter().filter(|(_, matched)| !matched).count();
    for (body, _matched) in anchors
        .iter()
        .chain([&(clause_anchor_body, clause_anchor_matched)])
    {
        output.emit(body);
    }
    let failed = readings.iter().filter(|reading| !reading.passed).count();
    output.emit_closing_line(
        "selfcheck_done",
        &format!("stop_checks={} failed={failed} anchors={} anchors_mismatching={anchors_mismatching} clause_anchor_matches={clause_anchor_matched}", readings.len(), anchors.len() + 1),
    );
    if failed > 0 || anchors_mismatching > 0 {
        EXIT_CODE_STOP_CLAUSE_FAILED
    } else {
        EXIT_CODE_OK
    }
}

fn main() {
    let forced_summary = write_dump::preflight(OWN_SOURCE_FILE_NAME);
    if let Some(summary) = forced_summary {
        println!("E7RESULT name=preflight forced={summary}");
    }
    let arguments =
        write_dump::arguments_without_the_force_flag(std::env::args().skip(1).collect());
    let exit_code = match arguments.split_first() {
        Some((subcommand, rest)) => match subcommand.as_str() {
            "selfcheck" => run_selfcheck_command(rest),
            "record-fixed-chains" => run_record_fixed_chains_command(rest),
            "keys" => run_keys_command(rest),
            "share" => run_share_command(rest),
            "library" => run_library_command(rest),
            "compare" => run_compare_command(rest),
            "compose" => run_compose_command(rest),
            other => usage_error(&format!("认不出的子命令 {other}")),
        },
        None => usage_error(
            "缺子命令：selfcheck、record-fixed-chains、keys、share、library、compare、compose 之一",
        ),
    };
    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_of(reuse: u8, plan: u8, start: u64) -> VerdictBlockKey {
        VerdictBlockKey {
            node_input_digest: NodeInputDigest([reuse; 32]),
            enumeration_plan_hash: EnumerationPlanHash([plan; 32]),
            block_start: BlockStartStateIndex(start),
        }
    }

    fn block_of(reuse: u8, plan: u8, start: u64, states: u64, point_index: usize) -> BlockOfAPlan {
        BlockOfAPlan {
            key: key_of(reuse, plan, start),
            states,
            point_index,
        }
    }

    fn row_of(
        label: &str,
        family: &str,
        tier: &str,
        writes: usize,
        two_state: Option<u64>,
        longest_segment: usize,
    ) -> FlowRow {
        FlowRow {
            family: family.to_string(),
            tier: tier.to_string(),
            label: label.to_string(),
            seed: None,
            ending: "Completed".to_string(),
            writes,
            segments: 3,
            longest_segment,
            two_state,
            full: two_state,
            quick_tier: two_state,
            blocks_full: Some(3),
            blocks_quick: Some(3),
            points: Some(2),
            keys_seconds: 1.5,
            two_state_agrees_with_crates: two_state.map(|_| true),
        }
    }

    fn keys_file_of(row: FlowRow, blocks: Vec<BlockOfAPlan>) -> KeysFile {
        KeysFile {
            expansion: Expansion::Full,
            row,
            blocks,
        }
    }

    #[test]
    fn the_two_state_closed_form_gives_the_pinned_numbers_of_the_snapshot_creation_and_new_pool_shapes(
    ) {
        assert_eq!(
            two_state_closed_form(&segments_of_sizes(&SNAPSHOT_CREATION_SEGMENT_SIZES)),
            Some(16_842_785)
        );
        assert_eq!(
            two_state_closed_form(&segments_of_sizes(&NEW_POOL_SEGMENT_SIZES)),
            Some(16_777_243)
        );
        assert_eq!(NEW_POOL_SEGMENT_SIZES.iter().sum::<usize>(), 43);
        assert_eq!(
            two_state_closed_form(&segments_of_sizes(&[1, 2, 3])),
            Some(1 + 1 + 3 + 7),
            "手算：1 + (2 − 1) + (4 − 1) + (8 − 1)"
        );
        assert_eq!(
            two_state_closed_form(&segments_of_sizes(&[1, 2, 3, 24])),
            Some(closed_form_state_count(&segments_of_sizes(&[1, 2, 3, 24]))),
            "本地复算与 crates 的闭式函数在小段形上相等"
        );
    }

    #[test]
    fn sixty_four_writes_in_one_segment_have_no_closed_form_and_a_sum_past_u64_has_none_either() {
        assert_eq!(two_state_closed_form(&segments_of_sizes(&[64])), None);
        assert_eq!(
            two_state_closed_form(&segments_of_sizes(&[63])),
            Some(1u64 << 63)
        );
        assert_eq!(
            two_state_closed_form(&segments_of_sizes(&[63, 63])),
            Some(u64::MAX)
        );
        assert_eq!(
            two_state_closed_form(&segments_of_sizes(&[63, 63, 63])),
            None
        );
        assert_eq!(LONGEST_SEGMENT_WITH_A_CLOSED_FORM, 63);
    }

    #[test]
    fn the_block_count_cuts_at_the_local_block_length() {
        let cases: [(u64, u64); 7] = [
            (0, 0),
            (1, 1),
            (65_535, 1),
            (65_536, 1),
            (65_537, 2),
            (131_072, 2),
            (131_073, 3),
        ];
        for (states, blocks) in cases {
            assert_eq!(blocks_needed_for(states), blocks, "{states} 个状态");
        }
        assert_eq!(
            LOCAL_STATES_PER_BLOCK, STATES_PER_BLOCK,
            "本地块长与 crates 的块长相等（停机 T1）"
        );
    }

    #[test]
    fn the_longest_segment_is_the_largest_one_and_not_the_first() {
        assert_eq!(
            longest_segment_of(&segments_of_sizes(&SNAPSHOT_CREATION_SEGMENT_SIZES)),
            24
        );
        assert_eq!(longest_segment_of(&segments_of_sizes(&[3, 1, 2])), 3);
        assert_eq!(longest_segment_of(&segments_of_sizes(&[1, 5, 2])), 5);
        assert_eq!(longest_segment_of(&[]), 0);
    }

    #[test]
    fn the_share_verdict_has_inclusive_thresholds_and_a_partial_band() {
        assert_eq!(share_verdict(0.5, 1.0), ShareVerdict::Renamed);
        assert_eq!(share_verdict(0.49, 0.9), ShareVerdict::Partial);
        assert_eq!(share_verdict(0.0, 0.05), ShareVerdict::Independent);
        assert_eq!(share_verdict(0.0, 0.06), ShareVerdict::Partial);
        assert_eq!(share_verdict(0.2, 0.03), ShareVerdict::Independent);
    }

    #[test]
    fn the_overlap_verdict_has_inclusive_thresholds_and_a_partial_band() {
        assert_eq!(overlap_verdict(0.5), OverlapVerdict::Overlap);
        assert_eq!(overlap_verdict(0.1), OverlapVerdict::Separate);
        assert_eq!(overlap_verdict(0.3), OverlapVerdict::Partial);
        assert_eq!(overlap_verdict(0.0), OverlapVerdict::Separate);
        assert_eq!(overlap_verdict(1.0), OverlapVerdict::Overlap);
    }

    #[test]
    fn the_interval_of_the_smoke_layer_counts_both_ends_as_inside() {
        assert_eq!(
            place_against(Some(100), 100, 200),
            PlaceAgainstInterval::Inside,
            "恰好等于最小值算在里面"
        );
        assert_eq!(
            place_against(Some(200), 100, 200),
            PlaceAgainstInterval::Inside,
            "恰好等于最大值算在里面"
        );
        assert_eq!(
            place_against(Some(99), 100, 200),
            PlaceAgainstInterval::Below
        );
        assert_eq!(
            place_against(Some(201), 100, 200),
            PlaceAgainstInterval::Above
        );
        assert_eq!(
            place_against(None, 100, 200),
            PlaceAgainstInterval::Above,
            "装不进 u64 的比区间上端大"
        );
    }

    #[test]
    fn candidate_blocks_all_inside_a_larger_comparison_set_share_all_of_them() {
        let candidate = vec![
            block_of(1, 1, 0, 10, 0),
            block_of(2, 2, 0, 20, 1),
            block_of(3, 3, 0, 30, 2),
        ];
        let mut against_blocks = candidate.clone();
        against_blocks.push(block_of(4, 4, 0, 5, 3));
        against_blocks.push(block_of(5, 5, 0, 5, 4));
        let against = block_keys_of(&against_blocks);
        assert_eq!(against.len(), 5);
        assert!(
            (block_share_fraction(&candidate, &against) - 1.0).abs() < f64::EPSILON,
            "分母是候选自己的块数 3，不是比较集的 5"
        );
        let partly = vec![
            block_of(1, 1, 0, 10, 0),
            block_of(9, 9, 0, 30, 1),
            block_of(8, 8, 0, 30, 2),
            block_of(7, 7, 0, 30, 3),
        ];
        assert!((block_share_fraction(&partly, &against) - 0.25).abs() < f64::EPSILON);
        assert!(
            (weighted_share_fraction(&partly, &against) - 10.0 / 100.0).abs() < f64::EPSILON,
            "共用块里的态数 10 ÷ 候选的态数 100"
        );
    }

    #[test]
    fn blocks_with_the_same_reuse_key_and_another_plan_hash_are_not_shared_but_count_for_the_reuse_key_share(
    ) {
        let plan_with_the_original_hash =
            vec![block_of(1, 1, 0, 10, 0), block_of(1, 1, 65_536, 10, 0)];
        let other_plan = vec![block_of(1, 2, 0, 10, 0)];
        let against = block_keys_of(&plan_with_the_original_hash);
        assert_eq!(
            shared_blocks_and_states(&other_plan, &against),
            (0, 0),
            "计划哈希不同的块键不同，哪怕复用键相同"
        );
        let against_reuse = reuse_keys_of(&plan_with_the_original_hash);
        assert!(
            (reuse_key_share_fraction(&other_plan, &against_reuse) - 1.0).abs() < f64::EPSILON,
            "复用键是块键共用的上界"
        );
        let same_start_other_reuse = vec![block_of(2, 1, 0, 10, 0)];
        assert_eq!(
            shared_blocks_and_states(&same_start_other_reuse, &against),
            (0, 0)
        );
    }

    #[test]
    fn the_invalidated_blocks_are_the_after_blocks_missing_in_the_before_flow() {
        let before = vec![
            block_of(1, 1, 0, 10, 0),
            block_of(2, 2, 0, 20, 1),
            block_of(3, 3, 0, 30, 2),
            block_of(4, 4, 0, 40, 3),
        ];
        let after = vec![
            block_of(1, 1, 0, 10, 0),
            block_of(2, 2, 0, 20, 1),
            block_of(3, 3, 0, 30, 2),
            block_of(5, 5, 0, 50, 3),
            block_of(6, 6, 0, 60, 4),
            block_of(7, 7, 0, 70, 5),
        ];
        let invalidated = invalidated_blocks(&before, &after);
        assert_eq!(invalidated.len(), 3, "after 的 6 块里有 3 块 before 没有");
        assert_eq!(
            invalidated.iter().map(|block| block.states).sum::<u64>(),
            180
        );
        assert_eq!(
            invalidated_blocks(&after, &before).len(),
            1,
            "反过来数只有 1 块（before 里的第四块）：方向不能弄反"
        );
    }

    #[test]
    fn the_flow_row_and_the_keys_file_read_back_what_was_written() {
        let mut row = row_of(
            "random_history-broad-FourGibibytes-steps0100-seed03",
            "random_history",
            "candidate",
            2262,
            Some(1 << 30),
            30,
        );
        row.seed = Some(SEED_BASE_OF_THE_RANDOM_HISTORIES + 3);
        row.ending = "KnownRed:form=7".to_string();
        assert_eq!(FlowRow::from_body(&row.body()), Ok(row.clone()));
        let beyond = FlowRow {
            two_state: None,
            full: None,
            quick_tier: None,
            blocks_full: None,
            blocks_quick: None,
            points: None,
            two_state_agrees_with_crates: None,
            ..row.clone()
        };
        assert_eq!(FlowRow::from_body(&beyond.body()), Ok(beyond.clone()));
        assert!(beyond
            .body()
            .contains("two_state=beyond_u64 log2_two_state=beyond_u64"));
        assert_eq!(
            beyond.group(),
            "random_history-broad-FourGibibytes-steps0100"
        );
        let blocks = vec![
            block_of(1, 2, 0, 65_536, 0),
            block_of(1, 2, 65_536, 7, 0),
            block_of(9, 8, 0, 1, 1),
        ];
        let file = KeysFile {
            expansion: Expansion::QuickTier,
            row,
            blocks,
        };
        let text = keys_file_text(&file.row, file.expansion, &file.blocks);
        assert_eq!(KeysFile::from_text(&text), Ok(file.clone()));
        assert_eq!(file.state_total(), 65_544);
        let without_trailer = text
            .lines()
            .take(text.lines().count() - 1)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(KeysFile::from_text(&without_trailer)
            .expect_err("没写完")
            .contains("keys_done"));
        let wrong_count = text.replace("# keys_done blocks=3", "# keys_done blocks=4");
        assert!(KeysFile::from_text(&wrong_count)
            .expect_err("块数对不上")
            .contains("实际读到 3"));
    }

    #[test]
    fn the_group_of_a_label_drops_only_the_seed_suffix() {
        assert_eq!(
            group_of_the_label("random_history-broad-FourGibibytes-steps0100-seed07"),
            "random_history-broad-FourGibibytes-steps0100"
        );
        assert_eq!(
            group_of_the_label("overwrite_chain-04_units_per_overwrite"),
            "overwrite_chain-04_units_per_overwrite"
        );
        assert_eq!(
            group_of_the_label("smoke-snapshot_creation_stream"),
            "smoke-snapshot_creation_stream"
        );
        assert_eq!(group_of_the_label("odd-seedx"), "odd-seedx");
    }

    #[test]
    fn the_spread_takes_the_middle_value_or_the_mean_of_the_two_middle_ones() {
        assert_eq!(Spread::of(&[]), None);
        assert_eq!(
            Spread::of(&[5.0]),
            Some(Spread {
                minimum: 5.0,
                median: 5.0,
                maximum: 5.0
            })
        );
        assert_eq!(
            Spread::of(&[3.0, 1.0, 2.0]),
            Some(Spread {
                minimum: 1.0,
                median: 2.0,
                maximum: 3.0
            })
        );
        assert_eq!(
            Spread::of(&[4.0, 1.0, 3.0, 2.0]),
            Some(Spread {
                minimum: 1.0,
                median: 2.5,
                maximum: 4.0
            })
        );
        assert_eq!(Spread::text(None), "minimum=none median=none maximum=none");
    }

    #[test]
    fn the_options_accept_repeated_values_and_flags_and_refuse_unknown_ones() {
        let words = |list: &[&str]| -> Vec<String> {
            list.iter().map(|word| (*word).to_string()).collect()
        };
        let parsed = ParsedOptions::parse(
            &words(&["--dump", "/a", "--dump", "/b", "--out", "/o", "--controls"]),
            &["--dump", "--out"],
            &["--controls"],
        )
        .expect("合法");
        assert_eq!(
            parsed.paths("--dump"),
            Ok(vec![PathBuf::from("/a"), PathBuf::from("/b")])
        );
        assert_eq!(parsed.single("--out"), Ok("/o"));
        assert!(parsed.flag("--controls"));
        assert!(
            parsed.single("--dump").is_err(),
            "--dump 给了两次，single 要拒"
        );
        assert!(ParsedOptions::parse(&words(&["--out"]), &["--out"], &[])
            .expect_err("缺值")
            .contains("缺值"));
        assert!(
            ParsedOptions::parse(&words(&["--nonsense"]), &["--out"], &[])
                .expect_err("认不出")
                .contains("--nonsense")
        );
        assert!(parsed.paths("--missing").is_err());
    }

    fn line_with<'lines>(output: &'lines Output, prefix: &str) -> &'lines str {
        output
            .lines
            .iter()
            .find(|line| line.starts_with(prefix))
            .unwrap_or_else(|| panic!("没有以 {prefix:?} 起头的行：{:?}", output.lines))
    }

    /// 比较集（冒烟层一条流，三块）和五条候选流：三条随机历史（共用 3 / 4 块、1 / 4 块、0 / 5 块）、一条随机历史写数不到 1000、一条覆盖写链。
    fn share_fixture() -> (Vec<KeysFile>, Vec<KeysFile>) {
        let smoke = keys_file_of(
            row_of(
                "smoke-snapshot_creation_stream",
                "smoke",
                "baseline",
                62,
                Some(100),
                10,
            ),
            vec![
                block_of(1, 1, 0, 10, 0),
                block_of(2, 2, 0, 20, 1),
                block_of(3, 3, 0, 30, 2),
            ],
        );
        let history = |seed: u8, writes: usize, two_state: u64, blocks: Vec<BlockOfAPlan>| {
            keys_file_of(
                row_of(
                    &format!("random_history-broad-FourGibibytes-steps0100-seed{seed:02}"),
                    "random_history",
                    "candidate",
                    writes,
                    Some(two_state),
                    20,
                ),
                blocks,
            )
        };
        let candidates = vec![
            history(
                0,
                1500,
                50,
                vec![
                    block_of(1, 1, 0, 10, 0),
                    block_of(2, 2, 0, 20, 1),
                    block_of(3, 3, 0, 30, 2),
                ],
            ),
            history(
                1,
                1500,
                100,
                vec![
                    block_of(1, 1, 0, 10, 0),
                    block_of(11, 11, 0, 10, 1),
                    block_of(12, 12, 0, 10, 2),
                    block_of(13, 13, 0, 10, 3),
                ],
            ),
            history(
                2,
                1500,
                5000,
                (0..5)
                    .map(|index| block_of(20 + index, 20 + index, 0, 10, usize::from(index)))
                    .collect(),
            ),
            history(3, 10, 5000, vec![block_of(30, 30, 0, 10, 0)]),
            keys_file_of(
                row_of(
                    "overwrite_chain-04_units_per_overwrite",
                    "overwrite_chain",
                    "candidate",
                    1200,
                    Some(70),
                    28,
                ),
                vec![block_of(40, 40, 0, 10, 0), block_of(41, 41, 0, 10, 1)],
            ),
        ];
        (candidates, vec![smoke])
    }

    #[test]
    fn the_share_lines_report_each_flow_each_group_and_the_overlap_with_the_smoke_interval() {
        let (candidates, against) = share_fixture();
        let mut output = Output::new();
        assert_eq!(
            share_lines(
                candidates.iter().cloned().map(Ok),
                &against,
                &[],
                &mut output
            ),
            Ok(EXIT_CODE_OK)
        );
        let group = line_with(
            &output,
            "E7RESULT name=share_group scope=group:random_history-broad-FourGibibytes-steps0100 ",
        );
        assert!(group.contains("flows=4 planned_flows=4"), "{group}");
        assert!(
            group.contains("median_share_fraction=0.125000 maximum_share_fraction=1.000000"),
            "四条流的占比 1、0.25、0、0：中位 0.125，最大 1：{group}"
        );
        assert!(group.ends_with("verdict=partial"), "{group}");
        let chain = line_with(
            &output,
            "E7RESULT name=share_group scope=family:overwrite_chain_main ",
        );
        assert!(
            chain.contains("median_share_fraction=0.000000 maximum_share_fraction=0.000000")
                && chain.ends_with("verdict=independent"),
            "{chain}"
        );
        let second = line_with(
            &output,
            "E7RESULT name=share flow=random_history-broad-FourGibibytes-steps0100-seed01 ",
        );
        assert!(second.contains("blocks=4 shared_blocks=1 share_fraction=0.250000 states=40 shared_states=10 weighted_fraction=0.250000 reuse_fraction=0.250000"), "{second}");
        let overlap = line_with(
            &output,
            "E7RESULT name=overlap scope=family:random_history_main ",
        );
        assert!(overlap.contains("smoke_interval_two_state_minimum=100 smoke_interval_two_state_maximum=100 smoke_interval_longest_segment_minimum=10 smoke_interval_longest_segment_maximum=10"), "{overlap}");
        assert!(
            overlap.contains("flows=4 flows_with_writes_at_least_1000=3 inside=1 above=1 below=1"),
            "{overlap}"
        );
        assert!(
            overlap.contains("inside_fraction=0.333333") && overlap.ends_with("verdict=partial"),
            "{overlap}"
        );
        assert!(
            !output
                .lines
                .iter()
                .any(|line| line.contains("passed=false")),
            "自比对照每条都过"
        );
        let closing = output.lines.last().expect("有收尾行");
        assert!(
            closing.contains("name=share_done candidates=5 against=1 controls_failed=0"),
            "{closing}"
        );
    }

    #[test]
    fn the_overlap_counts_use_only_flows_with_a_thousand_writes_and_put_the_boundary_inside() {
        let interval = SmokeInterval {
            two_state_minimum: 100,
            two_state_maximum: 200,
            longest_segment_minimum: 10,
            longest_segment_maximum: 20,
        };
        let rows = [
            row_of("a", "random_history", "candidate", 1000, Some(100), 10),
            row_of("b", "random_history", "candidate", 999, Some(100), 10),
            row_of("c", "random_history", "candidate", 5000, None, 64),
            row_of("d", "random_history", "candidate", 5000, Some(200), 21),
            row_of("e", "random_history", "candidate", 5000, Some(99), 9),
        ];
        let members: Vec<&FlowRow> = rows.iter().collect();
        let counts = OverlapCounts::of(&members, &interval);
        assert_eq!(
            counts,
            OverlapCounts {
                flows: 5,
                flows_with_writes_at_least_1000: 4,
                inside: 2,
                above: 1,
                below: 1,
                longest_segment_inside: 1
            },
            "a 恰好 1000 写算进去，b 差 1 写不算；c 的态数装不进 u64 算在上面"
        );
        assert_eq!(counts.fraction_of(counts.inside), Some(0.5));
        assert_eq!(
            OverlapCounts::of(&[&rows[1]], &interval).fraction_of(0),
            None,
            "没有写数不少于 1000 的流时占比读不出，不当 0"
        );
        assert!(OverlapCounts::of(&[&rows[1]], &interval)
            .body("scope", &interval)
            .ends_with("verdict=no_flow_with_writes_at_least_1000"));
    }

    #[test]
    fn the_smoke_interval_takes_the_recorded_flows_and_the_pinned_ones() {
        let recorded = [
            row_of("s1", "smoke", "baseline", 62, Some(16_842_782), 24),
            row_of("s2", "smoke", "baseline", 41, Some(16_777_240), 24),
        ];
        let pinned = parse_pinned_smoke(
            "# 注释\nlifecycle_closure 288751838 26\n\ncompaction_relocation 1377370253 30\n",
        )
        .expect("合法");
        assert_eq!(pinned.len(), 2);
        assert_eq!(
            smoke_interval(&recorded, &pinned),
            Ok(SmokeInterval {
                two_state_minimum: 16_777_240,
                two_state_maximum: 1_377_370_253,
                longest_segment_minimum: 24,
                longest_segment_maximum: 30
            })
        );
        assert!(
            smoke_interval(&[], &[]).is_err(),
            "一条冒烟流都没有，区间定不出"
        );
        assert!(parse_pinned_smoke("only_two_words 12").is_err());
    }

    #[test]
    fn the_wall_clock_band_puts_the_two_boundaries_in_the_lower_band() {
        assert_eq!(
            band_of_seconds(3600.0),
            WallClockBand::WithinTwoHours,
            "一小时落在 2 小时以内"
        );
        assert_eq!(band_of_seconds(7200.0), WallClockBand::WithinTwoHours);
        assert_eq!(
            band_of_seconds(7200.5),
            WallClockBand::BetweenTwoHoursAndADay
        );
        assert_eq!(
            band_of_seconds(86_400.0),
            WallClockBand::BetweenTwoHoursAndADay
        );
        assert_eq!(band_of_seconds(86_400.5), WallClockBand::OverADay);
    }

    #[test]
    fn the_commit_pairs_verdict_needs_ten_pairs_and_then_reads_the_median() {
        assert_eq!(
            commit_pairs_verdict(&[0.9; 9]),
            CommitPairsVerdict::StillMissing,
            "9 对不到 10 对"
        );
        assert_eq!(
            commit_pairs_verdict(&[0.5; 10]),
            CommitPairsVerdict::MoreThanHalfInvalidated
        );
        assert_eq!(
            commit_pairs_verdict(&[0.1; 10]),
            CommitPairsVerdict::MostlyReused
        );
        assert_eq!(
            commit_pairs_verdict(&[0.3; 10]),
            CommitPairsVerdict::Partial
        );
        assert_eq!(commit_pairs_verdict(&[]), CommitPairsVerdict::StillMissing);
    }

    #[test]
    fn the_commit_pair_trajectory_counts_the_positive_pairs_and_keeps_peak_median_and_final() {
        let trajectory = commit_pair_trajectory(&[0.0, 0.2, 0.5], Some(2)).expect("三对");
        assert_eq!(trajectory.computable, 3);
        assert_eq!(trajectory.positive, 2, "作废 0 的那一对不算为正");
        assert!(
            (trajectory.peak - 0.5).abs() < f64::EPSILON
                && (trajectory.median - 0.2).abs() < f64::EPSILON
        );
        assert_eq!(trajectory.final_value, Some(0.5));
        assert_eq!(
            commit_pair_trajectory(&[0.0, 0.2], None)
                .expect("两对")
                .final_value,
            None
        );
        assert_eq!(commit_pair_trajectory(&[], None), None);
    }

    #[test]
    fn the_budget_opens_points_in_ascending_state_order_until_the_next_one_no_longer_fits() {
        let points = [1000, 10, 30, 20];
        assert_eq!(
            open_under_budget(&points, 60.0, 1.0),
            OpenUnderBudget {
                points: 3,
                states: 60
            },
            "10 + 20 + 30 = 60 秒恰好开得下"
        );
        assert_eq!(
            open_under_budget(&points, 59.0, 1.0),
            OpenUnderBudget {
                points: 2,
                states: 30
            }
        );
        assert_eq!(
            open_under_budget(&points, 5.0, 1.0),
            OpenUnderBudget {
                points: 0,
                states: 0
            }
        );
        assert_eq!(
            open_under_budget(&points, 2000.0, 2.0),
            OpenUnderBudget {
                points: 4,
                states: 1060
            }
        );
    }

    #[test]
    fn the_cumulative_curve_has_ten_ticks_over_the_points_in_ascending_order() {
        let points: Vec<u64> = (1..=10).rev().collect();
        let curve = cumulative_curve(&points, 1.0);
        let seconds: Vec<f64> = curve.iter().map(|(_, seconds)| *seconds).collect();
        assert_eq!(
            seconds,
            vec![1.0, 3.0, 6.0, 10.0, 15.0, 21.0, 28.0, 36.0, 45.0, 55.0]
        );
        assert!((curve[0].0 - 0.1).abs() < f64::EPSILON && (curve[9].0 - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn two_points_fit_a_fixed_cost_and_a_slope() {
        let (fixed, slope) = fit_through_two_points(100, 10.0, 300, 20.0).expect("两点上升");
        assert!(
            (slope - 20.0).abs() < 1e-9 && (fixed - 5.0).abs() < 1e-9,
            "斜率 (300 − 100) ÷ (20 − 10) = 20 态每秒，固定 10 − 100 ÷ 20 = 5：{fixed} {slope}"
        );
        assert_eq!(
            fit_through_two_points(100, 10.0, 300, 10.0),
            None,
            "秒数不升就拟合不出"
        );
        assert_eq!(
            fit_through_two_points(300, 10.0, 100, 20.0),
            None,
            "态数不升就拟合不出"
        );
    }

    fn cold_log(
        item: &str,
        states: u64,
        reused: u64,
        seconds: f64,
        compile: f64,
        judge: f64,
        storing: f64,
    ) -> String {
        format!(
            "some other line\nCRASH_AMPLIFICATION_GPU_TOTAL cards_used=2 blocks=1 states={states} compile_seconds={compile} judge_seconds={judge} states_per_second=1 seconds_storing={storing} expand_up_to=6 exhaustive=true\nCRASH_AMPLIFICATION_ITEM item={item} exit=0 states={states} reused={reused} red=0 lines=1 expand_up_to=6 exhaustive=true verdict=ok seconds={seconds} ignored_points=0\n"
        )
    }

    #[test]
    fn cold_run_log_gives_the_state_count_the_seconds_and_the_two_throughputs() {
        let run = ColdRun::from_log(&cold_log(
            "snapshot-creation",
            16_842_807,
            0,
            2000.0,
            100.0,
            400.0,
            100.0,
        ))
        .expect("日志读得出");
        assert_eq!(
            (run.item.as_str(), run.states, run.reused, run.exit_code),
            ("snapshot-creation", 16_842_807, 0, 0)
        );
        assert!(
            (run.recording_seconds() - 1400.0).abs() < 1e-9,
            "2000 − 100 − 400 − 100"
        );
        assert!(run.outer_seconds_cover_the_inner_ones());
        assert!(
            (run.rate_total() - 16_842_807.0 / 2000.0).abs() < 1e-6
                && (run.rate_judge() - 16_842_807.0 / 400.0).abs() < 1e-6
        );
        assert!(ColdRun::from_log(
            "CRASH_AMPLIFICATION_ITEM item=x exit=0 states=1 reused=0 seconds=1"
        )
        .expect_err("没有判段行")
        .contains("GPU_TOTAL"));
        assert!(ColdRun::from_log("nothing")
            .expect_err("没有项行")
            .contains("恰好一行"));
        let two_hosts = format!("{}{}", cold_log("x", 10, 0, 100.0, 10.0, 40.0, 5.0), "CRASH_AMPLIFICATION_GPU_TOTAL cards_used=1 blocks=1 states=10 compile_seconds=12.0 judge_seconds=30.0 states_per_second=1 seconds_storing=7.0 expand_up_to=6 exhaustive=true\n");
        let merged = ColdRun::from_log(&two_hosts).expect("两台");
        assert_eq!(
            (
                merged.gpu_total_lines,
                merged.compile_seconds,
                merged.judge_seconds,
                merged.seconds_storing
            ),
            (2, 12.0, 40.0, 7.0),
            "两台同时判，各段取最大"
        );
    }

    #[test]
    fn cold_run_that_is_not_cold_or_not_monotone_or_not_the_pinned_item_is_void() {
        let small = ColdRun::from_log(&cold_log(
            "snapshot-creation",
            16_842_807,
            0,
            2000.0,
            100.0,
            400.0,
            100.0,
        ))
        .expect("小项");
        let medium = ColdRun::from_log(&cold_log(
            "overwrite-and-unlink-under-a-snapshot",
            100_000_000,
            0,
            10_000.0,
            100.0,
            5000.0,
            400.0,
        ))
        .expect("中项");
        assert_eq!(voided_cold_runs(&small, &medium), Vec::<String>::new());
        let warm = ColdRun {
            reused: 5,
            ..medium.clone()
        };
        assert!(voided_cold_runs(&small, &warm)
            .iter()
            .any(|reason| reason.contains("库不冷")));
        let not_rising = ColdRun {
            seconds: 1999.0,
            ..medium.clone()
        };
        assert!(voided_cold_runs(&small, &not_rising)
            .iter()
            .any(|reason| reason.contains("分不出规模")));
        let not_covering = ColdRun {
            compile_seconds: 6000.0,
            ..medium.clone()
        };
        assert!(voided_cold_runs(&small, &not_covering)
            .iter()
            .any(|reason| reason.contains("包不住")));
        let another_item = ColdRun {
            states: 16_842_808,
            ..small.clone()
        };
        assert!(voided_cold_runs(&another_item, &medium)
            .iter()
            .any(|reason| reason.contains("不是 16842807")));
    }

    #[test]
    fn the_library_bytes_and_recorded_blocks_are_read_by_the_end_of_the_path() {
        let du = "168428070\t/work/e184/library-small\n1000000000\t/work/e184/library-medium\n";
        assert_eq!(library_bytes_of(du, "library-small"), Ok(168_428_070));
        assert_eq!(library_bytes_of(du, "library-medium"), Ok(1_000_000_000));
        assert!(library_bytes_of(du, "library-large").is_err());
        let reader = "CRASH_LIBRARY_KEYS\tlibrary=/work/e184/library-small\trecorded_blocks=257\tjudged_under_the_current_key=0\n\
                      CRASH_LIBRARY_KEYS\tlibrary=/work/e184/library-medium\trecorded_blocks=2561\tjudged_under_the_current_key=0\n";
        assert_eq!(recorded_blocks_of(reader, "library-small"), Ok(257));
        assert_eq!(recorded_blocks_of(reader, "library-medium"), Ok(2561));
        assert!(
            recorded_blocks_of(
                "CRASH_LIBRARY_KEYS\tlibrary=/x/library-small\tmissing\n",
                "library-small"
            )
            .is_err(),
            "读不到不当 0"
        );
    }

    fn cell_seconds(bodies: &[String], candidate: &str, commit_class: &str) -> f64 {
        let prefix =
            format!("name=compose_cell candidate={candidate} commit_class={commit_class} ");
        let body = bodies
            .iter()
            .find(|body| body.starts_with(&prefix))
            .unwrap_or_else(|| panic!("没有 {prefix:?} 的格子：{bodies:?}"));
        fields_of(body)["seconds"]
            .parse::<f64>()
            .expect("seconds 是数")
    }

    #[test]
    fn the_cells_judge_the_invalidated_states_at_the_judge_throughput_not_at_the_total_one() {
        let inputs = ComposeInputs {
            keys_seconds_total: 30.0,
            full_total: 4_000_000,
            quick_total: 400_000,
            point_states: vec![600_000, 400_000, 1_000_000, 2_000_000],
            write_path_median_fraction: Some(0.5),
            write_path_upper_bound_fraction: None,
        };
        let bodies = cell_bodies(
            &inputs,
            "medium",
            &Rates {
                total: 1000.0,
                judge: 4000.0,
            },
        );
        assert!(
            (cell_seconds(
                &bodies,
                "every_commit_judges_the_full_domain_by_block_keys",
                "write_path_median"
            ) - (30.0 + 0.5 * 4_000_000.0 / 4000.0))
                .abs()
                < 1e-9,
            "30 + 500 = 530 秒；按总吞吐算会是 2030"
        );
        assert!(
            (cell_seconds(
                &bodies,
                "commit_judges_the_block_reuse_domain_only",
                "write_path_median"
            ) - (30.0 + 0.5 * 400_000.0 / 4000.0))
                .abs()
                < 1e-9
        );
        assert!(
            (cell_seconds(
                &bodies,
                "commit_queries_the_library_only",
                "write_path_median"
            ) - 30.0)
                .abs()
                < 1e-9,
            "丙只查库"
        );
        assert!(
            (cell_seconds(
                &bodies,
                "every_commit_judges_the_full_domain_by_block_keys",
                "judge_only"
            ) - (30.0 + 4_000_000.0 / 4000.0))
                .abs()
                < 1e-9,
            "只动判器的提交作废占比取 1"
        );
        assert!(
            bodies
                .iter()
                .any(|body| body.contains("commit_class=write_path_upper_bound")
                    && body.contains("unavailable=no_invalidated_fraction")),
            "补丁那一格没有占比时写取不出，不当 0"
        );
        let full_by_the_main_agent = bodies
            .iter()
            .find(|body| body.contains("commit_queries_the_library_only_full_by_the_main_agent"))
            .expect("丙另列一格全量");
        assert!(
            full_by_the_main_agent.contains("seconds=4000.0"),
            "4,000,000 ÷ 1000 = 4000 秒：{full_by_the_main_agent}"
        );
        assert!(
            full_by_the_main_agent.contains("band=within_2h"),
            "{full_by_the_main_agent}"
        );
        assert_eq!(
            bodies
                .iter()
                .filter(|body| body.starts_with("name=compose_curve "))
                .count(),
            10
        );
        let two_hours = bodies
            .iter()
            .find(|body| body.contains("budget_hours=2 "))
            .expect("2 小时预算");
        assert!(
            two_hours.contains("open_points=4")
                && two_hours.contains("open_point_fraction=1.000000"),
            "4000 态每秒 × 7200 秒 = 28,800,000 ≥ 4,000,000：{two_hours}"
        );
    }

    fn compose_fixture() -> (ComposeTexts, Vec<KeysFile>) {
        let history = FlowRow {
            keys_seconds: 10.0,
            full: Some(1_000_000),
            quick_tier: Some(100_000),
            ..row_of(
                "random_history-broad-FourGibibytes-steps0100-seed00",
                "random_history",
                "candidate",
                2262,
                Some(900_000),
                30,
            )
        };
        let chain = FlowRow {
            keys_seconds: 20.0,
            full: Some(3_000_000),
            quick_tier: Some(300_000),
            ..row_of(
                "overwrite_chain-04_units_per_overwrite",
                "overwrite_chain",
                "candidate",
                1200,
                Some(2_900_000),
                28,
            )
        };
        let snapshot = FlowRow {
            full: Some(16_842_807),
            ..row_of(
                "smoke-snapshot_creation_stream",
                "smoke",
                "baseline",
                62,
                Some(16_842_782),
                24,
            )
        };
        let keys_output: String = [&history, &chain, &snapshot]
            .iter()
            .map(|row| format!("E7RESULT {} reused=false\n", row.body()))
            .collect();
        let keys_files = vec![
            keys_file_of(
                history,
                vec![block_of(1, 1, 0, 600_000, 0), block_of(2, 2, 0, 400_000, 1)],
            ),
            keys_file_of(
                chain,
                vec![
                    block_of(3, 3, 0, 1_000_000, 0),
                    block_of(4, 4, 0, 2_000_000, 1),
                ],
            ),
        ];
        let mut compare_output = String::new();
        for pair_number in 1..=10u64 {
            let pair = ComparedFlow {
                label: format!(
                    "random_history-broad-FourGibibytes-steps0100-seed00-of-pair{pair_number}"
                ),
                group: "random_history-broad-FourGibibytes-steps0100".to_string(),
                first_difference: FirstDifference::Write(3),
                segment: Some(1),
                point: Some(1),
                first_journal_record_write: Some(3),
                blocks_before: 100,
                blocks_after: 100,
                invalidated_blocks: 5,
                states_after: 1000,
                invalidated_states: pair_number * 100,
            };
            compare_output.push_str(&format!(
                "E7RESULT {}\n",
                compare_summary_body(
                    "commit-pair",
                    &format!("pair{pair_number:02}"),
                    &pair.group,
                    &[&pair],
                    0,
                    "no"
                )
            ));
        }
        let patch = |group: &str, invalidated_states: u64| ComparedFlow {
            label: format!("{group}-seed00"),
            group: group.to_string(),
            first_difference: FirstDifference::Base,
            segment: None,
            point: None,
            first_journal_record_write: None,
            blocks_before: 10,
            blocks_after: 10,
            invalidated_blocks: 10,
            states_after: 1000,
            invalidated_states,
        };
        for (group, invalidated_states) in [
            ("random_history-broad-FourGibibytes-steps0100", 800),
            ("overwrite_chain-04_units_per_overwrite", 400),
            (
                "random_history-broad_with_compaction_while_mounted-FourGibibytes-steps0100",
                0,
            ),
        ] {
            compare_output.push_str(&format!(
                "E7RESULT {}\n",
                compare_summary_body(
                    "patch",
                    "journal-mac",
                    group,
                    &[&patch(group, invalidated_states)],
                    0,
                    "unavailable"
                )
            ));
        }
        let texts = ComposeTexts {
            keys_output,
            compare_output,
            small_log: cold_log("snapshot-creation", 16_842_807, 0, 2000.0, 100.0, 400.0, 100.0),
            medium_log: cold_log("overwrite-and-unlink-under-a-snapshot", 100_000_000, 0, 10_000.0, 100.0, 5000.0, 400.0),
            library_bytes: "168428070\t/work/e184/library-small\n1000000000\t/work/e184/library-medium\n".to_string(),
            recorded_blocks: "CRASH_LIBRARY_KEYS\tlibrary=/work/e184/library-small\trecorded_blocks=257\nCRASH_LIBRARY_KEYS\tlibrary=/work/e184/library-medium\trecorded_blocks=2561\n".to_string(),
            final_pair: Some("pair10".to_string()),
        };
        (texts, keys_files)
    }

    #[test]
    fn the_compose_lines_give_the_pinned_cells_for_a_hand_made_load() {
        let (texts, keys_files) = compose_fixture();
        let mut output = Output::new();
        assert_eq!(
            compose_lines(&texts, &keys_files, &mut output),
            Ok(EXIT_CODE_OK)
        );
        assert!(line_with(
            &output,
            "E7RESULT name=stop_check item=small_item_states_match_the_recorded_snapshot_full "
        )
        .contains("result=pass small_item_states=16842807"));
        let small = line_with(&output, "E7RESULT name=compose_rate rate_source=small ");
        assert!(
            small.contains("states_per_second_of_judging=42107.0")
                && small.contains("library_bytes_per_state=10.0000"),
            "{small}"
        );
        let medium = line_with(&output, "E7RESULT name=compose_rate rate_source=medium ");
        assert!(
            medium.contains("states_per_second_of_recording_and_judging=10000.0 states_per_second_of_judging=20000.0")
                && medium.contains("recording_seconds=4500.0"),
            "{medium}"
        );
        assert!(line_with(
            &output,
            "E7RESULT name=anchor identifier=small_item_states "
        )
        .ends_with("expected=16842807 actual=16842807 match=true"));
        assert!(
            line_with(
                &output,
                "E7RESULT name=anchor identifier=medium_item_states "
            )
            .ends_with("expected=167837792 actual=100000000 match=false"),
            "中项的态数与测试文件钉的不符只报不符，不作废、不改退出码"
        );
        let input = line_with(&output, "E7RESULT name=compose_input ");
        assert!(input.contains("flows_in_the_nominal_load=2 keys_seconds_total=30.0 full_total=4000000 quick_total=400000 points_total=4"), "{input}");
        let by_states = line_with(
            &output,
            "E7RESULT name=commit_pairs_trajectory basis=states ",
        );
        assert!(by_states.contains("computable=10 positive=10 peak=1.000000 median=0.550000 final=1.000000 verdict=more_than_half_invalidated"), "{by_states}");
        let by_blocks = line_with(
            &output,
            "E7RESULT name=commit_pairs_trajectory basis=blocks ",
        );
        assert!(
            by_blocks.contains("median=0.050000 final=0.050000 verdict=mostly_reused"),
            "按块与按态数各判一次门槛：{by_blocks}"
        );
        let medium_cells: Vec<String> = output
            .lines
            .iter()
            .filter(|line| line.contains("rate_source=medium"))
            .map(|line| line.trim_start_matches("E7RESULT ").to_string())
            .collect();
        for (candidate, class, seconds) in [
            (
                "every_commit_judges_the_full_domain_by_block_keys",
                "write_path_median",
                140.0,
            ),
            (
                "every_commit_judges_the_full_domain_by_block_keys",
                "write_path_upper_bound",
                150.0,
            ),
            (
                "every_commit_judges_the_full_domain_by_block_keys",
                "judge_only",
                230.0,
            ),
            (
                "commit_judges_the_block_reuse_domain_only",
                "write_path_median",
                41.0,
            ),
            ("commit_queries_the_library_only", "write_path_median", 30.0),
        ] {
            assert!(
                (cell_seconds(&medium_cells, candidate, class) - seconds).abs() < 1e-6,
                "{candidate} {class} 应是 {seconds} 秒"
            );
        }
        assert!(medium_cells.iter().any(|body| body
            .contains("commit_queries_the_library_only_full_by_the_main_agent")
            && body.contains("seconds=400.0")));
        assert!(output
            .lines
            .last()
            .expect("有收尾行")
            .contains("name=compose_done commit_pairs_computable=10"));
    }

    #[test]
    fn warm_library_voids_the_cold_runs_and_a_small_item_other_than_the_recorded_snapshot_stops_the_compose(
    ) {
        let (mut texts, keys_files) = compose_fixture();
        texts.medium_log = cold_log(
            "overwrite-and-unlink-under-a-snapshot",
            100_000_000,
            3,
            10_000.0,
            100.0,
            5000.0,
            400.0,
        );
        let mut output = Output::new();
        assert_eq!(
            compose_lines(&texts, &keys_files, &mut output),
            Ok(EXIT_CODE_POSITIVE_CONTROL_FAILED)
        );
        assert!(
            line_with(&output, "E7RESULT name=void item=cold_library_runs ").contains("库不冷")
        );
        assert!(
            !output
                .lines
                .iter()
                .any(|line| line.contains("name=compose_cell")),
            "作废之后不出格子"
        );
        let (mut other_texts, other_keys) = compose_fixture();
        other_texts.small_log = cold_log(
            "snapshot-creation",
            16_842_808,
            0,
            2000.0,
            100.0,
            400.0,
            100.0,
        );
        let mut stopped = Output::new();
        assert_eq!(
            compose_lines(&other_texts, &other_keys, &mut stopped),
            Ok(EXIT_CODE_STOP_CLAUSE_FAILED)
        );
        assert!(line_with(
            &stopped,
            "E7RESULT name=stop_check item=small_item_states_match_the_recorded_snapshot_full "
        )
        .contains("result=fail"));
    }

    #[test]
    fn the_nominal_load_is_the_hundred_step_histories_and_the_overwrite_chains_only() {
        let in_the_load = |label: &str| {
            is_in_the_nominal_regular_layer_load(&row_of(label, "x", "candidate", 1, Some(1), 1))
        };
        assert!(in_the_load(
            "random_history-broad-FourGibibytes-steps0100-seed19"
        ));
        assert!(in_the_load("overwrite_chain-16_units_per_overwrite"));
        assert!(!in_the_load(
            "random_history-broad-FourGibibytes-steps0050-seed00"
        ));
        assert!(!in_the_load(
            "random_history-broad_with_compaction_while_mounted-FourGibibytes-steps0100-seed00"
        ));
        assert!(!in_the_load("smoke-snapshot_creation_stream"));
    }

    fn tiny_write(device: u32, kind: StepKind, offset: u64, fill: u8) -> RetainedWrite {
        RetainedWrite {
            device: DeviceIdentity(device),
            kind,
            is_force_unit_access: kind == StepKind::RootRecordFua,
            offset: DeviceOffsetInBytes(offset),
            contents: WrittenContents::Bytes(vec![fill; 512]),
        }
    }

    /// 三次写分两段（写 0、1 一段，写 2 一段），第二次写是第一条 journal 记录写。
    fn tiny_dump(label: &str, fill_of_the_record_write: u8) -> FlowDump {
        FlowDump {
            family: "random_history".to_string(),
            label: label.to_string(),
            tier: "candidate".to_string(),
            seed: Some(SEED_BASE_OF_THE_RANDOM_HISTORIES),
            ending: "Completed".to_string(),
            device_bytes: 1 << 20,
            device_identities: vec![0, 1],
            operations_of_the_make_filesystem: 1,
            base_writes: vec![tiny_write(0, StepKind::UnitWrite, 0, 0x11)],
            writes: vec![
                tiny_write(0, StepKind::UnitWrite, 4096, 1),
                tiny_write(1, StepKind::JournalRecord, 4096, fill_of_the_record_write),
                tiny_write(0, StepKind::RootRecordFua, 8192, 3),
            ],
            segments: vec![vec![0, 1], vec![2]],
            judged_root_index: Some(2),
            versions_with_a_file: Vec::new(),
            versions_without_a_file: Vec::new(),
            compaction_operations: None,
        }
    }

    #[test]
    fn the_first_difference_is_the_base_or_the_first_write_that_differs_or_the_shorter_length() {
        let before = tiny_dump("flow", 2);
        assert_eq!(
            first_difference(&before, &tiny_dump("flow", 2)),
            FirstDifference::Identical
        );
        assert_eq!(
            first_difference(&before, &tiny_dump("flow", 9)),
            FirstDifference::Write(1)
        );
        let mut longer = tiny_dump("flow", 2);
        longer
            .writes
            .push(tiny_write(0, StepKind::UnitWrite, 12_288, 4));
        assert_eq!(
            first_difference(&before, &longer),
            FirstDifference::Write(3),
            "后一份多写一次：第一处不同是较短那份的写数"
        );
        let mut other_base = tiny_dump("flow", 9);
        other_base.base_writes[0] = tiny_write(0, StepKind::UnitWrite, 0, 0x22);
        assert_eq!(
            first_difference(&before, &other_base),
            FirstDifference::Base,
            "基镜像不同时先报 base，哪怕写表也不同"
        );
        assert_eq!(FirstDifference::Write(7).text(), "write:7");
        assert_eq!(FirstDifference::Base.text(), "base");
        assert_eq!(FirstDifference::Identical.text(), "none");
    }

    #[test]
    fn write_index_is_placed_in_its_segment_and_in_the_crash_point_of_the_enumeration_cases() {
        let dump = tiny_dump("flow", 2);
        assert_eq!(segment_of_write(&dump.segments, 0), Some(0));
        assert_eq!(segment_of_write(&dump.segments, 1), Some(0));
        assert_eq!(segment_of_write(&dump.segments, 2), Some(1));
        assert_eq!(segment_of_write(&dump.segments, 3), None);
        assert_eq!(
            point_of_write(&dump, 1),
            Some(0),
            "写 0、1 是第一步，写 2 是第二步"
        );
        assert_eq!(point_of_write(&dump, 2), Some(1));
        assert_eq!(point_of_write(&dump, 3), None);
    }

    #[test]
    fn compared_flow_reports_where_it_first_differs_and_how_many_blocks_and_states_it_invalidates()
    {
        let before_dump = tiny_dump("flow", 2);
        let after_dump = tiny_dump("flow", 9);
        let row = row_of("flow", "random_history", "candidate", 3, Some(12), 2);
        let before_keys = keys_file_of(
            row.clone(),
            vec![block_of(1, 1, 0, 10, 0), block_of(2, 2, 0, 20, 1)],
        );
        let after_keys = keys_file_of(
            row,
            vec![
                block_of(1, 1, 0, 10, 0),
                block_of(5, 5, 0, 20, 1),
                block_of(6, 6, 0, 30, 1),
            ],
        );
        let compared = compare_one_flow(&before_dump, &after_dump, &before_keys, &after_keys);
        assert_eq!(
            (
                compared.first_difference,
                compared.segment,
                compared.point,
                compared.first_journal_record_write
            ),
            (FirstDifference::Write(1), Some(0), Some(0), Some(1))
        );
        assert_eq!(
            (
                compared.blocks_before,
                compared.blocks_after,
                compared.invalidated_blocks
            ),
            (2, 3, 2)
        );
        assert_eq!(
            (compared.states_after, compared.invalidated_states),
            (60, 50)
        );
        assert!(
            (compared.block_fraction() - 2.0 / 3.0).abs() < 1e-12
                && (compared.state_fraction() - 50.0 / 60.0).abs() < 1e-12
        );
        assert!(
            ExpectedFirstDifference::JournalRecord.holds_for(&compared),
            "第一处不同的写就是第一条 journal 记录写"
        );
        assert!(!ExpectedFirstDifference::Identical.holds_for(&compared));
        let unchanged = compare_one_flow(
            &before_dump,
            &tiny_dump("flow", 2),
            &before_keys,
            &before_keys,
        );
        assert!(
            ExpectedFirstDifference::Identical.holds_for(&unchanged)
                && !ExpectedFirstDifference::JournalRecord.holds_for(&unchanged)
        );
        let summary = compare_summary_body(
            "patch",
            "journal-mac",
            "g",
            &[&compared, &unchanged],
            1,
            "unavailable",
        );
        assert!(
            summary.contains("flows_compared=2 flows_unavailable=1 flows_with_a_difference=1 blocks_after=5 invalidated_blocks=2 invalidated_block_fraction=0.400000"),
            "{summary}"
        );
    }

    /// 这条用例自己的临时目录，交回认领着它的守卫：用例做完、断言失败、panic 都删掉它。
    fn scratch_directory(
        name: &str,
    ) -> singlefs_harness::temporary_files::TemporaryPathRemovedOnDrop {
        let directory = singlefs_harness::temporary_files::TemporaryPathRemovedOnDrop::claiming(
            std::env::temp_dir().join(format!("e184-main-test-{}-{name}", std::process::id())),
        );
        let _ = std::fs::remove_dir_all(&*directory);
        std::fs::create_dir_all(&*directory).expect("建临时目录");
        directory
    }

    fn write_flow_files(directory: &Path, dump: &FlowDump, blocks: Vec<BlockOfAPlan>) {
        dump.write_atomically(&directory.join(format!("{}{DUMP_FILE_SUFFIX}", dump.label)))
            .expect("写转储");
        let row = row_of(
            &dump.label,
            &dump.family,
            &dump.tier,
            dump.writes.len(),
            Some(12),
            2,
        );
        let text = keys_file_text(&row, Expansion::Full, &blocks);
        std::fs::write(
            directory.join(format!("{}{FULL_KEYS_FILE_SUFFIX}", dump.label)),
            text,
        )
        .expect("写块键文件");
    }

    #[test]
    fn the_compare_command_pairs_flows_by_label_writes_the_lines_and_checks_the_expected_first_difference(
    ) {
        let (before_directory, after_directory) = (
            scratch_directory("compare-before"),
            scratch_directory("compare-after"),
        );
        write_flow_files(
            &before_directory,
            &tiny_dump("flow-a", 2),
            vec![block_of(1, 1, 0, 10, 0), block_of(2, 2, 0, 20, 1)],
        );
        write_flow_files(
            &after_directory,
            &tiny_dump("flow-a", 9),
            vec![block_of(1, 1, 0, 10, 0), block_of(5, 5, 0, 20, 1)],
        );
        write_flow_files(
            &after_directory,
            &tiny_dump("flow-only-after", 2),
            vec![block_of(7, 7, 0, 5, 0)],
        );
        let out_file = after_directory.join("compare.out");
        let arguments: Vec<String> = [
            "--before",
            before_directory.to_str().expect("路径"),
            "--after",
            after_directory.to_str().expect("路径"),
            "--kind",
            "patch",
            "--pair",
            "journal-mac",
            "--expect-first-difference",
            "journal-record",
            "--out",
            out_file.to_str().expect("路径"),
        ]
        .iter()
        .map(|word| (*word).to_string())
        .collect();
        assert_eq!(run_compare_command(&arguments), EXIT_CODE_OK);
        let text = std::fs::read_to_string(&out_file).expect("读得到产物");
        assert!(
            text.contains(
                "name=compare flow=flow-a group=flow-a first_difference=write:1 segment=0 point=0"
            ),
            "{text}"
        );
        assert!(
            text.contains("invalidated_blocks=1 states_after=30 invalidated_states=20"),
            "{text}"
        );
        assert!(
            text.contains(
                "name=compare_unavailable flow=flow-only-after reason=missing_on_one_side"
            ),
            "{text}"
        );
        assert!(
            text.contains("name=control kind=expected_first_difference flow=flow-a passed=true"),
            "{text}"
        );
        assert!(text.contains("name=compare_summary kind=patch pair=journal-mac group=flow-a flows_compared=1 flows_unavailable=1"), "{text}");
        assert!(text.contains("name=compare_done kind=patch pair=journal-mac flows_compared=1 flows_unavailable=1 controls_failed=0"), "{text}");
        let wrong_expectation: Vec<String> = arguments
            .iter()
            .map(|word| {
                if word == "journal-record" {
                    "none".to_string()
                } else {
                    word.clone()
                }
            })
            .collect();
        assert_eq!(
            run_compare_command(&wrong_expectation),
            EXIT_CODE_POSITIVE_CONTROL_FAILED,
            "期望两份逐字相同而实际不同：阳性对照没过"
        );
    }

    fn planned_blocks_of(
        blocks: Vec<BlockOfAPlan>,
        blocks_of_points_without_segments: usize,
    ) -> PlannedBlocks {
        let state_count = blocks.iter().map(|block| block.states).sum();
        PlannedBlocks {
            state_count,
            state_count_by_the_counting_function: state_count,
            point_count: blocks
                .iter()
                .map(|block| block.point_index)
                .max()
                .map_or(0, |last| last + 1),
            blocks,
            blocks_of_points_without_segments,
        }
    }

    #[test]
    fn the_two_expansions_share_exactly_the_blocks_of_the_points_without_segments_and_every_reuse_key(
    ) {
        let full = planned_blocks_of(
            vec![
                block_of(1, 10, 0, 7, 0),
                block_of(2, 20, 0, 7, 1),
                block_of(3, 30, 0, 1, 2),
            ],
            1,
        );
        let quick = planned_blocks_of(
            vec![
                block_of(1, 11, 0, 3, 0),
                block_of(2, 21, 0, 3, 1),
                block_of(3, 30, 0, 1, 2),
            ],
            1,
        );
        let control = two_expansions_control("flow", &full, &quick);
        assert!(control.passed, "{control:?}");
        let end_node_differs = planned_blocks_of(
            vec![
                block_of(1, 11, 0, 3, 0),
                block_of(2, 21, 0, 3, 1),
                block_of(3, 31, 0, 1, 2),
            ],
            1,
        );
        assert!(
            !two_expansions_control("flow", &full, &end_node_differs).passed,
            "末节点的块键也不同：共用的块数 0 ≠ 1"
        );
        let plan_hash_ignored = planned_blocks_of(
            vec![
                block_of(1, 10, 0, 7, 0),
                block_of(2, 20, 0, 7, 1),
                block_of(3, 30, 0, 1, 2),
            ],
            1,
        );
        assert!(
            !two_expansions_control("flow", &full, &plan_hash_ignored).passed,
            "两种展开的块键全部相同：计划哈希没有区分开它们"
        );
        let other_reuse_key = planned_blocks_of(
            vec![
                block_of(1, 11, 0, 3, 0),
                block_of(9, 21, 0, 3, 1),
                block_of(3, 30, 0, 1, 2),
            ],
            1,
        );
        assert!(
            !two_expansions_control("flow", &full, &other_reuse_key).passed,
            "复用键两边不是同一批"
        );
    }

    #[test]
    fn the_count_disagreements_name_each_pair_of_numbers_that_do_not_agree() {
        let consistent =
            planned_blocks_of(vec![block_of(1, 1, 0, 7, 0), block_of(2, 2, 0, 3, 1)], 0);
        assert_eq!(
            count_disagreements_of("flow", Expansion::Full, Some(10), &consistent),
            Vec::<String>::new()
        );
        let counted_otherwise = PlannedBlocks {
            state_count_by_the_counting_function: 11,
            ..planned_blocks_of(vec![block_of(1, 1, 0, 7, 0), block_of(2, 2, 0, 3, 1)], 0)
        };
        assert_eq!(
            count_disagreements_of("flow", Expansion::Full, Some(10), &counted_otherwise).len(),
            1
        );
        let blocks_do_not_add_up = PlannedBlocks {
            state_count: 12,
            state_count_by_the_counting_function: 12,
            ..planned_blocks_of(vec![block_of(1, 1, 0, 7, 0)], 0)
        };
        assert_eq!(
            count_disagreements_of(
                "flow",
                Expansion::QuickTier,
                Some(12),
                &blocks_do_not_add_up
            )
            .len(),
            1,
            "各块态数之和 7 ≠ 12"
        );
        assert_eq!(
            count_disagreements_of("flow", Expansion::Full, Some(9), &consistent).len(),
            1,
            "本地按段数的态数 9 ≠ 计划的 10"
        );
    }

    #[test]
    fn the_dump_round_trip_control_passes_on_a_dump_and_the_name_of_a_flow_is_not_in_its_keys() {
        assert!(dump_round_trip_control(&tiny_dump("flow", 2)).passed);
        assert!(
            controls_apply_to(&tiny_dump("flow", 2)),
            "种子下标 0 的随机历史跑对照"
        );
        let mut other_seed = tiny_dump("flow", 2);
        other_seed.seed = Some(SEED_BASE_OF_THE_RANDOM_HISTORIES + 1);
        assert!(!controls_apply_to(&other_seed));
        other_seed.seed = None;
        assert!(
            controls_apply_to(&other_seed),
            "手写流（没有种子）每条都跑对照"
        );
    }

    #[test]
    fn the_copied_segments_are_found_verbatim_in_the_test_files_and_a_changed_line_is_caught() {
        let segments = copied_segments_of(OWN_SOURCE_TEXT).expect("这份源文件里的标记配得上");
        let names: Vec<&str> = segments.iter().map(|(name, _, _)| name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "fixed_script_head",
                "fixed_script_body",
                "multi_record_publish",
                "new_pool_prepare_body"
            ]
        );
        assert!(
            segments
                .iter()
                .all(|(_, _, text)| text.lines().count() >= 10),
            "每一段都不是空的"
        );
        let readings = copied_text_checks(OWN_SOURCE_TEXT);
        assert_eq!(readings.len(), EXPECTED_COPIED_SEGMENTS + 1);
        assert!(
            readings.iter().all(|reading| reading.passed),
            "{readings:?}"
        );
        let changed = OWN_SOURCE_TEXT.replacen(
            "const SECOND_FILE_BYTES: usize = 4100;",
            "const SECOND_FILE_BYTES: usize = 4101;",
            1,
        );
        let readings_after_the_change = copied_text_checks(&changed);
        assert!(
            readings_after_the_change.iter().any(|reading| reading.item
                == "copied_text_verbatim_fixed_script_head"
                && !reading.passed),
            "改了抄来的一行，对原文的逐字比对要红：{readings_after_the_change:?}"
        );
    }

    #[test]
    fn unbalanced_or_badly_indented_copied_markers_are_refused() {
        let begin = ["    // COPIED", "-BEGIN a crates/x.rs:1-2"].concat();
        let end = ["    // COPIED", "-END a"].concat();
        assert!(copied_segments_of(&format!("{begin}\n    line\n"))
            .expect_err("没有结束")
            .contains("没有结束"));
        assert!(copied_segments_of(&format!("{end}\n"))
            .expect_err("没有开始")
            .contains("没有开始"));
        assert!(
            copied_segments_of(&format!("{begin}\nline without indentation\n{end}\n"))
                .expect_err("缩进不够")
                .contains("缩进不够")
        );
        assert_eq!(
            copied_segments_of(&format!("{begin}\n    first\n\n      second\n{end}\n")),
            Ok(vec![(
                "a".to_string(),
                "crates/x.rs:1-2".to_string(),
                "first\n\n  second".to_string()
            )])
        );
    }

    #[test]
    fn the_numbers_pinned_by_the_tests_are_written_in_the_test_files_and_numbers_are_matched_with_or_without_underscores(
    ) {
        for pinned in &PINNED_BY_THE_TESTS {
            for number in [Some(pinned.two_state), Some(pinned.full), pinned.quick_tier]
                .into_iter()
                .flatten()
            {
                assert!(
                    number_appears_in(pinned.test_text, number),
                    "{} 的 {number} 不在测试文件里",
                    pinned.label
                );
            }
        }
        assert!(number_appears_in("const A: u64 = 16_777_240;", 16_777_240));
        assert!(number_appears_in("共 1678377089 个", 1_678_377_089));
        assert!(number_appears_in("x 123 y", 123));
        assert!(number_appears_in("x 1_234 y", 1234));
        assert!(!number_appears_in("x 16_777_241 y", 16_777_240));
    }

    #[test]
    fn the_local_constants_agree_with_the_crates_and_the_stop_checks_of_the_constants_pass() {
        let readings = local_constants_checks();
        assert_eq!(readings.len(), 3);
        assert!(
            readings.iter().all(|reading| reading.passed),
            "{readings:?}"
        );
    }

    #[test]
    fn the_family_summary_reports_spread_and_the_count_of_flows_with_a_thousand_writes() {
        let rows = vec![
            row_of(
                "random_history-broad-FourGibibytes-steps0100-seed00",
                "random_history",
                "candidate",
                900,
                Some(1 << 10),
                12,
            ),
            row_of(
                "random_history-broad-FourGibibytes-steps0100-seed01",
                "random_history",
                "candidate",
                1000,
                Some(1 << 20),
                28,
            ),
            row_of(
                "random_history-broad-FourGibibytes-steps0100-seed02",
                "random_history",
                "candidate",
                2000,
                None,
                40,
            ),
            row_of(
                "overwrite_chain-04_units_per_overwrite",
                "overwrite_chain",
                "candidate",
                1100,
                Some(1 << 30),
                28,
            ),
        ];
        let bodies = family_summary_bodies(&rows);
        assert_eq!(bodies.len(), 2, "两个（族，组）");
        let histories = bodies
            .iter()
            .find(|body| body.contains("family=random_history "))
            .expect("随机历史那一行");
        assert!(histories.contains("group=random_history-broad-FourGibibytes-steps0100 flows=3 flows_with_writes_at_least_1000=2 flows_beyond_u64=1"), "{histories}");
        assert!(
            histories.contains("writes=[minimum=900 median=1000 maximum=2000]")
                && histories.contains("longest_segment=[minimum=12 median=28 maximum=40]"),
            "{histories}"
        );
        assert!(
            histories.contains("log2_two_state=[minimum=10 median=15 maximum=20]"),
            "装不进 u64 的那条不进 log2 的统计：{histories}"
        );
    }

    #[test]
    fn the_snapshot_creation_stream_is_recorded_with_the_pinned_numbers_and_every_check_and_control_passes(
    ) {
        let prepared = prepared_snapshot_creation_stream();
        let readings = recorded_flow_readings(&prepared, &PINNED_BY_THE_TESTS[3]);
        assert_eq!(readings.len(), 5);
        assert!(
            readings.iter().all(|reading| reading.passed),
            "{readings:?}"
        );
        let dump = prepared.dump();
        let reading = read_the_flow(&dump, &prepared.base).expect("建快照流排得了计划");
        assert_eq!(
            (
                reading.row.writes,
                reading.row.segments,
                reading.row.longest_segment
            ),
            (64, 16, 24)
        );
        assert_eq!(
            (reading.row.two_state, reading.row.full),
            (Some(16_842_785), Some(16_842_815))
        );
        assert_eq!(reading.row.two_state_agrees_with_crates, Some(true));
        assert!(
            reading.count_disagreements.is_empty(),
            "{:?}",
            reading.count_disagreements
        );
        let full = reading.full.as_ref().expect("全量计划");
        let quick = reading.quick.as_ref().expect("甲二计划");
        assert_eq!(full.state_count, 16_842_815);
        assert!(quick.state_count < full.state_count);
        assert_eq!(reading.row.blocks_full, Some(full.blocks.len()));
        let controls = controls_of_the_flow(&dump, &prepared.base, &reading).expect("对照造得出");
        assert_eq!(
            controls
                .iter()
                .map(|control| control.kind)
                .collect::<Vec<_>>(),
            vec![
                "dump_round_trip",
                "base_sector_changed",
                "name_not_in_key",
                "two_expansions"
            ]
        );
        assert!(
            controls.iter().all(|control| control.passed),
            "{controls:?}"
        );
        let direct = direct_base_control(&prepared, &dump).expect("直接的基镜像对得上");
        assert!(direct.passed, "{direct:?}");
    }

    #[test]
    fn the_block_count_bound_adds_the_points_left_over_to_the_blocks_of_the_states() {
        assert_eq!(block_count_bound_of(0, 0), 2, "没有态也留末节点与一点余量");
        assert_eq!(block_count_bound_of(65_536, 3), 1 + 3 + 2);
        assert_eq!(block_count_bound_of(65_537, 3), 2 + 3 + 2);
        assert_eq!(
            block_count_bound_of((1u64 << 56) - 1, 100),
            (1u64 << 40) + 102,
            "一个 56 写的段：2^56 − 1 个状态正好 2^40 块"
        );
        assert_eq!(
            block_count_bound_of(u64::MAX, usize::MAX),
            u64::MAX,
            "越界饱和，不回绕成小数"
        );
        assert_eq!(MOST_BLOCKS_PLANNED_PER_FLOW, 1 << 26);
        assert_eq!(MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP, 1 << 24);
    }

    /// 一条段写数 `long_segment_writes` 的单元写段，后面接一条根槽写的段：两态闭式 2^n + 1，只拿来数态数、不排计划。
    fn dump_with_one_long_segment(label: &str, long_segment_writes: u64) -> FlowDump {
        let root_index = usize::try_from(long_segment_writes).expect("装得进 usize");
        let mut dump = tiny_dump(label, 2);
        dump.writes = (0..long_segment_writes)
            .map(|index| tiny_write(0, StepKind::UnitWrite, 4096 + 512 * index, 1))
            .chain(std::iter::once(tiny_write(
                0,
                StepKind::RootRecordFua,
                1 << 19,
                3,
            )))
            .collect();
        dump.segments = vec![(0..root_index).collect(), vec![root_index]];
        dump.judged_root_index = Some(root_index);
        dump
    }

    #[test]
    fn a_flow_whose_blocks_would_not_fit_in_memory_is_counted_but_not_planned() {
        let dump = dump_with_one_long_segment("fifty-six", 56);
        let base = dump.base_pool();
        let counted = state_count_of_the_flow(&base, &dump, Expansion::Full);
        assert_eq!(
            counted,
            Some((1u64 << 56) + 1),
            "1 + (2^56 − 1) + (2^1 − 1)"
        );
        assert_eq!(
            state_count_to_plan(&base, &dump, Expansion::Full, MOST_BLOCKS_PLANNED_PER_FLOW),
            None,
            "2^40 块，覆盖写链 u = 16 的单元段就是这个形"
        );
        assert_eq!(
            state_count_to_plan(&base, &dump, Expansion::Full, u64::MAX),
            counted,
            "上限放开就有值：拦的只是块数"
        );
        let refusal = plan_blocks(&base, &dump, Expansion::Full, default_item())
            .expect_err("块数上界超过上限，排计划之前就拒");
        assert!(
            refusal.contains("超过上限") && refusal.contains(&((1u64 << 56) + 1).to_string()),
            "{refusal}"
        );
    }

    #[test]
    fn the_two_limits_split_flows_by_the_blocks_of_their_states() {
        let (base_of_forty, forty) = {
            let dump = dump_with_one_long_segment("forty", 40);
            (dump.base_pool(), dump)
        };
        assert_eq!(
            state_count_to_plan(
                &base_of_forty,
                &forty,
                Expansion::Full,
                MOST_BLOCKS_PLANNED_PER_FLOW
            ),
            Some((1u64 << 40) + 1),
            "2^24 + 1 块排得了计划"
        );
        assert_eq!(
            state_count_to_plan(
                &base_of_forty,
                &forty,
                Expansion::Full,
                MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP
            ),
            None,
            "转储往返要摆两份计划，上限再小一档，2^24 + 1 块就不比块键"
        );
        let bound_of_forty = (1u64 << 24) + 1 + 2 + 2;
        assert_eq!(
            block_count_bound_of((1u64 << 40) + 1, forty.segments.len()),
            bound_of_forty,
            "2^24 + 1 块，两段加末节点各留一块的零头"
        );
        assert_eq!(
            state_count_to_plan(&base_of_forty, &forty, Expansion::Full, bound_of_forty),
            Some((1u64 << 40) + 1),
            "上界正好等于上限还排得了计划"
        );
        assert_eq!(
            state_count_to_plan(&base_of_forty, &forty, Expansion::Full, bound_of_forty - 1),
            None,
            "上界比上限多一块就不排"
        );
        let thirty_nine = dump_with_one_long_segment("thirty-nine", 39);
        assert_eq!(
            state_count_to_plan(
                &thirty_nine.base_pool(),
                &thirty_nine,
                Expansion::Full,
                MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP
            ),
            Some((1u64 << 39) + 1)
        );
    }

    #[test]
    fn a_small_limit_refuses_the_full_plan_of_a_real_flow_and_leaves_the_quick_tier_plan() {
        let prepared = prepared_new_pool_stream();
        let dump = prepared.dump();
        let refusal =
            plan_blocks_under(&prepared.base, &dump, Expansion::Full, default_item(), 100)
                .expect_err("16777268 个状态切 257 块，上界 271 超过 100");
        assert!(refusal.contains("超过上限 100"), "{refusal}");
        let quick = plan_blocks_under(
            &prepared.base,
            &dump,
            Expansion::QuickTier,
            default_item(),
            100,
        )
        .expect("54 个状态的快档排得了");
        assert_eq!(quick.state_count, 54);
        let reading = read_the_flow_under(&dump, &prepared.base, 100).expect("读得出");
        assert_eq!(
            (
                reading.row.full,
                reading.row.blocks_full,
                reading.row.blocks_quick.is_some(),
                reading.full.is_none(),
                reading.quick.is_some()
            ),
            (Some(16_777_268), None, true, true, true),
            "态数照报，全量不排计划写 not_planned，快档照排"
        );
        assert!(
            reading.row.body().contains(" blocks_full=not_planned "),
            "{}",
            reading.row.body()
        );
        let unrestricted = read_the_flow(&dump, &prepared.base).expect("读得出");
        assert_eq!(unrestricted.row.full, Some(16_777_268));
        assert!(unrestricted.row.blocks_full.is_some() && unrestricted.full.is_some());
    }

    #[test]
    fn the_round_trip_control_compares_the_keys_only_when_both_plans_fit_under_its_limit() {
        let prepared = prepared_new_pool_stream();
        let dump = prepared.dump();
        let compared = direct_base_control_under(&prepared, &dump, 1 << 24).expect("对得上");
        assert!(
            compared.passed && compared.detail.ends_with("keys_equal=true"),
            "{compared:?}"
        );
        let base_only = direct_base_control_under(&prepared, &dump, 100).expect("对得上");
        assert!(
            base_only.passed && base_only.detail.ends_with("keys_equal=not_planned"),
            "{base_only:?}"
        );
        assert_eq!(base_only.kind, "dump_round_trip_direct_base");
    }

    #[test]
    fn the_sixteen_unit_overwrite_chain_is_read_without_its_full_plan_and_with_its_quick_tier_plan()
    {
        let (flow, _overwrites) = prepared_overwrite_chain(16);
        let dump = flow.dump();
        let reading = read_the_flow(&dump, &flow.base).expect("键读得出，不因为块数而 Err");
        assert!(
            reading.row.longest_segment >= 40,
            "u = 16 的每次覆盖写有一个很长的单元段：{}",
            reading.row.longest_segment
        );
        assert_eq!(
            (reading.row.blocks_full, reading.full.is_none()),
            (None, true),
            "全量的块数超过上限（或态数装不进 u64），不排计划"
        );
        assert!(
            reading.quick.is_some() && reading.row.blocks_quick.is_some(),
            "快档照排：长段不展开，块数很少"
        );
    }

    #[test]
    fn the_new_pool_file_creation_stream_is_recorded_with_the_pinned_numbers() {
        let prepared = prepared_new_pool_stream();
        let readings = recorded_flow_readings(&prepared, &PINNED_BY_THE_TESTS[0]);
        assert!(
            readings.iter().all(|reading| reading.passed),
            "{readings:?}"
        );
        let dump = prepared.dump();
        assert_eq!((dump.writes.len(), dump.segments.len()), (43, 12));
        assert_eq!(
            (
                dump.family.as_str(),
                dump.tier.as_str(),
                dump.versions_with_a_file.len()
            ),
            ("smoke", "baseline", 1)
        );
    }

    #[test]
    fn the_multi_record_publish_stream_is_recorded_with_the_pinned_numbers() {
        let prepared = prepared_multi_record_stream();
        let readings = recorded_flow_readings(&prepared, &PINNED_BY_THE_TESTS[2]);
        assert!(
            readings.iter().all(|reading| reading.passed),
            "{readings:?}"
        );
        assert_eq!(
            prepared.versions_with_a_file.len(),
            3,
            "A、B、C 三版带文件的版本"
        );
        assert_eq!(prepared.writes.len(), 117);
    }

    #[test]
    fn the_fixed_script_stream_is_recorded_with_the_pinned_numbers() {
        let prepared = prepared_fixed_script_stream();
        let readings = recorded_flow_readings(&prepared, &PINNED_BY_THE_TESTS[1]);
        assert!(
            readings.iter().all(|reading| reading.passed),
            "{readings:?}"
        );
        assert_eq!(prepared.segments.len(), 80);
        assert_eq!(
            prepared.versions_with_a_file.len(),
            17,
            "脚本里 versions 的条数：A、B（2），写行与暖机三个（5），C（6），回退 D（7），五次覆盖写（12），抬 F 两个（14），E（15），卸载两个（17）"
        );
    }

    #[test]
    fn an_overwrite_chain_runs_until_a_thousand_writes_and_keeps_one_version_per_overwrite() {
        let (flow, overwrites) = prepared_overwrite_chain(16);
        assert!(
            flow.writes.len() >= WRITES_OF_A_THOUSAND_WRITE_STREAM,
            "写数 {} 不到 1000",
            flow.writes.len()
        );
        assert!(overwrites >= 1);
        assert_eq!(
            flow.versions_with_a_file.len(),
            overwrites + 1,
            "第一个文件一版，加每次覆盖写一版"
        );
        assert_eq!(flow.label, "overwrite_chain-16_units_per_overwrite");
        let capacity_of_one_data_unit = singlefs_core::unit::data_unit_payload_capacity();
        assert!(
            flow.versions_with_a_file[1].content.len() > capacity_of_one_data_unit,
            "u = 16 的每一版都占多个数据单元，一个单元装不下"
        );
        assert_eq!(
            flow.versions_with_a_file
                .last()
                .map(|version| version.content.clone()),
            Some(common_crash_points::content_needing(16, overwrites)),
            "最后一版的内容就是第 K 次的 content_needing(16, K)"
        );
        assert_eq!(flow.ending, format!("NotAHistory:overwrites={overwrites}"));
        assert_eq!(
            Some(flow.judged_root_index),
            flow.writes
                .iter()
                .rposition(|write| write.kind == StepKind::RootRecordFua)
        );
        let dump = flow.dump();
        assert_eq!(FlowDump::from_bytes(&dump.to_bytes()), Ok(dump.clone()));
        assert_eq!(
            dump.base_pool(),
            flow.base,
            "转储里的基镜像叠回去得到建好的池"
        );
    }

    #[test]
    fn the_fixed_flows_are_named_and_ordered_chains_first_then_the_four_smoke_flows() {
        let labels: Vec<String> = FixedFlowKind::all()
            .iter()
            .map(|kind| kind.label())
            .collect();
        assert_eq!(
            labels,
            vec![
                "overwrite_chain-01_units_per_overwrite",
                "overwrite_chain-04_units_per_overwrite",
                "overwrite_chain-16_units_per_overwrite",
                "smoke-new_pool_file_creation_stream",
                "smoke-overwrite_fixed_script_stream",
                "smoke-multi_record_publish_stream",
                "smoke-snapshot_creation_stream",
            ]
        );
        assert_eq!(OVERWRITE_CHAIN_UNITS_PER_OVERWRITE, [1, 4, 16]);
    }

    #[test]
    fn the_keys_command_writes_key_files_and_controls_for_a_dump_and_a_second_run_reuses_them() {
        let (dumps, keys) = (
            scratch_directory("keys-dumps"),
            scratch_directory("keys-out"),
        );
        let prepared = prepared_snapshot_creation_stream();
        let dump_path = dumps.join("smoke-snapshot_creation_stream.dump");
        prepared
            .dump()
            .write_atomically(&dump_path)
            .expect("写转储");
        let arguments: Vec<String> = [
            "--dump",
            dumps.to_str().expect("路径"),
            "--out",
            keys.to_str().expect("路径"),
            "--controls",
        ]
        .iter()
        .map(|word| (*word).to_string())
        .collect();
        assert_eq!(run_keys_command(&arguments), EXIT_CODE_OK);
        let full = KeysFile::from_text(
            &std::fs::read_to_string(keys.join("smoke-snapshot_creation_stream.keys"))
                .expect("全量块键文件"),
        )
        .expect("读得回");
        assert_eq!(
            (full.expansion, full.row.full, full.state_total()),
            (Expansion::Full, Some(16_842_815), 16_842_815)
        );
        let quick = KeysFile::from_text(
            &std::fs::read_to_string(keys.join("smoke-snapshot_creation_stream.quick.keys"))
                .expect("甲二块键文件"),
        )
        .expect("读得回");
        assert_eq!(
            (quick.expansion, Some(quick.state_total())),
            (Expansion::QuickTier, full.row.quick_tier)
        );
        let controls_text =
            std::fs::read_to_string(keys.join("smoke-snapshot_creation_stream.controls"))
                .expect("对照文件");
        assert_eq!(
            control_lines_of(&controls_text).map(|lines| lines.len()),
            Some(4)
        );
        match keys_of_one_dump(&dump_path, &keys, true) {
            KeysOutcome::Done {
                reused,
                controls,
                row,
                ..
            } => {
                assert!(reused, "写完的块键文件与对照整份跳过");
                assert_eq!(controls.len(), 4);
                assert!(controls.iter().all(|(_, passed)| *passed));
                assert_eq!(row.label, "smoke-snapshot_creation_stream");
            }
            KeysOutcome::Failed { message, .. } => panic!("第二趟不该失败：{message}"),
        }
    }

    #[test]
    fn the_record_fixed_chains_command_writes_seven_dumps_and_skips_them_the_second_time() {
        let out = scratch_directory("chains");
        let arguments = vec!["--out".to_string(), out.to_str().expect("路径").to_string()];
        assert_eq!(run_record_fixed_chains_command(&arguments), EXIT_CODE_OK);
        let dumps =
            files_with_suffix(&[out.to_path_buf()], DUMP_FILE_SUFFIX, None).expect("列得出");
        assert_eq!(dumps.len(), 7);
        for path in &dumps {
            let dump = FlowDump::read_from(path).expect("每份转储读得回");
            assert!(
                dump.judged_root_index.is_some() && !dump.writes.is_empty(),
                "{}",
                dump.label
            );
        }
        let chain = FlowDump::read_from(&out.join("overwrite_chain-04_units_per_overwrite.dump"))
            .expect("覆盖写链 units=4");
        assert!(
            chain.writes.len() >= WRITES_OF_A_THOUSAND_WRITE_STREAM
                && chain.ending.starts_with("NotAHistory:overwrites=")
        );
        assert_eq!(
            run_record_fixed_chains_command(&arguments),
            EXIT_CODE_OK,
            "已有写完的转储整份跳过"
        );
    }

    #[test]
    fn the_library_command_refuses_a_relative_library_path_before_touching_anything() {
        let keys = scratch_directory("library-keys");
        let arguments: Vec<String> = [
            "--keys",
            keys.to_str().expect("路径"),
            "--library",
            "relative/library",
        ]
        .iter()
        .map(|word| (*word).to_string())
        .collect();
        assert_eq!(run_library_command(&arguments), EXIT_CODE_USAGE);
    }

    #[test]
    fn the_own_source_file_name_points_at_this_very_source_in_the_source_directory_of_the_binaries()
    {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/bin")
            .join(OWN_SOURCE_FILE_NAME);
        let text = std::fs::read_to_string(&path).expect("preflight 要判的源文件读得到");
        assert_eq!(
            text,
            OWN_SOURCE_TEXT,
            "{} 就是编进来的这一份",
            path.display()
        );
        assert!(
            text.contains("// admission:") && text.contains("// run-condition:"),
            "文件头要有准入与运行条件"
        );
    }

    #[test]
    fn the_selfcheck_passes_every_stop_check_and_every_anchor_on_the_current_crates() {
        assert_eq!(
            run_selfcheck_command(&["unexpected".to_string()]),
            EXIT_CODE_USAGE
        );
        assert_eq!(
            run_selfcheck_command(&[]),
            EXIT_CODE_OK,
            "停机 T1–T4 与锚点 A1–A6 全过（B1 对不上是登记预计的，不算在内）"
        );
    }

    #[test]
    fn baseline_flow_among_the_candidates_is_ignored_and_a_candidate_among_the_comparison_set_is_dropped(
    ) {
        let (mut candidates, mut against) = share_fixture();
        let smoke_among_the_candidates = against[0].clone();
        candidates.push(smoke_among_the_candidates);
        let history_among_the_comparison_set = candidates[0].clone();
        against.push(history_among_the_comparison_set);
        let mut output = Output::new();
        assert_eq!(
            share_lines(
                candidates.iter().cloned().map(Ok),
                &against,
                &[],
                &mut output
            ),
            Ok(EXIT_CODE_OK)
        );
        assert!(line_with(
            &output,
            "E7RESULT name=share_ignored flow=smoke-snapshot_creation_stream "
        )
        .ends_with("reason=tier_is_baseline"));
        let closing = output.lines.last().expect("有收尾行");
        assert!(
            closing.contains("name=share_done candidates=5 against=1 controls_failed=0"),
            "候选还是 5 条、比较集还是 1 条：{closing}"
        );
        let first = line_with(
            &output,
            "E7RESULT name=share flow=random_history-broad-FourGibibytes-steps0100-seed00 ",
        );
        assert!(
            first.contains("shared_blocks=3 share_fraction=1.000000"),
            "{first}"
        );
    }

    #[test]
    fn the_full_state_count_pinned_for_the_medium_item_is_written_in_its_test_file() {
        assert!(number_appears_in(
            OVERWRITE_AND_UNLINK_TEST_TEXT,
            OVERWRITE_AND_UNLINK_FULL_STATES_PINNED_BY_THE_TEST
        ));
    }
}
