//! 找卡与起卡：Vulkan 下的 NVIDIA 独显对上 `nvidia-smi` 报的序号与显存，在卡上起设备、编单元校验和的管线，
//! 设备上装没接住的错误与设备丢失的回调，不归卡池的设备按总线记数；不走卡池的取卡入口按配置的优先级取卡；驱动的盘上着色器缓存。

use super::*;

/// NVIDIA 的厂商号（Vulkan 适配器与 PCI 设备报的都是它）：崩溃放量只用 NVIDIA 独显，[`gpu_cards_by_priority`] 按它筛卡，
/// 测试取卡的判定（[`gpu_test_start`]）按它认 Vulkan 适配器与 PCI 上的显示控制器。
pub const NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE;

/// wgpu 没接住的错误（装在每个设备上，[`start_the_card`]）：Out of Memory 先记进这个设备的情况，再与别的错一样照 wgpu 默认的做法 panic
/// （缓冲、管线没建成，往下跑判出来的也不作数）。
fn handle_an_uncaptured_wgpu_error(condition: &DeviceConditionOfACard, error: wgpu::Error) {
    match &error {
        wgpu::Error::OutOfMemory { .. } => {
            condition.record_an_out_of_memory(format!("没接住的 {error}"));
        }
        wgpu::Error::Validation { .. } | wgpu::Error::Internal { .. } => {}
    }
    panic!("wgpu error: {error}\n")
}

/// 设备丢失的回调（装在每个设备上，[`start_the_card`]）：记进这个设备的情况，连同 wgpu 给的丢失原因与那一句、照那一句认出的缘由
/// （[`cause_of_the_device_loss`]；卡池下一次借这张卡时照它重起，因显存不够丢的、或丢了之后撞上 Out of Memory 的，照它认成显存不够）。
fn record_the_loss_of_the_device(
    condition: &DeviceConditionOfACard,
    reason: wgpu::DeviceLostReason,
    message: &str,
) {
    condition.record_the_device_lost(
        format!("{reason:?}：{message}"),
        cause_of_the_device_loss(message),
    );
}

/// wgpu 丢设备时给的那一句是哪一种：与它因显存不够丢设备时给的那一句（`wgpu::hal::DeviceError::OutOfMemory`，wgpu-core 把它当致命、
/// 照它的这一句丢设备）相同的是显存不够（攻方甲形）。wgpu 不另给丢失的缘由
/// （回调的原因只有 `Unknown`、`Destroyed`），只能比那一句；那一句取自 wgpu 自己，不在这里另抄一份。
fn cause_of_the_device_loss(message: &str) -> CauseOfTheDeviceLoss {
    if message == wgpu::hal::DeviceError::OutOfMemory.to_string() {
        CauseOfTheDeviceLoss::GpuMemoryShort
    } else {
        CauseOfTheDeviceLoss::Other
    }
}

/// 一张卡起不来的两种。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardStartFailure {
    /// 起设备、编单元校验和的管线时显存不够（`request_device` 交回 wgpu 的 Out of Memory，或编管线时错误范围接到 Out of Memory）：
    /// 卡池的调用方报本次未跑。
    OutOfGpuMemory { reason: String },
    /// 别的原因起不来（驱动坏了、适配器不给要的上限）。
    DeviceNotStarted { reason: String },
}

/// `request_device` 交回的错是哪一种：与 wgpu 的「设备显存不够」那一句（`wgpu_core::device::DeviceError::OutOfMemory`）相同的是显存不够。
/// wgpu 不公开这个错的成员，只能比它说的那一句；那一句取自 wgpu 自己，不在这里另抄一份。
fn failure_of_a_device_request(
    error: &wgpu::RequestDeviceError,
    which_card: &str,
) -> CardStartFailure {
    let reason = format!("{which_card}上起不了设备（{error}）");
    if error.to_string() == wgpu::wgc::device::DeviceError::OutOfMemory.to_string() {
        CardStartFailure::OutOfGpuMemory { reason }
    } else {
        CardStartFailure::DeviceNotStarted { reason }
    }
}

/// 这个进程在哪几张卡（PCI 总线号）上有不走卡池账的设备、每张几台（起之前就记、那一台的句柄都丢了就减，[`started_with_its_owner_recorded`]）。
/// 是这个进程的事实，所以是进程里一份（攻方乙形）。
static BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL: BusesWithDevicesStartedOutsideTheBooks =
    BusesWithDevicesStartedOutsideTheBooks::new();

/// 这个进程在这张卡（PCI 总线号，[`pci_bus_identifier_without_the_domain`] 写法）上有（或正在起、起不来过）不走卡池账的设备没有：有，卡池读这张卡时
/// 这个进程占着的那一份分不出哪些是借据建的（`singlefs_verification::gpu_memory_ledger::OccupiedByThisProcess::AlsoOutsideTheBooks`）。
pub(super) fn has_devices_started_outside_the_card_pool_on_the_bus(bus: &str) -> bool {
    BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL.has_devices_on_the_bus(bus)
}

/// 这个进程此刻记着几台不走卡池账的设备（各条总线加起来：还有句柄的，加起不来过的）。只有卡池起设备的进程是 0：同一个进程里只有一套显存账。
#[must_use]
pub fn devices_marked_outside_the_card_pool_in_this_process() -> usize {
    BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL.devices_marked_on_every_bus()
}

/// 照设备归谁起（`start` 起设备）：卡池的直接起，交设备与 `None`；不归卡池的先在总线 `bus` 上多记一台、再起（攻方乙形：
/// 起好了才记，起设备到记下那一段里卡池读到的这个进程占着的已经有它，被当成借据建上的），交设备与这一台在总线上的那一记
/// （调用方放进这台设备的每一个克隆，都丢了才减一台）。
///
/// # Errors
/// `start` 的错。
fn started_with_its_owner_recorded<Device>(
    owner: DeviceOwnerOnTheCard,
    bus: &str,
    start: impl FnOnce() -> Result<Device, CardStartFailure>,
) -> Result<(Device, Option<DeviceOutsideTheBooksMarkedOnTheBus<'static>>), CardStartFailure> {
    match owner {
        DeviceOwnerOnTheCard::TheCardPool => start().map(|device| (device, None)),
        DeviceOwnerOnTheCard::OutsideTheCardPool => {
            BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL
                .start_a_device_marked_first(bus, start)
                .map(|(device, marked_on_the_bus)| (device, Some(marked_on_the_bus)))
        }
    }
}

/// 起设备那一行 GPU 阶段现场的阶段名（文件头）。
pub const DEVICE_START_LIFECYCLE_PHASE: &str = "device_start";

/// 编管线那一行 GPU 阶段现场的阶段名（文件头）。
pub const PIPELINE_COMPILE_LIFECYCLE_PHASE: &str = "pipeline_compile";

/// 调 `start` 起序号 `card_index`、总线 `bus` 那张卡上归 `owner` 的一台设备，起完（起没起好都算）在 `trace` 上打一行
/// [`DEVICE_START_LIFECYCLE_PHASE`]（细节的写法在文件头），交 `start` 的结果。
///
/// # Errors
/// `start` 的错。
fn device_started_with_a_lifecycle_line<Device>(
    card_index: u32,
    bus: &str,
    owner: DeviceOwnerOnTheCard,
    start: impl FnOnce() -> Result<Device, CardStartFailure>,
    trace: &dyn Fn(&str, std::fmt::Arguments<'_>),
) -> Result<Device, CardStartFailure> {
    let start_began = std::time::Instant::now();
    let started = start();
    let start_microseconds = start_began.elapsed().as_micros();
    let outcome = match &started {
        Ok(_device) => "started",
        Err(CardStartFailure::OutOfGpuMemory { .. }) => "out-of-gpu-memory",
        Err(CardStartFailure::DeviceNotStarted { .. }) => "device-not-started",
    };
    trace(
        DEVICE_START_LIFECYCLE_PHASE,
        format_args!(
            "card={card_index} pci={bus:?} owner={} outcome={outcome} start_microseconds={start_microseconds}",
            owner.name_in_the_lifecycle_line()
        ),
    );
    started
}

/// 卡池读 `buses` 这几张卡显存的那一份：每读一次取一份 `run_the_report` 的报告（卡池是跑一次 `nvidia-smi`），按这个进程的进程号认它占着的，
/// 这个进程在哪张卡上起过不走卡池账的设备照 [`has_devices_started_outside_the_card_pool_on_the_bus`] 认。
pub(super) fn card_memory_reader_of_the_card_pool(
    run_the_report: Box<dyn Fn() -> Result<String, String> + Send + Sync>,
    buses: Vec<String>,
) -> CardMemoryReader {
    card_memory_reader_of_one_report_per_reading(
        run_the_report,
        std::process::id(),
        buses,
        Box::new(has_devices_started_outside_the_card_pool_on_the_bus),
    )
}

/// `nvidia-smi` 报的每张卡：PCI 总线号（小写、去掉前导的 0 域）→ 序号与空闲显存。起不来 `nvidia-smi` 就是空表。
pub(super) fn reported_cards_by_pci_bus() -> std::collections::BTreeMap<String, ReportedCard> {
    let Ok(output) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=pci.bus_id,index,memory.free",
            "--format=csv,noheader,nounits",
        ])
        .output()
    else {
        return std::collections::BTreeMap::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split(',');
            let bus = fields.next()?;
            let index = fields.next()?.trim().parse().ok()?;
            let free_memory_mebibytes = fields.next()?.trim().parse().ok()?;
            Some((
                pci_bus_identifier_without_the_domain(bus),
                ReportedCard {
                    index,
                    free_memory_mebibytes,
                },
            ))
        })
        .collect()
}

/// 卡池读显存用的那一份报告：跑一次 `nvidia-smi`（`singlefs_verification::gpu_memory_ledger::NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS`），
/// 每张卡的显存与每个进程占的出自这一次。
///
/// # Errors
/// `nvidia-smi` 起不来或退了非 0。
pub(super) fn nvidia_smi_memory_and_processes_report() -> Result<String, String> {
    let output = std::process::Command::new("nvidia-smi")
        .args(NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS)
        .output()
        .map_err(|error| format!("nvidia-smi 起不来：{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "nvidia-smi {NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS:?} 退了 {}：{}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// 这个进程的 Vulkan 实例，整个进程只建一份、第一次用时建。
///
/// 为什么只建一份：每建一个实例，Vulkan 加载器都把本机装着的每个 ICD（intel、radeon、lvp 带整份 LLVM、nouveau……）dlopen 一遍，
/// 它们的线程局部存储吃掉 glibc 给动态库预留的静态 TLS 余量，dlclose 不一定还回来；几十次之后再 dlopen NVIDIA 的
/// `libnvidia-tls.so` 就报「cannot allocate memory in static TLS block」，加载器只剩 llvmpipe，配置点名的卡「用不了」
/// （库单测进程里跑到 `crash_injection::tests::` 那一段才起第一个实例就撞上，单跑同一条用例绿）。
/// 一份实例从头用到尾，ICD 只装一次。
pub(super) fn vulkan_instance_of_this_process() -> &'static wgpu::Instance {
    static VULKAN_INSTANCE: std::sync::OnceLock<wgpu::Instance> = std::sync::OnceLock::new();
    VULKAN_INSTANCE.get_or_init(|| {
        VULKAN_INSTANCES_CREATED_IN_THIS_PROCESS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        enable_the_driver_shader_disk_cache();
        load_only_the_nvidia_vulkan_driver_when_the_user_named_none();
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        })
    })
}

/// [`vulkan_instance_of_this_process`] 建了几次实例（只供单测核「整个进程一份」）。
static VULKAN_INSTANCES_CREATED_IN_THIS_PROCESS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// 本机 NVIDIA 驱动的 Vulkan ICD 清单文件（Ubuntu 的包放在这里）。
const NVIDIA_VULKAN_ICD_MANIFEST: &str = "/usr/share/vulkan/icd.d/nvidia_icd.json";

/// 用户没点名 Vulkan 驱动（`VK_DRIVER_FILES`、`VK_ICD_FILENAMES` 都没设）而本机有 NVIDIA 的 ICD 清单时，只让加载器装 NVIDIA 这一份：
/// 判器只认 NVIDIA 独显，别的 ICD（Mesa 的 intel、radeon、lvp……）装了也只是白占静态 TLS 与时间；用户设了的照用户的。
fn load_only_the_nvidia_vulkan_driver_when_the_user_named_none() {
    if std::env::var_os("VK_DRIVER_FILES").is_some()
        || std::env::var_os("VK_ICD_FILENAMES").is_some()
    {
        return;
    }
    if std::path::Path::new(NVIDIA_VULKAN_ICD_MANIFEST).is_file() {
        std::env::set_var("VK_DRIVER_FILES", NVIDIA_VULKAN_ICD_MANIFEST);
    }
}

/// 本机 Vulkan 下每张 NVIDIA 独显，带 `nvidia-smi` 按 PCI 总线号报的序号与空闲显存（`nvidia-smi` 里对不上的卡不算）；一个设备都不起。
pub(super) fn nvidia_discrete_cards_reported_by_nvidia_smi() -> Vec<DiscoveredCard> {
    let instance = vulkan_instance_of_this_process();
    let reported_by_bus = reported_cards_by_pci_bus();
    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN));
    if adapters.is_empty() {
        // 一张卡都没枚举到时把看到的说出来：是 Vulkan 这一层没给适配器（驱动、ICD、实例起不来），还是 nvidia-smi 那边对不上
        eprintln!(
            "GPU_DISCOVERY_EMPTY backend=vulkan adapters=0 nvidia_smi_rows={} __GL_SHADER_DISK_CACHE_PATH={:?} VK_ICD_FILENAMES={:?} VK_DRIVER_FILES={:?}",
            reported_by_bus.len(),
            std::env::var_os("__GL_SHADER_DISK_CACHE_PATH"),
            std::env::var_os("VK_ICD_FILENAMES"),
            std::env::var_os("VK_DRIVER_FILES"),
        );
    }
    let adapter_summaries: Vec<String> = adapters
        .iter()
        .map(|adapter| {
            let information = adapter.get_info();
            format!(
                "(vendor={:#x} type={:?} pci={:?} name={:?})",
                information.vendor,
                information.device_type,
                information.device_pci_bus_id,
                information.name
            )
        })
        .collect();
    let discovered: Vec<DiscoveredCard> = adapters
        .into_iter()
        .filter_map(|adapter| {
            let information = adapter.get_info();
            if information.vendor != NVIDIA_VENDOR_IDENTIFIER
                || information.device_type != wgpu::DeviceType::DiscreteGpu
            {
                return None;
            }
            let pci_bus_identifier =
                pci_bus_identifier_without_the_domain(&information.device_pci_bus_id);
            let reported = *reported_by_bus.get(&pci_bus_identifier)?;
            Some(DiscoveredCard {
                adapter,
                name: information.name,
                driver_version: driver_version_of_the_adapter(
                    &information.driver,
                    &information.driver_info,
                ),
                pci_bus_identifier,
                reported,
            })
        })
        .collect();
    if discovered.is_empty() && !adapter_summaries.is_empty() {
        // 枚举到了适配器却一张都没对上：把每张的厂商、类型、PCI 号与 nvidia-smi 报的总线号都说出来
        eprintln!(
            "GPU_DISCOVERY_UNMATCHED adapters={} nvidia_smi_buses={:?}",
            adapter_summaries.join(" "),
            reported_by_bus.keys().collect::<Vec<_>>()
        );
    }
    discovered
}

/// 驱动的名字与版本那一句（wgpu adapter info 的 `driver`、`driver_info` 两项，空格接起来；两项都空写一句没报）。
#[must_use]
pub fn driver_version_of_the_adapter(driver: &str, driver_information: &str) -> String {
    match (driver.trim(), driver_information.trim()) {
        ("", "") => "driver-not-reported-by-wgpu".to_string(),
        (name, "") => name.to_string(),
        ("", version) => version.to_string(),
        (name, version) => format!("{name} {version}"),
    }
}

/// 在找到的这张卡上起设备、编单元校验和的管线；起不来交为什么（显存不够与别的分开，[`CardStartFailure`]）。不看空闲显存（看不看由调用方定）。
/// 设备上装好接 wgpu 没接住的错误与设备丢失的回调（[`DeviceConditionOfACard`]）；不归卡池的设备起之前记下它在哪张卡上、起好的那一记放进卡里
/// （[`started_with_its_owner_recorded`]、[`has_devices_started_outside_the_card_pool_on_the_bus`]）；起完打一行 [`DEVICE_START_LIFECYCLE_PHASE`]。
pub(super) fn start_the_card(
    discovered: &DiscoveredCard,
    owner: DeviceOwnerOnTheCard,
) -> Result<GpuCard, CardStartFailure> {
    let (mut card, marked_outside_the_books) =
        started_with_its_owner_recorded(owner, &discovered.pci_bus_identifier, || {
            device_started_with_a_lifecycle_line(
                discovered.reported.index,
                &discovered.pci_bus_identifier,
                owner,
                || start_the_device_and_the_pipeline(discovered),
                &gpu_work_trace,
            )
        })?;
    card.device_outside_the_books = marked_outside_the_books.map(std::sync::Arc::new);
    Ok(card)
}

/// [`start_the_card`] 起设备、编管线的那一段（不管设备归谁）。
fn start_the_device_and_the_pipeline(
    discovered: &DiscoveredCard,
) -> Result<GpuCard, CardStartFailure> {
    let which_card = format!(
        "序号 {} 的卡（{}）",
        discovered.reported.index, discovered.name
    );
    let adapter = &discovered.adapter;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("singlefs-gpu-unit-checks"),
        required_features: wgpu::Features::empty(),
        // 崩溃放量的判器把整条流的版本字节与每个状态的草稿区各绑成一张存储缓冲，默认的 128 MiB 绑定上限装不下大流：
        // 按这张卡实际支持的上限要（NVIDIA Vulkan 上是 2 GiB − 1）。
        required_limits: adapter.limits(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .map_err(|error| failure_of_a_device_request(&error, &which_card))?;
    let device_condition =
        std::sync::Arc::new(DeviceConditionOfACard::on_the_card(discovered.name.clone()));
    let condition_for_uncaptured_errors = std::sync::Arc::clone(&device_condition);
    device.on_uncaptured_error(std::sync::Arc::new(move |error: wgpu::Error| {
        handle_an_uncaptured_wgpu_error(&condition_for_uncaptured_errors, error);
    }));
    let condition_for_the_loss = std::sync::Arc::clone(&device_condition);
    device.set_device_lost_callback(move |reason, message| {
        record_the_loss_of_the_device(&condition_for_the_loss, reason, &message);
    });
    // 编单元校验和的管线：这条线程上推一个 Out of Memory 范围，卡被别的进程占满时显存不够报成起不来，不让默认处理 panic
    let out_of_memory_scope = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("crc32c"),
        source: wgpu::ShaderSource::Wgsl(CRC32_CASTAGNOLI_SHADER_SOURCE.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("crc32c"),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    if let Some(error) = pollster::block_on(out_of_memory_scope.pop()) {
        let reason = format!("{which_card}上编单元校验和的管线时 wgpu 报了 {error}");
        device_condition.record_an_out_of_memory(reason.clone());
        return Err(CardStartFailure::OutOfGpuMemory { reason });
    }
    Ok(GpuCard {
        name: discovered.name.clone(),
        driver_version: discovered.driver_version.clone(),
        pci_bus_identifier: discovered.pci_bus_identifier.clone(),
        index: discovered.reported.index,
        free_memory_mebibytes: discovered.reported.free_memory_mebibytes,
        usable_memory_of_the_empty_card_mebibytes: None,
        memory_quota_mebibytes: None,
        device,
        queue,
        pipeline,
        compiled_pipelines_by_source_digest: std::sync::Arc::default(),
        device_condition,
        device_on_the_books: None,
        device_outside_the_books: None,
    })
}

/// 起好的卡记上判它放得下的那一次读数（[`card_memory_once_a_judge_fits`]）：空闲与空卡上能用的（额度照它们算，
/// [`GpuCard::memory_budget_mebibytes`]、[`GpuCard::memory_budget_when_the_card_is_empty_mebibytes`]）。
fn card_with_the_memory_reading_it_fit_by(
    mut card: GpuCard,
    reading: CardMemoryReading,
) -> GpuCard {
    card.free_memory_mebibytes = reading.free_mebibytes;
    card.usable_memory_of_the_empty_card_mebibytes = Some(reading.total_less_reserved_mebibytes);
    card
}

/// 配置点名的卡，按配置的次序（优先级）：`(nvidia-smi 序号, 显存额度 MiB)`。只在点名的卡上起设备（没点名的卡一个设备都不起）；
/// 起之前照 [`card_memory_once_a_judge_fits`] 判（照此刻空闲算的额度不低于 [`MINIMUM_MEMORY_BUDGET_MEBIBYTES`]，被这个进程自己占着的等它还）。
/// 点名而用不了的卡（本机没有这个序号、额度不够而不是被这个进程自己占着、等这个进程还等到头、上下文起不来）打一行说明，不悄悄换成别的卡。
/// 起的设备不进卡池的账（崩溃放量没有卡池的进程走这里；这个进程起过卡池的，崩溃放量照卡池里的卡用）。
#[must_use]
pub fn gpu_cards_by_priority(configured: &[(u32, u64)]) -> Vec<GpuCard> {
    let discovered_cards = nvidia_discrete_cards_reported_by_nvidia_smi();
    // 点名的卡找不到时下面那一行 GPU_CARD_NOT_USABLE 分不出是哪一步没过，先把找到的每张卡（序号、空闲显存）与 nvidia-smi 报的行数记下来
    let discovered_summary: Vec<(u32, u64)> = discovered_cards
        .iter()
        .map(|card| (card.reported.index, card.reported.free_memory_mebibytes))
        .collect();
    let nvidia_smi_rows = reported_cards_by_pci_bus().len();
    let mut usable: Vec<GpuCard> = discovered_cards
        .iter()
        .filter_map(|discovered| {
            let (_, quota) = configured
                .iter()
                .find(|(index, _)| *index == discovered.reported.index)?;
            card_memory_once_a_judge_fits(discovered, Some(*quota))
                .map_err(|reason| {
                    // 判成用不了的原因打出来，不让下面那一行 GPU_CARD_NOT_USABLE 把它吞成「用不了」三个字
                    eprintln!(
                        "GPU_CARD_SHORT_OF_MEMORY index={}：{reason}",
                        discovered.reported.index
                    );
                })
                .ok()
                .map(|reading| (discovered, reading))
        })
        .filter_map(|(discovered, reading)| {
            start_the_card(discovered, DeviceOwnerOnTheCard::OutsideTheCardPool)
                .map_err(|failure| {
                    // 起不来的原因打出来，不让下面那一行 GPU_CARD_NOT_USABLE 把它吞成「用不了」三个字
                    let reason = match &failure {
                        CardStartFailure::OutOfGpuMemory { reason }
                        | CardStartFailure::DeviceNotStarted { reason } => reason,
                    };
                    eprintln!(
                        "GPU_CARD_START_FAILED index={}：{reason}",
                        discovered.reported.index
                    );
                })
                .ok()
                .map(|card| card_with_the_memory_reading_it_fit_by(card, reading))
        })
        .collect();
    let mut cards = Vec::with_capacity(configured.len());
    for (index, quota) in configured {
        match usable.iter().position(|card| card.index == *index) {
            Some(position) => {
                let mut card = usable.remove(position);
                card.memory_quota_mebibytes = Some(*quota);
                cards.push(card);
            }
            None => eprintln!(
                "GPU_CARD_NOT_USABLE index={index}：配置点名了这张卡，本机却用不了它（没有这个序号、给崩溃放量的额度低于 {MINIMUM_MEMORY_BUDGET_MEBIBYTES} MiB 而不是被这个进程自己占着、\
                 等这个进程还等到头、或上下文起不来：上面 GPU_CARD_SHORT_OF_MEMORY、GPU_CARD_START_FAILED 那几行）；\
                 这一次 Vulkan 下与 nvidia-smi 对上的 NVIDIA 独显（序号, 空闲 MiB）：{discovered_summary:?}，nvidia-smi 报了 {nvidia_smi_rows} 张卡"
            ),
        }
    }
    cards
}

/// 驱动把编好的 Vulkan 管线存到盘上的目录：跨轮复用的缓存放 `GATE_CROSS_RUN_TMPDIR` 下；没设就放家目录 `~/.cache/singlefs`
/// （`/tmp` 开机会清空，冷编要十几分钟的内核每次重启都得重编）；连 `HOME` 都没有才退到 `TMPDIR`、`/tmp`
/// （`command-safety.md`「测试镜像一律放临时目录」：有意跨轮复用的缓存不算镜像）。
#[must_use]
pub fn driver_shader_disk_cache_directory() -> std::path::PathBuf {
    let base = std::env::var_os("GATE_CROSS_RUN_TMPDIR")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| std::path::PathBuf::from(home).join(".cache/singlefs"))
        })
        .or_else(|| std::env::var_os("TMPDIR").map(std::path::PathBuf::from))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    base.join("singlefs-gpu-shader-cache")
}

/// NVIDIA 驱动的盘上着色器缓存默认不罩 Vulkan 管线（崩溃放量的判器内核冷编要几分钟，每个进程都重编）；
/// 起 Vulkan 实例之前把它开起来、指到 [`driver_shader_disk_cache_directory`]，同一版内核第二个进程起就直接装（实测 57 秒变 0.5 秒）。
/// 用户已经设了这几个变量的照用户的；`SKIP_CLEANUP` 让驱动不按默认的容量上限清掉大内核的那一份。
fn enable_the_driver_shader_disk_cache() {
    if std::env::var_os("__GL_SHADER_DISK_CACHE").is_none() {
        std::env::set_var("__GL_SHADER_DISK_CACHE", "1");
    }
    if std::env::var_os("__GL_SHADER_DISK_CACHE_PATH").is_none() {
        let directory = driver_shader_disk_cache_directory();
        let _ = std::fs::create_dir_all(&directory);
        std::env::set_var("__GL_SHADER_DISK_CACHE_PATH", &directory);
    }
    if std::env::var_os("__GL_SHADER_DISK_CACHE_SKIP_CLEANUP").is_none() {
        std::env::set_var("__GL_SHADER_DISK_CACHE_SKIP_CLEANUP", "1");
    }
    // 驱动按可执行文件分缓存目录；崩溃放量七个分项各是一个测试二进制，同一版内核每个都要冷编一遍（release 十几分钟）。
    // 给一个共用的应用名，认得这个变量的驱动把它们放进同一份缓存，只编一次；不认得的照旧按可执行文件分。
    if std::env::var_os("__GL_SHADER_DISK_CACHE_APP_NAME").is_none() {
        std::env::set_var("__GL_SHADER_DISK_CACHE_APP_NAME", "singlefs-crash-judge");
    }
}

#[cfg(test)]
mod tests;
