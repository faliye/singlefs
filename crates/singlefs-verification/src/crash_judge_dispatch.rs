//! GPU 判器的派活：把一条流打包好的表与草稿区放上一张卡、编内核、把一段状态分成几次派活交给卡算、回读结论。
//! 判法（内核原文、表怎么打包、派活参数与结论各是哪几个字、草稿区每个状态多大）在 `crate::crash_judge_gpu`；这一份只管怎么上卡
//! （「派活和判法拆出来最好」）。GPU 判器那一组判法单元从 [`GpuCrashJudge::on_card_with_the_given_dispatch_width`]、
//! `GpuCrashJudge::judge_one_dispatch` 这些入口按函数单元收闭包（`crate::judging_units_entries`）：上卡、派活与回读归位
//! （决定一个状态的结论落到哪里的那几个函数）改了，那一把键跟着变；这一份里够不着的函数改了不换键。
//! 一次派活多宽、草稿区开几条由调用方定好交给判器（[`DispatchWidthGivenToTheJudge`]）：定宽度、量显存额度的函数与常量
//! （[`states_in_flight_within`]、[`states_per_dispatch_within_the_scratch_memory`]、[`MAXIMUM_SCRATCH_LANES`]、
//! [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]）与派活宽度调度的起步（`DispatchWidthSchedule::starting_narrow`）不在那一组的闭包里。
//!
//! 一张卡上同时在算的就是一次派活里的状态。草稿区一条缓冲受这张卡的存储绑定上限管（wgpu 压到 2 GiB − 4，装几个状态照 `SCRATCH_WORDS` 算，`states_per_scratch_lane`）；
//! 一次派活要带更多状态，就每条缓冲各派一次活、同一个内核的几次派活排在同一趟里（它们碰的草稿与结论缓冲互不相同），卡一起算。
//! 5090 上量到的（固定脚本到回退、展开 16，131 195 态）：一次 256 / 2170 / 4340 / 8680 / 13020 个状态，
//! 每秒 9 405 / 75 360 / 139 970 / 241 726 / 310 100 态，每次派活 27 / 28 / 30 / 34 / 38 毫秒。
use std::ops::Range;

use crate::crash_amplification::STUCK_AT_THE_INDIVISIBLE_FLOOR_LINE_PREFIX;
use crate::crash_judge_gpu::{
    dispatch_words, pack_tables, persisted_mask_probe_kernel_source,
    verdict_was_written_by_both_kernels, JudgeKernelSources,
    CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT, CHECKER_SCAN_REBUILD_ENTRY_POINT,
    DISPATCH_BINDING, JUDGE_KERNEL_WORKGROUP_SIZE, KERNEL_ENTRY_POINT, PERSISTED_MASK_PROBE_WORDS,
    SCRATCH_BINDING, SCRATCH_WORDS, TABLES_BINDING, VERDICTS_BINDING, VERDICT_WORDS,
    VERDICT_WORD_NOT_WRITTEN,
};
use crate::crash_judge_tables::JudgeTables;
// 两样分两条 use：GPU 判器那一组的闭包只用得到设备丢失的记账，一条 use 一起引进的话封顶派活宽度的函数会跟着那条 use 进闭包
use crate::gpu_memory_ledger::states_in_one_dispatch_capped_by_the_state_count;
use crate::gpu_memory_ledger::CauseOfTheDeviceLoss;
use crate::gpu_unit_checks::{
    gpu_lifecycle_trace, gpu_work_trace, readback_mapping_callback,
    take_the_readback_after_the_wait_or_record_the_device_lost, words_to_bytes, GpuCard,
};
use crate::sha256::{sha256_digest, DIGEST_BYTES};

mod dispatch_width_rules;
mod judge_passes_and_their_submission_waits;
mod kernel_inlined_sizes;
mod persisted_mask_probe_on_a_card;

pub use dispatch_width_rules::*;
pub use judge_passes_and_their_submission_waits::*;
pub use kernel_inlined_sizes::*;
pub use persisted_mask_probe_on_a_card::*;

/// 草稿区最多分几条缓冲（32 条是 64 GiB，比今天任何一张卡的显存都大：条数实际由显存额度定）。
pub const MAXIMUM_SCRATCH_LANES: usize = 32;
/// 表与草稿区之外这个进程在卡上还要占的显存（MiB）：驱动的上下文、三个编好的内核、结论与回读缓冲。
/// 判器的草稿区按「这张卡的显存额度 − 表 − 这个数」定。5090 上量到的是 2326 MiB（草稿区从 241 MiB 到 12283 MiB 五档都是这个数），留到 2560。
pub const MEBIBYTES_LEFT_FOR_THE_DRIVER: u64 = 2560;
/// 还没量过拆不开的下限时，一次提交卡上最多算多久（秒）。提交在卡上跑的时候卡让不出来，跑得太久驱动会报上下文切换超时、把这条通道掐掉
/// （内核日志 Xid 109），程序从此等不到结果。状态的轻重事先不知道，所以一次派活从 [`FIRST_DISPATCH_STATES`] 个状态起步，
/// 每次派活按最久那一次提交的用时加宽或减半（[`dispatch_width_after_a_dispatch`]），涨满一条草稿缓冲才量宽度（[`calibration_goes_on`]）。
/// 宽度 1 那一次提交是拆不开的下限（[`indivisible_floor_after_a_dispatch`]；单个状态的一趟，恢复、走读这几趟不分片）。量过它之后，
/// 最久那一次提交许到驱动会掐的那条线 [`SUBMISSION_SECONDS_THE_DRIVER_MAY_CUT`]，不看下限多长（[`longest_tolerated_submission_seconds`]）：
/// 状态在卡上并行，加宽多半不加时，宽度涨上去卡才用得满。
pub const LONGEST_DISPATCH_SECONDS: f64 = 0.5;
/// 驱动会掐的那条线（秒）：一次提交在卡上算过它，就有被驱动报上下文切换超时（内核日志 Xid 109）掐掉通道的危险。
/// 这条线不是定值，掐不掐看那一刻有没有别的上下文要这张卡。量到的：同一个状态单次提交约 5.78 秒，在第二台的 5060 Ti 上算完，
/// 在本机同型卡上被掐（`records/2026-11-10-Xid109复盘.md`「经过」）；取 2 秒，不到被掐过的那一次的一半（推的：被掐的那一次本身多长没量过，
/// 5.78 秒是第二台上同一个状态的用时）。
pub const SUBMISSION_SECONDS_THE_DRIVER_MAY_CUT: f64 = 2.0;
/// 宽度停在拆不开的下限上、加不了宽，最久那一次提交又超过 [`LONGEST_DISPATCH_SECONDS`]，连着这么多次派活就报错、不再派
/// （[`dispatches_stuck_at_the_floor_after_a_dispatch`]）：一条流里零星几个重状态（连着两三个）不报；连着 8 个都这样是这一段状态单个就重到
/// 加宽要冒驱动掐通道的险，照这样一个一个派，每秒判不到一个状态，大的流要跑上几天。
pub const DISPATCHES_STUCK_AT_THE_FLOOR_BEFORE_REFUSING: u64 = 8;
/// 等卡算完最多等多久（秒）：到点还没算完就报错，不干等（通道被驱动掐掉之后结果永远不回来）。
/// 几条 GPU 对拍同卡并行时一次派活要排在别的用例后面：删除线三条同卡并行时一条等过了 120 秒（单跑那条整条 254 秒跑完）。
pub const LONGEST_WAIT_FOR_THE_CARD_SECONDS: u64 = 300;
/// 一个判器第一次派活带几个状态：这时还不知道状态有多重，带得多的那一次可能就撞驱动的超时，撞了就没有下一次可调。
const FIRST_DISPATCH_STATES: usize = 1;
/// 一次派活带的状态数最少几个（算得太久减半时的下限）。
const NARROWEST_DISPATCH: usize = 1;
/// 量宽度时，这一档每个状态的用时比前面最好那一档多出这么多倍就不再加宽。
pub const CALIBRATION_WORSE_THAN_THE_BEST: f64 = 1.25;
/// 每派多少次活重量一次宽度：状态变重变轻，拐点跟着挪。
pub const DISPATCHES_BETWEEN_CALIBRATIONS: u64 = 512;
/// 判器内核一条一条编的环境变量：值恰好是 `1` 时 `compile_the_judge_kernels` 那几条管线一条编完才编下一条
/// （驱动编大内核吃几 GiB 主机内存，内存紧的机器写它）；别的值与没设都照几条一起编。
/// 双机崩溃放量入口 `scripts/crash/crash-amplification-two-hosts.sh` 给多机配置 `GPU_KERNEL_SERIAL_COMPILE_HOSTS` 点名的那几台设的就是这个变量
/// （`serial_compile_environment=(` 那一行）。
pub const GPU_KERNELS_ONE_AT_A_TIME_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME";

/// 扫描内核源码里扫描那三组判后两组的入口（主入口 [`KERNEL_ENTRY_POINT`] 判前两判；`crash_judge_checker_scan.wgsl`「入口」那一节）：
/// I-5.4 那一趟、之后那七判那一趟。
pub const CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT: &str =
    "allocation_records_disjoint_main";
pub const CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT: &str =
    "allocation_generations_through_the_rollback_floor_raises_main";
/// 扫描内核源码里记账那一判拆成的四趟的入口：头、分片（每片一次派活）、决断、收尾。
pub const CHECKER_ACCOUNTING_HEAD_ENTRY_POINT: &str = "accounting_head_main";
pub const CHECKER_ACCOUNTING_SLICE_ENTRY_POINT: &str = "accounting_slice_main";
pub const CHECKER_ACCOUNTING_DECIDE_ENTRY_POINT: &str = "accounting_decide_main";
pub const CHECKER_ACCOUNTING_FINAL_ENTRY_POINT: &str = "accounting_final_main";
/// 记账分片那一趟派活参数的绑定号（扫描内核的 `accounting_slice_parameters`）：四个字，哪一轮、这一片罩的位置 low、high、补齐到 16 字节的一个 0。
/// 只有分片那一条管线的布局里有它（管线照入口用到的绑定建布局）。
pub const ACCOUNTING_SLICE_BINDING: u32 = 4;
/// 记账分片一轮的项按权重摊在几个位置上（扫描内核的 `C_ACCOUNTING_SLICE_POSITIONS` 同值，库单测钉两份相同）：一片罩 [low, high) 那一段，
/// 一轮的各片首尾相接、末片到这里。
pub const ACCOUNTING_SLICE_POSITIONS: u32 = 65_536;
/// 记账分片的片宽（几个位置）从多宽起步：一轮 64 片。整理夹着回退那一组比重最大那张给定状态表（落点 10124）按记录自述那一轮约 5.9 秒，
/// 每片约 0.09 秒（推的：两份分段探针量的秒数除以 64）。
pub const FIRST_ACCOUNTING_SLICE_WIDTH: u32 = ACCOUNTING_SLICE_POSITIONS / 64;
/// 片宽最窄几个位置（片算得太久减半的下限）。
pub const NARROWEST_ACCOUNTING_SLICE: u32 = 16;
/// 一轮最多交几片：都按最窄的片宽交也够罩满一轮。交满这么多片还没罩满的（派活这一侧的毛病），内核在收尾那一趟把这个状态记判不了、交 CPU。
pub const MOST_ACCOUNTING_SLICES_OF_A_ROUND: u32 =
    ACCOUNTING_SLICE_POSITIONS / NARROWEST_ACCOUNTING_SLICE;

/// 记账分片的两轮（扫描内核的 `C_SPAN_READING_AS_RECORDED`、`C_SPAN_READING_AS_THE_UNIT_CLASSES`，派活参数里「哪一轮」那个字就是它）：
/// 按记录自述的跨度数（每块盘，另判 I-3.21 每条引用），按单元那一类数（只数决断那一趟记了要再数一份的那几块盘）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AccountingRound {
    CountingAsRecorded,
    CountingAsTheUnitClasses,
}

impl AccountingRound {
    /// 一次派活里两轮的次序（两轮之间是决断那一趟）。
    const IN_THE_ORDER_OF_THE_DISPATCH: [Self; 2] =
        [Self::CountingAsRecorded, Self::CountingAsTheUnitClasses];

    /// 派活参数里「哪一轮」那个字，也是判器里这一轮那一份片宽、片数的下标。
    fn kernel_word(self) -> u32 {
        match self {
            Self::CountingAsRecorded => 0,
            Self::CountingAsTheUnitClasses => 1,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::CountingAsRecorded => 0,
            Self::CountingAsTheUnitClasses => 1,
        }
    }
}

/// 一次派活里记账分片怎么切。
#[derive(Clone, Copy, Debug)]
enum AccountingSlicePlan<'plan> {
    /// 照判器此刻这一轮的片宽切，每片等完按 [`accounting_slice_width_after_a_slice`] 调片宽（下一次派活接着用）。
    FollowTheSliceWidthsOfTheJudge,
    /// 两轮都照这几片的上端切（各片首尾相接、末片到 [`ACCOUNTING_SLICE_POSITIONS`]），不动判器的片宽：对拍「切几片判出来的都一样」用。
    TheseSliceEnds(&'plan [u32]),
}

/// 一次派活判完：每个状态的结论，与这一次里卡算得最久的那一次提交几秒（判器按它调一次派活带几个状态）。
struct OneDispatchJudged {
    verdicts: Vec<[u32; VERDICT_WORDS]>,
    seconds_of_the_longest_submission: f64,
}

/// [`GpuCrashJudge`] 一次派活交回的结论与它的两个用时：卡把这一次全部提交算完一共几秒（量宽度用）、最久的那一次提交几秒。
struct TimedDispatch {
    verdicts: Vec<[u32; VERDICT_WORDS]>,
    seconds: f64,
    seconds_of_the_longest_submission: f64,
}

/// [`PackedJudgeTables::to_bytes`] 的头：换了编码就换头。
const PACKED_JUDGE_TABLES_MAGIC: &[u8; 8] = b"SFJTAB01";

/// 一条流打包好的表：录入段打包一次、写进库，判段从库里读回、几张卡各上一份。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedJudgeTables {
    words: Vec<u32>,
    state_count: u64,
}

impl PackedJudgeTables {
    #[must_use]
    pub fn of(tables: &JudgeTables) -> Self {
        Self {
            words: pack_tables(&tables.words()),
            state_count: tables.state_count,
        }
    }

    /// 录入段写进库的字节：头 8 字节、状态数（8 字节小端）、每个字 4 字节小端。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = PACKED_JUDGE_TABLES_MAGIC.to_vec();
        bytes.extend_from_slice(&self.state_count.to_le_bytes());
        for word in &self.words {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    /// 从库里的字节读回；头不对、长度不是 4 的倍数交回 `None`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let rest = bytes.strip_prefix(PACKED_JUDGE_TABLES_MAGIC.as_slice())?;
        let (state_count, words) = rest.split_at_checked(8)?;
        let (chunks, remainder) = words.as_chunks::<4>();
        if !remainder.is_empty() {
            return None;
        }
        Some(Self {
            words: chunks
                .iter()
                .map(|chunk| u32::from_le_bytes(*chunk))
                .collect(),
            state_count: u64::from_le_bytes(state_count.try_into().ok()?),
        })
    }

    /// 这几张表的状态数。
    #[must_use]
    pub fn state_count(&self) -> u64 {
        self.state_count
    }

    /// 表上卡之后占多少显存（MiB，向上取整）。
    #[must_use]
    pub fn mebibytes(&self) -> u64 {
        u64::try_from(self.words.len() * 4)
            .expect("装得进 u64")
            .div_ceil(1024 * 1024)
    }
}

/// 从判定库里读回、核过摘要的一份打包判表：GPU 判器（[`GpuCrashJudge`]）只收它。字段私有，只有
/// [`PackedJudgeTablesReadBack::from_the_library_bytes`] 造得出（读库、写库那两步在判定库那一侧：`crash_library::read_packed_judge_tables_back`、
/// `crash_library::record_the_judge_tables_and_read_them_back`）：判表先入库、再读库，GPU 才照库里那一份算（卡怎么分活也照它）。
/// 这一份不引判定库，派活里没有存取布局。读它照读 [`PackedJudgeTables`]。
#[derive(Debug)]
pub struct PackedJudgeTablesReadBack {
    tables: PackedJudgeTables,
}

impl PackedJudgeTablesReadBack {
    /// 库里按 `digest` 存的那份字节：核摘要、解码。只给判定库那一侧读回时用。
    ///
    /// # Errors
    /// 字节的摘要对不上（库里那份被改过）；解不开。
    pub(crate) fn from_the_library_bytes(
        bytes: &[u8],
        digest: &[u8; DIGEST_BYTES],
    ) -> Result<Self, String> {
        if sha256_digest(bytes) != *digest {
            return Err("GPU 判表摘要对不上：库里那份坏了，重跑录入段".to_string());
        }
        let tables = PackedJudgeTables::from_bytes(bytes)
            .ok_or_else(|| "GPU 判表解不出（字节坏了或编码换了）：重跑录入段".to_string())?;
        Ok(Self { tables })
    }
}

impl std::ops::Deref for PackedJudgeTablesReadBack {
    type Target = PackedJudgeTables;

    fn deref(&self) -> &Self::Target {
        &self.tables
    }
}

// 对拍用例借卡判一条流（`judge_every_state_on_a_card_leased_from_the_pool`）要经对拍那一类的库：测试文件照这一份的路径调它，留在这里
// （对拍经判定库存取的那一段不在 GPU 判器那一组的入口里，引判定库不进判器的键）
use crate::crash_library::{judge_tables_through_the_pairing_library, StageOutcome};

/// 调用方定好交给判器的派活宽度（[`GpuCrashJudge::on_card_with_the_given_dispatch_width`]）：一条草稿缓冲装几个状态、
/// 一次派活最多几个状态（草稿区开后者除以前者向上取整那么多条缓冲），连同从窄处起步的派活宽度调度。
/// 判器照它开草稿区、切派活，自己不读卡的显存额度、不定宽度。
pub struct DispatchWidthGivenToTheJudge {
    states_per_lane: usize,
    states_per_dispatch: usize,
    dispatch_width_schedule: DispatchWidthSchedule,
}

impl DispatchWidthGivenToTheJudge {
    /// 照给的两个数：一条草稿缓冲装 `states_per_lane` 个状态，一次派活最多 `states_per_dispatch` 个状态。
    /// 一条缓冲的字节数要在判器那张卡的存储绑定上限以内（[`states_per_scratch_lane`] 照上限算的就在）。
    ///
    /// # Panics
    /// 两个数有一个是 0。
    #[must_use]
    pub fn of_lanes_holding(states_per_lane: usize, states_per_dispatch: usize) -> Self {
        assert!(
            states_per_lane >= 1 && states_per_dispatch >= 1,
            "一条草稿缓冲装 {states_per_lane} 个、一次派活带 {states_per_dispatch} 个：两个都至少 1"
        );
        Self {
            states_per_lane,
            states_per_dispatch,
            dispatch_width_schedule: DispatchWidthSchedule::starting_narrow(
                states_per_lane,
                states_per_dispatch,
            ),
        }
    }

    /// 照这张卡的显存额度定：一条草稿缓冲照这张卡的存储绑定上限装（[`states_per_scratch_lane`]）；一次派活的状态数被
    /// 这张卡的显存额度（[`GpuCard::memory_budget_mebibytes`] 扣掉表与 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]，[`states_in_flight_within`]）、
    /// 调用方要的 `requested_states_per_dispatch` 与这条流的状态数压住，至少 1（[`states_in_one_dispatch_capped_by_the_state_count`]）。
    #[must_use]
    pub fn within_the_memory_budget_of_the_card(
        card: &GpuCard,
        tables: &PackedJudgeTables,
        requested_states_per_dispatch: usize,
    ) -> Self {
        let limit = card.device().limits().max_storage_buffer_binding_size;
        Self::of_lanes_holding(
            states_per_scratch_lane(limit),
            states_in_one_dispatch_capped_by_the_state_count(
                states_in_flight_within(limit, card.memory_budget_mebibytes(), tables),
                requested_states_per_dispatch,
                tables.state_count,
            ),
        )
    }
}

/// 三份内核源码合起来的摘要：同一张卡上同一套源码只编一次（[`GpuCard::pipelines_compiled_once_on_this_device`]）的键。
/// 每份前面带它的标签与字节数，三份之间的边界挪了摘要跟着变。
fn kernel_sources_digest(kernel_sources: &JudgeKernelSources) -> [u8; DIGEST_BYTES] {
    let mut message = Vec::new();
    for (label, source) in [
        ("recovery", &kernel_sources.recovery),
        ("checker-walk", &kernel_sources.checker_walk),
        ("checker-scan", &kernel_sources.checker_scan),
    ] {
        message.extend_from_slice(format!("{label}\n{}\n", source.len()).as_bytes());
        message.extend_from_slice(source.as_bytes());
    }
    sha256_digest(&message)
}

/// 一张卡上装好表的判器。
pub struct GpuCrashJudge<'card> {
    card: &'card GpuCard,
    recovery_pipeline: wgpu::ComputePipeline,
    checker_walk_pipeline: wgpu::ComputePipeline,
    checker_scan_pipeline: wgpu::ComputePipeline,
    checker_scan_allocation_records_disjoint_pipeline: wgpu::ComputePipeline,
    checker_scan_allocation_generations_through_the_rollback_floor_raises_pipeline:
        wgpu::ComputePipeline,
    checker_accounting_head_pipeline: wgpu::ComputePipeline,
    checker_accounting_slice_pipeline: wgpu::ComputePipeline,
    checker_accounting_decide_pipeline: wgpu::ComputePipeline,
    checker_accounting_final_pipeline: wgpu::ComputePipeline,
    checker_references_and_snapshots_pipeline: wgpu::ComputePipeline,
    checker_scan_rebuild_pipeline: wgpu::ComputePipeline,
    tables: wgpu::Buffer,
    /// 草稿区，一条缓冲装 `states_per_lane` 个状态（最后一条可以不满），条数照一次派活带的状态数定。
    scratch_lanes: Vec<wgpu::Buffer>,
    state_count: u64,
    /// 一条草稿缓冲装几个状态（[`states_per_scratch_lane`]）。
    states_per_lane: usize,
    /// 一次派活最多几个状态（调用方给的 [`DispatchWidthGivenToTheJudge`]：草稿区按它开）。
    states_per_dispatch: usize,
    /// 此刻一次派活带几个状态、什么时候量宽度。
    dispatch_width_schedule: DispatchWidthSchedule,
    /// 到此刻为止带得最多的一次派活几个状态、卡算得最久的一次多久（纳秒）。
    widest_dispatch: std::sync::atomic::AtomicUsize,
    nanoseconds_of_the_longest_dispatch: std::sync::atomic::AtomicU64,
    /// 挂钟花在哪（纳秒，累加）：备缓冲与提交（CPU）、等卡算完、回读与拆结论（CPU）；派了几次活。
    nanoseconds_preparing: std::sync::atomic::AtomicU64,
    nanoseconds_waiting_for_the_card: std::sync::atomic::AtomicU64,
    nanoseconds_reading_back: std::sync::atomic::AtomicU64,
    dispatches: std::sync::atomic::AtomicU64,
    /// 各趟的提交账：每次派活收尾并一次（派活进门拿着 `one_dispatch_at_a_time`，这把锁上没有争抢）。
    submission_waits_of_the_passes: std::sync::Mutex<SubmissionWaitsOfThePasses>,
    /// 记账分片两轮各自此刻的片宽（几个位置，[`AccountingRound::index`] 为下标）：从 [`FIRST_ACCOUNTING_SLICE_WIDTH`] 起，每片等完按
    /// [`accounting_slice_width_after_a_slice`] 调，下一次派活接着用。两轮各一份：按单元那一类那一轮在多数状态上什么都不数、片片都快，
    /// 与按记录自述那一轮共用一份时会把片宽涨回一轮一片，下一次派活按记录自述那一轮头一片就整轮一次交。
    accounting_slice_width_of_each_round: [std::sync::atomic::AtomicU32; 2],
    /// 一次派活的几次提交之间草稿区要留着：同一个判器的两次派活不交错，派活进门先拿它。
    one_dispatch_at_a_time: std::sync::Mutex<()>,
}

/// 编一个内核（编之前先照 [`refuse_a_kernel_too_large_for_the_driver`] 量一遍）。冷编由驱动做（checker 内核几分钟）；
/// 驱动自己的盘上着色器缓存（NVIDIA 的 `~/.nv/GLCache`）管跨进程复用，
/// wgpu 的 `PipelineCache` 要 `unsafe` 装数据、本 crate `forbid(unsafe_code)`，不用它。
fn compute_pipeline(
    card: &GpuCard,
    label: &str,
    source: &str,
    entry_point: &str,
) -> wgpu::ComputePipeline {
    refuse_a_kernel_entry_point_too_large_for_the_driver(label, source, entry_point);
    let device = card.device();
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: None,
        module: &module,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

/// 判器的管线有几条：恢复、走读，扫描内核源码里扫描那三组判、记账那四趟、引用与快照、扫描重建。
const JUDGE_PIPELINES: usize = 11;

/// 在这张卡上编判器的 [`JUDGE_PIPELINES`] 条管线，按派活的次序交回：恢复、走读、扫描（主入口 [`KERNEL_ENTRY_POINT`]）、
/// I-5.4 那一趟（[`CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT`]）、之后那七判那一趟
/// （[`CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT`]）、记账头、分片、决断、收尾（`CHECKER_ACCOUNTING_*_ENTRY_POINT`）、
/// 引用与快照（[`CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT`]）、扫描重建（[`CHECKER_SCAN_REBUILD_ENTRY_POINT`]）。
/// 分五条线程编（驱动的编译是单线程的，checker 两个内核冷编各要几分钟；盘上缓存装好之后几秒）：恢复、走读各一条，扫描内核源码里扫描那三组、
/// 记账那四趟、引用与快照连同扫描重建各一条线程依次编——同时在编的仍是五条，扫描内核拆成几个入口之后每一条展开得比原来那一整条小。
/// 驱动编大内核吃内存（595 版三个一起编常驻 8.6 GiB，580 版一起编撞过 40G 上限）：内存紧的机器把
/// [`GPU_KERNELS_ONE_AT_A_TIME_ENVIRONMENT_VARIABLE`] 写成 `1`，一条一条编。
///
/// # Panics
/// 有一条管线超了展开预算（[`refuse_a_kernel_entry_point_too_large_for_the_driver`]）。
fn compile_the_judge_kernels(
    card: &GpuCard,
    kernel_sources: &JudgeKernelSources,
) -> [wgpu::ComputePipeline; JUDGE_PIPELINES] {
    let recovery = || {
        compute_pipeline(
            card,
            "crash-judge-recovery",
            &kernel_sources.recovery,
            KERNEL_ENTRY_POINT,
        )
    };
    let walk = || {
        compute_pipeline(
            card,
            "crash-judge-checker-walk",
            &kernel_sources.checker_walk,
            KERNEL_ENTRY_POINT,
        )
    };
    let checker_scan_entry = |label: &str, entry_point: &str| {
        compute_pipeline(card, label, &kernel_sources.checker_scan, entry_point)
    };
    let scan_judges = || {
        [
            checker_scan_entry("crash-judge-checker-scan", KERNEL_ENTRY_POINT),
            checker_scan_entry(
                "crash-judge-checker-scan-allocation-records-disjoint",
                CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT,
            ),
            checker_scan_entry(
                "crash-judge-checker-scan-allocation-generations-through-the-rollback-floor-raises",
                CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT,
            ),
        ]
    };
    let accounting_passes = || {
        [
            checker_scan_entry(
                "crash-judge-checker-accounting-head",
                CHECKER_ACCOUNTING_HEAD_ENTRY_POINT,
            ),
            checker_scan_entry(
                "crash-judge-checker-accounting-slice",
                CHECKER_ACCOUNTING_SLICE_ENTRY_POINT,
            ),
            checker_scan_entry(
                "crash-judge-checker-accounting-decide",
                CHECKER_ACCOUNTING_DECIDE_ENTRY_POINT,
            ),
            checker_scan_entry(
                "crash-judge-checker-accounting-final",
                CHECKER_ACCOUNTING_FINAL_ENTRY_POINT,
            ),
        ]
    };
    let references_and_snapshots_and_scan_rebuild = || {
        [
            checker_scan_entry(
                "crash-judge-checker-references-and-snapshots",
                CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT,
            ),
            checker_scan_entry(
                "crash-judge-checker-scan-rebuild",
                CHECKER_SCAN_REBUILD_ENTRY_POINT,
            ),
        ]
    };
    let in_the_order_of_the_dispatch =
        |recovery_pipeline: wgpu::ComputePipeline,
         walk_pipeline: wgpu::ComputePipeline,
         scan_judge_pipelines: [wgpu::ComputePipeline; 3],
         accounting_pipelines: [wgpu::ComputePipeline; 4],
         tail_pipelines: [wgpu::ComputePipeline; 2]| {
            let [scan, allocation_records_disjoint, allocation_generations_through_the_rollback_floor_raises] =
                scan_judge_pipelines;
            let [accounting_head, accounting_slice, accounting_decide, accounting_final] =
                accounting_pipelines;
            let [references_and_snapshots, scan_rebuild] = tail_pipelines;
            [
                recovery_pipeline,
                walk_pipeline,
                scan,
                allocation_records_disjoint,
                allocation_generations_through_the_rollback_floor_raises,
                accounting_head,
                accounting_slice,
                accounting_decide,
                accounting_final,
                references_and_snapshots,
                scan_rebuild,
            ]
        };
    let one_at_a_time = std::env::var(GPU_KERNELS_ONE_AT_A_TIME_ENVIRONMENT_VARIABLE)
        .is_ok_and(|value| value == "1");
    if one_at_a_time {
        return in_the_order_of_the_dispatch(
            recovery(),
            walk(),
            scan_judges(),
            accounting_passes(),
            references_and_snapshots_and_scan_rebuild(),
        );
    }
    std::thread::scope(|scope| {
        let recovery = scope.spawn(recovery);
        let scan_judges = scope.spawn(scan_judges);
        let accounting_passes = scope.spawn(accounting_passes);
        let references_and_snapshots_and_scan_rebuild =
            scope.spawn(references_and_snapshots_and_scan_rebuild);
        let walk = walk();
        in_the_order_of_the_dispatch(
            recovery.join().expect("恢复内核的编译线程不 panic"),
            walk,
            scan_judges.join().expect("扫描那三组判的编译线程不 panic"),
            accounting_passes
                .join()
                .expect("记账那四趟的编译线程不 panic"),
            references_and_snapshots_and_scan_rebuild
                .join()
                .expect("引用与快照、扫描重建那两截的编译线程不 panic"),
        )
    })
}

impl<'card> GpuCrashJudge<'card> {
    /// 把一条流从判定库读回的打包判表上卡、编内核（几张卡共用一份读回的）。一次派活的状态数照旧被这张卡的显存额度、
    /// 草稿缓冲的条数与这条流的状态数压住，至少 1（[`states_in_one_dispatch_capped_by_the_state_count`]）。
    ///
    /// # Panics
    /// 打包的表超过这张卡的存储缓冲绑定上限（表太大，GPU 判不了这条流）。
    #[must_use]
    pub fn on_card_with_packed_tables(
        card: &'card GpuCard,
        tables: &PackedJudgeTablesReadBack,
        requested_states_per_dispatch: usize,
    ) -> Self {
        Self::on_card_compiling_these_kernel_sources(
            card,
            tables,
            requested_states_per_dispatch,
            &JudgeKernelSources::of_the_judge(),
        )
    }

    /// 同 [`Self::on_card_with_packed_tables`]，编的是调用方给的三份内核源码；一次派活的宽度照这张卡的显存额度算好
    /// （[`DispatchWidthGivenToTheJudge::within_the_memory_budget_of_the_card`]）交给 [`Self::on_card_with_the_given_dispatch_width`]。
    /// 判定入库的路径只编一种别的源码：给定状态表那一栏旋钮值编进扫描内核（[`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`]，
    /// `crate::crash_amplification::GivenStatesEngine` 照表上那一栏编；旋钮值进那张表的块键「给定状态表加旋钮栏」）。
    /// 对拍用例另拿它在内核里种假违例位、判旋钮压小写出的镜像，那几份结论不入库。
    ///
    /// # Panics
    /// 同 [`Self::on_card_with_packed_tables`]。
    #[must_use]
    pub fn on_card_compiling_these_kernel_sources(
        card: &'card GpuCard,
        tables: &PackedJudgeTablesReadBack,
        requested_states_per_dispatch: usize,
        kernel_sources: &JudgeKernelSources,
    ) -> Self {
        Self::on_card_with_the_given_dispatch_width(
            card,
            tables,
            DispatchWidthGivenToTheJudge::within_the_memory_budget_of_the_card(
                card,
                tables,
                requested_states_per_dispatch,
            ),
            kernel_sources,
        )
    }

    /// 把打包判表上卡、编调用方给的三份内核源码，草稿区与派活宽度照调用方给的 `width_given_by_the_caller` 开：判器不读卡的显存额度、不定宽度。
    /// 判器的结论按 GPU 判器那一组判法单元的键入库（这个函数与 [`JudgeKernelSources::of_the_judge`] 都是那一组的入口）；
    /// 宽度只管一段序号切成几次派活，不进结论，定宽度的函数与常量不在那一组的闭包里。
    ///
    /// # Panics
    /// 同 [`Self::on_card_with_packed_tables`]。
    #[must_use]
    pub fn on_card_with_the_given_dispatch_width(
        card: &'card GpuCard,
        tables: &PackedJudgeTablesReadBack,
        width_given_by_the_caller: DispatchWidthGivenToTheJudge,
        kernel_sources: &JudgeKernelSources,
    ) -> Self {
        let packed = &tables.words;
        gpu_lifecycle_trace(
            card,
            "judge_resources_begin",
            format_args!("table_words={} states={}", packed.len(), tables.state_count),
        );
        let device = card.device();
        let table_bytes = u64::try_from(packed.len() * 4).expect("装得进 u64");
        let limit = device.limits().max_storage_buffer_binding_size;
        assert!(
            table_bytes <= limit,
            "判器的表 {table_bytes} 字节超过这张卡的绑定上限 {limit}：这条流 GPU 判不了"
        );
        let DispatchWidthGivenToTheJudge {
            states_per_lane,
            states_per_dispatch,
            dispatch_width_schedule,
        } = width_given_by_the_caller;
        let tables_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("crash-judge-tables"),
                contents: &words_to_bytes(packed),
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let scratch_lanes: Vec<wgpu::Buffer> = (0..states_per_dispatch.div_ceil(states_per_lane))
            .map(|lane| {
                let states_in_this_lane =
                    (states_per_dispatch - lane * states_per_lane).min(states_per_lane);
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("crash-judge-scratch"),
                    size: u64::try_from(states_in_this_lane * SCRATCH_WORDS * 4)
                        .expect("装得进 u64"),
                    usage: wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                })
            })
            .collect();
        // 同一张卡（同一个设备）上同一套源码只编一次：共用一张卡的几条用例里第一条编，别的等它编完直接拿
        let [recovery_pipeline, checker_walk_pipeline, checker_scan_pipeline, checker_scan_allocation_records_disjoint_pipeline, checker_scan_allocation_generations_through_the_rollback_floor_raises_pipeline, checker_accounting_head_pipeline, checker_accounting_slice_pipeline, checker_accounting_decide_pipeline, checker_accounting_final_pipeline, checker_references_and_snapshots_pipeline, checker_scan_rebuild_pipeline]: [wgpu::ComputePipeline;
            JUDGE_PIPELINES] = card
            .pipelines_compiled_once_on_this_device(kernel_sources_digest(kernel_sources), || {
                compile_the_judge_kernels(card, kernel_sources).to_vec()
            })
            .try_into()
            .expect("这个摘要那一格只由 compile_the_judge_kernels 填，里面恰好 JUDGE_PIPELINES 条管线");
        gpu_lifecycle_trace(
            card,
            "judge_resources_end",
            format_args!("states_per_dispatch={states_per_dispatch}"),
        );
        Self {
            card,
            recovery_pipeline,
            checker_walk_pipeline,
            checker_scan_pipeline,
            checker_scan_allocation_records_disjoint_pipeline,
            checker_scan_allocation_generations_through_the_rollback_floor_raises_pipeline,
            checker_accounting_head_pipeline,
            checker_accounting_slice_pipeline,
            checker_accounting_decide_pipeline,
            checker_accounting_final_pipeline,
            checker_references_and_snapshots_pipeline,
            checker_scan_rebuild_pipeline,
            tables: tables_buffer,
            scratch_lanes,
            state_count: tables.state_count,
            states_per_lane,
            states_per_dispatch,
            dispatch_width_schedule,
            widest_dispatch: std::sync::atomic::AtomicUsize::new(0),
            nanoseconds_of_the_longest_dispatch: std::sync::atomic::AtomicU64::new(0),
            nanoseconds_preparing: std::sync::atomic::AtomicU64::new(0),
            nanoseconds_waiting_for_the_card: std::sync::atomic::AtomicU64::new(0),
            nanoseconds_reading_back: std::sync::atomic::AtomicU64::new(0),
            dispatches: std::sync::atomic::AtomicU64::new(0),
            submission_waits_of_the_passes: std::sync::Mutex::new(
                SubmissionWaitsOfThePasses::default(),
            ),
            accounting_slice_width_of_each_round: [
                std::sync::atomic::AtomicU32::new(FIRST_ACCOUNTING_SLICE_WIDTH),
                std::sync::atomic::AtomicU32::new(FIRST_ACCOUNTING_SLICE_WIDTH),
            ],
            one_dispatch_at_a_time: std::sync::Mutex::new(()),
        }
    }

    /// 到此刻为止挂钟花在哪。
    #[must_use]
    pub fn time_spent(&self) -> GpuJudgeTimeSpent {
        let seconds = |counter: &std::sync::atomic::AtomicU64| {
            counter.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1e9
        };
        GpuJudgeTimeSpent {
            dispatches: self.dispatches.load(std::sync::atomic::Ordering::Relaxed),
            states_in_the_widest_dispatch: u64::try_from(
                self.widest_dispatch
                    .load(std::sync::atomic::Ordering::Relaxed),
            )
            .expect("装得进 u64"),
            seconds_of_the_longest_dispatch: seconds(&self.nanoseconds_of_the_longest_dispatch),
            seconds_preparing: seconds(&self.nanoseconds_preparing),
            seconds_waiting_for_the_card: seconds(&self.nanoseconds_waiting_for_the_card),
            seconds_reading_back: seconds(&self.nanoseconds_reading_back),
        }
    }

    /// 到此刻为止各趟的提交账（各趟等卡之和与 [`Self::time_spent`] 的 `seconds_waiting_for_the_card` 是同一笔）。
    #[must_use]
    pub fn submission_waits_of_the_passes(&self) -> SubmissionWaitsOfThePasses {
        self.submission_waits_of_the_passes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// 一次派活最多带几个状态（草稿区开了这么多；实际带几个在卡上现量，见 [`calibration_goes_on`]）。
    #[must_use]
    pub fn states_per_dispatch(&self) -> usize {
        self.states_per_dispatch
    }

    /// 此刻一次派活带几个状态（从 [`FIRST_DISPATCH_STATES`] 个起步，按用时调；量过宽度之后是量出来的那个）。
    #[must_use]
    pub fn dispatch_width(&self) -> usize {
        self.dispatch_width_schedule.width()
    }

    /// 这条流一共几个状态（序号从 0 起）。
    #[must_use]
    pub fn state_count(&self) -> u64 {
        self.state_count
    }

    /// 草稿区占这张卡多少显存（MiB）。
    #[must_use]
    pub fn scratch_mebibytes(&self) -> usize {
        self.states_per_dispatch * SCRATCH_WORDS * 4 / (1024 * 1024)
    }

    /// 判一段序号的状态，每个状态 [`VERDICT_WORDS`] 个字，按序号次序。
    ///
    /// # Panics
    /// 序号越过整条流的状态数；GPU 没写到某个状态（内核与派发对不上）；宽度停在拆不开的下限上
    /// （[`Self::judge_states_unless_stuck_at_the_indivisible_floor`] 报的错，消息是 [`StuckAtTheIndivisibleFloor::line`] 那一行）。
    #[must_use]
    pub fn judge_states(&self, ordinals: Range<u64>) -> Vec<[u32; VERDICT_WORDS]> {
        self.judge_states_unless_stuck_at_the_indivisible_floor(ordinals)
            .unwrap_or_else(|stuck| panic!("{}", stuck.line(None)))
    }

    /// 同 [`Self::judge_states`]，宽度停在拆不开的下限上时报错、不 panic（崩溃放量判段照它拒判、整趟报错）。
    ///
    /// # Errors
    /// 宽度停在拆不开的下限上连着 [`DISPATCHES_STUCK_AT_THE_FLOOR_BEFORE_REFUSING`] 次（[`StuckAtTheIndivisibleFloor`]）：之后的序号不再派。
    ///
    /// # Panics
    /// 序号越过整条流的状态数；GPU 没写到某个状态（内核与派发对不上）。
    pub fn judge_states_unless_stuck_at_the_indivisible_floor(
        &self,
        ordinals: Range<u64>,
    ) -> Result<Vec<[u32; VERDICT_WORDS]>, StuckAtTheIndivisibleFloor> {
        assert!(
            ordinals.end <= self.state_count,
            "序号段 {ordinals:?} 越过状态数 {}",
            self.state_count
        );
        self.dispatch_width_schedule.judge_ordinals(
            ordinals,
            &self.card.name,
            self.state_count,
            |one_dispatch| self.timed_dispatch(one_dispatch),
        )
    }

    /// 一次派活，交回结论与卡算了几秒（这一次里全部提交的等卡之和、最久的那一次提交）；记下带得最多的一次、算得最久的一次。
    fn timed_dispatch(&self, ordinals: Range<u64>) -> TimedDispatch {
        let relaxed = std::sync::atomic::Ordering::Relaxed;
        let waited_before = self.nanoseconds_waiting_for_the_card.load(relaxed);
        let states = usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize");
        let judged = self.judge_one_dispatch(
            ordinals,
            AccountingSlicePlan::FollowTheSliceWidthsOfTheJudge,
        );
        let nanoseconds = self.nanoseconds_waiting_for_the_card.load(relaxed) - waited_before;
        self.widest_dispatch.fetch_max(states, relaxed);
        self.nanoseconds_of_the_longest_dispatch
            .fetch_max(nanoseconds, relaxed);
        TimedDispatch {
            verdicts: judged.verdicts,
            seconds: nanoseconds as f64 / 1e9,
            seconds_of_the_longest_submission: judged.seconds_of_the_longest_submission,
        }
    }

    /// 一次派活判完一段状态，不按用时调宽度（量派活宽度与用时的关系用）；这一次卡算了多久计进 [`Self::time_spent`]。
    /// 记账分片照判器此刻的片宽切、照每片的用时调片宽。
    ///
    /// # Panics
    /// 状态数超过 [`Self::states_per_dispatch`]；同 [`Self::judge_states`]。
    #[must_use]
    pub fn judge_states_in_one_dispatch(&self, ordinals: Range<u64>) -> Vec<[u32; VERDICT_WORDS]> {
        self.judge_one_dispatch(
            ordinals,
            AccountingSlicePlan::FollowTheSliceWidthsOfTheJudge,
        )
        .verdicts
    }

    /// 同 [`Self::judge_states_in_one_dispatch`]，记账分片两轮都照 `ends_of_the_slices`（各片的上端，[0, [`ACCOUNTING_SLICE_POSITIONS`]) 里的位置）切，
    /// 不动判器的片宽：对拍「记账整轮一片判与切成很多片判，结论逐字相同」用。
    ///
    /// # Panics
    /// `ends_of_the_slices` 空着、多于 [`MOST_ACCOUNTING_SLICES_OF_A_ROUND`] 片、后一片的上端小于前一片的、末片不到 [`ACCOUNTING_SLICE_POSITIONS`]；
    /// 同 [`Self::judge_states_in_one_dispatch`]。
    #[must_use]
    pub fn judge_states_in_one_dispatch_slicing_the_accounting_at(
        &self,
        ordinals: Range<u64>,
        ends_of_the_slices: &[u32],
    ) -> Vec<[u32; VERDICT_WORDS]> {
        assert!(
            !ends_of_the_slices.is_empty()
                && u32::try_from(ends_of_the_slices.len())
                    .is_ok_and(|slices| slices <= MOST_ACCOUNTING_SLICES_OF_A_ROUND)
                && ends_of_the_slices
                    .windows(2)
                    .all(|neighbors| neighbors[0] <= neighbors[1])
                && ends_of_the_slices.last() == Some(&ACCOUNTING_SLICE_POSITIONS),
            "记账分片的各片上端 {ends_of_the_slices:?}：至少一片、至多 {MOST_ACCOUNTING_SLICES_OF_A_ROUND} 片、不往回走、末片到 {ACCOUNTING_SLICE_POSITIONS}"
        );
        self.judge_one_dispatch(
            ordinals,
            AccountingSlicePlan::TheseSliceEnds(ends_of_the_slices),
        )
        .verdicts
    }

    /// 一次派活判完一段状态：一趟一次提交、等它算完再交下一次——恢复、走读、扫描那三组判、记账头、按记录自述那一轮的各片、决断、按单元那一类那一轮的各片、
    /// 记账收尾、引用与快照、扫描重建，最后一次提交把结论拷回。后几趟读前几趟留在草稿区里的东西（走读内核的交接段、扫描内核几趟之间的交接段
    /// `C_ACCOUNTING_CARRY`），同一个状态每一趟用同一条草稿缓冲；一趟里每条草稿缓冲各派一次活，它们碰的草稿与结论缓冲互不相同、读的是同一份只读的表。
    /// 驱动掐的是卡上一段不可抢占的活：一次提交只带一趟（一片），宿主等完才交下一次，两次之间卡上没有这个上下文的活，别的判器、别的进程的提交插进来
    /// 不碍事（各用各的缓冲）。草稿区要在这几次提交之间留着，所以同一个判器的两次派活不交错（进门先拿 `one_dispatch_at_a_time`）。
    /// 记账分片怎么切照 `slices`；每片打一行 `CRASH_JUDGE_ACCOUNTING_SLICE`、这一次打一行 `CRASH_JUDGE_ACCOUNTING_SLICES`（测试框架照常收进输出）。
    fn judge_one_dispatch(
        &self,
        ordinals: Range<u64>,
        slices: AccountingSlicePlan<'_>,
    ) -> OneDispatchJudged {
        let count = usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize");
        assert!(count <= self.states_per_dispatch);
        if count == 0 {
            return OneDispatchJudged {
                verdicts: Vec::new(),
                seconds_of_the_longest_submission: 0.0,
            };
        }
        // 前一次派活 panic 在拿着它的时候（等卡没等成）：草稿区里那一次留下的东西下一次派活从恢复那一趟起全部重写，接着用
        let _one_dispatch_at_a_time = self
            .one_dispatch_at_a_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        gpu_lifecycle_trace(
            self.card,
            "dispatch_prepare_begin",
            format_args!("states={ordinals:?}"),
        );
        let dispatch_started = std::time::Instant::now();
        let device = self.card.device();
        // 这一次派活分几份上卡，一条草稿缓冲一份：每份是（第一个状态在这次派活里的序号、几个状态、它的草稿缓冲）
        let parts: Vec<(usize, usize, &wgpu::Buffer)> = (0..count.div_ceil(self.states_per_lane))
            .map(|lane| {
                let first = lane * self.states_per_lane;
                (
                    first,
                    (count - first).min(self.states_per_lane),
                    &self.scratch_lanes[lane],
                )
            })
            .collect();
        let dispatch_and_verdict_buffers: Vec<(wgpu::Buffer, wgpu::Buffer)> = parts
            .iter()
            .map(|(first, states, _)| {
                let dispatch = dispatch_words(
                    ordinals.start + u64::try_from(*first).expect("装得进 u64"),
                    *states,
                );
                (
                    wgpu::util::DeviceExt::create_buffer_init(
                        device,
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("crash-judge-dispatch"),
                            contents: &words_to_bytes(&dispatch),
                            usage: wgpu::BufferUsages::UNIFORM,
                        },
                    ),
                    wgpu::util::DeviceExt::create_buffer_init(
                        device,
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("crash-judge-verdicts"),
                            contents: &words_to_bytes(&vec![
                                VERDICT_WORD_NOT_WRITTEN;
                                states * VERDICT_WORDS
                            ]),
                            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                        },
                    ),
                )
            })
            .collect();
        let passes = PassesOfOneDispatch {
            ordinals: &ordinals,
            parts: &parts,
            dispatch_and_verdict_buffers: &dispatch_and_verdict_buffers,
        };
        let mut waits = SubmissionWaitsOfOneDispatch::default();
        // 几趟按次序：扫描内核源码里的几趟读走读内核留在草稿区交接段里的东西，后一趟接着前一趟的违例位与判到位记（扫描重建那一截要读另几截写下的违例位）
        for (pass, pipeline) in [
            (GpuJudgePass::Recovery, &self.recovery_pipeline),
            (GpuJudgePass::CheckerWalk, &self.checker_walk_pipeline),
            (GpuJudgePass::CheckerScan, &self.checker_scan_pipeline),
            (
                GpuJudgePass::CheckerScanAllocationRecordsDisjoint,
                &self.checker_scan_allocation_records_disjoint_pipeline,
            ),
            (
                GpuJudgePass::CheckerScanAllocationGenerationsThroughTheRollbackFloorRaises,
                &self
                    .checker_scan_allocation_generations_through_the_rollback_floor_raises_pipeline,
            ),
            (
                GpuJudgePass::CheckerAccountingHead,
                &self.checker_accounting_head_pipeline,
            ),
        ] {
            waits.note_a_submission(
                pass,
                self.submit_one_pass_and_wait(&passes, pass.name(), pipeline, None),
            );
        }
        let relaxed = std::sync::atomic::Ordering::Relaxed;
        let mut slices_of_each_round = [0u32; 2];
        let mut nanoseconds_of_the_longest_slice = 0u64;
        for round in AccountingRound::IN_THE_ORDER_OF_THE_DISPATCH {
            let mut low = 0u32;
            while low < ACCOUNTING_SLICE_POSITIONS
                && slices_of_each_round[round.index()] < MOST_ACCOUNTING_SLICES_OF_A_ROUND
            {
                let high = match slices {
                    AccountingSlicePlan::FollowTheSliceWidthsOfTheJudge => low
                        .saturating_add(
                            self.accounting_slice_width_of_each_round[round.index()].load(relaxed),
                        )
                        .min(ACCOUNTING_SLICE_POSITIONS),
                    AccountingSlicePlan::TheseSliceEnds(ends_of_the_slices) => {
                        ends_of_the_slices[usize::try_from(slices_of_each_round[round.index()])
                            .expect("片数小于一轮的片数上限，装得进 usize")]
                    }
                };
                let parameters = wgpu::util::DeviceExt::create_buffer_init(
                    device,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("crash-judge-accounting-slice"),
                        contents: &words_to_bytes(&[round.kernel_word(), low, high, 0]),
                        usage: wgpu::BufferUsages::UNIFORM,
                    },
                );
                let nanoseconds = self.submit_one_pass_and_wait(
                    &passes,
                    GpuJudgePass::CheckerAccountingSlice.name(),
                    &self.checker_accounting_slice_pipeline,
                    Some(&parameters),
                );
                waits.note_a_submission(GpuJudgePass::CheckerAccountingSlice, nanoseconds);
                nanoseconds_of_the_longest_slice =
                    nanoseconds_of_the_longest_slice.max(nanoseconds);
                let seconds = nanoseconds as f64 / 1e9;
                eprintln!(
                    "CRASH_JUDGE_ACCOUNTING_SLICE card={:?} first_state={} states={count} round={round:?} slice={} low={low} high={high} seconds={seconds:.3}",
                    self.card.name,
                    ordinals.start,
                    slices_of_each_round[round.index()]
                );
                if matches!(slices, AccountingSlicePlan::FollowTheSliceWidthsOfTheJudge) {
                    let width = &self.accounting_slice_width_of_each_round[round.index()];
                    width.store(
                        accounting_slice_width_after_a_slice(width.load(relaxed), seconds),
                        relaxed,
                    );
                }
                low = high;
                slices_of_each_round[round.index()] += 1;
            }
            // 决断那一趟在按记录自述那一轮罩满之后、按单元那一类那一轮之前
            if round == AccountingRound::CountingAsRecorded {
                waits.note_a_submission(
                    GpuJudgePass::CheckerAccountingDecide,
                    self.submit_one_pass_and_wait(
                        &passes,
                        GpuJudgePass::CheckerAccountingDecide.name(),
                        &self.checker_accounting_decide_pipeline,
                        None,
                    ),
                );
            }
        }
        for (pass, pipeline) in [
            (
                GpuJudgePass::CheckerAccountingFinal,
                &self.checker_accounting_final_pipeline,
            ),
            (
                GpuJudgePass::CheckerReferencesAndSnapshots,
                &self.checker_references_and_snapshots_pipeline,
            ),
            (
                GpuJudgePass::CheckerScanRebuild,
                &self.checker_scan_rebuild_pipeline,
            ),
        ] {
            waits.note_a_submission(
                pass,
                self.submit_one_pass_and_wait(&passes, pass.name(), pipeline, None),
            );
        }
        let result_bytes = u64::try_from(count * VERDICT_WORDS * 4).expect("装得进 u64");
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("crash-judge-staging"),
            size: result_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        for ((first, states, _), (_, verdicts)) in parts.iter().zip(&dispatch_and_verdict_buffers) {
            encoder.copy_buffer_to_buffer(
                verdicts,
                0,
                &staging,
                u64::try_from(first * VERDICT_WORDS * 4).expect("装得进 u64"),
                u64::try_from(states * VERDICT_WORDS * 4).expect("装得进 u64"),
            );
        }
        gpu_lifecycle_trace(
            self.card,
            "queue_submit_begin",
            format_args!("states={ordinals:?} pass=readback"),
        );
        let submission = self.card.queue().submit(std::iter::once(encoder.finish()));
        gpu_lifecycle_trace(
            self.card,
            "queue_submit_end",
            format_args!("states={ordinals:?} pass=readback"),
        );
        let waiting_started = std::time::Instant::now();
        let slice = staging.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, readback_mapping_callback(sender));
        // 只等这一次提交：几条用例共用这张卡的设备时，等整个设备会把别的用例的活算进这一次的用时（量宽度、超时都跟着偏）
        gpu_lifecycle_trace(
            self.card,
            "submission_wait_begin",
            format_args!("states={ordinals:?} pass=readback"),
        );
        let polled = device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(std::time::Duration::from_secs(
                LONGEST_WAIT_FOR_THE_CARD_SECONDS,
            )),
        });
        gpu_lifecycle_trace(
            self.card,
            "submission_wait_end",
            format_args!("states={ordinals:?} pass=readback result={polled:?}"),
        );
        // 等不成：这张卡的设备记成丢了（下一条用例借这张卡时重起），只这一次派活红
        take_the_readback_after_the_wait_or_record_the_device_lost(
            self.card.device_condition(),
            polled,
            &receiver,
            |error| {
                format!(
                    "等了 {LONGEST_WAIT_FOR_THE_CARD_SECONDS} 秒，卡 {:?} 没把这次派活（{count} 个状态，序号 {} 起）的结论拷回：{error:?}。\
                     多半是卡上跑得太久被驱动掐了通道（看内核日志里的 Xid 109）",
                    self.card.name, ordinals.start
                )
            },
        );
        let reading_started = std::time::Instant::now();
        waits.note_a_submission(
            GpuJudgePass::Readback,
            nanoseconds_of(reading_started - waiting_started),
        );
        let mapped = slice.get_mapped_range().expect("读得到回读缓冲区");
        let (entries, remainder) = mapped.as_chunks::<4>();
        assert!(remainder.is_empty(), "回读缓冲区的长度是 4 的整数倍");
        assert_eq!(entries.len(), count * VERDICT_WORDS, "回读的字数与派的相同");
        let mut out = Vec::with_capacity(count);
        for state in 0..count {
            let mut words = [0u32; VERDICT_WORDS];
            for (word, chunk) in words
                .iter_mut()
                .zip(&entries[state * VERDICT_WORDS..(state + 1) * VERDICT_WORDS])
            {
                *word = u32::from_le_bytes(*chunk);
            }
            assert!(
                verdict_was_written_by_both_kernels(&words),
                "状态 {} 没被两个内核都写到（结论仍是初值）",
                ordinals.start + u64::try_from(state).expect("装得进")
            );
            out.push(words);
        }
        drop(mapped);
        staging.unmap();
        let nanoseconds_before_the_readback = nanoseconds_of(reading_started - dispatch_started);
        let nanoseconds_waited = waits.nanoseconds_waiting_for_the_card();
        self.nanoseconds_preparing.fetch_add(
            nanoseconds_before_the_readback.saturating_sub(nanoseconds_waited),
            relaxed,
        );
        self.nanoseconds_waiting_for_the_card
            .fetch_add(nanoseconds_waited, relaxed);
        self.nanoseconds_reading_back
            .fetch_add(nanoseconds_of(reading_started.elapsed()), relaxed);
        self.dispatches.fetch_add(1, relaxed);
        self.submission_waits_of_the_passes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .add_a_dispatch(
                u64::try_from(count).expect("一次派活的状态数装得进 u64"),
                &waits,
            );
        let seconds_of_the_longest_submission =
            waits.nanoseconds_of_the_longest_submission() as f64 / 1e9;
        eprintln!(
            "CRASH_JUDGE_ACCOUNTING_SLICES card={:?} first_state={} states={count} slices_counting_as_recorded={} slices_counting_as_the_unit_classes={} \
             longest_slice_seconds={:.3} longest_submission_seconds={seconds_of_the_longest_submission:.3} slice_width_counting_as_recorded={} \
             slice_width_counting_as_the_unit_classes={}",
            self.card.name,
            ordinals.start,
            slices_of_each_round[AccountingRound::CountingAsRecorded.index()],
            slices_of_each_round[AccountingRound::CountingAsTheUnitClasses.index()],
            nanoseconds_of_the_longest_slice as f64 / 1e9,
            self.accounting_slice_width_of_each_round[AccountingRound::CountingAsRecorded.index()]
                .load(relaxed),
            self.accounting_slice_width_of_each_round
                [AccountingRound::CountingAsTheUnitClasses.index()]
            .load(relaxed)
        );
        gpu_lifecycle_trace(
            self.card,
            "dispatch_readback_end",
            format_args!("states={ordinals:?}"),
        );
        OneDispatchJudged {
            verdicts: out,
            seconds_of_the_longest_submission,
        }
    }

    /// 把一趟（`pipeline`）在这次派活的每条草稿缓冲上各派一次活，交一次提交、等它算完，交回等了几纳秒。分片那一趟另绑派活参数
    /// （`accounting_slice_parameters`，第 [`ACCOUNTING_SLICE_BINDING`] 号），别的趟交 `None`。等不成：这张卡的设备记成丢了（下一条用例借这张卡时重起）、
    /// panic，只这一次派活红（同拷回结论那一次）。
    ///
    /// # Panics
    /// 等不成（超时、设备丢了）。
    fn submit_one_pass_and_wait(
        &self,
        passes: &PassesOfOneDispatch<'_>,
        pass_name: &str,
        pipeline: &wgpu::ComputePipeline,
        accounting_slice_parameters: Option<&wgpu::Buffer>,
    ) -> u64 {
        let device = self.card.device();
        let ordinals = passes.ordinals;
        let layout = pipeline.get_bind_group_layout(0);
        let bind_groups: Vec<wgpu::BindGroup> = passes
            .parts
            .iter()
            .zip(passes.dispatch_and_verdict_buffers)
            .map(|((_, _, scratch), (dispatch_buffer, verdicts))| {
                let mut entries = vec![
                    wgpu::BindGroupEntry {
                        binding: DISPATCH_BINDING,
                        resource: dispatch_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: TABLES_BINDING,
                        resource: self.tables.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: VERDICTS_BINDING,
                        resource: verdicts.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: SCRATCH_BINDING,
                        resource: scratch.as_entire_binding(),
                    },
                ];
                if let Some(parameters) = accounting_slice_parameters {
                    entries.push(wgpu::BindGroupEntry {
                        binding: ACCOUNTING_SLICE_BINDING,
                        resource: parameters.as_entire_binding(),
                    });
                }
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("crash-judge"),
                    layout: &layout,
                    entries: &entries,
                })
            })
            .collect();
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            for ((_, states, _), bind_group) in passes.parts.iter().zip(&bind_groups) {
                pass.set_bind_group(0, bind_group, &[]);
                pass.dispatch_workgroups(
                    u32::try_from(states.div_ceil(JUDGE_KERNEL_WORKGROUP_SIZE))
                        .expect("工作组数装得进 u32"),
                    1,
                    1,
                );
            }
        }
        gpu_lifecycle_trace(
            self.card,
            "queue_submit_begin",
            format_args!("states={ordinals:?} pass={pass_name}"),
        );
        let submission = self.card.queue().submit(std::iter::once(encoder.finish()));
        gpu_lifecycle_trace(
            self.card,
            "queue_submit_end",
            format_args!("states={ordinals:?} pass={pass_name}"),
        );
        let waiting_started = std::time::Instant::now();
        gpu_lifecycle_trace(
            self.card,
            "submission_wait_begin",
            format_args!("states={ordinals:?} pass={pass_name}"),
        );
        let polled = device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(std::time::Duration::from_secs(
                LONGEST_WAIT_FOR_THE_CARD_SECONDS,
            )),
        });
        let nanoseconds = nanoseconds_of(waiting_started.elapsed());
        gpu_lifecycle_trace(
            self.card,
            "submission_wait_end",
            format_args!("states={ordinals:?} pass={pass_name} result={polled:?}"),
        );
        if let Err(error) = polled {
            let failure = format!(
                "等了 {LONGEST_WAIT_FOR_THE_CARD_SECONDS} 秒，卡 {:?} 没把这次派活的 {pass_name} 那一趟（{} 个状态，序号 {} 起）算完：{error:?}。\
                 多半是单次派活跑得太久被驱动掐了通道（看内核日志里的 Xid 109）；记账分片的片宽按每片的用时调，一次派活带的状态数按 longest_tolerated_submission_seconds 调",
                self.card.name,
                ordinals.end - ordinals.start,
                ordinals.start
            );
            let condition = self.card.device_condition();
            if !condition.is_lost() {
                condition.record_the_device_lost(
                    format!("等卡没等成：{failure}"),
                    CauseOfTheDeviceLoss::Other,
                );
            }
            panic!("{failure}");
        }
        nanoseconds
    }
}

/// 一次派活带几个状态的调度：怎么把一段序号切成一次次派活。只管切，不碰卡：派活交给调用方，结论按序号次序接回，切法不进结论。
/// 从 [`FIRST_DISPATCH_STATES`] 个起步，每次派活之后按 [`dispatch_width_after_a_dispatch`] 调；宽度到了一条草稿缓冲才量宽度。
struct DispatchWidthSchedule {
    /// 一条草稿缓冲装几个状态（[`states_per_scratch_lane`]）。
    states_per_lane: usize,
    /// 一次派活最多几个状态（调用方给的 [`DispatchWidthGivenToTheJudge`]：草稿区按它开）。
    capacity: usize,
    /// 此刻一次派活带几个状态。
    width: std::sync::atomic::AtomicUsize,
    /// 量过宽度没有；上次量宽度之后派了几次活（到 [`DISPATCHES_BETWEEN_CALIBRATIONS`] 重量）。
    calibrated: std::sync::atomic::AtomicBool,
    dispatches_since_calibration: std::sync::atomic::AtomicU64,
    /// 量宽度时试到过、过了拐点的最窄那一档（[`knee_after_calibration`]）：以后量宽度只试比它窄的；没撞过是 `usize::MAX`。
    knee: std::sync::atomic::AtomicUsize,
    /// 拆不开的下限与宽度停在下限上连着几次：两样一起读、一起改。
    floor_watch: std::sync::Mutex<IndivisibleFloorWatch>,
    /// 经这个调度判过几个状态（停在下限上报错时报）。
    states_judged: std::sync::atomic::AtomicU64,
}

/// [`DispatchWidthSchedule`] 记着的拆不开的下限（[`indivisible_floor_after_a_dispatch`]）与宽度停在下限上连着几次
/// （[`dispatches_stuck_at_the_floor_after_a_dispatch`]）。
struct IndivisibleFloorWatch {
    /// 调宽度只看量没量过（[`longest_tolerated_submission_seconds`]）；秒数进 `CRASH_JUDGE_DISPATCH_WIDTH` 行。
    seconds_of_the_indivisible_floor: Option<f64>,
    dispatches_stuck_in_a_row: u64,
}

impl DispatchWidthSchedule {
    fn starting_narrow(states_per_lane: usize, capacity: usize) -> Self {
        Self {
            states_per_lane,
            capacity,
            width: std::sync::atomic::AtomicUsize::new(FIRST_DISPATCH_STATES),
            calibrated: std::sync::atomic::AtomicBool::new(false),
            dispatches_since_calibration: std::sync::atomic::AtomicU64::new(0),
            knee: std::sync::atomic::AtomicUsize::new(usize::MAX),
            floor_watch: std::sync::Mutex::new(IndivisibleFloorWatch {
                seconds_of_the_indivisible_floor: None,
                dispatches_stuck_in_a_row: 0,
            }),
            states_judged: std::sync::atomic::AtomicU64::new(0),
        }
    }

    fn width(&self) -> usize {
        self.width.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 把 `ordinals` 切成一次次派活交给 `timed_dispatch`，结论按序号次序接起来。每个序号恰好判一次。
    /// `card_name`、`state_count`（这条流一共几个状态）只进打的行与报的错。
    ///
    /// # Errors
    /// 宽度停在拆不开的下限上连着 [`DISPATCHES_STUCK_AT_THE_FLOOR_BEFORE_REFUSING`] 次（[`dispatches_stuck_at_the_floor_after_a_dispatch`]）：
    /// 停在那一次派活之后，之后的序号不再派。
    fn judge_ordinals(
        &self,
        ordinals: Range<u64>,
        card_name: &str,
        state_count: u64,
        mut timed_dispatch: impl FnMut(Range<u64>) -> TimedDispatch,
    ) -> Result<Vec<[u32; VERDICT_WORDS]>, StuckAtTheIndivisibleFloor> {
        let mut out = Vec::with_capacity(
            usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize"),
        );
        let relaxed = std::sync::atomic::Ordering::Relaxed;
        let lane = self.states_per_lane.min(self.capacity);
        let mut start = ordinals.start;
        while start < ordinals.end {
            let left = usize::try_from(ordinals.end - start).expect("装得进 usize");
            let width_now = self.width.load(relaxed);
            let calibration_is_due = !self.calibrated.load(relaxed)
                || self.dispatches_since_calibration.load(relaxed)
                    >= DISPATCHES_BETWEEN_CALIBRATIONS;
            let widest_to_try = self
                .capacity
                .min(self.knee.load(relaxed).saturating_sub(1))
                .max(lane);
            // 量宽度头一档就带一条缓冲：宽度还没涨到一条缓冲时（更窄的宽度上已经算得不快）不量
            if calibration_is_due
                && width_now >= lane
                && left >= 2 * lane
                && widest_to_try >= 2 * lane
            {
                // 量宽度：同一段起点一档加一条缓冲地派；最宽那一档判的状态包住了前面几档，它的结论照用
                let seconds_the_longest_submission_may_take = longest_tolerated_submission_seconds(
                    self.floor_watch
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .seconds_of_the_indivisible_floor
                        .is_some(),
                );
                let mut samples: Vec<(usize, f64)> = Vec::new();
                let widest_verdicts = loop {
                    let width = ((samples.len() + 1) * lane).min(widest_to_try).min(left);
                    let timed =
                        timed_dispatch(start..start + u64::try_from(width).expect("装得进 u64"));
                    samples.push((width, timed.seconds));
                    if width == left
                        || !calibration_goes_on(
                            &samples,
                            timed.seconds_of_the_longest_submission,
                            widest_to_try,
                            seconds_the_longest_submission_may_take,
                        )
                    {
                        break timed.verdicts;
                    }
                };
                let chosen = calibrated_width(&samples);
                eprintln!(
                    "CRASH_JUDGE_DISPATCH_CALIBRATED card={card_name:?} first_state={start} samples={} chosen_states={chosen} capacity={}",
                    samples
                        .iter()
                        .map(|(width, seconds)| format!("{width}:{seconds:.3}"))
                        .collect::<Vec<_>>()
                        .join(","),
                    self.capacity
                );
                self.width.store(chosen, relaxed);
                self.knee.store(
                    knee_after_calibration(&samples, self.knee.load(relaxed)),
                    relaxed,
                );
                self.calibrated.store(true, relaxed);
                self.dispatches_since_calibration.store(0, relaxed);
                self.states_judged.fetch_add(
                    u64::try_from(widest_verdicts.len()).expect("装得进 u64"),
                    relaxed,
                );
                start += u64::try_from(widest_verdicts.len()).expect("装得进 u64");
                out.extend(widest_verdicts);
                continue;
            }
            let end = (start + u64::try_from(width_now).expect("装得进")).min(ordinals.end);
            let timed = timed_dispatch(start..end);
            out.extend(timed.verdicts);
            self.dispatches_since_calibration.fetch_add(1, relaxed);
            // 驱动掐的是卡上一段不可抢占的活：一次派活拆成几次提交之后比的是最久的那一次提交（整次的用时随片数变长，不说明哪一段跑得太久）
            let states_carried = usize::try_from(end - start).expect("装得进 usize");
            let mut floor_watch = self
                .floor_watch
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let seconds_of_the_indivisible_floor = indivisible_floor_after_a_dispatch(
                floor_watch.seconds_of_the_indivisible_floor,
                states_carried,
                timed.seconds_of_the_longest_submission,
            );
            let next = dispatch_width_after_a_dispatch(
                width_now,
                states_carried,
                timed.seconds_of_the_longest_submission,
                self.states_per_lane,
                self.capacity,
                seconds_of_the_indivisible_floor.is_some(),
            );
            let dispatches_stuck_in_a_row = dispatches_stuck_at_the_floor_after_a_dispatch(
                floor_watch.dispatches_stuck_in_a_row,
                states_carried,
                timed.seconds_of_the_longest_submission,
                next,
            );
            floor_watch.seconds_of_the_indivisible_floor = seconds_of_the_indivisible_floor;
            floor_watch.dispatches_stuck_in_a_row = dispatches_stuck_in_a_row;
            drop(floor_watch);
            if next != width_now
                || timed.seconds_of_the_longest_submission
                    > longest_tolerated_submission_seconds(
                        seconds_of_the_indivisible_floor.is_some(),
                    )
            {
                eprintln!(
                    "CRASH_JUDGE_DISPATCH_WIDTH card={card_name:?} states={states_carried} seconds={:.3} seconds_of_the_dispatch={:.3} next_states={next} capacity={} \
                     seconds_of_the_indivisible_floor={} dispatches_stuck_at_the_floor={dispatches_stuck_in_a_row}",
                    timed.seconds_of_the_longest_submission,
                    timed.seconds,
                    self.capacity,
                    seconds_of_the_indivisible_floor
                        .map_or_else(|| "-".to_string(), |floor_seconds| format!("{floor_seconds:.3}"))
                );
            }
            self.width.store(next, relaxed);
            let states_judged = self
                .states_judged
                .fetch_add(u64::try_from(states_carried).expect("装得进 u64"), relaxed)
                + u64::try_from(states_carried).expect("装得进 u64");
            start = end;
            if dispatches_stuck_in_a_row >= DISPATCHES_STUCK_AT_THE_FLOOR_BEFORE_REFUSING {
                return Err(StuckAtTheIndivisibleFloor {
                    card_name: card_name.to_string(),
                    seconds_of_the_floor: timed.seconds_of_the_longest_submission,
                    dispatches_in_a_row: dispatches_stuck_in_a_row,
                    next_ordinal: start,
                    states_judged_by_the_judge: states_judged,
                    state_count,
                });
            }
        }
        Ok(out)
    }
}

/// 一次派活里每一趟都要的那几样：这一次派的序号段、每条草稿缓冲那一份（第一个状态的序号、几个状态、草稿缓冲）与它的派活参数、结论缓冲。
struct PassesOfOneDispatch<'dispatch> {
    ordinals: &'dispatch Range<u64>,
    parts: &'dispatch [(usize, usize, &'dispatch wgpu::Buffer)],
    dispatch_and_verdict_buffers: &'dispatch [(wgpu::Buffer, wgpu::Buffer)],
}

/// 一段挂钟的纳秒数。
///
/// # Panics
/// 超过 u64 装得下的纳秒数（五百多年）。
fn nanoseconds_of(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_nanos()).expect("一次派活的纳秒数装得进 u64")
}

/// GPU 对拍用例在卡池里借一张卡判一条流的每个状态，按序号次序交回每个状态的结论字：表打包、先入库再读库（项名是用例名，
/// [`judge_tables_through_the_pairing_library`]），一次派活带一条草稿缓冲那么多个状态
/// （按池里绑定上限最小的那张卡估，与这条流的状态数取小，[`mebibytes_a_judge_needs`] 按它借），判完先丢判器、再还卡。
/// 此刻借不到时借卡那一层排队（`crate::gpu_unit_checks::gpu_test_start::CardLeaseWaitingRoom`），没有可等的才打「本次未跑」那一行，这里交 `None`（调用的用例当场返回）；只录那一趟同样交 `None`、打一行 `JUDGE_TABLES_RECORDED_ONLY`。
///
/// # Panics
/// 同 [`crate::gpu_unit_checks::gpu_test_start::GpuCardPool::lease_card_or_report_not_run`]、[`judge_tables_through_the_pairing_library`] 与 [`GpuCrashJudge::on_card_with_packed_tables`]。
#[must_use]
pub fn judge_every_state_on_a_card_leased_from_the_pool(
    cards: crate::gpu_unit_checks::gpu_test_start::GpuCardPoolStartedAtTheFirstLease,
    test_name: &str,
    tables: &JudgeTables,
) -> Option<Vec<[u32; VERDICT_WORDS]>> {
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    gpu_work_trace("library_begin", format_args!("test={test_name:?}"));
    let packed =
        match judge_tables_through_the_pairing_library(test_name, &PackedJudgeTables::of(tables)) {
            StageOutcome::Judged(packed) => packed,
            StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => {
                gpu_work_trace("library_record_only", format_args!("test={test_name:?}"));
                println!("JUDGE_TABLES_RECORDED_ONLY test={test_name}");
                return None;
            }
        };
    gpu_work_trace("library_end", format_args!("test={test_name:?}"));
    let states_per_dispatch =
        states_per_scratch_lane(cards.smallest_maximum_storage_buffer_binding_size());
    let lease = cards.lease_card_or_report_not_run(
        test_name,
        mebibytes_a_judge_needs(&packed, states_per_dispatch),
    )?;
    gpu_lifecycle_trace(
        lease.card(),
        "lease_acquired",
        format_args!("test={test_name:?}"),
    );
    let judge =
        GpuCrashJudge::on_card_with_packed_tables(lease.card(), &packed, states_per_dispatch);
    let verdicts = judge.judge_states(0..tables.state_count);
    gpu_lifecycle_trace(
        lease.card(),
        "judge_buffers_drop_begin",
        format_args!("test={test_name:?}"),
    );
    drop(judge);
    gpu_lifecycle_trace(
        lease.card(),
        "judge_buffers_drop_end",
        format_args!("test={test_name:?}"),
    );
    drop(lease);
    Some(verdicts)
}

#[cfg(test)]
mod tests;
