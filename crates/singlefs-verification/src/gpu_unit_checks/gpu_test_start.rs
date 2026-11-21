use super::{GpuCard, NVIDIA_VENDOR_IDENTIFIER};
use crate::crash_amplification::RecordedGivenStatesTable;
use crate::crash_library::{
    two_stages, CrashLibrary, CrashLibraryClass, ItemManifest, ItemName, RecordingInputsDigest,
    StageOutcome,
};
use crate::gpu_card_configuration::{CardsOfTheCallerForTheGpuTests, GpuConfigurationOfTheCaller};
use crate::sha256::{sha256_digest, DIGEST_BYTES};

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

/// GPU 对拍用例里 CPU 参照的输入经对拍那一类的库（CPU 参照也从库读，两段之间不在内存里交数据）：
/// 录入段把 CPU 参照要判的那几张镜像（一张给定状态表：基线、写表、持久集合）按摘要写进
/// [`CrashLibraryClass::JudgeTablesPairing`] 那一类的库（prune-server 那一份）、按项名（[`cpu_reference_item_name`]）登清单；
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
        &item,
        &RecordingInputsDigest(digest),
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
    base: &crate::memory_pool::MemoryPool,
    writes: &[crate::memory_pool::RetainedWrite],
) -> Option<crate::memory_pool::MemoryPool> {
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
    /// 崩溃放量认的那一种卡：NVIDIA 独显（[`gpu_cards_by_priority`] 只从这一种里起上下文）。
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
/// 起好上下文的卡数（[`gpu_cards_by_priority`] 交回几张），卡池是找到的点名卡数（卡池起池时不起设备，借到哪张才起哪张）。
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

/// 这条用例报「本次未跑」时打的那一行（记号与写法在 `gpu_memory_ledger` 一处定义，崩溃放量产品路径此刻借不到、卡上报 Out of Memory 也打它：
/// `singlefs_verification::gpu_memory_ledger::gpu_not_run_line`）。
#[must_use]
pub fn not_run_line(test_name: &str, reason: &str) -> String {
    crate::gpu_memory_ledger::gpu_not_run_line(test_name, reason)
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

/// 本机 Vulkan 枚举出的每个适配器（与 [`gpu_cards_by_priority`] 同一个后端）。
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
    let unreadable = |what: String| PciNvidiaDisplayControllers::ListingUnreadable { reason: what };
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

/// 服务交给这一台的卡（`handed_over`：序号与额度 MiB）里，`nvidia-smi` 此刻报的（`reported`：序号与空闲显存 MiB）那几张的序号与额度，
/// 空闲显存从大到小（一样大的照 `reported` 的次序）。点名之外的卡再空也不挑；点名而 `nvidia-smi` 没报的不挑。不碰卡。
#[must_use]
pub(in crate::gpu_unit_checks) fn handed_over_cards_emptiest_first(
    handed_over: &[(u32, u64)],
    reported: &[(u32, u64)],
) -> Vec<(u32, u64)> {
    let mut picked: Vec<(u64, (u32, u64))> = reported
        .iter()
        .filter_map(|(index, free_memory_mebibytes)| {
            handed_over
                .iter()
                .find(|(handed_over_index, _)| handed_over_index == index)
                .map(|(_, memory_quota_mebibytes)| {
                    (*free_memory_mebibytes, (*index, *memory_quota_mebibytes))
                })
        })
        .collect();
    picked.sort_by_key(|(free_memory_mebibytes, _)| std::cmp::Reverse(*free_memory_mebibytes));
    picked.into_iter().map(|(_, card)| card).collect()
}

/// 服务交了卡时起出来的卡（`started_cards`）：至少一张照交回；一张都没起来交判红的原因，写交了哪几张、`nvidia-smi` 报了哪几张。
///
/// # Errors
/// 一张都没起来：交回的串是判红的原因。
pub(in crate::gpu_unit_checks) fn started_cards_or_the_reason_the_handed_over_cards_do_not_run<
    StartedCard,
>(
    started_cards: Vec<StartedCard>,
    handed_over: &[(u32, u64)],
    reported: &[(u32, u64)],
) -> Result<Vec<StartedCard>, String> {
    if started_cards.is_empty() {
        return Err(format!(
            "服务经 GPU_CARDS_BY_PRIORITY 交给这一台的卡（序号, 额度 MiB）{handed_over:?} 一张都用不了（交的卡都在另一台、nvidia-smi 没报这个序号、空闲显存低于门槛、或上下文起不来，上面 GPU_CARD_NOT_USABLE 那几行）；nvidia-smi 报的卡（序号, 空闲 MiB）：{reported:?}"
        ));
    }
    Ok(started_cards)
}

/// 服务交了卡时的取卡：只在交给这一台的卡（`handed_over`）上起设备，空闲显存最大的排头（[`handed_over_cards_emptiest_first`]），额度照交的；
/// 按 PCI 总线号对序号、起设备都经 [`super::gpu_cards_by_priority`]。
///
/// # Panics
/// 交的卡一张都用不了：服务交了卡就是要这个用例上卡，用不了不算「本次未跑」。
fn usable_cards_among_those_handed_over(
    test_name: &str,
    handed_over: &[(u32, u64)],
) -> Vec<GpuCard> {
    let reported: Vec<(u32, u64)> = super::reported_cards_by_pci_bus()
        .values()
        .map(|card| (card.index, card.free_memory_mebibytes))
        .collect();
    let picked = handed_over_cards_emptiest_first(handed_over, &reported);
    let cards = started_cards_or_the_reason_the_handed_over_cards_do_not_run(
        super::gpu_cards_by_priority(&picked),
        handed_over,
        &reported,
    )
    .unwrap_or_else(|reason| panic!("{test_name}：{reason}"));
    println!(
        "GPU_CARDS_PICKED_AMONG_THOSE_HANDED_OVER test={test_name} handed_over={handed_over:?} started_indices={:?}",
        cards.iter().map(|card| card.index).collect::<Vec<u32>>()
    );
    cards
}

/// 取卡、取卡池、借卡的结局：取到了，或这一次未跑、带着「本次未跑」那一行（[`not_run_line`]）。打不打那一行由调用方定：
/// GPU 用例经 `…_or_report_not_run` 那几个入口取，未跑的打出那一行、用例当场返回（门禁照那一行报「本次未跑」）；
/// 钉取卡判法的单测照这个结局断言、不打，那一行打出来门禁就把这条单测当成本次未跑。
#[derive(Debug)]
pub enum TakenOrNotRun<Taken> {
    /// 取到了。
    Taken(Taken),
    /// 这一次未跑：`line` 是「本次未跑」那一行（[`not_run_line`] 打的写法），还没打。
    NotRun { line: String },
}

impl<Taken> TakenOrNotRun<Taken> {
    /// GPU 用例的收法：未跑的把那一行打到标准输出、交 `None`；取到的交 `Some`。
    #[must_use]
    pub fn into_option_printing_the_not_run_line(self) -> Option<Taken> {
        match self {
            Self::Taken(taken) => Some(taken),
            Self::NotRun { line } => {
                println!("{line}");
                None
            }
        }
    }
}

/// 取卡：照调用方给这个进程的显卡配置（[`GpuConfigurationOfTheCaller::of_this_process`]）取，判法见
/// [`usable_cards_of_the_configuration_or_the_not_run_line`]；未跑的打「本次未跑」那一行、交回 `None`（调用的用例当场返回）。
///
/// # Panics
/// 同 [`usable_cards_of_the_configuration_or_the_not_run_line`]。
#[must_use]
pub fn usable_cards_or_report_not_run(test_name: &str) -> Option<Vec<GpuCard>> {
    usable_cards_of_the_configuration_or_the_not_run_line(
        test_name,
        GpuConfigurationOfTheCaller::of_this_process(),
    )
    .into_option_printing_the_not_run_line()
}

/// 照 `configuration` 取卡（[`GpuConfigurationOfTheCaller::cards_for_the_gpu_tests`]）：服务经环境变量 `GPU_CARDS_BY_PRIORITY` 交了卡
/// 就只在交给这一台的卡里挑，空闲显存从大到小，每张的额度照交的，同一台别的卡可能分给了别的任务，一个设备都不起。
/// 没交卡串（没设、空串或只有空白）：当这台机器没有卡，交回未跑与 [`not_run_line`] 那一行（不打），
/// 不借卡、不找本机的卡、不起设备。
///
/// # Panics
/// 服务交的卡读不成（写法不对、地址不在 `HOSTS` 里、多机配置判法认不出这一台），或交的卡一张都用不了。
#[must_use]
pub(crate) fn usable_cards_of_the_configuration_or_the_not_run_line(
    test_name: &str,
    configuration: &GpuConfigurationOfTheCaller,
) -> TakenOrNotRun<Vec<GpuCard>> {
    match configuration
        .cards_for_the_gpu_tests()
        .unwrap_or_else(|reason| panic!("{test_name}：服务交的卡认不成：{reason}"))
    {
        CardsOfTheCallerForTheGpuTests::NotHandedOver { reason_not_run } => TakenOrNotRun::NotRun {
            line: not_run_line(test_name, &reason_not_run),
        },
        CardsOfTheCallerForTheGpuTests::HandedOverByTheService(handed_out) => TakenOrNotRun::Taken(
            usable_cards_among_those_handed_over(test_name, &handed_out.cards_of_this_host()),
        ),
    }
}

/// 空闲显存最大的那张卡，服务交了卡时限在交的卡里；别的同 [`usable_cards_or_report_not_run`]。
///
/// # Panics
/// 同 [`usable_cards_or_report_not_run`]。
#[must_use]
pub fn first_usable_card_or_report_not_run(test_name: &str) -> Option<GpuCard> {
    usable_cards_or_report_not_run(test_name).map(|cards| {
        cards.into_iter().next().expect(
            "usable_cards_or_report_not_run 交 Some 只在服务交了卡、起好的卡数大于 0 时（started_cards_or_the_reason_the_handed_over_cards_do_not_run），交回的卡表不会是空的",
        )
    })
}

/// 这个进程里卡池的借据由哪几条线程借着、哪几条线程正在借，与账变过几回（[`CardLeaseWaitingRoom`] 护着它）。
struct LeasesOfTheCardPoolInThisProcess {
    /// 每条线程借着几笔（借据丢掉时减；减到 0 的线程不留在表里）。
    leases_out_by_thread: std::collections::HashMap<std::thread::ThreadId, u64>,
    /// 每条线程正在借的有几次：从照账借那一步起头到交成借据或交回借不到为止，账上记着的那一笔还没数进借着的。
    attempts_in_progress_by_thread: std::collections::HashMap<std::thread::ThreadId, u64>,
    /// 一笔借据还回来、或一次借的那一步做完，各加一：排队的用例照它认「自己那一次借之后账变过没有」。
    changes_so_far: u64,
}

/// `thread` 那一格加一（没有就起一格）。
fn add_one_for_the_thread(
    counts_by_thread: &mut std::collections::HashMap<std::thread::ThreadId, u64>,
    thread: std::thread::ThreadId,
) {
    *counts_by_thread.entry(thread).or_insert(0) += 1;
}

/// `thread` 那一格减一，减到 0 就去掉那一格。
///
/// # Panics
/// `thread` 没有那一格（加一与减一没配对）。
fn take_one_for_the_thread(
    counts_by_thread: &mut std::collections::HashMap<std::thread::ThreadId, u64>,
    thread: std::thread::ThreadId,
) {
    let count = counts_by_thread
        .get_mut(&thread)
        .expect("每一笔只减一次：加的时候这条线程那一格加过一，减到 0 的格已经去掉");
    *count -= 1;
    if *count == 0 {
        counts_by_thread.remove(&thread);
    }
}

impl LeasesOfTheCardPoolInThisProcess {
    /// `thread` 此刻借不到时有没有可等的：它自己一笔都没借着，而这个进程别的线程借着卡或正在借。
    /// 它自己借着的那一笔要它自己还，等它就等不到头，所以借着卡的线程再借不等；别的线程借着的每一笔丢掉时都还，
    /// 正在借的每一次都做完（panic 了也算做完，[`AttemptInTheWaitingRoom`]），等它们等得到头。
    fn is_there_anything_for_the_thread_to_wait_for(&self, thread: std::thread::ThreadId) -> bool {
        !self.leases_out_by_thread.contains_key(&thread)
            && self
                .leases_out_by_thread
                .keys()
                .chain(self.attempts_in_progress_by_thread.keys())
                .any(|other_thread| *other_thread != thread)
    }

    /// `thread` 正在借的那一次做完：不再算它在借，账变过加一。
    fn end_an_attempt_of(&mut self, thread: std::thread::ThreadId) {
        take_one_for_the_thread(&mut self.attempts_in_progress_by_thread, thread);
        self.changes_so_far += 1;
    }
}

/// GPU 用例向卡池借卡时排的队（「放开并按显存排队」）：同一个测试二进制里几条用例并行起，每条按它要的显存
/// 照卡池那一份账借（[`GpuCardPool::lease_card_or_report_not_run`]），此刻借不到就等这个进程别的线程还卡、或别的线程借的那一步做完，
/// 每回醒来照账再借一次，借到了用完还。账只有卡池那一份（额度照多机配置清场之后那一份显卡与显存，`singlefs_verification::gpu_memory_ledger`），
/// 这里不另记显存，只记借据在谁手里、谁在借、账变过几回。只等这个进程里的：别的进程占着的不等（`singlefs_verification::gpu_memory_ledger`
/// 文件头：只记这个进程借的，不判别的进程活没活，也不等），所以这个进程别的线程一笔都没借着、也没在借时照旧报本次未跑；
/// 借着卡的线程再借不等（[`LeasesOfTheCardPoolInThisProcess::is_there_anything_for_the_thread_to_wait_for`]）。
/// 要的比每张卡空着时能借的都大，等多久都放不下，不等、交调用方判红。
/// 只按显存排队，不按主机内存排队：用例的并发数是测试线程数，取登记的每台默认重活线程数（按内存限并发的入口
/// 只登记在 `.claude/memory-bounded-parallelism.tsv`，GPU 用例不在里面）。
pub struct CardLeaseWaitingRoom {
    leases: std::sync::Mutex<LeasesOfTheCardPoolInThisProcess>,
    the_books_changed: std::sync::Condvar,
}

/// 照账借一次的结局（[`CardLeaseWaitingRoom::lease_waiting_for_the_other_threads`] 的 `attempt` 交它）。
pub enum LeaseAttempt<Lease> {
    /// 借到了：交回之前已经数进借着的（[`CardLeaseWaitingRoom::count_a_lease_out`]）。
    Leased(Lease),
    /// 此刻每张卡都放不下，空着时放得下；`reason` 是「本次未跑」那一行的原因。
    NoRoomNow { reason: String },
    /// 要的比每张卡空着时能借的都大：等多久都放不下；`reason` 写要多少、最大的一张空着时借得出多少。
    LargerThanEveryCard { reason: String },
}

/// 排队借的结局（[`CardLeaseWaitingRoom::lease_waiting_for_the_other_threads`]）。
pub enum LeaseAfterWaiting<Lease> {
    Leased(Lease),
    /// 此刻借不到，也没有可等的：这个进程别的线程一笔都没借着、也没在借，或借的这条线程自己借着卡。调用方报本次未跑。
    NoRoomAndNothingToWaitFor {
        reason: String,
    },
    /// 要的比每张卡空着时能借的都大，没有等。调用方判红。
    LargerThanEveryCard {
        reason: String,
    },
}

/// 一条线程正在借的那一次（[`CardLeaseWaitingRoom::begin_an_attempt`]）：做完时不再算它在借、账变过加一、叫醒排队的；
/// 借的那一步 panic 了、在展开中丢掉它也一样，排队的不会等一次永远做不完的借。
struct AttemptInTheWaitingRoom<'room> {
    room: &'room CardLeaseWaitingRoom,
    thread: std::thread::ThreadId,
    is_ended: bool,
}

impl<'room> AttemptInTheWaitingRoom<'room> {
    /// 做完这一次，交回账的锁（调用方接着在同一把锁里判等不等）。
    fn end_holding_the_lock(
        mut self,
    ) -> std::sync::MutexGuard<'room, LeasesOfTheCardPoolInThisProcess> {
        let room = self.room;
        let mut leases = room.leases_locked();
        leases.end_an_attempt_of(self.thread);
        self.is_ended = true;
        room.the_books_changed.notify_all();
        leases
    }
}

impl Drop for AttemptInTheWaitingRoom<'_> {
    fn drop(&mut self) {
        if !self.is_ended {
            self.room.leases_locked().end_an_attempt_of(self.thread);
            self.room.the_books_changed.notify_all();
        }
    }
}

/// 数进 [`CardLeaseWaitingRoom`] 的一笔借据（[`CardLeaseWaitingRoom::count_a_lease_out`]）：丢掉时借它的那条线程借着的减一、
/// 账变过加一、叫醒排队的。放在借据最后一个字段：借据别的字段（连账上那一笔）都还了才丢它，醒来的用例照账借得到这一份。
pub struct LeaseCountedOutOfTheWaitingRoom<'room> {
    room: &'room CardLeaseWaitingRoom,
    thread: std::thread::ThreadId,
}

impl Drop for LeaseCountedOutOfTheWaitingRoom<'_> {
    fn drop(&mut self) {
        let mut leases = self.room.leases_locked();
        take_one_for_the_thread(&mut leases.leases_out_by_thread, self.thread);
        leases.changes_so_far += 1;
        drop(leases);
        self.room.the_books_changed.notify_all();
    }
}

impl CardLeaseWaitingRoom {
    /// 一笔借据都没借出去、也没有线程在借。
    #[must_use]
    pub fn with_no_lease_out() -> Self {
        Self {
            leases: std::sync::Mutex::new(LeasesOfTheCardPoolInThisProcess {
                leases_out_by_thread: std::collections::HashMap::new(),
                attempts_in_progress_by_thread: std::collections::HashMap::new(),
                changes_so_far: 0,
            }),
            the_books_changed: std::sync::Condvar::new(),
        }
    }

    fn leases_locked(&self) -> std::sync::MutexGuard<'_, LeasesOfTheCardPoolInThisProcess> {
        // 持锁只为加减几个计数，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
        self.leases
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// 这条线程借到了一笔：数进它借着的，交回的那一份放进借据最后一个字段（丢掉时还回来、叫醒排队的）。
    #[must_use]
    pub fn count_a_lease_out(&self) -> LeaseCountedOutOfTheWaitingRoom<'_> {
        let thread = std::thread::current().id();
        add_one_for_the_thread(&mut self.leases_locked().leases_out_by_thread, thread);
        LeaseCountedOutOfTheWaitingRoom { room: self, thread }
    }

    /// 这条线程起头借一次：算它在借，交回那一次与起头时账变过几回。
    fn begin_an_attempt(&self) -> (AttemptInTheWaitingRoom<'_>, u64) {
        let thread = std::thread::current().id();
        let mut leases = self.leases_locked();
        add_one_for_the_thread(&mut leases.attempts_in_progress_by_thread, thread);
        let changes_before_the_attempt = leases.changes_so_far;
        drop(leases);
        (
            AttemptInTheWaitingRoom {
                room: self,
                thread,
                is_ended: false,
            },
            changes_before_the_attempt,
        )
    }

    /// 照 `attempt` 借（它照卡池那一份账借一次）；此刻借不到时有可等的
    /// （[`LeasesOfTheCardPoolInThisProcess::is_there_anything_for_the_thread_to_wait_for`]），就等到账变过再借一次，没有就交回。
    /// 每一圈要么交回，要么等到账变过再借：可等的那些借据丢掉时都还、正在借的都做完，等得到头。
    pub fn lease_waiting_for_the_other_threads<Lease>(
        &self,
        mut attempt: impl FnMut() -> LeaseAttempt<Lease>,
    ) -> LeaseAfterWaiting<Lease> {
        match self.lease_waiting_for_the_other_threads_keeping_the_refusal(|| match attempt() {
            LeaseAttempt::Leased(lease) => {
                AttemptKeepingTheRefusal::Settled(LeaseAfterWaiting::Leased(lease))
            }
            LeaseAttempt::LargerThanEveryCard { reason } => {
                AttemptKeepingTheRefusal::Settled(LeaseAfterWaiting::LargerThanEveryCard { reason })
            }
            LeaseAttempt::NoRoomNow { reason } => AttemptKeepingTheRefusal::NoRoomNow(reason),
        }) {
            Ok(settled) => settled,
            Err(reason) => LeaseAfterWaiting::NoRoomAndNothingToWaitFor { reason },
        }
    }

    /// 同 [`Self::lease_waiting_for_the_other_threads`]，借不到的结局照 `attempt` 给的原样交回（调用方照它自己的拒绝类型报）：
    /// `attempt` 交 [`AttemptKeepingTheRefusal::Settled`] 当场交回 `Ok`（借到了，或等多久都借不到的拒绝，由调用方装在里面）；
    /// 交 [`AttemptKeepingTheRefusal::NoRoomNow`] 时有可等的就等到账变过再借一次，没有可等的交 `Err`（最后那一次此刻借不到的拒绝）。
    ///
    /// # Errors
    /// 此刻借不到，也没有可等的：最后那一次 `attempt` 给的拒绝。
    pub fn lease_waiting_for_the_other_threads_keeping_the_refusal<Settled, RefusedForNow>(
        &self,
        mut attempt: impl FnMut() -> AttemptKeepingTheRefusal<Settled, RefusedForNow>,
    ) -> Result<Settled, RefusedForNow> {
        let this_thread = std::thread::current().id();
        loop {
            let (attempt_in_progress, changes_before_the_attempt) = self.begin_an_attempt();
            let attempted = attempt();
            let mut leases = attempt_in_progress.end_holding_the_lock();
            let refused_for_now = match attempted {
                AttemptKeepingTheRefusal::Settled(settled) => return Ok(settled),
                AttemptKeepingTheRefusal::NoRoomNow(refused_for_now) => refused_for_now,
            };
            // 这一次借做完时自己加了一回：比它多才是别的线程还了卡或做完了借，那时这一次借不到的账可能已经变了，再借
            let changes_after_this_attempt = changes_before_the_attempt + 1;
            while leases.changes_so_far == changes_after_this_attempt {
                if !leases.is_there_anything_for_the_thread_to_wait_for(this_thread) {
                    return Err(refused_for_now);
                }
                leases = self
                    .the_books_changed
                    .wait(leases)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
        }
    }
}

/// 照账借一次、拒绝留原样的结局（[`CardLeaseWaitingRoom::lease_waiting_for_the_other_threads_keeping_the_refusal`] 的 `attempt` 交它）。
pub enum AttemptKeepingTheRefusal<Settled, RefusedForNow> {
    /// 定了，不用等：借到了（交回之前已经数进借着的），或等多久都借不到。
    Settled(Settled),
    /// 此刻借不到，账变过可能借得到：带着这一次的拒绝。
    NoRoomNow(RefusedForNow),
}

/// GPU 用例共用的卡池（[`card_pool::GpuCardPool`]）：只开 `gpu` 也编得进（显卡配置的读法在 `crate::gpu_card_configuration`）。
/// 用例经 [`gpu_card_pool_or_report_not_run`] 取的是第一次借卡才起池的把手（[`GpuCardPoolStartedAtTheFirstLease`]）。
pub use card_pool::{
    card_pool_not_run_reason_in_this_process, card_pool_of_this_process_started_now,
    card_pool_started_in_this_process, CardPoolLeaseRefusal, GpuCardLease, GpuCardPool,
    CARD_QUOTA_ENVIRONMENT_VARIABLE,
};

/// 这台机器上有没有 NVIDIA 的卡，只看 Vulkan 枚举与 PCI 设备表（[`decide_gpu_test_start`] 在一张卡都还没数的时候判的那一份）：
/// 不读显卡配置、不领卡、不起卡池。GPU 用例开头在调用方交了卡串之后照它判「本次未跑」（[`gpu_card_pool_or_report_not_run`]），卡池等第一次借卡时才起。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NvidiaHardwareOfThisMachine {
    /// Vulkan 枚举里没有 NVIDIA 独显，PCI 上也没有 NVIDIA 显示控制器（或看不见 PCI 总线）：起池也只会判成没卡，报「本次未跑」。
    /// `reason` 是「本次未跑」那一行的原因。
    Absent { reason: String },
    /// 有 NVIDIA 独显，或 PCI 上有 NVIDIA 显示控制器而 Vulkan 枚举不出、或 PCI 设备表读不出：用不用得上、点名的卡在不在，
    /// 第一次借卡起池时照调用方的显卡配置判（用不上照旧判红）。
    PresentOrToBeJudgedByTheCardPool,
}

/// 判 [`NvidiaHardwareOfThisMachine`] 时交给 [`decide_gpu_test_start`] 的用得上的卡数：卡池还没起，一张卡都没数。
const USABLE_CARDS_COUNTED_BEFORE_THE_CARD_POOL_STARTS: usize = 0;

/// 这台机器的 [`NvidiaHardwareOfThisMachine`]：进程里第一次问时枚举一次 Vulkan 适配器、读一次 PCI 设备表，之后每次交同一份
/// （卡池起池也是进程里只判一次）。
#[must_use]
pub fn nvidia_hardware_of_this_machine() -> NvidiaHardwareOfThisMachine {
    static NVIDIA_HARDWARE_OF_THIS_MACHINE: std::sync::OnceLock<NvidiaHardwareOfThisMachine> =
        std::sync::OnceLock::new();
    NVIDIA_HARDWARE_OF_THIS_MACHINE
        .get_or_init(|| {
            match decide_gpu_test_start(
                &enumerate_vulkan_adapters(),
                &read_pci_nvidia_display_controllers_on_the_bus(std::path::Path::new(
                    PCI_BUS_DIRECTORY,
                )),
                USABLE_CARDS_COUNTED_BEFORE_THE_CARD_POOL_STARTS,
            ) {
                GpuTestStart::NotRunNoGpuAdapter { reason } => {
                    NvidiaHardwareOfThisMachine::Absent { reason }
                }
                GpuTestStart::RunOnTheUsableCards
                | GpuTestStart::AdapterPresentButNoCardStarted { .. }
                | GpuTestStart::NvidiaCardOnPciButVulkanEnumeratesNone { .. }
                | GpuTestStart::PciBusVisibleButListingUnreadable { .. } => {
                    NvidiaHardwareOfThisMachine::PresentOrToBeJudgedByTheCardPool
                }
            }
        })
        .clone()
}

/// GPU 用例拿的卡池把手：经 [`gpu_card_pool_or_report_not_run`] 拿到时只判过调用方交了卡串、这台机器有 NVIDIA 的卡
/// （`gpu_card_pool_of_the_configuration_or_the_not_run_line`），没读卡串里的卡项、没领卡、没起卡池；
/// 第一次借卡（[`Self::lease_card_or_report_not_run`]）或第一次要池里的卡（[`Self::card_pool_started_now`]）时才起这个进程的卡池
/// （`card_pool::gpu_card_pool_or_the_not_run_line`：照调用方给这个进程的同一份显卡配置挑卡，调用方没交卡串的判成没卡、交回「本次未跑」那一行，不领卡）。
/// CPU 参照过库、只录那一趟交回 `None` 而收手的用例，从头到尾不起池、不领卡。可以拷贝，传进几个帮手、几条线程都是同一个池。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuCardPoolStartedAtTheFirstLease {
    /// 起池时判成本机没卡打「本次未跑」、有卡而用不上 panic 时报的用例名（进程里每个名字只留一份，[`test_name_kept_for_this_process`]）。
    test_name: &'static str,
}

/// `test_name` 在这个进程里留一份、交回它（同一个名字交同一份，留下的份数是这个进程里拿过卡池把手的不同用例名的个数）：
/// 卡池把手要拷贝着传进帮手与线程，用例名跟着它活到进程结束。
fn test_name_kept_for_this_process(test_name: &str) -> &'static str {
    static TEST_NAMES_KEPT: std::sync::Mutex<std::collections::BTreeSet<&'static str>> =
        std::sync::Mutex::new(std::collections::BTreeSet::new());
    let mut kept = TEST_NAMES_KEPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(name) = kept.get(test_name).copied() {
        return name;
    }
    let name: &'static str = Box::leak(test_name.to_string().into_boxed_str());
    kept.insert(name);
    name
}

/// GPU 用例开头取卡池：照调用方给这个进程的显卡配置（[`GpuConfigurationOfTheCaller::of_this_process`]，起池照的也是这一份）
/// 与这台机器的硬件（[`nvidia_hardware_of_this_machine`]）判，判法见 `gpu_card_pool_of_the_configuration_or_the_not_run_line`：
/// 调用方没交卡串、或这台机器没有 NVIDIA 的卡，就打一行 [`not_run_line`]、交 `None`（调用的用例当场返回）；都有就交卡池把手
/// （[`GpuCardPoolStartedAtTheFirstLease`]），此刻不读卡串里的卡项、不领卡、不起池，第一次借卡时才起。
/// 先判有没有卡、建好表知道要多少显存之后再借（[`GpuCardPoolStartedAtTheFirstLease::lease_card_or_report_not_run`]）。
#[must_use]
pub fn gpu_card_pool_or_report_not_run(
    test_name: &str,
) -> Option<GpuCardPoolStartedAtTheFirstLease> {
    gpu_card_pool_of_the_configuration_of_the_caller_or_the_not_run_line(
        test_name,
        GpuConfigurationOfTheCaller::of_this_process,
        nvidia_hardware_of_this_machine,
    )
    .into_option_printing_the_not_run_line()
}

/// [`gpu_card_pool_or_report_not_run`] 的接线，那一行不打：取一回调用方那一份显卡配置（`configuration_of_the_caller`，取卡池入口交的是
/// [`GpuConfigurationOfTheCaller::of_this_process`]），连同判硬件的那一步交给 `gpu_card_pool_of_the_configuration_or_the_not_run_line` 判，
/// 结局原样交回。读调用方那一份的那一步从参数交进来：钉接线的单测自己定卡串设没设，不看这一趟的环境。
#[must_use]
fn gpu_card_pool_of_the_configuration_of_the_caller_or_the_not_run_line<'configuration>(
    test_name: &str,
    configuration_of_the_caller: impl FnOnce() -> &'configuration GpuConfigurationOfTheCaller,
    hardware_of_this_machine: impl FnOnce() -> NvidiaHardwareOfThisMachine,
) -> TakenOrNotRun<GpuCardPoolStartedAtTheFirstLease> {
    gpu_card_pool_of_the_configuration_or_the_not_run_line(
        test_name,
        configuration_of_the_caller(),
        hardware_of_this_machine,
    )
}

/// 拿卡池把手时交给 [`GpuConfigurationOfTheCaller::cards_for_the_gpu_tests_given`] 的「这一台是 `HOSTS` 里哪一项」：不起多机配置判法、
/// 交回这一句错。拿把手只问交没交卡串，卡项读不读得成、这一台认不认得出，起池时照同一份配置判（`card_pool::card_pool_start_outcome_on`）。
const HOSTS_NOT_READ_WHEN_TAKING_THE_CARD_POOL_HANDLE: &str =
    "拿卡池把手时不认这一台是 HOSTS 里哪一项：卡项留给起池时读";

/// GPU 用例开头取卡池的判法（[`gpu_card_pool_or_report_not_run`] 照它判，结局交回、那一行不打）：
/// `configuration` 没交卡串（没设、空串或只有空白，[`GpuConfigurationOfTheCaller::cards_for_the_gpu_tests_given`] 交
/// [`CardsOfTheCallerForTheGpuTests::NotHandedOver`]）就当这台机器没有卡，交回未跑与「本次未跑」那一行（原因与起池判成没卡的同一句），
/// 不调 `hardware_of_this_machine`、不找本机的卡、不起池；交了卡串（读不读得成都算交了，卡项在起池时读）才照 `hardware_of_this_machine`
/// 判这台机器有没有 NVIDIA 的卡：没有交回未跑与那一行，有就交卡池把手。拿把手与起池照的是同一份配置，交了卡串、拿把手时判过有卡而起池时判成没卡
/// 只剩硬件前后不一致那一种（[`GpuCardPoolStartedAtTheFirstLease::card_pool_started_now`] 照旧 panic）。
#[must_use]
fn gpu_card_pool_of_the_configuration_or_the_not_run_line(
    test_name: &str,
    configuration: &GpuConfigurationOfTheCaller,
    hardware_of_this_machine: impl FnOnce() -> NvidiaHardwareOfThisMachine,
) -> TakenOrNotRun<GpuCardPoolStartedAtTheFirstLease> {
    let cards_of_the_caller = configuration.cards_for_the_gpu_tests_given(&|| {
        Err(HOSTS_NOT_READ_WHEN_TAKING_THE_CARD_POOL_HANDLE.to_string())
    });
    match cards_of_the_caller {
        Ok(CardsOfTheCallerForTheGpuTests::NotHandedOver { reason_not_run }) => {
            TakenOrNotRun::NotRun {
                line: not_run_line(test_name, &reason_not_run),
            }
        }
        Ok(CardsOfTheCallerForTheGpuTests::HandedOverByTheService(_)) | Err(_) => {
            // 交了卡串而读不成（`Err`）也往下判硬件：卡项读不读得成留给起池那一步判，拿把手这一步不读也不报。
            match hardware_of_this_machine() {
                NvidiaHardwareOfThisMachine::Absent { reason } => TakenOrNotRun::NotRun {
                    line: not_run_line(test_name, &reason),
                },
                NvidiaHardwareOfThisMachine::PresentOrToBeJudgedByTheCardPool => {
                    TakenOrNotRun::Taken(GpuCardPoolStartedAtTheFirstLease {
                        test_name: test_name_kept_for_this_process(test_name),
                    })
                }
            }
        }
    }
}

/// 同 [`gpu_card_pool_or_report_not_run`]，这台机器有没有 NVIDIA 的卡由调用方给（不枚举 Vulkan、不读 PCI），也不看调用方交没交卡串：
/// 钉「拿到把手时不起池、不领卡」的用例在没卡、没交卡串的机器上也给「有」，走得到借卡起池那一步。
#[must_use]
pub fn gpu_card_pool_or_report_not_run_given_the_hardware(
    test_name: &str,
    hardware: &NvidiaHardwareOfThisMachine,
) -> Option<GpuCardPoolStartedAtTheFirstLease> {
    match hardware {
        NvidiaHardwareOfThisMachine::Absent { reason } => {
            println!("{}", not_run_line(test_name, reason));
            None
        }
        NvidiaHardwareOfThisMachine::PresentOrToBeJudgedByTheCardPool => {
            Some(GpuCardPoolStartedAtTheFirstLease {
                test_name: test_name_kept_for_this_process(test_name),
            })
        }
    }
}

impl GpuCardPoolStartedAtTheFirstLease {
    /// 这个进程的卡池，没起过就在这一刻起（`card_pool::gpu_card_pool_or_the_not_run_line`）：判成本机没卡交回未跑与
    /// 「本次未跑」那一行（不打）。
    ///
    /// # Panics
    /// 同 `card_pool::gpu_card_pool_or_the_not_run_line`：有卡而用不上、显卡配置写错、点名的卡少一张找不到。
    fn card_pool_or_the_not_run_line(self) -> TakenOrNotRun<&'static GpuCardPool> {
        card_pool::gpu_card_pool_or_the_not_run_line(self.test_name)
    }

    /// 向这个进程的卡池借一张卡（[`GpuCardPool::lease_card_or_report_not_run`]）；卡池还没起的在这一刻起。
    /// 起池判成本机没卡、或借不到：打「本次未跑」那一行，交 `None`。
    ///
    /// # Panics
    /// 同 [`Self::lease_card_or_the_not_run_line`]。
    #[must_use]
    pub fn lease_card_or_report_not_run(
        self,
        test_name: &str,
        need_mebibytes: u64,
    ) -> Option<GpuCardLease> {
        self.lease_card_or_the_not_run_line(test_name, need_mebibytes)
            .into_option_printing_the_not_run_line()
    }

    /// 同 [`Self::lease_card_or_report_not_run`]，起池判成本机没卡、或借不到时交回未跑与「本次未跑」那一行（不打）：
    /// 钉起池与借卡判法的用例照这个结局断言。
    ///
    /// # Panics
    /// 同 [`Self::card_pool_started_now`] 起池那一步与 [`GpuCardPool::lease_card_or_the_not_run_line`]。
    #[must_use]
    pub fn lease_card_or_the_not_run_line(
        self,
        test_name: &str,
        need_mebibytes: u64,
    ) -> TakenOrNotRun<GpuCardLease> {
        match self.card_pool_or_the_not_run_line() {
            TakenOrNotRun::Taken(pool) => {
                pool.lease_card_or_the_not_run_line(test_name, need_mebibytes)
            }
            TakenOrNotRun::NotRun { line } => TakenOrNotRun::NotRun { line },
        }
    }

    /// 池里存储缓冲绑定上限最小的那张卡的上限（[`GpuCardPool::smallest_maximum_storage_buffer_binding_size`]）；卡池还没起的在这一刻起。
    ///
    /// # Panics
    /// 同 [`Self::card_pool_started_now`]。
    #[must_use]
    pub fn smallest_maximum_storage_buffer_binding_size(self) -> u64 {
        self.card_pool_started_now()
            .smallest_maximum_storage_buffer_binding_size()
    }

    /// 这个进程的卡池，没起过就在这一刻起：要池里有哪几张卡、每张能借多少的用例与帮手经它取。
    ///
    /// # Panics
    /// 起池判成本机没卡：经 [`gpu_card_pool_or_report_not_run`] 拿到把手时判过调用方交了卡串、这台机器有 NVIDIA 的卡，起池照同一份配置
    /// 却判成没有（同一个进程里 Vulkan 枚举与 PCI 设备表前后不一致），或用例经 [`gpu_card_pool_or_report_not_run_given_the_hardware`]
    /// 在没卡或没交卡串的机器上给了「有」；起池那一步的 panic 同 `card_pool::gpu_card_pool_or_the_not_run_line`。
    /// 调用方没交卡串的不走到这里：拿把手那一步就交回本次未跑（`gpu_card_pool_of_the_configuration_or_the_not_run_line`）。
    #[must_use]
    pub fn card_pool_started_now(self) -> &'static GpuCardPool {
        match self.card_pool_or_the_not_run_line() {
            TakenOrNotRun::Taken(pool) => pool,
            TakenOrNotRun::NotRun { line } => panic!(
                "{}：拿卡池把手时判过调用方交了卡串、这台机器有 NVIDIA 的卡，起池时却判成没有（起池时的「本次未跑」那一行：{line}）",
                self.test_name
            ),
        }
    }
}

/// 起池那一步（单测在别的用例拿着改环境的锁换了配置时现起一次，核它照的是哪一份显卡配置）。
#[cfg(test)]
pub(in crate::gpu_unit_checks) use card_pool::{
    attempt_keeping_the_refusal_of, card_pool_start_outcome_on, start_the_gpu_card_pool,
    GpuCardPoolStartOutcome,
};

mod card_pool {
    use super::super::{
        card_memory_reader_of_the_card_pool, device_found_idle_by_a_poll,
        device_wait_before_returning_a_lease, memory_budget_mebibytes_of,
        nvidia_discrete_cards_reported_by_nvidia_smi, nvidia_smi_memory_and_processes_report,
        poll_type_before_returning_a_lease, start_the_card, CardStartFailure, DeviceOwnerOnTheCard,
        DiscoveredCard, GpuCard,
    };
    use super::{
        decide_gpu_test_start, enumerate_vulkan_adapters, not_run_line,
        read_pci_nvidia_display_controllers_on_the_bus, AttemptKeepingTheRefusal,
        CardLeaseWaitingRoom, GpuTestStart, LeaseAfterWaiting, LeaseAttempt,
        LeaseCountedOutOfTheWaitingRoom, TakenOrNotRun, PCI_BUS_DIRECTORY,
    };
    use crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
    use crate::gpu_card_configuration::{
        configured_cards_not_usable, CardsOfTheCallerForTheGpuTests, ConfiguredCards,
        GpuConfigurationOfTheCaller,
    };
    use crate::gpu_memory_ledger::{
        card_ledger_capacity_mebibytes, reason_for_a_booking_without_room_now,
        reason_for_running_out_of_gpu_memory, wait_for_the_device_and_take_in_the_outcome,
        BookingRefusal, CardRoomForThisProcess, DeviceStartedOnDemand, DeviceStartedOnTheBooks,
        GpuMemoryBooking, GpuMemoryLedger,
    };
    use crate::numeric_environment_variable::{
        number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
    };

    /// 每张卡账的上限（MiB）：设了，每张卡上这个进程一共能占的取它、配置给这张卡的额度减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]、
    /// 这个进程此刻能占的减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`] 三者里最小的
    /// （`singlefs_verification::gpu_memory_ledger::card_ledger_capacity_mebibytes`）；不是每条用例的上限，借得比它多的照常判「放不下」。
    pub const CARD_QUOTA_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES";

    /// 池里的一张卡：照配置点了名、Vulkan 与 `nvidia-smi` 里都找得到；设备等这个进程在这张卡上借到一笔之后才起
    /// （[`GpuCardPool::started_card`]）：起好了留着给后面借的用，起不来不留、下一次借到再起，设备丢了也重起
    /// （起设备失败不缓存成这个进程永久的错）。
    struct CardOfThePool {
        discovered: DiscoveredCard,
        /// `GPU_CARDS_BY_PRIORITY` 配给这张卡的额度（MiB）；没配显卡优先级的是 `None`。
        configured_card_quota_mebibytes: Option<u64>,
        started: DeviceStartedOnDemand<GpuCard>,
    }

    /// 向卡池借、借不到的两种（崩溃放量产品路径照它报）。
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum CardPoolLeaseRefusal {
        /// 账上借不到（`singlefs_verification::gpu_memory_ledger::BookingRefusal`）。
        Booking(BookingRefusal),
        /// 账上借到了，那张卡上起设备时显存不够（[`CardStartFailure::OutOfGpuMemory`]）：那一笔已经还了，调用方报本次未跑。
        /// `reason` 是「本次未跑」那一行的原因（以「显存不够：」起头）。
        OutOfGpuMemoryStartingTheDevice { reason: String },
    }

    /// 向卡池借一次的结局在排队时等不等（[`GpuCardPool::lease_the_first_card_with_room_waiting_for_the_other_threads`]）：
    /// 账上每张卡此刻都放不下、借到的卡上起设备时显存不够是此刻借不到（别的线程还了卡可能借得到，等）；
    /// 借到了、要的比每张卡空着时能借的都大、卡上的显存读不出都定了（不等）。
    pub(in crate::gpu_unit_checks) fn attempt_keeping_the_refusal_of<Lease>(
        leased: Result<Lease, CardPoolLeaseRefusal>,
    ) -> AttemptKeepingTheRefusal<Result<Lease, CardPoolLeaseRefusal>, CardPoolLeaseRefusal> {
        match leased {
            Ok(lease) => AttemptKeepingTheRefusal::Settled(Ok(lease)),
            Err(
                refusal @ (CardPoolLeaseRefusal::Booking(BookingRefusal::NoCardHasRoomNow {
                    ..
                })
                | CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice { .. }),
            ) => AttemptKeepingTheRefusal::NoRoomNow(refusal),
            Err(
                refusal @ CardPoolLeaseRefusal::Booking(
                    BookingRefusal::NeedLargerThanEveryCard(_)
                    | BookingRefusal::CardMemoryUnreadable { .. },
                ),
            ) => AttemptKeepingTheRefusal::Settled(Err(refusal)),
        }
    }

    /// GPU 用例共用的卡：这台机器点名的每张卡在这个进程里一本显存账（[`GpuMemoryLedger`]，只记这个进程借的，
    /// 每次借现读卡上此刻的显存：`singlefs_verification::gpu_memory_ledger` 文件头），每张卡这个进程只起一个设备；
    /// 用例按自己实际要的量借（[`GpuCardPool::lease_card_or_report_not_run`]），借据丢掉时还。
    /// 一张卡这个进程一共能占的照 [`card_ledger_capacity_mebibytes`]：驱动的上下文与编好的内核每个进程在每张卡上付一份
    /// [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]。起池不读卡上此刻空闲多少，能不能借在借的那一刻判；这个进程第一次在一张卡上借到
    /// （连它的驱动那一份）了，才在这张卡上起设备，没借过的卡不起。
    /// 用例借不到时排队等这个进程别的线程还卡（`waiting_room`，[`CardLeaseWaitingRoom`]）；池借出去的每一笔都数进它。
    pub struct GpuCardPool {
        cards: Vec<CardOfThePool>,
        ledger: GpuMemoryLedger,
        waiting_room: CardLeaseWaitingRoom,
    }

    /// 借来的一张卡：[`GpuCardLease::card`] 与池里那张共用设备、编好的内核，显存额度与空闲改成借的量加 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]
    /// （判器照「额度 − 表 − 那一份」开草稿区，开出来正好是借的那么多）。丢掉时先等设备把这张卡上提交了的活做完、丢掉的缓冲交回驱动，
    /// 再销账（照 [`poll_type_before_returning_a_lease`] 等；等不成照旧记在账上）。字段照声明的次序丢：卡、账上的那一台、借据、
    /// 数进排队的那一笔（账上那一笔销了才叫醒排队的用例）。
    pub struct GpuCardLease {
        card: GpuCard,
        /// 借据那张卡上的设备在账上的那一台（`card` 里那一份的克隆，借据等的就是它）。
        device_on_the_books: std::sync::Arc<DeviceStartedOnTheBooks<'static>>,
        booking: GpuMemoryBooking<'static>,
        /// 不读它，只为丢借据时最后还：[`LeaseCountedOutOfTheWaitingRoom`]。
        _counted_out_of_the_waiting_room: LeaseCountedOutOfTheWaitingRoom<'static>,
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

        /// 这一笔此刻还记在这个进程的账上没有（核账：有卡用例照它判借据活到了用完）。
        #[must_use]
        pub fn is_on_the_books(&self) -> bool {
            self.booking.is_on_the_books()
        }
    }

    impl Drop for GpuCardLease {
        fn drop(&mut self) {
            // wgpu 丢掉的缓冲要等设备做完这张卡上提交了的活才交回驱动：销账之前等完（岔路 G2「改用乙」，
            // E174 两张卡上等完之后下一次读数里那笔缓冲都已不在），等不成（超时、wgpu 报错或 panic）这一笔照旧记在账上；wgpu 把等的时候
            // 设备上的错当致命、panic：接住、记成没等成，不在展开中照样往外抛、已经在展开中不再抛
            // （`singlefs_verification::gpu_memory_ledger::wait_for_the_device_and_take_in_the_outcome`）
            let device = self.card.device();
            let index = self.card.index;
            let lease_identifier = self.booking.lease_identifier();
            let mebibytes = self.booking.mebibytes();
            crate::gpu_unit_checks::gpu_lifecycle_trace(
                &self.card,
                "lease_return_wait_begin",
                format_args!("lease={lease_identifier} mebibytes={mebibytes}"),
            );
            wait_for_the_device_and_take_in_the_outcome(
                &mut self.booking,
                &self.device_on_the_books,
                || device.poll(poll_type_before_returning_a_lease()),
                device_wait_before_returning_a_lease,
                |polled| {
                    crate::gpu_unit_checks::gpu_lifecycle_trace(
                        &self.card,
                        "lease_return_wait_failed",
                        format_args!("lease={lease_identifier}"),
                    );
                    eprintln!(
                        "GPU_CARD_LEASE_RETURN_WAIT_FAILED index={index} lease={lease_identifier} mebibytes={mebibytes} polled={:?}：\
                             这一笔照旧记在账上，到那台设备上提交的活做完了、或那台设备记成丢了、放掉了才销",
                        polled.as_ref().map_err(|_panic| "等的时候 wgpu panic 了")
                    );
                    self.card
                        .device_condition()
                        .print_the_device_lost_marker_line_the_first_time(&format!(
                            "借据 {lease_identifier} 丢掉时等设备没等成：{:?}",
                            polled.as_ref().map_err(|_panic| "等的时候 wgpu panic 了")
                        ));
                },
            );
            crate::gpu_unit_checks::gpu_lifecycle_trace(
                &self.card,
                "lease_return_handler_end",
                format_args!("lease={lease_identifier}"),
            );
        }
    }

    impl GpuCardPool {
        /// 池里每张卡存储缓冲绑定上限里最小的那个（用例借之前按它估一条草稿缓冲装几个状态）。设备按适配器的上限要
        /// （`start_the_card`），没起设备也照适配器读得到。
        /// 池里有几张卡。
        #[must_use]
        pub fn card_count(&self) -> usize {
            self.cards.len()
        }

        #[must_use]
        pub fn smallest_maximum_storage_buffer_binding_size(&self) -> u64 {
            self.cards
                .iter()
                .map(|card| {
                    card.discovered
                        .adapter
                        .limits()
                        .max_storage_buffer_binding_size
                })
                .min()
                .expect("池里至少一张卡：没有卡的机器上起不出池（gpu_card_pool_or_report_not_run）")
        }

        /// 借之前：列出账上有等设备没等成而照旧记着的借据的卡、连同那一刻照旧记着的那几笔，对池里那张卡留着的设备 poll 一次（不等）；
        /// 交回队列空了，那台设备上到 poll 那一刻为止提交的活都做完了、丢掉的缓冲交回了驱动，列出来的那几笔销账（
        /// 等超时的那一笔不永久少借，两次超时之间没有别的等成时也出得来）。留着的设备丢了的不 poll
        /// （wgpu 在丢了的设备上 poll 会 panic），当场告诉账、照重起记（那几笔销账、这张卡照分不出算），不等借到之后走到重起。
        /// 取留着的设备不等别的线程起设备（`DeviceStartedOnDemand::started_now`）。
        /// 不在账的锁里 poll，只销列卡那一刻记着的：它们丢掉的缓冲在列卡之前提交、丢掉，poll 交回队列空了之后什么时候销都不多给；
        /// 列卡之后才记成没等成的那一笔不销，它的活可以是 poll 之后才提交的。
        /// 列卡、丢没丢、poll 的结局怎么记都在 `gpu_memory_ledger`（`GpuMemoryLedger::take_in_the_devices_kept_before_booking`，不要卡就测得到），
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
        /// （`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`）。
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

        /// 借 `need_mebibytes`：照卡的优先级记在第一张此刻放得下的卡上（前一张放满了才用下一张）；此刻每张都放不下时排队
        /// （[`CardLeaseWaitingRoom`]）：这个进程别的线程借着卡或正在借，就等到它们还了卡或借完再照账借一次；
        /// 没有可等的（别的线程一笔都没借着也没在借，卡上是别的 GPU 用户占着；或这条线程自己借着卡）打 [`not_run_line`] 那一行
        /// （原因写显存不够）、交回 `None`（调用的用例当场返回）。每借到一次打一行 `GPU_CARD_LEASE test=… index=… mebibytes=… room_mebibytes=…`。
        ///
        /// 借到的卡上起设备时显存不够同借不到：有可等的就等，没有就报「本次未跑」（原因写显存不够）。
        ///
        /// # Panics
        /// 要的比每张卡空着时能借的都大（配了 GPU 就一定用 GPU，放不下判红，不悄悄换小，也不等）；卡上的显存读不出；
        /// 借到的卡上起不了设备（显存不够之外的原因）；要 0 MiB。
        #[must_use]
        pub fn lease_card_or_report_not_run(
            &'static self,
            test_name: &str,
            need_mebibytes: u64,
        ) -> Option<GpuCardLease> {
            self.lease_card_or_the_not_run_line(test_name, need_mebibytes)
                .into_option_printing_the_not_run_line()
        }

        /// 同 [`Self::lease_card_or_report_not_run`]，没有可等的时交回未跑与「本次未跑」那一行（不打）。
        ///
        /// # Panics
        /// 同 [`Self::lease_card_or_report_not_run`]。
        #[must_use]
        pub fn lease_card_or_the_not_run_line(
            &'static self,
            test_name: &str,
            need_mebibytes: u64,
        ) -> TakenOrNotRun<GpuCardLease> {
            match self
                .waiting_room
                .lease_waiting_for_the_other_threads(|| self.lease_card(test_name, need_mebibytes))
            {
                LeaseAfterWaiting::Leased(lease) => TakenOrNotRun::Taken(lease),
                LeaseAfterWaiting::NoRoomAndNothingToWaitFor { reason } => TakenOrNotRun::NotRun {
                    line: not_run_line(test_name, &reason),
                },
                LeaseAfterWaiting::LargerThanEveryCard { reason } => {
                    panic!("{test_name}：{reason}")
                }
            }
        }

        /// 照账借一次：借到交借来的卡；此刻借不到、借到的卡上起设备时显存不够交「此刻借不到」与一句为什么；
        /// 要的比每张卡空着时能借的都大交「比每张卡都大」与要多少、最大的一张借得出多少。
        fn lease_card(
            &'static self,
            test_name: &str,
            need_mebibytes: u64,
        ) -> LeaseAttempt<GpuCardLease> {
            self.take_in_the_devices_found_idle();
            let booking = match self.ledger.book_on_the_first_card_with_room(need_mebibytes) {
                Ok(booking) => booking,
                Err(BookingRefusal::NeedLargerThanEveryCard(refusal)) => {
                    return LeaseAttempt::LargerThanEveryCard {
                        reason: format!(
                            "要借 {} MiB，池里最大的一张卡空着时也只借得出 {} MiB：这台机器的卡放不下这条用例，\
                                 给卡更大的额度（GPU_CARDS_BY_PRIORITY），或缩小用例",
                            refusal.need_mebibytes, refusal.largest_ceiling_mebibytes
                        ),
                    }
                }
                Err(BookingRefusal::NoCardHasRoomNow {
                    need_mebibytes: wanted_mebibytes,
                    card_rooms_mebibytes,
                }) => {
                    return LeaseAttempt::NoRoomNow {
                        reason: reason_for_a_booking_without_room_now(
                            wanted_mebibytes,
                            &card_rooms_mebibytes,
                        ),
                    }
                }
                Err(BookingRefusal::CardMemoryUnreadable { reason }) => {
                    panic!("{test_name}：卡上的显存读不出：{reason}")
                }
            };
            match self.card_lent_for_the_booking(test_name, booking) {
                Ok(lease) => LeaseAttempt::Leased(lease),
                Err(reason) => LeaseAttempt::NoRoomNow { reason },
            }
        }

        /// 同 [`Self::lease_card_or_report_not_run`]（照卡的优先级记在第一张此刻放得下的卡上），借不到不排队、交账的结局、
        /// 由调用方定怎么报（借到的照样数进排队那一份，还的时候叫醒排队的用例）：崩溃放量产品路径在起过卡池的进程里，第 ② 段核对与单元校验和对拍在卡上建的缓冲照它借
        /// （`singlefs_verification::gpu_memory_ledger::scratch_booking_refusal_outcome`：此刻借不到报本次未跑，别的判红）。
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
                .map_err(|reason| CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice { reason })
        }

        /// 同 [`Self::lease_the_first_card_with_room_now`]，此刻借不到时排队（[`CardLeaseWaitingRoom`]）：这个进程别的线程借着卡或正在借，
        /// 就等到它们还了卡或借完再照账借一次；没有可等的才交那一次的拒绝。一个进程里几条线程并行各借各的（给定状态表的判器每条工作线程判一张表）
        /// 走它：自己这个进程借满的卡等得到头，不当成显存不够。要的比每张卡空着时能借的都大、卡上的显存读不出不等，当场交回。
        ///
        /// # Errors
        /// 见 [`CardPoolLeaseRefusal`]。
        ///
        /// # Panics
        /// 借到的卡上起不了设备（显存不够之外的原因）；要 0 MiB。
        pub fn lease_the_first_card_with_room_waiting_for_the_other_threads(
            &'static self,
            run_name: &str,
            need_mebibytes: u64,
        ) -> Result<GpuCardLease, CardPoolLeaseRefusal> {
            self.waiting_room
                .lease_waiting_for_the_other_threads_keeping_the_refusal(|| {
                    attempt_keeping_the_refusal_of(
                        self.lease_the_first_card_with_room_now(run_name, need_mebibytes),
                    )
                })
                .and_then(|settled| settled)
        }

        /// 序号是 `index`（`nvidia-smi` 给的）的那张卡在池里排第几。
        ///
        /// # Panics
        /// 池里没有这张卡（调用方点的卡都取自这个池，[`Self::cards_in_priority_order_as_configured`]）。
        fn position_of_the_card_with_index(&self, index: u32) -> usize {
            self.cards
                .iter()
                .position(|card| card.discovered.reported.index == index)
                .unwrap_or_else(|| {
                    panic!("序号 {index} 的卡不在这个进程的卡池里：产品路径的卡都取自这个池")
                })
        }

        /// 记上了的那一笔交成借来的卡：在那张卡上起设备（还没有起好的设备时，[`Self::started_card`]），与池里那张共用设备，
        /// 额度与空闲改成借的量加驱动那一份；打一行 `GPU_CARD_LEASE`；数进排队那一份（[`CardLeaseWaitingRoom::count_a_lease_out`]）。
        /// 起设备时显存不够：还掉这一笔，交「本次未跑」那一行的原因。
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
                    panic!(
                        "{test_name}：{reason}：账上登记了这个进程，卡却起不来上下文（驱动坏了）"
                    )
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
                _counted_out_of_the_waiting_room: self.waiting_room.count_a_lease_out(),
            })
        }

        /// 这个进程此刻与这几张卡空着时，在 `card_indices` 这几张卡（都是池里的卡）上各能借多少（MiB，现读一次；与 `card_indices` 逐张对上）。
        /// 调用方只有集成测试 `tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`（照池里最小一张卡空着时能借的定对拍判器开多宽）；
        /// 集成测试看不到库里 `#[cfg(test)]` 的条目，所以照 `pub` 编。
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

        /// 序号是 `index` 的那张卡上，借据号 `lease_identifier` 的那一笔此刻还记在这个进程的账上没有（核账）。
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

        /// 序号是 `index` 的那张卡上这个进程起了设备没有（只在借到过的卡上起）。
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
    }

    /// 起池那一次：照哪一份显卡配置挑的卡（调用方给这个进程的那一份）与结局。进程里只判一次，之后每条用例照它。
    pub(in crate::gpu_unit_checks) struct GpuCardPoolStart {
        pub(in crate::gpu_unit_checks) configuration: &'static GpuConfigurationOfTheCaller,
        outcome: GpuCardPoolStartOutcome,
    }

    /// 起池那一次的结局（池放在堆上：池比另两种大得多，结局一个进程只有一份）。
    pub(in crate::gpu_unit_checks) enum GpuCardPoolStartOutcome {
        Started(Box<GpuCardPool>),
        NotRunNoGpuAdapter { reason: String },
        CardsPresentButNotUsable { reason: String },
    }

    /// 这台机器给卡池点名的卡（还没起设备）。
    struct CardsNamedForThisMachine {
        /// 点名的卡与配置给它的额度，按配置的次序。
        cards: Vec<(DiscoveredCard, Option<u64>)>,
        /// 配置点了名、本机 Vulkan 或 `nvidia-smi` 里却找不到的卡（序号，按配置的次序）。
        named_but_not_found: Vec<u32>,
        /// 服务交的卡串读不出、写法不对、没给这台机器点卡：一张卡都不取，原因在这里。
        configuration_refusal: Option<String>,
    }

    impl CardsNamedForThisMachine {
        /// 一张卡都不取，原因是 `reason`。
        fn refused(reason: String) -> Self {
            Self {
                cards: Vec::new(),
                named_but_not_found: Vec::new(),
                configuration_refusal: Some(reason),
            }
        }
    }

    /// 照服务交的卡（`configured`：调用方给这个进程的显卡配置里交了的卡串读成的，[`GpuConfigurationOfTheCaller::cards_for_the_gpu_tests`]；
    /// 没交卡串的不走到这里，[`card_pool_start_outcome_on`]）挑这台机器的卡：这一台地址的那几项（这一台是 `HOSTS` 里哪一项由多机配置判法认，
    /// `ConfiguredCards::cards_of_this_host`），照交的次序与额度（与崩溃放量取卡同一个约定）。只找卡、不起设备，
    /// 也不看此刻空闲多少（能不能借在借的那一刻判）。
    fn cards_named_for_this_machine(configured: &ConfiguredCards) -> CardsNamedForThisMachine {
        let mine = configured.cards_of_this_host();
        if mine.is_empty() {
            return CardsNamedForThisMachine::refused(format!(
                "服务交的 GPU_CARDS_BY_PRIORITY 里没有这一台（HOSTS 里的 {}）的卡",
                configured.this_host().as_str()
            ));
        }
        let mut discovered = nvidia_discrete_cards_reported_by_nvidia_smi();
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
        let found: Vec<u32> = cards.iter().map(|(card, _)| card.reported.index).collect();
        CardsNamedForThisMachine {
            named_but_not_found: configured_cards_not_usable(&mine, &found),
            cards,
            configuration_refusal: None,
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
        // 每次借读一份 nvidia-smi 报告，空闲与这个进程占着的出自同一份
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
        GpuCardPool {
            cards,
            ledger,
            waiting_room: CardLeaseWaitingRoom::with_no_lease_out(),
        }
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

    /// 照 `configuration` 挑卡、判有没有卡、开账：起池那一次的结局。调用方没交卡串（没设、空串或只有空白）的当这台机器没有卡
    /// （[`GpuConfigurationOfTheCaller::cards_for_the_gpu_tests`]）：判成没卡、报「本次未跑」，不起池、不领卡、不找本机的卡。
    pub(in crate::gpu_unit_checks) fn card_pool_start_outcome_on(
        configuration: &GpuConfigurationOfTheCaller,
    ) -> GpuCardPoolStartOutcome {
        let named = match configuration.cards_for_the_gpu_tests() {
            Ok(CardsOfTheCallerForTheGpuTests::NotHandedOver { reason_not_run }) => {
                return GpuCardPoolStartOutcome::NotRunNoGpuAdapter {
                    reason: reason_not_run,
                };
            }
            Ok(CardsOfTheCallerForTheGpuTests::HandedOverByTheService(configured)) => {
                cards_named_for_this_machine(&configured)
            }
            Err(reason) => CardsNamedForThisMachine::refused(reason),
        };
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
                    GpuCardPoolStartOutcome::Started(Box::new(open_the_card_pool(named.cards)))
                } else {
                    // 点名的卡少一张都不跑（配了 GPU 就要它干活，不行就明确报错，不接受少用几张的默认降级）
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
    /// 设备等借到那张卡时才起），之后每条用例拿同一个。本机没有 NVIDIA 卡时每条来要的用例各交回未跑与一行
    /// [`not_run_line`]（不打，打不打由卡池把手的调用方定：[`super::GpuCardPoolStartedAtTheFirstLease::lease_card_or_report_not_run`] 打）。
    /// 先判有没有卡、建好表知道要多少显存之后再借（[`GpuCardPool::lease_card_or_report_not_run`]）。
    ///
    /// # Panics
    /// 有卡而用不上（同 [`super::usable_cards_or_report_not_run`]）；显卡配置写错、没给这台机器点卡、点名的卡少一张找不到；
    /// [`CARD_QUOTA_ENVIRONMENT_VARIABLE`] 设了而认不出。
    #[must_use]
    pub fn gpu_card_pool_or_the_not_run_line(
        test_name: &str,
    ) -> TakenOrNotRun<&'static GpuCardPool> {
        let start = GPU_CARD_POOL.get_or_init(start_the_gpu_card_pool);
        match &start.outcome {
            GpuCardPoolStartOutcome::Started(pool) => TakenOrNotRun::Taken(&**pool),
            GpuCardPoolStartOutcome::NotRunNoGpuAdapter { reason } => TakenOrNotRun::NotRun {
                line: not_run_line(test_name, reason),
            },
            GpuCardPoolStartOutcome::CardsPresentButNotUsable { reason } => {
                panic!(
                    "{test_name}：{reason}（卡池照调用方给这个进程的显卡配置挑的卡：{:?}）",
                    start.configuration
                )
            }
        }
    }

    /// 这个进程的卡池，没起过就当场起（与 [`gpu_card_pool_or_the_not_run_line`] 起的是同一个，进程里只起一次）：不交「本次未跑」那一行、
    /// 不 panic，起好交池，判成本机没卡交 `Ok(None)`。崩溃放量取卡经它
    /// （`crate::crash_amplification::gpu_cards_in_use`：同一个进程里经引擎起步的判与直接借卡的 GPU 用例走同一套显存账）。
    ///
    /// # Errors
    /// 有卡而用不上（显卡配置写错、没给这台机器点卡、点名的卡少一张找不到）：交的那一句同 [`gpu_card_pool_or_the_not_run_line`] panic 的那一句。
    pub fn card_pool_of_this_process_started_now() -> Result<Option<&'static GpuCardPool>, String> {
        let start = GPU_CARD_POOL.get_or_init(start_the_gpu_card_pool);
        match &start.outcome {
            GpuCardPoolStartOutcome::Started(pool) => Ok(Some(&**pool)),
            GpuCardPoolStartOutcome::NotRunNoGpuAdapter { .. } => Ok(None),
            GpuCardPoolStartOutcome::CardsPresentButNotUsable { reason } => Err(format!(
                "{reason}（卡池照调用方给这个进程的显卡配置挑的卡：{:?}）",
                start.configuration
            )),
        }
    }

    /// 这个进程起池判成没卡时的原因（[`gpu_card_pool_or_the_not_run_line`] 交回的「本次未跑」那一行的原因）；还没起过池、起好了、
    /// 有卡而用不上，交 `None`（不起池，什么都不打）。
    #[must_use]
    pub fn card_pool_not_run_reason_in_this_process() -> Option<&'static str> {
        match GPU_CARD_POOL.get().map(|start| &start.outcome) {
            Some(GpuCardPoolStartOutcome::NotRunNoGpuAdapter { reason }) => Some(reason.as_str()),
            Some(
                GpuCardPoolStartOutcome::Started(_)
                | GpuCardPoolStartOutcome::CardsPresentButNotUsable { .. },
            )
            | None => None,
        }
    }

    /// 这个进程已经起好的卡池；还没起过池、或起池判成没卡、有卡而用不上，交 `None`（不起池，什么都不打）。
    #[must_use]
    pub fn card_pool_started_in_this_process() -> Option<&'static GpuCardPool> {
        match GPU_CARD_POOL.get().map(|start| &start.outcome) {
            Some(GpuCardPoolStartOutcome::Started(pool)) => Some(&**pool),
            Some(
                GpuCardPoolStartOutcome::NotRunNoGpuAdapter { .. }
                | GpuCardPoolStartOutcome::CardsPresentButNotUsable { .. },
            )
            | None => None,
        }
    }
}

/// 代码三方 test-script-align-code-r2 判决 J2 G1：看不见 PCI 总线（总线目录整个不在）算没卡、报本次未跑；
/// 总线看得见而设备表列不出、某个设备读不出照旧判红。判定是纯函数，读 PCI 用假的 sysfs 树，不要显卡。
#[cfg(test)]
mod tests;
