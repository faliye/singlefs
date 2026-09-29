//! GPU 判器的派活：把一条流打包好的表与草稿区放上一张卡、编内核、把一段状态分成几次派活交给卡算、回读结论。
//! 判法（内核原文、表怎么打包、派活参数与结论各是哪几个字、草稿区每个状态多大）在 `crate::crash_judge_gpu`；这一份只管怎么上卡，
//! 不在判法闭包里（`crate::crash_identity::JUDGE_CLOSURE_STOP_FILES`）：改一次派活带几个状态、草稿区分几条缓冲、显存怎么算，
//! 不换判法版本、不用把判过的状态重判一遍（用户 2026-09-29 定「派活和判法拆出来最好」）。
//! 这一份修了会改结论的毛病时，抬 `crate::crash_judge_gpu::GPU_JUDGE_REVISION`。
//!
//! 一张卡上同时在算的就是一次派活里的状态。草稿区一条缓冲受这张卡的存储绑定上限管（wgpu 压到 2 GiB − 4，装 2170 个状态）；
//! 一次派活要带更多状态，就每条缓冲各派一次活、同一个内核的几次派活排在同一趟里（它们碰的草稿与结论缓冲互不相同），卡一起算。
//! 5090 上量到的（固定脚本到回退、展开 16，131 195 态）：一次 256 / 2170 / 4340 / 8680 / 13020 个状态，
//! 每秒 9 405 / 75 360 / 139 970 / 241 726 / 310 100 态，每次派活 27 / 28 / 30 / 34 / 38 毫秒。
use std::ops::Range;

use crate::crash_judge_gpu::{
    checker_scan_kernel_source, checker_walk_kernel_source, dispatch_words, pack_tables,
    recovery_kernel_source, verdict_was_written_by_both_kernels, DISPATCH_BINDING, SCRATCH_BINDING,
    SCRATCH_WORDS, TABLES_BINDING, VERDICTS_BINDING, VERDICT_WORDS, VERDICT_WORD_NOT_WRITTEN,
    WORKGROUP_SIZE,
};
use crate::crash_judge_tables::JudgeTables;
use crate::gpu_unit_checks::GpuCard;

/// 草稿区最多分几条缓冲（32 条是 64 GiB，比今天任何一张卡的显存都大：条数实际由显存额度定）。
pub const MAXIMUM_SCRATCH_LANES: usize = 32;
/// 表与草稿区之外这个进程在卡上还要占的显存（MiB）：驱动的上下文、三个编好的内核、结论与回读缓冲。
/// 判器的草稿区按「这张卡的显存额度 − 表 − 这个数」定。5090 上量到的是 2326 MiB（草稿区从 241 MiB 到 12283 MiB 五档都是这个数），留到 2560。
pub const MEBIBYTES_LEFT_FOR_THE_DRIVER: u64 = 2560;
/// 一次派活卡上最多算多久（秒）。派活在卡上跑的时候卡让不出来，跑得太久驱动会报上下文切换超时、把这条通道掐掉
/// （内核日志 Xid 109；固定脚本到 E 那条流一次派约三万个状态时撞过，程序从此等不到结果）。一次派活带几个状态按它调。
pub const LONGEST_DISPATCH_SECONDS: f64 = 0.5;
/// 等卡算完最多等多久（秒）：到点还没算完就报错，不干等（通道被驱动掐掉之后结果永远不回来）。
pub const LONGEST_WAIT_FOR_THE_CARD_SECONDS: u64 = 120;
/// 一次派活带的状态数最少几个（算得太久减半时的下限）。
const NARROWEST_DISPATCH: usize = 256;
/// 量宽度时，这一档每个状态的用时比前面最好那一档多出这么多倍就不再加宽。
pub const CALIBRATION_WORSE_THAN_THE_BEST: f64 = 1.25;
/// 每派多少次活重量一次宽度：状态变重变轻，拐点跟着挪。
pub const DISPATCHES_BETWEEN_CALIBRATIONS: u64 = 512;

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

fn words_to_bytes(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

/// 一条流打包好的表：打包一次，几张卡各上一份。
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

    /// 表上卡之后占多少显存（MiB，向上取整）。
    #[must_use]
    pub fn mebibytes(&self) -> u64 {
        u64::try_from(self.words.len() * 4)
            .expect("装得进 u64")
            .div_ceil(1024 * 1024)
    }
}

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

/// 一张卡上装好表的判器。
pub struct GpuCrashJudge<'card> {
    card: &'card GpuCard,
    recovery_pipeline: wgpu::ComputePipeline,
    checker_walk_pipeline: wgpu::ComputePipeline,
    checker_scan_pipeline: wgpu::ComputePipeline,
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
/// 编不编得过的真实边界在 27 万与 98 万之间，没量过。
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
///
/// # Panics
/// 源码解析不了（内核源码自己的单测先判这一条）。
#[must_use]
pub fn kernel_inlined_size(source: &str) -> KernelInlinedSize {
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
        .find(|entry| entry.name == "main")
        .expect("内核的入口叫 main");
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
    largest_contributors.sort_by_key(|(_, copies, own)| std::cmp::Reverse(copies * own));
    largest_contributors.truncate(5);
    KernelInlinedSize {
        expressions,
        largest_contributors,
    }
}

/// 内核展开之后超过 [`LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL`] 就拒绝，不交给驱动去编。
///
/// # Panics
/// 超过预算：消息里给展开份数最多的几个函数。
pub fn refuse_a_kernel_too_large_for_the_driver(label: &str, source: &str) {
    let size = kernel_inlined_size(source);
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
fn compute_pipeline(card: &GpuCard, label: &str, source: &str) -> wgpu::ComputePipeline {
    refuse_a_kernel_too_large_for_the_driver(label, source);
    let device = card.device();
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

impl<'card> GpuCrashJudge<'card> {
    /// 把一条流的表上卡、编内核。
    ///
    /// # Panics
    /// 打包的表超过这张卡的存储缓冲绑定上限（表太大，GPU 判不了这条流）。
    #[must_use]
    pub fn new(card: &'card GpuCard, tables: &JudgeTables) -> Self {
        Self::with_states_per_dispatch(card, tables, usize::MAX)
    }

    /// 同 [`Self::new`]，一次派活的状态数另给（量速度、规划显卡用）：照旧被这张卡的显存额度与草稿缓冲的条数压住，至少 1。
    ///
    /// # Panics
    /// 同 [`Self::new`]。
    #[must_use]
    pub fn with_states_per_dispatch(
        card: &'card GpuCard,
        tables: &JudgeTables,
        requested_states_per_dispatch: usize,
    ) -> Self {
        Self::on_card_with_packed_tables(
            card,
            &PackedJudgeTables::of(tables),
            requested_states_per_dispatch,
        )
    }

    /// 同 [`Self::with_states_per_dispatch`]，表是打包好的（几张卡共用一份打包）。
    ///
    /// # Panics
    /// 同 [`Self::new`]。
    #[must_use]
    pub fn on_card_with_packed_tables(
        card: &'card GpuCard,
        tables: &PackedJudgeTables,
        requested_states_per_dispatch: usize,
    ) -> Self {
        let packed = &tables.words;
        let device = card.device();
        let table_bytes = u64::try_from(packed.len() * 4).expect("装得进 u64");
        let limit = device.limits().max_storage_buffer_binding_size;
        assert!(
            table_bytes <= limit,
            "判器的表 {table_bytes} 字节超过这张卡的绑定上限 {limit}：这条流 GPU 判不了"
        );
        let states_per_lane = states_per_scratch_lane(limit);
        let states_per_dispatch =
            states_in_flight_within(limit, card.memory_budget_mebibytes(), tables)
                .min(requested_states_per_dispatch.max(1));
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
        // 三个内核各在一条线程上编（驱动的编译是单线程的，checker 两个内核冷编各要几分钟；盘上缓存装好之后几秒）。
        // 驱动编大内核吃内存（595 版三个一起编常驻 8.6 GiB，580 版一起编撞过 40G 上限）：内存紧的机器写
        // `SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME=1`，三个内核一个一个编。
        let one_at_a_time =
            std::env::var("SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME").is_ok_and(|value| value == "1");
        let (recovery_pipeline, checker_walk_pipeline, checker_scan_pipeline) = if one_at_a_time {
            (
                compute_pipeline(card, "crash-judge-recovery", &recovery_kernel_source()),
                compute_pipeline(
                    card,
                    "crash-judge-checker-walk",
                    &checker_walk_kernel_source(),
                ),
                compute_pipeline(
                    card,
                    "crash-judge-checker-scan",
                    &checker_scan_kernel_source(),
                ),
            )
        } else {
            std::thread::scope(|scope| {
                let recovery = scope.spawn(|| {
                    compute_pipeline(card, "crash-judge-recovery", &recovery_kernel_source())
                });
                let scan = scope.spawn(|| {
                    compute_pipeline(
                        card,
                        "crash-judge-checker-scan",
                        &checker_scan_kernel_source(),
                    )
                });
                let walk = compute_pipeline(
                    card,
                    "crash-judge-checker-walk",
                    &checker_walk_kernel_source(),
                );
                (
                    recovery.join().expect("恢复内核的编译线程不 panic"),
                    walk,
                    scan.join().expect("扫描内核的编译线程不 panic"),
                )
            })
        };
        Self {
            card,
            recovery_pipeline,
            checker_walk_pipeline,
            checker_scan_pipeline,
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
                    let (verdicts, seconds) = self
                        .timed_dispatch(start..start + u64::try_from(width).expect("装得进 u64"));
                    samples.push((width, seconds));
                    if width == left || !calibration_goes_on(&samples, widest_to_try) {
                        break verdicts;
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
            let (verdicts, seconds) = self.timed_dispatch(start..end);
            out.extend(verdicts);
            self.dispatches_since_calibration.fetch_add(1, relaxed);
            if seconds > LONGEST_DISPATCH_SECONDS {
                let next = width_after_a_slow_dispatch(width, self.states_per_dispatch);
                eprintln!(
                    "CRASH_JUDGE_DISPATCH_WIDTH card={:?} states={} seconds={seconds:.3} next_states={next} capacity={}",
                    self.card.name,
                    end - start,
                    self.states_per_dispatch
                );
                self.dispatch_width.store(next, relaxed);
            }
            start = end;
        }
        out
    }

    /// 一次派活，交回结论与卡算了几秒；记下带得最多的一次、算得最久的一次。
    fn timed_dispatch(&self, ordinals: Range<u64>) -> (Vec<[u32; VERDICT_WORDS]>, f64) {
        let relaxed = std::sync::atomic::Ordering::Relaxed;
        let waited_before = self.nanoseconds_waiting_for_the_card.load(relaxed);
        let states = usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize");
        let verdicts = self.judge_one_dispatch(ordinals);
        let nanoseconds = self.nanoseconds_waiting_for_the_card.load(relaxed) - waited_before;
        self.widest_dispatch.fetch_max(states, relaxed);
        self.nanoseconds_of_the_longest_dispatch
            .fetch_max(nanoseconds, relaxed);
        (verdicts, nanoseconds as f64 / 1e9)
    }

    /// 一次派活判完一段状态，不按用时调宽度（量派活宽度与用时的关系用）；这一次卡算了多久计进 [`Self::time_spent`]。
    ///
    /// # Panics
    /// 状态数超过 [`Self::states_per_dispatch`]；同 [`Self::judge_states`]。
    #[must_use]
    pub fn judge_states_in_one_dispatch(&self, ordinals: Range<u64>) -> Vec<[u32; VERDICT_WORDS]> {
        self.judge_one_dispatch(ordinals)
    }

    fn judge_one_dispatch(&self, ordinals: Range<u64>) -> Vec<[u32; VERDICT_WORDS]> {
        let count = usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize");
        assert!(count <= self.states_per_dispatch);
        if count == 0 {
            return Vec::new();
        }
        let preparing_started = std::time::Instant::now();
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
        let result_bytes = u64::try_from(count * VERDICT_WORDS * 4).expect("装得进 u64");
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("crash-judge-staging"),
            size: result_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        // 三趟按次序：扫描内核读走读内核留在草稿区交接段里的东西，同一个编码器里的派活按提交次序可见；
        // 一趟里每条草稿缓冲各派一次活，它们碰的草稿与结论缓冲互不相同、读的是同一份只读的表
        for pipeline in [
            &self.recovery_pipeline,
            &self.checker_walk_pipeline,
            &self.checker_scan_pipeline,
        ] {
            let layout = pipeline.get_bind_group_layout(0);
            let bind_groups: Vec<wgpu::BindGroup> = parts
                .iter()
                .zip(&dispatch_and_verdict_buffers)
                .map(|((_, _, scratch), (dispatch_buffer, verdicts))| {
                    let entries = [
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
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("crash-judge"),
                        layout: &layout,
                        entries: &entries,
                    })
                })
                .collect();
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            for ((_, states, _), bind_group) in parts.iter().zip(&bind_groups) {
                pass.set_bind_group(0, bind_group, &[]);
                pass.dispatch_workgroups(
                    u32::try_from(states.div_ceil(WORKGROUP_SIZE)).expect("工作组数装得进 u32"),
                    1,
                    1,
                );
            }
        }
        for ((first, states, _), (_, verdicts)) in parts.iter().zip(&dispatch_and_verdict_buffers) {
            encoder.copy_buffer_to_buffer(
                verdicts,
                0,
                &staging,
                u64::try_from(first * VERDICT_WORDS * 4).expect("装得进 u64"),
                u64::try_from(states * VERDICT_WORDS * 4).expect("装得进 u64"),
            );
        }
        self.card.queue().submit(std::iter::once(encoder.finish()));
        let waiting_started = std::time::Instant::now();
        let slice = staging.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).expect("回调送得出去")
        });
        if let Err(error) = device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(
                LONGEST_WAIT_FOR_THE_CARD_SECONDS,
            )),
        }) {
            panic!(
                "等了 {LONGEST_WAIT_FOR_THE_CARD_SECONDS} 秒，卡 {:?} 没把这次派活（{count} 个状态，序号 {} 起）算完：{error:?}。\
                 多半是单次派活跑得太久被驱动掐了通道（看内核日志里的 Xid 109）；一次派活带的状态数按 LONGEST_DISPATCH_SECONDS 调",
                self.card.name, ordinals.start
            );
        }
        receiver.recv().expect("回调送达").expect("回读映射成功");
        let reading_started = std::time::Instant::now();
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
        let nanoseconds = |duration: std::time::Duration| {
            u64::try_from(duration.as_nanos()).expect("一次派活的纳秒数装得进 u64")
        };
        let relaxed = std::sync::atomic::Ordering::Relaxed;
        self.nanoseconds_preparing
            .fetch_add(nanoseconds(waiting_started - preparing_started), relaxed);
        self.nanoseconds_waiting_for_the_card
            .fetch_add(nanoseconds(reading_started - waiting_started), relaxed);
        self.nanoseconds_reading_back
            .fetch_add(nanoseconds(reading_started.elapsed()), relaxed);
        self.dispatches.fetch_add(1, relaxed);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{
        states_per_dispatch, states_per_scratch_lane, MAXIMUM_SCRATCH_LANES, SCRATCH_WORDS,
    };
    use crate::crash_judge_gpu::recovery_kernel_source;

    /// NVIDIA 卡上 wgpu 给的存储绑定上限。
    const BINDING_LIMIT: u64 = 2_147_483_644;

    /// 一条草稿缓冲装几个状态由绑定上限定；一次派活带几个由给草稿区的显存定，到 [`MAXIMUM_SCRATCH_LANES`] 条为止，至少 1 个。
    #[test]
    fn states_in_one_dispatch_follow_the_scratch_memory_up_to_the_lane_limit() {
        let bytes_per_state = u64::try_from(SCRATCH_WORDS * 4).expect("装得进 u64");
        assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 2170);
        assert_eq!(states_per_scratch_lane(1), 1, "再小的上限也带一个");
        // 12 GiB 的草稿区：12 × 1024 × 1024 × 1024 ÷ 989 248 = 13 024 个（向下取整）
        assert_eq!(
            states_per_dispatch(BINDING_LIMIT, 12 * 1024),
            usize::try_from(12 * 1024 * 1024 * 1024 / bytes_per_state).expect("装得进 usize")
        );
        assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 13_024);
        assert_eq!(
            states_per_dispatch(BINDING_LIMIT, 0),
            1,
            "一点显存都不给也带一个"
        );
        assert_eq!(
            states_per_dispatch(BINDING_LIMIT, 1024 * 1024),
            2170 * MAXIMUM_SCRATCH_LANES,
            "显存再多也只到条数上限"
        );
    }

    /// 量宽度照 2026-09-29 两张卡上量到的数（固定脚本到回退、展开 16、流的中间，一档加一条 2170 个状态的缓冲）：
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
    #[ignore = "要本机一张能用的 NVIDIA 卡；冷编每个内核几分钟"]
    fn each_kernel_compile_time_and_peak_memory_one_at_a_time() {
        use crate::crash_judge_gpu::{checker_scan_kernel_source, checker_walk_kernel_source};
        use std::io::Write;
        let cards = crate::gpu_unit_checks::usable_gpu_cards();
        let card = cards.first().expect("本机有一张能用的卡");
        println!(
            "GPU_KERNEL_COMPILE_CARD card={:?} index={} free_mebibytes={} peak_resident_mebibytes_before={}",
            card.name,
            card.index,
            card.free_memory_mebibytes,
            peak_resident_mebibytes()
        );
        for (kernel, source) in [
            ("recovery", recovery_kernel_source()),
            ("checker-scan", checker_scan_kernel_source()),
            ("checker-walk", checker_walk_kernel_source()),
        ] {
            println!(
                "GPU_KERNEL_COMPILE_STARTED kernel={kernel} source_bytes={}",
                source.len()
            );
            std::io::stdout().flush().expect("刷得出去");
            let started = std::time::Instant::now();
            let _pipeline = super::compute_pipeline(card, kernel, &source);
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

    /// 三个内核都在预算之内；打各自展开之后的大小。
    #[test]
    fn every_kernel_stays_within_the_inlined_size_budget() {
        for (name, source) in [
            ("recovery", recovery_kernel_source()),
            (
                "checker-scan",
                crate::crash_judge_gpu::checker_scan_kernel_source(),
            ),
            (
                "checker-walk",
                crate::crash_judge_gpu::checker_walk_kernel_source(),
            ),
        ] {
            let size = super::kernel_inlined_size(&source);
            println!(
                "GPU_KERNEL_INLINED kernel={name} expressions={} budget={} largest={:?}",
                size.expressions,
                super::LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL,
                size.largest_contributors
            );
            super::refuse_a_kernel_too_large_for_the_driver(name, &source);
        }
    }

    /// 走读内核里「走一遍根」回到六个调用点（第二台 580 版驱动 48G 里编不完的那个形态），编之前就被拒。
    #[test]
    fn the_walk_kernel_with_the_root_walk_called_from_six_sites_is_refused() {
        const CALL: &str =
            "w_walk_one(walk_kind, walk_base, walk_applied_on, walk_is_newest, fsid_low);";
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
    #[ignore = "要本机一张能用的 NVIDIA 卡，编一次几十秒"]
    fn recovery_kernel_compile_time_on_the_first_usable_card() {
        let cards = crate::gpu_unit_checks::usable_gpu_cards();
        let card = cards.first().expect("本机有一张能用的卡");
        let started = std::time::Instant::now();
        let _pipeline =
            super::compute_pipeline(card, "crash-judge-recovery", &recovery_kernel_source());
        println!(
            "GPU_KERNEL_COMPILE kernel=recovery seconds={:.1}",
            started.elapsed().as_secs_f64()
        );
    }
}
