//! 本包真卡测试目标共用：服务经 `GPU_CARDS_BY_PRIORITY` 交给这个进程、属于这一台的那几张卡（次序照交来的次序），对上本机的 Vulkan 独显，
//! 与给每张参与的卡的显存上限。
//! 卡由服务分（`task-server/README.md`「源文件头的声明」：测试源头几行写 `// task-server-gpu-cards: <张数>`，交来的卡在环境变量 `GPU_CARDS_BY_PRIORITY`，
//! 每项 `<HOSTS 里的地址>:<nvidia-smi 序号>:<额度>`，空白隔开）：真卡用例只点名交来的这几张（挂载参数 `background_gpu_devices`），不再点本机全部独显——
//! 同一台上别的卡可能分给了别的任务。这一台是 `HOSTS` 里哪一项只由多机配置判法认（`scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses`
//! 打的 `THIS_HOST_ADDRESS`，全仓只那一份判法）；`nvidia-smi` 序号对到 wgpu 的适配器按 PCI 总线地址对（去掉域、小写，
//! 同 checker 档 `gpu_unit_checks` 文件头「按 PCI 总线号对上 wgpu 的适配器」的对法；harness 一侧不依赖 checker 档，这里自己写一份）。
//! 变量没设、或这一台一张都没交：打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」；交了卡却对不上本机任何适配器、或交来的比这一份要的少：判红。
//! 七·3 验收「真卡用例每张参与的卡显存上限给 1 GiB（测小上限切片的几条照用更小的）」（用户 2026-11-01 定）：每张参与的卡各给
//! [`MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU`]；专门测小上限切片的那几条照用各自更小的值。
//! 解析与对卡是纯函数（单测在 `tests/real_gpu_tests_open_only_the_gpus_the_task_server_handed_over.rs`，不挂特性、不要卡），读环境、起判法、起 `nvidia-smi`、
//! 枚举适配器的那一层薄壳在 [`handed_over_gpus_or_report_not_run`]。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use singlefs_background_gpu::adapter_inventory::{
    gpu_device_identifier_of_the_pci_bus_address,
    identifiers_of_the_discrete_gpus_in_the_enumeration, AdapterDeviceKind, BackgroundGpuSelection,
    EnumeratedAdapterDescription,
};
use singlefs_background_gpu::background_gpu::{
    open_background_gpu, wait_for_the_wgpu_future, BackgroundGpu,
};
use singlefs_background_gpu::waiting_for_the_gpu::SteadyClockOfThisProcess;
use singlefs_core::background_compute::{GpuDeviceIdentifier, GpuMemoryInBytes};

/// 真卡用例给每张参与的卡的显存上限：1 GiB。
pub const MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU: GpuMemoryInBytes = GpuMemoryInBytes(1 << 30);

/// 服务把分给这条任务的卡交给命令的环境变量（`task-server/task_server.py` 文件头 ②「GPU」）。
pub const VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER: &str = "GPU_CARDS_BY_PRIORITY";

/// 服务读张数声明只读源文件头这么多行（`task-server/task_server_lib.py` 的 `DECLARED_HEADER_LINES`）。
pub const LINES_OF_THE_SOURCE_HEADER_THE_TASK_SERVER_READS: usize = 60;

/// 测试源头声明要同一台几张卡的那一行的开头（`task-server/task_server_lib.py` 的 `DECLARED_SOURCE_GPU_CARDS_MARK`）。
pub const GPU_CARD_COUNT_DECLARATION_MARK: &str = "// task-server-gpu-cards:";

/// 一张卡在 `nvidia-smi` 里的序号（`GPU_CARDS_BY_PRIORITY` 每项中间那一段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NvidiaSmiIndex(pub u32);

/// `GPU_CARDS_BY_PRIORITY` 的一项：在 `HOSTS` 里哪个地址那一台、`nvidia-smi` 序号。额度这里不用（显存上限照七·3 验收另给）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuHandedOverByTheTaskServer {
    pub host_address: String,
    pub nvidia_smi_index: NvidiaSmiIndex,
}

/// `GPU_CARDS_BY_PRIORITY` 读不成。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuHandoverUnreadable {
    /// 一项从右边切两刀切不出地址、序号、额度三段（或其中一段是空的）。
    EntryWithoutAddressIndexAndQuota { entry: String },
    /// 序号那一段不是数。
    IndexNotANumber { entry: String },
    /// 同一台的同一张卡交了两次。
    SameGpuHandedOverTwice { entry: String },
}

/// 读 `GPU_CARDS_BY_PRIORITY` 的值：空白隔开的若干项，每项 `<HOSTS 里的地址>:<nvidia-smi 序号>:<额度>`，从右边切两刀（地址可以是 IPv6、自带冒号）。
/// 次序照写的次序。值是空白的交回空表。
///
/// # Errors
/// [`GpuHandoverUnreadable`] 的三种。
pub fn gpus_handed_over_by_the_task_server(
    handover_text: &str,
) -> Result<Vec<GpuHandedOverByTheTaskServer>, GpuHandoverUnreadable> {
    let mut handed_over: Vec<GpuHandedOverByTheTaskServer> = Vec::new();
    for entry in handover_text.split_whitespace() {
        let without_three_fields = || GpuHandoverUnreadable::EntryWithoutAddressIndexAndQuota {
            entry: entry.to_string(),
        };
        let (address_and_index, quota) = entry.rsplit_once(':').ok_or_else(without_three_fields)?;
        let (host_address, index_text) = address_and_index
            .rsplit_once(':')
            .ok_or_else(without_three_fields)?;
        if host_address.is_empty() || index_text.is_empty() || quota.is_empty() {
            return Err(without_three_fields());
        }
        let nvidia_smi_index = NvidiaSmiIndex(index_text.parse().map_err(|_| {
            GpuHandoverUnreadable::IndexNotANumber {
                entry: entry.to_string(),
            }
        })?);
        let gpu = GpuHandedOverByTheTaskServer {
            host_address: host_address.to_string(),
            nvidia_smi_index,
        };
        if handed_over.contains(&gpu) {
            return Err(GpuHandoverUnreadable::SameGpuHandedOverTwice {
                entry: entry.to_string(),
            });
        }
        handed_over.push(gpu);
    }
    Ok(handed_over)
}

/// 交给这一台的卡：要么这一次不跑（服务没交卡、或交的卡里没有这一台的，带一句原因），要么这一台那几张的序号（照交来的次序，至少一张）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpusHandedToThisHost {
    NotRun { reason: String },
    Handed(Vec<NvidiaSmiIndex>),
}

/// 照 `GPU_CARDS_BY_PRIORITY` 的值（没设是 `None`）挑这一台的卡。变量没设或是空白的不问这一台是谁（`this_host_address` 不调），交回不跑；
/// 交了卡的才调 `this_host_address`（多机配置判法打的 `THIS_HOST_ADDRESS`）取这一台的地址，只留地址逐字相同的那几项。
///
/// # Errors
/// 值读不成（[`gpus_handed_over_by_the_task_server`] 的三种）；取不到这一台的地址（`this_host_address` 交回的原因）。
pub fn gpus_handed_to_this_host(
    handover_variable_value: Option<&str>,
    this_host_address: impl FnOnce() -> Result<String, String>,
) -> Result<GpusHandedToThisHost, String> {
    let handover_text = match handover_variable_value {
        Some(handover_text) if !handover_text.trim().is_empty() => handover_text,
        Some(_) | None => {
            return Ok(GpusHandedToThisHost::NotRun {
                reason: format!(
                    "服务没经 {VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER} 交卡（没设或是空的）"
                ),
            })
        }
    };
    let handed_over = gpus_handed_over_by_the_task_server(handover_text).map_err(|unreadable| {
        format!(
            "{VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER}=「{handover_text}」读不成：{unreadable:?}"
        )
    })?;
    let this_host = this_host_address()?;
    let indices_of_this_host: Vec<NvidiaSmiIndex> = handed_over
        .iter()
        .filter(|gpu| gpu.host_address == this_host)
        .map(|gpu| gpu.nvidia_smi_index)
        .collect();
    if indices_of_this_host.is_empty() {
        return Ok(GpusHandedToThisHost::NotRun {
            reason: format!(
                "服务交的卡（{VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER}=「{handover_text}」）里没有这一台（HOSTS 里的 {this_host}）的"
            ),
        });
    }
    Ok(GpusHandedToThisHost::Handed(indices_of_this_host))
}

/// bash `printf %q` 打出来的一个地址还原成原样：判法只打地址（十六进制数字、点、冒号）与隔开它们的空格，`%q` 只给空格前面加反斜杠，空串打成 `''`；
/// 别的引号写法判法不会打给这几行，认不出就报错，不猜（同 checker 档 `gpu_card_configuration` 的同名读法）。
fn value_printed_by_printf_quoted(quoted: &str) -> Result<String, String> {
    if quoted == "''" {
        return Ok(String::new());
    }
    if quoted.starts_with('\'') || quoted.starts_with("$'") {
        return Err(format!("「{quoted}」的引号写法不是判法给地址打的那一种"));
    }
    let mut value = String::with_capacity(quoted.len());
    let mut characters = quoted.chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            value.push(
                characters
                    .next()
                    .ok_or_else(|| format!("「{quoted}」结尾多了一个反斜杠"))?,
            );
        } else {
            value.push(character);
        }
    }
    Ok(value)
}

/// 多机配置判法 `--emit-addresses` 打的那几行（`KEY=<printf %q 的值>`）里这一台的地址（`THIS_HOST_ADDRESS` 那一行；`PEER_HOST_ADDRESS` 不用）。
///
/// # Errors
/// 没打那一行、值认不出、值是空的。
pub fn this_host_address_emitted_by_the_configuration_check(
    printed: &str,
) -> Result<String, String> {
    let quoted = printed
        .lines()
        .find_map(|line| line.strip_prefix("THIS_HOST_ADDRESS="))
        .ok_or_else(|| {
            format!("多机配置判法没打 THIS_HOST_ADDRESS 那一行（打的是「{printed}」）")
        })?;
    let address = value_printed_by_printf_quoted(quoted)?;
    if address.is_empty() {
        return Err("多机配置判法打的 THIS_HOST_ADDRESS 是空的".to_string());
    }
    Ok(address)
}

/// PCI 总线地址只留 `总线:设备.功能` 那一段、小写：`nvidia-smi` 报的 `00000000:01:00.0` 与 wgpu 报的 `0000:01:00.0` 认成同一张卡
/// （同 harness 档 `gpu_memory_ledger` 的同名函数；本包测试不依赖 harness 档，自己留一份）。
#[must_use]
pub fn pci_bus_identifier_without_the_domain(pci_bus_address: &str) -> String {
    let lowered = pci_bus_address.trim().to_ascii_lowercase();
    let parts: Vec<&str> = lowered.split(':').collect();
    parts[parts.len().saturating_sub(2)..].join(":")
}

/// 读卡号与总线号要的 `nvidia-smi` 参数：每张卡一行「序号, 总线地址」。
pub const NVIDIA_SMI_INDEX_AND_PCI_BUS_ARGUMENTS: [&str; 2] =
    ["--query-gpu=index,pci.bus_id", "--format=csv,noheader"];

/// 一份 `nvidia-smi --query-gpu=index,pci.bus_id --format=csv,noheader` 的输出读成「序号 → 去掉域的总线地址」（[`pci_bus_identifier_without_the_domain`]）。
///
/// # Errors
/// 一行切不出两段、序号不是数、同一个序号报了两次。
pub fn pci_buses_by_nvidia_smi_index(
    report: &str,
) -> Result<BTreeMap<NvidiaSmiIndex, String>, String> {
    let mut buses = BTreeMap::new();
    for line in report.lines().filter(|line| !line.trim().is_empty()) {
        let (index_text, pci_bus_address) = line
            .split_once(',')
            .ok_or_else(|| format!("nvidia-smi 的一行「{line}」切不出序号与总线地址"))?;
        let nvidia_smi_index = NvidiaSmiIndex(
            index_text
                .trim()
                .parse()
                .map_err(|_| format!("nvidia-smi 的一行「{line}」序号不是数"))?,
        );
        if buses
            .insert(
                nvidia_smi_index,
                pci_bus_identifier_without_the_domain(pci_bus_address),
            )
            .is_some()
        {
            return Err(format!("nvidia-smi 把序号 {} 报了两次", nvidia_smi_index.0));
        }
    }
    Ok(buses)
}

/// 交来的一张卡对上的本机 Vulkan 独显：`nvidia-smi` 序号与 wgpu 报的总线地址原样（挂载参数 `background_gpu_devices` 点的就是这个串）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandedOverGpu {
    pub nvidia_smi_index: NvidiaSmiIndex,
    pub pci_bus_address: String,
}

impl HandedOverGpu {
    /// 挂载参数 `background_gpu_devices` 里点这张卡的标识。
    #[must_use]
    pub fn identifier(&self) -> GpuDeviceIdentifier {
        gpu_device_identifier_of_the_pci_bus_address(&self.pci_bus_address)
    }
}

/// 交来的一张卡对不上本机的独显。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HandedOverGpuNotMatched {
    /// `nvidia-smi` 没报这个序号。
    NotReportedByNvidiaSmi { nvidia_smi_index: NvidiaSmiIndex },
    /// 这个序号的总线上没有 Vulkan 独显。
    NoDiscreteAdapterOnTheBus {
        nvidia_smi_index: NvidiaSmiIndex,
        pci_bus: String,
    },
    /// 这个序号的总线上有几个 Vulkan 独显（同一张卡经两份驱动各枚举一次）：点哪一个不定，判红。
    SeveralDiscreteAdaptersOnTheBus {
        nvidia_smi_index: NvidiaSmiIndex,
        pci_bus: String,
    },
}

/// 交来的卡（`nvidia-smi` 序号，照交来的次序）逐张对上本机的 Vulkan 独显（`pci_bus_addresses_of_the_discrete_adapters`，wgpu 报的原样）：
/// 序号经 `nvidia-smi` 报的总线地址、去掉域之后与适配器的总线地址去掉域之后相同，恰好一个。交回的次序照交来的次序。
///
/// # Errors
/// 有一张对不上（[`HandedOverGpuNotMatched`] 的三种）：不跳过，判红。
pub fn handed_over_gpus_matched_to_the_adapters(
    handed_over_indices: &[NvidiaSmiIndex],
    pci_buses_by_index: &BTreeMap<NvidiaSmiIndex, String>,
    pci_bus_addresses_of_the_discrete_adapters: &[String],
) -> Result<Vec<HandedOverGpu>, HandedOverGpuNotMatched> {
    handed_over_indices
        .iter()
        .map(|nvidia_smi_index| {
            let pci_bus = pci_buses_by_index.get(nvidia_smi_index).ok_or(
                HandedOverGpuNotMatched::NotReportedByNvidiaSmi {
                    nvidia_smi_index: *nvidia_smi_index,
                },
            )?;
            let adapters_on_the_bus: Vec<&String> = pci_bus_addresses_of_the_discrete_adapters
                .iter()
                .filter(|pci_bus_address| {
                    pci_bus_identifier_without_the_domain(pci_bus_address) == *pci_bus
                })
                .collect();
            match adapters_on_the_bus.as_slice() {
                [pci_bus_address] => Ok(HandedOverGpu {
                    nvidia_smi_index: *nvidia_smi_index,
                    pci_bus_address: (*pci_bus_address).clone(),
                }),
                [] => Err(HandedOverGpuNotMatched::NoDiscreteAdapterOnTheBus {
                    nvidia_smi_index: *nvidia_smi_index,
                    pci_bus: pci_bus.clone(),
                }),
                [_, _, ..] => Err(HandedOverGpuNotMatched::SeveralDiscreteAdaptersOnTheBus {
                    nvidia_smi_index: *nvidia_smi_index,
                    pci_bus: pci_bus.clone(),
                }),
            }
        })
        .collect()
}

/// 交来的卡里照交来的次序取头 `declared_gpu_count` 张（测试源头 `// task-server-gpu-cards:` 声明的张数）；多交的不用。
///
/// # Errors
/// 交来的比声明的少：判红，不降级成少几张跑。
pub fn handed_over_gpus_of_the_declared_count(
    handed_over: Vec<HandedOverGpu>,
    declared_gpu_count: usize,
) -> Result<Vec<HandedOverGpu>, String> {
    if handed_over.len() < declared_gpu_count {
        return Err(format!(
            "服务交给这一台 {} 张卡（{handed_over:?}），这一份源头声明要 {declared_gpu_count} 张：判红，不降级成少几张跑",
            handed_over.len()
        ));
    }
    Ok(handed_over.into_iter().take(declared_gpu_count).collect())
}

/// 一份测试源头 [`LINES_OF_THE_SOURCE_HEADER_THE_TASK_SERVER_READS`] 行里 [`GPU_CARD_COUNT_DECLARATION_MARK`] 那一行声明的张数；没写是 `None`，
/// 写的不是一个数也是 `None`（服务照写坏办）。
#[must_use]
pub fn gpu_card_count_declared_in_the_source_header(source: &str) -> Option<usize> {
    source
        .lines()
        .take(LINES_OF_THE_SOURCE_HEADER_THE_TASK_SERVER_READS)
        .find_map(|line| line.trim().strip_prefix(GPU_CARD_COUNT_DECLARATION_MARK))
        .and_then(|declared| declared.trim().parse().ok())
}

/// 照交来的卡开卡的那一组：点名这几张（照交来的次序）、每张给 `memory_limit_of_each_gpu`、等 GPU 的上限 `wait_limit`。
#[must_use]
pub fn selection_over_the_handed_over_gpus(
    handed_over_gpus: &[HandedOverGpu],
    memory_limit_of_each_gpu: GpuMemoryInBytes,
    wait_limit: Duration,
) -> BackgroundGpuSelection {
    BackgroundGpuSelection {
        background_gpu_devices: Some(identifiers_of_the_handed_over_gpus(handed_over_gpus)),
        background_gpu_memory_limit: Some(vec![memory_limit_of_each_gpu; handed_over_gpus.len()]),
        background_gpu_wait_limit: wait_limit,
    }
}

/// 交来的卡各自的标识，照交来的次序。
#[must_use]
pub fn identifiers_of_the_handed_over_gpus(
    handed_over_gpus: &[HandedOverGpu],
) -> Vec<GpuDeviceIdentifier> {
    handed_over_gpus
        .iter()
        .map(HandedOverGpu::identifier)
        .collect()
}

/// 照 `selection` 开卡。卡是服务交来、对上了本机适配器的，开不了判红。
///
/// # Panics
/// 开卡拒了。
#[must_use]
pub fn opened_background_gpu(selection: &BackgroundGpuSelection) -> BackgroundGpu {
    open_background_gpu(selection).unwrap_or_else(|refused| {
        panic!("服务交来、对上了本机适配器的卡开不了后台 GPU：{refused:?}（点名的 {selection:?}）")
    })
}

/// 本机这一次枚举到的适配器（另起一个只开 Vulkan 的 wgpu 实例，与产品开卡同一种实例），在 `wait_limit` 之内等。
///
/// # Panics
/// 等枚举超过 `wait_limit`。
fn adapter_descriptions_on_this_host(wait_limit: Duration) -> Vec<EnumeratedAdapterDescription> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    wait_for_the_wgpu_future(
        instance.enumerate_adapters(wgpu::Backends::VULKAN),
        wait_limit,
        &SteadyClockOfThisProcess::starting_now(),
    )
    .expect("枚举适配器在等的上限之内")
    .iter()
    .map(|adapter| {
        let adapter_information = adapter.get_info();
        EnumeratedAdapterDescription {
            name: adapter_information.name,
            device_kind: AdapterDeviceKind::of_the_wgpu_device_type(
                adapter_information.device_type,
            ),
            pci_bus_address: adapter_information.device_pci_bus_id,
            device_local_memory: None,
        }
    })
    .collect()
}

/// 此刻本机在的独立显卡，按 wgpu 枚举的次序（照产品那一份判法数：集显、虚拟显卡、软件 Vulkan、认不出类型的与没有总线地址的不算，
/// `adapter_inventory::identifiers_of_the_discrete_gpus_in_the_enumeration`）。只给「Vulkan 驱动全藏起来时一张都看不到」那一条数卡用，开卡点名交来的卡。
///
/// # Panics
/// 等枚举超过 `wait_limit`。
#[must_use]
pub fn discrete_gpus_on_this_host(wait_limit: Duration) -> Vec<GpuDeviceIdentifier> {
    identifiers_of_the_discrete_gpus_in_the_enumeration(&adapter_descriptions_on_this_host(
        wait_limit,
    ))
}

/// 此刻本机 Vulkan 独显各自的总线地址（wgpu 报的原样），哪些算独显照产品那一份判法（[`discrete_gpus_on_this_host`]）。
fn pci_bus_addresses_of_the_discrete_gpus_on_this_host(wait_limit: Duration) -> Vec<String> {
    let descriptions = adapter_descriptions_on_this_host(wait_limit);
    let discrete_gpus = identifiers_of_the_discrete_gpus_in_the_enumeration(&descriptions);
    descriptions
        .into_iter()
        .map(|description| description.pci_bus_address)
        .filter(|pci_bus_address| {
            discrete_gpus.contains(&gpu_device_identifier_of_the_pci_bus_address(
                pci_bus_address,
            ))
        })
        .collect()
}

/// 起多机配置判法 `bash <仓根>/scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses`（仓根是编这份测试的那棵树，判法自己照
/// `SINGLEFS_MULTI_HOST_CONFIG`、没设照主工作树找多机配置），读它打的这一台的地址。
fn this_host_address_by_the_configuration_check() -> Result<String, String> {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/crash/crash-point-replay-shard-configuration-check.sh");
    let output = std::process::Command::new("bash")
        .arg(&script)
        .arg("--emit-addresses")
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|error| {
            format!(
                "起不了多机配置判法 bash {} --emit-addresses：{error}（这一台是 HOSTS 里哪一项只由它认）",
                script.display()
            )
        })?;
    let printed = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        return Err(format!(
            "多机配置判法认不出这一台（bash {} --emit-addresses 退 {:?}）：{}{}",
            script.display(),
            output.status.code(),
            printed.trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    this_host_address_emitted_by_the_configuration_check(&printed)
}

/// 跑一次 `nvidia-smi`（[`NVIDIA_SMI_INDEX_AND_PCI_BUS_ARGUMENTS`]）读每张卡的序号与总线地址。
fn pci_buses_reported_by_nvidia_smi() -> Result<BTreeMap<NvidiaSmiIndex, String>, String> {
    let output = std::process::Command::new("nvidia-smi")
        .args(NVIDIA_SMI_INDEX_AND_PCI_BUS_ARGUMENTS)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|error| format!("nvidia-smi 起不来：{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "nvidia-smi {NVIDIA_SMI_INDEX_AND_PCI_BUS_ARGUMENTS:?} 退了 {}：{}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    pci_buses_by_nvidia_smi_index(&String::from_utf8_lossy(&output.stdout))
}

/// 服务交给这个进程、属于这一台的卡，逐张对上本机的 Vulkan 独显，照交来的次序。服务没交卡、或交的卡里没有这一台的：打
/// `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」、交回 `None`。
///
/// # Panics
/// 变量读不成、多机配置判法认不出这一台、`nvidia-smi` 起不来或读不成、交来的卡有一张对不上本机的独显：判红。
#[must_use]
pub fn handed_over_gpus_or_report_not_run(
    test_name: &str,
    wait_limit: Duration,
) -> Option<Vec<HandedOverGpu>> {
    let handover_variable_value =
        std::env::var(VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER).ok();
    let handed_to_this_host = gpus_handed_to_this_host(
        handover_variable_value.as_deref(),
        this_host_address_by_the_configuration_check,
    )
    .unwrap_or_else(|reason| panic!("服务交的卡认不成：{reason}"));
    let handed_over_indices = match handed_to_this_host {
        GpusHandedToThisHost::NotRun { reason } => {
            println!("NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}");
            return None;
        }
        GpusHandedToThisHost::Handed(handed_over_indices) => handed_over_indices,
    };
    let pci_buses_by_index = pci_buses_reported_by_nvidia_smi().unwrap_or_else(|reason| {
        panic!("服务交了卡（序号 {handed_over_indices:?}），读不成本机的卡：{reason}")
    });
    let pci_bus_addresses_of_the_discrete_adapters =
        pci_bus_addresses_of_the_discrete_gpus_on_this_host(wait_limit);
    let handed_over_gpus = handed_over_gpus_matched_to_the_adapters(
        &handed_over_indices,
        &pci_buses_by_index,
        &pci_bus_addresses_of_the_discrete_adapters,
    )
    .unwrap_or_else(|not_matched| {
        panic!(
            "服务交的卡对不上本机的 Vulkan 独显：{not_matched:?}（nvidia-smi 报的 {pci_buses_by_index:?}，本机独显 {pci_bus_addresses_of_the_discrete_adapters:?}）"
        )
    });
    println!(
        "GPUS_HANDED_OVER_BY_THE_TASK_SERVER test={test_name} gpus={}",
        handed_over_gpus
            .iter()
            .map(|gpu| format!("{}@{}", gpu.nvidia_smi_index.0, gpu.pci_bus_address))
            .collect::<Vec<_>>()
            .join(",")
    );
    Some(handed_over_gpus)
}

/// 同 [`handed_over_gpus_or_report_not_run`]，再照交来的次序取头 `declared_gpu_count` 张（这一份源头声明的张数，[`handed_over_gpus_of_the_declared_count`]）。
///
/// # Panics
/// 同 [`handed_over_gpus_or_report_not_run`]；交来的比声明的少。
#[must_use]
pub fn handed_over_gpus_of_the_declared_count_or_report_not_run(
    test_name: &str,
    declared_gpu_count: usize,
    wait_limit: Duration,
) -> Option<Vec<HandedOverGpu>> {
    let handed_over_gpus = handed_over_gpus_or_report_not_run(test_name, wait_limit)?;
    Some(
        handed_over_gpus_of_the_declared_count(handed_over_gpus, declared_gpu_count)
            .unwrap_or_else(|reason| panic!("{reason}")),
    )
}

/// 开交来的卡、每张卡的显存上限 [`MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU`]、等 GPU 的上限 `wait_limit`；服务没交这一台的卡时打
/// `NOT_RUN_NO_GPU_ADAPTER` 那一行、交回 `None`。
///
/// # Panics
/// 同 [`handed_over_gpus_or_report_not_run`]；开卡拒了。
#[must_use]
pub fn background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run(
    test_name: &str,
    wait_limit: Duration,
) -> Option<BackgroundGpu> {
    let handed_over_gpus = handed_over_gpus_or_report_not_run(test_name, wait_limit)?;
    Some(opened_background_gpu(&selection_over_the_handed_over_gpus(
        &handed_over_gpus,
        MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU,
        wait_limit,
    )))
}

/// `card_count` 张参与的卡各一个 [`MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU`]（挂载参数 `background_gpu_memory_limit` 按次序一张卡一个、不补齐）。
#[must_use]
pub fn memory_limits_of_one_gibibyte_each(card_count: usize) -> Vec<GpuMemoryInBytes> {
    vec![MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU; card_count]
}
