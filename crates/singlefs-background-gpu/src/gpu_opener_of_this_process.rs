//! 开卡器（D26（后台整理与放置回收） 已定项 13 第 4 条「挂载时由调用方把它的实现递进去」）：运行库的特质
//! [`singlefs_mount_runtime::OpensTheBackgroundGpu`] 在产品 GPU 包这一侧的实现。进程级的四样由它持有：Vulkan 实例（第一次要开卡才建，之后反复用）、
//! 泄漏账（这个进程里之前几次挂载泄漏在一张卡上的字节，[`GpuMemoryLeakedInThisProcess`]）、各张卡上此刻在开的开卡
//! （[`OpeningsNotReturnedInThisProcess`]，各次挂载与挂着切换开卡共用：同一张卡上没到点、还在开的，后到的一方在自己的期限之内等它开完再开；
//! 到点之后还没回来的，同一次挂载重开、之后的挂载与切换同样不再起）、建过几个实例的计数。
//! 开成交给运行库的那一份带着读这一次挂载泄漏了哪几张卡的读法（切换关卡时与卸载时，[`opened_background_gpu_for_the_mount_runtime`]）。
//!
//! 开卡（枚举适配器在内：wgpu 在造 future 那一步同步做完枚举，等待上限管不到它）放工作线程，调用线程的期限分两步（[`OpeningDeadlinesOfTheOpener`]）：
//! 建实例、枚举、判卡清单与校验那一步至多等枚举期限；工作线程枚举完报回要开几张卡，期限延到「枚举期限 + 卡数 × 一张卡的期限」
//! （开卡按卡串行、只开参与的卡，卡数是参与的卡数；每一次开卡至多等单卡上限，到点当那一次没开成，由 `waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit` 管；
//! 开不成隔 1 分钟重开、至多重开 3 次，所以一张卡的期限是 4 个单卡上限加 3 个间隔，`waiting_for_the_gpu::ReopeningsOfAGpuNotOpened::longest_wait_of_one_gpu`）。
//! 到点挂载拒绝（[`BackgroundGpuNotOpened::TimedOut`] 带到点那一刻的期限），工作线程留在驱动里不杀（没有安全的杀法）；它回来之前闸住新开卡，不再多起一条枚举线程
//! （[`BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung`]；同一个驱动加载器里再起一条会不会也挂没量过）。同一个理由管到每一张卡：一张卡的一次开卡到点之后
//! 还没回来，重开（挂载那一刻与挂着切换同一个规矩）与之后的挂载都不在这张卡上再起一条，算那一次又没开成（`background_gpu::opening_of_a_card_within_the_limit`），
//! 所以整个进程里挂在驱动里的开卡线程每张卡至多一条；出路只有重启进程。它回来之后闸放开、开出来的东西没人要：
//! 开成的把手照卸载丢（还记着等超过上限的提交的卡泄漏、记进泄漏账；开成时量代价的提交都已做完，没有这种卡），开卡途中量代价出错而提交还在跑的卡
//! 在开那张卡的线程上已经泄漏、记进泄漏账（`background_gpu::started_gpu_with_its_cost_measured`）。它 panic 了闸同样放开。
//! 实例对象在超时之后的状态读不到：造不出真挂住的驱动，这一格核不了。
//!
//! 实例枚举到的驱动清单定在建它的那一刻：修好驱动要新建。[`GpuOpenerOfThisProcess::forget_the_instance_so_the_next_opening_creates_one`] 显式重建；
//! 同一进程里新建到一定数量之后 Vulkan 驱动不再认得出独显（前面的实例丢掉也不还额度），所以新建的次数有上限（[`VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST`]），到了就拒，出路是重启进程。
//!
//! 同一时刻只有一次开卡：多个挂载同时开卡时后到的排队，期限从轮到它才算，从调用起算的总等待可以超过期限（排在 n 个前面的之后，至多 (n + 1) 个期限）。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use singlefs_core::background_compute::{
    BackgroundComputeSettings, HardwareJudgementOfTheGpu, ReasonTheGpuIsOffTheBus,
};
use singlefs_mount_runtime::{
    BackgroundGpuNotOpened, GpuLeakedByTheBackgroundGpu, OpenedBackgroundGpu,
    OpensTheBackgroundGpu, ReadsTheGpusLeakedByTheBackgroundGpu,
};

use crate::adapter_inventory::BackgroundGpuSelection;
use crate::background_gpu::{
    open_background_gpu_with_the_memory_leaked_in_this_process, BackgroundGpuOpenRefused,
    OpeningsNotReturnedInThisProcess, VulkanInstanceOfThisProcess,
};
use crate::background_gpu_compute::{
    background_gpu_compute_of_the_opened_gpu, OpenedBackgroundGpuCompute,
};
use crate::gpus_of_this_host::{
    GpuLeakedWhenDropped, GpuMemoryLeakedInThisProcess, GpusLeakedWhenClosedAtASwitch,
    GpusLeakedWhenTheBackgroundGpuIsDropped,
};
use crate::waiting_for_the_gpu::{
    ReopeningsOfAGpuNotOpened, WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
};

/// 一个进程至多新建几个 Vulkan 实例（数是拍的：比第二台那一套驱动上量到的约 12 个低得多，那个数只对那一套驱动成立）。
pub const VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST: u32 = 4;

/// 调用线程等一次开卡的期限（从开卡起算）：枚举那一步（建实例、枚举适配器、判卡清单与校验）至多等 `enumeration`；工作线程枚举完报回要开 N 张卡，
/// 期限延到 `enumeration + N × each_gpu`。`each_gpu` 是一张卡连同重开至多等多久（产品的 [`Self::OF_THE_PRODUCT`]）；单测换上短的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpeningDeadlinesOfTheOpener {
    pub enumeration: Duration,
    pub each_gpu: Duration,
}

impl OpeningDeadlinesOfTheOpener {
    /// 产品的：枚举期限取单卡上限 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]；每张卡的期限是它连同产品的重开（[`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]）
    /// 至多等的那么久：每一次开卡至多单卡上限，开 1 + 重开次数 次，中间隔重开次数个间隔。
    pub const OF_THE_PRODUCT: Self = Self {
        enumeration: WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
        each_gpu: ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT
            .longest_wait_of_one_gpu(WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT),
    };

    /// 工作线程枚举完报回要开 `gpus_to_open` 张卡之后的期限（从开卡起算）：枚举期限加卡数乘每张卡的期限，超出 `Duration` 取它的最大值。
    #[must_use]
    pub fn after_the_enumeration(self, gpus_to_open: usize) -> Duration {
        let gpus_to_open_counted = u32::try_from(gpus_to_open).unwrap_or(u32::MAX);
        self.enumeration
            .saturating_add(self.each_gpu.saturating_mul(gpus_to_open_counted))
    }
}

/// 工作线程上这一次开卡要做的事：拿到这个进程的实例、设置、泄漏账与还没回来的开卡那一张表，枚举完把要开几张卡交给最后那个参数（调用线程照它延期限），
/// 交回开成的把手或开卡器报的原样。
type OpeningOnTheWorkerThread<Instance> = Arc<
    dyn Fn(
            &Instance,
            &BackgroundGpuSelection,
            &GpuMemoryLeakedInThisProcess,
            &OpeningsNotReturnedInThisProcess,
            &dyn Fn(usize),
        ) -> Result<OpenedBackgroundGpu, String>
        + Send
        + Sync,
>;

/// 工作线程送给调用线程的：枚举完要开几张卡（至多一次，先于结局），与这一次开卡的结局。
enum ReportOfTheOpeningThread {
    GpusToOpenAfterTheEnumeration(usize),
    Outcome(Result<OpenedBackgroundGpu, BackgroundGpuNotOpened>),
}

/// 建这个进程的一个实例。
type InstanceMaker<Instance> = Arc<dyn Fn() -> Instance + Send + Sync>;

/// 进程级实例与它的计数。建实例不持这把锁（驱动可能挂在那里）：读计数、丢实例都不被挂住。
struct InstanceOfThisProcess<Instance> {
    instance: Option<Instance>,
    instances_created: u32,
    /// 丢过几次实例：建实例途中丢过的，建好的那一个这一次开卡照用、不留给下一次开卡（它是丢之前的驱动清单建的）。
    instances_forgotten: u64,
}

/// 建实例途中占着的那一个额度：建实例 panic 了（展开走到这里）没占住实例，额度退回；建完了就不退。建到一半挂住时它留在那条线程上，额度照旧占着。
struct LimitGivenBackIfTheMakingPanics<'opener, Instance> {
    instance_of_this_process: &'opener Mutex<InstanceOfThisProcess<Instance>>,
    is_the_instance_made: bool,
}

impl<Instance> Drop for LimitGivenBackIfTheMakingPanics<'_, Instance> {
    fn drop(&mut self) {
        if !self.is_the_instance_made {
            self.instance_of_this_process
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .instances_created -= 1;
        }
    }
}

/// 工作线程结束时放开闸（走到末尾与 panic 展开都走到）。
struct GateOpenedWhenTheOpeningEnds {
    an_opening_is_hung: Arc<AtomicBool>,
}

impl Drop for GateOpenedWhenTheOpeningEnds {
    fn drop(&mut self) {
        self.an_opening_is_hung.store(false, Ordering::SeqCst);
    }
}

/// 见模块说明。泛型 `Instance` 只为让不要卡的单测换上不碰 Vulkan 的替身；产品是 [`VulkanInstanceOfThisProcess`]（[`Self::of_this_process`]）。
pub struct GpuOpenerOfThisProcess<Instance> {
    opening_deadlines: OpeningDeadlinesOfTheOpener,
    /// 同一时刻只有一次开卡：两次挂载同时开卡时后到的那一次在这里等（这段等待不算进期限）。
    one_opening_at_a_time: Mutex<()>,
    /// 上一次开卡的工作线程还没回来（调用线程已经到点不等了）。
    an_opening_is_hung: Arc<AtomicBool>,
    instance_of_this_process: Arc<Mutex<InstanceOfThisProcess<Instance>>>,
    memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
    openings_not_returned_in_this_process: OpeningsNotReturnedInThisProcess,
    make_an_instance: InstanceMaker<Instance>,
    open_on_the_worker_thread: OpeningOnTheWorkerThread<Instance>,
}

impl GpuOpenerOfThisProcess<VulkanInstanceOfThisProcess> {
    /// 产品的开卡器：一个进程建一份、跨多次挂载留着，期限是 [`OpeningDeadlinesOfTheOpener::OF_THE_PRODUCT`]。实例第一次要开卡才建：选 `cpu` 的挂载不碰 Vulkan。
    #[must_use]
    pub fn of_this_process() -> Self {
        Self::with_the_instance_maker_and_the_opening(
            OpeningDeadlinesOfTheOpener::OF_THE_PRODUCT,
            Arc::new(VulkanInstanceOfThisProcess::created_for_this_process),
            Arc::new(the_background_gpu_opened_by_the_product_path),
        )
    }

    /// 同 [`Self::of_this_process`]，实例用调用方已经建好的那一份（真卡测试枚举服务交来的卡时建过：一个进程只建一个）。
    #[must_use]
    pub fn over_the_vulkan_instance(vulkan_instance: VulkanInstanceOfThisProcess) -> Self {
        Self::with_the_instance_maker_and_the_opening(
            OpeningDeadlinesOfTheOpener::OF_THE_PRODUCT,
            Arc::new(move || vulkan_instance.clone()),
            Arc::new(the_background_gpu_opened_by_the_product_path),
        )
    }
}

/// 工作线程上的产品开卡（[`open_background_gpu_with_the_memory_leaked_in_this_process`]，枚举完把要开几张卡交给 `report_the_gpus_to_open`），
/// 开成的交成核心层收的把手（[`background_gpu_compute_of_the_opened_gpu`]），结局换成运行库收的样子（[`opened_background_gpu_for_the_mount_runtime`]）。
fn the_background_gpu_opened_by_the_product_path(
    vulkan_instance: &VulkanInstanceOfThisProcess,
    selection: &BackgroundGpuSelection,
    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
    openings_not_returned_in_this_process: &OpeningsNotReturnedInThisProcess,
    report_the_gpus_to_open: &dyn Fn(usize),
) -> Result<OpenedBackgroundGpu, String> {
    open_background_gpu_with_the_memory_leaked_in_this_process(
        selection,
        memory_leaked_in_this_process,
        openings_not_returned_in_this_process,
        vulkan_instance,
        report_the_gpus_to_open,
    )
    .map(background_gpu_compute_of_the_opened_gpu)
    .map(opened_background_gpu_for_the_mount_runtime)
    .map_err(|refused| description_of_the_refused_opening(&refused))
}

/// 挂载被拒时报给运行库的那一段文字：拒绝原样（`Debug`）；一张参与的卡重开用完都没开成（[`BackgroundGpuOpenRefused::GpuUnusable`]）的，
/// 后面接着写出照判它开不成那一刻读的 PCI 状态判出的结论（D26（后台整理与放置回收） 已定项 13 第 4 条「只有卡不在总线上……才判硬件坏」，
/// `singlefs_core::background_compute::hardware_judgement_of_the_pci_state`）：卡不在总线上与依据是哪一条，或没判成不在总线上，没读成的写没读。
/// 运行库只收文字（`BackgroundGpuNotOpened::RefusedByTheOpening`），判定写进文字里，看报错的人不用从读数里自己判。
fn description_of_the_refused_opening(refused: &BackgroundGpuOpenRefused) -> String {
    match refused {
        BackgroundGpuOpenRefused::GpuUnusable {
            pci_state_when_judged_unusable,
            ..
        } => format!(
            "{refused:?}; hardware_judgement={}",
            text_of_the_hardware_judgement(pci_state_when_judged_unusable.hardware_judgement())
        ),
        BackgroundGpuOpenRefused::SettingsRefused(..)
        | BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(..)
        | BackgroundGpuOpenRefused::DiscreteGpuListNotSupported(..)
        | BackgroundGpuOpenRefused::AdapterEnumerationFailed { .. } => format!("{refused:?}"),
    }
}

/// 照 PCI 读数判出的结论写成挂载报错里的那一句；`None` 是没读成（卡的标识当不了目录名，或不是读的时刻）。
fn text_of_the_hardware_judgement(judgement: Option<HardwareJudgementOfTheGpu>) -> &'static str {
    match judgement {
        Some(HardwareJudgementOfTheGpu::OffTheBus(
            ReasonTheGpuIsOffTheBus::DeviceDirectoryNotPresent,
        )) => "卡不在总线上：那个 PCI 地址的目录不在",
        Some(HardwareJudgementOfTheGpu::OffTheBus(ReasonTheGpuIsOffTheBus::DeviceDisabled)) => {
            "卡不在总线上：使能位是关的"
        }
        Some(HardwareJudgementOfTheGpu::NotJudgedOffTheBus) => "没判成卡不在总线上",
        None => "没读到 PCI 状态，不判",
    }
}

/// 开成的后台 GPU 换成运行库收的样子：核心层收的把手、开卡的报告（开着的、因显存上限不用的、没收进清单的卡），与读这一次挂载泄漏了哪几张卡的那一份
/// （切换关卡时累计的、卸载时丢掉的，[`GpusLeakedByTheOpenedBackgroundGpu`]）。
#[must_use]
pub fn opened_background_gpu_for_the_mount_runtime(
    opened: OpenedBackgroundGpuCompute,
) -> OpenedBackgroundGpu {
    OpenedBackgroundGpu {
        report_of_the_opening: format!(
            "gpus_opened={:?} gpus_left_out={:?} unnamed_gpus_left_off={:?}",
            opened.gpus_opened_at_the_mount,
            opened.gpus_left_out_under_their_memory_limit,
            opened.unnamed_discrete_gpus_left_off_the_list
        ),
        gpus_leaked: Some(Arc::new(GpusLeakedByTheOpenedBackgroundGpu {
            when_closed_at_a_switch: opened.gpus_leaked_when_closed_at_a_switch,
            when_dropped: opened.gpus_leaked_when_dropped,
        })),
        gpu_compute: opened.gpu_compute,
    }
}

/// 读产品开卡器开成的后台 GPU 泄漏了哪几张卡（运行库的 [`ReadsTheGpusLeakedByTheBackgroundGpu`]）：切换关卡时累计的那一份与卸载时丢掉的那一份，
/// 各带泄漏那一刻本包在那张卡上占着的显存。
#[derive(Debug)]
struct GpusLeakedByTheOpenedBackgroundGpu {
    when_closed_at_a_switch: GpusLeakedWhenClosedAtASwitch,
    when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
}

/// 本包记的一张泄漏的卡换成运行库报的样子。
fn gpu_leaked_for_the_mount_runtime(leaked: GpuLeakedWhenDropped) -> GpuLeakedByTheBackgroundGpu {
    GpuLeakedByTheBackgroundGpu {
        device: leaked.device,
        memory_held_on_the_device: leaked.bytes_held_on_the_device,
    }
}

impl ReadsTheGpusLeakedByTheBackgroundGpu for GpusLeakedByTheOpenedBackgroundGpu {
    fn gpus_leaked_when_closed_at_a_switch(&self) -> Vec<GpuLeakedByTheBackgroundGpu> {
        self.when_closed_at_a_switch
            .gpus_leaked()
            .into_iter()
            .map(gpu_leaked_for_the_mount_runtime)
            .collect()
    }

    fn gpus_leaked_when_dropped(&self) -> Option<Vec<GpuLeakedByTheBackgroundGpu>> {
        self.when_dropped.gpus_leaked().map(|gpus_leaked| {
            gpus_leaked
                .into_iter()
                .map(gpu_leaked_for_the_mount_runtime)
                .collect()
        })
    }
}

impl<Instance: Clone + Send + 'static> GpuOpenerOfThisProcess<Instance> {
    /// 实例怎么建、工作线程上怎么开卡由调用方给：产品走 [`GpuOpenerOfThisProcess::of_this_process`]，不要卡的单测换上替身钉期限、闸与计数。
    #[must_use]
    pub fn with_the_instance_maker_and_the_opening(
        opening_deadlines: OpeningDeadlinesOfTheOpener,
        make_an_instance: InstanceMaker<Instance>,
        open_on_the_worker_thread: OpeningOnTheWorkerThread<Instance>,
    ) -> Self {
        GpuOpenerOfThisProcess {
            opening_deadlines,
            one_opening_at_a_time: Mutex::new(()),
            an_opening_is_hung: Arc::new(AtomicBool::new(false)),
            instance_of_this_process: Arc::new(Mutex::new(InstanceOfThisProcess {
                instance: None,
                instances_created: 0,
                instances_forgotten: 0,
            })),
            memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            openings_not_returned_in_this_process:
                OpeningsNotReturnedInThisProcess::with_none_not_returned(),
            make_an_instance,
            open_on_the_worker_thread,
        }
    }

    /// 这个进程建过几个实例。
    #[must_use]
    pub fn instances_created(&self) -> u32 {
        self.instance_of_this_process
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .instances_created
    }

    /// 这个进程里各张卡上在开的开卡（到点之后还没回来的照它不再起，没到点的照它排队）：交给每一次开卡的都是这一张（[`OpeningsNotReturnedInThisProcess`]）。
    #[must_use]
    pub fn openings_not_returned_in_this_process(&self) -> OpeningsNotReturnedInThisProcess {
        self.openings_not_returned_in_this_process.clone()
    }

    /// 上一次开卡的工作线程还在驱动里没回来。
    #[must_use]
    pub fn is_an_opening_hung(&self) -> bool {
        self.an_opening_is_hung.load(Ordering::SeqCst)
    }

    /// 修好驱动之后用：丢掉这个进程留着的实例，下一次开卡新建一个（已开着的把手持有自己的设备，不动）。只改内存，新建的计数不还。
    pub fn forget_the_instance_so_the_next_opening_creates_one(&self) {
        let mut held = self
            .instance_of_this_process
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        held.instance = None;
        held.instances_forgotten += 1;
    }

    /// 这一次开卡用的实例：有就用留着的，没有就新建、计数加一（额度在建之前就记，建到一半挂住也算用了一个；建实例 panic 了退回）；计数到上限就拒。
    /// 在工作线程上调：建实例装 Vulkan 加载器，驱动也可能挂在这里，所以建的时候不持那把锁。同一时刻至多一条工作线程走到这里（闸）。
    fn the_instance_or_a_new_one_under_the_limit(
        instance_of_this_process: &Mutex<InstanceOfThisProcess<Instance>>,
        make_an_instance: &(dyn Fn() -> Instance + Send + Sync),
    ) -> Result<Instance, BackgroundGpuNotOpened> {
        let mut held = instance_of_this_process
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(instance) = &held.instance {
            return Ok(instance.clone());
        }
        if held.instances_created >= VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST {
            return Err(BackgroundGpuNotOpened::InstanceLimitOfThisProcessReached {
                instances_created: held.instances_created,
            });
        }
        held.instances_created += 1;
        let instances_forgotten_before_the_making = held.instances_forgotten;
        drop(held);
        let mut limit_given_back_if_the_making_panics = LimitGivenBackIfTheMakingPanics {
            instance_of_this_process,
            is_the_instance_made: false,
        };
        let instance = make_an_instance();
        limit_given_back_if_the_making_panics.is_the_instance_made = true;
        drop(limit_given_back_if_the_making_panics);
        let mut held_after_the_making = instance_of_this_process
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if held_after_the_making.instances_forgotten == instances_forgotten_before_the_making {
            held_after_the_making.instance = Some(instance.clone());
        }
        Ok(instance)
    }
}

impl<Instance: Clone + Send + 'static> OpensTheBackgroundGpu for GpuOpenerOfThisProcess<Instance> {
    fn open_the_background_gpu(
        &self,
        settings: &BackgroundComputeSettings,
    ) -> Result<OpenedBackgroundGpu, BackgroundGpuNotOpened> {
        let _only_this_opening = self
            .one_opening_at_a_time
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if self.an_opening_is_hung.load(Ordering::SeqCst) {
            return Err(BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung);
        }
        self.an_opening_is_hung.store(true, Ordering::SeqCst);
        let (outcome_sender, outcome_receiver) = channel();
        let an_opening_is_hung = Arc::clone(&self.an_opening_is_hung);
        let instance_of_this_process = Arc::clone(&self.instance_of_this_process);
        let make_an_instance = Arc::clone(&self.make_an_instance);
        let open_on_the_worker_thread = Arc::clone(&self.open_on_the_worker_thread);
        let memory_leaked_in_this_process = self.memory_leaked_in_this_process.clone();
        let openings_not_returned_in_this_process =
            self.openings_not_returned_in_this_process.clone();
        let selection = BackgroundGpuSelection::of_the_settings(settings);
        let started = std::thread::Builder::new()
            .name("singlefs-gpu-opener".to_string())
            .spawn(move || {
                let gate_opened_when_the_opening_ends =
                    GateOpenedWhenTheOpeningEnds { an_opening_is_hung };
                let report_the_gpus_to_open = |gpus_to_open: usize| {
                    // 送不出去只有一种：调用线程到点不等了，这个数没人要。
                    let _nobody_waits_for_this_opening_any_more = outcome_sender.send(
                        ReportOfTheOpeningThread::GpusToOpenAfterTheEnumeration(gpus_to_open),
                    );
                };
                let outcome = Self::the_instance_or_a_new_one_under_the_limit(
                    &instance_of_this_process,
                    &*make_an_instance,
                )
                .and_then(|instance| {
                    open_on_the_worker_thread(
                        &instance,
                        &selection,
                        &memory_leaked_in_this_process,
                        &openings_not_returned_in_this_process,
                        &report_the_gpus_to_open,
                    )
                    .map_err(|description| {
                        BackgroundGpuNotOpened::RefusedByTheOpening { description }
                    })
                });
                // 闸先放开再送：调用线程收到结局时下一次开卡不会被这一次闸住。
                drop(gate_opened_when_the_opening_ends);
                // 送不出去只有一种：调用线程到点不等了、这次的结局没人要。开成的把手随这里丢掉，照卸载丢（模块说明）。
                let _nobody_waits_for_this_opening_any_more =
                    outcome_sender.send(ReportOfTheOpeningThread::Outcome(outcome));
            });
        if let Err(error) = started {
            self.an_opening_is_hung.store(false, Ordering::SeqCst);
            return Err(BackgroundGpuNotOpened::RefusedByTheOpening {
                description: format!("起不了开卡的工作线程：{error}"),
            });
        }
        let opening_started_at = Instant::now();
        let mut deadline_of_the_opening = self.opening_deadlines.enumeration;
        // 每一轮要么交回，要么收到工作线程报的卡数、把期限换成枚举完的那一个；工作线程至多报一次卡数，之后只剩结局、断开或到点。
        loop {
            let time_left = deadline_of_the_opening.saturating_sub(opening_started_at.elapsed());
            match outcome_receiver.recv_timeout(time_left) {
                Ok(ReportOfTheOpeningThread::GpusToOpenAfterTheEnumeration(gpus_to_open)) => {
                    deadline_of_the_opening =
                        self.opening_deadlines.after_the_enumeration(gpus_to_open);
                }
                Ok(ReportOfTheOpeningThread::Outcome(outcome)) => return outcome,
                Err(RecvTimeoutError::Timeout) => {
                    return Err(BackgroundGpuNotOpened::TimedOut {
                        deadline: deadline_of_the_opening,
                    })
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(BackgroundGpuNotOpened::RefusedByTheOpening {
                        description: "开卡的工作线程 panic 了".to_string(),
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use singlefs_core::background_compute::{
        GpuDeviceIdentifier, PciDeviceDirectoryReading, PciDeviceEnable, PciFileReading,
        PciStateOfTheGpu, PciStateOfTheGpuReadWhenJudgedUnusable,
    };

    use crate::background_gpu::GpuFailureCause;
    use crate::pci_state_of_the_gpu::pci_state_read_under_the_root_not_present_for_tests;

    /// 照 `device_enable` 造一份读数：目录在，链路与出错计数都读到。
    fn pci_state_read_with_the_device_present_and_enabled_as(
        device_enable: PciDeviceEnable,
    ) -> PciStateOfTheGpuReadWhenJudgedUnusable {
        let read = |text: &str| PciFileReading::Read(text.to_string());
        PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(PciStateOfTheGpu {
            pci_bus_address: "0000:41:00.0".to_string(),
            device_directory: PathBuf::from("/sys/bus/pci/devices/0000:41:00.0"),
            device_directory_reading: PciDeviceDirectoryReading::Present,
            device_enable: PciFileReading::Read(device_enable),
            current_link_speed: read("2.5 GT/s PCIe"),
            current_link_width: read("1"),
            maximum_link_speed: read("32.0 GT/s PCIe"),
            maximum_link_width: read("16"),
            advanced_error_reporting_correctable: read("TOTAL_ERR_COR 0"),
            advanced_error_reporting_fatal: read("TOTAL_ERR_FATAL 0"),
            advanced_error_reporting_nonfatal: read("TOTAL_ERR_NONFATAL 0"),
        }))
    }

    fn refused_with_the_card_unusable(
        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
    ) -> BackgroundGpuOpenRefused {
        BackgroundGpuOpenRefused::GpuUnusable {
            device: GpuDeviceIdentifier::of_opaque_bytes(b"0000:41:00.0".to_vec()),
            causes_of_each_opening: vec![GpuFailureCause::DeviceOrPipelineDidNotStart {
                reason: "替身：起不了设备".to_string(),
            }],
            pci_state_when_judged_unusable,
        }
    }

    /// D26（后台整理与放置回收） 已定项 13 第 4 条「挂载那一刻开卡重开用完、拒绝挂载的，读数写进挂载的报错」与「只有卡不在总线上……才判硬件坏」：
    /// 挂载拒绝的文字是拒绝原样接着照读数判出的结论——目录不在、使能位关着各写卡不在总线上与那一条依据，开着的写没判成不在总线上，没读成的写没读；
    /// 读数本身照原样留在文字里。
    #[test]
    fn the_mount_refused_for_a_card_unusable_writes_the_hardware_judgement_of_its_pci_state() {
        let readings_and_judgements = [
            (
                pci_state_read_under_the_root_not_present_for_tests("0000:41:00.0"),
                "卡不在总线上：那个 PCI 地址的目录不在",
            ),
            (
                pci_state_read_with_the_device_present_and_enabled_as(PciDeviceEnable::Disabled),
                "卡不在总线上：使能位是关的",
            ),
            (
                pci_state_read_with_the_device_present_and_enabled_as(PciDeviceEnable::Enabled {
                    enable_count: 1,
                }),
                "没判成卡不在总线上",
            ),
            (
                PciStateOfTheGpuReadWhenJudgedUnusable::PciBusAddressNotADirectoryName {
                    device: GpuDeviceIdentifier::of_opaque_bytes(b"..".to_vec()),
                },
                "没读到 PCI 状态，不判",
            ),
        ];
        for (pci_state_when_judged_unusable, judgement) in readings_and_judgements {
            let refused = refused_with_the_card_unusable(pci_state_when_judged_unusable);
            assert_eq!(
                description_of_the_refused_opening(&refused),
                format!("{refused:?}; hardware_judgement={judgement}")
            );
        }
    }

    /// 别的几种挂载拒绝不带 PCI 读数：文字是拒绝原样，不接判定。
    #[test]
    fn the_mount_refused_for_another_reason_is_described_as_the_refusal_alone() {
        let refused = BackgroundGpuOpenRefused::AdapterEnumerationFailed {
            cause: GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: Duration::from_secs(10),
            },
        };
        assert_eq!(
            description_of_the_refused_opening(&refused),
            format!("{refused:?}")
        );
    }
}
