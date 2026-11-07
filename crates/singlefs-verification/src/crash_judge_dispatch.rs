//! GPU 判器的派活：把一条流打包好的表与草稿区放上一张卡、编内核、把一段状态分成几次派活交给卡算、回读结论。
//! 判法（内核原文、表怎么打包、派活参数与结论各是哪几个字、草稿区每个状态多大）在 `crate::crash_judge_gpu`；这一份只管怎么上卡
//! （「派活和判法拆出来最好」）。GPU 判器那一组判法单元从 [`GpuCrashJudge::judge_states`] 这些入口按函数单元收闭包
//! （`crate::judging_units_entries`）：上卡、派活与回读归位（决定一个状态的结论落到哪里的那几个函数）改了，那一把键跟着变；
//! 这一份里够不着的函数改了不换键。
//!
//! 一张卡上同时在算的就是一次派活里的状态。草稿区一条缓冲受这张卡的存储绑定上限管（wgpu 压到 2 GiB − 4，装几个状态照 `SCRATCH_WORDS` 算，`states_per_scratch_lane`）；
//! 一次派活要带更多状态，就每条缓冲各派一次活、同一个内核的几次派活排在同一趟里（它们碰的草稿与结论缓冲互不相同），卡一起算。
//! 5090 上量到的（固定脚本到回退、展开 16，131 195 态）：一次 256 / 2170 / 4340 / 8680 / 13020 个状态，
//! 每秒 9 405 / 75 360 / 139 970 / 241 726 / 310 100 态，每次派活 27 / 28 / 30 / 34 / 38 毫秒。
use std::ops::Range;

use crate::crash_judge_gpu::{
    dispatch_words, pack_tables, persisted_mask_probe_kernel_source,
    verdict_was_written_by_both_kernels, JudgeKernelSources,
    CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT, CHECKER_SCAN_REBUILD_ENTRY_POINT,
    DISPATCH_BINDING, JUDGE_KERNEL_WORKGROUP_SIZE, KERNEL_ENTRY_POINT, PERSISTED_MASK_PROBE_WORDS,
    SCRATCH_BINDING, SCRATCH_WORDS, TABLES_BINDING, VERDICTS_BINDING, VERDICT_WORDS,
    VERDICT_WORD_NOT_WRITTEN,
};
use crate::crash_judge_tables::JudgeTables;
use crate::gpu_memory_ledger::{
    states_in_one_dispatch_capped_by_the_state_count, CauseOfTheDeviceLoss,
};
use crate::gpu_unit_checks::{
    gpu_lifecycle_trace, gpu_work_trace, readback_mapping_callback,
    take_the_readback_after_the_wait_or_record_the_device_lost, words_to_bytes, GpuCard,
};
use crate::sha256::{sha256_digest, DIGEST_BYTES};

/// 草稿区最多分几条缓冲（32 条是 64 GiB，比今天任何一张卡的显存都大：条数实际由显存额度定）。
pub const MAXIMUM_SCRATCH_LANES: usize = 32;
/// 表与草稿区之外这个进程在卡上还要占的显存（MiB）：驱动的上下文、三个编好的内核、结论与回读缓冲。
/// 判器的草稿区按「这张卡的显存额度 − 表 − 这个数」定。5090 上量到的是 2326 MiB（草稿区从 241 MiB 到 12283 MiB 五档都是这个数），留到 2560。
pub const MEBIBYTES_LEFT_FOR_THE_DRIVER: u64 = 2560;
/// 一次派活卡上最多算多久（秒）。派活在卡上跑的时候卡让不出来，跑得太久驱动会报上下文切换超时、把这条通道掐掉
/// （内核日志 Xid 109；固定脚本到 E 那条流一次派约三万个状态时撞过，程序从此等不到结果）。一次派活带几个状态按它调。
pub const LONGEST_DISPATCH_SECONDS: f64 = 0.5;
/// 等卡算完最多等多久（秒）：到点还没算完就报错，不干等（通道被驱动掐掉之后结果永远不回来）。
/// 几条 GPU 对拍同卡并行时一次派活要排在别的用例后面：删除线三条同卡并行时一条等过了 120 秒（单跑那条整条 254 秒跑完），
/// 从 120 抬到 300。
pub const LONGEST_WAIT_FOR_THE_CARD_SECONDS: u64 = 300;
/// 一次派活带的状态数最少几个（算得太久减半时的下限）。
const NARROWEST_DISPATCH: usize = 256;
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

/// 量宽度：同一段状态从一条草稿缓冲那么宽起，一档加一条缓冲地派，`samples` 是量过的各档（带几个状态、卡算了几秒）。
/// 下一档还量不量：最新一档已经到这张卡带得了的上限、单次超过 [`LONGEST_DISPATCH_SECONDS`] 的一半、
/// 或每个状态的用时比前面最好那一档多出 [`CALIBRATION_WORSE_THAN_THE_BEST`] 倍，就不量了。
/// 一次派活带的状态多到一定程度，每个状态的用时猛涨（5090 过了 21 700 个、5060 Ti 过了 6 510 个，那时显存远没用满；
/// 拐点随状态轻重挪），所以宽度在卡上现量，不按显存算。
#[must_use]
pub fn calibration_goes_on(samples: &[(usize, f64)], widest_possible: usize) -> bool {
    let Some((&(width, seconds), earlier)) = samples.split_last() else {
        return true;
    };
    if width >= widest_possible || seconds > LONGEST_DISPATCH_SECONDS / 2.0 {
        return false;
    }
    let best_earlier = earlier
        .iter()
        .map(|(earlier_width, earlier_seconds)| earlier_seconds / *earlier_width as f64)
        .fold(f64::INFINITY, f64::min);
    seconds / width as f64 <= best_earlier * CALIBRATION_WORSE_THAN_THE_BEST
}

/// 量过的各档里每个状态用时最短的那一档带几个状态。
///
/// # Panics
/// 一档都没量。
#[must_use]
pub fn calibrated_width(samples: &[(usize, f64)]) -> usize {
    samples
        .iter()
        .min_by(|(left_width, left_seconds), (right_width, right_seconds)| {
            (left_seconds / *left_width as f64).total_cmp(&(right_seconds / *right_width as f64))
        })
        .expect("至少量了一档")
        .0
}

/// 量完一次之后，以后量宽度最宽试到几个状态之前：最宽那一档不是选中的那一档（它每个状态更慢、或单次太久），
/// 以后就不再试到它——过了拐点的那一档一次能慢到几秒（第二台的 5060 Ti 上 8 680 个要 3.6 秒），每次重量都试它既费时又可能撞驱动的超时。
#[must_use]
pub fn knee_after_calibration(samples: &[(usize, f64)], knee: usize) -> usize {
    let widest = samples.last().expect("至少量了一档").0;
    if widest == calibrated_width(samples) {
        knee
    } else {
        knee.min(widest)
    }
}

/// 一次派活算得超过 [`LONGEST_DISPATCH_SECONDS`]（驱动会掐掉跑得太久的派活，内核日志 Xid 109）：下一次减半，
/// 不少于 [`NARROWEST_DISPATCH`]（容量比它还小时取容量）。
#[must_use]
pub fn width_after_a_slow_dispatch(width: usize, capacity: usize) -> usize {
    (width / 2).clamp(NARROWEST_DISPATCH.min(capacity).max(1), capacity.max(1))
}

/// 记账分片交完、等完一片（卡算了 `seconds` 秒）之后下一片的片宽（几个位置）：多于 [`LONGEST_DISPATCH_SECONDS`] 的四分之一（0.125 秒）减半，
/// 不窄过 [`NARROWEST_ACCOUNTING_SLICE`]；少于它的十六分之一（0.03125 秒）加倍，不宽过 [`ACCOUNTING_SLICE_POSITIONS`]；中间不动。
/// 门槛比每片的上限 0.25 秒（它的一半）低一倍：片宽只在一片算完之后才调，相邻几片每个位置的用时能差到两倍多
/// （落点 10124 那张表上照 0.25 秒那一档调，2048 个位置的一片从 0.184 秒跳到 0.283 秒）。轻的流头一次派活六片之内涨到一轮一片。
#[must_use]
pub fn accounting_slice_width_after_a_slice(width: u32, seconds: f64) -> u32 {
    if seconds > LONGEST_DISPATCH_SECONDS / 4.0 {
        (width / 2).max(NARROWEST_ACCOUNTING_SLICE)
    } else if seconds < LONGEST_DISPATCH_SECONDS / 16.0 {
        width.saturating_mul(2).min(ACCOUNTING_SLICE_POSITIONS)
    } else {
        width
    }
}

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

/// 一张卡按给它的显存（MiB）一次派活带得了几个状态：表与留给驱动的那一份先扣掉，剩下的装草稿区。
/// 本机的卡传它的 [`GpuCard::memory_budget_mebibytes`]；别的机器上的卡按配置的额度算（绑定上限按本机的卡估）。
#[must_use]
pub fn states_in_flight_within(
    max_storage_buffer_binding_size: u64,
    memory_mebibytes: u64,
    tables: &PackedJudgeTables,
) -> usize {
    let scratch_memory_mebibytes = memory_mebibytes
        .saturating_sub(tables.mebibytes())
        .saturating_sub(MEBIBYTES_LEFT_FOR_THE_DRIVER);
    states_per_dispatch(max_storage_buffer_binding_size, scratch_memory_mebibytes)
}

/// 一个判器在卡上要占多少显存（MiB，向上取整），不算 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`] 那一份（那一份每张卡只付一次，归卡池的账）：
/// 表，加一次派活带 `requested_states_per_dispatch` 与这条流状态数里小的那个（至少 1）的草稿区，
/// 加每次派活的结论与回读缓冲（每个状态各 [`VERDICT_WORDS`] 个字）。用例向卡池借卡时按它借
/// （`crate::gpu_unit_checks::gpu_test_start::GpuCardPool::lease_card_or_report_not_run`，此刻借不到排队等这个进程别的用例还卡，没有可等的才报本次未跑）；
/// 借来的卡额度是它加那一份，判器照 [`states_in_flight_within`] 开出来的草稿区不少于这里算进去的状态数。
#[must_use]
pub fn mebibytes_a_judge_needs(
    tables: &PackedJudgeTables,
    requested_states_per_dispatch: usize,
) -> u64 {
    let states = u64::try_from(states_in_one_dispatch_capped_by_the_state_count(
        usize::MAX,
        requested_states_per_dispatch,
        tables.state_count,
    ))
    .expect("装得进 u64");
    let bytes_per_state =
        u64::try_from(SCRATCH_WORDS * 4 + 2 * VERDICT_WORDS * 4).expect("装得进 u64");
    tables.mebibytes() + (states * bytes_per_state).div_ceil(1024 * 1024)
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
    /// 一次派活最多几个状态（[`states_per_dispatch`]：草稿区按它开）。
    states_per_dispatch: usize,
    /// 此刻一次派活带几个状态：从一条草稿缓冲那么多起步，在卡上现量（[`calibration_goes_on`]），算得太久减半。
    dispatch_width: std::sync::atomic::AtomicUsize,
    /// 量过宽度没有；上次量宽度之后派了几次活（到 [`DISPATCHES_BETWEEN_CALIBRATIONS`] 重量）。
    calibrated: std::sync::atomic::AtomicBool,
    dispatches_since_calibration: std::sync::atomic::AtomicU64,
    /// 量宽度时试到过、过了拐点的最窄那一档（[`knee_after_calibration`]）：以后量宽度只试比它窄的；没撞过是 `usize::MAX`。
    knee: std::sync::atomic::AtomicUsize,
    /// 到此刻为止带得最多的一次派活几个状态、卡算得最久的一次多久（纳秒）。
    widest_dispatch: std::sync::atomic::AtomicUsize,
    nanoseconds_of_the_longest_dispatch: std::sync::atomic::AtomicU64,
    /// 挂钟花在哪（纳秒，累加）：备缓冲与提交（CPU）、等卡算完、回读与拆结论（CPU）；派了几次活。
    nanoseconds_preparing: std::sync::atomic::AtomicU64,
    nanoseconds_waiting_for_the_card: std::sync::atomic::AtomicU64,
    nanoseconds_reading_back: std::sync::atomic::AtomicU64,
    dispatches: std::sync::atomic::AtomicU64,
    /// 记账分片两轮各自此刻的片宽（几个位置，[`AccountingRound::index`] 为下标）：从 [`FIRST_ACCOUNTING_SLICE_WIDTH`] 起，每片等完按
    /// [`accounting_slice_width_after_a_slice`] 调，下一次派活接着用。两轮各一份：按单元那一类那一轮在多数状态上什么都不数、片片都快，
    /// 与按记录自述那一轮共用一份时会把片宽涨回一轮一片，下一次派活按记录自述那一轮头一片就整轮一次交。
    accounting_slice_width_of_each_round: [std::sync::atomic::AtomicU32; 2],
    /// 一次派活的几次提交之间草稿区要留着：同一个判器的两次派活不交错，派活进门先拿它。
    one_dispatch_at_a_time: std::sync::Mutex<()>,
}

/// 判器的挂钟花在哪：卡在算的那一段之外都是 CPU 这一侧的。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GpuJudgeTimeSpent {
    pub dispatches: u64,
    /// 带得最多的一次派活几个状态。
    pub states_in_the_widest_dispatch: u64,
    /// 卡算得最久的一次派活多久。
    pub seconds_of_the_longest_dispatch: f64,
    pub seconds_preparing: f64,
    pub seconds_waiting_for_the_card: f64,
    pub seconds_reading_back: f64,
}

/// 一条草稿缓冲装几个状态：每个状态 `SCRATCH_WORDS × 4` 字节，一条不超过这张卡的存储绑定上限；至少 1。
#[must_use]
pub fn states_per_scratch_lane(max_storage_buffer_binding_size: u64) -> usize {
    let per_state = u64::try_from(SCRATCH_WORDS * 4).expect("装得进 u64");
    usize::try_from(max_storage_buffer_binding_size / per_state)
        .unwrap_or(usize::MAX)
        .max(1)
}

/// 一张卡一次派活带几个状态：草稿区不超过给它的显存（MiB），也不超过 [`MAXIMUM_SCRATCH_LANES`] 条缓冲装得下的；至少 1。
#[must_use]
pub fn states_per_dispatch(
    max_storage_buffer_binding_size: u64,
    scratch_memory_mebibytes: u64,
) -> usize {
    let per_state = u64::try_from(SCRATCH_WORDS * 4).expect("装得进 u64");
    let by_memory =
        usize::try_from(scratch_memory_mebibytes * 1024 * 1024 / per_state).unwrap_or(usize::MAX);
    let by_lanes = states_per_scratch_lane(max_storage_buffer_binding_size) * MAXIMUM_SCRATCH_LANES;
    by_memory.min(by_lanes).max(1)
}

/// 一个内核全部调用逐处展开之后最多多少个表达式（naga 中间表示里的表达式个数，[`kernel_inlined_size`]）。
/// GPU 程序没有真正的函数调用，驱动的编译器把每个调用点原地展开成被调函数的一份拷贝，编译的用时与内存随展开后的大小涨得比正比快：
/// 走读内核的「走一遍根」有六个调用点时（展开之后 98 万个表达式），第二台 580 版驱动在 48G 里编不完；改成一个调用点之后
/// （22 万）那台 94 秒、3.4 GiB 编完，恢复与扫描两个内核（各 27 万）在那台也编得过。预算取已知编得过的最大那个的一倍半左右，
/// 编不编得过的真实边界在 27 万与 98 万之间，没量过。内核超出预算先压展开量，不抬预算。
pub const LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL: u64 = 400_000;

/// 一个内核展开之后多大、谁占得最多。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelInlinedSize {
    /// 入口函数展开之后的表达式个数。
    pub expressions: u64,
    /// 展开后占得最多的几个函数：名字、被展开几份、自己几个表达式，按「份数 × 自己的表达式数」从大到小。
    pub largest_contributors: Vec<(String, u64, u64)>,
}

/// 量一份内核源码全部调用逐处展开之后多大：函数自己的表达式数，加上它每个调用点上被调函数展开后的表达式数（WGSL 不许递归，调用关系无环）。
/// 量的是入口 `main`（[`KERNEL_ENTRY_POINT`]）；一份源码里的别的入口用 [`kernel_inlined_size_of_the_entry_point`]。
///
/// # Panics
/// 源码解析不了（内核源码自己的单测先判这一条）。
#[must_use]
pub fn kernel_inlined_size(source: &str) -> KernelInlinedSize {
    kernel_inlined_size_of_the_entry_point(source, KERNEL_ENTRY_POINT)
}

/// 同 [`kernel_inlined_size`]，量的是名叫 `entry_point` 的那个入口（扫描内核源码里的第二个入口，引用与快照那一截，
/// [`CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT`]）：驱动按入口编一条管线，展开预算各量各的。
///
/// # Panics
/// 源码解析不了；源码里没有这个入口。
#[must_use]
pub fn kernel_inlined_size_of_the_entry_point(
    source: &str,
    entry_point: &str,
) -> KernelInlinedSize {
    use std::collections::HashMap;
    use wgpu::naga::{Block, Function, Handle, Statement};
    #[allow(
        clippy::wildcard_enum_match_arm,
        reason = "naga 的语句是外部的枚举，只有这四种里还套着语句块，其余一律不含调用"
    )]
    fn calls_in(block: &Block, calls: &mut Vec<Handle<Function>>) {
        for statement in block.iter() {
            match statement {
                Statement::Call { function, .. } => calls.push(*function),
                Statement::Block(inner) => calls_in(inner, calls),
                Statement::If { accept, reject, .. } => {
                    calls_in(accept, calls);
                    calls_in(reject, calls);
                }
                Statement::Switch { cases, .. } => {
                    for case in cases {
                        calls_in(&case.body, calls);
                    }
                }
                Statement::Loop {
                    body, continuing, ..
                } => {
                    calls_in(body, calls);
                    calls_in(continuing, calls);
                }
                _ => {}
            }
        }
    }
    fn inlined(
        function: Handle<Function>,
        own: &HashMap<Handle<Function>, u64>,
        calls: &HashMap<Handle<Function>, Vec<Handle<Function>>>,
        memo: &mut HashMap<Handle<Function>, u64>,
    ) -> u64 {
        if let Some(size) = memo.get(&function) {
            return *size;
        }
        let size = own[&function]
            + calls[&function]
                .iter()
                .map(|callee| inlined(*callee, own, calls, memo))
                .sum::<u64>();
        memo.insert(function, size);
        size
    }
    let module = wgpu::naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("内核解析失败：\n{}", error.emit_to_string(source)));
    let entry = module
        .entry_points
        .iter()
        .find(|entry| entry.name == entry_point)
        .unwrap_or_else(|| panic!("内核源码里有入口 {entry_point}"));
    let mut own: HashMap<Handle<Function>, u64> = HashMap::new();
    let mut calls: HashMap<Handle<Function>, Vec<Handle<Function>>> = HashMap::new();
    for (handle, function) in module.functions.iter() {
        own.insert(
            handle,
            u64::try_from(function.expressions.len()).expect("装得进 u64"),
        );
        let mut sites = Vec::new();
        calls_in(&function.body, &mut sites);
        calls.insert(handle, sites);
    }
    let mut entry_calls = Vec::new();
    calls_in(&entry.function.body, &mut entry_calls);
    let mut memo = HashMap::new();
    let expressions = u64::try_from(entry.function.expressions.len()).expect("装得进 u64")
        + entry_calls
            .iter()
            .map(|callee| inlined(*callee, &own, &calls, &mut memo))
            .sum::<u64>();
    // 每个函数被展开几份：从入口往下，调用方的份数乘以调用点数，按「先调用方、后被调方」的次序累加
    let mut order: Vec<Handle<Function>> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    fn postorder(
        function: Handle<Function>,
        calls: &HashMap<Handle<Function>, Vec<Handle<Function>>>,
        seen: &mut std::collections::HashSet<Handle<Function>>,
        order: &mut Vec<Handle<Function>>,
    ) {
        if !seen.insert(function) {
            return;
        }
        for callee in &calls[&function] {
            postorder(*callee, calls, seen, order);
        }
        order.push(function);
    }
    for callee in &entry_calls {
        postorder(*callee, &calls, &mut seen, &mut order);
    }
    let mut copies: HashMap<Handle<Function>, u64> = HashMap::new();
    for callee in &entry_calls {
        *copies.entry(*callee).or_insert(0) += 1;
    }
    for function in order.iter().rev() {
        let copies_of_caller = copies.get(function).copied().unwrap_or(0);
        for callee in &calls[function] {
            *copies.entry(*callee).or_insert(0) += copies_of_caller;
        }
    }
    let mut largest_contributors: Vec<(String, u64, u64)> = order
        .iter()
        .map(|function| {
            (
                module.functions[*function].name.clone().unwrap_or_default(),
                copies[function],
                own[function],
            )
        })
        .collect();
    largest_contributors.sort_by_key(|(_, copies_of_function, own_expressions)| {
        std::cmp::Reverse(copies_of_function * own_expressions)
    });
    largest_contributors.truncate(5);
    KernelInlinedSize {
        expressions,
        largest_contributors,
    }
}

/// 内核展开之后超过 [`LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL`] 就拒绝，不交给驱动去编（量入口 `main`）。
///
/// # Panics
/// 超过预算：消息里给展开份数最多的几个函数。
pub fn refuse_a_kernel_too_large_for_the_driver(label: &str, source: &str) {
    refuse_a_kernel_entry_point_too_large_for_the_driver(label, source, KERNEL_ENTRY_POINT);
}

/// 同 [`refuse_a_kernel_too_large_for_the_driver`]，量的是名叫 `entry_point` 的那个入口。
///
/// # Panics
/// 超过预算；源码里没有这个入口。
pub fn refuse_a_kernel_entry_point_too_large_for_the_driver(
    label: &str,
    source: &str,
    entry_point: &str,
) {
    let size = kernel_inlined_size_of_the_entry_point(source, entry_point);
    assert!(
        size.expressions <= LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL,
        "内核 {label} 全部调用展开之后 {} 个表达式，超过预算 {}：驱动的编译器把每个调用点展开成一份拷贝，\
         编译用时与内存随之猛涨（第二台 580 版驱动编一个超了三四倍的内核，48G 里编不完）。\
         把被多处调用的大函数改成只有一个调用点（例：走读内核的 w_walk_one 由 walk_and_hand_off 的阶段循环驱动）。\
         展开后占得最多的函数（名字、份数、自己的表达式数）：{:?}",
        size.expressions,
        LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL,
        size.largest_contributors
    );
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

    /// 同 [`Self::on_card_with_packed_tables`]，编的是调用方给的三份内核源码。判器的结论按 GPU 判器那一组判法单元的键入库
    /// （这个函数与 [`JudgeKernelSources::of_the_judge`] 都是那一组的入口）。
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
        let states_per_lane = states_per_scratch_lane(limit);
        let states_per_dispatch = states_in_one_dispatch_capped_by_the_state_count(
            states_in_flight_within(limit, card.memory_budget_mebibytes(), tables),
            requested_states_per_dispatch,
            tables.state_count,
        );
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
            dispatch_width: std::sync::atomic::AtomicUsize::new(
                states_per_dispatch.min(states_per_lane),
            ),
            calibrated: std::sync::atomic::AtomicBool::new(false),
            dispatches_since_calibration: std::sync::atomic::AtomicU64::new(0),
            knee: std::sync::atomic::AtomicUsize::new(usize::MAX),
            widest_dispatch: std::sync::atomic::AtomicUsize::new(0),
            nanoseconds_of_the_longest_dispatch: std::sync::atomic::AtomicU64::new(0),
            nanoseconds_preparing: std::sync::atomic::AtomicU64::new(0),
            nanoseconds_waiting_for_the_card: std::sync::atomic::AtomicU64::new(0),
            nanoseconds_reading_back: std::sync::atomic::AtomicU64::new(0),
            dispatches: std::sync::atomic::AtomicU64::new(0),
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

    /// 一次派活最多带几个状态（草稿区开了这么多；实际带几个在卡上现量，见 [`calibration_goes_on`]）。
    #[must_use]
    pub fn states_per_dispatch(&self) -> usize {
        self.states_per_dispatch
    }

    /// 此刻一次派活带几个状态（量过宽度之后就是量出来的那个）。
    #[must_use]
    pub fn dispatch_width(&self) -> usize {
        self.dispatch_width
            .load(std::sync::atomic::Ordering::Relaxed)
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
    /// 序号越过整条流的状态数；GPU 没写到某个状态（内核与派发对不上）。
    #[must_use]
    pub fn judge_states(&self, ordinals: Range<u64>) -> Vec<[u32; VERDICT_WORDS]> {
        assert!(
            ordinals.end <= self.state_count,
            "序号段 {ordinals:?} 越过状态数 {}",
            self.state_count
        );
        let mut out = Vec::with_capacity(
            usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize"),
        );
        let relaxed = std::sync::atomic::Ordering::Relaxed;
        let lane = self.states_per_lane.min(self.states_per_dispatch);
        let mut start = ordinals.start;
        while start < ordinals.end {
            let left = usize::try_from(ordinals.end - start).expect("装得进 usize");
            let calibration_is_due = !self.calibrated.load(relaxed)
                || self.dispatches_since_calibration.load(relaxed)
                    >= DISPATCHES_BETWEEN_CALIBRATIONS;
            let widest_to_try = self
                .states_per_dispatch
                .min(self.knee.load(relaxed).saturating_sub(1))
                .max(lane);
            if calibration_is_due && left >= 2 * lane && widest_to_try >= 2 * lane {
                // 量宽度：同一段起点一档加一条缓冲地派；最宽那一档判的状态包住了前面几档，它的结论照用
                let mut samples: Vec<(usize, f64)> = Vec::new();
                let widest_verdicts = loop {
                    let width = ((samples.len() + 1) * lane).min(widest_to_try).min(left);
                    let timed = self
                        .timed_dispatch(start..start + u64::try_from(width).expect("装得进 u64"));
                    samples.push((width, timed.seconds));
                    if width == left || !calibration_goes_on(&samples, widest_to_try) {
                        break timed.verdicts;
                    }
                };
                let chosen = calibrated_width(&samples);
                eprintln!(
                    "CRASH_JUDGE_DISPATCH_CALIBRATED card={:?} first_state={start} samples={} chosen_states={chosen} capacity={}",
                    self.card.name,
                    samples
                        .iter()
                        .map(|(width, seconds)| format!("{width}:{seconds:.3}"))
                        .collect::<Vec<_>>()
                        .join(","),
                    self.states_per_dispatch
                );
                self.dispatch_width.store(chosen, relaxed);
                self.knee.store(
                    knee_after_calibration(&samples, self.knee.load(relaxed)),
                    relaxed,
                );
                self.calibrated.store(true, relaxed);
                self.dispatches_since_calibration.store(0, relaxed);
                start += u64::try_from(widest_verdicts.len()).expect("装得进 u64");
                out.extend(widest_verdicts);
                continue;
            }
            let width = self.dispatch_width.load(relaxed);
            let end = (start + u64::try_from(width).expect("装得进")).min(ordinals.end);
            let timed = self.timed_dispatch(start..end);
            out.extend(timed.verdicts);
            self.dispatches_since_calibration.fetch_add(1, relaxed);
            // 驱动掐的是卡上一段不可抢占的活：一次派活拆成几次提交之后比的是最久的那一次提交（整次的用时随片数变长，不说明哪一段跑得太久）
            if timed.seconds_of_the_longest_submission > LONGEST_DISPATCH_SECONDS {
                let next = width_after_a_slow_dispatch(width, self.states_per_dispatch);
                eprintln!(
                    "CRASH_JUDGE_DISPATCH_WIDTH card={:?} states={} seconds={:.3} seconds_of_the_dispatch={:.3} next_states={next} capacity={}",
                    self.card.name,
                    end - start,
                    timed.seconds_of_the_longest_submission,
                    timed.seconds,
                    self.states_per_dispatch
                );
                self.dispatch_width.store(next, relaxed);
            }
            start = end;
        }
        out
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
        let mut nanoseconds_waited = 0u64;
        let mut nanoseconds_of_the_longest_submission = 0u64;
        let mut note_a_submission = |nanoseconds: u64| {
            nanoseconds_waited += nanoseconds;
            nanoseconds_of_the_longest_submission =
                nanoseconds_of_the_longest_submission.max(nanoseconds);
        };
        // 几趟按次序：扫描内核源码里的几趟读走读内核留在草稿区交接段里的东西，后一趟接着前一趟的违例位与判到位记（扫描重建那一截要读另几截写下的违例位）
        for (pass_name, pipeline) in [
            ("recovery", &self.recovery_pipeline),
            ("checker_walk", &self.checker_walk_pipeline),
            ("checker_scan", &self.checker_scan_pipeline),
            (
                "checker_scan_allocation_records_disjoint",
                &self.checker_scan_allocation_records_disjoint_pipeline,
            ),
            (
                "checker_scan_allocation_generations_through_the_rollback_floor_raises",
                &self
                    .checker_scan_allocation_generations_through_the_rollback_floor_raises_pipeline,
            ),
            (
                "checker_accounting_head",
                &self.checker_accounting_head_pipeline,
            ),
        ] {
            note_a_submission(self.submit_one_pass_and_wait(&passes, pass_name, pipeline, None));
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
                    "checker_accounting_slice",
                    &self.checker_accounting_slice_pipeline,
                    Some(&parameters),
                );
                note_a_submission(nanoseconds);
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
                note_a_submission(self.submit_one_pass_and_wait(
                    &passes,
                    "checker_accounting_decide",
                    &self.checker_accounting_decide_pipeline,
                    None,
                ));
            }
        }
        for (pass_name, pipeline) in [
            (
                "checker_accounting_final",
                &self.checker_accounting_final_pipeline,
            ),
            (
                "checker_references_and_snapshots",
                &self.checker_references_and_snapshots_pipeline,
            ),
            ("checker_scan_rebuild", &self.checker_scan_rebuild_pipeline),
        ] {
            note_a_submission(self.submit_one_pass_and_wait(&passes, pass_name, pipeline, None));
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
        note_a_submission(nanoseconds_of(reading_started - waiting_started));
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
        self.nanoseconds_preparing.fetch_add(
            nanoseconds_before_the_readback.saturating_sub(nanoseconds_waited),
            relaxed,
        );
        self.nanoseconds_waiting_for_the_card
            .fetch_add(nanoseconds_waited, relaxed);
        self.nanoseconds_reading_back
            .fetch_add(nanoseconds_of(reading_started.elapsed()), relaxed);
        self.dispatches.fetch_add(1, relaxed);
        let seconds_of_the_longest_submission = nanoseconds_of_the_longest_submission as f64 / 1e9;
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
                 多半是单次派活跑得太久被驱动掐了通道（看内核日志里的 Xid 109）；记账分片的片宽按每片的用时调，一次派活带的状态数按 LONGEST_DISPATCH_SECONDS 调",
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

/// 对拍用：在这张卡上跑序号 → 持久掩码的探针内核（`crate::crash_judge_gpu::persisted_mask_probe_kernel_source`，三个判器内核拆掩码的同一份
/// `begin_state`），一次派活拆完 `ordinals` 这一段，按序号次序交回每个状态的探针字（持久掩码 64 个字与判不了的原因）。
///
/// # Panics
/// 打包的表超过这张卡的绑定上限；这一段多于一次派活的工作组上限装得下的状态；回读长度对不上。
#[must_use]
pub fn persisted_masks_on_the_card(
    card: &GpuCard,
    tables: &PackedJudgeTables,
    ordinals: Range<u64>,
) -> Vec<[u32; PERSISTED_MASK_PROBE_WORDS]> {
    let count = usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize");
    if count == 0 {
        return Vec::new();
    }
    let device = card.device();
    let packed = &tables.words;
    let table_bytes = u64::try_from(packed.len() * 4).expect("装得进 u64");
    let limit = device.limits().max_storage_buffer_binding_size;
    assert!(
        table_bytes <= limit,
        "判器的表 {table_bytes} 字节超过这张卡的绑定上限 {limit}"
    );
    let pipeline = compute_pipeline(
        card,
        "crash-judge-persisted-mask-probe",
        &persisted_mask_probe_kernel_source(),
        KERNEL_ENTRY_POINT,
    );
    let tables_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("crash-judge-persisted-mask-probe-tables"),
            contents: &words_to_bytes(packed),
            usage: wgpu::BufferUsages::STORAGE,
        },
    );
    let dispatch_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("crash-judge-persisted-mask-probe-dispatch"),
            contents: &words_to_bytes(&dispatch_words(ordinals.start, count)),
            usage: wgpu::BufferUsages::UNIFORM,
        },
    );
    let result_bytes = u64::try_from(count * PERSISTED_MASK_PROBE_WORDS * 4).expect("装得进 u64");
    let masks = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("crash-judge-persisted-mask-probe-masks"),
            contents: &words_to_bytes(&vec![
                VERDICT_WORD_NOT_WRITTEN;
                count * PERSISTED_MASK_PROBE_WORDS
            ]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        },
    );
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("crash-judge-persisted-mask-probe-staging"),
        size: result_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let layout = pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("crash-judge-persisted-mask-probe"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: DISPATCH_BINDING,
                resource: dispatch_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: TABLES_BINDING,
                resource: tables_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: VERDICTS_BINDING,
                resource: masks.as_entire_binding(),
            },
        ],
    });
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(
            u32::try_from(count.div_ceil(JUDGE_KERNEL_WORKGROUP_SIZE)).expect("工作组数装得进 u32"),
            1,
            1,
        );
    }
    encoder.copy_buffer_to_buffer(&masks, 0, &staging, 0, result_bytes);
    let submission = card.queue().submit(std::iter::once(encoder.finish()));
    let slice = staging.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, readback_mapping_callback(sender));
    let polled = device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission),
        timeout: Some(std::time::Duration::from_secs(
            LONGEST_WAIT_FOR_THE_CARD_SECONDS,
        )),
    });
    take_the_readback_after_the_wait_or_record_the_device_lost(
        card.device_condition(),
        polled,
        &receiver,
        |error| {
            format!(
                "等了 {LONGEST_WAIT_FOR_THE_CARD_SECONDS} 秒，卡 {:?} 没把持久掩码探针算完：{error:?}",
                card.name
            )
        },
    );
    let mapped = slice.get_mapped_range().expect("读得到回读缓冲区");
    let (entries, remainder) = mapped.as_chunks::<4>();
    assert!(remainder.is_empty(), "回读缓冲区的长度是 4 的整数倍");
    assert_eq!(
        entries.len(),
        count * PERSISTED_MASK_PROBE_WORDS,
        "回读的字数与派的相同"
    );
    let out = entries
        .chunks(PERSISTED_MASK_PROBE_WORDS)
        .map(|state| {
            let mut words = [0u32; PERSISTED_MASK_PROBE_WORDS];
            for (word, chunk) in words.iter_mut().zip(state) {
                *word = u32::from_le_bytes(*chunk);
            }
            words
        })
        .collect();
    drop(mapped);
    staging.unmap();
    out
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
    cards: &'static crate::gpu_unit_checks::gpu_test_start::GpuCardPool,
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
        states_per_scratch_lane(cards.smallest_max_storage_buffer_binding_size());
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
mod tests {
    use super::{
        states_per_dispatch, states_per_scratch_lane, CHECKER_ACCOUNTING_DECIDE_ENTRY_POINT,
        CHECKER_ACCOUNTING_FINAL_ENTRY_POINT, CHECKER_ACCOUNTING_HEAD_ENTRY_POINT,
        CHECKER_ACCOUNTING_SLICE_ENTRY_POINT,
        CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT,
        CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT, MAXIMUM_SCRATCH_LANES,
    };
    use crate::crash_judge_gpu::{
        recovery_kernel_source, CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT,
        CHECKER_SCAN_REBUILD_ENTRY_POINT, GPU_JUDGE_CHECKER_COMMON_SOURCE, KERNEL_ENTRY_POINT,
        VERDICT_WORDS,
    };

    /// NVIDIA 卡上 wgpu 给的存储绑定上限。
    const BINDING_LIMIT: u64 = 2_147_483_644;

    /// 一个字几个字节（草稿区、结论都按 u32 字排）。
    const BYTES_PER_WORD: u64 = 4;

    /// 一 MiB 几个字节。
    const BYTES_PER_MEBIBYTE: u64 = 1024 * 1024;

    /// 内核里每个状态草稿区的末端（`crash_judge_checker_common.wgsl` 里 `const SCRATCH_END: u32 = …u;` 那一行，按字计）：
    /// 派活这一侧的钉值都从它算，不从派活自己用的 [`super::SCRATCH_WORDS`] 算——派活给每个状态的字数与内核的草稿区对不上时，
    /// 下面几条钉值跟着红；内核再加表、草稿区再抬，这几个数不用手改。
    fn scratch_words_of_the_kernels() -> u64 {
        let declaration_prefix = "const SCRATCH_END: u32 = ";
        GPU_JUDGE_CHECKER_COMMON_SOURCE
            .lines()
            .find_map(|line| line.strip_prefix(declaration_prefix))
            .expect("池级 checker 共用那一半里有 SCRATCH_END 那一行")
            .strip_suffix("u;")
            .expect("那一行以 `u;` 收尾")
            .parse()
            .expect("草稿区末端是一个十进制字数")
    }

    /// 一条草稿缓冲装几个状态由绑定上限定；一次派活带几个由给草稿区的显存定，到 [`MAXIMUM_SCRATCH_LANES`] 条为止，至少 1 个。
    /// 此刻每个状态 2 009 599 × 4 = 8 038 396 字节：一条缓冲 2 147 483 644 ÷ 8 038 396 = 267 个、12 GiB 装 1602 个（都向下取整）。
    #[test]
    fn states_in_one_dispatch_follow_the_scratch_memory_up_to_the_lane_limit() {
        let bytes_per_state = scratch_words_of_the_kernels() * BYTES_PER_WORD;
        let states_per_lane =
            usize::try_from(BINDING_LIMIT / bytes_per_state).expect("装得进 usize");
        assert_eq!(
            states_per_scratch_lane(BINDING_LIMIT),
            states_per_lane,
            "一条缓冲装绑定上限 ÷ 每个状态 {bytes_per_state} 字节个状态（向下取整）"
        );
        assert_eq!(states_per_scratch_lane(1), 1, "再小的上限也带一个");
        let twelve_gibibytes_in_mebibytes = 12 * 1024;
        let states_in_twelve_gibibytes =
            usize::try_from(twelve_gibibytes_in_mebibytes * BYTES_PER_MEBIBYTE / bytes_per_state)
                .expect("装得进 usize");
        assert!(
            states_in_twelve_gibibytes < states_per_lane * MAXIMUM_SCRATCH_LANES,
            "12 GiB 装的状态少于条数上限：这一格判的是显存那一边"
        );
        assert_eq!(
            states_per_dispatch(BINDING_LIMIT, twelve_gibibytes_in_mebibytes),
            states_in_twelve_gibibytes,
            "12 GiB 的草稿区装 12 GiB ÷ 每个状态 {bytes_per_state} 字节个状态（向下取整）"
        );
        assert_eq!(
            states_per_dispatch(BINDING_LIMIT, 0),
            1,
            "一点显存都不给也带一个"
        );
        assert_eq!(
            states_per_dispatch(BINDING_LIMIT, 1024 * 1024),
            states_per_lane * MAXIMUM_SCRATCH_LANES,
            "显存再多也只到条数上限"
        );
    }

    /// 卡池按 [`super::mebibytes_a_judge_needs`] 借给用例的卡（额度是借的量加驱动那一份）上，判器开的草稿区不少于借的时候算进去的状态数：
    /// 小流（45、489 个状态）照状态数、大流（131 195 个）照要的宽度（一条缓冲那么窄、三条缓冲加窄的那一档那么宽）。
    /// 表取 1 MiB：45 个状态要 1 + ⌈45 × (每个状态的草稿区 + 两份结论字 2 × 25 × 4) ÷ 1 048 576⌉ MiB；
    /// 此刻 1 + ⌈45 × 8 038 596 ÷ 1 048 576⌉ = 1 + ⌈361 736 820 ÷ 1 048 576⌉ = 1 + 345 = 346 MiB。
    #[test]
    fn a_card_lent_what_a_judge_needs_opens_scratch_for_every_state_it_was_lent_for() {
        use super::{mebibytes_a_judge_needs, states_in_flight_within, PackedJudgeTables};
        let tables_mebibytes = 1;
        let tables_of = |state_count: u64| PackedJudgeTables {
            words: vec![
                0;
                usize::try_from(tables_mebibytes * BYTES_PER_MEBIBYTE / BYTES_PER_WORD)
                    .expect("装得进 usize")
            ],
            state_count,
        };
        let verdict_words = u64::try_from(VERDICT_WORDS).expect("装得进 u64");
        let bytes_per_state_with_its_verdicts =
            (scratch_words_of_the_kernels() + 2 * verdict_words) * BYTES_PER_WORD;
        let small_flow_states = 45;
        assert_eq!(
            mebibytes_a_judge_needs(&tables_of(small_flow_states), usize::MAX),
            tables_mebibytes
                + (small_flow_states * bytes_per_state_with_its_verdicts)
                    .div_ceil(BYTES_PER_MEBIBYTE),
            "表 {tables_mebibytes} MiB 加 {small_flow_states} 个状态各 {bytes_per_state_with_its_verdicts} 字节（向上取整到 MiB）"
        );
        for (state_count, requested) in [
            (45, usize::MAX),
            (489, usize::MAX),
            (131_195, 1278),
            (131_195, 3 * 1278),
            (131_195, 256),
        ] {
            let tables = tables_of(state_count);
            let lent = mebibytes_a_judge_needs(&tables, requested);
            let states_lent_for =
                requested.min(usize::try_from(state_count).expect("装得进 usize"));
            assert!(
                states_in_flight_within(
                    BINDING_LIMIT,
                    lent + super::MEBIBYTES_LEFT_FOR_THE_DRIVER,
                    &tables
                ) >= states_lent_for,
                "{state_count} 个状态、要 {requested} 个一次：借了 {lent} MiB，判器开出来的草稿区装不下借的时候算进去的 {states_lent_for} 个"
            );
        }
    }

    /// 量宽度照两张卡上量到的数（固定脚本到回退、展开 16、流的中间，一档加一条 2170 个状态的缓冲）：
    /// 5090 到 23 870 个时每个状态的用时翻倍，停，取 21 700；5060 Ti 到 8 680 个时一次 0.324 秒，停，取 6 510。
    #[test]
    fn the_dispatch_width_is_the_quickest_per_state_before_the_card_slows_down() {
        use super::{calibrated_width, calibration_goes_on};
        let rtx_5090 = [
            (2170, 0.028),
            (4340, 0.028),
            (6510, 0.028),
            (8680, 0.029),
            (10_850, 0.029),
            (13_020, 0.031),
            (15_190, 0.031),
            (17_360, 0.032),
            (19_530, 0.033),
            (21_700, 0.034),
            (23_870, 0.071),
        ];
        for measured in 1..rtx_5090.len() {
            assert!(
                calibration_goes_on(&rtx_5090[..measured], 31_291),
                "5090 前 {measured} 档每个状态越来越快：接着加宽"
            );
        }
        assert!(
            !calibration_goes_on(&rtx_5090, 31_291),
            "5090 到 23870 个每个状态慢了一倍：停"
        );
        assert_eq!(calibrated_width(&rtx_5090), 21_700);
        let rtx_5060_ti = [(2170, 0.029), (4340, 0.032), (6510, 0.039), (8680, 0.324)];
        assert!(calibration_goes_on(&rtx_5060_ti[..3], 14_072));
        assert!(!calibration_goes_on(&rtx_5060_ti, 14_072));
        assert_eq!(calibrated_width(&rtx_5060_ti), 6510);
        assert!(
            !calibration_goes_on(&[(2170, 0.2), (4340, 0.26)], 31_291),
            "每个状态还在变快，可单次超过 0.25 秒：停"
        );
        assert!(
            !calibration_goes_on(&[(2170, 0.02)], 2170),
            "到了这张卡带得了的上限：停"
        );
    }

    /// 量完一次，最宽那一档不是选中的（过了拐点）就记下它，以后不再试到它；最宽那一档就是选中的（没撞到拐点）不记。
    #[test]
    fn a_width_past_the_knee_is_not_tried_again() {
        use super::knee_after_calibration;
        let past_the_knee = [(2170, 0.076), (4340, 0.110), (6510, 0.180), (8680, 3.6)];
        assert_eq!(knee_after_calibration(&past_the_knee, usize::MAX), 8680);
        assert_eq!(
            knee_after_calibration(&past_the_knee, 6000),
            6000,
            "已经记下更窄的拐点：不放宽"
        );
        let up_to_the_capacity = [(2170, 0.03), (4340, 0.035)];
        assert_eq!(
            knee_after_calibration(&up_to_the_capacity, usize::MAX),
            usize::MAX
        );
    }

    /// 一次派活算得太久：下一次减半，不少于 256（容量比它小取容量）。
    #[test]
    fn a_dispatch_that_took_too_long_halves_the_next_one() {
        use super::width_after_a_slow_dispatch;
        assert_eq!(width_after_a_slow_dispatch(21_700, 31_291), 10_850);
        assert_eq!(width_after_a_slow_dispatch(300, 31_291), 256);
        assert_eq!(width_after_a_slow_dispatch(100, 100), 100);
    }

    /// 记账分片一片算得多于 0.125 秒（[`super::LONGEST_DISPATCH_SECONDS`] 的四分之一）下一片减半、不窄过 16 个位置；少于 0.03125 秒（十六分之一）
    /// 加倍、不宽过一轮的位置数 65 536；中间不动。0.2 秒与 0.05 秒两档照每片 0.25 秒那一档调时一个不动、一个加倍，钉的是门槛低了一倍。
    #[test]
    fn an_accounting_slice_slower_than_an_eighth_of_a_second_halves_the_next_and_one_quicker_than_a_thirty_second_doubles_it(
    ) {
        use super::accounting_slice_width_after_a_slice;
        for (seconds, width_after, what) in [
            (0.3, 512, "0.3 秒：减半"),
            (0.2, 512, "0.2 秒：减半（还没到每片的上限 0.25 秒也减）"),
            (0.13, 512, "0.13 秒：减半"),
            (0.12, 1024, "0.12 秒：不动"),
            (0.05, 1024, "0.05 秒：不动（不加倍）"),
            (0.035, 1024, "0.035 秒：不动"),
            (0.03, 2048, "0.03 秒：加倍"),
        ] {
            assert_eq!(
                accounting_slice_width_after_a_slice(1024, seconds),
                width_after,
                "片宽 1024、{what}"
            );
        }
        assert_eq!(
            accounting_slice_width_after_a_slice(16, 0.3),
            16,
            "最窄 16 个位置：不再减半"
        );
        assert_eq!(
            accounting_slice_width_after_a_slice(65_536, 0.01),
            65_536,
            "整轮一片：不再加倍"
        );
    }

    /// 派活这一侧一轮的位置数与绑定号是扫描内核读的那两样：内核里 `C_ACCOUNTING_SLICE_POSITIONS` 是 1 左移 `C_ACCOUNTING_SLICE_POSITION_BITS` 位，
    /// 分片那一趟的派活参数绑在第 [`super::ACCOUNTING_SLICE_BINDING`] 号。两边一漂，末片的上端不是内核认的「罩满」，每个状态记判不了（或只判了一截）。不用卡。
    #[test]
    fn the_accounting_slice_positions_and_binding_of_the_dispatch_are_the_ones_the_scan_kernel_reads(
    ) {
        let source = crate::crash_judge_gpu::checker_scan_kernel_source();
        let declaration_prefix = "const C_ACCOUNTING_SLICE_POSITION_BITS: u32 = ";
        let bits: u32 = source
            .lines()
            .find_map(|line| line.strip_prefix(declaration_prefix))
            .expect("扫描内核里有 C_ACCOUNTING_SLICE_POSITION_BITS 那一行")
            .strip_suffix("u;")
            .expect("那一行以 `u;` 收尾")
            .parse()
            .expect("位数是一个十进制数");
        assert_eq!(
            1u32 << bits,
            super::ACCOUNTING_SLICE_POSITIONS,
            "内核一轮 1 << {bits} 个位置，派活这一侧 {} 个",
            super::ACCOUNTING_SLICE_POSITIONS
        );
        let binding_declaration = format!(
            "@group(0) @binding({}) var<uniform> accounting_slice_parameters:",
            super::ACCOUNTING_SLICE_BINDING
        );
        assert_eq!(
            source.matches(binding_declaration.as_str()).count(),
            1,
            "扫描内核里分片那一趟的派活参数绑在第 {} 号",
            super::ACCOUNTING_SLICE_BINDING
        );
    }

    /// 双机崩溃放量入口给多机配置 `GPU_KERNEL_SERIAL_COMPILE_HOSTS` 点名的那几台设的变量（`serial_compile_environment=(` 那一行里的 `<变量名>=1`）
    /// 就是判器读的那一个：两边的名字一漂，那几台照几条一起编内核、内存紧时撞上限，判出来的结论不变，别的用例都看不见。不用卡。
    #[test]
    fn the_two_host_crash_amplification_entry_sets_the_variable_the_judge_reads_to_compile_its_kernels_one_at_a_time(
    ) {
        let entry_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scripts/crash/crash-amplification-two-hosts.sh");
        let entry = std::fs::read_to_string(&entry_path).unwrap_or_else(|error| {
            panic!("双机崩溃放量入口 {} 读不出：{error}", entry_path.display())
        });
        let serial_compile_environment_line = entry
            .lines()
            .find(|line| line.starts_with("serial_compile_environment=("))
            .expect("双机崩溃放量入口里有给内核一个一个编的那几台设环境的 serial_compile_environment=( 那一行");
        let assignment = format!(
            " {}=1 ",
            super::GPU_KERNELS_ONE_AT_A_TIME_ENVIRONMENT_VARIABLE
        );
        assert!(
            serial_compile_environment_line.contains(&assignment),
            "双机崩溃放量入口给内核一个一个编的那几台设的环境里要有「{}」（判器读的变量名），实际那一行：{serial_compile_environment_line}",
            assignment.trim()
        );
    }

    /// 这个进程到此刻为止常驻内存的峰值（MiB，`/proc/self/status` 的 VmHWM）；读不到报 0。
    fn peak_resident_mebibytes() -> u64 {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|status| {
                status.lines().find_map(|line| {
                    line.strip_prefix("VmHWM:")?
                        .split_whitespace()
                        .next()?
                        .parse::<u64>()
                        .ok()
                })
            })
            .map_or(0, |kibibytes| kibibytes / 1024)
    }

    /// 三个内核在本机第一张能用的卡上一个一个编（从小到大），每编完一个打一行：编了多久、到那一刻进程常驻内存的峰值、
    /// 内核原文多大。查「这台机器编判器内核要多少内存、卡在哪一个上」用；中途被内存上限杀掉，已经打出来的行还在。
    /// 要量冷编，把驱动的盘上缓存指到一个空目录（环境变量 `__GL_SHADER_DISK_CACHE_PATH`）。
    #[test]
    #[ignore = "要本机一张能用的 NVIDIA 卡（没有 NVIDIA 卡报「本次未跑」，有卡而用不上判红）；冷编每个内核几分钟"]
    fn each_kernel_compile_time_and_peak_memory_one_at_a_time() {
        use crate::crash_judge_gpu::{checker_scan_kernel_source, checker_walk_kernel_source};
        use std::io::Write;
        let Some(card) =
            crate::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run(
                "each_kernel_compile_time_and_peak_memory_one_at_a_time",
            )
        else {
            return;
        };
        println!(
            "GPU_KERNEL_COMPILE_CARD card={:?} index={} free_mebibytes={} peak_resident_mebibytes_before={}",
            card.name,
            card.index,
            card.free_memory_mebibytes,
            peak_resident_mebibytes()
        );
        for (kernel, source, entry_point) in [
            ("recovery", recovery_kernel_source(), KERNEL_ENTRY_POINT),
            (
                "checker-scan",
                checker_scan_kernel_source(),
                KERNEL_ENTRY_POINT,
            ),
            (
                "checker-scan-allocation-records-disjoint",
                checker_scan_kernel_source(),
                CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT,
            ),
            (
                "checker-scan-allocation-generations-through-the-rollback-floor-raises",
                checker_scan_kernel_source(),
                CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT,
            ),
            (
                "checker-accounting-head",
                checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_HEAD_ENTRY_POINT,
            ),
            (
                "checker-accounting-slice",
                checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_SLICE_ENTRY_POINT,
            ),
            (
                "checker-accounting-decide",
                checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_DECIDE_ENTRY_POINT,
            ),
            (
                "checker-accounting-final",
                checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_FINAL_ENTRY_POINT,
            ),
            (
                "checker-references-and-snapshots",
                checker_scan_kernel_source(),
                CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT,
            ),
            (
                "checker-scan-rebuild",
                checker_scan_kernel_source(),
                CHECKER_SCAN_REBUILD_ENTRY_POINT,
            ),
            (
                "checker-walk",
                checker_walk_kernel_source(),
                KERNEL_ENTRY_POINT,
            ),
        ] {
            println!(
                "GPU_KERNEL_COMPILE_STARTED kernel={kernel} source_bytes={}",
                source.len()
            );
            std::io::stdout().flush().expect("刷得出去");
            let started = std::time::Instant::now();
            let _pipeline = super::compute_pipeline(&card, kernel, &source, entry_point);
            println!(
                "GPU_KERNEL_COMPILE kernel={kernel} seconds={:.1} peak_resident_mebibytes={}",
                started.elapsed().as_secs_f64(),
                peak_resident_mebibytes()
            );
            std::io::stdout().flush().expect("刷得出去");
        }
    }

    /// 展开之后的大小按调用点逐处累加：`leaf` 自己几个表达式、`middle` 调它两处、入口调 `middle` 三处，入口展开之后正好是
    /// 入口自己 + 3 ×（middle 自己 + 2 × leaf 自己）；份数最多的是 `leaf`（6 份）。
    #[test]
    fn inlined_size_adds_one_copy_of_the_callee_at_every_call_site() {
        let source = "
            @group(0) @binding(0) var<storage, read_write> out: array<u32>;
            fn leaf(x: u32) -> u32 { return x * 3u + 1u; }
            fn middle(x: u32) -> u32 { return leaf(x) + leaf(x + 2u); }
            @compute @workgroup_size(1) fn main() {
                out[0] = middle(1u);
                out[1] = middle(2u);
                out[2] = middle(3u);
            }";
        let module = wgpu::naga::front::wgsl::parse_str(source).expect("解析得了");
        let own_of = |name: &str| -> u64 {
            let (_, function) = module
                .functions
                .iter()
                .find(|(_, function)| function.name.as_deref() == Some(name))
                .expect("有这个函数");
            u64::try_from(function.expressions.len()).expect("装得进 u64")
        };
        let entry_own =
            u64::try_from(module.entry_points[0].function.expressions.len()).expect("装得进 u64");
        let size = super::kernel_inlined_size(source);
        assert_eq!(
            size.expressions,
            entry_own + 3 * (own_of("middle") + 2 * own_of("leaf")),
            "入口自己 + 三份 middle，每份 middle 带两份 leaf"
        );
        assert_eq!(
            size.largest_contributors[0],
            ("leaf".to_string(), 6, own_of("leaf")),
            "leaf 被展开 6 份，占得最多"
        );
    }

    /// 判器的全部管线（三个内核，扫描内核源码另有扫描那两组、记账那四趟、引用与快照那一截、扫描重建那一截几个入口）都在预算之内；打各自展开之后的大小。
    #[test]
    fn every_kernel_stays_within_the_inlined_size_budget() {
        for (name, source, entry_point) in [
            ("recovery", recovery_kernel_source(), KERNEL_ENTRY_POINT),
            (
                "checker-scan",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                KERNEL_ENTRY_POINT,
            ),
            (
                "checker-scan-allocation-records-disjoint",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT,
            ),
            (
                "checker-scan-allocation-generations-through-the-rollback-floor-raises",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT,
            ),
            (
                "checker-accounting-head",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_HEAD_ENTRY_POINT,
            ),
            (
                "checker-accounting-slice",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_SLICE_ENTRY_POINT,
            ),
            (
                "checker-accounting-decide",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_DECIDE_ENTRY_POINT,
            ),
            (
                "checker-accounting-final",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_FINAL_ENTRY_POINT,
            ),
            (
                "checker-references-and-snapshots",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT,
            ),
            (
                "checker-scan-rebuild",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_SCAN_REBUILD_ENTRY_POINT,
            ),
            (
                "checker-walk",
                crate::crash_judge_gpu::checker_walk_kernel_source(),
                KERNEL_ENTRY_POINT,
            ),
        ] {
            let size = super::kernel_inlined_size_of_the_entry_point(&source, entry_point);
            println!(
                "GPU_KERNEL_INLINED kernel={name} expressions={} budget={} largest={:?}",
                size.expressions,
                super::LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL,
                size.largest_contributors
            );
            super::refuse_a_kernel_entry_point_too_large_for_the_driver(name, &source, entry_point);
        }
    }

    /// 在扫描内核里把这七个调用方各自的多处调用并成了一处（调用方，它里面那个被调函数）。
    const SCAN_KERNEL_JUDGES_MERGED_INTO_ONE_CALL_SITE: [(&str, &str); 7] = [
        (
            "judge_release_generation_and_tree_table_birth",
            "r_references_of_root",
        ),
        (
            "judge_allocation_records_disjoint",
            "judge_allocation_records_disjoint_of_a_version",
        ),
        (
            "judge_allocation_generations_against_unit_births",
            "judge_allocation_generations_under_tree_table",
        ),
        ("accounting_head", "c_quarantine_exempted"),
        (
            "judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark",
            "judge_first_data_unit_of_a_version",
        ),
        (
            "judge_deletion_state_of_the_versions",
            "d_judge_deletion_state_of_a_version",
        ),
        (
            "judge_tombstones_and_the_lineage_of_the_versions",
            "d_judge_tombstones_and_the_lineage_of_a_version",
        ),
    ];

    /// 一份内核源码里每个（调用方, 被调方）有几个调用点：naga 解析之后逐个函数数调用语句，入口函数记成 `entry:<名字>`。
    /// 与 [`super::kernel_inlined_size`] 各数各的（它不交回调用点表），这一份只给钉调用点数的用例用。
    fn call_sites_by_caller_and_callee(
        source: &str,
    ) -> std::collections::BTreeMap<(String, String), u64> {
        use wgpu::naga::{Block, Function, Handle, Statement};
        #[allow(
            clippy::wildcard_enum_match_arm,
            reason = "naga 的语句是外部的枚举，只有这四种里还套着语句块，其余一律不含调用"
        )]
        fn callees_in(block: &Block, callees: &mut Vec<Handle<Function>>) {
            for statement in block.iter() {
                match statement {
                    Statement::Call { function, .. } => callees.push(*function),
                    Statement::Block(nested) => callees_in(nested, callees),
                    Statement::If { accept, reject, .. } => {
                        callees_in(accept, callees);
                        callees_in(reject, callees);
                    }
                    Statement::Switch { cases, .. } => {
                        for switch_case in cases {
                            callees_in(&switch_case.body, callees);
                        }
                    }
                    Statement::Loop {
                        body, continuing, ..
                    } => {
                        callees_in(body, callees);
                        callees_in(continuing, callees);
                    }
                    _ => {}
                }
            }
        }
        let module = wgpu::naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|error| panic!("内核解析失败：\n{}", error.emit_to_string(source)));
        let mut sites = std::collections::BTreeMap::new();
        let callers = module
            .functions
            .iter()
            .map(|(_, function)| (function.name.clone().unwrap_or_default(), &function.body))
            .chain(
                module
                    .entry_points
                    .iter()
                    .map(|entry| (format!("entry:{}", entry.name), &entry.function.body)),
            );
        for (caller, body) in callers {
            let mut callees = Vec::new();
            callees_in(body, &mut callees);
            for callee in callees {
                *sites
                    .entry((
                        caller.clone(),
                        module.functions[callee].name.clone().unwrap_or_default(),
                    ))
                    .or_insert(0) += 1;
            }
        }
        sites
    }

    /// 并成一处调用的七个函数：每个被调函数在整个扫描内核里只有一个调用点、就在它的调用方里，调用方自己也只被调一处
    /// （数是在主工作区的扫描内核上现量的）。别处再多调一次（哪怕总数还在预算里，例：`judge_accounting` 加回一处只到 392 383），
    /// 先红在这里，不等上卡才撞展开预算；真要多一处，先量展开之后的大小，再改这里的钉值。
    #[test]
    fn each_judge_merged_into_one_call_site_keeps_that_single_call_site_in_the_scan_kernel() {
        const CALLEE_SITES_INSIDE_THE_CALLER: u64 = 1;
        const CALLEE_SITES_IN_THE_WHOLE_KERNEL: u64 = 1;
        const CALLER_SITES_IN_THE_WHOLE_KERNEL: u64 = 1;
        let sites =
            call_sites_by_caller_and_callee(&crate::crash_judge_gpu::checker_scan_kernel_source());
        let sites_calling = |callee: &str| -> u64 {
            sites
                .iter()
                .filter(|((_, called), _)| called == callee)
                .map(|(_, count)| *count)
                .sum()
        };
        for (caller, callee) in SCAN_KERNEL_JUDGES_MERGED_INTO_ONE_CALL_SITE {
            assert_eq!(
                sites
                    .get(&(caller.to_string(), callee.to_string()))
                    .copied()
                    .unwrap_or(0),
                CALLEE_SITES_INSIDE_THE_CALLER,
                "{caller} 里调 {callee} 的地方只许一处"
            );
            assert_eq!(
                sites_calling(callee),
                CALLEE_SITES_IN_THE_WHOLE_KERNEL,
                "整个扫描内核里调 {callee} 的地方只许一处（就是 {caller} 里那一处）"
            );
            assert_eq!(
                sites_calling(caller),
                CALLER_SITES_IN_THE_WHOLE_KERNEL,
                "整个扫描内核里调 {caller} 的地方只许一处"
            );
        }
    }

    /// 每个入口此刻展开之后的表达式数与离预算的余量（用 naga 解析、校验，数法同 `kernel_inlined_size`：调用点展开之后数表达式；预算见 `LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL`，五份以上 WGSL 照库里的拼法拼）。
    /// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
    /// 全部入口逐个量完再判：对不上的一次列全（名字、量到的数、钉值、离预算的余量、钉的余量），一趟就拿得到重钉要的全部数；
    /// 每个入口另打一行 `GPU_KERNEL_INLINED_PIN`，对得上的也打。
    #[test]
    fn each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget() {
        let budget = super::LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL;
        let mut kernels_off_their_pins: Vec<String> = Vec::new();
        for (name, source, entry_point, pinned_expressions, pinned_margin) in [
            (
                "recovery",
                recovery_kernel_source(),
                KERNEL_ENTRY_POINT,
                249_396,
                150_604,
            ),
            (
                "checker-scan",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                KERNEL_ENTRY_POINT,
                7_885,
                392_115,
            ),
            (
                "checker-scan-allocation-records-disjoint",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT,
                27_912,
                372_088,
            ),
            (
                "checker-scan-allocation-generations-through-the-rollback-floor-raises",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT,
                277_387,
                122_613,
            ),
            (
                "checker-accounting-head",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_HEAD_ENTRY_POINT,
                34_463,
                365_537,
            ),
            (
                "checker-accounting-slice",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_SLICE_ENTRY_POINT,
                6_955,
                393_045,
            ),
            (
                "checker-accounting-decide",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_DECIDE_ENTRY_POINT,
                3_077,
                396_923,
            ),
            (
                "checker-accounting-final",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_ACCOUNTING_FINAL_ENTRY_POINT,
                51_003,
                348_997,
            ),
            (
                "checker-references-and-snapshots",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT,
                191_118,
                208_882,
            ),
            (
                "checker-scan-rebuild",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
                CHECKER_SCAN_REBUILD_ENTRY_POINT,
                69_417,
                330_583,
            ),
            (
                "checker-walk",
                crate::crash_judge_gpu::checker_walk_kernel_source(),
                KERNEL_ENTRY_POINT,
                357_405,
                42_595,
            ),
        ] {
            let size = super::kernel_inlined_size_of_the_entry_point(&source, entry_point);
            let measured_margin = budget.checked_sub(size.expressions);
            let measured_margin_text = measured_margin.map_or_else(
                || "没有（超了预算）".to_string(),
                |remaining_expressions| remaining_expressions.to_string(),
            );
            println!(
                "GPU_KERNEL_INLINED_PIN kernel={name} expressions={} pinned_expressions={pinned_expressions} \
                 margin={measured_margin_text} pinned_margin={pinned_margin} budget={budget}",
                size.expressions
            );
            if size.expressions != pinned_expressions || measured_margin != Some(pinned_margin) {
                kernels_off_their_pins.push(format!(
                    "内核 {name}：量到 {} 个表达式、钉值 {pinned_expressions}；离预算 {budget} 的余量 {measured_margin_text}、钉的余量 {pinned_margin}",
                    size.expressions
                ));
            }
        }
        assert!(
            kernels_off_their_pins.is_empty(),
            "{} 个内核展开之后的表达式数或离预算的余量与钉值对不上（全部入口都量完了，下面逐个列全）：\
             量过、确认余量还够之后改这里的钉值\n{}",
            kernels_off_their_pins.len(),
            kernels_off_their_pins.join("\n")
        );
    }

    /// 走读内核里「走一遍根」回到六个调用点（第二台 580 版驱动 48G 里编不完的那个形态），编之前就被拒。
    #[test]
    fn the_walk_kernel_with_the_root_walk_called_from_six_sites_is_refused() {
        const CALL: &str = "w_walk_one(walk_kind, walk_base, walk_is_newest, fsid_low);";
        let source = crate::crash_judge_gpu::checker_walk_kernel_source();
        assert_eq!(
            source.matches(CALL).count(),
            1,
            "走读内核里「走一遍根」只有一个调用点"
        );
        let six_sites = source.replacen(CALL, &CALL.repeat(6), 1);
        let size = super::kernel_inlined_size(&six_sites);
        println!(
            "GPU_KERNEL_INLINED kernel=checker-walk-six-sites expressions={}",
            size.expressions
        );
        let refused = std::panic::catch_unwind(|| {
            super::refuse_a_kernel_too_large_for_the_driver("checker-walk-six-sites", &six_sites);
        });
        assert!(
            refused.is_err(),
            "六个调用点展开之后 {} 个表达式，该被拒",
            size.expressions
        );
    }

    /// 量一次恢复内核在本机第一张能用的卡上的编译时间（要显卡；打一行 `GPU_KERNEL_COMPILE kernel=recovery seconds=…`）：
    /// 连跑两次、比第二次快不快，就知道驱动的盘上着色器缓存管不管 Vulkan 管线。
    #[test]
    #[ignore = "要本机一张能用的 NVIDIA 卡（没有 NVIDIA 卡报「本次未跑」，有卡而用不上判红），编一次几十秒"]
    fn recovery_kernel_compile_time_on_the_first_usable_card() {
        let Some(card) =
            crate::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run(
                "recovery_kernel_compile_time_on_the_first_usable_card",
            )
        else {
            return;
        };
        let started = std::time::Instant::now();
        let _pipeline = super::compute_pipeline(
            &card,
            "crash-judge-recovery",
            &recovery_kernel_source(),
            KERNEL_ENTRY_POINT,
        );
        println!(
            "GPU_KERNEL_COMPILE kernel=recovery seconds={:.1}",
            started.elapsed().as_secs_f64()
        );
    }
}
