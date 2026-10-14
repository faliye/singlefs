//! 崩溃放量的 GPU 核对（里程碑二「增补 4」第四项；用户 2026-09-28 定：数据从 KV 按批读出来交给 GPU 核对，写回，门禁一次读违例）。
//! 这一版 GPU 接单元级那一截：一批单元的头校验和与载荷 CRC 在 GPU 上算（CRC-32C，参数与 D18（块里携带什么信息） 已定项 17 相同），
//! CPU 参照（`check_unit_checksums_on_the_cpu`）是另一趟、由对拍用例逐个比，两边的输入都经库读回
//! （`gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library`，用户 2026-10-13 定）；走树、恢复、记录核对器照旧在 CPU 上。
//! 着色器与多卡枚举取自 E163（GPU多卡算单元校验和） 的装置（那边两卡 2560/2560 与 CPU 逐个相同）。
//! 分工按显卡空闲显存（`nvidia-smi` 的 memory.free，按 PCI 总线号对上 wgpu 的适配器），空闲显存低于门槛的卡不用；
//! GPU 用例共用的卡池（[`gpu_test_start::GpuCardPool`]）不看这个门槛，能不能借由这个进程的显存账在借的那一刻判
//! （`singlefs_harness::gpu_memory_ledger` 文件头：只记这个进程借的，借不到不等）。
//! 每个设备起的时候装上 wgpu 没接住的错误与设备丢失的回调（[`DeviceConditionOfACard`]）：别的线程上报的 Out of Memory
//! （判器内核在另起的线程上编）记在设备上，接 Out of Memory 的那一层照它认出是显存不够（第 10x 批代码轮判决 K1-3），
//! 按记的那条线程上正在做的那段活认（第 10z 批：别的线程上一次 Out of Memory 不把这段活真正的 panic 说成显存不够；
//! 编判器内核另起的线程算在那一刻在这个设备上编内核的那段活上，[`GpuCard::pipelines_compiled_once_on_this_device`]）。

use singlefs_checker::{
    crc32_castagnoli_table, unit_checksum_layout, UNIT_HEADER_CHECKSUM_BYTE_OFFSET,
};
pub use singlefs_harness::gpu_memory_ledger::DeviceConditionOfACard;
use singlefs_harness::gpu_memory_ledger::{
    card_memory_reader_of_one_report_per_reading, pci_bus_identifier_without_the_domain,
    BusesWithDevicesStartedOutsideTheBooks, CardMemoryReader, CauseOfTheDeviceLoss,
    DeviceStartedOnTheBooks, DeviceWaitBeforeReturningTheLease,
    NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
};
use singlefs_harness::sha256::DIGEST_BYTES;

/// 卡池的借据丢掉时最多等设备多久（E174 跑前登记 `research/prompts/e174-preregistration.md` 第五节「等待」那一臂：
/// `PollType::Wait { submission_index: None, timeout: Some(30 秒) }`；E174 两段里这一臂每次都等完了，`wait_always_returns=true`）。
const WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE: std::time::Duration =
    std::time::Duration::from_secs(30);

/// 借据丢掉时那一次等（`polled` 是 `poll(Wait)` 的结局，wgpu panic 了是外层的 `Err`）算没算等完：队列空了或等到了才算；
/// 超时、wgpu 报错、panic 都算没等成。`Wait` 不交 [`wgpu::PollStatus::Poll`]（那是 `PollType::Poll` 的结局），交了照没等成算。
fn device_wait_before_returning_a_lease(
    polled: &std::thread::Result<Result<wgpu::PollStatus, wgpu::PollError>>,
) -> DeviceWaitBeforeReturningTheLease {
    match polled {
        Ok(Ok(wgpu::PollStatus::QueueEmpty | wgpu::PollStatus::WaitSucceeded)) => {
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished
        }
        Ok(Ok(wgpu::PollStatus::Poll) | Err(_)) | Err(_) => {
            DeviceWaitBeforeReturningTheLease::TheWaitFailed
        }
    }
}

/// 卡池的借据丢掉时那一次等：等这张卡的设备上到此刻为止提交的活都做完（`submission_index: None`，E174 跑前登记第五节「等待」那一臂），
/// 最多等 [`WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE`]（不设上限会让丢借据的那条线程卡在一台出了事的设备上，设 0 等于不等）。
fn poll_type_before_returning_a_lease() -> wgpu::PollType {
    wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE),
    }
}

/// 卡池借之前对一台设备 poll 一次（`PollType::Poll`，不等，wgpu panic 了是外层的 `Err`）交回的算不算那台设备上提交的活都做完了：
/// 交队列空了才算（wgpu-core 30 `Device::maintain` 先收完做完的提交、丢掉的缓冲交回驱动，再看队列空没空）；还有在飞的、报错、panic 都不算。
fn device_found_idle_by_a_poll(
    polled: &std::thread::Result<Result<wgpu::PollStatus, wgpu::PollError>>,
) -> bool {
    match polled {
        Ok(Ok(wgpu::PollStatus::QueueEmpty)) => true,
        Ok(Ok(wgpu::PollStatus::WaitSucceeded | wgpu::PollStatus::Poll) | Err(_)) | Err(_) => false,
    }
}

/// 空闲显存低于这个数（MiB）的卡不用：本机有卡被别的服务占着时只剩几百 MiB，起上下文都难（E163 M2 那张卡 141 MiB 起不来）。
/// 只管 [`usable_gpu_cards`] 与 [`gpu_cards_by_priority`]；卡池按账判，此刻被别的进程占着的卡照样进池，借的那一刻借不到就报本次未跑（不等）。
pub const MINIMUM_FREE_MEMORY_MEBIBYTES: u64 = 2048;
/// 一个 GPU 槽装一段字节：最大 32 KiB（数据单元的长度），短的按长度截断。
const SLOT_BYTES: usize = 32_768;
const WORKGROUP_SIZE: usize = 64;
const RESULT_ENTRY_BYTES: usize = 8;
/// NVIDIA 的厂商号（Vulkan 适配器与 PCI 设备报的都是它）：崩溃放量只用 NVIDIA 独显，[`usable_gpu_cards`] 按它筛卡，
/// 测试取卡的判定（[`gpu_test_start`]）按它认 Vulkan 适配器与 PCI 上的显示控制器。
pub const NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE;

const CRC32C_SHADER_SOURCE: &str = r#"
struct Params { count: u32, reserved0: u32, reserved1: u32, reserved2: u32, };
struct ResultEntry { crc: u32, local_offset: u32, };
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> slot_words: array<u32>;
@group(0) @binding(2) var<storage, read> slot_lengths: array<u32>;
@group(0) @binding(3) var<storage, read_write> results: array<ResultEntry>;
const CRC32C_REFLECTED_POLYNOMIAL: u32 = 0x82F63B78u;
const CRC32C_INITIAL_VALUE: u32 = 0xFFFFFFFFu;
const WORDS_PER_SLOT: u32 = 8192u;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let local_offset = global_id.x;
    if (local_offset >= params.count) { return; }
    let length_in_bytes = slot_lengths[local_offset];
    let base_word = local_offset * WORDS_PER_SLOT;
    var crc_register: u32 = CRC32C_INITIAL_VALUE;
    var byte_index: u32 = 0u;
    loop {
        if (byte_index >= length_in_bytes) { break; }
        let word = slot_words[base_word + byte_index / 4u];
        let current_byte = (word >> (8u * (byte_index % 4u))) & 0xFFu;
        crc_register = crc_register ^ current_byte;
        var bit_index: u32 = 0u;
        loop {
            if (bit_index >= 8u) { break; }
            if ((crc_register & 1u) == 1u) {
                crc_register = (crc_register >> 1u) ^ CRC32C_REFLECTED_POLYNOMIAL;
            } else {
                crc_register = crc_register >> 1u;
            }
            bit_index = bit_index + 1u;
        }
        byte_index = byte_index + 1u;
    }
    results[local_offset].crc = crc_register ^ CRC32C_INITIAL_VALUE;
    results[local_offset].local_offset = local_offset;
}
"#;

/// 没给额度的卡，崩溃放量最多用它空闲显存的几分之几（分子、分母）：剩下的留给卡上别的进程。
const SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA: (u64, u64) = (3, 4);

/// 一个设备上按源码摘要编好的几组管线：每个摘要一格，第一个要它的线程编、同时要它的等那一次编完（[`GpuCard::pipelines_compiled_once_on_this_device`]）。
type CompiledPipelinesBySourceDigest = std::sync::Mutex<
    std::collections::BTreeMap<
        [u8; DIGEST_BYTES],
        std::sync::Arc<std::sync::OnceLock<Vec<wgpu::ComputePipeline>>>,
    >,
>;

/// 一张起好上下文的卡。克隆出来的与原来的共用同一个设备、队列与编好的管线（wgpu 的这几样克隆的是句柄）：
/// 一个进程里几条 GPU 用例共用一张卡就靠克隆（[`gpu_test_start::GpuCardPool`]），每张卡只起一个设备。
#[derive(Clone)]
pub struct GpuCard {
    pub name: String,
    /// 驱动的名字与版本（wgpu adapter info 的 `driver` 与 `driver_info`，空格接起来）：崩溃放量 GPU 结论的块属性照它写，不进版本
    /// （第三轮判决 J8、M4 第 11 条：两台驱动不同不分版本，靠对照与轻型的小输入对拍抓差）。
    pub driver_version: String,
    pub pci_bus_identifier: String,
    /// `nvidia-smi` 给这张卡的序号（配置按它点名）。
    pub index: u32,
    /// 起上下文之前 `nvidia-smi` 报的空闲显存（MiB；卡池里的卡是起池时找卡那一刻报的，只当说明，借卡不看它）。
    /// 卡池借出去的卡改成借的量加驱动那一份：账判过卡上放得下这么多，不让找卡那一刻的读数把草稿区压得比借的小。
    pub free_memory_mebibytes: u64,
    /// 配置给这张卡的显存额度（MiB）：崩溃放量在这张卡上占的显存不超过它；没给额度的是 `None`。
    /// 卡池借出去的卡改成借的量加驱动那一份。
    pub memory_quota_mebibytes: Option<u64>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    /// 这个设备上编好的管线，几个克隆共用：同一套源码在同一个设备上只编一次。
    compiled_pipelines_by_source_digest: std::sync::Arc<CompiledPipelinesBySourceDigest>,
    /// 这个设备上 wgpu 报过的 Out of Memory 与设备丢没丢，几个克隆共用（[`start_the_card`] 装的回调往里记）。
    device_condition: std::sync::Arc<DeviceConditionOfACard>,
    /// 卡池起的设备在账上的那一台（[`gpu_test_start::GpuCardPool`] 起好时放进来，不走卡池的是 `None`），几个克隆共用：最后一个克隆丢掉时
    /// 这一台放掉（`singlefs_harness::gpu_memory_ledger::DeviceStartedOnTheBooks`）。放在最后一个字段：字段照声明的次序丢，
    /// 这一份最后丢的时候，同一个克隆里的设备、队列与编好的管线已经先丢了（第 10z 批第八块）。
    device_on_the_books: Option<std::sync::Arc<DeviceStartedOnTheBooks<'static>>>,
}

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
/// 照它的这一句丢设备）相同的是显存不够（第 10z 批第五块，第 10z 批第三块代码轮判决 K1-4 攻方甲形）。wgpu 不另给丢失的缘由
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
    /// 卡池的调用方报本次未跑（第 10x 批代码轮判决 K1-3）。
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

/// 这一个设备归谁：卡池的设备走这个进程的显存账；别的（[`gpu_cards_by_priority`]、[`usable_gpu_cards`] 起的）不走，
/// 起之前就记下它在哪张卡上（[`has_devices_started_outside_the_card_pool_on_the_bus`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeviceOwnerOnTheCard {
    TheCardPool,
    OutsideTheCardPool,
}

/// 这个进程在哪几张卡（PCI 总线号）上起过不走卡池账的设备（起之前就记，[`started_with_its_owner_recorded`]）。是这个进程的事实，
/// 所以是进程里一份（第 10x 批代码轮判决 K1-1，攻方乙形）。
static BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL: BusesWithDevicesStartedOutsideTheBooks =
    BusesWithDevicesStartedOutsideTheBooks::new();

/// 这个进程在这张卡（PCI 总线号，[`pci_bus_identifier_without_the_domain`] 写法）上起过（或正在起）不走卡池账的设备没有：起过，卡池读这张卡时
/// 这个进程占着的那一份分不出哪些是借据建的（`singlefs_harness::gpu_memory_ledger::OccupiedByThisProcess::AlsoOutsideTheBooks`）。
fn has_devices_started_outside_the_card_pool_on_the_bus(bus: &str) -> bool {
    BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL.has_devices_on_the_bus(bus)
}

/// 照设备归谁起（`start` 起设备）：卡池的直接起；不归卡池的先在总线 `bus` 上记下、再起（第 10z 批，第 10y 批代码轮判决 K1-1 攻方乙形：
/// 起好了才记，起设备到记下那一段里卡池读到的这个进程占着的已经有它，被当成借据建上的）。
///
/// # Errors
/// `start` 的错。
fn started_with_its_owner_recorded<Device>(
    owner: DeviceOwnerOnTheCard,
    bus: &str,
    start: impl FnOnce() -> Result<Device, CardStartFailure>,
) -> Result<Device, CardStartFailure> {
    match owner {
        DeviceOwnerOnTheCard::TheCardPool => start(),
        DeviceOwnerOnTheCard::OutsideTheCardPool => {
            BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL.start_a_device_marked_first(bus, start)
        }
    }
}

/// 卡池读 `buses` 这几张卡显存的那一份：每读一次取一份 `run_the_report` 的报告（卡池是跑一次 `nvidia-smi`），按这个进程的进程号认它占着的，
/// 这个进程在哪张卡上起过不走卡池账的设备照 [`has_devices_started_outside_the_card_pool_on_the_bus`] 认。
fn card_memory_reader_of_the_card_pool(
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

/// `nvidia-smi` 报的一张卡：序号与空闲显存（MiB）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReportedCard {
    index: u32,
    free_memory_mebibytes: u64,
}

/// `nvidia-smi` 报的每张卡：PCI 总线号（小写、去掉前导的 0 域）→ 序号与空闲显存。起不来 `nvidia-smi` 就是空表。
fn reported_cards_by_pci_bus() -> std::collections::BTreeMap<String, ReportedCard> {
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

/// 卡池读显存用的那一份报告：跑一次 `nvidia-smi`（`singlefs_harness::gpu_memory_ledger::NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS`），
/// 每张卡的显存与每个进程占的出自这一次。
///
/// # Errors
/// `nvidia-smi` 起不来或退了非 0。
fn nvidia_smi_memory_and_processes_report() -> Result<String, String> {
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

/// Vulkan 下的一张 NVIDIA 独显与 `nvidia-smi` 报的它（序号、那一刻的空闲显存）：还没起设备。
/// 卡池先找到点名的卡、在账上登记了这个进程，才在卡上起设备（[`gpu_test_start::GpuCardPool`]）；别的取卡入口找到就起。
#[derive(Clone)]
struct DiscoveredCard {
    adapter: wgpu::Adapter,
    name: String,
    driver_version: String,
    pci_bus_identifier: String,
    reported: ReportedCard,
}

/// 这个进程的 Vulkan 实例，整个进程只建一份、第一次用时建。
///
/// 为什么只建一份：每建一个实例，Vulkan 加载器都把本机装着的每个 ICD（intel、radeon、lvp 带整份 LLVM、nouveau……）dlopen 一遍，
/// 它们的线程局部存储吃掉 glibc 给动态库预留的静态 TLS 余量，dlclose 不一定还回来；几十次之后再 dlopen NVIDIA 的
/// `libnvidia-tls.so` 就报「cannot allocate memory in static TLS block」，加载器只剩 llvmpipe，配置点名的卡「用不了」
/// （2026-10-14 库单测进程里跑到 `crash_injection::tests::` 那一段才起第一个实例就撞上，单跑同一条用例绿）。
/// 一份实例从头用到尾，ICD 只装一次。
fn vulkan_instance_of_this_process() -> &'static wgpu::Instance {
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
fn nvidia_discrete_cards_reported_by_nvidia_smi() -> Vec<DiscoveredCard> {
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
pub fn driver_version_of_the_adapter(driver: &str, driver_info: &str) -> String {
    match (driver.trim(), driver_info.trim()) {
        ("", "") => "driver-not-reported-by-wgpu".to_string(),
        (name, "") => name.to_string(),
        ("", version) => version.to_string(),
        (name, version) => format!("{name} {version}"),
    }
}

/// 在找到的这张卡上起设备、编单元校验和的管线；起不来交为什么（显存不够与别的分开，[`CardStartFailure`]）。不看空闲显存（看不看由调用方定）。
/// 设备上装好接 wgpu 没接住的错误与设备丢失的回调（[`DeviceConditionOfACard`]）；不归卡池的设备起之前记下它在哪张卡上
/// （[`started_with_its_owner_recorded`]、[`has_devices_started_outside_the_card_pool_on_the_bus`]）。
fn start_the_card(
    discovered: &DiscoveredCard,
    owner: DeviceOwnerOnTheCard,
) -> Result<GpuCard, CardStartFailure> {
    started_with_its_owner_recorded(owner, &discovered.pci_bus_identifier, || {
        start_the_device_and_the_pipeline(discovered)
    })
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
    let device_condition = std::sync::Arc::new(DeviceConditionOfACard::default());
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
        source: wgpu::ShaderSource::Wgsl(CRC32C_SHADER_SOURCE.into()),
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
        memory_quota_mebibytes: None,
        device,
        queue,
        pipeline,
        compiled_pipelines_by_source_digest: std::sync::Arc::default(),
        device_condition,
        device_on_the_books: None,
    })
}

/// 配置点名的卡，按配置的次序（优先级）：`(nvidia-smi 序号, 显存额度 MiB)`。只在点名的卡上起设备（没点名的卡一个设备都不起）；
/// 点名而用不了的卡（本机没有这个序号、空闲显存低于 [`MINIMUM_FREE_MEMORY_MEBIBYTES`]、上下文起不来）打一行说明，不悄悄换成别的卡。
/// 起的设备不进卡池的账（崩溃放量没有卡池的进程走这里；这个进程起过卡池的，崩溃放量照卡池里的卡用）。
#[must_use]
pub fn gpu_cards_by_priority(configured: &[(u32, u64)]) -> Vec<GpuCard> {
    let named_indices: Vec<u32> = configured.iter().map(|(index, _)| *index).collect();
    let discovered_cards = nvidia_discrete_cards_reported_by_nvidia_smi();
    // 点名的卡找不到时下面那一行 GPU_CARD_NOT_USABLE 分不出是哪一步没过，先把找到的每张卡（序号、空闲显存）与 nvidia-smi 报的行数记下来
    let discovered_summary: Vec<(u32, u64)> = discovered_cards
        .iter()
        .map(|card| (card.reported.index, card.reported.free_memory_mebibytes))
        .collect();
    let nvidia_smi_rows = reported_cards_by_pci_bus().len();
    let mut usable: Vec<GpuCard> = discovered_cards
        .iter()
        .filter(|discovered| named_indices.contains(&discovered.reported.index))
        .filter(|discovered| {
            discovered.reported.free_memory_mebibytes >= MINIMUM_FREE_MEMORY_MEBIBYTES
        })
        .filter_map(|discovered| {
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
                "GPU_CARD_NOT_USABLE index={index}：配置点名了这张卡，本机却用不了它（没有这个序号、空闲显存低于 {MINIMUM_FREE_MEMORY_MEBIBYTES} MiB、或上下文起不来）；\
                 这一次 Vulkan 下与 nvidia-smi 对上的 NVIDIA 独显（序号, 空闲 MiB）：{discovered_summary:?}，nvidia-smi 报了 {nvidia_smi_rows} 张卡"
            ),
        }
    }
    cards
}

/// 本机能用的卡：Vulkan 下的 NVIDIA 独显，空闲显存不低于 [`MINIMUM_FREE_MEMORY_MEBIBYTES`]，上下文起得来；按空闲显存从大到小排。
/// 起的设备不进卡池的账。
#[must_use]
pub fn usable_gpu_cards() -> Vec<GpuCard> {
    let mut cards: Vec<GpuCard> = nvidia_discrete_cards_reported_by_nvidia_smi()
        .iter()
        .filter(|discovered| {
            discovered.reported.free_memory_mebibytes >= MINIMUM_FREE_MEMORY_MEBIBYTES
        })
        .filter_map(|discovered| {
            start_the_card(discovered, DeviceOwnerOnTheCard::OutsideTheCardPool).ok()
        })
        .collect();
    cards.sort_by_key(|card| std::cmp::Reverse(card.free_memory_mebibytes));
    cards
}

/// 一张卡给崩溃放量的额度（MiB）：给了额度（`memory_quota_mebibytes`）的取额度与空闲显存里小的那个，
/// 没给的取空闲显存的 [`SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA`]（[`GpuCard::memory_budget_mebibytes`] 照它算；卡池照它、
/// 拿这个进程此刻能在卡上占的算每张卡的账，`singlefs_harness::gpu_memory_ledger` 文件头「口径」）。
#[must_use]
pub fn memory_budget_mebibytes_of(
    memory_quota_mebibytes: Option<u64>,
    free_memory_mebibytes: u64,
) -> u64 {
    match memory_quota_mebibytes {
        Some(quota) => quota.min(free_memory_mebibytes),
        None => {
            let (numerator, denominator) = SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA;
            free_memory_mebibytes / denominator * numerator
        }
    }
}

/// 在卡上算 `range_count` 段字节的 CRC-32C（[`GpuCard::crc32c_of_ranges`]）一次要多少显存（MiB，向上取整）：每段一个 32 KiB 的槽、
/// 一个长度字、一条结果与它的回读，加一份派活参数。核一批单元的两道校验和（[`check_unit_checksums_on_gpu`]）每个单元两段。
/// 用例向卡池借卡时按它借（`gpu_test_start::GpuCardPool::lease_card_or_report_not_run`）。
#[must_use]
pub fn mebibytes_crc32c_of_ranges_needs(range_count: usize) -> u64 {
    bytes_crc32c_of_ranges_needs(range_count).div_ceil(1024 * 1024)
}

/// 同 [`mebibytes_crc32c_of_ranges_needs`]，按字节、不取整：[`GpuCard::crc32c_of_ranges`] 一次建的五个缓冲
/// （派活参数 16 字节、每段一个 32 KiB 的槽、一个长度字、一条结果与它的回读各 8 字节）。崩溃放量第 ① 段由 CPU 判、
/// 单元校验和交卡对拍时按它算几条线程同时派活占多少（`crate::crash_amplification` 文件头第 5 条）。
#[must_use]
pub fn bytes_crc32c_of_ranges_needs(range_count: usize) -> u64 {
    const PARAMETER_BYTES: usize = 16;
    let bytes = range_count * (SLOT_BYTES + 4 + 2 * RESULT_ENTRY_BYTES) + PARAMETER_BYTES;
    u64::try_from(bytes).expect("装得进 u64")
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

impl GpuCard {
    /// 崩溃放量在这张卡上最多占多少显存（MiB）：给了额度的取额度与空闲显存里小的那个，
    /// 没给额度的取空闲显存的 [`SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA`]。
    #[must_use]
    pub fn memory_budget_mebibytes(&self) -> u64 {
        memory_budget_mebibytes_of(self.memory_quota_mebibytes, self.free_memory_mebibytes)
    }

    /// 这张卡的设备（崩溃放量第 ② 段的核对内核在同一张卡上另起自己的管线）。
    #[must_use]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// 这张卡的队列。
    #[must_use]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// 这个设备上 wgpu 报过的 Out of Memory 与设备丢没丢（几个克隆共用一份）。
    #[must_use]
    pub fn device_condition(&self) -> &DeviceConditionOfACard {
        &self.device_condition
    }

    /// 这个设备上按 `source_digest` 编好的那组管线：这个设备（这张卡与它的每个克隆）上第一次要这个摘要时调 `compile` 编，
    /// 别的线程同时要同一个摘要就等这一次编完、拿同一组；以后再要直接拿。管线绑在设备上，另起的设备（另一次 [`usable_gpu_cards`]）另有一份。
    /// 冷编一套判器内核要几分钟、驱动编的时候吃几 GiB 主机内存：几十条用例共用一张卡时，每条各编一遍既慢又撞主机内存上限。
    /// 编的那一段在设备上记着是为这条线程上的哪段活编的（[`DeviceConditionOfACard::compile_for_the_work_of_this_thread`]）：
    /// `compile` 另起的线程上没接住的 Out of Memory 算那段活的。
    ///
    /// # Panics
    /// `compile` panic：原样传出，这一格照旧空着，下一个要这个摘要的再编一次。
    #[must_use]
    pub fn pipelines_compiled_once_on_this_device(
        &self,
        source_digest: [u8; DIGEST_BYTES],
        compile: impl FnOnce() -> Vec<wgpu::ComputePipeline>,
    ) -> Vec<wgpu::ComputePipeline> {
        // 持锁只为取出这个摘要那一格（不在锁里编），没有会 panic 到一半的写：别的线程 panic 留下的毒不影响这张表，照用
        let compiled_once = {
            let mut by_source_digest = self
                .compiled_pipelines_by_source_digest
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::sync::Arc::clone(by_source_digest.entry(source_digest).or_default())
        };
        compiled_once
            .get_or_init(|| {
                self.device_condition
                    .compile_for_the_work_of_this_thread(compile)
            })
            .clone()
    }

    /// 一批字节段（每段不超过 32 KiB）在这张卡上算 CRC-32C，按次序交回。
    ///
    /// # Panics
    /// 有一段超过 32 KiB；回读的结果序号对不上（着色器与派发错了，不许带着错结果往下走）。
    #[must_use]
    pub fn crc32c_of_ranges(&self, ranges: &[&[u8]]) -> Vec<u32> {
        if ranges.is_empty() {
            return Vec::new();
        }
        let count = ranges.len();
        let mut slots = vec![0u8; count * SLOT_BYTES];
        let mut lengths = Vec::with_capacity(count);
        for (index, range) in ranges.iter().enumerate() {
            assert!(
                range.len() <= SLOT_BYTES,
                "一段 {} 字节，超过一个槽的 {SLOT_BYTES}",
                range.len()
            );
            slots[index * SLOT_BYTES..index * SLOT_BYTES + range.len()].copy_from_slice(range);
            lengths.push(u32::try_from(range.len()).expect("不超过 32 KiB"));
        }
        let words_to_bytes = |values: &[u32]| {
            values
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect::<Vec<u8>>()
        };
        let count_u32 = u32::try_from(count).expect("一批的段数装得进 u32");
        let device = &self.device;
        let parameters = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &words_to_bytes(&[count_u32, 0, 0, 0]),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let input = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &slots,
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let length_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &words_to_bytes(&lengths),
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let result_bytes = u64::try_from(count * RESULT_ENTRY_BYTES).expect("装得进 u64");
        let results = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &vec![0xFFu8; count * RESULT_ENTRY_BYTES],
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            },
        );
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: result_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: parameters.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: length_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: results.as_entire_binding(),
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
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                u32::try_from(count.div_ceil(WORKGROUP_SIZE)).expect("工作组数装得进 u32"),
                1,
                1,
            );
        }
        encoder.copy_buffer_to_buffer(&results, 0, &staging, 0, result_bytes);
        let submission = self.queue.submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).expect("回调送得出去")
        });
        // 只等这一次提交：几条用例共用这张卡的设备时，不等别的用例提交的活
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: None,
            })
            .expect("等 GPU 算完");
        receiver.recv().expect("回调送达").expect("回读映射成功");
        let mapped = slice.get_mapped_range().expect("读得到回读缓冲区");
        let mut crcs = Vec::with_capacity(count);
        let (entries, remainder) = mapped.as_chunks::<RESULT_ENTRY_BYTES>();
        assert!(
            remainder.is_empty(),
            "回读缓冲区的长度是每条结果 8 字节的整数倍"
        );
        for (index, chunk) in entries.iter().enumerate() {
            let crc = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let local_offset = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
            assert_eq!(
                usize::try_from(local_offset).expect("装得进 usize"),
                index,
                "GPU 回来的第 {index} 条结果序号不对"
            );
            crcs.push(crc);
        }
        drop(mapped);
        staging.unmap();
        crcs
    }
}

/// 一个单元两道校验和的核对结果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitChecksumVerdict {
    BothHold,
    /// 类标签不认得、长度不对：不是这一截能判的，交回 CPU 那一整套去判。
    NotAUnitThisCheckReads,
    HeaderChecksumMismatch,
    PayloadCrcMismatch,
}

/// 一个单元按它头里的布局切成要算 CRC 的两段：头那一段（头校验和字段清零之后的副本）与载荷那一段，连同头里存着的两道校验和。
/// 类标签不认得、长度不对的交回 None（[`UnitChecksumVerdict::NotAUnitThisCheckReads`]）。
struct UnitChecksumRanges<'unit> {
    header_with_its_checksum_field_zeroed: Vec<u8>,
    payload: &'unit [u8],
    stored_header_checksum: u32,
    stored_payload_crc: u32,
}

fn unit_checksum_ranges(unit: &[u8]) -> Option<UnitChecksumRanges<'_>> {
    let layout = unit_checksum_layout(unit).filter(|layout| {
        u64::try_from(unit.len()).ok() == Some(layout.expected_length_in_bytes)
            && layout.header_end <= unit.len()
    })?;
    let mut header_with_its_checksum_field_zeroed = unit[..layout.header_end].to_vec();
    let field = UNIT_HEADER_CHECKSUM_BYTE_OFFSET..UNIT_HEADER_CHECKSUM_BYTE_OFFSET + 32;
    let stored_header_checksum = u32::from_le_bytes(
        unit[field.start..field.start + 4]
            .try_into()
            .expect("4 字节"),
    );
    header_with_its_checksum_field_zeroed[field].fill(0);
    let stored_payload_crc = u32::from_le_bytes(
        unit[layout.payload_crc_offset..layout.payload_crc_offset + 4]
            .try_into()
            .expect("4 字节"),
    );
    Some(UnitChecksumRanges {
        header_with_its_checksum_field_zeroed,
        payload: &unit[layout.header_end..],
        stored_header_checksum,
        stored_payload_crc,
    })
}

/// 两道算出来的 CRC 与头里存着的比，交回这个单元的核对结果。
fn unit_checksum_verdict(
    ranges: &UnitChecksumRanges<'_>,
    computed_header_checksum: u32,
    computed_payload_crc: u32,
) -> UnitChecksumVerdict {
    if computed_header_checksum != ranges.stored_header_checksum {
        UnitChecksumVerdict::HeaderChecksumMismatch
    } else if computed_payload_crc != ranges.stored_payload_crc {
        UnitChecksumVerdict::PayloadCrcMismatch
    } else {
        UnitChecksumVerdict::BothHold
    }
}

/// 一批单元在一张卡上核两道校验和（CRC 在卡上算）。CPU 参照另是一趟（[`check_unit_checksums_on_the_cpu`]）：
/// 两边各判各的、由对拍用例逐个比（用户 2026-10-11 定：一段里 CPU 不既跑数据又对拍），两边的输入都从库里读回（用户 2026-10-13 定）。
#[must_use]
pub fn check_unit_checksums_on_gpu(card: &GpuCard, units: &[&[u8]]) -> Vec<UnitChecksumVerdict> {
    let plan: Vec<Option<UnitChecksumRanges<'_>>> = units
        .iter()
        .map(|unit| unit_checksum_ranges(unit))
        .collect();
    let ranges: Vec<&[u8]> = plan
        .iter()
        .flatten()
        .flat_map(|unit_ranges| {
            [
                unit_ranges.header_with_its_checksum_field_zeroed.as_slice(),
                unit_ranges.payload,
            ]
        })
        .collect();
    let gpu = card.crc32c_of_ranges(&ranges);
    // 每个切得开的单元在卡上算两段 CRC（头那一段、载荷那一段），按切得开的单元的次序挨着回来
    let mut next_crc = 0usize;
    plan.iter()
        .map(|entry| match entry {
            None => UnitChecksumVerdict::NotAUnitThisCheckReads,
            Some(unit_ranges) => {
                let (header_crc, payload_crc) = (gpu[next_crc], gpu[next_crc + 1]);
                next_crc += 2;
                unit_checksum_verdict(unit_ranges, header_crc, payload_crc)
            }
        })
        .collect()
}

/// [`check_unit_checksums_on_gpu`] 的 CPU 参照：同一套切段、同一个判法，CRC 在 CPU 上算。
#[must_use]
pub fn check_unit_checksums_on_the_cpu(units: &[&[u8]]) -> Vec<UnitChecksumVerdict> {
    units
        .iter()
        .map(|unit| match unit_checksum_ranges(unit) {
            None => UnitChecksumVerdict::NotAUnitThisCheckReads,
            Some(unit_ranges) => unit_checksum_verdict(
                &unit_ranges,
                crc32_castagnoli_table(&unit_ranges.header_with_its_checksum_field_zeroed),
                crc32_castagnoli_table(unit_ranges.payload),
            ),
        })
        .collect()
}

/// GPU 对拍用例开跑前取卡，只给测试用（`tests/` 下的 GPU 对拍与库内要卡的计时用例都经它）。
/// 本机没有 NVIDIA 卡时打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`、用例当场返回成功（门禁按这一行逐条报「本次未跑」，
/// 上游 `test-script-discipline.md`「分快慢两档」：要特定硬件的那几条在没有那种硬件的机器上报「本次未跑」）；有卡而用不上判红
/// （同一篇「用上现有的硬件加速」：有那种硬件而用不上要报错，不悄悄退回）。
/// 「没卡」看两处：Vulkan 枚举里有没有 NVIDIA 独显，PCI 上有没有 NVIDIA 显示控制器——Vulkan 驱动坏了枚举不出来时，PCI 上照样看得见那张卡
/// （C620：GPU 对拍认「没卡」只看 Vulkan 枚举，驱动坏了被当成本次未跑）。看不见 PCI 总线（[`gpu_test_start::PCI_BUS_DIRECTORY`] 整个不在）
/// 也算没卡；总线看得见而设备表读不出照旧判红（代码三方 test-script-align-code-r2 判决 J2 G1）。判定是纯函数 [`gpu_test_start::decide_gpu_test_start`]，
/// 读 Vulkan 与 PCI 在取卡那一层（[`gpu_test_start::usable_cards_or_report_not_run`]）。
pub mod gpu_test_start {
    use super::{usable_gpu_cards, GpuCard, NVIDIA_VENDOR_IDENTIFIER};
    use crate::crash_amplification::RecordedGivenStatesTable;
    use crate::crash_library::{
        repository_root_of_this_crate, two_stages, CrashLibrary, CrashLibraryClass, ItemManifest,
        ItemName, StageOutcome,
    };
    use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

    /// CPU 参照经库的那一份在清单里叫什么。
    pub const CPU_REFERENCE_TABLE_IN_THE_MANIFEST: &str = "cpu_reference_given_states_table";

    /// CPU 参照那一份给定状态表在对拍那一类库里的项名：按这张表的摘要起（同一张表同一个名字，几条用例、几个线程录同一张表不撞名）。
    #[must_use]
    pub fn cpu_reference_item_name(digest: &[u8; DIGEST_BYTES]) -> String {
        let mut name = String::from("cpu_reference/");
        for byte in &digest[..16] {
            name.push_str(&format!("{byte:02x}"));
        }
        name
    }

    /// GPU 对拍用例里 CPU 参照的输入经对拍那一类的库（用户 2026-10-13 定：CPU 参照也从库读，两段之间不在内存里交数据）：
    /// 录入段把 CPU 参照要判的那几张镜像（一张给定状态表：基线、写表、持久集合）按摘要写进
    /// [`CrashLibraryClass::JudgeTablesPairing`] 那一类的库（家目录下长期存放）、按项名（[`cpu_reference_item_name`]）登清单；
    /// 判段凭项名读清单、按摘要读回、核摘要，CPU 参照只判读回的那一份。只录那一趟打一行 `CPU_REFERENCE_RECORDED_ONLY`、交 `None`
    /// （调用的用例当场返回）。镜像是用例手造的那一张时，写表是空的、持久集合是一个空集合，基线就是那张镜像。
    ///
    /// # Panics
    /// 库开不了、写不进、读不回；读回的字节解不出、摘要对不上。
    #[must_use]
    pub fn cpu_reference_table_read_back_from_the_pairing_library(
        table: &RecordedGivenStatesTable,
    ) -> Option<RecordedGivenStatesTable> {
        let bytes = table.to_bytes();
        let digest = sha256_digest(&bytes);
        let item_text = cpu_reference_item_name(&digest);
        let item =
            ItemName::new(&item_text).unwrap_or_else(|refusal| panic!("CPU 参照的项名：{refusal}"));
        let outcome = two_stages(
            CrashLibraryClass::JudgeTablesPairing,
            &repository_root_of_this_crate(),
            &item,
            |library| {
                library
                    .lock()
                    .store_given_states_table(&digest, &bytes)
                    .map_err(|error| format!("CPU 参照那张表写不进库：{error:?}"))?;
                Ok(ItemManifest {
                    kind: "cpu_reference_pairing".to_string(),
                    digests: vec![(CPU_REFERENCE_TABLE_IN_THE_MANIFEST.to_string(), digest)],
                })
            },
            |library, manifest| {
                Ok(StageOutcome::Judged(cpu_reference_table_of_the_manifest(
                    library, manifest,
                )?))
            },
        )
        .unwrap_or_else(|refusal| panic!("CPU 参照那张表（{item_text}）过库失败：{refusal}"));
        let read_back = outcome.judged();
        if read_back.is_none() {
            println!("CPU_REFERENCE_RECORDED_ONLY item={item_text}");
        }
        read_back
    }

    /// CPU 参照要判的「全部写都落了」那一张镜像经对拍那一类的库读回（[`cpu_reference_table_read_back_from_the_pairing_library`]：
    /// 基线与写表作一张一个状态、全落的给定状态表写进去），交回读回的那一份叠出来的镜像；只录那一趟交 `None`。
    ///
    /// # Panics
    /// 同 [`cpu_reference_table_read_back_from_the_pairing_library`]。
    #[must_use]
    pub fn image_with_every_write_landed_read_back_from_the_pairing_library(
        base: &singlefs_harness::memory_pool::MemoryPool,
        writes: &[singlefs_harness::memory_pool::RetainedWrite],
    ) -> Option<singlefs_harness::memory_pool::MemoryPool> {
        let read_back =
            cpu_reference_table_read_back_from_the_pairing_library(&RecordedGivenStatesTable {
                table_name: "every-write-landed".to_string(),
                base: base.clone(),
                writes: writes.to_vec(),
                segments: Vec::new(),
                seams: Vec::new(),
                persisted_sets: vec![vec![true; writes.len()]],
            })?;
        let mut image = read_back.base;
        image.apply_writes(&read_back.writes);
        Some(image)
    }

    /// 判段：按清单里 [`CPU_REFERENCE_TABLE_IN_THE_MANIFEST`] 那一样的摘要从库里读回那张表、核摘要、解码。
    ///
    /// # Errors
    /// 清单里没有那一样；库读失败；库里没有那个摘要；摘要对不上；解不出。
    pub fn cpu_reference_table_of_the_manifest(
        library: &CrashLibrary,
        manifest: &ItemManifest,
    ) -> Result<RecordedGivenStatesTable, String> {
        let digest = manifest.digest_named(CPU_REFERENCE_TABLE_IN_THE_MANIFEST)?;
        let bytes = library
            .lock()
            .read_given_states_table(&digest)
            .map_err(|error| format!("读 CPU 参照那张表失败：{error:?}"))?
            .ok_or_else(|| "CPU 参照那张表不在库里：先跑录入段".to_string())?;
        if sha256_digest(&bytes) != digest {
            return Err("CPU 参照那张表从库里读回的摘要与清单里的对不上".to_string());
        }
        RecordedGivenStatesTable::from_bytes(&bytes)
            .ok_or_else(|| "CPU 参照那张表从库里读回的字节解不出".to_string())
    }

    /// PCI 设备 class（24 位：基类、子类、编程接口各一字节）的基类是这个值的是显示控制器；同一块卡的 HDA 音频功能基类是 0x04，不算。
    pub const PCI_DISPLAY_CONTROLLER_BASE_CLASS: u32 = 0x03;
    /// sysfs 里 PCI 总线的目录：它整个不在就是这台机器看不见 PCI 总线（没有 PCI 的微虚机、沙箱，或没挂 sysfs）。
    pub const PCI_BUS_DIRECTORY: &str = "/sys/bus/pci";
    /// [`PCI_BUS_DIRECTORY`] 下列 PCI 设备的子目录：每个设备一个子目录，里面 `vendor` 与 `class` 两个文件各是一行 `0x` 起头的十六进制。
    pub const PCI_DEVICES_SUBDIRECTORY_NAME: &str = "devices";

    /// Vulkan 适配器的种类（与 Vulkan 报的物理设备类型一一对应）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum AdapterDeviceKind {
        DiscreteGpu,
        IntegratedGpu,
        VirtualGpu,
        SoftwareRenderingOnTheCpu,
        OtherOrUnknown,
    }

    /// Vulkan 枚举出来的一个适配器里，判定要看的两样。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EnumeratedAdapter {
        pub vendor_identifier: u32,
        pub device_kind: AdapterDeviceKind,
    }

    impl EnumeratedAdapter {
        /// 崩溃放量认的那一种卡：NVIDIA 独显（[`usable_gpu_cards`] 只从这一种里起上下文）。
        #[must_use]
        pub fn is_an_nvidia_discrete_gpu(&self) -> bool {
            self.vendor_identifier == NVIDIA_VENDOR_IDENTIFIER
                && self.device_kind == AdapterDeviceKind::DiscreteGpu
        }
    }

    /// PCI 上一个设备的厂商号与 class（24 位）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PciDeviceIdentity {
        pub vendor_identifier: u32,
        pub class_code: u32,
    }

    impl PciDeviceIdentity {
        /// 从 sysfs 里这个设备的 `vendor` 与 `class` 两个文件的原文读出来（各一行 `0x` 起头的十六进制，带不带行尾换行都认）；
        /// 有一个读不出交 `None`。
        #[must_use]
        pub fn from_sysfs_text(vendor_text: &str, class_text: &str) -> Option<Self> {
            let hexadecimal_value = |text: &str| {
                let digits = text.trim().strip_prefix("0x")?;
                u32::from_str_radix(digits, 16).ok()
            };
            Some(Self {
                vendor_identifier: hexadecimal_value(vendor_text)?,
                class_code: hexadecimal_value(class_text)?,
            })
        }

        /// NVIDIA 的显示控制器（显卡本身，不是同一块卡上的音频功能）。
        #[must_use]
        pub fn is_an_nvidia_display_controller(&self) -> bool {
            self.vendor_identifier == NVIDIA_VENDOR_IDENTIFIER
                && self.class_code >> 16 == PCI_DISPLAY_CONTROLLER_BASE_CLASS
        }
    }

    /// 读 PCI 设备表的结果。
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum PciNvidiaDisplayControllers {
        /// 每个设备的厂商号与 class 都读出来了：其中 NVIDIA 显示控制器有几个。
        Counted {
            nvidia_display_controller_count: usize,
        },
        /// PCI 总线看得见，而设备表列不出、或某个设备的 `vendor`、`class` 读不出、认不出。
        ListingUnreadable { reason: String },
        /// 看不见 PCI 总线：[`PCI_BUS_DIRECTORY`] 整个不在。这台机器上没有 PCI 设备可数，判定与「数到 0 个」同。
        PciBusNotVisible { reason: String },
    }

    /// 一串 PCI 设备里 NVIDIA 显示控制器的个数。
    #[must_use]
    pub fn count_nvidia_display_controllers(devices: &[PciDeviceIdentity]) -> usize {
        devices
            .iter()
            .filter(|device| device.is_an_nvidia_display_controller())
            .count()
    }

    /// 一条 GPU 对拍用例开跑前的结局。
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum GpuTestStart {
        /// 至少一张卡起好了上下文：照常跑。
        RunOnTheUsableCards,
        /// Vulkan 枚举里没有 NVIDIA 独显，PCI 上也没有 NVIDIA 显示控制器（或看不见 PCI 总线）：这台机器没有这种硬件，报「本次未跑」。
        NotRunNoGpuAdapter { reason: String },
        /// Vulkan 枚举出 NVIDIA 独显，却一张卡都没起来（`nvidia-smi` 读不到、空闲显存低于门槛、上下文起不来）：判红。
        AdapterPresentButNoCardStarted { reason: String },
        /// PCI 上有 NVIDIA 显示控制器，Vulkan 枚举里却没有一张 NVIDIA 独显（用户态 Vulkan 驱动坏了或不在）：判红。
        NvidiaCardOnPciButVulkanEnumeratesNone { reason: String },
        /// Vulkan 枚举里没有 NVIDIA 独显，PCI 总线看得见而设备表读不出：分不清没卡与驱动坏，判红
        /// （代码三方 test-script-align-code-r2 判决 J2 G1 定了 C620 留下的这一支：看不见总线才算没卡）。
        PciBusVisibleButListingUnreadable { reason: String },
    }

    /// `adapters` 是 Vulkan 枚举的结果，`pci` 是读 PCI 设备表的结果，`usable_card_count` 是用得上的卡数：
    /// 起好上下文的卡数（[`usable_gpu_cards`] 交回几张），卡池是找到的点名卡数（卡池起池时不起设备，借到哪张才起哪张）。
    #[must_use]
    pub fn decide_gpu_test_start(
        adapters: &[EnumeratedAdapter],
        pci: &PciNvidiaDisplayControllers,
        usable_card_count: usize,
    ) -> GpuTestStart {
        if usable_card_count > 0 {
            return GpuTestStart::RunOnTheUsableCards;
        }
        let nvidia_discrete_adapter_count = adapters
            .iter()
            .filter(|adapter| adapter.is_an_nvidia_discrete_gpu())
            .count();
        if nvidia_discrete_adapter_count > 0 {
            return GpuTestStart::AdapterPresentButNoCardStarted {
                reason: format!(
                    "Vulkan 枚举出 {nvidia_discrete_adapter_count} 张 NVIDIA 独显，一张都没起来（nvidia-smi 读不到这张卡、空闲显存低于门槛、或上下文起不来）：修好驱动或腾出显存再跑"
                ),
            };
        }
        match pci {
            PciNvidiaDisplayControllers::Counted {
                nvidia_display_controller_count,
            } => {
                if *nvidia_display_controller_count == 0 {
                    GpuTestStart::NotRunNoGpuAdapter {
                        reason: format!(
                            "Vulkan 枚举出 {} 个适配器，没有一张 NVIDIA 独显，PCI 上也没有 NVIDIA 显示控制器：{adapters:?}",
                            adapters.len()
                        ),
                    }
                } else {
                    GpuTestStart::NvidiaCardOnPciButVulkanEnumeratesNone {
                        reason: format!(
                            "PCI 上有 {nvidia_display_controller_count} 个 NVIDIA 显示控制器，Vulkan 枚举出的 {} 个适配器里却没有一张 NVIDIA 独显（{adapters:?}）。驱动坏了：修驱动，或改配置关 GPU",
                            adapters.len()
                        ),
                    }
                }
            }
            PciNvidiaDisplayControllers::ListingUnreadable { reason } => {
                GpuTestStart::PciBusVisibleButListingUnreadable {
                    reason: format!(
                        "Vulkan 枚举出的 {} 个适配器里没有一张 NVIDIA 独显，PCI 总线看得见而设备表读不出（{reason}）：分不清是没卡还是驱动坏了，不报本次未跑",
                        adapters.len()
                    ),
                }
            }
            PciNvidiaDisplayControllers::PciBusNotVisible { reason } => {
                GpuTestStart::NotRunNoGpuAdapter {
                    reason: format!(
                        "Vulkan 枚举出 {} 个适配器，没有一张 NVIDIA 独显，也看不见 PCI 总线（{reason}）：{adapters:?}",
                        adapters.len()
                    ),
                }
            }
        }
    }

    /// 这条用例报「本次未跑」时打的那一行（记号与写法在 harness 档一处定义，崩溃放量产品路径此刻借不到、卡上报 Out of Memory 也打它：
    /// `singlefs_harness::gpu_memory_ledger::gpu_not_run_line`）。
    #[must_use]
    pub fn not_run_line(test_name: &str, reason: &str) -> String {
        singlefs_harness::gpu_memory_ledger::gpu_not_run_line(test_name, reason)
    }

    fn adapter_device_kind(device_type: wgpu::DeviceType) -> AdapterDeviceKind {
        match device_type {
            wgpu::DeviceType::DiscreteGpu => AdapterDeviceKind::DiscreteGpu,
            wgpu::DeviceType::IntegratedGpu => AdapterDeviceKind::IntegratedGpu,
            wgpu::DeviceType::VirtualGpu => AdapterDeviceKind::VirtualGpu,
            wgpu::DeviceType::Cpu => AdapterDeviceKind::SoftwareRenderingOnTheCpu,
            wgpu::DeviceType::Other => AdapterDeviceKind::OtherOrUnknown,
        }
    }

    /// 本机 Vulkan 枚举出的每个适配器（与 [`usable_gpu_cards`] 同一个后端）。
    #[must_use]
    pub fn enumerate_vulkan_adapters() -> Vec<EnumeratedAdapter> {
        let instance = super::vulkan_instance_of_this_process();
        pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN))
            .into_iter()
            .map(|adapter| {
                let information = adapter.get_info();
                EnumeratedAdapter {
                    vendor_identifier: information.vendor,
                    device_kind: adapter_device_kind(information.device_type),
                }
            })
            .collect()
    }

    /// 读 `bus_directory`（平常是 [`PCI_BUS_DIRECTORY`]；测试给一棵假的）：它整个不在（`NotFound`）交
    /// [`PciNvidiaDisplayControllers::PciBusNotVisible`]；在，就照 [`read_pci_nvidia_display_controllers_under`] 读它下面的
    /// [`PCI_DEVICES_SUBDIRECTORY_NAME`]。总线目录在而设备表列不出、某个设备读不出，与总线目录本身看不了（`NotFound` 之外的错，
    /// 权限这类）一样交 [`PciNvidiaDisplayControllers::ListingUnreadable`]，不当成看不见总线。
    #[must_use]
    pub fn read_pci_nvidia_display_controllers_on_the_bus(
        bus_directory: &std::path::Path,
    ) -> PciNvidiaDisplayControllers {
        match std::fs::metadata(bus_directory) {
            Ok(_) => read_pci_nvidia_display_controllers_under(
                &bus_directory.join(PCI_DEVICES_SUBDIRECTORY_NAME),
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                PciNvidiaDisplayControllers::PciBusNotVisible {
                    reason: format!("{} 不在：{error}", bus_directory.display()),
                }
            }
            Err(error) => PciNvidiaDisplayControllers::ListingUnreadable {
                reason: format!("{} 看不了：{error}", bus_directory.display()),
            },
        }
    }

    /// 读 `devices_directory`（平常是 [`PCI_BUS_DIRECTORY`] 下的 [`PCI_DEVICES_SUBDIRECTORY_NAME`]；测试给一棵假的）下每个设备的
    /// `vendor` 与 `class`，数 NVIDIA 显示控制器；目录或任何一个设备的这两个文件读不出、认不出，交 [`PciNvidiaDisplayControllers::ListingUnreadable`]。
    #[must_use]
    pub fn read_pci_nvidia_display_controllers_under(
        devices_directory: &std::path::Path,
    ) -> PciNvidiaDisplayControllers {
        let unreadable =
            |what: String| PciNvidiaDisplayControllers::ListingUnreadable { reason: what };
        let entries = match std::fs::read_dir(devices_directory) {
            Ok(entries) => entries,
            Err(error) => {
                return unreadable(format!("{} 列不出：{error}", devices_directory.display()))
            }
        };
        let mut devices = Vec::new();
        for entry in entries {
            let device_directory = match entry {
                Ok(entry) => entry.path(),
                Err(error) => {
                    return unreadable(format!(
                        "{} 里一项读不出：{error}",
                        devices_directory.display()
                    ))
                }
            };
            let read = |file_name: &str| {
                std::fs::read_to_string(device_directory.join(file_name)).map_err(|error| {
                    format!(
                        "{} 读不出：{error}",
                        device_directory.join(file_name).display()
                    )
                })
            };
            let (vendor_text, class_text) = match (read("vendor"), read("class")) {
                (Ok(vendor_text), Ok(class_text)) => (vendor_text, class_text),
                (Err(what), Ok(_) | Err(_)) | (Ok(_), Err(what)) => return unreadable(what),
            };
            match PciDeviceIdentity::from_sysfs_text(&vendor_text, &class_text) {
                Some(device) => devices.push(device),
                None => {
                    return unreadable(format!(
                        "{} 的 vendor {:?}、class {:?} 认不出",
                        device_directory.display(),
                        vendor_text.trim(),
                        class_text.trim()
                    ))
                }
            }
        }
        PciNvidiaDisplayControllers::Counted {
            nvidia_display_controller_count: count_nvidia_display_controllers(&devices),
        }
    }

    /// 本机起好的每张卡（按空闲显存从大到小，至少一张）；本机没有 NVIDIA 卡时打 [`not_run_line`] 那一行、交回 `None`（调用的用例当场返回）。
    ///
    /// # Panics
    /// 有卡而用不上：Vulkan 枚举出 NVIDIA 独显而一张都没起来，PCI 上有 NVIDIA 显示控制器而 Vulkan 枚举不出，
    /// 或 Vulkan 里没有、PCI 总线看得见而设备表读不出。
    #[must_use]
    pub fn usable_cards_or_report_not_run(test_name: &str) -> Option<Vec<GpuCard>> {
        let cards = usable_gpu_cards();
        match decide_gpu_test_start(
            &enumerate_vulkan_adapters(),
            &read_pci_nvidia_display_controllers_on_the_bus(std::path::Path::new(
                PCI_BUS_DIRECTORY,
            )),
            cards.len(),
        ) {
            GpuTestStart::RunOnTheUsableCards => Some(cards),
            GpuTestStart::NotRunNoGpuAdapter { reason } => {
                println!("{}", not_run_line(test_name, &reason));
                None
            }
            GpuTestStart::AdapterPresentButNoCardStarted { reason }
            | GpuTestStart::NvidiaCardOnPciButVulkanEnumeratesNone { reason }
            | GpuTestStart::PciBusVisibleButListingUnreadable { reason } => {
                panic!("{test_name}：{reason}")
            }
        }
    }

    /// 空闲显存最大的那张卡；别的同 [`usable_cards_or_report_not_run`]。
    ///
    /// # Panics
    /// 同 [`usable_cards_or_report_not_run`]。
    #[must_use]
    pub fn first_usable_card_or_report_not_run(test_name: &str) -> Option<GpuCard> {
        usable_cards_or_report_not_run(test_name).map(|cards| {
            cards.into_iter().next().expect(
                "判成照跑只在起好的卡数大于 0 时（decide_gpu_test_start 第一句），交回的卡表不会是空的",
            )
        })
    }

    /// GPU 用例共用的卡池（[`card_pool::GpuCardPool`]）：只开 `gpu` 也编得进（显卡配置的读法在 `crate::gpu_card_configuration`）。
    pub use card_pool::{
        card_pool_started_in_this_process, gpu_card_pool_or_report_not_run,
        CardOfThePoolForTheProductPath, CardPoolLeaseRefusal, GpuCardLease, GpuCardPool,
        ScratchLeaseOnACard, CARD_QUOTA_ENVIRONMENT_VARIABLE,
    };

    /// 起池那一步（单测在别的用例拿着改环境的锁换了配置时现起一次，核它照的是哪一份显卡配置）。
    #[cfg(test)]
    pub(in crate::gpu_unit_checks) use card_pool::start_the_gpu_card_pool;

    mod card_pool {
        use super::super::{
            card_memory_reader_of_the_card_pool, device_found_idle_by_a_poll,
            device_wait_before_returning_a_lease, memory_budget_mebibytes_of,
            nvidia_discrete_cards_reported_by_nvidia_smi, nvidia_smi_memory_and_processes_report,
            poll_type_before_returning_a_lease, start_the_card, CardStartFailure,
            DeviceOwnerOnTheCard, DiscoveredCard, GpuCard,
        };
        use super::{
            decide_gpu_test_start, enumerate_vulkan_adapters, not_run_line,
            read_pci_nvidia_display_controllers_on_the_bus, GpuTestStart, PCI_BUS_DIRECTORY,
        };
        use crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
        use crate::gpu_card_configuration::{
            configured_cards_not_usable, GpuConfigurationOfTheCaller,
        };
        use singlefs_harness::gpu_memory_ledger::{
            card_ledger_capacity_mebibytes, reason_for_a_booking_without_room_now,
            reason_for_running_out_of_gpu_memory, wait_for_the_device_and_take_in_the_outcome,
            BookingRefusal, CardRoomForThisProcess, DeviceStartedOnDemand, DeviceStartedOnTheBooks,
            GpuMemoryBooking, GpuMemoryLedger, StatesBookedOnACard,
        };
        use singlefs_harness::numeric_environment_variable::{
            number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
        };

        /// 每张卡账的上限（MiB）：设了，每张卡上这个进程一共能占的取它、配置给这张卡的额度减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]、
        /// 这个进程此刻能占的减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`] 三者里最小的
        /// （`singlefs_harness::gpu_memory_ledger::card_ledger_capacity_mebibytes`）；不是每条用例的上限，借得比它多的照常判「放不下」。
        pub const CARD_QUOTA_ENVIRONMENT_VARIABLE: &str =
            "SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES";

        /// 池里的一张卡：照配置点了名、Vulkan 与 `nvidia-smi` 里都找得到；设备等这个进程在这张卡上借到一笔之后才起
        /// （[`GpuCardPool::started_card`]）：起好了留着给后面借的用，起不来不留、下一次借到再起，设备丢了也重起
        /// （第 10x 批代码轮判决 K1-3：起设备失败不缓存成这个进程永久的错）。
        struct CardOfThePool {
            discovered: DiscoveredCard,
            /// `GPU_CARDS_BY_PRIORITY` 配给这张卡的额度（MiB）；没配显卡优先级的是 `None`。
            configured_card_quota_mebibytes: Option<u64>,
            started: DeviceStartedOnDemand<GpuCard>,
        }

        /// 向卡池借、借不到的两种（崩溃放量产品路径照它报）。
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub enum CardPoolLeaseRefusal {
            /// 账上借不到（`singlefs_harness::gpu_memory_ledger::BookingRefusal`）。
            Booking(BookingRefusal),
            /// 账上借到了，那张卡上起设备时显存不够（[`CardStartFailure::OutOfGpuMemory`]）：那一笔已经还了，调用方报本次未跑。
            /// `reason` 是「本次未跑」那一行的原因（以「显存不够：」起头）。
            OutOfGpuMemoryStartingTheDevice { reason: String },
        }

        /// GPU 用例共用的卡：这台机器点名的每张卡在这个进程里一本显存账（[`GpuMemoryLedger`]，只记这个进程借的，
        /// 每次借现读卡上此刻的显存：`singlefs_harness::gpu_memory_ledger` 文件头），每张卡这个进程只起一个设备；
        /// 用例按自己实际要的量借（[`GpuCardPool::lease_card_or_report_not_run`]），借据丢掉时还。
        /// 一张卡这个进程一共能占的照 [`card_ledger_capacity_mebibytes`]：驱动的上下文与编好的内核每个进程在每张卡上付一份
        /// [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]。起池不读卡上此刻空闲多少，能不能借在借的那一刻判；这个进程第一次在一张卡上借到
        /// （连它的驱动那一份）了，才在这张卡上起设备，没借过的卡不起（第 10t 批代码轮判决 K2-3）。
        pub struct GpuCardPool {
            cards: Vec<CardOfThePool>,
            ledger: GpuMemoryLedger,
        }

        /// 借来的一张卡：[`GpuCardLease::card`] 与池里那张共用设备、编好的内核，显存额度与空闲改成借的量加 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]
        /// （判器照「额度 − 表 − 那一份」开草稿区，开出来正好是借的那么多）。丢掉时先等设备把这张卡上提交了的活做完、丢掉的缓冲交回驱动，
        /// 再销账（照 [`poll_type_before_returning_a_lease`] 等；等不成照旧记在账上）。字段照声明的次序丢：卡、账上的那一台、借据。
        pub struct GpuCardLease {
            card: GpuCard,
            /// 借据那张卡上的设备在账上的那一台（`card` 里那一份的克隆，借据等的就是它）。
            device_on_the_books: std::sync::Arc<DeviceStartedOnTheBooks<'static>>,
            booking: GpuMemoryBooking<'static>,
        }

        impl GpuCardLease {
            #[must_use]
            pub fn card(&self) -> &GpuCard {
                &self.card
            }

            /// 借的显存（MiB）。
            #[must_use]
            pub fn mebibytes(&self) -> u64 {
                self.booking.mebibytes()
            }

            /// 这一笔的借据号（[`GpuCardPool::is_the_lease_on_the_books`] 照它查）。
            #[must_use]
            pub fn lease_identifier(&self) -> u64 {
                self.booking.lease_identifier()
            }

            /// 这一笔此刻还记在这个进程的账上没有（核账：有卡用例照它判借据活到了用完，第 10t 批代码轮判决 K4-1）。
            #[must_use]
            pub fn is_on_the_books(&self) -> bool {
                self.booking.is_on_the_books()
            }
        }

        /// 崩溃放量产品路径在一张卡上借到的草稿区：借来的卡与借了几个状态（在账的锁里照此刻能借的压过）。
        pub struct ScratchLeaseOnACard {
            pub lease: GpuCardLease,
            pub states: u64,
        }

        /// 崩溃放量产品路径看池里的一张卡（还没起设备，[`GpuCardPool::cards_for_the_product_path`]）：序号、名字与适配器的存储缓冲绑定上限
        /// （设备按适配器的上限要，`start_the_card`）。
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct CardOfThePoolForTheProductPath {
            pub index: u32,
            pub name: String,
            pub max_storage_buffer_binding_size: u64,
        }

        impl Drop for GpuCardLease {
            fn drop(&mut self) {
                // wgpu 丢掉的缓冲要等设备做完这张卡上提交了的活才交回驱动：销账之前等完（用户 2026-10-08 定第 10z 批代码轮岔路 G2「改用乙」，
                // E174 两张卡上等完之后下一次读数里那笔缓冲都已不在），等不成（超时、wgpu 报错或 panic）这一笔照旧记在账上；wgpu 把等的时候
                // 设备上的错当致命、panic：接住、记成没等成，不在展开中照样往外抛、已经在展开中不再抛
                // （`singlefs_harness::gpu_memory_ledger::wait_for_the_device_and_take_in_the_outcome`，第 10z 批第八块）
                let device = self.card.device();
                let index = self.card.index;
                let lease_identifier = self.booking.lease_identifier();
                let mebibytes = self.booking.mebibytes();
                wait_for_the_device_and_take_in_the_outcome(
                    &mut self.booking,
                    &self.device_on_the_books,
                    || device.poll(poll_type_before_returning_a_lease()),
                    device_wait_before_returning_a_lease,
                    |polled| {
                        eprintln!(
                            "GPU_CARD_LEASE_RETURN_WAIT_FAILED index={index} lease={lease_identifier} mebibytes={mebibytes} polled={:?}：\
                             这一笔照旧记在账上，到那台设备上提交的活做完了、或那台设备记成丢了、放掉了才销",
                            polled.as_ref().map_err(|_panic| "等的时候 wgpu panic 了")
                        );
                    },
                );
            }
        }

        impl GpuCardPool {
            /// 池里每张卡存储缓冲绑定上限里最小的那个（用例借之前按它估一条草稿缓冲装几个状态）。设备按适配器的上限要
            /// （`start_the_card`），没起设备也照适配器读得到。
            #[must_use]
            pub fn smallest_max_storage_buffer_binding_size(&self) -> u64 {
                self.cards
                    .iter()
                    .map(|card| {
                        card.discovered
                            .adapter
                            .limits()
                            .max_storage_buffer_binding_size
                    })
                    .min()
                    .expect(
                        "池里至少一张卡：没有卡的机器上起不出池（gpu_card_pool_or_report_not_run）",
                    )
            }

            /// 借之前：列出账上有等设备没等成而照旧记着的借据的卡、连同那一刻照旧记着的那几笔，对池里那张卡留着的设备 poll 一次（不等）；
            /// 交回队列空了，那台设备上到 poll 那一刻为止提交的活都做完了、丢掉的缓冲交回了驱动，列出来的那几笔销账（第 10z 批第八块，
            /// 第 10z 批第六块代码轮判决 K1-2：等超时的那一笔不永久少借，两次超时之间没有别的等成时也出得来）。留着的设备丢了的不 poll
            /// （wgpu 在丢了的设备上 poll 会 panic），当场告诉账、照重起记（那几笔销账、这张卡照分不出算），不等借到之后走到重起
            /// （第 10z 批第十三块，第 10z 批第十、十一块代码轮判决 K4-2）。取留着的设备不等别的线程起设备（`DeviceStartedOnDemand::started_now`，
            /// 第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K4-2）。
            /// 不在账的锁里 poll，只销列卡那一刻记着的：它们丢掉的缓冲在列卡之前提交、丢掉，poll 交回队列空了之后什么时候销都不多给；
            /// 列卡之后才记成没等成的那一笔不销，它的活可以是 poll 之后才提交的（第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K1-1）。
            /// 列卡、丢没丢、poll 的结局怎么记都在 harness 档（`GpuMemoryLedger::take_in_the_devices_kept_before_booking`，不要卡就测得到），
            /// 这里只接卡池这一侧的四样：哪一格留着的设备、丢没丢、poll 一次、账上的那一台。
            fn take_in_the_devices_found_idle(&self) {
                self.ledger.take_in_the_devices_kept_before_booking(
                    |card_position| self.cards[card_position].started.started_now(),
                    |started: &GpuCard| started.device_condition().is_lost(),
                    |started: &GpuCard| {
                        device_found_idle_by_a_poll(&std::panic::catch_unwind(
                            std::panic::AssertUnwindSafe(|| {
                                started.device().poll(wgpu::PollType::Poll)
                            }),
                        ))
                    },
                    |started: &GpuCard| {
                        started
                            .device_on_the_books
                            .as_deref()
                            .expect("卡池起的设备起好时放进了账上的那一台（started_card）")
                    },
                );
            }

            /// 借到 `booking` 的那张卡起好的设备：留着的设备没丢就拿它，没有（第一次要、上一次起不来）或丢了就起一个
            /// （借到就登记了这个进程，先登记、后建设备）；起不来交为什么、不留着。要一笔借据才起得了：没借过的卡起不了设备。
            /// 丢了的设备重起之前，驱动那一份在账上重新登记，此刻放不下就不起、交显存不够
            /// （`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`，第 10z 批）。
            ///
            /// # Panics
            /// 这个进程还没在这张卡的账上登记（`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`）。
            fn started_card(
                &'static self,
                booking: &GpuMemoryBooking<'static>,
            ) -> Result<GpuCard, CardStartFailure> {
                let card_position = booking.card_position();
                let card = &self.cards[card_position];
                card.started.started_or_start(
                    |started: &GpuCard| started.device_condition().is_lost(),
                    || {
                        let which_card = format!(
                            "序号 {} 的卡（{}）",
                            card.discovered.reported.index, card.discovered.name
                        );
                        self.ledger
                            .prepare_the_driver_share_before_starting_the_device(card_position)
                            .map_err(|refusal| match refusal {
                                BookingRefusal::NoCardHasRoomNow { .. } => {
                                    CardStartFailure::OutOfGpuMemory {
                                        reason: format!(
                                            "{which_card}上丢了的设备要重起，账上此刻放不下新设备的驱动那一份（{refusal:?}）"
                                        ),
                                    }
                                }
                                BookingRefusal::NeedLargerThanEveryCard(_)
                                | BookingRefusal::CardMemoryUnreadable { .. } => {
                                    CardStartFailure::DeviceNotStarted {
                                        reason: format!(
                                            "{which_card}上丢了的设备要重起，账上重新登记驱动那一份时 {refusal:?}"
                                        ),
                                    }
                                }
                            })?;
                        let mut started =
                            start_the_card(&card.discovered, DeviceOwnerOnTheCard::TheCardPool)?;
                        let device_on_the_books = self
                            .ledger
                            .record_the_device_started_on_the_card(card_position);
                        started.memory_quota_mebibytes = card.configured_card_quota_mebibytes;
                        started.device_on_the_books = Some(std::sync::Arc::new(device_on_the_books));
                        Ok(started)
                    },
                )
            }

            /// 借 `need_mebibytes`：照卡的优先级记在第一张此刻放得下的卡上（前一张放满了才用下一张）；此刻每张都放不下
            /// （这个进程别的用例或卡上别的 GPU 用户占着）不等，打 [`not_run_line`] 那一行（原因写显存不够）、交回 `None`
            /// （调用的用例当场返回）。每借到一次打一行 `GPU_CARD_LEASE test=… index=… mebibytes=… room_mebibytes=…`。
            ///
            /// 借到的卡上起设备时显存不够同样报「本次未跑」（原因写显存不够）。
            ///
            /// # Panics
            /// 要的比每张卡空着时能借的都大（配了 GPU 就一定用 GPU，放不下判红，不悄悄换小）；卡上的显存读不出；
            /// 借到的卡上起不了设备（显存不够之外的原因）；要 0 MiB。
            #[must_use]
            pub fn lease_card_or_report_not_run(
                &'static self,
                test_name: &str,
                need_mebibytes: u64,
            ) -> Option<GpuCardLease> {
                match self.lease_card(test_name, need_mebibytes) {
                    Ok(lease) => Some(lease),
                    Err(reason) => {
                        println!("{}", not_run_line(test_name, &reason));
                        None
                    }
                }
            }

            /// 借一次：借到交借来的卡，此刻借不到、借到的卡上起设备时显存不够交一句为什么。
            fn lease_card(
                &'static self,
                test_name: &str,
                need_mebibytes: u64,
            ) -> Result<GpuCardLease, String> {
                self.take_in_the_devices_found_idle();
                let booking = match self.ledger.book_on_the_first_card_with_room(need_mebibytes) {
                    Ok(booking) => booking,
                    Err(BookingRefusal::NeedLargerThanEveryCard(refusal)) => panic!(
                        "{test_name}：要借 {} MiB，池里最大的一张卡空着时也只借得出 {} MiB：这台机器的卡放不下这条用例，\
                         给卡更大的额度（GPU_CARDS_BY_PRIORITY），或缩小用例",
                        refusal.need_mebibytes, refusal.largest_ceiling_mebibytes
                    ),
                    Err(BookingRefusal::NoCardHasRoomNow {
                        need_mebibytes: wanted_mebibytes,
                        card_rooms_mebibytes,
                    }) => {
                        return Err(reason_for_a_booking_without_room_now(
                            wanted_mebibytes,
                            &card_rooms_mebibytes,
                        ))
                    }
                    Err(BookingRefusal::CardMemoryUnreadable { reason }) => {
                        panic!("{test_name}：卡上的显存读不出：{reason}")
                    }
                };
                self.card_lent_for_the_booking(test_name, booking)
            }

            /// 同 [`Self::lease_card_or_report_not_run`]（照卡的优先级记在第一张此刻放得下的卡上），借不到交账的结局、
            /// 由调用方定怎么报：崩溃放量产品路径在起过卡池的进程里，第 ② 段核对与单元校验和对拍在卡上建的缓冲照它借
            /// （`singlefs_harness::gpu_memory_ledger::scratch_booking_refusal_outcome`：此刻借不到报本次未跑，别的判红）。
            ///
            /// # Errors
            /// 见 [`CardPoolLeaseRefusal`]。
            ///
            /// # Panics
            /// 借到的卡上起不了设备（显存不够之外的原因）；要 0 MiB。
            pub fn lease_the_first_card_with_room_now(
                &'static self,
                run_name: &str,
                need_mebibytes: u64,
            ) -> Result<GpuCardLease, CardPoolLeaseRefusal> {
                self.take_in_the_devices_found_idle();
                let booking = self
                    .ledger
                    .book_on_the_first_card_with_room(need_mebibytes)
                    .map_err(CardPoolLeaseRefusal::Booking)?;
                self.card_lent_for_the_booking(run_name, booking)
                    .map_err(
                        |reason| CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice { reason },
                    )
            }

            /// 序号是 `index`（`nvidia-smi` 给的）的那张卡在池里排第几。
            ///
            /// # Panics
            /// 池里没有这张卡（崩溃放量产品路径的卡都取自这个池，[`Self::cards_for_the_product_path`]）。
            fn position_of_the_card_with_index(&self, index: u32) -> usize {
                self.cards
                    .iter()
                    .position(|card| card.discovered.reported.index == index)
                    .unwrap_or_else(|| {
                        panic!("序号 {index} 的卡不在这个进程的卡池里：产品路径的卡都取自这个池")
                    })
            }

            /// 崩溃放量产品路径第 ① 段借草稿区：`card_indices` 是这一趟用的卡（按优先级，都是池里的卡），每张最多装
            /// `states_each_card_holds` 个状态；把 `pending_states` 个状态装到此刻借得到的卡上，在账的锁里现读每张卡、照此刻能借的
            /// 压状态数再借（几个状态要多少显存照 `mebibytes_for_states`），借不到的卡不装
            /// （`singlefs_harness::gpu_memory_ledger::GpuMemoryLedger::book_states_on_the_cards_with_room`）。
            /// 借到的每张交成借来的卡（额度是借的量加驱动那一份，判器照它开出来的草稿区正好是借的状态数；借到了才在那张卡上起设备）；
            /// 起设备时显存不够的卡还掉那一笔、这一趟不用（同借不到的卡）。交回与 `card_indices` 逐张对上。
            ///
            /// # Errors
            /// 见 [`CardPoolLeaseRefusal`]：借到的每张卡起设备时都显存不够是 [`CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice`]。
            ///
            /// # Panics
            /// 有一张不是池里的卡；借到的卡上起不了设备（显存不够之外的原因）。
            pub fn book_the_scratch_of_the_cards_with_room_now(
                &'static self,
                run_name: &str,
                card_indices: &[u32],
                pending_states: u64,
                states_each_card_holds: &[u64],
                mebibytes_for_states: &dyn Fn(u64) -> u64,
            ) -> Result<Vec<Option<ScratchLeaseOnACard>>, CardPoolLeaseRefusal> {
                let card_positions: Vec<usize> = card_indices
                    .iter()
                    .map(|index| self.position_of_the_card_with_index(*index))
                    .collect();
                self.take_in_the_devices_found_idle();
                let booked = self
                    .ledger
                    .book_states_on_the_cards_with_room(
                        &card_positions,
                        pending_states,
                        states_each_card_holds,
                        mebibytes_for_states,
                    )
                    .map_err(CardPoolLeaseRefusal::Booking)?;
                let mut first_card_short_of_memory = None;
                let leases: Vec<Option<ScratchLeaseOnACard>> = booked
                    .into_iter()
                    .map(|booked| {
                        let StatesBookedOnACard { states, booking } = booked?;
                        match self.card_lent_for_the_booking(run_name, booking) {
                            Ok(lease) => Some(ScratchLeaseOnACard { lease, states }),
                            Err(reason) => {
                                first_card_short_of_memory.get_or_insert(reason);
                                None
                            }
                        }
                    })
                    .collect();
                if leases.iter().all(Option::is_none) {
                    return Err(CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice {
                        reason: first_card_short_of_memory.expect(
                            "账交回借到了至少一张：一张都没交成借来的卡，只能是借到的每张起设备时都显存不够",
                        ),
                    });
                }
                Ok(leases)
            }

            /// 记上了的那一笔交成借来的卡：在那张卡上起设备（还没有起好的设备时，[`Self::started_card`]），与池里那张共用设备，
            /// 额度与空闲改成借的量加驱动那一份；打一行 `GPU_CARD_LEASE`。起设备时显存不够：还掉这一笔，交「本次未跑」那一行的原因。
            ///
            /// # Panics
            /// 那张卡上起不了设备（显存不够之外的原因：驱动坏了、适配器不给要的上限）。
            fn card_lent_for_the_booking(
                &'static self,
                test_name: &str,
                booking: GpuMemoryBooking<'static>,
            ) -> Result<GpuCardLease, String> {
                let need_mebibytes = booking.mebibytes();
                let mut card = match self.started_card(&booking) {
                    Ok(card) => card,
                    Err(CardStartFailure::OutOfGpuMemory { reason }) => {
                        return Err(reason_for_running_out_of_gpu_memory(&format!(
                            "起设备时（{reason}）"
                        )));
                    }
                    Err(CardStartFailure::DeviceNotStarted { reason }) => {
                        panic!("{test_name}：{reason}：账上登记了这个进程，卡却起不来上下文（驱动坏了）")
                    }
                };
                let device_on_the_books = std::sync::Arc::clone(
                    card.device_on_the_books
                        .as_ref()
                        .expect("卡池起的设备起好时放进了账上的那一台（started_card）"),
                );
                card.memory_quota_mebibytes = Some(need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER);
                card.free_memory_mebibytes = need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER;
                eprintln!(
                    "GPU_CARD_LEASE test={test_name} card={:?} index={} mebibytes={need_mebibytes} room_mebibytes={}",
                    card.name,
                    card.index,
                    booking.card_room_mebibytes_when_booked()
                );
                Ok(GpuCardLease {
                    card,
                    device_on_the_books,
                    booking,
                })
            }

            /// 这个进程此刻与这几张卡空着时，在 `card_indices` 这几张卡（都是池里的卡）上各能借多少（MiB，现读一次；与 `card_indices` 逐张对上）。
            ///
            /// # Errors
            /// 卡上的显存读不出。
            ///
            /// # Panics
            /// 有一张不是池里的卡。
            pub fn card_rooms_now_and_when_empty_of(
                &self,
                card_indices: &[u32],
            ) -> Result<Vec<CardRoomForThisProcess>, String> {
                let card_positions: Vec<usize> = card_indices
                    .iter()
                    .map(|index| self.position_of_the_card_with_index(*index))
                    .collect();
                self.take_in_the_devices_found_idle();
                self.ledger
                    .card_rooms_now_and_when_empty(&card_positions)
                    .map_err(|refusal| format!("读不出卡上此刻的显存：{refusal:?}"))
            }

            /// 序号是 `index` 的那张卡上，借据号 `lease_identifier` 的那一笔此刻还记在这个进程的账上没有（核账，第 10t 批代码轮判决 K4-1）。
            ///
            /// # Panics
            /// 池里没有这张卡。
            #[must_use]
            pub fn is_the_lease_on_the_books(&self, index: u32, lease_identifier: u64) -> bool {
                self.ledger.is_the_lease_on_the_books(
                    self.position_of_the_card_with_index(index),
                    lease_identifier,
                )
            }

            /// 序号是 `index` 的那张卡上这个进程起了设备没有（只在借到过的卡上起，第 10t 批代码轮判决 K2-3）。
            ///
            /// # Panics
            /// 池里没有这张卡。
            #[must_use]
            pub fn is_the_device_started_on_the_card(&self, index: u32) -> bool {
                self.cards[self.position_of_the_card_with_index(index)]
                    .started
                    .is_started()
            }

            /// 池里每张卡（按优先级）的序号与配置给它的额度（MiB，没配显卡优先级的是 `None`），不起设备。
            #[must_use]
            pub fn cards_in_priority_order_as_configured(&self) -> Vec<(u32, Option<u64>)> {
                self.cards
                    .iter()
                    .map(|card| {
                        (
                            card.discovered.reported.index,
                            card.configured_card_quota_mebibytes,
                        )
                    })
                    .collect()
            }

            /// 崩溃放量产品路径这一趟用池里哪几张卡（`crate::crash_amplification`：这个进程起过卡池时，放量照池里的卡判，不另起一套设备）：
            /// 池里每张卡，按池的优先级，不登记、不起设备（第 10t 批代码轮判决 K2-3：只在用到的卡上起设备）。放量借到哪张卡
            /// （[`Self::book_the_scratch_of_the_cards_with_room_now`]、[`Self::lease_the_first_card_with_room_now`]）才在哪张上起。
            #[must_use]
            pub fn cards_for_the_product_path(&self) -> Vec<CardOfThePoolForTheProductPath> {
                self.cards
                    .iter()
                    .map(|card| CardOfThePoolForTheProductPath {
                        index: card.discovered.reported.index,
                        name: card.discovered.name.clone(),
                        max_storage_buffer_binding_size: card
                            .discovered
                            .adapter
                            .limits()
                            .max_storage_buffer_binding_size,
                    })
                    .collect()
            }
        }

        /// 起池那一次：照哪一份显卡配置挑的卡（调用方给这个进程的那一份）与结局。进程里只判一次，之后每条用例照它。
        pub(in crate::gpu_unit_checks) struct GpuCardPoolStart {
            pub(in crate::gpu_unit_checks) configuration: &'static GpuConfigurationOfTheCaller,
            outcome: GpuCardPoolStartOutcome,
        }

        /// 起池那一次的结局。
        enum GpuCardPoolStartOutcome {
            Started(GpuCardPool),
            NotRunNoGpuAdapter { reason: String },
            CardsPresentButNotUsable { reason: String },
        }

        /// 这台机器给卡池点名的卡（还没起设备）。
        struct CardsNamedForThisMachine {
            /// 点名的卡与配置给它的额度，按配置的次序（没配显卡优先级时是本机每张卡，按空闲显存从大到小、没有额度）。
            cards: Vec<(DiscoveredCard, Option<u64>)>,
            /// 配置点了名、本机 Vulkan 或 `nvidia-smi` 里却找不到的卡（序号，按配置的次序）。
            named_but_not_found: Vec<u32>,
            /// 配置读不出、写法不对、没给这台机器点卡：一张卡都不取，原因在这里。
            configuration_refusal: Option<String>,
        }

        /// 照 `configuration`（调用方给这个进程的显卡配置）挑这台机器的卡：`GPU_CARDS_BY_PRIORITY`（那一份里的环境变量，没设读多机配置里的那两个键，
        /// [`GpuConfigurationOfTheCaller::cards_by_priority`]）
        /// 里这台机器（`SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`，没设是本机）的那几项，照配置的次序与额度（与崩溃放量取卡同一个约定）；
        /// 都没配就是本机每张 NVIDIA 独显，按空闲显存从大到小、不设额度。只找卡、不起设备，也不看此刻空闲多少（能不能借在借的那一刻判）。
        fn cards_named_for_this_machine(
            configuration: &GpuConfigurationOfTheCaller,
        ) -> CardsNamedForThisMachine {
            let refused = |reason: String| CardsNamedForThisMachine {
                cards: Vec::new(),
                named_but_not_found: Vec::new(),
                configuration_refusal: Some(reason),
            };
            let host = match configuration.host() {
                Ok(host) => host,
                Err(reason) => return refused(reason),
            };
            // 显卡先后写在多机配置里（判决 J5、J11）：仓根取这一份源码编进来的那个包往上两层，多机配置按它的 git common-dir 找主工作树那一份
            let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(std::path::Path::parent)
                .expect("包目录在 crates/ 下，往上两层是仓根");
            let configuration_file =
                match configuration.multi_host_configuration_file(repository_root) {
                    Ok(configuration_file) => configuration_file,
                    Err(reason) => return refused(reason),
                };
            let configured = match configuration.cards_by_priority(configuration_file.as_deref()) {
                Ok(configured) => configured,
                Err(reason) => return refused(reason),
            };
            let mut discovered = nvidia_discrete_cards_reported_by_nvidia_smi();
            match configured {
                Some(configured) => {
                    let mine: Vec<(u32, u64)> = configured
                        .iter()
                        .filter(|card| card.host == host)
                        .map(|card| (card.index, card.memory_quota_mebibytes))
                        .collect();
                    if mine.is_empty() {
                        return refused(format!(
                            "配置的 GPU_CARDS_BY_PRIORITY 里没有 {} 这台机器的卡",
                            host.name()
                        ));
                    }
                    let mut cards = Vec::with_capacity(mine.len());
                    for (index, quota) in &mine {
                        match discovered
                            .iter()
                            .position(|card| card.reported.index == *index)
                        {
                            Some(position) => cards.push((discovered.remove(position), Some(*quota))),
                            None => eprintln!(
                                "GPU_CARD_NOT_USABLE index={index}：配置点名了这张卡，本机 Vulkan 或 nvidia-smi 里却找不到它"
                            ),
                        }
                    }
                    let found: Vec<u32> =
                        cards.iter().map(|(card, _)| card.reported.index).collect();
                    CardsNamedForThisMachine {
                        named_but_not_found: configured_cards_not_usable(&mine, &found),
                        cards,
                        configuration_refusal: None,
                    }
                }
                None => {
                    discovered
                        .sort_by_key(|card| std::cmp::Reverse(card.reported.free_memory_mebibytes));
                    CardsNamedForThisMachine {
                        cards: discovered.into_iter().map(|card| (card, None)).collect(),
                        named_but_not_found: Vec::new(),
                        configuration_refusal: None,
                    }
                }
            }
        }

        /// 开点名的那几张卡的账（这个进程里，不读卡、不起设备）；交回池。
        fn open_the_card_pool(named: Vec<(DiscoveredCard, Option<u64>)>) -> GpuCardPool {
            let card_quota = number_from_the_variable_value_panicking_when_unreadable::<u64>(
                CARD_QUOTA_ENVIRONMENT_VARIABLE,
                process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),
            );
            let buses: Vec<String> = named
                .iter()
                .map(|(card, _)| card.pci_bus_identifier.clone())
                .collect();
            let configured_card_quotas: Vec<Option<u64>> =
                named.iter().map(|(_, quota)| *quota).collect();
            // 每次借读一份 nvidia-smi 报告，空闲与这个进程占着的出自同一份（第 10x 批代码轮判决 K1-1）
            let ledger = GpuMemoryLedger::new(
                named.len(),
                card_memory_reader_of_the_card_pool(
                    Box::new(nvidia_smi_memory_and_processes_report),
                    buses,
                ),
                Box::new(move |position, may_hold_mebibytes| {
                    let configured = configured_card_quotas[position];
                    card_ledger_capacity_mebibytes(
                        card_quota,
                        configured,
                        memory_budget_mebibytes_of(configured, may_hold_mebibytes),
                        may_hold_mebibytes,
                        MEBIBYTES_LEFT_FOR_THE_DRIVER,
                    )
                }),
                MEBIBYTES_LEFT_FOR_THE_DRIVER,
            );
            let cards = named
                .into_iter()
                .map(
                    |(discovered, configured_card_quota_mebibytes)| CardOfThePool {
                        discovered,
                        configured_card_quota_mebibytes,
                        started: DeviceStartedOnDemand::new(),
                    },
                )
                .collect();
            GpuCardPool { cards, ledger }
        }

        /// 起池：照调用方给这个进程的显卡配置挑卡（[`GpuConfigurationOfTheCaller::of_this_process`]），不读此刻的进程环境——
        /// 第一次起池时别的用例可能正拿着改环境的锁、把多机配置换成它的临时那一份，整个进程的卡池不跟它走。
        pub(in crate::gpu_unit_checks) fn start_the_gpu_card_pool() -> GpuCardPoolStart {
            let configuration = GpuConfigurationOfTheCaller::of_this_process();
            GpuCardPoolStart {
                configuration,
                outcome: card_pool_start_outcome_on(configuration),
            }
        }

        /// 照 `configuration` 挑卡、判有没有卡、开账：起池那一次的结局。
        fn card_pool_start_outcome_on(
            configuration: &GpuConfigurationOfTheCaller,
        ) -> GpuCardPoolStartOutcome {
            let named = cards_named_for_this_machine(configuration);
            let decision = decide_gpu_test_start(
                &enumerate_vulkan_adapters(),
                &read_pci_nvidia_display_controllers_on_the_bus(std::path::Path::new(
                    PCI_BUS_DIRECTORY,
                )),
                named.cards.len(),
            );
            let configuration_note = named
                .configuration_refusal
                .as_ref()
                .map_or(String::new(), |reason| format!("（显卡配置：{reason}）"));
            match decision {
                GpuTestStart::RunOnTheUsableCards => {
                    if named.named_but_not_found.is_empty() {
                        GpuCardPoolStartOutcome::Started(open_the_card_pool(named.cards))
                    } else {
                        // 点名的卡少一张都不跑（用户 2026-09-29 定：配了 GPU 就要它干活，不行就明确报错，不接受少用几张的默认降级）
                        GpuCardPoolStartOutcome::CardsPresentButNotUsable {
                            reason: format!(
                                "配置点名给这台机器的卡里，序号 {:?} 用不了（上面 GPU_CARD_NOT_USABLE 那几行）：点名的卡每一张都要用得上",
                                named.named_but_not_found
                            ),
                        }
                    }
                }
                GpuTestStart::NotRunNoGpuAdapter { reason } => {
                    GpuCardPoolStartOutcome::NotRunNoGpuAdapter { reason }
                }
                GpuTestStart::AdapterPresentButNoCardStarted { reason }
                | GpuTestStart::NvidiaCardOnPciButVulkanEnumeratesNone { reason }
                | GpuTestStart::PciBusVisibleButListingUnreadable { reason } => {
                    GpuCardPoolStartOutcome::CardsPresentButNotUsable {
                        reason: format!("{reason}{configuration_note}"),
                    }
                }
            }
        }

        static GPU_CARD_POOL: std::sync::OnceLock<GpuCardPoolStart> = std::sync::OnceLock::new();

        /// 这个进程的卡池：第一条来要的用例起池（[`start_the_gpu_card_pool`]：只在这时判一次有没有卡、找一次点名的卡，挑卡照 [`cards_named_for_this_machine`]；
        /// 设备等借到那张卡时才起），之后每条用例拿同一个。本机没有 NVIDIA 卡时每条来要的用例各打一行
        /// [`not_run_line`]、交回 `None`（调用的用例当场返回）。先判有没有卡、建好表知道要多少显存之后再借（[`GpuCardPool::lease_card_or_report_not_run`]）。
        ///
        /// # Panics
        /// 有卡而用不上（同 [`super::usable_cards_or_report_not_run`]）；显卡配置写错、没给这台机器点卡、点名的卡少一张找不到；
        /// [`CARD_QUOTA_ENVIRONMENT_VARIABLE`] 设了而认不出。
        #[must_use]
        pub fn gpu_card_pool_or_report_not_run(test_name: &str) -> Option<&'static GpuCardPool> {
            let start = GPU_CARD_POOL.get_or_init(start_the_gpu_card_pool);
            match &start.outcome {
                GpuCardPoolStartOutcome::Started(pool) => Some(pool),
                GpuCardPoolStartOutcome::NotRunNoGpuAdapter { reason } => {
                    println!("{}", not_run_line(test_name, reason));
                    None
                }
                GpuCardPoolStartOutcome::CardsPresentButNotUsable { reason } => {
                    panic!(
                        "{test_name}：{reason}（卡池照调用方给这个进程的显卡配置挑的卡：{:?}）",
                        start.configuration
                    )
                }
            }
        }

        /// 这个进程已经起好的卡池；还没起过池、或起池判成没卡、有卡而用不上，交 `None`（不起池，什么都不打）。
        #[must_use]
        pub fn card_pool_started_in_this_process() -> Option<&'static GpuCardPool> {
            match GPU_CARD_POOL.get().map(|start| &start.outcome) {
                Some(GpuCardPoolStartOutcome::Started(pool)) => Some(pool),
                Some(
                    GpuCardPoolStartOutcome::NotRunNoGpuAdapter { .. }
                    | GpuCardPoolStartOutcome::CardsPresentButNotUsable { .. },
                )
                | None => None,
            }
        }
    }
}

/// 代码三方 test-script-align-code-r2 判决 J2 G1：看不见 PCI 总线（总线目录整个不在）算没卡、报本次未跑；
/// 总线看得见而设备表列不出、某个设备读不出照旧判红。判定是纯函数，读 PCI 用假的 sysfs 树，不要显卡。
#[cfg(test)]
mod tests {
    /// 整个进程只建一份 Vulkan 实例：两次取到的是同一份，建的次数记 1（每建一次加载器就把本机每个 ICD 再 dlopen 一遍，
    /// 静态 TLS 吃光之后 NVIDIA 的驱动装不上，`vulkan_instance_of_this_process` 的文档注释写了来历）。
    /// 判别力（`crates/mutations.tsv`「GPU 取卡 Vulkan 实例每次新建」）：改成每次新建一份，两次取到的不是同一份、建的次数是 2。
    #[test]
    fn the_vulkan_instance_is_created_once_per_process_and_reused() {
        let first = super::vulkan_instance_of_this_process();
        let second = super::vulkan_instance_of_this_process();
        assert!(
            std::ptr::eq(first, second),
            "两次取到的 Vulkan 实例要是同一份"
        );
        assert_eq!(
            super::VULKAN_INSTANCES_CREATED_IN_THIS_PROCESS
                .load(std::sync::atomic::Ordering::SeqCst),
            1,
            "整个进程只建一份实例"
        );
    }

    use super::gpu_test_start::{
        decide_gpu_test_start, read_pci_nvidia_display_controllers_on_the_bus, GpuTestStart,
        PciNvidiaDisplayControllers, PCI_DEVICES_SUBDIRECTORY_NAME,
    };
    use super::{
        card_memory_reader_of_the_card_pool, cause_of_the_device_loss, device_found_idle_by_a_poll,
        device_wait_before_returning_a_lease, failure_of_a_device_request,
        handle_an_uncaptured_wgpu_error, has_devices_started_outside_the_card_pool_on_the_bus,
        poll_type_before_returning_a_lease, record_the_loss_of_the_device,
        started_with_its_owner_recorded, CardStartFailure, DeviceConditionOfACard,
        DeviceOwnerOnTheCard,
    };
    use singlefs_harness::gpu_memory_ledger::{
        CauseOfTheDeviceLoss, DeviceWaitBeforeReturningTheLease, OccupiedByThisProcess,
    };

    // 扫源码的用例里，串里的花括号一律写成 \u 转义：门禁 code-source-discipline 的 mutation-tables 格配花括号时不跳字符串，
    // 串里落单的半边括号会让它把后面的用例算到别的模块下（第 10z 批第八块）。
    /// 这份文件测试模块的头两行（扫源码的用例从这里切掉测试模块）。
    const TESTS_MODULE_OPENING: &str = "#[cfg(test)]\nmod tests \u{7b}";
    /// 顶格的右括号自成一行（函数体在这里收尾）。
    const TOP_LEVEL_CLOSING_BRACE_LINE: &str = "\n\u{7d}\n";

    /// J8、M4 第 11 条：块属性里的驱动版本取 wgpu adapter info 的 `driver` 与 `driver_info` 两项，空格接起来；一项空只取另一项，
    /// 两项都空写一句没报（不写空串）。
    #[test]
    fn the_driver_version_of_a_card_joins_the_driver_name_and_its_information() {
        assert_eq!(
            super::driver_version_of_the_adapter("NVIDIA", "580.95.05"),
            "NVIDIA 580.95.05"
        );
        assert_eq!(
            super::driver_version_of_the_adapter("NVIDIA", " "),
            "NVIDIA"
        );
        assert_eq!(
            super::driver_version_of_the_adapter("", "580.95.05"),
            "580.95.05"
        );
        assert_eq!(
            super::driver_version_of_the_adapter("", ""),
            "driver-not-reported-by-wgpu"
        );
    }

    /// 假的 sysfs 里 PCI 总线那一截的样子。
    enum FakePciBus<'devices> {
        /// `<根>/bus` 在，`<根>/bus/pci` 不在。
        NotThere,
        /// `<根>/bus/pci` 在，它下面没有设备子目录。
        ThereWithoutItsDevicesDirectory,
        /// `<根>/bus/pci/devices/<地址>` 每个设备一个子目录，`vendor` 与 `class` 照写；`None` 的那个文件不建。
        ThereWithDevices(&'devices [(&'devices str, Option<&'devices str>, Option<&'devices str>)]),
    }

    /// 照 `pci_bus` 建一棵假的 sysfs，交回 `(根, 总线目录 <根>/bus/pci)`；用完由调用方删根。
    fn fake_sysfs(
        test_name: &str,
        pci_bus: &FakePciBus<'_>,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "singlefs-gpu-test-start-sysfs-{}-{test_name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bus_directory = root.join("bus").join("pci");
        std::fs::create_dir_all(root.join("bus")).expect("临时目录建得出");
        match pci_bus {
            FakePciBus::NotThere => {}
            FakePciBus::ThereWithoutItsDevicesDirectory => {
                std::fs::create_dir_all(&bus_directory).expect("临时目录建得出");
            }
            FakePciBus::ThereWithDevices(devices) => {
                for (address, vendor_text, class_text) in *devices {
                    let device_directory = bus_directory
                        .join(PCI_DEVICES_SUBDIRECTORY_NAME)
                        .join(address);
                    std::fs::create_dir_all(&device_directory).expect("临时目录建得出");
                    if let Some(vendor_text) = vendor_text {
                        std::fs::write(device_directory.join("vendor"), vendor_text)
                            .expect("写得进临时目录");
                    }
                    if let Some(class_text) = class_text {
                        std::fs::write(device_directory.join("class"), class_text)
                            .expect("写得进临时目录");
                    }
                }
            }
        }
        (root, bus_directory)
    }

    #[test]
    fn no_nvidia_adapter_and_an_invisible_pci_bus_is_reported_as_not_run() {
        let start = decide_gpu_test_start(
            &[],
            &PciNvidiaDisplayControllers::PciBusNotVisible {
                reason: "假的：/sys/bus/pci 不在".to_string(),
            },
            0,
        );
        assert!(
            matches!(start, GpuTestStart::NotRunNoGpuAdapter { .. }),
            "Vulkan 里没有 NVIDIA 独显、又看不见 PCI 总线的机器没有这种硬件，要报「本次未跑」；判成了 {start:?}"
        );
    }

    #[test]
    fn a_pci_bus_directory_that_is_not_there_is_read_as_the_bus_not_being_visible() {
        let (root, bus_directory) = fake_sysfs("bus-not-there", &FakePciBus::NotThere);
        let read = read_pci_nvidia_display_controllers_on_the_bus(&bus_directory);
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            matches!(read, PciNvidiaDisplayControllers::PciBusNotVisible { .. }),
            "总线目录整个不在是看不见 PCI 总线，不是设备表读不出：{read:?}"
        );
    }

    #[test]
    fn a_visible_pci_bus_without_its_devices_directory_is_an_unreadable_listing_not_an_invisible_bus(
    ) {
        let (root, bus_directory) = fake_sysfs(
            "bus-without-devices",
            &FakePciBus::ThereWithoutItsDevicesDirectory,
        );
        let read = read_pci_nvidia_display_controllers_on_the_bus(&bus_directory);
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            matches!(read, PciNvidiaDisplayControllers::ListingUnreadable { .. }),
            "总线目录在、设备子目录列不出是读不出，不许当成看不见总线：{read:?}"
        );
    }

    #[test]
    fn a_visible_pci_bus_with_a_device_whose_vendor_cannot_be_read_and_no_nvidia_adapter_is_red() {
        let (root, bus_directory) = fake_sysfs(
            "device-vendor-unreadable",
            &FakePciBus::ThereWithDevices(&[("0000:00:1f.3", None, Some("0x040300\n"))]),
        );
        let read = read_pci_nvidia_display_controllers_on_the_bus(&bus_directory);
        let _ = std::fs::remove_dir_all(&root);
        let start = decide_gpu_test_start(&[], &read, 0);
        assert!(
            matches!(
                start,
                GpuTestStart::PciBusVisibleButListingUnreadable { .. }
            ),
            "总线看得见而某个设备的 vendor 读不出，分不清没卡与驱动坏，照旧判红、不报「本次未跑」；读到 {read:?}，判成了 {start:?}"
        );
    }

    /// 第 10x 批代码轮判决 K1-3（攻方 R1）：`request_device` 交回 wgpu 的设备显存不够是显存不够（卡池报本次未跑），
    /// 设备丢了这类别的错是起不来（判红）。错是照 wgpu 的错类型造的，不要显卡。
    #[test]
    fn a_device_request_refused_for_lack_of_memory_is_told_apart_from_other_refusals() {
        let refused = |device_error| {
            failure_of_a_device_request(
                &wgpu::RequestDeviceError::from(wgpu::wgc::instance::RequestDeviceError::Device(
                    device_error,
                )),
                "序号 0 的卡",
            )
        };
        assert!(
            matches!(
                refused(wgpu::wgc::device::DeviceError::OutOfMemory),
                CardStartFailure::OutOfGpuMemory { .. }
            ),
            "起设备时显存不够：报本次未跑那一种"
        );
        assert!(
            matches!(
                refused(wgpu::wgc::device::DeviceError::Lost),
                CardStartFailure::DeviceNotStarted { .. }
            ),
            "别的错：判红那一种"
        );
    }

    /// 第 10x 批代码轮判决 K1-3（攻方 R2）：wgpu 没接住的 Out of Memory（判器内核在另起的线程上编，那里没有错误范围）先记在设备上、
    /// 再照默认的做法 panic；校验错照样 panic、不记成显存不够。错是照 wgpu 的错类型造的，不要显卡。
    #[test]
    fn an_uncaptured_out_of_memory_is_recorded_on_the_device_before_the_panic() {
        let condition = DeviceConditionOfACard::default();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                handle_an_uncaptured_wgpu_error(
                    &condition,
                    wgpu::Error::OutOfMemory {
                        source: Box::new(std::fmt::Error),
                    },
                );
            }))
            .is_err(),
            "没接住的 Out of Memory 照默认的做法 panic"
        );
        assert_eq!(condition.out_of_memory_reports(), 1, "panic 之前记在设备上");
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                handle_an_uncaptured_wgpu_error(
                    &condition,
                    wgpu::Error::Validation {
                        source: Box::new(std::fmt::Error),
                        description: "绑定组布局对不上".to_string(),
                    },
                );
            }))
            .is_err(),
            "校验错照样 panic"
        );
        assert_eq!(condition.out_of_memory_reports(), 1, "校验错不记成显存不够");
    }

    /// 第 10z 批（第 10y 批代码轮判决 K1-1 攻方乙形、K3）：不归卡池的设备起之前就在它的总线上记下（起的那一段里卡池已经认得出），
    /// 卡池的设备不记。总线号是造的，不与本机的卡重。
    #[test]
    fn a_device_outside_the_card_pool_is_marked_on_its_bus_before_it_starts_and_a_pool_device_is_not(
    ) {
        assert_eq!(
            started_with_its_owner_recorded(
                DeviceOwnerOnTheCard::OutsideTheCardPool,
                "fe:00.0",
                || {
                    Ok(has_devices_started_outside_the_card_pool_on_the_bus(
                        "fe:00.0",
                    ))
                }
            ),
            Ok(true),
            "不归卡池：起的那一段里已经记着"
        );
        assert_eq!(
            started_with_its_owner_recorded(DeviceOwnerOnTheCard::TheCardPool, "fd:00.0", || {
                Ok(has_devices_started_outside_the_card_pool_on_the_bus(
                    "fd:00.0",
                ))
            }),
            Ok(false),
            "卡池的：不记"
        );
        assert!(!has_devices_started_outside_the_card_pool_on_the_bus(
            "fd:00.0"
        ));
    }

    /// 第 10z 批（第 10y 批代码轮攻方 K3：认出不走卡池设备的接线没有测试）：卡池读显存的那一份照这个进程起过不归卡池设备的总线认，
    /// 那张卡上这个进程占着的记成分不出；别的卡照都走账。报告是造的，总线号不与本机的卡重。
    #[test]
    fn the_card_pool_reads_this_process_as_also_outside_the_books_on_a_bus_with_a_device_outside_the_pool(
    ) {
        assert_eq!(
            started_with_its_owner_recorded(
                DeviceOwnerOnTheCard::OutsideTheCardPool,
                "fc:00.0",
                || { Ok(()) }
            ),
            Ok(())
        );
        let this_process = std::process::id();
        let report = ["00000000:FC:00.0", "00000000:FB:00.0"]
            .iter()
            .map(|bus| {
                format!(
                    "GPU {bus}\n    FB Memory Usage\n        Total                                          : 16311 MiB\n\
                     \x20       Reserved                                       : 462 MiB\n\
                     \x20       Free                                           : 10000 MiB\n    Processes\n\
                     \x20       Process ID                                     : {this_process}\n\
                     \x20           Used GPU Memory                            : 1000 MiB\n"
                )
            })
            .collect::<String>();
        let read = card_memory_reader_of_the_card_pool(
            Box::new(move || Ok(report.clone())),
            vec!["fc:00.0".to_string(), "fb:00.0".to_string()],
        );
        assert_eq!(
            read().map(|readings| readings
                .iter()
                .map(|reading| reading.occupied_by_this_process)
                .collect::<Vec<OccupiedByThisProcess>>()),
            Ok(vec![
                OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 1000 },
                OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 1000 },
            ])
        );
    }

    /// 第 10z 批第三块（第 10z 批代码轮正推 K3 第 3 条）：设备丢失的回调把这个设备记成丢了，连同 wgpu 给的原因与那一句（卡池照它重起）。不要卡。
    #[test]
    fn the_device_lost_callback_records_the_device_as_lost_with_the_reason_and_the_message() {
        let condition = DeviceConditionOfACard::default();
        record_the_loss_of_the_device(&condition, wgpu::DeviceLostReason::Unknown, "驱动重置");
        assert!(condition.is_lost());
        assert_eq!(
            condition.lost_reason(),
            Some("Unknown：驱动重置".to_string())
        );
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-4 攻方甲形，wgpu-core 30.0.1 `src/device/resource.rs` 的 `handle_hal_error`、wgpu-hal 30.0.1
    /// `src/lib.rs` 的 `DeviceError::OutOfMemory`（`#[error("Out of memory")]`））：wgpu 因显存不够丢设备时给的那一句认成显存不够，别的认成别的原因；回调照它记，之后一段因设备丢了而失败的活、
    /// 不在同一组而这段活开始之后一条 Out of Memory 都没记过，也说成显存不够。不要卡。
    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K3，攻方 F5）：期望的那一句写死成 wgpu-hal 30.0.1 `DeviceError::OutOfMemory` 的
    /// `#[error("Out of memory")]`，不与产品代码取自同一个 `to_string()`：wgpu 改了这一句，这里先红。
    #[test]
    fn a_device_lost_for_lack_of_memory_explains_a_failure_on_it_as_gpu_memory_short() {
        let out_of_memory = "Out of memory";
        assert_eq!(
            cause_of_the_device_loss(out_of_memory),
            CauseOfTheDeviceLoss::GpuMemoryShort
        );
        for other in [
            wgpu::hal::DeviceError::Lost.to_string(),
            wgpu::hal::DeviceError::Unexpected.to_string(),
            "驱动重置".to_string(),
        ] {
            assert_eq!(
                cause_of_the_device_loss(&other),
                CauseOfTheDeviceLoss::Other,
                "{other}"
            );
        }
        let condition = DeviceConditionOfACard::default();
        let work = singlefs_harness::gpu_memory_ledger::WorkOnTheCardOfThisThread::begin();
        let reports_before = condition.out_of_memory_reports();
        record_the_loss_of_the_device(&condition, wgpu::DeviceLostReason::Unknown, out_of_memory);
        assert!(
            condition
                .out_of_memory_explaining_a_failure_of_the_work(
                    work.identifier(),
                    reports_before,
                    condition.device_when_the_work_returns()
                )
                .is_some(),
            "设备因显存不够丢了：说成显存不够"
        );
    }

    /// 第 10z 批第三块（第 10z 批代码轮正推 K3 第 1 条）：不走卡池的两个取卡入口起设备都照不归卡池起（起之前记下总线号），卡池起设备只有一处、
    /// 照归卡池起。扫这份源码（测试那一段之外），不要卡。
    ///
    /// 拦不住的写法，一种一条（第 10z 批第二十五块写明）：
    /// - 两个入口各取第一处：前面放一份挂 `#[cfg(any())]` 的干净副本、后面写真生效的另一份（照不归卡池起的那一句换成先 `let` 一个
    ///   `DeviceOwnerOnTheCard` 再交进去），读到的是编译时去掉的那一份；与正推 K4-1 G14、G16 同一类，第二十五块只把借据丢掉时那一步、
    ///   四个借卡入口与卡池模块改成按词恰好一处、不挂 `#[cfg]`（[`index_of_the_one_pinned_item_without_a_conditional_attribute`]），这两个入口没改。
    #[test]
    fn the_entries_outside_the_card_pool_start_their_devices_as_outside_the_card_pool() {
        let source = include_str!("gpu_unit_checks.rs");
        let code = code_before_the_tests_module(source);
        let body_of = |signature: &str| {
            let start = code
                .find(signature)
                .unwrap_or_else(|| panic!("源码里有 {signature}"));
            let after = &code[start + signature.len()..];
            &after[..after
                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
                .expect("函数体以顶格的右括号收尾")]
        };
        let outside_the_pool = concat!(
            "start_the_card(discovered, DeviceOwnerOnTheCard::",
            "OutsideTheCardPool)"
        );
        for entry in ["pub fn gpu_cards_by_priority(", "pub fn usable_gpu_cards("] {
            assert_eq!(
                body_of(entry).matches(outside_the_pool).count(),
                1,
                "{entry} 起设备照不归卡池起"
            );
        }
        assert_eq!(
            code.matches(concat!("DeviceOwnerOnTheCard::", "TheCardPool)"))
                .count(),
            1,
            "照归卡池起设备的只有卡池那一处"
        );
    }

    /// 第 10z 批第六块（G2 乙）：借据丢掉时那一次等，队列空了或等到了才算等完、照常销账；超时、wgpu 报错、等的时候 panic、`Wait` 交回
    /// `Poll`（那是不等的结局）都算没等成，这一笔照旧记在账上。不要卡。
    #[test]
    fn a_lease_is_returned_only_when_the_wait_for_the_device_finished() {
        let finished = DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished;
        let failed = DeviceWaitBeforeReturningTheLease::TheWaitFailed;
        assert_eq!(
            device_wait_before_returning_a_lease(&Ok(Ok(wgpu::PollStatus::QueueEmpty))),
            finished
        );
        assert_eq!(
            device_wait_before_returning_a_lease(&Ok(Ok(wgpu::PollStatus::WaitSucceeded))),
            finished
        );
        assert_eq!(
            device_wait_before_returning_a_lease(&Ok(Ok(wgpu::PollStatus::Poll))),
            failed,
            "Wait 交回 Poll：没说等完"
        );
        assert_eq!(
            device_wait_before_returning_a_lease(&Ok(Err(wgpu::PollError::Timeout))),
            failed,
            "超时"
        );
        assert_eq!(
            device_wait_before_returning_a_lease(&Err(Box::new("等的时候 wgpu panic 了"))),
            failed,
            "等的时候 panic"
        );
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K4，攻方 H6：超时 30 秒改成 0 或不设都照绿）：借据丢掉时那一次等的是到此刻为止的全部提交，
    /// 最多等 E174 跑前登记的 30 秒（`research/prompts/e174-preregistration.md` 第五节「等待」那一臂）。不要卡。
    #[test]
    fn a_lease_waits_for_every_submission_so_far_for_at_most_the_thirty_seconds_registered_by_e174()
    {
        match poll_type_before_returning_a_lease() {
            wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(timeout),
            } => assert_eq!(
                timeout,
                std::time::Duration::from_secs(30),
                "最多等 E174 登记的 30 秒：不设会让丢借据的线程卡在出了事的设备上，0 等于不等"
            ),
            wgpu::PollType::Wait {
                submission_index: Some(_),
                ..
            }
            | wgpu::PollType::Wait { timeout: None, .. }
            | wgpu::PollType::Poll => {
                panic!(
                    "借据丢掉时等到此刻为止的全部提交、有上限：{:?}",
                    poll_type_before_returning_a_lease()
                )
            }
        }
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K1-2 的出路）：卡池借之前对有没等成借据的那张卡 poll 一次，交回队列空了才算那台设备上
    /// 提交的活都做完了（销那几笔）；还有在飞的、`Wait` 才交的那一种、报错、panic 都不算。不要卡。
    #[test]
    fn a_poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty() {
        assert!(device_found_idle_by_a_poll(&Ok(Ok(
            wgpu::PollStatus::QueueEmpty
        ))));
        assert!(
            !device_found_idle_by_a_poll(&Ok(Ok(wgpu::PollStatus::Poll))),
            "还有在飞的提交"
        );
        assert!(
            !device_found_idle_by_a_poll(&Ok(Ok(wgpu::PollStatus::WaitSucceeded))),
            "Poll 不交这一种，交了也说不清全部提交做完了"
        );
        assert!(
            !device_found_idle_by_a_poll(&Ok(Err(wgpu::PollError::Timeout))),
            "报错"
        );
        assert!(
            !device_found_idle_by_a_poll(&Err(Box::new("poll 的时候 wgpu panic 了"))),
            "panic"
        );
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K4，攻方 H6、正推第 4 点：`GpuCardLease::drop` 等不等、记没记结局没有不要卡的测试）：
    /// 借据丢掉时那一步整个交给 harness 档的 `wait_for_the_device_and_take_in_the_outcome`（等、接住 panic、记结局、展开中不再抛都在那边测），
    /// 等的是 [`poll_type_before_returning_a_lease`]、结局照 [`device_wait_before_returning_a_lease`] 判、记进这一笔借据；卡池的四个入口借之前
    /// 都先 poll 没等成的那几张卡（K1-2 的出路）。扫这份源码（测试那一段之外），不要卡。
    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K4，正推 K4-1 G14、G16）：借据丢掉时那一步（按词 `Drop for GpuCardLease`）
    /// 与四个入口（按词 `fn 名字`）各恰好一处、前面不挂 `#[cfg]`、`#[cfg_attr]`，挂 `#[cfg(any())]` 的干净副本在前、坏版本在后判红
    /// （`the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module`）。
    ///
    /// 拦不住的写法，一种一条：
    /// - 坏版本的借据丢掉时那一步把类型写成路径或别名（`impl Drop for self::GpuCardLease`、`type 别名 = GpuCardLease;`），干净的那一份
    ///   不靠挂在它上面的属性去掉（包进一个展开成空的宏调用、放在函数体里）：按词只数得到干净的那一份；
    /// - 宏（过程宏、拼名字的宏）展开出来的同名方法或 `impl Drop`，干净的那一份照上一条去掉。
    #[test]
    fn the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first() {
        assert_the_lease_return_and_every_booking_entry_are_pinned(include_str!(
            "gpu_unit_checks.rs"
        ));
    }

    /// `source`（这份源文件，或用例在它上面改过的一份）测试那一段之外，借据丢掉时那一步与卡池的四个借卡入口照钉住的那样写
    /// （判法在 `the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first`）。
    ///
    /// # Panics
    /// 有一样没照钉住的那样写。
    fn assert_the_lease_return_and_every_booking_entry_are_pinned(source: &str) {
        let code = code_before_the_tests_module(source);
        let tokens_with_their_offsets = rust_tokens_with_their_offsets(code);
        let tokens: Vec<&str> = tokens_with_their_offsets
            .iter()
            .map(|(_offset, token)| *token)
            .collect();
        let body_until = |signature: &[&str], closing_line: &str| -> String {
            let signature_index =
                index_of_the_one_pinned_item_without_a_conditional_attribute(&tokens, signature);
            let (last_offset, last_token) =
                tokens_with_their_offsets[signature_index + signature.len() - 1];
            let after = &code[last_offset + last_token.len()..];
            after[..after
                .find(closing_line)
                .unwrap_or_else(|| panic!("{signature:?} 收得了尾"))]
                .to_string()
        };
        let drop_body = body_until(&["Drop", "for", "GpuCardLease"], "\n        \u{7d}\n");
        for expected in [
            "wait_for_the_device_and_take_in_the_outcome(",
            "&mut self.booking,",
            "&self.device_on_the_books,",
            "|| device.poll(poll_type_before_returning_a_lease()),",
            "device_wait_before_returning_a_lease,",
        ] {
            assert_eq!(
                drop_body.matches(expected).count(),
                1,
                "借据丢掉时那一步要有一处 {expected}：{drop_body}"
            );
        }
        for entry in [
            "lease_card",
            "lease_the_first_card_with_room_now",
            "book_the_scratch_of_the_cards_with_room_now",
            "card_rooms_now_and_when_empty_of",
        ] {
            // 第 10z 批第十一块（第 10z 批第七八九块代码轮判决 K5，正推六·二：那一句注释掉、包进 `if false` 照绿）：注释行不算，
            // 那一句要整行、缩进十六格（方法体顶层的语句），包进别的块里的缩进更深、同一行里另有别的不是整行
            let body = body_until(&["fn", entry], "\n            \u{7d}\n");
            let code_lines: Vec<&str> = body
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect();
            let polled = code_lines
                .iter()
                .position(|line| *line == "                self.take_in_the_devices_found_idle();")
                .unwrap_or_else(|| {
                    panic!("{entry} 借之前先 poll 没等成的那几张卡（方法体顶层整行一句）：{code_lines:#?}")
                });
            let booked = code_lines
                .iter()
                .position(|line| line.contains("ledger"))
                .unwrap_or_else(|| panic!("{entry} 照账借：{code_lines:#?}"));
            assert!(
                polled < booked,
                "{entry} 先 poll、再照账借：{code_lines:#?}"
            );
        }
    }

    /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K4-2、K5：借之前那一步里面没有用例，丢了的设备到借到之后的重起才告诉账）：
    /// 借之前那一步整个交给 harness 档的 `GpuMemoryLedger::take_in_the_devices_kept_before_booking`（看池里每张卡、丢了的告诉账不 poll、
    /// 列出来的 poll、只销列出来的，都在那边不要卡地测，`singlefs_harness::gpu_memory_ledger` 的库单测），卡池这一侧只接四样：那一格留着的设备
    /// （不等正在起的）、丢没丢照设备的情况、poll 一次不等且照 [`device_found_idle_by_a_poll`] 判、账上的那一台。这四样要真卡上起好的设备才走得到，
    /// 换不成不要卡的行为用例，这里扫这份源码（测试那一段之外、去掉注释行与空白）：方法体整个等于钉住的那一句。不要卡。
    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4）：原先只数几段文本各出现一处、闭包三个，被这几种改了行为而编得过的写法骗过：
    /// 方法体最前面 `if true { return; }`（正推 A2）、丢没丢与 poll 两个同型闭包对调（正推 A3）、整个包进 `if false { … }`（攻方 gpu-2）、
    /// 方法体里另起一个同名的 `device_found_idle_by_a_poll` 恒交真遮住外面那个（攻方 gpu-3）；整句比之后这四种都红。
    /// 整句比管不到（文本都在这个方法体之外）、由别的用例管的：
    /// - 改 [`device_found_idle_by_a_poll`] 本身：`a_poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty`；
    /// - 卡池模块里另起模块（攻方 `mod wgpu`、`mod std`）、经 `use` 改名或通配、在条目这一层展开宏，遮住方法体里的 `std`、`wgpu`；去掉 `use`
    ///   另起一个同名的 `device_found_idle_by_a_poll`（第 10z 批第十八块）：
    ///   `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call`，它也只看文本；
    /// - `GpuCard::device`、`GpuCard::device_condition` 交的是哪一台、哪一份（第 10z 批第十八块，正推 K4）：
    ///   `a_card_hands_out_its_own_device_and_its_own_device_condition`，它也只看文本；
    /// - 四个入口不调这一步：`the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first`，它也只看文本。
    ///
    /// 拦不住的写法，一种一条：
    /// - `device_on_the_books` 放进去的是哪一台（`started_card`）；
    /// - `GpuCard` 的 `device`、`device_condition` 两格里放的是哪一台、哪一份（`start_the_card` 起设备时放进去）；
    /// - harness 档的 `DeviceStartedOnDemand::started_now` 与 `DeviceConditionOfACard::is_lost` 改了；
    /// - 在方法上面加 `#[cfg]` 让另一份同名方法编进来。
    #[test]
    fn the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking() {
        let source = include_str!("gpu_unit_checks.rs");
        let code = code_before_the_tests_module(source);
        let signature = "fn take_in_the_devices_found_idle(&self) \u{7b}";
        assert_eq!(
            code.matches(signature).count(),
            1,
            "源码里恰好一处 {signature}"
        );
        let start = code
            .find(signature)
            .unwrap_or_else(|| panic!("源码里有 {signature}"));
        let after = &code[start + signature.len()..];
        let body = &after[..after
            .find("\n            \u{7d}\n")
            .expect("借之前那一步以缩进十二格的右括号收尾")];
        let body_without_comments_and_spaces: String = body
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .flat_map(str::chars)
            .filter(|character| !character.is_whitespace())
            .collect();
        let expected_body = concat!(
            "self.ledger.take_in_the_devices_kept_before_booking(",
            "|card_position|self.cards[card_position].started.started_now(),",
            "|started:&GpuCard|started.device_condition().is_lost(),",
            "|started:&GpuCard|\u{7b}device_found_idle_by_a_poll(&std::panic::catch_unwind(",
            "std::panic::AssertUnwindSafe(||\u{7b}started.device().poll(wgpu::PollType::Poll)\u{7d}),))\u{7d},",
            "|started:&GpuCard|\u{7b}started.device_on_the_books.as_deref().expect(",
            "\"卡池起的设备起好时放进了账上的那一台（started_card）\")\u{7d},",
            ");"
        );
        assert_eq!(
            body_without_comments_and_spaces, expected_body,
            "借之前那一步整个交给账、四样各接一处，次序是那一格留着的设备、丢没丢、poll、账上的那一台，此外什么都没有"
        );
    }

    /// `text` 以 `"` 起头时，那一个字符串字面量连两头引号的长度（反斜杠连同它转义的那个字跳过）。
    fn length_of_the_quoted_literal_at_the_start(text: &str) -> usize {
        let bytes = text.as_bytes();
        let mut position = 1;
        loop {
            let byte = *bytes.get(position).expect("字符串字面量有收尾的引号");
            if byte == b'"' {
                return position + 1;
            }
            position += if byte == b'\\' { 2 } else { 1 };
        }
    }

    /// `text` 以 `'` 起头时，那一个字符字面量连两头引号的长度；是生命周期（`'static`、`'_`）交 `None`。
    fn length_of_the_character_literal_at_the_start(text: &str) -> Option<usize> {
        let after_the_quote = &text[1..];
        if after_the_quote.starts_with('\\') {
            // 转义的（`'\''`、`'\n'`、`'\u{7b}'`）：收尾的引号在反斜杠与它后面那个字之后
            let closing_quote = 3 + text[3..].find('\'').expect("转义的字符字面量有收尾的引号");
            return Some(closing_quote + 1);
        }
        let first_character = after_the_quote.chars().next().expect("引号后面还有字");
        after_the_quote[first_character.len_utf8()..]
            .starts_with('\'')
            .then(|| first_character.len_utf8() + 2)
    }

    /// `text` 是原始字符串去掉 `r`、`br`、`cr` 之后那一截（以若干个 `#` 与一个 `"` 起头）时，到收尾的 `"` 与同样多个 `#` 为止的长度。
    fn length_of_the_raw_string_at_the_start(text: &str) -> usize {
        let hashes = text.len() - text.trim_start_matches('#').len();
        let closing = format!("\"{}", "#".repeat(hashes));
        let body_start = hashes + 1;
        body_start + text[body_start..].find(&closing).expect("原始字符串有收尾") + closing.len()
    }

    /// `text` 以 `/*` 起头时，那一段块注释（可以嵌套）的长度。
    fn length_of_the_block_comment_at_the_start(text: &str) -> usize {
        let mut depth = 0_usize;
        let mut position = 0;
        loop {
            let rest = &text[position..];
            if rest.starts_with("/*") {
                depth += 1;
                position += 2;
            } else if rest.starts_with("*/") {
                depth -= 1;
                position += 2;
                if depth == 0 {
                    return position;
                }
            } else {
                position += rest.chars().next().expect("块注释有收尾").len_utf8();
            }
        }
    }

    /// 拼成关键字的原始标识符（`r#mod`、`r#use`）是名字，不是关键字：分词时这几个连着 `r#` 出词，免得被当成关键字（第 10z 批第二十三块，
    /// 正推 K4-2 L14）；别的原始标识符（`r#std`）与不带 `r#` 的同名，取它后面的名字。
    const KEYWORDS_A_RAW_IDENTIFIER_CAN_SPELL: [&str; 48] = [
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do",
        "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in",
        "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
        "return", "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe",
        "unsized", "use", "virtual", "where", "while", "yield",
    ];

    /// `code` 拆成的词：标识符与关键字（`r#` 原始标识符取它后面的名字，拼成关键字的连着 `r#`）、生命周期（连着它前面的 `'`）、
    /// 数字（照标识符那样收一段）、`::`、`!=`，其余标点一个字一个词；注释（行注释、可以嵌套的块注释）、字符串与字符字面量
    /// （普通的、原始的 `r#"…"#`、`br`、`cr`，字节的、C 的）不出词。认原始字符串的写法与崩溃放量那一份数花括号
    /// （`crash_amplification.rs` 的 `lines_read_past_comments_and_literals`）相同：`r`、`br`、`cr` 后面紧跟若干个 `#` 与一个 `"`。
    /// 扫源码的用例照它找条目、`use` 引进来的名字与花括号的层数：字面量与注释里的 `mod`、`use`、花括号都不算。
    /// 非 ASCII 的字只落在注释、字面量与标识符里，切出来的词因此总在字符边界上。
    fn rust_tokens_without_comments_and_literals(code: &str) -> Vec<&str> {
        rust_tokens_with_their_offsets(code)
            .into_iter()
            .map(|(_offset, token)| token)
            .collect()
    }

    /// [`rust_tokens_without_comments_and_literals`] 的词，各连同它在 `code` 里起头的字节位置。
    fn rust_tokens_with_their_offsets(code: &str) -> Vec<(usize, &str)> {
        let is_identifier_character =
            |character: char| character.is_alphanumeric() || character == '_';
        let mut tokens = Vec::new();
        let mut position = 0;
        while position < code.len() {
            let rest = &code[position..];
            let first_character = rest.chars().next().expect("没到结尾就还有字");
            if first_character.is_whitespace() {
                position += first_character.len_utf8();
            } else if rest.starts_with("//") {
                position += rest.find('\n').unwrap_or(rest.len());
            } else if rest.starts_with("/*") {
                position += length_of_the_block_comment_at_the_start(rest);
            } else if first_character == '"' {
                position += length_of_the_quoted_literal_at_the_start(rest);
            } else if first_character == '\'' {
                match length_of_the_character_literal_at_the_start(rest) {
                    Some(literal_length) => position += literal_length,
                    None => {
                        let lifetime_length = 1 + rest[1..]
                            .find(|character: char| !is_identifier_character(character))
                            .unwrap_or(rest.len() - 1);
                        tokens.push((position, &rest[..lifetime_length]));
                        position += lifetime_length;
                    }
                }
            } else if is_identifier_character(first_character) {
                let word_length = rest
                    .find(|character: char| !is_identifier_character(character))
                    .unwrap_or(rest.len());
                let word = &rest[..word_length];
                let after_the_word = &rest[word_length..];
                if matches!(word, "r" | "br" | "cr")
                    && after_the_word.trim_start_matches('#').starts_with('"')
                {
                    position += word_length + length_of_the_raw_string_at_the_start(after_the_word);
                } else if word == "r" && after_the_word.starts_with('#') {
                    // 原始标识符 `r#名字`：拼成关键字的连着 `r#` 出词（`r#mod` 是名字，不是关键字），别的名字下一圈照标识符收
                    let name = &after_the_word[1..];
                    let name_length = name
                        .find(|character: char| !is_identifier_character(character))
                        .unwrap_or(name.len());
                    if KEYWORDS_A_RAW_IDENTIFIER_CAN_SPELL.contains(&&name[..name_length]) {
                        tokens.push((position, &rest[..word_length + 1 + name_length]));
                        position += word_length + 1 + name_length;
                    } else {
                        position += word_length + 1;
                    }
                } else if matches!(word, "b" | "c") && after_the_word.starts_with('"') {
                    position +=
                        word_length + length_of_the_quoted_literal_at_the_start(after_the_word);
                } else if word == "b" && after_the_word.starts_with('\'') {
                    position += word_length
                        + length_of_the_character_literal_at_the_start(after_the_word)
                            .expect("b 后面紧跟引号的是字节字面量");
                } else {
                    tokens.push((position, word));
                    position += word_length;
                }
            } else if rest.starts_with("::") {
                tokens.push((position, "::"));
                position += 2;
            } else if rest.starts_with("!=") {
                // 不等号整个一个词：拆成 `!`、`=` 时，条目层 `const 名: bool = 甲 != 乙;` 的 `甲 !` 会被认成调宏
                tokens.push((position, "!="));
                position += 2;
            } else {
                tokens.push((position, &rest[..first_character.len_utf8()]));
                position += first_character.len_utf8();
            }
        }
        tokens
    }

    /// `source`（这份源文件，或用例在它上面改过的一份）测试模块头两行之前的那一段。头两行只认这一种，与崩溃放量那一份
    /// （`crash_amplification.rs` 的 `text_outside_the_tests_module`）同一口径、按行逐字：
    /// - 相邻两整行逐字是 [`TESTS_MODULE_OPENING`] 那两行（`#[cfg(test)]`，与 `mod tests`、一个空格、一个左花括号；行首起、行尾止，不带别的空白与注释，换行是 `\n`），在代码里（注释与字面量之外）；
    /// - 在花括号的第 0 层，也在圆括号、方括号的第 0 层；
    /// - 前一个词（空白、注释、字面量都跳过）是分号、右花括号、右方括号或文件开头。
    ///
    /// 照原文找第一处时，注释或字符串里写一份头两行就把切口挪到那里，它与真测试模块之间的代码哪一条扫源码的用例都看不见（第 10z 批第二十三块，
    /// 攻方 K4-2）；拿一个吃掉 `#[cfg(test)]` 的宏把测试模块包进一次宏调用的圆括号里，那个「测试模块」在非测试构建里照编，头两行却仍在
    /// 花括号第 0 层（第 10z 批第二十五块，正推 K4-1 S14），宏调用的括号里头两行前一个词是 `(`，前面先放一条以分号收尾的条目也还在
    /// 圆括号里。卡池这一份原先按词认头两行，第二十五块对齐成按行逐字：这两份文件是 rustfmt 出的，头两行就是这一种写法，
    /// 别的写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释、CRLF 换行）一律判红，是故意拒。
    ///
    /// # Panics
    /// 照上面那样认的头两行不是恰好一处。
    fn code_before_the_tests_module(source: &str) -> &str {
        let tokens = rust_tokens_with_their_offsets(source);
        let mut brace_depth = 0_usize;
        let mut parenthesis_and_bracket_depth = 0_usize;
        let mut openings = Vec::new();
        for (index, (offset, token)) in tokens.iter().enumerate() {
            let token_before_the_attribute = index
                .checked_sub(1)
                .map(|index_before| tokens[index_before].1);
            if *token == "#"
                && is_the_tests_module_opening_as_two_whole_lines_at(source, *offset)
                && brace_depth == 0
                && parenthesis_and_bracket_depth == 0
                && token_before_the_attribute
                    .is_none_or(|token_before| matches!(token_before, ";" | "\u{7d}" | "]"))
            {
                openings.push(*offset);
            }
            if *token == "\u{7b}" {
                brace_depth += 1;
            } else if *token == "\u{7d}" {
                brace_depth = brace_depth.checked_sub(1).expect("右花括号不多于左花括号");
            } else if matches!(*token, "(" | "[") {
                parenthesis_and_bracket_depth += 1;
            } else if matches!(*token, ")" | "]") {
                parenthesis_and_bracket_depth = parenthesis_and_bracket_depth
                    .checked_sub(1)
                    .expect("右圆括号、右方括号不多于左的");
            }
        }
        assert_eq!(
            openings.len(),
            1,
            "代码里测试模块的头两行恰好一处（两整行逐字、括号第 0 层、前一个词是条目收尾；注释、字符串、宏调用括号里的不算），读到的字节位置：{openings:?}"
        );
        &source[..openings[0]]
    }

    /// `source` 从 `offset` 起是不是测试模块头两行那两整行：`offset` 在行首（文件开头或紧跟 `\n`），往后逐字是 [`TESTS_MODULE_OPENING`]，
    /// 再往后是 `\n` 或文件结尾。
    fn is_the_tests_module_opening_as_two_whole_lines_at(source: &str, offset: usize) -> bool {
        let starts_a_line = offset == 0 || source[..offset].ends_with('\n');
        starts_a_line
            && source[offset..]
                .strip_prefix(TESTS_MODULE_OPENING)
                .is_some_and(|after_the_opening| {
                    after_the_opening.is_empty() || after_the_opening.starts_with('\n')
                })
    }

    /// 这份源码（测试那一段之外）里卡池模块（`mod card_pool`）的词，各带它在模块里第几层花括号里：从模块的左花括号之后
    /// 到与它配对的右花括号之前，模块的条目在第 0 层（左花括号算它外面那一层，右花括号算它收回到的那一层）。
    /// 卡池模块按词恰好一处、不挂 `#[cfg]`（[`index_of_the_one_pinned_item_without_a_conditional_attribute`]）：取第一处时，前面放一份挂 `#[cfg(any())]`、
    /// 只有那四条 `use` 的干净卡池模块，扫的就是它，真生效的那一份里另起 `mod wgpu` 也看不见（第 10z 批第二十五块，与正推 K4-1 G14 同形）。
    fn card_pool_module_tokens_with_their_brace_depth(code: &str) -> Vec<(usize, &str)> {
        let tokens = rust_tokens_without_comments_and_literals(code);
        let module_keyword = index_of_the_one_pinned_item_without_a_conditional_attribute(
            &tokens,
            &["mod", "card_pool"],
        );
        assert_eq!(
            tokens.get(module_keyword + 2),
            Some(&"\u{7b}"),
            "卡池模块写在这一段里（mod card_pool 后面紧跟左花括号，不是 mod card_pool; 另起文件）"
        );
        let opening = module_keyword + 3;
        let mut depth = 0_usize;
        let mut module_tokens = Vec::new();
        for token in &tokens[opening..] {
            if *token == "\u{7b}" {
                module_tokens.push((depth, *token));
                depth += 1;
            } else if *token == "\u{7d}" {
                let Some(depth_after_the_closing) = depth.checked_sub(1) else {
                    return module_tokens;
                };
                depth = depth_after_the_closing;
                module_tokens.push((depth, *token));
            } else {
                module_tokens.push((depth, *token));
            }
        }
        panic!("卡池模块有与它的左花括号配对的右花括号")
    }

    /// 一条 `use` 引进来的一样东西。
    enum BroughtInByAUse {
        /// 引进一个名字：`source` 是被引的那一处的全路径（`甲::乙`；花括号里的 `self` 取花括号之前那一段），
        /// `name` 是它在这里叫什么（有 `as` 的取改成的那个名字）。
        Name { source: String, name: String },
        /// 通配（`甲::乙::*`）：引进来哪些名字，扫这一个模块的源码看不见。
        Glob { source: String },
    }

    /// `outer_path` 后面接上 `segments`，用 `::` 连起来（`outer_path` 空着就只是 `segments`）。
    fn path_joined(outer_path: &str, segments: &[&str]) -> String {
        outer_path
            .split("::")
            .filter(|segment| !segment.is_empty())
            .chain(segments.iter().copied())
            .collect::<Vec<&str>>()
            .join("::")
    }

    /// `statement`（一条 `use` 从 `use` 那个词到分号的词）从 `start` 起的一棵 use 树（`甲::乙`、`甲::乙 as 丙`、`甲::self`、`甲::*`、
    /// `甲::{…}`，花括号可以套几层）引进来的东西，路径前面接着外层花括号之前那一段 `outer_path`；连同这棵树收尾之后那个词的位置
    /// （逗号、右花括号或分号）。
    fn brought_in_by_a_use_tree(
        statement: &[&str],
        start: usize,
        outer_path: &str,
    ) -> (Vec<BroughtInByAUse>, usize) {
        let mut segments: Vec<&str> = Vec::new();
        let mut position = start;
        loop {
            let token = *statement.get(position).expect("use 树在分号之前收尾");
            if token == "::" {
                position += 1;
            } else if token == "*" {
                let glob = BroughtInByAUse::Glob {
                    source: path_joined(outer_path, &segments),
                };
                return (vec![glob], position + 1);
            } else if token == "\u{7b}" {
                let group_path = path_joined(outer_path, &segments);
                let mut brought_in = Vec::new();
                position += 1;
                while statement[position] != "\u{7d}" {
                    let (brought_in_by_the_inner_tree, after_the_inner_tree) =
                        brought_in_by_a_use_tree(statement, position, &group_path);
                    brought_in.extend(brought_in_by_the_inner_tree);
                    position =
                        after_the_inner_tree + usize::from(statement[after_the_inner_tree] == ",");
                }
                return (brought_in, position + 1);
            } else {
                segments.push(token);
                position += 1;
                if statement.get(position) == Some(&"::") {
                    continue;
                }
                let is_self = token == "self";
                let source = path_joined(
                    outer_path,
                    &segments[..segments.len() - usize::from(is_self)],
                );
                let name_without_a_rename = if is_self {
                    source
                        .rsplit("::")
                        .next()
                        .expect("rsplit 至少交一段")
                        .to_string()
                } else {
                    token.to_string()
                };
                let (name, after_the_tree) = if statement.get(position) == Some(&"as") {
                    (statement[position + 1].to_string(), position + 2)
                } else {
                    (name_without_a_rename, position)
                };
                return (vec![BroughtInByAUse::Name { source, name }], after_the_tree);
            }
        }
    }

    /// 钉住那两段（借之前那一步、借据丢掉时那一步）按名字调的函数，各连同它该从哪一处 `use` 进卡池模块。
    const FUNCTIONS_THE_PINNED_BODIES_CALL_BY_NAME: [(&str, &str); 4] = [
        ("super::super", "device_found_idle_by_a_poll"),
        ("super::super", "poll_type_before_returning_a_lease"),
        ("super::super", "device_wait_before_returning_a_lease"),
        (
            "singlefs_harness::gpu_memory_ledger",
            "wait_for_the_device_and_take_in_the_outcome",
        ),
    ];

    /// 钉住那两段（借之前那一步、借据丢掉时那一步）的签名，按词。
    const SIGNATURES_OF_THE_PINNED_BODIES: [&[&str]; 2] = [
        &["fn", "take_in_the_devices_found_idle"],
        &["impl", "Drop", "for", "GpuCardLease"],
    ];

    /// 卡池模块的词（`module_tokens`，各带花括号层数）里第 `index` 个词所在的作用域罩不罩得到钉住那两段：在模块这一层（第 0 层）的罩得到；
    /// 在花括号里的（函数体、块）只罩它那一对花括号里，那一对花括号里有钉住那两段的签名才罩得到。
    fn does_the_scope_of_the_token_reach_the_pinned_bodies(
        module_tokens: &[(usize, &str)],
        index: usize,
    ) -> bool {
        let Some(enclosing_depth) = module_tokens[index].0.checked_sub(1) else {
            return true;
        };
        let opening = module_tokens[..index]
            .iter()
            .rposition(|(depth, token)| *depth == enclosing_depth && *token == "\u{7b}")
            .expect("第 1 层往里的词前面有开这一层的左花括号");
        let closing = index
            + module_tokens[index..]
                .iter()
                .position(|(depth, token)| *depth == enclosing_depth && *token == "\u{7d}")
                .expect("第 1 层往里的词后面有收这一层的右花括号");
        let block: Vec<&str> = module_tokens[opening..=closing]
            .iter()
            .map(|(_depth, token)| *token)
            .collect();
        SIGNATURES_OF_THE_PINNED_BODIES.iter().any(|signature| {
            block
                .windows(signature.len())
                .any(|window| window == *signature)
        })
    }

    /// `tokens` 里第 `index` 个词（一个条目的头一个词）前面挂着的属性，各取 `#[` 之后的头一个词（`cfg`、`cfg_attr`、`allow`……），
    /// 照源码倒着；属性与条目之间的 `pub`、`pub(…)` 跳过。
    fn names_of_the_attributes_right_before<'token>(
        tokens: &[&'token str],
        index: usize,
    ) -> Vec<&'token str> {
        let mut item_start = index;
        if item_start > 0 && tokens[item_start - 1] == "pub" {
            item_start -= 1;
        } else if item_start > 1 && tokens[item_start - 1] == ")" {
            let visibility_opening = tokens[..item_start - 1]
                .iter()
                .rposition(|token| *token == "(");
            if let Some(parenthesis) = visibility_opening.filter(|parenthesis| {
                parenthesis.checked_sub(1).map(|before| tokens[before]) == Some("pub")
            }) {
                item_start = parenthesis - 1;
            }
        }
        let mut names = Vec::new();
        while item_start > 0 && tokens[item_start - 1] == "]" {
            let mut unclosed_brackets = 0_usize;
            let mut attribute_opening = None;
            for token_index in (0..item_start).rev() {
                if tokens[token_index] == "]" {
                    unclosed_brackets += 1;
                } else if tokens[token_index] == "[" {
                    unclosed_brackets -= 1;
                    if unclosed_brackets == 0 {
                        attribute_opening = Some(token_index);
                        break;
                    }
                }
            }
            let Some(bracket) = attribute_opening
                .filter(|bracket| bracket.checked_sub(1).map(|before| tokens[before]) == Some("#"))
            else {
                break;
            };
            names.extend(tokens.get(bracket + 1).copied());
            item_start = bracket - 1;
        }
        names
    }

    /// `tokens`（一段代码的词）里钉住的那一个条目，交 `signature`（按词：`["fn", 名字]`、`["mod", 名字]`、`["Drop", "for", 类型]`）
    /// 在 `tokens` 里那一处的下标。按词恰好一处，那个条目（往前到最近的 `impl`、`fn`、`mod`，再往前跳过 `const`、`async`、`unsafe`、
    /// `extern`、`safe`）前面不挂 `#[cfg]`、`#[cfg_attr]`：取第一处时，前面先放一份挂 `#[cfg(any())]` 的干净副本、后面写真生效的坏版本，
    /// 读到的是编译时去掉的那一份（第 10z 批第二十五块，正推 K4-1 G14、G16）；只有一份而挂着 `#[cfg]` 的，编译时整个去掉。
    /// 与第二十三块 `use` 上挂 `#[cfg]` 判红是同一个判法（[`names_of_the_attributes_right_before`]）。
    ///
    /// # Panics
    /// 按词不是恰好一处，或那一处的条目挂着 `#[cfg]`、`#[cfg_attr]`。
    fn index_of_the_one_pinned_item_without_a_conditional_attribute(
        tokens: &[&str],
        signature: &[&str],
    ) -> usize {
        let places: Vec<usize> = tokens
            .windows(signature.len())
            .enumerate()
            .filter(|(_index, window)| *window == signature)
            .map(|(index, _window)| index)
            .collect();
        assert_eq!(
            places.len(),
            1,
            "钉住的 {signature:?} 按词恰好一处（挂 #[cfg(any())] 的干净副本在前、真生效的在后，取第一处就读到副本），读到的词下标：{places:?}"
        );
        let item_keyword = tokens[..=places[0]]
            .iter()
            .rposition(|token| matches!(*token, "impl" | "fn" | "mod"))
            .expect("钉住的条目由 impl、fn 或 mod 起头");
        let qualifiers = tokens[..item_keyword]
            .iter()
            .rev()
            .take_while(|token| matches!(**token, "const" | "async" | "unsafe" | "extern" | "safe"))
            .count();
        let attributes = names_of_the_attributes_right_before(tokens, item_keyword - qualifiers);
        assert!(
            !attributes
                .iter()
                .any(|attribute| matches!(*attribute, "cfg" | "cfg_attr")),
            "钉住的 {signature:?} 前面挂着 {attributes:?}：#[cfg] 让它编译时去掉"
        );
        places[0]
    }

    /// `code`（这份源码测试那一段之外，或用例造的一段）的卡池模块里，能把钉住那两段按路径首段解析的 `std`、`wgpu` 与按名字调的
    /// 那几个函数换掉的写法，一处一句；一处没有交空。判法与名单在 `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call`。
    fn ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(
        code: &str,
    ) -> Vec<String> {
        let module_tokens = card_pool_module_tokens_with_their_brace_depth(code);
        let tokens: Vec<&str> = module_tokens.iter().map(|(_depth, token)| *token).collect();
        let mut found = Vec::new();
        if tokens.contains(&"mod") {
            found.push(
                "卡池模块里另起模块：叫 std、wgpu 的直接遮住外部 crate，别的名字的经 #[macro_use] 把宏漏到这一层、mod 名字; 与 #[path] 的源码不在这一段里"
                    .to_string(),
            );
        }
        for pair in tokens.windows(2) {
            if pair == ["extern", "crate"] {
                found.push(
                    "卡池模块里写 extern crate：extern crate … as std 遮住外部 crate".to_string(),
                );
            }
        }
        for (index, pair) in module_tokens.windows(2).enumerate() {
            // 条目这一层的宏前面是上一个条目的收尾、属性的右方括号，或者模块的开头；跟在 = 后面、属性方括号里的是表达式宏
            // （`const _: () = assert!(…);`、`#[doc = include_str!(…)]`），展开不出条目（第 10z 批第二十三块，正推 K4-2 L7–L9）
            let token_before = index
                .checked_sub(1)
                .map(|index_before| module_tokens[index_before].1);
            if pair[0].0 == 0
                && pair[1].1 == "!"
                && pair[0]
                    .1
                    .starts_with(|character: char| character.is_alphanumeric() || character == '_')
                && token_before.is_none_or(|token| matches!(token, ";" | "\u{7d}" | "]"))
            {
                found.push(format!(
                    "卡池模块的条目这一层调宏或定义宏 {}!：宏展开出来的条目也能叫 std、wgpu",
                    pair[0].1
                ));
            }
        }
        // 一条 use 后面不会紧跟 <：`impl Trait + use<'a>` 里的 use 是精确捕获（第 10z 批第二十三块，正推 K4-2 L10）；
        // 块里的 use 只在那一对花括号里有效，块里没有钉住那两段的签名就罩不到它们（攻方 K4-2 g2）
        let use_indices: Vec<usize> = tokens
            .iter()
            .enumerate()
            .filter(|(use_index, token)| {
                **token == "use"
                    && tokens.get(use_index + 1) != Some(&"<")
                    && does_the_scope_of_the_token_reach_the_pinned_bodies(
                        &module_tokens,
                        *use_index,
                    )
            })
            .map(|(use_index, _token)| use_index)
            .collect();
        let use_statement_at = |use_index: usize| -> &[&str] {
            let length = tokens[use_index..]
                .iter()
                .position(|token| *token == ";")
                .expect("use 以分号收尾")
                + 1;
            &tokens[use_index..use_index + length]
        };
        let use_statements: Vec<&[&str]> = use_indices
            .iter()
            .map(|use_index| use_statement_at(*use_index))
            .collect();
        let mut brought_in = Vec::new();
        for statement in &use_statements {
            let (brought_in_by_the_statement, end_of_the_tree) =
                brought_in_by_a_use_tree(statement, 1, "");
            assert_eq!(
                end_of_the_tree,
                statement.len() - 1,
                "use 树读到分号收尾：{statement:?}"
            );
            brought_in.extend(brought_in_by_the_statement);
        }
        for use_index in &use_indices {
            let attributes = names_of_the_attributes_right_before(&tokens, *use_index);
            let Some(conditional_attribute) = attributes
                .iter()
                .find(|attribute| matches!(**attribute, "cfg" | "cfg_attr"))
            else {
                continue;
            };
            let (brought_in_by_the_statement, _end_of_the_tree) =
                brought_in_by_a_use_tree(use_statement_at(*use_index), 1, "");
            for brought_in_by_a_use in &brought_in_by_the_statement {
                match brought_in_by_a_use {
                    BroughtInByAUse::Name { source: _, name }
                        if FUNCTIONS_THE_PINNED_BODIES_CALL_BY_NAME
                            .iter()
                            .any(|(_prefix, function)| function == name) =>
                    {
                        found.push(format!(
                            "卡池模块里挂 #[{conditional_attribute}] 的 use 引进钉住那两段按名字调的 {name}：编译时这条 use 可以被去掉，按名字调的就落到模块里另起的同名函数上"
                        ));
                    }
                    BroughtInByAUse::Name { .. } | BroughtInByAUse::Glob { .. } => {}
                }
            }
        }
        for brought_in_by_a_use in &brought_in {
            match brought_in_by_a_use {
                BroughtInByAUse::Glob { source } => found.push(format!(
                    "卡池模块里通配 use {source}::*：引进来的名字（叫 std、wgpu 的模块也算）扫这一段看不见"
                )),
                BroughtInByAUse::Name { source, name } => {
                    // use wgpu; 引的就是外部 crate 自己，遮不住它（攻方 K4-2 g1）
                    if matches!(name.as_str(), "std" | "wgpu") && source != name {
                        found.push(format!(
                            "卡池模块经 use 引进叫 {name} 的名字（引的是 {source}），遮住外部 crate"
                        ));
                    }
                }
            }
        }
        for (expected_prefix, function) in FUNCTIONS_THE_PINNED_BODIES_CALL_BY_NAME {
            let sources_brought_in_under_the_name: Vec<&str> = brought_in
                .iter()
                .filter_map(|brought_in_by_a_use| match brought_in_by_a_use {
                    BroughtInByAUse::Name { source, name } if name == function => {
                        Some(source.as_str())
                    }
                    BroughtInByAUse::Name { .. } | BroughtInByAUse::Glob { .. } => None,
                })
                .collect();
            let expected_source = format!("{expected_prefix}::{function}");
            if sources_brought_in_under_the_name != [expected_source.as_str()] {
                found.push(format!(
                    "钉住那两段按名字调的 {function} 要由一条 use 从 {expected_source} 引进卡池模块，引进来的是 {sources_brought_in_under_the_name:?}"
                ));
            }
        }
        found
    }

    /// 第 10z 批第十八块（第 10z 批第十四、十五块代码轮判决 K4，攻方：卡池模块里加 `mod wgpu { … }` 或 `mod std { … }` 遮住方法体里的
    /// `wgpu::PollType::Poll`、`std::panic::catch_unwind`，方法体一个字不动、`cargo check` 0 警告编过，整句比照绿）：钉住的两段（借之前那一步、
    /// 借据丢掉时那一步）里按路径首段解析的 `std`、`wgpu` 与按名字调的那几个函数，在卡池模块这一层只落到外部 crate 与 `use` 点名的那一处。
    /// 这一层能把一个名字换掉的写法，逐样拒：`use` 引进叫 `std` 或 `wgpu` 的名字（改名 `as`、花括号里的 `self`、几层花括号里的都照引进来的名字认）、
    /// `extern crate`；那几个函数各由一条 `use` 从钉住的那一处引进来（引了它，模块里再起同名的条目编译不过；去掉 `use` 另起一个、
    /// 或 `use 别处::某函数 as 那个名字` 就是这里红）。扫这份源码（测试那一段之外，注释与字面量不算），不要卡。
    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K5，正推：以后合法的写法被判红）逐类判过：`!=` 被当成调宏是误判，改了；
    /// `use … as 别的名字`、两层花括号、`use 甲::{self, …}`、`extern "C" { … }` 换不掉钉住那两段用的名字，是误判，改了
    /// （`the_card_pool_scan_reads_an_item_level_not_equal_as_a_comparison_and_still_finds_an_item_level_macro`、
    /// `the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu`、
    /// `the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_brought_in_under_a_rename_a_nested_group_or_self`、
    /// `the_card_pool_scan_accepts_an_extern_block_and_still_finds_extern_crate`）。
    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4）：挂 `#[cfg]`、`#[cfg_attr]` 的 `use` 引进钉住那两段调的函数判红——编译时
    /// 那条 `use` 去掉，按名字调的就落到模块里另起的同名函数上（正推 K4-1 B12、B13，
    /// `the_card_pool_scan_finds_a_pinned_function_brought_in_by_a_use_under_a_cfg`）；换不掉那几个名字的几样判绿
    /// （`the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call`）：`use wgpu;` 引的就是外部 crate 自己；
    /// 块里的 `use`（函数体里 `use core as std;`）只罩那一对花括号，里面没有钉住那两段的签名就不拒，有就照拒
    /// （`the_card_pool_scan_finds_a_rename_in_a_block_holding_a_pinned_body_and_an_item_level_macro_under_an_attribute`）；
    /// 属性与常量、静态量初始值里的表达式宏（`include_str!`、`assert!`、`concat!`）展开不出条目；`impl … + use<'a>` 的 `use` 是精确捕获；
    /// 分词认 `cr#"…"#`、`c"…"`（与崩溃放量那一份认法相同），拼成关键字的原始标识符（`r#mod`）是名字。测试模块只从代码里那一处头两行切
    /// （`the_card_pool_scans_cut_the_tests_module_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string`）。
    ///
    /// 故意拒的写法（换不换得掉那几个名字，扫这一个模块的源码判不出），一种一条，各写为什么
    /// （`the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned`）：
    /// - 通配 `use 甲::*`：引进来哪些名字要看 `甲` 里有什么，`甲` 里有叫 `std`、`wgpu` 的模块就遮住外部 crate；要用就逐个点名；
    /// - 条目这一层调宏或定义宏（`thread_local!` 也算）：宏名按文本作用域解析，卡池模块外面先定义的同名 `macro_rules!` 会顶替标准库那一个，
    ///   展开出来的条目也能叫 `std`、`wgpu`；要用就放进函数体里；
    /// - 另起模块（`#[cfg(test)] mod …` 也算）：叫 `std`、`wgpu` 的直接遮住外部 crate，别的名字的经 `#[macro_use]` 把里面的宏漏到这一层，
    ///   `mod 名字;` 与 `#[path]` 的源码不在这一段里；单测写在这份文件的测试模块里。函数体里的 `mod` 也拒（第 10z 批第二十五块写明，
    ///   正推 K4-2 N33）：它只罩那个函数体、遮不住钉住那两段，是合法写法，判红是故意的，判法不分模块写在哪一层；
    /// - `extern crate`，挂 `#[cfg(any())]` 的也拒（第 10z 批第二十五块写明，正推 K4-2）：编译时去掉的那一条遮不住什么，是合法写法，
    ///   判红是故意的，判法不看 `extern crate` 上挂的属性。
    ///
    /// 钉住的条目（卡池模块按词 `mod card_pool`、借据丢掉时那一步按词 `Drop for GpuCardLease`、四个借卡入口按词 `fn 名字`）在测试那一段之外
    /// 按词恰好一处、前面不挂 `#[cfg]`、`#[cfg_attr]`（[`index_of_the_one_pinned_item_without_a_conditional_attribute`]），下面几种合法写法因此判红，
    /// 也是故意拒，一种一条（第 10z 批第二十六块写明，第 10z 批第二十四、二十五块代码轮判决第二节 K3）：
    /// - 卡池模块挂 `#[cfg(feature = …)]`（攻方 `g16`），红在「前面挂着」：判法不求 `cfg` 的谓词在跑的那一次构建里真不真，挂了就当它可能
    ///   编译时去掉、换成别处另一份；这份文件在 `lib.rs` 里整个已经挂特性 `gpu`，要再挂特性就挂在整个文件那一级；
    /// - 测试那一段之外另有一个与卡池无关、也叫 `card_pool` 的模块（攻方 `g17`），红在「按词恰好一处」：判法按词数整份文件，不解析路径，
    ///   分不出钉住那几段的 `card_pool` 指的是哪一个；改成只在卡池所在的父模块里数，前面先放干净副本、取第一处那一类绕法就在父模块之外放回来；
    ///   另起的模块换个名字；
    /// - 卡池之外另一个类型上也有 `fn lease_card`（攻方 `g19`），红在「按词恰好一处」：同上按词数整份文件，调到的是哪一个 `lease_card`
    ///   要看类型推断，扫源码判不出；另一个方法换个名字；
    /// - 借据丢掉时那一步挂 `#[cfg_attr(…)]`（攻方 `g20`，如 `#[cfg_attr(feature = "trace", allow(dead_code))]`），红在「前面挂着」：
    ///   `cfg_attr` 能展开出 `cfg`（`#[cfg_attr(谓词, cfg(any()))]`），判法不拆它里面写的是什么，一律当成可能编译时去掉；要挂就直接写属性；
    /// - 唯一一份借据丢掉时那一步写成 `impl Drop for self::GpuCardLease`（正推 `G14h`），红在「按词恰好一处」：判法按词认
    ///   `Drop for GpuCardLease`，不解析路径，写成路径就数不到这一份；类型照 `use` 进来的名字直接写。
    ///
    /// 测试模块头两行换写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释）与 CRLF 换行，切口当场判红，
    /// 也是故意拒（[`code_before_the_tests_module`]）。
    ///
    /// 拦不住的写法（扫这一个模块的源码看不见），一种一条：
    /// - 过程宏（派生宏、属性宏）在卡池模块里展开出来的条目；
    /// - crate 根上改 `std` 指什么（`#![no_std]` 加 `extern crate … as std`）；
    /// - `Cargo.toml` 里把 `wgpu` 换成另一个包（`package = …`）；
    /// - 被引的那一处自己换了：`super::super` 那一层的同名函数改了，或改成从别处 `pub use` 进来；
    /// - 过程宏（属性宏）挂在测试模块上、或把它包住，把测试模块编进非测试构建：切口（[`code_before_the_tests_module`]）只认头两行的写法与
    ///   所在的层，宏做了什么扫源码看不见。
    #[test]
    fn the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call()
    {
        let source = include_str!("gpu_unit_checks.rs");
        let code = code_before_the_tests_module(source);
        assert_eq!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(code),
            Vec::<String>::new(),
            "卡池模块这一层不换掉钉住那两段按路径首段解析的 std、wgpu 与按名字调的那几个函数"
        );
    }

    /// 用例造的卡池模块：头上照真源码那样把钉住那两段按名字调的四个函数各 `use` 进来，再接 `items`（花括号写成 \u 转义）。
    fn card_pool_module_with_the_pinned_uses_and(items: &str) -> String {
        let mut module = String::from(concat!(
            "mod card_pool \u{7b}\n",
            "    use super::super::\u{7b}\n",
            "        device_found_idle_by_a_poll, device_wait_before_returning_a_lease,\n",
            "        poll_type_before_returning_a_lease,\n",
            "    \u{7d};\n",
            "    use singlefs_harness::gpu_memory_ledger::wait_for_the_device_and_take_in_the_outcome;\n",
        ));
        module.push_str(items);
        module.push_str("\n\u{7d}\n");
        module
    }

    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K5，正推 `K5_const_ne`、`K5_const_ne_ident`）：条目这一层的
    /// `const 名: bool = 甲 != 乙;` 是比较，不是调宏 `甲!`；条目这一层真调宏（`thread_local!`）照旧拒。不要卡。
    /// 第 10z 批第二十三块：条目层的宏另看前一个词（跟在 `=` 后面的不算）之后，`!=` 拆成两个词也不再被认成调宏，两道各自拦得住这一形；
    /// 分词把 `!=` 当一个词这一道另由头一句断言钉住。
    #[test]
    fn the_card_pool_scan_reads_an_item_level_not_equal_as_a_comparison_and_still_finds_an_item_level_macro(
    ) {
        assert_eq!(
            rust_tokens_without_comments_and_literals("FIRST_LIMIT!=SECOND_LIMIT"),
            ["FIRST_LIMIT", "!=", "SECOND_LIMIT"],
            "分词把不等号当一个词：拆成 ! 与 = 时 甲 ! 读起来像调宏"
        );
        let comparisons = card_pool_module_with_the_pinned_uses_and(concat!(
            "    const FIRST_LIMIT: u8 = 1;\n",
            "    const SECOND_LIMIT: u8 = 2;\n",
            "    const THE_LIMITS_DIFFER: bool = FIRST_LIMIT != SECOND_LIMIT;\n",
            "    const THE_LITERALS_DIFFER: bool = 1 != 2;\n",
            "    const THE_LIMITS_DIFFER_WITHOUT_SPACES: bool = FIRST_LIMIT!=SECOND_LIMIT;",
        ));
        assert_eq!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&comparisons),
            Vec::<String>::new(),
            "条目这一层的 != 是比较，不算调宏：{comparisons}"
        );
        let item_level_macro = card_pool_module_with_the_pinned_uses_and(
            "    thread_local! \u{7b} static COUNT: std::cell::Cell<u8> = std::cell::Cell::new(0); \u{7d}",
        );
        assert_ne!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&item_level_macro),
            Vec::<String>::new(),
            "条目这一层调宏照旧拒：{item_level_macro}"
        );
    }

    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K5，正推 `K5_use_as`、`K5_nested_use_braces`、`K5_pub_use_self_group`）：
    /// 改成别的名字、两层花括号、花括号里的 `self` 引进来的名字既不叫 `std`、`wgpu`，也不是钉住那两段调的函数，不拒。不要卡。
    #[test]
    fn the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu(
    ) {
        let module = card_pool_module_with_the_pinned_uses_and(concat!(
            "    use std::sync::Arc as SharedArc;\n",
            "    use std::collections::\u{7b}BTreeMap, btree_map::\u{7b}Entry\u{7d}\u{7d};\n",
            "    use std::sync::\u{7b}self, Mutex\u{7d};\n",
            "    use ::std::fmt::\u{7b}Display as Shown,\u{7d};",
        ));
        assert_eq!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
            Vec::<String>::new(),
            "这几条 use 引进来的是 SharedArc、BTreeMap、Entry、sync、Mutex、Shown：{module}"
        );
    }

    /// 第 10z 批第二十一块（同上一条的反面）：改名、套在里层花括号里、花括号里的 `self` 引进叫 `std`、`wgpu` 的名字照旧拒；
    /// 钉住那两段调的函数换成 `use 别处::某函数 as 那个名字` 也拒。不要卡。
    #[test]
    fn the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_brought_in_under_a_rename_a_nested_group_or_self(
    ) {
        for (form, items) in [
            (
                "改名成 std",
                "    use super::a_module_shadowing_the_crate as std;",
            ),
            (
                "里层花括号里的 wgpu",
                "    use super::\u{7b}outer::\u{7b}wgpu, other\u{7d}\u{7d};",
            ),
            (
                "花括号里的 self 引进 wgpu",
                "    use super::shadowing::wgpu::\u{7b}self\u{7d};",
            ),
            (
                "花括号里的 self 改名成 std",
                "    use super::\u{7b}shadowing::\u{7b}self as std\u{7d}\u{7d};",
            ),
        ] {
            let module = card_pool_module_with_the_pinned_uses_and(items);
            assert_ne!(
                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
                Vec::<String>::new(),
                "{form} 拒：{module}"
            );
        }
        let pinned_function_renamed_in = card_pool_module_with_the_pinned_uses_and("")
            .replacen("device_found_idle_by_a_poll, ", "", 1)
            .replacen(
                "    use singlefs_harness",
                "    use super::super::a_poll_that_always_finds_the_device_idle as device_found_idle_by_a_poll;\n    use singlefs_harness",
                1,
            );
        assert!(
            !pinned_function_renamed_in.contains("device_found_idle_by_a_poll, "),
            "造的模块去掉了从钉住那一处引的 device_found_idle_by_a_poll：{pinned_function_renamed_in}"
        );
        assert_eq!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(
                &pinned_function_renamed_in
            ),
            vec![
                "钉住那两段按名字调的 device_found_idle_by_a_poll 要由一条 use 从 super::super::device_found_idle_by_a_poll 引进卡池模块，引进来的是 [\"super::super::a_poll_that_always_finds_the_device_idle\"]"
                    .to_string()
            ],
            "别处的函数改名成钉住那两段调的名字引进来，拒：{pinned_function_renamed_in}"
        );
    }

    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K5，正推 `K5_extern_c_block`）：`extern "C" { … }` 里声明的外部函数
    /// 在值的名字空间，遮不住按路径首段解析的 `std`、`wgpu`，与 `use` 进来的同名函数撞了编译不过，不拒；`extern crate` 照旧拒。不要卡。
    #[test]
    fn the_card_pool_scan_accepts_an_extern_block_and_still_finds_extern_crate() {
        let extern_block = card_pool_module_with_the_pinned_uses_and(
            "    extern \"C\" \u{7b} fn abs(input: i32) -> i32; \u{7d}",
        );
        assert_eq!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&extern_block),
            Vec::<String>::new(),
            "extern 块不拒：{extern_block}"
        );
        let extern_crate =
            card_pool_module_with_the_pinned_uses_and("    extern crate alloc as std;");
        assert_ne!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&extern_crate),
            Vec::<String>::new(),
            "extern crate 照旧拒：{extern_crate}"
        );
    }

    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K5，正推 `K5_use_wildcard_enum`、`K5_thread_local_macro`、
    /// `K5_cfg_test_mod_inside`）：故意拒的三类（为什么写在
    /// `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call` 的名单里），
    /// 写的东西本身不叫 `std`、`wgpu` 也拒。不要卡。
    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K4，正推 K4-2：合法写法判红的写明故意拒）：函数体里的 `mod`、
    /// 挂 `#[cfg(any())]` 的 `extern crate` 两样也在这里钉成拒。
    #[test]
    fn the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned(
    ) {
        for (form, items) in [
            ("通配", "    use std::cmp::Ordering::*;"),
            (
                "条目这一层调宏",
                "    thread_local! \u{7b} static COUNT: std::cell::Cell<u8> = std::cell::Cell::new(0); \u{7d}",
            ),
            (
                "另起模块",
                "    #[cfg(test)]\n    mod unit_checks_of_the_pool \u{7b} \u{7d}",
            ),
            (
                "函数体里另起模块",
                "    fn a_helper_with_an_inner_module() -> u8 \u{7b} mod inner \u{7b}\u{7d} 0 \u{7d}",
            ),
            (
                "挂 cfg 的 extern crate",
                "    #[cfg(any())]\n    extern crate alloc;",
            ),
        ] {
            let module = card_pool_module_with_the_pinned_uses_and(items);
            assert_ne!(
                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
                Vec::<String>::new(),
                "{form} 故意拒：{module}"
            );
        }
    }

    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，攻方 K4-2）：卡池模块之前写一份测试模块的头两行在块注释里（或跨行的字符串里），
    /// 扫源码的用例照旧切到真测试模块的头两行：切出来的那一段只多了写进去的那几行。不要卡。
    #[test]
    fn the_card_pool_scans_cut_the_tests_module_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string(
    ) {
        let source = include_str!("gpu_unit_checks.rs");
        assert_eq!(
            source.matches(TESTS_MODULE_OPENING).count(),
            1,
            "这份源码里测试模块的头两行原文只有代码里那一处"
        );
        let code = code_before_the_tests_module(source);
        assert!(
            source[code.len()..].starts_with(TESTS_MODULE_OPENING),
            "切口在真测试模块的头两行"
        );
        let card_pool_opening = "    mod card_pool \u{7b}\n";
        assert_eq!(
            code.matches(card_pool_opening).count(),
            1,
            "测试那一段之外恰好一处卡池模块"
        );
        for (form, copy_of_the_opening) in [
            (
                "块注释",
                format!("    /*\n{TESTS_MODULE_OPENING}\n    */\n"),
            ),
            (
                "跨行的字符串",
                format!(
                    "    pub const A_COPY_OF_THE_OPENING: &str = \"\n{TESTS_MODULE_OPENING}\n\";\n"
                ),
            ),
        ] {
            let with_a_copy = source.replacen(
                card_pool_opening,
                &format!("{copy_of_the_opening}{card_pool_opening}"),
                1,
            );
            assert_eq!(
                code_before_the_tests_module(&with_a_copy).len(),
                code.len() + copy_of_the_opening.len(),
                "卡池模块之前{form}里写一份测试模块的头两行，切口不挪"
            );
        }
    }

    /// 跑 `check`，它 panic 时交 panic 的消息，不 panic 交 `None`：扫源码的用例看判法在改过的源码上红没红、红在哪一句。
    fn panic_message_of(check: impl FnOnce() + std::panic::UnwindSafe) -> Option<String> {
        std::panic::catch_unwind(check).err().map(|payload| {
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|message| (*message).to_string())
                })
                .unwrap_or_default()
        })
    }

    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K3、K4，正推 K3-5 `W8`、`W9`，K4-1 `S14`、`S19`）：真测试模块头两行之前
    /// 写一份头两行，认不认照 [`code_before_the_tests_module`] 那几条。宏调用圆括号里的（前一个词是左括号，`S19`；前面先放一条以分号收尾的条目）、
    /// 花括号里的（前面先放一条以分号收尾的条目）、前一个词不是条目收尾的，都不认，切口照旧在真测试模块；代码里两处都认的（恰好一处的上界）、
    /// 真测试模块包进宏调用的圆括号（`S14`）、头两行换写法与 CRLF 换行（按行逐字的口径，故意拒），都红在「恰好一处」那一句。不要卡。
    /// 第 10z 批第二十六块（第 10z 批第二十四、二十五块代码轮判决第二节 K2，攻方 B3：括号那一道只数圆括号照绿）：真测试模块包进方括号的
    /// 宏调用（`keep_hidden![` 起、前面先放一条以分号收尾的条目、`];` 收尾，攻方 `g09`；rustc 编得过、非测试构建里照编），也红在「恰好一处」。
    #[test]
    fn the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item(
    ) {
        let source = include_str!("gpu_unit_checks.rs");
        let code = code_before_the_tests_module(source);
        for (form, copy_of_the_opening) in [
            (
                "宏调用圆括号里的一份（S19）",
                "keep_hidden!(\n#[cfg(test)]\nmod tests \u{7b}\n    fn hidden_in_the_wrapped_copy() \u{7b}\u{7d}\n\u{7d}\n);\n",
            ),
            (
                "宏调用圆括号里、前面先放一条以分号收尾的条目的一份",
                "keep_hidden!(\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n);\n",
            ),
            (
                "花括号里、前面先放一条以分号收尾的条目的一份",
                "mod a_module_holding_a_copy \u{7b}\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n\u{7d}\n",
            ),
            (
                "前一个词不是条目收尾的一份",
                "static NOT_ENDED_BY_A_SEMICOLON: u8 = 0\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n",
            ),
        ] {
            let with_a_copy = source.replacen(
                TESTS_MODULE_OPENING,
                &format!("{copy_of_the_opening}{TESTS_MODULE_OPENING}"),
                1,
            );
            assert_eq!(
                code_before_the_tests_module(&with_a_copy).len(),
                code.len() + copy_of_the_opening.len(),
                "{form}不认，切口照旧在真测试模块"
            );
        }
        let opening_offset = source
            .find(TESTS_MODULE_OPENING)
            .expect("这份源文件里有测试模块的头两行");
        let end_of_the_tests_module = opening_offset
            + source[opening_offset..]
                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
                .expect("测试模块在顶格的右括号行收尾")
            + TOP_LEVEL_CLOSING_BRACE_LINE.len();
        let tests_wrapped_in_square_brackets_after_an_item =
            String::from(&source[..opening_offset])
                + "keep_hidden![\nconst BEFORE_THE_TESTS: u8 = 0;\n"
                + &source[opening_offset..end_of_the_tests_module]
                + "];\n"
                + &source[end_of_the_tests_module..];
        let mut refused_sources: Vec<(&str, String)> = vec![
            (
                "代码里两处都认的头两行",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &format!(
                        "{TESTS_MODULE_OPENING}{TOP_LEVEL_CLOSING_BRACE_LINE}{TESTS_MODULE_OPENING}"
                    ),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的圆括号（S14）",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &format!("keep_hidden!(\n{TESTS_MODULE_OPENING}"),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的圆括号、前面先放一条以分号收尾的条目",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &(String::from("keep_hidden!(\nconst BEFORE_THE_TESTS: u8 = 0;\n")
                        + TESTS_MODULE_OPENING),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的花括号、前面先放一条以分号收尾的条目",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &(String::from("keep_hidden! \u{7b}\nconst BEFORE_THE_TESTS: u8 = 0;\n")
                        + TESTS_MODULE_OPENING),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的方括号、前面先放一条以分号收尾的条目（攻方 g09）",
                tests_wrapped_in_square_brackets_after_an_item,
            ),
            ("CRLF 换行", source.replace('\n', "\r\n")),
        ];
        for (form, other_writing) in [
            ("mod 行少一个空格", "#[cfg(test)]\nmod tests\u{7b}"),
            ("写在一行", "#[cfg(test)] mod tests \u{7b}"),
            ("属性里多空格", "#[ cfg ( test ) ]\nmod tests \u{7b}"),
            ("mod 行多空格", "#[cfg(test)]\nmod  tests  \u{7b}"),
            ("头一行缩进", "    #[cfg(test)]\nmod tests \u{7b}"),
            ("头一行行尾带空白", "#[cfg(test)]  \nmod tests \u{7b}"),
            ("头一行行尾带注释", "#[cfg(test)] // 单测\nmod tests \u{7b}"),
            ("第二行行尾带注释", "#[cfg(test)]\nmod tests \u{7b} // 单测"),
        ] {
            refused_sources.push((
                form,
                source.replacen(TESTS_MODULE_OPENING, other_writing, 1),
            ));
        }
        for (form, refused_source) in &refused_sources {
            let message = panic_message_of(|| {
                code_before_the_tests_module(refused_source);
            });
            assert!(
                message
                    .as_deref()
                    .is_some_and(|message| message.contains("测试模块的头两行恰好一处")),
                "{form}：切口红在「恰好一处」那一句，读到 {message:?}"
            );
        }
    }

    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K4，正推 K4-1 G14、G16：借据丢掉时那一步、四个借卡入口取第一处，前面先放一份挂
    /// `#[cfg(any())]` 的干净副本、后面写真生效的坏版本，14 条卡池用例全绿）：借据丢掉时那一步、借卡入口、卡池模块各按词恰好一处、不挂 `#[cfg]`
    /// （[`index_of_the_one_pinned_item_without_a_conditional_attribute`]）。干净副本在前、坏版本在后（G14、G16；干净副本不挂 `#[cfg]`、包进函数体里一次展开成空的
    /// 宏调用的也算）与干净的卡池模块副本在前，红在「恰好一处」；
    /// 唯一的一份借据丢掉时那一步挂 `#[cfg(any())]`，红在「前面挂着」；只有坏版本（G15、G17），照旧红在钉住的那几句。扫这份源码改过的一份，不要卡。
    #[test]
    fn the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module(
    ) {
        let source = include_str!("gpu_unit_checks.rs");
        let piece_of_the_source = |opening: &str, closing: &str| -> (usize, usize) {
            let start = source
                .find(opening)
                .unwrap_or_else(|| panic!("源码里有 {opening}"));
            let end = start
                + source[start..]
                    .find(closing)
                    .unwrap_or_else(|| panic!("{opening} 收得了尾"))
                + closing.len();
            (start, end)
        };
        let replaced = |start: usize, end: usize, replacement: &str| {
            format!("{}{replacement}{}", &source[..start], &source[end..])
        };
        let (drop_start, drop_end) = piece_of_the_source(
            "        impl Drop for GpuCardLease \u{7b}\n",
            "\n        \u{7d}\n",
        );
        let pinned_drop = &source[drop_start..drop_end];
        let drop_doing_nothing = "        impl Drop for GpuCardLease \u{7b}\n            fn drop(&mut self) \u{7b}\u{7d}\n        \u{7d}\n";
        let (entry_start, entry_end) = piece_of_the_source(
            "            pub fn lease_the_first_card_with_room_now(",
            "\n            \u{7d}\n",
        );
        let pinned_entry = &source[entry_start..entry_end];
        let entry_without_the_poll = pinned_entry.replacen(
            "                self.take_in_the_devices_found_idle();\n",
            "",
            1,
        );
        assert_ne!(
            entry_without_the_poll, pinned_entry,
            "造的坏借卡入口少了借之前那一步"
        );
        for (form, changed_source, expected_red) in [
            (
                "干净的借据丢掉时那一步挂 cfg 在前、坏版本在后（G14）",
                replaced(
                    drop_start,
                    drop_end,
                    &format!("        #[cfg(any())]\n{pinned_drop}{drop_doing_nothing}"),
                ),
                "按词恰好一处",
            ),
            (
                "干净的借据丢掉时那一步包进函数体里一次展开成空的宏调用在前、坏版本在后",
                replaced(
                    drop_start,
                    drop_end,
                    &(String::from(
                        "        fn a_place_for_the_clean_copy() \u{7b}\n            expands_to_nothing! \u{7b}\n",
                    ) + pinned_drop
                        + "            \u{7d}\n        \u{7d}\n"
                        + drop_doing_nothing),
                ),
                "按词恰好一处",
            ),
            (
                "只有坏的借据丢掉时那一步（G15）",
                replaced(drop_start, drop_end, drop_doing_nothing),
                "借据丢掉时那一步要有一处",
            ),
            (
                "唯一的一份借据丢掉时那一步挂 cfg",
                replaced(
                    drop_start,
                    drop_end,
                    &format!("        #[cfg(any())]\n{pinned_drop}"),
                ),
                "前面挂着",
            ),
            (
                "干净的借卡入口挂 cfg 在前、坏版本在后（G16）",
                replaced(
                    entry_start,
                    entry_end,
                    &format!("            #[cfg(any())]\n{pinned_entry}{entry_without_the_poll}"),
                ),
                "按词恰好一处",
            ),
            (
                "只有坏的借卡入口（G17）",
                replaced(entry_start, entry_end, &entry_without_the_poll),
                "借之前先 poll 没等成的那几张卡",
            ),
        ] {
            let message = panic_message_of(|| {
                assert_the_lease_return_and_every_booking_entry_are_pinned(&changed_source);
            });
            assert!(
                message
                    .as_deref()
                    .is_some_and(|message| message.contains(expected_red)),
                "{form}：红在「{expected_red}」那一句，读到 {message:?}"
            );
        }
        let pool_opening = "    mod card_pool \u{7b}\n";
        let with_a_clean_pool_copy_first = source.replacen(
            pool_opening,
            &format!(
                "    #[cfg(any())]\n{}{pool_opening}",
                card_pool_module_with_the_pinned_uses_and("")
            ),
            1,
        );
        let message = panic_message_of(|| {
            let found = ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(
                code_before_the_tests_module(&with_a_clean_pool_copy_first),
            );
            drop(found);
        });
        assert!(
            message
                .as_deref()
                .is_some_and(|message| message.contains("按词恰好一处")),
            "干净的卡池模块挂 cfg 在前：红在「恰好一处」那一句，读到 {message:?}"
        );
    }

    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，正推 K4-1 B12、B13：钉住那两段调的 `device_found_idle_by_a_poll` 改由一条挂
    /// `#[cfg(any())]` 的 `use` 引进、模块里另起一个同名函数，编译时那条 `use` 去掉、按名字调的是另起的那一个，扫源码照绿）：挂 `#[cfg]`、
    /// 展开成 `cfg` 的 `#[cfg_attr]`、`pub(crate)` 之前的 `#[cfg]`，都红。不要卡。
    #[test]
    fn the_card_pool_scan_finds_a_pinned_function_brought_in_by_a_use_under_a_cfg() {
        for (form, conditional_use) in [
            (
                "cfg",
                "    #[cfg(any())]\n    use super::super::device_found_idle_by_a_poll;\n",
            ),
            (
                "展开成 cfg 的 cfg_attr",
                "    #[cfg_attr(all(), cfg(any()))]\n    use super::super::device_found_idle_by_a_poll;\n",
            ),
            (
                "pub(crate) 之前的 cfg，另隔一条别的属性",
                "    #[cfg(any())]\n    #[allow(unused_imports, reason = \"造的\")]\n    pub(crate) use super::super::device_found_idle_by_a_poll;\n",
            ),
        ] {
            let module = card_pool_module_with_the_pinned_uses_and(
                "    fn device_found_idle_by_a_poll(_outcome: &u8) -> bool \u{7b} true \u{7d}",
            )
            .replacen("device_found_idle_by_a_poll, ", "", 1)
            .replacen(
                "    use singlefs_harness",
                &format!("{conditional_use}    use singlefs_harness"),
                1,
            );
            let found = ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module);
            assert!(
                found.iter().any(|finding| finding
                    .contains("的 use 引进钉住那两段按名字调的 device_found_idle_by_a_poll")),
                "挂 {form} 的 use 引进钉住那两段调的函数，红：{found:#?}\n{module}"
            );
        }
    }

    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，攻方 K4-2 g1、g2，正推 K4-2 L7–L10、L13b、L14）：换不掉钉住那两段用的名字的写法，
    /// 一样一个模块，都判绿。不要卡。
    #[test]
    fn the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call() {
        for (form, items) in [
            ("引外部 crate 自己", "    use wgpu;\n    use ::std;"),
            (
                "别的函数体里块作用域的改名",
                "    fn a_helper_with_its_own_std() -> u8 \u{7b} use core as std; std::convert::identity(0) \u{7d}",
            ),
            (
                "属性里的表达式宏",
                "    #[doc = include_str!(\"card_pool.md\")]\n    const POOL_SIZE: u8 = 2;",
            ),
            ("常量初始值里的表达式宏", "    const _: () = assert!(1 < 2);"),
            (
                "静态量初始值里的表达式宏",
                "    static JOINED_NAME: &str = concat!(\"card\", \"pool\");",
            ),
            (
                "精确捕获",
                "    fn borrowed_names<'pool>(names: &'pool [u8]) -> impl Iterator<Item = &'pool u8> + use<'pool> \u{7b} names.iter() \u{7d}",
            ),
            (
                "C 风格原始字符串里有引号与花括号",
                "    const QUOTED: &core::ffi::CStr = cr#\"say \"\u{7b}\" here\"#;\n    const PLAIN: &core::ffi::CStr = c\"say \u{7d} here\";",
            ),
            (
                "拼成关键字的原始标识符",
                "    fn r#mod() -> u8 \u{7b} 0 \u{7d}\n    fn r#use(r#extern: u8) -> u8 \u{7b} r#extern \u{7d}",
            ),
            (
                "挂 cfg 的 use 引的不是钉住那两段调的函数",
                "    #[cfg(any())]\n    use std::sync::Arc as SharedArcWhenConfigured;",
            ),
        ] {
            let module = card_pool_module_with_the_pinned_uses_and(items);
            assert_eq!(
                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
                Vec::<String>::new(),
                "{form} 换不掉钉住那两段用的名字，判绿：{module}"
            );
        }
    }

    /// 第 10z 批第二十三块（上一条的反面）：块里的改名所在那一对花括号里有钉住那两段的签名（`impl Drop for GpuCardLease` 挪进函数体）照拒；
    /// 属性下面的条目层宏照拒。不要卡。
    #[test]
    fn the_card_pool_scan_finds_a_rename_in_a_block_holding_a_pinned_body_and_an_item_level_macro_under_an_attribute(
    ) {
        for (form, items) in [
            (
                "块里改名、同一块里有借据丢掉时那一步",
                "    fn holding_the_lease_drop() \u{7b}\n        use core as std;\n        impl Drop for GpuCardLease \u{7b} fn drop(&mut self) \u{7b}\u{7d} \u{7d}\n    \u{7d}",
            ),
            (
                "属性下面的条目层宏",
                "    #[allow(unused, reason = \"造的\")]\n    thread_local! \u{7b} static COUNT: std::cell::Cell<u8> = std::cell::Cell::new(0); \u{7d}",
            ),
        ] {
            let module = card_pool_module_with_the_pinned_uses_and(items);
            assert_ne!(
                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
                Vec::<String>::new(),
                "{form} 拒：{module}"
            );
        }
    }

    /// 第 10z 批第十八块（第 10z 批第十四、十五块代码轮正推 K4：钉住的那一句里 `started.device()` 交的是哪一台不在整句里，
    /// `started.device_condition()` 同理）：一张卡交出去的设备与设备的情况就是它自己那两格。要真卡上起好的设备才造得出 `GpuCard`，
    /// 换不成不要卡的行为用例，这里扫这份源码（测试那一段之外、去掉注释行与空白）：两个方法各恰好一处，方法体整个是那一格。不要卡。
    ///
    /// 拦不住的写法，一种一条：
    /// - 这两格里放的是哪一台、哪一份（`start_the_card` 起设备时放进去，克隆共用同一台）；
    /// - 在方法上面加 `#[cfg]`，让签名写法与钉住的不同的另一份同名方法编进来。
    #[test]
    fn a_card_hands_out_its_own_device_and_its_own_device_condition() {
        let source = include_str!("gpu_unit_checks.rs");
        let code = code_before_the_tests_module(source);
        for (signature, expected_body) in [
            (
                "pub fn device(&self) -> &wgpu::Device \u{7b}",
                "&self.device",
            ),
            (
                "pub fn device_condition(&self) -> &DeviceConditionOfACard \u{7b}",
                "&self.device_condition",
            ),
        ] {
            assert_eq!(
                code.matches(signature).count(),
                1,
                "源码里恰好一处 {signature}"
            );
            let start = code
                .find(signature)
                .unwrap_or_else(|| panic!("源码里有 {signature}"));
            let after = &code[start + signature.len()..];
            let body = &after[..after
                .find("\n    \u{7d}\n")
                .expect("GpuCard 的方法以缩进四格的右括号收尾")];
            let body_without_comments_and_spaces: String = body
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .flat_map(str::chars)
                .filter(|character| !character.is_whitespace())
                .collect();
            assert_eq!(
                body_without_comments_and_spaces, expected_body,
                "{signature} 交的是这张卡自己那一格"
            );
        }
    }

    /// Cargo.lock 里 wgpu-core 的版本（`name = "wgpu-core"` 下一行的 `version = "…"`）。
    fn wgpu_core_version_in_the_lock(lock: &str) -> Option<String> {
        let mut lines = lock.lines();
        lines.find(|line| line.trim() == "name = \"wgpu-core\"")?;
        let version_line = lines.next()?.trim();
        Some(
            version_line
                .strip_prefix("version = \"")?
                .strip_suffix('"')?
                .to_string(),
        )
    }

    /// cargo 缓存（`CARGO_HOME`，没设是 `~/.cargo`）的 `registry/src/*/wgpu-core-<版本>/src/device/resource.rs`；取不到是 `None`。
    fn wgpu_core_device_source_in_the_cargo_registry(version: &str) -> Option<String> {
        let cargo_home = std::env::var_os("CARGO_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".cargo"))
            })?;
        std::fs::read_dir(cargo_home.join("registry").join("src"))
            .ok()?
            .filter_map(Result::ok)
            .map(|sources_of_one_registry| {
                sources_of_one_registry
                    .path()
                    .join(format!("wgpu-core-{version}"))
                    .join("src")
                    .join("device")
                    .join("resource.rs")
            })
            .find_map(|path| std::fs::read_to_string(path).ok())
    }

    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K3，攻方 F5）：wgpu-core 因 hal 的错丢设备时，交给设备丢失回调的那一句是那个 hal 错的
    /// `to_string()`（[`cause_of_the_device_loss`] 照「Out of memory」那一句比，上一条用例钉 hal 那一句）：扫 cargo 缓存里这一版 wgpu-core 的源码，
    /// `handle_hal_error` 里 `OutOfMemory` 那一支走 `self.lose(&error.to_string());`。版本取仓根 Cargo.lock；源码取不到打「本次未跑」那一行、
    /// 不报绿。不要卡。
    #[test]
    fn wgpu_core_loses_the_device_with_the_sentence_of_the_hal_error_it_failed_with() {
        const TEST_NAME: &str = "gpu_unit_checks::tests::wgpu_core_loses_the_device_with_the_sentence_of_the_hal_error_it_failed_with";
        const LOSS_WITH_THE_SENTENCE_OF_THE_HAL_ERROR: &str = "self.lose(&error.to_string());";
        let lock = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock"),
        )
        .expect("仓根有 Cargo.lock");
        let version =
            wgpu_core_version_in_the_lock(&lock).expect("Cargo.lock 里有 wgpu-core（wgpu 依赖它）");
        let Some(source) = wgpu_core_device_source_in_the_cargo_registry(&version) else {
            println!(
                "{}",
                singlefs_harness::gpu_memory_ledger::gpu_not_run_line(
                    TEST_NAME,
                    &format!(
                        "cargo 缓存里没有 wgpu-core {version} 的 src/device/resource.rs（CARGO_HOME 或 ~/.cargo 下的 registry/src）"
                    )
                )
            );
            return;
        };
        let handle_start = source
            .find("pub fn handle_hal_error(")
            .expect("wgpu-core 的设备上有 handle_hal_error：没有就是这一版改了丢设备的路，要重看");
        let handle_body = &source[handle_start..];
        let handle_body = &handle_body[..handle_body
            .find("\n    \u{7d}\n")
            .expect("handle_hal_error 以缩进四格的右括号收尾")];
        assert!(
            handle_body.contains("hal::DeviceError::OutOfMemory"),
            "wgpu-core {version} 的 handle_hal_error 不再认 hal 的 OutOfMemory：{handle_body}"
        );
        assert_eq!(
            handle_body
                .matches(LOSS_WITH_THE_SENTENCE_OF_THE_HAL_ERROR)
                .count(),
            1,
            "wgpu-core {version} 丢设备时交给回调的不再是 hal 错的那一句：{handle_body}"
        );
    }

    /// 主 agent 2026-10-10 定的条款：卡池只认进程起来时（第一次换环境之前）调用方的那份配置，不跟用例临时改的环境走。确定地造出
    /// 「别的用例拿着改环境的锁换着临时配置时第一次起池」那种交错（`crate::gpu_card_configuration::while_another_thread_holds_a_temporary_gpu_configuration`：
    /// 那条线程把多机配置、显卡先后、这台机器的角色换成它的临时那一份，这边起完池它才放）：起池照的是调用方给这个进程的那一份
    /// （`GpuConfigurationOfTheCaller::of_this_process`），不是那条线程的。起池那一步只找卡、开账，不起设备、不借显存，没有卡的机器上照样判
    /// （判的是起池照哪一份配置挑卡，不看找没找到卡）。
    #[cfg(feature = "verdict-store")]
    #[test]
    fn the_card_pool_started_while_another_thread_holds_a_temporary_configuration_picks_from_the_configuration_of_the_caller(
    ) {
        use super::gpu_test_start::start_the_gpu_card_pool;
        use crate::gpu_card_configuration::{
            while_another_thread_holds_a_temporary_gpu_configuration, GpuConfigurationOfTheCaller,
            CARDS_OF_THE_TEMPORARY_GPU_CONFIGURATION,
        };
        let (start, temporary_configuration) =
            while_another_thread_holds_a_temporary_gpu_configuration(
                "card-pool-start",
                start_the_gpu_card_pool,
            );
        let picked_from = format!("{:?}", start.configuration);
        assert!(
            !picked_from.contains(CARDS_OF_THE_TEMPORARY_GPU_CONFIGURATION)
                && !picked_from.contains(&*temporary_configuration.to_string_lossy()),
            "起池照的是拿着锁那条线程换上的临时配置 {}：{picked_from}",
            temporary_configuration.display()
        );
        assert_eq!(
            start.configuration,
            GpuConfigurationOfTheCaller::of_this_process(),
            "起池照的是调用方给这个进程的那一份"
        );
    }
}
